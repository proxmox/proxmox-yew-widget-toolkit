use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput};
use syn::{Attribute, Error, Fields, Ident, Path, Result, Token};

#[derive(Debug)]
pub(crate) struct WidgetSetup {
    pwt_crate_name: Option<Ident>,
    component_name: Option<Path>,
    is_input: bool,
    is_container: bool,
    is_element: bool,
    is_svg: bool,
}

impl Parse for WidgetSetup {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pwt_crate_name = None;
        let mut component_name = None;
        let mut is_input = false;
        let mut is_container = false;
        let mut is_element = false;
        let mut is_svg = false;

        loop {
            if input.is_empty() {
                break;
            }
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![@]) {
                let _: Token![@] = input.parse()?;
                let mixin: Ident = input.parse()?;
                if mixin == "input" {
                    is_input = true;
                } else if mixin == "container" {
                    is_container = true;
                } else if mixin == "element" {
                    is_element = true;
                } else if mixin == "svg" {
                    is_svg = true;
                } else {
                    return Err(Error::new(mixin.span(), "no such widget mixin"));
                }

                if input.is_empty() {
                    break;
                }
            } else if lookahead.peek(Token![!]) {
                pwt_crate_name = Some(input.parse()?);
            } else {
                let opt: Ident = input.parse()?;
                let _: Token![=] = input.parse()?;
                match opt.to_string().as_ref() {
                    "pwt" => {
                        let name: Ident = input.call(Ident::parse_any)?;
                        if pwt_crate_name.is_some() {
                            return Err(Error::new(name.span(), "multiple pwt name definitions"));
                        }
                        pwt_crate_name = Some(name);
                    }
                    "comp" => {
                        let path: Path = input.parse()?;
                        if component_name.is_some() {
                            return Err(Error::new(path.span(), "multiple component definitions"));
                        }
                        component_name = Some(path);
                    }
                    _ => {
                        return Err(Error::new(opt.span(), "unknown widget option"));
                    }
                }
            }
            if input.is_empty() {
                break;
            }
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                break;
            }
        }

        Ok(WidgetSetup {
            pwt_crate_name,
            component_name,
            is_input,
            is_container,
            is_element,
            is_svg,
        })
    }
}

pub(crate) fn handle_widget_struct(setup: &WidgetSetup, input: TokenStream) -> TokenStream {
    let widget = parse_macro_input!(input as DeriveInput);

    derive_widget(setup, widget)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn has_property_derive(attrs: &Vec<Attribute>) -> Result<bool> {
    // SIGH!!
    for attr in attrs {
        if attr.style != syn::AttrStyle::Outer {
            continue;
        }
        if let Some(ident) = attr.path().get_ident() {
            if ident != "derive" {
                continue;
            }
            let mut has_properties = false;
            attr.parse_nested_meta(|nested| {
                if nested.path.is_ident("Properties") {
                    has_properties = true;
                }
                Ok(())
            })?;

            return Ok(has_properties);
        }
    }

    Ok(false)
}

fn derive_widget(setup: &WidgetSetup, widget: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = widget;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let has_property_derive = has_property_derive(&attrs)?;

    let pwt: Ident = setup.pwt_crate_name.clone().unwrap_or(format_ident!("pwt"));

    let fields = match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields,
            _ => {
                return Err(Error::new(
                    data.struct_token.span,
                    "expected `struct` with named fields",
                ));
            }
        },
        Data::Enum(data) => {
            return Err(Error::new(data.enum_token.span, "expected `struct`"));
        }
        Data::Union(data) => {
            return Err(Error::new(data.union_token.span, "expected `struct`"));
        }
    };

    let fields = fields.named; // remove braces

    let mut opt_fields: Vec<_> = Vec::new();

    let prop_or_default = if has_property_derive {
        quote! { #[prop_or_default] }
    } else {
        quote! {}
    };

    if setup.is_input {
        opt_fields.push(quote! {
            #[doc(hidden)]
            #prop_or_default
            pub input_props: #pwt::props::FieldStdProps,
        });
    }

    if setup.is_container {
        opt_fields.push(quote! {
            #[doc(hidden)]
            #prop_or_default
            pub children: Vec<::yew::virtual_dom::VNode>,
        });
    }

    if setup.is_element {
        opt_fields.push(quote! {
            #[doc(hidden)]
            #prop_or_default
            pub listeners: #pwt::props::ListenersWrapper,
        });
    }

    let mut output = quote! {
        #(#attrs)*
        #vis struct #ident #generics {

            #[doc(hidden)]
            #prop_or_default
            pub std_props: #pwt::props::WidgetStdProps,

            #(#opt_fields)*

            #fields
        }
    };

    output.extend(quote! {
        impl #impl_generics #pwt::props::WidgetBuilder for #ident #ty_generics #where_clause {
            fn as_std_props_mut(&mut self) -> &mut #pwt::props::WidgetStdProps {
                &mut self.std_props
            }
        }
    });

    if !setup.is_svg {
        output.extend(quote!{
            impl #impl_generics #pwt::props::AsClassesMut for #ident #ty_generics #where_clause {
                fn as_classes_mut(&mut self) -> &mut ::yew::Classes {
                    &mut self.std_props.class
                }
            }
            impl #impl_generics #pwt::props::CssMarginBuilder for #ident #ty_generics #where_clause {}
            impl #impl_generics #pwt::props::CssPaddingBuilder for #ident #ty_generics #where_clause {}
            impl #impl_generics #pwt::props::CssBorderBuilder for #ident #ty_generics #where_clause {}
        });
    }

    if setup.is_element {
        output.extend(quote! {
            impl #impl_generics #pwt::props::EventSubscriber for #ident #ty_generics #where_clause {
                fn as_listeners_mut(&mut self) -> &mut #pwt::props::ListenersWrapper {
                    &mut self.listeners
                }
            }
        });
    }

    if setup.is_container {
        output.extend(quote!{
            impl #impl_generics #pwt::props::ContainerBuilder for #ident #ty_generics #where_clause {
                fn as_children_mut(&mut self) -> &mut Vec<::yew::virtual_dom::VNode> {
                    &mut self.children
                }
            }
        });
    }

    if setup.is_input {
        output.extend(quote! {
            impl #impl_generics #pwt::props::FieldBuilder for #ident #ty_generics #where_clause {
                fn as_input_props_mut(&mut self) -> &mut #pwt::props::FieldStdProps {
                    &mut self.input_props
                }
                fn as_input_props(&self) -> & #pwt::props::FieldStdProps {
                    &self.input_props
                }
            }
        });
    }

    if let Some(component_name) = &setup.component_name {
        output.extend(quote!{
            impl #impl_generics Into<::yew::virtual_dom::VNode> for #ident #ty_generics #where_clause {
                fn into(self) -> ::yew::virtual_dom::VNode {
                    let key = self.std_props.key.clone();
                    let comp = ::yew::virtual_dom::VComp::new::<#component_name>(
                        ::std::rc::Rc::new(self), key,
                    );
                    ::yew::virtual_dom::VNode::from(comp)
                }
            }
        });
    } else {
        output.extend(quote!{
            impl #impl_generics Into<::yew::virtual_dom::VNode> for #ident #ty_generics #where_clause {
                fn into(self) -> ::yew::virtual_dom::VNode {
                    let vtag: ::yew::virtual_dom::VTag = self.into();
                    ::yew::virtual_dom::VNode::from(vtag)
                }
            }
        });
    }

    //eprintln!("TEST {}", output);
    Ok(output)
}
