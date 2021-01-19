use common::get_ident_string;
use proc_macro::TokenStream;
use syn;

pub fn impl_into_cdrs_value(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if let syn::Data::Struct(syn::DataStruct { ref fields, .. }) = ast.data {
        let conver_into_bytes = fields.iter().map(|field| {
            let field_ident = field.ident.clone().unwrap();
            if get_ident_string(field.ty.clone()).as_str() == "Option" {
                return quote!{
                  match self.#field_ident {
                    Some(ref val) => {
                      let field_bytes: cdrs_tokio::types::value::Bytes = val.clone().into();
                      bytes.append(&mut cdrs_tokio::types::value::Value::new_normal(field_bytes).into_cbytes());
                    },
                    None => {
                      bytes.append(&mut cdrs_tokio::types::value::Value::new_not_set().into_cbytes());
                    }
                  }
                };
            } else {
                return quote! {
                  let field_bytes: cdrs_tokio::types::value::Bytes = self.#field_ident.into();
                  bytes.append(&mut cdrs_tokio::types::value::Value::new_normal(field_bytes).into_cbytes());
                };
            }
        });
        // As Value has following implementation impl<T: Into<Bytes>> From<T> for Value
        // for a struct it's enough to implement Into<Bytes> in order to be convertable into Value
        // wich is used for making queries
        let gen = quote! {
            #[allow(clippy::from_over_into)]
            impl Into<cdrs_tokio::types::value::Bytes> for #name {
                fn into(self) -> cdrs_tokio::types::value::Bytes {
                    let mut bytes: Vec<u8> = vec![];
                    #(#conver_into_bytes)*
                    cdrs_tokio::types::value::Bytes::new(bytes)
                }
            }
        };
        gen.into()
    } else {
        panic!("#[derive(IntoCDRSValue)] is only defined for structs, not for enums!");
    }
}
