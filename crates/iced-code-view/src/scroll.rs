use crate::measurement::{MeasurementOutput, MeasurementResult};
use crate::{Document, WrapMode};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct ScrollExtent {
  pub(crate) horizontal: AxisExtent,
  pub(crate) vertical: AxisExtent,
}

impl ScrollExtent {
  pub(crate) fn new(
    document: &Document,
    wrap_mode: WrapMode,
    line_height: f32,
    measurement_result: Option<&MeasurementResult>,
  ) -> Self {
    match wrap_mode {
      WrapMode::NoWrap => Self {
        horizontal: no_wrap_horizontal_extent(measurement_result),
        vertical: AxisExtent::Exact {
          content: document.line_count() as f32 * line_height,
        },
      },
      WrapMode::SoftWrap => Self {
        horizontal: AxisExtent::Disabled,
        vertical: AxisExtent::Unknown,
      },
    }
  }

  pub(crate) fn clamp_offset(
    &self,
    offset: iced::Vector,
    viewport_size: iced::Size,
  ) -> iced::Vector {
    iced::Vector::new(
      self.horizontal.clamp_offset(offset.x, viewport_size.width),
      self.vertical.clamp_offset(offset.y, viewport_size.height),
    )
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) enum AxisExtent {
  #[default]
  Unknown,
  Disabled,
  Exact {
    content: f32,
  },
  // Provisional { content: f32 },
}

impl AxisExtent {
  pub(crate) fn clamp_offset(&self, offset: f32, viewport: f32) -> f32 {
    match self {
      AxisExtent::Unknown => offset.max(0.0),
      AxisExtent::Disabled => 0.0,
      AxisExtent::Exact { content } => {
        let max = (*content - viewport).max(0.0);
        offset.clamp(0.0, max)
      }
    }
  }
}

fn no_wrap_horizontal_extent(measurement_result: Option<&MeasurementResult>) -> AxisExtent {
  match measurement_result.map(|result| result.output) {
    Some(MeasurementOutput::NoWrapHorizontalExtent { content_width }) => AxisExtent::Exact {
      content: content_width,
    },
    None => AxisExtent::Unknown,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unknown_axis_clamps_only_to_zero() {
    let extent = AxisExtent::Unknown;

    assert_eq!(extent.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(extent.clamp_offset(50.0, 300.0), 50.0);
  }

  #[test]
  fn disabled_axis_always_clamps_to_zero() {
    let extent = AxisExtent::Disabled;

    assert_eq!(extent.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(extent.clamp_offset(50.0, 300.0), 0.0);
  }

  #[test]
  fn exact_axis_clamps_to_scrollable_range() {
    let extent = AxisExtent::Exact { content: 1000.0 };

    assert_eq!(extent.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(extent.clamp_offset(50.0, 300.0), 50.0);
    assert_eq!(extent.clamp_offset(900.0, 300.0), 700.0);
  }

  #[test]
  fn exact_axis_clamps_to_zero_when_content_fits_viewport() {
    let extent = AxisExtent::Exact { content: 200.0 };

    assert_eq!(extent.clamp_offset(50.0, 300.0), 0.0);
  }

  #[test]
  fn scroll_extent_clamps_each_axis_independently() {
    let extent = ScrollExtent {
      horizontal: AxisExtent::Unknown,
      vertical: AxisExtent::Exact { content: 1000.0 },
    };

    assert_eq!(
      extent.clamp_offset(
        iced::Vector::new(-20.0, 900.0),
        iced::Size::new(300.0, 300.0)
      ),
      iced::Vector::new(0.0, 700.0)
    );
  }

  #[test]
  fn no_wrap_extent_counts_trailing_empty_source_line() {
    let document = Document::new("a\n");

    assert_eq!(
      ScrollExtent::new(&document, WrapMode::NoWrap, 20.0, None).vertical,
      AxisExtent::Exact { content: 40.0 }
    );
  }

  #[test]
  fn no_wrap_horizontal_extent_stays_unknown_without_measurement() {
    let document = Document::new("a very long line");

    assert_eq!(
      ScrollExtent::new(&document, WrapMode::NoWrap, 20.0, None).horizontal,
      AxisExtent::Unknown
    );
  }

  #[test]
  fn no_wrap_horizontal_extent_uses_measurement_result() {
    use crate::text_layout::TextLayoutConfig;

    let document = Document::new("short\nlonger line");
    let request =
      crate::measurement::MeasurementRequest::new(&document, TextLayoutConfig::default(), 300.0);
    let result = MeasurementResult::no_wrap_horizontal_extent(request.key, 1234.0);

    assert_eq!(
      ScrollExtent::new(&document, WrapMode::NoWrap, 20.0, Some(&result)).horizontal,
      AxisExtent::Exact { content: 1234.0 }
    );
  }

  #[test]
  fn soft_wrap_horizontal_extent_is_disabled() {
    let document = Document::new("a very long line that may wrap later");

    assert_eq!(
      ScrollExtent::new(&document, WrapMode::SoftWrap, 20.0, None),
      ScrollExtent {
        horizontal: AxisExtent::Disabled,
        vertical: AxisExtent::Unknown,
      }
    );
  }
}
