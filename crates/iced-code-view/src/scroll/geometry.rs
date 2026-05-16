use crate::WrapMode;
use crate::measurement::MeasurementKey;
use crate::measurement::{MeasurementOutput, MeasurementResult};

#[derive(Debug, Clone)]
pub(crate) struct GeometryInputs {
  pub(crate) source_line_count: usize,
  pub(crate) mode: WrapMode,
  pub(crate) line_height: f32,
  pub(crate) measurement_key: MeasurementKey,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct ScrollGeometry {
  pub(super) horizontal: HorizontalGeometry,
  pub(super) vertical: VerticalGeometry,
}

impl ScrollGeometry {
  pub(crate) fn reconcile(&mut self, inputs: &GeometryInputs) {
    self.horizontal.reconcile(inputs);
    self.vertical.reconcile(inputs);
  }

  pub(crate) fn apply_measurement_result(&mut self, result: &MeasurementResult) {
    self.horizontal.apply_measurement_result(result);
    self.vertical.apply_measurement_result(result);
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) enum HorizontalGeometry {
  #[default]
  Disabled,
  Unknown {
    key: MeasurementKey,
  },
  Exact {
    key: MeasurementKey,
    content_width: f32,
  },
}

impl HorizontalGeometry {
  pub(super) fn clamp_offset(&self, offset: f32, viewport_width: f32) -> f32 {
    match self {
      HorizontalGeometry::Unknown { .. } => offset.max(0.0),
      HorizontalGeometry::Disabled => 0.0,
      HorizontalGeometry::Exact { content_width, .. } => {
        let max = (*content_width - viewport_width).max(0.0);
        offset.clamp(0.0, max)
      }
    }
  }

  fn key(&self) -> Option<MeasurementKey> {
    match self {
      Self::Disabled => None,
      Self::Unknown { key } | Self::Exact { key, .. } => Some(*key),
    }
  }

  fn reconcile(&mut self, inputs: &GeometryInputs) {
    match inputs.mode {
      WrapMode::SoftWrap => *self = Self::Disabled,
      WrapMode::NoWrap => {
        if self.key() != Some(inputs.measurement_key) {
          *self = Self::Unknown {
            key: inputs.measurement_key,
          };
        }
      }
    }
  }

  fn apply_measurement_result(&mut self, result: &MeasurementResult) {
    if self.key() != Some(result.key) {
      return;
    }

    match &result.output {
      MeasurementOutput::NoWrapHorizontalExtent { content_width } => {
        *self = Self::Exact {
          key: result.key,
          content_width: *content_width,
        }
      }
      MeasurementOutput::SoftWrapLineHeights { wrap_row_counts: _ } => {}
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum VerticalGeometry {
  Trivial {
    source_line_count: usize,
    line_height: f32,
  },
  Wrapped(WrappedVerticalGeometry),
}

impl Default for VerticalGeometry {
  fn default() -> Self {
    Self::Trivial {
      source_line_count: 0,
      line_height: 0.0,
    }
  }
}

impl VerticalGeometry {
  fn reconcile(&mut self, inputs: &GeometryInputs) {
    match (inputs.mode, &*self) {
      (
        WrapMode::NoWrap,
        Self::Trivial {
          source_line_count,
          line_height,
        },
      ) if *source_line_count == inputs.source_line_count && *line_height == inputs.line_height => { /*  no-op */
      }
      (WrapMode::NoWrap, _) => {
        *self = Self::Trivial {
          source_line_count: inputs.source_line_count,
          line_height: inputs.line_height,
        }
      }
      (WrapMode::SoftWrap, Self::Wrapped(geo))
        if geo.measurement_key == inputs.measurement_key
          && geo.source_line_count == inputs.source_line_count =>
      { /* no-op */ }
      (WrapMode::SoftWrap, _) => *self = Self::Wrapped(WrappedVerticalGeometry::unmeasured(inputs)),
    }
  }

  fn apply_measurement_result(&mut self, result: &MeasurementResult) {
    let Self::Wrapped(geometry) = self else {
      return;
    };

    geometry.apply_measurement_result(result);
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WrappedVerticalGeometry {
  pub(super) measurement_key: MeasurementKey,
  pub(super) source_line_count: usize,
  pub(super) line_height: f32,
  pub(super) per_line: Vec<LineHeightState>,
  pub(super) measured_line_count: usize,
}

impl WrappedVerticalGeometry {
  fn unmeasured(inputs: &GeometryInputs) -> Self {
    Self {
      measurement_key: inputs.measurement_key,
      source_line_count: inputs.source_line_count,
      line_height: inputs.line_height,
      per_line: vec![LineHeightState::Unknown; inputs.source_line_count],
      measured_line_count: 0,
    }
  }

  fn apply_measurement_result(&mut self, result: &MeasurementResult) {
    if result.key != self.measurement_key {
      return;
    }

    let MeasurementOutput::SoftWrapLineHeights { wrap_row_counts } = &result.output else {
      return;
    };

    if wrap_row_counts.len() != self.source_line_count {
      return;
    }

    for (line_state, &wrap_row_count) in self.per_line.iter_mut().zip(wrap_row_counts.iter()) {
      debug_assert!(wrap_row_count >= 1);

      *line_state = LineHeightState::Measured { wrap_row_count };
    }

    self.measured_line_count = self.source_line_count;
  }

  pub(crate) fn fully_measured(&self) -> bool {
    self.measured_line_count == self.source_line_count
  }

  #[allow(dead_code)]
  pub(crate) fn exact_total_height(&self) -> Option<f64> {
    if !self.fully_measured() {
      return None;
    }

    let line_height = self.line_height as f64;
    let total_wrap_rows: usize = self
      .per_line
      .iter()
      .map(|line| match line {
        LineHeightState::Unknown => 0,
        LineHeightState::Measured { wrap_row_count } => *wrap_row_count,
      })
      .sum();

    Some((total_wrap_rows as f64) * line_height)
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) enum LineHeightState {
  #[default]
  Unknown,
  Measured {
    wrap_row_count: usize,
  },
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use iced::advanced::graphics::text::Version;

  use crate::measurement::{MeasurementKey, MeasurementKind, MeasurementResult};
  use crate::policies::TabDisplayPolicy;

  use super::super::tests::{test_inputs, test_measurement_key};
  use super::super::{ScrollChange, ScrollModel};
  use super::*;

  fn soft_wrap_key(document_revision: u64, content_width: f32, line_height: f32) -> MeasurementKey {
    MeasurementKey {
      document_revision,
      kind: MeasurementKind::SoftWrapLineHeights {
        content_width_bits: content_width.to_bits(),
      },
      font: iced::Font::DEFAULT,
      font_size_bits: 14.0_f32.to_bits(),
      line_height_bits: line_height.to_bits(),
      tab_policy: TabDisplayPolicy::default(),
      font_system_version: Version::default(),
    }
  }

  fn soft_wrap_inputs(
    source_line_count: usize,
    line_height: f32,
    key: MeasurementKey,
  ) -> GeometryInputs {
    GeometryInputs {
      source_line_count,
      mode: WrapMode::SoftWrap,
      line_height,
      measurement_key: key,
    }
  }

  fn soft_wrap_result(key: MeasurementKey, counts: &[usize]) -> MeasurementResult {
    MeasurementResult::soft_wrap_line_heights(key, Arc::from(counts))
  }

  fn wrapped_geometry(geometry: &ScrollGeometry) -> &WrappedVerticalGeometry {
    let VerticalGeometry::Wrapped(wrapped) = &geometry.vertical else {
      panic!("expected wrapped vertical geometry");
    };

    wrapped
  }

  #[test]
  fn unknown_horizontal_geometry_clamps_only_to_zero() {
    let geo = HorizontalGeometry::Unknown {
      key: test_measurement_key(),
    };

    assert_eq!(geo.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(geo.clamp_offset(50.0, 300.0), 50.0);
  }

  #[test]
  fn not_scrollable_horizontal_geometry_always_clamps_to_zero() {
    let geo = HorizontalGeometry::Disabled;

    assert_eq!(geo.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(geo.clamp_offset(50.0, 300.0), 0.0);
  }

  #[test]
  fn exact_horizontal_geometry_clamps_to_scrollable_range() {
    let geo = HorizontalGeometry::Exact {
      content_width: 1000.0,
      key: test_measurement_key(),
    };

    assert_eq!(geo.clamp_offset(-10.0, 300.0), 0.0);
    assert_eq!(geo.clamp_offset(50.0, 300.0), 50.0);
    assert_eq!(geo.clamp_offset(900.0, 300.0), 700.0);
    assert_eq!(
      HorizontalGeometry::Exact {
        content_width: 200.0,
        key: test_measurement_key(),
      }
      .clamp_offset(50.0, 300.0),
      0.0
    );
  }

  #[test]
  fn no_wrap_horizontal_geometry_is_unknown_without_measurement() {
    let mut geo = ScrollGeometry::default();
    geo.reconcile(&test_inputs(WrapMode::NoWrap));

    assert_eq!(
      geo.horizontal,
      HorizontalGeometry::Unknown {
        key: test_measurement_key(),
      }
    );
  }

  #[test]
  fn soft_wrap_horizontal_geometry_is_not_scrollable() {
    let mut geo = ScrollGeometry::default();
    geo.reconcile(&test_inputs(WrapMode::SoftWrap));

    assert_eq!(geo.horizontal, HorizontalGeometry::Disabled);
  }

  #[test]
  fn horizontal_wheel_delta_updates_horizontal_scroll() {
    let mut scroll = ScrollModel::default();
    scroll.reset(test_inputs(WrapMode::NoWrap));
    let result = MeasurementResult::no_wrap_horizontal_extent(test_measurement_key(), 1000.0);
    scroll.apply_measurement_result(&result, iced::Size::new(300.0, 100.0));
    let change =
      scroll.apply_wheel_delta(iced::Vector::new(-10.0, 0.0), iced::Size::new(300.0, 100.0));
    assert_eq!(change, Some(ScrollChange::RedrawOnly));
    assert_eq!(scroll.horizontal_px(), 10.0);
  }

  #[test]
  fn soft_wrap_line_heights_result_updates_wrapped_vertical_geometry() {
    let key = soft_wrap_key(1, 320.0, 20.0);
    let mut geometry = ScrollGeometry::default();

    geometry.reconcile(&soft_wrap_inputs(3, 20.0, key));
    geometry.apply_measurement_result(&soft_wrap_result(key, &[1, 3, 2]));

    let wrapped = wrapped_geometry(&geometry);

    assert!(wrapped.fully_measured());
    assert_eq!(wrapped.measured_line_count, 3);
    assert_eq!(wrapped.exact_total_height(), Some(120.0));
    assert_eq!(
      wrapped.per_line,
      vec![
        LineHeightState::Measured { wrap_row_count: 1 },
        LineHeightState::Measured { wrap_row_count: 3 },
        LineHeightState::Measured { wrap_row_count: 2 },
      ]
    );
  }

  #[test]
  fn soft_wrap_line_heights_result_with_stale_key_is_ignored() {
    let current_key = soft_wrap_key(1, 320.0, 20.0);
    let stale_key = soft_wrap_key(1, 480.0, 20.0);
    let mut geometry = ScrollGeometry::default();

    geometry.reconcile(&soft_wrap_inputs(3, 20.0, current_key));
    geometry.apply_measurement_result(&soft_wrap_result(stale_key, &[1, 3, 2]));

    let wrapped = wrapped_geometry(&geometry);

    assert!(!wrapped.fully_measured());
    assert_eq!(wrapped.measured_line_count, 0);
    assert_eq!(wrapped.exact_total_height(), None);
    assert_eq!(
      wrapped.per_line,
      vec![
        LineHeightState::Unknown,
        LineHeightState::Unknown,
        LineHeightState::Unknown,
      ]
    );
  }

  #[test]
  fn soft_wrap_line_heights_result_with_wrong_line_count_is_ignored() {
    let key = soft_wrap_key(1, 320.0, 20.0);
    let mut geometry = ScrollGeometry::default();

    geometry.reconcile(&soft_wrap_inputs(3, 20.0, key));
    geometry.apply_measurement_result(&soft_wrap_result(key, &[1, 3]));

    let wrapped = wrapped_geometry(&geometry);

    assert!(!wrapped.fully_measured());
    assert_eq!(wrapped.measured_line_count, 0);
    assert_eq!(wrapped.exact_total_height(), None);
    assert_eq!(
      wrapped.per_line,
      vec![
        LineHeightState::Unknown,
        LineHeightState::Unknown,
        LineHeightState::Unknown,
      ]
    );
  }
}
