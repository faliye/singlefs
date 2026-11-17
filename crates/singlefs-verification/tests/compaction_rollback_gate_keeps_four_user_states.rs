//! 回退门（D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」，D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：周期整理之下根环里不同用户状态不少于 4 个。
//! 两段历史（根环读法同 `research/prompts/m4-gpugc-background-runtime-design-r1-opus-model/src/runtime_design_attack_probes.rs` 那两条探针 `rollback_states`、`crash_loop_step`）：
//! 有用户改动时每次改动之后醒几次；一次用户改动都没有、只有崩溃循环（挂载不给新预算）。开关照水位判（门只在这一支判），越线靠只供测试的开关，选段强制、一批一个。
//! 门给根环留 S 槽余量（这里 S = 8，别的 S 在运行库的 `mount_runtime_rollback_gate_margin_follows_the_root_ring_slots.rs`）：门关的那一刻，再发 8 次发布还剩 4 个状态。崩溃循环分开钉三件事：叫起发的整理不挤掉用户状态、挂载不给新预算（门与预算管得到）；
//! 连崩 2 次（挂 3 次）连同挂载自己的写行、暖机与续做（门不判这几次发布）一共保得住 4 个状态；余量用完之后挂载自己的发布挤掉状态（接受的边界）。
//! 另有破例：全空段低于底额、前台写不进时预算扣完、门也不放都至少发 1 次。搬得出空段的池上前台与整理互相等的死局走得出；只搬满段的那条意图腾出的段抵不掉它开的新段，
//! 停在「上一条无产出」（破例不穿过这一判：选得出的就这一段时再搬也腾不出来）。
mod common;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::time::Duration;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::compaction_plan::{
    foreground_writes_are_below_the_empty_cluster_segments_floor,
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits, CompactionPacing,
    CompactionRunForcing, CompactionRunStop, CompactionSwitches, CompactionTimerFallback,
    CompactionWatermarkCrossingForcing, CompactionWriteBandwidthLimit, EmptyClusterSegmentsFloor,
    WatermarkQuantity,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, BackgroundCompactionNotStarted, CompactionRunOutcome, MountedSession,
    UserChange,
};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::transaction::{PartialWriteBytes, TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::ROOT_RING_REGIONS;

type Devices = Vec<(DeviceIdentity, Recorded)>;

/// `N₀` 压到此刻全空段数加这么多：池子一共没有这么多段，哪次发布都补不上这个差，前台一直写不进。
const SEGMENTS_SHORT_OF_THE_FLOOR_NO_PUBLISH_MAKES_UP: u64 = 1 << 40;
/// 定时兜底间隔。
const INTERVAL_IN_MILLISECONDS: u64 = 30_000;
/// 起点那一次顺序写的数据单元数：数据单元最多的那一段搬不完一份 `B_gc`。
const DATA_UNITS_OF_THE_STARTING_POINT: usize = 40;
/// 回退门保住的状态数。
const ROLLBACK_STATES_KEPT: usize = 4;
/// 破例那几格至多醒几次：每次醒至少发 1 次，一批一个时一条意图 30 多次发布。
const WAKES_OUT_OF_THE_DEADLOCK_AT_MOST: u64 = 64;
/// 带洞的池：隔一个数据单元部分写一个，写几个（旧落点放掉）。
const DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES: usize = 20;
/// 带洞的池：之后再建几次 inode，让根环转过一圈（旧落点过了 defer 窗口、算得上洞）。
const USER_CHANGES_TURNING_THE_ROOT_RING: usize = 30;
/// 连崩 2 次：一共挂几次。
const MOUNTS_OF_A_CRASH_LOOP_WITHIN_THE_MARGIN: usize = 3;
/// 连崩到余量用完：一共挂几次（挂载每次至少推 2 槽，第一次挂载之后再挂 5 次至少推 10 槽）。
const MOUNTS_OF_A_CRASH_LOOP_PAST_THE_MARGIN: usize = 6;
/// 树表的字节数（一个索引节点）。
const BYTES_OF_A_TREE_TABLE: usize = 16384;
/// 回退门给根环留的余量：这些用例的几何 S = 8，余量取 S 槽，照条款写死（改掉产品那一个函数，这里不跟着变）。
const MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS: u64 = 8;
/// 门关之前至多醒几次：定时兜底那一趟推到门关，用户改动之后那一趟至多一批。
const WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST: u64 = 4;

struct CallerOfTheHistories {
    clock_in_milliseconds: Cell<u64>,
}

impl CallerOfTheBackgroundCompaction for CallerOfTheHistories {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::from_millis(self.clock_in_milliseconds.get()))
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

fn pacing_of_the_histories() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::EveryMilliseconds(
            NonZeroU64::new(INTERVAL_IN_MILLISECONDS).expect("不是 0"),
        ),
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

fn session_of(pool: &mut BuiltPool) -> (MountedSession, Devices) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (session, devices)
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

/// 顺序写之后丢掉会话：段都差不多是满的，数据单元最多的那一段一批一个搬得久。
fn pool_after_a_sequential_write(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let (mut session, mut devices) = session_of(&mut pool);
    let content: Vec<u8> = (0..DATA_UNITS_OF_THE_STARTING_POINT * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("顺序写第一个文件");
    drop(session);
    pool.devices = Some(devices);
    pool
}

fn fullest_data_segment_of(session: &MountedSession) -> CompactionRegion {
    let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
    for record in current_of(session)
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
        })
    {
        *data_units_by_segment
            .entry(record.slot.0 / 64 * 64)
            .or_default() += 1;
    }
    let (segment, _data_units) = data_units_by_segment
        .into_iter()
        .max_by_key(|(_segment, data_units)| *data_units)
        .expect("写了数据单元");
    CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(segment))
        .expect("按 64 取整的段起点")
}

/// 一批一个单元：一条意图要发很多次。
fn one_unit_a_batch() -> CompactionBatchSize {
    CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(1).expect("至少 1"))
}

/// 一批两个单元：换掉批大小开关，门关那一次的备忘就不算数（[`BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing`] 要现行那一版与开关都没变）。
fn two_units_a_batch() -> CompactionBatchSize {
    CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(2).expect("至少 1"))
}

/// 开关照水位判（门在这一支判）、越线靠只供测试的开关；选段（`None` 是产品的选段）与批大小照给；`N₀` 给了就压成它。
fn force_the_compaction_over_the_watermark(
    session: &mut MountedSession,
    region: Option<CompactionRegion>,
    batch_size: CompactionBatchSize,
    empty_cluster_segments_floor: Option<u64>,
) {
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: floor_of_the_product,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size,
            forced_region: region,
            run_forcing: CompactionRunForcing::FollowTheWatermarks,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor: empty_cluster_segments_floor.map_or(
                floor_of_the_product,
                EmptyClusterSegmentsFloor::ForcedForTests,
            ),
            relocation_publish_order,
        });
    session
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
}

fn create_one_inode(session: &mut MountedSession, devices: &mut Devices) -> CheckpointTxg {
    session
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("建一个 inode");
    current_of(session).root.checkpoint_txg
}

/// 根环里读得出、不低于 F 的根，按用户改动分到它属于哪一个用户状态（某次用户改动的 txg 起到下一次用户改动之前）：环里还留得下几个用户状态。
fn user_states_reachable(devices: &Devices, txgs_of_the_user_changes: &[CheckpointTxg]) -> usize {
    user_states_among_the_roots_from(devices, txgs_of_the_user_changes, CheckpointTxg(0))
}

/// 从 `current` 起再发 `publishes_ahead` 次发布之后还留在根环里的根里有几个用户状态：txg 为 x 的根由 x 加根环槽数那一次发布盖掉。
fn user_states_left_after_the_next_publishes(
    devices: &Devices,
    txgs_of_the_user_changes: &[CheckpointTxg],
    current: CheckpointTxg,
    publishes_ahead: u64,
) -> usize {
    let root_ring_slots =
        ROOT_RING_REGIONS * parameters().geometry.root_ring_slots_per_region.count();
    user_states_among_the_roots_from(
        devices,
        txgs_of_the_user_changes,
        CheckpointTxg((current.0 + publishes_ahead + 1).saturating_sub(root_ring_slots)),
    )
}

/// 根环里读得出、不低于 F、txg 不低于 `oldest_root_kept` 的根属于几个用户状态。
fn user_states_among_the_roots_from(
    devices: &Devices,
    txgs_of_the_user_changes: &[CheckpointTxg],
    oldest_root_kept: CheckpointTxg,
) -> usize {
    let pool_parameters = parameters();
    let roots = readable_roots(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    let floor = effective_rollback_floor(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    txgs_of_the_user_changes
        .iter()
        .enumerate()
        .filter(|(index, txg)| {
            let next = txgs_of_the_user_changes.get(index + 1);
            roots.iter().any(|root| {
                root.checkpoint_txg >= floor
                    && root.checkpoint_txg >= oldest_root_kept
                    && root.checkpoint_txg >= **txg
                    && next.is_none_or(|next_txg| root.checkpoint_txg < *next_txg)
            })
        })
        .count()
}

/// 一次叫起发了几版。
fn publishes_of_a_wake(
    session: &mut MountedSession,
    devices: &mut Devices,
    caller: &CallerOfTheHistories,
) -> u64 {
    let call = session
        .run_compaction_in_the_background(devices, pacing_of_the_histories(), caller)
        .expect("盘表是挂载的那一份");
    match call {
        BackgroundCompactionCall::NotStarted { .. } => 0,
        BackgroundCompactionCall::Ran { run, .. } => {
            assert!(
                !matches!(run.outcome, CompactionRunOutcome::Failed(_)),
                "{:?}",
                run.outcome
            );
            u64::try_from(run.versions_published.len()).expect("发布数")
        }
    }
}

/// 有用户改动时：一次用户改动之后醒 `wakes_after_each_change` 次，每一步之后环里至少 4 个用户状态。交回整理发了几次（门不是因为没发才过）。
fn publishes_while_the_ring_keeps_four_states_with_wakes_after_each_change(
    tag: &str,
    wakes_after_each_change: u64,
) -> u64 {
    let mut pool = pool_after_a_sequential_write(tag);
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, Some(region), one_unit_a_batch(), None);
    let mut txgs = Vec::new();
    for _change in 0..ROLLBACK_STATES_KEPT {
        txgs.push(create_one_inode(&mut session, &mut devices));
    }
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let mut clock = 0_u64;
    let mut compaction_publishes = 0_u64;
    for _change in 0..4 {
        txgs.push(create_one_inode(&mut session, &mut devices));
        for _wake in 0..wakes_after_each_change {
            clock += INTERVAL_IN_MILLISECONDS;
            caller.clock_in_milliseconds.set(clock);
            compaction_publishes += publishes_of_a_wake(&mut session, &mut devices, &caller);
            let states = user_states_reachable(&devices, &txgs);
            assert!(
                states >= ROLLBACK_STATES_KEPT,
                "醒 {wakes_after_each_change} 次那一臂：环里只剩 {states} 个用户状态（整理已发 {compaction_publishes} 次）"
            );
        }
    }
    compaction_publishes
}

#[test]
fn one_wake_after_each_user_change_keeps_four_user_states_in_the_ring() {
    let publishes = publishes_while_the_ring_keeps_four_states_with_wakes_after_each_change(
        "rollback-gate-one-wake",
        1,
    );
    assert!(publishes > 0, "整理真发了");
}

#[test]
fn two_wakes_after_each_user_change_keep_four_user_states_in_the_ring() {
    let publishes = publishes_while_the_ring_keeps_four_states_with_wakes_after_each_change(
        "rollback-gate-two-wakes",
        2,
    );
    assert!(publishes > 0, "整理真发了");
}

#[test]
fn three_wakes_after_each_user_change_keep_four_user_states_and_the_gate_is_what_stops_the_compaction(
) {
    let publishes = publishes_while_the_ring_keeps_four_states_with_wakes_after_each_change(
        "rollback-gate-three-wakes",
        3,
    );
    assert!(publishes > 0, "整理真发了");
}

fn is_the_compaction_stopped_at_the_rollback_gate(session: &MountedSession) -> bool {
    matches!(
        session.compaction.observations.last_stop,
        Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. })
    )
}

/// 门关的那一刻钉到槽：六次用户改动之后叫起到门关，再发 8 次发布恰好还剩 4 个用户状态，再多 1 次就只剩 3 个。
/// 余量多留一槽、少留一槽，门关的位置都差一次发布，这一条都红。
#[test]
fn the_gate_closes_exactly_where_the_next_eight_publishes_still_keep_four_user_states() {
    let mut pool = pool_after_a_sequential_write("rollback-gate-margin-of-the-ring");
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, Some(region), one_unit_a_batch(), None);
    let mut txgs = Vec::new();
    for _change in 0..6 {
        txgs.push(create_one_inode(&mut session, &mut devices));
    }
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let mut compaction_publishes = 0_u64;
    for wake in 0..WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST {
        caller
            .clock_in_milliseconds
            .set(wake * INTERVAL_IN_MILLISECONDS);
        compaction_publishes += publishes_of_a_wake(&mut session, &mut devices, &caller);
        if is_the_compaction_stopped_at_the_rollback_gate(&session) {
            break;
        }
    }
    assert!(
        matches!(
            session.compaction.observations.last_stop,
            Some(
                CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
                    states_after_the_publish_and_the_margin: 3,
                    ..
                }
            )
        ),
        "整理发了 {compaction_publishes} 次之后停在回退门上：{:?}",
        session.compaction.observations.last_stop
    );
    let current = current_of(&session).root.checkpoint_txg;
    assert_eq!(
        user_states_left_after_the_next_publishes(
            &devices,
            &txgs,
            current,
            MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS
        ),
        ROLLBACK_STATES_KEPT,
        "门关的那一刻（整理发了 {compaction_publishes} 次）：再发 {MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS} 次还剩 4 个用户状态"
    );
    assert_eq!(
        user_states_left_after_the_next_publishes(
            &devices,
            &txgs,
            current,
            MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS + 1
        ),
        ROLLBACK_STATES_KEPT - 1,
        "门关的那一刻（整理发了 {compaction_publishes} 次）：门放的这一次发布再加余量就挤掉一个，所以关了"
    );
}

/// 回退门读根环与每条有效根的树表（`mounted_session::stop_of_the_rollback_gate`）读到只坏一份进别的读那个数（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。
/// 门关之后才弄坏现行树表的一份。现行那一版与开关都没变时后台入口照备忘不起跑（[`BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing`]），不判门、不读盘，所以先钉这一次账上还是 0；
/// 开关一变备忘不算数，下一次醒真判门。这一趟停在门上，发不出发布、不写系统配置、不释放；后台入口自己读根环那一读在发成用户改动之后不走，门这一读是唯一读到现行树表的读。
/// 判别力：`crates/mutations.tsv`「可写挂载别的读：回退门读根环不进账」那一行。
#[test]
fn the_rollback_gate_closed_after_the_read_of_the_ring_of_the_mount_counts_the_damaged_tree_table()
{
    let mut pool = pool_after_a_sequential_write("rollback-gate-reads-the-tree-table");
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, Some(region), one_unit_a_batch(), None);
    for _change in 0..6 {
        create_one_inode(&mut session, &mut devices);
    }
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let mut wakes = 0;
    // 迭代上界是这个常量；跨轮带的只有会话，门关就停。
    for wake in 0..WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST {
        wakes = wake + 1;
        caller
            .clock_in_milliseconds
            .set(wake * INTERVAL_IN_MILLISECONDS);
        publishes_of_a_wake(&mut session, &mut devices, &caller);
        if is_the_compaction_stopped_at_the_rollback_gate(&session) {
            break;
        }
    }
    assert!(
        is_the_compaction_stopped_at_the_rollback_gate(&session),
        "前提：醒 {wakes} 次之后门关了：{:?}",
        session.compaction.observations.last_stop
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "前提：弄坏之前账上是空的"
    );
    let location = current_of(&session).root.tree_table.locations[0];
    let (_, device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == location.device)
        .expect("池里有这块盘");
    let mut tree_table = vec![0u8; BYTES_OF_A_TREE_TABLE];
    device
        .read_at(location.slot.to_device_offset(), &mut tree_table)
        .expect("读得出那一份");
    tree_table[BYTES_OF_A_TREE_TABLE / 2] ^= 0xFF;
    device
        .write_at(
            location.slot.to_device_offset(),
            &tree_table,
            WriteDurability::Plain,
        )
        .expect("写回改过的那一份");
    caller
        .clock_in_milliseconds
        .set(wakes * INTERVAL_IN_MILLISECONDS);
    let call_with_the_stop_still_holding = session
        .run_compaction_in_the_background(&mut devices, pacing_of_the_histories(), &caller)
        .expect("盘表是挂载的那一份");
    assert!(
        matches!(
            call_with_the_stop_still_holding,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing {
                    stop: CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. },
                },
                ..
            }
        ),
        "现行那一版与开关都没变的这一次醒照门关那一次的备忘不起跑：{call_with_the_stop_still_holding:?}"
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        0,
        "备忘挡下的这一次醒没判门、没读盘，账上还是空的"
    );
    force_the_compaction_over_the_watermark(&mut session, Some(region), two_units_a_batch(), None);
    caller
        .clock_in_milliseconds
        .set((wakes + 1) * INTERVAL_IN_MILLISECONDS);
    let call_judging_the_gate = session
        .run_compaction_in_the_background(&mut devices, pacing_of_the_histories(), &caller)
        .expect("盘表是挂载的那一份");
    match &call_judging_the_gate {
        BackgroundCompactionCall::Ran { run, .. } => {
            assert!(
                matches!(
                    run.outcome,
                    CompactionRunOutcome::Stopped(
                        CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
                    )
                ),
                "这一趟自己判了门、停在门上（这个停只有回退门交得出）：{:?}",
                run.outcome
            );
            assert!(
                run.versions_published.is_empty(),
                "门还关着，这一趟一次都不发"
            );
        }
        BackgroundCompactionCall::NotStarted { reason, .. } => {
            panic!("开关变了，备忘不算数，这一次醒起跑：{reason:?}")
        }
    }
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        1,
        "门这一读读到只坏一份的现行树表，进账一次"
    );
}

/// 留了余量，产品批大小（16）下一条意图的 1 次签发加几批照样发得完（整段 64 槽至多 5 次发布）：叫起之后不用等下一次用户改动就做完。
#[test]
fn under_the_default_batch_size_one_intent_finishes_within_what_the_gate_lets_through() {
    let mut pool = pool_after_a_sequential_write("rollback-gate-one-intent-of-the-default-batch");
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(
        &mut session,
        Some(region),
        CompactionSwitches::PRODUCT.batch_size,
        None,
    );
    for _change in 0..6 {
        create_one_inode(&mut session, &mut devices);
    }
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    for wake in 0..WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST {
        caller
            .clock_in_milliseconds
            .set(wake * INTERVAL_IN_MILLISECONDS);
        publishes_of_a_wake(&mut session, &mut devices, &caller);
    }
    assert_eq!(
        session.compaction.observations.intents_completed, 1,
        "这条意图做完了：{:?}",
        session.compaction.observations
    );
}

/// 崩溃循环里一次挂载量到的：挂载自己发了几次、这之后环里还有几个用户状态，叫起发了几次、这之后还有几个，这次挂载那一份预算算到扣了几次。
#[derive(Debug)]
struct ReadingsOfOneMountOfTheCrashLoop {
    /// 写行一次加暖机几次。
    row_and_warm_up_publishes: u64,
    /// 挂载续做盘上那条意图发了几次（回退门不判这一趟）。
    publishes_continuing_the_intent: u64,
    /// 挂载前后现行那一版的 txg 差：写行、暖机、抬 F、续做与清扫都在里面。
    txgs_advanced_by_the_mount: u64,
    states_after_the_mount: usize,
    compaction_publishes_of_the_wakes: u64,
    states_after_the_wakes: usize,
    publishes_counted_against_the_budget: u64,
}

/// 崩溃循环：六次用户改动之后一次用户改动都没有，每次挂上（产品批大小）叫起两次、丢掉会话不卸载。
fn readings_of_a_crash_loop(tag: &str, mounts: usize) -> Vec<ReadingsOfOneMountOfTheCrashLoop> {
    let mut pool = pool_after_a_sequential_write(tag);
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    let mut txgs = Vec::new();
    for _change in 0..6 {
        txgs.push(create_one_inode(&mut session, &mut devices));
    }
    let mut txg_before_the_mount = current_of(&session).root.checkpoint_txg;
    drop(session);
    pool.devices = Some(devices);
    let mut readings = Vec::new();
    for _mount in 0..mounts {
        let mut devices_of_the_mount = pool.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices_of_the_mount).expect("可写挂载");
        let (output, mut session_of_the_mount) = MountedSession::of_the_mount(mounted);
        let row_and_warm_up_publishes =
            1 + u64::try_from(output.warm_up_publishes.len()).expect("暖机次数");
        let publishes_continuing_the_intent = output
            .compaction_continuation
            .run
            .as_ref()
            .map_or(0, |run| {
                u64::try_from(run.versions_published.len()).expect("发布数")
            });
        let txg_after_the_mount = current_of(&session_of_the_mount).root.checkpoint_txg;
        let states_after_the_mount = user_states_reachable(&devices_of_the_mount, &txgs);
        force_the_compaction_over_the_watermark(
            &mut session_of_the_mount,
            Some(region),
            one_unit_a_batch(),
            None,
        );
        let caller = CallerOfTheHistories {
            clock_in_milliseconds: Cell::new(0),
        };
        let mut compaction_publishes_of_the_wakes = 0_u64;
        for wake in 0..2_u64 {
            caller
                .clock_in_milliseconds
                .set(wake * INTERVAL_IN_MILLISECONDS);
            compaction_publishes_of_the_wakes += publishes_of_a_wake(
                &mut session_of_the_mount,
                &mut devices_of_the_mount,
                &caller,
            );
        }
        readings.push(ReadingsOfOneMountOfTheCrashLoop {
            row_and_warm_up_publishes,
            publishes_continuing_the_intent,
            txgs_advanced_by_the_mount: txg_after_the_mount.0 - txg_before_the_mount.0,
            states_after_the_mount,
            compaction_publishes_of_the_wakes,
            states_after_the_wakes: user_states_reachable(&devices_of_the_mount, &txgs),
            publishes_counted_against_the_budget: session_of_the_mount
                .compaction
                .publishes_since_the_last_user_change,
        });
        txg_before_the_mount = current_of(&session_of_the_mount).root.checkpoint_txg;
        drop(session_of_the_mount);
        pool.devices = Some(devices_of_the_mount);
    }
    readings
}

/// 叫起发的整理不挤掉用户状态；后面每次挂载那一份预算都从根环读、算上前面那次挂载的整理（挂载不给新预算），门也关着，一次不发。
#[test]
fn a_crash_loop_gets_no_new_budget_from_the_mounts_and_its_wakes_push_out_no_user_state() {
    let readings = readings_of_a_crash_loop(
        "rollback-gate-crash-loop-wakes",
        MOUNTS_OF_A_CRASH_LOOP_WITHIN_THE_MARGIN,
    );
    for reading in &readings {
        assert!(
            reading.txgs_advanced_by_the_mount
                >= reading.row_and_warm_up_publishes + reading.publishes_continuing_the_intent,
            "挂载推进的 txg 罩住写行、暖机与续做：{readings:#?}"
        );
        assert!(
            reading.states_after_the_wakes
                >= ROLLBACK_STATES_KEPT.min(reading.states_after_the_mount),
            "叫起之后环里只剩 {} 个用户状态：{readings:#?}",
            reading.states_after_the_wakes
        );
    }
    assert!(
        readings[0].compaction_publishes_of_the_wakes > 0,
        "第一次挂载还有那一份里剩的预算：{readings:#?}"
    );
    assert_eq!(
        readings[2].compaction_publishes_of_the_wakes, 0,
        "门关着：{readings:#?}"
    );
    assert!(
        readings[2].publishes_counted_against_the_budget
            >= readings[0].compaction_publishes_of_the_wakes,
        "挂载不给新预算：那一份从根环读，前面那次挂载的整理算在里面：{readings:#?}"
    );
}

/// D26（后台整理与放置回收） 已定项 13 第 5 条守的「4 个不同状态」放到整段历史上：连崩 2 次，挂载自己的写行、暖机与续做（门不判这几次发布）也算在内，
/// 每次挂载之后环里至少 4 个用户状态。一次挂载自己至多发 8 次，落在余量里。
#[test]
fn a_crash_loop_keeps_four_user_states_across_the_publishes_of_its_mounts() {
    let readings = readings_of_a_crash_loop(
        "rollback-gate-crash-loop-mounts",
        MOUNTS_OF_A_CRASH_LOOP_WITHIN_THE_MARGIN,
    );
    for reading in &readings {
        assert!(
            reading.states_after_the_wakes >= ROLLBACK_STATES_KEPT,
            "环里只剩 {} 个用户状态（根环三个区域、每区 {} 槽）：{readings:#?}",
            reading.states_after_the_wakes,
            parameters().geometry.root_ring_slots_per_region.count(),
        );
        assert!(
            reading.row_and_warm_up_publishes + reading.publishes_continuing_the_intent
                <= MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS,
            "一次挂载自己发的次数落在余量里：{readings:#?}"
        );
    }
}

/// 接受的边界（D26（后台整理与放置回收） 已定项 13 第 5 条的门在 S = 8 时只留 8 槽）：崩溃次数没有上界，门关之后挂载自己的发布不经门，连崩到推过余量，用户状态照样少于 4 个。
/// 钉的是现状：丢状态只出现在门关之后挂载推过的 txg 多于余量的时候，不更早。
#[test]
fn a_crash_loop_past_the_margin_of_the_gate_loses_user_states_and_that_is_the_accepted_boundary() {
    let readings = readings_of_a_crash_loop(
        "rollback-gate-crash-loop-past-the-margin",
        MOUNTS_OF_A_CRASH_LOOP_PAST_THE_MARGIN,
    );
    let mount_losing_a_state = readings
        .iter()
        .position(|reading| reading.states_after_the_mount < ROLLBACK_STATES_KEPT)
        .unwrap_or_else(|| panic!("连崩到推过余量之后用户状态少于 4 个：{readings:#?}"));
    assert!(
        mount_losing_a_state >= 1,
        "第一次挂载之前还没有整理：{readings:#?}"
    );
    let txgs_advanced_by_the_mounts_after_the_gate_closed: u64 = readings[1..=mount_losing_a_state]
        .iter()
        .map(|reading| reading.txgs_advanced_by_the_mount)
        .sum();
    assert!(
        txgs_advanced_by_the_mounts_after_the_gate_closed > MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS,
        "门关之后挂载推了 {txgs_advanced_by_the_mounts_after_the_gate_closed} 次就丢了状态，没推过余量：{readings:#?}"
    );
}

/// 带洞的池：顺序写之后隔一个数据单元部分写一个（旧落点放掉）、再建几次 inode 让根环转过一圈，会话丢掉。之后这一池的数据段不是这次挂载开的，R 外有洞装得下 R 里的单元。
fn pool_with_holes_in_the_data_segments(tag: &str) -> BuiltPool {
    let mut pool = pool_after_a_sequential_write(tag);
    let (mut session, mut devices) = session_of(&mut pool);
    let one_byte = [0x5A_u8];
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    for rewritten in 0..DATA_UNITS_REWRITTEN_TO_LEAVE_HOLES {
        let data_unit_index = u64::try_from(2 * rewritten).expect("单元下标");
        session
            .publish_user_change(
                &mut devices,
                UserChange::WriteAtOffset {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    offset: FileOffsetInBytes(data_unit_index * payload),
                    bytes: PartialWriteBytes::new(&one_byte).expect("一个字节不是空的"),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
                },
            )
            .expect("部分写一个数据单元");
    }
    for _turn in 0..USER_CHANGES_TURNING_THE_ROOT_RING {
        create_one_inode(&mut session, &mut devices);
    }
    drop(session);
    pool.devices = Some(devices);
    pool
}

/// 破例：全空段低于底额（`N₀` 压在此刻全空段数加一，前台写从头被合取 7 拒），预算扣完的时候每趟至少发 1 次：
/// 产品的选段挑带洞的段，意图做完腾出全空段，前台写得进。前台与整理互相等的死局走得出。
#[test]
fn below_the_empty_cluster_segments_floor_a_spent_budget_still_publishes_on_every_trip_until_the_foreground_gets_through(
) {
    let mut pool = pool_with_holes_in_the_data_segments("rollback-gate-exception-with-holes");
    let (mut session, mut devices) = session_of(&mut pool);
    let empty_now = session.allocator.devices[0].empty_segments();
    force_the_compaction_over_the_watermark(
        &mut session,
        None,
        CompactionSwitches::PRODUCT.batch_size,
        Some(empty_now + 1),
    );
    session.compaction.publishes_since_the_last_user_change = 1_000;
    let content = b"foreground write while the empty segments are below the floor".to_vec();
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let mut wake_at_which_the_foreground_got_through = None;
    for wake in 1..=WAKES_OUT_OF_THE_DEADLOCK_AT_MOST {
        caller
            .clock_in_milliseconds
            .set(wake * INTERVAL_IN_MILLISECONDS);
        assert!(
            publishes_of_a_wake(&mut session, &mut devices, &caller) >= 1,
            "第 {wake} 次醒：低于底额时每趟至少发 1 次，last_stop={:?}",
            session.compaction.observations.last_stop
        );
        let attempt = session.publish_user_change(
            &mut devices,
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        );
        if attempt.is_ok() {
            wake_at_which_the_foreground_got_through = Some(wake);
            break;
        }
    }
    assert!(
        wake_at_which_the_foreground_got_through.is_some(),
        "{WAKES_OUT_OF_THE_DEADLOCK_AT_MOST} 次醒之内前台写得进：last_stop={:?}",
        session.compaction.observations.last_stop
    );
}

/// 破例不穿过「上一条无产出」：只顺序写的池上强制选一个满段，全部单元都活着，搬走开了新段、腾出的那一段只抵掉它。
/// 这条意图做完之前每趟至少发 1 次，做完之后全空段数没涨、前台照旧写不进，这一趟停在「上一条无产出」，不再发。
#[test]
fn below_the_floor_an_intent_over_a_full_segment_publishes_on_every_trip_and_then_stops_on_no_yield(
) {
    let mut pool = pool_after_a_sequential_write("rollback-gate-exception-full-segment");
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    let empty_now = session.allocator.devices[0].empty_segments();
    force_the_compaction_over_the_watermark(
        &mut session,
        Some(region),
        one_unit_a_batch(),
        Some(empty_now + 1),
    );
    let content = b"foreground write while the empty segments are below the floor".to_vec();
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let mut trips_that_published = 0_u64;
    for wake in 1..=WAKES_OUT_OF_THE_DEADLOCK_AT_MOST {
        caller
            .clock_in_milliseconds
            .set(wake * INTERVAL_IN_MILLISECONDS);
        let publishes = publishes_of_a_wake(&mut session, &mut devices, &caller);
        let attempt = session.publish_user_change(
            &mut devices,
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        );
        assert!(attempt.is_err(), "第 {wake} 次醒之后全空段数没过底额");
        if publishes == 0 {
            break;
        }
        trips_that_published += 1;
    }
    assert!(trips_that_published > 0, "这条意图真发过");
    assert_eq!(
        session.compaction.observations.intents_completed, 1,
        "这条意图做完了：{:?}",
        session.compaction.observations
    );
    assert!(
        matches!(
            session.compaction.observations.last_stop,
            Some(CompactionRunStop::NoYieldFromTheLastIntent { .. })
        ),
        "头一趟不发布的停在「上一条无产出」：{:?}",
        session.compaction.observations.last_stop
    );
}

/// 有用户改动的历史一路醒到整理停在回退门上，再把 `N₀` 压到此刻全空段数加 `segments_short_of_the_floor`。交回这个会话、盘与钟走到哪了。
fn session_below_the_floor_with_the_rollback_gate_closed(
    tag: &str,
    segments_short_of_the_floor: u64,
) -> (MountedSession, Devices, CallerOfTheHistories, u64) {
    let mut pool = pool_after_a_sequential_write(tag);
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, Some(region), one_unit_a_batch(), None);
    for _change in 0..ROLLBACK_STATES_KEPT {
        create_one_inode(&mut session, &mut devices);
    }
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let mut clock = 0_u64;
    let mut is_the_gate_closed = false;
    for _change in 0..4 {
        create_one_inode(&mut session, &mut devices);
        for _wake in 0..3 {
            clock += INTERVAL_IN_MILLISECONDS;
            caller.clock_in_milliseconds.set(clock);
            publishes_of_a_wake(&mut session, &mut devices, &caller);
            is_the_gate_closed = is_the_compaction_stopped_at_the_rollback_gate(&session);
            if is_the_gate_closed {
                break;
            }
        }
        if is_the_gate_closed {
            break;
        }
    }
    assert!(
        is_the_gate_closed,
        "前提：整理停在回退门上：{:?}",
        session.compaction.observations.last_stop
    );
    let empty_now = session.allocator.devices[0].empty_segments();
    force_the_compaction_over_the_watermark(
        &mut session,
        Some(region),
        one_unit_a_batch(),
        Some(empty_now + segments_short_of_the_floor),
    );
    (session, devices, caller, clock)
}

/// 破例过得了门：门关着、低于底额，下一趟第一次发布不判门、照发。
#[test]
fn below_the_floor_the_exception_publishes_at_least_once_past_a_closed_rollback_gate() {
    let (mut session, mut devices, caller, clock) =
        session_below_the_floor_with_the_rollback_gate_closed(
            "rollback-gate-exception-past-the-gate",
            1,
        );
    caller
        .clock_in_milliseconds
        .set(clock + INTERVAL_IN_MILLISECONDS);
    assert!(
        publishes_of_a_wake(&mut session, &mut devices, &caller) >= 1,
        "门关着、低于底额：这一趟第一次发布破例：{:?}",
        session.compaction.observations.last_stop
    );
}

/// 破例每趟都有：一趟发 1 次之后第二次发布判门、停在门上，记进备忘；现行那一版没再变，下一趟也不让备忘挡住，还是发 1 次。
/// 这几趟前台一直写不进：`N₀` 压得这几次发布腾出多少段都够不到（开关不再动，备忘认的开关因此一直对得上）。
#[test]
fn below_the_floor_every_trip_past_a_closed_rollback_gate_publishes_exactly_once() {
    let (mut session, mut devices, caller, clock) =
        session_below_the_floor_with_the_rollback_gate_closed(
            "rollback-gate-exception-on-every-trip",
            SEGMENTS_SHORT_OF_THE_FLOOR_NO_PUBLISH_MAKES_UP,
        );
    for wake in 1..=3_u64 {
        caller
            .clock_in_milliseconds
            .set(clock + wake * INTERVAL_IN_MILLISECONDS);
        assert_eq!(
            publishes_of_a_wake(&mut session, &mut devices, &caller),
            1,
            "第 {wake} 趟：第一次发布破例，第二次判门：{:?}",
            session.compaction.observations.last_stop
        );
        assert!(
            is_the_compaction_stopped_at_the_rollback_gate(&session),
            "第 {wake} 趟停在门上：{:?}",
            session.compaction.observations.last_stop
        );
    }
}

/// 破例只在前台写不进时有：`N₀` 只比此刻全空段多一段，破例那一趟腾出一段就够到了，前台写得进；现行那一版没再变，下一趟停在门上的备忘照旧算数，不起跑。
#[test]
fn a_trip_lifting_the_empty_segments_to_the_floor_ends_the_exception_and_the_stop_at_the_gate_holds(
) {
    let (mut session, mut devices, caller, clock) =
        session_below_the_floor_with_the_rollback_gate_closed(
            "rollback-gate-exception-ends-at-the-floor",
            1,
        );
    caller
        .clock_in_milliseconds
        .set(clock + INTERVAL_IN_MILLISECONDS);
    assert_eq!(
        publishes_of_a_wake(&mut session, &mut devices, &caller),
        1,
        "破例那一趟发 1 次：{:?}",
        session.compaction.observations.last_stop
    );
    assert!(
        is_the_compaction_stopped_at_the_rollback_gate(&session),
        "破例那一趟第二次发布停在门上：{:?}",
        session.compaction.observations.last_stop
    );
    assert!(
        !foreground_writes_are_below_the_empty_cluster_segments_floor(&session.allocator),
        "前提：破例那一趟把全空段抬到了 N₀：全空段 {} 段，N₀ {} 段",
        session.allocator.devices[0].empty_segments(),
        session
            .allocator
            .compaction()
            .switches
            .empty_cluster_segments_floor
            .segments()
    );
    caller
        .clock_in_milliseconds
        .set(clock + 2 * INTERVAL_IN_MILLISECONDS);
    let call = session
        .run_compaction_in_the_background(&mut devices, pacing_of_the_histories(), &caller)
        .expect("盘表是挂载的那一份");
    assert!(
        matches!(
            call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing {
                    stop:
                        CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. },
                },
                ..
            }
        ),
        "{call:?}"
    );
}

/// 后台入口问的前台 fsync 照给、钟走过一个定时兜底间隔的调用方。
struct CallerAnsweringTheForegroundFsync {
    answer_before_the_next_batch: AnswerOfTheCallerBeforeTheNextBatch,
}

impl CallerOfTheBackgroundCompaction for CallerAnsweringTheForegroundFsync {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::from_millis(INTERVAL_IN_MILLISECONDS))
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        self.answer_before_the_next_batch
    }
}

/// 全空段低于底额、前台写不进的池上叫起一趟，调用方答 `answer_before_the_next_batch`：交回这一趟发了几版，连同会话（读停因）。
fn publishes_of_a_trip_below_the_floor_answering(
    tag: &str,
    answer_before_the_next_batch: AnswerOfTheCallerBeforeTheNextBatch,
) -> (u64, MountedSession) {
    let mut pool = pool_after_a_sequential_write(tag);
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    let empty_now = session.allocator.devices[0].empty_segments();
    force_the_compaction_over_the_watermark(
        &mut session,
        Some(region),
        one_unit_a_batch(),
        Some(empty_now + SEGMENTS_SHORT_OF_THE_FLOOR_NO_PUBLISH_MAKES_UP),
    );
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_of_the_histories(),
            &CallerAnsweringTheForegroundFsync {
                answer_before_the_next_batch,
            },
        )
        .expect("盘表是挂载的那一份");
    let publishes = match call {
        BackgroundCompactionCall::NotStarted { .. } => 0,
        BackgroundCompactionCall::Ran { run, .. } => {
            u64::try_from(run.versions_published.len()).expect("发布数")
        }
    };
    (publishes, session)
}

/// 破例排在让路之前：前台有 fsync 在等，这一趟第一次发布照发，第二次才让路。
#[test]
fn below_the_floor_the_first_publish_of_a_trip_does_not_yield_to_a_waiting_fsync() {
    let (publishes, session) = publishes_of_a_trip_below_the_floor_answering(
        "rollback-gate-exception-past-a-waiting-fsync",
        AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
    );
    assert_eq!(
        publishes, 1,
        "{:?}",
        session.compaction.observations.last_stop
    );
    assert_eq!(
        session.compaction.observations.last_stop,
        Some(CompactionRunStop::YieldedToAForegroundFsync),
        "第二次发布让路"
    );
}

/// 破例只过前台 fsync，不过停线程：调用方在停线程时这一趟一次都不发，线程很快 join 得上。
#[test]
fn below_the_floor_the_first_publish_of_a_trip_stops_for_a_caller_stopping_its_thread() {
    let (publishes, session) = publishes_of_a_trip_below_the_floor_answering(
        "rollback-gate-exception-not-past-a-stopping-thread",
        AnswerOfTheCallerBeforeTheNextBatch::CallerIsStoppingItsThread,
    );
    assert_eq!(
        publishes, 0,
        "{:?}",
        session.compaction.observations.last_stop
    );
    assert_eq!(
        session.compaction.observations.last_stop,
        Some(CompactionRunStop::YieldedToTheCallerStoppingItsThread),
        "停因说得出是停线程，不是前台 fsync"
    );
}

/// 没到底额时预算扣完就不起跑（破例只在低于底额时）：同一个池不压 `N₀`。
#[test]
fn above_the_floor_a_spent_budget_starts_nothing() {
    let mut pool = pool_after_a_sequential_write("rollback-gate-no-exception-above-the-floor");
    let (mut session, mut devices) = session_of(&mut pool);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, Some(region), one_unit_a_batch(), None);
    create_one_inode(&mut session, &mut devices);
    session.compaction.publishes_since_the_last_user_change = 1_000;
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let _due_trip_is_taken_by_the_first_wake =
        publishes_of_a_wake(&mut session, &mut devices, &caller);
    caller.clock_in_milliseconds.set(INTERVAL_IN_MILLISECONDS);
    let call = session
        .run_compaction_in_the_background(&mut devices, pacing_of_the_histories(), &caller)
        .expect("盘表是挂载的那一份");
    assert!(
        matches!(
            call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,
                ..
            }
        ),
        "{call:?}"
    );
}
