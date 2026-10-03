//! m3-sweep-and-tree-id-r1 攻方腿原型的共用件：内存稀疏盘上造合法历史、录流、调池级 checker、按单元头判「某实例的未发布单元」。
//! 只在攻方副本里有；清扫器是这里按条款拼的原型（今天的 `crates/` 里没有清扫器）。
#![allow(clippy::missing_panics_doc, clippy::must_use_candidate, clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"M3SWEEP-R1-OPUS1";
pub const WRITE_TIME: u64 = 1_788_000_000;

pub fn image_bytes() -> u64 {
    std::env::var("PROTO_IMAGE_BYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(4 << 30)
}

pub fn parameters(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
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

pub fn content(salt: u64, length: usize) -> Vec<u8> {
    let salt = usize::try_from(salt % 251).expect("小");
    (0..length).map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256")).collect()
}

pub type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn fresh_devices() -> Devices {
    (0..2u32)
        .map(|n| (DeviceIdentity(n), SparseBlockDevice::new(image_bytes(), PhysicalBlockSizeInBytes(512))))
        .collect()
}

pub fn pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: image_bytes(),
    }
}

pub fn devices_of(pool: &MemoryPool) -> Devices {
    pool.devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(image_bytes(), PhysicalBlockSizeInBytes(512));
            device.image = image.clone();
            (*identity, device)
        })
        .collect()
}

pub fn recording_of(pool: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_of(pool)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect()
}

/// 池级 checker 判违例的那几条（名字加细节前 160 字符）。
pub fn violated(pool: &MemoryPool) -> Vec<String> {
    singlefs_checker::walk::check_pool_image(pool)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(detail) => {
                Some(format!("{name}:{}", detail.chars().take(160).collect::<String>()))
            }
            _ => None,
        })
        .collect()
}

/// 一个槽开头的单元头（只认头校验和过的三类）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitHeader {
    pub class: u8,
    pub instance: u32,
    pub birth: u64,
    /// 码 1 / 码 3 写序里的事务号低 48 位；码 2 没有（0）。
    pub transaction: u64,
    pub fsid_low: u64,
    /// 码 2 的树 ID；别的类 0。
    pub tree: u64,
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"))
}
fn u48_at(bytes: &[u8], offset: usize) -> u64 {
    let mut b = [0u8; 8];
    b[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(b)
}

/// 头校验和：CRC32C 罩 [0, 头末)，校验和字段 10..42 置零后算，存在 10..14、其余 28 字节为零（D18 已定项 17）。
pub fn header_checksum_holds(bytes: &[u8], header_end: usize) -> bool {
    if bytes.len() < header_end || header_end < 42 {
        return false;
    }
    let mut covered = bytes[..header_end].to_vec();
    covered[10..42].fill(0);
    let expected = singlefs_checker::crc32_castagnoli_bitwise(&covered).to_le_bytes();
    bytes[10..14] == expected && bytes[14..42].iter().all(|b| *b == 0)
}

pub fn reseal_header(bytes: &mut [u8], header_end: usize) {
    bytes[10..42].fill(0);
    let crc = singlefs_checker::crc32_castagnoli_bitwise(&bytes[..header_end]).to_le_bytes();
    bytes[10..14].copy_from_slice(&crc);
}

pub fn header_end_of(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 52 || &bytes[..4] != b"SFSU" {
        return None;
    }
    match bytes[6] {
        1 => Some(105),
        2 => Some(86 + 2 * usize::from(bytes[51])),
        3 => Some(107),
        _ => None,
    }
}

pub fn parse_unit_header(bytes: &[u8]) -> Option<UnitHeader> {
    let header_end = header_end_of(bytes)?;
    if !header_checksum_holds(bytes, header_end) {
        return None;
    }
    Some(match bytes[6] {
        1 => UnitHeader { class: 1, birth: u64_at(bytes, 75), fsid_low: u64_at(bytes, 83), instance: u32_at(bytes, 91), transaction: u48_at(bytes, 95), tree: 0 },
        2 => {
            let k = 2 * usize::from(bytes[51]);
            UnitHeader { class: 2, tree: u64_at(bytes, 42), birth: u64_at(bytes, 52 + k), fsid_low: u64_at(bytes, 60 + k), instance: u32_at(bytes, 68 + k), transaction: 0 }
        }
        3 => UnitHeader { class: 3, birth: u64_at(bytes, 73), fsid_low: u64_at(bytes, 81), instance: u32_at(bytes, 93), transaction: u48_at(bytes, 97), tree: 0 },
        _ => return None,
    })
}

pub fn fsid_low() -> u64 {
    u64::from_le_bytes(FILESYSTEM_IDENTIFIER[..8].try_into().expect("8"))
}

/// 一块盘上单元区里写过的槽（稀疏盘里有扇区的槽）；没写过的槽全零，按「已抹头」同待遇。
pub fn written_unit_slots(pool: &MemoryPool, device: DeviceIdentity) -> Vec<u64> {
    let image = &pool.devices[&device];
    let sectors_per_slot = SLOT_BYTES / 512;
    let slots: BTreeSet<u64> = image
        .written_sectors()
        .keys()
        .map(|sector| sector / sectors_per_slot)
        .filter(|slot| *slot >= UNIT_AREA_START_SLOT)
        .collect();
    slots.into_iter().collect()
}

pub fn read_slot_head(pool: &MemoryPool, device: DeviceIdentity, slot: u64, length: usize) -> Vec<u8> {
    pool.devices[&device].read(singlefs_core::address::DeviceOffsetInBytes(slot * SLOT_BYTES), length)
}

/// 全池单元头：(盘, 槽) → 头。
pub fn every_unit_header(pool: &MemoryPool) -> BTreeMap<(u32, u64), UnitHeader> {
    let mut out = BTreeMap::new();
    for device in pool.devices.keys() {
        for slot in written_unit_slots(pool, *device) {
            if let Some(h) = parse_unit_header(&read_slot_head(pool, *device, slot, 4096)) {
                out.insert((device.0, slot), h);
            }
        }
    }
    out
}

/// 已发布谓词（D18 已定项 11）里只管有行的那一支：有行 (i, T, W) 时，码 1 ⇒ b ≤ T ∨ n ≤ W，码 2 / 3 ⇒ b ≤ T。
pub fn unpublished_by_row(header: &UnitHeader, row_txg: u64, row_w: u64) -> bool {
    match header.class {
        1 => !(header.birth <= row_txg || header.transaction <= row_w),
        _ => header.birth > row_txg,
    }
}

pub fn instance(i: u32) -> InstanceGeneration {
    InstanceGeneration(i)
}

use singlefs_core::address::{CheckpointTxg, DeviceOffsetInBytes};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::journal::back_chain_of;
use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root, readable_roots_with_ring_slots};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_version, InstanceTablePlan, InstanceTableRewrite, PoolWriter, PublishError, PublishPlan, TransactionOutput};

/// 根环全部可读根（自证过的），与环的槽数是否全都自证过。
pub fn ring_roots(pool: &MemoryPool) -> (Vec<RootRecord>, bool) {
    let c = choose_system_configuration(pool).expect("系统配置");
    let roots: Vec<RootRecord> = readable_roots_with_ring_slots(pool, &c.immutable.region_devices, &c.immutable.sizes, &c.immutable.filesystem_identifier)
        .into_iter()
        .map(|(_, r)| r)
        .collect();
    let slots = 3 * c.immutable.sizes.root_ring_slots_per_region.count();
    let every_slot = u64::try_from(roots.len()).expect("小") == slots;
    (roots, every_slot)
}

pub fn rows_of(pool: &MemoryPool, root: &RootRecord) -> Vec<InstanceRow> {
    instance_table_chain_of_root(pool, root).expect("实例表").records.rows
}

/// 清扫判出来的东西：每个候选行（环里没有它的根）在池里还剩的未发布单元（盘, 槽）。
#[derive(Clone, Debug, Default)]
pub struct SweepJudgement {
    pub every_ring_slot_self_verified: bool,
    pub candidate_rows: Vec<InstanceRow>,
    pub unpublished_by_instance: BTreeMap<u32, Vec<(u32, u64)>>,
    pub kept_rows: Vec<InstanceRow>,
}

/// 按条款拼的整轮清扫判定（零轮原型）：候选行 = 根环每槽自证过时、环里没有该实例任何一条根的行；
/// 池中每一个写过的槽读头，头校验和过、fsid 是本池、写序实例是候选、按行判未发布 ⇒ 记下。
/// `skip_allocated` 给出时跳过当前账里已分配的槽（Q3 的一种读法）。
pub fn judge_sweep(pool: &MemoryPool, current_root: &RootRecord, skip_allocated: Option<&BTreeSet<(u32, u64)>>) -> SweepJudgement {
    let (roots, every_slot) = ring_roots(pool);
    let rows = rows_of(pool, current_root);
    let instances_with_roots: BTreeSet<u32> = roots.iter().map(|r| r.instance.0).collect();
    let candidate_rows: Vec<InstanceRow> = if every_slot {
        rows.iter().copied().filter(|r| !instances_with_roots.contains(&r.instance.0)).collect()
    } else {
        Vec::new()
    };
    let mut unpublished_by_instance: BTreeMap<u32, Vec<(u32, u64)>> = BTreeMap::new();
    for ((device, slot), header) in every_unit_header(pool) {
        if header.fsid_low != fsid_low() {
            continue;
        }
        if skip_allocated.is_some_and(|set| set.contains(&(device, slot))) {
            continue;
        }
        if let Some(row) = candidate_rows.iter().find(|r| r.instance.0 == header.instance) {
            if unpublished_by_row(&header, row.selected_root_txg.0, row.applied_transaction_high_water) {
                unpublished_by_instance.entry(header.instance).or_default().push((device, slot));
            }
        }
    }
    let kept_rows = rows.iter().copied().filter(|r| !candidate_rows.iter().any(|c| c.instance == r.instance)).collect();
    SweepJudgement { every_ring_slot_self_verified: every_slot, candidate_rows, unpublished_by_instance, kept_rows }
}

/// 抹头：每个落点写 512 字节零（单元头全在第一扇区里，D18 已定项 18 的偏移表），FUA；之后每块盘一道屏障。
pub fn erase_headers<Device: BlockDevice>(devices: &mut [(DeviceIdentity, Device)], targets: &[(u32, u64)]) {
    for (device, slot) in targets {
        let (_, d) = devices.iter_mut().find(|(id, _)| id.0 == *device).expect("盘");
        d.write_at(DeviceOffsetInBytes(slot * SLOT_BYTES), &[0u8; 512], WriteDurability::ForceUnitAccess).expect("抹头");
    }
    for (_, d) in devices.iter_mut() {
        d.barrier().expect("屏障");
    }
}

/// 专用清扫发布（零轮原型）：同一实例、事务号 0、接在现行那一版之后，实例表整条链重写成 `rows`（与写行那次发布同一个计划形态）。
pub fn sweep_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    current: &TransactionOutput,
    rows: Vec<InstanceRow>,
    replaced_chain: Vec<singlefs_core::pointer::NodePointer>,
    rewrite: bool,
) -> Result<TransactionOutput, PublishError> {
    let mut pool = PoolWriter::new(parameters, devices);
    let instance_table = if rewrite {
        InstanceTablePlan::Rewrite(InstanceTableRewrite { rows, replaced_chain })
    } else {
        InstanceTablePlan::Carry(current.root.instance_table)
    };
    publish_version(
        &mut pool,
        allocator,
        PublishPlan {
            txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
            counter: current.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: current.highest_transaction_number_in_this_instance,
            instance: current.root.instance,
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table,
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        },
        Some(current),
    )
}

/// 当前账里已分配的（盘, 槽）。
pub fn allocated_slots(current: &TransactionOutput) -> BTreeSet<(u32, u64)> {
    current
        .allocation_records
        .iter()
        .filter(|r| !r.is_released)
        .flat_map(|r| (r.slot.0..r.slot.0 + u64::from(r.span_slots)).map(move |s| (r.device.0, s)))
        .collect()
}

/// 一个崩溃状态落成一份池。
pub fn materialize(base: &MemoryPool, writes: &[singlefs_harness::memory_pool::RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    let landed: Vec<_> = writes.iter().zip(persisted).filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect();
    pool.apply_writes(&landed);
    pool
}

pub fn short_err(e: &impl std::fmt::Debug) -> String {
    format!("{e:?}").chars().take(220).collect()
}

use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::transaction::{publish_first_file, publish_overwrite, publish_sequential_write, FirstFile};
use singlefs_harness::segments::StepKind;

pub fn overwrite_with(parameters: &MakeFilesystemParameters, devices: &mut Devices, allocator: &mut PoolAllocator, current: &TransactionOutput, bytes: &[u8], salt: u64, inst: u32) -> TransactionOutput {
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    publish_overwrite(&mut pool, allocator, current, FirstFile { content: bytes, write_time_seconds: WRITE_TIME + salt }, instance(inst)).expect("覆盖写")
}

/// 「死实例」前缀：mkfs → 实例 1 第一个文件、覆盖写 2 次 → 顺序写 N 个数据单元切在第一条记录写之前（孤儿）
/// → 可写挂载实例 2 → 实例 2 覆盖写到根环里没有实例 1 的根。
pub struct DeadInstancePrefix {
    pub parameters: MakeFilesystemParameters,
    pub devices: Devices,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
    pub i1: u32,
    pub i2: u32,
    pub orphan_unit_writes: usize,
    pub rows_after_mount2: Vec<(u32, u64, u64)>,
    pub unpublished_after_mount2: usize,
    pub overwrites: u64,
    pub last_content: Vec<u8>,
}

pub fn dead_instance_prefix(s: u64, n: usize) -> DeadInstancePrefix {
    let parameters = parameters(s);
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted { output, mut allocator, current } = mount_writable(&parameters, &mut devices).expect("挂载 1");
    let i1 = output.instance.0;
    let mut cur = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(&mut pool, &mut allocator, current.root(), FirstFile { content: &content(0, 3000), write_time_seconds: WRITE_TIME }, instance(i1), current.record_bytes()).expect("第一个文件")
    };
    for step in 1..=2 {
        cur = overwrite_with(&parameters, &mut devices, &mut allocator, &cur, &content(step, 3000), step, i1);
    }
    let before = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    if n > 0 {
        let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
        let _ = publish_sequential_write(&mut pool, &mut allocator.clone(), &cur, FirstFile { content: &content(9, n * 32_634), write_time_seconds: WRITE_TIME + 9 }, instance(i1)).expect("顺序写");
    }
    let ops = stream.retained_operations();
    let g = geometry(&parameters);
    let cut = ops.iter().position(|o| g.classify(&o.operation) == StepKind::JournalRecord).unwrap_or(ops.len());
    let mut image = before;
    image.apply(&ops[..cut]);
    let orphan_unit_writes = ops[..cut].iter().filter(|o| g.classify(&o.operation) == StepKind::UnitWrite).count();
    let mut devices = devices_of(&image);
    let m2 = mount_writable(&parameters, &mut devices).expect("挂载 2");
    let i2 = m2.output.instance.0;
    let mut allocator = m2.allocator.clone();
    let mut current = m2.current.clone().into_file_version().expect("带文件");
    let rows_after_mount2: Vec<(u32, u64, u64)> = m2.output.rows_written.iter().map(|r| (r.instance.0, r.selected_root_txg.0, r.applied_transaction_high_water)).collect();
    let (t1, w1) = (rows_after_mount2[0].1, rows_after_mount2[0].2);
    let unpublished_after_mount2 = every_unit_header(&pool_of(&devices)).values().filter(|h| h.instance == i1 && h.fsid_low == fsid_low() && unpublished_by_row(h, t1, w1)).count();
    let mut overwrites = 0u64;
    let mut last_content = content(0, 3000);
    loop {
        let (roots, _) = ring_roots(&pool_of(&devices));
        if !roots.iter().any(|r| r.instance.0 == i1) {
            break;
        }
        overwrites += 1;
        last_content = content(100 + overwrites, 3000);
        current = overwrite_with(&parameters, &mut devices, &mut allocator, &current, &last_content, 100 + overwrites, i2);
    }
    DeadInstancePrefix { parameters, devices, allocator, current, i1, i2, orphan_unit_writes, rows_after_mount2, unpublished_after_mount2, overwrites, last_content }
}

/// 认领规则扫描（D18 已定项 6 射程里那一条，E88 的口径）：每块盘按槽升序，解出一个头就按它的跨度把后面的槽认领掉、不再当探针。
/// 跨度：码 1 / 码 3 两槽，码 2 一槽。交回不被认领的、头校验和过的 (盘, 槽) → 头。
pub fn claim_rule_scan(pool: &MemoryPool) -> BTreeMap<(u32, u64), UnitHeader> {
    let mut out = BTreeMap::new();
    for device in pool.devices.keys() {
        let mut claimed_until = 0u64;
        for slot in written_unit_slots(pool, *device) {
            if slot < claimed_until {
                continue;
            }
            if let Some(h) = parse_unit_header(&read_slot_head(pool, *device, slot, 4096)) {
                claimed_until = slot + if h.class == 2 { 1 } else { 2 };
                out.insert((device.0, slot), h);
            }
        }
    }
    out
}

/// 根环里每一条可读根的账里仍分配的 (盘, 槽)（候选集与被抛弃根一起）。
pub fn slots_referenced_by_any_ring_root(pool: &MemoryPool) -> BTreeSet<(u32, u64)> {
    let (roots, _) = ring_roots(pool);
    let mut out = BTreeSet::new();
    for root in roots {
        if let Ok(records) = singlefs_core::recovery::allocation_records_under_root(pool, &root) {
            for r in records.iter().filter(|r| !r.is_released) {
                for s in r.slot.0..r.slot.0 + u64::from(r.span_slots) {
                    out.insert((r.device.0, s));
                }
            }
        }
    }
    out
}

/// 照一个真码 1 头造一个伪头：写序实例、事务号、诞生代号改掉，头校验和重算。
pub fn forged_data_unit_header(template: &[u8], inst: u32, birth: u64, transaction: u64) -> Vec<u8> {
    let mut fake = template[..105].to_vec();
    fake[75..83].copy_from_slice(&birth.to_le_bytes());
    fake[91..95].copy_from_slice(&inst.to_le_bytes());
    fake[95..101].copy_from_slice(&transaction.to_le_bytes()[..6]);
    reseal_header(&mut fake, 105);
    fake
}

/// 一条根指着的树表条目：(树 ID, 诞生 txg)。树表单元两份里第一份解得开的那份。
pub fn tree_table_of_root(pool: &MemoryPool, root: &RootRecord) -> Option<Vec<(u64, u64)>> {
    for location in &root.tree_table.locations {
        let bytes = singlefs_core::recovery::PoolReader::read(pool, location.device, DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES), 16384)?;
        if let Ok(node) = singlefs_core::unit::parse_index_node(&bytes) {
            return Some(node.entries.iter().filter_map(|e| singlefs_core::records::TreeTableEntry::parse(e)).map(|e| (e.tree.0, e.birth_txg.0)).collect());
        }
    }
    None
}

/// S2 的几条候选判法，在一份镜像上各判一次（零轮原型，读盘用的是 core 的读者——只为在副本上量，判法本身要与运行时分开写，D13 已定项 5）。
#[derive(Debug, Default)]
pub struct TreeIdJudgements {
    /// T1-全环：根环里任两条可读根的树表里，同一树 ID 的诞生 txg 不同。
    pub t1_all_roots: Vec<String>,
    /// T1-同线：只比按最新根的实例表判有效的根（I-9.14 的射程）。
    pub t1_same_line: Vec<String>,
    /// T2：某条可读根 r 的水位 ≤ 某个码 2 节点的树 ID，而那个节点的诞生代号 ≤ r 的 txg（节点在 r 之前就有了、r 的水位却没罩住它）。
    /// 后缀：noexcl 不排除孤儿；excl 照 I-7.8 两道排除（诞生代号 ≤ 环里最大 txg、行判未发布的不数）；claim 再加认领规则。
    pub t2_noexcl: Vec<String>,
    pub t2_excl: Vec<String>,
    pub t2_claim_excl: Vec<String>,
    /// T3：可读根按 txg 排，水位不许下降。
    pub t3_monotone: Vec<String>,
}

pub fn judge_tree_identifiers(pool: &MemoryPool) -> TreeIdJudgements {
    let mut out = TreeIdJudgements::default();
    let (roots, _) = ring_roots(pool);
    let Some(newest) = roots.iter().copied().max_by_key(|r| (r.checkpoint_txg, r.instance)) else { return out; };
    let rows = rows_of(pool, &newest);
    let valid = |r: &RootRecord| !singlefs_core::recovery::root_is_abandoned_by_the_instance_table(r, &singlefs_core::instance_table::InstanceTableRecords { rows: rows.clone() });
    let tables: Vec<(RootRecord, Vec<(u64, u64)>)> = roots.iter().filter_map(|r| tree_table_of_root(pool, r).map(|t| (*r, t))).collect();
    for (i, (ra, ta)) in tables.iter().enumerate() {
        for (rb, tb) in &tables[i + 1..] {
            for (tree, birth_a) in ta {
                if let Some((_, birth_b)) = tb.iter().find(|(t, _)| t == tree) {
                    if birth_a != birth_b {
                        let line = format!("tree {tree}: ({},{}) birth {birth_a} vs ({},{}) birth {birth_b}", ra.instance.0, ra.checkpoint_txg.0, rb.instance.0, rb.checkpoint_txg.0);
                        if valid(ra) && valid(rb) { out.t1_same_line.push(line.clone()); }
                        out.t1_all_roots.push(line);
                    }
                }
            }
        }
    }
    let newest_txg = roots.iter().map(|r| r.checkpoint_txg.0).max().unwrap_or(0);
    let excluded = |h: &UnitHeader| h.birth > newest_txg || rows.iter().any(|row| row.instance.0 == h.instance && row.selected_root_txg.0 > 0 && h.birth > row.selected_root_txg.0);
    let judge_nodes = |nodes: &BTreeMap<(u32, u64), UnitHeader>, exclude: bool, sink: &mut Vec<String>| {
        for ((d, s), h) in nodes.iter().filter(|(_, h)| h.class == 2) {
            if exclude && excluded(h) { continue; }
            for r in &roots {
                if h.birth <= r.checkpoint_txg.0 && r.tree_identifier_watermark <= h.tree {
                    sink.push(format!("node (dev {d}, slot {s}) tree {} birth {} inst {} vs root ({},{}) wm {}", h.tree, h.birth, h.instance, r.instance.0, r.checkpoint_txg.0, r.tree_identifier_watermark));
                }
            }
        }
    };
    let all = every_unit_header(pool);
    judge_nodes(&all, false, &mut out.t2_noexcl);
    judge_nodes(&all, true, &mut out.t2_excl);
    judge_nodes(&claim_rule_scan(pool), true, &mut out.t2_claim_excl);
    let mut by_txg = roots.clone();
    by_txg.sort_by_key(|r| (r.checkpoint_txg, r.instance));
    for w in by_txg.windows(2) {
        if w[1].tree_identifier_watermark < w[0].tree_identifier_watermark {
            out.t3_monotone.push(format!("({},{}) wm {} > ({},{}) wm {}", w[0].instance.0, w[0].checkpoint_txg.0, w[0].tree_identifier_watermark, w[1].instance.0, w[1].checkpoint_txg.0, w[1].tree_identifier_watermark));
        }
    }
    out
}

pub fn summarize(j: &TreeIdJudgements) -> String {
    let f = |v: &Vec<String>| if v.is_empty() { "0".to_string() } else { format!("{}[{}]", v.len(), v[0]) };
    format!("T1_all={} T1_same_line={} T2_noexcl={} T2_excl={} T2_claim_excl={} T3_monotone={}", f(&j.t1_all_roots), f(&j.t1_same_line), f(&j.t2_noexcl), f(&j.t2_excl), f(&j.t2_claim_excl), f(&j.t3_monotone))
}
