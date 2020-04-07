// Basic newtype definition
newtype!(Channel, u8, 15, channel);

// Conversions between newtypes
impl_from_newtype_to_newtype!(Channel, crate::U4);
impl_from_newtype_to_newtype!(crate::U4, Channel);

// Conversions to primitives
impl_from_newtype_to_primitive!(Channel, u8);
impl_from_newtype_to_primitive!(Channel, i32);
impl_from_newtype_to_primitive!(Channel, usize);

// Conversions from primitives
impl_try_from_primitive_to_newtype!(i32, Channel);
