use std::marker::PhantomData;
use std::mem::MaybeUninit;

pub trait Array<T>: AsRef<[T]> + AsMut<[T]> + IntoIterator<Item = T> + Sized {
    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> T;

    fn try_from_fn<F, E>(f: F) -> Result<Self, E>
    where
        F: FnMut(usize) -> Result<T, E>;
}

impl<const N: usize, T> Array<T> for [T; N] {
    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        std::array::from_fn(f)
    }

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

    /// Convert to usize index
    fn variant_index(self) -> usize;
}

pub(crate) struct DictVisitor<K, V>(PhantomData<(K, V)>);

impl<K, V> DictVisitor<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use std::fmt;

    use serde::Deserialize;
    use serde::de::{MapAccess, Visitor};

    use super::*;

    impl<'de, K: DictKey, V: Deserialize<'de>> Visitor<'de> for DictVisitor<K, V> {
        type Value = Vec<Option<V>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with optional keys")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut vec = K::VARIANTS.iter().map(|_| None).collect::<Vec<_>>();
            while let Some((key, value)) = map.next_entry::<String, V>()? {
                // ignore unknown keys
                for (index, &name) in K::VARIANTS.iter().enumerate() {
                    if name == key {
                        vec[index] = Some(value);
                        break;
                    }
                }
            }
            Ok(vec)
        }
    }
}
