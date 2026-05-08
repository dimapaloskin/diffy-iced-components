#[derive(Default)]
pub(crate) struct Viewport {
  pub(crate) widget_size: iced::Size,
  pub(crate) content_bounds: iced::Rectangle,
  pub(crate) scroll_offset: iced::Vector,
}

impl Viewport {
  pub(crate) fn new(
    widget_size: iced::Size,
    padding: iced::padding::Padding,
    scroll_offset: iced::Vector,
  ) -> Self {
    let width = (widget_size.width - padding.left - padding.right).max(0.0);
    let height = (widget_size.height - padding.top - padding.bottom).max(0.0);

    Self {
      widget_size,
      content_bounds: iced::Rectangle {
        x: padding.left,
        y: padding.top,
        width,
        height,
      },
      scroll_offset,
    }
  }

  pub(crate) fn absolute_content_bounds(&self, widget_bounds: iced::Rectangle) -> iced::Rectangle {
    iced::Rectangle {
      x: widget_bounds.x + self.content_bounds.x,
      y: widget_bounds.y + self.content_bounds.y,
      width: self.content_bounds.width,
      height: self.content_bounds.height,
    }
  }

  pub(crate) fn with_scroll_offset(mut self, scroll_offset: iced::Vector) -> Self {
    self.scroll_offset = scroll_offset;
    self
  }
}
