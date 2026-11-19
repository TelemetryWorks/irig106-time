//! Partial CLI-104/107 CSV transaction clauses, not whole markers.
#[path = "common/mod.rs"]
mod common;
use irig106_time_cli::{ExitClass, run};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Files {
    dir: PathBuf,
    input: PathBuf,
    output: PathBuf,
}
impl Files {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "irigtime-csv-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        let input = dir.join("input.ch10");
        let output = dir.join("report.csv");
        std::fs::write(&input, bytes).unwrap();
        std::fs::write(&output, b"original").unwrap();
        Self { dir, input, output }
    }
    fn args(&self) -> Vec<&str> {
        vec![
            "csv",
            self.input.to_str().unwrap(),
            "--output",
            self.output.to_str().unwrap(),
        ]
    }
    fn intact(&self) {
        assert_eq!(std::fs::read(&self.output).unwrap(), b"original");
        self.no_temps();
    }
    fn no_temps(&self) {
        let names: Vec<_> = std::fs::read_dir(&self.dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert!(
            names.iter().all(|n| n == "input.ch10" || n == "report.csv"),
            "{names:?}"
        );
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
struct Broken(bool);
impl Write for Broken {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0 {
            Ok(bytes.len())
        } else {
            Err(io::Error::other("write probe"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("flush probe"))
    }
}
#[test]
fn csv_replaces_existing_file_only_after_success() {
    let f = Files::new(&common::NO_TIME);
    assert_eq!(
        run(&f.args(), &mut Vec::new(), &mut Vec::new()),
        ExitClass::Success
    );
    let report = std::fs::read_to_string(&f.output).unwrap();
    assert_eq!(report.lines().count(), 2);
    assert!(report.starts_with("packet_num,offset_hex,"));
    f.no_temps();
}
#[test]
fn csv_failure_classes_leave_existing_destination_and_no_temp() {
    for class in [
        ExitClass::Findings,
        ExitClass::Usage,
        ExitClass::Input,
        ExitClass::Output,
    ] {
        let f = Files::new(if class == ExitClass::Input {
            &[0; 24]
        } else {
            &common::NO_TIME
        });
        let mut args = f.args();
        if class == ExitClass::Findings {
            args.extend(["--fail-on", "warning"]);
        }
        if class == ExitClass::Usage {
            args.extend(["--fail-on", "bogus"]);
        }
        let actual = if class == ExitClass::Output {
            run(&args, &mut Vec::new(), &mut Broken(false))
        } else {
            run(&args, &mut Vec::new(), &mut Vec::new())
        };
        assert_eq!(actual, class);
        f.intact();
    }
    for fail_stdout in [true, false] {
        let f = Files::new(&common::NO_TIME);
        let actual = if fail_stdout {
            run(&f.args(), &mut Broken(true), &mut Vec::new())
        } else {
            run(&f.args(), &mut Vec::new(), &mut Broken(true))
        };
        assert_eq!(actual, ExitClass::Output);
        f.intact();
    }
}
#[test]
fn stdout_csv_uses_and_cleans_its_isolated_system_temp_directory() {
    for bytes in [&common::NO_TIME[..], &[0; 24][..]] {
        let f = Files::new(bytes);
        let temp = f.dir.join("temp");
        std::fs::create_dir(&temp).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_irigtime"))
            .args(["csv", f.input.to_str().unwrap()])
            .env("TMPDIR", &temp)
            .env("TMP", &temp)
            .env("TEMP", &temp)
            .output()
            .unwrap();
        assert_eq!(
            result.status.code(),
            Some(if bytes == common::NO_TIME { 0 } else { 3 })
        );
        assert_eq!(std::fs::read_dir(&temp).unwrap().count(), 0);
        if bytes == common::NO_TIME {
            assert_eq!(String::from_utf8(result.stdout).unwrap().lines().count(), 2);
        } else {
            assert!(result.stdout.is_empty());
        }
    }
}
#[test]
fn csv_replacement_preserves_hard_link_to_previous_contents() {
    let f = Files::new(&common::NO_TIME);
    let previous = f.dir.join("previous.csv");
    std::fs::hard_link(&f.output, &previous).unwrap();
    assert_eq!(
        run(&f.args(), &mut Vec::new(), &mut Vec::new()),
        ExitClass::Success
    );
    assert_eq!(std::fs::read(previous).unwrap(), b"original");
    // In-place truncation would alter both links; replacement must not.
}
#[test]
fn stdout_csv_without_a_usable_temp_directory_returns_output() {
    let f = Files::new(&common::NO_TIME);
    let missing = f.dir.join("absent");
    let result = Command::new(env!("CARGO_BIN_EXE_irigtime"))
        .args(["csv", f.input.to_str().unwrap()])
        .env("TMPDIR", &missing)
        .env("TMP", &missing)
        .env("TEMP", &missing)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(4));
    assert!(result.stdout.is_empty());
    assert!(!Path::new(&missing).exists());
}

#[test]
fn broken_stdout_pipe_cleans_the_system_temp_spool() {
    let f = Files::new(&common::NO_TIME);
    let temp = f.dir.join("temp");
    std::fs::create_dir(&temp).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_irigtime"))
        .args(["csv", f.input.to_str().unwrap()])
        .env("TMPDIR", &temp)
        .env("TMP", &temp)
        .env("TEMP", &temp)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(4));
    assert_eq!(std::fs::read_dir(&temp).unwrap().count(), 0);
}
