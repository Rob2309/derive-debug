use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{
    parse_macro_input, Attribute, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, Ident,
    Meta, NestedMeta, MetaNameValue, Lit, Variant, FieldsUnnamed,
};

#[proc_macro_derive(Dbg, attributes(dbg))]
pub fn derive_debug(target: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(target as DeriveInput);
    derive_debug_impl(item).into()
}

fn derive_debug_impl(item: DeriveInput) -> TokenStream {
    let name = &item.ident;
    let (impl_generics, type_generics, where_clause) = &item.generics.split_for_impl();

    let res = match &item.data {
        syn::Data::Struct(data) => derive_struct(name, data),
        syn::Data::Enum(data) => derive_enum(data),
        syn::Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "#[derive(Dbg)] not supported on unions",
        )),
    };

    match res {
        Ok(res) => quote! {
            impl #impl_generics ::std::fmt::Debug for #name #type_generics #where_clause {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    #res
                }
            }
        },
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_struct(name: &Ident, data: &DataStruct) -> Result<TokenStream, syn::Error> {
    let name_str = name.to_string();

    let fields = match &data.fields {
        Fields::Named(fields) => derive_named_fields(fields, true)?,
        Fields::Unnamed(fields) => derive_unnamed_fields(fields, true)?,
        Fields::Unit => quote! {},
    };

    Ok(quote! {
        f.debug_struct(#name_str)
            #fields
            .finish()
    })
}

fn derive_enum(data: &DataEnum) -> Result<TokenStream, syn::Error> {
    if data.variants.is_empty() {
        return Ok(quote! {
            unsafe { ::core::hint::unreachable_unchecked() }
        });
    }

    let variants = derive_enum_variants(data.variants.iter())?;

    Ok(quote! {
        match self {
            #variants
        }
    })
}

fn derive_enum_variants<'a>(variants: impl Iterator<Item = &'a Variant>) -> Result<TokenStream, syn::Error> {
    let mut res = TokenStream::new();
    
    for variant in variants {
        let name = &variant.ident;

        let options = parse_options(&variant.attrs, true)?;
        let derive_variant = match options {
            FieldOutputOptions::Normal => derive_variant(name, &variant.fields)?,
            FieldOutputOptions::Skip => skip_variant(name, &variant.fields)?,
            _ => return Err(syn::Error::new_spanned(variant, "Internal error")),
        };

        res.extend(derive_variant);
    }

    Ok(res)
}

fn derive_variant(name: &Ident, fields: &Fields) -> Result<TokenStream, syn::Error> {
    let name_str = name.to_string();

    let match_list = derive_match_list(fields)?;

    match fields {
        Fields::Named(fields) => {
            let fields = derive_named_fields(fields, false)?;
            Ok(quote! {
                Self::#name #match_list => f.debug_struct(#name_str) #fields .finish(),
            })
        },
        Fields::Unnamed(fields) => {
            let fields = derive_unnamed_fields(fields, false)?;
            Ok(quote! {
                Self::#name #match_list => f.debug_tuple(#name_str) #fields .finish(),
            })
        },
        Fields::Unit => Ok(quote!{ Self::#name => write!(f, #name_str), }),
    }
}

fn skip_variant(name: &Ident, fields: &Fields) -> Result<TokenStream, syn::Error> {
    let name_str = name.to_string();

    match fields {
        Fields::Named(_) => Ok(quote!{ Self::#name{..} => f.debug_struct(#name_str).finish(), }),
        Fields::Unnamed(_) => Ok(quote!{ Self::#name(..) => f.debug_tuple(#name_str).finish(), }),
        Fields::Unit => Ok(quote!{ Self::#name => write!(f, #name_str), }),
    }
}

fn derive_match_list(fields: &Fields) -> Result<TokenStream, syn::Error> {
    match fields {
        Fields::Named(fields) => {
            let mut res = TokenStream::new();
            for field in &fields.named {
                let name = field.ident.as_ref().unwrap();
                let options = parse_options(&field.attrs, false)?;

                match options {
                    FieldOutputOptions::Skip => res.extend(quote!{ #name: _, }),
                    _ => res.extend(quote!{ #name, }),
                }
            }
            Ok(quote!{ { #res } })
        },
        Fields::Unnamed(fields) => {
            let mut res = TokenStream::new();
            for (i, field) in fields.unnamed.iter().enumerate() {
                let name = format_ident!("field_{}", i);
                let options = parse_options(&field.attrs, false)?;

                match options {
                    FieldOutputOptions::Skip => res.extend(quote!{ _, }),
                    _ => res.extend(quote!{ #name, }),
                }
            }
            Ok(quote!{ (#res) })
        },
        Fields::Unit => {
            Ok(quote!{})
        },
    }
}

fn derive_named_fields(fields: &FieldsNamed, use_self: bool) -> Result<TokenStream, syn::Error> {
    let mut res = TokenStream::new();

    for field in &fields.named {
        let name = field.ident.as_ref().unwrap();
        let name_str = name.to_string();

        let options = parse_options(&field.attrs, false)?;

        match options {
            FieldOutputOptions::Normal => {
                let field_ref = if use_self { quote!{ &self.#name } } else { quote!{ #name } };
                res.extend(quote! { .field(#name_str, #field_ref) });
            },
            FieldOutputOptions::Placeholder(placeholder) => {
                res.extend(quote! { .field(#name_str, &format_args!(#placeholder)) })
            }
            FieldOutputOptions::Skip => {},
        }
    }

    Ok(res)
}

fn derive_unnamed_fields(fields: &FieldsUnnamed, use_self: bool) -> Result<TokenStream, syn::Error> {
    let mut res = TokenStream::new();

    for (i, field) in fields.unnamed.iter().enumerate() {
        let options = parse_options(&field.attrs, false)?;

        match options {
            FieldOutputOptions::Normal => {
                let field_ref = if use_self { quote!{ &self.#i } } else { format_ident!("field_{}", i).to_token_stream() };
                res.extend(quote! { .field(#field_ref) });
            },
            FieldOutputOptions::Placeholder(placeholder) => {
                res.extend(quote! { .field(&format_args!(#placeholder)) })
            }
            FieldOutputOptions::Skip => {},
        }
    }

    Ok(res)
}

enum FieldOutputOptions {
    Normal,
    Placeholder(String),
    Skip,
}

fn parse_options(attributes: &[Attribute], is_enum_variant: bool) -> Result<FieldOutputOptions, syn::Error> {
    let mut res = FieldOutputOptions::Normal;

    for attrib in attributes {
        if !attrib.path.is_ident("dbg") {
            continue;
        }

        let meta = attrib.parse_meta()?;
        let meta = if let Meta::List(m) = meta {
            m
        } else {
            return Err(syn::Error::new_spanned(
                meta,
                "invalid #[dbg(...)] attribute",
            ));
        };

        for option in meta.nested {
            match option {
                NestedMeta::Meta(Meta::Path(option)) if option.is_ident("skip") => res = FieldOutputOptions::Skip,
                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit: Lit::Str(placeholder), .. })) if path.is_ident("placeholder") && !is_enum_variant => res = FieldOutputOptions::Placeholder(placeholder.value()),
                _ => return Err(syn::Error::new_spanned(option, "invalid option")),
            }
        }
    }

    Ok(res)
}
