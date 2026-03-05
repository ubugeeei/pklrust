use pklrs::decoder::decode_pkl_binary;
use pklrs::value::{ObjectMember, PklValue};
use rmpv::Value;

fn encode_value(v: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    rmpv::encode::write_value(&mut buf, v).unwrap();
    buf
}

#[test]
fn test_decode_null() {
    let bytes = encode_value(&Value::Nil);
    let result = decode_pkl_binary(&bytes).unwrap();
    assert_eq!(result, PklValue::Null);
}

#[test]
fn test_decode_bool() {
    let bytes = encode_value(&Value::Boolean(true));
    let result = decode_pkl_binary(&bytes).unwrap();
    assert_eq!(result, PklValue::Bool(true));
}

#[test]
fn test_decode_integer() {
    let bytes = encode_value(&Value::from(42));
    let result = decode_pkl_binary(&bytes).unwrap();
    assert_eq!(result, PklValue::Int(42));
}

#[test]
fn test_decode_float() {
    let bytes = encode_value(&Value::F64(3.14));
    let result = decode_pkl_binary(&bytes).unwrap();
    assert_eq!(result, PklValue::Float(3.14));
}

#[test]
fn test_decode_string() {
    let bytes = encode_value(&Value::String("hello".into()));
    let result = decode_pkl_binary(&bytes).unwrap();
    assert_eq!(result, PklValue::String("hello".into()));
}

#[test]
fn test_decode_object_with_properties() {
    let obj = Value::Array(vec![
        Value::from(0x01u64),
        Value::String("Server".into()),
        Value::String("file:///config.pkl".into()),
        Value::Array(vec![
            Value::Array(vec![
                Value::from(0x10u64),
                Value::String("host".into()),
                Value::String("localhost".into()),
            ]),
            Value::Array(vec![
                Value::from(0x10u64),
                Value::String("port".into()),
                Value::from(8080),
            ]),
        ]),
    ]);

    let bytes = encode_value(&obj);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::Object {
            class_name,
            module_uri,
            members,
        } => {
            assert_eq!(class_name, "Server");
            assert_eq!(module_uri, "file:///config.pkl");
            assert_eq!(members.len(), 2);

            match &members[0] {
                ObjectMember::Property { name, value } => {
                    assert_eq!(name, "host");
                    assert_eq!(*value, PklValue::String("localhost".into()));
                }
                _ => panic!("expected Property"),
            }
            match &members[1] {
                ObjectMember::Property { name, value } => {
                    assert_eq!(name, "port");
                    assert_eq!(*value, PklValue::Int(8080));
                }
                _ => panic!("expected Property"),
            }
        }
        _ => panic!("expected Object, got: {result:?}"),
    }
}

#[test]
fn test_decode_list() {
    let list = Value::Array(vec![
        Value::from(0x04u64),
        Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)]),
    ]);

    let bytes = encode_value(&list);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], PklValue::Int(1));
            assert_eq!(items[1], PklValue::Int(2));
            assert_eq!(items[2], PklValue::Int(3));
        }
        _ => panic!("expected List"),
    }
}

#[test]
fn test_decode_map() {
    let map = Value::Array(vec![
        Value::from(0x02u64),
        Value::Map(vec![
            (Value::String("a".into()), Value::from(1)),
            (Value::String("b".into()), Value::from(2)),
        ]),
    ]);

    let bytes = encode_value(&map);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::Map(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, PklValue::String("a".into()));
            assert_eq!(entries[0].1, PklValue::Int(1));
        }
        _ => panic!("expected Map"),
    }
}

#[test]
fn test_decode_duration() {
    let dur = Value::Array(vec![
        Value::from(0x07u64),
        Value::F64(5.0),
        Value::String("s".into()),
    ]);

    let bytes = encode_value(&dur);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::Duration(d) => {
            assert_eq!(d.value, 5.0);
            assert_eq!(d.unit, pklrs::DurationUnit::S);
        }
        _ => panic!("expected Duration"),
    }
}

#[test]
fn test_decode_data_size() {
    let ds = Value::Array(vec![
        Value::from(0x08u64),
        Value::F64(512.0),
        Value::String("mb".into()),
    ]);

    let bytes = encode_value(&ds);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::DataSize(d) => {
            assert_eq!(d.value, 512.0);
            assert_eq!(d.unit, pklrs::DataSizeUnit::Mb);
        }
        _ => panic!("expected DataSize"),
    }
}

#[test]
fn test_decode_pair() {
    let pair = Value::Array(vec![
        Value::from(0x09u64),
        Value::String("key".into()),
        Value::from(42),
    ]);

    let bytes = encode_value(&pair);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::Pair(first, second) => {
            assert_eq!(*first, PklValue::String("key".into()));
            assert_eq!(*second, PklValue::Int(42));
        }
        _ => panic!("expected Pair"),
    }
}

#[test]
fn test_decode_intseq() {
    let seq = Value::Array(vec![
        Value::from(0x0Au64),
        Value::from(0),
        Value::from(10),
        Value::from(2),
    ]);

    let bytes = encode_value(&seq);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::IntSeq(s) => {
            assert_eq!(s.start, 0);
            assert_eq!(s.end, 10);
            assert_eq!(s.step, 2);
        }
        _ => panic!("expected IntSeq"),
    }
}

#[test]
fn test_decode_set() {
    let set = Value::Array(vec![
        Value::from(0x06u64),
        Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)]),
    ]);

    let bytes = encode_value(&set);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::Set(items) => {
            assert_eq!(items.len(), 3);
        }
        _ => panic!("expected Set"),
    }
}

#[test]
fn test_decode_regex() {
    let regex = Value::Array(vec![
        Value::from(0x0Bu64),
        Value::String("^hello.*$".into()),
    ]);

    let bytes = encode_value(&regex);
    let result = decode_pkl_binary(&bytes).unwrap();

    match result {
        PklValue::Regex(r) => {
            assert_eq!(r.pattern, "^hello.*$");
        }
        _ => panic!("expected Regex"),
    }
}

#[test]
fn test_decode_nested_object() {
    let obj = Value::Array(vec![
        Value::from(0x01u64),
        Value::String("Config".into()),
        Value::String("file:///config.pkl".into()),
        Value::Array(vec![
            Value::Array(vec![
                Value::from(0x10u64),
                Value::String("server".into()),
                Value::Array(vec![
                    Value::from(0x01u64),
                    Value::String("Server".into()),
                    Value::String("file:///config.pkl".into()),
                    Value::Array(vec![Value::Array(vec![
                        Value::from(0x10u64),
                        Value::String("host".into()),
                        Value::String("localhost".into()),
                    ])]),
                ]),
            ]),
            Value::Array(vec![
                Value::from(0x10u64),
                Value::String("debug".into()),
                Value::Boolean(true),
            ]),
        ]),
    ]);

    let bytes = encode_value(&obj);
    let result = decode_pkl_binary(&bytes).unwrap();

    match &result {
        PklValue::Object { members, .. } => {
            assert_eq!(members.len(), 2);
            if let ObjectMember::Property { name, value } = &members[0] {
                assert_eq!(name, "server");
                assert!(matches!(value, PklValue::Object { .. }));
            }
        }
        _ => panic!("expected Object"),
    }
}
