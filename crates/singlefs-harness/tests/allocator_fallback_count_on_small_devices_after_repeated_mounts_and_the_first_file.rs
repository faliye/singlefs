//! 分配器回落那一支的看门（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「开放段回落误排」那一行）：
//! 池级 checker 的 I-5.5 拿树表单元所在的段当这一版的开放段，假定「mkfs 之后连续可写挂载（每次挂载写行、暖机两次零单元发布）、
//! 再写第一个文件」那一族上提交内生块不走回落（`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`）。
//! 单元区 240 / 256 / 384 槽的小盘、挂载 1、12、60 次，各一条用例断言分配器的回落计数
//! （`PoolAllocator::commit_generated_placements_that_fell_back_to_the_lowest_free_slot`）是 0；哪天走进回落，用例红，回来重看 I-5.5 的开放段认法。
//! 阳性对照照 `commit_generated_units_fall_back_when_the_segment_runs_out.rs` 的造法（没有全空段、开放段用满）断言计数大于 0。

mod common;

use common::{build_pool, parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionSwitches, EmptyClusterSegmentsFloor};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable_with_space_admission, Mounted};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, UNIT_AREA_START_SLOT};
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
    HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::SharedStream;

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// mkfs 之后的整份盘面（随机历史的执行器，起点 `AfterMakeFilesystem`、一步都不走）。
fn pool_after_make_filesystem(width: HistoryDeviceWidth) -> MemoryPool {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: Vec::new(),
    };
    execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |_observation| {},
    );
    let mut pool = MemoryPool::with_devices(&DEVICES, width.device_bytes());
    pool.apply(&stream.retained_operations());
    pool
}

fn devices_of(
    pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    pool.devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                pool.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

fn pool_of(
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes,
    }
}

/// 第一个文件的内容（装得进一个数据单元）。
fn first_file_content() -> Vec<u8> {
    (0..200u32)
        .map(|index| u8::try_from(index % 251).expect("小于 251") ^ 0x3c)
        .collect()
}

/// mkfs 之后连续可写挂载 `mounts` 次（每次挂载写行、暖机两次零单元发布；前几次挂上就关、不卸载），最后那次挂载之后写第一个文件：
/// 交回每次挂载交回的分配器（带着挂载里那几次发布）与最后那次写第一个文件的分配器一共走了几次回落。
fn fallbacks_over_mounts_then_the_first_file(width: HistoryDeviceWidth, mounts: usize) -> u64 {
    assert!(mounts >= 1, "至少挂载一次才写得了第一个文件");
    let parameters = width.parameters();
    let mut pool = pool_after_make_filesystem(width);
    let mut fallbacks = 0u64;
    for mount_index in 1..=mounts {
        let mut devices = devices_of(&pool, &parameters);
        let Mounted {
            output,
            mut allocator,
            current,
        } = mount_writable_with_space_admission(
            &parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
        )
        .unwrap_or_else(|error| panic!("第 {mount_index} 次可写挂载该成：{error:?}"));
        if mount_index == mounts {
            let PoolVersion::WithoutFile(current) = &current else {
                panic!("这一族一直没写文件，现行那一版是树表 0 条的一版：{current:?}");
            };
            let content = first_file_content();
            let mut writer = PoolWriter::new(&parameters, &mut devices);
            publish_first_file(
                &mut writer,
                &mut allocator,
                &current.root,
                FirstFile {
                    content: &content,
                    write_time_seconds: 1_000,
                },
                output.instance,
                &current.record_bytes,
            )
            .expect("最后那次挂载之后写第一个文件该成");
        }
        fallbacks += allocator.commit_generated_placements_that_fell_back_to_the_lowest_free_slot();
        pool = pool_of(devices, pool.device_size_in_bytes);
    }
    fallbacks
}

fn assert_no_fallback(width: HistoryDeviceWidth, mounts: usize) {
    assert_eq!(
        fallbacks_over_mounts_then_the_first_file(width, mounts),
        0,
        "{width:?} 上挂载 {mounts} 次再写第一个文件，提交内生块一次都不该走回落（I-5.5 的开放段认法压在这上面）"
    );
}

#[test]
fn no_fallback_on_a_unit_area_of_240_slots_after_one_mount_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf240Slots, 1);
}

#[test]
fn no_fallback_on_a_unit_area_of_240_slots_after_twelve_mounts_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf240Slots, 12);
}

#[test]
fn no_fallback_on_a_unit_area_of_240_slots_after_sixty_mounts_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf240Slots, 60);
}

#[test]
fn no_fallback_on_a_unit_area_of_256_slots_after_one_mount_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf256Slots, 1);
}

#[test]
fn no_fallback_on_a_unit_area_of_256_slots_after_twelve_mounts_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf256Slots, 12);
}

#[test]
fn no_fallback_on_a_unit_area_of_256_slots_after_sixty_mounts_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf256Slots, 60);
}

#[test]
fn no_fallback_on_a_unit_area_of_384_slots_after_one_mount_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf384Slots, 1);
}

#[test]
fn no_fallback_on_a_unit_area_of_384_slots_after_twelve_mounts_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf384Slots, 12);
}

#[test]
fn no_fallback_on_a_unit_area_of_384_slots_after_sixty_mounts_and_the_first_file() {
    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf384Slots, 60);
}

/// 每块盘上把开放段剩下的槽占满，再给每个全空段占掉段首一槽（`commit_generated_units_fall_back_when_the_segment_runs_out.rs` 同名函数的造法）。
fn leave_no_empty_cluster_segment(allocator: &mut PoolAllocator) {
    let open = allocator.open_segment().expect("发布之后开放段还开着");
    for device_map in &mut allocator.devices {
        for slot in open.0..open.0 + CLUSTER_SEGMENT_SLOTS {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
        let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
        for segment_index in 0..full_segments {
            if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
                device_map.mark_allocated(
                    SlotNumber(UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS),
                    1,
                );
            }
        }
    }
}

/// 整理开关取产品值，只把合取 7 的 `N₀` 压成 0（判决 G6「强制合取 7 的 `N₀`」）：没有全空段的池上前台写入不被合取 7 拦，
/// 摸得到回落那一支（产品值下这一形被合取 7 拒，钉在 `commit_generated_units_fall_back_when_the_segment_runs_out.rs` 的
/// `an_overwrite_on_a_pool_without_an_empty_cluster_segment_is_refused_by_the_empty_cluster_segments_floor_by_default`）。
fn compaction_switches_with_the_empty_cluster_segments_floor_at_zero() -> CompactionSwitches {
    let product = CompactionSwitches::PRODUCT;
    CompactionSwitches {
        batch_size: product.batch_size,
        forced_region: product.forced_region,
        run_forcing: product.run_forcing,
        region_exclusion: product.region_exclusion,
        destination_policy: product.destination_policy,
        release_checksum_mismatch_after_issue: product.release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::ForcedForTests(0),
        relocation_publish_order: product.relocation_publish_order,
    }
}

/// 阳性对照：新池新建文件之后没有全空段、开放段用满，覆盖写一次——提交内生块走回落，计数大于 0（那一次落点钉在
/// `commit_generated_units_fall_back_when_the_segment_runs_out.rs` 里）。判别力：回落那一支不加计数时这一条红。
/// 合取 7 的 `N₀` 压成 0，只测回落那一支（[`compaction_switches_with_the_empty_cluster_segments_floor_at_zero`]）。
#[test]
fn the_fallback_count_is_above_zero_once_no_empty_cluster_segment_is_left() {
    let mut pool = build_pool("allocator-fallback-count-positive-control");
    assert_eq!(
        pool.allocator
            .commit_generated_placements_that_fell_back_to_the_lowest_free_slot(),
        0,
        "前提：新池新建文件那一次没走回落"
    );
    leave_no_empty_cluster_segment(&mut pool.allocator);
    pool.allocator.set_compaction_switches(
        compaction_switches_with_the_empty_cluster_segments_floor_at_zero(),
    );
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("没有全空段、每块盘仍有空槽：覆盖写要成功（C369）");
    }
    assert!(
        pool.allocator
            .commit_generated_placements_that_fell_back_to_the_lowest_free_slot()
            > 0,
        "没有全空段时提交内生块走回落，计数要大于 0"
    );
}
