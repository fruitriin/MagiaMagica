//! CI 実地確認用プローブ (Phase 3.3)。
//! spell-diff ワークフローの動作確認 PR でこのファイルを変更する。
//! テストからは参照しない (cli_integration の FIXTURES に含めない)。
fn ci_probe(x: u32) -> u32 {
    if x > 10 {
        flare(x);
    }
    shimmer(x)
}
