use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod structs;
mod structs2;

#[proc_macro]
pub fn struct_new(input: TokenStream) -> TokenStream {
    let new_struct: structs::NewItemStruct = parse_macro_input!(input);
    let (item_struct, item_impl) = new_struct.split();
    // dbg!(&normal);
    TokenStream::from(quote! {#item_struct #item_impl})
}

#[proc_macro]
pub fn struct_new2(input: TokenStream) -> TokenStream {
    let new_struct: structs2::NewItemStruct = parse_macro_input!(input);
    let (item_struct, item_impl) = new_struct.split();
    TokenStream::from(quote! {#item_struct #item_impl})
}

#[proc_macro]
pub fn structs_new2(input: TokenStream) -> TokenStream {
    use structs2::NewItemStruct;
    let new_structs = parse_macro_input!(input with NewItemStruct::parse_multi);
    let expand = new_structs.into_iter()
                            .map(NewItemStruct::split)
                            .map(|(item_struct, item_impl)| quote! {#item_struct #item_impl});
    TokenStream::from(quote! { #(#expand)* })
}
