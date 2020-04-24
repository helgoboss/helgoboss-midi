use derive_more::{Display, Error};

#[derive(Debug, Clone, Eq, PartialEq, Display, Error)]
#[display(fmt = "value out of range")]
pub struct ValueOutOfRangeError;

/// Creates a new type which is represented by a primitive type but has a restricted value range.
// TODO Consider get() method (https://rust-lang.github.io/api-guidelines/naming.html)
//  mmh, not so good because it exposes once more the internal representation which is actually not
//  so important, and we want to prevent unwrapping as far as possible
// TODO Consider implementing bitwise operations
macro_rules! newtype {
    ($name: ident, $repr: ty, $max: literal, $factory: ident) => {
        #[cfg(feature = "serde")]
        use serde::{Deserialize, Serialize};

        #[derive(
            Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, derive_more::Display,
        )]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub struct $name(pub(crate) $repr);

        impl $name {
            pub const MIN: Self = Self(0);

            pub const MAX: Self = Self($max);

            fn is_valid<T: PartialOrd + From<$repr>>(number: T) -> bool {
                number <= $max.into()
            }

            /// Panics if given number is greater than MAX!
            //
            // - Okay to panic here if we document the preconditions. Also used in example at https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html
            // - NonZeroU8's new returns an Option, but that's probably pre-TryFrom era
            // - Not having a new() at all is probably not a good idea because new() is what most
            //   people look for first (C-CTOR convention)
            pub fn new(number: $repr) -> Self {
                assert!(Self::is_valid(number));
                Self(number)
            }

            // This is good practice
            pub const unsafe fn new_unchecked(number: $repr) -> Self {
                Self(number)
            }

            // This aligns with C-GETTER convention and to std::num::NonZeroU8
            pub fn get(self) -> $repr {
                self.0
            }
        }
    };
}

/// Creates a `From` trait implementation from another newtype which has the same or smaller value
/// range.
macro_rules! impl_from_newtype_to_newtype {
    ($from: ty, $into: ty) => {
        impl From<$from> for $into {
            fn from(value: $from) -> Self {
                Self(value.0 as _)
            }
        }
    };
}

/// Creates a `From` trait implementation from a newtype to a primitive type with a higher
/// value range.
macro_rules! impl_from_newtype_to_primitive {
    ($from: ty, $into: ty) => {
        impl From<$from> for $into {
            fn from(value: $from) -> Self {
                value.0 as _
            }
        }
    };
}

/// Creates a `From` trait implementation from a primitive with a lower value range to a newtype.
macro_rules! impl_from_primitive_to_newtype {
    ($from: ty, $into: ty) => {
        impl From<$from> for $into {
            fn from(value: $from) -> Self {
                Self(value as _)
            }
        }
    };
}

/// Creates a `TryFrom` trait implementation from a newtype with a higher value range to a newtype.
macro_rules! impl_try_from_newtype_to_newtype {
    ($from: ty, $into: ty) => {
        impl std::convert::TryFrom<$from> for $into {
            type Error = $crate::ValueOutOfRangeError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                if !Self::is_valid(value.0) {
                    return Err($crate::ValueOutOfRangeError);
                }
                Ok(Self(value.0 as _))
            }
        }
    };
}

/// Creates a `TryFrom` trait implementation from a primitive type with a higher value range to a
/// newtype.
macro_rules! impl_try_from_primitive_to_newtype {
    ($from: ty, $into: ty) => {
        impl std::convert::TryFrom<$from> for $into {
            type Error = $crate::ValueOutOfRangeError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                if !Self::is_valid(value) {
                    return Err($crate::ValueOutOfRangeError);
                }
                Ok(Self(value as _))
            }
        }
    };
}
