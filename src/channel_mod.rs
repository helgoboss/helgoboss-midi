// Basic newtype definition
newtype!(Channel, u8, 15, channel);

// From related newtype to this newtype and back
impl_from_newtype_to_newtype!(Channel, crate::U4);
impl_from_newtype_to_newtype!(crate::U4, Channel);

// From lower primitives to this newtype
// -

// From this newtype to higher primitives
impl_from_newtype_to_primitive!(Channel, u8);
impl_from_newtype_to_primitive!(Channel, i8);
impl_from_newtype_to_primitive!(Channel, u16);
impl_from_newtype_to_primitive!(Channel, i16);
impl_from_newtype_to_primitive!(Channel, u32);
impl_from_newtype_to_primitive!(Channel, i32);
impl_from_newtype_to_primitive!(Channel, u64);
impl_from_newtype_to_primitive!(Channel, i64);
impl_from_newtype_to_primitive!(Channel, u128);
impl_from_newtype_to_primitive!(Channel, i128);
impl_from_newtype_to_primitive!(Channel, usize);
impl_from_newtype_to_primitive!(Channel, isize);

// TryFrom higher primitives to this newtype
impl_try_from_primitive_to_newtype!(u8, Channel);
impl_try_from_primitive_to_newtype!(u16, Channel);
impl_try_from_primitive_to_newtype!(i16, Channel);
impl_try_from_primitive_to_newtype!(u32, Channel);
impl_try_from_primitive_to_newtype!(i32, Channel);
impl_try_from_primitive_to_newtype!(u64, Channel);
impl_try_from_primitive_to_newtype!(i64, Channel);
impl_try_from_primitive_to_newtype!(u128, Channel);
impl_try_from_primitive_to_newtype!(i128, Channel);
impl_try_from_primitive_to_newtype!(usize, Channel);
impl_try_from_primitive_to_newtype!(isize, Channel);
