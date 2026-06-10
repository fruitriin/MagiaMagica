//! unsafe (赤) のサンプル。unsafe fn のコンテキストが全記号に伝播する。
unsafe fn unsafe_ptr(ptr: *const u8, fallback: u8) -> u8 {
    if ptr.is_null() {
        return fallback;
    }
    *ptr
}
