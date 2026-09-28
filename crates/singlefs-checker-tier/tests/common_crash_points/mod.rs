//! 崩溃放量共用的流程步骤。几个测试文件调这里的同一个函数，就得到同一个身份、同一串路径段：前缀节点在库里只核一次，
//! 别的流直接复用（用户 2026-09-28 定：前缀共享是剪枝的基础，没有它每次放量都从头核）。
//! 崩溃点的文件是这一份（`file!()`），不是调它的测试文件；判法代码也按这一份的闭包取，改调用方的测试文件不动这里的节点。
#![allow(
    dead_code,
    reason = "共用的步骤模块：每个测试文件只调其中几步，没调的在那个测试二进制里就是没用到"
)]

use super::common::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
use singlefs_core::address::CheckpointTxg;
use singlefs_core::address::InstanceGeneration;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, Mounted,
    RaisedFloor, RollbackTarget, ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::RecordedPublishEntry;

/// 这一份的仓内路径，与 `file!()` 归一之后相同；测试拿它核身份里的文件。
pub const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/common_crash_points/mod.rs";

/// `build_pool` 已经发出去的两段：取号加暖机、新池新建文件。
pub fn record_pool_build(recorder: &mut CrashPointRecorder, pool: &BuiltPool) {
    recorder.record_span(
        file!(),
        "acquire_instance_and_warm_up",
        pool.mkfs_operation_count..pool.warm_up_operation_count,
    );
    recorder.record_span(
        file!(),
        "publish_first_file",
        pool.warm_up_operation_count..pool.stream.operation_count(),
    );
}

/// 覆盖写：把现行那一版的内容换成 `content`。
pub fn overwrite_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    recorder.step(file!(), "publish_overwrite", || {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("覆盖写")
    })
}

/// 关掉镜像再可写挂载（重开：取号、写行、暖机）。
pub fn mount_writable_step(recorder: &mut CrashPointRecorder, pool: &mut BuiltPool) -> Mounted {
    recorder.step(file!(), "mount_writable", || {
        let mut devices = pool.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        pool.devices = Some(devices);
        mounted
    })
}

/// 挂着的时候用一次向前发布回退到 `target`。
pub fn roll_back_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    target: RollbackTarget,
) {
    recorder.step(file!(), "roll_back_by_a_forward_publish", || {
        let rollback_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &rollback_parameters,
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            target,
        )
        .expect("挂着的时候回退");
    });
}

/// 抬回退下界 F 到 `new_floor`：先把新 F 写进每块盘的系统配置，再推带新 F 的空发布直到每块盘上都有一条（D16 已定项 1「抬 F 那一串」）。
pub fn raise_rollback_floor_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    new_floor: CheckpointTxg,
) -> RaisedFloor {
    recorder.step(file!(), "raise_rollback_floor", || {
        let mut current = pool.output.clone();
        let raise_devices = pool.devices.as_mut().expect("镜像还开着");
        let raised = raise_rollback_floor(
            &parameters(),
            raise_devices,
            &mut pool.allocator,
            &mut current,
            new_floor,
            ShadowLedger::On,
        )
        .expect("抬 F");
        pool.output = current;
        raised
    })
}

/// 正常卸载（影子账开着），录制流上把这一段记成卸载入口发的（C557：带卸载记号的根槽写只许落在卸载入口里）：
/// 带文件的一版上卸载必抬 F，交回那一串；现行版本换成那一串最后落盘的那一次。
pub fn unmount_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
) -> UnmountRaisedTheFloor {
    recorder.step(file!(), "unmount", || {
        let mut current = PoolVersion::WithFile(pool.output.clone());
        let stream = pool.stream.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let allocator = &mut pool.allocator;
        let unmounted = stream
            .record_entry(RecordedPublishEntry::Unmount, || {
                unmount(
                    &parameters(),
                    devices,
                    allocator,
                    &mut current,
                    ShadowLedger::On,
                )
            })
            .expect("正常卸载");
        pool.output = current
            .into_file_version()
            .expect("带文件的一版卸载之后仍带文件");
        match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
            Unmounted::NothingWrittenOnAVersionWithoutFile {
                current: reported_version,
            } => {
                panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
            }
        }
    })
}
