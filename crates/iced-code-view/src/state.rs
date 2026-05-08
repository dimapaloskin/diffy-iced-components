use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text;

use crate::layout::LayoutKey;
use crate::scroll::ScrollExtent;
use crate::viewport::Viewport;

#[derive(Default)]
pub(crate) struct CodeViewState {
  pub(crate) line: Option<LayoutCacheEntry>,
  pub(crate) viewport: Viewport,
  pub(crate) scroll_extent: ScrollExtent,
}

pub(crate) struct LayoutCacheEntry {
  pub(crate) key: LayoutKey,
  pub(crate) snapshot: LayoutSnapshot,
  pub(crate) payload: CosmicLayoutPayload,
  pub(crate) scroll_offset: iced::Vector,
}

pub(crate) struct LayoutSnapshot {
  pub(crate) text_size: iced::Size,
  pub(crate) visual_lines: Vec<VisualLineSnapshot>,
}

pub(crate) struct VisualLineSnapshot {
  pub(crate) source_line_index: usize,
  pub(crate) y: f32,
  pub(crate) height: f32,
  pub(crate) width: f32,
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
