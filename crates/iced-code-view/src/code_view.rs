use std::sync::Arc;

use iced::Element;
use iced::Length;
use iced::advanced::graphics::text;
use iced::advanced::graphics::text::Renderer as TextRendererTrait;
use iced::advanced::widget;
use iced::advanced::{layout, renderer::Renderer as RendererTrait, widget::Widget};

use crate::document::CodeDocument;
use crate::layout::LayoutKey;
use crate::layout::LayoutRequest;
use crate::layout::WrapMode;
use crate::layout_engine;
use crate::policies::TabDisplayPolicy;
use crate::state::CodeViewState;
use crate::viewport::Viewport;

pub struct CodeView {
  document: CodeDocument,
  width: Length,
  height: Length,
  font: iced::Font,
  font_size: f32,
  line_height: f32,
  wrap_mode: WrapMode,
  tab_display_policy: TabDisplayPolicy,
  padding: iced::padding::Padding,
  border_radius: iced::border::Radius,
}

impl CodeView {
  pub fn new(document: CodeDocument) -> Self {
    Self {
      document,
      width: Length::Fill,
      height: Length::Fill,
      font: iced::Font::MONOSPACE,
      font_size: 16.0,
      line_height: 24.0,
      wrap_mode: WrapMode::default(),
      tab_display_policy: TabDisplayPolicy::default(),
      padding: iced::padding::Padding::default(),
      border_radius: iced::border::Radius::default(),
    }
  }

  pub fn border_radius(mut self, border_radius: iced::border::Radius) -> Self {
    self.border_radius = border_radius;
    self
  }

  pub fn width(mut self, width: Length) -> Self {
    self.width = width;
    self
  }

  pub fn height(mut self, height: Length) -> Self {
    self.height = height;
    self
  }

  pub fn font(mut self, font: iced::Font) -> Self {
    self.font = font;
    self
  }

  pub fn font_size(mut self, font_size: f32) -> Self {
    self.font_size = font_size;
    self
  }

  pub fn line_height(mut self, line_height: f32) -> Self {
    self.line_height = line_height;
    self
  }

  pub fn padding(mut self, padding: iced::padding::Padding) -> Self {
    self.padding = padding;
    self
  }

  pub fn wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
    self.wrap_mode = wrap_mode;
    self
  }

  pub fn tab_display_policy(mut self, tab_display_policy: TabDisplayPolicy) -> Self {
    self.tab_display_policy = tab_display_policy;
    self
  }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CodeView
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
    use iced::mouse::{Event as MouseEvent, ScrollDelta};

    // Handle wheel over the whole CodeView area, including padding.
    // If text-only behavior is needed later, narrow this to the content bounds
    if !cursor.is_over(layout.bounds()) {
      return;
    }

    let iced::Event::Mouse(MouseEvent::WheelScrolled { delta }) = event else {
      return;
    };

    // Stop wheel events at CodeView, so scrolling does not chain to a parent at edges.
    // For web-like scroll chaining, move this inside the `state.viewport.scroll_offset != old` check
    shell.capture_event();

    let delta = match delta {
      ScrollDelta::Pixels { x, y } => [*x, *y],
      ScrollDelta::Lines { x, y } => {
        let step = self.line_height * 3.0;
        [*x * step, *y * step]
      }
    };

    let state = tree.state.downcast_mut::<CodeViewState>();

    let old = state.viewport.scroll_offset;

    state.viewport.scroll_offset.x = (old.x - delta[0]).max(0.0);
    state.viewport.scroll_offset.y = (old.y - delta[1]).max(0.0);

    if state.viewport.scroll_offset != old {
      shell.invalidate_layout();
      shell.request_redraw();
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
    let viewport = Viewport::new(resolved_size, self.padding, state.viewport.scroll_offset);
    let previous = state.line.take();

    let layout_request = LayoutRequest {
      document: &self.document,
      content_size: viewport.content_bounds.size(),
      scroll_offset: viewport.scroll_offset,
      font: self.font,
      font_size: self.font_size,
      line_height: self.line_height,
    };

    let key = LayoutKey::from_request(&layout_request);

    let needs_rebuild = previous.as_ref().is_none_or(|p| p.key != key);
    let needs_scroll_sync = previous
      .as_ref()
      .is_some_and(|p| p.scroll_offset != viewport.scroll_offset);

    state.line = if needs_rebuild || needs_scroll_sync {
      Some(layout_engine::rebuild(layout_request, previous))
    } else {
      previous
    };
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
    if let (Some(entry), Some(clip_bounds)) = (&state.line, content_bounds.intersection(viewport)) {
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

impl<'a, Message, Theme, Renderer> From<CodeView> for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: 'a + RendererTrait + TextRendererTrait,
{
  fn from(code_view: CodeView) -> Self {
    Self::new(code_view)
  }
}
