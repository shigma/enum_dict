#![doc = include_str!("../README.md")]
#![no_std]

mod dict_key;
pub(crate) mod optional_dict;
pub(crate) mod required_dict;

pub use dict_key::DictKey;
#[cfg(feature = "derive")]
pub use enum_dict_derive::DictKey;
pub use optional_dict::OptionalDict;
pub use required_dict::RequiredDict;
