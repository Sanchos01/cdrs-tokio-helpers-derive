use common::get_struct_fields;
use proc_macro::TokenStream;
use syn;

pub fn impl_try_from_row(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = get_struct_fields(ast);

    TokenStream::from(quote! {
        impl TryFromRow for #name {
            fn try_from_row(cdrs: cdrs_tokio::types::rows::Row) -> cdrs_tokio::Result<Self> {
                Ok(#name {
                    #(#fields),*
                })
            }
        }
    })
}
