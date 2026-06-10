//! 中規模サンプル (write_document 相当)。Phase 1.8 の意匠回帰基準。
//! オーナーが「お洒落」と判定した規模感: 補助リング3つ + io 呼び出し。
fn medium_render_doc(out: &mut String, items: &[u8]) -> Result<(), Error> {
    let header = format!("count={}", items.len());
    writeln!(out, "{header}")?;
    for item in items {
        push_row(out, *item);
    }
    if out.len() > 4096 {
        truncate_tail(out);
    }
    while needs_padding(out) {
        out.push(' ');
    }
    writeln!(out, "done")?;
    Ok(())
}
