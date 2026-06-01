use diffy_ui::widgets::scrollbar;

use crate::font_lock;
use crate::gutter::engine as gutter_engine;
use crate::gutter::{
  GutterMetrics, GutterMetricsRequest, GutterRenderArtifactKey, GutterRenderArtifactRequest,
  GutterRowsSignature, MeasuredGutter,
};
use crate::measurement::{MeasurementKey, MeasurementRequest, MeasurementResult};
use crate::scroll::ScrollModel;
use crate::text_layout::VisibleTextLayout;
use crate::text_layout::engine as text_layout_engine;
use crate::text_layout::{TextLayoutKey, TextLayoutRequest};
use crate::viewport::Viewport;

#[derive(Default)]
pub(crate) struct State {
  pub(crate) text_layout: TextLayoutState,
  pub(crate) gutter: GutterState,
  pub(crate) scroll: ScrollModel,
  pub(crate) scrollbars: ScrollbarStates,
  pub(crate) viewport: Viewport,
  pub(crate) pending_measurement_request: Option<MeasurementRequest>,
  pub(crate) last_published_measurement_key: Option<MeasurementKey>,
}

#[derive(Default)]
pub(crate) struct TextLayoutState {
  visible_layout: Option<VisibleTextLayout>,
}

#[derive(Default)]
pub(crate) struct ScrollbarStates {
  pub(crate) vertical: scrollbar::State,
  pub(crate) horizontal: scrollbar::State,
}

impl TextLayoutState {
  pub(crate) fn ensure_visible_layout(
    &mut self,
    request: TextLayoutRequest<'_>,
  ) -> &VisibleTextLayout {
    let key = TextLayoutKey::from_request(&request, font_lock::font_system_version());
    let prev = self.visible_layout.take();

    self.visible_layout = match prev {
      Some(mut prev) if prev.key == key => {
        text_layout_engine::sync_visible_viewport(&mut prev, &request);
        Some(prev)
      }
      prev => Some(text_layout_engine::rebuild_visible_layout(
        request, key, prev,
      )),
    };

    self
      .visible_layout
      .as_ref()
      .expect("visible text layout is ensured by TextLayoutState::ensure_visible_layout")
  }

  pub(crate) fn visible_layout(&self) -> Option<&VisibleTextLayout> {
    self.visible_layout.as_ref()
  }
}

#[derive(Default)]
pub(crate) struct GutterState {
  measured: Option<MeasuredGutter>,
}

impl GutterState {
  pub(crate) fn ensure_metrics(&mut self, request: GutterMetricsRequest<'_>) -> GutterMetrics {
    let prev = self.measured.take();
    let (metrics, measured) = gutter_engine::ensure_measured_gutter(request, prev);

    self.measured = measured;

    metrics
  }

  pub(crate) fn ensure_render_artifact(&mut self, render_request: GutterRenderArtifactRequest<'_>) {
    let Some(measured) = self.measured.as_mut() else {
      return;
    };

    let Some(key) = GutterRenderArtifactKey::for_request(&render_request, measured.metrics_key)
    else {
      measured.render_artifact = None;
      return;
    };

    let projection = render_request.projection;

    measured.render_artifact = match measured.render_artifact.take() {
      Some(mut render) if render.key == key => {
        gutter_engine::sync_render_artifact_origin(&mut render, projection);
        Some(render)
      }
      prev => Some(gutter_engine::rebuild_render_artifact(
        render_request,
        key,
        prev,
      )),
    };

    debug_assert_eq!(
      GutterRowsSignature::from_projection(projection).as_ref(),
      measured
        .render_artifact
        .as_ref()
        .map(|artifact| &artifact.key.rows_signature),
    );
  }

  pub(crate) fn measured(&self) -> Option<&MeasuredGutter> {
    self.measured.as_ref()
  }
}

impl State {
  pub(crate) fn update_pending_measurement<'a>(
    &mut self,
    request: MeasurementRequest,
    result: Option<&'a MeasurementResult>,
  ) -> Option<&'a MeasurementResult> {
    let fresh_result = result.filter(|result| result.key == request.key);

    if fresh_result.is_some() {
      self.pending_measurement_request = None;
    } else {
      self.pending_measurement_request = Some(request);
    }

    fresh_result
  }

  pub(crate) fn measurement_request_to_publish(&mut self) -> Option<MeasurementRequest> {
    let request = self.pending_measurement_request.as_ref()?;

    if self.last_published_measurement_key == Some(request.key) {
      return None;
    }

    self.last_published_measurement_key = Some(request.key);
    Some(request.clone())
  }
}
