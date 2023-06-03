#[cfg(feature = "egui_serde")]
pub mod egui;
#[cfg(feature = "imgui_serde")]
pub mod imgui;
/*
#[cfg(feature = "imgui_serde")]
pub mod imgui_2;

pub use serde_reflection;
*/

use serde::Deserialize;
use serde::Deserializer;
use serde::de::Visitor;
use serde::Serializer;
use serde::ser::*;

#[derive(Clone,Debug, PartialEq)]
pub struct StructField {
    pub name: String,
    pub value: ReflectType,
}

#[derive(Clone,Debug, PartialEq)]
pub struct EnumVariant {
    pub enum_name: String,
    pub variant_name: String,
    pub value: ReflectType,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum ReflectType {
    bool(bool),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    f32(f32),
    f64(f64),
    char(char),
    bytes(Vec<u8>),
    Option(Option<Box<ReflectType>>),
    Tuple(Vec<ReflectType>),
    Seq(Vec<ReflectType>),
    Enum(Box<EnumVariant>),
    String(String),
    Struct(String, Vec<StructField>),
    Unit(),
}

impl<'a> ReflectType {
    pub fn deserialize<T: Deserialize<'a>>(&self) -> T {
        T::deserialize(self.clone()).unwrap()
    }
}

pub struct ReflectedStructSerializer {
    data: Option<ReflectType>,
}

impl ReflectedStructSerializer {
    pub fn serialize<T: Serialize>(data: &T) -> ReflectType {
        data.serialize(ReflectedStructSerializer { data: None }).unwrap()
    }
}

macro_rules! impl_types {
    ($function: ident, $type: ident) => {
      fn $function(self, v: $type) -> Result<Self::Ok, Self::Error> {
          Ok(ReflectType::$type(v))
      }
    };
}

macro_rules! type_ok_err {
    ($Ok: ident, $Error: ident) => {
        type $Ok = ReflectType;
        type $Error = serde::de::value::Error;
    };
}

macro_rules! type_serialize_impossible {
    ($($type_impl: ident),+) => {
        $(type $type_impl = $crate::Impossible<ReflectType, serde::de::value::Error>;)+
    };
}

macro_rules! type_serialize_self {
    ($($type_impl: ident),+) => {
        $(type $type_impl = Self;)+
    };
}

impl Serializer for ReflectedStructSerializer {
    type_ok_err!(Ok, Error);
    type_serialize_impossible!(SerializeMap, SerializeStructVariant, SerializeTupleStruct, SerializeTupleVariant);
    type_serialize_self!(SerializeStruct, SerializeSeq, SerializeTuple);
    impl_types!(serialize_bool, bool);
    impl_types!(serialize_i8, i8);
    impl_types!(serialize_i16, i16);
    impl_types!(serialize_i32, i32);
    impl_types!(serialize_i64, i64);
    impl_types!(serialize_u8, u8);
    impl_types!(serialize_u16, u16);
    impl_types!(serialize_u32, u32);
    impl_types!(serialize_u64, u64);
    impl_types!(serialize_f32, f32);
    impl_types!(serialize_f64, f64);
    impl_types!(serialize_char, char);

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(ReflectType::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(ReflectType::bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(ReflectType::Option(None))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
            Ok(ReflectType::Option(Some(Box::new(
                value.serialize(self).unwrap()
            ))))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(ReflectType::Enum(Box::new(EnumVariant { enum_name: name.to_string(), variant_name: variant.to_string(), value: ReflectType::Unit() })))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
            Ok(ReflectType::Enum(Box::new(EnumVariant { enum_name: name.to_string(), variant_name: variant.to_string(), value: value.serialize(self).unwrap() })))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ReflectedStructSerializer { data: Some(ReflectType::Seq(Vec::new())) })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(ReflectedStructSerializer { data: Some(ReflectType::Tuple(Vec::new())) })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ReflectedStructSerializer { data: Some(ReflectType::Struct(name.to_string(), Vec::new())) })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl SerializeTuple for ReflectedStructSerializer {
    type_ok_err!(Ok, Error);
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
            T: Serialize {
                if let ReflectType::Tuple(fields) = self.data.as_mut().unwrap() {
                    fields.push(value.serialize(ReflectedStructSerializer { data: None }).unwrap());
                }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.data.unwrap())
    }
}

impl SerializeStruct for ReflectedStructSerializer {
    type_ok_err!(Ok, Error);
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
            T: Serialize {
        if let ReflectType::Struct(name, fields) = self.data.as_mut().unwrap() {
            fields.push(
                StructField {
                    name: key.to_string(),
                    value: value.serialize(ReflectedStructSerializer { data: None }).unwrap(),
                }
            )
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.data.unwrap())
    }
}

impl<'de> SerializeSeq for ReflectedStructSerializer {
    type_ok_err!(Ok, Error);
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
            T: Serialize {
                if let ReflectType::Seq(fields) = self.data.as_mut().unwrap() {
                    fields.push(value.serialize(ReflectedStructSerializer { data: None }).unwrap());
                }
                Ok(())
        
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.data.unwrap())
    }
}

impl<'de> Deserializer<'de> for ReflectType {
    type Error = serde::de::value::Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: Visitor<'de> {
                match self {
                    ReflectType::bool(v) => { visitor.visit_bool(v) }
                    ReflectType::i8(v) => { visitor.visit_i8(v) }
                    ReflectType::i16(v) => { visitor.visit_i16(v) }
                    ReflectType::i32(v) => { visitor.visit_i32(v) }
                    ReflectType::i64(v) => { visitor.visit_i64(v) }
                    ReflectType::u8(v) => { visitor.visit_u8(v) }
                    ReflectType::u16(v) => { visitor.visit_u16(v) }
                    ReflectType::u32(v) => { visitor.visit_u32(v) }
                    ReflectType::u64(v) => { visitor.visit_u64(v) }
                    ReflectType::f32(v) => { visitor.visit_f32(v) }
                    ReflectType::f64(v) => { visitor.visit_f64(v) }
                    ReflectType::char(v) => { visitor.visit_char(v) }
                    ReflectType::bytes(v) => { visitor.visit_bytes(&v) }
                    ReflectType::Option(v) => { 
                        if let Some(v_unwrap) = v {
                            visitor.visit_some(*v_unwrap)
                        } else {
                            visitor.visit_none()
                        }
                    }
                    ReflectType::Seq(v) => {
                        let seq: serde::de::value::SeqDeserializer<std::vec::IntoIter<ReflectType>, Self::Error> = serde::de::value::SeqDeserializer::new(v.into_iter());
                        visitor.visit_seq(seq)
                    }
                    ReflectType::Tuple(v) => {
                        let seq: serde::de::value::SeqDeserializer<std::vec::IntoIter<ReflectType>, Self::Error> = serde::de::value::SeqDeserializer::new(v.into_iter());
                        visitor.visit_seq(seq)
                    }
                    ReflectType::Enum(v) => {
                        let enums = [(v.variant_name, v.value)];
                        let seq = serde::de::value::MapDeserializer::new(enums.iter().map(|s|  (s.0.clone(), s.1.clone()) ));
                        let seq_b = serde::de::value::MapAccessDeserializer::new(seq);
                        visitor.visit_enum(seq_b)
                    }
                    ReflectType::String(v) => { visitor.visit_string(v) }
                    ReflectType::Struct(name, v) => {
                        let seq: serde::de::value::SeqDeserializer<std::slice::Iter<StructField>, Self::Error> = serde::de::value::SeqDeserializer::new(v.iter());
                        visitor.visit_seq(seq)
                        /*
                        let pairs: Vec<(String, ReflectType)> = v.iter().map(|v| (v.name, v.value)).collect();
                        let pairs = pairs.iter();
                        let seq: serde::de::value::MapDeserializer<std::slice::Iter<(String, ReflectType)>, Self::Error> = serde::de::value::MapDeserializer::new(pairs);
                        visitor.visit_bool(false)
                        */
                    }
                    ReflectType::Unit() => { visitor.visit_unit() }
                }
    }
    serde::forward_to_deserialize_any!(bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any);
}

impl<'de> serde::de::IntoDeserializer<'de> for &StructField {
    type Deserializer = ReflectType;
    fn into_deserializer(self) -> Self::Deserializer {
        self.value.clone()
    }
}

impl<'de> serde::de::IntoDeserializer<'de> for ReflectType {
    type Deserializer = ReflectType;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}