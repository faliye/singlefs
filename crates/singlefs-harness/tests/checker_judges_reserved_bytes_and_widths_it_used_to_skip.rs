//! 池级 checker 与实现同一处漏判的几样（代码审阅第 29、38 条落在 checker 这一侧的那一半；实现那一半是实审 A3a）。
//! checker 与实现分开写（D13（验证路线） 已定项 5），这几条判定 checker 自己判，用例也只调 checker 的函数：
//! ① 单元头偏移 4 的格式版本认得、类身份段之后那 29 字节 nonce / MAC / 算法类型预留位恒 0（I-2.4（头校验和覆盖范围）
//!    那一行「恒 0、读者遇到非 0 一律判该结构损坏」）；
//! ② 码 2 条目宽 0 而条目数非 0、码 3 记录宽 0 而记录数非 0（声明长度 0 过得了「= 条数 × 宽」，切出来一条都没有）；
//! ③ 系统配置的格式版本、加密类型（偏移 219）、固定结构槽距、`physical_block_size`、journal 环长的上下界。
//! 每条都从实现写出来的一份真单元 / 真系统配置槽出发，改一处、按类重封（校验和都对），改之前的 checker 照收
//! （三类单元照交类标签、宽 0 的节点交出 0 条、改坏的系统配置槽照交几何，池级报告里一条违例都没有）。
//!
//! 实审 A3-checker-2 把这几样接进池级走读（`check_pool_image`）：
//! ④ 三种码的单元头在走读里判格式版本与 29 字节预留位，报 I-2.4（改之前码 1 / 码 3 不判，码 2 只经解码失败报成 I-7.2）；
//! ⑤ 走读跟随的每条指针，头部 MAC 16 + nonce 12 全 0，报 I-2.4（D19（块指针的结构与宽度预算） 已定项 3 射程；改之前不判）；
//! ⑥ 登记树的码 2 节点条目宽 0 而条目数非 0，报 I-1.10（改之前只记走读失败）；
//! ⑦ 系统配置只有一部分槽带越界值时报违例、整池不作保（改之前那一槽不可择，拿别的槽照判，一条违例都没有）。
//!
//! 偏移都按字段表写死在用例里，不从 checker 或实现里取：取了的话，两边把字段挪到别处这几条也跟着走。

mod common;
mod common_tree_split;

use common_tree_split::{
    capacities, propagate_the_new_checksum, reseal_unit_by_its_class, root_ring_slot_offsets,
    write_unit_to_the_same_slot_on_both_devices, TreeSplitPool, ACCOUNTING_OF_THE_NODE_FORMAT,
    CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
};
use singlefs_checker::image::{
    geometry_of, InvariantVerdict, SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker::{
    check_system_configuration_slot, check_unit, index_node_view, packed_unit_view, Verdict,
};
use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::transaction::{PublishedUnit, TransactionUnit};
use singlefs_harness::memory_pool::MemoryPool;

/// 单元头共同前缀里格式版本那 2 字节（magic 4 之后，D18（块里携带什么信息） 已定项 7）；系统配置自举头同一个偏移。
const FORMAT_VERSION_OFFSET: usize = 4;
/// nonce 12 + MAC 16 + 算法类型 1（D18（块里携带什么信息） 已定项 16）。
const ENCRYPTION_RESERVED_BYTES: usize = 29;
/// 三类单元预留位的起点 = 明文头末尾（D18（块里携带什么信息） 已定项 18 的带偏移字段表）：码 1 在 105、码 3 在 107、
/// 码 2 在 86 + 2 × key 宽；inode 树的 key 宽是 8（inode 号），所以 inode 树根的在 102。
const DATA_UNIT_RESERVED_START: usize = 105;
const INODE_TREE_KEY_WIDTH: u8 = 8;
const INODE_TREE_ROOT_RESERVED_START: usize = 86 + 2 * 8;
const PACKED_UNIT_RESERVED_START: usize = 107;
/// 码 2 头里 key 宽那 1 字节（D18（块里携带什么信息） 已定项 18：固定偏移 51）。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
/// 码 3 头（D18（块里携带什么信息） 已定项 11）：声明长度 8、记录数 69、记录宽 71。
const DECLARED_LENGTH_OFFSET: usize = 8;
const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
const PACKED_UNIT_RECORD_WIDTH_OFFSET: usize = 71;

/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：加密类型 219、`physical_block_size` 317、journal 环起点 325 / 环长 333、
/// 根环起点 371、单元区起始槽号 417、固定结构槽距 429；整槽校验和 32 字节在 155、罩整槽 4096。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize = 219;
const SYSTEM_CONFIGURATION_PHYSICAL_BLOCK_SIZE_OFFSET: usize = 317;
const SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET: usize = 333;
const SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET: usize = 429;
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
/// 每盘两个系统配置槽：槽 0 在 0、槽 1 在槽距处（`common::parameters` 的槽距 4096）。
const SYSTEM_CONFIGURATION_SLOT_OFFSETS: [u64; 2] = [0, 4096];
/// `common::parameters` 的几何写进系统配置的几样：槽距 4096、根环起点槽 64（1 MiB）、journal 环起点槽 1024、
/// 单元区起始槽 50176、环长 768 MiB。环末端 1024 × 16384 + 768 MiB 恰好等于单元区起点 50176 × 16384。
const SLOT_SPACING_OF_THE_POOL: u32 = 4096;
const ROOT_RING_BASE_OFFSET_OF_THE_POOL: u32 = 64 * 16384;
const JOURNAL_RING_BYTES_OF_THE_POOL: u64 = 805_306_368;
/// 根记录宽（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_RECORD_BYTES: u32 = 457;
/// 在飞记录数上限 = 环槽数 ÷ F，F = 3（D23（journal 的角色与格式） 已定项 18）：环长至少 3 条 4096 字节的记录。
const SMALLEST_JOURNAL_RING_BYTES_HOLDING_THE_SAFETY_FACTOR_RECORDS: u64 = 3 * 4096;

fn first_file_version_pool() -> TreeSplitPool {
    TreeSplitPool::with_the_first_file_version_under(capacities(
        ACCOUNTING_OF_THE_NODE_FORMAT,
        CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
    ))
}

/// 一个单元改一个字节再按类重封（载荷 CRC 与头校验和都重算）：两道校验和都对得上，剩下的只有这一个字节的值。
fn resealed_with_byte(original: &[u8], offset: usize, value: u8) -> Vec<u8> {
    let mut unit = original.to_vec();
    unit[offset] = value;
    reseal_unit_by_its_class(&mut unit);
    unit
}

fn resealed_with_format_version(original: &[u8], format_version: u16) -> Vec<u8> {
    let mut unit = original.to_vec();
    unit[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        .copy_from_slice(&format_version.to_le_bytes());
    reseal_unit_by_its_class(&mut unit);
    unit
}

/// 第一个文件版本里三类单元各一个（数据单元、inode 树根、inode 叶容器），带它的类标签与预留位起点。
fn one_unit_of_each_class(pool: &TreeSplitPool) -> Vec<(&'static str, Vec<u8>, u8, usize)> {
    let inode_root = pool.output.unit(TransactionUnit::InodeRoot).bytes.clone();
    assert_eq!(
        inode_root[INDEX_NODE_KEY_WIDTH_OFFSET], INODE_TREE_KEY_WIDTH,
        "inode 树根的 key 宽是 8，预留位起点 102 按它算"
    );
    vec![
        (
            "数据单元（码 1）",
            pool.output
                .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
                .bytes
                .clone(),
            1,
            DATA_UNIT_RESERVED_START,
        ),
        (
            "inode 树根（码 2）",
            inode_root,
            2,
            INODE_TREE_ROOT_RESERVED_START,
        ),
        (
            "inode 叶容器（码 3）",
            pool.output
                .unit(TransactionUnit::InodeLeafContainer(
                    InodeLeafContainerIndexInTree::LEFTMOST,
                ))
                .bytes
                .clone(),
            3,
            PACKED_UNIT_RESERVED_START,
        ),
    ]
}

#[test]
fn a_unit_whose_format_version_is_not_the_one_this_checker_reads_is_refused_once_both_checksums_hold(
) {
    let pool = first_file_version_pool();
    for (what, original, unit_class, _) in one_unit_of_each_class(&pool) {
        assert_eq!(
            check_unit(&original),
            Ok(unit_class),
            "{what}：实现写出来的原样先要过"
        );
        for format_version in [0u16, 2, u16::MAX] {
            assert_eq!(
                check_unit(&resealed_with_format_version(&original, format_version)),
                Err(Verdict::FormatVersionNotRecognized),
                "{what}：格式版本 {format_version}（两道校验和都对）要拒，不按今天的字段表往下解"
            );
        }
        let mut damaged_version = original.clone();
        damaged_version[FORMAT_VERSION_OFFSET] ^= 1;
        assert_eq!(
            check_unit(&damaged_version),
            Err(Verdict::ChecksumMismatch),
            "{what}：版本字节坏了而没重封的，仍报头校验和对不上——版本判在两道校验和之后"
        );
    }
}

#[test]
fn a_unit_with_a_non_zero_byte_among_the_twenty_nine_encryption_reserved_bytes_is_refused() {
    let pool = first_file_version_pool();
    for (what, original, unit_class, reserved_start) in one_unit_of_each_class(&pool) {
        assert!(
            original[reserved_start..reserved_start + ENCRYPTION_RESERVED_BYTES]
                .iter()
                .all(|byte| *byte == 0),
            "{what}：实现写出来的预留位全 0"
        );
        // nonce 的首尾、MAC 的首尾、算法类型：29 字节的两端与三段的分界各取一个。
        for position_in_reserved_bytes in [0usize, 11, 12, 27, 28] {
            assert_eq!(
                check_unit(&resealed_with_byte(
                    &original,
                    reserved_start + position_in_reserved_bytes,
                    1
                )),
                Err(Verdict::EncryptionReservedBytesNotZero),
                "{what}：预留位第 {position_in_reserved_bytes} 字节是 1（两道校验和都对）要拒"
            );
        }
        let first_byte_after_the_reserved_bytes = reserved_start + ENCRYPTION_RESERVED_BYTES;
        let changed_after_the_reserved_bytes = resealed_with_byte(
            &original,
            first_byte_after_the_reserved_bytes,
            original[first_byte_after_the_reserved_bytes] ^ 1,
        );
        assert_eq!(
            check_unit(&changed_after_the_reserved_bytes),
            Ok(unit_class),
            "{what}：预留位只有 29 字节，紧接着的那一字节是载荷，check_unit 不管它的值"
        );
    }
}

/// 池级：最新根下 inode 树根的预留位改成非 0、校验和沿引用链补到根槽。改之前池级 checker 一条违例都不报
/// （走读判单元头用的是自己那一份 I-2.4，只看头校验和；节点解码 `index_node_view` 经 `check_unit` 也不看预留位）。
/// 实审 A3-checker 之后 `index_node_view` 拒这个节点；实审 A3-checker-2 之后走读判单元头那一步就判它头用不了，
/// 最新根走不完，I-7.2（最新根可完整遍历） 红，断在「头用不了」那一步。
#[test]
fn an_inode_tree_root_with_a_non_zero_encryption_reserved_byte_breaks_the_walk_of_the_newest_root()
{
    let pool = first_file_version_pool();
    let image = pool.memory_pool();
    let clean_violations: Vec<&'static str> = check_pool_image(&image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| invariant)
        .collect();
    assert!(
        clean_violations.is_empty(),
        "改之前的镜像一条违例都没有：{clean_violations:?}"
    );
    let inode_root = pool.output.unit(TransactionUnit::InodeRoot);
    let damaged_root = resealed_with_byte(&inode_root.bytes, INODE_TREE_ROOT_RESERVED_START, 1);
    let mut damaged = image.clone();
    write_unit_to_the_same_slot_on_both_devices(&mut damaged, inode_root.slot.0, &damaged_root);
    let units_of_the_version: Vec<(u64, usize)> = pool
        .output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect();
    propagate_the_new_checksum(
        &mut damaged,
        &units_of_the_version,
        (
            inode_root.slot.0,
            crc32_castagnoli(&inode_root.bytes),
            crc32_castagnoli(&damaged_root),
        ),
    );
    let newest_root_walk = check_pool_image(&damaged)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-7.2")
        .expect("I-7.2 在第一版清单里")
        .1;
    match newest_root_walk {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("头用不了"),
            "最新根走不完，断在 inode 树根头用不了那一步：{detail}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
            panic!("inode 树根的预留位非 0，最新根不该算走得完：{other:?}")
        }
    }
}

/// 码 2 节点：key 宽 0、条目宽 0、条目数 3、声明长度 0（= 3 × 0）。按 key 宽 0 的偏移表（D18（块里携带什么信息） 已定项 18）
/// 手搭：52 诞生代号、60 fsid、68 写序、72 出生序号、76 载荷 CRC、80 预留 2、82 条目数、84 条目宽，明文头到 86。
fn index_node_with_zero_key_width_and_zero_entry_width(entry_count: u16) -> Vec<u8> {
    let mut node = vec![0u8; 16384];
    node[..4].copy_from_slice(b"SFSU");
    node[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2].copy_from_slice(&1u16.to_le_bytes());
    node[6] = 2;
    node[42..50].copy_from_slice(&12u64.to_le_bytes());
    node[INDEX_NODE_KEY_WIDTH_OFFSET] = 0;
    node[82..84].copy_from_slice(&entry_count.to_le_bytes());
    node[84..86].copy_from_slice(&0u16.to_le_bytes());
    reseal_unit_by_its_class(&mut node);
    node
}

#[test]
fn a_code_two_node_whose_entry_width_is_zero_while_its_entry_count_is_not_is_refused() {
    assert_eq!(
        index_node_view(&index_node_with_zero_key_width_and_zero_entry_width(3))
            .map(|view| (view.entries.len(), view.entry_width)),
        Err(Verdict::EntryWidthZeroWithEntries),
        "头里说 3 条、每条 0 字节：声明长度 0 过得了「= 条目数 × 条目宽」，但切不出 3 条，要拒"
    );
    let empty = index_node_view(&index_node_with_zero_key_width_and_zero_entry_width(0))
        .expect("条目数也是 0 的空节点照收：头里说 0 条，切出 0 条");
    assert!(empty.entries.is_empty(), "空节点交出 0 条");
}

/// 码 3 单元：实现写出来的 inode 叶容器，记录数改 3、记录宽改 0、声明长度改 0（= 3 × 0），按类重封。
fn packed_unit_with_zero_record_width(pool: &TreeSplitPool, record_count: u16) -> Vec<u8> {
    let mut unit = pool
        .output
        .unit(TransactionUnit::InodeLeafContainer(
            InodeLeafContainerIndexInTree::LEFTMOST,
        ))
        .bytes
        .clone();
    unit[DECLARED_LENGTH_OFFSET..DECLARED_LENGTH_OFFSET + 2].copy_from_slice(&0u16.to_le_bytes());
    unit[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
        .copy_from_slice(&record_count.to_le_bytes());
    unit[PACKED_UNIT_RECORD_WIDTH_OFFSET..PACKED_UNIT_RECORD_WIDTH_OFFSET + 2]
        .copy_from_slice(&0u16.to_le_bytes());
    reseal_unit_by_its_class(&mut unit);
    unit
}

#[test]
fn a_code_three_unit_whose_record_width_is_zero_while_its_record_count_is_not_is_refused() {
    let pool = first_file_version_pool();
    assert_eq!(
        packed_unit_view(&packed_unit_with_zero_record_width(&pool, 3))
            .map(|view| (view.records.len(), view.record_width)),
        Err(Verdict::RecordWidthZeroWithRecords),
        "头里说 3 条、每条 0 字节：切不出 3 条，要拒"
    );
    let empty = packed_unit_view(&packed_unit_with_zero_record_width(&pool, 0))
        .expect("记录数也是 0 的照收");
    assert!(empty.records.is_empty(), "交出 0 条");
}

/// 实现写出来的系统配置槽（盘 0 槽 0）改一处、重算整槽校验和。
fn system_configuration_slot_with(image: &MemoryPool, change: impl Fn(&mut [u8])) -> Vec<u8> {
    let mut slot = image
        .devices
        .get(&DeviceIdentity(0))
        .expect("盘 0")
        .read(DeviceOffsetInBytes(0), SYSTEM_CONFIGURATION_SLOT_BYTES);
    change(&mut slot);
    reseal_system_configuration_slot(&mut slot);
    slot
}

fn reseal_system_configuration_slot(slot: &mut [u8]) {
    let digest = wide_checksum_with_field_zeroed(
        slot,
        SYSTEM_CONFIGURATION_SLOT_BYTES,
        SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
    );
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
}

/// checker 读一槽系统配置的两步都走：自证与自举头（`check_system_configuration_slot`），再取几何（`geometry_of`）。
fn verdict_of_the_slot(slot: &[u8]) -> Result<(), Verdict> {
    let view = check_system_configuration_slot(slot)?;
    geometry_of(slot, &view).map(|_| ())
}

fn with_u32_at(offset: usize, value: u32) -> impl Fn(&mut [u8]) {
    move |slot: &mut [u8]| slot[offset..offset + 4].copy_from_slice(&value.to_le_bytes())
}

fn with_u64_at(offset: usize, value: u64) -> impl Fn(&mut [u8]) {
    move |slot: &mut [u8]| slot[offset..offset + 8].copy_from_slice(&value.to_le_bytes())
}

/// 对一个系统配置槽的一处改动（改完由调用方重算整槽校验和）。
type SystemConfigurationSlotChange = Box<dyn Fn(&mut [u8])>;

#[test]
fn a_system_configuration_slot_whose_format_version_is_not_the_one_this_checker_reads_is_refused() {
    let image = first_file_version_pool().memory_pool();
    assert_eq!(
        verdict_of_the_slot(&system_configuration_slot_with(&image, |_| {})),
        Ok(()),
        "实现写出来的原样先要过"
    );
    for format_version in [0u16, 2, u16::MAX] {
        let slot = system_configuration_slot_with(&image, |slot| {
            slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
                .copy_from_slice(&format_version.to_le_bytes());
        });
        assert_eq!(
            verdict_of_the_slot(&slot),
            Err(Verdict::FormatVersionNotRecognized),
            "系统配置格式版本 {format_version}（整槽校验和对）要拒"
        );
    }
}

#[test]
fn a_system_configuration_slot_whose_encryption_type_is_not_off_is_refused() {
    let image = first_file_version_pool().memory_pool();
    for encryption_type in [1u8, 2, u8::MAX] {
        let slot = system_configuration_slot_with(&image, |slot| {
            slot[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] = encryption_type;
        });
        assert_eq!(
            verdict_of_the_slot(&slot),
            Err(Verdict::EncryptionTypeNotOff),
            "加密类型 {encryption_type}：这一版不读加密的卷（1、2 是登记过的算法，别的不认得）"
        );
    }
}

#[test]
fn a_fixed_structure_slot_spacing_below_four_kibibytes_or_reaching_into_the_root_ring_is_refused() {
    let image = first_file_version_pool().memory_pool();
    let largest_slot_spacing_before_the_root_ring = ROOT_RING_BASE_OFFSET_OF_THE_POOL - 4096;
    for (slot_spacing, expected) in [
        (SLOT_SPACING_OF_THE_POOL, Ok(())),
        (
            4095,
            Err(Verdict::FixedStructureSlotSpacingOutsideTheFormatRange),
        ),
        (
            0,
            Err(Verdict::FixedStructureSlotSpacingOutsideTheFormatRange),
        ),
        (largest_slot_spacing_before_the_root_ring, Ok(())),
        (
            largest_slot_spacing_before_the_root_ring + 1,
            Err(Verdict::FixedStructureSlotSpacingOutsideTheFormatRange),
        ),
        (
            u32::MAX,
            Err(Verdict::FixedStructureSlotSpacingOutsideTheFormatRange),
        ),
    ] {
        let slot = system_configuration_slot_with(
            &image,
            with_u32_at(SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET, slot_spacing),
        );
        assert_eq!(
            verdict_of_the_slot(&slot),
            expected,
            "槽距 {slot_spacing}：下界 4096，上界是槽 1 整槽落在根环起点 {ROOT_RING_BASE_OFFSET_OF_THE_POOL} 之前"
        );
    }
}

#[test]
fn a_physical_block_size_that_cannot_hold_a_root_record_or_is_wider_than_the_slot_spacing_is_refused(
) {
    let image = first_file_version_pool().memory_pool();
    for (physical_block_size, expected) in [
        (ROOT_RECORD_BYTES, Ok(())),
        (
            ROOT_RECORD_BYTES - 1,
            Err(Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds),
        ),
        (0, Err(Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds)),
        (SLOT_SPACING_OF_THE_POOL, Ok(())),
        (
            SLOT_SPACING_OF_THE_POOL + 1,
            Err(Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds),
        ),
        (
            u32::MAX,
            Err(Verdict::PhysicalBlockSizeOutsideTheRootSlotBounds),
        ),
    ] {
        let slot = system_configuration_slot_with(
            &image,
            with_u32_at(
                SYSTEM_CONFIGURATION_PHYSICAL_BLOCK_SIZE_OFFSET,
                physical_block_size,
            ),
        );
        assert_eq!(
            verdict_of_the_slot(&slot),
            expected,
            "physical_block_size {physical_block_size}：根槽要装得下 {ROOT_RECORD_BYTES} 字节的根记录、不宽过槽距 {SLOT_SPACING_OF_THE_POOL}"
        );
    }
}

#[test]
fn a_journal_ring_that_holds_fewer_than_the_safety_factor_records_or_ends_past_the_unit_area_is_refused(
) {
    let image = first_file_version_pool().memory_pool();
    for (journal_ring_bytes, expected) in [
        (JOURNAL_RING_BYTES_OF_THE_POOL, Ok(())),
        // 装得下 F 条记录的最短环过得了环长那一判；单元区起始槽号照旧 50176、不是这个环长现算的槽 1025，
        // 报的是单元区起点那个成员（实审 Y4-a：checker 与实现收同一张表）。
        (
            SMALLEST_JOURNAL_RING_BYTES_HOLDING_THE_SAFETY_FACTOR_RECORDS,
            Err(Verdict::UnitAreaStartNotTheSlotAfterTheJournalRing),
        ),
        (
            SMALLEST_JOURNAL_RING_BYTES_HOLDING_THE_SAFETY_FACTOR_RECORDS - 1,
            Err(Verdict::JournalRingBytesOutsideTheSupportedRange),
        ),
        (0, Err(Verdict::JournalRingBytesOutsideTheSupportedRange)),
        (
            JOURNAL_RING_BYTES_OF_THE_POOL + 1,
            Err(Verdict::JournalRingBytesOutsideTheSupportedRange),
        ),
        (
            u64::MAX,
            Err(Verdict::JournalRingBytesOutsideTheSupportedRange),
        ),
    ] {
        let slot = system_configuration_slot_with(
            &image,
            with_u64_at(
                SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET,
                journal_ring_bytes,
            ),
        );
        assert_eq!(
            verdict_of_the_slot(&slot),
            expected,
            "环长 {journal_ring_bytes}：至少 3 条记录，末端不越过单元区起点（环起点槽 1024、单元区起始槽 50176）"
        );
    }
}

/// 池级：四个系统配置槽（两盘各两槽）都带同一个这一版不收的值、整槽校验和都对。改之前池级 checker 照这份几何判下去，
/// 报一串「成立」（加密类型、格式版本、环长那几份与没改的镜像一样 42 条成立）；改之后一个槽都择不出来，
/// checker 不替这个挂不上的镜像作保——一条「成立」都不报。
#[test]
fn a_pool_whose_every_system_configuration_slot_carries_a_refused_value_gets_no_invariant_vouched_for(
) {
    let image = first_file_version_pool().memory_pool();
    let holds_on = |candidate: &MemoryPool| {
        check_pool_image(candidate)
            .into_iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Holds))
            .count()
    };
    assert!(holds_on(&image) > 0, "没改的镜像上有成立的不变量");
    let refused_values: [(&str, SystemConfigurationSlotChange); 5] = [
        (
            "格式版本 2",
            Box::new(|slot: &mut [u8]| {
                slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
                    .copy_from_slice(&2u16.to_le_bytes());
            }),
        ),
        (
            "加密类型 1",
            Box::new(|slot: &mut [u8]| slot[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] = 1),
        ),
        (
            "槽距 4095",
            Box::new(with_u32_at(SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET, 4095)),
        ),
        (
            "physical_block_size 456",
            Box::new(with_u32_at(
                SYSTEM_CONFIGURATION_PHYSICAL_BLOCK_SIZE_OFFSET,
                ROOT_RECORD_BYTES - 1,
            )),
        ),
        (
            "环长越过单元区起点一条记录",
            Box::new(with_u64_at(
                SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET,
                JOURNAL_RING_BYTES_OF_THE_POOL + 4096,
            )),
        ),
    ];
    for (what, change) in refused_values {
        let mut refused = image.clone();
        for device in [0u32, 1] {
            let disk = refused
                .devices
                .get_mut(&DeviceIdentity(device))
                .expect("两块盘");
            for offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
                let mut slot =
                    disk.read(DeviceOffsetInBytes(offset), SYSTEM_CONFIGURATION_SLOT_BYTES);
                change(&mut slot);
                reseal_system_configuration_slot(&mut slot);
                disk.write(DeviceOffsetInBytes(offset), &slot);
            }
        }
        assert_eq!(
            holds_on(&refused),
            0,
            "四槽都是「{what}」：这个池挂不上，checker 不该报任何一条成立"
        );
        assert!(
            matches!(
                verdict_on(
                    &refused,
                    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
                ),
                InvariantVerdict::Violated(_)
            ),
            "四槽都是「{what}」：带越界值的系统配置槽报违例，不止是不作保"
        );
    }
}

/// 池级判决里一条不变量的那一格。
fn verdict_on(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .unwrap_or_else(|| panic!("{invariant} 在池级 checker 的清单里"))
        .1
}

/// 这一版全部角色的单元（槽, 字节数）：补校验和时沿它们找位置条目。
fn units_of_the_version(pool: &TreeSplitPool) -> Vec<(u64, usize)> {
    pool.output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect()
}

/// 现行那一版里的一个单元换成 `replacement`（两块盘同槽），新的整单元校验和沿引用链补到根槽。
fn replace_unit_and_propagate(
    pool: &TreeSplitPool,
    image: &mut MemoryPool,
    original: &PublishedUnit,
    replacement: &[u8],
) {
    write_unit_to_the_same_slot_on_both_devices(image, original.slot.0, replacement);
    propagate_the_new_checksum(
        image,
        &units_of_the_version(pool),
        (
            original.slot.0,
            crc32_castagnoli(&original.bytes),
            crc32_castagnoli(replacement),
        ),
    );
}

/// 一种坏法：改一处、按类重封，交回说明文字里该有的那一段。
enum HeaderDamage {
    FormatVersion(u16),
    ReservedByte { position_in_reserved_bytes: usize },
}

#[test]
fn every_unit_class_with_an_unrecognized_format_version_or_a_non_zero_reserved_byte_reddens_the_header_invariant_in_the_pool_walk(
) {
    let pool = first_file_version_pool();
    let image = pool.memory_pool();
    assert!(
        matches!(verdict_on(&image, "I-2.4"), InvariantVerdict::Holds),
        "没改的镜像上 I-2.4 成立"
    );
    let unit_of_each_class = [
        (
            TransactionUnit::Data(DataUnitIndexInFile::FIRST),
            DATA_UNIT_RESERVED_START,
        ),
        (TransactionUnit::InodeRoot, INODE_TREE_ROOT_RESERVED_START),
        (
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
            PACKED_UNIT_RESERVED_START,
        ),
    ];
    for (identity, reserved_start) in unit_of_each_class {
        let original = pool.output.unit(identity);
        for damage in [
            HeaderDamage::FormatVersion(2),
            HeaderDamage::ReservedByte {
                position_in_reserved_bytes: 0,
            },
            HeaderDamage::ReservedByte {
                position_in_reserved_bytes: ENCRYPTION_RESERVED_BYTES - 1,
            },
        ] {
            let (damaged_unit, expected_in_the_detail) = match damage {
                HeaderDamage::FormatVersion(format_version) => (
                    resealed_with_format_version(&original.bytes, format_version),
                    "格式版本",
                ),
                HeaderDamage::ReservedByte {
                    position_in_reserved_bytes,
                } => (
                    resealed_with_byte(
                        &original.bytes,
                        reserved_start + position_in_reserved_bytes,
                        1,
                    ),
                    "预留位",
                ),
            };
            let mut damaged = image.clone();
            replace_unit_and_propagate(&pool, &mut damaged, original, &damaged_unit);
            match verdict_on(&damaged, "I-2.4") {
                InvariantVerdict::Violated(detail) => assert!(
                    detail.contains(expected_in_the_detail),
                    "{identity:?}：I-2.4 的说明要点名{expected_in_the_detail}：{detail}"
                ),
                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
                    panic!("{identity:?} 的{expected_in_the_detail}改坏、两道校验和都对：池级走读要报 I-2.4，得到 {other:?}")
                }
            }
            match verdict_on(&damaged, "I-7.2") {
                InvariantVerdict::Violated(detail) => assert!(
                    detail.contains("头用不了"),
                    "{identity:?}：最新根走不完，断在头用不了那一步：{detail}"
                ),
                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
                    panic!("{identity:?} 的头用不了，最新根不该算走得完：{other:?}")
                }
            }
        }
    }
}

/// 指针头部 MAC 16（偏移 0）与 nonce 12（偏移 16）；位置条目在偏移 50（D19（块指针的结构与宽度预算） 已定项 11）。
const POINTER_LOCATION_ENTRIES_OFFSET: usize = 50;
/// 根记录里树表指针与实例表指针的偏移（D22（单元原子性怎么合成） 已定项 7 的字段表）。
const ROOT_RECORD_TREE_TABLE_POINTER_OFFSET: usize = 36;
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
const NODE_POINTER_BYTES: usize = 86;
/// 根槽：宽 512（`common::parameters` 的 `physical_block_size`），magic `SFSR`，自证校验和 32 字节在 138、罩整槽。
const ROOT_SLOT_BYTES: usize = 512;
const ROOT_CHECKSUM_OFFSET: usize = 138;

/// 根环里每一条根记录的 `pointer_offset` 那条指针（不全零的），头部第 `byte_in_pointer_head` 字节改成 1、根槽重算自证校验和。
fn root_ring_with_a_pointer_head_byte_set(
    image: &mut MemoryPool,
    pointer_offset: usize,
    byte_in_pointer_head: usize,
) {
    let mut roots_changed = 0;
    for (device, offset) in root_ring_slot_offsets() {
        let disk = image
            .devices
            .get_mut(&DeviceIdentity(device))
            .expect("两块盘");
        let mut slot = disk.read(DeviceOffsetInBytes(offset), ROOT_SLOT_BYTES);
        let pointer = pointer_offset..pointer_offset + NODE_POINTER_BYTES;
        if &slot[..4] != b"SFSR" || slot[pointer].iter().all(|byte| *byte == 0) {
            continue;
        }
        slot[pointer_offset + byte_in_pointer_head] = 1;
        let digest = wide_checksum_with_field_zeroed(&slot, ROOT_SLOT_BYTES, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        disk.write(DeviceOffsetInBytes(offset), &slot);
        roots_changed += 1;
    }
    assert!(roots_changed > 0, "根环里有根记录的这条指针不全零");
}

/// 父单元里指向 `child` 的那条指针：按它的第一条位置条目（盘 0、槽、整单元校验和）逐字节找，恰好一处；头部第
/// `byte_in_pointer_head` 字节改成 1，父单元按类重封。
fn parent_with_a_pointer_head_byte_set(
    parent: &[u8],
    child: &PublishedUnit,
    byte_in_pointer_head: usize,
) -> Vec<u8> {
    let first_location_entry = [
        0u32.to_le_bytes().to_vec(),
        child.slot.0.to_le_bytes()[..6].to_vec(),
        crc32_castagnoli(&child.bytes).to_le_bytes().to_vec(),
    ]
    .concat();
    let positions: Vec<usize> = parent
        .windows(first_location_entry.len())
        .enumerate()
        .filter(|(_, window)| *window == first_location_entry.as_slice())
        .map(|(position, _)| position)
        .collect();
    assert_eq!(positions.len(), 1, "父单元里指向它的指针恰好一条");
    let mut damaged = parent.to_vec();
    damaged[positions[0] - POINTER_LOCATION_ENTRIES_OFFSET + byte_in_pointer_head] = 1;
    reseal_unit_by_its_class(&mut damaged);
    damaged
}

/// 走读跟随指针的四处（`walk` 里判 I-2.4 指针那一半的四个调用点各一条）。
#[derive(Clone, Copy, Debug)]
enum FollowedPointer {
    /// 根记录里的树表指针：`read_index_node` 那一处（码 2 节点都经它）。
    TreeTablePointerInTheRootRecord,
    /// 根记录里的实例表指针：实例表链那一处。
    InstanceTablePointerInTheRootRecord,
    /// inode 树根里指向叶容器的子指针：inode 内部条目那一处。
    InodeRootChildPointer,
    /// extent 树根（上段根兼叶）里内联的数据指针：数据指针那一处。
    ExtentRootDataPointer,
}

#[test]
fn a_followed_pointer_whose_mac_or_nonce_is_not_zero_reddens_the_header_invariant() {
    let pool = first_file_version_pool();
    let image = pool.memory_pool();
    for followed_pointer in [
        FollowedPointer::TreeTablePointerInTheRootRecord,
        FollowedPointer::InstanceTablePointerInTheRootRecord,
        FollowedPointer::InodeRootChildPointer,
        FollowedPointer::ExtentRootDataPointer,
    ] {
        // MAC 的首尾、nonce 的首尾。
        for byte_in_pointer_head in [0usize, 15, 16, 27] {
            let mut damaged = image.clone();
            let expected_in_the_detail = match followed_pointer {
                FollowedPointer::TreeTablePointerInTheRootRecord => {
                    root_ring_with_a_pointer_head_byte_set(
                        &mut damaged,
                        ROOT_RECORD_TREE_TABLE_POINTER_OFFSET,
                        byte_in_pointer_head,
                    );
                    "树表单元"
                }
                FollowedPointer::InstanceTablePointerInTheRootRecord => {
                    root_ring_with_a_pointer_head_byte_set(
                        &mut damaged,
                        ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET,
                        byte_in_pointer_head,
                    );
                    "实例表指针"
                }
                FollowedPointer::InodeRootChildPointer => {
                    let parent = pool.output.unit(TransactionUnit::InodeRoot);
                    let child = pool.output.unit(TransactionUnit::InodeLeafContainer(
                        InodeLeafContainerIndexInTree::LEFTMOST,
                    ));
                    let damaged_parent = parent_with_a_pointer_head_byte_set(
                        &parent.bytes,
                        child,
                        byte_in_pointer_head,
                    );
                    replace_unit_and_propagate(&pool, &mut damaged, parent, &damaged_parent);
                    "inode 内部条目的子指针"
                }
                FollowedPointer::ExtentRootDataPointer => {
                    let parent = pool.output.unit(TransactionUnit::ExtentRoot);
                    let child = pool
                        .output
                        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST));
                    let damaged_parent = parent_with_a_pointer_head_byte_set(
                        &parent.bytes,
                        child,
                        byte_in_pointer_head,
                    );
                    replace_unit_and_propagate(&pool, &mut damaged, parent, &damaged_parent);
                    "extent 记录的数据指针"
                }
            };
            match verdict_on(&damaged, "I-2.4") {
                InvariantVerdict::Violated(detail) => assert!(
                    detail.contains(expected_in_the_detail) && detail.contains("MAC 16 / nonce 12"),
                    "{followed_pointer:?} 头部第 {byte_in_pointer_head} 字节是 1：I-2.4 的说明要点名{expected_in_the_detail}：{detail}"
                ),
                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => panic!(
                    "{followed_pointer:?} 头部第 {byte_in_pointer_head} 字节是 1（加密关着时恒 0）：池级走读要报 I-2.4，得到 {other:?}"
                ),
            }
        }
    }
}

/// 码 2 节点：key 宽 0、条目宽 0、条目数 3、声明长度 0，树 ID、层级与 fsid 抄实现写出来的 inode 树根（fsid 在 60 + 2 × key 宽，
/// D18（块里携带什么信息） 已定项 18）——头能用，解码时条目宽 0 而条目数非 0 被拒。
fn zero_width_node_in_place_of_the_inode_tree_root(inode_root: &[u8]) -> Vec<u8> {
    let inode_root_fsid_offset = 60 + 2 * usize::from(INODE_TREE_KEY_WIDTH);
    let mut node = index_node_with_zero_key_width_and_zero_entry_width(3);
    node[42..51].copy_from_slice(&inode_root[42..51]);
    node[60..68].copy_from_slice(&inode_root[inode_root_fsid_offset..inode_root_fsid_offset + 8]);
    reseal_unit_by_its_class(&mut node);
    node
}

#[test]
fn a_node_of_a_registered_tree_whose_entry_width_is_zero_while_its_entry_count_is_not_reddens_the_entry_width_invariant(
) {
    let pool = first_file_version_pool();
    let image = pool.memory_pool();
    assert!(
        matches!(verdict_on(&image, "I-1.10"), InvariantVerdict::Holds),
        "没改的镜像上 I-1.10 成立"
    );
    let inode_root = pool.output.unit(TransactionUnit::InodeRoot);
    let mut damaged = image.clone();
    replace_unit_and_propagate(
        &pool,
        &mut damaged,
        inode_root,
        &zero_width_node_in_place_of_the_inode_tree_root(&inode_root.bytes),
    );
    match verdict_on(&damaged, "I-1.10") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("条目宽 0 而条目数不是 0"),
            "inode 树（登记了条目字段表 120）的根条目宽 0、条目数 3：I-1.10 的说明要点名宽 0：{detail}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => panic!(
            "登记树的节点条目宽 0 而条目数非 0：0 不是任何登记宽，要报 I-1.10，得到 {other:?}"
        ),
    }
}

/// 池级：只有盘 1 的一个系统配置槽带这一版不收的值（整槽校验和对），别的三槽照原样。改之前 checker 把那一槽当不可择、
/// 拿别的槽照判，一条违例都没有；实现遇到它整池拒绝挂载。改之后报违例（点名盘与偏移），别的不变量一条「成立」都不报。
#[test]
fn a_pool_where_one_system_configuration_slot_carries_a_refused_value_is_reported_and_not_vouched_for(
) {
    let image = first_file_version_pool().memory_pool();
    assert!(
        matches!(
            verdict_on(
                &image,
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
            ),
            InvariantVerdict::Holds
        ),
        "没改的镜像上四槽都在范围里"
    );
    let refused_values: [(&str, SystemConfigurationSlotChange); 5] = [
        (
            "格式版本 2",
            Box::new(|slot: &mut [u8]| {
                slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
                    .copy_from_slice(&2u16.to_le_bytes());
            }),
        ),
        (
            "加密类型 1",
            Box::new(|slot: &mut [u8]| slot[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] = 1),
        ),
        (
            "槽距 4095",
            Box::new(with_u32_at(SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET, 4095)),
        ),
        (
            "physical_block_size 456",
            Box::new(with_u32_at(
                SYSTEM_CONFIGURATION_PHYSICAL_BLOCK_SIZE_OFFSET,
                ROOT_RECORD_BYTES - 1,
            )),
        ),
        (
            "环长越过单元区起点一条记录",
            Box::new(with_u64_at(
                SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET,
                JOURNAL_RING_BYTES_OF_THE_POOL + 4096,
            )),
        ),
    ];
    for (what, change) in refused_values {
        for offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
            let mut refused = image.clone();
            let disk = refused.devices.get_mut(&DeviceIdentity(1)).expect("盘 1");
            let mut slot = disk.read(DeviceOffsetInBytes(offset), SYSTEM_CONFIGURATION_SLOT_BYTES);
            change(&mut slot);
            reseal_system_configuration_slot(&mut slot);
            disk.write(DeviceOffsetInBytes(offset), &slot);
            let verdicts = check_pool_image(&refused);
            let reported = verdicts
                .iter()
                .find(|(invariant, _)| {
                    *invariant == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
                })
                .expect("在清单里");
            match &reported.1 {
                InvariantVerdict::Violated(detail) => assert!(
                    detail.contains(&format!("盘 1 偏移 {offset}")),
                    "只有盘 1 偏移 {offset} 那一槽是「{what}」：违例要点名那一槽：{detail}"
                ),
                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => panic!(
                    "只有盘 1 偏移 {offset} 那一槽是「{what}」：实现整池拒绝挂载，checker 要报违例，得到 {other:?}"
                ),
            }
            let vouched_for: Vec<&str> = verdicts
                .iter()
                .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Holds))
                .map(|(invariant, _)| *invariant)
                .collect();
            assert!(
                vouched_for.is_empty(),
                "只有盘 1 偏移 {offset} 那一槽是「{what}」：这个池挂不上，别的槽照判出来的成立不作数：{vouched_for:?}"
            );
        }
    }
}
