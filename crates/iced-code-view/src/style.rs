#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
  pub background_color: iced::Color,
  pub text_color: iced::Color,
  pub gutter: GutterStyle,
}

impl Default for Style {
  fn default() -> Self {
    Self {
      background_color: iced::Color::BLACK,
      text_color: iced::Color::WHITE,
      gutter: GutterStyle::default(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutterStyle {
  pub line_number_color: iced::Color,
  pub background_color: iced::Color,
  pub separator_color: iced::Color,
}

impl Default for GutterStyle {
  fn default() -> Self {
    Self {
      line_number_color: iced::Color::from_rgb(0.45, 0.48, 0.54),
      background_color: iced::Color::from_rgb(0.20, 0.22, 0.20),
      separator_color: iced::Color::from_rgb(0.10, 0.18, 0.10),
    }
  }
}
