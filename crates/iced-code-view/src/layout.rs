use iced::advanced::graphics::text::cosmic_text;

use crate::{TabDisplayPolicy, document::Document};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WrapMode {
  #[default]
  NoWrap,
  SoftWrap,
}

impl WrapMode {
  pub(crate) fn to_cosmic(self) -> cosmic_text::Wrap {
    match self {
      WrapMode::NoWrap => cosmic_text::Wrap::None,
      WrapMode::SoftWrap => unimplemented!("SoftWrap is not supported yet"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LayoutConfig {
  pub(crate) font: iced::Font,
  pub(crate) font_size: f32,
  pub(crate) line_height: f32,
  pub(crate) wrap_mode: WrapMode,
  pub(crate) tab_display_policy: TabDisplayPolicy,
}

impl Default for LayoutConfig {
  fn default() -> Self {
    Self {
      font: iced::Font::MONOSPACE,
      font_size: 16.0,
      line_height: 24.0,
      wrap_mode: WrapMode::NoWrap,
      tab_display_policy: TabDisplayPolicy::default(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LayoutKey {
  pub(crate) text_revision: u64,
  pub(crate) content_width_bits: u32,
  pub(crate) content_height_bits: u32,
  pub(crate) font_size_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) font: iced::Font,
  pub(crate) wrap_mode: WrapMode,
  pub(crate) tab_policy: TabDisplayPolicy,
}

impl LayoutKey {
  pub(crate) fn from_request(request: &LayoutRequest) -> Self {
    Self {
      text_revision: request.document.id(),
      content_width_bits: request.content_size.width.to_bits(),
      content_height_bits: request.content_size.height.to_bits(),
      font: request.config.font,
      font_size_bits: request.config.font_size.to_bits(),
      line_height_bits: request.config.line_height.to_bits(),
      wrap_mode: request.config.wrap_mode,
      tab_policy: request.config.tab_display_policy.normalized(),
    }
  }
}

pub(crate) struct LayoutRequest<'a> {
  pub(crate) document: &'a Document,
  pub(crate) content_size: iced::Size,
  pub(crate) scroll_offset: iced::Vector,
  pub(crate) config: LayoutConfig,
}
