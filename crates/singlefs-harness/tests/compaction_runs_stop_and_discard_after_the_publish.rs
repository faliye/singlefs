//! 整理运行（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G2、G4、G5、G8；判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
//! 第二节 S6）：会话的显式入口 `MountedSession::run_compaction` 先发一次签发意图的发布、再一批一批推，最后一批同一次发布删意图，停在判据上
//! （签发前判、停机谓词、隔离槽丢意图、预算、开关）；用户改动发成之后强制起跑时只跑至多一批（只数搬了单元的发布：丢意图、签发不算，
//! 判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J3）；合取 7 只拦前台写入类发布、拒成它自己的成员；
//! 发布返回之后整段可再分配的段发 discard，扣住、隔离、段里还有占着的槽、装着还没清扫的孤儿头都不发；这次挂载里被区域扫描拒过的段只记在
//! 这一次挂载的会话上；文件删光之后快照照样删得掉。
//! 池按 `common::build_pool` 起（第一个文件的数据单元在段 50176，那一次发布的提交内生块在段 50240），重开之后可写挂载：
//! 挂载开的聚簇段是 50304，段 50240 不是这次挂载开的，强制选它当 R。
mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, disk_snapshot, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber};
use singlefs_core::admission::EmptyClusterSegmentsBelowTheFloorOnSomeDevices;
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::compaction_plan::{
    region_is_empty_by_the_records, CompactionBatchSize, CompactionBatchUnits,
    CompactionRegionExclusion, CompactionRunForcing, CompactionRunStop, CompactionSwitches,
    EmptyClusterSegmentsFloor, ForcedReleaseChecksumMismatchAfterIssue,
    RelocationDestinationPolicy, RelocationPublishOrder,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{
    CompactionRunOutcome, MountedSession, UserChange, UserChangeRefused,
};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::recovery::walk_to_file;
use singlefs_core::region_scan::RegionScanRefusal;
use singlefs_core::transaction::{
    publish_overwrite, publish_unlink_intent, FirstFile, PoolVersion, PoolWriter, PublishError,
    TransactionOutput,
};
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::{DATA_UNIT_HEADER_BYTES, SLOT_BYTES};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 第一个文件那次发布的提交内生块落在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)）。
const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 50240;
/// 每区根槽数（mkfs 写的 S，`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`）：一环 3S 次发布之后，最早一批释放的槽回收得到。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;

fn region_of_the_first_publish() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH),
    )
    .expect("64 的倍数")
}

/// 整理开关：每个字段都写出来，除了下面几个参数，取产品值。
fn switches(
    batch_units: Option<usize>,
    forced_region: Option<CompactionRegion>,
    run_forcing: CompactionRunForcing,
    release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue,
    empty_cluster_segments_floor: EmptyClusterSegmentsFloor,
) -> CompactionSwitches {
    CompactionSwitches {
        batch_size: match batch_units {
            Some(units) => CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(units).expect("至少 1"),
            ),
            None => CompactionBatchSize::NotDecidedByTheClauses,
        },
        forced_region,
        run_forcing,
        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
        destination_policy:
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    }
}

/// 重开、可写挂载、开会话；会话的分配器装上 `compaction_switches`。
fn mounted_session(
    pool: &mut BuiltPool,
    compaction_switches: CompactionSwitches,
) -> (MountedSession, Vec<(DeviceIdentity, common::Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_compaction_switches(compaction_switches);
    (session, devices)
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

fn first_file_reads_back(pool: &BuiltPool, version: &TransactionOutput) {
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(&pool.memory_pool(), &version.root, &mut mapping_fallbacks),
        Ok(Some(file_content())),
        "第一个文件读得回"
    );
}

/// 强制选段 50240、一批两个：先发一次签发意图的发布（logged_ops 里一条），再一批一批搬到 R 按记录全空，最后一批同一次发布删意图；
/// 下一条意图签不出（那一段没有能搬的了），停在「没有合格段」。每批之前对 R 重扫：各批搬的 key 两两不同；目的地都不在 R 里；
/// 观测记下一次起跑、一条意图签发与完成、发了几批、用了几个根槽（批数加签发那一次）。
#[test]
fn a_run_on_a_forced_region_relocates_it_batch_by_batch_until_it_is_empty_by_the_records() {
    let mut pool = build_pool("compaction-run-empties-the-forced-region");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(2),
            Some(region_of_the_first_publish()),
            CompactionRunForcing::FollowTheWatermarks,
            ForcedReleaseChecksumMismatchAfterIssue::NotForced,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoEligibleSegment)
        ),
        "R 搬空之后下一条签不出：{:?}",
        run.outcome
    );
    assert!(
        run.batches_published >= 2,
        "两个以上的单元、一批两个：至少两批"
    );
    let current = current_of(&session).clone();
    assert!(
        region_is_empty_by_the_records(&current.allocation_records, region_of_the_first_publish()),
        "R 按记录全空"
    );
    let mut relocated_keys = BTreeSet::new();
    for version in &run.versions_published {
        for relocated in &version.units_relocated_by_this_publish {
            assert!(
                relocated_keys.insert(relocated.mapping_key.clone()),
                "各批搬的 key 两两不同"
            );
            assert!(
                !region_of_the_first_publish().contains_slot(relocated.destination[0].slot),
                "目的地不在活意图的 R 里"
            );
        }
    }
    let observations = session.compaction.observations;
    assert_eq!(observations.runs_started, 1);
    assert_eq!(observations.intents_issued, 1);
    assert_eq!(observations.intents_completed, 1);
    assert_eq!(observations.batches_published, run.batches_published);
    assert_eq!(
        observations.root_ring_slots_used,
        run.batches_published + 1,
        "签发那一次加每一批；删意图与最后一批同一次发布，不另用根槽"
    );
    assert_eq!(
        observations.last_stop,
        Some(CompactionRunStop::NoEligibleSegment)
    );
    assert!(
        session.allocator.compaction().live_intent().is_none(),
        "分配器这一侧撤了排他"
    );
    assert!(
        current.logged_operations.is_empty(),
        "logged_ops 里的意图删了"
    );
    let issue = &run.versions_published[0];
    assert!(
        issue.units_relocated_by_this_publish.is_empty() && issue.logged_operations.len() == 1,
        "第一次是只签发意图的发布：一个单元都不搬、logged_ops 里一条"
    );
    let last_batch = run.versions_published.last().expect("发了几批");
    assert!(
        !last_batch.units_relocated_by_this_publish.is_empty()
            && last_batch.logged_operations.is_empty(),
        "最后一批同一次发布删意图"
    );
    first_file_reads_back(&pool, &current);
}

/// 不强制选段：greedy 选到装着唯一一份用户数据的段 50176，R 外没有装用户数据的段 ⇒ 空洞 0 少于要搬的 1 个，不签（攻方 K-g），一个写都不发。
#[test]
fn a_run_stops_before_issuing_when_the_holes_outside_the_region_are_too_few() {
    let mut pool = build_pool("compaction-run-holes-too-few");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(2),
            None,
            CompactionRunForcing::FollowTheWatermarks,
            ForcedReleaseChecksumMismatchAfterIssue::NotForced,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    let operations_before = pool.stream.operations().len();
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove {
                    holes: 0,
                    units_to_move: 1
                }
            )
        ),
        "签发前判不过：{:?}",
        run.outcome
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "一个写都没发"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none(),
        "没签"
    );
}

/// 签发之后强制一次「释放前核出对不上」（判决 G6）：第一批的第一个源落点隔离、两块盘上的记录留在已分配；下一步 R 里有隔离槽，丢意图连同
/// R 排他（攻方 K-c）——丢意图是一次只删 logged_ops 那一条的发布；同一段因隔离槽没资格，停在「没有合格段」。
#[test]
fn a_quarantine_after_the_issue_drops_the_intent_and_its_region_exclusion() {
    let mut pool = build_pool("compaction-run-quarantine-drops-the-intent");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(1),
            Some(region_of_the_first_publish()),
            CompactionRunForcing::FollowTheWatermarks,
            ForcedReleaseChecksumMismatchAfterIssue::OnceOnTheFirstSourceOfTheFirstBatchForTests,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(run.batches_published, 1, "隔离之后不再发");
    assert_eq!(
        run.versions_published.len(),
        3,
        "签发、那一批、丢意图各一次发布：{:?}",
        run.outcome
    );
    let quarantined = &run.versions_published[1].quarantined_after_release_checksum_mismatch;
    assert_eq!(quarantined.len(), 2, "两块盘各一份隔离");
    let quarantined_slot = quarantined[0].placement.slot;
    assert!(
        region_of_the_first_publish().contains_slot(quarantined_slot),
        "隔离槽在 R 里"
    );
    for device in BOTH_DEVICES {
        assert!(
            session
                .allocator
                .record_for(device, quarantined_slot)
                .is_some_and(|record| !record.is_released),
            "{device:?}：隔离那一份的记录留在已分配"
        );
    }
    let observations = session.compaction.observations;
    assert_eq!(observations.intents_dropped, 1, "丢了意图");
    assert!(session.allocator.compaction().live_intent().is_none());
    assert!(
        current_of(&session).logged_operations.is_empty(),
        "丢意图那一次发布把 logged_ops 里那一条删了"
    );
    assert_eq!(
        session.allocator.region_excluded_from_placement(),
        None,
        "R 排他跟着撤了"
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoEligibleSegment)
        ),
        "{:?}",
        run.outcome
    );
}

/// 强制起跑（判决 G8 丁，开关强制）：用户改动发成之后只跑至多一批，停在「用户改动之后那一批跑完了」；那一批在改动之后、会话的 current 就是它。
#[test]
fn a_user_change_with_the_forced_start_runs_at_most_one_compaction_batch_after_it() {
    let mut pool = build_pool("compaction-run-after-a-user-change");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(1),
            Some(region_of_the_first_publish()),
            CompactionRunForcing::ForcedToStartForTests,
            ForcedReleaseChecksumMismatchAfterIssue::NotForced,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("建一个 inode");
    pool.devices = Some(devices);
    let run = published.compaction_after_the_change.expect("强制起跑");
    assert_eq!(run.batches_published, 1, "至多一批");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone)
        ),
        "{:?}",
        run.outcome
    );
    let batch = run.versions_published.last().expect("发了一批");
    assert!(
        batch.root.checkpoint_txg
            > published
                .versions_published
                .last()
                .expect("改动发了")
                .root
                .checkpoint_txg,
        "那一批在改动之后"
    );
    assert_eq!(current_of(&session), batch, "会话的 current 是那一批");
}

/// `common::build_pool` 之后重开、可写挂载开的聚簇段（默认几何）：那次挂载的写行与之后发布的提交内生块落在这一段；再重开一次，
/// 它就不是那时挂载开的段。
const CLUSTER_SEGMENT_OF_THE_MOUNT_AFTER_THE_BUILD: u64 = 50304;

/// 用户改动之后那一趟发几次（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J3；G8 丁「至多一批」）：一批 = 搬了单元的
/// 那一次发布（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2），上一条意图的收尾与签发不算批，一趟可以发三次。
/// 池建好之后挂一次、建一个 inode（它的提交内生块落在段 50304），关掉重开；重开之后那次挂载强制起跑、强制选段 50304、签发之后强制一次
/// 「释放前核出对不上」：建快照之后那一趟签发、搬一批（源落点隔离），停在「那一批跑完了」，意图留着、R 里有隔离槽。改成强制选段 50240、
/// 不再强制核出对不上，再建一个快照：那一趟先发一次丢意图（R 里有隔离槽，攻方 K-c），再在 50240 上签发、搬一批，才停在「那一批跑完了」——
/// 三次发布、记一批。
#[test]
fn a_run_after_a_user_change_publishes_the_drop_of_a_quarantined_intent_an_issue_and_one_batch() {
    let mut pool =
        build_pool("compaction-run-after-a-user-change-drops-issues-and-moves-one-batch");
    let (mut session_creating_the_inode, mut devices_of_that_mount) = mounted_session(
        &mut pool,
        switches(
            None,
            None,
            CompactionRunForcing::FollowTheWatermarks,
            ForcedReleaseChecksumMismatchAfterIssue::NotForced,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    session_creating_the_inode
        .publish_user_change(
            &mut devices_of_that_mount,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("池建好之后那次挂载里建一个 inode");
    drop(session_creating_the_inode);
    pool.devices = Some(devices_of_that_mount);
    let region_of_the_mount_after_the_build = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(CLUSTER_SEGMENT_OF_THE_MOUNT_AFTER_THE_BUILD),
    )
    .expect("64 的倍数");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(1),
            Some(region_of_the_mount_after_the_build),
            CompactionRunForcing::ForcedToStartForTests,
            ForcedReleaseChecksumMismatchAfterIssue::OnceOnTheFirstSourceOfTheFirstBatchForTests,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    let quarantining = session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建快照")
        .compaction_after_the_change
        .expect("强制起跑");
    assert!(
        matches!(
            quarantining.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone)
        ) && quarantining.batches_published == 1
            && quarantining.versions_published.len() == 2,
        "签发、搬一批，停在那一批跑完了：{:?}，发了 {} 次、记 {} 批（这次挂载开的聚簇段 {:?}）",
        quarantining.outcome,
        quarantining.versions_published.len(),
        quarantining.batches_published,
        session.allocator.cluster_segments()
    );
    assert_eq!(
        quarantining.versions_published[1]
            .quarantined_after_release_checksum_mismatch
            .len(),
        2,
        "那一批的源落点两块盘各一份隔离"
    );
    assert_eq!(
        session
            .allocator
            .compaction()
            .live_intent()
            .map(|live| live.intent.region),
        Some(region_of_the_mount_after_the_build),
        "意图留着，R 里有隔离槽"
    );
    session.allocator.set_compaction_switches(switches(
        Some(1),
        Some(region_of_the_first_publish()),
        CompactionRunForcing::ForcedToStartForTests,
        ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        EmptyClusterSegmentsFloor::OfTheFirstVersion,
    ));
    let run = session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("再建一个快照")
        .compaction_after_the_change
        .expect("强制起跑");
    pool.devices = Some(devices);
    let relocated_by_each_publish: Vec<usize> = run
        .versions_published
        .iter()
        .map(|version| version.units_relocated_by_this_publish.len())
        .collect();
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone)
        ),
        "停在那一批跑完了：{:?}（每次发布搬的个数 {relocated_by_each_publish:?}）",
        run.outcome
    );
    assert_eq!(
        run.versions_published.len(),
        3,
        "丢意图、签发、一批各一次发布（每次发布搬的个数 {relocated_by_each_publish:?}）"
    );
    assert_eq!(
        run.batches_published, 1,
        "只记搬了单元的那一次：丢意图与签发不算批"
    );
    let dropping = &run.versions_published[0];
    assert!(
        dropping.units_relocated_by_this_publish.is_empty()
            && dropping.logged_operations.is_empty(),
        "丢意图那一次：一个单元都不搬，logged_ops 里那一条删了"
    );
    let issuing = &run.versions_published[1];
    assert!(
        issuing.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&issuing.logged_operations)
                .is_some_and(|(_, intent)| intent.region == region_of_the_first_publish()),
        "签发那一次在 50240 上签：一个单元都不搬，logged_ops 里是 50240 那一条"
    );
    assert!(
        !run.versions_published[2]
            .units_relocated_by_this_publish
            .is_empty(),
        "搬一批的那一次搬了单元"
    );
    assert_eq!(
        session.compaction.observations.intents_dropped, 1,
        "丢了 50304 上那一条意图"
    );
}

/// 只供测试的开关叫停：显式入口一个写都不发，停在「开关叫停」。
#[test]
fn a_run_stopped_by_the_test_only_switch_writes_nothing() {
    let mut pool = build_pool("compaction-run-forced-to-stop");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(1),
            Some(region_of_the_first_publish()),
            CompactionRunForcing::ForcedToStopForTests,
            ForcedReleaseChecksumMismatchAfterIssue::NotForced,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    let operations_before = pool.stream.operations().len();
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::ForcedToStopForTests)
        ),
        "{:?}",
        run.outcome
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "一个写都没发"
    );
}

/// 产品路径：批大小条款没写，签任何意图、发任何一个写之前停下（`BatchSizeNotDecidedByTheClauses`）。
#[test]
fn a_run_on_the_product_path_stops_before_issuing_because_the_batch_size_is_not_decided() {
    let mut pool = build_pool("compaction-run-product-batch-size");
    let (mut session, mut devices) = mounted_session(&mut pool, CompactionSwitches::PRODUCT);
    let operations_before = pool.stream.operations().len();
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses)
        ),
        "{:?}",
        run.outcome
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "一个写都没发"
    );
    assert!(session.allocator.compaction().live_intent().is_none());
}

/// 同一个进程里的池装上 `N₀` 很大的合取 7 开关（判决 G6「强制合取 7 的 `N₀`」）。
fn pool_with_an_empty_cluster_segments_floor_no_device_meets(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    pool.allocator.set_compaction_switches(switches(
        None,
        None,
        CompactionRunForcing::FollowTheWatermarks,
        ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        EmptyClusterSegmentsFloor::ForcedForTests(1 << 40),
    ));
    pool
}

/// 合取 7 只拦前台写入类发布（D3（空间分配） 已定项 7 登记表第 7 行，用户定）：覆盖写被拒成合取 7 自己的成员（不借空间准入不够那一个：
/// 调用方分得出「先整理」与「真没空间」），带着 `N₀` 与两块盘各自的全空段数，在任何写之前。
#[test]
fn a_foreground_write_is_refused_by_the_empty_cluster_segments_floor() {
    let mut pool = pool_with_an_empty_cluster_segments_floor_no_device_meets(
        "compaction-floor-refuses-the-overwrite",
    );
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let content = vec![5u8; 1000];
    let refused = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            INSTANCE,
        )
    };
    let Err(PublishError::EmptyClusterSegmentsBelowTheFloor(
        EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
            floor_in_segments,
            short_devices,
        },
    )) = refused
    else {
        panic!("合取 7 拒成它自己的成员：{refused:?}");
    };
    assert_eq!(floor_in_segments, 1 << 40, "带着开关给的 N₀");
    assert!(
        short_devices
            .iter()
            .all(|short| short.empty_cluster_segments < floor_in_segments),
        "每一块短的盘报的全空段数都在 N₀ 之下：{short_devices:?}"
    );
    assert_eq!(
        short_devices
            .iter()
            .map(|short| short.device)
            .collect::<Vec<_>>(),
        BOTH_DEVICES.to_vec(),
        "两块盘都短"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "盘上逐项不变"
    );
}

/// 会话里的前台写被合取 7 拒：原样交回 `Publish`、带合取 7 自己的成员，不为它推抬 F（`mounted_session::refusal_is_short_of_space`
/// 对它答否）——全空段数按记录口径数（defer 里已释放的槽本来就不算占用），抬 F 把 defer 里的槽回收不改这个数，推了只是多发几次空发布。
/// 先在产品值下覆盖写六次，让 F 抬得动（有足够的非空根，推得出抬 F 的空发布），再装上 `N₀` 很大的合取 7 开关覆盖写一次。
/// 判别力：合取 7 被拒仍算空间不够时，会话推抬 F 再判，交回的不是 `Publish`、录制流多出抬 F 那几次空发布的写。
#[test]
fn a_foreground_write_refused_by_the_empty_cluster_segments_floor_in_a_session_pushes_no_floor_raise(
) {
    let mut pool = build_pool("compaction-floor-refusal-pushes-no-floor-raise");
    let (mut session, mut devices) = mounted_session(&mut pool, CompactionSwitches::PRODUCT);
    for seed in [31u8, 37, 41, 43, 47, 53] {
        session
            .publish_user_change(
                &mut devices,
                UserChange::Overwrite(FirstFile {
                    content: &[seed; 1000],
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                }),
            )
            .expect("产品值下有全空段：覆盖写发得成");
    }
    session.allocator.set_compaction_switches(switches(
        None,
        None,
        CompactionRunForcing::FollowTheWatermarks,
        ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        EmptyClusterSegmentsFloor::ForcedForTests(1 << 40),
    ));
    let operations_before = pool.stream.operations().len();
    let txg_before = current_of(&session).root.checkpoint_txg;
    let refused = session.publish_user_change(
        &mut devices,
        UserChange::Overwrite(FirstFile {
            content: &[5u8; 1000],
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        }),
    );
    let Err(UserChangeRefused::Publish {
        cause: PublishError::EmptyClusterSegmentsBelowTheFloor(below_the_floor),
        floor_raises,
        versions_published,
    }) = refused
    else {
        panic!("合取 7 被拒原样交回 Publish、不推抬 F：{refused:?}");
    };
    assert_eq!(
        below_the_floor.floor_in_segments,
        1 << 40,
        "带着开关给的 N₀"
    );
    assert!(floor_raises.is_empty(), "一串抬 F 都没推：{floor_raises:?}");
    assert!(
        versions_published.is_empty(),
        "被拒之前一版都没发：{versions_published:?}"
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "一个写都没发：抬 F 的空发布一次都没有"
    );
    assert_eq!(
        current_of(&session).root.checkpoint_txg,
        txg_before,
        "会话的现行那一版没动"
    );
}

/// 合取 7 不拦删除类（D3 已定项 7 登记表第 7 行）：同一个开关下 unlink 的意图照样发成。
#[test]
fn an_unlink_intent_is_exempt_from_the_empty_cluster_segments_floor() {
    let mut pool = pool_with_an_empty_cluster_segments_floor_no_device_meets(
        "compaction-floor-exempts-an-unlink",
    );
    let previous = pool.output.clone();
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let intent = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(1),
        FIXED_WRITE_TIME_SECONDS + 60,
        INSTANCE,
    );
    assert!(intent.is_ok(), "删除类免合取 7：{intent:?}");
}

/// 一块盘、三个整段单元区的空闲图（默认环下从 50176 起），段 50240 里一个两槽落点已分配、释放、回收过。
fn device_map_with_a_reclaimed_placement_in_the_middle_segment() -> DeviceFreeMap {
    let mut device_map = DeviceFreeMap::new(DeviceIdentity(0), (50176 + 3 * 64) * 16384);
    device_map.mark_allocated(SlotNumber(50250), 2);
    device_map.mark_released(SlotNumber(50250), 2);
    device_map.mark_reclaimed(SlotNumber(50250), 2);
    device_map
}

/// discard 的段（判决 G5）：回收之后整段可再分配的段交出去一次，之后不再交。
#[test]
fn a_segment_reclaimed_as_a_whole_is_handed_out_for_discard_once() {
    let mut device_map = device_map_with_a_reclaimed_placement_in_the_middle_segment();
    assert_eq!(
        device_map.take_the_segments_to_discard(),
        vec![SlotNumber(50240)],
        "整段发"
    );
    assert!(
        device_map.take_the_segments_to_discard().is_empty(),
        "发过的不再发"
    );
}

/// 抬 F 扣住（F 还没在每块盘上生效）的段不发；放开扣住之后再发。
#[test]
fn a_segment_held_until_the_floor_takes_effect_is_discarded_only_after_the_hold_is_released() {
    let mut device_map = device_map_with_a_reclaimed_placement_in_the_middle_segment();
    device_map.hold_until_floor_takes_effect(SlotNumber(50250), 2);
    assert!(
        device_map.take_the_segments_to_discard().is_empty(),
        "扣着不发"
    );
    device_map.release_holds();
    assert_eq!(
        device_map.take_the_segments_to_discard(),
        vec![SlotNumber(50240)],
        "放开之后发"
    );
}

/// 影子账隔离着一个槽的段不发。
#[test]
fn a_segment_holding_an_isolated_slot_is_not_discarded() {
    let mut device_map = device_map_with_a_reclaimed_placement_in_the_middle_segment();
    device_map.isolate(SlotNumber(50260), 1);
    assert!(
        device_map.take_the_segments_to_discard().is_empty(),
        "有隔离槽不发"
    );
}

/// 段里还有占着的槽（另一个落点仍分配）就不发：discard 只发整段，不发段里的零散空槽。
#[test]
fn a_segment_with_a_slot_still_allocated_is_not_discarded() {
    let mut device_map = device_map_with_a_reclaimed_placement_in_the_middle_segment();
    device_map.mark_allocated(SlotNumber(50290), 1);
    assert!(
        device_map.take_the_segments_to_discard().is_empty(),
        "段里还有已分配的槽不发"
    );
}

/// 一环用户改动（每次建一个 inode）各自发布返回之后发了 discard 的段（交回 `Applied` 与 `Unsupported` 的都算）与没发、记成装着还没清扫的
/// 孤儿头的段；每一次发 discard 那一刻核段里没有未释放的记录。
struct DiscardsOfOneRevolution {
    discarded: Vec<(DeviceIdentity, SlotNumber)>,
    withheld_holding_unswept_orphan_heads: Vec<(DeviceIdentity, SlotNumber)>,
}

fn user_changes_for_one_revolution_and_their_discards(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, common::Recorded)>,
) -> DiscardsOfOneRevolution {
    let mut discards = DiscardsOfOneRevolution {
        discarded: Vec::new(),
        withheld_holding_unswept_orphan_heads: Vec::new(),
    };
    for _publish in 0..(3 * ROOT_RING_SLOTS_PER_REGION + 2) {
        let published = session
            .publish_user_change(
                devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
            )
            .expect("建一个 inode");
        let discarded_by_this_change: Vec<(DeviceIdentity, SlotNumber)> = published
            .discards_after_the_change
            .applied
            .iter()
            .chain(published.discards_after_the_change.unsupported.iter())
            .copied()
            .collect();
        let current = current_of(session);
        for (_, segment) in &discarded_by_this_change {
            assert!(
                !current
                    .allocation_records
                    .iter()
                    .any(|record: &AllocationRecord| {
                        !record.is_released
                            && record.slot.0 >= segment.0
                            && record.slot.0 < segment.0 + 64
                    }),
                "发 discard 那一刻，段 {} 里没有未释放的记录",
                segment.0
            );
        }
        discards.discarded.extend(discarded_by_this_change);
        discards.withheld_holding_unswept_orphan_heads.extend(
            published
                .discards_after_the_change
                .withheld_holding_unswept_orphan_heads
                .iter()
                .copied(),
        );
    }
    discards
}

/// 整理搬空段 50240、再推一环发布让它的槽都回收之后：会话发布返回时对两块盘各发那一段的 discard，长度一整段；发过的段里没有未释放的记录。
#[test]
fn a_segment_emptied_by_the_compaction_is_discarded_on_both_devices_once_its_slots_are_reclaimed() {
    let mut pool = build_pool("compaction-run-then-discard");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(4),
            Some(region_of_the_first_publish()),
            CompactionRunForcing::FollowTheWatermarks,
            ForcedReleaseChecksumMismatchAfterIssue::NotForced,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    session.run_compaction(&mut devices).expect("整理一趟");
    let discards = user_changes_for_one_revolution_and_their_discards(&mut session, &mut devices);
    pool.devices = Some(devices);
    for device in BOTH_DEVICES {
        assert_eq!(
            discards
                .discarded
                .iter()
                .filter(|(discarded_device, segment)| *discarded_device == device
                    && *segment == SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH))
                .count(),
            1,
            "{device:?}：搬空的那一段发了一次 discard"
        );
    }
    assert!(
        discards.withheld_holding_unswept_orphan_heads.is_empty(),
        "没有孤儿头：一段都不扣"
    );
}

/// 码 1 数据单元头里诞生代号、事务号的偏移（D18（块里携带什么信息） 已定项 7 / 已定项 18 的偏移表；事务号 6 字节）。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
const DATA_UNIT_TRANSACTION_OFFSET: usize = 95;
/// 孤儿头的诞生代号与事务号：都远在实例 1 那一行的 T 与 W 之上，按那一行判未发布（D18 已定项 11「已发布谓词」）。
const ORPHAN_BIRTH_TXG_AND_TRANSACTION: u64 = 1_000_000;

/// 第一个文件那个数据单元（实例 1 写的）所在的那一槽的字节，头里的诞生代号与事务号换成 [`ORPHAN_BIRTH_TXG_AND_TRANSACTION`]、头校验和重封：
/// 一个实例 1 写了、journal 记录没落盘的单元头（掉电留下的孤儿），清扫按实例 1 那一行判它未发布、要抹。
fn orphan_slot_of_instance_one(
    session: &MountedSession,
    devices: &[(DeviceIdentity, common::Recorded)],
) -> Vec<u8> {
    let data_unit_slot = current_of(session).data_pointers[0].locations[0].slot;
    let mut bytes = vec![0u8; usize::try_from(SLOT_BYTES).expect("一槽")];
    devices[0]
        .1
        .read_at(data_unit_slot.to_device_offset(), &mut bytes)
        .expect("读得出第一个文件的数据单元");
    bytes[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
        .copy_from_slice(&ORPHAN_BIRTH_TXG_AND_TRANSACTION.to_le_bytes());
    bytes[DATA_UNIT_TRANSACTION_OFFSET..DATA_UNIT_TRANSACTION_OFFSET + 6]
        .copy_from_slice(&ORPHAN_BIRTH_TXG_AND_TRANSACTION.to_le_bytes()[..6]);
    seal_header_checksum(
        &mut bytes,
        usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105"),
    );
    bytes
}

/// 主 agent 2026-10-15 定（零轮）：discard 的结局是三态之一，不能拿它当擦除——装着还没清扫的孤儿头的段不发 discard。整理搬空段 50240 之后，
/// 在段里第一个槽放一个实例 1 的孤儿头（实例 1 那一行还在实例表里、按它判未发布）；一次一次推用户改动，到那一段整段可再分配的那一次：
/// 两块盘上都不发 discard、记成装着还没清扫的孤儿头，那一刻孤儿头原样在盘上（等清扫抹）。之后那一段照常可再分配，不再追着它。
#[test]
fn a_segment_holding_an_unswept_orphan_head_is_withheld_from_the_discard() {
    let mut pool = build_pool("compaction-run-then-discard-withheld-for-an-orphan-head");
    let (mut session, mut devices) = mounted_session(
        &mut pool,
        switches(
            Some(4),
            Some(region_of_the_first_publish()),
            CompactionRunForcing::FollowTheWatermarks,
            ForcedReleaseChecksumMismatchAfterIssue::NotForced,
            EmptyClusterSegmentsFloor::OfTheFirstVersion,
        ),
    );
    session.run_compaction(&mut devices).expect("整理一趟");
    let segment = SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH);
    let orphan = orphan_slot_of_instance_one(&session, &devices);
    for (_, device) in &mut devices {
        device
            .write_at(segment.to_device_offset(), &orphan, WriteDurability::Plain)
            .expect("孤儿头写得进");
    }
    // 迭代上界是一环加两次发布（同上一条用例）；提前出口是那一段被扣下的那一次。
    let mut withheld_on: Vec<DeviceIdentity> = Vec::new();
    for _publish in 0..(3 * ROOT_RING_SLOTS_PER_REGION + 2) {
        let published = session
            .publish_user_change(
                &mut devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
            )
            .expect("建一个 inode");
        let discards = &published.discards_after_the_change;
        for device in BOTH_DEVICES {
            assert!(
                !discards.applied.contains(&(device, segment))
                    && !discards.unsupported.contains(&(device, segment)),
                "{device:?}：装着孤儿头的那一段没发 discard"
            );
        }
        withheld_on = discards
            .withheld_holding_unswept_orphan_heads
            .iter()
            .filter(|(_, withheld_segment)| *withheld_segment == segment)
            .map(|(device, _)| *device)
            .collect();
        if !withheld_on.is_empty() {
            break;
        }
    }
    assert_eq!(
        withheld_on,
        BOTH_DEVICES.to_vec(),
        "那一段整段可再分配的那一次，两块盘上都记成装着还没清扫的孤儿头"
    );
    for (device_identity, device) in &devices {
        let mut read_back = vec![0u8; orphan.len()];
        device
            .read_at(segment.to_device_offset(), &mut read_back)
            .expect("读得出");
        assert_eq!(read_back, orphan, "{device_identity:?}：孤儿头原样留着");
    }
    pool.devices = Some(devices);
}

/// 丁 1 交判第 2 点：被区域扫描拒过的段只记这一次挂载（住在这次挂载的会话上）；重开再挂载之后表是空的。
#[test]
fn the_segments_refused_by_the_region_scan_are_forgotten_by_the_next_mount() {
    let mut pool = build_pool("compaction-refused-segments-last-one-mount");
    let (mut session, devices) = mounted_session(&mut pool, CompactionSwitches::PRODUCT);
    session
        .compaction
        .segments_refused_by_the_region_scan
        .note_refusal(
            region_of_the_first_publish(),
            RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
                device: DeviceIdentity(0),
                slot: SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH - 1),
                span_slots: 2,
            },
        );
    assert!(session
        .compaction
        .segments_refused_by_the_region_scan
        .contains(SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH)));
    pool.devices = Some(devices);
    let (next_session, devices_of_the_next_mount) =
        mounted_session(&mut pool, CompactionSwitches::PRODUCT);
    pool.devices = Some(devices_of_the_next_mount);
    assert!(
        !next_session
            .compaction
            .segments_refused_by_the_region_scan
            .contains(SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH)),
        "下一次挂载的表是空的"
    );
}

/// 文件删光之后删快照（丙 2 交判第 5 条，主 agent 定是缺陷）：有树而文件集合为空的一版上删快照照样发成，删完之后树表里没有快照。
#[test]
fn a_snapshot_is_destroyed_on_a_version_whose_files_are_all_deleted() {
    let mut pool = build_pool("compaction-session-destroys-a-snapshot-after-the-files-are-gone");
    let (mut session, mut devices) = mounted_session(&mut pool, CompactionSwitches::PRODUCT);
    session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let snapshot = current_of(&session)
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("建了一个快照")
        .tree;
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(1),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
        )
        .expect("删第一个文件");
    assert!(
        matches!(session.current, PoolVersion::WithTreesAndNoFile(_)),
        "文件删光了"
    );
    let destroyed =
        session.publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot });
    pool.devices = Some(devices);
    assert!(destroyed.is_ok(), "删得掉：{destroyed:?}");
    assert!(
        !current_of(&session)
            .tree_table_entries
            .iter()
            .any(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT),
        "树表里没有快照了"
    );
}
