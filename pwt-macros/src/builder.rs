use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};

use syn::spanned::Spanned;
use syn::{parse::Parse, parse_macro_input, Data, DeriveInput, Token};

use syn::{parenthesized, Error, Fields, Result};

pub(crate) fn handle_builder_struct(input: TokenStream) -> TokenStream {
    let builder = parse_macro_input!(input as DeriveInput);

    derive_builder(builder)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

enum BuilderType {
    Field,
    Callback,
}

fn parse_comma(input: syn::parse::ParseStream) -> Result<()> {
    if !parse_optional_comma(input)? {
        return Err(Error::new(input.span(), "expected ','"));
    }
    Ok(())
}

fn parse_optional_comma(input: syn::parse::ParseStream) -> Result<bool> {
    if input.is_empty() {
        return Ok(false);
    }
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![,]) {
        let _: Token![,] = input.parse().unwrap();
    } else {
        let err = lookahead.error();
        return Err(Error::new(err.span(), "expected ','"));
    }
    Ok(true)
}

// options for normal fields with the #[builder] attribute
struct FieldOptions {
    into_trait: syn::Type,
    into_fn: syn::Ident,
    default_value: Option<syn::Lit>,
}

impl Parse for FieldOptions {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let _: syn::Ident = input.parse()?;

        let content;
        parenthesized!(content in input);

        let into_trait: syn::Type = content
            .parse()
            .map_err(|err| Error::new(input.span(), format!("expected Into trait: {err}")))?;

        parse_comma(&content)?;

        let into_fn: syn::Ident = content
            .parse()
            .map_err(|err| Error::new(input.span(), format!("expected into function: {err}")))?;
        let default_value = if !parse_optional_comma(&content)? {
            None
        } else {
            Some(
                content
                    .parse()
                    .map_err(|err| Error::new(err.span(), "expected default literal"))?,
            )
        };

        Ok(Self {
            into_trait,
            into_fn,
            default_value,
        })
    }
}

// options for callback fields with the #[builder_cb] attribute
struct CallbackOptions {
    into_trait: syn::Type,
    into_fn: syn::Ident,
    inner_type: syn::Type,
}

impl Parse for CallbackOptions {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let _: syn::Ident = input.parse()?;

        let content;
        parenthesized!(content in input);

        let into_trait: syn::Type = content
            .parse()
            .map_err(|err| Error::new(err.span(), format!("expected Into trait: {err}")))?;

        parse_comma(&content).map_err(|err| Error::new(err.span(), "missing into_fn"))?;

        let into_fn: syn::Ident = content
            .parse()
            .map_err(|err| Error::new(err.span(), format!("expected into function: {err}")))?;

        parse_comma(&content).map_err(|err| Error::new(err.span(), "missing inner type"))?;

        let inner_type: syn::Type = content.parse().map_err(|err| {
            Error::new(err.span(), format!("expected inner callback type: {err}"))
        })?;

        Ok(Self {
            into_trait,
            into_fn,
            inner_type,
        })
    }
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
            if attr.path().is_ident("builder") {
                let attr = attr.clone();
                builder.push((field.clone(), attr, BuilderType::Field));
                field.attrs.remove(i);
                break;
            } else if attr.path().is_ident("builder_cb") {
                let attr = attr.clone();
                builder.push((field.clone(), attr, BuilderType::Callback));
                field.attrs.remove(i);
                break;
            }
        }
    }

    let mut quotes = quote! {};

    for (field, attr, builder_type) in builder {
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
                    if attr.path().is_ident("doc") {
                        if let Ok(syn::Expr::Lit(literal)) =
                            attr.meta.require_name_value().map(|n| &n.value)
                        {
                            if let syn::Lit::Str(text) = &literal.lit {
                                doc.push_str(&text.value());
                                doc.push('\n');
                            }
                        }
                    }
                }
                let setter_doc = format!("Set {field_name}\n\n{doc}");
                let builder_doc = format!("Builder style method for [`{setter}`](Self::{setter})");
                (builder_doc, setter_doc)
            }
        };

        let attr_span = attr.path().span();

        let (param_type, convert) = if let Ok(list) = attr.meta.require_list() {
            let tokens = list.to_token_stream();
            match builder_type {
                BuilderType::Field => {
                    let options = syn::parse2::<FieldOptions>(tokens)?; //.map_err(|err| Error::new(span, err))?;

                    let into_fn = options.into_fn;
                    let into_trait = options.into_trait;
                    if let Some(default) = options.default_value {
                        (
                            quote_spanned! { attr_span => impl #into_trait<Option<#field_type>>},
                            quote_spanned! { attr_span => #field_ident.#into_fn().unwrap_or(#default)},
                        )
                    } else {
                        (
                            quote_spanned! {attr_span => impl #into_trait<#field_type>},
                            quote_spanned! {attr_span => #field_ident.#into_fn()},
                        )
                    }
                }
                BuilderType::Callback => {
                    let options = syn::parse2::<CallbackOptions>(tokens)?;
                    let into_fn = options.into_fn;
                    let into_trait = options.into_trait;
                    let inner_type = options.inner_type;
                    (
                        quote_spanned! { attr_span => impl #into_trait<#inner_type>},
                        quote_spanned! { attr_span => #field_ident.#into_fn()},
                    )
                }
            }
        } else {
            match builder_type {
                BuilderType::Field => (
                    quote_spanned! { attr_span => #field_type },
                    quote! { #field_ident},
                ),
                BuilderType::Callback => {
                    return Err(Error::new(
                        attr_span,
                        "missing 'builder_cb' parameters, maybe you want to use 'builder'?",
                    ))
                }
            }
        };

        quotes.extend(quote_spanned! { attr_span =>
            #[doc = #setter_doc]
            pub fn #setter(&mut self, #field_ident: #param_type) {
                self.#field_ident = #convert;
            }

            #[doc = #builder_doc]
            pub fn #field_ident(mut self, #field_ident: #param_type) -> Self {
                self.#setter(#field_ident);
                self
            }
        });
    }

    Ok(quote! {
        #(#attrs)*
        #vis struct #ident #generics {
            #fields
        }

        /// Auto-generated builder methods
        impl #impl_generics #ident #ty_generics #where_clause {
            #quotes
        }
    })
}
