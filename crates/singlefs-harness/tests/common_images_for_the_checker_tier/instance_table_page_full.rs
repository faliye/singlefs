//! harness 测试文件 `crates/singlefs-harness/tests/instance_table_page_full.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::{InstanceTablePage, InstanceTablePageIndex};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{instance_table_chain_of_root, PoolReader};
use singlefs_core::transaction::{acquire_instance, PoolWriter};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::run_new_pool_file_creation;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 一片实例表里数据行的上限：370 条记录减去链指针那一条。
pub const ROWS_PER_PAGE: usize = 369;

pub type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 新池新建文件（发布 A，txg 3，实例 1）；录制流只记步数，不留内容。
pub fn pool_after_the_new_pool_file_creation() -> (Devices, SharedStream) {
    let stream = SharedStream::new();
    let mut devices: Devices = DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    run_new_pool_file_creation(&parameters(), &mut devices, &stream, |_, _| {})
        .expect("新池新建文件");
    (devices, stream)
}

/// 连着 `count` 次「取号之后崩溃」：每次取一个新号写进两块盘的系统配置，写行那次发布没发出去。
pub fn crash_right_after_acquisition(devices: &mut Devices, count: u32) {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    for _ in 0..count {
        acquire_instance(&mut writer).expect("取号");
    }
}

/// 写行那次发布写出的实例表每一片的行数，按链上的次序：从盘上沿链读（根记录指着第 0 片）。
pub fn rows_of_each_page_of_the_row_publish(devices: &Devices, mounted: &Mounted) -> Vec<usize> {
    let chain = instance_table_chain_of_root(devices, mounted.output.row_publish.root())
        .expect("写出的实例表沿链读得出、解得开");
    chain
        .page_pointers
        .iter()
        .enumerate()
        .map(|(position, pointer)| {
            let location = pointer.locations[0];
            let bytes = PoolReader::read(
                devices.as_slice(),
                location.device,
                location.slot.to_device_offset(),
                32768,
            )
            .expect("那一片读得到");
            InstanceTablePage::parse(
                &bytes,
                InstanceTablePageIndex(u64::try_from(position).expect("片序号")),
            )
            .expect("那一片按第几片解得开")
            .rows
            .len()
        })
        .collect()
}

/// 验收：一路可写挂载过一片。先连着 360 次取号之后崩溃把号推到 361（第 1 次挂载一次写 [1, 362) 共 361 行），之后每次挂载写一行：
/// 第 9 次取号 370、写满一片（369 行 + 链指针 = 370 条）；**第 10 次取号 371**（这一版 369 行、要写 1 行）在改之前于取号之前拒绝，
/// 现在写第二片：第 0 片 369 行、第 1 片 1 行，池级 checker 全绿。
pub fn writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page(
    images: &mut PoolCheckerImages,
) {
    let (mut devices, _stream) = pool_after_the_new_pool_file_creation();
    crash_right_after_acquisition(&mut devices, 360);
    for mount_number in 1..=9usize {
        let mounted = mount_writable(&parameters(), &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_number} 次可写挂载要成立：{error:?}"));
        assert_eq!(
            mounted.output.instance.0,
            u32::try_from(361 + mount_number).expect("号"),
            "第 {mount_number} 次挂载取的号"
        );
        assert_eq!(
            rows_of_each_page_of_the_row_publish(&devices, &mounted),
            vec![360 + mount_number],
            "第 {mount_number} 次挂载写出的表：一片"
        );
    }
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("第 10 次可写挂载（取号 371）要成立：{error:?}"));
    assert_eq!(mounted.output.instance, InstanceGeneration(371));
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&devices, &mounted),
        vec![ROWS_PER_PAGE, 1],
        "第 0 片写满 369 行、第 1 片装剩下的 1 行"
    );
    images.hand_out(
        "第 10 次可写挂载写出第二片之后",
        || memory_pool_of_sparse_devices(&devices),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
