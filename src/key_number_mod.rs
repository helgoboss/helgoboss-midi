// Basic newtype definition
newtype!(KeyNumber, u8, 127, key_number);

// Conversions between newtypes
impl_from_newtype_to_newtype!(KeyNumber, crate::U7);
impl_from_newtype_to_newtype!(crate::U7, KeyNumber);

// Conversions to primitives
impl_from_newtype_to_primitive!(KeyNumber, u8);
impl_from_newtype_to_primitive!(KeyNumber, i32);
impl_from_newtype_to_primitive!(KeyNumber, usize);

// Conversions from primitives
impl_try_from_primitive_to_newtype!(i32, KeyNumber);
