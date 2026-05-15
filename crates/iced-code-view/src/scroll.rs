mod geometry;
mod vertical_scroll;

use iced::advanced::graphics::text::cosmic_text;

pub(crate) use geometry::{GeometryInputs, ScrollGeometry};
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
  geometry: ScrollGeometry,
}

impl ScrollModel {
  pub(crate) fn vertical(&self) -> VerticalScroll {
    self.state.vertical
  }

  pub(crate) fn horizontal_px(&self) -> f32 {
    self.state.horizontal_px
  }

  pub(crate) fn reset(&mut self, inputs: GeometryInputs) {
    self.state = ScrollState::ZERO;
    self.geometry = ScrollGeometry::default();
    self.geometry.reconcile(&inputs);
  }

  pub(crate) fn reconcile(&mut self, inputs: GeometryInputs, viewport_size: iced::Size) {
    self.geometry.reconcile(&inputs);
    self.clamp_horizontal(viewport_size);
  }

  pub(crate) fn apply_measurement_result(
    &mut self,
    result: &MeasurementResult,
    viewport_size: iced::Size,
  ) {
    self.geometry.apply_measurement_result(result);
    self.clamp_horizontal(viewport_size);
  }

  fn clamp_horizontal(&mut self, viewport_size: iced::Size) {
    self.state.horizontal_px = self
      .geometry
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
      .geometry
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
}

impl Default for ScrollModel {
  fn default() -> Self {
    Self {
      state: ScrollState::ZERO,
      geometry: ScrollGeometry::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::measurement::MeasurementKey;
  use crate::text_layout::WrapMode;

  use super::*;

  pub(crate) fn test_inputs(mode: WrapMode) -> GeometryInputs {
    GeometryInputs {
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
