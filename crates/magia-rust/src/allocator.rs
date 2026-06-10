//! 決定論的な `SigilId` 採番器。
//!
//! IR 定義の `SigilId(pub u32)` のフィールド直接構築では採番ルールを保てないため、
//! `parse_function` 内では本アロケータを通じてのみ ID を発行する。これは
//! CLAUDE.repo.md の「情報隠蔽 (POSD)」原則と整合する: ID の単調増加と一意性は
//! ここで一元的に守り、呼び出し側は ID の出所を意識しない。

use magia_core::ir::SigilId;

/// 決定論的に `SigilId` を採番するアロケータ。
#[derive(Debug, Default, Clone)]
pub(crate) struct SigilIdAllocator {
    next: u32,
}

impl SigilIdAllocator {
    /// 新しいアロケータを作成する (採番は 0 から)。
    pub(crate) const fn new() -> Self {
        Self { next: 0 }
    }

    /// 次の `SigilId` を発行し、内部カウンタを進める。
    pub(crate) fn allocate(&mut self) -> SigilId {
        let id = SigilId(self.next);
        self.next += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_yields_monotonic_ids() {
        let mut a = SigilIdAllocator::new();
        assert_eq!(a.allocate(), SigilId(0));
        assert_eq!(a.allocate(), SigilId(1));
        assert_eq!(a.allocate(), SigilId(2));
    }
}
