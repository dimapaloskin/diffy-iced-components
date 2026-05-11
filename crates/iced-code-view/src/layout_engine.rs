use iced::advanced::graphics::text::{self, cosmic_text};

use crate::font_lock;
use crate::layout::{LayoutKey, LayoutRequest};
use crate::layout_cache::{CosmicLayoutPayload, LayoutCacheEntry};
use crate::projection::LayoutProjection;
use crate::source_line::SourceLineHeights;

pub(crate) fn rebuild_layout(
  request: LayoutRequest,
  key: LayoutKey,
  prev: Option<LayoutCacheEntry>,
) -> LayoutCacheEntry {
  let mut font_system = font_lock::foreground_font_system_write();
  let raw_fs = font_system.raw();

  let prev_key = prev.as_ref().map(|entry| entry.key);
  let metrics = metrics_from_request(&request);
  let mut payload = take_or_create_payload(prev, raw_fs, metrics);

  let buffer = payload.buffer_mut();

  sync_buffer_config(buffer, &request, metrics);
  sync_buffer_text(buffer, &request, prev_key);

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
) -> CosmicLayoutPayload {
  prev.map(|entry| entry.payload).unwrap_or_else(|| {
    let buffer = cosmic_text::Buffer::new(font_system, metrics);

    CosmicLayoutPayload::new(buffer)
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

// Buffer picks up metrics/size/wrap/tab via setters in sync_buffer_config.
// Only update text and font here — nothing else affects attrs.
fn sync_buffer_text(
  buffer: &mut cosmic_text::Buffer,
  request: &LayoutRequest<'_>,
  prev_key: Option<LayoutKey>,
) {
  let attrs = text::to_attributes(request.config.font);

  if needs_set_text(prev_key, request) {
    buffer.set_text(
      request.document.text(),
      &attrs,
      cosmic_text::Shaping::Advanced,
      None,
    );
  } else if needs_update_attrs(prev_key, request) {
    update_plain_attrs(buffer, &attrs);
  }
}

fn needs_set_text(prev_key: Option<LayoutKey>, request: &LayoutRequest<'_>) -> bool {
  prev_key.is_none_or(|key| key.text_revision != request.document.id())
}

fn needs_update_attrs(prev_key: Option<LayoutKey>, request: &LayoutRequest<'_>) -> bool {
  prev_key.is_some_and(|key| {
    key.text_revision == request.document.id() && key.font != request.config.font
  })
}

fn update_plain_attrs(buffer: &mut cosmic_text::Buffer, attrs: &cosmic_text::Attrs<'_>) {
  for line in &mut buffer.lines {
    line.set_attrs_list(cosmic_text::AttrsList::new(attrs));
  }
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
