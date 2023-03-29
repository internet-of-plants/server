use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error, ResultExt};
use syn::{spanned::Spanned, DeriveInput, Fields};
use quote::quote;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn id(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("#[id] is only defined for unit structs");

    // Build the impl
    let gen = produce(&ast);

    // Return the generated impl
    gen.into()
}

fn produce(ast: &DeriveInput) -> TokenStream  {
    let name = &ast.ident;
    let generics = &ast.generics;
    let visibility = &ast.vis;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match &ast.data {
        syn::Data::Struct(struct_data) => {           
            if !matches!(struct_data.fields, Fields::Unit) {
                abort!(struct_data.fields.span(), "#[id] is only defined for unit structs");
            }

            quote! {
                #[derive(
                    ::sqlx::Type,
                    ::serde::Serialize,
                    ::serde::Deserialize,
                    ::derive_more::Display,
                    ::derive_more::FromStr,
                    Clone,
                    Copy,
                    Debug,
                    PartialEq,
                    Eq,
                    Hash,
                    PartialOrd,
                    Ord,
                )]
                #[sqlx(transparent)]
                #visibility struct #name #generics(i64) #where_clause;

                impl #impl_generics ::sqlx::postgres::PgHasArrayType for #name #ty_generics #where_clause {
                    fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
                        use ::sqlx::postgres::PgHasArrayType;
                        i64::array_type_info()
                    }

                    fn array_compatible(ty: &::sqlx::postgres::PgTypeInfo) -> bool {
                        use ::sqlx::postgres::PgHasArrayType;
                        i64::array_compatible(ty)
                    }
                }

                impl #impl_generics From<i64> for #name #ty_generics #where_clause {
                    fn from(id: i64) -> #name #ty_generics {
                        Self(0)
                    }
                }

                impl #impl_generics From<#name #ty_generics> for i64 #where_clause {
                    fn from(name: #name #ty_generics) -> i64 {
                        name.0
                    }
                }
            }.into()
        }
        syn::Data::Enum(_) => {
            // Nope. This is an Enum. We cannot handle these!
            abort_call_site!("#[id] is only defined for unit structs, not for enums!");
        }
        syn::Data::Union(_) => {
            // Nope. This is an Enum. We cannot handle these!
            abort_call_site!("#[id] is only defined for unit structs, not for unions!");
        }
    }
}
