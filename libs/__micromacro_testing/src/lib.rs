extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree, Literal};

#[proc_macro]
pub fn output_tokenstream_as_string(ts: TokenStream) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::string(
        &ts.to_string()
    )))
}

#[proc_macro]
pub fn output_parsing_stage(ts: TokenStream) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::string(
        &micromacro::__output_parsing_stage(ts)
    )))
}
