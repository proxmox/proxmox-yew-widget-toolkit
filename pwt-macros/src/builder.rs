use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput};

use syn::{Error, Fields, Result};

pub(crate) fn handle_builder_struct(input: TokenStream) -> TokenStream {
    let builder = parse_macro_input!(input as DeriveInput);

    derive_builder(builder)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_builder(builder: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = builder;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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

    let mut fields = fields.named; // remove braces

    //    let mut setters = Vec::new();
    let mut builder = Vec::new();
    for field in fields.iter_mut() {
        for (i, attr) in field.attrs.iter_mut().enumerate() {
            if let Ok(meta) = attr.parse_meta() {
                if meta.path().is_ident("builder") {
                    builder.push((field.clone(), meta.clone()));
                    field.attrs.remove(i);
                    break;
                }
            }
        }
    }

    let mut output = quote! {
        #(#attrs)*
        #vis struct #ident #generics {
            #fields
        }
    };

    for (field, attr) in builder {
        let field_ident = field.ident.unwrap();
        let field_name = field_ident.to_string();
        let field_type = field.ty;
        let setter = format_ident!("set_{}", field_name);
        let (builder_doc, setter_doc) = match field.vis {
            syn::Visibility::Public(_) => {
                let link =
                    format!("[`{field_name}`](struct.{ident}.html#structfield.{field_name})");
                let setter_doc = format!("Set {link}");
                let builder_doc = format!("Builder style method to set {link}");
                (builder_doc, setter_doc)
            }
            _ => {
                let mut doc = String::new();
                for attr in field.attrs {
                    if attr.path.is_ident("doc") {
                        if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
                            if let syn::Lit::Str(text) = meta.lit {
                                doc.push_str(&text.value());
                                doc.push('\n');
                            }
                        }
                    }
                }
                let setter_doc = format!("Set {field_name}\n\n{doc}");
                let builder_doc = format!("Builder style method for [`Self::{setter}`]");
                (builder_doc, setter_doc)
            }
        };

        match attr {
            syn::Meta::Path(_) => {
                output.extend(quote! {
                    impl #impl_generics #ident #ty_generics #where_clause {
                        #[doc = #setter_doc]
                        pub fn #setter(&mut self, #field_ident: #field_type) {
                            self.#field_ident = #field_ident;
                        }

                        #[doc = #builder_doc]
                        pub fn #field_ident(mut self, #field_ident: #field_type) -> Self {
                            self.#setter(#field_ident);
                            self
                        }
                    }
                });
            }
            syn::Meta::List(syn::MetaList { nested: list, .. }) => {
                let mut iter = list.into_iter();
                let into_trait = iter
                    .next()
                    .expect("List must not contain the generic trait.");
                let into_fn = iter.next().expect("List must contain the 'into' function");
                let (param_type, convert) = match iter.next() {
                    Some(default) => (
                        quote! {impl #into_trait<Option<#field_type>>},
                        quote! {#field_ident.#into_fn().unwrap_or(#default)},
                    ),
                    None => (
                        quote! {impl #into_trait<#field_type>},
                        quote! {#field_ident.#into_fn()},
                    ),
                };

                output.extend(quote! {
                    impl #impl_generics #ident #ty_generics #where_clause {
                        #[doc = #setter_doc]
                        pub fn #setter(&mut self, #field_ident: #param_type) {
                            self.#field_ident = #convert;
                        }

                        #[doc = #builder_doc]
                        pub fn #field_ident(mut self, #field_ident: #param_type) -> Self {
                            self.#setter(#field_ident);
                            self
                        }
                    }
                });
            }
            syn::Meta::NameValue(_) => unreachable!("not implemented"),
        }
    }

    Ok(output)
}
