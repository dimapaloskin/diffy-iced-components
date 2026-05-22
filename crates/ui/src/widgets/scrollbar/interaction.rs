use iced::Point;

use super::{Action, Axis, Geometry};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct State {
  interaction: Interaction,
}

impl State {
  pub fn interaction(&self) -> Interaction {
    self.interaction
  }

  pub fn is_dragging(&self) -> bool {
    matches!(self.interaction, Interaction::Dragging { .. })
  }

  pub fn press(&mut self, cursor_position: Point, geometry: &Geometry) -> Update {
    let Some(thumb) = geometry.thumb else {
      return Update::ignored();
    };

    if !thumb.hit_bounds.contains(cursor_position) {
      return Update::ignored();
    }

    let axis = geometry.axis;

    let grab_offset =
      Self::axis_position(axis, cursor_position) - Self::axis_start(axis, thumb.hit_bounds);

    self.interaction = Interaction::Dragging { axis, grab_offset };

    Update {
      action: None,
      capture_event: true,
      request_redraw: true,
    }
  }

  pub fn drag_to(&self, cursor_position: Point, geometry: &Geometry) -> Update {
    let Interaction::Dragging { axis, grab_offset } = self.interaction else {
      return Update::ignored();
    };

    if axis != geometry.axis {
      return Update::ignored();
    }

    let Some(thumb) = geometry.thumb else {
      return Update::ignored();
    };

    let thumb_start = Self::axis_position(axis, cursor_position)
      - grab_offset
      - Self::axis_start(axis, geometry.track_bounds);

    let Some(offset) = thumb.offset_for_thumb_start(thumb_start) else {
      return Update::ignored();
    };

    Update {
      action: Some(Action::DragTo { axis, offset }),
      capture_event: true,
      request_redraw: true,
    }
  }

  pub fn release(&mut self) -> Update {
    if !self.is_dragging() {
      return Update::ignored();
    }

    self.interaction = Interaction::Idle;

    Update {
      action: None,
      capture_event: true,
      request_redraw: true,
    }
  }

  fn axis_position(axis: Axis, point: Point) -> f32 {
    match axis {
      Axis::Vertical => point.y,
      Axis::Horizontal => point.x,
    }
  }

  fn axis_start(axis: Axis, bounds: iced::Rectangle) -> f32 {
    match axis {
      Axis::Vertical => bounds.y,
      Axis::Horizontal => bounds.x,
    }
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Interaction {
  #[default]
  Idle,
  Dragging {
    axis: Axis,
    grab_offset: f32,
  },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Update {
  pub action: Option<Action>,
  pub capture_event: bool,
  pub request_redraw: bool,
}

impl Update {
  pub const fn ignored() -> Self {
    Self {
      action: None,
      capture_event: false,
      request_redraw: false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::widgets::scrollbar::{AxisSnapshot, Metrics, ScrollExtent};

  fn geometry() -> Geometry {
    Geometry::new(
      AxisSnapshot {
        axis: Axis::Vertical,
        offset: 0.0,
        viewport_len: 100.0,
        extent: ScrollExtent::Exact {
          content_len: 1000.0,
        },
      },
      iced::Rectangle {
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 300.0,
      },
      Metrics::default(),
    )
  }

  #[test]
  fn dragging_thumb_maps_pointer_position_to_scroll_offset() {
    let geometry = geometry();
    let thumb = geometry.thumb.expect("test geometry should have thumb");
    let mut state = State::default();

    let press = state.press(
      iced::Point::new(thumb.hit_bounds.x + 4.0, thumb.hit_bounds.y + 10.0),
      &geometry,
    );

    assert!(press.capture_event);
    assert!(press.request_redraw);
    assert_eq!(
      state.interaction(),
      Interaction::Dragging {
        axis: Axis::Vertical,
        grab_offset: 10.0,
      }
    );

    let drag = state.drag_to(
      iced::Point::new(
        thumb.hit_bounds.x + 4.0,
        geometry.track_bounds.y + thumb.travel_len + 10.0,
      ),
      &geometry,
    );

    assert_eq!(
      drag.action,
      Some(Action::DragTo {
        axis: Axis::Vertical,
        offset: thumb.max_scroll,
      })
    );
    assert!(drag.capture_event);

    let release = state.release();

    assert!(release.capture_event);
    assert_eq!(state.interaction(), Interaction::Idle);
  }
}
