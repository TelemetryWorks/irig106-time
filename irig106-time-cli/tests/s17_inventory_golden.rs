//! Partial CLI-105/106: native GPS/network inventory output, not TMATS joining.
use irig106_time_cli::{ExitClass, run};
use serde_json::Value;
use std::{ffi::OsStr, path::PathBuf};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn packet(channel: u16, kind: u8, seconds: u64, body: &[u8]) -> Vec<u8> {
    // §11.2.1.1: primary header LE word-sum; 100ns RTC, no secondary header.
    let mut p = vec![0xdd; (24 + body.len()).next_multiple_of(4)];
    p[..24].fill(0);
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&channel.to_le_bytes());
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8..12].copy_from_slice(&(body.len() as u32).to_le_bytes());
    p[12] = 6;
    p[15] = kind;
    p[16..22].copy_from_slice(&(seconds * 10_000_000).to_le_bytes()[..6]);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p[24..24 + body.len()].copy_from_slice(body);
    p
}
#[cfg(unix)]
fn native(name: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    name.as_bytes().to_vec()
}
#[cfg(windows)]
fn native(name: &OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    name.encode_wide().flat_map(u16::to_le_bytes).collect()
}
#[test]
fn native_gps_and_every_network_protocol_match_full_inventory_golden() {
    // 106-24R1 Fig11-12/13 (11-23/25): SRC1/FMT5/DATE0, DOY100 12:00:00.
    let mut bytes = packet(7, 0x11, 0, &[0x51, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]);
    // Fig11-15/16: NTF0/TS1; RFC5905 Fig4 epoch1970=2208988800,
    // fraction0x80000000=0.5s. Legacy CLI era-context limitation is explicit.
    bytes.extend(packet(
        8,
        0x12,
        1,
        &[1, 0, 0, 0, 0x80, 0x7e, 0xaa, 0x83, 0, 0, 0, 0x80],
    ));
    // Fig11-15/17 (11-27/PDF35): NTF1/NTF2, TS1; Unix2017-01-02
    // =1483315200, plus37 TAI-UTC ->1483315237 TAI;123ns retained.
    for (channel, ntf) in [(9, 1), (10, 2)] {
        let mut body = vec![(ntf << 4) | 1, 0, 0, 0];
        body.extend(1483315237u32.to_le_bytes());
        body.extend(123u32.to_le_bytes());
        bytes.extend(packet(channel, 0x12, u64::from(channel - 7), &body));
    }
    // Figure11-15 NTF3 is reserved; preserve its raw selector, no guessed scale.
    bytes.extend(packet(
        11,
        0x12,
        4,
        &[0x31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ));
    let input =
        Fixture(std::env::temp_dir().join(format!("s17-inventory-{}.ch10", std::process::id())));
    std::fs::write(&input.0, &bytes).unwrap();
    let mut out = Vec::new();
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap(), "--format", "json"],
            &mut out,
            &mut Vec::new()
        ),
        ExitClass::Success
    );
    assert!(out.is_ascii());
    let mut v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["input"]["name"], input.0.to_str().unwrap());
    let hex: String = native(input.0.as_os_str())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(v["input"]["native_name_hex"], hex);
    assert_eq!(
        v["input"]["native_name_encoding"],
        if cfg!(windows) {
            "windows-utf16le"
        } else {
            "posix-bytes"
        }
    );
    assert_eq!(v["tool"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(v["tool"]["library_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        v["tool"]["source_revision"],
        serde_json::json!(option_env!("IRIGTIME_SOURCE_REVISION").or(option_env!("GITHUB_SHA")))
    );
    v["input"]["name"] = "fixture.ch10".into();
    v["input"]["native_name_hex"] = "666978747572652e63683130".into();
    v["input"]["native_name_encoding"] = "posix-bytes".into();
    v["tool"]["version"] = "<package-version>".into();
    v["tool"]["library_version"] = "<package-version>".into();
    v["tool"]["source_revision"] = Value::Null;
    assert_eq!(
        serde_json::to_string_pretty(&v).unwrap() + "\n",
        include_str!("golden/s17_protocol_inventory.json")
    );
}
