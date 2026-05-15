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
    // TODO: vertical is not done yet
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

  pub(super) fn apply_measurement_result(&mut self, result: &MeasurementResult) {
    if self.key() != Some(result.key) {
      return;
    }

    match result.output {
      MeasurementOutput::NoWrapHorizontalExtent { content_width } => {
        *self = Self::Exact {
          key: result.key,
          content_width,
        }
      }
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
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) enum LineHeightState {
  #[default]
  Unknown,
  #[allow(dead_code)]
  Measured { wrap_row_count: usize },
}

#[cfg(test)]
mod tests {
  use super::super::tests::{test_inputs, test_measurement_key};
  use super::super::{ScrollChange, ScrollModel};
  use super::*;

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
}
