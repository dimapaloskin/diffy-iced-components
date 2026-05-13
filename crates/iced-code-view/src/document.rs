use std::fmt;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::line_index::LineIndex;
use crate::policies::LineEndingPolicy;

static NEXT_DOCUMENT_REVISION: AtomicU64 = AtomicU64::new(1);

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
        revision: Self::next_document_revision(),
        text,
        line_ending_policy,
        line_index,
      }),
    }
  }

  pub(crate) fn revision(&self) -> u64 {
    self.data.revision
  }

  pub fn text(&self) -> &str {
    &self.data.text
  }

  pub fn line_ending_policy(&self) -> LineEndingPolicy {
    self.data.line_ending_policy
  }

  pub fn line_count(&self) -> usize {
    self.data.line_index.line_count()
  }

  fn next_document_revision() -> u64 {
    NEXT_DOCUMENT_REVISION.fetch_add(1, Ordering::Relaxed)
  }
}

pub(crate) struct DocumentData {
  revision: u64,
  text: String,
  line_ending_policy: LineEndingPolicy,
  line_index: LineIndex,
}

impl fmt::Debug for DocumentData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DocumentData")
      .field("revision", &self.revision)
      .field("text_len", &self.text.len())
      .field("source_line_count", &self.line_index.line_count())
      .field("line_ending_policy", &self.line_ending_policy)
      .finish()
  }
}
