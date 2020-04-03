pub type SevenBitValue = u8;
pub type FourteenBitValue = u16;

pub const SEVEN_BIT_VALUE_MAX: SevenBitValue = 127;
pub const SEVEN_BIT_VALUE_COUNT: u32 = SEVEN_BIT_VALUE_MAX as u32 + 1;
pub const FOURTEEN_BIT_VALUE_MAX: FourteenBitValue = 16383;
pub const FOURTEEN_BIT_VALUE_COUNT: u32 = FOURTEEN_BIT_VALUE_MAX as u32 + 1;
