use __micromacro_testing::output_parsing_stage;

#[test]
fn asdf() {
    let x = output_parsing_stage!(
        2 + 3 * 4
    );

    let expected = r#"Err(
    ParseError(
        "(\"expected \\\"ident\\\" got \\\"2\\\"\", \"expected \\\"ident\\\" got \\\"2\\\"\")",
    ),
)"#;

    println!("{}", x);
    assert_eq!(x, expected);
}
