mod macros;
pub mod metrics;
pub mod style;
mod widget;

use iced::advanced::renderer::Renderer as RendererTrait;
use iced::{Element, Length};

use crate::size::{Density, Sizing};
use crate::theme::Theme;
use crate::widgets::group::widget::GroupWidget;

pub use metrics::Metrics;
pub use style::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupPosition {
  Only,
  First,
  Middle,
  Last,
}

impl GroupPosition {
  pub(crate) fn for_index(index: usize, len: usize) -> Self {
    match (index, len) {
      (_, 0 | 1) => GroupPosition::Only,
      (0, _) => GroupPosition::First,
      (i, l) if i + 1 == l => GroupPosition::Last,
      _ => GroupPosition::Middle,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct GroupContext {
  position: GroupPosition,
  density: Density,
  pill: bool,
  frame_width: f32,
}

impl GroupContext {
  pub fn position(&self) -> GroupPosition {
    self.position
  }

  pub fn density(&self) -> Density {
    self.density
  }

  pub fn pill(&self) -> bool {
    self.pill
  }

  pub fn frame_width(&self) -> f32 {
    self.frame_width
  }

  pub fn radius_for(&self, theme: &Theme) -> iced::border::Radius {
    let outer_radius = if self.pill {
      theme.radius().pill
    } else {
      theme.radius().control
    };

    let shrink = self.frame_width.max(0.0);

    let inner_radius = iced::border::Radius {
      top_left: (outer_radius.top_left - shrink).max(0.0),
      top_right: (outer_radius.top_right - shrink).max(0.0),
      bottom_right: (outer_radius.bottom_right - shrink).max(0.0),
      bottom_left: (outer_radius.bottom_left - shrink).max(0.0),
    };

    match self.position {
      GroupPosition::Only => inner_radius,
      GroupPosition::First => iced::border::Radius {
        top_left: inner_radius.top_left,
        bottom_left: inner_radius.bottom_left,
        top_right: 0.0,
        bottom_right: 0.0,
      },
      GroupPosition::Middle => iced::border::Radius::default(),
      GroupPosition::Last => iced::border::Radius {
        top_left: 0.0,
        bottom_left: 0.0,
        top_right: inner_radius.top_right,
        bottom_right: inner_radius.bottom_right,
      },
    }
  }
}

pub struct Group<'a, Message, Renderer = iced::Renderer> {
  pub(crate) items: Vec<GroupItem<'a, Message, Renderer>>,
  pub(crate) width: Length,
  pub(crate) height: Length,
  pub(crate) density: Density,
  pub(crate) pill: bool,
  pub(crate) separators: bool,
  pub(crate) metrics: Metrics,
  pub(crate) style: Option<style::StyleFn<'a>>,
}

impl<'a, Message, Renderer> From<Group<'a, Message, Renderer>>
  for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Renderer: RendererTrait + 'a,
{
  fn from(group: Group<'a, Message, Renderer>) -> Self {
    let Group {
      items,
      width,
      height,
      density,
      pill,
      separators,
      metrics,
      style,
    } = group;

    let total = items.len();

    let children = items
      .into_iter()
      .enumerate()
      .map(|(index, item)| {
        let context = GroupContext {
          position: GroupPosition::for_index(index, total),
          density,
          pill,
          frame_width: metrics.frame_width.max(0.0),
        };

        item.grouped(context)
      })
      .collect::<Vec<_>>();

    GroupWidget::new(children, width, height, pill, separators, metrics, style).into()
  }
}

impl<'a, Message, Renderer> Group<'a, Message, Renderer> {
  pub fn size(mut self, size: impl Into<Density>) -> Self {
    self.density = size.into();
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

  pub fn pill(mut self) -> Self {
    self.pill = true;
    self
  }

  pub fn without_separators(mut self) -> Self {
    self.separators = false;
    self
  }

  pub fn style(mut self, style: impl Fn(&Theme, Style) -> style::Style + 'a) -> Self {
    self.style = Some(Box::new(style));
    self
  }

  pub fn metrics(mut self, metrics: Metrics) -> Self {
    self.metrics = metrics;
    self
  }

  pub fn frame_width(mut self, width: f32) -> Self {
    self.metrics.frame_width = width;
    self
  }

  pub fn separator_width(mut self, width: f32) -> Self {
    self.metrics.separator_width = width;
    self
  }
}

pub struct GroupItem<'a, Message, Renderer = iced::Renderer> {
  pub(crate) inner: Box<dyn GroupableWidget<'a, Message, Renderer> + 'a>,
}

impl<'a, Message, Renderer> GroupItem<'a, Message, Renderer> {
  pub(crate) fn grouped(self, context: GroupContext) -> Element<'a, Message, Theme, Renderer> {
    self.inner.grouped(context)
  }
}

pub trait GroupableWidget<'a, Message, Renderer = iced::Renderer> {
  fn grouped(self: Box<Self>, context: GroupContext) -> Element<'a, Message, Theme, Renderer>;
}

pub fn group_item<'a, Message, Renderer, W>(widget: W) -> GroupItem<'a, Message, Renderer>
where
  W: GroupableWidget<'a, Message, Renderer> + 'a,
{
  GroupItem {
    inner: Box::new(widget),
  }
}

pub fn group<'a, Message, Renderer, Items, Item>(items: Items) -> Group<'a, Message, Renderer>
where
  Items: IntoIterator<Item = Item>,
  Item: Into<GroupItem<'a, Message, Renderer>>,
{
  Group {
    items: items.into_iter().map(Into::into).collect(),
    width: Length::Shrink,
    height: Length::Shrink,
    density: Sizing::Md.into(),
    pill: false,
    separators: true,
    metrics: Metrics::default(),
    style: None,
  }
}
