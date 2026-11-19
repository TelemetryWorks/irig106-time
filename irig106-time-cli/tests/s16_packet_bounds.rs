//! Host input-boundary regressions; partial CLI-102/107 and ERR-101 clauses.
use irig106_time_cli::{ExitClass, run};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "irigtime-bounds-{}-{}",
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
fn packet(kind: u8, flags: u8, length: usize, data_length: u32) -> Vec<u8> {
    let mut bytes = vec![0; length];
    bytes[..2].copy_from_slice(&0xEB25u16.to_le_bytes());
    bytes[4..8].copy_from_slice(&(length as u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&data_length.to_le_bytes());
    bytes[14] = flags;
    bytes[15] = kind;
    bytes
}
fn input_error(bytes: &[u8]) {
    let input = Input::new(bytes);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        run(
            &["summary", input.0.to_str().unwrap()],
            &mut Vec::new(),
            &mut Vec::new(),
        )
    }));
    assert_eq!(
        outcome.expect("run must not panic into the host"),
        ExitClass::Input
    );
}
#[test]
fn f2_missing_secondary_header_is_an_input_error_not_host_panic() {
    input_error(&packet(0x12, 0x04, 24, 0));
}
#[test]
fn packet_and_time_payload_truncations_are_input_errors() {
    for kind in [0x11, 0x12] {
        for flags in [0, 4] {
            for length in 24..48 {
                let start = if flags == 4 { 36 } else { 24 };
                let minimum = if kind == 0x11 { 10 } else { 12 };
                if length < start + minimum {
                    input_error(&packet(
                        kind,
                        flags,
                        length,
                        (length.saturating_sub(start)) as u32,
                    ));
                }
            }
        }
    }
    for length in 1..24 {
        input_error(&vec![0; length]);
    }
    let mut broken = packet(0x09, 0, 24, 0);
    broken[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    input_error(&broken);
    broken[4..8].copy_from_slice(&0u32.to_le_bytes());
    input_error(&broken);
}
#[test]
fn declared_data_length_bounds_f1_and_f2_even_with_trailer_bytes() {
    for kind in [0x11, 0x12] {
        input_error(&packet(kind, 0, 48, 0));
        input_error(&packet(kind, 0, 48, u32::MAX));
    }
}

#[cfg(unix)]
mod native_argv {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, process::Command};
    fn native_path(path: &std::path::Path, byte: u8) -> PathBuf {
        let mut name = path.as_os_str().to_os_string();
        name.push(OsString::from_vec(vec![byte]));
        PathBuf::from(name)
    }
    #[test]
    fn non_utf8_input_and_output_paths_work_in_host_and_binary() {
        let mut input = Input::new(&packet(0x09, 0, 24, 0));
        let native = native_path(&input.0, 0xff);
        std::fs::rename(&input.0, &native).unwrap();
        input.0 = native;
        let output = Input(native_path(&input.0, 0xfe));
        let args = [
            OsString::from("csv"),
            input.0.as_os_str().to_owned(),
            OsString::from("--output"),
            output.0.as_os_str().to_owned(),
        ];
        assert_eq!(
            run(&args, &mut Vec::new(), &mut Vec::new()),
            ExitClass::Success
        );
        let bytes = std::fs::read(&output.0).unwrap();
        assert!(bytes.starts_with(b"packet_num,"));
        assert_eq!(bytes.iter().filter(|b| **b == b'\n').count(), 2);
        let status = Command::new(env!("CARGO_BIN_EXE_irigtime"))
            .args(&args)
            .output()
            .unwrap();
        assert!(status.status.success(), "{:?}", status);
        assert_eq!(std::fs::read(&output.0).unwrap(), bytes);
    }
    #[test]
    fn non_utf8_command_and_option_are_usage_errors_not_panics() {
        let input = Input::new(&packet(0x09, 0, 24, 0));
        let bad = OsString::from_vec(vec![b'-', b'-', 0xff]);
        for args in [
            vec![bad.clone()],
            vec![
                OsString::from("summary"),
                input.0.as_os_str().to_owned(),
                bad,
            ],
        ] {
            let result = catch_unwind(AssertUnwindSafe(|| {
                run(&args, &mut Vec::new(), &mut Vec::new())
            }));
            assert_eq!(result.expect("host argv must not panic"), ExitClass::Usage);
            let status = Command::new(env!("CARGO_BIN_EXE_irigtime"))
                .args(&args)
                .output()
                .unwrap();
            assert_eq!(status.status.code(), Some(2));
            assert!(!String::from_utf8_lossy(&status.stderr).contains("panicked"));
        }
    }
}
