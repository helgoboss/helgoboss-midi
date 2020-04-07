// Basic newtype definition
newtype!(U7, u8, 127, u7);

// Conversions to primitives
impl_from_newtype_to_primitive!(U7, u8);
impl_from_newtype_to_primitive!(U7, u16);
impl_from_newtype_to_primitive!(U7, i32);
impl_from_newtype_to_primitive!(U7, usize);

// Conversions from primitives
impl_try_from_primitive_to_newtype!(i32, U7);
