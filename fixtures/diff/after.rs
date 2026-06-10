//! Spell Diff 用の変更後リビジョン (Phase 3.1)。
//! before.rs に対して:
//! - if 分岐に audit 召喚を追加 (変更 + 追加)
//! - else 分岐を新設 (追加)
//! - notify 召喚を削除 (削除)
//! - for ループ本体は不変
fn process_order(count: u32) -> u32 {
    let base = prepare(count);
    if base > 10 {
        log_large(base);
        audit(base);
    } else {
        log_small(base);
    }
    let mut total = 0;
    for i in 0..base {
        total += step(i);
    }
    total
}
