use quote::format_ident;
use syn::{parse_quote, Type, TypePath};

fn main() {
    // let item_impl: ItemImpl = parse_quote!(
    //     impl<T> A for B<T> {
    //         fn new() -> Self {}
    //     }
    // );
    // dbg!(item_impl);

    let ident = format_ident!("Self");
    let ty_from_macro: Type = parse_quote!( #ident );
    let ty = Type::Path(TypePath { qself: None,
                                   path:  ident.into(), });
    assert_eq!(ty_from_macro, ty);
}
