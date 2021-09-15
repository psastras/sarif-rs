struct ProblemMatchers {
  problem_matcher: Vec<ProblemMatcher>,
}

struct ProblemMatcher {
  owner: String,
  pattern: Vec<ProblemMatcherPattern>,
}

struct ProblemMatcherPattern {
  regexp: String,
  severity: i32,
  file: i32,
  line: i32,
  column: i32,
  message: i32,
}

pub fn enable_problem_matcher() {
  if is_running_in_gha() && atty::is(atty::Stream::Stdout) {
    let matcher_str = include_str!("problem-matchers/sarif-plain-matcher.json");

    println!("::add-matcher::sarif-plain-matcher.json")
  }
}

pub fn disable_problem_matcher() {
  if is_running_in_gha() && atty::is(atty::Stream::Stdout) {
    println!("::remove-matcher owner=sarif-plain::")
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
