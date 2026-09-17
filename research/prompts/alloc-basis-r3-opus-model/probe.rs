//! alloc-basis-r3 Opus 攻方腿的探针（副本装置）。只在草稿目录的 crates 副本里编译、跑，不进仓；
//! 按 `run-probe.sh` 拷到副本的 `crates/singlefs-harness/src/bin/alloc_basis_r3_probe.rs`。
//! 只调 crates 的公开函数（加上 patch-copy.py 改成 pub 的两个 mount.rs 函数与三个 allocator 访问器）；读数都是副本上的数。
//! 环境变量（探针这一侧）：SFPROBE_T1=1 分配器在每次探针驱动的发布之前按「此刻盘上的环与 F_生效」回收（记账那一半由补丁做）；
//! SFPROBE_G5_TURNOVER=1 同一时刻按此刻的候选集重算影子账（本腿提的收严，被攻过零轮）；SFPROBE_CONSERVATIVE=1 回退之后把被抛弃根账里未释放的槽全部隔离（保守读法）。
#![allow(clippy::all)]
use std::collections::{BTreeMap, BTreeSet};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse, UnitFootprint};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{
    abandoned_by_table, isolate_slots_referenced_only_by_abandoned_roots, mount_rollback, mount_writable, reclaim_floor,
    rollback_floor_ceiling, InstanceTableRecords, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_superblock, effective_rollback_floor, instance_table_of_root, readable_roots,
    scan_journal,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::superblock::FormatTimeGeometry;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_version, warm_up, FileVersionPlan, FirstFile, InstanceTablePlan, PoolWriter,
    PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::crash::{MemoryPool, SparseDevice};

const IMAGE_BYTES: u64 = 4 << 30;
const FSID: [u8; 16] = [0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00];
const WRITE_TIME: u64 = 1_788_000_000;

#[derive(Clone)]
struct MemDevice {
    image: SparseDevice,
    /// 落在这些偏移上的写报 I/O 错、不落盘（根槽写失败：槽里留旧内容）。
    failing_offsets: BTreeSet<u64>,
    /// 落在这些偏移上的写静默丢掉（造崩溃态：单元写了、根没写）。
    dropped_offsets: BTreeSet<u64>,
}
impl BlockDevice for MemDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        buffer.copy_from_slice(&self.image.read(offset, buffer.len()));
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], _d: WriteDurability) -> Result<(), BlockDeviceError> {
        if self.failing_offsets.contains(&offset.0) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("probe: injected root slot write failure")));
        }
        if !self.dropped_offsets.contains(&offset.0) {
            self.image.write(offset, bytes);
        }
        Ok(())
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(512)
    }
    fn size_in_bytes(&self) -> u64 {
        IMAGE_BYTES
    }
}

type Devices = Vec<(DeviceIdentity, MemDevice)>;

#[derive(Clone)]
struct Pool {
    devices: Devices,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
    /// 探针发过的每一版（记账第 1 项等从这里取）。
    history: BTreeMap<(u64, u32), TransactionOutput>,
    /// 最近一次回退的目标与那一刻被抛弃的最新根（第九项原式用）。
    last_rollback: Option<((u32, u64), (u32, u64))>,
    seed: usize,
}

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FSID,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: FormatTimeGeometry {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    }
}

fn env_on(name: &str) -> bool {
    std::env::var(name).is_ok()
}

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length).map(|i| u8::try_from((i * 7 + seed) % 253).unwrap()).collect()
}

fn remember(pool: &mut Pool, output: &TransactionOutput) {
    pool.history.insert((output.root.checkpoint_txg.0, output.root.instance.0), output.clone());
}

fn build() -> Pool {
    let params = parameters();
    let mut devices: Devices = (0..2u32)
        .map(|d| (DeviceIdentity(d), MemDevice { image: SparseDevice::default(), failing_offsets: BTreeSet::new(), dropped_offsets: BTreeSet::new() }))
        .collect();
    let genesis = make_filesystem(&params, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES), DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES)]);
    allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
    let content = content_of(3000, 0);
    let (instance, output) = {
        let mut pool = PoolWriter::new(&params, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let output = publish_first_file(&mut pool, &mut allocator, &genesis.root, FirstFile { content: &content, write_time_seconds: WRITE_TIME }, instance, &warm.last_record_bytes)
            .expect("第一个事务");
        (instance, output)
    };
    let mut pool = Pool { devices, allocator, current: output.clone(), instance, history: BTreeMap::new(), last_rollback: None, seed: 100 };
    remember(&mut pool, &output);
    pool
}

fn roots_of(devices: &Devices) -> Vec<RootRecord> {
    let sb = choose_superblock(devices).expect("超级块");
    readable_roots(devices, &sb.region_devices, &sb.geometry, &sb.filesystem_identifier)
}

fn effective_floor(devices: &Devices) -> CheckpointTxg {
    let sb = choose_superblock(devices).expect("超级块");
    effective_rollback_floor(devices, &sb.region_devices, &sb.geometry, &sb.filesystem_identifier)
}

fn newest_table(devices: &Devices) -> Option<InstanceTableRecords> {
    let sb = choose_superblock(devices).ok()?;
    let newest = choose_root(devices, &sb)?;
    instance_table_of_root(devices, &newest)
}

fn is_abandoned(root: &RootRecord, table: &Option<InstanceTableRecords>) -> bool {
    table.as_ref().is_some_and(|t| abandoned_by_table(root, t))
}

/// 可再分配谓词的门槛（D16 已定项 1）：max(F_生效, 环里最旧有效根)，全部从盘上现算。
fn floor_now(devices: &Devices) -> CheckpointTxg {
    let table = newest_table(devices);
    let oldest = roots_of(devices).into_iter().filter(|r| !is_abandoned(r, &table)).map(|r| r.checkpoint_txg).min();
    reclaim_floor(effective_floor(devices), oldest)
}

/// 一条根引用的槽（盘 0；它那一版分配记录里仍分配的，逐槽展开）。读不出给空集。
fn referenced_slots(devices: &Devices, root: &RootRecord) -> BTreeSet<u64> {
    let mut out = BTreeSet::new();
    if let Ok(records) = allocation_records_under_root(devices, root) {
        for r in records {
            if r.device == DeviceIdentity(0) && !r.is_released {
                for s in r.slot.0..r.slot.0 + u64::from(r.span_slots) {
                    out.insert(s);
                }
            }
        }
    }
    out
}

fn image(devices: &Devices) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(id, d)| (*id, d.image.clone())).collect(), device_size_in_bytes: IMAGE_BYTES }
}

/// 红的不变量与第一处细节；全绿返回 GREEN。
fn reds(devices: &Devices) -> String {
    let mut out = Vec::new();
    for (name, verdict) in check_pool_image(&image(devices)) {
        if let InvariantVerdict::Violated(detail) = verdict {
            out.push(format!("{name}[{detail}]"));
        }
    }
    if out.is_empty() { "GREEN".to_string() } else { out.join(" ; ") }
}

fn stat(output: &TransactionOutput, statistic: u16) -> u64 {
    output.accounting_entries.iter().find(|e| e.statistic == statistic && e.device == DeviceIdentity(0)).map(|e| e.value / 16384).unwrap_or(u64::MAX)
}

fn compact(values: &BTreeSet<u64>) -> String {
    let mut parts = Vec::new();
    let mut iter = values.iter().copied().peekable();
    while let Some(start) = iter.next() {
        let mut end = start;
        while iter.peek() == Some(&(end + 1)) {
            end = iter.next().unwrap();
        }
        parts.push(if start == end { format!("{start}") } else { format!("{start}-{end}") });
    }
    format!("[{}]", parts.join(","))
}

fn written_slots(output: &TransactionOutput) -> BTreeSet<u64> {
    output.rewritten.iter().flat_map(|role| {
        let slot = output.unit(*role).slot.0;
        slot..slot + role.span_slots()
    }).collect()
}

/// 按此刻的候选集（按最新根实例表有效 ∧ txg ≥ floor）重算影子账：调 mount.rs 里那个函数本身（补丁只改了可见性）。
fn recompute_g5(pool: &mut Pool, floor: CheckpointTxg) -> u64 {
    let roots = roots_of(&pool.devices);
    let table = newest_table(&pool.devices);
    let current_records = pool.current.allocation_records.clone();
    isolate_slots_referenced_only_by_abandoned_roots(&pool.devices, &mut pool.allocator, &roots, &|r| is_abandoned(r, &table), floor, &current_records)
}

/// 保守读法：被抛弃根账里每个未释放的槽都隔离，含候选根也引用的。
fn isolate_conservatively(pool: &mut Pool) {
    let table = newest_table(&pool.devices);
    for root in roots_of(&pool.devices).into_iter().filter(|r| is_abandoned(r, &table)) {
        if let Ok(records) = allocation_records_under_root(&pool.devices, &root) {
            for record in records.into_iter().filter(|r| !r.is_released) {
                pool.allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));
            }
        }
    }
}

/// 探针驱动的每一次发布之前：甲-T1 的分配器那一半（按此刻的门槛回收），可选地按此刻的候选集重算影子账。
fn before_publish(pool: &mut Pool) {
    if env_on("SFPROBE_T1") {
        let floor = floor_now(&pool.devices);
        pool.allocator.reclaim_released_up_to(floor, ReclaimedReuse::Immediately);
    }
    if env_on("SFPROBE_G5_TURNOVER") {
        let floor = effective_floor(&pool.devices);
        recompute_g5(pool, floor);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Overwrite,
    Empty(u64),
}

/// 发布之前的环：每条根（txg, 实例）→ (被抛弃?, 候选?, 引用的槽)。
struct RingSnapshot {
    roots: Vec<(RootRecord, bool, bool, BTreeSet<u64>)>,
}

fn snapshot(devices: &Devices) -> RingSnapshot {
    let table = newest_table(devices);
    let floor = effective_floor(devices);
    let roots = roots_of(devices)
        .into_iter()
        .map(|r| {
            let abandoned = is_abandoned(&r, &table);
            let candidate = !abandoned && r.checkpoint_txg >= floor;
            let refs = referenced_slots(devices, &r);
            (r, abandoned, candidate, refs)
        })
        .collect();
    RingSnapshot { roots }
}

/// 这次发布写出的槽落在发布之前环里哪些根引用的槽上：(根 txg, 这条根是不是正被这次发布的根盖掉, 撞到的槽)。
fn collisions(before: &RingSnapshot, output: &TransactionOutput, want_abandoned: bool) -> String {
    let written = written_slots(output);
    let overwritten_target = target_for_publish(output.root.checkpoint_txg);
    let mut hits = Vec::new();
    for (root, abandoned, candidate, refs) in &before.roots {
        let wanted = if want_abandoned { *abandoned } else { *candidate };
        if !wanted {
            continue;
        }
        let common: BTreeSet<u64> = refs.intersection(&written).copied().collect();
        if common.is_empty() {
            continue;
        }
        let in_same_slot = target_for_publish(root.checkpoint_txg) == overwritten_target;
        hits.push(format!("({},{}){}{}", root.instance.0, root.checkpoint_txg.0, if in_same_slot { "window" } else { "durable" }, compact(&common)));
    }
    if hits.is_empty() { "-".to_string() } else { hits.join("") }
}

fn root_slot_location(txg: u64) -> (usize, u64) {
    let target = target_for_publish(CheckpointTxg(txg));
    let device = parameters().region_devices[target.region as usize].0 as usize;
    (device, singlefs_core::root_ring::slot_offset(target, 4096).0)
}

/// 一次发布（覆盖写或空发布）：txg 在 `failing` 里的那次根槽写报错，按 D23 已定项 14「推进一格再发」换 txg + 1（jsn + 1）重发同一个事务。
/// 返回写成的那一版与撞槽报告。
fn step(pool: &mut Pool, kind: Kind, failing: &BTreeSet<u64>) -> (TransactionOutput, String, String, Vec<u64>) {
    before_publish(pool);
    let before = snapshot(&pool.devices);
    let params = parameters();
    let previous = pool.current.clone();
    let mut txg = previous.root.checkpoint_txg.0 + 1;
    let mut counter = previous.record.counter + 1;
    let transaction = match kind {
        Kind::Overwrite => previous.record.transaction + 1,
        Kind::Empty(_) => 0,
    };
    let content = content_of(3000 + pool.seed % 97, pool.seed);
    pool.seed += 1;
    let mut failed = Vec::new();
    loop {
        let (device, offset) = root_slot_location(txg);
        let inject = failing.contains(&txg);
        if inject {
            pool.devices[device].1.failing_offsets.insert(offset);
        }
        let plan = PublishPlan {
            txg: CheckpointTxg(txg),
            counter,
            transaction,
            instance: pool.instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: match kind {
                Kind::Overwrite => Some(FileVersionPlan { content: &content, write_time_seconds: WRITE_TIME + 60, inode_object_birth: previous.inode_record.object_birth, change_count: txg }),
                Kind::Empty(_) => None,
            },
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: match kind {
                Kind::Overwrite => previous.root.rollback_floor,
                Kind::Empty(floor) => CheckpointTxg(floor),
            },
        };
        let result = {
            let mut writer = PoolWriter::new(&params, pool.devices.as_mut_slice());
            publish_version(&mut writer, &mut pool.allocator, plan, Some(&previous))
        };
        if inject {
            pool.devices[device].1.failing_offsets.remove(&offset);
        }
        match result {
            Ok(output) => {
                let ab = collisions(&before, &output, true);
                let cand = collisions(&before, &output, false);
                pool.current = output.clone();
                remember(pool, &output);
                return (output, ab, cand, failed);
            }
            Err(PublishError::BlockDevice(_)) if inject => {
                failed.push(txg);
                txg += 1;
                counter += 1;
            }
            Err(error) => panic!("发布失败：{error:?}"),
        }
    }
}

fn line(tag: &str, pool: &Pool, extra: &str) -> String {
    let d0 = &pool.allocator.devices[0];
    format!(
        "{tag} txg={} inst={} F_root={} F_eff={} floor_now={} acct_alloc={} acct_free={} acct_defer={} mem_alloc={} mem_defer={} mem_free={} iso={} held={} {extra} | {}",
        pool.current.root.checkpoint_txg.0,
        pool.current.root.instance.0,
        pool.current.root.rollback_floor.0,
        effective_floor(&pool.devices).0,
        floor_now(&pool.devices).0,
        stat(&pool.current, 1),
        stat(&pool.current, 2),
        stat(&pool.current, 5),
        d0.allocated_slots(),
        d0.deferred_slots(),
        d0.free_slots(),
        d0.isolated_slots(),
        d0.held_slot_numbers().len(),
        reds(&pool.devices)
    )
}

fn publish_and_report(pool: &mut Pool, tag: &str, kind: Kind, failing: &BTreeSet<u64>) -> TransactionOutput {
    let (output, ab, cand, failed) = step(pool, kind, failing);
    let failed_text = if failed.is_empty() { String::new() } else { format!("root_write_failed_at={failed:?} ") };
    let extra = format!("{failed_text}wrote={} hit_abandoned={ab} hit_candidate={cand}", compact(&written_slots(&output)));
    println!("{}", line(tag, pool, &extra));
    output
}

fn remount(pool: &mut Pool) {
    let mounted = mount_writable(&parameters(), &mut pool.devices).expect("可写挂载");
    remember(pool, &mounted.output.row_publish);
    for warm in &mounted.output.warm_up_publishes {
        remember(pool, warm);
    }
    pool.allocator = mounted.allocator;
    pool.current = mounted.current;
    pool.instance = mounted.output.instance;
}

fn rollback(pool: &mut Pool, instance: u32, txg: u64) -> Result<(), String> {
    let sb = choose_superblock(&pool.devices).expect("超级块");
    let newest = choose_root(&pool.devices, &sb).expect("最新根");
    let mounted = mount_rollback(
        &parameters(),
        &mut pool.devices,
        RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) },
        ShadowLedger::On,
    )
    .map_err(|e| format!("{e:?}"))?;
    remember(pool, &mounted.output.row_publish);
    for warm in &mounted.output.warm_up_publishes {
        remember(pool, warm);
    }
    pool.allocator = mounted.allocator;
    pool.current = mounted.current;
    pool.instance = mounted.output.instance;
    pool.last_rollback = Some(((instance, txg), (newest.instance.0, newest.checkpoint_txg.0)));
    if env_on("SFPROBE_CONSERVATIVE") {
        isolate_conservatively(pool);
    }
    Ok(())
}

/// 第九项与影子账这一格（Z4′）：此刻环里的被抛弃根 / 候选根 / 低于 F 的有效根各引用什么，两种公式各算多少，影子账隔离了什么。
fn ninth_item_line(tag: &str, pool: &Pool) -> String {
    let table = newest_table(&pool.devices);
    let floor = effective_floor(&pool.devices);
    let roots = roots_of(&pool.devices);
    let mut abandoned_txgs = BTreeSet::new();
    let mut candidate_txgs = BTreeSet::new();
    let mut below_floor_txgs = BTreeSet::new();
    let mut union_abandoned = BTreeSet::new();
    let mut union_candidates = BTreeSet::new();
    let mut union_valid = BTreeSet::new();
    for r in &roots {
        let refs = referenced_slots(&pool.devices, r);
        if is_abandoned(r, &table) {
            abandoned_txgs.insert(r.checkpoint_txg.0);
            union_abandoned.extend(refs);
        } else {
            union_valid.extend(refs.iter().copied());
            if r.checkpoint_txg >= floor {
                candidate_txgs.insert(r.checkpoint_txg.0);
                union_candidates.extend(refs);
            } else {
                below_floor_txgs.insert(r.checkpoint_txg.0);
            }
        }
    }
    let current: BTreeSet<u64> = pool
        .current
        .allocation_records
        .iter()
        .filter(|r| r.device == DeviceIdentity(0) && !r.is_released)
        .flat_map(|r| r.slot.0..r.slot.0 + u64::from(r.span_slots))
        .collect();
    let set_reading: BTreeSet<u64> = union_abandoned.difference(&union_candidates).copied().collect();
    let g5_definition: BTreeSet<u64> = set_reading.difference(&current).copied().collect();
    let narrow_by_valid: BTreeSet<u64> = union_abandoned.difference(&union_valid).copied().filter(|s| !current.contains(s)).collect();
    let count_reading = union_abandoned.len().saturating_sub(union_candidates.len());
    let isolated: BTreeSet<u64> = pool.allocator.devices[0].isolated_slot_numbers().into_iter().collect();
    let stale: BTreeSet<u64> = isolated.difference(&union_abandoned).copied().collect();
    let missing: BTreeSet<u64> = g5_definition.difference(&isolated).copied().collect();
    let original = match pool.last_rollback {
        Some(((target_instance, target_txg), (abandoned_instance, abandoned_txg))) => {
            let target_stat = pool.history.get(&(target_txg, target_instance)).map(|o| stat(o, 1) as i64);
            let abandoned_stat = pool.history.get(&(abandoned_txg, abandoned_instance)).map(|o| stat(o, 1) as i64);
            let cleared = !roots.iter().any(|r| r.checkpoint_txg.0 == abandoned_txg && r.instance.0 == abandoned_instance);
            match (abandoned_stat, target_stat) {
                (Some(a), Some(t)) => format!("orig(({abandoned_instance},{abandoned_txg})−({target_instance},{target_txg}))={}{}", a - t, if cleared { "→cleared0" } else { "" }),
                _ => "orig=?".to_string(),
            }
        }
        None => "orig=-".to_string(),
    };
    let lowest = pool.allocator.devices[0].lowest_user_data_slot(pool.allocator.open_segment()).map_or(0, |s| s.0);
    format!(
        "{tag} txg={} F_eff={} candidates={} abandoned={} below_F={} | {} set_reading={} g5_def={} narrow_by_valid={} count_reading={} G5_isolated={}{} stale_isolated={} missing_isolation={} lowest_user_slot={}",
        pool.current.root.checkpoint_txg.0,
        floor.0,
        compact(&candidate_txgs),
        compact(&abandoned_txgs),
        compact(&below_floor_txgs),
        original,
        set_reading.len(),
        g5_definition.len(),
        narrow_by_valid.len(),
        count_reading,
        isolated.len(),
        compact(&isolated),
        compact(&stale),
        compact(&missing),
        lowest
    )
}

fn none() -> BTreeSet<u64> {
    BTreeSet::new()
}

fn device_of_txg(txg: u64) -> u32 {
    parameters().region_devices[target_for_publish(CheckpointTxg(txg)).region as usize].0
}

/// 覆盖写到 current txg == last，每次一行，外加第九项那一行（`with_ninth`）。`crash_window_at`：那一次发布的根槽写静默丢掉、在副本上报崩溃态之后不再往下走。
fn overwrite_until(pool: &mut Pool, tag: &str, last: u64, with_ninth: bool, crash_window_at: Option<u64>) {
    while pool.current.root.checkpoint_txg.0 < last {
        let next = pool.current.root.checkpoint_txg.0 + 1;
        if crash_window_at == Some(next) {
            let mut crashed = pool.clone();
            let (device, offset) = root_slot_location(next);
            crashed.devices[device].1.dropped_offsets.insert(offset);
            let (output, ab, cand, _) = step(&mut crashed, Kind::Overwrite, &none());
            crashed.devices[device].1.dropped_offsets.remove(&offset);
            println!(
                "{tag} CRASH-WINDOW txg={} (units + record written, root not) wrote={} hit_abandoned={ab} hit_candidate={cand} | disk-state {}",
                next,
                compact(&written_slots(&output)),
                reds(&crashed.devices)
            );
            println!("{}", ninth_item_line(&format!("{tag} CRASH-WINDOW ninth"), &crashed));
        }
        if pool.allocator.records().len() + 16 * 2 > 813 {
            println!("{tag} stop: 分配记录树一个节点装不下下一次覆盖写");
            break;
        }
        publish_and_report(pool, &format!("{tag} overwrite"), Kind::Overwrite, &none());
        if with_ninth {
            println!("{}", ninth_item_line(&format!("{tag} ninth"), pool));
        }
    }
}

/// H1：回退之后接着转环。A(3)，同一实例再覆盖写 `extra` 次，回退到 (1,3)，之后覆盖写到 last。
fn scenario_rollback_turnover(extra: usize, last: u64, crash_window_at: Option<u64>) {
    let mut pool = build();
    for _ in 0..extra {
        publish_and_report(&mut pool, "H1 before-rollback", Kind::Overwrite, &none());
    }
    rollback(&mut pool, 1, 3).expect("回退到 (1,3)");
    println!("{}", line("H1 after-rollback(row+warm)", &pool, ""));
    println!("{}", ninth_item_line("H1 after-rollback ninth", &pool));
    overwrite_until(&mut pool, "H1", last, true, crash_window_at);
}

/// H1m：里程碑脚本的形状（A、B、重开、C、回退到 (1,3)），之后覆盖写到 last。
fn scenario_milestone_turnover(last: u64, crash_window_at: Option<u64>) {
    let mut pool = build();
    publish_and_report(&mut pool, "H1m B", Kind::Overwrite, &none());
    remount(&mut pool);
    println!("{}", line("H1m remount(row+warm)", &pool, ""));
    publish_and_report(&mut pool, "H1m C", Kind::Overwrite, &none());
    rollback(&mut pool, 1, 3).expect("回退到 (1,3)");
    println!("{}", line("H1m after-rollback(row+warm)", &pool, ""));
    println!("{}", ninth_item_line("H1m after-rollback ninth", &pool));
    overwrite_until(&mut pool, "H1m", last, true, crash_window_at);
}

/// H4：两次回退。A(3)、覆盖写 4–7，回退到 (1,5)（行 8、暖机 9–10），覆盖写 11–12，第二次回退到 `second`，之后覆盖写到 last。
fn scenario_two_rollbacks(second_instance: u32, second_txg: u64, last: u64) {
    let tag = format!("H4({second_instance},{second_txg})");
    let mut pool = build();
    for _ in 0..4 {
        publish_and_report(&mut pool, &format!("{tag} first-timeline"), Kind::Overwrite, &none());
    }
    rollback(&mut pool, 1, 5).expect("第一次回退到 (1,5)");
    println!("{}", line(&format!("{tag} after-rollback-1"), &pool, ""));
    println!("{}", ninth_item_line(&format!("{tag} after-rollback-1 ninth"), &pool));
    for _ in 0..2 {
        publish_and_report(&mut pool, &format!("{tag} second-timeline"), Kind::Overwrite, &none());
        println!("{}", ninth_item_line(&format!("{tag} second-timeline ninth"), &pool));
    }
    match rollback(&mut pool, second_instance, second_txg) {
        Ok(()) => {
            println!("{}", line(&format!("{tag} after-rollback-2"), &pool, ""));
            println!("{}", ninth_item_line(&format!("{tag} after-rollback-2 ninth"), &pool));
        }
        Err(error) => {
            println!("{tag} second rollback refused: {error}");
            return;
        }
    }
    overwrite_until(&mut pool, &tag, last, true, None);
}

fn raise_with_crates(pool: &mut Pool, tag: &str, wanted: u64) {
    let sb = choose_superblock(&pool.devices).expect("超级块");
    let records = scan_journal(&pool.devices, &sb);
    let table = newest_table(&pool.devices).expect("实例表");
    let ceiling = rollback_floor_ceiling(&pool.devices, &sb, &records, pool.current.root.rollback_floor, &table).expect("上限").0;
    let floor = wanted.min(ceiling);
    let raised = singlefs_core::mount::raise_rollback_floor(&parameters(), &mut pool.devices, &mut pool.allocator, &mut pool.current, CheckpointTxg(floor), ShadowLedger::On)
        .expect("抬 F");
    for publish in &raised.publishes {
        remember(pool, publish);
    }
    println!(
        "{tag} raise F wanted={wanted} ceiling={ceiling} used={floor} publishes={:?} reclaimed={}",
        raised.publishes.iter().map(|p| p.root.checkpoint_txg.0).collect::<Vec<_>>(),
        raised.reclaimed.len()
    );
    println!("{}", line(&format!("{tag} after-raise"), pool, ""));
    println!("{}", ninth_item_line(&format!("{tag} after-raise ninth"), pool));
}

/// H4c：回退之前已经抬过 F、回退目标在 F 之上。A(3)、覆盖写 4–9，重开（行 10、暖机 11），覆盖写 12–15，抬 F 到 12（16、17），
/// 覆盖写 18–20，回退到 (2,14)（行 21、暖机 22），覆盖写到 `raise_again_at`，再抬 F 到上限，之后覆盖写到 last。
fn scenario_rollback_after_raise(raise_again_at: u64, last: u64) {
    let tag = "H4c";
    let mut pool = build();
    for _ in 0..6 {
        publish_and_report(&mut pool, &format!("{tag} inst1"), Kind::Overwrite, &none());
    }
    remount(&mut pool);
    println!("{}", line(&format!("{tag} remount(row+warm)"), &pool, ""));
    for _ in 0..4 {
        publish_and_report(&mut pool, &format!("{tag} inst2"), Kind::Overwrite, &none());
    }
    raise_with_crates(&mut pool, tag, 12);
    for _ in 0..3 {
        publish_and_report(&mut pool, &format!("{tag} abandoned-to-be"), Kind::Overwrite, &none());
    }
    rollback(&mut pool, 2, 14).expect("回退到 (2,14)");
    println!("{}", line(&format!("{tag} after-rollback"), &pool, ""));
    println!("{}", ninth_item_line(&format!("{tag} after-rollback ninth"), &pool));
    overwrite_until(&mut pool, tag, raise_again_at, true, None);
    raise_with_crates(&mut pool, tag, u64::MAX);
    overwrite_until(&mut pool, tag, last, true, None);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RaiseArm {
    /// 今天的实现：第一条带新 F 的根之前按新 F 重算影子账、回收；hold = 扣住到生效；release_unconditionally = 照 mount.rs 在循环之后无条件放开。
    FKou { hold: bool, release_unconditionally: bool },
    /// G7：两块盘都带上新 F 的根之后才重算影子账、回收（记账那一半由 SFPROBE_T1 的补丁按持久之后的 F_生效 写）。
    G7,
}

/// 照 mount.rs 的 raise_rollback_floor 一步一步做，能停在第 `stop_after` 次发布之后（崩溃态）、能在根槽写失败时推进一格重发、
/// 能按「成功发布次数 ≤ success_cap」截断循环（mount.rs 今天的上限写法）。返回是否落满两块盘。
fn raise_manual(pool: &mut Pool, tag: &str, new_floor: u64, arm: RaiseArm, stop_after: Option<usize>, failing: &BTreeSet<u64>, success_cap: Option<usize>) -> bool {
    let sb = choose_superblock(&pool.devices).expect("超级块");
    let records = scan_journal(&pool.devices, &sb);
    let table = pool
        .current
        .units
        .iter()
        .find(|u| u.identity == TransactionUnit::InstanceTable)
        .and_then(|u| InstanceTableRecords::parse(&u.bytes))
        .or_else(|| newest_table(&pool.devices))
        .expect("实例表");
    let ceiling = rollback_floor_ceiling(&pool.devices, &sb, &records, pool.current.root.rollback_floor, &table).expect("上限").0;
    assert!(new_floor <= ceiling, "要抬的 F {new_floor} 超过上限 {ceiling}");
    let mut reclaimed_before = 0;
    if let RaiseArm::FKou { hold, .. } = arm {
        recompute_g5(pool, CheckpointTxg(new_floor));
        let oldest = roots_of(&pool.devices).into_iter().filter(|r| !abandoned_by_table(r, &table)).map(|r| r.checkpoint_txg).min();
        let reuse = if hold { ReclaimedReuse::HeldUntilFloorTakesEffect } else { ReclaimedReuse::Immediately };
        reclaimed_before = pool.allocator.reclaim_released_up_to(reclaim_floor(CheckpointTxg(new_floor), oldest), reuse).len();
    }
    println!("{tag} raise arm={arm:?} F={new_floor} ceiling={ceiling} reclaimed_before_first_new_F_root={reclaimed_before}");
    let mut covered = BTreeSet::new();
    let mut successes = 0usize;
    while covered.len() < 2 && stop_after.map_or(true, |s| successes < s) && success_cap.map_or(true, |c| successes < c) {
        let (output, ab, cand, failed) = step(pool, Kind::Empty(new_floor), failing);
        successes += 1;
        covered.insert(device_of_txg(output.root.checkpoint_txg.0));
        let extra = format!("root_write_failed_at={failed:?} wrote={} hit_abandoned={ab} hit_candidate={cand}", compact(&written_slots(&output)));
        println!("{}", line(&format!("{tag} raise-publish"), pool, &extra));
    }
    let is_covered = covered.len() == 2;
    match arm {
        RaiseArm::FKou { hold: true, release_unconditionally } => {
            if stop_after.is_none() && (is_covered || release_unconditionally) {
                pool.allocator.release_reclaim_holds();
                println!("{tag} released holds covered={is_covered}");
            }
        }
        RaiseArm::FKou { hold: false, .. } => {}
        RaiseArm::G7 => {
            if is_covered {
                recompute_g5(pool, CheckpointTxg(new_floor));
                let table_now = newest_table(&pool.devices);
                let oldest = roots_of(&pool.devices).into_iter().filter(|r| !is_abandoned(r, &table_now)).map(|r| r.checkpoint_txg).min();
                let count = pool.allocator.reclaim_released_up_to(reclaim_floor(CheckpointTxg(new_floor), oldest), ReclaimedReuse::Immediately).len();
                println!("{tag} G7 reclaimed after both devices carry F: {count}");
            } else {
                println!("{tag} G7 not covered: nothing reclaimed");
            }
        }
    }
    is_covered
}

/// Z2′ 的底：A(3)、覆盖写 4、重开（行 5、暖机 6、7）、覆盖写 8–13（alloc-basis 第二轮 2.1 那一格，步 5 的扣住用例同形）。
fn z2_base() -> Pool {
    let mut pool = build();
    step(&mut pool, Kind::Overwrite, &none());
    remount(&mut pool);
    for _ in 0..6 {
        step(&mut pool, Kind::Overwrite, &none());
    }
    pool
}

fn z2_arm(name: &str) -> RaiseArm {
    match name {
        "fkou" => RaiseArm::FKou { hold: true, release_unconditionally: true },
        "fkou-nohold" => RaiseArm::FKou { hold: false, release_unconditionally: true },
        "g7" => RaiseArm::G7,
        other => panic!("未知臂 {other}"),
    }
}

fn scenario_z2_crash_between(arm_name: &str) {
    let arm = z2_arm(arm_name);
    let tag = format!("Z2[{arm_name}]");
    let base = z2_base();
    println!("{}", line(&format!("{tag} before-raise"), &base, ""));
    let mut crash = base.clone();
    raise_manual(&mut crash, &tag, 8, arm, Some(1), &none(), None);
    println!("{}", line(&format!("{tag} crash-after-first-new-F-root"), &crash, ""));
    let mut remounted = crash.clone();
    remount(&mut remounted);
    println!("{}", line(&format!("{tag} crash->remount"), &remounted, ""));
    let mut rolled = crash.clone();
    match rollback(&mut rolled, 1, 3) {
        Ok(()) => println!("{}", line(&format!("{tag} crash->rollback(1,3)"), &rolled, "")),
        Err(error) => println!("{tag} crash->rollback(1,3) failed: {error}"),
    }
    let mut full = base.clone();
    raise_manual(&mut full, &format!("{tag} full"), 8, arm, None, &none(), None);
    println!("{}", line(&format!("{tag} after-F-effective"), &full, ""));
    publish_and_report(&mut full, &format!("{tag} E"), Kind::Overwrite, &none());
}

fn scenario_z2_advance_one(arm_name: &str, capped: bool) {
    let arm = z2_arm(arm_name);
    let tag = format!("Z2adv[{arm_name},cap={capped}]");
    let mut pool = z2_base();
    println!("{}", line(&format!("{tag} before-raise"), &pool, ""));
    let failing: BTreeSet<u64> = [16].into_iter().collect();
    let covered = raise_manual(&mut pool, &tag, 8, arm, None, &failing, if capped { Some(3) } else { None });
    println!("{tag} loop ended covered={covered}");
    for _ in 0..3 {
        publish_and_report(&mut pool, &format!("{tag} overwrite"), Kind::Overwrite, &none());
    }
}

/// X8-A 那一格的分配器层：每盘 128 槽两段，段 0 全部仍分配、段 1 全部已释放（释放代 5）；抬 F 到 8。
fn scenario_x8a() {
    let bytes = (50176 + 128) * 16384;
    let mut records = Vec::new();
    for device in 0..2u32 {
        for slot in 50176..50240u64 {
            records.push(AllocationRecord { device: DeviceIdentity(device), slot: SlotNumber(slot), span_slots: 1, generation: CheckpointTxg(1), is_released: false });
        }
        for slot in 50240..50304u64 {
            records.push(AllocationRecord { device: DeviceIdentity(device), slot: SlotNumber(slot), span_slots: 1, generation: CheckpointTxg(5), is_released: true });
        }
    }
    let fresh = || PoolAllocator::rebuild_from_records(vec![DeviceFreeMap::new(DeviceIdentity(0), bytes), DeviceFreeMap::new(DeviceIdentity(1), bytes)], records.clone());
    let mut fkou = fresh();
    let reclaimed = fkou.reclaim_released_up_to(CheckpointTxg(8), ReclaimedReuse::HeldUntilFloorTakesEffect).len();
    let first = fkou.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(9)).map(|p| p.slot.0);
    let again = fkou.reclaim_released_up_to(CheckpointTxg(8), ReclaimedReuse::HeldUntilFloorTakesEffect).len();
    let retry = fkou.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(9)).map(|p| p.slot.0);
    println!("X8A F-扣: reclaimed={reclaimed} first_fixed_point={first:?} retry_reclaimed={again} retry_fixed_point={retry:?} free_slots={} allocated_slots={} deferred={}", fkou.devices[0].free_slots(), fkou.devices[0].allocated_slots(), fkou.devices[0].deferred_slots());
    let mut g7 = fresh();
    let first = g7.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(9)).map(|p| p.slot.0);
    println!("X8A G7: first_fixed_point={first:?} free_slots={} allocated_slots={} deferred={}", g7.devices[0].free_slots(), g7.devices[0].allocated_slots(), g7.devices[0].deferred_slots());
    let mut alpha = fresh();
    alpha.reclaim_released_up_to(CheckpointTxg(8), ReclaimedReuse::HeldUntilFloorTakesEffect);
    let first = alpha.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(9)).map(|p| p.slot.0);
    alpha.release_reclaim_holds();
    let after_release = alpha.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(9)).map(|p| p.slot.0);
    println!("X8A F-扣+α(失败路径放开扣住): first={first:?} after_release={after_release:?}（发出去的是生效之前回收的槽）");
}

/// 甲-b′ 的读数：环里有效根（与候选根）引用的并集，与今天记账行的第 1 / 2 / 5 项、单元区容量并排。
fn bprime_line(tag: &str, pool: &Pool) -> String {
    let table = newest_table(&pool.devices);
    let floor = effective_floor(&pool.devices);
    let mut union_valid = BTreeSet::new();
    let mut union_candidates = BTreeSet::new();
    let mut ring = BTreeSet::new();
    for r in roots_of(&pool.devices) {
        ring.insert(r.checkpoint_txg.0);
        if is_abandoned(&r, &table) {
            continue;
        }
        let refs = referenced_slots(&pool.devices, &r);
        if r.checkpoint_txg >= floor {
            union_candidates.extend(refs.iter().copied());
        }
        union_valid.extend(refs);
    }
    let capacity = pool.allocator.devices[0].unit_area_slots();
    let (s1, s2, s5) = (stat(&pool.current, 1), stat(&pool.current, 2), stat(&pool.current, 5));
    let mem_free = pool.allocator.devices[0].free_slots();
    format!(
        "{tag} txg={} ring_txgs={} acct1={s1} acct2={s2} acct5={s5} capacity={capacity} | T1: acct1+acct2−cap={} | b′: precise_candidates={} precise_valid={} acct1−precise={} ; b′-I52(precise+acct2−cap)={} ; b′-I52(precise+mem_free−cap)={}",
        pool.current.root.checkpoint_txg.0,
        compact(&ring),
        (s1 + s2) as i64 - capacity as i64,
        union_candidates.len(),
        union_valid.len(),
        s1 as i64 - union_candidates.len() as i64,
        (union_candidates.len() as u64 + s2) as i64 - capacity as i64,
        (union_candidates.len() as u64 + mem_free) as i64 - capacity as i64,
    )
}

/// H5：单实例覆盖写，txg `fail_txg` 那次的根槽写报错、推进一格重发；current txg == remount_at 时重开一次；覆盖写到 last。
fn scenario_hole_advance_one(fail_txg: u64, remount_at: u64, last: u64) {
    let mut pool = build();
    println!("{}", bprime_line("H5 A", &pool));
    let failing: BTreeSet<u64> = [fail_txg].into_iter().collect();
    let mut remounted = false;
    while pool.current.root.checkpoint_txg.0 < last {
        if !remounted && pool.current.root.checkpoint_txg.0 >= remount_at {
            remounted = true;
            remount(&mut pool);
            println!("{}", line("H5 remount(row+warm)", &pool, ""));
            println!("{}", bprime_line("H5 remount(row+warm) b′", &pool));
            continue;
        }
        if pool.allocator.records().len() + 16 * 2 > 813 {
            println!("H5 stop: 分配记录树一个节点装不下下一次覆盖写");
            break;
        }
        publish_and_report(&mut pool, "H5 overwrite", Kind::Overwrite, &failing);
        println!("{}", bprime_line("H5 b′", &pool));
    }
}

/// H6：Z5′ 的坏镜像与对照：单实例覆盖写到 last（坏不坏由 SFPROBE_BUG_RELEASE_CARRIED_INSTANCE_TABLE_AT 决定）。
fn scenario_z5(last: u64) {
    let mut pool = build();
    println!("{}", line("H6 A", &pool, ""));
    overwrite_until(&mut pool, "H6", last, false, None);
    let m1: Vec<String> = pool.allocator.records().iter().filter(|r| r.slot == INSTANCE_TABLE_SLOT).map(|r| format!("dev{} gen={} released={}", r.device.0, r.generation.0, r.is_released)).collect();
    println!("H6 final record at 50176: {m1:?} ; newest root instance table slot={}", pool.current.root.instance_table.locations[0].slot.0);
}

/// H7：一个单元被多个根共享——覆盖写与空发布交替（空发布照抄四个文件单元），中途一次 G7 形态的抬 F（两块盘都带上新 F 之后才回收），
/// 之后接着交替到 last。每一次发布报记账、checker 与撞槽。
fn scenario_shared_units(pattern_overwrites: usize, pattern_empties: usize, raise_at: u64, last: u64) {
    let mut pool = build();
    remount(&mut pool);
    println!("{}", line("H7 remount(row+warm)", &pool, ""));
    let mut raised = false;
    while pool.current.root.checkpoint_txg.0 < last {
        if !raised && pool.current.root.checkpoint_txg.0 >= raise_at {
            raised = true;
            let sb = choose_superblock(&pool.devices).expect("超级块");
            let records = scan_journal(&pool.devices, &sb);
            let table = pool.current.units.iter().find(|u| u.identity == TransactionUnit::InstanceTable).and_then(|u| InstanceTableRecords::parse(&u.bytes)).expect("实例表");
            let ceiling = rollback_floor_ceiling(&pool.devices, &sb, &records, pool.current.root.rollback_floor, &table).expect("上限").0;
            raise_manual(&mut pool, "H7", ceiling, RaiseArm::G7, None, &none(), None);
            continue;
        }
        for _ in 0..pattern_overwrites {
            if pool.allocator.records().len() + 16 * 2 > 813 { println!("H7 stop: 分配记录树装不下"); println!("PROBE-COMPLETE"); std::process::exit(0); }
            publish_and_report(&mut pool, "H7 overwrite", Kind::Overwrite, &none());
        }
        for _ in 0..pattern_empties {
            let floor = pool.current.root.rollback_floor.0;
            publish_and_report(&mut pool, "H7 empty", Kind::Empty(floor), &none());
        }
    }
}

/// H1 的后果：回退之后转环到 stop，把回退实例（当前实例）发布的每一条根槽抹成零（D23 已定项 14 的 ⚠️「再让回退实例的根全读不出就挂上被抛弃的根」），
/// 再可写挂载：挂上的是被抛弃时间线的根，读它引用的单元。
fn scenario_rollback_turnover_fallback(extra: usize, stop: u64) {
    let mut pool = build();
    for _ in 0..extra {
        step(&mut pool, Kind::Overwrite, &none());
    }
    rollback(&mut pool, 1, 3).expect("回退到 (1,3)");
    while pool.current.root.checkpoint_txg.0 < stop {
        before_publish_report(&mut pool);
    }
    println!("{}", line("H1F at-stop", &pool, ""));
    let rollback_instance = pool.instance.0;
    let wiped: Vec<u64> = roots_of(&pool.devices).into_iter().filter(|r| r.instance.0 == rollback_instance).map(|r| r.checkpoint_txg.0).collect();
    for txg in &wiped {
        let (device, offset) = root_slot_location(*txg);
        pool.devices[device].1.image.write(DeviceOffsetInBytes(offset), &[0u8; 512]);
    }
    let sb = choose_superblock(&pool.devices).expect("超级块");
    let fallback = choose_root(&pool.devices, &sb).expect("还有根");
    println!("H1F wiped root slots of instance {rollback_instance}: txgs={wiped:?} ; recovery now chooses ({},{}) ; checker on the wiped image: {}", fallback.instance.0, fallback.checkpoint_txg.0, reds(&pool.devices));
    match mount_writable(&parameters(), &mut pool.devices) {
        Ok(mounted) => println!("H1F mount on the fallback root ok: instance={} txg={}", mounted.output.instance.0, mounted.current.root.checkpoint_txg.0),
        Err(error) => println!("H1F mount on the fallback root failed: {error:?}"),
    }
}

fn before_publish_report(pool: &mut Pool) {
    step(pool, Kind::Overwrite, &none());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let number = |index: usize| -> u64 { args[index].parse().expect("数") };
    let optional = |index: usize| -> Option<u64> { args.get(index).and_then(|a| a.parse().ok()).filter(|v| *v > 0) };
    match args.get(1).map(String::as_str) {
        Some("rollback-turnover") => scenario_rollback_turnover(number(2) as usize, number(3), optional(4)),
        Some("milestone-turnover") => scenario_milestone_turnover(number(2), optional(3)),
        Some("two-rollbacks") => scenario_two_rollbacks(number(2) as u32, number(3), number(4)),
        Some("rollback-after-raise") => scenario_rollback_after_raise(number(2), number(3)),
        Some("z2-crash-between") => scenario_z2_crash_between(&args[2]),
        Some("z2-advance-one") => scenario_z2_advance_one(&args[2], args[3] == "cap"),
        Some("x8a") => scenario_x8a(),
        Some("hole-advance-one") => scenario_hole_advance_one(number(2), number(3), number(4)),
        Some("z5") => scenario_z5(number(2)),
        Some("rollback-turnover-fallback") => scenario_rollback_turnover_fallback(number(2) as usize, number(3)),
        Some("shared-units") => scenario_shared_units(number(2) as usize, number(3) as usize, number(4), number(5)),
        _ => eprintln!("用法见 run-probe.sh"),
    }
    println!("PROBE-COMPLETE");
}
