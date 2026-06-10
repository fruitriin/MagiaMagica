//! match (アームごとの補助リング) のサンプル。
fn match_dispatch(command: u8) -> &'static str {
    match command {
        0 => "halt",
        1 => spin_up(),
        2 => { prepare(); "ready" }
        _ => fallback(),
    }
}
