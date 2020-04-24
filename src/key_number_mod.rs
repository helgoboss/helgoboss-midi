// Basic newtype definition
newtype! {
    #[doc = r"A key number (0 - 127), e.g. of a MIDI Note On message."]
    name = KeyNumber, repr = u8, max = 127
}

// From related newtype to this newtype and back
impl_from_newtype_to_newtype!(KeyNumber, crate::U7);
impl_from_newtype_to_newtype!(crate::U7, KeyNumber);

// From lower primitives to this newtype
// -

// From this newtype to higher primitives
impl_from_newtype_to_primitive!(KeyNumber, u8);
impl_from_newtype_to_primitive!(KeyNumber, i8);
impl_from_newtype_to_primitive!(KeyNumber, u16);
impl_from_newtype_to_primitive!(KeyNumber, i16);
impl_from_newtype_to_primitive!(KeyNumber, u32);
impl_from_newtype_to_primitive!(KeyNumber, i32);
impl_from_newtype_to_primitive!(KeyNumber, u64);
impl_from_newtype_to_primitive!(KeyNumber, i64);
impl_from_newtype_to_primitive!(KeyNumber, u128);
impl_from_newtype_to_primitive!(KeyNumber, i128);
impl_from_newtype_to_primitive!(KeyNumber, usize);
impl_from_newtype_to_primitive!(KeyNumber, isize);

// TryFrom higher primitives to this newtype
impl_try_from_primitive_to_newtype!(u8, KeyNumber);
impl_try_from_primitive_to_newtype!(u16, KeyNumber);
impl_try_from_primitive_to_newtype!(i16, KeyNumber);
impl_try_from_primitive_to_newtype!(u32, KeyNumber);
impl_try_from_primitive_to_newtype!(i32, KeyNumber);
impl_try_from_primitive_to_newtype!(u64, KeyNumber);
impl_try_from_primitive_to_newtype!(i64, KeyNumber);
impl_try_from_primitive_to_newtype!(u128, KeyNumber);
impl_try_from_primitive_to_newtype!(i128, KeyNumber);
impl_try_from_primitive_to_newtype!(usize, KeyNumber);
impl_try_from_primitive_to_newtype!(isize, KeyNumber);
