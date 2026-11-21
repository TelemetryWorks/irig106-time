//! ADR-0019 / API-110 removal boundary. Native replay clauses remain covered
//! separately by s12a2_answers; this single check claims no whole requirement.
use std::{
    path::Path,
    process::{Command, Output},
};

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

fn library_tree(root: &Path) -> Output {
    Command::new(env!("CARGO"))
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
        ])
        .current_dir(root)
        .output()
        .expect("cargo tree")
}

#[test]
fn library_normal_and_build_tree_remains_serde_free() {
    // Dev tools do not ship; every normal/build edge and library feature must
    // remain serde-free. Real package names expose renamed dependency edges.
    let output = library_tree(Path::new(env!("CARGO_MANIFEST_DIR")));
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

#[test]
fn target_specific_edges_are_checked_but_dev_edges_are_excluded() {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let cases = [
        (r#"target.'cfg(windows)'.dependencies"#, "serde", false),
        (
            r#"target.'cfg(target_arch = "wasm32")'.dependencies"#,
            "serde_json",
            false,
        ),
        (
            r#"target.'cfg(target_os = "none")'.build-dependencies"#,
            "serde_core",
            false,
        ),
        ("dev-dependencies", "serde", true),
    ];
    for (section, package, dev) in cases {
        let path = std::env::temp_dir().join(format!(
            "irigtime-boundary-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("exclusive owned fixture directory");
        let root = Fixture(path);
        fs::create_dir(root.0.join("stub")).unwrap();
        fs::write(root.0.join("lib.rs"), "").unwrap();
        fs::write(root.0.join("stub/lib.rs"), "").unwrap();
        fs::write(
            root.0.join("stub/Cargo.toml"),
            format!(
                r#"[package]
name = "{package}"
version = "0.0.0"
[lib]
path = "lib.rs"
"#
            ),
        )
        .unwrap();
        fs::write(
            root.0.join("Cargo.toml"),
            format!(
                r#"[workspace]
[package]
name = "irig106-time"
version = "0.0.0"
[lib]
path = "lib.rs"
[{section}]
alias = {{ package = "{package}", path = "stub" }}
"#
            ),
        )
        .unwrap();
        let lock = Command::new(env!("CARGO"))
            .args(["generate-lockfile", "--offline"])
            .current_dir(&root.0)
            .output()
            .unwrap();
        assert!(
            lock.status.success(),
            "{}",
            String::from_utf8_lossy(&lock.stderr)
        );
        let output = library_tree(&root.0);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let tree = String::from_utf8(output.stdout).unwrap();
        let present = tree
            .lines()
            .any(|line| line.split_whitespace().next() == Some(package));
        assert_eq!(present, !dev, "{section}: {tree}");
    }
}
