use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use crate::DictKey;
use crate::dict_key::Array;

/// A dictionary where keys may or may not have values
pub struct OptionalDict<K: DictKey, V> {
    inner: K::Array<Option<V>>,
    phantom: PhantomData<K>,
}

impl<K: DictKey, V> OptionalDict<K, V> {
    /// Create a new empty OptionalDict
    pub fn new() -> Self {
        Default::default()
    }
}

impl<K: DictKey, V> OptionalDict<K, V> {
    pub fn len(&self) -> usize {
        self.inner.as_ref().iter().filter(|&v| v.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> OptionalDict<K, V>
where
    K: DictKey + FromStr,
    K::Err: Debug,
{
    pub fn from_fn<F>(f: F) -> Self
    where
        F: Fn(K) -> Option<V>,
    {
        Self {
            // SAFETY: K::VARIANTS are all valid keys
            inner: Array::from_fn(|i| f(K::VARIANTS[i].parse().unwrap())),
            phantom: PhantomData,
        }
    }

    pub fn try_from_fn<F, E>(f: F) -> Result<Self, E>
    where
        F: Fn(K) -> Result<Option<V>, E>,
    {
        Ok(Self {
            // SAFETY: K::VARIANTS are all valid keys
            inner: Array::try_from_fn(|i| f(K::VARIANTS[i].parse().unwrap()))?,
            phantom: PhantomData,
        })
    }
}

impl<K, V> Default for OptionalDict<K, V>
where
    K: DictKey,
{
    fn default() -> Self {
        Self {
            inner: Array::from_fn(|_| None),
            phantom: PhantomData,
        }
    }
}

impl<K: DictKey, V: Clone> Clone for OptionalDict<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Array::from_fn(|i| self.inner.as_ref()[i].clone()),
            phantom: PhantomData,
        }
    }
}

impl<K: DictKey, V: PartialEq> PartialEq for OptionalDict<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref() == other.inner.as_ref()
    }
}

impl<K: DictKey, V: Eq> Eq for OptionalDict<K, V> {}

impl<K: DictKey, V: PartialOrd> PartialOrd for OptionalDict<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.as_ref().partial_cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Ord> Ord for OptionalDict<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.as_ref().cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Hash> Hash for OptionalDict<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.as_ref().hash(state);
    }
}

impl<K: DictKey, V> Index<K> for OptionalDict<K, V> {
    type Output = Option<V>;

    fn index(&self, key: K) -> &Self::Output {
        &self.inner.as_ref()[key.variant_index()]
    }
}

impl<K: DictKey, V> IndexMut<K> for OptionalDict<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner.as_mut()[key.variant_index()]
    }
}

impl<K: DictKey, V: Debug> Debug for OptionalDict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.inner
                    .as_ref()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, value)| value.as_ref().map(|value| (K::VARIANTS[index], value))),
            )
            .finish()
    }
}

impl<K: DictKey, V: Display> Display for OptionalDict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut is_first = true;
        for (index, value) in self.inner.as_ref().iter().enumerate() {
            let Some(value) = value else {
                continue;
            };
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

    impl<K: DictKey, V: Serialize> Serialize for OptionalDict<K, V> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(Some(self.inner.as_ref().len()))?;
            for (index, value) in self.inner.as_ref().iter().enumerate() {
                if let Some(value) = value {
                    map.serialize_entry(K::VARIANTS[index], value)?;
                }
            }
            map.end()
        }
    }

    impl<'de, K: DictKey, V: Deserialize<'de>> Deserialize<'de> for OptionalDict<K, V> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let vec = deserializer.deserialize_map(DictVisitor::<K, V>::new())?;
            let mut iter = vec.into_iter();
            Ok(Self {
                inner: Array::from_fn(|_| iter.next().unwrap()),
                phantom: PhantomData,
            })
        }
    }
}

#[macro_export]
macro_rules! optional_dict {
    ($($key:pat => $value:expr),* $(,)?) => {{
        $crate::OptionalDict::from_fn(|k| {
            match k {
                $($key => Some($value)),* ,
                _ => None,
            }
        })
    }};
}
