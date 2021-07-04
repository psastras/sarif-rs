use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::predicate;
use serde_sarif::converters::hadolint::parse_to_string;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_basic_lint() -> Result<()> {
  let test_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let mut dockerfile = test_directory.clone();
  dockerfile.push("tests/data/Dockerfile");
  let cmd = format!(
    "nix-shell --run 'hadolint -f json {}'",
    dockerfile.to_str().unwrap()
  );

  let output = duct_sh::sh_dangerous(cmd.as_str())
    .dir(test_directory)
    .unchecked()
    .read()?;

  let sarif = parse_to_string(output.as_bytes())?;
  let mut tmpfile: NamedTempFile = NamedTempFile::new()?;
  tmpfile.write_all(sarif.as_bytes()).unwrap();
  let mut cmd = Command::cargo_bin("sarif-fmt").unwrap();
  cmd.arg(tmpfile.path());
  let assert = cmd.assert().success();
  assert.stderr(predicate::str::contains("Pin versions in npm."));

  Ok(())
}
