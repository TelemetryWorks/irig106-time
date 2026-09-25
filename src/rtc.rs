//! 48-bit Relative Time Counter (RTC) for IRIG 106 Chapter 10.
//!
//! [`Rtc`] is defined in [`irig106_types`] and re-exported here so that
//! existing `irig106_time::rtc::Rtc` paths keep working.
//!
//! The RTC is a free-running 10 MHz counter providing 100 ns resolution.
//! It occupies 6 bytes (48 bits) in the primary packet header at bytes [16..22].
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-RTC-001  | Newtype `Rtc(u64)` with 48-bit invariant |
//! | L3-RTC-002  | `MASK_48` constant |
//! | L3-RTC-003  | `NANOS_PER_TICK` constant |
//! | L3-RTC-004  | `Rtc::ZERO` |
//! | L3-RTC-005  | `Rtc::MAX` |
//! | L3-RTC-006  | `from_le_bytes` |
//! | L3-RTC-007  | `from_raw` with masking |
//! | L3-RTC-008  | `as_raw` |
//! | L3-RTC-009  | `elapsed_ticks` with 48-bit wrap handling |
//! | L3-RTC-010  | `elapsed_nanos` |
//! | L3-RTC-011  | `to_nanos` |
//! | L3-RTC-012  | `Ord`/`PartialOrd` implementation |
//! | L3-RTC-013  | Standard derives |

pub use irig106_types::Rtc;
