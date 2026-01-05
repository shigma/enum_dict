#[allow(unused_imports)]
use enum_dict::DictKey;

#[rustfmt::skip]
#[derive(DictKey)]
#[enum_dict(rename_all = "lowercase")]
pub enum Foo {
    Alpha,
    Beta,
}

#[rustfmt::skip]
#[derive(DictKey)]
#[repr(i8)]
pub enum Bar {
    Alpha = 2,
    Beta = 1,
}
