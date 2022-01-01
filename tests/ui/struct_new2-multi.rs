#![allow(unused)]
use proc::structs_new2 as structs_new;
fn main() {
    structs_new! {
        #[derive(Debug)]
        struct A {
            foo: u8,
            pub bar: String,
            abc: u8 = 255,
        };
        #[derive(Debug)]
        struct B<T> {
            foo: u8,
            pub bar: T,
            abc: u8 = 255,
        };
        #[derive(Debug)]
        struct C<'a> {
            foo: &'a str,
            abc: u8 = 255,
        };
        #[derive(Debug)]
        struct D<'a, T> {
            foo: &'a str,
            pub bar: T,
            abc: u8 = 255,
        };
        #[derive(Debug)]
        struct E<T> where T: Copy {
            foo: u8,
            pub bar: T,
            abc: u8 = 255,
        };
        #[derive(Debug)]
        struct F<I: Iterator> where I::Item: Copy {
            pub bar: I,
            abc: u8 = 255,
        };
        #[derive(Debug)]
        struct G<'a, 'b, 'c: 'a+'b> {
            foo: &'c str,
            pub bar: &'a str,
            abc: &'b str = "",
        };
    }

    dbg!(A::new(0, "".into()));
    dbg!(B::new(1, 2));
    dbg!(C::new(""));
    dbg!(D::new("", 1));
    dbg!(E::new(1, 2));
    dbg!(F::new(vec![1].into_iter()));
    dbg!(G::new("", ""));
}
