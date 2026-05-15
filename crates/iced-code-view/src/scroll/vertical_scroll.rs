#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VerticalScroll {
  pub(crate) source_line_index: usize,
  pub(crate) y_inside_source_line: f32,
}

impl VerticalScroll {
  pub(crate) const ZERO: Self = Self {
    source_line_index: 0,
    y_inside_source_line: 0.0,
  };

  pub(crate) fn to_cosmic(self) -> cosmic_text::Scroll {
    // Horizontal scroll is applied at draw time, not via cosmic Scroll
    cosmic_text::Scroll::new(self.source_line_index, self.y_inside_source_line, 0.0)
  }

  pub(crate) fn from_cosmic(scroll: cosmic_text::Scroll) -> Self {
    Self {
      source_line_index: scroll.line,
      y_inside_source_line: scroll.vertical,
    }
  }

  pub(crate) fn scrolled_by(self, delta_y: f32) -> Self {
    let y_inside_source_line = self.y_inside_source_line + delta_y;
    // Prevent a no-op wheel at the document top from forcing a relayout.
    let y_inside_source_line = if self.source_line_index == 0 {
      y_inside_source_line.max(0.0)
    } else {
      y_inside_source_line
    };

    Self {
      y_inside_source_line,
      ..self
    }
  }
}

impl Default for VerticalScroll {
  fn default() -> Self {
    Self::ZERO
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn vertical_scroll_at_top_does_not_go_negative() {
    assert_eq!(
      VerticalScroll::ZERO.scrolled_by(-50.0),
      VerticalScroll::ZERO
    );
  }

  #[test]
  fn vertical_scroll_inside_document_can_go_negative() {
    assert_eq!(
      VerticalScroll {
        source_line_index: 5,
        y_inside_source_line: 10.0,
      }
      .scrolled_by(-30.0),
      VerticalScroll {
        source_line_index: 5,
        y_inside_source_line: -20.0,
      }
    );
  }
}
