//! 根记录带回收门槛 t 与钉槽 p（C773（读坏与全零根槽的回收门槛钉只在内存里）；D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」，
//! 用户 2026-10-20 定）：带单元的发布写 t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)、p = 记完之后还钉着的槽里钉值最小的那个，
//! 零单元发布照抄上一版的，施加记录拼根时从记录新根段带进来；池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛）、
//! I-7.19（钉着的槽不住本实例写的根） 只判建账的发布，I-3.1（已分配统计对得上） 把「钉带出来的那一截」不看引用并进遍历得到的和。
//!
//! 回归形取根环三方第三轮攻方 `zero.out` 那一形（`research/prompts/m3-close-rootring-r3-opus-model/tests/rootring_r3_attack_zero_unit_roots_after_a_full_ring.rs`
//! 的 `pinned` 组）：环洞池 S = 4，实例 2 覆盖写到 txg 19（环里 8–19）；实例 3 可写挂载那一刻根环区域 0 槽 2（住着 txg 18）读回全零，
//! 写者把门槛钉在那一槽可能住过的最旧 txg 6，之后写行 20、暖机 21、22。改之前这一刻 I-3.1 红在记账多的一边（钉只在内存里）。

#![allow(dead_code, reason = "两档各只调用其中一部分场景与帮手")]

use crate::common::{
    build_pool, crash_state_devices, format_pool,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_root_ring_holes::{
    content_of_the_version, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use crate::common_unwitnessed_publish::{
    recorded_devices_reading_back_all_zero_on, RangesReadingBackAllZero,
};
use crate::scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::pool_with_a_dead_instance;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches, unmount, Mounted, ShadowLedger,
    Unmounted,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::root_record::{PinnedRingSlot, RootRecord};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, writes_and_segments_with_stream_indexes_and_entries, MemoryPool,
    RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedPublishEntry, SharedStream};

/// 根记录里自证校验和 32 字节的偏移（D22（单元原子性怎么合成） 已定项 7）、回收门槛 t 8 字节与钉槽 p 1 字节的偏移（C773：分配记录树根指针之后）。
const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 428;
const ROOT_RECORD_PINNED_RING_SLOT_OFFSET: usize = 436;
/// journal 记录头校验和 32 字节的偏移（罩整条 4096）、新根段里 t 与 p 的偏移（新根段从 99 起，段内 364、372）。
const JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET: usize = 46;
const JOURNAL_RECORD_BYTES: usize = 4096;
const JOURNAL_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 99 + 364;
const JOURNAL_RECORD_PINNED_RING_SLOT_OFFSET: usize = 99 + 372;
/// journal 记录头里实例代号 4 字节与 `checkpoint_txg` 8 字节的偏移；根记录里同两样的偏移。
const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;

/// 死实例池上实例 3 的可写挂载连同之后的正常卸载：挂载那一串写与卸载那一串写分开交回，基盘面是挂载之前那一份。
struct MountAndUnmountOfTheThirdInstance {
    base: MemoryPool,
    writes_of_the_mount: Vec<RetainedWrite>,
    writes_of_the_unmount: Vec<RetainedWrite>,
}

/// 在死实例池（`pool_with_a_dead_instance`：环里 txg 8–19）上做实例 3 的可写挂载（写行 20、暖机 21、22），挂载那一刻 `unreadable_at_the_mount`
/// 那几段读回全零；挂载之后正常卸载（抬 F 到 22，根 23、24、25）。
fn mount_and_unmount_the_third_instance(
    unreadable_at_the_mount: &RangesReadingBackAllZero,
) -> MountAndUnmountOfTheThirdInstance {
    let parameters = parameters_of_the_ring_hole_pools();
    let base = pool_with_a_dead_instance(None).image;
    let stream = SharedStream::retaining_contents();
    let mut devices =
        recorded_devices_reading_back_all_zero_on(&base, unreadable_at_the_mount, &stream);
    let mut mounted = mount_writable_with_test_only_switches(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
    )
    .expect("实例 3 的可写挂载");
    let (writes_of_the_mount, _segments_of_the_mount) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    // 卸载那一串记成录制流里卸载入口发的一段：带卸载记号的根槽写只许出自那一段（C557 那道检查）。
    stream
        .record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                &parameters,
                &mut devices,
                &mut mounted.allocator,
                &mut mounted.current,
                ShadowLedger::On,
            )
        })
        .expect("正常卸载");
    let (every_write, _segments_of_the_mount_and_the_unmount, _stream_indexes) =
        writes_and_segments_with_stream_indexes_and_entries(
            &stream.retained_operations(),
            &stream.entry_spans(),
            &fixed_geometry_of_the_ring_hole_pools(),
        );
    let writes_of_the_unmount = every_write[writes_of_the_mount.len()..].to_vec();
    MountAndUnmountOfTheThirdInstance {
        base,
        writes_of_the_mount,
        writes_of_the_unmount,
    }
}

/// 环洞池根环一槽（512 字节）在挂载那一刻读回全零。
fn ring_slot_reading_back_all_zero(ring_slot: RootRingSlot) -> RangesReadingBackAllZero {
    let parameters = parameters_of_the_ring_hole_pools();
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0;
    RangesReadingBackAllZero(vec![(device, offset, 512)])
}

/// 根环区域 0 槽 2：落在它上面的 txg 是 6、18（区域 txg mod 3 = 0，槽 (txg div 3) mod 4 = 2），挂载那一刻住着 txg 18，钉在 6（`zero.out` 那一形）。
const RING_SLOT_HOLDING_TXG_EIGHTEEN: RootRingSlot = RootRingSlot { region: 0, slot: 2 };
/// 根环区域 2 槽 2：落在它上面的 txg 是 8、20，挂载那一刻住着 txg 8（环里最旧的那条），钉在 8；写行那次（txg 20）就把它写掉、解开。
const RING_SLOT_HOLDING_TXG_EIGHT: RootRingSlot = RootRingSlot { region: 2, slot: 2 };

/// 盘面上 txg `txg` 的那条根（环洞池的几何）。
fn root_at_txg(image: &MemoryPool, txg: u64) -> RootRecord {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_slot = target_for_publish(
        CheckpointTxg(txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let bytes = singlefs_core::recovery::PoolReader::read(
        image,
        device,
        slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing),
        usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽"),
    )
    .expect("根槽读得出");
    RootRecord::parse_slot(&bytes, &parameters.filesystem_identifier)
        .unwrap_or_else(|| panic!("txg {txg} 那一槽住着一条自证过的根"))
}

/// 基盘面施加一串写之后的盘面。
fn image_after(base: &MemoryPool, writes: &[RetainedWrite]) -> MemoryPool {
    let mut image = base.clone();
    image.apply_writes(writes);
    image
}

/// 一串写里 txg `txg` 那条根的根槽写与那次发布的每一份 journal 记录写，t、p 改成给定的值、重封（记录新根段与根同改，I-8.10 照旧成立）。
/// 只改这几次写自己的字节：施加之后环里同盘后面没有这一实例的记录（txg `txg` 是挂载那一串最后一次发布），反向链不用往下传。
fn rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(
    writes: &mut [RetainedWrite],
    txg: u64,
    reclaim_threshold: u64,
    pinned_ring_slot: u8,
) {
    let mut rewritten_roots = 0usize;
    let mut rewritten_records = 0usize;
    for write in writes.iter_mut() {
        let kind = write.kind;
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            continue;
        };
        if kind == StepKind::RootRecordFua
            && bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                == txg.to_le_bytes()
        {
            bytes[ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET..ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET + 8]
                .copy_from_slice(&reclaim_threshold.to_le_bytes());
            bytes[ROOT_RECORD_PINNED_RING_SLOT_OFFSET] = pinned_ring_slot;
            let slot_bytes = bytes.len();
            let digest =
                wide_checksum_with_field_zeroed(bytes, slot_bytes, ROOT_RECORD_CHECKSUM_OFFSET);
            bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            rewritten_roots += 1;
        } else if kind == StepKind::JournalRecord
            && bytes[JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET..JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                == txg.to_le_bytes()
        {
            bytes[JOURNAL_RECORD_RECLAIM_THRESHOLD_OFFSET
                ..JOURNAL_RECORD_RECLAIM_THRESHOLD_OFFSET + 8]
                .copy_from_slice(&reclaim_threshold.to_le_bytes());
            bytes[JOURNAL_RECORD_PINNED_RING_SLOT_OFFSET] = pinned_ring_slot;
            let digest = wide_checksum_with_field_zeroed(
                bytes,
                JOURNAL_RECORD_BYTES,
                JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET,
            );
            bytes
                [JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET..JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            rewritten_records += 1;
        }
    }
    assert_eq!(rewritten_roots, 1, "txg {txg} 的根槽写恰一次");
    assert!(
        rewritten_records >= 1,
        "txg {txg} 那次发布有 journal 记录写（两盘各一份）"
    );
}

/// C773 登记那一形的回归（`zero.out` 的 `pinned` 组，崩在暖机 txg 22 的根槽之后那一刻）：挂载那一刻区域 0 槽 2 读回全零，写者钉在 6，
/// 写行 20、暖机 21、22 三条根都带 t = 6、p = 6（max(F = 0, min(环里最旧有效根 9 / 10 / 11, 钉 6))；钉值 6 = 3 × 槽 2 + 区域 0）。
/// 改之前这一刻 I-3.1 红在记账多的一边（差的是释放代落在 (6, 11] 的已释放槽，钉只在内存里、checker 看不见）；改之后 I-3.1 并上
/// 「钉带出来的那一截」成立，I-7.18、I-7.19 判过而成立。
pub fn a_ring_slot_reading_back_all_zero_at_the_mount_is_carried_as_the_pinned_ring_slot_and_the_allocated_statistic_holds(
    images: &mut PoolCheckerImages,
) {
    let mount = mount_and_unmount_the_third_instance(&ring_slot_reading_back_all_zero(
        RING_SLOT_HOLDING_TXG_EIGHTEEN,
    ));
    let image = image_after(&mount.base, &mount.writes_of_the_mount);
    for txg in [20u64, 21, 22] {
        let root = root_at_txg(&image, txg);
        assert_eq!(
            (root.reclaim_threshold, root.pinned_ring_slot),
            (CheckpointTxg(6), PinnedRingSlot(6)),
            "txg {txg} 的根：t = 6、p = 6（钉着的区域 0 槽 2，钉值 6）"
        );
    }
    images.hand_out(
        "回收门槛与钉槽的正常镜像",
        || image,
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds("I-3.1")
            .and_holds("I-7.18")
            .and_holds("I-7.19"),
    );
}

/// 同一段历史、挂载那一刻没有读故障：没有钉，暖机 22 的根带 t = 11（环里最旧有效根）、p = 255；三条判过而成立。
pub fn a_mount_without_read_faults_pins_nothing_and_writes_the_oldest_valid_root_as_the_reclaim_threshold(
    images: &mut PoolCheckerImages,
) {
    let mount = mount_and_unmount_the_third_instance(&RangesReadingBackAllZero::default());
    let image = image_after(&mount.base, &mount.writes_of_the_mount);
    let root = root_at_txg(&image, 22);
    assert_eq!(
        (root.reclaim_threshold, root.pinned_ring_slot),
        (CheckpointTxg(11), PinnedRingSlot::NO_SLOT_PINNED),
        "暖机 22：环里 11–22，没有钉着的槽"
    );
    images.hand_out(
        "回收门槛与钉槽的正常镜像",
        || image,
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds("I-3.1")
            .and_holds("I-7.18")
            .and_holds("I-7.19"),
    );
}

/// 根环三方第三轮攻方 Z4.2 那一形：正常卸载把 F 抬到 22，卸载那一串的根带 F = 22；分配器记根时的回收门槛按挂载时的 F（0）与环里最旧有效根
/// （12–14）算，t 要再与这条根自己带的 F 取大，写 22。checker 照根自己带的 F 现算 22，I-7.18 判过而成立。
pub fn the_unmount_raising_the_floor_writes_the_raised_floor_as_the_reclaim_threshold(
    images: &mut PoolCheckerImages,
) {
    let mount = mount_and_unmount_the_third_instance(&RangesReadingBackAllZero::default());
    let mut every_write = mount.writes_of_the_mount.clone();
    every_write.extend(mount.writes_of_the_unmount.iter().cloned());
    let image = image_after(&mount.base, &every_write);
    for txg in [23u64, 24, 25] {
        let root = root_at_txg(&image, txg);
        assert_eq!(
            (root.rollback_floor, root.reclaim_threshold),
            (CheckpointTxg(22), CheckpointTxg(22)),
            "卸载那一串的 txg {txg}：F = 22，t = max(F, 回收门槛) = 22"
        );
    }
    images.hand_out(
        &format!("{}-{}", images.handed_out(), "I-7.18"),
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-7.18"),
    );
}

/// 坏镜像「t 恒取 F」：`zero.out` 那一形暖机 22 的根与记录里 t 写成 F（0），p 照旧 6。checker 现算 max(0, min(11, 6)) = 6，I-7.18 红。
pub fn a_reclaim_threshold_written_as_the_floor_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant(
    images: &mut PoolCheckerImages,
) {
    let mut mount = mount_and_unmount_the_third_instance(&ring_slot_reading_back_all_zero(
        RING_SLOT_HOLDING_TXG_EIGHTEEN,
    ));
    rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(
        &mut mount.writes_of_the_mount,
        22,
        0,
        6,
    );
    let image = image_after(&mount.base, &mount.writes_of_the_mount);
    images.hand_out(
        &format!("{}-{}", images.handed_out(), "I-7.18"),
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants().and_violated("I-7.18"),
    );
}

/// 坏镜像「p 写 255 而还钉着」：`zero.out` 那一形暖机 22 的根与记录里 p 写成 255，t 照旧 6。checker 现算不带钉 max(0, 11) = 11，I-7.18 红；
/// 钉带出来的那一截照 t = 6 并，I-3.1 不红。
pub fn a_pinned_ring_slot_written_as_none_while_a_slot_stays_pinned_reddens_the_reclaim_threshold_invariant(
    images: &mut PoolCheckerImages,
) {
    let mut mount = mount_and_unmount_the_third_instance(&ring_slot_reading_back_all_zero(
        RING_SLOT_HOLDING_TXG_EIGHTEEN,
    ));
    rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(
        &mut mount.writes_of_the_mount,
        22,
        6,
        PinnedRingSlot::NO_SLOT_PINNED.0,
    );
    let image = image_after(&mount.base, &mount.writes_of_the_mount);
    images.hand_out(
        "钉槽改坏的镜像与未受影响的不变量",
        || image,
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated("I-7.18")
            .and_holds("I-3.1"),
    );
}

/// 坏镜像「写掉钉着的槽不解钉」：挂载那一刻区域 2 槽 2（txg 8）读回全零、钉在 8，写行那次（txg 20）写掉它、解开（真写者交 p = 255）；
/// 暖机 22 的根与记录里 p 写成 8、t 写成 8（与那个钉自洽，I-7.18 不红）。钉槽指着的区域 2 槽 2 住着实例 3 自己写的 txg 20，I-7.19 红。
pub fn a_pinned_ring_slot_left_on_a_slot_this_instance_wrote_reddens_the_pinned_slot_invariant(
    images: &mut PoolCheckerImages,
) {
    let mut mount = mount_and_unmount_the_third_instance(&ring_slot_reading_back_all_zero(
        RING_SLOT_HOLDING_TXG_EIGHT,
    ));
    let image_before_the_rewrite = image_after(&mount.base, &mount.writes_of_the_mount);
    for txg in [20u64, 21, 22] {
        assert_eq!(
            root_at_txg(&image_before_the_rewrite, txg).pinned_ring_slot,
            PinnedRingSlot::NO_SLOT_PINNED,
            "写行那次（txg 20）写掉了钉着的槽、钉随之解开：真写者从 txg 20 起交 p = 255（txg {txg}）"
        );
    }
    rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(
        &mut mount.writes_of_the_mount,
        22,
        8,
        8,
    );
    let image = image_after(&mount.base, &mount.writes_of_the_mount);
    images.hand_out(
        "钉槽改坏的镜像与未受影响的不变量",
        || image,
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated("I-7.19")
            .and_holds("I-7.18"),
    );
}

/// 只做过 mkfs 的池：第一次可写挂载（取号 1、不写行，暖机 txg 1、2）之后把根环区域 0 槽 0（第 0 代根）抹成全零，第二次可写挂载那一刻它读回全零、
/// 钉在 0。写行那次（txg 3，带单元）写 t = 0、p = 0；之后的暖机（txg 4，零单元）照抄写行那一版的 t、p。
pub fn a_zero_unit_warm_up_copies_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_row_publish(
) {
    let mut formatted = format_pool("reclaim-threshold-copied-by-a-zero-unit-warm-up");
    let mut first_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first_mount_devices)
        .expect("第一次可写挂载：取号 1、不写行");
    formatted.devices = Some(first_mount_devices);
    let mut image = formatted.memory_pool();
    let genesis_ring_slot = RootRingSlot { region: 0, slot: 0 };
    let device = parameters().region_devices[0];
    image.devices.get_mut(&device).expect("区域 0 的盘").write(
        slot_offset(
            genesis_ring_slot,
            parameters().geometry.fixed_structure_slot_spacing,
        ),
        &[0u8; 512],
    );
    let stream = SharedStream::retaining_contents();
    let mut second_mount_devices = crash_state_devices(&image, &[], &[], &stream);
    let Mounted {
        output, current, ..
    } = mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载：写行");
    let row_root = *output.row_publish.root();
    let warm_up_root = *current.root();
    assert_eq!(
        (
            row_root.checkpoint_txg,
            row_root.reclaim_threshold,
            row_root.pinned_ring_slot
        ),
        (CheckpointTxg(3), CheckpointTxg(0), PinnedRingSlot(0)),
        "写行那次：区域 0 槽 0 钉在 0，t = max(0, min(1, 0)) = 0，p = 0"
    );
    assert!(
        matches!(current, PoolVersion::WithoutFile(_)),
        "写行之后的暖机仍在树表 0 条的一版上：零单元发布"
    );
    assert_eq!(
        (
            warm_up_root.checkpoint_txg,
            warm_up_root.reclaim_threshold,
            warm_up_root.pinned_ring_slot
        ),
        (CheckpointTxg(4), CheckpointTxg(0), PinnedRingSlot(0)),
        "零单元发布照抄写行那一版的 t、p"
    );
}

/// 新池新建文件（txg 3）之后覆盖写三版（txg 4、5、6）的填充字节：现行 txg 6 除以 3 余 0，卸载那一串推两次空发布（txg 7、8）。
const FILL_BYTES_OF_THE_THREE_OVERWRITES: [u8; 3] = [0xB0, 0xC0, 0xD0];

/// 卸载那一串（txg 7、8，带 F = 6）的根槽都没落、记录都落了：只读挂载择到 txg 6 那条（t = 0），施加 txg 7、8 两条记录；
/// 施加出来的根与 txg 8 那次写的根逐字段相同——t、p 取记录新根段里的（t = max(F = 6, 回收门槛) = 6），不照抄所选根的。
pub fn a_root_rebuilt_from_records_takes_the_reclaim_threshold_and_the_pinned_ring_slot_from_the_new_root_segment(
) {
    let mut pool = build_pool("rebuilt-root-reclaim-threshold-from-the-record");
    for fill_byte in FILL_BYTES_OF_THE_THREE_OVERWRITES {
        overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B],
        );
    }
    let root_of_txg_six = pool.output.root;
    assert_eq!(root_of_txg_six.checkpoint_txg, CheckpointTxg(6));
    let mut current = PoolVersion::WithFile(pool.output.clone());
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let unmounted = unmount(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载");
    let Unmounted::FloorRaisedToTheCurrentVersion(_) = unmounted else {
        panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」");
    };
    let root_of_txg_eight = *current.root();
    assert_eq!(
        (
            root_of_txg_eight.checkpoint_txg,
            root_of_txg_eight.rollback_floor,
            root_of_txg_eight.reclaim_threshold
        ),
        (CheckpointTxg(8), CheckpointTxg(6), CheckpointTxg(6)),
        "卸载那一串第二次空发布：F = 6，t = max(6, 回收门槛) = 6"
    );
    assert_ne!(
        root_of_txg_six.reclaim_threshold, root_of_txg_eight.reclaim_threshold,
        "所选根与施加出来的根的 t 不同：照抄所选根就看得出"
    );
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 7);
    tear_the_root_slot_written_for_txg(&mut image, 8);
    let read_only = mount_read_only(&image).expect("只读挂载");
    assert_eq!(
        read_only.chosen_root, root_of_txg_six,
        "txg 7、8 的根槽没落：所选根是 txg 6"
    );
    assert_eq!(
        read_only.effective_root, root_of_txg_eight,
        "施加出来的根与 txg 8 那次写的根逐字段相同（t、p 取记录里的）"
    );
}

/// physical_block_size 4096 的环洞池（S = 4、环长 12）：根槽 4 KiB，根记录 466 字节全在头一个 512 字节扇区里，后七个扇区是补齐的 0。
fn parameters_of_a_ring_hole_pool_with_four_kibibyte_physical_blocks() -> MakeFilesystemParameters {
    let mut parameters = parameters_of_the_ring_hole_pools();
    parameters.geometry.physical_block_size = 4096;
    parameters.geometry.minimum_input_output_bytes = 4096;
    parameters
}

/// 第一次可写挂载（暖机 1、2）→ 第一个文件（txg 3）→ 覆盖写到 txg 14：环里 3–14 正好一圈，下一次发布（txg 15）要写的那一槽住着环里最旧的 txg 3。
/// 交回到 txg 14 为止的盘面、txg 15 那次覆盖写的写（录下来的）与 txg 15 根槽写之前那一槽的旧字节。
fn four_kibibyte_pool_and_the_publish_overwriting_the_oldest_root(
) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_a_ring_hole_pool_with_four_kibibyte_physical_blocks();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(4096),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    // 迭代上界：txg 逐次加一，到 14 为止。
    while current.root.checkpoint_txg.0 < 14 {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            instance,
        )
        .expect("覆盖写");
    }
    let image_through_txg_fourteen = pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recording: Vec<(
        DeviceIdentity,
        singlefs_harness::RecordingBlockDevice<SparseBlockDevice>,
    )> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                singlefs_harness::RecordingBlockDevice::with_shared_stream(
                    identity,
                    device,
                    stream.clone(),
                ),
            )
        })
        .collect();
    {
        let salt = 15;
        let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            instance,
        )
        .expect("txg 15 的覆盖写");
    }
    let mut fixed_geometry = fixed_geometry_of_the_ring_hole_pools();
    fixed_geometry.fixed_structure_slot_spacing = parameters.geometry.fixed_structure_slot_spacing;
    let (writes_of_txg_fifteen, _segments_of_txg_fifteen) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry);
    (image_through_txg_fourteen, writes_of_txg_fifteen)
}

/// 根环三方第三轮辩方 Z4.3 没核的那一格（C773 登记行「`physical_block_size` 大于 512 时根槽撕在扇区边界、现算门槛那一条会不会红没人核」）：
/// physical_block_size 4096，txg 15 那次覆盖写的根槽写撕在扇区边界——只落了头一个扇区，或只没落头一个扇区。
/// 根记录 466 字节全在头一个扇区里，新根与旧根（txg 3）的后七个扇区都是补齐的 0：撕出来的槽要么就是新根整槽、要么就是旧根整槽，
/// 都自证得过。池级 checker 判 I-7.18：新根落了的那一张最新根是 txg 15（写者记它时环里最旧的是 4），旧根还在的那一张最新根是 txg 14
/// （写者记它时 txg 3 还在环里，checker 也读得到 txg 3）——两张都判过而成立，不误红。
pub fn a_root_slot_torn_at_a_sector_boundary_with_four_kibibyte_physical_blocks_does_not_redden_the_reclaim_threshold_invariant(
    images: &mut PoolCheckerImages,
) {
    let (image_through_txg_fourteen, writes_of_txg_fifteen) =
        four_kibibyte_pool_and_the_publish_overwriting_the_oldest_root();
    let root_write_index = writes_of_txg_fifteen
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("txg 15 的根槽写");
    let root_write = &writes_of_txg_fifteen[root_write_index];
    let new_slot = root_write.bytes().expect("根槽写带字节").to_vec();
    assert_eq!(new_slot.len(), 4096, "根槽写整槽 4 KiB");
    let old_slot = singlefs_core::recovery::PoolReader::read(
        &image_through_txg_fourteen,
        root_write.device,
        root_write.offset,
        4096,
    )
    .expect("那一槽的旧字节读得出");
    assert!(
        new_slot[512..].iter().all(|byte| *byte == 0)
            && old_slot[512..].iter().all(|byte| *byte == 0),
        "新根与旧根的后七个扇区都是补齐的 0"
    );
    let torn_slot_with_only_the_first_sector_written: Vec<u8> = new_slot[..512]
        .iter()
        .chain(old_slot[512..].iter())
        .copied()
        .collect();
    let torn_slot_without_the_first_sector: Vec<u8> = old_slot[..512]
        .iter()
        .chain(new_slot[512..].iter())
        .copied()
        .collect();
    let image_torn_with = |torn_slot: Vec<u8>| -> MemoryPool {
        let mut image = image_after(
            &image_through_txg_fourteen,
            &writes_of_txg_fifteen[..root_write_index],
        );
        image.apply_writes(&[RetainedWrite {
            device: root_write.device,
            kind: root_write.kind,
            is_force_unit_access: root_write.is_force_unit_access,
            offset: root_write.offset,
            contents: WrittenContents::Bytes(torn_slot),
        }]);
        image
    };
    let parameters = parameters_of_a_ring_hole_pool_with_four_kibibyte_physical_blocks();
    let newest_root_txg = |image: &MemoryPool| -> u64 {
        let configuration =
            singlefs_core::recovery::choose_system_configuration(image).expect("系统配置");
        singlefs_core::recovery::readable_roots(
            image,
            &configuration.immutable.region_devices,
            &configuration.immutable.sizes,
            &parameters.filesystem_identifier,
        )
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .max()
        .expect("环里有根")
    };
    let first_sector_written = image_torn_with(torn_slot_with_only_the_first_sector_written);
    assert_eq!(
        newest_root_txg(&first_sector_written),
        15,
        "只落了头一个扇区：槽里就是新根整槽"
    );
    images.hand_out(
        &format!("{}-{}", images.handed_out(), "I-7.18"),
        || first_sector_written.clone(),
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-7.18"),
    );
    let first_sector_not_written = image_torn_with(torn_slot_without_the_first_sector);
    assert_eq!(
        newest_root_txg(&first_sector_not_written),
        14,
        "只没落头一个扇区：槽里还是旧根（txg 3）整槽"
    );
    images.hand_out(
        &format!("{}-{}", images.handed_out(), "I-7.18"),
        || first_sector_not_written.clone(),
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-7.18"),
    );
}
