//! harness 测试文件 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
    DeletionQueueContinuationOutcome, FloorRaiseStop, MountError, MountSpaceAdmission, Mounted,
    ShadowLedger, Unmounted,
};
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::{choose_system_configuration, recover, JournalPolicy};
use singlefs_core::root_record::RootRecord;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion, PoolWriter, PublishError,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_core::write_accounting::WrittenStructureKind;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 挂载续做那两条的窄盘：不为环窗口补足（准入关着造盘面，`start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off`）、
/// 单元区 292 槽；第一个文件之后覆盖写 [`OVERWRITES_BEFORE_THE_CONTINUATION`] 次，再顺序写把它长到 [`DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION`] 个数据单元。
/// 第十位实现员副本上的草稿探针（单元区 192 / 256 / 320 / 384 槽 × 覆盖写 0–12 次 × 长到 1–14 个单元 × 崩在意图之后或第一批之后，逐格现跑）定的是
/// 256 槽、覆盖写 2 次、14 个单元；第 50 批退役树进准入之后挂载那一判每块盘多扣约 42 槽（标签 6 退役 3 槽、ckpt_cost 退役 3 块进保留池、切换预留跟着涨），
/// 那一格崩在意图之后变成「取号之前就不够、先推一串」。第 50 批补做重扫（单元区 256–304 槽每 4 槽 × 覆盖写 0–4 次 × 长到 10–24 个单元 ×
/// 崩在意图之后、第一批之后、第二批之后，逐格现跑）：292 槽、覆盖写 1 次、16 个单元那一格崩在意图之后是「取号之前那一判放行、续做里一批被落得下那一判拒、
/// 推一串两次空发布之后走完」，崩在第一批之后是「取号之前放行、续做推不出空间、挂载照样做成可写」，不先覆盖写、崩在第二批之后是「续做走到空、一串都没推」——
/// 三种场景同在一格的，256–304 槽里只有这一格。
pub const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION: u64 = 292;

pub const OVERWRITES_BEFORE_THE_CONTINUATION: usize = 1;

pub const DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION: usize = 16;

/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
pub fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 窄盘上（见 [`UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION`]）造好文件，在这个进程里直接发 unlink 的意图那次与之后 `batches` 批
/// （批大小 1，不经会话、不走空），然后进程退出（会话丢掉）。交回池与退出那一刻现行那一版的根。
pub fn pool_crashed_between_deletion_batches_on_narrow_devices(
    batches: usize,
) -> (PoolUnderTest<SparseBlockDevice>, RootRecord) {
    pool_crashed_between_deletion_batches(NarrowPoolCrashedBetweenDeletionBatches {
        unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
        overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
        data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
        batches_published_after_the_intent: batches,
    })
}

/// 不补环窗口、准入关着造的窄盘上崩在两批删除之间的那一刻（草稿探针的网格就按这四样排）：单元区几槽、unlink 之前先覆盖写几次、
/// 第一个文件顺序写长到几个数据单元、意图之后又发了几批（批大小 1）。
#[derive(Clone, Copy, Debug)]
pub struct NarrowPoolCrashedBetweenDeletionBatches {
    pub unit_area_slots: u64,
    pub overwrites_before_the_unlink: usize,
    pub data_units_of_the_file: usize,
    pub batches_published_after_the_intent: usize,
}

/// 照 `shape` 造盘：准入关着的窄盘上第一个文件之后覆盖写、顺序写长大，在这个进程里直接发 unlink 的意图那次与之后那几批
/// （批大小 1，不经会话、不走空），然后进程退出（会话丢掉）。交回池与退出那一刻现行那一版的根。
pub fn pool_crashed_between_deletion_batches(
    shape: NarrowPoolCrashedBetweenDeletionBatches,
) -> (PoolUnderTest<SparseBlockDevice>, RootRecord) {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            shape.unit_area_slots,
        );
    for overwrite_index in 1..=shape.overwrites_before_the_unlink {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
    }
    pool.sequential_write((shape.data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
        .unwrap_or_else(|refusal| panic!("顺序写把文件长到几个单元：{refusal:?}"));
    let PoolUnderTest {
        session,
        devices,
        parameters,
        write_time_seconds,
        ..
    } = &mut pool;
    let session = session.as_mut().expect("这一刻有可写会话");
    let mut current = session
        .current
        .version_of_trees()
        .expect("第一个文件之后有树")
        .clone();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    current = publish_unlink_intent(
        &mut writer,
        &mut session.allocator,
        &current,
        InodeNumber(FIRST_INODE_NUMBER),
        *write_time_seconds,
        session.instance,
    )
    .expect("意图那次");
    for batch_index in 0..shape.batches_published_after_the_intent {
        current = publish_unlink_step(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            deletion_switches_with_batches_of_one().batch_size,
            session.instance,
        )
        .unwrap_or_else(|refusal| panic!("第 {batch_index} 批：{refusal:?}"));
    }
    let root_at_the_crash = current.root;
    pool.session = None;
    (pool, root_at_the_crash)
}

/// 崩了之后重开：空间准入判着、影子账开、删除批大小 1。
pub fn mount_after_the_crash_with_batches_of_one(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
) -> Result<Mounted, MountError> {
    mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
}

/// 挂载续做里删除的一批被落得下那一判拒、推一串抬 F、回收之后接着走（用户 2026-10-03 定案「挂载续做被空间拒：同样推抬 F 再试」，
/// 与会话那一处同一个做法）：窄盘上 16 个单元的文件发完意图就崩（队列一条、单元都在），重开、准入判着、删除批大小 1。
/// 取号之前那一判放行；写行与暖机之后续做，走到一半一批被落得下那一判拒（`SpaceAdmissionRefused`；删除类发布不判式子那一判），
/// 推一串两次空发布的抬 F（插在续做的两步之间）、回收之后接着走，16 步走完：挂载交回的现行那一版有树而文件集合为空、队列空，F 抬上去了，
/// 池级 checker 全绿。
/// 判别力：续做里不推（这一条定案之前的做法）时挂载交回 `MountError::Publish { cause: SpaceAdmissionRefused }`。
pub fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, root_at_the_crash) = pool_crashed_between_deletion_batches_on_narrow_devices(0);
    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("续做推过抬 F 之后走完：{error:?}"));
    let output = &mounted.output;
    assert!(
        matches!(
            output.space_admission,
            MountSpaceAdmission::AdmittedBeforeAcquisition
        ),
        "取号之前那一判放行：{:?}",
        output.space_admission
    );
    assert_eq!(
        (
            output
                .refusals_that_pushed_the_continuation_floor_raises
                .iter()
                .map(kind_of_the_refusal_that_pushed)
                .collect::<Vec<_>>(),
            output
                .floor_raises_pushed_by_the_continuation
                .iter()
                .map(|raised| raised.publishes.len())
                .collect::<Vec<_>>(),
            output.deletion_queue_continuation.len(),
        ),
        (vec!["SpaceAdmissionRefused"], vec![2], 16),
        "续做里推过一串（两次空发布），推它的是一批的落得下那一判；续做走了 16 步"
    );
    let first_raise_txg = output.floor_raises_pushed_by_the_continuation[0].publishes[0]
        .root
        .checkpoint_txg;
    let continuation_txgs: Vec<CheckpointTxg> = output
        .deletion_queue_continuation
        .iter()
        .map(|step| step.root.checkpoint_txg)
        .collect();
    assert!(
        continuation_txgs.first().expect("续做至少一步") < &first_raise_txg
            && &first_raise_txg < continuation_txgs.last().expect("续做至少一步"),
        "推的那一串插在续做的两步之间：抬 F 第一次空发布 {first_raise_txg:?}，续做 {continuation_txgs:?}"
    );
    let PoolVersion::WithTreesAndNoFile(emptied) = &mounted.current else {
        panic!("续做走完：现行那一版有树而文件集合为空");
    };
    assert_eq!(
        (emptied.deleted_inodes.len(), emptied.root.checkpoint_txg),
        (0, *continuation_txgs.last().expect("续做至少一步")),
        "队列走空，现行那一版就是续做最后一步"
    );
    assert!(
        emptied.root.rollback_floor > root_at_the_crash.rollback_floor,
        "F 从 {:?} 抬上去了：{:?}",
        root_at_the_crash.rollback_floor,
        emptied.root.rollback_floor
    );
    drop(mounted);
    images.hand_out(
        "续做推过抬 F、走完之后",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 推满仍放不下（用户 2026-10-03 定「照 D16 做成可写」：挂载照样做成可写、队列留着）：同一份窄盘造法，发完意图与第一批才崩，重开、准入判着、
/// 删除批大小 1。续做里一批被落得下那一判拒，为它推的那一串推不出空间（一串都没推成，F 没动）：挂载照样交回可写，
/// 续做停在 `StillShortAfterTheFloorRaises`、最后那一次被拒是落得下那一判，现行那一版的队列还留着第一个文件；
/// 挂着时一次用户改动先走空、走不动报空间不够（一版都没发成、盘不动）。挂载写下的 17 次发布（写行、暖机与续做发成的几步）留在盘上：
/// 只读挂载走的是挂载最后落盘的那一版，第一个文件仍在删除中（打开它答「没有文件」）。
/// 正常卸载照常做成（用户 2026-10-03 定甲 GA，`m3-unlink-code-r3-forks`）：槽被续做吃光了，卸载那一串的预演算上新 F 回收出来、当场放开的那一截
/// （释放代不高于每块盘上最新有效根的那几个槽），
/// 抬 F 到现行那一版、两次空发布；之后池级 checker 全绿，再挂可写、续做接着走到空（一步）。
/// 判别力：推满仍放不下照旧交回 `MountError::Publish`（这一条定案之前的做法）、或结局报成 `WalkedToEmpty`，挂载那一步就红；
/// 卸载回收的槽照旧整截扣到生效（`ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect`）时卸载在预演里取不到落点，卸载那一步红。
pub fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, root_at_the_crash) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
    let DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { last_refusal, .. } =
        &mounted.output.deletion_queue_continuation_outcome
    else {
        panic!(
            "续做推满仍放不下：{:?}",
            mounted.output.deletion_queue_continuation_outcome
        );
    };
    assert_eq!(
        kind_of_the_refusal_that_pushed(last_refusal),
        "SpaceAdmissionRefused",
        "最后那一次被拒是续做里那一批的落得下那一判：{last_refusal:?}"
    );
    assert_eq!(
        mounted
            .current
            .version_of_trees()
            .expect("续做走到的那一版有树")
            .deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "队列留着第一个文件"
    );
    let image_after_the_mount = pool.image();
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content = vec![9_u8; 100];
    let change = session.publish_user_change(
        &mut pool.devices,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1,
        }),
    );
    let Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) = &change else {
        panic!("这次挂载的会话里一次用户改动先走空、走不动报空间不够：{change:?}");
    };
    assert_eq!(
        (
            kind_of_the_refusal_that_pushed(&no_space.last_refusal),
            no_space.versions_published.len(),
            no_space.floor_raises.len(),
            matches!(
                no_space.stop,
                FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
            ),
        ),
        ("SpaceAdmissionRefused", 0, 0, true),
        "走空的那一批照样被落得下那一判拒、为它推的那一串自己也落不下：一版都没发成，报空间不够"
    );
    assert!(
        pool.image() == image_after_the_mount,
        "那一次用户改动在任何写之前被拒：两块盘逐字节停在挂载之后"
    );
    let read_only = mount_read_only(&image_after_the_mount).expect("只读挂载");
    assert_eq!(
        (
            read_only.effective_root.checkpoint_txg.0 - root_at_the_crash.checkpoint_txg.0,
            read_only.effective_root.rollback_floor.0 - root_at_the_crash.rollback_floor.0,
        ),
        (17, 0),
        "挂载写下的 17 次发布（写行、暖机与续做发成的几步）留在盘上；为那一批推的抬 F 一串都没推成，F 没动"
    );
    assert!(
        matches!(
            read_only
                .mounted
                .open_file(&image_after_the_mount, InodeNumber(FIRST_INODE_NUMBER)),
            Err(OpenFileFailure::NoSuchInode { .. })
        ),
        "第一个文件仍在删除中：打开它答「没有文件」"
    );
    // 正常卸载照常做成（D16（发布语义） 已定项 1「正常卸载」，甲 GA）：这块窄盘的槽被续做吃光了，卸载那一串回收出来的槽里当场放开的那一截不扣到生效，
    // 预演与真发都落在它们上面。
    let current_txg = session.current.root().checkpoint_txg;
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    );
    let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
        panic!(
            "卸载那一串的预演算上新 F 回收出来的槽，做成：{:?}",
            unmounted.as_ref().err()
        );
    };
    assert_eq!(
        (raised.rollback_floor, raised.publishes.len()),
        (current_txg, 2),
        "F 抬到现行那一版的 txg，两次带卸载记号的空发布"
    );
    drop(session);
    let image_after_the_unmount = pool.image();
    images.hand_out(
        "卸载之后",
        || image_after_the_unmount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let remounted = mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("卸载之后再挂可写：{error:?}"));
    assert!(
        matches!(
            remounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "再挂之后续做接着走：{:?}",
        remounted.output.deletion_queue_continuation_outcome
    );
    assert_eq!(
        (
            remounted.output.deletion_queue_continuation.len(),
            member_name_of(&remounted.current)
        ),
        (1, "WithTreesAndNoFile"),
        "再挂之后续做走了一步、走到有树而文件集合为空的一版"
    );
    drop(remounted);
    images.hand_out(
        "再挂续做之后",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 现行那一版是 `PoolVersion` 的哪一个成员。
pub fn member_name_of(version: &PoolVersion) -> &'static str {
    match version {
        PoolVersion::WithoutFile(_) => "WithoutFile",
        PoolVersion::WithFile(_) => "WithFile",
        PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
    }
}

/// 续做走到空、F 一次都没抬、可删的单元都还在 defer 里的盘面上卸载（攻方 `m3-unmount-after-short-continuation-r1` 表 A 那 9 格的形：
/// 卸载被拒不绑队列）：单元区 292 槽、不先覆盖写、16 个单元，崩在意图与两批之后。重开、准入判着、删除批大小 1：取号之前那一判放行、
/// 没推抬 F，续做 14 步走到空。挂载之后每块盘只剩 10 个没挡的空槽，卸载那一串两次空发布要 14 个——扣着回收出来的槽就在预演里取不到落点；
/// 甲 GA 之下预演算上新 F 回收出来、当场放开的那一截，卸载做成、F 抬到现行那一版，池级 checker 全绿，再挂可写做成。
/// 判别力：卸载回收的槽照旧扣到生效时卸载在预演里被拒，卸载那一步红。
pub fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, root_at_the_crash) =
        pool_crashed_between_deletion_batches(NarrowPoolCrashedBetweenDeletionBatches {
            unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
            overwrites_before_the_unlink: 0,
            data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
            batches_published_after_the_intent: 2,
        });
    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("续做走到空：{error:?}"));
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "续做走到空：{:?}",
        mounted.output.deletion_queue_continuation_outcome
    );
    assert_eq!(
        (
            mounted.output.space_admission.floor_raises().len(),
            mounted.output.floor_raises_pushed_by_the_continuation.len(),
            mounted.output.deletion_queue_continuation.len(),
            mounted.current.root().rollback_floor,
        ),
        (0, 0, 14, root_at_the_crash.rollback_floor),
        "准入与续做都没推抬 F、F 没动，续做 14 步走到空"
    );
    let free_slots_before_the_unmount: Vec<u64> = mounted
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.free_slots())
        .collect();
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let current_txg = session.current.root().checkpoint_txg;
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    );
    let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
        panic!(
            "卸载那一串的预演算上新 F 回收出来的槽，做成：{:?}",
            unmounted.as_ref().err()
        );
    };
    let slots_taken_by_the_unmount_sequence: Vec<SlotNumber> = raised
        .publishes
        .iter()
        .flat_map(|publish| {
            publish
                .units
                .iter()
                .filter(|unit| publish.rewritten.contains(&unit.identity))
                .map(|unit| unit.slot)
        })
        .collect();
    let reclaimed_slots: Vec<SlotNumber> = raised
        .reclaimed
        .iter()
        .map(|placement| placement.slot)
        .collect();
    assert_eq!(
        (
            free_slots_before_the_unmount,
            slots_taken_by_the_unmount_sequence.len(),
            raised.rollback_floor,
        ),
        (vec![10, 10], 14, current_txg),
        "卸载之前每块盘 10 个空槽，那一串要 14 个；F 抬到现行那一版"
    );
    assert!(
        slots_taken_by_the_unmount_sequence
            .iter()
            .any(|slot| reclaimed_slots.contains(slot)),
        "那一串有单元落在它自己回收出来的槽上：{slots_taken_by_the_unmount_sequence:?}"
    );
    drop(session);
    images.hand_out(
        "卸载之后",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("卸载之后再挂可写：{error:?}"));
}

/// 一块稀疏盘外面先包录制器（录下真落到盘上的每一步），再包可注错的那一层。
pub type FaultInjectingRecordedDevice =
    FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

/// 一份镜像交给两块新的 [`FaultInjectingRecordedDevice`]（`schedule` 为 `None` 时只数调用、不注错），连同它们共用的那条录制流。
pub fn fault_injecting_recorded_devices_on(
    image: &MemoryPool,
    schedule: Option<FaultSchedule>,
) -> (
    Vec<(DeviceIdentity, FaultInjectingRecordedDevice)>,
    SharedStream,
) {
    let geometry = HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry();
    let plan = match schedule {
        Some(schedule) => SharedFaultPlan::armed(geometry, schedule),
        None => SharedFaultPlan::unarmed(geometry),
    };
    let stream = SharedStream::new();
    let devices = plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                    plan.clone(),
                ),
            )
        })
        .collect();
    (devices, stream)
}

/// 这几块录制着、可注错的盘此刻的镜像。
pub fn image_of_the_fault_injecting_recorded_devices(
    devices: &[(DeviceIdentity, FaultInjectingRecordedDevice)],
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| {
                (
                    *identity,
                    device.wrapped_device().wrapped_device().image.clone(),
                )
            })
            .collect(),
        device_size_in_bytes,
    }
}

/// 整池第 `ordinal` 次写报块设备错（从 1 数）；`ThisWriteAndEveryLaterOne` 时之后每一次写都报错（掉电的那一形：盘停在这次写之前）。
#[derive(Clone, Copy, Debug)]
pub enum FailingWrites {
    OnlyThisWrite,
    ThisWriteAndEveryLaterOne,
}

pub fn write_failing_from(ordinal: u64, failing_writes: FailingWrites) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::AcrossThePool,
        occurrence: match failing_writes {
            FailingWrites::OnlyThisWrite => FaultOccurrence::TheNthMatchingCall(ordinal),
            FailingWrites::ThisWriteAndEveryLaterOne => {
                FaultOccurrence::EveryMatchingCallFromTheNthOnward(ordinal)
            }
        },
    }
}

/// 录制流里真落到盘上的写（屏障不算）与其中的根槽写（根槽是唯一走 FUA 的写，`transaction::CommitStep::WriteRootRecordForceUnitAccess`）。
pub fn landed_writes_and_root_slot_writes(stream: &SharedStream) -> (u64, u64) {
    let operations = stream.operations();
    let count_of = |kinds: &[RecordedOperationKind]| {
        u64::try_from(
            operations
                .iter()
                .filter(|operation| kinds.contains(&operation.kind))
                .count(),
        )
        .expect("步数装得进 u64")
    };
    (
        count_of(&[
            RecordedOperationKind::Write,
            RecordedOperationKind::WriteForceUnitAccess,
            RecordedOperationKind::WriteZeroes,
        ]),
        count_of(&[RecordedOperationKind::WriteForceUnitAccess]),
    )
}

/// 卸载那一串（甲 GA：回收出来的槽里释放代不高于每块盘上最新有效根的那一截当场放开，其余扣到生效）在先写系统配置之后、第一条带新 F 的根之前掉电
/// （攻方 `m3-unmount-after-short-continuation-r1` 丙 1 那一节自列没判的第一件）：续做推满仍放不下的那块窄盘
/// （[`a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`]
/// 的造法），先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布写几个单元、其中落在这一串回收出来的槽上的那几个各是什么释放代；
/// 再在另一份上从「先写系统配置那 2 个写 + 第一次空发布的单元写」之后的第一个写起每一个写都报错——单元已写进当场放开的槽，journal 记录与根都没落。
/// GA 的结局（现行那一版 txg 24，第一版区域归属 0 / 1 / 0：盘 0 上最新的根 24、盘 1 上 22，取小 22）：第一次空发布落在回收槽上的单元，
/// 那几个槽的释放代都不高于 22；回收出来的槽里释放代 23、24 的那几个（txg 22 那条根还引用着）扣到生效，第一次空发布一个都没落上去。
/// 改钉这一句的理由：原来钉的「有单元落在回收出来的槽上」是丙 1 的说法（回收的槽整截当场能发）；GA 下当场能发的只有释放代不高于每块盘上最新根的那一截
/// （判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 1 条，用户 2026-10-03 定甲 GA），钉的就改成落上去的那几个槽的释放代。
/// 掉电之后：池级 checker 全绿；只读挂载走的是卸载之前现行那一版（根上带旧 F），而系统配置里已是新 F；再挂可写做成、续做接着走到空，池级 checker 全绿。
/// 判别力：卸载回收的槽照旧整截扣到生效时拷贝上那一次卸载就在预演里被拒，第一步红。整截当场放开（丙 1）时这一形照样绿：这块盘面上那一串的单元
/// 本来就没落到释放代 23、24 的槽上（判决第二节 N1「X 第 1404 行那一份盘面的交集」坐实为空；本批证红时量过，`crates/mutations.tsv` 不留这一行），
/// 释放代那一项只钉 GA 的结局、不分辨丙 1；分辨丙 1 的是 `a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` 那两形。
pub fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, _) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
    let image_after_the_mount = pool.image();
    let (_, session) = MountedSession::of_the_mount(mounted);
    let current_root = *session.current.root();
    let (unit_writes_of_the_first_publish, publishes_of_the_sequence) = {
        let mut allocator = session.allocator.clone();
        let mut current = session.current.clone();
        let (mut devices, _) = fault_injecting_recorded_devices_on(&image_after_the_mount, None);
        let unmounted = unmount(
            &pool.parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            session.shadow_ledger,
        );
        let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
            panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
        };
        let first = &raised.publishes[0];
        assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device(
            &session.allocator,
            &current_root,
            raised,
        );
        let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
        (
            first.writes.total().write_calls
                - structure_writes(WrittenStructureKind::JournalRecord)
                - structure_writes(WrittenStructureKind::RootSlot)
                - structure_writes(WrittenStructureKind::SystemConfigurationSlot),
            raised.publishes.len(),
        )
    };
    assert_eq!(publishes_of_the_sequence, 2, "卸载那一串两次空发布");
    let system_configuration_writes_of_the_sequence = 2;
    let (mut devices, stream) = fault_injecting_recorded_devices_on(
        &image_after_the_mount,
        Some(write_failing_from(
            system_configuration_writes_of_the_sequence + unit_writes_of_the_first_publish + 1,
            FailingWrites::ThisWriteAndEveryLaterOne,
        )),
    );
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    let unmounted = unmount(
        &pool.parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        session.shadow_ledger,
    );
    assert!(
        matches!(
            unmounted,
            Err(MountError::RaiseFloorSequencePublishFailed(_))
        ),
        "掉电之后那一串第一次空发布写 journal 记录时报错：{:?}",
        unmounted.as_ref().err()
    );
    assert_eq!(
        landed_writes_and_root_slot_writes(&stream),
        (
            system_configuration_writes_of_the_sequence + unit_writes_of_the_first_publish,
            0
        ),
        "落下的只有先写系统配置那两个写与第一次空发布的单元写，一个根都没落"
    );
    let crashed = image_of_the_fault_injecting_recorded_devices(
        &devices,
        image_after_the_mount.device_size_in_bytes,
    );
    images.hand_out(
        "掉电那一刻的盘面上",
        || crashed.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let read_only = mount_read_only(&crashed).expect("只读挂载");
    assert_eq!(
        (
            read_only.effective_root.checkpoint_txg,
            read_only.effective_root.rollback_floor,
            choose_system_configuration(&crashed)
                .expect("系统配置读得出")
                .quantities
                .rollback_floor,
        ),
        (
            current_root.checkpoint_txg,
            current_root.rollback_floor,
            current_root.checkpoint_txg,
        ),
        "只读挂载走卸载之前现行那一版（根上带旧 F），系统配置里已是新 F（现行那一版的 txg）"
    );
    let mut remount_devices = plain_devices_on(&crashed);
    let remounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut remount_devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .unwrap_or_else(|error| panic!("掉电之后再挂可写：{error:?}"));
    assert!(
        matches!(
            remounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ) && member_name_of(&remounted.current) == "WithTreesAndNoFile",
        "再挂之后续做接着走到空：{:?}",
        remounted.output.deletion_queue_continuation_outcome
    );
    drop(remounted);
    let image_after_the_remount = MemoryPool {
        devices: remount_devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: crashed.device_size_in_bytes,
    };
    images.hand_out(
        "再挂续做之后",
        || image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 续做推满仍放不下的那块窄盘上，卸载之前现行那一版的 txg，与那时每块盘上最新的根的 txg 取小（第一版区域归属 0 / 1 / 0：txg 24 的根在盘 0，
/// 盘 1 上最新的是 txg 22；这块盘的根从写行起连号写下来、没有回退，没有被抛弃的根）。
pub const CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg =
    CheckpointTxg(24);

pub const NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg =
    CheckpointTxg(22);

/// 甲 GA 在续做推满仍放不下的那块窄盘上的结局（用户 2026-10-03 定，`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`）：卸载那一串
/// 第一次空发布落在这一串回收出来的槽上的单元至少一个（槽被续做吃光，卸载靠当场放开的那一截做成），那几个槽在卸载之前的分配记录里的释放代都不高于
/// 每块盘上最新的根的 txg 取小；回收出来的槽里另有释放代高于它的（那几条根还引用着，扣到生效），第一次空发布没落上去。
pub fn assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device(
    allocator_before_the_unmount: &singlefs_core::allocator::PoolAllocator,
    current_root: &RootRecord,
    raised: &singlefs_core::mount::UnmountRaisedTheFloor,
) {
    let release_generation_of = |slot: SlotNumber| {
        allocator_before_the_unmount
            .records()
            .iter()
            .find(|record| {
                record.device == DeviceIdentity(0) && record.slot == slot && record.is_released
            })
            .map(|record| record.generation)
            .expect("这一串回收的落点在卸载之前的分配记录里是已释放的")
    };
    let first = &raised.publishes[0];
    let release_generations_under_the_first_publish: Vec<CheckpointTxg> = first
        .units
        .iter()
        .filter(|unit| first.rewritten.contains(&unit.identity))
        .filter(|unit| {
            raised
                .reclaimed
                .iter()
                .any(|placement| placement.slot == unit.slot)
        })
        .map(|unit| release_generation_of(unit.slot))
        .collect();
    let reclaimed_released_after_the_newest_root_of_every_device: Vec<CheckpointTxg> = raised
        .reclaimed
        .iter()
        .map(|placement| release_generation_of(placement.slot))
        .filter(|generation| {
            *generation > NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION
        })
        .collect();
    assert_eq!(
        current_root.checkpoint_txg, CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION,
        "卸载之前现行那一版 txg 24（盘 0 上），盘 1 上最新的根 txg 22"
    );
    assert!(
        !release_generations_under_the_first_publish.is_empty()
            && release_generations_under_the_first_publish.iter().all(|generation| {
                *generation <= NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION
            }),
        "第一次空发布落在回收槽上的单元有，那几个槽的释放代都不高于 22：{release_generations_under_the_first_publish:?}"
    );
    assert!(
        !reclaimed_released_after_the_newest_root_of_every_device.is_empty(),
        "回收出来的槽里有释放代高于 22 的（txg 22 那条根还引用着，GA 下扣到生效）"
    );
}

/// 同一刀（卸载那一串先写系统配置之后、第一次空发布的单元写完、journal 记录与根都没落）之后一块盘丢掉、只剩一块（攻方丙 1 那一节自列没判的第二件）：
/// 丢掉的是住着现行那一版的根的盘 0（现行那一版 txg 24，24 mod 3 = 0，第一版区域归属 0 / 1 / 0），剩下的盘 1 上最新的根是 txg 22，
/// 早于它系统配置里的新 F（24）。择根与重放：恢复择到盘 1 上最新的那条根、journal 没往后施加，只读挂载走 txg 22 那一版、根上带旧 F；
/// 可写挂载在读任何盘之前就被 w 的下限拒。
/// GA 的结局（用户 2026-10-03 定甲 GA）：第一次空发布写下去、落在回收槽上的那几个单元，槽的释放代都不高于 22——txg 22 那条根不引用它们；
/// 释放代 23、24 的那几个（txg 22 那条根还引用着）扣到生效、没被写。改钉这一句的理由：原来钉的「写进回收出来的槽的那几个单元没让池级 checker
/// 多判出一条」分不出 GA 与丙 1——单盘镜像上卸载之前 checker 就红六七条，条名已饱和（判决 `research/prompts/m3-unlink-code-r3-main-verification.md`
/// 第二节 N1「第五节跑前判据的字面」那一行）；分得出的是被写的槽的释放代，与剩下那块盘的读回（`a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs`）。
/// checker 那一项照旧判（与「只写了系统配置、单元一个都没写」那一刀逐条相同），只记饱和、不靠它分辨。
/// 判别力：卸载回收的槽照旧整截扣到生效时拷贝上那一次卸载就在预演里被拒（这一刀摆不出来），第一步红。整截当场放开（丙 1）时这一形照样绿
/// （同 [`an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version`] 的说明）。
pub fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, _) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
    let mounted = mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
    let image_after_the_mount = pool.image();
    let (_, session) = MountedSession::of_the_mount(mounted);
    let current_root = *session.current.root();
    let unit_writes_of_the_first_publish = {
        let mut allocator = session.allocator.clone();
        let mut current = session.current.clone();
        let (mut devices, _) = fault_injecting_recorded_devices_on(&image_after_the_mount, None);
        let unmounted = unmount(
            &pool.parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            session.shadow_ledger,
        );
        let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
            panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
        };
        assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device(
            &session.allocator,
            &current_root,
            raised,
        );
        let first = &raised.publishes[0];
        let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
        first.writes.total().write_calls
            - structure_writes(WrittenStructureKind::JournalRecord)
            - structure_writes(WrittenStructureKind::RootSlot)
            - structure_writes(WrittenStructureKind::SystemConfigurationSlot)
    };
    let system_configuration_writes_of_the_sequence = 2;
    let surviving_device_after_losing_power_at = |first_failing_write: u64| {
        let (mut devices, _) = fault_injecting_recorded_devices_on(
            &image_after_the_mount,
            Some(write_failing_from(
                first_failing_write,
                FailingWrites::ThisWriteAndEveryLaterOne,
            )),
        );
        let mut allocator = session.allocator.clone();
        let mut current = session.current.clone();
        let unmounted = unmount(
            &pool.parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            session.shadow_ledger,
        );
        assert!(
            matches!(
                unmounted,
                Err(MountError::RaiseFloorSequencePublishFailed(_))
            ),
            "掉电之后那一串报错：{:?}",
            unmounted.as_ref().err()
        );
        let crashed = image_of_the_fault_injecting_recorded_devices(
            &devices,
            image_after_the_mount.device_size_in_bytes,
        );
        MemoryPool {
            devices: crashed
                .devices
                .iter()
                .filter(|(identity, _)| **identity != DeviceIdentity(0))
                .map(|(identity, device)| (*identity, device.clone()))
                .collect(),
            device_size_in_bytes: crashed.device_size_in_bytes,
        }
    };
    let survivor = surviving_device_after_losing_power_at(
        system_configuration_writes_of_the_sequence + unit_writes_of_the_first_publish + 1,
    );
    let survivor_with_only_the_system_configuration_written =
        surviving_device_after_losing_power_at(system_configuration_writes_of_the_sequence + 1);
    assert_eq!(
        (
            current_root.checkpoint_txg,
            choose_system_configuration(&survivor)
                .expect("剩下那块盘的系统配置读得出")
                .quantities
                .rollback_floor,
            recover(&survivor, JournalPolicy::Consult).effective_root,
        ),
        (
            CheckpointTxg(24),
            CheckpointTxg(24),
            Some((current_root.instance, CheckpointTxg(22)))
        ),
        "剩下的盘 1 系统配置里是新 F 24，恢复择到它上面最新的根 txg 22、journal 没往后施加"
    );
    let read_only = mount_read_only(&survivor).expect("剩下那块盘只读挂载");
    assert_eq!(
        (
            read_only.effective_root.checkpoint_txg,
            read_only.effective_root.rollback_floor,
        ),
        (CheckpointTxg(22), current_root.rollback_floor),
        "只读挂载走 txg 22 那一版，根上带旧 F，低于系统配置里的新 F"
    );
    let mut survivor_devices = plain_devices_on(&survivor);
    assert!(
        matches!(
            mount_after_the_crash_on(&pool.parameters, &mut survivor_devices),
            Err(MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. })
        ),
        "只剩一块盘：可写挂载被 w 的下限拒"
    );
    // 池级 checker 在单盘镜像上卸载之前就饱和（条名与只写了系统配置那一刀逐条相同），不靠它分辨
    images.hand_out(
        "只写了系统配置那一刀之后只剩盘 1",
        || survivor_with_only_the_system_configuration_written,
        PoolCheckerExpectation::only_the_named_invariants(),
    );
    images.hand_out(
        "第一次空发布的单元落下之后只剩盘 1",
        || survivor,
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated_like_the_image("只写了系统配置那一刀之后只剩盘 1"),
    );
}

/// 同 [`mount_after_the_crash_with_batches_of_one`]，盘由调用方给。
pub fn mount_after_the_crash_on(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
) -> Result<Mounted, MountError> {
    mount_writable_with_test_only_switches_and_deletion_switches(
        parameters,
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
}

/// 被拒的那一种（只分空间不够的两种；会话只因这两种推抬 F，`mounted_session::refusal_is_short_of_space`）。
pub fn kind_of_the_refusal_that_pushed(refusal: &PublishError) -> &'static str {
    if matches!(refusal, PublishError::SpaceAdmissionRefused(_)) {
        "SpaceAdmissionRefused"
    } else if matches!(refusal, PublishError::PlacementRefused { .. }) {
        "PlacementRefused"
    } else {
        panic!("会话只因空间不够推抬 F：{refusal:?}")
    }
}

/// 删掉之后同样大小的写回（D3（空间分配） 已定项 9 第 2 条：删掉 s 字节之后同样大小的写在 3 次改变用户可见状态的发布内做成）：
/// 两块单元区 256 槽的小盘，崩了再挂之后先覆盖写 3 次，再用顺序写把文件一个数据单元一个数据单元地长大，长到会话报空间不够（6 个单元：
/// 第 7 个单元准入先拒（C545，用户 2026-09-27 定；此前是走固定点时落点取不到），推抬 F 推到上限仍不够）；
/// 截断成 0 字节（删），再写回 6 个单元。写回被拒时发一次改变用户可见状态的小发布
/// （建一个空 inode）再试：第 1、2、3 次写回被拒（推抬 F 推到上限仍不够），各发一次建 inode 之后，第 4 次写回做成——
/// 删与写回之间夹 3 次改变用户可见状态的发布，在界 3 之内（条款逐字：删掉的块「要再发生 3 次这样的发布才回可分配集合」，写回接在那之后）。
/// ckpt_cost 按最坏情况计（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定）之前这一格夹 1 到 2 次；之后保留池与切换预留多扣，
/// 写回等到第 3 次之后。写回之后池级 checker 0 违例。
/// 先覆盖写几次：单元区起点随环长走之后（实审 A3b：小盘的环 6 MiB、单元区从槽 1408 起，分配记录树的叶按绝对槽号罩）改之前的 20 次
/// 写回一次都不被拒、走不到界 3 那一格；先覆盖写 0–45 次各跑一遍（实测），写回被拒三次的是 11、34、43 次，34 次那一格长到的单元数、
/// 被拒的次数与改之前 20 次那一格相同；另有几格截断本身就在上限处被空间拒，不取。
/// 记账标签 6 按这一版现算接进准入之后（D3（空间分配） 已定项 2，每块盘多扣 49193 字节，约 3 槽），0–45 次重扫一遍
/// （三·2 第 6 条实现员副本上实测）：写回被拒三次的只剩 26、44 次两格（都长到 5 个单元），取 26 次。
/// 环窗口真扣、小盘放大（用户 2026-10-02 定「甲」）与 ckpt_cost 加谱系那一项之后 0–9 次重扫（三·2 第 6 条实现员副本上的草稿探针，
/// 第 10 次那一格写回被拒之后建 inode 本身也被空间拒，探针停在那里）：写回被拒三次的是 3、9 次两格（都长到 6 个单元），取 3 次。
/// 判别力：准入拒不推时，写回一次都做不成（推不出被钉着的块之外的空间）。
pub fn after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes(
    images: &mut PoolCheckerImages,
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf256Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for overwrite_index in 1..=3 {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
    let unit_bytes = data_unit_payload_capacity();
    let mut units = 0;
    let growth_refusal = loop {
        match pool.sequential_write((units + 1) * unit_bytes) {
            Ok(_) => units += 1,
            Err(refusal) => break refusal,
        }
        assert!(units < 100, "256 槽的盘长不到 100 个单元");
    };
    assert_eq!(units, 6, "长到 6 个单元，第 7 个装不下");
    assert!(
        matches!(
            &growth_refusal,
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)
                if matches!(no_space.last_refusal, PublishError::SpaceAdmissionRefused(_))
                    && matches!(no_space.stop, FloorRaiseStop::FloorAlreadyAtTheCeiling { .. })
        ),
        "长大被拒：准入先拒（段外落不下）、推抬 F 推到上限仍落不下：{growth_refusal:?}"
    );
    pool.overwrite(0).expect("截断成 0 字节（删）做成");
    let mut user_visible_publishes_after_the_truncation = 0;
    let mut refused_write_backs = 0;
    loop {
        let write_back = pool.sequential_write(units * unit_bytes);
        match write_back {
            Ok(_) => {
                user_visible_publishes_after_the_truncation += 1;
                break;
            }
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => {
                refused_write_backs += 1;
                assert!(refused_write_backs <= 7, "写回在 8 次之内做成");
                pool.create_one_inode()
                    .expect("建一个空 inode（改变用户可见状态的小发布）做成");
                user_visible_publishes_after_the_truncation += 1;
            }
            Err(refusal) => panic!("写回只会因为空间不够被拒：{refusal:?}"),
        }
    }
    assert_eq!(
        (
            refused_write_backs,
            user_visible_publishes_after_the_truncation
        ),
        (3, 4),
        "写回被拒三次、建三次 inode，删之后第 4 次改变用户可见状态的发布（写回本身）做成"
    );
    let user_visible_publishes_between_the_truncation_and_the_write_back =
        user_visible_publishes_after_the_truncation - 1;
    assert!(
        user_visible_publishes_between_the_truncation_and_the_write_back <= 3,
        "D3（空间分配） 已定项 9 第 2 条的界：删掉的块要再发生 3 次改变用户可见状态的发布才回可分配集合，删与写回之间至多夹 3 次"
    );
    images.hand_out(
        "写回之后",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 只崩了再挂、一个字节不写（攻方「只崩了再挂」那一形）：两块单元区 240 槽的小盘，第一个文件之后连着崩了再挂 15 次。
/// 每次挂载写行与暖机，换下的固定点进 defer，崩了再挂不抬 F，环里留着约 6 次挂载的 defer；取号之前那一判从第 7 次起不够
/// （环窗口真扣、小盘放大与 ckpt_cost 加谱系那一项之前是第 8 次，副本上实测）。
/// 不推的话第 7 次在取号之前被拒、之后每一次判的都是同一份盘，挂死在那里。可写挂载准入不够时写行之后推抬 F 的空发布再判：
/// ckpt_cost 按最坏情况计（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定；窄盘更早报 ENOSPC 用户认了）之后，第 7 次起每次推满仍不够（第 7 次 F 已在上限 0、一串都推不了，第 8 次起各推两串）
/// （一次准入的预算做满）仍不够、挂载照样做成（C565（挂载处推满仍不够怎么收尾没定） 那一种收尾，D2（RAID 条带策略） 已定项 13 那一格的例外），
/// 这块盘上再也回不到「够」。15 次挂载全做成，每次挂载之后池级 checker 0 违例，F 只升不降。
/// 判别力：挂载那一处不推时，第 7 次在取号之前被拒，之后每一次都被拒。
pub fn crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck(
    images: &mut PoolCheckerImages,
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf240Slots);
    let mut admissions: Vec<&'static str> = Vec::new();
    let mut floor_before = CheckpointTxg(0);
    for mount_index in 1..=15 {
        let output = pool
            .crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
        admissions.push(match output.space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition => "取号之前就够",
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => "推了再判够了",
            MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => "推满仍不够",
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "没判",
        });
        assert!(
            pool.rollback_floor() >= floor_before,
            "第 {mount_index} 次：F 只升不降（{floor_before:?} → {:?}）",
            pool.rollback_floor()
        );
        floor_before = pool.rollback_floor();
        images.hand_out(
            &format!("第 {mount_index} 次挂载之后"),
            || pool.image(),
            PoolCheckerExpectation::no_invariant_violated(),
        );
    }
    assert_eq!(
        admissions,
        [["取号之前就够"; 6].as_slice(), &["推满仍不够"; 9]].concat(),
        "第 7 次起每次推满仍不够、挂载照样做成"
    );
}

/// 推满仍不够怎么收尾（C565（挂载处推满仍不够怎么收尾没定）；实现员提的那一种，交主 agent 定）：挂载照样做成——实例已取、行已写、
/// 暖机与推的那几串都已落盘，交回 `MountSpaceAdmission::StillShortAfterTheFloorRaises`；之后的发布照发布路径的准入判，不够就报空间不够；
/// 正常卸载照常可走（它抬 F 到现行那一版、不判上限），卸载之后再挂取号之前就够。
/// 这一格：两块单元区 240 槽的小盘，第一个文件之后连着崩了再挂，第 10 次取号之前每块盘短 30 槽；写行与暖机之后推两串（F 抬到 20、再到 23，
/// 各 3 次空发布），再判每块盘仍短 28 槽（单元区起点随环长走之后单元区 [1408, 1648) 跨叶 1、叶 2 的槽数从 168 / 72 变成 216 / 24，
/// 每次发布重写的叶跟着变，短的槽数从 36 / 34 变成 30 / 28，实测；F 的两个上限与推的次数不变），再推一串就是 6 + 3 + 1 = 10 次发布、超过一次准入的 8 次，不再推（ckpt_cost 按最坏情况计，
/// D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定：第 8、9 次挂载已经推满仍不够、各推两串，F 在第 10 次之前已到 14）。
/// 环窗口真扣、小盘放大（单元区 [1408, 2387)）与 ckpt_cost 加谱系那一项之后，短的槽数按槽整除是 37 / 39（副本上实测），F 的两个上限与推的次数不变。
/// 这次挂载的会话里覆盖写一次：
/// 推两串（又做满预算）仍不够，报空间不够（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`）。正常卸载做成，卸载之后再挂取号之前就够、
/// 池级 checker 0 违例。
/// 判别力：换成另一种收尾（推满仍不够就报可写挂载被拒），第 10 次挂载交回错误，用例在那一步就红。
pub fn mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount(
    images: &mut PoolCheckerImages,
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf240Slots);
    for mount_index in 1..=9 {
        pool.crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
    }
    let output = pool
        .crash_and_mount_writable()
        .expect("第 10 次：推满仍不够，挂载照样做成");
    let MountSpaceAdmission::StillShortAfterTheFloorRaises {
        refusal_before_acquisition,
        floor_raises,
        last_refusal,
        stop,
    } = &output.space_admission
    else {
        panic!("第 10 次推满仍不够：{:?}", output.space_admission);
    };
    let slots_short = |refusal: &singlefs_core::admission::AdmissionRefusedOnSomeDevices| {
        refusal
            .short_devices
            .iter()
            .map(|short| {
                (
                    short.device,
                    short.available.0 / i128::from(singlefs_format::SLOT_BYTES),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        slots_short(refusal_before_acquisition),
        vec![(DeviceIdentity(0), -37), (DeviceIdentity(1), -37)],
        "取号之前按槽整除每块盘短 37 槽（含标签 6 摊到每块盘的 49193 字节；环窗口真扣、小盘放大、ckpt_cost 加谱系那一项之后副本上实测，此前 −33）"
    );
    assert_eq!(
        floor_raises
            .iter()
            .map(|raised| (raised.ceiling, raised.publishes.len()))
            .collect::<Vec<_>>(),
        vec![(CheckpointTxg(20), 3), (CheckpointTxg(23), 3)],
        "写行与暖机之后推两串：F 抬到 20、再到 23"
    );
    assert_eq!(
        slots_short(last_refusal),
        vec![(DeviceIdentity(0), -39), (DeviceIdentity(1), -39)],
        "推两串之后再判按槽整除每块盘仍短 39 槽（同上实测，此前 −31）"
    );
    assert!(
        matches!(
            stop,
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded {
                publishes_pushed: 6,
                publishes_of_the_next_raise: 3,
            }
        ),
        "再推一串就超过一次准入的 8 次发布：{stop:?}"
    );
    assert_eq!(pool.rollback_floor(), CheckpointTxg(23), "现行那一版带新 F");
    let write = pool.overwrite(OVERWRITE_BYTES);
    assert!(
        matches!(
            &write,
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space))
                if matches!(no_space.last_refusal, PublishError::SpaceAdmissionRefused(_))
                    && matches!(
                        no_space.stop,
                        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
                    )
        ),
        "这次挂载的会话里写：推满仍不够，报空间不够：{write:?}"
    );
    let mut session = pool.session.take().expect("第 10 次挂载的会话还在");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .expect("正常卸载做成");
    assert!(
        matches!(unmounted, Unmounted::FloorRaisedToTheCurrentVersion(_)),
        "卸载推了一串带卸载记号的空发布"
    );
    let after_unmount = pool.crash_and_mount_writable().expect("卸载之后再挂");
    assert!(
        matches!(
            after_unmount.space_admission,
            MountSpaceAdmission::AdmittedBeforeAcquisition
        ),
        "卸载之后再挂取号之前就够：{:?}",
        after_unmount.space_admission
    );
    images.hand_out(
        "卸载之后再挂的镜像",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
