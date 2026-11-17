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

#[test]
fn the_minimal_history_with_roots_reporting_read_errors_and_system_configuration_slots_reading_back_zeros_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::DeviceReportsAReadError,
        ReadFaultKind::ReadsBackAllZeros,
    );
}

#[test]
fn the_minimal_history_with_roots_reporting_read_errors_and_system_configuration_slots_failing_self_verification_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::DeviceReportsAReadError,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
}

#[test]
fn the_minimal_history_with_roots_failing_self_verification_and_system_configuration_slots_reading_back_zeros_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
        ReadFaultKind::ReadsBackAllZeros,
    );
}

#[test]
fn the_minimal_history_with_roots_and_system_configuration_slots_failing_self_verification_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
}

#[test]
fn the_minimal_history_with_roots_and_system_configuration_slots_reading_back_zeros_is_refused() {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
}

#[test]
fn the_minimal_history_with_roots_reading_back_zeros_and_system_configuration_slots_failing_self_verification_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
}

/// 乙的手造盘面：最小历史、不注读故障，`devices` 那几块盘最新那一槽系统配置在盘上翻一个字节（读得出而自证不过）。
fn minimal_history_with_the_newest_system_configuration_slot_flipped_on(
    parameters: &MakeFilesystemParameters,
    devices: &[DeviceIdentity],
) -> MemoryPool {
    let mut image = minimal_history_of_c971(parameters).image;
    for device in devices {
        let offset = newest_system_configuration_slot(&image, *device);
        image.flip_byte(
            *device,
            DeviceOffsetInBytes(offset),
            FLIPPED_BYTE_INSIDE_A_RANGE,
        );
    }
    image
}

/// 手造盘面一（只翻盘 0）：盘 1 两槽都自证过，R2 的另一半不成立，判得出，照常可写挂载、取到实例 2（乙入判不出「不拒」）。
#[test]
fn a_hand_made_image_with_only_device_zero_failing_self_verification_mounts_writable() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_newest_system_configuration_slot_flipped_on(
        &parameters,
        &[DeviceIdentity(0)],
    );
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "一块盘两槽都自证过：判得出，照常可写挂载；得到 {:?}",
        outcome.as_ref().err()
    );
}

/// 手造盘面二（两块盘都翻）：设备没报读错，没有一块盘两槽都自证过，读得出而自证不过的槽当读错计 ⇒ 判不出，拒可写、盘上逐字节不变
/// （乙入判不出「拒」）；挂载透明判官判这一拒合法。
#[test]
fn a_hand_made_image_with_every_device_failing_self_verification_is_refused() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_newest_system_configuration_slot_flipped_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "两块盘最新那一槽都自证不过：乙入判不出拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在同一份盘面上复算乙入判不出为真：这一拒合法"
    );
}

/// 手造盘面三（都不翻），也是阳性对照：不注坏照常可写挂载、取到实例 2，新实例的第一次发布落在 D（txg 6）之后（乙入判不出「不拒」）。
#[test]
fn a_hand_made_image_without_any_flipped_system_configuration_slot_mounts_writable() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image =
        minimal_history_with_the_newest_system_configuration_slot_flipped_on(&parameters, &[]);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    let mounted = outcome.expect("不注坏：照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(TXG_OF_THE_NEWEST_VERSION)
        ),
        "择到 D"
    );
    assert!(
        mounted.output.row_publish.root().checkpoint_txg.0 > TXG_OF_THE_NEWEST_VERSION,
        "新实例的第一次发布落在 D 之后，没有倒挂"
    );
}

/// 撕裂轮换那一态（E205 第二段 `slots_per_region=4 segment=11 kind=publish_end_rotation state=[torn,torn]`）：mkfs 之后第一次可写挂载、
/// 接着新池新建文件的写流，第 11 段两写都撕裂、之前各段整段持久。交回那一态的镜像。
fn torn_rotation_state(parameters: &MakeFilesystemParameters) -> MemoryPool {
    let mut formatted = empty_sparse_devices();
    make_filesystem(parameters, &mut formatted).expect("mkfs");
    let base = memory_pool_of_sparse_devices(&formatted, |device| device);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = formatted
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    assert!(
        current.file_version().is_none(),
        "前提：mkfs 之后第一次可写挂载交回的那一版还没有文件版本"
    );
    let content = file_content_of(1);
    let first_file_version = publish_first_file(
        &mut PoolWriter::new(parameters, devices.as_mut_slice()),
        &mut allocator,
        current.root(),
        FirstFile {
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS,
        },
        output.instance,
        current.record_bytes(),
    )
    .expect("新池新建文件");
    assert_eq!(
        (
            first_file_version.root.instance,
            first_file_version.root.checkpoint_txg
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(TXG_OF_THE_PUBLISH_BEFORE_THE_TORN_ROTATION)
        ),
        "前提：第一个文件版本是实例 1 的 txg 3"
    );
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    let torn_segment = &segments[SEGMENT_OF_THE_TORN_ROTATION_STATE];
    assert_eq!(
        torn_segment
            .iter()
            .map(|index| (writes[*index].kind, writes[*index].device))
            .collect::<Vec<_>>(),
        vec![
            (StepKind::SystemConfigurationSlot, DeviceIdentity(0)),
            (StepKind::SystemConfigurationSlot, DeviceIdentity(1)),
        ],
        "前提：第 11 段是两块盘各一写的系统配置轮换"
    );
    let write_before_the_segment = &writes[torn_segment[0] - 1];
    assert_eq!(
        (
            write_before_the_segment.kind,
            root_identity_of_write(write_before_the_segment)
        ),
        (
            StepKind::RootRecordFua,
            (
                InstanceGeneration(1),
                CheckpointTxg(TXG_OF_THE_PUBLISH_BEFORE_THE_TORN_ROTATION)
            )
        ),
        "前提：那一段是第一个文件版本（实例 1 txg 3）那次发布的根槽写之后的轮换（发布末尾的轮换）"
    );
    let mut image = base;
    for segment in &segments[..SEGMENT_OF_THE_TORN_ROTATION_STATE] {
        image.apply_writes(
            &segment
                .iter()
                .map(|index| writes[*index].clone())
                .collect::<Vec<_>>(),
        );
    }
    for index in torn_segment {
        let write = &writes[*index];
        let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
        let old = PoolReader::read(&image, write.device, write.offset, length).expect("写落在盘内");
        let torn = torn_image_of_in_place_overwrite(&old, &write.contents);
        image
            .devices
            .get_mut(&write.device)
            .expect("有这块盘")
            .write(write.offset, &torn);
    }
    image
}

/// 承诺内单故障（一次断电撕了第一个文件版本发布末尾那一段轮换的两写）：设备没报读错、每块盘都还有一槽自证过、坏的两槽都读得出而自证不过——
/// C596 已定的冲突格（D22（单元原子性怎么合成） 已定项 8 射程与 C596 写明这一形不拒可写）。乙入判不出照条款拒可写、盘上逐字节不变，
/// 只读挂载照常（只剩只读挂载，C596 那一形）；挂载透明判官把这一态当没注读故障的崩溃态判这一拒合法。
#[test]
fn torn_rotation_on_both_devices_after_the_first_file_version_is_refused_in_the_system_configuration_conflict_cell(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_TORN_ROTATION_STATE);
    let image = torn_rotation_state(&parameters);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let slots = system_configuration_slots_of_one_device_read_once(
            &image,
            device,
            u64::from(FIXED_STRUCTURE_SLOT_SPACING),
            &FILESYSTEM_IDENTIFIER,
        );
        assert_eq!(
            (
                slots.slots_reporting_a_read_error,
                slots.self_verified.len()
            ),
            (0, 1),
            "前提（C596 冲突格）：盘 {device:?} 设备没报读错、两槽里恰好一槽自证过"
        );
        let slots_failing_self_verification = (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
            .filter(|slot_index| {
                let bytes = PoolReader::read(
                    &image,
                    device,
                    DeviceOffsetInBytes(slot_index * u64::from(FIXED_STRUCTURE_SLOT_SPACING)),
                    slot_bytes,
                )
                .expect("系统配置槽读得到");
                matches!(
                    SystemConfiguration::parse_slot(&bytes),
                    Err(SystemConfigurationSlotRefusal::NotSelfDescribing)
                )
            })
            .count();
        assert_eq!(
            slots_failing_self_verification, 1,
            "前提（C596 冲突格）：盘 {device:?} 坏的那一槽是撕裂（读得出而自证不过）"
        );
    }
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "两块盘各撕一槽、没有一块盘两槽都自证过：乙入判不出拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    mount_read_only(&image).expect("只读挂载照常");
    let Err(refusal) = &outcome else {
        panic!("上面判过是拒");
    };
    let fields = read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
            &image,
            &FILESYSTEM_IDENTIFIER,
            fields.report.effective_root,
            refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在这一态上复算乙入判不出为真、c_见证 跳过撕裂的槽之后判据不为真（C670 那一条不成立）：这一拒合法"
    );
}

/// 重读一次：两块盘最新那一槽第一遍读自证不过（读得出、翻了一个字节），「重读一次」之前撤掉 ⇒ 第一遍判乙入判不出、重读那一遍判得出，
/// 照常可写挂载、取到实例 2，读阶段在重读那一遍判完（这一拒只在重读之后仍是那一形时才拒，不在第一遍读到就拒）。
#[test]
fn system_configuration_slots_failing_self_verification_only_on_the_first_read_let_the_mount_through_on_the_reread(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let faults = newest_system_configuration_slot_of_each_device_read_bad(
        &image,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &faults, &faults_lifted);
    let mut lift_the_faults = || faults_lifted.set(true);
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut lift_the_faults),
    )
    .expect("重读那一遍读得出：照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
    let ReadStageSettled::OnTheOneReread { first_read, reread } = mounted.output.rereads.read_stage
    else {
        panic!(
            "读阶段要在重读那一遍判完：得到 {:?}",
            mounted.output.rereads.read_stage
        );
    };
    assert_eq!(
        first_read.witness.comparison,
        WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots,
        "第一遍：两块盘最新那一槽都自证不过，乙入判不出"
    );
    assert!(
        !reread.witnesses_a_publish_newer_than_the_selected_version(),
        "重读那一遍判得出、判据为假：{reread:?}"
    );
}

/// 乙的别池手造盘面：最小历史、不注读故障，`devices` 那几块盘两槽里较旧的那一槽换成一份别的池的系统配置
/// （同一个世代号，fsid 改一个字节、整槽校验和重封：magic、整槽校验和、布局位、每区槽数都过，只是 fsid 不是本池的）。
/// 换的是较旧那一槽：每块盘世代号最大的那一槽仍是本池的，挂载透明判官择系统配置（不按 fsid 筛）与可写挂载择到的是同一份。
fn minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
    parameters: &MakeFilesystemParameters,
    devices: &[DeviceIdentity],
) -> MemoryPool {
    let mut image = minimal_history_of_c971(parameters).image;
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in devices {
        let offset = DeviceOffsetInBytes(older_system_configuration_slot(&image, *device));
        let bytes =
            PoolReader::read(&image, *device, offset, slot_bytes).expect("系统配置槽读得到");
        let mut another_pools = SystemConfiguration::parse_slot(&bytes).expect(
            "最小历史里每块盘两槽都自证过（系统配置轮换退回 D 之前的字节，两槽都是本池写的）",
        );
        another_pools.immutable.filesystem_identifier[15] ^= 0xff;
        image
            .devices
            .get_mut(device)
            .expect("有这块盘")
            .write(offset, &another_pools.to_slot());
    }
    image
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

/// 别池盘面一（两块盘较旧那一槽都换成别的池的）：设备没报读错、没有撕裂与全零的槽，每块盘一槽本池自证过、一槽整槽校验和过而 fsid 不是本池的；
/// 「自证过」要 fsid 与本池相同（登记「两种读法共用」那一行，C707 定按自证不过处置），于是没有一块盘两槽都自证过、有槽读得出而自证不过 ⇒
/// 乙入判不出，两遍读都是这一形，拒可写、盘上逐字节不变；挂载透明判官在同一份盘面上复算判这一拒合法。
/// 只换一块盘时另一块盘两槽都自证过、判得出（下一条阳性对照），所以两块盘都换：这是「没有一块盘两槽都自证过」而坏槽只有别的池的槽的最小一形。
#[test]
fn hand_made_image_with_the_older_slot_of_every_device_from_another_pool_is_refused() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            slot_counts_of(&image, device),
            (0, 1, 1),
            "前提：盘 {device:?} 设备没报读错、一槽本池自证过、一槽是别的池的（没有撕裂与全零的槽）"
        );
    }
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "两块盘各有一槽是别的池的、没有一块盘两槽都自证过：乙入判不出拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在同一份盘面上把别的池的槽也算读得出而自证不过、复算乙入判不出为真：这一拒合法"
    );
}

/// 别池盘面二（阳性对照：盘 0 那一槽换回本池自证过的，只剩盘 1 较旧那一槽是别的池的）：盘 0 两槽都自证过，R2 的另一半不成立，判得出，
/// 照常可写挂载、取到实例 2，读阶段第一遍就判完（乙入判不出「不拒」）。
#[test]
fn hand_made_image_with_the_older_slot_of_only_device_one_from_another_pool_mounts_writable() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    assert_eq!(
        (
            slot_counts_of(&image, DeviceIdentity(0)),
            slot_counts_of(&image, DeviceIdentity(1))
        ),
        ((0, 2, 0), (0, 1, 1)),
        "前提：盘 0 两槽都本池自证过，盘 1 一槽本池自证过、一槽是别的池的"
    );
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    let mounted = outcome.expect("盘 0 两槽都自证过：判得出，照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
    assert!(
        matches!(
            mounted.output.rereads.read_stage,
            ReadStageSettled::OnTheFirstRead { .. }
        ),
        "判得出、判据为假：读阶段第一遍就判完，不重读；得到 {:?}",
        mounted.output.rereads.read_stage
    );
}

/// 重读粒度：读阶段那一次重读是整个读阶段重做一遍、只按重读那一遍的读数判。两块盘世代号最大的那一槽第一遍读自证不过（翻一个字节），
/// 较旧那一槽第一遍读好；「重读一次」之前撤掉第一遍的故障、改让较旧那一槽读得出而自证不过 ⇒ 第一遍乙入判不出为真，重读那一遍每块盘
/// 仍只有一槽自证过、仍是乙入判不出 ⇒ 照重读那一遍判、拒可写，盘上逐字节不变（只重读第一遍坏的那几槽时那几槽重读读好、会放行）；
/// 挂载透明判官在重读那一遍的盘面上判这一拒合法。
#[test]
fn system_configuration_slots_good_on_the_first_read_and_failing_self_verification_on_the_reread_are_judged_by_the_reread_and_refused(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let faults_on_the_first_read = newest_system_configuration_slot_of_each_device_read_bad(
        &image,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
    let faults_from_the_one_reread_on: Vec<ReadFault> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| ReadFault {
            device,
            offset_in_bytes: older_system_configuration_slot(&image, device),
            length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
            kind: ReadFaultKind::ReadsBackBytesFailingSelfVerification,
        })
        .collect();
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults_replaced_at_the_one_reread(
        &image,
        &faults_on_the_first_read,
        &faults_from_the_one_reread_on,
        &faults_lifted,
    );
    let hook_calls = Cell::new(0u32);
    let mut replace_the_faults = || {
        hook_calls.set(hook_calls.get() + 1);
        faults_lifted.set(true);
    };
    let outcome = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut replace_the_faults),
    );
    assert_eq!(
        hook_calls.get(),
        1,
        "读阶段第一遍判真、重读了一次（重读之前调了一次钩子）"
    );
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "第一遍好的槽在重读那一遍读坏：照重读那一遍判、仍是乙入判不出，拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    let image_after = memory_pool_of_sparse_devices(&devices, |device| &device.inner);
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    assert_eq!(
        verdict_of_the_refusal_on(
            &ImageWithReadFaults {
                image: &image,
                faults: &faults_from_the_one_reread_on,
            },
            &outcome
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在重读那一遍的盘面上复算乙入判不出为真：这一拒合法"
    );
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

/// 同一盘一份别的池的系统配置，槽内 `offset_in_the_slot` 处那一个字节改成 `value`（产品与 checker 各自判不判这个字段不同），整槽校验和重封。
fn slot_of_another_pool_with_one_byte_changed_and_resealed(
    image: &MemoryPool,
    device: DeviceIdentity,
    generation: u64,
    offset_in_the_slot: usize,
    value: u8,
) -> Vec<u8> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut slot = slot_of_another_pool(image, device, generation);
    slot[offset_in_the_slot] = value;
    let digest =
        wide_checksum_with_field_zeroed(&slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    slot
}

/// `devices` 那几块盘较旧那一槽的位置放一份世代号比本池最新那一槽大 100 的别的池的槽（不筛 fsid 的择法在这块盘上择到它）。
fn minimal_history_with_a_newest_generation_of_another_pool_on(
    parameters: &MakeFilesystemParameters,
    devices: &[DeviceIdentity],
) -> MemoryPool {
    let mut image = minimal_history_of_c971(parameters).image;
    for device in devices {
        let newest = newest_system_configuration_slot(&image, *device);
        let generation = generation_at(&image, *device, newest) + 100;
        let older = older_system_configuration_slot(&image, *device);
        let bytes = slot_of_another_pool(&image, *device, generation);
        write_slot(&mut image, *device, older, &bytes);
    }
    image
}

/// 最小历史上可写挂载读阶段落到的那一版：实例 1 的最新那一版 D。
fn newest_version_of_the_minimal_history() -> (InstanceGeneration, CheckpointTxg) {
    (
        InstanceGeneration(1),
        CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
    )
}

/// 所选那一版是 D 的读数（判官自己在镜像上复算判据，不看这里填的见证值）。
fn newest_version_against_an_undecidable_witness() -> SelectedVersionAgainstTheWitness {
    let (instance, checkpoint_txg) = newest_version_of_the_minimal_history();
    SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance,
            checkpoint_txg,
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 1,
            comparison: WitnessedCounterComparison::Undecidable,
        },
    }
}

/// 用例自己拼的判据 N-配置 之拒。
fn witness_refusal_claimed_on_the_newest_version() -> MountError {
    let reading = newest_version_against_an_undecidable_witness();
    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
            first_read: reading,
            reread: reading,
        },
    ))
}

/// 用例自己拼的「读阶段判完之后系统配置又读不出」之拒（判官只认拒因的外层成员、自己复算判据）。
fn refusal_after_the_read_stage_claimed_on_the_newest_version() -> MountError {
    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
        SystemConfigurationUnreadableAfterTheReadStage {
            read_stage: newest_version_against_an_undecidable_witness(),
            unreadable: SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device: DeviceIdentity(1),
                not_rewritten_because: DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet(
                    RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice,
                ),
            },
        },
    ))
}

/// 判官判 `claimed` 这一拒，所选那一版取 D（可写挂载在本池的槽上落到的那一版），U6 另两条都满足。
fn verdict_of_a_claimed_refusal_on(
    image: &MemoryPool,
    claimed: &MountError,
) -> WritableMountRefusalVerdict {
    judge_writable_mount_refusal(
        image,
        &FILESYSTEM_IDENTIFIER,
        Some(newest_version_of_the_minimal_history()),
        claimed,
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
    )
}

fn says_the_criterion_is_false(verdict: &WritableMountRefusalVerdict) -> bool {
    matches!(
        verdict,
        WritableMountRefusalVerdict::Illegal { conditions_not_met, .. }
            if conditions_not_met.contains(
                &RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            )
    )
}

/// 盘 0 两槽都是别的池的，盘 1 较旧位置是世代号更大的别的池的槽（每块盘世代号最大的一槽都是同一个别的池的）。
fn minimal_history_with_both_slots_of_device_zero_from_another_pool(
    parameters: &MakeFilesystemParameters,
) -> MemoryPool {
    let mut image = minimal_history_with_a_newest_generation_of_another_pool_on(
        parameters,
        &[DeviceIdentity(1)],
    );
    let device_zero = DeviceIdentity(0);
    let newest = newest_system_configuration_slot(&image, device_zero);
    let older = older_system_configuration_slot(&image, device_zero);
    let generation = generation_at(&image, device_zero, newest) + 200;
    let older_slot_of_another_pool = slot_of_another_pool(&image, device_zero, generation);
    let newest_slot_of_another_pool = slot_of_another_pool(&image, device_zero, generation + 1);
    write_slot(&mut image, device_zero, older, &older_slot_of_another_pool);
    write_slot(
        &mut image,
        device_zero,
        newest,
        &newest_slot_of_another_pool,
    );
    assert_eq!(
        slot_counts_of(&image, device_zero),
        (0, 0, 2),
        "前提：盘 0 两槽都是别的池的"
    );
    assert_eq!(
        slot_counts_of(&image, DeviceIdentity(1)),
        (0, 1, 1),
        "前提：盘 1 一槽本池、一槽别池"
    );
    image
}

/// 判官按本池 fsid 择系统配置，合法的拒判合法：盘面见 [`minimal_history_with_both_slots_of_device_zero_from_another_pool`]。可写挂载按本池 fsid 判乙入判不出、拒；
/// 判官同样只从本池的槽里择（盘 1 那一份），数出没有一块盘两槽都自证过、有别的池的槽，判据复算为真。
#[test]
fn the_judge_chooses_the_slots_of_this_pool_and_calls_the_refusal_legal_when_both_slots_of_device_zero_are_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_both_slots_of_device_zero_from_another_pool(&parameters);
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "可写挂载乙入判不出拒"
    );
    assert!(image_after == image, "拒在取号之前");
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "判官只从本池的槽里择，这一拒的判据复算为真：合法"
    );
}

/// 判官按本池 fsid 择系统配置，不该有的拒判不合法：一块盘世代号最大的那一槽是别的池的（单故障），可写挂载照常做成；同一份盘面上喂一个声称的判据 N-配置 之拒，
/// 判官只从本池的槽里择、各盘 fsid 对得上，见证值比出来不超过所选那一版，判据复算为假。别的池的槽世代号小的对照盘面同样判不合法。
#[test]
fn the_judge_chooses_the_slots_of_this_pool_and_calls_a_claimed_refusal_illegal_when_the_newest_slot_of_one_device_is_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "单故障，可写挂载做成"
    );
    let claimed = witness_refusal_claimed_on_the_newest_version();
    let verdict = verdict_of_a_claimed_refusal_on(&image, &claimed);
    assert!(
        says_the_criterion_is_false(&verdict),
        "这一拒不该有，判官判判据复算为假；得到 {verdict:?}"
    );
    let control = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    let verdict_on_the_control = verdict_of_a_claimed_refusal_on(&control, &claimed);
    assert!(
        says_the_criterion_is_false(&verdict_on_the_control),
        "对照盘面上判官同样判不合法；得到 {verdict_on_the_control:?}"
    );
}

/// 单子上别的理由也按本池 fsid 复算：同一份一块盘世代号最大的那一槽是别的池的盘面，每块盘都有本池自证过的槽、设备没报读错，
/// 「读阶段判完之后系统配置又读不出」的判据复算为假（择系统配置不筛 fsid 时各盘对不上、择不到、按真）。
#[test]
fn the_judge_chooses_the_slots_of_this_pool_and_calls_a_claimed_refusal_after_the_read_stage_illegal_when_the_newest_slot_of_one_device_is_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    let verdict = verdict_of_a_claimed_refusal_on(
        &image,
        &refusal_after_the_read_stage_claimed_on_the_newest_version(),
    );
    assert!(
        says_the_criterion_is_false(&verdict),
        "判官判判据复算为假；得到 {verdict:?}"
    );
}

/// 每块盘世代号最大的那一槽是别的池的：可写挂载乙入判不出拒；只读挂载不筛 fsid，择到别的池那一份、一条根都读不到；
/// 只剩离线出路（D22（单元原子性怎么合成） 已定项 8），它照断言的那一版重写别的池的那两槽，之后可写挂载做成。
#[test]
fn newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    assert_eq!(
        slot_counts_of(&image, DeviceIdentity(0)),
        (0, 1, 1),
        "前提：盘 0 一槽本池、一槽别池"
    );
    assert_eq!(
        slot_counts_of(&image, DeviceIdentity(1)),
        (0, 1, 1),
        "前提：盘 1 一槽本池、一槽别池"
    );
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "可写挂载乙入判不出拒"
    );
    assert!(image_after == image, "拒在取号之前");
    assert_eq!(
        mount_read_only(&image).err(),
        Some(MountReadOnlyFailure::Recovery(RecoveryFailure::NoValidRoot)),
        "只读挂载择到别的池那一份、一条根都读不到"
    );
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
    let rewritten = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
        },
    )
    .expect("离线出路照断言的那一版重写别的池的那两槽");
    assert_eq!(rewritten.slots_rewritten.len(), 2, "每块盘重写一槽");
    let repaired = memory_pool_of_sparse_devices(&devices, |device| &device.inner);
    let (after_repair, _) = writable_mount_under_read_faults(&parameters, &repaired, &[]);
    assert_eq!(
        after_repair
            .as_ref()
            .map(|mounted| mounted.output.instance)
            .ok(),
        Some(InstanceGeneration(2)),
        "离线出路之后可写挂载做成"
    );
}

/// 上一条的对照：每块盘别的池的槽世代号小，只读挂载择到本池那一份，照常做成。
#[test]
fn older_generation_of_another_pool_on_every_device_still_mounts_read_only() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    assert!(
        mount_read_only(&image).is_ok(),
        "择到本池那一份，只读挂载做成"
    );
}

/// 钉现状，条款不罩这一形（归 C6（块层语义假设写错） 那一类丢写，D22（单元原子性怎么合成） 已定项 8 射程写明不罩）：C971 的根与记录读坏（C、D），
/// 盘 1 最新那一槽读回全 0，盘 0 没收到那次轮换（两槽都自证过、见证值都不超过 B）。「没有一块盘两槽都自证过」不成立，乙入判不出不判，
/// c_见证 落回 B，可写挂载做成，新实例第一次发布落到被藏的 D 之下。
#[test]
fn a_device_holding_two_stale_self_verified_slots_lets_the_inverted_first_txg_through() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let mut image = history.image.clone();
    let device_zero = DeviceIdentity(0);
    let newest = newest_system_configuration_slot(&image, device_zero);
    let older = older_system_configuration_slot(&image, device_zero);
    let mut stale = SystemConfiguration::parse_slot(&slot_bytes_at(&image, device_zero, older))
        .expect("较旧那一槽本池自证过");
    stale.quantities.slot_generation -= 1;
    write_slot(&mut image, device_zero, newest, &stale.to_slot());
    assert_eq!(
        slot_counts_of(&image, device_zero),
        (0, 2, 0),
        "前提：盘 0 两槽都自证过"
    );
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| {
        fault.device != device_zero
            || fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING)
    });
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &faults);
    let mounted = outcome.expect("可写挂载做成");
    assert!(
        mounted.output.row_publish.root().checkpoint_txg.0 <= TXG_OF_THE_NEWEST_VERSION,
        "新实例第一次发布落到被藏的 D 之下"
    );
}

/// 上一条的对照：同一组读坏，盘 0 没落后（最新那一槽见证到 C）。c_见证 比所选那一版 B 新，判据 N-配置 比出来为真，拒。
#[test]
fn the_same_faults_are_refused_when_device_zero_is_not_behind() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let device_zero = DeviceIdentity(0);
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| {
        fault.device != device_zero
            || fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING)
    });
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &history.image, &faults);
    assert!(
        matches!(
            outcome,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(_))
        ),
        "盘 0 没落后时照判据 N-配置 拒；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
}

/// 钉现状，条款不罩这一形（这一槽要有别的版本布局的池，第一版没有；「布局位不认识的槽不算坏」与「别的池的槽算坏」两句的优先次序条款没定）：
/// C971 的根与记录读坏（C、D），每块盘最新那一槽位置换成别的池的、布局位不认识的槽（整槽校验和过）。实现照「布局位不认识不算坏」不并进乙入判不出，
/// 可写挂载做成，新实例第一次发布落到被藏的 D 之下。
#[test]
fn a_slot_of_another_pool_with_unknown_incompat_bits_lets_the_inverted_first_txg_through() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let mut image = history.image.clone();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let newest = newest_system_configuration_slot(&image, device);
        let generation = generation_at(&image, device, newest);
        let mut slot = slot_of_another_pool(&image, device, generation);
        slot[INCOMPAT_BITMAP_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0x80;
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            slot_bytes,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        write_slot(&mut image, device, newest, &slot);
        assert_eq!(
            slot_counts_of(&image, device),
            (0, 1, 0),
            "前提：这一槽不进任何一个计数"
        );
    }
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING));
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &faults);
    let mounted = outcome.expect("可写挂载做成");
    assert!(
        mounted.output.row_publish.root().checkpoint_txg.0 <= TXG_OF_THE_NEWEST_VERSION,
        "新实例第一次发布落到被藏的 D 之下"
    );
}

/// 一段字节只在第 `earliest_bad_read_number` 到 `latest_bad_read_number` 次读（从 1 数）读回全零的盘：外面套在带常驻读故障的盘上。写到这一段就撤。
struct DeviceWithOneRangeBadOnChosenReads {
    inner: DeviceWithReadFaults,
    range: Option<(u64, u64)>,
    earliest_bad_read_number: u64,
    latest_bad_read_number: u64,
    reads_of_the_range: Rc<Cell<u64>>,
}

impl DeviceWithOneRangeBadOnChosenReads {
    fn lift_when_written_over(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        if let Some((start, length)) = self.range {
            if offset_in_bytes < start + length && start < offset_in_bytes + length_in_bytes {
                self.range = None;
            }
        }
    }
}

impl BlockDevice for DeviceWithOneRangeBadOnChosenReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let buffer_length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if let Some((start, length)) = self.range {
            if offset.0 < start + length && start < offset.0 + buffer_length {
                let read_number = self.reads_of_the_range.get() + 1;
                self.reads_of_the_range.set(read_number);
                if self.earliest_bad_read_number <= read_number
                    && read_number <= self.latest_bad_read_number
                {
                    let from = usize::try_from(start.saturating_sub(offset.0)).expect("缓冲内偏移");
                    let to =
                        usize::try_from((start + length).min(offset.0 + buffer_length) - offset.0)
                            .expect("缓冲内偏移");
                    buffer[from..to].fill(0);
                }
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.lift_when_written_over(
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
        self.lift_when_written_over(offset.0, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.lift_when_written_over(offset.0, length);
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

/// D 的根自证过、D 的记录两份都读回全零（不带瞬时读错）：新实例第一次发布越过 D（txg 7），环里 D 这条根把 max 抬上去。
#[test]
fn root_read_with_its_records_unreadable_takes_the_first_txg_past_it() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = faults_on_the_records_of_the_newest_version(&history);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &history.image, &faults);
    let mounted = outcome.expect("可写挂载做成");
    assert_eq!(
        (
            mounted.output.effective_root.checkpoint_txg.0,
            mounted.output.row_publish.root().checkpoint_txg.0
        ),
        (TXG_OF_THE_NEWEST_VERSION, TXG_OF_THE_NEWEST_VERSION + 1),
        "落到 D，写行那次发布越过 D"
    );
}

/// D23（journal 的角色与格式） 已定项 14 第 3 条：盘面同上一条（D 的记录两份都读回全零，崩在 D 的根槽之后、轮换之前），D 的根槽在这次挂载的第几次读读回全零都扫一遍
/// （只这一次 / 从这一次起一直；读阶段读到 D、取第一个 txg 那一遍直接读盘读坏的那几格都在里面，第几次读落在那一遍里不写死）。
/// 每一格可写挂载做成、写行 txg 大于所选那一版、池级 checker 无违例、第二次可写挂载做成；每格打一行 `FIRST_TXG_CELL`。
#[test]
fn root_read_bad_only_after_the_read_stage_still_numbers_the_new_instance_above_the_selected_version(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = faults_on_the_records_of_the_newest_version(&history);
    let target = target_for_publish(
        CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0;
    let mut cells_selecting_the_newest_version = 0u64;
    for only_this_read in [true, false] {
        for earliest_bad_read_number in 1..=16u64 {
            let latest_bad_read_number = if only_this_read {
                earliest_bad_read_number
            } else {
                u64::MAX
            };
            let faults_lifted = Rc::new(Cell::new(false));
            let reads_of_the_range = Rc::new(Cell::new(0u64));
            let mut devices: Vec<(DeviceIdentity, DeviceWithOneRangeBadOnChosenReads)> =
                devices_with_read_faults(&history.image, &faults, &faults_lifted)
                    .into_iter()
                    .map(|(identity, inner)| {
                        (
                            identity,
                            DeviceWithOneRangeBadOnChosenReads {
                                inner,
                                range: (identity == root_device)
                                    .then_some((root_offset, ROOT_SLOT_BYTES)),
                                earliest_bad_read_number,
                                latest_bad_read_number,
                                reads_of_the_range: Rc::clone(&reads_of_the_range),
                            },
                        )
                    })
                    .collect();
            let mounted = mount_writable(&parameters, &mut devices).unwrap_or_else(|refusal| {
                panic!("只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：可写挂载做成；得到 {refusal:?}")
            });
            let image_after = memory_pool_of_sparse_devices(&devices, |device| &device.inner.inner);
            let effective = mounted.output.effective_root.checkpoint_txg.0;
            let row = mounted.output.row_publish.root().checkpoint_txg.0;
            let violated = violated_invariants(&image_after);
            let (second_mount, _) =
                writable_mount_under_read_faults(&parameters, &image_after, &[]);
            println!(
                "FIRST_TXG_CELL only_this_read={only_this_read} earliest_bad_read_number={earliest_bad_read_number} reads={} effective={effective} row={row} violated={violated:?} second_mount_ok={}",
                reads_of_the_range.get(),
                second_mount.is_ok()
            );
            assert!(
                row > effective,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：写行 txg {row} 大于所选那一版 {effective}"
            );
            assert_eq!(
                violated,
                Vec::<&'static str>::new(),
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：池级 checker 无违例"
            );
            assert!(
                second_mount.is_ok(),
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：第二次可写挂载做成"
            );
            if effective == TXG_OF_THE_NEWEST_VERSION {
                cells_selecting_the_newest_version += 1;
            }
        }
    }
    assert!(
        cells_selecting_the_newest_version > 0,
        "前提：扫到了读阶段读到 D 的那几格"
    );
}

/// 一块盘最新一槽是别的池的盘面上，判官的所选那一版取按本池 fsid 的恢复落到的那一版：盘 1 较旧那一槽换成世代号大 100 的别的池的槽（盘 0 两槽本池自证过，单故障），
/// C、D 的根与记录读回全零。产品按本池 fsid 走：盘 0 两槽都自证过，乙入判不出不判，c_见证 是 C 的计数器、所选那一版是 B，比法 `AgainstTheSelectedVersionsLastRecord` 为真，拒。
/// 不筛 fsid 的恢复报各盘 fsid 对不上、落不到任何一版；按本池 fsid 的恢复落到 B，与被拒的那次挂载无关地取得，判官判合法。
#[test]
fn the_judge_fed_the_version_recovery_of_the_pool_lands_on_calls_a_legal_witness_refusal_legal_when_one_slot_is_from_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let mut image = history.image.clone();
    let device_one = DeviceIdentity(1);
    let newest = newest_system_configuration_slot(&image, device_one);
    let older = older_system_configuration_slot(&image, device_one);
    let generation = generation_at(&image, device_one, newest) + 100;
    let bytes = slot_of_another_pool(&image, device_one, generation);
    write_slot(&mut image, device_one, older, &bytes);
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING));
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &faults);
    assert!(image_after == image, "拒在取号之前");
    let Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) = &outcome
    else {
        panic!("可写挂载按判据 N-配置 拒");
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        reread,
        ..
    } = still_unreadable.as_ref()
    else {
        panic!("拒因是判据 N-配置");
    };
    assert!(
        matches!(
            reread.witness.comparison,
            WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord { .. }
        ),
        "盘 0 两槽都自证过，比法是计数器比，不是判不出"
    );
    let crash_image = ImageWithReadFaults {
        image: &image,
        faults: &faults,
    };
    assert_eq!(
        read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
            .report
            .effective_root,
        None,
        "不筛 fsid 的恢复落不到任何一版"
    );
    assert_eq!(
        selected_version_of_the_pool_on_the_crash_image(&crash_image, &FILESYSTEM_IDENTIFIER),
        Some((
            reread.selected_version.instance,
            reread.selected_version.checkpoint_txg
        )),
        "按本池 fsid 的恢复落到被拒的那次挂载落到的那一版"
    );
    assert_eq!(
        verdict_of_the_refusal_on(&crash_image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "判官的所选那一版取自按本池 fsid 的恢复，判合法"
    );
}

/// 只有一块盘带世代号大的别的池的槽。不带 fsid 的只读挂载与恢复报各盘 fsid 对不上；带本池 fsid 的只读挂载与恢复读到本池最新那一版；盘 0 两槽本池自证过时可写挂载做成。
/// 盘 0 再撕一槽时可写挂载乙入判不出拒，不带 fsid 的只读挂载同样对不上，带本池 fsid 的只读挂载照样读到本池。
#[test]
fn newest_generation_of_another_pool_on_only_one_device_reads_this_pool_read_only_given_the_pool_fsid(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(1)],
    );
    assert_eq!(
        mount_read_only(&image).err(),
        Some(MountReadOnlyFailure::Recovery(
            RecoveryFailure::SystemConfigurationsDisagree
        )),
        "不带 fsid 的只读挂载，两块盘择到的 fsid 对不上"
    );
    assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(&image);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "可写挂载筛 fsid，做成"
    );
    let mut with_a_torn_slot = image.clone();
    let device_zero = DeviceIdentity(0);
    let older = older_system_configuration_slot(&with_a_torn_slot, device_zero);
    let mut torn = slot_bytes_at(&with_a_torn_slot, device_zero, older);
    torn[usize::try_from(FLIPPED_BYTE_INSIDE_A_RANGE).expect("100")] ^= 0xff;
    write_slot(&mut with_a_torn_slot, device_zero, older, &torn);
    let (refused, image_after) =
        writable_mount_under_read_faults(&parameters, &with_a_torn_slot, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&refused),
        "盘 0 撕一槽、盘 1 一槽别池：乙入判不出拒"
    );
    assert!(image_after == with_a_torn_slot, "拒在取号之前");
    assert_eq!(
        mount_read_only(&with_a_torn_slot).err(),
        Some(MountReadOnlyFailure::Recovery(
            RecoveryFailure::SystemConfigurationsDisagree
        )),
        "不带 fsid 的只读挂载同样对不上"
    );
    assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(
        &with_a_torn_slot,
    );
}

/// 带本池 fsid 的只读挂载落到 D，带本池 fsid 的恢复读出 D 的文件内容（最小历史第 4 版内容）。
fn assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(
    image: &MemoryPool,
) {
    let (newest_instance, newest_txg) = newest_version_of_the_minimal_history();
    let read_only = mount_read_only_of_the_pool(image, &FILESYSTEM_IDENTIFIER)
        .unwrap_or_else(|failure| panic!("带本池 fsid 的只读挂载做成；得到 {failure:?}"));
    assert_eq!(
        (
            read_only.effective_root.instance,
            read_only.effective_root.checkpoint_txg
        ),
        (newest_instance, newest_txg),
        "带本池 fsid 的只读挂载落到 D"
    );
    let report = recover_of_the_pool(image, JournalPolicy::Consult, &FILESYSTEM_IDENTIFIER);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (newest_instance, newest_txg),
            content: file_content_of(4),
        },
        "带本池 fsid 的恢复读出 D 的文件内容"
    );
}

/// 每块盘世代号最大的一槽都是别的池的：不带 fsid 的只读挂载报 `NoValidRoot`（[`newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only`]），
/// 带本池 fsid 的只读挂载与恢复读到本池。
#[test]
fn newest_generation_of_another_pool_on_every_device_reads_this_pool_read_only_given_the_pool_fsid()
{
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_a_newest_generation_of_another_pool_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    assert_the_read_only_mount_and_the_recovery_given_the_pool_fsid_read_the_newest_version(&image);
}

/// 交进来的 fsid 盘上一槽都没有：带 fsid 的只读挂载与恢复报 `NoValidSystemConfiguration`，不读盘上那个调用方没点名的池。
#[test]
fn the_read_only_mount_given_a_filesystem_identifier_no_slot_carries_does_not_read_the_pool_on_the_disks(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let mut filesystem_identifier_of_no_slot = FILESYSTEM_IDENTIFIER;
    filesystem_identifier_of_no_slot[15] ^= 0xff;
    assert!(
        matches!(
            mount_read_only_of_the_pool(&image, &filesystem_identifier_of_no_slot),
            Err(MountReadOnlyFailure::Recovery(
                RecoveryFailure::NoValidSystemConfiguration { .. }
            ))
        ),
        "只读挂载拒，不读别的池"
    );
    let report = recover_of_the_pool(
        &image,
        JournalPolicy::Consult,
        &filesystem_identifier_of_no_slot,
    );
    assert!(
        matches!(
            report.outcome,
            RecoveryOutcome::Failed {
                root: None,
                failure: RecoveryFailure::NoValidSystemConfiguration { .. }
            }
        ),
        "恢复报择不到系统配置；得到 {:?}",
        report.outcome
    );
}

/// 判官与产品读别的池的槽同一个口径：两块盘较旧那一槽换成别的池的、区域数字段改成 2、整槽校验和重封的槽。
/// 产品 `SystemConfiguration::parse_slot` 不看区域数，算别的池的槽并进乙入判不出，拒；判官照产品读这一槽的那几关数它（不走 checker 的几何判），判据复算为真，判合法。
#[test]
fn slot_of_another_pool_with_a_region_count_the_checker_refuses_is_counted_by_the_judge_like_the_product(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let mut image = minimal_history_of_c971(&parameters).image;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let older = older_system_configuration_slot(&image, device);
        let generation = generation_at(&image, device, older);
        let slot = slot_of_another_pool_with_one_byte_changed_and_resealed(
            &image,
            device,
            generation,
            REGION_COUNT_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
            2,
        );
        write_slot(&mut image, device, older, &slot);
        assert_eq!(
            slot_counts_of(&image, device),
            (0, 1, 1),
            "前提：产品把它算别的池的槽"
        );
    }
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "产品乙入判不出拒"
    );
    assert!(image_after == image, "拒在取号之前");
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "判官与产品一样数这一槽，判合法"
    );
}

/// 判官与产品读别的池的槽同一个口径的另一面：别的池的槽布局位不认识、每区槽数越界，产品不把它算别的池的槽（两块盘各一份），判官也不算；
/// 乙入判不出不成立，声称的判据 N-配置 之拒判判据复算为假。
#[test]
fn slot_of_another_pool_the_product_does_not_count_is_not_counted_by_the_judge() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    for (offset_in_the_slot, value_not_accepted) in [
        (
            INCOMPAT_BITMAP_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
            0x80,
        ),
        (
            ROOT_RING_SLOTS_PER_REGION_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
            17,
        ),
    ] {
        let mut image = minimal_history_of_c971(&parameters).image;
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let older = older_system_configuration_slot(&image, device);
            let generation = generation_at(&image, device, older);
            let slot = slot_of_another_pool_with_one_byte_changed_and_resealed(
                &image,
                device,
                generation,
                offset_in_the_slot,
                value_not_accepted,
            );
            write_slot(&mut image, device, older, &slot);
            assert_eq!(
                slot_counts_of(&image, device),
                (0, 1, 0),
                "前提：偏移 {offset_in_the_slot} 改了之后产品不把它算别的池的槽"
            );
        }
        let verdict = verdict_of_a_claimed_refusal_on(
            &image,
            &witness_refusal_claimed_on_the_newest_version(),
        );
        assert!(
            says_the_criterion_is_false(&verdict),
            "偏移 {offset_in_the_slot}：判官不数这一槽，判据复算为假；得到 {verdict:?}"
        );
    }
}

/// 崩溃态判官带本池 fsid，别的池的槽世代号再大也择不到：盘 0 两槽都是别的池的、盘 1 较旧位置是世代号更大的别的池的槽，可写挂载按本池 fsid 判乙入判不出、拒，判官只从本池的槽里择，判这一拒合法。
#[test]
fn the_crash_state_judge_given_the_pool_fsid_calls_a_legal_refusal_legal_when_every_device_chooses_a_slot_of_another_pool(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_both_slots_of_device_zero_from_another_pool(&parameters);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "可写挂载乙入判不出拒"
    );
    let Err(refusal) = &outcome else {
        panic!("上面判过是拒");
    };
    let selected_version =
        selected_version_of_the_pool_on_the_crash_image(&image, &FILESYSTEM_IDENTIFIER);
    assert_eq!(
        selected_version,
        Some(newest_version_of_the_minimal_history()),
        "按本池 fsid 的恢复落到 D"
    );
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
            &image,
            &FILESYSTEM_IDENTIFIER,
            selected_version,
            refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "带本池 fsid：只从本池的槽里择，判合法"
    );
}

/// mkfs 之后第一次可写挂载的写流里第 `segment` 段（取号写的一轮，两块盘各一写）两写都撕裂、之前各段整段持久。
fn torn_acquisition_round_on_a_formatted_pool(
    parameters: &MakeFilesystemParameters,
    segment: usize,
) -> MemoryPool {
    let mut formatted = empty_sparse_devices();
    make_filesystem(parameters, &mut formatted).expect("mkfs");
    let base = memory_pool_of_sparse_devices(&formatted, |device| device);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = formatted
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    mount_writable(parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    let torn_segment = &segments[segment];
    assert_eq!(
        torn_segment
            .iter()
            .map(|index| (writes[*index].kind, writes[*index].device))
            .collect::<Vec<_>>(),
        vec![
            (StepKind::SystemConfigurationSlot, DeviceIdentity(0)),
            (StepKind::SystemConfigurationSlot, DeviceIdentity(1)),
        ],
        "前提：第 {segment} 段是取号写的一轮，两块盘各一写"
    );
    let mut image = base;
    for earlier in &segments[..segment] {
        image.apply_writes(
            &earlier
                .iter()
                .map(|index| writes[*index].clone())
                .collect::<Vec<_>>(),
        );
    }
    for index in torn_segment {
        let write = &writes[*index];
        let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
        let old = PoolReader::read(&image, write.device, write.offset, length).expect("写落在盘内");
        let torn = torn_image_of_in_place_overwrite(&old, &write.contents);
        image
            .devices
            .get_mut(&write.device)
            .expect("有这块盘")
            .write(write.offset, &torn);
    }
    image
}

/// 一次断电撕了 mkfs 之后第一次可写挂载取号写一轮的两写（单故障、设备没报读错）：可写挂载乙入判不出拒，只读挂载答树表 0 条，
/// 离线出路断言第 0 代（mkfs 不写记录，tail 取 mkfs 写的 0）重写撕裂的两槽；之后可写挂载做成、取号照常加一，发得出第一个文件版本，池级 checker 无违例，只读恢复读得出它。
#[test]
fn torn_acquisition_round_on_a_formatted_pool_is_repaired_offline_and_publishes_the_first_file_version(
) {
    for slots_per_region in [4u64, 8, 16] {
        for segment in [0usize, 1] {
            let parameters = pool_parameters(slots_per_region);
            let image = torn_acquisition_round_on_a_formatted_pool(&parameters, segment);
            let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
            assert!(
                refused_as_failing_self_verification_on_both_reads(&outcome),
                "S = {slots_per_region} 段 {segment}：乙入判不出拒"
            );
            assert!(image_after == image, "拒在取号之前");
            assert!(
                matches!(
                    mount_read_only(&image),
                    Err(MountReadOnlyFailure::Open(
                        OpenPoolForReadFailure::TreeTableHasNoEntries
                    ))
                ),
                "S = {slots_per_region} 段 {segment}：只读挂载答树表 0 条"
            );
            let faults_lifted = Rc::new(Cell::new(false));
            let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
            let rewritten = rewrite_system_configuration_slots_for_the_asserted_version(
                &parameters,
                &mut devices,
                AssertedVersion {
                    instance: InstanceGeneration(0),
                    checkpoint_txg: CheckpointTxg(0),
                },
            )
            .unwrap_or_else(|refusal| {
                panic!("S = {slots_per_region} 段 {segment}：离线出路断言第 0 代做成；得到 {refusal:?}")
            });
            assert_eq!(
                (
                    rewritten.journal_tail_written,
                    rewritten.slots_rewritten.len()
                ),
                (0, 2),
                "S = {slots_per_region} 段 {segment}：tail 取 mkfs 写的 0，每块盘重写撕裂的那一槽"
            );
            let Mounted {
                output,
                mut allocator,
                current,
            } = mount_writable(&parameters, &mut devices).unwrap_or_else(|refusal| {
                panic!("S = {slots_per_region} 段 {segment}：离线出路之后可写挂载做成；得到 {refusal:?}")
            });
            assert_eq!(
                output.instance.0,
                rewritten.journal_instance_written.0 + 1,
                "S = {slots_per_region} 段 {segment}：离线出路不加一，取号照常加一"
            );
            let content = file_content_of(1);
            publish_first_file(
                &mut PoolWriter::new(&parameters, devices.as_mut_slice()),
                &mut allocator,
                current.root(),
                FirstFile {
                    content: &content,
                    write_time_seconds: FIRST_WRITE_TIME_SECONDS,
                },
                output.instance,
                current.record_bytes(),
            )
            .unwrap_or_else(|refusal| {
                panic!(
                    "S = {slots_per_region} 段 {segment}：发得出第一个文件版本；得到 {refusal:?}"
                )
            });
            let image_after_the_file_version =
                memory_pool_of_sparse_devices(&devices, |device| &device.inner);
            assert_eq!(
                violated_invariants(&image_after_the_file_version),
                Vec::<&'static str>::new(),
                "S = {slots_per_region} 段 {segment}：池级 checker 无违例"
            );
            assert!(
                matches!(
                    recover_of_the_pool(
                        &image_after_the_file_version,
                        JournalPolicy::Consult,
                        &FILESYSTEM_IDENTIFIER
                    )
                    .outcome,
                    RecoveryOutcome::FileRead { content: read_back, .. } if read_back == content
                ),
                "S = {slots_per_region} 段 {segment}：恢复读得出第一个文件版本"
            );
        }
    }
}

/// 离线出路断言第 0 代之外的版本照拒：D 的根读得出、D 的记录两份都读回全零，断言 D 时写进系统配置的 tail 条款没写，一个字节不写。
#[test]
fn the_offline_way_out_asserting_a_version_after_the_genesis_without_its_last_record_is_still_refused(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = faults_on_the_records_of_the_newest_version(&history);
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&history.image, &faults, &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion { .. })
        ),
        "断言 D：根读得出、末条读不出，拒；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == history.image,
        "拒的时候一个字节不写"
    );
}

/// 断言第 0 代而第 0 代之后写过单元：tail 取得到，`birth_txg` 闸照拒，一个字节不写。
#[test]
fn the_offline_way_out_asserting_the_genesis_after_units_were_written_is_refused_by_the_birth_gate()
{
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::UnitBornAfterTheAssertedVersion { .. })
        ),
        "断言第 0 代：比它新的版本写过单元，拒；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == image,
        "拒的时候一个字节不写"
    );
}

/// mkfs 之后第一次可写挂载整段落盘（取号、写行、暖机）：交回落盘之后的盘面、那次挂载的全部写与它推了几次暖机。
fn image_and_writes_after_the_first_mount_on_a_formatted_pool(
    parameters: &MakeFilesystemParameters,
) -> (MemoryPool, Vec<RetainedWrite>, usize) {
    let mut formatted = empty_sparse_devices();
    make_filesystem(parameters, &mut formatted).expect("mkfs");
    let mut image = memory_pool_of_sparse_devices(&formatted, |device| device);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = formatted
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable(parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    image.apply_writes(&writes);
    (image, writes, mounted.output.warm_up_publishes.len())
}

/// 第一次可写挂载写下的 `kinds` 那几种写的落点，每段读回全零。
fn faults_on_the_writes_of_the_kinds(
    writes: &[RetainedWrite],
    kinds: &[StepKind],
) -> Vec<ReadFault> {
    writes
        .iter()
        .filter(|write| kinds.contains(&write.kind))
        .map(|write| ReadFault {
            device: write.device,
            offset_in_bytes: write.offset.0,
            length_in_bytes: write.length_in_bytes(),
            kind: ReadFaultKind::ReadsBackAllZeros,
        })
        .collect()
}

/// 第一次可写挂载（写行与暖机）整段落盘的盘面上，`kinds_reading_back_zero` 那几种写读回全零，离线出路断言第 0 代：
/// 单元读不出时 birth_txg 闸数不到（模块文档「本实例写下的单元一个都解不开」那一形），环里读得到比第 0 代新的东西就照拒，tail 条款没写；一个字节不写。
/// mkfs 之后第一次可写挂载的写行与暖机都是零单元发布、一个单元都没写，`StepKind::UnitWrite` 那一种在这份写表里没有落点；判的只是读坏的那一半环。
fn assert_the_offline_way_out_asserting_the_genesis_is_refused_with_the_tail_undecided(
    kinds_reading_back_zero: &[StepKind],
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let (image, writes, warm_up_publishes) =
        image_and_writes_after_the_first_mount_on_a_formatted_pool(&parameters);
    assert!(warm_up_publishes > 0, "前提：这次挂载推了暖机");
    let faults = faults_on_the_writes_of_the_kinds(&writes, kinds_reading_back_zero);
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &faults, &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion { .. })
        ),
        "断言第 0 代：写下的单元读不出、环里读得到比它新的东西，拒；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == image,
        "拒的时候一个字节不写"
    );
}

/// 写行与暖机的根读得出，单元与记录都读回全零：判的是根环那一半。
#[test]
fn the_offline_way_out_asserting_the_genesis_is_refused_when_newer_roots_read_while_units_and_records_after_it_read_back_zero(
) {
    assert_the_offline_way_out_asserting_the_genesis_is_refused_with_the_tail_undecided(&[
        StepKind::UnitWrite,
        StepKind::JournalRecord,
    ]);
}

/// 写行与暖机的记录读得出，单元与根都读回全零：判的是 journal 环那一半。
#[test]
fn the_offline_way_out_asserting_the_genesis_is_refused_when_newer_records_read_while_units_and_roots_after_it_read_back_zero(
) {
    assert_the_offline_way_out_asserting_the_genesis_is_refused_with_the_tail_undecided(&[
        StepKind::UnitWrite,
        StepKind::RootRecordFua,
    ]);
}

/// 同一份盘面没注入读故障：mkfs 之后第一次可写挂载要写的行为空（上一个实例是 mkfs 的第 0 代），写行那次发布与暖机都是零单元发布
/// （`mount.rs` 的 `acquire_then_publish_until_the_continuation` 里 `PreviousVersion::WithoutFile` 那一支），单元区里没有比第 0 代新的单元，
/// birth_txg 闸数不到东西；根环与 journal 环里读得到写行与暖机的根和记录，照拒在 tail 条款没写上，一个字节不写。
#[test]
fn the_offline_way_out_asserting_the_genesis_after_the_first_mount_of_zero_unit_publishes_is_refused_with_the_tail_undecided_without_read_faults(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let (image, writes, warm_up_publishes) =
        image_and_writes_after_the_first_mount_on_a_formatted_pool(&parameters);
    assert!(warm_up_publishes > 0, "前提：这次挂载推了暖机");
    assert!(
        writes.iter().all(|write| write.kind != StepKind::UnitWrite),
        "前提：mkfs 之后第一次可写挂载的写行与暖机都是零单元发布，一个单元都没写"
    );
    let faults_lifted = Rc::new(Cell::new(false));
    let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
    let outcome = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters,
        &mut devices,
        AssertedVersion {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
    );
    assert!(
        matches!(
            outcome,
            Err(SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion { .. })
        ),
        "断言第 0 代：环里读得到比它新的根与记录，拒在 tail 条款没写上；得到 {outcome:?}"
    );
    assert!(
        memory_pool_of_sparse_devices(&devices, |device| &device.inner) == image,
        "拒的时候一个字节不写"
    );
}
