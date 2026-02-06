mod adapters;
mod parse_error;
pub mod iter_parser;
pub use parse_error::ParseError;
pub use adapters::*;

use proc_macro::TokenTree;
use std::marker::PhantomData;

pub type Input<'a> = &'a [TokenTree];
pub type ParseResult<'src, T> = Result<(Input<'src>, T), ParseError>;

pub trait Parser<'src> {
    type Output;
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output>;

    fn or_not(self) -> OrNot<Self>
    where
        Self: Sized,
    {
        OrNot(self)
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        Map {
            parser: self,
            mapper: f,
        }
    }

    fn try_map<F, U>(self, f: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<U, ParseError>,
    {
        TryMap {
            parser: self,
            mapper: f,
        }
    }

    fn repeated(self) -> iter_parser::Repeated<Self>
    where
        Self: Sized,
    {
        iter_parser::Repeated {
            parser: self,
            at_least: 0,
            at_most: u32::MAX,
        }
    }

    fn fold_left<I, F>(self, i: I, f: F) -> iter_parser::FoldLeft<I, Self, F>
    where
        Self: Sized,
    {
        iter_parser::FoldLeft {
            single_parser: self,
            iter_parser: i,
            folder: f,
        }
    }

    fn try_fold_left<I, F>(self, i: I, f: F) -> iter_parser::TryFoldLeft<I, Self, F>
    where
        Self: Sized,
    {
        iter_parser::TryFoldLeft {
            single_parser: self,
            iter_parser: i,
            folder: f,
        }
    }

    fn or<P>(self, other: P) -> Or<Self, P>
    where
        Self: Sized,
        P: Parser<'src, Output = Self::Output>,
    {
        Or(self, other)
    }

    fn then<P: Parser<'src>>(self, other: P) -> Then<Self, P>
    where
        Self: Sized,
    {
        Then(self, other)
    }
}

#[derive(Clone)]
pub struct Any;

pub fn any() -> Any {
    Any
}

impl<'src> Parser<'src> for Any
{
    type Output = &'src TokenTree;
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        match input {
            [] => Err(ParseError::expected("anything", "end of input".to_string())),
            [first, tail @ ..] => Ok((tail, first)),
        }
    }
}

#[derive(Clone)]
pub struct End;

impl<'src> Parser<'src> for End
{
    type Output = ();
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        match input {
            [] => Ok((input, ())),
            [something, ..] => Err(ParseError::expected("end", something.to_string())),
        }
    }
}

#[derive(Clone)]
pub struct InGroup<P>(pub P);

impl<'src, O, P> Parser<'src> for InGroup<P>
where
    P: for<'a> Parser<'a, Output = O>,
{
    type Output = O;
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let (group, tail) = match input {
            [TokenTree::Group(g), tail @ ..] => (g, tail),
            [] => return Err(ParseError::expected("token group", "end of input".to_string())),
            [first, ..] => return Err(ParseError::expected("token group", first.to_string())),
        };

        let v = group.stream().into_iter().collect::<Vec<_>>();
        return match self.0.parse(v.as_slice()) {
            Ok((_tail, o)) => Ok((tail, o)),
            Err(e) => Err(e),
        };
    }
}
