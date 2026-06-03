use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::widget;
use iced::time::Instant;

use diffy_ui::Theme;
use diffy_ui::widgets::scrollbar;

use super::Inputs;
use super::effect::Effect;
use crate::state::State;

pub(super) struct Scrollbars {
  metrics: scrollbar::Metrics,
  behavior: scrollbar::Behavior,
}

impl Scrollbars {
  pub(super) fn from_inputs(_inputs: &Inputs) -> Self {
    Self {
      metrics: scrollbar::Metrics::default(),
      behavior: scrollbar::Behavior::default(),
    }
  }

  pub(super) fn right_chrome_reserve(&self) -> f32 {
    self.metrics.edge_reserve()
  }

  // TODO: replace this adapter override with per-axis scrollbar metrics.
  fn metrics_for_axis(&self, axis: scrollbar::Axis) -> scrollbar::Metrics {
    match axis {
      scrollbar::Axis::Vertical => self.metrics,
      scrollbar::Axis::Horizontal => scrollbar::Metrics {
        track_inset_start: 0.0,
        track_inset_end: 0.0,
        ..self.metrics
      },
    }
  }

  pub(super) fn geometry(
    &self,
    state: &State,
    axis: scrollbar::Axis,
    placement_bounds: iced::Rectangle,
  ) -> scrollbar::Geometry {
    let snapshot = state
      .scroll
      .scrollbar_snapshot(axis, state.viewport.scroll_viewport_size());

    scrollbar::Geometry::new(snapshot, placement_bounds, self.metrics_for_axis(axis))
  }

  fn axis_geometry(
    &self,
    state: &State,
    axis: scrollbar::Axis,
    widget_bounds: iced::Rectangle,
  ) -> scrollbar::Geometry {
    let placement_bounds = self.placement_bounds(state, axis, widget_bounds);
    self.geometry(state, axis, placement_bounds)
  }

  fn placement_bounds(
    &self,
    state: &State,
    axis: scrollbar::Axis,
    widget_bounds: iced::Rectangle,
  ) -> iced::Rectangle {
    match axis {
      scrollbar::Axis::Vertical => widget_bounds,
      scrollbar::Axis::Horizontal => state.viewport.absolute_text_bounds(widget_bounds),
    }
  }

  fn target_axis_at(
    &self,
    state: &State,
    widget_bounds: iced::Rectangle,
    cursor_position: iced::Point,
  ) -> Option<(scrollbar::Axis, scrollbar::Geometry)> {
    for axis in [scrollbar::Axis::Horizontal, scrollbar::Axis::Vertical] {
      let geometry = self.axis_geometry(state, axis, widget_bounds);

      let is_visible = state
        .scrollbars
        .for_axis(axis)
        .is_visible(&geometry, self.behavior);
      let is_hovering = geometry.track_bounds.contains(cursor_position);

      if is_visible && is_hovering {
        return Some((axis, geometry));
      }
    }

    None
  }

  fn dragging_axis(&self, state: &State) -> Option<scrollbar::Axis> {
    [scrollbar::Axis::Horizontal, scrollbar::Axis::Vertical]
      .into_iter()
      .find(|&axis| state.scrollbars.for_axis(axis).is_dragging())
  }

  pub(super) fn draw_overlays<Renderer>(
    &self,
    state: &State,
    renderer: &mut Renderer,
    theme: &Theme,
    widget_bounds: iced::Rectangle,
  ) where
    Renderer: RendererTrait,
  {
    for axis in [scrollbar::Axis::Vertical, scrollbar::Axis::Horizontal] {
      let placement_bounds = self.placement_bounds(state, axis, widget_bounds);

      self.draw_axis_overlay(
        state,
        renderer,
        theme,
        axis,
        placement_bounds,
        widget_bounds,
      );
    }
  }

  fn draw_axis_overlay<Renderer>(
    &self,
    state: &State,
    renderer: &mut Renderer,
    theme: &Theme,
    axis: scrollbar::Axis,
    placement_bounds: iced::Rectangle,
    layer_bounds: iced::Rectangle,
  ) where
    Renderer: RendererTrait,
  {
    let geometry = self.geometry(state, axis, placement_bounds);
    let style = scrollbar::Style::resolve(theme);
    let scrollbar_state = state.scrollbars.for_axis(axis);

    if !scrollbar_state.is_visible(&geometry, self.behavior) {
      return;
    }

    renderer.with_layer(layer_bounds, |renderer| {
      scrollbar::draw(renderer, &geometry, &style, scrollbar_state.status(), 1.0);
    });
  }

  pub(super) fn note_activity(
    &self,
    tree: &mut widget::Tree,
    axis: scrollbar::Axis,
    now: Instant,
  ) -> Effect {
    let state = tree.state.downcast_mut::<State>();
    let update = state.scrollbars.for_axis_mut(axis).note_activity(now);

    self.apply_update(state, update, None)
  }

  pub(super) fn on_redraw_requested(&self, tree: &mut widget::Tree, now: Instant) -> Effect {
    let state = tree.state.downcast_mut::<State>();
    let mut effect = Effect::none();

    for axis in [scrollbar::Axis::Vertical, scrollbar::Axis::Horizontal] {
      let update = state
        .scrollbars
        .for_axis_mut(axis)
        .redraw_requested(now, self.behavior);

      effect.merge(self.apply_update(state, update, None));
    }

    effect
  }

  fn apply_pointer_update(
    &self,
    state: &mut State,
    update: scrollbar::Update,
    geometry: &scrollbar::Geometry,
    now: Instant,
  ) -> Effect {
    let captured = update.capture_event;
    let mut effect = self.apply_update(state, update, Some(geometry));

    // A captured scrollbar event means the user interacted with it,
    // so auto-hide activity is refreshed too.
    if captured {
      let update = state
        .scrollbars
        .for_axis_mut(geometry.axis)
        .note_activity(now);
      effect.merge(self.apply_update(state, update, None));
    }

    effect
  }

  fn apply_update(
    &self,
    state: &mut State,
    update: scrollbar::Update,
    geometry: Option<&scrollbar::Geometry>,
  ) -> Effect {
    let mut effect = Effect {
      capture_event: update.capture_event,
      request_redraw: update.request_redraw,
      invalidate_layout: false,
      request_redraw_at: update.request_redraw_at,
    };

    if let Some(action) = update.action {
      effect.merge(self.apply_action(state, action, geometry));
    }

    effect
  }

  fn apply_action(
    &self,
    state: &mut State,
    action: scrollbar::Action,
    geometry: Option<&scrollbar::Geometry>,
  ) -> Effect {
    match action {
      scrollbar::Action::DragTo { axis, offset } => self.apply_axis_offset(state, axis, offset),
      scrollbar::Action::TrackPress {
        axis,
        pointer_offset,
        ..
      } => self.apply_track_press(state, pointer_offset, axis, geometry),
    }
  }

  fn apply_axis_offset(&self, state: &mut State, axis: scrollbar::Axis, offset: f64) -> Effect {
    Effect::from_scroll_change(state.scroll.set_scrollbar_offset(
      axis,
      offset,
      state.viewport.scroll_viewport_size(),
    ))
  }

  fn apply_track_press(
    &self,
    state: &mut State,
    pointer_offset: f64,
    axis: scrollbar::Axis,
    geometry: Option<&scrollbar::Geometry>,
  ) -> Effect {
    let Some(geometry) = geometry else {
      return Effect::none();
    };

    if geometry.axis != axis {
      return Effect::none();
    }

    let Some(thumb) = geometry.thumb else {
      return Effect::none();
    };

    let target_thumb_start = pointer_offset as f32 - thumb.len / 2.0;
    let Some(target_thumb_start) = thumb.clamp_thumb_start(target_thumb_start) else {
      return Effect::none();
    };

    let Some(offset) = thumb.offset_for_thumb_start(target_thumb_start) else {
      return Effect::none();
    };

    let grab_offset = pointer_offset as f32 - target_thumb_start;

    let mut effect = Effect::from_scroll_change(state.scroll.set_scrollbar_offset(
      axis,
      offset,
      state.viewport.scroll_viewport_size(),
    ));

    let update = state
      .scrollbars
      .for_axis_mut(axis)
      .begin_drag(axis, grab_offset);

    effect.merge(self.apply_update(state, update, None));

    effect
  }
}

impl Scrollbars {
  pub(super) fn on_press(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
    cursor: iced::advanced::mouse::Cursor,
  ) -> Effect {
    let Some(cursor_position) = cursor.position() else {
      return Effect::none();
    };

    let now = Instant::now();
    let state = tree.state.downcast_mut::<State>();
    let Some((axis, geometry)) = self.target_axis_at(state, widget_bounds, cursor_position) else {
      return Effect::none();
    };

    let update = state
      .scrollbars
      .for_axis_mut(axis)
      .press(cursor_position, &geometry);

    self.apply_pointer_update(state, update, &geometry, now)
  }

  pub(super) fn on_cursor_moved(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
    cursor: iced::advanced::mouse::Cursor,
  ) -> Effect {
    let now = Instant::now();
    let cursor_position = cursor.position();
    let state = tree.state.downcast_mut::<State>();

    if let Some(axis) = self.dragging_axis(state) {
      let Some(cursor_position) = cursor_position else {
        return Effect::none();
      };

      let geometry = self.axis_geometry(state, axis, widget_bounds);
      let update = state
        .scrollbars
        .for_axis_mut(axis)
        .drag_to(cursor_position, &geometry);

      return self.apply_pointer_update(state, update, &geometry, now);
    }

    let mut effect = Effect::none();

    for axis in [scrollbar::Axis::Horizontal, scrollbar::Axis::Vertical] {
      let geometry = self.axis_geometry(state, axis, widget_bounds);
      let update =
        state
          .scrollbars
          .for_axis_mut(axis)
          .cursor_moved(cursor_position, &geometry, now);

      effect.merge(self.apply_pointer_update(state, update, &geometry, now));
    }

    effect
  }

  pub(super) fn on_cursor_left(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
  ) -> Effect {
    let now = Instant::now();
    let state = tree.state.downcast_mut::<State>();
    let mut effect = Effect::none();

    for axis in [scrollbar::Axis::Horizontal, scrollbar::Axis::Vertical] {
      let geometry = self.axis_geometry(state, axis, widget_bounds);
      let update = state
        .scrollbars
        .for_axis_mut(axis)
        .cursor_moved(None, &geometry, now);

      effect.merge(self.apply_pointer_update(state, update, &geometry, now));
    }

    effect
  }

  pub(super) fn on_release(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
    cursor: iced::advanced::mouse::Cursor,
  ) -> Effect {
    let now = Instant::now();
    let state = tree.state.downcast_mut::<State>();
    let mut effect = Effect::none();

    for axis in [scrollbar::Axis::Horizontal, scrollbar::Axis::Vertical] {
      let geometry = self.axis_geometry(state, axis, widget_bounds);
      let update = state
        .scrollbars
        .for_axis_mut(axis)
        .release(cursor.position(), &geometry);

      effect.merge(self.apply_pointer_update(state, update, &geometry, now));
    }

    effect
  }
}
