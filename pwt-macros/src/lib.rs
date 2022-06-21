use proc_macro::TokenStream;
use syn::parse_macro_input;

mod widget;
use widget::*;

#[proc_macro_attribute]
pub fn widget(attr: TokenStream, item: TokenStream) -> TokenStream {
    //eprintln!("attr: \"{}\"", attr.to_string());
    let setup = parse_macro_input!(attr as WidgetSetup);

    handle_widget_struct(&setup, item)
}
