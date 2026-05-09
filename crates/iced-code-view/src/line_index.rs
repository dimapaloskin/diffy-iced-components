use std::ops::Range;

use memchr::memchr2_iter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LineIndex {
  line_starts: Vec<usize>,
  text_len: usize,
}

impl LineIndex {
  pub(crate) fn new(text: &str) -> Self {
    let bytes = text.as_bytes();
    let mut line_starts = vec![0];

    let mut iter = memchr2_iter(b'\n', b'\r', bytes);

    while let Some(index) = iter.next() {
      let mut next_line_start = index + 1;

      if index + 1 < bytes.len() {
        let current = bytes[index];
        let next = bytes[index + 1];

        if (current == b'\r' && next == b'\n') || (current == b'\n' && next == b'\r') {
          let _ = iter.next();
          next_line_start = index + 2;
        }
      }

      line_starts.push(next_line_start);
    }

    Self {
      line_starts,
      text_len: text.len(),
    }
  }

  pub(crate) fn line_count(&self) -> usize {
    self.line_starts.len()
  }

  pub(crate) fn line_start(&self, line_index: usize) -> Option<usize> {
    self.line_starts.get(line_index).copied()
  }

  pub(crate) fn byte_range_for_lines(&self, lines: Range<usize>) -> Option<Range<usize>> {
    if lines.start > lines.end || lines.end > self.line_count() {
      return None;
    }

    let start = self.line_start(lines.start)?;
    let end = if lines.end == self.line_count() {
      self.text_len
    } else {
      self.line_start(lines.end)?
    };

    Some(start..end)
  }

  pub(crate) fn line_for_byte(&self, byte_offset: usize) -> Option<usize> {
    if byte_offset > self.text_len {
      return None;
    }

    match self.line_starts.binary_search(&byte_offset) {
      Ok(line) => Some(line),
      Err(next_line) => Some(next_line.saturating_sub(1)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builds_line_starts_for_empty_text_and_line_endings() {
    let empty = LineIndex::new("");
    assert_eq!(empty.line_count(), 1);
    assert_eq!(empty.line_start(0), Some(0));
    assert_eq!(empty.byte_range_for_lines(0..1), Some(0..0));

    let index = LineIndex::new("a\r\nb\nc");
    assert_eq!(index.line_count(), 3);
    assert_eq!(index.line_start(0), Some(0));
    assert_eq!(index.line_start(1), Some(3));
    assert_eq!(index.line_start(2), Some(5));
  }

  #[test]
  fn byte_range_for_lines_uses_exclusive_line_end() {
    let index = LineIndex::new("abc\nde\nxyz");

    assert_eq!(index.byte_range_for_lines(0..1), Some(0..4));
    assert_eq!(index.byte_range_for_lines(1..3), Some(4..10));
    assert_eq!(index.byte_range_for_lines(2..3), Some(7..10));
    assert_eq!(index.byte_range_for_lines(3..4), None);
  }

  #[test]
  fn line_for_byte_returns_containing_line() {
    let index = LineIndex::new("abc\nde\nxyz");

    assert_eq!(index.line_for_byte(0), Some(0));
    assert_eq!(index.line_for_byte(3), Some(0));
    assert_eq!(index.line_for_byte(4), Some(1));
    assert_eq!(index.line_for_byte(6), Some(1));
    assert_eq!(index.line_for_byte(7), Some(2));
    assert_eq!(index.line_for_byte(10), Some(2));
    assert_eq!(index.line_for_byte(11), None);
  }
}
