use iced::Point;
use iced::time::Instant;

use super::behavior;
use super::{Action, Axis, Geometry, Status, ThumbGeometry, TrackPressRegion};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct State {
  interaction: Interaction,
  last_activity_at: Option<Instant>,
}

impl State {
  pub fn status(&self) -> Status {
    match self.interaction {
      Interaction::Idle => Status::Idle,
      Interaction::Hover => Status::Hovered,
      Interaction::Dragging { .. } => Status::Pressed,
    }
  }

  pub fn is_visible(&self, geometry: &Geometry, behavior: behavior::Behavior) -> bool {
    if !geometry.is_renderable() {
      return false;
    }

    if behavior.visibility.always_visible {
      return true;
    }

    if self.last_activity_at.is_some() {
      return true;
    }

    match self.interaction {
      Interaction::Dragging { .. } => true,
      Interaction::Hover => behavior.visibility.reveal_on_hover,
      Interaction::Idle => false,
    }
  }

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

    if thumb.hit_bounds.contains(cursor_position) {
      return self.thumb_press(&thumb, cursor_position, geometry);
    }

    if geometry.track_bounds.contains(cursor_position) {
      return self.track_press(&thumb, cursor_position, geometry);
    }

    Update::ignored()
  }

  pub fn cursor_moved(
    &mut self,
    cursor_position: Option<Point>,
    geometry: &Geometry,
    now: Instant,
  ) -> Update {
    if self.is_dragging() {
      return Update::ignored();
    }

    let previous_interaction = self.interaction;
    let next_interaction = Self::hover_interaction(cursor_position, geometry);

    if next_interaction == previous_interaction {
      return Update::ignored();
    }

    self.interaction = next_interaction;

    if matches!(previous_interaction, Interaction::Hover)
      && matches!(next_interaction, Interaction::Idle)
      && self.last_activity_at.is_some()
    {
      self.last_activity_at = Some(now);
    }

    Update {
      action: None,
      capture_event: false,
      request_redraw: true,
      request_redraw_at: None,
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

    let thumb_start =
      axis.coord_of(cursor_position) - grab_offset - axis.start_of(geometry.track_bounds);

    let Some(offset) = thumb.offset_for_thumb_start(thumb_start) else {
      return Update::ignored();
    };

    Update {
      action: Some(Action::DragTo { axis, offset }),
      capture_event: true,
      request_redraw: true,
      request_redraw_at: None,
    }
  }

  pub fn note_activity(&mut self, now: Instant) -> Update {
    self.last_activity_at = Some(now);

    Update {
      request_redraw: true,
      ..Update::ignored()
    }
  }

  pub fn redraw_requested(&mut self, now: Instant, behavior: behavior::Behavior) -> Update {
    if behavior.visibility.always_visible {
      return Update::ignored();
    }

    let Some(last_activity_at) = self.last_activity_at else {
      return Update::ignored();
    };

    let hide_at = last_activity_at + behavior.visibility.hide_delay;

    if now < hide_at {
      return Update {
        request_redraw_at: Some(hide_at),
        ..Update::ignored()
      };
    }

    if matches!(
      self.interaction,
      Interaction::Hover | Interaction::Dragging { .. }
    ) {
      self.last_activity_at = Some(now);

      return Update {
        request_redraw_at: Some(now + behavior.visibility.hide_delay),
        ..Update::ignored()
      };
    }

    self.last_activity_at = None;

    Update {
      request_redraw: true,
      ..Update::ignored()
    }
  }

  pub fn release(&mut self, cursor_position: Option<Point>, geometry: &Geometry) -> Update {
    match self.interaction {
      Interaction::Idle | Interaction::Hover => Update::ignored(),
      Interaction::Dragging { .. } => {
        self.interaction = Self::hover_interaction(cursor_position, geometry);

        Update {
          action: None,
          capture_event: true,
          request_redraw: true,
          request_redraw_at: None,
        }
      }
    }
  }

  fn hover_interaction(cursor_position: Option<Point>, geometry: &Geometry) -> Interaction {
    let Some(cursor_position) = cursor_position else {
      return Interaction::Idle;
    };

    if geometry.is_renderable() && geometry.track_bounds.contains(cursor_position) {
      Interaction::Hover
    } else {
      Interaction::Idle
    }
  }

  fn thumb_press(
    &mut self,
    thumb: &ThumbGeometry,
    cursor_position: Point,
    geometry: &Geometry,
  ) -> Update {
    let axis = geometry.axis;

    let grab_offset = axis.coord_of(cursor_position) - axis.start_of(thumb.hit_bounds);

    self.interaction = Interaction::Dragging { axis, grab_offset };

    Update {
      action: None,
      capture_event: true,
      request_redraw: true,
      request_redraw_at: None,
    }
  }

  fn track_press(
    &mut self,
    thumb: &ThumbGeometry,
    cursor_position: Point,
    geometry: &Geometry,
  ) -> Update {
    let axis = geometry.axis;

    let track_len = axis.len_of(geometry.track_bounds);
    if track_len <= 0.0 {
      return Update::ignored();
    }

    let pointer_position = axis.coord_of(cursor_position);
    let track_start = axis.start_of(geometry.track_bounds);
    let pointer_offset = (pointer_position - track_start).clamp(0.0, track_len) as f64;
    let track_ratio = pointer_offset / track_len as f64;

    let region = if pointer_position < axis.start_of(thumb.hit_bounds) {
      TrackPressRegion::BeforeThumb
    } else {
      TrackPressRegion::AfterThumb
    };

    self.interaction = Interaction::Dragging {
      axis,
      grab_offset: thumb.len / 2.0,
    };

    Update {
      action: Some(Action::TrackPress {
        axis,
        region,
        track_ratio,
        pointer_offset,
      }),
      capture_event: true,
      request_redraw: true,
      request_redraw_at: None,
    }
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Interaction {
  #[default]
  Idle,
  Hover,
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
  pub request_redraw_at: Option<Instant>,
}

impl Update {
  pub const fn ignored() -> Self {
    Self {
      action: None,
      capture_event: false,
      request_redraw: false,
      request_redraw_at: None,
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

    let release = state.release(None, &geometry);

    assert!(release.capture_event);
    assert_eq!(state.interaction(), Interaction::Idle);
  }
}
