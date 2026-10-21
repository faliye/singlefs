//! 签发那次发布写进整理意图的 `segs0` 取签发之前（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H3，用户 2026-10-16
//! 定「取签发之前」）：记签发那次发布之前的全空聚簇段数，停机谓词「全局差值」（一条意图完成之后 `全空聚簇段数(现在) − segs0 ≥ 1` 才签下一条，
//! 攻方 K-h）相对它比。签发那次发布自己开了一个新聚簇段时，这个数比那次写下的记账第 11 项多一，这条意图就要多腾出那一段才算有产出
//! （修复批一 F3 取签发那次的记账第 11 项，把基线挤低一格，辩方 `the_completed_intent_yielded(3310, 3310)=false`、`(3310, 3309)=true`）。
//! 前两格在发布入口上直接签：签发落在开放段里、签发自己开新段，两格里 logged_ops 写下的都是交进去的签发之前那个数。
//! 后一格在会话上推完一条意图：签发开了新段、意图腾空 R，全空段数回到签发之前那个数，没有产出，这一趟停在「无产出就停」。
mod common;

use common::{build_pool, parameters, BuiltPool};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::UnitFootprint;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{
    empty_cluster_segments_by_the_records, region_is_empty_by_the_records, CompactionBatchSize,
    CompactionBatchUnits, CompactionRunStop, CompactionSwitches, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};
use singlefs_core::records::STATISTIC_EMPTY_CLUSTER_SEGMENTS;
use singlefs_core::transaction::{
    publish_compaction_step, CompactionIntentStep, PoolWriter, TransactionOutput,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::SharedStream;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
/// 第一个文件那次发布的提交内生块那一段（`build_pool` 之后 0 号盘上的候选段，同 `compaction_while_mounted_in_random_histories.rs`）。
const REGION_START_SLOT: u64 = 50_240;
/// 填开放段时留下几个空槽：签发那次发布要写的提交内生块比这多，就得开一个新聚簇段。
const FREE_SLOTS_LEFT_IN_THE_OPEN_SEGMENT: u64 = 2;
/// 一个聚簇段的槽数。
const SLOTS_OF_A_CLUSTER_SEGMENT: u64 = 64;
/// 会话那一格的 R：两次顺序写之后 0 号盘上装着用户数据的那一段（攻方 `research/prompts/m4-segment-code-r2-opus-model/src/attack_r2_transient_read.rs` 的前缀）。
const REGION_START_SLOT_AFTER_THE_TWO_SEQUENTIAL_WRITES: u64 = 50_496;

/// 一块盘的单元区（按记录数全空段用）。
fn unit_area_of_the_first_device(pool: &BuiltPool) -> UnitAreaOfOneDevice {
    let device_map = &pool.allocator.devices[0];
    UnitAreaOfOneDevice {
        device: device_map.device,
        start: device_map.unit_area_start(),
        slots: device_map.unit_area_slots(),
    }
}

/// 签发一条意图：`segs0` 照会话的取法交签发之前那一版按记录数出的全空段数，分配器先排他 R（同会话），交回签发那一版与交进去的那一条。
fn issue_with_the_empty_segments_counted_before_the_issue(
    pool: &mut BuiltPool,
) -> (TransactionOutput, CompactionIntent) {
    let previous = pool.output.clone();
    let handed_in = CompactionIntent {
        region: CompactionRegion::of_one_cluster_segment(
            DeviceIdentity(0),
            SlotNumber(REGION_START_SLOT),
        )
        .expect("段起点是 64 的倍数"),
        allocation_generation_at_issue: previous.root.checkpoint_txg,
        empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
            &previous.allocation_records,
            unit_area_of_the_first_device(pool),
        ),
    };
    pool.allocator.compaction_mut().issue(handed_in);
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let issued = publish_compaction_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        &[],
        CompactionIntentStep::Issue(handed_in),
        INSTANCE,
    )
    .expect("签发");
    (issued, handed_in)
}

/// 一版写下的记账第 11 项里 0 号盘那一行。
fn accounting_row_eleven_of_the_first_device(version: &TransactionOutput) -> u64 {
    version
        .accounting_entries
        .iter()
        .find(|entry| {
            entry.statistic == STATISTIC_EMPTY_CLUSTER_SEGMENTS
                && entry.device == DeviceIdentity(0)
                && entry.generation == version.root.checkpoint_txg
        })
        .expect("这一版的发布写了第 11 项")
        .value
}

/// 两格共同的判：logged_ops 里那一条就是交进去的那一条（`segs0` 是签发之前那个数，R 与 `c0` 照交进去的），分配器上记着的活意图就是盘上那一条。
fn assert_the_logged_intent_is_the_one_handed_in(
    pool: &BuiltPool,
    issued: &TransactionOutput,
    handed_in: CompactionIntent,
) {
    let (_, logged) = live_compaction_intent_of_the_entries(&issued.logged_operations)
        .expect("签发那一版的 logged_ops 里有这一条意图");
    assert_eq!(
        logged, handed_in,
        "签发那次写下的意图就是交进去的那一条：segs0 = 签发那次发布之前的全空聚簇段数（判决 m4-segment-code-r2 第三节 H3）"
    );
    assert_eq!(
        pool.allocator
            .compaction()
            .live_intent()
            .map(|live| live.intent),
        Some(logged),
        "分配器上记着的活意图就是盘上写下的那一条（会话每一步核这两样相同）"
    );
}

/// 签发落在开放段里：签发前后全空段数不变，写下的签发之前那个数就等于签发那次的记账第 11 项。
#[test]
fn an_issue_inside_the_open_segment_writes_the_empty_segments_before_the_issue() {
    let mut pool = build_pool("compaction-segs0-before-the-issue-inside-the-open-segment");
    let open_segment_before = pool.allocator.open_segment();
    let (issued, handed_in) = issue_with_the_empty_segments_counted_before_the_issue(&mut pool);
    assert_eq!(
        pool.allocator.open_segment(),
        open_segment_before,
        "这一格签发那次发布没开新段"
    );
    assert_eq!(
        handed_in.empty_cluster_segments_at_issue,
        accounting_row_eleven_of_the_first_device(&issued),
        "这一格签发前后全空段数相同"
    );
    assert_the_logged_intent_is_the_one_handed_in(&pool, &issued, handed_in);
}

/// 签发自己开一个新聚簇段（开放段先填到只剩两槽）：写下的是签发之前那个数，比签发那次的记账第 11 项多一。
#[test]
fn an_issue_opening_a_new_cluster_segment_writes_the_empty_segments_before_the_issue_not_its_own_accounting(
) {
    let mut pool = build_pool("compaction-segs0-before-the-issue-opening-a-new-segment");
    let open_segment_before = pool
        .allocator
        .open_segment()
        .expect("第一个文件那次发布之后有开放段");
    let generation_of_the_filler = CheckpointTxg(pool.output.root.checkpoint_txg.0);
    // 迭代上界：开放段 64 槽，每轮多占一槽，占到只剩两槽为止。
    loop {
        let used_slots: u64 = pool
            .allocator
            .records()
            .iter()
            .filter(|record| {
                record.device == DeviceIdentity(0)
                    && (open_segment_before.0..open_segment_before.0 + SLOTS_OF_A_CLUSTER_SEGMENT)
                        .contains(&record.slot.0)
            })
            .map(|record| u64::from(record.span_slots))
            .sum();
        if used_slots + FREE_SLOTS_LEFT_IN_THE_OPEN_SEGMENT >= SLOTS_OF_A_CLUSTER_SEGMENT {
            break;
        }
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, generation_of_the_filler)
            .expect("开放段里还有空槽");
    }
    let (issued, handed_in) = issue_with_the_empty_segments_counted_before_the_issue(&mut pool);
    assert_ne!(
        pool.allocator.open_segment(),
        Some(open_segment_before),
        "这一格签发那次发布开了新段"
    );
    assert_eq!(
        handed_in.empty_cluster_segments_at_issue,
        accounting_row_eleven_of_the_first_device(&issued) + 1,
        "这一格签发之前数出的全空段数比签发那次的记账第 11 项多一（新段是那次发布开的）"
    );
    assert_the_logged_intent_is_the_one_handed_in(&pool, &issued, handed_in);
}

/// 从第一个文件起两次顺序写（152 个、290 个数据单元）之后的整份镜像（每一版与模型对得上）。
fn image_after_the_two_sequential_writes() -> MemoryPool {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 152,
                    selector: 9_852_620_462_860_606_767,
                },
                fill_seed: 2_832_512_656_265_544_978,
            }),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 290,
                    selector: 15_334_540_613_569_234_429,
                },
                fill_seed: 10_383_977_899_441_184_023,
            }),
        ],
    };
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
        "两次顺序写每一版与模型对得上：{:?}",
        run.outcomes
    );
    image_after_the_last_step.expect("走完的历史每一步之后都交镜像")
}

/// 会话上推完一条意图：开放段先填到只剩两槽、签发那次发布开了新段，意图在装着用户数据的那一段上一批三个搬到 R 按记录全空、同一次发布删掉。
/// 全空段数回到签发之前那个数：意图腾出的那一段刚好抵掉签发开的新段，没有产出，这一趟停在「无产出就停」（`segs0` 取签发之后那个数时，
/// 现在比它多一、照签下一条）。
#[test]
fn an_intent_whose_issue_opened_a_new_segment_yields_only_after_freeing_one_more_segment() {
    let image = image_after_the_two_sequential_writes();
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
        mount_writable(&width.parameters(), &mut devices).expect("两次顺序写之后的镜像可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let before_the_run = session
        .current
        .version_of_trees()
        .expect("有树的一版")
        .clone();
    let unit_area = {
        let device_map = &session.allocator.devices[0];
        UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        }
    };
    let empty_segments_before_the_issue =
        empty_cluster_segments_by_the_records(&before_the_run.allocation_records, unit_area);
    let open_segment = session
        .allocator
        .open_segment()
        .expect("可写挂载之后有开放段");
    let generation_of_the_filler = before_the_run.root.checkpoint_txg;
    // 迭代上界：开放段 64 槽，每轮多占一槽，占到只剩两槽为止。
    loop {
        let used_slots: u64 = session
            .allocator
            .records()
            .iter()
            .filter(|record| {
                record.device == DeviceIdentity(0)
                    && (open_segment.0..open_segment.0 + SLOTS_OF_A_CLUSTER_SEGMENT)
                        .contains(&record.slot.0)
            })
            .map(|record| u64::from(record.span_slots))
            .sum();
        if used_slots + FREE_SLOTS_LEFT_IN_THE_OPEN_SEGMENT >= SLOTS_OF_A_CLUSTER_SEGMENT {
            break;
        }
        session
            .allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, generation_of_the_filler)
            .expect("开放段里还有空槽");
    }
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(REGION_START_SLOT_AFTER_THE_TWO_SEQUENTIAL_WRITES),
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
                CompactionBatchUnits::new(3).expect("至少 1"),
            ),
            forced_region: Some(region),
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    let issue = run
        .versions_published
        .first()
        .expect("这一趟头一次发布签发意图");
    let (_, logged) = live_compaction_intent_of_the_entries(&issue.logged_operations)
        .expect("头一次发布签发意图");
    assert_eq!(
        logged.empty_cluster_segments_at_issue, empty_segments_before_the_issue,
        "segs0 = 签发那次发布之前的全空聚簇段数"
    );
    assert_eq!(
        accounting_row_eleven_of_the_first_device(issue) + 1,
        empty_segments_before_the_issue,
        "签发那次发布开了新段：它写下的记账第 11 项比签发之前少一"
    );
    let last = run.versions_published.last().expect("这一趟发了布");
    assert!(
        live_compaction_intent_of_the_entries(&last.logged_operations).is_none()
            && region_is_empty_by_the_records(&last.allocation_records, region),
        "意图推完：R 按记录全空，意图删了"
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent {
                empty_cluster_segments_now,
                empty_cluster_segments_at_issue,
            }) if empty_cluster_segments_now == empty_segments_before_the_issue
                && empty_cluster_segments_at_issue == empty_segments_before_the_issue
        ),
        "意图腾出的那一段只抵掉签发开的新段（全空段数 {empty_segments_before_the_issue} 没变）：没有产出，这一趟停在「无产出就停」，实际 {:?}",
        run.outcome
    );
}
