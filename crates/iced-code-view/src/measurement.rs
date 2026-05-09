use std::sync::TryLockError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iced::advanced::graphics::text::{self, cosmic_text};

use crate::document::CodeDocument;
use crate::font_lock::{font_system_version, foreground_font_lock_requested};
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
}

#[derive(Debug, Clone)]
pub(crate) struct MeasurementRequest {
  pub(crate) key: MeasurementKey,
  pub(crate) document: CodeDocument,
  pub(crate) layout_config: LayoutConfig,
}

impl MeasurementRequest {
  pub(crate) fn new(
    document: &CodeDocument,
    layout_config: LayoutConfig,
    resolved_content_width: f32,
  ) -> Self {
    let mode = MeasurementMode::new(layout_config.wrap_mode, resolved_content_width);
    let key = MeasurementKey::new(document.id(), layout_config, mode, font_system_version());

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
  let key = request.key;
  let mut line_measurer = NoWrapLineMeasurer::new(request.layout_config);

  let mut max_width: f32 = 0.0;

  for line_index in 0..request.document.source_line_count() {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    let line = request
      .document
      .source_line_text(line_index)
      .unwrap_or_default();

    if line.is_empty() {
      thread::yield_now();
      continue;
    }

    max_width = max_width.max(line_measurer.measure_line_width(line, cancel)?);

    thread::yield_now();
  }

  Some(MeasurementResult::no_wrap_horizontal_extent(key, max_width))
}

struct NoWrapLineMeasurer {
  shape: Option<cosmic_text::ShapeLine>,
  layout_scratch: cosmic_text::ShapeBuffer,
  layout_lines: Vec<cosmic_text::LayoutLine>,
  attrs_list: cosmic_text::AttrsList,
  font_size: f32,
  tab_width: u16,
}

impl NoWrapLineMeasurer {
  fn new(layout_config: LayoutConfig) -> Self {
    let attrs = text::to_attributes(layout_config.font);

    Self {
      shape: None,
      layout_scratch: cosmic_text::ShapeBuffer::default(),
      layout_lines: Vec::new(),
      attrs_list: cosmic_text::AttrsList::new(&attrs),
      font_size: layout_config.font_size,
      tab_width: layout_config.tab_display_policy.spaces_per_tab().into(),
    }
  }

  fn measure_line_width(&mut self, line: &str, cancel: &AtomicBool) -> Option<f32> {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    with_worker_font_system(cancel, |font_system| match &mut self.shape {
      Some(shape) => shape.build(
        font_system,
        line,
        &self.attrs_list,
        cosmic_text::Shaping::Advanced,
        self.tab_width,
      ),
      None => {
        self.shape = Some(cosmic_text::ShapeLine::new(
          font_system,
          line,
          &self.attrs_list,
          cosmic_text::Shaping::Advanced,
          self.tab_width,
        ));
      }
    })?;

    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    let shape = self
      .shape
      .as_ref()
      .expect("line shape should be initialized after successful measurement");

    shape.layout_to_buffer(
      &mut self.layout_scratch,
      self.font_size,
      None,
      cosmic_text::Wrap::None,
      cosmic_text::Ellipsize::None,
      None,
      &mut self.layout_lines,
      None,
      cosmic_text::Hinting::default(),
    );

    Some(
      self
        .layout_lines
        .iter()
        .fold(0.0_f32, |max_width, line| max_width.max(line.w)),
    )
  }
}

fn with_worker_font_system<T>(
  cancel: &AtomicBool,
  mut f: impl FnMut(&mut cosmic_text::FontSystem) -> T,
) -> Option<T> {
  // Do not grow the work done under this lock without a separate decision.
  // Lightweight foreground reads like `FontSystem::version()` do not raise the
  // flag and will wait until the worker releases the current write lock.
  loop {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    if foreground_font_lock_requested() {
      thread::yield_now();
      continue;
    }

    match text::font_system().try_write() {
      Ok(mut font_system) => {
        if foreground_font_lock_requested() {
          drop(font_system);
          thread::yield_now();
          continue;
        }

        return Some(f(font_system.raw()));
      }
      Err(TryLockError::WouldBlock) => {
        thread::yield_now();
      }
      Err(TryLockError::Poisoned(_)) => {
        panic!("iced shared font system lock should not be poisoned");
      }
    }
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
