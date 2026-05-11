use std::sync::Arc;

use iced::Element;
use iced::Length;
use iced::advanced::graphics::text;
use iced::advanced::graphics::text::Renderer as TextRendererTrait;
use iced::advanced::widget;
use iced::advanced::{layout, renderer::Renderer as RendererTrait, widget::Widget};
use iced::mouse::ScrollDelta;

use crate::document::Document;
use crate::layout::LayoutConfig;
use crate::layout::LayoutRequest;
use crate::measurement::{MeasurementRequest, MeasurementResult};
use crate::scroll::ScrollExtent;
use crate::state::CodeViewState;
use crate::viewport::Viewport;

pub(crate) struct CodeView<'a, Message> {
  inputs: CodeViewInputs<'a>,
  on_measure_request: fn(MeasurementRequest) -> Message,
}

pub(crate) struct CodeViewInputs<'a> {
  pub(crate) document: &'a Document,
  pub(crate) width: Length,
  pub(crate) height: Length,
  pub(crate) layout_config: LayoutConfig,
  pub(crate) padding: iced::padding::Padding,
  pub(crate) border_radius: iced::border::Radius,
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
    let state = tree.state.downcast_mut::<CodeViewState>();
    let resolved_size = limits.resolve(self.inputs.width, self.inputs.height, iced::Size::ZERO);
    let document_changed = state
      .layout_entry
      .as_ref()
      .is_some_and(|entry| entry.key.text_revision != self.inputs.document.id());

    // TODO: temporarily reset scroll offset if document changed.
    // In feature consider ability to reset scroll offset to previous value when file re-opened.
    let scroll_offset = if document_changed {
      iced::Vector::ZERO
    } else {
      state.viewport.scroll_offset
    };

    let viewport = Viewport::new(resolved_size, self.inputs.padding, scroll_offset);

    let measurement_request = MeasurementRequest::new(
      self.inputs.document,
      self.inputs.layout_config,
      viewport.content_bounds.size().width,
    );

    let measurement_result =
      state.update_pending_measurement(measurement_request, self.inputs.measurement_result);

    let scroll_extent = ScrollExtent::new(
      self.inputs.document,
      self.inputs.layout_config.wrap_mode,
      self.inputs.layout_config.line_height,
      measurement_result,
    );

    let scroll_offset =
      scroll_extent.clamp_offset(viewport.scroll_offset, viewport.content_bounds.size());

    let viewport = viewport.with_scroll_offset(scroll_offset);

    let layout_request = LayoutRequest {
      document: self.inputs.document,
      content_size: viewport.content_bounds.size(),
      scroll_offset: viewport.scroll_offset,
      config: self.inputs.layout_config,
    };

    state.refresh_layout(layout_request);
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

    let quad = Quad {
      bounds: layout.bounds(),
      border: iced::Border {
        color: iced::Color::TRANSPARENT,
        width: 0.0,
        radius: self.inputs.border_radius,
      },
      ..Quad::default()
    };

    renderer.fill_quad(quad, iced::Color::BLACK);

    let bounds = layout.bounds();
    let content_bounds = state.viewport.absolute_content_bounds(bounds);
    if let (Some(entry), Some(clip_bounds)) =
      (&state.layout_entry, content_bounds.intersection(viewport))
    {
      let position = iced::Point::new(
        content_bounds.x - state.viewport.scroll_offset.x,
        content_bounds.y,
      );

      renderer.fill_raw(text::Raw {
        buffer: Arc::downgrade(entry.payload.buffer()),
        position,
        color: iced::Color::WHITE,
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
        let step = self.inputs.layout_config.line_height * 3.0;
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
    // Handle wheel over the whole CodeView area, including padding.
    // If text-only behavior is needed later, narrow this to the content bounds
    if !cursor.is_over(layout.bounds()) {
      return;
    }

    // Stop wheel events at CodeView, so scrolling does not chain to a parent at edges.
    // For web-like scroll chaining, capture only when `try_apply_wheel_delta` returns true.
    shell.capture_event();

    let delta = self.scroll_delta_to_pixels(delta);
    let state = tree.state.downcast_mut::<CodeViewState>();

    if state.try_apply_wheel_delta(delta) {
      shell.invalidate_layout();
      shell.request_redraw();
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
