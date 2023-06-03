#[cfg(feature = "egui_impl")]
pub mod egui_impl;
#[cfg(feature = "imgui_impl")]
pub mod imgui_impl;

pub use reflect_derive::Reflect;
use crate as reflect;
use serde::de::{Deserializer, Visitor};
use std::any::TypeId;
use std::sync::Arc;

pub trait Reflect {
    fn to_value(&self) -> Value;
    fn to_type() -> Type;
}

#[derive(Clone, Debug, PartialEq, Eq, Reflect)]
pub struct SeqType {
    pub id: TypeId,
    pub seq_type: TypeGet,
    pub fixed_size: Option<usize>,
}

impl std::hash::Hash for SeqType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct SeqValue {
    pub info: SeqType,
    pub values: Vec<Value>
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct OptionType {
    pub some: TypeGet,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct OptionValue {
    pub info: OptionType,
    pub value: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Reflect)]
pub struct EnumType {
    pub id: TypeId,
    pub enum_name: &'static str,
    pub variants: Vec<EnumVariant>
}

impl std::hash::Hash for EnumType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct EnumValue {
    pub info: EnumType,
    pub variant: EnumVariantValue
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct EnumVariant {
    pub variant_name: &'static str,
    pub variant_type: TypeGet,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct EnumVariantValue {
    pub info: EnumVariant,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Reflect)]
pub struct StructType {
    pub id: TypeId,
    pub name: &'static str,
    pub fields: Vec<StructField>,
}

impl std::hash::Hash for StructType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct StructValue {
    pub info: StructType,
    pub fields: Vec<StructFieldValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct StructField {
    pub name: &'static str,
    pub field_type: TypeGet,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct StructFieldValue {
    pub info: StructField,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum Type {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    Bytes,
    Option(Box<TypeGet>),
    Tuple(Vec<TypeGet>),
    Seq(SeqType),
    Enum(Box<EnumType>),
    String,
    Struct(StructType),
    Fn,
    Unit,
}

impl Type {
    pub fn from<T: Reflect>() -> Type {
        T::to_type()
    }

    pub fn get_type_id(&self) -> Option<TypeId> {
        match self {
            Type::Bool => Some(TypeId::of::<bool>()),
            Type::I8 => Some(TypeId::of::<i8>()),
            Type::I16 => Some(TypeId::of::<i16>()),
            Type::I32 => Some(TypeId::of::<i32>()),
            Type::I64 => Some(TypeId::of::<i64>()),
            Type::U8 => Some(TypeId::of::<u8>()),
            Type::U16 => Some(TypeId::of::<u16>()),
            Type::U32 => Some(TypeId::of::<u32>()),
            Type::U64 => Some(TypeId::of::<u64>()),
            Type::F32 => Some(TypeId::of::<f32>()),
            Type::F64 => Some(TypeId::of::<f64>()),
            Type::Char => Some(TypeId::of::<char>()),
            Type::Bytes => Some(TypeId::of::<Vec<u8>>()),
            Type::Option(_type_get) => None,
            Type::Tuple(_array_type) => None,
            Type::Seq(ty) => Some(ty.id),
            Type::Enum(enum_type) => { Some(enum_type.id) },
            Type::String => Some(TypeId::of::<String>()),
            Type::Struct(struct_type) => { Some(struct_type.id) },
            Type::Fn => None,
            Type::Unit => Some(TypeId::of::<()>()),
        }
    }

    pub fn default_value(&self) -> Option<Value> {
        match self {
            Type::Bool => Some(Value::Bool(false)),
            Type::I8 => Some(Value::I8(0)),
            Type::I16 => Some(Value::I16(0)),
            Type::I32 => Some(Value::I32(0)),
            Type::I64 => Some(Value::I64(0)),
            Type::U8 => Some(Value::U8(0)),
            Type::U16 => Some(Value::U16(0)),
            Type::U32 => Some(Value::U32(0)),
            Type::U64 => Some(Value::U64(0)),
            Type::F32 => Some(Value::F32(0.0)),
            Type::F64 => Some(Value::F64(0.0)),
            Type::Char => Some(Value::Char('0')),
            Type::Bytes => Some(Value::Bytes(vec![0, 0, 0, 0])),
            Type::Option(type_get) => Some(Value::Option(Box::new(OptionValue { info: OptionType{ some: *type_get.clone() }, value: None }))),
            Type::Tuple(array_type) => Some(Value::Tuple(array_type.iter().map(|ty| ty.get().default_value().unwrap()).collect())),
            Type::Seq(ty) => Some(Value::Seq(SeqValue { info: ty.clone(), values: ty.fixed_size.map_or(vec![], |size| vec![ty.seq_type.get().default_value().unwrap();size] ) })),
            Type::Enum(enum_type) => {
                let variant = enum_type.variants.first().unwrap();
                Some(Value::Enum(
                    Box::new(EnumValue {
                        info: *enum_type.clone(),
                        variant: EnumVariantValue {
                            info: variant.clone(),
                            value: variant.variant_type.get().default_value().unwrap()
                        }
                    })
                ))
            },
            Type::String => Some(Value::String(String::new())),
            Type::Struct(struct_type) => {
                let mut struct_value = StructValue {
                    info: struct_type.clone(),
                    fields: Vec::new(),
                };
                for field_type in struct_type.fields.iter() {
                    struct_value.fields.push(StructFieldValue {
                        info: field_type.clone(),
                        value: field_type.field_type.get().default_value().unwrap()
                    });
                }

                Some(Value::Struct(struct_value))
            },
            Type::Fn => Some(Value::Fn),
            Type::Unit => Some(Value::Unit),
        }
    }
}
#[derive(Clone)]
pub struct TypeGet {
    inner: Arc<dyn Fn() -> Type>,
}

impl TypeGet {
    pub fn get(&self) -> Type {
        (self.inner)()
    }

    pub fn from_type(_type: Type) -> TypeGet {
        TypeGet {
            inner: std::sync::Arc::new( move || { _type.clone() })
        }
    }

    pub fn from<T: Reflect>() -> TypeGet {
        TypeGet {
            inner: std::sync::Arc::new( || { T::to_type() })
        }
    }
}

impl std::fmt::Debug for TypeGet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TypeGet").unwrap();
        Ok(())
    }
}

impl PartialEq for TypeGet {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.inner) == Arc::as_ptr(&other.inner)
    }
}
impl Eq for TypeGet {}

impl std::hash::Hash for TypeGet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl Reflect for TypeGet {
    fn to_value(&self) -> Value {
        Value::Fn
    }
    fn to_type() -> Type {
        Type::Fn
    }
}


#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum Value {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Bytes(Vec<u8>),
    Option(Box<OptionValue>),
    Tuple(Vec<Value>),
    Seq(SeqValue),
    Enum(Box<EnumValue>),
    String(String),
    Struct(StructValue),
    Fn,
    Unit,
}

impl<'de> Deserializer<'de> for Value {
    type Error = serde::de::value::Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
            V: Visitor<'de> {
                match self {
                    Value::Bool(v) => { visitor.visit_bool(v) }
                    Value::I8(v) => { visitor.visit_i8(v) }
                    Value::I16(v) => { visitor.visit_i16(v) }
                    Value::I32(v) => { visitor.visit_i32(v) }
                    Value::I64(v) => { visitor.visit_i64(v) }
                    Value::U8(v) => { visitor.visit_u8(v) }
                    Value::U16(v) => { visitor.visit_u16(v) }
                    Value::U32(v) => { visitor.visit_u32(v) }
                    Value::U64(v) => { visitor.visit_u64(v) }
                    Value::F32(v) => { visitor.visit_f32(v) }
                    Value::F64(v) => { visitor.visit_f64(v) }
                    Value::Char(v) => { visitor.visit_char(v) }
                    Value::Bytes(v) => { visitor.visit_bytes(&v) }
                    Value::Option(v) => { 
                        if let Some(v_unwrap) = v.value {
                            visitor.visit_some(v_unwrap)
                        } else {
                            visitor.visit_none()
                        }
                    }
                    Value::Seq(v) => {
                        let seq: serde::de::value::SeqDeserializer<std::vec::IntoIter<Value>, Self::Error> = serde::de::value::SeqDeserializer::new(v.values.into_iter());
                        visitor.visit_seq(seq)
                    }
                    Value::Tuple(v) => {
                        let seq: serde::de::value::SeqDeserializer<std::vec::IntoIter<Value>, Self::Error> = serde::de::value::SeqDeserializer::new(v.into_iter());
                        visitor.visit_seq(seq)
                    }
                    Value::Enum(v) => {
                        let enums = [(v.variant.info.variant_name, v.variant.value)];
                        let seq = serde::de::value::MapDeserializer::new(enums.iter().map(|s|  (s.0.clone(), s.1.clone()) ));
                        let seq_b = serde::de::value::MapAccessDeserializer::new(seq);
                        visitor.visit_enum(seq_b)
                    }
                    Value::String(v) => { visitor.visit_string(v) }
                    Value::Struct(struct_type) => {
                        let seq: serde::de::value::SeqDeserializer<std::slice::Iter<StructFieldValue>, Self::Error> = serde::de::value::SeqDeserializer::new(struct_type.fields.iter());
                        visitor.visit_seq(seq)
                    }
                    Value::Fn => { visitor.visit_unit() }
                    Value::Unit => { visitor.visit_unit() }
                }
    }
    serde::forward_to_deserialize_any!(bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any);
}

impl<'de> serde::de::IntoDeserializer<'de> for EnumVariantValue {
    type Deserializer = Value;
    fn into_deserializer(self) -> Self::Deserializer {
        self.value.clone()
    }
}

impl<'de> serde::de::IntoDeserializer<'de> for &StructFieldValue {
    type Deserializer = Value;
    fn into_deserializer(self) -> Self::Deserializer {
        self.value.clone()
    }
}

impl<'de> serde::de::IntoDeserializer<'de> for Value {
    type Deserializer = Value;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}



















macro_rules! impl_reflect_primitive {
    ($primitive: ty, $value: ident) => {
        impl Reflect for $primitive {
            fn to_value(&self) -> Value {
                Value::$value(*self)
            }
            fn to_type() -> Type {
                Type::$value
            }
        }
    };
}

macro_rules! impl_reflect_array {
    ($array: ty, $len: tt) => {
        impl<T: Reflect + 'static> Reflect for $array {
            fn to_value(&self) -> Value {
                let mut values = Vec::new();
                for value in self.iter() {
                    values.push(value.to_value());
                }
                Value::Seq(
                    SeqValue {
                        info: SeqType {
                            id: TypeId::of::<T>(),
                            seq_type: TypeGet::from::<T>(),
                            fixed_size: Some($len),
                        },
                        values: values
                    }
                )
            }
            fn to_type() -> Type {
                Type::Seq(SeqType {
                    id: TypeId::of::<T>(),
                    seq_type: TypeGet::from::<T>(),
                    fixed_size: Some($len),
                })
            }
        }
    };
}

macro_rules! impl_reflect_tuple {
    (($($tuple: ident), *), ($($number: tt), *)) => {
        impl<$($tuple: Reflect),*> Reflect for ($($tuple), *) {
            fn to_value(&self) -> Value {
                Value::Tuple(
                    vec![
                        $(self.$number.to_value()),*
                    ]
                )
            }
            fn to_type() -> Type {
                Type::Tuple(
                    vec![
                        $(TypeGet::from::<$tuple>()),*
                    ]
                )
            }
        }
    };
}


impl_reflect_primitive!(bool, Bool);

impl_reflect_primitive!(i8, I8);
impl_reflect_primitive!(i16, I16);
impl_reflect_primitive!(i32, I32);
impl_reflect_primitive!(i64, I64);

impl_reflect_primitive!(u8, U8);
impl_reflect_primitive!(u16, U16);
impl_reflect_primitive!(u32, U32);
impl_reflect_primitive!(u64, U64);


impl_reflect_primitive!(f32, F32);
impl_reflect_primitive!(f64, F64);

impl_reflect_primitive!(char, Char);


impl_reflect_array!([T;1], 1);
impl_reflect_array!([T;2], 2);
impl_reflect_array!([T;3], 3);
impl_reflect_array!([T;4], 4);

impl_reflect_tuple!((T, U), (0, 1));
impl_reflect_tuple!((T, U, V), (0, 1, 2));

impl Reflect for isize {
    fn to_value(&self) -> Value {
        Value::I64(*self as i64)
    }
    fn to_type() -> Type {
        Type::I64
    }
}

impl Reflect for usize {
    fn to_value(&self) -> Value {
        Value::U64(*self as u64)
    }
    fn to_type() -> Type {
        Type::U64
    }
}

impl<T: Reflect> Reflect for Option<T> {
    fn to_value(&self) -> Value {
        Value::Option(
            Box::new(OptionValue {
                info: OptionType {
                    some: TypeGet::from::<T>(),
                },
                value: self.as_ref().map(|o| (o.to_value())),
            })
        )
    }
    fn to_type() -> Type {
        Type::Option(
            Box::new(TypeGet::from::<T>())
        )
    }
}

impl Reflect for &str {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
    fn to_type() -> Type {
        Type::String
    }
}

impl Reflect for String {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
    fn to_type() -> Type {
        Type::String
    }
}

impl Reflect for std::any::TypeId {
    fn to_value(&self) -> Value {
        Value::U64( unsafe { std::mem::transmute(self) } )
    }
    fn to_type() -> Type {
        Type::U64
    }
}

impl<T: Reflect> Reflect for &T {
    fn to_value(&self) -> Value {
        (*self).to_value()
    }
    fn to_type() -> Type {
        T::to_type()
    }
}

impl<T: Reflect> Reflect for Box<T> {
    fn to_value(&self) -> Value {
        (**self).to_value()
    }
    fn to_type() -> Type {
        T::to_type()
    }
}

impl<T: Reflect + 'static> Reflect for Vec<T> {
    fn to_value(&self) -> Value {
        let mut values = Vec::new();
        for value in self.iter() {
            values.push(value.to_value());
        }
        Value::Seq(
            SeqValue {
                info: SeqType {
                    id: TypeId::of::<Vec<T>>(),
                    seq_type: TypeGet::from::<T>(),
                    fixed_size: None,
                },
                values: values
            }
        )
    }
    fn to_type() -> Type {
        Type::Seq(SeqType {
            id: TypeId::of::<Vec<T>>(),
            seq_type: TypeGet::from::<T>(),
            fixed_size: None,
        })
    }
}


impl<T: Reflect + 'static> Reflect for std::ops::Range<T> {
    fn to_value(&self) -> Value {
        let ty = Self::to_type();
        if let Type::Struct(struct_type) = ty {
            Value::Struct(
                StructValue {
                    info: struct_type.clone(),
                    fields: vec![
                        StructFieldValue {
                            info: struct_type.fields[0].clone(),
                            value: self.start.to_value(),
                        },
                        StructFieldValue {
                            info: struct_type.fields[1].clone(),
                            value: self.end.to_value(),
                        }
                    ]
                }
            )
        } else {
            panic!()
        }
    }
    fn to_type() -> Type {
        Type::Struct(StructType {
            id: std::any::TypeId::of::<std::ops::Range<T>>(),
            name: "Range",
            fields: vec![
                StructField {
                    name: "start",
                    field_type: TypeGet::from_type(T::to_type()),
                },
                
                StructField {
                    name: "end",
                    field_type: TypeGet::from_type(T::to_type()),
                }
            ],
        })
    }
}


impl<T: Reflect + 'static> Reflect for std::ops::RangeInclusive<T> {
    fn to_value(&self) -> Value {
        let ty = Self::to_type();
        if let Type::Struct(struct_type) = ty {
            Value::Struct(
                StructValue {
                    info: struct_type.clone(),
                    fields: vec![
                        StructFieldValue {
                            info: struct_type.fields[0].clone(),
                            value: self.start().to_value(),
                        },
                        StructFieldValue {
                            info: struct_type.fields[1].clone(),
                            value: self.end().to_value(),
                        }
                    ]
                }
            )
        } else {
            panic!()
        }
    }
    fn to_type() -> Type {
        Type::Struct(StructType {
            id: std::any::TypeId::of::<std::ops::RangeInclusive<T>>(),
            name: "Range",
            fields: vec![
                StructField {
                    name: "start",
                    field_type: TypeGet::from_type(T::to_type()),
                },
                
                StructField {
                    name: "end",
                    field_type: TypeGet::from_type(T::to_type()),
                }
            ],
        })
    }
}

macro_rules! _impl_reflect_struct_2_fields_1_generic_struct {
    ($struct: ty, $name_struct: expr, $field_type: ty, $field_1_name: expr, $field_2_name: expr) => {
        StructType {
            id: std::any::TypeId::of::<$struct>(),
            name: $name_struct,
            fields: vec![
                StructField {
                    name: $field_1_name,
                    field_type: TypeGet::from::<$field_type>()
                },
                StructField {
                    name: $field_2_name,
                    field_type: TypeGet::from::<$field_type>()
                }
            ]
        }
    };
}

macro_rules! impl_reflect_struct_2_fields_1_generic {
    ($struct: ty, $name_struct: expr, $field_1: ident, $field_2: ident, $field_1_name: expr, $field_2_name: expr) => {
        impl<T: Reflect + 'static> Reflect for $struct {
            fn to_value(&self) -> Value {
                let struct_type = _impl_reflect_struct_2_fields_1_generic_struct!($struct, $name_struct, T, $field_1_name, $field_2_name);
                Value::Struct(
                    StructValue {
                        info: struct_type.clone(),
                        fields: vec![
                            StructFieldValue {
                                info: struct_type.fields[0].clone(),
                                value: self.$field_1.to_value(),
                            },
                            StructFieldValue {
                                info: struct_type.fields[1].clone(),
                                value: self.$field_2.to_value(),
                            }
                        ]
                    }
                )
            }
            fn to_type() -> Type {
                Type::Struct(_impl_reflect_struct_2_fields_1_generic_struct!($struct, $name_struct, T, $field_1_name, $field_2_name))
            }
        }
    };
}

impl_reflect_struct_2_fields_1_generic!(euclid::default::Point2D<T>, "Point2D", x, y, "x", "y");
impl_reflect_struct_2_fields_1_generic!(euclid::default::Vector2D<T>, "Vector2D", x, y, "x", "y");
impl_reflect_struct_2_fields_1_generic!(euclid::default::Size2D<T>, "Size2D", width, height, "width", "height");