//! m3-batch23-code-r1 攻方原型（只在仓副本里编、跑；不入库代码）。
//! 盘用内存稀疏盘（`SparseBlockDevice`），起点池建一次、之后每段历史从内存里那一份拷。
//! 池与用户动作那一段照 `research/prompts/m3-batch22-code-r1-opus-model/opus_b22_attack.rs` 的 `SparsePool` 抄来改的；
//! GPU 扫描内核墓碑与谱系那一遍按 `crash_judge_checker_scan.wgsl` 逐函数译成 Rust（`gpu_model`），GPU 不跑。
//! 输出一律是 `RESULT ` 打头的行，`rerun.sh compare` 只比这些行（排序后）。
#![allow(dead_code, clippy::all, clippy::pedantic, unused_imports)]

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::collections::{BTreeMap, BTreeSet};

use common::{file_content, named_unit_bytes, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::{attack_capture, check_pool_image};
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write,
    publish_unlink_intent, publish_unlink_step, publish_unlink_steps_until_the_queue_is_empty,
    warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const WT: u64 = 1_788_000_100;

fn new_device() -> SparseBlockDevice {
    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

/// 两块内存稀疏盘上的池：设备、现行版本、分配器、只供测试的删除开关（`None` = 产品容量）。
pub struct SparsePool {
    pub devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
    pub switches: Option<singlefs_core::unlink::DeletionSwitches>,
}

impl Clone for SparsePool {
    fn clone(&self) -> Self {
        Self {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| {
                    let mut copy = new_device();
                    copy.image = device.image.clone();
                    (*identity, copy)
                })
                .collect(),
            output: self.output.clone(),
            allocator: self.allocator.clone(),
            switches: self.switches,
        }
    }
}

impl SparsePool {
    /// mkfs → 取号 → 暖机 → 新池新建文件（与 `common::build_pool` 同一串，盘换成内存稀疏盘）。
    pub fn build() -> Self {
        let parameters = parameters();
        let mut devices = vec![(DeviceIdentity(0), new_device()), (DeviceIdentity(1), new_device())];
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
            Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
        let content = file_content();
        let output = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut pool,
                &mut allocator,
                warm.roots.last().expect("暖机两代根"),
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warm.last_record_bytes,
            )
            .expect("新池新建文件")
        };
        Self { devices, output, allocator, switches: None }
    }

    pub fn image(&self) -> MemoryPool {
        let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        for (identity, device) in &self.devices {
            image.devices.insert(*identity, device.image.clone());
        }
        image
    }

    pub fn instance(&self) -> InstanceGeneration {
        self.output.root.instance
    }

    pub fn here(&self) -> (InstanceGeneration, u64) {
        (self.instance(), self.output.root.checkpoint_txg.0)
    }

    pub fn queue(&self) -> Vec<u64> {
        self.output.deleted_inodes.iter().map(|inode| inode.0).collect()
    }

    /// (号, nlink, 对象出生代)。
    pub fn records(&self) -> Vec<(u64, u32, u64)> {
        self.output
            .inode_leaf_container_contents()
            .iter()
            .flat_map(|container| container.records.iter())
            .map(|record| (record.inode, record.nlink, record.object_birth.0))
            .collect()
    }

    fn with_writer<T>(
        &mut self,
        act: impl FnOnce(&mut PoolWriter<'_, SparseBlockDevice>, &mut PoolAllocator, &TransactionOutput, InstanceGeneration) -> Result<T, String>,
    ) -> Result<T, String> {
        let parameters = parameters();
        let previous = self.output.clone();
        let instance = self.instance();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        if let Some(switches) = self.switches {
            writer.set_container_tree_capacities(switches.container_tree_capacities());
        }
        act(&mut writer, &mut self.allocator, &previous, instance)
    }
}

/// 用户动作（放开扫的那几步）。失败的交回 `Err`，调用方把池换回动作之前那一份。
impl SparsePool {
    pub fn create(&mut self, count: u64) -> Result<(), String> {
        self.drain()?;
        let out = self.with_writer(|writer, allocator, previous, instance| {
            publish_new_inodes(writer, allocator, previous, count, WT, instance).map_err(|e| format!("{e:?}"))
        })?;
        self.output = out;
        Ok(())
    }

    /// 顺序写第一个文件成 `units` 个数据单元。
    pub fn grow_the_first_file(&mut self, units: usize) -> Result<(), String> {
        let capacity = singlefs_core::unit::data_unit_payload_capacity();
        let content: Vec<u8> = (0..(units - 1) * capacity + capacity / 2).map(|i| (i % 251) as u8).collect();
        let out = self.with_writer(|writer, allocator, previous, instance| {
            publish_sequential_write(
                writer,
                allocator,
                previous,
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
                instance,
            )
            .map_err(|e| format!("{e:?}"))
        })?;
        self.output = out;
        Ok(())
    }

    /// unlink 意图：先把队列走空（会话纪律）。
    pub fn intent(&mut self, inode: u64) -> Result<(), String> {
        self.drain()?;
        let out = self.with_writer(|writer, allocator, previous, instance| {
            publish_unlink_intent(writer, allocator, previous, InodeNumber(inode), WT, instance).map_err(|e| format!("{e:?}"))
        })?;
        self.output = out;
        Ok(())
    }

    /// 队列里那个号按批大小 1 删一批。
    pub fn step(&mut self) -> Result<(), String> {
        let Some(inode) = self.queue().first().copied() else { return Err("队列空".to_string()) };
        let out = self.with_writer(|writer, allocator, previous, instance| {
            publish_unlink_step(
                writer,
                allocator,
                previous,
                InodeNumber(inode),
                UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
                instance,
            )
            .map_err(|e| format!("{e:?}"))
        })?;
        self.output = out;
        Ok(())
    }

    pub fn drain(&mut self) -> Result<(), String> {
        if self.queue().is_empty() {
            return Ok(());
        }
        let mut current = self.output.clone();
        self.with_writer(|writer, allocator, _previous, instance| {
            publish_unlink_steps_until_the_queue_is_empty(writer, allocator, &mut current, UnlinkBatchSize::OneExtentLowerLeaf, instance)
                .map(|_| ())
                .map_err(|e| format!("{e:?}"))
        })?;
        self.output = current;
        Ok(())
    }

    /// 管理员回退到 `target` 那一版（一次向前发布）。
    pub fn rollback(&mut self, target: (InstanceGeneration, u64)) -> Result<(), String> {
        let parameters = parameters();
        roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: target.0, checkpoint_txg: singlefs_core::address::CheckpointTxg(target.1) },
        )
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
    }

    /// 进程退出（什么都已持久）之后再可写挂载：新实例写行、暖机，删除续做。
    pub fn remount(&mut self) -> Result<(), String> {
        let parameters = parameters();
        let mounted = mount_with(&parameters, &mut self.devices, self.switches).map_err(|e| format!("{e:?}"))?;
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.version_of_trees().ok_or_else(|| "落到的那一版树表 0 条".to_string())?.clone();
        Ok(())
    }

    /// 崩溃恢复落到最新那条根之前那条根、最新那条根之后又读得出（照 m3-batch22 攻方模型的 `abandon_the_newest`）：
    /// 最新那次发布的根槽与它那条记录点名的单元一时读不出、见证它的系统配置轮换没落盘；挂载写到了藏起来的地方的算做不成。
    pub fn abandon_the_newest(&mut self) -> Result<(), String> {
        let parameters = parameters();
        let newest = self.output.clone();
        let target = singlefs_core::root_ring::target_for_publish(newest.root.checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
        let root_slot_device = parameters.region_devices[target.region as usize];
        let root_slot_offset = singlefs_core::root_ring::slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
        let mut places: Vec<(DeviceIdentity, singlefs_core::address::DeviceOffsetInBytes, usize)> =
            vec![(root_slot_device, root_slot_offset, parameters.geometry.physical_block_size as usize)];
        for named in &newest.record.named {
            for location in named.locations {
                places.push((location.device, location.slot.to_device_offset(), named_unit_bytes(named.unit_class)));
            }
        }
        let index_of = |devices: &[(DeviceIdentity, SparseBlockDevice)], identity: DeviceIdentity| {
            devices.iter().position(|(candidate, _)| *candidate == identity).expect("池里有这块盘")
        };
        let mut saved = Vec::new();
        for (identity, offset, length) in places {
            let index = index_of(&self.devices, identity);
            let mut bytes = vec![0u8; length];
            self.devices[index].1.read_at(offset, &mut bytes).map_err(|e| format!("{e:?}"))?;
            self.devices[index].1.write_at(offset, &vec![0u8; length], WriteDurability::Plain).map_err(|e| format!("{e:?}"))?;
            saved.push((identity, offset, bytes));
        }
        let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
        let mut rotations = Vec::new();
        for (identity, _) in &self.devices {
            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(
                self.devices.as_slice(),
                *identity,
                spacing,
                &parameters.filesystem_identifier,
            );
            verified.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest_slot] = <[_; 2]>::try_from(verified).map_err(|_| "两槽都自证".to_string())?;
            let generation = newest_slot.quantities.slot_generation;
            let mut not_written = older;
            not_written.quantities.slot_generation = generation.checked_sub(2).ok_or("轮换次数不够")?;
            rotations.push((
                singlefs_core::address::DeviceOffsetInBytes((generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing),
                not_written.to_slot(),
            ));
        }
        for ((_, device), (offset, bytes)) in self.devices.iter_mut().zip(rotations) {
            device.write_at(offset, &bytes, WriteDurability::Plain).map_err(|e| format!("{e:?}"))?;
        }
        let mounted = mount_with(&parameters, &mut self.devices, self.switches);
        for (identity, offset, bytes) in &saved {
            let index = index_of(&self.devices, *identity);
            let mut now = vec![0u8; bytes.len()];
            self.devices[index].1.read_at(*offset, &mut now).map_err(|e| format!("{e:?}"))?;
            if now.iter().any(|byte| *byte != 0) {
                return Err(format!("挂载写到了藏起来的盘 {} 偏移 {}", identity.0, offset.0));
            }
        }
        for (identity, offset, bytes) in &saved {
            let index = index_of(&self.devices, *identity);
            self.devices[index].1.write_at(*offset, bytes, WriteDurability::Plain).map_err(|e| format!("{e:?}"))?;
        }
        let mounted = mounted.map_err(|e| format!("{e:?}"))?;
        if mounted.output.effective_root.checkpoint_txg >= newest.root.checkpoint_txg {
            return Err(format!("恢复没落到前一条根（落到 txg {}）", mounted.output.effective_root.checkpoint_txg.0));
        }
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.version_of_trees().ok_or("落到的那一版树表 0 条")?.clone();
        Ok(())
    }
}

/// 可写挂载：装了删除开关的走带开关的入口（续做与之后的会话照它），没装的走产品入口。
fn mount_with(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    switches: Option<singlefs_core::unlink::DeletionSwitches>,
) -> Result<singlefs_core::mount::Mounted, singlefs_core::mount::MountError> {
    match switches {
        None => mount_writable(parameters, devices),
        Some(switches) => singlefs_core::mount::mount_writable_with_test_only_switches_and_deletion_switches(
            parameters,
            devices,
            singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
            singlefs_core::mount::ShadowLedger::On,
            switches,
        ),
    }
}

/// 墓碑与谱系容器一片 `records` 条、码 2 节点 `entries` 条、删除批大小 1 的开关（树会长高、容器会多片）。
fn switches_of(records: u64, entries: usize) -> singlefs_core::unlink::DeletionSwitches {
    singlefs_core::unlink::DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
        tombstone_container_capacity: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(records).expect("1..=815"),
        ),
        lineage_container_capacity: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(records).expect("1..=2039"),
        ),
        container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(entries).expect("2..=135"),
        ),
    }
}

/// 池级 checker 旧入口（层 0 与放量走它，GPU 那一侧也只有它）判一张镜像：每条的判定种类与说明，连同钩子记下的版本清单与滤法。
pub struct CpuVerdicts {
    pub kinds: BTreeMap<&'static str, (&'static str, String)>,
    pub versions: Vec<(u64, Vec<u8>)>,
    pub filtered: Option<bool>,
}

impl CpuVerdicts {
    pub fn of(image: &MemoryPool) -> Self {
        let verdicts = check_pool_image(image);
        let captured = attack_capture::last();
        let kinds = verdicts
            .into_iter()
            .map(|(invariant, verdict)| {
                let kind = match verdict {
                    InvariantVerdict::Holds => ("成立", String::new()),
                    InvariantVerdict::NotApplicable(reason) => ("不适用", reason.to_string()),
                    InvariantVerdict::PartiallyJudged(detail) => ("部分没判", detail),
                    InvariantVerdict::Violated(detail) => ("不成立", detail),
                };
                (invariant, kind)
            })
            .collect();
        let (versions, filtered) = match captured {
            Some((versions, filtered)) => (versions, Some(filtered)),
            None => (Vec::new(), None),
        };
        Self { kinds, versions, filtered }
    }
    pub fn violated(&self) -> Vec<&'static str> {
        self.kinds.iter().filter(|(_, (kind, _))| *kind == "不成立").map(|(name, _)| *name).collect()
    }
    pub fn kind(&self, invariant: &str) -> &'static str {
        self.kinds.get(invariant).map(|(kind, _)| *kind).unwrap_or("没有这一条")
    }
    pub fn detail(&self, invariant: &str) -> &str {
        self.kinds.get(invariant).map(|(_, detail)| detail.as_str()).unwrap_or("")
    }
}

pub struct SplitMix(pub u64);
impl SplitMix {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound.max(1) as u64) as usize
    }
}

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn get_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2"))
}
fn get_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"))
}
fn get_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
}
fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// GPU 扫描内核墓碑与谱系那一遍、删除态扫描里 I-9.24 那一判，按 `crash_judge_checker_scan.wgsl` 逐函数译成 Rust（函数名照内核，去掉 `d_` 前缀）。
/// 读单元照内核的 `read_unit_via_locations`（两条位置条目里第一份整单元 CRC 对得上的那一份）；解码借池级 checker 的 `index_node_view` /
/// `packed_unit_view`（内核的 `parse_index_node` / `parse_packed_unit` 是先前就有、对拍过的那一份，这里不再译）。版本清单取 CPU 那一份（钩子记下的）。
pub mod gpu_model {
    use super::*;
    use singlefs_checker::image::{parse_node_pointer, PointerView};

    const NODE_BYTES: usize = 16384;
    const DATA_UNIT_BYTES: usize = 32768;
    const SLOT_BYTES: u64 = 16384;
    const TREE_TABLE_ENTRY_BYTES: usize = 200;
    const KIND_OFFSET: usize = 10;
    const ROOT_POINTER_OFFSET: usize = 14;
    const HEAD_IDENTIFIER_OFFSET: usize = 116;
    const CONTAINER_TREE_ENTRY_BYTES: usize = 120;
    const ENTRY_TYPE_OFFSET: usize = 16;
    const CHILD_POINTER_OFFSET: usize = 34;
    const KIND_INODE: u16 = 2;
    const KIND_DELETED_INODES: u16 = 9;
    const KIND_TOMBSTONE: u16 = 10;
    const KIND_LINEAGE: u16 = 11;
    const PACKED_TOMBSTONE: u16 = 1;
    const PACKED_INODE: u16 = 2;
    const PACKED_LINEAGE: u16 = 5;
    const TOMBSTONE_BYTES: usize = 40;
    const LINEAGE_BYTES: usize = 16;
    const INODE_RECORD_BYTES: usize = 140;
    const TOMBSTONE_CONTAINER_RECORDS: usize = 815;
    const MAX_INODE_RECORDS: usize = 1024;
    const MAX_PENDING: usize = 512;
    const MAX_CONTAINERS: usize = 256;
    const MAX_LINEAGE_RECORDS: usize = 256;
    const MAX_OBJECT_BIRTHS: usize = 2048;
    const MAX_TOMBSTONES_COMPARED_PAIRWISE: usize = 4096;

    pub struct Model<'a> {
        image: &'a MemoryPool,
        pub violated: BTreeSet<&'static str>,
        /// 第一处判不了的站点号（内核写进结论的 `UNDECIDABLE_TOO_MANY_ENTRIES | (站点 << 8)` 里那个站点）。
        pub undecidable: Option<u32>,
        /// 自证用的故障（`OPUS_GPU_MODEL_FAULT`）：译本里故意改坏一处，看对拍能不能红。
        fault: String,
        inode_records: Vec<(u64, u32, u64)>,
        containers: Vec<(Vec<Vec<u8>>, u64)>,
        lineage: Vec<(u64, u64)>,
        births: Vec<((u64, u64), u64)>,
    }

    fn read_via_locations(image: &MemoryPool, pointer: &PointerView, length: usize) -> Option<Vec<u8>> {
        pointer.locations.iter().find_map(|location| {
            image
                .read(location.device, location.slot * SLOT_BYTES, length)
                .filter(|bytes| singlefs_checker::crc32_castagnoli_table(bytes) == location.checksum)
        })
    }

    impl<'a> Model<'a> {
        pub fn new(image: &'a MemoryPool) -> Self {
            Self {
                image,
                violated: BTreeSet::new(),
                undecidable: None,
                fault: std::env::var("OPUS_GPU_MODEL_FAULT").unwrap_or_default(),
                inode_records: Vec::new(),
                containers: Vec::new(),
                lineage: Vec::new(),
                births: Vec::new(),
            }
        }
        fn violate(&mut self, invariant: &'static str) {
            self.violated.insert(invariant);
        }
        fn undecided(&mut self, site: u32) {
            self.undecidable.get_or_insert(site);
        }
        fn read_index_node(&self, pointer: &PointerView) -> Option<singlefs_checker::IndexNodeView> {
            read_via_locations(self.image, pointer, NODE_BYTES).and_then(|unit| singlefs_checker::index_node_view(&unit).ok())
        }
        fn read_packed(&self, pointer: &PointerView) -> Option<singlefs_checker::PackedUnitView> {
            read_via_locations(self.image, pointer, DATA_UNIT_BYTES).and_then(|unit| singlefs_checker::packed_unit_view(&unit).ok())
        }

        /// d_note_inode_record：同一个号已有一行就盖掉；表满判不了（站点 49）。
        fn note_inode_record(&mut self, inode: u64, nlink: u32, object_birth: u64) -> bool {
            if let Some(row) = self.inode_records.iter_mut().find(|row| row.0 == inode) {
                row.1 = nlink;
                row.2 = object_birth;
                return true;
            }
            if self.inode_records.len() >= MAX_INODE_RECORDS {
                self.undecided(49);
                return false;
            }
            self.inode_records.push((inode, nlink, object_birth));
            true
        }

        /// d_push_container_tree_entries_in_reverse：条目倒着压栈（弹出来是条目序）。
        fn push_in_reverse(&mut self, pending: &mut Vec<(Vec<u8>, u8)>, node: &singlefs_checker::IndexNodeView, site: u32) -> bool {
            for entry in node.entries.iter().rev() {
                if pending.len() >= MAX_PENDING {
                    self.undecided(site);
                    return false;
                }
                pending.push((entry.clone(), node.level));
            }
            true
        }

        /// d_read_inode_records（站点 50 / 51 是压栈越界）。
        fn read_inode_records(&mut self, root_pointer: &PointerView) -> bool {
            self.inode_records.clear();
            if root_pointer.all_zero {
                return true;
            }
            let Some(root) = self.read_index_node(root_pointer) else { return false };
            if root.level == 0 {
                return root.entries.is_empty();
            }
            if root.entry_width < CONTAINER_TREE_ENTRY_BYTES {
                return false;
            }
            let mut pending = Vec::new();
            if !self.push_in_reverse(&mut pending, &root, 50) {
                return false;
            }
            while let Some((entry, parent_level)) = pending.pop() {
                let child_pointer = parse_node_pointer(&entry[CHILD_POINTER_OFFSET..CHILD_POINTER_OFFSET + 86]);
                let entry_type = get_u16(&entry, ENTRY_TYPE_OFFSET);
                if entry_type == PACKED_INODE {
                    if child_pointer.all_zero {
                        return false;
                    }
                    let Some(container) = self.read_packed(&child_pointer) else { return false };
                    if container.record_type != PACKED_INODE || container.record_width < INODE_RECORD_BYTES {
                        return false;
                    }
                    for record in &container.records {
                        if !self.note_inode_record(get_u64(record, 0), get_u32(record, 36), get_u64(record, 8)) {
                            return false;
                        }
                    }
                } else if entry_type == 0 {
                    if child_pointer.all_zero {
                        return false;
                    }
                    let Some(child) = self.read_index_node(&child_pointer) else { return false };
                    if u32::from(child.level) + 1 != u32::from(parent_level) || child.level == 0 || child.entry_width < CONTAINER_TREE_ENTRY_BYTES {
                        return false;
                    }
                    if !self.push_in_reverse(&mut pending, &child, 51) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }

        /// d_read_containers_of_a_container_tree（站点 55 / 56 压栈越界，57 容器多于 256 片）。
        fn read_containers(&mut self, root_pointer: &PointerView, record_type: u16, registered_record_bytes: usize) -> bool {
            self.containers.clear();
            if root_pointer.all_zero {
                return true;
            }
            let Some(root) = self.read_index_node(root_pointer) else { return false };
            if root.level == 0 || root.entry_width < CONTAINER_TREE_ENTRY_BYTES {
                return false;
            }
            let mut pending = Vec::new();
            if !self.push_in_reverse(&mut pending, &root, 55) {
                return false;
            }
            while let Some((entry, parent_level)) = pending.pop() {
                let child_pointer = parse_node_pointer(&entry[CHILD_POINTER_OFFSET..CHILD_POINTER_OFFSET + 86]);
                let entry_type = get_u16(&entry, ENTRY_TYPE_OFFSET);
                if entry_type == 0 {
                    if child_pointer.all_zero {
                        return false;
                    }
                    let Some(child) = self.read_index_node(&child_pointer) else { return false };
                    if u32::from(child.level) + 1 != u32::from(parent_level) || child.level == 0 || child.entry_width < CONTAINER_TREE_ENTRY_BYTES {
                        return false;
                    }
                    if !self.push_in_reverse(&mut pending, &child, 56) {
                        return false;
                    }
                } else if entry_type == record_type {
                    if child_pointer.all_zero {
                        return false;
                    }
                    let Some(container) = self.read_packed(&child_pointer) else { return false };
                    if container.record_type != record_type || container.record_width != registered_record_bytes {
                        return false;
                    }
                    if self.containers.len() >= MAX_CONTAINERS {
                        self.undecided(57);
                        return false;
                    }
                    self.containers.push((container.records.clone(), container.container));
                } else {
                    return false;
                }
            }
            true
        }

        /// d_read_the_lineage_of_a_version（站点 58：谱系记录多于 256 条）。
        fn read_the_lineage_of_a_version(&mut self, entries: &[Vec<u8>]) -> bool {
            self.lineage.clear();
            let lineage_entries: Vec<&Vec<u8>> = entries.iter().filter(|entry| get_u16(entry, KIND_OFFSET) == KIND_LINEAGE).collect();
            if lineage_entries.len() != 1 {
                return false;
            }
            let root_pointer = parse_node_pointer(&lineage_entries[0][ROOT_POINTER_OFFSET..ROOT_POINTER_OFFSET + 86]);
            if !self.read_containers(&root_pointer, PACKED_LINEAGE, LINEAGE_BYTES) {
                return false;
            }
            let containers = self.containers.clone();
            for (records, _) in &containers {
                for record in records {
                    if self.lineage.len() >= MAX_LINEAGE_RECORDS {
                        self.undecided(58);
                        return false;
                    }
                    self.lineage.push((get_u64(record, 0), get_u64(record, 8)));
                }
            }
            // 内核是选择排序、按 T_rb 从大到小；同 T_rb 的两条下面判接不成链，次序不影响结局。
            self.lineage.sort_by(|a, b| b.0.cmp(&a.0));
            for index in 0..self.lineage.len() {
                if !(self.lineage[index].1 < self.lineage[index].0) {
                    return false;
                }
                if index > 0 && self.lineage[index - 1].0 == self.lineage[index].0 {
                    return false;
                }
            }
            true
        }

        /// d_txg_is_on_the_lineage。
        fn on_lineage(&self, txg: u64, version_txg: u64) -> bool {
            let mut walked_to = version_txg;
            for (rollback_txg, parent_txg) in &self.lineage {
                if walked_to < *rollback_txg {
                    continue;
                }
                if *rollback_txg <= txg && txg <= walked_to {
                    return true;
                }
                walked_to = *parent_txg;
            }
            txg <= walked_to
        }

        /// d_note_object_birth（站点 59：跨版本那张表多于 2048 个 (树, 号)）。
        fn note_object_birth(&mut self, inode_tree: u64, inode: u64, object_birth: u64) {
            if let Some((_, first_birth)) = self.births.iter().find(|(key, _)| *key == (inode_tree, inode)) {
                if *first_birth != object_birth {
                    self.violate("I-9.29");
                }
                return;
            }
            if self.births.len() >= MAX_OBJECT_BIRTHS {
                self.undecided(59);
                return;
            }
            self.births.push(((inode_tree, inode), object_birth));
        }

        fn covers_the_whole_object(record: &[u8]) -> bool {
            let (start, end) = (get_u64(record, 16), get_u64(record, 24));
            start < end && start == 0 && end == u64::MAX
        }

        /// d_judge_tombstone_container_capacity（I-9.21 没装旋钮那一支）。
        fn judge_tombstone_container_capacity(&mut self) {
            if self.containers.len() < 2 {
                return;
            }
            for index in 1..self.containers.len() {
                if !(self.containers[index - 1].1 < self.containers[index].1) {
                    self.violate("I-9.21");
                }
            }
            for index in 0..self.containers.len() - 1 {
                if self.containers[index].0.len() != TOMBSTONE_CONTAINER_RECORDS {
                    self.violate("I-9.21");
                }
            }
        }

        /// d_judge_every_tombstone_is_well_formed。
        fn judge_every_tombstone_is_well_formed(&mut self, version_txg: u64) {
            let death_bound = if self.fault == "death-bound-plus-one" { version_txg.saturating_add(1) } else { version_txg };
            for (records, _) in self.containers.clone() {
                if records.len() > TOMBSTONE_CONTAINER_RECORDS {
                    self.violate("I-9.20");
                }
                for record in &records {
                    if !Self::covers_the_whole_object(record) {
                        self.violate("I-9.20");
                    }
                    let death_txg = get_u64(record, 32);
                    if !(get_u64(record, 8) <= death_txg) || !(death_txg <= death_bound) {
                        self.violate("I-9.20");
                    }
                }
            }
        }

        /// d_judge_tombstones_on_the_lineage_are_distinct（站点 60：一棵墓碑树多于 4096 条）。
        fn judge_tombstones_on_the_lineage_are_distinct(&mut self, version_txg: u64) {
            let all: Vec<Vec<u8>> = self.containers.iter().flat_map(|(records, _)| records.iter().cloned()).collect();
            if all.len() > MAX_TOMBSTONES_COMPARED_PAIRWISE {
                self.undecided(60);
                return;
            }
            for (index, record) in all.iter().enumerate() {
                if !self.on_lineage(get_u64(record, 32), version_txg) {
                    continue;
                }
                for earlier in &all[..index] {
                    if get_u64(earlier, 0) == get_u64(record, 0)
                        && get_u64(earlier, 8) == get_u64(record, 8)
                        && self.on_lineage(get_u64(earlier, 32), version_txg)
                    {
                        self.violate("I-9.20");
                    }
                }
            }
        }

        fn inode_record_of(&self, inode: u64) -> Option<(u64, u32, u64)> {
            self.inode_records.iter().copied().find(|row| row.0 == inode)
        }

        /// d_judge_tombstones_against_the_records（I-9.10 墓碑那一个等号、I-9.19、谱系上的墓碑进 I-9.29 那张表）。
        fn judge_tombstones_against_the_records(&mut self, inode_tree: u64, version_txg: u64, lineage_holds: bool) {
            let all: Vec<Vec<u8>> = self.containers.iter().flat_map(|(records, _)| records.iter().cloned()).collect();
            for record in &all {
                let (object, object_birth) = (get_u64(record, 0), get_u64(record, 8));
                let row = self.inode_record_of(object);
                if let Some(row) = row {
                    if row.2 != object_birth {
                        self.violate("I-9.10");
                    }
                }
                let on = self.on_lineage(get_u64(record, 32), version_txg) || self.fault == "no-lineage-filter-29";
                if !lineage_holds || !on {
                    continue;
                }
                if let Some(row) = row {
                    if row.2 == object_birth && row.1 != 0 {
                        self.violate("I-9.19");
                    }
                }
                self.note_object_birth(inode_tree, object, object_birth);
            }
        }

        /// d_read_deletion_queue：(读得出, 有根, 号)。
        fn read_deletion_queue(&self, entry: &[u8]) -> (bool, bool, Vec<u64>) {
            let pointer = parse_node_pointer(&entry[ROOT_POINTER_OFFSET..ROOT_POINTER_OFFSET + 86]);
            if pointer.all_zero {
                return (true, false, Vec::new());
            }
            match self.read_index_node(&pointer) {
                Some(queue) if queue.level == 0 && queue.key_width == 8 && queue.entry_width >= 8 => {
                    (true, true, queue.entries.iter().map(|entry| get_u64(entry, 0)).collect())
                }
                _ => (false, true, Vec::new()),
            }
        }

        /// d_judge_queued_inodes_have_tombstones_on_the_lineage（I-9.18）。
        fn judge_queued_inodes_have_tombstones_on_the_lineage(&mut self, entries: &[Vec<u8>], inode_tree_entry: &[u8], version_txg: u64, lineage_holds: bool) {
            let inode_tree = get_u64(inode_tree_entry, 0);
            let deleted: Vec<&Vec<u8>> = entries
                .iter()
                .filter(|entry| get_u16(entry, KIND_OFFSET) == KIND_DELETED_INODES && get_u64(entry, HEAD_IDENTIFIER_OFFSET) == inode_tree)
                .collect();
            let tombstone: Vec<&Vec<u8>> = entries
                .iter()
                .filter(|entry| get_u16(entry, KIND_OFFSET) == KIND_TOMBSTONE && get_u64(entry, HEAD_IDENTIFIER_OFFSET) == inode_tree)
                .collect();
            if deleted.len() != 1 {
                return;
            }
            let (readable, present, queue) = self.read_deletion_queue(deleted[0]);
            if !readable || !present || queue.is_empty() {
                return;
            }
            let inode_root = parse_node_pointer(&inode_tree_entry[ROOT_POINTER_OFFSET..ROOT_POINTER_OFFSET + 86]);
            if !self.read_inode_records(&inode_root) {
                return;
            }
            if tombstone.len() != 1 {
                return;
            }
            let tombstone_root = parse_node_pointer(&tombstone[0][ROOT_POINTER_OFFSET..ROOT_POINTER_OFFSET + 86]);
            if !self.read_containers(&tombstone_root, PACKED_TOMBSTONE, TOMBSTONE_BYTES) {
                return;
            }
            if !lineage_holds {
                return;
            }
            let all: Vec<Vec<u8>> = self.containers.iter().flat_map(|(records, _)| records.iter().cloned()).collect();
            for inode in queue {
                let Some(row) = self.inode_record_of(inode) else { continue };
                let found = all.iter().any(|record| {
                    get_u64(record, 0) == inode
                        && get_u64(record, 8) == row.2
                        && (Self::covers_the_whole_object(record) || self.fault == "no-whole-object-18")
                        && self.on_lineage(get_u64(record, 32), version_txg)
                });
                if !found {
                    self.violate("I-9.18");
                }
            }
        }

        /// d_judge_tombstones_and_the_lineage_of_a_version。
        fn judge_tombstones_and_the_lineage_of_a_version(&mut self, tree_table_pointer: &[u8], version_txg: u64) {
            let pointer = parse_node_pointer(tree_table_pointer);
            if pointer.all_zero {
                return;
            }
            let Some(tt) = self.read_index_node(&pointer) else { return };
            if tt.entries.is_empty() || tt.entry_width < TREE_TABLE_ENTRY_BYTES {
                return;
            }
            let entries = tt.entries.clone();
            let lineage_holds = self.read_the_lineage_of_a_version(&entries);
            for entry in &entries {
                if get_u16(entry, KIND_OFFSET) != KIND_INODE {
                    continue;
                }
                let root = parse_node_pointer(&entry[ROOT_POINTER_OFFSET..ROOT_POINTER_OFFSET + 86]);
                if !self.read_inode_records(&root) {
                    continue;
                }
                let inode_tree = get_u64(entry, 0);
                for (inode, _, object_birth) in self.inode_records.clone() {
                    self.note_object_birth(inode_tree, inode, object_birth);
                }
            }
            for entry in &entries {
                if get_u16(entry, KIND_OFFSET) != KIND_TOMBSTONE {
                    continue;
                }
                let root = parse_node_pointer(&entry[ROOT_POINTER_OFFSET..ROOT_POINTER_OFFSET + 86]);
                if !self.read_containers(&root, PACKED_TOMBSTONE, TOMBSTONE_BYTES) {
                    continue;
                }
                self.judge_tombstone_container_capacity();
                self.judge_every_tombstone_is_well_formed(version_txg);
                if lineage_holds {
                    self.judge_tombstones_on_the_lineage_are_distinct(version_txg);
                }
                let head_identifier = get_u64(entry, HEAD_IDENTIFIER_OFFSET);
                let paired: Vec<&Vec<u8>> = entries
                    .iter()
                    .filter(|other| get_u16(other, KIND_OFFSET) == KIND_INODE && get_u64(other, 0) == head_identifier)
                    .collect();
                if paired.len() != 1 {
                    continue;
                }
                let paired_root = parse_node_pointer(&paired[0][ROOT_POINTER_OFFSET..ROOT_POINTER_OFFSET + 86]);
                if !self.read_inode_records(&paired_root) {
                    continue;
                }
                self.judge_tombstones_against_the_records(head_identifier, version_txg, lineage_holds);
            }
            for entry in &entries {
                if get_u16(entry, KIND_OFFSET) == KIND_INODE {
                    self.judge_queued_inodes_have_tombstones_on_the_lineage(&entries, entry, version_txg, lineage_holds);
                }
            }
        }

        /// 删除态扫描（d_judge_deletion_state_of_a_version）里只译 I-9.24 那一判：每条 deleted_inodes 条目读得出时号至多一个（读队列在配对之前）。
        fn judge_deletion_queue_bound_of_a_version(&mut self, tree_table_pointer: &[u8]) {
            let pointer = parse_node_pointer(tree_table_pointer);
            if pointer.all_zero {
                return;
            }
            let Some(tt) = self.read_index_node(&pointer) else { return };
            if tt.entries.is_empty() || tt.entry_width < TREE_TABLE_ENTRY_BYTES {
                return;
            }
            for entry in &tt.entries {
                if get_u16(entry, KIND_OFFSET) != KIND_DELETED_INODES {
                    continue;
                }
                let (readable, present, queue) = self.read_deletion_queue(entry);
                let at_most_one = !present || queue.iter().all(|inode| *inode == queue[0]) || self.fault == "queue-bound-off-24";
                if readable && !at_most_one {
                    self.violate("I-9.24");
                }
            }
        }

        /// 扫描内核里两遍的次序照 scan_stage：删除态扫描（这里只 I-9.24）在前，墓碑与谱系那一遍在后；版本清单两遍同一份。
        pub fn run(&mut self, versions: &[(u64, Vec<u8>)]) {
            for (_, pointer) in versions {
                self.judge_deletion_queue_bound_of_a_version(pointer);
            }
            self.births.clear();
            for (txg, pointer) in versions {
                self.judge_tombstones_and_the_lineage_of_a_version(pointer, *txg);
            }
        }
    }
}

// ───────────────────────── CPU 与 GPU 译本逐状态比 ─────────────────────────

/// 比的那几位：墓碑与谱系那一遍置的五位与删除态扫描里 I-9.24 那一位；I-9.10 只比墓碑那一个等号（见 `compare`）。
const TARGETS: [&str; 6] = ["I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.24", "I-9.29"];

/// 一张镜像上 CPU（池级 checker 旧入口）与 GPU 译本各判一次。交回 (对不上的那几条说明, GPU 译本判不了的站点, CPU 违例)。
/// I-9.29：CPU 滤不掉被抛弃的根（最新根实例表读不出）时 GPU 那一侧照比，是第 24 批的活，这里不比。
/// I-9.10：GPU 译本只译了墓碑那一个等号——它红而 CPU 不红算对不上；CPU 红、说明里点名墓碑而译本不红也算。
pub fn compare(image: &MemoryPool) -> (Vec<String>, Option<u32>, Vec<&'static str>, CpuVerdicts) {
    let cpu = CpuVerdicts::of(image);
    let mut model = gpu_model::Model::new(image);
    model.run(&cpu.versions);
    let mut mismatches = Vec::new();
    if model.undecidable.is_none() {
        for target in TARGETS {
            if target == "I-9.29" && cpu.filtered == Some(false) {
                continue;
            }
            let cpu_red = cpu.kind(target) == "不成立";
            let gpu_red = model.violated.contains(target);
            if cpu_red != gpu_red {
                mismatches.push(format!("{target} cpu={} gpu={}", cpu.kind(target), if gpu_red { "红" } else { "不红" }));
            }
        }
        let cpu_red = cpu.kind("I-9.10") == "不成立";
        let gpu_red = model.violated.contains("I-9.10");
        if (gpu_red && !cpu_red) || (cpu_red && !gpu_red && cpu.detail("I-9.10").contains("墓碑")) {
            mismatches.push(format!("I-9.10 cpu={} gpu={}", cpu.kind("I-9.10"), if gpu_red { "红" } else { "不红" }));
        }
    }
    let violated = cpu.violated();
    (mismatches, model.undecidable, violated, cpu)
}

const NODE: usize = 16384;
const DATA: usize = 32768;

fn read_unit_at(image: &MemoryPool, pointer: &singlefs_checker::image::PointerView, length: usize) -> Option<Vec<u8>> {
    pointer.locations.iter().find_map(|location| {
        image
            .read(location.device, location.slot * 16384, length)
            .filter(|bytes| singlefs_checker::crc32_castagnoli_table(bytes) == location.checksum)
    })
}

/// 改坏的对象：一个单元（两盘同槽的槽号、宽度）与它是什么。
#[derive(Clone, Debug)]
enum Target {
    /// 码 2 节点：所在树的种类、层级。
    Node { slot: u64, kind: u16, level: u8 },
    /// 码 3 容器：打包记录类型。
    Container { slot: u64, record_type: u16 },
}

/// 一版树表里墓碑树、谱系树、inode 树、deleted_inodes 的每个单元（读得出的才收）。
fn targets_of_a_version(image: &MemoryPool, tree_table_pointer: &[u8], lineage_txgs: &mut Vec<u64>) -> Vec<Target> {
    use singlefs_checker::image::parse_node_pointer;
    let mut targets = Vec::new();
    let pointer = parse_node_pointer(tree_table_pointer);
    if pointer.all_zero {
        return targets;
    }
    let Some(tt) = read_unit_at(image, &pointer, NODE).and_then(|unit| singlefs_checker::index_node_view(&unit).ok()) else {
        return targets;
    };
    for entry in &tt.entries {
        if entry.len() < 200 {
            continue;
        }
        let kind = get_u16(entry, 10);
        if ![2u16, 9, 10, 11].contains(&kind) {
            continue;
        }
        let mut stack = vec![parse_node_pointer(&entry[14..100])];
        while let Some(node_pointer) = stack.pop() {
            if node_pointer.all_zero || node_pointer.locations[0].slot != node_pointer.locations[1].slot {
                continue;
            }
            let Some(node) = read_unit_at(image, &node_pointer, NODE).and_then(|unit| singlefs_checker::index_node_view(&unit).ok()) else {
                continue;
            };
            targets.push(Target::Node { slot: node_pointer.locations[0].slot, kind, level: node.level });
            if kind == 9 || node.level == 0 || node.entry_width < 120 {
                continue;
            }
            for child_entry in &node.entries {
                let child = parse_node_pointer(&child_entry[34..120]);
                let entry_type = get_u16(child_entry, 16);
                if entry_type == 0 {
                    stack.push(child);
                } else if !child.all_zero && child.locations[0].slot == child.locations[1].slot {
                    if let Some(container) = read_unit_at(image, &child, DATA).and_then(|unit| singlefs_checker::packed_unit_view(&unit).ok()) {
                        if container.record_type == 5 {
                            for record in &container.records {
                                lineage_txgs.extend([get_u64(record, 0), get_u64(record, 8)]);
                            }
                        }
                        targets.push(Target::Container { slot: child.locations[0].slot, record_type: container.record_type });
                    }
                }
            }
        }
    }
    targets
}

/// 在 `target` 上随机改一处、重封、沿引用链补校验和（`common_deletion_in_progress::change_the_unit_and_propagate_its_checksum`）。
/// `numbers` 是这张镜像上见过的 inode 号、`txgs` 是见过的 txg，取值从里面抽（让改动落在判据的边界上）。交回改了什么。
fn mutate(image: &mut MemoryPool, target: &Target, rng: &mut SplitMix, numbers: &[u64], txgs: &[u64], version_txg: u64) -> String {
    let pick = |rng: &mut SplitMix, values: &[u64], fallback: u64| -> u64 {
        if values.is_empty() { fallback } else { values[rng.below(values.len())] }
    };
    let choice = rng.next();
    let mut note = String::new();
    match target {
        Target::Container { slot, record_type } => {
            let record_type = *record_type;
            let width = match record_type { 1 => 40, 5 => 16, _ => 140 };
            let number = pick(rng, numbers, 1);
            let txg = pick(rng, txgs, 1);
            let delta = if rng.below(2) == 0 { 1u64 } else { u64::MAX };
            common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, SlotNumber(*slot), DATA, |bytes| {
                let count = usize::from(get_u16(bytes, 69));
                let record = |index: usize| 136 + index * width;
                let index = if count == 0 { 0 } else { (choice as usize >> 8) % count };
                let op = choice % 10;
                note = format!("码3 类型{record_type} 槽{slot} 记录{index}/{count} op{op}");
                match (record_type, op) {
                    (_, 6) if count > 0 && (136 + (count + 1) * width) <= DATA => {
                        let copy = bytes[record(index)..record(index) + width].to_vec();
                        bytes[record(count)..record(count) + width].copy_from_slice(&copy);
                        set_u16(bytes, 69, (count + 1) as u16);
                        set_u16(bytes, 8, ((count + 1) * width) as u16);
                        note.push_str(" 追加一份同样的记录");
                    }
                    (_, 7) if count > 0 => {
                        let last = record(count - 1);
                        for byte in &mut bytes[last..last + width] {
                            *byte = 0;
                        }
                        set_u16(bytes, 69, (count - 1) as u16);
                        set_u16(bytes, 8, ((count - 1) * width) as u16);
                        note.push_str(" 去掉最后一条");
                    }
                    (_, 8) if count > 1 => {
                        let other = (index + 1) % count;
                        let (a, b) = (bytes[record(index)..record(index) + width].to_vec(), bytes[record(other)..record(other) + width].to_vec());
                        bytes[record(index)..record(index) + width].copy_from_slice(&b);
                        bytes[record(other)..record(other) + width].copy_from_slice(&a);
                        note.push_str(&format!(" 与第 {other} 条对调"));
                    }
                    (_, 9) => {
                        let value = get_u64(bytes, 53).wrapping_add(delta);
                        set_u64(bytes, 53, value);
                        note.push_str(&format!(" 容器号改成 {value}"));
                    }
                    (_, _) if count == 0 => note.push_str(" 空容器不改"),
                    (1, op) => {
                        let at = record(index);
                        let (field, value) = match op {
                            0 => (0, get_u64(bytes, at).wrapping_add(delta)),
                            1 => (0, number),
                            2 => (8, get_u64(bytes, at + 8).wrapping_add(delta)),
                            3 => (32, txg),
                            4 => (32, if delta == 1 { version_txg.wrapping_add(1) } else { txg.wrapping_add(delta) }),
                            _ => if rng.below(2) == 0 { (16, 1) } else { (24, u64::MAX - 1) },
                        };
                        set_u64(bytes, at + field, value);
                        note.push_str(&format!(" 墓碑偏移 {field} 改成 {value}"));
                    }
                    (5, op) => {
                        let at = record(index);
                        let (field, value) = match op {
                            0 | 1 => (0, get_u64(bytes, at).wrapping_add(delta)),
                            2 | 3 => (8, get_u64(bytes, at + 8).wrapping_add(delta)),
                            4 => (8, get_u64(bytes, at)),
                            _ => (0, txg),
                        };
                        set_u64(bytes, at + field, value);
                        note.push_str(&format!(" 谱系偏移 {field} 改成 {value}"));
                    }
                    (_, op) => {
                        let at = record(index);
                        let (field, value) = match op {
                            0 | 1 => (8, get_u64(bytes, at + 8).wrapping_add(delta)),
                            2 | 3 => (36, u64::from(get_u32(bytes, at + 36) == 0)),
                            4 => (0, number),
                            _ => (8, txg),
                        };
                        if field == 36 {
                            bytes[at + 36..at + 40].copy_from_slice(&(value as u32).to_le_bytes());
                        } else {
                            set_u64(bytes, at + field, value);
                        }
                        note.push_str(&format!(" inode 记录偏移 {field} 改成 {value}"));
                    }
                }
            });
        }
        Target::Node { slot, kind, level } => {
            let number = pick(rng, numbers, 2);
            common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, SlotNumber(*slot), NODE, |bytes| {
                let key_width = usize::from(bytes[51]);
                let header_end = 86 + 2 * key_width;
                let count = usize::from(get_u16(bytes, header_end - 4));
                let width = usize::from(get_u16(bytes, header_end - 2));
                let start = header_end + 29;
                let index = if count == 0 { 0 } else { (choice as usize >> 8) % count };
                note = format!("码2 种类{kind} 层级{level} 槽{slot} 条目{index}/{count}");
                if *kind == 9 {
                    if count > 0 && choice % 2 == 0 {
                        set_u64(bytes, start + index * width, number);
                        note.push_str(&format!(" 队列号改成 {number}"));
                    } else if start + (count + 1) * width <= NODE && width >= 8 {
                        let mut entry = vec![0u8; width];
                        set_u64(&mut entry, 0, number);
                        bytes[start + count * width..start + (count + 1) * width].copy_from_slice(&entry);
                        set_u16(bytes, header_end - 4, (count + 1) as u16);
                        set_u16(bytes, 8, ((count + 1) * width) as u16);
                        note.push_str(&format!(" 队列追加号 {number}"));
                    }
                } else if count > 0 && width >= 120 {
                    let at = start + index * width;
                    match choice % 4 {
                        0 => {
                            let value = [0u16, 1, 2, 5, 7][rng.below(5)];
                            set_u16(bytes, at + 16, value);
                            note.push_str(&format!(" 类型段改成 {value}"));
                        }
                        1 => {
                            let value = get_u64(bytes, at + 18).wrapping_add(1);
                            set_u64(bytes, at + 18, value);
                            note.push_str(&format!(" 身份容器号改成 {value}"));
                        }
                        2 if count > 1 => {
                            let other = (index + 1) % count;
                            let (a, b) = (bytes[at..at + width].to_vec(), bytes[start + other * width..start + (other + 1) * width].to_vec());
                            bytes[at..at + width].copy_from_slice(&b);
                            bytes[start + other * width..start + (other + 1) * width].copy_from_slice(&a);
                            note.push_str(&format!(" 与第 {other} 条对调"));
                        }
                        _ => {
                            let value = get_u64(bytes, at + 8).wrapping_add(1);
                            set_u64(bytes, at + 8, value);
                            note.push_str(&format!(" 身份出生树改成 {value}"));
                        }
                    }
                }
            });
        }
    }
    note
}

// ───────────────────────── 随机历史（用户动作放开扫）＋每一步随机改坏 ─────────────────────────

/// 起点：新池新建文件 → 第一个文件顺序写成 3 个单元 → 建 2、3 号 → 建 4 号。`records > 0` 时装上删除开关（容器一片 `records` 条、节点 2 条）。
pub fn starting_pool(records: u64) -> (SparsePool, Vec<(InstanceGeneration, u64)>) {
    let mut pool = SparsePool::build();
    if records > 0 {
        let switches = switches_of(records, 2);
        pool.switches = Some(switches);
        pool.allocator.set_deletion_switches(switches);
    }
    let mut versions = vec![pool.here()];
    pool.grow_the_first_file(3).expect("顺序写成 3 个单元");
    versions.push(pool.here());
    pool.create(2).expect("建 2、3 号");
    versions.push(pool.here());
    pool.create(1).expect("建 4 号");
    versions.push(pool.here());
    (pool, versions)
}

#[derive(Default)]
pub struct Campaign {
    pub clean_checks: u64,
    pub mutated_checks: u64,
    pub mismatches: u64,
    pub undecidable: BTreeMap<u32, u64>,
    pub unfiltered_states: u64,
    pub acts: BTreeMap<String, (u64, u64)>,
    pub clean_reds: BTreeMap<String, u64>,
    pub mutated_red_targets: BTreeMap<&'static str, u64>,
    pub i929_kinds_clean: BTreeMap<&'static str, u64>,
}

/// 一段随机历史；每做成一步，干净镜像比一次，再在这一版的版本清单里随机挑一版、挑一个单元改坏 `mutations` 次，各比一次。
pub fn run_history(base: &SparsePool, base_versions: &[(InstanceGeneration, u64)], seed: u64, steps: usize, mutations: u64, campaign: &mut Campaign) -> String {
    let mut rng = SplitMix(seed);
    let mut pool = base.clone();
    let mut versions = base_versions.to_vec();
    let mut trace = Vec::new();
    for step_index in 0..steps {
        let mut attempt = pool.clone();
        let choice = rng.below(100);
        let (label, outcome) = if choice < 20 {
            let count = 1 + rng.below(3) as u64;
            (format!("create{count}"), attempt.create(count))
        } else if choice < 40 {
            let live: Vec<u64> = attempt.records().iter().filter(|(_, nlink, _)| *nlink >= 1).map(|(inode, _, _)| *inode).collect();
            if live.is_empty() {
                ("intent-none".to_string(), Err("没有活的号".to_string()))
            } else {
                let inode = live[rng.below(live.len())];
                (format!("intent{inode}"), attempt.intent(inode))
            }
        } else if choice < 50 {
            ("step".to_string(), attempt.step())
        } else if choice < 60 {
            let outcome = if attempt.queue().is_empty() { Err("队列空".to_string()) } else { attempt.drain() };
            ("drain".to_string(), outcome)
        } else if choice < 80 {
            let current = attempt.here();
            let choices: Vec<(InstanceGeneration, u64)> = versions.iter().copied().filter(|v| *v != current).collect();
            let back = 1 + rng.below(choices.len().min(20));
            let target = choices[choices.len() - back];
            (format!("rollback(i{},t{})", target.0 .0, target.1), attempt.rollback(target))
        } else if choice < 88 {
            ("remount".to_string(), attempt.remount())
        } else {
            ("abandon".to_string(), attempt.abandon_the_newest())
        };
        let kind: String = label.chars().take_while(|c| c.is_ascii_lowercase() || *c == '-').collect();
        let entry = campaign.acts.entry(kind).or_insert((0, 0));
        let Ok(()) = outcome else {
            entry.1 += 1;
            trace.push(format!("{label}:skip"));
            continue;
        };
        entry.0 += 1;
        pool = attempt;
        versions.push(pool.here());
        trace.push(format!("{label}->t{}", pool.output.root.checkpoint_txg.0));
        let image = pool.image();
        let (mismatches, undecidable, violated, cpu) = compare(&image);
        campaign.clean_checks += 1;
        *campaign.i929_kinds_clean.entry(cpu.kind("I-9.29")).or_insert(0) += 1;
        if !violated.is_empty() {
            *campaign.clean_reds.entry(format!("{violated:?}")).or_insert(0) += 1;
        }
        if let Some(site) = undecidable {
            *campaign.undecidable.entry(site).or_insert(0) += 1;
        }
        if cpu.filtered == Some(false) {
            campaign.unfiltered_states += 1;
        }
        for mismatch in &mismatches {
            campaign.mismatches += 1;
            println!("RESULT MISMATCH seed={seed} step={step_index} clean {mismatch}");
        }
        let numbers: Vec<u64> = pool.records().iter().map(|(inode, _, _)| *inode).chain(pool.queue()).chain([1, 2, 3, 4, 5, 6, 7, 8]).collect();
        let txgs: Vec<u64> = cpu.versions.iter().map(|(txg, _)| *txg).collect();
        for mutation_index in 0..mutations {
            if cpu.versions.is_empty() {
                break;
            }
            let (version_txg, pointer) = &cpu.versions[rng.below(cpu.versions.len())];
            let mut lineage_txgs = Vec::new();
            let targets = targets_of_a_version(&image, pointer, &mut lineage_txgs);
            let txgs: Vec<u64> = txgs.iter().copied().chain(lineage_txgs.iter().flat_map(|txg| [txg.saturating_sub(1), *txg, txg + 1])).collect();
            if targets.is_empty() {
                continue;
            }
            let target = targets[rng.below(targets.len())].clone();
            let mut mutated = image.clone();
            let note = mutate(&mut mutated, &target, &mut rng, &numbers, &txgs, *version_txg);
            let (mismatches, undecidable, violated, mutated_cpu) = compare(&mutated);
            campaign.mutated_checks += 1;
            if let Some(site) = undecidable {
                *campaign.undecidable.entry(site).or_insert(0) += 1;
            }
            if mutated_cpu.filtered == Some(false) {
                campaign.unfiltered_states += 1;
            }
            for target_name in TARGETS.iter().chain(["I-9.10"].iter()) {
                if violated.contains(target_name) {
                    *campaign.mutated_red_targets.entry(target_name).or_insert(0) += 1;
                }
            }
            for mismatch in &mismatches {
                campaign.mismatches += 1;
                println!("RESULT MISMATCH seed={seed} step={step_index} mutation={mutation_index} ({note}) {mismatch} cpu_violated={violated:?}");
            }
        }
    }
    format!("RESULT history seed={seed} trace={}", trace.join(" "))
}

/// 随机历史批：`OPUS_SEED_FROM`.. 共 `OPUS_HISTORIES` 段，每段 `OPUS_STEPS` 步、每步改坏 `OPUS_MUTATIONS` 次。`OPUS_KNOB`：0 产品容量，k ≥ 1 删除开关（容器一片 k 条、节点 2 条）。
/// `OPUS_PREFIX=reissue`：固定前缀「建一个号 → 那次发布被崩溃恢复抛弃 → 新线重发同号」只建一次。`OPUS_GPU_MODEL_FAULT` 给了时是译本的自证跑。
#[test]
fn opus_b23_gpu_model_against_the_cpu_over_random_histories() {
    let histories = environment_number("OPUS_HISTORIES", 2);
    let steps = environment_number("OPUS_STEPS", 6) as usize;
    let seed_from = environment_number("OPUS_SEED_FROM", 0);
    let mutations = environment_number("OPUS_MUTATIONS", 4);
    let knob = environment_number("OPUS_KNOB", 0);
    let fault = std::env::var("OPUS_GPU_MODEL_FAULT").unwrap_or_default();
    let (mut base, mut base_versions) = starting_pool(knob);
    if std::env::var("OPUS_PREFIX").as_deref() == Ok("reissue") {
        base.create(1).expect("前缀：建一个号");
        base_versions.push(base.here());
        base.abandon_the_newest().expect("前缀：建号那次被崩溃恢复抛弃");
        base_versions.push(base.here());
        base.create(1).expect("前缀：新线重发同号");
        base_versions.push(base.here());
        println!("RESULT prefix=reissue records={:?}", base.records());
    }
    // OPUS_PREFIX=rich：墓碑、谱系、队列都先有——建 5、6、7 → 删完 5 → 删完 6 → 回退到删 6 之前 → 再发 6 的意图、删一批（「删 → 回退 → 再删」）。
    if std::env::var("OPUS_PREFIX").as_deref() == Ok("rich") {
        base.create(3).expect("前缀：建 5、6、7");
        base_versions.push(base.here());
        base.intent(5).expect("前缀：5 的意图");
        base.drain().expect("前缀：删完 5");
        base_versions.push(base.here());
        let before_six = base.here();
        base.intent(6).expect("前缀：6 的意图");
        base.drain().expect("前缀：删完 6");
        base_versions.push(base.here());
        base.rollback(before_six).expect("前缀：回退到删 6 之前");
        base_versions.push(base.here());
        base.intent(6).expect("前缀：再发 6 的意图");
        base_versions.push(base.here());
        let _ = base.step();
        base_versions.push(base.here());
        let image = base.image();
        let (mismatches, _, violated, cpu) = compare(&image);
        println!("RESULT prefix=rich records={:?} queue={:?} violated={violated:?} mismatches={mismatches:?} I-9.18={} I-9.20={} I-9.29={}", base.records(), base.queue(), cpu.kind("I-9.18"), cpu.kind("I-9.20"), cpu.kind("I-9.29"));
    }
    let mut campaign = Campaign::default();
    let started = std::time::Instant::now();
    for seed in seed_from..seed_from + histories {
        println!("{}", run_history(&base, &base_versions, seed, steps, mutations, &mut campaign));
    }
    eprintln!("TIMING histories={histories} steps={steps} mutations={mutations} checks={} seconds={:.1}", campaign.clean_checks + campaign.mutated_checks, started.elapsed().as_secs_f64());
    let tag = format!("knob={knob} fault={fault:?} seeds={seed_from}..{} steps={steps} mutations={mutations}", seed_from + histories);
    println!("RESULT campaign {tag} clean_checks={} mutated_checks={} mismatches={} unfiltered_states={}", campaign.clean_checks, campaign.mutated_checks, campaign.mismatches, campaign.unfiltered_states);
    for (site, count) in &campaign.undecidable {
        println!("RESULT campaign {tag} gpu_model_undecidable_site={site} states={count}");
    }
    for (act, (done, skipped)) in &campaign.acts {
        println!("RESULT campaign {tag} act={act} done={done} skipped={skipped}");
    }
    for (signature, count) in &campaign.clean_reds {
        println!("RESULT campaign {tag} clean_red {signature} {count}");
    }
    for (kind, count) in &campaign.i929_kinds_clean {
        println!("RESULT campaign {tag} clean I-9.29={kind} {count}");
    }
    for (target, count) in &campaign.mutated_red_targets {
        println!("RESULT campaign {tag} mutated_cpu_red {target} {count}");
    }
}

// ───────────────────────── N5：I-9.29 实例表读得出 / 读不出 ─────────────────────────

/// 「建一个号 → 那次发布被崩溃恢复抛弃 → 新线重发同号」的干净镜像：被抛弃的那条根读得出，同号不同出生代，只靠最新根的实例表滤掉。
fn image_with_an_abandoned_root_naming_another_object_birth() -> (SparsePool, MemoryPool) {
    let (mut pool, _) = starting_pool(0);
    pool.create(1).expect("建 5 号");
    pool.abandon_the_newest().expect("建号那次被崩溃恢复抛弃");
    let (instance, txg) = pool.here();
    println!("RESULT N5-setup 崩溃恢复之后现行那一版 实例{} txg{} records={:?}", instance.0, txg, pool.records());
    pool.create(1).expect("新线重发 5 号");
    let image = pool.image();
    (pool, image)
}

fn instance_table_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
    let root = common_pointer_head_corruption::newest_root(image);
    assert_eq!(root.instance_table.locations[0].slot, root.instance_table.locations[1].slot, "两盘同槽");
    root.instance_table.locations[0].slot
}

fn report_n5(name: &str, image: &MemoryPool, clean_version_count: usize) {
    let cpu = CpuVerdicts::of(image);
    let detail: String = cpu.detail("I-9.29").chars().take(24).collect();
    println!(
        "RESULT {name} violated={:?} I-9.29={}（{detail}） filtered_flag={:?} versions={} (干净镜像 {clean_version_count}) I-7.9={} I-3.8={}",
        cpu.violated(),
        cpu.kind("I-9.29"),
        cpu.filtered,
        cpu.versions.len(),
        cpu.kind("I-7.9"),
        cpu.kind("I-3.8"),
    );
}

/// 实例表单元：记录从 136 起、每条 88；记录数在 69、声明长度在 8（码 3 打包单元头）。
const INSTANCE_TABLE_RECORD: usize = 88;

#[test]
fn n5_the_one_birth_invariant_against_what_the_newest_instance_table_reads_back() {
    let (pool, clean) = image_with_an_abandoned_root_naming_another_object_birth();
    println!("RESULT N5-setup records={:?}", pool.records());
    let clean_cpu = CpuVerdicts::of(&clean);
    let clean_versions = clean_cpu.versions.len();
    report_n5("N5-0 干净镜像", &clean, clean_versions);
    let slot = instance_table_slot_of_the_newest_root(&clean);
    let table = common_pointer_head_corruption::unit_bytes_at(&clean, slot, DATA);
    let count = usize::from(get_u16(&table, 69));
    let kinds: Vec<u8> = (0..count).map(|index| table[136 + index * INSTANCE_TABLE_RECORD]).collect();
    println!("RESULT N5-setup 最新根实例表 槽{} 记录数{count} 各条 kind {kinds:?}", slot.0);

    // a：链指针记录之前插一条 kind 2 的记录（重封、补引用链）：主走读照样把 kind 0 那几行都收进候选集的滤法，只读不判那个读法交回 None。
    let mut inserted = clean.clone();
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(&mut inserted, slot, DATA, |bytes| {
        let count = usize::from(get_u16(bytes, 69));
        let chain = bytes[136 + (count - 1) * INSTANCE_TABLE_RECORD..136 + count * INSTANCE_TABLE_RECORD].to_vec();
        let mut unknown = vec![0u8; INSTANCE_TABLE_RECORD];
        unknown[0] = 2;
        bytes[136 + (count - 1) * INSTANCE_TABLE_RECORD..136 + count * INSTANCE_TABLE_RECORD].copy_from_slice(&unknown);
        bytes[136 + count * INSTANCE_TABLE_RECORD..136 + (count + 1) * INSTANCE_TABLE_RECORD].copy_from_slice(&chain);
        set_u16(bytes, 69, (count + 1) as u16);
        set_u16(bytes, 8, ((count + 1) * INSTANCE_TABLE_RECORD) as u16);
    });
    report_n5("N5-a 链指针记录之前多一条 kind 2", &inserted, clean_versions);

    // b：链指针记录「有无下一片」写 2（重封、补引用链）：主走读先把这一片的行收下再停，I-3.8 红。
    let mut malformed_chain = clean.clone();
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(&mut malformed_chain, slot, DATA, |bytes| {
        let count = usize::from(get_u16(bytes, 69));
        bytes[136 + (count - 1) * INSTANCE_TABLE_RECORD + 1] = 2;
    });
    report_n5("N5-b 链指针记录有无下一片写 2", &malformed_chain, clean_versions);

    // c：载荷里补齐区最后一字节翻一下、不重封载荷 CRC，整单元校验和照新内容补进引用链：主走读头判得过、载荷 CRC 只红 I-2.3、行照用。
    let mut payload_crc = clean.clone();
    let mut bytes = table.clone();
    bytes[DATA - 1] ^= 0xff;
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(&mut payload_crc, slot, &bytes);
    report_n5("N5-c 载荷 CRC 对不上、整单元校验和补过", &payload_crc, clean_versions);

    // d：两份各翻一个载荷字节、不补引用链（批里自己那条用例的造法，对照；每块盘读它自己那一份再翻）。
    let mut unreadable = clean.clone();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut copy = unreadable.devices.get(&device).expect("盘").read(slot.to_device_offset(), DATA);
        copy[DATA - 100] ^= 0xff;
        unreadable.devices.get_mut(&device).expect("盘").write(slot.to_device_offset(), &copy);
    }
    report_n5("N5-d 两份都读不出（对照）", &unreadable, clean_versions);

    // e：最新那一版的 inode 叶里 1 号的对象出生代加一（同一个实例里更早的几版仍是原值），实例表照 d 读不出：现行实例里两版就对不上。
    let mut conflict = clean.clone();
    common_deletion_in_progress::change_the_inode_record(&mut conflict, 1, |record| {
        let birth = get_u64(record, 8) + 1;
        set_u64(record, 8, birth);
    });
    report_n5("N5-e0 最新那一版 1 号出生代加一、实例表读得出", &conflict, clean_versions);
    let conflict_cpu = CpuVerdicts::of(&conflict);
    println!("RESULT N5-e0 I-9.29 说明 {}", conflict_cpu.detail("I-9.29"));
    let conflict_slot = instance_table_slot_of_the_newest_root(&conflict);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut copy = conflict.devices.get(&device).expect("盘").read(conflict_slot.to_device_offset(), DATA);
        copy[DATA - 100] ^= 0xff;
        conflict.devices.get_mut(&device).expect("盘").write(conflict_slot.to_device_offset(), &copy);
    }
    report_n5("N5-e1 同上、实例表两份都读不出", &conflict, clean_versions);
    let root = common_pointer_head_corruption::newest_root(&clean);
    println!("RESULT N5-setup 最新根 实例{} txg{}", root.instance.0, root.checkpoint_txg.0);
}

// ───────────────────────── N6 第二次抽样：产品容量的墓碑容器（I-9.21），连 GPU 译本 ─────────────────────────

/// 删到墓碑树里一共 `total` 条墓碑：活的号（1 号除外）不够就一次建至多 `chunk` 个，再一个个发意图、走空。交回删掉的号的次序。
fn delete_until(pool: &mut SparsePool, deleted: &mut Vec<u64>, total: usize, chunk: u64) {
    while deleted.len() < total {
        let live: Vec<u64> = pool.records().iter().filter(|(inode, nlink, _)| *inode != 1 && *nlink >= 1).map(|(inode, _, _)| *inode).collect();
        let Some(inode) = live.first().copied() else {
            pool.create(chunk.min((total - deleted.len()) as u64).max(1)).expect("建");
            continue;
        };
        pool.intent(inode).expect("意图");
        pool.drain().expect("走空");
        deleted.push(inode);
    }
}

fn report_n6(name: &str, image: &MemoryPool) {
    let (mismatches, undecidable, violated, cpu) = compare(image);
    let mut lineage_txgs = Vec::new();
    let containers: Vec<usize> = cpu
        .versions
        .iter()
        .max_by_key(|(txg, _)| *txg)
        .map(|(_, pointer)| targets_of_a_version(image, pointer, &mut lineage_txgs))
        .unwrap_or_default()
        .iter()
        .filter_map(|target| match target {
            Target::Container { slot, record_type: 1 } => {
                let unit = common_pointer_head_corruption::unit_bytes_at(image, SlotNumber(*slot), DATA);
                Some(usize::from(get_u16(&unit, 69)))
            }
            _ => None,
        })
        .collect();
    println!(
        "RESULT {name} 最后一版墓碑容器记录数={containers:?} violated={violated:?} I-9.20={} I-9.21={} I-9.29={} gpu_undecidable={undecidable:?} mismatches={mismatches:?}",
        cpu.kind("I-9.20"),
        cpu.kind("I-9.21"),
        cpu.kind("I-9.29")
    );
}

/// 改 txg 最大的那一版墓碑树第 `index` 片容器（`targets_of_a_version` 收的次序里码 3 类型 1 那几个，根层级 1 时就是叶序），`change` 拿到整片单元。
fn change_the_tombstone_container(image: &mut MemoryPool, index: usize, change: impl FnOnce(&mut Vec<u8>)) {
    let cpu = CpuVerdicts::of(image);
    let (_, pointer) = cpu.versions.iter().max_by_key(|(txg, _)| *txg).expect("有版本").clone();
    let mut lineage_txgs = Vec::new();
    let slots: Vec<u64> = targets_of_a_version(image, &pointer, &mut lineage_txgs)
        .iter()
        .filter_map(|target| match target {
            Target::Container { slot, record_type: 1 } => Some(*slot),
            _ => None,
        })
        .collect();
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, SlotNumber(slots[index]), DATA, change);
}

#[test]
fn n6_product_tombstone_containers_second_sample() {
    let started = std::time::Instant::now();
    let mut pool = SparsePool::build();
    let mut deleted = Vec::new();
    delete_until(&mut pool, &mut deleted, 815, 100);
    report_n6("N6-s0 815 条（一片满）", &pool.image());
    let before_the_second_container = pool.clone();
    // s1：开第二片的那次意图被崩溃恢复抛弃（落到前一条根），新线重发号、删另一个号开第二片。
    let mut abandoned = before_the_second_container.clone();
    let live: Vec<u64> = abandoned.records().iter().filter(|(inode, nlink, _)| *inode != 1 && *nlink >= 1).map(|(inode, _, _)| *inode).collect();
    let first_number = match live.first() {
        Some(inode) => *inode,
        None => {
            abandoned.create(2).expect("建两个");
            abandoned.records().iter().filter(|(inode, nlink, _)| *inode != 1 && *nlink >= 1).map(|(inode, _, _)| *inode).next().expect("有活的号")
        }
    };
    abandoned.intent(first_number).expect("第 816 条墓碑的意图");
    match abandoned.abandon_the_newest() {
        Ok(()) => {
            abandoned.create(2).expect("新线建两个");
            let live: Vec<u64> = abandoned.records().iter().filter(|(inode, nlink, _)| *inode != 1 && *nlink >= 1).map(|(inode, _, _)| *inode).collect();
            let other = *live.last().expect("有活的号");
            abandoned.intent(other).expect("新线删另一个号");
            abandoned.drain().expect("走空");
            println!("RESULT N6-s1 被抛弃的意图删的是 {first_number}，新线删的是 {other}，新线的号 {:?}", abandoned.records().iter().map(|(inode, _, birth)| (*inode, *birth)).filter(|(inode, _)| *inode >= first_number.saturating_sub(2)).collect::<Vec<_>>());
            report_n6("N6-s1 开第二片那次被崩溃恢复抛弃、新线删另一个号", &abandoned.image());
        }
        Err(reason) => println!("RESULT N6-s1 造不出：{reason}"),
    }
    delete_until(&mut pool, &mut deleted, 1631, 100);
    report_n6("N6-s2 1631 条（815 + 815 + 1）", &pool.image());
    let three = pool.image();
    for (name, index, change) in [
        ("N6-s3a 第一片去掉最后一条", 0usize, 0u8),
        ("N6-s3b 第二片容器号改成第一片的", 1, 1),
        ("N6-s3c 最右那片容器号改成第二片的减一", 2, 2),
        ("N6-s3d 最右那片多一条（复制第一条）", 2, 3),
    ] {
        let mut image = three.clone();
        let first_container_number = {
            let cpu = CpuVerdicts::of(&image);
            let (_, pointer) = cpu.versions.iter().max_by_key(|(txg, _)| *txg).expect("有版本").clone();
            let mut lineage_txgs = Vec::new();
            targets_of_a_version(&image, &pointer, &mut lineage_txgs)
                .iter()
                .filter_map(|target| match target {
                    Target::Container { slot, record_type: 1 } => Some(get_u64(&common_pointer_head_corruption::unit_bytes_at(&image, SlotNumber(*slot), DATA), 53)),
                    _ => None,
                })
                .collect::<Vec<u64>>()
        };
        change_the_tombstone_container(&mut image, index, |bytes| {
            let count = usize::from(get_u16(bytes, 69));
            match change {
                0 => {
                    set_u16(bytes, 69, (count - 1) as u16);
                    set_u16(bytes, 8, ((count - 1) * 40) as u16);
                    for byte in &mut bytes[136 + (count - 1) * 40..136 + count * 40] {
                        *byte = 0;
                    }
                }
                1 => set_u64(bytes, 53, first_container_number[0]),
                2 => set_u64(bytes, 53, first_container_number[1].saturating_sub(1)),
                _ => {
                    let copy = bytes[136..176].to_vec();
                    bytes[136 + count * 40..136 + (count + 1) * 40].copy_from_slice(&copy);
                    set_u16(bytes, 69, (count + 1) as u16);
                    set_u16(bytes, 8, ((count + 1) * 40) as u16);
                }
            }
        });
        report_n6(name, &image);
    }
    eprintln!("TIMING n6 seconds={:.1}", started.elapsed().as_secs_f64());
}

/// 线索（不是 N6 那一格）：墓碑只追加、谱系上的墓碑每一版都进 I-9.29 跨版本那张表，删过的号一多，GPU 那张表（2048 个 (树, 号)）就满。
/// 合法历史上删到 `OPUS_CLIFF_TOTAL` 条，几个点上看 GPU 译本判不判得了（站点号照内核）与 CPU 的判定。
#[test]
fn n6_gpu_tables_on_a_long_legal_deletion_history() {
    let started = std::time::Instant::now();
    let total = environment_number("OPUS_CLIFF_TOTAL", 2100) as usize;
    let mut pool = SparsePool::build();
    let mut deleted = Vec::new();
    for checkpoint in [1000usize, 2000, 2040, 2044, 2048, total] {
        if checkpoint > total {
            continue;
        }
        delete_until(&mut pool, &mut deleted, checkpoint, 100);
        let live = pool.records().iter().filter(|(_, nlink, _)| *nlink >= 1).count();
        report_n6(&format!("N6-cliff 删过 {} 个号、活的 {live} 个", deleted.len()), &pool.image());
    }
    eprintln!("TIMING cliff seconds={:.1}", started.elapsed().as_secs_f64());
}

// ───────────────────────── N4 第二次抽样：I-9.17 / I-9.23 别的几形只红自己 ─────────────────────────

/// txg 为 `txg` 的那一版（版本清单里取）树表里种类 `kind` 那棵树的根：交回 (根的槽, 根的层级)。
fn root_of_the_tree_in_version(image: &MemoryPool, txg: u64, kind: u16) -> (u64, u8) {
    use singlefs_checker::image::parse_node_pointer;
    let cpu = CpuVerdicts::of(image);
    let (_, pointer) = cpu.versions.iter().find(|(version_txg, _)| *version_txg == txg).expect("版本清单里有这一版").clone();
    let tt = read_unit_at(image, &parse_node_pointer(&pointer), NODE).and_then(|unit| singlefs_checker::index_node_view(&unit).ok()).expect("树表读得出");
    let entry = tt.entries.iter().find(|entry| get_u16(entry, 10) == kind).expect("树表里有这种树");
    let root = parse_node_pointer(&entry[14..100]);
    let node = read_unit_at(image, &root, NODE).and_then(|unit| singlefs_checker::index_node_view(&unit).ok()).expect("根读得出");
    (root.locations[0].slot, node.level)
}

/// 码 2 节点 `slot` 的第 `index` 条条目改一处（`change` 拿到那 120 字节），重封、补引用链。
fn change_an_entry_of_the_node(image: &mut MemoryPool, slot: u64, index: usize, change: impl FnOnce(&mut [u8])) {
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, SlotNumber(slot), NODE, |bytes| {
        let start = 115 + 2 * usize::from(bytes[51]) + index * 120;
        change(&mut bytes[start..start + 120]);
    });
}

fn report_n4(name: &str, image: &MemoryPool) {
    let (mismatches, _, violated, cpu) = compare(image);
    println!("RESULT {name} violated={violated:?} I-9.23={} I-9.17={} scan_mismatches={mismatches:?}", cpu.kind("I-9.23"), cpu.kind("I-9.17"));
}

#[test]
fn n4_second_sample_of_container_tree_entry_identities() {
    let (base, _) = starting_pool(0);
    // 墓碑树层级 1：建 5 号、发它的意图（墓碑树根由这次发布写出）。
    let mut tombstone = base.clone();
    tombstone.create(1).expect("建 5 号");
    tombstone.intent(5).expect("5 号意图");
    let tombstone_txg = tombstone.here().1;
    let image = tombstone.image();
    let (slot, level) = root_of_the_tree_in_version(&image, tombstone_txg, 10);
    report_n4(&format!("N4-s0 墓碑树层级 {level} 干净"), &image);
    let mut s1 = image.clone();
    change_an_entry_of_the_node(&mut s1, slot, 0, |entry| { let value = get_u64(entry, 26) + 1; set_u64(entry, 26, value) });
    report_n4("N4-s1 墓碑树根类型段 1 条目身份出生代加一", &s1);
    let mut s2 = image.clone();
    change_an_entry_of_the_node(&mut s2, slot, 0, |entry| { let value = get_u64(entry, 8) + 1; set_u64(entry, 8, value) });
    report_n4("N4-s2 墓碑树根类型段 1 条目身份出生树加一", &s2);
    // 旧一版独有的墓碑树根：再删 4 号（墓碑树根换新），改 5 号意图那一版的根。
    let mut later = tombstone.clone();
    later.drain().expect("走空");
    later.intent(4).expect("4 号意图");
    let later_image = later.image();
    let (old_slot, _) = root_of_the_tree_in_version(&later_image, tombstone_txg, 10);
    let (new_slot, _) = root_of_the_tree_in_version(&later_image, later.here().1, 10);
    let mut s6 = later_image.clone();
    change_an_entry_of_the_node(&mut s6, old_slot, 0, |entry| { let value = get_u64(entry, 18) + 1; set_u64(entry, 18, value) });
    report_n4(&format!("N4-s6 只有旧一版（txg {tombstone_txg}，根槽 {old_slot}，最新根槽 {new_slot}）的墓碑树根条目身份容器号加一"), &s6);
    // 谱系树层级 1：建 5 号、删完、回退到意图之前。
    let mut lineage = base.clone();
    lineage.create(1).expect("建 5 号");
    let before = lineage.here();
    lineage.intent(5).expect("5 号意图");
    lineage.drain().expect("删完");
    lineage.rollback(before).expect("回退到意图之前");
    let lineage_image = lineage.image();
    let (lineage_slot, lineage_level) = root_of_the_tree_in_version(&lineage_image, lineage.here().1, 11);
    report_n4(&format!("N4-s0 谱系树层级 {lineage_level} 干净"), &lineage_image);
    let mut s3 = lineage_image.clone();
    change_an_entry_of_the_node(&mut s3, lineage_slot, 0, |entry| { let value = get_u64(entry, 18) + 1; set_u64(entry, 18, value) });
    report_n4("N4-s3 谱系树根类型段 5 条目身份容器号加一", &s3);
    // 墓碑树长到层级 2（一片 1 条、节点 2 条）：删 2、3、4 号。
    let (mut grown, _) = starting_pool(1);
    grown.create(3).expect("建 5、6、7");
    for inode in [5u64, 6, 7] {
        grown.intent(inode).expect("意图");
        grown.drain().expect("删完");
    }
    let grown_image = grown.image();
    let (grown_slot, grown_level) = root_of_the_tree_in_version(&grown_image, grown.here().1, 10);
    report_n4(&format!("N4-s0 墓碑树层级 {grown_level} 干净（旋钮镜像，I-9.21 照产品读法红是预期）"), &grown_image);
    let mut s4 = grown_image.clone();
    change_an_entry_of_the_node(&mut s4, grown_slot, 0, |entry| set_u64(entry, 26, 1));
    report_n4("N4-s4 层级 2 根类型段 0 条目身份出生代写 1", &s4);
    let inner_slot = {
        let root = common_pointer_head_corruption::unit_bytes_at(&grown_image, SlotNumber(grown_slot), NODE);
        let start = 115 + 2 * usize::from(root[51]);
        let child = singlefs_checker::image::parse_node_pointer(&root[start + 34..start + 120]);
        child.locations[0].slot
    };
    let mut s5 = grown_image.clone();
    change_an_entry_of_the_node(&mut s5, inner_slot, 0, |entry| { let value = get_u64(entry, 18) + 1; set_u64(entry, 18, value) });
    report_n4("N4-s5 层级 1 内部节点类型段 1 条目身份容器号加一", &s5);
}

// ───────────────────────── N7 第二次抽样：I-9.24 在旧一版、在乱序的队列上 ─────────────────────────

/// `deleted_inodes` 根（码 2、层级 0、key 宽 8）的号整个换成 `keys`（条目宽照原样，条目数与声明长度跟着改），重封、补引用链。
fn set_the_deleted_inodes_keys(image: &mut MemoryPool, slot: u64, keys: &[u64]) {
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, SlotNumber(slot), NODE, |bytes| {
        let key_width = usize::from(bytes[51]);
        let header_end = 86 + 2 * key_width;
        let width = usize::from(get_u16(bytes, header_end - 2)).max(8);
        let start = header_end + 29;
        for (index, key) in keys.iter().enumerate() {
            let at = start + index * width;
            for byte in &mut bytes[at..at + width] {
                *byte = 0;
            }
            set_u64(bytes, at, *key);
        }
        set_u16(bytes, header_end - 4, keys.len() as u16);
        set_u16(bytes, header_end - 2, width as u16);
        set_u16(bytes, 8, (keys.len() * width) as u16);
    });
}

fn report_n7(name: &str, image: &MemoryPool) {
    let (mismatches, _, violated, cpu) = compare(image);
    println!("RESULT {name} violated={violated:?} I-9.24={} I-9.5={} scan_mismatches={mismatches:?}", cpu.kind("I-9.24"), cpu.kind("I-9.5"));
}

#[test]
fn n7_second_sample_of_the_deletion_queue_bound() {
    let (mut pool, _) = starting_pool(0);
    pool.create(2).expect("建 5、6");
    pool.intent(5).expect("5 号意图");
    let queued_txg = pool.here().1;
    pool.drain().expect("走空");
    pool.create(1).expect("再建一个（让最新那一版的队列根换掉）");
    let image = pool.image();
    let (old_slot, old_level) = root_of_the_tree_in_version(&image, queued_txg, 9);
    report_n7(&format!("N7-s0 干净（意图那一版 txg {queued_txg} 的队列根槽 {old_slot} 层级 {old_level}）"), &image);
    let mut s1 = image.clone();
    set_the_deleted_inodes_keys(&mut s1, old_slot, &[5, 6]);
    report_n7("N7-s1 只有旧一版的队列是 {5, 6}", &s1);
    let mut s2 = image.clone();
    set_the_deleted_inodes_keys(&mut s2, old_slot, &[6, 5]);
    report_n7("N7-s2 旧一版队列乱序 [6, 5]", &s2);
    let mut s3 = image.clone();
    set_the_deleted_inodes_keys(&mut s3, old_slot, &[5, 5]);
    report_n7("N7-s3 旧一版队列同号两条 [5, 5]（上一轮判决第三节第 3 条那一问，这里只记）", &s3);
}
