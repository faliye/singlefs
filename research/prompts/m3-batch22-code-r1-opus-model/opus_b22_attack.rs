//! m3-batch22-code-r1 攻方原型（只在仓副本里编、跑；不入库代码）。
//! 盘用内存稀疏盘（`SparseBlockDevice`），起点池建一次、之后每段历史从内存里那一份拷。
//! 输出一律是 `RESULT ` 打头的行，`rerun.sh compare` 只比这些行（排序后）。
#![allow(dead_code, clippy::all, clippy::pedantic, unused_imports)]

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::collections::{BTreeMap, BTreeSet};

use common::{file_content, named_unit_bytes, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_tombstone_container_capacity_knob,
    TombstoneContainerCapacityKnob,
};
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
use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_LINEAGE, TREE_KIND_TOMBSTONE};
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
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

/// 两块内存稀疏盘上的池：设备、现行版本、分配器、写入口要装的容器树旋钮（`None` = 产品容量）。
pub struct SparsePool {
    pub devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
    pub capacities: Option<ContainerTreeCapacities>,
    /// 只供测试的删除开关（批大小、墓碑与谱系容器容量、节点条目数）：装上写入口、分配器（回退那次读它）与挂载（续做读它）。
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
            capacities: self.capacities,
            switches: self.switches,
        }
    }
}

impl SparsePool {
    /// mkfs → 取号 → 暖机 → 新池新建文件（与 `common::build_pool` 同一串，只是盘换成内存稀疏盘）。
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
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &parameters,
            &genesis,
        ));
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
        Self { devices, output, allocator, capacities: None, switches: None }
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
}

/// 用户动作（放开扫的那几步）。失败的交回 `Err`，调用方把池换回动作之前那一份。
impl SparsePool {
    fn with_writer<T>(
        &mut self,
        act: impl FnOnce(&mut PoolWriter<'_, SparseBlockDevice>, &mut PoolAllocator, &TransactionOutput, InstanceGeneration) -> Result<T, String>,
    ) -> Result<T, String> {
        let parameters = parameters();
        let previous = self.output.clone();
        let instance = self.instance();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        if let Some(capacities) = self.capacities {
            writer.set_container_tree_capacities(capacities);
        }
        if let Some(switches) = self.switches {
            writer.set_container_tree_capacities(switches.container_tree_capacities());
        }
        act(&mut writer, &mut self.allocator, &previous, instance)
    }

    pub fn create(&mut self, count: u64) -> Result<(), String> {
        self.drain()?;
        let out = self.with_writer(|writer, allocator, previous, instance| {
            publish_new_inodes(writer, allocator, previous, count, WT, instance).map_err(|e| format!("{e:?}"))
        })?;
        self.output = out;
        Ok(())
    }

    /// 顺序写第一个文件成 `units` 个数据单元（只在起点用）。
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

    /// unlink 意图：先把队列走空（会话纪律，D8 已定项 16 第 6 条的增补）。
    pub fn intent(&mut self, inode: u64) -> Result<(), String> {
        self.drain()?;
        let out = self.with_writer(|writer, allocator, previous, instance| {
            publish_unlink_intent(writer, allocator, previous, InodeNumber(inode), WT, instance)
                .map_err(|e| format!("{e:?}"))
        })?;
        self.output = out;
        Ok(())
    }

    /// 队列里那个号按批大小 1 删一批。
    pub fn step(&mut self) -> Result<(), String> {
        let Some(inode) = self.queue().first().copied() else { return Ok(()) };
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
            publish_unlink_steps_until_the_queue_is_empty(
                writer,
                allocator,
                &mut current,
                UnlinkBatchSize::OneExtentLowerLeaf,
                instance,
            )
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
            RollbackTarget {
                instance: target.0,
                checkpoint_txg: singlefs_core::address::CheckpointTxg(target.1),
            },
        )
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
    }

    /// 进程退出（什么都已持久）之后再可写挂载：新实例写行、暖机，删除续做。
    pub fn remount(&mut self) -> Result<(), String> {
        let parameters = parameters();
        let mounted = mount_with(&parameters, &mut self.devices, self.switches).map_err(|e| format!("{e:?}"))?;
        self.allocator = mounted.allocator.clone();
        self.output = mounted
            .current
            .version_of_trees()
            .ok_or_else(|| "落到的那一版树表 0 条".to_string())?
            .clone();
        Ok(())
    }
}

impl SparsePool {
    /// 崩溃恢复落到最新那条根之前那条根，最新那条根之后又读得出（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`
    /// 那一形，换成稀疏盘、藏的是最新那次发布的根槽与它那条记录点名的单元）：见证它的系统配置轮换没落盘（改写成轮换之前那一代），
    /// 挂载时根槽与点名单元一时读不出（清零），挂载交回之后原样写回。恢复没落到前一条根的交回 `Err`。
    pub fn abandon_the_newest(&mut self) -> Result<(), String> {
        let parameters = parameters();
        let newest = self.output.clone();
        let target = singlefs_core::root_ring::target_for_publish(
            newest.root.checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        let root_slot_device = parameters.region_devices[target.region as usize];
        let root_slot_offset = singlefs_core::root_ring::slot_offset(
            target,
            parameters.geometry.fixed_structure_slot_spacing,
        );
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
                singlefs_core::address::DeviceOffsetInBytes(
                    (generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,
                ),
                not_written.to_slot(),
            ));
        }
        for ((_, device), (offset, bytes)) in self.devices.iter_mut().zip(rotations) {
            device.write_at(offset, &bytes, WriteDurability::Plain).map_err(|e| format!("{e:?}"))?;
        }
        let mounted = mount_with(&parameters, &mut self.devices, self.switches);
        // 挂载在藏起来的地方写过（新实例把那几个槽拿去用了）：再原样写回会盖掉挂载写的东西，造出来的不是一段合法历史，这一步算做不成。
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
            return Err(format!(
                "恢复没落到前一条根（落到 txg {}）",
                mounted.output.effective_root.checkpoint_txg.0
            ));
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

/// 违例的不变量与细节。
pub fn violations(image: &MemoryPool, knob: Option<u64>) -> Vec<(&'static str, String)> {
    let verdicts = match knob {
        None => check_pool_image(image),
        Some(value) => check_pool_image_given_the_tombstone_container_capacity_knob(
            image,
            Some(TombstoneContainerCapacityKnob::new(value).expect("1..=815")),
        ),
    };
    verdicts
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            _ => None,
        })
        .collect()
}

pub fn verdict_kind(image: &MemoryPool, knob: Option<u64>, invariant: &str) -> String {
    let verdicts = match knob {
        None => check_pool_image(image),
        Some(value) => check_pool_image_given_the_tombstone_container_capacity_knob(
            image,
            Some(TombstoneContainerCapacityKnob::new(value).expect("1..=815")),
        ),
    };
    match verdicts.into_iter().find(|(name, _)| *name == invariant).expect("清单里有").1 {
        InvariantVerdict::Holds => "成立".to_string(),
        InvariantVerdict::Violated(detail) => format!("不成立（{detail}）"),
        InvariantVerdict::NotApplicable(reason) => format!("不适用（{reason}）"),
        InvariantVerdict::PartiallyJudged(detail) => format!("部分没判（{detail}）"),
    }
}

pub fn names(violated: &[(&'static str, String)]) -> Vec<&'static str> {
    violated.iter().map(|(name, _)| *name).collect()
}

// ---------------------------------------------------------------- N4：I-9.17 / I-9.23 只红自己

const CONTAINER_TREE_ENTRY_BYTES: usize = 120;

/// 最新那条根的树表里种类 `kind` 那棵容器树的码 2 根，第 `entry_index` 条条目改一处；交回根的层级。
fn change_an_entry_of_the_container_tree_root(
    image: &mut MemoryPool,
    kind: u16,
    entry_index: usize,
    change: impl FnOnce(&mut [u8]),
) -> u8 {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
    let mut level = 0;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        common_pointer_head_corruption::node_bytes(),
        |root| {
            level = root[50];
            let start = 115 + 2 * usize::from(root[51]) + entry_index * CONTAINER_TREE_ENTRY_BYTES;
            change(&mut root[start..start + CONTAINER_TREE_ENTRY_BYTES]);
        },
    );
    level
}

fn entry_type(entry: &[u8]) -> u16 {
    u16::from_le_bytes([entry[16], entry[17]])
}

fn set_entry_type(entry: &mut [u8], value: u16) {
    entry[16..18].copy_from_slice(&value.to_le_bytes());
}

fn report_n4(name: &str, clean: &MemoryPool, knob: Option<u64>, mutate: impl FnOnce(&mut MemoryPool) -> String) {
    let clean_violations = names(&violations(clean, knob));
    let mut image = clean.clone();
    let note = mutate(&mut image);
    let violated = violations(&image, knob);
    println!(
        "RESULT {name} clean={clean_violations:?} mutated({note}) violated={:?} I-9.23={} I-9.17={}",
        names(&violated),
        verdict_kind(&image, knob, "I-9.23").chars().take(4).collect::<String>(),
        verdict_kind(&image, knob, "I-9.17").chars().take(4).collect::<String>()
    );
    for (invariant, detail) in &violated {
        println!("DETAIL {name} {invariant}: {detail}");
    }
}

/// 建一个空 inode（2 号）再发它的 unlink 意图：墓碑树根由最新那次发布写出，层级 1、直接管一片容器。
fn image_with_a_tombstone_written_by_the_newest_publish(base: &SparsePool) -> MemoryPool {
    let mut pool = base.clone();
    pool.create(1).expect("建 2 号");
    pool.intent(2).expect("2 号意图");
    pool.image()
}

/// 建 2 号、删完、回退到删之前：谱系树根由最新那次发布（回退）写出。
fn image_with_a_lineage_tree_written_by_the_newest_publish(base: &SparsePool) -> MemoryPool {
    let mut pool = base.clone();
    pool.create(1).expect("建 2 号");
    let before = (pool.instance(), pool.output.root.checkpoint_txg.0);
    pool.intent(2).expect("2 号意图");
    pool.drain().expect("删完");
    pool.rollback(before).expect("回退到意图之前");
    pool.image()
}

fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1"),
        ),
        lineage_container: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(1).expect("1"),
        ),
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2"),
        ),
    }
}

/// 旋钮（一片 1 条、一个节点 2 条）下删 2、3、4 号：第 3 次意图那次墓碑树长到层级 2（照
/// `container_trees_grow_a_level_when_the_root_is_full.rs` 那条用例的造法）。
fn image_with_a_tombstone_tree_of_level_two(base: &SparsePool) -> MemoryPool {
    let mut pool = base.clone();
    pool.capacities = Some(capacities_of_one_record_and_two_entries());
    pool.create(5).expect("建 2..=6 号");
    for inode in [2, 3, 4] {
        pool.intent(inode).expect("意图");
        pool.drain().expect("删完");
    }
    pool.image()
}

#[test]
fn n4_container_tree_entries_whose_identity_does_not_match_redden_only_their_own_invariant() {
    let base = SparsePool::build();
    let tombstone = image_with_a_tombstone_written_by_the_newest_publish(&base);
    report_n4("N4-a 墓碑树根(层级1)条目类型段1->0", &tombstone, None, |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| {
            assert_eq!(entry_type(entry), 1);
            set_entry_type(entry, 0);
        });
        format!("root_level={level}")
    });
    report_n4("N4-b 墓碑树根(层级1)条目类型段1->5", &tombstone, None, |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| set_entry_type(entry, 5));
        format!("root_level={level}")
    });
    let lineage = image_with_a_lineage_tree_written_by_the_newest_publish(&base);
    report_n4("N4-c 谱系树根(层级1)条目类型段5->0", &lineage, None, |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_LINEAGE, 0, |entry| {
            assert_eq!(entry_type(entry), 5);
            set_entry_type(entry, 0);
        });
        format!("root_level={level}")
    });
    report_n4("N4-d 谱系树根(层级1)条目类型段5->1", &lineage, None, |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_LINEAGE, 0, |entry| set_entry_type(entry, 1));
        format!("root_level={level}")
    });
    let grown = image_with_a_tombstone_tree_of_level_two(&base);
    report_n4("N4-e 墓碑树根(层级2)条目类型段0->7", &grown, Some(1), |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| {
            assert_eq!(entry_type(entry), 0);
            set_entry_type(entry, 7);
        });
        format!("root_level={level}")
    });
    report_n4("N4-f 墓碑树根(层级2)条目类型段0->1", &grown, Some(1), |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| set_entry_type(entry, 1));
        format!("root_level={level}")
    });
    report_n4("N4-g 墓碑树根(层级2)类型段0条目身份容器号写1", &grown, Some(1), |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| {
            entry[18..26].copy_from_slice(&1u64.to_le_bytes());
        });
        format!("root_level={level}")
    });
    report_n4("N4-i 墓碑树根(层级1)条目类型段1->0且身份三段清零", &tombstone, None, |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| {
            set_entry_type(entry, 0);
            entry[8..16].fill(0);
            entry[18..34].fill(0);
        });
        format!("root_level={level}")
    });
    // 墓碑树长到层级 2 之后再建 30 次（根环转一圈多）：根以下那几个节点只剩现行这一棵墓碑树引用，别的候选版不再替它登记。
    let mut turned = base.clone();
    turned.capacities = Some(capacities_of_one_record_and_two_entries());
    turned.create(5).expect("建 2..=6 号");
    for inode in [2, 3, 4] {
        turned.intent(inode).expect("意图");
        turned.drain().expect("删完");
    }
    for _ in 0..30 {
        turned.create(1).expect("建一个");
    }
    let turned = turned.image();
    report_n4("N4-h 墓碑树根(层级2)条目类型段0->7(根环转过一圈)", &turned, Some(1), |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| {
            assert_eq!(entry_type(entry), 0);
            set_entry_type(entry, 7);
        });
        format!("root_level={level}")
    });
    report_n4("N4-j 墓碑树根(层级2)条目类型段0->1(根环转过一圈)", &turned, Some(1), |image| {
        let level = change_an_entry_of_the_container_tree_root(image, TREE_KIND_TOMBSTONE, 0, |entry| set_entry_type(entry, 1));
        format!("root_level={level}")
    });
}

// ---------------------------------------------------------------- N7：队列读不出、头 ID 配不上时 I-9.24 报什么

/// `deleted_inodes` 根（码 2、层级 0、key 宽 8、条目宽 8）再加一条 key（比现有的都大）。
fn add_a_key_to_the_deleted_inodes_root(image: &mut MemoryPool, key: u64) {
    let slot = common_deletion_in_progress::root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_DELETED_INODES);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        slot,
        common_pointer_head_corruption::node_bytes(),
        |root| {
            let key_width = usize::from(root[51]);
            assert_eq!(key_width, 8);
            let count_offset = 82 + 2 * key_width;
            let count = usize::from(u16::from_le_bytes([root[count_offset], root[count_offset + 1]]));
            let start = 115 + 2 * key_width + count * 8;
            root[start..start + 8].copy_from_slice(&key.to_le_bytes());
            root[count_offset..count_offset + 2].copy_from_slice(&u16::try_from(count + 1).expect("条目数").to_le_bytes());
            // 声明长度（偏移 8，2 字节）随条目数：8 × 条目数（`put_a_second_inode_being_deleted_into_the_deletion_queue` 同一改法）。
            root[8..10].copy_from_slice(&u16::try_from((count + 1) * 8).expect("声明长度").to_le_bytes());
            root[52 + key_width..52 + 2 * key_width].copy_from_slice(&key.to_le_bytes());
        },
    );
}

fn report_n7(name: &str, clean: &MemoryPool, mutate: impl FnOnce(&mut MemoryPool)) {
    let mut image = clean.clone();
    mutate(&mut image);
    let violated = violations(&image, None);
    println!(
        "RESULT {name} I-9.24={} violated={:?}",
        verdict_kind(&image, None, "I-9.24"),
        names(&violated)
    );
    for (invariant, detail) in &violated {
        println!("DETAIL {name} {invariant}: {detail}");
    }
}

#[test]
fn n7_the_queue_bound_when_the_queue_is_unreadable_or_its_head_identifier_names_no_inode_tree() {
    let base = SparsePool::build();
    let mut pool = base.clone();
    pool.create(2).expect("建 2、3 号");
    pool.intent(2).expect("2 号意图");
    let clean = pool.image();
    println!("RESULT N7-0 干净（队列 {{2}}） I-9.24={}", verdict_kind(&clean, None, "I-9.24"));
    report_n7("N7-a 头ID写成999", &clean, |image| {
        let slot = common_deletion_in_progress::tree_table_slot_of_the_newest_root(image);
        common_deletion_in_progress::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(image, slot, TREE_KIND_DELETED_INODES, 999);
    });
    report_n7("N7-b 队列根两份清零", &clean, |image| {
        let slot = common_deletion_in_progress::root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_DELETED_INODES);
        common_deletion_in_progress::zero_both_copies_of_the_unit_at(image, slot, common_pointer_head_corruption::node_bytes());
    });
    report_n7("N7-c 队列加一条key3", &clean, |image| add_a_key_to_the_deleted_inodes_root(image, 3));
    report_n7("N7-d 队列加一条key3且头ID写成999", &clean, |image| {
        add_a_key_to_the_deleted_inodes_root(image, 3);
        let slot = common_deletion_in_progress::tree_table_slot_of_the_newest_root(image);
        common_deletion_in_progress::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(image, slot, TREE_KIND_DELETED_INODES, 999);
    });
    report_n7("N7-e 队列加一条key3且根层级写成1", &clean, |image| {
        add_a_key_to_the_deleted_inodes_root(image, 3);
        let slot = common_deletion_in_progress::root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_DELETED_INODES);
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, slot, common_pointer_head_corruption::node_bytes(), |root| root[50] = 1);
    });
}

// ---------------------------------------------------------------- N5、N9：合法历史上有没有误红（用户决定的那几步放开扫）

/// 盯着的几条：判定种类计数（成立 / 不适用 / 部分没判 / 不成立）。
const WATCHED: [&str; 12] = [
    "I-9.5", "I-9.10", "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29", "I-9.6",
];

struct SplitMix(u64);
impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

#[derive(Default)]
pub struct Tally {
    pub checks: u64,
    pub red_checks: u64,
    pub kinds: BTreeMap<(&'static str, &'static str), u64>,
    pub acts: BTreeMap<String, (u64, u64)>,
    pub red_signatures: BTreeMap<String, u64>,
    /// 一段历史里同一个号先后见过不止一个对象出生代（崩溃恢复之后重发）的段数：I-9.29 那一格真被走到过的计数。
    pub histories_with_a_reissued_number: u64,
}

/// 判一张镜像：违例的那几条，盯着的几条记进计数。
pub fn judge_into(image: &MemoryPool, knob: Option<u64>, tally: &mut Tally) -> Vec<(&'static str, String)> {
    let verdicts = match knob {
        None => check_pool_image(image),
        Some(value) => check_pool_image_given_the_tombstone_container_capacity_knob(
            image,
            Some(TombstoneContainerCapacityKnob::new(value).expect("1..=815")),
        ),
    };
    tally.checks += 1;
    let mut violated = Vec::new();
    for (invariant, verdict) in verdicts {
        let kind = match &verdict {
            InvariantVerdict::Holds => "成立",
            InvariantVerdict::NotApplicable(_) => "不适用",
            InvariantVerdict::PartiallyJudged(_) => "部分没判",
            InvariantVerdict::Violated(_) => "不成立",
        };
        if WATCHED.contains(&invariant) {
            *tally.kinds.entry((invariant, kind)).or_insert(0) += 1;
        }
        if let InvariantVerdict::Violated(detail) = verdict {
            violated.push((invariant, detail));
        }
    }
    if !violated.is_empty() {
        tally.red_checks += 1;
    }
    violated
}

/// 起点：新池新建文件 → 第一个文件顺序写成 3 个数据单元 → 建 2、3 号（同一次）→ 建 4 号。交回池与这几版 (实例, txg)。
/// `switches` 给了就从新池新建文件之后起装上（写入口、分配器、之后每次挂载）。
pub fn starting_pool(switches: Option<singlefs_core::unlink::DeletionSwitches>) -> (SparsePool, Vec<(InstanceGeneration, u64)>) {
    let mut pool = SparsePool::build();
    if let Some(switches) = switches {
        pool.switches = Some(switches);
        pool.allocator.set_deletion_switches(switches);
    }
    let mut versions = vec![(pool.instance(), pool.output.root.checkpoint_txg.0)];
    pool.grow_the_first_file(3).expect("顺序写成 3 个单元");
    versions.push((pool.instance(), pool.output.root.checkpoint_txg.0));
    pool.create(2).expect("建 2、3 号");
    versions.push((pool.instance(), pool.output.root.checkpoint_txg.0));
    pool.create(1).expect("建 4 号");
    versions.push((pool.instance(), pool.output.root.checkpoint_txg.0));
    (pool, versions)
}

/// 一段随机历史：每一步从七种用户动作里抽（建、意图、删一批、走空、回退到第几新的版、进程退出再挂、崩溃恢复落到前一条根），
/// 做不成的（被拒）换回动作之前那一份、记 skip；做成的判一次。交回这段历史的一行摘要。
pub fn run_history(base: &SparsePool, base_versions: &[(InstanceGeneration, u64)], knob: Option<u64>, seed: u64, steps: usize, tally: &mut Tally) -> String {
    let mut rng = SplitMix(seed);
    let mut pool = base.clone();
    let mut versions = base_versions.to_vec();
    let mut trace = Vec::new();
    let mut reds = Vec::new();
    let mut births_seen: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
    for step_index in 0..steps {
        let mut attempt = pool.clone();
        let choice = rng.below(100);
        let (label, outcome) = if choice < 16 {
            let count = 1 + rng.below(2) as u64;
            (format!("create{count}"), attempt.create(count))
        } else if choice < 36 {
            let live: Vec<u64> = attempt.records().iter().filter(|(_, nlink, _)| *nlink >= 1).map(|(inode, _, _)| *inode).collect();
            if live.is_empty() {
                ("intent-none".to_string(), Err("没有活的号".to_string()))
            } else {
                let inode = live[rng.below(live.len())];
                (format!("intent{inode}"), attempt.intent(inode))
            }
        } else if choice < 50 {
            let outcome = if attempt.queue().is_empty() { Err("队列空".to_string()) } else { attempt.step() };
            ("step".to_string(), outcome)
        } else if choice < 56 {
            let outcome = if attempt.queue().is_empty() { Err("队列空".to_string()) } else { attempt.drain() };
            ("drain".to_string(), outcome)
        } else if choice < 82 {
            let current = (attempt.instance(), attempt.output.root.checkpoint_txg.0);
            let choices: Vec<(InstanceGeneration, u64)> = versions.iter().copied().filter(|v| *v != current).collect();
            let back = 1 + rng.below(choices.len().min(12));
            let target = choices[choices.len() - back];
            (format!("rollback(i{},t{})", target.0 .0, target.1), attempt.rollback(target))
        } else if choice < 91 {
            ("remount".to_string(), attempt.remount())
        } else {
            ("abandon".to_string(), attempt.abandon_the_newest())
        };
        let kind: String = label.chars().take_while(|c| c.is_ascii_lowercase() || *c == '-').collect();
        let entry = tally.acts.entry(kind).or_insert((0, 0));
        match outcome {
            Ok(()) => {
                entry.0 += 1;
                pool = attempt;
                versions.push((pool.instance(), pool.output.root.checkpoint_txg.0));
                for (inode, _, birth) in pool.records() {
                    births_seen.entry(inode).or_default().insert(birth);
                }
                let violated = judge_into(&pool.image(), knob, tally);
                trace.push(format!("{label}->t{}", pool.output.root.checkpoint_txg.0));
                if !violated.is_empty() {
                    let signature = format!("{:?}", names(&violated));
                    *tally.red_signatures.entry(signature.clone()).or_insert(0) += 1;
                    reds.push(format!("step{step_index}:{signature}"));
                    for (invariant, detail) in &violated {
                        println!("DETAIL seed={seed} step={step_index} {invariant}: {detail}");
                    }
                }
            }
            Err(reason) => {
                entry.1 += 1;
                trace.push(format!("{label}:skip({})", reason.chars().take(40).collect::<String>()));
            }
        }
    }
    let reissued: Vec<u64> = births_seen.iter().filter(|(_, births)| births.len() > 1).map(|(inode, _)| *inode).collect();
    if !reissued.is_empty() {
        tally.histories_with_a_reissued_number += 1;
    }
    format!("RESULT history seed={seed} reds={reds:?} reissued={reissued:?} trace={}", trace.join(" "))
}

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

/// 随机历史批：`OPUS_SEED_FROM`..`OPUS_SEED_FROM + OPUS_HISTORIES` 这几个种子，每段 `OPUS_STEPS` 步。起点池建一次，每段从内存里那一份拷。
#[test]
fn n5_n9_random_legal_histories() {
    let histories = environment_number("OPUS_HISTORIES", 4);
    let steps = environment_number("OPUS_STEPS", 8) as usize;
    let seed_from = environment_number("OPUS_SEED_FROM", 0);
    // OPUS_KNOB = 0：产品容量；k ≥ 1：墓碑与谱系容器一片 k 条、码 2 节点 2 条、删除批大小 1（树会长高、谱系树会长容器），池级 checker 走带旋钮值的入口。
    let knob = environment_number("OPUS_KNOB", 0);
    let switches = (knob > 0).then(|| singlefs_core::unlink::DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
        tombstone_container_capacity: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(knob).expect("1..=815"),
        ),
        lineage_container_capacity: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(knob).expect("1..=2039"),
        ),
        container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        ),
    });
    let knob = (knob > 0).then_some(knob);
    let (mut base, mut base_versions) = starting_pool(switches);
    let mut tally = Tally::default();
    // OPUS_PREFIX=reissue：固定前缀与故障（只建一次，之后每段从内存里那一份拷）——建一个号、那次发布被崩溃恢复抛弃（落到前一条根）、
    // 新线再建（重发同号，对象出生代不同）；之后的用户动作照常随机。
    if std::env::var("OPUS_PREFIX").as_deref() == Ok("reissue") {
        base.create(1).expect("前缀：建一个号");
        base_versions.push((base.instance(), base.output.root.checkpoint_txg.0));
        base.abandon_the_newest().expect("前缀：建号那次被崩溃恢复抛弃");
        base_versions.push((base.instance(), base.output.root.checkpoint_txg.0));
        base.create(1).expect("前缀：新线重发同号");
        base_versions.push((base.instance(), base.output.root.checkpoint_txg.0));
        let violated = judge_into(&base.image(), knob, &mut tally);
        println!("RESULT sweep prefix=reissue records={:?} violated={:?}", base.records(), names(&violated));
    }
    let started = std::time::Instant::now();
    for seed in seed_from..seed_from + histories {
        println!("{}", run_history(&base, &base_versions, knob, seed, steps, &mut tally));
    }
    eprintln!("TIMING histories={histories} steps={steps} checks={} seconds={:.1}", tally.checks, started.elapsed().as_secs_f64());
    println!("RESULT sweep knob={knob:?} seeds={seed_from}..{} steps={steps} checks={} red_checks={}", seed_from + histories, tally.checks, tally.red_checks);
    println!("RESULT sweep histories_with_a_reissued_number={}", tally.histories_with_a_reissued_number);
    for (act, (done, skipped)) in &tally.acts {
        println!("RESULT sweep act={act} done={done} skipped={skipped}");
    }
    for ((invariant, kind), count) in &tally.kinds {
        println!("RESULT sweep verdict {invariant} {kind} {count}");
    }
    for (signature, count) in &tally.red_signatures {
        println!("RESULT sweep red {signature} {count}");
    }
}

/// 一段写死前缀的历史逐步判；`steps` 每一步交回 Err 就停。
fn scenario(name: &str, base: &SparsePool, steps: Vec<(&str, Box<dyn Fn(&mut SparsePool, &BTreeMap<&str, (InstanceGeneration, u64)>) -> Result<(), String>>)>) {
    let mut pool = base.clone();
    let mut marks: BTreeMap<&str, (InstanceGeneration, u64)> = BTreeMap::new();
    marks.insert("start", (pool.instance(), pool.output.root.checkpoint_txg.0));
    let mut tally = Tally::default();
    for (label, act) in steps {
        match act(&mut pool, &marks) {
            Ok(()) => {
                marks.insert(label, (pool.instance(), pool.output.root.checkpoint_txg.0));
                let image = pool.image();
                let violated = judge_into(&image, None, &mut tally);
                println!(
                    "RESULT scenario {name} {label} i{}t{} queue={:?} records={:?} violated={:?} I-9.18={} I-9.20={} I-9.29={}",
                    pool.instance().0,
                    pool.output.root.checkpoint_txg.0,
                    pool.queue(),
                    pool.records(),
                    names(&violated),
                    verdict_kind(&image, None, "I-9.18").chars().take(12).collect::<String>(),
                    verdict_kind(&image, None, "I-9.20").chars().take(12).collect::<String>(),
                    verdict_kind(&image, None, "I-9.29").chars().take(12).collect::<String>(),
                );
                for (invariant, detail) in &violated {
                    println!("DETAIL scenario {name} {label} {invariant}: {detail}");
                }
            }
            Err(reason) => {
                println!("RESULT scenario {name} {label} refused: {}", reason.chars().take(120).collect::<String>());
                return;
            }
        }
    }
}

macro_rules! act {
    (|$p:ident, $m:pat_param| $body:expr) => {
        Box::new(|$p: &mut SparsePool, $m: &BTreeMap<&str, (InstanceGeneration, u64)>| $body)
            as Box<dyn Fn(&mut SparsePool, &BTreeMap<&str, (InstanceGeneration, u64)>) -> Result<(), String>>
    };
}

#[test]
fn n5_n9_scenarios_named_in_the_body() {
    let (base, _) = starting_pool(None);
    scenario("S1 删到底->回退到意图之前->再回退回删完那一版", &base, vec![
        ("intent2", act!(|p, _| p.intent(2))),
        ("done2", act!(|p, _| p.drain())),
        ("rb-before", act!(|p, m| p.rollback(m["start"]))),
        ("rb-done", act!(|p, m| p.rollback(m["done2"]))),
        ("rb-intent", act!(|p, m| p.rollback(m["intent2"]))),
        ("resume", act!(|p, _| p.drain())),
    ]);
    scenario("S2 删->回退->再删", &base, vec![
        ("intent2", act!(|p, _| p.intent(2))),
        ("done2", act!(|p, _| p.drain())),
        ("rb-before", act!(|p, m| p.rollback(m["start"]))),
        ("re-intent2", act!(|p, _| p.intent(2))),
        ("re-done2", act!(|p, _| p.drain())),
        ("rb-first-done", act!(|p, m| p.rollback(m["done2"]))),
        ("rb-second-intent", act!(|p, m| p.rollback(m["re-intent2"]))),
    ]);
    scenario("S3 三单元文件删除中间态回退", &base, vec![
        ("intent1", act!(|p, _| p.intent(1))),
        ("batch1", act!(|p, _| p.step())),
        ("batch2", act!(|p, _| p.step())),
        ("done1", act!(|p, _| p.drain())),
        ("rb-batch1", act!(|p, m| p.rollback(m["batch1"]))),
        ("resume-batch", act!(|p, _| p.step())),
        ("rb-intent1", act!(|p, m| p.rollback(m["intent1"]))),
        ("rb-start", act!(|p, m| p.rollback(m["start"]))),
        ("re-intent1", act!(|p, _| p.intent(1))),
    ]);
    scenario("S4 删除中途进程退出再挂（挂载里续做）", &base, vec![
        ("intent1", act!(|p, _| p.intent(1))),
        ("batch1", act!(|p, _| p.step())),
        ("remount", act!(|p, _| p.remount())),
        ("rb-intent1", act!(|p, m| p.rollback(m["intent1"]))),
        ("remount2", act!(|p, _| p.remount())),
    ]);
    scenario("S5 建号那次被崩溃恢复抛弃->新线重发同号", &base, vec![
        ("create5", act!(|p, _| p.create(1))),
        ("abandon", act!(|p, _| p.abandon_the_newest())),
        ("create-again", act!(|p, _| p.create(1))),
        ("intent-reissued", act!(|p, _| p.intent(5))),
        ("done", act!(|p, _| p.drain())),
    ]);
    scenario("S6 意图那次被崩溃恢复抛弃->再删", &base, vec![
        ("intent3", act!(|p, _| p.intent(3))),
        ("abandon", act!(|p, _| p.abandon_the_newest())),
        ("re-intent3", act!(|p, _| p.intent(3))),
        ("done3", act!(|p, _| p.drain())),
    ]);
    scenario("S7 回退到建号之前->再建", &base, vec![
        ("create5", act!(|p, _| p.create(1))),
        ("rb-start", act!(|p, m| p.rollback(m["start"]))),
        ("create-again", act!(|p, _| p.create(1))),
    ]);
    scenario("S8 回退那次被崩溃恢复抛弃", &base, vec![
        ("intent2", act!(|p, _| p.intent(2))),
        ("done2", act!(|p, _| p.drain())),
        ("rb-start", act!(|p, m| p.rollback(m["start"]))),
        ("abandon-rb", act!(|p, _| p.abandon_the_newest())),
        ("create", act!(|p, _| p.create(1))),
        ("rb-intent2", act!(|p, m| p.rollback(m["intent2"]))),
    ]);
}

// ---------------------------------------------------------------- N5 附：谱系记录本身坏了时那几条报什么；N6：产品镜像两片墓碑容器

#[test]
fn n5_a_lineage_record_whose_parent_is_not_older_than_its_rollback() {
    let base = SparsePool::build();
    let mut pool = base.clone();
    pool.create(1).expect("建 2 号");
    let before = (pool.instance(), pool.output.root.checkpoint_txg.0);
    pool.intent(2).expect("2 号意图");
    pool.drain().expect("删完");
    pool.rollback(before).expect("回退到意图之前");
    let clean = pool.image();
    println!("RESULT N5-L0 回退之后干净 violated={:?} I-9.18={} I-9.19={}", names(&violations(&clean, None)),
        verdict_kind(&clean, None, "I-9.18").chars().take(6).collect::<String>(),
        verdict_kind(&clean, None, "I-9.19").chars().take(6).collect::<String>());
    for (name, parent_of) in [("N5-L1 T_old改成T_rb", 0u64), ("N5-L2 T_old改成T_rb+1", 1u64)] {
        let mut image = clean.clone();
        let slot = common_deletion_in_progress::first_container_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_LINEAGE);
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            &mut image,
            slot,
            common_pointer_head_corruption::data_unit_bytes(),
            |container| {
                let rollback_txg = u64::from_le_bytes(container[136..144].try_into().expect("8"));
                container[144..152].copy_from_slice(&(rollback_txg + parent_of).to_le_bytes());
            },
        );
        let violated = violations(&image, None);
        println!(
            "RESULT {name} violated={:?} I-9.18={} I-9.19={} I-9.20={} I-9.29={}",
            names(&violated),
            verdict_kind(&image, None, "I-9.18"),
            verdict_kind(&image, None, "I-9.19").chars().take(6).collect::<String>(),
            verdict_kind(&image, None, "I-9.20").chars().take(6).collect::<String>(),
            verdict_kind(&image, None, "I-9.29").chars().take(6).collect::<String>()
        );
    }
}

/// 产品容量（一片 815 条）：建 820 个空 inode，删到底 816 个，墓碑树两片容器（第一片满 815、最右那片 1 条）；再回退到第二片容器出现之前那一版。
#[test]
fn n6_a_product_tombstone_tree_of_two_containers() {
    let mut pool = SparsePool::build();
    pool.create(820).expect("建 820 个");
    let mut before_the_second_container = None;
    for inode in 2..=817u64 {
        if inode == 817 {
            before_the_second_container = Some((pool.instance(), pool.output.root.checkpoint_txg.0));
        }
        pool.intent(inode).expect("意图");
        pool.drain().expect("删完");
    }
    let image = pool.image();
    println!(
        "RESULT N6-a 两片墓碑容器 violated={:?} I-9.20={} I-9.21={}",
        names(&violations(&image, None)),
        verdict_kind(&image, None, "I-9.20").chars().take(6).collect::<String>(),
        verdict_kind(&image, None, "I-9.21").chars().take(6).collect::<String>()
    );
    pool.rollback(before_the_second_container.expect("记下了")).expect("回退到第二片容器出现之前");
    pool.intent(818).expect("回退之后再删一个");
    pool.drain().expect("删完");
    let image = pool.image();
    println!(
        "RESULT N6-b 回退之后再删 violated={:?} I-9.20={} I-9.21={}",
        names(&violations(&image, None)),
        verdict_kind(&image, None, "I-9.20").chars().take(6).collect::<String>(),
        verdict_kind(&image, None, "I-9.21").chars().take(6).collect::<String>()
    );
}
