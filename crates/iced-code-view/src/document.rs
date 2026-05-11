use std::fmt;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::line_index::LineIndex;
use crate::policies::LineEndingPolicy;

static NEXT_DOCUMENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct Document {
  data: Arc<DocumentData>,
}

impl Document {
  pub fn new(text: impl Into<String>) -> Self {
    // TODO: line ending should be auto-detected
    Self::with_line_ending_policy(text, LineEndingPolicy::default())
  }

  pub fn with_line_ending_policy(
    text: impl Into<String>,
    line_ending_policy: LineEndingPolicy,
  ) -> Self {
    let text = text.into();
    let line_index = LineIndex::new(&text);

    Self {
      data: Arc::new(DocumentData {
        id: Self::next_document_id(),
        text,
        line_ending_policy,
        line_index,
      }),
    }
  }

  pub fn id(&self) -> u64 {
    self.data.id
  }

  pub fn text(&self) -> &str {
    &self.data.text
  }

  pub fn line_ending_policy(&self) -> LineEndingPolicy {
    self.data.line_ending_policy
  }

  pub(crate) fn source_line_count(&self) -> usize {
    self.data.line_index.line_count()
  }

  fn next_document_id() -> u64 {
    NEXT_DOCUMENT_ID.fetch_add(1, Ordering::Relaxed)
  }
}

pub(crate) struct DocumentData {
  id: u64,
  text: String,
  line_ending_policy: LineEndingPolicy,
  line_index: LineIndex,
}

impl fmt::Debug for DocumentData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CodeDocumentData")
      .field("id", &self.id)
      .field("text_len", &self.text.len())
      .field("source_line_count", &self.line_index.line_count())
      .field("line_ending_policy", &self.line_ending_policy)
      .finish()
  }
}
