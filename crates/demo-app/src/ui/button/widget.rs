use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::text::Renderer as TextRendererTrait;
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{self, Operation, Tree, operation};
use iced::advanced::{Layout, Shell, Widget, layout, mouse, renderer};
use iced::{Alignment, Element, Event, Length, Rectangle, Size, keyboard, touch};

use crate::ui::theme::Theme;

use super::FocusRing;
use super::metrics::Metrics;
use super::style::{self, Class, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State {
  press: Option<Press>,
  focus: Focus,
  status: Status,
}

impl Default for State {
  fn default() -> Self {
    Self {
      press: None,
      focus: Focus::None,
      status: Status::Active,
    }
  }
}

impl operation::Focusable for State {
  fn is_focused(&self) -> bool {
    self.focus != Focus::None
  }

  fn focus(&mut self) {
    self.focus = Focus::Visible;
  }

  fn unfocus(&mut self) {
    self.focus = Focus::None;
    self.press = None;
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
  None,
  Pointer,
  Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
  Pointer,
  Keyboard,
}

pub(super) struct ButtonWidget<'a, Message, Renderer> {
  pub(super) content: Element<'a, Message, Theme, Renderer>,
  pub(super) metrics: Metrics,
  pub(super) width: Length,
  pub(super) height: Length,
  pub(super) class: Class<'a>,
  pub(super) on_press: Option<Message>,
  pub(super) main_content_index: usize,
}

impl<'a, Message, Renderer> ButtonWidget<'a, Message, Renderer> {
  fn status(&self, state: &State, cursor: mouse::Cursor, bounds: Rectangle) -> Status {
    if self.on_press.is_none() {
      Status::Disabled
    } else if matches!(state.press, Some(Press::Keyboard))
      || matches!(state.press, Some(Press::Pointer)) && cursor.is_over(bounds)
    {
      Status::Pressed
    } else if state.focus == Focus::Visible {
      Status::Focused
    } else if cursor.is_over(bounds) {
      Status::Hovered
    } else {
      Status::Active
    }
  }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for ButtonWidget<'a, Message, Renderer>
where
  Message: Clone + 'a,
  Renderer: RendererTrait + TextRendererTrait + 'a,
{
  fn tag(&self) -> widget::tree::Tag {
    widget::tree::Tag::of::<State>()
  }

  fn state(&self) -> widget::tree::State {
    widget::tree::State::new(State::default())
  }

  fn children(&self) -> Vec<Tree> {
    vec![Tree::new(&self.content)]
  }

  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(std::slice::from_ref(&self.content));
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
    layout::positioned(
      limits,
      self.width,
      self.height,
      self.metrics.padding,
      |limits| {
        self
          .content
          .as_widget_mut()
          .layout(&mut tree.children[0], renderer, &limits.loose())
      },
      |content, size| content.align(Alignment::Center, Alignment::Center, size),
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

    {
      let state = tree.state.downcast_mut::<State>();

      if self.on_press.is_some() {
        operation.focusable(None, layout.bounds(), state);
      } else {
        state.unfocus();
      }
    }

    operation.traverse(&mut |operation| {
      self.content.as_widget_mut().operate(
        &mut tree.children[0],
        layout.children().next().unwrap(),
        renderer,
        operation,
      );
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
    self.content.as_widget_mut().update(
      &mut tree.children[0],
      event,
      layout.children().next().unwrap(),
      cursor,
      renderer,
      shell,
      viewport,
    );

    let bounds = layout.bounds();
    let state = tree.state.downcast_mut::<State>();
    let cursor_is_over = cursor.is_over(bounds);

    match event {
      Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
      | Event::Touch(touch::Event::FingerPressed { .. }) => {
        state.focus = if self.on_press.is_some() && cursor_is_over {
          Focus::Pointer
        } else {
          Focus::None
        };
        state.press = None;
      }
      _ => {}
    }

    if shell.is_event_captured() {
      return;
    }

    match event {
      Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
      | Event::Touch(touch::Event::FingerPressed { .. })
        if self.on_press.is_some() && cursor_is_over =>
      {
        state.press = Some(Press::Pointer);
        shell.capture_event();
      }
      Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
      | Event::Touch(touch::Event::FingerLifted { .. })
        if state.press == Some(Press::Pointer) =>
      {
        state.press = None;

        if cursor_is_over && let Some(message) = &self.on_press {
          shell.publish(message.clone());
        }

        shell.capture_event();
      }
      Event::Touch(touch::Event::FingerLost { .. }) if state.press == Some(Press::Pointer) => {
        state.press = None;
        shell.capture_event();
      }
      Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(keyboard::key::Named::Enter),
        repeat,
        ..
      }) if self.on_press.is_some() && state.focus != Focus::None => {
        if !repeat && let Some(message) = &self.on_press {
          shell.publish(message.clone());
        }

        shell.capture_event();
      }
      Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(keyboard::key::Named::Space),
        repeat,
        ..
      }) if self.on_press.is_some() && state.focus != Focus::None => {
        if !repeat {
          state.press = Some(Press::Keyboard);
        }

        shell.capture_event();
      }
      Event::Keyboard(keyboard::Event::KeyReleased {
        key: keyboard::Key::Named(keyboard::key::Named::Space),
        ..
      }) if state.press == Some(Press::Keyboard) => {
        state.press = None;

        if state.focus != Focus::None
          && let Some(message) = &self.on_press
        {
          shell.publish(message.clone());
        }

        shell.capture_event();
      }
      _ => {}
    }

    let status = self.status(state, cursor, bounds);

    if state.status != status {
      state.status = status;
      shell.request_redraw();
    }
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    _style: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
  ) {
    let bounds = layout.bounds();
    let state = tree.state.downcast_ref::<State>();
    let status = self.status(state, cursor, bounds);
    let button_style = style::resolve(theme, &self.class, status);

    if let Some(focus_ring) = button_style.focus_ring {
      draw_focus_ring(renderer, bounds, button_style.border.radius, focus_ring);
    }

    if button_style.background.is_some()
      || button_style.border.width > 0.0
      || button_style.shadow != iced::Shadow::default()
    {
      renderer.fill_quad(
        renderer::Quad {
          bounds,
          border: button_style.border,
          shadow: button_style.shadow,
          ..renderer::Quad::default()
        },
        button_style
          .background
          .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
      );
    }

    self.content.as_widget().draw(
      &tree.children[0],
      renderer,
      theme,
      &renderer::Style {
        text_color: button_style.text_color,
      },
      layout.children().next().unwrap(),
      cursor,
      viewport,
    );

    if let Some(underline) = button_style.underline {
      let content_layout = layout.children().next().unwrap();
      let main_content_layout = content_layout.child(self.main_content_index);
      let content_bounds = visual_bounds(main_content_layout);
      let underline_bounds = Rectangle {
        x: content_bounds.x,
        y: content_bounds.y + content_bounds.height + underline.offset,
        width: content_bounds.width,
        height: underline.width,
      };

      renderer.fill_quad(
        renderer::Quad {
          bounds: underline_bounds,
          ..renderer::Quad::default()
        },
        iced::Background::Color(button_style.text_color),
      );
    }
  }
}

fn draw_focus_ring<Renderer>(
  renderer: &mut Renderer,
  bounds: Rectangle,
  radius: iced::border::Radius,
  focus_ring: FocusRing,
) where
  Renderer: RendererTrait,
{
  if focus_ring.width <= 0.0 {
    return;
  }

  let expansion = focus_ring.offset + focus_ring.width;
  let bounds = Rectangle {
    x: bounds.x - expansion,
    y: bounds.y - expansion,
    width: bounds.width + expansion * 2.0,
    height: bounds.height + expansion * 2.0,
  };

  renderer.fill_quad(
    renderer::Quad {
      bounds,
      border: iced::Border {
        radius: expand_radius(radius, expansion),
        width: focus_ring.width,
        color: focus_ring.color,
      },
      ..renderer::Quad::default()
    },
    iced::Background::Color(iced::Color::TRANSPARENT),
  );
}

fn expand_radius(radius: iced::border::Radius, amount: f32) -> iced::border::Radius {
  iced::border::Radius {
    top_left: radius.top_left + amount,
    top_right: radius.top_right + amount,
    bottom_right: radius.bottom_right + amount,
    bottom_left: radius.bottom_left + amount,
  }
}

fn visual_bounds(layout: Layout<'_>) -> Rectangle {
  if layout.children().len() == 1 {
    layout.child(0).bounds()
  } else {
    layout.bounds()
  }
}
