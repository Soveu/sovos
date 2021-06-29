use impl_bits::impl_bits;

struct Flags(u32);

impl_bits! {
    Flags = {
        one = 1,
        two = 2,
        three = 3,
        zero = 0,
    }
}

impl Flags {
    pub const fn new() -> Self {
        Self(0)
    }
    pub const fn with_all_flags() -> Self {
        Self::__with_all_flags()
    }
}

#[test]
fn one() {
    let f = Flags::new();
    let f = f.set_one().set_zero();
    let f = f.clear_one();

    assert!(f.zero());
    assert!(!f.one());
    assert!(!f.two());
    assert!(!f.three());
}
