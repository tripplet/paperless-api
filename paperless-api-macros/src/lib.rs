use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }

    false
}

#[allow(dead_code)]
struct DtoFieldAttributes {
    /// If true, this field can not be used for creating or updating the DTO.
    /// e.g. the id of the entity
    skip: bool,
}

struct BaseStruct<'a> {
    fields: Vec<&'a syn::Field>,
}

impl DtoFieldAttributes {
    fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut skip = false;

        for attr in attrs {
            if attr.path().is_ident("dto") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        skip = true;
                    }
                    Ok(())
                })?;
            }
        }

        Ok(Self { skip })
    }
}

fn non_dto_attrs(attrs: &[syn::Attribute]) -> Vec<&syn::Attribute> {
    attrs.iter().filter(|a| !a.path().is_ident("dto")).collect()
}

fn new_struct(
    base_struct: &BaseStruct,
    new_name: &Ident,
    all_optional: bool,
) -> proc_macro2::TokenStream {
    let mut field_defs = Vec::new();
    for field in &base_struct.fields {
        // Check if the field should be skipped
        let dto = match DtoFieldAttributes::parse(&field.attrs) {
            Ok(dto) => dto,
            Err(e) => return e.to_compile_error(),
        };
        if dto.skip {
            continue;
        }

        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let vis = &field.vis;
        let attrs = non_dto_attrs(&field.attrs);

        let def = if all_optional && !is_option_type(ty) {
            quote! {
                #(#attrs)*
                #[serde(skip_serializing_if = "Option::is_none")]
                #vis #ident: Option<#ty>,
            }
        } else {
            quote! {
                #(#attrs)*
                #vis #ident: #ty,
            }
        };
        field_defs.push(def);
    }

    // Generate the struct
    quote! {
        #[derive(Debug, Default, Clone, serde::Serialize)]
        pub struct #new_name {
            #(#field_defs)*
        }
    }
}

fn derive_create_or_update(input: TokenStream, update: bool) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let dto_name = if update {
        format_ident!("Update{}", name)
    } else {
        format_ident!("Create{}", name)
    };

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "DTO derive only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input.ident, "DTO derive only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut field_defs = Vec::new();
    for f in fields {
        let dto = match DtoFieldAttributes::parse(&f.attrs) {
            Ok(dto) => dto,
            Err(e) => return e.to_compile_error().into(),
        };
        if dto.skip {
            continue;
        }

        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let vis = &f.vis;
        let attrs = non_dto_attrs(&f.attrs);

        let def = if update && !is_option_type(ty) {
            quote! {
                #(#attrs)*
                #[serde(skip_serializing_if = "Option::is_none")]
                #vis #ident: Option<#ty>,
            }
        } else {
            quote! {
                #(#attrs)*
                #vis #ident: #ty,
            }
        };
        field_defs.push(def);
    }

    let trait_path = if update {
        quote!(crate::dto::UpdateDto)
    } else {
        quote!(crate::dto::CreateDtoObject)
    };

    let expanded = quote! {
        #[derive(Debug, Default, Clone, serde::Serialize)]
        pub struct #dto_name {
            #(#field_defs)*
        }

        impl #trait_path for #dto_name {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(UpdateDto, attributes(dto))]
pub fn derive_update_dto(input: TokenStream) -> TokenStream {
    derive_create_or_update(input, true)
}

#[proc_macro_derive(CreateDto, attributes(dto, api_info))]
pub fn derive_create_dto(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "DTO derive only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input.ident, "DTO derive only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Parse #[api_info(endpoint = "...")] attribute
    let mut endpoint = None;
    for attr in &input.attrs {
        if attr.path().is_ident("api_info") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("endpoint") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    endpoint = Some(lit.value());
                }
                Ok(())
            })
            .unwrap();
        }
    }

    let Some(endpoint) = endpoint else {
        return syn::Error::new_spanned(
            &input.ident,
            "CreateDtoObject requires a #[api_info(endpoint = \"...\")] attribute",
        )
        .to_compile_error()
        .into();
    };

    let new_struct_name = format_ident!("Create{}", name);

    let new_struct = new_struct(
        &BaseStruct {
            fields: fields.iter().collect(),
        },
        &new_struct_name,
        false,
    );

    TokenStream::from(quote! {
        #new_struct

        impl crate::dto::CreateDtoObject for #new_struct_name {
            type BaseType = #name;

            fn endpoint() -> &'static str {
                #endpoint
            }
        }
    })
}
