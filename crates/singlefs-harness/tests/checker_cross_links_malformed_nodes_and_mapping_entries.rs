//! 池级 checker 的三处盲区（代码审阅 6c 第 5、6、8 条，里程碑二收尾批「实审 B1」）各配会红的坏镜像：
//! ① 同一个根（或由记录施加出来的同一版）里第二次引用同一个单元，判 I-5.1（物理范围不重叠）；不同根之间共享没改过的单元照旧合法；
//! ② 盘上自述的 key 宽 / 层级 / journal 计数器是坏值时 checker 不许 panic（panic 等于那个状态上一条不变量都没判）；
//! ③ 中央映射条目：key 的类标签与出生身份要与它指的单元头相符，认不得的类标签不按 32 KiB 硬读，指的物理范围进引用集合。
//! 镜像都从 `TreeSplitPool` 的第一个文件版本（或再覆盖写一次）出发，改一处，被改单元重封、新校验和沿引用链补到根槽。

mod common;
mod common_tree_split;

use common::parameters;
use common_tree_split::{
    capacities, propagate_the_new_checksum, read_unit_on_device,
    write_unit_to_the_same_slot_on_both_devices, TreeSplitPool, ACCOUNTING_OF_THE_NODE_FORMAT,
    CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
};
use singlefs_checker::image::{InvariantVerdict, MAPPING_KEY_MATCHES_THE_UNIT_HEADER};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
use singlefs_format::{DATA_UNIT_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// 中央映射树叶压到 3 条：第一个文件版本的映射树长成两层（根是内部节点）。
const SMALL_CENTRAL_MAPPING: (usize, usize) = (3, 3);

/// extent 树上段叶条目（113）：locality 8、inode 8、offset 8、标签 1、载荷 88（标签 2 = 内联数据指针）。
const EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET: usize = 8;
const EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET: usize = 24;
const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;

/// 中央映射条目 55（D19（块指针的结构与宽度预算） 已定项 6 / 10）：key 27 = 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6，
/// value 28 = 位置条目 14 × 2（设备身份 4 + 槽号 6 + 整单元校验和 4）。
const MAPPING_KEY_CLASS_OFFSET: usize = 0;
const MAPPING_KEY_BIRTH_TREE_OFFSET: usize = 1;
const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 9;
const MAPPING_KEY_INSTANCE_OFFSET: usize = 17;
const MAPPING_KEY_TAIL_OFFSET: usize = 21;
const MAPPING_KEY_BYTES: usize = 27;
const MAPPING_LOCATION_ENTRY_BYTES: usize = 14;
const UNIT_CLASS_DATA: u8 = 1;
const UNIT_CLASS_PACKED: u8 = 3;

/// journal 记录头（D23（journal 的角色与格式） 已定项 17）：计数器 6 字节在 20；自证校验和 32 字节在 46、罩整条 4096。
const JOURNAL_RECORD_COUNTER_OFFSET: usize = 20;
const JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET: usize = 46;
const JOURNAL_RECORD_BYTES: usize = 4096;

/// 第一个文件版本的单元区只用到 50 26x 附近：这个槽在两块盘上都没被分配、没被引用。
const SLOT_NOBODY_ALLOCATED: u64 = 50_400;

fn product_capacity_pool() -> TreeSplitPool {
    TreeSplitPool::with_the_first_file_version_under(capacities(
        ACCOUNTING_OF_THE_NODE_FORMAT,
        CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
    ))
}

fn two_level_central_mapping_pool() -> TreeSplitPool {
    TreeSplitPool::with_the_first_file_version_under(capacities(
        ACCOUNTING_OF_THE_NODE_FORMAT,
        SMALL_CENTRAL_MAPPING,
    ))
}

/// 一份镜像上判红的不变量（带说明）。
fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn violated_names(violations: &[(&'static str, String)]) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = violations.iter().map(|(name, _)| *name).collect();
    names.dedup();
    names
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .expect("清单里有")
        .1
}

/// 这一版引用得到的单元（槽, 字节数）：补校验和时按它找父单元。
fn units_of(output: &TransactionOutput) -> Vec<(u64, usize)> {
    output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect()
}

/// 按解开的头重建一个码 2 节点：只换给定的几样，别的字段照原样。
fn rebuilt_index_node(
    header: &IndexNodeHeader,
    level: u8,
    key_width: usize,
    smallest_key: &[u8],
    largest_key: &[u8],
    entries: &[Vec<u8>],
) -> Vec<u8> {
    build_index_node(
        header.tree,
        level,
        key_width,
        smallest_key,
        largest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽"),
        entries,
    )
}

/// 一版里某个角色的码 2 单元换成 `replacement`（写回原槽），新整单元校验和沿引用链补到根槽；交回改过的镜像。
fn image_with_the_node_replaced(
    image: &MemoryPool,
    output: &TransactionOutput,
    role: TransactionUnit,
    replacement: &[u8],
) -> MemoryPool {
    let unit = output.unit(role);
    let mut damaged = image.clone();
    write_unit_to_the_same_slot_on_both_devices(&mut damaged, unit.slot.0, replacement);
    propagate_the_new_checksum(
        &mut damaged,
        &units_of(output),
        (
            unit.slot.0,
            crc32_castagnoli(&unit.bytes),
            crc32_castagnoli(replacement),
        ),
    );
    damaged
}

/// 重建时先核一遍：原样的字段重建出来的字节与发布写出的逐字节相同（重建没有改到别的字段）。
fn assert_the_rebuild_reproduces(unit_bytes: &[u8], header: &IndexNodeHeader) {
    assert_eq!(
        rebuilt_index_node(
            header,
            header.level,
            header.key_width,
            &header.smallest_key,
            &header.largest_key,
            &header.entries,
        ),
        unit_bytes,
        "原样重建要与发布写出的逐字节相同"
    );
}

// ─── ① 同一个根里第二次引用同一个单元 ───

/// 第一个文件版本的 extent 根兼叶里，把那条内联数据指针抄一份给下一个 inode 号：同一个根里两个 inode 的 extent 指向同一个数据单元。
/// 改之前的 checker 在第二次遇到这个单元时直接跳过（整个走读共用一份「走过的单元」），五元组、出生身份、I-5.1 全不判，全绿。
#[test]
fn two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_only_the_disjoint_ranges_invariant(
) {
    let pool = product_capacity_pool();
    let clean = pool.memory_pool();
    assert_eq!(violations(&clean), Vec::new(), "干净镜像全绿");
    let extent_root = pool.output.unit(TransactionUnit::ExtentRoot);
    let header = parse_index_node(&extent_root.bytes).expect("extent 根解得开");
    assert_the_rebuild_reproduces(&extent_root.bytes, &header);
    assert_eq!(header.level, 0, "第一个文件版本的 extent 上段只有根兼叶");
    let inline_entry = header
        .entries
        .iter()
        .find(|entry| {
            entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET]
                == EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT
        })
        .expect("一个数据单元的文件内联在上段叶里")
        .clone();
    let inode = u64::from_le_bytes(
        inline_entry
            [EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET..EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET + 8]
            .try_into()
            .expect("8 字节"),
    );
    let mut second_inode_entry = inline_entry.clone();
    second_inode_entry
        [EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET..EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET + 8]
        .copy_from_slice(&(inode + 1).to_le_bytes());
    let mut entries = header.entries.clone();
    entries.push(second_inode_entry);
    entries.sort_by_key(|entry| {
        let field = |offset: usize| {
            u64::from_le_bytes(entry[offset..offset + 8].try_into().expect("8 字节"))
        };
        (field(0), field(8), field(16))
    });
    let damaged_root = rebuilt_index_node(
        &header,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        &entries,
    );
    let image = image_with_the_node_replaced(
        &clean,
        &pool.output,
        TransactionUnit::ExtentRoot,
        &damaged_root,
    );
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        ["I-5.1"],
        "判红的该只有 I-5.1：{red:?}"
    );
    assert!(
        red[0].1.contains("被同一版引用了两次"),
        "说明要点出同一版里的第二次引用：{}",
        red[0].1
    );
}

/// 一条指针自己的两条位置条目写成同一个 (设备, 槽)（第二条抄第一条）：那是 I-2.5（位置条目按设备身份升序） 的事，
/// 一条指针是一个引用，不算「同一版里引用了两次」，I-5.1 不跟着红。
#[test]
fn one_pointer_whose_two_location_entries_name_one_placement_is_one_reference_not_a_cross_link() {
    /// 上段叶条目里内联数据指针的两条位置条目：载荷从 25 起，指针头部 50 之后各 14 字节。
    const FIRST_LOCATION_ENTRY_OFFSET: usize = 25 + 50;
    const SECOND_LOCATION_ENTRY_OFFSET: usize = 25 + 50 + 14;
    let pool = product_capacity_pool();
    let extent_root = pool.output.unit(TransactionUnit::ExtentRoot);
    let header = parse_index_node(&extent_root.bytes).expect("extent 根解得开");
    let mut entries = header.entries.clone();
    let inline_entry = entries
        .iter_mut()
        .find(|entry| {
            entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET]
                == EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT
        })
        .expect("一个数据单元的文件内联在上段叶里");
    let first_location =
        inline_entry[FIRST_LOCATION_ENTRY_OFFSET..FIRST_LOCATION_ENTRY_OFFSET + 14].to_vec();
    inline_entry[SECOND_LOCATION_ENTRY_OFFSET..SECOND_LOCATION_ENTRY_OFFSET + 14]
        .copy_from_slice(&first_location);
    let damaged_root = rebuilt_index_node(
        &header,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        &entries,
    );
    let image = image_with_the_node_replaced(
        &pool.memory_pool(),
        &pool.output,
        TransactionUnit::ExtentRoot,
        &damaged_root,
    );
    let red = violations(&image);
    let names = violated_names(&red);
    assert!(
        names.contains(&"I-2.5"),
        "两条位置条目设备身份相同：I-2.5 红：{red:?}"
    );
    assert!(
        !names.contains(&"I-5.1"),
        "一条指针是一个引用：I-5.1 不红：{red:?}"
    );
}

/// 不同根之间共享没改过的单元是合法的 COW：覆盖写一次之后，新旧两版（连同暖机那两代）都指着 mkfs 种的实例表单元，
/// 一版一版地走、每版各记一份「这一版引用过什么」，不把别的版引用过的算成第二次。
#[test]
fn a_unit_shared_by_the_roots_of_two_versions_is_not_a_cross_link() {
    let mut pool = product_capacity_pool();
    let content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    pool.overwrite(
        &content,
        capacities(
            ACCOUNTING_OF_THE_NODE_FORMAT,
            CENTRAL_MAPPING_OF_THE_NODE_FORMAT,
        ),
    );
    let image = pool.memory_pool();
    assert_eq!(
        pool.output.root.instance_table, pool.earlier_versions[0].root.instance_table,
        "覆盖写照抄实例表：两版的根指着同一个实例表单元"
    );
    assert_eq!(verdict_of(&image, "I-5.1"), InvariantVerdict::Holds);
    assert_eq!(violations(&image), Vec::new(), "覆盖写之后的干净镜像全绿");
}

// ─── ② 盘上自述的坏值不许让 checker panic ───

/// 两层中央映射树的根把自述的 key 宽改成 60（映射 key 是 27），条目宽 113 照旧：改之前 checker 按头里的 key 宽切子指针
/// `entry[60..146]`，条目只有 113 字节，当场越界 panic。
#[test]
fn a_central_mapping_root_whose_key_width_is_not_the_mapping_key_width_is_judged_without_a_panic() {
    let pool = two_level_central_mapping_pool();
    let clean = pool.memory_pool();
    assert_eq!(violations(&clean), Vec::new(), "干净镜像全绿");
    let root = pool.output.unit(TransactionUnit::MappingTree);
    let header = parse_index_node(&root.bytes).expect("映射根解得开");
    assert_the_rebuild_reproduces(&root.bytes, &header);
    assert!(header.level >= 1, "压小之后映射根是内部节点");
    let widened_key = |key: &[u8]| -> Vec<u8> {
        let mut widened = key.to_vec();
        widened.resize(60, 0);
        widened
    };
    let damaged_root = rebuilt_index_node(
        &header,
        header.level,
        60,
        &widened_key(&header.smallest_key),
        &widened_key(&header.largest_key),
        &header.entries,
    );
    let image = image_with_the_node_replaced(
        &clean,
        &pool.output,
        TransactionUnit::MappingTree,
        &damaged_root,
    );
    let red = violations(&image);
    let names = violated_names(&red);
    assert!(
        names.contains(&"I-1.1"),
        "key 宽不是映射 key 的宽：I-1.1 红：{red:?}"
    );
    assert!(
        names.contains(&"I-7.2"),
        "最新根的映射树走不下去：I-7.2 红：{red:?}"
    );
}

/// 两层中央映射树的一片叶把自述的 key 宽改成 20：改之前 checker 把叶头里 20 字节的最小 key 交回父节点、按 27 字节的映射 key
/// 形态逐字段读，读出切片之外，当场 panic。
#[test]
fn a_central_mapping_leaf_whose_key_width_is_narrower_than_the_mapping_key_is_judged_without_a_panic(
) {
    let pool = two_level_central_mapping_pool();
    let clean = pool.memory_pool();
    let leaf_role = pool
        .output
        .units
        .iter()
        .find(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            ) && parse_index_node(&unit.bytes).expect("映射节点解得开").level == 0
        })
        .expect("两层映射树有叶")
        .identity;
    let leaf = pool.output.unit(leaf_role);
    let header = parse_index_node(&leaf.bytes).expect("叶解得开");
    assert_the_rebuild_reproduces(&leaf.bytes, &header);
    let damaged_leaf = rebuilt_index_node(
        &header,
        header.level,
        20,
        &header.smallest_key[..20],
        &header.largest_key[..20],
        &header.entries,
    );
    let image = image_with_the_node_replaced(&clean, &pool.output, leaf_role, &damaged_leaf);
    let red = violations(&image);
    let names = violated_names(&red);
    assert!(
        names.contains(&"I-1.1"),
        "叶的 key 宽不是映射 key 的宽：I-1.1 红：{red:?}"
    );
    assert!(
        names.contains(&"I-7.2"),
        "最新根的映射树走不全：I-7.2 红：{red:?}"
    );
}

/// 覆盖写之后（最新根的账里有已释放的记录，引用集合那一遍要把每条候选根走到叶），两层中央映射树的一片叶把头里的层级改成 255：
/// 改之前「按条目往下数全部节点」那一遍拿孩子的层级加一去比父层级，u8 在 255 上溢出，当场 panic。
#[test]
fn a_central_mapping_leaf_whose_level_is_the_largest_level_byte_is_judged_without_a_panic() {
    let mut pool = two_level_central_mapping_pool();
    let content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    pool.overwrite(
        &content,
        capacities(ACCOUNTING_OF_THE_NODE_FORMAT, SMALL_CENTRAL_MAPPING),
    );
    let clean = pool.memory_pool();
    assert_eq!(violations(&clean), Vec::new(), "覆盖写之后的干净镜像全绿");
    let leaf_role = pool
        .output
        .units
        .iter()
        .find(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            ) && parse_index_node(&unit.bytes).expect("映射节点解得开").level == 0
        })
        .expect("两层映射树有叶")
        .identity;
    let leaf = pool.output.unit(leaf_role);
    let header = parse_index_node(&leaf.bytes).expect("叶解得开");
    assert_the_rebuild_reproduces(&leaf.bytes, &header);
    let damaged_leaf = rebuilt_index_node(
        &header,
        u8::MAX,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        &header.entries,
    );
    let image = image_with_the_node_replaced(&clean, &pool.output, leaf_role, &damaged_leaf);
    let red = violations(&image);
    assert!(
        violated_names(&red).contains(&"I-1.1"),
        "叶的层级不是父层级减一：I-1.1 红：{red:?}"
    );
}

/// journal 环里一条记录的计数器改成 0（头校验和重封，两盘一起改）：改之前反向链那一判拿「计数器 − 1」去找逻辑前一条，
/// 在 0 上下溢，当场 panic。计数器从 1 起，计数器 0 没有本实例内逻辑前一条、链无从定义，I-8.6 判违例（用户 2026-09-27 定）。
#[test]
fn a_journal_record_whose_counter_is_zero_is_judged_without_a_panic() {
    let pool = product_capacity_pool();
    let mut image = pool.memory_pool();
    let offset = singlefs_core::journal::record_offset(2, JOURNAL_RING_DEFAULT_BYTES).0;
    for device in [0u32, 1] {
        let mut record = image
            .devices
            .get(&DeviceIdentity(device))
            .expect("盘")
            .read(DeviceOffsetInBytes(offset), JOURNAL_RECORD_BYTES);
        assert_eq!(
            record[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6],
            2u64.to_le_bytes()[..6],
            "计数器 2 那条记录在它的环槽上"
        );
        record[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6].fill(0);
        let digest = wide_checksum_with_field_zeroed(
            &record,
            JOURNAL_RECORD_BYTES,
            JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET,
        );
        record[JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET..JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        image
            .devices
            .get_mut(&DeviceIdentity(device))
            .expect("盘")
            .write(DeviceOffsetInBytes(offset), &record);
    }
    match verdict_of(&image, "I-8.6") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("计数器 0"),
            "I-8.6 的说明要点出是计数器 0 那一条：{detail}"
        ),
        other_verdict @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
            panic!("计数器 0 那一条没有本实例内逻辑前一条，I-8.6 该判违例，实际 {other_verdict:?}")
        }
    }
}

/// 系统配置槽（4096）：自证校验和 32 字节在 155、罩整槽；单元区起点 8 字节在 417；槽距 4 字节在 429（槽 1 的偏移）。
const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
const SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET: usize = 417;
const SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET: usize = 429;

/// 两块盘各两个系统配置槽里的单元区起点都改成 2^40（4 GiB 的盘只有 2^18 个槽），自证校验和重封：
/// 改之前 checker 拿「盘上槽数 − 单元区起点」算单元区容量，u64 下溢，当场 panic。起点越过盘末，单元区没有容量可比，I-5.2 判红。
#[test]
fn a_system_configuration_whose_unit_area_starts_past_the_device_end_is_judged_without_a_panic() {
    let pool = product_capacity_pool();
    let mut image = pool.memory_pool();
    for device in [0u32, 1] {
        let slot_zero = image
            .devices
            .get(&DeviceIdentity(device))
            .expect("盘")
            .read(DeviceOffsetInBytes(0), SYSTEM_CONFIGURATION_SLOT_BYTES);
        let slot_spacing = u64::from(u32::from_le_bytes(
            slot_zero[SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET
                ..SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET + 4]
                .try_into()
                .expect("4 字节"),
        ));
        for slot_offset in [0, slot_spacing] {
            let mut slot = image
                .devices
                .get(&DeviceIdentity(device))
                .expect("盘")
                .read(
                    DeviceOffsetInBytes(slot_offset),
                    SYSTEM_CONFIGURATION_SLOT_BYTES,
                );
            slot[SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET
                ..SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET + 8]
                .copy_from_slice(&(1u64 << 40).to_le_bytes());
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                SYSTEM_CONFIGURATION_SLOT_BYTES,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            image
                .devices
                .get_mut(&DeviceIdentity(device))
                .expect("盘")
                .write(DeviceOffsetInBytes(slot_offset), &slot);
        }
    }
    let red = violations(&image);
    assert!(
        violated_names(&red).contains(&"I-5.2"),
        "单元区起点越过盘末：I-5.2 红：{red:?}"
    );
}

/// 记账树（第一个文件版本是根兼叶）里盘 0 的「空闲」那一行改成 u64 的最大值：改之前 checker 拿「空闲 + 已分配」去比单元区容量，
/// u64 上溢，当场 panic。加起来溢出就是对不上，I-5.2 判红。
#[test]
fn an_accounting_row_whose_free_bytes_overflow_the_sum_is_judged_without_a_panic() {
    const ACCOUNTING_ROW_STATISTIC_OFFSET: usize = 0;
    const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 2 + 8;
    const ACCOUNTING_ROW_VALUE_OFFSET: usize = 2 + 8 + 4 + 8;
    const STATISTIC_FREE_BYTES: u16 = 2;
    let pool = product_capacity_pool();
    let accounting = pool.output.unit(TransactionUnit::AccountingTree);
    let header = parse_index_node(&accounting.bytes).expect("记账树解得开");
    assert_the_rebuild_reproduces(&accounting.bytes, &header);
    let mut entries = header.entries.clone();
    let free_row_of_device_zero = entries
        .iter_mut()
        .find(|row| {
            row[ACCOUNTING_ROW_STATISTIC_OFFSET..ACCOUNTING_ROW_STATISTIC_OFFSET + 2]
                == STATISTIC_FREE_BYTES.to_le_bytes()
                && row[ACCOUNTING_ROW_DEVICE_OFFSET..ACCOUNTING_ROW_DEVICE_OFFSET + 4]
                    == 0u32.to_le_bytes()
        })
        .expect("盘 0 有一行「空闲」");
    free_row_of_device_zero[ACCOUNTING_ROW_VALUE_OFFSET..ACCOUNTING_ROW_VALUE_OFFSET + 8]
        .copy_from_slice(&u64::MAX.to_le_bytes());
    let damaged = rebuilt_index_node(
        &header,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        &entries,
    );
    let image = image_with_the_node_replaced(
        &pool.memory_pool(),
        &pool.output,
        TransactionUnit::AccountingTree,
        &damaged,
    );
    let red = violations(&image);
    assert!(
        violated_names(&red).contains(&"I-5.2"),
        "空闲 + 已分配溢出：I-5.2 红：{red:?}"
    );
}

// ─── ③ 中央映射条目 ───

/// 映射 key 的字段序（类标签, 出生树, 出生 txg, 实例代号, 尾段 6 字节），按字段比无符号整数（D8（核心索引结构） 已定项 11）。
fn mapping_key_order(entry: &[u8]) -> (u8, u64, u64, u32, u64) {
    let read_u64 =
        |offset: usize| u64::from_le_bytes(entry[offset..offset + 8].try_into().expect("8 字节"));
    let mut tail = [0u8; 8];
    tail[..6].copy_from_slice(&entry[MAPPING_KEY_TAIL_OFFSET..MAPPING_KEY_TAIL_OFFSET + 6]);
    (
        entry[MAPPING_KEY_CLASS_OFFSET],
        read_u64(MAPPING_KEY_BIRTH_TREE_OFFSET),
        read_u64(MAPPING_KEY_BIRTH_TXG_OFFSET),
        u32::from_le_bytes(
            entry[MAPPING_KEY_INSTANCE_OFFSET..MAPPING_KEY_INSTANCE_OFFSET + 4]
                .try_into()
                .expect("4 字节"),
        ),
        u64::from_le_bytes(tail),
    )
}

/// 第一个文件版本的中央映射树（根兼叶）里的条目按 `change` 改，改完按 key 重新排序、头里的区间贴紧首末条目，重建、补校验和。
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
        header.key_width,
        &smallest_key,
        &largest_key,
        &entries,
    );
    image_with_the_node_replaced(
        &pool.memory_pool(),
        &pool.output,
        TransactionUnit::MappingTree,
        &damaged,
    )
}

fn entry_of_class(entries: &mut [Vec<u8>], unit_class: u8) -> &mut Vec<u8> {
    let mut matching = entries
        .iter_mut()
        .filter(|entry| entry[MAPPING_KEY_CLASS_OFFSET] == unit_class);
    let entry = matching.next().expect("这一类有一条映射条目");
    assert!(
        matching.next().is_none(),
        "第一个文件版本里这一类只有一条映射条目"
    );
    entry
}

/// 数据单元那条映射条目的 key 尾段（码 1 的事务号）加一：key 说的出生身份与它指的数据单元头里的不再相同，
/// 判映射 key 与单元头相符那一条（实审 B2 从 I-1.2 搬过去）。改之前 checker 只核映射条目的校验和与升序，全绿。
#[test]
fn a_mapping_key_whose_birth_identity_is_not_the_one_in_the_unit_header_reddens_only_the_mapping_key_invariant(
) {
    let pool = product_capacity_pool();
    let image = image_with_the_mapping_entries_changed(&pool, |entries| {
        let entry = entry_of_class(entries, UNIT_CLASS_DATA);
        entry[MAPPING_KEY_TAIL_OFFSET] = entry[MAPPING_KEY_TAIL_OFFSET].wrapping_add(1);
    });
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        [MAPPING_KEY_MATCHES_THE_UNIT_HEADER],
        "判红的该只有映射 key 与单元头相符那一条：{red:?}"
    );
    assert!(
        red[0].1.contains("映射条目"),
        "说明要点出是映射条目：{}",
        red[0].1
    );
}

/// inode 叶容器（码 3）那条映射条目的 key 类标签改成 1：key 说它是数据单元，指的单元头里是码 3（两者都是 32 KiB，读得出、校验和对得上），
/// 判映射 key 与单元头相符那一条（实审 B2 从 I-1.6 搬过去）。改之前全绿。
#[test]
fn a_mapping_key_whose_class_is_not_the_class_of_the_unit_it_names_reddens_only_the_mapping_key_invariant(
) {
    let pool = product_capacity_pool();
    let image = image_with_the_mapping_entries_changed(&pool, |entries| {
        entry_of_class(entries, UNIT_CLASS_PACKED)[MAPPING_KEY_CLASS_OFFSET] = UNIT_CLASS_DATA;
    });
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        [MAPPING_KEY_MATCHES_THE_UNIT_HEADER],
        "判红的该只有映射 key 与单元头相符那一条：{red:?}"
    );
    assert!(
        red[0].1.contains("映射条目"),
        "说明要点出是映射条目：{}",
        red[0].1
    );
}

/// 数据单元那条映射条目的 key 类标签改成登记表之外的码（0、4、255），同时把它两条位置条目里的整单元校验和改坏：
/// 认不得的类标签判映射 key 与单元头相符那一条（实审 B2 从 I-1.6 搬过去）、不去读那个单元——读了就会在校验和上再红 I-2.1。
/// 改之前按 32 KiB 硬读，红在 I-2.1 上。
#[test]
fn a_mapping_key_with_an_unregistered_class_reddens_only_the_mapping_key_invariant_and_is_not_read()
{
    let pool = product_capacity_pool();
    for unregistered_class in [0u8, 4, u8::MAX] {
        let image = image_with_the_mapping_entries_changed(&pool, |entries| {
            let entry = entry_of_class(entries, UNIT_CLASS_DATA);
            entry[MAPPING_KEY_CLASS_OFFSET] = unregistered_class;
            for location in 0..2 {
                let checksum_offset =
                    MAPPING_KEY_BYTES + location * MAPPING_LOCATION_ENTRY_BYTES + 10;
                entry[checksum_offset] ^= 0xff;
            }
        });
        let red = violations(&image);
        assert_eq!(
            violated_names(&red),
            [MAPPING_KEY_MATCHES_THE_UNIT_HEADER],
            "类标签 {unregistered_class}：判红的该只有映射 key 与单元头相符那一条（不读、I-2.1 不红）：{red:?}"
        );
    }
}

/// 数据单元那条映射条目的两条位置条目改指一个没人分配的槽，那里放一份逐字节相同的数据单元（校验和对得上）：
/// 映射是解引用的唯一入口（D19（块指针的结构与宽度预算） 已定项 5），它指的物理范围占着空间，要进引用集合。
/// 改之前映射条目不进引用集合，I-3.1 与 I-3.11 照旧对得上、全绿。
#[test]
fn a_mapping_entry_naming_an_unallocated_copy_counts_as_a_reference_and_reddens_the_allocated_bytes_invariants(
) {
    let pool = product_capacity_pool();
    let data_unit = pool
        .output
        .units
        .iter()
        .find(|unit| matches!(unit.identity, TransactionUnit::Data(_)))
        .expect("第一个文件有一个数据单元");
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let copy = read_unit_on_device(&pool.memory_pool(), 0, data_unit.slot.0, data_unit_bytes);
    assert_eq!(copy, data_unit.bytes, "盘上那一份就是发布写出的");
    let checksum = crc32_castagnoli(&copy);
    let mut image = image_with_the_mapping_entries_changed(&pool, |entries| {
        let entry = entry_of_class(entries, UNIT_CLASS_DATA);
        for (location, device) in [0u32, 1].into_iter().enumerate() {
            let start = MAPPING_KEY_BYTES + location * MAPPING_LOCATION_ENTRY_BYTES;
            entry[start..start + 4].copy_from_slice(&device.to_le_bytes());
            entry[start + 4..start + 10].copy_from_slice(&SLOT_NOBODY_ALLOCATED.to_le_bytes()[..6]);
            entry[start + 10..start + 14].copy_from_slice(&checksum.to_le_bytes());
        }
    });
    write_unit_to_the_same_slot_on_both_devices(&mut image, SLOT_NOBODY_ALLOCATED, &copy);
    let red = violations(&image);
    assert_eq!(
        violated_names(&red),
        ["I-3.1", "I-3.11"],
        "映射条目指的那一份算进遍历：已分配统计两条对不上，别的不红：{red:?}"
    );
}
