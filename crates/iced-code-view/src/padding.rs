#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeViewPadding {
  pub top: f32,
  pub bottom: f32,
  pub text_left: f32,
  pub text_right: f32,
}

// TODO: not a very clean paddings model, requires rethinking and a cleaner API
impl CodeViewPadding {
  pub const ZERO: Self = Self {
    top: 0.0,
    bottom: 0.0,
    text_left: 0.0,
    text_right: 0.0,
  };

  pub const fn new(top: f32, bottom: f32, text_left: f32, text_right: f32) -> Self {
    Self {
      top,
      bottom,
      text_left,
      text_right,
    }
  }

  pub const fn text_horizontal(amount: f32) -> Self {
    Self {
      text_left: amount,
      text_right: amount,
      ..Self::ZERO
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
  pub left: f32,
  pub right: f32,
}

impl GutterPadding {
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

impl Default for GutterPadding {
  fn default() -> Self {
    Self::symmetric(8.0)
  }
}
