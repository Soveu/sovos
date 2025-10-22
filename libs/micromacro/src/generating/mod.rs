mod token_sink;
mod to_literal;

pub use token_sink::TokenSink;
pub use to_literal::ToLiteral;

use proc_macro::TokenTree;

pub trait Generator: Sized {
    fn generate(self, sink: &mut TokenSink);

    fn then<G: Generator>(self, other: G) -> Then<Self, G> {
        Then(self, other)
    }
}

impl<F> Generator for F
where
    F: FnOnce(&mut TokenSink),
{
    fn generate(self, sink: &mut TokenSink) {
        (self)(sink);
    }
}

impl Generator for TokenTree {
    fn generate(self, sink: &mut TokenSink) {
        sink.extend_one(self);
    }
}

#[derive(Clone, Copy)]
pub struct Nothing;

impl Generator for Nothing {
    fn generate(self, _sink: &mut TokenSink) { }
}

pub struct Then<G1, G2>(G1, G2);

impl<G1, G2> Generator for Then<G1, G2>
where
    G1: Generator,
    G2: Generator,
{
    fn generate(self, sink: &mut TokenSink) {
        self.0.generate(sink);
        self.1.generate(sink);
    }
}
