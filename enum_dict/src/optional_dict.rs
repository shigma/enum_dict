use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use crate::DictKey;

/// A dictionary where keys may or may not have values
pub struct OptionalDict<K, V> {
    inner: Vec<Option<V>>,
    phantom: PhantomData<K>,
}

impl<K, V> OptionalDict<K, V> {
    /// Create a new empty OptionalDict
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.inner.iter().filter(|&v| v.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V, F> From<F> for OptionalDict<K, V>
where
    K: DictKey + FromStr,
    K::Err: Debug,
    F: Fn(K) -> Option<V>,
{
    fn from(f: F) -> Self {
        Self {
            // SAFETY: K::VARIANTS are all valid keys
            inner: K::VARIANTS.iter().map(|s| f(s.parse().unwrap())).collect(),
            phantom: PhantomData,
        }
    }
}

impl<K, V> Default for OptionalDict<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<K, V: Clone> Clone for OptionalDict<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

impl<K, V: PartialEq> PartialEq for OptionalDict<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K, V: Eq> Eq for OptionalDict<K, V> {}

impl<K, V: PartialOrd> PartialOrd for OptionalDict<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<K, V: Ord> Ord for OptionalDict<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<K, V: Hash> Hash for OptionalDict<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<K: DictKey, V> Index<K> for OptionalDict<K, V> {
    type Output = Option<V>;

    fn index(&self, key: K) -> &Self::Output {
        &self.inner[key.variant_index()]
    }
}

impl<K: DictKey, V> IndexMut<K> for OptionalDict<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner[key.variant_index()]
    }
}

impl<K: DictKey, V: Debug> Debug for OptionalDict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.inner
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
        for (index, value) in self.inner.iter().enumerate() {
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
            let mut map = serializer.serialize_map(Some(self.inner.len()))?;
            for (index, value) in self.inner.iter().enumerate() {
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

            Ok(Self {
                inner: vec,
                phantom: PhantomData,
            })
        }
    }
}

#[macro_export]
macro_rules! optional_dict {
    ($($key:pat => $value:expr),* $(,)?) => {{
        $crate::OptionalDict::from(|k| {
            match k {
                $($key => Some($value)),* ,
                _ => None,
            }
        })
    }};
}
