mod derive_base;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

use crate::derive_base::{BaseStruct, ItemStruct};

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

    // Generate the final output with the trait implementation
    TokenStream::from(quote! {
        #new_struct

        #[automatically_derived]
        impl crate::dto::CreateDtoObject for #new_struct_name {}
    })
}

/// Derives a `Create..` struct for the given input struct.
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

    // Generate the final output with the trait implementation
    TokenStream::from(quote! {
        #new_struct

        #[automatically_derived]
        impl crate::dto::UpdateDtoObject for #new_struct_name {}
    })
}

/// Derives a `Create..` struct for the given input struct.
#[proc_macro_derive(Item, attributes(dto, api_info))]
pub fn derive_item_trait(input: TokenStream) -> TokenStream {
    // Parse the input
    let input = parse_macro_input!(input as DeriveInput);
    let input_struct = match ItemStruct::try_from(input) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error().into(),
    };

    let update_dto = format_ident!("Update{}", input_struct.base_struct.name);
    let create_dto = format_ident!("Create{}", input_struct.base_struct.name);
    let id_type_name = format_ident!("{}Id", input_struct.base_struct.name);
    let id_type_name = quote!(crate::id::#id_type_name);

    let endpoint = input_struct.endpoint.clone();
    let name = input_struct.base_struct.name;

    // Generate the final output with the trait implementation
    TokenStream::from(quote! {
        #[automatically_derived]
        impl crate::dto::Item for #name {
            type Id = #id_type_name;
            type BaseType = #name;
            type CreateDto = #create_dto;
            type UpdateDto = #update_dto;

            fn endpoint() -> &'static str {
                #endpoint
            }

            fn id(&self) -> Self::Id {
                self.id
            }
        }
    })
}
