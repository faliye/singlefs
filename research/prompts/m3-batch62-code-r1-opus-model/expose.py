#!/usr/bin/env python3
"""攻方腿（m3-batch62-code-r1 opus）：在仓副本的 verdict_store.rs 里加两个公开包装，让原型 crate 从库外调到
私有的 import_the_planned_rows_in_batches_of、只在 cfg(test) 下有的 every_stored_entry 与按编号读内存表（不改任何已有的行，只在
`#[cfg(test)]\n    fn every_stored_entry` 之前插一段）；另在 write_the_planned_import_batch 算完行数之后插一行设了环境变量
OPUS_TRACE_BATCHES 才打的 OPUS_BATCH（每批写几行、这一批新登记几种判定向量），不设不改行为。用法：expose.py <副本根>。旧串不恰好命中一次退出 2。"""
import sys
from pathlib import Path

STORE = "crates/singlefs-checker-tier/src/verdict_store.rs"
ANCHOR = "    #[cfg(test)]\n    fn every_stored_entry(&self) -> Vec<StoredEntry> {"
INSERT = '''    /// 攻方原型（m3-batch62-code-r1 opus，只在仓副本里）：按给定批长导入，同 import_the_planned_rows_in_batches_of。
    ///
    /// # Errors
    /// 同 [`Self::import_the_planned_rows_from`]。
    pub fn opus_import_in_batches_of(
        &mut self,
        other: &VerdictStore,
        planned: &PlannedImportKeys,
        batch_size_in_bytes: usize,
    ) -> Result<PlannedImportTally, VerdictStoreError> {
        self.import_the_planned_rows_in_batches_of(other, planned, batch_size_in_bytes)
    }

    /// 攻方原型（m3-batch62-code-r1 opus，只在仓副本里）：库里每个键值行，按键序。
    ///
    /// # Panics
    /// 读整个库失败。
    #[must_use]
    pub fn opus_every_stored_entry(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.database
            .iterator(rocksdb::IteratorMode::Start)
            .map(|entry| entry.expect("原型里读整个库不该失败"))
            .map(|(key, value)| (key.to_vec(), value.to_vec()))
            .collect()
    }

    /// 攻方原型（m3-batch62-code-r1 opus，只在仓副本里）：内存表里第 `number` 号判定向量的内容（编号的构造器是私有的）。
    #[must_use]
    pub fn opus_vector_of_number_u8(&self, number: u8) -> Option<Vec<u8>> {
        self.verdict_vector_of_number(VerdictVectorNumber(number))
            .map(|vector| vector.0.clone())
    }

'''
TRACE_ANCHOR = '        let rows = u64::try_from(pending.batch.len()).expect("一批的行数装得进 u64");\n'
TRACE_INSERT = (
    '        if std::env::var_os("OPUS_TRACE_BATCHES").is_some() {\n'
    '            eprintln!(\n'
    '                "OPUS_BATCH rows={rows} new_vectors={}",\n'
    '                pending.verdict_vectors_registered_in_the_batch.len()\n'
    '            );\n'
    '        }\n'
)


def main():
    root = Path(sys.argv[1])
    path = root / STORE
    text = path.read_text()
    if text.count(ANCHOR) != 1:
        print(f"expose：{STORE} 里锚点命中 {text.count(ANCHOR)} 次", file=sys.stderr)
        return 2
    if text.count(TRACE_ANCHOR) != 1:
        print(f"expose：{STORE} 里批的锚点命中 {text.count(TRACE_ANCHOR)} 次", file=sys.stderr)
        return 2
    text = text.replace(ANCHOR, INSERT + ANCHOR).replace(TRACE_ANCHOR, TRACE_ANCHOR + TRACE_INSERT)
    path.write_text(text)
    print(f"expose：{STORE} 加了三个公开包装与一行设了 OPUS_TRACE_BATCHES 才打的批账")
    return 0


sys.exit(main())
