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
    Self::with_line_ending_policy(text, LineEndingPolicy::default())
  }

  pub fn with_line_ending_policy(
    text: impl Into<String>,
    line_ending_policy: LineEndingPolicy,
  ) -> Self {
    let text = text.into();
    let source_line_count = count_source_lines(&text);

    Self {
      data: Arc::new(CodeDocumentData {
        id: Self::next_document_id(),
        text,
        line_ending_policy,
        source_line_count,
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
    self.data.source_line_count
  }

  fn next_document_id() -> u64 {
    NEXT_DOCUMENT_ID.fetch_add(1, Ordering::Relaxed)
  }
}

pub(crate) struct CodeDocumentData {
  id: u64,
  text: String,
  line_ending_policy: LineEndingPolicy,
  source_line_count: usize,
}

// consider memchr
fn count_source_lines(text: &str) -> usize {
  if text.is_empty() {
    return 1;
  }

  let bytes = text.as_bytes();
  let mut count = 1;
  let mut i = 0;

  while i < bytes.len() {
    match bytes[i] {
      b'\r' => {
        count += 1;

        if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
          i += 2;
        } else {
          i += 1;
        }
      }
      b'\n' => {
        count += 1;

        if i + 1 < bytes.len() && bytes[i + 1] == b'\r' {
          i += 2;
        } else {
          i += 1;
        }
      }
      _ => {
        i += 1;
      }
    }
  }

  count
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_count_source_lines() {
    assert_eq!(count_source_lines(""), 1);
    assert_eq!(count_source_lines("a"), 1);
    assert_eq!(count_source_lines("\n"), 2);
    assert_eq!(count_source_lines("a\n"), 2);
    assert_eq!(count_source_lines("a\nb"), 2);
    assert_eq!(count_source_lines("\r\n"), 2);
    assert_eq!(count_source_lines("a\n\nb"), 3);
    assert_eq!(count_source_lines("a\r\nb\nc"), 3);
    assert_eq!(count_source_lines("a\n\nb\n\n"), 5);
    assert_eq!(count_source_lines("a\r\nb\nc\n\rd"), 4);
  }
}
