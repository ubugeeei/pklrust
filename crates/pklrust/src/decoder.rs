use std::io::Cursor;

use rmpv::Value;

use crate::error::{Error, Result};
use crate::types::*;
use crate::value::*;

/// Type codes for pkl-binary encoded values.
const TYPE_OBJECT: u8 = 0x01;
const TYPE_MAP: u8 = 0x02;
const TYPE_MAPPING: u8 = 0x03;
const TYPE_LIST: u8 = 0x04;
const TYPE_LISTING: u8 = 0x05;
const TYPE_SET: u8 = 0x06;
const TYPE_DURATION: u8 = 0x07;
const TYPE_DATA_SIZE: u8 = 0x08;
const TYPE_PAIR: u8 = 0x09;
const TYPE_INT_SEQ: u8 = 0x0A;
const TYPE_REGEX: u8 = 0x0B;
const TYPE_CLASS: u8 = 0x0C;
const TYPE_TYPE_ALIAS: u8 = 0x0D;
#[allow(dead_code)]
const TYPE_FUNCTION: u8 = 0x0E;
const TYPE_BYTES: u8 = 0x0F;

/// Member codes within an Object's members array.
const MEMBER_PROPERTY: u8 = 0x10;
const MEMBER_ENTRY: u8 = 0x11;
const MEMBER_ELEMENT: u8 = 0x12;

/// Decode pkl-binary bytes (from EvaluateResponse.result) into a PklValue.
pub fn decode_pkl_binary(bytes: &[u8]) -> Result<PklValue> {
    let mut cursor = Cursor::new(bytes);
    let value = rmpv::decode::read_value(&mut cursor)?;
    decode_pkl_value(&value)
}

/// Decode an rmpv Value (already parsed from MessagePack) into a PklValue.
pub fn decode_pkl_value(v: &Value) -> Result<PklValue> {
    match v {
        Value::Nil => Ok(PklValue::Null),
        Value::Boolean(b) => Ok(PklValue::Bool(*b)),
        Value::Integer(i) => {
            if let Some(n) = i.as_i64() {
                Ok(PklValue::Int(n))
            } else if let Some(n) = i.as_u64() {
                Ok(PklValue::Int(n as i64))
            } else {
                Err(Error::Decode("integer out of range".into()))
            }
        }
        Value::F32(f) => Ok(PklValue::Float(*f as f64)),
        Value::F64(f) => Ok(PklValue::Float(*f)),
        Value::String(s) => {
            let s = s
                .as_str()
                .ok_or_else(|| Error::Decode("invalid UTF-8 string".into()))?;
            Ok(PklValue::String(s.to_string()))
        }
        Value::Binary(b) => Ok(PklValue::Bytes(b.clone())),
        Value::Array(arr) if !arr.is_empty() => {
            // Check if this is a typed pkl value (first element is a type code integer)
            if let Some(code) = arr[0].as_u64() {
                let code = code as u8;
                decode_typed_value(code, &arr[1..])
            } else {
                // Plain array (shouldn't normally appear in pkl-binary, but handle gracefully)
                let items: Result<Vec<PklValue>> = arr.iter().map(decode_pkl_value).collect();
                Ok(PklValue::List(items?))
            }
        }
        Value::Array(_) => Ok(PklValue::List(vec![])),
        Value::Map(entries) => {
            let mut pairs = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                pairs.push((decode_pkl_value(k)?, decode_pkl_value(v)?));
            }
            Ok(PklValue::Map(pairs))
        }
        Value::Ext(_, _) => Err(Error::Decode("unexpected ext type".into())),
    }
}

fn decode_typed_value(code: u8, slots: &[Value]) -> Result<PklValue> {
    match code {
        TYPE_OBJECT => decode_object(slots),
        TYPE_MAP | TYPE_MAPPING => decode_map(slots),
        TYPE_LIST | TYPE_LISTING => decode_list(slots),
        TYPE_SET => decode_set(slots),
        TYPE_DURATION => decode_duration(slots),
        TYPE_DATA_SIZE => decode_data_size(slots),
        TYPE_PAIR => decode_pair(slots),
        TYPE_INT_SEQ => decode_int_seq(slots),
        TYPE_REGEX => decode_regex(slots),
        TYPE_CLASS => decode_class(slots),
        TYPE_TYPE_ALIAS => decode_type_alias(slots),
        TYPE_BYTES => decode_bytes(slots),
        0x0E => Ok(PklValue::Function),
        _ => Err(Error::UnknownTypeCode(code)),
    }
}

fn decode_object(slots: &[Value]) -> Result<PklValue> {
    if slots.len() < 3 {
        return Err(Error::Decode("object needs at least 3 slots".into()));
    }
    let class_name = slots[0]
        .as_str()
        .ok_or_else(|| Error::Decode("object class_name must be string".into()))?
        .to_string();
    let module_uri = slots[1]
        .as_str()
        .ok_or_else(|| Error::Decode("object module_uri must be string".into()))?
        .to_string();
    let members_arr = slots[2]
        .as_array()
        .ok_or_else(|| Error::Decode("object members must be array".into()))?;

    let mut members = Vec::with_capacity(members_arr.len());
    for member_val in members_arr {
        let parts = member_val
            .as_array()
            .ok_or_else(|| Error::Decode("member must be array".into()))?;
        if parts.is_empty() {
            return Err(Error::Decode("empty member array".into()));
        }
        let member_code = parts[0]
            .as_u64()
            .ok_or_else(|| Error::Decode("member code must be integer".into()))?
            as u8;

        match member_code {
            MEMBER_PROPERTY => {
                if parts.len() < 3 {
                    return Err(Error::Decode("property needs 3 elements".into()));
                }
                let name = parts[1]
                    .as_str()
                    .ok_or_else(|| Error::Decode("property name must be string".into()))?
                    .to_string();
                let value = decode_pkl_value(&parts[2])?;
                members.push(ObjectMember::Property { name, value });
            }
            MEMBER_ENTRY => {
                if parts.len() < 3 {
                    return Err(Error::Decode("entry needs 3 elements".into()));
                }
                let key = decode_pkl_value(&parts[1])?;
                let value = decode_pkl_value(&parts[2])?;
                members.push(ObjectMember::Entry { key, value });
            }
            MEMBER_ELEMENT => {
                if parts.len() < 3 {
                    return Err(Error::Decode("element needs 3 elements".into()));
                }
                let index = parts[1]
                    .as_u64()
                    .ok_or_else(|| Error::Decode("element index must be integer".into()))?
                    as usize;
                let value = decode_pkl_value(&parts[2])?;
                members.push(ObjectMember::Element { index, value });
            }
            _ => return Err(Error::UnknownMemberCode(member_code)),
        }
    }

    Ok(PklValue::Object {
        class_name,
        module_uri,
        members,
    })
}

fn decode_map(slots: &[Value]) -> Result<PklValue> {
    if slots.is_empty() {
        return Ok(PklValue::Map(vec![]));
    }
    let map_val = slots[0]
        .as_map()
        .ok_or_else(|| Error::Decode("map slot must be msgpack map".into()))?;
    let mut pairs = Vec::with_capacity(map_val.len());
    for (k, v) in map_val {
        pairs.push((decode_pkl_value(k)?, decode_pkl_value(v)?));
    }
    Ok(PklValue::Map(pairs))
}

fn decode_list(slots: &[Value]) -> Result<PklValue> {
    if slots.is_empty() {
        return Ok(PklValue::List(vec![]));
    }
    let arr = slots[0]
        .as_array()
        .ok_or_else(|| Error::Decode("list slot must be msgpack array".into()))?;
    let items: Result<Vec<PklValue>> = arr.iter().map(decode_pkl_value).collect();
    Ok(PklValue::List(items?))
}

fn decode_set(slots: &[Value]) -> Result<PklValue> {
    if slots.is_empty() {
        return Ok(PklValue::Set(vec![]));
    }
    let arr = slots[0]
        .as_array()
        .ok_or_else(|| Error::Decode("set slot must be msgpack array".into()))?;
    let items: Result<Vec<PklValue>> = arr.iter().map(decode_pkl_value).collect();
    Ok(PklValue::Set(items?))
}

fn decode_duration(slots: &[Value]) -> Result<PklValue> {
    if slots.len() < 2 {
        return Err(Error::Decode("duration needs 2 slots".into()));
    }
    let value = slots[0]
        .as_f64()
        .ok_or_else(|| Error::Decode("duration value must be float".into()))?;
    let unit_str = slots[1]
        .as_str()
        .ok_or_else(|| Error::Decode("duration unit must be string".into()))?;
    let unit = unit_str
        .parse::<DurationUnit>()
        .map_err(Error::Decode)?;
    Ok(PklValue::Duration(Duration::new(value, unit)))
}

fn decode_data_size(slots: &[Value]) -> Result<PklValue> {
    if slots.len() < 2 {
        return Err(Error::Decode("data size needs 2 slots".into()));
    }
    let value = slots[0]
        .as_f64()
        .ok_or_else(|| Error::Decode("data size value must be float".into()))?;
    let unit_str = slots[1]
        .as_str()
        .ok_or_else(|| Error::Decode("data size unit must be string".into()))?;
    let unit = unit_str
        .parse::<DataSizeUnit>()
        .map_err(Error::Decode)?;
    Ok(PklValue::DataSize(DataSize::new(value, unit)))
}

fn decode_pair(slots: &[Value]) -> Result<PklValue> {
    if slots.len() < 2 {
        return Err(Error::Decode("pair needs 2 slots".into()));
    }
    let first = decode_pkl_value(&slots[0])?;
    let second = decode_pkl_value(&slots[1])?;
    Ok(PklValue::Pair(Box::new(first), Box::new(second)))
}

fn decode_int_seq(slots: &[Value]) -> Result<PklValue> {
    if slots.len() < 3 {
        return Err(Error::Decode("intseq needs 3 slots".into()));
    }
    let start = slots[0]
        .as_i64()
        .ok_or_else(|| Error::Decode("intseq start must be integer".into()))?;
    let end = slots[1]
        .as_i64()
        .ok_or_else(|| Error::Decode("intseq end must be integer".into()))?;
    let step = slots[2]
        .as_i64()
        .ok_or_else(|| Error::Decode("intseq step must be integer".into()))?;
    Ok(PklValue::IntSeq(IntSeq::new(start, end, step)))
}

fn decode_regex(slots: &[Value]) -> Result<PklValue> {
    if slots.is_empty() {
        return Err(Error::Decode("regex needs 1 slot".into()));
    }
    let pattern = slots[0]
        .as_str()
        .ok_or_else(|| Error::Decode("regex pattern must be string".into()))?
        .to_string();
    Ok(PklValue::Regex(PklRegex { pattern }))
}

fn decode_class(slots: &[Value]) -> Result<PklValue> {
    if slots.len() < 2 {
        return Err(Error::Decode("class needs 2 slots".into()));
    }
    let class_name = slots[0]
        .as_str()
        .ok_or_else(|| Error::Decode("class name must be string".into()))?
        .to_string();
    let module_uri = slots[1]
        .as_str()
        .ok_or_else(|| Error::Decode("class module_uri must be string".into()))?
        .to_string();
    Ok(PklValue::Class {
        class_name,
        module_uri,
    })
}

fn decode_type_alias(slots: &[Value]) -> Result<PklValue> {
    if slots.len() < 2 {
        return Err(Error::Decode("type alias needs 2 slots".into()));
    }
    let name = slots[0]
        .as_str()
        .ok_or_else(|| Error::Decode("type alias name must be string".into()))?
        .to_string();
    let module_uri = slots[1]
        .as_str()
        .ok_or_else(|| Error::Decode("type alias module_uri must be string".into()))?
        .to_string();
    Ok(PklValue::TypeAlias { name, module_uri })
}

fn decode_bytes(slots: &[Value]) -> Result<PklValue> {
    if slots.is_empty() {
        return Err(Error::Decode("bytes needs 1 slot".into()));
    }
    let data = slots[0]
        .as_slice()
        .ok_or_else(|| Error::Decode("bytes data must be binary".into()))?
        .to_vec();
    Ok(PklValue::Bytes(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_primitives() {
        assert_eq!(decode_pkl_value(&Value::Nil).unwrap(), PklValue::Null);
        assert_eq!(
            decode_pkl_value(&Value::Boolean(true)).unwrap(),
            PklValue::Bool(true)
        );
        assert_eq!(
            decode_pkl_value(&Value::from(42)).unwrap(),
            PklValue::Int(42)
        );
        assert_eq!(
            decode_pkl_value(&Value::F64(3.14)).unwrap(),
            PklValue::Float(3.14)
        );
        assert_eq!(
            decode_pkl_value(&Value::String("hello".into())).unwrap(),
            PklValue::String("hello".into())
        );
    }

    #[test]
    fn test_decode_object() {
        let obj = Value::Array(vec![
            Value::from(TYPE_OBJECT as u64),
            Value::String("Config".into()),
            Value::String("file:///test.pkl".into()),
            Value::Array(vec![
                Value::Array(vec![
                    Value::from(MEMBER_PROPERTY as u64),
                    Value::String("name".into()),
                    Value::String("test".into()),
                ]),
                Value::Array(vec![
                    Value::from(MEMBER_PROPERTY as u64),
                    Value::String("port".into()),
                    Value::from(8080),
                ]),
            ]),
        ]);
        let result = decode_pkl_value(&obj).unwrap();
        match result {
            PklValue::Object {
                class_name,
                module_uri,
                members,
            } => {
                assert_eq!(class_name, "Config");
                assert_eq!(module_uri, "file:///test.pkl");
                assert_eq!(members.len(), 2);
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn test_decode_duration() {
        let dur = Value::Array(vec![
            Value::from(TYPE_DURATION as u64),
            Value::F64(5.0),
            Value::String("s".into()),
        ]);
        let result = decode_pkl_value(&dur).unwrap();
        match result {
            PklValue::Duration(d) => {
                assert_eq!(d.value, 5.0);
                assert_eq!(d.unit, DurationUnit::S);
            }
            _ => panic!("expected Duration"),
        }
    }

    #[test]
    fn test_decode_list() {
        let list = Value::Array(vec![
            Value::from(TYPE_LIST as u64),
            Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)]),
        ]);
        let result = decode_pkl_value(&list).unwrap();
        match result {
            PklValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], PklValue::Int(1));
            }
            _ => panic!("expected List"),
        }
    }
}
