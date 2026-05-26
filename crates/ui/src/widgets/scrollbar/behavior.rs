#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Behavior {
  pub visibility: Visibility,
}

impl Behavior {
  pub const fn always_visible(mut self) -> Self {
    self.visibility.always_visible = true;
    self
  }

  pub const fn reveal_on_hover(mut self) -> Self {
    self.visibility.reveal_on_hover = true;
    self
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visibility {
  pub always_visible: bool,
  pub reveal_on_hover: bool,
}

impl Default for Visibility {
  fn default() -> Self {
    Self {
      always_visible: true, // TODO: must be `false` as soon as scrollbar show/hide is implemented
      reveal_on_hover: false,
    }
  }
}
