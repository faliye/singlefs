//! 回锁时签发：意图的量取现行那一版的，窗口里被区扫描拒的段、被腾空的段、进了隔离槽的段、落进了钉住单元的段与记成这次挂载开过的段放弃签发、行程照旧，
//! 填满空段的写之后取现行那一版的 segs0，签发前判的空洞数取现行那一版的记录，R 的记录变了时要搬数取现行那一版 R 里的未释放记录数，
//! 锁外为这一批读的单元在挂载之外的盘表上都读不出。

use super::*;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::compaction_plan::holes_outside_the_region_in_segments_holding_user_data;
use singlefs_core::mounted_session::central_mapping_as_the_compaction_reads_it;
use singlefs_core::transaction::{PartialWriteBytes, TransactionUnit};
use singlefs_core::unit::UNIT_CLASS_DATA;

/// 另一次挂载里顺序写第一个文件，重挂、套上攒批的会话：交回会话、盘，与第一块盘上两槽的数据单元最多的那一段。
fn session_after_a_sequential_write_in_an_earlier_mount(
    pool: &mut BuiltPool,
) -> (MountedSessionWithPendingChanges, Devices, CompactionRegion) {
    let (mut pending_of_the_write, mut devices_of_the_write) =
        session_with_pending_changes_of(pool);
    let content: Vec<u8> = (0..DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT
        * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    pending_of_the_write
        .publish_user_change(
            &mut devices_of_the_write,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            CallerMonotonicClockReading(Duration::ZERO),
        )
        .expect("顺序写第一个文件");
    drop(pending_of_the_write);
    pool.devices = Some(devices_of_the_write);
    let (pending, devices) = session_with_pending_changes_of(pool);
    let mut data_units_by_segment: std::collections::BTreeMap<u64, u64> =
        std::collections::BTreeMap::new();
    for record in current_of(pending.session())
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
    let (fullest_segment, _data_units_in_it) = data_units_by_segment
        .into_iter()
        .max_by_key(|(segment, data_units)| (*data_units, std::cmp::Reverse(*segment)))
        .expect("写过数据单元");
    (pending, devices, region_starting_at(fullest_segment))
}

/// 签发那一支回锁（D26（后台整理与放置回收） 已定项 13 第 5 条）：锁外期间前台建了一个 inode（现行那一版变了，选中那一段的记录没变），
/// 回锁在现行那一版上重判资格与签发前那一判照常签；意图里的量取现行那一版的——`c0` 是签发那次发布接着的那一版的 txg（不是票里那一版的），
/// `segs0` 是那一版按记录数出的全空段数；签发之后池级 checker 在 `judge` 上判不红（I-5.6）。
fn the_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version_judged_by(
    judge: JudgeOfAPoolCheckerJudgement,
) {
    let mut pool = build_pool("relock-issue-takes-the-quantities-of-the-current-version");
    let (mut pending, mut devices, region) =
        session_after_a_sequential_write_in_an_earlier_mount(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region),
        1,
    );
    let caller = CallerForTests::with_nothing_waiting();
    create_one_inode(&mut pending, &mut devices, &caller);
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let txg_of_the_ticket = current_of(pending.session()).root.checkpoint_txg;
    create_one_inode(&mut pending, &mut devices, &caller);
    let version_before_the_issue = current_of(pending.session()).clone();
    assert!(
        version_before_the_issue.root.checkpoint_txg > txg_of_the_ticket,
        "前提：锁外期间现行那一版变了"
    );
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        run.versions_published.len(),
        1,
        "回锁签发了：{:?}",
        run.outcome
    );
    let (_, issued) =
        live_compaction_intent_of_the_entries(&run.versions_published[0].logged_operations)
            .expect("签发那一版里有活意图");
    assert_eq!(issued.region, region);
    assert_eq!(
        issued.allocation_generation_at_issue, version_before_the_issue.root.checkpoint_txg,
        "c0 取签发那次发布接着的那一版（现行那一版）的 txg"
    );
    assert_eq!(
        issued.empty_cluster_segments_at_issue,
        empty_cluster_segments_by_the_records(
            &version_before_the_issue.allocation_records,
            unit_area_of_the_first_device(pending.session()),
        ),
        "segs0 取现行那一版按记录数出的全空段数"
    );
    if let Some(violated) = invariants_violated_on(
        &pool,
        "relock-issue-takes-the-quantities-of-the-current-version",
        judge,
    ) {
        assert_eq!(
            violated,
            Vec::<&str>::new(),
            "池级 checker 不红（{judge:?}）"
        );
    }
}

/// 照调用方的多机配置判池级 checker（配了 GPU 而这个进程拿不到卡记「本次未跑」）。
#[test]
fn an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version() {
    the_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version_judged_by(
        JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
    );
}

/// 在只开 CPU 的多机配置下由 CPU 判器判池级 checker（不打「本次未跑」）。
#[test]
fn an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version_on_the_cpu_judge(
) {
    the_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version_judged_by(
        JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
    );
}

/// 起池、另一次挂载里顺序写第一个文件，重挂之后强制起跑、强制选数据单元最多的那一段、一批一个，建一个 inode（「该跑了」），取输入：
/// 签发那一批交到锁外（同 [`the_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version_judged_by`] 的起法）。
/// 交回会话、盘、选中那一段、票与票里那一版。
fn ticket_issuing_on_the_fullest_segment_of_an_earlier_mount(
    pool: &mut BuiltPool,
    caller: &CallerForTests,
) -> (
    MountedSessionWithPendingChanges,
    Devices,
    CompactionRegion,
    Box<BackgroundCompactionBatchTicket>,
    TransactionOutput,
) {
    let (mut pending, mut devices, region) =
        session_after_a_sequential_write_in_an_earlier_mount(pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region),
        1,
    );
    create_one_inode(&mut pending, &mut devices, caller);
    let ticket = ticket_taken(&mut pending, &mut devices, caller);
    let version_of_the_ticket = current_of(pending.session()).clone();
    (pending, devices, region, ticket, version_of_the_ticket)
}

/// 一版里 `region` 那块盘上起点在 `region` 里、未释放的分配记录条数。
fn unreleased_records_starting_in(version: &TransactionOutput, region: CompactionRegion) -> usize {
    version
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .count()
}

/// 签发那一支回锁重判资格（D26（后台整理与放置回收） 已定项 13 第 5 条）：锁外期间选中的那一段记成扫描拒过、前台建了一个 inode（现行那一版变了，
/// 那一段的记录没变）；回锁在现行那一版上重判资格，扫描拒过的段没资格：这一次不签、停在 `ChosenSegmentNoLongerEligibleAtTheRelock`，
/// 一个写都不发，观测记一次没资格换段，这一趟还在途（下一批重选）。
#[test]
fn an_issue_at_the_relock_on_a_segment_refused_by_the_region_scan_in_the_window_is_given_up_and_keeps_the_trip(
) {
    let mut pool = build_pool("relock-issue-on-a-segment-refused-in-the-window");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, region, ticket, _version_of_the_ticket) =
        ticket_issuing_on_the_fullest_segment_of_an_earlier_mount(&mut pool, &caller);
    pending
        .session_for_tests_only()
        .compaction
        .segments_refused_by_the_region_scan
        .note_refusal(
            region,
            RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey {
                device: region.device(),
                slots: [region.start_slot(), SlotNumber(region.start_slot().0 + 2)],
            },
        );
    create_one_inode(&mut pending, &mut devices, &caller);
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock,
        "扫描拒过的段在现行那一版上没资格：这一次不签"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .observations
            .issues_given_up_for_a_region_no_longer_eligible_at_the_relock,
        1,
        "观测记一次没资格换段"
    );
    assert!(
        pending
            .session()
            .compaction
            .has_a_background_trip_under_way(),
        "没资格这一次不签，这一趟还在途：下一批重选"
    );
}

/// 签发那一支回锁重判资格的另一样（D26（后台整理与放置回收） 已定项 13 第 5 条）：锁外期间覆盖写第一个文件，选中那一段（另一次挂载里顺序写第一个文件、
/// 数据单元最多的那一段）里未释放的记录全放掉；回锁在现行那一版上重判资格，R 里没有未释放的记录就没资格：这一次不签、一个写都不发。
#[test]
fn an_issue_at_the_relock_on_a_segment_emptied_in_the_window_is_given_up() {
    let mut pool = build_pool("relock-issue-on-a-segment-emptied-in-the-window");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, region, ticket, version_of_the_ticket) =
        ticket_issuing_on_the_fullest_segment_of_an_earlier_mount(&mut pool, &caller);
    assert!(
        unreleased_records_starting_in(&version_of_the_ticket, region) > 0,
        "前提：取输入那一刻选中那一段里有未释放的记录"
    );
    make_the_foreground_change(
        ForegroundChangeInTheWindow::OverwriteOfTheFirstFile,
        &mut pending,
        &mut devices,
        &caller,
    );
    assert_eq!(
        unreleased_records_starting_in(current_of(pending.session()), region),
        0,
        "前提：覆盖写第一个文件之后选中那一段里没有未释放的记录"
    );
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock,
        "R 里没有未释放的记录：这一次不签"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
}

/// 签发那一支回锁时意图里的 `segs0` 取现行那一版的（D26（后台整理与放置回收） 已定项 13 第 5 条「各个量照现行那一版取」）：锁外期间前台顺序写
/// 另一个 inode（取输入之前建的那一个）几十个数据单元，按记录口径的全空聚簇段数在票里那一版与现行那一版上不同；回锁照常签，
/// 意图里的 `segs0` 是现行那一版按记录数出的那一个、不是票里那一版的。
#[test]
fn an_issue_at_the_relock_after_a_write_filling_empty_segments_takes_segs0_of_the_current_version()
{
    let mut pool = build_pool("relock-issue-takes-segs0-of-the-current-version");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, region, ticket, version_of_the_ticket) =
        ticket_issuing_on_the_fullest_segment_of_an_earlier_mount(&mut pool, &caller);
    let content_of_the_other_inode: Vec<u8> = (0..DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT
        * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 239).expect("小于 256"))
        .collect();
    pending
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER + 1),
                content: &content_of_the_other_inode,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            caller.clock_reading(),
        )
        .expect("锁外期间顺序写另一个 inode");
    let unit_area = unit_area_of_the_first_device(pending.session());
    let version_before_the_issue = current_of(pending.session()).clone();
    let segs0_of_the_current_version = empty_cluster_segments_by_the_records(
        &version_before_the_issue.allocation_records,
        unit_area,
    );
    assert_ne!(
        segs0_of_the_current_version,
        empty_cluster_segments_by_the_records(&version_of_the_ticket.allocation_records, unit_area),
        "前提：全空聚簇段数在票里那一版与现行那一版上不同"
    );
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        run.versions_published.len(),
        1,
        "回锁签发了：{:?}",
        run.outcome
    );
    let (_, issued) =
        live_compaction_intent_of_the_entries(&run.versions_published[0].logged_operations)
            .expect("签发那一版里有活意图");
    assert_eq!(issued.region, region);
    assert_eq!(
        issued.empty_cluster_segments_at_issue, segs0_of_the_current_version,
        "segs0 取现行那一版按记录数出的全空段数"
    );
}

/// 锁外那一段回会话锁读盘时交进来的盘表不是挂载核过的那一份（两块盘换了次序）：一个盘都不读、每个单元都当读不到；
/// 同一串交挂载的那一份读得到（D26（后台整理与放置回收） 已定项 13 第 5 条：回锁时盘表照样要对上才发）。
#[test]
fn units_read_for_the_batch_out_of_the_lock_on_a_device_table_other_than_the_mount_are_all_unreadable(
) {
    let (_pool, pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "out-of-the-lock-reads-on-another-device-table",
        );
    let reads = unit_reads_of_the_region_scan(
        region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH),
        &current_of(pending.session()).allocation_records,
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    assert!(!reads.reads().is_empty(), "前提：那一段里有要读的单元");
    let read_on_the_device_table_of_the_mount =
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads.clone());
    assert!(
        reads
            .reads()
            .iter()
            .any(|read| read_on_the_device_table_of_the_mount
                .unit_bytes(read.device, read.slot, read.unit_bytes)
                .is_some()),
        "前提：挂载核过的那一份盘表读得到"
    );
    devices.swap(0, 1);
    let read_on_the_swapped_device_table =
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads.clone());
    devices.swap(0, 1);
    assert!(
        reads
            .reads()
            .iter()
            .all(|read| read_on_the_swapped_device_table
                .unit_bytes(read.device, read.slot, read.unit_bytes)
                .is_none()),
        "换了次序的盘表：一个盘都不读、每个单元都当读不到"
    );
}

/// 签发那一支回锁重判资格的「钉住与隔离的槽」那一项（D26（后台整理与放置回收） 已定项 13 第 5 条）：锁外期间选中那一段里第一个文件的一个数据单元
/// 第二块盘上那一份被改坏、再覆盖写第一个文件——释放之前读盘核没过的那一个两块盘的记录留在已分配、带隔离记号（R 里还有未释放的记录），
/// 其余放掉；回锁重取隔离集之后那个槽在常驻隔离集里，在现行那一版上重判资格没资格：这一次不签、停在 `ChosenSegmentNoLongerEligibleAtTheRelock`，
/// 一个写都不发，观测记一次没资格换段。
#[test]
fn an_issue_at_the_relock_on_a_segment_with_a_slot_quarantined_in_the_window_is_given_up() {
    let mut pool = build_pool("relock-issue-on-a-segment-with-a-slot-quarantined-in-the-window");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, region, ticket, version_of_the_ticket) =
        ticket_issuing_on_the_fullest_segment_of_an_earlier_mount(&mut pool, &caller);
    let data_slot = version_of_the_ticket
        .data_pointers
        .iter()
        .map(|pointer| pointer.locations[0].slot)
        .find(|slot| region.contains_slot(*slot))
        .expect("前提：第一个文件有数据单元在选中那一段里");
    corrupt_the_copy_on(&mut devices, DeviceIdentity(1), data_slot);
    make_the_foreground_change(
        ForegroundChangeInTheWindow::OverwriteOfTheFirstFile,
        &mut pending,
        &mut devices,
        &caller,
    );
    let records_of_the_data_slot: Vec<_> = current_of(pending.session())
        .allocation_records
        .iter()
        .filter(|record| record.slot == data_slot)
        .copied()
        .collect();
    assert!(
        records_of_the_data_slot.len() == 2
            && records_of_the_data_slot
                .iter()
                .all(|record| !record.is_released
                    && record.quarantine_mark
                        == AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck),
        "前提：两块盘的记录留在已分配、带隔离记号：{records_of_the_data_slot:?}"
    );
    assert!(
        unreleased_records_starting_in(current_of(pending.session()), region) > 0,
        "前提：R 里还有未释放的记录（只有这一项不让这一段没资格）"
    );
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock,
        "R 里有隔离的槽：在现行那一版上没资格，这一次不签"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
    assert!(
        pending
            .session()
            .compaction
            .quarantined_slots()
            .contains(&data_slot),
        "前提：回锁时重取了隔离集，那个槽进了会话的常驻隔离集"
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .observations
            .issues_given_up_for_a_region_no_longer_eligible_at_the_relock,
        1,
        "观测记一次没资格换段"
    );
}

/// 签发前判那一刻 `region` 外的空洞数（D26（后台整理与放置回收） 已定项 1 第 2 条的左边，同回锁签发那一判的读法）：段里有没有用户数据按
/// `allocation_records` 那一份记录判；码 1 数据单元落着的槽与每个槽此刻能不能当目的地取会话此刻的（现行那一版的映射、此刻的分配器）。
fn holes_outside_the_region_judged_on_the_records_of(
    pending: &MountedSessionWithPendingChanges,
    allocation_records: &[AllocationRecord],
    region: CompactionRegion,
) -> u64 {
    let session = pending.session();
    let mut slots_holding_user_data: BTreeSet<SlotNumber> = BTreeSet::new();
    for entry in central_mapping_as_the_compaction_reads_it(current_of(session)) {
        if entry.key.first() == Some(&UNIT_CLASS_DATA) {
            let slot = entry.locations[0].slot;
            slots_holding_user_data.insert(slot);
            slots_holding_user_data.insert(SlotNumber(slot.0 + 1));
        }
    }
    let device_map = session.allocator.devices.first().expect("池里有盘");
    holes_outside_the_region_in_segments_holding_user_data(
        allocation_records,
        &slots_holding_user_data,
        unit_area_of_the_first_device(session),
        region,
        |slot| device_map.is_free(slot),
    )
}

/// 锁外期间一个新 inode 写一个数据单元、一个接一个写，至多写几个（取输入之前一次建好这么多 inode）。
const INODES_WRITTEN_ONE_DATA_UNIT_EACH_IN_THE_WINDOW_AT_MOST: u64 = 96;

/// 签发那一支回锁时签发前判的空洞数取现行那一版的记录（D26（后台整理与放置回收） 已定项 13 第 5 条「各个量取现行那一版的」）：另一次挂载里
/// 顺序写第一个文件、重挂之后强制选数据单元最多的那一段，取输入之前一次建好一批 inode；锁外期间一个 inode 写一个数据单元（用户数据取起点最小的空槽），
/// 写到票里那一版的记录口径下 R 外装用户数据的段里的空洞少于 R 里的未释放记录数、而现行那一版的记录口径下不少于它（新写进的那几段
/// 只在现行那一版的记录里装着用户数据）为止；回锁照现行那一版的空洞数判，照常签。
#[test]
fn an_issue_at_the_relock_counts_the_holes_of_the_segments_holding_user_data_in_the_current_version(
) {
    let mut pool =
        build_pool("relock-issue-counts-the-holes-on-the-records-of-the-current-version");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, region) =
        session_after_a_sequential_write_in_an_earlier_mount(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region),
        1,
    );
    pending
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: INODES_WRITTEN_ONE_DATA_UNIT_EACH_IN_THE_WINDOW_AT_MOST,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            caller.clock_reading(),
        )
        .expect("取输入之前建一批 inode");
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let version_of_the_ticket = current_of(pending.session()).clone();
    let content_of_one_data_unit = vec![0xC3u8; data_unit_payload_capacity()];
    let mut inodes_written = 0_u64;
    loop {
        let records_in_the_region_now = u64::try_from(unreleased_records_starting_in(
            current_of(pending.session()),
            region,
        ))
        .expect("一段 64 槽");
        let holes_on_the_records_of_the_ticket = holes_outside_the_region_judged_on_the_records_of(
            &pending,
            &version_of_the_ticket.allocation_records,
            region,
        );
        let holes_on_the_current_records = holes_outside_the_region_judged_on_the_records_of(
            &pending,
            &current_of(pending.session()).allocation_records,
            region,
        );
        if holes_on_the_records_of_the_ticket < records_in_the_region_now
            && holes_on_the_current_records >= records_in_the_region_now
        {
            break;
        }
        assert!(
            inodes_written < INODES_WRITTEN_ONE_DATA_UNIT_EACH_IN_THE_WINDOW_AT_MOST,
            "前提：写满 {inodes_written} 个 inode 之前，票里那一版记录口径下的空洞（{holes_on_the_records_of_the_ticket}）降到 R 里的未释放记录数（{records_in_the_region_now}）以下、现行那一版记录口径下的空洞（{holes_on_the_current_records}）不少于它"
        );
        pending
            .publish_user_change(
                &mut devices,
                UserChange::SequentialWriteOfAnInode {
                    inode: InodeNumber(FIRST_INODE_NUMBER + 1 + inodes_written),
                    content: &content_of_one_data_unit,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                caller.clock_reading(),
            )
            .expect("锁外期间给一个新 inode 写一个数据单元");
        inodes_written += 1;
    }
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        run.versions_published.len(),
        1,
        "现行那一版的空洞够装下 R 里要搬的：回锁签发了，{:?}",
        run.outcome
    );
    let (_, issued) =
        live_compaction_intent_of_the_entries(&run.versions_published[0].logged_operations)
            .expect("签发那一版里有活意图");
    assert_eq!(issued.region, region);
}

/// 签发那一支回锁重判资格的「这次挂载开过的段」那一项（D26（后台整理与放置回收） 已定项 13 第 5 条）：锁外期间前台建了一个 inode（现行那一版变了，
/// 选中那一段的记录没变），再经只供测试的开关（`PoolAllocator::note_a_cluster_segment_opened_in_this_mount_for_tests`）把选中那一段记成这次挂载开过的
/// 聚簇段（产品路径上要锁外期间那一段先腾空、再被提交内生块开成新段才走得到）；取输入时拷进票里的那一份聚簇段集合里没有它，回锁在现行那一版与此刻的分配器上
/// 重判资格，这次挂载开过的段没资格：这一次不签、停在 `ChosenSegmentNoLongerEligibleAtTheRelock`，一个写都不发，观测记一次没资格换段。
#[test]
fn an_issue_at_the_relock_on_a_segment_opened_by_this_mount_in_the_window_is_given_up() {
    let mut pool = build_pool("relock-issue-on-a-segment-opened-by-this-mount-in-the-window");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, region, ticket, version_of_the_ticket) =
        ticket_issuing_on_the_fullest_segment_of_an_earlier_mount(&mut pool, &caller);
    create_one_inode(&mut pending, &mut devices, &caller);
    pending
        .session_for_tests_only()
        .allocator
        .note_a_cluster_segment_opened_in_this_mount_for_tests(region);
    assert!(
        pending
            .session()
            .allocator
            .cluster_segments()
            .contains(&region.start_slot()),
        "前提：选中那一段记成了这次挂载开过的聚簇段"
    );
    assert!(
        current_of(pending.session()).root.checkpoint_txg
            > version_of_the_ticket.root.checkpoint_txg,
        "前提：锁外期间现行那一版变了（回锁才在现行那一版上重判资格）"
    );
    assert!(
        unreleased_records_starting_in(current_of(pending.session()), region) > 0
            && unreleased_records_starting_in(current_of(pending.session()), region)
                == unreleased_records_starting_in(&version_of_the_ticket, region),
        "前提：选中那一段的记录没变、R 里还有未释放的记录（只有这一项不让这一段没资格）"
    );
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock,
        "这次挂载开过的段在现行那一版上没资格：这一次不签"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .observations
            .issues_given_up_for_a_region_no_longer_eligible_at_the_relock,
        1,
        "观测记一次没资格换段"
    );
}

/// 锁外那一段算（同 [`computed_reading_in_the_session`] 回会话里读），只是选中那一段要读的单元里只有头一个读得出、其余当读盘读不到
/// （瞬时读错：那几条记录判成读不出头、不当可搬）：锁外扫出的能搬的单元只有一个。
fn computed_with_only_the_first_unit_of_the_region_readable(
    ticket: BackgroundCompactionBatchTicket,
    pending: &MountedSessionWithPendingChanges,
    devices: &Devices,
) -> BackgroundCompactionBatchComputed {
    ticket.compute_out_of_the_lock(&mut |reads| {
        let readable_unit = reads.reads().first().map(|read| (read.device, read.slot));
        let read_in_the_session =
            pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads.clone());
        reads.read_with(|device, slot, unit_bytes| {
            if readable_unit == Some((device, slot)) {
                read_in_the_session.unit_bytes(device, slot, unit_bytes)
            } else {
                None
            }
        })
    })
}

/// 签发那一支回锁时 R 里的记录变了，签发前判的要搬数取现行那一版 R 里未释放的记录数（D26（后台整理与放置回收） 已定项 13 第 5 条「各个量取现行那一版的」，
/// 上界：只多拒、不少拒），不取锁外扫出的能搬数：另一次挂载里顺序写第一个文件、重挂之后强制选数据单元最多的那一段，取输入之前一次建好一批 inode；
/// 锁外那一圈读选中那一段时只有头一个单元读得出（锁外扫出的能搬数是 1）；锁外期间按偏移写第一个文件在那一段里的一个数据单元（旧落点放掉，R 的记录变了），
/// 再一个 inode 写一个数据单元、一个接一个写，写到现行那一版记录口径下 R 外装用户数据的段里的空洞少于现行那一版 R 里的未释放记录数为止；
/// 回锁照现行那一版的未释放记录数判，不签、停在 `HolesOutsideTheRegionFewerThanItsUnitsToMove`，要搬数是现行那一版 R 里的未释放记录数，一个写都不发。
#[test]
fn an_issue_at_the_relock_after_the_records_of_the_region_changed_in_the_window_takes_the_unreleased_records_of_the_current_version_as_the_units_to_move(
) {
    let mut pool =
        build_pool("relock-issue-takes-the-records-of-the-current-version-as-the-units-to-move");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, region) =
        session_after_a_sequential_write_in_an_earlier_mount(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region),
        1,
    );
    pending
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: INODES_WRITTEN_ONE_DATA_UNIT_EACH_IN_THE_WINDOW_AT_MOST,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            caller.clock_reading(),
        )
        .expect("取输入之前建一批 inode");
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let version_of_the_ticket = current_of(pending.session()).clone();
    let computed =
        computed_with_only_the_first_unit_of_the_region_readable(*ticket, &pending, &devices);
    let data_unit_index_in_the_region = version_of_the_ticket
        .data_pointers
        .iter()
        .position(|pointer| region.contains_slot(pointer.locations[0].slot))
        .expect("前提：第一个文件有数据单元在选中那一段里");
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let one_byte = [0x3Cu8];
    pending
        .publish_user_change(
            &mut devices,
            UserChange::WriteAtOffset {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                offset: FileOffsetInBytes(
                    u64::try_from(data_unit_index_in_the_region).expect("单元下标") * payload,
                ),
                bytes: PartialWriteBytes::new(&one_byte).expect("一个字节不是空的"),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            caller.clock_reading(),
        )
        .expect("锁外期间按偏移写第一个文件在选中那一段里的一个数据单元");
    let content_of_one_data_unit = vec![0x96u8; data_unit_payload_capacity()];
    let mut inodes_written = 0_u64;
    let (records_in_the_region_now, holes_on_the_current_records) = loop {
        let records_in_the_region_now = u64::try_from(unreleased_records_starting_in(
            current_of(pending.session()),
            region,
        ))
        .expect("一段 64 槽");
        let holes_on_the_current_records = holes_outside_the_region_judged_on_the_records_of(
            &pending,
            &current_of(pending.session()).allocation_records,
            region,
        );
        if holes_on_the_current_records < records_in_the_region_now {
            break (records_in_the_region_now, holes_on_the_current_records);
        }
        assert!(
            inodes_written < INODES_WRITTEN_ONE_DATA_UNIT_EACH_IN_THE_WINDOW_AT_MOST,
            "前提：写满 {inodes_written} 个 inode 之前，现行那一版记录口径下的空洞（{holes_on_the_current_records}）降到 R 里的未释放记录数（{records_in_the_region_now}）以下"
        );
        pending
            .publish_user_change(
                &mut devices,
                UserChange::SequentialWriteOfAnInode {
                    inode: InodeNumber(FIRST_INODE_NUMBER + 1 + inodes_written),
                    content: &content_of_one_data_unit,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                caller.clock_reading(),
            )
            .expect("锁外期间给一个新 inode 写一个数据单元");
        inodes_written += 1;
    };
    let slots_of_the_records_in_the_region = |version: &TransactionOutput| -> BTreeSet<SlotNumber> {
        version
            .allocation_records
            .iter()
            .filter(|record| {
                record.device == region.device()
                    && !record.is_released
                    && region.contains_slot(record.slot)
            })
            .map(|record| record.slot)
            .collect()
    };
    assert_ne!(
        slots_of_the_records_in_the_region(current_of(pending.session())),
        slots_of_the_records_in_the_region(&version_of_the_ticket),
        "前提：锁外期间 R 里的未释放记录变了（按偏移写放掉了那一个数据单元的旧落点）"
    );
    assert!(
        records_in_the_region_now >= 2,
        "前提：现行那一版 R 里的未释放记录不止一条，与锁外扫出的能搬数（1）分得开"
    );
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    let stop = stop_of(&run);
    let CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove {
        holes,
        units_to_move,
    } = stop
    else {
        panic!(
            "现行那一版 R 里的未释放记录数（{records_in_the_region_now}）多于空洞（{holes_on_the_current_records}）：回锁不签，停在空洞不够上，却停在 {stop:?}"
        )
    };
    assert_eq!(
        units_to_move, records_in_the_region_now,
        "R 里的记录变了：要搬数取现行那一版 R 里的未释放记录数，不取锁外扫出的能搬数（空洞 {holes}）"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
}

/// 一版里钉住的单元（映射树各节点、树表、实例表链：根记录直接指着它们）罩住的每一个槽，与会话判选段资格读的那一份（`pinned_slots_of_the_version`）
/// 同一个口径：只读这一版的单元表。
fn slots_of_the_units_pinned_in(version: &TransactionOutput) -> BTreeSet<SlotNumber> {
    version
        .units
        .iter()
        .filter(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::MappingTreeNodeBelowTheRoot(_)
                    | TransactionUnit::MappingTree
                    | TransactionUnit::TreeTable
                    | TransactionUnit::InstanceTable
                    | TransactionUnit::InstanceTablePageAfterTheFirst(_)
            )
        })
        .flat_map(|unit| (unit.slot.0..unit.slot.0 + unit.identity.span_slots()).map(SlotNumber))
        .collect()
}

/// 另一次挂载里顺序写第一个文件、重挂之后，第一块盘上起点在段里、未释放的两槽记录（数据单元）至少一条，现行那一版里没有钉住的槽、
/// 不是这次挂载开过的段，在这几样里取每块盘都空的槽最多的那一段（一样多取段起点小的）：锁外期间提交内生块 bump 进这一段时，
/// 一次发布的钉住单元装得下。交回那一段与它里面每块盘都空的槽数。
fn segment_of_an_earlier_mount_holding_data_units_with_the_most_slots_free_on_every_device(
    pending: &MountedSessionWithPendingChanges,
) -> (CompactionRegion, u64) {
    let session = pending.session();
    let version = current_of(session);
    let pinned_slots = slots_of_the_units_pinned_in(version);
    let segments_holding_data_units: BTreeSet<u64> = version
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
        })
        .map(|record| record.slot.0 / 64 * 64)
        .collect();
    segments_holding_data_units
        .into_iter()
        .map(region_starting_at)
        .filter(|region| {
            pinned_slots
                .range(region.start_slot()..region.end_slot())
                .next()
                .is_none()
                && !session
                    .allocator
                    .cluster_segments()
                    .contains(&region.start_slot())
        })
        .map(|region| {
            let slots_free_on_every_device = (region.start_slot().0..region.end_slot().0)
                .filter(|slot| {
                    session
                        .allocator
                        .devices
                        .iter()
                        .all(|device_map| device_map.is_free(SlotNumber(*slot)))
                })
                .count();
            (
                region,
                u64::try_from(slots_free_on_every_device).expect("一段 64 槽"),
            )
        })
        .max_by_key(|(region, slots_free_on_every_device)| {
            (
                *slots_free_on_every_device,
                std::cmp::Reverse(region.start_slot()),
            )
        })
        .expect("前提：另一次挂载里写的数据单元至少落在一段没有钉住的槽、不是这次挂载开过的段里")
}

/// 签发那一支回锁重判资格的「钉住的槽」那一项（D26（后台整理与放置回收） 已定项 13 第 5 条）：另一次挂载里顺序写第一个文件、重挂之后强制选
/// 装着数据单元、每块盘都空的槽最多的那一段，取输入那一刻那一段里没有钉住的槽；锁外期间经只供测试的开关
/// （`PoolAllocator::bump_commit_generated_units_into_a_segment_not_noted_as_opened_in_this_mount_for_tests`）把提交内生块 bump 进选中那一段、
/// 不把它记成这次挂载开过的段，再建一个 inode：那次发布的钉住单元（映射树根这一类）落进选中那一段（产品路径上提交内生块开新段时那一段同时记成
/// 这次挂载开过的段，只有每块盘都没有全空段、回落时才走得到）。回锁在现行那一版上重判资格，这一段里有钉住的槽、别的几项都有资格：
/// 这一次不签、停在 `ChosenSegmentNoLongerEligibleAtTheRelock`，一个写都不发，观测记一次没资格换段。
#[test]
fn an_issue_at_the_relock_on_a_segment_where_a_publish_in_the_window_placed_a_pinned_unit_is_given_up(
) {
    let mut pool =
        build_pool("relock-issue-on-a-segment-holding-a-pinned-unit-placed-in-the-window");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, _fullest_segment) =
        session_after_a_sequential_write_in_an_earlier_mount(&mut pool);
    let (region, slots_free_on_every_device) =
        segment_of_an_earlier_mount_holding_data_units_with_the_most_slots_free_on_every_device(
            &pending,
        );
    assert!(
        slots_free_on_every_device > 0,
        "前提：选中那一段里有每块盘都空的槽（提交内生块 bump 得进去）"
    );
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region),
        1,
    );
    create_one_inode(&mut pending, &mut devices, &caller);
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let version_of_the_ticket = current_of(pending.session()).clone();
    assert!(
        slots_of_the_units_pinned_in(&version_of_the_ticket)
            .range(region.start_slot()..region.end_slot())
            .next()
            .is_none(),
        "前提：取输入那一刻选中那一段里没有钉住的槽"
    );
    pending
        .session_for_tests_only()
        .allocator
        .bump_commit_generated_units_into_a_segment_not_noted_as_opened_in_this_mount_for_tests(
            region,
        );
    create_one_inode(&mut pending, &mut devices, &caller);
    let pinned_slots_in_the_region_now: Vec<SlotNumber> =
        slots_of_the_units_pinned_in(current_of(pending.session()))
            .range(region.start_slot()..region.end_slot())
            .copied()
            .collect();
    assert!(
        !pinned_slots_in_the_region_now.is_empty(),
        "前提：锁外期间那次发布的钉住单元落进了选中那一段（每块盘都空的槽 {slots_free_on_every_device} 个）"
    );
    assert!(
        !pending
            .session()
            .allocator
            .cluster_segments()
            .contains(&region.start_slot()),
        "前提：选中那一段不是这次挂载开过的段"
    );
    assert!(
        current_of(pending.session()).root.checkpoint_txg
            > version_of_the_ticket.root.checkpoint_txg,
        "前提：锁外期间现行那一版变了（回锁才在现行那一版上重判资格）"
    );
    assert!(
        unreleased_records_starting_in(current_of(pending.session()), region) > 0,
        "前提：R 里还有未释放的记录"
    );
    assert!(
        pending
            .session()
            .compaction
            .quarantined_slots()
            .range(region.start_slot()..region.end_slot())
            .next()
            .is_none()
            && !pending
                .session()
                .compaction
                .segments_refused_by_the_region_scan
                .contains(region.start_slot()),
        "前提：选中那一段里没有隔离的槽、没被扫描拒过（只有钉住那一项不让这一段有资格）"
    );
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock,
        "锁外期间那次发布的钉住单元落进了选中那一段（{pinned_slots_in_the_region_now:?}）：在现行那一版上没资格，这一次不签"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .observations
            .issues_given_up_for_a_region_no_longer_eligible_at_the_relock,
        1,
        "观测记一次没资格换段"
    );
}
