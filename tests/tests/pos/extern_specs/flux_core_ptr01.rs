extern crate flux_core;

#[flux::spec(fn (ptr: *const{v: ptr_size(v) >= 8 && ptr_offset(v) >= 0} i32))]
pub fn test_add_read(ptr: *const i32) {
    unsafe {
        let _val0 = std::ptr::read(ptr.add(0));
        let _val1 = std::ptr::read(ptr.add(1));
    }
}

#[flux::spec(fn (ptr: *mut{v: ptr_size(v) >= 8 && ptr_offset(v) >= 0} i32))]
pub fn test_add_write_mut(ptr: *mut i32) {
    unsafe {
        std::ptr::write(ptr.add(0), 10);
        std::ptr::write(ptr.add(1), 20);
    }
}

#[flux::spec(
    fn (ptr: *const{v: ptr_size(v) == 20 && ptr_offset(v) == 10} i32) ->
        *const{v: ptr_size(v) == 12 && ptr_offset(v) == 18} i32)]
pub fn test_add_two_size_and_offset(ptr: *const i32) -> *const i32 {
    unsafe { ptr.add(2) }
}

#[flux::spec(
    fn (ptr: *mut{v: ptr_size(v) == 20 && ptr_offset(v) == 10} i32) ->
        *mut{v: ptr_size(v) == 28 && ptr_offset(v) == 2} i32)]
pub fn test_sub_two_size_and_offset_mut(ptr: *mut i32) -> *mut i32 {
    unsafe { ptr.sub(2) }
}

#[flux::spec(
    fn (ptr: *const{v: ptr_size(v) == 20 && ptr_offset(v) == 10} i32) ->
        *const{v: ptr_size(v) == 20 && ptr_offset(v) == 10} i32)]
pub fn test_add_zero_identity(ptr: *const i32) -> *const i32 {
    unsafe { ptr.add(0) }
}
