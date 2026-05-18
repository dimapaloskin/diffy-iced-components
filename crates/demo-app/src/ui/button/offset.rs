use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::widget::{self, Tree};
use iced::advanced::{Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use crate::ui::theme::Theme;

pub(super) fn y<'a, Message, Renderer>(
  content: impl Into<Element<'a, Message, Theme, Renderer>>,
  offset: f32,
) -> Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Renderer: RendererTrait + 'a,
{
  let content = content.into();

  if offset == 0.0 {
    return content;
  }

  Element::new(YOffset { content, offset })
}

struct YOffset<'a, Message, Renderer> {
  content: Element<'a, Message, Theme, Renderer>,
  offset: f32,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for YOffset<'_, Message, Renderer>
where
  Renderer: RendererTrait,
{
  fn children(&self) -> Vec<Tree> {
    vec![Tree::new(&self.content)]
  }

  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(std::slice::from_ref(&self.content));
  }

  fn size(&self) -> Size<Length> {
    self.content.as_widget().size()
  }

  fn layout(
    &mut self,
    tree: &mut Tree,
    renderer: &Renderer,
    limits: &layout::Limits,
  ) -> layout::Node {
    let child = self
      .content
      .as_widget_mut()
      .layout(&mut tree.children[0], renderer, limits);
    let size = child.size();
    let child = child.translate(Vector::new(0.0, self.offset));

    layout::Node::with_children(size, vec![child])
  }

  fn operate(
    &mut self,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    operation: &mut dyn widget::Operation,
  ) {
    self.content.as_widget_mut().operate(
      &mut tree.children[0],
      layout.children().next().unwrap(),
      renderer,
      operation,
    );
  }

  fn update(
    &mut self,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
  ) {
    self.content.as_widget_mut().update(
      &mut tree.children[0],
      event,
      layout.children().next().unwrap(),
      cursor,
      renderer,
      shell,
      viewport,
    );
  }

  fn mouse_interaction(
    &self,
    tree: &Tree,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
    renderer: &Renderer,
  ) -> mouse::Interaction {
    self.content.as_widget().mouse_interaction(
      &tree.children[0],
      layout.children().next().unwrap(),
      cursor,
      viewport,
      renderer,
    )
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
  ) {
    self.content.as_widget().draw(
      &tree.children[0],
      renderer,
      theme,
      style,
      layout.children().next().unwrap(),
      cursor,
      viewport,
    );
  }

  fn overlay<'a>(
    &'a mut self,
    tree: &'a mut Tree,
    layout: Layout<'a>,
    renderer: &Renderer,
    viewport: &Rectangle,
    translation: Vector,
  ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
    self.content.as_widget_mut().overlay(
      &mut tree.children[0],
      layout.children().next().unwrap(),
      renderer,
      viewport,
      translation,
    )
  }
}
