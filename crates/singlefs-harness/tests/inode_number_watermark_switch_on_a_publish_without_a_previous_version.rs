//! inode 号水位那个只供测试的开关（`PoolWriter::set_inode_number_watermark_source`，`.claude/kb/checks-owed.md` C479）
//! 只给有上一版的发布：新池新建文件没有上一版、水位恒 2，开关装着走到那一次是开关给错了，当场 panic，
//! 而且 panic 在任何落盘动作之前（录制流一步都不多）。产品路径不装开关，不受影响。

mod common;

use std::panic::{catch_unwind, AssertUnwindSafe};

use common::{file_content, format_pool, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::make_filesystem::{
    root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile,
    InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource, PoolWriter,
};

/// mkfs → 取号 → 暖机之后、新池新建文件之前装上压回去的水位，再发新池新建文件：panic 消息说开关给错了，
/// 录制流在这次发布前后一样长（panic 在任何写与屏障之前）。
#[test]
fn a_pressed_back_inode_number_watermark_on_the_new_pool_file_creation_panics_before_any_write() {
    let publish_parameters = parameters();
    let mut formatted = format_pool("inode-number-watermark-switch-on-new-pool-file-creation");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &publish_parameters,
        &formatted.genesis,
    ));
    let content = file_content();
    let genesis_root = formatted.genesis.root;
    let stream = formatted.stream.clone();
    let devices = formatted.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    assert_eq!(instance, InstanceGeneration(1));
    let warmed_up = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
    let pressed_back = InodeNumberWatermarkSource::PressedBackForTests(
        InodeNumberWatermarkPressedBackForTests::new(2).expect("2 不小于第一个 inode 号"),
    );
    writer.set_inode_number_watermark_source(pressed_back);
    assert_eq!(
        writer.inode_number_watermark_source().branch_name(),
        "inode_number_watermark=pressed_back_for_tests",
        "运行时看得出开关装着"
    );
    let operations_before_the_publish = stream.operation_count();

    let outcome = catch_unwind(AssertUnwindSafe(|| {
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed_up.last_record_bytes,
        )
    }));

    let panic_payload = match outcome {
        Ok(published) => panic!(
            "开关装着发新池新建文件应当当场 panic，却交回了 {:?}",
            published.map(|output| output.root.checkpoint_txg)
        ),
        Err(payload) => payload,
    };
    let panic_message = panic_payload
        .downcast_ref::<String>()
        .cloned()
        .expect("panic! 带格式参数，载荷是 String");
    assert!(
        panic_message.contains("开关只给有上一版的发布，装着它走到这里是开关给错了"),
        "panic 说的是开关给错了：{panic_message}"
    );
    assert_eq!(
        stream.operation_count(),
        operations_before_the_publish,
        "panic 在任何写与屏障之前：录制流一步都不多"
    );
}
