fn for_loop(items: &[u8]) -> u32 {
    let mut total = 0;
    for item in items {
        total += accumulate(*item);
    }
    total
}
