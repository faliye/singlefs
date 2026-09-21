//! 第一个事务写到的那 21 个区域，以及把它们的字节打成结果行的只读导出口。
//!
//! 这张表是 E142（第一个事务的干跑） 第十一次跑的跑前登记（`research/prompts/e142-r11-prereg.md` 第一节第 3 条）
//! 写死的那份区域清单，逐行抄自 `.claude/kb/layout/01-first-txn.md` 零那一节的写清单：
//! 第一个事务写到的 8 个单元的落点（t1..t8，两盘各一份 = 16 行）、jsn (1, 3) 那条 journal 记录的两个落点（t9）、
//! 这次发布的根槽（t10，区域 0 槽 1，只落在根环区域 0 那块盘）、这次发布写的系统配置槽（t11，两盘各一份）
//! ⇒ 16 + 2 + 1 + 2 = **21**，与那一节「⇒ 写请求数 … 21 条」逐项对得上。
//!
//! 这张表只对 E142 那套几何成立（两盘、`physical_block_size` = 512、io_min = 512 ⇒ 固定结构槽距 4096、根槽宽 512），
//! 换几何要连表一起改：`region_table_against_writes` 就是把它钉在实装身上的那道检查。
//!
//! **只读**：本模块与 [`crate::scenario`] 之外的写路径一个字节都不碰（量 5 要的是「实装写出来的字节」，不是「另写一遍」）。
//! 装置（`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`）与这里是两份代码，不共用，
//! 对不上时两边都查（`.claude/rules/implementation-first.md` 第 4 条）。

use std::collections::BTreeMap;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES,
    ROOT_RING_PRIME_STEP, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::crash::MemoryPool;
use crate::hexadecimal::hexadecimal_text;
use crate::sha256::sha256_hexadecimal;
use crate::{RecordedOperation, RecordedOperationKind};

/// 区域清单的行数：登记第一节第 3 条写死的 21 行。
pub const FIRST_TRANSACTION_REGION_COUNT: usize = 21;

/// 长度大的区域只打前后各这么多字节的十六进制（整段太长，逐字节比对靠 sha256）。
pub const HEAD_AND_TAIL_BYTES: usize = 32;

/// E142 几何的固定结构槽距：io_min = 512 ⇒ max(4096, 512) = 4096（D2（RAID 条带策略） 已定项 19）。
const E142_FIXED_STRUCTURE_SLOT_SPACING_BYTES: u64 = 4096;
/// E142 几何的根槽宽 = 探到的 `physical_block_size` = 512（字节表零那一节的 m3「371 / 512 槽」）。
const E142_ROOT_SLOT_BYTES: u64 = 512;
/// 这次发布写的系统配置槽是槽 1（t11：世代号 5、tail = 3，根槽之后再更新）。
const FIRST_TRANSACTION_SYSTEM_CONFIGURATION_SLOT_INDEX: u64 = 1;
/// 这次发布的根槽：区域 `3 mod 3` = 0 的槽 `(3 div 3) mod 8` = 1（t10）。
const FIRST_TRANSACTION_ROOT_RING_REGION: u64 = 0;
const FIRST_TRANSACTION_ROOT_RING_SLOT_INDEX: u64 = 1;
/// 根环区域 0 落在哪块盘：mkfs 的 `region_devices` 第一项（E142 取 `[0, 1, 0]`）。
const FIRST_TRANSACTION_ROOT_RING_REGION_DEVICE: u32 = 0;
/// jsn (1, 3) 那条记录在环内的偏移：两次暖机各占一条 4096（w1 在 0、w4 在 4096）⇒ 第一个事务这条在 8192。
const FIRST_TRANSACTION_JOURNAL_RECORD_RING_OFFSET: u64 = 2 * JOURNAL_RECORD_BYTES;

/// t1..t8 的落点槽号（字节表零那一节的写清单）。
const DATA_UNIT_SLOT: u64 = 50180;
const EXTENT_ROOT_SLOT: u64 = 50240;
const INODE_LEAF_SLOT: u64 = 50242;
const INODE_ROOT_SLOT: u64 = 50244;
const ALLOCATION_ROOT_SLOT: u64 = 50245;
const ACCOUNTING_ROOT_SLOT: u64 = 50246;
const MAPPING_ROOT_SLOT: u64 = 50247;
const TREE_TABLE_SLOT: u64 = 50248;

/// 十六进制打多少：整段照打，还是只打前后各 [`HEAD_AND_TAIL_BYTES`] 字节。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexadecimalExtent {
    /// 整段十六进制照打（固定结构那 5 行：根槽 512、journal 记录 4096 两份、系统配置槽 4096 两份）。
    WholeRegion,
    /// 只打 sha256 与前后各 32 字节（16 KiB / 32 KiB 的单元那 16 行）。
    HeadAndTail,
}

impl HexadecimalExtent {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            HexadecimalExtent::WholeRegion => "whole_region",
            HexadecimalExtent::HeadAndTail => "head_and_tail",
        }
    }
}

/// 区域清单的一行。`name` 与 E142 装置里同一个结构的标签同名，量 5 按 `region=` 加 `device=` 两边配对。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstTransactionRegion {
    pub name: &'static str,
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
    pub hexadecimal_extent: HexadecimalExtent,
}

const fn unit_region(
    name: &'static str,
    device_number: u32,
    slot: u64,
    span_in_slots: u64,
) -> FirstTransactionRegion {
    FirstTransactionRegion {
        name,
        device: DeviceIdentity(device_number),
        offset: SlotNumber(slot).to_device_offset(),
        length_in_bytes: span_in_slots * SLOT_BYTES,
        hexadecimal_extent: HexadecimalExtent::HeadAndTail,
    }
}

const fn fixed_structure_region(
    name: &'static str,
    device_number: u32,
    offset_in_bytes: u64,
    length_in_bytes: u64,
) -> FirstTransactionRegion {
    FirstTransactionRegion {
        name,
        device: DeviceIdentity(device_number),
        offset: DeviceOffsetInBytes(offset_in_bytes),
        length_in_bytes,
        hexadecimal_extent: HexadecimalExtent::WholeRegion,
    }
}

const FIRST_TRANSACTION_ROOT_SLOT_OFFSET: u64 =
    SlotNumber(ROOT_RING_BASE_SLOT).to_device_offset().0
        + FIRST_TRANSACTION_ROOT_RING_REGION * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES
        + FIRST_TRANSACTION_ROOT_RING_SLOT_INDEX * E142_FIXED_STRUCTURE_SLOT_SPACING_BYTES;

const FIRST_TRANSACTION_JOURNAL_RECORD_OFFSET: u64 =
    SlotNumber(JOURNAL_RING_START_SLOT).to_device_offset().0
        + FIRST_TRANSACTION_JOURNAL_RECORD_RING_OFFSET;

const FIRST_TRANSACTION_SYSTEM_CONFIGURATION_SLOT_OFFSET: u64 =
    FIRST_TRANSACTION_SYSTEM_CONFIGURATION_SLOT_INDEX * E142_FIXED_STRUCTURE_SLOT_SPACING_BYTES;

/// 登记里那 21 行，顺序就是结果行的顺序：8 个单元各两盘（t1..t8）、根记录（t10）、journal 记录两盘（t9）、系统配置槽两盘（t11）。
/// 前 16 行是 16 KiB / 32 KiB 的单元，只打 sha256 与前后 32 字节；后 5 行（根槽、journal 记录两份、系统配置槽两份）整段十六进制照打。
pub const FIRST_TRANSACTION_REGIONS: [FirstTransactionRegion; FIRST_TRANSACTION_REGION_COUNT] = [
    unit_region("data_unit", 0, DATA_UNIT_SLOT, 2),
    unit_region("data_unit", 1, DATA_UNIT_SLOT, 2),
    unit_region("extent_root", 0, EXTENT_ROOT_SLOT, 1),
    unit_region("extent_root", 1, EXTENT_ROOT_SLOT, 1),
    unit_region("inode_leaf", 0, INODE_LEAF_SLOT, 2),
    unit_region("inode_leaf", 1, INODE_LEAF_SLOT, 2),
    unit_region("inode_root", 0, INODE_ROOT_SLOT, 1),
    unit_region("inode_root", 1, INODE_ROOT_SLOT, 1),
    unit_region("allocation_root", 0, ALLOCATION_ROOT_SLOT, 1),
    unit_region("allocation_root", 1, ALLOCATION_ROOT_SLOT, 1),
    unit_region("accounting_root", 0, ACCOUNTING_ROOT_SLOT, 1),
    unit_region("accounting_root", 1, ACCOUNTING_ROOT_SLOT, 1),
    unit_region("mapping_root", 0, MAPPING_ROOT_SLOT, 1),
    unit_region("mapping_root", 1, MAPPING_ROOT_SLOT, 1),
    unit_region("tree_table", 0, TREE_TABLE_SLOT, 1),
    unit_region("tree_table", 1, TREE_TABLE_SLOT, 1),
    fixed_structure_region(
        "root_record",
        FIRST_TRANSACTION_ROOT_RING_REGION_DEVICE,
        FIRST_TRANSACTION_ROOT_SLOT_OFFSET,
        E142_ROOT_SLOT_BYTES,
    ),
    fixed_structure_region(
        "journal_record",
        0,
        FIRST_TRANSACTION_JOURNAL_RECORD_OFFSET,
        JOURNAL_RECORD_BYTES,
    ),
    fixed_structure_region(
        "journal_record",
        1,
        FIRST_TRANSACTION_JOURNAL_RECORD_OFFSET,
        JOURNAL_RECORD_BYTES,
    ),
    fixed_structure_region(
        "system_configuration",
        0,
        FIRST_TRANSACTION_SYSTEM_CONFIGURATION_SLOT_OFFSET,
        SYSTEM_CONFIGURATION_SLOT_BYTES,
    ),
    fixed_structure_region(
        "system_configuration",
        1,
        FIRST_TRANSACTION_SYSTEM_CONFIGURATION_SLOT_OFFSET,
        SYSTEM_CONFIGURATION_SLOT_BYTES,
    ),
];

/// 一个区域此刻在镜像上的字节。区域落在池外（盘不够大、表写错）是 bug，不带着坏坐标往下算。
#[must_use]
pub fn region_bytes(image: &MemoryPool, region: &FirstTransactionRegion) -> Vec<u8> {
    let end = region
        .offset
        .0
        .checked_add(region.length_in_bytes)
        .expect("区域末端放得进 u64");
    assert!(
        end <= image.device_size_in_bytes,
        "区域 {} 落在盘外：末端 {end} > 盘 {} 字节",
        region.name,
        image.device_size_in_bytes
    );
    let device_image = image
        .devices
        .get(&region.device)
        .expect("区域清单里的设备身份都是这个池里的盘（E142 两盘 0 / 1）");
    device_image.read(
        region.offset,
        usize::try_from(region.length_in_bytes).expect("区域长度放得进 usize"),
    )
}

/// 一个区域一行结果行：设备、绝对偏移、长度、内容的 sha256，再按 `hexadecimal_extent` 打十六进制。
#[must_use]
pub fn region_result_line(image: &MemoryPool, region: &FirstTransactionRegion) -> String {
    let bytes = region_bytes(image, region);
    let head = format!(
        "name=impl_region_bytes region={} device={} offset={} length={} sha256={} hexadecimal_extent={}",
        region.name,
        region.device.0,
        region.offset.0,
        region.length_in_bytes,
        sha256_hexadecimal(&bytes),
        region.hexadecimal_extent.name()
    );
    match region.hexadecimal_extent {
        HexadecimalExtent::WholeRegion => {
            format!("{head} hexadecimal={}", hexadecimal_text(&bytes))
        }
        HexadecimalExtent::HeadAndTail => {
            assert!(
                bytes.len() >= HEAD_AND_TAIL_BYTES * 2,
                "只打前后各 {HEAD_AND_TAIL_BYTES} 字节的区域，长度至少是它的两倍：{} 只有 {} 字节",
                region.name,
                bytes.len()
            );
            format!(
                "{head} head_and_tail_bytes={HEAD_AND_TAIL_BYTES} head_hexadecimal={} tail_hexadecimal={}",
                hexadecimal_text(&bytes[..HEAD_AND_TAIL_BYTES]),
                hexadecimal_text(&bytes[bytes.len() - HEAD_AND_TAIL_BYTES..])
            )
        }
    }
}

/// 整张表的结果行，顺序照表。
#[must_use]
pub fn region_result_lines(image: &MemoryPool) -> Vec<String> {
    FIRST_TRANSACTION_REGIONS
        .iter()
        .map(|region| region_result_line(image, region))
        .collect()
}

/// 表与「第一个事务真正发出的写」对得上吗：两边都是 (设备, 偏移, 长度) 的多重集，一一配对，剩下的两边各自列出来。
/// 表是从登记抄来的常量，实装是另一份代码——这道检查不让它们悄悄分叉（分叉时量 5 比的就不是同一批字节了）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionTableAgainstWrites {
    /// 第一个事务发出的写调用数（屏障不算）。
    pub write_calls: usize,
    /// 表里有、这一趟没写到的行。
    pub regions_without_a_write: Vec<&'static str>,
    /// 写到了、表里没有的落点。
    pub writes_outside_the_table: Vec<(DeviceIdentity, DeviceOffsetInBytes, u64)>,
}

impl RegionTableAgainstWrites {
    #[must_use]
    pub fn matches(&self) -> bool {
        self.write_calls == FIRST_TRANSACTION_REGION_COUNT
            && self.regions_without_a_write.is_empty()
            && self.writes_outside_the_table.is_empty()
    }
}

/// `first_transaction_operations` 是录制流里第一个事务那一段（暖机之后的全部步骤，屏障在内）。
#[must_use]
pub fn region_table_against_writes(
    first_transaction_operations: &[RecordedOperation],
) -> RegionTableAgainstWrites {
    let mut unmatched_regions: BTreeMap<
        (DeviceIdentity, DeviceOffsetInBytes, u64),
        Vec<&'static str>,
    > = BTreeMap::new();
    for region in &FIRST_TRANSACTION_REGIONS {
        unmatched_regions
            .entry((region.device, region.offset, region.length_in_bytes))
            .or_default()
            .push(region.name);
    }
    let mut write_calls = 0;
    let mut writes_outside_the_table = Vec::new();
    for operation in first_transaction_operations {
        match operation.kind {
            RecordedOperationKind::Barrier => continue,
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                write_calls += 1;
            }
        }
        let key = (operation.device, operation.offset, operation.length);
        match unmatched_regions.get_mut(&key) {
            Some(names) => {
                names.pop();
                if names.is_empty() {
                    unmatched_regions.remove(&key);
                }
            }
            None => writes_outside_the_table.push(key),
        }
    }
    RegionTableAgainstWrites {
        write_calls,
        regions_without_a_write: unmatched_regions.into_values().flatten().collect(),
        writes_outside_the_table,
    }
}
