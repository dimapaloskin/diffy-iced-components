pub mod metrics;
pub mod style;
mod widget;

use std::borrow::Cow;

use iced::Element;
use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::text::Renderer as TextRenderer;

use crate::icon::Icon;
use crate::size::Density;
use crate::theme::Theme;
use crate::widgets::group::{GroupContext, GroupItem, GroupableWidget, group_item};

pub use metrics::Metrics;
pub use style::*;

#[derive(Debug, Clone)]
pub enum Content<'a> {
  Label(Cow<'a, str>),
  Icon(Icon),
}

impl<'a> From<&'a str> for Content<'a> {
  fn from(label: &'a str) -> Self {
    Self::Label(Cow::Borrowed(label))
  }
}

impl From<String> for Content<'_> {
  fn from(label: String) -> Self {
    Self::Label(Cow::Owned(label))
  }
}

impl From<Icon> for Content<'_> {
  fn from(icon: Icon) -> Self {
    Self::Icon(icon)
  }
}

pub struct Button<'a, Message> {
  pub(super) content: Content<'a>,
  pub(super) metrics: Metrics,
  pub(super) width: iced::Length,
  pub(super) height: iced::Length,
  pub(super) leading_icon: Option<Icon>,
  pub(super) trailing_icon: Option<Icon>,
  pub(super) variant: Variant,
  pub(super) mode: Mode,
  pub(super) on_press: Option<Message>,
  pub(super) style: Option<StyleFn<'a>>,
  pub(super) pill: bool,
  pub(super) grouped: Option<GroupContext>,
}

impl<'a, Message> Button<'a, Message> {
  pub fn size(mut self, size: impl Into<Density>) -> Self {
    let metrics = Metrics::from_density(size.into());

    self.height = iced::Length::Fixed(metrics.height);
    self.metrics = metrics;

    self
  }

  pub fn width(mut self, width: impl Into<iced::Length>) -> Self {
    self.width = width.into();
    self
  }

  pub fn height(mut self, height: impl Into<iced::Length>) -> Self {
    self.height = height.into();
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

  pub fn variant(mut self, variant: Variant) -> Self {
    self.variant = variant;
    self
  }

  pub fn mode(mut self, mode: Mode) -> Self {
    self.mode = mode;
    self
  }

  pub fn on_press(mut self, message: Message) -> Self {
    self.on_press = Some(message);
    self
  }

  pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
    self.on_press = message;
    self
  }

  pub fn style(mut self, style: impl Fn(&Theme, Status, Style) -> Style + 'a) -> Self {
    self.style = Some(Box::new(style));
    self
  }

  pub fn pill(mut self) -> Self {
    self.pill = true;
    self
  }

  pub fn fill(mut self) -> Self {
    self.mode = Mode::Fill;
    self
  }

  pub fn light(mut self) -> Self {
    self.mode = Mode::Light;
    self
  }

  pub fn outline(mut self) -> Self {
    self.mode = Mode::Outline;
    self
  }

  pub fn ghost(mut self) -> Self {
    self.mode = Mode::Ghost;
    self
  }

  pub fn primary(mut self) -> Self {
    self.variant = Variant::Primary;
    self
  }

  pub fn neutral(mut self) -> Self {
    self.variant = Variant::Neutral;
    self
  }

  pub fn danger(mut self) -> Self {
    self.variant = Variant::Danger;
    self
  }

  pub(crate) fn with_group_context(mut self, context: GroupContext) -> Self {
    let metrics = Metrics::from_density(context.density());

    self.metrics = metrics;
    self.height = iced::Length::Fixed(metrics.height);
    self.grouped = Some(context);

    self
  }
}

pub fn button<'a, Message>(content: impl Into<Content<'a>>) -> Button<'a, Message> {
  let metrics = Metrics::default();

  Button {
    content: content.into(),
    metrics,
    width: iced::Length::Shrink,
    height: iced::Length::Fixed(metrics.height),
    leading_icon: None,
    trailing_icon: None,
    variant: Variant::default(),
    mode: Mode::default(),
    on_press: None,
    style: None,
    pill: false,
    grouped: None,
  }
}

impl<'a, Message, Renderer> GroupableWidget<'a, Message, Renderer> for Button<'a, Message>
where
  Message: Clone + 'a,
  Renderer: RendererTrait + TextRenderer + 'a,
  <Renderer as TextRenderer>::Font: From<iced::Font>,
{
  fn grouped(self: Box<Self>, context: GroupContext) -> Element<'a, Message, Theme, Renderer> {
    (*self).with_group_context(context).into()
  }
}

impl<'a, Message, Renderer> From<Button<'a, Message>> for GroupItem<'a, Message, Renderer>
where
  Message: Clone + 'a,
  Renderer: RendererTrait + TextRenderer + 'a,
  <Renderer as TextRenderer>::Font: From<iced::Font>,
{
  fn from(button: Button<'a, Message>) -> Self {
    group_item(button)
  }
}
