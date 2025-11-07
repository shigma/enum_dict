use enum_dict::{DictKey, OptionalDict, RequiredDict};
use serde::{Deserialize, Serialize};

#[derive(DictKey)]
enum Key {
    A,
    B,
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    required: RequiredDict<Key, u32>,
    optional: OptionalDict<Key, u32>,
}

#[test]
fn test_serde() {
    let json = r#"{
        "required": {
            "A": 1,
            "B": 2
        },
        "optional": {
            "A": 3,
            "X": 4
        }
    }"#;

    let data: Data = serde_json::from_str(json).unwrap();

    assert_eq!(data.required.len(), 2);
    assert_eq!(data.required[Key::A], 1);
    assert_eq!(data.required[Key::B], 2);
    assert_eq!(data.optional.len(), 1);
    assert_eq!(data.optional[Key::A], Some(3));
    assert_eq!(data.optional[Key::B], None);

    assert_eq!(
        serde_json::to_string(&data).unwrap(),
        r#"{"required":{"A":1,"B":2},"optional":{"A":3}}"#
    );
}

#[test]
fn test_validate() {
    let json = r#"{
        "required": {},
        "optional": {
            "A": 3
        }
    }"#;

    let err = serde_json::from_str::<Data>(json).unwrap_err();
    assert_eq!(err.to_string(), "Missing keys: A, B at line 2 column 22");
}
