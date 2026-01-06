use core::fmt::{Debug, Display};
use core::hash::{Hash, Hasher};
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Index, IndexMut};

use crate::dict_key::Array;
use crate::iter::Values;
use crate::{DictKey, OptionalDict};

/// A dictionary that requires all keys to have values
#[repr(transparent)]
pub struct RequiredDict<K: DictKey, V> {
    pub(crate) inner: K::Array<V>,
    pub(crate) phantom: PhantomData<K>,
}

impl<K: DictKey, V> RequiredDict<K, V> {
    #[inline]
    pub(crate) fn from_inner(inner: K::Array<V>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.as_ref().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(K) -> V,
    {
        Self::from_inner(Array::from_fn(|i| f(K::from_index(i))))
    }

    #[inline]
    pub fn try_from_fn<F, E>(mut f: F) -> Result<Self, E>
    where
        F: FnMut(K) -> Result<V, E>,
    {
        Ok(Self::from_inner(Array::try_from_fn(|i| f(K::from_index(i)))?))
    }

    #[inline]
    pub fn map<F, U>(self, mut f: F) -> RequiredDict<K, U>
    where
        F: FnMut(V) -> U,
    {
        let mut iter = self.inner.into_iter();
        RequiredDict::from_inner(Array::from_fn(|_| f(iter.next().unwrap())))
    }

    #[inline]
    pub fn try_map<F, U, E>(self, mut f: F) -> Result<RequiredDict<K, U>, E>
    where
        F: FnMut(V) -> Result<U, E>,
    {
        let mut iter = self.inner.into_iter();
        Ok(RequiredDict::from_inner(Array::try_from_fn(|_| {
            f(iter.next().unwrap())
        })?))
    }

    #[inline]
    pub fn each_ref(&self) -> RequiredDict<K, &V> {
        let mut iter = self.inner.as_ref().iter();
        RequiredDict::from_inner(Array::from_fn(|_| iter.next().unwrap()))
    }

    #[inline]
    pub fn each_mut(&mut self) -> RequiredDict<K, &mut V> {
        let mut iter = self.inner.as_mut().iter_mut();
        RequiredDict::from_inner(Array::from_fn(|_| iter.next().unwrap()))
    }

    #[inline]
    pub fn into_values(self) -> Values<IntoIter<K, V>> {
        Values(self.into_iter())
    }

    #[inline]
    pub fn downgrade(self) -> OptionalDict<K, V> {
        let mut iter = self.inner.into_iter();
        OptionalDict::from_inner(Array::from_fn(|_| Some(iter.next().unwrap())))
    }
}

impl<K: DictKey, V> From<OptionalDict<K, V>> for RequiredDict<K, Option<V>> {
    #[inline]
    fn from(dict: OptionalDict<K, V>) -> Self {
        Self::from_inner(dict.inner)
    }
}

impl<K: DictKey, V: Default> Default for RequiredDict<K, V> {
    #[inline]
    fn default() -> Self {
        Self::from_inner(Array::from_fn(|_| V::default()))
    }
}

impl<K: DictKey, V: Clone> Clone for RequiredDict<K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from_inner(Array::from_fn(|i| self.inner.as_ref()[i].clone()))
    }
}

// Actually, `K::Array<V>: Copy` always holds when `V: Copy`
// (because `K::Array<V>` is an array), but Rust currently cannot recognize it.
impl<K: DictKey, V: Copy> Copy for RequiredDict<K, V> where K::Array<V>: Copy {}

impl<K: DictKey, V: PartialEq> PartialEq for RequiredDict<K, V> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref() == other.inner.as_ref()
    }
}

impl<K: DictKey, V: Eq> Eq for RequiredDict<K, V> {}

impl<K: DictKey, V: PartialOrd> PartialOrd for RequiredDict<K, V> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.inner.as_ref().partial_cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Ord> Ord for RequiredDict<K, V> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.as_ref().cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Hash> Hash for RequiredDict<K, V> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.as_ref().hash(state);
    }
}

impl<K: DictKey, V> Index<K> for RequiredDict<K, V> {
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        &self.inner.as_ref()[key.as_index()]
    }
}

impl<K: DictKey, V> IndexMut<K> for RequiredDict<K, V> {
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner.as_mut()[key.as_index()]
    }
}

impl<K: DictKey, V: Debug> Debug for RequiredDict<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(
                self.inner
                    .as_ref()
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (K::VARIANTS[index], value)),
            )
            .finish()
    }
}

impl<K: DictKey, V: Display> Display for RequiredDict<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{{")?;
        let mut is_first = true;
        for (index, value) in self.inner.as_ref().iter().enumerate() {
            if !is_first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", K::VARIANTS[index], value)?;
            is_first = false;
        }
        write!(f, "}}")
    }
}

pub struct IntoIter<K: DictKey, V> {
    inner: K::Array<MaybeUninit<V>>,
    start: usize,
    end: usize,
}

impl<K: DictKey, V> From<RequiredDict<K, V>> for IntoIter<K, V> {
    #[inline]
    fn from(dict: RequiredDict<K, V>) -> Self {
        let len = dict.inner.as_ref().len();
        // SAFETY: `K::Array<V>` is effectively `[V; N]`, and `[V; N]` has the same layout as `[MaybeUninit<V>; N]`.
        let inner = ManuallyDrop::new(dict.inner);
        let ptr = &*inner as *const K::Array<V> as *const K::Array<MaybeUninit<V>>;
        let inner = unsafe { ptr.read() };
        Self {
            inner,
            start: 0,
            end: len,
        }
    }
}

impl<K: DictKey, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }
        let index = self.start;
        self.start += 1;
        // Safety: index is in [0, end), so it's valid and initialized
        let value = unsafe { self.inner.as_ref()[index].assume_init_read() };
        Some((K::from_index(index), value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

impl<K: DictKey, V> ExactSizeIterator for IntoIter<K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl<K: DictKey, V> FusedIterator for IntoIter<K, V> {}

impl<K: DictKey, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }
        self.end -= 1;
        let index = self.end;
        // Safety: index is in [start, original_end), so it's valid and initialized
        let value = unsafe { self.inner.as_ref()[index].assume_init_read() };
        Some((K::from_index(index), value))
    }
}

impl<K: DictKey, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        // Drop remaining elements that haven't been yielded
        for i in self.start..self.end {
            // Safety: elements in [start, end) are still initialized
            unsafe {
                self.inner.as_mut()[i].assume_init_drop();
            }
        }
    }
}

impl<K: DictKey, V> IntoIterator for RequiredDict<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<K: DictKey, V> RequiredDict<K, MaybeUninit<V>> {
    /// Transpose `RequiredDict<K, MaybeUninit<V>>` into `MaybeUninit<RequiredDict<K, V>>`.
    #[inline]
    pub fn transpose(self) -> MaybeUninit<RequiredDict<K, V>> {
        // SAFETY: `MaybeUninit<RequiredDict<K, V>>` and `RequiredDict<K, MaybeUninit<V>>`
        // have the same layout because:
        // 1. `RequiredDict<K, V>` is `#[repr(transparent)]` over `K::Array<V>`
        // 2. `K::Array<V>` is effectively `[V; N]`
        // 3. `[V; N]` has the same layout as `[MaybeUninit<V>; N]`
        let this = ManuallyDrop::new(self);
        let ptr = &*this as *const Self as *const MaybeUninit<_>;
        unsafe { ptr.read() }
    }
}

#[macro_export]
macro_rules! required_dict {
    ($($(|)? $head:ident $(::$tail:ident)* $(|$or_head:ident $(::$or_tail:ident)*)* => $value:expr),* $(,)?) => {{
        // Compile-time exhaustiveness check
        let _ = |key: _| match key {
            $(
                $head $(::$tail)*
                $(|$or_head $(::$or_tail)*)*
                => (),
            )*
        };

        let mut dict = $crate::RequiredDict::from_fn(|_| ::core::mem::MaybeUninit::uninit());
        $(
            dict[$head $(::$tail)*].write($value);
            $(dict[$or_head $(::$or_tail)*].write($value);)*
        )*
        // Safety: exhaustiveness check ensures all variants are covered,
        // so all `MaybeUninit` slots have been initialized via `write()`.
        unsafe { dict.transpose().assume_init() }
    }};

    ($($(|)? $head:ident $(::$tail:ident)* $(|$or_head:ident $(::$or_tail:ident)*)* => $value:expr,)* _ => $default:expr $(,)?) => {{
        let mut dict = $crate::RequiredDict::from_fn(|_| $default);
        $(
            dict[$head $(::$tail)*] = $value;
            $(dict[$or_head $(::$or_tail)*] = $value;)*
        )*
        dict
    }};
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::ser::SerializeMap;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    impl<K: DictKey, V: Serialize> Serialize for RequiredDict<K, V> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(Some(self.inner.as_ref().len()))?;
            for (index, value) in self.inner.as_ref().iter().enumerate() {
                map.serialize_entry(K::VARIANTS[index], value)?;
            }
            map.end()
        }
    }

    struct MissingKeys<K: DictKey, V>(OptionalDict<K, V>);

    impl<K: DictKey, V> core::fmt::Display for MissingKeys<K, V> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "Missing keys: ")?;
            let mut is_first = true;
            for (index, value) in self.0.inner.as_ref().iter().enumerate() {
                if value.is_some() {
                    continue;
                }
                if !is_first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", K::VARIANTS[index])?;
                is_first = false;
            }
            Ok(())
        }
    }

    impl<'de, K: DictKey, V: Deserialize<'de>> Deserialize<'de> for RequiredDict<K, V> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            OptionalDict::<K, V>::deserialize(deserializer)?
                .upgrade()
                .map_err(|dict| serde::de::Error::custom(MissingKeys(dict)))
        }
    }
}

#[cfg(feature = "std")]
mod std_impl {
    extern crate std;

    use std::collections::{BTreeMap, HashMap};

    use super::*;

    pub struct MissingKey<K>(K);

    impl<K: DictKey> core::fmt::Debug for MissingKey<K> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "MissingKey({})", K::VARIANTS[self.0.as_index()])
        }
    }

    impl<K: DictKey> core::fmt::Display for MissingKey<K> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "Key {} missing when converting to RequiredDict",
                K::VARIANTS[self.0.as_index()]
            )
        }
    }

    impl<K: DictKey> core::error::Error for MissingKey<K> {}

    impl<K: DictKey + Eq + Hash, V> TryFrom<HashMap<K, V>> for RequiredDict<K, V> {
        type Error = MissingKey<K>;

        #[inline]
        fn try_from(mut map: HashMap<K, V>) -> Result<Self, Self::Error> {
            RequiredDict::try_from_fn(|key| map.remove(&key).ok_or(MissingKey(key)))
        }
    }

    impl<K: DictKey + Ord, V> TryFrom<BTreeMap<K, V>> for RequiredDict<K, V> {
        type Error = MissingKey<K>;

        #[inline]
        fn try_from(mut map: BTreeMap<K, V>) -> Result<Self, Self::Error> {
            RequiredDict::try_from_fn(|key| map.remove(&key).ok_or(MissingKey(key)))
        }
    }
}
