use crate::{U14, U7};

pub(crate) fn extract_high_7_bit_value_from_14_bit_value(value: U14) -> U7 {
    U7(((u16::from(value) >> 7) & 0x7f) as u8)
}

pub(crate) fn extract_low_7_bit_value_from_14_bit_value(value: U14) -> U7 {
    U7((u16::from(value) & 0x7f) as u8)
}

pub(crate) fn build_14_bit_value_from_two_7_bit_values(high: U7, low: U7) -> U14 {
    U14((u16::from(high) << 7) | u16::from(low))
}
