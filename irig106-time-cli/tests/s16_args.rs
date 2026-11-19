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
    ] {
        let mut args: Vec<_> = case.split_ascii_whitespace().collect();
        args.insert(1, missing.path());
        assert_eq!(invoke(&args), 2, "{case}");
    }
    assert_eq!(invoke(&["--version", "extra"]), 2);
}
