//! ネットワーク効果 (紫) のサンプル。
fn network_fetch(url: &str) -> Vec<u8> {
    let response = reqwest::blocking::get(url);
    extract_body(response)
}
