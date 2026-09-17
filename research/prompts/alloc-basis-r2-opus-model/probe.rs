//! alloc-basis-r2 Opus 攻方腿的探针（副本装置）。只在草稿目录的 crates 副本里编译、跑，不进仓；
//! 按 `run-probe.sh` 拷到副本的 `crates/singlefs-harness/src/bin/alloc_basis_r2_probe.rs`。
//! 只调 crates 的公开函数；读数都是副本上的数，不是入库装置上的数。
#![allow(clippy::all)]
use std::collections::{BTreeMap, BTreeSet};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, reclaim_floor, rollback_floor_ceiling, InstanceTableRecords,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_superblock, effective_rollback_floor,
    instance_table_of_root, readable_roots, scan_journal,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::superblock::FormatTimeGeometry;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up, FirstFile,
    InstanceTablePlan, PoolWriter, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::crash::{MemoryPool, SparseDevice};

const IMAGE_BYTES: u64 = 4 << 30;
const FSID: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const WRITE_TIME: u64 = 1_788_000_000;

#[derive(Clone)]
struct MemDevice {
    image: SparseDevice,
    /// 落在这些偏移上的写静默丢掉（根槽写没落下：槽里留旧内容，造环上的洞）。
    dropped_offsets: BTreeSet<u64>,
}
impl BlockDevice for MemDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        buffer.copy_from_slice(&self.image.read(offset, buffer.len()));
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], _d: WriteDurability) -> Result<(), BlockDeviceError> {
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

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length).map(|i| u8::try_from((i * 7 + seed) % 253).unwrap()).collect()
}

fn build() -> Pool {
    let params = parameters();
    let mut devices: Devices = (0..2u32)
        .map(|d| (DeviceIdentity(d), MemDevice { image: SparseDevice::default(), dropped_offsets: BTreeSet::new() }))
        .collect();
    let genesis = make_filesystem(&params, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let content = content_of(3000, 0);
    let (instance, output) = {
        let mut pool = PoolWriter::new(&params, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let output = publish_first_file(
            &mut pool,
            &mut allocator,
            &genesis.root,
            FirstFile { content: &content, write_time_seconds: WRITE_TIME },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个事务");
        (instance, output)
    };
    Pool { devices, allocator, current: output, instance }
}

fn overwrite(pool: &mut Pool, seed: usize) {
    let params = parameters();
    let mut writer = PoolWriter::new(&params, pool.devices.as_mut_slice());
    let content = content_of(3000 + seed % 97, seed);
    let previous = pool.current.clone();
    pool.current = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 },
        pool.instance,
    )
    .expect("覆盖写");
}

/// 空发布（与 mount.rs 抬 F 循环里那一次同形：文件角色照抄、实例表照抄、F 取参数）。
fn empty_publish(pool: &mut Pool, floor: CheckpointTxg) -> TransactionOutput {
    let params = parameters();
    let mut writer = PoolWriter::new(&params, pool.devices.as_mut_slice());
    let current = pool.current.clone();
    let next = publish_version(
        &mut writer,
        &mut pool.allocator,
        PublishPlan {
            txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
            counter: current.record.counter + 1,
            transaction: 0,
            instance: current.root.instance,
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            instance_table: InstanceTablePlan::Carry(current.root.instance_table),
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: floor,
        },
        Some(&current),
    )
    .expect("空发布");
    pool.current = next.clone();
    next
}

fn image(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(id, d)| (*id, d.image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 红的不变量与第一处细节；全绿返回空串。
fn reds(devices: &Devices) -> String {
    let verdicts = check_pool_image(&image(devices));
    let mut out = Vec::new();
    for (name, verdict) in verdicts {
        if let InvariantVerdict::Violated(detail) = verdict {
            out.push(format!("{name}[{detail}]"));
        }
    }
    if out.is_empty() { "GREEN".to_string() } else { out.join(" ; ") }
}

fn accounting_of(output: &TransactionOutput, statistic: u16) -> u64 {
    output
        .accounting_entries
        .iter()
        .find(|e| e.statistic == statistic && e.device == DeviceIdentity(0))
        .map(|e| e.value / 16384)
        .unwrap_or(u64::MAX)
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

fn abandoned(root: &RootRecord, table: &Option<InstanceTableRecords>) -> bool {
    table.as_ref().is_some_and(|t| {
        t.rows.iter().any(|r| r.instance == root.instance && root.checkpoint_txg > r.selected_root_txg)
    })
}

/// 一条根引用的槽（盘 0；它那一版分配记录里仍分配的，逐槽展开）。
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

fn written_slots(output: &TransactionOutput) -> Vec<(TransactionUnit, u64, u64)> {
    output
        .rewritten
        .iter()
        .map(|role| (*role, output.unit(*role).slot.0, role.span_slots()))
        .collect()
}

/// 照 mount.rs 的 raise_rollback_floor 一步一步做（F-今：回收在第一条带新 F 的根之前），`stop_after` 次发布之后停下
/// ——停下那一刻的盘就是「崩在第 stop_after 条与下一条带新 F 的根之间」的持久态。返回写出的发布。
fn raise_floor_manual(pool: &mut Pool, new_floor: CheckpointTxg, stop_after: usize, arm_fsheng: bool) -> (CheckpointTxg, Vec<TransactionOutput>, Vec<Placement>) {
    let sb = choose_superblock(&pool.devices).expect("超级块");
    let records = scan_journal(&pool.devices, &sb);
    // mount.rs 从 current 的实例表单元解；单实例进程里那一版从没重写过、`unit()` 会 panic，这里改从盘上最新根读同一片。
    let table = newest_table(&pool.devices).expect("实例表");
    let ceiling = rollback_floor_ceiling(&pool.devices, &sb, &records, pool.current.root.rollback_floor, &table)
        .expect("上限");
    assert!(new_floor <= ceiling, "要抬的 F {new_floor:?} 超过上限 {ceiling:?}");
    let oldest = roots_of(&pool.devices)
        .into_iter()
        .filter(|r| !abandoned(r, &Some(table.clone())))
        .map(|r| r.checkpoint_txg)
        .min();
    let mut reclaimed = Vec::new();
    if !arm_fsheng {
        reclaimed = pool.allocator.reclaim_released_up_to(reclaim_floor(new_floor, oldest));
    }
    let mut covered: BTreeSet<u32> = BTreeSet::new();
    let mut publishes = Vec::new();
    while covered.len() < 2 && publishes.len() < stop_after {
        let next = empty_publish(pool, new_floor);
        let region = target_for_publish(next.root.checkpoint_txg).region;
        covered.insert(parameters().region_devices[region as usize].0);
        publishes.push(next);
    }
    if arm_fsheng && covered.len() == 2 {
        let oldest = roots_of(&pool.devices)
            .into_iter()
            .filter(|r| !abandoned(r, &Some(table.clone())))
            .map(|r| r.checkpoint_txg)
            .min();
        reclaimed = pool.allocator.reclaim_released_up_to(reclaim_floor(new_floor, oldest));
    }
    (ceiling, publishes, reclaimed)
}

fn remount(pool: &mut Pool) {
    let mounted = mount_writable(&parameters(), &mut pool.devices).expect("可写挂载");
    pool.allocator = mounted.allocator;
    pool.current = mounted.current;
    pool.instance = mounted.output.instance;
}

fn rollback(pool: &mut Pool, instance: u32, txg: u64) -> Result<(), String> {
    let mounted = mount_rollback(
        &parameters(),
        &mut pool.devices,
        RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) },
        ShadowLedger::On,
    )
    .map_err(|e| format!("{e:?}"))?;
    pool.allocator = mounted.allocator;
    pool.current = mounted.current;
    pool.instance = mounted.output.instance;
    Ok(())
}

fn state_line(tag: &str, pool: &Pool) -> String {
    let d0 = &pool.allocator.devices[0];
    format!(
        "{tag} txg={} inst={} F_root={} F_eff={} acct_alloc={} acct_free={} acct_defer={} mem_alloc={} mem_defer={} mem_isolated={} | {}",
        pool.current.root.checkpoint_txg.0,
        pool.current.root.instance.0,
        pool.current.root.rollback_floor.0,
        effective_floor(&pool.devices).0,
        accounting_of(&pool.current, 1),
        accounting_of(&pool.current, 2),
        accounting_of(&pool.current, 5),
        d0.allocated_slots(),
        d0.deferred_slots(),
        d0.isolated_slots(),
        reds(&pool.devices)
    )
}

/// S1：同一次挂载里一直覆盖写，根环转过去之后回收有没有发生、I-3.1 红不红；最后重开一次看重建能不能把它拉回来。
fn scenario_turnover(last_txg: u64, remount_at: Option<u64>) {
    let mut pool = build();
    println!("{}", state_line("S1 A", &pool));
    let mut seed = 1;
    while pool.current.root.checkpoint_txg.0 < last_txg {
        if Some(pool.current.root.checkpoint_txg.0) == remount_at {
            remount(&mut pool);
            println!("{}", state_line("S1 remount+warm", &pool));
            continue;
        }
        overwrite(&mut pool, seed);
        seed += 1;
        println!("{}", state_line("S1 overwrite", &pool));
    }
    remount(&mut pool);
    println!("{}", state_line("S1 final-remount+warm", &pool));
}

/// S2：在同一次挂载里 k 次覆盖写、e 次空发布之后抬 F 到 f（F-今，照 mount.rs）；抬 F 的几次发布里有没有哪一次把新单元写在
/// 「txg 在 [旧 F, 新 F) 的有效根」引用的槽上——那几条根在 F 生效之前按恢复的定义仍是回退候选。
fn scenario_window_search(max_k: usize, max_e: usize) {
    let mut hits = 0u64;
    let mut cells = 0u64;
    for e in 0..=max_e {
        let mut base = build();
        let mut seed = 100;
        for k in 1..=max_k {
            overwrite(&mut base, seed);
            seed += 1;
            if base.allocator.records().len() + 16 * 8 > 813 {
                break;
            }
            let mut with_empties = base.clone();
            for _ in 0..e {
                let floor = with_empties.current.root.rollback_floor;
                empty_publish(&mut with_empties, floor);
            }
            let sb = choose_superblock(&with_empties.devices).unwrap();
            let records = scan_journal(&with_empties.devices, &sb);
            let table = newest_table(&with_empties.devices);
            let ceiling = rollback_floor_ceiling(&with_empties.devices, &sb, &records, CheckpointTxg(0), table.as_ref().unwrap()).unwrap();
            let old_floor = effective_floor(&with_empties.devices);
            for f in (old_floor.0 + 1)..=ceiling.0 {
                cells += 1;
                let mut pool = with_empties.clone();
                let before: Vec<RootRecord> = roots_of(&pool.devices)
                    .into_iter()
                    .filter(|r| r.checkpoint_txg.0 >= old_floor.0 && r.checkpoint_txg.0 < f && !abandoned(r, &table))
                    .collect();
                let referenced: BTreeMap<u64, BTreeSet<u64>> = before
                    .iter()
                    .map(|r| (r.checkpoint_txg.0, referenced_slots(&pool.devices, r)))
                    .collect();
                let (_, publishes, reclaimed) = raise_floor_manual(&mut pool, CheckpointTxg(f), 3, false);
                for (index, p) in publishes.iter().enumerate() {
                    let txg = p.root.checkpoint_txg.0;
                    // 这次发布之前还没被盖掉的根（每次发布盖掉 txg − 24 那一条）。
                    let overwritten_before: BTreeSet<u64> = publishes[..index]
                        .iter()
                        .map(|q| q.root.checkpoint_txg.0.saturating_sub(24))
                        .collect();
                    for (role, slot, span) in written_slots(p) {
                        for (root_txg, slots) in &referenced {
                            if overwritten_before.contains(root_txg) {
                                continue;
                            }
                            if (slot..slot + span).any(|s| slots.contains(&s)) {
                                hits += 1;
                                println!(
                                    "S2 HIT e={e} k={k} f={f} ceiling={} publish#{index} txg={txg} last={} role={role:?} slot={slot} span={span} referenced_by_root_txg={root_txg} reclaimed={}",
                                    ceiling.0,
                                    index + 1 == publishes.len(),
                                    reclaimed.len()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    println!("S2 SUMMARY cells={cells} hits={hits}");
}

/// 里程碑固定脚本到 txg 14：A、B、重开（写行 5、暖机 6、7）、C、回退到 (1, 3)（D 9、暖机 10）、覆盖写 11–14。
fn milestone_through_14() -> Pool {
    let mut pool = build();
    overwrite(&mut pool, 3);
    remount(&mut pool);
    overwrite(&mut pool, 11);
    rollback(&mut pool, 1, 3).expect("回退");
    for seed in [17, 19, 23, 29] {
        overwrite(&mut pool, seed);
    }
    pool
}

/// S3：F-今 / F-生 在「第一条带新 F 的根持久、第二条没持久」那一格：崩溃态上 checker 判什么，重开之后判什么，
/// 回退到 [旧 F, 新 F) 里的根走不走得通。
fn scenario_crash_between(arm_fsheng: bool) {
    let arm = if arm_fsheng { "F-生(pre)" } else { "F-今" };
    let mut pool = milestone_through_14();
    println!("{}", state_line(&format!("S3 {arm} before-raise"), &pool));
    let mut crash = pool.clone();
    let (ceiling, first, reclaimed) = raise_floor_manual(&mut crash, CheckpointTxg(11), 1, arm_fsheng);
    println!("S3 {arm} ceiling={} first_publish_txg={} reclaimed_before_it={}", ceiling.0, first[0].root.checkpoint_txg.0, reclaimed.len());
    println!("{}", state_line(&format!("S3 {arm} crash-after-first-F-root"), &crash));
    let mut after_remount = crash.clone();
    remount(&mut after_remount);
    println!("{}", state_line(&format!("S3 {arm} crash->remount"), &after_remount));
    let mut rolled = crash.clone();
    match rollback(&mut rolled, 3, 9) {
        Ok(()) => println!("{}", state_line(&format!("S3 {arm} crash->rollback(3,9)"), &rolled)),
        Err(e) => println!("S3 {arm} crash->rollback(3,9) refused: {e}"),
    }
    let (_, both, reclaimed_both) = raise_floor_manual(&mut pool, CheckpointTxg(11), 3, arm_fsheng);
    println!("S3 {arm} full-raise publishes={:?} reclaimed={}", both.iter().map(|p| p.root.checkpoint_txg.0).collect::<Vec<_>>(), reclaimed_both.len());
    println!("{}", state_line(&format!("S3 {arm} after-F-effective"), &pool));
    overwrite(&mut pool, 31);
    println!("{}", state_line(&format!("S3 {arm} E"), &pool));
    let e_slot = pool.current.data_pointer.locations[0].slot.0;
    println!("S3 {arm} E data slot={e_slot}");
}

/// S4：S2 找到的一格（同一次挂载里 k 次覆盖写、e 次空发布、抬 F 到 f），停在第 stop 次发布之后（崩溃态），看它的后果。
fn scenario_window_hit(k: usize, e: usize, f: u64, stop: usize, victim_txg: u64) {
    let mut pool = build();
    let mut seed = 100;
    for _ in 0..k {
        overwrite(&mut pool, seed);
        seed += 1;
    }
    for _ in 0..e {
        let floor = pool.current.root.rollback_floor;
        empty_publish(&mut pool, floor);
    }
    let old_floor = effective_floor(&pool.devices);
    let victim = roots_of(&pool.devices).into_iter().find(|r| r.checkpoint_txg.0 == victim_txg).expect("受害根");
    let victim_slots = referenced_slots(&pool.devices, &victim);
    println!("{}", state_line("S4 before-raise", &pool));
    println!("S4 old F_eff={} victim root txg={} referenced slots={:?}", old_floor.0, victim_txg, victim_slots);
    let (ceiling, publishes, reclaimed) = raise_floor_manual(&mut pool, CheckpointTxg(f), stop, false);
    println!("S4 ceiling={} reclaimed={:?}", ceiling.0, reclaimed.iter().map(|p| p.slot.0).collect::<Vec<_>>());
    for p in &publishes {
        println!("S4 publish txg={} F={} wrote={:?}", p.root.checkpoint_txg.0, p.root.rollback_floor.0, written_slots(p));
    }
    println!("{}", state_line("S4 crash-state", &pool));
    let sb = choose_superblock(&pool.devices).unwrap();
    let newest = choose_root(&pool.devices, &sb).unwrap();
    println!("S4 crash-state newest root txg={} F={} ; F_eff (recovery definition)={} ; victim still in ring={}",
        newest.checkpoint_txg.0, newest.rollback_floor.0, effective_floor(&pool.devices).0,
        roots_of(&pool.devices).iter().any(|r| r.checkpoint_txg.0 == victim_txg));
    let mut rolled = pool.clone();
    match rollback(&mut rolled, 1, victim_txg) {
        Ok(()) => println!("{}", state_line("S4 crash->rollback(victim)", &rolled)),
        Err(error) => println!("S4 crash->rollback(victim) refused/failed: {error}"),
    }
    let mut remounted = pool.clone();
    remount(&mut remounted);
    println!("{}", state_line("S4 crash->remount", &remounted));
}

/// S2′：只用合法的操作（覆盖写、重开、抬 F）找 S2 那种格：k1 次覆盖写 → 重开（写行 + 暖机）→ k2 次覆盖写 → 抬 F 到 f。
/// 只报「干净的崩溃态」：写到受害槽的那次发布不是让 F 生效的最后一次、受害根没被这次发布自己的根盖掉、受害根 txg ≥ 旧 F。
fn scenario_legal_search(max_k1: usize, max_k2: usize) {
    let fsheng = std::env::var("SFPROBE_RAISE").as_deref() == Ok("fsheng");
    let mut cells = 0u64;
    let mut clean_hits = 0u64;
    for k1 in 1..=max_k1 {
        let mut stage1 = build();
        for seed in 0..k1 {
            overwrite(&mut stage1, 200 + seed);
        }
        remount(&mut stage1);
        let mut base = stage1;
        for k2 in 0..=max_k2 {
            if k2 > 0 {
                overwrite(&mut base, 400 + k2);
            }
            if base.allocator.records().len() + 8 * 3 + 16 > 813 {
                break;
            }
            let sb = choose_superblock(&base.devices).unwrap();
            let records = scan_journal(&base.devices, &sb);
            let table = newest_table(&base.devices);
            let old_floor = effective_floor(&base.devices);
            let Some(ceiling) = rollback_floor_ceiling(&base.devices, &sb, &records, base.current.root.rollback_floor, table.as_ref().unwrap()) else { continue };
            for f in (old_floor.0 + 1)..=ceiling.0 {
                cells += 1;
                let mut pool = base.clone();
                let (_, publishes, reclaimed) = raise_floor_manual(&mut pool, CheckpointTxg(f), 3, fsheng);
                let reclaimed_slots: BTreeSet<u64> = reclaimed.iter().flat_map(|p| p.slot.0..p.slot.0 + p.span).collect();
                for (index, p) in publishes.iter().enumerate() {
                    let last = index + 1 == publishes.len();
                    let txg = p.root.checkpoint_txg.0;
                    for (role, slot, span) in written_slots(p) {
                        if !fsheng && !(slot..slot + span).any(|s| reclaimed_slots.contains(&s)) {
                            continue;
                        }
                        let overwritten: BTreeSet<u64> = publishes[..=index].iter().map(|q| q.root.checkpoint_txg.0.saturating_sub(24)).collect();
                        for victim in roots_of(&base.devices) {
                            let vt = victim.checkpoint_txg.0;
                            if vt < old_floor.0 || vt >= f || abandoned(&victim, &table) || overwritten.contains(&vt) {
                                continue;
                            }
                            if (slot..slot + span).any(|s| referenced_slots(&base.devices, &victim).contains(&s)) {
                                if !last {
                                    clean_hits += 1;
                                }
                                println!("S2L HIT k1={k1} k2={k2} f={f} old_F={} publish#{index} txg={txg} last={last} role={role:?} slot={slot} victim_root=({},{})", old_floor.0, victim.instance.0, vt);
                            }
                        }
                    }
                }
            }
        }
    }
    println!("S2L SUMMARY cells={cells} clean_hits={clean_hits}");
}

/// S4′：S2′ 的一格停在崩溃态上看后果。
fn scenario_legal_hit(k1: usize, k2: usize, f: u64, stop: usize, victim_instance: u32, victim_txg: u64) {
    let mut pool = build();
    for seed in 0..k1 {
        overwrite(&mut pool, 200 + seed);
    }
    remount(&mut pool);
    for index in 1..=k2 {
        overwrite(&mut pool, 400 + index);
    }
    println!("{}", state_line("S4L before-raise", &pool));
    let fsheng = std::env::var("SFPROBE_RAISE").as_deref() == Ok("fsheng");
    let (ceiling, publishes, reclaimed) = raise_floor_manual(&mut pool, CheckpointTxg(f), stop, fsheng);
    println!("S4L ceiling={} reclaimed_count={}", ceiling.0, reclaimed.len());
    for p in &publishes {
        println!("S4L publish txg={} F={} wrote={:?}", p.root.checkpoint_txg.0, p.root.rollback_floor.0, written_slots(p));
    }
    println!("{}", state_line("S4L crash-state", &pool));
    let mut rolled = pool.clone();
    match rollback(&mut rolled, victim_instance, victim_txg) {
        Ok(()) => println!("{}", state_line("S4L crash->rollback(victim)", &rolled)),
        Err(error) => println!("S4L crash->rollback(victim) failed: {error}"),
    }
    let mut remounted = pool.clone();
    remount(&mut remounted);
    println!("{}", state_line("S4L crash->remount", &remounted));
}

/// 下一次发布的根槽写落不下（槽里留旧内容）：返回要清掉的 (盘, 偏移)。
fn drop_next_root_write(pool: &mut Pool) -> (usize, u64) {
    let target = target_for_publish(CheckpointTxg(pool.current.root.checkpoint_txg.0 + 1));
    let device = parameters().region_devices[target.region as usize].0 as usize;
    let offset = singlefs_core::root_ring::slot_offset(target, 4096).0;
    pool.devices[device].1.dropped_offsets.insert(offset);
    (device, offset)
}

/// S5：单实例覆盖写到 drop_txg − 1，drop_txg 那次的根槽写落不下（环满之后槽里留 24 代之前的旧根），再覆盖写到 last_txg。
fn scenario_hole(drop_txg: u64, last_txg: u64) {
    let mut pool = build();
    let mut seed = 500;
    while pool.current.root.checkpoint_txg.0 < last_txg {
        let dropped = if pool.current.root.checkpoint_txg.0 + 1 == drop_txg { Some(drop_next_root_write(&mut pool)) } else { None };
        overwrite(&mut pool, seed);
        seed += 1;
        if let Some((device, offset)) = dropped {
            pool.devices[device].1.dropped_offsets.remove(&offset);
        }
        if pool.current.root.checkpoint_txg.0 + 3 >= drop_txg {
            println!("{}", state_line(&format!("S5 hole@{drop_txg}"), &pool));
        }
        if pool.allocator.records().len() + 16 > 813 {
            println!("S5 stop: 分配记录树一个节点装不下下一次覆盖写");
            break;
        }
    }
}

/// S6：短历史 + 抬 F（F-生：两条带新 F 的根都持久之后才回收），用来在上面叠坏镜像的环境变量。
fn scenario_short(overwrites: usize, floor: u64, fsheng: bool) {
    let mut pool = build();
    println!("{}", state_line("S6 A", &pool));
    for index in 0..overwrites {
        overwrite(&mut pool, 600 + index);
        println!("{}", state_line("S6 overwrite", &pool));
    }
    let (ceiling, publishes, reclaimed) = raise_floor_manual(&mut pool, CheckpointTxg(floor), 3, fsheng);
    println!("S6 raise F={floor} ceiling={} publishes={:?} reclaimed={}", ceiling.0, publishes.iter().map(|p| p.root.checkpoint_txg.0).collect::<Vec<_>>(), reclaimed.len());
    println!("{}", state_line("S6 after-raise", &pool));
    overwrite(&mut pool, 700);
    println!("{}", state_line("S6 overwrite-after-raise", &pool));
}

/// S7：坏镜像 ②——不抬 F 就回收（复用窗口置 0），下一次覆盖写把仍被候选根引用的槽发出去（步 5 验收那条必红的形态）。
fn scenario_reuse_candidate() {
    let mut pool = milestone_through_14();
    let reclaimed = pool.allocator.reclaim_released_up_to(CheckpointTxg(11));
    overwrite(&mut pool, 31);
    println!("S7 reclaimed={} E data slot={}", reclaimed.len(), pool.current.data_pointer.locations[0].slot.0);
    println!("{}", state_line("S7 reuse-without-raising-F", &pool));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("turnover") => scenario_turnover(31, None),
        Some("turnover-remount26") => scenario_turnover(31, Some(26)),
        Some("window-search") => scenario_window_search(args[2].parse().unwrap(), args[3].parse().unwrap()),
        Some("crash-between") => {
            scenario_crash_between(false);
            scenario_crash_between(true);
        }
        Some("window-hit") => scenario_window_hit(
            args[2].parse().unwrap(),
            args[3].parse().unwrap(),
            args[4].parse().unwrap(),
            args[5].parse().unwrap(),
            args[6].parse().unwrap(),
        ),
        Some("hole") => scenario_hole(args[2].parse().unwrap(), args[3].parse().unwrap()),
        Some("short") => scenario_short(args[2].parse().unwrap(), args[3].parse().unwrap(), args[4] == "fsheng"),
        Some("reuse-candidate") => scenario_reuse_candidate(),
        Some("crash-between-post") => scenario_crash_between(true),
        Some("legal-search") => scenario_legal_search(args[2].parse().unwrap(), args[3].parse().unwrap()),
        Some("legal-hit") => scenario_legal_hit(
            args[2].parse().unwrap(),
            args[3].parse().unwrap(),
            args[4].parse().unwrap(),
            args[5].parse().unwrap(),
            args[6].parse().unwrap(),
            args[7].parse().unwrap(),
        ),
        _ => eprintln!("用法：turnover | turnover-remount26 | window-search K E | crash-between | window-hit k e f stop victim"),
    }
    println!("PROBE-COMPLETE");
}
