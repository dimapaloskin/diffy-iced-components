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
use crate::layout_engine::LayoutEngine;
use crate::state::CodeViewState;

// TODO: use real content padding
const CONTENT_PADDING: f32 = 8.0;

pub struct CodeView {
  document: CodeDocument,
  width: Length,
  height: Length,
  font: iced::Font,
  font_size: f32,
  line_height: f32,
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

  fn layout(
    &mut self,
    tree: &mut iced::advanced::widget::Tree,
    _renderer: &Renderer,
    limits: &iced::advanced::layout::Limits,
  ) -> layout::Node {
    let state = tree.state.downcast_mut::<CodeViewState>();
    let resolved_size = limits.resolve(self.width, self.height, iced::Size::ZERO);
    let previous = state.line.take();

    let layout_request = LayoutRequest {
      document: &self.document,
      width: resolved_size.width,
      font: self.font,
      font_size: self.font_size,
      line_height: self.line_height,
    };

    let key = LayoutKey::from_request(&layout_request);

    let needs_rebuild = previous.as_ref().is_none_or(|p| p.key != key);

    state.line = if needs_rebuild {
      Some(LayoutEngine::rebuild(layout_request, previous))
    } else {
      previous
    };

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
    if let (Some(entry), Some(clip_bounds)) = (&state.line, bounds.intersection(viewport)) {
      let position = iced::Point::new(bounds.x + CONTENT_PADDING, bounds.y + CONTENT_PADDING);

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
