use std::sync::Arc;

use iced::advanced::graphics::text::{self, Renderer as TextRendererTrait};
use iced::advanced::renderer::{Quad, Renderer as RendererTrait};

use super::CodeView;
use crate::state::CodeViewState;
use crate::viewport::ViewportArea;

impl<'a, Message> CodeView<'a, Message> {
  pub(super) fn draw_gutter<Renderer>(
    &self,
    state: &CodeViewState,
    renderer: &mut Renderer,
    widget_bounds: iced::Rectangle,
    viewport: &iced::Rectangle,
  ) where
    Renderer: RendererTrait + TextRendererTrait,
  {
    let Some(gutter) = state.viewport.absolute_gutter(widget_bounds) else {
      return;
    };

    let Some(background_clip_bounds) = gutter.bounds.intersection(viewport) else {
      return;
    };

    renderer.fill_quad(
      Quad {
        bounds: background_clip_bounds,
        border: iced::Border {
          color: iced::Color::TRANSPARENT,
          width: 0.0,
          radius: self.inputs.border_radius.top_right(0.0).bottom_right(0.0),
        },
        ..Quad::default()
      },
      self.inputs.style.gutter.background_color,
    );

    self.draw_gutter_separator(state, renderer, gutter.bounds, viewport);
    self.draw_gutter_labels(state, renderer, gutter, viewport);
  }

  fn draw_gutter_separator<Renderer>(
    &self,
    state: &CodeViewState,
    renderer: &mut Renderer,
    gutter_bounds: iced::Rectangle,
    viewport: &iced::Rectangle,
  ) where
    Renderer: RendererTrait,
  {
    let Some(entry) = state.gutter.entry() else {
      return;
    };

    let separator_width = entry.metrics.visible_separator_width(gutter_bounds.width);

    if separator_width <= 0.0 {
      return;
    }

    let separator_bounds = iced::Rectangle {
      x: gutter_bounds.x + gutter_bounds.width - separator_width,
      y: gutter_bounds.y,
      width: separator_width,
      height: gutter_bounds.height,
    };

    let Some(separator_bounds) = separator_bounds.intersection(viewport) else {
      return;
    };

    renderer.fill_quad(
      Quad {
        bounds: separator_bounds,
        border: iced::Border::default(),
        ..Quad::default()
      },
      self.inputs.style.gutter.separator_color,
    );
  }

  fn draw_gutter_labels<Renderer>(
    &self,
    state: &CodeViewState,
    renderer: &mut Renderer,
    gutter: ViewportArea,
    viewport: &iced::Rectangle,
  ) where
    Renderer: TextRendererTrait,
  {
    let Some(entry) = state.gutter.entry() else {
      return;
    };

    let Some(render_artifact) = entry.render_artifact.as_ref() else {
      return;
    };

    let Some(clip_bounds) = gutter.content_bounds.intersection(viewport) else {
      return;
    };

    let render_label_width = entry
      .metrics
      .render_label_width(gutter.content_bounds.width);
    if render_label_width <= 0.0 {
      return;
    }

    let x = gutter.content_bounds.x + entry.metrics.padding.left;
    let y = gutter.content_bounds.y + render_artifact.first_row_viewport_y;

    renderer.fill_raw(text::Raw {
      buffer: Arc::downgrade(render_artifact.payload.buffer()),
      position: iced::Point::new(x, y),
      color: self.inputs.style.gutter.line_number_color,
      clip_bounds,
    });
  }
}
