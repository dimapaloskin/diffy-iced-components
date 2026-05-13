use crate::cosmic_buffer::CosmicBufferPayload;
use crate::text_layout::{TextLayoutKey, VisibleTextProjection};

pub(crate) struct VisibleTextLayout {
  pub(crate) key: TextLayoutKey,
  pub(crate) projection: VisibleTextProjection,
  pub(crate) payload: CosmicBufferPayload,
  pub(crate) prepared_content_height_bits: u32,
  pub(crate) prepared_document_scroll_y_bits: u32,
}
