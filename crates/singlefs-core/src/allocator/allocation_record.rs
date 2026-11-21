//! 分配记录：盘上一条分配记录的字段、释放标志与隔离标记，连同它的编解码。

use super::*;

/// 跨度段的最高位借作已释放标志（D3（空间分配） 已定项 7 / 已定项 11）：0 = 仍分配、1 = 已释放，不占表达跨度值的位。
pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

/// 跨度段的次高位借作隔离记号（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，落点甲；D3（空间分配） 已定项 7 跨度段那一行）：
/// 写者隔离那次发布置上，回退复活与释放时清掉；不占表达跨度值的位。与已释放标志同时置的记录是坏的（[`AllocationRecordEntryRefusal`]）。
pub const ALLOCATION_RECORD_QUARANTINE_MARK: u16 = 0x4000;

/// 跨度段里表达跨度值的那几位：去掉已释放标志与隔离记号。
const ALLOCATION_RECORD_SPAN_VALUE_BITS: u16 =
    !(ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINE_MARK);

/// 一条分配记录跨度段次高位的读法（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllocationRecordQuarantineMark {
    /// 次高位 0：不是隔离记录。已释放的记录恒取这一个（释放隔离记录时记号随之清掉）。
    NotQuarantined,
    /// 次高位 1：释放之前读盘核出这个单元有一份对不上，隔离那次发布在每块盘上那条记录里置上；记录仍分配、分配代不变。
    QuarantinedAfterTheReleaseChecksumCheck,
}

/// 盘上 20 字节解不成一条分配记录的样子。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationRecordEntryRefusal {
    /// 条目宽不等于字段表的 20（窄了切不出整条，宽了同样不收：I-1.10、C867）。
    EntryWidthDiffersFromTheFieldTable { entry_bytes: usize },
    /// 跨度段的隔离记号与已释放标志同时置（0x4000 与 0x8000，D19（块指针的结构与宽度预算） 已定项 5「带记号的记录必须未释放」）。
    QuarantineMarkOnAReleasedRecord,
}

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志与隔离记号, 分配代或释放代 8)。
/// 释放时条目不删、不点删：改写成已释放 + 释放代，留到该落点被重新分配时覆盖（覆盖是步 5 回收接上时的事：
/// 回收过的落点被再分配时那条记录改写、不追加，槽号在每盘仍唯一；新落点罩住的别的已回收记录随这次分配删掉，同一块盘上的记录
/// 罩住的槽互不相交（I-5.4（分配记录罩住的槽互不相交））；回收要 F_生效 抬到释放代之上，见 `reclaim_released_up_to`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocationRecord {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    /// 纯跨度，不含已释放标志与隔离记号。
    pub span_slots: u16,
    /// 仍分配时是分配代；已释放时是释放代。
    pub generation: CheckpointTxg,
    pub is_released: bool,
    /// 已释放的记录恒是 [`AllocationRecordQuarantineMark::NotQuarantined`]（[`Self::to_bytes`] 断言）。
    pub quarantine_mark: AllocationRecordQuarantineMark,
}

impl AllocationRecord {
    /// # Panics
    /// 跨度值占到已释放标志或隔离记号那一位；已释放的记录带着隔离记号（释放隔离记录时记号随之清掉，
    /// [`PoolAllocator::release_leaving_the_record_allocated_on`]）。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        assert_eq!(
            self.span_slots & !ALLOCATION_RECORD_SPAN_VALUE_BITS,
            0,
            "跨度值不许占到已释放标志与隔离记号那两位"
        );
        let released_flag = if self.is_released {
            ALLOCATION_RECORD_RELEASED_FLAG
        } else {
            0
        };
        let quarantine_mark = match self.quarantine_mark {
            AllocationRecordQuarantineMark::NotQuarantined => 0,
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck => {
                assert!(
                    !self.is_released,
                    "已释放的记录不带隔离记号：释放隔离记录时记号随之清掉（D19 已定项 5）"
                );
                ALLOCATION_RECORD_QUARANTINE_MARK
            }
        };
        let span_field = self.span_slots | released_flag | quarantine_mark;
        let mut writer = ByteWriter::new(20);
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(span_field);
        writer.put_u64(self.generation.0);
        writer.assert_position(20, "分配记录");
        writer.into_bytes()
    }
    /// 按字段比：设备身份，再槽号（D8（核心索引结构） 已定项 11）。
    #[must_use]
    pub fn sort_key(&self) -> (u32, u64) {
        (self.device.0, self.slot.0)
    }
    /// 分配记录 20 的读者。条目宽是索引节点头里的一个盘上字段（`parse_index_node` 只判了它 ≥ key 宽 10），
    /// 不等于 20 时拒：这里是盘上字节进字段表的边界，往里就按 20 信它。窄于 20 的切不出整条记录；
    /// 宽于 20 的同样不收（I-1.10（码 2 条目宽等于字段表宽） 写的是「等于」，欠账 C867）。
    /// 跨度段次高位 0x4000 读成隔离记号、跨度取去掉两个标志位之后的值（D19（块指针的结构与宽度预算） 已定项 5）。
    ///
    /// # Errors
    /// 条目宽不是 20 ⇒ [`AllocationRecordEntryRefusal::EntryWidthDiffersFromTheFieldTable`]；
    /// 隔离记号与已释放标志同时置 ⇒ [`AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord`]。
    pub fn parse(bytes: &[u8]) -> Result<Self, AllocationRecordEntryRefusal> {
        if bytes.len() != usize::try_from(ALLOCATION_RECORD_BYTES).expect("20") {
            return Err(
                AllocationRecordEntryRefusal::EntryWidthDiffersFromTheFieldTable {
                    entry_bytes: bytes.len(),
                },
            );
        }
        let mut reader = crate::bytes::ByteReader::at(bytes, 0);
        let device = DeviceIdentity(reader.get_u32());
        let slot = SlotNumber(reader.get_six_byte_unsigned());
        let span_field = reader.get_u16();
        let generation = CheckpointTxg(reader.get_u64());
        let is_released = span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0;
        let quarantine_mark = if span_field & ALLOCATION_RECORD_QUARANTINE_MARK == 0 {
            AllocationRecordQuarantineMark::NotQuarantined
        } else if is_released {
            return Err(AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord);
        } else {
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck
        };
        Ok(Self {
            device,
            slot,
            span_slots: span_field & ALLOCATION_RECORD_SPAN_VALUE_BITS,
            generation,
            is_released,
            quarantine_mark,
        })
    }
}

#[cfg(test)]
mod tests;
