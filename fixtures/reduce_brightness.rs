//! Reducer 形 (`(A, B) -> A`) のサンプル。ベルカ式の推奨ヒント (spec v0.3 §14.3) が出る。
fn reduce_brightness(acc: u32, sample: u8) -> u32 {
    let gain = amplify(sample);
    acc + gain
}
