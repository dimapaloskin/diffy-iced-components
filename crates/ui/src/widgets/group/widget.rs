use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::{Alignment, Element, Event, Length, Rectangle, Size, Vector};

use super::Metrics;
use super::style::{Style, StyleFn};
use crate::theme::Theme;

pub(super) struct GroupWidget<'a, Message, Renderer = iced::Renderer> {
  children: Vec<Element<'a, Message, Theme, Renderer>>,
  width: Length,
  height: Length,
  pill: bool,
  separators: bool,
  metrics: Metrics,
  style: Option<StyleFn<'a>>,
}

impl<'a, Message, Renderer> GroupWidget<'a, Message, Renderer> {
  pub(super) fn new(
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    pill: bool,
    separators: bool,
    metrics: Metrics,
    style: Option<StyleFn<'a>>,
  ) -> Self {
    Self {
      children,
      width,
      height,
      pill,
      separators,
      metrics,
      style,
    }
  }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for GroupWidget<'_, Message, Renderer>
where
  Renderer: renderer::Renderer,
{
  fn children(&self) -> Vec<Tree> {
    self.children.iter().map(Tree::new).collect()
  }

  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(&self.children);
  }

  fn size(&self) -> Size<Length> {
    Size::new(self.width, self.height)
  }

  fn layout(
    &mut self,
    tree: &mut Tree,
    renderer: &Renderer,
    limits: &layout::Limits,
  ) -> layout::Node {
    let spacing = if self.separators {
      self.metrics.separator_width.max(0.0)
    } else {
      0.0
    };

    layout::flex::resolve(
      layout::flex::Axis::Horizontal, // TODO: Vertical support?
      renderer,
      limits,
      self.width,
      self.height,
      iced::Padding::from(self.metrics.frame_width.max(0.0)),
      spacing,
      Alignment::Center,
      &mut self.children,
      &mut tree.children,
    )
  }

  fn operate(
    &mut self,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    operation: &mut dyn Operation,
  ) {
    operation.container(None, layout.bounds());

    operation.traverse(&mut |operation| {
      for ((child, tree), layout) in self
        .children
        .iter_mut()
        .zip(&mut tree.children)
        .zip(layout.children())
      {
        child
          .as_widget_mut()
          .operate(tree, layout, renderer, operation);
      }
    });
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
    for ((child, tree), layout) in self
      .children
      .iter_mut()
      .zip(&mut tree.children)
      .zip(layout.children())
    {
      child
        .as_widget_mut()
        .update(tree, event, layout, cursor, renderer, shell, viewport);
    }
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
    if let Some(viewport) = layout.bounds().intersection(viewport) {
      for ((child, tree), layout) in self
        .children
        .iter()
        .zip(&tree.children)
        .zip(layout.children())
        .filter(|(_, layout)| layout.bounds().intersects(&viewport))
      {
        child
          .as_widget()
          .draw(tree, renderer, theme, style, layout, cursor, &viewport);
      }

      if self.children.is_empty() {
        return;
      }

      let base_style = Style::resolve(theme);
      let group_style = if let Some(style) = &self.style {
        style(theme, base_style)
      } else {
        base_style
      };

      self.draw_frame(renderer, theme, layout.bounds(), group_style);
      self.draw_separators(renderer, layout, group_style);
    }
  }

  fn mouse_interaction(
    &self,
    tree: &Tree,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
    renderer: &Renderer,
  ) -> mouse::Interaction {
    self
      .children
      .iter()
      .zip(&tree.children)
      .zip(layout.children())
      .map(|((child, tree), layout)| {
        child
          .as_widget()
          .mouse_interaction(tree, layout, cursor, viewport, renderer)
      })
      .max()
      .unwrap_or_default()
  }

  fn overlay<'b>(
    &'b mut self,
    tree: &'b mut Tree,
    layout: Layout<'b>,
    renderer: &Renderer,
    viewport: &Rectangle,
    translation: Vector,
  ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
    overlay::from_children(
      &mut self.children,
      tree,
      layout,
      renderer,
      viewport,
      translation,
    )
  }
}

impl<'a, Message, Renderer> GroupWidget<'a, Message, Renderer>
where
  Renderer: renderer::Renderer,
{
  fn draw_frame(&self, renderer: &mut Renderer, theme: &Theme, bounds: Rectangle, style: Style) {
    let frame_width = self.metrics.frame_width.max(0.0);

    if bounds.width <= 0.0 || bounds.height <= 0.0 || frame_width <= 0.0 {
      return;
    }

    let radius = if self.pill {
      theme.radius().pill
    } else {
      theme.radius().control
    };

    renderer.fill_quad(
      renderer::Quad {
        bounds,
        border: iced::Border {
          radius,
          width: frame_width,
          color: style.frame_color,
        },
        ..renderer::Quad::default()
      },
      iced::Background::Color(iced::Color::TRANSPARENT),
    );
  }

  fn draw_separators(&self, renderer: &mut Renderer, layout: Layout<'_>, style: Style) {
    let frame_width = self.metrics.frame_width.max(0.0);
    let separator_width = self.metrics.separator_width.max(0.0);

    if !self.separators || separator_width <= 0.0 {
      return;
    }

    let bounds = layout.bounds();
    let separator_height = (bounds.height - frame_width * 2.0).max(0.0);

    if separator_height <= 0.0 {
      return;
    }

    let y = bounds.y + frame_width;

    for child_layout in layout
      .children()
      .take(self.children.len().saturating_sub(1))
    {
      let child_bounds = child_layout.bounds();
      let x = child_bounds.x + child_bounds.width;

      renderer.fill_quad(
        renderer::Quad {
          bounds: Rectangle {
            x,
            y,
            width: separator_width,
            height: separator_height,
          },
          ..renderer::Quad::default()
        },
        iced::Background::Color(style.separator_color),
      );
    }
  }
}

impl<'a, Message, Renderer> From<GroupWidget<'a, Message, Renderer>>
  for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Renderer: renderer::Renderer + 'a,
{
  fn from(widget: GroupWidget<'a, Message, Renderer>) -> Self {
    Element::new(widget)
  }
}
