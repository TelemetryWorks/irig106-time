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
    pub fn serialize<S: Serializer>(v: &usize, s: S) -> core::result::Result<S::Ok, S::Error> {
        Decimal::new(*v).serialize(s)
    }
}
