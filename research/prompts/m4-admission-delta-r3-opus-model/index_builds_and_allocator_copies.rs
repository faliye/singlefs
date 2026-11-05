//! 设计轮 m4-admission-delta-r3 攻方腿的模型（攻方副本用，不进仓的 crates/）。`tools/rerun.sh` 先用 `tools/apply_copy_counters.py` 给副本的
//! 发布路径套上三个只读出的计数（不改任何判定），再把这一份拷成 `crates/singlefs-harness/tests/m4_admission_delta_r3_attack_counts.rs` 跑。
//! 数的是今天的代码在一次发布两道准入都判完之前做的三样，它们都不在 B6、B7 的定义里：
//! - 释放判定（`placements_released_by_this_publish`，固定点每一轮调一次）按角色查上一版映射条目时，第一次查就把上一版整张 `mapped_units`
//!   与整张 `units` 各建一份按身份的索引（`MappedUnitsOfAVersionReadByRole`、`CentralMappingOfAVersionReadLeafByLeaf`）：用 harness 档已开的
//!   `lookup-work-counters` 逐处数建了几次、查过的索引最大几项；
//! - `publish_version_of_trees_relocating_units_after_a_check` 在两道准入之前整份拷一次分配器（失败就换回去的那一份），固定点每一轮再拷一次：
//!   用套上的计数数每次用户改动拷了几次、判了几次；
//! - 建 inode 的发布之后那一版的已承诺预留（标签 6，式子那一判读这次之后的值）比之前那一版多多少：上界快路若取上一版记下的那一份，差的就是它。
//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`）。没有崩溃点：只数今天代码走过的东西，不判任何一版落盘之后读回什么。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::{content_of, plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::committed_reservation_recorded_in;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::lookup_work::{lookup_work_while, LookupWorkOfEachSite};
use singlefs_core::transaction::m4_admission_delta_r3_attack_counts::{take, CopiesAndJudgements};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::e142_parameters;

/// 6 MiB 的 journal 环：一次发布的记录上限 512 条，单元区从槽 1408 起（同第二轮攻方模型那一档）。
const JOURNAL_RING_OF_SIX_MEBIBYTES: u64 = 6 << 20;
/// 小池的单元区槽数（256 MiB）。
const UNIT_AREA_SLOTS_OF_THE_SMALL_POOL: u64 = 16_384;
/// 一次顺序写至多切几个数据单元：比 512 条的记录上限留出余量。
const DATA_UNITS_OF_ONE_SEQUENTIAL_WRITE_AT_MOST: u64 = 400;
/// 池里每块盘一份副本（第一版两块盘）。
const REPLICAS_OF_THE_POOL: u64 = 2;
/// 标签 6 里每条 inode 记录的两项：n_live 一条 41 字节、n_记录 一条 35 字节（`committed_reservation_for_tombstones_and_retirement_records_after_this_publish`）。
const COMMITTED_BYTES_OF_ONE_LIVE_INODE_ON_ONE_REPLICA: u64 = 41 + 35;

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

/// mkfs、取号、暖机、第一个文件（一个数据单元）之后的池，两块盘的单元区各 `unit_area_slots` 槽。
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

/// 建一个空 inode（号取这一版的水位），交回它的号。
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

/// 删 `inode`：意图那次发布，会话接着把队列走空（各批）。
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

/// 打出一次用户改动里每一处的查找计数（连同不在点名的几处里的那一格），一处一行。
fn print_the_lookup_work_of_each_site(label: &str, data_units: u64, work: &LookupWorkOfEachSite) {
    for (site, site_work) in &work.work_of_each_site {
        println!(
            "name=lookup_work change={label} data_units_in_the_version_before={data_units} site={site:?} \
             times_entered={} index_builds={} index_lookups={} index_entries_examined={} largest_index_looked_up={} \
             mapping_leaf_decodes={}",
            site_work.times_entered,
            site_work.index_builds,
            site_work.index_lookups,
            site_work.index_entries_examined,
            site_work.largest_index_looked_up,
            site_work.mapping_leaf_decodes
        );
    }
}

/// 第一个文件顺序写 N 个数据单元之后覆盖写一个单元：释放判定那一处（`LookupSite::MappingEntriesOfTheReleaseJudgement`，固定点每一轮进一次）
/// 每进一次就把上一版的 `mapped_units` 与 `units` 各建一份索引，查过的索引随这一版的单元数涨。B7 的五件（取空槽、记录、释放按索引找；
/// 根头按下标；容器按引用；记账树差量）都不碰这两份索引。每个 N 一个新池；另打一行这一次改动里套上的计数。
#[test]
fn the_release_judgement_indexes_every_unit_of_the_version_on_every_settling_round() {
    let mut largest_at_each_size: Vec<u64> = Vec::new();
    for data_units in [1_u64, 16, 128, DATA_UNITS_OF_ONE_SEQUENTIAL_WRITE_AT_MOST] {
        let mut pool = pool_with_a_unit_area_of(UNIT_AREA_SLOTS_OF_THE_SMALL_POOL);
        pool.sequential_write(content_bytes_of_a_sequential_write_into(data_units))
            .expect("顺序写发成");
        let (units, mapped_units) = {
            let version = pool.current_file_version();
            (version.units.len(), version.mapped_units.len())
        };
        let _ = take();
        let (outcome, work) = lookup_work_while(|| pool.overwrite(OVERWRITE_BYTES));
        outcome.expect("覆盖写发成");
        let counted: CopiesAndJudgements = take();
        print_the_lookup_work_of_each_site("overwrite_of_one_unit", data_units, &work);
        let release_judgement = work.of_the_site(
            singlefs_core::transaction::lookup_work::LookupSite::MappingEntriesOfTheReleaseJudgement,
        );
        println!(
            "name=release_judgement_indexes data_units_in_the_version_before={data_units} units_in_the_version_before={units} \
             mapped_units_in_the_version_before={mapped_units} rounds_entering_the_release_judgement={} index_builds={} \
             largest_index_looked_up={} judgements={} whole_allocator_copies_before_the_judgement={} \
             whole_allocator_copies_in_settling_rounds={}",
            release_judgement.times_entered,
            release_judgement.index_builds,
            release_judgement.largest_index_looked_up,
            counted.judgements,
            counted.whole_allocator_copies_before_the_judgement,
            counted.whole_allocator_copies_in_settling_rounds
        );
        assert!(
            release_judgement.times_entered >= 1
                && release_judgement.index_builds >= release_judgement.times_entered,
            "释放判定那一处每进一次至少建一份索引：{release_judgement:?}"
        );
        largest_at_each_size.push(release_judgement.largest_index_looked_up);
    }
    assert!(
        largest_at_each_size.windows(2).all(|pair| pair[0] < pair[1]),
        "释放判定查过的索引随这一版的单元数涨：{largest_at_each_size:?}"
    );
}

/// 单元区 2^14、2^17、2^20 槽（每块盘 256 MiB、2 GiB、16 GiB）三个池上各做一串用户改动（覆盖写、建 inode、给它写 3 个单元、删它），
/// 每次改动数：两道准入之前那一判进了几次、判之前整份拷了几次分配器、固定点每一轮又拷了几次。整份拷贝至少要拷每块盘按槽的三张位图
/// （已分配、影子账隔离、抬 F 扣住，各一槽一字节），那一项随单元区槽数涨。
#[test]
fn every_publish_copies_the_whole_allocator_before_its_judgement() {
    for exponent in [14_u32, 17, 20] {
        let unit_area_slots = 1_u64 << exponent;
        let mut pool = pool_with_a_unit_area_of(unit_area_slots);
        let mut rows: Vec<(&str, CopiesAndJudgements)> = Vec::new();
        let _ = take();
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
        rows.push(("overwrite_of_one_unit", take()));
        let inode = create_an_inode(&mut pool);
        rows.push(("create_one_inode", take()));
        write_into_the_inode(&mut pool, inode, 3);
        rows.push(("sequential_write_of_three_units_into_the_inode", take()));
        unlink(&mut pool, inode);
        rows.push(("unlink_of_the_inode", take()));
        let allocation_records = pool.session().allocator.records().len();
        let devices = u64::try_from(pool.session().allocator.devices.len()).expect("盘数");
        for (label, counted) in &rows {
            println!(
                "name=allocator_copies change={label} unit_area_slots={unit_area_slots} devices={devices} \
                 allocation_records_after_the_history={allocation_records} judgements={} \
                 whole_allocator_copies_before_the_judgement={} whole_allocator_copies_in_settling_rounds={} \
                 slot_bitmap_bytes_per_copy_at_least={}",
                counted.judgements,
                counted.whole_allocator_copies_before_the_judgement,
                counted.whole_allocator_copies_in_settling_rounds,
                3 * unit_area_slots * devices
            );
            assert!(
                counted.judgements >= 1
                    && counted.whole_allocator_copies_before_the_judgement >= 1
                    && counted.whole_allocator_copies_in_settling_rounds >= 1,
                "每次用户改动至少一次判、判之前至少一份整份拷贝、固定点至少一份：{label} {counted:?}"
            );
        }
    }
}

/// 一次建 k 个 inode 的发布：式子那一判按这次之后那一版的标签 6 判（「写入类发布按发布之后的标签判」），这一版记下的值比上一版多
/// 副本数 × 76 × k 字节。上界快路的已承诺预留若取上一版记下的那一份（`committed_reservation_recorded_in` 对上一版，装置与用例的读法，
/// 它的文档写着「不建 inode 的发布前后相同」），少扣的就是这一截：可用落在这一截里时快路放行、今天拒。
#[test]
fn an_inode_creating_publish_is_judged_by_a_committed_reservation_larger_than_the_one_recorded_before_it() {
    let mut pool = pool_with_a_unit_area_of(UNIT_AREA_SLOTS_OF_THE_SMALL_POOL);
    for inodes in [1_u64, 8, 32] {
        let before = committed_reservation_recorded_in(Some(pool.current_file_version())).0;
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        pool.publish(
            UserChange::NewInodes {
                count: inodes,
                write_time_seconds,
            },
            None,
        )
        .expect("建 inode 发成");
        let after = committed_reservation_recorded_in(Some(pool.current_file_version())).0;
        let expected = REPLICAS_OF_THE_POOL * COMMITTED_BYTES_OF_ONE_LIVE_INODE_ON_ONE_REPLICA * inodes;
        println!(
            "name=committed_reservation_of_an_inode_creating_publish inodes_created={inodes} recorded_before_bytes={before} \
             recorded_after_bytes={after} short_if_the_previous_one_is_read_bytes={} expected_replicas_times_76_times_k={expected} \
             equals_the_expectation={}",
            after.saturating_sub(before),
            if after.saturating_sub(before) == expected { "yes" } else { "no" }
        );
        assert!(after > before, "建 inode 之后那一版的标签 6 比之前大：{before} → {after}");
    }
}
