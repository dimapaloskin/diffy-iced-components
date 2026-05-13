#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeViewPadding {
  // Top/bottom gap for text and gutter labels.
  // Background still fills the whole widget.
  pub vertical: VerticalPadding,
  // Left/right gap only for the text area.
  pub text: HorizontalPadding,
}

impl CodeViewPadding {
  pub const ZERO: Self = Self {
    vertical: VerticalPadding::ZERO,
    text: HorizontalPadding::ZERO,
  };

  pub fn new(vertical: impl Into<VerticalPadding>, text: impl Into<HorizontalPadding>) -> Self {
    Self {
      vertical: vertical.into(),
      text: text.into(),
    }
  }
}

impl Default for CodeViewPadding {
  fn default() -> Self {
    Self::ZERO
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutterPadding {
  pub horizontal: HorizontalPadding,
}

impl GutterPadding {
  pub const ZERO: Self = Self {
    horizontal: HorizontalPadding::ZERO,
  };

  pub fn new(horizontal: impl Into<HorizontalPadding>) -> Self {
    Self {
      horizontal: horizontal.into(),
    }
  }

  pub fn symmetric(value: f32) -> Self {
    Self::new(value)
  }

  pub(crate) fn to_bits(self) -> u64 {
    self.horizontal.to_bits()
  }
}

impl From<HorizontalPadding> for GutterPadding {
  fn from(horizontal: HorizontalPadding) -> Self {
    Self { horizontal }
  }
}

impl From<(f32, f32)> for GutterPadding {
  fn from(horizontal: (f32, f32)) -> Self {
    Self::new(horizontal)
  }
}

impl From<f32> for GutterPadding {
  fn from(value: f32) -> Self {
    Self::symmetric(value)
  }
}

impl Default for GutterPadding {
  fn default() -> Self {
    Self::symmetric(8.0)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizontalPadding {
  pub left: f32,
  pub right: f32,
}

impl HorizontalPadding {
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

impl From<(f32, f32)> for HorizontalPadding {
  fn from((left, right): (f32, f32)) -> Self {
    Self::new(left, right)
  }
}

impl From<f32> for HorizontalPadding {
  fn from(value: f32) -> Self {
    Self::symmetric(value)
  }
}

impl Default for HorizontalPadding {
  fn default() -> Self {
    Self::ZERO
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerticalPadding {
  pub top: f32,
  pub bottom: f32,
}

impl VerticalPadding {
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

impl From<(f32, f32)> for VerticalPadding {
  fn from((top, bottom): (f32, f32)) -> Self {
    Self::new(top, bottom)
  }
}

impl From<f32> for VerticalPadding {
  fn from(value: f32) -> Self {
    Self::symmetric(value)
  }
}

impl Default for VerticalPadding {
  fn default() -> Self {
    Self::ZERO
  }
}
