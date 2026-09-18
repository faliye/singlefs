//! 里程碑「第二个事务」步 3 的另一格（2026-09-17 用户定：只做过 mkfs 的池允许可写挂载）：mkfs 之后进程退出、重开两个镜像走可写挂载——
//! 恢复择到 mkfs 的第 0 代根、环里一条记录都没有；取号 1；上一个实例是 0，不写行（D18（块里携带什么信息） 已定项 11）；
//! 树表 0 条 ⇒ 写行那次发布与暖机都写零个单元（D16（发布语义） 已定项 9），txg 1 落盘 1、txg 2 落盘 0；之后在同一个进程里发布第一个文件版本（txg 3）。
//! mkfs 之后的整条录制流与 mkfs 同一个进程里跑第一个事务逐字节相同（第一次可写挂载要写的区间是空的，第一个事务的字节不变）；
//! 冷启动读回那个文件；每一步的镜像池级 checker 一条违例都没有（没有文件的两步里没有记账树与 inode 树的 8 条报不适用）。

mod common;

use common::{
    build_pool, crash_state_devices, disk_snapshot, file_content, format_pool, geometry,
    memory_pool_of_sparse_devices, parameters, FormattedPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{mount_writable, MountError, RollbackTarget};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_without_units, warm_up, FirstFile, PoolVersion,
    PoolWriter, PublishError, ZeroUnitPublishPlan,
};
use singlefs_harness::crash::writes_and_segments;
use singlefs_harness::SharedStream;
use std::cell::Cell;

/// 读超级块槽 0（偏移 0）时按「这块盘第几次读槽 0」注入一次瞬时读错的盘，别的读写原样交给内层（m2-emptypool-nonempty-r1 云端攻方腿
/// 模型 `opus_attack_emptypool.rs` 第 294 行起的做法）。
struct TransientSuperblockReadErrorDevice<Inner: BlockDevice> {
    inner: Inner,
    slot_zero_reads: Cell<u32>,
    failing_slot_zero_read_ordinal: u32,
}

impl<Inner: BlockDevice> BlockDevice for TransientSuperblockReadErrorDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if offset.0 == 0 {
            let ordinal = self.slot_zero_reads.get() + 1;
            self.slot_zero_reads.set(ordinal);
            if ordinal == self.failing_slot_zero_read_ordinal {
                return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                    "注入的瞬时读错",
                )));
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 取号之前判定算出的号与取号写之前重算的号不同，取号不写、报错（m2-emptypool-nonempty-r1 云端攻方腿 Z2）：mkfs 之后第一次可写挂载崩在
/// 取号两写之后（同一个进程里取号就停、镜像关掉；两盘超级块槽 0 已是号 1），重开时两块盘第 2 次读超级块槽 0 各报一次瞬时读错——
/// 判定那一遍只看到 mkfs 的槽 1（号 0）、算出号 1、要写的行区间为空放行；取号重算读到号 1、算出号 2 ⇒ 返回
/// `InstanceGenerationChangedBeforeAcquisition { expected: 1, recomputed: 2 }`，`DiskSnapshot` 不变。
#[test]
fn transient_superblock_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write(
) {
    let mut formatted = format_pool("step-three-formatted-transient-read-error");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        assert_eq!(
            acquire_instance(&mut writer).expect("取号"),
            InstanceGeneration(1)
        );
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let mut devices: Vec<(DeviceIdentity, TransientSuperblockReadErrorDevice<_>)> = formatted
        .reopen_recorded()
        .into_iter()
        .map(|(identity, recorded)| {
            (
                identity,
                TransientSuperblockReadErrorDevice {
                    inner: recorded,
                    slot_zero_reads: Cell::new(0),
                    failing_slot_zero_read_ordinal: 2,
                },
            )
        })
        .collect();
    let refused = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(
        devices
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    assert!(
        matches!(
            refused,
            Err(MountError::InstanceGenerationChangedBeforeAcquisition {
                expected: InstanceGeneration(1),
                recomputed: InstanceGeneration(2),
            })
        ),
        "判定看到号 1、取号重算得号 2：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&formatted.memory_pool(), &formatted.stream),
        before,
        "取号一个字节都没写"
    );
}

/// 没有文件版本的一版上要写实例表行，在取号之前拒绝——只靠「要写的行」这一道判定就拦住的那一格：mkfs 之后第一次可写挂载崩在取号两写之后
/// （盘上只多了号 1，没有记录、没有新根），重开可写挂载：新实例的第一次发布仍是 txg 1、jsn 1，空池挂载的形状判定放行；要取的号 2、
/// 要写的行区间 [1, 2) 不为空 ⇒ 返回 `InstanceRowsOnVersionWithoutFileUnsupported`，`DiskSnapshot` 不变（暖机之后崩溃那一格，
/// 形状判定也会拦，拦不出这道判定挪到取号之后的差别）。
#[test]
fn writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again(
) {
    let mut formatted = format_pool("step-three-formatted-crash-after-acquisition");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        acquire_instance(&mut writer).expect("取号");
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let mut devices = formatted.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(devices);
    assert!(
        matches!(
            refused,
            Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported {
                chosen_root: RollbackTarget {
                    instance: InstanceGeneration(0),
                    checkpoint_txg: CheckpointTxg(0)
                },
                first_row_instance: InstanceGeneration(1),
                instance_to_acquire: InstanceGeneration(2),
            })
        ),
        "号 1 被取过、一条根一条记录都没写出：要写 (1, 0, 0)：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&formatted.memory_pool(), &formatted.stream),
        before,
        "两盘超级块槽逐字节不变、根环没有新根、一个写都没发"
    );
}

/// 空池挂载的形状不是第一个事务那一种，在取号之前拒绝（m2-emptypool-nonempty-r1 云端攻方腿 Z3-A）：mkfs 之后第一次可写挂载崩在
/// 取号两写与 txg 1 的记录两写都持久、txg 1 的根槽没持久；两块盘超级块槽 0（取号写进号 1 的那一槽）各坏一个字节——择超级块只剩 mkfs 的槽 1、
/// 根环只有第 0 代根，要取的号 1、要写的行为空；而环里那条 txg 1 的记录让新实例从 txg 2、jsn 2 起 ⇒ 返回
/// `FormattedPoolMountNotShapedLikeTheFirstTransaction`，整份镜像与录制流都不变。
#[test]
fn formatted_pool_mount_starting_after_a_leftover_record_is_refused_before_acquiring_an_instance() {
    let mut formatted = format_pool("step-three-formatted-leftover-record");
    let mut first_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first_mount_devices).expect("第一次可写挂载");
    formatted.devices = Some(first_mount_devices);
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    let mut persisted = vec![false; writes.len()];
    for write_index in segments[0].iter().chain(segments[1].iter()) {
        persisted[*write_index] = true;
    }
    let crash_stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(
        &formatted.memory_pool_after_mkfs(),
        &writes,
        &persisted,
        &crash_stream,
    );
    for (_, device) in &mut devices {
        let mut sector = vec![0u8; 512];
        device
            .inner()
            .read_at(DeviceOffsetInBytes(0), &mut sector)
            .expect("读超级块槽 0");
        sector[100] ^= 0xff;
        device
            .inner_mut()
            .image
            .write(DeviceOffsetInBytes(0), &sector);
    }
    let image_before = memory_pool_of_sparse_devices(&devices);
    let before = disk_snapshot(&image_before, &crash_stream);
    assert_eq!(
        before
            .readable_roots
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .collect::<Vec<_>>(),
        vec![(InstanceGeneration(0), CheckpointTxg(0)); 3],
        "根环只有三份第 0 代根"
    );
    let refused = mount_writable(&parameters(), &mut devices);
    assert!(
        matches!(
            refused,
            Err(
                MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction {
                    chosen_root: RollbackTarget {
                        instance: InstanceGeneration(0),
                        checkpoint_txg: CheckpointTxg(0)
                    },
                    first_txg: CheckpointTxg(2),
                    first_counter: 2,
                }
            )
        ),
        "环里那条 txg 1 的记录让新实例从 txg 2 起：{:?}",
        refused.as_ref().err()
    );
    let image_after = memory_pool_of_sparse_devices(&devices);
    assert_eq!(disk_snapshot(&image_after, &crash_stream), before);
    assert!(image_after == image_before, "整份镜像逐字节不变");
}

/// `publish_first_file` 写死 txg 3、jsn 3，上一版的记录不是 txg 2、jsn 2 就不写、报错（m2-emptypool-nonempty-r1 云端攻方腿 Z3-A 那一层）：
/// mkfs 之后取号、暖机两次（txg 1、2），再发一次零单元发布（txg 3、jsn 3），接着在这一版上调 `publish_first_file` ⇒ 返回
/// `FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record: Some((3, 3)) }`，录制流一步不多、分配器不动。
#[test]
fn first_file_version_after_a_third_zero_unit_publish_is_refused_before_any_write() {
    let mut formatted = format_pool("step-three-formatted-first-file-after-txg-three");
    let publish_parameters = parameters();
    let genesis_root = formatted.genesis.root;
    let open_devices = formatted.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
    let third = publish_without_units(
        &mut writer,
        warmed.roots.last().expect("暖机两代根"),
        ZeroUnitPublishPlan {
            txg: CheckpointTxg(3),
            counter: 3,
            instance,
            back_chain: back_chain_of(&warmed.last_record_bytes),
            rollback_floor: CheckpointTxg(0),
        },
    )
    .expect("第三次零单元发布");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
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
    let records_before = allocator.records().to_vec();
    let recorded_before = formatted.stream.operations().len();
    let content = file_content();
    let refused = publish_first_file(
        &mut writer,
        &mut allocator,
        &third.root,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &third.record_bytes,
    );
    assert!(
        matches!(
            refused,
            Err(PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp {
                previous_record: Some((CheckpointTxg(3), 3))
            })
        ),
        "上一版是 txg 3、jsn 3：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        formatted.stream.operations().len(),
        recorded_before,
        "一个写、一道屏障都没发"
    );
    assert_eq!(allocator.records(), records_before.as_slice(), "分配器没动");
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 11 条；
/// 其余 18 条要真被评估过且成立。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
/// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_TREE_TABLE`）；
/// I-5.4（分配记录罩住的槽互不相交） 同一个理由：第 0 版树表里没有分配记录树，候选集里一条根的记录都走不到。
const NOT_APPLICABLE_WITHOUT_FILE: [&str; 11] = [
    "I-3.1", "I-3.9", "I-5.2", "I-5.4", "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10", "I-9.13",
    "I-9.14",
];

/// 写完第一个文件版本之后仍报不适用的那一条：树表条目只出现在这一条根的树表里（mkfs 种的第 0 版树表是空的、
/// 两次暖机空发布不写树表），I-9.14 要同一棵树的条目出现在两个树表单元里才比得出来。
/// 比得出来的镜像在 `checker_known_bad_images.rs`（再覆盖写一次、树表两版）。
const NOT_APPLICABLE_WITH_ONE_TREE_TABLE: [&str; 1] = ["I-9.14"];

/// 这一步的镜像上 checker 的 29 条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
fn assert_checker_verdicts(pool: &FormattedPool, step: &str, not_applicable: &[&str]) {
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        verdicts.len(),
        singlefs_checker::image::IMPLEMENTED_INVARIANTS.len()
    );
    for (invariant, verdict) in &verdicts {
        if not_applicable.contains(invariant) {
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "{step}：{invariant} 该报不适用：{verdict:?}"
            );
        } else {
            assert_eq!(
                *verdict,
                InvariantVerdict::Holds,
                "{step}：{invariant} 要真被评估过且成立"
            );
        }
    }
}

/// 没有文件版本的一版上要写实例表行，第一版不支持（设计没定），在取号之前拒绝：第一个事务取号、暖机两次之后、第一个文件版本之前崩溃
/// （同一个进程里做到暖机就停、镜像关掉），重开可写挂载——所选根 (1, 2)、树表 0 条，要取的号 2，要写的行区间 [1, 2) 不为空 ⇒
/// 返回 `InstanceRowsOnVersionWithoutFileUnsupported`；两盘超级块四个槽逐字节不变（实例代号仍是 1）、根环没有新根、录制流一步都没多。
#[test]
fn writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance(
) {
    let mut formatted = format_pool("step-three-formatted-crash-after-warm-up");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        warm_up(&mut writer, &formatted.genesis.root, instance).expect("暖机");
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        before
            .readable_roots
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .max(),
        Some((InstanceGeneration(1), CheckpointTxg(2))),
        "崩溃点：暖机两次的根都在盘上"
    );
    let mut devices = formatted.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(devices);
    assert!(
        matches!(
            refused,
            Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported {
                chosen_root: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(2)
                },
                first_row_instance: InstanceGeneration(1),
                instance_to_acquire: InstanceGeneration(2),
            })
        ),
        "树表 0 条的一版上要写 (1, 2, 0)：{:?}",
        refused.as_ref().err()
    );
    let after = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        after.superblock_slots, before.superblock_slots,
        "两盘超级块槽逐字节不变：没有取号"
    );
    assert_eq!(after.readable_roots, before.readable_roots, "根环没有新根");
    assert_eq!(
        after.recorded_operations, before.recorded_operations,
        "一个写、一道屏障都没发"
    );
}

/// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
/// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
/// mkfs 之后的录制流与第一个事务那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有——mkfs 之后与挂载之后
/// 18 条成立、没有记账树 / inode 树 / 分配记录树的 10 条报不适用，第一个文件版本之后 27 条成立、I-9.14 仍报不适用
/// （树表条目只出现在这一条根的树表里）。
#[test]
fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
) {
    let mut formatted = format_pool("step-three-formatted-pool");
    assert_checker_verdicts(&formatted, "mkfs 之后", &NOT_APPLICABLE_WITHOUT_FILE);

    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    formatted.devices = Some(devices);
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(1),
        "取号 = max(超级块 0, 根环 0) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(0), CheckpointTxg(0)),
        "所选根是 mkfs 的第 0 代根"
    );
    assert_eq!(output.effective_root, output.chosen_root);
    assert_eq!(output.journal.valid_records, 0, "环里一条记录都没有");
    assert!(output.rows_written.is_empty(), "上一个实例是 0：不写行");
    let PoolVersion::WithoutFile(row) = &output.row_publish else {
        panic!("树表 0 条：写行那次发布写零个单元")
    };
    assert_eq!(
        (
            row.root.instance,
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.transaction,
            row.record.back_chain,
            row.record.named.len()
        ),
        (InstanceGeneration(1), CheckpointTxg(1), 1, 0, 0, 0),
        "txg = max(根环 0, 记录无) + 1；jsn 从 1 起；事务号 0；本实例第一条反向链 0；不点名"
    );
    assert_eq!(
        (
            row.root.tree_table,
            row.root.instance_table,
            row.root.mapping_root
        ),
        (
            formatted.genesis.root.tree_table,
            formatted.genesis.root.instance_table,
            formatted.genesis.root.mapping_root
        ),
        "根记录照 mkfs 的根：树表、实例表、映射树根都没换"
    );
    assert_eq!(region_device(1), DeviceIdentity(1), "txg 1 落盘 1");
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "txg 2 落盘 0，一次就覆盖两块盘"
    );
    let PoolVersion::WithoutFile(warm_up) = &output.warm_up_publishes[0] else {
        panic!("树表 0 条：暖机写零个单元")
    };
    assert_eq!(
        (
            warm_up.root.checkpoint_txg,
            warm_up.record.counter,
            warm_up.record.transaction,
            warm_up.record.back_chain
        ),
        (CheckpointTxg(2), 2, 0, back_chain_of(&row.record_bytes))
    );
    assert_eq!(region_device(2), DeviceIdentity(0), "txg 2 落盘 0");
    assert_eq!(
        mounted.current, output.warm_up_publishes[0],
        "接下来的发布接在 txg 2 后面"
    );
    assert_eq!(
        output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "没有被抛弃的根"
    );
    for device_map in &mounted.allocator.devices {
        assert_eq!(
            (device_map.allocated_slots(), device_map.deferred_slots()),
            (3, 0),
            "只有 mkfs 的实例表 2 槽与树表 1 槽"
        );
    }
    assert_checker_verdicts(&formatted, "可写挂载之后", &NOT_APPLICABLE_WITHOUT_FILE);
    assert_eq!(
        recover(&formatted.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2))
        }
    );

    // 第一个文件版本：接在 txg 2 那条零单元发布后面。
    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (
            first.root.instance,
            first.root.checkpoint_txg,
            first.record.counter,
            first.record.transaction,
            first.record.back_chain
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(3),
            3,
            1,
            back_chain_of(current.record_bytes())
        )
    );
    assert_eq!(
        first.data_pointer.locations[0].slot,
        SlotNumber(50180),
        "mkfs 的两个单元占着 50176–50178"
    );
    assert_eq!(
        first.released,
        vec![Placement {
            slot: SlotNumber(50178),
            span: 1
        }],
        "换下 mkfs 那片第 0 版树表"
    );

    let reference = build_pool("step-three-formatted-pool-reference");
    assert_eq!(
        formatted.retained_operations()[formatted.mkfs_operation_count..],
        reference.retained_operations()[reference.mkfs_operation_count..],
        "mkfs 之后的录制流与 mkfs 同一个进程里跑第一个事务逐字节相同"
    );
    assert_eq!(first, reference.output, "写出的那一版也相同");

    assert_checker_verdicts(
        &formatted,
        "第一个文件版本之后",
        &NOT_APPLICABLE_WITH_ONE_TREE_TABLE,
    );
    let reopened = formatted.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content
        }
    );
    assert_eq!(report.journal.valid_records, 3);
}
