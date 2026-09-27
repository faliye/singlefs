//! 记账树与中央映射树的分裂（D8（核心索引结构） 已定项 11）用例共用的搭建：两块稀疏内存盘上 mkfs → 取号 → 暖机 → 第一个文件版本，
//! 写入口装上压小节点容量的只供测试的开关（`CodeTwoTreeNodeCapacities::CappedForTests`），之后按场景接着推空发布或覆盖写。
//! 录制流开内容保留：层 0 用例按「分裂那一次发布之前的流全部施加成起点镜像、只录那一次」切（三方 `m2-treesplit-r1` T4）。
//! 盘是内存里的（`SparseBlockDevice`），不建文件镜像：内容与文件镜像同一份字节，「进程退出、重开」就是按此刻的盘面重新包一层录制器。
#![allow(dead_code, reason = "两份用例各自只用到其中一部分")]

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::code_two_tree::{
    CodeTwoTreeNodeCapacity, CodeTwoTreeNodeContents, CodeTwoTreeVersion,
};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up,
    CodeTwoTreeNodeCapacities, FirstFile, InstanceTablePlan, MultiLevelCodeTwoTree, PoolWriter,
    PublishPlan, TransactionOutput,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

use crate::common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};

/// 产品容量（按节点格式算）：记账树叶 477、内部 150；中央映射树叶 294、内部 143。
pub const ACCOUNTING_OF_THE_NODE_FORMAT: (usize, usize) = (477, 150);
pub const CENTRAL_MAPPING_OF_THE_NODE_FORMAT: (usize, usize) = (294, 143);

pub type RecordedMemoryDevice = RecordingBlockDevice<SparseBlockDevice>;

/// 两棵树各自的 (叶, 内部节点) 容量。两个都等于格式算出来的那一档时交回 `FromTheNodeFormat`（产品路径）。
#[must_use]
pub fn capacities(
    accounting: (usize, usize),
    central_mapping: (usize, usize),
) -> CodeTwoTreeNodeCapacities {
    if accounting == ACCOUNTING_OF_THE_NODE_FORMAT
        && central_mapping == CENTRAL_MAPPING_OF_THE_NODE_FORMAT
    {
        return CodeTwoTreeNodeCapacities::FromTheNodeFormat;
    }
    CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: CodeTwoTreeNodeCapacity {
            leaf_entries: accounting.0,
            internal_entries: accounting.1,
        },
        central_mapping: CodeTwoTreeNodeCapacity {
            leaf_entries: central_mapping.0,
            internal_entries: central_mapping.1,
        },
    }
}

/// 一版树的样子，按层级从下往上、同层从左往右：`L`（叶，装几条）/ `I`（内部节点，几个孩子）。断言与报数都读它。
#[must_use]
pub fn shape_text(tree: &CodeTwoTreeVersion) -> String {
    tree.shape
        .nodes()
        .iter()
        .map(|node| match &node.contents {
            CodeTwoTreeNodeContents::Leaf { keys } => format!("L{}", keys.len()),
            CodeTwoTreeNodeContents::Internal { children } => format!("I{}", children.len()),
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// 一版树里层级 0 的节点数（叶片数）。
#[must_use]
pub fn leaf_count(tree: &CodeTwoTreeVersion) -> usize {
    tree.shape
        .nodes()
        .iter()
        .filter(|node| node.position.level == 0)
        .count()
}

/// 一个单元（槽号 `slot`）在一块盘上的字节。
#[must_use]
pub fn read_unit_on_device(image: &MemoryPool, device: u32, slot: u64, length: usize) -> Vec<u8> {
    image
        .devices
        .get(&DeviceIdentity(device))
        .expect("盘")
        .read(DeviceOffsetInBytes(slot * SLOT_BYTES), length)
}

/// 两块盘同一个槽写同一份字节（这几份镜像上一个单元两份落点同槽）。
pub fn write_unit_to_the_same_slot_on_both_devices(
    image: &mut MemoryPool,
    slot: u64,
    bytes: &[u8],
) {
    for device in [0u32, 1] {
        image
            .devices
            .get_mut(&DeviceIdentity(device))
            .expect("盘")
            .write(DeviceOffsetInBytes(slot * SLOT_BYTES), bytes);
    }
}

fn replace_every(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
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

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
pub fn reseal_unit_by_its_class(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105usize, 101usize),
        3 => (107, 89),
        _index_node_class => (
            86 + 2 * usize::from(bytes[51]),
            76 + 2 * usize::from(bytes[51]),
        ),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096（`common::parameters` 的几何）。
#[must_use]
pub fn root_ring_slot_offsets() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| {
                (
                    [0u32, 1, 0][usize::try_from(region).expect("区域")],
                    64 * SLOT_BYTES + region * 3 * (1 << 20) + slot * 4096,
                )
            })
        })
        .collect()
}

/// checker 的已知坏镜像要的：把一个单元的新整单元校验和沿引用链补到根槽（父节点里的位置条目、中央映射条目、根槽的自证校验和）。
/// 位置条目按 (设备 4, 槽 6, 校验和 4) 逐字节找、逐个换，被换的单元按类重封再往上补。`units` 是这份镜像里被引用的单元（槽, 字节数）。
pub fn propagate_the_new_checksum(
    image: &mut MemoryPool,
    units: &[(u64, usize)],
    changed: (u64, u32, u32),
) {
    let mut pending = vec![changed];
    while let Some((slot, old, new)) = pending.pop() {
        let pattern = |device: u32, checksum: u32| -> Vec<u8> {
            [
                device.to_le_bytes().to_vec(),
                slot.to_le_bytes()[..6].to_vec(),
                checksum.to_le_bytes().to_vec(),
            ]
            .concat()
        };
        for (container, length) in units.iter().copied() {
            if container == slot {
                continue;
            }
            let before = read_unit_on_device(image, 0, container, length);
            let mut bytes = before.clone();
            let mut touched = false;
            for device in [0u32, 1] {
                touched |= replace_every(&mut bytes, &pattern(device, old), &pattern(device, new));
            }
            if touched {
                reseal_unit_by_its_class(&mut bytes);
                write_unit_to_the_same_slot_on_both_devices(image, container, &bytes);
                pending.push((
                    container,
                    crc32_castagnoli(&before),
                    crc32_castagnoli(&bytes),
                ));
            }
        }
        for (device, offset) in root_ring_slot_offsets() {
            let mut bytes = image
                .devices
                .get(&DeviceIdentity(device))
                .expect("盘")
                .read(DeviceOffsetInBytes(offset), 512);
            let mut touched = false;
            for location_device in [0u32, 1] {
                touched |= replace_every(
                    &mut bytes,
                    &pattern(location_device, old),
                    &pattern(location_device, new),
                );
            }
            if touched {
                let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                bytes[138..170].copy_from_slice(&digest);
                image
                    .devices
                    .get_mut(&DeviceIdentity(device))
                    .expect("盘")
                    .write(DeviceOffsetInBytes(offset), &bytes);
            }
        }
    }
}

fn memory_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, RecordedMemoryDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 录着的池：盘、分配器与现行那一版，外加每次发布之前录制流里有几步（层 0 按它切起点）。
pub struct TreeSplitPool {
    pub stream: SharedStream,
    pub devices: Vec<(DeviceIdentity, RecordedMemoryDevice)>,
    pub allocator: PoolAllocator,
    pub output: TransactionOutput,
    /// 现行那一版之前的每一版，按发布先后（第一个文件版本在最前）。
    pub earlier_versions: Vec<TransactionOutput>,
    /// 现行那一版的发布之前录制流里有几步：起点镜像施加到这里，层 0 只录它之后的。
    pub operations_before_the_current_version: usize,
}

impl TreeSplitPool {
    /// 两块 4 GiB 内存盘上 mkfs、取号、暖机两次，写入口装上 `first_file_capacities` 发第一个文件版本（一个数据单元）。
    #[must_use]
    pub fn with_the_first_file_version_under(
        first_file_capacities: CodeTwoTreeNodeCapacities,
    ) -> Self {
        let publish_parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices = memory_devices(&stream);
        let genesis = make_filesystem(&publish_parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
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
        let content = file_content();
        let (output, operations_before_the_first_file_version) = {
            let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let operations_before_the_first_file_version = stream.operations().len();
            writer.set_code_two_tree_node_capacities(first_file_capacities);
            let output = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed_up.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed_up.last_record_bytes,
            )
            .expect("第一个文件版本");
            (output, operations_before_the_first_file_version)
        };
        Self {
            stream,
            devices,
            allocator,
            output,
            earlier_versions: Vec::new(),
            operations_before_the_current_version: operations_before_the_first_file_version,
        }
    }

    /// 接着现行那一版推一次空发布（不写文件、不碰 inode 树、照抄实例表），写入口装上给定的容量。
    pub fn empty_publish(&mut self, capacities_of_this_publish: CodeTwoTreeNodeCapacities) {
        let publish_parameters = parameters();
        let previous = self.output.clone();
        let operations_before = self.stream.operations().len();
        let mut writer = PoolWriter::new(&publish_parameters, self.devices.as_mut_slice());
        writer.set_code_two_tree_node_capacities(capacities_of_this_publish);
        let output = publish_version(
            &mut writer,
            &mut self.allocator,
            PublishPlan {
                txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
                counter: previous.record.counter + 1,
                transaction: 0,
                highest_transaction_number_before_this_publish: previous
                    .highest_transaction_number_in_this_instance,
                instance: previous.root.instance,
                back_chain: back_chain_of(&previous.record_bytes),
                file: None,
                new_inode_records: &[],
                instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
                tree_birth_txg: previous.tree_birth_txg(),
                tree_identifier_watermark: previous.root.tree_identifier_watermark,
                rollback_floor: previous.root.rollback_floor,
            },
            Some(&previous),
        )
        .expect("空发布");
        self.earlier_versions.push(previous);
        self.output = output;
        self.operations_before_the_current_version = operations_before;
    }

    /// 接着现行那一版覆盖写一次（一个数据单元的内容），写入口装上给定的容量。
    pub fn overwrite(
        &mut self,
        content: &[u8],
        capacities_of_this_publish: CodeTwoTreeNodeCapacities,
    ) {
        let publish_parameters = parameters();
        let previous = self.output.clone();
        let operations_before = self.stream.operations().len();
        let mut writer = PoolWriter::new(&publish_parameters, self.devices.as_mut_slice());
        writer.set_code_two_tree_node_capacities(capacities_of_this_publish);
        let output = publish_overwrite(
            &mut writer,
            &mut self.allocator,
            &previous,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            previous.root.instance,
        )
        .expect("覆盖写");
        self.earlier_versions.push(previous);
        self.output = output;
        self.operations_before_the_current_version = operations_before;
    }

    /// 「进程退出、重开」：按此刻的盘面重新包一层录制器，写照旧录进同一条流。
    #[must_use]
    pub fn reopened_devices(&self) -> Vec<(DeviceIdentity, RecordedMemoryDevice)> {
        self.devices
            .iter()
            .map(|(identity, device)| {
                let mut reopened =
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
                reopened.image = device.wrapped_device().image.clone();
                (
                    *identity,
                    RecordingBlockDevice::with_shared_stream(
                        *identity,
                        reopened,
                        self.stream.clone(),
                    ),
                )
            })
            .collect()
    }

    /// 整条录制流带内容。
    #[must_use]
    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }

    /// 此刻的盘面。
    #[must_use]
    pub fn memory_pool(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: IMAGE_BYTES,
        }
    }

    /// 现行那一版的发布之前的镜像（层 0 的起点，不枚举）：录制流从头施加到那一次发布之前。
    #[must_use]
    pub fn memory_pool_before_the_current_version(&self) -> MemoryPool {
        let mut pool =
            MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        pool.apply(&self.retained_operations()[..self.operations_before_the_current_version]);
        pool
    }

    /// 现行那一版之前的那一版。
    ///
    /// # Panics
    /// 现行那一版就是第一个文件版本：它之前没有带文件的一版。
    #[must_use]
    pub fn version_before_the_current_one(&self) -> &TransactionOutput {
        self.earlier_versions
            .last()
            .expect("现行那一版之前还有一版带文件的")
    }

    /// 现行那一版一棵多层码 2 树的高，从根节点头现读（D28（挂载期承诺量） 已定项 4）。
    #[must_use]
    pub fn height(&self, tree: MultiLevelCodeTwoTree) -> u64 {
        self.output.height_read_from_the_root_node_header(tree)
    }
}
