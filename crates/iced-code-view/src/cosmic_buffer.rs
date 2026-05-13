use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text;

pub(crate) struct CosmicBufferPayload {
  buffer: Arc<cosmic_text::Buffer>,
}

impl CosmicBufferPayload {
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
