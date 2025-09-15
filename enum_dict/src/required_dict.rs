use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use crate::DictKey;

/// A dictionary that requires all keys to have values
pub struct RequiredDict<K, V> {
    inner: Vec<V>,
    phantom: PhantomData<K>,
}

impl<K, V> RequiredDict<K, V> {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: DictKey, V: Default> Default for RequiredDict<K, V> {
    fn default() -> Self {
        Self {
            inner: K::FIELDS.iter().map(|_| V::default()).collect(),
            phantom: PhantomData,
        }
    }
}

impl<K, V: Clone> Clone for RequiredDict<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

impl<K, V: PartialEq> PartialEq for RequiredDict<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K, V: Eq> Eq for RequiredDict<K, V> {}

impl<K, V: PartialOrd> PartialOrd for RequiredDict<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<K, V: Ord> Ord for RequiredDict<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<K, V: Hash> Hash for RequiredDict<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<K: DictKey, V> Index<K> for RequiredDict<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        &self.inner[key.into_usize()]
    }
}

impl<K: DictKey, V> IndexMut<K> for RequiredDict<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner[key.into_usize()]
    }
}

impl<K: DictKey, V: Debug> Debug for RequiredDict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.inner
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (K::FIELDS[index], value)),
            )
            .finish()
    }
}

impl<K: DictKey, V: Display> Display for RequiredDict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut is_first = true;
        for (index, value) in self.inner.iter().enumerate() {
            if is_first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", K::FIELDS[index], value)?;
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
            let mut map = serializer.serialize_map(Some(self.inner.len()))?;
            for (index, value) in self.inner.iter().enumerate() {
                map.serialize_entry(K::FIELDS[index], value)?;
            }
            map.end()
        }
    }

    impl<'de, K: DictKey, V: Deserialize<'de>> Deserialize<'de> for RequiredDict<K, V> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let vec = deserializer.deserialize_map(DictVisitor::<K, V>::new())?;

            // Check for missing keys
            let mut missing_keys = Vec::new();
            for (index, &name) in K::FIELDS.iter().enumerate() {
                if vec[index].is_none() {
                    missing_keys.push(name);
                }
            }
            if !missing_keys.is_empty() {
                return Err(serde::de::Error::custom(format!(
                    "Missing keys: {}",
                    missing_keys.join(", ")
                )));
            }

            Ok(Self {
                inner: vec.into_iter().map(Option::unwrap).collect(),
                phantom: PhantomData,
            })
        }
    }
}
