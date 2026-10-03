//! m3-ring-r1 攻方原型的共用件：在内存稀疏盘上造「记录已持久、根槽没持久」的洞，录可写挂载那一条流，量盘上的门槛与多扣。
//! 只在攻方副本里有；`singlefs-core` 的补洞挂点开关是副本上的改动（`mount::set_hole_fill_point`）。
#![allow(clippy::missing_panics_doc, clippy::must_use_candidate)]

use std::collections::BTreeSet;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, effective_rollback_floor,
    instance_table_of_root, readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table,
    PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{target_for_publish, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, PublishedVersion, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"M3RING-R1-OPUS01";
pub const IMAGE_BYTES: u64 = 4 << 30;
pub const WRITE_TIME: u64 = 1_788_000_000;
const INSTANCE_TABLE_SPAN_SLOTS: u64 = 2;

pub fn parameters(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("S 在 4..16"),
        },
    }
}

pub fn geometry(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

pub fn content(salt: u64) -> Vec<u8> {
    let salt = usize::try_from(salt % 251).expect("小");
    (0..3000usize)
        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256"))
        .collect()
}

pub type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

pub fn devices_of(pool: &MemoryPool) -> Devices {
    pool.devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = image.clone();
            (*identity, device)
        })
        .collect()
}

pub fn recording_of(
    pool: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_of(pool)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 洞落在哪一类：前洞（环还没转过来，slot(h) 从没写过）、后洞（slot(h) 留着 h − 3S 的根）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HolePosition {
    Front,
    Back,
}

/// 挂载之前的前缀：mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 h − 1 → txg h 的覆盖写在根槽写之前崩溃。
pub struct Prefix {
    pub slots_per_region: u64,
    pub hole: u64,
    pub image: MemoryPool,
    pub versions: Vec<PublishedVersion>,
    /// 洞那一版（被施加的记录那一版）的内容。
    pub hole_content: Vec<u8>,
}

pub fn hole_txg(slots_per_region: u64, position: HolePosition) -> u64 {
    if let Ok(value) = std::env::var("HOLE_TXG") {
        return value.parse().expect("HOLE_TXG");
    }
    match position {
        HolePosition::Front => 6,
        HolePosition::Back => 3 * slots_per_region + 4,
    }
}

/// 造前缀。`hole` 由 [`hole_txg`] 给；前缀里全部可写挂载都在补洞开关关着时做（前缀里没有洞，开关不起作用）。
pub fn build_prefix(slots_per_region: u64, hole: u64) -> Prefix {
    let parameters = parameters(slots_per_region);
    let mut devices: Devices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("前缀的可写挂载");
    let instance = output.instance;
    let mut versions = Vec::new();
    let first_content = content(0);
    let mut current: TransactionOutput = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &first_content,
                write_time_seconds: WRITE_TIME,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    versions.push(PublishedVersion {
        instance,
        checkpoint_txg: current.root.checkpoint_txg,
        content: first_content,
    });
    while current.root.checkpoint_txg.0 + 1 < hole {
        let salt = current.root.checkpoint_txg.0 + 1;
        let next_content = content(salt);
        let empty = std::env::var("PREFIX_EMPTY_PUBLISHES").is_ok_and(|v| v == "1");
        let kept_content = versions.last().map(|v: &PublishedVersion| v.content.clone()).expect("有一版");
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = if empty {
            publish_empty(&mut pool, &mut allocator, &current, instance).expect("空发布")
        } else {
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &current,
                FirstFile {
                    content: &next_content,
                    write_time_seconds: WRITE_TIME + salt,
                },
                instance,
            )
            .expect("覆盖写")
        };
        versions.push(PublishedVersion {
            instance,
            checkpoint_txg: current.root.checkpoint_txg,
            content: if empty { kept_content } else { next_content },
        });
    }
    if std::env::var("CRASH_THE_ROW_PUBLISH").is_ok_and(|v| v == "1") {
        // 进程在 h − 1 之后正常退出；下一次可写挂载（新实例）的写行那次发布（txg h）记录已持久、根槽没持久。
        let before = pool_of(&devices);
        let stream = SharedStream::retaining_contents();
        let mut recording = recording_of(&before, &stream);
        let mounted = mount_writable(&parameters, &mut recording).expect("被截的那次挂载");
        assert_eq!(mounted.output.row_publish.root().checkpoint_txg.0, hole, "写行那次的 txg 就是洞");
        let operations = stream.retained_operations();
        let geometry = geometry(&parameters);
        let root_write = operations
            .iter()
            .position(|retained| geometry.classify(&retained.operation) == StepKind::RootRecordFua)
            .expect("写行那次根槽写");
        let mut image = before;
        image.apply(&operations[..root_write]);
        let hole_content = versions.last().expect("有一版").content.clone();
        return Prefix { slots_per_region, hole, image, versions, hole_content };
    }
    // txg h 那次覆盖写：录下它的写，切在根槽写之前。
    let before = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let hole_content = content(hole);
    {
        let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
        let crashed = publish_overwrite(
            &mut pool,
            &mut allocator.clone(),
            &current,
            FirstFile {
                content: &hole_content,
                write_time_seconds: WRITE_TIME + hole,
            },
            instance,
        )
        .expect("洞那一版");
        assert_eq!(crashed.root.checkpoint_txg.0, hole);
    }
    let operations = stream.retained_operations();
    let geometry = geometry(&parameters);
    let root_write = operations
        .iter()
        .position(|retained| geometry.classify(&retained.operation) == StepKind::RootRecordFua)
        .expect("一次根槽写");
    let mut image = before;
    image.apply(&operations[..root_write]);
    versions.push(PublishedVersion {
        instance,
        checkpoint_txg: CheckpointTxg(hole),
        content: hole_content.clone(),
    });
    Prefix {
        slots_per_region,
        hole,
        image,
        versions,
        hole_content,
    }
}

/// 盘上读出来的门槛与多扣（盘 0）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiskMetrics {
    pub newest_txg: u64,
    pub newest_instance: u32,
    pub oldest_valid: u64,
    pub effective_floor: u64,
    pub floor: u64,
    /// [环里最旧有效根, 最新根] 里环中没有读得出的根的 txg。
    pub missing_txgs: Vec<u64>,
    /// 门槛扣住（已释放、释放代 > 门槛）的槽数。
    pub withheld_by_floor: u64,
    /// 其中没有任何候选根（有效 ∧ txg ≥ F_生效）引用的槽数：多扣。
    pub over_withheld: u64,
    pub candidate_txgs: Vec<u64>,
    pub abandoned_txgs: Vec<u64>,
    /// 对照臂「一圈窗口」：有效根只认 txg > 最新根 − 3S 的，门槛 = max(F_生效, 窗口里最旧有效根)。
    pub window_floor: u64,
    /// 「一圈窗口」门槛下仍扣住的槽数。
    pub withheld_by_window_floor: u64,
    /// 窗口之外（比最新根旧一整圈以上）的有效根。
    pub valid_roots_outside_the_window: Vec<u64>,
}

fn slots_referenced_by(reader: &dyn PoolReader, root: &RootRecord) -> Option<BTreeSet<u64>> {
    let records = allocation_records_under_root(reader, root).ok()?;
    if records.is_empty() {
        let mut slots = BTreeSet::new();
        for (locations, span) in [
            (&root.instance_table.locations, INSTANCE_TABLE_SPAN_SLOTS),
            (
                &root.tree_table.locations,
                TransactionUnit::TreeTable.span_slots(),
            ),
        ] {
            for location in locations.iter().filter(|l| l.device == DeviceIdentity(0)) {
                slots.extend(location.slot.0..location.slot.0 + span);
            }
        }
        return Some(slots);
    }
    Some(
        records
            .iter()
            .filter(|r| r.device == DeviceIdentity(0) && !r.is_released)
            .flat_map(|r| r.slot.0..r.slot.0 + u64::from(r.span_slots))
            .collect(),
    )
}

pub fn disk_metrics(image: &MemoryPool) -> Option<DiskMetrics> {
    let configuration = choose_system_configuration(image).ok()?;
    let immutable = &configuration.immutable;
    let roots = readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let newest = roots
        .iter()
        .map(|(_, root)| *root)
        .max_by_key(|root| (root.checkpoint_txg, root.instance))?;
    let table = instance_table_of_root(image, &newest)?;
    let effective_floor = effective_rollback_floor(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let (valid, abandoned): (Vec<RootRecord>, Vec<RootRecord>) = roots
        .iter()
        .map(|(_, root)| *root)
        .partition(|root| !root_is_abandoned_by_the_instance_table(root, &table));
    let oldest_valid = valid.iter().map(|r| r.checkpoint_txg.0).min()?;
    let floor = oldest_valid.max(effective_floor.0);
    let all_txgs: BTreeSet<u64> = roots.iter().map(|(_, r)| r.checkpoint_txg.0).collect();
    let missing_txgs: Vec<u64> = (oldest_valid..=newest.checkpoint_txg.0)
        .filter(|txg| !all_txgs.contains(txg))
        .collect();
    let records = allocation_records_under_root(image, &newest).ok()?;
    let withheld: BTreeSet<u64> = records
        .iter()
        .filter(|r| r.device == DeviceIdentity(0) && r.is_released && r.generation.0 > floor)
        .flat_map(|r| r.slot.0..r.slot.0 + u64::from(r.span_slots))
        .collect();
    let candidates: Vec<&RootRecord> = valid
        .iter()
        .filter(|r| r.checkpoint_txg >= effective_floor || **r == newest)
        .collect();
    let mut referenced = BTreeSet::new();
    for root in &candidates {
        referenced.extend(slots_referenced_by(image, root)?);
    }
    let over = withheld.difference(&referenced).count();
    let ring_length = 3 * immutable.sizes.root_ring_slots_per_region.count();
    let inside = |txg: u64| txg + ring_length > newest.checkpoint_txg.0;
    let window_oldest = valid.iter().map(|r| r.checkpoint_txg.0).filter(|txg| inside(*txg)).min()?;
    let window_floor = window_oldest.max(effective_floor.0);
    let withheld_by_window_floor = records
        .iter()
        .filter(|r| r.device == DeviceIdentity(0) && r.is_released && r.generation.0 > window_floor)
        .map(|r| u64::from(r.span_slots))
        .sum();
    let mut valid_roots_outside_the_window: Vec<u64> =
        valid.iter().map(|r| r.checkpoint_txg.0).filter(|txg| !inside(*txg)).collect();
    valid_roots_outside_the_window.sort_unstable();
    let mut candidate_txgs: Vec<u64> = candidates.iter().map(|r| r.checkpoint_txg.0).collect();
    candidate_txgs.sort_unstable();
    let mut abandoned_txgs: Vec<u64> = abandoned.iter().map(|r| r.checkpoint_txg.0).collect();
    abandoned_txgs.sort_unstable();
    Some(DiskMetrics {
        newest_txg: newest.checkpoint_txg.0,
        newest_instance: newest.instance.0,
        oldest_valid,
        effective_floor: effective_floor.0,
        floor,
        missing_txgs,
        withheld_by_floor: u64::try_from(withheld.len()).expect("小"),
        over_withheld: u64::try_from(over).expect("小"),
        candidate_txgs,
        abandoned_txgs,
        window_floor,
        withheld_by_window_floor,
        valid_roots_outside_the_window,
    })
}

/// slot(h) 上现在住着哪条根（txg），读不出为 None。
pub fn occupant_of_the_hole_slot(image: &MemoryPool, hole: u64) -> Option<u64> {
    let configuration = choose_system_configuration(image).ok()?;
    let immutable = &configuration.immutable;
    let target = target_for_publish(CheckpointTxg(hole), immutable.sizes.root_ring_slots_per_region);
    readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|(slot, _)| *slot == target)
    .map(|(_, root)| root.checkpoint_txg.0)
}

pub fn violated(image: &MemoryPool) -> Vec<&'static str> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| {
            if let singlefs_checker::image::InvariantVerdict::Violated(detail) = &verdict {
                if std::env::var("SHOW_VIOLATION_DETAIL").is_ok_and(|v| v == "1") {
                    eprintln!("VIOLATION {name}: {detail}");
                }
            }
            matches!(verdict, singlefs_checker::image::InvariantVerdict::Violated(_)).then_some(name)
        })
        .collect()
}

pub fn instance_of(generation: u32) -> InstanceGeneration {
    InstanceGeneration(generation)
}

/// 环里读得出的、txg 为 `txg` 的那条根。
pub fn readable_root_at_txg(image: &MemoryPool, txg: u64) -> Option<RootRecord> {
    let configuration = choose_system_configuration(image).ok()?;
    let immutable = &configuration.immutable;
    readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|(_, root)| root)
    .find(|root| root.checkpoint_txg.0 == txg)
}

/// 带文件的一版上的空发布（与 E156 装置的 `publish_empty` 同一个做法：不改文件，只换根与记账）。
pub fn publish_empty<Device: singlefs_core::block_device::BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut singlefs_core::allocator::PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, singlefs_core::transaction::PublishError> {
    use singlefs_core::transaction::{publish_version, InstanceTablePlan, PublishPlan};
    let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: singlefs_core::journal::back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 树表 0 条的池上的后洞：mkfs 之后反复可写挂载（每次写行一次、暖机零到两次），直到某次挂载的写行那一版 txg ≥ `at_least`，
/// 那一次挂载录流、切在写行那次根槽写之前（记录已持久、根槽没持久）。洞 = 那次写行的 txg。
pub fn build_prefix_without_file(slots_per_region: u64, at_least: u64) -> Prefix {
    let parameters = parameters(slots_per_region);
    let mut devices: Devices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    loop {
        let newest = disk_metrics(&pool_of(&devices)).map_or(0, |m| m.newest_txg);
        if newest + 1 >= at_least {
            let before = pool_of(&devices);
            let stream = SharedStream::retaining_contents();
            let mut recording = recording_of(&before, &stream);
            let mounted = mount_writable(&parameters, &mut recording).expect("被截的那次挂载");
            let hole = mounted.output.row_publish.root().checkpoint_txg.0;
            let operations = stream.retained_operations();
            let geometry = geometry(&parameters);
            let root_write = operations
                .iter()
                .position(|retained| geometry.classify(&retained.operation) == StepKind::RootRecordFua)
                .expect("写行那次根槽写");
            let warm_crash = std::env::var("CRASH_THE_WARM_UP").is_ok_and(|v| v == "1");
            if warm_crash && mounted.output.warm_up_publishes.is_empty() {
                // 这次挂载没有暖机：让它做完，下一轮再找。
                devices = crate::devices_of(&{
                    let mut image = before;
                    image.apply(&operations);
                    image
                });
                continue;
            }
            let cut = if warm_crash {
                // 同一个实例的第一次暖机：记录已持久、根槽没持久（与写行那次的实例相同，下一次挂载会施加它）。
                operations
                    .iter()
                    .enumerate()
                    .filter(|(_, retained)| geometry.classify(&retained.operation) == StepKind::RootRecordFua)
                    .nth(1)
                    .map(|(index, _)| index)
                    .expect("暖机那次根槽写")
            } else {
                root_write
            };
            let hole = if warm_crash { mounted.output.warm_up_publishes[0].root().checkpoint_txg.0 } else { hole };
            let mut image = before;
            image.apply(&operations[..cut]);
            return Prefix {
                slots_per_region,
                hole,
                image,
                versions: Vec::new(),
                hole_content: Vec::new(),
            };
        }
        mount_writable(&parameters, &mut devices).expect("前缀里的可写挂载");
    }
}

/// 在 `image` 上做一次可写挂载，切在写行那次根槽写之前（写行那次的记录已持久、根槽没持久）。返回 (崩溃后的盘面, 写行那次的 txg)。
pub fn crash_a_mount_before_its_row_root(slots_per_region: u64, image: &MemoryPool) -> (MemoryPool, u64) {
    let parameters = parameters(slots_per_region);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("被截的那次挂载");
    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
    let operations = stream.retained_operations();
    let (writes, _) = singlefs_harness::memory_pool::writes_and_segments(&operations, &geometry(&parameters));
    let cut = writes
        .iter()
        .position(|write| {
            write.kind == StepKind::RootRecordFua
                && write
                    .bytes()
                    .and_then(|b| RootRecord::parse_slot(b, &FILESYSTEM_IDENTIFIER))
                    .is_some_and(|root| root.checkpoint_txg.0 == row_txg)
        })
        .expect("写行那次根槽写");
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..cut]);
    (crashed, row_txg)
}
