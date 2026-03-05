use std::collections::HashMap;

use pklrs::de::from_pkl_value;
use pklrs::value::{ObjectMember, PklValue};
use serde::Deserialize;

#[test]
fn test_deserialize_primitives() {
    assert_eq!(from_pkl_value::<bool>(&PklValue::Bool(true)).unwrap(), true);
    assert_eq!(from_pkl_value::<i64>(&PklValue::Int(42)).unwrap(), 42);
    assert_eq!(from_pkl_value::<f64>(&PklValue::Float(3.14)).unwrap(), 3.14);
    assert_eq!(
        from_pkl_value::<String>(&PklValue::String("hello".into())).unwrap(),
        "hello"
    );
}

#[test]
fn test_deserialize_option_none() {
    let result: Option<i64> = from_pkl_value(&PklValue::Null).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_deserialize_option_some() {
    let result: Option<i64> = from_pkl_value(&PklValue::Int(42)).unwrap();
    assert_eq!(result, Some(42));
}

#[test]
fn test_deserialize_vec() {
    let list = PklValue::List(vec![PklValue::Int(1), PklValue::Int(2), PklValue::Int(3)]);
    let result: Vec<i64> = from_pkl_value(&list).unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_deserialize_hashmap() {
    let map = PklValue::Map(vec![
        (PklValue::String("a".into()), PklValue::Int(1)),
        (PklValue::String("b".into()), PklValue::Int(2)),
    ]);
    let result: HashMap<String, i64> = from_pkl_value(&map).unwrap();
    assert_eq!(result.get("a"), Some(&1));
    assert_eq!(result.get("b"), Some(&2));
}

#[test]
fn test_deserialize_struct_from_object() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Server {
        host: String,
        port: i64,
        debug: bool,
    }

    let obj = PklValue::Object {
        class_name: "Server".into(),
        module_uri: "file:///config.pkl".into(),
        members: vec![
            ObjectMember::Property {
                name: "host".into(),
                value: PklValue::String("127.0.0.1".into()),
            },
            ObjectMember::Property {
                name: "port".into(),
                value: PklValue::Int(3000),
            },
            ObjectMember::Property {
                name: "debug".into(),
                value: PklValue::Bool(false),
            },
        ],
    };

    let result: Server = from_pkl_value(&obj).unwrap();
    assert_eq!(
        result,
        Server {
            host: "127.0.0.1".into(),
            port: 3000,
            debug: false,
        }
    );
}

#[test]
fn test_deserialize_nested_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Database {
        host: String,
        port: i64,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Config {
        name: String,
        database: Database,
    }

    let obj = PklValue::Object {
        class_name: "Config".into(),
        module_uri: "file:///config.pkl".into(),
        members: vec![
            ObjectMember::Property {
                name: "name".into(),
                value: PklValue::String("my-app".into()),
            },
            ObjectMember::Property {
                name: "database".into(),
                value: PklValue::Object {
                    class_name: "Database".into(),
                    module_uri: "file:///config.pkl".into(),
                    members: vec![
                        ObjectMember::Property {
                            name: "host".into(),
                            value: PklValue::String("db.example.com".into()),
                        },
                        ObjectMember::Property {
                            name: "port".into(),
                            value: PklValue::Int(5432),
                        },
                    ],
                },
            },
        ],
    };

    let result: Config = from_pkl_value(&obj).unwrap();
    assert_eq!(
        result,
        Config {
            name: "my-app".into(),
            database: Database {
                host: "db.example.com".into(),
                port: 5432,
            },
        }
    );
}

#[test]
fn test_deserialize_struct_with_list() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Project {
        name: String,
        tags: Vec<String>,
    }

    let obj = PklValue::Object {
        class_name: "Project".into(),
        module_uri: "file:///project.pkl".into(),
        members: vec![
            ObjectMember::Property {
                name: "name".into(),
                value: PklValue::String("pklrs".into()),
            },
            ObjectMember::Property {
                name: "tags".into(),
                value: PklValue::List(vec![
                    PklValue::String("rust".into()),
                    PklValue::String("pkl".into()),
                    PklValue::String("config".into()),
                ]),
            },
        ],
    };

    let result: Project = from_pkl_value(&obj).unwrap();
    assert_eq!(
        result,
        Project {
            name: "pklrs".into(),
            tags: vec!["rust".into(), "pkl".into(), "config".into()],
        }
    );
}

#[test]
fn test_deserialize_struct_with_optional_field() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Config {
        name: String,
        description: Option<String>,
    }

    let obj = PklValue::Object {
        class_name: "Config".into(),
        module_uri: "file:///config.pkl".into(),
        members: vec![
            ObjectMember::Property {
                name: "name".into(),
                value: PklValue::String("app".into()),
            },
            ObjectMember::Property {
                name: "description".into(),
                value: PklValue::Null,
            },
        ],
    };

    let result: Config = from_pkl_value(&obj).unwrap();
    assert_eq!(
        result,
        Config {
            name: "app".into(),
            description: None,
        }
    );
}

#[test]
fn test_deserialize_duration() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Timeout {
        value: f64,
        unit: String,
    }

    let dur = PklValue::Duration(pklrs::Duration::new(30.0, pklrs::DurationUnit::S));
    let result: Timeout = from_pkl_value(&dur).unwrap();
    assert_eq!(
        result,
        Timeout {
            value: 30.0,
            unit: "s".into(),
        }
    );
}

#[test]
fn test_deserialize_data_size() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Size {
        value: f64,
        unit: String,
    }

    let ds = PklValue::DataSize(pklrs::DataSize::new(256.0, pklrs::DataSizeUnit::Mb));
    let result: Size = from_pkl_value(&ds).unwrap();
    assert_eq!(
        result,
        Size {
            value: 256.0,
            unit: "mb".into(),
        }
    );
}
