//! checker 档模块：无（自己逐个造崩溃状态，只用 harness 档的内存池与池级 checker）
//! C577（系统配置没见证到的最新根，乙罩不到）：发布返回之前，系统配置轮换之后再一道屏障（D16（发布语义） 已定项 7「根槽（FUA）→
//! 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回」，用户 2026-09-27 JST 15:1x 定「发布返回前加屏障」）。
//!
//! 改之前发布路径根槽 FUA 之后轮换系统配置就返回：返回的那一刻轮换还可能没持久，崩溃之后系统配置只见证到上一次发布；这次的根槽
//! 一时读不出时，可写挂载的见证判据（D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」）看不出有更新的状态，
//! 把这次发布当被抛弃、它引用的单元再发出去。
//!
//! 判据按录制流量（D13（验证路线） 已定项 4 的切段：屏障与 FUA 写切段，段内任意整写子集）：一次发布返回之后的每个崩溃点，
//! 每个可达的崩溃状态里，池里两块盘两槽全部自证过的系统配置的 journal tail 取最大（见证值，与可写挂载判 N-配置、取号写的见证值同一取法）
//! 都不小于这次发布末条记录的计数器。计数器全池接着走（D23（journal 的角色与格式） 已定项 14 第 3 条），后面换了实例也照比。
//! 见证值只由落在系统配置两槽上的写定：崩溃状态只在「还没被屏障放行的那一段」里的系统配置槽写上取子集，别的写不改它。
//! 撕开的原地覆写不另枚举：被撕的是较旧的那一槽（两槽轮换），撕坏自证不过，与那一写没持久对见证值是同一个数。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, format_pool, geometry, parameters, publish_overwrite_in_process, BuiltPool,
    Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, unmount, ShadowLedger, Unmounted,
};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::transaction::{
    publish_overwrite, resend_the_frozen_publish, FirstFile, PoolWriter, PublishError,
    TransactionOutput,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::{SegmentAfterOperation, SegmentClosingRule, StepKind};
use singlefs_harness::RetainedOperation;

/// 一块盘两槽一份自证过的系统配置都没有时的见证值：没见证任何发布。
const WITNESS_OF_NO_SELF_VERIFIED_SYSTEM_CONFIGURATION: u64 = 0;

/// 一个崩溃点上还没被屏障放行、可以任取子集的系统配置槽写最多几个：两块盘一次轮换两个，加上前后相邻的一次系统配置写。
/// 超过它说明录制流里出现了没想到的形状，停下来看，不去枚举一个大的子集族。
const OPEN_SYSTEM_CONFIGURATION_WRITES_AT_MOST: usize = 8;

/// 返回之前那个崩溃点（这次发布最后一步还没发出）上，见证值该不该还落在它后面：判法看不看得见「没见证」的那一核。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WitnessJustBeforeTheReturn {
    /// 这次的轮换那一刻还没全放行，可达状态里有见证值小于它计数器的：判法在那里必须报得出。
    StillBehindItsLastRecord,
    /// 原样重发的那一次：失败那一遍已经发过同一个 tail 的轮换写，重发的头一道屏障就放行了它，返回之前见证值可以已经到了。
    AlreadyWitnessedByTheFailedAttempt,
}

/// 一次返回了的发布：录制流在它返回那一刻有几步（崩溃点从这里起算「返回之后」）、它末条记录的计数器、叫什么（报错时认得出）。
#[derive(Clone, Copy, Debug)]
struct ReturnedPublish {
    name: &'static str,
    operations_when_it_returned: usize,
    last_record_counter: u64,
    witness_just_before_the_return: WitnessJustBeforeTheReturn,
}

/// 一处违例：返回之后的第 `crash_point` 个崩溃点（已发出录制流的前 `crash_point` 步）上，有一个可达的崩溃状态的见证值小于那次发布末条记录的计数器。
#[derive(Debug, PartialEq, Eq)]
struct WitnessBehindAReturnedPublish {
    publish: &'static str,
    crash_point: usize,
    lowest_witness: u64,
    last_record_counter: u64,
}

/// 两块盘两槽里全部自证过（fsid 与本池相同）的系统配置的 journal tail 的最大值。
fn witness_on(image: &MemoryPool) -> u64 {
    let publish_parameters = parameters();
    let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                image,
                device,
                slot_spacing_in_bytes,
                &publish_parameters.filesystem_identifier,
            )
        })
        .map(|system_configuration| system_configuration.quantities.journal_tail)
        .max()
        .unwrap_or(WITNESS_OF_NO_SELF_VERIFIED_SYSTEM_CONFIGURATION)
}

/// 已持久的那几个系统配置槽写之上，再任取还没放行的那几个的一个子集（按录制流次序施加），全部子集里见证值的最小值。
///
/// 循环：子集按位掩码逐个（轮数 = 2^还没放行的写数，上界 2^[`OPEN_SYSTEM_CONFIGURATION_WRITES_AT_MOST`]），每一轮从已持久的镜像拷一份起，
/// 不跨轮携带状态；没有提前出口。
fn lowest_witness_over_the_open_subsets(
    persisted_image: &MemoryPool,
    open_system_configuration_writes: &[&RetainedOperation],
) -> u64 {
    assert!(
        open_system_configuration_writes.len() <= OPEN_SYSTEM_CONFIGURATION_WRITES_AT_MOST,
        "还没放行的系统配置槽写有 {} 个，多于 {OPEN_SYSTEM_CONFIGURATION_WRITES_AT_MOST}：录制流的形状出乎预料",
        open_system_configuration_writes.len()
    );
    let subset_count = 1usize << open_system_configuration_writes.len();
    (0..subset_count)
        .map(|subset_mask| {
            let mut image = persisted_image.clone();
            let persisted_in_this_subset: Vec<RetainedOperation> = open_system_configuration_writes
                .iter()
                .enumerate()
                .filter(|(write_index, _)| subset_mask & (1usize << write_index) != 0)
                .map(|(_, write)| (*write).clone())
                .collect();
            image.apply(&persisted_in_this_subset);
            witness_on(&image)
        })
        .min()
        .expect("子集族至少有空集那一个")
}

/// 录制流每个崩溃点（已发出前 p 步，p 从 0 到流长）上，全部可达崩溃状态里见证值的最小值；交回的第 p 项是第 p 个崩溃点的。
/// 切段照 [`SegmentClosingRule`]（与层 0 同一条规则）：段关上时段里的写全持久，还开着的那一段里的写任取子集。
///
/// 循环：按录制流逐步（轮数 = 步数），跨轮携带「已持久的系统配置槽写施加出的镜像」与「当前段里的系统配置槽写」，
/// 段关上时把后者按次序施加进前者再清空；没有提前出口。
fn lowest_witness_at_every_crash_point(operations: &[RetainedOperation]) -> Vec<u64> {
    let classifying_geometry = geometry();
    let system_configuration_end_in_bytes = SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
        * u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let mut persisted_image =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    let mut open_system_configuration_writes: Vec<&RetainedOperation> = Vec::new();
    let mut closing_rule = SegmentClosingRule::default();
    let mut lowest_witnesses = vec![witness_on(&persisted_image)];
    for retained in operations {
        match classifying_geometry.classify(&retained.operation) {
            StepKind::SystemConfigurationSlot => open_system_configuration_writes.push(retained),
            StepKind::ZeroFill => assert!(
                retained.operation.offset.0 >= system_configuration_end_in_bytes,
                "整段清零落在系统配置两槽上：见证值不再只由系统配置槽写定，这份判法不成立：{:?}",
                retained.operation
            ),
            StepKind::UnitWrite
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::Barrier => {}
        }
        match closing_rule.after(&retained.operation) {
            SegmentAfterOperation::Closes => {
                let released: Vec<RetainedOperation> = open_system_configuration_writes
                    .drain(..)
                    .cloned()
                    .collect();
                persisted_image.apply(&released);
            }
            SegmentAfterOperation::StaysOpen => {}
        }
        lowest_witnesses.push(lowest_witness_over_the_open_subsets(
            &persisted_image,
            &open_system_configuration_writes,
        ));
    }
    lowest_witnesses
}

/// 每次返回了的发布：从它返回那一刻到录制流末尾的每个崩溃点上，见证值的最小值不小于它末条记录的计数器；交回全部违例。
/// 另核判法看得见「没见证」：每次发布（原样重发的除外，见 [`WitnessJustBeforeTheReturn`]）返回之前的那个崩溃点（它最后一步还没发出）上，最小见证值小于它的计数器——
/// 那一刻这次的轮换还没全放行，判法要是在那里也报「见证到了」，返回之后的绿就什么都没说明。
fn witnesses_behind_returned_publishes(
    operations: &[RetainedOperation],
    returned_publishes: &[ReturnedPublish],
) -> Vec<WitnessBehindAReturnedPublish> {
    let lowest_witnesses = lowest_witness_at_every_crash_point(operations);
    let mut violations = Vec::new();
    for returned in returned_publishes {
        let crash_point_before_its_last_step = returned.operations_when_it_returned - 1;
        match returned.witness_just_before_the_return {
            WitnessJustBeforeTheReturn::StillBehindItsLastRecord => assert!(
                lowest_witnesses[crash_point_before_its_last_step] < returned.last_record_counter,
                "{}：返回之前最后一步还没发出时，最小见证值 {} 已不小于它的计数器 {}，判法看不出「没见证」",
                returned.name,
                lowest_witnesses[crash_point_before_its_last_step],
                returned.last_record_counter
            ),
            WitnessJustBeforeTheReturn::AlreadyWitnessedByTheFailedAttempt => {}
        }
        for (crash_point, lowest_witness) in lowest_witnesses
            .iter()
            .enumerate()
            .skip(returned.operations_when_it_returned)
        {
            if *lowest_witness < returned.last_record_counter {
                violations.push(WitnessBehindAReturnedPublish {
                    publish: returned.name,
                    crash_point,
                    lowest_witness: *lowest_witness,
                    last_record_counter: returned.last_record_counter,
                });
            }
        }
    }
    violations
}

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 13 + seed * 7 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 同一个进程里覆盖写一次，交回返回了的那次发布。
fn overwrite_and_note_the_return(
    pool: &mut BuiltPool,
    seed: usize,
    name: &'static str,
) -> ReturnedPublish {
    let previous = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        pool,
        &previous,
        &content_seeded_by(4100 + seed, seed),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    ReturnedPublish {
        name,
        operations_when_it_returned: pool.stream.operation_count(),
        last_record_counter: pool.output.record.counter,
        witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
    }
}

/// 经分配器与不经分配器的每条发布路径各返回一次：暖机的零单元发布（`publish_without_units`）、第一个文件与覆盖写
/// （带单元、经冻结那一层）、抬 F 那一串（先写系统配置、再推空发布）、重开可写挂载（取号、带文件的一版上写行、暖机）、正常卸载
/// （带卸载记号的空发布）。每次返回之后直到流尾的每个崩溃点，系统配置都见证到它。
#[test]
fn every_crash_point_after_a_publish_returns_keeps_a_system_configuration_witness_of_its_last_record(
) {
    let mut pool = build_pool("c577-every-publish-path");
    let mut returned_publishes = vec![
        ReturnedPublish {
            name: "暖机第二次空发布",
            operations_when_it_returned: pool.warm_up_operation_count,
            last_record_counter: pool.warm_up.records.last().expect("暖机两次空发布").counter,
            witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
        },
        ReturnedPublish {
            name: "第一个文件",
            operations_when_it_returned: pool.stream.operation_count(),
            last_record_counter: pool.output.record.counter,
            witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
        },
    ];
    for (seed, name) in [
        (3usize, "覆盖写 txg 4"),
        (5, "覆盖写 txg 5"),
        (7, "覆盖写 txg 6"),
    ] {
        returned_publishes.push(overwrite_and_note_the_return(&mut pool, seed, name));
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(6));
    let mut current = pool.output.clone();
    let raised = raise_rollback_floor(
        &parameters(),
        pool.devices.as_mut().expect("镜像还开着"),
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    )
    .expect("F 抬到 3");
    pool.output = current;
    returned_publishes.push(ReturnedPublish {
        name: "抬 F 那一串的最后一次空发布",
        operations_when_it_returned: pool.stream.operation_count(),
        last_record_counter: raised
            .publishes
            .last()
            .expect("抬 F 至少推一次空发布")
            .record
            .counter,
        witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
    });

    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开之后可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "重开取号 2");
    returned_publishes.push(ReturnedPublish {
        name: "重开可写挂载的最后一次发布",
        operations_when_it_returned: pool.stream.operation_count(),
        last_record_counter: mounted.current.record().counter,
        witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
    });
    let mut allocator = mounted.allocator;
    let mut current_version = mounted.current;
    let unmounted = unmount(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current_version,
        ShadowLedger::On,
    )
    .expect("正常卸载");
    let unmount_publishes = match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised_by_the_unmount) => {
            raised_by_the_unmount.publishes
        }
        Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: version_without_file,
        } => {
            panic!("现行那一版带文件，卸载要推空发布：{version_without_file:?}")
        }
    };
    returned_publishes.push(ReturnedPublish {
        name: "正常卸载的最后一次空发布",
        operations_when_it_returned: pool.stream.operation_count(),
        last_record_counter: unmount_publishes
            .last()
            .expect("卸载至少推一次空发布")
            .record
            .counter,
        witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
    });
    pool.devices = Some(devices);

    assert_eq!(
        witnesses_behind_returned_publishes(&pool.retained_operations(), &returned_publishes),
        Vec::new(),
        "每次发布返回之后的每个崩溃点，系统配置都见证到它末条记录的计数器"
    );
}

/// 树表 0 条的一版上的两条路径：只做过 mkfs 的池第一次可写挂载（零单元写行与暖机），重开再挂一次（树表 0 条的一版上写行，
/// 写实例表与分配记录节点，`publish_instance_table_on_version_without_file`）。每次返回之后直到流尾的每个崩溃点，系统配置都见证到它。
#[test]
fn every_crash_point_after_a_publish_on_a_version_without_file_returns_keeps_the_witness() {
    let mut pool = format_pool("c577-version-without-file");
    let mut returned_publishes = Vec::new();
    {
        let devices = pool.devices.as_mut().expect("mkfs 之后盘还开着");
        let mounted = mount_writable(&parameters(), devices).expect("只做过 mkfs 的池可写挂载");
        assert_eq!(mounted.output.instance, InstanceGeneration(1));
        returned_publishes.push(ReturnedPublish {
            name: "只做过 mkfs 的池第一次可写挂载的最后一次发布",
            operations_when_it_returned: pool.stream.operation_count(),
            last_record_counter: mounted.current.record().counter,
            witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
        });
    }
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开之后可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    assert!(
        mounted.current.file_version().is_none(),
        "一个文件都没发布过，现行那一版仍是树表 0 条的"
    );
    returned_publishes.push(ReturnedPublish {
        name: "树表 0 条的一版上重开可写挂载的最后一次发布",
        operations_when_it_returned: pool.stream.operation_count(),
        last_record_counter: mounted.current.record().counter,
        witness_just_before_the_return: WitnessJustBeforeTheReturn::StillBehindItsLastRecord,
    });
    pool.devices = Some(devices);

    assert_eq!(
        witnesses_behind_returned_publishes(&pool.retained_operations(), &returned_publishes),
        Vec::new(),
        "每次发布返回之后的每个崩溃点，系统配置都见证到它末条记录的计数器"
    );
}

/// 盘 1 上系统配置槽写之后的第一道屏障报块设备错一次（不转给里面的设备、录制流里没有它），之后照转；写、读都照转。
struct FirstBarrierAfterASystemConfigurationWriteFails<Inner: BlockDevice> {
    inner: Inner,
    is_the_failing_device: bool,
    has_written_a_system_configuration_slot_since_the_last_barrier: bool,
    barriers_refused: u64,
}

impl<Inner: BlockDevice> BlockDevice for FirstBarrierAfterASystemConfigurationWriteFails<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let system_configuration_end_in_bytes = SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
            * u64::from(parameters().geometry.fixed_structure_slot_spacing);
        if offset.0 < system_configuration_end_in_bytes {
            self.has_written_a_system_configuration_slot_since_the_last_barrier = true;
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        if self.is_the_failing_device
            && self.has_written_a_system_configuration_slot_since_the_last_barrier
            && self.barriers_refused == 0
        {
            self.barriers_refused += 1;
            return Err(injected_block_device_error("刷盘"));
        }
        self.has_written_a_system_configuration_slot_since_the_last_barrier = false;
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 轮换之后那道屏障报错，与轮换报错同一格（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：这次发布报块设备错、冻结在分配器上，
/// 分配器回到发布之前；原样重发成功之后才返回，返回之后的每个崩溃点系统配置都见证到它。
/// 历史：第一个文件 A（txg 3）→ 覆盖写 B（txg 4），盘 1 上 B 的轮换之后那道屏障报错 → 重发 B。
#[test]
fn a_barrier_failing_after_the_rotation_freezes_the_publish_and_the_resend_returns_only_after_its_rotation_is_durable(
) {
    let mut pool = build_pool("c577-barrier-after-the-rotation-fails");
    let previous = pool.output.clone();
    let mut wrapped: Vec<(
        DeviceIdentity,
        FirstBarrierAfterASystemConfigurationWriteFails<Recorded>,
    )> = pool
        .devices
        .take()
        .expect("镜像还开着")
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FirstBarrierAfterASystemConfigurationWriteFails {
                    inner: device,
                    is_the_failing_device: identity == DeviceIdentity(1),
                    has_written_a_system_configuration_slot_since_the_last_barrier: false,
                    barriers_refused: 0,
                },
            )
        })
        .collect();
    let publish_parameters = parameters();
    let content = content_seeded_by(4100, 3);
    let (failure, resent) = {
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        let failure = publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect_err("盘 1 上轮换之后那道屏障报错：发布不许在轮换持久之前返回成功");
        let frozen = pool
            .allocator
            .frozen_publish()
            .expect("屏障报错与轮换报错同一格：这次发布冻结着等原样重发");
        assert_eq!(
            (frozen.checkpoint_txg(), frozen.instance()),
            (CheckpointTxg(4), InstanceGeneration(1)),
            "冻结的是 B"
        );
        let resent = resend_the_frozen_publish(&mut writer, &mut pool.allocator)
            .expect("屏障只报错一次：重发成功")
            .expect("有冻结着的发布");
        (failure, resent)
    };
    assert!(
        matches!(failure, PublishError::BlockDevice(_)),
        "报的是块设备错：{failure:?}"
    );
    assert!(
        pool.allocator.frozen_publish().is_none(),
        "重发成功之后不再冻结"
    );
    let barriers_refused: Vec<u64> = wrapped
        .iter()
        .map(|(_, device)| device.barriers_refused)
        .collect();
    assert_eq!(barriers_refused, vec![0, 1], "盘 1 拒了恰好一道屏障");
    let resent_output: TransactionOutput =
        resent.into_file_version().expect("重发的是带文件的覆盖写");
    assert_eq!(resent_output.root.checkpoint_txg, CheckpointTxg(4));
    let returned_publishes = [ReturnedPublish {
        name: "轮换之后屏障报错、原样重发的 B",
        operations_when_it_returned: pool.stream.operation_count(),
        last_record_counter: resent_output.record.counter,
        witness_just_before_the_return:
            WitnessJustBeforeTheReturn::AlreadyWitnessedByTheFailedAttempt,
    }];
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    pool.output = resent_output;

    assert_eq!(
        witnesses_behind_returned_publishes(&pool.retained_operations(), &returned_publishes),
        Vec::new(),
        "重发返回之后的每个崩溃点，系统配置都见证到 B 末条记录的计数器"
    );
}
