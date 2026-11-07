//! E205（多处读坏倒挂三条改法的代价）：入库装置。跑前登记 `research/prompts/e205-preregistration.md`（第五节「装置写在哪」写死入库装置）。
//!
//! 回答岔路单 `research/prompts/m3-close-b19-c971-forks.md` 五行：甲并全零、甲只并读坏、乙直拒、乙入判不出、丙。
//!
//! 只读、只驱动、只观测：只调 `crates/` 的公开入口造历史与读盘，不改 `crates/singlefs-core`、`crates/singlefs-checker`。
//! 臂的名字（产物里写这些，登记里的叫法对应如下）：
//!   丙 = `today`（产品入口 `mount_writable`）；丙经开关今天那一档 = `today_via_switch`；
//!   甲只并读坏 = `past_read_bad`；甲并全零 = `past_read_bad_or_all_zero`（新实例第一个 txg 的只供测试开关
//!   `FirstTxgOfTheNewInstance` 的另两档，入口 `mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance`）；
//!   乙直拒 = `refuse_directly`；乙入判不出 = `into_undecidable`（乙两种读法按挂载那一刻的读数在装置里现算，不要写者）。
//!
//! 几何、工作量与判据门槛一律用本文件的本地常量，值抄自跑前登记第五节并注明出处；`constant_checks` 逐个与实现侧同名常量回比，
//! 不等就是作废条款（登记第十一节第一条），整趟不出结论。
//!
//! 模式（第一个参数）：
//!   `checks`                    本地常量回比，不建历史
//!   `six-cells`                 第一段：六格、对照格、阳性对照、负对照、停机对拍 S1–S3、S5–S7、S9，单线程与多线程逐行对拍
//!   `torn-rotation`             第二段：撕裂轮换族，三组 S
//!   `first-txg-skips s8`        第三段 S = 8 的两组 (p = 2、p = 24)
//!   `first-txg-skips s4-s16`    第三段 S = 4、16 的四组
//!
//! 逐格判定入库（十三节）：四段一律经 `scripts/experiment-apparatus/device-code-key.py run` 起，每段给它判的那几个格入口
//! （第一段 `fault_cell_verdict`、`control_cell_verdict` 与单线程对拍那一条 `segment_one_thread_cross_check_verdict`，
//! 第二段 `torn_state_verdict`，第三段 `skip_state_verdict`），例
//! `python3 scripts/experiment-apparatus/device-code-key.py run . --binary e205_c971_inverted_first_txg_arm_costs
//! --entry e205_c971_inverted_first_txg_arm_costs::fault_cell_verdict --entry e205_c971_inverted_first_txg_arm_costs::control_cell_verdict
//! --entry e205_c971_inverted_first_txg_arm_costs::segment_one_thread_cross_check_verdict -- six-cells`；
//! 它照各入口的单元闭包算代码键、经 `--code-key-file` 交进来，判定经它开的两根管道转给 prune-server 的那一份判定库。
//! 键相同的格复用、不重判；判红的记进红表、下一趟照判。成功句（标准错误）报「现判 N、复用 M、拒算 R」；标准输出的产物与复用与否无关。
//! 第一段的单线程对拍（登记第十四节「第一段 249 态全量单线程再判一遍」、第十二节修订十二）本身是一条判定：键对不上的那一趟
//! 第一段全部的件切片多线程判一遍、单线程再判一遍，逐件逐字相同才记绿；键相同的复用这一条、不重判；成功句另报一行它是现判还是复用。
//!
//! 环境变量：`E205_THREADS`（工作线程数，没设取每台默认重活线程数：task-server 设好的 `SINGLEFS_THREAD_CAP`，
//! 再没设取 `.claude/heavy-thread-default` 的登记值）。
//!
//! 产物按 `E7RESULT` 行打到标准输出：全部判完之后按件号的次序一次打出（与线程数无关，逐字节可比）；进度打到标准错误。
//! 装置不调崩溃判段入口，只用 `memory_pool` 取写表、分段，在 CPU 上逐态判（登记第五节）：`cargo run` 不带 `--features gpu`。
//!
//! 准入写 `always`：复跑登记（`replay.sh` 的 `driver_e205` 几行）要在输入不变时再跑同一段、核产物逐字节相同。

// admission: always 复跑登记要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
    GENESIS_CHECKPOINT_TXG, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance,
    ring_slots_bad_or_reading_back_all_zero_at_mount_by, unmount, FirstTxgOfTheNewInstance,
    MountError, Mounted, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, instance_table_of_root,
    read_root_ring_slot, root_is_abandoned_by_the_instance_table, scan_journal, PoolReader,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
};
use singlefs_core::system_configuration_repair::{
    rewrite_system_configuration_slots_for_the_asserted_version, AssertedVersion,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_verification::crash::torn_image_of_in_place_overwrite;
use singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads;
use singlefs_verification::memory_pool::{
    root_identity_of_write, writes_and_segments_with_stream_indexes_and_entries, MemoryPool,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{
    RecordedPublishEntry, RecordingBlockDevice, RetainedOperation, SharedStream,
};

// ============================================================================
// 一、本地常量（登记第五节「装置要读的 `crates/` 常量」：逐个写成本地常量，`constant_checks` 与实现侧同名常量回比）。
// ============================================================================

/// 根环区域数：`crates/singlefs-format/src/lib.rs` 第 291 行 `ROOT_RING_REGIONS`。
const ROOT_RING_REGIONS_LOCAL: u64 = 3;
/// 每区槽数下限：同一文件第 300 行 `ROOT_RING_SLOTS_PER_REGION_MINIMUM`。
const SLOTS_PER_REGION_MINIMUM_LOCAL: u64 = 4;
/// 每区槽数上限：同一文件第 304 行 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM`。
const SLOTS_PER_REGION_MAXIMUM_LOCAL: u64 = 16;
/// mkfs 显式入口的每区槽数：同一文件第 309 行 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`（G 主点）。
const SLOTS_PER_REGION_AT_MAKE_FILESYSTEM_LOCAL: u64 = 8;
/// 每块盘系统配置槽数：同一文件第 280 行 `SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE`。
const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE_LOCAL: u64 = 2;
/// 系统配置槽字节：同一文件第 287 行 `SYSTEM_CONFIGURATION_SLOT_BYTES`。
const SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL: u64 = 4096;
/// journal 环默认字节：同一文件第 265 行 `JOURNAL_RING_DEFAULT_BYTES`（副本模型的几何，丙六格要与它比）。
const JOURNAL_RING_DEFAULT_BYTES_LOCAL: u64 = 805_306_368;
/// journal 记录字节：同一文件第 241 行 `JOURNAL_RECORD_BYTES`（点名读坏的记录按它取长度）。
const JOURNAL_RECORD_BYTES_LOCAL: u64 = 4096;
/// 第 0 代根的 txg：`crates/singlefs-core/src/make_filesystem.rs` 第 55 行 `GENESIS_CHECKPOINT_TXG`；它住三个区域各自的槽 0。
const GENESIS_CHECKPOINT_TXG_LOCAL: u64 = 0;
const GENESIS_RING_SLOT_LOCAL: u64 = 0;

/// 每块盘的字节数与几何：照副本模型（`research/prompts/m3-close-b19-e165-r1-opus-b-model/e165_attack_form_b_sliced.rs`
/// 第 72–74、113–125 行）取，丙六格要与它逐字段比（登记第四节）。
const DEVICE_BYTES: u64 = 4 << 30;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
/// 根槽一段读坏的长度（一个物理块，根记录住在里面）。
const ROOT_SLOT_BYTES: u64 = 512;
/// 同副本模型第 76–78 行的 fsid。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 读得出、不是全零而自证不过那一种读坏：读回的字节里翻这一个（同副本模型第 81 行）。
const FLIPPED_BYTE_INSIDE_A_RANGE: u64 = 100;
/// 第一个文件的 inode 号。
const FIRST_FILE: InodeNumber = InodeNumber(1);
/// 第一个文件的内容长（同副本模型 `file_content_of`：一个数据单元）。
const FILE_CONTENT_BYTES: u64 = 3000;
/// 起点历史里 C、D 两版的 txg（登记第一节「六格」）。
const TXG_OF_SECOND_OVERWRITE: u64 = 5;
const TXG_OF_THIRD_OVERWRITE: u64 = 6;
/// 起点历史里各版内容的序号：新池新建文件 1、B 2、C 3、D 4（序号也是内容的种子，大的新）。
const CONTENT_INDEX_OF_SECOND_OVERWRITE: u64 = 3;
/// 第三段每条族历史的挂载次数与态取前几次挂载（登记第六节第三段「mkfs 之后 8 次……前 4 次挂载那一段写流」）。
const MOUNTS_OF_A_FAMILY_HISTORY: u64 = 8;
const MOUNTS_WHOSE_STATES_ARE_TAKEN: u64 = 4;
/// 第二段至多取几段系统配置写（登记第六节第二段「至多 20 段系统配置写」）。
const TORN_SEGMENTS_AT_MOST: usize = 20;
/// 甲跳号轨迹那条持续读坏的根环槽：区域 0 的槽 1（登记第六节第三段「甲跳号轨迹」）。
const TRAJECTORY_FAULTED_RING_SLOT: RootRingSlot = RootRingSlot { region: 0, slot: 1 };

/// 本地常量与实现侧同名常量回比（作废条款第一条）：交回每一项 (名字, 本地值, 实现侧的值)。
fn constant_checks() -> Vec<(&'static str, u64, u64)> {
    vec![
        (
            "root_ring_regions",
            ROOT_RING_REGIONS_LOCAL,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        (
            "slots_per_region_minimum",
            SLOTS_PER_REGION_MINIMUM_LOCAL,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        ),
        (
            "slots_per_region_maximum",
            SLOTS_PER_REGION_MAXIMUM_LOCAL,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        ),
        (
            "slots_per_region_at_make_filesystem",
            SLOTS_PER_REGION_AT_MAKE_FILESYSTEM_LOCAL,
            RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
        ),
        (
            "system_configuration_slots_per_device",
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE_LOCAL,
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        ),
        (
            "system_configuration_slot_bytes",
            SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
        ),
        (
            "journal_ring_default_bytes",
            JOURNAL_RING_DEFAULT_BYTES_LOCAL,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "journal_record_bytes",
            JOURNAL_RECORD_BYTES_LOCAL,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        (
            "genesis_checkpoint_txg",
            GENESIS_CHECKPOINT_TXG_LOCAL,
            GENESIS_CHECKPOINT_TXG.0,
        ),
    ]
}

/// 第 0 代根住的槽（三个区域各自的槽 0）与实现侧回比：mkfs 之后的盘面上自证读出的 txg 0 的根落在哪几槽。
fn genesis_slots_read_on(
    image: &MemoryPool,
    parameters: &MakeFilesystemParameters,
) -> BTreeSet<RootRingSlot> {
    ring_slot_readings(&FaultyImage { image, faults: &[] }, parameters)
        .into_iter()
        .filter_map(|(slot, reading)| match reading {
            RingSlotReading::SelfVerified { txg, .. } if txg == GENESIS_CHECKPOINT_TXG_LOCAL => {
                Some(slot)
            }
            RingSlotReading::SelfVerified { .. }
            | RingSlotReading::BadNotAllZero
            | RingSlotReading::AllZero => None,
        })
        .collect()
}

fn expected_genesis_slots() -> BTreeSet<RootRingSlot> {
    (0..ROOT_RING_REGIONS_LOCAL)
        .map(|region| RootRingSlot {
            region,
            slot: GENESIS_RING_SLOT_LOCAL,
        })
        .collect()
}

// ============================================================================
// 二、几何、内容、读故障。
// ============================================================================

fn pool_parameters(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES_LOCAL,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("装置取的 S 是 4、8、16，都在系统配置收的区间里"),
        },
    }
}

fn fixed_geometry(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn slots_per_region_of(parameters: &MakeFilesystemParameters) -> u64 {
    parameters.geometry.root_ring_slots_per_region.count()
}

/// 第 `index` 版内容（同副本模型 `file_content_of`）：序号大的版本新，读回来的字节认得出是哪一版。
fn file_content_of(index: u64) -> Vec<u8> {
    (0..FILE_CONTENT_BYTES)
        .map(|byte| u8::try_from((byte * 7 + index) % 251).expect("模 251 装得进 u8"))
        .collect()
}

/// 一次读坏是哪一种（登记第一节「六格」：设备报读错 / 读回全零 / 读得出而自证不过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ReadFaultKind {
    DeviceError,
    AllZero,
    NotSelfVerified,
}

impl ReadFaultKind {
    const EVERY: [ReadFaultKind; 3] = [
        ReadFaultKind::DeviceError,
        ReadFaultKind::AllZero,
        ReadFaultKind::NotSelfVerified,
    ];

    fn name(self) -> &'static str {
        match self {
            ReadFaultKind::DeviceError => "device_error",
            ReadFaultKind::AllZero => "all_zero",
            ReadFaultKind::NotSelfVerified => "not_self_verified",
        }
    }
}

/// 一段读坏的落点：哪块盘、从哪个字节起、多长、哪一种。被写过就撤掉（之后读回新字节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadFault {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    kind: ReadFaultKind,
}

impl ReadFault {
    fn overlaps(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && offset < self.offset + self.length && self.offset < offset + length
    }
}

/// 照读故障改读回的字节；设备报读错交回 `false`（调用方报读错）。
fn apply_read_faults(
    faults: &[ReadFault],
    device: DeviceIdentity,
    offset: u64,
    buffer: &mut [u8],
) -> bool {
    let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
    for fault in faults
        .iter()
        .filter(|fault| fault.overlaps(device, offset, length))
    {
        let start = usize::try_from(fault.offset.max(offset) - offset).expect("落在缓冲区里");
        let end = usize::try_from((fault.offset + fault.length).min(offset + length) - offset)
            .expect("落在缓冲区里");
        match fault.kind {
            ReadFaultKind::DeviceError => return false,
            ReadFaultKind::AllZero => buffer[start..end].fill(0),
            ReadFaultKind::NotSelfVerified => {
                let flipped = fault.offset + FLIPPED_BYTE_INSIDE_A_RANGE;
                let index = if flipped >= offset && flipped < offset + length {
                    usize::try_from(flipped - offset).expect("落在缓冲区里")
                } else {
                    start
                };
                buffer[index] ^= 0xff;
            }
        }
    }
    true
}

/// 读的时候按 `faults` 读坏的盘；写、写零、discard 原样交给里面那块盘，写到的故障段当场撤掉。`faults` 清空就是一块好盘。
struct DeviceUnderTest<Inner: BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    faults: Vec<ReadFault>,
}

impl<Inner: BlockDevice> DeviceUnderTest<Inner> {
    fn lift_the_faults_written_over(&mut self, offset: u64, length: u64) {
        let identity = self.identity;
        self.faults
            .retain(|fault| !fault.overlaps(identity, offset, length));
    }
}

impl<Inner: BlockDevice> BlockDevice for DeviceUnderTest<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        if apply_read_faults(&self.faults, self.identity, offset.0, buffer) {
            Ok(())
        } else {
            Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "装置点名的这一段读坏：设备报读错",
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

/// 一张镜像带一份读故障：装置自己读根环、记录与系统配置槽用它（与挂载读到的是同一份字节）。
struct FaultyImage<'image> {
    image: &'image MemoryPool,
    faults: &'image [ReadFault],
}

impl PoolReader for FaultyImage<'_> {
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

/// 一张镜像拷成两块内存盘（不带读故障、不录制）。
fn devices_of(
    image: &MemoryPool,
    faults: &[ReadFault],
) -> Vec<(DeviceIdentity, DeviceUnderTest<SparseBlockDevice>)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut inner = SparseBlockDevice::new(
                DEVICE_BYTES,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            );
            inner.image = sparse.clone();
            let own_faults = faults
                .iter()
                .filter(|fault| fault.device == *identity)
                .copied()
                .collect();
            (
                *identity,
                DeviceUnderTest {
                    identity: *identity,
                    inner,
                    faults: own_faults,
                },
            )
        })
        .collect()
}

/// 同上，再接一层录制（写表与分段要它）。
fn recorded_devices_of(
    image: &MemoryPool,
    faults: &[ReadFault],
    stream: &SharedStream,
) -> Vec<(
    DeviceIdentity,
    DeviceUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
)> {
    devices_of(image, faults)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceUnderTest {
                    identity,
                    inner: RecordingBlockDevice::with_shared_stream(
                        identity,
                        device.inner,
                        stream.clone(),
                    ),
                    faults: device.faults,
                },
            )
        })
        .collect()
}

fn image_of_plain_devices(
    devices: &[(DeviceIdentity, DeviceUnderTest<SparseBlockDevice>)],
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

fn image_of_recorded_devices(
    devices: &[(
        DeviceIdentity,
        DeviceUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    )],
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
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

/// 只做过 mkfs 的盘面。
fn image_right_after_make_filesystem(parameters: &MakeFilesystemParameters) -> MemoryPool {
    let mut devices = empty_sparse_devices();
    make_filesystem(parameters, &mut devices).expect("两块空盘上 mkfs");
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

// ============================================================================
// 三、臂、装置自己的现算（今天的第一个 txg、甲两档、乙两种读法、R2、挂载时读坏槽能住的最大 txg）。
// ============================================================================

/// 挂载走哪一条臂（登记第五节「臂」）。乙两种读法不挂载，按读数现算（`yi_refuses`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Arm {
    /// 丙：产品入口 `mount_writable`。
    Today,
    /// 丙经开关今天那一档（停机对拍 S7：与产品入口逐字节相同）。
    TodayViaSwitch,
    /// 甲只并读坏。
    PastReadBad,
    /// 甲并全零。
    PastReadBadOrAllZero,
}

impl Arm {
    const MOUNTED_ON_THE_SIX_CELLS: [Arm; 4] = [
        Arm::Today,
        Arm::TodayViaSwitch,
        Arm::PastReadBad,
        Arm::PastReadBadOrAllZero,
    ];
    const OF_THE_FAMILIES: [Arm; 3] = [Arm::Today, Arm::PastReadBadOrAllZero, Arm::PastReadBad];
    const JIA: [Arm; 2] = [Arm::PastReadBadOrAllZero, Arm::PastReadBad];

    fn name(self) -> &'static str {
        match self {
            Arm::Today => "today",
            Arm::TodayViaSwitch => "today_via_switch",
            Arm::PastReadBad => "past_read_bad",
            Arm::PastReadBadOrAllZero => "past_read_bad_or_all_zero",
        }
    }
}

/// 这条臂的挂载传开关的哪一档；丙走产品入口、不传开关。
fn switch_setting_of(arm: Arm) -> Option<FirstTxgOfTheNewInstance> {
    match arm {
        Arm::Today => None,
        Arm::TodayViaSwitch => Some(FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing),
        Arm::PastReadBad => Some(FirstTxgOfTheNewInstance::PastRingSlotsReadBadAtMount),
        Arm::PastReadBadOrAllZero => {
            Some(FirstTxgOfTheNewInstance::PastRingSlotsReadBadOrAllZeroAtMount)
        }
    }
}

/// 照这条臂做一次可写挂载。
fn mount_under_arm<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    arm: Arm,
) -> Result<Mounted, MountError> {
    match switch_setting_of(arm) {
        None => mount_writable(parameters, devices),
        Some(setting) => {
            mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance(
                parameters,
                devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                setting,
            )
        }
    }
}

/// 挂载报错的短名（`Debug` 的第一个词）。
fn short_name_of_debug(text: &str) -> String {
    let cut = text.find(['(', '{', ' ']).unwrap_or(text.len());
    text[..cut].to_string()
}

fn refusal_name(error: &MountError) -> String {
    short_name_of_debug(&format!("{error:?}"))
}

/// 装置自己读一个根环槽读到的（只看字节，不经挂载）：读不出、读得出不是全零而自证不过（含错位读）、读回全零、自证过。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RingSlotReading {
    BadNotAllZero,
    AllZero,
    SelfVerified { instance: u32, txg: u64 },
}

/// 根环每一槽各读一次（经带读故障的读者），按读根环次序。
fn ring_slot_readings(
    reader: &dyn PoolReader,
    parameters: &MakeFilesystemParameters,
) -> Vec<(RootRingSlot, RingSlotReading)> {
    let slot_bytes = usize::try_from(ROOT_SLOT_BYTES).expect("512");
    every_root_ring_slot(&parameters.geometry)
        .into_iter()
        .map(|ring_slot| {
            let device =
                parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
            let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
            let reading = match reader.read(device, offset, slot_bytes) {
                None => RingSlotReading::BadNotAllZero,
                Some(bytes) if bytes.iter().all(|byte| *byte == 0) => RingSlotReading::AllZero,
                Some(_) => match read_root_ring_slot(
                    reader,
                    &parameters.region_devices,
                    &parameters.geometry,
                    &parameters.filesystem_identifier,
                    ring_slot,
                ) {
                    RootRingSlotReading::SelfVerified(root) => RingSlotReading::SelfVerified {
                        instance: root.instance.0,
                        txg: root.checkpoint_txg.0,
                    },
                    RootRingSlotReading::Bad(_) => RingSlotReading::BadNotAllZero,
                },
            };
            (ring_slot, reading)
        })
        .collect()
}

fn highest_self_verified_root_txg(readings: &[(RootRingSlot, RingSlotReading)]) -> u64 {
    readings
        .iter()
        .filter_map(|(_, reading)| match reading {
            RingSlotReading::SelfVerified { txg, .. } => Some(*txg),
            RingSlotReading::BadNotAllZero | RingSlotReading::AllZero => None,
        })
        .max()
        .unwrap_or(0)
}

/// 今天的第一个 txg（`first_txg_of_new_instance` 的式子，D23（journal 的角色与格式） 已定项 14 注 3）：
/// max(根环里自证过的根的最大 txg, 环里自证通过的记录的最大 checkpoint_txg) + 1。
fn first_txg_today(highest_root_txg: u64, highest_record_txg: u64) -> u64 {
    highest_root_txg.max(highest_record_txg) + 1
}

/// 甲越过哪几类槽（登记第一节「甲的两种读法」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotsPassed {
    /// 甲只并读坏：读不出、读得出不是全零而自证不过。
    ReadBad,
    /// 甲并全零：再加读回全零。
    ReadBadOrAllZero,
}

fn slots_passed_by(arm: Arm) -> Option<SlotsPassed> {
    match arm {
        Arm::Today | Arm::TodayViaSwitch => None,
        Arm::PastReadBad => Some(SlotsPassed::ReadBad),
        Arm::PastReadBadOrAllZero => Some(SlotsPassed::ReadBadOrAllZero),
    }
}

/// 这一槽算不算进甲那一档要越过的槽。
fn ring_slot_is_passed(slots_passed: SlotsPassed, reading: RingSlotReading) -> bool {
    match (slots_passed, reading) {
        (_, RingSlotReading::SelfVerified { .. }) => false,
        (_, RingSlotReading::BadNotAllZero) => true,
        (SlotsPassed::ReadBadOrAllZero, RingSlotReading::AllZero) => true,
        (SlotsPassed::ReadBad, RingSlotReading::AllZero) => false,
    }
}

/// 落点公式（`root_ring.rs` 的 `target_for_publish`，装置另写一份：区域 = txg mod 3，区域内槽 = ⌊txg / 3⌋ mod S）。
fn ring_slot_of_txg(txg: u64, slots_per_region: u64) -> RootRingSlot {
    RootRingSlot {
        region: txg % ROOT_RING_REGIONS_LOCAL,
        slot: (txg / ROOT_RING_REGIONS_LOCAL) % slots_per_region,
    }
}

/// 甲给的第一个 txg（登记第一节「两种读法共用的取法」）：H_槽 = [N + 1, N + 3S − 1] 里落到这一槽的那个 txg，
/// 甲给的 = max(今天的值, max H_槽 + 1)；没有算进来的槽时等于今天的值。
fn first_txg_past_the_ring_slots(
    slots_passed: SlotsPassed,
    readings: &[(RootRingSlot, RingSlotReading)],
    slots_per_region: u64,
    first_txg_of_today: u64,
) -> u64 {
    let highest_root_txg = highest_self_verified_root_txg(readings);
    let ring_length_in_txgs = ROOT_RING_REGIONS_LOCAL * slots_per_region;
    let passed: BTreeSet<RootRingSlot> = readings
        .iter()
        .filter(|(_, reading)| ring_slot_is_passed(slots_passed, *reading))
        .map(|(slot, _)| *slot)
        .collect();
    let mut highest_txg_a_passed_slot_can_hold = None;
    for txg in (highest_root_txg + 1)..=(highest_root_txg + ring_length_in_txgs - 1) {
        if passed.contains(&ring_slot_of_txg(txg, slots_per_region)) {
            highest_txg_a_passed_slot_can_hold = Some(txg);
        }
    }
    highest_txg_a_passed_slot_can_hold
        .map_or(first_txg_of_today, |held| first_txg_of_today.max(held + 1))
}

/// 装置在一张镜像、一份读故障上现算的这次挂载的读数（挂载之前、不经挂载）。
#[derive(Clone, Debug)]
struct ReadingsBeforeTheMount {
    ring: Vec<(RootRingSlot, RingSlotReading)>,
    highest_record_txg: u64,
    first_txg_of_today: u64,
    first_txg_past_read_bad: u64,
    first_txg_past_read_bad_or_all_zero: u64,
    system_configuration: Vec<[SystemConfigurationSlotReading; 2]>,
}

impl ReadingsBeforeTheMount {
    fn first_txg_of(&self, arm: Arm) -> u64 {
        match slots_passed_by(arm) {
            None => self.first_txg_of_today,
            Some(SlotsPassed::ReadBad) => self.first_txg_past_read_bad,
            Some(SlotsPassed::ReadBadOrAllZero) => self.first_txg_past_read_bad_or_all_zero,
        }
    }
}

fn readings_before_the_mount(
    image: &MemoryPool,
    faults: &[ReadFault],
    parameters: &MakeFilesystemParameters,
) -> ReadingsBeforeTheMount {
    let reader = FaultyImage { image, faults };
    let ring = ring_slot_readings(&reader, parameters);
    let highest_record_txg = choose_system_configuration(image).map_or(0, |configuration| {
        scan_journal(&reader, &configuration)
            .values()
            .map(|record| record.checkpoint_txg.0)
            .max()
            .unwrap_or(0)
    });
    let first_txg_of_today =
        first_txg_today(highest_self_verified_root_txg(&ring), highest_record_txg);
    let slots_per_region = slots_per_region_of(parameters);
    ReadingsBeforeTheMount {
        first_txg_past_read_bad: first_txg_past_the_ring_slots(
            SlotsPassed::ReadBad,
            &ring,
            slots_per_region,
            first_txg_of_today,
        ),
        first_txg_past_read_bad_or_all_zero: first_txg_past_the_ring_slots(
            SlotsPassed::ReadBadOrAllZero,
            &ring,
            slots_per_region,
            first_txg_of_today,
        ),
        system_configuration: system_configuration_slot_readings(&reader),
        ring,
        highest_record_txg,
        first_txg_of_today,
    }
}

/// 装置读一槽系统配置读到的（与 `system_configuration_slots_of_one_device_read_once` 同口径：整槽校验和过、fsid 与本池相同才算自证过；
/// 布局号不认、每区槽数越界的不算坏，登记第一节「乙」那几条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SystemConfigurationSlotReading {
    SelfVerified,
    DeviceReadError,
    AllZero,
    NotSelfVerified,
    NotCountedAsBad,
}

fn system_configuration_slot_readings(
    reader: &dyn PoolReader,
) -> Vec<[SystemConfigurationSlotReading; 2]> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    let read_one = |device: DeviceIdentity, offset: u64| match reader.read(
        device,
        DeviceOffsetInBytes(offset),
        slot_bytes,
    ) {
        None => SystemConfigurationSlotReading::DeviceReadError,
        Some(bytes) if bytes.iter().all(|byte| *byte == 0) => {
            SystemConfigurationSlotReading::AllZero
        }
        Some(bytes) => match SystemConfiguration::parse_slot(&bytes) {
            Ok(configuration)
                if configuration.immutable.filesystem_identifier == FILESYSTEM_IDENTIFIER =>
            {
                SystemConfigurationSlotReading::SelfVerified
            }
            Ok(_) => SystemConfigurationSlotReading::NotCountedAsBad,
            Err(
                SystemConfigurationSlotRefusal::NotSelfDescribing
                | SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },
            ) => SystemConfigurationSlotReading::NotSelfVerified,
            Err(
                SystemConfigurationSlotRefusal::IncompatBitsNotRecognized { .. }
                | SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(_),
            ) => SystemConfigurationSlotReading::NotCountedAsBad,
        },
    };
    let spacing = u64::from(FIXED_STRUCTURE_SLOT_SPACING);
    reader
        .device_identities()
        .into_iter()
        .map(|device| [read_one(device, 0), read_one(device, spacing)])
        .collect()
}

/// 乙两种读法（登记第一节「乙的两种读法」）。重读一次读回同样的字节（读故障在一次挂载里不变，S5 核），所以按一遍读数判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum YiReading {
    /// 乙直拒：有一槽读回全零或读得出而自证不过就拒。
    RefuseDirectly,
    /// 乙入判不出：仍坏的槽并进 R2「有槽设备报读错」那一半，R2 判不出就拒。
    IntoUndecidable,
}

impl YiReading {
    const EVERY: [YiReading; 2] = [YiReading::RefuseDirectly, YiReading::IntoUndecidable];

    fn name(self) -> &'static str {
        match self {
            YiReading::RefuseDirectly => "refuse_directly",
            YiReading::IntoUndecidable => "into_undecidable",
        }
    }
}

fn both_slots_self_verified(slots: &[SystemConfigurationSlotReading; 2]) -> bool {
    slots
        .iter()
        .all(|reading| *reading == SystemConfigurationSlotReading::SelfVerified)
}

/// R2（`system_configuration_witness_is_undecidable` 的定义，停机对拍 S2）：有槽设备报读错 ∧ 没有一块盘两槽都自证过。
fn witness_undecidable_today(devices: &[[SystemConfigurationSlotReading; 2]]) -> bool {
    let some_slot_reports_a_read_error = devices
        .iter()
        .flatten()
        .any(|reading| *reading == SystemConfigurationSlotReading::DeviceReadError);
    some_slot_reports_a_read_error && !devices.iter().any(both_slots_self_verified)
}

/// 乙拒不拒可写。
fn yi_refuses(yi: YiReading, devices: &[[SystemConfigurationSlotReading; 2]]) -> bool {
    match yi {
        YiReading::RefuseDirectly => devices.iter().flatten().any(|reading| {
            matches!(
                reading,
                SystemConfigurationSlotReading::AllZero
                    | SystemConfigurationSlotReading::NotSelfVerified
            )
        }),
        YiReading::IntoUndecidable => {
            let counted_as_a_read_error = devices.iter().flatten().any(|reading| {
                matches!(
                    reading,
                    SystemConfigurationSlotReading::DeviceReadError
                        | SystemConfigurationSlotReading::AllZero
                        | SystemConfigurationSlotReading::NotSelfVerified
                )
            });
            let no_device_has_both_slots_self_verified =
                !devices.iter().any(both_slots_self_verified);
            counted_as_a_read_error && no_device_has_both_slots_self_verified
        }
    }
}

/// 与 C596（见证判不出的系统配置盘面只剩只读挂载） 已定的冲突格：设备没报读错、每块盘至少一槽自证过、坏的槽都是撕裂或读得出自证不过。
fn conflicts_with_c596(devices: &[[SystemConfigurationSlotReading; 2]]) -> bool {
    devices.iter().flatten().all(|reading| {
        matches!(
            reading,
            SystemConfigurationSlotReading::SelfVerified
                | SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::NotCountedAsBad
        )
    }) && devices
        .iter()
        .all(|slots| slots.contains(&SystemConfigurationSlotReading::SelfVerified))
}

fn system_configuration_readings_text(devices: &[[SystemConfigurationSlotReading; 2]]) -> String {
    let name = |reading: &SystemConfigurationSlotReading| match reading {
        SystemConfigurationSlotReading::SelfVerified => "ok",
        SystemConfigurationSlotReading::DeviceReadError => "device_error",
        SystemConfigurationSlotReading::AllZero => "all_zero",
        SystemConfigurationSlotReading::NotSelfVerified => "not_self_verified",
        SystemConfigurationSlotReading::NotCountedAsBad => "not_counted",
    };
    devices
        .iter()
        .map(|slots| format!("{}/{}", name(&slots[0]), name(&slots[1])))
        .collect::<Vec<_>>()
        .join(",")
}

/// 停机对拍 S3：挂载那一刻读坏或读回全零、这次挂载没写过的根环槽，各按 (N − 3S, first_txg − 1] 里落到它的最大 txg
/// （照 `mount.rs` 的 `txgs_ring_slots_bad_at_mount_can_hold`：只收 [0, min(first_txg, 3S)) 里落得到 txg 的槽）。
fn expected_bad_slot_txgs(
    readings: &[(RootRingSlot, RingSlotReading)],
    slots_written_by_the_mount: &BTreeSet<RootRingSlot>,
    first_txg: u64,
    slots_per_region: u64,
) -> BTreeMap<RootRingSlot, u64> {
    let highest_root_txg = highest_self_verified_root_txg(readings);
    let ring_length_in_txgs = ROOT_RING_REGIONS_LOCAL * slots_per_region;
    let lowest_txg_inside_the_window = highest_root_txg
        .checked_sub(ring_length_in_txgs)
        .map_or(0, |below| below + 1);
    let walked = first_txg
        .saturating_sub(lowest_txg_inside_the_window)
        .min(ring_length_in_txgs);
    let bad: BTreeSet<RootRingSlot> = readings
        .iter()
        .filter(|(slot, reading)| {
            !matches!(reading, RingSlotReading::SelfVerified { .. })
                && !slots_written_by_the_mount.contains(slot)
        })
        .map(|(slot, _)| *slot)
        .collect();
    let reachable: BTreeSet<RootRingSlot> = (0..first_txg.min(ring_length_in_txgs))
        .map(|txg| ring_slot_of_txg(txg, slots_per_region))
        .collect();
    let mut highest = BTreeMap::new();
    for txg in (first_txg - walked)..first_txg {
        let slot = ring_slot_of_txg(txg, slots_per_region);
        if bad.contains(&slot) && reachable.contains(&slot) {
            highest.insert(slot, txg);
        }
    }
    highest
}

/// S3 的前提（登记第十一节 S3「只在环里没有被抛弃根的态上比」）：挂载之前、带着这一态的读故障，根环里自证过的根里，
/// 按最新那条根（txg 最大、同 txg 取实例代号大的）的实例表判被抛弃的有几条。环里没有自证过的根、或最新那条根的实例表读不出时 `None`（判不了）。
fn abandoned_roots_on_the_ring_before_the_mount(
    reader: &dyn PoolReader,
    parameters: &MakeFilesystemParameters,
) -> Option<usize> {
    let roots: Vec<RootRecord> = every_root_ring_slot(&parameters.geometry)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                reader,
                &parameters.region_devices,
                &parameters.geometry,
                &parameters.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some(root),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect();
    let newest = roots
        .iter()
        .max_by_key(|root| (root.checkpoint_txg.0, root.instance.0))?;
    let table = instance_table_of_root(reader, newest)?;
    Some(
        roots
            .iter()
            .filter(|root| root_is_abandoned_by_the_instance_table(root, &table))
            .count(),
    )
}

/// S3 一态比出来的。
#[derive(Clone, Debug, PartialEq, Eq)]
enum BadSlotTxgsComparison {
    Equal,
    Differs {
        apparatus: BTreeMap<RootRingSlot, u64>,
    },
    NotComparedBecauseOfAbandonedRoots,
    NotComparedBecauseAbandonmentIsUndecidable,
}

impl BadSlotTxgsComparison {
    fn name(&self) -> &'static str {
        match self {
            Self::Equal => "equal",
            Self::Differs { .. } => "differs",
            Self::NotComparedBecauseOfAbandonedRoots => "not_compared_abandoned_roots_on_the_ring",
            Self::NotComparedBecauseAbandonmentIsUndecidable => {
                "not_compared_abandonment_undecidable"
            }
        }
    }
}

/// 停机对拍 S3 一态（第一段与第三段共用）：环里没有被抛弃的根时，装置现算的读坏槽能住的最大 txg 与分配器交回的比。
fn bad_slot_txgs_compared_with_the_allocator(
    reader_before_the_mount: &dyn PoolReader,
    parameters: &MakeFilesystemParameters,
    ring_before_the_mount: &[(RootRingSlot, RingSlotReading)],
    slots_written_by_the_mount: &BTreeSet<RootRingSlot>,
    first_txg: u64,
    allocator_map: &BTreeMap<RootRingSlot, u64>,
) -> BadSlotTxgsComparison {
    match abandoned_roots_on_the_ring_before_the_mount(reader_before_the_mount, parameters) {
        None => BadSlotTxgsComparison::NotComparedBecauseAbandonmentIsUndecidable,
        Some(abandoned) if abandoned > 0 => {
            BadSlotTxgsComparison::NotComparedBecauseOfAbandonedRoots
        }
        Some(_) => {
            let apparatus = expected_bad_slot_txgs(
                ring_before_the_mount,
                slots_written_by_the_mount,
                first_txg,
                slots_per_region_of(parameters),
            );
            if apparatus == *allocator_map {
                BadSlotTxgsComparison::Equal
            } else {
                BadSlotTxgsComparison::Differs { apparatus }
            }
        }
    }
}

/// 第三段的挂载不录制：挂载前后字节变了的根环槽，就是这次挂载写过的槽（每次写根都是新的 txg，字节必变）。
fn ring_slots_changed_between(
    before: &MemoryPool,
    after: &MemoryPool,
    parameters: &MakeFilesystemParameters,
) -> BTreeSet<RootRingSlot> {
    let slot_bytes = usize::try_from(ROOT_SLOT_BYTES).expect("512");
    every_root_ring_slot(&parameters.geometry)
        .into_iter()
        .filter(|ring_slot| {
            let device =
                parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
            let offset = slot_offset(*ring_slot, parameters.geometry.fixed_structure_slot_spacing);
            PoolReader::read(before, device, offset, slot_bytes)
                != PoolReader::read(after, device, offset, slot_bytes)
        })
        .collect()
}

// ============================================================================
// 四、读一张镜像（故障撤掉之后）：根与记录的键、形甲 / 形乙、倒挂、池级 checker、择根与内容、丢没丢已确认的写。
// ============================================================================

/// (实例代号, txg)。
type VersionKey = (u32, u64);

fn key_text(key: Option<VersionKey>) -> String {
    key.map_or_else(
        || "none".to_string(),
        |(instance, txg)| format!("({instance},{txg})"),
    )
}

fn keys_text(keys: &[VersionKey]) -> String {
    keys.iter()
        .map(|key| key_text(Some(*key)))
        .collect::<Vec<_>>()
        .join(",")
}

/// (形甲在不在, 形乙在不在)：形甲 = 同 txg、不同实例；形乙 = 实例代号更高、txg 更低的一条与实例代号低、txg 高的一条同在。
fn forms_in(keys: &[VersionKey]) -> (bool, bool) {
    let mut roots_of_two_instances_at_one_txg = false;
    let mut instance_and_txg_order_crossed_on_roots = false;
    for left in keys {
        for right in keys {
            roots_of_two_instances_at_one_txg |= left.1 == right.1 && left.0 != right.0;
            instance_and_txg_order_crossed_on_roots |= left.0 > right.0 && left.1 < right.1;
        }
    }
    (
        roots_of_two_instances_at_one_txg,
        instance_and_txg_order_crossed_on_roots,
    )
}

/// 倒挂（登记第一节，I-8.14 的可执行形式）：实例代号最大的那个实例 k，环里有一条实例代号小于 k 的根或记录，
/// txg 不小于实例 k 读得出的根与记录里最小的 txg。
fn inverted(keys_of_roots_and_records: &[VersionKey]) -> bool {
    let Some(newest_instance) = keys_of_roots_and_records.iter().map(|key| key.0).max() else {
        return false;
    };
    let lowest_txg_of_the_newest_instance = keys_of_roots_and_records
        .iter()
        .filter(|key| key.0 == newest_instance)
        .map(|key| key.1)
        .min()
        .expect("最大实例至少有一条");
    keys_of_roots_and_records
        .iter()
        .any(|key| key.0 < newest_instance && key.1 >= lowest_txg_of_the_newest_instance)
}

/// `check_pool_image` 判红的不变量名（`Violated` 的那几条；`PartiallyJudged`、`NotApplicable` 不算红）。
fn names_of_violated(verdicts: Vec<(&'static str, InvariantVerdict)>) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = verdicts
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// 读回来的字节是哪一版的内容（序号）；不是任何一版的 `None`。`highest_index` 是这段历史写过的最大序号。
fn content_index_of(bytes: &[u8], highest_index: u64) -> Option<u64> {
    (1..=highest_index).find(|index| file_content_of(*index) == bytes)
}

/// 按这一条根打开、读第一个文件整份：(读出的内容序号, 文字)。
fn content_under(
    image: &MemoryPool,
    root: &RootRecord,
    highest_index: u64,
) -> (Option<u64>, String) {
    let mounted = match open_pool_for_read(image, root) {
        Ok(mounted) => mounted,
        Err(error) => {
            return (
                None,
                format!(
                    "unreadable:open:{}",
                    short_name_of_debug(&format!("{error:?}"))
                ),
            )
        }
    };
    let file = match mounted.open_file(image, FIRST_FILE) {
        Ok(file) => file,
        Err(error) => {
            return (
                None,
                format!("no_file:{}", short_name_of_debug(&format!("{error:?}"))),
            )
        }
    };
    match file.read_at(image, FileOffsetInBytes(0), file.inode_record().size) {
        Ok(output) => match content_index_of(&output.bytes, highest_index) {
            Some(index) => (Some(index), format!("content_{index}")),
            None => (None, "other".to_string()),
        },
        Err(error) => (
            None,
            format!(
                "unreadable:read:{}",
                short_name_of_debug(&format!("{error:?}"))
            ),
        ),
    }
}

/// 这个态之前最后一次已确认的覆盖写的内容序号：`confirmed` 是 (那次发布交回时录制流的长度, 内容序号)，`persisted_stream_end` 是这个态持久到录制流的哪一步。
fn last_confirmed_content(confirmed: &[(usize, u64)], persisted_stream_end: usize) -> Option<u64> {
    confirmed
        .iter()
        .filter(|(stream_end, _)| *stream_end <= persisted_stream_end)
        .map(|(_, index)| *index)
        .max()
}

/// 丢了已确认的写：读出来的内容比最后一次已确认的那一版旧，或读不出。
fn confirmed_write_lost(content_index: Option<u64>, last_confirmed: Option<u64>) -> bool {
    last_confirmed.is_some_and(|confirmed| content_index.is_none_or(|index| index < confirmed))
}

/// 一张镜像（故障撤掉之后）上看到的。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImageReading {
    roots: Vec<VersionKey>,
    records: Vec<VersionKey>,
    roots_of_two_instances_at_one_txg: bool,
    /// 形乙只在根上判（与副本模型同口径，丙六格拿它比）。
    instance_and_txg_order_crossed_on_roots: bool,
    /// 形乙在根与记录合起来的键上判（倒挂对照镜像的「形乙」按它判，见报告「待主 agent 定」一条）。
    instance_and_txg_order_crossed_on_roots_and_records: bool,
    inverted: bool,
    violated: Vec<String>,
    chosen: Option<VersionKey>,
    content_index: Option<u64>,
    content: String,
}

impl ImageReading {
    fn text(&self, last_confirmed: Option<u64>) -> String {
        format!(
            "roots=[{}] records=[{}] roots_of_two_instances_at_one_txg={} instance_and_txg_order_crossed_on_roots={} instance_and_txg_order_crossed_on_roots_and_records={} inverted={} checker_red=[{}] checker_red_count={} choose_root={} content={} confirmed_lost={}",
            keys_text(&self.roots),
            keys_text(&self.records),
            self.roots_of_two_instances_at_one_txg,
            self.instance_and_txg_order_crossed_on_roots,
            self.instance_and_txg_order_crossed_on_roots_and_records,
            self.inverted,
            self.violated.join(","),
            self.violated.len(),
            key_text(self.chosen),
            self.content,
            confirmed_write_lost(self.content_index, last_confirmed)
        )
    }
}

/// 读一张镜像。读不出系统配置时交回 `Err`（登记第十一节「读不到 ≠ 读到 0」：那一态作废、报出）。
fn read_the_image(
    image: &MemoryPool,
    faults_still_applied: &[ReadFault],
    parameters: &MakeFilesystemParameters,
    highest_index: u64,
) -> Result<ImageReading, String> {
    let reader = FaultyImage {
        image,
        faults: faults_still_applied,
    };
    let configuration = choose_system_configuration(&reader).map_err(|failure| {
        format!(
            "system_configuration_unchosen:{}",
            short_name_of_debug(&format!("{failure:?}"))
        )
    })?;
    let mut roots: Vec<VersionKey> = ring_slot_readings(&reader, parameters)
        .into_iter()
        .filter_map(|(_, reading)| match reading {
            RingSlotReading::SelfVerified { instance, txg } => Some((instance, txg)),
            RingSlotReading::BadNotAllZero | RingSlotReading::AllZero => None,
        })
        .collect();
    roots.sort_unstable();
    roots.dedup();
    let mut records: Vec<VersionKey> = scan_journal(&reader, &configuration)
        .iter()
        .map(|((instance, _), record)| (instance.0, record.checkpoint_txg.0))
        .collect();
    records.sort_unstable();
    records.dedup();
    let (roots_of_two_instances_at_one_txg, instance_and_txg_order_crossed_on_roots) =
        forms_in(&roots);
    let mut roots_and_records = roots.clone();
    roots_and_records.extend(records.iter().copied());
    let (_, instance_and_txg_order_crossed_on_roots_and_records) = forms_in(&roots_and_records);
    let chosen_root = choose_root(&reader, &configuration);
    let (content_index, content) = chosen_root
        .as_ref()
        .map_or((None, "no_root".to_string()), |root| {
            content_under(image, root, highest_index)
        });
    Ok(ImageReading {
        inverted: inverted(&roots_and_records),
        violated: names_of_violated(check_pool_image(image))
            .into_iter()
            .map(str::to_string)
            .collect(),
        chosen: chosen_root.map(|root| (root.instance.0, root.checkpoint_txg.0)),
        roots,
        records,
        roots_of_two_instances_at_one_txg,
        instance_and_txg_order_crossed_on_roots,
        instance_and_txg_order_crossed_on_roots_and_records,
        content_index,
        content,
    })
}

/// 之后再做一次不带故障的可写挂载（经同一条臂）：结局与写行那一版读出的内容。
fn mount_after(
    image: &MemoryPool,
    faults_of_the_mount_after: &[ReadFault],
    parameters: &MakeFilesystemParameters,
    arm: Arm,
    highest_index: u64,
) -> (String, Option<u64>, String) {
    let mut devices = devices_of(image, faults_of_the_mount_after);
    match mount_under_arm(parameters, &mut devices, arm) {
        Ok(mounted) => {
            let row_root = *mounted.output.row_publish.root();
            let after = image_of_plain_devices(&devices);
            let (index, content) = content_under(&after, &row_root, highest_index);
            (
                format!(
                    "ok:chosen=({},{}),row=({},{})",
                    mounted.output.chosen_root.instance.0,
                    mounted.output.chosen_root.checkpoint_txg.0,
                    row_root.instance.0,
                    row_root.checkpoint_txg.0
                ),
                index,
                content,
            )
        }
        Err(error) => (
            format!("refused:{}", refusal_name(&error)),
            None,
            "not_mounted".to_string(),
        ),
    }
}

// ============================================================================
// 五、起点历史与六格（登记第一节「六格」；造法照副本模型 `build_starting_history`）。
// ============================================================================

/// 起点历史：实例 1 到 D 为止的镜像，各版的记录计数器，与造倒挂对照镜像要的 B 那条根。
struct StartingHistory {
    parameters: MakeFilesystemParameters,
    image: MemoryPool,
    /// (txg, 那次发布全部记录的计数器)，txg 3 起。
    record_counters_by_txg: Vec<(u64, Vec<u64>)>,
    /// 倒挂对照镜像（登记第五节「丙」那条阳性对照）：起点镜像上实例 2 取号，接在 B（txg 4）那一版上覆盖写两次（txg 5、6）盖掉 C、D 的根槽，
    /// 实例 1 txg 6 那次发布的记录仍在环里。交回 (镜像, 写出的两条根的键)；写不成交回原因。
    inverted_control: Result<(MemoryPool, Vec<VersionKey>), String>,
}

fn record_counters_of(output: &TransactionOutput) -> Vec<u64> {
    output
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain(std::iter::once(output.record.counter))
        .collect()
}

fn memory_pool_of_sparse(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// mkfs 两块盘 → 实例 1 取号、暖机 txg 1、2 → 新池新建文件 txg 3 → 覆盖写 B txg 4、C txg 5、D txg 6；
/// D 的根槽 FUA 持久、两块盘的系统配置轮换没持久（两个系统配置槽退回 D 之前的字节）。
fn build_starting_history(slots_per_region: u64) -> StartingHistory {
    let parameters = pool_parameters(slots_per_region);
    let mut devices = empty_sparse_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
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
        &parameters,
        &genesis,
    ));
    let mut publishes: Vec<TransactionOutput> = Vec::new();
    let mut before_third_overwrite = None;
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1), "起点历史是实例 1");
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
    for index in 2..=4u64 {
        if index == 4 {
            before_third_overwrite = Some(memory_pool_of_sparse(&devices));
        }
        let content = file_content_of(index);
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
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
    let txgs: Vec<u64> = publishes
        .iter()
        .map(|publish| publish.root.checkpoint_txg.0)
        .collect();
    assert_eq!(
        txgs,
        vec![3, 4, TXG_OF_SECOND_OVERWRITE, TXG_OF_THIRD_OVERWRITE],
        "新池新建文件 txg 3，B、C、D 是 txg 4、5、6"
    );
    let before_third_overwrite = before_third_overwrite.expect("D 之前拍过镜像");
    let mut image = memory_pool_of_sparse(&devices);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, u64::from(FIXED_STRUCTURE_SLOT_SPACING)] {
            let bytes = PoolReader::read(
                &before_third_overwrite,
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
    let inverted_control = {
        let mut control_devices = devices_of(&image, &[]);
        let mut control_allocator = allocator.clone();
        let mut writer = PoolWriter::new(&parameters, control_devices.as_mut_slice());
        let written = acquire_instance(&mut writer)
            .map_err(|failure| format!("acquire:{}", short_name_of_debug(&format!("{failure:?}"))))
            .and_then(|instance| {
                let mut previous = publishes[1].clone();
                let mut keys = Vec::new();
                for index in [5u64, 6] {
                    let content = file_content_of(index);
                    let next = publish_overwrite(
                        &mut writer,
                        &mut control_allocator,
                        &previous,
                        FirstFile {
                            content: &content,
                            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 60 * (index - 1),
                        },
                        instance,
                    )
                    .map_err(|failure| {
                        format!("publish:{}", short_name_of_debug(&format!("{failure:?}")))
                    })?;
                    keys.push((next.root.instance.0, next.root.checkpoint_txg.0));
                    previous = next;
                }
                Ok(keys)
            });
        drop(writer);
        written.map(|keys| (image_of_plain_devices(&control_devices), keys))
    };
    StartingHistory {
        inverted_control,
        record_counters_by_txg: publishes
            .iter()
            .map(|publish| (publish.root.checkpoint_txg.0, record_counters_of(publish)))
            .collect(),
        parameters,
        image,
    }
}

/// 第一段的起点历史与第二段的撕裂轮换族历史各建哪几组 S（第一段四组几何与全部对照都落在这三组上）。
const SLOTS_PER_REGION_OF_THE_HISTORIES: [u64; 3] = [4, 8, 16];

/// 第一段三组 S 的起点历史：判到哪一组才建（全部格都从判定库复用的那一趟一份都不建）；几个线程同时要同一组时只建一次。
struct StartingHistories {
    by_slots_per_region: BTreeMap<u64, std::sync::OnceLock<StartingHistory>>,
}

fn starting_histories() -> StartingHistories {
    StartingHistories {
        by_slots_per_region: SLOTS_PER_REGION_OF_THE_HISTORIES
            .iter()
            .map(|slots_per_region| (*slots_per_region, std::sync::OnceLock::new()))
            .collect(),
    }
}

/// 这一组 S 的起点历史，第一次要时现建（`build_starting_history` 因此在第一段两个格入口的单元闭包里）。
fn starting_history_of(histories: &StartingHistories, slots_per_region: u64) -> &StartingHistory {
    histories
        .by_slots_per_region
        .get(&slots_per_region)
        .expect("第一段的格与对照只落在 SLOTS_PER_REGION_OF_THE_HISTORIES 那三组 S 上")
        .get_or_init(|| build_starting_history(slots_per_region))
}

/// txg 那次发布的根按落点公式住的那一槽：(盘, 偏移)。
fn root_slot_place(parameters: &MakeFilesystemParameters, txg: u64) -> (DeviceIdentity, u64) {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    (
        device,
        slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0,
    )
}

/// 每块盘两槽里自证过、世代号最大的那一槽的偏移。
fn newest_system_configuration_slot(image: &MemoryPool, device: DeviceIdentity) -> u64 {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
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

/// 第一段的几组几何（登记第八节）：G 主点、G 小环、G 大环、G 藏一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellGeometry {
    name: &'static str,
    slots_per_region: u64,
    roots_hidden: u64,
}

const CELL_GEOMETRIES: [CellGeometry; 4] = [
    CellGeometry {
        name: "main_point",
        slots_per_region: 8,
        roots_hidden: 2,
    },
    CellGeometry {
        name: "small_ring",
        slots_per_region: 4,
        roots_hidden: 2,
    },
    CellGeometry {
        name: "large_ring",
        slots_per_region: 16,
        roots_hidden: 2,
    },
    CellGeometry {
        name: "hide_one",
        slots_per_region: 8,
        roots_hidden: 1,
    },
];

/// 一格：根与记录的读坏种类 × 系统配置最新槽的读坏种类（`None` 是对照格：系统配置槽不注坏）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FaultCell {
    root_fault: ReadFaultKind,
    system_configuration_fault: Option<ReadFaultKind>,
}

impl FaultCell {
    fn label(self) -> String {
        format!(
            "root_and_record_fault={} newest_system_configuration_fault={}",
            self.root_fault.name(),
            self.system_configuration_fault
                .map_or("none", ReadFaultKind::name)
        )
    }
}

/// 六格（根三种 × 系统配置两种），次序照副本模型最小历史那条用例。
fn six_cells() -> Vec<FaultCell> {
    let mut cells = Vec::new();
    for root_fault in [
        ReadFaultKind::DeviceError,
        ReadFaultKind::NotSelfVerified,
        ReadFaultKind::AllZero,
    ] {
        for system_configuration_fault in [ReadFaultKind::AllZero, ReadFaultKind::NotSelfVerified] {
            cells.push(FaultCell {
                root_fault,
                system_configuration_fault: Some(system_configuration_fault),
            });
        }
    }
    cells
}

/// 对照格（第五节「系统配置槽不注坏那一格，产品挂载必须拒」）：根与记录取设备报读错（副本模型 `cell_mount` 那一行同格）。
const CONTROL_CELL: FaultCell = FaultCell {
    root_fault: ReadFaultKind::DeviceError,
    system_configuration_fault: None,
};

fn fault_ranges_of(start: &StartingHistory, roots_hidden: u64, cell: FaultCell) -> Vec<ReadFault> {
    let mut faults = Vec::new();
    for txg in (0..roots_hidden).map(|back| TXG_OF_THIRD_OVERWRITE - back) {
        let (root_device, root_offset) = root_slot_place(&start.parameters, txg);
        faults.push(ReadFault {
            device: root_device,
            offset: root_offset,
            length: ROOT_SLOT_BYTES,
            kind: cell.root_fault,
        });
        let counters = &start
            .record_counters_by_txg
            .iter()
            .find(|(record_txg, _)| *record_txg == txg)
            .expect("起点历史里有这一版")
            .1;
        for counter in counters {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                faults.push(ReadFault {
                    device,
                    offset: record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES_LOCAL).0,
                    length: JOURNAL_RECORD_BYTES_LOCAL,
                    kind: cell.root_fault,
                });
            }
        }
    }
    if let Some(kind) = cell.system_configuration_fault {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            faults.push(ReadFault {
                device,
                offset: newest_system_configuration_slot(&start.image, device),
                length: SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
                kind,
            });
        }
    }
    faults
}

/// 一次带读故障、带录制的可写挂载：结局、挂载输出里要的几样、写表与每条写在录制流里的下标。
struct RecordedMount {
    outcome: String,
    instance: Option<u32>,
    row_txg: Option<u64>,
    effective_txg: Option<u64>,
    branch_name: Option<&'static str>,
    bad_slot_txgs_of_the_allocator: Option<BTreeMap<RootRingSlot, u64>>,
    writes: Vec<RetainedWrite>,
    stream_indexes: Vec<usize>,
    /// 挂载整串写完的镜像（故障撤掉）。
    image_after: MemoryPool,
}

fn recorded_mount(
    image: &MemoryPool,
    faults: &[ReadFault],
    parameters: &MakeFilesystemParameters,
    arm: Arm,
) -> RecordedMount {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_of(image, faults, &stream);
    let result = mount_under_arm(parameters, &mut devices, arm);
    let (outcome, instance, row_txg, effective_txg, branch_name, bad_slot_txgs_of_the_allocator) =
        match &result {
            Ok(mounted) => (
                format!(
                    "ok instance={} chosen=({},{}) effective=({},{}) row_txg={}",
                    mounted.output.instance.0,
                    mounted.output.chosen_root.instance.0,
                    mounted.output.chosen_root.checkpoint_txg.0,
                    mounted.output.effective_root.instance.0,
                    mounted.output.effective_root.checkpoint_txg.0,
                    mounted.output.row_publish.root().checkpoint_txg.0
                ),
                Some(mounted.output.instance.0),
                Some(mounted.output.row_publish.root().checkpoint_txg.0),
                Some(mounted.output.effective_root.checkpoint_txg.0),
                Some(mounted.output.first_txg_of_the_new_instance_branch),
                Some(
                    ring_slots_bad_or_reading_back_all_zero_at_mount_by(&mounted.allocator)
                        .into_iter()
                        .map(|(slot, txg)| (slot, txg.0))
                        .collect(),
                ),
            ),
            Err(error) => (
                format!("refused:{}", refusal_name(error)),
                None,
                None,
                None,
                None,
                None,
            ),
        };
    let image_after = image_of_recorded_devices(&devices);
    let operations: Vec<RetainedOperation> = stream.retained_operations();
    let (writes, _segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations,
        &stream.entry_spans(),
        &fixed_geometry(parameters),
    );
    RecordedMount {
        outcome,
        instance,
        row_txg,
        effective_txg,
        branch_name,
        bad_slot_txgs_of_the_allocator,
        writes,
        stream_indexes,
        image_after,
    }
}

/// 固定崩溃点（登记第一节）：写行那次发布的写全部持久、崩在实例 2 第二次发布（txg = 写行 txg + 1）的根槽写之前。
/// 交回那一刻的镜像与持久了几条写；挂载被拒或这次挂载没写那条根时 `None`。
fn image_at_the_fixed_crash_point(
    base: &MemoryPool,
    mount: &RecordedMount,
) -> Option<(MemoryPool, usize)> {
    let (instance, row_txg) = (mount.instance?, mount.row_txg?);
    let crash_write = mount.writes.iter().position(|write| {
        write.kind == StepKind::RootRecordFua && {
            let (written_instance, txg) = root_identity_of_write(write);
            written_instance.0 == instance && txg.0 == row_txg + 1
        }
    })?;
    let crash_stream_index = mount.stream_indexes[crash_write];
    let mut image = base.clone();
    let mut persisted = 0;
    for (write, stream_index) in mount.writes.iter().zip(&mount.stream_indexes) {
        if *stream_index < crash_stream_index {
            image.apply_writes(std::slice::from_ref(write));
            persisted += 1;
        }
    }
    Some((image, persisted))
}

/// 这次挂载写过的根环槽（写行、暖机写的根）。
fn ring_slots_written_by(mount: &RecordedMount, slots_per_region: u64) -> BTreeSet<RootRingSlot> {
    mount
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| ring_slot_of_txg(root_identity_of_write(write).1 .0, slots_per_region))
        .collect()
}

// ============================================================================
// 六、第一段：一格一条臂一件（登记第十四节「独立单位」），件里两个崩溃点各一行。
// ============================================================================

/// 六格历史写过的最大内容序号（D = 4）。
const HIGHEST_CONTENT_INDEX_OF_THE_START: u64 = 4;
/// 倒挂对照镜像上实例 2 写的两版内容序号是 5、6。
const HIGHEST_CONTENT_INDEX_OF_THE_CONTROL: u64 = 6;

/// 六格里已确认的覆盖写（新池新建文件、B、C 都在挂载之前交回，D 没确认）：(交回时录制流的长度, 内容序号)。
const CONFIRMED_BEFORE_THE_SIX_CELL_MOUNT: [(usize, u64); 3] =
    [(0, 1), (0, 2), (0, CONTENT_INDEX_OF_SECOND_OVERWRITE)];

#[derive(Clone, Copy, Debug)]
struct CellJob {
    geometry: CellGeometry,
    cell: FaultCell,
    arm: Arm,
}

/// 一件判出来的（判定行要的几样；产物行另在 `line`）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FaultCellJudgement {
    lines: Vec<String>,
    refusal: Option<String>,
    row_txg: Option<u64>,
    yi_refuses: Vec<(YiReading, bool)>,
    fixed: Option<ImageReading>,
    fixed_lost: bool,
    fixed_mount_after_refused: bool,
    stop_mismatches: Vec<String>,
    voided: Option<String>,
}

fn cell_jobs() -> Vec<CellJob> {
    let mut jobs = Vec::new();
    for geometry in CELL_GEOMETRIES {
        let mut cells = six_cells();
        cells.push(CONTROL_CELL);
        for cell in cells {
            for arm in Arm::MOUNTED_ON_THE_SIX_CELLS {
                if cell.system_configuration_fault.is_some()
                    || matches!(arm, Arm::Today | Arm::TodayViaSwitch)
                {
                    jobs.push(CellJob {
                        geometry,
                        cell,
                        arm,
                    });
                }
            }
        }
    }
    jobs
}

/// S9 的算术推法（第一份登记）：甲给的 f 不大于实例 1 读得出的最大 txg 就记倒挂留着；假想键表 = 起点根环加 (2, f)，按 (txg, 实例) 择根，
/// 择到实例 2 那条读出的是这次挂载接着写的那一版（effective），否则读出择到那一版；比 C 旧就记丢。
fn model_of_the_fixed_crash_point(
    start: &StartingHistory,
    first_txg: u64,
    effective_txg: u64,
) -> (bool, bool) {
    let model_inverted = first_txg <= TXG_OF_THIRD_OVERWRITE;
    let slots_per_region = slots_per_region_of(&start.parameters);
    let mut keys: BTreeMap<RootRingSlot, VersionKey> = ring_slot_readings(
        &FaultyImage {
            image: &start.image,
            faults: &[],
        },
        &start.parameters,
    )
    .into_iter()
    .filter_map(|(slot, reading)| match reading {
        RingSlotReading::SelfVerified { instance, txg } => Some((slot, (instance, txg))),
        RingSlotReading::BadNotAllZero | RingSlotReading::AllZero => None,
    })
    .collect();
    keys.insert(
        ring_slot_of_txg(first_txg, slots_per_region),
        (2, first_txg),
    );
    let chosen = keys
        .values()
        .copied()
        .max_by_key(|(instance, txg)| (*txg, *instance))
        .expect("键表非空");
    let version_txg = if chosen.0 == 2 {
        effective_txg
    } else {
        chosen.1
    };
    // 起点历史里 txg t 那一版的内容序号是 t − 2（txg 3 是新池新建文件）。
    let model_lost = version_txg.saturating_sub(2) < CONTENT_INDEX_OF_SECOND_OVERWRITE;
    (model_inverted, model_lost)
}

fn judge_a_cell_job(start: &StartingHistory, job: CellJob) -> FaultCellJudgement {
    let parameters = &start.parameters;
    let slots_per_region = slots_per_region_of(parameters);
    let faults = fault_ranges_of(start, job.geometry.roots_hidden, job.cell);
    let before = readings_before_the_mount(&start.image, &faults, parameters);
    let mount = recorded_mount(&start.image, &faults, parameters, job.arm);
    let label = format!(
        "geometry={} {} arm={}",
        job.geometry.name,
        job.cell.label(),
        job.arm.name()
    );
    let mut result = FaultCellJudgement {
        row_txg: mount.row_txg,
        ..FaultCellJudgement::default()
    };
    result.refusal = mount.outcome.strip_prefix("refused:").map(str::to_string);
    result.yi_refuses = YiReading::EVERY
        .iter()
        .map(|yi| (*yi, yi_refuses(*yi, &before.system_configuration)))
        .collect();
    let apparatus_first_txg = before.first_txg_of(job.arm);
    // 停机对拍 S1（丙）、S6（开关挂载）：做成的态上写行 txg 等于装置现算的那一档。
    if let Some(row_txg) = mount.row_txg {
        if row_txg != apparatus_first_txg {
            result.stop_mismatches.push(format!(
                "S1_or_S6 {label} row_txg={row_txg} apparatus={apparatus_first_txg}"
            ));
        }
    }
    // S2：装置现算 R2 为真的态，产品挂载必须拒。
    let undecidable = witness_undecidable_today(&before.system_configuration);
    if undecidable && mount.row_txg.is_some() {
        result.stop_mismatches.push(format!(
            "S2 {label} system_configuration_witness_undecidable=true mount={}",
            mount.outcome
        ));
    }
    // S3：挂载时读坏或读回全零的槽能住的最大 txg；只在环里没有被抛弃根的态上比（登记第十一节 S3）。
    let bad_slot_txgs_against_the_allocator =
        match (&mount.bad_slot_txgs_of_the_allocator, mount.row_txg) {
            (Some(allocator_map), Some(row_txg)) => {
                let comparison = bad_slot_txgs_compared_with_the_allocator(
                    &FaultyImage {
                        image: &start.image,
                        faults: &faults,
                    },
                    parameters,
                    &before.ring,
                    &ring_slots_written_by(&mount, slots_per_region),
                    row_txg,
                    allocator_map,
                );
                if let BadSlotTxgsComparison::Differs { apparatus } = &comparison {
                    result.stop_mismatches.push(format!(
                        "S3 {label} apparatus={apparatus:?} allocator={allocator_map:?}"
                    ));
                }
                comparison.name()
            }
            _ => "not_mounted",
        };
    // S5：重读一次读回同样的系统配置读数。
    let reread = system_configuration_slot_readings(&FaultyImage {
        image: &start.image,
        faults: &faults,
    });
    if reread != before.system_configuration {
        result
            .stop_mismatches
            .push(format!("S5 {label} reread_differs"));
    }
    let head = format!(
        "{label} mount={} row_txg={} apparatus_first_txg={apparatus_first_txg} apparatus_highest_record_txg={} branch={} system_configuration_witness_undecidable={undecidable} system_configuration=[{}] refuse_directly={} into_undecidable={} bad_slot_txgs_against_the_allocator={bad_slot_txgs_against_the_allocator}",
        mount.outcome,
        mount.row_txg.map_or("none".to_string(), |txg| txg.to_string()),
        before.highest_record_txg,
        mount.branch_name.unwrap_or("not_mounted"),
        system_configuration_readings_text(&before.system_configuration),
        yi_refuses(YiReading::RefuseDirectly, &before.system_configuration),
        yi_refuses(YiReading::IntoUndecidable, &before.system_configuration),
    );
    let last_confirmed = last_confirmed_content(&CONFIRMED_BEFORE_THE_SIX_CELL_MOUNT, 0);
    let fixed = image_at_the_fixed_crash_point(&start.image, &mount);
    let points: Vec<(&str, MemoryPool, String)> = match fixed {
        Some((image, persisted)) => vec![
            (
                "fixed",
                image,
                format!("persisted_writes={persisted}/{}", mount.writes.len()),
            ),
            (
                "whole",
                mount.image_after.clone(),
                format!("persisted_writes={0}/{0}", mount.writes.len()),
            ),
        ],
        None => vec![(
            "absent",
            mount.image_after.clone(),
            format!("persisted_writes={0}/{0}", mount.writes.len()),
        )],
    };
    for (point, image, persisted_text) in points {
        let reading =
            match read_the_image(&image, &[], parameters, HIGHEST_CONTENT_INDEX_OF_THE_START) {
                Ok(reading) => reading,
                Err(reason) => {
                    result.voided = Some(format!("{label} crash_point={point} {reason}"));
                    result.lines.push(format!(
                        "name=cell {head} crash_point={point} {persisted_text} voided={reason}"
                    ));
                    continue;
                }
            };
        let (after, after_index, after_content) = mount_after(
            &image,
            &[],
            parameters,
            job.arm,
            HIGHEST_CONTENT_INDEX_OF_THE_START,
        );
        let mut model = String::new();
        if point == "fixed" {
            result.fixed_lost = confirmed_write_lost(reading.content_index, last_confirmed);
            result.fixed_mount_after_refused = after.starts_with("refused:");
            if let (Some(row_txg), Some(effective_txg), true) = (
                mount.row_txg,
                mount.effective_txg,
                Arm::JIA.contains(&job.arm),
            ) {
                let (model_inverted, model_lost) =
                    model_of_the_fixed_crash_point(start, row_txg, effective_txg);
                let agrees = model_inverted == reading.inverted && model_lost == result.fixed_lost;
                if !agrees {
                    result.stop_mismatches.push(format!(
                        "S9 {label} real_inverted={} model_inverted={model_inverted} real_lost={} model_lost={model_lost}",
                        reading.inverted, result.fixed_lost
                    ));
                }
                model = format!(
                    " model_inverted={model_inverted} model_lost={model_lost} fixed_crash_point_model_against_the_written_image={}",
                    if agrees { "equal" } else { "differs" }
                );
            }
            result.fixed = Some(reading.clone());
        }
        result.lines.push(format!(
            "name=cell {head} crash_point={point} {persisted_text} {} mount_after={after} mount_after_content={after_content} mount_after_confirmed_lost={}{model}",
            reading.text(last_confirmed),
            confirmed_write_lost(after_index, last_confirmed)
        ));
    }
    if job.arm == Arm::TodayViaSwitch {
        // S7：同一盘面、同一读故障，经开关今天那一档与产品入口结局与盘面逐字节相同。
        let product = recorded_mount(&start.image, &faults, parameters, Arm::Today);
        let same_outcome = product.outcome == mount.outcome;
        let same_image = product.image_after == mount.image_after;
        let same_fixed = image_at_the_fixed_crash_point(&start.image, &product)
            .map(|(image, _)| image)
            == image_at_the_fixed_crash_point(&start.image, &mount).map(|(image, _)| image);
        if !(same_outcome && same_image && same_fixed) {
            result.stop_mismatches.push(format!("S7 {label} same_outcome={same_outcome} same_image={same_image} same_fixed={same_fixed}"));
        }
        result.lines.push(format!("name=s7 {label} same_outcome={same_outcome} same_whole_image={same_image} same_fixed_image={same_fixed}"));
    }
    result
}

// ============================================================================
// 七、第一段的阳性对照与负对照（登记第五节；答案写死在第七节）。一件一行，判过没过另打 `verdict` 字段。
// ============================================================================

/// 第七节「独立算出的」甲锚点：(S, 盘面, 今天, 甲只并读坏, 甲并全零)。盘面：起点镜像不注故障、起点镜像只坏 txg 5 那一槽（设备报读错）。
const START_IMAGE_ANCHORS: [(u64, &str, u64, u64, u64); 6] = [
    (4, "start_no_fault", 7, 7, 12),
    (8, "start_no_fault", 7, 7, 24),
    (16, "start_no_fault", 7, 7, 48),
    (4, "start_slot_of_txg_5_device_error", 7, 18, 18),
    (8, "start_slot_of_txg_5_device_error", 7, 30, 30),
    (16, "start_slot_of_txg_5_device_error", 7, 54, 54),
];

/// 第七节：六格上各臂的写行 txg（G 主点、G 小环、G 大环藏两条；G 藏一条）：(几何名, 根与记录读坏种类, 今天, 甲只并读坏, 甲并全零)。
fn six_cell_anchor(geometry: CellGeometry, root_fault: ReadFaultKind) -> (u64, u64, u64) {
    let root_ring_length_in_slots = ROOT_RING_REGIONS_LOCAL * geometry.slots_per_region;
    match (geometry.roots_hidden, root_fault) {
        (2, ReadFaultKind::AllZero) => (5, 5, root_ring_length_in_slots),
        (2, _) => (5, 7, root_ring_length_in_slots),
        (_, ReadFaultKind::AllZero) => (6, 6, root_ring_length_in_slots),
        (_, _) => (6, 7, root_ring_length_in_slots),
    }
}

fn expected_row_txg_of(arm: Arm, anchors: (u64, u64, u64)) -> u64 {
    match arm {
        Arm::Today | Arm::TodayViaSwitch => anchors.0,
        Arm::PastReadBad => anchors.1,
        Arm::PastReadBadOrAllZero => anchors.2,
    }
}

#[derive(Clone, Copy, Debug)]
enum ControlJob {
    /// 负对照：起点历史不注读故障，产品挂载做成、挂载写完之后池级 checker 0 条红。
    Negative(CellGeometry),
    /// 倒挂对照镜像：实例 2 接在 B 上写两代根盖掉 C、D 的根槽，实例 1 的 txg 5、6 记录仍在。
    InvertedImage(CellGeometry),
    /// 甲的手给读数（起点镜像那两种）：一条臂一件。
    StartImageAnchor(usize, Arm),
    /// mkfs 之后第一次可写挂载：S 与臂。
    RightAfterMakeFilesystem(u64, Arm),
    /// 乙的手造盘面：S 与盘面号（盘 0 一槽撕裂、盘 1 两槽好 / 两块盘各一槽撕裂 / 四槽都好）。
    YiHandMadeImage(u64, usize),
}

fn control_jobs() -> Vec<ControlJob> {
    let mut jobs: Vec<ControlJob> = CELL_GEOMETRIES
        .iter()
        .map(|geometry| ControlJob::Negative(*geometry))
        .collect();
    jobs.extend(
        CELL_GEOMETRIES
            .iter()
            .map(|geometry| ControlJob::InvertedImage(*geometry)),
    );
    for index in 0..START_IMAGE_ANCHORS.len() {
        for arm in [Arm::Today, Arm::PastReadBad, Arm::PastReadBadOrAllZero] {
            jobs.push(ControlJob::StartImageAnchor(index, arm));
        }
    }
    for slots_per_region in [4, 8, 16] {
        for arm in [
            Arm::TodayViaSwitch,
            Arm::PastReadBad,
            Arm::PastReadBadOrAllZero,
        ] {
            jobs.push(ControlJob::RightAfterMakeFilesystem(slots_per_region, arm));
        }
    }
    for slots_per_region in [4, 8, 16] {
        for image in 0..3 {
            jobs.push(ControlJob::YiHandMadeImage(slots_per_region, image));
        }
    }
    jobs
}

/// 乙三张手造盘面写死的答案（第五节）：乙直拒「拒、拒、不拒」，乙入判不出「不拒、拒、不拒」。
const YI_HAND_MADE_ANSWERS: [(bool, bool); 3] = [(true, false), (true, true), (false, false)];

/// 翻一个系统配置槽里的字节（不经写者，读得出而自证不过）。
fn corrupt_the_newest_system_configuration_slot(image: &mut MemoryPool, device: DeviceIdentity) {
    let offset = newest_system_configuration_slot(image, device);
    image.flip_byte(
        device,
        DeviceOffsetInBytes(offset),
        FLIPPED_BYTE_INSIDE_A_RANGE,
    );
}

fn verdict_text(passed: bool) -> &'static str {
    if passed {
        "as_expected"
    } else {
        "CONTROL_FAILED"
    }
}

/// 一件对照：(产物行, 过没过)。`histories` 是三组 S 的起点历史（判到哪一组才建）。
fn judge_a_control_job(histories: &StartingHistories, job: ControlJob) -> (String, bool) {
    match job {
        ControlJob::Negative(geometry) => {
            let start = starting_history_of(histories, geometry.slots_per_region);
            let mount = recorded_mount(&start.image, &[], &start.parameters, Arm::Today);
            let reading = read_the_image(
                &mount.image_after,
                &[],
                &start.parameters,
                HIGHEST_CONTENT_INDEX_OF_THE_START,
            );
            let red = reading
                .as_ref()
                .map_or(usize::MAX, |reading| reading.violated.len());
            let passed = mount.row_txg.is_some() && red == 0;
            (
                format!(
                    "name=negative_control geometry={} mount={} checker_red=[{}] control={}",
                    geometry.name,
                    mount.outcome,
                    reading.map_or_else(|reason| reason, |reading| reading.violated.join(",")),
                    verdict_text(passed)
                ),
                passed,
            )
        }
        ControlJob::InvertedImage(geometry) => {
            let start = starting_history_of(histories, geometry.slots_per_region);
            let (written, reading) = match &start.inverted_control {
                Ok((image, keys)) => (
                    Some(keys.clone()),
                    read_the_image(
                        image,
                        &[],
                        &start.parameters,
                        HIGHEST_CONTENT_INDEX_OF_THE_CONTROL,
                    ),
                ),
                Err(reason) => (None, Err(reason.clone())),
            };
            let passed = written.as_deref()
                == Some(&[(2, TXG_OF_SECOND_OVERWRITE), (2, TXG_OF_THIRD_OVERWRITE)][..])
                && reading.as_ref().is_ok_and(|reading| {
                    reading.violated.iter().any(|name| name == "I-8.14")
                        && reading.inverted
                        && reading.instance_and_txg_order_crossed_on_roots_and_records
                });
            (
                format!(
                    "name=inverted_image_control geometry={} written_roots=[{}] {} control={}",
                    geometry.name,
                    written
                        .as_deref()
                        .map_or("not_written".to_string(), keys_text),
                    reading.map_or_else(
                        |reason| reason,
                        |reading| reading.text(Some(CONTENT_INDEX_OF_SECOND_OVERWRITE))
                    ),
                    verdict_text(passed)
                ),
                passed,
            )
        }
        ControlJob::StartImageAnchor(index, arm) => {
            let (slots_per_region, image_name, today, past_read_bad, past_read_bad_or_all_zero) =
                START_IMAGE_ANCHORS[index];
            let start = starting_history_of(histories, slots_per_region);
            let faults = if image_name == "start_no_fault" {
                Vec::new()
            } else {
                let (device, offset) = root_slot_place(&start.parameters, TXG_OF_SECOND_OVERWRITE);
                vec![ReadFault {
                    device,
                    offset,
                    length: ROOT_SLOT_BYTES,
                    kind: ReadFaultKind::DeviceError,
                }]
            };
            let expected =
                expected_row_txg_of(arm, (today, past_read_bad, past_read_bad_or_all_zero));
            let before = readings_before_the_mount(&start.image, &faults, &start.parameters);
            let mount = recorded_mount(&start.image, &faults, &start.parameters, arm);
            let passed = mount.row_txg == Some(expected) && before.first_txg_of(arm) == expected;
            (
                format!(
                    "name=jia_anchor slots_per_region={slots_per_region} image={image_name} arm={} expected_row_txg={expected} row_txg={} apparatus_first_txg={} mount={} control={}",
                    arm.name(),
                    mount.row_txg.map_or("none".to_string(), |txg| txg.to_string()),
                    before.first_txg_of(arm),
                    mount.outcome,
                    verdict_text(passed)
                ),
                passed,
            )
        }
        ControlJob::RightAfterMakeFilesystem(slots_per_region, arm) => {
            let parameters = pool_parameters(slots_per_region);
            let image = image_right_after_make_filesystem(&parameters);
            let genesis_ok = genesis_slots_read_on(&image, &parameters) == expected_genesis_slots();
            let today = recorded_mount(&image, &[], &parameters, Arm::Today);
            let mount = recorded_mount(&image, &[], &parameters, arm);
            let before = readings_before_the_mount(&image, &[], &parameters);
            let expected =
                expected_row_txg_of(arm, (1, 1, ROOT_RING_REGIONS_LOCAL * slots_per_region));
            let same_bytes_as_today = mount.image_after == today.image_after;
            let bytes_ok = match arm {
                Arm::PastReadBadOrAllZero => !same_bytes_as_today,
                Arm::Today | Arm::TodayViaSwitch | Arm::PastReadBad => same_bytes_as_today,
            };
            let passed = genesis_ok
                && mount.row_txg == Some(expected)
                && before.first_txg_of(arm) == expected
                && bytes_ok;
            (
                format!(
                    "name=right_after_make_filesystem slots_per_region={slots_per_region} arm={} expected_row_txg={expected} row_txg={} apparatus_first_txg={} same_bytes_as_today={same_bytes_as_today} genesis_slots_ok={genesis_ok} control={}",
                    arm.name(),
                    mount.row_txg.map_or("none".to_string(), |txg| txg.to_string()),
                    before.first_txg_of(arm),
                    verdict_text(passed)
                ),
                passed,
            )
        }
        ControlJob::YiHandMadeImage(slots_per_region, image_number) => {
            let start = starting_history_of(histories, slots_per_region);
            let mut image = start.image.clone();
            if image_number <= 1 {
                corrupt_the_newest_system_configuration_slot(&mut image, DeviceIdentity(0));
            }
            if image_number == 1 {
                corrupt_the_newest_system_configuration_slot(&mut image, DeviceIdentity(1));
            }
            let readings = system_configuration_slot_readings(&FaultyImage {
                image: &image,
                faults: &[],
            });
            let (refuse_directly, into_undecidable) = YI_HAND_MADE_ANSWERS[image_number];
            let mut devices = devices_of(&image, &[]);
            let today = mount_writable(&start.parameters, &mut devices).map_or_else(
                |error| format!("refused:{}", refusal_name(&error)),
                |_| "ok".to_string(),
            );
            let passed = yi_refuses(YiReading::RefuseDirectly, &readings) == refuse_directly
                && yi_refuses(YiReading::IntoUndecidable, &readings) == into_undecidable;
            (
                format!(
                    "name=yi_hand_made_image slots_per_region={slots_per_region} image={image_number} system_configuration=[{}] refuse_directly={} into_undecidable={} expected_refuse_directly={refuse_directly} expected_into_undecidable={into_undecidable} today_mount={today} control={}",
                    system_configuration_readings_text(&readings),
                    yi_refuses(YiReading::RefuseDirectly, &readings),
                    yi_refuses(YiReading::IntoUndecidable, &readings),
                    verdict_text(passed)
                ),
                passed,
            )
        }
    }
}

// ============================================================================
// 八、第一段的判定行（登记第六节第一段那张表，一个量一行；判定只在 G 主点上写死，另三组各记翻不翻）。
// ============================================================================

/// 副本模型那一行（第四节，读回全零 × 读回全零格）里入库装置也报的字段：逐字段比。
const MODEL_MOUNT_OF_THE_ALL_ZERO_CELL: &str =
    "ok instance=2 chosen=(1,4) effective=(1,4) row_txg=5";
const MODEL_PERSISTED_WRITES_OF_THE_ALL_ZERO_CELL: &str = "persisted_writes=45/69";
const MODEL_ROOTS_OF_THE_ALL_ZERO_CELL: [VersionKey; 7] =
    [(0, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 6), (2, 5)];
const MODEL_CHOSEN_OF_THE_ALL_ZERO_CELL: VersionKey = (1, 6);
const MODEL_CONTENT_OF_THE_ALL_ZERO_CELL: u64 = 4;
const MODEL_CHECKER_RED_OF_THE_ALL_ZERO_CELL: [&str; 6] =
    ["I-3.1", "I-3.21", "I-3.5", "I-3.9", "I-4.2", "I-8.14"];
const MODEL_MOUNT_AFTER_OF_THE_ALL_ZERO_CELL: &str =
    "refused:NewerStateStillUnreadableAfterOneReread";

/// 判别力自证（登记第八节）：门槛 ≥ t 挪到观测值加一必须由「是」转「否」，观测 0 挪到 ≥ 0 必须由「否」转「是」。
fn discrimination_line(quantity: &str, observed: usize, threshold: usize) -> String {
    let at = observed >= threshold;
    let moved = if observed == 0 { 0 } else { observed + 1 };
    let at_moved = observed >= moved;
    format!(
        "name=discrimination quantity={quantity} observed={observed} threshold={threshold} judged={at} moved_threshold={moved} judged_at_moved={at_moved} flips={}",
        at != at_moved
    )
}

fn segment_one_judgements(jobs: &[CellJob], results: &[FaultCellJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let find = |geometry: &str, cell: FaultCell, arm: Arm| {
        jobs.iter()
            .zip(results)
            .find(|(job, _)| job.geometry.name == geometry && job.cell == cell && job.arm == arm)
            .map(|(_, result)| result)
            .expect("每格每臂都有一件")
    };
    // 丙六格读数：全零 × 全零格逐字段比。
    let all_zero_cell = FaultCell {
        root_fault: ReadFaultKind::AllZero,
        system_configuration_fault: Some(ReadFaultKind::AllZero),
    };
    let today = find("main_point", all_zero_cell, Arm::Today);
    let fixed_line = today
        .lines
        .iter()
        .find(|line| line.contains("crash_point=fixed"))
        .cloned()
        .unwrap_or_default();
    let mut differing = Vec::new();
    let mut compare = |field: &str, equal: bool| {
        if !equal {
            differing.push(field.to_string());
        }
    };
    compare(
        "mount",
        fixed_line.contains(&format!("mount={MODEL_MOUNT_OF_THE_ALL_ZERO_CELL} ")),
    );
    compare(
        "persisted_writes",
        fixed_line.contains(MODEL_PERSISTED_WRITES_OF_THE_ALL_ZERO_CELL),
    );
    compare(
        "mount_after",
        fixed_line.contains(&format!(
            "mount_after={MODEL_MOUNT_AFTER_OF_THE_ALL_ZERO_CELL} "
        )),
    );
    match &today.fixed {
        Some(reading) => {
            compare("roots", reading.roots == MODEL_ROOTS_OF_THE_ALL_ZERO_CELL);
            compare(
                "roots_of_two_instances_at_one_txg",
                !reading.roots_of_two_instances_at_one_txg,
            );
            compare(
                "instance_and_txg_order_crossed_on_roots",
                reading.instance_and_txg_order_crossed_on_roots,
            );
            compare(
                "choose_root",
                reading.chosen == Some(MODEL_CHOSEN_OF_THE_ALL_ZERO_CELL),
            );
            compare(
                "content",
                reading.content_index == Some(MODEL_CONTENT_OF_THE_ALL_ZERO_CELL),
            );
            compare(
                "checker_red",
                reading.violated == MODEL_CHECKER_RED_OF_THE_ALL_ZERO_CELL,
            );
        }
        None => compare("fixed_crash_point", false),
    }
    lines.push(format!(
        "name=today_six_cells_reading cell={} reproduced={} differing_fields=[{}] model_line=research/prompts/e205-preregistration.md:336",
        all_zero_cell.label(),
        differing.is_empty(),
        differing.join(",")
    ));
    let mut other_cells_reproduced = 0;
    for cell in six_cells()
        .into_iter()
        .filter(|cell| *cell != all_zero_cell)
    {
        let result = find("main_point", cell, Arm::Today);
        let reproduced = result
            .fixed
            .as_ref()
            .is_some_and(|reading| reading.instance_and_txg_order_crossed_on_roots)
            && !result.fixed_lost;
        other_cells_reproduced += usize::from(reproduced);
        lines.push(format!(
            "name=today_six_cells_reading cell={} crossed_order_on_roots_and_last_confirmed_overwrite_kept={reproduced}",
            cell.label()
        ));
    }
    lines.push(format!(
        "name=today_six_cells_reading_other_five reproduced_cells={other_cells_reproduced} of=5"
    ));
    for geometry in CELL_GEOMETRIES {
        let cells: Vec<&FaultCellJudgement> = six_cells()
            .into_iter()
            .map(|cell| find(geometry.name, cell, Arm::Today))
            .collect();
        let red_cells = cells
            .iter()
            .filter(|result| {
                result
                    .fixed
                    .as_ref()
                    .is_some_and(|reading| !reading.violated.is_empty())
            })
            .count();
        let refused_after = cells
            .iter()
            .filter(|result| result.fixed_mount_after_refused)
            .count();
        lines.push(format!(
            "name=today_d22_clause geometry={} cells_with_checker_red={red_cells} judged_at_main_point={} clause_holds={}",
            geometry.name,
            geometry.name == "main_point",
            red_cells == 0
        ));
        lines.push(format!(
            "name=today_refused_afterwards geometry={} cells_refused_afterwards={refused_after}",
            geometry.name
        ));
        if geometry.name == "main_point" {
            lines.push(discrimination_line(
                "today_d22_clause_red_cells",
                red_cells,
                1,
            ));
            lines.push(discrimination_line(
                "today_refused_afterwards",
                refused_after,
                1,
            ));
        }
        let control = find(geometry.name, CONTROL_CELL, Arm::Today);
        let control_refused = control
            .refusal
            .as_deref()
            .is_some_and(|name| name.starts_with("NewerStateStillUnreadableAfterOneReread"));
        lines.push(format!(
            "name=control_cell geometry={} {} refusal={} control={}",
            geometry.name,
            CONTROL_CELL.label(),
            control.refusal.as_deref().unwrap_or("none"),
            verdict_text(control_refused)
        ));
        for arm in Arm::JIA {
            let jia: Vec<(FaultCell, &FaultCellJudgement)> = six_cells()
                .into_iter()
                .map(|cell| (cell, find(geometry.name, cell, arm)))
                .collect();
            let inverted: Vec<String> = jia
                .iter()
                .filter(|(_, result)| {
                    result
                        .fixed
                        .as_ref()
                        .is_some_and(|reading| reading.inverted)
                })
                .map(|(cell, _)| cell.label().replace(' ', "/"))
                .collect();
            let lost_where_today_kept = jia
                .iter()
                .filter(|(cell, result)| {
                    result.fixed_lost && !find(geometry.name, *cell, Arm::Today).fixed_lost
                })
                .count();
            let red = jia
                .iter()
                .filter(|(_, result)| {
                    result
                        .fixed
                        .as_ref()
                        .is_some_and(|reading| !reading.violated.is_empty())
                })
                .count();
            let jia_refused_after = jia
                .iter()
                .filter(|(_, result)| result.fixed_mount_after_refused)
                .count();
            lines.push(format!(
                "name=jia_six_cells_inversion arm={} geometry={} cells_still_inverted={} cells=[{}] covers_the_six_cells={}",
                arm.name(),
                geometry.name,
                inverted.len(),
                inverted.join(","),
                inverted.is_empty()
            ));
            lines.push(format!(
                "name=jia_six_cells_image arm={} geometry={} cells_losing_the_last_confirmed_overwrite_where_today_keeps_it={lost_where_today_kept} cells_with_checker_red={red} cells_refused_afterwards={jia_refused_after}",
                arm.name(),
                geometry.name
            ));
            if geometry.name == "main_point" {
                lines.push(discrimination_line(
                    &format!("jia_six_cells_inversion.{}", arm.name()),
                    inverted.len(),
                    1,
                ));
                lines.push(discrimination_line(
                    &format!("jia_six_cells_lost.{}", arm.name()),
                    lost_where_today_kept,
                    1,
                ));
            }
        }
        for yi in YiReading::EVERY {
            let refused = cells
                .iter()
                .filter(|result| {
                    result
                        .yi_refuses
                        .iter()
                        .any(|(reading, refuses)| *reading == yi && *refuses)
                })
                .count();
            lines.push(format!(
                "name=yi_six_cells yi={} geometry={} cells_refused={refused} of=6 covers_the_six_cells={}",
                yi.name(),
                geometry.name,
                refused == 6
            ));
        }
    }
    lines
}

/// 六格上写行 txg 的锚点（第七节「出自被测条款本身的」丙那一条与「独立算出的」甲六格那一条；阳性对照：任一个不等那一档整轮作废）。
fn six_cell_anchor_lines(
    jobs: &[CellJob],
    results: &[FaultCellJudgement],
) -> (Vec<String>, Vec<Arm>) {
    let mut lines = Vec::new();
    let mut failed_arms = BTreeSet::new();
    for (job, result) in jobs.iter().zip(results) {
        if job.cell.system_configuration_fault.is_none() {
            continue;
        }
        let expected =
            expected_row_txg_of(job.arm, six_cell_anchor(job.geometry, job.cell.root_fault));
        let passed = result.row_txg == Some(expected);
        if !passed {
            failed_arms.insert(job.arm);
        }
        lines.push(format!(
            "name=six_cell_anchor geometry={} {} arm={} expected_row_txg={expected} row_txg={} control={}",
            job.geometry.name,
            job.cell.label(),
            job.arm.name(),
            result.row_txg.map_or("none".to_string(), |txg| txg.to_string()),
            verdict_text(passed)
        ));
    }
    (lines, failed_arms.into_iter().collect())
}

// ============================================================================
// 九、并行（登记第十四节）：件号从共享计数器取，结果按件号排回，与线程数无关。
// ============================================================================

/// 把一处自己的线程变量从进程环境取：设了照它，没设取 `SINGLEFS_THREAD_CAP`，再没设取登记的每台默认重活线程数。
fn own_variable_or_default_heavy_threads_from_the_process_environment(
    own_variable: &str,
) -> std::num::NonZeroUsize {
    own_variable_or_default_heavy_threads(
        own_variable,
        std::env::var(own_variable).ok().as_deref(),
        std::env::var("SINGLEFS_THREAD_CAP").ok().as_deref(),
    )
}

/// 工作线程数：`E205_THREADS`（变量名写在这一条语句里：门禁 code-tooling 的 heavy-thread-defaults 格按读线程变量的那一条语句认它有没有回退到登记值）。
fn worker_threads() -> usize {
    own_variable_or_default_heavy_threads_from_the_process_environment("E205_THREADS").get()
}

fn in_parallel<Job: Sync, Output: Send>(
    jobs: &[Job],
    threads: usize,
    judge: impl Fn(&Job) -> Output + Sync,
) -> Vec<Output> {
    let next_job = AtomicUsize::new(0);
    let finished: Mutex<Vec<(usize, Output)>> = Mutex::new(Vec::new());
    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1).min(jobs.len().max(1)) {
            scope.spawn(|| loop {
                let index = next_job.fetch_add(1, Ordering::SeqCst);
                let Some(job) = jobs.get(index) else {
                    return;
                };
                let output = judge(job);
                let mut finished = finished.lock().expect("结果表的锁没被毒");
                finished.push((index, output));
                if finished.len().is_multiple_of(256) || finished.len() == jobs.len() {
                    eprintln!(
                        "  进度：判完 {} / {} 件，已用 {:.0} 秒",
                        finished.len(),
                        jobs.len(),
                        started.elapsed().as_secs_f64()
                    );
                }
            });
        }
    });
    let mut finished = finished.into_inner().expect("结果表的锁没被毒");
    finished.sort_by_key(|(index, _)| *index);
    finished.into_iter().map(|(_, output)| output).collect()
}

fn constant_lines() -> (Vec<String>, bool) {
    let checks = constant_checks();
    let all_equal = checks
        .iter()
        .all(|(_, local, implementation)| local == implementation);
    let mut lines: Vec<String> = checks
        .iter()
        .map(|(name, local, implementation)| {
            format!("name=constant quantity={name} local={local} implementation={implementation} equal={}", local == implementation)
        })
        .collect();
    lines.push(format!("name=constants_verdict all_equal={all_equal}"));
    (lines, all_equal)
}

fn run_segment_one(threads: usize, keying: &CellKeying) -> (Vec<String>, CellCounts) {
    let (mut lines, constants_equal) = constant_lines();
    lines.push(format!(
        "name=config segment=six_cells threads={threads} geometries={}",
        CELL_GEOMETRIES.len()
    ));
    let histories = starting_histories();
    let jobs = cell_jobs();
    let controls = control_jobs();
    // 单线程与多线程对拍（登记第十四节：第一段全量单线程再判一遍，逐字相同才往下；第十二节修订十二）是一条入库的判定：
    // 键对不上就第一段全部的件切片多线程判一遍、单线程再判一遍；键相同就复用（`judge_segment_one_through_the_library`）。
    let segment_one =
        segment_one_thread_cross_check_verdict(keying, &histories, &jobs, &controls, threads);
    eprintln!(
        "{}",
        thread_cross_check_sentence(segment_one.cross_check_outcome, &segment_one.cross_check)
    );
    let cells = segment_one.fault;
    let control_cells = segment_one.control;
    let identical = segment_one.cross_check.is_identical();
    let counts = cells.counts.plus(control_cells.counts);
    let results = cells.verdicts;
    let control_results = control_cells.verdicts;
    for result in &results {
        lines.extend(result.lines.iter().cloned());
    }
    lines.extend(control_results.iter().map(|control| control.line.clone()));
    let (anchor_lines, failed_arms) = six_cell_anchor_lines(&jobs, &results);
    lines.extend(anchor_lines);
    lines.extend(segment_one_judgements(&jobs, &results));
    let stops: Vec<&String> = results
        .iter()
        .flat_map(|result| result.stop_mismatches.iter())
        .collect();
    lines.extend(stops.iter().map(|stop| format!("name=stop_clause {stop}")));
    let voided: Vec<&String> = results
        .iter()
        .filter_map(|result| result.voided.as_ref())
        .collect();
    lines.extend(
        voided
            .iter()
            .map(|reason| format!("name=voided_state {reason}")),
    );
    let controls_failed = control_results
        .iter()
        .filter(|control| !control.passed)
        .count();
    let mounts = jobs.len()
        + jobs
            .iter()
            .filter(|job| job.arm == Arm::TodayViaSwitch)
            .count();
    lines.push(format!(
        "name=verdict segment=six_cells constants_equal={constants_equal} thread_cross_check_identical={identical} cell_jobs={} control_jobs={} mounts_of_the_cells={mounts} controls_failed={controls_failed} six_cell_anchor_failed_arms=[{}] stop_clause_mismatches={} voided_states={}",
        jobs.len(),
        controls.len(),
        failed_arms.iter().map(|arm| arm.name()).collect::<Vec<_>>().join(","),
        stops.len(),
        voided.len()
    ));
    (lines, counts)
}

// ============================================================================
// 十、族历史的写者（第二、三段共用）：mkfs 之后在带录制的盘上一次接一次地「可写挂载 → p 次覆盖写 → 正常卸载」。
// ============================================================================

/// 录制流里一段是哪个入口发的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryKind {
    Mount,
    Publish,
    Unmount,
}

/// 一次挂载循环的读数（第三段轨迹与卸载空发布次数的核要它）。
#[derive(Clone, Debug)]
struct CycleReading {
    outcome: String,
    row_txg: Option<u64>,
    first_txg_of_today: u64,
    publishes_of_the_mount: u64,
    overwrites_done: u64,
    unmount: String,
    /// (卸载开始时现行那一版的 txg, 推了几次空发布)。
    unmount_publishes: Option<(u64, u64)>,
}

struct HistoryWriter {
    parameters: MakeFilesystemParameters,
    base: MemoryPool,
    stream: SharedStream,
    devices: Vec<(
        DeviceIdentity,
        DeviceUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    )>,
    entries: Vec<(EntryKind, std::ops::Range<usize>)>,
    /// (那次覆盖写交回时录制流的长度, 内容序号)。
    confirmed: Vec<(usize, u64)>,
    next_content_index: u64,
}

impl HistoryWriter {
    fn after_make_filesystem(slots_per_region: u64) -> Self {
        let parameters = pool_parameters(slots_per_region);
        let base = image_right_after_make_filesystem(&parameters);
        let stream = SharedStream::retaining_contents();
        let devices = recorded_devices_of(&base, &[], &stream);
        HistoryWriter {
            parameters,
            base,
            stream,
            devices,
            entries: Vec::new(),
            confirmed: Vec::new(),
            next_content_index: 1,
        }
    }

    fn image_now(&self) -> MemoryPool {
        image_of_recorded_devices(&self.devices)
    }

    fn set_faults(&mut self, faults: &[ReadFault]) {
        for (identity, device) in &mut self.devices {
            device.faults = faults
                .iter()
                .filter(|fault| fault.device == *identity)
                .copied()
                .collect();
        }
    }

    /// 一次「可写挂载（经那一条臂、挂载里带 `mount_faults`）→ p 次覆盖写 → 正常卸载」。挂载被拒就只记结局。
    fn mount_cycle(
        &mut self,
        arm: Arm,
        overwrites: u64,
        mount_faults: &[ReadFault],
    ) -> CycleReading {
        let first_txg_of_today =
            readings_before_the_mount(&self.image_now(), mount_faults, &self.parameters)
                .first_txg_of_today;
        self.set_faults(mount_faults);
        let mount_stream_start = self.stream.operation_count();
        let mounted = mount_under_arm(&self.parameters, &mut self.devices, arm);
        self.entries.push((
            EntryKind::Mount,
            mount_stream_start..self.stream.operation_count(),
        ));
        self.set_faults(&[]);
        let Mounted {
            output,
            mut allocator,
            mut current,
        } = match mounted {
            Ok(mounted) => mounted,
            Err(error) => {
                return CycleReading {
                    outcome: format!("refused:{}", refusal_name(&error)),
                    row_txg: None,
                    first_txg_of_today,
                    publishes_of_the_mount: 0,
                    overwrites_done: 0,
                    unmount: "not_mounted".to_string(),
                    unmount_publishes: None,
                }
            }
        };
        let row_txg = output.row_publish.root().checkpoint_txg.0;
        let publishes_of_the_mount = current.root().checkpoint_txg.0 + 1 - row_txg;
        let mut overwrites_done = 0;
        let mut publish_refusal = None;
        for _ in 0..overwrites {
            let index = self.next_content_index;
            let content = file_content_of(index);
            let file = FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + index,
            };
            let publish_stream_start = self.stream.operation_count();
            let published = {
                let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
                match current.file_version() {
                    None => publish_first_file(
                        &mut writer,
                        &mut allocator,
                        current.root(),
                        file,
                        output.instance,
                        current.record_bytes(),
                    ),
                    Some(previous) => publish_overwrite(
                        &mut writer,
                        &mut allocator,
                        previous,
                        file,
                        output.instance,
                    ),
                }
            };
            let end = self.stream.operation_count();
            self.entries
                .push((EntryKind::Publish, publish_stream_start..end));
            match published {
                Ok(next) => {
                    current = PoolVersion::WithFile(std::sync::Arc::new(next));
                    self.confirmed.push((end, index));
                    self.next_content_index += 1;
                    overwrites_done += 1;
                }
                Err(error) => {
                    publish_refusal = Some(short_name_of_debug(&format!("{error:?}")));
                    break;
                }
            }
        }
        let txg_at_the_unmount = current.root().checkpoint_txg.0;
        let unmount_stream_start = self.stream.operation_count();
        let (parameters, devices, stream) = (&self.parameters, &mut self.devices, &self.stream);
        let unmounted = stream.record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                parameters,
                devices,
                &mut allocator,
                &mut current,
                ShadowLedger::On,
            )
        });
        self.entries.push((
            EntryKind::Unmount,
            unmount_stream_start..self.stream.operation_count(),
        ));
        let (unmount_text, unmount_publishes) = match unmounted {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => (
                "floor_raised".to_string(),
                Some((
                    txg_at_the_unmount,
                    u64::try_from(raised.publishes.len()).expect("几次空发布"),
                )),
            ),
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {
                ("nothing_written".to_string(), None)
            }
            Err(error) => (format!("refused:{}", refusal_name(&error)), None),
        };
        CycleReading {
            outcome: publish_refusal.map_or_else(
                || "ok".to_string(),
                |refusal| format!("ok_publish_refused:{refusal}"),
            ),
            row_txg: Some(row_txg),
            first_txg_of_today,
            publishes_of_the_mount,
            overwrites_done,
            unmount: unmount_text,
            unmount_publishes,
        }
    }

    /// 写到这一刻的写表、分段与每条写在录制流里的下标。
    fn writes_and_segments(&self) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<usize>) {
        writes_and_segments_with_stream_indexes_and_entries(
            &self.stream.retained_operations(),
            &self.stream.entry_spans(),
            &fixed_geometry(&self.parameters),
        )
    }
}

/// D16（发布语义） 已定项 1「正常卸载的代价」：现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次。
fn unmount_publishes_by_the_rule(current_txg: u64) -> u64 {
    if current_txg % 3 == 1 {
        3
    } else {
        2
    }
}

// ============================================================================
// 十一、第二段：撕裂轮换族（登记第一节「承诺内单故障」与「撕裂轮换」、第六节第二段）。
// ============================================================================

/// 一段里每次写取的三态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteState {
    NotPersisted,
    Persisted,
    Torn,
}

const WRITE_STATES: &[WriteState] = &[
    WriteState::NotPersisted,
    WriteState::Persisted,
    WriteState::Torn,
];

/// 一段 `write_count` 次写的全部组合（每次写各取一态），次序按三进制从低位起。
fn torn_segment_states(write_count: usize) -> Vec<Vec<WriteState>> {
    let mut states = vec![Vec::new()];
    for _ in 0..write_count {
        states = states
            .into_iter()
            .flat_map(|prefix| {
                WRITE_STATES.iter().map(move |state| {
                    let mut next = prefix.clone();
                    next.push(*state);
                    next
                })
            })
            .collect();
    }
    states
}

/// 撕裂轮换族那段历史：mkfs → 可写挂载 → 新池新建文件 → 覆盖写两次 → 正常卸载 → 可写挂载 → 覆盖写一次 → 正常卸载（都经产品入口）。
fn torn_rotation_history(slots_per_region: u64) -> HistoryWriter {
    let mut history = HistoryWriter::after_make_filesystem(slots_per_region);
    history.mount_cycle(Arm::Today, 3, &[]);
    history.mount_cycle(Arm::Today, 1, &[]);
    history
}

/// 一段系统配置写是哪一种（发布末尾的轮换、取号两轮、抬 F 先写）。
fn kind_of_the_system_configuration_segment(
    history: &HistoryWriter,
    writes: &[RetainedWrite],
    stream_indexes: &[usize],
    segment: &[usize],
) -> &'static str {
    let opening_write = segment[0];
    let entry = history
        .entries
        .iter()
        .find(|(_, range)| range.contains(&stream_indexes[opening_write]))
        .map(|(kind, _)| *kind);
    let previous_is_a_root_of_the_same_entry = segment
        .iter()
        .any(|index| writes[*index].kind == StepKind::RootRecordFua)
        || opening_write > 0
            && writes[opening_write - 1].kind == StepKind::RootRecordFua
            && history.entries.iter().any(|(_, range)| {
                range.contains(&stream_indexes[opening_write])
                    && range.contains(&stream_indexes[opening_write - 1])
            });
    match (previous_is_a_root_of_the_same_entry, entry) {
        (true, _) => "publish_end_rotation",
        (false, Some(EntryKind::Mount)) => "acquisition",
        (false, Some(EntryKind::Unmount)) => "floor_raise_pre_write",
        (false, Some(EntryKind::Publish) | None) => "other",
    }
}

/// 乙离线出路（登记第六节第二段「乙离线出路」，附带量）：在这张盘面上照 C596 的离线出路
/// （`rewrite_system_configuration_slots_for_the_asserted_version`）走一次，断言的版本取恢复择到的那一版（`choose_root`）。
/// 交回 (结局名, 走成了的话重写之后的盘面)。
fn offline_way_out(
    image: &MemoryPool,
    parameters: &MakeFilesystemParameters,
) -> (String, Option<MemoryPool>) {
    let reader = FaultyImage { image, faults: &[] };
    let configuration = match choose_system_configuration(&reader) {
        Ok(configuration) => configuration,
        Err(failure) => {
            return (
                format!(
                    "system_configuration_unchosen:{}",
                    short_name_of_debug(&format!("{failure:?}"))
                ),
                None,
            )
        }
    };
    let Some(chosen_root) = choose_root(&reader, &configuration) else {
        return ("no_root_chosen".to_string(), None);
    };
    let asserted = AssertedVersion {
        instance: chosen_root.instance,
        checkpoint_txg: chosen_root.checkpoint_txg,
    };
    let mut devices = devices_of(image, &[]);
    match rewrite_system_configuration_slots_for_the_asserted_version(
        parameters,
        &mut devices,
        asserted,
    ) {
        Ok(rewritten) => (
            format!(
                "rewritten:asserted=({},{}):slots={}",
                asserted.instance.0,
                asserted.checkpoint_txg.0,
                rewritten.slots_rewritten.len()
            ),
            Some(image_of_plain_devices(&devices)),
        ),
        Err(refusal) => (
            format!("refused:{}", short_name_of_debug(&format!("{refusal:?}"))),
            None,
        ),
    }
}

/// 走过离线出路之后乙两种读法拒不拒（在重写之后的盘面上重读每块盘两槽）；没走成的 `None`。
fn yi_refusals_after_the_offline_way_out(
    rewritten: Option<&MemoryPool>,
) -> Option<Vec<(YiReading, bool)>> {
    rewritten.map(|image| {
        let readings = system_configuration_slot_readings(&FaultyImage { image, faults: &[] });
        YiReading::EVERY
            .iter()
            .map(|yi| (*yi, yi_refuses(*yi, &readings)))
            .collect()
    })
}

/// 撕裂轮换族一组 S 的历史（录制流拆成写表与分段，能跨线程）与取到的那几段系统配置写（至多 `TORN_SEGMENTS_AT_MOST` 段）。
struct TornRotationHistory {
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    taken: Vec<TornSegment>,
}

/// 取到的一段：段号、种类、这一段之前整段持久的镜像。
struct TornSegment {
    segment_index: usize,
    kind: &'static str,
    prefix: MemoryPool,
}

fn build_torn_rotation_history(slots_per_region: u64) -> TornRotationHistory {
    let history = torn_rotation_history(slots_per_region);
    let (writes, segments, stream_indexes) = history.writes_and_segments();
    let mut prefix = history.base.clone();
    let mut taken = Vec::new();
    for (segment_index, segment) in segments.iter().enumerate() {
        let has_system_configuration = segment
            .iter()
            .any(|index| writes[*index].kind == StepKind::SystemConfigurationSlot);
        if has_system_configuration && taken.len() < TORN_SEGMENTS_AT_MOST {
            taken.push(TornSegment {
                segment_index,
                kind: kind_of_the_system_configuration_segment(
                    &history,
                    &writes,
                    &stream_indexes,
                    segment,
                ),
                prefix: prefix.clone(),
            });
        }
        prefix.apply_writes(
            &segment
                .iter()
                .map(|index| writes[*index].clone())
                .collect::<Vec<_>>(),
        );
    }
    TornRotationHistory {
        writes,
        segments,
        taken,
    }
}

/// 第二段三组 S 的历史：要时才建，几个线程同时要同一组时只建一次。
struct TornRotationHistories {
    by_slots_per_region: BTreeMap<u64, std::sync::OnceLock<TornRotationHistory>>,
}

fn torn_rotation_histories() -> TornRotationHistories {
    TornRotationHistories {
        by_slots_per_region: SLOTS_PER_REGION_OF_THE_HISTORIES
            .iter()
            .map(|slots_per_region| (*slots_per_region, std::sync::OnceLock::new()))
            .collect(),
    }
}

/// 这一组 S 的撕裂轮换族历史，第一次要时现建（`build_torn_rotation_history` 因此在第二段格入口的单元闭包里）。
fn torn_rotation_history_of(
    histories: &TornRotationHistories,
    slots_per_region: u64,
) -> &TornRotationHistory {
    histories
        .by_slots_per_region
        .get(&slots_per_region)
        .expect("第二段只取 SLOTS_PER_REGION_OF_THE_HISTORIES 那三组 S")
        .get_or_init(|| build_torn_rotation_history(slots_per_region))
}

/// 第二段一态：哪一组 S、哪一段、段里每次写取的那一态。
#[derive(Clone, Debug, PartialEq, Eq)]
struct TornStateJob {
    slots_per_region: u64,
    segment_index: usize,
    state: Vec<WriteState>,
}

/// 一态的镜像：段之前整段持久，段里每次写照 `state` 取不持久、持久或撕裂。
fn torn_state_image(
    history: &TornRotationHistory,
    segment: &TornSegment,
    state: &[WriteState],
) -> MemoryPool {
    let mut image = segment.prefix.clone();
    for (index, write_state) in history.segments[segment.segment_index].iter().zip(state) {
        let write = &history.writes[*index];
        match write_state {
            WriteState::NotPersisted => {}
            WriteState::Persisted => image.apply_writes(std::slice::from_ref(write)),
            WriteState::Torn => {
                let length = usize::try_from(write.length_in_bytes()).expect("写长");
                let old = PoolReader::read(&image, write.device, write.offset, length)
                    .expect("写落在盘内");
                let torn = torn_image_of_in_place_overwrite(&old, &write.contents);
                image
                    .devices
                    .get_mut(&write.device)
                    .expect("有这块盘")
                    .write(write.offset, &torn);
            }
        }
    }
    image
}

fn judge_a_torn_state(
    slots_per_region: u64,
    segment_index: usize,
    kind: &'static str,
    state: &[WriteState],
    image: &MemoryPool,
) -> TornStateResult {
    let parameters = pool_parameters(slots_per_region);
    let mut devices = devices_of(image, &[]);
    let today = mount_writable(&parameters, &mut devices).map_or_else(
        |error| format!("refused:{}", refusal_name(&error)),
        |_| "ok".to_string(),
    );
    let readings = system_configuration_slot_readings(&FaultyImage { image, faults: &[] });
    let state_text: Vec<&str> = state
        .iter()
        .map(|write_state| match write_state {
            WriteState::NotPersisted => "not_persisted",
            WriteState::Persisted => "persisted",
            WriteState::Torn => "torn",
        })
        .collect();
    let refusals: Vec<(YiReading, bool)> = YiReading::EVERY
        .iter()
        .map(|yi| (*yi, yi_refuses(*yi, &readings)))
        .collect();
    // 乙离线出路（附带）：只在多拒的态上走（乙有一种读法拒、丙做成）。
    let over_refused = today.starts_with("ok") && refusals.iter().any(|(_, refuses)| *refuses);
    let (offline_outcome, refusals_after_the_offline_way_out) = if over_refused {
        let (outcome, rewritten) = offline_way_out(image, &parameters);
        let after = yi_refusals_after_the_offline_way_out(rewritten.as_ref());
        (outcome, after)
    } else {
        ("not_walked_not_over_refused".to_string(), None)
    };
    let after_text = refusals_after_the_offline_way_out.as_ref().map_or_else(
        || "none".to_string(),
        |after| {
            after
                .iter()
                .map(|(yi, refuses)| format!("{}:{refuses}", yi.name()))
                .collect::<Vec<_>>()
                .join(",")
        },
    );
    let line = format!(
        "name=torn_state slots_per_region={slots_per_region} segment={segment_index} kind={kind} state=[{}] today_mount={today} system_configuration=[{}] refuse_directly={} into_undecidable={} c596_shape={} offline_way_out={offline_outcome} refusals_after_the_offline_way_out=[{after_text}]",
        state_text.join(","),
        system_configuration_readings_text(&readings),
        refusals[0].1,
        refusals[1].1,
        conflicts_with_c596(&readings)
    );
    (
        slots_per_region,
        kind,
        today.starts_with("ok"),
        refusals,
        conflicts_with_c596(&readings),
        line,
        offline_outcome,
        refusals_after_the_offline_way_out,
    )
}

fn run_segment_two(threads: usize, keying: &CellKeying) -> (Vec<String>, CellCounts) {
    let (mut lines, constants_equal) = constant_lines();
    lines.push(format!("name=config segment=torn_rotation threads={threads} segments_at_most={TORN_SEGMENTS_AT_MOST}"));
    let histories = torn_rotation_histories();
    let mut jobs = Vec::new();
    let mut voided_segments = 0;
    for slots_per_region in SLOTS_PER_REGION_OF_THE_HISTORIES {
        let history = torn_rotation_history_of(&histories, slots_per_region);
        for taken in &history.taken {
            let segment = &history.segments[taken.segment_index];
            let states = torn_segment_states(segment.len());
            let expected = 3usize.pow(u32::try_from(segment.len()).expect("段宽"));
            if states.len() != expected {
                voided_segments += 1;
            }
            let kinds_text: Vec<&str> = segment
                .iter()
                .map(|index| history.writes[*index].kind.name())
                .collect();
            lines.push(format!(
                "name=torn_segment slots_per_region={slots_per_region} segment={} kind={} writes=[{}] states={} expected_states={expected}",
                taken.segment_index,
                taken.kind,
                kinds_text.join(","),
                states.len()
            ));
            jobs.extend(states.into_iter().map(|state| TornStateJob {
                slots_per_region,
                segment_index: taken.segment_index,
                state,
            }));
        }
    }
    let cells = judge_cells_through_the_library(
        keying,
        CellEntry::TornState,
        &jobs,
        |job| format!("{job:?}"),
        threads,
        ThreadCrossCheck::StoreRightAfterJudging,
        |job| torn_state_verdict(&histories, job),
    );
    let results = cells.verdicts;
    lines.extend(results.iter().map(|result| result.5.clone()));
    lines.extend(segment_two_judgements(&results));
    lines.push(format!(
        "name=verdict segment=torn_rotation constants_equal={constants_equal} states={} voided_segments={voided_segments}",
        results.len()
    ));
    (lines, cells.counts)
}

/// 第二段一态判出来的：(S, 段的种类, 丙做成没有, 乙两种读法拒不拒, 盘面是不是 C596 那一形, 产物行,
/// 离线出路的结局名, 走成了离线出路之后乙两种读法拒不拒)。
type TornStateResult = (
    u64,
    &'static str,
    bool,
    Vec<(YiReading, bool)>,
    bool,
    String,
    String,
    Option<Vec<(YiReading, bool)>>,
);

fn segment_two_judgements(results: &[TornStateResult]) -> Vec<String> {
    let mut lines = Vec::new();
    for yi in YiReading::EVERY {
        let refused = |result: &TornStateResult| {
            result
                .3
                .iter()
                .any(|(reading, refuses)| *reading == yi && *refuses)
        };
        let mut total_over_refusals = 0;
        let mut total_conflicts = 0;
        for slots_per_region in [4u64, 8, 16] {
            for kind in [
                "publish_end_rotation",
                "acquisition",
                "floor_raise_pre_write",
                "other",
            ] {
                let group: Vec<&TornStateResult> = results
                    .iter()
                    .filter(|result| result.0 == slots_per_region && result.1 == kind)
                    .collect();
                if group.is_empty() {
                    continue;
                }
                let today_refused = group.iter().filter(|result| !result.2).count();
                let over_refusals: Vec<&&TornStateResult> = group
                    .iter()
                    .filter(|result| refused(result) && result.2)
                    .collect();
                let conflicts = over_refusals.iter().filter(|result| result.4).count();
                total_over_refusals += over_refusals.len();
                total_conflicts += conflicts;
                lines.push(format!(
                    "name=yi_torn_over_refusals yi={} slots_per_region={slots_per_region} kind={kind} states={} today_refused_states={today_refused} over_refused_states={} conflicting_with_c596={conflicts}",
                    yi.name(),
                    group.len(),
                    over_refusals.len()
                ));
            }
        }
        lines.push(format!(
            "name=yi_torn_over_refusals_total yi={} over_refused_states={total_over_refusals} over_refuses_within_the_promise={} conflicting_with_c596={total_conflicts} conflicts_with_settled_clauses={}",
            yi.name(),
            total_over_refusals >= 1,
            total_conflicts >= 1
        ));
        lines.push(discrimination_line(
            &format!("yi_torn_over_refusals.{}", yi.name()),
            total_over_refusals,
            1,
        ));
        lines.push(discrimination_line(
            &format!("yi_torn_c596_conflicts.{}", yi.name()),
            total_conflicts,
            1,
        ));
        // 乙离线出路（附带，不进判定）：这一读法多拒的态里，离线出路走成、走过之后这一读法不再拒的态数（「走得出来」）。
        let over_refused: Vec<&TornStateResult> = results
            .iter()
            .filter(|result| refused(result) && result.2)
            .collect();
        let walked_out = over_refused
            .iter()
            .filter(|result| {
                result.7.as_ref().is_some_and(|after| {
                    after
                        .iter()
                        .any(|(reading, refuses)| *reading == yi && !*refuses)
                })
            })
            .count();
        let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
        for result in &over_refused {
            let outcome_name = result.6.split(":slots=").next().unwrap_or_default();
            let outcome_name = outcome_name.split(":asserted=").next().unwrap_or_default();
            *outcomes.entry(outcome_name.to_string()).or_default() += 1;
        }
        lines.push(format!(
            "name=yi_offline_way_out yi={} over_refused_states={} walked_out_states={walked_out} outcomes=[{}] attached_quantity=true",
            yi.name(),
            over_refused.len(),
            outcomes
                .iter()
                .map(|(name, count)| format!("{name}:{count}"))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    lines
}

// ============================================================================
// 十二、第三段：三档各自的族历史、无故障族与单故障族、两族盘面后果、跳号轨迹（真写，模型对拍）。
// ============================================================================

/// 第三段的 (S, p) 组（登记第八节）：S = 8 的两组先跑。
const SKIP_GROUPS_OF_8_SLOTS_PER_REGION: [(u64, u64); 2] = [(8, 2), (8, 24)];
const SKIP_GROUPS_OF_4_AND_16_SLOTS_PER_REGION: [(u64, u64); 4] =
    [(4, 2), (4, 12), (16, 2), (16, 48)];

/// 一条族历史建完交回的（录制流拆成写表与分段，能跨线程）：每次挂载循环的读数、前 4 次挂载写流的长度、每次挂载边界的镜像（S8 对拍用）。
struct FamilyHistory {
    slots_per_region: u64,
    overwrites: u64,
    arm: Arm,
    with_the_persistent_fault: bool,
    parameters: MakeFilesystemParameters,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    stream_indexes: Vec<usize>,
    barrier_stream_indexes: Vec<usize>,
    confirmed: Vec<(usize, u64)>,
    highest_content_index: u64,
    cycles: Vec<CycleReading>,
    stream_cut_after_the_taken_mounts: usize,
    images_at_the_mount_boundaries: Vec<String>,
}

/// 一张镜像的摘要（S8 逐字节比挂载边界的镜像用：每块盘写过的扇区号与字节按次序进 SHA-256）。
fn digest_of(image: &MemoryPool) -> String {
    let mut message = Vec::new();
    for (identity, device) in &image.devices {
        message.extend_from_slice(&identity.0.to_le_bytes());
        for (sector, bytes) in device.written_sectors() {
            message.extend_from_slice(&sector.to_le_bytes());
            message.extend_from_slice(bytes);
        }
    }
    singlefs_verification::sha256::sha256_hexadecimal(&message)
}

fn trajectory_fault(parameters: &MakeFilesystemParameters) -> ReadFault {
    let device = parameters.region_devices
        [usize::try_from(TRAJECTORY_FAULTED_RING_SLOT.region).expect("区域号")];
    ReadFault {
        device,
        offset: slot_offset(
            TRAJECTORY_FAULTED_RING_SLOT,
            parameters.geometry.fixed_structure_slot_spacing,
        )
        .0,
        length: ROOT_SLOT_BYTES,
        kind: ReadFaultKind::DeviceError,
    }
}

fn build_family_history(
    slots_per_region: u64,
    overwrites: u64,
    arm: Arm,
    with_the_persistent_fault: bool,
) -> FamilyHistory {
    let mut writer = HistoryWriter::after_make_filesystem(slots_per_region);
    let faults = if with_the_persistent_fault {
        vec![trajectory_fault(&writer.parameters)]
    } else {
        Vec::new()
    };
    let mut cycles = Vec::new();
    let mut stream_cut_after_the_taken_mounts = 0;
    let mut images_at_the_mount_boundaries = vec![digest_of(&writer.image_now())];
    for mount_number in 1..=MOUNTS_OF_A_FAMILY_HISTORY {
        cycles.push(writer.mount_cycle(arm, overwrites, &faults));
        images_at_the_mount_boundaries.push(digest_of(&writer.image_now()));
        if mount_number == MOUNTS_WHOSE_STATES_ARE_TAKEN {
            stream_cut_after_the_taken_mounts = writer.stream.operation_count();
        }
    }
    let (writes, segments, stream_indexes) = writer.writes_and_segments();
    let barrier_stream_indexes = writer
        .stream
        .retained_operations()
        .iter()
        .enumerate()
        .filter(|(_, retained)| {
            retained.operation.kind == singlefs_verification::RecordedOperationKind::Barrier
        })
        .map(|(index, _)| index)
        .collect();
    FamilyHistory {
        slots_per_region,
        overwrites,
        arm,
        with_the_persistent_fault,
        parameters: writer.parameters.clone(),
        base: writer.base.clone(),
        writes,
        segments,
        stream_indexes,
        barrier_stream_indexes,
        confirmed: writer.confirmed.clone(),
        highest_content_index: writer.next_content_index - 1,
        cycles,
        stream_cut_after_the_taken_mounts,
        images_at_the_mount_boundaries,
    }
}

/// 算术模型的跳号序列（S9 第三段那一半）：照真写历史每次挂载实数的发布次数，按落点公式记每一槽住着的 txg，
/// 每次挂载 N = 读得出的槽里最大的 txg，今天 = N + 1，甲照第一节的式子越过那一档算进来的槽；这一次挂载的发布从模型自己给的第一个 txg 起连着落。
fn model_skips(history: &FamilyHistory) -> Vec<u64> {
    model_skips_of_the_cycles(
        &history.cycles,
        history.slots_per_region,
        history.arm,
        history.with_the_persistent_fault,
    )
}

fn model_skips_of_the_cycles(
    cycles: &[CycleReading],
    slots_per_region: u64,
    arm: Arm,
    with_the_persistent_fault: bool,
) -> Vec<u64> {
    let mut held: BTreeMap<RootRingSlot, u64> = expected_genesis_slots()
        .into_iter()
        .map(|slot| (slot, 0))
        .collect();
    let mut skips = Vec::new();
    for cycle in cycles {
        let readings: Vec<(RootRingSlot, RingSlotReading)> = (0..ROOT_RING_REGIONS_LOCAL)
            .flat_map(|region| (0..slots_per_region).map(move |slot| RootRingSlot { region, slot }))
            .map(|slot| {
                let reading = if with_the_persistent_fault && slot == TRAJECTORY_FAULTED_RING_SLOT {
                    RingSlotReading::BadNotAllZero
                } else {
                    held.get(&slot).map_or(RingSlotReading::AllZero, |txg| {
                        RingSlotReading::SelfVerified {
                            instance: 1,
                            txg: *txg,
                        }
                    })
                };
                (slot, reading)
            })
            .collect();
        let today = highest_self_verified_root_txg(&readings) + 1;
        let model_first_txg = slots_passed_by(arm).map_or(today, |passed| {
            first_txg_past_the_ring_slots(passed, &readings, slots_per_region, today)
        });
        skips.push(model_first_txg - today);
        if cycle.row_txg.is_none() {
            continue;
        }
        let published = cycle.publishes_of_the_mount
            + cycle.overwrites_done
            + cycle.unmount_publishes.map_or(0, |(_, count)| count);
        for txg in model_first_txg..model_first_txg + published {
            held.insert(ring_slot_of_txg(txg, slots_per_region), txg);
        }
    }
    skips
}

/// 第三段一条族历史是哪一条：(S, p)、臂、带不带那一处持续的读故障。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FamilySpecification {
    slots_per_region: u64,
    overwrites: u64,
    arm: Arm,
    with_the_persistent_fault: bool,
}

/// 第三段一态：族历史前 4 次挂载写流里的一个前缀（写表前 `persisted_writes` 条），单故障族另带一处读故障。
#[derive(Clone, Copy, Debug)]
struct SkipStateJob {
    history: FamilySpecification,
    family: &'static str,
    persisted_writes: usize,
    persisted_stream_end: usize,
    fault: Option<(RootRingSlot, ReadFaultKind)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SkipStateResult {
    refusal: Option<String>,
    skip: Option<u64>,
    red: usize,
    violated: Vec<String>,
    lost: bool,
    chosen: Option<VersionKey>,
    /// S3 比出来的（`BadSlotTxgsComparison::name`；挂载被拒是 `not_mounted`）。
    bad_slot_txgs_against_the_allocator: &'static str,
    stop_mismatches: Vec<String>,
    voided: Option<String>,
}

impl FamilyHistory {
    fn specification(&self) -> FamilySpecification {
        FamilySpecification {
            slots_per_region: self.slots_per_region,
            overwrites: self.overwrites,
            arm: self.arm,
            with_the_persistent_fault: self.with_the_persistent_fault,
        }
    }

    fn name(&self) -> String {
        format!(
            "slots_per_region={} overwrites={} arm={} history={}",
            self.slots_per_region,
            self.overwrites,
            self.arm.name(),
            if self.with_the_persistent_fault {
                "persistent_fault"
            } else {
                "no_fault"
            }
        )
    }

    /// 写表前 `persisted_writes` 条持久的镜像。
    fn image_with(&self, persisted_writes: usize) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply_writes(&self.writes[..persisted_writes]);
        image
    }

    /// 无故障族：前 4 次挂载写流的段边界崩溃态（第 k 段之前整段持久；k 从 0 到段数）。
    fn segment_boundary_prefixes(&self) -> Vec<(usize, usize)> {
        let mut prefixes = vec![(0, 0)];
        let mut persisted = 0;
        for segment in &self.segments {
            if segment
                .iter()
                .any(|index| self.stream_indexes[*index] >= self.stream_cut_after_the_taken_mounts)
            {
                break;
            }
            persisted += segment.len();
            prefixes.push((persisted, self.stream_indexes[persisted - 1] + 1));
        }
        prefixes
    }

    /// 单故障族的干净态：mkfs 之后那一态，与前 4 次挂载里每次发布（根槽写之后的那一道屏障）之后。
    fn clean_prefixes(&self) -> Vec<(usize, usize)> {
        let mut prefixes = vec![(0, 0)];
        for (index, write) in self.writes.iter().enumerate() {
            let stream_index = self.stream_indexes[index];
            if write.kind != StepKind::RootRecordFua
                || stream_index >= self.stream_cut_after_the_taken_mounts
            {
                continue;
            }
            let barrier = self
                .barrier_stream_indexes
                .iter()
                .copied()
                .find(|barrier| *barrier > stream_index)
                .unwrap_or(usize::MAX);
            let persisted = self
                .stream_indexes
                .iter()
                .filter(|written| **written < barrier)
                .count();
            let prefix = (persisted, self.stream_indexes[persisted - 1] + 1);
            if prefixes.last() != Some(&prefix) {
                prefixes.push(prefix);
            }
        }
        prefixes
    }
}

fn judge_a_skip_state(history: &FamilyHistory, job: SkipStateJob) -> SkipStateResult {
    let parameters = &history.parameters;
    let image = history.image_with(job.persisted_writes);
    let faults: Vec<ReadFault> = job
        .fault
        .map(|(slot, kind)| {
            let device = parameters.region_devices[usize::try_from(slot.region).expect("区域号")];
            ReadFault {
                device,
                offset: slot_offset(slot, parameters.geometry.fixed_structure_slot_spacing).0,
                length: ROOT_SLOT_BYTES,
                kind,
            }
        })
        .into_iter()
        .collect();
    let before = readings_before_the_mount(&image, &faults, parameters);
    let mut devices = devices_of(&image, &faults);
    let mounted = mount_under_arm(parameters, &mut devices, history.arm);
    let image_after_the_single_fault_mount = image_of_plain_devices(&devices);
    let mut stop_mismatches = Vec::new();
    let (refusal, skip, bad_slot_txgs_against_the_allocator) = match &mounted {
        Ok(mounted) => {
            let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
            let apparatus_first_txg = before.first_txg_of(history.arm);
            if row_txg != apparatus_first_txg {
                stop_mismatches.push(format!("S1_or_S6 {} family={} persisted_writes={} row_txg={row_txg} apparatus={apparatus_first_txg}", history.name(), job.family, job.persisted_writes));
            }
            // S3（登记第十一节）：开关挂载取那一档的第一个 txg，丙取今天的；挂载写过的槽按挂载前后的盘面比出。
            let allocator_map: BTreeMap<RootRingSlot, u64> =
                ring_slots_bad_or_reading_back_all_zero_at_mount_by(&mounted.allocator)
                    .into_iter()
                    .map(|(slot, txg)| (slot, txg.0))
                    .collect();
            let comparison = bad_slot_txgs_compared_with_the_allocator(
                &FaultyImage {
                    image: &image,
                    faults: &faults,
                },
                parameters,
                &before.ring,
                &ring_slots_changed_between(
                    &image,
                    &image_after_the_single_fault_mount,
                    parameters,
                ),
                row_txg,
                &allocator_map,
            );
            if let BadSlotTxgsComparison::Differs { apparatus } = &comparison {
                stop_mismatches.push(format!(
                    "S3 {} family={} persisted_writes={} fault={:?} apparatus={apparatus:?} allocator={allocator_map:?}",
                    history.name(),
                    job.family,
                    job.persisted_writes,
                    job.fault
                ));
            }
            (
                None,
                Some(row_txg.saturating_sub(before.first_txg_of_today)),
                comparison.name(),
            )
        }
        Err(error) => (Some(refusal_name(error)), None, "not_mounted"),
    };
    let last_confirmed = last_confirmed_content(&history.confirmed, job.persisted_stream_end);
    match read_the_image(
        &image_after_the_single_fault_mount,
        &[],
        parameters,
        history.highest_content_index,
    ) {
        Ok(reading) => SkipStateResult {
            refusal,
            skip,
            red: reading.violated.len(),
            lost: confirmed_write_lost(reading.content_index, last_confirmed),
            chosen: reading.chosen,
            violated: reading.violated,
            bad_slot_txgs_against_the_allocator,
            stop_mismatches,
            voided: None,
        },
        Err(reason) => SkipStateResult {
            refusal,
            skip,
            red: 0,
            violated: Vec::new(),
            lost: false,
            chosen: None,
            bad_slot_txgs_against_the_allocator,
            stop_mismatches,
            voided: Some(format!(
                "{} family={} persisted_writes={} {reason}",
                history.name(),
                job.family,
                job.persisted_writes
            )),
        },
    }
}

fn skip_state_jobs(histories: &[&FamilyHistory]) -> Vec<SkipStateJob> {
    let mut jobs = Vec::new();
    for history in histories {
        if history.with_the_persistent_fault {
            continue;
        }
        for (persisted_writes, persisted_stream_end) in history.segment_boundary_prefixes() {
            jobs.push(SkipStateJob {
                history: history.specification(),
                family: "no_fault",
                persisted_writes,
                persisted_stream_end,
                fault: None,
            });
        }
        let slots = every_root_ring_slot(&history.parameters.geometry);
        for (persisted_writes, persisted_stream_end) in history.clean_prefixes() {
            for slot in &slots {
                for kind in ReadFaultKind::EVERY {
                    jobs.push(SkipStateJob {
                        history: history.specification(),
                        family: "single_fault",
                        persisted_writes,
                        persisted_stream_end,
                        fault: Some((*slot, kind)),
                    });
                }
            }
        }
    }
    jobs
}

fn mean_text(values: &[u64]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    let sum: u64 = values.iter().sum();
    format!("{sum}/{}", values.len())
}

/// 第三段的族历史：照规格要时才建，几个线程同时要同一条时只建一次。
struct FamilyHistories {
    by_specification: BTreeMap<FamilySpecification, std::sync::OnceLock<FamilyHistory>>,
}

fn family_histories(specifications: &[FamilySpecification]) -> FamilyHistories {
    FamilyHistories {
        by_specification: specifications
            .iter()
            .map(|specification| (*specification, std::sync::OnceLock::new()))
            .collect(),
    }
}

/// 这一条族历史，第一次要时现建（`build_family_history` 因此在第三段格入口的单元闭包里）。
fn family_history_of(
    histories: &FamilyHistories,
    specification: FamilySpecification,
) -> &FamilyHistory {
    histories
        .by_specification
        .get(&specification)
        .expect("第三段的态只落在这一趟列出的族历史上")
        .get_or_init(|| {
            build_family_history(
                specification.slots_per_region,
                specification.overwrites,
                specification.arm,
                specification.with_the_persistent_fault,
            )
        })
}

fn run_segment_three(
    threads: usize,
    groups: &[(u64, u64)],
    group_name: &str,
    keying: &CellKeying,
) -> (Vec<String>, CellCounts) {
    let (mut lines, constants_equal) = constant_lines();
    lines.push(format!("name=config segment=first_txg_skips groups={group_name} threads={threads} mounts_per_history={MOUNTS_OF_A_FAMILY_HISTORY} mounts_whose_states_are_taken={MOUNTS_WHOSE_STATES_ARE_TAKEN}"));
    let mut specifications = Vec::new();
    for (slots_per_region, overwrites) in groups {
        for arm in Arm::OF_THE_FAMILIES {
            for with_the_persistent_fault in [false, true] {
                specifications.push(FamilySpecification {
                    slots_per_region: *slots_per_region,
                    overwrites: *overwrites,
                    arm,
                    with_the_persistent_fault,
                });
            }
        }
    }
    // 跳号轨迹与 S8 每一趟都要全部族历史（不是格、不入库），在这里并行建齐；格入口照同一份取。
    let family_histories_by_specification = family_histories(&specifications);
    in_parallel(&specifications, threads, |specification| {
        family_history_of(&family_histories_by_specification, *specification);
    });
    let histories: Vec<&FamilyHistory> = specifications
        .iter()
        .map(|specification| family_history_of(&family_histories_by_specification, *specification))
        .collect();
    let mut stops = Vec::new();
    let mut unmount_rule_mismatches = 0;
    for history in &histories {
        let real: Vec<String> = history
            .cycles
            .iter()
            .map(|cycle| {
                cycle.row_txg.map_or_else(
                    || "refused".to_string(),
                    |row| row.saturating_sub(cycle.first_txg_of_today).to_string(),
                )
            })
            .collect();
        let real_numbers: Vec<u64> = history
            .cycles
            .iter()
            .filter_map(|cycle| {
                cycle
                    .row_txg
                    .map(|row| row.saturating_sub(cycle.first_txg_of_today))
            })
            .collect();
        let rule_ok = history.cycles.iter().all(|cycle| {
            cycle
                .unmount_publishes
                .is_none_or(|(txg, count)| count == unmount_publishes_by_the_rule(txg))
        });
        unmount_rule_mismatches += usize::from(!rule_ok);
        let model = model_skips(history);
        let model_text: Vec<String> = model.iter().map(u64::to_string).collect();
        let model_agrees = real == model_text;
        if rule_ok && !model_agrees {
            stops.push(format!(
                "S9 {} real=[{}] model=[{}]",
                history.name(),
                real.join(","),
                model_text.join(",")
            ));
        }
        lines.push(format!(
            "name=skip_trajectory {} skips=[{}] peak={} positive_mounts={} final={} mean={} model=[{}] model_column={} cycles=[{}] every_mount_skips={}",
            history.name(),
            real.join(","),
            real_numbers.iter().max().map_or("none".to_string(), u64::to_string),
            real_numbers.iter().filter(|skip| **skip > 0).count(),
            real.last().cloned().unwrap_or_default(),
            mean_text(&real_numbers),
            model_text.join(","),
            if !rule_ok { "voided_by_the_unmount_rule" } else if model_agrees { "equal" } else { "differs" },
            history.cycles.iter().map(|cycle| format!("{}|{}", cycle.outcome, cycle.unmount)).collect::<Vec<_>>().join(","),
            real_numbers.iter().filter(|skip| **skip > 0).count() >= 2
        ));
    }
    // S8：甲只并读坏那条无故障族历史在每个挂载边界上与丙那条逐字节相同。
    for narrow in histories
        .iter()
        .copied()
        .filter(|history| history.arm == Arm::PastReadBad && !history.with_the_persistent_fault)
    {
        let today = histories
            .iter()
            .find(|history| {
                history.arm == Arm::Today
                    && !history.with_the_persistent_fault
                    && history.slots_per_region == narrow.slots_per_region
                    && history.overwrites == narrow.overwrites
            })
            .expect("同组有丙那条");
        let same = narrow.images_at_the_mount_boundaries == today.images_at_the_mount_boundaries;
        if !same {
            stops.push(format!(
                "S8 {} differs_from_today_at_a_mount_boundary",
                narrow.name()
            ));
        }
        lines.push(format!(
            "name=s8 {} same_as_today_at_every_mount_boundary={same}",
            narrow.name()
        ));
    }
    let jobs = skip_state_jobs(&histories);
    let cells = judge_cells_through_the_library(
        keying,
        CellEntry::SkipState,
        &jobs,
        |job| format!("{job:?}"),
        threads,
        ThreadCrossCheck::StoreRightAfterJudging,
        |job| skip_state_verdict(&family_histories_by_specification, job),
    );
    let results = cells.verdicts;
    lines.extend(segment_three_judgements(&jobs, &results));
    stops.extend(
        results
            .iter()
            .flat_map(|result| result.stop_mismatches.iter().cloned()),
    );
    // S3 逐态比出来的（登记第十一节 S3：判不了与有被抛弃根的态不比，计数报出）。
    let mut bad_slot_comparisons: BTreeMap<&'static str, usize> = BTreeMap::new();
    for result in &results {
        *bad_slot_comparisons
            .entry(result.bad_slot_txgs_against_the_allocator)
            .or_default() += 1;
    }
    lines.push(format!(
        "name=bad_slot_txgs_against_the_allocator segment=first_txg_skips groups={group_name} states=[{}]",
        bad_slot_comparisons
            .iter()
            .map(|(name, count)| format!("{name}:{count}"))
            .collect::<Vec<_>>()
            .join(",")
    ));
    let voided: Vec<&String> = results
        .iter()
        .filter_map(|result| result.voided.as_ref())
        .collect();
    lines.extend(stops.iter().map(|stop| format!("name=stop_clause {stop}")));
    lines.extend(
        voided
            .iter()
            .map(|reason| format!("name=voided_state {reason}")),
    );
    lines.push(format!(
        "name=verdict segment=first_txg_skips groups={group_name} constants_equal={constants_equal} histories={} states={} unmount_rule_mismatches={unmount_rule_mismatches} stop_clause_mismatches={} voided_states={}",
        histories.len(),
        results.len(),
        stops.len(),
        voided.len()
    ));
    (lines, cells.counts)
}

/// 第三段两族的判定行（登记第六节第三段：一档一族一组一行，丙同报作基线）。
fn segment_three_judgements(jobs: &[SkipStateJob], results: &[SkipStateResult]) -> Vec<String> {
    #[derive(Default)]
    struct Tally {
        states: usize,
        refusals: BTreeMap<String, usize>,
        skips: Vec<u64>,
        at_the_ring_length_less_one: usize,
        red_states: usize,
        lost_states: usize,
        red_names: BTreeSet<String>,
        lost_by_fault_kind: BTreeMap<&'static str, usize>,
    }
    let mut tallies: BTreeMap<(u64, u64, Arm, &'static str), Tally> = BTreeMap::new();
    for (job, result) in jobs.iter().zip(results) {
        let history = job.history;
        let root_ring_length_in_slots = ROOT_RING_REGIONS_LOCAL * history.slots_per_region;
        let tally = tallies
            .entry((
                history.slots_per_region,
                history.overwrites,
                history.arm,
                job.family,
            ))
            .or_default();
        tally.states += 1;
        if let Some(refusal) = &result.refusal {
            *tally.refusals.entry(refusal.clone()).or_default() += 1;
        }
        if let Some(skip) = result.skip {
            tally.skips.push(skip);
            tally.at_the_ring_length_less_one += usize::from(skip == root_ring_length_in_slots - 1);
        }
        if result.red > 0 {
            tally.red_states += 1;
            tally.red_names.extend(result.violated.iter().cloned());
        }
        if result.lost {
            tally.lost_states += 1;
            let kind = job.fault.map_or("none", |(_, kind)| kind.name());
            *tally.lost_by_fault_kind.entry(kind).or_default() += 1;
        }
    }
    let mut lines = Vec::new();
    for ((slots_per_region, overwrites, arm, family), tally) in &tallies {
        let today = tallies.get(&(*slots_per_region, *overwrites, Arm::Today, *family));
        let positive = tally.skips.iter().filter(|skip| **skip > 0).count();
        let refusals_today_has_not: Vec<&String> = tally
            .refusals
            .keys()
            .filter(|name| today.is_none_or(|today| !today.refusals.contains_key(*name)))
            .collect();
        let red_where_today_has_none =
            tally.red_states >= 1 && today.is_some_and(|today| today.red_states == 0);
        let lost_where_today_has_none =
            tally.lost_states >= 1 && today.is_some_and(|today| today.lost_states == 0);
        lines.push(format!(
            "name=jia_skip_family arm={} family={family} slots_per_region={slots_per_region} overwrites={overwrites} states={} mounted_states={} refusals=[{}] skip_maximum={} skip_mean={} skip_positive_states={positive} skip_at_ring_length_less_one={} changes_behaviour_without_a_fault={} more_refusals_than_today={}",
            arm.name(),
            tally.states,
            tally.skips.len(),
            tally.refusals.iter().map(|(name, count)| format!("{name}:{count}")).collect::<Vec<_>>().join(","),
            tally.skips.iter().max().map_or("none".to_string(), u64::to_string),
            mean_text(&tally.skips),
            tally.at_the_ring_length_less_one,
            positive >= 1,
            !refusals_today_has_not.is_empty()
        ));
        lines.push(format!(
            "name=jia_family_image arm={} family={family} slots_per_region={slots_per_region} overwrites={overwrites} checker_red_states={} checker_red=[{}] confirmed_lost_states={} lost_by_fault_kind=[{}] red_where_today_has_none={red_where_today_has_none} lost_where_today_has_none={lost_where_today_has_none}",
            arm.name(),
            tally.red_states,
            tally.red_names.iter().cloned().collect::<Vec<_>>().join(","),
            tally.lost_states,
            tally.lost_by_fault_kind.iter().map(|(kind, count)| format!("{kind}:{count}")).collect::<Vec<_>>().join(",")
        ));
        if *arm != Arm::Today {
            lines.push(discrimination_line(
                &format!(
                    "jia_skip_positive_states.{}.{family}.s{slots_per_region}.p{overwrites}",
                    arm.name()
                ),
                positive,
                1,
            ));
        }
    }
    lines
}

// ============================================================================
// 十三、逐格判定入库（`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」；判定库只有一份，
// `.claude/rules/verification.md`「判定库只有一份」）。
// ============================================================================
//
// 一格是一件独立判的工作：第一段的六格一格一臂一件与对照一件，第二段一态，第三段一态。
// 一格的键 = sha256（这一格入口函数的代码键、这一格自己的参数）。代码键由 `scripts/experiment-apparatus/device-code-key.py`
// 照入口的单元闭包、闭包真引到的 cfg 原子与真值、工具链、闭包真引到的依赖版本算好，经 `--code-key-file` 交进来；
// 参数取这一格工作 `derive(Debug)` 的全部字段值。路径、名字、行号、线程数、日期、机器、副本还是主树都不进键。
// 四个格入口：`fault_cell_verdict`、`control_cell_verdict`（第一段）、`torn_state_verdict`（第二段）、`skip_state_verdict`（第三段）；
// 各段建的历史在入口闭包里（入口经 `starting_history_of` 一族现建），各段的判定行与锚点是每趟照格的判定现算的，不是格、不入库。
// 第一段另有一条整段的判定：单线程对拍（入口 `segment_one_thread_cross_check_verdict`，写法在本节「第一段单线程对拍那一条」）。
// 判定库是 prune-server 里的那一份，读写经 device-code-key.py run 开的两根管道转（`crates/singlefs-device-library/src/lib.rs` 文件头），这里不开本地库、不连服务；
// 复用哨兵与强制现判在转交那一侧（device-code-key.py 文件头「判定库的转交」复用哨兵那一段）。库里的相对键：
//   items/<格键>.out     一格绿的判定：键行、parameters 行、meta 行（取键那天与机器，不进键、哨兵比判定时去掉）、字段行、done 行
//   red/<代码键>.tsv     这个代码键底下判红的格（`<格键>\t<参数>` 一行一格）：读回时先看它，记着红的照判、不复用，盖住库里原有的绿
//   codes/<代码键>.txt   组成清单原文（只给人查键对不上时变了哪些单元，不进键）
// 判红的格：停机对拍有不等、态作废、对照没过、单线程与多线程对拍不等（`StoredCellVerdict::is_red` 与第一段的对拍）。
// 键算不出的一趟（device-code-key.py 拒算时不给 `--code-key-file`）：每一格照判、不读不写库，记成拒算。

use singlefs_device_library::DeviceLibrary;

/// 一格从哪个入口判：device-code-key.py `--entry` 给的那几个函数，组成清单里 code_key 那一行的入口以 `::<函数名>` 收尾。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CellEntry {
    FaultCell,
    ControlCell,
    TornState,
    SkipState,
    /// 第一段单线程对拍那一条（整段一条判定，不是一格一条）。
    SegmentOneThreadCrossCheck,
}

impl CellEntry {
    fn function_name(self) -> &'static str {
        match self {
            CellEntry::FaultCell => "fault_cell_verdict",
            CellEntry::ControlCell => "control_cell_verdict",
            CellEntry::SegmentOneThreadCrossCheck => "segment_one_thread_cross_check_verdict",
            CellEntry::TornState => "torn_state_verdict",
            CellEntry::SkipState => "skip_state_verdict",
        }
    }
}

/// 每种模式判哪几个入口的格（组成清单里要恰好这几行 code_key）。
fn cell_entries_of_the_mode(words: &[&str]) -> &'static [CellEntry] {
    match words {
        ["six-cells"] => &[
            CellEntry::FaultCell,
            CellEntry::ControlCell,
            CellEntry::SegmentOneThreadCrossCheck,
        ],
        ["torn-rotation"] => &[CellEntry::TornState],
        ["first-txg-skips", "s8" | "s4-s16"] => &[CellEntry::SkipState],
        _ => &[],
    }
}

/// 取键脚本打的组成清单的第一行（device-code-key.py 的 FORM_LINE）。
const COMPOSITION_FORM_LINE: &str = "form\tdevice-code-key-v1";

/// 这一趟的判定库口子、各入口的代码键、组成清单原文与写进每格 meta 行的那一句（取键那天与机器）。
#[derive(Debug)]
struct CellLibrary {
    store: DeviceLibrary,
    code_keys: BTreeMap<CellEntry, String>,
    composition: String,
    meta_line: String,
}

/// 一趟按不按键读写判定库。
#[derive(Debug)]
enum CellKeying {
    Library(CellLibrary),
    /// 键算不出（没给 `--code-key-file`）：每一格照判、不读不写库，记成拒算。
    KeyRefused,
}

/// 读组成清单：第一行是 form；code_key 恰好是这一模式那几个入口各一行（入口以 `::<函数名>` 收尾、键是十六进制）；judged 与 host 两行可缺（缺写 `-`）。
fn cell_library_from_composition(
    store: DeviceLibrary,
    composition: &str,
    entries: &[CellEntry],
) -> Result<CellLibrary, String> {
    if composition.lines().next() != Some(COMPOSITION_FORM_LINE) {
        return Err("--code-key-file 不是 device-code-key.py 打的组成清单（第一行不是 form\tdevice-code-key-v1）".to_string());
    }
    let code_key_lines: Vec<(&str, &str)> = composition
        .lines()
        .filter_map(|line| line.strip_prefix("code_key\t"))
        .filter_map(|rest| rest.split_once('\t'))
        .collect();
    let names: Vec<&str> = entries.iter().map(|entry| entry.function_name()).collect();
    if code_key_lines.len() != entries.len() {
        return Err(format!(
            "组成清单里要恰好 {} 行 code_key（入口 {}），读到 {} 行",
            entries.len(),
            names.join("、"),
            code_key_lines.len()
        ));
    }
    let mut code_keys = BTreeMap::new();
    for entry in entries {
        let suffix = format!("::{}", entry.function_name());
        let found: Vec<&str> = code_key_lines
            .iter()
            .filter(|(entry_path, _)| entry_path.ends_with(&suffix))
            .map(|(_, code_key)| *code_key)
            .collect();
        let [code_key] = found.as_slice() else {
            return Err(format!(
                "组成清单里入口以 {suffix} 收尾的 code_key 要恰好一行，读到 {} 行",
                found.len()
            ));
        };
        if code_key.is_empty()
            || !code_key
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(format!("组成清单里的代码键不是十六进制：{code_key}"));
        }
        code_keys.insert(*entry, (*code_key).to_string());
    }
    let field = |name: &str| {
        composition
            .lines()
            .find_map(|line| {
                line.strip_prefix(name)
                    .and_then(|rest| rest.strip_prefix('\t'))
            })
            .unwrap_or("-")
            .to_string()
    };
    Ok(CellLibrary {
        store,
        code_keys,
        composition: composition.to_string(),
        meta_line: format!("meta judged={} host={}", field("judged"), field("host")),
    })
}

/// 一格的键：sha256（`code\t<代码键>\nparameters\t<参数>\n`）。只这两样进键。
fn cell_key(code_key: &str, parameters: &str) -> String {
    singlefs_verification::sha256::sha256_hexadecimal(
        format!("code\t{code_key}\nparameters\t{parameters}\n").as_bytes(),
    )
}

/// 字段值里的反斜杠、换行、回车、制表符写成 `\\`、`\n`、`\r`、`\t`（一个字段一行，按制表符切名与值）。
fn escaped_field_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// `escaped_field_value` 的逆；认不出的转义交 None（这一格按读不回现判）。
fn unescaped_field_value(text: &str) -> Option<String> {
    let mut value = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            value.push(character);
            continue;
        }
        match characters.next()? {
            '\\' => value.push('\\'),
            'n' => value.push('\n'),
            'r' => value.push('\r'),
            't' => value.push('\t'),
            _ => return None,
        }
    }
    Some(value)
}

/// 一格判定写成的字段行：`<字段名>\t<转义过的值>`，同名字段可有几行（按次序读回）。
struct VerdictFields {
    lines: Vec<String>,
}

impl VerdictFields {
    fn empty() -> VerdictFields {
        VerdictFields { lines: Vec::new() }
    }

    fn push(&mut self, name: &str, value: &str) {
        self.lines
            .push(format!("{name}\t{}", escaped_field_value(value)));
    }
}

/// 从库里读回的字段行。
struct StoredFields {
    fields: Vec<(String, String)>,
}

impl StoredFields {
    fn parse(lines: &[String]) -> Option<StoredFields> {
        let fields = lines
            .iter()
            .map(|line| {
                let (name, value) = line.split_once('\t')?;
                Some((name.to_string(), unescaped_field_value(value)?))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(StoredFields { fields })
    }

    /// 恰好一行的字段；没有或不止一行交 None。
    fn single(&self, name: &str) -> Option<&str> {
        let mut found = self.fields.iter().filter(|(field, _)| field == name);
        let (_, value) = found.next()?;
        found.next().is_none().then_some(value.as_str())
    }

    /// 同名的全部字段，按写进去的次序。
    fn every(&self, name: &str) -> Vec<String> {
        self.fields
            .iter()
            .filter(|(field, _)| field == name)
            .map(|(_, value)| value.clone())
            .collect()
    }
}

fn bool_from_text(text: &str) -> Option<bool> {
    match text {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn optional_number_text(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

fn optional_number_from_text(text: &str) -> Option<Option<u64>> {
    if text == "none" {
        return Some(None);
    }
    text.parse().ok().map(Some)
}

fn optional_text(value: Option<&str>) -> String {
    value.map_or_else(|| "absent".to_string(), |text| format!("present:{text}"))
}

fn optional_text_from(text: &str) -> Option<Option<String>> {
    if text == "absent" {
        return Some(None);
    }
    text.strip_prefix("present:")
        .map(|value| Some(value.to_string()))
}

fn version_key_from_text(text: &str) -> Option<VersionKey> {
    let (instance, txg) = text.split_once(',')?;
    Some((instance.parse().ok()?, txg.parse().ok()?))
}

/// `实例,txg;实例,txg`；空表写空串。
fn stored_version_keys_text(keys: &[VersionKey]) -> String {
    keys.iter()
        .map(|(instance, txg)| format!("{instance},{txg}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn stored_version_keys_from_text(text: &str) -> Option<Vec<VersionKey>> {
    if text.is_empty() {
        return Some(Vec::new());
    }
    text.split(';').map(version_key_from_text).collect()
}

fn optional_version_key_text(key: Option<VersionKey>) -> String {
    key.map_or_else(
        || "none".to_string(),
        |(instance, txg)| format!("{instance},{txg}"),
    )
}

fn optional_version_key_from_text(text: &str) -> Option<Option<VersionKey>> {
    if text == "none" {
        return Some(None);
    }
    version_key_from_text(text).map(Some)
}

/// `refuse_directly:true,into_undecidable:false`；空表写空串。
fn yi_refusals_text(refusals: &[(YiReading, bool)]) -> String {
    refusals
        .iter()
        .map(|(yi, refuses)| format!("{}:{refuses}", yi.name()))
        .collect::<Vec<_>>()
        .join(",")
}

fn yi_refusals_from_text(text: &str) -> Option<Vec<(YiReading, bool)>> {
    if text.is_empty() {
        return Some(Vec::new());
    }
    text.split(',')
        .map(|pair| {
            let (name, refuses) = pair.split_once(':')?;
            let yi = YiReading::EVERY.into_iter().find(|yi| yi.name() == name)?;
            Some((yi, bool_from_text(refuses)?))
        })
        .collect()
}

/// 读回一个只在几个写死的名字里取值的字段（段的种类、S3 的比法）。
fn static_name_among(text: &str, names: &[&'static str]) -> Option<&'static str> {
    names.iter().copied().find(|name| *name == text)
}

/// 第二段段的种类（`kind_of_the_system_configuration_segment` 交的那几个）。
const TORN_SEGMENT_KINDS: [&str; 4] = [
    "publish_end_rotation",
    "acquisition",
    "floor_raise_pre_write",
    "other",
];

/// S3 比出来的名字（`BadSlotTxgsComparison::name` 的四个与挂载被拒的 `not_mounted`）。
const BAD_SLOT_COMPARISON_NAMES: [&str; 5] = [
    "equal",
    "differs",
    "not_compared_abandoned_roots_on_the_ring",
    "not_compared_abandonment_undecidable",
    "not_mounted",
];

fn push_image_reading(fields: &mut VerdictFields, prefix: &str, reading: &ImageReading) {
    fields.push(
        &format!("{prefix}.roots"),
        &stored_version_keys_text(&reading.roots),
    );
    fields.push(
        &format!("{prefix}.records"),
        &stored_version_keys_text(&reading.records),
    );
    fields.push(
        &format!("{prefix}.roots_of_two_instances_at_one_txg"),
        &reading.roots_of_two_instances_at_one_txg.to_string(),
    );
    fields.push(
        &format!("{prefix}.instance_and_txg_order_crossed_on_roots"),
        &reading.instance_and_txg_order_crossed_on_roots.to_string(),
    );
    fields.push(
        &format!("{prefix}.instance_and_txg_order_crossed_on_roots_and_records"),
        &reading
            .instance_and_txg_order_crossed_on_roots_and_records
            .to_string(),
    );
    fields.push(&format!("{prefix}.inverted"), &reading.inverted.to_string());
    for name in &reading.violated {
        fields.push(&format!("{prefix}.violated"), name);
    }
    fields.push(
        &format!("{prefix}.chosen"),
        &optional_version_key_text(reading.chosen),
    );
    fields.push(
        &format!("{prefix}.content_index"),
        &optional_number_text(reading.content_index),
    );
    fields.push(&format!("{prefix}.content"), &reading.content);
}

fn image_reading_from(stored: &StoredFields, prefix: &str) -> Option<ImageReading> {
    let flag = |name: &str| bool_from_text(stored.single(&format!("{prefix}.{name}"))?);
    Some(ImageReading {
        roots: stored_version_keys_from_text(stored.single(&format!("{prefix}.roots"))?)?,
        records: stored_version_keys_from_text(stored.single(&format!("{prefix}.records"))?)?,
        roots_of_two_instances_at_one_txg: flag("roots_of_two_instances_at_one_txg")?,
        instance_and_txg_order_crossed_on_roots: flag("instance_and_txg_order_crossed_on_roots")?,
        instance_and_txg_order_crossed_on_roots_and_records: flag(
            "instance_and_txg_order_crossed_on_roots_and_records",
        )?,
        inverted: flag("inverted")?,
        violated: stored.every(&format!("{prefix}.violated")),
        chosen: optional_version_key_from_text(stored.single(&format!("{prefix}.chosen"))?)?,
        content_index: optional_number_from_text(
            stored.single(&format!("{prefix}.content_index"))?,
        )?,
        content: stored.single(&format!("{prefix}.content"))?.to_string(),
    })
}

/// 一格判定怎么存进库、怎么读回、算不算红（判红的记进红表、不当绿复用）。
trait StoredCellVerdict: Sized {
    fn verdict_fields(&self) -> VerdictFields;
    /// 读不回（缺字段、值认不出）交 None：这一格现判。
    fn from_stored_fields(stored: &StoredFields) -> Option<Self>;
    fn is_red(&self) -> bool;
}

impl StoredCellVerdict for FaultCellJudgement {
    fn verdict_fields(&self) -> VerdictFields {
        let mut fields = VerdictFields::empty();
        for line in &self.lines {
            fields.push("line", line);
        }
        fields.push("refusal", &optional_text(self.refusal.as_deref()));
        fields.push("row_txg", &optional_number_text(self.row_txg));
        fields.push("yi_refuses", &yi_refusals_text(&self.yi_refuses));
        match &self.fixed {
            Some(reading) => {
                fields.push("fixed", "present");
                push_image_reading(&mut fields, "fixed", reading);
            }
            None => fields.push("fixed", "absent"),
        }
        fields.push("fixed_lost", &self.fixed_lost.to_string());
        fields.push(
            "fixed_mount_after_refused",
            &self.fixed_mount_after_refused.to_string(),
        );
        for stop in &self.stop_mismatches {
            fields.push("stop_mismatch", stop);
        }
        fields.push("voided", &optional_text(self.voided.as_deref()));
        fields
    }

    fn from_stored_fields(stored: &StoredFields) -> Option<FaultCellJudgement> {
        let fixed = match stored.single("fixed")? {
            "present" => Some(image_reading_from(stored, "fixed")?),
            "absent" => None,
            _ => return None,
        };
        Some(FaultCellJudgement {
            lines: stored.every("line"),
            refusal: optional_text_from(stored.single("refusal")?)?,
            row_txg: optional_number_from_text(stored.single("row_txg")?)?,
            yi_refuses: yi_refusals_from_text(stored.single("yi_refuses")?)?,
            fixed,
            fixed_lost: bool_from_text(stored.single("fixed_lost")?)?,
            fixed_mount_after_refused: bool_from_text(stored.single("fixed_mount_after_refused")?)?,
            stop_mismatches: stored.every("stop_mismatch"),
            voided: optional_text_from(stored.single("voided")?)?,
        })
    }

    fn is_red(&self) -> bool {
        !self.stop_mismatches.is_empty() || self.voided.is_some()
    }
}

/// 一件对照判出来的：产物行与过没过。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ControlJudgement {
    line: String,
    passed: bool,
}

impl StoredCellVerdict for ControlJudgement {
    fn verdict_fields(&self) -> VerdictFields {
        let mut fields = VerdictFields::empty();
        fields.push("line", &self.line);
        fields.push("passed", &self.passed.to_string());
        fields
    }

    fn from_stored_fields(stored: &StoredFields) -> Option<ControlJudgement> {
        Some(ControlJudgement {
            line: stored.single("line")?.to_string(),
            passed: bool_from_text(stored.single("passed")?)?,
        })
    }

    fn is_red(&self) -> bool {
        !self.passed
    }
}

impl StoredCellVerdict for TornStateResult {
    fn verdict_fields(&self) -> VerdictFields {
        let (
            slots_per_region,
            kind,
            today_mounted,
            refusals,
            c596_shape,
            line,
            offline_outcome,
            refusals_after_the_offline_way_out,
        ) = self;
        let mut fields = VerdictFields::empty();
        fields.push("slots_per_region", &slots_per_region.to_string());
        fields.push("kind", kind);
        fields.push("today_mounted", &today_mounted.to_string());
        fields.push("refusals", &yi_refusals_text(refusals));
        fields.push("c596_shape", &c596_shape.to_string());
        fields.push("line", line);
        fields.push("offline_outcome", offline_outcome);
        fields.push(
            "refusals_after_the_offline_way_out",
            &refusals_after_the_offline_way_out.as_ref().map_or_else(
                || "none".to_string(),
                |walked| format!("walked:{}", yi_refusals_text(walked)),
            ),
        );
        fields
    }

    fn from_stored_fields(stored: &StoredFields) -> Option<TornStateResult> {
        let after_text = stored.single("refusals_after_the_offline_way_out")?;
        let after = if after_text == "none" {
            None
        } else {
            Some(yi_refusals_from_text(after_text.strip_prefix("walked:")?)?)
        };
        Some((
            stored.single("slots_per_region")?.parse().ok()?,
            static_name_among(stored.single("kind")?, &TORN_SEGMENT_KINDS)?,
            bool_from_text(stored.single("today_mounted")?)?,
            yi_refusals_from_text(stored.single("refusals")?)?,
            bool_from_text(stored.single("c596_shape")?)?,
            stored.single("line")?.to_string(),
            stored.single("offline_outcome")?.to_string(),
            after,
        ))
    }

    fn is_red(&self) -> bool {
        false
    }
}

impl StoredCellVerdict for SkipStateResult {
    fn verdict_fields(&self) -> VerdictFields {
        let mut fields = VerdictFields::empty();
        fields.push("refusal", &optional_text(self.refusal.as_deref()));
        fields.push("skip", &optional_number_text(self.skip));
        fields.push("red", &self.red.to_string());
        for name in &self.violated {
            fields.push("violated", name);
        }
        fields.push("lost", &self.lost.to_string());
        fields.push("chosen", &optional_version_key_text(self.chosen));
        fields.push(
            "bad_slot_txgs_against_the_allocator",
            self.bad_slot_txgs_against_the_allocator,
        );
        for stop in &self.stop_mismatches {
            fields.push("stop_mismatch", stop);
        }
        fields.push("voided", &optional_text(self.voided.as_deref()));
        fields
    }

    fn from_stored_fields(stored: &StoredFields) -> Option<SkipStateResult> {
        Some(SkipStateResult {
            refusal: optional_text_from(stored.single("refusal")?)?,
            skip: optional_number_from_text(stored.single("skip")?)?,
            red: stored.single("red")?.parse().ok()?,
            violated: stored.every("violated"),
            lost: bool_from_text(stored.single("lost")?)?,
            chosen: optional_version_key_from_text(stored.single("chosen")?)?,
            bad_slot_txgs_against_the_allocator: static_name_among(
                stored.single("bad_slot_txgs_against_the_allocator")?,
                &BAD_SLOT_COMPARISON_NAMES,
            )?,
            stop_mismatches: stored.every("stop_mismatch"),
            voided: optional_text_from(stored.single("voided")?)?,
        })
    }

    fn is_red(&self) -> bool {
        !self.stop_mismatches.is_empty() || self.voided.is_some()
    }
}

fn cell_record_path(cell_key: &str) -> String {
    format!("items/{cell_key}.out")
}

fn red_list_path(code_key: &str) -> String {
    format!("red/{code_key}.tsv")
}

fn composition_path(code_key: &str) -> String {
    format!("codes/{code_key}.txt")
}

fn cell_record_header(cell_key: &str, code_key: &str) -> String {
    format!("key cell_key={cell_key} code_key={code_key}")
}

/// 一格的全文：键行、parameters 行、meta 行、字段行、`done lines=<字段行数>`。
fn cell_record_text(
    cell_key: &str,
    code_key: &str,
    parameters: &str,
    meta_line: &str,
    verdict_lines: &[String],
) -> String {
    let mut text = format!(
        "{}\nparameters {}\n{meta_line}\n",
        cell_record_header(cell_key, code_key),
        escaped_field_value(parameters)
    );
    for line in verdict_lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str(&format!("done lines={}\n", verdict_lines.len()));
    text
}

/// 读回一格（库里那一条的内容，不在是 None）：键行、parameters 行都要对得上，第三行是 meta（不比），末行 done 报的行数对得上才算数；不然 None（现判）。
fn verdict_lines_read_back(
    contents: Option<String>,
    cell_key: &str,
    code_key: &str,
    parameters: &str,
) -> Option<Vec<String>> {
    let contents = contents?;
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
    let done = lines.pop()?;
    if lines.len() < 3
        || lines[0] != cell_record_header(cell_key, code_key)
        || lines[1] != format!("parameters {}", escaped_field_value(parameters))
        || !lines[2].starts_with("meta ")
    {
        return None;
    }
    let verdict_lines: Vec<String> = lines.into_iter().skip(3).collect();
    (done == format!("done lines={}", verdict_lines.len())).then_some(verdict_lines)
}

/// 红表：格键 → 参数（转义过的）。不在、读不出交空表。
fn red_cells_of(store: &DeviceLibrary, code_key: &str) -> BTreeMap<String, String> {
    store
        .read(&red_list_path(code_key))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(cell_key, parameters)| (cell_key.to_string(), parameters.to_string()))
        .collect()
}

fn red_list_text(red_cells: &BTreeMap<String, String>) -> String {
    red_cells
        .iter()
        .map(|(cell_key, parameters)| format!("{cell_key}\t{parameters}\n"))
        .collect()
}

/// 一个入口这一趟的格数：现判（有键、照判）、复用（键相同、库里有绿的）、拒算（没有键、照判、不入库），与现判里判红的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellCounts {
    fresh: usize,
    reused: usize,
    refused: usize,
    red: usize,
}

impl CellCounts {
    fn plus(self, other: CellCounts) -> CellCounts {
        CellCounts {
            fresh: self.fresh + other.fresh,
            reused: self.reused + other.reused,
            refused: self.refused + other.refused,
            red: self.red + other.red,
        }
    }
}

/// 现判的格判完怎么落库。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreadCrossCheck {
    /// 一格判完当场写进库（被打断之后重跑同一条命令就是续跑，只补没判的）。
    StoreRightAfterJudging,
    /// 第一段（登记第十四节）：现判的格多线程判一遍、单线程再判一遍，逐字相同的才写进库，对不上的记红。
    JudgeAgainOnOneThreadBeforeStoring,
}

/// 一个入口这一趟的结果：按工作的次序的判定（复用的与现判的合在一起）、格数、现判的格里单线程与多线程对拍对不上的那几格。
struct CellsJudged<Verdict> {
    verdicts: Vec<Verdict>,
    counts: CellCounts,
    /// 这一趟现判、单线程再判一遍对不上的格的参数（按工作的次序）。
    differing_parameters: Vec<String>,
}

/// 按键读回能复用的格，其余照判、按 `cross_check` 落库，红表照这一趟现判的结局改写。判定按工作的次序交回，与线程数、复用与否无关。
fn judge_cells_through_the_library<Job: Sync, Verdict: StoredCellVerdict + Send>(
    keying: &CellKeying,
    entry: CellEntry,
    jobs: &[Job],
    parameters_of: impl Fn(&Job) -> String,
    threads: usize,
    cross_check: ThreadCrossCheck,
    judge: impl Fn(&Job) -> Verdict + Sync,
) -> CellsJudged<Verdict> {
    let keyed: Option<(&CellLibrary, &str)> = match keying {
        CellKeying::Library(library) => Some((
            library,
            library
                .code_keys
                .get(&entry)
                .expect("组成清单读进来时这一模式的每个入口都核过有一行 code_key")
                .as_str(),
        )),
        CellKeying::KeyRefused => None,
    };
    let parameters: Vec<String> = jobs.iter().map(parameters_of).collect();
    let keys: Vec<Option<String>> = parameters
        .iter()
        .map(|parameter| keyed.map(|(_, code_key)| cell_key(code_key, parameter)))
        .collect();
    let red_before = keyed.map_or_else(BTreeMap::new, |(library, code_key)| {
        red_cells_of(&library.store, code_key)
    });
    let mut verdicts: Vec<Option<Verdict>> = keys
        .iter()
        .zip(&parameters)
        .map(|(key, parameter)| {
            let (library, code_key) = keyed?;
            let key = key.as_ref()?;
            if red_before.contains_key(key) {
                return None;
            }
            let contents = library.store.read(&cell_record_path(key));
            let lines = verdict_lines_read_back(contents, key, code_key, parameter)?;
            Verdict::from_stored_fields(&StoredFields::parse(&lines)?)
        })
        .collect();
    let missing: Vec<usize> = (0..jobs.len())
        .filter(|index| verdicts[*index].is_none())
        .collect();
    let store_green = |index: usize, verdict_lines: &[String]| {
        if let (Some((library, code_key)), Some(key)) = (keyed, &keys[index]) {
            library.store.write(
                &cell_record_path(key),
                &cell_record_text(
                    key,
                    code_key,
                    &parameters[index],
                    &library.meta_line,
                    verdict_lines,
                ),
            );
        }
    };
    // (下标, 判定, 红不红, 单线程再判一遍逐字相同没有)
    let fresh: Vec<(usize, Verdict, bool, bool)> = match cross_check {
        ThreadCrossCheck::StoreRightAfterJudging => in_parallel(&missing, threads, |index| {
            let verdict = judge(&jobs[*index]);
            let red = verdict.is_red();
            if !red {
                store_green(*index, &verdict.verdict_fields().lines);
            }
            (*index, verdict, red, true)
        }),
        ThreadCrossCheck::JudgeAgainOnOneThreadBeforeStoring => {
            let parallel = in_parallel(&missing, threads, |index| judge(&jobs[*index]));
            missing
                .iter()
                .zip(parallel)
                .map(|(index, verdict)| {
                    let lines = verdict.verdict_fields().lines;
                    let identical = judge(&jobs[*index]).verdict_fields().lines == lines;
                    let red = verdict.is_red() || !identical;
                    if !red {
                        store_green(*index, &lines);
                    }
                    (*index, verdict, red, identical)
                })
                .collect()
        }
    };
    let mut red_after = red_before.clone();
    let mut red_count = 0;
    let mut differing_parameters = Vec::new();
    for (index, verdict, red, identical) in fresh {
        red_count += usize::from(red);
        if !identical {
            differing_parameters.push(parameters[index].clone());
        }
        if let Some(key) = &keys[index] {
            if red {
                red_after.insert(key.clone(), escaped_field_value(&parameters[index]));
            } else {
                red_after.remove(key);
            }
        }
        verdicts[index] = Some(verdict);
    }
    if let Some((library, code_key)) = keyed {
        if red_after != red_before {
            library
                .store
                .write(&red_list_path(code_key), &red_list_text(&red_after));
        }
    }
    let judged_now = missing.len();
    let counts = CellCounts {
        fresh: if keyed.is_some() { judged_now } else { 0 },
        reused: jobs.len() - judged_now,
        refused: if keyed.is_some() { 0 } else { judged_now },
        red: red_count,
    };
    report_cell_counts(entry, counts);
    CellsJudged {
        verdicts: verdicts
            .into_iter()
            .map(|verdict| verdict.expect("读回的与现判的合起来每一格都有判定"))
            .collect(),
        counts,
        differing_parameters,
    }
}

// ---- 第一段单线程对拍那一条（登记第十四节「第一段 249 态全量单线程再判一遍，逐字相同才往下」；第十二节修订十二） ----
//
// 整段一条判定，键 = sha256（入口 `segment_one_thread_cross_check_verdict` 的代码键、第一段全部的件的参数），与格键同一套组成
// （`cell_key`）；线程数、路径、日期、机器不进键。这个入口的闭包含两个格入口与对拍的判法：判法里改一个单元，这一条就现判。
//   键对不上（库里没有这个键、读不回）或上一趟记了红：第一段全部的件切片多线程判一遍、单线程再判一遍，逐件逐字相同、
//     且与这一趟产物用的判定逐字相同才记绿（写 items/<键>.out、撤出红表）；
//   键相同、库里那一条是绿的：复用这一条，不重判（这一趟现判的格照旧各自单线程再判一遍，对不上的照样让这一条判红）；
//   判红：记进红表 red/<代码键>.tsv，并把 items/<键>.out 改写成判红的这一条（撤掉原有的绿）；
//   没有代码键（拒算）：照现判那样全量对拍，不读不写库。

/// 第一段单线程对拍那一条判定：比了几件、哪几件对不上（`fault <参数>` / `control <参数>`，按工作的次序）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ThreadCrossCheckJudgement {
    compared_jobs: usize,
    differing_jobs: Vec<String>,
}

impl ThreadCrossCheckJudgement {
    fn is_identical(&self) -> bool {
        self.differing_jobs.is_empty()
    }
}

impl StoredCellVerdict for ThreadCrossCheckJudgement {
    fn verdict_fields(&self) -> VerdictFields {
        let mut fields = VerdictFields::empty();
        fields.push("compared_jobs", &self.compared_jobs.to_string());
        for job in &self.differing_jobs {
            fields.push("differing_job", job);
        }
        fields
    }

    fn from_stored_fields(stored: &StoredFields) -> Option<ThreadCrossCheckJudgement> {
        Some(ThreadCrossCheckJudgement {
            compared_jobs: stored.single("compared_jobs")?.parse().ok()?,
            differing_jobs: stored.every("differing_job"),
        })
    }

    fn is_red(&self) -> bool {
        !self.is_identical()
    }
}

/// 这一条这一趟是怎么来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreadCrossCheckOutcome {
    /// 键对不上（库里没有这个键、读不回）：全量现判。
    JudgedNowKeyDidNotMatch,
    /// 键相同而上一趟记了红（红表里有它，或库里那一条是判红的）：全量现判。
    JudgedNowRedBefore,
    /// 键相同、库里那一条是绿的：复用，不重判。
    Reused,
    /// 没有代码键：全量现判，不读不写库。
    KeyRefused,
}

/// 这一趟这一条在库里的键。
struct CrossCheckKey<'library> {
    library: &'library CellLibrary,
    code_key: &'library str,
    key: String,
}

/// 这一条的参数：第一段全部的件（故障格、对照格各按工作的次序）`derive(Debug)` 的全部字段值，一件一行。
fn segment_one_cross_check_parameters<FaultWork: std::fmt::Debug, ControlWork: std::fmt::Debug>(
    fault_jobs: &[FaultWork],
    control_jobs: &[ControlWork],
) -> String {
    let mut parameters = String::new();
    for job in fault_jobs {
        parameters.push_str(&format!("fault {job:?}\n"));
    }
    for job in control_jobs {
        parameters.push_str(&format!("control {job:?}\n"));
    }
    parameters
}

/// 有代码键的一趟交这一条的键（入口 `segment_one_thread_cross_check_verdict` 的代码键与参数，`cell_key`）；拒算的一趟交 None。
fn cross_check_key<'library>(
    keying: &'library CellKeying,
    parameters: &str,
) -> Option<CrossCheckKey<'library>> {
    match keying {
        CellKeying::Library(library) => {
            let code_key = library
                .code_keys
                .get(&CellEntry::SegmentOneThreadCrossCheck)
                .expect("组成清单读进来时 six-cells 的三个入口都核过有一行 code_key")
                .as_str();
            Some(CrossCheckKey {
                library,
                code_key,
                key: cell_key(code_key, parameters),
            })
        }
        CellKeying::KeyRefused => None,
    }
}

/// 键相同、红表里没有、库里那一条读得回而且是绿的：交它（复用）；不然交为什么要现判。
fn reused_thread_cross_check(
    keyed: &CrossCheckKey<'_>,
    parameters: &str,
) -> Result<ThreadCrossCheckJudgement, ThreadCrossCheckOutcome> {
    if red_cells_of(&keyed.library.store, keyed.code_key).contains_key(&keyed.key) {
        return Err(ThreadCrossCheckOutcome::JudgedNowRedBefore);
    }
    let record = keyed.library.store.read(&cell_record_path(&keyed.key));
    let Some(stored_lines) =
        verdict_lines_read_back(record, &keyed.key, keyed.code_key, parameters)
    else {
        return Err(ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch);
    };
    match StoredFields::parse(&stored_lines)
        .as_ref()
        .and_then(ThreadCrossCheckJudgement::from_stored_fields)
    {
        Some(stored) if stored.is_identical() => Ok(stored),
        Some(_) => Err(ThreadCrossCheckOutcome::JudgedNowRedBefore),
        None => Err(ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch),
    }
}

/// 全量对拍一类件：全部切片多线程判一遍、单线程再判一遍，逐件逐字比；另比这一趟产物用的判定（`used`，复用的与现判的合在一起）。
/// 对不上的件以 `<类> <参数>` 记进 `differing`。
fn cross_check_every_job<Work: Sync + std::fmt::Debug, Verdict: StoredCellVerdict + Send>(
    kind: &str,
    jobs: &[Work],
    used: &[Verdict],
    threads: usize,
    judge: impl Fn(&Work) -> Verdict + Sync,
    differing: &mut Vec<String>,
) {
    let sliced = in_parallel(jobs, threads, &judge);
    for ((job, on_many_threads), in_the_product) in jobs.iter().zip(&sliced).zip(used) {
        let sliced_lines = on_many_threads.verdict_fields().lines;
        let one_thread_lines = judge(job).verdict_fields().lines;
        if one_thread_lines != sliced_lines || in_the_product.verdict_fields().lines != sliced_lines
        {
            differing.push(format!("{kind} {job:?}"));
        }
    }
}

/// 落库：判红的先记进红表、再把库里那一条改写成判红的这一条（撤掉原有的绿）；现判判绿的先写绿、再撤出红表；复用的绿不再写。
fn store_thread_cross_check(
    keyed: &CrossCheckKey<'_>,
    parameters: &str,
    judgement: &ThreadCrossCheckJudgement,
    outcome: ThreadCrossCheckOutcome,
) {
    let store = &keyed.library.store;
    let listed_red = red_cells_of(store, keyed.code_key);
    let mut red_cells = listed_red.clone();
    let record = || {
        store.write(
            &cell_record_path(&keyed.key),
            &cell_record_text(
                &keyed.key,
                keyed.code_key,
                parameters,
                &keyed.library.meta_line,
                &judgement.verdict_fields().lines,
            ),
        );
    };
    if judgement.is_red() {
        red_cells.insert(keyed.key.clone(), escaped_field_value(parameters));
        if red_cells != listed_red {
            store.write(&red_list_path(keyed.code_key), &red_list_text(&red_cells));
        }
        record();
        return;
    }
    if outcome != ThreadCrossCheckOutcome::Reused {
        record();
    }
    red_cells.remove(&keyed.key);
    if red_cells != listed_red {
        store.write(&red_list_path(keyed.code_key), &red_list_text(&red_cells));
    }
}

/// 第一段一趟判下来的：两类格（按工作的次序）、单线程对拍那一条与它这一趟是现判还是复用。
struct SegmentOneJudged<FaultVerdict, ControlVerdict> {
    fault: CellsJudged<FaultVerdict>,
    control: CellsJudged<ControlVerdict>,
    cross_check: ThreadCrossCheckJudgement,
    cross_check_outcome: ThreadCrossCheckOutcome,
}

/// 第一段：两类格按键读回或现判（现判的格各自单线程再判一遍，`judge_cells_through_the_library`），再判单线程对拍那一条、落库。
fn judge_segment_one_through_the_library<FaultWork, FaultVerdict, ControlWork, ControlVerdict>(
    keying: &CellKeying,
    fault_jobs: &[FaultWork],
    control_jobs: &[ControlWork],
    threads: usize,
    judge_fault: impl Fn(&FaultWork) -> FaultVerdict + Sync,
    judge_control: impl Fn(&ControlWork) -> ControlVerdict + Sync,
) -> SegmentOneJudged<FaultVerdict, ControlVerdict>
where
    FaultWork: Sync + std::fmt::Debug,
    FaultVerdict: StoredCellVerdict + Send,
    ControlWork: Sync + std::fmt::Debug,
    ControlVerdict: StoredCellVerdict + Send,
{
    let fault = judge_cells_through_the_library(
        keying,
        CellEntry::FaultCell,
        fault_jobs,
        |job| format!("{job:?}"),
        threads,
        ThreadCrossCheck::JudgeAgainOnOneThreadBeforeStoring,
        &judge_fault,
    );
    let control = judge_cells_through_the_library(
        keying,
        CellEntry::ControlCell,
        control_jobs,
        |job| format!("{job:?}"),
        threads,
        ThreadCrossCheck::JudgeAgainOnOneThreadBeforeStoring,
        &judge_control,
    );
    let parameters = segment_one_cross_check_parameters(fault_jobs, control_jobs);
    let keyed = cross_check_key(keying, &parameters);
    let looked_up = match &keyed {
        Some(key_in_the_library) => reused_thread_cross_check(key_in_the_library, &parameters),
        None => Err(ThreadCrossCheckOutcome::KeyRefused),
    };
    let (mut cross_check, cross_check_outcome) = match looked_up {
        Ok(stored) => (stored, ThreadCrossCheckOutcome::Reused),
        Err(outcome) => {
            let mut differing_jobs = Vec::new();
            cross_check_every_job(
                "fault",
                fault_jobs,
                &fault.verdicts,
                threads,
                &judge_fault,
                &mut differing_jobs,
            );
            cross_check_every_job(
                "control",
                control_jobs,
                &control.verdicts,
                threads,
                &judge_control,
                &mut differing_jobs,
            );
            (
                ThreadCrossCheckJudgement {
                    compared_jobs: fault_jobs.len() + control_jobs.len(),
                    differing_jobs,
                },
                outcome,
            )
        }
    };
    // 这一趟现判的格单线程再判一遍对不上的，同样让这一条判红（复用的那一趟也算）。
    let differing_this_pass = fault
        .differing_parameters
        .iter()
        .map(|parameter| format!("fault {parameter}"))
        .chain(
            control
                .differing_parameters
                .iter()
                .map(|parameter| format!("control {parameter}")),
        );
    for differing in differing_this_pass {
        if !cross_check.differing_jobs.contains(&differing) {
            cross_check.differing_jobs.push(differing);
        }
    }
    if let Some(key_in_the_library) = &keyed {
        store_thread_cross_check(
            key_in_the_library,
            &parameters,
            &cross_check,
            cross_check_outcome,
        );
    }
    SegmentOneJudged {
        fault,
        control,
        cross_check,
        cross_check_outcome,
    }
}

/// 成功句另报的那一行（标准错误）：这一条这一趟是现判还是复用、判绿还是判红。
fn thread_cross_check_sentence(
    outcome: ThreadCrossCheckOutcome,
    judgement: &ThreadCrossCheckJudgement,
) -> String {
    let compared = judgement.compared_jobs;
    let how = match outcome {
        ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch => {
            format!("现判（键对不上：第一段 {compared} 件切片多线程判一遍、单线程再判一遍）")
        }
        ThreadCrossCheckOutcome::JudgedNowRedBefore => format!(
            "现判（键相同而上一趟记了红：第一段 {compared} 件切片多线程判一遍、单线程再判一遍）"
        ),
        ThreadCrossCheckOutcome::Reused => {
            format!("复用（键相同，不重判；库里那一条比过第一段 {compared} 件）")
        }
        ThreadCrossCheckOutcome::KeyRefused => format!(
            "拒算（没有代码键：第一段 {compared} 件切片多线程判一遍、单线程再判一遍，不读不写库）"
        ),
    };
    let verdict = match (judgement.is_identical(), outcome) {
        (true, ThreadCrossCheckOutcome::KeyRefused) => "逐件逐字相同".to_string(),
        (true, ThreadCrossCheckOutcome::Reused) => "绿".to_string(),
        (
            true,
            ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch
            | ThreadCrossCheckOutcome::JudgedNowRedBefore,
        ) => "逐件逐字相同，记绿".to_string(),
        (false, ThreadCrossCheckOutcome::KeyRefused) => format!(
            "{} 件对不上（不入库）：{}",
            judgement.differing_jobs.len(),
            judgement.differing_jobs.join("；")
        ),
        (
            false,
            ThreadCrossCheckOutcome::Reused
            | ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch
            | ThreadCrossCheckOutcome::JudgedNowRedBefore,
        ) => format!(
            "{} 件对不上，记红、撤掉原有的绿：{}",
            judgement.differing_jobs.len(),
            judgement.differing_jobs.join("；")
        ),
    };
    format!(
        "e205：第一段单线程对拍（入口 segment_one_thread_cross_check_verdict）：{how}；{verdict}"
    )
}

/// 第一段单线程对拍那一条的入口（`--entry …::segment_one_thread_cross_check_verdict`）：闭包含两个格入口与对拍的判法。
fn segment_one_thread_cross_check_verdict(
    keying: &CellKeying,
    histories: &StartingHistories,
    jobs: &[CellJob],
    controls: &[ControlJob],
    threads: usize,
) -> SegmentOneJudged<FaultCellJudgement, ControlJudgement> {
    judge_segment_one_through_the_library(
        keying,
        jobs,
        controls,
        threads,
        |job| fault_cell_verdict(histories, job),
        |job| control_cell_verdict(histories, job),
    )
}

/// 第一段一格（六格一格一臂一件）的入口：device-code-key.py 照它的单元闭包算这一类格的代码键（`--entry …::fault_cell_verdict`）。
fn fault_cell_verdict(histories: &StartingHistories, job: &CellJob) -> FaultCellJudgement {
    judge_a_cell_job(
        starting_history_of(histories, job.geometry.slots_per_region),
        *job,
    )
}

/// 第一段一件对照的入口（`--entry …::control_cell_verdict`）。
fn control_cell_verdict(histories: &StartingHistories, job: &ControlJob) -> ControlJudgement {
    let (line, passed) = judge_a_control_job(histories, *job);
    ControlJudgement { line, passed }
}

/// 第二段一态的入口（`--entry …::torn_state_verdict`）。
fn torn_state_verdict(histories: &TornRotationHistories, job: &TornStateJob) -> TornStateResult {
    let history = torn_rotation_history_of(histories, job.slots_per_region);
    let segment = history
        .taken
        .iter()
        .find(|taken| taken.segment_index == job.segment_index)
        .expect("第二段的态只照取到的段列出（run_segment_two）");
    let image = torn_state_image(history, segment, &job.state);
    judge_a_torn_state(
        job.slots_per_region,
        job.segment_index,
        segment.kind,
        &job.state,
        &image,
    )
}

/// 第三段一态的入口（`--entry …::skip_state_verdict`）。
fn skip_state_verdict(histories: &FamilyHistories, job: &SkipStateJob) -> SkipStateResult {
    judge_a_skip_state(family_history_of(histories, job.history), *job)
}

/// 这一趟每个入口的格数打到标准错误（产物只在标准输出，与复用与否无关、逐字节可比）。
fn report_cell_counts(entry: CellEntry, counts: CellCounts) {
    eprintln!(
        "e205：入口 {} 逐格判定：现判 {}、复用 {}、拒算 {}；现判里判红 {} 格（记进红表，下一趟照判）",
        entry.function_name(),
        counts.fresh,
        counts.reused,
        counts.refused,
        counts.red
    );
}

// ============================================================================
// 十四、入口：准入、模式、产物行。
// ============================================================================

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e205_c971_inverted_first_txg_arm_costs.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn usage() {
    eprintln!("用法：e205_c971_inverted_first_txg_arm_costs <checks | six-cells | torn-rotation | first-txg-skips s8 | first-txg-skips s4-s16> [--code-key-file <组成清单>] [--force]");
    eprintln!("  逐格判定入库要经 scripts/experiment-apparatus/device-code-key.py run 起：它算各格入口的代码键、给 --code-key-file、开转交判定库的两根管道");
}

fn print_result_lines(lines: &[String]) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for line in lines {
        writeln!(handle, "E7RESULT {line}").expect("标准输出写得进");
    }
    writeln!(handle, "E7RESULT name=done result_lines={}", lines.len()).expect("标准输出写得进");
}

/// 从参数里取出 `--code-key-file <组成清单>`：交回 (余下的参数, 组成清单的路径)；给了两次或后面缺值交 Err。
fn split_code_key_file(arguments: Vec<String>) -> Result<(Vec<String>, Option<String>), String> {
    let mut rest = Vec::new();
    let mut code_key_file = None;
    let mut words = arguments.into_iter();
    while let Some(argument) = words.next() {
        if argument != "--code-key-file" {
            rest.push(argument);
            continue;
        }
        let path = words.next().ok_or("--code-key-file 后面没有值")?;
        if code_key_file.replace(path).is_some() {
            return Err("--code-key-file 给了两次".to_string());
        }
    }
    Ok((rest, code_key_file))
}

/// 这一趟按不按键读写判定库：没给 `--code-key-file`（device-code-key.py 算不出键时不给）是拒算；给了就要有转交的管道、
/// 组成清单读得进、这一模式的入口各一行 code_key。组成清单原文照存进库的 `codes/<代码键>.txt`。
fn cell_keying_of(
    code_key_file: Option<&str>,
    entries: &[CellEntry],
) -> Result<CellKeying, String> {
    let Some(path) = code_key_file else {
        eprintln!(
            "e205：没有 --code-key-file（键算不出）：这一趟每一格照判、不读不写判定库，记成拒算"
        );
        return Ok(CellKeying::KeyRefused);
    };
    let composition = std::fs::read_to_string(path)
        .map_err(|error| format!("读不了 --code-key-file {path}：{error}"))?;
    let store = DeviceLibrary::from_environment()?;
    let library = cell_library_from_composition(store, &composition, entries)?;
    for code_key in library.code_keys.values() {
        library
            .store
            .write(&composition_path(code_key), &library.composition);
    }
    Ok(CellKeying::Library(library))
}

/// 没有格的模式（`checks`）的格数。
const NO_CELLS: CellCounts = CellCounts {
    fresh: 0,
    reused: 0,
    refused: 0,
    red: 0,
};

fn main() {
    let forced_summary = preflight();
    let (arguments, code_key_file) = match split_code_key_file(
        std::env::args()
            .skip(1)
            .filter(|argument| argument != "--force")
            .collect(),
    ) {
        Ok(split) => split,
        Err(reason) => {
            eprintln!("  ✗ e205：{reason}");
            usage();
            std::process::exit(2);
        }
    };
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let Some(summary) = forced_summary {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let threads = worker_threads();
    let entries = cell_entries_of_the_mode(&words);
    let keying = if entries.is_empty() {
        CellKeying::KeyRefused
    } else {
        match cell_keying_of(code_key_file.as_deref(), entries) {
            Ok(keying) => keying,
            Err(reason) => {
                eprintln!("  ✗ e205：{reason}");
                eprintln!("  → 怎么办：经 scripts/experiment-apparatus/device-code-key.py run 起（文件头「模式」那一段的整条命令），不手给 --code-key-file");
                std::process::exit(2);
            }
        }
    };
    let (lines, counts) = match words.as_slice() {
        ["checks"] => (constant_lines().0, NO_CELLS),
        ["six-cells"] => run_segment_one(threads, &keying),
        ["torn-rotation"] => run_segment_two(threads, &keying),
        ["first-txg-skips", "s8"] => {
            run_segment_three(threads, &SKIP_GROUPS_OF_8_SLOTS_PER_REGION, "s8", &keying)
        }
        ["first-txg-skips", "s4-s16"] => run_segment_three(
            threads,
            &SKIP_GROUPS_OF_4_AND_16_SLOTS_PER_REGION,
            "s4-s16",
            &keying,
        ),
        _ => {
            usage();
            std::process::exit(2);
        }
    };
    print_result_lines(&lines);
    eprintln!(
        "e205：{} 跑完：现判 {}、复用 {}、拒算 {}",
        words.join(" "),
        counts.fresh,
        counts.reused,
        counts.refused
    );
}

// ============================================================================
// 十五、单测：纯算术的钉绝对值断言（答案出自登记第七节与第五节），与几条经真挂载走到的取样点。
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 根环读数：`self_verified` 里的 txg 各住落点公式那一槽（txg 0 住三个区域的槽 0），`bad` 里的 txg 那一槽读坏成 `bad_reading`，别的槽读回全零。
    fn ring_of(
        slots_per_region: u64,
        self_verified: &[u64],
        bad: &[u64],
        bad_reading: RingSlotReading,
    ) -> Vec<(RootRingSlot, RingSlotReading)> {
        let mut readings: BTreeMap<RootRingSlot, RingSlotReading> = (0..ROOT_RING_REGIONS_LOCAL)
            .flat_map(|region| (0..slots_per_region).map(move |slot| RootRingSlot { region, slot }))
            .map(|slot| (slot, RingSlotReading::AllZero))
            .collect();
        for txg in self_verified {
            let slots: Vec<RootRingSlot> = if *txg == 0 {
                expected_genesis_slots().into_iter().collect()
            } else {
                vec![ring_slot_of_txg(*txg, slots_per_region)]
            };
            for slot in slots {
                readings.insert(
                    slot,
                    RingSlotReading::SelfVerified {
                        instance: u32::from(*txg != 0),
                        txg: *txg,
                    },
                );
            }
        }
        for txg in bad {
            readings.insert(ring_slot_of_txg(*txg, slots_per_region), bad_reading);
        }
        readings.into_iter().collect()
    }

    fn both_arms(
        ring: &[(RootRingSlot, RingSlotReading)],
        slots_per_region: u64,
        today: u64,
    ) -> (u64, u64) {
        (
            first_txg_past_the_ring_slots(SlotsPassed::ReadBad, ring, slots_per_region, today),
            first_txg_past_the_ring_slots(
                SlotsPassed::ReadBadOrAllZero,
                ring,
                slots_per_region,
                today,
            ),
        )
    }

    #[test]
    fn local_constants_equal_the_implementation_constants() {
        for (name, local, implementation) in constant_checks() {
            assert_eq!(
                local, implementation,
                "本地常量 {name} 与实现侧同名常量不等"
            );
        }
        assert_eq!(constant_checks().len(), 9, "回比九项");
    }

    /// 第七节「mkfs 之后第一次可写挂载」：甲只并读坏 1，甲并全零 3S。
    #[test]
    fn past_the_ring_slots_right_after_make_filesystem_takes_1_past_read_bad_and_3s_past_read_bad_or_all_zero(
    ) {
        for slots_per_region in [4u64, 8, 16] {
            let ring = ring_of(slots_per_region, &[0], &[], RingSlotReading::AllZero);
            assert_eq!(
                both_arms(&ring, slots_per_region, 1),
                (1, 3 * slots_per_region),
                "S = {slots_per_region}：甲只并读坏、甲并全零"
            );
        }
    }

    /// 第七节「只坏 txg 5 那一槽（设备报读错）」：甲两种读法 3S + 6（S = 4、8、16 时 18、30、54）；「起点镜像不注故障」：甲只并读坏 7，甲并全零 3S。
    #[test]
    fn past_the_ring_slots_on_the_start_image_takes_the_registered_anchors() {
        for (slots_per_region, with_the_slot_of_txg_5_bad) in [(4u64, 18u64), (8, 30), (16, 54)] {
            let bad = ring_of(
                slots_per_region,
                &[0, 1, 2, 3, 4, 6],
                &[5],
                RingSlotReading::BadNotAllZero,
            );
            assert_eq!(
                both_arms(&bad, slots_per_region, 7),
                (with_the_slot_of_txg_5_bad, with_the_slot_of_txg_5_bad),
                "S = {slots_per_region} 坏 txg 5 那一槽"
            );
            let clean = ring_of(
                slots_per_region,
                &[0, 1, 2, 3, 4, 5, 6],
                &[],
                RingSlotReading::AllZero,
            );
            assert_eq!(
                both_arms(&clean, slots_per_region, 7),
                (7, 3 * slots_per_region),
                "S = {slots_per_region} 不注故障"
            );
        }
    }

    /// 第七节「甲六格」：根与记录读坏不是全零的格甲只并读坏 7、读回全零的格 5，甲并全零 3S；G 藏一条甲只并读坏 7 / 6。
    #[test]
    fn past_the_ring_slots_on_the_six_cells_takes_the_registered_anchors() {
        for slots_per_region in [4u64, 8, 16] {
            let not_zero = ring_of(
                slots_per_region,
                &[0, 1, 2, 3, 4],
                &[5, 6],
                RingSlotReading::BadNotAllZero,
            );
            assert_eq!(
                both_arms(&not_zero, slots_per_region, 5),
                (7, 3 * slots_per_region),
                "S = {slots_per_region} 读坏不是全零"
            );
            let zero = ring_of(
                slots_per_region,
                &[0, 1, 2, 3, 4],
                &[5, 6],
                RingSlotReading::AllZero,
            );
            assert_eq!(
                both_arms(&zero, slots_per_region, 5),
                (5, 3 * slots_per_region),
                "S = {slots_per_region} 读回全零"
            );
        }
        let hide_one = ring_of(8, &[0, 1, 2, 3, 4, 5], &[6], RingSlotReading::BadNotAllZero);
        assert_eq!(both_arms(&hide_one, 8, 6), (7, 24), "G 藏一条读坏不是全零");
        let hide_one_zero = ring_of(8, &[0, 1, 2, 3, 4, 5], &[6], RingSlotReading::AllZero);
        assert_eq!(both_arms(&hide_one_zero, 8, 6), (6, 24), "G 藏一条读回全零");
    }

    /// D23（journal 的角色与格式） 已定项 14 注 3：记录比根新时取记录。
    #[test]
    fn first_txg_today_takes_the_highest_record_when_it_is_above_the_highest_root() {
        assert_eq!(first_txg_today(4, 6), 7, "根最大 4、记录最大 6");
        assert_eq!(first_txg_today(6, 4), 7, "根最大 6、记录最大 4");
    }

    /// 倒挂的判法：同 txg、更旧实例的一条根算倒挂（txg 不小于 f_k）；都更低不算。
    #[test]
    fn inversion_counts_an_older_instance_at_the_same_txg_and_not_below_it() {
        assert!(
            inverted(&[(1, 5), (1, 6), (2, 6), (2, 7)]),
            "实例 1 的 txg 6 与实例 2 最小的 txg 6 相等"
        );
        assert!(
            !inverted(&[(1, 5), (1, 6), (2, 7), (2, 8)]),
            "实例 1 都低于实例 2 的最小 txg"
        );
        assert!(inverted(&[(1, 6), (2, 5)]), "形乙");
    }

    #[test]
    fn forms_tell_the_same_txg_pair_from_the_crossed_pair() {
        assert_eq!(forms_in(&[(1, 6), (2, 6)]), (true, false), "形甲");
        assert_eq!(forms_in(&[(1, 6), (2, 5)]), (false, true), "形乙");
        assert_eq!(forms_in(&[(1, 4), (2, 5)]), (false, false), "不倒挂");
    }

    /// 最后一次已确认的覆盖写取交回得最晚的那一次；比它旧的内容算丢，比它新的（没确认的）不算丢。
    #[test]
    fn last_confirmed_content_is_the_latest_confirmed_overwrite() {
        let confirmed = [(10, 1), (20, 2), (30, 3)];
        assert_eq!(
            last_confirmed_content(&confirmed, 25),
            Some(2),
            "持久到 25：第二次已确认"
        );
        assert_eq!(last_confirmed_content(&confirmed, 5), None, "一次都没确认");
        assert_eq!(
            last_confirmed_content(&CONFIRMED_BEFORE_THE_SIX_CELL_MOUNT, 0),
            Some(CONTENT_INDEX_OF_SECOND_OVERWRITE),
            "六格里最后确认的是 C"
        );
        assert!(confirmed_write_lost(Some(2), Some(3)), "读出 B、C 已确认");
        assert!(
            !confirmed_write_lost(Some(4), Some(3)),
            "读出没确认的 D 不算丢"
        );
        assert!(confirmed_write_lost(None, Some(3)), "读不出算丢");
        assert!(!confirmed_write_lost(None, None), "没有已确认的写");
    }

    /// 第五节乙的三张手造盘面：乙直拒「拒、拒、不拒」，乙入判不出「不拒、拒、不拒」；第七节「乙在六格」两种读法都拒；R2 在六格上判得出。
    #[test]
    fn yi_readings_on_the_hand_made_tables_and_the_six_cell_table() {
        use SystemConfigurationSlotReading::{
            AllZero, DeviceReadError, NotSelfVerified, SelfVerified,
        };
        let tables = [
            vec![
                [NotSelfVerified, SelfVerified],
                [SelfVerified, SelfVerified],
            ],
            vec![
                [NotSelfVerified, SelfVerified],
                [SelfVerified, NotSelfVerified],
            ],
            vec![[SelfVerified, SelfVerified], [SelfVerified, SelfVerified]],
        ];
        for (table, (refuse_directly, into_undecidable)) in tables.iter().zip(YI_HAND_MADE_ANSWERS)
        {
            assert_eq!(
                yi_refuses(YiReading::RefuseDirectly, table),
                refuse_directly,
                "乙直拒 {table:?}"
            );
            assert_eq!(
                yi_refuses(YiReading::IntoUndecidable, table),
                into_undecidable,
                "乙入判不出 {table:?}"
            );
        }
        let six_cell_all_zero = vec![[AllZero, SelfVerified], [SelfVerified, AllZero]];
        assert!(
            yi_refuses(YiReading::RefuseDirectly, &six_cell_all_zero),
            "六格系统配置读回全零：乙直拒拒"
        );
        assert!(
            yi_refuses(YiReading::IntoUndecidable, &six_cell_all_zero),
            "六格系统配置读回全零：乙入判不出拒"
        );
        assert!(
            !witness_undecidable_today(&six_cell_all_zero),
            "R2 在六格上判得出"
        );
        assert!(
            witness_undecidable_today(&[[DeviceReadError, SelfVerified], [SelfVerified, AllZero]]),
            "有槽报读错、没有一块盘两槽都自证过"
        );
        assert!(
            conflicts_with_c596(&tables[1]),
            "两块盘各一槽撕裂是 C596 那一形"
        );
        assert!(!conflicts_with_c596(&six_cell_all_zero), "读回全零的不是");
    }

    /// 第七节「撕裂态」：一段两次系统配置写，每次三态，共 9 态。
    #[test]
    fn one_torn_segment_of_two_writes_takes_nine_states() {
        let states = torn_segment_states(2);
        assert_eq!(states.len(), 9, "3 × 3");
        assert_eq!(
            states
                .iter()
                .filter(|state| state.contains(&WriteState::Torn))
                .count(),
            5,
            "至少一次撕裂的 5 态"
        );
        assert_eq!(torn_segment_states(1).len(), 3, "一次写三态");
    }

    #[test]
    fn violated_names_keep_invariant_8_14_and_drop_the_partially_judged() {
        let verdicts = vec![
            ("I-8.14", InvariantVerdict::Violated("倒挂".to_string())),
            ("I-3.1", InvariantVerdict::Violated("账".to_string())),
            (
                "I-9.28",
                InvariantVerdict::PartiallyJudged("没判到".to_string()),
            ),
            ("I-7.1", InvariantVerdict::Holds),
        ];
        assert_eq!(names_of_violated(verdicts), vec!["I-3.1", "I-8.14"]);
    }

    /// D16（发布语义） 已定项 1「正常卸载的代价」：余 1 时 3 次，余 0 或余 2 时 2 次。
    #[test]
    fn unmount_publishes_follow_the_rule_of_the_remainder() {
        assert_eq!(
            (
                unmount_publishes_by_the_rule(7),
                unmount_publishes_by_the_rule(6),
                unmount_publishes_by_the_rule(8)
            ),
            (3, 2, 2)
        );
    }

    /// 停机对拍 S3 的式子：mkfs 之后 S = 4 第一次挂载（第一个 txg 1）只收 txg 0 落到的三槽之外、[0, 1) 里落得到的槽：一个也没有。
    #[test]
    fn expected_bad_slot_txgs_take_only_slots_reachable_below_the_first_txg() {
        let ring = ring_of(4, &[0, 1, 2, 3, 4], &[5, 6], RingSlotReading::BadNotAllZero);
        let expected = expected_bad_slot_txgs(&ring, &BTreeSet::new(), 7, 4);
        assert_eq!(
            expected.get(&ring_slot_of_txg(5, 4)),
            Some(&5),
            "txg 5 那一槽在 (N − 12, 6] 里只落 txg 5"
        );
        assert_eq!(
            expected.get(&ring_slot_of_txg(6, 4)),
            Some(&6),
            "txg 6 那一槽"
        );
        let right_after_make_filesystem = ring_of(4, &[0], &[], RingSlotReading::AllZero);
        assert!(
            expected_bad_slot_txgs(&right_after_make_filesystem, &BTreeSet::new(), 1, 4).is_empty(),
            "[0, 1) 只落 txg 0"
        );
    }

    /// 算术模型（S9）在「每次挂载发 3 次、不带故障、S = 4」上给 11, 9, 9, 9, 9, 9, 9, 9（第七节，`calc/jia_skip_trajectory.py` 第一行）。
    #[test]
    fn the_skip_model_gives_the_registered_trajectory_for_three_publishes_per_mount() {
        // 模型只看每次挂载做没做成与发了几次，写行 txg 由模型自己算；这里的 `row_txg` 只表示「做成了」。
        let three_publishes_per_mount: Vec<CycleReading> = (0..MOUNTS_OF_A_FAMILY_HISTORY)
            .map(|_| CycleReading {
                outcome: "ok".to_string(),
                row_txg: Some(0),
                first_txg_of_today: 0,
                publishes_of_the_mount: 3,
                overwrites_done: 0,
                unmount: "nothing_written".to_string(),
                unmount_publishes: None,
            })
            .collect();
        let skips = model_skips_of_the_cycles(
            &three_publishes_per_mount,
            4,
            Arm::PastReadBadOrAllZero,
            false,
        );
        assert_eq!(skips, vec![11, 9, 9, 9, 9, 9, 9, 9]);
    }

    fn row_txg_under(
        image: &MemoryPool,
        faults: &[ReadFault],
        parameters: &MakeFilesystemParameters,
        arm: Arm,
    ) -> Option<u64> {
        let mut devices = devices_of(image, faults);
        mount_under_arm(parameters, &mut devices, arm)
            .ok()
            .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0)
    }

    /// 经真挂载走到开关的三档（第七节锚点，S = 4）：mkfs 之后 1 / 1 / 1 / 12；起点镜像只坏 txg 5 那一槽 7 / 7 / 18 / 18。
    #[test]
    fn switch_arms_take_the_anchor_row_txgs_right_after_make_filesystem_and_on_the_start_image() {
        let parameters = pool_parameters(4);
        let image = image_right_after_make_filesystem(&parameters);
        let arms = [
            Arm::Today,
            Arm::TodayViaSwitch,
            Arm::PastReadBad,
            Arm::PastReadBadOrAllZero,
        ];
        let right_after: Vec<Option<u64>> = arms
            .iter()
            .map(|arm| row_txg_under(&image, &[], &parameters, *arm))
            .collect();
        assert_eq!(
            right_after,
            vec![Some(1), Some(1), Some(1), Some(12)],
            "mkfs 之后 S = 4"
        );
        let start = build_starting_history(4);
        let (device, offset) = root_slot_place(&start.parameters, TXG_OF_SECOND_OVERWRITE);
        let faults = [ReadFault {
            device,
            offset,
            length: ROOT_SLOT_BYTES,
            kind: ReadFaultKind::DeviceError,
        }];
        let on_the_start: Vec<Option<u64>> = arms
            .iter()
            .map(|arm| row_txg_under(&start.image, &faults, &start.parameters, *arm))
            .collect();
        assert_eq!(
            on_the_start,
            vec![Some(7), Some(7), Some(18), Some(18)],
            "起点镜像只坏 txg 5 那一槽，S = 4"
        );
        let before = readings_before_the_mount(&start.image, &faults, &start.parameters);
        assert_eq!(
            (
                before.first_txg_of_today,
                before.first_txg_past_read_bad,
                before.first_txg_past_read_bad_or_all_zero
            ),
            (7, 18, 18),
            "装置现算与开关写行同值（S6）"
        );
    }

    /// 之后那一次挂载不带故障：G 主点读回全零 × 读回全零格，件里报的 `mount_after` 与在同一张固定崩溃点镜像上不带故障另挂一次的结局相同。
    #[test]
    fn the_mount_after_of_a_six_cell_runs_without_the_read_faults() {
        let start = build_starting_history(8);
        let cell = FaultCell {
            root_fault: ReadFaultKind::AllZero,
            system_configuration_fault: Some(ReadFaultKind::AllZero),
        };
        let job = CellJob {
            geometry: CELL_GEOMETRIES[0],
            cell,
            arm: Arm::Today,
        };
        let result = judge_a_cell_job(&start, job);
        let faults = fault_ranges_of(&start, 2, cell);
        let mount = recorded_mount(&start.image, &faults, &start.parameters, Arm::Today);
        let (image, _) = image_at_the_fixed_crash_point(&start.image, &mount)
            .expect("写行那一次做成、有第二次发布的根写");
        let (expected, _, _) = mount_after(
            &image,
            &[],
            &start.parameters,
            Arm::Today,
            HIGHEST_CONTENT_INDEX_OF_THE_START,
        );
        let fixed_line = result
            .lines
            .iter()
            .find(|line| line.contains("crash_point=fixed"))
            .expect("有固定崩溃点那一行");
        assert!(
            fixed_line.contains(&format!("mount_after={expected} ")),
            "件里报的 mount_after 是不带故障那一次：{fixed_line}"
        );
        let with_faults = mount_after(
            &image,
            &faults,
            &start.parameters,
            Arm::Today,
            HIGHEST_CONTENT_INDEX_OF_THE_START,
        )
        .0;
        assert_ne!(
            with_faults, expected,
            "这个取样点分得出带不带故障（不同才证得了上一条）"
        );
    }

    /// 单故障族判盘面后果时故障已撤：起点镜像 S = 8 只坏区域 2 槽 2（今天那一次挂载暖机写 txg 8 的那一槽），挂载写过它，
    /// 撤掉故障之后择到 (2, 8)；带着故障判会读不到它、择到 (2, 7)。
    #[test]
    fn one_single_fault_state_is_judged_with_the_fault_lifted() {
        let start = build_starting_history(8);
        let history = family_history_of_the_start_image(&start);
        let job = SkipStateJob {
            history: history.specification(),
            family: "single_fault",
            persisted_writes: 0,
            persisted_stream_end: 0,
            fault: Some((
                RootRingSlot { region: 2, slot: 2 },
                ReadFaultKind::DeviceError,
            )),
        };
        let result = judge_a_skip_state(&history, job);
        assert_eq!(
            (result.skip, result.chosen),
            (Some(0), Some((2, 8))),
            "写行 txg 7、暖机 txg 8，撤掉故障择到 (2, 8)"
        );
    }

    /// 起点镜像当一条没有写流的族历史（丙那一档）：第三段一态的判法拿它当取样点。
    fn family_history_of_the_start_image(start: &StartingHistory) -> FamilyHistory {
        FamilyHistory {
            slots_per_region: slots_per_region_of(&start.parameters),
            overwrites: 0,
            arm: Arm::Today,
            with_the_persistent_fault: false,
            parameters: start.parameters.clone(),
            base: start.image.clone(),
            writes: Vec::new(),
            segments: Vec::new(),
            stream_indexes: Vec::new(),
            barrier_stream_indexes: Vec::new(),
            confirmed: CONFIRMED_BEFORE_THE_SIX_CELL_MOUNT.to_vec(),
            highest_content_index: HIGHEST_CONTENT_INDEX_OF_THE_START,
            cycles: Vec::new(),
            stream_cut_after_the_taken_mounts: 0,
            images_at_the_mount_boundaries: Vec::new(),
        }
    }

    /// 第三段的 S3（登记第十一节）：起点镜像 S = 8 只坏 txg 5 那一槽（设备报读错，今天那一次挂载不写它），
    /// 环里没有被抛弃的根，装置现算的读坏槽能住的最大 txg（那一槽住 txg 5）与分配器交回的相等。
    #[test]
    fn one_single_fault_state_compares_the_bad_slot_txgs_with_the_allocator_and_finds_them_equal() {
        let start = build_starting_history(8);
        assert_eq!(
            abandoned_roots_on_the_ring_before_the_mount(
                &FaultyImage {
                    image: &start.image,
                    faults: &[],
                },
                &start.parameters,
            ),
            Some(0),
            "起点历史是一条时间线，环里没有被抛弃的根"
        );
        let history = family_history_of_the_start_image(&start);
        let job = SkipStateJob {
            history: history.specification(),
            family: "single_fault",
            persisted_writes: 0,
            persisted_stream_end: 0,
            fault: Some((
                ring_slot_of_txg(TXG_OF_SECOND_OVERWRITE, 8),
                ReadFaultKind::DeviceError,
            )),
        };
        let result = judge_a_skip_state(&history, job);
        assert_eq!(
            (
                result.bad_slot_txgs_against_the_allocator,
                result.stop_mismatches.clone()
            ),
            ("equal", Vec::<String>::new()),
            "S3 比了且相等：{result:?}"
        );
    }

    /// 第二段「乙离线出路」：起点镜像 S = 4 上翻坏盘 0 最新那一槽系统配置（乙直拒拒），照 C596 的离线出路断言恢复择到的那一版（D，txg 6）
    /// 走一次，只重写那一槽；重写之后乙两种读法都不拒。
    #[test]
    fn the_offline_way_out_rewrites_the_corrupted_newest_slot_so_neither_yi_reading_refuses() {
        let start = build_starting_history(4);
        let mut image = start.image.clone();
        corrupt_the_newest_system_configuration_slot(&mut image, DeviceIdentity(0));
        let before = system_configuration_slot_readings(&FaultyImage {
            image: &image,
            faults: &[],
        });
        assert!(
            yi_refuses(YiReading::RefuseDirectly, &before),
            "翻坏之后乙直拒拒：{before:?}"
        );
        let (outcome, rewritten) = offline_way_out(&image, &start.parameters);
        assert_eq!(
            outcome,
            format!("rewritten:asserted=(1,{TXG_OF_THIRD_OVERWRITE}):slots=1"),
            "断言 D、只重写翻坏的那一槽"
        );
        assert_eq!(
            yi_refusals_after_the_offline_way_out(rewritten.as_ref()),
            Some(vec![
                (YiReading::RefuseDirectly, false),
                (YiReading::IntoUndecidable, false)
            ]),
            "重写之后乙两种读法都不拒"
        );
    }

    // ---- 逐格判定入库（十三节）：内存假库上的复用三问、红表、对拍、拒算与存法 ----

    /// 入库单测的一格判定：只有一行产物与红不红。
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ProbeVerdict {
        line: String,
        red: bool,
    }

    impl StoredCellVerdict for ProbeVerdict {
        fn verdict_fields(&self) -> VerdictFields {
            let mut fields = VerdictFields::empty();
            fields.push("line", &self.line);
            fields.push("red", &self.red.to_string());
            fields
        }

        fn from_stored_fields(stored: &StoredFields) -> Option<ProbeVerdict> {
            Some(ProbeVerdict {
                line: stored.single("line")?.to_string(),
                red: bool_from_text(stored.single("red")?)?,
            })
        }

        fn is_red(&self) -> bool {
            self.red
        }
    }

    const PROBE_FAULT_JOBS: [u64; 4] = [1, 2, 3, 4];
    const PROBE_CONTROL_JOBS: [u64; 3] = [1, 2, 3];

    /// 单测造的组成清单：每个入口一行 code_key，另带取键那天、机器与一行工具链（后三样不进格键）。
    fn probe_composition(code_keys: &[(CellEntry, &str)], judged: &str, host: &str) -> String {
        let mut text = format!("{COMPOSITION_FORM_LINE}\njudged\t{judged}\nhost\t{host}\n");
        for (entry, code_key) in code_keys {
            text.push_str(&format!(
                "code_key\te205_c971_inverted_first_txg_arm_costs::{}\t{code_key}\n",
                entry.function_name()
            ));
        }
        text.push_str("toolchain\trustc -vV: host: x86_64-unknown-linux-gnu\n");
        text
    }

    fn probe_keying(store: &DeviceLibrary, composition: &str, entries: &[CellEntry]) -> CellKeying {
        CellKeying::Library(
            cell_library_from_composition(store.clone(), composition, entries)
                .expect("单测造的组成清单合式"),
        )
    }

    /// 一趟判下来的：故障格与对照格各自的格数、两类按工作次序合起来的产物行、单线程对拍那一条与它是现判还是复用。
    struct ProbePass {
        fault: CellCounts,
        control: CellCounts,
        lines: Vec<String>,
        cross_check: ThreadCrossCheckJudgement,
        cross_check_outcome: ThreadCrossCheckOutcome,
    }

    /// 故障格四件、对照格三件与单线程对拍那一条照第一段的判法判一趟（`judge_segment_one_through_the_library`），两类格用同一个判法。
    fn judge_probe_entries_with(
        keying: &CellKeying,
        threads: usize,
        judge: impl Fn(&u64) -> ProbeVerdict + Sync,
    ) -> ProbePass {
        let judged = judge_segment_one_through_the_library(
            keying,
            &PROBE_FAULT_JOBS,
            &PROBE_CONTROL_JOBS,
            threads,
            &judge,
            &judge,
        );
        let lines = judged
            .fault
            .verdicts
            .iter()
            .chain(&judged.control.verdicts)
            .map(|verdict| verdict.line.clone())
            .collect();
        ProbePass {
            fault: judged.fault.counts,
            control: judged.control.counts,
            lines,
            cross_check: judged.cross_check,
            cross_check_outcome: judged.cross_check_outcome,
        }
    }

    /// 每一格都判成绿、一行 `cell=<件>` 的那一种判法。
    fn judge_probe_entries(keying: &CellKeying, threads: usize) -> ProbePass {
        judge_probe_entries_with(keying, threads, |job: &u64| ProbeVerdict {
            line: format!("cell={job}"),
            red: false,
        })
    }

    /// 第一段三个入口的组成清单：故障格入口的代码键固定 a1，对照格入口与单线程对拍那一条入口的代码键、取键那天与机器由参数给
    /// （对拍那一条的闭包含两个格入口，格入口闭包里的改动也改它的代码键）。
    fn six_cells_probe_keying(
        store: &DeviceLibrary,
        control_code_key: &str,
        cross_check_code_key: &str,
        judged: &str,
        host: &str,
    ) -> CellKeying {
        probe_keying(
            store,
            &probe_composition(
                &[
                    (CellEntry::FaultCell, "a1"),
                    (CellEntry::ControlCell, control_code_key),
                    (CellEntry::SegmentOneThreadCrossCheck, cross_check_code_key),
                ],
                judged,
                host,
            ),
            &[
                CellEntry::FaultCell,
                CellEntry::ControlCell,
                CellEntry::SegmentOneThreadCrossCheck,
            ],
        )
    }

    /// 复用三问之一：同一份库上同一份输入跑两趟；交回两趟与库里存下的格数。
    fn passes_with_the_same_input() -> (ProbePass, ProbePass, usize) {
        let store = DeviceLibrary::in_memory();
        let keying = six_cells_probe_keying(&store, "b2", "c1", "2026-11-06", "本机");
        let first = judge_probe_entries(&keying, 4);
        let second = judge_probe_entries(&keying, 4);
        (first, second, store.count_entries("items/", ".out"))
    }

    /// 复用三问之二：头一趟之后改一处不在任何格闭包里的输入（代码键不变；取键那天、机器、线程数变了），交回再跑的那一趟。
    fn pass_after_a_change_outside_every_cell_closure() -> ProbePass {
        let store = DeviceLibrary::in_memory();
        judge_probe_entries(
            &six_cells_probe_keying(&store, "b2", "c1", "2026-11-06", "本机"),
            4,
        );
        judge_probe_entries(
            &six_cells_probe_keying(&store, "b2", "c1", "2026-11-07", "第二台"),
            1,
        )
    }

    /// 复用三问之三：头一趟之后改对照格入口闭包里的一个单元（那个入口与闭包含它的单线程对拍那一条的代码键变），交回再跑的那一趟。
    fn pass_after_a_change_inside_the_control_entry_closure() -> ProbePass {
        let store = DeviceLibrary::in_memory();
        judge_probe_entries(
            &six_cells_probe_keying(&store, "b2", "c1", "2026-11-06", "本机"),
            4,
        );
        judge_probe_entries(
            &six_cells_probe_keying(&store, "b3", "c2", "2026-11-06", "本机"),
            4,
        )
    }

    const PROBE_LINES: [&str; 7] = [
        "cell=1", "cell=2", "cell=3", "cell=4", "cell=1", "cell=2", "cell=3",
    ];

    /// 复用三问之一：第二趟现判 0、复用的恰好是头一趟现判的那七格，产物按工作的次序不变。
    #[test]
    fn cells_judged_again_with_the_same_input_are_all_reused() {
        let (first, second, records) = passes_with_the_same_input();
        assert_eq!(
            (
                first.fault.fresh,
                first.fault.reused,
                first.control.fresh,
                first.control.reused
            ),
            (4, 0, 3, 0),
            "头一趟全部现判"
        );
        assert_eq!(
            (
                second.fault.fresh,
                second.fault.reused,
                second.control.fresh,
                second.control.reused
            ),
            (0, 4, 0, 3),
            "第二趟现判 0、复用头一趟现判的那七格"
        );
        assert_eq!(records, 8, "七格各一条，单线程对拍一条");
        assert_eq!(first.lines, PROBE_LINES, "头一趟按工作的次序");
        assert_eq!(second.lines, first.lines, "复用的产物与现判的逐字相同");
    }

    /// 复用三问之二：闭包外的改动之后现判 0。
    #[test]
    fn one_change_outside_every_cell_closure_judges_no_cell_again() {
        let pass = pass_after_a_change_outside_every_cell_closure();
        assert_eq!(
            (
                pass.fault.fresh,
                pass.control.fresh,
                pass.fault.reused + pass.control.reused
            ),
            (0, 0, 7),
            "闭包外的改动不让任何一格现判"
        );
        assert_eq!(pass.lines, PROBE_LINES, "复用的产物按工作的次序");
    }

    /// 复用三问之三：现判的恰好是对照格那三格，故障格全部复用。
    #[test]
    fn one_change_inside_one_entry_closure_judges_only_the_cells_of_that_entry_again() {
        let pass = pass_after_a_change_inside_the_control_entry_closure();
        assert_eq!(
            (
                pass.fault.fresh,
                pass.fault.reused,
                pass.control.fresh,
                pass.control.reused
            ),
            (0, 4, 3, 0),
            "只现判对照格那三格"
        );
        assert_eq!(pass.lines, PROBE_LINES, "现判的与复用的按工作的次序合起来");
    }

    /// 复用三问的规范输出（`scripts/gate/lib_reuse_checks.py` 文件头「一」；门禁 code-tooling 的 reuse-selftests 格只认这一行）：
    /// 三问照上面三条用例的取法各问一次，哪一问不成立写 red。
    #[test]
    fn reuse_three_questions_print_the_canonical_line() {
        let (first, second, _) = passes_with_the_same_input();
        let first_fresh = first.fault.fresh + first.control.fresh;
        let same_inputs = if first_fresh > 0
            && second.fault.fresh + second.control.fresh == 0
            && second.fault.reused + second.control.reused == first_fresh
            && first.cross_check_outcome == ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch
            && second.cross_check_outcome == ThreadCrossCheckOutcome::Reused
        {
            "reused"
        } else {
            "red"
        };
        let outside = pass_after_a_change_outside_every_cell_closure();
        let unrelated_change = if outside.fault.fresh + outside.control.fresh == 0
            && outside.cross_check_outcome == ThreadCrossCheckOutcome::Reused
        {
            "reused"
        } else {
            "red"
        };
        // 改判法的一个单元（对照格入口闭包里的），单线程对拍那一条要现判：它是 affected 里的一项。
        let inside = pass_after_a_change_inside_the_control_entry_closure();
        let unit_change = if inside.fault.fresh == 0
            && inside.fault.reused == PROBE_FAULT_JOBS.len()
            && inside.control.fresh == PROBE_CONTROL_JOBS.len()
            && inside.control.reused == 0
            && inside.cross_check_outcome == ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch
        {
            "rejudged-only-affected"
        } else {
            "red"
        };
        for (pass_name, pass) in [
            ("same-inputs 第二趟", &second),
            ("unrelated-change", &outside),
            ("unit-change", &inside),
        ] {
            println!(
                "  · 第一段单线程对拍（{pass_name}）：{}，{}",
                if pass.cross_check.is_identical() {
                    "绿"
                } else {
                    "红"
                },
                probe_cross_check_state(pass.cross_check_outcome)
            );
        }
        println!(
            "REUSE_SELFTEST script=crates/singlefs-verification/src/bin/e205_c971_inverted_first_txg_arm_costs.rs same-inputs={same_inputs} unrelated-change={unrelated_change} unit-change={unit_change} items={} affected={}",
            PROBE_FAULT_JOBS.len() + PROBE_CONTROL_JOBS.len() + 1,
            PROBE_CONTROL_JOBS.len() + 1
        );
        assert_eq!(
            (same_inputs, unrelated_change, unit_change),
            ("reused", "reused", "rejudged-only-affected"),
            "复用三问都过"
        );
    }

    /// 对拍那一条的键收代码键：头一趟在代码键 c1 下存下，第二趟代码键换成 c2（现判并存下自己的一条），第三趟代码键回到 c1 时
    /// 头一趟那一条还在、复用；库里对拍那一条共两条（每个代码键各一条）。键不收代码键时第二趟的存盘盖掉头一趟那一条，第三趟只能现判、库里只剩一条。
    #[test]
    fn cross_check_record_of_one_code_key_survives_a_pass_under_another_code_key() {
        let store = DeviceLibrary::in_memory();
        let judge_under = |cross_check_code_key: &str| {
            judge_probe_entries(
                &six_cells_probe_keying(&store, "b2", cross_check_code_key, "2026-11-06", "本机"),
                4,
            )
        };
        let first = judge_under("c1");
        let second = judge_under("c2");
        let records_after_second = store.count_entries("items/", ".out");
        let third = judge_under("c1");
        assert_eq!(
            first.cross_check_outcome,
            ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch
        );
        assert_eq!(
            second.cross_check_outcome,
            ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch,
            "代码键换了，对拍那一条现判"
        );
        assert_eq!(
            records_after_second, 9,
            "七格各一条、对拍两条（c1、c2 各一条）"
        );
        assert_eq!(
            third.cross_check_outcome,
            ThreadCrossCheckOutcome::Reused,
            "代码键回到 c1，头一趟那一条还在、复用"
        );
    }

    /// 判红的格记进红表、不存绿的；下一趟照判它（其余复用）。
    #[test]
    fn one_red_cell_goes_into_the_red_list_and_is_judged_again_on_the_next_pass() {
        let store = DeviceLibrary::in_memory();
        let keying = probe_keying(
            &store,
            &probe_composition(&[(CellEntry::FaultCell, "a1")], "2026-11-06", "本机"),
            &[CellEntry::FaultCell],
        );
        let judge_with_cell_two_red = |job: &u64| ProbeVerdict {
            line: format!("cell={job}"),
            red: *job == 2,
        };
        let judge_pass = || {
            judge_cells_through_the_library(
                &keying,
                CellEntry::FaultCell,
                &PROBE_FAULT_JOBS,
                |job| format!("{job:?}"),
                2,
                ThreadCrossCheck::StoreRightAfterJudging,
                judge_with_cell_two_red,
            )
            .counts
        };
        let first = judge_pass();
        assert_eq!((first.fresh, first.red), (4, 1), "头一趟四格现判、一格红");
        assert_eq!(
            store.count_entries("items/", ".out"),
            3,
            "红的那一格不存绿的"
        );
        assert_eq!(store.count_entries("red/", ".tsv"), 1, "红表写了");
        let second = judge_pass();
        assert_eq!(
            (second.fresh, second.reused, second.red),
            (1, 3, 1),
            "记着红的照判，其余复用"
        );
    }

    /// 红表盖住库里原有的绿：记着红的格照判、不复用；这一趟判绿之后从红表里撤掉，再下一趟复用。
    #[test]
    fn one_cell_in_the_red_list_is_judged_again_even_with_a_green_record_in_the_library() {
        let store = DeviceLibrary::in_memory();
        let keying = probe_keying(
            &store,
            &probe_composition(&[(CellEntry::FaultCell, "a1")], "2026-11-06", "本机"),
            &[CellEntry::FaultCell],
        );
        let judge_pass = || {
            judge_cells_through_the_library(
                &keying,
                CellEntry::FaultCell,
                &PROBE_FAULT_JOBS,
                |job| format!("{job:?}"),
                2,
                ThreadCrossCheck::StoreRightAfterJudging,
                |job: &u64| ProbeVerdict {
                    line: format!("cell={job}"),
                    red: false,
                },
            )
            .counts
        };
        judge_pass();
        store.write(
            &red_list_path("a1"),
            &format!("{}\t1\n", cell_key("a1", "1")),
        );
        let shadowed = judge_pass();
        assert_eq!(
            (shadowed.fresh, shadowed.reused),
            (1, 3),
            "红表里那一格照判"
        );
        let cleared = judge_pass();
        assert_eq!(
            (cleared.fresh, cleared.reused),
            (0, 4),
            "判绿之后撤出红表，再一趟全复用"
        );
    }

    /// 第一段的单线程与多线程对拍：两遍判得不一样的格记红、不存，对拍那一项报不同。
    #[test]
    fn one_cell_judged_differently_on_one_thread_is_red_and_not_stored() {
        let store = DeviceLibrary::in_memory();
        let keying = probe_keying(
            &store,
            &probe_composition(&[(CellEntry::ControlCell, "b2")], "2026-11-06", "本机"),
            &[CellEntry::ControlCell],
        );
        let calls = AtomicUsize::new(0);
        let judged = judge_cells_through_the_library(
            &keying,
            CellEntry::ControlCell,
            &[7u64],
            |job| format!("{job:?}"),
            1,
            ThreadCrossCheck::JudgeAgainOnOneThreadBeforeStoring,
            |job: &u64| ProbeVerdict {
                line: format!("cell={job} call={}", calls.fetch_add(1, Ordering::SeqCst)),
                red: false,
            },
        );
        assert_eq!(
            judged.differing_parameters,
            vec!["7".to_string()],
            "两遍判得不同"
        );
        assert_eq!(judged.counts.red, 1, "对不上的那一格记红");
        assert_eq!(store.count_entries("items/", ".out"), 0, "对不上的不存");
    }

    /// 键算不出的一趟：每一格照判、记成拒算，库里一条都不写。
    #[test]
    fn without_a_code_key_every_cell_is_judged_and_counted_as_refused() {
        let judged = judge_cells_through_the_library(
            &CellKeying::KeyRefused,
            CellEntry::FaultCell,
            &PROBE_FAULT_JOBS,
            |job| format!("{job:?}"),
            2,
            ThreadCrossCheck::StoreRightAfterJudging,
            |job: &u64| ProbeVerdict {
                line: format!("cell={job}"),
                red: false,
            },
        );
        assert_eq!(
            judged.counts,
            CellCounts {
                fresh: 0,
                reused: 0,
                refused: 4,
                red: 0
            },
            "四格拒算"
        );
    }

    /// 格键只由代码键与参数定。
    #[test]
    fn the_cell_key_takes_only_the_code_key_and_the_parameters() {
        assert_ne!(cell_key("a1", "1"), cell_key("a2", "1"), "代码键变了键变");
        assert_ne!(cell_key("a1", "1"), cell_key("a1", "2"), "参数变了键变");
    }

    /// 存回去读得回：值里的换行、制表符、反斜杠照原样；键行、参数行对不上或行数对不上的读成不在。
    #[test]
    fn one_stored_record_reads_back_only_under_its_own_key_and_whole() {
        let verdict = ProbeVerdict {
            line: "a\tb\nc\\d\re".to_string(),
            red: false,
        };
        let lines = verdict.verdict_fields().lines;
        let text = cell_record_text("k1", "a1", "p\t1", "meta judged=- host=-", &lines);
        let read_back = verdict_lines_read_back(Some(text.clone()), "k1", "a1", "p\t1")
            .expect("自己的键读得回");
        assert_eq!(
            ProbeVerdict::from_stored_fields(&StoredFields::parse(&read_back).expect("字段行合式")),
            Some(verdict),
            "转义来回逐字相同"
        );
        assert_eq!(
            verdict_lines_read_back(Some(text.clone()), "k1", "a2", "p\t1"),
            None,
            "代码键对不上读成不在"
        );
        let truncated: String = text
            .lines()
            .filter(|line| !line.starts_with("red\t"))
            .map(|line| format!("{line}\n"))
            .collect();
        assert_eq!(
            verdict_lines_read_back(Some(truncated), "k1", "a1", "p\t1"),
            None,
            "少了一行读成不在"
        );
    }

    /// 组成清单要恰好这一模式那几个入口各一行 code_key。
    #[test]
    fn the_composition_must_name_exactly_the_entries_of_the_mode() {
        let store = DeviceLibrary::in_memory();
        let six_cells = [CellEntry::FaultCell, CellEntry::ControlCell];
        assert!(
            cell_library_from_composition(
                store.clone(),
                &probe_composition(&[(CellEntry::FaultCell, "a1")], "-", "-"),
                &six_cells
            )
            .is_err(),
            "缺对照格入口"
        );
        assert!(
            cell_library_from_composition(
                store.clone(),
                &probe_composition(
                    &[
                        (CellEntry::FaultCell, "a1"),
                        (CellEntry::ControlCell, "b2"),
                        (CellEntry::TornState, "c3")
                    ],
                    "-",
                    "-"
                ),
                &six_cells
            )
            .is_err(),
            "多一个入口"
        );
        let library = cell_library_from_composition(
            store,
            &probe_composition(
                &[(CellEntry::ControlCell, "b2"), (CellEntry::FaultCell, "a1")],
                "2026-11-06",
                "本机",
            ),
            &six_cells,
        )
        .expect("两个入口各一行");
        assert_eq!(
            (
                library
                    .code_keys
                    .get(&CellEntry::FaultCell)
                    .map(String::as_str),
                library
                    .code_keys
                    .get(&CellEntry::ControlCell)
                    .map(String::as_str)
            ),
            (Some("a1"), Some("b2")),
            "按入口名取代码键，不按行序"
        );
        assert_eq!(library.meta_line, "meta judged=2026-11-06 host=本机");
    }

    /// 四种格判定各存一次读一次，逐字段相同。
    #[test]
    fn every_cell_verdict_reads_back_equal_to_what_was_stored() {
        let reading = ImageReading {
            roots: vec![(0, 0), (1, 6), (2, 5)],
            records: Vec::new(),
            roots_of_two_instances_at_one_txg: false,
            instance_and_txg_order_crossed_on_roots: true,
            instance_and_txg_order_crossed_on_roots_and_records: true,
            inverted: true,
            violated: vec!["I-3.1".to_string(), "I-8.14".to_string()],
            chosen: Some((1, 6)),
            content_index: None,
            content: "unreadable\tx".to_string(),
        };
        let fault = FaultCellJudgement {
            lines: vec!["name=cell one".to_string(), "name=cell two".to_string()],
            refusal: Some("NewerStateStillUnreadableAfterOneReread".to_string()),
            row_txg: Some(5),
            yi_refuses: vec![
                (YiReading::RefuseDirectly, true),
                (YiReading::IntoUndecidable, false),
            ],
            fixed: Some(reading),
            fixed_lost: true,
            fixed_mount_after_refused: false,
            stop_mismatches: vec!["S3 differs".to_string()],
            voided: None,
        };
        let round_trip_fault = FaultCellJudgement::from_stored_fields(
            &StoredFields::parse(&fault.verdict_fields().lines).expect("字段行合式"),
        );
        assert_eq!(round_trip_fault, Some(fault), "六格一件");
        let control = ControlJudgement {
            line: "name=negative_control control=as_expected".to_string(),
            passed: true,
        };
        let round_trip_control = ControlJudgement::from_stored_fields(
            &StoredFields::parse(&control.verdict_fields().lines).expect("字段行合式"),
        );
        assert_eq!(round_trip_control, Some(control), "对照一件");
        let torn: TornStateResult = (
            8,
            "acquisition",
            true,
            vec![
                (YiReading::RefuseDirectly, true),
                (YiReading::IntoUndecidable, true),
            ],
            false,
            "name=torn_state".to_string(),
            "rewritten:asserted=(1,6):slots=1".to_string(),
            Some(vec![
                (YiReading::RefuseDirectly, false),
                (YiReading::IntoUndecidable, false),
            ]),
        );
        let round_trip_torn = TornStateResult::from_stored_fields(
            &StoredFields::parse(&torn.verdict_fields().lines).expect("字段行合式"),
        );
        assert_eq!(round_trip_torn, Some(torn), "撕裂一态");
        let skip = SkipStateResult {
            refusal: None,
            skip: Some(17),
            red: 1,
            violated: vec!["I-8.14".to_string()],
            lost: false,
            chosen: Some((2, 8)),
            bad_slot_txgs_against_the_allocator: "not_compared_abandoned_roots_on_the_ring",
            stop_mismatches: Vec::new(),
            voided: Some("voided because\nof a reason".to_string()),
        };
        let round_trip_skip = SkipStateResult::from_stored_fields(
            &StoredFields::parse(&skip.verdict_fields().lines).expect("字段行合式"),
        );
        assert_eq!(round_trip_skip, Some(skip), "跳号一态");
    }

    /// 判红的那几种：停机对拍有不等、态作废、对照没过；撕裂一态不判红。
    #[test]
    fn stop_mismatches_voided_states_and_failed_controls_are_red() {
        let with_stop = FaultCellJudgement {
            stop_mismatches: vec!["S1_or_S6".to_string()],
            ..FaultCellJudgement::default()
        };
        let voided = FaultCellJudgement {
            voided: Some("unreadable".to_string()),
            ..FaultCellJudgement::default()
        };
        assert!(
            with_stop.is_red() && voided.is_red(),
            "六格：停机对拍不等、作废都红"
        );
        assert!(!FaultCellJudgement::default().is_red(), "六格：都没有不红");
        let failed = ControlJudgement {
            line: String::new(),
            passed: false,
        };
        assert!(failed.is_red(), "对照没过红");
        let skip_with_stop = SkipStateResult {
            refusal: None,
            skip: None,
            red: 0,
            violated: Vec::new(),
            lost: false,
            chosen: None,
            bad_slot_txgs_against_the_allocator: "differs",
            stop_mismatches: vec!["S3".to_string()],
            voided: None,
        };
        assert!(skip_with_stop.is_red(), "跳号一态：停机对拍不等红");
    }

    // ---- 第一段单线程对拍那一条（十三节「第一段单线程对拍那一条」；登记第十二节修订十二） ----

    /// 单线程对拍那一条这一趟的结局写法（复用三问的结局行用）。
    fn probe_cross_check_state(outcome: ThreadCrossCheckOutcome) -> &'static str {
        match outcome {
            ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch
            | ThreadCrossCheckOutcome::JudgedNowRedBefore => "现判",
            ThreadCrossCheckOutcome::Reused => "复用",
            ThreadCrossCheckOutcome::KeyRefused => "拒算",
        }
    }

    /// 每一格都判成绿、一行 `cell=<件>`，每判一遍记一次数。
    fn counted_probe_judge(calls: &AtomicUsize) -> impl Fn(&u64) -> ProbeVerdict + Sync + '_ {
        move |job: &u64| {
            calls.fetch_add(1, Ordering::SeqCst);
            ProbeVerdict {
                line: format!("cell={job}"),
                red: false,
            }
        }
    }

    /// 探针第一段那一条在库里的键（对拍入口的代码键与第一段全部的件的参数）。
    fn probe_cross_check_key(cross_check_code_key: &str) -> String {
        cell_key(
            cross_check_code_key,
            &segment_one_cross_check_parameters(&PROBE_FAULT_JOBS, &PROBE_CONTROL_JOBS),
        )
    }

    /// 库里那一条读回来的判定（读不回交 None）。
    fn probe_stored_cross_check(
        store: &DeviceLibrary,
        cross_check_code_key: &str,
    ) -> Option<ThreadCrossCheckJudgement> {
        let key = probe_cross_check_key(cross_check_code_key);
        let lines = verdict_lines_read_back(
            store.read(&cell_record_path(&key)),
            &key,
            cross_check_code_key,
            &segment_one_cross_check_parameters(&PROBE_FAULT_JOBS, &PROBE_CONTROL_JOBS),
        )?;
        ThreadCrossCheckJudgement::from_stored_fields(&StoredFields::parse(&lines)?)
    }

    /// 键对不上：格都复用了，对拍那一条照样把第一段全部的件切片多线程判一遍、单线程再判一遍，逐字相同记绿、写进库。
    #[test]
    fn one_thread_cross_check_whose_key_does_not_match_judges_every_job_again_on_one_thread() {
        assert_eq!(
            cell_entries_of_the_mode(&["six-cells"]),
            &[
                CellEntry::FaultCell,
                CellEntry::ControlCell,
                CellEntry::SegmentOneThreadCrossCheck
            ],
            "six-cells 的组成清单要有对拍那一条的入口"
        );
        let store = DeviceLibrary::in_memory();
        judge_probe_entries(
            &six_cells_probe_keying(&store, "b2", "c1", "2026-11-06", "本机"),
            4,
        );
        let calls = AtomicUsize::new(0);
        let pass = judge_probe_entries_with(
            &six_cells_probe_keying(&store, "b2", "c9", "2026-11-06", "本机"),
            4,
            counted_probe_judge(&calls),
        );
        assert_eq!(
            (
                pass.fault.fresh + pass.control.fresh,
                pass.fault.reused + pass.control.reused
            ),
            (0, 7),
            "格的代码键没变，七格复用"
        );
        assert_eq!(
            pass.cross_check_outcome,
            ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch,
            "对拍那一条的键对不上，现判"
        );
        assert_eq!(
            calls.load(Ordering::SeqCst),
            14,
            "第一段七件切片多线程判一遍、单线程再判一遍"
        );
        let identical = ThreadCrossCheckJudgement {
            compared_jobs: 7,
            differing_jobs: Vec::new(),
        };
        assert_eq!(pass.cross_check, identical, "七件逐字相同");
        assert_eq!(
            probe_stored_cross_check(&store, "c9"),
            Some(identical),
            "现判判绿写进库"
        );
    }

    /// 键对不上的那一趟全量对拍：单线程那一遍对不上、或切片多线程那一遍与产物用的（库里复用的）对不上，都判红。
    #[test]
    fn one_thread_cross_check_whose_key_does_not_match_is_red_when_one_thread_or_the_product_differs(
    ) {
        let store = DeviceLibrary::in_memory();
        judge_probe_entries(
            &six_cells_probe_keying(&store, "b2", "c1", "2026-11-06", "本机"),
            4,
        );
        // 格全复用，判法只被对拍那一条调：每件头一遍（切片多线程）照旧，第二遍（单线程）多一个字。
        let calls_of_each_job: Mutex<BTreeMap<u64, usize>> = Mutex::new(BTreeMap::new());
        let one_thread_differs = judge_probe_entries_with(
            &six_cells_probe_keying(&store, "b2", "c8", "2026-11-06", "本机"),
            4,
            |job: &u64| {
                let mut calls = calls_of_each_job.lock().expect("单测计数的锁没被毒");
                let call = calls.entry(*job).or_insert(0);
                *call += 1;
                ProbeVerdict {
                    line: if *call == 1 {
                        format!("cell={job}")
                    } else {
                        format!("cell={job} again")
                    },
                    red: false,
                }
            },
        );
        assert_eq!(
            one_thread_differs.cross_check.differing_jobs.len(),
            7,
            "七件单线程那一遍都对不上"
        );
        // 切片多线程与单线程两遍相同，可都与库里复用的那一份不同。
        let product_differs = judge_probe_entries_with(
            &six_cells_probe_keying(&store, "b2", "c7", "2026-11-06", "本机"),
            4,
            |job: &u64| ProbeVerdict {
                line: format!("cell={job} changed"),
                red: false,
            },
        );
        assert_eq!(
            product_differs.cross_check.differing_jobs.len(),
            7,
            "七件与产物用的那一份都对不上"
        );
    }

    /// 键相同：复用库里那一条，不重判（判法一遍都不调），读回的就是头一趟判的那一条，不是空真。
    #[test]
    fn one_thread_cross_check_with_the_same_key_is_reused_and_not_judged_again() {
        let store = DeviceLibrary::in_memory();
        let first = judge_probe_entries(
            &six_cells_probe_keying(&store, "b2", "c1", "2026-11-06", "本机"),
            4,
        );
        assert_eq!(
            first.cross_check_outcome,
            ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch,
            "头一趟库是空的，现判"
        );
        let calls = AtomicUsize::new(0);
        let second = judge_probe_entries_with(
            &six_cells_probe_keying(&store, "b2", "c1", "2026-11-07", "第二台"),
            1,
            counted_probe_judge(&calls),
        );
        assert_eq!(
            second.cross_check_outcome,
            ThreadCrossCheckOutcome::Reused,
            "键相同（取键那天、机器、线程数不进键），复用"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0, "不重判");
        assert_eq!(
            second.cross_check, first.cross_check,
            "复用的是库里比过七件的那一条"
        );
        assert_eq!(second.cross_check.compared_jobs, 7, "比过七件");
    }

    /// 判红记红、撤掉原有的绿：键相同而这一趟现判的一格两遍对不上，这一条判红、记进红表、库里那一条改写成红的；
    /// 下一趟照判，判绿之后撤出红表、写回绿的；红表里有它（库里那一条是绿的）也照判。
    #[test]
    fn one_thread_cross_check_judged_red_is_recorded_red_and_its_green_withdrawn() {
        let store = DeviceLibrary::in_memory();
        let keying = six_cells_probe_keying(&store, "b2", "c1", "2026-11-06", "本机");
        let green = judge_probe_entries(&keying, 4);
        assert!(green.cross_check.is_identical(), "头一趟绿");
        // 故障格那一件 1 的记录读不回（同哨兵抽中强制现判的那一格），这一趟它现判；判法每遍判得不同。
        store.write(&cell_record_path(&cell_key("a1", "1")), "读不回");
        let calls = AtomicUsize::new(0);
        let red = judge_probe_entries_with(&keying, 4, |job: &u64| ProbeVerdict {
            line: format!("cell={job} call={}", calls.fetch_add(1, Ordering::SeqCst)),
            red: false,
        });
        assert_eq!(
            red.cross_check_outcome,
            ThreadCrossCheckOutcome::Reused,
            "对拍那一条的键相同"
        );
        assert_eq!(
            red.cross_check.differing_jobs,
            vec!["fault 1".to_string()],
            "现判的那一格两遍对不上，这一条判红"
        );
        let key = probe_cross_check_key("c1");
        assert!(red_cells_of(&store, "c1").contains_key(&key), "记进红表");
        assert_eq!(
            probe_stored_cross_check(&store, "c1").map(|stored| stored.is_red()),
            Some(true),
            "库里原有的绿改写成红的"
        );
        let again = judge_probe_entries(&keying, 4);
        assert_eq!(
            (again.cross_check_outcome, again.cross_check.is_identical()),
            (ThreadCrossCheckOutcome::JudgedNowRedBefore, true),
            "记着红的照判，这一趟判绿"
        );
        assert!(
            !red_cells_of(&store, "c1").contains_key(&key),
            "判绿撤出红表"
        );
        assert_eq!(
            probe_stored_cross_check(&store, "c1").map(|stored| stored.is_red()),
            Some(false),
            "写回绿的"
        );
        store.write(&red_list_path("c1"), &format!("{key}\t-\n"));
        let listed = judge_probe_entries(&keying, 4);
        assert_eq!(
            listed.cross_check_outcome,
            ThreadCrossCheckOutcome::JudgedNowRedBefore,
            "红表里有它，库里那一条是绿的也照判"
        );
    }

    /// 成功句另报的那一行：写明这一条是现判还是复用、判绿还是判红。
    #[test]
    fn the_thread_cross_check_sentence_says_whether_it_was_judged_now_or_reused() {
        let identical = ThreadCrossCheckJudgement {
            compared_jobs: 7,
            differing_jobs: Vec::new(),
        };
        let differing = ThreadCrossCheckJudgement {
            compared_jobs: 7,
            differing_jobs: vec!["fault 1".to_string(), "control 3".to_string()],
        };
        assert_eq!(
            thread_cross_check_sentence(ThreadCrossCheckOutcome::JudgedNowKeyDidNotMatch, &identical),
            "e205：第一段单线程对拍（入口 segment_one_thread_cross_check_verdict）：现判（键对不上：第一段 7 件切片多线程判一遍、单线程再判一遍）；逐件逐字相同，记绿"
        );
        assert_eq!(
            thread_cross_check_sentence(ThreadCrossCheckOutcome::Reused, &identical),
            "e205：第一段单线程对拍（入口 segment_one_thread_cross_check_verdict）：复用（键相同，不重判；库里那一条比过第一段 7 件）；绿"
        );
        assert_eq!(
            thread_cross_check_sentence(ThreadCrossCheckOutcome::JudgedNowRedBefore, &differing),
            "e205：第一段单线程对拍（入口 segment_one_thread_cross_check_verdict）：现判（键相同而上一趟记了红：第一段 7 件切片多线程判一遍、单线程再判一遍）；2 件对不上，记红、撤掉原有的绿：fault 1；control 3"
        );
        assert_eq!(
            thread_cross_check_sentence(ThreadCrossCheckOutcome::KeyRefused, &identical),
            "e205：第一段单线程对拍（入口 segment_one_thread_cross_check_verdict）：拒算（没有代码键：第一段 7 件切片多线程判一遍、单线程再判一遍，不读不写库）；逐件逐字相同"
        );
    }
}
