use super::{Parser, Input, ParseResult, ParseError};

use std::marker::PhantomData;

#[derive(Clone)]
pub struct OrNot<T>(pub T);

impl<'src, T: Parser<'src>> Parser<'src> for OrNot<T> {
    type Output = Option<T::Output>;
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        match self.0.parse(input) {
            Ok((i, o)) => Ok((i, Some(o))),
            Err(_) => Ok((input, None)),
        }
    }
}

#[derive(Clone)]
pub struct Map<P, F> {
    pub parser: P,
    pub mapper: F,
}

impl<'src, P, F, U> Parser<'src> for Map<P, F>
where
    P: Parser<'src>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        match self.parser.parse(input) {
            Err(e) => Err(e),
            Ok((tail, o)) => Ok((tail, (self.mapper)(o))),
        }
    }
}

#[derive(Clone)]
pub struct TryMap<P, F> {
    pub parser: P,
    pub mapper: F,
}

impl<'src, P, F, U> Parser<'src> for TryMap<P, F>
where
    P: Parser<'src>,
    F: Fn(P::Output) -> Result<U, ParseError>,
{
    type Output = U;
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let (tail, o1) = self.parser.parse(input)?;
        let o2 = (self.mapper)(o1)?;
        return Ok((tail, o2));
    }
}

#[derive(Clone)]
pub struct Repeated<P>(pub P);

impl<'src, P: Parser<'src>> Parser<'src> for Repeated<P> {
    type Output = Vec<P::Output>;
    fn parse(&self, mut input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let mut v = Vec::new();

        while let Ok((i, o)) = (self.0).parse(input) {
            input = i;
            v.push(o);
        }

        return Ok((input, v));
    }
}

#[derive(Clone)]
pub struct RepeatedFold<P, F, U> {
    pub parser: P,
    pub folder: F,
    pub phantom: PhantomData<U>,
}

impl<'src, P, F, U> Parser<'src> for RepeatedFold<P, F, U>
where
    P: Parser<'src>,
    F: Fn(U, P::Output) -> Result<U, ParseError>,
    U: Default,
{
    type Output = U;
    fn parse(&self, mut input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let mut u = U::default();

        while let Ok((i, o)) = self.parser.parse(input) {
            input = i;
            u = (self.folder)(u, o)?;
        }

        return Ok((input, u));
    }
}

#[derive(Clone)]
pub struct Then<P1, P2>(pub P1, pub P2);

impl<'src, P1, P2> Parser<'src> for Then<P1, P2>
where
    P1: Parser<'src>,
    P2: Parser<'src>,
{
    type Output = (P1::Output, P2::Output);
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let (i, o1) = self.0.parse(input)?;
        let (i, o2) = self.1.parse(i)?;
        return Ok((i, (o1, o2)));
    }
}

#[derive(Clone)]
pub struct Or<P1, P2>(pub P1, pub P2);

impl<'src, P1, P2> Parser<'src> for Or<P1, P2>
where
    P1: Parser<'src>,
    P2: Parser<'src, Output = P1::Output>,
{
    type Output = P1::Output;
    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let err0 = match self.0.parse(input) {
            Err(e) => e,
            Ok(x) => return Ok(x),
        };

        let err1 = match self.1.parse(input) {
            Err(e) => e,
            Ok(x) => return Ok(x),
        };

        return Err(ParseError::merge(err0, err1));
    }
}
