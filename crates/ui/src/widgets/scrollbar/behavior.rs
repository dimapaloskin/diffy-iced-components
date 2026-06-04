use iced::time::Duration;

#[derive(Debug, Default, Clone, Copy)]
pub struct Behavior {
  pub visibility: Visibility,
  pub motion: Motion,
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
    self.motion.fade_out_delay = duration;
    self
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Visibility {
  pub always_visible: bool,
  pub reveal_on_hover: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Motion {
  pub fade_in_duration: Duration,
  pub fade_out_duration: Duration,
  pub fade_out_delay: Duration,
}

impl Default for Motion {
  fn default() -> Self {
    Self {
      fade_in_duration: Duration::from_millis(200),
      fade_out_duration: Duration::from_millis(300),
      fade_out_delay: Duration::from_millis(1200),
    }
  }
}
