#[repr(align(8))]
pub struct Aligned64<T>(pub T);

// TODO: sse2 memcpy
#[repr(align(16))]
pub struct Aligned128<T>(pub T);

// TODO: avx2 memcpy
#[repr(align(32))]
pub struct Aligned256<T>(pub T);
