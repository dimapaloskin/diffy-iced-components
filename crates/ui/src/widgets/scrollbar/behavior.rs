use iced::time::Duration;

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

  pub const fn auto_hide_after(mut self, duration: Duration) -> Self {
    self.visibility.always_visible = false;
    self.visibility.hide_delay = duration;
    self
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visibility {
  pub always_visible: bool,
  pub reveal_on_hover: bool,
  pub hide_delay: Duration,
}

impl Default for Visibility {
  fn default() -> Self {
    Self {
      always_visible: false,
      reveal_on_hover: false,
      hide_delay: Duration::from_millis(900),
    }
  }
}
