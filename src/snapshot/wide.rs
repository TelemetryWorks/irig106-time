//! Exact fields use one fixed token serializer; recursive evidence never nests
//! an unbounded Serializer<Serializer<...>> monomorphization.
use super::*;
pub(crate) struct Exact<'a, T: ?Sized>(pub &'a T);
impl<T: Serialize + ?Sized> Serialize for Exact<'_, T> {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        super::data::capture(self.0)
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }
}
pub(crate) fn serialize<T: Serialize + ?Sized, S: Serializer>(
    v: &T,
    s: S,
) -> core::result::Result<S::Ok, S::Error> {
    Exact(v).serialize(s)
}
macro_rules! serialize_struct {
    ($name:ident { $($field:ident),* $(,)? })=>{
        #[cfg(feature="serde")]
        impl serde::Serialize for $name {
            fn serialize<S:serde::Serializer>(&self,s:S)->core::result::Result<S::Ok,S::Error>{
                use serde::ser::SerializeStruct;
                let mut out=s.serialize_struct(stringify!($name),[$(stringify!($field)),*].len())?;
                $(out.serialize_field(stringify!($field),&crate::snapshot::wide::Exact(&self.$field))?;)*
                out.end()
            }
        }
    };
}
pub(crate) use serialize_struct;
