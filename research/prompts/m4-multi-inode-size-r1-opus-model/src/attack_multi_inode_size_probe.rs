//! 攻方腿 m4-multi-inode-size-r1 的探针（只在攻方副本里，不进仓）：第一个文件之外的 inode 改 size 之后，三处读者与池级 checker 各怎么判。
//! 池在内存稀疏盘上建（mkfs、取号、暖机、新池新建文件、建 inode 2 与 3、inode 3 写 100 字节），起点只建一次，每段历史从内存里那一份拷。
//! 副本里 `publish_size_change` 只删了「不是第一个文件而 N ≠ S 就拒」那一支，别处一行不改。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_deletion_in_progress::{change_the_inode_record, change_the_unit_and_propagate_its_checksum};
use common_pointer_head_corruption::{newest_root, the_slot_of, tree_table_entry_of_kind, unit_bytes_at, node_bytes};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::extent_tree::{ExtentUpperLeafEntry, ExtentUpperLeafTarget};
use singlefs_core::journal::JournalRecord;
use singlefs_harness::model_comparison::observed_inodes_other_than_the_first_of_file_version;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::records::{parse_mapping_entry, InodeRecord, TREE_KIND_EXTENT};
use singlefs_core::recovery::{rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write_of_an_inode,
    publish_size_change, walk_the_deletion_queue_until_it_is_empty, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const SECONDS: u64 = FIXED_WRITE_TIME_SECONDS + 100;

struct State {
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

fn sparse_device_with(image: &singlefs_harness::memory_pool::SparseDevice) -> SparseBlockDevice {
    let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
    device.image = image.clone();
    device
}

impl State {
    fn copy(&self) -> State {
        State {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, sparse_device_with(&device.image)))
                .collect(),
            allocator: self.allocator.clone(),
            output: self.output.clone(),
        }
    }
    fn memory_pool(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }
}

fn payload() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// mkfs、取号、暖机、新池新建文件（3000 字节）、建 inode 2 与 3（size 0）、inode 3 按 inode 号写 `inode_three_bytes` 字节。
fn starting_state(inode_three_bytes: usize) -> State {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
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
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let content = common::file_content();
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, INSTANCE);
        let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        let with_inodes =
            publish_new_inodes(&mut pool, &mut allocator, &first, 2, FIXED_WRITE_TIME_SECONDS, instance)
                .expect("建 inode 2、3");
        let bytes: Vec<u8> = (0..inode_three_bytes).map(|i| u8::try_from(i % 251).expect("字节")).collect();
        publish_sequential_write_of_an_inode(
            &mut pool,
            &mut allocator,
            &with_inodes,
            InodeNumber(3),
            &bytes,
            FIXED_WRITE_TIME_SECONDS,
            instance,
        )
        .expect("inode 3 写数据")
    };
    State { devices, allocator, output }
}

fn change_size(state: &mut State, inode: u64, size: u64) -> String {
    let parameters = parameters();
    let previous = state.output.clone();
    let mut pool = PoolWriter::new(&parameters, &mut state.devices);
    match publish_size_change(&mut pool, &mut state.allocator, &previous, InodeNumber(inode), size, SECONDS, INSTANCE) {
        Ok(output) => {
            state.output = output;
            format!("publish_size_change(inode {inode}, {size}) 发成")
        }
        Err(error) => format!("publish_size_change(inode {inode}, {size}) 拒：{error:?}"),
    }
}

fn walk(state: &mut State) -> String {
    let parameters = parameters();
    let mut current = state.output.clone();
    let mut pool = PoolWriter::new(&parameters, &mut state.devices);
    let walked = walk_the_deletion_queue_until_it_is_empty(
        &mut pool,
        &mut state.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE,
    );
    state.output = current;
    format!("走空：发成 {} 步，停在 {:?}", walked.steps_published.len(), walked.stopped_by.map_err(|e| format!("{e:?}")))
}

fn short(text: String) -> String {
    text.chars().take(420).collect()
}

/// 三处读者与池级 checker 在这一份镜像上各自的判法，逐行打出来。
fn judge(name: &str, image: &MemoryPool, inode: u64, record: &JournalRecord) {
    let root = newest_root(image);
    let mounted = match open_pool_for_read(image, &root) {
        Ok(mounted) => match mounted.open_file(image, InodeNumber(inode)) {
            Ok(file) => {
                let size = file.inode_record().size;
                match file.read_at(image, FileOffsetInBytes(0), size) {
                    Ok(read) => format!("收：size {size}、读出 {} 字节、非零 {} 个", read.bytes.len(), read.bytes.iter().filter(|b| **b != 0).count()),
                    Err(error) => format!("读拒：{error:?}"),
                }
            }
            Err(error) => format!("打开拒：{error:?}"),
        },
        Err(error) => format!("开池拒：{error:?}"),
    };
    println!("name={name} reader=mounted_read inode={inode} {}", short(mounted));
    let rebuilt = match rebuild_version(image, &root, Some(record.clone())) {
        Ok(RebuiltVersion::WithFile(version)) => {
            let record = version
                .inode_leaf_containers
                .iter()
                .flat_map(|container| container.contents.records.iter())
                .find(|record| record.inode == inode)
                .map(|record| record.size);
            format!("收：inode {inode} 的 size {record:?}、第一个文件之外有单元的文件 {:?}", version.files_other_than_the_first.iter().map(|file| (file.inode.0, file.data_pointers.len())).collect::<Vec<_>>())
        }
        Ok(other) => format!("收（别的成员）：{}", short(format!("{other:?}"))),
        Err(error) => format!("拒：{error:?}"),
    };
    println!("name={name} reader=rebuild_version inode={inode} {}", short(rebuilt));
    let recovered = match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => format!("读回第一个文件 {} 字节", content.len()),
        RecoveryOutcome::NoFile { .. } => "没有文件".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("拒：{failure:?}"),
    };
    println!("name={name} reader=recover {}", short(recovered));
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, device)| (*identity, sparse_device_with(device)))
        .collect();
    let writable = match mount_writable(&parameters(), &mut devices) {
        Ok(_mounted) => "收".to_string(),
        Err(error) => format!("拒：{error:?}"),
    };
    println!("name={name} reader=mount_writable {}", short(writable));
    let verdicts = check_pool_image(image);
    let violated: Vec<String> = verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}〔{}〕", detail.chars().take(200).collect::<String>())),
            _ => None,
        })
        .collect();
    println!("name={name} checker=pool judged={} violated={} {}", verdicts.len(), violated.len(), violated.join(" ; "));
}

/// inode `inode` 在最新那一版上段叶里的条目标签（没有条目交 None）；上段只有根兼叶。
fn upper_tag_of(image: &MemoryPool, inode: u64) -> Option<String> {
    let slot = the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root);
    let node = parse_index_node(&unit_bytes_at(image, slot, node_bytes())).expect("上段根解得开");
    node.entries
        .iter()
        .map(|entry| ExtentUpperLeafEntry::parse(entry).expect("上段叶条目解得开"))
        .find(|entry| entry.inode == inode)
        .map(|entry| format!("{:?}", entry.target).chars().take(40).collect())
}


fn record_of(output: &TransactionOutput, inode: u64) -> Option<InodeRecord> {
    output
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode)
        .copied()
}

/// 码 2 节点的条目整份交给 `change` 改（照 `whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader.rs` 的同名函数抄）：
/// 条目数、声明长度跟着改，`follows` 为真时 key 区间跟着条目改，重封、校验和补到根槽。
fn rewrite_the_entries_of_the_index_node_at(
    image: &mut MemoryPool,
    slot: singlefs_core::address::SlotNumber,
    follows: bool,
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
            let start = entries_start + position * entry_width;
            node[start..start + entry_width].copy_from_slice(entry);
        }
        let count_offset = 82 + 2 * key_width;
        node[count_offset..count_offset + 2].copy_from_slice(&u16::try_from(entries.len()).expect("条目数").to_le_bytes());
        node[8..10].copy_from_slice(&u16::try_from(entries.len() * entry_width).expect("声明长度").to_le_bytes());
        if let (true, Some(first), Some(last)) = (follows, entries.first(), entries.last()) {
            node[52..52 + key_width].copy_from_slice(&first[..key_width]);
            node[52 + key_width..52 + 2 * key_width].copy_from_slice(&last[..key_width]);
        }
    });
}

/// 「别的文件变长也写标签 0」那一臂的目标形：起点镜像上 inode 3（内联一个单元、100 字节）那一条上段叶条目改成标签 0、载荷清零，
/// 那个数据单元的映射条目拿掉，记录 size 改成 `size`。
fn image_with_inode_three_as_a_tag_zero_hole(base: &State, size: u64) -> MemoryPool {
    let mut image = base.memory_pool();
    let extent_root_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root);
    let node = parse_index_node(&unit_bytes_at(&image, extent_root_slot, node_bytes())).expect("上段根解得开");
    let data_unit = node
        .entries
        .iter()
        .map(|entry| ExtentUpperLeafEntry::parse(entry).expect("条目"))
        .find(|entry| entry.inode == 3)
        .map(|entry| match entry.target {
            ExtentUpperLeafTarget::InlineDataUnit(pointer) => pointer,
            other => panic!("前提：inode 3 内联一个单元，实际 {other:?}"),
        })
        .expect("上段叶里有 inode 3");
    let mapping_root_slot = the_slot_of(&newest_root(&image).mapping_root);
    rewrite_the_entries_of_the_index_node_at(&mut image, mapping_root_slot, true, |entries| {
        let before = entries.len();
        entries.retain(|entry| {
            let (_key, locations) = parse_mapping_entry(entry).expect("映射条目");
            locations[0].slot != data_unit.locations[0].slot
        });
        assert_eq!(entries.len() + 1, before, "恰好一条映射条目落在 inode 3 的单元上");
    });
    rewrite_the_entries_of_the_index_node_at(&mut image, extent_root_slot, false, |entries| {
        let entry = entries
            .iter_mut()
            .find(|entry| u64::from_le_bytes(entry[8..16].try_into().expect("8")) == 3)
            .expect("inode 3 那一条");
        entry[24] = 0;
        entry[25..].fill(0);
    });
    change_the_inode_record(&mut image, 3, |bytes| {
        let mut record = InodeRecord::parse(bytes).expect("记录解得开");
        record.size = size;
        bytes.copy_from_slice(&record.to_bytes());
    });
    image
}

#[test]
fn attack_probe_size_changes_of_inodes_other_than_the_first() {
    let started = std::time::Instant::now();
    let base = starting_state(100);
    let multi = starting_state(usize::try_from(3 * payload()).expect("长度"));
    println!("name=start inode3_bytes=100 inode2_tag={:?} inode3_tag={:?}", upper_tag_of(&base.memory_pool(), 2), upper_tag_of(&base.memory_pool(), 3));
    println!("name=start-multi inode3_units={:?}", multi.output.files_other_than_the_first.iter().map(|f| (f.inode.0, f.data_pointers.len())).collect::<Vec<_>>());
    judge("start", &base.memory_pool(), 3, &base.output.record);

    // H1：inode 2（size 0、上段没有条目）变长到 5000，经副本的写者。
    let mut h1 = base.copy();
    println!("name=h1 {}", change_size(&mut h1, 2, 5000));
    println!("name=h1 inode2_tag={:?}", upper_tag_of(&h1.memory_pool(), 2));
    judge("h1", &h1.memory_pool(), 2, &h1.output.record);
    println!("name=h1 model_glue={:?}", observed_inodes_other_than_the_first_of_file_version(&h1.output).get(&2));

    // H1 手造：不经写者，起点镜像上 inode 2 那条记录的 size 直接改成 5000（重封叶、补校验和到根）。
    let mut crafted = base.memory_pool();
    change_the_inode_record(&mut crafted, 2, |bytes| {
        let mut record = InodeRecord::parse(bytes).expect("记录解得开");
        record.size = 5000;
        bytes.copy_from_slice(&record.to_bytes());
    });
    judge("h1-crafted", &crafted, 2, &base.output.record);

    // H2：inode 3（内联一个单元、100 字节）变长到 5000。
    let mut h2 = base.copy();
    println!("name=h2 {}", change_size(&mut h2, 3, 5000));
    judge("h2", &h2.memory_pool(), 3, &h2.output.record);

    // H3：inode 3 变长跨单元（100 → 2 个净荷 + 7）。
    let mut h3 = base.copy();
    println!("name=h3 {}", change_size(&mut h3, 3, 2 * payload() + 7));
    println!("name=h3 inode3_tag={:?}", upper_tag_of(&h3.memory_pool(), 3));
    judge("h3", &h3.memory_pool(), 3, &h3.output.record);

    // H4：inode 3（100 字节）截到 50：意图那次，再共用走空。
    let mut h4 = base.copy();
    println!("name=h4 {}", change_size(&mut h4, 3, 50));
    judge("h4-intent", &h4.memory_pool(), 3, &h4.output.record);
    println!("name=h4-intent model_glue={:?}", observed_inodes_other_than_the_first_of_file_version(&h4.output).get(&3).map(|f| format!("{f:?}").chars().take(160).collect::<String>()));
    println!("name=h4-intent inode3_truncation_field={:?}", record_of(&h4.output, 3).map(|r| r.truncation_field));
    println!("name=h4 {}", walk(&mut h4));
    judge("h4-walked", &h4.memory_pool(), 3, &h4.output.record);

    // H5：inode 3（三个满单元）截到 100：意图那次，再共用走空。
    let mut h5 = multi.copy();
    println!("name=h5 {}", change_size(&mut h5, 3, 100));
    judge("h5-intent", &h5.memory_pool(), 3, &h5.output.record);
    println!("name=h5-intent model_glue={:?}", observed_inodes_other_than_the_first_of_file_version(&h5.output).get(&3).map(|f| format!("{f:?}").chars().take(160).collect::<String>()));
    println!("name=h5-intent inode3_truncation_field={:?}", record_of(&h5.output, 3).map(|r| r.truncation_field));
    println!("name=h5 {}", walk(&mut h5));
    judge("h5-walked", &h5.memory_pool(), 3, &h5.output.record);
    // H6 手造：「别的文件也写标签 0」那一臂的目标形（inode 3 标签 0、size 5000），与它的对照：同一形把 size 留 0。
    let tag_zero = image_with_inode_three_as_a_tag_zero_hole(&base, 5000);
    println!("name=h6 inode3_tag={:?}", upper_tag_of(&tag_zero, 3));
    judge("h6-tag-zero-size-5000", &tag_zero, 3, &base.output.record);
    let tag_zero_empty = image_with_inode_three_as_a_tag_zero_hole(&base, 0);
    judge("h6-tag-zero-size-0", &tag_zero_empty, 3, &base.output.record);
    println!("name=elapsed seconds={}", started.elapsed().as_secs());
}
