use core::fmt;
use std::marker::PhantomData;

/// Trait for types that can be used as dictionary keys
pub trait DictKey: Sized {
    const FIELDS: &'static [&'static str];

    /// Convert to usize index
    fn into_usize(self) -> usize;
}
