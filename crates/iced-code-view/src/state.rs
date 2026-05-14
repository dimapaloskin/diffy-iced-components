use crate::font_lock;
use crate::gutter::engine as gutter_engine;
use crate::gutter::{
  GutterMetrics, GutterMetricsRequest, GutterRenderArtifactKey, GutterRenderArtifactRequest,
  GutterRowsSignature, MeasuredGutter,
};
use crate::measurement::{MeasurementKey, MeasurementRequest, MeasurementResult};
use crate::scroll::{ScrollExtent, ScrollState};
use crate::text_layout::VisibleTextLayout;
use crate::text_layout::engine as text_layout_engine;
use crate::text_layout::{TextLayoutKey, TextLayoutRequest};
use crate::viewport::Viewport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollChange {
  RedrawOnly,
  RequiresLayout,
}

#[derive(Default)]
pub(crate) struct CodeViewState {
  pub(crate) text_layout: TextLayoutState,
  pub(crate) gutter: GutterState,
  pub(crate) scroll: ScrollState,
  pub(crate) viewport: Viewport,
  pub(crate) scroll_extent: ScrollExtent,
  pub(crate) pending_measurement_request: Option<MeasurementRequest>,
  pub(crate) last_published_measurement_key: Option<MeasurementKey>,
}

#[derive(Default)]
pub(crate) struct TextLayoutState {
  visible_layout: Option<VisibleTextLayout>,
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

impl CodeViewState {
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

  pub(crate) fn try_apply_wheel_delta(&mut self, delta: iced::Vector) -> Option<ScrollChange> {
    if !delta.x.is_finite() || !delta.y.is_finite() {
      return None;
    }

    let horizontal_px = self.scroll_extent.horizontal.clamp_offset(
      self.scroll.horizontal_px - delta.x,
      self.viewport.scroll_viewport_size().width,
    );
    // iced wheel delta is positive when scrolling up.
    // cosmic Scroll is positive when scrolled down.
    let vertical = self.scroll.vertical.scrolled_by(-delta.y);

    let horizontal_changed = horizontal_px != self.scroll.horizontal_px;
    let vertical_changed = vertical != self.scroll.vertical;

    match (vertical_changed, horizontal_changed) {
      (false, false) => None,
      (false, true) => {
        self.scroll.horizontal_px = horizontal_px;
        Some(ScrollChange::RedrawOnly)
      }
      (true, _) => {
        self.scroll.vertical = vertical;
        self.scroll.horizontal_px = horizontal_px;
        Some(ScrollChange::RequiresLayout)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::scroll::{HorizontalExtent, VerticalScroll};

  #[test]
  fn wheel_up_at_document_top_is_noop() {
    let mut state = CodeViewState::default();

    let change = state.try_apply_wheel_delta(iced::Vector::new(0.0, 50.0));

    assert_eq!(change, None);
    assert_eq!(state.scroll.vertical, VerticalScroll::ZERO);
  }

  #[test]
  fn wheel_at_top_with_horizontal_delta_only_needs_redraw() {
    let mut state = CodeViewState::default();
    state.scroll_extent.horizontal = HorizontalExtent::Exact { content: 1000.0 };
    state.viewport.text.content_bounds.width = 300.0;

    let change = state.try_apply_wheel_delta(iced::Vector::new(-10.0, 30.0));

    assert_eq!(change, Some(ScrollChange::RedrawOnly));
    assert_eq!(state.scroll.vertical, VerticalScroll::ZERO);
    assert_eq!(state.scroll.horizontal_px, 10.0);
  }

  #[test]
  fn non_finite_wheel_delta_is_ignored() {
    let mut state = CodeViewState::default();
    state.scroll.vertical = VerticalScroll {
      source_line_index: 3,
      y_inside_source_line: 12.0,
    };
    let scroll = state.scroll;

    let change = state.try_apply_wheel_delta(iced::Vector::new(f32::NAN, 50.0));

    assert_eq!(change, None);
    assert_eq!(state.scroll, scroll);
  }
}
