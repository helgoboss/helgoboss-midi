// Basic newtype definition
newtype!(name = U14, repr = u16, max = 16383);

// From lower newtypes to this newtype
impl_from_newtype_to_newtype!(crate::U4, U14);
impl_from_newtype_to_newtype!(crate::U7, U14);

// From lower primitives to this newtype
impl_from_primitive_to_newtype!(u8, U14);
impl_from_primitive_to_newtype!(i8, U14);

// From this newtype to higher primitives
impl_from_newtype_to_primitive!(U14, u16);
impl_from_newtype_to_primitive!(U14, i16);
impl_from_newtype_to_primitive!(U14, u32);
impl_from_newtype_to_primitive!(U14, i32);
impl_from_newtype_to_primitive!(U14, u64);
impl_from_newtype_to_primitive!(U14, i64);
impl_from_newtype_to_primitive!(U14, u128);
impl_from_newtype_to_primitive!(U14, i128);
impl_from_newtype_to_primitive!(U14, usize);
impl_from_newtype_to_primitive!(U14, isize);

// TryFrom higher newtypes to this newtype
// -

// TryFrom higher primitives to this newtype
impl_try_from_primitive_to_newtype!(u16, U14);
impl_try_from_primitive_to_newtype!(u32, U14);
impl_try_from_primitive_to_newtype!(i32, U14);
impl_try_from_primitive_to_newtype!(u64, U14);
impl_try_from_primitive_to_newtype!(i64, U14);
impl_try_from_primitive_to_newtype!(u128, U14);
impl_try_from_primitive_to_newtype!(i128, U14);
impl_try_from_primitive_to_newtype!(usize, U14);
