use std::fmt::Debug;
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

impl<K, V: Clone> Clone for RequiredDict<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
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

impl<K, V: PartialEq> PartialEq for RequiredDict<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
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
