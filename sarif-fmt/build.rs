use std::{fs, io::Write, path};

fn main() {
  let mut f = fs::File::open("src/bin.rs").unwrap();
  let content = cargo_readme::generate_readme(
    &path::PathBuf::from("./"),
    &mut f,
    None,
    true,
    true,
    true,
    false,
  )
  .unwrap();

  let mut f = fs::File::create("README.md").unwrap();
  f.write_all(content.as_bytes()).unwrap();
}
