use flux_attrs::*;

#[spec(fn(n:i64{n: n < 13}) -> u64{v: v < 13})]
pub fn i64_u64_lossy_neg(n: i64) -> u64 {
    n as u64 //~ ERROR refinement type
}

#[spec(fn({i16[@n] | n >= 0}) -> u8[n])]
pub fn i16_u8_lossy_mag(n: i16) -> u8 {
    n as u8 //~ ERROR refinement type
}

#[spec(fn() -> i8[-1])]
pub fn i64_i8_lossy_const() -> i8 {
    -1i8 as u8 as i8 //~ ERROR refinement type
}

#[spec(fn(x:i32) -> u32[x])]
pub fn i32_u32_unconstrained(x: i32) -> u32 {
    x as u32 //~ ERROR refinement type
}

#[spec(fn(x:i64{0 <= x}) -> u32[x])]
pub fn i64_u32_missing_upper(x: i64) -> u32 {
    x as u32 //~ ERROR refinement type
}

#[spec(fn(x:i64{x <= 5}) -> usize[x])]
pub fn i64_usize_negative_possible(x: i64) -> usize {
    x as usize //~ ERROR refinement type
}
