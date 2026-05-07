use iced::advanced::graphics::text::{self, cosmic_text};

use crate::state::{CosmicLayoutPayload, LayoutCacheEntry, LayoutSnapshot};

pub(crate) struct LayoutEngine;

impl LayoutEngine {
  pub(crate) fn build_or_update(
    view_size: iced::Size,
    previous: Option<LayoutCacheEntry>,
  ) -> LayoutCacheEntry {
    let mut font_system = text::font_system()
      .write()
      .expect("iced shared font system lock should not be poisoned");

    let raw_font_system = font_system.raw();
    let metrics = cosmic_text::Metrics::new(16.0, 24.0);

    let mut payload = previous.map(|entry| entry.payload).unwrap_or_else(|| {
      let buffer = cosmic_text::Buffer::new(raw_font_system, metrics);

      CosmicLayoutPayload::new(buffer)
    });

    let buffer = payload.buffer_mut();

    buffer.set_wrap(cosmic_text::Wrap::None);
    buffer.set_size(None, None);

    let attrs = text::to_attributes(iced::Font::MONOSPACE);

    buffer.set_text("Hello", &attrs, cosmic_text::Shaping::Advanced, None);
    buffer.shape_until_scroll(raw_font_system, false);

    let (text_size, _) = text::measure(buffer);

    let text_origin = iced::Vector::new(
      (view_size.width - text_size.width) / 2.0,
      (view_size.height - text_size.height) / 2.0,
    );

    let snapshot = LayoutSnapshot {
      text_size,
      text_origin,
    };

    LayoutCacheEntry { snapshot, payload }
  }
}
