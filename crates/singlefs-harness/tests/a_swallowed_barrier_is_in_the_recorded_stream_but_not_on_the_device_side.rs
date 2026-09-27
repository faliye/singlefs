//! 里程碑「第二个事务」收尾批「55 号装置包装次序」：`first_transaction_on_device.rs` 的 `CountedDevice`
//! 改回录制器在外、故障注入在里——录制流记的是程序发给这块盘的每一次调用，`FaultInjectingBlockDevice`
//! 转发给它下面那块真设备的调用另记一份数（`barriers_forwarded` / `barriers_swallowed`）。一道被吞的屏障，
//! 调用方发过、录制流里有，转发给真设备的那份计数没涨——这就是「录制流记程序发了什么，设备侧记设备
//! 收到了什么」（D13（验证路线） 已定项 4、D13（验证路线） 已定项 7；`.claude/kb/vm-harness.md`
//! 「设备侧独立录制：blklogwrites 模式」）在最小可行的两层包装上的形态。
//!
//! 这条测试不搭 `CountedDevice`（它是 `first_transaction_on_device.rs` 私有的类型别名，集成测试拿不到）：
//! 直接手搭同样的两层顺序，验的是这层顺序本身的性质，与那个二进制今天用它做什么无关。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::e142_parameters;
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

const SPARSE_DEVICE_BYTES: u64 = 4 << 30;

/// 与 `first_transaction_on_device.rs` 里 `swallow_the_next_barrier_on` 同一条计划：只吞这块盘接下来的第一道屏障，
/// 调用方看不出它被吞了（`barrier()` 照样报成功）。
fn swallow_the_next_barrier_on(device: DeviceIdentity) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::BarrierIsSwallowed,
        device: FaultDeviceSelector::OnlyDevice(device),
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(1),
    }
}

#[test]
fn a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side() {
    let parameters = e142_parameters(512, 512);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let stream = SharedStream::new();
    let plan = SharedFaultPlan::unarmed(geometry);
    let identity = DeviceIdentity(0);
    let mut device = RecordingBlockDevice::with_shared_stream(
        identity,
        FaultInjectingBlockDevice::new(
            identity,
            SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
            plan.clone(),
        ),
        stream.clone(),
    );
    plan.arm(swallow_the_next_barrier_on(identity));

    device
        .barrier()
        .expect("吞掉的屏障也报成功：调用方看不出它被吞了");

    let operations = stream.operations();
    assert_eq!(
        operations.len(),
        1,
        "程序发的这一次屏障调用要进录制流，即使它被下面那层吞掉：{operations:?}"
    );
    assert_eq!(
        operations[0].kind,
        RecordedOperationKind::Barrier,
        "录制流里这一条是屏障，不是别的种类：{operations:?}"
    );

    let counted = plan.counts_of_device(identity);
    assert_eq!(
        counted.barriers_forwarded, 0,
        "被吞的屏障没有转发给它下面那块真设备：设备侧不该看到它"
    );
    assert_eq!(
        counted.barriers_swallowed, 1,
        "故障计划要记下它吞掉了一道屏障"
    );
}
