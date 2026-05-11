use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text;

use crate::layout::LayoutKey;
use crate::projection::LayoutProjection;

pub(crate) struct LayoutCacheEntry {
  pub(crate) key: LayoutKey,
  pub(crate) projection: LayoutProjection,
  pub(crate) payload: CosmicLayoutPayload,
  // The real scroll offset lives in Viewport.
  // This is just the Y offset already applied to this buffer/projection.
  pub(crate) prepared_document_scroll_y: f32,
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
