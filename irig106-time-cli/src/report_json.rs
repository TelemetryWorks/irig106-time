//! CLI-owned draft v1 report JSON (ADR-0019; owner approval 2026-11-20).
//! Partial Traces: L3-CLI-106 <- L2-CLI-106 <- L1-CLI-006; no loading API.
use super::{args::Args, report::Report, *};
use serde_json::{Value, json};

pub(super) fn write(r: &Report<'_>, args: &Args, size: u64, out: &mut dyn Write) -> io::Result<()> {
    let mut value = document(r, args, size);
    value.sort_all_objects(); // Canonical order even if a host enables preserve_order.
    serde_json::to_writer(&mut *out, &value).map_err(io::Error::other)?;
    writeln!(out)
}
fn nonzero(days: u16) -> Option<u16> {
    (days != 0).then_some(days)
}
fn decimal(value: impl std::fmt::Display) -> String {
    value.to_string()
}
fn level(value: Option<Severity>) -> &'static str {
    match value {
        Some(Severity::Error) => "error",
        Some(Severity::Warning) => "warning",
        Some(Severity::Information) => "info",
        None => "never",
    }
}
fn clock(t: AbsoluteTime, basis: Option<&str>, years: usize) -> Value {
    json!({"year": t.year(), "day": t.day_of_year(), "hour": t.hours(),
        "minute": t.minutes(), "second": t.seconds(), "nanosecond": t.nanoseconds(),
        "clock_count_ns": decimal(t.as_total_ns()), "scale_basis": basis.unwrap_or("scale unknown"), "year_offset": decimal(years)})
}
fn channel(info: &TimeChannelInfo) -> Value {
    let (scale, basis) = report::native_scale(info);
    json!({"id": info.channel_id, "packets": decimal(info.packet_count),
        "format": report::format_label(info), "native_scale": scale, "scale_basis": basis,
        "format2_word": info.network_word.map(TimeF2Csdw::as_raw),
        "status": info.network_word.map(|w| format!("{:?}", w.time_status())),
        "format1_low_nibble": info.source.map(|s| match s {
            TimeSource::Internal => 0, TimeSource::External => 1,
            TimeSource::InternalFromRmm => 2, TimeSource::None => 15,
            TimeSource::Reserved(n) => n,
        }),
        "segments": decimal(info.segment),
        "first": info.first_time.map(|t| clock(t, info.first_basis, 0)), "last": info.last_time.map(|t| clock(t, info.last_basis, info.year_rollovers)),
        "first_rtc": info.first_rtc.map(|r| decimal(r.as_raw())),
        "last_rtc": info.last_rtc.map(|r| decimal(r.as_raw())),
        "min_gap_ns": info.min_gap_ns.map(decimal), "max_gap_ns": info.max_gap_ns.map(decimal)})
}
fn document(r: &Report<'_>, args: &Args, size: u64) -> Value {
    let path = args.input.as_os_str();
    let bytes: String = path
        .as_encoded_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    // BTreeMap iteration and serde_json's default sorted object map fix ordering.
    json!({"schema": "irigtime-report", "schema_version": 1, "schema_status": "draft",
        "tool": {"name": "irigtime", "version": PKG_VERSION},
        "input": {"name": path.to_string_lossy(), "name_lossy": path.to_str().is_none(),
            "native_name_hex": bytes, "native_name_encoding": "OsStr::as_encoded_bytes (platform-dependent)",
            "size_bytes": decimal(size)},
        "options": {"command": args.command, "format": "json", "fail_on": level(args.fail_on),
            "threshold_ms": if args.command != "channels" { Some(decimal(args.threshold_ms)) } else { None }},
        "units": {"rtc": "100 ns ticks", "durations": "ns", "file_size": "bytes", "counts": "items",
            "clock_count": "ns since day 1 00:00:00; year unspecified"},
        "report": {"qualification": "observed inventory; qualified joining/basis pending",
            "total_packets": decimal(r.total_packets), "time_packets": decimal(r.time_packets),
            "references_seen": decimal(r.references_seen), "reference_count": decimal(r.reference_count),
            "references_evicted": decimal(r.references_evicted), "reset_count": decimal(r.reset_count),
            "first_rtc": r.first_rtc.map(|v| decimal(v.as_raw())), "last_rtc": r.last_rtc.map(|v| decimal(v.as_raw())),
            "channels": r.time_channels.values().map(channel).collect::<Vec<_>>(),
            "data_types": r.data_type_counts.iter().map(|(id, count)| json!({"id": id, "packets": decimal(count)})).collect::<Vec<_>>(),
            "jumps": {"count": decimal(r.jumps_seen), "omitted": decimal(r.jumps_seen - r.jumps.len()),
                "samples": r.jumps.iter().enumerate().map(|(index, j)| json!({"channel": j.channel_id, "index": decimal(j.index),
                    "expected_ns": decimal(j.expected_nanos), "actual_ns": decimal(j.actual_nanos),
                    "delta_ns": decimal(j.delta_nanos),
                    "expected_year_days": nonzero(r.jump_context(index).0), "actual_year_days": nonzero(r.jump_context(index).1)})).collect::<Vec<_>>()},
            "findings": {"error": decimal(r.finding_counts[2]), "warning": decimal(r.finding_counts[1]),
                "info": decimal(r.finding_counts[0]), "omitted": decimal(r.omitted_findings()), "samples": r.errors}}})
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test fixtures only.
mod tests {
    use super::*;
    #[test]
    fn exact_wide_values_and_order_survive_json_projection() {
        let mut s = Ch10TimeScanner::new();
        s.total_packets = usize::MAX;
        s.finding_counts = [0, (1u64 << 53) + 1, 0];
        s.jumps_seen = 1;
        s.jump_year_days.push((366, 366));
        s.jumps.push(TimeJump {
            index: usize::MAX,
            channel_id: 7,
            expected_nanos: u64::MAX,
            actual_nanos: 0,
            delta_nanos: i64::MIN,
        });
        let mut info = TimeChannelInfo::new(7);
        info.first_time = Some(AbsoluteTime::new(100, 12, 0, 0, 123).unwrap());
        s.time_channels.insert(7, info);
        s.time_channels.insert(2, TimeChannelInfo::new(2));
        let a = match args::parse(&["jumps", "input", "--format", "json", "--fail-on", "never"])
            .unwrap()
        {
            args::Request::File(a) => Some(a),
            _ => None,
        }
        .unwrap();
        let v = document(&Report::from(&s), &a, u64::MAX);
        let schema: Value = serde_json::from_str(include_str!(
            "../../docs/schema/irigtime-report-v1.schema.json"
        ))
        .unwrap();
        assert_eq!(v["units"], schema["properties"]["units"]["const"]);
        assert_eq!(
            v["schema_version"],
            schema["properties"]["schema_version"]["const"]
        );
        assert_eq!(v["input"]["size_bytes"], "18446744073709551615");
        assert_eq!(v["report"]["total_packets"], usize::MAX.to_string());
        assert_eq!(v["report"]["findings"]["warning"], "9007199254740993");
        assert_eq!(v["report"]["channels"][0]["id"], 2);
        assert_eq!(v["report"]["channels"][1]["id"], 7);
        let sample = &v["report"]["jumps"]["samples"][0];
        assert_eq!(sample["index"], usize::MAX.to_string());
        assert_eq!(sample["expected_ns"], "18446744073709551615");
        assert_eq!(sample["actual_ns"], "0");
        assert_eq!(sample["delta_ns"], "-9223372036854775808");
        assert_eq!(sample["expected_year_days"], 366);
        assert_eq!(sample["actual_year_days"], 366);
        assert_eq!(
            decimal(i128::MIN),
            "-170141183460469231731687303715884105728"
        );
        assert_eq!(
            decimal(u128::MAX),
            "340282366920938463463374607431768211455"
        );
        let info = s.time_channels.get_mut(&7).unwrap();
        info.format = Some(TimeFormat::IrigA);
        info.first_basis = Some("UTC (assumed, RDG-009)");
        info.last_time = Some(AbsoluteTime::new(1, 0, 0, 0, 0).unwrap());
        info.last_basis = info.first_basis;
        info.year_rollovers = 1;
        let projected = document(&Report::from(&s), &a, 24);
        let row = &projected["report"]["channels"][1];
        assert_eq!(row["native_scale"], "UTC");
        assert_eq!(row["scale_basis"], "assumed, RDG-009");
        assert_eq!(row["first"]["scale_basis"], "UTC (assumed, RDG-009)");
        assert_eq!(row["first"]["year_offset"], "0");
        assert_eq!(row["last"]["year_offset"], "1");
    }
}
