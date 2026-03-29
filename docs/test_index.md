# Test Documentation Index — irig106-time

**Document:** TEST_INDEX.md
**Crate:** irig106-time v0.5.0
**Total Tests:** 203 (136 unit, 50 integration, 17 property)
**Date:** 2026-03-29

---

## 1. Test Organization

Unit tests live in `src/<module>_tests.rs` files alongside the source, per project convention.
Integration tests live in `tests/` and exercise the full parsing → correlation pipeline.

```
src/
  error.rs            ← source
  error_tests.rs      ← 7 unit tests
  rtc.rs
  rtc_tests.rs        ← 18 unit tests
  absolute.rs
  absolute_tests.rs   ← 23 unit tests
  csdw.rs
  csdw_tests.rs       ← 14 unit tests
  bcd.rs
  bcd_tests.rs        ← 13 unit tests
  secondary.rs
  secondary_tests.rs  ← 10 unit tests
  intra_packet.rs
  intra_packet_tests.rs ← 8 unit tests
  correlation.rs
  correlation_tests.rs  ← 11 unit tests
  network_time.rs
  network_time_tests.rs ← 22 unit tests
  version.rs
  version_tests.rs      ← 10 unit tests
tests/
  pipeline.rs         ← 50 integration tests
  properties.rs       ← 17 property-based tests
```

---

## 2. Unit Test Summary by Module

### error (7 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `display_invalid_bcd_digit` | Display output for InvalidBcdDigit | L3-ERR-002 |
| `display_reserved_bit_set` | Display output for ReservedBitSet | L3-ERR-002 |
| `display_out_of_range` | Display output for OutOfRange | L3-ERR-002 |
| `display_checksum_mismatch` | Display output for ChecksumMismatch | L3-ERR-002 |
| `display_no_reference_point` | Display output for NoReferencePoint | L3-ERR-002 |
| `display_buffer_too_short` | Display output for BufferTooShort | L3-ERR-002 |
| `error_is_clone_eq` | Derive traits (Clone, Eq) | L3-ERR-001 |

### rtc (18 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `zero_constant` | `Rtc::ZERO` is 0 | L3-RTC-004 |
| `max_constant` | `Rtc::MAX` is 2^48−1 | L3-RTC-005 |
| `from_le_bytes_all_zeros` | Zero bytes → zero RTC | L3-RTC-006 |
| `from_le_bytes_known_value` | Known byte pattern | L3-RTC-006 |
| `from_le_bytes_max` | All 0xFF → MAX | L3-RTC-006 |
| `from_raw_masks_upper_bits` | Upper bits cleared | L3-RTC-007 |
| `from_raw_preserves_lower` | Lower 48 bits kept | L3-RTC-007 |
| `as_raw_round_trip` | from_raw → as_raw | L3-RTC-008 |
| `elapsed_ticks_simple` | Forward elapsed | L3-RTC-009 |
| `elapsed_ticks_wrap_around` | 48-bit wrap | L3-RTC-009 |
| `elapsed_ticks_same_value` | Equal → 0 | L3-RTC-009 |
| `elapsed_nanos_simple` | ticks × 100 | L3-RTC-010 |
| `to_nanos_zero` | 0 → 0 ns | L3-RTC-011 |
| `to_nanos_one_tick` | 1 → 100 ns | L3-RTC-011 |
| `to_nanos_one_second` | 10M → 1s | L3-RTC-011 |
| `ordering` | Ord comparison | L3-RTC-012 |
| `debug_display` | Debug trait | L3-RTC-013 |
| `clone_copy_eq` | Copy + Clone + Eq | L3-RTC-013 |

### absolute (23 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `absolute_time_valid` | Valid construction | L3-ABS-002 |
| `absolute_time_day_zero_rejected` | day=0 rejected | L3-ABS-002 |
| `absolute_time_day_367_rejected` | day=367 rejected | L3-ABS-002 |
| `absolute_time_hours_24_rejected` | hours=24 rejected | L3-ABS-002 |
| `absolute_time_minutes_60_rejected` | min=60 rejected | L3-ABS-002 |
| `absolute_time_seconds_60_rejected` | sec=60 rejected | L3-ABS-002 |
| `absolute_time_nanos_overflow_rejected` | ns≥1B rejected | L3-ABS-002 |
| `with_date_valid` | Attach DMY | L3-ABS-003 |
| `with_date_month_zero_rejected` | month=0 rejected | L3-ABS-003 |
| `with_date_month_13_rejected` | month=13 rejected | L3-ABS-003 |
| `add_nanos_subsecond` | Sub-second add | L3-ABS-004 |
| `add_nanos_carry_to_seconds` | Carry ns→s | L3-ABS-004 |
| `add_nanos_carry_to_minutes` | Carry s→m | L3-ABS-004 |
| `add_nanos_carry_to_hours` | Carry m→h | L3-ABS-004 |
| `add_nanos_carry_to_days` | Carry h→day | L3-ABS-004 |
| `total_nanos_of_day_midnight` | 0 at midnight | L3-ABS-005 |
| `total_nanos_of_day_noon` | 12h in ns | L3-ABS-005 |
| `ieee1588_from_le_bytes` | Parse IEEE-1588 | L3-1588-002 |
| `ieee1588_nanos_overflow_rejected` | ns≥1B rejected | L3-1588-004 |
| `ieee1588_to_nanos_since_epoch` | Epoch ns | L3-1588-003 |
| `ertc_from_le_bytes` | Parse ERTC | L3-ERTC-002 |
| `ertc_to_nanos` | ns conversion | L3-ERTC-003 |
| `ertc_max_no_overflow` | u128 safety | L3-ERTC-003 |

### csdw (14 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `time_source_internal` | 0 → Internal | L3-CSDW-004 |
| `time_source_external` | 1 → External | L3-CSDW-004 |
| `time_source_gps` | 3 → Gps | L3-CSDW-004 |
| `time_source_none` | 0xF → None | L3-CSDW-004 |
| `time_source_reserved` | 7 → Reserved(7) | L3-CSDW-004 |
| `time_format_irig_b` | 0 → IrigB | L3-CSDW-005 |
| `time_format_gps` | 5 → Gps | L3-CSDW-005 |
| `time_format_reserved` | 9 → Reserved(9) | L3-CSDW-005 |
| `leap_year_set` | bit8=1 → true | L3-CSDW-006 |
| `leap_year_clear` | bit8=0 → false | L3-CSDW-006 |
| `date_format_doy` | bit9=0 → DOY | L3-CSDW-007 |
| `date_format_dmy` | bit9=1 → DMY | L3-CSDW-007 |
| `from_le_bytes_round_trip` | LE parse | L3-CSDW-003 |
| `combined_fields` | All fields | L3-CSDW-001..010 |

### bcd (13 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `day_fmt_decode_known` | Day100 12:30:25.340 | L3-BCD-003 |
| `day_fmt_midnight_day1` | Midnight DOY 001 | L3-BCD-003 |
| `day_fmt_max_day` | DOY 366 23:59:59.990 | L3-BCD-003 |
| `day_fmt_invalid_bcd_digit` | Nibble>9 rejected | L3-BCD-001 |
| `day_fmt_reserved_bit_set` | Reserved non-zero | L3-BCD-002 |
| `day_fmt_day_zero_rejected` | Day=0 OOR | L3-BCD-006 |
| `day_fmt_buffer_too_short` | <8B rejected | L3-BCD-003 |
| `day_fmt_to_absolute` | → AbsoluteTime | L3-BCD-007 |
| `dmy_fmt_decode_known` | Mar15 2025 | L3-BCD-004 |
| `dmy_fmt_to_absolute_with_date` | Full date | L3-BCD-007 |
| `dmy_fmt_invalid_month_zero` | Month=0 | L3-BCD-006 |
| `dmy_fmt_buffer_too_short` | <10B rejected | L3-BCD-004 |
| `millisecond_resolution_10ms` | 10ms granularity | L3-BCD-005 |

### secondary (10 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `sec_hdr_time_format_ch4` | Flags→Ch4 | L3-SEC-001 |
| `sec_hdr_time_format_ieee1588` | Flags→1588 | L3-SEC-001 |
| `sec_hdr_time_format_ertc` | Flags→Ertc | L3-SEC-001 |
| `sec_hdr_time_format_reserved` | Flags→Reserved | L3-SEC-001 |
| `checksum_valid` | Good checksum | L3-SEC-002 |
| `checksum_invalid` | Bad checksum | L3-SEC-002 |
| `checksum_buffer_too_short` | Short buffer | L3-SEC-002 |
| `parse_ieee1588_valid` | Full parse | L3-SEC-004 |
| `parse_ertc_valid` | Full parse | L3-SEC-004 |
| `parse_reserved_rejected` | Reserved err | L3-SEC-004 |

### intra_packet (8 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `format_no_secondary_is_rtc` | Default→Rtc48 | L3-IPT-003 |
| `format_ch4_binary` | Enum variant | L3-IPT-003 |
| `parse_rtc48_known` | RTC extraction | L3-IPT-002 |
| `parse_rtc48_reserved_ignored` | Bytes 6-7 ignored | L3-IPT-002 |
| `parse_ieee1588_known` | 1588 extraction | L3-IPT-002 |
| `parse_ertc64_known` | ERTC extraction | L3-IPT-002 |
| `parse_buffer_too_short` | Short buf err | L3-IPT-002 |
| `intra_packet_time_is_enum` | 4 variants | L3-IPT-001 |

### correlation (11 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `new_correlator_empty` | Init empty | L3-COR-002 |
| `add_reference_sorted` | Sorted insert | L3-COR-003 |
| `correlate_exact_match` | Exact RTC | L3-COR-004 |
| `correlate_interpolation_forward` | +15ms | L3-COR-004 |
| `correlate_nearest_point` | Nearest ref | L3-COR-004 |
| `correlate_channel_filter` | By channel | L3-COR-005 |
| `correlate_no_ref_returns_error` | Empty → err | L3-COR-002 |
| `correlate_no_channel_ref_returns_error` | Wrong ch → err | L3-COR-005 |
| `detect_no_jump` | Stable time | L3-COR-007 |
| `detect_gps_lock_jump` | 5s jump | L3-COR-007 |
| `detect_jump_threshold` | Threshold logic | L3-COR-007 |

### network_time (22 tests)
| Test | Validates | Traces |
|------|-----------|--------|
| `f2_csdw_ntp` | Protocol=0 → NTP | L1-F2CSDW-002 |
| `f2_csdw_ptp` | Protocol=1 → PTP | L1-F2CSDW-002 |
| `f2_csdw_reserved` | Protocol=5 → Reserved | L1-F2CSDW-002 |
| `f2_csdw_reserved_bits_clean` | Reserved zero → Ok | L1-F2CSDW-003 |
| `f2_csdw_reserved_bits_dirty` | Reserved set → Err | L1-F2CSDW-003 |
| `ntp_from_le_bytes` | Parse 8-byte NTP | L1-NTP-001 |
| `ntp_fraction_to_nanos` | 2⁻³² → nanoseconds | L1-NTP-003 |
| `ntp_to_unix_seconds` | NTP → Unix epoch | L1-NTP-004 |
| `ntp_before_unix_epoch` | Pre-1970 → None | L1-NTP-004 |
| `ntp_to_absolute` | NTP → AbsoluteTime | L1-NTP-001..004 |
| `ntp_buffer_too_short` | <8 bytes → Err | L1-NTP-001 |
| `ptp_from_le_bytes` | Parse 10-byte PTP | L1-PTP-001 |
| `ptp_nanos_overflow` | nanos >= 1B → Err | L1-PTP-003 |
| `ptp_to_utc_seconds` | TAI → UTC with offset | L1-PTP-004 |
| `ptp_to_absolute` | PTP → AbsoluteTime | L1-PTP-001..004 |
| `ptp_buffer_too_short` | <10 bytes → Err | L1-PTP-001 |
| `parse_f2_ntp_payload` | Full NTP payload dispatch | L1-F2CSDW-001, L1-NTP-001 |
| `parse_f2_ptp_payload` | Full PTP payload dispatch | L1-F2CSDW-001, L1-PTP-001 |
| `leap_second_table_builtin` | Built-in ≥27 entries | L1-TAI-002 |
| `leap_second_table_lookup_2024` | Offset @2024 = 37 | L1-TAI-001 |
| `leap_second_table_lookup_1980` | Offset @1980 = 19 | L1-TAI-001 |
| `leap_second_table_custom` | Custom add + lookup | L1-TAI-003 |

---

## 3. Integration Tests (tests/pipeline.rs, 50 tests)

| Test | Scenario | Traces |
|------|----------|--------|
| `full_day_format_pipeline` | CSDW→BCD DOY→Correlate | L1-CSDW-*, L1-BCD-001, L1-COR-* |
| `full_dmy_format_pipeline` | CSDW→BCD DMY→AbsoluteTime | L1-CSDW-005, L1-BCD-002 |
| `multi_channel_correlation` | IRIG-B vs GPS channels | L1-COR-003 |
| `gps_lock_time_jump_detection` | Detect GPS lock jump | L1-COR-004 |
| `secondary_header_to_correlation` | IEEE-1588 sec hdr parse | L1-SEC-002, L1-COR-001 |
| `intra_packet_rtc_to_absolute` | IPT→Correlate→Absolute | L1-IPT-001, L1-COR-002 |
| `rtc_large_delta_correlation` | Cross midnight | L1-RTC-003, L1-COR-002 |
| `invalid_bcd_propagates_error` | Bad BCD → typed error | L1-ERR-002 |
| `all_error_variants_display` | Display coverage | L1-ERR-001 |
| `no_std_types_are_copy` | Copy+Clone+Eq (incl. F2 types) | L1-API-001 |
| `full_ntp_pipeline` | F2 NTP→AbsoluteTime→Correlate | L1-NTP-*, L1-F2COR-001 |
| `full_ptp_pipeline` | F2 PTP→LeapTable→Correlate via add_reference_f2 | L1-PTP-*, L1-F2COR-001 |
| `mixed_f1_f2_correlation` | F1 + F2 sources coexist | L1-F2COR-001 |
| `ntp_sub_millisecond_precision` | Fractional second precision | L1-NTP-003 |
| `leap_second_table_historical_accuracy` | Historical offsets 1975/2000/2017/2026 | L1-TAI-001 |
