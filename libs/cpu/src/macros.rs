macro_rules! impl_bits {
    {
        struct $structname:ident(new = $new:expr),

        $(
            $(#[$attr:meta])*
            $fname:ident = $bit:expr,
        )*
    } => {
        impl $structname {
            #[inline(always)]
            pub const fn new() -> Self {
                Self($new)
            }
            #[inline(always)]
            pub const fn with_all_flags() -> Self {
                Self($new $(| (1<<$bit))*)
            }

            $(

            $(#[$attr])*
            #[inline(always)]
            pub const fn $fname(self) -> bool {
                (self.0 >> $bit) & 1 == 1
            }

            ::paste::paste! {
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

        impl const Default for $structname {
            fn default() -> $structname {
                $structname::new()
            }
        }

        impl core::fmt::Debug for $structname {
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

macro_rules! impl_pagelevel {
    {
        pub struct $structname:ident,
        pub struct $flagsname:ident = {$(
            $(#[$attr:meta])*
            $fname:ident = $bit:expr,
        )*}
    } => {
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        pub struct $flagsname(u64);

        impl_bits! {
            struct $flagsname(new = 0),

            $(
            $(#[$attr])*
            $fname = $bit,
            )*
        }

        impl const Bits for $flagsname {
            fn as_u64(&self) -> u64 {
                self.0
            }
            unsafe fn from_u64_unchecked(x: u64) -> Self {
                Self(x)
            }
        }

        #[repr(transparent)]
        #[derive(Clone, Copy)]
        pub struct $structname(u64);
        
        impl $structname {
            pub const fn new(addr: PhysAddr, flags: $flagsname) -> Self {
                Self(flags.as_u64() | addr.as_u64())
            }
        }
        impl const Default for $structname {
            fn default() -> Self {
                Self(0)
            }
        }
        impl const Bits for $structname {
            fn as_u64(&self) -> u64 {
                self.0
            }
            unsafe fn from_u64_unchecked(x: u64) -> Self {
                Self(x)
            }
        }
        impl const Entry for $structname {
            type Flags = $flagsname;
            const ZEROED: Self = Self(0);
        }
    }
}
