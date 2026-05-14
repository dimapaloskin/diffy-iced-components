use iced::advanced::graphics::text::cosmic_text;

use crate::WrapMode;
use crate::measurement::{MeasurementOutput, MeasurementResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScrollState {
  // The value stored here can be in two states.
  // After wheel: where we want to scroll.
  // After layout: the scroll accepted by cosmic-text.
  // layouting must run before anyone uses this value as the final scroll position,
  // because only cosmic-text knows the valid line + y offset.
  pub(crate) vertical: VerticalScroll,
  pub(crate) horizontal_px: f32,
}

impl ScrollState {
  pub(crate) const ZERO: Self = Self {
    vertical: VerticalScroll::ZERO,
    horizontal_px: 0.0,
  };
}

impl Default for ScrollState {
  fn default() -> Self {
    Self::ZERO
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VerticalScroll {
  pub(crate) source_line_index: usize,
  pub(crate) y_inside_source_line: f32,
}

impl VerticalScroll {
  pub(crate) const ZERO: Self = Self {
    source_line_index: 0,
    y_inside_source_line: 0.0,
  };

  pub(crate) fn to_cosmic(self) -> cosmic_text::Scroll {
    // Horizontal scroll is applied at draw time, not via cosmic Scroll
    cosmic_text::Scroll::new(self.source_line_index, self.y_inside_source_line, 0.0)
  }

  pub(crate) fn from_cosmic(scroll: cosmic_text::Scroll) -> Self {
    Self {
      source_line_index: scroll.line,
      y_inside_source_line: scroll.vertical,
    }
  }

  pub(crate) fn scrolled_by(self, delta_y: f32) -> Self {
    let y_inside_source_line = self.y_inside_source_line + delta_y;
    // Prevent a no-op wheel at the document top from forcing a relayout.
    let y_inside_source_line = if self.source_line_index == 0 {
      y_inside_source_line.max(0.0)
    } else {
      y_inside_source_line
    };

    Self {
      y_inside_source_line,
      ..self
    }
  }
}

impl Default for VerticalScroll {
  fn default() -> Self {
    Self::ZERO
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct ScrollExtent {
  pub(crate) horizontal: HorizontalExtent,
}

impl ScrollExtent {
  pub(crate) fn new(wrap_mode: WrapMode, measurement_result: Option<&MeasurementResult>) -> Self {
    match wrap_mode {
      WrapMode::NoWrap => Self {
        horizontal: no_wrap_horizontal_extent(measurement_result),
      },
      WrapMode::SoftWrap => Self {
        horizontal: HorizontalExtent::NotScrollable,
      },
    }
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) enum HorizontalExtent {
  #[default]
  Unknown,
  NotScrollable,
  Exact {
    content: f32,
  },
}

impl HorizontalExtent {
  pub(crate) fn clamp_offset(&self, offset: f32, viewport: f32) -> f32 {
    match self {
      HorizontalExtent::Unknown => offset.max(0.0),
      HorizontalExtent::NotScrollable => 0.0,
      HorizontalExtent::Exact { content } => {
        let max = (*content - viewport).max(0.0);
        offset.clamp(0.0, max)
      }
    }
  }
}

fn no_wrap_horizontal_extent(measurement_result: Option<&MeasurementResult>) -> HorizontalExtent {
  match measurement_result.map(|result| result.output) {
    Some(MeasurementOutput::NoWrapHorizontalExtent { content_width }) => HorizontalExtent::Exact {
      content: content_width,
    },
    None => HorizontalExtent::Unknown,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unknown_horizontal_extent_clamps_only_to_zero() {
    let extent = HorizontalExtent::Unknown;

    assert_eq!(extent.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(extent.clamp_offset(50.0, 300.0), 50.0);
  }

  #[test]
  fn not_scrollable_horizontal_extent_always_clamps_to_zero() {
    let extent = HorizontalExtent::NotScrollable;

    assert_eq!(extent.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(extent.clamp_offset(50.0, 300.0), 0.0);
  }

  #[test]
  fn exact_horizontal_extent_clamps_to_scrollable_range() {
    let extent = HorizontalExtent::Exact { content: 1000.0 };

    assert_eq!(extent.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(extent.clamp_offset(50.0, 300.0), 50.0);
    assert_eq!(extent.clamp_offset(900.0, 300.0), 700.0);
    assert_eq!(
      HorizontalExtent::Exact { content: 200.0 }.clamp_offset(50.0, 300.0),
      0.0
    );
  }

  #[test]
  fn no_wrap_horizontal_extent_is_unknown_without_measurement() {
    assert_eq!(
      ScrollExtent::new(WrapMode::NoWrap, None).horizontal,
      HorizontalExtent::Unknown
    );
  }

  #[test]
  fn soft_wrap_horizontal_extent_is_not_scrollable() {
    assert_eq!(
      ScrollExtent::new(WrapMode::SoftWrap, None),
      ScrollExtent {
        horizontal: HorizontalExtent::NotScrollable,
      }
    );
  }

  #[test]
  fn vertical_scroll_at_top_does_not_go_negative() {
    assert_eq!(
      VerticalScroll::ZERO.scrolled_by(-50.0),
      VerticalScroll::ZERO
    );
  }

  #[test]
  fn vertical_scroll_inside_document_can_go_negative() {
    assert_eq!(
      VerticalScroll {
        source_line_index: 5,
        y_inside_source_line: 10.0,
      }
      .scrolled_by(-30.0),
      VerticalScroll {
        source_line_index: 5,
        y_inside_source_line: -20.0,
      }
    );
  }
}
