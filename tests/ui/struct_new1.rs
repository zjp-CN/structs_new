#![allow(unused)]
use proc::struct_new;

fn main() {
    struct_new! {
        #[derive(Debug)]
        struct A {
            foo: u8,
            pub bar: String,
            abc: u8 = 255,
        }
    }

    dbg!(A::new(0, "".into()));

    struct_new! {
        #[derive(Debug)]
        struct B<T> {
            foo: u8,
            pub bar: T,
            abc: u8 = 255,
        }
    }

    dbg!(B::new(1, 2));

    struct_new! {
        #[derive(Debug)]
        struct C<'a> {
            foo: &'a str,
            abc: u8 = 255,
        }
    }

    dbg!(C::new(""));

    struct_new! {
        #[derive(Debug)]
        struct D<'a, T> {
            foo: &'a str,
            pub bar: T,
            abc: u8 = 255,
        }
    }

    dbg!(D::new("", 1));

    struct_new! {
        #[derive(Debug)]
        struct E<T> where T: Copy {
            foo: u8,
            pub bar: T,
            abc: u8 = 255,
        }
    }

    dbg!(E::new(1, 2));

    struct_new! {
        #[derive(Debug)]
        struct F<I: Iterator> where I::Item: Copy {
            pub bar: I,
            abc: u8 = 255,
        }
    }

    dbg!(F::new(vec![1].into_iter()));

    struct_new! {
        #[derive(Debug)]
        struct G<'a, 'b, 'c: 'a+'b> {
            foo: &'c str,
            pub bar: &'a str,
            abc: &'b str = "",
        }
    }

    dbg!(G::new("", ""));
}
