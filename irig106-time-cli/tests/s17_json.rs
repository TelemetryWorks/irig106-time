//! CLI-106 JSON subset: no whole-requirement coverage marker.
#[path = "common/mod.rs"]
mod common;
use irig106_time_cli::{ExitClass, run};
use serde_json::Value;
use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-quote\"-{}-{}.ch10",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, common::NO_TIME).unwrap();
        Self(path)
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
#[test]
fn versioned_report_is_deterministic_exact_and_carries_provenance() {
    let input = Input::new();
    for command in ["summary", "channels", "jumps"] {
        let args = [
            command,
            input.0.to_str().unwrap(),
            "--format",
            "json",
            "--fail-on",
            "warning",
        ];
        let (mut first, mut second) = (Vec::new(), Vec::new());
        assert_eq!(run(&args, &mut first, &mut Vec::new()), ExitClass::Findings);
        assert_eq!(
            run(&args, &mut second, &mut Vec::new()),
            ExitClass::Findings
        );
        assert_eq!(first, second);
        let v: Value = serde_json::from_slice(&first).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["schema_status"], "draft");
        assert_eq!(v["tool"]["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(v["input"]["name"], input.0.to_str().unwrap());
        assert_eq!(v["input"]["size_bytes"], "24");
        assert_eq!(v["options"]["command"], command);
        assert_eq!(v["options"]["fail_on"], "warning");
        assert_eq!(v["units"]["rtc"], "100 ns ticks");
        assert_eq!(
            v["units"]["clock_count"],
            "ns since day 1 00:00:00; year unspecified"
        );
        assert_eq!(v["report"]["total_packets"], "1");
        assert_eq!(v["report"]["time_packets"], "0");
        assert_eq!(v["report"]["findings"]["warning"], "1");
        assert_eq!(v["report"]["findings"]["omitted"], "0");
        assert_eq!(v["report"]["channels"], serde_json::json!([]));
        assert_eq!(
            v["report"]["data_types"],
            serde_json::json!([{"id":9,"packets":"1"}])
        );
    }
}
#[test]
fn json_options_are_strict_and_precede_input_access() {
    for options in [
        vec!["--format", "JSON"],
        vec!["--format", "json", "--format", "text"],
        vec!["--format"],
    ] {
        let mut args = vec!["summary", "/missing-input"];
        args.extend(options);
        assert_eq!(
            run(&args, &mut Vec::new(), &mut Vec::new()),
            ExitClass::Usage
        );
    }
    assert_eq!(
        run(
            &["csv", "/missing-input", "--format", "json"],
            &mut Vec::new(),
            &mut Vec::new()
        ),
        ExitClass::Usage
    );
}
#[test]
fn json_writer_failure_is_output_not_findings() {
    struct Broken;
    impl io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("JSON writer"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new();
    assert_eq!(
        run(
            &[
                "summary",
                input.0.to_str().unwrap(),
                "--format",
                "json",
                "--fail-on",
                "warning"
            ],
            &mut Broken,
            &mut Vec::new()
        ),
        ExitClass::Output
    );
}
