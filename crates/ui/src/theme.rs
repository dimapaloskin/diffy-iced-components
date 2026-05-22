pub mod catalog;
pub mod presets;
pub mod tokens;

pub use tokens::{
  BorderTokens, ColorTokens, FamilyTone, FocusTokens, RadiusTokens, ScrollbarStateTokens,
  ScrollbarTokens, ShadowTokens,
};

use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
  Dark,
  Light,
}

impl ColorScheme {
  pub fn iced_mode(&self) -> iced::theme::Mode {
    match self {
      Self::Dark => iced::theme::Mode::Dark,
      Self::Light => iced::theme::Mode::Light,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ThemeTokens {
  pub name: Cow<'static, str>,
  pub scheme: ColorScheme,
  pub border: BorderTokens,
  pub colors: ColorTokens,
  pub focus: FocusTokens,
  pub radius: RadiusTokens,
  pub shadows: ShadowTokens,
  pub scrollbar: ScrollbarTokens,
}

#[derive(Debug, Clone)]
pub struct Theme {
  tokens: ThemeTokens,
}

impl Theme {
  pub fn light() -> Self {
    Self::from_tokens(presets::default::tokens(ColorScheme::Light))
  }

  pub fn dark() -> Self {
    Self::from_tokens(presets::default::tokens(ColorScheme::Dark))
  }

  pub fn from_tokens(tokens: ThemeTokens) -> Self {
    Self { tokens }
  }

  pub fn from_scheme(scheme: ColorScheme) -> Self {
    Self::from_tokens(presets::default::tokens(scheme))
  }

  pub fn tokens(&self) -> &ThemeTokens {
    &self.tokens
  }

  pub fn with_tokens(mut self, update: impl FnOnce(&mut ThemeTokens)) -> Self {
    update(&mut self.tokens);
    self
  }

  pub fn name(&self) -> &str {
    &self.tokens.name
  }

  pub fn scheme(&self) -> ColorScheme {
    self.tokens.scheme
  }

  pub fn border(&self) -> &BorderTokens {
    &self.tokens.border
  }

  pub fn colors(&self) -> &ColorTokens {
    &self.tokens.colors
  }

  pub fn focus(&self) -> &FocusTokens {
    &self.tokens.focus
  }

  pub fn radius(&self) -> &RadiusTokens {
    &self.tokens.radius
  }

  pub fn shadows(&self) -> &ShadowTokens {
    &self.tokens.shadows
  }

  pub fn scrollbar(&self) -> &ScrollbarTokens {
    &self.tokens.scrollbar
  }
}

impl iced::theme::Base for Theme {
  fn default(preference: iced::theme::Mode) -> Self {
    match preference {
      iced::theme::Mode::Dark => Self::dark(),
      iced::theme::Mode::Light | iced::theme::Mode::None => Self::light(),
    }
  }

  fn mode(&self) -> iced::theme::Mode {
    self.scheme().iced_mode()
  }

  fn base(&self) -> iced::theme::Style {
    iced::theme::Style {
      background_color: self.colors().background,
      text_color: self.colors().foreground,
    }
  }

  fn seed(&self) -> Option<iced::theme::palette::Seed> {
    None
  }

  fn name(&self) -> &str {
    self.name()
  }
}
