use iced::advanced::graphics::text::{self, cosmic_text};

use crate::font_lock;
use crate::layout::{LayoutKey, LayoutRequest};
use crate::layout_cache::{CosmicBufferPayload, LayoutCacheEntry};
use crate::projection::LayoutProjection;
use crate::source_line::SourceLineHeights;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufferUpdateKind {
  FirstBuild,
  TextChanged,
  FontRuntimeChanged,
  FontChanged,
  LayoutOnly,
}

impl BufferUpdateKind {
  fn classify(prev_key: Option<LayoutKey>, new_key: LayoutKey) -> Self {
    let Some(prev_key) = prev_key else {
      return Self::FirstBuild;
    };

    if prev_key.document_revision != new_key.document_revision {
      return Self::TextChanged;
    }

    if prev_key.font_system_version != new_key.font_system_version {
      return Self::FontRuntimeChanged;
    }

    if prev_key.font != new_key.font {
      return Self::FontChanged;
    }

    Self::LayoutOnly
  }

  fn requires_set_text(self) -> bool {
    matches!(
      self,
      // Font changes could only update attrs.
      // For now reset text too because it is rare and avoids stale font caches.
      Self::FirstBuild | Self::TextChanged | Self::FontRuntimeChanged | Self::FontChanged
    )
  }
}

pub(crate) fn rebuild_layout(
  request: LayoutRequest,
  key: LayoutKey,
  prev: Option<LayoutCacheEntry>,
) -> LayoutCacheEntry {
  let mut font_system = font_lock::foreground_font_system_write();
  let raw_fs = font_system.raw();

  let prev_key = prev.as_ref().map(|entry| entry.key);
  let update_kind = BufferUpdateKind::classify(prev_key, key);

  let metrics = metrics_from_request(&request);
  let mut payload = take_or_create_payload(prev, raw_fs, metrics);

  let buffer = payload.buffer_mut();

  sync_buffer_config(buffer, &request, metrics);
  if update_kind.requires_set_text() {
    sync_buffer_text(buffer, &request);
  }

  let projection = sync_buffer_scroll_and_projection(buffer, raw_fs, &request);

  LayoutCacheEntry {
    key,
    projection,
    payload,
    prepared_document_scroll_y: request.scroll_offset.y,
  }
}

pub(crate) fn scroll_to(entry: &mut LayoutCacheEntry, request: &LayoutRequest<'_>) {
  if entry.prepared_document_scroll_y == request.scroll_offset.y {
    return;
  }

  let mut font_system = font_lock::foreground_font_system_write();

  let raw_fs = font_system.raw();
  let buffer = entry.payload.buffer_mut();

  entry.projection = sync_buffer_scroll_and_projection(buffer, raw_fs, request);
  entry.prepared_document_scroll_y = request.scroll_offset.y;
}

fn metrics_from_request(request: &LayoutRequest<'_>) -> cosmic_text::Metrics {
  cosmic_text::Metrics::new(request.config.font_size, request.config.line_height)
}

fn take_or_create_payload(
  prev: Option<LayoutCacheEntry>,
  font_system: &mut cosmic_text::FontSystem,
  metrics: cosmic_text::Metrics,
) -> CosmicBufferPayload {
  prev.map(|entry| entry.payload).unwrap_or_else(|| {
    let buffer = cosmic_text::Buffer::new(font_system, metrics);

    CosmicBufferPayload::new(buffer)
  })
}

fn sync_buffer_config(
  buffer: &mut cosmic_text::Buffer,
  request: &LayoutRequest<'_>,
  metrics: cosmic_text::Metrics,
) {
  buffer.set_wrap(request.config.wrap_mode.to_cosmic());
  buffer.set_metrics_and_size(
    metrics,
    Some(request.content_size.width),
    Some(request.content_size.height),
  );
  buffer.set_tab_width(request.config.tab_display_policy.spaces_per_tab().into());
}

fn sync_buffer_text(buffer: &mut cosmic_text::Buffer, request: &LayoutRequest<'_>) {
  let attrs = text::to_attributes(request.config.font);

  buffer.set_text(
    request.document.text(),
    &attrs,
    cosmic_text::Shaping::Advanced,
    None,
  );
}

fn sync_buffer_scroll_and_projection(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  request: &LayoutRequest<'_>,
) -> LayoutProjection {
  let source_line_heights = SourceLineHeights::for_request(request);
  let source_offset = source_line_heights.resolve_document_y(request.scroll_offset.y);

  buffer.set_scroll(source_offset.to_cosmic_scroll());
  buffer.shape_until_scroll(font_system, false);

  LayoutProjection::build(buffer, &source_line_heights, request.content_size)
}
