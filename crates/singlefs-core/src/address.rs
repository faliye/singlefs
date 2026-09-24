//! 地址与代号的 newtype。每一种都是它自己的类型：逻辑上不同的量传错了要编译不过。
//!
//! ```compile_fail
//! use singlefs_core::address::{DeviceOffsetInBytes, SlotNumber};
//! fn wants_an_offset(_offset: DeviceOffsetInBytes) {}
//! // 16 KiB 槽号不是设备内字节偏移：直接传过去编译不过。
//! wants_an_offset(SlotNumber(3));
//! ```

use singlefs_format::SLOT_BYTES;

/// 池内一块设备的身份，与位置条目里那 4 字节同宽（D19（块指针的结构与宽度预算） 已定项 4）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceIdentity(pub u32);

/// 设备内的 16 KiB 槽号（D3（空间分配） 已定项 7）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SlotNumber(pub u64);

/// 设备内字节偏移。只由 [`SlotNumber::to_device_offset`] 或固定结构的落点表产生。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceOffsetInBytes(pub u64);

/// checkpoint 号：发布计数、轮转键、记账代、重放水位是同一个数（D16（发布语义） 已定项 6）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheckpointTxg(pub u64);

/// journal 实例代号：mkfs 写 0，第一次可写挂载取 1（D23（journal 的角色与格式） 已定项 16）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceGeneration(pub u32);

/// journal 记录序号 jsn：实例代号 4 字节 + 计数器 6 字节（D23（journal 的角色与格式） 已定项 9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JournalSequenceNumber {
    pub instance_generation: InstanceGeneration,
    pub counter: u64,
}

/// 树 ID，从 11 起编号、与树的种类码错开（D8（核心索引结构） 已定项 11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TreeIdentifier(pub u64);

/// inode 号。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InodeNumber(pub u64);

/// 文件内的字节偏移，从文件第 0 个字节起算。与设备内字节偏移、16 KiB 槽号是三个地址空间
/// （`.claude/rules/fs-design.md`「不同地址空间必须是不同的 Rust 类型」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileOffsetInBytes(pub u64);

/// 一个文件里第几个数据单元，从 0 起：文件字节偏移除以数据单元的净荷容量取商
/// （D4（校验和位置） 已定项 5：净荷不是 2 的幂，文件偏移到单元做除法而不是移位）。
///
/// ⚠️ 它**不是** extent 叶记录 key 里那个 offset 段：那一段是文件字节偏移（D8（核心索引结构） 已定项 3，
/// 2026-09-23 定），第 n 个单元是 n × 净荷容量，由 [`DataUnitIndexInFile::first_file_byte`] 换算。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataUnitIndexInFile(pub u64);

impl DataUnitIndexInFile {
    /// 文件的第一个数据单元。只有一个单元的文件（第一个事务那一档）就是它。
    pub const FIRST: Self = Self(0);

    /// 这个单元载荷的第一个字节在文件里的偏移：单元序号乘净荷容量（D4（校验和位置） 已定项 5 的除法倒过来）。
    /// extent 叶记录 key 的 offset 段与数据单元头的锚点偏移写的都是它（D8（核心索引结构） 已定项 3；
    /// D9（加密） 已定项 6「锚点偏移 = `key.offset − ptr.extent_off`」，第一版 extent 距起点偏移恒 0）。
    ///
    /// # Panics
    /// 乘出来越过 `u64`：单元序号由一段内存里的内容长度除净荷容量得出，乘回去不超过内容长度加一个净荷。
    #[must_use]
    pub fn first_file_byte(self, payload_capacity_in_bytes: u64) -> FileOffsetInBytes {
        FileOffsetInBytes(
            self.0
                .checked_mul(payload_capacity_in_bytes)
                .expect("单元序号由内容长度除净荷容量得出，乘回去不越过 u64"),
        )
    }
}

impl SlotNumber {
    /// 槽号乘落点粒度就是设备内字节偏移；这是两个地址空间之间唯一的换算。
    #[must_use]
    pub const fn to_device_offset(self) -> DeviceOffsetInBytes {
        DeviceOffsetInBytes(self.0 * SLOT_BYTES)
    }
}

impl JournalSequenceNumber {
    /// 计数器只有 6 字节宽；超过它的值在盘上写不下。
    pub const COUNTER_LIMIT: u64 = 1 << 48;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_number_converts_to_offset_by_the_placement_granularity() {
        assert_eq!(SlotNumber(0).to_device_offset(), DeviceOffsetInBytes(0));
        assert_eq!(
            SlotNumber(50176).to_device_offset(),
            DeviceOffsetInBytes(50176 * 16384)
        );
        assert_eq!(
            SlotNumber(64).to_device_offset(),
            DeviceOffsetInBytes(1024 * 1024),
            "根环起点槽 64 = 1 MiB"
        );
    }

    #[test]
    fn journal_sequence_numbers_order_by_instance_then_counter() {
        let earlier_instance = JournalSequenceNumber {
            instance_generation: InstanceGeneration(1),
            counter: 9,
        };
        let later_instance = JournalSequenceNumber {
            instance_generation: InstanceGeneration(2),
            counter: 1,
        };
        assert!(earlier_instance < later_instance);
        assert_eq!(JournalSequenceNumber::COUNTER_LIMIT, 281_474_976_710_656);
    }
}
