use iced::advanced::graphics::text::{self, cosmic_text};

use crate::text_layout::{TextLayoutConfig, WrapMode};

pub(super) enum BufferKind {
  SoftWrap { text_content_width: f32 },
  NoWrap,
}

impl BufferKind {
  fn wrap_mode(&self) -> WrapMode {
    match self {
      BufferKind::SoftWrap { .. } => WrapMode::SoftWrap,
      BufferKind::NoWrap => WrapMode::NoWrap,
    }
  }
}

pub(super) fn build_buffer(
  text_layout_config: TextLayoutConfig,
  text: &str,
  kind: BufferKind,
) -> cosmic_text::Buffer {
  let metrics =
    cosmic_text::Metrics::new(text_layout_config.font_size, text_layout_config.line_height);
  let attrs = text::to_attributes(text_layout_config.font);
  let mut buffer = cosmic_text::Buffer::new_empty(metrics);

  let content_width = match kind {
    BufferKind::SoftWrap { text_content_width } => Some(text_content_width),
    BufferKind::NoWrap => None,
  };

  buffer.set_wrap(kind.wrap_mode().to_cosmic());
  buffer.set_metrics_and_size(metrics, content_width, None);
  buffer.set_tab_width(
    text_layout_config
      .tab_display_policy
      .spaces_per_tab()
      .into(),
  );
  buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced, None);

  buffer
}
