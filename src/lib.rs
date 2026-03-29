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
//!   multi-channel support, time-jump detection, and drift estimation
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

/// Error types and `Result` alias for time parsing operations.
pub mod error;
/// 48-bit Relative Time Counter (RTC) newtype.
pub mod rtc;
/// Absolute time representations: `AbsoluteTime`, `Ch4BinaryTime`, `Ieee1588Time`, `Ertc`.
pub mod absolute;
/// Time Data Format 1 (0x11) Channel-Specific Data Word parsing.
pub mod csdw;
/// BCD-encoded Day-of-Year and Day-Month-Year time message decoding.
pub mod bcd;
/// Secondary header time extraction and checksum validation.
pub mod secondary;
/// Intra-packet timestamp format dispatch (RTC, Ch4, IEEE-1588, ERTC).
pub mod intra_packet;
/// RTC-to-absolute-time correlation engine with multi-channel support.
pub mod correlation;
/// Time Data Format 2 (0x12) Network Time: NTP, PTP, and leap-second table.
pub mod network_time;

// Re-export key types at crate root for convenience.
pub use error::{TimeError, Result};
pub use rtc::Rtc;
pub use absolute::{AbsoluteTime, Ch4BinaryTime, Ieee1588Time, Ertc};
pub use csdw::{TimeF1Csdw, TimeSource, TimeFormat, DateFormat};
pub use bcd::{DayFormatTime, DmyFormatTime};
pub use secondary::{SecHdrTimeFormat, SecondaryHeaderTime};
pub use intra_packet::{IntraPacketTime, IntraPacketTimeFormat};
pub use correlation::{TimeCorrelator, ReferencePoint, TimeJump};
pub use network_time::{
    TimeF2Csdw, NetworkTimeProtocol, NtpTime, PtpTime, NetworkTime,
    LeapSecondTable, LeapSecondEntry, NTP_UNIX_EPOCH_OFFSET, DEFAULT_TAI_UTC_OFFSET,
};
