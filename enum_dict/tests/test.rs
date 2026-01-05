use enum_dict::{DictKey, OptionalDict, RequiredDict};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, DictKey)]
#[enum_dict(rename_all = "lowercase")]
enum Key {
    A = 1,
    B = 0,
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
            "b": 2,
            "a": 1
        },
        "optional": {
            "B": 4,
            "a": 3
        }
    }"#;

    let data: Data = serde_json::from_str(json).unwrap();

    assert_eq!(data.required.len(), 2);
    assert_eq!(data.required[Key::A], 1);
    assert_eq!(data.required[Key::B], 2);
    assert_eq!(data.optional.len(), 1);
    assert_eq!(data.optional[Key::A], Some(3));
    assert_eq!(data.optional[Key::B], None);

    assert_eq!(data.required.to_string(), "{a: 1, b: 2}");
    assert_eq!(data.optional.to_string(), "{a: 3}");

    assert_eq!(
        serde_json::to_string(&data).unwrap(),
        r#"{"required":{"a":1,"b":2},"optional":{"a":3}}"#
    );

    assert_eq!(
        data.required.into_iter().collect::<Vec<_>>(),
        vec![(Key::A, 1), (Key::B, 2)],
    );
    assert_eq!(data.optional.into_iter().collect::<Vec<_>>(), vec![(Key::A, 3)],);
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
    assert_eq!(err.to_string(), "Missing keys: a, b at line 2 column 22");
}
