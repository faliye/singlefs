//! m3-delete-r3 攻方腿（副本专用，被攻过零轮）：在 m3-delete-r2 攻方原型（`core-prototype-r2.diff`：从尾部截 extent 的删除批、
//! nlink = 0 开关、测试当账房的 `LEDGER_FOR_TESTS`、只关准入式子那一判 / 落点那一判的两个开关）之上，量全池一份环窗口（S2 第 10 条、K-U4）
//! 在几种盘大小与根环长度上的顶点、环窗口本身多大、删除途中式子算出的可用、回退那次加一个码 3 谱系容器之后过不过准入。
//! 原型里没有墓碑树、`deleted_inodes`、谱系记录：它们的单元数按 S2 的形态推（报告里逐处标「推的」）。全部在内存稀疏盘上跑；
//! 不枚举任何崩溃状态。只打印 `E name=...` 行；断言只钉看到的事实。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::{
    admission_reading_before_a_publish, admit_on_every_device, checkpoint_cost_of_the_version_to_build_on,
    BytesOnOneDevice, DemandOnDevice, LEDGER_FOR_TESTS,
};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_sequential_write, publish_unlink_batch_for_tests, warm_up, FirstFile,
    PoolVersion, PoolWriter, PublishError, TransactionOutput, FORMULA_JUDGEMENT_SKIPPED_FOR_TESTS,
    LANDING_JUDGEMENT_SKIPPED_FOR_TESTS,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const WRITE_TIME: u64 = 1_788_000_000;
const DEVICES: u64 = 2;
const MIB: u64 = 1 << 20;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

#[derive(Clone, Copy, Debug)]
struct Geometry {
    image_bytes: u64,
    ring_slots_per_region: u64,
    journal_ring_bytes: u64,
}

fn parameters(geometry: Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"_SFS-M3DEL-ATK3\0",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: geometry.journal_ring_bytes,
            root_ring_slots_per_region: if geometry.ring_slots_per_region == 8 {
                RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM
            } else {
                RootRingSlotsPerRegion::from_system_configuration_field(geometry.ring_slots_per_region).expect("S 在区间里")
            },
        },
    }
}

fn content_needing(units: u64, seed: u64) -> Vec<u8> {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    (0..(units - 1) * payload + payload / 2)
        .map(|index| u8::try_from((index * 31 + seed * 7 + 1) % 251).expect("小于 256"))
        .collect()
}

fn memory_pool_of(devices: &Devices, geometry: Geometry) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: geometry.image_bytes,
    }
}

fn devices_of(image: &MemoryPool, geometry: Geometry) -> Devices {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(geometry.image_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

struct Pool {
    geometry: Geometry,
    devices: Devices,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
}

impl Clone for Pool {
    fn clone(&self) -> Self {
        Pool {
            geometry: self.geometry,
            devices: devices_of(&memory_pool_of(&self.devices, self.geometry), self.geometry),
            allocator: self.allocator.clone(),
            current: self.current.clone(),
            instance: self.instance,
        }
    }
}

/// mkfs → 取号 → 暖机 → 第一个文件 →（`units` > 1 时）顺序写成 `units` 个单元 → 关进程重开可写挂载（分配器按根环回收）。
fn mounted_pool_with_a_file(units: u64, geometry: Geometry) -> Result<Pool, String> {
    let parameters = parameters(geometry);
    let mut devices: Devices = (0..DEVICES)
        .map(|number| {
            (DeviceIdentity(u32::try_from(number).expect("盘号")), SparseBlockDevice::new(geometry.image_bytes, PhysicalBlockSizeInBytes(512)))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs:{error:?}"))?;
    let mut allocator = PoolAllocator::new(
        (0..DEVICES)
            .map(|number| {
                DeviceFreeMap::with_unit_area_start(
                    DeviceIdentity(u32::try_from(number).expect("盘号")),
                    geometry.image_bytes,
                    UnitAreaStart::following_the_journal_ring(geometry.journal_ring_bytes).expect("单元区起点在段边界上"),
                )
            })
            .collect(),
    );
    allocator.mark_format_time_units(genesis.instance_table_placement(), genesis.tree_table_genesis_placement());
    let first_content = content_needing(1, 1);
    {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).map_err(|error| format!("取号:{error:?}"))?;
        let warm = warm_up(&mut pool, &genesis.root, instance).map_err(|error| format!("暖机:{error:?}"))?;
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机根"),
            FirstFile { content: &first_content, write_time_seconds: WRITE_TIME },
            instance,
            &warm.last_record_bytes,
        )
        .map_err(|error| format!("第一个文件:{error:?}"))?;
        if units > 1 {
            let content = content_needing(units, 2);
            publish_sequential_write(&mut pool, &mut allocator, &first, FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 }, instance)
                .map_err(|error| format!("顺序写:{error:?}"))?;
        }
    }
    let mounted = mount_writable(&parameters, &mut devices).map_err(|error| format!("可写挂载:{error:?}"))?;
    let PoolVersion::WithFile(current) = mounted.current else { return Err("挂载之后没有文件".to_owned()) };
    Ok(Pool { geometry, devices, allocator: mounted.allocator, current, instance: mounted.output.instance })
}

fn publish_on(
    pool: &mut Pool,
    step: impl FnOnce(&mut PoolWriter<'_, SparseBlockDevice>, &mut PoolAllocator, &TransactionOutput, InstanceGeneration) -> Result<TransactionOutput, PublishError>,
) -> Result<TransactionOutput, PublishError> {
    let parameters = parameters(pool.geometry);
    let next = {
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        step(&mut writer, &mut pool.allocator, &pool.current, pool.instance)?
    };
    pool.current = next.clone();
    Ok(next)
}

fn set_nlink_zero_for(inode: Option<u64>) {
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(inode));
}

fn set_ledger(ledger: Option<(u64, u64)>) {
    LEDGER_FOR_TESTS.with(|cell| cell.set(ledger));
}

fn set_delete_switches(skip_formula: bool, skip_landing: bool) {
    FORMULA_JUDGEMENT_SKIPPED_FOR_TESTS.with(|cell| cell.set(skip_formula));
    LANDING_JUDGEMENT_SKIPPED_FOR_TESTS.with(|cell| cell.set(skip_landing));
}

/// 这次发布新占的槽（盘 0；分配代 = 这次 txg、未释放的分配记录）。
fn new_slots_of(output: &TransactionOutput) -> u64 {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released && record.generation == output.root.checkpoint_txg)
        .map(|record| u64::from(record.span_slots))
        .sum()
}

/// 删除计划：意图那次（全留、size 不变）+ 从尾部每批去 `batch` 个单元。
fn delete_plan(current: &TransactionOutput, batch: u64) -> Vec<(String, u64, u64)> {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let units = u64::try_from(current.data_pointers.len()).expect("单元数");
    let mut steps = vec![("intent".to_owned(), units, current.inode_record.size)];
    let mut keep = units;
    let mut number = 0;
    while keep > 0 {
        number += 1;
        keep = keep.saturating_sub(batch);
        steps.push((format!("batch_{number}"), keep, keep * payload));
    }
    steps
}

/// 按计划删；每次发布之后调 `after`（拿名字、这次新占的槽与池）。出错时交回出错那一步的名字。
fn run_delete(pool: &mut Pool, batch: u64, after: &mut dyn FnMut(&str, u64, &Pool)) -> Result<usize, (String, String)> {
    set_nlink_zero_for(Some(1));
    let steps = delete_plan(&pool.current, batch);
    let count = steps.len();
    for (name, keep, size) in steps {
        let result = publish_on(pool, |writer, allocator, previous, instance| {
            publish_unlink_batch_for_tests(writer, allocator, previous, keep, size, instance)
        });
        match result {
            Ok(output) => after(&name, new_slots_of(&output), pool),
            Err(error) => {
                set_nlink_zero_for(None);
                return Err((name, short_error(&error)));
            }
        }
    }
    set_nlink_zero_for(None);
    Ok(count)
}

fn short_error(error: &PublishError) -> String {
    match error {
        PublishError::SpaceAdmissionRefused(refusal) => format!(
            "SpaceAdmissionRefused{:?}",
            refusal.short_devices.iter().map(|short| (short.available.0, short.demand.0)).collect::<Vec<_>>()
        ),
        other => {
            let text = format!("{other:?}");
            text.chars().take(160).collect()
        }
    }
}

/// 式子算出的逐盘可用（字节），账取线程局部里此刻那一对。
fn available(pool: &Pool) -> Vec<i128> {
    admission_reading_before_a_publish(&pool.allocator, Some(&pool.current))
        .available_on_each_device()
        .into_iter()
        .map(|(_, available)| available.0)
        .collect()
}

/// 账为 0 时式子扣掉的保留量（挂载期承诺量 + checkpoint 保留池 ÷ 副本数），盘 0，槽。
fn reserve_slots(pool: &Pool) -> i128 {
    let saved = LEDGER_FOR_TESTS.with(std::cell::Cell::get);
    set_ledger(Some((0, 0)));
    let free = i128::from(pool.allocator.devices[0].free_slots());
    let reserve = free - available(pool)[0] / i128::from(SLOT_BYTES);
    set_ledger(saved);
    reserve
}

/// 把每块盘的空闲槽从单元区末尾往前标成已分配（只动分配器，盘上不写），直到式子算出的可用 ≤ `leave_bytes`。
fn fill_until_available_is_at_most(pool: &mut Pool, leave_bytes: i128) {
    let available_now = available(pool);
    for (map, available) in pool.allocator.devices.iter_mut().zip(available_now) {
        let mut to_take = (available - leave_bytes).max(0) / i128::from(SLOT_BYTES);
        let start = map.unit_area_start().slot().0;
        let mut slot = start + map.unit_area_slots();
        while to_take > 0 && slot > start {
            slot -= 1;
            if map.is_free(SlotNumber(slot)) {
                map.mark_allocated(SlotNumber(slot), 1);
                to_take -= 1;
            }
        }
    }
}

fn bytes_summed_of_slots(slots: u64) -> u64 {
    slots * SLOT_BYTES * DEVICES
}

/// 一个几何上：探针量 W（同一个池拷一份、不填满、照常删一遍：意图那次与各批新占的槽），环窗口 = (3S + 1) × W_max（S2 第 10 条
/// 「(环里根数 + 1) × 每批上界」，上界取探针量到的最大一次，含意图那次；墓碑树与 `deleted_inodes` 那 3–4 槽原型里没有，推的）。
/// 账里只给环窗口那一份（已承诺预留）、待删占用给 0，把池填到式子可用 = 0，删除各批关式子那一判、落点那一判照判，量借到的顶点。
fn ring_window_row(journal_mib: u64, image_mib: u64, slots_per_region: u64, units: u64, batch: u64) {
    ring_window_row_with(journal_mib, image_mib, slots_per_region, units, batch, None);
}

/// 同上；`w_bound` 给了时不探针，环窗口按「(3S + 1) × w_bound」算（w_intent、w_batch_max 两格报 0）。
fn ring_window_row_with(journal_mib: u64, image_mib: u64, slots_per_region: u64, units: u64, batch: u64, w_bound: Option<u64>) {
    let geometry = Geometry { image_bytes: image_mib * MIB, ring_slots_per_region: slots_per_region, journal_ring_bytes: journal_mib * MIB };
    let head = format!(
        "journal_mib={journal_mib} image_mib={image_mib} slots_per_region={slots_per_region} ring_roots={} units={units} batch={batch} w_bound={}",
        3 * slots_per_region,
        w_bound.map_or("probe".to_owned(), |w| w.to_string())
    );
    let base = match mounted_pool_with_a_file(units, geometry) {
        Ok(pool) => pool,
        Err(error) => {
            println!("E name=ring_window {head} setup={error}");
            return;
        }
    };
    let ring_roots = 3 * slots_per_region;
    let unit_area = base.allocator.devices[0].unit_area_slots();
    let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on(Some(&base.current), &base.allocator).0;
    set_ledger(None);
    set_delete_switches(false, false);
    let (w_intent, w_batch_max, w_max) = if let Some(w) = w_bound {
        (0, 0, w)
    } else {
        let mut probe = base.clone();
        let mut probe_rows: Vec<u64> = Vec::new();
        let probe_result = run_delete(&mut probe, batch, &mut |_, slots, _| probe_rows.push(slots));
        drop(probe);
        if let Err((name, error)) = probe_result {
            println!("E name=ring_window {head} probe=Err({name}:{error})");
            return;
        }
        let w_intent = probe_rows[0];
        let w_batch_max = *probe_rows[1..].iter().max().expect("至少一批");
        (w_intent, w_batch_max, w_intent.max(w_batch_max))
    };
    let window_slots = (ring_roots + 1) * w_max;
    let mut pool = base;
    set_ledger(Some((0, 0)));
    let reserve = reserve_slots(&pool);
    set_ledger(Some((0, bytes_summed_of_slots(window_slots))));
    let available_with_the_window_before_fill = available(&pool)[0] / i128::from(SLOT_BYTES);
    fill_until_available_is_at_most(&mut pool, 0);
    let free_before = i128::from(pool.allocator.devices[0].free_slots());
    set_delete_switches(true, false);
    let mut consumed: Vec<i128> = Vec::new();
    let mut negative = 0u64;
    let mut min_available = i128::MAX;
    let result = run_delete(&mut pool, batch, &mut |_, _, pool| {
        consumed.push(free_before - i128::from(pool.allocator.devices[0].free_slots()));
        let now = available(pool)[0];
        if now < 0 {
            negative += 1;
        }
        min_available = min_available.min(now);
    });
    set_delete_switches(false, false);
    set_ledger(None);
    let (at, vertex) = consumed.iter().enumerate().max_by_key(|(_, value)| **value).map(|(index, value)| (index + 1, *value)).unwrap_or((0, 0));
    println!(
        "E name=ring_window {head} unit_area_slots={unit_area} checkpoint_cost={checkpoint_cost} reserve_slots={reserve} w_intent={w_intent} w_batch_max={w_batch_max} window_slots={window_slots} window_per_mille_of_unit_area={} available_slots_with_the_window_before_fill={available_with_the_window_before_fill} free_slots_at_full={free_before} publishes={} result={} vertex_slots={vertex} at_publish={at} vertex_le_window={} publishes_with_negative_available={negative} min_available_bytes={min_available} consumed_after_last={}",
        window_slots * 1000 / unit_area,
        consumed.len(),
        match &result { Ok(_) => "Ok".to_owned(), Err((name, error)) => format!("Err({name}:{error})") },
        vertex <= i128::from(window_slots),
        consumed.last().copied().unwrap_or(0)
    );
}

fn selected(variable: &str, index: usize) -> bool {
    std::env::var(variable).map_or(true, |list| list.split(',').any(|item| item.trim() == index.to_string()))
}

/// G1：环窗口在几种几何上（journal 环 / 盘大小：1 MiB / 32 MiB（最小档）、16 / 128、64 / 512、768 / 3072（默认环的最小盘）、768 / 4096；S = 4 / 8 / 16）。`M3R3_RING_ROWS` 给逐个下标时只跑那几行（分批跑）。
#[test]
fn ring_window_across_geometries() {
    let rows: [(u64, u64, u64, u64, u64); 14] = [
        (1, 32, 4, 30, 1),
        (1, 32, 16, 70, 1),
        (16, 128, 4, 30, 1),
        (16, 128, 16, 70, 1),
        (64, 512, 4, 30, 1),
        (64, 512, 16, 70, 1),
        (64, 512, 16, 2000, 144),
        (768, 3072, 4, 30, 1),
        (768, 3072, 16, 70, 1),
        (768, 4096, 4, 30, 1),
        (768, 4096, 8, 40, 1),
        (768, 4096, 16, 70, 1),
        (768, 4096, 16, 1000, 144),
        (768, 4096, 16, 3000, 144),
    ];
    for (index, (journal_mib, image_mib, slots_per_region, units, batch)) in rows.into_iter().enumerate() {
        if selected("M3R3_RING_ROWS", index) {
            ring_window_row(journal_mib, image_mib, slots_per_region, units, batch);
        }
    }
}

fn admission_line(pool: &Pool, demand_slots: u64, raise_bytes_summed: u64) -> String {
    let demand: Vec<DemandOnDevice> = pool
        .allocator
        .devices
        .iter()
        .map(|map| DemandOnDevice { device: map.device, bytes: BytesOnOneDevice(demand_slots * SLOT_BYTES + raise_bytes_summed / DEVICES) })
        .collect();
    match admit_on_every_device(&admission_reading_before_a_publish(&pool.allocator, Some(&pool.current)), &demand) {
        Ok(()) => "Ok".to_owned(),
        Err(refusal) => format!("Refused{:?}", refusal.short_devices.iter().map(|short| (short.available.0, short.demand.0)).collect::<Vec<_>>()),
    }
}

/// G3：池（4 GiB、S = 8）上一个 300 单元文件 A，删完（K = 144，意图 + 3 批；原型里 inode 1 留着、nlink = 0、0 个单元）之后，
/// 账里给「环窗口 + 每对象一份墓碑」，填到式子可用 = 0，再回退到 A。今天的回退那次发布没有普通分配、不判；S2 加了谱系容器之后
/// 它多写 2 槽（谱系容器）或 3 槽（挂墓碑树下、连墓碑树根），若照「有普通分配就判」判、需求再加「标签 6 从 R_old 重算抬上去的那一份」，
/// 过不过准入。每对象墓碑按记录（40 字节一份）与按整容器（32 KiB 一份）两种读法；对象数按「nlink = 1 的 inode」数。
#[test]
fn rollback_with_a_lineage_container_on_a_full_pool() {
    let geometry = Geometry { image_bytes: 4096 * MIB, ring_slots_per_region: 8, journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES };
    let base = mounted_pool_with_a_file(300, geometry).expect("池");
    let version_a = base.current.clone();
    set_ledger(None);
    set_delete_switches(false, false);
    let mut probe_rows: Vec<u64> = Vec::new();
    let mut pool = base;
    run_delete(&mut pool, 144, &mut |_, slots, _| probe_rows.push(slots)).expect("删 A");
    let w_max = *probe_rows.iter().max().expect("有");
    let window = bytes_summed_of_slots((3 * 8 + 1) * w_max);
    for (reading, tomb_one_copy) in [("per_record", 40u64), ("per_container", 32768)] {
        let ledger_at_cur = window;
        let ledger_at_a = window + tomb_one_copy * DEVICES;
        let mut full = pool.clone();
        set_ledger(Some((0, ledger_at_cur)));
        fill_until_available_is_at_most(&mut full, 0);
        let available_at_cur = available(&full);
        let raise = ledger_at_a - ledger_at_cur;
        let judged: Vec<String> = [(0u64, 0u64), (2, 0), (3, 0), (0, raise), (2, raise), (3, raise)]
            .into_iter()
            .map(|(slots, raise)| format!("{slots}slots+{raise}raise:{}", admission_line(&full, slots, raise)))
            .collect();
        let rolled = roll_back_by_a_forward_publish(
            &parameters(geometry),
            &mut full.devices,
            &mut full.allocator,
            &mut full.current,
            RollbackTarget { instance: version_a.root.instance, checkpoint_txg: CheckpointTxg(version_a.root.checkpoint_txg.0) },
        );
        let rollback_new_slots = new_slots_of(&full.current);
        set_ledger(Some((0, ledger_at_a)));
        let available_after = available(&full);
        set_ledger(None);
        println!(
            "E name=rollback_full_pool tomb_reading={reading} w_max={w_max} window_bytes_summed={window} ledger_at_cur={ledger_at_cur} ledger_at_a={ledger_at_a} available_at_cur={available_at_cur:?} judged=[{}] today_rollback={} rollback_new_slots={rollback_new_slots} units_after={} available_after_with_tag6_from_r_old={available_after:?}",
            judged.join(" "),
            match &rolled { Ok(_) => "Ok".to_owned(), Err(error) => format!("Err({error:?})") },
            full.current.data_pointers.len()
        );
    }
}

/// G6：墓碑树根装几个容器、一个容器装几条墓碑（40 字节）、几条谱系记录（16 字节）；容量旋钮压到 1 / 2 条时根装满要几次 unlink。
#[test]
fn tomb_tree_root_capacity() {
    let unit_bytes: u64 = 32768;
    let header = singlefs_format::PACKED_UNIT_RECORDS_OFFSET;
    let root_entries = u64::try_from(singlefs_core::inode_tree::leaf_containers_one_root_node_holds()).expect("135");
    let tombs = (unit_bytes - header) / 40;
    let lineage = (unit_bytes - header) / 16;
    println!(
        "E name=tomb_root_capacity containers_one_root_holds={root_entries} tombs_per_container={tombs} unlinks_until_the_root_is_full={} with_knob_1={} with_knob_2={} lineage_records_per_container={lineage}",
        root_entries * tombs,
        root_entries,
        root_entries * 2
    );
}

fn role_kind(role: &singlefs_core::transaction::TransactionUnit) -> &'static str {
    use singlefs_core::transaction::TransactionUnit as U;
    match role {
        U::Data(_) => "data",
        U::ExtentLowerNode(_) => "extent_lower",
        U::ExtentUpperNodeBelowTheRoot(_) => "extent_upper",
        U::ExtentRoot => "extent_root",
        U::InodeLeafContainer(_) => "inode_leaf",
        U::InodeRoot => "inode_root",
        U::AllocationTreeNodeBelowTheRoot(_) => "allocation_node",
        U::AllocationTree => "allocation_root",
        U::AccountingTreeNodeBelowTheRoot(_) => "accounting_node",
        U::AccountingTree => "accounting_root",
        U::MappingTreeNodeBelowTheRoot(_) => "mapping_node",
        U::MappingTree => "mapping_root",
        U::TreeTable => "tree_table",
        U::InstanceTable | U::InstanceTablePageAfterTheFirst(_) => "instance_table",
    }
}

/// G1：一批（K = 144）新写多少槽随文件大小怎么走（4 GiB、S = 8；不填满、照常删一遍），最大那一批按角色拆开。
/// `M3R3_W_UNITS` 给逗号分隔的单元数（默认 1000,3000,6000,10000）。
#[test]
fn batch_w_against_file_size() {
    let list = std::env::var("M3R3_W_UNITS").unwrap_or_else(|_| "1000,3000,6000,10000".to_owned());
    for units in list.split(',').map(|item| item.trim().parse::<u64>().expect("单元数")) {
        let geometry = Geometry { image_bytes: 4096 * MIB, ring_slots_per_region: 8, journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES };
        let mut pool = mounted_pool_with_a_file(units, geometry).expect("池");
        set_ledger(None);
        set_delete_switches(false, false);
        let mut rows: Vec<(String, u64, String)> = Vec::new();
        run_delete(&mut pool, 144, &mut |name, slots, pool| {
            let mut kinds: std::collections::BTreeMap<&str, u64> = std::collections::BTreeMap::new();
            for role in &pool.current.rewritten {
                *kinds.entry(role_kind(role)).or_default() += role.span_slots();
            }
            rows.push((name.to_owned(), slots, format!("{kinds:?}").replace(' ', "")));
        })
        .expect("删");
        let (max_name, max_slots, max_kinds) = rows.iter().max_by_key(|row| row.1).cloned().expect("有");
        let series: Vec<u64> = rows.iter().map(|row| row.1).collect();
        println!(
            "E name=batch_w units={units} batch=144 publishes={} max_new_slots={max_slots} at={max_name} rewritten_slots_by_kind_at_max={max_kinds} new_slots_series={series:?}",
            rows.len()
        );
    }
}

/// G1：大文件（K = 144，批数多于环长）时，环窗口的 W 若取小文件量到的一批（24 槽：1000 单元那一行）够不够；同一几何上再按
/// 这个文件自己量到的最大一批（87 槽：10000 单元那一行）算一次。`M3R3_LARGE` 给 `单元数:w_bound`（4 GiB 盘）或 `环 MiB:盘 MiB:单元数:w_bound`，逗号分隔（默认 4 GiB 与 512 MiB 盘各两行）。
#[test]
fn ring_window_with_a_large_file() {
    let list = std::env::var("M3R3_LARGE").unwrap_or_else(|_| "10000:24,10000:87,64:512:5000:24,64:512:5000:52".to_owned());
    for item in list.split(',') {
        let fields: Vec<&str> = item.trim().split(':').collect();
        let (journal_mib, image_mib, units, w) = match fields.as_slice() {
            [units, w] => (768, 4096, *units, *w),
            [journal, image, units, w] => (journal.parse().expect("环"), image.parse().expect("盘"), *units, *w),
            _ => panic!("单元数:w 或 环:盘:单元数:w"),
        };
        ring_window_row_with(journal_mib, image_mib, 16, units.parse().expect("单元数"), 144, Some(w.parse().expect("w")));
    }
}
