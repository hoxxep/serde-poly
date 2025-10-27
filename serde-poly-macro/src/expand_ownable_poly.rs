use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    spanned::Spanned, Data, DeriveInput, Fields, GenericArgument, GenericParam, Lifetime,
    PathArguments, Type,
};

pub fn expand_ownable_poly(input: DeriveInput) -> syn::Result<TokenStream2> {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    // Only support structs
    let fields = match data {
        Data::Struct(data_struct) => data_struct.fields,
        Data::Enum(data_enum) => {
            return Err(syn::Error::new(
                data_enum.enum_token.span(),
                "OwnablePoly derive does not support enums",
            ));
        }
        Data::Union(data_union) => {
            return Err(syn::Error::new(
                data_union.union_token.span(),
                "OwnablePoly derive does not support unions",
            ));
        }
    };

    // Extract lifetime parameters
    let lifetime_params: Vec<_> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Lifetime(lt) => Some(lt.lifetime.clone()),
            _ => None,
        })
        .collect();

    // For types without lifetimes, we implement OwnablePoly with Owned = Self
    if lifetime_params.is_empty() {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        return Ok(quote! {
            impl #impl_generics ::serde_poly::OwnablePoly for #ident #ty_generics #where_clause {
                type Owned = Self;

                fn into_owned(self) -> Self::Owned {
                    self
                }
            }
        });
    }

    // Generate the Owned type with all lifetimes replaced by 'static
    let mut owned_generics = generics.clone();
    for param in &mut owned_generics.params {
        if let GenericParam::Lifetime(lt) = param {
            lt.lifetime = Lifetime::new("'static", lt.lifetime.span());
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (_, owned_ty_generics, _) = owned_generics.split_for_impl();

    // Generate field transformations
    let field_transformations = generate_field_transformations(&fields, &lifetime_params)?;

    Ok(quote! {
        impl #impl_generics ::serde_poly::OwnablePoly for #ident #ty_generics #where_clause {
            type Owned = #ident #owned_ty_generics;

            fn into_owned(self) -> Self::Owned {
                #ident #field_transformations
            }
        }
    })
}

fn generate_field_transformations(
    fields: &Fields,
    lifetime_params: &[Lifetime],
) -> syn::Result<TokenStream2> {
    match fields {
        Fields::Named(fields_named) => {
            let field_inits = fields_named.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let has_lifetime = type_contains_any_lifetime(&field.ty, lifetime_params);

                if has_lifetime {
                    quote! {
                        #field_name: ::serde_poly::OwnablePoly::into_owned(self.#field_name)
                    }
                } else {
                    quote! {
                        #field_name: self.#field_name
                    }
                }
            });

            Ok(quote! {
                {
                    #(#field_inits),*
                }
            })
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_inits = fields_unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let index = syn::Index::from(i);
                    let has_lifetime = type_contains_any_lifetime(&field.ty, lifetime_params);

                    if has_lifetime {
                        quote! {
                            ::serde_poly::OwnablePoly::into_owned(self.#index)
                        }
                    } else {
                        quote! {
                            self.#index
                        }
                    }
                });

            Ok(quote! {
                (
                    #(#field_inits),*
                )
            })
        }
        Fields::Unit => Ok(quote! {}),
    }
}

/// Check if a type contains any of the specified lifetimes
fn type_contains_any_lifetime(ty: &Type, lifetimes: &[Lifetime]) -> bool {
    match ty {
        Type::Reference(type_ref) => {
            // Check if the reference's lifetime matches any of our lifetimes
            if let Some(ref lt) = type_ref.lifetime {
                if lifetimes.iter().any(|param_lt| lt.ident == param_lt.ident) {
                    return true;
                }
            }
            // Recursively check the referenced type
            type_contains_any_lifetime(&type_ref.elem, lifetimes)
        }
        Type::Path(type_path) => {
            // Check if any generic arguments contain our lifetimes
            for segment in &type_path.path.segments {
                match &segment.arguments {
                    PathArguments::AngleBracketed(args) => {
                        for arg in &args.args {
                            match arg {
                                GenericArgument::Lifetime(lt) => {
                                    if lifetimes.iter().any(|param_lt| lt.ident == param_lt.ident)
                                    {
                                        return true;
                                    }
                                }
                                GenericArgument::Type(inner_ty) => {
                                    if type_contains_any_lifetime(inner_ty, lifetimes) {
                                        return true;
                                    }
                                }
                                GenericArgument::AssocType(assoc) => {
                                    if type_contains_any_lifetime(&assoc.ty, lifetimes) {
                                        return true;
                                    }
                                }
                                GenericArgument::Constraint(constraint) => {
                                    for bound in &constraint.bounds {
                                        if let syn::TypeParamBound::Lifetime(lt) = bound {
                                            if lifetimes
                                                .iter()
                                                .any(|param_lt| lt.ident == param_lt.ident)
                                            {
                                                return true;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    PathArguments::Parenthesized(args) => {
                        for input in &args.inputs {
                            if type_contains_any_lifetime(input, lifetimes) {
                                return true;
                            }
                        }
                        if let syn::ReturnType::Type(_, ty) = &args.output {
                            if type_contains_any_lifetime(ty, lifetimes) {
                                return true;
                            }
                        }
                    }
                    PathArguments::None => {}
                }
            }
            false
        }
        Type::Tuple(type_tuple) => type_tuple
            .elems
            .iter()
            .any(|elem| type_contains_any_lifetime(elem, lifetimes)),
        Type::Array(type_array) => type_contains_any_lifetime(&type_array.elem, lifetimes),
        Type::Ptr(type_ptr) => type_contains_any_lifetime(&type_ptr.elem, lifetimes),
        Type::Slice(type_slice) => type_contains_any_lifetime(&type_slice.elem, lifetimes),
        Type::Paren(type_paren) => type_contains_any_lifetime(&type_paren.elem, lifetimes),
        Type::Group(type_group) => type_contains_any_lifetime(&type_group.elem, lifetimes),
        _ => false,
    }
}