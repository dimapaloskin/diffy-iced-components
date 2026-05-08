use iced::advanced::graphics::text::{self, cosmic_text};

use crate::layout::{LayoutKey, LayoutRequest};
use crate::state::{CosmicLayoutPayload, LayoutCacheEntry, LayoutSnapshot, VisualLineSnapshot};

pub(crate) fn rebuild(
  request: LayoutRequest,
  previous: Option<LayoutCacheEntry>,
) -> LayoutCacheEntry {
  let mut font_system = text::font_system()
    .write()
    .expect("iced shared font system lock should not be poisoned");

  let needs_set_text = previous
    .as_ref()
    .is_none_or(|entry| entry.key.text_revision != request.document.id());

  let needs_update_attrs = previous.as_ref().is_some_and(|entry| {
    entry.key.text_revision == request.document.id() && entry.key.font != request.font
  });

  let raw_font_system = font_system.raw();
  let metrics = cosmic_text::Metrics::new(request.font_size, request.line_height);

  let mut payload = previous.map(|entry| entry.payload).unwrap_or_else(|| {
    let buffer = cosmic_text::Buffer::new(raw_font_system, metrics);

    CosmicLayoutPayload::new(buffer)
  });

  let buffer = payload.buffer_mut();

  buffer.set_wrap(cosmic_text::Wrap::None);
  buffer.set_metrics_and_size(
    metrics,
    Some(request.content_size.width),
    Some(request.content_size.height),
  );

  // `layout_runs()` accounts for `Scroll::vertical` when choosing visible lines,
  // but glyph `x` positions stay relative to the start of each line.
  // Keep horizontal at zero here and apply `scroll_offset.x` in draw translation
  buffer.set_scroll(cosmic_text::Scroll::new(0, request.scroll_offset.y, 0.0));

  let attrs = text::to_attributes(request.font);

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

  buffer.shape_until_scroll(raw_font_system, false);

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

  let snapshot = LayoutSnapshot {
    text_size: iced::Size::new(text_width, text_height),
    visual_lines,
  };

  let key = LayoutKey::from_request(&request);

  LayoutCacheEntry {
    key,
    snapshot,
    payload,
    scroll_offset: request.scroll_offset,
  }
}

fn update_plain_attrs(buffer: &mut cosmic_text::Buffer, attrs: &cosmic_text::Attrs<'_>) {
  for line in &mut buffer.lines {
    line.set_attrs_list(cosmic_text::AttrsList::new(attrs));
  }
}
