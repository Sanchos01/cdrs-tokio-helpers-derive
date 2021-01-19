use common::get_struct_fields;
use proc_macro::TokenStream;
use syn;

pub fn impl_try_from_udt(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = get_struct_fields(ast);
    TokenStream::from(quote! {
        impl TryFromUDT for #name {
            fn try_from_udt(cdrs: cdrs_tokio::types::udt::UDT) -> cdrs_tokio::Result<Self> {
                Ok(#name {
                    #(#fields),*
                })
            }
        }
    })
}
