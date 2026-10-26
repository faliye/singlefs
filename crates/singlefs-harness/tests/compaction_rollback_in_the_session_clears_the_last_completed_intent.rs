//! 会话内回滚清掉上一条完成的意图（用户 2026-10-26 定「回滚时清掉」，`records/2026-10-19-里程碑四GPU垃圾回收开工.md`「写段冒出的新发现」那一行；
//! D26（后台整理与放置回收） 已定项 13 第 6 条「签下一条意图那一判跨趟留着」）：会话上跨趟留着的上一条完成的意图
//! （`compaction_plan::CompactionOfThisMount::last_completed_intent`）留到回滚为止；回退做成时清掉，下一趟从空起签新意图、发批
//! （`mount::roll_back_by_a_forward_publish_in_the_session`）；回退被拒时一样不动，下一趟照旧停在「上一条无产出」。
//! 池照 `compaction_off_the_calling_thread_and_admission.rs` 的做法：`common::build_pool` 起，可写挂载，强制选第一个文件那次发布的提交内生块
//! 所在的段 50240、一批一个，开放段先填到只剩两槽（签发那次发布开了新段），搬空 50240 只抵掉那个新段，这一趟停在「上一条无产出」
//! （不靠用户改动占掉全空段：用户改动发成会清掉那一条，用户 2026-10-26 定「每次用户改动发成就清掉」）；回退的目标是挂载之后、整理之前那一版
//! （回退撤销那一趟整理，50240 里的单元复活，同一段照样搬得动）。
mod common;

use common::{build_pool, parameters, BuiltPool, Recorded};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::UnitFootprint;
use singlefs_core::compaction_plan::{
    empty_cluster_segments_by_the_records, CompactionBatchSize, CompactionBatchUnits,
    CompactionRunForcing, CompactionRunStop, CompactionSwitches, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::{CompactionIntent, CompactionRegion};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish_in_the_session, RollbackCandidateExclusion,
    RollbackError, RollbackTarget,
};
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};
use singlefs_core::transaction::{PoolVersion, TransactionOutput};

/// 第一个文件那次发布的提交内生块落在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)）。
const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 50240;
/// 填开放段时留下几个空槽：签发那次发布要写的提交内生块比这多，就得开一个新聚簇段
/// （同 `compaction_off_the_calling_thread_and_admission.rs` 那一条）。
const FREE_SLOTS_LEFT_IN_THE_OPEN_SEGMENT: u64 = 2;
/// 一个聚簇段的槽数。
const SLOTS_OF_A_CLUSTER_SEGMENT: u64 = 64;

/// 重开、可写挂载（产品默认 `cpu`）、开会话。
fn mounted_session_of(pool: &mut BuiltPool) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (session, devices)
}

/// 会话装上整理只供测试的开关：强制选 `region`、一批一个、起跑照水位，其余照产品值。
fn force_one_unit_batches_on(session: &mut MountedSession, region: CompactionRegion) {
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing: _,
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
            run_forcing: CompactionRunForcing::FollowTheWatermarks,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
}

fn region_of_the_first_publish() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH),
    )
    .expect("64 的倍数")
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

fn target_of(version: &TransactionOutput) -> RollbackTarget {
    RollbackTarget {
        instance: version.root.instance,
        checkpoint_txg: version.root.checkpoint_txg,
    }
}

/// 现行那一版按记录数出的全空聚簇段数（R 所在的那块盘）。
fn empty_cluster_segments_now(session: &MountedSession) -> u64 {
    let device_map = session.allocator.devices.first().expect("池里至少一块盘");
    empty_cluster_segments_by_the_records(
        &current_of(session).allocation_records,
        UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        },
    )
}

/// 分配器把开放段占到只剩两槽（只动分配器，不写盘）：之后签发那次发布要开一个新聚簇段。
fn fill_the_open_segment_up_to_two_free_slots(session: &mut MountedSession) {
    let open_segment = session
        .allocator
        .open_segment()
        .expect("可写挂载之后有开放段");
    let generation_of_the_filler = current_of(session).root.checkpoint_txg;
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
}

/// 会话停在「上一条无产出」时的样子：开放段填到只剩两槽，一趟在 50240 上签发（签发那次发布开了新段）、搬空 R、删意图，
/// 意图腾出的那一段只抵掉签发开的新段，这一趟停在「上一条无产出」，那一条完成的意图记在会话上。
/// 交回会话、盘表、那一条完成的意图与挂载之后、整理之前那一版。
struct SessionStoppedOnNoYield {
    session: MountedSession,
    devices: Vec<(DeviceIdentity, Recorded)>,
    completed: CompactionIntent,
    version_before_the_compaction: TransactionOutput,
}

fn session_stopped_on_no_yield(pool: &mut BuiltPool) -> SessionStoppedOnNoYield {
    let (mut session, mut devices) = mounted_session_of(pool);
    force_one_unit_batches_on(&mut session, region_of_the_first_publish());
    let version_before_the_compaction = current_of(&session).clone();
    fill_the_open_segment_up_to_two_free_slots(&mut session);
    let stopped = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            stopped.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent { .. })
        ),
        "前提：签发开了新段、搬空 R 只抵掉它，这一趟停在「上一条无产出」，实际 {:?}",
        stopped.outcome
    );
    let completed = session
        .compaction
        .last_completed_intent
        .expect("前提：那一趟完成的意图记在会话上");
    assert_eq!(
        completed.region,
        region_of_the_first_publish(),
        "前提：完成的是 50240 上那一条"
    );
    SessionStoppedOnNoYield {
        session,
        devices,
        completed,
        version_before_the_compaction,
    }
}

/// 会话内回滚到 `target`：会话的现行那一版、分配器与整理状态一起交进去，交回回退的结果，做成时现行那一版换成回退那次发布。
fn roll_back_in_the_session(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
    target: RollbackTarget,
) -> Result<(), RollbackError> {
    let mut current = current_of(session).clone();
    roll_back_by_a_forward_publish_in_the_session(
        &parameters(),
        devices,
        &mut session.allocator,
        &mut current,
        &mut session.compaction,
        target,
    )?;
    session.current = PoolVersion::of_a_version_of_trees(current);
    Ok(())
}

/// 停在「上一条无产出」之后会话内回滚到挂载之后、整理之前那一版：回退做成，会话上的上一条完成的意图清掉；
/// 回退之后全空聚簇段数仍不多于那一条签发时的数（留着那一条，下一趟照样停在「上一条无产出」）。仍强制选 50240（回退复活了它里面的单元），
/// 下一趟照常签新意图、至少发一批。判别力：回退做成时不清那一条（`mount::roll_back_by_a_forward_publish_in_the_session` 不置空），
/// 下一趟读到它、停在「上一条无产出」、一批不发，这一条红。
#[test]
fn a_rollback_in_the_session_after_a_run_stopped_on_no_yield_lets_the_next_run_issue_a_new_intent_and_publish_batches(
) {
    let mut pool =
        build_pool("compaction-rollback-in-the-session-clears-the-last-completed-intent");
    let SessionStoppedOnNoYield {
        mut session,
        mut devices,
        completed,
        version_before_the_compaction,
    } = session_stopped_on_no_yield(&mut pool);
    roll_back_in_the_session(
        &mut session,
        &mut devices,
        target_of(&version_before_the_compaction),
    )
    .expect("挂着时回退到挂载之后、整理之前那一版");
    let empty_now = empty_cluster_segments_now(&session);
    assert!(
        empty_now <= completed.empty_cluster_segments_at_issue,
        "前提：回退之后全空聚簇段数 {empty_now} 不多于那一条签发时的 {}，留着那一条下一趟照样停在「上一条无产出」",
        completed.empty_cluster_segments_at_issue
    );
    let intents_issued_before = session.compaction.observations.intents_issued;
    let next_run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(
        session.compaction.observations.intents_issued,
        intents_issued_before + 1,
        "回退做成时清掉了上一条完成的意图：下一趟不停在「上一条无产出」，在 50240 上签一条新意图，停在 {:?}",
        next_run.outcome
    );
    assert!(
        next_run.batches_published >= 1,
        "下一趟至少发一批，实际 {} 批，停在 {:?}",
        next_run.batches_published,
        next_run.outcome
    );
    let completed_by_the_next_run = session
        .compaction
        .last_completed_intent
        .expect("下一趟搬空 R、完成了那一条新意图");
    assert_ne!(
        completed_by_the_next_run, completed,
        "会话上记的是下一趟新签、新完成的那一条，不是回退之前那一条"
    );
}

/// 停在「上一条无产出」之后会话内回滚到根环最新那条之后的 txg：不在根环里，回退被拒，会话上的上一条完成的意图照旧留着，
/// 下一趟照旧停在「上一条无产出」。判别力：不论回退成不成都先清那一条，回退被拒之后它就空了、下一趟签新意图，这一条红。
#[test]
fn a_refused_rollback_in_the_session_keeps_the_last_completed_intent_and_the_next_run_still_stops_on_no_yield(
) {
    let mut pool =
        build_pool("compaction-refused-rollback-in-the-session-keeps-the-last-completed-intent");
    let SessionStoppedOnNoYield {
        mut session,
        mut devices,
        completed,
        version_before_the_compaction: _,
    } = session_stopped_on_no_yield(&mut pool);
    let newest = current_of(&session).root;
    let target_beyond_the_newest_root = RollbackTarget {
        instance: newest.instance,
        checkpoint_txg: CheckpointTxg(newest.checkpoint_txg.0 + 1),
    };
    let refused =
        roll_back_in_the_session(&mut session, &mut devices, target_beyond_the_newest_root)
            .expect_err("根环里没有那个 txg 的根：回退被拒");
    assert!(
        matches!(
            refused,
            RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::NotInRing,
                ..
            }
        ),
        "回退被拒在候选集那一判：{refused:?}"
    );
    assert_eq!(
        session.compaction.last_completed_intent,
        Some(completed),
        "回退没做成：会话上的上一条完成的意图照旧留着"
    );
    let next_run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            next_run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent { .. })
        ),
        "下一趟照旧读到留着的那一条、停在「上一条无产出」，实际 {:?}",
        next_run.outcome
    );
    assert!(next_run.versions_published.is_empty(), "一个写都没发");
}
