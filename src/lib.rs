//! This trait provides functionality for derivation  `IntoCDRSBytes` trait implementation
//! for underlying

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate rand;
extern crate syn;

mod common;
mod db_mirror;
mod into_cdrs_value;
mod try_from_row;
mod try_from_udt;

use db_mirror::impl_db_mirror;
use into_cdrs_value::impl_into_cdrs_value;
use proc_macro::TokenStream;
use try_from_row::impl_try_from_row;
use try_from_udt::impl_try_from_udt;

#[proc_macro_derive(DBMirror)]
pub fn db_mirror(input: TokenStream) -> TokenStream {
    // Construct ast of the type definition
    let ast = syn::parse(input).unwrap();
    impl_db_mirror(&ast)
}

#[proc_macro_derive(IntoCDRSValue)]
pub fn into_cdrs_value(input: TokenStream) -> TokenStream {
    // Construct ast of the type definition
    let ast = syn::parse(input).unwrap();
    impl_into_cdrs_value(&ast)
}

#[proc_macro_derive(TryFromRow)]
pub fn try_from_row(input: TokenStream) -> TokenStream {
    // Construct ast of the type definition
    let ast = syn::parse(input).unwrap();
    impl_try_from_row(&ast)
}

#[proc_macro_derive(TryFromUDT)]
pub fn try_from_udt(input: TokenStream) -> TokenStream {
    // Construct ast of the type definition
    let ast = syn::parse(input).unwrap();
    impl_try_from_udt(&ast)
}
