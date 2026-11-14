#![cfg(feature = "serde")]
//! A minimal Serde value source, not a JSON/schema commitment or new dependency.
use irig106_time::LeapSecondTable;
use serde::{
    Deserialize, Deserializer,
    de::{
        IntoDeserializer, Visitor,
        value::{Error, SeqDeserializer},
    },
};
enum Value {
    U(u64),
    I(i32),
    S(&'static str),
    Seq(Vec<Value>),
}
impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self {
        self
    }
}
impl<'de> Deserializer<'de> for Value {
    type Error = Error;
    fn deserialize_any<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        match self {
            Self::U(x) => v.visit_u64(x),
            Self::I(x) => v.visit_i32(x),
            Self::S(x) => v.visit_borrowed_str(x),
            Self::Seq(x) => v.visit_seq(SeqDeserializer::new(x.into_iter())),
        }
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        v: V,
    ) -> Result<V::Value, Error> {
        v.visit_newtype_struct(self)
    }
    serde::forward_to_deserialize_any! {bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf option unit unit_struct seq tuple tuple_struct map struct enum identifier ignored_any}
}
fn wire(duplicate: bool) -> Value {
    let entry = || Value::Seq(vec![Value::U(63_072_000), Value::I(10)]);
    Value::Seq(vec![
        Value::Seq(if duplicate {
            vec![entry(), entry()]
        } else {
            vec![entry()]
        }),
        Value::Seq(vec![
            Value::S("serde-fixture"),
            Value::S("caller"),
            Value::U(63_072_000),
            Value::U(100_000_000),
        ]),
    ])
}
/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101, L1-TAI-003, L2-TAI-103, L3-TAI-103
#[test]
fn deserialization_uses_the_same_table_validation() {
    assert_eq!(LeapSecondTable::deserialize(wire(false)).unwrap().len(), 1);
    assert!(LeapSecondTable::deserialize(wire(true)).is_err());
}
