use assert_cmd::Command;
use cargo_metadata::camino::Utf8Path;

pub fn release_cmd(target_dir: &Utf8Path) -> Command {
  let mut cmd = Command::cargo_bin("zen-release").expect("zen-release binary should be built");
  // Run tests in isolation to avoid flakiness
  cmd.env("CARGO_TARGET_DIR", target_dir.as_str());
  cmd.env("ZEN_RELEASE_NO_ANSI", "1");
  cmd
}
