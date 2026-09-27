//! 层 0 流 L8（里程碑「第二个事务」增补 2 收口表第 36 行、调度表「实二十」；三方 `m2-treesplit-r1` T4 列的八条流里的最后一条）：
//! 一次发布多条记录、共享点名跨记录。D23（journal 的角色与格式） 已定项 17（用户 2026-09-24 定）：最后一个事务要点名的项多于
//! 一条记录装得下的 67 项时末条再跨记录，只有真正的最后一条带「本次发布末条」标志；已定项 7：同一事务的记录共享事务号、
//! 提交标记只在它的最后一条；已定项 14 第六条：恢复按记录标志位 0 认发布边界，末条没到的发布整体不施加。
//!
//! 写法照 T4「起点镜像不枚举 + 只录那一次发布」：mkfs → 取号 → 暖机 × 2 → A（第一个事务）整段做完的镜像当起点、不枚举；
//! 只把建 N 个 inode 的那一次发布（叶容器 + 叶容器之外的角色 = 68 个单元，一个事务，点名 68 项 ⇒ 两条记录：
//! 67 项不带提交标记与末条标志、1 项两样都带）录进被枚举的流。叶容器之外的角色有几个不写死，由实现现给
//! （[`leaf_containers_naming_one_more_entry_than_a_record_holds`]）：今天是 11 个（inode 根、分配记录树两盘各叶 61 与叶 62、
//! 两盘各第 1 层、根，记账树、中央映射树、树表），叶容器 57 片。它的段序列是 `136+4+1+2`：单元段 68 个单元 × 2 盘，
//! 记录段两条 × 2 盘，根槽 FUA，系统配置槽轮换 × 2 盘。
//!
//! **枚举了哪些崩溃状态**：记录段、根槽两段逐个子集全展开（15 + 1），系统配置槽那一段的两次轮换是原地覆写、各取三态
//! （3² − 1 = 8，`crash::TearableInPlaceOverwrites`），加全部持久那一个，共 25 个；
//! 单元段 136 写的 2^136 − 1 个子集枚举不了，它只以整段持久进入后面的状态。单元段里的状态上这次发布的记录一份都没落
//! （记录段在一道屏障之后），恢复见到的与起点镜像相同的那一版加上几份没人点名的单元——那一格已有的流都罩着。
//! **比已有的流多罩了什么**：已有的多记录发布（并行线一的层 0 流）一条记录一个事务、每条都带提交标记；这里一个事务跨两条记录，
//! 第一条不带提交标记，共享的提交内生块分在两条里点名——记录段 15 个状态里：两条都至少落了一份的 8 个（加上根槽那一段的 1 个）
//! 要由记录施加这一版（恢复不许停在不带提交标记的第一条上），只落了其中一条的 6 个要整次不施加（第一条落了而末条没落是半次发布，
//! 末条落了而第一条没落是断号）。多跑的一步只有恢复本身（这条流上没有重开、挂载）。每个状态照常跑两遍恢复 + 多版本 oracle、
//! 池级 checker（I-8.8（前缀里的事务不被切开）、I-8.9（一次发布的记录序号连续且只有末条带标志） 在内）、记录核对器。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, crash_state_devices, file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state,
    layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion, Layer0Tally,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::journal::record_offset;
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{
    publish_new_inodes, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_format::{
    INODE_LEAF_RECORDS, JOURNAL_NAMED_ENTRIES_PER_RECORD, JOURNAL_RING_DEFAULT_BYTES,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 起点镜像里第一个事务之后已有的 inode（inode 1）。
const INODES_BEFORE: u64 = 1;

/// 建 inode 那一次发布要点名的项数：比一条 journal 记录装得下的多一项，末条恰好跨出去 1 项。
const NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH: u64 = JOURNAL_NAMED_ENTRIES_PER_RECORD + 1;

/// 建 `leaf_containers` 片满的 inode 叶容器（第一片里已有 inode 1）要建几个 inode。
fn new_inode_count_filling(leaf_containers: u64) -> u64 {
    leaf_containers * INODE_LEAF_RECORDS - INODES_BEFORE
}

/// 在起点镜像的一份内存拷贝上（分配器与上一版也是拷贝）试发一次建 inode、填满 `leaf_containers` 片叶容器，
/// 数这次点名的单元里叶容器之外的角色有几个：`publish_new_inodes` 交回的 `TransactionOutput::rewritten`
/// 除去 `TransactionUnit::InodeLeafContainer`。拷贝上的写进一条另起的流，不进被录的那一条。
fn roles_other_than_leaf_containers_named_by_a_trial_publish(
    start: &MemoryPool,
    allocator: &PoolAllocator,
    before: &TransactionOutput,
    leaf_containers: u64,
) -> u64 {
    let mut trial_devices = crash_state_devices(start, &[], &[], &SharedStream::new());
    let mut trial_allocator = allocator.clone();
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, trial_devices.as_mut_slice());
    let trial = publish_new_inodes(
        &mut writer,
        &mut trial_allocator,
        before,
        new_inode_count_filling(leaf_containers),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("试发：建 inode 在内存拷贝上发得出来");
    let roles = trial
        .rewritten
        .iter()
        .filter(|unit| !matches!(unit, TransactionUnit::InodeLeafContainer(_)))
        .count();
    u64::try_from(roles).expect("一次发布的角色数装得进 u64")
}

/// 叶容器取几片，这次发布点名的单元才恰好是 [`NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH`] 项。叶容器之外的角色数随叶容器数变
/// （叶容器落点越过分配记录树叶 61 的末槽时，两块盘各多改一片叶 62：今天 1 片叶容器时 9 个角色、57 片时 11 个），
/// 所以按「叶容器数 = 项数 − 这么多叶容器时的角色数」迭代到不动点，每一轮的角色数都由试发现数。
///
/// 迭代上界 8 轮：角色数随叶容器数不减，从最多的叶容器数（只留 1 个角色）起往下收；今天第 2 轮就不动。
fn leaf_containers_naming_one_more_entry_than_a_record_holds(
    start: &MemoryPool,
    allocator: &PoolAllocator,
    before: &TransactionOutput,
) -> u64 {
    let mut leaf_containers = NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH - 1;
    for _round in 0..8 {
        let roles = roles_other_than_leaf_containers_named_by_a_trial_publish(
            start,
            allocator,
            before,
            leaf_containers,
        );
        let next = NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH
            .checked_sub(roles)
            .expect("叶容器之外的角色不多于要点名的项数");
        if next == leaf_containers {
            return leaf_containers;
        }
        leaf_containers = next;
    }
    panic!("叶容器数 8 轮没收拢到不动点：角色数随叶容器数来回变，这条流的布置要重想");
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    /// 这次发布的两条记录按 jsn 升序，每条两盘各一份：写表下标。
    record_writes_in_order: Vec<Vec<usize>>,
    version_before: (InstanceGeneration, CheckpointTxg),
    version_of_the_spilling_publish: (InstanceGeneration, CheckpointTxg),
}

/// 写表里写到某个 journal 记录槽（两盘同偏移）的那几格。
fn writes_to_record(writes: &[RetainedWrite], counter: u64) -> Vec<usize> {
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord && write.offset == offset)
        .map(|(index, _)| index)
        .collect()
}

fn prepare(tag: &str) -> Prepared {
    let mut pool = build_pool(tag);
    let before = pool.output.clone();
    // 起点镜像：A 整段做完（系统配置槽轮换在内），不枚举。
    let base = pool.memory_pool();
    let operations_before = pool.retained_operations().len();
    // 叶容器数由实现现给的角色数定（试发在 `base` 的拷贝上，不进录制流）：点名 68 项，比一条记录装得下的 67 多一项。
    let leaf_containers =
        leaf_containers_naming_one_more_entry_than_a_record_holds(&base, &pool.allocator, &before);
    let new_inode_count = new_inode_count_filling(leaf_containers);
    let publish_parameters = parameters();
    let spilling = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &before,
            new_inode_count,
            FIXED_WRITE_TIME_SECONDS + 120,
            InstanceGeneration(1),
        )
        .expect("点名 68 项：末条再跨记录")
    };
    assert_eq!(
        spilling.rewritten.len(),
        68,
        "{leaf_containers} 片叶容器 + 叶容器之外的角色（由试发现数），恰好比一条记录装得下的 67 多一项：{:?}",
        spilling.rewritten
    );
    assert_eq!(
        u64::try_from(spilling.rewritten.len()).expect("68"),
        NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH,
        "68 就是一条记录装得下的项数加一（`JOURNAL_NAMED_ENTRIES_PER_RECORD` + 1）"
    );
    assert_eq!(
        (
            spilling.earlier_records_of_this_publish.len(),
            spilling.earlier_records_of_this_publish[0]
                .record
                .named
                .len(),
            spilling.record.named.len()
        ),
        (1, 67, 1),
        "两条记录：装满 67 项的一条 + 跨出去的 1 项"
    );
    let operations = pool.retained_operations();
    let (writes, segments) = writes_and_segments(&operations[operations_before..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![136, 4, 1, 2],
        "只录这一次发布：68 个单元 × 2 盘、两条记录 × 2 盘、根槽 FUA、系统配置槽 × 2 盘"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 1, "只录了这一次发布的根槽写");
    let record_writes_in_order: Vec<Vec<usize>> = [
        spilling.earlier_records_of_this_publish[0].record.counter,
        spilling.record.counter,
    ]
    .into_iter()
    .map(|counter| writes_to_record(&writes, counter))
    .collect();
    for copies in &record_writes_in_order {
        assert_eq!(copies.len(), 2, "每条记录两盘各一份");
    }
    let version_before = (before.root.instance, before.root.checkpoint_txg);
    let version_of_the_spilling_publish = (spilling.root.instance, spilling.root.checkpoint_txg);
    // 建 inode 不改第一个文件：两版读回的都是它。版本按 (实例, txg) 认，oracle 判的是走到哪一版的根。
    let versions = vec![
        PublishedVersion {
            instance: version_before.0,
            checkpoint_txg: version_before.1,
            content: file_content(),
        },
        PublishedVersion {
            instance: version_of_the_spilling_publish.0,
            checkpoint_txg: version_of_the_spilling_publish.1,
            content: file_content(),
        },
    ];
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: root_indexes[0],
        versions,
        record_writes_in_order,
        version_before,
        version_of_the_spilling_publish,
    }
}

/// 一个状态上这次发布的两条记录各落了没有（任一份算落了）与根槽落了没有，恢复该走哪一版的根、该施加几条。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PublishBoundaryStates {
    /// 两条记录都一份没落：还是上一版。
    no_record: u64,
    /// 只落了其中一条：第一条落了、末条没落是半次发布；末条落了、第一条没落是断号。都整次不施加。
    only_one_of_the_two_records: u64,
    /// 两条都至少落了一份、根槽没落：由记录整次施加这一版（第一条不带提交标记，恢复不许停在它那里）。
    both_records_without_the_root: u64,
    /// 根槽落了：所选根就是这一版。
    root_persisted: u64,
}

fn observe_the_publish_boundary(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut PublishBoundaryStates,
) {
    let is_persisted = |copies: &[usize]| copies.iter().any(|copy| image.persisted[*copy]);
    let first_record = is_persisted(&prepared.record_writes_in_order[0]);
    let last_record = is_persisted(&prepared.record_writes_in_order[1]);
    let root = image.persisted[prepared.judged_root_index];
    let (expected_root, expected_applied, what) = match (root, first_record, last_record) {
        (true, _, _) => {
            counts.root_persisted += 1;
            (
                prepared.version_of_the_spilling_publish,
                0,
                "根槽落了：所选根就是这一版",
            )
        }
        (false, true, true) => {
            counts.both_records_without_the_root += 1;
            (
                prepared.version_of_the_spilling_publish,
                2,
                "两条记录都落了、根没落：由记录整次施加这一版",
            )
        }
        (false, true, false) | (false, false, true) => {
            counts.only_one_of_the_two_records += 1;
            (
                prepared.version_before,
                0,
                "只落了其中一条：这次发布整体不施加",
            )
        }
        (false, false, false) => {
            counts.no_record += 1;
            (prepared.version_before, 0, "记录一条都没落：还是上一版")
        }
    };
    assert_eq!(
        (report.effective_root, report.journal.prefix_applied),
        (Some(expected_root), expected_applied),
        "{what}（{:?}）",
        report.journal
    );
}

fn assert_clean(tally: &Layer0Tally) {
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
    for must_evaluate in ["I-8.8", "I-8.9"] {
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

/// L8 能枚举的全部状态：记录段（4 写）、根槽（1 写）逐个子集全展开，系统配置槽（2 写，原地覆写各取三态）全展开，
/// 加全部持久那一个，共 25 个；
/// 单元段 136 写不展开（2^136 − 1 个子集，只以整段持久进入后面的状态）。每个状态按两条记录落没落核发布边界。
#[test]
fn every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all(
) {
    let prepared = prepare("spill-over-layer0");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 64;
    let mut counts = PublishBoundaryStates::default();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
        &mut |image, report| {
            observe_the_publish_boundary(&prepared, image, report, &mut counts);
        },
    );
    assert_eq!(
        tally.states,
        layer0_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &|segment_index, segment| {
                if expand(segment_index, segment) {
                    Layer0SegmentExpansion::EveryProperSubset
                } else {
                    Layer0SegmentExpansion::NotExpanded
                }
            },
        ),
        "展开的段按层 0 的枚举域数（原地覆写取三态）"
    );
    assert_eq!(
        tally.states,
        1 + ((1 << 4) - 1) + ((1 << 1) - 1) + (3 * 3 - 1),
        "记录段 15、根槽 1、系统配置槽 3² − 1（两次轮换各取三态；每次写只取两态时 3）、全部持久 1"
    );
    assert_eq!(
        counts,
        PublishBoundaryStates {
            no_record: 1,
            only_one_of_the_two_records: 6,
            both_records_without_the_root: 9,
            root_persisted: 1 + (3 * 3 - 1),
        },
        "记录段 15 个状态：一条都没落 1、只落一条 3 + 3、两条都落 8；根槽那一段 1 个两条都落；\
         之后 9 个根槽已落（系统配置槽那一段 3² − 1 + 全部持久 1）"
    );
    println!(
        "LAYER0_SPILL_OVER states={} publish_boundary={counts:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
}
