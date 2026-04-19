use flux_attrs::*;

extern crate flux_core;

defs! {
    fn valid_offset(p: ptr) -> bool { ptr_offset(p) >= 0 }
}

#[flux::spec(fn (ptr: *const {v: ptr_size(v) >= 4 && valid_offset(v)} i32) -> i32)]
fn read(ptr: *const i32) -> i32 {
    unsafe { std::ptr::read(ptr) }
}

#[flux::spec(fn (ptr: *const{v: ptr_size(v) == 10 && valid_offset(v)} i32) -> i32)]
fn read_ix(ptr: *const i32) -> i32 {
    unsafe { std::ptr::read(ptr) }
}

#[flux::spec(fn (ptr: *mut{v: ptr_size(v) >= 4 && valid_offset(v)} i32, value: i32))]
fn write_i32(ptr: *mut i32, value: i32) {
    unsafe { std::ptr::write(ptr, value); }
}

#[flux::spec(fn (ptr: *mut{v: ptr_size(v) >= T::size_of() && valid_offset(v)} T, value: T))]
fn write<T>(ptr: *mut T, value: T) {
    unsafe { std::ptr::write(ptr, value); }
}

#[flux::spec(fn (ptr: *const{v: ptr_size(v) >= 4 && ptr_offset(v) == 21} i32) -> i32)]
fn read_offset_i32(ptr: *const i32) -> i32 {
    unsafe { std::ptr::read(ptr) }
}

#[flux::spec(fn (ptr: *mut{v: ptr_size(v) >= 4 && ptr_offset(v) == 21} i32, value: i32))]
fn write_offset_i32(ptr: *mut i32, value: i32) {
    unsafe { std::ptr::write(ptr, value) }
}
