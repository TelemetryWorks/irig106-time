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
fn csv_operational_failure_classes_leave_existing_destination_and_no_temp() {
    for class in [ExitClass::Usage, ExitClass::Input, ExitClass::Output] {
        let f = Files::new(if class == ExitClass::Input {
            &[0; 24]
        } else {
            &common::NO_TIME
        });
        let mut args = f.args();
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
    // Exceed pipe capacity: the child cannot finish before its reader closes.
    let f = Files::new(&common::NO_TIME.repeat(32_768));
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

#[test]
fn csv_publishes_findings_exit_one_just_like_stdout() {
    let f = Files::new(&common::NO_TIME);
    let mut args = f.args();
    args.extend(["--fail-on", "warning"]);
    assert_eq!(
        run(&args, &mut Vec::new(), &mut Vec::new()),
        ExitClass::Findings
    );
    assert!(
        std::fs::read_to_string(&f.output)
            .unwrap()
            .starts_with("packet_num,")
    );
    f.no_temps();
}

#[cfg(unix)]
#[test]
fn new_csv_permissions_follow_child_umask() {
    use std::os::unix::fs::PermissionsExt;
    let f = Files::new(&common::NO_TIME);
    std::fs::remove_file(&f.output).unwrap();
    let result = Command::new("sh")
        .args([
            "-c",
            "umask 027; exec \"$1\" csv \"$2\" --output \"$3\"",
            "sh",
        ])
        .arg(env!("CARGO_BIN_EXE_irigtime"))
        .arg(&f.input)
        .arg(&f.output)
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        std::fs::metadata(&f.output).unwrap().permissions().mode() & 0o777,
        0o640
    );
    f.no_temps();
}

#[cfg(unix)]
#[test]
fn existing_csv_permissions_are_preserved() {
    use std::os::unix::fs::PermissionsExt;
    let f = Files::new(&common::NO_TIME);
    std::fs::set_permissions(&f.output, std::fs::Permissions::from_mode(0o604)).unwrap();
    assert_eq!(
        run(&f.args(), &mut Vec::new(), &mut Vec::new()),
        ExitClass::Success
    );
    assert_eq!(
        std::fs::metadata(&f.output).unwrap().permissions().mode() & 0o777,
        0o604
    );
    f.no_temps();
}

#[cfg(unix)]
#[test]
fn symlink_destination_is_untouched() {
    use std::os::unix::fs::symlink;
    let target = Files::new(&common::NO_TIME);
    let f = Files::new(&common::NO_TIME);
    std::fs::remove_file(&f.output).unwrap();
    symlink(&target.output, &f.output).unwrap();
    assert_eq!(
        run(&f.args(), &mut Vec::new(), &mut Vec::new()),
        ExitClass::Output
    );
    assert!(
        std::fs::symlink_metadata(&f.output)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    target.intact();
    f.no_temps();
}
#[cfg(unix)]
#[test]
fn dev_null_destination_is_untouched() {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let f = Files::new(&common::NO_TIME);
    let before = std::fs::symlink_metadata("/dev/null").unwrap();
    assert_eq!(
        run(
            &["csv", f.input.to_str().unwrap(), "--output", "/dev/null"],
            &mut Vec::new(),
            &mut Vec::new()
        ),
        ExitClass::Output
    );
    let after = std::fs::symlink_metadata("/dev/null").unwrap();
    assert!(after.file_type().is_char_device());
    assert_eq!((after.ino(), after.rdev()), (before.ino(), before.rdev()));
    f.no_temps();
}

#[test]
fn missing_output_directory_precedes_structurally_bad_input() {
    let f = Files::new(&[0; 24]);
    let missing = f.dir.join("missing/report.csv");
    assert_eq!(
        run(
            &[
                "csv",
                f.input.to_str().unwrap(),
                "--output",
                missing.to_str().unwrap()
            ],
            &mut Vec::new(),
            &mut Vec::new()
        ),
        ExitClass::Output
    );
    f.intact();
    assert!(!missing.parent().unwrap().exists());
}
