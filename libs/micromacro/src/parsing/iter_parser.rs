use crate::parsing::{Parser, Input, ParseResult, ParseError};

pub enum IterParseResult<'src, T> {
    Next(Input<'src>, T),
    Err(ParseError),
    End,
}

pub trait IterParser<'src> {
    type Output;
    type State: Default;

    fn parse_next(
        &self,
        state: &mut Self::State,
        input: Input<'src>,
    ) -> IterParseResult<'src, Self::Output>;

    fn fold_right<P, F, R>(self, p: P, f: F) -> FoldRight<Self, P, F>
    where
        Self: Sized,
    {
        FoldRight {
            iter_parser: self,
            single_parser: p,
            folder: f,
        }
    }

    fn collect_vec(self) -> CollectVec<Self>
    where
        Self: Sized,
    {
        CollectVec(self)
    }
}

#[derive(Clone)]
pub struct Repeated<P> {
    pub parser: P,
    pub at_least: u32,
    pub at_most: u32,
}

impl<'src, P: Parser<'src>> IterParser<'src> for Repeated<P> {
    type Output = P::Output;
    type State = u32;

    fn parse_next(
        &self,
        state: &mut Self::State,
        input: Input<'src>,
    ) -> IterParseResult<'src, Self::Output> {
        if *state > self.at_most {
            return IterParseResult::Err(ParseError::expected("at most", String::new()));
        }

        if let Ok((inp, x)) = self.parser.parse(input) {
            *state += 1;
            return IterParseResult::Next(inp, x);
        }

        if *state < self.at_least {
            return IterParseResult::Err(ParseError::expected("at least", String::new()));
        }

        return IterParseResult::End;
    }
}

#[derive(Clone)]
pub struct FoldRight<I, P, F> {
    pub iter_parser: I,
    pub single_parser: P,
    pub folder: F,
}

impl<'src, I, P, F> FoldRight<I, P, F>
where
    I: IterParser<'src>,
    P: Parser<'src>,
    F: Fn(P::Output, I::Output) -> P::Output,
{
    fn _parse(&self, state: &mut I::State, input: Input<'src>) -> ParseResult<'src, P::Output> {
        match self.iter_parser.parse_next(state, input) {
            IterParseResult::End | IterParseResult::Err(_) => self.single_parser.parse(input),
            IterParseResult::Next(inp2, to_be_folded) => match self._parse(state, inp2) {
                Ok((inp3, acc)) => Ok((inp3, ((self.folder)(acc, to_be_folded)))),
                e => e,
            },
        }
    }
}

impl<'src, I, P, F> Parser<'src> for FoldRight<I, P, F>
where
    I: IterParser<'src>,
    P: Parser<'src>,
    F: Fn(P::Output, I::Output) -> P::Output,
{
    type Output = P::Output;

    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let mut state = I::State::default();
        return self._parse(&mut state, input);
    }
}

#[derive(Clone)]
pub struct TryFoldLeft<I, P, F> {
    pub single_parser: P,
    pub iter_parser: I,
    pub folder: F,
}

impl<'src, I, P, F> TryFoldLeft<I, P, F>
where
    I: IterParser<'src>,
    P: Parser<'src>,
    F: Fn(P::Output, I::Output) -> Result<P::Output, ParseError>,
{
    fn _parse(
        &self,
        state: &mut I::State,
        input: Input<'src>,
        acc: P::Output,
    ) -> ParseResult<'src, P::Output> {
        match self.iter_parser.parse_next(state, input) {
            IterParseResult::End | IterParseResult::Err(_) => Ok((input, acc)),
            IterParseResult::Next(inp2, to_be_folded) => self._parse(
                state,
                inp2,
                (self.folder)(acc, to_be_folded)?,
            ),
        }
    }
}

impl<'src, I, P, F> Parser<'src> for TryFoldLeft<I, P, F>
where
    I: IterParser<'src>,
    P: Parser<'src>,
    F: Fn(P::Output, I::Output) -> Result<P::Output, ParseError>,
{
    type Output = P::Output;

    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let mut state = I::State::default();
        let (inp2, first) = self.single_parser.parse(input)?;
        return self._parse(&mut state, inp2, first);
    }
}


#[derive(Clone)]
pub struct FoldLeft<I, P, F> {
    pub single_parser: P,
    pub iter_parser: I,
    pub folder: F,
}

impl<'src, I, P, F> FoldLeft<I, P, F>
where
    I: IterParser<'src>,
    P: Parser<'src>,
    F: Fn(P::Output, I::Output) -> P::Output,
{
    fn _parse(
        &self,
        state: &mut I::State,
        input: Input<'src>,
        acc: P::Output,
    ) -> ParseResult<'src, P::Output> {
        match self.iter_parser.parse_next(state, input) {
            IterParseResult::End | IterParseResult::Err(_) => Ok((input, acc)),
            IterParseResult::Next(inp2, to_be_folded) => self._parse(
                state,
                inp2,
                (self.folder)(acc, to_be_folded),
            ),
        }
    }
}

impl<'src, I, P, F> Parser<'src> for FoldLeft<I, P, F>
where
    I: IterParser<'src>,
    P: Parser<'src>,
    F: Fn(P::Output, I::Output) -> P::Output,
{
    type Output = P::Output;

    fn parse(&self, input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let mut state = I::State::default();
        let (inp2, first) = self.single_parser.parse(input)?;
        return self._parse(&mut state, inp2, first);
    }
}

#[derive(Clone)]
pub struct CollectVec<I>(pub I);

impl<'src, I> Parser<'src> for CollectVec<I>
where
    I: IterParser<'src>,
{
    type Output = Vec<I::Output>;

    fn parse(&self, mut input: Input<'src>) -> ParseResult<'src, Self::Output> {
        let mut v = Vec::new();
        let mut s = Default::default();

        loop {
            match self.0.parse_next(&mut s, input) {
                IterParseResult::End => return Ok((input, v)),
                IterParseResult::Err(e) => return Err(e),
                IterParseResult::Next(new_input, item) => {
                    v.push(item);
                    input = new_input;
                    continue;
                },
            }
        }
    }
}
