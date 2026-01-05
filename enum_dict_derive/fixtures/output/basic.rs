#[allow(unused_imports)]
use enum_dict::DictKey;
#[rustfmt::skip]
pub enum Foo {
    Alpha,
    Beta,
}
#[rustfmt::skip]
#[automatically_derived]
impl ::enum_dict::DictKey for Foo {
    type Array<T> = [T; 2usize];
    const VARIANTS: &'static [&'static str] = &["alpha", "beta"];
    fn from_index(index: usize) -> Self {
        match index {
            0usize => Self::Alpha,
            1usize => Self::Beta,
            _ => panic!("invalid index for DictKey"),
        }
    }
    #[inline]
    fn as_index(&self) -> usize {
        match self {
            Self::Alpha => 0usize,
            Self::Beta => 1usize,
        }
    }
}
#[rustfmt::skip]
#[repr(i8)]
pub enum Bar {
    Alpha = 2,
    Beta = 1,
}
#[rustfmt::skip]
#[automatically_derived]
impl ::enum_dict::DictKey for Bar {
    type Array<T> = [T; 2usize];
    const VARIANTS: &'static [&'static str] = &["Alpha", "Beta"];
    fn from_index(index: usize) -> Self {
        match index {
            0usize => Self::Alpha,
            1usize => Self::Beta,
            _ => panic!("invalid index for DictKey"),
        }
    }
    #[inline]
    fn as_index(&self) -> usize {
        match self {
            Self::Alpha => 0usize,
            Self::Beta => 1usize,
        }
    }
}
