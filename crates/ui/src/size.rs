#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Sizing {
  Xs,
  Sm,
  #[default]
  Md,
  Lg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingLevel {
  Tight,
  Regular,
  Loose,
  Spacious,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaddingScale {
  tight: f32,
  regular: f32,
  loose: f32,
  spacious: f32,
}

impl PaddingScale {
  pub fn new(tight: f32, regular: f32, loose: f32, spacious: f32) -> Self {
    Self {
      tight,
      regular,
      loose,
      spacious,
    }
  }

  pub fn value(&self, level: PaddingLevel) -> f32 {
    match level {
      PaddingLevel::Tight => self.tight,
      PaddingLevel::Regular => self.regular,
      PaddingLevel::Loose => self.loose,
      PaddingLevel::Spacious => self.spacious,
    }
  }

  pub fn all(&self, level: PaddingLevel) -> iced::Padding {
    iced::Padding::from(self.value(level))
  }

  pub fn axis(&self, vertical: PaddingLevel, horizontal: PaddingLevel) -> iced::Padding {
    iced::Padding {
      top: self.value(vertical),
      bottom: self.value(vertical),
      left: self.value(horizontal),
      right: self.value(horizontal),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Density {
  height: f32,
  font_size: f32,
  padding: PaddingScale,
  icon_size: f32,
  gap: f32,
}

impl Density {
  pub fn new(height: f32, font_size: f32, padding: PaddingScale) -> Self {
    Self {
      height,
      font_size,
      padding,
      icon_size: font_size,
      gap: font_size * 0.5,
    }
  }

  pub fn with_icon_size(mut self, icon_size: f32) -> Self {
    self.icon_size = icon_size;
    self
  }

  pub fn with_gap(mut self, gap: f32) -> Self {
    self.gap = gap;
    self
  }
}

impl Density {
  pub fn height(&self) -> f32 {
    self.height
  }

  pub fn font_size(&self) -> f32 {
    self.font_size
  }

  pub fn padding_scale(&self) -> PaddingScale {
    self.padding
  }

  pub fn icon_size(&self) -> f32 {
    self.icon_size
  }

  pub fn gap(&self) -> f32 {
    self.gap
  }
}

impl From<Sizing> for Density {
  fn from(size: Sizing) -> Self {
    match size {
      Sizing::Xs => Density::new(24.0, 12.0, PaddingScale::new(4.0, 6.0, 8.0, 10.0))
        .with_icon_size(12.0)
        .with_gap(6.0),
      Sizing::Sm => Density::new(28.0, 13.0, PaddingScale::new(5.0, 7.0, 10.0, 13.0))
        .with_icon_size(13.0)
        .with_gap(6.0),
      Sizing::Md => Density::new(32.0, 14.0, PaddingScale::new(6.0, 8.0, 12.0, 16.0))
        .with_icon_size(14.0)
        .with_gap(8.0),
      Sizing::Lg => Density::new(40.0, 16.0, PaddingScale::new(8.0, 12.0, 16.0, 20.0))
        .with_icon_size(16.0)
        .with_gap(10.0),
    }
  }
}
