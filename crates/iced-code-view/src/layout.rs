use crate::document::CodeDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LayoutKey {
  pub(crate) text_revision: u64,
  pub(crate) font_size_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) font: iced::Font,
}

impl LayoutKey {
  pub(crate) fn from_request(request: &LayoutRequest) -> Self {
    Self {
      text_revision: request.document.id(),
      font: request.font,
      font_size_bits: request.font_size.to_bits(),
      line_height_bits: request.line_height.to_bits(),
    }
  }
}

pub(crate) struct LayoutRequest<'a> {
  pub(crate) document: &'a CodeDocument,
  pub(crate) width: f32,
  pub(crate) font: iced::Font,
  pub(crate) font_size: f32,
  pub(crate) line_height: f32,
}
