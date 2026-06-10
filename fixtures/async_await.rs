//! async fn (メインリング二重線) と await のサンプル。
async fn async_await(url: &str) -> String {
    let first = fetch(url).await;
    let second = fetch_more(first).await;
    second
}
