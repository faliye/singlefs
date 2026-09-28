// A7 候选一（弱）：复用键是只能从写表算出来的新类型，标签不实现 Hash / Eq / Ord，但给了 Display（覆盖报告要打它）。
pub struct WriteTable(pub Vec<(u32, u64, Vec<u8>)>);
pub struct PathWriteTableHash([u8; 32]);
impl PathWriteTableHash {
    pub fn of_write_table(table: &WriteTable) -> Self {
        let mut digest = [0u8; 32];
        for (index, (device, offset, bytes)) in table.0.iter().enumerate() {
            digest[index % 32] ^= (*device as u8) ^ (*offset as u8) ^ bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        }
        Self(digest)
    }
}
pub struct NodeLabel(String);
impl NodeLabel {
    pub fn of_kinds_and_overlaps(kinds: &str, overlaps: &str) -> Self {
        Self(format!("{kinds}/{overlaps}"))
    }
}
impl std::fmt::Display for NodeLabel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
pub struct Verdict(pub u8);
pub struct VerdictStore(Vec<([u8; 32], u8)>);
impl VerdictStore {
    pub fn reuse(&self, key: &PathWriteTableHash) -> Option<Verdict> {
        self.0.iter().find(|(stored, _)| *stored == key.0).map(|(_, verdict)| Verdict(*verdict))
    }
}
