use derive_setters::Setters;

use crate::size::{Density, PaddingLevel, Sizing};

#[derive(Debug, Setters, Clone, Copy)]
pub struct Metrics {
  pub height: f32,
  pub font_size: f32,
  pub icon_size: f32,
  pub gap: f32,
  pub padding: iced::Padding,
}

impl Default for Metrics {
  fn default() -> Self {
    Self::from_density(Sizing::Md.into())
  }
}

impl Metrics {
  pub(crate) fn from_density(density: Density) -> Self {
    Self {
      height: density.height(),
      font_size: density.font_size(),
      icon_size: density.icon_size(),
      gap: density.gap(),
      padding: density
        .padding_scale()
        .axis(PaddingLevel::Tight, PaddingLevel::Loose),
    }
  }

  pub(super) fn icon_only_padding(&self) -> iced::Padding {
    let padding = ((self.height - self.icon_size) / 2.0).max(0.0);

    iced::Padding::from(padding)
  }
}
