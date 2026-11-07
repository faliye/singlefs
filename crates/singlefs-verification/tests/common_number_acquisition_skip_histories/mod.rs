//! 取号跳过那一判（I-3.5（引用区间的精确性） 的缺口条款，`crates/singlefs-checker/src/walk.rs` 的 `hole_in_the_candidate_set_between`）要的几段历史与镜像改法。
//! 几何与搭法照 E198（取号跳过认出生代号的前提） 装置 `crates/singlefs-verification/src/bin/e198_instance_table_pointer_birth_skip_premise.rs`
//! 另写一份（装置的搭法是那个二进制私有的）：两块 256 MiB 盘、每区 4 槽、1 MiB journal 环、区域到盘 [0, 1, 0]；每一步（mkfs、一次挂载、一次发布）
//! 在此刻的镜像上起一份录制盘、录下这一步的写流，再按次序施加回镜像；「崩」是只施加到那一步第一个根槽写之前。读坏一个根槽同装置：槽内第 40 字节取反。
//! 只搭镜像、不判：池级 checker 的 harness 用例与 GPU 对拍用例经 `#[path]` 共用这一份；它经 `crate::common` 用共用的那一份
//! （把根抄进 journal 记录的新根段），引它的测试文件同时引 `tests/common/mod.rs`。
#![allow(dead_code, reason = "每个引它的测试文件各自只用到其中一部分")]

use crate::common::{
    copy_each_root_into_the_new_root_segment_of_its_journal_records_on, PoolImageGeometry,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration,
    roll_back_by_a_forward_publish, MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{readable_roots, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_new_inodes, publish_overwrite, publish_overwrite_of_an_inode,
    FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED, UNIT_MAGIC};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, SLOT_BYTES};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

/// 每块盘的字节数（E198 登记第五节「几何」一段）。
pub const DEVICE_BYTES: u64 = 256 << 20;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
/// 根环每区的槽数：E198 主取样点 S = 4。
const ROOT_RING_SLOTS_PER_REGION: u64 = 4;
const ROOT_RING_REGIONS: u64 = 3;
/// journal 环长：E198 主取样点 1 MiB。
const JOURNAL_RING_BYTES: u64 = 1 << 20;
/// 区域到盘（E198 登记第五节「几何」一段「区域到盘 [0, 1, 0]」）。
const REGION_DEVICES: [u32; 3] = [0, 1, 0];
/// 池里的两块盘：一个单元两份落在两块盘的同一个槽号上。
const DEVICES: [u32; 2] = [0, 1];
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"E198-birth-skip!";
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 覆盖写与写别的 inode 的内容长（E198 的 3000 字节覆盖写）。
const OVERWRITE_CONTENT_BYTES: usize = 3000;
/// 写别的 inode 时内容的盐比那一版的 txg 多这么多（攻方模型 `attack_publish_on_a_second_inode` 的取法），与写第一个文件的内容错开。
const SALT_OFFSET_OF_THE_OTHER_INODE_CONTENT: u64 = 1_000_000;
/// 读坏一个根槽时取反的字节（E198 登记第五节本地常量表第 8 行「槽内第 40 字节取反」）。
const BYTE_FLIPPED_TO_READ_A_ROOT_SLOT_BAD: u64 = 40;
/// 根槽宽是物理块大小；根记录的整槽自证校验和在 138 起 32 字节（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_SLOT_BYTES: usize = 512;
const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_CHECKSUM_BYTES: usize = 32;
/// 单元头里的类标签在偏移 6；码 2 节点的 key 宽在 51。
const UNIT_CLASS_OFFSET: usize = 6;
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
/// 单元头里诞生代号的偏移（D18（块里携带什么信息） 已定项 7 / 已定项 11）：码 1 在 75，码 3 在 73，码 2 在 52 + 2k。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
/// 重封时头的末端与载荷 CRC 的偏移：码 1 头 105、CRC 在 101；码 3 头 107、CRC 在 89；码 2 头 86 + 2k、CRC 在 76 + 2k。
const DATA_UNIT_HEADER_END: usize = 105;
const DATA_UNIT_PAYLOAD_CRC_OFFSET: usize = 101;
const PACKED_UNIT_HEADER_END: usize = 107;
const PACKED_UNIT_PAYLOAD_CRC_OFFSET: usize = 89;
const INDEX_NODE_HEADER_END_BEFORE_THE_KEYS: usize = 86;
const INDEX_NODE_PAYLOAD_CRC_OFFSET_BEFORE_THE_KEYS: usize = 76;
/// 找单元时每个槽读开头一个扇区看单元头。
const SECTOR_BYTES_READ_FOR_A_UNIT_HEADER: usize = 512;

/// 这几段历史的 mkfs 参数（E198 主取样点那一组）。
pub fn parameters_of_the_number_acquisition_histories() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: REGION_DEVICES.map(DeviceIdentity),
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING,
            journal_ring_bytes: JOURNAL_RING_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                ROOT_RING_SLOTS_PER_REGION,
            )
            .expect("每区 4 槽在系统配置收的区间里"),
        },
    }
}

/// 这一组盘的几何（改坏单元之后沿引用链补校验和、把根抄进 journal 记录时按它找单元区与环）。
pub fn pool_image_geometry() -> PoolImageGeometry {
    PoolImageGeometry {
        parameters: parameters_of_the_number_acquisition_histories(),
        device_bytes: DEVICE_BYTES,
    }
}

fn fixed_geometry() -> FixedGeometry {
    let parameters = parameters_of_the_number_acquisition_histories();
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 确定性内容（同 E198 装置的 `deterministic_content`）：同一个盐同一个长度两次逐字节相同。
fn deterministic_content(salt: u64, length: usize) -> Vec<u8> {
    let salt_term = usize::try_from(salt % 251).expect("模 251 装得进 usize");
    (0..length)
        .map(|index| u8::try_from((index + salt_term * 7 + 3) % 251).expect("模 251 装得进 u8"))
        .collect()
}

type RecordedDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 一次可写挂载走哪个入口。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountConfiguration {
    /// 产品挂载 `mount_writable`（nrb、rs1、skr、rs-two-files）。
    Product,
    /// `mount_writable_with_instance_table_sweep_configuration`：挂载末尾行数多于 0 就扫（E198 sw 三组主点的阈值 0），
    /// 清扫发布的落点取挂载的分配器。sw 那几段历史的每一次挂载都走它。
    SweptAtTheEndWhenAnyRowIsLeft,
}

/// 一个实例挂着时手里的东西：实例代号、分配器、现行那一版。
pub struct InstanceSession {
    pub instance: InstanceGeneration,
    allocator: PoolAllocator,
    current: PoolVersion,
}

impl InstanceSession {
    /// 这个实例发出的最后一版的 txg。
    pub fn last_txg(&self) -> u64 {
        self.current.root().checkpoint_txg.0
    }
}

/// 一次可写挂载交回的：会话、写行那次发布的 txg、挂载末尾的清扫删没删行。
pub struct MountOfAnInstance {
    pub session: InstanceSession,
    pub row_publish_txg: u64,
    pub sweep_deleted_rows: bool,
}

/// 清扫结局里删没删行（同 E198 装置的 `rows_deleted_by`，只取删没删）。
fn sweep_deleted_rows(sweep: &InstanceTableSweepAtTheEndOfTheMount) -> bool {
    match sweep {
        InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
            ..
        }) => true,
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
        | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
        | InstanceTableSweepAtTheEndOfTheMount::Swept(
            InstanceTableSweepOutcome::NoRowDeleted(_)
            | InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
            | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. }
            | InstanceTableSweepOutcome::SweepPublishFailed { .. },
        ) => false,
    }
}

fn mount_on(
    parameters: &MakeFilesystemParameters,
    devices: &mut RecordedDevices,
    configuration: MountConfiguration,
) -> Result<Mounted, MountError> {
    match configuration {
        MountConfiguration::Product => mount_writable(parameters, devices),
        MountConfiguration::SweptAtTheEndWhenAnyRowIsLeft => {
            mount_writable_with_instance_table_sweep_configuration(
                parameters,
                devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                InstanceTableSweepConfiguration {
                    switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
                    row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
                    sweep_publish_placements:
                        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
                },
            )
        }
    }
}

/// 写流里这一条是不是一次根槽写（同 E198 装置的 `is_a_root_slot_write`）。
fn is_a_root_slot_write(operation: &RetainedOperation) -> bool {
    matches!(
        operation.operation.kind,
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
    ) && fixed_geometry().classify(&operation.operation) == StepKind::RootRecordFua
}

/// 一段历史此刻的镜像：每一步在它上面起录制盘、录完把写流施加回来。
pub struct NumberAcquisitionSkipHistory {
    image: MemoryPool,
}

impl NumberAcquisitionSkipHistory {
    /// 两块空盘上 mkfs 之后。
    pub fn after_making_the_filesystem() -> Self {
        let mut history = Self {
            image: MemoryPool::with_devices(&DEVICES.map(DeviceIdentity), DEVICE_BYTES),
        };
        let parameters = parameters_of_the_number_acquisition_histories();
        history
            .landed(|devices| make_filesystem(&parameters, devices.as_mut_slice()))
            .expect("两块空盘上 mkfs");
        history
    }

    /// 在此刻的镜像上起两块录制盘（共用一条留内容的录制流），做 `body`，交回它的结果与这一步录下的写流；镜像不动。
    fn recorded<Value>(
        &self,
        body: impl FnOnce(&mut RecordedDevices) -> Value,
    ) -> (Value, Vec<RetainedOperation>) {
        let stream = SharedStream::retaining_contents();
        let mut devices: RecordedDevices = DEVICES
            .map(DeviceIdentity)
            .into_iter()
            .map(|identity| {
                let mut device = SparseBlockDevice::new(
                    DEVICE_BYTES,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                );
                if let Some(sparse) = self.image.devices.get(&identity) {
                    device.image = sparse.clone();
                }
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
        let value = body(&mut devices);
        drop(devices);
        (value, stream.retained_operations())
    }

    /// 做 `body`，把录下的写流整条施加回镜像。
    fn landed<Value>(&mut self, body: impl FnOnce(&mut RecordedDevices) -> Value) -> Value {
        let (value, operations) = self.recorded(body);
        self.image.apply(&operations);
        value
    }

    /// 一次可写挂载（写行、暖机，配置要扫的在末尾扫），写流整条落下。
    pub fn mount(&mut self, configuration: MountConfiguration) -> MountOfAnInstance {
        let parameters = parameters_of_the_number_acquisition_histories();
        let mounted = self
            .landed(|devices| mount_on(&parameters, devices, configuration))
            .expect("可写挂载");
        let Mounted {
            output,
            allocator,
            current,
        } = mounted;
        MountOfAnInstance {
            row_publish_txg: output.row_publish.root().checkpoint_txg.0,
            sweep_deleted_rows: sweep_deleted_rows(&output.instance_table_sweep),
            session: InstanceSession {
                instance: output.instance,
                allocator,
                current,
            },
        }
    }

    /// 产品挂载崩在它第一个根槽写（写行那次发布）之前：记录已落、那一版的根槽没落（E198 skr 组里实例 2 那一次）。交回那一版的 txg。
    pub fn mount_crashing_before_its_first_root_slot_write(&mut self) -> u64 {
        let parameters = parameters_of_the_number_acquisition_histories();
        let (mounted, operations) = self.recorded(|devices| mount_writable(&parameters, devices));
        let mounted = mounted.expect("要崩的那次可写挂载在内存里做完");
        let cut = operations
            .iter()
            .position(is_a_root_slot_write)
            .expect("挂载的写流里有根槽写");
        self.image.apply(&operations[..cut]);
        mounted.output.row_publish.root().checkpoint_txg.0
    }

    /// 新池新建文件（第一个文件、inode 1）。
    pub fn publish_first_file(&mut self, session: &mut InstanceSession) {
        let parameters = parameters_of_the_number_acquisition_histories();
        let instance = session.instance;
        let content = deterministic_content(0, OVERWRITE_CONTENT_BYTES);
        let write_time_seconds = WRITE_TIME_SECONDS + session.last_txg() + 1;
        let version_to_build_on = *session.current.root();
        let previous_record_bytes = session.current.record_bytes().to_vec();
        let allocator = &mut session.allocator;
        let output = self
            .landed(|devices| {
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_first_file(
                    &mut writer,
                    allocator,
                    &version_to_build_on,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    instance,
                    &previous_record_bytes,
                )
            })
            .expect("新池新建文件");
        session.current = PoolVersion::of_a_version_of_trees(output);
    }

    /// 覆盖写第一个文件一次（3000 字节，每次一个新的数据单元）。交回这一版。
    pub fn overwrite_the_first_file(&mut self, session: &mut InstanceSession) -> TransactionOutput {
        let parameters = parameters_of_the_number_acquisition_histories();
        let instance = session.instance;
        let salt = session.last_txg() + 1;
        let content = deterministic_content(salt, OVERWRITE_CONTENT_BYTES);
        let previous = session
            .current
            .file_version()
            .expect("覆盖写之前现行那一版带文件")
            .clone();
        let allocator = &mut session.allocator;
        let output = self
            .landed(|devices| {
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_overwrite(
                    &mut writer,
                    allocator,
                    &previous,
                    FirstFile {
                        content: &content,
                        write_time_seconds: WRITE_TIME_SECONDS + salt,
                    },
                    instance,
                )
            })
            .expect("覆盖写第一个文件");
        session.current = PoolVersion::of_a_version_of_trees(output.clone());
        output
    }

    /// 覆盖写第一个文件，直到这个实例的最后一版 txg 不小于 `last_txg`（E198 装置的 `write_until`）。
    pub fn overwrite_the_first_file_until(&mut self, session: &mut InstanceSession, last_txg: u64) {
        // 迭代上界：每次覆盖写 txg 加 1，至多 last_txg 次。
        while session.last_txg() < last_txg {
            self.overwrite_the_first_file(session);
        }
    }

    /// 建一个 inode（`publish_new_inodes` 建 1 个）。交回它的号（建之前那一版的 inode 号水位）。
    pub fn create_an_inode(&mut self, session: &mut InstanceSession) -> InodeNumber {
        let parameters = parameters_of_the_number_acquisition_histories();
        let instance = session.instance;
        let write_time_seconds = WRITE_TIME_SECONDS + session.last_txg() + 1;
        let previous = session
            .current
            .file_version()
            .expect("建 inode 之前现行那一版带文件")
            .clone();
        let created = InodeNumber(previous.inode_number_watermark());
        let allocator = &mut session.allocator;
        let output = self
            .landed(|devices| {
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_new_inodes(
                    &mut writer,
                    allocator,
                    &previous,
                    1,
                    write_time_seconds,
                    instance,
                )
            })
            .expect("建一个 inode");
        session.current = PoolVersion::of_a_version_of_trees(output);
        created
    }

    /// 覆盖写一个不是第一个文件的 inode（3000 字节）。交回这一版。
    pub fn overwrite_an_inode(
        &mut self,
        session: &mut InstanceSession,
        inode: InodeNumber,
    ) -> TransactionOutput {
        let parameters = parameters_of_the_number_acquisition_histories();
        let instance = session.instance;
        let salt = session.last_txg() + 1;
        let content = deterministic_content(
            salt + SALT_OFFSET_OF_THE_OTHER_INODE_CONTENT,
            OVERWRITE_CONTENT_BYTES,
        );
        let previous = session
            .current
            .file_version()
            .expect("写那个 inode 之前现行那一版带文件")
            .clone();
        let allocator = &mut session.allocator;
        let output = self
            .landed(|devices| {
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_overwrite_of_an_inode(
                    &mut writer,
                    allocator,
                    &previous,
                    inode,
                    &content,
                    WRITE_TIME_SECONDS + salt,
                    instance,
                )
            })
            .expect("覆盖写那个 inode");
        session.current = PoolVersion::of_a_version_of_trees(output.clone());
        output
    }

    /// 管理员回退（一次向前发布）到 (实例 1, `target_txg`)。交回回退那一版的 txg。
    pub fn roll_back_to_the_first_instance_at(
        &mut self,
        session: &mut InstanceSession,
        target_txg: u64,
    ) -> u64 {
        let parameters = parameters_of_the_number_acquisition_histories();
        let mut current = session
            .current
            .file_version()
            .expect("回退之前现行那一版带文件")
            .clone();
        let allocator = &mut session.allocator;
        self.landed(|devices| {
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                allocator,
                &mut current,
                RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(target_txg),
                },
            )
        })
        .expect("回退到实例 1 的那一版");
        session.current = PoolVersion::of_a_version_of_trees(current);
        session.last_txg()
    }

    /// 此刻镜像的一份拷贝。
    pub fn image(&self) -> MemoryPool {
        self.image.clone()
    }
}

/// E198 nrb 组主点那一段（`nrb.main.S4.ring1048576.last10.k3.bad*.overwrite`）的历史：实例 1 新建文件、覆盖写到 txg 10 →
/// 实例 2 产品挂载（写行 11、暖机 12、13）→ 覆盖写第一个文件 3 次（14、15、16）。交回此刻的历史与那三次覆盖写；读坏哪几槽由用例定。
/// txg 照 E198 第一段产物第 62 行（`newest=2@16`、洞对 `1@10>2@12`、`birth11`）核。
pub fn newest_row_bad_history() -> (NumberAcquisitionSkipHistory, Vec<TransactionOutput>) {
    let mut history = NumberAcquisitionSkipHistory::after_making_the_filesystem();
    let mut first = history.mount(MountConfiguration::Product).session;
    history.publish_first_file(&mut first);
    history.overwrite_the_first_file_until(&mut first, 10);
    assert_eq!(first.last_txg(), 10, "实例 1 最后一版是 txg 10");
    drop(first);
    let second = history.mount(MountConfiguration::Product);
    assert_eq!(
        (second.session.instance, second.row_publish_txg),
        (InstanceGeneration(2), 11),
        "实例 2 写行那次发布是 txg 11"
    );
    let mut session = second.session;
    assert_eq!(session.last_txg(), 13, "写行 11 之后暖机两版 12、13");
    let overwrites: Vec<TransactionOutput> = (0..3)
        .map(|_| history.overwrite_the_first_file(&mut session))
        .collect();
    assert_eq!(
        session.last_txg(),
        16,
        "挂载之后覆盖写 3 次，最后一版 txg 16"
    );
    (history, overwrites)
}

/// [`newest_row_bad_history`] 那一段读坏实例 2 写行与暖机那三槽 (2, 11)、(2, 12)、(2, 13)，R = (2, 14)（挂载之后第一次覆盖写）新写的数据单元
/// 诞生代号改小到 R⁻ = (1, 10) 的 txg。洞对 (1@10, 2@14)、R 的实例表指针出生 txg 11（写行那一版建的）：只认行的判法认作接得上、照 (a) 判那个
/// 数据单元（extent 树根生在 14，不是整棵继承）、I-3.5 红；加了第四条件（C699（取号跳过改认出生代号））认作接不上、N3 不判它。
/// 池级 checker 的 harness 用例与 GPU 对拍用例判的是这同一张。
pub fn image_whose_first_overwrite_after_an_unreadable_row_publish_and_warm_ups_holds_a_data_unit_born_at_the_previous_version(
) -> MemoryPool {
    let (history, overwrites) = newest_row_bad_history();
    let first_overwrite = &overwrites[0];
    assert_eq!(
        first_overwrite.root.checkpoint_txg.0, 14,
        "挂载之后第一次覆盖写是 txg 14"
    );
    let mut image = history.image();
    rewrite_unit_and_propagate(&mut image, data_unit_slot_of(first_overwrite), |bytes| {
        set_the_birth_txg_in_the_unit_header(bytes, 10);
    });
    read_the_root_slots_of_these_versions_bad(&mut image, &[(2, 11), (2, 12), (2, 13)]);
    assert!(
        has_a_readable_root_of(&image, 1, 10) && has_a_readable_root_of(&image, 2, 14),
        "R⁻ (1, 10) 与 R (2, 14) 的根读得出"
    );
    assert_eq!(
        readable_root_of(&image, 2, 14)
            .instance_table
            .head
            .birth_txg
            .0,
        11,
        "R 的实例表指针生在写行那一版 11"
    );
    image
}

/// 镜像上读得出、自证过的每一条根（读根环的次序）。
pub fn readable_roots_on(image: &MemoryPool) -> Vec<RootRecord> {
    let parameters = parameters_of_the_number_acquisition_histories();
    readable_roots(
        image,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
}

/// 镜像上有没有一条读得出的根是 (实例, txg) 这一版。
pub fn has_a_readable_root_of(image: &MemoryPool, instance: u32, txg: u64) -> bool {
    readable_roots_on(image)
        .iter()
        .any(|root| (root.instance.0, root.checkpoint_txg.0) == (instance, txg))
}

/// 读得出的 (实例, txg) 那一条根。
pub fn readable_root_of(image: &MemoryPool, instance: u32, txg: u64) -> RootRecord {
    readable_roots_on(image)
        .into_iter()
        .find(|root| (root.instance.0, root.checkpoint_txg.0) == (instance, txg))
        .unwrap_or_else(|| panic!("镜像上读得出 (实例 {instance}, txg {txg}) 那一条根"))
}

/// txg 那一版的根槽在哪块盘、哪个偏移（`target_for_publish` 按 txg 算的落点）。
fn root_slot_location_of_txg(txg: u64) -> (DeviceIdentity, DeviceOffsetInBytes) {
    let parameters = parameters_of_the_number_acquisition_histories();
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    (
        parameters.region_devices[usize::try_from(target.region).expect("区域号装得进 usize")],
        slot_offset(target, parameters.geometry.fixed_structure_slot_spacing),
    )
}

/// 把点名的每一版 (实例, txg) 的根槽读坏（槽内第 40 字节取反）。读坏之前核那一槽此刻住着的就是这一版的根（没被环盖掉）。
pub fn read_the_root_slots_of_these_versions_bad(image: &mut MemoryPool, versions: &[(u32, u64)]) {
    for (instance, txg) in versions.iter().copied() {
        let (device, offset) = root_slot_location_of_txg(txg);
        let slot_bytes =
            PoolReader::read(&*image, device, offset, ROOT_SLOT_BYTES).expect("根槽在盘内，读得到");
        let root = RootRecord::parse_slot(&slot_bytes, &FILESYSTEM_IDENTIFIER)
            .unwrap_or_else(|| panic!("读坏之前 txg {txg} 的根槽住着一条自证过的根"));
        assert_eq!(
            (root.instance.0, root.checkpoint_txg.0),
            (instance, txg),
            "读坏之前那一槽住着的是这一版的根（没被环盖掉）"
        );
        image.flip_byte(device, offset, BYTE_FLIPPED_TO_READ_A_ROOT_SLOT_BAD);
    }
}

/// 根环全部槽：(盘, 偏移)。
fn root_slot_locations() -> Vec<(DeviceIdentity, DeviceOffsetInBytes)> {
    let parameters = parameters_of_the_number_acquisition_histories();
    (0..ROOT_RING_REGIONS)
        .flat_map(|region| (0..ROOT_RING_SLOTS_PER_REGION).map(move |slot| (region, slot)))
        .map(|(region, slot)| {
            (
                parameters.region_devices[usize::try_from(region).expect("区域号装得进 usize")],
                slot_offset(
                    RootRingSlot { region, slot },
                    parameters.geometry.fixed_structure_slot_spacing,
                ),
            )
        })
        .collect()
}

fn read(image: &MemoryPool, device: u32, offset_in_bytes: u64, length: usize) -> Vec<u8> {
    PoolReader::read(
        image,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset_in_bytes),
        length,
    )
    .expect("这几份镜像上读的都是盘内的落点")
}

fn write(
    image: &mut MemoryPool,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    bytes: &[u8],
) {
    image
        .devices
        .get_mut(&device)
        .expect("这几份镜像都是两块盘")
        .write(offset, bytes);
}

/// 盘 0 单元区里写过的每个单元：(起点槽, 字节数)。两盘同槽，读一块盘就够。迭代上界是单元区的槽数；码 1 / 码 3 的单元占两槽，第二槽不当成单元头读。
fn units_on_disk(image: &MemoryPool) -> Vec<(u64, usize)> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384 装得进 usize");
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768 装得进 usize");
    let last_slot = DEVICE_BYTES / SLOT_BYTES;
    let mut units = Vec::new();
    let mut slot = pool_image_geometry().unit_area_start_slot().0;
    while slot < last_slot {
        let header = read(
            image,
            0,
            slot * SLOT_BYTES,
            SECTOR_BYTES_READ_FOR_A_UNIT_HEADER,
        );
        if header[..UNIT_MAGIC.len()] != UNIT_MAGIC {
            slot += 1;
            continue;
        }
        if header[UNIT_CLASS_OFFSET] == UNIT_CLASS_INDEX_NODE {
            units.push((slot, node_bytes));
            slot += 1;
        } else {
            units.push((slot, data_unit_bytes));
            slot += 2;
        }
    }
    units
}

/// 按类重封：先算载荷 CRC，再算头校验和。
fn reseal_unit(bytes: &mut [u8]) {
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let (header_end, payload_crc_offset) = match bytes[UNIT_CLASS_OFFSET] {
        UNIT_CLASS_DATA => (DATA_UNIT_HEADER_END, DATA_UNIT_PAYLOAD_CRC_OFFSET),
        UNIT_CLASS_PACKED => (PACKED_UNIT_HEADER_END, PACKED_UNIT_PAYLOAD_CRC_OFFSET),
        UNIT_CLASS_INDEX_NODE => (
            INDEX_NODE_HEADER_END_BEFORE_THE_KEYS + 2 * key_width,
            INDEX_NODE_PAYLOAD_CRC_OFFSET_BEFORE_THE_KEYS + 2 * key_width,
        ),
        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

/// 单元头里的诞生代号改成 `birth_txg`（按类取偏移）。
pub fn set_the_birth_txg_in_the_unit_header(bytes: &mut [u8], birth_txg: u64) {
    let offset = match bytes[UNIT_CLASS_OFFSET] {
        UNIT_CLASS_DATA => DATA_UNIT_BIRTH_TXG_OFFSET,
        UNIT_CLASS_PACKED => PACKED_UNIT_BIRTH_TXG_OFFSET,
        UNIT_CLASS_INDEX_NODE => {
            INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS
                + 2 * usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET])
        }
        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
    };
    bytes[offset..offset + 8].copy_from_slice(&birth_txg.to_le_bytes());
}

/// 一条位置条目：设备 4 + 槽号 6 + 整单元校验和 4。
fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [
        device.to_le_bytes().to_vec(),
        slot.to_le_bytes()[..6].to_vec(),
        checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
    let mut changed = false;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            changed = true;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    changed
}

/// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上（单元里的、根槽里的），被补的单元重封、再往上补，直到根槽；
/// 最后把每条根照抄进同 (实例, txg) 的 journal 记录的新根段。写法同 `pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs` 的那一份，
/// 单元区与根环按这一组盘的几何找。迭代上界：每补一个单元，它自己的新校验和进待补的表一次；引用链是有向无环的（子先于父写），补到根槽就停。
fn propagate(image: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
    while let Some((slot, old, new)) = changes.pop() {
        if old == new {
            continue;
        }
        for (container, length) in units.iter().copied() {
            if container == slot {
                continue;
            }
            let before = read(image, 0, container * SLOT_BYTES, length);
            let mut bytes = before.clone();
            let mut touched = false;
            for device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(device, slot, old),
                    &location_pattern(device, slot, new),
                );
            }
            if touched {
                reseal_unit(&mut bytes);
                for device in DEVICES {
                    write(
                        image,
                        DeviceIdentity(device),
                        DeviceOffsetInBytes(container * SLOT_BYTES),
                        &bytes,
                    );
                }
                changes.push((
                    container,
                    crc32_castagnoli(&before),
                    crc32_castagnoli(&bytes),
                ));
            }
        }
        for (device, offset) in root_slot_locations() {
            let mut bytes = read(image, device.0, offset.0, ROOT_SLOT_BYTES);
            let mut touched = false;
            for location_device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(location_device, slot, old),
                    &location_pattern(location_device, slot, new),
                );
            }
            if touched {
                let digest = wide_checksum_with_field_zeroed(
                    &bytes,
                    ROOT_SLOT_BYTES,
                    ROOT_RECORD_CHECKSUM_OFFSET,
                );
                bytes[ROOT_RECORD_CHECKSUM_OFFSET
                    ..ROOT_RECORD_CHECKSUM_OFFSET + ROOT_RECORD_CHECKSUM_BYTES]
                    .copy_from_slice(&digest);
                write(image, device, offset, &bytes);
            }
        }
    }
    copy_each_root_into_the_new_root_segment_of_its_journal_records_on(
        image,
        &pool_image_geometry(),
    );
}

/// 改一个单元（两盘一起改）：改完重封它的两道校验和，新的整单元校验和沿引用链补到根槽。
pub fn rewrite_unit_and_propagate(
    image: &mut MemoryPool,
    slot: u64,
    change: impl Fn(&mut Vec<u8>),
) {
    let units = units_on_disk(image);
    let length = units
        .iter()
        .find(|(start, _)| *start == slot)
        .unwrap_or_else(|| panic!("槽 {slot} 是盘上写过的一个单元的起点"))
        .1;
    let before = read(image, 0, slot * SLOT_BYTES, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    reseal_unit(&mut bytes);
    for device in DEVICES {
        write(
            image,
            DeviceIdentity(device),
            DeviceOffsetInBytes(slot * SLOT_BYTES),
            &bytes,
        );
    }
    propagate(
        image,
        &units,
        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
    );
}

/// (实例, txg) 那一版的根指着的实例表单元，头里的诞生代号改成 `birth_txg`（重封、补到根槽与 journal 记录）。
/// 根记录里实例表指针头的出生 txg 不改：取号跳过的第四条件读的是指针那一份，这一改只让 I-3.5 照 (a) 判这个单元时报违例、
/// 违例说明里带这一对的洞分类（`HoleInTheCandidateSet` 的成员名）。交回那个单元的槽号。
pub fn lower_the_birth_txg_of_the_instance_table_of_the_version(
    image: &mut MemoryPool,
    instance: u32,
    txg: u64,
    birth_txg: u64,
) -> u64 {
    let instance_table = readable_root_of(image, instance, txg).instance_table;
    assert_eq!(
        instance_table.locations[0].slot, instance_table.locations[1].slot,
        "两盘同槽：实例表单元按一个槽号改两盘"
    );
    let slot = instance_table.locations[0].slot.0;
    rewrite_unit_and_propagate(image, slot, |bytes| {
        assert_eq!(
            bytes[UNIT_CLASS_OFFSET], UNIT_CLASS_PACKED,
            "实例表单元是码 3 打包记录单元"
        );
        set_the_birth_txg_in_the_unit_header(bytes, birth_txg);
    });
    slot
}

/// 这一版新写的那一个数据单元的槽（这几段历史每次覆盖写都只写一个数据单元）。
pub fn data_unit_slot_of(version: &TransactionOutput) -> u64 {
    let [pointer] = version.data_pointers.as_slice() else {
        panic!(
            "这几版每次写都只有一个数据单元：{} 条数据指针",
            version.data_pointers.len()
        );
    };
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "两盘同槽：数据单元按一个槽号改两盘"
    );
    pointer.locations[0].slot.0
}
