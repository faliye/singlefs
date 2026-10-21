//! 欠账 C563（`.claude/kb/checks-owed.md`「读根环一槽读坏就当没有根，别的调用方没判」）剩下两格与 C584（「写系统配置前与回退水位读不出仍当没有根」）
//! 叠加故障那一格的回归用例：三格都量过、今天都没有后果，这几条钉住今天的结局。
//! - 只读挂载择根：择根读坏的槽给不出根（`recovery::choose_root` 把读不出与自证不过都当没有根），择到环里次新的那条，
//!   journal 重放把版本补回来；跨实例那一形重放停在实例边界，落到上一个实例的最后一版，那几次发布不改文件，读到的内容相同。
//! - 「环里最旧有效根」门槛（D16（发布语义） 已定项 1）：挂载那一刻读坏的槽不当没有根，钉在它可能住过的最旧 txg
//!   （`allocator::RootRingOccupancy` 的 `BadAtMount`）；钉是承重的，去掉它门槛越过那条根、回收它还引用着的单元。
//! - C584 叠加故障：抬 F 先写系统配置那一步只落了一块盘的一槽就断电，那一槽是带新 F 的唯一载体；它在下一次可写挂载里一直读不出时，
//!   重建按 F 0 回收、之后的写也按 0 写，盘上 F 落回 0，回收与盘上的 F 前后一致，回退回去的那一版内容原样。
//!   「抬 F 那一串」那一行写「F 的生效值只增不减」，盘上 F 落回 0 在字面上打破它：这条钉的是今天的结局，条款怎么处置归主 agent。
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs"]
mod scenarios;

use scenarios::{
    content_of, pool_after_the_first_file, PoolAfterTheFirstFile, SparseDevices, FIRST_FILE_BYTES,
    FOUR_GIBIBYTES, WRITE_TIME_SECONDS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish,
    RollbackCandidateExclusion, RollbackError, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, effective_rollback_floor, readable_roots_with_ring_slots,
    walk_to_file, PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{SparseBlockDevice, WrittenContents};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// 点名的那一段每一次读怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BadReadOfTheChosenRange {
    /// 块设备报 `InputOutput`。
    Unreadable,
    /// 读得出，这一段的第一个字节取反（根槽里是 magic 那一字节：自证不过）。
    FirstByteFlipped,
}

/// 一块盘上点名的一段：碰到它的每一次读都坏。
#[derive(Clone, Copy, Debug)]
struct RangeReadingBad {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    bad_read: BadReadOfTheChosenRange,
}

/// 落到盘上的一次写（写清零只带长度），按发出的先后记：拿来造「这一串写只落了前几次就断电」的镜像。
#[derive(Clone, Debug)]
struct WriteThatLanded {
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    contents: WrittenContents,
}

/// 一块内存盘：点名的那一段每次读都坏（没点名就不坏），写照转给里面那块稀疏盘并记进几块盘共用的写表。
/// discard 走默认实现（不认），不进写表：这几条用例按写的先后造断电前缀。
struct DeviceWithARangeReadingBad {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    range_reading_bad: Option<RangeReadingBad>,
    writes_that_landed: Rc<RefCell<Vec<WriteThatLanded>>>,
}

impl BlockDevice for DeviceWithARangeReadingBad {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let overlapping_bad_range = self.range_reading_bad.filter(|range| {
            range.device == self.identity
                && offset.0 < range.offset_in_bytes + range.length_in_bytes
                && range.offset_in_bytes < offset.0 + length_in_bytes
        });
        match overlapping_bad_range {
            None => self.inner.read_at(offset, buffer),
            Some(RangeReadingBad {
                bad_read: BadReadOfTheChosenRange::Unreadable,
                ..
            }) => Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一段读不出",
            ))),
            Some(RangeReadingBad {
                bad_read: BadReadOfTheChosenRange::FirstByteFlipped,
                offset_in_bytes,
                ..
            }) => {
                self.inner.read_at(offset, buffer)?;
                let first_byte_of_the_range_in_the_read =
                    usize::try_from(offset_in_bytes.saturating_sub(offset.0))
                        .expect("一次读里的偏移装得进 usize");
                buffer[first_byte_of_the_range_in_the_read] ^= 0xff;
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
        self.inner.write_at(offset, bytes, durability)?;
        self.writes_that_landed.borrow_mut().push(WriteThatLanded {
            device: self.identity,
            offset,
            contents: WrittenContents::Bytes(bytes.to_vec()),
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)?;
        self.writes_that_landed.borrow_mut().push(WriteThatLanded {
            device: self.identity,
            offset,
            contents: WrittenContents::Zeros { length },
        });
        Ok(())
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

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

type DevicesWithARangeReadingBad = Vec<(DeviceIdentity, DeviceWithARangeReadingBad)>;

fn copy_of(devices: &SparseDevices) -> SparseDevices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copied =
                SparseBlockDevice::new(device.size_in_bytes(), device.probe_physical_block_size());
            copied.image = device.image.clone();
            (*identity, copied)
        })
        .collect()
}

/// 给一份盘面的拷贝包上读坏那一层（`range_reading_bad` 是 None 就不坏），交回包好的盘与它们共用的写表。
fn copy_reading_bad(
    devices: &SparseDevices,
    range_reading_bad: Option<RangeReadingBad>,
) -> (
    DevicesWithARangeReadingBad,
    Rc<RefCell<Vec<WriteThatLanded>>>,
) {
    let writes_that_landed = Rc::new(RefCell::new(Vec::new()));
    let wrapped = copy_of(devices)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceWithARangeReadingBad {
                    identity,
                    inner,
                    range_reading_bad,
                    writes_that_landed: writes_that_landed.clone(),
                },
            )
        })
        .collect();
    (wrapped, writes_that_landed)
}

/// 这一刻每块盘上的盘面（拷贝，不带读坏那一层）。
fn images_of(devices: &DevicesWithARangeReadingBad) -> SparseDevices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copied = SparseBlockDevice::new(
                device.inner.size_in_bytes(),
                device.inner.probe_physical_block_size(),
            );
            copied.image = device.inner.image.clone();
            (*identity, copied)
        })
        .collect()
}

fn without_the_bad_reads(devices: DevicesWithARangeReadingBad) -> SparseDevices {
    devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect()
}

/// `before` 上只落了 `writes` 这几次写就断电的盘面。
fn crash_image(before: &SparseDevices, writes: &[WriteThatLanded]) -> SparseDevices {
    let mut image = copy_of(before);
    for write in writes {
        let device = &mut image
            .iter_mut()
            .find(|(identity, _)| *identity == write.device)
            .expect("写表里的写都落在池里的盘上")
            .1;
        write.contents.apply_to(&mut device.image, write.offset);
    }
    image
}

/// 根环里住着 `version` 那条根的槽（读得出、自证过的那一条）。
fn ring_slot_holding(
    devices: &SparseDevices,
    version: (InstanceGeneration, CheckpointTxg),
) -> RootRingSlot {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得出");
    readable_roots_with_ring_slots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|(_, root)| key_of(root) == version)
    .map(|(ring_slot, _)| ring_slot)
    .expect("那一版的根在根环里读得出")
}

/// 一个根环槽整槽每次读都按 `bad_read` 坏。
fn root_ring_slot_reading_bad(
    devices: &SparseDevices,
    ring_slot: RootRingSlot,
    bad_read: BadReadOfTheChosenRange,
) -> RangeReadingBad {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得出");
    let sizes = system_configuration.immutable.sizes;
    RangeReadingBad {
        device: system_configuration.immutable.region_devices
            [usize::try_from(ring_slot.region).expect("区域号装得进 usize")],
        offset_in_bytes: slot_offset(ring_slot, sizes.fixed_structure_slot_spacing).0,
        length_in_bytes: u64::from(sizes.physical_block_size),
        bad_read,
    }
}

fn key_of(root: &RootRecord) -> (InstanceGeneration, CheckpointTxg) {
    (root.instance, root.checkpoint_txg)
}

fn version_key(
    instance_generation: u32,
    checkpoint_txg: u64,
) -> (InstanceGeneration, CheckpointTxg) {
    (
        InstanceGeneration(instance_generation),
        CheckpointTxg(checkpoint_txg),
    )
}

/// 只读挂载看到的：所选根、施加之后的根、文件（inode 1）的内容。
#[derive(Debug, PartialEq, Eq)]
struct ReadOnlyMountView {
    chosen_root: (InstanceGeneration, CheckpointTxg),
    effective_root: (InstanceGeneration, CheckpointTxg),
    file_content: Vec<u8>,
}

fn read_only_mount_view(reader: &dyn PoolReader) -> ReadOnlyMountView {
    let mounted = mount_read_only(reader).expect("只读挂载做得成");
    let inode_record = mounted
        .mounted
        .inode_records()
        .iter()
        .find(|record| record.inode == 1)
        .copied()
        .expect("第一个文件那一版起每一版都有 inode 1");
    let file_content = mounted
        .mounted
        .open_file(reader, InodeNumber(1))
        .expect("挂上的那一版打得开 inode 1")
        .read_at(reader, FileOffsetInBytes(0), inode_record.size)
        .expect("挂上的那一版读得出 inode 1 的全部内容")
        .bytes;
    ReadOnlyMountView {
        chosen_root: key_of(&mounted.chosen_root),
        effective_root: key_of(&mounted.effective_root),
        file_content,
    }
}

/// 第 `seed` 次覆盖写的内容：长度 2000 + 97 × seed，各次不同。
fn content_of_the_overwrite(seed: u64) -> Vec<u8> {
    content_of(
        2000 + 97 * usize::try_from(seed).expect("覆盖写的序号装得进 usize"),
        seed,
    )
}

fn overwrite_with<Device: BlockDevice>(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &TransactionOutput,
    instance: InstanceGeneration,
    content: &[u8],
) -> TransactionOutput {
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        allocator,
        current,
        FirstFile {
            content,
            write_time_seconds: WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写做得成")
}

/// 第一个文件（txg 3）之后在同一个进程里覆盖写 `overwrites` 次（txg 4..3 + overwrites），不断电。
fn pool_after_overwrites(overwrites: u64) -> PoolAfterTheFirstFile {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    for seed in 1..=overwrites {
        pool.first = overwrite_with(
            &pool.parameters,
            &mut pool.devices,
            &mut pool.allocator,
            &pool.first,
            pool.instance,
            &content_of_the_overwrite(seed),
        );
    }
    pool
}

/// 每块盘的 (已分配, defer 着, 空闲) 槽数。
fn slot_counts_per_device(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64, u64, u64)> {
    allocator
        .devices
        .iter()
        .map(|device| {
            (
                device.device,
                device.allocated_slots(),
                device.deferred_slots(),
                device.free_slots(),
            )
        })
        .collect()
}

// ───────── C563 只读挂载择根 ─────────

/// 第一个文件之后覆盖写五次（txg 3..8），最新那条根（txg 8）的槽读坏：择到 txg 7，journal 重放补回 txg 8，读到的文件与不坏时相同。
fn a_read_only_mount_with_the_newest_root_ring_slot_reading_bad(bad_read: BadReadOfTheChosenRange) {
    let pool = pool_after_overwrites(5);
    let clean = read_only_mount_view(&pool.devices);
    assert_eq!(
        (clean.chosen_root, clean.effective_root),
        (version_key(1, 8), version_key(1, 8)),
        "不坏时择到最新那条根 txg 8"
    );
    let newest_slot = ring_slot_holding(&pool.devices, version_key(1, 8));
    let (devices_reading_bad, _) = copy_reading_bad(
        &pool.devices,
        Some(root_ring_slot_reading_bad(
            &pool.devices,
            newest_slot,
            bad_read,
        )),
    );
    let faulty = read_only_mount_view(&devices_reading_bad);
    assert_eq!(
        faulty.chosen_root,
        version_key(1, 7),
        "txg 8 那一槽读坏给不出根，择到环里次新的 txg 7"
    );
    assert_eq!(
        faulty.effective_root,
        version_key(1, 8),
        "txg 8 那次发布的记录在 journal 环里，重放把版本补回 txg 8"
    );
    assert_eq!(
        faulty.file_content,
        content_of_the_overwrite(5),
        "读到的是第五次覆盖写的内容，与不坏时相同"
    );
    assert_eq!(faulty.file_content, clean.file_content);
}

#[test]
fn a_read_only_mount_with_the_newest_root_ring_slot_unreadable_chooses_the_next_newest_root_and_the_replay_brings_back_the_newest_version(
) {
    a_read_only_mount_with_the_newest_root_ring_slot_reading_bad(
        BadReadOfTheChosenRange::Unreadable,
    );
}

#[test]
fn a_read_only_mount_with_the_newest_root_ring_slot_not_self_verified_chooses_the_next_newest_root_and_the_replay_brings_back_the_newest_version(
) {
    a_read_only_mount_with_the_newest_root_ring_slot_reading_bad(
        BadReadOfTheChosenRange::FirstByteFlipped,
    );
}

/// 第一个文件之后覆盖写六次（txg 9 写 5000 字节），之后可写挂载换实例 2：写行那次发布（txg 10）的根落盘之后、暖机写下一条根之前断电。
/// 这一刻最新的根是 (2, 10)，它的槽读不出时只读挂载择到 (1, 9)，重放只沿实例 1 往下接、停在实例边界，落到 (1, 9)；
/// 写行那次发布不改文件，读到的内容与不坏时相同（不坏时择到、施加之后都是 (2, 10)）。
#[test]
fn a_read_only_mount_after_a_crash_inside_the_writable_mount_with_the_new_instances_root_unreadable_stays_on_the_last_version_of_the_previous_instance(
) {
    let mut pool = pool_after_overwrites(5);
    let content_of_txg_nine = content_of(5000, 77);
    pool.first = overwrite_with(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &pool.first,
        pool.instance,
        &content_of_txg_nine,
    );
    assert_eq!(key_of(&pool.first.root), version_key(1, 9));
    let (mut recording, writes_of_the_mount) = copy_reading_bad(&pool.devices, None);
    mount_writable(&pool.parameters, &mut recording).expect("可写挂载做得成");
    let after_the_mount = without_the_bad_reads(recording);
    let ring_slot_of_the_row_publish = ring_slot_holding(&after_the_mount, version_key(2, 10));
    let row_publish_slot = root_ring_slot_reading_bad(
        &after_the_mount,
        ring_slot_of_the_row_publish,
        BadReadOfTheChosenRange::Unreadable,
    );
    let writes_of_the_mount = writes_of_the_mount.borrow().clone();
    let root_write_of_the_row_publish = writes_of_the_mount
        .iter()
        .position(|write| {
            write.device == row_publish_slot.device
                && write.offset == DeviceOffsetInBytes(row_publish_slot.offset_in_bytes)
        })
        .expect("可写挂载写了写行那次发布的根");
    let crashed = crash_image(
        &pool.devices,
        &writes_of_the_mount[..=root_write_of_the_row_publish],
    );
    let clean = read_only_mount_view(&crashed);
    assert_eq!(
        (clean.chosen_root, clean.effective_root),
        (version_key(2, 10), version_key(2, 10)),
        "不坏时择到写行那次发布的根 (2, 10)"
    );
    let (devices_reading_bad, _) = copy_reading_bad(&crashed, Some(row_publish_slot));
    let faulty = read_only_mount_view(&devices_reading_bad);
    assert_eq!(
        faulty.chosen_root,
        version_key(1, 9),
        "(2, 10) 那一槽读不出给不出根，择到实例 1 的最后一条 (1, 9)"
    );
    assert_eq!(
        faulty.effective_root,
        version_key(1, 9),
        "重放只沿实例 1 往下接，停在实例边界：落到 (1, 9)，与可写挂载不坏时的 (2, 10) 不同"
    );
    assert_eq!(
        faulty.file_content, content_of_txg_nine,
        "写行那次发布不改文件：读到的仍是 txg 9 写的 5000 字节"
    );
    assert_eq!(faulty.file_content, clean.file_content);
}

// ───────── C563 「环里最旧有效根」门槛 ─────────

/// 同一份池（第一个文件之后覆盖写 26 次，根环 24 槽住着 txg 6..29）可写挂载，再在同一个进程里覆盖写两次；`range_reading_bad` 一直坏着。
/// 交回挂载之后与两次覆盖写之后各一份逐盘槽数，挂载那一刻钉住的读坏槽，与第一次覆盖写之后那一刻的盘面。
struct MountAndTwoOverwrites {
    slot_counts_after_the_mount: Vec<(DeviceIdentity, u64, u64, u64)>,
    slot_counts_after_the_first_overwrite: Vec<(DeviceIdentity, u64, u64, u64)>,
    slot_counts_after_the_second_overwrite: Vec<(DeviceIdentity, u64, u64, u64)>,
    ring_slots_pinned_at_the_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
    devices_after_the_first_overwrite: SparseDevices,
}

fn mount_and_two_overwrites(
    pool: &PoolAfterTheFirstFile,
    range_reading_bad: Option<RangeReadingBad>,
) -> MountAndTwoOverwrites {
    let (mut devices, _) = copy_reading_bad(&pool.devices, range_reading_bad);
    let mounted = mount_writable(&pool.parameters, &mut devices).expect("可写挂载做得成");
    let mut allocator = mounted.allocator;
    let instance = mounted.output.instance;
    let current = mounted
        .current
        .into_file_version()
        .expect("第一个文件之后每一版都带文件");
    let slot_counts_after_the_mount = slot_counts_per_device(&allocator);
    let ring_slots_pinned_at_the_mount = allocator
        .root_ring_occupancy()
        .expect("可写挂载重建分配器时装上了根环表")
        .ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held();
    let after_the_first_overwrite = overwrite_with(
        &pool.parameters,
        &mut devices,
        &mut allocator,
        &current,
        instance,
        &content_of(3100, 900),
    );
    let slot_counts_after_the_first_overwrite = slot_counts_per_device(&allocator);
    let devices_after_the_first_overwrite = images_of(&devices);
    overwrite_with(
        &pool.parameters,
        &mut devices,
        &mut allocator,
        &after_the_first_overwrite,
        instance,
        &content_of(3111, 901),
    );
    MountAndTwoOverwrites {
        slot_counts_after_the_mount,
        slot_counts_after_the_first_overwrite,
        slot_counts_after_the_second_overwrite: slot_counts_per_device(&allocator),
        ring_slots_pinned_at_the_mount,
        devices_after_the_first_overwrite,
    }
}

/// 环里第四旧那条根（txg 9）的槽在可写挂载与之后两次覆盖写里一直读不出：挂载把它钉在 txg 9，回收门槛不越过它，
/// 逐盘的已分配、defer、空闲槽数在挂载之后、每次覆盖写之后都与不坏时相同；第一次覆盖写之后 txg 9 那一版引用的单元没被复用，那一版照样走得通。
/// 读坏的槽当没有根（去掉钉）时门槛越过 txg 9、多回收它引用的单元：槽数与不坏时不同（调查时在这一形的断电前缀上量到池级 checker 报 I-3.1）。
#[test]
fn a_root_ring_slot_unreadable_through_the_writable_mount_is_pinned_at_its_oldest_txg_and_reclaims_no_more_than_when_it_reads(
) {
    let pool = pool_after_overwrites(26);
    let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置择得出");
    let mut roots_in_the_ring: Vec<(RootRingSlot, RootRecord)> = readable_roots_with_ring_slots(
        &pool.devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    roots_in_the_ring.sort_by_key(|(_, root)| key_of(root));
    assert_eq!(
        roots_in_the_ring
            .iter()
            .map(|(_, root)| root.checkpoint_txg)
            .collect::<Vec<_>>(),
        (6..=29).map(CheckpointTxg).collect::<Vec<_>>(),
        "根环 24 槽住满 txg 6..29"
    );
    let (fourth_oldest_slot, fourth_oldest_root) = roots_in_the_ring[3];
    assert_eq!(fourth_oldest_root.checkpoint_txg, CheckpointTxg(9));
    let reading = mount_and_two_overwrites(&pool, None);
    let reading_bad = mount_and_two_overwrites(
        &pool,
        Some(root_ring_slot_reading_bad(
            &pool.devices,
            fourth_oldest_slot,
            BadReadOfTheChosenRange::Unreadable,
        )),
    );
    assert_eq!(
        reading_bad.ring_slots_pinned_at_the_mount,
        BTreeMap::from([(fourth_oldest_slot, CheckpointTxg(9))]),
        "挂载那一刻读坏的槽钉在它可能住过的最旧 txg：txg 9"
    );
    assert_eq!(
        reading_bad.slot_counts_after_the_mount, reading.slot_counts_after_the_mount,
        "挂载重建分配器回收的与不坏时相同"
    );
    assert_eq!(
        reading_bad.slot_counts_after_the_first_overwrite,
        reading.slot_counts_after_the_first_overwrite,
        "第一次覆盖写（它的根盖掉 txg 8 那一槽）之后门槛仍停在钉住的 txg 9，回收的与不坏时相同"
    );
    assert_eq!(
        reading_bad.slot_counts_after_the_second_overwrite,
        reading.slot_counts_after_the_second_overwrite,
        "第二次覆盖写之后回收的与不坏时相同"
    );
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(
            &reading_bad.devices_after_the_first_overwrite,
            &fourth_oldest_root,
            &mut mapping_fallbacks,
        )
        .expect("txg 9 那一版引用的单元没被第一次覆盖写复用，走得通"),
        Some(content_of_the_overwrite(6)),
        "txg 9 那一版读出的是第六次覆盖写的内容"
    );
}

// ───────── C584 叠加故障 ─────────

/// 第一个文件之后覆盖写五次（txg 3..8），抬 F 到 5 那一串只落了它的第一写（盘 0 的一个系统配置槽）就断电：
/// 交回这张盘面与那一槽（整槽一段）。带新 F 的根一条都没发，那一槽是 F 5 唯一的载体。
fn crash_after_the_first_system_configuration_write_of_raising_the_floor_to_five(
) -> (PoolAfterTheFirstFile, SparseDevices, RangeReadingBad) {
    let pool = pool_after_overwrites(5);
    let (mut recording, writes_of_the_raise) = copy_reading_bad(&pool.devices, None);
    let mut allocator = pool.allocator.clone();
    let mut current = pool.first.clone();
    raise_rollback_floor(
        &pool.parameters,
        &mut recording,
        &mut allocator,
        &mut current,
        CheckpointTxg(5),
        ShadowLedger::On,
    )
    .expect("抬 F 到 5 做得成");
    let first_write = writes_of_the_raise
        .borrow()
        .first()
        .cloned()
        .expect("抬 F 那一串至少写了一次");
    let slot_spacing_in_bytes = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    assert_eq!(
        first_write.device,
        DeviceIdentity(0),
        "抬 F 那一串先写盘 0 的系统配置"
    );
    assert!(
        first_write.offset == DeviceOffsetInBytes(0)
            || first_write.offset == DeviceOffsetInBytes(slot_spacing_in_bytes),
        "抬 F 那一串的第一写落在盘 0 的一个系统配置槽上，实际偏移 {:?}",
        first_write.offset
    );
    let the_only_carrier = RangeReadingBad {
        device: first_write.device,
        offset_in_bytes: first_write.offset.0,
        length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
        bad_read: BadReadOfTheChosenRange::Unreadable,
    };
    let crashed = crash_image(&pool.devices, &[first_write]);
    (pool, crashed, the_only_carrier)
}

fn effective_floor_on_the_disk(devices: &SparseDevices) -> CheckpointTxg {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得出");
    effective_rollback_floor(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
}

/// 不坏时：断电之后的盘面上 F 生效值已是 5，可写挂载照 5 写下去；回退到 txg 3 低于 F，拒。
#[test]
fn a_floor_raise_cut_after_its_first_system_configuration_slot_is_kept_by_a_writable_mount_that_reads_that_slot(
) {
    let (pool, crashed, _) =
        crash_after_the_first_system_configuration_write_of_raising_the_floor_to_five();
    assert_eq!(
        effective_floor_on_the_disk(&crashed),
        CheckpointTxg(5),
        "断电之后盘上 F 生效值是那一槽带的 5"
    );
    let mut devices = copy_of(&crashed);
    let mounted = mount_writable(&pool.parameters, &mut devices).expect("可写挂载做得成");
    assert_eq!(
        effective_floor_on_the_disk(&devices),
        CheckpointTxg(5),
        "挂载读得出那一槽，照 5 写下去"
    );
    let mut allocator = mounted.allocator;
    let mut current = mounted
        .current
        .into_file_version()
        .expect("第一个文件之后每一版都带文件");
    let refused = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    assert!(
        matches!(
            refused,
            Err(RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
                ..
            })
        ),
        "txg 3 低于 F 5，回退拒成 BelowEffectiveFloor，实际 {refused:?}"
    );
}

/// 叠加故障：那一槽在下一次可写挂载里一直读不出。挂载做得成，重建按 F 0 回收、之后写下的 F 也是 0：盘上 F 生效值落回 0。
/// 回收与盘上的 F 前后一致：再覆盖写六次之后回退到 txg 3 做得成，读出的是第一个文件原样——txg 3 引用的单元没被复用。
#[test]
fn a_floor_raise_cut_after_its_first_system_configuration_slot_falls_back_to_zero_without_reusing_units_when_that_slot_stays_unreadable_through_the_writable_mount(
) {
    let (pool, crashed, the_only_carrier) =
        crash_after_the_first_system_configuration_write_of_raising_the_floor_to_five();
    let (mut devices_reading_bad, _) = copy_reading_bad(&crashed, Some(the_only_carrier));
    let mounted = mount_writable(&pool.parameters, &mut devices_reading_bad)
        .expect("那一槽一直读不出，可写挂载照样做得成");
    let mut devices = without_the_bad_reads(devices_reading_bad);
    assert_eq!(
        effective_floor_on_the_disk(&devices),
        CheckpointTxg(0),
        "重建读不出那一槽，按 F 0 算；挂完之后盘上的 F 生效值落回 0（「只增不减」字面上被打破，今天的结局）"
    );
    let mut allocator = mounted.allocator;
    let instance = mounted.output.instance;
    let mut current = mounted
        .current
        .into_file_version()
        .expect("第一个文件之后每一版都带文件");
    for seed in 0..6 {
        current = overwrite_with(
            &pool.parameters,
            &mut devices,
            &mut allocator,
            &current,
            instance,
            &content_of(2500 + 13 * seed, 400 + u64::try_from(seed).expect("小数")),
        );
    }
    roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    )
    .expect("F 生效值 0，txg 3 在候选集里，它引用的单元没被复用：回退做得成");
    assert_eq!(
        read_only_mount_view(&devices).file_content,
        content_of(FIRST_FILE_BYTES, 0),
        "回退回去读出的是第一个文件原样"
    );
}
