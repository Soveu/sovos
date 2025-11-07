use proc_macro::{TokenStream, TokenTree, Ident, Span};
use crate::generating::{ToLiteral, Generator};

pub struct TokenSink(TokenStream);

impl TokenSink {
    pub fn new() -> Self {
        Self(TokenStream::new())
    }

    pub fn into_stream(self) -> TokenStream {
        self.0
    }

    pub fn add(&mut self, g: impl Generator) {
        g.generate(self);
    }

    pub fn add_ident(&mut self, id: &str) {
        self.extend_one(TokenTree::Ident(Ident::new(id, Span::call_site())));
    }

    pub fn add_literal<T: ToLiteral>(&mut self, lit: T) {
        self.extend_one(TokenTree::Literal(lit.to_literal()));
    }

    pub fn extend_one(&mut self, tt: TokenTree) {
        self.0.extend([tt]);
    }
}
