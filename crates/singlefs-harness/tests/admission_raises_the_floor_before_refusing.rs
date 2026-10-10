//! 里程碑「覆盖写、释放、回退与复用」收尾实现批次实五：准入不够时先推抬 F 的空发布再判（D16（发布语义） 已定项 1「准入」那一行，
//! C283（准入失败时不先推发布就报 ENOSPC）），挂载那一处写行之后也推，推满仍不够怎么收尾（C565（挂载处推满仍不够怎么收尾没定），
//! 实现员提的那一种），发布路径的切换预留按下一次挂载的 rows0（D28（挂载期承诺量） 已定项 3「发布路径用 rows0 + 1」），
//! 与算抬 F 上限时根槽这一次读坏（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，C562（抬 F 算上限时根槽读坏当没有根））。
//!
//! 用例的形状照安全设计轮第三轮攻方（`research/prompts/m2-safety-r3-opus-output.md`）：会话里数据单元落不下（准入先拒之后是准入拒，
//! 关掉准入时是落点被拒，C545）、删掉之后同样大小的写回、
//! 只崩了再挂的窄池、实例表行数逼近 369 的整数倍；445 那一格照实四丙的构造（种子基 + 339 第 1 步）。
//! 盘都是内存里的稀疏盘，两块、单元区 240 / 256 / 384 槽（`HistoryDeviceWidth` 那几档），一个进程里 mkfs、取号、暖机、第一个文件起步。
//! 崩在准入推的那一串空发布中间那一形在 `crash_enumeration_floor_raise_pushed_by_admission.rs`（层 0 的枚举，单独一个测试二进制）。

mod common_admission;

use common_admission::{
    checker_violations_on, plain_devices_on, space_admission_of_a_crash_remount_on_a_copy,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, DeviceOverASparseImage, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{PlacementRefusal, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable_with_space_admission,
    mount_writable_with_test_only_switches_and_deletion_switches, raise_rollback_floor,
    raise_rollback_floor_to_the_admission_ceiling, ring_slots_known_to_hold_a_root_by,
    rollback_floor_ceiling, unmount, DeletionQueueContinuationOutcome, FloorRaiseStop, MountError,
    MountSpaceAdmission, Mounted, RaiseToTheAdmissionCeiling, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, recover, BadRootRingSlotReading,
    JournalPolicy,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion, PoolWriter, PublishError,
    TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_core::write_accounting::{
    WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    execute_history_with_faults, generate_history_with_weights, GenerationWeights,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    StepOutcome, StepPosition,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
use singlefs_harness::model_comparison::refusal_reason_of_mount_error;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 会话里 unlink 之前覆盖写几次：换下的单元进 defer，等推的抬 F 回收（意图那次被拒之后靠它们腾出槽）。
const OVERWRITES_BEFORE_THE_UNLINK: usize = 10;

/// 每块盘留下几个没挡的槽（其余的标成已分配，造盘面）：比 unlink 意图那次落得下那一判要的少，又够推的那一串抬 F 的空发布落下。
/// 草稿探针（三·2 第 8 位实现员副本上）在 0–80 槽上每隔 2 槽现跑：0–12 槽推的那一串自己也落不下（推满仍不够）；14–18 槽意图被落得下那一判拒、
/// 推一串抬 F 之后做成；20–28 槽意图放行、最后一批被落得下那一判拒（那时走空里的删除类发布不推抬 F，原样交回）；30 槽起一次都不拒。取窗口中间的 16。
const UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE: usize = 16;

/// 意图放行、最后一批被落得下那一判拒、推的抬 F 落得下的那一格（Q-V 那条用例）：覆盖写 11 次、每块盘留 28 槽。
/// 第九位实现员副本上的草稿探针（覆盖写 8–13 次 × 留 14–32 槽逐格现跑，Q-V 之后）：覆盖写 10 次时 20–29 槽意图放行、最后一批被拒，
/// 推的那一串要三次空发布、自己第 1–3 次就落不下（推满仍不够，交回 `NoSpaceAfterRaisingTheFloor`、意图已落盘）；
/// 覆盖写 8、9、11、12 次时 27–29 槽推的那一串两次空发布落得下：意图（+1）、推的两次（+2、+3）、最后一批（+4）。取 11 次、28 槽。
const OVERWRITES_BEFORE_THE_UNLINK_WHOSE_LAST_BATCH_IS_REFUSED: usize = 11;

/// 意图放行、最后一批被落得下那一判拒、为它推的那一串在预演里也落不下的那一格（推满仍不够）：覆盖写 [`OVERWRITES_BEFORE_THE_UNLINK`] 次、
/// 每块盘留 24 槽。第九位实现员的草稿探针在覆盖写 10 次上量到 20–29 槽都是这一形（推的那一串要三次空发布，第 1–3 次就落不下），取中间的 24。
const UNBLOCKED_SLOTS_LEFT_WHEN_THE_RAISE_ALSO_FINDS_NO_SLOT: usize = 24;
const UNBLOCKED_SLOTS_LEFT_FOR_THE_LAST_BATCH_TO_BE_REFUSED: usize = 28;

/// 会话里「落得下那一判拒、推抬 F、回收之后再发」那一形（C545 准入先拒之后、D16（发布语义） 已定项 1「准入」那一行）：小盘为环窗口补足之后
/// 普通写被式子先拒、走不到这一形，用户 2026-10-02 定 Q-T「甲」改用删除类发布去撞（删除类发布不判式子、落点那一判照判，D28（挂载期承诺量） 已定项 1）。
/// 两块单元区 384 槽的小盘，第一个文件之后崩了再挂，这次挂载里覆盖写 10 次（换下的单元进 defer），再把每块盘没挡的槽标成已分配、
/// 只留 [`UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE`] 槽（造盘面，比意图那次要的少）。经会话 unlink 第一个文件：意图那次落得下那一判拒
/// （`SpaceAdmissionRefused`），会话推一串抬 F 的空发布（没有普通分配、不判准入，落得下），F 抬上去、defer 里的落点回收之后再发，
/// 意图与最后一批做成：推它的只这一次、是落得下那一判；现行那一版是有树而文件集合为空的一版、队列走空、墓碑树一条。
/// 池级 checker 不判：标成已分配的槽只在分配器的位图上（不写记录），之后的发布照分配器的计数写记账行，记账的已分配与遍历按造法就对不上。
/// 判别力：落得下那一判拿掉时推它的是 `PlacementRefused`（会话照样推、照样做成），那一项红。
#[test]
fn an_unlink_whose_intent_finds_no_unblocked_slot_is_refused_by_the_landing_judgement_and_published_after_raising_the_floor_in_the_session(
) {
    let mut pool = pool_ready_to_unlink_with_few_unblocked_slots(
        OVERWRITES_BEFORE_THE_UNLINK,
        UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE,
    );
    let floor_before = pool.rollback_floor();
    let published = pool
        .publish(
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: pool.write_time_seconds,
            },
            None,
        )
        .unwrap_or_else(|refusal| panic!("unlink 推过抬 F 之后做成：{refusal:?}"));
    assert_eq!(
        published
            .refusals_that_pushed_the_floor_raises
            .iter()
            .map(kind_of_the_refusal_that_pushed)
            .collect::<Vec<_>>(),
        vec!["SpaceAdmissionRefused"],
        "推过一串抬 F，推它的是意图那次落得下那一判"
    );
    assert!(
        pool.rollback_floor() > floor_before,
        "F 从 {floor_before:?} 抬上去了"
    );
    let PoolVersion::WithTreesAndNoFile(emptied) = &pool.session().current else {
        panic!("意图与最后一批都做成：现行那一版有树而文件集合为空");
    };
    assert_eq!(
        (
            emptied.deleted_inodes.len(),
            emptied
                .tombstone_containers
                .iter()
                .map(|container| container.contents.records.len())
                .sum::<usize>(),
        ),
        (0, 1),
        "队列走空、墓碑树一条（第一个文件那一条）"
    );
}

/// 会话里 unlink 走空时删除的一批被落得下那一判拒、推抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」）：
/// 盘面照上一条的造法（两块 384 槽的小盘、覆盖写 [`OVERWRITES_BEFORE_THE_UNLINK_WHOSE_LAST_BATCH_IS_REFUSED`] 次之后每块盘只留
/// [`UNBLOCKED_SLOTS_LEFT_FOR_THE_LAST_BATCH_TO_BE_REFUSED`] 个没挡的槽），意图那次放行、最后一批被落得下那一判拒（`SpaceAdmissionRefused`）。会话推一串抬 F 的空发布、回收之后接着走空，unlink 做成：
/// 推它的只这一次、是落得下那一判；推的那一串接在意图之后（它的第一次空发布的 txg 比 unlink 之前那一版至少大 2：意图那次占了一个）；
/// F 抬上去了；现行那一版有树而文件集合为空、队列走空、墓碑树一条。池级 checker 不判（同上一条：标成已分配的槽只在分配器的位图上）。
/// 判别力：走空里不推（Q-V 之前的做法）时会话交回 `Publish { cause: SpaceAdmissionRefused, floor_raises: [] }`，意图已落盘、队列留一条。
#[test]
fn an_unlink_whose_last_batch_finds_no_unblocked_slot_is_refused_by_the_landing_judgement_and_published_after_raising_the_floor_in_the_session(
) {
    let mut pool = pool_ready_to_unlink_with_few_unblocked_slots(
        OVERWRITES_BEFORE_THE_UNLINK_WHOSE_LAST_BATCH_IS_REFUSED,
        UNBLOCKED_SLOTS_LEFT_FOR_THE_LAST_BATCH_TO_BE_REFUSED,
    );
    let floor_before = pool.rollback_floor();
    let checkpoint_txg_before_the_unlink = pool.session().current.root().checkpoint_txg;
    let published = pool
        .publish(
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: pool.write_time_seconds,
            },
            None,
        )
        .unwrap_or_else(|refusal| panic!("unlink 走空时推过抬 F 之后做成：{refusal:?}"));
    assert_eq!(
        published
            .refusals_that_pushed_the_floor_raises
            .iter()
            .map(kind_of_the_refusal_that_pushed)
            .collect::<Vec<_>>(),
        vec!["SpaceAdmissionRefused"],
        "推过一串抬 F，推它的是删除那一批的落得下那一判"
    );
    let first_floor_raise_publish_txg = published.floor_raises[0]
        .publishes
        .first()
        .expect("推的那一串至少一次空发布")
        .root
        .checkpoint_txg;
    assert!(
        first_floor_raise_publish_txg.0 >= checkpoint_txg_before_the_unlink.0 + 2,
        "推的那一串接在意图那次之后：第一次空发布 {first_floor_raise_publish_txg:?}，unlink 之前 {checkpoint_txg_before_the_unlink:?}"
    );
    assert!(
        pool.rollback_floor() > floor_before,
        "F 从 {floor_before:?} 抬上去了"
    );
    let PoolVersion::WithTreesAndNoFile(emptied) = &pool.session().current else {
        panic!("意图与最后一批都做成：现行那一版有树而文件集合为空");
    };
    assert_eq!(
        (
            emptied.deleted_inodes.len(),
            emptied
                .tombstone_containers
                .iter()
                .map(|container| container.contents.records.len())
                .sum::<usize>(),
        ),
        (0, 1),
        "队列走空、墓碑树一条（第一个文件那一条）"
    );
}

/// 会话里 unlink 走空时最后一批被落得下那一判拒、推的那一串自己也落不下（推满仍不够，用户 2026-10-03 定案 Q-V「每次被拒各算一次」）：
/// 盘面照上一条的造法，覆盖写 [`OVERWRITES_BEFORE_THE_UNLINK`] 次、每块盘只留 [`UNBLOCKED_SLOTS_LEFT_WHEN_THE_RAISE_ALSO_FINDS_NO_SLOT`] 槽。
/// 意图那次放行（已落盘）、最后一批被落得下那一判拒（`SpaceAdmissionRefused`），为它推的那一串在预演里就落不下：
/// 会话交回 `NoSpaceAfterRaisingTheFloor`——最后那一次被拒是落得下那一判、一串都没推成、停因是抬 F 那一串自己被空间拒，
/// 交回的发成的几版只有意图那一版；现行那一版就是它（队列留第一个文件那一条、F 没动）。
/// 判别力：推满仍不够时把已落盘的意图那一版丢掉不交（`versions_published` 空）、或照「整次调用共用一份预算」先拒意图，那几项红。
#[test]
fn an_unlink_whose_last_batch_and_the_floor_raise_pushed_for_it_both_find_no_unblocked_slot_hands_back_no_space_with_the_intent_published(
) {
    let mut pool = pool_ready_to_unlink_with_few_unblocked_slots(
        OVERWRITES_BEFORE_THE_UNLINK,
        UNBLOCKED_SLOTS_LEFT_WHEN_THE_RAISE_ALSO_FINDS_NO_SLOT,
    );
    let floor_before = pool.rollback_floor();
    let checkpoint_txg_before_the_unlink = pool.session().current.root().checkpoint_txg;
    let refusal = pool
        .publish(
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: pool.write_time_seconds,
            },
            None,
        )
        .expect_err("最后一批推满仍不够");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("推满仍不够交回空间不够：{refusal:?}");
    };
    assert_eq!(
        (
            kind_of_the_refusal_that_pushed(&no_space.last_refusal),
            no_space.floor_raises.len(),
            matches!(
                no_space.stop,
                FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
            ),
        ),
        ("SpaceAdmissionRefused", 0, true),
        "最后那一次被拒是落得下那一判、一串都没推成、停因是抬 F 那一串自己被空间拒：{:?}",
        no_space.stop
    );
    assert_eq!(
        no_space
            .versions_published
            .iter()
            .map(|version| (
                version.root.checkpoint_txg.0 - checkpoint_txg_before_the_unlink.0,
                version.deleted_inodes.clone(),
            ))
            .collect::<Vec<_>>(),
        vec![(1, vec![InodeNumber(FIRST_INODE_NUMBER)])],
        "交回的发成的几版只有意图那一版（txg 差 1、队列里是第一个文件）"
    );
    let current = pool
        .session()
        .current
        .version_of_trees()
        .expect("意图那一版有树");
    assert_eq!(
        (
            current.root.checkpoint_txg.0 - checkpoint_txg_before_the_unlink.0,
            current.deleted_inodes.clone(),
            current.root.rollback_floor,
        ),
        (1, vec![InodeNumber(FIRST_INODE_NUMBER)], floor_before),
        "现行那一版是意图那一版：队列留一条、F 没动"
    );
}

/// 挂载续做那两条的窄盘：不为环窗口补足（准入关着造盘面，`start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off`）、
/// 单元区 292 槽；第一个文件之后覆盖写 [`OVERWRITES_BEFORE_THE_CONTINUATION`] 次，再顺序写把它长到 [`DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION`] 个数据单元。
/// 第十位实现员副本上的草稿探针（单元区 192 / 256 / 320 / 384 槽 × 覆盖写 0–12 次 × 长到 1–14 个单元 × 崩在意图之后或第一批之后，逐格现跑）定的是
/// 256 槽、覆盖写 2 次、14 个单元；第 50 批退役树进准入之后挂载那一判每块盘多扣约 42 槽（标签 6 退役 3 槽、ckpt_cost 退役 3 块进保留池、切换预留跟着涨），
/// 那一格崩在意图之后变成「取号之前就不够、先推一串」。第 50 批补做重扫（单元区 256–304 槽每 4 槽 × 覆盖写 0–4 次 × 长到 10–24 个单元 ×
/// 崩在意图之后、第一批之后、第二批之后，逐格现跑）：292 槽、覆盖写 1 次、16 个单元那一格崩在意图之后是「取号之前那一判放行、续做里一批被落得下那一判拒、
/// 推一串两次空发布之后走完」，崩在第一批之后是「取号之前放行、续做推不出空间、挂载照样做成可写」，不先覆盖写、崩在第二批之后是「续做走到空、一串都没推」——
/// 三种场景同在一格的，256–304 槽里只有这一格。
const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION: u64 = 292;
const OVERWRITES_BEFORE_THE_CONTINUATION: usize = 1;
const DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION: usize = 16;
/// 单元区 192 槽那几条（准入与续做都推过抬 F）的文件单元数：它们的造形没随第 50 批改，仍是 14 个单元。
const DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES: usize = 14;

/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
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
fn pool_crashed_between_deletion_batches_on_narrow_devices(
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
struct NarrowPoolCrashedBetweenDeletionBatches {
    unit_area_slots: u64,
    overwrites_before_the_unlink: usize,
    data_units_of_the_file: usize,
    batches_published_after_the_intent: usize,
}

/// 照 `shape` 造盘：准入关着的窄盘上第一个文件之后覆盖写、顺序写长大，在这个进程里直接发 unlink 的意图那次与之后那几批
/// （批大小 1，不经会话、不走空），然后进程退出（会话丢掉）。交回池与退出那一刻现行那一版的根。
fn pool_crashed_between_deletion_batches(
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
fn mount_after_the_crash_with_batches_of_one(
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
#[test]
fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version(
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
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "续做推过抬 F、走完之后池级 checker 全绿"
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
#[test]
fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue(
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
    assert_eq!(
        checker_violations_on(&image_after_the_unmount),
        Vec::<String>::new(),
        "卸载之后池级 checker 全绿"
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
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "再挂续做之后池级 checker 全绿"
    );
}

/// 现行那一版是 `PoolVersion` 的哪一个成员。
fn member_name_of(version: &PoolVersion) -> &'static str {
    match version {
        PoolVersion::WithoutFile(_) => "WithoutFile",
        PoolVersion::WithFile(_) => "WithFile",
        PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
    }
}

/// 推满仍放不下、挂载照样做成可写之后进程退出，同一份窄盘再可写挂载（代码轮 U5，攻方 `m3-unlink-code-r1` A5 / E8c 那一形）：
/// 上一次挂载把最后的空槽吃掉了，这一次取号之前在分配器的拷贝上预演写行那一串就取不到树表的落点，拒成
/// `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`（要取的号 3、写行那次、暖机计划一次、树表、每块盘上都没有），
/// 胶水把它映射成模型的单元区墙；一个写都不发：录制流 0 步、两块盘逐字节不变。
/// 判别力：预演取不到落点也照样取号（变异行「取号之前的预演取不到落点也照样取号」）时挂载写了盘才被拒，录制流不是 0 步；
/// 落点被拒的原因报成别的成员、或胶水把它映射成模型没有的理由，那几项红。
#[test]
fn after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the_placement_before_acquisition_writing_nothing(
) {
    let (mut pool, _) = pool_crashed_between_deletion_batches_on_narrow_devices(1);
    let first = mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("推满仍放不下：挂载照样做成可写：{error:?}"));
    assert!(
        matches!(
            first.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
        ),
        "第一次挂载续做推满仍放不下：{:?}",
        first.output.deletion_queue_continuation_outcome
    );
    drop(first);
    let image_before = pool.image();
    let stream = SharedStream::new();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&image_before)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
    let refusal = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut recorded,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect_err("取号之前预演取不到落点：拒");
    assert!(
        matches!(
            refusal,
            MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
                instance_to_acquire: InstanceGeneration(3),
                publish_index: 0,
                warm_up_publishes_planned: 1,
                unit: TransactionUnit::TreeTable,
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
            }
        ),
        "取号之前在拷贝上预演写行那一串，树表在每块盘上都没有落点：{refusal:?}"
    );
    assert_eq!(
        refusal_reason_of_mount_error(&refusal),
        ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),
        "胶水把它映射成模型的单元区墙"
    );
    assert_eq!(
        stream.operation_count(),
        0,
        "录制流 0 步：一个写、一道屏障都没发"
    );
    let image_after = singlefs_harness::memory_pool::MemoryPool {
        devices: recorded
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: image_before.device_size_in_bytes,
    };
    assert!(image_after == image_before, "两块盘逐字节不变");
}

/// 续做走到空、F 一次都没抬、可删的单元都还在 defer 里的盘面上卸载（攻方 `m3-unmount-after-short-continuation-r1` 表 A 那 9 格的形：
/// 卸载被拒不绑队列）：单元区 292 槽、不先覆盖写、16 个单元，崩在意图与两批之后。重开、准入判着、删除批大小 1：取号之前那一判放行、
/// 没推抬 F，续做 14 步走到空。挂载之后每块盘只剩 10 个没挡的空槽，卸载那一串两次空发布要 14 个——扣着回收出来的槽就在预演里取不到落点；
/// 甲 GA 之下预演算上新 F 回收出来、当场放开的那一截，卸载做成、F 抬到现行那一版，池级 checker 全绿，再挂可写做成。
/// 判别力：卸载回收的槽照旧扣到生效时卸载在预演里被拒，卸载那一步红。
#[test]
fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims(
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
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "卸载之后池级 checker 全绿"
    );
    mount_after_the_crash_with_batches_of_one(&mut pool)
        .unwrap_or_else(|error| panic!("卸载之后再挂可写：{error:?}"));
}

/// 一块稀疏盘外面先包录制器（录下真落到盘上的每一步），再包可注错的那一层。
type FaultInjectingRecordedDevice =
    FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

/// 一份镜像交给两块新的 [`FaultInjectingRecordedDevice`]（`schedule` 为 `None` 时只数调用、不注错），连同它们共用的那条录制流。
fn fault_injecting_recorded_devices_on(
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
fn image_of_the_fault_injecting_recorded_devices(
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
enum FailingWrites {
    OnlyThisWrite,
    ThisWriteAndEveryLaterOne,
}

fn write_failing_from(ordinal: u64, failing_writes: FailingWrites) -> FaultSchedule {
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
fn landed_writes_and_root_slot_writes(stream: &SharedStream) -> (u64, u64) {
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

/// 几份写账的写调用数之和。
fn write_calls_of<'ledger>(
    ledgers: impl IntoIterator<Item = &'ledger WritesByStructureKind>,
) -> u64 {
    ledgers
        .into_iter()
        .fold(WriteCallsAndBytes::NONE, |sum, ledger| {
            sum.plus(ledger.total())
        })
        .write_calls
}

/// 取号那两次系统配置槽写（每块盘一次）：不是发布，挂载交回的错与输出里的写账都不含它们（`PublishSequenceFailed` 的契约）。
const ACQUISITION_WRITES: u64 = 2;

/// 一次不注错的可写挂载里，txg 低于 `txg` 的那几次发布（写行、暖机、准入推的与续做推的空发布、续做的各步）与第一次发布低于它的
/// 那几串抬 F 先写系统配置那一步，一共交给设备几个写（连取号那两次）；交回 (写数, 发布数)。注错点就按它摆在 txg 那一次发布的第一个写上。
fn writes_and_publishes_of_the_mount_before(
    output: &singlefs_core::mount::MountOutput,
    txg: CheckpointTxg,
) -> (u64, usize) {
    let publishes: Vec<(CheckpointTxg, &WritesByStructureKind)> =
        std::iter::once(&output.row_publish)
            .chain(&output.warm_up_publishes)
            .map(|version| (version.root().checkpoint_txg, version.writes()))
            .chain(
                output
                    .space_admission
                    .floor_raises()
                    .iter()
                    .chain(&output.floor_raises_pushed_by_the_continuation)
                    .flat_map(|raised| raised.publishes.iter())
                    .chain(&output.deletion_queue_continuation)
                    .map(|publish| (publish.root.checkpoint_txg, &publish.writes)),
            )
            .filter(|(publish_txg, _)| *publish_txg < txg)
            .collect();
    let system_configuration_writes_of_the_raises = write_calls_of(
        output
            .space_admission
            .floor_raises()
            .iter()
            .chain(&output.floor_raises_pushed_by_the_continuation)
            .filter(|raised| {
                raised
                    .publishes
                    .first()
                    .is_some_and(|first| first.root.checkpoint_txg < txg)
            })
            .map(|raised| &raised.system_configuration_writes_before_the_first_publish),
    );
    (
        ACQUISITION_WRITES
            + write_calls_of(publishes.iter().map(|(_, writes)| *writes))
            + system_configuration_writes_of_the_raises,
        publishes.len(),
    )
}

/// 挂载续做推过抬 F 的三种起点（代码轮 V2、V3，攻方 `m3-unlink-code-r2` G2、G3 那几格）：同一份崩溃镜像先在一份拷贝上不注错地挂一次，
/// 数清各次发布的写，再在另一份上让 txg 为 `failing_publish` 的那一次续做发布的第一个写报块设备错。交回不注错那一次的挂载输出、
/// 注错那一次的错与设备一层真落下的写与根槽写。
fn mount_failing_on_the_first_write_of_a_continuation_step(
    shape: NarrowPoolCrashedBetweenDeletionBatches,
    choose_the_failing_step: impl Fn(&singlefs_core::mount::MountOutput) -> CheckpointTxg,
) -> (
    singlefs_core::mount::MountOutput,
    CheckpointTxg,
    MountError,
    (u64, u64),
) {
    mount_failing_on_a_write_of_a_continuation_publish(shape, choose_the_failing_step, |_, _| 1)
}

/// 同 [`mount_failing_on_the_first_write_of_a_continuation_step`]，注错点摆在 txg 为 `failing_publish` 的那一次发布（推的抬 F 那一串的第一次空发布时，
/// 从它先写系统配置那一步数起）的第 `ordinal_within_the_publish(输出, failing_publish)` 个写上（从 1 数）。
fn mount_failing_on_a_write_of_a_continuation_publish(
    shape: NarrowPoolCrashedBetweenDeletionBatches,
    choose_the_failing_step: impl Fn(&singlefs_core::mount::MountOutput) -> CheckpointTxg,
    ordinal_within_the_publish: impl Fn(&singlefs_core::mount::MountOutput, CheckpointTxg) -> u64,
) -> (
    singlefs_core::mount::MountOutput,
    CheckpointTxg,
    MountError,
    (u64, u64),
) {
    let (pool, _) = pool_crashed_between_deletion_batches(shape);
    let image = pool.image();
    let mut counting_devices = plain_devices_on(&image);
    let counted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut counting_devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .unwrap_or_else(|error| panic!("拷贝上不注错的那一次挂载做成：{error:?}"));
    let failing_step = choose_the_failing_step(&counted.output);
    let (writes_before_the_failing_step, _) =
        writes_and_publishes_of_the_mount_before(&counted.output, failing_step);
    let (mut failing_devices, stream) = fault_injecting_recorded_devices_on(
        &image,
        Some(write_failing_from(
            writes_before_the_failing_step
                + ordinal_within_the_publish(&counted.output, failing_step),
            FailingWrites::OnlyThisWrite,
        )),
    );
    let refusal = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut failing_devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect_err("续做那一步的第一个写报块设备错");
    (
        counted.output,
        failing_step,
        refusal,
        landed_writes_and_root_slot_writes(&stream),
    )
}

/// 续做那一格交回的 `Publish` 错：已落盘份数 = 设备一层落下的根槽写数，写账合计（这一串里不属于任何发布的写 + 已落盘各份 + 失败账）
/// = 落下的写 − 取号两次（代码轮 V2 / V3 的判法）。
fn assert_the_continuation_publish_error_accounts_for_every_landed_write(
    refusal: &MountError,
    landed_writes: u64,
    landed_root_slot_writes: u64,
    publishes_before_the_failing_step: usize,
) {
    let MountError::Publish(failed) = refusal else {
        panic!("续做那一步的块设备错交回 MountError::Publish：{refusal:?}");
    };
    assert!(
        matches!(failed.cause, PublishError::BlockDevice(_)),
        "错是那一次写的块设备错：{:?}",
        failed.cause
    );
    assert_eq!(
        (
            failed.writes_of_persisted_publishes.len(),
            u64::try_from(failed.writes_of_persisted_publishes.len()).expect("份数"),
        ),
        (publishes_before_the_failing_step, landed_root_slot_writes),
        "已落盘的发布一次一份，份数等于设备一层落下的根槽写数（推的抬 F 的空发布也在里面）"
    );
    assert_eq!(
        write_calls_of(
            std::iter::once(&failed.writes_before_the_first_publish)
                .chain(&failed.writes_of_persisted_publishes)
                .chain(&failed.writes_of_failed_publishes)
        ),
        landed_writes - ACQUISITION_WRITES,
        "写账合计等于落下的写减去取号那两次（推的每一串先写系统配置那一步在 writes_before_the_first_publish 里）"
    );
}

/// 准入推过抬 F 的起点（代码轮 V3 第一形）：单元区 292 槽、覆盖写 1 次、16 个单元、崩在第 7 批之后；取号之前不够、写行与暖机之后推了一串
/// （三次空发布），续做 9 步、没再推。续做第二步的第一个写报块设备错：错里 7 份（写行、暖机两次、推的三次、续做第一步）= 落下的 7 个根槽写，
/// 写账合计 = 落下的写 − 取号两次（推的那一串先写系统配置那 2 个写在内）。
/// 判别力：续做之前的写账去掉准入推的那几次空发布（攻方 MU1）时份数少 3；先写系统配置那一步不进错（改之前）时合计少 2。
#[test]
fn a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_every_landed_write()
{
    let (output, failing_step, refusal, (landed_writes, landed_root_slot_writes)) =
        mount_failing_on_the_first_write_of_a_continuation_step(
            NarrowPoolCrashedBetweenDeletionBatches {
                unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
                overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
                batches_published_after_the_intent: 7,
            },
            |output| output.deletion_queue_continuation[1].root.checkpoint_txg,
        );
    assert_eq!(
        (
            output
                .space_admission
                .floor_raises()
                .iter()
                .map(|raised| raised.publishes.len())
                .collect::<Vec<_>>(),
            output.floor_raises_pushed_by_the_continuation.len(),
            output.deletion_queue_continuation.len(),
        ),
        (vec![3], 0, 9),
        "起点：准入推了一串三次空发布，续做 9 步、没推"
    );
    let (_, publishes_before_the_failing_step) =
        writes_and_publishes_of_the_mount_before(&output, failing_step);
    assert_eq!(
        publishes_before_the_failing_step, 7,
        "写行、暖机两次、推的三次、续做第一步"
    );
    assert_the_continuation_publish_error_accounts_for_every_landed_write(
        &refusal,
        landed_writes,
        landed_root_slot_writes,
        publishes_before_the_failing_step,
    );
}

/// 续做里推过抬 F 的起点（代码轮 V3 第二形）：同一种盘崩在第 0 批之后（只发了意图）；续做 16 步，第 14 步之后被落得下那一判拒、推一串两次空发布、
/// 回收之后接着走。推的那一串之后的第一步的第一个写报块设备错：错里 19 份（写行、暖机两次、续做 14 步、推的两次）= 落下的 16 个根槽写，
/// 写账合计 = 落下的写 − 取号两次。
/// 判别力：续做的写账去掉续做推的那几次空发布（攻方 MU2）时份数少 2；先写系统配置那一步不进错时合计少 2。
#[test]
fn a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write(
) {
    let (output, failing_step, refusal, (landed_writes, landed_root_slot_writes)) =
        mount_failing_on_the_first_write_of_a_continuation_step(
            NarrowPoolCrashedBetweenDeletionBatches {
                unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
                overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
                batches_published_after_the_intent: 0,
            },
            first_continuation_step_after_the_continuation_raised_the_floor,
        );
    assert_eq!(
        (
            output.space_admission.floor_raises().len(),
            output
                .floor_raises_pushed_by_the_continuation
                .iter()
                .map(|raised| raised.publishes.len())
                .collect::<Vec<_>>(),
            output.deletion_queue_continuation.len(),
        ),
        (0, vec![2], 16),
        "起点：准入没推，续做 16 步、中间推了一串两次空发布"
    );
    let (_, publishes_before_the_failing_step) =
        writes_and_publishes_of_the_mount_before(&output, failing_step);
    assert_eq!(
        publishes_before_the_failing_step, 19,
        "写行、暖机两次、续做 14 步、推的两次"
    );
    assert_the_continuation_publish_error_accounts_for_every_landed_write(
        &refusal,
        landed_writes,
        landed_root_slot_writes,
        publishes_before_the_failing_step,
    );
}

/// 准入与续做都推过抬 F 的起点（代码轮 V2 第三形）：单元区 192 槽、不先覆盖写、14 个单元、崩在第 1 批之后；准入推了一串两次空发布，
/// 续做 13 步、第 7 步之后推了一串两次空发布。续做推的那一串之后的第一步的第一个写报块设备错：错里 13 份 = 落下的 13 个根槽写，
/// 写账合计 = 落下的写 − 取号两次（两串先写系统配置那 4 个写在内）。
/// 判别力：先写系统配置那一步不进错时合计少 4。
#[test]
fn a_continuation_step_failing_after_both_the_admission_and_the_continuation_raised_the_floor_hands_back_every_landed_write(
) {
    let (output, failing_step, refusal, (landed_writes, landed_root_slot_writes)) =
        mount_failing_on_the_first_write_of_a_continuation_step(
            NarrowPoolCrashedBetweenDeletionBatches {
                unit_area_slots: 192,
                overwrites_before_the_unlink: 0,
                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES,
                batches_published_after_the_intent: 1,
            },
            first_continuation_step_after_the_continuation_raised_the_floor,
        );
    assert_eq!(
        (
            output
                .space_admission
                .floor_raises()
                .iter()
                .map(|raised| raised.publishes.len())
                .collect::<Vec<_>>(),
            output
                .floor_raises_pushed_by_the_continuation
                .iter()
                .map(|raised| raised.publishes.len())
                .collect::<Vec<_>>(),
            output.deletion_queue_continuation.len(),
        ),
        (vec![2], vec![2], 13),
        "起点：准入推了一串、续做里又推了一串，各两次空发布，续做 13 步"
    );
    let (_, publishes_before_the_failing_step) =
        writes_and_publishes_of_the_mount_before(&output, failing_step);
    assert_eq!(
        publishes_before_the_failing_step, 13,
        "写行、暖机、准入推的两次、续做 7 步、续做推的两次"
    );
    assert_the_continuation_publish_error_accounts_for_every_landed_write(
        &refusal,
        landed_writes,
        landed_root_slot_writes,
        publishes_before_the_failing_step,
    );
}

/// 续做推的第一串抬 F 之后的第一步（它的 txg 比那一串最后一次空发布大的第一步）。
fn first_continuation_step_after_the_continuation_raised_the_floor(
    output: &singlefs_core::mount::MountOutput,
) -> CheckpointTxg {
    let last_publish_of_the_raise = output.floor_raises_pushed_by_the_continuation[0]
        .publishes
        .last()
        .expect("推成的一串至少一次空发布")
        .root
        .checkpoint_txg;
    output
        .deletion_queue_continuation
        .iter()
        .map(|step| step.root.checkpoint_txg)
        .find(|step_txg| *step_txg > last_publish_of_the_raise)
        .expect("推的那一串之后续做还有一步")
}

/// 续做里为空间推的抬 F 自己报了不是空间不够的错（代码轮 V1，攻方 `m3-unlink-code-r2` H2 / G1）：192 槽那一种盘（准入推过一串、续做第 7 步之后
/// 又推一串）上，续做推的那一串第二次空发布的第一个写报块设备错。挂载不再原样交回抬 F 的错，交 `FloorRaiseFailedAfterTheMountsPublishes`：
/// `cause` 是那一串的 `RaiseFloorSequencePublishFailed`（第一次空发布已落盘），`writes_of_persisted_publishes` 是之前按 txg 落盘的 11 份
/// （写行、暖机、准入推的两次、续做 7 步），`floor_raises` 是准入推成的那一串；写账合计（各份 + 推成的那几串先写系统配置那一步 + `cause` 里那一串
/// 自己的账）= 落下的写 − 取号两次，已落盘份数（这里的 11 份 + `cause` 里的 1 份）= 落下的根槽写数。
/// 判别力：抬 F 的错照旧原样交回（改之前）时交的是 `RaiseFloorSequencePublishFailed`，第一项就红。
#[test]
fn a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_writes_of_the_mounts_publishes(
) {
    let shape = NarrowPoolCrashedBetweenDeletionBatches {
        unit_area_slots: 192,
        overwrites_before_the_unlink: 0,
        data_units_of_the_file: DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES,
        batches_published_after_the_intent: 1,
    };
    let (output, failing_publish, refusal, (landed_writes, landed_root_slot_writes)) =
        mount_failing_on_the_first_write_of_a_continuation_step(shape, |output| {
            output.floor_raises_pushed_by_the_continuation[0].publishes[1]
                .root
                .checkpoint_txg
        });
    let MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) = &refusal else {
        panic!("续做推的抬 F 报块设备错：交 FloorRaiseFailedAfterTheMountsPublishes：{refusal:?}");
    };
    let MountError::RaiseFloorSequencePublishFailed(failed_raise) = &failed.cause else {
        panic!("cause 是续做推的那一串的错原样：{:?}", failed.cause);
    };
    let (_, publishes_before_the_failing_publish) =
        writes_and_publishes_of_the_mount_before(&output, failing_publish);
    assert_eq!(
        (
            failed.writes_of_persisted_publishes.len(),
            failed_raise.writes_of_persisted_publishes.len(),
            failed
                .floor_raises
                .iter()
                .map(|raised| raised.publishes.len())
                .collect::<Vec<_>>(),
        ),
        (11, 1, vec![2]),
        "之前落盘的 11 份、那一串自己落了 1 次、之前推成的是准入那一串"
    );
    assert_eq!(
        (
            u64::try_from(
                failed.writes_of_persisted_publishes.len()
                    + failed_raise.writes_of_persisted_publishes.len()
            )
            .expect("份数"),
            failed.writes_of_persisted_publishes.len()
                + failed_raise.writes_of_persisted_publishes.len(),
        ),
        (
            landed_root_slot_writes,
            publishes_before_the_failing_publish
        ),
        "已落盘份数等于落下的根槽写数"
    );
    assert_eq!(
        write_calls_of(
            failed
                .writes_of_persisted_publishes
                .iter()
                .chain(
                    failed
                        .floor_raises
                        .iter()
                        .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
                )
                .chain(std::iter::once(
                    &failed_raise.writes_before_the_first_publish
                ))
                .chain(&failed_raise.writes_of_persisted_publishes)
                .chain(&failed_raise.writes_of_failed_publishes)
        ),
        landed_writes - ACQUISITION_WRITES,
        "写账合计等于落下的写减去取号那两次"
    );
}

/// 续做里 txg 为 `step_txg` 的那一步的写，按结构分：单元写、journal 记录写、根槽写（一次发布的写依次是单元写 → 屏障 → 记录 → 屏障 → 根槽 FUA →
/// 系统配置槽轮换，`transaction` 模块头）。
fn unit_record_and_root_slot_writes_of_the_continuation_step(
    output: &singlefs_core::mount::MountOutput,
    step_txg: CheckpointTxg,
) -> (u64, u64, u64) {
    let step = output
        .deletion_queue_continuation
        .iter()
        .find(|step| step.root.checkpoint_txg == step_txg)
        .expect("注错的那一步是续做里的一步");
    let structure_writes = |kind: WrittenStructureKind| step.writes.of(kind).write_calls;
    (
        step.writes.total().write_calls
            - structure_writes(WrittenStructureKind::JournalRecord)
            - structure_writes(WrittenStructureKind::RootSlot)
            - structure_writes(WrittenStructureKind::SystemConfigurationSlot),
        structure_writes(WrittenStructureKind::JournalRecord),
        structure_writes(WrittenStructureKind::RootSlot),
    )
}

/// 注错点在续做推的那一串之后的第一步、它第一个写之后的某一个写上（代码轮第三轮判决第三节第 6 条 N2：之前的用例只摆在发布的第一个写上）的那一形：
/// 起点同 [`a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write`]（窄盘崩在第 0 批之后，续做 16 步、
/// 第 14 步之后推一串两次空发布）。交回那一步之前落盘的发布数（19 份）、注错那一步写了几个单元、注错的错与真落下的写和根槽写。
fn mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
    ordinal_within_the_step: impl Fn(u64, u64) -> u64,
) -> (usize, u64, MountError, (u64, u64)) {
    let (output, failing_step, refusal, landed) =
        mount_failing_on_a_write_of_a_continuation_publish(
            NarrowPoolCrashedBetweenDeletionBatches {
                unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
                overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
                batches_published_after_the_intent: 0,
            },
            first_continuation_step_after_the_continuation_raised_the_floor,
            |output, step_txg| {
                let (unit_writes, record_writes, _) =
                    unit_record_and_root_slot_writes_of_the_continuation_step(output, step_txg);
                ordinal_within_the_step(unit_writes, record_writes)
            },
        );
    let (_, publishes_before_the_failing_step) =
        writes_and_publishes_of_the_mount_before(&output, failing_step);
    let (unit_writes, _, _) =
        unit_record_and_root_slot_writes_of_the_continuation_step(&output, failing_step);
    (
        publishes_before_the_failing_step,
        unit_writes,
        refusal,
        landed,
    )
}

/// 续做那一步的第二个单元写报块设备错（N2，注错点在发布第一个写之后）：第一个单元写已落盘、属于失败的那一次发布。
/// 错是 `Publish`、块设备错；已落盘份数 19 = 落下的根槽写数；写账合计（含失败那一次已记的那 1 个单元写）= 落下的写 − 取号两次。
/// 判别力：续做那一格交回的失败账照旧是空的（不带失败那一次已落下的写）时合计少 1，那一项红。
#[test]
fn a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_write() {
    let (
        publishes_before_the_failing_step,
        unit_writes,
        refusal,
        (landed_writes, landed_root_slot_writes),
    ) = mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
        |_, _| 2,
    );
    assert!(unit_writes >= 2, "那一步至少写两个单元：{unit_writes}");
    assert_eq!(
        publishes_before_the_failing_step, 19,
        "写行、暖机两次、续做 14 步、推的两次"
    );
    assert_the_continuation_publish_error_accounts_for_every_landed_write(
        &refusal,
        landed_writes,
        landed_root_slot_writes,
        publishes_before_the_failing_step,
    );
}

/// 续做那一步的 journal 记录写报块设备错（N2）：那一步的单元写全落盘、属于失败的那一次发布，记录与根都没落。
/// 判法同上一条：份数 19 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
/// 判别力：失败账照旧是空的时合计少那一步的单元写数，那一项红。
#[test]
fn a_continuation_step_failing_on_its_journal_record_write_hands_back_every_landed_write() {
    let (publishes_before_the_failing_step, _, refusal, (landed_writes, landed_root_slot_writes)) =
        mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
            |unit_writes, _| unit_writes + 1,
        );
    assert_the_continuation_publish_error_accounts_for_every_landed_write(
        &refusal,
        landed_writes,
        landed_root_slot_writes,
        publishes_before_the_failing_step,
    );
}

/// 续做那一步的根槽写报块设备错（N2）：那一步的单元写与 journal 记录写落盘，根没落（只读恢复会靠那条记录多走一次，见
/// `unlink_mount_continuation_and_the_emptied_version.rs` 设备错那一条的说明；这里只比根槽写，不比只读挂载走到的版）。
/// 判法同上：份数 19 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
/// 判别力：失败账照旧是空的时合计少那一步的单元写与记录写，那一项红。
#[test]
fn a_continuation_step_failing_on_its_root_slot_write_hands_back_every_landed_write() {
    let (publishes_before_the_failing_step, _, refusal, (landed_writes, landed_root_slot_writes)) =
        mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
            |unit_writes, record_writes| unit_writes + record_writes + 1,
        );
    assert_the_continuation_publish_error_accounts_for_every_landed_write(
        &refusal,
        landed_writes,
        landed_root_slot_writes,
        publishes_before_the_failing_step,
    );
}

/// 续做里为空间推的抬 F 先写系统配置那一步报块设备错（N2，判决第三节第 6 条「续做那一处抬 F 先写系统配置报错」）：192 槽那一种盘
/// （准入推过一串、续做第 7 步之后又推一串，同 [`a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_writes_of_the_mounts_publishes`]），
/// 续做推的那一串先写系统配置的第二个写（盘 1 那一个）报错：第一个已落盘、一条带新 F 的根都没发。挂载交 `FloorRaiseFailedAfterTheMountsPublishes`，
/// `cause` 是那一串的 `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`（它的写账是已落下的那 1 个系统配置写）；已落盘份数 11 = 落下的根槽写数；
/// 写账合计（各份 + 推成的那一串先写系统配置那一步 + `cause` 里那一步的写）= 落下的写 − 取号两次。
/// 判别力：先写系统配置那一步报错时不带已落下的写（写账交空）时合计少 1，那一项红。
#[test]
fn a_floor_raise_in_the_continuation_failing_on_its_system_configuration_write_hands_back_every_landed_write(
) {
    let (output, failing_publish, refusal, (landed_writes, landed_root_slot_writes)) =
        mount_failing_on_a_write_of_a_continuation_publish(
            NarrowPoolCrashedBetweenDeletionBatches {
                unit_area_slots: 192,
                overwrites_before_the_unlink: 0,
                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES,
                batches_published_after_the_intent: 1,
            },
            |output| {
                output.floor_raises_pushed_by_the_continuation[0].publishes[0]
                    .root
                    .checkpoint_txg
            },
            |_, _| 2,
        );
    let MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) = &refusal else {
        panic!("续做推的抬 F 先写系统配置那一步报块设备错：交 FloorRaiseFailedAfterTheMountsPublishes：{refusal:?}");
    };
    let MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed_step) =
        &failed.cause
    else {
        panic!(
            "cause 是续做推的那一串先写系统配置那一步的错原样：{:?}",
            failed.cause
        );
    };
    let (_, publishes_before_the_failing_raise) =
        writes_and_publishes_of_the_mount_before(&output, failing_publish);
    assert_eq!(
        (
            failed.writes_of_persisted_publishes.len(),
            failed
                .floor_raises
                .iter()
                .map(|raised| raised.publishes.len())
                .collect::<Vec<_>>(),
            failed_step.writes.total().write_calls,
            publishes_before_the_failing_raise,
        ),
        (11, vec![2], 1, 11),
        "之前落盘的 11 份（写行、暖机、准入推的两次、续做 7 步）、之前推成的是准入那一串、先写系统配置那一步落了 1 个写"
    );
    assert_eq!(
        u64::try_from(failed.writes_of_persisted_publishes.len()).expect("份数"),
        landed_root_slot_writes,
        "已落盘份数等于落下的根槽写数"
    );
    assert_eq!(
        write_calls_of(
            failed
                .writes_of_persisted_publishes
                .iter()
                .chain(
                    failed
                        .floor_raises
                        .iter()
                        .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
                )
                .chain(std::iter::once(&failed_step.writes))
        ),
        landed_writes - ACQUISITION_WRITES,
        "写账合计等于落下的写减去取号那两次"
    );
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
#[test]
fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(
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
    assert_eq!(
        checker_violations_on(&crashed),
        Vec::<String>::new(),
        "掉电那一刻的盘面上池级 checker 全绿"
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
    assert_eq!(
        checker_violations_on(&image_after_the_remount),
        Vec::<String>::new(),
        "再挂续做之后池级 checker 全绿"
    );
}

/// 续做推满仍放不下的那块窄盘上，卸载之前现行那一版的 txg，与那时每块盘上最新的根的 txg 取小（第一版区域归属 0 / 1 / 0：txg 24 的根在盘 0，
/// 盘 1 上最新的是 txg 22；这块盘的根从写行起连号写下来、没有回退，没有被抛弃的根）。
const CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg = CheckpointTxg(24);
const NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg =
    CheckpointTxg(22);

/// 甲 GA 在续做推满仍放不下的那块窄盘上的结局（用户 2026-10-03 定，`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`）：卸载那一串
/// 第一次空发布落在这一串回收出来的槽上的单元至少一个（槽被续做吃光，卸载靠当场放开的那一截做成），那几个槽在卸载之前的分配记录里的释放代都不高于
/// 每块盘上最新的根的 txg 取小；回收出来的槽里另有释放代高于它的（那几条根还引用着，扣到生效），第一次空发布没落上去。
fn assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device(
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
#[test]
fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor(
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
    assert_eq!(
        checker_violations_on(&survivor),
        checker_violations_on(&survivor_with_only_the_system_configuration_written),
        "池级 checker 在单盘镜像上卸载之前就饱和（条名与只写了系统配置那一刀逐条相同），不靠它分辨"
    );
}

/// 同 [`mount_after_the_crash_with_batches_of_one`]，盘由调用方给。
fn mount_after_the_crash_on(
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
fn kind_of_the_refusal_that_pushed(refusal: &PublishError) -> &'static str {
    if matches!(refusal, PublishError::SpaceAdmissionRefused(_)) {
        "SpaceAdmissionRefused"
    } else if matches!(refusal, PublishError::PlacementRefused { .. }) {
        "PlacementRefused"
    } else {
        panic!("会话只因空间不够推抬 F：{refusal:?}")
    }
}

/// 两块单元区 384 槽的小盘，第一个文件之后崩了再挂、覆盖写 `overwrites` 次，再把每块盘没挡的槽标成已分配、只留槽号最低的 `unblocked_slots_left` 槽
/// （两块盘的空闲图此刻相同，留同样的槽号；位图上占着、不写记录，只供用例造盘面）。
fn pool_ready_to_unlink_with_few_unblocked_slots(
    overwrites: usize,
    unblocked_slots_left: usize,
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for overwrite_index in 1..=overwrites {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
    }
    let session = pool.session.as_mut().expect("这次挂载的会话");
    let slots_left: Vec<u64> = {
        let device_map = &session.allocator.devices[0];
        let first_slot = device_map.unit_area_start().slot().0;
        (first_slot..first_slot + device_map.unit_area_slots())
            .filter(|slot| device_map.is_free(SlotNumber(*slot)))
            .take(unblocked_slots_left)
            .collect()
    };
    for device_map in &mut session.allocator.devices {
        let first_slot = device_map.unit_area_start().slot().0;
        // 迭代上界是单元区槽数，跨轮只改这一槽的分配位。
        for slot in first_slot..first_slot + device_map.unit_area_slots() {
            if !slots_left.contains(&slot) && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    pool
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
#[test]
fn after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes() {
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
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "写回之后池级 checker 0 违例"
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
#[test]
fn crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck(
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
        assert_eq!(
            checker_violations_on(&pool.image()),
            Vec::<String>::new(),
            "第 {mount_index} 次挂载之后池级 checker 0 违例"
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
#[test]
fn mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount(
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
    assert_eq!(
        checker_violations_on(&pool.image()),
        Vec::<String>::new(),
        "卸载之后再挂的镜像池级 checker 0 违例"
    );
}

/// 上一版树表 0 条（只做过 mkfs、还没有文件）的可写挂载准入不够：抬 F 要带文件的现行版本、这一版上没有可退的带文件版本，推不出空间，
/// 照旧在取号之前拒（`MountError::SpaceAdmissionRefusedBeforeAcquisition`），一个写都没发、两块盘逐字节不变（两块盘系统配置里的实例代号还是 0）。
/// 这一格：两块单元区只有 16 槽的盘上 mkfs（实例表一片 2 槽、树表 1 槽），挂载判实例切换的预留（每块盘 4 × (2 + 3 × c_max) 槽）拿不到。
/// 单元区起点随环长走（实审 A3b）：环取 1 MiB，单元区从环末尾的下一个槽 1088（= 17 × 64）起，盘宽 1088 + 16 = 1104 槽；
/// 小盘那一档的 6 MiB 环在这么宽的盘上过不了「环不超过设备容量的四分之一」。
/// 判别力：树表 0 条这一格也照带文件的那一格写行之后推，挂载就取了号、写了盘。
#[test]
fn writable_mount_on_a_version_without_file_short_of_its_reserve_is_refused_before_acquisition_with_the_disk_unchanged(
) {
    let mut parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    parameters.geometry.journal_ring_bytes = 1 << 20;
    let unit_area_start =
        UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes)
            .expect("1 MiB 的环末尾的下一个槽 1088 = 17 × 64，落在聚簇段边界上");
    assert_eq!(unit_area_start.slot(), SlotNumber(1088));
    let device_bytes = (unit_area_start.slot().0 + 16) * singlefs_format::SLOT_BYTES;
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("单元区 16 槽的盘上 mkfs 做得成");
    let operations_after_mkfs = stream.operation_count();
    let image_after_mkfs: Vec<SparseDevice> = devices
        .iter()
        .map(|(_, device)| device.sparse_image().clone())
        .collect();
    let refused = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let Err(MountError::SpaceAdmissionRefusedBeforeAcquisition {
        instance_to_acquire,
        refusal,
    }) = refused
    else {
        panic!(
            "树表 0 条的一版上准入不够在取号之前拒：{:?}",
            refused.as_ref().err()
        );
    };
    assert_eq!(instance_to_acquire, InstanceGeneration(1));
    assert_eq!(refusal.short_devices.len(), 2, "两块盘都短");
    assert_eq!(
        stream.operation_count(),
        operations_after_mkfs,
        "一个写、一道屏障都没发"
    );
    assert!(
        devices
            .iter()
            .zip(&image_after_mkfs)
            .all(|((_, device), before)| device.sparse_image() == before),
        "两块盘逐字节不变"
    );
}

/// 实例表行数逼近 369 的整数倍（攻方「挂载路径的 rows0」那一形，D28（挂载期承诺量） 已定项 3「发布路径用 rows0 + 1」）：两块单元区 384 槽的小盘，
/// 第一个文件之后连着崩了再挂 366 次（mkfs 那个进程的实例 1 不写行，每次挂载写一行），把这次挂载的 rows0 推到 366（⌈(366 + 3) ÷ 369⌉ = 1 片），下一次挂载写一行、rows0 = 367，
/// 链要两片、切换预留每块盘多 8 槽。在这次挂载里一直覆盖写到会话报空间不够（至多 200 次）：发布路径的读数按下一次挂载的 367 行留份额，
/// 每一次覆盖写做成之后，在盘面的拷贝上只崩了再挂，取号之前就够（一次都不用推）。
/// 判别力：发布路径的切换预留照旧按这次挂载的 366 行算时，会话放行得更靠满，有一次覆盖写之后崩了再挂取号之前不够。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn at_366_instance_rows_every_admitted_overwrite_leaves_the_next_mounts_row_so_a_crash_remount_is_admitted_before_acquisition(
) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    for mount_index in 1..=366 {
        pool.crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
    }
    assert_eq!(
        pool.session()
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        366,
        "这次挂载的 rows0"
    );
    let mut admitted_overwrites = 0;
    for overwrite_index in 1..=200 {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(_) => admitted_overwrites += 1,
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) => break,
            Err(refusal) => {
                panic!("第 {overwrite_index} 次覆盖写只会因为空间不够被拒：{refusal:?}")
            }
        }
        let remount = space_admission_of_a_crash_remount_on_a_copy(&pool);
        assert!(
            matches!(remount, Ok(MountSpaceAdmission::AdmittedBeforeAcquisition)),
            "第 {overwrite_index} 次覆盖写做成之后崩了再挂，取号之前就够：{remount:?}"
        );
    }
    assert!(
        admitted_overwrites >= 1,
        "这次挂载里至少做成一次覆盖写（不然这一格什么都没判）"
    );
}

/// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）用的池：mkfs 同一个进程里取号、暖机、第一个文件，
/// 盘外面包一层注入故障的（计划先不开）。根环这时：(区域 0, 槽 0) 是 mkfs 的第 0 代根（txg 0，区域 1、2 的槽 0 已被暖机那两条盖掉），
/// 第一个文件那条根在 (区域 0, 槽 1)。有效根里非空的不足 4 个，上限取最旧的有效根 = 0。
fn pool_for_the_ceiling_reading() -> (
    PoolUnderTest<FaultInjectingBlockDevice<SparseBlockDevice>>,
    SharedFaultPlan,
) {
    let plan = SharedFaultPlan::unarmed(HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry());
    let plan_for_the_devices = plan.clone();
    let pool = PoolUnderTest::start_after_the_first_file(
        HistoryDeviceWidth::UnitAreaOf384Slots,
        move |identity, device| {
            FaultInjectingBlockDevice::new(identity, device, plan_for_the_devices.clone())
        },
    );
    (pool, plan)
}

/// 根环 (区域 0, 槽 0) 在盘 0 上的偏移（第一版区域归属 0 / 1 / 0）。
fn offset_of_the_genesis_root_slot_on_device_zero(
    parameters: &MakeFilesystemParameters,
) -> DeviceOffsetInBytes {
    slot_offset(
        RootRingSlot { region: 0, slot: 0 },
        parameters.geometry.fixed_structure_slot_spacing,
    )
}

/// 只在盘 0 那个根槽的读上注入 `fault`，从第 `first_occurrence` 次命中起（`every_time_from_then_on` 为真时之后每次都注入）。
fn schedule_on_the_genesis_root_slot(
    parameters: &MakeFilesystemParameters,
    occurrence: FaultOccurrence,
) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(0)),
        placement: FaultPlacement::OffsetExactly(offset_of_the_genesis_root_slot_on_device_zero(
            parameters,
        )),
        counting: FaultCounting::AcrossThePool,
        occurrence,
    }
}

/// 算上限的输入：系统配置、现行那一版的实例表、这个进程知道住着根的根环槽。
fn ceiling_of(
    pool: &PoolUnderTest<FaultInjectingBlockDevice<SparseBlockDevice>>,
) -> Result<CheckpointTxg, MountError> {
    let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置读得出");
    let table = instance_table_chain_of_root(&pool.devices, &pool.current_file_version().root)
        .expect("实例表读得出")
        .records;
    rollback_floor_ceiling(
        &pool.devices,
        &system_configuration,
        pool.rollback_floor(),
        &table,
        &ring_slots_known_to_hold_a_root_by(&pool.session().allocator),
    )
}

/// 根槽这一次读坏、重读读出来（C562（抬 F 算上限时根槽读坏当没有根） 的正方向）：(区域 0, 槽 0) 是 mkfs 写下、这个进程的根环表里记着的槽，
/// 算上限那一次读它报一次读错，重读读得出，上限照常是 0（最旧的有效根就是它）。注入正好打中一次。
/// 判别力：读坏就当没有根（改之前的做法）时，最旧的有效根成了暖机那条 txg 1，上限算成 1，F 会被抬过条款上限（实四丙查出的那一格）。
#[test]
fn known_root_slot_that_reads_bad_once_is_reread_and_the_ceiling_is_unchanged() {
    let (pool, plan) = pool_for_the_ceiling_reading();
    assert_eq!(
        ceiling_of(&pool).expect("不注入时算得出上限"),
        CheckpointTxg(0)
    );
    assert!(
        ring_slots_known_to_hold_a_root_by(&pool.session().allocator)
            .contains(&RootRingSlot { region: 0, slot: 0 }),
        "mkfs 写下的第 0 代根那一槽，这个进程知道住着根"
    );
    plan.arm(schedule_on_the_genesis_root_slot(
        &pool.parameters,
        FaultOccurrence::TheNthMatchingCall(1),
    ));
    let ceiling = ceiling_of(&pool);
    plan.disarm();
    assert_eq!(plan.fired_count(), 1, "注入正好打中算上限那一读");
    assert_eq!(
        ceiling.expect("重读读得出，上限照常算"),
        CheckpointTxg(0),
        "重读读出第 0 代根，上限还是 0"
    );
}

/// 根槽这一次读坏、重读仍坏（C562 的反方向）：同一个池上抬 F 到 1，(区域 0, 槽 0) 的每一次读都报错：这个进程知道那里住着根，
/// 重读一次仍坏就拒这次抬 F，报 `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，不按有根或没根猜：
/// 一个写都没发（两块盘逐字节不变）、分配器与抬 F 之前逐项相同、现行那一版不动。
/// 判别力：读坏就当没有根时，上限算成 1、F 抬到 1、推了空发布——抬过了条款上限 0（checker 的 I-7.9 判红那一格）。
#[test]
fn known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any_write() {
    let (mut pool, plan) = pool_for_the_ceiling_reading();
    let image_before = pool.image();
    let mut session = pool.session.take().expect("mkfs 那条会话");
    let allocator_before = format!("{:?}", session.allocator);
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("第一个文件之后现行那一版带文件");
    };
    let current_before = current.clone();
    plan.arm(schedule_on_the_genesis_root_slot(
        &pool.parameters,
        FaultOccurrence::EVERY_MATCHING_CALL,
    ));
    let raised = raise_rollback_floor(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        CheckpointTxg(1),
        ShadowLedger::On,
    );
    plan.disarm();
    let Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
        ring_slot,
        first_reading,
        reread,
    }) = raised
    else {
        panic!(
            "重读仍坏就拒这次抬 F：{:?}",
            raised.as_ref().map(|raised| raised.ceiling).err()
        );
    };
    assert_eq!(
        (ring_slot, first_reading, reread),
        (
            RootRingSlot { region: 0, slot: 0 },
            BadRootRingSlotReading::Unreadable,
            BadRootRingSlotReading::Unreadable
        )
    );
    assert!(pool.image() == image_before, "两块盘逐字节不变");
    assert!(
        format!("{:?}", session.allocator) == allocator_before,
        "分配器与抬 F 之前逐项相同"
    );
    assert!(*current == current_before, "现行那一版不动");
}

/// 挂载那一刻就读不出、这个进程之后没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它
/// （D16（发布语义） 已定项 1「抬 F 的上限」那一行，用户 2026-10-04 定 RS3；同日 Q3 重定成读不出与自证不过两类都并，这一条是读不出那一类）：
/// 同一个池进程退出，(区域 0, 槽 0)
/// 从这时起每一次读都报错，再可写挂载——挂载读根环时它就读不出，这个进程知道住着根的槽里没有它，根环表把它记成窗口里读不出的槽、
/// 按它能住的最大 txg 记：那一槽在一圈窗口里只能住 txg 0（mkfs 那条根；下一次轮到它是 txg 3S，还没写到）。之后抬 F 到上限：
/// 读得出的根算出的上限是 1（最旧的有效根是暖机那条 txg 1），并进那一槽的 0 之后上限是 0、不高于现行的 F 0——
/// 报 `FloorAlreadyAtTheCeiling { floor: 0, ceiling: 0 }`，一个字节都不写；也不因为这一槽重读仍坏就拒（它不是这个进程知道住着根的槽）。
/// 判别力：不并这一槽时上限 1、F 抬到 1；挂载那一刻读不出的槽也重读、仍坏就拒时报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
#[test]
fn root_slot_already_unreadable_at_mount_is_taken_as_a_possible_root_and_holds_the_admission_ceiling_at_its_txg(
) {
    let (mut pool, plan) = pool_for_the_ceiling_reading();
    plan.arm(schedule_on_the_genesis_root_slot(
        &pool.parameters,
        FaultOccurrence::EVERY_MATCHING_CALL,
    ));
    pool.crash_and_mount_writable()
        .expect("那一槽读不出照样挂得上");
    assert!(
        !ring_slots_known_to_hold_a_root_by(&pool.session().allocator)
            .contains(&RootRingSlot { region: 0, slot: 0 }),
        "挂载那一刻读不出的槽不在这个进程知道住着根的槽里"
    );
    let image_before = pool.image();
    let mut session = pool.session.take().expect("挂载的会话");
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("挂载之后现行那一版带文件");
    };
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        ShadowLedger::On,
    );
    plan.disarm();
    let Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) = raised else {
        panic!("挂载时就读不出的槽按它能住的 txg 0 封顶上限、F 不越过它：{raised:?}");
    };
    assert_eq!(
        (floor, ceiling),
        (CheckpointTxg(0), CheckpointTxg(0)),
        "上限取读得出的根算出的 1 与那一槽能住的 0 里小的那个"
    );
    assert!(pool.image() == image_before, "没有可抬的：两块盘逐字节不变");
}

/// 实四丙的构造做成用例（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实四丙交回」那一行）：故障注入的种子基 + 339 那段历史
/// （宽泛的比重、两块 4 GiB 的盘、每一步之后跑池级 checker），第 1 步抬 F 到 1、条款上限 0（非空状态不足 4 个，取最旧的有效根 (0, 0)）。
/// 只在这一步里，对盘 0 上 (区域 0, 槽 0) 那个根槽第 2 次读注一次读错——第 1 次是现算 F 生效值那一遍读根环，第 2 次才是算上限那一读。
/// 这一槽这个进程知道住着根：重读读得出，上限照常是 0，这次抬 F 被上限拒（`RollbackFloorAboveCeiling`，与模型同一个结论），
/// 历史跑完、没有新发现（I-7.9 不红）。注入正好打中一次。
/// 判别力：读坏就当没有根时，实现接受 F = 1，模型判「该拒、实现做成了」，同一步 checker 判 I-7.9 红。
#[test]
fn the_seeded_history_whose_ceiling_read_of_the_genesis_root_slot_fails_once_refuses_the_raise_above_the_ceiling(
) {
    const SEED_OFFSET_OF_THE_CONSTRUCTION: u64 = 339;
    const STEP_WHOSE_READ_FAILS: usize = 1;
    const OPERATIONS: usize = 2;
    let history = generate_history_with_weights(
        HistorySeed(
            SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(SEED_OFFSET_OF_THE_CONSTRUCTION),
        ),
        OPERATIONS,
        &GenerationWeights::BROAD,
    );
    assert!(
        matches!(
            history.operations[STEP_WHOSE_READ_FAILS],
            HistoryOperation::RaiseRollbackFloor(_)
        ),
        "这段历史的第 1 步是抬 F：{:?}",
        history.operations
    );
    let execution = HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let plan = SharedFaultPlan::unarmed(execution.device_width.fixed_geometry());
    let parameters = execution.device_width.parameters();
    let schedule =
        schedule_on_the_genesis_root_slot(&parameters, FaultOccurrence::TheNthMatchingCall(2));
    let mut outcome_of_the_step = None;
    let run = execute_history_with_faults(
        &history,
        execution,
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if observation.position == StepPosition::Operation(STEP_WHOSE_READ_FAILS - 1) {
                plan.arm(schedule);
            }
            if observation.position == StepPosition::Operation(STEP_WHOSE_READ_FAILS) {
                plan.disarm();
                outcome_of_the_step = observation.outcome.cloned();
            }
        },
    );
    assert_eq!(plan.fired_count(), 1, "注入正好打中一次");
    assert_eq!(run.ending, HistoryEnding::Completed, "历史跑完、没有新发现");
    assert_eq!(
        outcome_of_the_step,
        Some(StepOutcome::Refused {
            member: "MountError::RollbackFloorAboveCeiling".to_string()
        }),
        "重读读得出，上限照常是 0，抬 F 到 1 被上限拒"
    );
}
