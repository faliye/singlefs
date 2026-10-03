//! 挂载透明的逐字段判（`singlefs_harness::mount_transparency`，r5 判决第二节 U5、U6 两行）在几个小池上：每一样一条阳性（合法挂载判绿）、
//! 一条阴性（把 R2 的那一样改坏判红，别的仍绿）。小池是单元区 384 槽（抬 F 那一条取 256 槽）的两块内存盘，历史由随机历史的执行器照抄
//! 一段写死的操作跑出来；「崩溃镜像」取那段历史停下时的整份盘面（或一次发布扣下根槽写之后的盘面），它就是一个不带故障的崩溃状态。
//! 挂载续做过的两形（用户 2026-10-03 定「续做算挂载的一部分」）：崩在 unlink 意图之后重挂；窄盘上续做里被空间拒、推抬 F 再走
//! （盘面照 `admission_raises_the_floor_before_refusing.rs` 那一条的造法，搭建经 `tests/common_admission`）。

mod common_admission;

use common_admission::{
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_space_admission,
    mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountOutput, Mounted,
    ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::root_record::UnmountMarker;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_unlink_intent, FirstFile, PoolVersion,
    PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
    WrittenContents,
};
use singlefs_harness::mount_transparency::{
    judge_mount_transparency, read_recovery_fields, ComparedVersion, FieldNotJudged, FieldOutcome,
    FloorRaiseInTheMountWrites, FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash,
    FloorRaisesInTheMountWrites, FloorRaisesOfTheMount, MountTransparency, MountTransparencyAction,
    MountTransparencyField, PosixContentRead, PublishInTheMountWrites, RecoveryFieldsRead,
    UserVisibleTreeRootPointersOfTheContinuation, VersionFieldsRead,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 挂载之后那一次发布写的内容（装得进一个数据单元）。
fn content_published_after_the_mount() -> Vec<u8> {
    (0..321u32)
        .map(|index| u8::try_from(index % 251).expect("小于 251") ^ 0x5a)
        .collect()
}

/// 照抄一段写死的历史跑到底，交回停下时的整份盘面。
fn pool_after_the_history(
    width: HistoryDeviceWidth,
    starting_point: HistoryStartingPoint,
    operations: Vec<HistoryOperation>,
    space_admission: SpaceAdmission,
) -> MemoryPool {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point,
        operations,
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission,
        },
        &stream,
        &mut |_observation| {},
    );
    assert_eq!(
        run.outcomes.len(),
        history.operations.len(),
        "写死的那段历史每一步都要跑完：{:?}",
        run.ending
    );
    let mut pool = MemoryPool::with_devices(&DEVICES, width.device_bytes());
    pool.apply(&stream.retained_operations());
    pool
}

fn devices_of(
    pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    pool.devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                pool.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect()
}

fn pool_of(
    devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.image))
            .collect(),
        device_size_in_bytes,
    }
}

fn mount(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
) -> Mounted {
    mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)
        .expect("小池上不带故障的盘面，可写挂载该成")
}

/// 挂载之后接着发一次布：现行那一版带文件时覆盖写，树表 0 条时发第一个文件（与崩溃注入同一种调法）。
fn publish_once_after_the_mount(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)],
    mounted: Mounted,
) {
    let content = content_published_after_the_mount();
    let file = FirstFile {
        content: &content,
        write_time_seconds: 1_000,
    };
    let mut allocator = mounted.allocator;
    let mut writer = PoolWriter::new(parameters, devices);
    match &mounted.current {
        PoolVersion::WithFile(current) => publish_overwrite(
            &mut writer,
            &mut allocator,
            current,
            file,
            mounted.output.instance,
        )
        .map(|_| ()),
        // 这几条流里没有 unlink：挂载之后的现行那一版不会是有树而文件集合为空的一版（D8（核心索引结构） 已定项 16 第 5 条）。
        PoolVersion::WithTreesAndNoFile(current) => panic!(
            "挂载之后的现行那一版是有树而文件集合为空的一版（txg {:?}）：这几条流不发 unlink",
            current.root.checkpoint_txg
        ),
        PoolVersion::WithoutFile(current) => publish_first_file(
            &mut writer,
            &mut allocator,
            &current.root,
            file,
            mounted.output.instance,
            &current.record_bytes,
        )
        .map(|_| ()),
    }
    .expect("挂载之后那一次发布该成");
}

/// 一次判下来要的四样：R1、R2、动作与挂载续做那几步的两根指针。
struct JudgedScenario {
    before_the_mount: RecoveryFieldsRead,
    after_the_action: RecoveryFieldsRead,
    action: MountTransparencyAction,
    continuation: UserVisibleTreeRootPointersOfTheContinuation,
}

impl JudgedScenario {
    fn judge(&self) -> MountTransparency {
        judge_mount_transparency(
            &self.before_the_mount,
            &self.after_the_action,
            self.action,
            &self.continuation,
        )
    }
}

/// 在 `crash_image` 上用 `mount_writable` 只挂载（v2a），连同挂载交回值一起交回。
fn mount_alone_with(
    parameters: &MakeFilesystemParameters,
    crash_image: &MemoryPool,
    mount_writable: impl FnOnce(
        &MakeFilesystemParameters,
        &mut Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
    ) -> Mounted,
) -> (JudgedScenario, MountOutput) {
    let stream = SharedStream::new();
    let mut devices = devices_of(crash_image, parameters, &stream);
    let mounted = mount_writable(parameters, &mut devices);
    let floor = FloorRaisedByTheMount::of_the_mount_output(&mounted.output);
    let continuation =
        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
    let output = mounted.output;
    let after = pool_of(devices, crash_image.device_size_in_bytes);
    (
        JudgedScenario {
            before_the_mount: read_recovery_fields(
                crash_image,
                ComparedVersion::TheRecoveredVersion,
            ),
            after_the_action: read_recovery_fields(&after, ComparedVersion::TheRecoveredVersion),
            action: MountTransparencyAction::MountAlone(floor),
            continuation,
        },
        output,
    )
}

/// 在 `crash_image` 上只挂载（v2a）。
fn mount_alone_on(width: HistoryDeviceWidth, crash_image: &MemoryPool) -> JudgedScenario {
    mount_alone_with(&width.parameters(), crash_image, mount).0
}

/// 在 `crash_image` 上挂载加一次发布，R2 比发布之前那一版。
fn mount_and_one_publish_on(width: HistoryDeviceWidth, crash_image: &MemoryPool) -> JudgedScenario {
    let parameters = width.parameters();
    let stream = SharedStream::new();
    let mut devices = devices_of(crash_image, &parameters, &stream);
    let mounted = mount(&parameters, &mut devices);
    let floor = FloorRaisedByTheMount::of_the_mount_output(&mounted.output);
    let continuation =
        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
    publish_once_after_the_mount(&parameters, &mut devices, mounted);
    let after = pool_of(devices, crash_image.device_size_in_bytes);
    JudgedScenario {
        before_the_mount: read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion),
        after_the_action: read_recovery_fields(
            &after,
            ComparedVersion::TheVersionBeforeTheRecoveredOne,
        ),
        action: MountTransparencyAction::MountAndOnePublish(floor),
        continuation,
    }
}

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;

fn pool_after_make_filesystem() -> MemoryPool {
    pool_after_the_history(
        SMALL,
        HistoryStartingPoint::AfterMakeFilesystem,
        Vec::new(),
        SpaceAdmission::JudgedByTheFormula,
    )
}

fn pool_after_the_first_file() -> MemoryPool {
    pool_after_the_history(
        SMALL,
        HistoryStartingPoint::AfterFirstFile,
        Vec::new(),
        SpaceAdmission::JudgedByTheFormula,
    )
}

/// 第一个文件之后挂载、正常卸载：最新那几条根带卸载记号。
fn pool_left_by_an_unmount() -> MemoryPool {
    let pool = pool_after_the_first_file();
    let parameters = SMALL.parameters();
    let stream = SharedStream::new();
    let mut devices = devices_of(&pool, &parameters, &stream);
    let mounted = mount(&parameters, &mut devices);
    let (_output, mut session) = MountedSession::of_the_mount(mounted);
    let unmounted = unmount(
        &parameters,
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .expect("正常卸载");
    assert!(
        matches!(unmounted, Unmounted::FloorRaisedToTheCurrentVersion(_)),
        "带文件的一版上卸载要推带卸载记号的空发布"
    );
    pool_of(devices, pool.device_size_in_bytes)
}

/// 第一个文件之后挂载、覆盖写一次，崩在那次覆盖写的根槽写之前：那次发布的单元与记录都落了，根没落。
/// 只读恢复由记录重建那一版（施加 1 条以上、W 不为 0）。
fn pool_rebuilt_from_a_record_of_an_overwrite() -> MemoryPool {
    let pool = pool_after_the_first_file();
    let parameters = SMALL.parameters();
    let mount_stream = SharedStream::new();
    let mut devices_of_the_mount = devices_of(&pool, &parameters, &mount_stream);
    let mounted = mount(&parameters, &mut devices_of_the_mount);
    let after_the_mount = pool_of(devices_of_the_mount, pool.device_size_in_bytes);
    let publish_stream = SharedStream::retaining_contents();
    let mut devices_of_the_publish = devices_of(&after_the_mount, &parameters, &publish_stream);
    publish_once_after_the_mount(&parameters, &mut devices_of_the_publish, mounted);
    let operations = publish_stream.retained_operations();
    let (writes, _segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &SMALL.fixed_geometry());
    let root_write = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("那次覆盖写有一次根槽写");
    let mut crash_image = after_the_mount;
    crash_image.apply(&operations[..stream_indexes[root_write]]);
    crash_image
}

/// 单元区 256 槽的小盘：第一个文件之后重挂、关掉空间准入覆盖写 12 次，再判准入挂载一次（攻方 r5 的 fms256-12 那一种造法）：
/// 挂载在写行与暖机之后推抬 F。
fn pool_where_the_mount_raises_the_floor() -> MemoryPool {
    let mut operations = vec![HistoryOperation::CloseAndMountWritable];
    operations.extend(std::iter::repeat_n(
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 200 },
            fill_seed: 11,
        }),
        12,
    ));
    pool_after_the_history(
        HistoryDeviceWidth::UnitAreaOf256Slots,
        HistoryStartingPoint::AfterFirstFile,
        operations,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
}

/// 第一个文件之后的盘面上挂载，崩在取号那几次系统配置槽写之后、写行那次发布的第一个别的写之前（挂载途中的二次崩溃，取号那一段）。
fn second_crash_after_the_acquisition(crash_image: &MemoryPool) -> JudgedScenario {
    let parameters = SMALL.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(crash_image, &parameters, &stream);
    let mounted = mount(&parameters, &mut devices);
    let continuation =
        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
    drop(mounted);
    let operations = stream.retained_operations();
    let (writes, _segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &SMALL.fixed_geometry());
    let first_write_after_the_acquisition = writes
        .iter()
        .position(|write| write.kind != StepKind::SystemConfigurationSlot)
        .expect("取号之后有写行那次发布的写");
    let mut second_crash_image = crash_image.clone();
    second_crash_image.apply(&operations[..stream_indexes[first_write_after_the_acquisition]]);
    JudgedScenario {
        before_the_mount: read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion),
        after_the_action: read_recovery_fields(
            &second_crash_image,
            ComparedVersion::TheRecoveredVersion,
        ),
        action: MountTransparencyAction::SecondCrashInsideTheMount(
            FloorRaisesAtTheSecondCrash::NoneLandedYet,
        ),
        continuation,
    }
}

/// 挂载会推抬 F 的那块小盘上挂载一次，二次崩溃摆在抬 F 第一个写的两侧（界由崩溃注入用的同一个分法 [`FloorRaisesInTheMountWrites`] 取）。
struct SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
    /// 界之前那几个写的种类：取号、写行、暖机连同暖机末尾的系统配置轮换。
    kinds_of_the_writes_before_the_floor_raises: Vec<StepKind>,
    /// 界之前的写全落、抬 F 的写一个没落。
    after_the_warm_up: JudgedScenario,
    /// 再多落抬 F 的第一个写（一块盘的系统配置里带上了新 F）。
    after_the_first_floor_raise_write: JudgedScenario,
}

/// 两态 F 那一样判不判，由 [`FloorRaisesInTheMountWrites::at_a_second_crash`] 按持久集合定（与崩溃注入同一个函数）。
fn second_crashes_on_both_sides_of_the_first_floor_raise_write(
) -> SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
    let width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let crash_image = pool_where_the_mount_raises_the_floor();
    let parameters = width.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&crash_image, &parameters, &stream);
    let mounted = mount(&parameters, &mut devices);
    let operations = stream.retained_operations();
    let (writes, _segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &width.fixed_geometry());
    let floor_raises = FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &writes);
    let continuation =
        UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output);
    let FloorRaisesInTheMountWrites::FirstWriteAt(first_floor_raise_write) = floor_raises else {
        panic!("前提：这次挂载在写行与暖机之后推了抬 F：{floor_raises:?}");
    };
    let before_the_mount = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    let second_crash_with_the_first_writes_landed = |landed_writes: usize| {
        let mut second_crash_image = crash_image.clone();
        second_crash_image.apply(&operations[..stream_indexes[landed_writes]]);
        let persisted_in_the_mount: Vec<bool> = (0..writes.len())
            .map(|write_index| write_index < landed_writes)
            .collect();
        JudgedScenario {
            before_the_mount: before_the_mount.clone(),
            after_the_action: read_recovery_fields(
                &second_crash_image,
                ComparedVersion::TheRecoveredVersion,
            ),
            action: MountTransparencyAction::SecondCrashInsideTheMount(
                floor_raises.at_a_second_crash(&persisted_in_the_mount),
            ),
            continuation: continuation.clone(),
        }
    };
    SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
        kinds_of_the_writes_before_the_floor_raises: writes[..first_floor_raise_write]
            .iter()
            .map(|write| write.kind)
            .collect(),
        after_the_warm_up: second_crash_with_the_first_writes_landed(first_floor_raise_write),
        after_the_first_floor_raise_write: second_crash_with_the_first_writes_landed(
            first_floor_raise_write + 1,
        ),
    }
}

fn assert_holds(verdict: &MountTransparency, field: MountTransparencyField) {
    assert_eq!(
        verdict.outcome_of(field),
        Some(&FieldOutcome::Holds),
        "{field:?} 要判绿：{verdict:#?}"
    );
    assert!(
        verdict.violated_fields().is_empty(),
        "合法挂载上一样都不许红：{verdict:#?}"
    );
}

fn assert_only_this_field_is_violated(verdict: &MountTransparency, field: MountTransparencyField) {
    let violated: Vec<MountTransparencyField> = verdict
        .violated_fields()
        .iter()
        .map(|field_verdict| field_verdict.field)
        .collect();
    assert_eq!(
        violated,
        vec![field],
        "改坏的只有 {field:?}，红的也只该是它：{verdict:#?}"
    );
}

fn compared_version_mut(fields: &mut RecoveryFieldsRead) -> &mut VersionFieldsRead {
    fields
        .compared_version
        .as_mut()
        .expect("动作之后的只读恢复有可比的那一版")
}

#[test]
fn posix_content_of_the_version_before_the_publish_after_the_mount_is_the_content_recovered_from_the_crash_image(
) {
    let scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
    assert_eq!(
        PosixContentRead::of_the_recovery_report(&scenario.after_the_action.report),
        PosixContentRead::File(content_published_after_the_mount()),
        "前提：R2 恢复落到的是挂载之后那次发布的内容，比的是它之前那一版"
    );
    assert_holds(&scenario.judge(), MountTransparencyField::PosixContent);
}

#[test]
fn posix_content_that_differs_from_the_crash_image_is_red() {
    let mut scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
    compared_version_mut(&mut scenario.after_the_action).content =
        PosixContentRead::File(content_published_after_the_mount());
    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::PosixContent);
}

#[test]
fn the_immutable_configuration_except_this_device_is_the_same_after_a_mount_alone() {
    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
    assert_holds(
        &scenario.judge(),
        MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
    );
}

#[test]
fn an_immutable_configuration_with_another_device_count_after_the_mount_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
    let immutable = scenario
        .after_the_action
        .immutable_configuration
        .as_mut()
        .expect("R2 择得到系统配置");
    immutable.device_count += 1;
    assert_only_this_field_is_violated(
        &scenario.judge(),
        MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
    );
}

#[test]
fn after_a_mount_on_a_pool_left_by_an_unmount_the_unmount_marker_is_not_written_by_the_unmount_sequence(
) {
    let scenario = mount_alone_on(SMALL, &pool_left_by_an_unmount());
    assert_eq!(
        scenario
            .before_the_mount
            .compared_version
            .as_ref()
            .expect("R1 落到一版")
            .root
            .unmount_marker,
        UnmountMarker::WrittenByTheUnmountSequence,
        "前提：R1 落到的是卸载那一串写的根（R1 的记号不进比较）"
    );
    assert_holds(&scenario.judge(), MountTransparencyField::UnmountMarker);
}

#[test]
fn an_unmount_marker_after_the_mount_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_left_by_an_unmount());
    compared_version_mut(&mut scenario.after_the_action)
        .root
        .unmount_marker = UnmountMarker::WrittenByTheUnmountSequence;
    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::UnmountMarker);
}

#[test]
fn a_mount_that_raises_no_floor_keeps_the_effective_floor_and_writes_it_into_its_root() {
    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
    assert_eq!(
        scenario.action,
        MountTransparencyAction::MountAlone(FloorRaisedByTheMount::NotRaised),
        "前提：这次挂载没推抬 F"
    );
    assert_holds(&scenario.judge(), MountTransparencyField::RollbackFloor);
}

#[test]
fn an_effective_floor_that_moved_although_the_mount_raised_none_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
    let effective_floor = scenario
        .after_the_action
        .effective_rollback_floor
        .expect("R2 算得出 F 生效值");
    scenario.after_the_action.effective_rollback_floor = Some(CheckpointTxg(effective_floor.0 + 1));
    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::RollbackFloor);
}

#[test]
fn a_mount_that_raises_the_floor_leaves_the_effective_floor_at_the_last_one_it_raised_to() {
    let width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let scenario = mount_alone_on(width, &pool_where_the_mount_raises_the_floor());
    assert!(
        matches!(
            scenario.action,
            MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(_))
        ),
        "前提：这次挂载在写行与暖机之后推了抬 F：{:?}",
        scenario.action
    );
    assert_holds(&scenario.judge(), MountTransparencyField::RollbackFloor);
}

#[test]
fn an_effective_floor_other_than_the_last_one_the_mount_raised_to_is_red() {
    let width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let mut scenario = mount_alone_on(width, &pool_where_the_mount_raises_the_floor());
    let MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(raised)) =
        scenario.action
    else {
        panic!("前提：这次挂载推了抬 F：{:?}", scenario.action);
    };
    scenario.action = MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(
        CheckpointTxg(raised.0 + 1),
    ));
    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::RollbackFloor);
}

#[test]
fn the_first_mount_of_a_new_pool_takes_one_above_every_instance_on_the_crash_image() {
    let scenario = mount_alone_on(SMALL, &pool_after_make_filesystem());
    assert_eq!(
        scenario
            .after_the_action
            .compared_version
            .as_ref()
            .expect("R2 落到一版")
            .root
            .instance,
        InstanceGeneration(1),
        "前提：mkfs 之后第一次可写挂载取 1（D23 已定项 16）"
    );
    assert_holds(
        &scenario.judge(),
        MountTransparencyField::InstanceGeneration,
    );
}

#[test]
fn an_instance_generation_one_too_high_after_the_mount_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_after_make_filesystem());
    let root = &mut compared_version_mut(&mut scenario.after_the_action).root;
    root.instance = InstanceGeneration(root.instance.0 + 1);
    assert_only_this_field_is_violated(
        &scenario.judge(),
        MountTransparencyField::InstanceGeneration,
    );
}

#[test]
fn instance_table_rows_follow_the_recovery_rule_when_the_crash_image_is_rebuilt_from_a_record() {
    let scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
    let journal = scenario.before_the_mount.report.journal;
    assert!(
        journal.prefix_applied > 0 && journal.maximum_applied_transaction > 0,
        "前提：R1 由记录重建那一版，施加过的事务号不为 0：{journal:?}"
    );
    assert_holds(&scenario.judge(), MountTransparencyField::InstanceTableRows);
}

#[test]
fn an_instance_table_row_whose_applied_transaction_is_dropped_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
    let recovered_instance = scenario
        .before_the_mount
        .compared_version
        .as_ref()
        .expect("R1 落到一版")
        .root
        .instance;
    let rows = compared_version_mut(&mut scenario.after_the_action)
        .instance_table_rows
        .as_mut()
        .expect("R2 的实例表读得出");
    let row = rows
        .iter_mut()
        .find(|row| row.instance == recovered_instance)
        .expect("挂载给 R1 那个实例写了一行");
    row.applied_transaction_high_water = 0;
    assert_only_this_field_is_violated(
        &scenario.judge(),
        MountTransparencyField::InstanceTableRows,
    );
}

#[test]
fn the_tree_identifier_watermark_is_unchanged_by_a_mount_alone() {
    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
    assert_holds(
        &scenario.judge(),
        MountTransparencyField::TreeIdentifierWatermark,
    );
}

#[test]
fn a_tree_identifier_watermark_that_moved_with_the_mount_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
    compared_version_mut(&mut scenario.after_the_action)
        .root
        .tree_identifier_watermark += 1;
    assert_only_this_field_is_violated(
        &scenario.judge(),
        MountTransparencyField::TreeIdentifierWatermark,
    );
}

#[test]
fn the_two_user_visible_tree_root_pointers_before_the_publish_after_the_mount_are_those_of_the_crash_image(
) {
    let scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
    assert_holds(
        &scenario.judge(),
        MountTransparencyField::UserVisibleTreeRootPointers,
    );
}

#[test]
fn an_inode_tree_root_pointer_that_moved_with_the_mount_is_red() {
    let mut scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
    let pointers = compared_version_mut(&mut scenario.after_the_action)
        .user_visible_tree_root_pointers
        .as_mut()
        .expect("R2 那一版的树表读得出");
    let inode_tree = pointers
        .inode_tree
        .as_mut()
        .expect("带文件的一版树表里有 inode 树");
    inode_tree[0] ^= 0xff;
    assert_only_this_field_is_violated(
        &scenario.judge(),
        MountTransparencyField::UserVisibleTreeRootPointers,
    );
}

#[test]
fn the_checkpoint_txg_lies_above_every_root_and_record_of_a_crash_image_rebuilt_from_a_record() {
    let scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
    assert!(
        scenario
            .before_the_mount
            .highest_record_checkpoint_txg_in_the_ring
            > scenario.before_the_mount.highest_root_txg_in_the_ring,
        "前提：环里有比全部根都新的记录（那次覆盖写的根没落）：{:?} / {:?}",
        scenario
            .before_the_mount
            .highest_record_checkpoint_txg_in_the_ring,
        scenario.before_the_mount.highest_root_txg_in_the_ring
    );
    assert_holds(&scenario.judge(), MountTransparencyField::CheckpointTxg);
}

#[test]
fn a_checkpoint_txg_that_does_not_clear_the_newest_record_of_the_crash_image_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
    let newest_record = scenario
        .before_the_mount
        .highest_record_checkpoint_txg_in_the_ring
        .expect("环里有记录");
    compared_version_mut(&mut scenario.after_the_action)
        .root
        .checkpoint_txg = newest_record;
    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::CheckpointTxg);
}

#[test]
fn the_recovery_after_the_mount_applies_no_record_even_when_the_crash_image_was_rebuilt_from_one() {
    let scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
    assert!(
        scenario.before_the_mount.report.journal.prefix_applied > 0,
        "前提：R1 施加过记录"
    );
    assert_holds(&scenario.judge(), MountTransparencyField::JournalScanReport);
}

#[test]
fn a_recovery_after_the_mount_that_applies_a_record_is_red() {
    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
    scenario.after_the_action.report.journal.prefix_applied = 1;
    assert_only_this_field_is_violated(
        &scenario.judge(),
        MountTransparencyField::JournalScanReport,
    );
}

#[test]
fn a_second_crash_right_after_the_acquisition_recovers_the_first_version_field_by_field() {
    let scenario = second_crash_after_the_acquisition(&pool_after_the_first_file());
    let verdict = scenario.judge();
    for field in MountTransparencyField::ALL {
        let expected = match field {
            MountTransparencyField::JournalScanReport => {
                FieldOutcome::NotJudged(FieldNotJudged::JournalScanReportOfASecondCrashImage)
            }
            MountTransparencyField::PosixContent
            | MountTransparencyField::ImmutableConfigurationWithoutThisDevice
            | MountTransparencyField::UnmountMarker
            | MountTransparencyField::RollbackFloor
            | MountTransparencyField::InstanceGeneration
            | MountTransparencyField::InstanceTableRows
            | MountTransparencyField::TreeIdentifierWatermark
            | MountTransparencyField::UserVisibleTreeRootPointers
            | MountTransparencyField::CheckpointTxg => FieldOutcome::Holds,
        };
        assert_eq!(
            verdict.outcome_of(field),
            Some(&expected),
            "{field:?}：{verdict:#?}"
        );
    }
}

#[test]
fn a_second_crash_recovering_content_other_than_the_first_version_is_red() {
    let mut scenario = second_crash_after_the_acquisition(&pool_after_the_first_file());
    compared_version_mut(&mut scenario.after_the_action).content = PosixContentRead::NoFile;
    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::PosixContent);
}

/// 代码轮第二轮判决 K2-1：挂载推了抬 F 时，暖机那一次发布的写崩出来的二次崩溃态照 r5 判决 U5 前半句判 F 不变，
/// 抬 F 的第一个写起才算 F 不判（攻方量到 fms12 那一次挂载暖机那几段 4 107 态以前整段记成不判）。
#[test]
fn a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged(
) {
    let SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
        kinds_of_the_writes_before_the_floor_raises,
        after_the_warm_up,
        after_the_first_floor_raise_write,
    } = second_crashes_on_both_sides_of_the_first_floor_raise_write();
    assert!(
        kinds_of_the_writes_before_the_floor_raises
            .iter()
            .filter(|kind| **kind == StepKind::RootRecordFua)
            .count()
            >= 2
            && kinds_of_the_writes_before_the_floor_raises.last()
                == Some(&StepKind::SystemConfigurationSlot),
        "抬 F 的界在暖机之后：界之前有写行与暖机两次发布的根槽写，紧挨着界的是暖机末尾的系统配置轮换：\
         {kinds_of_the_writes_before_the_floor_raises:?}"
    );
    assert_eq!(
        after_the_warm_up.action,
        MountTransparencyAction::SecondCrashInsideTheMount(
            FloorRaisesAtTheSecondCrash::NoneLandedYet
        ),
        "暖机全落、抬 F 的写一个没落：抬 F 一步都还没落"
    );
    assert_holds(
        &after_the_warm_up.judge(),
        MountTransparencyField::RollbackFloor,
    );
    assert_eq!(
        after_the_first_floor_raise_write.action,
        MountTransparencyAction::SecondCrashInsideTheMount(
            FloorRaisesAtTheSecondCrash::SomeMayHaveLanded
        ),
        "抬 F 的第一个写落了：抬 F 那几串可能落了一部分"
    );
    let verdict = after_the_first_floor_raise_write.judge();
    assert_eq!(
        verdict.outcome_of(MountTransparencyField::RollbackFloor),
        Some(&FieldOutcome::NotJudged(
            FieldNotJudged::FloorOfASecondCrashAfterTheFloorRaisesBegan
        )),
        "抬 F 开始之后的 F 不判：{verdict:#?}"
    );
    assert!(
        verdict.violated_fields().is_empty(),
        "合法挂载途中再崩一次，一样都不许红：{verdict:#?}"
    );
}

/// 挂载会推抬 F 的那块小盘上挂载一次：交回挂载那一段写表与挂载交回值。
fn writes_and_output_of_the_mount_that_raises_the_floor() -> (Vec<RetainedWrite>, MountOutput) {
    let width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let crash_image = pool_where_the_mount_raises_the_floor();
    let parameters = width.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&crash_image, &parameters, &stream);
    let mounted = mount(&parameters, &mut devices);
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &width.fixed_geometry(),
    );
    (writes, mounted.output)
}

/// 挂载交回的第一串抬 F 第一次发布的根里的 F：这一串抬到的 F。
fn floor_raised_by_the_first_raise(output: &MountOutput) -> CheckpointTxg {
    output
        .space_admission
        .floor_raises()
        .first()
        .expect("前提：这次挂载推了抬 F")
        .publishes
        .first()
        .expect("抬 F 那一串至少推一次带新 F 的空发布")
        .root
        .rollback_floor
}

/// 系统配置槽写里带的 F；不是系统配置槽写时 None。
fn floor_written_into_the_system_configuration(write: &RetainedWrite) -> Option<CheckpointTxg> {
    (write.kind == StepKind::SystemConfigurationSlot)
        .then(|| write.bytes())
        .flatten()
        .and_then(|bytes| SystemConfiguration::parse_slot(bytes).ok())
        .map(|written| written.quantities.rollback_floor)
}

/// 把一次系统配置槽写里的 F 改成 `floor`、重新序列化（整槽校验和随 `to_slot` 重算）。
fn rewrite_the_floor_of_the_system_configuration_write(
    write: &mut RetainedWrite,
    floor: CheckpointTxg,
) {
    let mut written = SystemConfiguration::parse_slot(write.bytes().expect("系统配置槽写带字节"))
        .expect("挂载写的系统配置槽解得开");
    written.quantities.rollback_floor = floor;
    write.contents = WrittenContents::Bytes(written.to_slot());
}

/// 第一个带着这一串抬到的 F 的系统配置槽写的下标（按内容认的旧取法）；没有时 None。
fn first_system_configuration_write_carrying(
    writes: &[RetainedWrite],
    floor: CheckpointTxg,
) -> Option<usize> {
    writes
        .iter()
        .position(|write| floor_written_into_the_system_configuration(write) == Some(floor))
}

/// 代码轮第三轮判决 K2 第 ① 种形状（攻方 `research/prompts/m3-formal-code-r3-opus-output.md` 第四节）：盘上已生效的 F 高于这一串抬到的 F 时，
/// 实现写进系统配置的是两者的大者（这次挂载的每一次系统配置槽写都带着更高的那个 F），没有一次系统配置槽写带着抬到的 F。
/// 实现今天的准入造不出这样的挂载（推的），这里把真挂载写表里的系统配置槽写改成带更高的 F：界按位置取，取出的仍是抬 F 的第一个写，不 panic。
#[test]
fn the_floor_raise_boundary_is_located_by_position_when_every_system_configuration_write_carries_a_higher_effective_floor(
) {
    let (mut writes, output) = writes_and_output_of_the_mount_that_raises_the_floor();
    let raised_floor = floor_raised_by_the_first_raise(&output);
    let boundary_by_content = first_system_configuration_write_carrying(&writes, raised_floor)
        .expect("前提：真挂载的写表里抬 F 的第一个写带着抬到的 F");
    assert_eq!(
        FloorRaisesInTheMountWrites::of_the_mount_output(&output, &writes),
        FloorRaisesInTheMountWrites::FirstWriteAt(boundary_by_content),
        "真挂载的写表上按位置取的界与按内容认的同一个写"
    );
    let higher_effective_floor = CheckpointTxg(raised_floor.0 + 1);
    for write in writes
        .iter_mut()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
    {
        rewrite_the_floor_of_the_system_configuration_write(write, higher_effective_floor);
    }
    assert_eq!(
        first_system_configuration_write_carrying(&writes, raised_floor),
        None,
        "前提：改过之后没有一次系统配置槽写带着抬到的 F（按内容认就找不到界）"
    );
    assert_eq!(
        FloorRaisesInTheMountWrites::of_the_mount_output(&output, &writes),
        FloorRaisesInTheMountWrites::FirstWriteAt(boundary_by_content),
        "界按抬 F 那几串在写表末尾的位置取，不看系统配置里写的 F"
    );
}

/// 代码轮第三轮判决 K2 第 ② 种形状：挂载里更早的一次系统配置槽写（取号那几次的第一次）恰带着这一串抬到的 F，按内容认会把界认早；
/// 按位置取，界仍是抬 F 的第一个写。
#[test]
fn the_floor_raise_boundary_is_not_taken_early_when_an_earlier_system_configuration_write_carries_the_raised_floor(
) {
    let (mut writes, output) = writes_and_output_of_the_mount_that_raises_the_floor();
    let raised_floor = floor_raised_by_the_first_raise(&output);
    let boundary_by_content = first_system_configuration_write_carrying(&writes, raised_floor)
        .expect("前提：真挂载的写表里抬 F 的第一个写带着抬到的 F");
    let first_acquisition_write = writes
        .iter()
        .position(|write| write.kind == StepKind::SystemConfigurationSlot)
        .expect("挂载第一个写是取号的系统配置槽写");
    assert!(
        first_acquisition_write < boundary_by_content,
        "前提：取号那几次系统配置槽写在抬 F 之前"
    );
    rewrite_the_floor_of_the_system_configuration_write(
        &mut writes[first_acquisition_write],
        raised_floor,
    );
    assert_eq!(
        first_system_configuration_write_carrying(&writes, raised_floor),
        Some(first_acquisition_write),
        "前提：改过之后按内容认会把界认在取号那一次"
    );
    assert_eq!(
        FloorRaisesInTheMountWrites::of_the_mount_output(&output, &writes),
        FloorRaisesInTheMountWrites::FirstWriteAt(boundary_by_content),
        "界按抬 F 那几串在写表末尾的位置取，不认更早的那一次"
    );
}

/// 交来的写表与挂载交回对不上（只有抬 F 之前的那些写）：界取不出，不 panic；这一次挂载的二次崩溃 F 那一样整段不判、按理由计。
#[test]
fn a_mount_whose_floor_raises_are_not_in_the_handed_writes_leaves_the_floor_of_every_second_crash_unjudged(
) {
    let (writes, output) = writes_and_output_of_the_mount_that_raises_the_floor();
    let raised_floor = floor_raised_by_the_first_raise(&output);
    let boundary_by_content = first_system_configuration_write_carrying(&writes, raised_floor)
        .expect("前提：真挂载的写表里抬 F 的第一个写带着抬到的 F");
    let writes_before_the_floor_raises = &writes[..boundary_by_content];
    let floor_raises =
        FloorRaisesInTheMountWrites::of_the_mount_output(&output, writes_before_the_floor_raises);
    assert_eq!(
        floor_raises,
        FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites,
        "抬 F 那几串的根槽写不在交来的写表里：界取不出"
    );
    let nothing_persisted = vec![false; writes_before_the_floor_raises.len()];
    assert_eq!(
        floor_raises.at_a_second_crash(&nothing_persisted),
        FloorRaisesAtTheSecondCrash::BoundaryNotLocatedInTheMountWrites,
        "界取不出时一个写都没持久也不判 F：分不开暖机与抬 F"
    );
    // 二次崩溃镜像借第一个文件之后那块盘上崩在取号之后的那一态（合法、F 不变）：只换动作里 F 那一样的判法。
    let mut scenario = second_crash_after_the_acquisition(&pool_after_the_first_file());
    scenario.action = MountTransparencyAction::SecondCrashInsideTheMount(
        floor_raises.at_a_second_crash(&nothing_persisted),
    );
    let verdict = scenario.judge();
    assert_eq!(
        verdict.outcome_of(MountTransparencyField::RollbackFloor),
        Some(&FieldOutcome::NotJudged(
            FieldNotJudged::FloorOfASecondCrashInAMountWhoseFloorRaisesWereNotLocated
        )),
        "F 那一样记不判、理由是界取不出：{verdict:#?}"
    );
    assert!(
        verdict.violated_fields().is_empty(),
        "合法挂载途中再崩一次，一样都不许红：{verdict:#?}"
    );
}

/// 第一个文件之后经挂着的会话 unlink 它（意图那次 + 最后一批），崩在意图那次的根槽写与紧跟着的系统配置槽写之后、最后一批的第一个写之前：
/// 只读恢复落到意图那一版（第一个文件在删除中，读出「没有文件」），可写挂载在写行与暖机之后续做那一批（D8（核心索引结构） 已定项 16 第 6 条）。
fn pool_crashed_right_after_the_unlink_intent() -> MemoryPool {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::Unlink],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: SMALL,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |_observation| {},
    );
    assert_eq!(
        run.outcomes.len(),
        history.operations.len(),
        "unlink 那一步要跑完：{:?}",
        run.ending
    );
    let operations = stream.retained_operations();
    let (writes, _segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &SMALL.fixed_geometry());
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(write_index, _)| write_index)
        .collect();
    let [.., intent_root_write, _last_batch_root_write] = root_writes.as_slice() else {
        panic!("前提：录制流里至少有意图与最后一批两次根槽写：{root_writes:?}");
    };
    let first_write_of_the_last_batch = (intent_root_write + 1..writes.len())
        .find(|write_index| writes[*write_index].kind != StepKind::SystemConfigurationSlot)
        .expect("意图那次之后还有最后一批的写");
    let mut crash_image = MemoryPool::with_devices(&DEVICES, SMALL.device_bytes());
    crash_image.apply(&operations[..stream_indexes[first_write_of_the_last_batch]]);
    crash_image
}

/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 窄盘上 unlink 一个 14 个数据单元的文件、发完意图就崩（单元区 256 槽、不为环窗口补足，第一个文件之后覆盖写 2 次再顺序写长到 14 个单元），
/// 与 `admission_raises_the_floor_before_refusing.rs` 里「挂载续做里一批被落得下那一判拒、推一串抬 F 之后走完」那一条同一种造法（那边第十位实现员的
/// 草稿探针定的 256 槽、2 次、14 个单元）。交回崩溃那一刻的盘面与建池那一份参数。
fn pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices(
) -> (MemoryPool, MakeFilesystemParameters) {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            256,
        );
    for overwrite_index in 1..=2 {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
    }
    pool.sequential_write(13 * data_unit_payload_capacity() + 100)
        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 14 个单元：{refusal:?}"));
    let PoolUnderTest {
        session,
        devices,
        parameters,
        write_time_seconds,
        ..
    } = &mut pool;
    let session = session.as_mut().expect("这一刻有可写会话");
    let current = session
        .current
        .version_of_trees()
        .expect("第一个文件之后有树")
        .clone();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    publish_unlink_intent(
        &mut writer,
        &mut session.allocator,
        &current,
        InodeNumber(FIRST_INODE_NUMBER),
        *write_time_seconds,
        session.instance,
    )
    .expect("意图那次");
    pool.session = None;
    (pool.image(), pool.parameters.clone())
}

/// 用户 2026-10-03 定「续做算挂载的一部分」：崩在 unlink 意图之后重挂，挂载在写行与暖机之后续做最后一批、把两棵用户可见树的根改了；
/// 只挂载之后的只读恢复落到续做那一版，两根指针比挂载交回的续做最后一步那一版的，不比 R1 那一版的，别的九样照常判绿。
/// 判别力：不认续做那几步、照旧比 R1 那一版的时，用户可见树根指针那一样判红（第 11 批实现员 B 的探针量到的那一形）。
#[test]
fn after_a_mount_that_continued_the_deletion_the_two_tree_root_pointers_are_those_of_its_last_continuation_step(
) {
    let crash_image = pool_crashed_right_after_the_unlink_intent();
    let (scenario, output) = mount_alone_with(&SMALL.parameters(), &crash_image, mount);
    assert!(
        !output.deletion_queue_continuation.is_empty(),
        "前提：崩在意图之后重挂，挂载续做了"
    );
    assert_eq!(
        scenario
            .before_the_mount
            .compared_version
            .as_ref()
            .expect("R1 落到意图那一版")
            .content,
        PosixContentRead::NoFile,
        "前提：R1 落到意图那一版，第一个文件在删除中、读出「没有文件」"
    );
    assert_ne!(
        scenario
            .after_the_action
            .compared_version
            .as_ref()
            .expect("R2 落到一版")
            .user_visible_tree_root_pointers,
        scenario
            .before_the_mount
            .compared_version
            .as_ref()
            .expect("R1 落到一版")
            .user_visible_tree_root_pointers,
        "前提：续做改了两棵树的根指针"
    );
    assert_holds(
        &scenario.judge(),
        MountTransparencyField::UserVisibleTreeRootPointers,
    );
}

/// 用户 2026-10-03 定「续做算挂载的一部分」：窄盘上崩在 unlink 意图之后重挂（删除批大小 1），准入在取号之前放行、一串都没推，
/// 续做里一批被空间拒、推一串抬 F 之后走完；只挂载之后 F 生效值是续做推的那一串抬到的 F，别的九样照常判绿。
/// 判别力：抬到的 F 只认挂载准入推的那几串时这一形交「没推」，F 照「生效值不变」判红。
#[test]
fn a_mount_whose_continuation_raised_the_floor_leaves_the_effective_floor_at_the_last_one_the_continuation_raised_to(
) {
    let (crash_image, parameters) =
        pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices();
    let (scenario, output) = mount_alone_with(&parameters, &crash_image, |parameters, devices| {
        mount_writable_with_test_only_switches_and_deletion_switches(
            parameters,
            devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            deletion_switches_with_batches_of_one(),
        )
        .unwrap_or_else(|error| panic!("续做推过抬 F 之后走完：{error:?}"))
    });
    assert_eq!(
        (
            output.space_admission.floor_raises().len(),
            output.floor_raises_pushed_by_the_continuation.len()
        ),
        (0, 1),
        "前提：准入一串都没推，续做里推了一串"
    );
    assert_holds(&scenario.judge(), MountTransparencyField::RollbackFloor);
}

/// 造写表用：一个 512 字节的写；根槽写的字节里在偏移 24、28 写上根身份（`root_identity_written_by` 的读法）。
fn synthetic_write(kind: StepKind, root: Option<(u32, u64)>) -> RetainedWrite {
    let mut bytes = vec![0u8; 512];
    if let Some((instance, checkpoint_txg)) = root {
        bytes[24..28].copy_from_slice(&instance.to_le_bytes());
        bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
    }
    RetainedWrite {
        device: DeviceIdentity(0),
        kind,
        is_force_unit_access: kind == StepKind::RootRecordFua,
        offset: DeviceOffsetInBytes(0),
        contents: WrittenContents::Bytes(bytes),
    }
}

fn publish_of(write_calls: u64, instance: u32, checkpoint_txg: u64) -> PublishInTheMountWrites {
    PublishInTheMountWrites {
        write_calls,
        root: (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg)),
    }
}

/// 挂载续做里为空间推的抬 F（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定「挂载续做被空间拒：同样推抬 F 再试」）：
/// 那一串夹在续做的两步之间、不在挂载写表末尾，界按它在写表里的位置取。造的写表：取号；写行（实例 2、txg 10）；暖机一次（txg 11）；
/// 续做第一步（txg 12）；续做推的一串抬 F（先把新 F 写进系统配置，再一次空发布 txg 13）；续做第二步（txg 14）。
/// 判别力：只认挂载准入推的那几串（这一份取界在第 12 批之前的做法）时这一形交「没推」，
/// 二次崩溃崩在那一串里时 F 照「没推」判。
#[test]
fn a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps() {
    let writes = vec![
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 14))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
    ];
    let floor_raises = FloorRaisesOfTheMount {
        pushed_by_the_admission: Vec::new(),
        continuation_steps: vec![publish_of(4, 2, 12), publish_of(4, 2, 14)],
        pushed_by_the_continuation: vec![FloorRaiseInTheMountWrites::new(
            1,
            vec![publish_of(3, 2, 13)],
        )],
    };
    let located =
        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&floor_raises, &writes);
    assert_eq!(
        located,
        FloorRaisesInTheMountWrites::FirstWriteAt(12),
        "续做推的那一串从它把新 F 写进系统配置的那个写（第 12 个）起，后面还有续做第二步的 4 个写"
    );
    let mut persisted_through_the_first_continuation_step = vec![false; writes.len()];
    persisted_through_the_first_continuation_step[..12].fill(true);
    assert_eq!(
        located.at_a_second_crash(&persisted_through_the_first_continuation_step),
        FloorRaisesAtTheSecondCrash::NoneLandedYet,
        "崩在续做第一步之后、抬 F 那一串之前：F 一步都还没落"
    );
    let mut persisted_into_the_floor_raise = persisted_through_the_first_continuation_step;
    persisted_into_the_floor_raise[12] = true;
    assert_eq!(
        located.at_a_second_crash(&persisted_into_the_floor_raise),
        FloorRaisesAtTheSecondCrash::SomeMayHaveLanded,
        "崩在续做推的那一串里：新 F 可能落了一部分，F 不照「没推」判"
    );
}

/// 挂载准入推了抬 F、之后又有续做：准入推的那几串不在写表末尾（后面接着续做的各步），界要把续做的写也算进去。
/// 造的写表：取号；写行（txg 10）；暖机（txg 11）；准入推的一串抬 F（系统配置写、空发布 txg 12）；续做一步（txg 13）。
/// 判别力：按「抬 F 那几串在写表末尾」取界时界落在续做那一步的单元写上，交「界取不出」，这一次挂载的二次崩溃整段 F 不判。
#[test]
fn floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them(
) {
    let writes = vec![
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
    ];
    let floor_raises = FloorRaisesOfTheMount {
        pushed_by_the_admission: vec![FloorRaiseInTheMountWrites::new(
            1,
            vec![publish_of(3, 2, 12)],
        )],
        continuation_steps: vec![publish_of(4, 2, 13)],
        pushed_by_the_continuation: Vec::new(),
    };
    assert_eq!(
        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&floor_raises, &writes),
        FloorRaisesInTheMountWrites::FirstWriteAt(8),
        "准入推的那一串从第 8 个写（新 F 写进系统配置）起，后面是它的空发布与续做那一步"
    );
    let nothing_pushed = FloorRaisesOfTheMount {
        pushed_by_the_admission: Vec::new(),
        continuation_steps: vec![publish_of(4, 2, 13)],
        pushed_by_the_continuation: Vec::new(),
    };
    assert_eq!(
        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&nothing_pushed, &writes),
        FloorRaisesInTheMountWrites::NotPushed,
        "准入与续做都没推：没推"
    );
}

/// V6（`research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：挂载准入推了一串抬 F、之后的续做里又为空间推了一串：
/// 界从准入推的那一串起，后面的续做各步与续做推的那一串都在界之后。造的写表：取号；写行（txg 10）；暖机（txg 11）；准入推的一串
/// （系统配置写、空发布 txg 12）；续做第一步（txg 13）；续做推的一串（系统配置写、空发布 txg 14）；续做第二步（txg 15）。
/// 判别力：准入推过时不把续做推的那几串的写算进去，界落在续做第一步的单元写上、取不出。
#[test]
fn floor_raises_pushed_by_both_the_admission_and_the_continuation_are_located_from_the_first_admission_raise(
) {
    let writes = vec![
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 13))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 14))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
        synthetic_write(StepKind::UnitWrite, None),
        synthetic_write(StepKind::JournalRecord, None),
        synthetic_write(StepKind::RootRecordFua, Some((2, 15))),
        synthetic_write(StepKind::SystemConfigurationSlot, None),
    ];
    let floor_raises = FloorRaisesOfTheMount {
        pushed_by_the_admission: vec![FloorRaiseInTheMountWrites::new(
            1,
            vec![publish_of(3, 2, 12)],
        )],
        continuation_steps: vec![publish_of(4, 2, 13), publish_of(4, 2, 15)],
        pushed_by_the_continuation: vec![FloorRaiseInTheMountWrites::new(
            1,
            vec![publish_of(3, 2, 14)],
        )],
    };
    let located =
        FloorRaisesInTheMountWrites::of_the_floor_raises_of_the_mount(&floor_raises, &writes);
    assert_eq!(
        located,
        FloorRaisesInTheMountWrites::FirstWriteAt(8),
        "准入推的那一串从第 8 个写（新 F 写进系统配置）起，后面是它的空发布、续做两步与续做推的那一串"
    );
    let mut persisted_through_the_warm_up = vec![false; writes.len()];
    persisted_through_the_warm_up[..8].fill(true);
    assert_eq!(
        located.at_a_second_crash(&persisted_through_the_warm_up),
        FloorRaisesAtTheSecondCrash::NoneLandedYet,
        "崩在暖机之后、准入推的那一串之前：F 一步都还没落"
    );
    let mut persisted_into_the_continuation_raise = vec![false; writes.len()];
    persisted_into_the_continuation_raise[..17].fill(true);
    assert_eq!(
        located.at_a_second_crash(&persisted_into_the_continuation_raise),
        FloorRaisesAtTheSecondCrash::SomeMayHaveLanded,
        "崩在续做推的那一串里：F 不照「没推」判"
    );
}
