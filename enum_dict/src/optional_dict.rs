use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use crate::DictKey;

/// A dictionary where keys may or may not have values
pub struct OptionalDict<K, V> {
    inner: Vec<Option<V>>,
    phantom: PhantomData<K>,
}

impl<K, V> Default for OptionalDict<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
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

impl<K, V: Clone> Clone for OptionalDict<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

impl<K: DictKey, V: Debug> Debug for OptionalDict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.inner
                    .iter()
                    .enumerate()
                    .filter_map(|(index, value)| value.as_ref().map(|value| (K::FIELDS[index], value))),
            )
            .finish()
    }
}

impl<K, V: PartialEq> PartialEq for OptionalDict<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K: DictKey, V> Index<K> for OptionalDict<K, V> {
    type Output = Option<V>;

    fn index(&self, key: K) -> &Self::Output {
        &self.inner[key.into_usize()]
    }
}

impl<K: DictKey, V> IndexMut<K> for OptionalDict<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner[key.into_usize()]
    }
}
