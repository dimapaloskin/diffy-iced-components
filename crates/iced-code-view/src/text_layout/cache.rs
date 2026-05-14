use crate::cosmic_buffer::CosmicBufferPayload;
use crate::scroll::VerticalScroll;
use crate::text_layout::{TextLayoutKey, VisibleTextProjection};

pub(crate) struct VisibleTextLayout {
  pub(crate) key: TextLayoutKey,
  pub(crate) projection: VisibleTextProjection,
  pub(crate) payload: CosmicBufferPayload,
  pub(crate) prepared_content_height: f32,
  pub(crate) prepared_vertical_scroll: VerticalScroll,
}
