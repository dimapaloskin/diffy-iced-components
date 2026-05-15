use crate::cosmic_buffer::CosmicBufferPayload;
use crate::scroll::VerticalScroll;
use crate::text_layout::{TextLayoutKey, VisibleTextProjection};
use iced::advanced::graphics::text::cosmic_text;

pub(crate) struct VisibleTextLayout {
  pub(crate) key: TextLayoutKey,
  pub(crate) projection: VisibleTextProjection,
  pub(crate) payload: CosmicBufferPayload,
  pub(crate) prepared_content_height: f32,
  pub(crate) cosmic_scroll: cosmic_text::Scroll,
}

impl VisibleTextLayout {
  pub(crate) fn vertical_scroll(&self) -> VerticalScroll {
    VerticalScroll::from_cosmic(self.cosmic_scroll)
  }
}
