use iced::advanced::graphics::text::{self, cosmic_text};

use crate::layout::{LayoutKey, LayoutRequest};
use crate::state::{CosmicLayoutPayload, LayoutCacheEntry, LayoutSnapshot};

pub(crate) struct LayoutEngine;

impl LayoutEngine {
  pub(crate) fn rebuild(
    request: LayoutRequest,
    previous: Option<LayoutCacheEntry>,
  ) -> LayoutCacheEntry {
    let mut font_system = text::font_system()
      .write()
      .expect("iced shared font system lock should not be poisoned");

    let raw_font_system = font_system.raw();
    let metrics = cosmic_text::Metrics::new(request.font_size, request.line_height);

    let mut payload = previous.map(|entry| entry.payload).unwrap_or_else(|| {
      let buffer = cosmic_text::Buffer::new(raw_font_system, metrics);

      CosmicLayoutPayload::new(buffer)
    });

    let buffer = payload.buffer_mut();

    buffer.set_wrap(cosmic_text::Wrap::None);
    buffer.set_size(None, None);

    let attrs = text::to_attributes(request.font);

    buffer.set_text(
      request.document.text(),
      &attrs,
      cosmic_text::Shaping::Advanced,
      None,
    );
    buffer.shape_until_scroll(raw_font_system, false);

    let (text_size, _) = text::measure(buffer);

    let snapshot = LayoutSnapshot { text_size };

    let key = LayoutKey::from_request(&request);

    LayoutCacheEntry {
      key,
      snapshot,
      payload,
    }
  }
}
