use crate::layout::{LayoutKey, LayoutRequest};
use crate::layout_cache::LayoutCacheEntry;
use crate::layout_engine;
use crate::measurement::{MeasurementKey, MeasurementRequest, MeasurementResult};
use crate::scroll::ScrollExtent;
use crate::viewport::Viewport;

#[derive(Default)]
pub(crate) struct CodeViewState {
  pub(crate) layout_entry: Option<LayoutCacheEntry>,
  pub(crate) viewport: Viewport,
  pub(crate) scroll_extent: ScrollExtent,
  pub(crate) pending_measurement_request: Option<MeasurementRequest>,
  pub(crate) last_published_measurement_key: Option<MeasurementKey>,
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

  pub(crate) fn refresh_layout(&mut self, request: LayoutRequest<'_>) {
    let key = LayoutKey::from_request(&request);
    let prev = self.layout_entry.take();

    self.layout_entry = match prev {
      Some(mut prev) if prev.key == key => {
        if prev.prepared_scroll_offset != request.scroll_offset {
          layout_engine::sync_scroll(&mut prev, request.scroll_offset);
        }

        Some(prev)
      }
      prev => Some(layout_engine::rebuild_layout(request, key, prev)),
    }
  }

  pub(crate) fn try_apply_wheel_delta(&mut self, delta: iced::Vector) -> bool {
    let old = self.viewport.scroll_offset;
    let candidate = iced::Vector::new(old.x - delta.x, old.y - delta.y);

    let scroll_offset = self
      .scroll_extent
      .clamp_offset(candidate, self.viewport.content_bounds.size());

    if scroll_offset == old {
      return false;
    }

    self.viewport.scroll_offset = scroll_offset;

    if let Some(entry) = &mut self.layout_entry {
      layout_engine::sync_scroll(entry, scroll_offset);
    }

    true
  }
}
