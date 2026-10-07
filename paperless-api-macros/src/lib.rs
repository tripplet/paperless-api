mod derive_base;
mod repr_serde;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

use crate::derive_base::BaseStruct;

/// Replaces each `{}` in a template with an ID's name, stripping `Id` and splitting CamelCase.
///
/// For example, `id_doc!(CustomFieldId, "ID of a {} entity.")` produces
/// `"ID of a custom field entity."`.
#[proc_macro]
pub fn id_doc(input: TokenStream) -> TokenStream {
    let parse_input = |input: syn::parse::ParseStream| {
        let name = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![,]>()?;
        let template = input.parse::<syn::LitStr>()?;
        Ok((name, template))
    };
    let (name, template) = parse_macro_input!(input with parse_input);
    let name = name.to_string();
    let entity = name.strip_suffix("Id").unwrap_or(&name);
    let chars: Vec<_> = entity.chars().collect();
    let mut words = String::new();

    for (index, &ch) in chars.iter().enumerate() {
        if index > 0
            && ch.is_uppercase()
            && (chars[index - 1].is_lowercase()
                || chars[index - 1].is_numeric()
                || chars.get(index + 1).is_some_and(|next| next.is_lowercase()))
        {
            words.push(' ');
        }
        words.extend(ch.to_lowercase());
    }

    let doc = template.value().replace("{}", &words);
    quote!(#doc).into()
}

/// Derives a `Create..` struct for the given input struct.
#[proc_macro_derive(CreateDto, attributes(dto, api_info))]
pub fn derive_create_dto(input: TokenStream) -> TokenStream {
    // Parse the input
    let input = parse_macro_input!(input as DeriveInput);
    let input_struct = match BaseStruct::try_from(input) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generate the new struct
    let new_struct_name = format_ident!("Create{}", input_struct.name);
    let new_struct = input_struct.generate_new_struct(&new_struct_name, false);

    let id_type_name = input_struct.id_type;
    let id_type_name = quote!(crate::id::#id_type_name);

    let name = input_struct.name;

    // Generate the final output with the trait implementation
    TokenStream::from(quote! {
        #new_struct

        #[automatically_derived]
        impl crate::dto::CreateDto for #new_struct_name {
            type Id = #id_type_name;
            type BaseType = #name;
        }
    })
}

/// Derives a `Update..` struct for the given input struct.
#[proc_macro_derive(UpdateDto, attributes(dto, api_info))]
pub fn derive_update_dto(input: TokenStream) -> TokenStream {
    // Parse the input
    let input = parse_macro_input!(input as DeriveInput);
    let input_struct = match BaseStruct::try_from(input) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generate the new struct
    let new_struct_name = format_ident!("Update{}", input_struct.name);
    let new_struct = input_struct.generate_new_struct(&new_struct_name, true);

    let id_type_name = input_struct.id_type;
    let id_type_name = quote!(crate::id::#id_type_name);

    let name = input_struct.name;

    // Generate the final output with the trait implementation
    TokenStream::from(quote! {
        #new_struct

        #[automatically_derived]
        impl crate::dto::UpdateDto for #new_struct_name {
            type Id = #id_type_name;
            type BaseType = #name;
        }
    })
}

/// Derives `Item` trait for the given input struct.
#[proc_macro_derive(Item, attributes(dto, api_info))]
pub fn derive_item_trait(input: TokenStream) -> TokenStream {
    // Parse the input
    let input = parse_macro_input!(input as DeriveInput);
    let input_struct = match BaseStruct::try_from(input) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error().into(),
    };

    let id_type_name = input_struct.id_type;
    let id_type_name = quote!(crate::id::#id_type_name);

    let name = input_struct.name;

    // Generate the final output with the trait implementation
    TokenStream::from(quote! {
        #[automatically_derived]
        impl crate::dto::Item for #name {
            type Id = #id_type_name;

            #[inline]
            fn id(&self) -> Self::Id {
                self.id
            }
        }
    })
}

/// Derives `Serialize` and `Deserialize` by reading discriminant values
/// directly from the AST.
///
/// The enum must have exactly one data-carrying variant (e.g. `Unknown(u8)`)
/// which is used as the fallback for unknown values.
#[proc_macro_derive(ReprSerde)]
pub fn derive_repr_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let repr_enum = match repr_serde::ReprEnum::try_from(input) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error().into(),
    };

    TokenStream::from(repr_enum.generate())
}
