use derive_setters::Setters;

#[derive(Debug, Clone, Copy, PartialEq, Setters)]
pub struct Metrics {
  pub thickness_idle: f32,
  pub thickness_active: f32,
  pub hit_thickness: f32,
  pub min_thumb_len: f32,
  pub edge_inset: f32,
  pub track_inset_start: f32,
  pub track_inset_end: f32,
}

impl Default for Metrics {
  fn default() -> Self {
    Self {
      thickness_idle: 6.0,
      thickness_active: 8.0,
      hit_thickness: 14.0,
      min_thumb_len: 28.0,
      edge_inset: 0.0,
      track_inset_start: 4.0,
      track_inset_end: 4.0,
    }
  }
}

impl Metrics {
  pub fn edge_reserve(&self) -> f32 {
    self.edge_inset.max(0.0) + self.hit_thickness.max(0.0)
  }
}
