use core::fmt::{Debug, Display};
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use crate::dict_key::Array;
use crate::{DictKey, RequiredDict};

/// A dictionary where keys may or may not have values
pub struct OptionalDict<K: DictKey, V> {
    pub(crate) inner: K::Array<Option<V>>,
    pub(crate) phantom: PhantomData<K>,
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

    pub fn upgrade(self) -> Result<RequiredDict<K, V>, Self> {
        let is_filled = self.inner.as_ref().iter().all(|v| v.is_some());
        if is_filled {
            let mut iter = self.inner.into_iter();
            Ok(RequiredDict {
                inner: Array::from_fn(|_| iter.next().unwrap().unwrap()),
                phantom: PhantomData,
            })
        } else {
            Err(self)
        }
    }
}

impl<K: DictKey, V> OptionalDict<K, V> {
    #[inline]
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(K) -> Option<V>,
    {
        Self {
            inner: Array::from_fn(|i| f(K::from_index(i))),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn try_from_fn<F, E>(mut f: F) -> Result<Self, E>
    where
        F: FnMut(K) -> Result<Option<V>, E>,
    {
        Ok(Self {
            inner: Array::try_from_fn(|i| f(K::from_index(i)))?,
            phantom: PhantomData,
        })
    }
}

impl<K: DictKey, V> Default for OptionalDict<K, V> {
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
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.inner.as_ref().partial_cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Ord> Ord for OptionalDict<K, V> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
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
        &self.inner.as_ref()[key.into_index()]
    }
}

impl<K: DictKey, V> IndexMut<K> for OptionalDict<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner.as_mut()[key.into_index()]
    }
}

impl<K: DictKey, V: Debug> Debug for OptionalDict<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
        #[inline]
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_map(DictVisitor::<K, V>::new())
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

#[macro_export]
macro_rules! try_optional_dict {
    ($($key:pat => $value:expr),* $(,)?) => {{
        $crate::OptionalDict::try_from_fn(|k| {
            Ok(match k {
                $($key => Some($value)),* ,
                _ => None,
            })
        })
    }};
}
