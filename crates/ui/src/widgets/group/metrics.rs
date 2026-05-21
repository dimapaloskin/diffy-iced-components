use derive_setters::Setters;

#[derive(Debug, Setters, Clone, Copy)]
pub struct Metrics {
  pub frame_width: f32,
  pub separator_width: f32,
}

impl Default for Metrics {
  fn default() -> Self {
    Self {
      frame_width: 1.0,
      separator_width: 1.0,
    }
  }
}
