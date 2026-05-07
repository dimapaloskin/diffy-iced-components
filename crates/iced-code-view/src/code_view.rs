use iced::Element;
use iced::Length;
use iced::advanced::layout;
use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::widget::Widget;

pub struct CodeView {
  width: Length,
  height: Length,
  border_radius: iced::border::Radius,
}

impl Default for CodeView {
  fn default() -> Self {
    Self {
      width: Length::Fill,
      height: Length::Fill,
      border_radius: iced::border::Radius::default(),
    }
  }
}

impl CodeView {
  pub fn new() -> Self {
    Self::default()
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
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CodeView
where
  Renderer: RendererTrait,
{
  fn size(&self) -> iced::Size<iced::Length> {
    iced::Size::new(self.width, self.height)
  }

  fn layout(
    &mut self,
    _tree: &mut iced::advanced::widget::Tree,
    _renderer: &Renderer,
    limits: &iced::advanced::layout::Limits,
  ) -> layout::Node {
    let resolved_size = limits.resolve(self.width, self.height, iced::Size::ZERO);
    layout::Node::new(resolved_size)
  }

  fn draw(
    &self,
    _tree: &iced::advanced::widget::Tree,
    renderer: &mut Renderer,
    _theme: &Theme,
    _style: &iced::advanced::renderer::Style,
    layout: iced::advanced::Layout<'_>,
    _cursor: iced::advanced::mouse::Cursor,
    _viewport: &iced::Rectangle,
  ) {
    use iced::advanced::renderer::Quad;
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
  }
}

impl<'a, Message, Theme, Renderer> From<CodeView> for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: 'a + RendererTrait,
{
  fn from(code_view: CodeView) -> Self {
    Self::new(code_view)
  }
}
