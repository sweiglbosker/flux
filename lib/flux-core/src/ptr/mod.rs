#[cfg(flux)]
use flux_attrs::*;

#[cfg(flux)]
#[extern_spec(core::ptr)]
#[spec(fn (src: *const{p: ptr_size(p) >= T::size_of() && ptr_offset(p) >= 0} T) ->  T)]
unsafe fn read<T>(src: *const T) -> T;

#[cfg(flux)]
#[extern_spec(core::ptr)]
#[spec(fn (dst: *mut{p: ptr_size(p) >= T::size_of() && ptr_offset(p) >= 0} T, src: T))]
unsafe fn write<T>(dst: *mut T, src: T);

macro_rules! ptr_specs {
    ($mutable:tt) => {
        #[extern_spec(core::ptr)]
        impl<T> *$mutable T {
            #[spec(fn (me: *$mutable[@src] T, count: usize) -> *$mutable{p:
                        ptr_size(p) == ptr_size(src) - count * T::size_of() &&
                        ptr_offset(p) == ptr_offset(src) + count * T::size_of()}
                    T)]
            unsafe fn add(self, count: usize) -> Self;

            #[spec(fn (me: *$mutable[@src] T, count: usize) -> *$mutable{p:
                        ptr_size(p) == ptr_size(src) + count * T::size_of() &&
                        ptr_offset(p) == ptr_offset(src) - count * T::size_of()
                    } T)]
            unsafe fn sub(self, count: usize) -> Self;
        }
    };
}

#[cfg(flux)]
ptr_specs!(const);

#[cfg(flux)]
ptr_specs!(mut);
