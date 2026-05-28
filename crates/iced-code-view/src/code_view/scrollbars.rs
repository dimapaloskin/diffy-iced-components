use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::widget;
use iced::time::Instant;

use diffy_ui::Theme;
use diffy_ui::widgets::scrollbar;

use super::CodeViewInputs;
use super::effect::Effect;
use crate::state::CodeViewState;

pub(super) struct CodeViewScrollbars {
  metrics: scrollbar::Metrics,
  behavior: scrollbar::Behavior,
}

impl CodeViewScrollbars {
  pub(super) fn from_inputs(_inputs: &CodeViewInputs) -> Self {
    Self {
      metrics: scrollbar::Metrics::default(),
      behavior: scrollbar::Behavior::default(),
    }
  }

  pub(super) fn right_chrome_reserve(&self) -> f32 {
    self.metrics.edge_reserve()
  }

  pub(super) fn vertical_geometry(
    &self,
    state: &CodeViewState,
    widget_bounds: iced::Rectangle,
  ) -> scrollbar::Geometry {
    let snapshot = state
      .scroll
      .vertical_scrollbar_snapshot(state.viewport.scroll_viewport_size().height);

    scrollbar::Geometry::new(snapshot, widget_bounds, self.metrics)
  }

  pub(super) fn draw_vertical_overlay<Renderer>(
    &self,
    state: &CodeViewState,
    renderer: &mut Renderer,
    theme: &Theme,
    widget_bounds: iced::Rectangle,
  ) where
    Renderer: RendererTrait,
  {
    let geometry = self.vertical_geometry(state, widget_bounds);
    let style = scrollbar::Style::resolve(theme);

    if !state.scrollbar.is_visible(&geometry, self.behavior) {
      return;
    }

    renderer.with_layer(widget_bounds, |renderer| {
      scrollbar::draw(renderer, &geometry, &style, state.scrollbar.status(), 1.0);
    });
  }

  pub(super) fn note_activity(&self, tree: &mut widget::Tree, now: Instant) -> Effect {
    let state = tree.state.downcast_mut::<CodeViewState>();
    let update = state.scrollbar.note_activity(now);

    self.apply_update(state, update, None)
  }

  pub(super) fn on_redraw_requested(&self, tree: &mut widget::Tree, now: Instant) -> Effect {
    let state = tree.state.downcast_mut::<CodeViewState>();
    let update = state.scrollbar.redraw_requested(now, self.behavior);

    self.apply_update(state, update, None)
  }

  fn apply_pointer_update(
    &self,
    state: &mut CodeViewState,
    update: scrollbar::Update,
    geometry: &scrollbar::Geometry,
    now: Instant,
  ) -> Effect {
    let captured = update.capture_event;
    let mut effect = self.apply_update(state, update, Some(geometry));

    if captured {
      let update = state.scrollbar.note_activity(now);
      effect.merge(self.apply_update(state, update, None));
    }

    effect
  }

  fn apply_update(
    &self,
    state: &mut CodeViewState,
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
    state: &mut CodeViewState,
    action: scrollbar::Action,
    geometry: Option<&scrollbar::Geometry>,
  ) -> Effect {
    match action.axis() {
      scrollbar::Axis::Vertical => self.apply_vertical_action(state, action, geometry),
      scrollbar::Axis::Horizontal => Effect::none(),
    }
  }

  fn apply_vertical_action(
    &self,
    state: &mut CodeViewState,
    action: scrollbar::Action,
    geometry: Option<&scrollbar::Geometry>,
  ) -> Effect {
    match action {
      scrollbar::Action::DragTo { offset, .. } => self.apply_vertical_offset(state, offset),
      scrollbar::Action::TrackPress { pointer_offset, .. } => {
        let Some(scrollbar::Geometry {
          thumb: Some(thumb), ..
        }) = geometry
        else {
          return Effect::none();
        };

        let target_thumb_start = pointer_offset as f32 - thumb.len / 2.0;
        let Some(offset) = thumb.offset_for_thumb_start(target_thumb_start) else {
          return Effect::none();
        };

        self.apply_vertical_offset(state, offset)
      }
    }
  }

  fn apply_vertical_offset(&self, state: &mut CodeViewState, offset: f64) -> Effect {
    Effect::from_scroll_change(state.scroll.set_vertical_offset_from_px(offset))
  }
}

impl CodeViewScrollbars {
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
    let state = tree.state.downcast_mut::<CodeViewState>();
    let geometry = self.vertical_geometry(state, widget_bounds);

    if !state.scrollbar.is_visible(&geometry, self.behavior) {
      return Effect::none();
    }

    let update = state.scrollbar.press(cursor_position, &geometry);
    self.apply_pointer_update(state, update, &geometry, now)
  }

  pub(super) fn on_cursor_moved(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
    cursor: iced::advanced::mouse::Cursor,
  ) -> Effect {
    let now = Instant::now();
    let state = tree.state.downcast_mut::<CodeViewState>();
    let geometry = self.vertical_geometry(state, widget_bounds);

    let update = if state.scrollbar.is_dragging() {
      let Some(cursor_position) = cursor.position() else {
        return Effect::none();
      };

      state.scrollbar.drag_to(cursor_position, &geometry)
    } else {
      state
        .scrollbar
        .cursor_moved(cursor.position(), &geometry, now)
    };

    self.apply_pointer_update(state, update, &geometry, now)
  }

  pub(super) fn on_cursor_left(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
  ) -> Effect {
    let now = Instant::now();
    let state = tree.state.downcast_mut::<CodeViewState>();
    let geometry = self.vertical_geometry(state, widget_bounds);

    let update = state.scrollbar.cursor_moved(None, &geometry, now);

    self.apply_pointer_update(state, update, &geometry, now)
  }

  pub(super) fn on_release(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
    cursor: iced::advanced::mouse::Cursor,
  ) -> Effect {
    let now = Instant::now();
    let state = tree.state.downcast_mut::<CodeViewState>();
    let geometry = self.vertical_geometry(state, widget_bounds);
    let update = state.scrollbar.release(cursor.position(), &geometry);

    self.apply_pointer_update(state, update, &geometry, now)
  }
}
