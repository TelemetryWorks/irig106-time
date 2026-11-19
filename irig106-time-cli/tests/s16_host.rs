//! S16a2 host/writer clauses; full CLI-102/107 evidence awaits strict args/exits.
use irig106_time_cli::{ExitClass, run};
use std::{io, process::Command};

struct FailedWriter(bool);
impl io::Write for FailedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0 {
            Ok(bytes.len())
        } else {
            Err(io::Error::other("injected writer"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected flush"))
    }
}

#[test]
fn help_and_version_are_host_callable_and_use_only_injected_writers() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        run(&["--help"], &mut stdout, &mut stderr),
        ExitClass::Success
    );
    assert!(String::from_utf8_lossy(&stdout).contains("irigtime summary"));
    stdout.clear();
    assert_eq!(
        run(&["--version"], &mut stdout, &mut stderr),
        ExitClass::Success
    );
    assert_eq!(
        stdout,
        concat!("irigtime ", env!("CARGO_PKG_VERSION"), "\n").as_bytes()
    );
    assert!(stderr.is_empty());
    let standalone = Command::new(env!("CARGO_BIN_EXE_irigtime"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(standalone.stdout, stdout);
}

#[test]
fn writer_failure_returns_output_class_and_host_can_continue() {
    let mut diagnostics = Vec::new();
    assert_eq!(
        run(&["--version"], &mut FailedWriter(false), &mut diagnostics),
        ExitClass::Output
    );
    assert!(String::from_utf8_lossy(&diagnostics).contains("injected writer"));
    assert_eq!(
        run(&["--version"], &mut FailedWriter(true), &mut Vec::new()),
        ExitClass::Output
    );
    assert_eq!(
        run(&["--help"], &mut Vec::new(), &mut Vec::new()),
        ExitClass::Success
    );
}

#[test]
fn invalid_command_returns_usage_and_input_failure_survives_diagnostic_failure() {
    assert_eq!(
        run(&["unknown"], &mut Vec::new(), &mut Vec::new()),
        ExitClass::Usage
    );
    assert_eq!(
        run(
            &["summary", "\0"],
            &mut Vec::new(),
            &mut FailedWriter(false)
        ),
        ExitClass::Input
    );
}
