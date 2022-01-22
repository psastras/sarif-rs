use anyhow::Result;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;

#[test]
// Test that the happy path linting works
fn test_clippy() -> Result<()> {
  let cargo_manifest_directory =
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")))?;
  let cargo_workspace_directory = fs::canonicalize(PathBuf::from_iter(
    [cargo_manifest_directory.clone(), PathBuf::from("..")].iter(),
  ))?;

  let clippy_project_directory = fs::canonicalize(PathBuf::from_iter(
    [cargo_manifest_directory, PathBuf::from("./tests/data")].iter(),
  ))?;

  duct_sh::sh(
    "RUSTFLAGS='-Z instrument-coverage' cargo +nightly-2022-01-14 build --bin clippy-sarif",
  )
  .dir(cargo_workspace_directory.clone())
  .run()?;

  duct_sh::sh(
    "RUSTFLAGS='-Z instrument-coverage' cargo +nightly-2022-01-14 build --bin sarif-fmt",
  )
  .dir(cargo_workspace_directory.clone())
  .run()?;

  let sarif_fmt_bin = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./target/debug/sarif-fmt"),
    ]
    .iter(),
  ))?;

  let clippy_sarif_bin = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory,
      PathBuf::from("./target/debug/clippy-sarif"),
    ]
    .iter(),
  ))?;

  let cmd = format!(
    "cargo clippy --message-format=json | {} | {}",
    clippy_sarif_bin.to_str().unwrap(),
    sarif_fmt_bin.to_str().unwrap()
  );

  let output = duct_sh::sh_dangerous(cmd.as_str())
    .dir(clippy_project_directory)
    .unchecked()
    .env("NO_COLOR", "1")
    .read()?;

  assert!(output.contains(
    "warning: this comparison involving the minimum or maximum element for this type contains a case that is always true or always false"
  ));
  assert!(output.contains("src/main.rs:3:6"));
  assert!(output.contains("if vec.len() <= 0 {}"));
  assert!(output
    .contains("#[deny(clippy::absurd_extreme_comparisons)]` on by default"));
  assert!(output
    .contains("for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#absurd_extreme_comparisons"));

  Ok(())
}
