#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets {
  // Top/bottom gap for text and gutter labels.
  // Background still fills the whole widget.
  pub vertical: VerticalInsets,
  // Left/right gap only for the text area.
  pub text: HorizontalInsets,
}

impl Insets {
  pub const ZERO: Self = Self {
    vertical: VerticalInsets::ZERO,
    text: HorizontalInsets::ZERO,
  };

  pub fn new(vertical: impl Into<VerticalInsets>, text: impl Into<HorizontalInsets>) -> Self {
    Self {
      vertical: vertical.into(),
      text: text.into(),
    }
  }
}

impl Default for Insets {
  fn default() -> Self {
    Self::ZERO
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutterInsets {
  pub horizontal: HorizontalInsets,
}

impl GutterInsets {
  pub const ZERO: Self = Self {
    horizontal: HorizontalInsets::ZERO,
  };

  pub fn new(horizontal: impl Into<HorizontalInsets>) -> Self {
    Self {
      horizontal: horizontal.into(),
    }
  }

  pub(crate) fn to_bits(self) -> u64 {
    self.horizontal.to_bits()
  }
}

impl From<HorizontalInsets> for GutterInsets {
  fn from(horizontal: HorizontalInsets) -> Self {
    Self { horizontal }
  }
}

impl From<f32> for GutterInsets {
  fn from(value: f32) -> Self {
    Self::new(value)
  }
}

impl Default for GutterInsets {
  fn default() -> Self {
    Self::new(8.0)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizontalInsets {
  pub left: f32,
  pub right: f32,
}

impl HorizontalInsets {
  pub const ZERO: Self = Self::new(0.0, 0.0);

  pub const fn new(left: f32, right: f32) -> Self {
    Self { left, right }
  }

  pub const fn symmetric(value: f32) -> Self {
    Self {
      left: value,
      right: value,
    }
  }

  pub(crate) fn to_bits(self) -> u64 {
    ((self.left.to_bits() as u64) << 32) | self.right.to_bits() as u64
  }
}

impl From<f32> for HorizontalInsets {
  fn from(value: f32) -> Self {
    Self::symmetric(value)
  }
}

impl Default for HorizontalInsets {
  fn default() -> Self {
    Self::ZERO
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerticalInsets {
  pub top: f32,
  pub bottom: f32,
}

impl VerticalInsets {
  pub const ZERO: Self = Self::new(0.0, 0.0);

  pub const fn new(top: f32, bottom: f32) -> Self {
    Self { top, bottom }
  }

  pub const fn symmetric(value: f32) -> Self {
    Self {
      top: value,
      bottom: value,
    }
  }
}

impl From<f32> for VerticalInsets {
  fn from(value: f32) -> Self {
    Self::symmetric(value)
  }
}

impl Default for VerticalInsets {
  fn default() -> Self {
    Self::ZERO
  }
}
