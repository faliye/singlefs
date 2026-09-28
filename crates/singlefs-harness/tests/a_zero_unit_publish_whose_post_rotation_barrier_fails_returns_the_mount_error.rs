//! C577（系统配置没见证到的最新根，乙罩不到） 结清 ③：零单元发布（`transaction::publish_without_units`）轮换之后那道屏障报错时，
//! 这次发布不冻结——零单元发布不经分配器，没有「冻结着等原样重发」那一层——整个可写挂载返回错误
//! （D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：由调用方冻结或整个挂载返回错误，射程写成了选择）。
//! 这条用例钉住这一支今天的结局，不改行为：挂载返回哪个错误成员、盘上哪几步已落、之后的可写挂载怎么判。
//!
//! 历史：只做过 mkfs 的池（树表 0 条、上一个实例是 0）第一次可写挂载：取号 1 → 写行那次发布是零单元发布（txg 1，根落盘 1）
//! → 盘 1 上它轮换之后那道屏障报错一次。之后换回不报错的设备、重开再可写挂载一次。
//!
//! 同一份文件还把 C577 结清 ② 的 litmus 绑到代码：`litmus/publish-returns-after-the-rotation-barrier.litmus` 与它的
//! `-nofence` 对照，P0 的写与屏障次序就是录制流里发布最后三步的次序（`.claude/scripts/lkmm.sh` 按文件名认这份测试）。

mod common;

use common::{disk_snapshot, format_pool, geometry, parameters};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::transaction::PublishError;
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::segments::StepKind;

/// 一块盘从上一道屏障起写过什么：发布的最后三步是根槽 FUA → 系统配置槽轮换 → 屏障（`persist_the_root_then_rotate_the_system_configuration`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SinceTheLastBarrier {
    /// 还没有根槽 FUA 写（取号那两次系统配置槽写之后的那道屏障落在这里，不拦）。
    NoRootSlotWritten,
    /// 写过根槽（FUA），还没轮换系统配置。
    RootSlotWrittenForceUnitAccess,
    /// 根槽之后轮换了系统配置：下一道屏障就是轮换之后那一道。
    SystemConfigurationRotatedAfterTheRootSlot,
}

/// 盘 1 上「根槽 FUA → 系统配置槽写」之后的第一道屏障报块设备错一次（不转给里面的设备、录制流里没有它），之后照转；写、读都照转。
struct BarrierAfterTheRotationFailsOnce<Inner: BlockDevice> {
    inner: Inner,
    is_the_failing_device: bool,
    since_the_last_barrier: SinceTheLastBarrier,
    barriers_refused: u64,
}

impl<Inner: BlockDevice> BlockDevice for BarrierAfterTheRotationFailsOnce<Inner> {
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
        let is_a_system_configuration_slot_write = offset.0 < system_configuration_end_in_bytes;
        self.since_the_last_barrier = match (durability, self.since_the_last_barrier) {
            (WriteDurability::ForceUnitAccess, _) => {
                SinceTheLastBarrier::RootSlotWrittenForceUnitAccess
            }
            (WriteDurability::Plain, SinceTheLastBarrier::RootSlotWrittenForceUnitAccess)
                if is_a_system_configuration_slot_write =>
            {
                SinceTheLastBarrier::SystemConfigurationRotatedAfterTheRootSlot
            }
            (
                WriteDurability::Plain,
                unchanged @ (SinceTheLastBarrier::NoRootSlotWritten
                | SinceTheLastBarrier::RootSlotWrittenForceUnitAccess
                | SinceTheLastBarrier::SystemConfigurationRotatedAfterTheRootSlot),
            ) => unchanged,
        };
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
            && self.since_the_last_barrier
                == SinceTheLastBarrier::SystemConfigurationRotatedAfterTheRootSlot
            && self.barriers_refused == 0
        {
            self.barriers_refused += 1;
            return Err(injected_block_device_error("刷盘"));
        }
        self.since_the_last_barrier = SinceTheLastBarrier::NoRootSlotWritten;
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 录制流里 mkfs 之后的每一步：(盘, 步骤种类)。
fn steps_after_the_mkfs(
    operations: &[singlefs_harness::RecordedOperation],
    mkfs_operation_count: usize,
) -> Vec<(DeviceIdentity, StepKind)> {
    operations[mkfs_operation_count..]
        .iter()
        .map(|operation| (operation.device, geometry().classify(operation)))
        .collect()
}

#[test]
fn a_zero_unit_publish_whose_post_rotation_barrier_fails_is_not_frozen_and_the_whole_mount_returns_the_error(
) {
    let mut pool = format_pool("c577-zero-unit-publish-barrier-after-the-rotation-fails");
    let mut wrapped: Vec<(
        DeviceIdentity,
        BarrierAfterTheRotationFailsOnce<common::Recorded>,
    )> = pool
        .devices
        .take()
        .expect("mkfs 之后盘还开着")
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                BarrierAfterTheRotationFailsOnce {
                    inner: device,
                    is_the_failing_device: identity == DeviceIdentity(1),
                    since_the_last_barrier: SinceTheLastBarrier::NoRootSlotWritten,
                    barriers_refused: 0,
                },
            )
        })
        .collect();

    let failure = match mount_writable(&parameters(), &mut wrapped) {
        Err(failure) => failure,
        Ok(mounted) => panic!(
            "盘 1 上零单元发布轮换之后那道屏障报错：挂载不许做成（今天不冻结、不重发），得到实例 {:?}",
            mounted.output.instance
        ),
    };
    let MountError::Publish(publish_sequence_failed) = &failure else {
        panic!("整个挂载返回取号之后那一串发布的失败（MountError::Publish），得到 {failure:?}")
    };
    assert!(
        matches!(publish_sequence_failed.cause, PublishError::BlockDevice(_)),
        "失败的是块设备错：{:?}",
        publish_sequence_failed.cause
    );
    assert!(
        publish_sequence_failed
            .writes_of_persisted_publishes
            .is_empty(),
        "报错的是取号之后第一次发布（写行那次零单元发布），之前没有落盘的发布：{:?}",
        publish_sequence_failed.writes_of_persisted_publishes
    );
    assert_eq!(
        publish_sequence_failed.writes_of_failed_publishes.len(),
        1,
        "落盘阶段失败的恰好一次"
    );
    let barriers_refused: Vec<u64> = wrapped
        .iter()
        .map(|(_, device)| device.barriers_refused)
        .collect();
    assert_eq!(barriers_refused, vec![0, 1], "盘 1 拒了恰好一道屏障");
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );

    // 盘上已落的：取号（两盘各一次系统配置槽写 + 屏障），零单元发布的空记录（两盘镜像）+ 屏障、根槽 FUA（txg 1 → 区域 1 → 盘 1）、
    // 两盘的轮换，再是盘 0 那一道屏障；盘 1 那一道报错、没进录制流。
    let steps = steps_after_the_mkfs(&pool.stream.operations(), pool.mkfs_operation_count);
    assert_eq!(
        steps,
        vec![
            (DeviceIdentity(0), StepKind::SystemConfigurationSlot),
            (DeviceIdentity(1), StepKind::SystemConfigurationSlot),
            (DeviceIdentity(0), StepKind::Barrier),
            (DeviceIdentity(1), StepKind::Barrier),
            (DeviceIdentity(0), StepKind::JournalRecord),
            (DeviceIdentity(1), StepKind::JournalRecord),
            (DeviceIdentity(0), StepKind::Barrier),
            (DeviceIdentity(1), StepKind::Barrier),
            (DeviceIdentity(1), StepKind::RootRecordFua),
            (DeviceIdentity(0), StepKind::SystemConfigurationSlot),
            (DeviceIdentity(1), StepKind::SystemConfigurationSlot),
            (DeviceIdentity(0), StepKind::Barrier),
        ],
        "取号与写行那次零单元发布发到盘上的每一步，止于盘 1 那道报错的屏障之前"
    );
    let snapshot_after_the_failure = disk_snapshot(&pool.memory_pool(), &pool.stream);
    assert!(
        snapshot_after_the_failure
            .readable_roots
            .iter()
            .any(|root| root.instance == InstanceGeneration(1)
                && root.checkpoint_txg == CheckpointTxg(1)),
        "写行那次零单元发布的根（实例 1、txg 1）已落盘：{:?}",
        snapshot_after_the_failure.readable_roots
    );

    // 之后的可写挂载：换回不报错的设备、重开。轮换已写进两块盘（盘 1 那一写没等到屏障，文件镜像上照样在），
    // 系统配置见证到了实例 1 的那条记录，恢复择到实例 1 的根；取号 2。
    let mut devices = pool.reopen_recorded();
    let mounted =
        mount_writable(&parameters(), &mut devices).expect("换回不报错的设备之后可写挂载做得成");
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (
            InstanceGeneration(2),
            InstanceGeneration(1),
            CheckpointTxg(1)
        ),
        "之后的可写挂载取号 2、择到失败那次挂载写行落下的根（实例 1、txg 1）"
    );
    pool.devices = Some(devices);
}

/// litmus 里发布一侧（P0）的写与屏障，按出现的次序认成步骤种类；`returned`（返回的那一刻）不是盘上的一步，不算。
fn litmus_writer_steps_of(litmus_file_name: &str) -> Vec<StepKind> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../litmus")
        .join(litmus_file_name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读不了 {}：{error}", path.display()));
    let writer = text
        .split("P0(")
        .nth(1)
        .expect("litmus 里有 P0")
        .split("\n}")
        .next()
        .expect("P0 有函数体");
    writer
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("smp_wmb()") {
                Some(StepKind::Barrier)
            } else if line.starts_with("WRITE_ONCE(*root") {
                Some(StepKind::RootRecordFua)
            } else if line.starts_with("WRITE_ONCE(*configuration") {
                Some(StepKind::SystemConfigurationSlot)
            } else {
                None
            }
        })
        .collect()
}

/// 录制流里一段步骤不分盘、相邻同类并成一步之后的次序（两块盘各一次轮换写、各一道屏障算一步）。
fn collapsed_step_kinds(operations: &[singlefs_harness::RecordedOperation]) -> Vec<StepKind> {
    let mut collapsed: Vec<StepKind> = Vec::new();
    for operation in operations {
        let kind = geometry().classify(operation);
        if collapsed.last() != Some(&kind) {
            collapsed.push(kind);
        }
    }
    collapsed
}

/// 发布的最后三步与 litmus 的 P0 逐项相同：只做过 mkfs 的池第一次可写挂载，写行那次零单元发布与之后每次暖机空发布返回的那一刻，
/// 录制流末尾是「根槽 FUA → 系统配置槽 → 屏障」；Never 那一条的 P0 就是这三步，`-nofence` 对照写的次序相同、少那一道屏障。
#[test]
fn the_rotation_barrier_litmus_writer_follows_the_last_three_steps_of_every_recorded_publish() {
    let mut pool = format_pool("c577-rotation-barrier-litmus-binding");
    let mounted = {
        let devices = pool.devices.as_mut().expect("mkfs 之后盘还开着");
        mount_writable(&parameters(), devices).expect("只做过 mkfs 的池可写挂载")
    };
    let publishes_in_the_mount = 1 + mounted.output.warm_up_publishes.len();
    let operations = pool.stream.operations();
    let collapsed = collapsed_step_kinds(&operations[pool.mkfs_operation_count..]);
    let last_three_steps_of_a_publish = vec![
        StepKind::RootRecordFua,
        StepKind::SystemConfigurationSlot,
        StepKind::Barrier,
    ];
    let publishes_ending_in_the_last_three_steps = collapsed
        .windows(last_three_steps_of_a_publish.len())
        .filter(|window| *window == last_three_steps_of_a_publish.as_slice())
        .count();
    assert_eq!(
        publishes_ending_in_the_last_three_steps, publishes_in_the_mount,
        "这次挂载的每次发布（写行一次、暖机 {} 次）都以根槽 FUA → 系统配置槽 → 屏障收尾：{collapsed:?}",
        mounted.output.warm_up_publishes.len()
    );
    assert!(
        collapsed.ends_with(&last_three_steps_of_a_publish),
        "挂载返回的那一刻录制流以这三步收尾：{collapsed:?}"
    );
    assert_eq!(
        litmus_writer_steps_of("publish-returns-after-the-rotation-barrier.litmus"),
        last_three_steps_of_a_publish,
        "publish-returns-after-the-rotation-barrier.litmus 的 P0 要与录制流里发布最后三步逐项相同"
    );
    assert_eq!(
        litmus_writer_steps_of("publish-returns-after-the-rotation-barrier-nofence.litmus"),
        vec![StepKind::RootRecordFua, StepKind::SystemConfigurationSlot],
        "对照组写的次序相同、去掉轮换之后那一道屏障"
    );
}
