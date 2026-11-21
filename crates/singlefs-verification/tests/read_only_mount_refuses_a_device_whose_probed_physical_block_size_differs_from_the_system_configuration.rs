//! 只读挂载择完系统配置之后逐盘比读者答的 `physical_block_size` 与系统配置记着的 mkfs 时的 `physical_block_size`（偏移 317，根槽宽）：
//! D12（目标介质） 已定项 4 比对清单第一行「拒绝挂载（可写与只读都拒）」，这份钉只读挂载那一半（可写那一半在
//! `writable_mount_refuses_a_device_whose_probed_physical_block_size_differs_from_the_system_configuration.rs`）；C1040。
//! 两盘同构、带第一个文件的池建好之后，把一块盘换成字节逐个相同、而探测值改一档的盘：只读挂载拒，点名那块盘、探测值与记录值；
//! 拒在择根之前——这次挂载只读过每块盘的两个系统配置槽，根环、journal 环一读都没发，读结论一样都没交出去。
//! 对照：同一份字节交回探测值与记录值相同的两块盘，只读挂载照常挂上，而且读到了系统配置槽之外的地方。
//! 两条用例各管一个方向：记录 512、探测 4096；记录 4096、探测 512。另两条：读者报了一块盘、问它多宽却答不出，只读挂载同样拒、
//! 点名那块盘；内存池（`MemoryPool`）答的是在它上面做 mkfs 的那块盘的块宽，4096 的池放进内存池照常只读挂上。

use std::cell::RefCell;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, Mounted, PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration,
};
use singlefs_core::mounted_read::{
    mount_read_only, MountReadOnlyFailure, PhysicalBlockSizeOfADeviceRefusingTheReadOnlyMount,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

const IMAGE_BYTES: u64 = 4 << 30;
/// 盘表里的两块盘。
const DEVICE_IDENTITIES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 第一个文件的内容与写入时刻：只读挂载挂上之后要打得开一个文件，挂上才不是「树表 0 条」。
const FIRST_FILE_CONTENT: &[u8] = b"read-only mount compares the probed physical block size";
const WRITE_TIME_SECONDS: u64 = 1_800_000_000;

fn parameters_with_the_physical_block_size(physical_block_size: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-probed!",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size,
            minimum_input_output_bytes: 0,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(0),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 两块探测值都是 `physical_block_size` 的空盘上照 `parameters` 做 mkfs、第一次可写挂载、发第一个文件，交回两块盘的字节。
fn images_of_a_pool_with_its_first_file_made_on_devices_probing(
    parameters: &MakeFilesystemParameters,
    physical_block_size: u32,
) -> Vec<SparseDevice> {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = DEVICE_IDENTITIES
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(physical_block_size)),
            )
        })
        .collect();
    make_filesystem(parameters, &mut devices).expect("两块盘的探测值与参数相同，mkfs 收");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(parameters, &mut devices)
        .expect("两块盘的探测值与记录值相同：第一次可写挂载做成");
    {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: FIRST_FILE_CONTENT,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            output.instance,
            current.record_bytes(),
        )
        .expect("第一个文件");
    }
    devices
        .into_iter()
        .map(|(_, device)| device.image)
        .collect()
}

/// `images` 逐块装进探测值为 `probed_physical_block_size_of_each_device` 的盘，按盘表次序交回。
fn devices_carrying(
    images: &[SparseDevice],
    probed_physical_block_size_of_each_device: [u32; 2],
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    DEVICE_IDENTITIES
        .into_iter()
        .zip(images)
        .zip(probed_physical_block_size_of_each_device)
        .map(|((identity, image), probed_physical_block_size)| {
            let mut device = SparseBlockDevice::new(
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(probed_physical_block_size),
            );
            device.image = image.clone();
            (identity, device)
        })
        .collect()
}

/// 包在一池盘外面的读者：每一读的落点与长度记下来，别的原样交给那池盘；`device_not_answered` 那块盘问它多宽时答「池里没有这块盘」。
struct ReaderRecordingEveryRead<'devices> {
    devices: &'devices [(DeviceIdentity, SparseBlockDevice)],
    device_not_answered: Option<DeviceIdentity>,
    reads: RefCell<Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)>>,
}

impl<'devices> ReaderRecordingEveryRead<'devices> {
    fn over(
        devices: &'devices [(DeviceIdentity, SparseBlockDevice)],
        device_not_answered: Option<DeviceIdentity>,
    ) -> Self {
        ReaderRecordingEveryRead {
            devices,
            device_not_answered,
            reads: RefCell::new(Vec::new()),
        }
    }

    /// 读过的每一读里，不是系统配置槽的那几读（每块盘两槽：偏移 0 与固定结构槽距，各 4096 字节）。
    fn reads_outside_the_system_configuration_slots(
        &self,
        parameters: &MakeFilesystemParameters,
    ) -> Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> {
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        let slot_offsets = [
            0,
            u64::from(parameters.geometry.fixed_structure_slot_spacing),
        ];
        self.reads
            .borrow()
            .iter()
            .filter(|(_, offset, length)| {
                !(slot_offsets.contains(&offset.0) && *length == slot_bytes)
            })
            .copied()
            .collect()
    }
}

impl PoolReader for ReaderRecordingEveryRead<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.devices)
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.devices, device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        if self.device_not_answered == Some(device) {
            return None;
        }
        PoolReader::physical_block_size_probed_on_the_device(self.devices, device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reads.borrow_mut().push((device, offset, length));
        PoolReader::read(self.devices, device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.devices, device, ring_start, ring_bytes)
    }
}

/// 同一份字节交回探测值都是 `recorded` 的两块盘：只读挂载照常挂上、打得开第一个文件，而且读过系统配置槽之外的地方
/// （拒的那几条用例断言「只读过系统配置槽」，这一条说明那个断言分得出差别）。
fn assert_the_same_images_on_devices_probing_the_recorded_value_mount_read_only(
    parameters: &MakeFilesystemParameters,
    images: &[SparseDevice],
    recorded: u32,
) {
    let devices_probing_the_recorded_value = devices_carrying(images, [recorded, recorded]);
    let reader = ReaderRecordingEveryRead::over(&devices_probing_the_recorded_value, None);
    let mounted = mount_read_only(&reader).unwrap_or_else(|failure| {
        panic!("同一份字节、两块盘都探测到 {recorded}：只读挂载照常挂上，实际 {failure:?}")
    });
    let first_file = mounted
        .mounted
        .open_file(&reader, InodeNumber(FIRST_INODE_NUMBER))
        .expect("挂上的是带第一个文件的那一版：第一个文件打得开");
    assert_eq!(
        first_file.inode_record().size,
        u64::try_from(FIRST_FILE_CONTENT.len()).expect("文件长装得进 u64"),
        "第一个文件的长度"
    );
    assert!(
        !reader
            .reads_outside_the_system_configuration_slots(parameters)
            .is_empty(),
        "挂上的那一次读过系统配置槽之外的地方（根环、journal 环、树）"
    );
}

/// 记录值 `recorded` 的池上，`swapped_device` 那块盘换成探测值 `probed` 的盘：只读挂载拒，只点名那一块盘、它的探测值与记录值，
/// 拒之前只读过系统配置槽；同一份字节交回探测值都是 `recorded` 的两块盘，只读挂载照常挂上。
fn assert_a_device_probing_another_physical_block_size_refuses_the_read_only_mount_before_the_root_is_chosen(
    recorded: u32,
    swapped_device: DeviceIdentity,
    probed: u32,
) {
    let parameters = parameters_with_the_physical_block_size(recorded);
    let images =
        images_of_a_pool_with_its_first_file_made_on_devices_probing(&parameters, recorded);
    let probed_of_each_device = DEVICE_IDENTITIES.map(|identity| {
        if identity == swapped_device {
            probed
        } else {
            recorded
        }
    });
    let devices = devices_carrying(&images, probed_of_each_device);
    let reader = ReaderRecordingEveryRead::over(&devices, None);
    let refusal = mount_read_only(&reader).err();
    assert_eq!(
        refusal,
        Some(
            MountReadOnlyFailure::DevicesWhosePhysicalBlockSizeDisagreesWithTheSystemConfiguration(
                vec![
                    PhysicalBlockSizeOfADeviceRefusingTheReadOnlyMount::ProbedValueDiffersFromTheSystemConfiguration(
                        PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration {
                            identity_handed_in: swapped_device,
                            probed_physical_block_size: PhysicalBlockSizeInBytes(probed),
                            physical_block_size_in_the_system_configuration: PhysicalBlockSizeInBytes(recorded),
                        }
                    )
                ]
            )
        ),
        "只读挂载拒，只点名换上去的那块盘：探测到 {probed}、系统配置记着 {recorded}"
    );
    assert_eq!(
        reader.reads_outside_the_system_configuration_slots(&parameters),
        Vec::new(),
        "拒在择根之前：只读过每块盘的两个系统配置槽"
    );
    assert_the_same_images_on_devices_probing_the_recorded_value_mount_read_only(
        &parameters,
        &images,
        recorded,
    );
}

/// 记录 512 的池，盘 1 换成探测 4096 的盘（探测值往大改一档）。
#[test]
fn a_device_probing_4096_on_a_pool_recording_512_refuses_the_read_only_mount_before_the_root_is_chosen(
) {
    assert_a_device_probing_another_physical_block_size_refuses_the_read_only_mount_before_the_root_is_chosen(
        512,
        DeviceIdentity(1),
        4096,
    );
}

/// 记录 4096 的池，盘 0 换成探测 512 的盘（探测值往小改一档；E72（设备描述符表的挂载复算） 那一格的方向）。
#[test]
fn a_device_probing_512_on_a_pool_recording_4096_refuses_the_read_only_mount_before_the_root_is_chosen(
) {
    assert_a_device_probing_another_physical_block_size_refuses_the_read_only_mount_before_the_root_is_chosen(
        4096,
        DeviceIdentity(0),
        512,
    );
}

/// 读者报了盘 1、问它多宽却答「池里没有这块盘」：这一比做不成，只读挂载拒、点名盘 1，不当成相等放行；拒之前只读过系统配置槽。
/// 对照：同一池盘、读者两块盘都答得出，只读挂载照常挂上。
#[test]
fn a_reader_that_does_not_answer_the_physical_block_size_of_one_of_its_devices_refuses_the_read_only_mount(
) {
    let parameters = parameters_with_the_physical_block_size(512);
    let images = images_of_a_pool_with_its_first_file_made_on_devices_probing(&parameters, 512);
    let devices = devices_carrying(&images, [512, 512]);
    let reader = ReaderRecordingEveryRead::over(&devices, Some(DeviceIdentity(1)));
    assert_eq!(
        mount_read_only(&reader).err(),
        Some(
            MountReadOnlyFailure::DevicesWhosePhysicalBlockSizeDisagreesWithTheSystemConfiguration(
                vec![
                    PhysicalBlockSizeOfADeviceRefusingTheReadOnlyMount::NotAnsweredByTheReader {
                        device: DeviceIdentity(1),
                    }
                ]
            )
        ),
        "读者答不出盘 1 多宽：只读挂载拒，只点名盘 1"
    );
    assert_eq!(
        reader.reads_outside_the_system_configuration_slots(&parameters),
        Vec::new(),
        "拒在择根之前：只读过每块盘的两个系统配置槽"
    );
    assert_the_same_images_on_devices_probing_the_recorded_value_mount_read_only(
        &parameters,
        &images,
        512,
    );
}

/// 内存池答的是在它上面做 mkfs 的那块盘的块宽（`SparseDevice::physical_block_size`）：4096 的池的字节放进内存池，每块盘答 4096，
/// 只读挂载照常挂上；内存池要是一律答扇区宽 512，这一条就被拒成探测 512、记录 4096。
#[test]
fn a_memory_pool_of_a_pool_made_on_4096_devices_answers_4096_and_mounts_read_only() {
    let parameters = parameters_with_the_physical_block_size(4096);
    let images = images_of_a_pool_with_its_first_file_made_on_devices_probing(&parameters, 4096);
    let memory_pool = MemoryPool {
        devices: DEVICE_IDENTITIES.into_iter().zip(images).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    };
    for identity in DEVICE_IDENTITIES {
        assert_eq!(
            memory_pool.physical_block_size_probed_on_the_device(identity),
            Some(PhysicalBlockSizeInBytes(4096)),
            "{identity:?} 答在它上面做 mkfs 的那块盘的块宽"
        );
    }
    let mounted = mount_read_only(&memory_pool).unwrap_or_else(|failure| {
        panic!("4096 的池放进内存池，每块盘答 4096：只读挂载照常挂上，实际 {failure:?}")
    });
    let first_file = mounted
        .mounted
        .open_file(&memory_pool, InodeNumber(FIRST_INODE_NUMBER))
        .expect("挂上的是带第一个文件的那一版：第一个文件打得开");
    assert_eq!(
        first_file.inode_record().size,
        u64::try_from(FIRST_FILE_CONTENT.len()).expect("文件长装得进 u64"),
        "第一个文件的长度"
    );
}
