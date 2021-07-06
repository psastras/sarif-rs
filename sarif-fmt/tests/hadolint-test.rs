use anyhow::Result;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;

#[test]
// Test that the happy path linting works
fn test_hadolint() -> Result<()> {
  let cargo_manifest_directory =
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")))?;
  let cargo_workspace_directory = fs::canonicalize(PathBuf::from_iter(
    [cargo_manifest_directory.clone(), PathBuf::from("..")].iter(),
  ))?;

  let nix_file = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("nix/hadolint.nix"),
    ]
    .iter(),
  ))?;

  let dockerfile_file = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_manifest_directory.clone(),
      PathBuf::from("tests/data/Dockerfile"),
    ]
    .iter(),
  ))?;

  duct_sh::sh(
    "RUSTFLAGS='-Z instrument-coverage' cargo +nightly build --bin hadolint-sarif",
  )
  .dir(cargo_workspace_directory.clone())
  .run()?;

  duct_sh::sh(
    "RUSTFLAGS='-Z instrument-coverage' cargo +nightly build --bin sarif-fmt",
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

  let hadolint_sarif_bin = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./target/debug/hadolint-sarif"),
    ]
    .iter(),
  ))?;

  let cmd = format!(
    "nix-shell --run 'hadolint -f json {} | {} | {}' {}",
    dockerfile_file.to_str().unwrap(),
    hadolint_sarif_bin.to_str().unwrap(),
    sarif_fmt_bin.to_str().unwrap(),
    nix_file.to_str().unwrap(),
  );

  let output = duct_sh::sh_dangerous(cmd.as_str())
    .dir(cargo_workspace_directory)
    .unchecked()
    .env("NO_COLOR", "1")
    .read()?;

  assert!(
    output.contains("warning: Always tag the version of an image explicitly")
  );
  assert!(output.contains("Dockerfile:1:1"));
  assert!(output.contains("FROM debian"));
  assert!(output.contains("DL3006"));
  assert!(output.contains(
    "For more information: https://github.com/hadolint/hadolint/wiki/DL3006"
  ));

  Ok(())
}
