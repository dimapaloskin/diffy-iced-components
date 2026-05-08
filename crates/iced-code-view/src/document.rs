use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::policies::LineEndingPolicy;

static NEXT_DOCUMENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct CodeDocument {
  data: Arc<CodeDocumentData>,
}

impl CodeDocument {
  pub fn new(text: impl Into<String>) -> Self {
    // TODO: line ending should be auto-detected
    Self::with_line_endings_policy(text, LineEndingPolicy::default())
  }

  pub fn with_line_endings_policy(
    text: impl Into<String>,
    line_ending_policy: LineEndingPolicy,
  ) -> Self {
    Self {
      data: Arc::new(CodeDocumentData {
        id: Self::next_document_id(),
        text: text.into(),
        line_ending_policy,
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

  fn next_document_id() -> u64 {
    NEXT_DOCUMENT_ID.fetch_add(1, Ordering::Relaxed)
  }
}

pub(crate) struct CodeDocumentData {
  id: u64,
  text: String,
  line_ending_policy: LineEndingPolicy,
}
