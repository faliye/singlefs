//! 攻方原型（m4-batch2-code-r1）：T1 / T2 合法与边界快照树表在各读者与池级 checker 上的结局；T4 / T5 真写者写出的各版上区域扫描的分类与选段。
//! 只读被测代码、不改产品代码；盘都是内存稀疏盘。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::copy_each_root_into_the_new_root_segment_of_its_journal_records;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, SlotNumber, TreeIdentifier};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    choose_region, plan_batch, CompactionBatchUnits, RegionChoiceCriterion, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::records::{
    TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_SNAPSHOT_EXTENT_ROOT,
    TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, every_root_ring_slot,
    read_root_ring_slot, rebuild_version, user_visible_tree_root_pointers, walk_to_file,
    PoolReader, RootRingSlotReading,
};
use singlefs_core::region_scan::{scan_region, AllocationInRegion, CentralMappingEntry};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::slot_offset;
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file,
    publish_new_inodes, publish_overwrite, publish_sequential_write, warm_up, FirstFile,
    MappingLookup, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_format::{NODE_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const WRITE_TIME_SECONDS: u64 = 1_788_100_000;

fn parameters() -> MakeFilesystemParameters {
    common::parameters()
}

fn content_of(length: usize, salt: u32) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let index = u32::try_from(index).expect("长度");
            u8::try_from((index.wrapping_mul(137) + salt) % 251).expect("小于 256")
        })
        .collect()
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}


/// mkfs → 取号 → 暖机 → 第一个文件，再按 `steps` 走前几步（建 300 个空 inode → 顺序写 200 KiB → 覆盖写）：最后那一版的镜像与发布输出。
fn version_after(steps: usize) -> (String, MemoryPool, TransactionOutput) {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator: PoolAllocator =
        allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first_content = content_of(3000, 3);
    let second_content = content_of(200 * 1024, 7);
    let third_content = content_of(5000, 11);
    let (name, last) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let mut last = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        let mut name = "第一个文件";
        if steps >= 1 {
            last = publish_new_inodes(
                &mut writer,
                &mut allocator,
                &last,
                300,
                WRITE_TIME_SECONDS,
                instance,
            )
            .expect("建 300 个 inode");
            name = "第一个文件 + 建 300 个 inode";
        }
        if steps >= 2 {
            last = publish_sequential_write(
                &mut writer,
                &mut allocator,
                &last,
                FirstFile {
                    content: &second_content,
                    write_time_seconds: WRITE_TIME_SECONDS,
                },
                instance,
            )
            .expect("顺序写 200 KiB");
            name = "第一个文件 + 建 300 个 inode + 顺序写 200 KiB";
        }
        if steps >= 3 {
            last = publish_overwrite(
                &mut writer,
                &mut allocator,
                &last,
                FirstFile {
                    content: &third_content,
                    write_time_seconds: WRITE_TIME_SECONDS,
                },
                instance,
            )
            .expect("覆盖写");
            name = "第一个文件 + 建 300 个 inode + 顺序写 200 KiB + 覆盖写 5000 字节";
        }
        (name, last)
    };
    (name.to_string(), image_of(&devices), last)
}

/// 这一版中央映射里每个进映射的单元一条（key 与它的两条位置条目），按写者记的 key 在这一版的映射叶里现查。
fn mapping_entries_of(version: &TransactionOutput) -> Vec<CentralMappingEntry> {
    version
        .mapped_units
        .iter()
        .map(|(role, key)| match central_mapping_locations_in_the_version(version, key) {
            MappingLookup::Found(locations) => CentralMappingEntry {
                key: key.clone(),
                locations,
            },
            other => panic!("{role:?} 的 key 在这一版映射里查不到：{other:?}"),
        })
        .collect()
}

fn class_name(class: &AllocationInRegion) -> String {
    match class {
        AllocationInRegion::Movable(_) => "可搬".to_string(),
        AllocationInRegion::Pinned => "钉住".to_string(),
        AllocationInRegion::Quarantined => "隔离".to_string(),
        AllocationInRegion::Dead => "已死".to_string(),
        AllocationInRegion::Inconsistent(why) => format!("不一致 {why:?}"),
    }
}

#[test]
#[ignore = "攻方原型：手动跑"]
fn region_scan_classes_on_versions_written_by_the_writer() {
    let unit_area_start = UnitAreaStart::of_the_default_journal_ring();
    let unit_area_slots = DEVICE_BYTES / SLOT_BYTES - unit_area_start.slot().0;
    for steps in 0..4 {
        let (name, image, version) = version_after(steps);
        let mapping = mapping_entries_of(&version);
        println!("ATTACK_ROOT 版本「{name}」 实例表指针 {:?}", version.root.instance_table);
        let mapped_keys: BTreeSet<Vec<u8>> = mapping.iter().map(|entry| entry.key.clone()).collect();
        let role_of_slot: BTreeMap<u64, String> = version
            .units
            .iter()
            .map(|unit| (unit.slot.0, format!("{:?}", unit.identity)))
            .collect();
        let exempt_slots: BTreeSet<SlotNumber> = version
            .units
            .iter()
            .filter(|unit| {
                !version
                    .mapped_units
                    .iter()
                    .any(|(role, _key)| *role == unit.identity)
            })
            .map(|unit| unit.slot)
            .collect();
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let segment_starts: BTreeSet<u64> = version
                .allocation_records
                .iter()
                .filter(|record| record.device == device && !record.is_released)
                .map(|record| {
                    let start = unit_area_start.slot().0;
                    start + (record.slot.0 - start) / 64 * 64
                })
                .collect();
            let mut tally: BTreeMap<String, u64> = BTreeMap::new();
            for segment_start in &segment_starts {
                let region = CompactionRegion::of_one_cluster_segment(device, SlotNumber(*segment_start))
                    .expect("段起点 64 对齐");
                let scan = scan_region(
                    region,
                    &version.allocation_records,
                    &mapping,
                    &BTreeSet::new(),
                    &BTreeSet::new(),
                    |read_device, slot, length| PoolReader::read(&image, read_device, slot.to_device_offset(), length),
                )
                .expect("写者的记录不跨段");
                for allocation in &scan.allocations {
                    let role = role_of_slot
                        .get(&allocation.placement.slot.0)
                        .cloned()
                        .unwrap_or_else(|| "（这一版没有这个槽上的角色）".to_string());
                    let label = match &allocation.class {
                        AllocationInRegion::Movable(_) => "可搬".to_string(),
                        other => format!("{}｜{}｜槽 {}", class_name(other), role, allocation.placement.slot.0),
                    };
                    *tally.entry(label).or_insert(0) += 1;
                }
            }
            println!("ATTACK_SCAN 版本「{name}」 盘 {} 段数 {} 映射条目 {}：{tally:?}", device.0, segment_starts.len(), mapped_keys.len());
        }
        let chosen = choose_region(
            &version.allocation_records,
            UnitAreaOfOneDevice {
                device: DeviceIdentity(0),
                start: unit_area_start,
                slots: unit_area_slots,
            },
            &BTreeSet::new(),
            None,
            &exempt_slots,
            RegionChoiceCriterion::Greedy,
        );
        match chosen {
            Some(region) => {
                let scan = scan_region(
                    region,
                    &version.allocation_records,
                    &mapping,
                    &exempt_slots,
                    &BTreeSet::new(),
                    |read_device, slot, length| PoolReader::read(&image, read_device, slot.to_device_offset(), length),
                )
                .expect("写者的记录不跨段");
                let plan = plan_batch(&scan, CompactionBatchUnits::new(3).expect("3"));
                let classes: Vec<String> = scan.allocations.iter().map(|allocation| class_name(&allocation.class)).collect();
                println!("ATTACK_CHOOSE 版本「{name}」 钉住 {} 槽：选中 R 起点 {}，R 里各项 {classes:?}，一批 3 交 {} 项", exempt_slots.len(), region.start_slot().0, plan.len());
            }
            None => println!("ATTACK_CHOOSE 版本「{name}」 钉住 {} 槽：一段都没资格", exempt_slots.len()),
        }
    }
}

/// 树表条目里 `previous_snapshot_txg` 那 8 字节的偏移（树 ID 8 + 长度 2 + 种类 2 + flags 2 + 根指针 86，D8 已定项 8）。
const PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 100;
/// flags 2 字节的偏移。
const FLAGS_OFFSET: usize = 12;

fn with_u64_at(mut entry: Vec<u8>, offset: usize, value: u64) -> Vec<u8> {
    entry[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    entry
}

/// 最新那条根的树表单元换成 `entries`（照原单元的头重建、两份写回、校验和补进根），树 ID 水位换成 `watermark`，根写回它的槽，
/// journal 记录的新根段照抄（照 `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 的改法，多了重建单元与水位两样）。
fn rewrite_the_newest_root_with_tree_table_entries(
    image: &mut MemoryPool,
    entries: &[Vec<u8>],
    watermark: u64,
) -> RootRecord {
    let system_configuration = choose_system_configuration(&*image).expect("系统配置");
    let sizes = system_configuration.immutable.sizes;
    let region_devices = system_configuration.immutable.region_devices;
    let (ring_slot, mut root) = every_root_ring_slot(&sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                &*image,
                &region_devices,
                &sizes,
                &system_configuration.immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .expect("根环里有自证过的根");
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let first_location = root.tree_table.locations[0];
    let old = PoolReader::read(&*image, first_location.device, first_location.slot.to_device_offset(), node_bytes)
        .expect("树表单元读得到");
    let header = parse_index_node(&old).expect("树表单元解得开");
    let key_of = |entry: &Vec<u8>| entry[..8].to_vec();
    let rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &key_of(entries.first().expect("有条目")),
        &key_of(entries.last().expect("有条目")),
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("200"),
        entries,
    );
    let checksum = crc32_castagnoli(&rebuilt);
    for location in &mut root.tree_table.locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(location.slot.to_device_offset(), &rebuilt);
        location.unit_checksum = checksum;
    }
    root.tree_identifier_watermark = watermark;
    let root_device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let root_slot_bytes = usize::try_from(sizes.physical_block_size).expect("根槽宽");
    image
        .devices
        .get_mut(&root_device)
        .expect("池里有这块盘")
        .write(
            slot_offset(ring_slot, sizes.fixed_structure_slot_spacing),
            &root.to_slot(root_slot_bytes),
        );
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
    root
}

fn report_every_reader(label: &str, image: &MemoryPool, root: &RootRecord, version: &TransactionOutput) {
    let mut fallbacks = 0;
    println!("ATTACK_READER {label} rebuild_version: {:?}", rebuild_version(image, root, Some(version.record.clone())).map(|_| ()));
    println!("ATTACK_READER {label} allocation_records_under_root: {:?}", allocation_records_under_root(image, root).map(|_| ()));
    println!("ATTACK_READER {label} walk_to_file: {:?}", walk_to_file(image, root, &mut fallbacks).map(|_| ()));
    println!("ATTACK_READER {label} user_visible_tree_root_pointers: {:?}", user_visible_tree_root_pointers(image, root).map(|_| ()));
    println!("ATTACK_READER {label} open_pool_for_read: {:?}", open_pool_for_read(image, root).map(|_| ()));
    let mut not_holding = 0;
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Holds => {}
            InvariantVerdict::Violated(detail) => {
                not_holding += 1;
                println!("ATTACK_CHECKER {label} {invariant} 违例：{detail}");
            }
            InvariantVerdict::NotApplicable(reason) => {
                println!("ATTACK_CHECKER_NA {label} {invariant}：{reason}");
            }
            InvariantVerdict::PartiallyJudged(detail) => {
                println!("ATTACK_CHECKER {label} {invariant} 部分没判：{detail}");
            }
        }
    }
    println!("ATTACK_CHECKER {label} 违例条数 {not_holding}");
}

#[test]
#[ignore = "攻方原型：手动跑"]
fn snapshot_tree_tables_on_every_reader_and_the_pool_checker() {
    let (_name, clean, version) = version_after(0);
    let original: Vec<Vec<u8>> = version.tree_table_entries.iter().map(TreeTableEntry::to_bytes).collect();
    let live_inode = *version.tree_table_entries.iter().find(|entry| entry.kind == TREE_KIND_INODE).expect("inode");
    let live_extent = *version.tree_table_entries.iter().find(|entry| entry.kind == TREE_KIND_EXTENT).expect("extent");
    let watermark = version.root.tree_identifier_watermark;
    let txg = version.root.checkpoint_txg.0;
    let pair = |position: u64, snapshot_txg: u64| {
        let mut inode_root = live_inode;
        inode_root.kind = TREE_KIND_SNAPSHOT_INODE_ROOT;
        inode_root.tree = TreeIdentifier(watermark + 2 * position);
        inode_root.birth_txg.0 = snapshot_txg;
        inode_root.head_identifier = live_inode.tree.0;
        let mut extent_root = live_extent;
        extent_root.kind = TREE_KIND_SNAPSHOT_EXTENT_ROOT;
        extent_root.tree = TreeIdentifier(watermark + 2 * position + 1);
        extent_root.birth_txg.0 = snapshot_txg;
        extent_root.head_identifier = watermark + 2 * position;
        (inode_root, extent_root)
    };
    let (first_inode_root, first_extent_root) = pair(0, txg - 1);
    let (second_inode_root, second_extent_root) = pair(1, txg);
    // 一、对照：不改树表，原样重写一遍（水位不变）。
    let mut unchanged = clean.clone();
    let root = rewrite_the_newest_root_with_tree_table_entries(&mut unchanged, &original, watermark);
    report_every_reader("对照（原样）", &unchanged, &root, &version);
    // 二、照判决 S1 / S2 字段取值的两对快照条目，previous_snapshot_txg 照 S1 / S2 填：活头 inode 条目 = 第二个快照的 S.txg、
    //     第一个码 14 = 0、第二个码 14 = 第一个快照的 S.txg、码 15 = 0。
    let mut entries: Vec<Vec<u8>> = original
        .iter()
        .map(|bytes| {
            let entry = TreeTableEntry::parse(bytes).expect("写者写的条目");
            if entry.kind == TREE_KIND_INODE {
                with_u64_at(bytes.clone(), PREVIOUS_SNAPSHOT_TXG_OFFSET, txg)
            } else {
                bytes.clone()
            }
        })
        .collect();
    entries.push(first_inode_root.to_bytes());
    entries.push(first_extent_root.to_bytes());
    entries.push(with_u64_at(second_inode_root.to_bytes(), PREVIOUS_SNAPSHOT_TXG_OFFSET, txg - 1));
    entries.push(second_extent_root.to_bytes());
    let mut with_snapshots = clean.clone();
    let root = rewrite_the_newest_root_with_tree_table_entries(&mut with_snapshots, &entries, watermark + 4);
    report_every_reader("两对快照（S1 字段取值）", &with_snapshots, &root, &version);
    // 三、同上但 previous_snapshot_txg 一律 0（今天写者的写法）。
    let mut zero_previous: Vec<Vec<u8>> = original.clone();
    zero_previous.extend([first_inode_root, first_extent_root, second_inode_root, second_extent_root].iter().map(TreeTableEntry::to_bytes));
    let mut with_zero_previous = clean.clone();
    let root = rewrite_the_newest_root_with_tree_table_entries(&mut with_zero_previous, &zero_previous, watermark + 4);
    report_every_reader("两对快照（前驱全 0）", &with_zero_previous, &root, &version);
    // 四、第一个码 14 带 flags 位 0（判决 S1「flags 位 0 留给销毁中」）。
    let mut destroying: Vec<Vec<u8>> = zero_previous.clone();
    let position = original.len();
    destroying[position][FLAGS_OFFSET] = 1;
    println!("ATTACK_PARSE flags 位 0 的码 14：TreeTableEntry::parse = {:?}", TreeTableEntry::parse(&destroying[position]).map(|entry| entry.kind));
}

/// T1 探针：两对快照，两条码 15 的头 ID 都写成第二个码 14 的树号（第一条码 15 本该配第一个码 14）。冻结副本上配不成对、判树表损坏；
/// 配对判定里「头 ID 等于树 ID」的等号改成「不大于」之后照样过六条用例，这张表却被收下、第一条码 15 配给了第一个码 14。
#[test]
#[ignore = "攻方原型：手动跑"]
fn two_extent_roots_both_naming_the_second_inode_root() {
    let (_name, clean, version) = version_after(0);
    let original: Vec<Vec<u8>> = version.tree_table_entries.iter().map(TreeTableEntry::to_bytes).collect();
    let live_inode = *version.tree_table_entries.iter().find(|entry| entry.kind == TREE_KIND_INODE).expect("inode");
    let live_extent = *version.tree_table_entries.iter().find(|entry| entry.kind == TREE_KIND_EXTENT).expect("extent");
    let watermark = version.root.tree_identifier_watermark;
    let txg = version.root.checkpoint_txg.0;
    let mut entries = original.clone();
    for position in 0..2u64 {
        let mut inode_root = live_inode;
        inode_root.kind = TREE_KIND_SNAPSHOT_INODE_ROOT;
        inode_root.tree = TreeIdentifier(watermark + 2 * position);
        inode_root.birth_txg.0 = txg;
        inode_root.head_identifier = live_inode.tree.0;
        let mut extent_root = live_extent;
        extent_root.kind = TREE_KIND_SNAPSHOT_EXTENT_ROOT;
        extent_root.tree = TreeIdentifier(watermark + 2 * position + 1);
        extent_root.birth_txg.0 = txg;
        extent_root.head_identifier = watermark + 2;
        entries.push(inode_root.to_bytes());
        entries.push(extent_root.to_bytes());
    }
    let mut image = clean.clone();
    let root = rewrite_the_newest_root_with_tree_table_entries(&mut image, &entries, watermark + 4);
    let rebuilt = singlefs_core::recovery::rebuild_version_with_its_snapshot_entries(&image, &root, Some(version.record.clone()));
    match rebuilt {
        Ok(rebuilt) => {
            for pair in &rebuilt.snapshot_entries {
                println!(
                    "ATTACK_PAIR 收下：码 14 树 {} ↔ 码 15 树 {}（码 15 头 ID {}）",
                    pair.inode_root.tree.0, pair.extent_root.tree.0, pair.extent_root.head_identifier
                );
            }
        }
        Err(refusal) => println!("ATTACK_PAIR 拒：{refusal:?}"),
    }
}

/// T5 探针：一条跨段的坏记录（起点在第 0 段末槽、跨 2）。选段按槽数把它算进两段、第 0 段活槽最少被选中；区域扫描整次拒；
/// 选段的输入里没有「上一次被扫描拒了的 R」，再选一次还是它。
#[test]
#[ignore = "攻方原型：手动跑"]
fn a_straddling_record_makes_greedy_choose_the_refused_region_every_time() {
    use singlefs_core::address::CheckpointTxg;
    use singlefs_core::allocator::AllocationRecord;
    let unit_area_start = UnitAreaStart::of_the_default_journal_ring();
    let start = unit_area_start.slot().0;
    let record = |slot: u64, span: u16| AllocationRecord {
        device: DeviceIdentity(0),
        slot: SlotNumber(slot),
        span_slots: span,
        generation: CheckpointTxg(3),
        is_released: false,
    };
    let mut records = vec![record(start + 63, 2)];
    for unit in 0..10 {
        records.push(record(start + 128 + 2 * unit, 2));
    }
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: unit_area_start,
        slots: 64 * 4,
    };
    for attempt in 0..2 {
        let chosen = choose_region(&records, unit_area, &BTreeSet::new(), None, &BTreeSet::new(), RegionChoiceCriterion::Greedy);
        let scanned = chosen.map(|region| scan_region(region, &records, &[], &BTreeSet::new(), &BTreeSet::new(), |_, _, _| None).map(|scan| scan.allocations.len()));
        println!("ATTACK_STRADDLE 第 {attempt} 次：选中 {:?}，扫描 {:?}", chosen.map(|region| region.start_slot().0 - start), scanned);
    }
}

/// T2 探针：带两对快照条目（S1 字段取值、前驱全 0）的镜像可写挂载一次：挂载收不收、收下之后现行那一版与盘上最新根的树表里还剩几条码 14 / 15。
#[test]
#[ignore = "攻方原型：手动跑"]
fn a_writable_mount_of_a_pool_carrying_snapshot_entries() {
    use singlefs_core::mount::mount_writable;
    use singlefs_core::recovery::tree_table_entry_bytes_of_root;
    use singlefs_core::transaction::PoolVersion;
    let (_name, clean, version) = version_after(0);
    let original: Vec<Vec<u8>> = version.tree_table_entries.iter().map(TreeTableEntry::to_bytes).collect();
    let live_inode = *version.tree_table_entries.iter().find(|entry| entry.kind == TREE_KIND_INODE).expect("inode");
    let live_extent = *version.tree_table_entries.iter().find(|entry| entry.kind == TREE_KIND_EXTENT).expect("extent");
    let watermark = version.root.tree_identifier_watermark;
    let txg = version.root.checkpoint_txg.0;
    let mut entries = original.clone();
    for position in 0..2u64 {
        let mut inode_root = live_inode;
        inode_root.kind = TREE_KIND_SNAPSHOT_INODE_ROOT;
        inode_root.tree = TreeIdentifier(watermark + 2 * position);
        inode_root.birth_txg.0 = txg;
        inode_root.head_identifier = live_inode.tree.0;
        let mut extent_root = live_extent;
        extent_root.kind = TREE_KIND_SNAPSHOT_EXTENT_ROOT;
        extent_root.tree = TreeIdentifier(watermark + 2 * position + 1);
        extent_root.birth_txg.0 = txg;
        extent_root.head_identifier = watermark + 2 * position;
        entries.push(inode_root.to_bytes());
        entries.push(extent_root.to_bytes());
    }
    let mut image = clean.clone();
    let root = rewrite_the_newest_root_with_tree_table_entries(&mut image, &entries, watermark + 4);
    let snapshot_kinds_of = |entry_bytes: &[Vec<u8>]| entry_bytes
        .iter()
        .filter_map(|bytes| TreeTableEntry::parse(bytes))
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT)
        .count();
    println!("ATTACK_MOUNT 挂载之前最新根（txg {}）树表里码 14 / 15 共 {} 条", root.checkpoint_txg.0, snapshot_kinds_of(&tree_table_entry_bytes_of_root(&image, &root).expect("树表读得出")));
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect();
    match mount_writable(&parameters(), &mut devices) {
        Err(refusal) => println!("ATTACK_MOUNT 可写挂载拒：{refusal:?}"),
        Ok(mounted) => {
            let current_root = match &mounted.current {
                PoolVersion::WithFile(current) => {
                    println!("ATTACK_MOUNT 现行那一版（内存里）树表 {} 条，码 14 / 15 {} 条", current.tree_table_entries.len(), current.tree_table_entries.iter().filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT).count());
                    current.root
                }
                other => panic!("现行那一版不带文件：{other:?}"),
            };
            let after = image_of(&devices);
            println!("ATTACK_MOUNT 挂载之后盘上最新根（txg {}）树表里码 14 / 15 共 {} 条", current_root.checkpoint_txg.0, snapshot_kinds_of(&tree_table_entry_bytes_of_root(&after, &current_root).expect("树表读得出")));
        }
    }
}
