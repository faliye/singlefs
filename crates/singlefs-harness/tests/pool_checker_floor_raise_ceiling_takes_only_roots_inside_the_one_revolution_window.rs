//! 池级 checker 的 I-7.9（回退下界 F 不高于抬 F 的上限） 两支（不带卸载记号的准入抬 F、带卸载记号的正常卸载那一串）：「之前的根」只取一圈窗口里的根
//! （`.claude/kb/invariants.md` I-7.9 那一行 2026-10-04 写回，照 D16（发布语义） 已定项 1「抬 F 的上限」只在一圈窗口里的根上取）。
//! 窗口取两个、一起用：锚在被判那条抬 F 的根上的（条款字面），与锚在这张镜像根环里最新有效根上的（候选过滤那一个）；只取前一个时下面三形
//! 合法镜像照样红（`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings` 写了理由）。
//! 用户 2026-10-04 定 RGall 之后抬 F 的上限本身取整个环，前一个窗口拿掉（镜像最新根不早于被判那条根，后一个窗口本来就罩着它），
//! 只剩锚在镜像最新根上的那一个；文件名与用例名留着两个窗口那一轮的叫法。
//! 环上有洞时洞那一槽留着上一圈的根，后来的发布盖掉的是比它新的根：在后来那张镜像上重算的上限不再「只会更宽」。三形合法镜像，抬 F 那一刻
//! 实现与 checker 自己的公式算出同一个上限、F 都等于它，再挂之后洞里的旧根顶上来：
//! - 删除与并发那条线交来的草稿：卸载那一串空发布记录已落、根没落那一形的最小复现（覆盖写切在「记录已落、根没落」再可写挂载）；
//! - 里程碑三第 8 批调查的 inject-6 种子 1：随机历史（比重「抬 F 之后复用」、4 GiB、20 步、种子 1）里实例 5 第二次发布 txg 25 的根槽写被扣下、
//!   记录已落（由记录施加出来那一类洞），再可写挂载加一次发布；
//! - 同一段历史里实例 6 写行那次发布 txg 26 的根槽写被扣下（K1 类洞：那一版没被施加、取号越过它），再可写挂载加一次发布。
//!
//! 另一份坏镜像：洞上那张镜像里把那条抬 F 的根带的 F 改得高于窗口里的根算出的上限，I-7.9 照旧红。
//!
//! 带卸载记号那一支取同一个交（用户 2026-10-04 定「乙，两支都取两个窗口」）：正常卸载之后再挂、覆盖写到一圈之后，
//! 一次覆盖写切在「记录已落、根没落」，再可写挂载——洞那一槽留着卸载前两版的那条根，卸载前最后一版的根被盖掉；
//! 只取锚在被判那条根上的窗口时「txg 比它小的最新那条根」成了那条旧根、F 高于它而误红，取两个窗口的交判绿。
//! 坏镜像：同一段历史挂载之前那张镜像上把卸载那一串第一条根带的 F 改高一，照旧红在带卸载记号那一支。
//! 这几份都不经 checker 档：崩溃状态只取一个（扣下的那一个根槽写之前全落），不枚举。

mod common_admission;

use common_admission::*;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::{mount_writable_with_space_admission, unmount, ShadowLedger, Unmounted};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
};
use singlefs_core::root_ring::slot_offset;
use singlefs_core::transaction::{
    publish_first_file, publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    root_identity_of_write, root_identity_written_by, writes_and_segments, MemoryPool,
    SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};
use std::collections::BTreeSet;

/// 根记录里回退下界 F 的偏移：树表指针 36 起 86、映射根指针之前（D22（单元原子性怎么合成） 已定项 7 的偏移表，池级 checker 读的那一处）。
const ROOT_RECORD_ROLLBACK_FLOOR_OFFSET: usize = 36 + 86 + 8;
/// 根记录的自证校验和在 138 起 32 字节，罩整个 512 字节的根槽（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_SLOT_BYTES: usize = 512;
/// 草稿那段历史：第一个文件之后覆盖写 3 次、崩了再挂一次，再覆盖写 16 次；第 17 次覆盖写切在「记录已落、根没落」。
const OVERWRITES_BEFORE_THE_CRASH_REMOUNT: usize = 3;
const OVERWRITES_AFTER_THE_CRASH_REMOUNT: usize = 16;
/// 草稿那一次切下的发布的 txg（它的根槽一直没写，留着上一圈的根）。
const TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT: u64 = 31;
/// inject-6 那段随机历史：比重「抬 F 之后复用」、20 步、种子 1、4 GiB 两盘。
const INJECT_SIX_HISTORY_OPERATIONS: usize = 20;
const INJECT_SIX_HISTORY_SEED: u64 = 1;
/// 那段历史里抬 F 的根：实例 1 txg 9 把 F 从 0 抬到 5。
const INJECT_SIX_RAISING_ROOT_TXG: u64 = 9;
/// 扣下的根槽写：实例 5 txg 25（由记录施加出来那一类洞，占洞的是 txg 1）、实例 6 txg 26（写行那次，K1 类洞，占洞的是 txg 2）。
const INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD: (u32, u64) = (5, 25);
const INJECT_SIX_WITHHELD_ROOT_OF_THE_ROW_PUBLISH: (u32, u64) = (6, 26);
/// 带卸载记号那一形：卸载之前覆盖写几次。
const OVERWRITES_BEFORE_THE_UNMOUNT: usize = 3;
/// 挂载之后那一次发布的内容长度：一个数据单元装得下。
const CONTENT_BYTES_OF_THE_PUBLISH_AFTER_THE_MOUNT: usize = 3000;
/// 挂载之后那一次发布的写入时间。
const WRITE_TIME_SECONDS_OF_THE_PUBLISH_AFTER_THE_MOUNT: u64 = 1_788_000_000;

fn rollback_floor_verdict(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == "I-7.9")
        .map(|(_, verdict)| verdict)
        .expect("池级 checker 判 I-7.9")
}

/// 镜像根环里自证过的根：txg 的集合。
fn root_txgs_in_the_ring(image: &MemoryPool) -> BTreeSet<u64> {
    let immutable = choose_system_configuration(image)
        .expect("系统配置读得出")
        .immutable;
    every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                image,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some(root.checkpoint_txg.0),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

/// 把根环里 txg 是 `txg` 的那条根带的 F 改成 `floor`、重封根槽自证校验和（只动根槽：同 (实例, txg) 的记录不跟着改，I-8.10 由它说话）。
fn rewrite_the_floor_of_the_root_at(image: &mut MemoryPool, txg: u64, floor: u64) {
    let immutable = choose_system_configuration(&*image)
        .expect("系统配置读得出")
        .immutable;
    let ring_slot = every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .find(|ring_slot| {
            matches!(
                read_root_ring_slot(
                    &*image,
                    &immutable.region_devices,
                    &immutable.sizes,
                    &immutable.filesystem_identifier,
                    *ring_slot,
                ),
                RootRingSlotReading::SelfVerified(root) if root.checkpoint_txg.0 == txg
            )
        })
        .unwrap_or_else(|| panic!("根环里有 txg {txg} 的根"));
    let device =
        immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
    let offset = slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing);
    let sectors = image.devices.get_mut(&device).expect("根环区域所在的盘");
    let mut bytes = sectors.read(offset, ROOT_SLOT_BYTES);
    bytes[ROOT_RECORD_ROLLBACK_FLOOR_OFFSET..ROOT_RECORD_ROLLBACK_FLOOR_OFFSET + 8]
        .copy_from_slice(&floor.to_le_bytes());
    let digest =
        wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_RECORD_CHECKSUM_OFFSET);
    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    sectors.write(offset, &bytes);
}

/// 一张崩溃镜像上可写挂载（准入照式子判），挂上之后再发一次（有文件就覆盖写、没有就写第一个文件）：交回之后的镜像。
fn image_after_a_writable_mount_and_one_publish(
    crashed: &MemoryPool,
    width: HistoryDeviceWidth,
) -> MemoryPool {
    let parameters = width.parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = crashed
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                crashed.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let mut mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("崩溃镜像上可写挂载做得成");
    let instance = mounted.output.instance;
    let content: Vec<u8> = (0..CONTENT_BYTES_OF_THE_PUBLISH_AFTER_THE_MOUNT)
        .map(|index| u8::try_from(index % 251).expect("小于 251"))
        .collect();
    let file = FirstFile {
        content: &content,
        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_PUBLISH_AFTER_THE_MOUNT,
    };
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        match &mounted.current {
            PoolVersion::WithFile(current) => {
                publish_overwrite(&mut writer, &mut mounted.allocator, current, file, instance)
                    .expect("挂载之后覆盖写一次");
            }
            // 删完那一版（D8（核心索引结构） 已定项 16 第 5 条）：之后只许建 inode、写行与空发布，建一个 size 0 的 inode。
            PoolVersion::WithTreesAndNoFile(current) => {
                publish_new_inodes(
                    &mut writer,
                    &mut mounted.allocator,
                    current,
                    1,
                    WRITE_TIME_SECONDS_OF_THE_PUBLISH_AFTER_THE_MOUNT,
                    instance,
                )
                .expect("挂载之后建一个 inode");
            }
            PoolVersion::WithoutFile(_) => {
                publish_first_file(
                    &mut writer,
                    &mut mounted.allocator,
                    mounted.current.root(),
                    file,
                    instance,
                    mounted.current.record_bytes(),
                )
                .expect("挂载之后写第一个文件");
            }
        }
    }
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: crashed.device_size_in_bytes,
    }
}

/// inject-6 那段随机历史录下来，崩在 (实例, txg) = `withheld` 那条根的根槽写上：那一段只有这一个写、它没落，之前的写全落。
fn inject_six_history_crashed_on_the_root_slot_write_of(withheld: (u32, u64)) -> MemoryPool {
    let width = HistoryDeviceWidth::FourGibibytes;
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let history = generate_history_with_weights(
        HistorySeed(INJECT_SIX_HISTORY_SEED),
        INJECT_SIX_HISTORY_OPERATIONS,
        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
    );
    let stream = SharedStream::retaining_contents();
    let _run = execute_history_with(&history, execution, &stream, &mut |_| {});
    let (writes, segments) =
        writes_and_segments(&stream.retained_operations(), &width.fixed_geometry());
    let segment = segments
        .iter()
        .find(|segment| {
            segment.iter().any(|write| {
                writes[*write].kind == StepKind::RootRecordFua && {
                    let (instance, txg) = root_identity_of_write(&writes[*write]);
                    (instance, txg) == (InstanceGeneration(withheld.0), CheckpointTxg(withheld.1))
                }
            })
        })
        .unwrap_or_else(|| panic!("这段历史写过实例 {} txg {} 的根槽", withheld.0, withheld.1));
    assert_eq!(segment.len(), 1, "扣下的那一段只有那一个根槽写");
    let mut crashed = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        width.device_bytes(),
    );
    crashed.apply_writes(&writes[..segment[0]]);
    crashed
}

/// 草稿用例（删除与并发那条线交来，`a_remount_over_a_root_ring_hole_reddens_an_old_floor_raise`）翻成判绿：单元区 240 槽的小盘，第一个文件之后
/// 覆盖写 3 次、崩了再挂一次（准入判着，这次挂载里有一条根把 F 从 0 抬到 9），再覆盖写 16 次；第 17 次覆盖写切在「journal 记录已落、txg 31
/// 那条根没落」，可写挂载一次。挂载把 txg 31 那一版当生效版（从 journal 施加），新实例的根从 txg 32 起写，txg 31 那个根槽一直没写、留着上一圈
/// 的根，txg 32 起的根盖掉了比它新的几条。挂载前后 I-7.9 都不许红：抬 F 那一刻 F 不高于上限，后来的镜像上从中间少了的根不能把它判成越界。
#[test]
fn a_remount_over_a_root_ring_hole_left_by_a_journal_applied_version_keeps_an_old_floor_raise_within_its_ceiling(
) {
    let stream = SharedStream::retaining_contents();
    let wrap = stream.clone();
    let mut pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>> =
        PoolUnderTest::start_after_the_first_file(
            HistoryDeviceWidth::UnitAreaOf240Slots,
            move |identity, device| {
                RecordingBlockDevice::with_shared_stream(identity, device, wrap.clone())
            },
        );
    for _ in 0..OVERWRITES_BEFORE_THE_CRASH_REMOUNT {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    pool.crash_and_mount_writable().expect("崩了再挂");
    for _ in 0..OVERWRITES_AFTER_THE_CRASH_REMOUNT {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    let before = pool.image();
    let first_operation = stream.retained_operations().len();
    pool.overwrite(OVERWRITE_BYTES).expect("第 17 次覆盖写");
    let operations = stream.retained_operations();
    let this_publish = &operations[first_operation..];
    let first_root = this_publish
        .iter()
        .position(|operation| {
            operation.operation.kind == RecordedOperationKind::WriteForceUnitAccess
                && operation
                    .contents
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() == ROOT_SLOT_BYTES)
        })
        .expect("这次发布写根");
    let (_, txg_of_the_missing_root) = root_identity_written_by(
        this_publish[first_root]
            .contents
            .as_ref()
            .expect("录制流留着写的内容"),
    );
    assert_eq!(
        txg_of_the_missing_root,
        CheckpointTxg(TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT)
    );
    let mut crashed = before.clone();
    crashed.apply(&this_publish[..first_root]);
    assert_eq!(
        rollback_floor_verdict(&crashed),
        InvariantVerdict::Holds,
        "挂载之前 I-7.9 成立"
    );
    let mut devices = plain_devices_on(&crashed);
    let mounted = mount_writable_with_space_admission(
        &pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("再挂做得成");
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT),
        "生效版是 journal 施加出来的 txg 31"
    );
    drop(mounted);
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: crashed.device_size_in_bytes,
    };
    assert!(
        !root_txgs_in_the_ring(&after)
            .contains(&TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT),
        "txg 31 那条根一直没写：根环上有洞"
    );
    assert!(
        !matches!(
            rollback_floor_verdict(&after),
            InvariantVerdict::Violated(_)
        ),
        "挂载之后洞里的旧根顶上来，I-7.9 不许红：{:?}",
        rollback_floor_verdict(&after)
    );
}

/// inject-6 种子 1 那一形（`r3probe ring reuse 4g 20 1 108 0`）：实例 5 txg 25 的根槽写被扣下、它的记录已落，下一次挂载由记录施加出那一版、
/// 从 txg 26 写起；txg 25 与 txg 1 同槽，txg 1 的根留在洞里，挂载与之后那次发布的根盖掉 2–5。实例 1 txg 9 那条根当初把 F 从 0 抬到 5，
/// 抬的那一刻上限就是 5：崩溃镜像上 I-7.9 成立，挂载加一次发布之后也不许红。
#[test]
fn a_remount_over_a_root_ring_hole_whose_withheld_root_its_record_applies_keeps_an_old_floor_raise_within_its_ceiling(
) {
    let crashed = inject_six_history_crashed_on_the_root_slot_write_of(
        INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD,
    );
    assert_eq!(
        rollback_floor_verdict(&crashed),
        InvariantVerdict::Holds,
        "崩溃镜像上（环里 1–24 都在）I-7.9 成立"
    );
    let after =
        image_after_a_writable_mount_and_one_publish(&crashed, HistoryDeviceWidth::FourGibibytes);
    let ring = root_txgs_in_the_ring(&after);
    assert!(
        ring.contains(&1) && !ring.contains(&INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD.1),
        "txg 25 那一槽没写过，txg 1 的根留在洞里：{ring:?}"
    );
    assert!(
        ring.contains(&INJECT_SIX_RAISING_ROOT_TXG),
        "抬 F 的根 txg 9 还在环里：{ring:?}"
    );
    assert!(
        !matches!(
            rollback_floor_verdict(&after),
            InvariantVerdict::Violated(_)
        ),
        "洞里的 txg 1 顶上来，I-7.9 不许红：{:?}",
        rollback_floor_verdict(&after)
    );
}

/// K1 类洞（`r3probe ring reuse 4g 20 1 113 0`）：同一段历史里扣下的是实例 6 写行那次发布 txg 26 的根槽写——下一次挂载没施加那一版、取号越过它，
/// txg 26 那一槽留着 txg 2 的根。I-7.9 与上一形同一个机理，挂载加一次发布之后不许红。
#[test]
fn a_remount_over_a_root_ring_hole_left_by_a_withheld_row_publish_keeps_an_old_floor_raise_within_its_ceiling(
) {
    let crashed = inject_six_history_crashed_on_the_root_slot_write_of(
        INJECT_SIX_WITHHELD_ROOT_OF_THE_ROW_PUBLISH,
    );
    assert_eq!(
        rollback_floor_verdict(&crashed),
        InvariantVerdict::Holds,
        "崩溃镜像上 I-7.9 成立"
    );
    let after =
        image_after_a_writable_mount_and_one_publish(&crashed, HistoryDeviceWidth::FourGibibytes);
    let ring = root_txgs_in_the_ring(&after);
    assert!(
        ring.contains(&2) && !ring.contains(&INJECT_SIX_WITHHELD_ROOT_OF_THE_ROW_PUBLISH.1),
        "txg 26 那一槽没写过，txg 2 的根留在洞里：{ring:?}"
    );
    assert!(
        !matches!(
            rollback_floor_verdict(&after),
            InvariantVerdict::Violated(_)
        ),
        "洞里的 txg 2 顶上来，I-7.9 不许红：{:?}",
        rollback_floor_verdict(&after)
    );
}

/// 坏镜像：inject-6 种子 1 那一形挂载加一次发布之后的镜像上，把实例 1 txg 9 那条抬 F 的根带的 F 从 5 改成 7（重封根槽），
/// 高于一圈窗口里它之前的根算出的上限：I-7.9 照旧红，红在「从 0 抬到 7」那一格。
#[test]
fn a_floor_raised_above_the_ceiling_of_the_roots_inside_the_window_still_reddens_the_rollback_floor_invariant(
) {
    let crashed = inject_six_history_crashed_on_the_root_slot_write_of(
        INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD,
    );
    let mut image =
        image_after_a_writable_mount_and_one_publish(&crashed, HistoryDeviceWidth::FourGibibytes);
    rewrite_the_floor_of_the_root_at(&mut image, INJECT_SIX_RAISING_ROOT_TXG, 7);
    match rollback_floor_verdict(&image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(&format!(
                "实例 1 txg {INJECT_SIX_RAISING_ROOT_TXG} 那条根把回退下界 F 从 0 抬到 7"
            )),
            "红在 txg 9 那条根从 0 抬到 7 上：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("F 抬到 7 高于窗口里的上限，I-7.9 要红，得到 {other:?}")
        }
    }
}

/// 正常卸载、再挂、洞上再挂那段历史的两张镜像与卸载那一串的几个数。
struct UnmountThenAHoleOneRevolutionLater {
    /// 切在洞那次覆盖写的根槽写之前（之前的写全落）：卸载前最后一版的根还在环里。
    crashed: MemoryPool,
    /// `crashed` 上再可写挂载一次之后。
    after_the_remount: MemoryPool,
    /// 卸载抬到的 F（卸载开始时现行那一版的 txg，记作 v）。
    raised_floor: u64,
    /// 卸载那一串带记号的空发布的 txg（第一条是 v + 1）。
    marked_root_txgs: Vec<u64>,
    /// 根槽一直没写的那次覆盖写的 txg：v − 1 + 3S，它那一槽留着 v − 1 的根。
    hole_txg: u64,
}

/// 两块 4 GiB 盘：第一个文件之后覆盖写三次（现行那一版 txg 记作 v），正常卸载（F 抬到 v，推带卸载记号的空发布 v + 1 起），
/// 再可写挂载、覆盖写到 v − 2 + 3S，下一次覆盖写（txg v − 1 + 3S）切在「journal 记录已落、根槽没落」：那一槽留着 v − 1 的根。
/// 再可写挂载一次：由记录施加出那一版，写行与暖机从 v + 3S 写起，盖掉 v 那一槽（与之后的一两槽）。
fn unmount_then_a_hole_one_revolution_later() -> UnmountThenAHoleOneRevolutionLater {
    let stream = SharedStream::retaining_contents();
    let wrap = stream.clone();
    let mut pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>> =
        PoolUnderTest::start_after_the_first_file(
            HistoryDeviceWidth::FourGibibytes,
            move |identity, device| {
                RecordingBlockDevice::with_shared_stream(identity, device, wrap.clone())
            },
        );
    for _ in 0..OVERWRITES_BEFORE_THE_UNMOUNT {
        pool.overwrite(OVERWRITE_BYTES).expect("卸载之前覆盖写");
    }
    let mut session = pool.session.take().expect("这一刻有可写会话");
    let Unmounted::FloorRaisedToTheCurrentVersion(unmounted) = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    )
    .expect("正常卸载") else {
        panic!("现行那一版带文件：卸载抬 F");
    };
    let raised_floor = unmounted.rollback_floor.0;
    let marked_root_txgs: Vec<u64> = unmounted
        .publishes
        .iter()
        .map(|publish| publish.root.checkpoint_txg.0)
        .collect();
    assert_eq!(
        marked_root_txgs.first().copied(),
        Some(raised_floor + 1),
        "卸载那一串从 v + 1 起"
    );
    pool.crash_and_mount_writable().expect("卸载之后再挂");
    let root_ring_slot_count = u64::try_from(pool.parameters.region_devices.len())
        .expect("区域数装得进 u64")
        * pool.parameters.geometry.root_ring_slots_per_region.count();
    let hole_txg = raised_floor - 1 + root_ring_slot_count;
    while pool.current_file_version().root.checkpoint_txg.0 + 1 < hole_txg {
        pool.overwrite(OVERWRITE_BYTES).expect("再挂之后覆盖写");
    }
    assert_eq!(
        pool.current_file_version().root.checkpoint_txg.0 + 1,
        hole_txg,
        "下一次覆盖写正好是洞那一次（中间没有推抬 F）"
    );
    let before = pool.image();
    let first_operation = stream.retained_operations().len();
    pool.overwrite(OVERWRITE_BYTES).expect("洞那一次覆盖写");
    let operations = stream.retained_operations();
    let this_publish = &operations[first_operation..];
    let first_root = this_publish
        .iter()
        .position(|operation| {
            operation.operation.kind == RecordedOperationKind::WriteForceUnitAccess
                && operation
                    .contents
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() == ROOT_SLOT_BYTES)
        })
        .expect("这次发布写根");
    let (_, txg_of_the_missing_root) = root_identity_written_by(
        this_publish[first_root]
            .contents
            .as_ref()
            .expect("录制流留着写的内容"),
    );
    assert_eq!(txg_of_the_missing_root, CheckpointTxg(hole_txg));
    let mut crashed = before;
    crashed.apply(&this_publish[..first_root]);
    let mut devices = plain_devices_on(&crashed);
    let mounted = mount_writable_with_space_admission(
        &pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("洞上再挂做得成");
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(hole_txg),
        "生效版是 journal 施加出来的洞那一版"
    );
    drop(mounted);
    let after_the_remount = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: crashed.device_size_in_bytes,
    };
    UnmountThenAHoleOneRevolutionLater {
        crashed,
        after_the_remount,
        raised_floor,
        marked_root_txgs,
        hole_txg,
    }
}

/// 带卸载记号那一支的误红形（随机历史种子 31045、1037 那一类）：洞上再挂之后，根环里卸载前最后一版 v 的根被盖掉，
/// 洞那一槽留着 v − 1 的根；卸载那一串里留在环里的带记号的根，同一实例前一条成了 v − 1、算抬 F。只取锚在它上面的窗口时
/// 「txg 比它小的最新那条根」是 v − 1、F = v 高于它而红；两个窗口的交里没有它之前的根（镜像最新根离它不到一圈，
/// 比它小的根都 ≤ N − 3S），这条根不判。挂载之前那张镜像（v 还在环里）I-7.9 按带记号那一支判 v ≤ v 成立。
#[test]
fn an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole(
) {
    let history = unmount_then_a_hole_one_revolution_later();
    let v = history.raised_floor;
    let ring_before = root_txgs_in_the_ring(&history.crashed);
    assert!(
        ring_before.contains(&v) && ring_before.contains(&(v - 1)),
        "挂载之前 v 与 v − 1 的根都在环里：{ring_before:?}"
    );
    assert_eq!(
        rollback_floor_verdict(&history.crashed),
        InvariantVerdict::Holds,
        "挂载之前带记号的根按前一条根 v 判，成立"
    );
    let ring_after = root_txgs_in_the_ring(&history.after_the_remount);
    assert!(
        !ring_after.contains(&history.hole_txg)
            && ring_after.contains(&(v - 1))
            && !ring_after.contains(&v),
        "洞那一槽留着 v − 1 的根，v 那一槽被盖掉：{ring_after:?}"
    );
    assert!(
        history
            .marked_root_txgs
            .iter()
            .any(|txg| ring_after.contains(txg)),
        "卸载那一串还有带记号的根留在环里：{:?} / {ring_after:?}",
        history.marked_root_txgs
    );
    assert!(
        !matches!(
            rollback_floor_verdict(&history.after_the_remount),
            InvariantVerdict::Violated(_)
        ),
        "洞里的 v − 1 顶上来，带记号那一支不许红：{:?}",
        rollback_floor_verdict(&history.after_the_remount)
    );
}

/// 坏镜像：同一段历史挂载之前那张镜像上，把卸载那一串第一条根（txg v + 1）带的 F 从 v 改成 v + 1（重封根槽）：
/// 两个窗口的交里它之前最新的根是 v，F 高于它，I-7.9 照旧红在带卸载记号那一支。
#[test]
fn an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant(
) {
    let history = unmount_then_a_hole_one_revolution_later();
    let v = history.raised_floor;
    let mut image = history.crashed;
    rewrite_the_floor_of_the_root_at(&mut image, v + 1, v + 1);
    match rollback_floor_verdict(&image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(&format!(
                "txg {} 那条带卸载记号的根把回退下界 F 从 0 抬到 {}，高于根环里 txg 比它小的最新那条根的 txg {v}",
                v + 1,
                v + 1
            )),
            "红在带卸载记号那一支、前一条根 v：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("带记号的根把 F 抬到 v + 1、高于前一条根 v，I-7.9 要红，得到 {other:?}")
        }
    }
}
