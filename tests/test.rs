#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/struct_new1.rs");
    t.pass("tests/ui/struct_new2.rs");
    t.pass("tests/ui/struct_new2-multi.rs");
}
