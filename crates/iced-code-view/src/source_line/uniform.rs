use crate::source_line::SourceLineOffset;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Heights {
  pub(crate) source_line_count: usize,
  pub(crate) line_height: f32,
}

impl Heights {
  pub(crate) fn new(source_line_count: usize, line_height: f32) -> Self {
    Self {
      source_line_count,
      line_height,
    }
  }

  pub(crate) fn resolve_document_y(&self, document_y: f32) -> SourceLineOffset {
    if self.is_invalid() {
      return SourceLineOffset {
        source_line_index: 0,
        source_line_start_document_y: 0.0,
        y_inside_source_line: 0.0,
      };
    }

    let content_height = self.source_line_count as f32 * self.line_height;
    let y = document_y.clamp(0.0, content_height);

    let raw_line = (y / self.line_height).floor() as usize;
    let max_line = self.source_line_count.saturating_sub(1);
    let source_line_index = raw_line.min(max_line);
    let source_line_start_document_y = source_line_index as f32 * self.line_height;

    // End of document can mean `line_height` inside the last line. cosmic-text normalizes scroll after shaping.
    let y_inside_source_line = (y - source_line_start_document_y).clamp(0.0, self.line_height);

    SourceLineOffset {
      source_line_index,
      source_line_start_document_y,
      y_inside_source_line,
    }
  }

  pub(crate) fn source_line_start_document_y(&self, source_line_index: usize) -> f32 {
    if self.is_invalid() {
      return 0.0;
    }

    let max_line = self.source_line_count.saturating_sub(1);
    source_line_index.min(max_line) as f32 * self.line_height
  }

  fn is_invalid(&self) -> bool {
    self.source_line_count == 0 || self.line_height <= 0.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolves_document_y_inside_source_line() {
    let heights = Heights::new(10, 24.0);

    assert_eq!(
      heights.resolve_document_y(25.0),
      SourceLineOffset {
        source_line_index: 1,
        source_line_start_document_y: 24.0,
        y_inside_source_line: 1.0,
      }
    );
  }

  #[test]
  fn clamps_negative_document_y_to_start() {
    let heights = Heights::new(10, 24.0);

    assert_eq!(
      heights.resolve_document_y(-10.0),
      SourceLineOffset {
        source_line_index: 0,
        source_line_start_document_y: 0.0,
        y_inside_source_line: 0.0,
      }
    );
  }

  #[test]
  fn clamps_past_document_end_to_last_source_line() {
    let heights = Heights::new(10, 24.0);

    assert_eq!(
      heights.resolve_document_y(999.0),
      SourceLineOffset {
        source_line_index: 9,
        source_line_start_document_y: 216.0,
        y_inside_source_line: 24.0,
      }
    );
  }
}
