mod uniform;

use crate::WrapMode;
use crate::layout::LayoutRequest;

pub(crate) enum SourceLineHeights {
  Uniform(uniform::Heights),
}

impl SourceLineHeights {
  pub(crate) fn for_request(request: &LayoutRequest<'_>) -> Self {
    match request.config.wrap_mode {
      WrapMode::NoWrap => Self::Uniform(uniform::Heights::new(
        request.document.source_line_count(),
        request.config.line_height,
      )),
      WrapMode::SoftWrap => {
        unimplemented!("SoftWrap source-line heights need measured height cache")
      }
    }
  }

  pub(crate) fn resolve_document_y(&self, document_y: f32) -> SourceLineOffset {
    match self {
      Self::Uniform(heights) => heights.resolve_document_y(document_y),
    }
  }

  pub(crate) fn source_line_start_document_y(&self, source_line_index: usize) -> f32 {
    match self {
      Self::Uniform(heights) => heights.source_line_start_document_y(source_line_index),
    }
  }
}

// Result of mapping absolute document Y to a source line and offset inside it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SourceLineOffset {
  // Source line / cosmic-text BufferLine index
  pub(crate) source_line_index: usize,
  // Absolute document Y where source line starts.
  pub(crate) source_line_start_document_y: f32,
  // Y offset from the start of this source line.
  pub(crate) y_inside_source_line: f32,
}

impl SourceLineOffset {
  pub(crate) fn to_cosmic_scroll(self) -> cosmic_text::Scroll {
    cosmic_text::Scroll::new(self.source_line_index, self.y_inside_source_line, 0.0)
  }
}
