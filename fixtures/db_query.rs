//! DB 効果 (緑) のサンプル。
fn db_query(id: u64) -> Option<String> {
    let connection = rusqlite::Connection::open("app.db");
    lookup_name(connection, id)
}
