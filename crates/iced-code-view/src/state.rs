use crate::font_lock;
use crate::gutter::engine as gutter_engine;
use crate::gutter::{
  GutterCacheEntry, GutterMeasureRequest, GutterMetrics, GutterRenderKey, GutterRenderRequest,
  GutterRowsSignature,
};
use crate::layout::{LayoutKey, LayoutRequest};
use crate::layout_cache::LayoutCacheEntry;
use crate::layout_engine;
use crate::measurement::{MeasurementKey, MeasurementRequest, MeasurementResult};
use crate::scroll::ScrollExtent;
use crate::viewport::Viewport;

#[derive(Default)]
pub(crate) struct CodeViewState {
  pub(crate) layout: LayoutState,
  pub(crate) gutter: GutterState,
  pub(crate) viewport: Viewport,
  pub(crate) scroll_extent: ScrollExtent,
  pub(crate) pending_measurement_request: Option<MeasurementRequest>,
  pub(crate) last_published_measurement_key: Option<MeasurementKey>,
}

#[derive(Default)]
pub(crate) struct LayoutState {
  entry: Option<LayoutCacheEntry>,
}

impl LayoutState {
  pub(crate) fn refresh(&mut self, request: LayoutRequest<'_>) {
    let key = LayoutKey::from_request(&request, font_lock::font_system_version());
    let prev = self.entry.take();

    self.entry = match prev {
      Some(mut prev) if prev.key == key => {
        layout_engine::scroll_to(&mut prev, &request);
        Some(prev)
      }
      prev => Some(layout_engine::rebuild_layout(request, key, prev)),
    }
  }

  pub(crate) fn entry(&self) -> Option<&LayoutCacheEntry> {
    self.entry.as_ref()
  }
}

#[derive(Default)]
pub(crate) struct GutterState {
  entry: Option<GutterCacheEntry>,
}

impl GutterState {
  pub(crate) fn measure(&mut self, request: GutterMeasureRequest<'_>) -> GutterMetrics {
    let prev = self.entry.take();
    let (metrics, entry) = gutter_engine::measure_gutter(request, prev);

    self.entry = entry;

    metrics
  }

  pub(crate) fn refresh(&mut self, render_request: GutterRenderRequest<'_>) {
    let Some(entry) = self.entry.as_mut() else {
      return;
    };

    let Some(key) = GutterRenderKey::for_render_request(&render_request, entry.width_key) else {
      entry.render_artifact = None;
      return;
    };

    let projection = render_request.projection;

    entry.render_artifact = match entry.render_artifact.take() {
      Some(mut render) if render.key == key => {
        gutter_engine::sync_render_origin(&mut render, projection);
        Some(render)
      }
      prev => Some(gutter_engine::rebuild_render(render_request, key, prev)),
    };

    debug_assert_eq!(
      GutterRowsSignature::from_projection(projection).as_ref(),
      entry
        .render_artifact
        .as_ref()
        .map(|artifact| &artifact.key.rows_signature),
    );
  }

  pub(crate) fn entry(&self) -> Option<&GutterCacheEntry> {
    self.entry.as_ref()
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

  pub(crate) fn try_apply_wheel_delta(&mut self, delta: iced::Vector) -> bool {
    let old = self.viewport.scroll_offset;
    let candidate = iced::Vector::new(old.x - delta.x, old.y - delta.y);

    let scroll_offset = self
      .scroll_extent
      .clamp_offset(candidate, self.viewport.scroll_viewport_size());

    if scroll_offset == old {
      return false;
    }

    self.viewport.scroll_offset = scroll_offset;
    true
  }
}
