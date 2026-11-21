//! 定长记录的存放与外排序（登记 5.1 第 4 步）。

use super::*;

// ───────────────────────────── 定长记录的存放与外排序（5.1 第 4 步） ─────────────────────────────

/// 一条记录在溢出文件里的字节数：键 16、序号 8、结局 16、大小 4、补齐 4。
const ARM_RECORD_BYTES: usize = 48;
/// 每条臂在内存里最多放这么多条，再多就排好序写成一段溢出文件（第二、三段的全量放不进内存上限）。
pub(super) const ARM_RECORDS_IN_MEMORY_BEFORE_SPILL: usize = 8 << 20;
static SPILL_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl ArmRecord {
    fn to_bytes(self) -> [u8; ARM_RECORD_BYTES] {
        let mut bytes = [0u8; ARM_RECORD_BYTES];
        bytes[..16].copy_from_slice(&self.key.0.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.ordinal.to_le_bytes());
        bytes[24..40].copy_from_slice(&self.outcome.0.to_le_bytes());
        bytes[40..44].copy_from_slice(&self.elements.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8; ARM_RECORD_BYTES]) -> Self {
        Self {
            key: Digest128(u128::from_le_bytes(
                bytes[..16].try_into().expect("16 字节"),
            )),
            ordinal: u64::from_le_bytes(bytes[16..24].try_into().expect("8 字节")),
            outcome: Digest128(u128::from_le_bytes(
                bytes[24..40].try_into().expect("16 字节"),
            )),
            elements: u32::from_le_bytes(bytes[40..44].try_into().expect("4 字节")),
        }
    }
}

/// 一条臂的记录：内存里一段，外加若干段排好序的溢出文件（放在 `TMPDIR` 下，读完即删）。
pub(super) struct ArmRecordStore {
    memory: Vec<ArmRecord>,
    runs: Vec<std::path::PathBuf>,
    records_in_memory_before_spill: usize,
}

impl ArmRecordStore {
    pub(super) fn new(records_in_memory_before_spill: usize) -> Self {
        Self {
            memory: Vec::new(),
            runs: Vec::new(),
            records_in_memory_before_spill,
        }
    }

    pub(super) fn push(&mut self, record: ArmRecord) {
        self.memory.push(record);
        if self.memory.len() >= self.records_in_memory_before_spill {
            self.spill();
        }
    }

    fn spill(&mut self) {
        use std::io::Write;
        self.memory.sort_unstable();
        let path = std::env::temp_dir().join(format!(
            "e161-arm-records-{}-{}.bin",
            std::process::id(),
            SPILL_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let file = std::fs::File::create_new(&path).expect("建溢出文件");
        // 建成就记进 runs：写到一半 panic（盘满）时 Drop 照 runs 删掉这一份
        self.runs.push(path);
        let mut writer = std::io::BufWriter::new(file);
        for record in self.memory.drain(..) {
            writer.write_all(&record.to_bytes()).expect("写溢出文件");
        }
        writer.flush().expect("写溢出文件");
    }

    pub(super) fn absorb(&mut self, mut other: ArmRecordStore) {
        self.runs.append(&mut other.runs);
        for record in std::mem::take(&mut other.memory) {
            self.push(record);
        }
    }

    /// 按（键、序号）从小到大交给 `visit`；溢出文件读完就删。
    pub(super) fn visit_sorted(mut self, visit: &mut dyn FnMut(ArmRecord)) {
        use std::io::Read;
        if self.runs.is_empty() {
            self.memory.sort_unstable();
            for record in self.memory.drain(..) {
                visit(record);
            }
            return;
        }
        if !self.memory.is_empty() {
            self.spill();
        }
        let mut readers: Vec<std::io::BufReader<std::fs::File>> = self
            .runs
            .iter()
            .map(|path| std::io::BufReader::new(std::fs::File::open(path).expect("开溢出文件")))
            .collect();
        let read_next = |reader: &mut std::io::BufReader<std::fs::File>| -> Option<ArmRecord> {
            let mut bytes = [0u8; ARM_RECORD_BYTES];
            match reader.read_exact(&mut bytes) {
                Ok(()) => Some(ArmRecord::from_bytes(&bytes)),
                Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => None,
                Err(error) => panic!("读溢出文件：{error}"),
            }
        };
        let mut heap = std::collections::BinaryHeap::new();
        for (run_index, reader) in readers.iter_mut().enumerate() {
            if let Some(record) = read_next(reader) {
                heap.push(std::cmp::Reverse((record, run_index)));
            }
        }
        while let Some(std::cmp::Reverse((record, run_index))) = heap.pop() {
            visit(record);
            if let Some(next) = read_next(&mut readers[run_index]) {
                heap.push(std::cmp::Reverse((next, run_index)));
            }
        }
        for path in self.runs.drain(..) {
            std::fs::remove_file(&path).expect("删溢出文件");
        }
    }
}

impl Drop for ArmRecordStore {
    fn drop(&mut self) {
        for path in &self.runs {
            let _removed = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests;
