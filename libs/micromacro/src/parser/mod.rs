use crate::parsing::{self, Parser, ParseError};
use proc_macro::TokenTree;

pub mod stmt;
pub mod typ;
pub mod expr;

#[derive(Clone, Debug)]
pub struct Todo;

fn ident(i: &'static str) -> impl for<'src> Parser<'src, Output = ()> + Clone {
    let f = move |tt: &TokenTree| -> Result<(), ParseError> {
        let ident = match tt {
            TokenTree::Ident(id) => id.to_string(),
            _ => return Err(ParseError::expected("ident", tt.to_string())),
        };
        if ident == i {
            return Ok(());
        }
        return Err(ParseError::expected(i, ident));
    };

    return parsing::any().try_map(f);
}

fn any_ident() -> impl for<'src> Parser<'src, Output = proc_macro::Ident> + Clone {
    let f = |tt: &TokenTree| {
        if let TokenTree::Ident(ident) = tt {
            return Ok(ident.clone());
        }
        return Err(ParseError::expected("any ident", tt.to_string()));
    };

    return parsing::any().try_map(f);
}
