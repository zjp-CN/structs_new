use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// struct D<T> where T: Copy {
///     foo: u8,
///     pub bar: T,
///     abc: u8 = 255,
/// }
/// 暂时忽略字段前的属性
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
