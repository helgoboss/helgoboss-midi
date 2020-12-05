/// An error which can occur when converting from a type with a greater value range to one with a
/// smaller one.
#[derive(Clone, Eq, PartialEq, Debug, derive_more::Display)]
#[display(fmt = "converting to type with smaller value range failed")]
pub struct TryFromGreaterError(pub(crate) ());

impl core_error::Error for TryFromGreaterError {}

/// An error which can occur when parsing a string to one of the MIDI integer types.
#[derive(Clone, Eq, PartialEq, Debug, derive_more::Display)]
#[display(fmt = "parsing string to MIDI type failed")]
pub struct ParseIntError(pub(crate) ());

impl core_error::Error for ParseIntError {}

// use core::fmt;

/// Creates a new type which is represented by a primitive type but has a restricted value range.
macro_rules! newtype {
    (
        $(#[$outer:meta])*
        name = $name: ident,
        repr = $repr: ty,
        max = $max: literal
    ) => {
        $(#[$outer])*
        #[derive(
            Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, derive_more::Display,
        )]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(try_from = "u16")
        )]
        pub struct $name(pub(crate) $repr);

        impl $name {
            /// The smallest value that can be represented by this type.
            pub const MIN: $name = $name(0);

            /// The largest value that can be represented by this type.
            pub const MAX: $name = $name($max);

            fn is_valid<T: PartialOrd + From<$repr>>(number: T) -> bool {
                number >= 0.into() && number <= $max.into()
            }

            doc_comment::doc_comment! {
                concat!(
"Creates a ", stringify!($name), ".

# Panics

This function panics if `value` is greater than ", $max, "."
                ),
                pub fn new(value: $repr) -> $name {
                    assert!(
                        $name::is_valid(value),
                        // requires std
                        //format!("{} is not a valid value", value)
                        "Not a valid value"
                    );
                    $name(value)
                }
            }

            doc_comment::doc_comment! {
                concat!(
"Creates a ", stringify!($name), " without checking `value`.

# Safety

`value` must not be greater than ", $max, "."
                ),
                pub const unsafe fn new_unchecked(value: $repr) -> $name {
                    $name(value)
                }
            }

            /// Returns the value as a primitive type.
            pub const fn get(self) -> $repr {
                self.0
            }
        }

        impl core::str::FromStr for $name {
            type Err = $crate::ParseIntError;

            fn from_str(source: &str) -> Result<Self, Self::Err> {
                let primitive = <$repr>::from_str(source).map_err(|_| $crate::ParseIntError(()))?;
                if !$name::is_valid(primitive) {
                    return Err($crate::ParseIntError(()));
                }
                Ok($name(primitive))
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
        impl core::convert::TryFrom<$from> for $into {
            type Error = $crate::TryFromGreaterError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                if !Self::is_valid(value.0) {
                    return Err($crate::TryFromGreaterError(()));
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
        impl core::convert::TryFrom<$from> for $into {
            type Error = $crate::TryFromGreaterError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                if !Self::is_valid(value) {
                    return Err($crate::TryFromGreaterError(()));
                }
                Ok(Self(value as _))
            }
        }
    };
}
