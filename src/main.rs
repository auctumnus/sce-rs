fn main() {
    sce::parse(
        r#"// abc
    A = b,c,d
    [A] > b / d"#,
    );
    println!("Hello, world!");
}
