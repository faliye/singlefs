//! 只供测试的开关 `FirstTxgOfTheNewInstance`（`.claude/rules/fs-design.md` 五条硬要求第 2 条）经真的可写挂载走到：
//! E205（多处读坏倒挂三条改法的代价） 跑前登记 `research/prompts/e205-preregistration.md` 第七节「独立算出、用命令核过的」那几行锚点上，
//! 三档各给写死的数——今天的实现、甲只并读坏、甲并全零（两种读法的定义在登记第一节「读法写死」里「甲的两种读法」那几行）。
//!
//! 每条用例一张盘面、一组每区槽数 S：同一张盘面拷三份，三档各做一次可写挂载（读故障各带一份，只在那一次挂载里有），
//! 比写行那次发布（新实例第一次发布）的 txg 与 `MountOutput::first_txg_of_the_new_instance_branch` 报的档名。
//! 产品入口 `mount_writable` 与今天的实现那一档给同一个数，在 mkfs 之后 S = 8 那一条里并排比；后台计算的两条产品入口各有一条 mkfs 之后 S = 8 的。
//!
//! 盘面的几何照 E198（取号跳过认出生代号的前提） 装置那一组：两块 256 MiB 盘、1 MiB journal 环、区域到盘 [0, 1, 0]，S 取 4、8、16，
//! 另有 S 取 5、6、7（不是 2 的幂）的几条。
//! - mkfs 之后：只做过 mkfs。
//! - 起点镜像：mkfs → 实例 1 可写挂载（写行 txg 1、暖机 txg 2）→ 新池新建文件 txg 3 → 覆盖写三次 txg 4、5、6；
//!   最后那次（txg 6）发布的系统配置轮换没落盘（登记第一节「六格」那段起点历史：最新那条根的根槽 FUA 持久、两块盘的系统配置轮换没持久）。
//! - 六格：起点镜像上 txg 5、6 两条根所在的根环槽与这两次发布的全部记录（两块盘各一份）读坏，每块盘自证过、世代号最大的那一槽系统配置读坏。
//!   根与记录的读坏三种、系统配置槽的读坏两种；每组 S 取一格，三格合起来三种根的读坏、两种系统配置的读坏各至少一次。
//!
//! 读故障三种：设备报读错；读回全零；读得出而自证不过（点名那一段第 40 个字节取反）。被这次挂载写过的落点当场撤掉，之后读回新写的字节
//! （登记第一节「六格」：故障只在这次挂载里有，被这次挂载写过的落点当场撤掉）。

use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::background_compute::{
    BackgroundComputeSettings, BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::compaction_plan::CompactionSwitches;
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable,
    mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold,
    mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches,
    mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance,
    FirstTxgOfTheNewInstance, MountError, Mounted, ShadowLedger,
};
use singlefs_core::recovery::RecoveryFailure;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_verification::memory_pool::{SparseBlockDevice, SparseDevice};

/// 每块盘的字节数（E198（取号跳过认出生代号的前提） 登记第五节「几何」一段）。
const DEVICE_BYTES: u64 = 256 << 20;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
/// journal 环长：E198（取号跳过认出生代号的前提） 主取样点 1 MiB。
const JOURNAL_RING_BYTES: u64 = 1 << 20;
/// 区域到盘（E198（取号跳过认出生代号的前提） 登记第五节「几何」一段「区域到盘 [0, 1, 0]」）。
const REGION_DEVICES: [u32; 3] = [0, 1, 0];
/// 池里的两块盘。
const DEVICES: [u32; 2] = [0, 1];
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"first-txg-switch";
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 新池新建文件与每次覆盖写的内容长：一个数据单元、一次发布一条记录。
const FILE_CONTENT_BYTES: usize = 3000;
/// 读得出而自证不过的读坏：取反点名那一段里这个下标的字节（同 E198（取号跳过认出生代号的前提） 登记第五节本地常量表第 8 行「槽内第 40 字节取反」）。
/// 根槽里它落在树表指针那一段，记录与系统配置槽里它落在 magic 之后、整槽校验和罩着的那一截：取反之后校验和对不上、字节也不是全零。
const BYTE_FLIPPED_INSIDE_A_RANGE_READ_BAD: u64 = 40;

fn parameters_with(declared_slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: REGION_DEVICES.map(DeviceIdentity),
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING,
            journal_ring_bytes: JOURNAL_RING_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                declared_slots_per_region,
            )
            .expect("用例给的 S 是 4 到 8 与 16，都在系统配置收的区间 [4, 16] 里"),
        },
    }
}

/// 确定性内容（同 E198（取号跳过认出生代号的前提） 装置的 `deterministic_content`）：同一个盐同一个长度两次逐字节相同。
fn deterministic_content(salt: u64) -> Vec<u8> {
    let salt_term = usize::try_from(salt % 251).expect("模 251 装得进 usize");
    (0..FILE_CONTENT_BYTES)
        .map(|index| u8::try_from((index + salt_term * 7 + 3) % 251).expect("模 251 装得进 u8"))
        .collect()
}

/// 一张盘面：mkfs 参数与两块盘的字节。每次挂载拷一份出去，盘面自己不动。
struct PoolImage {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, SparseDevice)>,
}

impl PoolImage {
    /// 照这张盘面拷两块内存盘。
    fn devices_copied(&self) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
        self.devices
            .iter()
            .map(|(identity, sparse)| {
                let mut device = SparseBlockDevice::new(
                    DEVICE_BYTES,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                );
                device.image = sparse.clone();
                (*identity, device)
            })
            .collect()
    }

    fn of_the_devices(
        parameters: MakeFilesystemParameters,
        devices: &[(DeviceIdentity, SparseBlockDevice)],
    ) -> Self {
        PoolImage {
            parameters,
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
        }
    }
}

fn empty_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    DEVICES
        .map(DeviceIdentity)
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(
                    DEVICE_BYTES,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                ),
            )
        })
        .collect()
}

/// 只做过 mkfs 的盘面。
fn image_right_after_make_filesystem(declared_slots_per_region: u64) -> PoolImage {
    let parameters = parameters_with(declared_slots_per_region);
    let mut devices = empty_devices();
    make_filesystem(&parameters, &mut devices).expect("两块空盘上 mkfs");
    PoolImage::of_the_devices(parameters, &devices)
}

/// 起点镜像连同 txg 5、6 那两次发布写出的东西（六格要按它们点名读坏的记录）。
struct StartingImage {
    image: PoolImage,
    publish_of_txg_five: TransactionOutput,
    publish_of_txg_six: TransactionOutput,
}

/// 每块盘两个系统配置槽此刻的字节，按 (盘, 偏移) 存。
fn system_configuration_slots_of_every_device(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Vec<(usize, DeviceOffsetInBytes, Vec<u8>)> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096 装得进 usize");
    let mut saved = Vec::new();
    for (index, (_, device)) in devices.iter().enumerate() {
        for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = DeviceOffsetInBytes(slot * u64::from(FIXED_STRUCTURE_SLOT_SPACING));
            let mut bytes = vec![0u8; slot_bytes];
            device
                .read_at(offset, &mut bytes)
                .expect("系统配置槽在盘内");
            saved.push((index, offset, bytes));
        }
    }
    saved
}

/// 起点镜像：mkfs → 实例 1 可写挂载（写行 txg 1、暖机 txg 2）→ 新池新建文件 txg 3 → 覆盖写 txg 4、5、6；
/// txg 6 那次发布写完之后把两块盘的系统配置槽写回它之前的字节（轮换没落盘，根槽、记录与单元都落了）。
fn starting_image(declared_slots_per_region: u64) -> StartingImage {
    let parameters = parameters_with(declared_slots_per_region);
    let mut devices = empty_devices();
    make_filesystem(&parameters, &mut devices).expect("两块空盘上 mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("mkfs 之后的可写挂载");
    assert_eq!(
        (
            output.row_publish.root().checkpoint_txg,
            output
                .warm_up_publishes
                .last()
                .expect("暖机至少一次")
                .root()
                .checkpoint_txg
        ),
        (CheckpointTxg(1), CheckpointTxg(2)),
        "起点历史：实例 1 写行 txg 1、暖机到 txg 2"
    );
    let instance = output.instance;
    let content_of_the_new_file = deterministic_content(3);
    let new_file = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_new_file,
                write_time_seconds: WRITE_TIME_SECONDS + 3,
            },
            instance,
            current.record_bytes(),
        )
        .expect("新池新建文件")
    };
    let mut overwrite = |previous: &TransactionOutput,
                         devices_written: &mut Vec<(DeviceIdentity, SparseBlockDevice)>|
     -> TransactionOutput {
        let salt = previous.root.checkpoint_txg.0 + 1;
        let content = deterministic_content(salt);
        let mut writer = PoolWriter::new(&parameters, devices_written.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS + salt,
            },
            instance,
        )
        .expect("覆盖写第一个文件")
    };
    let publish_of_txg_four = overwrite(&new_file, &mut devices);
    let publish_of_txg_five = overwrite(&publish_of_txg_four, &mut devices);
    let system_configuration_before_txg_six = system_configuration_slots_of_every_device(&devices);
    let publish_of_txg_six = overwrite(&publish_of_txg_five, &mut devices);
    for (index, offset, bytes) in &system_configuration_before_txg_six {
        devices[*index]
            .1
            .write_at(*offset, bytes, WriteDurability::Plain)
            .expect("txg 6 那次发布的系统配置轮换没落盘：写回轮换之前的字节");
    }
    assert_eq!(
        (
            new_file.root.checkpoint_txg,
            publish_of_txg_four.root.checkpoint_txg,
            publish_of_txg_five.root.checkpoint_txg,
            publish_of_txg_six.root.checkpoint_txg
        ),
        (
            CheckpointTxg(3),
            CheckpointTxg(4),
            CheckpointTxg(5),
            CheckpointTxg(6)
        ),
        "起点历史：新池新建文件 txg 3，覆盖写 txg 4、5、6"
    );
    StartingImage {
        image: PoolImage::of_the_devices(parameters, &devices),
        publish_of_txg_five,
        publish_of_txg_six,
    }
}

/// 一段读坏的落点读到什么（登记第一节「六格」：根与记录三种，系统配置槽两种）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadFaultKind {
    DeviceReportsAReadError,
    ReadsBackAllZero,
    /// 读得出、不是全零而自证不过：点名那一段第 [`BYTE_FLIPPED_INSIDE_A_RANGE_READ_BAD`] 个字节取反。
    ReadsBackBytesFailingSelfVerification,
}

/// 一段读坏的落点：哪块盘、从哪个字节起、多长、读到什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadFault {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    kind: ReadFaultKind,
}

impl ReadFault {
    fn overlaps(&self, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
        offset_in_bytes < self.offset_in_bytes + self.length_in_bytes
            && self.offset_in_bytes < offset_in_bytes + length_in_bytes
    }
}

/// 读的时候按点名的几段读坏、被写过就撤掉那一段的盘；写、写零、discard、屏障原样交给里面那块内存盘。
struct DeviceWithReadFaultsUntilWritten {
    memory_device: SparseBlockDevice,
    /// 这块盘上还在的读故障（被写过的那几段已经撤掉）。
    faults: Vec<ReadFault>,
}

impl DeviceWithReadFaultsUntilWritten {
    fn lift_the_faults_written_over(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        self.faults
            .retain(|fault| !fault.overlaps(offset_in_bytes, length_in_bytes));
    }
}

impl BlockDevice for DeviceWithReadFaultsUntilWritten {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        self.memory_device.read_at(offset, buffer)?;
        for fault in self
            .faults
            .iter()
            .filter(|fault| fault.overlaps(offset.0, length_in_bytes))
        {
            match fault.kind {
                ReadFaultKind::DeviceReportsAReadError => {
                    return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                        "用例点名的这一段读坏：设备报读错",
                    )));
                }
                ReadFaultKind::ReadsBackAllZero => {
                    let start_in_buffer = fault.offset_in_bytes.saturating_sub(offset.0);
                    let end_in_buffer = (fault.offset_in_bytes + fault.length_in_bytes - offset.0)
                        .min(length_in_bytes);
                    buffer[usize::try_from(start_in_buffer).expect("落在缓冲区里")
                        ..usize::try_from(end_in_buffer).expect("落在缓冲区里")]
                        .fill(0);
                }
                ReadFaultKind::ReadsBackBytesFailingSelfVerification => {
                    let flipped_offset =
                        fault.offset_in_bytes + BYTE_FLIPPED_INSIDE_A_RANGE_READ_BAD;
                    if (offset.0..offset.0 + length_in_bytes).contains(&flipped_offset) {
                        let index_in_buffer =
                            usize::try_from(flipped_offset - offset.0).expect("落在缓冲区里");
                        buffer[index_in_buffer] ^= 0xFF;
                    }
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
        self.lift_the_faults_written_over(
            offset.0,
            u64::try_from(bytes.len()).expect("一次写的长度装得进 u64"),
        );
        self.memory_device.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.lift_the_faults_written_over(offset.0, length);
        self.memory_device.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.lift_the_faults_written_over(offset.0, length);
        self.memory_device.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.memory_device.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.memory_device.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.memory_device.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.memory_device
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// txg 为 `checkpoint_txg` 的那次发布写的根槽（落点公式定的区域与槽、那个区域住的盘），整个根槽一段。
fn root_ring_slot_read_bad(
    parameters: &MakeFilesystemParameters,
    checkpoint_txg: CheckpointTxg,
    kind: ReadFaultKind,
) -> ReadFault {
    let target = target_for_publish(
        checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    ReadFault {
        device: parameters.region_devices[usize::try_from(target.region).expect("区域号")],
        offset_in_bytes: slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0,
        length_in_bytes: u64::from(parameters.geometry.physical_block_size),
        kind,
    }
}

/// 一次发布写的全部记录，两块盘各一份（D23（journal 的角色与格式） 已定项 14「两份镜像」：每块盘同一个偏移）。
fn records_of_the_publish_read_bad(
    parameters: &MakeFilesystemParameters,
    publish: &TransactionOutput,
    kind: ReadFaultKind,
) -> Vec<ReadFault> {
    publish
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain(std::iter::once(publish.record.counter))
        .flat_map(|counter| {
            DEVICES.map(DeviceIdentity).map(|device| ReadFault {
                device,
                offset_in_bytes: record_offset(counter, parameters.geometry.journal_ring_bytes).0,
                length_in_bytes: JOURNAL_RECORD_BYTES,
                kind,
            })
        })
        .collect()
}

/// 每块盘自证过（整槽校验和过、fsid 是本池的）、世代号最大的那一槽系统配置读坏。
fn newest_system_configuration_slot_of_every_device_read_bad(
    image: &PoolImage,
    kind: ReadFaultKind,
) -> Vec<ReadFault> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096 装得进 usize");
    image
        .devices
        .iter()
        .map(|(device, sparse)| {
            let newest_offset = (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                .map(|slot| slot * u64::from(FIXED_STRUCTURE_SLOT_SPACING))
                .filter_map(|offset| {
                    SystemConfiguration::parse_slot(
                        &sparse.read(DeviceOffsetInBytes(offset), slot_bytes),
                    )
                    .ok()
                    .filter(|system_configuration| {
                        system_configuration.immutable.filesystem_identifier
                            == FILESYSTEM_IDENTIFIER
                    })
                    .map(|system_configuration| {
                        (system_configuration.quantities.slot_generation, offset)
                    })
                })
                .max()
                .map(|(_, offset)| offset)
                .expect("起点镜像上每块盘至少一槽系统配置自证过");
            ReadFault {
                device: *device,
                offset_in_bytes: newest_offset,
                length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
                kind,
            }
        })
        .collect()
}

/// 六格里的一格（登记第一节「六格」）：txg 5、6 两条根所在的根环槽与这两次发布的全部记录照 `root_and_record_fault` 读坏，
/// 每块盘最新那一槽系统配置照 `system_configuration_fault` 读坏。
fn read_faults_of_the_six_cell_image(
    starting: &StartingImage,
    root_and_record_fault: ReadFaultKind,
    system_configuration_fault: ReadFaultKind,
) -> Vec<ReadFault> {
    let parameters = &starting.image.parameters;
    let mut faults = vec![
        root_ring_slot_read_bad(parameters, CheckpointTxg(5), root_and_record_fault),
        root_ring_slot_read_bad(parameters, CheckpointTxg(6), root_and_record_fault),
    ];
    faults.extend(records_of_the_publish_read_bad(
        parameters,
        &starting.publish_of_txg_five,
        root_and_record_fault,
    ));
    faults.extend(records_of_the_publish_read_bad(
        parameters,
        &starting.publish_of_txg_six,
        root_and_record_fault,
    ));
    faults.extend(newest_system_configuration_slot_of_every_device_read_bad(
        &starting.image,
        system_configuration_fault,
    ));
    faults
}

/// 照盘面拷两块内存盘，各带上自己那块盘的读故障。
fn devices_with_read_faults(
    image: &PoolImage,
    faults: &[ReadFault],
) -> Vec<(DeviceIdentity, DeviceWithReadFaultsUntilWritten)> {
    let faults_by_device: BTreeMap<DeviceIdentity, Vec<ReadFault>> =
        faults.iter().fold(BTreeMap::new(), |mut by_device, fault| {
            by_device.entry(fault.device).or_default().push(*fault);
            by_device
        });
    image
        .devices_copied()
        .into_iter()
        .map(|(identity, memory_device)| {
            (
                identity,
                DeviceWithReadFaultsUntilWritten {
                    memory_device,
                    faults: faults_by_device.get(&identity).cloned().unwrap_or_default(),
                },
            )
        })
        .collect()
}

/// 一次可写挂载取到的新实例第一个 txg（写行那次发布的 txg）与它报的档名。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FirstTxgTaken {
    row_publish_txg: u64,
    branch_name: &'static str,
}

/// 三档各做一次可写挂载取到的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FirstTxgsOfTheThreeSettings {
    past_the_highest_root_and_record_in_the_ring: FirstTxgTaken,
    past_ring_slots_read_bad_at_mount: FirstTxgTaken,
    past_ring_slots_read_bad_or_all_zero_at_mount: FirstTxgTaken,
}

/// 写死的三个数配上三档各自该报的档名（档名照 `FirstTxgOfTheNewInstance::branch_name` 逐字写死）。
fn expected_first_txgs(
    past_the_highest_root_and_record_in_the_ring: u64,
    past_ring_slots_read_bad_at_mount: u64,
    past_ring_slots_read_bad_or_all_zero_at_mount: u64,
) -> FirstTxgsOfTheThreeSettings {
    FirstTxgsOfTheThreeSettings {
        past_the_highest_root_and_record_in_the_ring: FirstTxgTaken {
            row_publish_txg: past_the_highest_root_and_record_in_the_ring,
            branch_name: "first_txg=past_the_highest_root_and_record_in_the_ring",
        },
        past_ring_slots_read_bad_at_mount: FirstTxgTaken {
            row_publish_txg: past_ring_slots_read_bad_at_mount,
            branch_name: "first_txg=past_ring_slots_read_bad_at_mount",
        },
        past_ring_slots_read_bad_or_all_zero_at_mount: FirstTxgTaken {
            row_publish_txg: past_ring_slots_read_bad_or_all_zero_at_mount,
            branch_name: "first_txg=past_ring_slots_read_bad_or_all_zero_at_mount",
        },
    }
}

/// 在盘面的一份拷贝上带着读故障做一次只供测试的可写挂载，开关给 `setting`。
fn first_txg_taken_under(
    image: &PoolImage,
    faults: &[ReadFault],
    setting: FirstTxgOfTheNewInstance,
) -> FirstTxgTaken {
    let mut devices = devices_with_read_faults(image, faults);
    let mounted = mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance(
        &image.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        setting,
    )
    .unwrap_or_else(|refused| panic!("{setting:?} 那一档的可写挂载做成：{refused:?}"));
    FirstTxgTaken {
        row_publish_txg: mounted.output.row_publish.root().checkpoint_txg.0,
        branch_name: mounted.output.first_txg_of_the_new_instance_branch,
    }
}

/// 同一张盘面、同一组读故障，三档各在自己那一份拷贝上挂一次（三次挂载互不相干：每次从盘面重拷、读故障重新带上）。
fn first_txgs_of_the_three_settings(
    image: &PoolImage,
    faults: &[ReadFault],
) -> FirstTxgsOfTheThreeSettings {
    FirstTxgsOfTheThreeSettings {
        past_the_highest_root_and_record_in_the_ring: first_txg_taken_under(
            image,
            faults,
            FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing,
        ),
        past_ring_slots_read_bad_at_mount: first_txg_taken_under(
            image,
            faults,
            FirstTxgOfTheNewInstance::PastRingSlotsReadBadAtMount,
        ),
        past_ring_slots_read_bad_or_all_zero_at_mount: first_txg_taken_under(
            image,
            faults,
            FirstTxgOfTheNewInstance::PastRingSlotsReadBadOrAllZeroAtMount,
        ),
    }
}

/// 产品入口 `mount_writable` 在盘面的一份拷贝上（不带读故障）取到的新实例第一个 txg。
fn first_txg_taken_by_the_product_entry(image: &PoolImage) -> u64 {
    let mut devices = image.devices_copied();
    mount_writable(&image.parameters, &mut devices)
        .expect("产品入口的可写挂载做成")
        .output
        .row_publish
        .root()
        .checkpoint_txg
        .0
}

// ===== mkfs 之后第一次可写挂载（登记第七节：甲并全零 12 / 24 / 48，甲只并读坏 1；第七节「出自被测条款本身的」：今天的实现 1）=====

/// 场景：S = 4，只做过 mkfs 的池上第一次可写挂载，不带读故障；环里只有三份第 0 代根，别的槽读回全零。
/// 预期：今天的实现 1；甲只并读坏 1（没有读坏的槽）；甲并全零越过 txg 11 那一槽（区域 2 槽 3），给 12。
#[test]
fn writable_mount_right_after_make_filesystem_with_4_slots_per_region_takes_txg_1_today_1_past_read_bad_12_past_read_bad_or_all_zero(
) {
    assert_eq!(
        first_txgs_of_the_three_settings(&image_right_after_make_filesystem(4), &[]),
        expected_first_txgs(1, 1, 12),
        "mkfs 之后 S = 4：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 8，同上。另用产品入口 `mount_writable` 挂一次。
/// 预期：今天的实现与产品入口都是 1；甲只并读坏 1；甲并全零 24（越过 txg 23 那一槽）。
#[test]
fn writable_mount_right_after_make_filesystem_with_8_slots_per_region_takes_txg_1_today_and_on_the_product_entry_1_past_read_bad_24_past_read_bad_or_all_zero(
) {
    let image = image_right_after_make_filesystem(8);
    assert_eq!(
        first_txg_taken_by_the_product_entry(&image),
        1,
        "产品入口恒走今天的实现那一档"
    );
    assert_eq!(
        first_txgs_of_the_three_settings(&image, &[]),
        expected_first_txgs(1, 1, 24),
        "mkfs 之后 S = 8：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 16，同上。
/// 预期：今天的实现 1；甲只并读坏 1；甲并全零 48（越过 txg 47 那一槽）。
#[test]
fn writable_mount_right_after_make_filesystem_with_16_slots_per_region_takes_txg_1_today_1_past_read_bad_48_past_read_bad_or_all_zero(
) {
    assert_eq!(
        first_txgs_of_the_three_settings(&image_right_after_make_filesystem(16), &[]),
        expected_first_txgs(1, 1, 48),
        "mkfs 之后 S = 16：今天的实现、甲只并读坏、甲并全零"
    );
}

// ===== 起点镜像不注故障（登记第七节：今天的实现 7；甲并全零 12 / 24 / 48；甲只并读坏 7）=====

/// 场景：S = 4 的起点镜像（txg 0..6 的根都在环里），不带读故障。
/// 预期：今天的实现 7；甲只并读坏 7（没有读坏的槽）；甲并全零越过 txg 7..11 那几个读回全零的槽，给 12。
#[test]
fn writable_mount_of_the_starting_image_without_read_faults_with_4_slots_per_region_takes_txg_7_today_7_past_read_bad_12_past_read_bad_or_all_zero(
) {
    assert_eq!(
        first_txgs_of_the_three_settings(&starting_image(4).image, &[]),
        expected_first_txgs(7, 7, 12),
        "起点镜像 S = 4、不注故障：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 8 的起点镜像，不带读故障。
/// 预期：今天的实现 7；甲只并读坏 7；甲并全零 24。
#[test]
fn writable_mount_of_the_starting_image_without_read_faults_with_8_slots_per_region_takes_txg_7_today_7_past_read_bad_24_past_read_bad_or_all_zero(
) {
    assert_eq!(
        first_txgs_of_the_three_settings(&starting_image(8).image, &[]),
        expected_first_txgs(7, 7, 24),
        "起点镜像 S = 8、不注故障：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 16 的起点镜像，不带读故障。
/// 预期：今天的实现 7；甲只并读坏 7；甲并全零 48。
#[test]
fn writable_mount_of_the_starting_image_without_read_faults_with_16_slots_per_region_takes_txg_7_today_7_past_read_bad_48_past_read_bad_or_all_zero(
) {
    assert_eq!(
        first_txgs_of_the_three_settings(&starting_image(16).image, &[]),
        expected_first_txgs(7, 7, 48),
        "起点镜像 S = 16、不注故障：今天的实现、甲只并读坏、甲并全零"
    );
}

// ===== 起点镜像只坏 txg 5 那一槽（设备报读错；登记第七节：今天的实现 7；甲两种读法 18 / 30 / 54）=====

/// txg 5 那条根所在的根环槽这一次挂载里读报设备错（单故障）。
fn ring_slot_of_txg_five_unreadable(starting: &StartingImage) -> Vec<ReadFault> {
    vec![root_ring_slot_read_bad(
        &starting.image.parameters,
        CheckpointTxg(5),
        ReadFaultKind::DeviceReportsAReadError,
    )]
}

/// 场景：S = 4 的起点镜像，txg 5 那一槽读报设备错；读得出的最新根 txg 6，环里记录最大 txg 6。
/// 预期：今天的实现 7；那一槽在 [7, 17] 里能住的是 17，甲两种读法都给 18（第一次发布落在 txg 6 那一槽）。
#[test]
fn writable_mount_of_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_with_4_slots_per_region_takes_txg_7_today_18_past_read_bad_18_past_read_bad_or_all_zero(
) {
    let starting = starting_image(4);
    assert_eq!(
        first_txgs_of_the_three_settings(
            &starting.image,
            &ring_slot_of_txg_five_unreadable(&starting)
        ),
        expected_first_txgs(7, 18, 18),
        "起点镜像 S = 4、只坏 txg 5 那一槽：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 8，同上。
/// 预期：今天的实现 7；甲两种读法都给 30（那一槽在 [7, 29] 里能住的是 29）。
#[test]
fn writable_mount_of_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_with_8_slots_per_region_takes_txg_7_today_30_past_read_bad_30_past_read_bad_or_all_zero(
) {
    let starting = starting_image(8);
    assert_eq!(
        first_txgs_of_the_three_settings(
            &starting.image,
            &ring_slot_of_txg_five_unreadable(&starting)
        ),
        expected_first_txgs(7, 30, 30),
        "起点镜像 S = 8、只坏 txg 5 那一槽：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 16，同上。
/// 预期：今天的实现 7；甲两种读法都给 54（那一槽在 [7, 53] 里能住的是 53）。
#[test]
fn writable_mount_of_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_with_16_slots_per_region_takes_txg_7_today_54_past_read_bad_54_past_read_bad_or_all_zero(
) {
    let starting = starting_image(16);
    assert_eq!(
        first_txgs_of_the_three_settings(
            &starting.image,
            &ring_slot_of_txg_five_unreadable(&starting)
        ),
        expected_first_txgs(7, 54, 54),
        "起点镜像 S = 16、只坏 txg 5 那一槽：今天的实现、甲只并读坏、甲并全零"
    );
}

// ===== 六格（登记第七节：甲并全零 12 / 24 / 48；甲只并读坏在根与记录设备报读错、读得出自证不过的格是 7，在根读回全零的格是 5；
// 第七节「出自被测条款本身的」：今天的实现 5）=====

/// 场景：S = 4，六格里根与记录设备报读错、每块盘最新那一槽系统配置读回全零的那一格；读得出的最新根 txg 4，记录最大 txg 4。
/// 预期：今天的实现 5；甲只并读坏越过 txg 5、6 两槽，给 7；甲并全零越过到 txg 11 那一槽，给 12。
#[test]
fn six_cell_image_with_the_two_newest_roots_and_their_records_unreadable_and_the_newest_system_configuration_slots_all_zero_with_4_slots_per_region_takes_txg_5_today_7_past_read_bad_12_past_read_bad_or_all_zero(
) {
    let starting = starting_image(4);
    let faults = read_faults_of_the_six_cell_image(
        &starting,
        ReadFaultKind::DeviceReportsAReadError,
        ReadFaultKind::ReadsBackAllZero,
    );
    assert_eq!(
        first_txgs_of_the_three_settings(&starting.image, &faults),
        expected_first_txgs(5, 7, 12),
        "六格 S = 4（根与记录设备报读错 × 系统配置读回全零）：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 8，六格里根与记录读回全零、每块盘最新那一槽系统配置读回全零的那一格（登记第四节副本读数那一行的格）。
/// 预期：今天的实现 5；甲只并读坏不越过读回全零的槽，给 5（= 今天的实现）；甲并全零给 24。
#[test]
fn six_cell_image_with_the_two_newest_roots_and_their_records_all_zero_and_the_newest_system_configuration_slots_all_zero_with_8_slots_per_region_takes_txg_5_today_5_past_read_bad_24_past_read_bad_or_all_zero(
) {
    let starting = starting_image(8);
    let faults = read_faults_of_the_six_cell_image(
        &starting,
        ReadFaultKind::ReadsBackAllZero,
        ReadFaultKind::ReadsBackAllZero,
    );
    assert_eq!(
        first_txgs_of_the_three_settings(&starting.image, &faults),
        expected_first_txgs(5, 5, 24),
        "六格 S = 8（根与记录读回全零 × 系统配置读回全零）：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 16，六格里根与记录读得出而自证不过、每块盘最新那一槽系统配置读得出而自证不过的那一格。
/// 预期：今天的实现 5；甲只并读坏越过 txg 5、6 两槽，给 7；甲并全零给 48。
#[test]
fn six_cell_image_with_the_two_newest_roots_and_their_records_failing_self_verification_and_the_newest_system_configuration_slots_failing_self_verification_with_16_slots_per_region_takes_txg_5_today_7_past_read_bad_48_past_read_bad_or_all_zero(
) {
    let starting = starting_image(16);
    let faults = read_faults_of_the_six_cell_image(
        &starting,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
    assert_eq!(
        first_txgs_of_the_three_settings(&starting.image, &faults),
        expected_first_txgs(5, 7, 48),
        "六格 S = 16（根与记录读得出自证不过 × 系统配置读得出自证不过）：今天的实现、甲只并读坏、甲并全零"
    );
}

// ===== S 不是 2 的幂（登记第一节「两种读法共用的取法」：3S 个连续的 txg 每槽恰好落一个，不靠 S 是 2 的幂）=====

/// 只做过 mkfs 的池上三档取到的：今天的实现 1；甲只并读坏 1；甲并全零越过 txg 3S − 1 那一槽，给 3S。
fn assert_right_after_make_filesystem_the_three_settings_take_1_1_and_the_ring_length(
    declared_slots_per_region: u64,
    ring_length_in_txgs: u64,
) {
    assert_eq!(
        first_txgs_of_the_three_settings(
            &image_right_after_make_filesystem(declared_slots_per_region),
            &[],
        ),
        expected_first_txgs(1, 1, ring_length_in_txgs),
        "mkfs 之后 S = {declared_slots_per_region}：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 5，只做过 mkfs 的池上第一次可写挂载，不带读故障。
/// 预期：今天的实现 1；甲只并读坏 1；甲并全零 15。
/// 判别力：`crates/mutations.tsv`「E205（多处读坏倒挂三条改法的代价） 新实例第一个 txg 的只供测试开关：甲窗口长把 S 取成不小于它的 2 的幂」那一行，
/// 甲并全零给 24，这一条红。
#[test]
fn writable_mount_right_after_make_filesystem_with_5_slots_per_region_takes_txg_1_today_1_past_read_bad_15_past_read_bad_or_all_zero(
) {
    assert_right_after_make_filesystem_the_three_settings_take_1_1_and_the_ring_length(5, 15);
}

/// 场景：S = 6，同上。
/// 预期：今天的实现 1；甲只并读坏 1；甲并全零 18。
#[test]
fn writable_mount_right_after_make_filesystem_with_6_slots_per_region_takes_txg_1_today_1_past_read_bad_18_past_read_bad_or_all_zero(
) {
    assert_right_after_make_filesystem_the_three_settings_take_1_1_and_the_ring_length(6, 18);
}

/// 场景：S = 7，同上。
/// 预期：今天的实现 1；甲只并读坏 1；甲并全零 21。
#[test]
fn writable_mount_right_after_make_filesystem_with_7_slots_per_region_takes_txg_1_today_1_past_read_bad_21_past_read_bad_or_all_zero(
) {
    assert_right_after_make_filesystem_the_three_settings_take_1_1_and_the_ring_length(7, 21);
}

/// 起点镜像不带读故障时三档取到的：今天的实现 7；甲只并读坏 7；甲并全零 3S。
fn assert_the_starting_image_without_read_faults_takes_7_7_and_the_ring_length(
    declared_slots_per_region: u64,
    ring_length_in_txgs: u64,
) {
    assert_eq!(
        first_txgs_of_the_three_settings(&starting_image(declared_slots_per_region).image, &[]),
        expected_first_txgs(7, 7, ring_length_in_txgs),
        "起点镜像 S = {declared_slots_per_region}、不注故障：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 5 的起点镜像，不带读故障。
/// 预期：今天的实现 7；甲只并读坏 7；甲并全零 15。
#[test]
fn writable_mount_of_the_starting_image_without_read_faults_with_5_slots_per_region_takes_txg_7_today_7_past_read_bad_15_past_read_bad_or_all_zero(
) {
    assert_the_starting_image_without_read_faults_takes_7_7_and_the_ring_length(5, 15);
}

/// 场景：S = 6 的起点镜像，不带读故障。
/// 预期：今天的实现 7；甲只并读坏 7；甲并全零 18。
#[test]
fn writable_mount_of_the_starting_image_without_read_faults_with_6_slots_per_region_takes_txg_7_today_7_past_read_bad_18_past_read_bad_or_all_zero(
) {
    assert_the_starting_image_without_read_faults_takes_7_7_and_the_ring_length(6, 18);
}

/// 场景：S = 7 的起点镜像，不带读故障。
/// 预期：今天的实现 7；甲只并读坏 7；甲并全零 21。
#[test]
fn writable_mount_of_the_starting_image_without_read_faults_with_7_slots_per_region_takes_txg_7_today_7_past_read_bad_21_past_read_bad_or_all_zero(
) {
    assert_the_starting_image_without_read_faults_takes_7_7_and_the_ring_length(7, 21);
}

/// 起点镜像只坏 txg 5 那一槽（设备报读错）时三档取到的：今天的实现 7；那一槽在 [7, 3S + 5] 里落的是 3S + 5，甲两种读法都给 3S + 6。
fn assert_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_takes_7_and_the_ring_length_plus_6(
    declared_slots_per_region: u64,
    ring_length_in_txgs: u64,
) {
    let starting = starting_image(declared_slots_per_region);
    assert_eq!(
        first_txgs_of_the_three_settings(
            &starting.image,
            &ring_slot_of_txg_five_unreadable(&starting)
        ),
        expected_first_txgs(7, ring_length_in_txgs + 6, ring_length_in_txgs + 6),
        "起点镜像 S = {declared_slots_per_region}、只坏 txg 5 那一槽：今天的实现、甲只并读坏、甲并全零"
    );
}

/// 场景：S = 5 的起点镜像，txg 5 那一槽读报设备错。
/// 预期：今天的实现 7；甲两种读法都给 21。
#[test]
fn writable_mount_of_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_with_5_slots_per_region_takes_txg_7_today_21_past_read_bad_21_past_read_bad_or_all_zero(
) {
    assert_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_takes_7_and_the_ring_length_plus_6(
        5, 15,
    );
}

/// 场景：S = 6 的起点镜像，txg 5 那一槽读报设备错。
/// 预期：今天的实现 7；甲两种读法都给 24。
#[test]
fn writable_mount_of_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_with_6_slots_per_region_takes_txg_7_today_24_past_read_bad_24_past_read_bad_or_all_zero(
) {
    assert_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_takes_7_and_the_ring_length_plus_6(
        6, 18,
    );
}

/// 场景：S = 7 的起点镜像，txg 5 那一槽读报设备错。
/// 预期：今天的实现 7；甲两种读法都给 27。
#[test]
fn writable_mount_of_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_with_7_slots_per_region_takes_txg_7_today_27_past_read_bad_27_past_read_bad_or_all_zero(
) {
    assert_the_starting_image_with_the_ring_slot_of_txg_5_unreadable_takes_7_and_the_ring_length_plus_6(
        7, 21,
    );
}

// ===== 今天的值压过甲（「max(今天的值, max H + 1)」里今天的值那一项）=====

/// 场景：S = 4 的起点镜像；txg 5 那一槽设备报读错、txg 6 那一槽读回全零（与「txg 6 的根槽写崩在记录之后、根槽之前」同一张读数），
/// 两次发布的记录都读得出。
/// 预期：今天的实现 7（记录到 txg 6）；甲只并读坏只越过 txg 5 那一槽（H = 5），max(7, 6) 给 7；甲并全零越过到 txg 11 那一槽，给 12。
/// 判别力：`crates/mutations.tsv`「E205（多处读坏倒挂三条改法的代价） 新实例第一个 txg 的只供测试开关：甲去掉 max(今天的值)」那一行，
/// 甲只并读坏给 6（与实例 1 txg 6 那条记录同 txg），这一条红。
#[test]
fn writable_mount_of_the_starting_image_with_records_past_every_ring_slot_read_bad_keeps_todays_value_past_read_bad(
) {
    let starting = starting_image(4);
    let parameters = &starting.image.parameters;
    let faults = vec![
        root_ring_slot_read_bad(
            parameters,
            CheckpointTxg(5),
            ReadFaultKind::DeviceReportsAReadError,
        ),
        root_ring_slot_read_bad(
            parameters,
            CheckpointTxg(6),
            ReadFaultKind::ReadsBackAllZero,
        ),
    ];
    assert_eq!(
        first_txgs_of_the_three_settings(&starting.image, &faults),
        expected_first_txgs(7, 7, 12),
        "起点镜像 S = 4、txg 5 读坏、txg 6 读回全零、记录读得出"
    );
}

// ===== 只有第 0 代根、整环读坏 =====

/// 场景：S = 4，只做过 mkfs（环里只有三份第 0 代根），区域 0 槽 0 设备报读错。
/// 预期：区域 0 槽 0 是 N = 0 自己那一槽（txg 0 按落点公式住它），不越过：今天的实现 1、甲只并读坏 1、甲并全零 12。
#[test]
fn writable_mount_right_after_make_filesystem_with_region_0_slot_0_unreadable_with_4_slots_per_region_takes_txg_1_today_1_past_read_bad_12_past_read_bad_or_all_zero(
) {
    let image = image_right_after_make_filesystem(4);
    let region_zero_slot_zero_unreadable = vec![root_ring_slot_read_bad(
        &image.parameters,
        CheckpointTxg(0),
        ReadFaultKind::DeviceReportsAReadError,
    )];
    assert_eq!(
        first_txgs_of_the_three_settings(&image, &region_zero_slot_zero_unreadable),
        expected_first_txgs(1, 1, 12),
        "mkfs 之后 S = 4、区域 0 槽 0 读坏"
    );
}

/// 场景：S = 4，只做过 mkfs，区域 2 槽 0 设备报读错。
/// 预期：区域 2 槽 0 在 [1, 11] 里落 txg 2：今天的实现 1、甲只并读坏 3、甲并全零 12。
#[test]
fn writable_mount_right_after_make_filesystem_with_region_2_slot_0_unreadable_with_4_slots_per_region_takes_txg_1_today_3_past_read_bad_12_past_read_bad_or_all_zero(
) {
    let image = image_right_after_make_filesystem(4);
    let region_two_slot_zero_unreadable = vec![root_ring_slot_read_bad(
        &image.parameters,
        CheckpointTxg(2),
        ReadFaultKind::DeviceReportsAReadError,
    )];
    assert_eq!(
        first_txgs_of_the_three_settings(&image, &region_two_slot_zero_unreadable),
        expected_first_txgs(1, 3, 12),
        "mkfs 之后 S = 4、区域 2 槽 0 读坏"
    );
}

/// 场景：S = 4，只做过 mkfs，根环 12 个槽全部设备报读错；三档各在自己那一份拷贝上挂一次（同一张盘面、同一组读故障）。
/// 预期：三档都在读阶段拒（择不到根，`Recovery(NoValidRoot)`），走不到取第一个 txg 那一步。
#[test]
fn writable_mount_with_every_root_ring_slot_unreadable_is_refused_before_the_first_txg_under_every_setting(
) {
    let image = image_right_after_make_filesystem(4);
    let faults: Vec<ReadFault> = (0..12u64)
        .map(|txg| {
            root_ring_slot_read_bad(
                &image.parameters,
                CheckpointTxg(txg),
                ReadFaultKind::DeviceReportsAReadError,
            )
        })
        .collect();
    for setting in [
        FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing,
        FirstTxgOfTheNewInstance::PastRingSlotsReadBadAtMount,
        FirstTxgOfTheNewInstance::PastRingSlotsReadBadOrAllZeroAtMount,
    ] {
        let mut devices = devices_with_read_faults(&image, &faults);
        let mounted = mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance(
            &image.parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            setting,
        );
        assert!(
            matches!(
                mounted,
                Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
            ),
            "{setting:?}：整环读坏在读阶段择不到根：{:?}",
            mounted.as_ref().map(|mounted_anyway| mounted_anyway
                .output
                .row_publish
                .root()
                .checkpoint_txg)
        );
    }
}

// ===== 后台计算的两条产品入口（与 `mount_writable` 一样恒走今天的实现那一档）=====

/// 场景：S = 8，只做过 mkfs，经 `mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`
/// （产品的后台计算设置、不带 GPU、默认补跑阈值）挂一次。
/// 预期：第一个 txg 是 1，与今天的实现那一档相同（甲并全零那一档给 24）。
/// 判别力：`crates/mutations.tsv`「E205（多处读坏倒挂三条改法的代价） 新实例第一个 txg 的只供测试开关：后台计算带补跑阈值的产品入口走甲并全零」那一行，这一条红。
#[test]
fn writable_mount_right_after_make_filesystem_with_8_slots_per_region_takes_txg_1_on_the_background_compute_entry_with_the_gc_cpu_compensation_threshold(
) {
    let image = image_right_after_make_filesystem(8);
    let mut devices = image.devices_copied();
    let row_publish_txg =
        mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
            &image.parameters,
            &mut devices,
            BackgroundComputeSettings::PRODUCT,
            None,
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
        )
        .expect("产品的后台计算设置、默认阈值：可写挂载做成")
        .output
        .row_publish
        .root()
        .checkpoint_txg
        .0;
    assert_eq!(
        row_publish_txg, 1,
        "后台计算带补跑阈值的产品入口恒走今天的实现那一档"
    );
}

/// 场景：S = 8，只做过 mkfs，经 `mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches`
/// （产品的后台计算设置、不带 GPU、默认补跑阈值、产品的整理开关）挂一次。
/// 预期：第一个 txg 是 1，与今天的实现那一档相同（甲并全零那一档给 24）。
/// 判别力：`crates/mutations.tsv`「E205（多处读坏倒挂三条改法的代价） 新实例第一个 txg 的只供测试开关：后台计算带补跑阈值与整理开关的产品入口走甲并全零」那一行，这一条红。
#[test]
fn writable_mount_right_after_make_filesystem_with_8_slots_per_region_takes_txg_1_on_the_background_compute_entry_with_the_gc_cpu_compensation_threshold_and_compaction_switches(
) {
    let image = image_right_after_make_filesystem(8);
    let mut devices = image.devices_copied();
    let row_publish_txg =
        mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches(
            &image.parameters,
            &mut devices,
            BackgroundComputeSettings::PRODUCT,
            None,
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            CompactionSwitches::PRODUCT,
        )
        .expect("产品的后台计算设置、默认阈值、产品的整理开关：可写挂载做成")
        .output
        .row_publish
        .root()
        .checkpoint_txg
        .0;
    assert_eq!(
        row_publish_txg, 1,
        "后台计算带补跑阈值与整理开关的产品入口恒走今天的实现那一档"
    );
}
