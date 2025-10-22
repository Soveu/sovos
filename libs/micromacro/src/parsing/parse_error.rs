#[derive(Debug)]
pub struct ParseError(pub String);

impl ParseError {
    pub fn expected(exp: &'static str, got: String) -> Self {
        Self(format!("expected {:?} got {:?}", exp, got))
    }

    pub fn cannot_merge(s: &'static str) -> Self {
        Self(s.to_string())
    }

    pub fn merge(lhs: Self, rhs: Self) -> Self {
        Self(format!("({:?}, {:?})", lhs.0, rhs.0))
    }
}
