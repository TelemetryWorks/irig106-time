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
