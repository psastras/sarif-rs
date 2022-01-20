use std::io::Write;
use tempfile::NamedTempFile;

pub fn enable_plain_problem_matcher() {
  if is_running_in_gha() && atty::is(atty::Stream::Stdout) {
    let matcher_str = include_str!("problem-matchers/sarif-plain-matcher.json");
    let _ = tempfile::tempfile().and_then(|mut file| {
      file.write_all(matcher_str.as_bytes())?;
      file.sync_all()?;
      Ok(())
    });
    println!("::add-matcher::sarif-plain-matcher.json")
  }
}

pub fn disable_plain_problem_matcher() {
  if is_running_in_gha() && atty::is(atty::Stream::Stdout) {
    println!("::remove-matcher owner=sarif-plain::")
  }
}

pub fn enable_pretty_problem_matcher() {
  println!("IS GHA: {}", is_running_in_gha());
  println!("IS TTY: {}", atty::is(atty::Stream::Stdout));
  if is_running_in_gha() && atty::is(atty::Stream::Stdout) {
    let matcher_str =
      include_str!("problem-matchers/sarif-pretty-matcher.json");
    let _ = NamedTempFile::new().and_then(|mut file| {
      file.write_all(matcher_str.as_bytes())?;
      file.flush()?;
      println!("::add-matcher::{}", file.path().display());
      Ok(())
    });
  }
}

pub fn disable_pretty_problem_matcher() {
  if is_running_in_gha() && atty::is(atty::Stream::Stdout) {
    println!("::remove-matcher owner=sarif-pretty::")
  }
}

fn is_running_in_gha() -> bool {
  match std::env::var("GITHUB_ACTIONS")
    .unwrap_or("false".to_string())
    .as_str()
  {
    "true" => true,
    _ => false,
  }
}
