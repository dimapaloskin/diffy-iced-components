mod no_wrap;

use std::sync::atomic::AtomicBool;

use iced::advanced::graphics::text;

use crate::document::Document;
use crate::font_lock;
use crate::policies::TabDisplayPolicy;
use crate::text_layout::{TextLayoutConfig, WrapMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MeasurementKind {
  NoWrapHorizontalExtent,
  SoftWrapLineHeights { content_width_bits: u32 },
}

impl MeasurementKind {
  pub(crate) fn new(wrap_mode: WrapMode, resolved_content_width: f32) -> Self {
    match wrap_mode {
      WrapMode::NoWrap => MeasurementKind::NoWrapHorizontalExtent,
      WrapMode::SoftWrap => MeasurementKind::SoftWrapLineHeights {
        content_width_bits: content_width_bits(resolved_content_width),
      },
    }
  }

  pub(crate) fn needs_background_worker(self) -> bool {
    matches!(self, MeasurementKind::NoWrapHorizontalExtent)
  }
}

#[derive(Debug, Clone)]
pub(crate) struct MeasurementRequest {
  pub(crate) key: MeasurementKey,
  pub(crate) document: Document,
  pub(crate) text_layout_config: TextLayoutConfig,
}

impl MeasurementRequest {
  pub(crate) fn new(
    document: &Document,
    text_layout_config: TextLayoutConfig,
    resolved_content_width: f32,
  ) -> Self {
    let mode = MeasurementKind::new(text_layout_config.wrap_mode, resolved_content_width);
    let key = MeasurementKey::new(
      document.revision(),
      text_layout_config,
      mode,
      font_lock::font_system_version(),
    );

    Self {
      key,
      document: document.clone(),
      text_layout_config,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MeasurementKey {
  pub(crate) document_revision: u64,
  pub(crate) kind: MeasurementKind,
  pub(crate) font: iced::Font,
  pub(crate) font_size_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) tab_policy: TabDisplayPolicy,
  pub(crate) font_system_version: text::Version,
}

impl MeasurementKey {
  fn new(
    document_revision: u64,
    text_layout_config: TextLayoutConfig,
    kind: MeasurementKind,
    font_system_version: text::Version,
  ) -> Self {
    Self {
      document_revision,
      kind,
      font: text_layout_config.font,
      font_size_bits: text_layout_config.font_size.to_bits(),
      line_height_bits: text_layout_config.line_height.to_bits(),
      tab_policy: text_layout_config.tab_display_policy.normalized(),
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
  match request.key.kind {
    MeasurementKind::NoWrapHorizontalExtent => no_wrap::measure_horizontal_extent(request, cancel),
    MeasurementKind::SoftWrapLineHeights { .. } => None,
  }
}

fn content_width_bits(value: f32) -> u32 {
  let width = if value.is_finite() {
    value.max(0.0)
  } else {
    0.0
  };

  width.to_bits()
}

fn sanitize_extent(value: f32) -> f32 {
  if value.is_finite() {
    value.max(0.0)
  } else {
    0.0
  }
}
