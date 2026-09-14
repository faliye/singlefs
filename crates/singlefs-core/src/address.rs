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
