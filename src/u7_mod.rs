// Basic newtype definition
newtype! {
    #[doc = r"A 7-bit integer (0 - 127)."]
    name = U7, repr = u8, max = 127
}

// From lower newtypes to this newtype
impl_from_newtype_to_newtype!(crate::U4, U7);

// From lower primitives to this newtype
// -

// From this newtype to higher primitives
impl_from_newtype_to_primitive!(U7, u8);
impl_from_newtype_to_primitive!(U7, i8);
impl_from_newtype_to_primitive!(U7, u16);
impl_from_newtype_to_primitive!(U7, i16);
impl_from_newtype_to_primitive!(U7, u32);
impl_from_newtype_to_primitive!(U7, i32);
impl_from_newtype_to_primitive!(U7, u64);
impl_from_newtype_to_primitive!(U7, i64);
impl_from_newtype_to_primitive!(U7, u128);
impl_from_newtype_to_primitive!(U7, i128);
impl_from_newtype_to_primitive!(U7, usize);
impl_from_newtype_to_primitive!(U7, isize);

// TryFrom higher newtypes to this newtype
impl_try_from_newtype_to_newtype!(crate::U14, U7);

// TryFrom higher primitives to this newtype
impl_try_from_primitive_to_newtype!(u8, U7);
impl_try_from_primitive_to_newtype!(u16, U7);
impl_try_from_primitive_to_newtype!(i16, U7);
impl_try_from_primitive_to_newtype!(u32, U7);
impl_try_from_primitive_to_newtype!(i32, U7);
impl_try_from_primitive_to_newtype!(u64, U7);
impl_try_from_primitive_to_newtype!(i64, U7);
impl_try_from_primitive_to_newtype!(u128, U7);
impl_try_from_primitive_to_newtype!(i128, U7);
impl_try_from_primitive_to_newtype!(usize, U7);
impl_try_from_primitive_to_newtype!(isize, U7);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_successful() {
        U7::new(127);
    }

    #[test]
    #[should_panic(expected = "128 is not a valid U7 value")]
    fn new_failing() {
        U7::new(128);
    }
}
