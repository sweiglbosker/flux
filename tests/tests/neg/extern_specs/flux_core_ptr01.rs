extern crate flux_core;

#[flux::spec(fn (ptr: *const{v: ptr_size(v) >= 8 && ptr_offset(v) >= 0} i32))]
pub fn test_add_read_out_of_bounds(ptr: *const i32) {
    unsafe {
        let _val0 = std::ptr::read(ptr.add(0));
        let _val1 = std::ptr::read(ptr.add(1));
        let _val2 = std::ptr::read(ptr.add(2)); //~ ERROR: refinement type error
    }
}

#[flux::spec(fn (ptr: *mut{v: ptr_size(v) >= 8 && ptr_offset(v) >= 0} i32))]
pub fn test_add_write_out_of_bounds_mut(ptr: *mut i32) {
    unsafe {
        std::ptr::write(ptr.add(0), 10);
        std::ptr::write(ptr.add(1), 20);
        std::ptr::write(ptr.add(2), 30); //~ ERROR: refinement type error
    }
}

#[flux::spec(
    fn (ptr: *const{v: ptr_size(v) == 20 && ptr_offset(v) == 10} i32) ->
        *const{v: ptr_offset(v) == 12} i32)]
pub fn test_add_two_offset_wrong(ptr: *const i32) -> *const i32 {
    unsafe { ptr.add(2) } //~ ERROR: refinement type error
}

#[flux::spec(
    fn (ptr: *mut{v: ptr_size(v) == 20 && ptr_offset(v) == 10} i32) ->
        *mut{v: ptr_size(v) == 12 && ptr_offset(v) == 18} i32)]
pub fn test_sub_two_size_and_offset_wrong_mut(ptr: *mut i32) -> *mut i32 {
    unsafe { ptr.sub(2) } //~ ERROR: refinement type error
                          //~| ERROR: refinement type error
}
