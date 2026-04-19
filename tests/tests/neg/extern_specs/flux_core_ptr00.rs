use flux_attrs::*;

extern crate flux_core;

pub fn test_read(x: *const i32) -> i32 {
    unsafe { std::ptr::read(x) } //~ ERROR refinement type error
                                 //~| ERROR refinement type error
}

fn test_write<T>(ptr: *mut T, value: T) {
    unsafe {
        std::ptr::write(ptr, value); //~ ERROR refinement type error
                                     //~| ERROR refinement type error
    }
}

fn test_write_i32(ptr: *mut i32, value: i32) {
    unsafe {
        std::ptr::write(ptr, value); //~ ERROR refinement type error
                                     //~| ERROR refinement type error
    }
}

#[flux::spec(fn(ptr: *const{p: ptr_size(p) >= 4 && ptr_offset(p) < 0} i32))]
fn test_read_bad_offset(ptr: *const i32) {
    unsafe {
        std::ptr::read(ptr); //~ ERROR refinement type error
    }
}
