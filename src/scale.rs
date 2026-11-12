//! Scale evidence for wire readings (RDG-009). No epoch/leap arithmetic here.
use crate::csdw::TimeFormat;

/// Time scale, independent of epoch or counter units.
/// **Traces:** L3-TAI-105 ← L2-TAI-105
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeScale {
    /// Coordinated Universal Time.
    Utc,
    /// International Atomic Time.
    Tai,
    /// Native GPS time, not UTC.
    Gps,
    /// Not established by the available evidence.
    Unknown,
}

/// Why the effective scale is assigned.
/// **Traces:** L3-TAI-105 ← L2-TAI-105
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScaleOrigin {
    /// Explicitly stated for this format by the standard.
    Standard,
    /// An explicit/default caller policy assumption.
    Policy,
    /// No scale has been established.
    Unspecified,
}

/// Stated and effective scales retained independently.
/// **Traces:** L3-TAI-105 ← L2-TAI-105
/// **Traces:** L3-TAI-106 ← L2-TAI-106
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScaleEvidence {
    stated: TimeScale,
    effective: TimeScale,
    origin: ScaleOrigin,
}
impl ScaleEvidence {
    /// A scale named by the standard, or unspecified for Unknown.
    pub fn standard(scale: TimeScale) -> Self {
        Self {
            stated: scale,
            effective: scale,
            origin: if scale == TimeScale::Unknown {
                ScaleOrigin::Unspecified
            } else {
                ScaleOrigin::Standard
            },
        }
    }
    /// No implicit scale for unqualified calendar values or relative counters.
    pub fn unknown() -> Self {
        Self::standard(TimeScale::Unknown)
    }
    /// A labelled assumption; retains the original unknown designation.
    pub fn assumed(scale: TimeScale) -> Self {
        Self {
            stated: TimeScale::Unknown,
            effective: scale,
            origin: if scale == TimeScale::Unknown {
                ScaleOrigin::Unspecified
            } else {
                ScaleOrigin::Policy
            },
        }
    }
    /// Original scale stated for the format, before any assumption.
    pub fn stated(self) -> TimeScale {
        self.stated
    }
    /// Scale used by the explicitly labelled reading.
    pub fn effective(self) -> TimeScale {
        self.effective
    }
    /// Source of the effective assignment.
    pub fn origin(self) -> ScaleOrigin {
        self.origin
    }
}

/// Explicit scale assumptions for otherwise unspecified Format 1 sources.
/// Stated UTC/GPS formats cannot be relabelled by these settings.
/// **Traces:** L3-TAI-106 ← L2-TAI-106
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScalePolicy {
    /// IRIG-A/B/G assumption. Default UTC, labelled Policy (RCC 200-16 §1).
    pub irig: TimeScale,
    /// Real-time-clock assumption. Default Unknown; never infer from SRC.
    pub rtc: TimeScale,
}
impl Default for ScalePolicy {
    fn default() -> Self {
        Self {
            irig: TimeScale::Utc,
            rtc: TimeScale::Unknown,
        }
    }
}
impl ScalePolicy {
    /// 106-24R1 §11.2.3.2a, printed 11-25; RDG-009.
    /// **Traces:** L3-TAI-105 ← L2-TAI-105
    pub fn for_format1(self, format: TimeFormat) -> ScaleEvidence {
        match format {
            TimeFormat::IrigA | TimeFormat::IrigB | TimeFormat::IrigG => {
                ScaleEvidence::assumed(self.irig)
            }
            TimeFormat::Rtc => ScaleEvidence::assumed(self.rtc),
            TimeFormat::Utc => ScaleEvidence::standard(TimeScale::Utc),
            TimeFormat::Gps => ScaleEvidence::standard(TimeScale::Gps),
            _ => ScaleEvidence::unknown(),
        }
    }
}

/// TAI minus GPS = 35 − 16 = 19 seconds, inferred in RDG-009 from RCC 200-16
/// Appendix A.2 (A-1), not an instruction to treat native GPS as UTC.
pub const TAI_MINUS_GPS_SECONDS: i32 = 19;

/// Value paired with scale provenance, without implicit normalization.
/// **Traces:** L3-TAI-105 ← L2-TAI-105
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scaled<T> {
    /// Unconverted decoded value.
    pub value: T,
    /// Stated/effective scale and origin.
    pub scale: ScaleEvidence,
}

/// Scale evidence attached to a decoded value. Unqualified legacy readers
/// cannot establish a scale: they report Unknown, rather than silently UTC.
/// **Traces:** L3-TAI-106 ← L2-TAI-106
pub trait TimeScaleTagged {
    /// Return scale evidence without converting the value.
    fn scale(&self) -> ScaleEvidence;
}
impl<T> TimeScaleTagged for Scaled<T> {
    fn scale(&self) -> ScaleEvidence {
        self.scale
    }
}

macro_rules! unknown_scale {
    ($($t:ty),+ $(,)?) => { $(impl TimeScaleTagged for $t {
        fn scale(&self) -> ScaleEvidence { ScaleEvidence::unknown() }
    })+ };
}
unknown_scale!(
    crate::AbsoluteTime,
    crate::CalendarTime,
    crate::DayFormatTime,
    crate::DmyFormatTime,
    crate::Rtc,
    crate::Ertc,
    crate::Ch4BinaryTime,
    crate::Ieee1588Time,
    crate::SecondaryHeaderTime,
    crate::IntraPacketTime
);
