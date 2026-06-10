//! Result 戻り値 (正常/異常の分岐線) と `?` 早期リターンのサンプル。
fn result_chain(input: &str) -> Result<u32, ParseError> {
    let trimmed = sanitize(input)?;
    let value = trimmed.parse()?;
    Ok(validate(value)?)
}
