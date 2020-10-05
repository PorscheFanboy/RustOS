/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if (align & (align - 1)) != 0 {
        panic!();
    }
    return (addr / align) * align;
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    if (align & (align - 1)) != 0 {
        panic!();
    }
    if addr % align == 0 {
        return addr;
    }
    return (addr / align + 1) * align;
}
