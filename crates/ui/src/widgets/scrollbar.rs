pub mod behavior;
pub mod draw;
pub mod geometry;
pub mod interaction;
pub mod metrics;
pub mod style;

pub use behavior::{Behavior, Visibility};
pub use draw::draw;
pub use geometry::{Geometry, ThumbGeometry};
pub use interaction::{Interaction, State, Update};
pub use metrics::Metrics;
pub use style::{Status, StatusStyle, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
  Vertical,
  Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollExtent {
  Disabled,
  Unknown,
  Exact { content_len: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisSnapshot {
  pub axis: Axis,
  pub offset: f64,
  pub viewport_len: f64,
  pub extent: ScrollExtent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
  DragTo {
    axis: Axis,
    offset: f64,
  },
  TrackPress {
    axis: Axis,
    region: TrackPressRegion,
    track_ratio: f64,
    pointer_offset: f64,
  },
}

impl Action {
  pub fn axis(&self) -> Axis {
    match self {
      Action::DragTo { axis, .. } => *axis,
      Action::TrackPress { axis, .. } => *axis,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackPressRegion {
  BeforeThumb,
  AfterThumb,
}
