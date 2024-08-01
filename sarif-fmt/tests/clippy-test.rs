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

  duct_sh::sh(
    "cargo build --bin clippy-sarif",
  )
  .dir(cargo_workspace_directory.clone())
  .run()?;

  duct_sh::sh("cargo build --bin sarif-fmt")
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
      cargo_workspace_directory.clone(),
      PathBuf::from("./target/debug/clippy-sarif"),
    ]
    .iter(),
  ))?;

  let clippy_output = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./sarif-fmt/tests/data/clippy.out"),
    ]
    .iter(),
  ))?;

  let data_dir = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./sarif-fmt/tests/data"),
    ]
    .iter(),
  ))?;

  let cmd = format!(
    "{} -i {} | {}",
    clippy_sarif_bin.to_str().unwrap(),
    clippy_output.to_str().unwrap(),
    sarif_fmt_bin.to_str().unwrap(),
  );

  let output = duct_sh::sh_dangerous(cmd.as_str())
    .dir(data_dir)
    .unchecked()
    .env("NO_COLOR", "1")
    .read()?;

  assert!(output.contains(
    "error: this comparison involving the minimum or maximum element for this type contains a case that is always true or always false"
  ));
  assert!(output.contains("src/main.rs:3:6"));
  assert!(output.contains("if vec.len() <= 0 {}"));
  assert!(output
    .contains("#[deny(clippy::absurd_extreme_comparisons)]` on by default"));
  assert!(output
    .contains("for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#absurd_extreme_comparisons"));

  Ok(())
}
