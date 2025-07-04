use std::mem::size_of;
use std::slice::{
    from_raw_parts,
    from_raw_parts_mut,
};

// Taken from https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
//
// This function is marked safe because it is read-only, e.g., we are just reading the
// bytes of the input; the input to the function will still be required to follow all
// the reference owner rules (e.g., no mutable and non-mutable reference at the same time).
pub(crate) fn as_bytes<T: Sized>(p: &T) -> &[u8] {
    unsafe { from_raw_parts((p as *const T) as *const u8, size_of::<T>()) }
}

// For the same reason as above, I think this function is actually safe, but don't quote me on that
pub(crate) fn as_bytes_mut<T: Sized>(p: &mut T) -> &mut [u8] {
    unsafe { from_raw_parts_mut((p as *mut T) as *mut u8, size_of::<T>()) }
}

pub(crate) fn copy_bytes<T: Sized>(dst: &mut [u8], src: &T) {
    let src_bytes = as_bytes(src);
    let len = src_bytes.len().min(dst.len());
    dst[..len].copy_from_slice(&src_bytes[..len]);
}
