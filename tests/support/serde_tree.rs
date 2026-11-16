//! Dependency-free Serde protocol fixture; JSON emission only, not a runtime parser.
#![allow(dead_code)]
use serde::de::value::{Error, MapDeserializer, SeqDeserializer};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, IntoDeserializer, Visitor},
    ser,
};
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Seq(Vec<Node>),
    Map(Vec<(String, Node)>),
}
impl Node {
    pub fn field(&mut self, key: &str) -> &mut Node {
        let Self::Map(fields) = self else {
            panic!("object")
        };
        &mut fields.iter_mut().find(|(k, _)| k == key).unwrap().1
    }
    pub fn json(&self) -> String {
        fn quote(s: &str) -> String {
            let mut out = String::from("\"");
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    c if c < ' ' => out.push_str(&format!("\\u{:04x}", u32::from(c))),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        match self {
            Self::Null => "null".into(),
            Self::Bool(b) => b.to_string(),
            Self::Number(n) => n.clone(),
            Self::String(s) => quote(s),
            Self::Seq(v) => format!(
                "[{}]",
                v.iter().map(Self::json).collect::<Vec<_>>().join(",")
            ),
            Self::Map(m) => format!(
                "{{{}}}",
                m.iter()
                    .map(|(k, v)| format!("{}:{}", quote(k), v.json()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}
pub fn encode<T: Serialize>(value: &T) -> Node {
    value.serialize(Tree).unwrap()
}
pub fn decode<T: for<'de> Deserialize<'de>>(value: Node) -> Result<T, Error> {
    T::deserialize(value)
}
pub struct Tree;
pub struct Seq {
    values: Vec<Node>,
    variant: Option<String>,
}
pub struct Map {
    fields: Vec<(String, Node)>,
    key: Option<String>,
    variant: Option<String>,
}
macro_rules! numeric {($($name:ident:$ty:ty),*)=>{$(fn $name(self,v:$ty)->Result<Node,Error>{Ok(Node::Number(v.to_string()))})*}}
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
    numeric!(serialize_i8:i8,serialize_i16:i16,serialize_i32:i32,serialize_i64:i64,serialize_i128:i128,
        serialize_u8:u8,serialize_u16:u16,serialize_u32:u32,serialize_u64:u64,serialize_u128:u128);
    fn serialize_f32(self, v: f32) -> Result<Node, Error> {
        Ok(Node::Number(format!("{v:?}")))
    }
    fn serialize_f64(self, v: f64) -> Result<Node, Error> {
        Ok(Node::Number(format!("{v:?}")))
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
            v.iter().map(|v| Node::Number(v.to_string())).collect(),
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
        let n = Node::Map(self.fields);
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
        let Node::String(k) = k.serialize(Tree)? else {
            return Err(ser::Error::custom("string key"));
        };
        self.key = Some(k);
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        let k = self.key.take().unwrap();
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
impl<'de> IntoDeserializer<'de, Error> for Node {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self {
        self
    }
}
impl<'de> Deserializer<'de> for Node {
    type Error = Error;
    fn deserialize_any<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        match self {
            Self::Null => v.visit_unit(),
            Self::Bool(b) => v.visit_bool(b),
            Self::String(s) => v.visit_string(s),
            Self::Seq(s) => v.visit_seq(SeqDeserializer::new(s.into_iter())),
            Self::Map(m) => v.visit_map(MapDeserializer::new(m.into_iter())),
            Self::Number(n) => {
                if let Ok(i) = n.parse::<u64>() {
                    v.visit_u64(i)
                } else if let Ok(i) = n.parse::<i128>() {
                    v.visit_i128(i)
                } else if let Ok(i) = n.parse::<u128>() {
                    v.visit_u128(i)
                } else {
                    v.visit_f64(n.parse().map_err(de::Error::custom)?)
                }
            }
        }
    }
    fn deserialize_option<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        if self == Self::Null {
            v.visit_none()
        } else {
            v.visit_some(self)
        }
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        v: V,
    ) -> Result<V::Value, Error> {
        v.visit_newtype_struct(self)
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        v: V,
    ) -> Result<V::Value, Error> {
        match self {
            Self::String(s) => v.visit_enum(s.into_deserializer()),
            Self::Map(mut m) if m.len() == 1 => {
                let (k, n) = m.remove(0);
                v.visit_enum(Enum(k, n))
            }
            _ => Err(de::Error::custom("enum")),
        }
    }
    serde::forward_to_deserialize_any! {bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf unit unit_struct seq tuple tuple_struct map struct identifier ignored_any}
}
struct Enum(String, Node);
impl<'de> de::EnumAccess<'de> for Enum {
    type Error = Error;
    type Variant = Node;
    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Node), Error> {
        Ok((seed.deserialize(self.0.into_deserializer())?, self.1))
    }
}
impl<'de> de::VariantAccess<'de> for Node {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        if self == Self::Null {
            Ok(())
        } else {
            Err(de::Error::custom("unit"))
        }
    }
    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        _: &'static [&'static str],
        v: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
}
