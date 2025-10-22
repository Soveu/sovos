use proc_macro::Literal;

pub trait ToLiteral {
    fn to_literal(self) -> Literal;
}

impl ToLiteral for u8 { fn to_literal(self) -> Literal { Literal::u8_suffixed(self) } }
impl ToLiteral for u16 { fn to_literal(self) -> Literal { Literal::u16_suffixed(self) } }
impl ToLiteral for u32 { fn to_literal(self) -> Literal { Literal::u32_suffixed(self) } }
impl ToLiteral for u64 { fn to_literal(self) -> Literal { Literal::u64_suffixed(self) } }
impl ToLiteral for usize { fn to_literal(self) -> Literal { Literal::usize_suffixed(self) } }

impl ToLiteral for i8 { fn to_literal(self) -> Literal { Literal::i8_suffixed(self) } }
impl ToLiteral for i16 { fn to_literal(self) -> Literal { Literal::i16_suffixed(self) } }
impl ToLiteral for i32 { fn to_literal(self) -> Literal { Literal::i32_suffixed(self) } }
impl ToLiteral for i64 { fn to_literal(self) -> Literal { Literal::i64_suffixed(self) } }
impl ToLiteral for isize { fn to_literal(self) -> Literal { Literal::isize_suffixed(self) } }

// Booleans are idents, byte characters would conflict with u8
impl ToLiteral for f32 { fn to_literal(self) -> Literal { Literal::f32_suffixed(self) } }
impl ToLiteral for f64 { fn to_literal(self) -> Literal { Literal::f64_suffixed(self) } }
impl ToLiteral for char { fn to_literal(self) -> Literal { Literal::character(self) } }
impl ToLiteral for &str { fn to_literal(self) -> Literal { Literal::string(self) } }
impl ToLiteral for &[u8] { fn to_literal(self) -> Literal { Literal::byte_string(self) } }
