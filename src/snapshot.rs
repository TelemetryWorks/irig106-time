//! S12b1 schema candidate: exact scalar/calendar/leap values, owner Q5.
//! The JSON schema is reviewed before freeze; policy/answer reports follow in S12b2.
//! No float coercion, implicit unit, default version or live table lookup.
//!
//! Complex engine-state loaders are intentionally absent (owner Q3):
//! ```compile_fail
//! use irig106_time::timeline::OfflineTimeline;
//! fn load<T: for<'de> serde::Deserialize<'de>>() {}
//! load::<OfflineTimeline>();
//! ```
//! ```compile_fail
//! use irig106_time::stream::StreamTimeline;
//! fn load<T: for<'de> serde::Deserialize<'de>>() {}
//! load::<StreamTimeline>();
//! ```
use crate::{format1::CalendarReading, *};
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};
use core::{fmt, marker::PhantomData, str::FromStr};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, Visitor, value::MapAccessDeserializer},
};
mod sealed {
    pub trait Sealed {}
}
/// Supported integer primitives for exact formatting; deliberately sealed.
pub trait ExactInteger: sealed::Sealed + Copy + fmt::Display + FromStr {}
macro_rules! integers {($($t:ty),*)=>{$(impl sealed::Sealed for $t {} impl ExactInteger for $t {})*}}
integers!(u64, i64, u128, i128, usize);
/// Exact canonical base10 integer. Serializes as a JSON string, never a number.
/// **Traces:** L3-API-107 ← L2-API-107 ← L1-API-007
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Decimal<T>(T);
impl<T: ExactInteger> Decimal<T> {
    /// Wrap a typed primitive without conversion or saturation.
    pub const fn new(value: T) -> Self {
        Self(value)
    }
    /// Original exact value.
    pub const fn get(self) -> T {
        self.0
    }
}
fn parse<T: ExactInteger>(text: &str) -> core::result::Result<T, &'static str> {
    if text.len() > 40 {
        return Err("decimal integer too long");
    }
    let value = text.parse::<T>().map_err(|_| "integer out of range")?;
    if value.to_string() != text {
        return Err("noncanonical decimal integer");
    }
    Ok(value)
}
impl<T: ExactInteger> Serialize for Decimal<T> {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.0.to_string())
    }
}
impl<'de, T: ExactInteger> Deserialize<'de> for Decimal<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        struct DecimalVisitor<T>(PhantomData<T>);
        impl<T: ExactInteger> Visitor<'_> for DecimalVisitor<T> {
            type Value = Decimal<T>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("canonical decimal integer string")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> core::result::Result<Self::Value, E> {
                parse(v).map(Decimal).map_err(E::custom)
            }
        }
        d.deserialize_str(DecimalVisitor(PhantomData))
    }
}
pub(crate) fn object<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> core::result::Result<T, D::Error> {
    struct Object<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Object<T> {
        type Value = T;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("snapshot object")
        }
        fn visit_map<A: MapAccess<'de>>(self, m: A) -> core::result::Result<T, A::Error> {
            T::deserialize(MapAccessDeserializer::new(m))
        }
    }
    d.deserialize_map(Object(PhantomData))
}
fn invalid() -> TimeError {
    TimeError::InvalidTimelineInput("snapshot schema/version/unit/kind")
}
fn header(schema: &str, version: u16, expected: &str) -> Result<()> {
    if schema != expected || version != 1 {
        Err(invalid())
    } else {
        Ok(())
    }
}
/// Typed quantities supported by the scalar schema; counts have no time epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarValue {
    /// Raw48-bit RTC ticks (10MHz); no epoch.
    Rtc(Rtc),
    /// Raw64-bit ERTC,1ns per tick; no epoch.
    Ertc(Ertc),
    /// Unsigned nanoseconds.
    Nanos(NanosDuration),
    /// Signed nanoseconds, including i128 extrema.
    SignedNanos(SignedNanos),
    /// Signed unwrapped RTC ticks, with session context supplied separately.
    SignedTicks(SignedRtcTicks),
    /// Unix-style UTC seconds (no cumulative leap count).
    Unix(UnixSeconds),
    /// Signed Unix-style UTC seconds, including pre1970.
    SignedUnix(SignedUnixSeconds),
    /// TAI seconds since1970.
    Tai(TaiSeconds),
    /// Dimensionless unsigned64-bit count.
    U64(u64),
    /// Dimensionless signed64-bit count.
    I64(i64),
    /// Dimensionless unsigned128-bit count.
    U128(u128),
    /// Dimensionless signed128-bit count.
    I128(i128),
}
impl ScalarValue {
    fn wire(self) -> (&'static str, &'static str, String) {
        match self {
            Self::Rtc(v) => ("rtc", "rtc_ticks", v.as_raw().to_string()),
            Self::Ertc(v) => ("ertc", "ns", v.as_raw().to_string()),
            Self::Nanos(v) => ("nanos", "ns", v.get().to_string()),
            Self::SignedNanos(v) => ("signed_nanos", "ns", v.get().to_string()),
            Self::SignedTicks(v) => ("signed_ticks", "rtc_ticks", v.get().to_string()),
            Self::Unix(v) => ("unix", "unix_s", v.get().to_string()),
            Self::SignedUnix(v) => ("signed_unix", "unix_s", v.get().to_string()),
            Self::Tai(v) => ("tai", "tai_s", v.get().to_string()),
            Self::U64(v) => ("u64", "count", v.to_string()),
            Self::I64(v) => ("i64", "count", v.to_string()),
            Self::U128(v) => ("u128", "count", v.to_string()),
            Self::I128(v) => ("i128", "count", v.to_string()),
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScalarWire {
    schema: String,
    version: u16,
    kind: String,
    unit: String,
    value: String,
}
/// Immutable version1 scalar snapshot with explicit kind, epoch/unit and string value.
/// **Traces:** L3-API-110 ← L2-API-110 ← L1-API-010
/// **Traces:** L3-POL-103 ← L2-POL-103 ← L1-POL-003
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalarSnapshot(ScalarValue);
impl ScalarSnapshot {
    /// Check wire width without masking caller data.
    pub fn new(value: ScalarValue) -> Result<Self> {
        if let ScalarValue::Rtc(r) = value {
            crate::rtc::checked_rtc(r.as_raw())?;
        }
        Ok(Self(value))
    }
    /// Actual typed quantity; no hidden epoch conversion.
    pub fn value(self) -> ScalarValue {
        self.0
    }
}
impl Serialize for ScalarSnapshot {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        let (kind, unit, value) = self.0.wire();
        ScalarWire {
            schema: "irig106-time/scalar".into(),
            version: 1,
            kind: kind.into(),
            unit: unit.into(),
            value,
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for ScalarSnapshot {
    fn deserialize<D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        let w: ScalarWire = object(d)?;
        let load = || -> Result<Self> {
            header(&w.schema, w.version, "irig106-time/scalar")?;
            let p = |e| TimeError::InvalidTimelineInput(e);
            let value = match w.kind.as_str() {
                "rtc" => ScalarValue::Rtc(crate::rtc::checked_rtc(parse(&w.value).map_err(p)?)?),
                "ertc" => ScalarValue::Ertc(Ertc::from_le_bytes(
                    &parse::<u64>(&w.value).map_err(p)?.to_le_bytes(),
                )?),
                "nanos" => ScalarValue::Nanos(NanosDuration::new(parse(&w.value).map_err(p)?)),
                "signed_nanos" => {
                    ScalarValue::SignedNanos(SignedNanos::new(parse(&w.value).map_err(p)?))
                }
                "signed_ticks" => {
                    ScalarValue::SignedTicks(SignedRtcTicks::new(parse(&w.value).map_err(p)?))
                }
                "unix" => ScalarValue::Unix(UnixSeconds::new(parse(&w.value).map_err(p)?)),
                "signed_unix" => {
                    ScalarValue::SignedUnix(SignedUnixSeconds::new(parse(&w.value).map_err(p)?))
                }
                "tai" => ScalarValue::Tai(TaiSeconds::new(parse(&w.value).map_err(p)?)),
                "u64" => ScalarValue::U64(parse(&w.value).map_err(p)?),
                "i64" => ScalarValue::I64(parse(&w.value).map_err(p)?),
                "u128" => ScalarValue::U128(parse(&w.value).map_err(p)?),
                "i128" => ScalarValue::I128(parse(&w.value).map_err(p)?),
                _ => return Err(invalid()),
            };
            if value.wire().1 != w.unit {
                return Err(invalid());
            }
            Self::new(value)
        };
        load().map_err(de::Error::custom)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    schema: String,
    version: u16,
    #[serde(deserialize_with = "unit_map")]
    units: BTreeMap<String, String>,
    value: T,
}
fn unit_map<'de, D: Deserializer<'de>>(
    d: D,
) -> core::result::Result<BTreeMap<String, String>, D::Error> {
    struct Units;
    impl<'de> Visitor<'de> for Units {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("unique unit keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut m: A,
        ) -> core::result::Result<Self::Value, A::Error> {
            let mut units = BTreeMap::new();
            while let Some((key, value)) = m.next_entry::<String, String>()? {
                if units.insert(key, value).is_some() {
                    return Err(de::Error::custom("duplicate unit key"));
                }
            }
            Ok(units)
        }
    }
    d.deserialize_map(Units)
}
fn units(fields: &[(&str, &str)]) -> BTreeMap<String, String> {
    fields
        .iter()
        .map(|(k, v)| ((*k).into(), (*v).into()))
        .collect()
}
fn calendar_units() -> BTreeMap<String, String> {
    units(&[
        ("year", "year"),
        ("day_of_year", "day"),
        ("hours", "h"),
        ("minutes", "min"),
        ("seconds", "s"),
        ("nanoseconds", "ns"),
    ])
}
fn leap_units() -> BTreeMap<String, String> {
    units(&[
        ("effective_unix", "unix_s"),
        ("tai_utc_offset", "s"),
        ("supported_start", "unix_s"),
        ("known_current_through", "unix_s"),
    ])
}
/// Validated calendar snapshot, including raw/stated/assumed scale provenance.
/// **Traces:** L3-API-110 ← L2-API-110 ← L1-API-010
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarSnapshot(CalendarReading);
impl CalendarSnapshot {
    /// Reject malformed fields or leap labels before snapshot creation.
    pub fn new(value: CalendarReading) -> Result<Self> {
        value.validate()?;
        Ok(Self(value))
    }
    /// Exact native or partial calendar; no inferred year or UTC conversion.
    pub fn value(self) -> CalendarReading {
        self.0
    }
}
impl Serialize for CalendarSnapshot {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        Envelope {
            schema: "irig106-time/calendar".into(),
            version: 1,
            units: calendar_units(),
            value: self.0,
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for CalendarSnapshot {
    fn deserialize<D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        let w: Envelope<CalendarReading> = object(d)?;
        header(&w.schema, w.version, "irig106-time/calendar").map_err(de::Error::custom)?;
        if w.units != calendar_units() {
            return Err(de::Error::custom(invalid()));
        }
        Self::new(w.value).map_err(de::Error::custom)
    }
}
/// Full validated leap-table snapshot, including independent currency metadata.
/// **Traces:** L3-API-110 ← L2-API-110 ← L1-API-010
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeapTableSnapshot(LeapSecondTable);
impl LeapTableSnapshot {
    /// Capture a table already validated by its constructor; no external table lookup.
    pub fn new(value: LeapSecondTable) -> Self {
        Self(value)
    }
    /// Complete immutable transition list/identity/coverage.
    pub fn value(&self) -> &LeapSecondTable {
        &self.0
    }
}
impl Serialize for LeapTableSnapshot {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        Envelope {
            schema: "irig106-time/leap-table".into(),
            version: 1,
            units: leap_units(),
            value: &self.0,
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for LeapTableSnapshot {
    fn deserialize<D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        let w: Envelope<LeapSecondTable> = object(d)?;
        header(&w.schema, w.version, "irig106-time/leap-table").map_err(de::Error::custom)?;
        if w.units != leap_units() {
            return Err(de::Error::custom(invalid()));
        }
        Ok(Self::new(w.value))
    }
}
/// Exact UnixSeconds field adapter; never invokes shared numeric Serde.
pub mod unix {
    use super::*;
    /// Serialize Unix seconds as an exact decimal string.
    pub fn serialize<S: Serializer>(
        v: &UnixSeconds,
        s: S,
    ) -> core::result::Result<S::Ok, S::Error> {
        Decimal::new(v.get()).serialize(s)
    }
    /// Decode a string with unsigned64-bit range and canonical spelling checks.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> core::result::Result<UnixSeconds, D::Error> {
        Ok(UnixSeconds::new(Decimal::<u64>::deserialize(d)?.get()))
    }
}
