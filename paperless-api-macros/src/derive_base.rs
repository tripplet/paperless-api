use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

#[allow(dead_code)]
pub(crate) struct DtoFieldAttributes {
    /// If true, this field can not be used for creating or updating the DTO.
    /// e.g. the id of the entity
    skip: bool,
}

/// Represents a base struct for a DTO, containing its name, fields, and endpoint URL.
pub(crate) struct BaseStruct {
    pub(crate) name: Ident,

    /// The fields of the DTO struct.
    pub(crate) fields: Vec<syn::Field>,
}

pub(crate) struct ItemStruct {
    /// The base struct for the Item.
    pub(crate) base_struct: BaseStruct,

    /// The endpoint URL for the Item.
    pub(crate) endpoint: String,
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

impl TryFrom<DeriveInput> for BaseStruct {
    type Error = syn::Error;

    fn try_from(input: DeriveInput) -> syn::Result<Self> {
        // Extract the fields
        let fields = match &input.data {
            Data::Struct(data) => match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => {
                    return Err(syn::Error::new_spanned(
                        &input.ident,
                        "DTO derive only supports structs with named fields",
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "DTO derive only supports structs",
                ));
            }
        };

        Ok(Self {
            name: input.ident,
            fields: fields.iter().cloned().collect(),
        })
    }
}

impl TryFrom<DeriveInput> for ItemStruct {
    type Error = syn::Error;

    fn try_from(input: DeriveInput) -> syn::Result<Self> {
        let base_struct = BaseStruct::try_from(input.clone())?;

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
                })?;
            }
        }

        let Some(endpoint) = endpoint else {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "CreateDtoObject requires a #[api_info(endpoint = \"...\")] attribute",
            ));
        };

        Ok(Self {
            base_struct,
            endpoint,
        })
    }
}

impl BaseStruct {
    pub(crate) fn generate_new_struct(
        &self,
        new_name: &Ident,
        all_optional: bool,
    ) -> proc_macro2::TokenStream {
        let mut field_defs = Vec::new();

        for field in &self.fields {
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
}

fn non_dto_attrs(attrs: &[syn::Attribute]) -> Vec<&syn::Attribute> {
    attrs.iter().filter(|a| !a.path().is_ident("dto")).collect()
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }

    false
}
