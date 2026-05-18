pub const FILES: &[(&str, &str)] = &[
  ("demo_1.rs", "./assets/files/demo_1.rs"),
  ("demo_2.rs", "./assets/files/demo_2.rs"),
  ("demo_3.txt", "./assets/files/demo_3.txt"),
];

pub fn read_demo_file(index: usize) -> String {
  let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FILES[index].1);

  std::fs::read_to_string(path).expect("demo file should be readable")
}
