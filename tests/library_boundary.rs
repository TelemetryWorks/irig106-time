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
        error.contains("does not") && error.contains("feature") && error.contains("serde"),
        "unexpected rejection: {error}"
    );
}
