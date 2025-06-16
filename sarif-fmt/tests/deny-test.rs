use anyhow::Result;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;

#[test]
// Test that the happy path linting works
fn test_deny() -> Result<()> {
  let cargo_manifest_directory =
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")))?;
  let cargo_workspace_directory = fs::canonicalize(PathBuf::from_iter(
    [cargo_manifest_directory.clone(), PathBuf::from("..")].iter(),
  ))?;

  duct_sh::sh(
    "cargo build --bin deny-sarif",
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

  let deny_sarif_bin = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./target/debug/deny-sarif"),
    ]
    .iter(),
  ))?;

  let deny_output = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./sarif-fmt/tests/data/deny.out"),
    ]
    .iter(),
  ))?;

  let cmd = format!(
    "{} -i {} | {}",
    deny_sarif_bin.to_str().unwrap(),
    deny_output.to_str().unwrap(),
    sarif_fmt_bin.to_str().unwrap(),
  );

  let output = duct_sh::sh_dangerous(cmd.as_str())
    .dir(cargo_workspace_directory)
    .unchecked()
    .env("NO_COLOR", "1")
    .read()?;

  assert!(output.contains("warning: Package in deny list"));
  assert!(output.contains("Cargo.toml:1:1"));
  assert!(output.contains("tokio"));

  Ok(())
}