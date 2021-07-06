use anyhow::Result;
use std::iter::FromIterator;
use std::path::PathBuf;

#[test]
fn test_basic_lint() -> Result<()> {
  let cargo_manifest_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let cargo_workspace_directory = PathBuf::from_iter(
    [cargo_manifest_directory.clone(), PathBuf::from("..")].iter(),
  );
  let nix_file = PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("nix/hadolint.nix"),
    ]
    .iter(),
  );
  let dockerfile_file = PathBuf::from_iter(
    [
      cargo_manifest_directory.clone(),
      PathBuf::from("tests/data/Dockerfile"),
    ]
    .iter(),
  );

  let cmd = format!(
    "nix-shell --run 'hadolint -f json {} | cargo run -q --bin hadolint-sarif | cargo run -q --bin sarif-fmt' {}",
    dockerfile_file.to_str().unwrap(),
    nix_file.to_str().unwrap(),
  );

  let output = duct_sh::sh_dangerous(cmd.as_str())
    .dir(cargo_workspace_directory)
    .unchecked()
    .stderr_to_stdout()
    .env("NO_COLOR", "1")
    .read()?;

  assert!(output.contains(
    "warning: Always tag the version of an image explicitly
  ┌─ /home/psastras/repos/sarif-rs/sarif-fmt/tests/data/Dockerfile:1:1
  │
1 │ FROM debian
  │ ^^^^^^^^^^^
  │
  = DLDL3006
  = For more information: https://github.com/hadolint/hadolint/wiki/DLDL3006",
  ));

  Ok(())
}
