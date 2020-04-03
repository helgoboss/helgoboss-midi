use crate::{Channel, SevenBitValue, U14};

pub(crate) fn extract_high_7_bit_value_from_14_bit_value(value: U14) -> SevenBitValue {
    ((u16::from(value) >> 7) & 0x7f) as u8
}

pub(crate) fn extract_low_7_bit_value_from_14_bit_value(value: U14) -> SevenBitValue {
    (u16::from(value) & 0x7f) as u8
}

pub(crate) fn build_14_bit_value_from_two_7_bit_values(
    high: SevenBitValue,
    low: SevenBitValue,
) -> U14 {
    debug_assert!(high <= 0x7f);
    debug_assert!(low <= 0x7f);
    unsafe { U14::new_unchecked(((high as u16) << 7) | (low as u16)) }
}
