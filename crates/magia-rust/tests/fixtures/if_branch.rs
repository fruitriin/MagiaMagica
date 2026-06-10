fn if_branch(flag: bool) -> i32 {
    let base = seed();
    if flag {
        grow(base)
    } else {
        shrink(base)
    }
}
