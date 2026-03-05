use std::collections::HashMap;

use crate::types::{DataSize, Duration, IntSeq, PklRegex};

/// A member of a Pkl object.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectMember {
    /// A named property: `name = value`
    Property { name: String, value: PklValue },
    /// A map entry: `[key] = value`
    Entry { key: PklValue, value: PklValue },
    /// An indexed element: `[index] = value`
    Element { index: usize, value: PklValue },
}

/// Decoded Pkl value tree.
#[derive(Debug, Clone, PartialEq)]
pub enum PklValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),

    /// Typed or Dynamic object.
    Object {
        class_name: String,
        module_uri: String,
        members: Vec<ObjectMember>,
    },

    /// Pkl `Map` or `Mapping`.
    Map(Vec<(PklValue, PklValue)>),

    /// Pkl `List` or `Listing`.
    List(Vec<PklValue>),

    /// Pkl `Set`.
    Set(Vec<PklValue>),

    Duration(Duration),
    DataSize(DataSize),
    Pair(Box<PklValue>, Box<PklValue>),
    IntSeq(IntSeq),
    Regex(PklRegex),

    /// Pkl `Class` reference.
    Class {
        class_name: String,
        module_uri: String,
    },

    /// Pkl `TypeAlias` reference.
    TypeAlias {
        name: String,
        module_uri: String,
    },

    /// Pkl functions cannot be materialized on the client side.
    Function,

    /// Raw bytes.
    Bytes(Vec<u8>),
}

impl PklValue {
    /// Get the properties of an Object as a HashMap.
    pub fn as_properties(&self) -> Option<HashMap<&str, &PklValue>> {
        match self {
            PklValue::Object { members, .. } => {
                let mut map = HashMap::new();
                for member in members {
                    if let ObjectMember::Property { name, value } = member {
                        map.insert(name.as_str(), value);
                    }
                }
                Some(map)
            }
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, PklValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PklValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PklValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PklValue::Float(f) => Some(*f),
            PklValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PklValue::String(s) => Some(s),
            _ => None,
        }
    }
}
