use iced::Rectangle;

use super::metrics::Metrics;
use super::{Axis, AxisSnapshot, ScrollExtent};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
  pub axis: Axis,
  pub placement_bounds: Rectangle,
  pub hit_bounds: Rectangle,
  pub track_bounds: Rectangle,
  pub thumb: Option<ThumbGeometry>,
}

impl Geometry {
  pub fn new(snapshot: AxisSnapshot, placement_bounds: Rectangle, metrics: Metrics) -> Self {
    let axis = snapshot.axis;
    let hit_bounds = Self::hit_bounds(axis, placement_bounds, metrics);
    let track_bounds = Self::track_bounds(axis, hit_bounds, metrics);
    let thumb = ThumbGeometry::new(snapshot, track_bounds, metrics);

    Self {
      axis,
      placement_bounds,
      hit_bounds,
      track_bounds,
      thumb,
    }
  }

  fn hit_bounds(axis: Axis, placement: Rectangle, metrics: Metrics) -> Rectangle {
    match axis {
      Axis::Vertical => {
        let width = metrics.hit_thickness.max(0.0).min(placement.width.max(0.0));
        let edge_inset = metrics
          .edge_inset
          .max(0.0)
          .min((placement.width - width).max(0.0));

        Rectangle {
          x: placement.x + placement.width - edge_inset - width,
          y: placement.y,
          width,
          height: placement.height.max(0.0),
        }
      }
      Axis::Horizontal => {
        let height = metrics
          .hit_thickness
          .max(0.0)
          .min(placement.height.max(0.0));
        let edge_inset = metrics
          .edge_inset
          .max(0.0)
          .min((placement.height - height).max(0.0));

        Rectangle {
          x: placement.x,
          y: placement.y + placement.height - edge_inset - height,
          width: placement.width.max(0.0),
          height,
        }
      }
    }
  }

  fn track_bounds(axis: Axis, hit: Rectangle, metrics: Metrics) -> Rectangle {
    match axis {
      Axis::Vertical => {
        let start = metrics.track_inset_start.max(0.0).min(hit.height);
        let end = metrics
          .track_inset_end
          .max(0.0)
          .min((hit.height - start).max(0.0));

        Rectangle {
          y: hit.y + start,
          height: (hit.height - start - end).max(0.0),
          ..hit
        }
      }
      Axis::Horizontal => {
        let start = metrics.track_inset_start.max(0.0).min(hit.width);
        let end = metrics
          .track_inset_end
          .max(0.0)
          .min((hit.width - start).max(0.0));

        Rectangle {
          x: hit.x + start,
          width: (hit.width - start - end).max(0.0),
          ..hit
        }
      }
    }
  }

  pub fn is_renderable(&self) -> bool {
    self.thumb.is_some()
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThumbGeometry {
  pub hit_bounds: Rectangle,
  pub visual_bounds: Rectangle,
  pub len: f32,
  pub travel_len: f32,
  pub max_scroll: f64,
}

impl ThumbGeometry {
  fn new(snapshot: AxisSnapshot, track_bounds: Rectangle, metrics: Metrics) -> Option<Self> {
    let ScrollExtent::Exact { content_len } = snapshot.extent else {
      return None;
    };

    if snapshot.viewport_len <= 0.0 || content_len <= snapshot.viewport_len {
      return None;
    }

    let track_len = Self::axis_len(snapshot.axis, track_bounds);
    if track_len <= 0.0 {
      return None;
    }

    let max_scroll = content_len - snapshot.viewport_len;
    let raw_thumb_len = (snapshot.viewport_len / content_len * (track_len as f64)) as f32;
    let min_thumb_len = metrics.min_thumb_len.max(0.0).min(track_len);
    let len = raw_thumb_len.clamp(min_thumb_len, track_len);
    let travel_len = (track_len - len).max(0.0);
    let offset = snapshot.offset.clamp(0.0, max_scroll);
    let thumb_start = if travel_len == 0.0 {
      0.0
    } else {
      (offset / max_scroll * f64::from(travel_len)) as f32
    };
    let hit_bounds = Self::thumb_bounds(snapshot.axis, track_bounds, thumb_start, len);
    let visual_bounds = Self::visual_bounds(snapshot.axis, hit_bounds, metrics.thickness_idle);

    Some(Self {
      hit_bounds,
      visual_bounds,
      len,
      travel_len,
      max_scroll,
    })
  }

  pub fn offset_for_thumb_start(&self, thumb_start: f32) -> Option<f64> {
    if self.travel_len == 0.0 || self.max_scroll <= 0.0 {
      return None;
    }

    let thumb_start = thumb_start.clamp(0.0, self.travel_len);
    let offset = (thumb_start / self.travel_len) as f64 * self.max_scroll;

    Some(offset)
  }

  fn axis_len(axis: Axis, bounds: Rectangle) -> f32 {
    match axis {
      Axis::Vertical => bounds.height,
      Axis::Horizontal => bounds.width,
    }
  }

  fn thumb_bounds(axis: Axis, track: Rectangle, start: f32, len: f32) -> Rectangle {
    match axis {
      Axis::Vertical => Rectangle {
        y: track.y + start,
        height: len,
        ..track
      },
      Axis::Horizontal => Rectangle {
        x: track.x + start,
        width: len,
        ..track
      },
    }
  }

  fn visual_bounds(axis: Axis, hit: Rectangle, thickness: f32) -> Rectangle {
    match axis {
      Axis::Vertical => {
        let thickness = thickness.max(0.0).min(hit.width);

        Rectangle {
          x: hit.x + (hit.width - thickness) / 2.0,
          width: thickness,
          ..hit
        }
      }
      Axis::Horizontal => {
        let thickness = thickness.max(0.0).min(hit.height);

        Rectangle {
          y: hit.y + (hit.height - thickness) / 2.0,
          height: thickness,
          ..hit
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use iced::Rectangle;

  use super::*;
  use crate::widgets::scrollbar::ScrollExtent;

  fn snapshot(extent: ScrollExtent) -> AxisSnapshot {
    AxisSnapshot {
      axis: Axis::Vertical,
      offset: 0.0,
      viewport_len: 100.0,
      extent,
    }
  }

  fn placement_bounds() -> Rectangle {
    Rectangle {
      x: 10.0,
      y: 20.0,
      width: 200.0,
      height: 300.0,
    }
  }

  fn scrollable_snapshot() -> AxisSnapshot {
    AxisSnapshot {
      axis: Axis::Vertical,
      offset: 0.0,
      viewport_len: 100.0,
      extent: ScrollExtent::Exact {
        content_len: 1000.0,
      },
    }
  }

  #[test]
  fn does_not_create_thumb_for_non_scrollable_snapshots() {
    let metrics = Metrics::default();

    assert!(
      Geometry::new(
        snapshot(ScrollExtent::Disabled),
        placement_bounds(),
        metrics
      )
      .thumb
      .is_none()
    );
    assert!(
      Geometry::new(snapshot(ScrollExtent::Unknown), placement_bounds(), metrics)
        .thumb
        .is_none()
    );
    assert!(
      Geometry::new(
        snapshot(ScrollExtent::Exact { content_len: 100.0 }),
        placement_bounds(),
        metrics,
      )
      .thumb
      .is_none()
    );
  }

  #[test]
  fn builds_vertical_geometry_with_stable_hit_and_thin_visual_thumb() {
    let metrics = Metrics::default();
    let geometry = Geometry::new(scrollable_snapshot(), placement_bounds(), metrics);
    let thumb = geometry
      .thumb
      .expect("scrollable exact snapshot should create thumb");

    assert_eq!(
      geometry.hit_bounds,
      Rectangle {
        x: 196.0,
        y: 20.0,
        width: 14.0,
        height: 300.0,
      }
    );

    assert_eq!(
      geometry.track_bounds,
      Rectangle {
        x: 196.0,
        y: 24.0,
        width: 14.0,
        height: 292.0,
      }
    );

    assert_eq!(thumb.hit_bounds.width, metrics.hit_thickness);
    assert_eq!(thumb.visual_bounds.width, metrics.thickness_idle);
    assert_eq!(thumb.visual_bounds.x, 200.0);
  }

  #[test]
  fn maps_thumb_start_back_to_scroll_offset() {
    let metrics = Metrics::default();
    let geometry = Geometry::new(scrollable_snapshot(), placement_bounds(), metrics);
    let thumb = geometry
      .thumb
      .expect("scrollable exact snapshot should create thumb");

    assert_eq!(thumb.offset_for_thumb_start(0.0), Some(0.0));
    assert_eq!(
      thumb.offset_for_thumb_start(thumb.travel_len),
      Some(thumb.max_scroll)
    );
    assert_eq!(
      thumb.offset_for_thumb_start(thumb.travel_len + 10.0),
      Some(thumb.max_scroll)
    );
  }

  #[test]
  fn edge_reserve_matches_vertical_hit_footprint() {
    let metrics = Metrics {
      edge_inset: 7.0,
      hit_thickness: 14.0,
      ..Metrics::default()
    };

    let placement = placement_bounds();
    let geometry = Geometry::new(scrollable_snapshot(), placement, metrics);
    let placement_right = placement.x + placement.width;

    assert_eq!(metrics.edge_reserve(), 21.0);
    assert_eq!(
      geometry.hit_bounds.x,
      placement_right - metrics.edge_reserve()
    );
  }
}
