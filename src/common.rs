use quote::TokenStreamExt;
use syn;

pub fn get_struct_fields(ast: &syn::DeriveInput) -> Vec<proc_macro2::TokenStream> {
    struct_fields(ast)
        .iter()
        .map(|field| {
            let name = field.ident.clone().unwrap();
            let value = convert_field_into_rust(field.clone());
            quote! {
              #name: #value
            }
        })
        .collect()
}

pub fn struct_fields(ast: &syn::DeriveInput) -> &syn::Fields {
    if let syn::Data::Struct(syn::DataStruct { ref fields, .. }) = ast.data {
        fields
    } else {
        panic!("The derive macro is defined for structs with named fields, not for enums or unit structs");
    }
}

pub fn get_map_params_string(ty: syn::Type) -> (syn::Type, syn::Type) {
    match ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => match segments.last() {
            Some(&syn::PathSegment {
                arguments: syn::PathArguments::AngleBracketed(ref angle_bracketed_data),
                ..
            }) => {
                let braket_types = angle_bracketed_data.args.clone();
                let first_generic = braket_types
                    .first()
                    .expect("Cannot define Option type")
                    .clone();
                let last_generic = braket_types
                    .last()
                    .expect("Cannot define Option type")
                    .clone();
                if let (syn::GenericArgument::Type(first), syn::GenericArgument::Type(last)) =
                    (first_generic, last_generic)
                {
                    (first, last)
                } else {
                    panic!("Cannot infer field type")
                }
            }
            _ => panic!("Cannot infer field type"),
        },
        _ => panic!("Cannot infer field type {:?}", get_ident_string(ty)),
    }
}

fn convert_field_into_rust(field: syn::Field) -> proc_macro2::TokenStream {
    let mut string_name = quote! {};
    let s = remove_r(format!("{}", field.ident.clone().unwrap()));
    string_name.append(proc_macro2::Literal::string(s.trim()));
    let arguments = get_arguments(string_name);

    into_rust_with_args(field.ty, arguments)
}

fn remove_r(s: String) -> String {
    if &s[..2] == "r#" {
        s[2..].to_string()
    } else {
        s
    }
}

fn get_arguments(name: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
      &cdrs, #name
    }
}

fn into_rust_with_args(
    field_type: syn::Type,
    arguments: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let field_type_ident = get_cdrs_type_ident(field_type.clone());
    match get_ident_string(field_type_ident.clone()).as_str() {
        "Blob" | "String" | "bool" | "i64" | "i32" | "i16" | "i8" | "f64" | "f32" | "Decimal"
        | "IpAddr" | "Uuid" | "Timespec" | "PrimitiveDateTime" | "NaiveDateTime" | "DateTime" => {
            quote! {
              #field_type_ident::from_cdrs_r(#arguments)?
            }
        }
        "List" => {
            let list_as_rust = as_rust(field_type, quote! {list});

            quote! {
              match cdrs_tokio::types::list::List::from_cdrs_r(#arguments) {
                Ok(ref list) => {
                  #list_as_rust
                },
                _ => return Err("List should not be empty".into())
              }
            }
        }
        "Map" => {
            let map_as_rust = as_rust(field_type, quote! {map});
            quote! {
              match cdrs_tokio::types::map::Map::from_cdrs_r(#arguments) {
                Ok(map) => {
                  #map_as_rust
                },
                _ => return Err("Map should not be empty".into())
              }
            }
        }
        "Option" => {
            let opt_type = get_ident_params_string(field_type);
            let opt_type_rustified = get_cdrs_type_ident(opt_type.clone());
            let opt_value_as_rust = as_rust(opt_type, quote! {opt_value});

            if is_non_zero_primitive(opt_type_rustified.clone()) {
                quote! {
                  #opt_type_rustified::from_cdrs_by_name(#arguments)?
                }
            } else {
                quote! {
                  {
                    match #opt_type_rustified::from_cdrs_by_name(#arguments)? {
                      Some(opt_value) => {
                        let decoded = #opt_value_as_rust;
                        Some(decoded)
                      },
                      _ => None
                    }
                  }
                }
            }
        }
        _ => {
            quote! {
              #field_type::try_from_udt(cdrs_tokio::types::udt::UDT::from_cdrs_r(#arguments)?)?
            }
        }
    }
}

fn is_non_zero_primitive(ident: syn::Type) -> bool {
    matches!(
        get_ident_string(ident).as_str(),
        "NonZeroI8" | "NonZeroI16" | "NonZeroI32" | "NonZeroI64"
    )
}

fn get_cdrs_type_ident(ty: syn::Type) -> syn::Type {
    let type_string = get_ident_string(ty);
    match type_string.as_str() {
        "Blob" => syn::parse_str("Blob").unwrap(),
        "String" => syn::parse_str("String").unwrap(),
        "bool" => syn::parse_str("bool").unwrap(),
        "i64" => syn::parse_str("i64").unwrap(),
        "i32" => syn::parse_str("i32").unwrap(),
        "i16" => syn::parse_str("i16").unwrap(),
        "i8" => syn::parse_str("i8").unwrap(),
        "f64" => syn::parse_str("f64").unwrap(),
        "f32" => syn::parse_str("f32").unwrap(),
        "Decimal" => syn::parse_str("Decimal").unwrap(),
        "IpAddr" => syn::parse_str("IpAddr").unwrap(),
        "Uuid" => syn::parse_str("Uuid").unwrap(),
        "Timespec" => syn::parse_str("Timespec").unwrap(),
        "PrimitiveDateTime" => syn::parse_str("PrimitiveDateTime").unwrap(),
        "Vec" => syn::parse_str("cdrs_tokio::types::list::List").unwrap(),
        "HashMap" => syn::parse_str("cdrs_tokio::types::map::Map").unwrap(),
        "Option" => syn::parse_str("Option").unwrap(),
        "NonZeroI8" => syn::parse_str("NonZeroI8").unwrap(),
        "NonZeroI16" => syn::parse_str("NonZeroI16").unwrap(),
        "NonZeroI32" => syn::parse_str("NonZeroI32").unwrap(),
        "NonZeroI64" => syn::parse_str("NonZeroI64").unwrap(),
        "NaiveDateTime" => syn::parse_str("NaiveDateTime").unwrap(),
        "DateTime" => syn::parse_str("DateTime").unwrap(),
        _ => syn::parse_str("cdrs_tokio::types::udt::UDT").unwrap(),
    }
}

fn get_ident(ty: syn::Type) -> syn::Ident {
    match ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => match segments.last() {
            Some(&syn::PathSegment { ref ident, .. }) => ident.clone(),
            _ => panic!("Cannot infer field type"),
        },
        _ => panic!("Cannot infer field type {:?}", get_ident_string(ty)),
    }
}

// returns single value decoded and optionally iterative mapping that uses decoded value
fn as_rust(ty: syn::Type, val: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let cdrs_type = get_cdrs_type_ident(ty.clone());
    match get_ident_string(cdrs_type).as_str() {
        "Blob" | "String" | "bool" | "i64" | "i32" | "i16" | "i8" | "f64" | "f32" | "IpAddr"
        | "Uuid" | "Timespec" | "Decimal" | "PrimitiveDateTime" => val,
        "List" => {
            let vec_type = get_ident_params_string(ty);
            let inter_rust_type = get_cdrs_type_ident(vec_type.clone());
            let decoded_item = as_rust(vec_type.clone(), quote! {item});
            quote! {
              {
                let inner: Vec<#inter_rust_type> = #val.as_rust_type()?.unwrap();
                let mut decoded: Vec<#vec_type> = Vec::with_capacity(inner.len());
                for item in inner {
                  decoded.push(#decoded_item);
                }
                decoded
              }
            }
        }
        "Map" => {
            let (map_key_type, map_value_type) = get_map_params_string(ty);
            let inter_rust_type = get_cdrs_type_ident(map_value_type.clone());
            let decoded_item = as_rust(map_value_type.clone(), quote! {val});
            quote! {
              {
                let inner: std::collections::HashMap<#map_key_type, #inter_rust_type> = #val.as_rust_type()?.unwrap();
                let mut decoded: std::collections::HashMap<#map_key_type, #map_value_type> = std::collections::HashMap::with_capacity(inner.len());
                for (key, val) in inner {
                  decoded.insert(key, #decoded_item);
                }
                decoded
              }
            }
        }
        "Option" => {
            let opt_type = get_ident_params_string(ty);
            as_rust(opt_type, val)
        }
        _ => {
            quote! {
              #ty::try_from_udt(#val)?
            }
        }
    }
}

pub fn get_ident_string(ty: syn::Type) -> String {
    get_ident(ty).to_string()
}

pub fn get_ident_params_string(ty: syn::Type) -> syn::Type {
    match ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => match segments.last() {
            Some(&syn::PathSegment {
                arguments: syn::PathArguments::AngleBracketed(ref angle_bracketed_data),
                ..
            }) => {
                let generic = angle_bracketed_data
                    .args
                    .last()
                    .expect("Cannot define Option type")
                    .clone();
                if let syn::GenericArgument::Type(t) = generic {
                    t
                } else {
                    panic!("Cannot infer field type")
                }
            }
            _ => panic!("Cannot infer field type"),
        },
        _ => panic!("Cannot infer field type {:?}", get_ident_string(ty)),
    }
}
