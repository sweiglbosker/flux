extern crate flux_core;

pub fn test(buf: &mut [u32; 2]) {
    // let mut buf: [u32; 2] = [67; 2]; // sigh, see https://github.com/flux-rs/flux/issues/1465

    let ptr: *mut u32 = buf.as_mut_ptr();

    unsafe {
        std::ptr::write(ptr.add(0), 10);
        std::ptr::write(ptr.add(1), 20);
    }
}

#[flux::spec(fn(buf: &mut [u32; 2]) -> *mut{p: ptr_size(p) == 8 && ptr_offset(p) == 0} u32)]
pub fn test_as_mut_ptr_offset(buf: &mut [u32; 2]) -> *mut u32 {
    buf.as_mut_ptr()
}
