use crate::document::CodeDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WrapMode {
  #[default]
  NoWrap,
  SoftWrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LayoutKey {
  pub(crate) text_revision: u64,
  pub(crate) content_width_bits: u32,
  pub(crate) content_height_bits: u32,
  pub(crate) font_size_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) font: iced::Font,
}

impl LayoutKey {
  pub(crate) fn from_request(request: &LayoutRequest) -> Self {
    Self {
      text_revision: request.document.id(),
      content_width_bits: request.content_size.width.to_bits(),
      content_height_bits: request.content_size.height.to_bits(),
      font: request.font,
      font_size_bits: request.font_size.to_bits(),
      line_height_bits: request.line_height.to_bits(),
    }
  }
}

pub(crate) struct LayoutRequest<'a> {
  pub(crate) document: &'a CodeDocument,
  pub(crate) content_size: iced::Size,
  pub(crate) scroll_offset: iced::Vector,
  pub(crate) font: iced::Font,
  pub(crate) font_size: f32,
  pub(crate) line_height: f32,
}
