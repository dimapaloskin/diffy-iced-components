use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iced::advanced::graphics::text::cosmic_text;

use super::buffer::{BufferKind, build_buffer};
use crate::font_lock;

use super::{MeasurementRequest, MeasurementResult};

pub(super) fn measure_line_heights(
  request: MeasurementRequest,
  cancel: &AtomicBool,
) -> Option<MeasurementResult> {
  let MeasurementRequest {
    key,
    document,
    text_layout_config,
    text_content_width,
  } = request;

  let mut buffer = build_buffer(
    text_layout_config,
    document.text(),
    BufferKind::SoftWrap { text_content_width },
  );

  let mut wrap_row_counts = vec![0; buffer.lines.len()];
  let mut next_line = 0;

  while next_line < buffer.lines.len() {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    next_line = font_lock::with_worker_font_system(cancel, |font_system, budget| {
      measure_chunk(
        &mut buffer,
        font_system,
        budget,
        next_line,
        cancel,
        &mut wrap_row_counts,
      )
    })?;

    thread::yield_now();
  }

  let wrap_row_counts: Arc<[usize]> = wrap_row_counts.into();

  Some(MeasurementResult::soft_wrap_line_heights(
    key,
    wrap_row_counts,
  ))
}

fn measure_chunk(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  budget: &font_lock::WorkerFontLockBudget,
  start_line: usize,
  cancel: &AtomicBool,
  wrap_row_counts: &mut [usize],
) -> Option<usize> {
  let mut next_line = start_line;

  while next_line < buffer.lines.len() {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    if font_lock::foreground_font_lock_requested() {
      break;
    }

    wrap_row_counts[next_line] = measure_line_wrap_rows_count(buffer, font_system, next_line);

    next_line += 1;
    if budget.exhausted() && next_line < buffer.lines.len() {
      break;
    }
  }

  Some(next_line)
}

fn measure_line_wrap_rows_count(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  line_index: usize,
) -> usize {
  let metrics = buffer.metrics();

  let layout_lines = buffer
    .line_layout(font_system, line_index)
    .expect("line index is bounded by buffer.lines.len()");

  for layout_line in layout_lines {
    debug_assert!(
      layout_line.line_height_opt.is_none()
        || layout_line.line_height_opt == Some(metrics.line_height)
    );
  }

  layout_lines.len().max(1)
}
