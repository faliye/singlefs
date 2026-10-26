//! 攻方腿 m4-validate-code-r3 的探针（只在腿的副本里）：从几种起点镜像放开扫用户动作序列，每一版核上段形状与盘上条目、第一个文件的形态，
//! 序列末尾四处读者与池级 checker；同一段序列在改法甲（变异 0）与改回旧规则（变异 1）上各跑一遍，比写出的字节。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_deletion_in_progress::change_the_unit_and_propagate_its_checksum;
use common_pointer_head_corruption::{
    first_upper_leaf_entry, newest_root, node_bytes, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::extent_tree::r3_probe;
use singlefs_core::extent_tree::{ExtentUpperLeafEntry, ExtentUpperLeafTarget};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::{parse_mapping_entry, TREE_KIND_EXTENT};
use singlefs_core::recovery::{recover, walk_to_file, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const WRITE_TIME_SECONDS: u64 = 1_788_000_300;
const INDEX_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_RANGE_OFFSET: usize = 52;
const EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET: usize = 8;
const EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET: usize = 24;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn payload() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

fn devices_of(image: &MemoryPool) -> Devices {
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

fn image_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 内存稀疏盘上 mkfs → 取号 → 暖机 → 新池新建文件（3000 字节、标签 2），同 `common::build_pool`，只是不建文件镜像。
fn normal_pool() -> MemoryPool {
    let parameters = parameters();
    let mut devices: Devices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let content = file_content();
    {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    image_of(&devices)
}

/// 照 `whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs` 的同名办法改一个码 2 节点的条目（中央映射树 key 区间跟着条目改）。
fn rewrite_entries(image: &mut MemoryPool, slot: SlotNumber, follows: bool, change: impl FnOnce(&mut Vec<Vec<u8>>)) {
    let parsed = parse_index_node(&unit_bytes_at(image, slot, node_bytes())).expect("节点解得开");
    let key_width = parsed.key_width;
    let entry_width = parsed.entry_width;
    let mut entries = parsed.entries;
    change(&mut entries);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let entries_start = 115 + 2 * key_width;
        node[entries_start..].fill(0);
        for (position, entry) in entries.iter().enumerate() {
            let start = entries_start + position * entry_width;
            node[start..start + entry_width].copy_from_slice(entry);
        }
        let count_offset = 82 + 2 * key_width;
        node[count_offset..count_offset + 2]
            .copy_from_slice(&u16::try_from(entries.len()).expect("条目数").to_le_bytes());
        node[INDEX_NODE_DECLARED_LENGTH_OFFSET..INDEX_NODE_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
            &u16::try_from(entries.len() * entry_width).expect("声明长度").to_le_bytes(),
        );
        if let (true, Some(first), Some(last)) = (follows, entries.first(), entries.last()) {
            let smallest = INDEX_NODE_KEY_RANGE_OFFSET;
            let largest = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
            node[smallest..smallest + key_width].copy_from_slice(&first[..key_width]);
            node[largest..largest + key_width].copy_from_slice(&last[..key_width]);
        }
    });
}

/// 第二轮两条腿与修补一用例那一种手造起点：上段叶条目改成标签 0、映射条目拿掉，分配记录不动（带一条已分配无引用的泄漏）。
fn hand_made_tag_zero(normal: &MemoryPool) -> MemoryPool {
    let mut image = normal.clone();
    let ExtentUpperLeafTarget::InlineDataUnit(data_unit) = first_upper_leaf_entry(&image).target else {
        panic!("前提：标签 2")
    };
    let mapping_root_slot = the_slot_of(&newest_root(&image).mapping_root);
    rewrite_entries(&mut image, mapping_root_slot, true, |entries| {
        entries.retain(|entry| {
            let (_key, locations) = parse_mapping_entry(entry).expect("映射条目");
            locations[0].slot != data_unit.locations[0].slot
        });
    });
    let extent_root_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root);
    rewrite_entries(&mut image, extent_root_slot, false, |entries| {
        let entry = entries
            .iter_mut()
            .find(|entry| {
                u64::from_le_bytes(
                    entry[EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET..EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET + 8]
                        .try_into()
                        .expect("8"),
                ) == FIRST_INODE_NUMBER
            })
            .expect("第一个文件那一条");
        entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET] = 0;
        entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET + 1..].fill(0);
    });
    image
}

/// 副本里的生成器开关打开（截断最后一批一个单元都不留），由写者自己写出标签 0、size `size` 的第一个文件：分配记录与记账都由写者算，
/// 不带手造泄漏。`size` > 3000 时先截到 100 再变长（变长不加单元）。交回之前开关关上。
fn generated_tag_zero(normal: &MemoryPool, size: u64, scrub: bool) -> MemoryPool {
    let mut devices = devices_of(normal);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let mut dropped: Vec<singlefs_core::pointer::DataPointer> = match &mounted.current {
        PoolVersion::WithFile(version) => version.data_pointers.clone(),
        _ => Vec::new(),
    };
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    r3_probe::set_truncation_keeps_no_unit(true);
    let first_target = if size >= 3000 { 100 } else { size };
    let truncated = session.publish_user_change(
        &mut devices,
        UserChange::Truncate {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            new_size: first_target,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
    );
    r3_probe::set_truncation_keeps_no_unit(false);
    for version in truncated.expect("生成器截断做成").versions_published {
        dropped.extend(version.data_pointers.iter().copied());
    }
    if size != first_target {
        session
            .publish_user_change(
                &mut devices,
                UserChange::Truncate {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    new_size: size,
                    write_time_seconds: WRITE_TIME_SECONDS,
                },
            )
            .expect("变长做成");
    }
    let mut image = image_of(&devices);
    if scrub {
        // 放掉的单元在盘上的字节清零：扫描重建（读法 Z）不再从盘上认回它们。
        let unit_bytes = singlefs_format::DATA_UNIT_BYTES;
        for pointer in &dropped {
            for location in pointer.locations {
                image
                    .devices
                    .get_mut(&location.device)
                    .expect("盘")
                    .zero_fill(location.slot.to_device_offset(), unit_bytes);
            }
        }
    }
    assert_eq!(first_upper_leaf_entry(&image).target, ExtentUpperLeafTarget::NoDataUnit, "生成器写出标签 0");
    image
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Truncate(u64),
    NewInodes(u64),
    OverwriteInode(u64),
    OverwriteFirst(usize),
    SequentialFirst(usize),
    Unlink(u64),
    Snapshot,
    RollBackToTheStart,
    Remount,
}

fn menu() -> Vec<Step> {
    let p = usize::try_from(payload()).expect("p");
    vec![
        Step::Truncate(0),
        Step::Truncate(100),
        Step::Truncate(3000),
        Step::Truncate(7000),
        Step::Truncate(3 * payload()),
        Step::NewInodes(1),
        Step::NewInodes(142),
        Step::OverwriteInode(2),
        Step::OverwriteInode(143),
        Step::OverwriteFirst(100),
        Step::SequentialFirst(p + 500),
        Step::Unlink(1),
        Step::Unlink(2),
        Step::Snapshot,
        Step::RollBackToTheStart,
        Step::Remount,
    ]
    .into_iter()
    // 一批一个单元那一档另加一步写五个单元：删除与截断分几批，中间那几版都过一遍核。
    .chain(std::env::var("R3_PROBE_BATCH_ONE").is_ok().then_some(Step::SequentialFirst(4 * p + 10)))
    .collect()
}

/// 从一条上段根指针往下读，交回盘上有条目的 inode 号（节点的诞生代号、序号与指针不符时交 `None`：那个槽后来被别的单元占了）。
fn entries_on_disk_below(image: &MemoryPool, pointer: &singlefs_core::pointer::NodePointer) -> Option<BTreeSet<u64>> {
    let node = parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes())).ok()?;
    if node.birth_txg != pointer.head.birth_txg || node.birth_sequence != pointer.birth_sequence {
        return None;
    }
    let mut inodes = BTreeSet::new();
    if node.level == 0 {
        for entry in &node.entries {
            inodes.insert(ExtentUpperLeafEntry::parse(entry).ok()?.inode);
        }
        return Some(inodes);
    }
    for entry in &node.entries {
        let child = singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            entry,
            node.key_width,
        ));
        inodes.extend(entries_on_disk_below(image, &child)?);
    }
    Some(inodes)
}

fn upper_root_pointer(version: &TransactionOutput) -> singlefs_core::pointer::NodePointer {
    version.extent_tree.upper_nodes.last().expect("上段有根").1
}

/// 一段历史的观测。
#[derive(Default, Clone)]
struct Observed {
    /// 第一个文件形态：有记录、0 个单元、size 0 / size > 0 的版本数。
    record_zero_units_size_zero: u64,
    record_zero_units_size_positive: u64,
    /// 内存里的上段形状集合与盘上条目集合不等的版本（版本名列出）。
    shape_mismatches: Vec<String>,
    unverifiable_versions: u64,
    /// C900 的判据：第一个文件有记录而盘上上段没有它的条目。
    record_without_entry: Vec<String>,
    /// 第一个文件没有记录而上段有它的条目（孤条目）。
    entry_without_record: Vec<String>,
    refused: Option<String>,
    steps_done: usize,
    versions_seen: u64,
}

fn observe_version(image: &MemoryPool, version: &TransactionOutput, name: &str, observed: &mut Observed) {
    observed.versions_seen += 1;
    let units = version.data_pointers.len();
    match (version.inode_record.map(|record| record.size), units) {
        (Some(0), 0) => observed.record_zero_units_size_zero += 1,
        (Some(_), 0) => observed.record_zero_units_size_positive += 1,
        _ => {}
    }
    let Some(on_disk) = entries_on_disk_below(image, &upper_root_pointer(version)) else {
        observed.unverifiable_versions += 1;
        return;
    };
    let in_memory = version.inodes_with_an_extent_upper_leaf_entry();
    if in_memory != on_disk {
        observed
            .shape_mismatches
            .push(format!("{name}: memory={in_memory:?} disk={on_disk:?}"));
    }
    let has_record = version.inode_record.is_some();
    let has_entry = on_disk.contains(&FIRST_INODE_NUMBER);
    if has_record && !has_entry {
        observed.record_without_entry.push(format!(
            "{name}: size={:?} units={units}",
            version.inode_record.map(|record| record.size)
        ));
    }
    if !has_record && has_entry {
        observed.entry_without_record.push(name.to_string());
    }
}

fn current_with_trees(session: &MountedSession) -> Option<&TransactionOutput> {
    match &session.current {
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => Some(version),
        PoolVersion::WithoutFile(_) => None,
    }
}

/// 第一个文件按用户动作该有的内容（`None` 是删了）。
fn expected_after(expected: &Option<Vec<u8>>, step: Step, start_content: &Option<Vec<u8>>) -> Option<Vec<u8>> {
    match step {
        Step::Truncate(size) => expected.clone().map(|mut content| {
            content.resize(usize::try_from(size).expect("size"), 0);
            content
        }),
        Step::OverwriteFirst(length) => expected.as_ref().map(|_| vec![0x5a; length]),
        Step::SequentialFirst(length) => expected.as_ref().map(|_| vec![0x3c; length]),
        Step::Unlink(1) => None,
        Step::RollBackToTheStart => start_content.clone(),
        Step::NewInodes(_) | Step::OverwriteInode(_) | Step::Unlink(_) | Step::Snapshot | Step::Remount => {
            expected.clone()
        }
    }
}

struct Start {
    name: &'static str,
    image: MemoryPool,
    content: Option<Vec<u8>>,
    rollback_target: RollbackTarget,
    baseline_violations: BTreeSet<String>,
}

fn mount_session(devices: &mut Devices) -> Result<MountedSession, String> {
    let mounted = mount_writable(&parameters(), devices).map_err(|error| format!("{error:?}"))?;
    let mut session = MountedSession::of_the_mount(mounted).1;
    if std::env::var("R3_PROBE_BATCH_ONE").is_ok() {
        let mut switches = session.allocator.deletion_switches();
        switches.batch_size = singlefs_core::unlink::UnlinkBatchSize::CappedForTests(
            singlefs_core::unlink::UnlinkBatchDataUnitsCappedForTests::new(1).expect("1"),
        );
        session.allocator.set_deletion_switches(switches);
    }
    Ok(session)
}

fn apply_step(session: &mut MountedSession, devices: &mut Devices, step: Step, start: &Start) -> Result<Vec<TransactionOutput>, String> {
    let content_inode = vec![9u8; 100];
    let content_first_overwrite = vec![0x5au8; 100];
    let content_first_sequential: Vec<u8> = match step {
        Step::SequentialFirst(length) => vec![0x3c; length],
        _ => Vec::new(),
    };
    let content_first = match step {
        Step::OverwriteFirst(length) => vec![0x5a; length],
        _ => content_first_overwrite,
    };
    let change = match step {
        Step::Truncate(size) => UserChange::Truncate {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            new_size: size,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        Step::NewInodes(count) => UserChange::NewInodes {
            count,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        Step::OverwriteInode(inode) => UserChange::OverwriteOfAnInode {
            inode: InodeNumber(inode),
            content: &content_inode,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        Step::OverwriteFirst(_) => UserChange::Overwrite(FirstFile {
            content: &content_first,
            write_time_seconds: WRITE_TIME_SECONDS,
        }),
        Step::SequentialFirst(_) => UserChange::SequentialWrite(FirstFile {
            content: &content_first_sequential,
            write_time_seconds: WRITE_TIME_SECONDS,
        }),
        Step::Unlink(inode) => UserChange::Unlink {
            inode: InodeNumber(inode),
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        Step::Snapshot => UserChange::CreateSnapshot,
        Step::RollBackToTheStart => {
            let PoolVersion::WithFile(mut current) = session.current.clone() else {
                return Err("回退：现行那一版不带文件".to_string());
            };
            roll_back_by_a_forward_publish(&parameters(), devices, &mut session.allocator, &mut current, start.rollback_target)
                .map_err(|error| format!("回退拒 {error:?}"))?;
            session.current = PoolVersion::WithFile(current.clone());
            return Ok(vec![current]);
        }
        Step::Remount => {
            *session = mount_session(devices).map_err(|error| format!("重挂拒 {error}"))?;
            return Ok(current_with_trees(session).into_iter().cloned().collect());
        }
    };
    let published = session
        .publish_user_change(devices, change)
        .map_err(|refusal| format!("{refusal:?}").chars().take(160).collect::<String>())?;
    Ok(published.versions_published)
}

/// 四处读者读第一个文件，交回一行判定（对得上的写 ok）。
fn readers_line(image: &MemoryPool, expected: &Option<Vec<u8>>) -> String {
    let mut line = String::new();
    let root = newest_root(image);
    let mut fallbacks = 0;
    let cold_walk = walk_to_file(image, &root, &mut fallbacks);
    let _ = write!(
        line,
        "cold_walk={} ",
        match &cold_walk {
            Ok(read) if read == expected => "ok".to_string(),
            Ok(read) => format!("content_differs(len={:?})", read.as_ref().map(Vec::len)),
            Err(failure) => format!("refused({failure:?})").chars().take(140).collect(),
        }
    );
    let recovered = match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } if Some(&content) == expected.as_ref() => "ok".to_string(),
        RecoveryOutcome::NoFile { .. } if expected.is_none() => "ok".to_string(),
        RecoveryOutcome::FileRead { content, .. } => format!("content_differs(len={})", content.len()),
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("refused({failure:?})").chars().take(140).collect(),
    };
    let _ = write!(line, "cold_recovery={recovered} ");
    let mut devices = devices_of(image);
    let remount = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => match (&mounted.current, expected) {
            (PoolVersion::WithFile(version), Some(content)) => {
                if version.inode_record.map(|record| record.size) == Some(u64::try_from(content.len()).expect("len")) {
                    "ok".to_string()
                } else {
                    format!("size_differs({:?})", version.inode_record.map(|record| record.size))
                }
            }
            (PoolVersion::WithTreesAndNoFile(_), None) => "ok".to_string(),
            (PoolVersion::WithFile(version), None) if version.inode_record.is_none() => "ok".to_string(),
            (other, _) => format!("shape_differs({})", match other {
                PoolVersion::WithFile(_) => "with_file",
                PoolVersion::WithTreesAndNoFile(_) => "trees_no_file",
                PoolVersion::WithoutFile(_) => "without_file",
            }),
        },
        Err(error) => format!("refused({error:?})").chars().take(140).collect(),
    };
    let _ = write!(line, "writable_remount={remount} ");
    let mounted_reader = match (mount_read_only(image), expected) {
        (Err(refusal), _) => format!("mount_refused({refusal:?})").chars().take(140).collect(),
        (Ok(_), None) => "ok".to_string(),
        (Ok(read_only), Some(content)) => match read_only.mounted.open_file(image, InodeNumber(FIRST_INODE_NUMBER)) {
            Err(failure) => format!("open_refused({failure:?})").chars().take(140).collect(),
            Ok(file) => match file.read_at(image, FileOffsetInBytes(0), u64::try_from(content.len()).expect("len")) {
                Ok(read) if &read.bytes == content => "ok".to_string(),
                Ok(read) => format!("content_differs(len={})", read.bytes.len()),
                Err(failure) => format!("read_refused({failure:?})").chars().take(140).collect(),
            },
        },
    };
    let _ = write!(line, "mounted_reader={mounted_reader}");
    line
}

fn violations(image: &MemoryPool) -> BTreeSet<String> {
    violation_details(image).into_keys().collect()
}

fn violation_details(image: &MemoryPool) -> BTreeMap<String, String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((name.to_string(), detail.chars().take(environment_number("R3_PROBE_DETAIL_CHARS", 400)).collect())),
            _ => None,
        })
        .collect()
}

struct HistoryResult {
    observed: Observed,
    final_image: MemoryPool,
    readers: String,
    new_violations: BTreeSet<String>,
}

fn run_history(start: &Start, steps: &[Step], mutant: u8, judge_the_end: bool) -> HistoryResult {
    r3_probe::set_mutant(mutant);
    let mut observed = Observed::default();
    let mut devices = devices_of(&start.image);
    let mut expected = start.content.clone();
    let mut session = mount_session(&mut devices).expect("起点可写挂载");
    for (position, step) in steps.iter().enumerate() {
        match apply_step(&mut session, &mut devices, *step, start) {
            Ok(versions) => {
                let image = image_of(&devices);
                for (index, version) in versions.iter().enumerate() {
                    observe_version(&image, version, &format!("step{position}.{step:?}.v{index}"), &mut observed);
                }
                expected = expected_after(&expected, *step, &start.content);
                observed.steps_done += 1;
            }
            Err(refusal) => {
                observed.refused = Some(format!("{step:?} => {}", refusal.chars().take(150).collect::<String>()));
                break;
            }
        }
    }
    let final_image = image_of(&devices);
    let (readers, new_violations) = if judge_the_end {
        let found = violations(&final_image);
        (
            readers_line(&final_image, &expected),
            found.difference(&start.baseline_violations).cloned().collect(),
        )
    } else {
        (String::new(), BTreeSet::new())
    };
    r3_probe::set_mutant(0);
    HistoryResult {
        observed,
        final_image,
        readers,
        new_violations,
    }
}

fn environment_number(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn sequences(length: usize) -> Vec<Vec<Step>> {
    let menu = menu();
    let mut all: Vec<Vec<Step>> = vec![Vec::new()];
    let mut layer: Vec<Vec<Step>> = vec![Vec::new()];
    for _ in 0..length {
        let mut next = Vec::new();
        for prefix in &layer {
            for step in &menu {
                let mut sequence = prefix.clone();
                sequence.push(*step);
                next.push(sequence);
            }
        }
        all.extend(next.iter().cloned());
        layer = next;
    }
    all
}

fn start_named(name: &'static str, image: MemoryPool, content: Option<Vec<u8>>) -> Start {
    let root = newest_root(&image);
    let baseline_violations = violations(&image);
    Start {
        name,
        rollback_target: RollbackTarget {
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
        },
        image,
        content,
        baseline_violations,
    }
}

fn after_truncating_to_zero(image: &MemoryPool) -> MemoryPool {
    let mut devices = devices_of(image);
    let mut session = mount_session(&mut devices).expect("可写挂载");
    session
        .publish_user_change(
            &mut devices,
            UserChange::Truncate {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                new_size: 0,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        )
        .expect("截到 0");
    image_of(&devices)
}

#[derive(Default)]
struct Summary {
    histories: usize,
    refused: BTreeMap<String, usize>,
    versions_seen: u64,
    record_zero_units_size_zero: u64,
    record_zero_units_size_positive: u64,
    unverifiable_versions: u64,
    shape_mismatches: Vec<String>,
    record_without_entry: Vec<String>,
    entry_without_record: Vec<String>,
    readers: BTreeMap<String, Vec<String>>,
    new_violations: BTreeMap<String, Vec<String>>,
    bytes_differ_from_the_old_rule: Vec<String>,
    histories_whose_bytes_differ: usize,
    old_rule: BTreeMap<String, Vec<String>>,
}

fn push_limited(list: &mut Vec<String>, item: String) {
    if list.len() < 6 {
        list.push(item);
    }
}

/// 扫：`R3_PROBE_STARTS`（逗号分开：normal、hand_made、hand_made_zero、generated_3000、generated_zero）、`R3_PROBE_LENGTH`（序列最长几步）、
/// `R3_PROBE_LIMIT`（只跑前几段，估时用）、`SINGLEFS_THREAD_CAP`（工作线程数）、`R3_PROBE_OUT`（汇总行追加到哪个文件）。
#[test]
fn r3_sweep() {
    let normal = normal_pool();
    let starts_wanted = std::env::var("R3_PROBE_STARTS").unwrap_or_else(|_| "normal".to_string());
    let mut starts = Vec::new();
    for name in starts_wanted.split(',') {
        let start = match name {
            "normal" => start_named("normal", normal.clone(), Some(file_content())),
            "hand_made" => start_named("hand_made", hand_made_tag_zero(&normal), Some(vec![0; 3000])),
            "hand_made_zero" => start_named("hand_made_zero", after_truncating_to_zero(&hand_made_tag_zero(&normal)), Some(Vec::new())),
            "generated_3000" => start_named("generated_3000", generated_tag_zero(&normal, 3000, false), Some(vec![0; 3000])),
            "generated_zero" => start_named("generated_zero", generated_tag_zero(&normal, 0, false), Some(Vec::new())),
            "scrubbed_3000" => start_named("scrubbed_3000", generated_tag_zero(&normal, 3000, true), Some(vec![0; 3000])),
            "scrubbed_zero" => start_named("scrubbed_zero", generated_tag_zero(&normal, 0, true), Some(Vec::new())),
            other => panic!("不认识的起点 {other}"),
        };
        starts.push(start);
    }
    let length = environment_number("R3_PROBE_LENGTH", 1);
    let limit = environment_number("R3_PROBE_LIMIT", usize::MAX);
    let threads = environment_number("SINGLEFS_THREAD_CAP", 1).max(1);
    let all_sequences: Vec<Vec<Step>> = sequences(length).into_iter().take(limit).collect();
    let mut out = String::new();
    for start in &starts {
        let readers_of_the_start = readers_line(&start.image, &start.content);
        let _ = writeln!(
            out,
            "name=start start={} baseline_violations={:?} readers: {}",
            start.name, start.baseline_violations, readers_of_the_start
        );
        let _ = writeln!(out, "name=compare start={} compared_mutant={}", start.name, environment_number("R3_PROBE_COMPARE_MUTANT", 1));
        for (name, detail) in violation_details(&start.image) {
            let _ = writeln!(out, "  baseline_violation {name}: {detail}");
        }
        let next = AtomicUsize::new(0);
        let summary = Mutex::new(Summary::default());
        let began = std::time::Instant::now();
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(steps) = all_sequences.get(index) else { break };
                    let name = format!("{steps:?}");
                    let fixed = run_history(start, steps, 0, true);
                    let judge_the_old = std::env::var("R3_PROBE_JUDGE_OLD").is_ok();
                    let compared_mutant = u8::try_from(environment_number("R3_PROBE_COMPARE_MUTANT", 1)).expect("变异号");
                    let old = run_history(start, steps, compared_mutant, judge_the_old);
                    let mut summary = summary.lock().expect("锁");
                    summary.histories += 1;
                    let observed = &fixed.observed;
                    if let Some(refusal) = &observed.refused {
                        *summary.refused.entry(refusal.clone()).or_default() += 1;
                    }
                    summary.versions_seen += observed.versions_seen;
                    summary.record_zero_units_size_zero += observed.record_zero_units_size_zero;
                    summary.record_zero_units_size_positive += observed.record_zero_units_size_positive;
                    summary.unverifiable_versions += observed.unverifiable_versions;
                    for item in &observed.shape_mismatches {
                        push_limited(&mut summary.shape_mismatches, format!("{name} {item}"));
                    }
                    for item in &observed.record_without_entry {
                        push_limited(&mut summary.record_without_entry, format!("{name} {item}"));
                    }
                    for item in &observed.entry_without_record {
                        push_limited(&mut summary.entry_without_record, format!("{name} {item}"));
                    }
                    let readers_list = summary.readers.entry(fixed.readers.clone()).or_default();
                    if readers_list.len() < 3 {
                        readers_list.push(name.clone());
                    }
                    if !fixed.new_violations.is_empty() {
                        let list = summary.new_violations.entry(format!("{:?}", fixed.new_violations)).or_default();
                        if list.len() < 3 {
                            list.push(name.clone());
                        }
                    }
                    if judge_the_old && fixed.final_image != old.final_image {
                        let key = format!(
                            "old_rule readers: {} new_violations={:?} record_without_entry={} entry_without_record={}",
                            old.readers,
                            old.new_violations,
                            old.observed.record_without_entry.len(),
                            old.observed.entry_without_record.len()
                        );
                        let list = summary.old_rule.entry(key).or_default();
                        if list.len() < 3 {
                            list.push(name.clone());
                        }
                    }
                    if fixed.final_image != old.final_image {
                        summary.histories_whose_bytes_differ += 1;
                        push_limited(&mut summary.bytes_differ_from_the_old_rule, name.clone());
                    }
                });
            }
        });
        let summary = summary.into_inner().expect("锁");
        let _ = writeln!(
            out,
            "name=summary start={} length={length} histories={} seconds={:.1} threads={threads} versions_seen={} record_zero_units_size_zero={} record_zero_units_size_positive={} unverifiable_versions={} shape_mismatches={} record_without_entry={} entry_without_record={} histories_whose_bytes_differ_from_the_old_rule={}",
            start.name,
            summary.histories,
            began.elapsed().as_secs_f64(),
            summary.versions_seen,
            summary.record_zero_units_size_zero,
            summary.record_zero_units_size_positive,
            summary.unverifiable_versions,
            summary.shape_mismatches.len(),
            summary.record_without_entry.len(),
            summary.entry_without_record.len(),
            summary.histories_whose_bytes_differ,
        );
        for (key, count) in &summary.refused {
            let _ = writeln!(out, "  refused count={count} why={key}");
        }
        for (line, examples) in &summary.readers {
            let _ = writeln!(out, "  readers {line} examples={examples:?}");
        }
        for (set, examples) in &summary.new_violations {
            let _ = writeln!(out, "  new_violations {set} examples={examples:?}");
        }
        for (line, examples) in &summary.old_rule {
            let _ = writeln!(out, "  {line} examples={examples:?}");
        }
        for (label, list) in [
            ("shape_mismatch", &summary.shape_mismatches),
            ("record_without_entry", &summary.record_without_entry),
            ("entry_without_record", &summary.entry_without_record),
            ("bytes_differ", &summary.bytes_differ_from_the_old_rule),
        ] {
            for item in list {
                let _ = writeln!(out, "  {label} {item}");
            }
        }
    }
    print!("{out}");
    if let Ok(path) = std::env::var("R3_PROBE_OUT") {
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path).expect("开输出");
        file.write_all(out.as_bytes()).expect("写输出");
    }
}
