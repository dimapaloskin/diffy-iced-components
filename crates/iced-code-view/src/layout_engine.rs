use iced::advanced::graphics::text::{self, cosmic_text};

use crate::font_lock;
use crate::layout::{LayoutKey, LayoutRequest};
use crate::layout_cache::{
  CosmicLayoutPayload, LayoutCacheEntry, LayoutSnapshot, VisualLineSnapshot,
};

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

  let snapshot = sync_buffer_scroll_and_snapshot(buffer, raw_fs, request.scroll_offset);
  LayoutCacheEntry {
    key,
    snapshot,
    payload,
    prepared_scroll_offset: request.scroll_offset,
  }
}

pub(crate) fn sync_scroll(entry: &mut LayoutCacheEntry, scroll_offset: iced::Vector) {
  if entry.prepared_scroll_offset == scroll_offset {
    return;
  }

  let mut font_system = font_lock::foreground_font_system_write();

  let raw_fs = font_system.raw();
  let buffer = entry.payload.buffer_mut();

  entry.snapshot = sync_buffer_scroll_and_snapshot(buffer, raw_fs, scroll_offset);
  entry.prepared_scroll_offset = scroll_offset;
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

fn sync_buffer_scroll_and_snapshot(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  scroll_offset: iced::Vector,
) -> LayoutSnapshot {
  // `layout_runs()` accounts for `Scroll::vertical` when choosing visible lines,
  // but glyph `x` positions stay relative to the start of each line.
  // Keep horizontal at zero here and apply `scroll_offset.x` in draw translation.
  buffer.set_scroll(cosmic_text::Scroll::new(0, scroll_offset.y, 0.0));
  buffer.shape_until_scroll(font_system, false);

  snapshot_from_buffer(buffer)
}

fn snapshot_from_buffer(buffer: &cosmic_text::Buffer) -> LayoutSnapshot {
  let mut text_width: f32 = 0.0;
  let mut text_height: f32 = 0.0;
  let mut visual_lines = Vec::new();

  for run in buffer.layout_runs() {
    text_width = text_width.max(run.line_w);
    text_height += run.line_height;

    visual_lines.push(VisualLineSnapshot {
      source_line_index: run.line_i,
      y: run.line_top,
      height: run.line_height,
      width: run.line_w,
    });
  }

  LayoutSnapshot {
    text_size: iced::Size::new(text_width, text_height),
    visual_lines,
  }
}
