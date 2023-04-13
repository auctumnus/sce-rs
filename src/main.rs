fn main() {
    if let Ok(ast) = sce::parse(
        r#"A = a,b,c
        A += d
        A -= c
        B = d"#,
    ) {
        let words = vec!["abc"].iter().map(|s| s.to_string()).collect();
        sce::apply::apply(ast, words, vec![], String::from("'"));
    };
    println!("Hello, world!");
}
