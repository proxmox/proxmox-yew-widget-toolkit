use pwt_macros::builder;

#[builder]
struct InvalidTypes {
    #[builder(IntoFoo, into_foo)]
    wrong_type: i32,
}

fn main() {}
