//! 欠账 C563（`.claude/kb/checks-owed.md`「读根环一槽读坏就当没有根，别的调用方没判」）F2 退回的候选那一处：
//! `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to` 经 `recovery::readable_roots`
//! 读根环，一个槽这一次读坏（读不出或自证不过）就当那里没有根。
//!
//! 盘面照 `recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 的 H4 那一形：崩在 R 的根槽之后、轮换之前，
//! 读的那一刻 R 的单元与每块盘较新的那个系统配置槽读回全零，c_见证 落回 P 之前那一版 Pm；不坏时恢复、只读挂载、可写挂载三处都退到已确认的 P。
//! P 的根槽只坏一次（碰到它的第几次读坏，逐个试遍不坏时那一趟读它的每一次）：今天择根那一读坏了没事，F2 判候选那两读（`fall_back_judged_on_the_three_readers`
//! 先问一次、触发之后再问一次）坏了任一读，P 掉出候选，[Pm, R) 里只剩 Pm，Y4 那一关（P 带末条标志的记录读得出、根读不出）不退，三处照报 R 读不出、
//! 可写挂载拒——一次瞬时读错就把一次本来做得成的退回变成拒。退不到 Pm 是 Y4 护着的：去掉 Y4 时同一读坏三处悄悄落到 Pm（调查量过）。
//! 这几条在今天的代码上红；改法与条款归主 agent（C562（抬 F 算上限时根槽读坏当没有根） 那一格的分法）。
//! P 的根槽每一读都坏时三处照报 R 读不出（Y4，用户 2026-10-08 定「两形都修」）：这一半今天就绿，钉住改法不把「仍坏」放成退到 Pm。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    record_one_publish_after_overwrites, DeviceReadingBackAllZeroOnRanges, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use std::cell::Cell;
use std::rc::Rc;

/// 没见证的那次发布 R 之前覆盖写几次（与 H4 那一组的一条同档）。
const OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH: u64 = 5;

/// 那一槽的那一读怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BadRead {
    /// 块设备报 `InputOutput`。
    Unreadable,
    /// 读得出，那一槽的第一个字节（magic 那一字节）取反：自证不过。
    FirstByteFlipped,
}

/// 碰到那一槽的读里哪几读坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadsOfTheSlotThatGoBad {
    Never,
    Every,
    /// 碰到它的第几次读（从 1 数），只这一读坏。
    OnlyTheReadNumbered(usize),
}

/// P 的根槽（盘、偏移、长度）哪几读坏、怎么坏；几块盘共用同一个读数计数。
#[derive(Clone)]
struct SlotReadingBad {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    reads_that_go_bad: ReadsOfTheSlotThatGoBad,
    bad_read: BadRead,
    reads_of_the_slot: Rc<Cell<usize>>,
}

/// 一块盘（里面是读回全零那一层）：碰到点名那一槽的读按 [`SlotReadingBad`] 坏，别的照转。
struct DeviceWithASlotReadingBad {
    identity: DeviceIdentity,
    inner: DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
    slot: SlotReadingBad,
}

impl BlockDevice for DeviceWithASlotReadingBad {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let slot = &self.slot;
        let touches_the_slot = slot.device == self.identity
            && offset.0 < slot.offset_in_bytes + slot.length_in_bytes
            && slot.offset_in_bytes < offset.0 + length_in_bytes;
        if !touches_the_slot {
            return self.inner.read_at(offset, buffer);
        }
        let reads_so_far = slot.reads_of_the_slot.get() + 1;
        slot.reads_of_the_slot.set(reads_so_far);
        let this_read_is_bad = match slot.reads_that_go_bad {
            ReadsOfTheSlotThatGoBad::Never => false,
            ReadsOfTheSlotThatGoBad::Every => true,
            ReadsOfTheSlotThatGoBad::OnlyTheReadNumbered(number) => number == reads_so_far,
        };
        if !this_read_is_bad {
            return self.inner.read_at(offset, buffer);
        }
        match slot.bad_read {
            BadRead::Unreadable => Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一读坏",
            ))),
            BadRead::FirstByteFlipped => {
                self.inner.read_at(offset, buffer)?;
                let first_byte_of_the_slot_in_the_read =
                    usize::try_from(slot.offset_in_bytes.saturating_sub(offset.0))
                        .expect("一次读里的偏移装得进 usize");
                buffer[first_byte_of_the_slot_in_the_read] ^= 0xff;
                Ok(())
            }
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
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

/// 每块盘上较新的那个系统配置槽（两槽里自证过、槽世代号大的那一槽）：盘、偏移、长度（同 H4 那一组的取法）。
fn newer_system_configuration_slot_of_each_device(
    image: &MemoryPool,
) -> Vec<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    image
        .devices
        .iter()
        .map(|(device, sparse_device)| {
            let newer_slot_offset = [0, slot_spacing_in_bytes]
                .into_iter()
                .filter_map(|offset| {
                    SystemConfiguration::parse_slot(
                        &sparse_device.read(DeviceOffsetInBytes(offset), slot_bytes),
                    )
                    .ok()
                    .map(|slot| (slot.quantities.slot_generation, offset))
                })
                .max()
                .map(|(_, offset)| offset)
                .expect("每块盘至少一槽自证过");
            (*device, newer_slot_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
        })
        .collect()
}

/// H4 那一形的盘面：R 录下来、崩在它的根槽之后轮换之前；读回全零的是 R 的单元与每块盘较新的那个系统配置槽。另交 P 的根槽。
struct FallBackToTheConfirmedPrevious {
    recorded: RecordedPublish,
    image: MemoryPool,
    ranges: RangesReadingBackAllZero,
    root_slot_of_the_previous_version: (DeviceIdentity, u64, u64),
}

fn fall_back_to_the_confirmed_previous() -> FallBackToTheConfirmedPrevious {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero(newer_system_configuration_slot_of_each_device(&image))
        .with_the_writes(&recorded.unit_writes());
    let root_slot_of_the_previous_version = RangesReadingBackAllZero::default()
        .with_the_root_slot_of(&image, recorded.previous_key)
        .0[0];
    FallBackToTheConfirmedPrevious {
        recorded,
        image,
        ranges,
        root_slot_of_the_previous_version,
    }
}

/// 读这份盘面的三处。
#[derive(Clone, Copy, Debug)]
enum Reader {
    Recovery,
    ReadOnlyMount,
    WritableMount,
}

/// 一处读完落到哪：落到的那一版 (实例, txg)，或照报读不出（拒）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum Landing {
    OnTheVersion((InstanceGeneration, CheckpointTxg)),
    ReportedTheChosenVersionUnreadable,
}

/// `reader` 读 `pool` 那一处，P 的根槽照 `reads_that_go_bad` 与 `bad_read` 坏；交回落到哪与这一趟碰到那一槽几次。
/// 可写挂载拒的时候另核在任何写之前拒（两块盘逐字节不变）；恢复退到的那一版另核读回的是它的内容。
fn landing_of(
    reader: Reader,
    pool: &FallBackToTheConfirmedPrevious,
    reads_that_go_bad: ReadsOfTheSlotThatGoBad,
    bad_read: BadRead,
) -> (Landing, usize) {
    let (device, offset_in_bytes, length_in_bytes) = pool.root_slot_of_the_previous_version;
    let slot = SlotReadingBad {
        device,
        offset_in_bytes,
        length_in_bytes,
        reads_that_go_bad,
        bad_read,
        reads_of_the_slot: Rc::new(Cell::new(0)),
    };
    let mut devices: Vec<(DeviceIdentity, DeviceWithASlotReadingBad)> =
        devices_reading_back_all_zero_on(&pool.image, &pool.ranges)
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    DeviceWithASlotReadingBad {
                        identity,
                        inner,
                        slot: slot.clone(),
                    },
                )
            })
            .collect();
    let landing = match reader {
        Reader::Recovery => match recover(&devices, JournalPolicy::Consult).outcome {
            RecoveryOutcome::FileRead { root, content } => {
                assert_eq!(
                    (root, &content),
                    (pool.recorded.previous_key, &pool.recorded.previous_content),
                    "恢复只会退到 P、读回 P 的内容"
                );
                Landing::OnTheVersion(root)
            }
            RecoveryOutcome::Failed {
                root: Some(root), ..
            } if root == pool.recorded.key => Landing::ReportedTheChosenVersionUnreadable,
            other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
                panic!("恢复要么退到 P，要么落在所选的 R 上照报读不出，实际 {other:?}")
            }
        },
        Reader::ReadOnlyMount => match mount_read_only(&devices) {
            Ok(mounted) => Landing::OnTheVersion((
                mounted.effective_root.instance,
                mounted.effective_root.checkpoint_txg,
            )),
            Err(_) => Landing::ReportedTheChosenVersionUnreadable,
        },
        Reader::WritableMount => {
            match mount_writable(&pool.recorded.pool.parameters, &mut devices) {
                Ok(mounted) => Landing::OnTheVersion((
                    mounted.output.effective_root.instance,
                    mounted.output.effective_root.checkpoint_txg,
                )),
                Err(_) => {
                    let devices_without_the_slot_reading_bad: Vec<(
                        DeviceIdentity,
                        DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
                    )> = devices
                        .into_iter()
                        .map(|(identity, device_with_the_slot_reading_bad)| {
                            (identity, device_with_the_slot_reading_bad.inner)
                        })
                        .collect();
                    assert!(
                        image_of_devices_reading_back_all_zero(
                            &devices_without_the_slot_reading_bad,
                            pool.recorded.pool.device_bytes
                        ) == pool.image,
                        "可写挂载拒的时候在任何写之前拒：两块盘逐字节不变"
                    );
                    Landing::ReportedTheChosenVersionUnreadable
                }
            }
        }
    };
    (landing, slot.reads_of_the_slot.get())
}

/// 三处各自：不坏时退到 P；P 的根槽每一读都坏时照报 R 读不出（Y4）；不坏那一趟碰到 P 的根槽的每一次读，单坏那一次，都与不坏时一样退到 P。
fn the_root_slot_of_the_previous_version_reading_bad_once_still_falls_back_to_it(
    bad_read: BadRead,
) {
    let pool = fall_back_to_the_confirmed_previous();
    for reader in [
        Reader::Recovery,
        Reader::ReadOnlyMount,
        Reader::WritableMount,
    ] {
        let (clean, reads_of_the_slot) =
            landing_of(reader, &pool, ReadsOfTheSlotThatGoBad::Never, bad_read);
        assert_eq!(
            clean,
            Landing::OnTheVersion(pool.recorded.previous_key),
            "{reader:?}：不坏时退到已确认的 P"
        );
        assert!(
            reads_of_the_slot >= 2,
            "{reader:?}：不坏那一趟至少读 P 的根槽两次（择根一次、F2 判候选至少一次），实际 {reads_of_the_slot}"
        );
        assert_eq!(
            landing_of(reader, &pool, ReadsOfTheSlotThatGoBad::Every, bad_read).0,
            Landing::ReportedTheChosenVersionUnreadable,
            "{reader:?}：P 的根槽每一读都坏，照报 R 读不出，不退过 P 落到 Pm（Y4）"
        );
        let landings_that_differ: Vec<(usize, Landing)> = (1..=reads_of_the_slot)
            .map(|number| {
                (
                    number,
                    landing_of(
                        reader,
                        &pool,
                        ReadsOfTheSlotThatGoBad::OnlyTheReadNumbered(number),
                        bad_read,
                    )
                    .0,
                )
            })
            .filter(|(_, landing)| *landing != clean)
            .collect();
        assert_eq!(
            landings_that_differ,
            Vec::new(),
            "{reader:?}：P 的根槽只坏一次（碰到它的第几次读），落到的应与不坏时相同（退到 P），列出的是坏了哪一读、落到哪"
        );
    }
}

#[test]
fn the_root_slot_of_the_previous_version_unreadable_once_still_falls_back_to_it_in_recovery_and_both_mounts(
) {
    the_root_slot_of_the_previous_version_reading_bad_once_still_falls_back_to_it(
        BadRead::Unreadable,
    );
}

#[test]
fn the_root_slot_of_the_previous_version_not_self_verified_once_still_falls_back_to_it_in_recovery_and_both_mounts(
) {
    the_root_slot_of_the_previous_version_reading_bad_once_still_falls_back_to_it(
        BadRead::FirstByteFlipped,
    );
}
