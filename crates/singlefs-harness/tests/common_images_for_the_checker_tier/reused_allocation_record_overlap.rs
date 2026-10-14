//! harness 测试文件 `crates/singlefs-harness/tests/reused_allocation_record_overlap.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::run_new_pool_file_creation;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

pub type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 探针的覆盖写内容：长度 3000 + seed % 97、字节 (index × 7 + seed) % 253。
pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 新池新建文件（录制流只记步数，不留内容）。
pub fn pool_after_the_new_pool_file_creation() -> Devices {
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
    devices
}

/// 这块镜像交 checker 档：池级 checker 对 I-5.4 的判定要是「成立」（真被评估过、一处重叠都没有）。别的不变量判不判红不归这条用例
/// （攻方报 I-3.1 从第 6 次挂载起就红、疑似收口表第 ② 行那一族）：checker 档那一侧每张镜像打一行 `HARNESS_SCENARIO_IMAGE`，在那里看。
pub fn hand_out_expecting_the_allocation_records_disjoint(
    images: &mut PoolCheckerImages,
    devices: &Devices,
    after: &str,
) {
    images.hand_out(
        after,
        || memory_pool_of_sparse_devices(devices),
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-5.4"),
    );
}

/// 同一个进程里接着现行版本覆盖写一次。
pub fn overwrite(
    devices: &mut Devices,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    seed: usize,
    instance: singlefs_core::address::InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        allocator,
        previous,
        FirstFile {
            content: &content_of(3000 + seed % 97, seed),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .unwrap_or_else(|error| panic!("第 {seed} 次覆盖写要成立：{error:?}"))
}

pub fn record_at(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    slot: u64,
) -> Option<AllocationRecord> {
    allocator.record_for(device, SlotNumber(slot)).copied()
}

/// txg 38 那次覆盖写换跨度复用的那一槽：它与下一槽上各一条跨 1 的已回收记录，两槽的 inode 叶容器落在它上面。
pub const SPAN_CHANGING_REUSE_SLOT: u64 = 50332;

/// 验收：攻方那条历史走完，第 9、10 次可写挂载都成功；每次挂载与每次覆盖写之后 I-5.4 都成立。txg 38 那次覆盖写正是复用时换跨度的那一次：
/// 之前 50332、50333 两块盘上各一条跨 1 的已释放记录，之后 50332 那条改写成跨 2、代 38，50333 那条删掉。
pub fn reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds(
    images: &mut PoolCheckerImages,
) {
    let mut devices = pool_after_the_new_pool_file_creation();
    hand_out_expecting_the_allocation_records_disjoint(images, &devices, "新池新建文件之后");
    let mut seed = 0usize;
    for mount_number in 1..=8usize {
        let mounted = mount_writable(&parameters(), &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_number} 次可写挂载要成立：{error:?}"));
        let after_mount = format!("第 {mount_number} 次可写挂载之后");
        hand_out_expecting_the_allocation_records_disjoint(images, &devices, &after_mount);
        let mut allocator = mounted.allocator;
        let instance = mounted.output.instance;
        let mut current = mounted
            .current
            .file_version()
            .expect("新池新建文件之后的挂载接在带文件的一版后面")
            .clone();
        let overwrites = if mount_number == 8 { 1 } else { 2 };
        for overwrite_number in 1..=overwrites {
            seed += 1;
            let is_the_span_changing_reuse = mount_number == 8;
            if is_the_span_changing_reuse {
                for device in DISKS {
                    for slot in [SPAN_CHANGING_REUSE_SLOT, SPAN_CHANGING_REUSE_SLOT + 1] {
                        let record = record_at(&allocator, device, slot).unwrap_or_else(|| {
                            panic!("盘 {device:?} 槽 {slot} 在 txg 38 之前有记录")
                        });
                        assert!(
                            record.is_released && record.span_slots == 1,
                            "盘 {device:?} 槽 {slot} 在 txg 38 之前是一条跨 1 的已释放记录：{record:?}"
                        );
                    }
                }
            }
            current = overwrite(&mut devices, &mut allocator, &current, seed, instance);
            let after_overwrite = format!(
                "第 {mount_number} 次挂载之后第 {overwrite_number} 次覆盖写（txg {}）之后",
                current.root.checkpoint_txg.0
            );
            hand_out_expecting_the_allocation_records_disjoint(images, &devices, &after_overwrite);
            if is_the_span_changing_reuse {
                assert_eq!(current.root.checkpoint_txg, CheckpointTxg(38));
                for device in DISKS {
                    let rewritten = record_at(&allocator, device, SPAN_CHANGING_REUSE_SLOT)
                        .expect("换跨度复用的那一槽那条改写了");
                    assert_eq!(
                        (
                            rewritten.span_slots,
                            rewritten.generation,
                            rewritten.is_released
                        ),
                        (2, CheckpointTxg(38), false),
                        "盘 {device:?}：{SPAN_CHANGING_REUSE_SLOT} 那条改写成这次的分配、跨 2"
                    );
                    assert_eq!(
                        record_at(&allocator, device, SPAN_CHANGING_REUSE_SLOT + 1),
                        None,
                        "盘 {device:?}：{} 那条被新记录罩住，随这次分配删掉",
                        SPAN_CHANGING_REUSE_SLOT + 1
                    );
                }
            }
        }
    }
    for mount_number in 9..=10usize {
        mount_writable(&parameters(), &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_number} 次可写挂载要成立：{error:?}"));
        let after_mount = format!("第 {mount_number} 次可写挂载之后");
        hand_out_expecting_the_allocation_records_disjoint(images, &devices, &after_mount);
    }
}
