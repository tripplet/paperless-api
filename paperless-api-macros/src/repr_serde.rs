use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Path};

pub(crate) struct ReprEnum {
    pub name: syn::Ident,
    pub repr_type: Path,
    pub unit_variants: Vec<(syn::Ident, syn::Expr)>,
    pub fallback: syn::Ident,
}

impl TryFrom<DeriveInput> for ReprEnum {
    type Error = syn::Error;

    fn try_from(input: DeriveInput) -> syn::Result<Self> {
        let name = input.ident;
        let repr_type = input
            .attrs
            .iter()
            .find(|a| a.path().is_ident("repr"))
            .ok_or_else(|| syn::Error::new_spanned(&name, "ReprSerde requires #[repr(...)]"))?
            .parse_args::<Path>()?;

        let Data::Enum(data) = input.data else {
            return Err(syn::Error::new_spanned(
                &name,
                "ReprSerde only supports enums",
            ));
        };

        let mut unit_variants = Vec::new();
        let mut fallback = None;

        for variant in data.variants {
            match &variant.fields {
                Fields::Unit => {
                    let (_, expr) = variant.discriminant.as_ref().ok_or_else(|| {
                        syn::Error::new_spanned(&variant, "explicit discriminant required")
                    })?;
                    unit_variants.push((variant.ident, expr.clone()));
                }
                Fields::Unnamed(u) if u.unnamed.len() == 1 => {
                    if fallback.is_some() {
                        return Err(syn::Error::new_spanned(
                            &variant,
                            "at most one fallback variant",
                        ));
                    }
                    fallback = Some(variant.ident);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &variant,
                        "variants must be unit or single-field unnamed",
                    ));
                }
            }
        }

        let fallback = fallback.ok_or_else(|| {
            syn::Error::new_spanned(
                &name,
                "needs exactly one fallback variant (e.g. Unknown(u8))",
            )
        })?;

        Ok(Self {
            name,
            repr_type,
            unit_variants,
            fallback,
        })
    }
}

impl ReprEnum {
    pub(crate) fn generate(&self) -> TokenStream {
        let name = &self.name;
        let repr_type = &self.repr_type;
        let fallback = &self.fallback;

        let ser_arms = self.unit_variants.iter().map(|(v, e)| {
            quote! { Self::#v => #e, }
        });

        let de_arms = self.unit_variants.iter().map(|(v, e)| {
            quote! { d if d == #e => Self::#v, }
        });

        quote! {
            #[automatically_derived]
            impl serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: serde::Serializer,
                {
                    let v: #repr_type = match self {
                        #(#ser_arms)*
                        Self::#fallback(x) => *x,
                    };
                    v.serialize(serializer)
                }
            }

            #[automatically_derived]
            impl<'de> serde::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: serde::Deserializer<'de>,
                {
                    let v: #repr_type = serde::Deserialize::deserialize(deserializer)?;
                    Ok(match v {
                        #(#de_arms)*
                        _ => Self::#fallback(v),
                    })
                }
            }
        }
    }
}
