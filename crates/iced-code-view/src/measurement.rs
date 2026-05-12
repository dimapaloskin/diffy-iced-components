mod no_wrap;

use std::sync::atomic::AtomicBool;

use iced::advanced::graphics::text;

use crate::document::Document;
use crate::font_lock;
use crate::layout::{LayoutConfig, WrapMode};
use crate::policies::TabDisplayPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MeasurementMode {
  NoWrapHorizontal,
  SoftWrap { content_width_px: u32 },
}

impl MeasurementMode {
  pub(crate) fn new(wrap_mode: WrapMode, resolved_content_width: f32) -> Self {
    match wrap_mode {
      WrapMode::NoWrap => MeasurementMode::NoWrapHorizontal,
      WrapMode::SoftWrap => MeasurementMode::SoftWrap {
        content_width_px: quantize_logical_px(resolved_content_width),
      },
    }
  }

  pub(crate) fn needs_background_worker(self) -> bool {
    matches!(self, MeasurementMode::NoWrapHorizontal)
  }
}

#[derive(Debug, Clone)]
pub(crate) struct MeasurementRequest {
  pub(crate) key: MeasurementKey,
  pub(crate) document: Document,
  pub(crate) layout_config: LayoutConfig,
}

impl MeasurementRequest {
  pub(crate) fn new(
    document: &Document,
    layout_config: LayoutConfig,
    resolved_content_width: f32,
  ) -> Self {
    let mode = MeasurementMode::new(layout_config.wrap_mode, resolved_content_width);
    let key = MeasurementKey::new(
      document.revision(),
      layout_config,
      mode,
      font_lock::font_system_version(),
    );

    Self {
      key,
      document: document.clone(),
      layout_config,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MeasurementKey {
  pub(crate) document_revision: u64,
  pub(crate) mode: MeasurementMode,
  pub(crate) font: iced::Font,
  pub(crate) font_size_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) tab_policy: TabDisplayPolicy,
  pub(crate) font_system_version: text::Version,
}

impl MeasurementKey {
  fn new(
    document_revision: u64,
    layout_config: LayoutConfig,
    mode: MeasurementMode,
    font_system_version: text::Version,
  ) -> Self {
    Self {
      document_revision,
      mode,
      font: layout_config.font,
      font_size_bits: layout_config.font_size.to_bits(),
      line_height_bits: layout_config.line_height.to_bits(),
      tab_policy: layout_config.tab_display_policy.normalized(),
      font_system_version,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MeasurementOutput {
  NoWrapHorizontalExtent { content_width: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MeasurementResult {
  pub(crate) key: MeasurementKey,
  pub(crate) output: MeasurementOutput,
}

impl MeasurementResult {
  pub(crate) fn no_wrap_horizontal_extent(key: MeasurementKey, content_width: f32) -> Self {
    Self {
      key,
      output: MeasurementOutput::NoWrapHorizontalExtent {
        content_width: sanitize_extent(content_width),
      },
    }
  }
}

pub(crate) fn measure_document(
  request: MeasurementRequest,
  cancel: &AtomicBool,
) -> Option<MeasurementResult> {
  match request.key.mode {
    MeasurementMode::NoWrapHorizontal => no_wrap::measure_horizontal_extent(request, cancel),
    MeasurementMode::SoftWrap { .. } => None,
  }
}

fn quantize_logical_px(value: f32) -> u32 {
  if !value.is_finite() || value <= 0.0 {
    return 0;
  }

  value.round().min(u32::MAX as f32) as u32
}

fn sanitize_extent(value: f32) -> f32 {
  if value.is_finite() {
    value.max(0.0)
  } else {
    0.0
  }
}
