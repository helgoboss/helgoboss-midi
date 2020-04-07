/// Creates a new type which is represented by a primitive type but has a restricted value range.
macro_rules! newtype {
    ($name: ident, $repr: ty, $max: literal, $factory: ident) => {
        #[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
        pub struct $name(pub(crate) $repr);

        impl $name {
            pub const MIN: Self = Self(0);

            pub const MAX: Self = Self($max);

            pub const COUNT: $repr = $max + 1;

            pub fn is_valid<T: PartialOrd + From<$repr>>(number: T) -> bool {
                number < Self::COUNT.into()
            }

            /// Panics if given number is greater than MAX!
            // TODO Not sure if this is a good idea. See
            //  https://doc.rust-lang.org/std/num/struct.NonZeroU8.html
            pub fn new(number: $repr) -> Self {
                assert!(Self::is_valid(number));
                Self(number)
            }

            pub const unsafe fn new_unchecked(number: $repr) -> Self {
                Self(number)
            }
        }

        /// Panics if given number is greater than MAX!
        pub fn $factory(number: $repr) -> $name {
            $name::new(number)
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

/// Creates a `TryFrom` trait implementation from a primitive type with a higher
/// value range to a newtype.
macro_rules! impl_try_from_primitive_to_newtype {
    ($from: ty, $into: ty) => {
        impl std::convert::TryFrom<$from> for $into {
            type Error = ();

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                if !Self::is_valid(value) {
                    return Err(());
                }
                Ok(Self(value as _))
            }
        }
    };
}
