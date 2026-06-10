//! 過密サンプル (Phase 1.8 の衝突回避検証用)。match 6アーム + 入れ子 + 呼び出し多数。
fn dense_dispatch(code: u8, payload: &[u8]) -> u32 {
    let normalized = normalize(code);
    match normalized {
        0 => reset_state(),
        1 => {
            if payload.is_empty() {
                return fallback_one();
            }
            consume(payload)
        }
        2 => {
            for byte in payload {
                feed(*byte);
            }
            flush_all()
        }
        3 => {
            while pending() {
                drain_once();
            }
            finish()
        }
        4 => std::fs::metadata("state.bin").map_or(0, |_| reload()),
        _ => {
            log_unknown(normalized);
            audit(normalized);
            default_code()
        }
    }
}
