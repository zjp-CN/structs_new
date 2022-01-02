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
        struct H<T, R, F> where F: FnOnce(T) -> R {
            val: T,
            fun: F,
        };
    }

    dbg!(A::new(0, "".into()),
         B::new(1, 2),
         C::new(""),
         D::new("", 1),
         E::new(1, 2),
         F::new(vec![1].into_iter()),
         G::new("", ""),);
    {
        let h = H::new(0, |a: u32| (a + 1) as u64);
        dbg!((h.fun)(h.val));
    }
}
