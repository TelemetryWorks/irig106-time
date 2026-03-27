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
//! - **Secondary headers** — Checksum validation and time extraction
//! - **Intra-packet timestamps** — Format-discriminated timestamp parsing
//! - **Correlation** — RTC-to-absolute-time interpolation engine with
//!   multi-channel support and time-jump detection
//!
//! ## `no_std` Support
//!
//! The crate is `no_std`-compatible. Disable the default `std` feature to
//! remove the `std::error::Error` implementation on `TimeError`.
//! The `correlation` module requires `alloc`.
//!
//! ## Requirement Traceability
//!
//! Every public type and function traces to requirements in
//! `docs/L1_REQUIREMENTS.md` → `L2_REQUIREMENTS.md` → `L3_REQUIREMENTS.md`,
//! which in turn trace to IRIG 106-17 Chapter 10 and RCC 123-20.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub mod error;
pub mod rtc;
pub mod absolute;
pub mod csdw;
pub mod bcd;
pub mod secondary;
pub mod intra_packet;
pub mod correlation;

// Re-export key types at crate root for convenience.
pub use error::{TimeError, Result};
pub use rtc::Rtc;
pub use absolute::{AbsoluteTime, Ch4BinaryTime, Ieee1588Time, Ertc};
pub use csdw::{TimeF1Csdw, TimeSource, TimeFormat, DateFormat};
pub use bcd::{DayFormatTime, DmyFormatTime};
pub use secondary::{SecHdrTimeFormat, SecondaryHeaderTime};
pub use intra_packet::{IntraPacketTime, IntraPacketTimeFormat};
pub use correlation::{TimeCorrelator, ReferencePoint, TimeJump};
