use core::fmt::{Debug, Display};
use core::hash::{Hash, Hasher};
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Index, IndexMut};

use crate::dict_key::Array;
use crate::{DictKey, RequiredDict};

type Iter<'a, K, V> = OptionalIter<crate::required_dict::Iter<'a, K, Option<V>>>;
type IterMut<'a, K, V> = OptionalIter<crate::required_dict::IterMut<'a, K, Option<V>>>;
type IntoIter<K, V> = OptionalIter<crate::required_dict::IntoIter<K, Option<V>>>;

type Values<'a, K, V> = crate::iter::Values<Iter<'a, K, V>>;
type ValuesMut<'a, K, V> = crate::iter::Values<IterMut<'a, K, V>>;
type IntoValues<K, V> = crate::iter::Values<IntoIter<K, V>>;

/// A dictionary where keys may or may not have values
#[repr(transparent)]
pub struct OptionalDict<K: DictKey, V> {
    pub(crate) inner: K::Array<Option<V>>,
    pub(crate) phantom: PhantomData<K>,
}

impl<K: DictKey, V> OptionalDict<K, V> {
    /// Create a new empty OptionalDict
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub(crate) fn from_inner(inner: K::Array<Option<V>>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.as_ref().iter().filter(|&v| v.is_some()).count()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(K) -> Option<V>,
    {
        Self::from_inner(Array::from_fn(|i| f(K::from_index(i))))
    }

    #[inline]
    pub fn try_from_fn<F, E>(mut f: F) -> Result<Self, E>
    where
        F: FnMut(K) -> Result<Option<V>, E>,
    {
        Ok(Self::from_inner(Array::try_from_fn(|i| f(K::from_index(i)))?))
    }

    #[inline]
    pub fn map<F, U>(self, mut f: F) -> OptionalDict<K, U>
    where
        F: FnMut(V) -> U,
    {
        let mut iter = self.inner.into_iter();
        OptionalDict::from_inner(Array::from_fn(|_| iter.next().unwrap().map(&mut f)))
    }

    #[inline]
    pub fn try_map<F, U, E>(self, mut f: F) -> Result<OptionalDict<K, U>, E>
    where
        F: FnMut(V) -> Result<U, E>,
    {
        let mut iter = self.inner.into_iter();
        Ok(OptionalDict::from_inner(Array::try_from_fn(|_| {
            iter.next().unwrap().map(&mut f).transpose()
        })?))
    }

    #[inline]
    pub fn each_ref(&self) -> OptionalDict<K, &V> {
        let mut iter = self.inner.as_ref().iter();
        OptionalDict::from_inner(Array::from_fn(|_| iter.next().unwrap().as_ref()))
    }

    #[inline]
    pub fn each_mut(&mut self) -> OptionalDict<K, &mut V> {
        let mut iter = self.inner.as_mut().iter_mut();
        OptionalDict::from_inner(Array::from_fn(|_| iter.next().unwrap().as_mut()))
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.into_iter()
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        self.into_iter().into()
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.into_iter().into()
    }

    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        self.into_iter().into()
    }

    pub fn upgrade(self) -> Result<RequiredDict<K, V>, Self> {
        let is_filled = self.inner.as_ref().iter().all(|v| v.is_some());
        if is_filled {
            let mut iter = self.inner.into_iter();
            Ok(RequiredDict::from_inner(Array::from_fn(|_| {
                iter.next().unwrap().unwrap()
            })))
        } else {
            Err(self)
        }
    }
}

impl<K: DictKey, V> From<RequiredDict<K, Option<V>>> for OptionalDict<K, V> {
    #[inline]
    fn from(dict: RequiredDict<K, Option<V>>) -> Self {
        Self::from_inner(dict.inner)
    }
}

impl<K: DictKey, V> Default for OptionalDict<K, V> {
    #[inline]
    fn default() -> Self {
        Self::from_inner(Array::from_fn(|_| None))
    }
}

impl<K: DictKey, V: Clone> Clone for OptionalDict<K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from_inner(Array::from_fn(|i| self.inner.as_ref()[i].clone()))
    }
}

// Actually, `K::Array<Option<V>>: Copy` always holds when `V: Copy`
// (because `K::Array<Option<V>>` is an array), but Rust currently cannot recognize it.
impl<K: DictKey, V: Copy> Copy for OptionalDict<K, V> where K::Array<Option<V>>: Copy {}

impl<K: DictKey, V: PartialEq> PartialEq for OptionalDict<K, V> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref() == other.inner.as_ref()
    }
}

impl<K: DictKey, V: Eq> Eq for OptionalDict<K, V> {}

impl<K: DictKey, V: PartialOrd> PartialOrd for OptionalDict<K, V> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.inner.as_ref().partial_cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Ord> Ord for OptionalDict<K, V> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.as_ref().cmp(other.inner.as_ref())
    }
}

impl<K: DictKey, V: Hash> Hash for OptionalDict<K, V> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.as_ref().hash(state);
    }
}

impl<K: DictKey, V> Index<K> for OptionalDict<K, V> {
    type Output = Option<V>;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        &self.inner.as_ref()[key.as_index()]
    }
}

impl<K: DictKey, V> IndexMut<K> for OptionalDict<K, V> {
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        &mut self.inner.as_mut()[key.as_index()]
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
            if !is_first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", K::VARIANTS[index], value)?;
            is_first = false;
        }
        write!(f, "}}")
    }
}

trait IntoOption {
    type Value;

    fn into_option(self) -> Option<Self::Value>;
}

impl<T> IntoOption for Option<T> {
    type Value = T;

    #[inline]
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<'a, T> IntoOption for &'a Option<T> {
    type Value = &'a T;

    #[inline]
    fn into_option(self) -> Option<&'a T> {
        self.as_ref()
    }
}

impl<'a, T> IntoOption for &'a mut Option<T> {
    type Value = &'a mut T;

    #[inline]
    fn into_option(self) -> Option<&'a mut T> {
        self.as_mut()
    }
}

pub struct OptionalIter<I> {
    inner: I,
    len: usize,
}

impl<I, K, V, T> Iterator for OptionalIter<I>
where
    I: Iterator<Item = (K, T)>,
    T: IntoOption<Value = V>,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for (key, value) in &mut self.inner {
            if let Some(value) = value.into_option() {
                self.len -= 1;
                return Some((key, value));
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len
    }
}

impl<I, K, V, T> ExactSizeIterator for OptionalIter<I>
where
    I: Iterator<Item = (K, T)>,
    T: IntoOption<Value = V>,
{
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl<I, K, V, T> DoubleEndedIterator for OptionalIter<I>
where
    I: DoubleEndedIterator<Item = (K, T)>,
    T: IntoOption<Value = V>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some((key, value)) = self.inner.next_back() {
            if let Some(value) = value.into_option() {
                self.len -= 1;
                return Some((key, value));
            }
        }
        None
    }
}

impl<I, K, V, T> FusedIterator for OptionalIter<I>
where
    I: FusedIterator<Item = (K, T)>,
    T: IntoOption<Value = V>,
{
}

impl<I: Clone> Clone for OptionalIter<I> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            len: self.len,
        }
    }
}

impl<K: DictKey, V> IntoIterator for OptionalDict<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        OptionalIter {
            inner: RequiredDict::from(self).into_iter(),
            len,
        }
    }
}

impl<'a, K: DictKey, V> IntoIterator for &'a OptionalDict<K, V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        OptionalIter {
            inner: self.inner.as_ref().into(),
            len,
        }
    }
}

impl<'a, K: DictKey, V> IntoIterator for &'a mut OptionalDict<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        OptionalIter {
            inner: self.inner.as_mut().into(),
            len,
        }
    }
}

impl<K: DictKey, V> OptionalDict<K, MaybeUninit<V>> {
    /// Transpose `OptionalDict<K, MaybeUninit<V>>` into `MaybeUninit<OptionalDict<K, V>>`.
    #[inline]
    pub fn transpose(self) -> MaybeUninit<OptionalDict<K, V>> {
        // Safety: `MaybeUninit<OptionalDict<K, V>>` and `OptionalDict<K, MaybeUninit<V>>`
        // have the same layout because:
        // 1. `OptionalDict<K, V>` is `#[repr(transparent)]` over `K::Array<Option<V>>`
        // 2. `K::Array<Option<V>>` is effectively `[Option<V>; N]`
        // 3. `[Option<V>; N]` has the same layout as `[MaybeUninit<Option<V>>; N]`
        let this = ManuallyDrop::new(self);
        let ptr = &*this as *const Self as *const MaybeUninit<_>;
        unsafe { ptr.read() }
    }
}

#[macro_export]
macro_rules! optional_dict {
    ($($(|)? $head:ident $(::$tail:ident)* $(|$or_head:ident $(::$or_tail:ident)*)* => $value:expr),* $(,)?) => {{
        let mut dict = $crate::OptionalDict::new();
        $(
            dict[$head $(::$tail)*] = Some($value);
            $(dict[$or_head $(::$or_tail)*] = Some($value);)*
        )*
        dict
    }};

    ($($(|)? $head:ident $(::$tail:ident)* $(|$or_head:ident $(::$or_tail:ident)*)* => $value:expr,)* _ => $default:expr $(,)?) => {{
        let mut dict = $crate::OptionalDict::from_fn(|_| Some($default));
        $(
            dict[$head $(::$tail)*] = Some($value);
            $(dict[$or_head $(::$or_tail)*] = Some($value);)*
        )*
        dict
    }};
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::de::{DeserializeSeed, MapAccess, Visitor};
    use serde::ser::SerializeMap;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    struct KeyMatcher<K>(PhantomData<K>);

    impl<'de, K: DictKey> DeserializeSeed<'de> for KeyMatcher<K> {
        type Value = Option<usize>;

        #[inline]
        fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_str(self)
        }
    }

    impl<'de, K: DictKey> Visitor<'de> for KeyMatcher<K> {
        type Value = Option<usize>;

        #[inline]
        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            f.write_str("a string key")
        }

        #[inline]
        fn visit_str<E: serde::de::Error>(self, str: &str) -> Result<Self::Value, E> {
            Ok(K::VARIANTS.iter().position(|&name| name == str))
        }
    }

    struct DictVisitor<K, V>(PhantomData<(K, V)>);

    impl<K, V> DictVisitor<K, V> {
        #[inline]
        fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<'de, K: DictKey, V: Deserialize<'de>> Visitor<'de> for DictVisitor<K, V> {
        type Value = OptionalDict<K, V>;

        #[inline]
        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("a map with optional keys")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut dict = OptionalDict::<K, V>::new();
            while let Some(key) = map.next_key_seed(KeyMatcher(PhantomData::<K>))? {
                let value = map.next_value()?;
                // ignore unknown keys
                if let Some(index) = key {
                    dict.inner.as_mut()[index] = Some(value);
                }
            }
            Ok(dict)
        }
    }

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

#[cfg(feature = "std")]
mod std_impl {
    extern crate std;

    use std::collections::{BTreeMap, HashMap};

    use super::*;

    impl<K: DictKey + Eq + Hash, V> From<HashMap<K, V>> for OptionalDict<K, V> {
        #[inline]
        fn from(mut map: HashMap<K, V>) -> Self {
            OptionalDict::from_fn(|key| map.remove(&key))
        }
    }

    impl<K: DictKey + Ord, V> From<BTreeMap<K, V>> for OptionalDict<K, V> {
        #[inline]
        fn from(mut map: BTreeMap<K, V>) -> Self {
            OptionalDict::from_fn(|key| map.remove(&key))
        }
    }
}
