//! 验证包模块：crash、crash_state_enumeration
//! 活意图期间写类改动的崩溃流（D26（后台整理与放置回收） 已定项 13 第 5 条挂载就起之后，整理批与前台写在一把锁上交错成的次序）：
//! 「取号 → 暖机 × 2 → A → B、C 两次顺序写 → 实例 1 里建空 inode 填满几个叶容器 → 重挂（取号、写行、暖机）→ 整理签发 → 一批 → 意图活着时会话里覆盖写第一个文件 → 一批 → 最后一批（同一次发布删意图）」
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。R 与批大小同整理搬迁流（第一个文件那次发布的提交内生块所在的段，一批两个）。
//! 建 inode 那一步是这条流站得住的前提：覆盖写第一个文件重写的单元把整理搬迁流 R 里第一批搬剩下的全放掉，R 按记录全空，之后只有一次不搬单元、只删意图的发布；
//! 那次发布的叶容器也落在 R 里，覆盖写只重写第一片，别的留给下一批。
//! 每个状态跑恢复 + 多版本 oracle（覆盖写之前读 C 那一份，之后读覆盖写那一份）、池级 checker、记录核对器；观察者同整理搬迁流：
//! 抹掉这一版之前整理搬走的单元的源落点再读、可写挂载一次看整理续做推到意图删了。
//!
//! **比已有的流多罩了什么**：已有的整理流上活意图期间只有 unlink（`crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs`），没有写类改动。
//! 这条流第一次枚举覆盖写放掉 R 里落点之后的状态：那次发布重写的节点在 R 里的旧落点进 defer，R 里剩下要搬的少了，下一批对 R 重扫；
//! 崩在那次发布与下一批之间的每个状态上，可写挂载续做读到的是覆盖写之后那一版的 R。多跑的步：覆盖写那一版也走一遍两个观察者。前缀到 C 与整理搬迁流逐字节相同。
//!
//! 每段都展开的那一条登记成崩溃枚举用例，带断点续跑、可双机分片。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

mod common;
mod common_compaction_crash_point_observers;
mod common_compaction_prefix;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_prefix;

use common_compaction_crash_point_observers::{
    assert_every_state_was_observed, landed_counter_of, landed_version_observer,
    mount_continuation_observer, observe_the_landed_version_reading_through_the_mapping,
    observe_the_mount_continuing_the_compaction,
};
use common_compaction_prefix::{
    free_slots_in_the_region_of_the_crash_flows_on_each_device,
    isolate_every_free_slot_outside_the_region_of_the_crash_flows,
    prepare_the_compaction_crash_flow, region_holds_a_placement_of_the_publish_of_the_version,
    region_of_the_crash_flows,
    session_with_the_region_emptied_by_an_overwrite_while_the_intent_is_live, version_of_trees,
    CompactionCrashFlow, CompactionFlowStepKind, PreparedCompactionCrashFlow,
    PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE,
};
use singlefs_core::compaction_plan::region_is_empty_by_the_records;
use singlefs_core::recovery::RecoveryReport;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplayVersionTable,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
};
use singlefs_verification::memory_pool::CrashImage;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件。
const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs";

fn prepare(tag: &str) -> PreparedCompactionCrashFlow {
    prepare_the_compaction_crash_flow(
        tag,
        CompactionCrashFlow::CompactionWithAWriteWhileTheIntentIsLive,
        &[
            common_crash_points::content_needing(2, 1),
            common_crash_points::content_needing(3, 2),
        ],
    )
}

fn version_table(prepared: &PreparedCompactionCrashFlow) -> CrashPointReplayVersionTable<'_> {
    CrashPointReplayVersionTable::new(
        &prepared.versions_with_a_file,
        &prepared.versions_without_a_file,
    )
}

/// 流里每一种那一版的计数名，每一种都要有状态。
fn landed_counters(prepared: &PreparedCompactionCrashFlow) -> Vec<String> {
    let mut counters: Vec<String> = Vec::new();
    for version in &prepared.versions_of_the_flow {
        let counter = landed_counter_of(version);
        if !counters.contains(&counter) {
            counters.push(counter);
        }
    }
    counters
}

/// 录流的形状：覆盖写夹在第一批与下一批之间、那一版意图还活着；覆盖写放掉了 R 里的落点（R 里未释放的记录比上一批少，不只少这一批搬走的）；
/// 之后接着一批、最后一批，都搬 R 里的单元（下一批对 R 重扫，搬的是覆盖写没动的那几个）；最后一批 R 按记录全空。
#[test]
fn the_overwrite_of_the_flow_sits_between_two_batches_and_releases_placements_in_the_region() {
    let prepared = prepare("compaction-write-while-the-intent-is-live-shape");
    let recorded = &prepared.recorded;
    let kinds: Vec<CompactionFlowStepKind> = recorded.steps.iter().map(|step| step.kind).collect();
    let position_of_the_overwrite = kinds
        .iter()
        .position(|kind| *kind == CompactionFlowStepKind::OverwriteWhileTheIntentIsLive)
        .expect("流里有那一次覆盖写");
    assert_eq!(
        kinds[position_of_the_overwrite - 1],
        CompactionFlowStepKind::CompactionBatch,
        "覆盖写接在一批之后：{kinds:?}"
    );
    assert_eq!(
        kinds[position_of_the_overwrite + 1..],
        [
            CompactionFlowStepKind::CompactionBatch,
            CompactionFlowStepKind::CompactionFinalBatch
        ],
        "覆盖写之后接着搬：一批照抄意图，再一批同一次发布删意图：{kinds:?}"
    );
    let region = region_of_the_crash_flows();
    for step in &recorded.steps[position_of_the_overwrite + 1..] {
        let relocated: Vec<_> = step
            .versions
            .iter()
            .flat_map(|version| version.units_relocated_by_this_publish.iter())
            .collect();
        assert!(!relocated.is_empty(), "{:?}：这一批真搬了单元", step.kind);
        assert!(
            relocated.iter().all(|unit| unit
                .source
                .iter()
                .all(|location| region.contains_slot(location.slot))),
            "{:?}：搬的是 R 里的单元（下一批对 R 重扫）",
            step.kind
        );
    }
    let unreleased_records_in_the_region = |step_index: usize| -> usize {
        let version = recorded.steps[step_index]
            .versions
            .last()
            .expect("这一步发成了一版");
        version
            .allocation_records
            .iter()
            .filter(|record| {
                record.device == region.device()
                    && !record.is_released
                    && region.contains_slot(record.slot)
            })
            .count()
    };
    assert!(
        unreleased_records_in_the_region(position_of_the_overwrite)
            < unreleased_records_in_the_region(position_of_the_overwrite - 1),
        "覆盖写重写的节点在 R 里的旧落点放掉了"
    );
    let overwrite_version = recorded.steps[position_of_the_overwrite]
        .versions
        .last()
        .expect("覆盖写发成了一版");
    assert_eq!(
        overwrite_version.logged_operations.len(),
        1,
        "覆盖写那一版意图还活着"
    );
    let last_version = recorded
        .steps
        .last()
        .and_then(|step| step.versions.last())
        .expect("最后一批那一版");
    assert!(
        last_version.logged_operations.is_empty()
            && region_is_empty_by_the_records(&last_version.allocation_records, region),
        "最后一批同一次发布删意图、R 按记录全空"
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读，并可写挂载一次核整理续做。登记成崩溃枚举用例。
#[test]
fn full_enumeration_of_the_compaction_with_a_write_while_the_intent_is_live_stream_is_exhaustive_and_clean(
) {
    let prepared = prepare("compaction-write-while-the-intent-is-live-full");
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_reading_through_the_mapping(
            versions_of_the_flow,
            image,
            report,
            counts,
        );
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuing_the_compaction(versions_of_the_flow, image, report, counts)
        };
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "compaction_with_a_write_while_the_intent_is_live::full",
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        version_table(&prepared),
        &full_expansion,
        &[landed_version_observer(&prepared, &observe)],
        &[mount_continuation_observer(&prepared, &observe_the_mount)],
    )
    .judged() else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_COMPACTION_WITH_A_WRITE_WHILE_THE_INTENT_IS_LIVE states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
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
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        "compaction_with_a_write_while_the_intent_is_live_stream：用例自己的展开下的闭式",
    );
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

/// R 已按记录全空、不带一批只删意图的那一次取落点放开 R（D26（后台整理与放置回收） 已定项 2）：不建空 inode 的前缀上，第一批搬走 R 里两个、
/// 意图活着时覆盖写第一个文件放掉剩下的两个，R 按记录全空、意图还活着；再把 R 之外的每一个空槽隔离（不写盘），整理推一次：
/// 这一次不带一批、只删意图，新落点只能落在 R 里；删意图那一核不把这次发布自己在 R 里的新落点算成「R 不空」，发成，意图连同 R 排他撤了。
/// 判别力：`crates/mutations.tsv`「不带一批的删意图那一核数自己在 R 里的新落点」那一行。
#[test]
fn the_deletion_without_a_batch_takes_its_placements_in_the_region_once_only_the_region_has_free_slots(
) {
    let (mut pool, mut session) =
        session_with_the_region_emptied_by_an_overwrite_while_the_intent_is_live(
            "compaction-deletion-without-a-batch-placed-in-the-region",
            &[
                common_crash_points::content_needing(2, 1),
                common_crash_points::content_needing(3, 2),
            ],
        );
    let free_in_the_region = free_slots_in_the_region_of_the_crash_flows_on_each_device(&session);
    assert!(
        free_in_the_region.iter().all(|(_, free)| *free > 0),
        "前提：R 里每块盘都有空槽（盘、空槽数）{free_in_the_region:?}"
    );
    isolate_every_free_slot_outside_the_region_of_the_crash_flows(&mut session);
    session.compaction.publishes_since_the_last_user_change =
        PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
    let txg_before_the_run = version_of_trees(&session.current).root.checkpoint_txg;
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let run = session
        .run_compaction(devices)
        .unwrap_or_else(|refusal| panic!("只删意图那一次发得成：{refusal:?}"));
    let [deleted] = run.versions_published.as_slice() else {
        panic!(
            "这一趟恰好发一次只删意图的发布，实际发了 {} 次，停在 {:?}",
            run.versions_published.len(),
            run.outcome
        );
    };
    assert!(
        deleted.root.checkpoint_txg.0 == txg_before_the_run.0 + 1
            && deleted.logged_operations.is_empty()
            && deleted.units_relocated_by_this_publish.is_empty()
            && !region_holds_an_unreleased_record_from_before(deleted),
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图，R 里上一版就在的记录一条都没有"
    );
    assert!(
        region_holds_a_placement_of_the_publish_of_the_version(deleted),
        "删意图那一次自己的新落点落在 R 里（R 之外一个空槽都没有）"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "分配器这一侧意图连同 R 排他一起撤了"
    );
}

/// 这一版 R 里有一条分配代不是它自己的 txg 的未释放记录（上一版就在 R 里的）。
fn region_holds_an_unreleased_record_from_before(
    version: &singlefs_core::transaction::TransactionOutput,
) -> bool {
    let region = region_of_the_crash_flows();
    version.allocation_records.iter().any(|record| {
        !record.is_released
            && record.generation != version.root.checkpoint_txg
            && region.contains_slot(record.slot)
    })
}
