//! 验证包模块：crash、crash_state_enumeration、crash_library、crash_amplification
//! 崩溃点重放流 L8（里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 36 行、调度表「实二十」；三方 `m2-treesplit-r1` T4 列的八条流里的最后一条）：
//! 一次发布多条记录、共享点名跨记录。D23（journal 的角色与格式） 已定项 17：最后一个事务要点名的项多于
//! 一条记录装得下的 64 项时末条再跨记录，只有真正的最后一条带「本次发布末条」标志；已定项 7：同一事务的记录共享事务号、
//! 提交标记只在它的最后一条；已定项 14 第六条：恢复按记录标志位 0 认发布边界，末条没到的发布整体不施加。
//!
//! 写法照 T4「起点镜像不枚举 + 只录那一次发布」：mkfs → 取号 → 暖机 × 2 → A（新池新建文件）整段做完的镜像当起点、不枚举；
//! 只把建 N 个 inode 的那一次发布（叶容器 + 叶容器之外的角色 = 65 个单元，一个事务，点名 65 项 ⇒ 两条记录：
//! 64 项不带提交标记与末条标志、1 项两样都带）录进被枚举的流。叶容器之外的角色有几个不写死，由实现现给
//! （[`leaf_containers_naming_one_more_entry_than_a_record_holds`]；叶容器跨过分配记录树叶 61 的末槽时是 11 个——inode 根、分配记录树两盘各叶 61
//! 与叶 62、两盘各第 1 层、根，记账树、中央映射树、树表；叶容器片数 = 65 − 角色数，记录头改成 487 字节之后没在这条流上跑过）。它的段序列是
//! `130+4+1+2`：单元段 65 个单元 × 2 盘，记录段两条 × 2 盘，根槽 FUA，系统配置槽轮换 × 2 盘。
//!
//! **枚举了哪些崩溃状态**：记录段、根槽两段逐个子集全展开（15 + 1），系统配置槽那一段的两次轮换是原地覆写、各取三态
//! （3² − 1 = 8，`crash::TearableInPlaceOverwrites`），加全部持久那一个，共 25 个；
//! 单元段 130 写的 2^130 − 1 个子集枚举不了，它只以整段持久进入后面的状态。单元段里的状态上这次发布的记录一份都没落
//! （记录段在一道屏障之后），恢复见到的与起点镜像相同的那一版加上几份没人点名的单元——那一格已有的流都罩着。
//! **比已有的流多罩了什么**：已有的多记录发布（并行线一的崩溃点重放流）一条记录一个事务、每条都带提交标记；这里一个事务跨两条记录，
//! 第一条不带提交标记，共享的提交内生块分在两条里点名——记录段 15 个状态里：两条都至少落了一份的 8 个（加上根槽那一段的 1 个）
//! 要由记录施加这一版（恢复不许停在不带提交标记的第一条上），只落了其中一条的 6 个要整次不施加（第一条落了而末条没落是半次发布，
//! 末条落了而第一条没落是断号）。多跑的一步只有恢复本身（这条流上没有重开、挂载）。每个状态照常跑两遍恢复 + 多版本 oracle、
//! 池级 checker（I-8.8（前缀里的事务不被切开）、I-8.9（一次发布的记录序号连续且只有末条带标志） 在内）、记录核对器。

mod common;

use common::{
    build_pool, crash_state_devices, file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_verification::crash_amplification::{
    one_test_changes_the_environment_at_a_time, CrashAmplificationEnvironment, ObserveACrashState,
    WeightedShares,
};
use singlefs_verification::crash_library::{
    DisposableLibrary, StageOutcome, TheOneLibraryKeptForThisProcess,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_expansion,
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table, CrashPointReplayItem,
    CrashPointReplayObserver,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_record_spill_over_stream.rs";
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
use singlefs_verification::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::SharedStream;
use std::sync::atomic::{AtomicBool, Ordering};

/// 起点镜像里新池新建文件之后已有的 inode（inode 1）。
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
/// （叶容器落点越过分配记录树叶 61 的末槽时，两块盘各多改一片叶 62：1 片叶容器时 9 个角色、跨过末槽之后 11 个），
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
    // 叶容器数由实现现给的角色数定（试发在 `base` 的拷贝上，不进录制流）：点名的项数比一条记录装得下的多一项。
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
        .expect("点名的项数比一条记录装得下的多一项：末条再跨记录")
    };
    assert_eq!(
        u64::try_from(spilling.rewritten.len()).expect("一次发布的单元数装得进 u64"),
        NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH,
        "{leaf_containers} 片叶容器 + 叶容器之外的角色（由试发现数），恰好是一条记录装得下的项数加一（`JOURNAL_NAMED_ENTRIES_PER_RECORD` + 1）：{:?}",
        spilling.rewritten
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
        (
            1,
            usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD)
                .expect("一条记录的点名项数装得进 usize"),
            1
        ),
        "两条记录：装满一条记录的项数的一条 + 跨出去的 1 项"
    );
    let operations = pool.retained_operations();
    let (writes, segments) = writes_and_segments(&operations[operations_before..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![
            2 * usize::try_from(NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH)
                .expect("点名项数装得进 usize"),
            4,
            1,
            2
        ],
        "只录这一次发布：点名的每个单元 × 2 盘、两条记录 × 2 盘、根槽 FUA、系统配置槽 × 2 盘"
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

impl PublishBoundaryStates {
    /// 从崩溃点重放计数里读回的观察者计数（判段从库里读回、这一趟看的与复用的合起来）。
    fn from_the_observer_counts(counts: &CrashPointReplayObserverCounts) -> Self {
        Self {
            no_record: counts.count_named("no_record"),
            only_one_of_the_two_records: counts.count_named("only_one_of_the_two_records"),
            both_records_without_the_root: counts.count_named("both_records_without_the_root"),
            root_persisted: counts.count_named("root_persisted"),
        }
    }
}

/// 观察者在一个状态上记的数经观察者计数（[`CrashPointReplayObserverCounts`]）带出来：四格各一个名字（[`PublishBoundaryStates::from_the_observer_counts`]）。
fn observe_the_publish_boundary(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let is_persisted = |copies: &[usize]| copies.iter().any(|copy| image.persisted[*copy]);
    let first_record = is_persisted(&prepared.record_writes_in_order[0]);
    let last_record = is_persisted(&prepared.record_writes_in_order[1]);
    let root = image.persisted[prepared.judged_root_index];
    let (expected_root, expected_applied, what) = match (root, first_record, last_record) {
        (true, _, _) => {
            counts.add("root_persisted", 1);
            (
                prepared.version_of_the_spilling_publish,
                0,
                "根槽落了：所选根就是这一版",
            )
        }
        (false, true, true) => {
            counts.add("both_records_without_the_root", 1);
            (
                prepared.version_of_the_spilling_publish,
                2,
                "两条记录都落了、根没落：由记录整次施加这一版",
            )
        }
        (false, true, false) | (false, false, true) => {
            counts.add("only_one_of_the_two_records", 1);
            (
                prepared.version_before,
                0,
                "只落了其中一条：这次发布整体不施加",
            )
        }
        (false, false, false) => {
            counts.add("no_record", 1);
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
    // 批 19 进清单的两条在这条流上前件都在（推的，没量）：I-8.14（新实例第一个 txg 高于更早实例的根与记录）——起点镜像里已有实例 1 的根与记录
    // 和 mkfs 的种子根（实例 0），根环只到 txg 4、没转过圈，每个状态都比得出来，推 25 个状态都判；
    // I-3.20（空闲、已分配、不可回收之和等于容量，已承诺预留等于闭式）——与 I-3.1、I-5.2 同一支，最新根是 txg 3（A）或 txg 4 那一版，
    // 下面都有记账树、树表里有 `deleted_inodes` 条目，推 25 个状态都判。
    for must_evaluate in [
        "I-8.8",
        "I-8.9",
        singlefs_checker::image::FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES,
        singlefs_checker::image::FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM,
        // I-3.21（走到的引用起点是最新根计入的分配记录的起点，C944）与 I-3.20 同一类：最新根的树表带分配记录树（有记账树的那几版都带）就判，
        // 树表 0 条的种子根、暖机根上报不适用（推的，没量；判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q2 新立）。
        singlefs_checker::image::WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
    ] {
        singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0),
            &format!("{must_evaluate} 至少在一个状态上真被评估过：{}", tally.checker_counts_by_invariant()),
        );
    }
    // 批 19 进清单的另两条在这条流上前件不在，一个状态都评估不到（推的，没量）：
    // I-3.19（实例表行的 W 是一次恢复施加得出的最大事务号） 逐行判最新根实例表里的行，这条流从 mkfs 起一直是实例 1、没有重开挂载，实例表里一行都没有；
    // I-5.7（由记录施加出来的一版的分配记录树根出生在这一版） 只判树表 0 条那一版上写过单元的发布：两条记录都落而根没落的状态上下一次挂载先施加的是
    // 建 inode 那一版，它的树表有条目、新根段里那条指针全零，不判。
    for evaluated_on_no_state in [
        singlefs_checker::image::INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED,
        singlefs_checker::image::ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION,
    ] {
        assert_eq!(
            tally
                .checker_evaluated_states
                .get(evaluated_on_no_state)
                .copied()
                .unwrap_or(0),
            0,
            "{evaluated_on_no_state} 在这条流上一个状态都评估不到（前件不在）：{}",
            tally.checker_counts_by_invariant()
        );
    }
}

/// L8 能枚举的全部状态：记录段（4 写）、根槽（1 写）逐个子集全展开，系统配置槽（2 写，原地覆写各取三态）全展开，
/// 加全部持久那一个，共 25 个；
/// 单元段 130 写不展开（2^130 − 1 个子集，只以整段持久进入后面的状态）。每个状态按两条记录落没落核发布边界。
#[test]
fn every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all(
) {
    // 同一个二进制里另一条用例先设一次性库、再拿环境守卫设两台分片：这一条整趟先拿全仓那一份的守卫（它设不了一次性库）、再拿环境锁（读不到它设的分片），
    // 只拿环境锁挡不住它先设上的一次性库
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
    let prepared = prepare("spill-over-crash-point-replay");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 64;
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_publish_boundary(&prepared, image, report, counts);
    };
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "record_spill_over::two_records_whole_or_not_at_all",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&prepared.versions),
        &|segment_index, segment| {
            if expand(segment_index, segment) {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        },
        &[CrashPointReplayObserver {
            name: "publish_boundary",
            // 观察者另读的输入：两条记录的写表下标、被判的根槽下标、这次发布与它之前那一版的根
            extra_inputs_digest: extra_inputs_digest_that_was_computed(singlefs_verification::sha256::sha256_digest(
                format!(
                    "record_writes_in_order={:?}\njudged_root_index={}\nversion_before={:?}\nversion_of_the_spilling_publish={:?}\n",
                    prepared.record_writes_in_order,
                    prepared.judged_root_index,
                    prepared.version_before,
                    prepared.version_of_the_spilling_publish
                )
                .as_bytes(),
            )),
            observe: &observe,
        }],
    )
    .judged() else {
        return;
    };
    let counts = PublishBoundaryStates::from_the_observer_counts(&tally.observer_counts);
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &|segment_index, segment| {
                if expand(segment_index, segment) {
                    CrashPointReplaySegmentExpansion::EveryProperSubset
                } else {
                    CrashPointReplaySegmentExpansion::NotExpanded
                }
            },
        ),
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）",
    );
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        1u64 + ((1 << 4) - 1) + ((1 << 1) - 1) + (3 * 3 - 1),
        "记录段 15、根槽 1、系统配置槽 3² − 1（两次轮换各取三态；每次写只取两态时 3）、全部持久 1",
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        counts,
        PublishBoundaryStates {
            no_record: 1,
            only_one_of_the_two_records: 6,
            both_records_without_the_root: 9,
            root_persisted: 1 + (3 * 3 - 1),
        },
        "记录段 15 个状态：一条都没落 1、只落一条 3 + 3、两条都落 8；根槽那一段 1 个两条都落；\
         之后 9 个根槽已落（系统配置槽那一段 3² − 1 + 全部持久 1）",
    );
    println!(
        "CRASH_POINT_REPLAY_SPILL_OVER states={} publish_boundary={counts:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
}

/// L8 同一批 25 个状态（展开同 [`every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all`]）
/// 改在 CPU 判器上逐状态判：取样那一支（`sample_crash_states_on_the_cpu_with_threads_against_the_version_table`）不经选路，多机配置开不开 GPU、
/// 编没编进 gpu 特性都在 CPU 上判，每个状态两遍 singlefs-core 的恢复、oracle、池级 checker、记录核对器；每个状态再拿看 journal 那一遍恢复的报告
/// 交同一个发布边界观察者（[`observe_the_publish_boundary`]）核该走哪一版的根、施加几条。
/// 枚举那一条在门禁开 GPU 时判段走 GPU 判器（恢复是照 singlefs-core 另写的 GPU 内核），观察者的结论又从库里复用，
/// singlefs-core 的恢复与前缀核在 CPU 上它跑不到；变异表里改坏那几处的行点名这一条。线程数取每台默认重活线程数。
/// 判别力（`crates/mutations.tsv` 里 L8 那两行：恢复在不带提交标记的记录上就停、恢复把不带末条标志的记录也当末条）：
/// 两条记录都落、根没落的状态上一条都不施加，或只落了第一条的状态上半次发布施加上去，观察者那一条 `assert_eq!` 在判定线程里 panic，这里红。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn a_publish_whose_one_transaction_spills_over_two_records_judged_on_the_cpu_judge_applies_it_whole_or_not_at_all(
) {
    // 同一个二进制里另几条用例设一次性库与两台分片：照枚举那一条先拿全仓那一份的守卫、再拿环境锁
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
    let prepared = prepare("spill-over-crash-point-replay-on-the-cpu-judge");
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() < 64 {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let state_count = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &expansion,
    );
    let counts_of_the_flow: std::sync::Mutex<CrashPointReplayObserverCounts> =
        std::sync::Mutex::new(CrashPointReplayObserverCounts::default());
    let Some(states_of_the_flow) =
        sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
            CrashPointReplayItem {
                name: "record_spill_over::two_records_whole_or_not_at_all_on_the_cpu_judge",
                setting_file: THIS_FILE,
            },
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            prepared.judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&prepared.versions),
            &expansion,
            0..state_count,
            singlefs_verification::heavy_threads::default_heavy_threads(),
            &|_ordinal, image, judgement| {
                let mut counts_of_the_state = CrashPointReplayObserverCounts::default();
                observe_the_publish_boundary(
                    &prepared,
                    image,
                    &judgement.consulted,
                    &mut counts_of_the_state,
                );
                let mut counts_of_the_flow = counts_of_the_flow.lock().expect("判定线程不 panic");
                for (name, amount) in counts_of_the_state.iter() {
                    counts_of_the_flow.add(name, amount);
                }
            },
        )
        .judged()
    else {
        return;
    };
    assert_eq!(
        states_of_the_flow,
        1 + ((1 << 4) - 1) + ((1 << 1) - 1) + (3 * 3 - 1),
        "记录段 15、根槽 1、系统配置槽 3² − 1、全部持久 1"
    );
    let counts_of_the_flow = counts_of_the_flow.into_inner().expect("判定线程不 panic");
    assert_eq!(
        PublishBoundaryStates::from_the_observer_counts(&counts_of_the_flow),
        PublishBoundaryStates {
            no_record: 1,
            only_one_of_the_two_records: 6,
            both_records_without_the_root: 9,
            root_persisted: 1 + (3 * 3 - 1),
        },
        "CPU 判器上每个状态都交发布边界观察者看过一次，四格的数与枚举那一条钉的相同"
    );
}

/// 两台分活的权重（本机、第二台），与双机四条用例那一趟的多机配置相同。
const HOST_WEIGHTS: &str = "2,3";

/// 发布边界观察者（[`observe_the_publish_boundary`]），另读的输入摘要照一句说明取：观察者的代码不动，版本随这句说明变。
fn publish_boundary_observer_under_the_extra_inputs<'observer>(
    extra_inputs_label: &[u8],
    observe: &'observer ObserveACrashState<'observer>,
) -> CrashPointReplayObserver<'observer> {
    // 不与 L8 那一条的写法同形：变异表里「另读输入摘要写回全零」那一行的原文要在这份文件里恰好命中一次
    let extra_inputs_digest = extra_inputs_digest_that_was_computed(
        singlefs_verification::sha256::sha256_digest(extra_inputs_label),
    );
    CrashPointReplayObserver {
        name: "publish_boundary",
        extra_inputs_digest,
        observe,
    }
}

/// 在 `library` 那一份一次性库里跑一趟这次发布（展开同 L8 那一条），挂上 `observer`；`share` 给了就按两台分活跑这一份（权重 [`HOST_WEIGHTS`]）。
/// 两台各领的那一份都直接写进同一份库（没有取回、导入这一步）。同一个二进制里改进程环境的用例一条接一条跑（一次性库设成这个进程开的那一份，
/// 环境守卫里设分片）。
fn enumerate_the_spilling_publish_in_the_library(
    prepared: &Prepared,
    library: &DisposableLibrary,
    share: Option<&str>,
    observer: CrashPointReplayObserver<'_>,
) -> StageOutcome<CrashPointReplayTally> {
    let _the_library_of_this_process = library.make_it_the_library_of_this_process();
    let mut settings = Vec::new();
    if let Some(share) = share {
        settings.push((
            "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
            std::ffi::OsStr::new(share),
        ));
        settings.push((
            "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
            std::ffi::OsStr::new(HOST_WEIGHTS),
        ));
    }
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
    // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
    environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
    let outcome = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "record_spill_over::share_with_every_judge_conclusion_reused",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&prepared.versions),
        &|_segment_index, segment| {
            if segment.len() < 64 {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        },
        &[observer],
    );
    drop(environment);
    outcome
}

/// 双机的一份判器结论全复用、观察者这一趟换了版本（双机四条用例两份都 panic 在重建计数读不到对方那几块的观察者结论的那一格）。
/// 一次性库里：先按两台分活跑本机那一份（库是空的，判器结论缺第二台那几块，照旧只交自己那一份：记下两份各领几块）；再整份跑一趟把库判满；
/// 换一份观察者另读的输入摘要（观察者的版本变了、代码不动）再跑本机那一份：判器结论全复用（这一份现判 0 块），观察者只判这一份领的块，
/// 第二台领的块缺这一把版本的观察者结论，这一份只交自己那一份、留下的块数等于第二台领的块数，不去重建整条流的计数；
/// 最后跑第二台那一份、同一把观察者版本：它的块判完之后整条流两种结论都齐，交整份的结果、每个状态看过一次，与判满库那一趟的计数相同
/// （本机那一份领的块上一趟确实存下了这一把版本的观察者结论，不然这一趟同样只交自己那一份）。
/// 判别力（`crates/mutations.tsv`）：收工的条件去掉观察者那一半（改回只看判器结论）、缺观察者结论那一问不问库、留下的块数只数缺判器结论的，
/// 各让这一条红。
#[test]
fn a_share_whose_judge_conclusions_are_all_reused_leaves_the_blocks_missing_the_new_observer_conclusions_to_the_other_host(
) {
    let prepared = prepare("spill-over-share-with-every-judge-conclusion-reused");
    // 一次性库留着：钉两台各领哪几块、观察者换版本之后留给另一台的块数，要从空库数账
    let library = DisposableLibrary::new(
        "crash-enumeration-spill-over-share-with-every-judge-conclusion-reused",
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_publish_boundary(&prepared, image, report, counts);
    };
    let observer_of_the_whole_run = || {
        publish_boundary_observer_under_the_extra_inputs(
            b"publish boundary observer of the full run",
            &observe,
        )
    };
    let observer_of_another_version = || {
        publish_boundary_observer_under_the_extra_inputs(
            b"publish boundary observer under other extra inputs",
            &observe,
        )
    };
    let (blocks_of_this_host, blocks_of_the_other_host) =
        match enumerate_the_spilling_publish_in_the_library(
            &prepared,
            &library,
            Some("0/2"),
            observer_of_the_whole_run(),
        ) {
            StageOutcome::JudgedMyShareOnly {
                blocks_judged_here,
                blocks_left_to_other_hosts,
            } => (blocks_judged_here, blocks_left_to_other_hosts),
            // 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑，这一条只在带特性那一趟判
            StageOutcome::RecordedOnly => return,
            StageOutcome::Judged(tally) => panic!(
                "前提：空库上本机那一份判不全第二台那几块，只交自己那一份（交了整份 {} 个状态）",
                tally.states
            ),
        };
    let block_count = blocks_of_this_host + blocks_of_the_other_host;
    let other_host_owns = WeightedShares::new(
        HOST_WEIGHTS
            .split(',')
            .map(|weight| weight.parse().expect("权重是十进制整数"))
            .collect(),
    );
    assert_eq!(
        blocks_of_the_other_host,
        u64::try_from(
            (0..block_count)
                .filter(|block| other_host_owns.owner_of_block(*block) == 1)
                .count()
        )
        .expect("块数装得进 u64"),
        "前提：空库上本机那一份留下的就是按权重 {HOST_WEIGHTS} 第二台领的块"
    );
    // 块数随这一趟的域：缩过域的块少，可能一台一块都领不到
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        blocks_of_this_host,
        &format!(
            "前提：本机领到块（本机 {blocks_of_this_host}、第二台 {blocks_of_the_other_host}）"
        ),
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
        blocks_of_the_other_host,
        &format!(
            "前提：第二台领到块（本机 {blocks_of_this_host}、第二台 {blocks_of_the_other_host}）"
        ),
    );
    let whole_run = enumerate_the_spilling_publish_in_the_library(
        &prepared,
        &library,
        None,
        observer_of_the_whole_run(),
    )
    .judged()
    .expect("整份跑一趟：第二台那几块现判，库判满，交整份的结果");
    assert_eq!(
        whole_run.observed_states, whole_run.states,
        "前提：判满库那一趟每个状态看过一次"
    );
    match enumerate_the_spilling_publish_in_the_library(
        &prepared,
        &library,
        Some("0/2"),
                observer_of_another_version(),
    ) {
        StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        } => assert_eq!(
            (blocks_judged_here, blocks_left_to_other_hosts),
            (0, blocks_of_the_other_host),
            "判器结论全复用（现判 0 块）；留下的是第二台领的、缺这一把版本观察者结论的块"
        ),
        StageOutcome::RecordedOnly => panic!("同一份二进制、同一份配置：前两趟判了，这一趟也判"),
        StageOutcome::Judged(tally) => panic!(
            "本机那一份不许重建整条流的计数（第二台那几块缺这一把版本的观察者结论）：交了整份 {} 个状态",
            tally.states
        ),
    }
    let other_host_share = enumerate_the_spilling_publish_in_the_library(
        &prepared,
        &library,
        Some("1/2"),
        observer_of_another_version(),
    )
    .judged()
    .expect("第二台那一份判完自己的块，整条流两种结论都齐：交整份的结果");
    assert_eq!(
        (
            other_host_share.states,
            other_host_share.observed_states,
            PublishBoundaryStates::from_the_observer_counts(&other_host_share.observer_counts),
        ),
        (
            whole_run.states,
            whole_run.states,
            PublishBoundaryStates::from_the_observer_counts(&whole_run.observer_counts),
        ),
        "本机那一份领的块上一趟存下了这一把版本的观察者结论：每个状态看过一次，计数与判满库那一趟相同"
    );
    drop(library);
}

/// panic 带的那一句（`panic!`、`assert!` 的消息是字符串）；带的不是字符串时写一句说明。
fn panic_message_of(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "panic 带的不是字符串".to_string())
}

/// 双机那两份里第二台领的块上观察者判红（D1：第二台那一份在收工之前不报自己那一半的观察者红，收尾那一趟只核本机领的块，
/// 两边都不报）。一份一次性库（两台都直接写它），观察者是发布边界那一个、另读的输入相同（两份的观察者版本相同），只在第二台那一份那一趟
/// 每个状态都判红（开关在测试里，不进观察者版本）。
/// 先跑第二台那一份（1/2，库是空的）：它自己领的块上观察者红，这一趟当场红、消息点名观察者（不收工交自己那一份）；
/// 再跑本机那一份（0/2）：本机领的块上观察者干净，第二台领的块上的观察者红只在第二台写进库的结论里，整条流两种结论都齐、重建计数时读到，
/// 这一趟照样红、消息点名观察者（只交整份的结果会是 `ok`）。
/// 判别力（`crates/mutations.tsv`）：分片那一份不报自己领的块上的观察者红（留到收尾那一趟）、重建计数时累加观察者红状态数那一步加 0，各让这一条红。
#[test]
fn an_observer_red_on_the_blocks_the_other_host_owns_reddens_both_its_share_and_this_hosts_share() {
    let prepared = prepare("spill-over-observer-red-on-the-other-host");
    // 一次性库留着：测试里的开关让观察者判红，红结论不许进全仓那一份
    let library =
        DisposableLibrary::new("crash-enumeration-spill-over-observer-red-on-the-other-host")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let every_state_is_red_in_this_run = AtomicBool::new(false);
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_publish_boundary(&prepared, image, report, counts);
        assert!(
            !every_state_is_red_in_this_run.load(Ordering::SeqCst),
            "单测造的观察者红：第二台那一份那一趟每个状态都判红"
        );
    };
    let observer = || {
        publish_boundary_observer_under_the_extra_inputs(
            b"publish boundary observer reddening the blocks of the other host",
            &observe,
        )
    };
    every_state_is_red_in_this_run.store(true, Ordering::SeqCst);
    let other_host_run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        enumerate_the_spilling_publish_in_the_library(&prepared, &library, Some("1/2"), observer())
    }));
    every_state_is_red_in_this_run.store(false, Ordering::SeqCst);
    let other_host_message = match other_host_run {
        Err(payload) => panic_message_of(payload.as_ref()),
        // 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑，这一条只在带特性那一趟判
        Ok(StageOutcome::RecordedOnly) => return,
        Ok(StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        }) => panic!(
            "第二台那一份自己领的块上观察者红了，要当场红，不收工交自己那一份（判了 {blocks_judged_here} 块、留下 {blocks_left_to_other_hosts} 块）"
        ),
        Ok(StageOutcome::Judged(tally)) => panic!(
            "前提：空库上第二台那一份判不全本机那几块，交不了整份（交了整份 {} 个状态）",
            tally.states
        ),
    };
    assert!(
        other_host_message.contains("publish_boundary"),
        "第二台那一份当场红、消息点名观察者：{other_host_message}"
    );
    let this_host_run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        enumerate_the_spilling_publish_in_the_library(&prepared, &library, Some("0/2"), observer())
    }));
    let this_host_message = match this_host_run {
        Err(payload) => panic_message_of(payload.as_ref()),
        Ok(StageOutcome::RecordedOnly) => {
            panic!("同一份二进制、同一份配置：第二台那一份判了，本机那一份也判")
        }
        Ok(StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        }) => panic!(
            "第二台那一份判完、本机这一份判完之后整条流两种结论都齐，不收工（判了 {blocks_judged_here} 块、留下 {blocks_left_to_other_hosts} 块）"
        ),
        Ok(StageOutcome::Judged(tally)) => panic!(
            "本机那一份读到了第二台领的块上存着的观察者红，要红；交了整份 {} 个状态",
            tally.states
        ),
    };
    assert!(
        this_host_message.contains("观察者 publish_boundary"),
        "本机那一份红、消息点名观察者：{this_host_message}"
    );
    drop(library);
}

/// 观察者另读的输入的摘要要真算：全零就是没算（另读输入不同的两趟会共用看过的块）。
fn extra_inputs_digest_that_was_computed(
    digest: [u8; singlefs_verification::sha256::DIGEST_BYTES],
) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
    assert_ne!(
        digest,
        [0; singlefs_verification::sha256::DIGEST_BYTES],
        "观察者另读的输入的摘要真算（全零就是没算：另读输入不同的两趟会共用看过的块）"
    );
    digest
}
