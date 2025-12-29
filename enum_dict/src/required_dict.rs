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
    pub fn len(&self) -> usize {
        self.inner.as_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn downgrade(self) -> OptionalDict<K, V> {
        let mut iter = self.inner.into_iter();
        OptionalDict {
            inner: Array::from_fn(|_| Some(iter.next().unwrap())),
            phantom: PhantomData,
        }
    }
}

impl<K: DictKey, V> RequiredDict<K, V> {
    #[inline]
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(K) -> V,
    {
        Self {
            inner: Array::from_fn(|i| f(K::from_index(i))),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn try_from_fn<F, E>(mut f: F) -> Result<Self, E>
    where
        F: FnMut(K) -> Result<V, E>,
    {
        Ok(Self {
            inner: Array::try_from_fn(|i| f(K::from_index(i)))?,
            phantom: PhantomData,
        })
    }
}

impl<K: DictKey, V: Default> Default for RequiredDict<K, V> {
    fn default() -> Self {
        Self {
            inner: Array::from_fn(|_| V::default()),
            phantom: PhantomData,
        }
    }
}

impl<K: DictKey, V: Clone> Clone for RequiredDict<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Array::from_fn(|i| self.inner.as_ref()[i].clone()),
            phantom: PhantomData,
        }
    }
}

impl<K: DictKey, V: PartialEq> PartialEq for RequiredDict<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref() == other.inner.as_ref()
    }
}

impl<K: DictKey, V: Eq> Eq for RequiredDict<K, V> {}

impl<K: DictKey, V: PartialOrd> PartialOrd for RequiredDict<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.inner.as_ref().partial_cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Ord> Ord for RequiredDict<K, V> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.as_ref().cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Hash> Hash for RequiredDict<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.as_ref().hash(state);
    }
}

impl<K: DictKey, V> Index<K> for RequiredDict<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        &self.inner.as_ref()[key.into_index()]
    }
}

impl<K: DictKey, V> IndexMut<K> for RequiredDict<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner.as_mut()[key.into_index()]
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
    use crate::dict_key::DictVisitor;

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
            deserializer
                .deserialize_map(DictVisitor::<K, V>::new())?
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
