#![no_std]

#[doc(hidden)]
pub use paste;

// Scraped from DebugList, as it only takes Debug parameters, not Display :(
pub struct DisplayList<'a, 'b: 'a> {
    pub fmt: &'a mut core::fmt::Formatter<'b>,
    pub result: core::fmt::Result,
    pub has_fields: bool,
}

impl<'a, 'b: 'a> DisplayList<'a, 'b> {
    pub fn new(fmt: &'a mut core::fmt::Formatter<'b>) -> Self {
        let result = fmt.write_str("[");
        Self {
            fmt,
            result,
            has_fields: false,
        }
    }

    pub fn entry(&mut self, entry: &dyn core::fmt::Display)
    {
        if self.result.is_err() {
            return;
        }

        self.result = self.result.and_then(|_| {
            if self.has_fields {
                self.fmt.write_str(", ")?;
            }
            entry.fmt(self.fmt)
        });

        self.has_fields = true;
    }

    pub fn finish(self) -> core::fmt::Result {
        self.result.and_then(|_| self.fmt.write_str("]"))
    }
}

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
                let mut f = $crate::DisplayList::new(f);

                $(
                if self.$fname() {
                    let bitname = stringify!($fname);
                    f.entry(&bitname);
                }
                )*

                f.finish()
            }
        }
    }
}
