use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;

#[test]
// Test that the happy path linting works
fn test_clang_tidy() -> Result<()> {
  let cargo_manifest_directory =
    fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")))?;
  let cargo_workspace_directory = fs::canonicalize(PathBuf::from_iter(
    [cargo_manifest_directory.clone(), PathBuf::from("..")].iter(),
  ))?;

  duct_sh::sh(
    "cargo build --bin clang-tidy-sarif",
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

  let clang_tidy_sarif_bin = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./target/debug/clang-tidy-sarif"),
    ]
    .iter(),
  ))?;

  let clang_tidy_output = fs::canonicalize(PathBuf::from_iter(
    [
      cargo_workspace_directory.clone(),
      PathBuf::from("./sarif-fmt/tests/data/clang-tidy.out"),
    ]
    .iter(),
  ))?;

  let cmd = format!(
    "{} {} | {}",
    clang_tidy_sarif_bin.to_str().unwrap(),
    clang_tidy_output.to_str().unwrap(),
    sarif_fmt_bin.to_str().unwrap(),
  );

  let mut env_map: HashMap<_, _> = std::env::vars().collect();
  env_map.insert("NO_COLOR".into(), "1".into());

  let output = duct_sh::sh_dangerous(cmd.as_str())
    .dir(cargo_workspace_directory)
    .unchecked()
    .full_env(&env_map)
    .read()?;

  assert!(output.contains("warning: Array access (from variable 'str') results in a null pointer dereference [clang-analyzer-core.NullDereference]"));
  assert!(output.contains("cpp.cpp:8:10"));
  assert!(output.contains("return str[0];"));
  // 1st note for the above error
  assert!(output.contains("cpp.cpp:12:25"));
  assert!(output.contains("return get_first_char(nullptr);"));
  assert!(output.contains("Passing null pointer value via 1st parameter 'str'"));
  // 2nd note, same line of code
  assert!(output.contains("cpp.cpp:12:10"));
  assert!(output.contains("Calling 'get_first_char'"));
  // 3rd note, same line of code as the original error
  assert!(output.contains("cpp.cpp:8:10"));
  assert!(output.contains("------- Array access (from variable 'str') results in a null pointer dereference"));

  Ok(())
}
