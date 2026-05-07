use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DOCUMENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct CodeDocument {
  data: Arc<CodeDocumentData>,
}

impl CodeDocument {
  pub fn new(text: impl Into<String>) -> Self {
    Self {
      data: Arc::new(CodeDocumentData {
        id: Self::next_document_id(),
        text: text.into(),
      }),
    }
  }

  pub fn id(&self) -> u64 {
    self.data.id
  }

  pub fn text(&self) -> &str {
    &self.data.text
  }

  fn next_document_id() -> u64 {
    NEXT_DOCUMENT_ID.fetch_add(1, Ordering::Relaxed)
  }
}

pub(crate) struct CodeDocumentData {
  id: u64,
  text: String,
}
