//! 腾空 R 的那一批同一次发布删意图（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2；
//! `research/prompts/m4-gc-r1-main-verification.md` 第三节 G4「目标态到达与最后一批同发布删」；D8（核心索引结构） 已定项 5）。
//! 计划里的单元都是这次发布本来就重写的节点时，这一批一个都没搬（`relocated = 0`），R 却在这次发布之后按记录全空：意图要同一次发布删，
//! 不能留到下一次只删意图的发布（模型判「一次整理发布要么搬一批、要么删掉意图」）；这样一次发布也不记成一批。
//! 历史照攻方腿种子下标 1055 缩出的 13 步（`research/prompts/m4-segment-code-r1-opus-model/out/compaction-seed1055-trace.txt` 的 OPS 那一行，
//! 起点是第一个文件）：最后那一趟整理第四次发布就是这一形。执行器每一版按模型比，历史要走完。
//!
//! 删意图那一支在算好的账上核不过时照这一批不删意图再发（`fs-design.md`「五条硬要求」第 2 条：每条分支必须能被测试强制进入）：
//! 同一段历史的前 12 步之后重开镜像、可写挂载，会话经只供测试的 `RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`
//! 把那一支强制进去。

use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{
    region_is_empty_by_the_records, CompactionBatchSize, CompactionBatchUnits, CompactionSwitches,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};
use singlefs_core::transaction::{RegionCheckOfTheIntentDeletion, TransactionOutput};
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, CompactionChoice, CompactionRunEnding, GeneratedHistory,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, SnapshotChoice, StepOutcome,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::SharedStream;

fn destroy_snapshot(index_among_the_snapshots: u64) -> HistoryOperation {
    HistoryOperation::DestroySnapshot(SnapshotChoice {
        index_among_the_snapshots,
    })
}

fn compaction(
    batch_units: u64,
    region_selector: u64,
    ending: CompactionRunEnding,
) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending,
    })
}

/// 攻方腿种子下标 1055 缩出的 13 步，逐个照 OPS 那一行。
fn history_of_the_attack_seed_1055() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            destroy_snapshot(16_576_431_429_050_751_641),
            destroy_snapshot(8_608_203_070_865_410_994),
            compaction(
                1,
                14_751_488_729_257_944_391,
                CompactionRunEnding::RunsUntilItStops,
            ),
            HistoryOperation::CreateSnapshot,
            destroy_snapshot(13_548_285_845_384_774_606),
            compaction(
                2,
                15_468_272_257_618_924_935,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
            ),
            compaction(
                1,
                13_606_409_341_791_376_233,
                CompactionRunEnding::RunsUntilItStops,
            ),
        ],
    }
}

/// 13 步每一版与模型对得上（最后那一趟第四次发布的计划只有一个本来就重写的节点：它不另搬，发布之后 R 按记录全空，意图与它同一次发布删），
/// 每一趟整理报的批数不多于搬的单元数（一批至少搬一个；没搬单元的发布记成一批时，最后那一趟报四批、只搬了三个）。
#[test]
fn a_batch_whose_planned_units_this_publish_rewrites_anyway_deletes_the_intent_once_it_empties_the_region(
) {
    let history = history_of_the_attack_seed_1055();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "每一版与模型对得上：腾空 R 的那一批不删意图时，下一次只删意图的发布模型判「一次整理发布要么搬一批、要么删掉意图」；结局 {:?}",
        run.outcomes
    );
    let compactions: Vec<(u64, u64, bool)> = run
        .outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            StepOutcome::Applied(AppliedEffect::Compacted {
                batches_published,
                units_relocated,
                is_the_intent_live_when_the_run_stopped,
                ..
            }) => Some((
                *batches_published,
                *units_relocated,
                *is_the_intent_live_when_the_run_stopped,
            )),
            StepOutcome::Applied(_)
            | StepOutcome::Refused { .. }
            | StepOutcome::NotApplicable(_) => None,
        })
        .collect();
    assert_eq!(compactions.len(), 3, "三趟整理都做成了：{:?}", run.outcomes);
    for (batches_published, units_relocated, _) in &compactions {
        assert!(
            batches_published <= units_relocated,
            "一批至少搬一个单元：报了 {batches_published} 批、只搬了 {units_relocated} 个（全部整理 {compactions:?}）"
        );
    }
    let (_, last_units_relocated, is_the_intent_live_after_the_last_run) =
        *compactions.last().expect("三趟整理");
    assert!(
        last_units_relocated >= 1 && !is_the_intent_live_after_the_last_run,
        "最后那一趟搬过单元、停下时意图已删（与腾空 R 的那一批同一次发布）：{compactions:?}"
    );
}

/// 攻方种子下标 1055 那段历史最后那一趟整理签发的 R：盘 0 上这一段（执行器按 `region_selector` 在可强制选的段里挑出的那一段）。
const REGION_START_SLOT_OF_THE_LAST_COMPACTION_OF_THE_ATTACK_SEED_1055: u64 = 50432;

/// 攻方种子下标 1055 那段历史最后那一趟整理之前的 12 步，走完之后的整份镜像（每一版与模型对得上）。
fn image_before_the_last_compaction_of_the_attack_seed_1055() -> MemoryPool {
    let mut history = history_of_the_attack_seed_1055();
    let last_compaction = history.operations.pop().expect("13 步");
    assert!(
        matches!(
            last_compaction,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                ending: CompactionRunEnding::RunsUntilItStops,
                ..
            })
        ),
        "去掉的是最后那一趟整理：{last_compaction:?}"
    );
    let mut image_after_the_last_step: Option<MemoryPool> = None;
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| image_after_the_last_step = Some(observation.image.clone()),
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "前 12 步每一版与模型对得上：{:?}",
        run.outcomes
    );
    image_after_the_last_step.expect("走完的历史每一步之后都交镜像")
}

/// 一版里还活着的整理意图是哪一次发布签的（签发 txg）；没有活意图是 `None`。
fn live_intent_issued_txg(version: &TransactionOutput) -> Option<u64> {
    live_compaction_intent_of_the_entries(&version.logged_operations)
        .map(|(issued_txg, _intent)| issued_txg.0)
}

/// 删意图那一支核不过（只供测试的开关强制）：前 12 步之后重开镜像、可写挂载，在 R 上签发、一批一个推到停。R 最后只剩两块盘各一个分配记录树叶
/// （这一批本来就重写的节点）时，计划的那一个不另搬、发布之后 R 按记录全空：照抄核出腾空了 R、删意图那一支被开关拒，这一批照抄意图发出去
/// （发出去的那一版 R 按记录全空、意图还在）；下一次发布照「R 已按记录全空」只删意图、不搬单元，这一趟照常停，这条意图收掉。
#[test]
fn the_intent_deletion_refused_after_a_batch_emptying_the_region_publishes_the_batch_carrying_the_intent_and_the_next_publish_deletes_it(
) {
    let image = image_before_the_last_compaction_of_the_attack_seed_1055();
    let width = HistoryDeviceWidth::FourGibibytes;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect();
    let mounted =
        mount_writable(&width.parameters(), &mut devices).expect("前 12 步之后的镜像可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let before_the_run = session
        .current
        .version_of_trees()
        .expect("有树的一版")
        .clone();
    assert_eq!(
        live_intent_issued_txg(&before_the_run),
        None,
        "前 12 步之后没有活意图：最后那一趟整理在这一版上签发"
    );
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(REGION_START_SLOT_OF_THE_LAST_COMPACTION_OF_THE_ATTACK_SEED_1055),
    )
    .expect("64 的倍数");
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(1).expect("至少 1"),
            ),
            forced_region: Some(region),
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    let run = session
        .run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region(
            &mut devices,
            RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests,
        )
        .expect("整理一趟");
    assert!(
        matches!(run.outcome, CompactionRunOutcome::Stopped(_)),
        "这一趟照常停，不是一次发布没发成：{:?}",
        run.outcome
    );
    let issued_txg = run
        .versions_published
        .first()
        .and_then(live_intent_issued_txg)
        .expect("这一趟头一次发布签发意图");
    let emptying_index = run
        .versions_published
        .iter()
        .position(|version| region_is_empty_by_the_records(&version.allocation_records, region))
        .expect("这一趟有一次发布之后 R 按记录全空");
    let before_the_emptying_batch = &run.versions_published[emptying_index
        .checked_sub(1)
        .expect("头一次发布是签发：签发不碰 R 里的记录，R 按记录全空的不是它")];
    assert!(
        !region_is_empty_by_the_records(&before_the_emptying_batch.allocation_records, region)
            && live_intent_issued_txg(before_the_emptying_batch) == Some(issued_txg),
        "腾空 R 的是这一批：它之前那一版 R 里还有未释放的记录、意图活着"
    );
    let emptying_batch = &run.versions_published[emptying_index];
    assert!(
        emptying_batch.units_relocated_by_this_publish.is_empty(),
        "这一批计划的是这次发布本来就重写的节点，不另搬：{:?}",
        emptying_batch.units_relocated_by_this_publish
    );
    assert_eq!(
        live_intent_issued_txg(emptying_batch),
        Some(issued_txg),
        "腾空 R 的那一批照抄意图发出去（删意图那一支核不过，照这一批不删意图再发）：第 {emptying_index} 次发布"
    );
    let deleting_publish = run
        .versions_published
        .get(emptying_index + 1)
        .expect("腾空 R 的那一批之后还有一次发布：只删意图");
    assert_eq!(
        live_intent_issued_txg(deleting_publish),
        None,
        "下一次发布照「R 已按记录全空」删意图"
    );
    assert!(
        deleting_publish.units_relocated_by_this_publish.is_empty()
            && region_is_empty_by_the_records(&deleting_publish.allocation_records, region),
        "删意图那一次不搬单元，删的那一刻 R 按记录全空：{:?}",
        deleting_publish.units_relocated_by_this_publish
    );
    assert_eq!(
        session.allocator.compaction().live_intent(),
        None,
        "这一趟停下时分配器上没有活意图（这一条收掉了，全局差值为假不签下一条）"
    );
}
