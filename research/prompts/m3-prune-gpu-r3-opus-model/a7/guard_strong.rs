// A7 候选二（收严）：同候选一，另外标签什么格式化都不实现（没有 Display、Debug、Clone），只能交给只写的覆盖报告；
// 标签的原料（种类串、覆盖签名）在这个模块里算、不外露；调用方所在的 crate 带 #![forbid(unsafe_code)]。
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
    pub fn of_write_table(table: &WriteTable) -> Self {
        Self(table.0.iter().map(|(device, _, _)| char::from(b'a' + (*device as u8 % 26))).collect())
    }
}
/// 只写的覆盖报告：收标签、写进文件，不把文字交回给调用方。
pub struct CoverageReport(Vec<String>);
impl CoverageReport {
    pub fn name(&mut self, label: &NodeLabel) {
        self.0.push(label.0.clone());
    }
}
pub struct Verdict(pub u8);
pub struct VerdictStore(Vec<([u8; 32], u8)>);
impl VerdictStore {
    pub fn reuse(&self, key: &PathWriteTableHash) -> Option<Verdict> {
        self.0.iter().find(|(stored, _)| *stored == key.0).map(|(_, verdict)| Verdict(*verdict))
    }
}
