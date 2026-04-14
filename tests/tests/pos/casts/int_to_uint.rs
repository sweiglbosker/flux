use flux_attrs::*;

#[spec(fn() -> u64[0x7FFFFFFFFFFFFFFF])]
pub fn i64_max() -> u64 {
    i64::MAX as u64
}

#[spec(fn() -> usize[1])]
pub fn i16_u16_lossless() -> usize {
    1isize as usize
}

#[spec(fn() -> u32[42])]
pub fn i64_u32_lossless() -> u32 {
    42i64 as u32
}

#[spec(fn({i32[@n] | 0 <= n && n < 12}) -> u8{v: v < 12})]
pub fn i32_u8_lossless_reft(x: i32) -> u8 {
    x as u8 
}

#[spec(fn(x:i32{0 <= x}) -> u32[x])]
pub fn i32_u32_nonneg(x: i32) -> u32 {
    x as u32
}

#[spec(fn(x:i64{0 <= x && x <= 4_294_967_295}) -> u32[x])]
pub fn i64_u32_bounded(x: i64) -> u32 {
    x as u32
}

#[spec(fn(x:i64{0 <= x && x <= 100}) -> usize{v: v <= 100})]
pub fn i64_usize_small(x: i64) -> usize {
    x as usize
}
