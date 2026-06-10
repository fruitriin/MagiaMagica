//! Spell Diff 用の変更前リビジョン (Phase 3.1)。
//! after.rs との差分が4象限 (追加/削除/変更/不変) を全て踏むよう設計している:
//! - notify 召喚は after で消える (削除)
//! - if 分岐は after で操作が増える (変更)
//! - for ループ本体と step 召喚は不変
fn process_order(count: u32) -> u32 {
    let base = prepare(count);
    if base > 10 {
        log_large(base);
    }
    let mut total = 0;
    for i in 0..base {
        total += step(i);
    }
    notify(total);
    total
}
