pub type Nibble = u8;
pub type SevenBitValue = u8;
pub type Byte = u8;
pub type FourteenBitValue = u16;

pub const NIBBLE_MAX: Nibble = 15;
pub const NIBBLE_COUNT: u32 = NIBBLE_MAX as u32 + 1;
pub const SEVEN_BIT_VALUE_MAX: SevenBitValue = 127;
pub const SEVEN_BIT_VALUE_COUNT: u32 = SEVEN_BIT_VALUE_MAX as u32 + 1;
pub const BYTE_MAX: Byte = std::u8::MAX;
pub const BYTE_COUNT: u32 = BYTE_MAX as u32 + 1;
pub const FOURTEEN_BIT_VALUE_MAX: FourteenBitValue = 16383;
pub const FOURTEEN_BIT_VALUE_COUNT: u32 = FOURTEEN_BIT_VALUE_MAX as u32 + 1;
