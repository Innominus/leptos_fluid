use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture_manifest(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("ui")
        .join(relative)
        .join("Cargo.toml")
}

fn cargo_check(manifest_path: &Path) -> std::process::Output {
    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("macro_ui");

    Command::new(env!("CARGO"))
        .arg("check")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_path)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("failed to run cargo check for macro UI fixture")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn macro_api_pass_fixture_builds() {
    let output = cargo_check(&fixture_manifest("pass/controller_timeline_macros"));
    assert!(
        output.status.success(),
        "expected pass fixture to build successfully, stderr was:\n{}",
        stderr(&output)
    );
}

#[test]
fn macro_api_rejects_missing_controller_attachment() {
    let output = cargo_check(&fixture_manifest("fail/controller_missing_attachment"));
    assert!(
        !output.status.success(),
        "expected fixture to fail compilation"
    );
    assert!(
        stderr(&output).contains("controller! requires exactly one of `target:` or `resolver:`"),
        "unexpected stderr:\n{}",
        stderr(&output)
    );
}

#[test]
fn macro_api_rejects_target_and_resolver_together() {
    let output = cargo_check(&fixture_manifest("fail/controller_target_and_resolver"));
    assert!(
        !output.status.success(),
        "expected fixture to fail compilation"
    );
    assert!(
        stderr(&output).contains("controller! accepts either `target:` or `resolver:`, not both"),
        "unexpected stderr:\n{}",
        stderr(&output)
    );
}

#[test]
fn macro_api_requires_timeline_steps() {
    let output = cargo_check(&fixture_manifest("fail/timeline_missing_steps"));
    assert!(
        !output.status.success(),
        "expected fixture to fail compilation"
    );
    assert!(
        stderr(&output).contains("timeline! requires a non-empty `steps:` field"),
        "unexpected stderr:\n{}",
        stderr(&output)
    );
}

#[test]
fn macro_api_requires_step_targets() {
    let output = cargo_check(&fixture_manifest("fail/timeline_step_missing_to"));
    assert!(
        !output.status.success(),
        "expected fixture to fail compilation"
    );
    assert!(
        stderr(&output).contains("timeline! step requires a `to:` field"),
        "unexpected stderr:\n{}",
        stderr(&output)
    );
}

#[test]
fn macro_api_rejects_unknown_step_fields() {
    let output = cargo_check(&fixture_manifest("fail/timeline_unknown_field"));
    assert!(
        !output.status.success(),
        "expected fixture to fail compilation"
    );
    assert!(
        stderr(&output).contains("unknown field in timeline! step: `delay_ms`"),
        "unexpected stderr:\n{}",
        stderr(&output)
    );
}

#[test]
fn builder_api_pass_fixture_builds() {
    let output = cargo_check(&fixture_manifest("pass/typed_builders"));
    assert!(
        output.status.success(),
        "expected builder fixture to build successfully, stderr was:\n{}",
        stderr(&output)
    );
}

#[test]
fn builder_api_requires_controller_attachment_before_install() {
    let output = cargo_check(&fixture_manifest(
        "fail/builder_controller_missing_attachment",
    ));
    assert!(
        !output.status.success(),
        "expected fixture to fail compilation"
    );
    assert!(
        stderr(&output).contains("no method named `install`"),
        "unexpected stderr:\n{}",
        stderr(&output)
    );
}

#[test]
fn builder_api_requires_timeline_steps_before_install() {
    let output = cargo_check(&fixture_manifest("fail/builder_timeline_missing_step"));
    assert!(
        !output.status.success(),
        "expected fixture to fail compilation"
    );
    assert!(
        stderr(&output).contains("no method named `install`"),
        "unexpected stderr:\n{}",
        stderr(&output)
    );
}
