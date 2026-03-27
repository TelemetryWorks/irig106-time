# L1 Requirements — irig106-time

**Document:** L1_REQUIREMENTS.md
**Crate:** irig106-time
**Version:** 0.1.0
**Standard Baseline:** IRIG 106-17 (Chapter 10), RCC 123-20 (Programmer's Handbook)
**Date:** 2026-03-25

---

## 1. Purpose

This document defines Level 1 (L1) requirements for the `irig106-time` crate. L1 requirements
are directly traceable to the IRIG 106 standard and establish the top-level capabilities the
crate shall provide. Each L1 requirement maps to one or more sections of the governing standard.

---

## 2. Conventions

| Prefix  | Domain                                      |
|---------|---------------------------------------------|
| L1-RTC  | 48-bit Relative Time Counter                |
| L1-ABS  | Absolute time representations               |
| L1-CSDW | Time Data Format 1 Channel-Specific Data Word |
| L1-BCD  | BCD-encoded time messages (Day / DMY)       |
| L1-SEC  | Secondary header time formats               |
| L1-IPT  | Intra-packet time stamps                    |
| L1-COR  | RTC-to-absolute-time correlation            |
| L1-ERR  | Error handling and validation               |
| L1-API  | Public API and type safety                  |

---

## 3. L1 Requirements

### 3.1 Relative Time Counter (RTC)

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-RTC-001 | The crate shall represent the 48-bit Relative Time Counter (RTC) as a distinct type with a resolution of 100 nanoseconds per least-significant bit (10 MHz clock). | Ch10 §10.6.1.1; RCC 123-20 §5.3; §6.6 |
| L1-RTC-002 | The crate shall support extraction of a 48-bit RTC value from a 6-byte little-endian buffer (bytes [16..22] of the primary packet header). | Ch10 §10.6.1.1 Table 10-3; RCC 123-20 §5.3 |
| L1-RTC-003 | The crate shall support arithmetic on RTC values including computing elapsed time between two RTC values in nanoseconds. | RCC 123-20 §6.6 |
| L1-RTC-004 | The crate shall convert an RTC tick count to a duration with nanosecond precision. | Ch10 §10.6.1.1 (10 MHz clock definition) |

### 3.2 Absolute Time Representations

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-ABS-001 | The crate shall represent absolute time with at least nanosecond precision to support all Chapter 10 time formats. | Ch10 §10.6.1.5 (IEEE-1588); §10.6.1.4 (Ch4 BWT) |
| L1-ABS-002 | The crate shall support the IRIG 106 Chapter 4 Binary Weighted Time (BWT) format consisting of a high-order time word, low-order time word, and microsecond time word. | Ch10 §10.6.1.4; RCC 123-20 §5.4 Figure 5-3 |
| L1-ABS-003 | The crate shall support the IEEE-1588 time format consisting of a 32-bit nanoseconds field and a 32-bit seconds field. | Ch10 §10.6.1.5; RCC 123-20 §5.4 Figure 5-4 |
| L1-ABS-004 | The crate shall support the Extended Relative Time Counter (ERTC) 64-bit format. | Ch10 §10.6.1.6; RCC 123-20 §5.4 Figure 5-5 |

### 3.3 Time Data Format 1 CSDW (Data Type 0x11)

| ID        | Requirement | Standard Reference |
|-----------|-------------|-------------------|
| L1-CSDW-001 | The crate shall parse the 32-bit Channel-Specific Data Word (CSDW) for Time Data Format 1 (Data Type 0x11) packets. | Ch10 §10.6.5.2; RCC 123-20 §5.5.3 Figure 5-12 |
| L1-CSDW-002 | The crate shall decode the 4-bit Time Source field (bits [3:0]) from the CSDW. | Ch10 §10.6.5.2 Table 10-11 |
| L1-CSDW-003 | The crate shall decode the 4-bit Time Format field (bits [7:4]) from the CSDW. | Ch10 §10.6.5.2 Table 10-12 |
| L1-CSDW-004 | The crate shall decode the 1-bit Leap Year indicator (bit 8) from the CSDW. | Ch10 §10.6.5.2 |
| L1-CSDW-005 | The crate shall decode the 1-bit Date Format indicator (bit 9): 0 = Day-of-Year, 1 = Day-Month-Year. | Ch10 §10.6.5.2 |

### 3.4 BCD Time Message Decoding

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-BCD-001 | The crate shall decode the 8-byte Day-of-Year (DOY) format BCD time message into milliseconds, seconds, minutes, hours, and day-of-year. | Ch10 §10.6.5.2; RCC 123-20 §5.5.3 Figure 5-13 |
| L1-BCD-002 | The crate shall decode the 10-byte Day-Month-Year (DMY) format BCD time message into milliseconds, seconds, minutes, hours, day, month, and year. | Ch10 §10.6.5.2; RCC 123-20 §5.5.3 Figure 5-14 |
| L1-BCD-003 | The crate shall validate that reserved bits in BCD time messages are zero. | Ch10 §10.6.5.2 (reserved fields "shall be zero") |
| L1-BCD-004 | The crate shall validate that each BCD digit is in the range 0–9. | Implied by BCD encoding (digits > 9 are undefined) |

### 3.5 Secondary Header Time

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-SEC-001 | The crate shall parse the 12-byte optional secondary header for Chapter 4 BWT time representation (Packet Flag bits [3:2] = 0b00). | Ch10 §10.6.1.4; RCC 123-20 §5.4 Figure 5-3 |
| L1-SEC-002 | The crate shall parse the 12-byte optional secondary header for IEEE-1588 time representation (Packet Flag bits [3:2] = 0b01). | Ch10 §10.6.1.5; RCC 123-20 §5.4 Figure 5-4 |
| L1-SEC-003 | The crate shall parse the 12-byte optional secondary header for ERTC time representation (Packet Flag bits [3:2] = 0b10). | Ch10 §10.6.1.6; RCC 123-20 §5.4 Figure 5-5 |
| L1-SEC-004 | The crate shall validate the secondary header checksum. | Ch10 §10.6.1.4 |

### 3.6 Intra-Packet Time Stamps

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-IPT-001 | The crate shall parse the 8-byte intra-packet time stamp in 48-bit RTC format (Packet Flag bit 2 = 0). | Ch10 §10.6.2; RCC 123-20 §5.4 Figure 5-6 |
| L1-IPT-002 | The crate shall parse the 8-byte intra-packet time stamp in IRIG 106 Chapter 4 Binary format (Packet Flag bits [3:2] = 0b00, secondary header present). | Ch10 §10.6.2; RCC 123-20 §5.4 Figure 5-7 |
| L1-IPT-003 | The crate shall parse the 8-byte intra-packet time stamp in IEEE-1588 format (Packet Flag bits [3:2] = 0b01). | Ch10 §10.6.2; RCC 123-20 §5.4 Figure 5-8 |
| L1-IPT-004 | The crate shall parse the 8-byte intra-packet time stamp in 64-bit ERTC format (Packet Flag bits [3:2] = 0b10). | Ch10 §10.6.2; RCC 123-20 §5.4 Figure 5-9 |

### 3.7 RTC-to-Absolute-Time Correlation

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-COR-001 | The crate shall correlate RTC values to absolute time using Time Data Format 1 packets that pair RTC with absolute clock time. | RCC 123-20 §6.6 |
| L1-COR-002 | The crate shall use the time packet nearest in RTC to the target data packet to minimize drift error. | RCC 123-20 §6.6 ("It is better to use the clock and relative time values from a time packet that occurs near the current data packet") |
| L1-COR-003 | The crate shall support multiple time sources (e.g., IRIG-B, GPS, internal clock) and allow the caller to select which source to use. | RCC 123-20 §6.6 ("there may be separate time channels for time derived from IRIG B, GPS, and an internal battery backed up clock") |
| L1-COR-004 | The crate shall detect and report jumps in the input clock time (e.g., GPS lock acquisition). | RCC 123-20 §6.6 ("there is a jump in input clock time during a recording, such as when GPS locks for the first time") |

### 3.8 Error Handling and Validation

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-ERR-001 | The crate shall return typed errors for all fallible operations; it shall not panic on invalid input data. | Crate design principle (aerospace-grade robustness) |
| L1-ERR-002 | The crate shall detect and report invalid BCD digits (values > 9). | Implied by BCD encoding |
| L1-ERR-003 | The crate shall detect and report out-of-range time field values (e.g., hours > 23, minutes > 59, seconds > 59, day > 366). | Domain validation |
| L1-ERR-004 | The crate shall detect and report secondary header checksum failures. | Ch10 §10.6.1.4 |

### 3.9 Public API and Type Safety

| ID       | Requirement | Standard Reference |
|----------|-------------|-------------------|
| L1-API-001 | All public types shall implement `Debug`, `Clone`, `Copy`, `PartialEq`, and `Eq` where semantically appropriate. | Crate design principle |
| L1-API-002 | The crate shall be `#![no_std]` compatible when the `std` feature is disabled. | Crate design principle (embedded/WASM targets) |
| L1-API-003 | The crate shall have zero required runtime dependencies beyond `core` and `alloc`. | Crate design principle (minimal dependency footprint) |
| L1-API-004 | All public types shall use newtypes or enums to prevent misuse (e.g., RTC tick counts shall not be confused with nanoseconds). | Crate design principle (type-driven correctness) |

---

## 4. Traceability Summary

| L1 Prefix | Count | Primary Standard Sections |
|-----------|-------|--------------------------|
| L1-RTC    | 4     | Ch10 §10.6.1.1; RCC 123-20 §5.3, §6.6 |
| L1-ABS    | 4     | Ch10 §10.6.1.4–10.6.1.6; RCC 123-20 §5.4 |
| L1-CSDW   | 5     | Ch10 §10.6.5.2; RCC 123-20 §5.5.3 |
| L1-BCD    | 4     | Ch10 §10.6.5.2; RCC 123-20 §5.5.3 |
| L1-SEC    | 4     | Ch10 §10.6.1.4–10.6.1.6; RCC 123-20 §5.4 |
| L1-IPT    | 4     | Ch10 §10.6.2; RCC 123-20 §5.4 |
| L1-COR    | 4     | RCC 123-20 §6.6 |
| L1-ERR    | 4     | Domain validation / Ch10 §10.6.1.4 |
| L1-API    | 4     | Crate design principles |
| **Total** | **37** | |
