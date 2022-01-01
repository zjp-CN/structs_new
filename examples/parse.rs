use syn::ItemImpl;

fn main() {
    use syn::parse_quote;
    let item_impl: ItemImpl = parse_quote!(
        impl<T> A for B<T> {
            fn new() -> Self {}
        }
    );
    dbg!(item_impl);
}
