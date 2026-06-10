//! ループ (補助リング + 内側矢印) のサンプル。
fn loop_accumulate(items: &[u8]) -> u32 {
    let mut total = 0;
    for item in items {
        if *item > 128 {
            total += weigh(*item);
        }
    }
    while total > 1000 {
        total /= 2;
    }
    total
}
