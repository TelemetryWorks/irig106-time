//! ADR-0019 / API-110 removal boundary. Native replay clauses remain covered
//! separately by s12a2_answers; this single check claims no whole requirement.
use std::process::Command;

#[test]
fn serde_feature_is_rejected_before_compilation() {
    // Cargo metadata performs feature resolution without building anything.
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--features",
            "serde",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cargo metadata");
    assert!(
        !output.status.success(),
        "the removed serde feature was accepted"
    );
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        (error.contains("does not")
            || error.contains("none of the selected packages contains this feature"))
            && error.contains("feature")
            && error.contains("serde"),
        "unexpected rejection: {error}"
    );
}

#[test]
fn library_normal_and_build_tree_remains_serde_free() {
    // Root dev tools may use serde; the shipped library and optional features
    // may not. Cargo tree reports real package names, including renamed edges.
    let output = Command::new(env!("CARGO"))
        .args([
            "tree",
            "-p",
            "irig106-time",
            "--all-features",
            "--edges",
            "normal,build",
            "--prefix",
            "none",
            "--format",
            "{p}",
            "--locked",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo tree");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8(output.stdout).expect("UTF-8 package names");
    let forbidden: Vec<_> = tree
        .lines()
        .filter(|line| {
            matches!(
                line.split_whitespace().next(),
                Some("serde" | "serde_core" | "serde_json")
            )
        })
        .collect();
    assert!(
        forbidden.is_empty(),
        "library JSON dependency leak: {forbidden:?}"
    );
}
