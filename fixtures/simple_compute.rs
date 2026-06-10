//! 純粋計算のみの最小サンプル。記号は黒一色になる。
fn simple_compute(a: i32, b: i32) -> i32 {
    let sum = a + b;
    let doubled = sum * 2;
    doubled - 1
}
