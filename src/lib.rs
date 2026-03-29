//! # irig106-time
//!
//! Nanosecond-precision time handling for IRIG 106 Chapter 10 telemetry data.
//!
//! This crate provides:
//!
//! - **RTC** — 48-bit Relative Time Counter newtype with wrap-safe arithmetic
//! - **Absolute time** — Day-of-year + time-of-day with optional calendar date
//! - **Wire formats** — Chapter 4 BWT, IEEE-1588, and Extended RTC decoding
//! - **CSDW parsing** — Time Data Format 1 (0x11) channel-specific data word
//! - **BCD decoding** — Day-of-Year and Day-Month-Year time message formats
//! - **Network time** — Time Data Format 2 (0x12): NTP (RFC 5905) and
//!   PTP/IEEE 1588 decoding with built-in leap-second table
//! - **Secondary headers** — Checksum validation and time extraction
//! - **Intra-packet timestamps** — Format-discriminated timestamp parsing
//! - **Correlation** — RTC-to-absolute-time interpolation engine with
//!   channel-indexed O(log n) lookup, time-jump detection, drift estimation,
//!   and RTC reset detection
//! - **Version detection** — IRIG 106 standard version identification from
//!   TMATS CSDW, with version-aware CSDW parsing for 106-04 through 106-23
//! - **Encoding** — `to_le_bytes()` on all wire-format types for packet
//!   construction (BCD, CSDW, NTP, PTP, RTC)
//! - **serde** — Optional `Serialize`/`Deserialize` on all public data types
//!   via the `serde` feature gate
//!
//! ## `no_std` Support
//!
//! The crate is `no_std`-compatible. Disable the default `std` feature to
//! remove the `std::error::Error` implementation on `TimeError`.
//! The `correlation` and `network_time` modules require `alloc`.
//!
//! ## Requirement Traceability
//!
//! Every public type and function traces to requirements in
//! `docs/L1_Requirements.md` → `L2_Requirements.md` → `L3_Requirements.md`,
//! which in turn trace to IRIG 106-17 Chapters 10/11 and RCC 123-20.

#![no_std]
#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

/// Absolute time representations: `AbsoluteTime`, `Ch4BinaryTime`, `Ieee1588Time`, `Ertc`.
pub mod absolute;
/// BCD-encoded Day-of-Year and Day-Month-Year time message decoding.
pub mod bcd;
/// RTC-to-absolute-time correlation engine with multi-channel support.
pub mod correlation;
/// Time Data Format 1 (0x11) Channel-Specific Data Word parsing.
pub mod csdw;
/// Error types and `Result` alias for time parsing operations.
pub mod error;
/// Intra-packet timestamp format dispatch (RTC, Ch4, IEEE-1588, ERTC).
pub mod intra_packet;
/// Time Data Format 2 (0x12) Network Time: NTP, PTP, and leap-second table.
pub mod network_time;
/// 48-bit Relative Time Counter (RTC) newtype.
pub mod rtc;
/// Secondary header time extraction and checksum validation.
pub mod secondary;
/// IRIG 106 standard version detection and version-aware dispatch.
pub mod version;

// Re-export key types at crate root for convenience.
pub use absolute::{AbsoluteTime, Ch4BinaryTime, Ertc, Ieee1588Time};
pub use bcd::{DayFormatTime, DmyFormatTime};
pub use correlation::{ReferencePoint, RtcReset, TimeCorrelator, TimeJump};
pub use csdw::{DateFormat, TimeF1Csdw, TimeFormat, TimeSource};
pub use error::{Result, TimeError};
pub use intra_packet::{IntraPacketTime, IntraPacketTimeFormat};
pub use network_time::{
    LeapSecondEntry, LeapSecondTable, NetworkTime, NetworkTimeProtocol, NtpTime, PtpTime,
    TimeF2Csdw, DEFAULT_TAI_UTC_OFFSET, NTP_UNIX_EPOCH_OFFSET,
};
pub use rtc::Rtc;
pub use secondary::{SecHdrTimeFormat, SecondaryHeaderTime};
pub use version::{detect_version, Irig106Version};
