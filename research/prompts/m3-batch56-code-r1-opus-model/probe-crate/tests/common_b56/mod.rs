//! 攻方原型共用（m3-batch56-code-r1 opus）：内存稀疏盘上的池（mkfs 与第一个文件只做一次，之后每段历史从它 clone），
//! 三棵容器树与码 2 节点的旋钮（写入口、分配器开关、池级 checker 新入口同一组值），几种用户动作。不建镜像文件。
#![allow(dead_code)]

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    ContainerCapacityKnobs, LineageContainerCapacityKnob, RetirementContainerCapacityKnob,
    TombstoneContainerCapacityKnob,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::retirement_tree::{
    RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;

pub type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
pub const DEVICE_BYTES: u64 = 4 << 30;
pub const WRITE_TIME_SECONDS: u64 = 1_788_300_000;
pub const FIRST_INSTANCE: InstanceGeneration = InstanceGeneration(1);

pub fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-opusb56",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

pub fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
    }
}

pub fn first_file_content() -> Vec<u8> {
    (0..3000u32).map(|index| u8::try_from(index % 251).expect("小于 256")).collect()
}

pub fn sparse_devices_of(image: &MemoryPool) -> Devices {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

pub fn image_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// 一组旋钮：写入口与分配器开关同一份，池级 checker 的新入口给同一组值（0 = 产品容量、不给旋钮）。
#[derive(Clone, Copy, Debug)]
pub struct Knobs {
    pub tombstone: u64,
    pub lineage: u64,
    pub retirement: u64,
    pub node_entries: usize,
}

impl Knobs {
    pub fn capacities(self) -> ContainerTreeCapacities {
        ContainerTreeCapacities {
            tombstone_container: match self.tombstone {
                0 => TombstoneContainerCapacity::FromTheContainerFormat,
                n => TombstoneContainerCapacity::CappedForTests(TombstoneRecordsPerContainerCappedForTests::new(n).expect("1..=815")),
            },
            lineage_container: match self.lineage {
                0 => LineageContainerCapacity::FromTheContainerFormat,
                n => LineageContainerCapacity::CappedForTests(LineageRecordsPerContainerCappedForTests::new(n).expect("1..=1359")),
            },
            retirement_container: match self.retirement {
                0 => RetirementContainerCapacity::FromTheContainerFormat,
                n => RetirementContainerCapacity::CappedForTests(RetirementRecordsPerContainerCappedForTests::new(n).expect("1..=959")),
            },
            node_entries: match self.node_entries {
                0 => ContainerTreeNodeEntries::FromTheNodeFormat,
                n => ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(n).expect("2..=135")),
            },
        }
    }

    pub fn switches(self) -> DeletionSwitches {
        let capacities = self.capacities();
        DeletionSwitches {
            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity: capacities.tombstone_container,
            lineage_container_capacity: capacities.lineage_container,
            retirement_container_capacity: capacities.retirement_container,
            container_tree_node_entries: capacities.node_entries,
        }
    }

    pub fn checker_knobs(self) -> ContainerCapacityKnobs {
        ContainerCapacityKnobs {
            tombstone: (self.tombstone > 0).then(|| TombstoneContainerCapacityKnob::new(self.tombstone).expect("1..=815")),
            lineage: (self.lineage > 0).then(|| LineageContainerCapacityKnob::new(self.lineage).expect("1..=1359")),
            retirement: (self.retirement > 0).then(|| RetirementContainerCapacityKnob::new(self.retirement).expect("1..=959")),
        }
    }
}

pub fn violations(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<String> {
    verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            _ => None,
        })
        .collect()
}

pub fn verdict_name(verdicts: &[(&'static str, InvariantVerdict)], invariant: &str) -> String {
    let verdict = &verdicts.iter().find(|(name, _)| *name == invariant).expect("清单里有").1;
    match verdict {
        InvariantVerdict::Holds => "Holds".to_string(),
        InvariantVerdict::Violated(_) => "Violated".to_string(),
        InvariantVerdict::NotApplicable(_) => "NotApplicable".to_string(),
        InvariantVerdict::PartiallyJudged(_) => "PartiallyJudged".to_string(),
    }
}

/// 一个池此刻的写入口状态：盘、分配器、现行那一版、这一进程的实例。
pub struct Pool<D: BlockDevice> {
    pub devices: Vec<(DeviceIdentity, D)>,
    pub allocator: PoolAllocator,
    pub output: TransactionOutput,
    pub instance: InstanceGeneration,
}

impl Clone for Pool<SparseBlockDevice> {
    fn clone(&self) -> Self {
        Self {
            devices: sparse_devices_of(&image_of(&self.devices)),
            allocator: self.allocator.clone(),
            output: self.output.clone(),
            instance: self.instance,
        }
    }
}

impl Pool<SparseBlockDevice> {
    /// mkfs、暖机、第一个文件（txg 3）。
    pub fn after_the_first_file() -> Self {
        let parameters = parameters();
        let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| (identity, SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512))))
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let content = first_file_content();
        let output = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("取号");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两条根"),
                FirstFile { content: &content, write_time_seconds: WRITE_TIME_SECONDS },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件")
        };
        Self { devices, allocator, output, instance: FIRST_INSTANCE }
    }

    pub fn image(&self) -> MemoryPool {
        image_of(&self.devices)
    }

    /// 崩溃之后（或任何一份镜像）按旋钮可写挂载：交回挂载之后的池（实例是这次挂载取的号）。
    pub fn mounted_from(image: &MemoryPool, knobs: Knobs) -> Result<Self, MountError> {
        let mut devices = sparse_devices_of(image);
        let Mounted { allocator, current, .. } = mount_writable_with_test_only_switches_and_deletion_switches(
            &parameters(),
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            knobs.switches(),
        )?;
        let instance = current.root().instance;
        let output = current.version_of_trees().expect("这几条流上每一版都有树").clone();
        Ok(Self { devices, allocator, output, instance })
    }
}

impl<D: BlockDevice> Pool<D> {
    pub fn create(&mut self, count: u64) {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let output = publish_new_inodes(&mut writer, &mut self.allocator, &self.output, count, WRITE_TIME_SECONDS, self.instance)
            .expect("建空 inode");
        self.output = output;
    }

    pub fn unlink(&mut self, inode: u64, knobs: Knobs) {
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        writer.set_container_tree_capacities(knobs.capacities());
        let mut current = publish_unlink_intent(&mut writer, &mut self.allocator, &self.output, InodeNumber(inode), WRITE_TIME_SECONDS, self.instance)
            .unwrap_or_else(|refusal| panic!("unlink {inode} 意图：{refusal:?}"));
        publish_unlink_steps_until_the_queue_is_empty(&mut writer, &mut self.allocator, &mut current, UnlinkBatchSize::OneExtentLowerLeaf, self.instance)
            .unwrap_or_else(|refusal| panic!("unlink {inode} 各批：{refusal:?}"));
        self.output = current;
    }

    /// 挂着回退到 `target`（实例、txg）：拒了交回拒绝（回退在任何写之前拒，盘不动）。
    pub fn roll_back(&mut self, target: (InstanceGeneration, u64)) -> Result<(), String> {
        let parameters = parameters();
        let mut current = self.output.clone();
        match roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut current,
            RollbackTarget { instance: target.0, checkpoint_txg: singlefs_core::address::CheckpointTxg(target.1) },
        ) {
            Ok(_) => {
                self.output = current;
                Ok(())
            }
            Err(refusal) => Err(format!("{refusal:?}")),
        }
    }

    pub fn live_inodes(&self) -> Vec<u64> {
        self.output
            .inode_leaf_containers
            .iter()
            .flat_map(|leaf| leaf.contents.records.iter().map(|record| record.inode))
            .filter(|inode| *inode != 1)
            .collect()
    }

    pub fn tree_summary(&self) -> String {
        let numbers = |identities: Vec<(u64, usize)>| format!("{identities:?}");
        format!(
            "tombstone={} lineage={} retirement={} leaves={:?}",
            numbers(self.output.tombstone_containers.iter().map(|c| (c.contents.identity.container, c.contents.records.len())).collect()),
            numbers(self.output.lineage_containers.iter().map(|c| (c.contents.identity.container, c.contents.records.len())).collect()),
            numbers(self.output.retirement_containers.iter().map(|c| (c.contents.identity.container, c.contents.records.len())).collect()),
            self.output.inode_leaf_containers.iter().map(|leaf| leaf.contents.records.len()).collect::<Vec<_>>(),
        )
    }

    pub fn txg(&self) -> u64 {
        self.output.root.checkpoint_txg.0
    }
}
