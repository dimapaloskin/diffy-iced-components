use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iced::advanced::graphics::text::{self, cosmic_text};

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
      document.id(),
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
  pub(crate) document_id: u64,
  pub(crate) mode: MeasurementMode,
  pub(crate) font: iced::Font,
  pub(crate) font_size_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) tab_policy: TabDisplayPolicy,
  pub(crate) font_system_version: text::Version,
}

impl MeasurementKey {
  fn new(
    document_id: u64,
    layout_config: LayoutConfig,
    mode: MeasurementMode,
    font_system_version: text::Version,
  ) -> Self {
    Self {
      document_id,
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
    MeasurementMode::NoWrapHorizontal => measure_no_wrap_horizontal(request, cancel),
    MeasurementMode::SoftWrap { .. } => None,
  }
}

fn measure_no_wrap_horizontal(
  request: MeasurementRequest,
  cancel: &AtomicBool,
) -> Option<MeasurementResult> {
  let MeasurementRequest {
    key,
    document,
    layout_config,
  } = request;
  let mut buffer = build_no_wrap_buffer(layout_config, document.text());

  let mut max_width: f32 = 0.0;
  let mut next_line = 0;
  let line_count = buffer.lines.len();

  while next_line < line_count {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    let chunk = font_lock::with_worker_font_system(cancel, |font_system, budget| {
      measure_no_wrap_chunk(&mut buffer, font_system, budget, next_line, cancel)
    })?;

    next_line = chunk.next_line;
    max_width = max_width.max(chunk.max_width);
    thread::yield_now();
  }

  Some(MeasurementResult::no_wrap_horizontal_extent(key, max_width))
}

fn build_no_wrap_buffer(layout_config: LayoutConfig, text: &str) -> cosmic_text::Buffer {
  let metrics = cosmic_text::Metrics::new(layout_config.font_size, layout_config.line_height);
  let attrs = text::to_attributes(layout_config.font);
  let mut buffer = cosmic_text::Buffer::new_empty(metrics);

  buffer.set_wrap(cosmic_text::Wrap::None);
  buffer.set_tab_width(layout_config.tab_display_policy.spaces_per_tab().into());
  buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced, None);

  buffer
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ChunkProgress {
  next_line: usize,
  max_width: f32,
}

fn measure_no_wrap_chunk(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  budget: &font_lock::WorkerFontLockBudget,
  start_line: usize,
  cancel: &AtomicBool,
) -> Option<ChunkProgress> {
  let mut next_line = start_line;
  let mut max_width: f32 = 0.0;

  while next_line < buffer.lines.len() {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    if font_lock::foreground_font_lock_requested() {
      break;
    }

    let line_index = next_line;
    let is_empty = buffer.lines[line_index].text().is_empty();

    next_line += 1;

    if !is_empty {
      max_width = max_width.max(measure_no_wrap_line_width(buffer, font_system, line_index));
    }

    if next_line < buffer.lines.len() && budget.exhausted() {
      break;
    }
  }

  Some(ChunkProgress {
    next_line,
    max_width,
  })
}

fn measure_no_wrap_line_width(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  line_index: usize,
) -> f32 {
  buffer
    .line_layout(font_system, line_index)
    .expect("line index is bounded by buffer.lines.len()")
    .iter()
    .fold(0.0_f32, |max_width, line| max_width.max(line.w))
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
