// Basic newtype definition
newtype!(U14, u16, 16383, u14);

// Conversions between newtypes
impl_from_newtype_to_newtype!(crate::U7, U14);

// Conversions to primitives
impl_from_newtype_to_primitive!(U14, u8);
impl_from_newtype_to_primitive!(U14, u16);
impl_from_newtype_to_primitive!(U14, i32);
impl_from_newtype_to_primitive!(U14, usize);

// Conversions from primitives
impl_from_primitive_to_newtype!(u8, U14);
impl_try_from_primitive_to_newtype!(i32, U14);
