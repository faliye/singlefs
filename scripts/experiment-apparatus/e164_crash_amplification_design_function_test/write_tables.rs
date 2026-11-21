//! E164 的切段、两态闭式与三种写表哈希（跑前登记 5.4「写表哈希甲 / 乙 / 丙」「mkfs 基线属性」「输入：父输出」、第七节序列化）。
//! 切段照 D13（验证路线） 已定项 4 的字面独立写：一次写只被它自己那块盘上之后的屏障（或那块盘上的 FUA）放行；
//! 当前段里每块有写的盘都放行了才关段；段里还没有写时的屏障并进下一段；流尾只有屏障的并进上一段。

use std::collections::{BTreeMap, BTreeSet};

use crate::common::sha256_hexadecimal;

pub const SECTOR_BYTES: u64 = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    Write,
    WriteForceUnitAccess,
    WriteZeroes,
    Barrier,
}

impl OperationKind {
    pub fn serialized_name(self) -> &'static str {
        match self {
            OperationKind::Write => "write",
            OperationKind::WriteForceUnitAccess => "write_fua",
            OperationKind::WriteZeroes => "write_zeroes",
            OperationKind::Barrier => "barrier",
        }
    }

    pub fn from_serialized_name(name: &str) -> Option<Self> {
        match name {
            "write" => Some(OperationKind::Write),
            "write_fua" => Some(OperationKind::WriteForceUnitAccess),
            "write_zeroes" => Some(OperationKind::WriteZeroes),
            "barrier" => Some(OperationKind::Barrier),
            _ => None,
        }
    }
}

/// 录制流里的一条操作（探针打出的 `name=op` 行）。清零写的字节摘要记成 `zeroes`，扇区摘要为空。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedOperation {
    pub device: u32,
    pub kind: OperationKind,
    pub offset_in_bytes: u64,
    pub length_in_bytes: u64,
    pub content_sha256: String,
    pub sector_sha256s: Vec<String>,
}

impl RecordedOperation {
    pub fn barrier(device: u32) -> Self {
        Self {
            device,
            kind: OperationKind::Barrier,
            offset_in_bytes: 0,
            length_in_bytes: 0,
            content_sha256: "none".to_string(),
            sector_sha256s: Vec::new(),
        }
    }

    pub fn write(device: u32, kind: OperationKind, offset_in_bytes: u64, bytes: &[u8]) -> Self {
        Self {
            device,
            kind,
            offset_in_bytes,
            length_in_bytes: u64::try_from(bytes.len()).expect("长度装得进 u64"),
            content_sha256: sha256_hexadecimal(bytes),
            sector_sha256s: bytes.chunks(512).map(sha256_hexadecimal).collect(),
        }
    }
}

/// D13 已定项 4「同一块盘上连着的屏障并成一道」：尾部那串屏障里这块盘已有一道时，新的一道不记。
pub fn append_with_barrier_merging(stream: &mut Vec<RecordedOperation>, operation: RecordedOperation) {
    let this_device_already_in_the_trailing_barriers = stream
        .iter()
        .rev()
        .take_while(|previous| previous.kind == OperationKind::Barrier)
        .any(|previous| previous.device == operation.device);
    if operation.kind == OperationKind::Barrier && this_device_already_in_the_trailing_barriers {
        return;
    }
    stream.push(operation);
}

/// 切段：交回每条操作落在第几段（从 0 起）。
pub fn segment_index_of_each_operation(operations: &[RecordedOperation]) -> Vec<usize> {
    let mut indexes = Vec::with_capacity(operations.len());
    let mut current_segment = 0usize;
    let mut unreleased_devices: BTreeSet<u32> = BTreeSet::new();
    let mut writes_in_current_segment = 0usize;
    for operation in operations {
        indexes.push(current_segment);
        let closes = match operation.kind {
            OperationKind::Write | OperationKind::WriteZeroes => {
                writes_in_current_segment += 1;
                unreleased_devices.insert(operation.device);
                false
            }
            OperationKind::WriteForceUnitAccess => {
                writes_in_current_segment += 1;
                unreleased_devices.remove(&operation.device);
                unreleased_devices.is_empty()
            }
            OperationKind::Barrier => {
                unreleased_devices.remove(&operation.device);
                writes_in_current_segment > 0 && unreleased_devices.is_empty()
            }
        };
        if closes {
            current_segment += 1;
            writes_in_current_segment = 0;
        }
    }
    let trailing_has_no_writes = writes_in_current_segment == 0;
    if trailing_has_no_writes && current_segment > 0 {
        for index in indexes.iter_mut() {
            if *index == current_segment {
                *index = current_segment - 1;
            }
        }
    }
    indexes
}

/// 每段的写数（屏障不算）。
pub fn segment_write_counts(operations: &[RecordedOperation]) -> Vec<usize> {
    let indexes = segment_index_of_each_operation(operations);
    let segment_count = indexes.iter().max().map_or(0, |last| last + 1);
    let mut counts = vec![0usize; segment_count];
    for (operation, segment) in operations.iter().zip(&indexes) {
        if operation.kind != OperationKind::Barrier {
            counts[*segment] += 1;
        }
    }
    counts
}

pub fn segment_sizes_text(counts: &[usize]) -> String {
    counts.iter().map(usize::to_string).collect::<Vec<String>>().join("+")
}

/// 两态闭式：1 + Σ(2^|段| − 1)。
pub fn closed_form_state_count(counts: &[usize]) -> u128 {
    1 + counts.iter().map(|writes| (1u128 << writes) - 1).sum::<u128>()
}

/// 写表哈希丙：写记 `W <设备> <种类> <偏移> <长度> <字节 SHA-256> <步内段号>`，屏障记 `B <设备>`。
pub fn write_table_hash_with_barriers(operations: &[RecordedOperation]) -> String {
    let indexes = segment_index_of_each_operation(operations);
    let mut text = String::new();
    for (operation, segment) in operations.iter().zip(&indexes) {
        match operation.kind {
            OperationKind::Barrier => text.push_str(&format!("B {}\n", operation.device)),
            OperationKind::Write | OperationKind::WriteForceUnitAccess | OperationKind::WriteZeroes => {
                text.push_str(&format!(
                    "W {} {} {} {} {} {}\n",
                    operation.device,
                    operation.kind.serialized_name(),
                    operation.offset_in_bytes,
                    operation.length_in_bytes,
                    operation.content_sha256,
                    segment
                ));
            }
        }
    }
    sha256_hexadecimal(text.as_bytes())
}

/// 写表哈希乙：每次写 `<设备> <偏移> <长度> <字节 SHA-256>`，屏障跳过，种类与 FUA 不进。
pub fn write_table_hash_of_bytes(operations: &[RecordedOperation]) -> String {
    let mut text = String::new();
    for operation in operations.iter().filter(|operation| operation.kind != OperationKind::Barrier) {
        text.push_str(&format!(
            "{} {} {} {}\n",
            operation.device, operation.offset_in_bytes, operation.length_in_bytes, operation.content_sha256
        ));
    }
    sha256_hexadecimal(text.as_bytes())
}

/// 扇区终值表：（设备，扇区号）→ 内容摘要，按录制次序叠。清零写在这一步里出现时停机（跑前登记 S-probe），
/// 只有 mkfs 那一段有清零，它的终值由探针的 mkfs 扇区表给。
pub fn overlay_sectors(table: &mut BTreeMap<(u32, u64), String>, operations: &[RecordedOperation]) {
    for operation in operations {
        match operation.kind {
            OperationKind::Barrier => {}
            OperationKind::WriteZeroes => panic!("步里出现清零写：扇区终值表只收 mkfs 之后的普通写与 FUA 写（S-probe）"),
            OperationKind::Write | OperationKind::WriteForceUnitAccess => {
                let first_sector = operation.offset_in_bytes / SECTOR_BYTES;
                for (sector_offset, digest) in operation.sector_sha256s.iter().enumerate() {
                    table.insert((operation.device, first_sector + sector_offset as u64), digest.clone());
                }
            }
        }
    }
}

pub fn sector_table_hash(table: &BTreeMap<(u32, u64), String>) -> String {
    let mut text = String::new();
    for ((device, sector), digest) in table {
        text.push_str(&format!("{device} {sector} {digest}\n"));
    }
    sha256_hexadecimal(text.as_bytes())
}

/// 写表哈希甲：这一步的写叠完后每个被写过的扇区的终值，按（设备，扇区）排序取 SHA-256。
pub fn write_table_hash_of_final_sectors(operations: &[RecordedOperation]) -> String {
    let mut table = BTreeMap::new();
    overlay_sectors(&mut table, operations);
    sector_table_hash(&table)
}
