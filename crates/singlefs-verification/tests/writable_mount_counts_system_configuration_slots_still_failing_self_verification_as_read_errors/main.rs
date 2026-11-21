//! 验证包模块：crash
//! C971（多处读坏时可写挂载写出倒挂的新实例根）取乙入判不出（读法逐字在 `research/prompts/e205-preregistration.md`「乙入判不出」那一行）：
//! 可写挂载读阶段的见证读里，读得出而自证不过（撕裂、读回全零；整槽校验和过而 fsid 不是本池的槽也算：登记下一行「两种读法共用」写
//! 「自证过」要 fsid 与本池相同，C707（系统配置判不出里 fsid 不符落哪一边没写）定按自证不过处置）的系统配置槽并进 R2「有槽设备报读错」那一半当读错计，
//! R2 的另一半（没有一块盘两槽都自证过）照旧；判不出按判据 N-配置 为真，重读一次（系统配置槽直接从盘上重读）仍是这一形就拒可写，
//! 拒因是已有的 `MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`，
//! 两遍读数的比法都是 `WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots`。
//!
//! 盘面：
//! - C971 最小历史：mkfs 两块盘 → 实例 1 取号、暖机 txg 1、2 → 新池新建文件 txg 3 → 覆盖写 B txg 4、
//!   C txg 5、D txg 6；D 的根槽持久、两块盘的系统配置轮换没持久。六格（每区 8 槽、藏两条根）：C、D 的根槽与两次发布的全部记录
//!   （两块盘各一份）按三种之一读坏，每块盘自证过、世代号最大的那一槽系统配置按两种之一读坏；故障在这次挂载里不变（重读读到同样的字节）。
//!   产物 `research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out` 里六格乙入判不出那一列都是拒；
//!   同一份产物里丙（不带乙这一拒的实现）在这六格上可写挂载做成、写出倒挂的新实例根。
//! - 乙的三张手造盘面：同一段最小历史、不注读故障，只翻盘 0、翻两块盘、都不翻最新那一槽系统配置的一个字节；
//!   乙入判不出的答案「不拒、拒、不拒」。都不翻的那一张也是阳性对照：不注坏照常做成。
//! - 撕裂轮换族的一个承诺内单故障态（起头那一次可写挂载与它的第一个文件版本）：每区 4 槽，
//!   mkfs 之后第一次可写挂载、接着新池新建文件（txg 3）的写流里第 11 段（实例 1 txg 3 那次发布末尾的系统配置轮换，两块盘各一写）
//!   两写都撕裂、之前各段整段持久；这一态（`.claude/kb/experiments/205-多处读坏倒挂三条改法的代价.md` 第二段撕裂轮换，每区 4 槽 `segment=11`）丙做成、乙入判不出拒，
//!   落在 C596（见证判不出的系统配置盘面只剩只读挂载） 已定的冲突格里（设备没报读错、每块盘至少一槽自证过、坏的槽都读得出而自证不过）。
//! - 重读一次：最新那一槽在第一遍读里自证不过、重读之前撤掉，重读那一遍判得出，照常做成（读阶段在重读那一遍判完）。
//! - 重读粒度：重读是整个读阶段重做一遍、只按重读那一遍的读数判。最新那一槽在第一遍读里自证不过、较旧那一槽在第一遍读里好；
//!   重读之前撤掉第一遍的故障、改让较旧那一槽读坏，重读那一遍仍是乙入判不出，拒。
//! - 别池的槽：同一段最小历史、不注读故障，较旧那一槽换成别的池的系统配置（fsid 改一个字节、整槽校验和重封）。两块盘都换时
//!   没有一块盘两槽都自证过、坏槽只有别的池的槽，乙入判不出拒；只换盘 1（盘 0 两槽本池自证过）是阳性对照，照常做成。
//!
//! - 判官按本池 fsid 择系统配置：一块盘两槽都是别的池的、别的池的槽世代号比本池的大，判官照可写挂载的口径只从本池的槽里择，
//!   合法的拒判合法，不该有的拒判不合法（判据 N-配置 与「读阶段判完之后系统配置又读不出」各一条）。
//! - 现状钉子：每块盘世代号最大的一槽是别的池的（不带 fsid 的只读挂载读不到，离线出路走得通）与它的对照；一块盘两槽都落后而自证过放行倒挂、别的池又布局位不认识的槽放行倒挂（这两形条款不罩）。
//! - 带本池 fsid 的只读挂载与恢复：一块盘或每块盘世代号最大的一槽是别的池的，读到本池；交进来的 fsid 盘上一槽都没有的拒，不读盘上的池。
//!   判官的所选那一版取自按本池 fsid 的恢复；判官照产品读一槽的那几关数别的池的槽；崩溃态判官带与不带本池 fsid 两支的结局。
//! - 取第一个 txg：读阶段读到所选根、取第一个 txg 那一遍读坏，写行 txg 仍大于所选那一版。
//! - 新格式化的池上取号写撕裂：离线出路断言第 0 代（tail 取 mkfs 写的 0）之后可写挂载做成、发得出第一个文件版本；第 0 代之外的版本末条读不出照拒。
//!
//! 被拒的每一处另交挂载透明判官 `singlefs_verification::mount_transparency` 在同一份盘面上复算（它不调实现、自己写了一份乙入判不出），判合法。
use std::cell::Cell;
use std::rc::Rc;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    BeforeTheOneReread, DeviceNotRewrittenByTheAcquisition, MountError, Mounted,
    NewerPublishWitness, ReadStageSettled, RewriteGateNotMet, RollbackTarget,
    SelectedVersionAgainstTheWitness, ShadowLedger, StillUnreadableAfterOneReread,
    SystemConfigurationUnreadableAfterTheReadStage, SystemConfigurationUnreadableWhen,
    WitnessedCounterComparison,
};
use singlefs_core::mounted_read::{
    mount_read_only, mount_read_only_of_the_pool, MountReadOnlyFailure, OpenPoolForReadFailure,
};
use singlefs_core::recovery::{
    recover_of_the_pool, system_configuration_slots_of_one_device_read_once, JournalPolicy,
    PoolReader, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::system_configuration_repair::{
    rewrite_system_configuration_slots_for_the_asserted_version, AssertedVersion,
    SystemConfigurationRepairRefused,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_verification::crash::torn_image_of_in_place_overwrite;
use singlefs_verification::memory_pool::{
    root_identity_of_write, writes_and_segments_with_stream_indexes_and_entries, MemoryPool,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::mount_transparency::{
    judge_writable_mount_refusal, judge_writable_mount_refusal_of_a_crash_state_of_the_pool,
    read_recovery_fields, selected_version_of_the_pool_on_the_crash_image, ComparedVersion,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount,
    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, RefusalReasonOnTheLegalRefusalList,
    WritableMountRefusalVerdict,
};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

mod offline_way_out_after_a_torn_acquisition_round;
mod roots_read_bad_and_the_inverted_first_txg;
mod six_cells_and_hand_made_images_of_the_minimal_history;
mod slots_of_another_pool;
mod torn_rotations_and_slots_judged_by_the_reread;

/// 每块盘的字节数。
const DEVICE_BYTES: u64 = 4 << 30;

const PHYSICAL_BLOCK_BYTES: u32 = 512;

const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;

/// 根槽一段读坏的长度：一个物理块，根记录住在里面。
const ROOT_SLOT_BYTES: u64 = 512;

const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x43, 0x39, 0x37, 0x31, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];

const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

/// 读得出、不是全零而自证不过那一种读坏：读回的字节里翻这一个（落在槽与记录的校验和罩着的范围里）。
const FLIPPED_BYTE_INSIDE_A_RANGE: u64 = 100;

/// 第一个文件的内容长（一个数据单元以内）。
const FILE_CONTENT_BYTES: u64 = 3000;

/// 最小历史里最新那一版 D 的 txg；六格藏 D 与它前一版 C 两条根（E205 G 主点「藏两条」）。
const TXG_OF_THE_NEWEST_VERSION: u64 = 6;

const ROOTS_HIDDEN_IN_THE_SIX_CELLS: u64 = 2;

/// 六格与手造盘面的每区槽数（E205 G 主点）。
const SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY: u64 = 8;

/// 撕裂轮换那一态的每区槽数与它在写流里的段号（E205 第二段产物 `slots_per_region=4 segment=11` 那一行）。
/// 段落在第一个文件版本（txg 3）的发布末尾：这一版的树表有条目，只读挂载打得开文件（暖机根那几版树表 0 条，
/// 只读挂载照报 `TreeTableHasNoEntries`，判不了「只读挂载照常」）。
const SLOTS_PER_REGION_OF_THE_TORN_ROTATION_STATE: u64 = 4;

const SEGMENT_OF_THE_TORN_ROTATION_STATE: usize = 11;

/// 那一段轮换前面那次发布：第一个文件版本。
const TXG_OF_THE_PUBLISH_BEFORE_THE_TORN_ROTATION: u64 = 3;

/// 系统配置槽里 incompat 位图的起始偏移（系统配置字段表）。
const INCOMPAT_BITMAP_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT: usize = 6;

/// 系统配置槽里区域数字段的偏移（系统配置字段表）：checker 判它的区间，产品读这一槽不判。
const REGION_COUNT_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT: usize = 361;

/// 系统配置槽里每区槽数的偏移（系统配置字段表）。
const ROOT_RING_SLOTS_PER_REGION_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT: usize = 362;

fn pool_parameters(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("用例取的每区槽数是 4 与 8，都在系统配置收的区间里"),
        },
    }
}

/// 第 `index` 版内容：序号大的版本新，读回来的字节认得出是哪一版。
fn file_content_of(index: u64) -> Vec<u8> {
    (0..FILE_CONTENT_BYTES)
        .map(|byte| u8::try_from((byte * 7 + index) % 251).expect("模 251 装得进 u8"))
        .collect()
}

fn empty_sparse_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                ),
            )
        })
        .collect()
}

fn memory_pool_of_sparse_devices<Device>(
    devices: &[(DeviceIdentity, Device)],
    image_of: impl Fn(&Device) -> &SparseBlockDevice,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, image_of(device).image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// 一次读坏是哪一种（E205 登记第一节「六格」：设备报读错 / 读回全零 / 读得出而自证不过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadFaultKind {
    DeviceReportsAReadError,
    ReadsBackAllZeros,
    ReadsBackBytesFailingSelfVerification,
}

/// 一段读坏的落点：哪块盘、从哪个字节起、多长、哪一种。被这次挂载写过就撤掉（之后读回新字节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadFault {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    kind: ReadFaultKind,
}

impl ReadFault {
    fn overlaps(&self, device: DeviceIdentity, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
        self.device == device
            && offset_in_bytes < self.offset_in_bytes + self.length_in_bytes
            && self.offset_in_bytes < offset_in_bytes + length_in_bytes
    }
}

/// 照读故障改读回的字节；设备报读错交回 `false`（调用方报读错）。
fn apply_read_faults(
    faults: &[ReadFault],
    device: DeviceIdentity,
    offset_in_bytes: u64,
    buffer: &mut [u8],
) -> bool {
    let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
    for fault in faults
        .iter()
        .filter(|fault| fault.overlaps(device, offset_in_bytes, length_in_bytes))
    {
        let start = usize::try_from(fault.offset_in_bytes.max(offset_in_bytes) - offset_in_bytes)
            .expect("落在缓冲区里");
        let end = usize::try_from(
            (fault.offset_in_bytes + fault.length_in_bytes).min(offset_in_bytes + length_in_bytes)
                - offset_in_bytes,
        )
        .expect("落在缓冲区里");
        match fault.kind {
            ReadFaultKind::DeviceReportsAReadError => return false,
            ReadFaultKind::ReadsBackAllZeros => buffer[start..end].fill(0),
            ReadFaultKind::ReadsBackBytesFailingSelfVerification => {
                let flipped = fault.offset_in_bytes + FLIPPED_BYTE_INSIDE_A_RANGE;
                let index =
                    if flipped >= offset_in_bytes && flipped < offset_in_bytes + length_in_bytes {
                        usize::try_from(flipped - offset_in_bytes).expect("落在缓冲区里")
                    } else {
                        start
                    };
                buffer[index] ^= 0xff;
            }
        }
    }
    true
}

/// 读的时候按 `faults` 读坏的盘：故障在这次挂载里不变（重读读到同样的字节），用例在「重读一次」之前的钩子里把 `faults_lifted` 置真，
/// `faults` 就整个撤掉、之后的读改按 `faults_from_the_one_reread_on` 读坏（空的就是只撤不换）；
/// 写、写零、discard 原样交给里面那块盘，写到的故障段（两份里的）当场撤掉。
struct DeviceWithReadFaults {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    faults: Vec<ReadFault>,
    faults_from_the_one_reread_on: Vec<ReadFault>,
    faults_lifted: Rc<Cell<bool>>,
}

impl DeviceWithReadFaults {
    fn lift_the_faults_written_over(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        let identity = self.identity;
        self.faults
            .retain(|fault| !fault.overlaps(identity, offset_in_bytes, length_in_bytes));
        self.faults_from_the_one_reread_on
            .retain(|fault| !fault.overlaps(identity, offset_in_bytes, length_in_bytes));
    }
}

impl BlockDevice for DeviceWithReadFaults {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let faults_in_force = if self.faults_lifted.get() {
            &self.faults_from_the_one_reread_on
        } else {
            &self.faults
        };
        if apply_read_faults(faults_in_force, self.identity, offset.0, buffer) {
            Ok(())
        } else {
            Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一段读坏：设备报读错",
            )))
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.lift_the_faults_written_over(
            offset.0,
            u64::try_from(bytes.len()).expect("写长装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.lift_the_faults_written_over(offset.0, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.lift_the_faults_written_over(offset.0, length);
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

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// 一张镜像带一份读故障，当挂载透明判官读的崩溃镜像（与被拒的那次挂载读到的是同一份字节）。
struct ImageWithReadFaults<'image> {
    image: &'image MemoryPool,
    faults: &'image [ReadFault],
}

impl PoolReader for ImageWithReadFaults<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.image.device_identities()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.image, device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        PoolReader::physical_block_size_probed_on_the_device(self.image, device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let mut bytes = PoolReader::read(self.image, device, offset, length)?;
        apply_read_faults(self.faults, device, offset.0, &mut bytes).then_some(bytes)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
    }
}

/// 一张镜像拷成两块带读故障的内存盘，`faults_lifted` 置真就整个撤掉；`faults_lifted` 几块盘共用一份。
fn devices_with_read_faults(
    image: &MemoryPool,
    faults: &[ReadFault],
    faults_lifted: &Rc<Cell<bool>>,
) -> Vec<(DeviceIdentity, DeviceWithReadFaults)> {
    devices_with_read_faults_replaced_at_the_one_reread(image, faults, &[], faults_lifted)
}

/// 一张镜像拷成两块带读故障的内存盘：`faults_lifted` 置真之前按 `faults` 读坏，之后按 `faults_from_the_one_reread_on` 读坏；
/// `faults_lifted` 几块盘共用一份。
fn devices_with_read_faults_replaced_at_the_one_reread(
    image: &MemoryPool,
    faults: &[ReadFault],
    faults_from_the_one_reread_on: &[ReadFault],
    faults_lifted: &Rc<Cell<bool>>,
) -> Vec<(DeviceIdentity, DeviceWithReadFaults)> {
    let faults_on = |all_faults: &[ReadFault], identity: DeviceIdentity| -> Vec<ReadFault> {
        all_faults
            .iter()
            .filter(|fault| fault.device == identity)
            .copied()
            .collect()
    };
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner = SparseBlockDevice::new(
                DEVICE_BYTES,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            );
            inner.image = sparse.clone();
            (
                *identity,
                DeviceWithReadFaults {
                    identity: *identity,
                    inner,
                    faults: faults_on(faults, *identity),
                    faults_from_the_one_reread_on: faults_on(
                        faults_from_the_one_reread_on,
                        *identity,
                    ),
                    faults_lifted: Rc::clone(faults_lifted),
                },
            )
        })
        .collect()
}

/// C971 最小历史（不造倒挂对照镜像）：交回 D 写完、系统配置轮换退回 D 之前字节的镜像，
/// 与 txg 3 起每次发布全部记录的计数器。
struct MinimalHistory {
    image: MemoryPool,
    record_counters_by_txg: Vec<(u64, Vec<u64>)>,
}

fn record_counters_of(output: &TransactionOutput) -> Vec<u64> {
    output
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain(std::iter::once(output.record.counter))
        .collect()
}

fn minimal_history_of_c971(parameters: &MakeFilesystemParameters) -> MinimalHistory {
    let mut devices = empty_sparse_devices();
    let genesis = make_filesystem(parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), DEVICE_BYTES),
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
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        parameters, &genesis,
    ));
    let mut publishes: Vec<TransactionOutput> = Vec::new();
    {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1), "最小历史是实例 1");
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_content = file_content_of(1);
        publishes.push(
            publish_first_file(
                &mut writer,
                &mut allocator,
                warm.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &first_content,
                    write_time_seconds: FIRST_WRITE_TIME_SECONDS,
                },
                instance,
                &warm.last_record_bytes,
            )
            .expect("新池新建文件"),
        );
    }
    let mut before_the_newest_version = None;
    for index in 2..=4u64 {
        if index == 4 {
            before_the_newest_version =
                Some(memory_pool_of_sparse_devices(&devices, |device| device));
        }
        let content = file_content_of(index);
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let next = publish_overwrite(
            &mut writer,
            &mut allocator,
            publishes.last().expect("有上一版"),
            FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 60 * (index - 1),
            },
            InstanceGeneration(1),
        )
        .expect("覆盖写");
        publishes.push(next);
    }
    assert_eq!(
        publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        vec![3, 4, 5, TXG_OF_THE_NEWEST_VERSION],
        "新池新建文件 txg 3，B、C、D 是 txg 4、5、6"
    );
    let before_the_newest_version = before_the_newest_version.expect("D 之前拍过镜像");
    let mut image = memory_pool_of_sparse_devices(&devices, |device| device);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, u64::from(FIXED_STRUCTURE_SLOT_SPACING)] {
            let bytes = PoolReader::read(
                &before_the_newest_version,
                device,
                DeviceOffsetInBytes(offset),
                slot_bytes,
            )
            .expect("D 之前的系统配置槽读得到");
            image
                .devices
                .get_mut(&device)
                .expect("有这块盘")
                .write(DeviceOffsetInBytes(offset), &bytes);
        }
    }
    MinimalHistory {
        image,
        record_counters_by_txg: publishes
            .iter()
            .map(|publish| (publish.root.checkpoint_txg.0, record_counters_of(publish)))
            .collect(),
    }
}

/// 每块盘两槽里自证过、世代号最大的那一槽的偏移。
fn newest_system_configuration_slot(image: &MemoryPool, device: DeviceIdentity) -> u64 {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    [0, u64::from(FIXED_STRUCTURE_SLOT_SPACING)]
        .into_iter()
        .filter_map(|offset| {
            let bytes = PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes)?;
            SystemConfiguration::parse_slot(&bytes)
                .ok()
                .map(|configuration| (configuration.quantities.slot_generation, offset))
        })
        .max()
        .expect("每块盘至少一槽系统配置自证过")
        .1
}

/// 每块盘两槽里不是世代号最大的那一槽（[`newest_system_configuration_slot`] 之外的另一槽）的偏移。
fn older_system_configuration_slot(image: &MemoryPool, device: DeviceIdentity) -> u64 {
    let newest = newest_system_configuration_slot(image, device);
    [0, u64::from(FIXED_STRUCTURE_SLOT_SPACING)]
        .into_iter()
        .find(|offset| *offset != newest)
        .expect("一块盘两槽，世代号最大的之外还有一槽")
}

/// 每块盘自证过、世代号最大的那一槽按 `kind` 读坏。
fn newest_system_configuration_slot_of_each_device_read_bad(
    image: &MemoryPool,
    kind: ReadFaultKind,
) -> Vec<ReadFault> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| ReadFault {
            device,
            offset_in_bytes: newest_system_configuration_slot(image, device),
            length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
            kind,
        })
        .collect()
}

/// 六格的一格：D、C 两条根的根槽与两次发布的全部记录（两块盘各一份）按 `root_fault` 读坏，
/// 每块盘最新那一槽系统配置按 `system_configuration_fault` 读坏。
fn read_faults_of_a_six_cell(
    parameters: &MakeFilesystemParameters,
    history: &MinimalHistory,
    root_fault: ReadFaultKind,
    system_configuration_fault: ReadFaultKind,
) -> Vec<ReadFault> {
    let mut faults = Vec::new();
    for txg in (0..ROOTS_HIDDEN_IN_THE_SIX_CELLS).map(|back| TXG_OF_THE_NEWEST_VERSION - back) {
        let target = target_for_publish(
            CheckpointTxg(txg),
            parameters.geometry.root_ring_slots_per_region,
        );
        faults.push(ReadFault {
            device: parameters.region_devices[usize::try_from(target.region).expect("区域号")],
            offset_in_bytes: slot_offset(target, parameters.geometry.fixed_structure_slot_spacing)
                .0,
            length_in_bytes: ROOT_SLOT_BYTES,
            kind: root_fault,
        });
        let counters = &history
            .record_counters_by_txg
            .iter()
            .find(|(record_txg, _)| *record_txg == txg)
            .expect("最小历史里有这一版")
            .1;
        for counter in counters {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                faults.push(ReadFault {
                    device,
                    offset_in_bytes: record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES).0,
                    length_in_bytes: JOURNAL_RECORD_BYTES,
                    kind: root_fault,
                });
            }
        }
    }
    faults.extend(newest_system_configuration_slot_of_each_device_read_bad(
        &history.image,
        system_configuration_fault,
    ));
    faults
}

/// 一次带读故障的可写挂载（产品入口 `mount_writable`）：交回结局与挂载之后的镜像。
fn writable_mount_under_read_faults(
    parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    faults: &[ReadFault],
) -> (Result<Mounted, MountError>, MemoryPool) {
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(image, faults, &faults_lifted);
    let outcome = mount_writable(parameters, &mut devices);
    let image_after = memory_pool_of_sparse_devices(&devices, |device| &device.inner);
    (outcome, image_after)
}

/// 拒因是判据 N-配置 为真、重读一次仍为真，两遍读数的比法都是乙入判不出那一形。
fn refused_as_failing_self_verification_on_both_reads(
    outcome: &Result<Mounted, MountError>,
) -> bool {
    let Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) = outcome else {
        return false;
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read,
        reread,
    } = still_unreadable.as_ref()
    else {
        return false;
    };
    [first_read, reread].iter().all(|reading| {
        reading.witness.comparison
            == WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots
    })
}

/// 挂载透明判官在被拒的那份盘面（带同一份读故障）上判这一拒（U6）：按本池 fsid 的只读恢复落到的那一版当所选那一版，盘上逐字节不变由用例另核。
fn verdict_of_the_refusal_on(
    crash_image: &dyn PoolReader,
    refusal: &Result<Mounted, MountError>,
) -> WritableMountRefusalVerdict {
    let Err(error) = refusal else {
        panic!("这里只判被拒的挂载");
    };
    judge_writable_mount_refusal(
        crash_image,
        &FILESYSTEM_IDENTIFIER,
        selected_version_of_the_pool_on_the_crash_image(crash_image, &FILESYSTEM_IDENTIFIER),
        error,
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
    )
}

/// 六格的一格：可写挂载重读一次仍判不出、拒，盘上逐字节不变（没写出倒挂的新实例根，也没写出别的什么）；挂载透明判官判这一拒合法。
fn a_six_cell_is_refused_before_any_write(
    root_fault: ReadFaultKind,
    system_configuration_fault: ReadFaultKind,
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        root_fault,
        system_configuration_fault,
    );
    let (outcome, image_after) =
        writable_mount_under_read_faults(&parameters, &history.image, &faults);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "根与记录 {root_fault:?}、两块盘最新那一槽系统配置 {system_configuration_fault:?}：乙入判不出拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    assert!(
        image_after == history.image,
        "拒在取号之前：盘上逐字节不变，没写出实例 2 的根"
    );
    assert_eq!(
        verdict_of_the_refusal_on(
            &ImageWithReadFaults {
                image: &history.image,
                faults: &faults,
            },
            &outcome
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在同一份盘面上复算乙入判不出为真：这一拒合法"
    );
}

/// 前提：盘 `device` 两槽各读一次，设备报读错、自证过、别的池的槽数各是几。
fn slot_counts_of(image: &MemoryPool, device: DeviceIdentity) -> (u64, usize, u64) {
    let slots = system_configuration_slots_of_one_device_read_once(
        image,
        device,
        u64::from(FIXED_STRUCTURE_SLOT_SPACING),
        &FILESYSTEM_IDENTIFIER,
    );
    (
        slots.slots_reporting_a_read_error,
        slots.self_verified.len(),
        slots.slots_of_another_pool_whose_checksum_verifies,
    )
}

/// 同一盘一槽读出的字节（盘面手造用）。
fn slot_bytes_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> Vec<u8> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes)
        .expect("系统配置槽读得到")
}

fn generation_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> u64 {
    SystemConfiguration::parse_slot(&slot_bytes_at(image, device, offset))
        .expect("这一槽本池自证过")
        .quantities
        .slot_generation
}

/// 同一盘一份别的池的系统配置：fsid 改一个字节，世代号给 `generation`，整槽校验和由 `to_slot` 封。
fn slot_of_another_pool(image: &MemoryPool, device: DeviceIdentity, generation: u64) -> Vec<u8> {
    let offset = newest_system_configuration_slot(image, device);
    let mut another_pools = SystemConfiguration::parse_slot(&slot_bytes_at(image, device, offset))
        .expect("这一槽本池自证过");
    another_pools.immutable.filesystem_identifier[15] ^= 0xff;
    another_pools.quantities.slot_generation = generation;
    another_pools.to_slot()
}

fn write_slot(image: &mut MemoryPool, device: DeviceIdentity, offset: u64, bytes: &[u8]) {
    image
        .devices
        .get_mut(&device)
        .expect("有这块盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

/// D（txg 6）那次发布的全部记录（两块盘各一份）读回全零：D 的根槽落了、轮换没落，c_见证 是 C 的计数器。
fn faults_on_the_records_of_the_newest_version(history: &MinimalHistory) -> Vec<ReadFault> {
    let counters = &history
        .record_counters_by_txg
        .iter()
        .find(|(record_txg, _)| *record_txg == TXG_OF_THE_NEWEST_VERSION)
        .expect("最小历史里有 D")
        .1;
    let mut faults = Vec::new();
    for counter in counters {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            faults.push(ReadFault {
                device,
                offset_in_bytes: record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES).0,
                length_in_bytes: JOURNAL_RECORD_BYTES,
                kind: ReadFaultKind::ReadsBackAllZeros,
            });
        }
    }
    faults
}

/// 池级 checker 在这张盘上判违例的不变量名。
fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| {
            matches!(
                verdict,
                singlefs_checker::image::InvariantVerdict::Violated(_)
            )
        })
        .map(|(identifier, _)| identifier)
        .collect()
}
