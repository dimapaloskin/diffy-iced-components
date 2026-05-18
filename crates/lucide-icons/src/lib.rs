#![forbid(unsafe_code)]

pub const FONT_NAME: &str = "lucide";
pub const LUCIDE_FONT_BYTES: &[u8] = include_bytes!("../assets/lucide.ttf");

include!(concat!(env!("OUT_DIR"), "/icons.rs"));

#[cfg(test)]
mod tests {
  use super::Icon;

  #[test]
  fn exposes_known_icons() {
    assert_eq!(Icon::Check.name(), "check");
    assert_eq!(Icon::Check.glyph(), char::from_u32(57452).unwrap());
    assert_eq!(Icon::from_name("check"), Some(Icon::Check));
    assert_eq!(Icon::from_name("not-a-lucide-icon"), None);
  }
}
