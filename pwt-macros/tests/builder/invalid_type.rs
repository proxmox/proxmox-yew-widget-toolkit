use pwt_macros::builder;

#[builder]
struct InvalidTypes {
    #[builder(Into, into, 0.0)]
    wrong_type: i32,
}

fn main() {}
