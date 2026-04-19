#[flux::spec(fn(p: *const{v: ptr_size(v) == 4} i32) -> *const{v: ptr_size(v) == 8} i32)]
fn bad_const_ptr_size(p: *const i32) -> *const i32 {
    p //~ ERROR refinement type error
}

#[flux::spec(fn(p: *mut{v: ptr_offset(v) == 0} i32) -> *mut{v: ptr_offset(v) == 1} i32)]
fn bad_mut_ptr_offset(p: *mut i32) -> *mut i32 {
    p //~ ERROR refinement type error
}
