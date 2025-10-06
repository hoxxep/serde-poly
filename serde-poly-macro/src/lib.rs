//! A derive macro for `#[derive(Poly)]` that implements:
//! - For types without lifetimes:
//!   - impl [`DeserializePoly`] for `Self` with `type Out<'de> = Self`
//!   - impl [`SerializePoly`] for `Self` with `type Out = Self`
//! - For types with lifetimes, such as `MyType<'a, T>`:
//!   - A tuple struct `MyTypePoly<T>(PhantomData<fn() -> T>)`, without lifetimes.
//!   - impl [`SerializePoly`] for `MyType<'a, T>` with `type Out = MyTypePoly<T>`
//!   - impl [`DeserializePoly`] for `MyTypePoly<T>` with `type Out<'de> = MyType<'de, T>`
//!
//! Supports `#[poly(name = "CustomName")]` attributes to customize the name of the
//! generated Poly type.
//!
//! For types with multiple lifetime parameters, the derive macro fails with a clear
//! error message.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute,
    DeriveInput,
    GenericParam,
    Ident,
    LitStr,
    Meta,
    Token,
    Visibility,
};

#[proc_macro_derive(Poly, attributes(poly))]
pub fn derive_poly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_poly(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_poly(input: DeriveInput) -> syn::Result<TokenStream2> {
    if let syn::Data::Union(data_union) = &input.data {
        return Err(syn::Error::new(
            data_union.union_token.span,
            "Poly derive does not support unions",
        ));
    }

    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        ..
    } = input;

    let poly_name = parse_poly_name(&attrs, &ident)?;

    let lifetime_params: Vec<_> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Lifetime(lt) => Some(lt.lifetime.clone()),
            _ => None,
        })
        .collect();

    if lifetime_params.len() > 1 {
        let offending = &lifetime_params[1];
        return Err(syn::Error::new(
            offending.span(),
            "Poly derive supports at most one lifetime parameter",
        ));
    }

    let has_lifetime = !lifetime_params.is_empty();
    if !has_lifetime {
        if let Some(span) = poly_name.attr_span {
            return Err(syn::Error::new(
                span,
                "poly(name = \"...\") is only valid for types with a single lifetime parameter",
            ));
        }
    }

    let poly_ident = poly_name.ident;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut poly_generics = generics.clone();
    poly_generics.params.clear();
    let mut poly_params = Punctuated::<GenericParam, Token![,]>::new();
    for param in generics.params.iter().cloned() {
        if matches!(param, GenericParam::Lifetime(_)) {
            continue;
        }
        poly_params.push(param);
    }
    poly_generics.params = poly_params;
    poly_generics.where_clause = None;

    let poly_generics_decl = if poly_generics.params.is_empty() {
        TokenStream2::new()
    } else {
        let params = poly_generics.params.iter();
        quote! { <#(#params),*> }
    };

    let poly_ty_args: Vec<_> = poly_generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(ty) => {
                let ident = &ty.ident;
                quote!(#ident)
            }
            GenericParam::Const(konst) => {
                let ident = &konst.ident;
                quote!(#ident)
            }
            GenericParam::Lifetime(_) => unreachable!(),
        })
        .collect();

    let serialize_out = if has_lifetime {
        if poly_ty_args.is_empty() {
            quote!(#poly_ident)
        } else {
            quote!(#poly_ident < #(#poly_ty_args),* >)
        }
    } else {
        quote!(Self)
    };

    let mut out_ty_args: Vec<TokenStream2> = Vec::new();
    if has_lifetime {
        for param in &generics.params {
            match param {
                GenericParam::Lifetime(_) => out_ty_args.push(quote!('de)),
                GenericParam::Type(ty) => {
                    let ident = &ty.ident;
                    out_ty_args.push(quote!(#ident));
                }
                GenericParam::Const(konst) => {
                    let ident = &konst.ident;
                    out_ty_args.push(quote!(#ident));
                }
            }
        }
    }

    let deserialize_out = if has_lifetime {
        if out_ty_args.is_empty() {
            quote!(#ident)
        } else {
            quote!(#ident < #(#out_ty_args),* >)
        }
    } else {
        quote!(Self)
    };

    let (poly_impl_generics, poly_ty_generics, poly_where_clause) = poly_generics.split_for_impl();

    let poly_items = if has_lifetime {
        let field_vis: TokenStream2 = match &vis {
            Visibility::Inherited => TokenStream2::new(),
            Visibility::Public(_) => quote!(pub),
            Visibility::Restricted(restricted) => {
                let restricted = restricted.clone();
                quote!(#restricted)
            }
        };

        let phantom_fields: Vec<_> = poly_generics
            .params
            .iter()
            .map(|param| {
                let field_vis = field_vis.clone();
                match param {
                    GenericParam::Type(ty) => {
                        let ident = &ty.ident;
                        quote!(#field_vis ::core::marker::PhantomData<fn() -> #ident>)
                    }
                    GenericParam::Const(_) => {
                        quote!(#field_vis ::core::marker::PhantomData<fn() -> ()>)
                    }
                    GenericParam::Lifetime(_) => unreachable!(),
                }
            })
            .collect();

        Some(quote! {
            #vis struct #poly_ident #poly_generics_decl ( #(#phantom_fields),* );

            impl #poly_impl_generics ::serde::Serialize for #poly_ident #poly_ty_generics #poly_where_clause {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    serializer.serialize_unit_struct(stringify!(#poly_ident))
                }
            }
        })
    } else {
        None
    };

    let deserialize_impl = if has_lifetime {
        quote! {
            impl #poly_impl_generics ::serde_poly::DeserializePoly for #poly_ident #poly_ty_generics #poly_where_clause {
                type Out<'de> = #deserialize_out;
            }
        }
    } else {
        quote! {
            impl #impl_generics ::serde_poly::DeserializePoly for #ident #ty_generics #where_clause {
                type Out<'de> = #deserialize_out;
            }
        }
    };

    let serialize_impl = if has_lifetime {
        quote! {
            impl #impl_generics ::serde_poly::SerializePoly for #ident #ty_generics #where_clause {
                type Out = #serialize_out;
            }
        }
    } else {
        quote! {
            impl #impl_generics ::serde_poly::SerializePoly for #ident #ty_generics #where_clause {
                type Out = #serialize_out;
            }
        }
    };

    Ok(quote! {
        #poly_items
        #deserialize_impl
        #serialize_impl
    })
}

struct PolyName {
    ident: Ident,
    attr_span: Option<Span>,
}

fn parse_poly_name(attrs: &[Attribute], original: &Ident) -> syn::Result<PolyName> {
    let mut provided = None;
    let mut span = None;

    for attr in attrs {
        if !attr.path().is_ident("poly") {
            continue;
        }

        match &attr.meta {
            Meta::Path(_) => {}
            Meta::List(_) => {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let lit: LitStr = meta.value()?.parse()?;
                        let ident = Ident::new(&lit.value(), lit.span());
                        provided = Some(ident);
                        span = Some(lit.span());
                        Ok(())
                    } else {
                        Err(meta.error("unsupported poly attribute"))
                    }
                })?;
            }
            Meta::NameValue(_) => {
                return Err(syn::Error::new(
                    attr.span(),
                    "unsupported poly attribute",
                ));
            }
        }
    }

    let ident = match provided {
        Some(ident) => ident,
        None => format_ident!("{}Poly", original),
    };

    Ok(PolyName {
        ident,
        attr_span: span,
    })
}
