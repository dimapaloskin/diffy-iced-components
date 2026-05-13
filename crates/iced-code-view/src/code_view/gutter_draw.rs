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
    let Some(gutter_area) = state.viewport.absolute_gutter(widget_bounds) else {
      return;
    };

    let Some(background_clip_bounds) = gutter_area.bounds.intersection(viewport) else {
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

    self.draw_gutter_separator(state, renderer, gutter_area, viewport);
    self.draw_gutter_labels(state, renderer, gutter_area, viewport);
  }

  fn draw_gutter_separator<Renderer>(
    &self,
    state: &CodeViewState,
    renderer: &mut Renderer,
    gutter_area: ViewportArea,
    viewport: &iced::Rectangle,
  ) where
    Renderer: RendererTrait,
  {
    let Some(measured_gutter) = state.gutter.measured() else {
      return;
    };

    let separator_width = measured_gutter
      .metrics
      .visible_separator_width(gutter_area.bounds.width);

    if separator_width <= 0.0 {
      return;
    }

    let separator_bounds = iced::Rectangle {
      x: gutter_area.bounds.x + gutter_area.bounds.width - separator_width,
      y: gutter_area.bounds.y,
      width: separator_width,
      height: gutter_area.bounds.height,
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
    gutter_area: ViewportArea,
    viewport: &iced::Rectangle,
  ) where
    Renderer: TextRendererTrait,
  {
    let Some(measured_gutter) = state.gutter.measured() else {
      return;
    };

    let Some(render_artifact) = measured_gutter.render_artifact.as_ref() else {
      return;
    };

    let Some(clip_bounds) = gutter_area.content_bounds.intersection(viewport) else {
      return;
    };

    let render_label_width = measured_gutter
      .metrics
      .render_label_width(gutter_area.content_bounds.width);
    if render_label_width <= 0.0 {
      return;
    }

    let x = gutter_area.content_bounds.x + measured_gutter.metrics.padding.horizontal.left;
    let y = gutter_area.content_bounds.y + render_artifact.first_row_viewport_y;

    renderer.fill_raw(text::Raw {
      buffer: Arc::downgrade(render_artifact.payload.buffer()),
      position: iced::Point::new(x, y),
      color: self.inputs.style.gutter.line_number_color,
      clip_bounds,
    });
  }
}
