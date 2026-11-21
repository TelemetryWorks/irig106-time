# Package-local Format 2 guide snippets

These snippets mirror docs/usage.md section 13. The repository workspace guard
checks synchronization; the Rust test also checks the compiled example.

```rust
use irig106_time::{
    Result,
    format2::NetworkConversion,
    network_time::{Format2Reading, LeapSecondTable, NetworkTime, decode_time_f2},
    ntp::{NtpContext, convert_ntp},
};

// Payload excludes headers; Data Length excludes trailer filler/checksum.
// Caller supplies this session's NTP evidence, never the host clock.
fn process_time_f2_packet(
    payload: &[u8],
    data_length: usize,
    ntp_context: &NtpContext,
    table: &LeapSecondTable,
) -> Result<(Format2Reading, Option<NetworkConversion>)> {
    let reading = decode_time_f2(payload, data_length)?;
    let conversion = if reading.is_usable() {
        match reading.time.map(|t| t.value) {
            Some(NetworkTime::Ntp(raw)) => Some(NetworkConversion::Ntp(convert_ntp(
                raw,
                ntp_context,
                Some(table),
            ))),
            Some(NetworkTime::Ptp(raw)) => Some(NetworkConversion::Ptp(table.convert_ptp(raw)?)),
            None => None,
        }
    } else {
        None // Invalid TS/reserved NTF is not promoted to absolute time.
    };
    Ok((reading, conversion)) // Retain raw/status and full conversion basis.
}
```

```rust
fn main() -> irig106_time::Result<()> {
    use irig106_time::{
        NtpTime, PtpTime,
        leap::PtpConversionStatus,
        ntp::{EraBasis, EraDeclaration},
    };
    let table = LeapSecondTable::builtin();
    let mut context = NtpContext::new(1); // Actual caller session.
    context.declared = Some(EraDeclaration {
        era: 0,
        id: "caller era".into(),
    });

    // Ch11 Fig11-16 / RDG-012: UTC follows NTP reckoning, not TAI offsets.
    let raw_ntp = NtpTime {
        seconds: 3_944_678_400,
        fraction: 1 << 31,
    };
    let ntp = convert_ntp(raw_ntp, &context, Some(&table));
    assert_eq!(ntp.unix_seconds().map(|s| s.get()), Some(1_735_689_600));
    assert_eq!(ntp.utc().map(|t| t.nanoseconds), Some(500_000_000));
    assert_eq!(ntp.raw(), raw_ntp);
    assert_eq!(ntp.basis().era_basis, Some(EraBasis::Declared));

    // Fig11-17: native TAI; table conversion retains leap/currency evidence.
    let raw_ptp = PtpTime {
        seconds: 1_735_689_637,
        nanoseconds: 0,
    };
    let ptp = table.convert_ptp(raw_ptp)?;
    assert_eq!(ptp.unix_seconds().map(|s| s.get()), Some(1_735_689_600));
    assert_eq!(ptp.status(), PtpConversionStatus::Covered);
    assert_eq!(ptp.raw(), raw_ptp); // Keep this and the basis, not just UTC.

    let mut payload = [0u8; 12];
    payload[0] = 1; // NTF0, TS1; seconds/fraction are little-endian words.
    payload[4..12].copy_from_slice(&raw_ntp.to_le_bytes());
    let (reading, conversion) = process_time_f2_packet(&payload, 12, &context, &table)?;
    assert!(reading.is_usable());
    assert_eq!(conversion, Some(NetworkConversion::Ntp(ntp)));
    payload[0] = 0; // Invalid TS: raw body remains, conversion withheld.
    let (reading, conversion) = process_time_f2_packet(&payload, 12, &context, &table)?;
    assert!(!reading.is_usable());
    assert!(reading.time.is_some());
    assert!(conversion.is_none());
    Ok(())
}
```
