//! Canonical JSON token tree: wide integer strings and strict duplicate keys.
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use serde::de::value::Error;
use serde::{Serialize, Serializer, ser};
#[derive(Debug, Clone)]
pub(crate) enum Node {
    Null,
    Bool(bool),
    Number(i64),
    Float(f64),
    String(String),
    Seq(Vec<Node>),
    Map(Vec<(String, Node)>),
}
pub(crate) struct Tree;
pub(crate) struct Seq {
    values: Vec<Node>,
    variant: Option<String>,
}
pub(crate) struct Map {
    fields: Vec<(String, Node)>,
    key: Option<String>,
    variant: Option<String>,
}
macro_rules! numeric {($($name:ident:$ty:ty),*)=>{$(fn $name(self,v:$ty)->Result<Node,Error>{Ok(Node::Number(i64::from(v)))})*}}
macro_rules! decimal {($($name:ident:$ty:ty),*)=>{$(fn $name(self,v:$ty)->Result<Node,Error>{Ok(Node::String(v.to_string()))})*}}
impl Serializer for Tree {
    type Ok = Node;
    type Error = Error;
    type SerializeSeq = Seq;
    type SerializeTuple = Seq;
    type SerializeTupleStruct = Seq;
    type SerializeTupleVariant = Seq;
    type SerializeMap = Map;
    type SerializeStruct = Map;
    type SerializeStructVariant = Map;
    numeric!(serialize_i8:i8,serialize_i16:i16,serialize_i32:i32,serialize_u8:u8,serialize_u16:u16,serialize_u32:u32);
    decimal!(serialize_i64:i64,serialize_i128:i128,serialize_u64:u64,serialize_u128:u128);
    fn serialize_f32(self, v: f32) -> Result<Node, Error> {
        self.serialize_f64(f64::from(v))
    }
    fn serialize_f64(self, v: f64) -> Result<Node, Error> {
        if v.is_finite() {
            Ok(Node::Float(v))
        } else {
            Err(ser::Error::custom("nonfinite report metric"))
        }
    }
    fn serialize_bool(self, v: bool) -> Result<Node, Error> {
        Ok(Node::Bool(v))
    }
    fn serialize_str(self, v: &str) -> Result<Node, Error> {
        Ok(Node::String(v.into()))
    }
    fn serialize_char(self, v: char) -> Result<Node, Error> {
        self.serialize_str(&v.to_string())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Node, Error> {
        Ok(Node::Seq(
            v.iter().map(|v| Node::Number(i64::from(*v))).collect(),
        ))
    }
    fn serialize_none(self) -> Result<Node, Error> {
        Ok(Node::Null)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<Node, Error> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<Node, Error> {
        Ok(Node::Null)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Node, Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
    ) -> Result<Node, Error> {
        self.serialize_str(v)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<Node, Error> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        k: &'static str,
        v: &T,
    ) -> Result<Node, Error> {
        Ok(Node::Map(vec![(k.into(), v.serialize(self)?)]))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Seq, Error> {
        Ok(Seq {
            values: Vec::new(),
            variant: None,
        })
    }
    fn serialize_tuple(self, n: usize) -> Result<Seq, Error> {
        self.serialize_seq(Some(n))
    }
    fn serialize_tuple_struct(self, _: &'static str, n: usize) -> Result<Seq, Error> {
        self.serialize_tuple(n)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
        _: usize,
    ) -> Result<Seq, Error> {
        Ok(Seq {
            values: Vec::new(),
            variant: Some(v.into()),
        })
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Map, Error> {
        Ok(Map {
            fields: Vec::new(),
            key: None,
            variant: None,
        })
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Map, Error> {
        self.serialize_map(None)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
        _: usize,
    ) -> Result<Map, Error> {
        Ok(Map {
            fields: Vec::new(),
            key: None,
            variant: Some(v.into()),
        })
    }
}
impl Seq {
    fn add<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        self.values.push(v.serialize(Tree)?);
        Ok(())
    }
    fn finish(self) -> Result<Node, Error> {
        let n = Node::Seq(self.values);
        Ok(if let Some(k) = self.variant {
            Node::Map(vec![(k, n)])
        } else {
            n
        })
    }
}
macro_rules! seq_impl {
    ($tr:ident,$method:ident) => {
        impl ser::$tr for Seq {
            type Ok = Node;
            type Error = Error;
            fn $method<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
                self.add(v)
            }
            fn end(self) -> Result<Node, Error> {
                self.finish()
            }
        }
    };
}
seq_impl!(SerializeSeq, serialize_element);
seq_impl!(SerializeTuple, serialize_element);
seq_impl!(SerializeTupleStruct, serialize_field);
seq_impl!(SerializeTupleVariant, serialize_field);
impl Map {
    fn field<T: Serialize + ?Sized>(&mut self, k: &str, v: &T) -> Result<(), Error> {
        self.fields.push((k.into(), v.serialize(Tree)?));
        Ok(())
    }
    fn finish(self) -> Result<Node, Error> {
        let mut fields = self.fields;
        fields.sort_by(|a, b| a.0.cmp(&b.0));
        let n = Node::Map(fields);
        Ok(if let Some(k) = self.variant {
            Node::Map(vec![(k, n)])
        } else {
            n
        })
    }
}
impl ser::SerializeMap for Map {
    type Ok = Node;
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Error> {
        let k = match k.serialize(Tree)? {
            Node::String(k) => k,
            Node::Number(k) => k.to_string(),
            _ => return Err(ser::Error::custom("scalar map key")),
        };
        self.key = Some(k);
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        let k = self
            .key
            .take()
            .ok_or_else(|| ser::Error::custom("map value without key"))?;
        self.field(&k, v)
    }
    fn end(self) -> Result<Node, Error> {
        self.finish()
    }
}
macro_rules! map_impl {
    ($tr:ident) => {
        impl ser::$tr for Map {
            type Ok = Node;
            type Error = Error;
            fn serialize_field<T: Serialize + ?Sized>(
                &mut self,
                k: &'static str,
                v: &T,
            ) -> Result<(), Error> {
                self.field(k, v)
            }
            fn end(self) -> Result<Node, Error> {
                self.finish()
            }
        }
    };
}
map_impl!(SerializeStruct);
map_impl!(SerializeStructVariant);
impl Serialize for Node {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Float(v) => s.serialize_f64(*v),
            Self::Null => s.serialize_none(),
            Self::Bool(v) => s.serialize_bool(*v),
            Self::String(v) => s.serialize_str(v),
            Self::Number(v) => {
                if *v < 0 {
                    s.serialize_i32(i32::try_from(*v).map_err(ser::Error::custom)?)
                } else {
                    s.serialize_u32(u32::try_from(*v).map_err(ser::Error::custom)?)
                }
            }
            Self::Seq(v) => v.serialize(s),
            Self::Map(fields) => {
                use ser::SerializeMap;
                let mut m = s.serialize_map(Some(fields.len()))?;
                for (k, v) in fields {
                    m.serialize_entry(k, v)?;
                }
                m.end()
            }
        }
    }
}
pub(crate) fn capture<T: Serialize + ?Sized>(v: &T) -> Result<Node, Error> {
    v.serialize(Tree)
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Float(a), Self::Number(b)) | (Self::Number(b), Self::Float(a)) => {
                *a == *b as f64
            }
            (Self::Seq(a), Self::Seq(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            _ => false,
        }
    }
}
impl<'de> serde::Deserialize<'de> for Node {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
        struct Seed(u8);
        struct Value(u8);
        impl<'de> DeserializeSeed<'de> for Seed {
            type Value = Node;
            fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Node, D::Error> {
                if self.0 >= 128 {
                    return Err(de::Error::custom("report value nesting depth"));
                }
                d.deserialize_any(Value(self.0))
            }
        }
        impl<'de> Visitor<'de> for Value {
            type Value = Node;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("canonical report JSON value")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Node, E> {
                Ok(Node::Null)
            }
            fn visit_none<E: de::Error>(self) -> Result<Node, E> {
                Ok(Node::Null)
            }
            fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Node, D::Error> {
                Seed(self.0 + 1).deserialize(d)
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Node, E> {
                Ok(Node::Bool(v))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Node, E> {
                Ok(Node::String(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Node, E> {
                i32::try_from(v).map_err(E::custom)?;
                Ok(Node::Number(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Node, E> {
                u32::try_from(v).map_err(E::custom)?;
                Ok(Node::Number(v as i64))
            }
            fn visit_i128<E: de::Error>(self, v: i128) -> Result<Node, E> {
                self.visit_i64(i64::try_from(v).map_err(E::custom)?)
            }
            fn visit_u128<E: de::Error>(self, v: u128) -> Result<Node, E> {
                self.visit_u64(u64::try_from(v).map_err(E::custom)?)
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Node, E> {
                if v.is_finite() {
                    Ok(Node::Float(v))
                } else {
                    Err(E::custom("nonfinite report metric"))
                }
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Node, A::Error> {
                let mut v = Vec::new();
                while let Some(n) = a.next_element_seed(Seed(self.0 + 1))? {
                    v.push(n);
                }
                Ok(Node::Seq(v))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Node, A::Error> {
                let mut v = alloc::collections::BTreeMap::new();
                while let Some(k) = a.next_key::<String>()? {
                    let n = a.next_value_seed(Seed(self.0 + 1))?;
                    if v.insert(k, n).is_some() {
                        return Err(de::Error::custom("duplicate report field"));
                    }
                }
                Ok(Node::Map(v.into_iter().collect()))
            }
        }
        Seed(0).deserialize(d)
    }
}
