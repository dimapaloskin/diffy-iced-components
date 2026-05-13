use iced::advanced::graphics::text::cosmic_text;

use crate::source_line::SourceLineHeights;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct VisibleTextRow {
  pub(crate) source_line_index: usize,
  pub(crate) wrap_row_index: usize,

  pub(crate) y_inside_source_line: f32,
  pub(crate) document_y: f32,
  pub(crate) viewport_y: f32,

  pub(crate) height: f32,
  pub(crate) width: f32,
}

#[allow(dead_code)]
pub(crate) struct VisibleTextProjection {
  pub(crate) visible_text_size: iced::Size,
  pub(crate) visible_rows: Vec<VisibleTextRow>,
}

impl VisibleTextProjection {
  pub(crate) fn build(
    buffer: &cosmic_text::Buffer,
    source_line_heights: &SourceLineHeights,
    content_size: iced::Size,
  ) -> Self {
    let backend_scroll = buffer.scroll();
    let metrics = buffer.metrics();

    let mut text_width: f32 = 0.0;
    let mut visible_height: f32 = 0.0;
    let mut visible_rows = Vec::new();

    // cosmic-text may start in the middle of a source line.
    // `vertical` is the skipped top part of that line, so its local Y starts negative.
    let mut viewport_y = -backend_scroll.vertical;

    let iter = buffer.lines.iter().enumerate().skip(backend_scroll.line);
    'source_lines: for (source_line_index, line) in iter {
      let source_line_start_document_y =
        source_line_heights.source_line_start_document_y(source_line_index);
      // Basically, layout_opt() is the vector of wrapped lines for a given source line.
      // In our terms, we call it `rows`, but here we stick to the cosmic-text convention.
      let layout_lines = line
        .layout_opt()
        .expect("cosmic-text did not prepare layout for a visible source line");

      let mut y_inside_source_line = 0.0;

      for (wrap_row_index, layout_line) in layout_lines.iter().enumerate() {
        let height = layout_line.line_height_opt.unwrap_or(metrics.line_height);

        let row_bottom = viewport_y + height;
        let is_visible = row_bottom > 0.0 && viewport_y < content_size.height;

        if is_visible {
          let document_y = source_line_start_document_y + y_inside_source_line;
          text_width = text_width.max(layout_line.w);
          visible_height += height;

          visible_rows.push(VisibleTextRow {
            source_line_index,
            wrap_row_index,
            y_inside_source_line,
            document_y,
            viewport_y,
            height,
            width: layout_line.w,
          });
        }

        viewport_y += height;
        y_inside_source_line += height;

        if viewport_y > content_size.height {
          break 'source_lines;
        }
      }
    }

    Self {
      visible_text_size: iced::Size::new(text_width, visible_height),
      visible_rows,
    }
  }
}
