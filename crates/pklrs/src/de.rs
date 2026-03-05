use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;

use crate::error::Error;
use crate::value::{ObjectMember, PklValue};

/// A serde Deserializer for PklValue.
pub struct PklDeserializer<'de> {
    value: &'de PklValue,
}

impl<'de> PklDeserializer<'de> {
    pub fn new(value: &'de PklValue) -> Self {
        Self { value }
    }
}

/// Deserialize a PklValue into any type that implements serde::Deserialize.
pub fn from_pkl_value<'de, T: serde::Deserialize<'de>>(value: &'de PklValue) -> Result<T, Error> {
    let deserializer = PklDeserializer::new(value);
    T::deserialize(deserializer)
}

impl<'de> serde::Deserializer<'de> for PklDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            PklValue::Null => visitor.visit_unit(),
            PklValue::Bool(b) => visitor.visit_bool(*b),
            PklValue::Int(i) => visitor.visit_i64(*i),
            PklValue::Float(f) => visitor.visit_f64(*f),
            PklValue::String(s) => visitor.visit_borrowed_str(unsafe {
                // Safety: the PklValue lives for 'de
                std::mem::transmute::<&str, &'de str>(s.as_str())
            }),
            PklValue::Object { members, .. } => {
                let map = ObjectMapAccess::new(members);
                visitor.visit_map(map)
            }
            PklValue::Map(entries) => {
                let map = PklMapAccess::new(entries);
                visitor.visit_map(map)
            }
            PklValue::List(items) | PklValue::Set(items) => {
                let seq = PklSeqAccess::new(items);
                visitor.visit_seq(seq)
            }
            PklValue::Duration(d) => {
                // Serialize as a map { "value": f64, "unit": string }
                let map = DurationMapAccess::new(d);
                visitor.visit_map(map)
            }
            PklValue::DataSize(d) => {
                let map = DataSizeMapAccess::new(d);
                visitor.visit_map(map)
            }
            PklValue::Pair(first, second) => {
                let seq = PairSeqAccess::new(first, second);
                visitor.visit_seq(seq)
            }
            PklValue::IntSeq(seq) => {
                let map = IntSeqMapAccess::new(seq);
                visitor.visit_map(map)
            }
            PklValue::Regex(r) => visitor.visit_borrowed_str(unsafe {
                std::mem::transmute::<&str, &'de str>(r.pattern.as_str())
            }),
            PklValue::Class { .. } | PklValue::TypeAlias { .. } | PklValue::Function => {
                visitor.visit_unit()
            }
            PklValue::Bytes(b) => visitor.visit_bytes(b),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            PklValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.value {
            PklValue::String(s) => visitor.visit_enum(s.as_str().into_deserializer()),
            _ => Err(Error::Deserialize(
                "expected string for enum variant".into(),
            )),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.value {
            PklValue::Object { members, .. } => {
                let map = ObjectMapAccess::new(members);
                visitor.visit_map(map)
            }
            PklValue::Map(entries) => {
                let map = PklMapAccess::new(entries);
                visitor.visit_map(map)
            }
            PklValue::Duration(d) => {
                let map = DurationMapAccess::new(d);
                visitor.visit_map(map)
            }
            PklValue::DataSize(d) => {
                let map = DataSizeMapAccess::new(d);
                visitor.visit_map(map)
            }
            PklValue::IntSeq(seq) => {
                let map = IntSeqMapAccess::new(seq);
                visitor.visit_map(map)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_map<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }

    fn deserialize_seq<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            PklValue::Null => visitor.visit_unit(),
            _ => Err(Error::Deserialize("expected null".into())),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            PklValue::Bool(b) => visitor.visit_bool(*b),
            _ => Err(Error::Deserialize("expected bool".into())),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            PklValue::String(s) => visitor.visit_string(s.clone()),
            _ => Err(Error::Deserialize("expected string".into())),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_string(visitor)
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char bytes byte_buf
        tuple tuple_struct identifier ignored_any
    }
}

// --- MapAccess for Object properties ---

struct ObjectMapAccess<'de> {
    members: &'de [ObjectMember],
    index: usize,
}

impl<'de> ObjectMapAccess<'de> {
    fn new(members: &'de [ObjectMember]) -> Self {
        Self { members, index: 0 }
    }
}

impl<'de> MapAccess<'de> for ObjectMapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        // Skip non-property members for struct deserialization
        while self.index < self.members.len() {
            if let ObjectMember::Property { .. } = &self.members[self.index] {
                break;
            }
            self.index += 1;
        }

        if self.index >= self.members.len() {
            return Ok(None);
        }

        if let ObjectMember::Property { name, .. } = &self.members[self.index] {
            seed.deserialize(serde::de::value::BorrowedStrDeserializer::new(unsafe {
                std::mem::transmute::<&str, &'de str>(name.as_str())
            }))
            .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        if let ObjectMember::Property { value, .. } = &self.members[self.index] {
            self.index += 1;
            seed.deserialize(PklDeserializer::new(value))
        } else {
            self.index += 1;
            Err(Error::Deserialize("expected property member".into()))
        }
    }
}

// --- MapAccess for Pkl Map ---

struct PklMapAccess<'de> {
    entries: &'de [(PklValue, PklValue)],
    index: usize,
}

impl<'de> PklMapAccess<'de> {
    fn new(entries: &'de [(PklValue, PklValue)]) -> Self {
        Self { entries, index: 0 }
    }
}

impl<'de> MapAccess<'de> for PklMapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        if self.index >= self.entries.len() {
            return Ok(None);
        }
        let (key, _) = &self.entries[self.index];
        seed.deserialize(PklDeserializer::new(key)).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let (_, value) = &self.entries[self.index];
        self.index += 1;
        seed.deserialize(PklDeserializer::new(value))
    }
}

// --- SeqAccess for List/Set ---

struct PklSeqAccess<'de> {
    items: &'de [PklValue],
    index: usize,
}

impl<'de> PklSeqAccess<'de> {
    fn new(items: &'de [PklValue]) -> Self {
        Self { items, index: 0 }
    }
}

impl<'de> SeqAccess<'de> for PklSeqAccess<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        if self.index >= self.items.len() {
            return Ok(None);
        }
        let item = &self.items[self.index];
        self.index += 1;
        seed.deserialize(PklDeserializer::new(item)).map(Some)
    }
}

// --- SeqAccess for Pair ---

struct PairSeqAccess<'de> {
    first: &'de PklValue,
    second: &'de PklValue,
    index: usize,
}

impl<'de> PairSeqAccess<'de> {
    fn new(first: &'de PklValue, second: &'de PklValue) -> Self {
        Self {
            first,
            second,
            index: 0,
        }
    }
}

impl<'de> SeqAccess<'de> for PairSeqAccess<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        match self.index {
            0 => {
                self.index += 1;
                seed.deserialize(PklDeserializer::new(self.first)).map(Some)
            }
            1 => {
                self.index += 1;
                seed.deserialize(PklDeserializer::new(self.second))
                    .map(Some)
            }
            _ => Ok(None),
        }
    }
}

// --- MapAccess for Duration ---

struct DurationMapAccess<'de> {
    duration: &'de crate::types::Duration,
    state: u8, // 0=value_key, 1=value_val, 2=unit_key, 3=unit_val, 4=done
}

impl<'de> DurationMapAccess<'de> {
    fn new(duration: &'de crate::types::Duration) -> Self {
        Self {
            duration,
            state: 0,
        }
    }
}

impl<'de> MapAccess<'de> for DurationMapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.state {
            0 => {
                self.state = 1;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new("value"))
                    .map(Some)
            }
            2 => {
                self.state = 3;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new("unit"))
                    .map(Some)
            }
            _ => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        match self.state {
            1 => {
                self.state = 2;
                seed.deserialize(serde::de::value::F64Deserializer::new(self.duration.value))
            }
            3 => {
                self.state = 4;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new(
                    self.duration.unit.as_str(),
                ))
            }
            _ => Err(Error::Deserialize("unexpected state".into())),
        }
    }
}

// --- MapAccess for DataSize ---

struct DataSizeMapAccess<'de> {
    data_size: &'de crate::types::DataSize,
    state: u8,
}

impl<'de> DataSizeMapAccess<'de> {
    fn new(data_size: &'de crate::types::DataSize) -> Self {
        Self {
            data_size,
            state: 0,
        }
    }
}

impl<'de> MapAccess<'de> for DataSizeMapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.state {
            0 => {
                self.state = 1;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new("value"))
                    .map(Some)
            }
            2 => {
                self.state = 3;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new("unit"))
                    .map(Some)
            }
            _ => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        match self.state {
            1 => {
                self.state = 2;
                seed.deserialize(serde::de::value::F64Deserializer::new(
                    self.data_size.value,
                ))
            }
            3 => {
                self.state = 4;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new(
                    self.data_size.unit.as_str(),
                ))
            }
            _ => Err(Error::Deserialize("unexpected state".into())),
        }
    }
}

// --- MapAccess for IntSeq ---

struct IntSeqMapAccess<'de> {
    int_seq: &'de crate::types::IntSeq,
    state: u8,
}

impl<'de> IntSeqMapAccess<'de> {
    fn new(int_seq: &'de crate::types::IntSeq) -> Self {
        Self { int_seq, state: 0 }
    }
}

impl<'de> MapAccess<'de> for IntSeqMapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.state {
            0 => {
                self.state = 1;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new("start"))
                    .map(Some)
            }
            2 => {
                self.state = 3;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new("end"))
                    .map(Some)
            }
            4 => {
                self.state = 5;
                seed.deserialize(serde::de::value::BorrowedStrDeserializer::new("step"))
                    .map(Some)
            }
            _ => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        match self.state {
            1 => {
                self.state = 2;
                seed.deserialize(serde::de::value::I64Deserializer::new(self.int_seq.start))
            }
            3 => {
                self.state = 4;
                seed.deserialize(serde::de::value::I64Deserializer::new(self.int_seq.end))
            }
            5 => {
                self.state = 6;
                seed.deserialize(serde::de::value::I64Deserializer::new(self.int_seq.step))
            }
            _ => Err(Error::Deserialize("unexpected state".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ObjectMember;

    #[test]
    fn test_deserialize_primitives() {
        assert_eq!(from_pkl_value::<bool>(&PklValue::Bool(true)).unwrap(), true);
        assert_eq!(from_pkl_value::<i64>(&PklValue::Int(42)).unwrap(), 42);
        assert_eq!(
            from_pkl_value::<f64>(&PklValue::Float(3.14)).unwrap(),
            3.14
        );
        assert_eq!(
            from_pkl_value::<String>(&PklValue::String("hello".into())).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_deserialize_option() {
        assert_eq!(
            from_pkl_value::<Option<i64>>(&PklValue::Null).unwrap(),
            None
        );
        assert_eq!(
            from_pkl_value::<Option<i64>>(&PklValue::Int(42)).unwrap(),
            Some(42)
        );
    }

    #[test]
    fn test_deserialize_list() {
        let list = PklValue::List(vec![PklValue::Int(1), PklValue::Int(2), PklValue::Int(3)]);
        let result: Vec<i64> = from_pkl_value(&list).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_deserialize_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Config {
            name: String,
            port: i64,
        }

        let obj = PklValue::Object {
            class_name: "Config".into(),
            module_uri: "file:///test.pkl".into(),
            members: vec![
                ObjectMember::Property {
                    name: "name".into(),
                    value: PklValue::String("test".into()),
                },
                ObjectMember::Property {
                    name: "port".into(),
                    value: PklValue::Int(8080),
                },
            ],
        };

        let result: Config = from_pkl_value(&obj).unwrap();
        assert_eq!(
            result,
            Config {
                name: "test".into(),
                port: 8080
            }
        );
    }

    #[test]
    fn test_deserialize_map() {
        use std::collections::HashMap;

        let map = PklValue::Map(vec![
            (PklValue::String("a".into()), PklValue::Int(1)),
            (PklValue::String("b".into()), PklValue::Int(2)),
        ]);
        let result: HashMap<String, i64> = from_pkl_value(&map).unwrap();
        assert_eq!(result.get("a"), Some(&1));
        assert_eq!(result.get("b"), Some(&2));
    }
}
