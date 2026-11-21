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
        Self::with_bytes(&common::NO_TIME)
    }
    fn with_bytes(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-{}-{}-{}.ch10",
            if cfg!(unix) {
                "quote\",comma,\n"
            } else {
                "comma,"
            },
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
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
    let input = Input::new();
    let (mut default, mut text) = (Vec::new(), Vec::new());
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap()],
            &mut default,
            &mut Vec::new()
        ),
        ExitClass::Success
    );
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap(), "--format", "text"],
            &mut text,
            &mut Vec::new()
        ),
        ExitClass::Success
    );
    assert_eq!(default, text);
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

#[test]
fn missing_year_report_matches_independent_golden() {
    // Fig11-12/13: FMT0/SRC1, day10012:00:00, no year, 10-byte Data Length.
    // §11.2.1.1 header sum EB25+7+24+A+6+1100=FC60; DD filler is excluded.
    let bytes = [
        0x25, 0xeb, 7, 0, 36, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0, 0, 0, 0, 0, 0, 0x60, 0xfc, 1,
        0, 0, 0, 0, 0, 0, 0x12, 0, 1, 0xdd, 0xdd,
    ];
    let input = Input::with_bytes(&bytes);
    let mut out = Vec::new();
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap(), "--format", "json"],
            &mut out,
            &mut Vec::new()
        ),
        ExitClass::Success
    );
    let mut value: Value = serde_json::from_slice(&out).unwrap();
    // Normalize only independently checked run-specific provenance.
    assert_eq!(value["input"]["name"], input.0.to_str().unwrap());
    assert_eq!(value["tool"]["version"], env!("CARGO_PKG_VERSION"));
    value["input"]["name"] = "fixture.ch10".into();
    value["input"]["native_name_hex"] = "666978747572652e63683130".into();
    value["tool"]["version"] = "<package-version>".into();
    assert_eq!(
        serde_json::to_string_pretty(&value).unwrap() + "\n",
        include_str!("golden/s17_missing_year.json")
    );
}
