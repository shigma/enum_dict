use core::fmt::{Debug, Display};
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use crate::dict_key::Array;
use crate::{DictKey, OptionalDict};

/// A dictionary that requires all keys to have values
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

    pub fn map<F, U>(self, mut f: F) -> RequiredDict<K, U>
    where
        F: FnMut(V) -> U,
    {
        let mut iter = self.inner.into_iter();
        RequiredDict::from_inner(Array::from_fn(|_| f(iter.next().unwrap())))
    }

    pub fn try_map<F, U, E>(self, mut f: F) -> Result<RequiredDict<K, U>, E>
    where
        F: FnMut(V) -> Result<U, E>,
    {
        let mut iter = self.inner.into_iter();
        Ok(RequiredDict::from_inner(Array::try_from_fn(|_| {
            f(iter.next().unwrap())
        })?))
    }

    pub fn each_ref(&self) -> RequiredDict<K, &V> {
        let mut iter = self.inner.as_ref().iter();
        RequiredDict::from_inner(Array::from_fn(|_| iter.next().unwrap()))
    }

    pub fn each_mut(&mut self) -> RequiredDict<K, &mut V> {
        let mut iter = self.inner.as_mut().iter_mut();
        RequiredDict::from_inner(Array::from_fn(|_| iter.next().unwrap()))
    }

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
            if is_first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", K::VARIANTS[index], value)?;
            is_first = false;
        }
        write!(f, "}}")
    }
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

#[macro_export]
macro_rules! required_dict {
    ($($key:pat => $value:expr),* $(,)?) => {{
        $crate::RequiredDict::from_fn(|k| {
            match k { $($key => $value),* }
        })
    }};
}

#[macro_export]
macro_rules! try_required_dict {
    ($($key:pat => $value:expr),* $(,)?) => {{
        $crate::RequiredDict::try_from_fn(|k| {
            Ok(match k { $($key => $value),* })
        })
    }};
}
