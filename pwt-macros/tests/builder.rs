use std::marker::PhantomData;

use pwt_macros::builder;

// bogus traits to test the macro
trait IntoSelf<T> {
    fn into_self(self) -> T;
}

impl<T> IntoSelf<T> for T {
    fn into_self(self) -> T {
        self
    }
}

#[derive(Debug, PartialEq)]
struct Callback<IN, OUT = ()> {
    _in: PhantomData<IN>,
    _out: PhantomData<OUT>,
}

impl<IN, OUT, F: Fn(IN) -> OUT + 'static> From<F> for Callback<IN, OUT> {
    fn from(_value: F) -> Self {
        Callback {
            _in: PhantomData,
            _out: PhantomData,
        }
    }
}

trait IntoCallback<T> {
    fn into_callback(self) -> Option<Callback<T>>;
}

impl<T: Fn(EVENT) + 'static, EVENT> IntoCallback<EVENT> for T {
    fn into_callback(self) -> Option<Callback<EVENT>> {
        Some(Callback::from(self))
    }
}

#[builder]
struct Foo {
    #[builder]
    /// normal field
    pub normal_field: i32,

    #[builder(IntoSelf, into_self)]
    /// into field
    into_field: i32,

    #[builder(IntoSelf, into_self, 42)]
    /// into with default
    into_with_default: i32,

    #[builder_cb(IntoCallback, into_callback, i32)]
    /// callback with normal type
    cb: Option<Callback<i32>>,

    #[builder_cb(IntoCallback, into_callback, Option<i32>)]
    /// callback with option type
    cb_with_option: Option<Callback<Option<i32>>>,
}

impl Foo {
    /// [Self::new]
    fn new() -> Self {
        Self {
            normal_field: 0,
            into_field: 0,
            into_with_default: 0,
            cb: None,
            cb_with_option: None,
        }
    }
}

#[test]
fn builder_test() {
    let mut element = Foo::new()
        .normal_field(42)
        .into_field(42)
        .into_with_default(None)
        .cb(|_i: i32| ())
        .cb_with_option(|_i: Option<i32>| ());

    assert_eq!(element.normal_field, 42);
    assert_eq!(element.into_field, 42);
    assert_eq!(element.into_with_default, 42);
    assert_eq!(element.cb, Some(Callback::from(|_| ())));
    assert_eq!(element.cb_with_option, Some(Callback::from(|_| ())));

    element.set_normal_field(1337);
    assert_eq!(element.normal_field, 1337);

    element.set_into_field(1337);
    assert_eq!(element.into_field, 1337);

    element.set_into_with_default(Some(1337));
    assert_eq!(element.into_with_default, 1337);

    element.set_into_with_default(None);
    assert_eq!(element.into_with_default, 42);

    element.set_cb(|_: i32| ());
    assert_eq!(element.cb, Some(Callback::from(|_| ())));

    element.set_cb_with_option(|_: Option<i32>| ());
    assert_eq!(element.cb_with_option, Some(Callback::from(|_| ())));
}
