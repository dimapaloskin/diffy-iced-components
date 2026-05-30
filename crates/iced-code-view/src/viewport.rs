use crate::gutter::GutterMetrics;
use crate::insets::Insets;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ViewportArea {
  pub(crate) bounds: iced::Rectangle,
  pub(crate) content_bounds: iced::Rectangle,
}

impl ViewportArea {
  fn offset(self, widget_bounds: iced::Rectangle) -> Self {
    Self {
      bounds: offset_bounds(self.bounds, widget_bounds),
      content_bounds: offset_bounds(self.content_bounds, widget_bounds),
    }
  }
}

#[derive(Default)]
pub(crate) struct Viewport {
  pub(crate) gutter: Option<ViewportArea>,
  pub(crate) text: ViewportArea,
}

impl Viewport {
  pub(crate) fn new(
    widget_size: iced::Size,
    insets: Insets,
    gutter: GutterMetrics,
    right_chrome_reserve: f32,
  ) -> Self {
    let surface_width = widget_size.width.max(0.0);
    let surface_height = widget_size.height.max(0.0);

    let surface_bounds = iced::Rectangle {
      x: 0.0,
      y: 0.0,
      width: surface_width,
      height: surface_height,
    };

    let content_height = (surface_height - insets.vertical.top - insets.vertical.bottom).max(0.0);

    let gutter_width = if gutter.enabled {
      gutter.requested_width.min(surface_width).max(0.0)
    } else {
      0.0
    };

    let right_chrome_reserve = right_chrome_reserve
      .max(0.0)
      .min((surface_width - gutter_width).max(0.0));

    let gutter = (gutter_width > 0.0).then_some(ViewportArea {
      bounds: iced::Rectangle {
        x: surface_bounds.x,
        y: surface_bounds.y,
        width: gutter_width,
        height: surface_bounds.height,
      },
      content_bounds: iced::Rectangle {
        x: surface_bounds.x,
        y: insets.vertical.top,
        width: gutter_width,
        height: content_height,
      },
    });

    let text_bounds = iced::Rectangle {
      x: surface_bounds.x + gutter_width,
      y: surface_bounds.y,
      width: (surface_width - gutter_width - right_chrome_reserve).max(0.0),
      height: surface_bounds.height,
    };

    let text_content_bounds = iced::Rectangle {
      x: text_bounds.x + insets.text.left,
      y: text_bounds.y + insets.vertical.top,
      width: (text_bounds.width - insets.text.left - insets.text.right).max(0.0),
      height: content_height,
    };

    Self {
      gutter,
      text: ViewportArea {
        bounds: text_bounds,
        content_bounds: text_content_bounds,
      },
    }
  }

  pub(crate) fn absolute_gutter(&self, widget_bounds: iced::Rectangle) -> Option<ViewportArea> {
    self.gutter.map(|area| area.offset(widget_bounds))
  }

  pub(crate) fn absolute_text_content_bounds(
    &self,
    widget_bounds: iced::Rectangle,
  ) -> iced::Rectangle {
    offset_bounds(self.text.content_bounds, widget_bounds)
  }

  pub(crate) fn scroll_viewport_size(&self) -> iced::Size {
    self.text.content_bounds.size()
  }
}

fn offset_bounds(bounds: iced::Rectangle, widget_bounds: iced::Rectangle) -> iced::Rectangle {
  iced::Rectangle {
    x: widget_bounds.x + bounds.x,
    y: widget_bounds.y + bounds.y,
    width: bounds.width,
    height: bounds.height,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::insets::{HorizontalInsets, VerticalInsets};

  fn gutter_metrics() -> GutterMetrics {
    GutterMetrics {
      enabled: true,
      requested_width: 40.0,
      insets: crate::insets::GutterInsets::ZERO,
      separator_width: 1.0,
    }
  }

  #[test]
  fn viewport_splits_surface_gutter_text_and_content_bounds() {
    let viewport = Viewport::new(
      iced::Size::new(200.0, 100.0),
      Insets::new(
        VerticalInsets::new(5.0, 11.0),
        HorizontalInsets::new(13.0, 7.0),
      ),
      gutter_metrics(),
      0.0,
    );

    assert_eq!(
      viewport.gutter,
      Some(ViewportArea {
        bounds: iced::Rectangle {
          x: 0.0,
          y: 0.0,
          width: 40.0,
          height: 100.0,
        },
        content_bounds: iced::Rectangle {
          x: 0.0,
          y: 5.0,
          width: 40.0,
          height: 84.0,
        },
      })
    );

    assert_eq!(
      viewport.text,
      ViewportArea {
        bounds: iced::Rectangle {
          x: 40.0,
          y: 0.0,
          width: 160.0,
          height: 100.0,
        },
        content_bounds: iced::Rectangle {
          x: 53.0,
          y: 5.0,
          width: 140.0,
          height: 84.0,
        },
      }
    );

    assert_eq!(
      viewport.scroll_viewport_size(),
      iced::Size::new(140.0, 84.0)
    );
  }

  #[test]
  fn viewport_clamps_gutter_to_surface_width() {
    let viewport = Viewport::new(
      iced::Size::new(10.0, 40.0),
      Insets::new(
        VerticalInsets::new(2.0, 3.0),
        HorizontalInsets::new(5.0, 4.0),
      ),
      GutterMetrics {
        requested_width: 100.0,
        ..gutter_metrics()
      },
      0.0,
    );

    assert_eq!(
      viewport.gutter,
      Some(ViewportArea {
        bounds: iced::Rectangle {
          x: 0.0,
          y: 0.0,
          width: 10.0,
          height: 40.0,
        },
        content_bounds: iced::Rectangle {
          x: 0.0,
          y: 2.0,
          width: 10.0,
          height: 35.0,
        },
      })
    );

    assert_eq!(
      viewport.text,
      ViewportArea {
        bounds: iced::Rectangle {
          x: 10.0,
          y: 0.0,
          width: 0.0,
          height: 40.0,
        },
        content_bounds: iced::Rectangle {
          x: 15.0,
          y: 2.0,
          width: 0.0,
          height: 35.0,
        },
      }
    );

    assert_eq!(viewport.scroll_viewport_size(), iced::Size::new(0.0, 35.0));
  }

  #[test]
  fn right_chrome_reserve_reduces_text_area_without_touching_gutter() {
    let viewport = Viewport::new(
      iced::Size::new(200.0, 100.0),
      Insets::new(
        VerticalInsets::new(5.0, 11.0),
        HorizontalInsets::new(13.0, 7.0),
      ),
      gutter_metrics(),
      14.0,
    );

    assert_eq!(
      viewport.gutter.expect("gutter should stay enabled").bounds,
      iced::Rectangle {
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 100.0,
      }
    );

    assert_eq!(
      viewport.text.bounds,
      iced::Rectangle {
        x: 40.0,
        y: 0.0,
        width: 146.0,
        height: 100.0,
      }
    );

    assert_eq!(
      viewport.text.content_bounds,
      iced::Rectangle {
        x: 53.0,
        y: 5.0,
        width: 126.0,
        height: 84.0,
      }
    );
  }
}
