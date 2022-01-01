#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/struct_new1.rs");
}
