use crate::parsing::{self, Parser};
use crate::parser::Todo;

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Star,
    Minus,
    And,
    Try,
    Neg,
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add { assign: bool },
    Sub { assign: bool },
    Mul { assign: bool },
    Div { assign: bool },
    Mod { assign: bool },
    Xor { assign: bool },

    BitAnd { assign: bool },
    BitOr { assign: bool },
    BitShiftLeft { assign: bool },
    BitShiftRight { assign: bool },

    Eq,
    NotEq,
    Gt,
    Lt,
    Ge,
    Le,

    BoolAnd,
    BoolOr,

    Assign,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(proc_macro::Literal),
    Ident(proc_macro::Ident),

    Path(Todo),
    Underscore,

    IfElse { condition: proc_macro::Group, if_block: proc_macro::Group, else_block: proc_macro::Group },
    Loop(proc_macro::Group),
    Match { value: Box<Expr>, match_block: proc_macro::Group },

    BinaryOp { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> },
    UnaryOp { op: UnaryOp, item: Box<Expr> },
    As { expr: Box<Expr>, typ: Todo },
    Paren(Box<Expr>),
    Range(Todo),

    Array(Vec<Expr>),
    ArraySub { arr: Box<Expr>, index: Box<Expr> },

    Tuple(Vec<Expr>),
    TupleSub { tup: Box<Expr>, index: Todo },

    Struct { path: Todo, fields: Vec<(Option<proc_macro::Ident>, Expr)>, update: Option<Box<Expr>> },
    TupleStruct { path: Todo, fields: Vec<Expr> },

    Call { fun: Box<Expr>, args: Vec<Expr> },
    MethodCall { obj: Box<Expr>, method: proc_macro::Ident, args: Vec<Expr> },
    Field { obj: Box<Expr>, field: proc_macro::Ident },
}

pub fn expr_parser() -> impl for<'src> Parser<'src, Output = Expr> {
    parsing::End.map(|_| -> Expr { todo!() })
}


