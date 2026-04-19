use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn cosmux() -> Command {
    Command::cargo_bin("cosmux").expect("cosmux binary should build")
}

fn write_pod(dir: &TempDir, name: &str, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(format!("{name}.yaml"));
    fs::write(&path, contents).expect("write pod yaml");
    path
}

#[test]
fn validate_minimal_pod_succeeds() {
    let dir = TempDir::new().unwrap();
    let path = write_pod(
        &dir,
        "smoke",
        r#"
name: smoke
root: "/tmp"
windows:
  - name: w
    panes:
      - cwd: "/tmp"
        command: "true"
"#,
    );
    cosmux()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("OK: 'smoke'"));
}

#[test]
fn validate_rejects_missing_name() {
    let dir = TempDir::new().unwrap();
    let path = write_pod(
        &dir,
        "noname",
        r#"
name: ""
windows:
  - name: w
    panes:
      - cwd: "/tmp"
"#,
    );
    cosmux()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("pod name is required"));
}

#[test]
fn validate_rejects_empty_windows() {
    let dir = TempDir::new().unwrap();
    let path = write_pod(
        &dir,
        "nowin",
        r#"
name: nowin
windows: []
"#,
    );
    cosmux()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("at least one window"));
}

#[test]
fn validate_rejects_empty_panes() {
    let dir = TempDir::new().unwrap();
    let path = write_pod(
        &dir,
        "nopanes",
        r#"
name: nopanes
windows:
  - name: w
    panes: []
"#,
    );
    cosmux()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("at least one pane"));
}

#[test]
fn show_resolves_template_and_emits_yaml() {
    let dir = TempDir::new().unwrap();
    let path = write_pod(
        &dir,
        "shown",
        r#"
name: shown
root: "/tmp"
windows:
  - name: w
    panes:
      - cwd: "/tmp"
        command: "echo from-pane"
"#,
    );
    cosmux()
        .args(["show", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("name: shown"))
        .stdout(predicates::str::contains("command: echo from-pane"));
}

#[test]
fn missing_pod_returns_clear_error() {
    cosmux()
        .args(["validate", "definitely-not-a-real-pod-12345"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no pod config found"));
}

#[test]
fn invalid_yaml_returns_parse_error() {
    let dir = TempDir::new().unwrap();
    let path = write_pod(
        &dir,
        "broken",
        r#"
name: broken
windows: this-should-be-a-list-not-a-string
"#,
    );
    cosmux()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid"));
}
