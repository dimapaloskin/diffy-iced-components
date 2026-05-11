use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iced::advanced::graphics::text::{self, cosmic_text};

use crate::font_lock;
use crate::layout::LayoutConfig;

use super::MeasurementRequest;
use super::MeasurementResult;

#[derive(Debug, Clone, Copy, PartialEq)]
struct ChunkProgress {
  next_line: usize,
  max_width: f32,
}

pub(super) fn measure_horizontal_extent(
  request: MeasurementRequest,
  cancel: &AtomicBool,
) -> Option<MeasurementResult> {
  let MeasurementRequest {
    key,
    document,
    layout_config,
  } = request;
  let mut buffer = build_buffer(layout_config, document.text());

  let mut max_width: f32 = 0.0;
  let mut next_line = 0;
  let line_count = buffer.lines.len();

  while next_line < line_count {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    let chunk = font_lock::with_worker_font_system(cancel, |font_system, budget| {
      measure_chunk(&mut buffer, font_system, budget, next_line, cancel)
    })?;

    next_line = chunk.next_line;
    max_width = max_width.max(chunk.max_width);
    thread::yield_now();
  }

  Some(MeasurementResult::no_wrap_horizontal_extent(key, max_width))
}

fn build_buffer(layout_config: LayoutConfig, text: &str) -> cosmic_text::Buffer {
  let metrics = cosmic_text::Metrics::new(layout_config.font_size, layout_config.line_height);
  let attrs = text::to_attributes(layout_config.font);
  let mut buffer = cosmic_text::Buffer::new_empty(metrics);

  buffer.set_wrap(cosmic_text::Wrap::None);
  buffer.set_tab_width(layout_config.tab_display_policy.spaces_per_tab().into());
  buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced, None);

  buffer
}

fn measure_chunk(
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
      max_width = max_width.max(measure_line_width(buffer, font_system, line_index));
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

fn measure_line_width(
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
