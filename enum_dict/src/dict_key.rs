use std::marker::PhantomData;

/// Trait for types that can be used as dictionary keys
pub trait DictKey {
    const VARIANTS: &'static [&'static str];

    /// Convert to usize index
    fn into_usize(self) -> usize;
}

pub struct DictVisitor<K, V>(PhantomData<(K, V)>);

impl<K, V> DictVisitor<K, V> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use std::fmt;

    use serde::Deserialize;
    use serde::de::{MapAccess, Visitor};

    use super::*;

    impl<'de, K: DictKey, V: Deserialize<'de>> Visitor<'de> for DictVisitor<K, V> {
        type Value = Vec<Option<V>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with optional keys")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut vec = K::VARIANTS.iter().map(|_| None).collect::<Vec<_>>();
            while let Some((key, value)) = map.next_entry::<String, V>()? {
                // ignore unknown keys
                for (index, &name) in K::VARIANTS.iter().enumerate() {
                    if name == key {
                        vec[index] = Some(value);
                        break;
                    }
                }
            }
            Ok(vec)
        }
    }
}
