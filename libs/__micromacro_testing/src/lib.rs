extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree, Literal};

fn output_literal(ts: TokenStream, f: fn(TokenStream) -> String) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::string(
        &f(ts)
    )))
}

#[proc_macro]
pub fn output_tokenstream_as_string(ts: TokenStream) -> TokenStream {
    output_literal(ts, |_ts| _ts.to_string())
}

#[proc_macro]
pub fn output_parsing_stage(ts: TokenStream) -> TokenStream {
    output_literal(ts, micromacro::__output_parsing_stage)
}

#[proc_macro]
pub fn output_expr(ts: TokenStream) -> TokenStream {
    output_literal(ts, micromacro::__output_expr_parse)
}

#[proc_macro]
pub fn output_type(ts: TokenStream) -> TokenStream {
    output_literal(ts, micromacro::__output_type_parse)
}
