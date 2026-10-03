//! m3-delete-r2 攻方腿（副本专用，被攻过零轮）：在 m3-delete-r1 攻方原型（`publish_unlink_batch_for_tests`：从尾部截 extent、
//! inode 1 记录 nlink = 0、size 跟着截）之上，再加一个测试当账房的 R3 账（`admission::LEDGER_FOR_TESTS`：待删占用、已承诺预留
//! 两个按全部副本之和记的字节，准入读数与记账行标签 4、6 都读它）、两个只关准入的开关（式子那一判、落点那一判）、一个「把某个
//! inode 的记录原样重写一遍」的发布（空 inode 意图那次的代用）。只打印 `E name=...` 行；断言只钉看到的事实。
//! 全部在内存稀疏盘上跑；没有意图记录、没有 `deleted_inodes`、没有墓碑（它们的量按代码推，报告里逐处标）。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::{
    admission_reading_before_a_publish, admit_on_every_device, checkpoint_cost_of_the_version_to_build_on,
    space_budget_of_role, units_of_a_publish_to_land, BytesOnOneDevice, DemandOnDevice, SpaceBudgetOfARole,
    LEDGER_FOR_TESTS,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_rewrite_inode_record_for_tests, publish_sequential_write, publish_unlink_batch_for_tests,
    warm_up, FirstFile, PoolVersion, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
    FORMULA_JUDGEMENT_SKIPPED_FOR_TESTS, LANDING_JUDGEMENT_SKIPPED_FOR_TESTS,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const WRITE_TIME: u64 = 1_788_000_000;
const DEVICES: u64 = 2;
/// 产品批大小（正文 S1 第 2 条：`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）。
const PRODUCT_BATCH: u64 = 144;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

#[derive(Clone, Copy, Debug)]
struct Geometry {
    image_bytes: u64,
    ring_slots_per_region: u64,
}

const DEFAULT_GEOMETRY: Geometry = Geometry { image_bytes: 4 << 30, ring_slots_per_region: 8 };

fn parameters(geometry: Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"_SFS-M3DEL-ATK2\0",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: if geometry.ring_slots_per_region == 8 {
                RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM
            } else {
                RootRingSlotsPerRegion::from_system_configuration_field(geometry.ring_slots_per_region)
                    .expect("环槽数在区间里")
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
            devices: devices_of(&self.image(), self.geometry),
            allocator: self.allocator.clone(),
            current: self.current.clone(),
            instance: self.instance,
        }
    }
}

impl Pool {
    fn image(&self) -> MemoryPool {
        memory_pool_of(&self.devices, self.geometry)
    }
}

/// mkfs → 取号 → 暖机 → 第一个文件（一个单元）→（`units` > 1 时）顺序写成 `units` 个数据单元，同一个实例、同一个进程。
fn pool_with_a_file(units: u64, geometry: Geometry) -> Pool {
    let parameters = parameters(geometry);
    let mut devices: Devices = (0..DEVICES)
        .map(|number| {
            let identity = DeviceIdentity(u32::try_from(number).expect("盘号"));
            (identity, SparseBlockDevice::new(geometry.image_bytes, PhysicalBlockSizeInBytes(512)))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(
        (0..DEVICES)
            .map(|number| DeviceFreeMap::new(DeviceIdentity(u32::try_from(number).expect("盘号")), geometry.image_bytes))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let first_content = content_needing(1, 1);
    let (instance, current) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机根"),
            FirstFile { content: &first_content, write_time_seconds: WRITE_TIME },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个文件");
        let current = if units > 1 {
            let content = content_needing(units, 2);
            publish_sequential_write(
                &mut pool,
                &mut allocator,
                &first,
                FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 },
                instance,
            )
            .expect("顺序写")
        } else {
            first
        };
        (instance, current)
    };
    Pool { geometry, devices, allocator, current, instance }
}

/// 关进程、重开可写挂载：新实例、写行、暖机；之后的分配器装着根环表（回收按环里最旧有效根走）。
fn remounted(pool: &Pool) -> Pool {
    let mut devices = devices_of(&pool.image(), pool.geometry);
    let mounted = mount_writable(&parameters(pool.geometry), &mut devices).expect("可写挂载");
    let PoolVersion::WithFile(current) = mounted.current else { panic!("带文件的一版") };
    Pool {
        geometry: pool.geometry,
        devices,
        allocator: mounted.allocator,
        current,
        instance: mounted.output.instance,
    }
}

/// 发布一次，成功就推进现行那一版。`step` 拿写入口、分配器与现行那一版。
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

fn unlink_batch(pool: &mut Pool, keep_units: u64, size_after: u64) -> Result<TransactionOutput, PublishError> {
    publish_on(pool, |writer, allocator, previous, instance| {
        publish_unlink_batch_for_tests(writer, allocator, previous, keep_units, size_after, instance)
    })
}

fn set_nlink_zero_for(inode: Option<u64>) {
    singlefs_core::records::NLINK_ZERO_INODE_FOR_TESTS.with(|cell| cell.set(inode));
}

fn set_ledger(ledger: Option<(u64, u64)>) {
    LEDGER_FOR_TESTS.with(|cell| cell.set(ledger));
}

fn set_r3_batch_switches(skip_formula: bool, skip_landing: bool) {
    FORMULA_JUDGEMENT_SKIPPED_FOR_TESTS.with(|cell| cell.set(skip_formula));
    LANDING_JUDGEMENT_SKIPPED_FOR_TESTS.with(|cell| cell.set(skip_landing));
}

/// 一次发布的形状：这次新占的槽（盘 0；分配代 = 这次 txg、未释放的记录）、重写角色按空间归哪一格分的槽数、
/// 发布前那一刻的 ckpt_cost 与落点那一判要的提交内生槽数（`units_of_a_publish_to_land`）。
#[derive(Clone, Copy, Debug)]
struct Shape {
    txg: u64,
    new_slots: u64,
    ordinary_slots: u64,
    fixed_point_slots: u64,
    switch_reserve_slots: u64,
    checkpoint_cost: u64,
    landing_commit_generated_slots: u64,
    landing_data_units: u64,
}

fn slots_taken_by(output: &TransactionOutput, device: DeviceIdentity) -> u64 {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == device && !record.is_released && record.generation == output.root.checkpoint_txg)
        .map(|record| u64::from(record.span_slots))
        .sum()
}

fn shape_of(output: &TransactionOutput, previous: &TransactionOutput, allocator_before: &PoolAllocator) -> Shape {
    let span_of = |budget: SpaceBudgetOfARole| -> u64 {
        output
            .rewritten
            .iter()
            .filter(|role| space_budget_of_role(**role) == budget)
            .map(|role| role.span_slots())
            .sum()
    };
    let ordinary_roles: Vec<TransactionUnit> = output
        .rewritten
        .iter()
        .copied()
        .filter(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation)
        .collect();
    let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on(Some(previous), allocator_before);
    let landing = units_of_a_publish_to_land(&ordinary_roles, checkpoint_cost);
    Shape {
        txg: output.root.checkpoint_txg.0,
        new_slots: slots_taken_by(output, DeviceIdentity(0)),
        ordinary_slots: span_of(SpaceBudgetOfARole::OrdinaryAllocation),
        fixed_point_slots: span_of(SpaceBudgetOfARole::CheckpointReservePool),
        switch_reserve_slots: span_of(SpaceBudgetOfARole::InstanceSwitchReserve),
        checkpoint_cost: checkpoint_cost.0,
        landing_commit_generated_slots: landing.commit_generated_slots,
        landing_data_units: landing.data_units,
    }
}

fn shape_line(shape: &Shape) -> String {
    format!(
        "txg={} new_slots={} ordinary_slots={} fixed_point_slots={} switch_reserve_slots={} checkpoint_cost={} landing_commit_generated_slots={} landing_data_units={}",
        shape.txg,
        shape.new_slots,
        shape.ordinary_slots,
        shape.fixed_point_slots,
        shape.switch_reserve_slots,
        shape.checkpoint_cost,
        shape.landing_commit_generated_slots,
        shape.landing_data_units
    )
}

/// 删除的每一次发布：意图那次（留全部、size 不变、nlink = 0）与各批（从尾部每批去 `batch` 个单元，size 跟着截）。
/// `before_each` 在每次发布之前调（账房在这里改账）；交回每次发布的名字、形状与发布之后的分配器。
struct DeletePublish {
    name: String,
    keep: u64,
    shape: Shape,
    allocator_after: PoolAllocator,
    output: TransactionOutput,
}

fn delete_the_file(
    pool: &mut Pool,
    batch: u64,
    before_each: &mut dyn FnMut(&str, u64, &Pool),
) -> Result<Vec<DeletePublish>, (String, PublishError)> {
    let units = u64::try_from(pool.current.data_pointers.len()).expect("单元数");
    let size = pool.current.inode_record.size;
    let mut published = run_delete_publishes(pool, vec![("intent".to_owned(), units, size)], before_each)?;
    published.extend(continue_the_delete(pool, batch, before_each)?);
    Ok(published)
}

/// 续做：从现行那一版（意图之后的任一版）起，按批大小从尾部截到 0。计划只读现行那一版的单元数。
fn plan_of_the_continuation(current: &TransactionOutput, batch: u64) -> Vec<(String, u64, u64)> {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let mut keep = u64::try_from(current.data_pointers.len()).expect("单元数");
    let mut steps = Vec::new();
    let mut batch_number = 0;
    while keep > 0 {
        batch_number += 1;
        keep = keep.saturating_sub(batch);
        steps.push((format!("batch_{batch_number}"), keep, keep * payload));
    }
    steps
}

fn continue_the_delete(
    pool: &mut Pool,
    batch: u64,
    before_each: &mut dyn FnMut(&str, u64, &Pool),
) -> Result<Vec<DeletePublish>, (String, PublishError)> {
    let steps = plan_of_the_continuation(&pool.current, batch);
    run_delete_publishes(pool, steps, before_each)
}

fn run_delete_publishes(
    pool: &mut Pool,
    steps: Vec<(String, u64, u64)>,
    before_each: &mut dyn FnMut(&str, u64, &Pool),
) -> Result<Vec<DeletePublish>, (String, PublishError)> {
    set_nlink_zero_for(Some(1));
    let mut published = Vec::new();
    for (name, keep, size) in steps {
        before_each(&name, keep, pool);
        let previous = pool.current.clone();
        let allocator_before = pool.allocator.clone();
        let output = unlink_batch(pool, keep, size).map_err(|error| {
            set_nlink_zero_for(None);
            (name.clone(), error)
        })?;
        let shape = shape_of(&output, &previous, &allocator_before);
        published.push(DeletePublish { name, keep, shape, allocator_after: pool.allocator.clone(), output });
    }
    set_nlink_zero_for(None);
    Ok(published)
}

fn per_device_counts(allocator: &PoolAllocator) -> String {
    allocator
        .devices
        .iter()
        .map(|map| {
            format!(
                "dev{}_allocated={} dev{}_free={} dev{}_deferred={}",
                map.device.0,
                map.allocated_slots(),
                map.device.0,
                map.free_slots(),
                map.device.0,
                map.deferred_slots()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 式子算出的逐盘可用（字节），账取线程局部里此刻那一对。
fn available(pool: &Pool) -> Vec<i128> {
    admission_reading_before_a_publish(&pool.allocator, Some(&pool.current))
        .available_on_each_device()
        .into_iter()
        .map(|(_, available)| available.0)
        .collect()
}

/// 账为 0 时式子扣掉的「挂载期承诺量 + checkpoint 保留池 ÷ 副本数」（逐盘、槽）：空闲槽 × 16384 − 可用。
fn reserve_slots(pool: &Pool) -> Vec<i128> {
    let saved = LEDGER_FOR_TESTS.with(std::cell::Cell::get);
    set_ledger(Some((0, 0)));
    let free: Vec<i128> = pool.allocator.devices.iter().map(|map| i128::from(map.free_slots())).collect();
    let reserve = available(pool)
        .into_iter()
        .zip(free)
        .map(|(available, free)| free - available / i128::from(SLOT_BYTES))
        .collect();
    set_ledger(saved);
    reserve
}

/// 把每块盘的空闲槽从单元区末尾往前标成已分配（只动分配器拷贝，盘上不写），直到式子算出的可用 ≤ `leave_bytes`。
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

fn checker_red(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}:{detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

/// 这一版 inode 1 那条记录盘上的 nlink（第一片叶容器里第 0 条记录偏移 36 的 4 字节小端）。
fn raw_nlink_of_inode_one(output: &TransactionOutput) -> u32 {
    let leaf = output
        .units
        .iter()
        .find(|unit| matches!(unit.identity, TransactionUnit::InodeLeafContainer(index) if index.position() == 0))
        .expect("第一片 inode 叶容器");
    let at = usize::try_from(singlefs_format::PACKED_UNIT_RECORDS_OFFSET).expect("偏移") + 36;
    u32::from_le_bytes(leaf.bytes[at..at + 4].try_into().expect("4 字节"))
}

fn rewrites_an_inode_leaf(output: &TransactionOutput) -> bool {
    output.rewritten.iter().any(|unit| matches!(unit, TransactionUnit::InodeLeafContainer(_)))
}

fn accounting_row(output: &TransactionOutput, statistic: u16) -> u64 {
    output
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == statistic)
        .map(|entry| entry.value)
        .expect("这一行在")
}

/// V2：意图那次与每一批新占的槽，按文件单元数 N 与批大小 K 扫（mkfs 同一个进程里删，分配器不回收）。
/// 「普通分配」那一半是 extent 节点、inode 叶与根；「固定点」是分配记录树、记账树、映射树、树表。
#[test]
fn w_of_each_delete_publish_across_file_and_batch_sizes() {
    let cases: [(u64, u64); 15] = [
        (1, 1),
        (1, PRODUCT_BATCH),
        (3, 1),
        (3, PRODUCT_BATCH),
        (6, 2),
        (6, PRODUCT_BATCH),
        (143, PRODUCT_BATCH),
        (144, PRODUCT_BATCH),
        (145, PRODUCT_BATCH),
        (145, 1),
        (289, PRODUCT_BATCH),
        (433, PRODUCT_BATCH),
        (600, PRODUCT_BATCH),
        (600, 100),
        (1000, PRODUCT_BATCH),
    ];
    let mut sums: Vec<(u64, u64, u64, u64, u64)> = Vec::new();
    for (units, batch) in cases {
        let mut pool = pool_with_a_file(units, DEFAULT_GEOMETRY);
        let published = delete_the_file(&mut pool, batch, &mut |_, _, _| {}).expect("删");
        for publish in &published {
            println!(
                "E name=w_row units={units} batch={batch} step={} keep={} {}",
                publish.name,
                publish.keep,
                shape_line(&publish.shape)
            );
        }
        let intent = published[0].shape.new_slots;
        let batches: Vec<u64> = published[1..].iter().map(|publish| publish.shape.new_slots).collect();
        let max_batch = *batches.iter().max().expect("至少一批");
        let sum: u64 = published.iter().map(|publish| publish.shape.new_slots).sum();
        println!(
            "E name=w_sum units={units} batch={batch} publishes={} intent_new_slots={intent} batch_new_slots={batches:?} max_batch_new_slots={max_batch} sum_new_slots={sum} max_fixed_point_slots={} max_landing_commit_generated_slots={}",
            published.len(),
            published.iter().map(|publish| publish.shape.fixed_point_slots).max().expect("有"),
            published.iter().map(|publish| publish.shape.landing_commit_generated_slots).max().expect("有")
        );
        sums.push((units, batch, intent, max_batch, sum));
    }
    // K 是只供测试的旋钮时预付按产品 K 算：三单元文件在 K = 144 下预付「意图 + 1 批」，K = 1 下实际走「意图 + 3 批」。
    let of = |units: u64, batch: u64| *sums.iter().find(|row| row.0 == units && row.1 == batch).expect("这一格跑过");
    for (units, test_batch) in [(3u64, 1u64), (145, 1), (600, 100)] {
        let (_, _, intent_product, max_batch_product, _) = of(units, PRODUCT_BATCH);
        let prepaid_with_the_product_batch = intent_product + units.div_ceil(PRODUCT_BATCH) * max_batch_product;
        let (_, _, _, _, consumed_with_the_test_batch) = of(units, test_batch);
        println!(
            "E name=batch_knob units={units} product_batch={PRODUCT_BATCH} test_batch={test_batch} prepaid_slots_with_the_product_batch={prepaid_with_the_product_batch} consumed_slots_with_the_test_batch={consumed_with_the_test_batch} short_slots={}",
            i128::from(consumed_with_the_test_batch) - i128::from(prepaid_with_the_product_batch)
        );
    }
}

/// V2：同一个文件（N = 145、K = 144：意图 + 两批），写完之后、删之前用户做过不同的事，删除各次发布新占的槽变不变。
/// 写入那一刻算的 W 是不是删除那一刻的上界，就看这里。
#[test]
fn w_depends_on_what_happened_between_the_write_and_the_delete() {
    let units = 145;
    let base = pool_with_a_file(units, DEFAULT_GEOMETRY);
    let parameters = parameters(DEFAULT_GEOMETRY);
    let mut states: Vec<(String, Pool)> = vec![("nothing".to_owned(), base.clone())];
    for leaves in [1u64, 4, 20, 60] {
        let mut pool = base.clone();
        for _ in 0..leaves {
            publish_on(&mut pool, |writer, allocator, previous, instance| {
                publish_new_inodes(writer, allocator, previous, 233, WRITE_TIME + 100, instance)
            })
            .expect("建 233 个 inode");
        }
        states.push((format!("new_inodes_{}", leaves * 233), pool));
    }
    let remounted_once = remounted(&base);
    states.push(("remount".to_owned(), remounted_once.clone()));
    let mut rewritten = remounted_once.clone();
    for round in 0..30u64 {
        let content = content_needing(units, 10 + round);
        publish_on(&mut rewritten, |writer, allocator, previous, instance| {
            publish_sequential_write(writer, allocator, previous, FirstFile { content: &content, write_time_seconds: WRITE_TIME + 200 + round }, instance)
        })
        .expect("重写整份");
    }
    states.push(("remount_then_30_rewrites".to_owned(), rewritten));
    let mut remounted_many = base.clone();
    for _ in 0..5 {
        remounted_many = remounted(&remounted_many);
    }
    states.push(("five_remounts".to_owned(), remounted_many));
    let _ = parameters;
    for (name, mut pool) in states {
        let published = delete_the_file(&mut pool, PRODUCT_BATCH, &mut |_, _, _| {}).expect("删");
        let new_slots: Vec<u64> = published.iter().map(|publish| publish.shape.new_slots).collect();
        let fixed: Vec<u64> = published.iter().map(|publish| publish.shape.fixed_point_slots).collect();
        let ordinary: Vec<u64> = published.iter().map(|publish| publish.shape.ordinary_slots).collect();
        let checkpoint_cost: Vec<u64> = published.iter().map(|publish| publish.shape.checkpoint_cost).collect();
        println!(
            "E name=w_state state={name} units={units} batch={PRODUCT_BATCH} new_slots={new_slots:?} ordinary_slots={ordinary:?} fixed_point_slots={fixed:?} checkpoint_cost={checkpoint_cost:?} sum_new_slots={}",
            new_slots.iter().sum::<u64>()
        );
    }
}

/// V2：建空 inode（`publish_new_inodes`）的发布不在正文点名的写入口里；而空 inode 照样能按号 unlink（意图 + 最后一批）。
/// 量：建 inode 那次今天新占多少；空 inode 意图那次的代用（把它的记录原样重写一遍、nlink = 0）新占多少；
/// 按「每个 inode 预付意图 + 一批」算，一块 4 GiB 盘的单元区装得下几个可删的空 inode（对照今天一个根装得下的 135 × 233 条）。
#[test]
fn deleting_an_empty_inode_needs_space_its_creation_never_prepaid() {
    let mut pool = pool_with_a_file(6, DEFAULT_GEOMETRY);
    let mut creations = Vec::new();
    for count in [1u64, 231, 1, 233] {
        let previous = pool.current.clone();
        let allocator_before = pool.allocator.clone();
        let output = publish_on(&mut pool, |writer, allocator, previous, instance| {
            publish_new_inodes(writer, allocator, previous, count, WRITE_TIME + 100, instance)
        })
        .expect("建 inode");
        let shape = shape_of(&output, &previous, &allocator_before);
        println!(
            "E name=create_inodes count={count} watermark_after={} leaves_rewritten={} {}",
            output.inode_number_watermark(),
            output.rewritten.iter().filter(|unit| matches!(unit, TransactionUnit::InodeLeafContainer(_))).count(),
            shape_line(&shape)
        );
        creations.push(shape);
    }
    // 空 inode 2 的「意图那次」与「最后一批」都用「把它那条记录重写一遍」代：意图那次写 nlink = 0；最后一批要从叶里去掉这条记录、
    // 同样重写这一片叶与根。两者都还缺 `deleted_inodes` 那棵树的节点与（意图那次）一个码 3 墓碑单元——那几格是推的，报告里另加。
    let mut proxies = Vec::new();
    for name in ["intent_proxy", "last_batch_proxy"] {
        set_nlink_zero_for(Some(2));
        let previous = pool.current.clone();
        let allocator_before = pool.allocator.clone();
        let output = publish_on(&mut pool, |writer, allocator, previous, instance| {
            publish_rewrite_inode_record_for_tests(writer, allocator, previous, 2, instance)
        })
        .expect("重写 inode 2 的记录");
        set_nlink_zero_for(None);
        let shape = shape_of(&output, &previous, &allocator_before);
        println!("E name=empty_inode_delete step={name} inode=2 {}", shape_line(&shape));
        proxies.push(shape);
    }
    let unit_area_slots = pool.allocator.devices[0].unit_area_slots();
    let per_inode_measured = proxies[0].new_slots + proxies[1].new_slots;
    let inode_cap_of_one_root: u64 = 135 * 233;
    println!(
        "E name=empty_inode_prepay unit_area_slots_per_device={unit_area_slots} prepay_slots_per_inode_measured_proxy={per_inode_measured} inodes_whose_deletion_fits_the_whole_unit_area={} inode_records_one_root_holds={inode_cap_of_one_root} create_233_new_slots={}",
        unit_area_slots / per_inode_measured,
        creations[3].new_slots
    );

    // 同一个池在「式子算出的可用 = 0」时（账：只有第一个文件的预付，空 inode 没有），空 inode 的意图那次走普通准入被拒。
    let mut full = pool.clone();
    set_ledger(Some((0, 0)));
    fill_until_available_is_at_most(&mut full, 0);
    set_nlink_zero_for(Some(3));
    let refused = publish_on(&mut full, |writer, allocator, previous, instance| {
        publish_rewrite_inode_record_for_tests(writer, allocator, previous, 3, instance)
    });
    set_nlink_zero_for(None);
    set_ledger(None);
    println!(
        "E name=empty_inode_delete_on_a_full_pool admission=ordinary result={:?} available_before={:?}",
        refused.as_ref().map(|output| output.root.checkpoint_txg.0),
        {
            set_ledger(Some((0, 0)));
            let value = available(&full);
            set_ledger(None);
            value
        }
    );
    assert!(matches!(refused, Err(PublishError::SpaceAdmissionRefused(_))), "满池上空 inode 的意图走普通准入被拒");
}

/// 逐盘「借了式子保留量多少槽」：保留量（挂载期承诺量 + checkpoint 保留池，账为 0 时）减空闲槽，正数是删除吃进了保留量。
fn borrowed_slots(pool: &Pool) -> i128 {
    let reserve = reserve_slots(pool)[0];
    reserve - i128::from(pool.allocator.devices[0].free_slots())
}

/// W 的探针：同一个池拷一份，在拷贝上按 `batch` 删一遍，交回（意图那次新占、各批新占的最大值）。
fn probe_w(pool: &Pool, batch: u64) -> (u64, u64) {
    let mut copy = pool.clone();
    let saved = LEDGER_FOR_TESTS.with(std::cell::Cell::get);
    set_ledger(None);
    let published = delete_the_file(&mut copy, batch, &mut |_, _, _| {}).expect("探针");
    set_ledger(saved);
    let intent = published[0].shape.new_slots;
    let max_batch = published[1..].iter().map(|publish| publish.shape.new_slots).max().expect("至少一批");
    (intent, max_batch)
}

fn bytes_of_slots_on_every_device(slots: u64) -> u64 {
    slots * SLOT_BYTES * DEVICES
}

/// V2：可写挂载之后（分配器按根环回收），式子算出的可用 = 0 的池上一条很长的删除（K = 1）。删除各批不走式子那一判（R3），
/// 账里一个字节都不给它（待删占用 = 0）：它借了保留量多少槽、借到第几次发布就不再往上涨、最后剩多少，与文件多大有没有关系。
#[test]
fn a_long_delete_borrows_at_most_a_ring_window_of_slots() {
    for (ring, units) in [(8u64, 100u64), (8, 200), (16, 200)] {
        let geometry = Geometry { image_bytes: DEFAULT_GEOMETRY.image_bytes, ring_slots_per_region: ring };
        let mut pool = remounted(&pool_with_a_file(units, geometry));
        let (intent_w, batch_w) = probe_w(&pool, 1);
        set_ledger(Some((0, 0)));
        fill_until_available_is_at_most(&mut pool, 0);
        let borrowed_before = borrowed_slots(&pool);
        set_r3_batch_switches(true, false);
        let mut borrowed: Vec<i128> = Vec::new();
        let mut deferred: Vec<u64> = Vec::new();
        let result = delete_the_file(&mut pool, 1, &mut |_, _, pool| {
            borrowed.push(borrowed_slots(pool));
            deferred.push(pool.allocator.devices[0].deferred_slots());
        });
        borrowed.push(borrowed_slots(&pool));
        deferred.push(pool.allocator.devices[0].deferred_slots());
        set_r3_batch_switches(false, false);
        set_ledger(None);
        let (max_index, max_borrowed) = borrowed
            .iter()
            .enumerate()
            .max_by_key(|(_, value)| **value)
            .map(|(index, value)| (index, *value))
            .expect("有");
        let sampled: Vec<String> = borrowed
            .iter()
            .enumerate()
            .filter(|(index, _)| index % 10 == 0 || *index + 1 == borrowed.len())
            .map(|(index, value)| format!("{index}:{value}"))
            .collect();
        println!(
            "E name=ring_window ring_slots_per_region={ring} ring_roots={} units={units} batch=1 publishes={} result={} borrowed_before={borrowed_before} max_borrowed_slots={max_borrowed} at_publish={max_index} final_borrowed_slots={} max_deferred_slots={} intent_w={intent_w} batch_w={batch_w} ring_roots_times_batch_w={} r3_per_file_prepay_slots={} reserve_slots_dev0={}",
            3 * ring,
            borrowed.len() - 1,
            match &result { Ok(_) => "Ok".to_owned(), Err((name, error)) => format!("Err({name}:{error:?})") },
            borrowed.last().expect("有"),
            deferred.iter().max().expect("有"),
            3 * ring * batch_w,
            intent_w + units * batch_w,
            reserve_slots(&pool)[0]
        );
        println!("E name=ring_window_series ring_slots_per_region={ring} units={units} borrowed_every_10th={sampled:?}");
        assert!(result.is_ok(), "删得完");
    }
}

/// V4 / V2：R3 的账按 R_old 重算时，管理员回退到一个更大的文件版本把「已承诺预留」抬上去，而回退那次发布不判准入（它没有普通分配）。
/// 历史：挂载之后顺序写成 300 个单元（A）→ 覆盖写成 1 个单元（C）→ 把池填到式子算出的可用 = 0 → 回退到 A → 删 A（K = 144，R3：各批不走式子那一判）。
#[test]
fn rollback_to_a_larger_version_raises_the_reservation_without_admission() {
    let geometry = DEFAULT_GEOMETRY;
    let mut pool = remounted(&pool_with_a_file(1, geometry));
    let (intent_w, batch_w) = probe_w(&pool_with_a_file(300, geometry), PRODUCT_BATCH);
    let prepay = |units: u64| bytes_of_slots_on_every_device(intent_w + units.div_ceil(PRODUCT_BATCH).max(1) * batch_w);
    let big = 300u64;
    set_ledger(Some((0, prepay(big))));
    let content = content_needing(big, 3);
    let version_a = publish_on(&mut pool, |writer, allocator, previous, instance| {
        publish_sequential_write(writer, allocator, previous, FirstFile { content: &content, write_time_seconds: WRITE_TIME + 10 }, instance)
    })
    .expect("A：300 个单元");
    set_ledger(Some((0, prepay(1))));
    let small = content_needing(1, 4);
    publish_on(&mut pool, |writer, allocator, previous, instance| {
        publish_overwrite(writer, allocator, previous, FirstFile { content: &small, write_time_seconds: WRITE_TIME + 20 }, instance)
    })
    .expect("C：1 个单元");
    fill_until_available_is_at_most(&mut pool, 0);
    let available_at_c = available(&pool);
    let mut probe = pool.clone();
    let one_more_inode = publish_on(&mut probe, |writer, allocator, previous, instance| {
        publish_new_inodes(writer, allocator, previous, 1, WRITE_TIME + 30, instance)
    });
    // 回退若照「增加空间占用的操作」判：需求 = 预付之差摊到每块盘。
    let raise = prepay(big) - prepay(1);
    let demand: Vec<DemandOnDevice> = pool
        .allocator
        .devices
        .iter()
        .map(|map| DemandOnDevice { device: map.device, bytes: BytesOnOneDevice(raise / DEVICES) })
        .collect();
    let judged = admit_on_every_device(&admission_reading_before_a_publish(&pool.allocator, Some(&pool.current)), &demand);
    set_ledger(Some((0, prepay(big))));
    let parameters = parameters(geometry);
    let rolled = roll_back_by_a_forward_publish(
        &parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        RollbackTarget { instance: version_a.root.instance, checkpoint_txg: CheckpointTxg(version_a.root.checkpoint_txg.0) },
    );
    let available_after = available(&pool);
    println!(
        "E name=rollback_raise intent_w={intent_w} batch_w={batch_w} prepay_a={} prepay_c={} raise_bytes_all_replicas={raise} available_at_c={available_at_c:?} new_inode_at_c={} rollback_if_judged={} rollback={} txg_after={} units_after={} available_after={available_after:?} committed_row_after={} {}",
        prepay(big),
        prepay(1),
        match &one_more_inode { Ok(_) => "Ok".to_owned(), Err(error) => format!("Err({})", short_error(error)) },
        match &judged { Ok(()) => "Ok".to_owned(), Err(refusal) => format!("Refused({:?})", refusal.short_devices.iter().map(|short| (short.available.0, short.demand.0)).collect::<Vec<_>>()) },
        match &rolled { Ok(_) => "Ok".to_owned(), Err(error) => format!("Err({error:?})") },
        pool.current.root.checkpoint_txg.0,
        pool.current.data_pointers.len(),
        accounting_row(&pool.current, singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES),
        per_device_counts(&pool.allocator)
    );
    assert!(rolled.is_ok(), "回退做成");
    assert!(available_after.iter().all(|value| *value < 0), "回退之后式子算出的可用是负的");

    // 删 A：账照 R3 搬（意图那次已承诺预留转成待删占用、各批递减），各批不走式子那一判、落点那一判照判。
    let batches = big.div_ceil(PRODUCT_BATCH);
    let mut borrowed: Vec<i128> = vec![borrowed_slots(&pool)];
    set_r3_batch_switches(true, false);
    let result = delete_the_file(&mut pool, PRODUCT_BATCH, &mut |name, keep, pool| {
        let left = if name == "intent" { batches } else { keep.div_ceil(PRODUCT_BATCH) };
        set_ledger(Some((bytes_of_slots_on_every_device(left * batch_w), 0)));
        borrowed.push(borrowed_slots(pool));
    });
    borrowed.push(borrowed_slots(&pool));
    set_r3_batch_switches(false, false);
    println!(
        "E name=rollback_then_delete result={} borrowed_slots_dev0={borrowed:?} available_after_the_delete={:?} {}",
        match &result { Ok(published) => format!("Ok(publishes={})", published.len()), Err((name, error)) => format!("Err({name}:{})", short_error(error)) },
        available(&pool),
        per_device_counts(&pool.allocator)
    );
    set_ledger(None);
}

fn short_error(error: &PublishError) -> String {
    match error {
        PublishError::SpaceAdmissionRefused(refusal) => format!(
            "SpaceAdmissionRefused{:?}",
            refusal.short_devices.iter().map(|short| (short.available.0, short.demand.0)).collect::<Vec<_>>()
        ),
        other => format!("{other:?}"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LedgerRule {
    /// 待删占用 = 剩下几批 × W（从镜像上的单元数现算）。
    Recomputed,
    /// 每一批减两次 W（饱和到 0），最后一批清零。
    SubtractedTwice,
    /// 意图那次记下之后不动，最后一批清零。
    NeverSubtracted,
}

/// V1：第一轮换上的三条判据（① 计划相等 ② 删完再续做一个写都不发 ③ 每个写点截断重开续做到底、终态逻辑内容相同、checker 全绿）
/// 判不判得出「待删占用减两次 / 不减」这种非幂等的续做。崩溃点只取「某一批整个落盘之后」那一格（发布边界），批内的写点没枚举
/// （一批的两态闭式 1.7 × 10⁷，超过一段约 10⁶ 的线）。
#[test]
fn three_continuation_criteria_pass_a_continuation_that_mishandles_the_pending_delete_row() {
    let geometry = DEFAULT_GEOMETRY;
    let base = remounted(&pool_with_a_file(3, geometry));
    let (_, batch_w) = probe_w(&base, 1);
    let w_bytes = bytes_of_slots_on_every_device(batch_w);
    let pending_statistic = singlefs_core::records::STATISTIC_PENDING_DELETE_BYTES;
    for rule in [LedgerRule::Recomputed, LedgerRule::SubtractedTwice, LedgerRule::NeverSubtracted] {
        let next_pending = |name: &str, keep: u64, before: u64| -> u64 {
            if name == "intent" {
                return keep * w_bytes;
            }
            match rule {
                LedgerRule::Recomputed => keep * w_bytes,
                _ if keep == 0 => 0,
                LedgerRule::SubtractedTwice => before.saturating_sub(2 * w_bytes),
                LedgerRule::NeverSubtracted => before,
            }
        };
        let pending_of = |pool: &Pool| accounting_row(&pool.current, pending_statistic);
        // 路径一：不崩溃，意图 + 三批。
        let mut straight = base.clone();
        set_ledger(Some((0, 0)));
        let mut rows_straight: Vec<(u64, u64)> = Vec::new();
        let mut images_straight: Vec<MemoryPool> = Vec::new();
        let mut snapshots: Vec<Pool> = Vec::new();
        let published = delete_the_file(&mut straight, 1, &mut |name, keep, pool| {
            set_ledger(Some((next_pending(name, keep, pending_of(pool)), 0)));
            snapshots.push(pool.clone());
        })
        .expect("删");
        snapshots.push(straight.clone());
        for publish in &published {
            rows_straight.push((accounting_row(&publish.output, pending_statistic), publish.keep * w_bytes));
        }
        for snapshot in &snapshots[1..] {
            images_straight.push(snapshot.image());
        }
        // 路径二：第一批整个落盘之后崩溃，重开可写挂载（新实例、写行、暖机；这几次发布照抄镜像上的账），续做到底。
        let after_the_first_batch = &snapshots[2];
        let image_rows = (
            accounting_row(&after_the_first_batch.current, pending_statistic),
            accounting_row(&after_the_first_batch.current, singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES),
        );
        set_ledger(Some(image_rows));
        let mut crashed = remounted(after_the_first_batch);
        let plan_once = plan_of_the_continuation(&crashed.current, 1);
        let plan_twice = plan_of_the_continuation(&crashed.current, 1);
        let mut rows_crashed: Vec<(u64, u64)> = Vec::new();
        let mut images_crashed: Vec<MemoryPool> = vec![crashed.image()];
        let continued = continue_the_delete(&mut crashed, 1, &mut |name, keep, pool| {
            set_ledger(Some((next_pending(name, keep, pending_of(pool)), 0)));
        })
        .expect("续做");
        for publish in &continued {
            rows_crashed.push((accounting_row(&publish.output, pending_statistic), publish.keep * w_bytes));
        }
        images_crashed.push(crashed.image());
        let plan_after_done = plan_of_the_continuation(&crashed.current, 1);
        set_ledger(None);
        let logical = |pool: &Pool| {
            (
                pool.current.data_pointers.len(),
                pool.current.inode_record.size,
                raw_nlink_of_inode_one(&pool.current),
                accounting_row(&pool.current, pending_statistic),
                accounting_row(&pool.current, singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES),
            )
        };
        let reds: usize = images_straight.iter().chain(&images_crashed).map(|image| checker_red(image).len()).sum();
        let criterion_one = plan_once == plan_twice;
        let criterion_two = plan_after_done.is_empty();
        let criterion_three = logical(&straight) == logical(&crashed) && reds == 0;
        let mismatches = rows_straight.iter().chain(&rows_crashed).filter(|(row, recomputed)| row != recomputed).count();
        println!(
            "E name=v1_criteria rule={rule:?} batch_w={batch_w} criterion_1_plan_equal={criterion_one} criterion_2_done_writes_nothing={criterion_two} criterion_3_final_equal_and_checker_green={criterion_three} final_straight={:?} final_crashed={:?} checker_red_total={reds} pending_row_vs_recomputed_straight={rows_straight:?} pending_row_vs_recomputed_after_crash={rows_crashed:?} rows_not_equal_to_the_recompute={mismatches}",
            logical(&straight),
            logical(&crashed)
        );
        assert!(criterion_one && criterion_two && criterion_three, "三条判据都过：{rule:?}");
        assert_eq!(mismatches == 0, rule == LedgerRule::Recomputed, "只有现算那一种每一版的待删占用都对得上镜像");
    }
}

/// V6：删除中（意图之后留 4）的一版上，哪些发布会重写 inode 1 那条记录所在的叶——重写时记录从内存里的 `InodeRecord` 重新序列化，
/// nlink 不是它的字段（这里把原型的 nlink 开关关掉来模拟「不带 nlink 的写者」），盘上的 nlink 就回到 1。
#[test]
fn which_publishes_rewrite_the_record_of_a_file_being_deleted() {
    let geometry = DEFAULT_GEOMETRY;
    let mut pool = remounted(&pool_with_a_file(6, geometry));
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    set_nlink_zero_for(Some(1));
    let size_before = pool.current.inode_record.size;
    let intent = unlink_batch(&mut pool, 6, size_before).expect("意图");
    unlink_batch(&mut pool, 4, 4 * payload).expect("留 4");
    let mid = pool.clone();
    set_nlink_zero_for(None);
    println!("E name=v6 op=state_mid_delete raw_nlink={}", raw_nlink_of_inode_one(&mid.current));

    let mut devices = devices_of(&mid.image(), geometry);
    let mounted = mount_writable(&parameters(geometry), &mut devices).expect("可写挂载");
    let row_rewrites = match &mounted.output.row_publish { PoolVersion::WithFile(version) => rewrites_an_inode_leaf(version), PoolVersion::WithoutFile(_) => false };
    let warm_rewrites = mounted.output.warm_up_publishes.iter().any(|version| match version { PoolVersion::WithFile(version) => rewrites_an_inode_leaf(version), PoolVersion::WithoutFile(_) => false });
    let PoolVersion::WithFile(after_mount) = &mounted.current else { panic!("带文件") };
    println!("E name=v6 op=writable_mount row_publish_rewrites_inode_leaf={row_rewrites} warm_up_rewrites_inode_leaf={warm_rewrites} raw_nlink_after={}", raw_nlink_of_inode_one(after_mount));

    let mut with_inode = mid.clone();
    let created = publish_on(&mut with_inode, |writer, allocator, previous, instance| {
        publish_new_inodes(writer, allocator, previous, 1, WRITE_TIME + 50, instance)
    })
    .expect("建一个 inode");
    println!("E name=v6 op=publish_new_inodes_1 rewrites_inode_leaf={} raw_nlink_after={} units_after={}", rewrites_an_inode_leaf(&created), raw_nlink_of_inode_one(&created), created.data_pointers.len());

    let mut written = mid.clone();
    let content = content_needing(2, 9);
    let rewritten = publish_on(&mut written, |writer, allocator, previous, instance| {
        publish_sequential_write(writer, allocator, previous, FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 }, instance)
    });
    println!(
        "E name=v6 op=sequential_write_mid_delete result={} raw_nlink_after={} object_birth_kept={}",
        match &rewritten { Ok(output) => format!("Ok(units={})", output.data_pointers.len()), Err(error) => format!("Err({error:?})") },
        rewritten.as_ref().map_or(99, raw_nlink_of_inode_one),
        rewritten.as_ref().is_ok_and(|output| output.inode_record.object_birth == mid.current.inode_record.object_birth)
    );

    let mut rolled = mid.clone();
    unlink_batch(&mut rolled, 2, 2 * payload).expect("留 2（开关关着）");
    let rollback = roll_back_by_a_forward_publish(
        &parameters(geometry),
        &mut rolled.devices,
        &mut rolled.allocator,
        &mut rolled.current,
        RollbackTarget { instance: intent.root.instance, checkpoint_txg: intent.root.checkpoint_txg },
    );
    println!(
        "E name=v6 op=rollback_to_the_intent result={} rewrites_inode_leaf={} raw_nlink_after={}",
        rollback.is_ok(),
        rewrites_an_inode_leaf(&rolled.current),
        raw_nlink_of_inode_one(&rolled.current)
    );
}

/// V1 那一格没枚举批内崩溃态的理由：同一段删除（N = 3、K = 1：意图 + 三批）每次发布的写表按屏障 / FUA 切段之后的两态闭式
/// （1 + Σ(2^|段| − 1)，不含原地覆写的第三态；`memory_pool::closed_form_state_count`），只算不枚举。
#[test]
fn two_state_closed_form_of_the_three_unit_delete() {
    use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments};
    use singlefs_harness::segments::FixedGeometry;
    use singlefs_harness::{RecordingBlockDevice, SharedStream};
    let geometry = DEFAULT_GEOMETRY;
    let pool = pool_with_a_file(3, geometry);
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = pool
        .devices
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let fixed = FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    };
    let parameters = parameters(geometry);
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let mut allocator = pool.allocator;
    let mut previous = pool.current;
    set_nlink_zero_for(Some(1));
    let mut total = 0u64;
    for (name, keep, size) in [("intent", 3u64, previous.inode_record.size), ("batch_1", 2, 2 * payload), ("batch_2", 1, payload), ("batch_3", 0, 0)] {
        let before = stream.retained_operations().len();
        let next = {
            let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
            publish_unlink_batch_for_tests(&mut writer, &mut allocator, &previous, keep, size, pool.instance).expect("一批")
        };
        let operations = stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations[before..], &fixed);
        let states = closed_form_state_count(&segments);
        total += states - 1;
        println!(
            "E name=closed_form publish={name} txg={} writes={} segment_sizes={:?} two_state_closed_form={states}",
            next.root.checkpoint_txg.0,
            writes.len(),
            segments.iter().map(Vec::len).collect::<Vec<_>>()
        );
        previous = next;
    }
    set_nlink_zero_for(None);
    println!("E name=closed_form_sum publishes=4 sum_of_states_minus_one_each={total}");
}
