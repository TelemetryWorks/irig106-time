//! Strict argument preflight only; threshold/complete exit work follows.
use irig106_time_cli::run;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "irigtime-exits-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn invoke(args: &[&str]) -> u8 {
    run(args, &mut Vec::new(), &mut Vec::new()) as u8
}
#[test]
fn exit_two_validates_all_usage_before_opening_input() {
    let missing = Input::new(&[]);
    std::fs::remove_file(&missing.0).unwrap();
    for case in [
        "jumps --threshold-ms bad",
        "jumps --threshold-ms -1",
        "jumps --threshold-ms 18446744073709551615",
        "jumps --threshold-ms 18446744073710",
        "jumps --threshold-ms",
        "jumps --threshold-ms 1 --threshold-ms 2",
        "timeline --limit bad",
        "timeline --limit 0",
        "timeline --limit -1",
        "timeline --limit +1",
        "timeline --limit 18446744073709551616",
        "timeline --limit",
        "timeline --limit 1 --limit 2",
        "summary --unknown",
        "summary extra",
        "summary --limit 1",
        "csv --output",
        "csv --output a --output b",
        "correlate 1000000000000",
        "correlate +1",
        "correlate not-hex",
    ] {
        let mut args: Vec<_> = case.split_ascii_whitespace().collect();
        args.insert(1, missing.path());
        assert_eq!(invoke(&args), 2, "{case}");
    }
    assert_eq!(invoke(&["--version", "extra"]), 2);
}

#[test]
fn grammar_bounds_and_command_help_are_explicit() {
    use irig106_time_cli::args::parse;
    for args in [
        vec!["summary", "input"],
        vec!["channels", "input"],
        vec!["jumps", "input", "--threshold-ms", "0"],
        vec!["jumps", "input", "--threshold-ms", "18446744073709"],
        vec!["timeline", "input", "--limit", "1"],
        vec!["csv", "input", "--output", "-out.csv"],
        vec!["correlate", "input", "0xFFFFFFFFFFFF"],
    ] {
        assert!(parse(&args).is_ok(), "{args:?}");
    }
    assert!(parse(&["summary"]).is_err());
    assert!(parse(&["correlate", "input", ""]).is_err());
    assert!(parse(&["--help", "extra"]).is_err());
    assert!(parse(&["summary", "input", "--fail-on", "error"]).is_ok());
    for command in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        assert_eq!(invoke(&[command, "--help"]), 0);
    }
}
#[test]
fn terminator_opens_a_real_dash_leading_filename_without_changing_host_cwd() {
    struct Directory(PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let file = Input::new(&[]);
    let dir = Directory(file.0.with_extension("dir"));
    std::fs::create_dir(&dir.0).unwrap();
    std::fs::write(dir.0.join("-capture file.ch10"), []).unwrap();
    for (args, expected) in [
        (["summary", "--", "-capture file.ch10"].to_vec(), 0),
        (["summary", "-capture file.ch10"].to_vec(), 2),
    ] {
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_irigtime"))
            .current_dir(&dir.0)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(expected));
    }
}
