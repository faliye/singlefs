//! 里程碑「覆盖写、释放、回退与复用」收尾实现批次实五：准入不够时先推抬 F 的空发布再判（D16（发布语义） 已定项 1「准入」那一行，
//! C283（准入失败时不先推发布就报 ENOSPC）），挂载那一处写行之后也推，推满仍不够怎么收尾（C565（挂载处推满仍不够怎么收尾没定），
//! 实现员提的那一种），发布路径的切换预留按下一次挂载的 rows0（D28（挂载期承诺量） 已定项 3「发布路径用 rows0 + 1」），
//! 与算抬 F 上限时根槽这一次读坏（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，C562（抬 F 算上限时根槽读坏当没有根））。
//!
//! 用例的形状照安全设计轮第三轮攻方（`research/prompts/m2-safety-r3-opus-output.md`）：会话里数据单元落不下（准入先拒之后是准入拒，
//! 关掉准入时是落点被拒，C545）、删掉之后同样大小的写回、
//! 只崩了再挂的窄池、实例表行数逼近 369 的整数倍；445 那一格照实四丙的构造（种子基 + 339 第 1 步）。
//! 盘都是内存里的稀疏盘，两块、单元区 240 / 256 / 384 槽（`HistoryDeviceWidth` 那几档），一个进程里 mkfs、取号、暖机、第一个文件起步。
//! 崩在准入推的那一串空发布中间那一形在 `crash_enumeration_floor_raise_pushed_by_admission.rs`（崩溃点重放的枚举，单独一个测试二进制）。
mod common_admission;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/admission_raises_the_floor_before_refusing.rs"]
mod scenarios_of_admission_raises_the_floor_before_refusing;

use common_admission::{
    plain_devices_on, space_admission_of_a_crash_remount_on_a_copy, start_plain,
    DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_admission_raises_the_floor_before_refusing::{
    deletion_switches_with_batches_of_one, fault_injecting_recorded_devices_on,
    kind_of_the_refusal_that_pushed, landed_writes_and_root_slot_writes,
    mount_after_the_crash_with_batches_of_one, pool_crashed_between_deletion_batches,
    pool_crashed_between_deletion_batches_on_narrow_devices, write_failing_from, FailingWrites,
    NarrowPoolCrashedBetweenDeletionBatches, DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
    OVERWRITES_BEFORE_THE_CONTINUATION, UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{PlacementRefusal, UnitAreaStart};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable_with_space_admission,
    mount_writable_with_test_only_switches_and_deletion_switches, raise_rollback_floor,
    raise_rollback_floor_to_the_admission_ceiling, ring_slots_known_to_hold_a_root_by,
    rollback_floor_ceiling, DeletionQueueContinuationOutcome, FloorRaiseStop, MountError,
    MountSpaceAdmission, RaiseToTheAdmissionCeiling, ShadowLedger,
};
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, BadRootRingSlotReading,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::transaction::{PoolVersion, PublishError, TransactionUnit, FIRST_INODE_NUMBER};
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
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
use singlefs_harness::model_comparison::refusal_reason_of_mount_error;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

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

/// 单元区 192 槽那几条（准入与续做都推过抬 F）的文件单元数：它们的造形没随第 50 批改，仍是 14 个单元。
const DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES: usize = 14;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue,
    );
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims,
    );
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

/// 取号那四次系统配置槽写（取号写两轮，每轮每块盘一次）：不是发布，挂载交回的错与输出里的写账都不含它们（`PublishSequenceFailed` 的契约）。
const ACQUISITION_WRITES: u64 = 4;

/// 一次不注错的可写挂载里，txg 低于 `txg` 的那几次发布（写行、暖机、准入推的与续做推的空发布、续做的各步）与第一次发布低于它的
/// 那几串抬 F 先写系统配置那一步，一共交给设备几个写（连取号那四次）；交回 (写数, 发布数)。注错点就按它摆在 txg 那一次发布的第一个写上。
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
/// = 落下的写 − 取号四次（代码轮 V2 / V3 的判法）。
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
        "写账合计等于落下的写减去取号那四次（推的每一串先写系统配置那一步在 writes_before_the_first_publish 里）"
    );
}

/// 准入推过抬 F 的起点（代码轮 V3 第一形）：单元区 292 槽、覆盖写 1 次、16 个单元、崩在第 7 批之后；取号之前不够、写行与暖机之后推了一串
/// （三次空发布），续做 9 步、没再推。续做第二步的第一个写报块设备错：错里 7 份（写行、暖机两次、推的三次、续做第一步）= 落下的 7 个根槽写，
/// 写账合计 = 落下的写 − 取号四次（推的那一串先写系统配置那 2 个写在内）。
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
/// 写账合计 = 落下的写 − 取号四次。
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
/// 写账合计 = 落下的写 − 取号四次（两串先写系统配置那 4 个写在内）。
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
/// 自己的账）= 落下的写 − 取号四次，已落盘份数（这里的 11 份 + `cause` 里的 1 份）= 落下的根槽写数。
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
        "写账合计等于落下的写减去取号那四次"
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
/// 错是 `Publish`、块设备错；已落盘份数 19 = 落下的根槽写数；写账合计（含失败那一次已记的那 1 个单元写）= 落下的写 − 取号四次。
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
/// 判法同上一条：份数 19 = 落下的根槽写数，写账合计 = 落下的写 − 取号四次。
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
/// 判法同上：份数 19 = 落下的根槽写数，写账合计 = 落下的写 − 取号四次。
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
/// 写账合计（各份 + 推成的那一串先写系统配置那一步 + `cause` 里那一步的写）= 落下的写 − 取号四次。
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
        "写账合计等于落下的写减去取号那四次"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor,
    );
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_admission_raises_the_floor_before_refusing::mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_admission_raises_the_floor_before_refusing::mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount,
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
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
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
