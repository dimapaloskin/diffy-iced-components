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
use crate::gutter::{GutterConfig, GutterMeasureRequest, GutterRenderRequest};
use crate::layout::LayoutConfig;
use crate::layout::LayoutRequest;
use crate::measurement::{MeasurementRequest, MeasurementResult};
use crate::padding::CodeViewPadding;
use crate::scroll::ScrollExtent;
use crate::state::CodeViewState;
use crate::style::CodeViewStyle;
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
  pub(crate) padding: CodeViewPadding,
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
      padding,
      layout_config,
      gutter_config,
      measurement_result,
      ..
    } = self.inputs;

    let state = tree.state.downcast_mut::<CodeViewState>();
    let resolved_size = limits.resolve(width, height, iced::Size::ZERO);
    let document_changed = state
      .layout
      .entry()
      .is_some_and(|entry| entry.key.document_revision != self.inputs.document.revision());

    // TODO: temporarily reset scroll offset if document changed.
    // In feature consider ability to reset scroll offset to previous value when file re-opened.
    let scroll_offset = if document_changed {
      iced::Vector::ZERO
    } else {
      state.viewport.scroll_offset
    };

    let gutter_metrics = state.gutter.measure(GutterMeasureRequest {
      document,
      layout_config,
      gutter_config,
    });

    let viewport = Viewport::new(resolved_size, padding, gutter_metrics, scroll_offset);

    let measurement_request = MeasurementRequest::new(
      document,
      layout_config,
      viewport.text.content_bounds.size().width,
    );

    let measurement_result =
      state.update_pending_measurement(measurement_request, measurement_result);

    let scroll_extent = ScrollExtent::new(
      document,
      layout_config.wrap_mode,
      layout_config.line_height,
      measurement_result,
    );

    let scroll_offset =
      scroll_extent.clamp_offset(viewport.scroll_offset, viewport.scroll_viewport_size());

    let viewport = viewport.with_scroll_offset(scroll_offset);

    let layout_request = LayoutRequest {
      document,
      content_size: viewport.text.content_bounds.size(),
      scroll_offset: viewport.scroll_offset,
      config: layout_config,
    };

    let gutter_size = viewport
      .gutter
      .map(|area| area.content_bounds.size())
      .unwrap_or(iced::Size::ZERO);

    state.layout.refresh(layout_request);

    let layout_entry = state
      .layout
      .entry()
      .expect("layout entry is prepared by layout refresh");

    let gutter_render_request = GutterRenderRequest {
      layout_config,
      metrics: gutter_metrics,
      gutter_size,
      projection: &layout_entry.projection,
    };

    state.gutter.refresh(gutter_render_request);
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
      state.layout.entry(),
      text_content_bounds.intersection(viewport),
    ) {
      let position = iced::Point::new(
        text_content_bounds.x - state.viewport.scroll_offset.x,
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
