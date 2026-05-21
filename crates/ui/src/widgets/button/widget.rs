use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{self, Operation, Tree};
use iced::advanced::{Layout, Widget, layout, mouse, overlay, renderer};
use iced::widget::row;
use iced::{Alignment, Element, Event, Length, Rectangle, Size, Vector, keyboard, touch};

use crate::icon;
use crate::theme::Theme;
use crate::widgets::group::{GroupContext, GroupPosition};

use super::metrics::Metrics;
use super::style::{Mode, Status, Style, StyleFn, Variant};
use super::{Button, Content};

pub(super) struct ButtonWidget<'a, Message, Renderer> {
  content: Element<'a, Message, Theme, Renderer>,
  metrics: Metrics,
  width: Length,
  height: Length,
  variant: Variant,
  mode: Mode,
  on_press: Option<Message>,
  style: Option<StyleFn<'a>>,
  icon_only: bool,
  pill: bool,
  grouped: Option<GroupContext>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Focus {
  #[default]
  None,
  Pointer,
  Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
  Pointer,
  Keyboard,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct State {
  press: Option<Press>,
  status: Status,
  focus: Focus,
}

impl State {
  fn is_pressed(&self) -> bool {
    self.press.is_some()
  }

  fn is_pressed_pointer(&self) -> bool {
    self.press == Some(Press::Pointer)
  }

  fn is_pressed_keyboard(&self) -> bool {
    self.press == Some(Press::Keyboard)
  }
}

impl<'a, Message, Renderer> ButtonWidget<'a, Message, Renderer> {
  fn status(&self, state: &State, cursor: mouse::Cursor, bounds: Rectangle) -> Status {
    if self.on_press.is_none() {
      Status::Disabled
    } else if state.is_pressed_keyboard() || (state.is_pressed_pointer() && cursor.is_over(bounds))
    {
      Status::Pressed
    } else if cursor.is_over(bounds) {
      Status::Hovered
    } else {
      Status::Active
    }
  }

  fn radius(&self, theme: &Theme) -> iced::border::Radius {
    let standalone_radius = if self.pill {
      theme.radius().pill
    } else {
      theme.radius().control
    };

    self
      .grouped
      .map(|context| context.radius_for(theme))
      .unwrap_or(standalone_radius)
  }

  fn resolved_style(&self, theme: &Theme, status: Status) -> Style {
    let base_style = Style::resolve(theme, self.variant, self.mode, status);

    let mut button_style = if let Some(style) = &self.style {
      style(theme, status, base_style)
    } else {
      base_style
    };

    button_style.border.radius = self.radius(theme);

    if self.grouped.is_some() {
      button_style.border.width = 0.0;
      button_style.border.color = iced::Color::TRANSPARENT;
      button_style.shadow = theme.shadows().none;
    }

    button_style
  }

  fn focused_style(&self, theme: &Theme, status: Status) -> Style {
    let mut button_style = self.resolved_style(theme, status);

    if button_style.focus_ring.is_none() {
      button_style = button_style.focused(theme);
    }

    button_style
  }

  fn background_bounds(&self, bounds: Rectangle) -> Rectangle {
    let Some(context) = self.grouped else {
      return bounds;
    };

    let overlap = context.frame_width().clamp(0.0, 1.0);

    if overlap <= 0.0 {
      return bounds;
    }

    let mut bounds = Rectangle {
      y: bounds.y - overlap,
      height: bounds.height + overlap * 2.0,
      ..bounds
    };

    match context.position() {
      GroupPosition::Only => {
        bounds.x -= overlap;
        bounds.width += overlap * 2.0;
      }
      GroupPosition::First => {
        bounds.x -= overlap;
        bounds.width += overlap;
      }
      GroupPosition::Middle => {}
      GroupPosition::Last => {
        bounds.width += overlap;
      }
    }

    bounds
  }
}

impl<'a, Message, Renderer> From<Button<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
  Message: Clone + 'a,
  Renderer: RendererTrait + TextRenderer + 'a,
  <Renderer as TextRenderer>::Font: From<iced::Font>,
{
  fn from(button: Button<'a, Message>) -> Self {
    let has_extra_icons = button.leading_icon.is_some() || button.trailing_icon.is_some();
    let icon_only = matches!(button.content, Content::Icon(_)) && !has_extra_icons;

    let main_content: Element<'a, Message, Theme, Renderer> = match button.content {
      Content::Label(label) => iced::widget::text(label)
        .size(button.metrics.font_size)
        .align_y(Alignment::Center)
        .into(),
      Content::Icon(icon) => icon::text(icon).size(button.metrics.icon_size).into(),
    };

    let content = if has_extra_icons {
      let mut row = row![]
        .spacing(button.metrics.gap)
        .align_y(Alignment::Center);

      if let Some(icon) = button.leading_icon {
        row = row.push(icon::text(icon).size(button.metrics.icon_size));
      }

      row = row.push(main_content);

      if let Some(icon) = button.trailing_icon {
        row = row.push(icon::text(icon).size(button.metrics.icon_size));
      }

      row.into()
    } else {
      main_content
    };

    Element::new(ButtonWidget {
      content,
      metrics: button.metrics,
      width: button.width,
      height: button.height,
      variant: button.variant,
      mode: button.mode,
      on_press: button.on_press,
      style: button.style,
      icon_only,
      pill: button.pill,
      grouped: button.grouped,
    })
  }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for ButtonWidget<'a, Message, Renderer>
where
  Message: Clone + 'a,
  Renderer: RendererTrait + TextRenderer + 'a,
{
  fn children(&self) -> Vec<Tree> {
    vec![Tree::new(&self.content)]
  }

  fn diff(&self, tree: &mut Tree) {
    tree.diff_children(std::slice::from_ref(&self.content));
  }

  fn size(&self) -> Size<Length> {
    Size::new(self.width, self.height)
  }

  fn tag(&self) -> iced::advanced::widget::tree::Tag {
    widget::tree::Tag::of::<State>()
  }

  fn state(&self) -> widget::tree::State {
    widget::tree::State::new(State::default())
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
        layout.children().next().expect("content layout not found"),
        renderer,
        operation,
      );
    });
  }

  fn layout(
    &mut self,
    tree: &mut Tree,
    renderer: &Renderer,
    limits: &layout::Limits,
  ) -> layout::Node {
    let padding = if self.icon_only {
      self.metrics.icon_only_padding()
    } else {
      self.metrics.padding
    };

    layout::positioned(
      limits,
      self.width,
      self.height,
      padding,
      |limits| {
        self
          .content
          .as_widget_mut()
          .layout(&mut tree.children[0], renderer, &limits.loose())
      },
      |content, size| content.align(Alignment::Center, Alignment::Center, size),
    )
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    _: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
  ) {
    let state = tree.state.downcast_ref::<State>();
    let bounds = layout.bounds();
    let status = self.status(state, cursor, bounds);
    let button_style = self.resolved_style(theme, status);
    let background_bounds = self.background_bounds(bounds);

    if button_style.background.is_some()
      || button_style.border.width > 0.0
      || button_style.shadow != iced::Shadow::default()
    {
      renderer.fill_quad(
        renderer::Quad {
          bounds: background_bounds,
          border: button_style.border,
          shadow: button_style.shadow,
          ..renderer::Quad::default()
        },
        button_style
          .background
          .unwrap_or(iced::Color::TRANSPARENT.into()),
      );
    }

    self.content.as_widget().draw(
      &tree.children[0],
      renderer,
      theme,
      &renderer::Style {
        text_color: button_style.text_color,
      },
      layout.children().next().expect("content layout not found"),
      cursor,
      viewport,
    )
  }

  fn overlay<'b>(
    &'b mut self,
    tree: &'b mut Tree,
    layout: Layout<'b>,
    _renderer: &Renderer,
    _viewport: &Rectangle,
    translation: Vector,
  ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
    let state = tree.state.downcast_ref::<State>();

    if state.focus != Focus::Visible || self.on_press.is_none() {
      return None;
    }

    let mut bounds = layout.bounds();
    bounds.x += translation.x;
    bounds.y += translation.y;

    Some(overlay::Element::new(Box::new(FocusRingOverlay {
      button: self,
      status: state.status,
      bounds,
    })))
  }

  fn update(
    &mut self,
    tree: &mut Tree,
    event: &iced::Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    shell: &mut iced::advanced::Shell<'_, Message>,
    viewport: &Rectangle,
  ) {
    use keyboard::Event::{KeyPressed, KeyReleased};
    use keyboard::Key::Named;
    use keyboard::key::Named::{Enter as KeyEnter, Space as KeySpace};
    use mouse::Button;
    use mouse::Event::{ButtonPressed, ButtonReleased};
    use touch::Event::{FingerLifted, FingerLost, FingerPressed};

    let content_layout = layout.children().next().expect("content layout not found");

    self.content.as_widget_mut().update(
      &mut tree.children[0],
      event,
      content_layout,
      cursor,
      renderer,
      shell,
      viewport,
    );

    let bounds = layout.bounds();
    let cursor_is_over = cursor.is_over(bounds);
    let state = tree.state.downcast_mut::<State>();

    let previous_status = state.status;
    let previous_focus = state.focus;

    match event {
      Event::Mouse(ButtonPressed(Button::Left)) | Event::Touch(FingerPressed { .. }) => {
        state.focus = if self.on_press.is_some() && cursor_is_over {
          Focus::Pointer
        } else {
          Focus::None
        };
      }
      _ => {}
    }

    if !shell.is_event_captured() {
      match event {
        Event::Mouse(ButtonPressed(Button::Left)) | Event::Touch(FingerPressed { .. })
          if self.on_press.is_some() && cursor_is_over =>
        {
          state.press = Some(Press::Pointer);
          shell.capture_event();
        }
        Event::Mouse(ButtonReleased(Button::Left)) | Event::Touch(FingerLifted { .. })
          if state.is_pressed() =>
        {
          state.press = None;

          if cursor_is_over && let Some(message) = &self.on_press {
            shell.publish(message.clone());
          }

          shell.capture_event();
        }
        Event::Touch(FingerLost { .. }) if state.is_pressed() => {
          state.press = None;
          shell.capture_event();
        }
        Event::Keyboard(KeyPressed {
          key: Named(KeyEnter),
          repeat,
          ..
        }) if self.on_press.is_some() && state.focus != Focus::None => {
          if !repeat && let Some(message) = &self.on_press {
            shell.publish(message.clone());
          }

          shell.capture_event();
        }
        Event::Keyboard(KeyPressed {
          key: Named(KeySpace),
          repeat,
          ..
        }) if self.on_press.is_some() && state.focus != Focus::None => {
          if !repeat {
            state.press = Some(Press::Keyboard);
          }

          shell.capture_event();
        }
        Event::Keyboard(KeyReleased {
          key: Named(KeySpace),
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
    }

    let status = self.status(state, cursor, bounds);
    if previous_status != status || previous_focus != state.focus {
      state.status = status;
      shell.request_redraw();
    }
  }
}

struct FocusRingOverlay<'a, 'b, Message, Renderer> {
  button: &'b ButtonWidget<'a, Message, Renderer>,
  status: Status,
  bounds: Rectangle,
}

impl<Message, Renderer> overlay::Overlay<Message, Theme, Renderer>
  for FocusRingOverlay<'_, '_, Message, Renderer>
where
  Renderer: renderer::Renderer,
{
  fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
    layout::Node::new(self.bounds.size()).move_to(self.bounds.position())
  }

  fn draw(
    &self,
    renderer: &mut Renderer,
    theme: &Theme,
    _style: &renderer::Style,
    layout: Layout<'_>,
    _cursor: mouse::Cursor,
  ) {
    let button_style = self.button.focused_style(theme, self.status);

    let Some(focus_ring) = button_style.focus_ring else {
      return;
    };

    if focus_ring.width <= 0.0 {
      return;
    }

    let expansion = focus_ring.offset + focus_ring.width;
    let radius = button_style.border.radius;
    let bounds = layout.bounds();

    let expanded_radius = iced::border::Radius {
      top_left: radius.top_left + expansion,
      top_right: radius.top_right + expansion,
      bottom_right: radius.bottom_right + expansion,
      bottom_left: radius.bottom_left + expansion,
    };

    renderer.fill_quad(
      renderer::Quad {
        bounds: Rectangle {
          x: bounds.x - expansion,
          y: bounds.y - expansion,
          width: bounds.width + expansion * 2.0,
          height: bounds.height + expansion * 2.0,
        },
        border: iced::Border {
          radius: expanded_radius,
          width: focus_ring.width,
          color: focus_ring.color,
        },
        ..renderer::Quad::default()
      },
      iced::Background::Color(iced::Color::TRANSPARENT),
    );
  }

  fn index(&self) -> f32 {
    1.0
  }
}

impl Focusable for State {
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
