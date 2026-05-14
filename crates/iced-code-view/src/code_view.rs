mod gutter_draw;

use std::sync::Arc;

use iced::Element;
use iced::Length;
use iced::advanced::graphics::text;
use iced::advanced::graphics::text::Renderer as TextRendererTrait;
use iced::advanced::widget;
use iced::advanced::{layout, renderer::Renderer as RendererTrait, widget::Widget};
use iced::mouse::ScrollDelta;

use crate::document::Document;
use crate::gutter::{GutterConfig, GutterMetricsRequest, GutterRenderArtifactRequest};
use crate::insets::CodeViewInsets;
use crate::measurement::{MeasurementRequest, MeasurementResult};
use crate::scroll::{ScrollExtent, ScrollState};
use crate::state::{CodeViewState, ScrollChange};
use crate::style::CodeViewStyle;
use crate::text_layout::TextLayoutConfig;
use crate::text_layout::TextLayoutRequest;
use crate::viewport::Viewport;

pub(crate) struct CodeView<'a, Message> {
  inputs: CodeViewInputs<'a>,
  on_measure_request: fn(MeasurementRequest) -> Message,
}

pub(crate) struct CodeViewInputs<'a> {
  pub(crate) document: &'a Document,
  pub(crate) width: Length,
  pub(crate) height: Length,
  pub(crate) text_layout_config: TextLayoutConfig,
  pub(crate) insets: CodeViewInsets,
  pub(crate) border_radius: iced::border::Radius,
  pub(crate) gutter_config: GutterConfig,
  pub(crate) style: CodeViewStyle,
  pub(crate) measurement_result: Option<&'a MeasurementResult>,
}

impl<'a, Message> CodeView<'a, Message> {
  pub(crate) fn new(
    inputs: CodeViewInputs<'a>,
    on_measure_request: fn(MeasurementRequest) -> Message,
  ) -> Self {
    Self {
      inputs,
      on_measure_request,
    }
  }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CodeView<'a, Message>
where
  Renderer: RendererTrait + TextRendererTrait,
{
  fn size(&self) -> iced::Size<iced::Length> {
    iced::Size::new(self.inputs.width, self.inputs.height)
  }

  fn tag(&self) -> iced::advanced::widget::tree::Tag {
    widget::tree::Tag::of::<CodeViewState>()
  }

  fn state(&self) -> widget::tree::State {
    widget::tree::State::new(CodeViewState::default())
  }

  fn update(
    &mut self,
    tree: &mut widget::Tree,
    event: &iced::Event,
    layout: layout::Layout<'_>,
    cursor: iced::advanced::mouse::Cursor,
    _renderer: &Renderer,
    shell: &mut iced::advanced::Shell<'_, Message>,
    _viewport: &iced::Rectangle,
  ) {
    use iced::mouse::Event as MouseEvent;
    use iced::window::Event::RedrawRequested;

    match event {
      iced::Event::Mouse(MouseEvent::WheelScrolled { delta }) => {
        self.on_mouse_wheel(tree, delta, layout, cursor, shell);
      }
      iced::Event::Window(RedrawRequested(_)) => {
        self.on_redraw_requested(tree, shell);
      }
      _ => {}
    }
  }

  fn layout(
    &mut self,
    tree: &mut iced::advanced::widget::Tree,
    _renderer: &Renderer,
    limits: &iced::advanced::layout::Limits,
  ) -> layout::Node {
    let CodeViewInputs {
      document,
      width,
      height,
      insets,
      text_layout_config,
      gutter_config,
      measurement_result,
      ..
    } = self.inputs;

    let state = tree.state.downcast_mut::<CodeViewState>();
    let resolved_size = limits.resolve(width, height, iced::Size::ZERO);
    let document_changed = state
      .text_layout
      .visible_layout()
      .is_some_and(|entry| entry.key.document_revision != self.inputs.document.revision());

    let mut scroll = if document_changed {
      ScrollState::ZERO
    } else {
      state.scroll
    };

    let gutter_metrics = state.gutter.ensure_metrics(GutterMetricsRequest {
      document,
      text_layout_config,
      gutter_config,
    });

    let viewport = Viewport::new(resolved_size, insets, gutter_metrics);

    let measurement_request = MeasurementRequest::new(
      document,
      text_layout_config,
      viewport.text.content_bounds.size().width,
    );

    let measurement_result =
      state.update_pending_measurement(measurement_request, measurement_result);

    let scroll_extent = ScrollExtent::new(text_layout_config.wrap_mode, measurement_result);

    scroll.horizontal_px = scroll_extent
      .horizontal
      .clamp_offset(scroll.horizontal_px, viewport.scroll_viewport_size().width);

    let text_layout_request = TextLayoutRequest {
      document,
      content_size: viewport.text.content_bounds.size(),
      vertical_scroll: scroll.vertical,
      config: text_layout_config,
    };

    let gutter_size = viewport
      .gutter
      .map(|area| area.content_bounds.size())
      .unwrap_or(iced::Size::ZERO);

    let visible_text_layout = state.text_layout.ensure_visible_layout(text_layout_request);
    scroll.vertical = visible_text_layout.prepared_vertical_scroll;

    let gutter_render_artifact_request = GutterRenderArtifactRequest {
      text_layout_config,
      metrics: gutter_metrics,
      gutter_size,
      projection: &visible_text_layout.projection,
    };

    state
      .gutter
      .ensure_render_artifact(gutter_render_artifact_request);
    state.scroll = scroll;
    state.scroll_extent = scroll_extent;
    state.viewport = viewport;

    layout::Node::new(resolved_size)
  }

  fn draw(
    &self,
    tree: &iced::advanced::widget::Tree,
    renderer: &mut Renderer,
    _theme: &Theme,
    _style: &iced::advanced::renderer::Style,
    layout: iced::advanced::Layout<'_>,
    _cursor: iced::advanced::mouse::Cursor,
    viewport: &iced::Rectangle,
  ) {
    use iced::advanced::renderer::Quad;

    let state = tree.state.downcast_ref::<CodeViewState>();
    let bounds = layout.bounds();
    let Some(visible_bounds) = bounds.intersection(viewport) else {
      return;
    };

    let quad = Quad {
      bounds: visible_bounds,
      border: iced::Border {
        color: iced::Color::TRANSPARENT,
        width: 0.0,
        radius: self.inputs.border_radius,
      },
      ..Quad::default()
    };

    renderer.fill_quad(quad, self.inputs.style.background_color);

    self.draw_gutter(state, renderer, bounds, viewport);

    let text_content_bounds = state.viewport.absolute_text_content_bounds(bounds);
    if let (Some(entry), Some(clip_bounds)) = (
      state.text_layout.visible_layout(),
      text_content_bounds.intersection(viewport),
    ) {
      let position = iced::Point::new(
        text_content_bounds.x - state.scroll.horizontal_px,
        text_content_bounds.y,
      );

      renderer.fill_raw(text::Raw {
        buffer: Arc::downgrade(entry.payload.buffer()),
        position,
        color: self.inputs.style.text_color,
        clip_bounds,
      });
    }
  }
}

impl<'a, Message> CodeView<'a, Message> {
  fn scroll_delta_to_pixels(&self, delta: &ScrollDelta) -> iced::Vector {
    match delta {
      ScrollDelta::Pixels { x, y } => iced::Vector::new(*x, *y),
      ScrollDelta::Lines { x, y } => {
        // TODO: move magic number to const after reshaping file structure
        let step = self.inputs.text_layout_config.line_height * 3.0;
        iced::Vector::new(*x * step, *y * step)
      }
    }
  }

  fn on_mouse_wheel(
    &mut self,
    tree: &mut widget::Tree,
    delta: &ScrollDelta,
    layout: layout::Layout<'_>,
    cursor: iced::advanced::mouse::Cursor,
    shell: &mut iced::advanced::Shell<'_, Message>,
  ) {
    // Handle wheel over the whole CodeView area.
    // If text-only behavior is needed later, narrow this to the content bounds
    if !cursor.is_over(layout.bounds()) {
      return;
    }

    // Stop wheel events at CodeView, so scrolling does not chain to a parent at edges.
    // For web-like scroll chaining, capture only when `try_apply_wheel_delta` returns true.
    shell.capture_event();

    let delta = self.scroll_delta_to_pixels(delta);
    let state = tree.state.downcast_mut::<CodeViewState>();

    match state.try_apply_wheel_delta(delta) {
      Some(ScrollChange::RedrawOnly) => {
        shell.request_redraw();
      }
      Some(ScrollChange::RequiresLayout) => {
        shell.invalidate_layout();
        shell.request_redraw();
      }
      None => {}
    }
  }

  fn on_redraw_requested(
    &mut self,
    tree: &mut widget::Tree,
    shell: &mut iced::advanced::Shell<'_, Message>,
  ) {
    let state = tree.state.downcast_mut::<CodeViewState>();

    if let Some(request) = state.measurement_request_to_publish() {
      shell.publish((self.on_measure_request)(request));
    }
  }
}

impl<'a, Message, Theme, Renderer> From<CodeView<'a, Message>>
  for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: 'a + RendererTrait + TextRendererTrait,
{
  fn from(code_view: CodeView<'a, Message>) -> Self {
    Self::new(code_view)
  }
}
