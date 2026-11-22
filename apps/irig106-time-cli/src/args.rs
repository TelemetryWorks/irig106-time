//! Hand-rolled OS-native arguments (ADR-0008); parsing never opens input.
use irig106_time::{Rtc, findings::Severity};
use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    path::PathBuf,
};

/// Validated request; the host removes its executable and `time` subcommand.
#[derive(Debug)]
pub enum Request {
    /// Global or command help.
    Help,
    /// Lockstep package version.
    Version,
    /// Validated file command.
    File(Args),
}
/// Parsed command settings. Only the parser constructs this value.
#[derive(Debug)]
pub struct Args {
    pub(crate) command: String,
    pub(crate) input: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) rtc: Rtc,
    pub(crate) threshold_ms: u64,
    pub(crate) limit: usize,
    pub(crate) fail_on: Option<Severity>,
    pub(crate) json: bool,
    pub(crate) ntp_era: Option<i64>,
}
fn decimal(value: &OsStr) -> Result<u64, String> {
    let text = value.to_str().ok_or("numeric argument is not UTF-8")?;
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("expected unsigned decimal integer".into());
    }
    text.parse().map_err(|_| "integer out of range".into())
}
/// Parse all syntax before any file or output operation. Separate option values
/// only; `--` makes all remaining arguments literal positional paths/RTC.
/// Traces: L3-CLI-103 <- L2-CLI-103 <- L1-CLI-003.
pub fn parse<S: AsRef<OsStr>>(raw: &[S]) -> Result<Request, String> {
    let Some(first) = raw.first() else {
        return Ok(Request::Help);
    };
    let command = first.as_ref().to_str().ok_or("command is not UTF-8")?;
    if matches!(command, "--help" | "-h" | "help" | "--version" | "-V") {
        validate_no_finding_options(&raw[1..])?;
        return Ok(if matches!(command, "--version" | "-V") {
            Request::Version
        } else {
            Request::Help
        });
    }
    if !matches!(
        command,
        "summary" | "channels" | "jumps" | "timeline" | "csv" | "correlate"
    ) {
        return Err(format!("unknown command: {command}"));
    }
    if raw
        .get(1)
        .is_some_and(|a| matches!(a.as_ref().to_str(), Some("--help" | "-h")))
    {
        validate_no_finding_options(&raw[2..])?;
        return Ok(Request::Help);
    }
    let mut parsed = Args {
        command: command.into(),
        json: false,
        ntp_era: None,
        input: PathBuf::new(),
        output: None,
        rtc: Rtc::from_raw(0),
        threshold_ms: 1000,
        limit: 100,
        // L2/L3-CLI-107, coordinator S16a3b shared irig106 convention.
        fail_on: Some(Severity::Error),
    };
    let (mut index, mut literal) = (1, false);
    let mut seen = BTreeSet::new();
    let mut positional: Vec<OsString> = Vec::new();
    while let Some(argument) = raw.get(index) {
        index += 1;
        let argument = argument.as_ref();
        if !literal && argument == "--" {
            literal = true;
            continue;
        }
        if literal || !argument.as_encoded_bytes().starts_with(b"-") {
            positional.push(argument.to_owned());
            continue;
        }
        let flag = argument.to_str().ok_or("option name is not UTF-8")?;
        let allowed = match flag {
            "--fail-on" | "--ntp-era" => true,
            "--format" => matches!(
                command,
                "summary" | "channels" | "jumps" | "timeline" | "correlate"
            ),
            "--threshold-ms" => command == "jumps",
            "--limit" => command == "timeline",
            "--output" => command == "csv",
            _ => false,
        };
        if !allowed {
            return Err(format!("unknown or inapplicable option: {flag}"));
        }
        if !seen.insert(flag) {
            return Err(format!("duplicate option: {flag}"));
        }
        let value = raw
            .get(index)
            .ok_or_else(|| format!("missing value for {flag}"))?
            .as_ref();
        index += 1;
        match flag {
            "--threshold-ms" => {
                let n = decimal(value)?;
                n.checked_mul(1_000_000)
                    .ok_or("threshold overflows nanoseconds")?;
                parsed.threshold_ms = n;
            }
            "--limit" => {
                let n = usize::try_from(decimal(value)?).map_err(|_| "limit out of range")?;
                if n == 0 {
                    return Err("limit must be positive".into());
                }
                parsed.limit = n;
            }
            "--format" => {
                parsed.json = match value.to_str() {
                    Some("text") => false,
                    Some("json") => true,
                    _ => return Err("format must be text or json".into()),
                }
            }
            "--ntp-era" => {
                let text = value.to_str().ok_or("NTP era is not UTF-8")?;
                let digits = text.strip_prefix('-').unwrap_or(text);
                if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    return Err("NTP era must be a signed decimal integer".into());
                }
                parsed.ntp_era = Some(text.parse().map_err(|_| "NTP era out of i64 range")?);
            }
            "--output" => parsed.output = Some(PathBuf::from(value)),
            "--fail-on" => parsed.fail_on = threshold(value)?,
            _ => return Err("unrecognized option".into()),
        }
    }
    let expected = if command == "correlate" { 2 } else { 1 };
    if positional.len() != expected {
        return Err(format!(
            "{command} requires {expected} positional argument(s)"
        ));
    }
    parsed.input = PathBuf::from(&positional[0]);
    if command == "correlate" {
        let text = positional[1].to_str().ok_or("RTC is not UTF-8")?;
        let digits = text
            .strip_prefix("0x")
            .or_else(|| text.strip_prefix("0X"))
            .unwrap_or(text);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid hex RTC".into());
        }
        let value = u64::from_str_radix(digits, 16).map_err(|_| "invalid hex RTC")?;
        if value >= 1u64 << 48 {
            return Err("RTC exceeds 48 bits".into());
        }
        parsed.rtc = Rtc::from_raw(value);
    }
    Ok(Request::File(parsed))
}

fn threshold(value: &OsStr) -> Result<Option<Severity>, String> {
    match value.to_str() {
        Some("error") => Ok(Some(Severity::Error)),
        Some("warning") => Ok(Some(Severity::Warning)),
        Some("info") => Ok(Some(Severity::Information)),
        Some("never") => Ok(None),
        _ => Err("fail-on must be error, warning, info or never".into()),
    }
}
// Help/version do not produce findings, but still validate the shared option.
fn validate_no_finding_options<S: AsRef<OsStr>>(raw: &[S]) -> Result<(), String> {
    let mut tail = raw.iter();
    let mut seen = false;
    while let Some(option) = tail.next() {
        if option.as_ref() != "--fail-on" || seen {
            return Err("unexpected/duplicate argument after help/version".into());
        }
        threshold(tail.next().ok_or("missing fail-on value")?.as_ref())?;
        seen = true;
    }
    Ok(())
}
