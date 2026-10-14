//! harness 测试文件 `crates/singlefs-harness/tests/overwrite_in_one_instance.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNode;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::STATISTIC_ALLOCATED_BYTES;
use singlefs_core::recovery::{recover, JournalPolicy, JournalScanReport, RecoveryOutcome};
use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
use singlefs_core::transaction::{PublishError, TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::index_node_entry_capacity;
use singlefs_format::{ROOT_RING_REGIONS, SLOT_BYTES};
use std::collections::BTreeMap;

/// 第二次写的内容：与第一次不同长、不同字节。
pub const SECOND_FILE_BYTES: usize = 4100;

pub fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 在新池新建文件之后、同一个进程同一个实例里覆盖写一次；返回 B 的输出与它在录制流里的起点。
pub fn overwrite(pool: &mut BuiltPool) -> (TransactionOutput, usize) {
    let operations_before = pool.stream.operations().len();
    let previous = pool.output.clone();
    let output = try_overwrite(pool, &previous).expect("覆盖写");
    (output, operations_before)
}

/// 接在 `previous` 之后覆盖写一次，错误原样交回（装不下、不在映射那几条用例要看它）。
pub fn try_overwrite(
    pool: &mut BuiltPool,
    previous: &TransactionOutput,
) -> Result<TransactionOutput, PublishError> {
    crate::common::publish_overwrite_in_process(
        pool,
        previous,
        &second_content(),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
}

pub fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("overwrite-cold");
    let (_, _) = overwrite(&mut pool);
    let image = pool.memory_pool();
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content()
        },
        "冷启动择 B 的根、读回第二次的内容"
    );
    assert_eq!(
        report.journal,
        JournalScanReport {
            valid_records: 4,
            above_water: 0,
            prefix_applied: 0,
            verification_passed: 0,
            verification_failed: 0,
            maximum_applied_transaction: 0
        },
        "暖机两条 + A + B 四条记录，没有一条高于所选根"
    );
    assert_eq!(report.mapping_fallbacks, 0);
    assert_eq!(
        recover(&reopened, JournalPolicy::Ignore).outcome,
        report.outcome,
        "根槽已持久：看不看 journal 结果一样"
    );
    assert_ne!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: file_content()
        },
        "第一次的内容从最新根出发读不到"
    );
    // 步 1 验收第 4 条「inode 记录的写入时间留成 A 的 ⇒ 判红」：从盘上读回 inode 记录，不看发布交回的内存结构——
    // 只读挂载走的是挂载态那一条读路径（`mounted_read`），与发布路径不共用代码。
    let mounted_read_only = mount_read_only(&reopened).expect("B 之后只读挂载");
    let first_file_inode_record = mounted_read_only
        .mounted
        .inode_records()
        .iter()
        .find(|record| record.inode == FIRST_INODE_NUMBER)
        .expect("第一个文件那条 inode 记录在挂载态里");
    assert_eq!(
        first_file_inode_record.write_time_seconds,
        FIXED_WRITE_TIME_SECONDS + 60,
        "盘上 inode 记录的写入时间是 B 那一次写入的时间，不是 A 的 {FIXED_WRITE_TIME_SECONDS}"
    );

    // 这几条要真被评估过、且成立
    images.hand_out(
        "覆盖写之后冷启动",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated()
            .and_holds("I-3.1")
            .and_holds("I-5.1")
            .and_holds("I-5.2")
            .and_holds("I-7.2"),
    );
}

/// 同一个进程同一个实例里连着覆盖写，分配记录攒到多于 812 条（一片叶的条目容量，原先「分配记录树只有一个节点」那道墙）之后照样做成——
/// 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1），每块盘上装着记录的叶不止一片，墙拆了。
/// `build_pool` 的分配器装着 mkfs 的根环（C551），根环转过一格之后照可再分配谓词回收（D16（发布语义） 已定项 1；这一形会话第一次回收
/// 在 txg 26，第 31 批代码轮判决第二节 N2）；回收的落点被重新分配时改写原来那条记录、不追加（D3（空间分配） 已定项 7），
/// 所以记录数只增不减，只是转环之后涨得慢。越过 812 条的那一版上池级 checker 判全部不变量（I-3.1 在内）、0 违例（C551 推翻条件
/// 「`build_pool` 装上根环表之后，同一条用例判全部不变量」），其中管按位置寻址的那两条（I-1.1 节点位置、I-3.10 每个节点都有分配记录）
/// 还要判过且成立、不是没判；冷启动读回最后写的内容。这条只在末尾那一版跑一次 checker，每一版都跑的是
/// `allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`。
pub fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(index_node_entry_capacity(10, 20), 812);
    let mut pool = build_pool("overwrites-past-one-allocation-leaf");
    let mut overwrites = 0;
    // 迭代上界 200 次（回收开始之前每次至少加 16 条，之后有的落点改写原来的记录）；跨轮携带的是现行那一版。
    while pool.output.allocation_records.len() <= 812 && overwrites < 200 {
        let previous = pool.output.clone();
        pool.output = try_overwrite(&mut pool, &previous)
            .unwrap_or_else(|error| panic!("第 {} 次覆盖写：{error:?}", overwrites + 1));
        overwrites += 1;
    }
    assert!(
        pool.output.allocation_records.len() > 812,
        "{overwrites} 次覆盖写之后 {} 条分配记录",
        pool.output.allocation_records.len()
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let leaves_holding_records = pool
            .output
            .allocation_record_tree
            .nodes
            .iter()
            .filter(|(node, _)| {
                matches!(
                    node,
                    AllocationRecordTreeNode::BelowTheRoot(position)
                        if position.level == 0 && position.device == device
                )
            })
            .count();
        assert!(
            leaves_holding_records > 1,
            "盘 {} 上装着记录的叶 {leaves_holding_records} 片",
            device.0
        );
    }
    let image = pool.memory_pool();
    // 越过 812 条之后一条都不红，I-1.1、I-3.10 判过且成立
    images.hand_out(
        &format!(
            "{overwrites} 次覆盖写（txg {}，{} 条分配记录）之后",
            pool.output.root.checkpoint_txg.0,
            pool.output.allocation_records.len()
        ),
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated()
            .and_holds("I-1.1")
            .and_holds("I-3.10"),
    );
    let report = recover(&image, JournalPolicy::Consult);
    let RecoveryOutcome::FileRead { content, .. } = report.outcome else {
        panic!("冷启动要读回文件，实际 {:?}", report.outcome);
    };
    assert_eq!(content, second_content(), "冷启动读回最后一次覆盖写的内容");
}

/// 这个进程从第 0 代根起每个 txg 写一条根、写到 `newest_txg` 之后，根环里最旧的那条根的 txg：mkfs 在三个区域的槽 0 各写一条第 0 代根
/// （`make_filesystem` 写的那三条），之后第 n 条根落在 `target_for_publish` 给的槽上、盖掉那槽原来的，环里每槽留最后写到它的那一条。
/// 一个实例、没有被抛弃的根，环里每一条都有效，所以它就是可再分配谓词里的「环里最旧有效根」（D16（发布语义） 已定项 1）。
/// 这里从 txg 现算，不读分配器的根环表。
pub fn oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
    newest_txg: CheckpointTxg,
    genesis_txg: CheckpointTxg,
) -> CheckpointTxg {
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    let mut txg_of_each_ring_slot: BTreeMap<RootRingSlot, CheckpointTxg> = (0..ROOT_RING_REGIONS)
        .map(|region| (RootRingSlot { region, slot: 0 }, genesis_txg))
        .collect();
    // 迭代上界：newest_txg − genesis_txg 次（这条用例里不过两百多）；不带提前出口。
    for written_txg in genesis_txg.0 + 1..=newest_txg.0 {
        txg_of_each_ring_slot.insert(
            target_for_publish(CheckpointTxg(written_txg), slots_per_region),
            CheckpointTxg(written_txg),
        );
    }
    *txg_of_each_ring_slot
        .values()
        .min()
        .expect("环里至少有 mkfs 写的那几槽")
}

/// 分配记录过 600 条时记账的「已分配」照样等于还没回收的分配记录的跨度之和（增补 3 第 2 件代码三方第二轮判决第三节第 3 条：攻方变异 m4b——
/// 分配记录过 600 条之后「已分配」少记一槽——整个 workspace 没有一条测试会红）。同一个进程同一个实例里从第一个文件起连着覆盖写，
/// 直到越过 812 条（一片叶的条目容量；分配记录树按位置寻址之后那道墙拆了，D8（核心索引结构） 已定项 14）。停在 812 而不再往上取：
/// 回收开始之后复用的落点改写原来那条记录、不再追加，记录涨得慢，200 次覆盖写之后是 882 条（2026-10-05 在装了根环表的 `build_pool` 上跑的）。
/// 没有挂载、没有抬 F（F 恒 0）；
/// `build_pool` 的分配器装着 mkfs 的根环（C551），根环转过一格之后照可再分配谓词回收（D16（发布语义） 已定项 1：已释放 ∧ 释放代 ≤
/// max(F, 环里最旧有效根)）。回收的落点那条记录不删、照旧写着已释放，留到它被重新分配时改写（D3（空间分配） 已定项 7「条目留到该落点
/// 被重新分配时覆盖」），回收的槽不再算已分配；还在 defer 窗口里的已释放落点仍算已分配（I-3.1 读法甲）。所以每一版每盘的「已分配」=
/// 这一版交回的分配记录里那块盘上不是已回收的记录的跨度之和 × 16 KiB，「已回收」= 已释放且释放代 ≤ max(这一版的 F, 写到这一版时环里最旧的根)
/// ——拿分配记录与根的 txg 现算，不看分配器的计数。逐版核，过 600 条的版本至少 5 个，其中带已回收记录的至少 1 个（判法里减掉已回收那一半真走到）。
/// 这个判法的释放代读的是实现交回的分配记录，根环的槽位走的是实现的 `target_for_publish`：释放代记错、根环公式少一槽时它跟着实现一起偏，
/// 照绿（第 31 批代码轮判决第二节 N3）。所以每一版另在盘上镜像上跑一遍池级 checker（全部不变量）、要求 0 违例——它自己从盘上读根环与
/// 分配记录树、自己算谓词，不借实现的这两样（C551 推翻条件「同一条用例判全部不变量」）。镜像接着上一版只补施加这一次覆盖写发出的那一截。
/// 每一次覆盖写接着上一版，次序本身就是被测对象，不切片并行。
pub fn allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("allocated-statistic-past-six-hundred-records");
    let genesis_txg = pool.genesis.root.checkpoint_txg;
    let mut image = pool.memory_pool();
    let mut operations_applied_to_the_image = pool.stream.operation_count();
    let mut versions_past_six_hundred_records = 0;
    let mut versions_carrying_reclaimed_records = 0;
    let mut overwrites = 0;
    // 迭代上界 200 次（回收开始之前每次至少加 16 条，之后有的落点改写原来的记录）；跨轮携带的是现行那一版、
    // 与它对应的盘上镜像和镜像已经施加到录制流第几步（不变量：等于流的长度，每一轮补施加之后重新对齐）。
    while pool.output.allocation_records.len() <= 812 && overwrites < 200 {
        let previous = pool.output.clone();
        let output = try_overwrite(&mut pool, &previous)
            .unwrap_or_else(|error| panic!("第 {} 次覆盖写：{error:?}", overwrites + 1));
        overwrites += 1;
        let operations = pool.retained_operations();
        image.apply(&operations[operations_applied_to_the_image..]);
        operations_applied_to_the_image = operations.len();
        images.hand_out(
            &format!(
                "第 {overwrites} 次覆盖写（txg {}，{} 条分配记录）之后",
                output.root.checkpoint_txg.0,
                output.allocation_records.len()
            ),
            || image.clone(),
            PoolCheckerExpectation::no_invariant_violated(),
        );
        let reclaim_floor = output.root.rollback_floor.max(
            oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
                output.root.checkpoint_txg,
                genesis_txg,
            ),
        );
        let is_reclaimed =
            |record: &&AllocationRecord| record.is_released && record.generation <= reclaim_floor;
        if output
            .allocation_records
            .iter()
            .any(|record| is_reclaimed(&record))
        {
            versions_carrying_reclaimed_records += 1;
        }
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let span_sum_in_slots: u64 = output
                .allocation_records
                .iter()
                .filter(|record| record.device == device)
                .filter(|record| !is_reclaimed(record))
                .map(|record| u64::from(record.span_slots))
                .sum();
            let allocated_bytes = output
                .accounting_entries
                .iter()
                .find(|entry| {
                    entry.statistic == STATISTIC_ALLOCATED_BYTES && entry.device == device
                })
                .map(|entry| entry.value)
                .expect("带设备维的行每盘一行");
            assert_eq!(
                allocated_bytes,
                span_sum_in_slots * SLOT_BYTES,
                "第 {overwrites} 次覆盖写（{} 条分配记录，回收门槛 txg {}）盘 {}：「已分配」要等于这一版不是已回收的分配记录的跨度之和",
                output.allocation_records.len(),
                reclaim_floor.0,
                device.0
            );
        }
        if output.allocation_records.len() > 600 {
            versions_past_six_hundred_records += 1;
        }
        pool.output = output;
    }
    assert!(
        pool.output.allocation_records.len() > 812,
        "{overwrites} 次覆盖写之后 {} 条",
        pool.output.allocation_records.len()
    );
    assert!(
        versions_past_six_hundred_records >= 5,
        "过 600 条的版本 {versions_past_six_hundred_records} 个"
    );
    assert!(
        versions_carrying_reclaimed_records >= 1,
        "带已回收记录的版本 {versions_carrying_reclaimed_records} 个：根环一次都没转过，判法里减掉已回收那一半没走到"
    );
}
