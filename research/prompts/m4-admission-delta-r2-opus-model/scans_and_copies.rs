//! 设计轮 m4-admission-delta-r2 攻方腿的模型（攻方副本用，不进仓的 crates/）。`tools/rerun.sh` 把它拷进冻结副本的
//! `crates/singlefs-harness/tests/m4_admission_delta_r2_attack_scans.rs`，在今天的代码上量第二轮正文第一轮没看过的几处：
//! - 落点那一判的参数里现算 ckpt_cost 时，按角色在 `units` 上线性找的根头（`transaction.rs` 的 `TransactionOutput::unit`）排在这一版
//!   每个数据单元后面；
//! - 墓碑每 unlink 一次多一条、第一版不回收，之后每次发布在落点那一判之前（`ordinary_allocation_part`）整份深拷一遍；
//! - 用户数据取落点从单元区起点逐对往上找，每个候选再把这次挂载开过的聚簇段逐个比一遍（`allocator.rs` 的
//!   `lowest_user_data_slot_outside_the_region`），固定点每一轮的预演都走它；
//! - 固定点每一轮整份拷的分配器（`settle_the_allocation_record_tree` 的 `allocator.clone()`）随单元区槽数涨。
//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`）。挂钟是 debug 构建、同一个测试二进制里几条用例可能并行时量的，
//! 只看同一组行里随参数怎么变，不看绝对值。没有崩溃点：这几条只数今天代码走过的东西，不判任何一版落盘之后读回什么。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use std::time::{Duration, Instant};

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, InodeNumber};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::e142_parameters;

/// 6 MiB 的 journal 环：一次发布的记录上限 1536 ÷ 3 = 512 条，单元区从槽 1408 起（`common_admission_pool` 那一档小盘用同一个环）。
const JOURNAL_RING_OF_SIX_MEBIBYTES: u64 = 6 << 20;
/// 小池的单元区槽数（256 MiB）：装得下几次 400 个数据单元的顺序写、几百次覆盖写。
const UNIT_AREA_SLOTS_OF_THE_SMALL_POOL: u64 = 16_384;
/// 一次顺序写至多切几个数据单元：比 512 条的记录上限留出余量。
const DATA_UNITS_OF_ONE_SEQUENTIAL_WRITE_AT_MOST: u64 = 400;
/// 每个挂钟数取几次的中位数。
const TIMING_SAMPLES: usize = 9;

fn parameters_on_the_six_mebibyte_ring() -> MakeFilesystemParameters {
    let mut parameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_OF_SIX_MEBIBYTES;
    parameters
}

fn unit_area_start_slot() -> u64 {
    UnitAreaStart::following_the_journal_ring(JOURNAL_RING_OF_SIX_MEBIBYTES)
        .expect("6 MiB 的环末尾的下一个槽 1408 = 22 × 64，落在聚簇段边界上")
        .slot()
        .0
}

/// mkfs、取号、暖机、第一个文件（3000 字节，一个数据单元）之后的池，两块盘的单元区各 `unit_area_slots` 槽。
fn pool_with_a_unit_area_of(unit_area_slots: u64) -> PoolUnderTest<SparseBlockDevice> {
    PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_on_the_six_mebibyte_ring(),
        (unit_area_start_slot() + unit_area_slots) * SLOT_BYTES,
        plain_device,
    )
}

/// 从文件偏移 0 顺序写这么多字节切成 `data_units` 个数据单元。
fn content_bytes_of_a_sequential_write_into(data_units: u64) -> usize {
    let payload_bytes =
        u64::try_from(data_unit_payload_capacity()).expect("一个数据单元的净荷装得进 u64");
    usize::try_from((data_units - 1) * payload_bytes + 1).expect("几兆字节装得进 usize")
}

/// 这一版 `units` 里角色是 `role` 的头一项的下标：`TransactionOutput::unit` 线性找到它之前走过的项数。
fn position_of_the_role(version: &TransactionOutput, role: TransactionUnit) -> usize {
    version
        .units
        .iter()
        .position(|unit| unit.identity == role)
        .expect("带文件的一版有这个根单元")
}

fn tombstone_records(version: &TransactionOutput) -> usize {
    version
        .tombstone_containers
        .iter()
        .map(|container| container.contents.records.len())
        .sum()
}

fn median_nanoseconds(mut samples: Vec<Duration>) -> u128 {
    samples.sort();
    samples[samples.len() / 2].as_nanos()
}

/// 建一个空 inode（号取这一版的水位：新号从水位起发），交回它的号。
fn create_an_inode(pool: &mut PoolUnderTest<SparseBlockDevice>) -> InodeNumber {
    let inode = InodeNumber(pool.current_file_version().inode_number_watermark());
    pool.create_one_inode().expect("建一个 inode 发成");
    inode
}

/// 给 `inode` 顺序写 `data_units` 个数据单元（一次发布）。
fn write_into_the_inode(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    inode: InodeNumber,
    data_units: u64,
) {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let content = content_of(
        content_bytes_of_a_sequential_write_into(data_units),
        write_time_seconds,
    );
    pool.publish(
        UserChange::SequentialWriteOfAnInode {
            inode,
            content: &content,
            write_time_seconds,
        },
        None,
    )
    .expect("按 inode 号顺序写发成");
}

/// 删 `inode`：意图那次发布追加一条墓碑，会话接着把队列走空。
fn unlink(pool: &mut PoolUnderTest<SparseBlockDevice>, inode: InodeNumber) {
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(
        UserChange::Unlink {
            inode,
            write_time_seconds,
        },
        None,
    )
    .expect("unlink 发成");
}

/// 在会话分配器的拷贝上取一次用户数据落点要多久（只量取落点那一下，不量拷贝），取 9 次的中位数。
fn one_user_data_placement_nanoseconds(pool: &PoolUnderTest<SparseBlockDevice>) -> u128 {
    let generation = CheckpointTxg(pool.session().current.root().checkpoint_txg.0 + 1);
    let samples: Vec<Duration> = (0..TIMING_SAMPLES)
        .map(|_| {
            let mut copy = pool.session().allocator.clone();
            let started = Instant::now();
            let placement = copy.try_allocate_user_data(generation);
            let elapsed = started.elapsed();
            std::hint::black_box(&placement);
            assert!(placement.is_ok(), "小池上还有成对的空槽");
            elapsed
        })
        .collect();
    median_nanoseconds(samples)
}

/// 落点那一判的参数里现算 ckpt_cost（`admission::checkpoint_cost_of_the_version_to_build_on_with_node_capacities`），读分配记录树、
/// extent 上段、中央映射树的高时各按角色在 `units` 上线性找一次根单元（`TransactionOutput::unit`）；式子那一判的读数再算一次。
/// `units` 按 bump 次序装：每个数据单元在最前，固定点那几族在后。所以每找一次根头要先走过这一版的全部数据单元。
/// 每个 N 一个新池，第一个文件之后顺序写 N 个数据单元，量那几个根头在 `units` 里的下标。
#[test]
fn the_root_heads_the_checkpoint_cost_walks_to_sit_behind_every_data_unit_of_the_version() {
    let mut allocation_tree_positions: Vec<usize> = Vec::new();
    for data_units in [1_u64, 16, 128, DATA_UNITS_OF_ONE_SEQUENTIAL_WRITE_AT_MOST] {
        let mut pool = pool_with_a_unit_area_of(UNIT_AREA_SLOTS_OF_THE_SMALL_POOL);
        pool.sequential_write(content_bytes_of_a_sequential_write_into(data_units))
            .expect("顺序写发成");
        let version = pool.current_file_version();
        let allocation_tree_at = position_of_the_role(version, TransactionUnit::AllocationTree);
        let mapping_tree_at = position_of_the_role(version, TransactionUnit::MappingTree);
        let accounting_tree_at = position_of_the_role(version, TransactionUnit::AccountingTree);
        let extent_root_at = position_of_the_role(version, TransactionUnit::ExtentRoot);
        println!(
            "name=root_heads_behind_the_data_units data_units_written={data_units} data_units_in_the_version={} units={} \
             extent_root_at={extent_root_at} allocation_tree_at={allocation_tree_at} accounting_tree_at={accounting_tree_at} \
             mapping_tree_at={mapping_tree_at}",
            version.data_pointers.len(),
            version.units.len()
        );
        assert!(
            allocation_tree_at >= version.data_pointers.len(),
            "分配记录树的根排在这一版每个数据单元之后"
        );
        allocation_tree_positions.push(allocation_tree_at);
    }
    assert!(
        allocation_tree_positions.windows(2).all(|pair| pair[0] < pair[1]),
        "根头的下标随数据单元数涨：{allocation_tree_positions:?}"
    );
}

/// 每 unlink 一次，意图那次发布往墓碑树追加一条（第一版不回收）；之后任何一次不是 unlink 意图的发布在落点那一判之前
/// （`PublishPlan::ordinary_allocation_part`，固定点每一轮的 `resolve` 里再一次）把全部墓碑容器的内容逐片 `clone` 一份。
/// 这里量墓碑条数随 unlink 次数怎么涨，与一次覆盖写之后的一版是不是照样装着全部墓碑（照抄的那一份就是整份）。
#[test]
fn every_tombstone_ever_written_is_copied_again_by_every_later_publish() {
    let mut pool = pool_with_a_unit_area_of(UNIT_AREA_SLOTS_OF_THE_SMALL_POOL);
    let mut unlinked: u64 = 0;
    let mut tombstones_at_each_checkpoint: Vec<usize> = Vec::new();
    for unlinked_at_this_checkpoint in [1_u64, 8, 32, 96] {
        while unlinked < unlinked_at_this_checkpoint {
            let inode = create_an_inode(&mut pool);
            unlink(&mut pool, inode);
            unlinked += 1;
        }
        let tombstones_before = tombstone_records(pool.current_file_version());
        let containers_before = pool.current_file_version().tombstone_containers.len();
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
        let tombstones_after = tombstone_records(pool.current_file_version());
        println!(
            "name=tombstones_every_publish_copies unlinked={unlinked} tombstone_records_before_an_overwrite={tombstones_before} \
             tombstone_containers_before_the_overwrite={containers_before} tombstone_records_carried_by_the_overwrite={tombstones_after}"
        );
        assert_eq!(
            tombstones_after, tombstones_before,
            "覆盖写不是 unlink 意图：墓碑树整份照抄"
        );
        tombstones_at_each_checkpoint.push(tombstones_before);
    }
    assert!(
        tombstones_at_each_checkpoint.windows(2).all(|pair| pair[0] < pair[1]),
        "墓碑条数随 unlink 次数涨：{tombstones_at_each_checkpoint:?}"
    );
}

/// 用户数据取落点（`PoolAllocator::try_allocate_user_data`）每块盘从单元区起点按两槽一步往上找第一对两槽都空、不在任何聚簇段里的槽，
/// 每个候选再把这次挂载开过的聚簇段（`PoolAllocator::cluster_segments`，挂载里只增不减）逐个比一遍。固定点每一轮的预演里每个数据角色走它一次，
/// 落点那一判之后、式子那一判之前。两组：别的 inode 里先写 B 个数据单元（压在单元区底下、一直活着），量下一次取落点要走过几个候选；
/// 同一次挂载里连发 M 次覆盖写，量聚簇段数怎么涨。两组都另量一次取落点的挂钟。
#[test]
fn a_user_data_placement_walks_every_occupied_pair_below_it_and_every_cluster_segment_opened_in_the_mount() {
    let mut candidates_at_each_size: Vec<u64> = Vec::new();
    for data_units_kept_below in [0_u64, 128, DATA_UNITS_OF_ONE_SEQUENTIAL_WRITE_AT_MOST, 1_200] {
        let mut pool = pool_with_a_unit_area_of(UNIT_AREA_SLOTS_OF_THE_SMALL_POOL);
        let mut written: u64 = 0;
        while written < data_units_kept_below {
            let chunk = (data_units_kept_below - written).min(DATA_UNITS_OF_ONE_SEQUENTIAL_WRITE_AT_MOST);
            let inode = create_an_inode(&mut pool);
            write_into_the_inode(&mut pool, inode, chunk);
            written += chunk;
        }
        let allocator = &pool.session().allocator;
        let device = &allocator.devices[0];
        let start = device.unit_area_start().slot().0;
        let lowest = device
            .lowest_user_data_slot(allocator.cluster_segments())
            .expect("小池上还有成对的空槽")
            .0;
        let candidates_walked = (lowest - start) / 2;
        let cluster_segments = u64::try_from(allocator.cluster_segments().len()).expect("段数");
        println!(
            "name=lowest_free_pair_search data_units_kept_below={data_units_kept_below} allocation_records={} \
             candidates_walked_before_the_free_pair={candidates_walked} cluster_segments_opened_in_the_mount={cluster_segments} \
             segment_comparisons_at_most={} one_user_data_placement_ns_median={}",
            allocator.records().len(),
            candidates_walked * cluster_segments,
            one_user_data_placement_nanoseconds(&pool)
        );
        candidates_at_each_size.push(candidates_walked);
    }
    assert!(
        candidates_at_each_size.windows(2).all(|pair| pair[0] < pair[1]),
        "取落点要走过的候选随压在底下的数据单元数涨：{candidates_at_each_size:?}"
    );
    let mut pool = pool_with_a_unit_area_of(UNIT_AREA_SLOTS_OF_THE_SMALL_POOL);
    let mut overwrites: u64 = 0;
    for overwrites_at_this_checkpoint in [8_u64, 64, 256] {
        while overwrites < overwrites_at_this_checkpoint {
            pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
            overwrites += 1;
        }
        let allocator = &pool.session().allocator;
        println!(
            "name=cluster_segments_in_one_mount overwrites_in_the_mount={overwrites} cluster_segments_opened_in_the_mount={} \
             allocation_records={} one_user_data_placement_ns_median={}",
            allocator.cluster_segments().len(),
            allocator.records().len(),
            one_user_data_placement_nanoseconds(&pool)
        );
    }
}

/// 固定点每一轮从 `allocator.clone()` 起（`settle_the_allocation_record_tree`，产品路径三四轮），分配器里每块盘按槽的三张位图
/// （已分配、影子账隔离、抬 F 扣住）与按段的五张计数表都随单元区槽数涨，记录表随分配过的单元涨。这里在单元区 2^14、2^17、2^20、2^24 槽
/// （每块盘 256 MiB、2 GiB、16 GiB、256 GiB）四个池上，第一个文件之后不写别的，量拷一份分配器要多久，与一次一个单元的覆盖写整次发布要多久。
#[test]
fn the_allocator_copied_on_every_settling_round_grows_with_the_unit_area() {
    let mut clone_nanoseconds_at_each_size: Vec<u128> = Vec::new();
    for exponent in [14_u32, 17, 20, 24] {
        let unit_area_slots = 1_u64 << exponent;
        let mut pool = pool_with_a_unit_area_of(unit_area_slots);
        let clone_samples: Vec<Duration> = (0..TIMING_SAMPLES)
            .map(|_| {
                let started = Instant::now();
                let copy = pool.session().allocator.clone();
                let elapsed = started.elapsed();
                std::hint::black_box(&copy);
                drop(copy);
                elapsed
            })
            .collect();
        let clone_nanoseconds = median_nanoseconds(clone_samples);
        let overwrite_samples: Vec<Duration> = (0..3)
            .map(|_| {
                let started = Instant::now();
                pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
                started.elapsed()
            })
            .collect();
        println!(
            "name=allocator_copy_by_unit_area unit_area_slots={unit_area_slots} unit_area_gib_per_device={} \
             allocation_records={} allocator_clone_ns_median={clone_nanoseconds} one_unit_overwrite_publish_ns_median={}",
            (unit_area_slots * SLOT_BYTES) >> 30,
            pool.session().allocator.records().len(),
            median_nanoseconds(overwrite_samples)
        );
        clone_nanoseconds_at_each_size.push(clone_nanoseconds);
    }
    assert!(
        clone_nanoseconds_at_each_size[3] > clone_nanoseconds_at_each_size[0],
        "单元区大 1024 倍，拷一份分配器至少要更久：{clone_nanoseconds_at_each_size:?}"
    );
}
