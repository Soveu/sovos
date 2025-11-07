use crate::parsing::{self, Parser};
use crate::parser::Todo; 

#[derive(Clone, Debug)]
pub enum PrimitiveType {
    Bool,
    Never,
    Char,
    Str,

    Int8,
    Int16,
    Int32,
    Int64,
    Int128,

    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,

    Float16,
    Float32,
    Float64,
    Float128,
}

#[derive(Clone, Debug)]
pub enum _Type {
    Primitive(PrimitiveType),
    Path(Todo),
    Impl(Todo),
    Dyn(Todo),

    Reference { is_mut: bool, lifetime: Todo, typ: Box<_Type> },
    Pointer { is_mut: bool, typ: Box<_Type> },

    Tuple(Vec<_Type>),
    Slice(Box<_Type>),
    Array { typ: Box<_Type>, size: Box<crate::parser::expr::Expr> },
    Function { args: Vec<_Type>, ret: Box<_Type> },
}

pub fn type_parser() -> impl for<'src> Parser<'src, Output = _Type> {
    parsing::End.map(|_| -> _Type { todo!() })
}
