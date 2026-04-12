#![flux::defs(
    fn pow2(x: int) -> bool { pow2bv(bv_int_to_bv64(x)) }
    fn pow2bv(x: bitvec<64>) -> bool {
        bv_and(x, bv_sub(x, bv_int_to_bv64(1))) == bv_int_to_bv64(0)
    }
    fn is_valid_layout(s: int, a: int) -> bool {
        (s + a - 1 <= isize::MAX) && a > 0 && pow2(a)
    }
)]

use flux_attrs::*;

#[extern_spec(core::alloc)]
#[opaque]
#[refined_by(size: int, align: int)]
#[invariant(align > 0 && pow2(align))]
#[invariant(size + align - 1 <= isize::MAX)]
struct Layout;

#[extern_spec(core::alloc)]
impl Layout {
    // Core impl: https://github.com/rust-lang/rust/blob/dab8d9d1066c4c95008163c7babf275106ce3f32/library/core/src/alloc/layout.rs#L206-L208
    #[no_panic]
    #[spec(fn() -> Self[T::size_of(), T::align_of()])]
    const fn new<T>() -> Self;

    // Core impl: https://github.com/rust-lang/rust/blob/dab8d9d1066c4c95008163c7babf275106ce3f32/library/core/src/alloc/layout.rs#L59-L66
    #[no_panic]
    #[spec(fn(s: usize, a: usize) -> Result<Self[s,a], _>[is_valid_layout(s, a)])]
    const fn from_size_align(size: usize, align: usize) -> Result<Layout, LayoutError>;

    // Core impl: https://github.com/rust-lang/rust/blob/dab8d9d1066c4c95008163c7babf275106ce3f32/library/core/src/alloc/layout.rs#L132-L144
    #[no_panic]
    #[spec(fn(s: usize, a: usize) -> Self[s, a] requires is_valid_layout(s, a))]
    const unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Self;

    // Core impl: https://github.com/rust-lang/rust/blob/dab8d9d1066c4c95008163c7babf275106ce3f32/library/core/src/alloc/layout.rs#L175-L177
    #[no_panic]
    #[spec(fn(&Self[@s, @a]) -> usize[s])]
    const fn size(&self) -> usize;

    // Core impl: https://github.com/rust-lang/rust/blob/dab8d9d1066c4c95008163c7babf275106ce3f32/library/core/src/alloc/layout.rs#L187-L189
    #[no_panic]
    #[spec(fn(&Self[@s, @a]) -> usize[a])]
    const fn align(&self) -> usize;

    // Core impl: https://github.com/rust-lang/rust/blob/dab8d9d1066c4c95008163c7babf275106ce3f32/library/core/src/alloc/layout.rs#L559-L589
    #[no_panic]
    #[spec(fn(n: usize) -> Result<Self[n * T::size_of(), T::align_of()],_>[
        T::size_of() == 0 || n * T::size_of() + T::align_of() - 1 <= isize::MAX
    ])]
    const fn array<T>(n: usize) -> Result<Layout, LayoutError>;
}
