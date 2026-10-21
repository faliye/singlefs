//! checker 档模块：crash、crash_state_enumeration、crash_identity、crash_amplification
//! 里程碑「覆盖写、释放、回退与复用」并行线一的崩溃点重放小负载（验收第 3 条）：多条记录的发布边界在小单元数上造出来——
//! 「取号 → 暖机 × 2 → A（第一个文件，一个数据单元：12 个单元）→ B（顺序写两个数据单元：14 个单元、两条记录）
//! → C（顺序写三个数据单元：15 个单元、三条记录）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，
//! 与新池新建文件同一种切法（C577 之后上一次发布的系统配置槽轮换过了发布末尾那道屏障、自成一段；改之前与下一次发布的单元写落在同一段）。每个状态跑恢复 + 多版本 oracle、
//! 池级 checker、记录核对器；另逐状态核发布边界（D23（journal 的角色与格式） 已定项 14 第六条）：
//! 一次发布的第一条记录落了而末条一份都没落 ⇒ 那次发布整体不施加，读回上一版、文件是旧长度（I-4.3（提交原子））。
//!
//! **流按今天的形状钉：B 14 个单元、C 15 个，验收第 3 条写的「一次发布的单元数不超过 10」这条流做不到。**
//! 分配记录树按位置寻址（D8（核心索引结构） 已定项 14），4 GiB × 2 的池上一次发布带着它的五个节点（两盘各叶 61、两盘各第 1 层、根），
//! 加上 extent 根、inode 叶容器与根、记账树、中央映射树、树表，数据单元之外的固定开销是 11 个单元，两个数据单元起 extent
//! 多一片下段节点、12 个。一次发布的记录条数等于它的数据单元数，要两条记录就至少两个数据单元：B 最少 14 个、C（三条记录）最少 15 个，
//! 挑内容大小压不下来（按内容大小逐档量的数在 `research/prompts/m2-rev-b3a3c-implementer-report.md` 第五节）。全量照这个形状跑。
//!
//! **比已有的流多罩了什么**：已有的流上每次发布恰一条记录，记录段恒是 2 写；这里 B、C 的记录段是 4 写与 6 写，
//! 崩在一次发布的记录之间的状态（B 3 个、C 12 个）第一次被枚举到；一次发布写两个、三个数据单元，
//! extent 下段一片根兼叶装两条、三条记录（上段那条条目换成下段根指针，D8（核心索引结构） 已定项 14），
//! 中央映射里两条、三条码 1 条目。多跑的一步只有恢复本身（这条流上没有重开、挂载）。
//!
//! 快的那条只展开写数小于 10 的段（A、B、C 三个单元段 24 写、28 写与 30 写不展开，只以整段持久进入后面的状态）；
//! 全量标 ignored（崩溃点重放的枚举域 1358954642 个状态，B、C 两个单元段占 1342177278；每次写只取两态时的闭式 1358954607、
//! 两段同样占 1342177278；C577 之前轮换并进单元段，12230590578 与 5435818083），登记成崩溃枚举用例，门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片（与新池新建文件流、覆盖写固定脚本流同一个入口）。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;

use common::{build_pool, file_content, geometry};
use common_crash_points::{content_needing, sequential_write};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_selecting_versions, enumerate_crash_states_with_expansion,
    CrashPointReplayItem, CrashPointReplayObserver,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs";
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::record_offset;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 发布边界那个观察者另读的输入（每次多条记录的发布：记录写的下标、它与它之前那一版的根、之前那一版的内容）的摘要：进观察者的版本，
/// 录制流一变它就变，看过的块不按旧的复用。
fn multi_record_publishes_digest(
    publishes: &[MultiRecordPublish],
) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    let mut text = String::new();
    for publish in publishes {
        text.push_str(&format!(
            "record_writes_in_order={:?} version={:?} version_before={:?} content_before={:?}\n",
            publish.record_writes_in_order,
            publish.version,
            publish.version_before,
            publish.content_before
        ));
    }
    singlefs_harness::sha256::sha256_digest(text.as_bytes())
}

/// 一次多条记录的发布在写表里的样子：它每一条记录的两份写各在写表的哪一格，它的根在哪一版、上一版是哪一版。
struct MultiRecordPublish {
    /// 这次发布的记录按 jsn 升序，每条两盘各一份：写表下标。
    record_writes_in_order: Vec<Vec<usize>>,
    version: (InstanceGeneration, CheckpointTxg),
    version_before: (InstanceGeneration, CheckpointTxg),
    content_before: Vec<u8>,
}

impl MultiRecordPublish {
    fn is_any_copy_persisted(persisted: &[bool], copies: &[usize]) -> bool {
        copies.iter().any(|copy| persisted[*copy])
    }

    /// 这一状态里这次发布的第一条记录至少落了一份，而末条一份都没落：前缀停在这次发布中间。
    fn is_cut_before_its_last_record(&self, persisted: &[bool]) -> bool {
        let first = self.record_writes_in_order.first().expect("至少两条记录");
        let last = self.record_writes_in_order.last().expect("至少两条记录");
        Self::is_any_copy_persisted(persisted, first)
            && !Self::is_any_copy_persisted(persisted, last)
    }
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    multi_record_publishes: Vec<MultiRecordPublish>,
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

fn multi_record_publish(
    writes: &[RetainedWrite],
    output: &TransactionOutput,
    before: &TransactionOutput,
    content_before: Vec<u8>,
) -> MultiRecordPublish {
    let mut record_writes_in_order: Vec<Vec<usize>> = output
        .earlier_records_of_this_publish
        .iter()
        .map(|earlier| writes_to_record(writes, earlier.record.counter))
        .collect();
    record_writes_in_order.push(writes_to_record(writes, output.record.counter));
    for copies in &record_writes_in_order {
        assert_eq!(copies.len(), 2, "每条记录两盘各一份");
    }
    MultiRecordPublish {
        record_writes_in_order,
        version: (output.root.instance, output.root.checkpoint_txg),
        version_before: (before.root.instance, before.root.checkpoint_txg),
        content_before,
    }
}

fn prepare(tag: &str) -> Prepared {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    assert_eq!(
        second.earlier_records_of_this_publish.len() + 1,
        2,
        "B：两个数据单元两条记录"
    );
    assert_eq!(
        second.rewritten.len(),
        2 + 2 + 2 + 5 + 3,
        "B：两个数据单元 + extent 下段根兼叶与上段根 + inode 叶容器与根 + 分配记录树五个节点 + 记账树、中央映射树、树表 = 14 个单元\
         （验收第 3 条的「≤ 10」今天不成立，见文件头）：{:?}",
        second.rewritten
    );
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    assert_eq!(
        third.earlier_records_of_this_publish.len() + 1,
        3,
        "C：三个数据单元三条记录"
    );
    assert_eq!(
        third.rewritten.len(),
        3 + 2 + 2 + 5 + 3,
        "C：三个数据单元 + 与 B 同样的 12 个单元 = 15 个单元（「≤ 10」今天不成立，见文件头）：{:?}",
        third.rewritten
    );

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2],
        "取号两轮各 2、暖机两次、上一次的系统配置槽轮换 2 与 A 的 24 个单元写各自成一段；A 的轮换 2、B 的 28 个单元写、两条记录 4 写；\
         B 的轮换 2、C 的 30 个单元写、三条记录 6 写（C577 之后轮换之后有屏障，改之前轮换并进下一次的单元段：26、30、32）"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 5, "暖机两次、A、B、C 各一条根槽写");
    let versions = vec![
        PublishedVersion {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: second.root.instance,
            checkpoint_txg: second.root.checkpoint_txg,
            content: second_content.clone(),
        },
        PublishedVersion {
            instance: third.root.instance,
            checkpoint_txg: third.root.checkpoint_txg,
            content: third_content,
        },
    ];
    let multi_record_publishes = vec![
        multi_record_publish(&writes, &second, &first, file_content()),
        multi_record_publish(&writes, &third, &second, second_content),
    ];
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        versions,
        multi_record_publishes,
    }
}

/// 每个状态上核发布边界：一次发布的第一条记录落了、末条没落 ⇒ 实际走的根不是这次发布的，读回上一版、文件是旧长度。
/// 这一条 oracle 判不出来：崩溃点重放按屏障切段，记录段里的状态上这次发布的单元全都落了，半次发布施加上去读回的也是完整的新内容。
fn observe_the_publish_boundary(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    for (publish_index, publish) in prepared.multi_record_publishes.iter().enumerate() {
        if !publish.is_cut_before_its_last_record(&image.persisted) {
            continue;
        }
        counts.add(&format!("cut_in_multi_record_publish_{publish_index}"), 1);
        assert_eq!(
            report.effective_root,
            Some(publish.version_before),
            "第 {} 次多记录发布的末条没落：那次发布整体不施加，走的是上一版的根（{:?}）",
            publish_index,
            publish.version
        );
        let RecoveryOutcome::FileRead { content, .. } = &report.outcome else {
            panic!("要读回上一版，实际 {:?}", report.outcome);
        };
        assert!(
            *content == publish.content_before,
            "读回上一版：文件是旧长度 {} 字节，实际 {} 字节",
            publish.content_before.len(),
            content.len()
        );
    }
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
}

/// 平时跑的那一份：A、B、C 三个单元段（24 写、28 写、30 写）不展开，其余每段任意子集——B 的两条记录段（4 写）、
/// C 的三条记录段（6 写）全展开，崩在一次发布的记录之间的每个状态都跑到。
#[test]
fn every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it() {
    let prepared = prepare("multi-record-publish-crash-point-replay-fast");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_publish_boundary(&prepared, image, report, counts);
    };
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "multi_record_publish::between_the_records",
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
            extra_inputs_digest: extra_inputs_digest_that_was_computed(
                multi_record_publishes_digest(&prepared.multi_record_publishes),
            ),
            observe: &observe,
        }],
    )
    .judged() else {
        return;
    };
    let cut_states_by_publish: Vec<u64> = (0..prepared.multi_record_publishes.len())
        .map(|publish_index| {
            tally
                .observer_counts
                .count_named(&format!("cut_in_multi_record_publish_{publish_index}"))
        })
        .collect();
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 10)
        .cloned()
        .collect();
    assert_eq!(
        closed_form_state_count(&expanded),
        114,
        "每次写只取两态时展开的段按闭式数（补第三态之前的口径）：1 + 十个 2 写段各 3 + 五个 1 写段各 1 + 4 写段 15 + 6 写段 63\
         （取号写一轮时九个 2 写段、111；C577 之前暖机第二次、A、B 的轮换并在单元段里不展开，六个 2 写段、102）"
    );
    assert_eq!(
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
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_eq!(
        tally.states,
        1 + 7 * (3 * 3 - 1) + 3 * 3 + 5 + 15 + 63,
        "1 + 七个系统配置槽 2 写段（取号两轮、五次轮换）各 3² − 1 + 三个记录 2 写段各 3 + 五个 1 写段各 1 + 4 写段 15 + 6 写段 63（每次写只取两态时 114；\
         取号写一轮时六个系统配置槽段、141；C577 之前三个系统配置槽段、117）"
    );
    // B：记录段 4 写里第一条落了（至少一份）、第二条一份没落 ⇒ 3 × 1 = 3 个状态。
    // C：记录段 6 写里第一条落了、第三条一份没落 ⇒ 第一条 3 种 × 第二条 4 种 = 12 个状态。
    assert_eq!(
        cut_states_by_publish,
        vec![3, 12],
        "崩在一次发布的记录之间的状态真跑到了"
    );
    println!(
        "CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH_FAST states={} cut_states_by_publish={cut_states_by_publish:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
}

/// 全量：20 段、闭式见下。A、B、C 三个单元段各 2^24 − 1、2^28 − 1 与 2^30 − 1 个状态（C577 之前各带上一次发布的两次系统配置槽轮换、
/// 取三态，各 3² · 2^n − 1，共 16 段、12230590578），
/// 每个状态两遍恢复 + checker。登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv`），checker-tier-crash-point-replay 认下面打印的
/// `CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH` 行里 `exhaustive=true`。判过的状态都在判定库里（崩溃枚举用例那一类，`crash_state_enumeration` 的两段：流先录进库，判段从库读回判，被杀之后下一趟接着判没判过的）。
/// 双机（`scripts/crash/crash-point-replay-shard-run.sh` 经崩溃放量的双机驱动）：两台各判自己那一份，只录或只判自己那一份的那一趟不打计数行、不判下面的断言；
/// 导入第二台的库之后再跑的那一趟拿整条流的计数照下面逐项判、逐字打同样的行。
#[test]
#[ignore = "全量 1358954642 个状态（十三亿多）、每个两遍恢复 + checker；门禁 checker-tier-crash-point-replay --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_multi_record_publish_stream_is_exhaustive_and_clean() {
    let prepared = prepare("multi-record-publish-crash-point-replay-full");
    let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
    assert_eq!(
        closed_form_with_two_states_per_write,
        1 + 10 * 3 + 5 + ((1 << 24) - 1) + ((1 << 28) - 1) + 15 + ((1 << 30) - 1) + 63,
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，二十段"
    );
    assert_eq!(closed_form_with_two_states_per_write, 1_358_954_607);
    // 计数行的 closed_form 报枚举域的闭式（原地覆写三态）：七个系统配置槽 2 写段各 3² − 1，每段 3^m · 2^(n−m) − 1。
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 7 * (3 * 3 - 1)
            + 3 * 3
            + 5
            + ((1 << 24) - 1)
            + ((1 << 28) - 1)
            + 15
            + ((1 << 30) - 1)
            + 63,
        "枚举域的闭式：七个只有系统配置槽写的 2 写段（取号两轮、五次轮换）各 3² − 1、三个记录 2 写段各 3、五个 1 写段、A / B / C 的单元段各 2^n − 1、4 写段 15、6 写段 63"
    );
    assert_eq!(closed_form, 1_358_954_642);
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "multi_record_publish::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&prepared.versions),
        &full_expansion,
        &[],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH states={} closed_form={closed_form} exhaustive={} violations={} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form);
    assert_clean(&tally);
}

/// 录制器那一份造法录下的多记录流：起点镜像、写表、段、崩溃点、被判的根槽写（最后一条）、版本表。
#[cfg(feature = "verdict-store")]
struct RecordedThroughTheCrashPointRecorder {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 录制器那一份造法（前缀节点与别的流共享：取号加暖机、新池新建文件两个点由公共模块设，B、C 两次顺序写也由公共模块设，
/// 删除流共享到 C，崩溃点重放合并第三轮判决 J3）。
#[cfg(feature = "verdict-store")]
fn record_through_the_crash_point_recorder(tag: &str) -> RecordedThroughTheCrashPointRecorder {
    use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    let publishes =
        common_crash_points::record_the_multi_record_publishes(&mut recorder, &mut pool);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) =
        singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(
            &operations[pool.mkfs_operation_count..],
            &geometry(),
        );
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedThroughTheCrashPointRecorder {
        base,
        writes,
        segments,
        crash_points,
        judged_root_index,
        versions: publishes.versions(),
    }
}

/// 崩溃放量的形态：按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判、CPU 判器同时对照。展开上限默认 6
/// （每段真子集全枚举时状态数 2^n − 1）。这条流整条走在标准前缀上：登记表挂在标准前缀节点上的观察者（可写挂载续做）照表跑，
/// 它另读的删除流照录（崩溃点重放合并第三轮判决 J4、M4 第 10 条），观察者计数的断言每一趟都判。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_multi_record_publish_stream_is_clean() {
    let recorded = record_through_the_crash_point_recorder("crash-amplification-multi-record");
    common_crash_point_observers::amplify_a_flow_through_the_standard_prefix(
        "multi-record-publish",
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &[],
        recorded.crash_points,
        6,
    );
}

/// 两份造法写表摘要相等（崩溃点重放合并第三轮判决 J3，第一轮判决 K2-2）：崩溃点重放那一份（[`prepare`]，不录崩溃点）与录制器那一份
/// （B、C 两次顺序写挪进公共模块之后）录下的起点镜像与整张带屏障的写表逐字节相同、段一样切，被判的根槽写是同一条、版本表相同。
/// 两份都调公共模块里同一个 `sequential_write`；崩溃点重放那一份的写法与挪之前逐字相同，所以这条也核「挪进公共模块之后写表与今天相同」。
/// 摘要打一行 `CRASH_POINT_REPLAY_WRITE_TABLE`，跑过一次之后可以钉成常量。
#[cfg(feature = "verdict-store")]
#[test]
fn the_multi_record_publish_stream_writes_the_same_table_through_the_crash_point_recorder_as_without_it(
) {
    use singlefs_checker_tier::crash_identity::{step_write_table_with_barriers, SectorImage};
    let without_the_recorder = prepare("multi-record-write-table-without-the-recorder");
    let through_the_recorder =
        record_through_the_crash_point_recorder("multi-record-write-table-through-the-recorder");
    let digests_of = |base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>]| {
        (
            SectorImage::of_base(base).digest(),
            step_write_table_with_barriers(writes, segments, 0..writes.len()),
        )
    };
    let (base_digest, write_table_digest) = digests_of(
        &without_the_recorder.base,
        &without_the_recorder.writes,
        &without_the_recorder.segments,
    );
    assert_eq!(
        digests_of(
            &through_the_recorder.base,
            &through_the_recorder.writes,
            &through_the_recorder.segments
        ),
        (base_digest, write_table_digest),
        "两份造法的起点镜像与整张带屏障的写表逐字节相同"
    );
    assert_eq!(
        through_the_recorder.segments, without_the_recorder.segments,
        "段一样切"
    );
    assert_eq!(
        through_the_recorder.judged_root_index, without_the_recorder.judged_root_index,
        "被判的根槽写都是最后一条"
    );
    assert_eq!(
        through_the_recorder.versions, without_the_recorder.versions,
        "版本表相同"
    );
    let hexadecimal =
        |digest: &[u8]| -> String { digest.iter().map(|byte| format!("{byte:02x}")).collect() };
    println!(
        "CRASH_POINT_REPLAY_WRITE_TABLE flow=multi-record-publish base_digest={} write_table_digest={}",
        hexadecimal(&base_digest),
        hexadecimal(&write_table_digest)
    );
}

/// 「根槽已持久」的计数事后数（崩溃点重放合并第三轮判决 J3 的 M2-2 甲、J7）：被判的根槽写不进块键之后，计数按计划的段现算
/// （`crash_amplification::states_with_the_root_slot_write_persisted`），与崩溃点重放逐状态数的（`CrashPointReplayTally::root_persisted_states`）
/// 在快档那个域（写数小于 10 的段全展开）上相同；被判根换成第一条根槽写也相同。
#[cfg(feature = "verdict-store")]
#[test]
fn the_root_slot_persisted_count_computed_from_the_plan_equals_the_one_counted_state_by_state() {
    use singlefs_checker_tier::crash_amplification::{
        plan_crash_points, states_with_the_root_slot_write_persisted, CrashFlow,
    };
    let recorded = record_through_the_crash_point_recorder("multi-record-root-persisted-count");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
    let expansion = |segment_index: usize, segment: &[usize]| {
        if expand(segment_index, segment) {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let first_root_slot_write = recorded
        .writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    assert_ne!(first_root_slot_write, recorded.judged_root_index);
    for judged_root_index in [recorded.judged_root_index, first_root_slot_write] {
        let item_name = format!("multi_record_publish::root_persisted_count::{judged_root_index}");
        let Some(tally) = enumerate_crash_states_selecting_versions(
            CrashPointReplayItem {
                name: &item_name,
                setting_file: THIS_FILE,
            },
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            judged_root_index,
            &recorded.versions,
            &expand,
        )
        .judged() else {
            continue;
        };
        let flow = CrashFlow {
            base: &recorded.base,
            writes: &recorded.writes,
            segments: &recorded.segments,
            judged_root_index,
            versions: &recorded.versions,
            versions_without_a_file: &[],
            expansion: &expansion,
            crash_points: recorded.crash_points.clone(),
        };
        let plan = plan_crash_points(&flow);
        assert_eq!(
            states_with_the_root_slot_write_persisted(&flow, &plan, judged_root_index),
            tally.root_persisted_states,
            "被判根是写表第 {judged_root_index} 次写：按计划的段现算与逐状态数的相同（共 {} 个状态）",
            tally.states
        );
    }
}

/// 观察者另读的输入的摘要要真算：全零就是没算（另读输入不同的两趟会共用看过的块）。
fn extra_inputs_digest_that_was_computed(
    digest: [u8; singlefs_harness::sha256::DIGEST_BYTES],
) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    assert_ne!(
        digest,
        [0; singlefs_harness::sha256::DIGEST_BYTES],
        "观察者另读的输入的摘要真算（全零就是没算：另读输入不同的两趟会共用看过的块）"
    );
    digest
}
