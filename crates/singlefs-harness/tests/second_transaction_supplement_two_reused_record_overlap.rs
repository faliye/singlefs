//! 里程碑「第二个事务」增补 2 收口，代码三方第二轮判决（`research/prompts/m2-wave2-code-r1-main-verification.md`）第二节第 1 行（Z1-d）：
//! 复用一个已回收的落点时跨度变了——两槽的单元落在一条跨 1 的已回收记录上，下一槽另一条跨 1 的已回收记录留着 ⇒ 盘上两条分配记录
//! 罩住同一个槽，下一次可写挂载重建分配器时在 `mark_allocated` 的断言上 panic（取号之前），池从此挂不上可写。
//! 改法：一次分配写下的记录罩住的槽上，别的已回收记录随这次分配删掉（`allocator.rs` 的 `make_room_for_record_on_device`）；
//! checker 新接 I-5.4（分配记录罩住的槽互不相交），每次发布之后判。
//!
//! 历史照攻方原样探针（`research/prompts/m2-wave2-code-r1-opus-model/opus_probe_pristine.rs` 的
//! `pristine_reuse_with_a_different_span_leaves_an_overlapping_record_and_the_next_mount_panics`，日志 `pristine-run.log`）：
//! 两块 4 GiB 内存盘，mkfs → 第一个文件 →「可写挂载 → 覆盖写 2 次」× 7 → 第 8 次可写挂载 → 覆盖写 1 次（txg 38，inode 叶容器
//! 两槽落在 50320，50320 与 50321 上各一条跨 1 的已回收记录）→ 第 9、10 次可写挂载。覆盖写的内容与写入时刻照探针取。
//! 改之前：txg 38 那次之后 50320 跨 2 与 50321 跨 1 两条记录重叠，第 9 次挂载在 `allocator.rs` 的 `mark_allocated` 断言上 panic。
//! 分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）每次发布多写几个节点，落点整体后移：同一条历史里换跨度的复用仍在 txg 38，
//! 落在 50332（50332 与 50333 上各一条跨 1 的已回收记录，草稿副本上沿这条历史逐次覆盖写查出来的）。

mod common;

use common::{memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::scenario::run_first_transaction;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 探针的覆盖写内容：长度 3000 + seed % 97、字节 (index × 7 + seed) % 253。
fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个事务（录制流只记步数，不留内容）。
fn pool_after_the_first_transaction() -> Devices {
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
    run_first_transaction(&parameters(), &mut devices, &stream, |_, _| {}).expect("第一个事务");
    devices
}

/// 这块镜像上 checker 对 I-5.4 的判定要是「成立」（真被评估过、一处重叠都没有）；顺带交回别的判红的不变量（不在这条用例里断言）。
fn assert_allocation_records_disjoint(devices: &Devices, after: &str) -> Vec<&'static str> {
    let verdicts = check_pool_image(&memory_pool_of_sparse_devices(devices));
    let disjointness = verdicts
        .iter()
        .find(|(invariant, _)| *invariant == "I-5.4")
        .expect("清单里有 I-5.4")
        .1
        .clone();
    assert_eq!(
        disjointness,
        InvariantVerdict::Holds,
        "{after}：I-5.4（分配记录罩住的槽互不相交） 要真被评估过且成立"
    );
    verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect()
}

/// 同一个进程里接着现行版本覆盖写一次。
fn overwrite(
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

fn record_at(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    slot: u64,
) -> Option<AllocationRecord> {
    allocator.record_for(device, SlotNumber(slot)).copied()
}

/// txg 38 那次覆盖写换跨度复用的那一槽：它与下一槽上各一条跨 1 的已回收记录，两槽的 inode 叶容器落在它上面。
const SPAN_CHANGING_REUSE_SLOT: u64 = 50332;

/// 验收：攻方那条历史走完，第 9、10 次可写挂载都成功；每次挂载与每次覆盖写之后 I-5.4 都成立。txg 38 那次覆盖写正是复用时换跨度的那一次：
/// 之前 50332、50333 两块盘上各一条跨 1 的已释放记录，之后 50332 那条改写成跨 2、代 38，50333 那条删掉。
#[test]
fn reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_leaves_no_overlap_and_the_ninth_mount_succeeds(
) {
    let mut devices = pool_after_the_first_transaction();
    let mut other_red_invariants: Vec<(String, Vec<&'static str>)> = Vec::new();
    other_red_invariants.push((
        "第一个事务之后".to_string(),
        assert_allocation_records_disjoint(&devices, "第一个事务之后"),
    ));
    let mut seed = 0usize;
    for mount_number in 1..=8usize {
        let mounted = mount_writable(&parameters(), &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_number} 次可写挂载要成立：{error:?}"));
        let after_mount = format!("第 {mount_number} 次可写挂载之后");
        other_red_invariants.push((
            after_mount.clone(),
            assert_allocation_records_disjoint(&devices, &after_mount),
        ));
        let mut allocator = mounted.allocator;
        let instance = mounted.output.instance;
        let mut current = mounted
            .current
            .file_version()
            .expect("第一个事务之后的挂载接在带文件的一版后面")
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
            other_red_invariants.push((
                after_overwrite.clone(),
                assert_allocation_records_disjoint(&devices, &after_overwrite),
            ));
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
        other_red_invariants.push((
            after_mount.clone(),
            assert_allocation_records_disjoint(&devices, &after_mount),
        ));
    }
    // 别的不变量在这条历史上判不判红不归这条用例（攻方报 I-3.1 从第 6 次挂载起就红、疑似收口表第 ② 行那一族）：只打出来。
    for (after, red) in &other_red_invariants {
        if !red.is_empty() {
            println!("OTHER_RED {after}: {red:?}");
        }
    }
}
