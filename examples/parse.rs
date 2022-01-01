use quote::format_ident;
use syn::{parse_quote, Type, TypePath};

fn main() {
    let ty: syn::Type = parse_quote!(Box<dyn Fn(T, T2) -> (R, R2)>);
    dbg!(ty);

    let ident = format_ident!("Self");
    let ty_from_macro: Type = parse_quote!( #ident );
    let ty = Type::Path(TypePath { qself: None,
                                   path:  ident.into(), });
    assert_eq!(ty_from_macro, ty);
}
