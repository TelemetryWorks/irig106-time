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
        assert_eq!(v["schema_status"], "stable");
        assert_eq!(v["tool"]["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(v["input"]["name"], input.0.to_str().unwrap());
        assert_eq!(
            v["input"]["native_name_encoding"],
            if cfg!(windows) {
                "windows-utf16le"
            } else {
                "posix-bytes"
            }
        );
        let hex = v["input"]["native_name_hex"].as_str().unwrap();
        let native = native_bytes(input.0.as_os_str());
        assert_eq!(hex.len(), 2 * native.len());
        let decoded: Vec<u8> = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        assert_eq!(decoded, native);
        assert_eq!(v["input"]["size_bytes"], "24");
        assert_eq!(v["options"]["command"], command);
        assert_eq!(v["options"]["fail_on"], "warning");
        assert_eq!(v["schema"], "telemetryworks/irigtime-report");
        assert_eq!(v["schema_revision"], 7);
        assert!(v["options"]["limit"].is_null());
        assert!(v["report"]["rows"].is_null());
        assert_eq!(v["highest_level"], "warning");
        assert_eq!(
            v["options"]["threshold_ms"],
            if command == "channels" {
                Value::Null
            } else {
                "1000".into()
            }
        );
        assert_eq!(v["report"]["jumps"].is_null(), command == "channels");
        assert_eq!(v["report"]["total_packets"], "1");
        assert_eq!(v["report"]["time_packets"], "0");
        assert_eq!(v["findings"]["warning"], "1");
        assert_eq!(v["findings"]["omitted"], "0");
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
    let native_hex: String = native_bytes(input.0.as_os_str())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(value["input"]["native_name_hex"], native_hex);
    assert_eq!(value["tool"]["version"], env!("CARGO_PKG_VERSION"));
    value["input"]["name"] = "fixture.ch10".into();
    value["input"]["native_name_hex"] = "666978747572652e63683130".into();
    value["tool"]["version"] = "<package-version>".into();
    assert_eq!(value["tool"]["library_version"], env!("CARGO_PKG_VERSION"));
    value["tool"]["library_version"] = "<package-version>".into();
    assert_eq!(
        value["tool"]["source_revision"],
        serde_json::json!(option_env!("IRIGTIME_SOURCE_REVISION").or(option_env!("GITHUB_SHA")))
    );
    value["tool"]["source_revision"] = Value::Null;
    assert_eq!(
        value["input"]["native_name_encoding"],
        if cfg!(windows) {
            "windows-utf16le"
        } else {
            "posix-bytes"
        }
    );
    value["input"]["native_name_encoding"] = "posix-bytes".into();
    assert_eq!(
        serde_json::to_string_pretty(&value).unwrap() + "\n",
        include_str!("golden/s17_missing_year.json")
    );
}

#[cfg(unix)]
fn native_bytes(name: &std::ffi::OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    name.as_bytes().to_vec()
}
#[cfg(windows)]
fn native_bytes(name: &std::ffi::OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    name.encode_wide().flat_map(u16::to_le_bytes).collect()
}

fn packet(channel: u16, kind: u8, rtc: u64, body: &[u8]) -> Vec<u8> {
    // §11.2.1.1 header checksum, Data Length excludes alignment filler.
    let mut p = vec![0xdd; (24 + body.len()).next_multiple_of(4)];
    p[..24].fill(0);
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&channel.to_le_bytes());
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8..12].copy_from_slice(&(body.len() as u32).to_le_bytes());
    p[12] = 6;
    p[15] = kind;
    p[16..22].copy_from_slice(&rtc.to_le_bytes()[..6]);
    let sum = p[..22].chunks_exact(2).fold(0u16, |sum, w| {
        sum.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p[24..24 + body.len()].copy_from_slice(body);
    p
}
fn f1(channel: u16, rtc_seconds: u64, seconds: u64, source: u8) -> Vec<u8> {
    // Fig11-12/13, printed11-23/25: DOY100, FMT0, SRC=source.
    // First body word fractions=0; second has seconds/minutes, third hour/day.
    let bcd = |v: u64| ((v / 10) * 16 + v % 10) as u8;
    packet(
        channel,
        0x11,
        rtc_seconds * 10_000_000,
        &[
            source,
            0,
            0,
            0,
            0,
            bcd(seconds % 60),
            bcd(seconds / 60 % 60),
            bcd(seconds / 3600),
            0,
            1,
        ],
    )
}
#[test]
fn reset_segments_are_complete_and_channels_jumps_are_not_evaluated() {
    let mut bytes = Vec::new();
    for n in 0..1200 {
        bytes.extend(f1(7, n, n * 6, 2));
    }
    // Interleaved channel plus a reset on 7; new first/last must not mix segments.
    bytes.extend(f1(8, 1200, 7200, 15));
    bytes.extend(f1(7, 1, 7201, 2));
    bytes.extend(f1(7, 3, 7203, 2));
    // Figure11-15 NTF0/TS3, nonzero raw CSDW, unreadable TS finding.
    bytes.extend(packet(
        9,
        0x12,
        40_000_000,
        &[3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    ));
    bytes.extend([0xff; 24]); // Info-level trailing bytes after the stream.
    let input = Input::with_bytes(&bytes);
    for command in ["summary", "channels", "jumps"] {
        let mut out = Vec::new();
        assert_eq!(
            run(
                &[
                    command,
                    input.0.to_str().unwrap(),
                    "--format",
                    "json",
                    "--fail-on",
                    "info"
                ],
                &mut out,
                &mut Vec::new()
            ),
            ExitClass::Findings
        );
        let v: Value = serde_json::from_slice(&out).unwrap();
        let c = &v["report"]["channels"][0];
        assert_eq!(c["segment_count"], "2");
        let segments = c["segments"].as_array().unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0]["index"], "1");
        assert_eq!(segments[0]["first_rtc"], "0");
        assert_eq!(segments[0]["last_rtc"], "11990000000");
        assert_eq!(segments[1]["index"], "2");
        assert_eq!(segments[1]["first_rtc"], "10000000");
        assert_eq!(segments[1]["last_rtc"], "30000000");
        assert_eq!(c["first_rtc"], "0");
        assert_eq!(c["last_rtc"], "30000000");
        assert_eq!(v["report"]["rtc_duration_ns"], Value::Null);
        assert_eq!(v["report"]["references_evicted"], "1202");
        assert_eq!(v["report"]["reset_count"], "1");
        assert_eq!(v["findings"]["info"], "1");
        assert_eq!(
            v["findings"]["warning"],
            if command == "channels" { "2" } else { "1201" }
        );
        if command == "channels" {
            assert!(v["report"]["jumps"].is_null());
        } else {
            assert_eq!(v["report"]["jumps"]["count"], "1199");
            assert_eq!(v["report"]["jumps"]["omitted"], "1179");
        }
    }
}
#[test]
fn ascii_formatter_preserves_unicode_control_and_surrogate_pairs() {
    let input = Input::new();
    let parent = input.0.parent().unwrap();
    let named = parent.join(format!(
        "caf\u{e9}-\u{1f680}-{}-quote\".ch10",
        std::process::id()
    ));
    // Windows forbids a quote in a filename; its native-name path is covered by
    // the platform-specific projection fixture instead.
    #[cfg(windows)]
    let named = named.with_file_name(format!("caf\u{e9}-\u{1f680}-{}.ch10", std::process::id()));
    std::fs::write(&named, common::NO_TIME).unwrap();
    let named = Input(named);
    let mut out = Vec::new();
    run(
        &["summary", named.0.to_str().unwrap(), "--format", "json"],
        &mut out,
        &mut Vec::new(),
    );
    assert!(out.is_ascii());
    let text = std::str::from_utf8(&out).unwrap();
    assert!(text.contains("caf\\u00e9-\\ud83d\\ude80"));
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["input"]["name"], named.0.to_str().unwrap());
}

#[test]
fn schema_describes_every_key_and_matches_the_golden_shape() {
    // A structural contract check, not a general-purpose JSON Schema validator.
    // It prevents the schema and producer from drifting on keys/closed enums.
    let schema: Value =
        serde_json::from_str(include_str!("../schema/irigtime-report-v1.schema.json")).unwrap();
    let golden: Value = serde_json::from_str(include_str!("golden/s17_missing_year.json")).unwrap();
    fn descriptions(s: &Value) {
        if let Some(properties) = s.get("properties").and_then(Value::as_object) {
            for field in properties.values() {
                assert!(field["description"].is_string());
            }
            assert_eq!(s["additionalProperties"], false);
            assert_eq!(s["required"].as_array().unwrap().len(), properties.len());
        }
        match s {
            Value::Object(o) => {
                for v in o.values() {
                    descriptions(v);
                }
            }
            Value::Array(a) => {
                for v in a {
                    descriptions(v);
                }
            }
            _ => {}
        }
    }
    fn shape(v: &Value, s: &Value, root: &Value) {
        if let Some(reference) = s.get("$ref").and_then(Value::as_str) {
            return shape(v, root.pointer(&reference[1..]).unwrap(), root);
        }
        if let Some(choices) = s.get("anyOf").and_then(Value::as_array) {
            if v.is_null() {
                assert!(choices.iter().any(|c| c["type"] == "null"));
                return;
            }
            return shape(v, &choices[0], root);
        }
        if let Some(expected) = s.get("const") {
            assert_eq!(v, expected);
        }
        if let Some(allowed) = s.get("enum").and_then(Value::as_array) {
            assert!(allowed.contains(v), "{v} not in {allowed:?}");
        }
        if let Some(o) = v.as_object() {
            let fields = s["properties"].as_object().unwrap();
            assert_eq!(
                o.keys().collect::<Vec<_>>(),
                fields.keys().collect::<Vec<_>>()
            );
            for (name, value) in o {
                shape(value, &fields[name], root);
            }
        }
        if let Some(a) = v.as_array() {
            for value in a {
                shape(value, &s["items"], root);
            }
        }
    }
    // AbsoluteTime's native calendar explicitly permits year zero.
    assert_eq!(schema["$defs"]["clock"]["properties"]["year"]["minimum"], 0);
    descriptions(&schema);
    shape(&golden, &schema, &schema);
    assert!(schema["$id"].as_str().unwrap().contains("/v0.8.0/"));
    assert!(!schema["$id"].as_str().unwrap().contains("/main/"));
    assert!(
        schema["description"]
            .as_str()
            .unwrap()
            .contains("Consumers MUST ignore unknown fields")
    );
}

fn invalid_status_finding(status: u8) -> Value {
    // Figure11-15: NTF0 (NTPv3), raw TS nibble; Figure11-16:
    // seconds1/fraction0. Status is checked before needing an NTP era.
    let input = Input::with_bytes(&packet(
        9,
        0x12,
        0,
        &[status, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    ));
    let mut out = Vec::new();
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
            &mut out,
            &mut Vec::new()
        ),
        ExitClass::Findings
    );
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["findings"]["warning"], "1");
    assert_eq!(v["report"]["channels"][0]["status_raw"], status);
    v["findings"]["samples"][0].clone()
}
#[test]
fn time_not_valid_status_uses_tf003() {
    // TF catalog: TS0 is Time Not Valid, not a reserved value.
    let finding = invalid_status_finding(0);
    assert_eq!(finding["code"], "TF-003");
    assert_eq!(finding["level"], "warning");
}
#[test]
fn reserved_time_status_values_use_tf006() {
    // TF catalog / Figure11-15: TS2..15 are reserved values.
    for status in 2..=15 {
        let finding = invalid_status_finding(status);
        assert_eq!(finding["code"], "TF-006", "TS={status}");
        assert_eq!(finding["level"], "warning");
    }
}
#[test]
fn jump_finding_pins_code_channel_and_packet_not_reference_index() {
    // Figure11-13: at RTC+1s native clock advances6s -> +5s residual.
    // Opaque data between references distinguishes packet2 from reference1.
    let bytes = [f1(7, 0, 0, 1), packet(8, 9, 5_000_000, &[]), f1(7, 1, 6, 1)].concat();
    let input = Input::with_bytes(&bytes);
    for command in ["summary", "jumps"] {
        let mut out = Vec::new();
        assert_eq!(
            run(
                &[command, input.0.to_str().unwrap(), "--format", "json"],
                &mut out,
                &mut Vec::new()
            ),
            ExitClass::Success
        );
        let v: Value = serde_json::from_slice(&out).unwrap();
        let findings = v["findings"]["samples"].as_array().unwrap();
        let jump = findings
            .iter()
            .find(|v| v["message"] == "Time jump on channel 7 at reference 1")
            .unwrap();
        assert_eq!(jump["code"], "TF-017");
        assert_eq!(jump["channel"], 7);
        assert_eq!(jump["packet"], "2");
        assert_eq!(jump["level"], "warning");
    }
}
