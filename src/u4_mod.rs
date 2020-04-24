// Basic newtype definition
newtype!(name = U4, repr = u8, max = 15);

// From lower newtypes to this newtype
// -

// From lower primitives to this newtype
// -

// From this newtype to higher primitives
impl_from_newtype_to_primitive!(U4, u8);
impl_from_newtype_to_primitive!(U4, i8);
impl_from_newtype_to_primitive!(U4, u16);
impl_from_newtype_to_primitive!(U4, i16);
impl_from_newtype_to_primitive!(U4, u32);
impl_from_newtype_to_primitive!(U4, i32);
impl_from_newtype_to_primitive!(U4, u64);
impl_from_newtype_to_primitive!(U4, i64);
impl_from_newtype_to_primitive!(U4, u128);
impl_from_newtype_to_primitive!(U4, i128);
impl_from_newtype_to_primitive!(U4, usize);
impl_from_newtype_to_primitive!(U4, isize);

// TryFrom higher newtypes to this newtype
impl_try_from_newtype_to_newtype!(crate::U14, U4);
impl_try_from_newtype_to_newtype!(crate::U7, U4);

// TryFrom higher primitives to this newtype
impl_try_from_primitive_to_newtype!(u8, U4);
impl_try_from_primitive_to_newtype!(u16, U4);
impl_try_from_primitive_to_newtype!(i16, U4);
impl_try_from_primitive_to_newtype!(u32, U4);
impl_try_from_primitive_to_newtype!(i32, U4);
impl_try_from_primitive_to_newtype!(u64, U4);
impl_try_from_primitive_to_newtype!(i64, U4);
impl_try_from_primitive_to_newtype!(u128, U4);
impl_try_from_primitive_to_newtype!(i128, U4);
impl_try_from_primitive_to_newtype!(usize, U4);
impl_try_from_primitive_to_newtype!(isize, U4);
