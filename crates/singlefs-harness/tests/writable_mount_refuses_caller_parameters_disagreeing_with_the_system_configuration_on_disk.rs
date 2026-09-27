//! 代码审阅第 17 条（实审 A1）：可写挂载读与判定用盘上择到的那份系统配置，写入口却拿调用方交进来的 mkfs 参数建——
//! 取号与每次发布末尾的轮换按它写系统配置槽（`transaction::PoolWriter::write_system_configuration_slot` 把 fsid、根环逐区域设备身份、
//! 几何照参数写进系统不可变配置那一段），盘上「mkfs 之后不可改」的那一段（D22（单元原子性怎么合成） 已定项 26 第一档）会被调用方的值改写。
//! 改成：写入口按择到的那一份建，调用方的参数与它逐项比，不一致就在任何写之前拒（`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`），
//! 报出不一致的是哪几项。

mod common;

use common::{build_pool, disk_snapshot, parameters};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable, MakeFilesystemParameterField, MountError};

/// 新池新建文件写完、进程退出之后，拿改过的参数可写挂载：交回挂载的错，与挂载前后两份盘面快照（系统配置槽原样字节、根环里自证过的根、
/// 录制流步数）。之后拿建池的那一份参数再挂一次，做成、取号 2：盘本身挂得上，被拒只因为参数。
fn mount_with_changed_parameters(
    tag: &str,
    change: impl FnOnce(&mut MakeFilesystemParameters),
) -> (MountError, common::DiskSnapshot, common::DiskSnapshot) {
    let mut pool = build_pool(tag);
    let mut devices = pool.reopen_recorded();
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut changed = parameters();
    change(&mut changed);
    let refusal = mount_writable(&changed, &mut devices)
        .expect_err("调用方参数与盘上系统配置不一致，可写挂载必须拒");
    let after = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("建池的那一份参数挂得上：{error:?}"));
    assert_eq!(
        mounted.output.instance,
        InstanceGeneration(2),
        "被拒的那次一个号都没取：这次取 2"
    );
    pool.devices = Some(devices);
    (refusal, before, after)
}

/// 只改 io_min（512 → 4096，槽距仍 4096）：改之前挂载照常做成，取号与轮换把盘上两块盘系统配置里的 io_min 改写成 4096。
/// 现在拒成 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`、只报 io_min 那一项，盘上逐字节不变、录制流一步不多。
#[test]
fn mount_with_a_different_minimum_input_output_size_is_refused_before_any_write_and_names_that_field(
) {
    let (refusal, before, after) =
        mount_with_changed_parameters("a1-parameters-io-min", |changed| {
            changed.geometry.minimum_input_output_bytes = 4096;
        });
    let MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields,
        disagreeing_device_table,
    } = refusal
    else {
        panic!("该报 CallerParametersDisagreeWithTheSelectedSystemConfiguration，实际 {refusal:?}")
    };
    assert_eq!(
        disagreeing_fields,
        vec![MakeFilesystemParameterField::MinimumInputOutputBytes],
        "只有 io_min 不一致"
    );
    assert_eq!(
        disagreeing_device_table,
        Vec::new(),
        "盘表交的就是建池的两块盘：盘数与 devs、本盘设备号都相符"
    );
    assert_eq!(
        after, before,
        "拒在任何写之前：两块盘各两个系统配置槽逐字节不变、根环不变、录制流一步不多"
    );
}

/// fsid 与根环逐区域设备身份一起改（[盘 1, 盘 0, 盘 1]）：拒成同一个成员，按字段表次序报出这两项。改之前写入口拿别的 fsid 验槽、
/// 按别的区域归属落根，取号写把别的 fsid 写进两块盘的系统配置。
#[test]
fn mount_with_another_filesystem_identifier_and_region_devices_is_refused_before_any_write_and_names_both_fields(
) {
    let (refusal, before, after) =
        mount_with_changed_parameters("a1-parameters-fsid-regions", |changed| {
            changed.filesystem_identifier[15] ^= 0xff;
            changed.region_devices = [DeviceIdentity(1), DeviceIdentity(0), DeviceIdentity(1)];
        });
    let MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields,
        disagreeing_device_table,
    } = refusal
    else {
        panic!("该报 CallerParametersDisagreeWithTheSelectedSystemConfiguration，实际 {refusal:?}")
    };
    assert_eq!(
        disagreeing_device_table,
        Vec::new(),
        "盘表交的就是建池的两块盘：盘数与 devs、本盘设备号都相符"
    );
    assert_eq!(
        disagreeing_fields,
        vec![
            MakeFilesystemParameterField::FilesystemIdentifier,
            MakeFilesystemParameterField::RegionDevices,
        ],
        "fsid 与根环逐区域设备身份两项不一致，按字段表次序"
    );
    assert_eq!(after, before, "拒在任何写之前：盘上逐字节不变");
}
