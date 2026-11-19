//! Hand-rolled OS-native arguments (ADR-0008); parsing never opens input.
use irig106_time::Rtc;
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
        if raw.len() != 1 {
            return Err("unexpected arguments after help/version".into());
        }
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
    if raw.len() == 2 && matches!(raw[1].as_ref().to_str(), Some("--help" | "-h")) {
        return Ok(Request::Help);
    }
    let mut parsed = Args {
        command: command.into(),
        input: PathBuf::new(),
        output: None,
        rtc: Rtc::from_raw(0),
        threshold_ms: 1000,
        limit: 100,
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
            "--output" => parsed.output = Some(PathBuf::from(value)),
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
