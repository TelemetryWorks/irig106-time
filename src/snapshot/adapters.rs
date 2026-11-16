//! Exact native leaf field adapters; no saturation or masked RTC construction.
use super::*;
macro_rules! field {
    ($module:ident,$t:ty,$p:ty,$get:expr,$make:expr) => {
        pub(crate) mod $module {
            use super::*;
            pub fn serialize<S: Serializer>(v: &$t, s: S) -> core::result::Result<S::Ok, S::Error> {
                ($make)(($get)(*v)).map_err(serde::ser::Error::custom)?;
                Decimal::<$p>::new(($get)(*v)).serialize(s)
            }
            pub fn deserialize<'de, D: Deserializer<'de>>(
                d: D,
            ) -> core::result::Result<$t, D::Error> {
                ($make)(Decimal::<$p>::deserialize(d)?.get()).map_err(de::Error::custom)
            }
        }
    };
}
field!(ptp_seconds, u32, u64, |v: u32| u64::from(v), |v: u64| {
    u32::try_from(v).map_err(|_| invalid())
});
field!(rtc, Rtc, u64, |v: Rtc| v.as_raw(), crate::rtc::checked_rtc);
field!(ertc, Ertc, u64, |v: Ertc| v.as_raw(), |v: u64| {
    Ertc::from_le_bytes(&v.to_le_bytes()).map_err(TimeError::from)
});

pub(crate) mod count {
    use super::*;
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> core::result::Result<usize, D::Error> {
        Ok(Decimal::<usize>::deserialize(d)?.get())
    }

    pub fn serialize<S: Serializer>(v: &usize, s: S) -> core::result::Result<S::Ok, S::Error> {
        Decimal::new(*v).serialize(s)
    }
}

pub(crate) mod ch4 {
    use super::*;
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        high_order: u16,
        low_order: u16,
        microseconds: u16,
    }
    fn validate(v: &Ch4BinaryTime) -> Result<()> {
        if v.microseconds > 9999 {
            return Err(TimeError::OutOfRange {
                field: "ch4_microseconds",
                value: v.microseconds.into(),
                max: 9999,
            });
        }
        Ok(())
    }
    pub fn serialize<S: Serializer>(
        v: &Ch4BinaryTime,
        s: S,
    ) -> core::result::Result<S::Ok, S::Error> {
        validate(v).map_err(serde::ser::Error::custom)?;
        v.serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> core::result::Result<Ch4BinaryTime, D::Error> {
        let w: Wire = object(d)?;
        let v = Ch4BinaryTime {
            high_order: w.high_order,
            low_order: w.low_order,
            microseconds: w.microseconds,
        };
        validate(&v).map_err(de::Error::custom)?;
        Ok(v)
    }
}
pub(crate) mod ieee {
    use super::*;
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        seconds: u32,
        nanoseconds: u32,
    }
    fn validate(v: &Ieee1588Time) -> Result<()> {
        if v.nanoseconds >= 1_000_000_000 {
            return Err(TimeError::OutOfRange {
                field: "ieee_nanoseconds",
                value: v.nanoseconds,
                max: 999_999_999,
            });
        }
        Ok(())
    }
    pub fn serialize<S: Serializer>(
        v: &Ieee1588Time,
        s: S,
    ) -> core::result::Result<S::Ok, S::Error> {
        validate(v).map_err(serde::ser::Error::custom)?;
        v.serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> core::result::Result<Ieee1588Time, D::Error> {
        let w: Wire = object(d)?;
        let v = Ieee1588Time {
            seconds: w.seconds,
            nanoseconds: w.nanoseconds,
        };
        validate(&v).map_err(de::Error::custom)?;
        Ok(v)
    }
}

pub(crate) fn era<'de, D: Deserializer<'de>>(d: D) -> core::result::Result<i64, D::Error> {
    Ok(Decimal::<i64>::deserialize(d)?.get())
}
