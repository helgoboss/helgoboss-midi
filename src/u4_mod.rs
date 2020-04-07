// Basic newtype definition
newtype!(U4, u8, 15, u4);

// Conversions to primitives
impl_from_newtype_to_primitive!(U4, u8);
impl_from_newtype_to_primitive!(U4, u16);
impl_from_newtype_to_primitive!(U4, i32);
impl_from_newtype_to_primitive!(U4, usize);

// Conversions from primitives
impl_try_from_primitive_to_newtype!(i32, U4);
