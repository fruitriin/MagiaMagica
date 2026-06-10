fn match_arms(code: u8) -> &'static str {
    match code {
        0 => "zero",
        1 => label_one(),
        _ => fallback(),
    }
}
