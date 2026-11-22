//! CLI-106 policy-identity clause only; no whole Requirements marker.
use irig106_time_cli::{ExitClass, run};
use serde_json::{Value, json};
#[test]
fn every_json_view_records_effective_cli_defaults_even_without_findings() {
    // 106-24R1 Fig11-12/13: FMT4/SRC1 DOY10012:00, no findings.
    // Primary header §11.2.1.1: packet36 bytes, Data Length10 (filler2).
    let mut p = vec![0; 36];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2] = 7;
    p[4] = 36;
    p[8] = 10;
    p[15] = 0x11;
    p[24..34].copy_from_slice(&[0x41, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    let path = std::env::temp_dir().join(format!("s17-policy-{}.ch10", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let input = Cleanup(path);
    std::fs::write(&input.0, p).unwrap();
    let expected = json!({
        "id":"telemetryworks/irigtime-cli-nearest-v1","defaults_id":"telemetryworks/irigtime-cli-defaults-v1",
        "ntp_era_source":"none","ntp_era_inference":"disabled",
        "reference_selection":"nearest-reference-counter-offset","drift_correction":false,
        "reference_window_ns":"30000000000","reference_cap":"1024",
        "finding_sample_cap":"20","jump_sample_cap":"20",
        "query_reference_storage":"target-brackets","query_candidates_per_channel":"6",
        "reset_backward_step_ns":"30000000000","reset_backward_comparison":"strictly-greater",
        "reset_run_references":"3","reset_run_order":"strictly-increasing-behind-baseline",
        "leap_table":{"id":"IERS-C72-2026-07-06/6cb6f5d4b819f2e568e25db4b0b26d89dedf031fdffb18bc94d40f4e94e268d7",
            "source":"https://hpiers.obspm.fr/iers/bul/bulc/Leap_Second.dat (Bulletin C72)",
            "supported_start_unix_s":"63072000","known_current_through_unix_s":"1814140799","entries":"28"}});
    for command in ["summary", "channels", "jumps", "timeline", "correlate"] {
        let mut args = vec![command, input.0.to_str().unwrap()];
        if command == "correlate" {
            args.push("0");
        }
        args.extend(["--format", "json", "--fail-on", "never"]);
        if command == "jumps" {
            args.extend(["--threshold-ms", "17"]);
        }
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(run(&args, &mut out, &mut err), ExitClass::Success);
        assert!(err.is_empty());
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert!(v["highest_level"].is_null());
        assert_eq!(v["report"]["policy"], expected, "{command}");
        assert_eq!(
            v["options"]["threshold_ms"],
            match command {
                "summary" => json!("1000"),
                "jumps" => json!("17"),
                _ => Value::Null,
            }
        );
        assert_eq!(v["schema_revision"], 13);
        let schema: Value =
            serde_json::from_str(include_str!("../schema/irigtime-report-v1.schema.json")).unwrap();
        let fields = schema["$defs"]["report_policy"]["properties"]
            .as_object()
            .unwrap();
        assert_eq!(
            v["report"]["policy"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            fields.keys().collect::<Vec<_>>()
        );
    }
}
