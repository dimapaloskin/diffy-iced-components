use std::sync::Arc;

use iced::Element;
use iced::Length;
use iced::advanced::graphics::text;
use iced::advanced::graphics::text::Renderer as TextRendererTrait;
use iced::advanced::widget;
use iced::advanced::{layout, renderer::Renderer as RendererTrait, widget::Widget};
use iced::mouse::ScrollDelta;

use crate::document::CodeDocument;
use crate::layout::LayoutConfig;
use crate::layout::LayoutKey;
use crate::layout::LayoutRequest;
use crate::layout_engine;
use crate::measurement::{MeasurementRequest, MeasurementResult};
use crate::scroll::ScrollExtent;
use crate::state::CodeViewState;
use crate::viewport::Viewport;

pub struct CodeView<Message> {
  document: CodeDocument,
  width: Length,
  height: Length,
  layout_config: LayoutConfig,
  padding: iced::padding::Padding,
  border_radius: iced::border::Radius,

  measurement_result: Option<MeasurementResult>,
  on_measure_request: fn(MeasurementRequest) -> Message,
}

impl<Message> CodeView<Message> {
  pub(crate) fn new(
    document: CodeDocument,
    on_measure_request: fn(MeasurementRequest) -> Message,
  ) -> Self {
    Self {
      document,
      width: Length::Fill,
      height: Length::Fill,
      layout_config: LayoutConfig::default(),
      padding: iced::padding::Padding::default(),
      border_radius: iced::border::Radius::default(),

      measurement_result: None,
      on_measure_request,
    }
  }

  pub(crate) fn width(mut self, width: Length) -> Self {
    self.width = width;
    self
  }

  pub(crate) fn height(mut self, height: Length) -> Self {
    self.height = height;
    self
  }

  pub(crate) fn layout_config(mut self, layout_config: LayoutConfig) -> Self {
    self.layout_config = layout_config;
    self
  }

  pub(crate) fn border_radius(mut self, border_radius: iced::border::Radius) -> Self {
    self.border_radius = border_radius;
    self
  }

  pub(crate) fn padding(mut self, padding: iced::padding::Padding) -> Self {
    self.padding = padding;
    self
  }

  pub(crate) fn measurement_result(
    mut self,
    measurement_result: Option<MeasurementResult>,
  ) -> Self {
    self.measurement_result = measurement_result;
    self
  }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CodeView<Message>
where
  Renderer: RendererTrait + TextRendererTrait,
{
  fn size(&self) -> iced::Size<iced::Length> {
    iced::Size::new(self.width, self.height)
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
    let resolved_size = limits.resolve(self.width, self.height, iced::Size::ZERO);
    let document_changed = state
      .layout_entry
      .as_ref()
      .is_some_and(|entry| entry.key.text_revision != self.document.id());

    // TODO: temporarily reset scroll offset if document changed.
    // In feature consider ability to reset scroll offset to previous value when file re-opened.
    let scroll_offset = if document_changed {
      iced::Vector::ZERO
    } else {
      state.viewport.scroll_offset
    };
    let viewport = Viewport::new(resolved_size, self.padding, scroll_offset);

    let measurement_request = MeasurementRequest::new(
      &self.document,
      self.layout_config,
      viewport.content_bounds.size().width,
    );

    let measurement_result = self
      .measurement_result
      .as_ref()
      .filter(|result| result.key == measurement_request.key);

    state.pending_measurement_request = if measurement_result.is_some() {
      None
    } else {
      Some(measurement_request)
    };

    let scroll_extent = ScrollExtent::new(
      &self.document,
      self.layout_config.wrap_mode,
      self.layout_config.line_height,
      measurement_result,
    );

    let scroll_offset =
      scroll_extent.clamp_offset(viewport.scroll_offset, viewport.content_bounds.size());

    let viewport = viewport.with_scroll_offset(scroll_offset);
    let previous = state.layout_entry.take();

    let layout_request = LayoutRequest {
      document: &self.document,
      content_size: viewport.content_bounds.size(),
      scroll_offset: viewport.scroll_offset,
      config: self.layout_config,
    };

    let key = LayoutKey::from_request(&layout_request);

    let needs_rebuild = previous.as_ref().is_none_or(|p| p.key != key);
    let needs_scroll_sync = previous
      .as_ref()
      .is_some_and(|p| p.prepared_scroll_offset != viewport.scroll_offset);

    state.layout_entry = if needs_rebuild || needs_scroll_sync {
      Some(layout_engine::rebuild_layout(layout_request, previous))
    } else {
      previous
    };

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
        radius: self.border_radius,
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

impl<Message> CodeView<Message> {
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
    // For web-like scroll chaining, move this inside the `state.viewport.scroll_offset != old` check
    shell.capture_event();

    let delta = match delta {
      ScrollDelta::Pixels { x, y } => [*x, *y],
      ScrollDelta::Lines { x, y } => {
        let step = self.layout_config.line_height * 3.0;
        [*x * step, *y * step]
      }
    };

    let state = tree.state.downcast_mut::<CodeViewState>();

    let old = state.viewport.scroll_offset;

    let candidate = iced::Vector::new(old.x - delta[0], old.y - delta[1]);
    state.viewport.scroll_offset = state
      .scroll_extent
      .clamp_offset(candidate, state.viewport.content_bounds.size());

    if state.viewport.scroll_offset != old {
      if let Some(entry) = &mut state.layout_entry {
        layout_engine::sync_scroll(entry, state.viewport.scroll_offset);
      }

      shell.request_redraw();
    }
  }

  fn on_redraw_requested(
    &mut self,
    tree: &mut widget::Tree,
    shell: &mut iced::advanced::Shell<'_, Message>,
  ) {
    let state = tree.state.downcast_mut::<CodeViewState>();

    let request = match &state.pending_measurement_request {
      Some(request) if state.last_published_measurement_key != Some(request.key) => {
        Some(request.clone())
      }
      _ => None,
    };

    if let Some(request) = request {
      state.last_published_measurement_key = Some(request.key);
      shell.publish((self.on_measure_request)(request));
    }
  }
}

impl<'a, Message, Theme, Renderer> From<CodeView<Message>> for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: 'a + RendererTrait + TextRendererTrait,
{
  fn from(code_view: CodeView<Message>) -> Self {
    Self::new(code_view)
  }
}
