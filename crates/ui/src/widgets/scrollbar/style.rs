use crate::theme::{ScrollbarStateTokens, Theme};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Status {
  #[default]
  Idle,
  Hovered,
  Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusStyle {
  pub thumb: iced::Color,
  pub track: iced::Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
  pub idle: StatusStyle,
  pub hovered: StatusStyle,
  pub pressed: StatusStyle,
  pub radius: iced::border::Radius,
}

impl Style {
  pub fn resolve(theme: &Theme) -> Self {
    let scrollbar = theme.scrollbar();

    Self {
      idle: Self::status_style(&scrollbar.idle),
      hovered: Self::status_style(&scrollbar.hovered),
      pressed: Self::status_style(&scrollbar.pressed),
      radius: theme.radius().scrollbar,
    }
  }

  pub fn for_status(&self, status: Status) -> StatusStyle {
    match status {
      Status::Idle => self.idle,
      Status::Hovered => self.hovered,
      Status::Pressed => self.pressed,
    }
  }

  fn status_style(tokens: &ScrollbarStateTokens) -> StatusStyle {
    StatusStyle {
      thumb: tokens.thumb,
      track: tokens.track,
    }
  }
}
