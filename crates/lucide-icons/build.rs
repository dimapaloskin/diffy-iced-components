use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
  println!("cargo:rerun-if-changed=assets/codepoints.json");
  println!("cargo:rerun-if-changed=assets/lucide.ttf");

  let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let codepoints_path = manifest_dir.join("assets/codepoints.json");
  let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("icons.rs");

  let codepoints = std::fs::read_to_string(&codepoints_path).unwrap_or_else(|error| {
    panic!(
      "failed to read {}: {error}",
      codepoints_path.to_string_lossy()
    )
  });

  let codepoints: BTreeMap<String, u32> =
    serde_json::from_str(&codepoints).unwrap_or_else(|error| {
      panic!(
        "failed to parse {}: {error}",
        codepoints_path.to_string_lossy()
      )
    });

  let icons = codepoints
    .iter()
    .map(|(name, codepoint)| {
      let variant = variant_name(name);

      if char::from_u32(*codepoint).is_none() {
        panic!("invalid Lucide codepoint for {name}: {codepoint}");
      }

      (name.as_str(), variant, *codepoint)
    })
    .collect::<Vec<_>>();

  assert_unique_variants(&icons);

  let mut generated = String::new();

  writeln!(
    generated,
    "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]"
  )
  .unwrap();
  writeln!(generated, "#[allow(missing_docs)]").unwrap();
  writeln!(generated, "pub enum Icon {{").unwrap();

  for (_, variant, _) in &icons {
    writeln!(generated, "  {variant},").unwrap();
  }

  writeln!(generated, "}}").unwrap();
  writeln!(generated).unwrap();
  writeln!(generated, "impl Icon {{").unwrap();
  writeln!(generated, "  pub const ALL: &'static [Self] = &[").unwrap();

  for (_, variant, _) in &icons {
    writeln!(generated, "    Self::{variant},").unwrap();
  }

  writeln!(generated, "  ];").unwrap();
  writeln!(generated).unwrap();
  writeln!(generated, "  #[must_use]").unwrap();
  writeln!(generated, "  pub const fn glyph(self) -> char {{").unwrap();
  writeln!(generated, "    match self {{").unwrap();

  for (_, variant, codepoint) in &icons {
    writeln!(
      generated,
      "      Self::{variant} => '\\u{{{codepoint:x}}}',"
    )
    .unwrap();
  }

  writeln!(generated, "    }}").unwrap();
  writeln!(generated, "  }}").unwrap();
  writeln!(generated).unwrap();
  writeln!(generated, "  #[must_use]").unwrap();
  writeln!(generated, "  pub const fn name(self) -> &'static str {{").unwrap();
  writeln!(generated, "    match self {{").unwrap();

  for (name, variant, _) in &icons {
    writeln!(generated, "      Self::{variant} => {name:?},").unwrap();
  }

  writeln!(generated, "    }}").unwrap();
  writeln!(generated, "  }}").unwrap();
  writeln!(generated).unwrap();
  writeln!(generated, "  #[must_use]").unwrap();
  writeln!(
    generated,
    "  pub fn from_name(name: &str) -> Option<Self> {{"
  )
  .unwrap();
  writeln!(generated, "    match name {{").unwrap();

  for (name, variant, _) in &icons {
    writeln!(generated, "      {name:?} => Some(Self::{variant}),").unwrap();
  }

  writeln!(generated, "      _ => None,").unwrap();
  writeln!(generated, "    }}").unwrap();
  writeln!(generated, "  }}").unwrap();
  writeln!(generated, "}}").unwrap();

  std::fs::write(&out_path, generated)
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", out_path.to_string_lossy()));
}

fn variant_name(name: &str) -> String {
  let mut variant = String::new();
  let mut previous_part_was_numeric = false;

  for part in name.split(|character: char| !character.is_ascii_alphanumeric()) {
    if part.is_empty() {
      continue;
    }

    let part_is_numeric = part.chars().all(|character| character.is_ascii_digit());

    if previous_part_was_numeric && part_is_numeric {
      variant.push('_');
    }

    let mut characters = part.chars();
    let Some(first) = characters.next() else {
      continue;
    };

    if variant.is_empty() && first.is_ascii_digit() {
      variant.push_str("Icon");
      variant.push(first);
    } else if first.is_ascii_digit() {
      variant.push(first);
    } else {
      variant.push(first.to_ascii_uppercase());
    }

    for character in characters {
      variant.push(character);
    }

    previous_part_was_numeric = part_is_numeric;
  }

  if variant.is_empty() {
    panic!("Lucide icon name produced empty Rust variant: {name:?}");
  }

  variant
}

fn assert_unique_variants(icons: &[(&str, String, u32)]) {
  let mut variants = HashSet::new();

  for (name, variant, _) in icons {
    if !variants.insert(variant.as_str()) {
      panic!("duplicate Rust variant generated for Lucide icon {name:?}: {variant}");
    }
  }
}
