unsafe fn unsafe_block(ptr: *const u8, fallback: u8) -> u8 {
    if ptr.is_null() {
        return fallback;
    }
    *ptr
}
