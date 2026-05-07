use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text;

#[derive(Default)]
pub(crate) struct CodeViewState {
  pub(crate) line: Option<LayoutCacheEntry>,
}

pub(crate) struct LayoutCacheEntry {
  pub(crate) snapshot: LayoutSnapshot,
  pub(crate) payload: CosmicLayoutPayload,
}

pub(crate) struct LayoutSnapshot {
  pub(crate) text_size: iced::Size,
  pub(crate) text_origin: iced::Vector,
}

pub(crate) struct CosmicLayoutPayload {
  buffer: Arc<cosmic_text::Buffer>,
}

impl CosmicLayoutPayload {
  pub(crate) fn new(buffer: cosmic_text::Buffer) -> Self {
    Self {
      buffer: Arc::new(buffer),
    }
  }

  pub(crate) fn buffer(&self) -> &Arc<cosmic_text::Buffer> {
    &self.buffer
  }

  pub(crate) fn buffer_mut(&mut self) -> &mut cosmic_text::Buffer {
    Arc::make_mut(&mut self.buffer)
  }
}
