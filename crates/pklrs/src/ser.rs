use serde::ser::{self, Serialize};

use crate::error::Error;
use crate::value::{ObjectMember, PklValue};

/// Serialize any `serde::Serialize` type into a `PklValue`.
pub fn to_pkl_value<T: Serialize>(value: &T) -> Result<PklValue, Error> {
    value.serialize(PklSerializer)
}

struct PklSerializer;

impl ser::Serializer for PklSerializer {
    type Ok = PklValue;
    type Error = Error;

    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeSeq;
    type SerializeTupleStruct = SerializeSeq;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<PklValue, Error> {
        Ok(PklValue::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v as i64))
    }

    fn serialize_i16(self, v: i16) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v as i64))
    }

    fn serialize_i32(self, v: i32) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v as i64))
    }

    fn serialize_i64(self, v: i64) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v))
    }

    fn serialize_u8(self, v: u8) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v as i64))
    }

    fn serialize_u16(self, v: u16) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v as i64))
    }

    fn serialize_u32(self, v: u32) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<PklValue, Error> {
        Ok(PklValue::Int(v as i64))
    }

    fn serialize_f32(self, v: f32) -> Result<PklValue, Error> {
        Ok(PklValue::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<PklValue, Error> {
        Ok(PklValue::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<PklValue, Error> {
        Ok(PklValue::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<PklValue, Error> {
        Ok(PklValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<PklValue, Error> {
        Ok(PklValue::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<PklValue, Error> {
        Ok(PklValue::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<PklValue, Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<PklValue, Error> {
        Ok(PklValue::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<PklValue, Error> {
        Ok(PklValue::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<PklValue, Error> {
        Ok(PklValue::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<PklValue, Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<PklValue, Error> {
        let inner = value.serialize(PklSerializer)?;
        Ok(PklValue::Map(vec![(
            PklValue::String(variant.to_string()),
            inner,
        )]))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SerializeSeq, Error> {
        Ok(SerializeSeq {
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeSeq, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SerializeSeq, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeTupleVariant, Error> {
        Ok(SerializeTupleVariant {
            variant: variant.to_string(),
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<SerializeMap, Error> {
        Ok(SerializeMap {
            entries: Vec::with_capacity(len.unwrap_or(0)),
            pending_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SerializeStruct, Error> {
        Ok(SerializeStruct {
            members: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeStructVariant, Error> {
        Ok(SerializeStructVariant {
            variant: variant.to_string(),
            members: Vec::with_capacity(len),
        })
    }
}

// --- SerializeSeq / SerializeTuple ---

struct SerializeSeq {
    items: Vec<PklValue>,
}

impl ser::SerializeSeq for SerializeSeq {
    type Ok = PklValue;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.items.push(value.serialize(PklSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<PklValue, Error> {
        Ok(PklValue::List(self.items))
    }
}

impl ser::SerializeTuple for SerializeSeq {
    type Ok = PklValue;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<PklValue, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeSeq {
    type Ok = PklValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<PklValue, Error> {
        ser::SerializeSeq::end(self)
    }
}

// --- SerializeTupleVariant ---

struct SerializeTupleVariant {
    variant: String,
    items: Vec<PklValue>,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = PklValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.items.push(value.serialize(PklSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<PklValue, Error> {
        Ok(PklValue::Map(vec![(
            PklValue::String(self.variant),
            PklValue::List(self.items),
        )]))
    }
}

// --- SerializeMap ---

struct SerializeMap {
    entries: Vec<(PklValue, PklValue)>,
    pending_key: Option<PklValue>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = PklValue;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        self.pending_key = Some(key.serialize(PklSerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| Error::Deserialize("serialize_value called before serialize_key".into()))?;
        self.entries.push((key, value.serialize(PklSerializer)?));
        Ok(())
    }

    fn end(self) -> Result<PklValue, Error> {
        Ok(PklValue::Map(self.entries))
    }
}

// --- SerializeStruct ---

struct SerializeStruct {
    members: Vec<ObjectMember>,
}

impl ser::SerializeStruct for SerializeStruct {
    type Ok = PklValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.members.push(ObjectMember::Property {
            name: key.to_string(),
            value: value.serialize(PklSerializer)?,
        });
        Ok(())
    }

    fn end(self) -> Result<PklValue, Error> {
        Ok(PklValue::Object {
            class_name: String::new(),
            module_uri: String::new(),
            members: self.members,
        })
    }
}

// --- SerializeStructVariant ---

struct SerializeStructVariant {
    variant: String,
    members: Vec<ObjectMember>,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = PklValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.members.push(ObjectMember::Property {
            name: key.to_string(),
            value: value.serialize(PklSerializer)?,
        });
        Ok(())
    }

    fn end(self) -> Result<PklValue, Error> {
        let inner = PklValue::Object {
            class_name: String::new(),
            module_uri: String::new(),
            members: self.members,
        };
        Ok(PklValue::Map(vec![(
            PklValue::String(self.variant),
            inner,
        )]))
    }
}

impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Deserialize(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test_primitives() {
        assert_eq!(to_pkl_value(&true).unwrap(), PklValue::Bool(true));
        assert_eq!(to_pkl_value(&42i64).unwrap(), PklValue::Int(42));
        assert_eq!(to_pkl_value(&3.14f64).unwrap(), PklValue::Float(3.14));
        assert_eq!(
            to_pkl_value(&"hello").unwrap(),
            PklValue::String("hello".into())
        );
    }

    #[test]
    fn test_option() {
        assert_eq!(to_pkl_value(&None::<i64>).unwrap(), PklValue::Null);
        assert_eq!(to_pkl_value(&Some(42i64)).unwrap(), PklValue::Int(42));
    }

    #[test]
    fn test_vec() {
        let v = vec![1i64, 2, 3];
        assert_eq!(
            to_pkl_value(&v).unwrap(),
            PklValue::List(vec![PklValue::Int(1), PklValue::Int(2), PklValue::Int(3)])
        );
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Server {
            host: String,
            port: i64,
        }
        let s = Server {
            host: "localhost".into(),
            port: 8080,
        };
        let val = to_pkl_value(&s).unwrap();
        match val {
            PklValue::Object { members, .. } => {
                assert_eq!(members.len(), 2);
                assert!(matches!(&members[0], ObjectMember::Property { name, value }
                    if name == "host" && *value == PklValue::String("localhost".into())));
                assert!(matches!(&members[1], ObjectMember::Property { name, value }
                    if name == "port" && *value == PklValue::Int(8080)));
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn test_hashmap() {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert("a", 1i64);
        let val = to_pkl_value(&m).unwrap();
        match val {
            PklValue::Map(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, PklValue::String("a".into()));
                assert_eq!(entries[0].1, PklValue::Int(1));
            }
            _ => panic!("expected Map"),
        }
    }

    #[test]
    fn test_unit_enum() {
        #[derive(Serialize)]
        enum Color {
            Red,
            Green,
        }
        assert_eq!(
            to_pkl_value(&Color::Red).unwrap(),
            PklValue::String("Red".into())
        );
        assert_eq!(
            to_pkl_value(&Color::Green).unwrap(),
            PklValue::String("Green".into())
        );
    }

    #[test]
    fn test_roundtrip() {
        use crate::de::from_pkl_value;

        #[derive(Debug, Serialize, serde::Deserialize, PartialEq)]
        struct Config {
            name: String,
            port: i64,
            debug: bool,
            tags: Vec<String>,
        }

        let original = Config {
            name: "app".into(),
            port: 3000,
            debug: true,
            tags: vec!["web".into(), "api".into()],
        };

        let pkl = to_pkl_value(&original).unwrap();
        let restored: Config = from_pkl_value(&pkl).unwrap();
        assert_eq!(original, restored);
    }
}
