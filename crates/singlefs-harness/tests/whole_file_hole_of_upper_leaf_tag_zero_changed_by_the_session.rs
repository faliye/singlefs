//! 上段叶标签 0 配 size > 0（一个数据单元都没有，整个文件都是洞：判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5、
//! 第六节 S1）的第一个文件，可写挂载之后会话里接着发布：写者接得住这一形（判决 `research/prompts/m4-validate-code-r1-main-verification.md`
//! 用户定案「丑」）。
//! - 改 size（截短、N = S、变长）只改 inode 记录，不加单元、标签留 0；改之前三条都 panic 在 `truncate.rs` 那两处断言。
//! - 重写第一个文件所在那片上段叶的发布（按号写 inode 2）、与长高之后照抄那片叶的发布（按号写第一片上段叶之外的号）都留着标签 0 那一条；
//!   改之前前一条丢掉那一条，后一条把那片叶整片换下，写出的那一版三处读者都拒。
//! - 截到 0 之后（标签留 0、size 0）再重写那片上段叶的发布（按号写 inode 2、上段长高、写 inode 2 再变长到 7000）照写标签 0 那一条，
//!   截到 0 再删第一个文件之后上段叶不留它那一条（判决 `research/prompts/m4-validate-code-r2-main-verification.md` 第二节 T1，
//!   用户 2026-10-24 定「甲」：第一个文件一个数据单元都没有时 size 0 也写标签 0）。改之前写者按 size 0 不写那一条：前三条写出的那一版
//!   冷走读、可写重挂、冷恢复都拒成「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」，删完的那一版上段叶留一条孤条目。
//! - 删第一个文件之后的上段（判决 `research/prompts/m4-validate-code-r3-main-verification.md` 第二节 T5）：建到 143 号、删完再写它，第一片上段叶
//!   不在形状里（143 在删之前建）；截到 0、写过 inode 2 之后删第一个文件，上段叶只剩 inode 2 那一条。这两条之后读者读到没有文件、挂载态读者读回那个别的 inode。
//!
//! 每条之后四处读者各读一遍第一个文件：冷走读、可写重挂（重建出的那一版）、冷恢复（看 journal）、挂载态读者（只读挂载、打开、从 0 读到 size）。
//! 镜像手造，办法同 `whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader.rs`：新池新建文件（3000 字节、标签 2
//! 内联一个单元）那一版，第一个文件那一条上段叶条目改成标签 0、载荷清零，中央映射树里那个数据单元的映射条目拿掉，新的整单元校验和沿引用链
//! 补到根槽。今天的写路径写不出这一形（调查员报告：约 5800 次发布里 0 次）。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, parameters};
use common_deletion_in_progress::change_the_unit_and_propagate_its_checksum;
use common_pointer_head_corruption::{
    first_upper_leaf_entry, newest_root, node_bytes, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, SlotNumber};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::ByteReader;
use singlefs_core::extent_tree::{ExtentUpperLeafEntry, ExtentUpperLeafTarget};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{parse_mapping_entry, TREE_KIND_EXTENT};
use singlefs_core::recovery::{recover, walk_to_file, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{PoolVersion, FIRST_INODE_NUMBER};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 新池新建文件那一版第一个文件的长度。
const SIZE_OF_THE_FIRST_FILE: u64 = 3000;
const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
const WRITE_TIME_SECONDS: u64 = 1_788_000_300;
/// extent 上段叶条目 113：key 24（locality 8、inode 8、offset 8）之后一字节标签、载荷 88。
const EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET: usize = 8;
const EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET: usize = 24;
/// extent 上段内部条目 110：key 24（locality 8、孩子按位置罩的第一个 inode 号 8、offset 8）之后是子指针 86。
const EXTENT_UPPER_INTERNAL_ENTRY_INODE_OFFSET: usize = 8;
/// 上段叶条目标签 0：没有数据单元。
const UPPER_LEAF_TAG_NO_DATA_UNIT: u8 = 0;
/// 一片上段叶罩的 inode 号个数（D8（核心索引结构） 已定项 14：`EXTENT_TREE_UPPER_LEAF_INODES`）：号 0 到 142 在第一片叶里，143 起在下一片。
const INODES_PER_UPPER_LEAF: u64 = 143;
/// 码 2 节点（key 宽 k）：声明长度在偏移 8、层级在 50、key 区间从 52 起两个 key、条目数在 82 + 2k、条目从 115 + 2k 起（D18（块里携带什么信息）
/// 已定项 18 的偏移表）。
const INDEX_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_RANGE_OFFSET: usize = 52;

fn payload_capacity_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 节点头里的 key 区间跟不跟着条目改：按 key 寻址的树（中央映射树）跟着改；按位置寻址的 extent 上段由位置定，不动。
#[derive(Clone, Copy)]
enum KeyRangeOfTheNode {
    FollowsTheEntries,
    FixedByThePosition,
}

/// `slot` 那个码 2 节点的条目整份交给 `change` 改，条目数、声明长度跟着改（key 区间照 `key_range`），重封、校验和补到根槽。
fn rewrite_the_entries_of_the_index_node_at(
    image: &mut MemoryPool,
    slot: SlotNumber,
    key_range: KeyRangeOfTheNode,
    change: impl FnOnce(&mut Vec<Vec<u8>>),
) {
    let parsed = parse_index_node(&unit_bytes_at(image, slot, node_bytes())).expect("节点解得开");
    let key_width = parsed.key_width;
    let entry_width = parsed.entry_width;
    let mut entries = parsed.entries;
    change(&mut entries);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let entries_start = 115 + 2 * key_width;
        node[entries_start..].fill(0);
        for (position, entry) in entries.iter().enumerate() {
            assert_eq!(entry.len(), entry_width, "条目宽不变");
            let start = entries_start + position * entry_width;
            node[start..start + entry_width].copy_from_slice(entry);
        }
        let count_offset = 82 + 2 * key_width;
        node[count_offset..count_offset + 2].copy_from_slice(
            &u16::try_from(entries.len())
                .expect("条目数装得进 2 字节")
                .to_le_bytes(),
        );
        node[INDEX_NODE_DECLARED_LENGTH_OFFSET..INDEX_NODE_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(
                &u16::try_from(entries.len() * entry_width)
                    .expect("声明长度装得进 2 字节")
                    .to_le_bytes(),
            );
        if let (KeyRangeOfTheNode::FollowsTheEntries, Some(first), Some(last)) =
            (key_range, entries.first(), entries.last())
        {
            let smallest = INDEX_NODE_KEY_RANGE_OFFSET;
            let largest = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
            node[smallest..smallest + key_width].copy_from_slice(&first[..key_width]);
            node[largest..largest + key_width].copy_from_slice(&last[..key_width]);
        }
    });
}

/// 新池新建文件那一版手造成「标签 0 配 size 3000」：第一个文件那一条上段叶条目改成标签 0、载荷清零，那个数据单元的映射条目拿掉。
fn image_whose_first_file_is_a_whole_file_hole(tag: &str) -> MemoryPool {
    let pool = build_pool(tag);
    let mut image = pool.memory_pool();
    let ExtentUpperLeafTarget::InlineDataUnit(data_unit) = first_upper_leaf_entry(&image).target
    else {
        panic!("前提：新池新建文件那一版第一个文件是标签 2、内联一个单元")
    };
    let mapping_root_slot = the_slot_of(&newest_root(&image).mapping_root);
    assert_eq!(
        unit_bytes_at(&image, mapping_root_slot, node_bytes())[INDEX_NODE_LEVEL_OFFSET],
        0,
        "前提：中央映射树只有根兼叶"
    );
    rewrite_the_entries_of_the_index_node_at(
        &mut image,
        mapping_root_slot,
        KeyRangeOfTheNode::FollowsTheEntries,
        |entries| {
            let before = entries.len();
            entries.retain(|entry| {
                let (_key, locations) = parse_mapping_entry(entry).expect("映射条目解得开");
                locations[0].slot != data_unit.locations[0].slot
            });
            assert_eq!(
                entries.len() + 1,
                before,
                "恰好一条映射条目落在那个数据单元上"
            );
        },
    );
    let extent_root_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root);
    rewrite_the_entries_of_the_index_node_at(
        &mut image,
        extent_root_slot,
        KeyRangeOfTheNode::FixedByThePosition,
        |entries| {
            let first_file_entry = entries
                .iter_mut()
                .find(|entry| {
                    read_u64_at(entry, EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET) == FIRST_INODE_NUMBER
                })
                .expect("上段叶里有第一个文件那一条");
            first_file_entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET] = UPPER_LEAF_TAG_NO_DATA_UNIT;
            first_file_entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET + 1..].fill(0);
        },
    );
    assert_eq!(
        first_upper_leaf_entry(&image).target,
        ExtentUpperLeafTarget::NoDataUnit,
        "前提：手造之后第一个文件那一条是标签 0"
    );
    image
}

type WritableDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn writable_devices_of(image: &MemoryPool) -> WritableDevices {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

fn image_of_the_devices(devices: &WritableDevices, device_size_in_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// 标签 0 镜像可写挂载、会话里按次序发 `changes`（每一条都要做成），交回之后的镜像。
fn image_after_the_session_publishes(
    image: &MemoryPool,
    changes: Vec<UserChange<'_>>,
) -> MemoryPool {
    let mut devices = writable_devices_of(image);
    let mounted = mount_writable(&parameters(), &mut devices).expect("标签 0 镜像可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    for change in changes {
        let what = format!("{change:?}").chars().take(120).collect::<String>();
        session
            .publish_user_change(&mut devices, change)
            .unwrap_or_else(|refusal| panic!("会话里这一条做成：{what}，实际拒 {refusal:?}"));
    }
    image_of_the_devices(&devices, image.device_size_in_bytes)
}

/// 四处读者读第一个文件都做成、size 是 `expected_size_in_bytes`、内容全 0、一个数据单元都没有。
fn every_reader_reads_the_first_file_as_a_whole_file_hole_of(
    image: &MemoryPool,
    expected_size_in_bytes: u64,
) {
    let expected_content =
        vec![0u8; usize::try_from(expected_size_in_bytes).expect("size 装得进 usize")];
    let root = newest_root(image);
    let mut mapping_fallbacks = 0;
    let cold_walk = walk_to_file(image, &root, &mut mapping_fallbacks)
        .unwrap_or_else(|failure| panic!("冷走读收下第一个文件，实际拒 {failure:?}"));
    assert!(
        cold_walk.as_ref() == Some(&expected_content),
        "冷走读读出 {expected_size_in_bytes} 个 0（读出 {:?} 字节）",
        cold_walk.as_ref().map(Vec::len)
    );
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => assert!(
            content == expected_content,
            "冷恢复读出 {expected_size_in_bytes} 个 0（读出 {} 字节）",
            content.len()
        ),
        other_outcome @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
            panic!("冷恢复读回第一个文件，实际 {other_outcome:?}")
        }
    }
    let mut devices = writable_devices_of(image);
    let remounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|refusal| panic!("可写重挂收下这一版，实际拒 {refusal:?}"));
    let PoolVersion::WithFile(rebuilt) = remounted.current else {
        panic!("可写重挂重建出带文件的一版")
    };
    assert_eq!(
        (
            rebuilt.inode_record.map(|record| record.size),
            rebuilt.data_pointers.len()
        ),
        (Some(expected_size_in_bytes), 0),
        "可写重挂重建出 size {expected_size_in_bytes}、一个数据单元都没有（整个文件都是洞，读出全 0）的第一个文件"
    );
    let read_only = mount_read_only(image)
        .unwrap_or_else(|refusal| panic!("只读挂载收下这一版，实际拒 {refusal:?}"));
    let file = read_only
        .mounted
        .open_file(image, FIRST_FILE)
        .unwrap_or_else(|failure| panic!("挂载态读者打开第一个文件，实际拒 {failure:?}"));
    assert_eq!(
        (file.inode_record().size, file.data_unit_count()),
        (expected_size_in_bytes, 0),
        "挂载态读者打开的第一个文件 size {expected_size_in_bytes}、一个数据单元都没有"
    );
    let read = file
        .read_at(image, FileOffsetInBytes(0), expected_size_in_bytes)
        .expect("挂载态读者从 0 读到 size");
    assert!(
        read.bytes == expected_content,
        "挂载态读者读出 {expected_size_in_bytes} 个 0（读出 {} 字节）",
        read.bytes.len()
    );
}

/// 一片上段叶：它按位置罩的第一个 inode 号（第 k 片是 143k）与叶里的条目。
struct ExtentUpperLeafRead {
    first_inode_covered: u64,
    entries: Vec<ExtentUpperLeafEntry>,
}

/// 最新那一版 extent 树上段的每片叶，按根里的次序（上段可能长高过一层：根兼叶时只有罩号 0 起的那一片）。
fn leaves_of_the_extent_upper_segment(image: &MemoryPool) -> Vec<ExtentUpperLeafRead> {
    let parse_entries = |entries: &[Vec<u8>]| -> Vec<ExtentUpperLeafEntry> {
        entries
            .iter()
            .map(|entry| ExtentUpperLeafEntry::parse(entry).expect("上段叶条目解得开"))
            .collect()
    };
    let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
    let root_bytes = unit_bytes_at(image, the_slot_of(&extent_root), node_bytes());
    let root = parse_index_node(&root_bytes).expect("extent 上段根解得开");
    if root_bytes[INDEX_NODE_LEVEL_OFFSET] == 0 {
        return vec![ExtentUpperLeafRead {
            first_inode_covered: 0,
            entries: parse_entries(&root.entries),
        }];
    }
    root.entries
        .iter()
        .map(|internal_entry| {
            // 上段内部条目 110：孩子那一段的起点 key（0, 第一个 inode 号, 0）之后是子指针 86。
            let child = NodePointer::read_from(&mut ByteReader::at(internal_entry, root.key_width));
            let leaf = parse_index_node(&unit_bytes_at(image, the_slot_of(&child), node_bytes()))
                .expect("extent 上段叶解得开");
            ExtentUpperLeafRead {
                first_inode_covered: read_u64_at(
                    internal_entry,
                    EXTENT_UPPER_INTERNAL_ENTRY_INODE_OFFSET,
                ),
                entries: parse_entries(&leaf.entries),
            }
        })
        .collect()
}

/// 最新那一版 extent 树上段里第一个文件那一条（逐片叶找）。
fn first_file_entry_of_the_extent_upper_segment(
    image: &MemoryPool,
) -> Option<ExtentUpperLeafTarget> {
    leaves_of_the_extent_upper_segment(image)
        .into_iter()
        .flat_map(|leaf| leaf.entries)
        .find(|entry| entry.inode == FIRST_INODE_NUMBER)
        .map(|entry| entry.target)
}

/// 第一个文件删完的一版：冷走读、冷恢复读到没有文件，可写重挂收下这一版，挂载态读者打开 `other_inode`、从 0 读到 size 读回 `expected_content`。
fn every_reader_reads_no_first_file_and_reads_back_the_other_inode(
    image: &MemoryPool,
    other_inode: InodeNumber,
    expected_content: &[u8],
) {
    let root = newest_root(image);
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(image, &root, &mut mapping_fallbacks),
        Ok(None),
        "冷走读读到没有文件"
    );
    let recovered = recover(image, JournalPolicy::Consult).outcome;
    assert!(
        matches!(recovered, RecoveryOutcome::NoFile { .. }),
        "冷恢复读到没有文件，实际 {recovered:?}"
    );
    let mut devices = writable_devices_of(image);
    mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|refusal| panic!("可写重挂收下这一版，实际拒 {refusal:?}"));
    let read_only = mount_read_only(image)
        .unwrap_or_else(|refusal| panic!("只读挂载收下这一版，实际拒 {refusal:?}"));
    let file = read_only
        .mounted
        .open_file(image, other_inode)
        .unwrap_or_else(|failure| {
            panic!("挂载态读者打开 inode {}，实际拒 {failure:?}", other_inode.0)
        });
    let size_in_bytes = u64::try_from(expected_content.len()).expect("内容长度装得进 u64");
    assert_eq!(
        file.inode_record().size,
        size_in_bytes,
        "挂载态读者打开的 inode {} size 是写进去的长度",
        other_inode.0
    );
    let read = file
        .read_at(image, FileOffsetInBytes(0), size_in_bytes)
        .expect("挂载态读者从 0 读到 size");
    assert!(
        read.bytes == expected_content,
        "挂载态读者读回 inode {} 写进去的内容（读出 {} 字节）",
        other_inode.0,
        read.bytes.len()
    );
}

/// 截短（3000 → 100）：意图、最后一批都只改记录与截断条目，不放单元（一个都没有）；三处读者读出 100 个 0。改之前 panic 在
/// `plan_the_extent_tree_after_a_size_change` 的「截断留至少一个单元」那一断言（之前 0、之后 0）。
#[test]
fn a_whole_file_hole_of_tag_zero_shrunk_by_the_session_reads_back_its_new_size_as_zeros() {
    let image = image_whose_first_file_is_a_whole_file_hole("tag-zero-shrunk-by-the-session");
    let after = image_after_the_session_publishes(
        &image,
        vec![UserChange::Truncate {
            inode: FIRST_FILE,
            new_size: 100,
            write_time_seconds: WRITE_TIME_SECONDS,
        }],
    );
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "标签留 0"
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, 100);
}

/// N = S（3000 → 3000）：一次只改记录的发布；三处读者读出 3000 个 0。改之前 panic 在同一断言。
#[test]
fn a_whole_file_hole_of_tag_zero_kept_at_its_size_by_the_session_reads_back_its_size_as_zeros() {
    let image =
        image_whose_first_file_is_a_whole_file_hole("tag-zero-kept-at-its-size-by-the-session");
    let after = image_after_the_session_publishes(
        &image,
        vec![UserChange::Truncate {
            inode: FIRST_FILE,
            new_size: SIZE_OF_THE_FIRST_FILE,
            write_time_seconds: WRITE_TIME_SECONDS,
        }],
    );
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "标签留 0"
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, SIZE_OF_THE_FIRST_FILE);
}

/// 变长（3000 → 三个净荷）：没有单元可补，一次只改记录的发布；三处读者读出三个净荷那么多个 0。改之前 panic 在
/// `tail_data_unit_rewrite_of_an_extension` 的「第一个文件至少一个数据单元」。
#[test]
fn a_whole_file_hole_of_tag_zero_extended_by_the_session_reads_back_its_new_size_as_zeros() {
    let image = image_whose_first_file_is_a_whole_file_hole("tag-zero-extended-by-the-session");
    let extended_size = 3 * payload_capacity_in_bytes();
    let after = image_after_the_session_publishes(
        &image,
        vec![UserChange::Truncate {
            inode: FIRST_FILE,
            new_size: extended_size,
            write_time_seconds: WRITE_TIME_SECONDS,
        }],
    );
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "标签留 0"
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, extended_size);
}

/// 建一个 inode、再按号写 inode 2：inode 2 与第一个文件在同一片上段叶（根兼叶）里，这片叶重写，第一个文件那一条照写标签 0；
/// 三处读者读第一个文件读出 3000 个 0。改之前（第一个文件零单元那一臂不写条目）这一条丢掉，三处读者都拒。
#[test]
fn a_whole_file_hole_of_tag_zero_keeps_its_entry_when_the_session_rewrites_its_upper_leaf_for_inode_two(
) {
    let image =
        image_whose_first_file_is_a_whole_file_hole("tag-zero-upper-leaf-rewritten-for-inode-two");
    let content_of_inode_two = vec![9u8; 100];
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(2),
                content: &content_of_inode_two,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "重写的那片上段叶里第一个文件那一条照写标签 0"
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, SIZE_OF_THE_FIRST_FILE);
}

/// 建到第一片上段叶之外的号（143）、再按号写它：上段长高一层，第一个文件那片叶不罩变了的号、照抄成根之下的叶，标签 0 那一条留着；
/// 三处读者读第一个文件读出 3000 个 0。改之前（有条目的号集合里第一个文件只在有数据单元时才算）那片叶不在新形状里、整片换下，三处读者都拒。
#[test]
fn a_whole_file_hole_of_tag_zero_keeps_its_upper_leaf_when_the_session_writes_an_inode_past_that_leaf(
) {
    let image =
        image_whose_first_file_is_a_whole_file_hole("tag-zero-upper-leaf-kept-past-that-leaf");
    let inode_past_the_first_upper_leaf = InodeNumber(INODES_PER_UPPER_LEAF);
    let content = vec![5u8; 100];
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::NewInodes {
                count: inode_past_the_first_upper_leaf.0 - 1,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::OverwriteOfAnInode {
                inode: inode_past_the_first_upper_leaf,
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    assert_ne!(
        unit_bytes_at(
            &after,
            the_slot_of(&tree_table_entry_of_kind(&after, TREE_KIND_EXTENT).root),
            node_bytes()
        )[INDEX_NODE_LEVEL_OFFSET],
        0,
        "前提：上段长高了一层（号 {} 在第一片叶之外）",
        inode_past_the_first_upper_leaf.0
    );
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "第一片上段叶连同标签 0 那一条留在新形状里"
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, SIZE_OF_THE_FIRST_FILE);
}

/// 截到 0（标签留 0、size 0），再建一个 inode、按号写 inode 2：inode 2 与第一个文件在同一片上段叶（根兼叶）里，这片叶重写，第一个文件那一条
/// 照写标签 0（size 0 也写，用户 2026-10-24 定「甲」）；四处读者读第一个文件读出 0 字节。改之前写者按 size 0 不写那一条：冷走读、可写重挂、
/// 冷恢复都拒成「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」。
#[test]
fn a_whole_file_hole_of_tag_zero_truncated_to_zero_keeps_its_entry_when_the_session_rewrites_its_upper_leaf_for_inode_two(
) {
    let image = image_whose_first_file_is_a_whole_file_hole(
        "tag-zero-truncated-to-zero-upper-leaf-rewritten-for-inode-two",
    );
    let content_of_inode_two = vec![9u8; 100];
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 0,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(2),
                content: &content_of_inode_two,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, 0);
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "重写的那片上段叶里 size 0 的第一个文件那一条照写标签 0"
    );
}

/// 截到 0，再建到第一片上段叶之外的号（143）、按号写它：上段长高一层，第一个文件那片叶留在新形状里、标签 0 那一条留着；四处读者读第一个文件
/// 读出 0 字节。改之前有条目的号集合里 size 0 的第一个文件不算，那片叶不在新形状里、整片换下，冷走读、可写重挂、冷恢复都拒。
#[test]
fn a_whole_file_hole_of_tag_zero_truncated_to_zero_keeps_its_upper_leaf_when_the_session_writes_an_inode_past_that_leaf(
) {
    let image = image_whose_first_file_is_a_whole_file_hole(
        "tag-zero-truncated-to-zero-upper-leaf-kept-past-that-leaf",
    );
    let inode_past_the_first_upper_leaf = InodeNumber(INODES_PER_UPPER_LEAF);
    let content = vec![5u8; 100];
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 0,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::NewInodes {
                count: inode_past_the_first_upper_leaf.0 - 1,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::OverwriteOfAnInode {
                inode: inode_past_the_first_upper_leaf,
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    assert_ne!(
        unit_bytes_at(
            &after,
            the_slot_of(&tree_table_entry_of_kind(&after, TREE_KIND_EXTENT).root),
            node_bytes()
        )[INDEX_NODE_LEVEL_OFFSET],
        0,
        "前提：上段长高了一层（号 {} 在第一片叶之外）",
        inode_past_the_first_upper_leaf.0
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, 0);
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "第一片上段叶连同 size 0 的第一个文件那一条标签 0 留在新形状里"
    );
}

/// 截到 0，再建一个 inode、按号写 inode 2（那片上段叶重写），再把第一个文件变长到 7000：变长只改记录、标签留 0；四处读者读第一个文件读出
/// 7000 个 0。改之前按号写 inode 2 那一次丢掉第一个文件那一条，变长之后挂载态读者也拒（没有条目而 size > 0）。
#[test]
fn a_whole_file_hole_of_tag_zero_truncated_to_zero_then_extended_after_its_upper_leaf_is_rewritten_reads_back_zeros(
) {
    /// 截到 0 之后变长到的 size（不到一个净荷）。
    const EXTENDED_SIZE_IN_BYTES: u64 = 7000;
    let image = image_whose_first_file_is_a_whole_file_hole(
        "tag-zero-truncated-to-zero-then-extended-after-its-upper-leaf-is-rewritten",
    );
    let content_of_inode_two = vec![9u8; 100];
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 0,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(2),
                content: &content_of_inode_two,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: EXTENDED_SIZE_IN_BYTES,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    every_reader_reads_the_first_file_as_a_whole_file_hole_of(&after, EXTENDED_SIZE_IN_BYTES);
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        Some(ExtentUpperLeafTarget::NoDataUnit),
        "变长之后第一个文件那一条仍是标签 0"
    );
}

/// 截到 0，再删第一个文件：删完的那一版 extent 树上段里没有第一个文件那一条（有条目的号集合按记录在不在算，删除计划照它换下那片叶）；
/// 冷走读、冷恢复读到没有文件，可写重挂重建出有树、文件集合为空的一版。改之前删除计划按 size 0 不把第一个文件算进有条目的号集合，
/// 那片叶照抄、标签 0 那一条留成孤条目。
#[test]
fn a_whole_file_hole_of_tag_zero_truncated_to_zero_then_unlinked_leaves_no_entry_in_the_upper_leaf()
{
    let image =
        image_whose_first_file_is_a_whole_file_hole("tag-zero-truncated-to-zero-then-unlinked");
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 0,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        None,
        "删完的那一版上段叶不留第一个文件那一条"
    );
    let root = newest_root(&after);
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(&after, &root, &mut mapping_fallbacks),
        Ok(None),
        "冷走读读到没有文件"
    );
    assert!(
        matches!(
            recover(&after, JournalPolicy::Consult).outcome,
            RecoveryOutcome::NoFile { .. }
        ),
        "冷恢复读到没有文件"
    );
    let mut devices = writable_devices_of(&after);
    let remounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|refusal| panic!("可写重挂收下删完的那一版，实际拒 {refusal:?}"));
    assert!(
        matches!(remounted.current, PoolVersion::WithTreesAndNoFile(_)),
        "可写重挂重建出有树、文件集合为空的一版"
    );
}

/// 建到第一片上段叶之外的号（143），删第一个文件（标签 0、size 3000），再按号写 143：有条目的号集合按记录在不在算，只剩 143，上段长高一层、
/// 根下只有罩 143 起的那一片叶，第一片叶不在形状里；冷走读、冷恢复读到没有文件，可写重挂收下这一版，挂载态读者读 143 读回写进去的内容。
/// 143 在删之前建：删完之后才建的号，今天的写者拒写它的数据（`DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`）。
/// 算上段形状的那一处把记录没了的第一个文件也算进去时（判决 `research/prompts/m4-validate-code-r3-main-verification.md` 第二节 T5、攻方变异 6），
/// 第一片叶留在形状里、成了零条目的非根叶，这一条红在形状那一条断言上（攻方在新池新建文件的起点上量到这一形冷走读、冷恢复、可写重挂
/// 都拒成「extent 树节点一条条目都没有」）。
#[test]
fn a_whole_file_hole_of_tag_zero_unlinked_then_an_inode_past_its_upper_leaf_written_leaves_that_leaf_out_of_the_shape(
) {
    let image = image_whose_first_file_is_a_whole_file_hole(
        "tag-zero-unlinked-then-an-inode-past-its-upper-leaf-written",
    );
    let inode_past_the_first_upper_leaf = InodeNumber(INODES_PER_UPPER_LEAF);
    let content = vec![5u8; 100];
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::NewInodes {
                count: inode_past_the_first_upper_leaf.0 - 1,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::OverwriteOfAnInode {
                inode: inode_past_the_first_upper_leaf,
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    assert_ne!(
        unit_bytes_at(
            &after,
            the_slot_of(&tree_table_entry_of_kind(&after, TREE_KIND_EXTENT).root),
            node_bytes()
        )[INDEX_NODE_LEVEL_OFFSET],
        0,
        "前提：上段长高了一层（号 {} 在第一片叶之外）",
        inode_past_the_first_upper_leaf.0
    );
    let leaves = leaves_of_the_extent_upper_segment(&after);
    assert_eq!(
        leaves
            .iter()
            .map(|leaf| leaf.first_inode_covered)
            .collect::<Vec<_>>(),
        vec![INODES_PER_UPPER_LEAF],
        "上段根下只有罩 {INODES_PER_UPPER_LEAF} 起的那一片叶，罩号 0 起的第一片叶不在形状里"
    );
    assert_eq!(
        first_file_entry_of_the_extent_upper_segment(&after),
        None,
        "上段里没有第一个文件那一条"
    );
    every_reader_reads_no_first_file_and_reads_back_the_other_inode(
        &after,
        inode_past_the_first_upper_leaf,
        &content,
    );
}

/// 截到 0，再建一个 inode、按号写 inode 2，再删第一个文件：删完的那一版上段叶（根兼叶）里只剩 inode 2 那一条、内联它那一个数据单元，
/// 第一个文件那一条没有；冷走读、冷恢复读到没有文件，可写重挂收下这一版，挂载态读者读 inode 2 读回写进去的内容。记录没了也写标签 0、或装上段叶
/// 的那一处不看记录一律写第一个文件那一条时（判决 `research/prompts/m4-validate-code-r3-main-verification.md` 第二节 T5、辩方第 2 条），
/// 这片叶多出第一个文件那一条标签 0。
#[test]
fn a_whole_file_hole_of_tag_zero_truncated_to_zero_then_unlinked_beside_inode_two_leaves_only_the_entry_of_inode_two(
) {
    let image = image_whose_first_file_is_a_whole_file_hole(
        "tag-zero-truncated-to-zero-then-unlinked-beside-inode-two",
    );
    let inode_two = InodeNumber(2);
    let content_of_inode_two = vec![9u8; 100];
    let after = image_after_the_session_publishes(
        &image,
        vec![
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 0,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::OverwriteOfAnInode {
                inode: inode_two,
                content: &content_of_inode_two,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            UserChange::Unlink {
                inode: FIRST_FILE,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        ],
    );
    let leaves = leaves_of_the_extent_upper_segment(&after);
    assert_eq!(
        leaves
            .iter()
            .map(|leaf| (
                leaf.first_inode_covered,
                leaf.entries
                    .iter()
                    .map(|entry| entry.inode)
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>(),
        vec![(0, vec![inode_two.0])],
        "上段只有根兼叶那一片，叶里只剩 inode 2 那一条，没有第一个文件那一条"
    );
    assert!(
        matches!(
            leaves[0].entries[0].target,
            ExtentUpperLeafTarget::InlineDataUnit(_)
        ),
        "inode 2 那一条内联它那一个数据单元，实际 {:?}",
        leaves[0].entries[0].target
    );
    every_reader_reads_no_first_file_and_reads_back_the_other_inode(
        &after,
        inode_two,
        &content_of_inode_two,
    );
}
