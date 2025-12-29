use core::mem::MaybeUninit;

pub trait Array<T>: AsRef<[T]> + AsMut<[T]> + IntoIterator<Item = T> + Sized {
    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> T;

    fn try_from_fn<F, E>(f: F) -> Result<Self, E>
    where
        F: FnMut(usize) -> Result<T, E>;
}

impl<const N: usize, T> Array<T> for [T; N] {
    #[inline]
    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        core::array::from_fn(f)
    }

    // We may replace this with core::array::try_from_fn when stabilized.
    // See: https://github.com/rust-lang/rust/issues/89379
    fn try_from_fn<F, E>(mut f: F) -> Result<Self, E>
    where
        F: FnMut(usize) -> Result<T, E>,
    {
        let mut arr = MaybeUninit::<Self>::uninit();
        unsafe {
            let ptr = arr.as_mut_ptr() as *mut T;
            for i in 0..N {
                ptr.add(i).write(f(i)?);
            }
            Ok(arr.assume_init())
        }
    }
}

/// Trait for types that can be used as dictionary keys
pub trait DictKey {
    type Array<T>: Array<T>;

    const VARIANTS: &'static [&'static str];

    fn from_index(index: usize) -> Self;

    fn into_index(self) -> usize;
}

#[cfg(feature = "serde")]
pub struct DictVisitor<K, V>(core::marker::PhantomData<(K, V)>);

#[cfg(feature = "serde")]
impl<K, V> DictVisitor<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use core::fmt;

    use serde::Deserialize;
    use serde::de::{MapAccess, Visitor};

    use super::*;
    use crate::OptionalDict;

    impl<'de, K: DictKey, V: Deserialize<'de>> Visitor<'de> for DictVisitor<K, V> {
        type Value = OptionalDict<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with optional keys")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut dict = OptionalDict::<K, V>::new();
            while let Some((key, value)) = map.next_entry::<&str, V>()? {
                // ignore unknown keys
                for (index, &name) in K::VARIANTS.iter().enumerate() {
                    if name == key {
                        dict.inner.as_mut()[index] = Some(value);
                        break;
                    }
                }
            }
            Ok(dict)
        }
    }
}
