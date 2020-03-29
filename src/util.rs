use crate::{Byte, FourteenBitValue, Nibble, SevenBitValue};

pub(crate) fn extract_high_nibble_from_byte(byte: Byte) -> Nibble {
    (byte >> 4) & 0x0f
}

pub(crate) fn extract_low_nibble_from_byte(byte: Byte) -> Nibble {
    byte & 0x0f
}

pub(crate) fn extract_high_7_bit_value_from_14_bit_value(value: FourteenBitValue) -> SevenBitValue {
    debug_assert!(value < 16384);
    ((value >> 7) & 0x7f) as u8
}

pub(crate) fn extract_low_7_bit_value_from_14_bit_value(value: FourteenBitValue) -> SevenBitValue {
    debug_assert!(value < 16384);
    (value & 0x7f) as u8
}

pub(crate) fn build_byte_from_nibbles(high_nibble: Nibble, low_nibble: Nibble) -> Byte {
    debug_assert!(high_nibble <= 0xf);
    debug_assert!(low_nibble <= 0xf);
    (high_nibble << 4) | low_nibble
}

pub(crate) fn build_14_bit_value_from_two_7_bit_values(
    high: SevenBitValue,
    low: SevenBitValue,
) -> FourteenBitValue {
    debug_assert!(high <= 0x7f);
    debug_assert!(low <= 0x7f);
    ((high as u16) << 7) | (low as u16)
}

pub(crate) fn with_low_nibble_added(byte: Byte, low_nibble: Nibble) -> Byte {
    debug_assert!(low_nibble <= 0xf);
    byte | low_nibble
}
