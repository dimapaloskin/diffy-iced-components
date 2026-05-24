use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::widget;

use diffy_ui::Theme;
use diffy_ui::widgets::scrollbar;

use super::CodeViewInputs;
use super::effect::Effect;
use crate::state::CodeViewState;

pub(super) struct CodeViewScrollbars {
  metrics: scrollbar::Metrics,
}

impl CodeViewScrollbars {
  pub(super) fn from_inputs(_inputs: &CodeViewInputs) -> Self {
    Self {
      metrics: scrollbar::Metrics::default(),
    }
  }

  pub(super) fn right_chrome_reserve(&self) -> f32 {
    self.metrics.hit_thickness
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

    renderer.with_layer(widget_bounds, |renderer| {
      scrollbar::draw(renderer, &geometry, &style, state.scrollbar.status(), 1.0);
    });
  }

  pub(super) fn apply_update(
    &self,
    state: &mut CodeViewState,
    update: scrollbar::Update,
    geometry: Option<&scrollbar::Geometry>,
  ) -> Effect {
    let mut effect = Effect {
      capture_event: update.capture_event,
      request_redraw: update.request_redraw,
      invalidate_layout: false,
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

    let state = tree.state.downcast_mut::<CodeViewState>();
    let geometry = self.vertical_geometry(state, widget_bounds);
    let update = state.scrollbar.press(cursor_position, &geometry);

    self.apply_update(state, update, Some(&geometry))
  }

  pub(super) fn on_cursor_moved(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
    cursor: iced::advanced::mouse::Cursor,
  ) -> Effect {
    let state = tree.state.downcast_mut::<CodeViewState>();
    let geometry = self.vertical_geometry(state, widget_bounds);

    let update = if state.scrollbar.is_dragging() {
      let Some(cursor_position) = cursor.position() else {
        return Effect::none();
      };

      state.scrollbar.drag_to(cursor_position, &geometry)
    } else {
      state.scrollbar.cursor_moved(cursor.position(), &geometry)
    };

    self.apply_update(state, update, Some(&geometry))
  }

  pub(super) fn on_release(
    &self,
    tree: &mut widget::Tree,
    widget_bounds: iced::Rectangle,
    cursor: iced::advanced::mouse::Cursor,
  ) -> Effect {
    let state = tree.state.downcast_mut::<CodeViewState>();
    let geometry = self.vertical_geometry(state, widget_bounds);
    let update = state.scrollbar.release(cursor.position(), &geometry);

    self.apply_update(state, update, Some(&geometry))
  }
}
