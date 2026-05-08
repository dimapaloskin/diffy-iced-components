#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineEndingPolicy {
  #[default]
  PreserveInput,
  Lf,
  Crlf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabDisplayPolicy {
  Spaces(u8),
}

impl TabDisplayPolicy {
  pub const fn normalized(self) -> Self {
    match self {
      Self::Spaces(0) => Self::Spaces(1),
      Self::Spaces(width) => Self::Spaces(width),
    }
  }

  pub const fn spaces_per_tab(self) -> u8 {
    match self.normalized() {
      Self::Spaces(width) => width,
    }
  }
}

impl Default for TabDisplayPolicy {
  fn default() -> Self {
    Self::Spaces(4)
  }
}
