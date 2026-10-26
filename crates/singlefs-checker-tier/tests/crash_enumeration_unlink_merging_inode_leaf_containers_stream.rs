//! checker 档模块：crash、crash_state_enumeration
//! C116（叶容器的分裂 / 合并纪律没有会失败的检查）欠的那一半：inode 树叶容器合并那几次发布的崩溃点重放。unlink 最后一批删掉一条 inode 记录之后
//! 照 E176 M3 那一臂先并后借（`crates/singlefs-core/src/inode_tree.rs` 的 `remove_record_from_leaf_containers`）：这条流里「左从右借」与
//! 「左吸收右」各发生一次，按 D13（验证路线） 已定项 4 枚举这几次发布的每个写请求之后截断的全部崩溃状态，每个状态跑两遍恢复（看 / 不看 journal）
//! + 多版本 oracle + 池级 checker（I-9.13 类型 2 容器不空、I-9.12 分隔 key、I-9.4 容器号、I-9.35 退役完备都在里面）+ 记录核对器。
//!
//! **起点镜像不枚举，只录合并那几次发布**（照 `crash_enumeration_tree_split_streams.rs` 的做法）：新池新建文件之后建 758 个空 inode
//! （第一个文件连同它们 759 条，叶从左往右 233 条一片装满：[1..=233]、[234..=466]、[467..=699]、[700..=759]），再把第二片最小的 116 个号
//! （234..=349）与第四片的 700 逐个 unlink 到底（意图、最后一批：空 inode 没有数据单元，意图之后直接是最后一批）——第二片剩 117 条（借的下限
//! ⌈233 ÷ 2⌉ = 117，不借）、第四片剩 59 条；这一大截（txg 1–238）全部施加成起点镜像。录制流从下面四次发布的第一个单元写切起：
//! - unlink 350 的意图（txg 239）与最后一批（txg 240）：第二片剩 116 条，与两边哪一对都合不进一片，从右邻借 ⌊(233 − 116) ÷ 2⌋ = 58 条
//!   （467..=524）：两片 174 / 175，右邻的分隔 key 改成 525；借出的那一片与第四片合计 175 + 59 = 234，不再并。两片都重写、身份都不变，不写退役记录；
//! - unlink 701 的意图（txg 241）与最后一批（txg 242）：第四片剩 58 条，与左邻合计 175 + 58 = 233 装得下一片，左邻吸收它（左片保留身份），
//!   第四片身份消失、同一次发布写一条退役记录（退役树开第一片容器、写根）。
//!
//! 第一个文件四次发布都不碰：五版（起点那一版与这四次）读回同一份内容，版本表照这五版判。
//!
//! **比已有的流多罩了什么**：已有的崩溃点重放流里 inode 树只有一片叶（删除流的最后一批把那一片删空不写），合并与借从没进过崩溃点；
//! 这里借的那一次（13 个单元：借方与右邻两片叶容器、inode 根、分配记录树五个节点、记账、映射、树表）与吸收的那一次（12 个单元：
//! 幸存的左片、inode 根、退役容器与退役树根……）的单元写段里，「借方新片落了、右邻新片没落」「新根落了、某片新叶没落」「退役记录落了、
//! 吸收那一片没落」这一类子集第一次被枚举到。多跑的一步只有恢复本身（没有重开、挂载）。与已有流的基线镜像、写表、段序列都不相同，全量枚举。
//!
//! 快档只展开写数小于 10 的段（四个单元段不展开，只以整段持久进入后面的状态）；全量标 ignored，登记成崩溃枚举用例。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。
//! 判器先判、形状后核：改坏合并或借那一步时先红在判器上（`crates/mutations.tsv` 里点名这份文件的两行），形状与计数在判干净之后才核。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[cfg(feature = "verdict-store")]
mod common_crash_points;

use common::{
    build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally,
};
use singlefs_checker_tier::crash_point_replay_provenance::{
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow, RunOfTheFlow,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_selecting_versions, CrashPointReplayItem,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_merging_inode_leaf_containers_stream.rs";
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::records::RetirementRecord;
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::PackedIdentity;
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 第一个文件之外建几个空 inode：759 条记录装成 233、233、233、60 四片。
const NEW_INODES: u64 = 758;
/// 建空 inode 那次与各次 unlink 意图的调用方时间。
const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = FIXED_WRITE_TIME_SECONDS + 100;
/// 起点镜像之前 unlink 到底的号：第二片最小的 116 个（剩 117 条，正好不低于借的下限），与第四片的 700（剩 59 条）。
const INODES_UNLINKED_BEFORE_THE_RECORDED_PUBLISHES: [std::ops::RangeInclusive<u64>; 2] =
    [234..=349, 700..=700];
/// 被录的那一次 unlink 让第二片低于借的下限、从右邻借。
const INODE_WHOSE_UNLINK_BORROWS: u64 = 350;
/// 被录的那一次 unlink 让第四片与左邻合计恰好 233 条、左邻吸收它。
const INODE_WHOSE_UNLINK_ABSORBS: u64 = 701;
/// 借之后右邻（第三片）的分隔 key：它这一版的最小 key（N1），467 + 58。
const SEPARATOR_KEY_OF_THE_LENDER_AFTER_THE_BORROW: u64 = 525;
/// 起点那一版的 txg：暖机两次、A、建空 inode 一次，再 117 次 unlink 各两次发布。
const TXG_OF_THE_VERSION_BEFORE_THE_RECORDED_PUBLISHES: u64 = 238;

/// 快档的状态按发布分：意图 txg 239 前面的轮换在起点镜像里，它只有记录段 2² − 1 = 3 与根槽段 1；之后每次发布前面多一段上一次的轮换 3² − 1 = 8
/// （系统配置槽写取三态）；最后一次根槽写之后是 txg 242 的轮换 8，再加全部持久那一个。四个单元段不展开、不出状态。
const QUICK_TIER_STATES_BY_PUBLISH: &str = "instance1_txg239=4 instance1_txg240=12 \
     instance1_txg241=12 instance1_txg242=12 after_the_last_root=8 every_write_persisted=1";
/// 快档的状态数：1 + 四次发布各（记录 3 + 根槽 1 + 轮换 8）。每次写只取两态时各（3 + 1 + 3）、29。
const QUICK_TIER_STATES: u64 = 1 + 4 * (3 + 1 + 8);
/// 全量的状态按发布分：单元段 13、13、13、12 个单元（每个两盘各一次写），2^26 − 1、2^26 − 1、2^26 − 1、2^24 − 1。
const FULL_STATES_BY_PUBLISH: &str = "instance1_txg239=67108867 instance1_txg240=67108875 \
     instance1_txg241=67108875 instance1_txg242=16777227 after_the_last_root=8 every_write_persisted=1";
/// 全量每次写只取两态时的闭式 1 + Σ(2^|段| − 1)，十六段。
const FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 =
    1 + 3 * ((1 << 26) - 1) + ((1 << 24) - 1) + 4 * (3 + 1 + 3);
/// 全量枚举的状态数：四个系统配置槽 2 写段各 3² − 1（比两态多 5）。
const FULL_STATES: u64 = FULL_STATES_WITH_TWO_STATES_PER_WRITE + 4 * 5;

/// 一次 unlink 一个空 inode 录下的两次发布。
struct UnlinkOfAnEmptyInode {
    intent: TransactionOutput,
    final_batch: TransactionOutput,
}

/// 接着现行那一版建 `count` 个空 inode（实例 1）。
fn publish_new_inodes_and_advance_the_pool(pool: &mut BuiltPool, count: u64) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        count,
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        InstanceGeneration(1),
    )
    .expect("建空 inode");
}

/// unlink 一个空 inode（实例 1，产品批大小）：意图那一次，再发一步——空 inode 没有数据单元，意图之后那一步就是最后一批。
/// 步数按计划定死、不按「队列空了没有」停：最后一批留着队列里的号时照样录完，红在判器上。
fn unlink_an_empty_inode(pool: &mut BuiltPool, inode: u64) -> UnlinkOfAnEmptyInode {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let intent = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        InstanceGeneration(1),
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    let final_batch = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &intent,
        InodeNumber(inode),
        UnlinkBatchSize::OneExtentLowerLeaf,
        InstanceGeneration(1),
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的最后一批：{refusal:?}"));
    pool.output = final_batch.clone();
    UnlinkOfAnEmptyInode {
        intent,
        final_batch,
    }
}

fn leaf_record_counts(version: &TransactionOutput) -> Vec<usize> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect()
}

fn leaf_identities(version: &TransactionOutput) -> Vec<PackedIdentity> {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.identity)
        .collect()
}

fn retirement_records_of(version: &TransactionOutput) -> Vec<RetirementRecord> {
    version
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect()
}

/// 录好的流：起点镜像、四次发布的写表与段、被判的根槽写（吸收那一次的）、oracle 认的五版，连同形状要核的那几版（判干净之后才核）。
struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    before: TransactionOutput,
    borrow: UnlinkOfAnEmptyInode,
    absorb: UnlinkOfAnEmptyInode,
}

fn prepare(tag: &str) -> Prepared {
    let mut pool = build_pool(tag);
    publish_new_inodes_and_advance_the_pool(&mut pool, NEW_INODES);
    let records_per_leaf = usize::try_from(INODE_LEAF_RECORDS).expect("233");
    assert_eq!(
        leaf_record_counts(&pool.output),
        vec![records_per_leaf, records_per_leaf, records_per_leaf, 60],
        "759 条记录从左往右 233 条一片装满"
    );
    // 迭代上界：两段号合计 117 个。
    for inode in INODES_UNLINKED_BEFORE_THE_RECORDED_PUBLISHES
        .iter()
        .flat_map(|range| range.clone())
    {
        unlink_an_empty_inode(&mut pool, inode);
    }
    let before = pool.output.clone();
    assert_eq!(
        (
            before.root.checkpoint_txg.0,
            leaf_record_counts(&before),
            leaf_identities(&before).len()
        ),
        (
            TXG_OF_THE_VERSION_BEFORE_THE_RECORDED_PUBLISHES,
            vec![records_per_leaf, 117, records_per_leaf, 59],
            4
        ),
        "起点那一版：第二片剩 117 条（不低于借的下限、不借），第四片剩 59 条，一次都没并没借"
    );
    let operations_before_the_recorded_publishes = pool.stream.operation_count();
    let borrow = unlink_an_empty_inode(&mut pool, INODE_WHOSE_UNLINK_BORROWS);
    let absorb = unlink_an_empty_inode(&mut pool, INODE_WHOSE_UNLINK_ABSORBS);
    let operations = pool.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..operations_before_the_recorded_publishes]);
    let (writes, segments) = writes_and_segments(
        &operations[operations_before_the_recorded_publishes..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let versions = [
        &before,
        &borrow.intent,
        &borrow.final_batch,
        &absorb.intent,
        &absorb.final_batch,
    ]
    .into_iter()
    .map(|version| PublishedVersion {
        instance: version.root.instance,
        checkpoint_txg: version.root.checkpoint_txg,
        content: file_content(),
    })
    .collect();
    Prepared {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        before,
        borrow,
        absorb,
    }
}

/// 判器那一半：oracle、不看 journal 那一遍、走读、记录核对器两条、池级 checker 每条不变量都不判违例；
/// 合并与借罩着的那几条（I-9.4、I-9.12、I-9.13、I-9.35）真被评估过（阴性结果要与「代码没跑到」分开）。
fn assert_clean(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    for must_evaluate in ["I-9.4", "I-9.12", "I-9.13", "I-9.35"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过：{}",
            tally.checker_counts_by_invariant()
        );
    }
}

/// 形状（判干净之后才核）：借那一次两片 174 / 175、四片身份都在、右邻分隔 key 525、不写退役记录；吸收那一次剩三片 233 / 174 / 233、
/// 左片保留身份、第四片写一条退役记录、死亡 txg 是吸收那一次的 txg；十六段、每次发布「单元写 → 记录 2 → 根槽 1 → 轮换 2」，四次根槽写。
fn assert_the_shape_of_the_merges(prepared: &Prepared) {
    let records_per_leaf = usize::try_from(INODE_LEAF_RECORDS).expect("233");
    let identities_before = leaf_identities(&prepared.before);
    let borrowed = &prepared.borrow.final_batch;
    assert_eq!(
        (leaf_identities(borrowed), leaf_record_counts(borrowed)),
        (
            identities_before.clone(),
            vec![records_per_leaf, 174, 175, 59]
        ),
        "第二片 116 条从右邻借 58 条：174 / 175，四片身份都在"
    );
    assert_eq!(
        borrowed.inode_leaf_containers[2]
            .contents
            .separator_key_in_the_parent_entry,
        SEPARATOR_KEY_OF_THE_LENDER_AFTER_THE_BORROW,
        "右邻的分隔 key 改成它这一版的最小 key（N1）"
    );
    assert_eq!(
        retirement_records_of(borrowed),
        Vec::<RetirementRecord>::new(),
        "借不写退役记录"
    );
    let absorbed = &prepared.absorb.final_batch;
    assert_eq!(
        (leaf_identities(absorbed), leaf_record_counts(absorbed)),
        (
            identities_before[..3].to_vec(),
            vec![records_per_leaf, 174, records_per_leaf]
        ),
        "第三片吸收第四片：175 + 58 = 233，保留第三片的身份"
    );
    assert_eq!(
        retirement_records_of(absorbed),
        vec![RetirementRecord {
            retired: identities_before[3],
            death_txg: absorbed.root.checkpoint_txg,
        }],
        "被吸收的第四片写一条退役记录，死亡 txg = 吸收那一次的 txg"
    );
    let sizes: Vec<usize> = prepared.segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![26, 2, 1, 2, 26, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2],
        "四次发布：两次意图各 13 个单元（墓碑容器与根、deleted_inodes 根、那条记录所在的叶、inode 根……）、借那一次 13 个（两片叶）、\
         吸收那一次 12 个（一片叶、退役容器与退役树根）；各自记录 2、根槽 1、轮换 2"
    );
    assert_eq!(
        prepared
            .writes
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .count(),
        4,
        "四次发布各一条根槽写"
    );
}

fn expansion_of_the_segments_shorter_than_ten(
    _segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if segment.len() < 10 {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

/// 平时跑的那一份：四个单元段不展开，记录段、根槽、轮换全展开——「单元全落、记录落了一份、根没落」这一类每次发布都跑到。
#[test]
fn every_crash_state_after_the_unit_writes_of_the_borrowing_and_the_absorbing_unlink_recovers_cleanly(
) {
    let prepared = prepare("unlink-merging-inode-leaf-containers-quick");
    let Some(tally) = enumerate_crash_states_selecting_versions(
        CrashPointReplayItem {
            name: "unlink_merging_inode_leaf_containers::quick_tier",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &|_segment_index, segment| segment.len() < 10,
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_UNLINK_MERGING_INODE_LEAF_CONTAINERS_FAST states={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "unlink_merging_inode_leaf_containers",
        &tally,
    );
    assert_the_shape_of_the_merges(&prepared);
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &expansion_of_the_segments_shorter_than_ten,
        ),
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_eq!(tally.states, QUICK_TIER_STATES, "快档的状态数");
    assert_eq!(
        tally.states_by_publish_text(),
        QUICK_TIER_STATES_BY_PUBLISH,
        "快档的状态按发布分"
    );
    assert_eq!(
        tally.states_by_publish.values().sum::<u64>(),
        tally.states,
        "按发布分的各格加起来就是状态数"
    );
}

/// 全量：十六段全展开，四个单元段各 2^n − 1（n = 26、26、26、24），闭式 [`FULL_STATES`] 个状态，每个两遍恢复 + checker + 记录核对器。
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv`），门禁 checker-tier-crash-point-replay --full 在 release 下跑，认下面打印的
/// `CRASH_POINT_REPLAY_UNLINK_MERGING_INODE_LEAF_CONTAINERS` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量 218103853 个状态（两亿多）、每个两遍恢复 + checker；门禁 checker-tier-crash-point-replay --full 在 release 下跑"]
fn full_enumeration_of_the_unlink_merging_inode_leaf_containers_stream_is_exhaustive_and_clean() {
    let prepared = prepare("unlink-merging-inode-leaf-containers-full");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_crash_states_selecting_versions(
        CrashPointReplayItem {
            name: "unlink_merging_inode_leaf_containers::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &|_segment_index, _segment| true,
    )
    .judged() else {
        return;
    };
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    println!(
        "CRASH_POINT_REPLAY_UNLINK_MERGING_INODE_LEAF_CONTAINERS states={} closed_form={closed_form} exhaustive={} violations={} ignored_violations={} failed={} root_persisted_states={} journal_differing_states={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.ignored_violations,
        tally.failed_states,
        tally.root_persisted_states,
        tally.journal_differing_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        "unlink_merging_inode_leaf_containers",
        &tally,
    );
    assert_the_shape_of_the_merges(&prepared);
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    assert_eq!(
        (closed_form_state_count(&prepared.segments), closed_form),
        (FULL_STATES_WITH_TWO_STATES_PER_WRITE, FULL_STATES),
        "两态闭式 1 + Σ(2^|段| − 1)；系统配置槽写取三态的四段各多 5"
    );
    assert_eq!(
        tally.states_by_publish_text(),
        FULL_STATES_BY_PUBLISH,
        "全量的状态按发布分"
    );
}

/// 崩溃放量的形态：一个崩溃点罩住整张写表（起点镜像不枚举、只录合并那几次发布，同 `crash_enumeration_tree_split_streams.rs`），
/// 按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判。展开上限默认 6。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_unlink_merging_inode_leaf_containers_stream_is_clean() {
    let prepared = prepare("crash-amplification-unlink-merging-inode-leaf-containers");
    let flow_name = "unlink-merging-inode-leaf-containers";
    let crash_points = common_crash_points::one_crash_point_over_every_write(
        file!(),
        flow_name,
        prepared.writes.len(),
    );
    common_crash_points::amplify_prepared_flow(
        flow_name,
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        crash_points,
        6,
    );
}
