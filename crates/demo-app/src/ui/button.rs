mod metrics;
mod offset;
mod style;
mod widget;

use std::borrow::Cow;

use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::text::Renderer as TextRendererTrait;
use iced::{Alignment, Element, Length};

use crate::ui::icon::{self, Icon};
use crate::ui::sizing::Sizing;
use crate::ui::theme::Theme;

pub use self::metrics::Metrics;
pub use self::style::{FocusRing, Status, Style, Variant};

use self::style::Class;
use self::widget::ButtonWidget;

#[derive(Debug, Clone)]
pub enum Content<'a> {
  Text(Cow<'a, str>),
  Icon(Icon),
}

impl<'a> From<&'a str> for Content<'a> {
  fn from(label: &'a str) -> Self {
    Self::Text(Cow::Borrowed(label))
  }
}

impl<'a> From<String> for Content<'a> {
  fn from(label: String) -> Self {
    Self::Text(Cow::Owned(label))
  }
}

impl<'a> From<Cow<'a, str>> for Content<'a> {
  fn from(label: Cow<'a, str>) -> Self {
    Self::Text(label)
  }
}

impl<'a> From<Icon> for Content<'a> {
  fn from(icon: Icon) -> Self {
    Self::Icon(icon)
  }
}

pub struct Button<'a, Message> {
  content: Content<'a>,
  metrics: Metrics,
  width: Length,
  height: Length,
  font: Option<iced::Font>,
  content_y_offset: Option<f32>,
  icon_y_offset: Option<f32>,
  leading_icon: Option<Icon>,
  trailing_icon: Option<Icon>,
  class: Class<'a>,
  on_press: Option<Message>,
}

pub fn button<'a, Message>(content: impl Into<Content<'a>>) -> Button<'a, Message> {
  let metrics: Metrics = Sizing::Md.into();

  Button {
    content: content.into(),
    metrics,
    width: Length::Shrink,
    height: Length::Fixed(metrics.height),
    font: None,
    content_y_offset: None,
    icon_y_offset: None,
    leading_icon: None,
    trailing_icon: None,
    class: Class::Variant(Variant::default()),
    on_press: None,
  }
}

impl<'a, Message> Button<'a, Message> {
  pub fn size(mut self, size: impl Into<Metrics>) -> Self {
    let metrics = size.into();

    self.height = Length::Fixed(metrics.height);
    self.metrics = metrics;
    self
  }

  pub fn width(mut self, width: impl Into<Length>) -> Self {
    self.width = width.into();
    self
  }

  pub fn height(mut self, height: impl Into<Length>) -> Self {
    self.height = height.into();
    self
  }

  pub fn font(mut self, font: iced::Font) -> Self {
    self.font = Some(font);
    self
  }

  pub fn leading_icon(mut self, icon: Icon) -> Self {
    self.leading_icon = Some(icon);
    self
  }

  pub fn trailing_icon(mut self, icon: Icon) -> Self {
    self.trailing_icon = Some(icon);
    self
  }

  pub fn content_y_offset(mut self, offset: f32) -> Self {
    self.content_y_offset = Some(offset);
    self
  }

  pub fn icon_y_offset(mut self, offset: f32) -> Self {
    self.icon_y_offset = Some(offset);
    self
  }

  pub fn variant(mut self, variant: Variant) -> Self {
    self.class = Class::Variant(variant);
    self
  }

  pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
    self.class = Class::Custom(Box::new(style));
    self
  }

  pub fn on_press(mut self, message: Message) -> Self {
    self.on_press = Some(message);
    self
  }
}

impl<'a, Message, Renderer> From<Button<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
  Message: Clone + 'a,
  Renderer: RendererTrait + TextRendererTrait + 'a,
  <Renderer as TextRendererTrait>::Font: From<iced::Font>,
{
  fn from(button: Button<'a, Message>) -> Self {
    let content_y_offset = button
      .content_y_offset
      .unwrap_or(button.metrics.content_y_offset);
    let icon_y_offset = button.icon_y_offset.unwrap_or(button.metrics.icon_y_offset);
    let main_content_index = usize::from(button.leading_icon.is_some());

    let mut content = iced::widget::row![]
      .spacing(button.metrics.gap)
      .align_y(Alignment::Center);

    if let Some(icon) = button.leading_icon {
      content = content.push(icon_content(icon, &button.metrics, icon_y_offset));
    }

    content = content.push(main_content(
      button.content,
      &button.metrics,
      button.font,
      content_y_offset,
      icon_y_offset,
    ));

    if let Some(icon) = button.trailing_icon {
      content = content.push(icon_content(icon, &button.metrics, icon_y_offset));
    }

    Element::new(ButtonWidget {
      content: content.into(),
      metrics: button.metrics,
      width: button.width,
      height: button.height,
      class: button.class,
      on_press: button.on_press,
      main_content_index,
    })
  }
}

fn main_content<'a, Message, Renderer>(
  content: Content<'a>,
  metrics: &Metrics,
  font: Option<iced::Font>,
  content_y_offset: f32,
  icon_y_offset: f32,
) -> Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Renderer: RendererTrait + TextRendererTrait + 'a,
  <Renderer as TextRendererTrait>::Font: From<iced::Font>,
{
  match content {
    Content::Text(label) => {
      let mut label = iced::widget::text(label)
        .size(metrics.font_size)
        .align_y(iced::alignment::Vertical::Center);

      if let Some(font) = font {
        label = label.font(font);
      }

      offset::y(label, content_y_offset)
    }
    Content::Icon(icon) => icon_content(icon, metrics, icon_y_offset),
  }
}

fn icon_content<'a, Message, Renderer>(
  icon: Icon,
  metrics: &Metrics,
  icon_y_offset: f32,
) -> Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Renderer: RendererTrait + TextRendererTrait + 'a,
  <Renderer as TextRendererTrait>::Font: From<iced::Font>,
{
  offset::y(icon::text(icon).size(metrics.icon_size), icon_y_offset)
}
