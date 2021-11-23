macro_rules! impl_pagelevel {
    {
        pub struct $structname:ident,
        pub struct $flagsname:ident = {$(
            $(#[$attr:meta])*
            $fname:ident = $bit:expr,
        )*}
    } => {
        #[repr(transparent)]
        pub struct $flagsname(pub u64);

        $crate::impl_bits! {
            $flagsname = {
                $(
                $(#[$attr])*
                $fname = $bit,
                )*
            }
        }

        impl Bits for $flagsname {
            fn as_u64(&self) -> u64 {
                self.0
            }
            unsafe fn from_u64_unchecked(x: u64) -> Self {
                Self(x)
            }
        }

        #[repr(transparent)]
        pub struct $structname(u64);

        //impl ::core::marker::Copy for $structname {}
        impl ::core::clone::Clone for $structname {
            fn clone(&self) -> Self {
                Self(self.0)
            }
        }

        impl $structname {
            pub fn new(addr: PhysAddr, flags: $flagsname) -> Self {
                Self(flags.as_u64() | addr.as_u64())
            }
        }
        impl Bits for $structname {
            fn as_u64(&self) -> u64 {
                self.0
            }
            unsafe fn from_u64_unchecked(x: u64) -> Self {
                Self(x)
            }
        }
        impl Entry for $structname {
            type Flags = $flagsname;
            const ZEROED: Self = Self(0);
        }
        impl ::core::convert::AsRef<u64> for $structname {
            fn as_ref(&self) -> &u64 {
                &self.0
            }
        }
    }
}
