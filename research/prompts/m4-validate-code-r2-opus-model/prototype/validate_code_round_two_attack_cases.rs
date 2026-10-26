//! m4-validate-code-r2 攻方腿在冻结副本拷贝上造的用例（只在副本里，不进仓）。
//! 盘一律是内存稀疏盘（`SparseBlockDevice`）：mkfs、取号、暖机、新池新建文件只跑一次，之后每段历史从内存里那一份拷。
//! 每条测试只打 `ATTACK-` 行，结局收成文字（panic 也收），打中与否在报告里按条款判；断言只用在造镜像的前提上。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

use common::{parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_deletion_in_progress::change_the_unit_and_propagate_its_checksum;
use common_pointer_head_corruption::{
    allocation_record_tree_nodes_as_far_as_readable, first_upper_leaf_entry, newest_root,
    node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{
    AllocationRecord, AllocationRecordQuarantineMark, DeviceFreeMap, Placement, PoolAllocator,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::bytes::ByteReader;
use singlefs_core::extent_tree::{ExtentUpperLeafEntry, ExtentUpperLeafTarget};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{parse_mapping_entry, TREE_KIND_EXTENT};
use singlefs_core::recovery::{allocation_records_under_root, recover, walk_to_file, JournalPolicy};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
const WRITE_TIME_SECONDS: u64 = 1_788_000_300;
const EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET: usize = 8;
const EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET: usize = 24;
const INDEX_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_RANGE_OFFSET: usize = 52;
/// 分配记录树的种类码（D8 已定项 9 登记表）。
const TREE_KIND_ALLOCATION_RECORDS: u16 = 3;
/// 分配记录 20 字节里跨度段的偏移（设备 4 + 槽号 6）。
const ALLOCATION_RECORD_SPAN_OFFSET: usize = 10;

fn payload() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
        .unwrap_or_else(|| "（panic 载荷不是字符串）".to_string())
}

fn short(text: impl std::fmt::Debug) -> String {
    format!("{text:?}").chars().take(220).collect()
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

fn devices_of(image: &MemoryPool) -> Devices {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

/// 起点：两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 新池新建文件（3000 字节，实例 1，txg 3），进程里接着用的分配器与这一版。
#[derive(Clone)]
struct StartingPoint {
    image: MemoryPool,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

fn starting_point() -> &'static StartingPoint {
    static STARTING_POINT: OnceLock<StartingPoint> = OnceLock::new();
    STARTING_POINT.get_or_init(|| {
        let parameters = parameters();
        let mut devices: Devices = (0..2u32)
            .map(|number| (DeviceIdentity(number), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
            Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
        let content = common::file_content();
        let output = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut pool,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("新池新建文件")
        };
        StartingPoint { image: image_of(&devices), allocator, output }
    })
}

// ── 标签 0 的镜像：办法同 tests/whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs，起点换成内存盘 ──

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn rewrite_entries(image: &mut MemoryPool, slot: SlotNumber, key_range_follows: bool, change: impl FnOnce(&mut Vec<Vec<u8>>)) {
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
        node[count_offset..count_offset + 2].copy_from_slice(&u16::try_from(entries.len()).expect("条目数").to_le_bytes());
        node[INDEX_NODE_DECLARED_LENGTH_OFFSET..INDEX_NODE_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(&u16::try_from(entries.len() * entry_width).expect("声明长度").to_le_bytes());
        if let (true, Some(first), Some(last)) = (key_range_follows, entries.first(), entries.last()) {
            let smallest = INDEX_NODE_KEY_RANGE_OFFSET;
            let largest = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
            node[smallest..smallest + key_width].copy_from_slice(&first[..key_width]);
            node[largest..largest + key_width].copy_from_slice(&last[..key_width]);
        }
    });
}

fn tag_zero_image() -> &'static MemoryPool {
    static IMAGE: OnceLock<MemoryPool> = OnceLock::new();
    IMAGE.get_or_init(|| {
        let mut image = starting_point().image.clone();
        let ExtentUpperLeafTarget::InlineDataUnit(data_unit) = first_upper_leaf_entry(&image).target else {
            panic!("前提：新池新建文件那一版第一个文件是标签 2")
        };
        let mapping_root_slot = the_slot_of(&newest_root(&image).mapping_root);
        assert_eq!(unit_bytes_at(&image, mapping_root_slot, node_bytes())[INDEX_NODE_LEVEL_OFFSET], 0);
        rewrite_entries(&mut image, mapping_root_slot, true, |entries| {
            entries.retain(|entry| {
                let (_key, locations) = parse_mapping_entry(entry).expect("映射条目");
                locations[0].slot != data_unit.locations[0].slot
            });
        });
        let extent_root_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root);
        rewrite_entries(&mut image, extent_root_slot, false, |entries| {
            let first = entries
                .iter_mut()
                .find(|entry| read_u64_at(entry, EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET) == FIRST_INODE_NUMBER)
                .expect("第一个文件那一条");
            first[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET] = 0;
            first[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET + 1..].fill(0);
        });
        assert_eq!(first_upper_leaf_entry(&image).target, ExtentUpperLeafTarget::NoDataUnit);
        image
    })
}

/// 最新那一版 extent 树上段里第一个文件那一条（上段可能长高过一层）；读不出也收成文字。
fn first_file_upper_entry(image: &MemoryPool) -> String {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
        let root_bytes = unit_bytes_at(image, the_slot_of(&extent_root), node_bytes());
        let root = parse_index_node(&root_bytes).expect("上段根");
        let leaves: Vec<Vec<Vec<u8>>> = if root_bytes[INDEX_NODE_LEVEL_OFFSET] == 0 {
            vec![root.entries]
        } else {
            root.entries
                .iter()
                .map(|internal| {
                    let child = NodePointer::read_from(&mut ByteReader::at(internal, root.key_width));
                    parse_index_node(&unit_bytes_at(image, the_slot_of(&child), node_bytes())).expect("上段叶").entries
                })
                .collect()
        };
        leaves
            .iter()
            .flatten()
            .map(|entry| ExtentUpperLeafEntry::parse(entry).expect("上段叶条目"))
            .find(|entry| entry.inode == FIRST_INODE_NUMBER)
            .map(|entry| entry.target)
    }));
    match outcome {
        Ok(Some(ExtentUpperLeafTarget::NoDataUnit)) => "标签0".into(),
        Ok(Some(ExtentUpperLeafTarget::InlineDataUnit(_))) => "标签2".into(),
        Ok(Some(ExtentUpperLeafTarget::LowerSegmentRoot(_))) => "标签1".into(),
        Ok(Some(ExtentUpperLeafTarget::LowerSegmentRootFollowedByAHole(_))) => "标签3".into(),
        Ok(Some(ExtentUpperLeafTarget::InlineDataUnitFollowedByAHole(_))) => "标签4".into(),
        Ok(None) => "没有条目".into(),
        Err(payload) => format!("读不出({})", panic_text(payload.as_ref())),
    }
}

fn content_summary(bytes: &[u8], expected: &[u8]) -> String {
    format!("len={} same_as_expected={}", bytes.len(), bytes == expected)
}

/// 三处读者（冷走读、可写重挂、挂载态读者）与冷恢复各读一遍第一个文件。`expected` 是该读出的内容。
fn readers(image: &MemoryPool, expected: &[u8]) -> String {
    let cold = catch_unwind(AssertUnwindSafe(|| {
        let root = newest_root(image);
        let mut fallbacks = 0;
        match walk_to_file(image, &root, &mut fallbacks) {
            Ok(Some(content)) => format!("Ok({})", content_summary(&content, expected)),
            Ok(None) => "Ok(None)".into(),
            Err(failure) => format!("Err({})", short(failure)),
        }
    }))
    .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    let remount = catch_unwind(AssertUnwindSafe(|| {
        let mut devices = devices_of(image);
        match mount_writable(&parameters(), &mut devices) {
            Ok(mounted) => match mounted.current {
                PoolVersion::WithFile(version) => format!(
                    "Ok(size={:?} units={})",
                    version.inode_record.map(|record| record.size),
                    version.data_pointers.len()
                ),
                PoolVersion::WithoutFile(_) => "Ok(没有文件)".into(),
                PoolVersion::WithTreesAndNoFile(_) => "Ok(有树没有文件)".into(),
            },
            Err(refusal) => format!("Err({})", short(refusal)),
        }
    }))
    .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    let mounted_reader = catch_unwind(AssertUnwindSafe(|| match mount_read_only(image) {
        Err(refusal) => format!("挂载Err({})", short(refusal)),
        Ok(read_only) => match read_only.mounted.open_file(image, FIRST_FILE) {
            Err(failure) => format!("打开Err({})", short(failure)),
            Ok(file) => {
                let size = file.inode_record().size;
                let units = file.data_unit_count();
                match file.read_at(image, FileOffsetInBytes(0), size) {
                    Ok(read) => format!("Ok(size={size} units={units} {})", content_summary(&read.bytes, expected)),
                    Err(failure) => format!("读Err(size={size} units={units} {})", short(failure)),
                }
            }
        },
    }))
    .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    let recovered = catch_unwind(AssertUnwindSafe(|| short(recover(image, JournalPolicy::Consult).outcome).chars().take(90).collect::<String>()))
        .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    format!("上段条目={} 冷走读={cold} 可写重挂={remount} 挂载态读者={mounted_reader} 冷恢复={recovered}", first_file_upper_entry(image))
}

// ── 会话里逐步发，每一步之后三处读者各读一遍 ──

#[derive(Clone, Debug)]
enum Step {
    Truncate(u64),
    NewInodes(u64),
    OverwriteOfAnInode(u64, Vec<u8>),
    SequentialWriteOfTheFirstFile(Vec<u8>),
}

fn pattern(length: usize, seed: u8) -> Vec<u8> {
    (0..length).map(|index| u8::try_from((index * 7 + usize::from(seed)) % 251 + 1).expect("小于 256")).collect()
}

/// `image` 可写挂载、会话里按次序发 `steps`；每一步打一行结局与读者；panic 或拒就停。交回最后的镜像与会话。
fn run_steps(what: &str, image: &MemoryPool, first_file_before: &[u8], steps: &[Step]) -> (MemoryPool, Option<MountedSession>, Devices) {
    let mut devices = devices_of(image);
    let mounted = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => {
            eprintln!("ATTACK-T1 {what} 起点可写挂载拒：{}", short(refusal));
            return (image.clone(), None, devices);
        }
    };
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let mut expected = first_file_before.to_vec();
    eprintln!("ATTACK-T1 {what} 第0步（起点） {}", readers(image, &expected));
    for (index, step) in steps.iter().enumerate() {
        let change = match step {
            Step::Truncate(size) => UserChange::Truncate { inode: FIRST_FILE, new_size: *size, write_time_seconds: WRITE_TIME_SECONDS },
            Step::NewInodes(count) => UserChange::NewInodes { count: *count, write_time_seconds: WRITE_TIME_SECONDS },
            Step::OverwriteOfAnInode(inode, content) => UserChange::OverwriteOfAnInode {
                inode: InodeNumber(*inode),
                content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            Step::SequentialWriteOfTheFirstFile(content) => UserChange::SequentialWriteOfAnInode {
                inode: FIRST_FILE,
                content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        };
        let outcome = catch_unwind(AssertUnwindSafe(|| session.publish_user_change(&mut devices, change)));
        let label = match step {
            Step::Truncate(size) => format!("改size到{size}"),
            Step::NewInodes(count) => format!("建{count}个inode"),
            Step::OverwriteOfAnInode(inode, content) => format!("覆盖写inode{inode}共{}字节", content.len()),
            Step::SequentialWriteOfTheFirstFile(content) => format!("顺序写第一个文件{}字节", content.len()),
        };
        match outcome {
            Err(payload) => {
                eprintln!("ATTACK-T1 {what} 第{}步 {label}：panic：{}", index + 1, panic_text(payload.as_ref()));
                return (image_of(&devices), None, devices);
            }
            Ok(Err(refused)) => {
                eprintln!("ATTACK-T1 {what} 第{}步 {label}：拒：{}", index + 1, short(refused));
                return (image_of(&devices), Some(session), devices);
            }
            Ok(Ok(published)) => {
                match step {
                    Step::Truncate(size) => expected.resize(usize::try_from(*size).expect("小"), 0),
                    Step::OverwriteOfAnInode(inode, content) if *inode == FIRST_INODE_NUMBER => expected = content.clone(),
                    Step::SequentialWriteOfTheFirstFile(content) => expected = content.clone(),
                    Step::OverwriteOfAnInode(..) | Step::NewInodes(_) => {}
                }
                let after = image_of(&devices);
                let record_size = match &session.current {
                    PoolVersion::WithFile(version) => format!("{:?}/{}", version.inode_record.map(|record| record.size), version.data_pointers.len()),
                    PoolVersion::WithoutFile(_) | PoolVersion::WithTreesAndNoFile(_) => "没有文件".into(),
                };
                eprintln!(
                    "ATTACK-T1 {what} 第{}步 {label}：做成{}版 内存里size/单元数={record_size} {}",
                    index + 1,
                    published.versions_published.len(),
                    readers(&after, &expected)
                );
            }
        }
    }
    (image_of(&devices), Some(session), devices)
}

const TAG_ZERO_SIZE: usize = 3000;

#[test]
fn t1_tag_zero_truncated_to_zero_then_its_upper_leaf_rewritten_then_extended() {
    run_steps(
        "截到0再重写上段叶再变长",
        tag_zero_image(),
        &vec![0u8; TAG_ZERO_SIZE],
        &[
            Step::Truncate(0),
            Step::NewInodes(1),
            Step::OverwriteOfAnInode(2, pattern(100, 9)),
            Step::Truncate(5000),
            Step::OverwriteOfAnInode(1, pattern(200, 3)),
        ],
    );
}

#[test]
fn t1_tag_zero_truncated_to_zero_then_extended() {
    run_steps("截到0再变长", tag_zero_image(), &vec![0u8; TAG_ZERO_SIZE], &[Step::Truncate(0), Step::Truncate(5000), Step::Truncate(0)]);
}

#[test]
fn t1_tag_zero_extended_past_several_payloads_then_written() {
    let extended = 3 * payload() + 17;
    run_steps(
        "变长到三个多净荷再写",
        tag_zero_image(),
        &vec![0u8; TAG_ZERO_SIZE],
        &[Step::Truncate(extended), Step::OverwriteOfAnInode(1, pattern(100, 1)), Step::Truncate(extended)],
    );
}

#[test]
fn t1_tag_zero_written_sequentially_over_two_units_then_truncated() {
    let two_units = usize::try_from(payload()).expect("小") + 500;
    run_steps(
        "顺序写两单元再截",
        tag_zero_image(),
        &vec![0u8; TAG_ZERO_SIZE],
        &[Step::SequentialWriteOfTheFirstFile(pattern(two_units, 5)), Step::Truncate(10), Step::Truncate(0)],
    );
}

#[test]
fn t1_tag_zero_upper_leaf_rewritten_after_truncating_to_a_positive_size() {
    run_steps(
        "截到100再重写上段叶",
        tag_zero_image(),
        &vec![0u8; TAG_ZERO_SIZE],
        &[Step::Truncate(100), Step::NewInodes(1), Step::OverwriteOfAnInode(2, pattern(50, 4)), Step::Truncate(0), Step::OverwriteOfAnInode(2, pattern(60, 6))],
    );
}

/// 标签 0 镜像可写挂载、截到 100，再回退到标签 0 那一版（实例 1、txg 3）：回退目标一个数据单元都没有。
#[test]
fn t1_rolling_back_to_the_tag_zero_version() {
    let (image, session, mut devices) =
        run_steps("截到100再回退到标签0那一版", tag_zero_image(), &vec![0u8; TAG_ZERO_SIZE], &[Step::Truncate(100)]);
    let Some(mut session) = session else { return };
    let PoolVersion::WithFile(mut current) = session.current.clone() else { panic!("带文件") };
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        roll_back_by_a_forward_publish(
            &parameters(),
            &mut devices,
            &mut session.allocator,
            &mut current,
            RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) },
        )
    }));
    let text = match outcome {
        Err(payload) => format!("panic：{}", panic_text(payload.as_ref())),
        Ok(Err(refused)) => format!("拒：{}", short(refused)),
        Ok(Ok(_)) => format!("做成 {}", readers(&image_of(&devices), &vec![0u8; TAG_ZERO_SIZE])),
    };
    drop(image);
    eprintln!("ATTACK-T1 回退到标签0那一版：{text}");
}

// ── 隔离记号（T2、T3、T4） ──

fn read_copy(image: &MemoryPool, device: DeviceIdentity, slot: SlotNumber, length: usize) -> Vec<u8> {
    image.devices[&device].read(slot.to_device_offset(), length)
}

fn write_copy(devices: &mut Devices, device: DeviceIdentity, slot: SlotNumber, bytes: &[u8]) {
    let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("池里有这块盘");
    block_device.write_at(slot.to_device_offset(), bytes, WriteDurability::Plain).expect("写一份");
}

fn corrupt_copy(devices: &mut Devices, device: DeviceIdentity, slot: SlotNumber, length: usize) -> Vec<u8> {
    let intact = read_copy(&image_of(devices), device, slot, length);
    let mut corrupted = intact.clone();
    corrupted[4096] ^= 0xff;
    write_copy(devices, device, slot, &corrupted);
    intact
}

fn mark_text(record: &AllocationRecord) -> String {
    let span_field = u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]);
    format!(
        "盘{}槽{}:{}代{}跨度段{:#06x}",
        record.device.0,
        record.slot.0,
        if record.is_released { "已释放" } else { "仍分配" },
        record.generation.0,
        span_field
    )
}

/// 最新根那棵分配记录树里起点是 `slots` 之一的记录（读不回来也收成文字）。
fn records_on_disk(image: &MemoryPool, slots: &[SlotNumber]) -> String {
    match allocation_records_under_root(image, &newest_root(image)) {
        Err(failure) => format!("读不回({})", short(failure)),
        Ok(records) => records
            .iter()
            .filter(|record| slots.contains(&record.slot))
            .map(mark_text)
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn marked_records_on_disk(image: &MemoryPool) -> String {
    match allocation_records_under_root(image, &newest_root(image)) {
        Err(failure) => format!("读不回({})", short(failure)),
        Ok(records) => {
            let marked: Vec<String> = records
                .iter()
                .filter(|record| record.quarantine_mark == AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck)
                .map(mark_text)
                .collect();
            format!("带记号{}条[{}]", marked.len(), marked.join(" "))
        }
    }
}

/// 池级 checker 判这张镜像：判红的那几条（名字与说明的开头）。
fn checker(image: &MemoryPool) -> String {
    let verdicts = catch_unwind(AssertUnwindSafe(|| check_pool_image(image)));
    match verdicts {
        Err(payload) => format!("checker panic({})", panic_text(payload.as_ref())),
        Ok(verdicts) => {
            let red: Vec<String> = verdicts
                .iter()
                .filter_map(|(name, verdict)| match verdict {
                    InvariantVerdict::Violated(why) => Some(format!("{name}:{}", why.chars().take(140).collect::<String>())),
                    _ => None,
                })
                .collect();
            if red.is_empty() { format!("checker 全不红（{} 条）", verdicts.len()) } else { format!("checker 判红 {} 条 [{}]", red.len(), red.join(" | ")) }
        }
    }
}

/// 进程里接着起点覆盖写一次（实例 1）：交回这一版。
fn overwrite_in_process(devices: &mut Devices, allocator: &mut PoolAllocator, previous: &TransactionOutput, content: &[u8], seconds: u64) -> TransactionOutput {
    let parameters = parameters();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(&mut writer, allocator, previous, FirstFile { content, write_time_seconds: seconds }, InstanceGeneration(1)).expect("覆盖写做成")
}

const SLOT_BYTES_USIZE: usize = 16384;

/// 起点 A（txg 3）进程里接着：`corrupt` 那几份（盘、槽、字节数）改坏之后覆盖写 B（txg 4）。交回 B 之后的盘、分配器、B、改坏之前的原样。
fn quarantined_after_b(
    corrupt: &dyn Fn(&TransactionOutput) -> Vec<(DeviceIdentity, SlotNumber, usize)>,
) -> (Devices, PoolAllocator, TransactionOutput, Vec<(DeviceIdentity, SlotNumber, Vec<u8>)>) {
    let start = starting_point();
    let mut devices = devices_of(&start.image);
    let mut allocator = start.allocator.clone();
    let mut intact = Vec::new();
    for (device, slot, length) in corrupt(&start.output) {
        intact.push((device, slot, corrupt_copy(&mut devices, device, slot, length)));
    }
    let b = overwrite_in_process(&mut devices, &mut allocator, &start.output, &pattern(2700, 2), FIXED_WRITE_TIME_SECONDS + 60);
    (devices, allocator, b, intact)
}

fn restore(devices: &mut Devices, intact: &[(DeviceIdentity, SlotNumber, Vec<u8>)]) {
    for (device, slot, bytes) in intact {
        write_copy(devices, *device, *slot, bytes);
    }
}

fn rollback(devices: &mut Devices, allocator: &mut PoolAllocator, current: &mut TransactionOutput, instance: u32, txg: u64) -> String {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        roll_back_by_a_forward_publish(
            &parameters(),
            devices,
            allocator,
            current,
            RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) },
        )
    }));
    match outcome {
        Err(payload) => format!("panic：{}", panic_text(payload.as_ref())),
        Ok(Err(refused)) => format!("拒：{}", short(refused)),
        Ok(Ok(rolled_back)) => format!(
            "做成（新版 txg {}，复活 {} 份、释放 {} 个、隔离 {} 份）",
            current.root.checkpoint_txg.0,
            rolled_back.resurrected_user_visible_copies.len(),
            rolled_back.released_user_visible_units.len(),
            rolled_back.quarantined_user_visible_copies.len()
        ),
    }
}

fn data_unit_of_a(output: &TransactionOutput) -> Vec<(DeviceIdentity, SlotNumber, usize)> {
    vec![(DeviceIdentity(1), output.data_pointers[0].locations[0].slot, 2 * SLOT_BYTES_USIZE)]
}

#[test]
fn q_baseline_images_judged_by_the_pool_checker() {
    eprintln!("ATTACK-Q 起点 A：{}", checker(&starting_point().image));
    let (devices, _, b, _) = quarantined_after_b(&data_unit_of_a);
    let image = image_of(&devices);
    eprintln!(
        "ATTACK-Q 隔离 A 的数据单元之后 B：隔离 {} 份 {} {}",
        b.quarantined_after_release_checksum_mismatch.len(),
        marked_records_on_disk(&image),
        checker(&image)
    );
}

/// T3：隔离之后进程重开可写挂载（分配器从盘上重建），盘 1 那一份写回原样，再回退到 A：记号清没清、checker、读者。
#[test]
fn q_rollback_after_a_remount_clears_the_mark_read_from_disk() {
    let (mut devices, _, _, intact) = quarantined_after_b(&data_unit_of_a);
    let slot = intact[0].1;
    restore(&mut devices, &intact);
    let image = image_of(&devices);
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    let mut allocator = mounted.allocator;
    let PoolVersion::WithFile(mut current) = mounted.current else { panic!("带文件") };
    let marks_in_memory: Vec<String> = [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .filter_map(|device| allocator.record_for(*device, slot).map(mark_text))
        .collect();
    eprintln!("ATTACK-Q 重挂之后分配器里：{}", marks_in_memory.join(" "));
    let text = rollback(&mut devices, &mut allocator, &mut current, 1, 3);
    let after = image_of(&devices);
    eprintln!("ATTACK-Q 重挂之后回退到 A：{text} 盘上 {} {} 读者 {}", records_on_disk(&after, &[slot]), checker(&after), readers(&after, &common::file_content()));
}

/// T2：两单元的文件 A2（顺序写，txg 4），单元 0 在盘 0 上改坏、单元 1 在盘 1 上改坏，覆盖写 B（txg 5）：同一次发布隔离两个单元。
#[test]
fn q_two_units_quarantined_in_one_publish_each_bad_on_a_different_device() {
    let start = starting_point();
    let mut devices = devices_of(&start.image);
    let mut allocator = start.allocator.clone();
    let parameters = parameters();
    let content_a2 = pattern(usize::try_from(payload()).expect("小") + 900, 8);
    let a2 = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        singlefs_core::transaction::publish_sequential_write(
            &mut writer,
            &mut allocator,
            &start.output,
            FirstFile { content: &content_a2, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 30 },
            InstanceGeneration(1),
        )
        .expect("顺序写 A2")
    };
    let slots: Vec<SlotNumber> = a2.data_pointers.iter().map(|pointer| pointer.locations[0].slot).collect();
    eprintln!("ATTACK-Q A2：txg {} 单元 {}", a2.root.checkpoint_txg.0, slots.len());
    let intact = vec![
        (DeviceIdentity(0), slots[0], corrupt_copy(&mut devices, DeviceIdentity(0), slots[0], 2 * SLOT_BYTES_USIZE)),
        (DeviceIdentity(1), slots[1], corrupt_copy(&mut devices, DeviceIdentity(1), slots[1], 2 * SLOT_BYTES_USIZE)),
    ];
    let b = overwrite_in_process(&mut devices, &mut allocator, &a2, &pattern(2700, 2), FIXED_WRITE_TIME_SECONDS + 90);
    let image = image_of(&devices);
    eprintln!(
        "ATTACK-Q 两单元各坏一盘之后 B：隔离 {} 份 盘上 {} {}",
        b.quarantined_after_release_checksum_mismatch.len(),
        records_on_disk(&image, &slots),
        checker(&image)
    );
    restore(&mut devices, &intact);
    let image = image_of(&devices);
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters, &mut devices).expect("重开可写挂载");
    let mut allocator = mounted.allocator;
    let PoolVersion::WithFile(mut current) = mounted.current else { panic!("带文件") };
    let text = rollback(&mut devices, &mut allocator, &mut current, 1, a2.root.checkpoint_txg.0);
    let after = image_of(&devices);
    eprintln!("ATTACK-Q 回退到 A2：{text} 盘上 {} {} 读者 {}", records_on_disk(&after, &slots), checker(&after), readers(&after, &content_a2));
}

/// T2、T3：隔离之后重挂，会话里接着覆盖写 `rounds` 次（转过根环、抬 F、回收、整理都可能跑），记号留不留、checker、再回退到 A。
#[test]
fn q_marks_survive_many_publishes_after_a_remount() {
    let (mut devices, _, _, intact) = quarantined_after_b(&data_unit_of_a);
    let slot = intact[0].1;
    let image = image_of(&devices);
    let mut devices_after = devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices_after).expect("重开可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    for round in 0..40u8 {
        let content = pattern(1000 + usize::from(round) * 37, round);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            session.publish_user_change(
                &mut devices_after,
                UserChange::OverwriteOfAnInode { inode: FIRST_FILE, content: &content, write_time_seconds: WRITE_TIME_SECONDS + u64::from(round) },
            )
        }));
        match outcome {
            Err(payload) => {
                eprintln!("ATTACK-Q 重挂后第{round}次覆盖写 panic：{}", panic_text(payload.as_ref()));
                return;
            }
            Ok(Err(refused)) => {
                eprintln!("ATTACK-Q 重挂后第{round}次覆盖写 拒：{}", short(refused));
                return;
            }
            Ok(Ok(published)) => {
                if round % 10 == 9 || published.compaction_after_the_change.is_some() || !published.floor_raises.is_empty() {
                    let now = image_of(&devices_after);
                    eprintln!(
                        "ATTACK-Q 重挂后第{round}次覆盖写：抬F {} 次 整理 {} 盘上 {} {}",
                        published.floor_raises.len(),
                        published.compaction_after_the_change.as_ref().map_or("没起".to_string(), |run| format!("发了{}版", run.versions_published.len())),
                        records_on_disk(&now, &[slot]),
                        checker(&now)
                    );
                }
            }
        }
    }
    restore(&mut devices_after, &intact);
    let PoolVersion::WithFile(mut current) = session.current.clone() else { panic!("带文件") };
    let text = rollback(&mut devices_after, &mut session.allocator, &mut current, 1, 3);
    let after = image_of(&devices_after);
    eprintln!("ATTACK-Q 四十次之后回退到 A：{text} 盘上 {} {}", records_on_disk(&after, &[slot]), checker(&after));
    drop(devices);
}

/// T3：A →（隔离）B → C → 回退到 B（不引用隔离单元，记号该留）→ 回退到 A（引用它，记号该清）。
#[test]
fn q_rollback_to_a_version_not_referencing_the_quarantined_unit_then_to_one_that_does() {
    let (mut devices, mut allocator, b, intact) = quarantined_after_b(&data_unit_of_a);
    let slot = intact[0].1;
    let mut current = overwrite_in_process(&mut devices, &mut allocator, &b, &pattern(1500, 7), FIXED_WRITE_TIME_SECONDS + 120);
    let to_b = rollback(&mut devices, &mut allocator, &mut current, 1, b.root.checkpoint_txg.0);
    let image = image_of(&devices);
    eprintln!("ATTACK-Q C 之后回退到 B：{to_b} 盘上 {} {}", records_on_disk(&image, &[slot]), checker(&image));
    restore(&mut devices, &intact);
    let to_a = rollback(&mut devices, &mut allocator, &mut current, 1, 3);
    let image = image_of(&devices);
    eprintln!("ATTACK-Q 再回退到 A：{to_a} 盘上 {} {} 读者 {}", records_on_disk(&image, &[slot]), checker(&image), readers(&image, &common::file_content()));
    let mut remount_devices = devices_of(&image);
    let remount = mount_writable(&parameters(), &mut remount_devices).map(|_| ()).map_err(short);
    eprintln!("ATTACK-Q 再回退到 A 之后重挂：{remount:?}");
}

/// T2、T3：A 引用、B 换下的每个单元在盘 1 上那一份都改坏（数据单元与各树节点），B 隔离它们；重挂；写回原样回退到 A。
#[test]
fn q_every_unit_b_replaces_quarantined_then_rolled_back() {
    let start = starting_point();
    let replaced: Vec<(SlotNumber, usize)> = {
        let mut devices = devices_of(&start.image);
        let mut allocator = start.allocator.clone();
        let b = overwrite_in_process(&mut devices, &mut allocator, &start.output, &pattern(2700, 2), FIXED_WRITE_TIME_SECONDS + 60);
        let slots_of_b: Vec<SlotNumber> = b.units.iter().map(|unit| unit.slot).collect();
        start
            .output
            .units
            .iter()
            .filter(|unit| !slots_of_b.contains(&unit.slot))
            .map(|unit| (unit.slot, usize::try_from(unit.identity.span_slots()).expect("小") * SLOT_BYTES_USIZE))
            .collect()
    };
    let replaced_for_closure = replaced.clone();
    let (mut devices, _, b, intact) = quarantined_after_b(&move |_| {
        replaced_for_closure.iter().map(|(slot, length)| (DeviceIdentity(1), *slot, *length)).collect()
    });
    let image = image_of(&devices);
    let slots: Vec<SlotNumber> = replaced.iter().map(|(slot, _)| *slot).collect();
    eprintln!(
        "ATTACK-Q B 换下 {} 个单元全在盘 1 改坏：隔离 {} 份 {} {}",
        replaced.len(),
        b.quarantined_after_release_checksum_mismatch.len(),
        marked_records_on_disk(&image),
        checker(&image)
    );
    let mut remount_devices = devices_of(&image);
    let remount = mount_writable(&parameters(), &mut remount_devices).map(|_| ()).map_err(short);
    eprintln!("ATTACK-Q 那张镜像重挂：{remount:?} 读者 {}", readers(&image, &pattern(2700, 2)));
    restore(&mut devices, &intact);
    let image = image_of(&devices);
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices).expect("写回原样之后重挂");
    let mut allocator = mounted.allocator;
    let PoolVersion::WithFile(mut current) = mounted.current else { panic!("带文件") };
    let text = rollback(&mut devices, &mut allocator, &mut current, 1, 3);
    let after = image_of(&devices);
    eprintln!("ATTACK-Q 写回原样回退到 A：{text} 盘上 {} {} 读者 {}", records_on_disk(&after, &slots), checker(&after), readers(&after, &common::file_content()));
}

// ── T4：手改跨度段的坏镜像与合法镜像，核心层各处读者与池级 checker 怎么判 ──

/// 最新根那棵分配记录树里 `device` 上起点是 `slot` 的那条记录的跨度段改成 `span_field`，重封、校验和沿引用链补到根槽。
fn set_span_field(image: &mut MemoryPool, device: DeviceIdentity, slot: SlotNumber, span_field: u16) {
    let root = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION_RECORDS).root;
    let mut target = None;
    for pointer in allocation_record_tree_nodes_as_far_as_readable(image, &root) {
        let node_slot = pointer.locations[0].slot;
        let bytes = unit_bytes_at(image, node_slot, node_bytes());
        if bytes[INDEX_NODE_LEVEL_OFFSET] != 0 {
            continue;
        }
        let parsed = parse_index_node(&bytes).expect("叶解得开");
        for (index, entry) in parsed.entries.iter().enumerate() {
            let entry_device = u32::from_le_bytes(entry[0..4].try_into().expect("4"));
            let mut widened = [0u8; 8];
            widened[..6].copy_from_slice(&entry[4..10]);
            if entry_device == device.0 && u64::from_le_bytes(widened) == slot.0 {
                target = Some((node_slot, 115 + 2 * parsed.key_width + index * parsed.entry_width + ALLOCATION_RECORD_SPAN_OFFSET));
            }
        }
    }
    let (node_slot, offset) = target.expect("找得到那条记录");
    change_the_unit_and_propagate_its_checksum(image, node_slot, node_bytes(), |node| {
        node[offset..offset + 2].copy_from_slice(&span_field.to_le_bytes());
    });
}

fn core_readers_of_the_pool(image: &MemoryPool) -> String {
    let writable = catch_unwind(AssertUnwindSafe(|| {
        let mut devices = devices_of(image);
        mount_writable(&parameters(), &mut devices).map(|_| "Ok".to_string()).unwrap_or_else(|refusal| format!("Err({})", short(refusal)))
    }))
    .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    let read_only = catch_unwind(AssertUnwindSafe(|| mount_read_only(image).map(|_| "Ok".to_string()).unwrap_or_else(|refusal| format!("Err({})", short(refusal)))))
        .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    let tree = catch_unwind(AssertUnwindSafe(|| {
        allocation_records_under_root(image, &newest_root(image)).map(|records| format!("Ok({} 条)", records.len())).unwrap_or_else(|failure| format!("Err({})", short(failure)))
    }))
    .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    let recovered = catch_unwind(AssertUnwindSafe(|| short(recover(image, JournalPolicy::Consult).outcome).chars().take(120).collect::<String>()))
        .unwrap_or_else(|payload| format!("panic({})", panic_text(payload.as_ref())));
    format!("可写挂载={writable} 只读挂载={read_only} 读分配记录树={tree} 冷恢复={recovered}")
}

#[test]
fn t4_span_fields_set_by_hand_on_the_quarantined_and_on_a_live_record() {
    let (devices, _, b, intact) = quarantined_after_b(&data_unit_of_a);
    let quarantined = intact[0].1;
    let live = b.data_pointers[0].locations[0].slot;
    let base = image_of(&devices);
    let cases: Vec<(&str, Vec<(DeviceIdentity, SlotNumber, u16)>)> = vec![
        ("隔离记录两盘 0xC002（记号加已释放）", vec![(DeviceIdentity(0), quarantined, 0xC002), (DeviceIdentity(1), quarantined, 0xC002)]),
        ("隔离记录盘 0 是 0xC002", vec![(DeviceIdentity(0), quarantined, 0xC002)]),
        ("隔离记录两盘 0xC000", vec![(DeviceIdentity(0), quarantined, 0xC000), (DeviceIdentity(1), quarantined, 0xC000)]),
        ("隔离记录两盘 0x4000（跨度 0）", vec![(DeviceIdentity(0), quarantined, 0x4000), (DeviceIdentity(1), quarantined, 0x4000)]),
        ("隔离记录两盘 0x4003（跨度 3）", vec![(DeviceIdentity(0), quarantined, 0x4003), (DeviceIdentity(1), quarantined, 0x4003)]),
        ("隔离记录只剩盘 1 带记号（盘 0 清成 0x0002）", vec![(DeviceIdentity(0), quarantined, 0x0002)]),
        ("隔离记录两盘清成 0x0002（漏记号）", vec![(DeviceIdentity(0), quarantined, 0x0002), (DeviceIdentity(1), quarantined, 0x0002)]),
        ("B 引用着的活单元两盘带记号 0x4002", vec![(DeviceIdentity(0), live, 0x4002), (DeviceIdentity(1), live, 0x4002)]),
    ];
    eprintln!("ATTACK-T4 合法的隔离镜像：{} {}", core_readers_of_the_pool(&base), checker(&base));
    for (what, changes) in cases {
        let mut image = base.clone();
        for (device, slot, span_field) in changes {
            set_span_field(&mut image, device, slot, span_field);
        }
        eprintln!("ATTACK-T4 {what}：{} {}", core_readers_of_the_pool(&image), checker(&image));
    }
}

/// T3、T4：活单元两盘带记号的镜像（坏镜像）可写挂载之后覆盖写一次：那条记录释放时记号怎么处置、写出的那一版 checker 怎么判。
#[test]
fn t4_a_marked_live_unit_released_by_the_next_overwrite() {
    let (devices, _, b, _) = quarantined_after_b(&data_unit_of_a);
    let live = b.data_pointers[0].locations[0].slot;
    let mut image = image_of(&devices);
    set_span_field(&mut image, DeviceIdentity(0), live, 0x4002);
    set_span_field(&mut image, DeviceIdentity(1), live, 0x4002);
    let mut devices = devices_of(&image);
    let mounted = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => {
            eprintln!("ATTACK-T4 活单元带记号的镜像可写挂载拒：{}", short(refusal));
            return;
        }
    };
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content = pattern(800, 11);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        session.publish_user_change(&mut devices, UserChange::OverwriteOfAnInode { inode: FIRST_FILE, content: &content, write_time_seconds: WRITE_TIME_SECONDS })
    }));
    let text = match outcome {
        Err(payload) => format!("panic：{}", panic_text(payload.as_ref())),
        Ok(Err(refused)) => format!("拒：{}", short(refused)),
        Ok(Ok(_)) => {
            let after = image_of(&devices);
            format!("做成 盘上 {} {}", records_on_disk(&after, &[live]), checker(&after))
        }
    };
    eprintln!("ATTACK-T4 活单元带记号的镜像覆盖写：{text}");
}

// ── T1 扫：前缀固定（标签 0、size 3000 的镜像可写挂载、截到 0），之后由用户定的那几步放开扫 ──

fn leak(bytes: Vec<u8>) -> &'static [u8] {
    Box::leak(bytes.into_boxed_slice())
}

fn sweep_after_truncating_to_zero(label: &str, follow_up: Vec<UserChange<'static>>, size_after: u64) {
    let image = tag_zero_image();
    let mut devices = devices_of(image);
    let mounted = mount_writable(&parameters(), &mut devices).expect("标签 0 镜像可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let mut changes = vec![UserChange::Truncate { inode: FIRST_FILE, new_size: 0, write_time_seconds: WRITE_TIME_SECONDS }];
    changes.extend(follow_up);
    for (index, change) in changes.into_iter().enumerate() {
        let what = short(&change).chars().take(60).collect::<String>();
        match catch_unwind(AssertUnwindSafe(|| session.publish_user_change(&mut devices, change))) {
            Err(payload) => {
                eprintln!("ATTACK-T1扫 {label} 第{index}步 {what} panic：{}", panic_text(payload.as_ref()));
                return;
            }
            Ok(Err(refused)) => {
                eprintln!("ATTACK-T1扫 {label} 第{index}步 {what} 拒：{}", short(refused));
                return;
            }
            Ok(Ok(_)) => {}
        }
    }
    let after = image_of(&devices);
    let expected = vec![0u8; usize::try_from(size_after).expect("小")];
    eprintln!("ATTACK-T1扫 {label}：{} {}", readers(&after, &expected), checker(&after));
}

#[test]
fn t1_sweep_follow_ups_after_truncating_a_tag_zero_file_to_zero() {
    eprintln!("ATTACK-T1扫 标签0起点镜像：{}", checker(tag_zero_image()));
    let seconds = WRITE_TIME_SECONDS;
    sweep_after_truncating_to_zero("只截到0", vec![], 0);
    sweep_after_truncating_to_zero("再截到0一次（N=S）", vec![UserChange::Truncate { inode: FIRST_FILE, new_size: 0, write_time_seconds: seconds }], 0);
    sweep_after_truncating_to_zero("建1个inode", vec![UserChange::NewInodes { count: 1, write_time_seconds: seconds }], 0);
    sweep_after_truncating_to_zero(
        "建1个再覆盖写inode2",
        vec![
            UserChange::NewInodes { count: 1, write_time_seconds: seconds },
            UserChange::OverwriteOfAnInode { inode: InodeNumber(2), content: leak(pattern(100, 1)), write_time_seconds: seconds },
        ],
        0,
    );
    sweep_after_truncating_to_zero(
        "建1个再顺序写inode2两单元",
        vec![
            UserChange::NewInodes { count: 1, write_time_seconds: seconds },
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(2),
                content: leak(pattern(usize::try_from(payload()).expect("小") + 10, 2)),
                write_time_seconds: seconds,
            },
        ],
        0,
    );
    sweep_after_truncating_to_zero(
        "建142个再覆盖写inode143（上段长高）",
        vec![
            UserChange::NewInodes { count: 142, write_time_seconds: seconds },
            UserChange::OverwriteOfAnInode { inode: InodeNumber(143), content: leak(pattern(100, 3)), write_time_seconds: seconds },
        ],
        0,
    );
    sweep_after_truncating_to_zero(
        "建1个写inode2再unlink它",
        vec![
            UserChange::NewInodes { count: 1, write_time_seconds: seconds },
            UserChange::OverwriteOfAnInode { inode: InodeNumber(2), content: leak(pattern(100, 4)), write_time_seconds: seconds },
            UserChange::Unlink { inode: InodeNumber(2), write_time_seconds: seconds },
        ],
        0,
    );
    sweep_after_truncating_to_zero("建1个再unlink它（没写过）", vec![UserChange::NewInodes { count: 1, write_time_seconds: seconds }, UserChange::Unlink { inode: InodeNumber(2), write_time_seconds: seconds }], 0);
    sweep_after_truncating_to_zero("建快照", vec![UserChange::CreateSnapshot], 0);
    sweep_after_truncating_to_zero("变长到7000", vec![UserChange::Truncate { inode: FIRST_FILE, new_size: 7000, write_time_seconds: seconds }], 7000);
    sweep_after_truncating_to_zero(
        "建1个写inode2再变长到7000",
        vec![
            UserChange::NewInodes { count: 1, write_time_seconds: seconds },
            UserChange::OverwriteOfAnInode { inode: InodeNumber(2), content: leak(pattern(100, 5)), write_time_seconds: seconds },
            UserChange::Truncate { inode: FIRST_FILE, new_size: 7000, write_time_seconds: seconds },
        ],
        7000,
    );
    sweep_after_truncating_to_zero(
        "unlink第一个文件",
        vec![UserChange::Unlink { inode: FIRST_FILE, write_time_seconds: seconds }],
        0,
    );
}

// ── T5：把带记号与不带记号的两张镜像（只写过的扇区）落进 `ATTACK_IMAGE_DUMP_DIR`，交给 HEAD 那一版的旧读者读 ──

fn dump(image: &MemoryPool, path: &std::path::Path) {
    let mut bytes = Vec::new();
    for (identity, device) in &image.devices {
        for (sector, contents) in device.written_sectors() {
            bytes.extend_from_slice(&identity.0.to_le_bytes());
            bytes.extend_from_slice(&sector.to_le_bytes());
            bytes.extend_from_slice(contents);
        }
    }
    std::fs::write(path, bytes).expect("落盘");
}

#[test]
fn t5_dump_images_for_the_old_readers() {
    let Some(directory) = std::env::var_os("ATTACK_IMAGE_DUMP_DIR") else {
        eprintln!("ATTACK-T5 没设 ATTACK_IMAGE_DUMP_DIR，不落");
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let (devices, _, _, _) = quarantined_after_b(&data_unit_of_a);
    let marked = image_of(&devices);
    let (control_devices, _, _, _) = quarantined_after_b(&|_: &TransactionOutput| Vec::new());
    let control = image_of(&control_devices);
    dump(&marked, &directory.join("marked.sectors"));
    dump(&control, &directory.join("unmarked.sectors"));
    eprintln!("ATTACK-T5 落了 marked：{} unmarked：{}", marked_records_on_disk(&marked), marked_records_on_disk(&control));
}
