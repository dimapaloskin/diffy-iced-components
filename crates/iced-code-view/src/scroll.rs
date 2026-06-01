mod map;
mod vertical_scroll;

use iced::advanced::graphics::text::cosmic_text;

use diffy_ui::widgets::scrollbar;

pub(crate) use map::{ScrollMap, ScrollMapInputs};
pub(crate) use vertical_scroll::VerticalScroll;

use crate::measurement::MeasurementResult;

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
pub(crate) enum ScrollChange {
  RedrawOnly,
  RequiresLayout,
}

pub(crate) struct ScrollModel {
  state: ScrollState,
  map: ScrollMap,
}

impl ScrollModel {
  pub(crate) fn vertical(&self) -> VerticalScroll {
    self.state.vertical
  }

  pub(crate) fn horizontal_px(&self) -> f32 {
    self.state.horizontal_px
  }

  pub(crate) fn reset(&mut self, inputs: ScrollMapInputs) {
    self.state = ScrollState::ZERO;
    self.map = ScrollMap::default();
    self.map.reconcile(&inputs);
  }

  pub(crate) fn reconcile(&mut self, inputs: ScrollMapInputs, viewport_size: iced::Size) {
    self.map.reconcile(&inputs);
    self.clamp_horizontal(viewport_size);
  }

  pub(crate) fn apply_measurement_result(
    &mut self,
    result: &MeasurementResult,
    viewport_size: iced::Size,
  ) {
    self.map.apply_measurement_result(result);
    self.clamp_horizontal(viewport_size);
  }

  fn clamp_horizontal(&mut self, viewport_size: iced::Size) {
    self.state.horizontal_px = self
      .map
      .horizontal
      .clamp_offset(self.state.horizontal_px, viewport_size.width);
  }

  // We pass requested scroll into cosmic-text before layout.
  // `shape_until_scroll(...)` may normalize it, so here we sync normalized scroll back after layout.
  pub(crate) fn sync_from_cosmic(&mut self, cosmic_scroll: cosmic_text::Scroll) {
    debug_assert_eq!(cosmic_scroll.horizontal, 0.0);
    debug_assert!(cosmic_scroll.vertical.is_finite());

    self.state.vertical = VerticalScroll::from_cosmic(cosmic_scroll);
  }

  pub(crate) fn apply_wheel_delta(
    &mut self,
    delta: iced::Vector,
    scroll_viewport: iced::Size,
  ) -> Option<ScrollChange> {
    if !delta.x.is_finite() || !delta.y.is_finite() {
      return None;
    }

    let horizontal_px = self
      .map
      .horizontal
      .clamp_offset(self.state.horizontal_px - delta.x, scroll_viewport.width);

    // iced wheel delta is positive when scrolling up.
    // cosmic Scroll is positive when scrolled down.
    let vertical_scroll = self.state.vertical.scrolled_by(-delta.y);

    let vertical_changed = vertical_scroll != self.state.vertical;
    let horizontal_changed = horizontal_px != self.state.horizontal_px;

    match (vertical_changed, horizontal_changed) {
      (false, false) => None,
      (false, true) => {
        self.state.horizontal_px = horizontal_px;
        Some(ScrollChange::RedrawOnly)
      }
      (true, _) => {
        self.state.vertical = vertical_scroll;
        self.state.horizontal_px = horizontal_px;
        Some(ScrollChange::RequiresLayout)
      }
    }
  }

  pub(crate) fn scrollbar_snapshot(
    &self,
    axis: scrollbar::Axis,
    viewport_size: iced::Size,
  ) -> scrollbar::AxisSnapshot {
    let viewport_len = axis.len_of_size(viewport_size).max(0.0) as f64;

    match axis {
      scrollbar::Axis::Vertical => self.vertical_scrollbar_snapshot(viewport_len),
      scrollbar::Axis::Horizontal => self.horizontal_scrollbar_snapshot(viewport_len),
    }
  }

  fn vertical_scrollbar_snapshot(&self, viewport_len: f64) -> scrollbar::AxisSnapshot {
    match &self.map.vertical {
      map::VerticalMap::Trivial {
        source_line_count,
        line_height,
      } => {
        let line_height = *line_height as f64;
        let offset = self.state.vertical.source_line_index as f64 * line_height
          + self.state.vertical.y_inside_source_line as f64;

        scrollbar::AxisSnapshot {
          axis: scrollbar::Axis::Vertical,
          offset,
          viewport_len,
          extent: scrollbar::ScrollExtent::Exact {
            content_len: *source_line_count as f64 * line_height,
          },
        }
      }
      map::VerticalMap::Wrapped(_) => scrollbar::AxisSnapshot {
        axis: scrollbar::Axis::Vertical,
        offset: 0.0,
        viewport_len,
        extent: scrollbar::ScrollExtent::Unknown,
      },
    }
  }

  fn horizontal_scrollbar_snapshot(&self, viewport_len: f64) -> scrollbar::AxisSnapshot {
    let offset = self.state.horizontal_px.max(0.0) as f64;

    match &self.map.horizontal {
      map::HorizontalMap::Disabled => scrollbar::AxisSnapshot {
        axis: scrollbar::Axis::Horizontal,
        offset: 0.0,
        viewport_len,
        extent: scrollbar::ScrollExtent::Disabled,
      },
      map::HorizontalMap::Unknown { .. } => scrollbar::AxisSnapshot {
        axis: scrollbar::Axis::Horizontal,
        offset,
        viewport_len,
        extent: scrollbar::ScrollExtent::Unknown,
      },
      map::HorizontalMap::Exact { content_width, .. } => scrollbar::AxisSnapshot {
        axis: scrollbar::Axis::Horizontal,
        offset,
        viewport_len,
        extent: scrollbar::ScrollExtent::Exact {
          content_len: f64::from(content_width.max(0.0)),
        },
      },
    }
  }

  pub(crate) fn set_scrollbar_offset(
    &mut self,
    axis: scrollbar::Axis,
    offset_px: f64,
    viewport_size: iced::Size,
  ) -> Option<ScrollChange> {
    match axis {
      scrollbar::Axis::Vertical => self.set_vertical_offset_from_px(offset_px),
      scrollbar::Axis::Horizontal => self.set_horizontal_offset_from_px(offset_px, viewport_size),
    }
  }

  fn set_vertical_offset_from_px(&mut self, offset_px: f64) -> Option<ScrollChange> {
    let map::VerticalMap::Trivial {
      source_line_count,
      line_height,
    } = self.map.vertical
    else {
      return None;
    };

    if source_line_count == 0 || line_height <= 0.0 {
      return None;
    }

    let max_offset = source_line_count.saturating_sub(1) as f64 * f64::from(line_height);
    let offset_px = offset_px.clamp(0.0, max_offset);

    let source_line_index = (offset_px / f64::from(line_height)).floor() as usize;
    let y_inside_source_line =
      (offset_px - source_line_index as f64 * f64::from(line_height)) as f32;

    let target = VerticalScroll {
      source_line_index,
      y_inside_source_line,
    };

    if target == self.state.vertical {
      return None;
    }

    self.state.vertical = target;
    Some(ScrollChange::RequiresLayout)
  }

  fn set_horizontal_offset_from_px(
    &mut self,
    offset_px: f64,
    viewport_size: iced::Size,
  ) -> Option<ScrollChange> {
    if !offset_px.is_finite() {
      return None;
    }

    let next = self
      .map
      .horizontal
      .clamp_offset(offset_px as f32, viewport_size.width);

    if next == self.state.horizontal_px {
      return None;
    }

    self.state.horizontal_px = next;
    Some(ScrollChange::RedrawOnly)
  }
}

impl Default for ScrollModel {
  fn default() -> Self {
    Self {
      state: ScrollState::ZERO,
      map: ScrollMap::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::measurement::MeasurementKey;
  use crate::text_layout::WrapMode;

  use super::*;

  pub(crate) fn test_inputs(mode: WrapMode) -> ScrollMapInputs {
    ScrollMapInputs {
      source_line_count: 0,
      mode,
      line_height: 18.0,
      measurement_key: test_measurement_key(),
    }
  }

  pub(crate) fn test_measurement_key() -> MeasurementKey {
    use crate::measurement::MeasurementKind;
    use crate::policies::TabDisplayPolicy;
    use iced::advanced::graphics::text::Version;

    MeasurementKey {
      document_revision: 0,
      kind: MeasurementKind::NoWrapHorizontalExtent,
      font: iced::Font::DEFAULT,
      font_size_bits: 14.0_f32.to_bits(),
      line_height_bits: 18.0_f32.to_bits(),
      tab_policy: TabDisplayPolicy::default(),
      font_system_version: Version::default(),
    }
  }

  #[test]
  fn wheel_up_at_document_top_is_noop() {
    let mut scroll = ScrollModel::default();
    scroll.reset(test_inputs(WrapMode::NoWrap));

    let change =
      scroll.apply_wheel_delta(iced::Vector::new(0.0, 50.0), iced::Size::new(300.0, 100.0));

    assert_eq!(change, None);
    assert_eq!(scroll.vertical(), VerticalScroll::ZERO);
  }

  #[test]
  fn non_finite_wheel_delta_is_ignored() {
    let mut scroll = ScrollModel::default();
    scroll.reset(test_inputs(WrapMode::NoWrap));

    let change = scroll.apply_wheel_delta(
      iced::Vector::new(f32::NAN, 50.0),
      iced::Size::new(300.0, 100.0),
    );

    assert_eq!(change, None);
    assert_eq!(scroll.vertical(), VerticalScroll::ZERO);
    assert_eq!(scroll.horizontal_px(), 0.0);
  }
}
