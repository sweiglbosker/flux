use flux_attrs::*;

defs! {
    fn valid_offset(p: ptr) -> bool { ptr_offset(p) >= 0 }
}

#[flux::spec(
    fn(p: *const{v: ptr_size(v) == 4 && valid_offset(v)} i32) ->
        *const{v: ptr_size(v) == 4 && valid_offset(v)} i32)]
fn const_ptr(p: *const i32) -> *const i32 {
    p
}

#[flux::spec(
    fn(p: *mut{v: ptr_size(v) == 8 && ptr_offset(v) == 0} i32) ->
        *mut{v: ptr_size(v) == 8 && ptr_offset(v) == 0} i32)]
fn mut_ptr(p: *mut i32) -> *mut i32 {
    p
}

#[flux::spec(
    fn(p: *const{v: ptr_size(v) >= T::size_of() && valid_offset(v)} T) ->
        *const{v: ptr_size(v) >= T::size_of() && valid_offset(v)} T)]
fn generic_const_ptr<T>(p: *const T) -> *const T {
    p
}
