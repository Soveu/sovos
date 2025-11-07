extern crate proc_macro;
use proc_macro::TokenStream;

pub mod parser;
pub mod parsing;
pub mod generating;

fn output_stage<P>(ts: TokenStream, p: P) -> String
where
    P: for<'a> parsing::Parser<'a>,
    for<'a> <P as parsing::Parser<'a>>::Output: std::fmt::Debug,
{
    let input = ts.into_iter().collect::<Vec<_>>();
    let result = p.parse(&input).map(|x| x.1);
    return format!("{:#?}", result);
}

pub fn __output_parsing_stage(ts: TokenStream) -> String {
    output_stage(ts, parser::stmt::derive_parser())
}
pub fn __output_type_parse(ts: TokenStream) -> String {
    output_stage(ts, parser::typ::type_parser())
}
pub fn __output_expr_parse(ts: TokenStream) -> String {
    output_stage(ts, parser::expr::expr_parser())
}

