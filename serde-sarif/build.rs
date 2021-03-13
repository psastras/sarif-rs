use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use anyhow::Result;
use schemafy_lib::Expander;
use schemafy_lib::Schema;

// Run `rustfmt` in the input text
fn reformat(
  preamble: impl std::fmt::Display,
  text: impl std::fmt::Display,
) -> Result<String> {
  let mut rustfmt = Command::new("rustfmt")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
  write!(rustfmt.stdin.take().unwrap(), "{}\n\n{}", preamble, text)?;
  let output = rustfmt.wait_with_output()?;
  let stdout = String::from_utf8(output.stdout)?;

  Ok(stdout)
}

fn main() -> Result<()> {
  // Rerun if the schema changes
  println!("cargo:rerun-if-changed=src/schema.json");
  let path = Path::new("src/schema.json");

  // Generate the Rust schema struct
  let json = std::fs::read_to_string(path).unwrap();
  let schema: Schema = serde_json::from_str(&json)?;
  let path_str = path.to_str().unwrap();
  let mut expander = Expander::new(Some("Sarif"), path_str, &schema);
  let x = expander.expand(&schema);

  // Write the struct to the $OUT_DIR/sarif.rs file.
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  let mut file = File::create(out_path.join("sarif.rs"))?;
  let preamble = "
/// Generated file, do not edit by hand.`

use serde::{{Serialize, Deserialize}};
";
  file.write_all(reformat(preamble, x.to_string())?.as_bytes())?;

  // Generate README.md with cargo_readme.
  let mut f = fs::File::open("src/lib.rs").unwrap();
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

  Ok(())
}
