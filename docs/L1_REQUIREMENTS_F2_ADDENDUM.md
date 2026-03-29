# L1 Requirements Addendum — Time Data Format 2 (Network Time)

**Document:** L1_REQUIREMENTS_F2_ADDENDUM.md
**Crate:** irig106-time
**Target Version:** 0.2.0
**Standard Baseline:** IRIG 106-17 Chapter 11 §11.2.3.3; IRIG 106-23 Chapter 11 §11.2.3.3
**Date:** 2026-03-27

---

## 1. Purpose

This addendum defines L1 requirements for Time Data Format 2 (Data Type 0x12,
Network Time), introduced in IRIG 106-17. These requirements supplement the
existing L1_REQUIREMENTS.md and follow the same conventions.

---

## 2. Background

Format 2 Network Time packets provide an alternative to Format 1 (BCD-encoded
IRIG/GPS/RTC time). Instead of analog-derived time codes, Format 2 carries
network-protocol time from NTP or PTP sources. This is increasingly common as
recorders integrate with Ethernet-based instrumentation networks.

### Key Differences from Format 1

| Aspect | Format 1 (0x11) | Format 2 (0x12) |
|--------|----------------|----------------|
| Time encoding | BCD (4-bit digits) | Binary (seconds + fractional) |
| Protocols | IRIG-B/A/G, GPS, RTC | NTP (RFC 5905), PTP (IEEE 1588) |
| Epoch (NTP) | N/A | January 1, 1900 00:00:00 UTC |
| Epoch (PTP) | N/A | January 1, 1970 00:00:00 TAI |
| Leap seconds | Not applicable | PTP uses TAI (no leap seconds); NTP uses UTC |
| Resolution | 10 ms (BCD) | NTP: ~233 ps (2⁻³²s); PTP: 1 ns |
| First appeared | IRIG 106-04 | IRIG 106-17 (CR93) |

---

## 3. L1 Requirements

### 3.1 Format 2 CSDW (Data Type 0x12)

| ID | Requirement | Standard Reference |
|----|-------------|-------------------|
| L1-F2CSDW-001 | The crate shall parse the 32-bit Channel-Specific Data Word for Time Data Format 2 (Data Type 0x12) packets. | Ch11 §11.2.3.3; Figure 11-15 (106-23) |
| L1-F2CSDW-002 | The crate shall decode the 4-bit Time Protocol field (bits [3:0]) from the F2 CSDW: 0 = NTP, 1 = PTP. | Ch11 §11.2.3.3 |
| L1-F2CSDW-003 | The crate shall validate that reserved bits [31:4] are zero. | Ch11 §11.2.1.1f ("All reserved bit fields... shall be set to zero") |

### 3.2 NTP Time Message

| ID | Requirement | Standard Reference |
|----|-------------|-------------------|
| L1-NTP-001 | The crate shall decode the 8-byte NTP time message consisting of 32-bit seconds and 32-bit fractional seconds, both little-endian. | Ch11 §11.2.3.3; Figure 11-16 (106-23) |
| L1-NTP-002 | The crate shall interpret NTP seconds as elapsed since the NTP epoch: January 1, 1900 00:00:00 UTC (RFC 5905). | Ch11 §11.2.3.3 ("NTP is referenced in UTC time with an epoch of January 1, 1900") |
| L1-NTP-003 | The crate shall convert NTP fractional seconds to nanoseconds with at least nanosecond precision. | Derived: 2⁻³² s ≈ 233 ps; ns is sufficient for correlation |
| L1-NTP-004 | The crate shall convert NTP time to a Unix-compatible representation for interop with common time libraries. | Derived: consumer convenience |

### 3.3 PTP Time Message

| ID | Requirement | Standard Reference |
|----|-------------|-------------------|
| L1-PTP-001 | The crate shall decode the 10-byte PTP time message consisting of 48-bit seconds and 32-bit nanoseconds, both little-endian. | Ch11 §11.2.3.3; Figure 11-17 (106-23) |
| L1-PTP-002 | The crate shall interpret PTP seconds as elapsed since the PTP epoch: January 1, 1970 00:00:00 TAI (International Atomic Time). | Ch11 §11.2.3.3 ("PTP is referenced in International Atomic Time with an epoch of January 1, 1970") |
| L1-PTP-003 | The crate shall validate that nanoseconds < 1,000,000,000. | Domain validation |
| L1-PTP-004 | The crate shall provide a mechanism to convert PTP/TAI time to UTC by applying a configurable leap-second offset. | Ch11 §11.2.3.3 ("The PTP time does not include leap seconds"); derived |

### 3.4 Correlation Integration

| ID | Requirement | Standard Reference |
|----|-------------|-------------------|
| L1-F2COR-001 | The crate shall support Time Data Format 2 packets as reference points for RTC-to-absolute-time correlation, alongside Format 1 packets. | RCC 123-20 §6.6 (correlation applies to all time packets) |
| L1-F2COR-002 | The correlator shall distinguish between NTP-sourced and PTP-sourced reference points to avoid mixing epoch/timescale assumptions. | Derived: NTP uses UTC, PTP uses TAI — mixing produces errors |

### 3.5 TAI-UTC Offset Handling

| ID | Requirement | Standard Reference |
|----|-------------|-------------------|
| L1-TAI-001 | The crate shall provide a `LeapSecondTable` type that maps date ranges to TAI-UTC offsets. | Derived from L1-PTP-004 |
| L1-TAI-002 | The crate shall ship with a compiled-in leap second table current as of the crate release date. | Derived: usability |
| L1-TAI-003 | The crate shall allow callers to supply an updated leap second table at runtime. | Derived: forward compatibility |

---

## 4. Traceability Summary

| L1 Prefix | Count | Primary Standard Sections |
|-----------|-------|--------------------------|
| L1-F2CSDW | 3 | Ch11 §11.2.3.3 |
| L1-NTP | 4 | Ch11 §11.2.3.3; RFC 5905 |
| L1-PTP | 4 | Ch11 §11.2.3.3; IEEE 1588-2019 |
| L1-F2COR | 2 | RCC 123-20 §6.6 |
| L1-TAI | 3 | Derived from PTP TAI epoch |
| **Total** | **16** | |

---

## 5. Epoch Reference Table

| Protocol | Epoch | Timescale | Leap Seconds | Seconds at Unix Epoch (1970-01-01) |
|----------|-------|-----------|-------------|-----------------------------------|
| NTP | 1900-01-01 00:00:00 | UTC | Included in UTC | 2,208,988,800 |
| PTP | 1970-01-01 00:00:00 | TAI | Not included | 0 (but TAI = UTC + offset) |
| Unix | 1970-01-01 00:00:00 | UTC | Not counted | 0 |

### Converting Between Epochs

```
NTP_to_Unix:  unix_seconds = ntp_seconds - 2_208_988_800
PTP_to_UTC:   utc_seconds  = ptp_tai_seconds - leap_second_offset(date)
Unix_to_NTP:  ntp_seconds  = unix_seconds + 2_208_988_800
```

As of 2026, the TAI-UTC offset is 37 seconds (since January 1, 2017).

---

## 6. Wire Format Summary

### Format 2 CSDW (4 bytes)

```
Bits [3:0]   = Time Protocol:  0 = NTP, 1 = PTP
Bits [31:4]  = Reserved (shall be zero)
```

### NTP Time Data (8 bytes)

```
Bytes [0..4] = Seconds since 1900-01-01 00:00:00 UTC (u32 LE)
Bytes [4..8] = Fractional seconds (u32 LE, units of 2⁻³² seconds)
```

### PTP Time Data (10 bytes)

```
Bytes [0..6] = Seconds since 1970-01-01 00:00:00 TAI (u48 LE)
Bytes [6..10] = Nanoseconds (u32 LE, 0–999,999,999)
```
