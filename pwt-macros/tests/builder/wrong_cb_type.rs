use pwt_macros::builder;

struct Callback<T> {
    field: T,
}
trait IntoEventCallback<T> {
    fn into_event_callback(self) -> Option<Callback<T>>;
}

#[builder]
struct InvalidTypes {
    #[builder_cb(IntoEventCallback, into_event_callback, f32)]
    cb: Option<Callback<i32>>,
}

fn main() {}
