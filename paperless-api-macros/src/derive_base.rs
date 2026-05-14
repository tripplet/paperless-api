use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Ident, Visibility, parse_quote};

#[allow(dead_code)]
pub(crate) struct DtoFieldAttributes {
    /// If true, this field can not be used for creating or updating the DTO.
    /// e.g. the id of the entity
    skip: bool,
}

/// Represents a base struct for a DTO, containing its name, fields, and endpoint URL.
pub(crate) struct BaseStruct {
    pub(crate) name: Ident,

    visiblity: Visibility,

    /// The fields of the DTO struct.
    pub(crate) fields: Vec<syn::Field>,

    /// The endpoint URL for the Item.
    pub endpoint: String,

    pub id_type: Ident,
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
        // Parse #[api_info(endpoint = "...")] attribute
        let mut endpoint = None;
        let mut visiblity = input.vis;
        let mut id_type = format_ident!("{}Id", input.ident);

        for attr in &input.attrs {
            if attr.path().is_ident("api_info") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("private") {
                        visiblity = parse_quote! { pub(crate) };
                    } else if meta.path.is_ident("endpoint") {
                        let value = meta.value()?;
                        let lit: syn::LitStr = value.parse()?;
                        endpoint = Some(lit.value());
                    } else if meta.path.is_ident("id") {
                        let value = meta.value()?;
                        id_type = value.parse()?;
                    }

                    Ok(())
                })?;
            }
        }

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

        let Some(endpoint) = endpoint else {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Derive requires #[api_info(endpoint = \"...\")] attribute",
            ));
        };

        Ok(Self {
            name: input.ident,
            visiblity,
            id_type,
            endpoint,
            fields: fields.iter().cloned().collect(),
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

            let def = if all_optional {
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

        let visibility = &self.visiblity;

        // Generate the struct
        quote! {
            #[derive(Debug, Default, Clone, serde::Serialize)]
            #[automatically_derived]
            #visibility struct #new_name {
                #(#field_defs)*
            }
        }
    }
}

fn non_dto_attrs(attrs: &[syn::Attribute]) -> Vec<&syn::Attribute> {
    attrs.iter().filter(|a| !a.path().is_ident("dto")).collect()
}
