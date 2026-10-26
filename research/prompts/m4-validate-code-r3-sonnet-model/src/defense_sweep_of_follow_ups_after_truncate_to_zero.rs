//! 辩方腿在自己副本里的后续序列扫描：标签 0 镜像可写挂载之后，一串用户改动（可分几次挂载），每串做完后四处读者与池级 checker 各怎么判。
#![allow(dead_code, clippy::too_many_lines)]
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, parameters};
use common_deletion_in_progress::change_the_unit_and_propagate_its_checksum;
use common_pointer_head_corruption::{
    first_upper_leaf_entry, newest_root, node_bytes, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at, data_unit_bytes,
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

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::mounted_read::MountedPoolForRead;
use singlefs_core::transaction::FirstFile;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// 一步用户改动，自己持有内容。
#[derive(Clone)]
enum Step {
    Truncate(u64),
    NewInodes(u64),
    OverwriteInode(u64, usize),
    SequentialWriteInode(u64, usize),
    Unlink(u64),
    Snapshot,
    OverwriteFirstFile(usize),
}

fn fnv_of_the_image(image: &MemoryPool) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for (identity, device) in &image.devices {
        for byte in identity.0.to_le_bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
        for (sector, bytes) in device.written_sectors() {
            for byte in sector.to_le_bytes().iter().chain(bytes.iter()) {
                hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    hash
}

fn short(text: String) -> String {
    let flat: String = text.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    flat.chars().take(150).collect()
}

/// 一次会话（一次可写挂载）里按次序发 `steps`；某步被拒就停，交回已发成的盘与拒绝说明。
fn run_one_session(image: &MemoryPool, steps: &[Step]) -> (MemoryPool, Option<String>) {
    let contents: Vec<Vec<u8>> = steps
        .iter()
        .map(|step| match step {
            Step::OverwriteInode(_, length)
            | Step::SequentialWriteInode(_, length)
            | Step::OverwriteFirstFile(length) => vec![7u8; *length],
            _ => Vec::new(),
        })
        .collect();
    let mut devices = writable_devices_of(image);
    let mounted = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => {
            return (
                image.clone(),
                Some(short(format!("可写挂载拒:{refusal:?}"))),
            )
        }
    };
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    for (step, content) in steps.iter().zip(contents.iter()) {
        let change = match step {
            Step::Truncate(new_size) => UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: *new_size,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            Step::NewInodes(count) => UserChange::NewInodes {
                count: *count,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            Step::OverwriteInode(inode, _) => UserChange::OverwriteOfAnInode {
                inode: InodeNumber(*inode),
                content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            Step::SequentialWriteInode(inode, _) => UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(*inode),
                content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            Step::Unlink(inode) => UserChange::Unlink {
                inode: InodeNumber(*inode),
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            Step::Snapshot => UserChange::CreateSnapshot,
            Step::OverwriteFirstFile(_) => UserChange::Overwrite(FirstFile {
                content,
                write_time_seconds: WRITE_TIME_SECONDS,
            }),
        };
        if let Err(refusal) = session.publish_user_change(&mut devices, change) {
            return (
                image_of_the_devices(&devices, image.device_size_in_bytes),
                Some(short(format!("会话拒:{refusal:?}"))),
            );
        }
    }
    (image_of_the_devices(&devices, image.device_size_in_bytes), None)
}

/// 四处读者与池级 checker 各自的答案，一行。
fn verdicts_of_the_readers(image: &MemoryPool) -> String {
    let root = newest_root(image);
    let mut fallbacks = 0;
    let cold_walk = match walk_to_file(image, &root, &mut fallbacks) {
        Ok(Some(content)) => format!(
            "收{}B{}",
            content.len(),
            if content.iter().all(|byte| *byte == 0) { "全0" } else { "有非0" }
        ),
        Ok(None) => "收(无文件)".to_string(),
        Err(failure) => short(format!("拒{failure:?}")),
    };
    let cold_recovery = match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => format!(
            "收{}B{}",
            content.len(),
            if content.iter().all(|byte| *byte == 0) { "全0" } else { "有非0" }
        ),
        RecoveryOutcome::NoFile { .. } => "收(无文件)".to_string(),
        RecoveryOutcome::Failed { .. } => "拒".to_string(),
    };
    let mut devices = writable_devices_of(image);
    let writable_remount = match mount_writable(&parameters(), &mut devices) {
        Ok(remounted) => match remounted.current {
            PoolVersion::WithFile(rebuilt) => format!(
                "收(size {:?},{}单元)",
                rebuilt.inode_record.map(|record| record.size),
                rebuilt.data_pointers.len()
            ),
            _ => "收(无文件版本)".to_string(),
        },
        Err(refusal) => short(format!("拒{refusal:?}")),
    };
    let mounted_state = match mount_read_only(image) {
        Ok(read_only) => reading_of_the_mounted_state(image, &read_only.mounted),
        Err(refusal) => short(format!("只读挂载拒{refusal:?}")),
    };
    let violated: Vec<&str> = check_pool_image(image)
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    format!(
        "冷走读={cold_walk} | 冷恢复={cold_recovery} | 可写重挂={writable_remount} | 挂载态={mounted_state} | checker红={violated:?}"
    )
}

fn reading_of_the_mounted_state(image: &MemoryPool, mounted: &MountedPoolForRead) -> String {
    match mounted.open_file(image, FIRST_FILE) {
        Ok(file) => {
            let size = file.inode_record().size;
            match file.read_at(image, FileOffsetInBytes(0), size) {
                Ok(read) => format!(
                    "收{}B{}/{}单元",
                    read.bytes.len(),
                    if read.bytes.iter().all(|byte| *byte == 0) { "全0" } else { "有非0" },
                    file.data_unit_count()
                ),
                Err(failure) => short(format!("读拒{failure:?}")),
            }
        }
        Err(failure) => short(format!("打开拒{failure:?}")),
    }
}


fn histories() -> Vec<(&'static str, Vec<Vec<Step>>)> {
    use Step::{
        NewInodes, OverwriteFirstFile, OverwriteInode, SequentialWriteInode, Snapshot, Truncate,
        Unlink,
    };
    let payload = usize::try_from(payload_capacity_in_bytes()).expect("净荷");
    vec![
        ("00 截到0，不再动", vec![vec![Truncate(0)]]),
        ("01 截到0，建1个，覆盖写号2", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100)]]),
        ("02 截到0，建1个，顺序写号2", vec![vec![Truncate(0), NewInodes(1), SequentialWriteInode(2, 100)]]),
        ("03 截到0，建142个，覆盖写号143(上段长高)", vec![vec![Truncate(0), NewInodes(142), OverwriteInode(143, 100)]]),
        ("04 截到0，建1个，写号2，再删号2", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100), Unlink(2)]]),
        ("05 截到0，建1个，写号2，变长到7000", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100), Truncate(7000)]]),
        ("06 截到0，建1个，写号2，变长到3个净荷", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100), Truncate(3 * payload as u64)]]),
        ("07 截到0，删第一个文件", vec![vec![Truncate(0), Unlink(1)]]),
        ("08 截到0，建1个，写号2，删第一个文件", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100), Unlink(1)]]),
        ("09 截到0，建只读快照", vec![vec![Truncate(0), Snapshot]]),
        ("10 截到0，只建1个inode", vec![vec![Truncate(0), NewInodes(1)]]),
        ("11 截到0，再截到0", vec![vec![Truncate(0), Truncate(0)]]),
        ("12 截到0，变长到7000，建1个，写号2", vec![vec![Truncate(0), Truncate(7000), NewInodes(1), OverwriteInode(2, 100)]]),
        ("13 截到0，覆盖写第一个文件", vec![vec![Truncate(0), OverwriteFirstFile(100)]]),
        ("14 截到0，建3个，写号2、号3，删号2", vec![vec![Truncate(0), NewInodes(3), OverwriteInode(2, 100), OverwriteInode(3, 100), Unlink(2)]]),
        ("15 截到0，建1个，写号2，建快照", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100), Snapshot]]),
        ("16 截到0后换一次挂载，建1个，写号2", vec![vec![Truncate(0)], vec![NewInodes(1), OverwriteInode(2, 100)]]),
        ("17 截到0，建1个，写号2，换一次挂载，写号2", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100)], vec![OverwriteInode(2, 50)]]),
        ("18 截到0，建1个，写号2，换一次挂载，变长到7000", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100)], vec![Truncate(7000)]]),
        ("19 截到0，建1个，写号2，换一次挂载，删第一个文件", vec![vec![Truncate(0), NewInodes(1), OverwriteInode(2, 100)], vec![Unlink(1)]]),
        ("20 对照：不截，建1个，写号2(size 3000)", vec![vec![NewInodes(1), OverwriteInode(2, 100)]]),
        ("21 对照：截到100，建1个，写号2", vec![vec![Truncate(100), NewInodes(1), OverwriteInode(2, 100)]]),
        ("22 对照：不截，删第一个文件", vec![vec![Unlink(1)]]),
        ("23 对照：不截，建142个，写号143", vec![vec![NewInodes(142), OverwriteInode(143, 100)]]),
        ("24 对照：变长到7000后建1个写号2", vec![vec![Truncate(7000), NewInodes(1), OverwriteInode(2, 100)]]),
    ]
}

/// 扫描：每串后续各打一行。`SWEEP_START` 取 `hole3000`（手造标签 0、size 3000，带那一条手造泄漏）。
#[test]
fn sweep_of_follow_ups_after_truncate_to_zero() {
    let start = match std::env::var("SWEEP_START_IMAGE") {
        Ok(path) => load_image(&path),
        Err(_) => image_whose_first_file_is_a_whole_file_hole("defense-sweep-start"),
    };
    println!(
        "SWEEP name=起点 hash={:016x} {}",
        fnv_of_the_image(&start),
        verdicts_of_the_readers(&start)
    );
    for (name, sessions) in histories() {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let mut image = start.clone();
            let mut refusals: Vec<String> = Vec::new();
            for steps in &sessions {
                let (after, refusal) = run_one_session(&image, steps);
                image = after;
                if let Some(refusal) = refusal {
                    refusals.push(refusal);
                    break;
                }
            }
            (image, refusals)
        }));
        match outcome {
            Ok((image, refusals)) => println!(
                "SWEEP name={name} 会话拒={refusals:?} hash={:016x} {}",
                fnv_of_the_image(&image),
                verdicts_of_the_readers(&image)
            ),
            Err(_) => println!("SWEEP name={name} 会话里panic"),
        }
    }
}

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_harness::memory_pool::SparseDevice;

/// 镜像存成文本：`device <号>` 起一块盘，之后每行 `<扇区号> <十六进制 512 字节>`；开头一行 `size <每盘字节数>`。
fn dump_image(image: &MemoryPool, path: &str) {
    use std::fmt::Write as _;
    let mut text = String::new();
    writeln!(text, "size {}", image.device_size_in_bytes).expect("写内存字符串");
    for (identity, device) in &image.devices {
        writeln!(text, "device {}", identity.0).expect("写内存字符串");
        for (sector, bytes) in device.written_sectors() {
            let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
            writeln!(text, "{sector} {hex}").expect("写内存字符串");
        }
    }
    std::fs::write(path, text).expect("写镜像文件");
}

fn load_image(path: &str) -> MemoryPool {
    let text = std::fs::read_to_string(path).expect("读镜像文件");
    let mut devices: std::collections::BTreeMap<DeviceIdentity, SparseDevice> =
        std::collections::BTreeMap::new();
    let mut size = 0u64;
    let mut current: Option<DeviceIdentity> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("size ") {
            size = rest.parse().expect("每盘字节数");
        } else if let Some(rest) = line.strip_prefix("device ") {
            let identity = DeviceIdentity(rest.parse().expect("盘号"));
            devices.insert(identity, SparseDevice::default());
            current = Some(identity);
        } else {
            let (sector, hex) = line.split_once(' ').expect("扇区行");
            let sector: u64 = sector.parse().expect("扇区号");
            let bytes: Vec<u8> = (0..hex.len() / 2)
                .map(|index| u8::from_str_radix(&hex[2 * index..2 * index + 2], 16).expect("十六进制"))
                .collect();
            devices
                .get_mut(&current.expect("先有盘"))
                .expect("盘在")
                .write(DeviceOffsetInBytes(sector * 512), &bytes);
        }
    }
    MemoryPool {
        devices,
        device_size_in_bytes: size,
    }
}

/// 生成：写路径做成的第一个文件（3000 字节、内联一个单元）可写挂载，会话里截到 0，存到 `GENERATE_IMAGE_PATH`。
/// 只在放宽了「截断留至少一个单元」的副本里做得成（产品代码拒这一步 panic）；做成的镜像没有手造泄漏。
#[test]
fn generate_the_image_of_a_first_file_truncated_to_zero_units_by_the_writer() {
    let Ok(path) = std::env::var("GENERATE_IMAGE_PATH") else {
        return;
    };
    let pool = build_pool("defense-generate");
    let image = pool.memory_pool();
    println!(
        "GENERATE 起点(写路径做成的 3000 字节文件) {}",
        verdicts_of_the_readers(&image)
    );
    let ExtentUpperLeafTarget::InlineDataUnit(old_data_unit) = first_upper_leaf_entry(&image).target
    else {
        panic!("前提：写路径做成的第一个文件是标签 2、内联一个单元")
    };
    let (mut after, refusal) = run_one_session(&image, &[Step::Truncate(0)]);
    println!("GENERATE 拒绝={refusal:?} 截完未抹旧单元 {}", verdicts_of_the_readers(&after));
    dump_image(&after, &path);
    // 旧单元已释放、不再被任何树引用，它的字节还留在盘上，扫描重建会把它认成第一个文件的单元；
    // 外来写者写出的这一形从来没有过那个单元，所以把它的字节抹成零（两盘各一份）。
    for location in &old_data_unit.locations {
        after
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .zero_fill(
                location.slot.to_device_offset(),
                u64::try_from(data_unit_bytes()).expect("单元宽"),
            );
    }
    println!("GENERATE 抹旧单元之后 {}", verdicts_of_the_readers(&after));
    for (name, verdict) in check_pool_image(&after) {
        if let InvariantVerdict::Violated(detail) = verdict {
            println!("GENERATE 违例 {name}: {}", short(detail));
        }
    }
    dump_image(&after, &format!("{path}.zeroed"));
}
