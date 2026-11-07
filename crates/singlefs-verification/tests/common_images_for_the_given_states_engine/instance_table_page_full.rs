//! harness 测试文件 `crates/singlefs-verification/tests/instance_table_page_full.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_given_states_engine/mod.rs`）。那份 harness 测试文件与给定状态那一份
//! `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "引它的测试目标各只调其中一部分")]

use crate::common::{memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_shared_by_harness_scenarios::{
    crash_right_after_acquisition, Devices, DISKS, ROWS_PER_PAGE,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::{InstanceTablePage, InstanceTablePageIndex};
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepConfiguration, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::mount::{
    mount_writable_with_instance_table_sweep_configuration, MountError, Mounted, ShadowLedger,
};
use singlefs_core::recovery::{instance_table_chain_of_root, PoolReader};

use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::scenario::run_new_pool_file_creation;
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 新池新建文件（发布 A，txg 3，实例 1）；录制流只记步数，不留内容。
pub fn pool_and_stream_after_the_new_pool_file_creation() -> (Devices, SharedStream) {
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

/// 可写挂载，实例表清扫取一片装满之后才扫的阈值（369 行，只供测试的清扫配置）；准入、影子账、清扫开关与落点照产品路径。
/// 产品路径的阈值 365（C690（清扫触发阈值的默认值没定））在第 6 次挂载（366 行）起的挂载末尾就扫，根环每槽都自证过的那一次挂载把行删回个位数，
/// 下面那条验收走不到一片写满；清扫被拒（根环有一槽读不出这类）时行数照这样涨，这一形在产品路径上走得到。
pub fn mount_writable_sweeping_only_above_one_full_page(
    devices: &mut Devices,
) -> Result<Mounted, MountError> {
    mount_writable_with_instance_table_sweep_configuration(
        &parameters(),
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        InstanceTableSweepConfiguration {
            switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
            row_count_threshold: InstanceTableRowCountThreshold(
                u64::try_from(ROWS_PER_PAGE).expect("一片的行数装得进 u64"),
            ),
            sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
        },
    )
}

/// 验收：一路可写挂载过一片。先连着 360 次取号之后崩溃把号推到 361（第 1 次挂载一次写 [1, 362) 共 361 行），之后每次挂载写一行：
/// 第 9 次取号 370、写满一片（369 行 + 链指针 = 370 条）；**第 10 次取号 371**（这一版 369 行、要写 1 行）不在取号之前拒绝，
/// 而是写第二片：第 0 片 369 行、第 1 片 1 行，池级 checker 全绿。十次挂载都经 [`mount_writable_sweeping_only_above_one_full_page`]：
/// 行数到 369 为止不扫，第 10 次那 370 行在挂载末尾照清扫（与 C690 的占位阈值 369 行同一个走法）。
pub fn run_the_scenario_where_writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page(
    images: &mut PoolCheckerImages,
) {
    let (mut devices, _stream) = pool_and_stream_after_the_new_pool_file_creation();
    crash_right_after_acquisition(&mut devices, 360);
    for mount_number in 1..=9usize {
        let mounted = mount_writable_sweeping_only_above_one_full_page(&mut devices)
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
    let mounted = mount_writable_sweeping_only_above_one_full_page(&mut devices)
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
