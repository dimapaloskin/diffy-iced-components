use iced::advanced::graphics::text::{self, cosmic_text};

use crate::font_lock::foreground_font_system_write;
use crate::layout::{LayoutKey, LayoutRequest};
use crate::state::{CosmicLayoutPayload, LayoutCacheEntry, LayoutSnapshot, VisualLineSnapshot};

pub(crate) fn rebuild_layout(
  request: LayoutRequest,
  previous: Option<LayoutCacheEntry>,
) -> LayoutCacheEntry {
  let mut font_system = foreground_font_system_write();

  let needs_set_text = previous
    .as_ref()
    .is_none_or(|entry| entry.key.text_revision != request.document.id());

  let needs_update_attrs = previous.as_ref().is_some_and(|entry| {
    entry.key.text_revision == request.document.id() && entry.key.font != request.config.font
  });

  let raw_font_system = font_system.raw();
  let metrics = cosmic_text::Metrics::new(request.config.font_size, request.config.line_height);

  let mut payload = previous.map(|entry| entry.payload).unwrap_or_else(|| {
    let buffer = cosmic_text::Buffer::new(raw_font_system, metrics);

    CosmicLayoutPayload::new(buffer)
  });

  let buffer = payload.buffer_mut();

  buffer.set_wrap(request.config.wrap_mode.to_cosmic());
  buffer.set_metrics_and_size(
    metrics,
    Some(request.content_size.width),
    Some(request.content_size.height),
  );
  buffer.set_tab_width(request.config.tab_display_policy.spaces_per_tab().into());

  let attrs = text::to_attributes(request.config.font);

  if needs_set_text {
    buffer.set_text(
      request.document.text(),
      &attrs,
      cosmic_text::Shaping::Advanced,
      None,
    );
  } else if needs_update_attrs {
    update_plain_attrs(buffer, &attrs);
  }

  let snapshot = sync_buffer_scroll_and_snapshot(buffer, raw_font_system, request.scroll_offset);

  let key = LayoutKey::from_request(&request);

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

  let mut font_system = foreground_font_system_write();

  let raw_font_system = font_system.raw();
  let buffer = entry.payload.buffer_mut();

  entry.snapshot = sync_buffer_scroll_and_snapshot(buffer, raw_font_system, scroll_offset);
  entry.prepared_scroll_offset = scroll_offset;
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

fn update_plain_attrs(buffer: &mut cosmic_text::Buffer, attrs: &cosmic_text::Attrs<'_>) {
  for line in &mut buffer.lines {
    line.set_attrs_list(cosmic_text::AttrsList::new(attrs));
  }
}
