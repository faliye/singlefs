//! 池级 checker 查窄的几条不变量补全（代码审阅 6c 第 7、9 条与实审 B1 留下的几件，里程碑二收尾批「实审 B2」，用户 2026-09-27 定），
//! 每一格配一份会红的坏镜像（或一份在改之前会误判的合法镜像）：
//! ① I-7.4（近 K 代块未被复用） 对被抛弃、还在根环里的根也判；根环有读不出的槽时这一半判不了，整条不报成立；
//! ② 老一版引用一棵已被前一版走过的共享子树时，子树里的落点并进这一版再比交叉（I-5.1（物理范围不重叠））；
//! ③ I-2.5（位置条目按设备身份升序） 判 journal 点名项；④ I-9.2（条目身份与子头相符） 认类型段 0、往下走；
//! ⑤ I-9.4（容器号不超最小 key） 三句都判；⑥ I-3.9 / I-5.4 读根记录直接持有的那棵分配记录树（C512）；
//! ⑦ I-7.7（系统配置实例代号不低于根环） 一个槽读不出时 ① ② 分开报；
//! ⑧ 新立的「中央映射条目的 key 与它指的单元头相符」（I-1.11（映射 key 与单元头相符）） 补上补零两字节与
//!    码 1 / 码 3 的出生树，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；
//! ⑨ journal 计数器 0 判 I-8.6（反向链算法） 违例。
//! 镜像都从合法历史出发，改一处；被改单元重封、新整单元校验和沿引用链补到根槽。

mod common;
mod common_tree_split;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    crash_state_devices, memory_pool_of_sparse_devices, parameters, publish_overwrite_in_process,
    BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use common_tree_split::{
    capacities, propagate_the_new_checksum, read_unit_on_device, reseal_unit_by_its_class,
    root_ring_slot_offsets, write_unit_to_the_same_slot_on_both_devices, TreeSplitPool,
    ACCOUNTING_OF_THE_NODE_FORMAT, CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
};
use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT};
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::BirthSequence;
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
use singlefs_format::{DATA_UNIT_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 新立那一条不变量：I-1.11（映射 key 与单元头相符），登记在 `.claude/kb/invariants.md`。
const MAPPING_KEY_INVARIANT: &str = "I-1.11";

/// 第一个文件版本的单元区只用到 50 26x 附近：这两个槽在两块盘上都没被分配、没被引用。
const SLOT_NOBODY_ALLOCATED: u64 = 50_400;

/// 根槽：判定宽度 512，自证校验和 32 字节在 138；实例代号在 24、checkpoint_txg 在 28；实例表指针 86 字节在 170
/// （D22（单元原子性怎么合成） 已定项 7 的偏移表）。
const ROOT_SLOT_BYTES: usize = 512;
const ROOT_SLOT_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;

/// 指针 86 / 88 里的几段（D19（块指针的结构与宽度预算） 的字段表）：出生树 34、出生 txg 42、两条位置条目 50 与 64
/// （设备身份 4 + 槽号 6 + 整单元校验和 4）、码 2 / 码 3 指针的实例代号 78 与出生序号 82。
const POINTER_BIRTH_TREE_OFFSET: usize = 34;
const POINTER_BIRTH_TXG_OFFSET: usize = 42;
const POINTER_LOCATION_ENTRY_OFFSETS: [usize; 2] = [50, 64];
const POINTER_INSTANCE_OFFSET: usize = 78;
const POINTER_BIRTH_SEQUENCE_OFFSET: usize = 82;
const LOCATION_ENTRY_SLOT_OFFSET: usize = 4;
const LOCATION_ENTRY_CHECKSUM_OFFSET: usize = 10;
const LOCATION_ENTRY_BYTES: usize = 14;

/// inode 树内部条目 120（D8（核心索引结构） 已定项 6）：分隔 key 8 + 身份引用 26（出生树 8、打包记录类型 2、容器号 8、容器出生代 8）+ 子指针 86。
const INODE_ENTRY_BIRTH_TREE_OFFSET: usize = 8;
const INODE_ENTRY_RECORD_TYPE_OFFSET: usize = 16;
const INODE_ENTRY_CONTAINER_OFFSET: usize = 18;
const INODE_ENTRY_CONTAINER_BIRTH_OFFSET: usize = 26;
const INODE_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
const INODE_ENTRY_BYTES: u16 = 120;
/// 类型段 0：子单元是码 2 的 inode 树内部节点（I-9.2（条目身份与子头相符））。
const INODE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;

/// 码 3 打包记录单元（D18（块里携带什么信息） 已定项 11）：声明长度 2 在 8、容器号 8 在 53、记录数 2 在 69、记录区从 136 起；
/// 打包记录类型 2 的记录 140，inode 号在记录偏移 0。
const PACKED_DECLARED_LENGTH_OFFSET: usize = 8;
const PACKED_CONTAINER_OFFSET: usize = 53;
const PACKED_RECORD_COUNT_OFFSET: usize = 69;
const PACKED_RECORDS_START: usize = 136;
const INODE_RECORD_BYTES: usize = 140;

/// 记账条目 34（D5（快照 / 空间记账机制） 已定项 10）：统计量 2 在 0、设备 4 在 10、值 8 在 22。
const ACCOUNTING_ROW_STATISTIC_OFFSET: usize = 0;
const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 2 + 8;
const ACCOUNTING_ROW_VALUE_OFFSET: usize = 2 + 8 + 4 + 8;
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
const STATISTIC_INODE_WATERMARK: u16 = 12;
const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;

/// 中央映射条目 55（D19（块指针的结构与宽度预算） 已定项 6 / 10）：key 27 = 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6，
/// 码 2 / 码 3 的尾段是出生序号 4 + 补零 2；value 28 = 位置条目 14 × 2。
const MAPPING_KEY_CLASS_OFFSET: usize = 0;
const MAPPING_KEY_BIRTH_TREE_OFFSET: usize = 1;
const MAPPING_KEY_TAIL_OFFSET: usize = 21;
const MAPPING_KEY_PADDING_OFFSET: usize = 25;
const MAPPING_KEY_BYTES: usize = 27;
const UNIT_CLASS_DATA: u8 = 1;
const UNIT_CLASS_INDEX_NODE: u8 = 2;
const UNIT_CLASS_PACKED: u8 = 3;

/// journal 记录（D23（journal 的角色与格式） 已定项 4 / 已定项 17）：点名项数 4 在 12、计数器 6 在 20、
/// 自证校验和 32 在 46（罩整条 4096）、载荷校验和 4 在 95（罩头之后的点名项）、头 311、点名项 56（两条位置条目在前）。
const JOURNAL_RECORD_NAMED_COUNT_OFFSET: usize = 12;
const JOURNAL_RECORD_COUNTER_OFFSET: usize = 20;
const JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET: usize = 46;
const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
const JOURNAL_RECORD_HEADER_BYTES: usize = 311;
const JOURNAL_NAMED_ENTRY_BYTES: usize = 56;
const JOURNAL_RECORD_BYTES: usize = 4096;

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：设备 4 + 槽号 6 + 跨度段 2（最高位是已释放标志）+ 分配代或释放代 8。
const ALLOCATION_RECORD_SLOT_OFFSET: usize = 4;
const ALLOCATION_RECORD_SPAN_OFFSET: usize = 10;
const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
/// 分配记录树（按位置寻址）的节点 key 宽 10，树 ID 0（C512：树表 0 条那一版由根记录直接持有，与树表、实例表同是树 ID 0）。
const ALLOCATION_RECORD_TREE_KEY_WIDTH: usize = 10;

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn read_six_byte_slot_at(bytes: &[u8], offset: usize) -> u64 {
    let mut slot = [0u8; 8];
    slot[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(slot)
}

fn read_bytes(image: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    image
        .devices
        .get(&DeviceIdentity(device))
        .expect("盘")
        .read(DeviceOffsetInBytes(offset), length)
}

fn write_bytes(image: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    image
        .devices
        .get_mut(&DeviceIdentity(device))
        .expect("盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}

/// 一份镜像上判红的不变量（带说明）。
fn violations(image: &dyn ImageReader) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn violated_names(violations: &[(&'static str, String)]) -> Vec<&'static str> {
    violations.iter().map(|(name, _)| *name).collect()
}

fn verdict_of(image: &dyn ImageReader, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .expect("清单里有")
        .1
}

fn detail_of_the_violation(image: &dyn ImageReader, invariant: &str) -> String {
    match verdict_of(image, invariant) {
        InvariantVerdict::Violated(detail) => detail,
        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
            panic!("{invariant} 该判违例，实际 {other:?}")
        }
    }
}

/// 一份镜像，其中一块盘上的一段字节读不出（暂时读错的样子）：读到跟这一段有交叠的都交 None，别的照原样读。
struct ImageWithAnUnreadableRange<'image> {
    image: &'image MemoryPool,
    device: u32,
    unreadable_offsets: std::ops::Range<u64>,
}

impl ImageReader for ImageWithAnUnreadableRange<'_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.image)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.image, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let end = offset + u64::try_from(length).expect("长度");
        if device == self.device
            && offset < self.unreadable_offsets.end
            && self.unreadable_offsets.start < end
        {
            return None;
        }
        ImageReader::read(self.image, device, offset, length)
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_unit_slots(self.image, device)
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_journal_slots(self.image, device)
    }
}

/// 根环里一个此刻没有根的槽（自证不过的槽）：(盘, 偏移)。读不出的那一段放在这里，择根一条都不少。
fn an_empty_root_ring_slot(image: &MemoryPool) -> (u32, u64) {
    root_ring_slot_offsets()
        .into_iter()
        .rev()
        .find(|(device, offset)| {
            &read_bytes(image, *device, *offset, ROOT_SLOT_BYTES)[..4] != b"SFSR"
        })
        .expect("根环 24 个槽里有空着的")
}

/// 根环里那条 (实例, txg) 的根住的槽：(盘, 偏移)。
fn root_slot_holding(image: &MemoryPool, instance: u32, checkpoint_txg: u64) -> (u32, u64) {
    root_ring_slot_offsets()
        .into_iter()
        .find(|(device, offset)| {
            let bytes = read_bytes(image, *device, *offset, ROOT_SLOT_BYTES);
            &bytes[..4] == b"SFSR"
                && read_u32_at(&bytes, ROOT_RECORD_INSTANCE_OFFSET) == instance
                && read_u64_at(&bytes, ROOT_RECORD_CHECKPOINT_TXG_OFFSET) == checkpoint_txg
        })
        .expect("根环里有这条根")
}

fn reseal_root_slot(bytes: &mut [u8]) {
    let digest = wide_checksum_with_field_zeroed(bytes, ROOT_SLOT_BYTES, ROOT_SLOT_CHECKSUM_OFFSET);
    bytes[ROOT_SLOT_CHECKSUM_OFFSET..ROOT_SLOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
}

/// journal 记录改过之后重封：先算载荷校验和（罩点名项），再算头自证校验和（罩整条 4096）。
fn reseal_journal_record(record: &mut [u8]) {
    let named_count =
        usize::try_from(read_u32_at(record, JOURNAL_RECORD_NAMED_COUNT_OFFSET)).expect("点名项数");
    let payload_end = JOURNAL_RECORD_HEADER_BYTES + named_count * JOURNAL_NAMED_ENTRY_BYTES;
    let payload_checksum = crc32_castagnoli(&record[JOURNAL_RECORD_HEADER_BYTES..payload_end]);
    record[JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET..JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    let digest = wide_checksum_with_field_zeroed(
        record,
        JOURNAL_RECORD_BYTES,
        JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET,
    );
    record[JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET..JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
}

/// 单元区起点（mkfs 种的实例表落在这里）与从它往后扫多少个槽：这几份镜像上用到的落点都在前几百个槽里。
const UNIT_AREA_FIRST_SLOT: u64 = 50_176;
const SLOTS_SCANNED_FOR_UNITS: u64 = 1024;

/// 单元区里盘 0 上此刻的单元：(起点槽, 字节数)，按头里的类标签认（码 2 一槽，码 1 / 码 3 两槽）。
/// 沿引用链补校验和（`propagate_the_new_checksum`）要一张单元表，镜像是现造的、没有写死的表时拿它。
fn units_found_on_device_zero(image: &MemoryPool) -> Vec<(u64, usize)> {
    let mut units = Vec::new();
    let mut slot = UNIT_AREA_FIRST_SLOT;
    while slot < UNIT_AREA_FIRST_SLOT + SLOTS_SCANNED_FOR_UNITS {
        let header = read_bytes(image, 0, slot * SLOT_BYTES, 512);
        let slots_taken = if &header[..4] == b"SFSU" {
            match header[6] {
                UNIT_CLASS_INDEX_NODE => {
                    units.push((slot, node_bytes()));
                    1
                }
                UNIT_CLASS_DATA | UNIT_CLASS_PACKED => {
                    units.push((slot, data_unit_bytes()));
                    2
                }
                _unregistered_unit_class => 1,
            }
        } else {
            1
        };
        slot += slots_taken;
    }
    units
}

/// 按解开的头重建一个码 2 节点：只换给定的几样，别的字段照原样。
fn rebuilt_index_node(
    header: &IndexNodeHeader,
    level: u8,
    smallest_key: &[u8],
    largest_key: &[u8],
    birth_sequence: BirthSequence,
    entries: &[Vec<u8>],
) -> Vec<u8> {
    build_index_node(
        header.tree,
        level,
        header.key_width,
        smallest_key,
        largest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽"),
        entries,
    )
}

/// 一个码 2 单元原样重建一遍，核它与盘上的逐字节相同（重建没有改到别的字段）。
fn assert_the_rebuild_reproduces(unit_bytes: &[u8], header: &IndexNodeHeader) {
    assert_eq!(
        rebuilt_index_node(
            header,
            header.level,
            &header.smallest_key,
            &header.largest_key,
            header.birth_sequence,
            &header.entries,
        ),
        unit_bytes,
        "原样重建要与盘上的逐字节相同"
    );
}

/// 一个单元（两盘同槽）换成 `replacement`，新整单元校验和沿引用链补到根槽。
fn replace_unit(image: &mut MemoryPool, units: &[(u64, usize)], slot: u64, replacement: &[u8]) {
    let before = read_unit_on_device(image, 0, slot, replacement.len());
    write_unit_to_the_same_slot_on_both_devices(image, slot, replacement);
    propagate_the_new_checksum(
        image,
        units,
        (
            slot,
            crc32_castagnoli(&before),
            crc32_castagnoli(replacement),
        ),
    );
}

/// 记账树（产品容量下第一个文件版本是根兼叶）里的行按 `change` 改，重建、补校验和。
fn change_accounting_rows(
    image: &mut MemoryPool,
    output: &TransactionOutput,
    change: impl Fn(&mut Vec<Vec<u8>>),
) {
    let accounting_slot = output.unit(TransactionUnit::AccountingTree).slot.0;
    let accounting = read_unit_on_device(image, 0, accounting_slot, node_bytes());
    let header = parse_index_node(&accounting).expect("记账树解得开");
    assert_eq!(header.level, 0, "产品容量下记账树是根兼叶");
    let mut rows = header.entries.clone();
    change(&mut rows);
    let rebuilt = rebuilt_index_node(
        &header,
        header.level,
        &header.smallest_key,
        &header.largest_key,
        header.birth_sequence,
        &rows,
    );
    replace_unit(
        image,
        &units_found_on_device_zero(image),
        accounting_slot,
        &rebuilt,
    );
}

/// 一行记账（统计量, 设备）的值加上 `delta`（可以是负数）。
fn adjust_accounting_row(rows: &mut [Vec<u8>], statistic: u16, device: u32, delta: i64) {
    let row = rows
        .iter_mut()
        .find(|row| {
            row[ACCOUNTING_ROW_STATISTIC_OFFSET..ACCOUNTING_ROW_STATISTIC_OFFSET + 2]
                == statistic.to_le_bytes()
                && row[ACCOUNTING_ROW_DEVICE_OFFSET..ACCOUNTING_ROW_DEVICE_OFFSET + 4]
                    == device.to_le_bytes()
        })
        .expect("记账里有这一行");
    let value = read_u64_at(row, ACCOUNTING_ROW_VALUE_OFFSET)
        .checked_add_signed(delta)
        .expect("调完不越界");
    row[ACCOUNTING_ROW_VALUE_OFFSET..ACCOUNTING_ROW_VALUE_OFFSET + 8]
        .copy_from_slice(&value.to_le_bytes());
}

/// 两块盘上各多占 `slots` 个槽：记账的「已分配」加、「空闲」减（新放进去的单元不在树里别处记账，这一步让 I-3.1 / I-3.11 / I-5.2 仍对得上）。
fn account_for_units_placed_by_hand(
    image: &mut MemoryPool,
    output: &TransactionOutput,
    slots: u64,
    inode_watermark: Option<u64>,
) {
    let bytes = i64::try_from(slots * SLOT_BYTES).expect("字节数");
    change_accounting_rows(image, output, |rows| {
        for device in [0u32, 1] {
            adjust_accounting_row(rows, STATISTIC_ALLOCATED_BYTES, device, bytes);
            adjust_accounting_row(rows, STATISTIC_FREE_BYTES, device, -bytes);
        }
        if let Some(watermark) = inode_watermark {
            let row = rows
                .iter_mut()
                .find(|row| {
                    row[ACCOUNTING_ROW_STATISTIC_OFFSET..ACCOUNTING_ROW_STATISTIC_OFFSET + 2]
                        == STATISTIC_INODE_WATERMARK.to_le_bytes()
                        && row[ACCOUNTING_ROW_DEVICE_OFFSET..ACCOUNTING_ROW_DEVICE_OFFSET + 4]
                            == STATISTIC_NO_DEVICE_DIMENSION.to_le_bytes()
                })
                .expect("记账里有 inode 号水位那一行");
            row[ACCOUNTING_ROW_VALUE_OFFSET..ACCOUNTING_ROW_VALUE_OFFSET + 8]
                .copy_from_slice(&watermark.to_le_bytes());
        }
    });
}

fn product_capacity_pool() -> TreeSplitPool {
    TreeSplitPool::with_the_first_file_version_under(capacities(
        ACCOUNTING_OF_THE_NODE_FORMAT,
        CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
    ))
}

// ─── ① I-7.4（近 K 代块未被复用） 被抛弃根那一半 ───

/// A (1, 3)、B (1, 4) → 重开取号 2、写行、暖机、C (2, 8)（与 `checker_known_bad_images.rs` 的同名搭建同一段）。交回池与 C 那一版。
fn pool_through_the_third_version(tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_second_instance)
        .expect("重开取号 2");
    pool.devices = Some(devices_reopened_for_the_second_instance);
    pool.allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third_content: Vec<u8> = (0..2500usize)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect();
    let third = publish_overwrite_in_process(
        &mut pool,
        &current,
        &third_content,
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(2),
    )
    .expect("覆盖写 C");
    pool.output = third.clone();
    (pool, third)
}

/// 崩溃恢复抛弃 C 之后的镜像（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元
/// 暂时读不出，恢复落到 (2, 7)，实例 3 写行与暖机，再把 C 写回）：C (2, 8) 读得出、按实例 3 的实例表判出局。
/// 实例 3 那次挂载看不见 C，影子账隔离 0（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到））：它的分配器把 C 独占的槽当空闲槽，
/// 写行与暖机这几次还没分到它们（接着写才会，见 [`image_after_the_blind_mount_handed_out_a_unit_of_the_abandoned_root`]）。
/// 交回镜像与 C 那一版。
fn image_after_a_recovery_abandoned_the_third_root(tag: &str) -> (MemoryPool, TransactionOutput) {
    let (mut pool, third) = pool_through_the_third_version(tag);
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    (pool.memory_pool(), third)
}

/// C 那一版写出的单元里，此刻盘上逐字节已不是 C 写的那几个（被实例 3 那次挂载重新分配、写了别的）。
fn units_of_the_abandoned_version_overwritten_since(
    image: &MemoryPool,
    abandoned: &TransactionOutput,
) -> Vec<u64> {
    abandoned
        .units
        .iter()
        .filter(|unit| read_unit_on_device(image, 0, unit.slot.0, unit.bytes.len()) != unit.bytes)
        .map(|unit| unit.slot.0)
        .collect()
}

/// 看不见 C 的那次挂载（实例 3）里最多接着覆盖写几次：它的分配器把 C 独占的槽当空闲槽，接着写总会分到其中一个。
const OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST: u64 = 6;

/// C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形：崩溃恢复抛弃 C 之后，看不见 C 的那次挂载（实例 3，影子账隔离 0）
/// 里接着覆盖写，直到它写的单元落在 C 独占的槽上。交回那一刻的镜像、C 那一版与被重写的 C 的槽。
fn image_after_the_blind_mount_handed_out_a_unit_of_the_abandoned_root(
    tag: &str,
) -> (MemoryPool, TransactionOutput, Vec<u64>) {
    let (mut pool, third) = pool_through_the_third_version(tag);
    let blind_mount =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    let instance = blind_mount.output.instance;
    for round in 0..OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST {
        let round_index = usize::try_from(round).expect("轮次");
        let content: Vec<u8> = (0..(1800 + round_index * 53))
            .map(|index| u8::try_from((index * 13 + round_index) % 251).expect("小于 256"))
            .collect();
        let previous = pool.output.clone();
        pool.output = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content,
            FIXED_WRITE_TIME_SECONDS + 300 + round,
            instance,
        )
        .expect("看不见 C 的那次挂载里接着覆盖写");
        let image = pool.memory_pool();
        let overwritten = units_of_the_abandoned_version_overwritten_since(&image, &third);
        if !overwritten.is_empty() {
            return (image, third, overwritten);
        }
    }
    panic!("看不见 C 的那次挂载写了 {OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST} 版，一个都没落在 C 独占的槽上");
}

/// C554 那一形（今天的实现照实会红，用户 2026-09-27 定「照实红、不放宽」，C554 修好之后转绿）：C 还在根环里、读得出，
/// 按 I-7.4（近 K 代块未被复用） 那一行「被抛弃时间线的根引用的块在离开根环之前同样不许重新分配」，从 C 出发读回来对不上
/// ⇒ 只红 I-7.4，说明点出是被抛弃的根。改之前 checker 只判回退候选集，C 一格都不判，这里全绿。
#[test]
fn an_abandoned_root_whose_units_a_mount_blind_to_it_handed_out_again_reddens_only_the_reuse_invariant(
) {
    let (image, _, overwritten) =
        image_after_the_blind_mount_handed_out_a_unit_of_the_abandoned_root(
            "b2-abandoned-root-reused-by-a-blind-mount",
        );
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        ["I-7.4"],
        "C 的槽 {overwritten:?} 被重写：只该红 I-7.4：{red:?}"
    );
    assert!(
        red[0].1.contains("被抛弃"),
        "红在被抛弃的根上：{}",
        red[0].1
    );
}

/// 崩溃恢复抛弃 C 之后那一刻（看不见 C 的那次挂载只写了行与暖机，没落在 C 的槽上）：一条都不红。把 C 的数据单元（只有被抛弃的 C 引用）
/// 两份都抹成 0 ⇒ 只红 I-7.4，说明点出被抛弃的根与读回来对不上的数据单元。改之前 checker 不走被抛弃的根，抹了照样全绿。
#[test]
fn erasing_a_unit_only_an_abandoned_root_references_reddens_only_the_reuse_invariant() {
    let (mut image, abandoned) =
        image_after_a_recovery_abandoned_the_third_root("b2-abandoned-root-erased-unit");
    assert_eq!(
        units_of_the_abandoned_version_overwritten_since(&image, &abandoned),
        Vec::<u64>::new(),
        "那次挂载只写行与暖机，C 的单元都还在"
    );
    assert_eq!(violations(&image), Vec::new(), "抛弃 C 之后那一刻全绿");
    let data_unit = abandoned.unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    for device in [0u32, 1] {
        write_bytes(
            &mut image,
            device,
            data_unit.slot.0 * SLOT_BYTES,
            &vec![0u8; data_unit.bytes.len()],
        );
    }
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        ["I-7.4"],
        "C 的数据单元只有被抛弃的 C 引用：抹掉它只红 I-7.4：{red:?}"
    );
    assert!(
        red[0].1.contains("被抛弃") && red[0].1.contains("数据单元"),
        "说明点出被抛弃的根与读回来对不上的数据单元：{}",
        red[0].1
    );
}

/// 根环有一个读不出的槽、最新根的实例表里又有行（有根可以被判抛弃）时：那个槽里可能坐着一条被抛弃的根，它引用的块有没有被复用判不了——
/// I-7.4 被抛弃根那一半报「判不了」，整条不报成立（用户 2026-09-27 定「被抛弃的根读不出时报判不了、不报绿」）。
/// 改之前候选集那一半判成立就报成立。
#[test]
fn an_unreadable_root_ring_slot_leaves_the_abandoned_half_of_the_reuse_invariant_not_judged() {
    let (pool, _) = pool_through_the_third_version("b2-unreadable-root-slot-with-rows");
    let image = pool.memory_pool();
    assert_eq!(
        verdict_of(&image, "I-7.4"),
        InvariantVerdict::Holds,
        "读得全的时候 I-7.4 真被评估过且成立"
    );
    let (device, offset) = an_empty_root_ring_slot(&image);
    let with_an_unreadable_slot = ImageWithAnUnreadableRange {
        image: &image,
        device,
        unreadable_offsets: offset..offset + 512,
    };
    match verdict_of(&with_an_unreadable_slot, "I-7.4") {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("读不出"),
            "理由要说是根环有读不出的槽：{reason}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => {
            panic!("被抛弃根那一半判不了，I-7.4 不许报成立：{other:?}")
        }
    }
}

// ─── ② 共享子树里的落点并进老一版 ───

/// 第一个文件版本 F 之后推一次空发布 N：extent 树没动，F 与 N 的树表都指着同一个 extent 根 E，E 内联着 F 的数据单元 D。
/// 把 F 那条根的实例表指针改指 D（位置条目带 D 的整单元校验和，读得出）：F 这一版里 D 被引用了两次（一次经实例表指针，一次经 E），
/// 是一个交叉链接。先走的 N 已经走过 E，F 走到 E 不重走——改之前 D 不进 F 这一版的集合，I-5.1（物理范围不重叠） 不红；
/// 把 E 下面缓存的落点并进 F 之后红，而且只红它。
#[test]
fn an_older_version_naming_again_a_placement_inside_a_subtree_it_shares_with_a_newer_version_reddens_the_disjoint_ranges_invariant(
) {
    let mut pool = product_capacity_pool();
    pool.empty_publish(capacities(
        ACCOUNTING_OF_THE_NODE_FORMAT,
        CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
    ));
    let older = pool.version_before_the_current_one().clone();
    let clean = pool.memory_pool();
    assert_eq!(violations(&clean), Vec::new(), "空发布之后的干净镜像全绿");
    let extent_root_of = |output: &TransactionOutput| {
        let tree_table = read_unit_on_device(
            &clean,
            0,
            output.unit(TransactionUnit::TreeTable).slot.0,
            node_bytes(),
        );
        let header = parse_index_node(&tree_table).expect("树表解得开");
        header
            .entries
            .iter()
            .find(|entry| u16::from_le_bytes([entry[10], entry[11]]) == 1)
            .expect("树表里有 extent 树那一条")[14..100]
            .to_vec()
    };
    assert_eq!(
        extent_root_of(&older),
        extent_root_of(&pool.output),
        "空发布照抄 extent 树：两版的树表指着同一个 extent 根"
    );
    let data_unit = older.unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let data_unit_checksum = crc32_castagnoli(&data_unit.bytes);
    let mut image = clean.clone();
    let (root_device, root_offset) =
        root_slot_holding(&image, older.root.instance.0, older.root.checkpoint_txg.0);
    let mut root_slot = read_bytes(&image, root_device, root_offset, ROOT_SLOT_BYTES);
    for location in POINTER_LOCATION_ENTRY_OFFSETS {
        let start = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + location;
        root_slot[start + LOCATION_ENTRY_SLOT_OFFSET..start + LOCATION_ENTRY_SLOT_OFFSET + 6]
            .copy_from_slice(&data_unit.slot.0.to_le_bytes()[..6]);
        root_slot
            [start + LOCATION_ENTRY_CHECKSUM_OFFSET..start + LOCATION_ENTRY_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&data_unit_checksum.to_le_bytes());
    }
    reseal_root_slot(&mut root_slot);
    write_bytes(&mut image, root_device, root_offset, &root_slot);
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        ["I-5.1"],
        "F 这一版里 D 被引用两次，只该红 I-5.1：{red:?}"
    );
    assert!(
        red[0].1.contains("被同一版引用了两次")
            && red[0].1.contains(&format!("槽 {}", data_unit.slot.0)),
        "说明要点出 D 那一槽被同一版引用了两次：{}",
        red[0].1
    );
}

// ─── ③ I-2.5（位置条目按设备身份升序） 判 journal 点名项 ───

/// 第一个文件版本那条 journal 记录（环里最后一条，后面没有同实例的记录接它的链）第一个点名项的两条位置条目对调（盘 1 在前），
/// 两盘各一份都改、重封：I-2.5 逐字罩「journal 点名项里的位置条目数组」。改之前 checker 解出点名项就丢，全绿。
#[test]
fn a_journal_named_entry_whose_location_entries_are_not_in_ascending_device_order_reddens_only_the_location_order_invariant(
) {
    let pool = product_capacity_pool();
    let mut image = pool.memory_pool();
    let counter = pool.output.record.counter;
    let offset = singlefs_core::journal::record_offset(counter, JOURNAL_RING_DEFAULT_BYTES).0;
    for device in [0u32, 1] {
        let mut record = read_bytes(&image, device, offset, JOURNAL_RECORD_BYTES);
        assert!(
            read_u32_at(&record, JOURNAL_RECORD_NAMED_COUNT_OFFSET) > 0,
            "第一个文件版本那条记录点名了它写的单元"
        );
        let entry = JOURNAL_RECORD_HEADER_BYTES;
        let first: Vec<u8> = record[entry..entry + LOCATION_ENTRY_BYTES].to_vec();
        let second: Vec<u8> =
            record[entry + LOCATION_ENTRY_BYTES..entry + 2 * LOCATION_ENTRY_BYTES].to_vec();
        assert_eq!(
            (read_u32_at(&first, 0), read_u32_at(&second, 0)),
            (0, 1),
            "点名项的两条位置条目本来按设备身份升序"
        );
        record[entry..entry + LOCATION_ENTRY_BYTES].copy_from_slice(&second);
        record[entry + LOCATION_ENTRY_BYTES..entry + 2 * LOCATION_ENTRY_BYTES]
            .copy_from_slice(&first);
        reseal_journal_record(&mut record);
        write_bytes(&mut image, device, offset, &record);
    }
    let red = violations(&image);
    assert_eq!(violated_names(&red), ["I-2.5"], "只该红 I-2.5：{red:?}");
    assert!(
        red[0].1.contains("点名项"),
        "说明要点出是 journal 点名项：{}",
        red[0].1
    );
}

// ─── ④ I-9.2（条目身份与子头相符） 认类型段 0 ───

/// 第一个文件版本的 inode 树（根层级 1，一条类型段 2 的条目指着唯一那片叶容器）改成三层：新放一个层级 1 的码 2 节点 M，
/// 装原来那条条目；根改成层级 2、只装一条类型段 0 的条目指着 M（身份引用的出生树、容器号、出生代三段 `identity_segments`）。
/// M 占的两盘各一槽记进记账（已分配加、空闲减）。交回镜像与 M 的槽。
fn image_with_the_inode_tree_grown_to_three_levels(
    pool: &TreeSplitPool,
    identity_segments: (u64, u64, u64),
) -> MemoryPool {
    let mut image = pool.memory_pool();
    let inode_root = pool.output.unit(TransactionUnit::InodeRoot);
    let root_header = parse_index_node(&inode_root.bytes).expect("inode 根解得开");
    assert_the_rebuild_reproduces(&inode_root.bytes, &root_header);
    assert_eq!(
        (root_header.level, root_header.entries.len()),
        (1, 1),
        "第一个文件版本的 inode 根层级 1、一条条目"
    );
    let middle_birth_sequence = BirthSequence(root_header.birth_sequence.0 + 1000);
    let middle = rebuilt_index_node(
        &root_header,
        1,
        &root_header.smallest_key,
        &root_header.largest_key,
        middle_birth_sequence,
        &root_header.entries,
    );
    write_unit_to_the_same_slot_on_both_devices(&mut image, SLOT_NOBODY_ALLOCATED, &middle);
    let leaf_entry = &root_header.entries[0];
    let mut middle_pointer = leaf_entry[INODE_ENTRY_CHILD_POINTER_OFFSET..].to_vec();
    middle_pointer[POINTER_BIRTH_TREE_OFFSET..POINTER_BIRTH_TREE_OFFSET + 8]
        .copy_from_slice(&root_header.tree.0.to_le_bytes());
    middle_pointer[POINTER_BIRTH_TXG_OFFSET..POINTER_BIRTH_TXG_OFFSET + 8]
        .copy_from_slice(&root_header.birth_txg.0.to_le_bytes());
    for location in POINTER_LOCATION_ENTRY_OFFSETS {
        middle_pointer
            [location + LOCATION_ENTRY_SLOT_OFFSET..location + LOCATION_ENTRY_SLOT_OFFSET + 6]
            .copy_from_slice(&SLOT_NOBODY_ALLOCATED.to_le_bytes()[..6]);
        middle_pointer[location + LOCATION_ENTRY_CHECKSUM_OFFSET
            ..location + LOCATION_ENTRY_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&crc32_castagnoli(&middle).to_le_bytes());
    }
    middle_pointer[POINTER_INSTANCE_OFFSET..POINTER_INSTANCE_OFFSET + 4]
        .copy_from_slice(&root_header.instance.0.to_le_bytes());
    middle_pointer[POINTER_BIRTH_SEQUENCE_OFFSET..POINTER_BIRTH_SEQUENCE_OFFSET + 4]
        .copy_from_slice(&middle_birth_sequence.0.to_le_bytes());
    let (birth_tree, container, container_birth) = identity_segments;
    let mut entry_to_the_middle = leaf_entry[..INODE_ENTRY_CHILD_POINTER_OFFSET].to_vec();
    entry_to_the_middle[INODE_ENTRY_BIRTH_TREE_OFFSET..INODE_ENTRY_BIRTH_TREE_OFFSET + 8]
        .copy_from_slice(&birth_tree.to_le_bytes());
    entry_to_the_middle[INODE_ENTRY_RECORD_TYPE_OFFSET..INODE_ENTRY_RECORD_TYPE_OFFSET + 2]
        .copy_from_slice(&INODE_ENTRY_TYPE_INTERNAL_NODE.to_le_bytes());
    entry_to_the_middle[INODE_ENTRY_CONTAINER_OFFSET..INODE_ENTRY_CONTAINER_OFFSET + 8]
        .copy_from_slice(&container.to_le_bytes());
    entry_to_the_middle[INODE_ENTRY_CONTAINER_BIRTH_OFFSET..INODE_ENTRY_CONTAINER_BIRTH_OFFSET + 8]
        .copy_from_slice(&container_birth.to_le_bytes());
    entry_to_the_middle.extend_from_slice(&middle_pointer);
    assert_eq!(entry_to_the_middle.len(), usize::from(INODE_ENTRY_BYTES));
    let grown_root = rebuilt_index_node(
        &root_header,
        2,
        &root_header.smallest_key,
        &root_header.largest_key,
        root_header.birth_sequence,
        &[entry_to_the_middle],
    );
    replace_unit(
        &mut image,
        &units_found_on_device_zero(&pool.memory_pool()),
        inode_root.slot.0,
        &grown_root,
    );
    account_for_units_placed_by_hand(&mut image, &pool.output, 1, None);
    image
}

/// 合法的三层 inode 树：根的类型段 0 条目（三段为 0）指着一个码 2 的层级 1 节点，它下面是那片叶容器。
/// I-9.2 逐字「类型段 ∈ {0, 2}；类型段 0 ⇒ 子单元偏移 6 = 2 且三段为 0」：这条条目合法，一条都不许红，
/// 叶容器那几条（I-9.4 / I-9.7 / I-9.12 / I-9.13 / I-9.15）照样判到。改之前 checker 只认类型段 2：I-9.2 误红，
/// 接着把码 2 节点当码 3 读（I-1.3 / I-1.6 误红），叶从不被走。
#[test]
fn a_legal_type_zero_inode_entry_is_walked_down_to_the_leaf_and_every_invariant_holds() {
    let pool = product_capacity_pool();
    let image = image_with_the_inode_tree_grown_to_three_levels(&pool, (0, 0, 0));
    let red = violations(&image);
    assert_eq!(red, Vec::new(), "合法的三层 inode 树一条都不许红：{red:?}");
    for invariant in ["I-9.2", "I-9.4", "I-9.12", "I-9.13", "I-9.15"] {
        assert_eq!(
            verdict_of(&image, invariant),
            InvariantVerdict::Holds,
            "{invariant} 要走到叶、真被评估过且成立"
        );
    }
}

/// 同一棵三层树，类型段 0 那条条目的容器号写成 7（条款要三段为 0）：只红 I-9.2。
#[test]
fn a_type_zero_inode_entry_whose_container_segment_is_not_zero_reddens_only_the_entry_identity_invariant(
) {
    let pool = product_capacity_pool();
    let image = image_with_the_inode_tree_grown_to_three_levels(&pool, (0, 7, 0));
    let red = violations(&image);
    assert_eq!(violated_names(&red), ["I-9.2"], "只该红 I-9.2：{red:?}");
}

// ─── ⑤ I-9.4（容器号不超最小 key） 三句都判 ───

/// 第一个文件版本那片叶容器 C1（容器号 1）改成装 inode 1 与 10 两条记录；新放一片 C2（容器号 `second_container`，装 inode 20 与 30），
/// 根里加一条分隔 key 15 的类型段 2 条目指它。两片占的槽记进记账，inode 号水位抬到 31。
/// I-9.12（分隔 key 落在孩子区间之外） 在这几份上都成立：15 ≤ 20、15 > 1、15 > 10。
fn image_with_two_leaf_containers(pool: &TreeSplitPool, second_container: u64) -> MemoryPool {
    let mut image = pool.memory_pool();
    let leaf_role = TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
    let leaf = pool.output.unit(leaf_role);
    let record_count = u16::from_le_bytes([
        leaf.bytes[PACKED_RECORD_COUNT_OFFSET],
        leaf.bytes[PACKED_RECORD_COUNT_OFFSET + 1],
    ]);
    assert_eq!(record_count, 1, "第一个文件版本的叶容器装一条记录");
    assert_eq!(
        read_u64_at(&leaf.bytes, PACKED_CONTAINER_OFFSET),
        1,
        "那片叶的容器号是 1"
    );
    let first_record =
        leaf.bytes[PACKED_RECORDS_START..PACKED_RECORDS_START + INODE_RECORD_BYTES].to_vec();
    let record_of_inode = |inode: u64| {
        let mut record = first_record.clone();
        record[..8].copy_from_slice(&inode.to_le_bytes());
        record
    };
    let container_with = |container: u64, inodes: [u64; 2]| {
        let mut unit = leaf.bytes.clone();
        unit[PACKED_CONTAINER_OFFSET..PACKED_CONTAINER_OFFSET + 8]
            .copy_from_slice(&container.to_le_bytes());
        unit[PACKED_RECORD_COUNT_OFFSET..PACKED_RECORD_COUNT_OFFSET + 2]
            .copy_from_slice(&2u16.to_le_bytes());
        let declared = u16::try_from(2 * INODE_RECORD_BYTES).expect("声明长度");
        unit[PACKED_DECLARED_LENGTH_OFFSET..PACKED_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(&declared.to_le_bytes());
        for (index, inode) in inodes.into_iter().enumerate() {
            let start = PACKED_RECORDS_START + index * INODE_RECORD_BYTES;
            unit[start..start + INODE_RECORD_BYTES].copy_from_slice(&record_of_inode(inode));
        }
        reseal_unit_by_its_class(&mut unit);
        unit
    };
    let first_container = container_with(1, [1, 10]);
    let second_container_unit = container_with(second_container, [20, 30]);
    let units = units_found_on_device_zero(&image);
    replace_unit(&mut image, &units, leaf.slot.0, &first_container);
    write_unit_to_the_same_slot_on_both_devices(
        &mut image,
        SLOT_NOBODY_ALLOCATED,
        &second_container_unit,
    );
    let inode_root_slot = pool.output.unit(TransactionUnit::InodeRoot).slot.0;
    let inode_root = read_unit_on_device(&image, 0, inode_root_slot, node_bytes());
    let root_header = parse_index_node(&inode_root).expect("inode 根解得开");
    let mut entry_to_the_second = root_header.entries[0].clone();
    entry_to_the_second[..8].copy_from_slice(&15u64.to_le_bytes());
    entry_to_the_second[INODE_ENTRY_CONTAINER_OFFSET..INODE_ENTRY_CONTAINER_OFFSET + 8]
        .copy_from_slice(&second_container.to_le_bytes());
    for location in POINTER_LOCATION_ENTRY_OFFSETS {
        let start = INODE_ENTRY_CHILD_POINTER_OFFSET + location;
        entry_to_the_second
            [start + LOCATION_ENTRY_SLOT_OFFSET..start + LOCATION_ENTRY_SLOT_OFFSET + 6]
            .copy_from_slice(&SLOT_NOBODY_ALLOCATED.to_le_bytes()[..6]);
        entry_to_the_second
            [start + LOCATION_ENTRY_CHECKSUM_OFFSET..start + LOCATION_ENTRY_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&crc32_castagnoli(&second_container_unit).to_le_bytes());
    }
    let entries = vec![root_header.entries[0].clone(), entry_to_the_second];
    let grown_root = rebuilt_index_node(
        &root_header,
        root_header.level,
        &entries[0][..8],
        &entries[1][..8],
        root_header.birth_sequence,
        &entries,
    );
    replace_unit(&mut image, &units, inode_root_slot, &grown_root);
    account_for_units_placed_by_hand(&mut image, &pool.output, 2, Some(31));
    image
}

/// 阳性对照：第二片的容器号 11（> 左边最大 key 10、> 左边容器号 1）：三句都成立，一条都不红，I-9.4 真被评估过。
#[test]
fn two_leaf_containers_in_key_order_hold_every_invariant() {
    let pool = product_capacity_pool();
    let image = image_with_two_leaf_containers(&pool, 11);
    let red = violations(&image);
    assert_eq!(red, Vec::new(), "两片叶按序排好，一条都不许红：{red:?}");
    assert_eq!(verdict_of(&image, "I-9.4"), InvariantVerdict::Holds);
}

/// 第二片的容器号 8：它装的 20、30 都 ≥ 8（第一句成立），容器号 1 < 8（第二句成立），而左边容器的最大 key 10 不小于右边的容器号 8
/// （第三句不成立）——这正是审阅那一例：I-9.12 的两条不等式都过，只有 I-9.4 第三句说话。改之前 checker 只判第一句，全绿。
#[test]
fn a_right_container_number_not_above_the_largest_key_on_its_left_reddens_only_the_container_number_invariant(
) {
    let pool = product_capacity_pool();
    let image = image_with_two_leaf_containers(&pool, 8);
    let red = violations(&image);
    assert_eq!(violated_names(&red), ["I-9.4"], "只该红 I-9.4：{red:?}");
    assert!(
        red[0].1.contains("最大 key"),
        "红在「左容器的最大 key < 右容器号」那一句：{}",
        red[0].1
    );
}

/// 第二片的容器号 0：沿叶序容器号 1 → 0 往回走（第二句不成立；第三句跟着不成立）。违例说明取第一处，要是第二句那一格。
#[test]
fn container_numbers_going_backwards_along_the_leaf_order_redden_only_the_container_number_invariant(
) {
    let pool = product_capacity_pool();
    let image = image_with_two_leaf_containers(&pool, 0);
    let red = violations(&image);
    assert_eq!(violated_names(&red), ["I-9.4"], "只该红 I-9.4：{red:?}");
    assert!(
        red[0].1.contains("严格递增"),
        "第一处违例是「沿叶序容器号严格递增」那一句：{}",
        red[0].1
    );
}

// ─── ⑥ I-3.9 / I-5.4 读根记录直接持有的那棵分配记录树（C512） ───

/// 两块内存盘上 mkfs → 可写挂载（取号 1、不写行、零单元发布 txg 1、暖机 txg 2）→ 进程退出 → 再可写挂载（取号 2、写行 (1, 2, 0)：
/// 写行那次发布 txg 3 重写实例表，被换下的 mkfs 实例表记在根记录直接持有的那棵分配记录树里、带已释放标志、释放代 3；暖机 txg 4）。
/// 与 `second_transaction_step_three_formatted_pool.rs` 那条零故障历史的前两次挂载同一段。
fn image_after_the_row_publish_of_a_version_without_file() -> MemoryPool {
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = [0u32, 1]
        .into_iter()
        .map(|device| {
            (
                DeviceIdentity(device),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let first_mount = mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    assert!(
        first_mount.output.rows_written.is_empty(),
        "上一个实例是 0：不写行"
    );
    drop(first_mount);
    let image_after_the_first_mount = memory_pool_of_sparse_devices(&devices);
    let mut reopened = crash_state_devices(&image_after_the_first_mount, &[], &[], &stream);
    let second_mount = mount_writable(&parameters(), &mut reopened).expect("第二次可写挂载");
    assert_eq!(
        second_mount.output.rows_written.len(),
        1,
        "上一个实例 1 那一行"
    );
    drop(second_mount);
    memory_pool_of_sparse_devices(&reopened)
}

/// 单元区里盘 0 上的分配记录树叶（树 ID 0、key 宽 10、层级 0）逐片按 `change` 改（交回这一片改了几条），改过的重建、
/// 新整单元校验和沿引用链（第 1 层节点、根、根记录里那条指针）补上。交回一共改了几条。
fn change_allocation_record_leaves(
    image: &mut MemoryPool,
    change: impl Fn(&mut Vec<Vec<u8>>) -> usize,
) -> usize {
    let units = units_found_on_device_zero(image);
    let mut changed_records = 0;
    for (slot, length) in units.iter().copied() {
        if length != node_bytes() {
            continue;
        }
        let bytes = read_unit_on_device(image, 0, slot, length);
        let Ok(header) = parse_index_node(&bytes) else {
            continue;
        };
        if header.tree.0 != 0
            || header.key_width != ALLOCATION_RECORD_TREE_KEY_WIDTH
            || header.level != 0
        {
            continue;
        }
        assert_the_rebuild_reproduces(&bytes, &header);
        let mut entries = header.entries.clone();
        let changed_here = change(&mut entries);
        if changed_here == 0 {
            continue;
        }
        changed_records += changed_here;
        let rebuilt = rebuilt_index_node(
            &header,
            header.level,
            &header.smallest_key,
            &header.largest_key,
            header.birth_sequence,
            &entries,
        );
        replace_unit(image, &units, slot, &rebuilt);
    }
    changed_records
}

fn allocation_record_is_released(record: &[u8]) -> bool {
    u16::from_le_bytes([
        record[ALLOCATION_RECORD_SPAN_OFFSET],
        record[ALLOCATION_RECORD_SPAN_OFFSET + 1],
    ]) & ALLOCATION_RECORD_RELEASED_FLAG
        != 0
}

/// 阳性对照与坏镜像：树表 0 条、写过行的那一版上，被换下的 mkfs 实例表记在根记录直接持有的那棵分配记录树里（已释放、释放代 3）。
/// 干净镜像上 I-3.9（释放代落在停止引用它的那一格区间里） 真被评估过且成立（还引用它的最新一条有效根是 txg 2，最早不再引用它的是 txg 3）；
/// 把释放代写成 2 ⇒ 只红 I-3.9。改之前 checker 只读树表里种类 3 那一棵，这一版上两份镜像 I-3.9 都报不适用。
#[test]
fn a_released_record_in_the_allocation_record_tree_the_root_record_holds_is_judged_by_the_release_generation_invariant(
) {
    let clean = image_after_the_row_publish_of_a_version_without_file();
    assert_eq!(violations(&clean), Vec::new(), "写行之后的干净镜像全绿");
    assert_eq!(
        verdict_of(&clean, "I-3.9"),
        InvariantVerdict::Holds,
        "根记录持有的那棵树里的已释放记录要真被评估过且成立"
    );
    let mut image = clean.clone();
    let rewritten = change_allocation_record_leaves(&mut image, |records| {
        let mut rewritten_here = 0;
        for record in records.iter_mut() {
            if allocation_record_is_released(record)
                && read_six_byte_slot_at(record, ALLOCATION_RECORD_SLOT_OFFSET)
                    == INSTANCE_TABLE_SLOT.0
            {
                assert_eq!(
                    read_u64_at(record, ALLOCATION_RECORD_GENERATION_OFFSET),
                    3,
                    "换下的 mkfs 实例表释放代本来是 3"
                );
                record
                    [ALLOCATION_RECORD_GENERATION_OFFSET..ALLOCATION_RECORD_GENERATION_OFFSET + 8]
                    .copy_from_slice(&2u64.to_le_bytes());
                rewritten_here += 1;
            }
        }
        rewritten_here
    });
    assert!(rewritten >= 2, "两块盘各一条已释放记录（两片叶各一条）");
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        ["I-3.9"],
        "释放代 2 不在 (2, 3] 里，只该红 I-3.9：{red:?}"
    );
}

/// 同一份写行之后的镜像：根记录直接持有的那棵分配记录树里，盘 0 那片叶上 mkfs 树表那条记录（槽 50178、跨 1）的跨度改宽到罩住下一条记录的起点槽
/// ⇒ 只红 I-5.4（分配记录罩住的槽互不相交）。改之前 checker 只读树表里种类 3 那一棵，这一版上报不适用。
#[test]
fn overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant(
) {
    const GENESIS_TREE_TABLE_SLOT: u64 = 50_178;
    let mut image = image_after_the_row_publish_of_a_version_without_file();
    let widened = change_allocation_record_leaves(&mut image, |records| {
        let on_device_zero = |record: &Vec<u8>| read_u32_at(record, 0) == 0;
        let Some(next_start) = records
            .iter()
            .filter(|record| on_device_zero(record))
            .map(|record| read_six_byte_slot_at(record, ALLOCATION_RECORD_SLOT_OFFSET))
            .filter(|slot| *slot > GENESIS_TREE_TABLE_SLOT)
            .min()
        else {
            return 0;
        };
        let mut widened_here = 0;
        for record in records.iter_mut() {
            if on_device_zero(record)
                && read_six_byte_slot_at(record, ALLOCATION_RECORD_SLOT_OFFSET)
                    == GENESIS_TREE_TABLE_SLOT
            {
                let span = u16::try_from(next_start - GENESIS_TREE_TABLE_SLOT + 1).expect("跨度");
                record[ALLOCATION_RECORD_SPAN_OFFSET..ALLOCATION_RECORD_SPAN_OFFSET + 2]
                    .copy_from_slice(&span.to_le_bytes());
                widened_here += 1;
            }
        }
        widened_here
    });
    assert!(widened >= 1, "盘 0 那片叶上有 mkfs 树表那条记录");
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        ["I-5.4"],
        "两条记录罩住同一个槽，只该红 I-5.4：{red:?}"
    );
}

// ─── ⑦ I-7.7（系统配置实例代号不低于根环） 一个槽读不出时 ① ② 分开报 ───

/// 写完第一个事务的镜像上放一份写序实例代号 2 的孤儿数据单元（没人引用的槽 50302，头与载荷校验和都过）；`raise_disk_zero` 为真时
/// 盘 0 较新那个系统配置槽的实例代号改成 2（各盘最大号 [2, 1] 不等）。与 `checker_known_bad_images.rs` 那两份 I-7.7 坏镜像同形。
fn image_with_an_orphan_of_instance_two(raise_disk_zero: bool) -> MemoryPool {
    const ORPHAN_SLOT: u64 = 50_302;
    const SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET: u64 = 4096;
    const SYSTEM_CONFIGURATION_INSTANCE_OFFSET: usize = 477;
    const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
    const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
    let pool = build_pool(if raise_disk_zero {
        "b2-instance-carriers-second-sentence"
    } else {
        "b2-instance-carriers-first-sentence"
    });
    let mut image = pool.memory_pool();
    let data_unit = pool
        .output
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let mut orphan = data_unit.bytes.clone();
    orphan[91..95].copy_from_slice(&2u32.to_le_bytes());
    reseal_unit_by_its_class(&mut orphan);
    write_bytes(&mut image, 0, ORPHAN_SLOT * SLOT_BYTES, &orphan);
    if raise_disk_zero {
        let mut slot = read_bytes(
            &image,
            0,
            SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET,
            SYSTEM_CONFIGURATION_SLOT_BYTES,
        );
        slot[SYSTEM_CONFIGURATION_INSTANCE_OFFSET..SYSTEM_CONFIGURATION_INSTANCE_OFFSET + 4]
            .copy_from_slice(&2u32.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            SYSTEM_CONFIGURATION_SLOT_BYTES,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        write_bytes(&mut image, 0, SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET, &slot);
    }
    image
}

/// ② 那一形（各盘最大号 [2, 1] 不等、孤儿带着较大的 2）再加根环一个空槽读不出：① 按条款报不适用，② 在读得出的载体上照判、找到就红。
/// 改之前两半捆在一起，一个槽读不出就整条报不适用，② 的违例看不见。
#[test]
fn the_second_sentence_of_the_instance_carrier_invariant_is_judged_on_the_readable_carriers_when_a_slot_is_unreadable(
) {
    let image = image_with_an_orphan_of_instance_two(true);
    let (device, offset) = an_empty_root_ring_slot(&image);
    let with_an_unreadable_slot = ImageWithAnUnreadableRange {
        image: &image,
        device,
        unreadable_offsets: offset..offset + 512,
    };
    let detail = detail_of_the_violation(&with_an_unreadable_slot, "I-7.7");
    assert!(detail.contains("（②）"), "红在第二句上：{detail}");
}

/// ① 那一形（两盘都还是实例 1，孤儿带 2）再加根环一个空槽读不出：条款逐字「任一根环区域、journal 记录槽、单元头读不出时这一半报不适用」，
/// ② 这里没有对象（各盘相等）——整条报不适用，不报红也不报成立。读得全的时候这份照红（阳性对照）。
#[test]
fn the_first_sentence_of_the_instance_carrier_invariant_is_not_judged_when_a_slot_is_unreadable() {
    let image = image_with_an_orphan_of_instance_two(false);
    assert!(
        matches!(verdict_of(&image, "I-7.7"), InvariantVerdict::Violated(_)),
        "读得全的时候 ① 红"
    );
    let (device, offset) = an_empty_root_ring_slot(&image);
    let with_an_unreadable_slot = ImageWithAnUnreadableRange {
        image: &image,
        device,
        unreadable_offsets: offset..offset + 512,
    };
    assert!(
        matches!(
            verdict_of(&with_an_unreadable_slot, "I-7.7"),
            InvariantVerdict::NotApplicable(_)
        ),
        "有槽读不出：① 报不适用、② 没有对象，整条不适用"
    );
}

/// 槽距 8192（io_min 8192）的池，mkfs 之后盘 0 的系统配置槽 0 翻一个只有整槽校验和罩着的保留字节（读得出、自证不过）：
/// 盘 1 的槽 0 自证过、记着槽距 8192，盘 0 的槽 1 就在 8192。checker 照恢复（实审 A2a 第 37 条改的 `choose_system_configuration`）
/// 借池里第一块槽 0 有效的盘记的槽距去找，找得到盘 0 的槽 1。改之前按最小槽距 4096 找，盘 0 一个自证过的槽都没有。
#[test]
fn a_device_whose_slot_zero_does_not_verify_has_its_slot_one_looked_for_at_the_spacing_another_device_records(
) {
    const SPACING: u32 = 8192;
    const BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT: usize = 2048;
    let mut make_filesystem_parameters = parameters();
    make_filesystem_parameters
        .geometry
        .minimum_input_output_bytes = SPACING;
    make_filesystem_parameters
        .geometry
        .fixed_structure_slot_spacing = SPACING;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
        .into_iter()
        .map(|device| {
            (
                DeviceIdentity(device),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    make_filesystem(&make_filesystem_parameters, &mut devices).expect("槽距 8192 的池");
    let mut image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    };
    let mut slot_zero = read_bytes(&image, 0, 0, 4096);
    slot_zero[BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT] ^= 0xff;
    write_bytes(&mut image, 0, 0, &slot_zero);
    let verified = singlefs_checker::image::verified_system_configuration_slots(&image);
    let verified_slot_count_of = |device: u32| {
        verified
            .iter()
            .find(|(identity, _)| *identity == device)
            .map_or(0, |(_, slots)| slots.len())
    };
    assert_eq!(
        (verified_slot_count_of(0), verified_slot_count_of(1)),
        (1, 2),
        "盘 0 的槽 1 在盘 1 记的槽距 8192 处找到；盘 1 两槽都自证过"
    );
    assert_eq!(
        verdict_of(&image, "I-7.7"),
        InvariantVerdict::Holds,
        "每块盘都交得出本池的系统配置槽，I-7.7 判得了且成立"
    );
}

// ─── ⑧ 新立的「中央映射条目的 key 与它指的单元头相符」（I-1.11（映射 key 与单元头相符）） ───

/// 映射 key 的字段序（类标签, 出生树, 出生 txg, 实例代号, 尾段 6 字节），按字段比无符号整数（D8（核心索引结构） 已定项 11）。
fn mapping_key_order(entry: &[u8]) -> (u8, u64, u64, u32, u64) {
    (
        entry[MAPPING_KEY_CLASS_OFFSET],
        read_u64_at(entry, MAPPING_KEY_BIRTH_TREE_OFFSET),
        read_u64_at(entry, MAPPING_KEY_BIRTH_TREE_OFFSET + 8),
        read_u32_at(entry, MAPPING_KEY_TAIL_OFFSET - 4),
        read_six_byte_slot_at(entry, MAPPING_KEY_TAIL_OFFSET),
    )
}

/// 第一个文件版本的中央映射树（产品容量下是根兼叶）里的条目按 `change` 改，改完按 key 重新排序、头里的区间贴紧首末条目，重建、补校验和。
fn image_with_the_mapping_entries_changed(
    pool: &TreeSplitPool,
    change: impl Fn(&mut Vec<Vec<u8>>),
) -> MemoryPool {
    let mapping = pool.output.unit(TransactionUnit::MappingTree);
    let header = parse_index_node(&mapping.bytes).expect("映射根解得开");
    assert_the_rebuild_reproduces(&mapping.bytes, &header);
    assert_eq!(header.level, 0, "产品容量下第一个文件版本的映射树是根兼叶");
    let mut entries = header.entries.clone();
    change(&mut entries);
    entries.sort_by_key(|entry| mapping_key_order(entry));
    let smallest_key = entries.first().expect("有条目")[..MAPPING_KEY_BYTES].to_vec();
    let largest_key = entries.last().expect("有条目")[..MAPPING_KEY_BYTES].to_vec();
    let damaged = rebuilt_index_node(
        &header,
        header.level,
        &smallest_key,
        &largest_key,
        header.birth_sequence,
        &entries,
    );
    let mut image = pool.memory_pool();
    let units = units_found_on_device_zero(&image);
    replace_unit(&mut image, &units, mapping.slot.0, &damaged);
    image
}

fn first_entry_of_class(entries: &mut [Vec<u8>], unit_class: u8) -> &mut Vec<u8> {
    entries
        .iter_mut()
        .find(|entry| entry[MAPPING_KEY_CLASS_OFFSET] == unit_class)
        .expect("这一类有映射条目")
}

/// inode 叶容器（码 3）那条映射条目的 key 尾段后面那 2 字节补零写成 1：条款「码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0」。
/// 改之前 checker 只读尾段前 4 字节，全绿。
#[test]
fn a_mapping_key_whose_padding_after_the_birth_sequence_is_not_zero_reddens_only_the_mapping_key_invariant(
) {
    let pool = product_capacity_pool();
    let image = image_with_the_mapping_entries_changed(&pool, |entries| {
        first_entry_of_class(entries, UNIT_CLASS_PACKED)[MAPPING_KEY_PADDING_OFFSET] = 1;
    });
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        [MAPPING_KEY_INVARIANT],
        "只该红映射 key 那一条：{red:?}"
    );
    assert!(red[0].1.contains("补零"), "说明要点出补零：{}", red[0].1);
}

/// 数据单元（码 1）与 inode 叶容器（码 3）那条映射条目 key 里的出生树加一：与被指单元头里偏移 43 的出生树不再相等。改之前全绿。
#[test]
fn a_mapping_key_whose_birth_tree_is_not_the_one_in_the_unit_header_reddens_only_the_mapping_key_invariant(
) {
    let pool = product_capacity_pool();
    for unit_class in [UNIT_CLASS_DATA, UNIT_CLASS_PACKED] {
        let image = image_with_the_mapping_entries_changed(&pool, |entries| {
            let entry = first_entry_of_class(entries, unit_class);
            let birth_tree = read_u64_at(entry, MAPPING_KEY_BIRTH_TREE_OFFSET) + 1;
            entry[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TREE_OFFSET + 8]
                .copy_from_slice(&birth_tree.to_le_bytes());
        });
        let red = violations(&image);
        assert_eq!(
            violated_names(&red),
            [MAPPING_KEY_INVARIANT],
            "类标签 {unit_class}：只该红映射 key 那一条：{red:?}"
        );
        assert!(
            red[0].1.contains("出生树"),
            "说明要点出出生树：{}",
            red[0].1
        );
    }
}

/// 码 2 节点那条映射条目 key 里的出生树加一：码 2 的出生树取哪个字段还开着（C289（码 2 的出生树取哪个字段没写）），今天不判，一条都不红。
#[test]
fn the_birth_tree_in_a_mapping_key_of_an_index_node_is_not_judged_while_its_field_is_open() {
    let pool = product_capacity_pool();
    let image = image_with_the_mapping_entries_changed(&pool, |entries| {
        let entry = first_entry_of_class(entries, UNIT_CLASS_INDEX_NODE);
        let birth_tree = read_u64_at(entry, MAPPING_KEY_BIRTH_TREE_OFFSET) + 1;
        entry[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TREE_OFFSET + 8]
            .copy_from_slice(&birth_tree.to_le_bytes());
    });
    assert_eq!(
        violations(&image),
        Vec::new(),
        "码 2 的出生树随 C289 定，今天不判"
    );
}

// ─── ⑨ journal 计数器 0 判 I-8.6（反向链算法） 违例 ───

/// journal 环里一条记录的计数器改成 0（头校验和重封，两盘一起改）：计数器从 1 起（D23（journal 的角色与格式） 已定项 18：
/// 记录 n 落在 `(计数器 − 1) mod 槽数`），0 这一格没有「本实例内逻辑前一条」可言、链无从定义 ⇒ 用户 2026-09-27 定判违例，记在 I-8.6 下。
/// 改之前（实审 B1）判不了、跳过，I-8.6 由计数器 1 那一条判成立。
#[test]
fn a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant() {
    let pool = product_capacity_pool();
    let mut image = pool.memory_pool();
    let offset = singlefs_core::journal::record_offset(2, JOURNAL_RING_DEFAULT_BYTES).0;
    for device in [0u32, 1] {
        let mut record = read_bytes(&image, device, offset, JOURNAL_RECORD_BYTES);
        assert_eq!(
            record[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6],
            2u64.to_le_bytes()[..6],
            "计数器 2 那条记录在它的环槽上"
        );
        record[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6].fill(0);
        reseal_journal_record(&mut record);
        write_bytes(&mut image, device, offset, &record);
    }
    let detail = detail_of_the_violation(&image, "I-8.6");
    assert!(
        detail.contains("计数器 0"),
        "红在计数器 0 那一条上：{detail}"
    );
}
