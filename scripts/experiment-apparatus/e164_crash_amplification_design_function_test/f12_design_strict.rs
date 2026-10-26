// E164（崩溃放量新设计的小范围功能测试）F12 收严形（跑前登记 5.7）：标签什么格式化与比较都不实现
// （没有 Display、Debug、Clone、Hash、PartialEq、Ord），只交给只写的覆盖报告。复用键、写表、判定库与弱形逐字相同。
pub mod design {
    use std::collections::BTreeMap;

    /// 节点标签：只能从身份四段造出来。
    pub struct NodeLabel {
        text: String,
    }

    impl NodeLabel {
        pub fn new(code_file_name: &str, target_name: &str, step_number: u32, hash_text: &str) -> Self {
            Self {
                text: format!("{code_file_name}::{target_name}::{step_number}::{hash_text}"),
            }
        }
    }

    /// 写表：复用键只能从它算。
    pub struct WriteTable {
        pub lines: Vec<String>,
    }

    /// 复用键：只能从写表算出来。
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct ReuseKey([u8; 32]);

    impl ReuseKey {
        pub fn from_write_table(table: &WriteTable) -> Self {
            let mut folded = [0u8; 32];
            for (line_number, line) in table.lines.iter().enumerate() {
                for (byte_number, byte) in line.bytes().enumerate() {
                    folded[(line_number + byte_number) % 32] ^= byte;
                }
            }
            Self(folded)
        }
    }

    /// 判定库：键只收复用键。
    pub struct VerdictStore {
        verdicts: BTreeMap<ReuseKey, u8>,
    }

    impl VerdictStore {
        pub fn new() -> Self {
            Self {
                verdicts: BTreeMap::new(),
            }
        }
        pub fn put(&mut self, key: ReuseKey, verdict: u8) {
            self.verdicts.insert(key, verdict);
        }
        pub fn stored_count(&self) -> usize {
            self.verdicts.len()
        }
    }

    /// 覆盖报告：只写，收下标签就不再交出去。
    pub struct CoverageReport {
        labels_recorded: usize,
    }

    impl CoverageReport {
        pub fn new() -> Self {
            Self { labels_recorded: 0 }
        }
        pub fn record(&mut self, label: NodeLabel) {
            drop(label.text);
            self.labels_recorded += 1;
        }
        pub fn recorded_count(&self) -> usize {
            self.labels_recorded
        }
    }
}
