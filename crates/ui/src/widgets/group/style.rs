use derive_setters::Setters;

use crate::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme, Style) -> Style + 'a>;

#[derive(Debug, Setters, Clone, Copy)]
pub struct Style {
  pub frame_color: iced::Color,
  pub separator_color: iced::Color,
}

impl Style {
  pub fn resolve(theme: &Theme) -> Self {
    Self {
      frame_color: theme.colors().border,
      separator_color: theme.colors().border,
    }
  }
}
