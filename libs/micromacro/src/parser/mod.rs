use crate::parsing::{self, Parser, ParseError};

use proc_macro::TokenTree;

#[derive(Clone, Copy, Debug)]
pub enum ReprInt {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

#[derive(Clone, Copy, Debug)]
pub enum ReprType {
    Rust,
    C,
    Transparent,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Repr {
    pub typ: Option<ReprType>,
    pub int: Option<ReprInt>,
    pub align: Option<u64>,
    pub packed: Option<u64>,
}

fn _merge<T>(lhs: Option<T>, rhs: Option<T>) -> Result<Option<T>, ()> {
    match (lhs, rhs) {
        (None, None) => Ok(None),
        (Some(x), None) => Ok(Some(x)),
        (None, Some(x)) => Ok(Some(x)),
        (Some(_), Some(_)) => Err(()),
    }
}

impl Repr {
    fn merge(self, other: Self) -> Result<Self, ParseError> {
        Ok(Self {
            typ: _merge(self.typ, other.typ).map_err(|_| ParseError::cannot_merge("typ"))?,
            int: _merge(self.int, other.int).map_err(|_| ParseError::cannot_merge("int"))?,
            align: _merge(self.align, other.align).map_err(|_| ParseError::cannot_merge("align"))?,
            packed: _merge(self.packed, other.packed).map_err(|_| ParseError::cannot_merge("packed"))?,
        })
    }

    fn parser() -> impl for<'src> Parser<'src, Output = Self> + Clone {
        let ftyp = |tt: &TokenTree| -> Result<ReprType, ParseError> {
            let i = match tt {
                TokenTree::Ident(ident) => ident.to_string(),
                x => return Err(ParseError::expected("Rust, C, transparent", x.to_string())),
            };
            let o = match i.as_str() {
                "C" => ReprType::C,
                "Rust" => ReprType::Rust,
                "transparent" => ReprType::Transparent,
                _ => return Err(ParseError::expected("Rust, C, transparent", i)),
            };
            return Ok(o);
        };
        let typ = parsing::any().try_map(ftyp);

        let fint = move |tt: &TokenTree| {
            let i = match tt {
                TokenTree::Ident(ident) => ident.to_string(),
                x => return Err(ParseError::expected("([iu](8|16|32|64))", x.to_string())),
            };
            let o = match i.as_str() {
                "u8" => ReprInt::U8,
                "u16" => ReprInt::U16,
                "u32" => ReprInt::U32,
                "u64" => ReprInt::U64,
                "i8" => ReprInt::I8,
                "i16" => ReprInt::I16,
                "i32" => ReprInt::I32,
                "i64" => ReprInt::I64,
                _ => return Err(ParseError::expected("([iu](8|16|32|64))", i)),
            };
            return Ok(o);
        };
        let int = parsing::any().try_map(fint);

        let typ = typ.map(|typ| Repr { typ: Some(typ), ..Repr::default() });
        let int = int.map(|int| Repr { int: Some(int), ..Repr::default() });
        let repr = typ.or(int);

        repr
            .repeated_fold(Self::merge)
            .map(|seif| Repr { typ: Some(seif.typ.unwrap_or(ReprType::Rust)), ..seif })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclType {
    Struct,
    Union,
}

#[derive(Clone, Debug)]
pub struct Decl {
    pub typ: DeclType,
    pub is_pub: bool,
    pub name: proc_macro::Ident,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub is_pub: bool,
    pub name: proc_macro::Ident,
    pub typ: proc_macro::Ident,
}

#[derive(Clone, Debug)]
pub struct Type {
    pub repr: Repr,
    pub decl: Decl,
    pub fields: Vec<Field>,
}

fn punct(exp: char) -> impl for<'src> Parser<'src, Output = ()> + Clone {
    let f = move |tt: &TokenTree| -> Result<(), ParseError> {
        if let TokenTree::Punct(p) = tt {
            if p == &exp {
                return Ok(());
            }
        }
        return Err(ParseError::expected("punct", tt.to_string()));
    };

    parsing::any().try_map(f)
}

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
pub struct Todo;

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
    Array { typ: Box<_Type>, size: Box<Expr> },
    Function { args: Vec<_Type>, ret: Box<_Type> },
}

pub fn type_parser() -> impl for<'src> Parser<'src, Output = _Type> {
    parsing::End.map(|_| -> _Type { todo!() })
}

pub fn derive_parser() -> impl for<'src> Parser<'src, Output = Type> {
    let repr = punct('#')
        .then(parsing::InGroup(
            ident("repr")
                .then(parsing::InGroup(
                    Repr::parser().then(parsing::End)
                ))
                .then(parsing::End)
        ))
        .map(|(_, ((_, (r, _)), _))| r);

    let repr = repr.or_not().map(Option::unwrap_or_default);

    let ppub = ident("pub").or_not().map(|x| x.is_some());

    let struct_or_union = ident("struct")
        .map(|_| DeclType::Struct)
        .or(ident("union").map(|_| DeclType::Union));

    let decl = ppub
        .clone()
        .then(struct_or_union)
        .then(any_ident())
        .map(|((is_pub, typ), name)| Decl { is_pub, typ, name });

    let field = ppub
        .then(any_ident())
        .then(punct(':'))
        .then(any_ident())
        .then(punct(','))
        .map(|((((is_pub, name), _), typ), _)| Field { is_pub, name, typ });

    let fields = parsing::InGroup(field.repeated().then(parsing::End));

    repr
        .then(decl)
        .then(fields)
        .then(parsing::End)
        .map(|(((repr, decl), (fields, _)), _)| Type { repr, decl, fields })
}
