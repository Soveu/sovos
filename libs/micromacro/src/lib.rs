extern crate proc_macro;
use proc_macro::TokenStream;

pub mod parser;
pub mod parsing;
pub mod generating;

pub fn __output_parsing_stage(ts: TokenStream) -> String {
    use parsing::Parser;

    let input = ts.into_iter().collect::<Vec<_>>();
    let p = parser::derive_parser();
    let result = p.parse(&input).map(|x| x.1);

    return format!("{:#?}", result);
}

pub fn __output_type_parse(ts: TokenStream) -> String {
    todo!()
}

pub fn __output_expr_parse(ts: TokenStream) -> String {
    todo!()
}

