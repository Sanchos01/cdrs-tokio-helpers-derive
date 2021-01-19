use common::struct_fields;
use proc_macro::TokenStream;

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let idents = struct_fields(ast)
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    let fields = idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>();
    let names = fields.join(", ");
    let question_marks = fields
        .iter()
        .map(|_| "?".to_string())
        .collect::<Vec<String>>()
        .join(", ");

    let gen = quote! {
        impl #name {
            pub fn insert_query() -> &'static str {
                concat!("insert into ", stringify!(#name), "(",
                  #names,
                 ") values (",
                 #question_marks,
                 ")")
            }

            pub fn into_query_values(self) -> cdrs_tokio::query::QueryValues {
                use std::collections::HashMap;
                let mut values: HashMap<String, cdrs_tokio::types::value::Value> = HashMap::new();

                #(
                    values.insert(stringify!(#idents).to_string(), self.#idents.into());
                )*

                cdrs_tokio::query::QueryValues::NamedValues(values)
            }
        }
    };
    gen.into()
}
