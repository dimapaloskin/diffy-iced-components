pub(crate) mod cache;
pub(crate) mod engine;
pub(crate) mod projection;

pub(crate) use cache::VisibleTextLayout;
pub(crate) use projection::VisibleTextProjection;

use iced::advanced::graphics::text::{self, cosmic_text};

use crate::scroll::VerticalScroll;
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
pub(crate) struct TextLayoutConfig {
  pub(crate) font: iced::Font,
  pub(crate) font_size: f32,
  pub(crate) line_height: f32,
  pub(crate) wrap_mode: WrapMode,
  pub(crate) tab_display_policy: TabDisplayPolicy,
}

impl Default for TextLayoutConfig {
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
pub(crate) struct TextLayoutKey {
  pub(crate) document_revision: u64,
  pub(crate) content_width_bits: u32,
  pub(crate) font_size_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) font: iced::Font,
  pub(crate) font_system_version: text::Version,
  pub(crate) wrap_mode: WrapMode,
  pub(crate) tab_policy: TabDisplayPolicy,
}

impl TextLayoutKey {
  pub(crate) fn from_request(
    request: &TextLayoutRequest,
    font_system_version: text::Version,
  ) -> Self {
    Self {
      document_revision: request.document.revision(),
      content_width_bits: request.content_size.width.to_bits(),
      font: request.config.font,
      font_system_version,
      font_size_bits: request.config.font_size.to_bits(),
      line_height_bits: request.config.line_height.to_bits(),
      wrap_mode: request.config.wrap_mode,
      tab_policy: request.config.tab_display_policy.normalized(),
    }
  }
}

pub(crate) struct TextLayoutRequest<'a> {
  pub(crate) document: &'a Document,
  pub(crate) content_size: iced::Size,
  pub(crate) vertical_scroll: VerticalScroll,
  pub(crate) config: TextLayoutConfig,
}
