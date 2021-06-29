#![no_std]

#[doc(hidden)]
pub use paste;

#[macro_export]
macro_rules! impl_bits {
    {
        $structname:ident = {$(
            $(#[$attr:meta])*
            $fname:ident = $bit:expr,
        )*}
    } => {
        impl $structname {
            #[inline(always)]
            const fn __with_all_flags() -> Self {
                Self(0 $(| (1<<$bit))*)
            }

            $(
            $(#[$attr])*
            #[inline(always)]
            pub const fn $fname(self) -> bool {
                (self.0 >> $bit) & 1 == 1
            }

            $crate::paste::paste! {
                #[inline(always)]
                pub const fn [<set_ $fname>](self) -> Self {
                    let mask = (1 << $bit);
                    Self(self.0 | mask)
                }
                #[inline(always)]
                pub const fn [<clear_ $fname>](self) -> Self {
                    let mask = !(1 << $bit);
                    Self(self.0 & mask)
                }
            }
            )*
        }

        impl ::core::marker::Copy for $structname {}
        impl ::core::clone::Clone for $structname {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl ::core::fmt::Debug for $structname {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("[")?;

                $(
                if self.$fname() {
                    let bitname = concat!(stringify!($fname), ", ");
                    f.write_str(bitname)?;
                }
                )*

                f.write_str("]")
            }
        }
    }
}
