//! m3-batch24-code-r1 攻方原型（只在仓副本里编、跑；不入库代码）。
//! 基础设施（`SparsePool`、随机历史的七种用户动作）照抄自 `research/prompts/m3-batch22-code-r1-opus-model/opus_b22_attack.rs`，
//! 判的东西是这一轮自己的：N1 类型段 × 孩子的类 × 层数矩阵、两份副本一份陈旧的两故障形、N3 新九条的不适用理由在合法历史上的真假。
//! 盘用内存稀疏盘（`SparseBlockDevice`），起点池建一次、之后每段从内存里那一份拷。输出一律是 `RESULT ` 打头的行。
#![allow(dead_code, clippy::all, clippy::pedantic, unused_imports, unused_variables, unused_mut)]

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


// ================================================================ 这一轮自己的部分

use singlefs_checker::image::parse_node_pointer;
use singlefs_core::records::TREE_KIND_INODE;

const NODE: usize = 16384;
const DATA: usize = 32768;
const ENTRY: usize = 120;
const DEFAULT_REASON: &str = "这个镜像上没有它判的对象";

fn verdicts_of(image: &MemoryPool, knob: Option<u64>) -> Vec<(&'static str, InvariantVerdict)> {
    match knob {
        None => check_pool_image(image),
        Some(value) => check_pool_image_given_the_tombstone_container_capacity_knob(
            image,
            Some(TombstoneContainerCapacityKnob::new(value).expect("1..=815")),
        ),
    }
}

fn kind_of(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "成立",
        InvariantVerdict::Violated(_) => "不成立",
        InvariantVerdict::NotApplicable(_) => "不适用",
        InvariantVerdict::PartiallyJudged(_) => "部分没判",
    }
}

fn read_device(image: &MemoryPool, device: u32, slot: u64, length: usize) -> Vec<u8> {
    image.devices.get(&DeviceIdentity(device)).expect("盘在池里").read(SlotNumber(slot).to_device_offset(), length)
}

fn write_device(image: &mut MemoryPool, device: u32, slot: u64, bytes: &[u8]) {
    image.devices.get_mut(&DeviceIdentity(device)).expect("盘在池里").write(SlotNumber(slot).to_device_offset(), bytes);
}

fn entry_type_at(entry: &[u8]) -> u16 {
    u16::from_le_bytes([entry[16], entry[17]])
}

/// 一棵码 2 根之下（类型段 0、层级 ≥ 2 的往下走）的每个节点：(槽, 层级, key 宽, 条目, 深度)。
#[derive(Clone, Debug)]
struct NodeAt {
    slot: u64,
    level: u8,
    key_width: usize,
    entries: Vec<Vec<u8>>,
    depth: usize,
}

fn nodes_of_the_tree(image: &MemoryPool, root_slot: u64) -> Vec<NodeAt> {
    let mut out = Vec::new();
    let mut pending = vec![(root_slot, 0usize)];
    while let Some((slot, depth)) = pending.pop() {
        let unit = read_device(image, 0, slot, NODE);
        let view = singlefs_checker::index_node_view(&unit).expect("干净镜像上的节点解得开");
        for (index, entry) in view.entries.iter().enumerate() {
            let start = 115 + 2 * view.key_width + index * ENTRY;
            assert_eq!(&unit[start..start + ENTRY], entry.as_slice(), "条目偏移公式");
            if view.level >= 2 && entry_type_at(entry) == 0 {
                pending.push((parse_node_pointer(&entry[34..120]).locations[0].slot, depth + 1));
            }
        }
        out.push(NodeAt { slot, level: view.level, key_width: view.key_width, entries: view.entries, depth });
    }
    out.sort_by_key(|node| (node.depth, node.slot));
    out
}

fn root_slot_of_kind(image: &MemoryPool, kind: u16) -> Option<u64> {
    let root = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    if root.locations[0].slot.0 == 0 && root.locations[1].slot.0 == 0 {
        return None;
    }
    Some(common_pointer_head_corruption::the_slot_of(&root).0)
}

/// 节点 `slot` 第 `index` 条条目改一处，重封、整单元校验和沿引用链补到根槽。
fn change_an_entry_of_the_node(image: &mut MemoryPool, slot: u64, index: usize, change: impl FnOnce(&mut [u8])) {
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, SlotNumber(slot), NODE, |unit| {
        let key_width = usize::from(unit[51]);
        let start = 115 + 2 * key_width + index * ENTRY;
        change(&mut unit[start..start + ENTRY]);
    });
}

/// 按原来那一类的版式重封（改的可能就是类标签）：码 1 头 105、CRC 在 101；码 3 头 107、CRC 在 89；码 2 头 86 + 2k、CRC 在 76 + 2k。
fn reseal_as(bytes: &mut [u8], original_class: u8, key_width: usize) {
    let (header_end, payload_crc_offset) = match original_class {
        1 => (105usize, 101usize),
        3 => (107, 89),
        _ => (86 + 2 * key_width, 76 + 2 * key_width),
    };
    let payload_crc = singlefs_core::checksum::crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

/// 孩子单元自己的类标签改成 `class`（按原类的版式重封），新的整单元校验和沿引用链补上去。
fn change_the_class_tag_of_the_unit(image: &mut MemoryPool, slot: u64, length: usize, class: u8) {
    let mut bytes = read_device(image, 0, slot, length);
    let original = bytes[6];
    let key_width = usize::from(bytes[51]);
    bytes[6] = class;
    reseal_as(&mut bytes, original, key_width);
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(image, SlotNumber(slot), &bytes);
}

/// 一张坏镜像判下来：目标那条的判定、目标之外红的、删除线九条里报部分没判的。
fn judge_shape(image: &MemoryPool, knob: Option<u64>, target: &str, allowed: &[&str]) -> (String, Vec<&'static str>, Vec<&'static str>, String) {
    let verdicts = verdicts_of(image, knob);
    let target_kind = verdicts.iter().find(|(name, _)| *name == target).map(|(_, v)| kind_of(v)).unwrap_or("?").to_string();
    let extra: Vec<&'static str> = verdicts
        .iter()
        .filter(|(name, v)| matches!(v, InvariantVerdict::Violated(_)) && *name != target && !allowed.contains(name))
        .map(|(name, _)| *name)
        .collect();
    let partial: Vec<&'static str> = verdicts
        .iter()
        .filter(|(name, v)| matches!(v, InvariantVerdict::PartiallyJudged(_)) && NINE.contains(name))
        .map(|(name, _)| *name)
        .collect();
    let target_detail = verdicts
        .iter()
        .find(|(name, _)| *name == target)
        .map(|(_, v)| match v {
            InvariantVerdict::Violated(d) => d.chars().take(90).collect::<String>(),
            _ => String::new(),
        })
        .unwrap_or_default();
    (target_kind, extra, partial, target_detail)
}

const NINE: [&str; 9] = ["I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29"];

/// 判一形：目标要红、别的不许红（`allowed` 是字面上也被违反、可以连带红的那几条）。不满足的记 HIT。
fn report_shape(name: &str, image: &MemoryPool, knob: Option<u64>, target: &str, allowed: &[&str], tally: &mut BTreeMap<String, u64>) {
    let (kind, extra, partial, detail) = judge_shape(image, knob, target, allowed);
    let hit = kind != "不成立" || !extra.is_empty();
    *tally.entry(format!("{}", if hit { "HIT" } else { "ok" })).or_insert(0) += 1;
    println!(
        "RESULT shape {} {name} target={target}:{kind} extra={extra:?} partial_nine={partial:?}",
        if hit { "HIT" } else { "ok" }
    );
    if hit {
        let verdicts = verdicts_of(image, knob);
        for (invariant, verdict) in verdicts {
            if let InvariantVerdict::Violated(d) = verdict {
                println!("DETAIL {name} {invariant}: {}", d.chars().take(300).collect::<String>());
            }
        }
    }
}

fn knob_capacities(per_container: u64, node_entries: usize) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(per_container).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(per_container).expect("1..=2039"),
        ),
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(node_entries).expect("2..=135"),
        ),
    }
}

/// 删除开关：批大小 1、墓碑与谱系容器一片 `per_container` 条、码 2 节点 2 条。
fn knob_switches(per_container: u64) -> singlefs_core::unlink::DeletionSwitches {
    singlefs_core::unlink::DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
        tombstone_container_capacity: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(per_container).expect("1..=815"),
        ),
        lineage_container_capacity: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(per_container).expect("1..=2039"),
        ),
        container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        ),
    }
}

/// 旋钮（一片 1 条、一个节点 2 条）下建 `created` 个空号、删掉 2..2+`deleted` 那几个：墓碑树 `deleted` 片容器。
fn tombstone_pool(base: &SparsePool, created: u64, deleted: u64) -> SparsePool {
    let mut pool = base.clone();
    pool.capacities = Some(knob_capacities(1, 2));
    pool.create(created).expect("建号");
    for inode in 2..2 + deleted {
        pool.intent(inode).expect("意图");
        pool.drain().expect("删完");
    }
    pool
}

/// 旋钮下回退 `rollbacks` 次：每次删 2 号、回退到这一次删之前（谱系树每次多一条记录、一片容器）。
fn lineage_pool(base: &SparsePool, rollbacks: usize) -> SparsePool {
    let mut pool = base.clone();
    pool.capacities = Some(knob_capacities(1, 2));
    // 回退那一次的谱系容器容量读分配器上装的删除开关（写入口的旋钮管不到回退）。
    let switches = knob_switches(1);
    pool.switches = Some(switches);
    pool.allocator.set_deletion_switches(switches);
    pool.create(1).expect("建 2 号");
    for _ in 0..rollbacks {
        // 每次回退到这一轮意图之前那一版（它已经带着前几次回退写的谱系记录）：谱系记录一条条攒起来。
        let before = (pool.instance(), pool.output.root.checkpoint_txg.0);
        pool.intent(2).expect("2 号意图");
        pool.drain().expect("删完");
        pool.rollback(before).expect("回退到意图之前");
    }
    pool
}

fn turn_the_ring_over(pool: &mut SparsePool, publishes: usize) {
    for _ in 0..publishes {
        pool.create(1).expect("建号转根环");
    }
}

/// 一棵树（种类 `kind`）每个节点的每条条目：类型段换成别的每一个值。
fn type_segment_matrix(pool_name: &str, clean: &MemoryPool, knob: Option<u64>, kind: u16, target: &str, tally: &mut BTreeMap<String, u64>) {
    let Some(root) = root_slot_of_kind(clean, kind) else {
        println!("RESULT shape {pool_name} kind={kind} 零根");
        return;
    };
    let clean_red = names(&violations(clean, knob));
    println!("RESULT pool {pool_name} kind={kind} clean_violated={clean_red:?}");
    let nodes = nodes_of_the_tree(clean, root);
    // OPUS_SKIP / OPUS_TAKE：只判第 skip 形起的 take 形（转过根环的起点每形要几秒，分几段跑）。
    let skip = environment_number("OPUS_SKIP", 0) as usize;
    let take = environment_number("OPUS_TAKE", u64::MAX) as usize;
    let mut shape_index = 0usize;
    for node in &nodes {
        for (index, entry) in node.entries.iter().enumerate() {
            let from = entry_type_at(entry);
            for to in [0u16, 1, 2, 5, 7, 0xFFFF] {
                if to == from {
                    continue;
                }
                shape_index += 1;
                if shape_index <= skip || shape_index > skip.saturating_add(take) {
                    continue;
                }
                let mut image = clean.clone();
                change_an_entry_of_the_node(&mut image, node.slot, index, |e| e[16..18].copy_from_slice(&to.to_le_bytes()));
                let name = format!("{pool_name} type depth={} level={} entry={index}/{} {from}->{to}", node.depth, node.level, node.entries.len());
                report_shape(&name, &image, knob, target, &[], tally);
            }
        }
    }
}

/// 一棵树每个节点的每条条目：孩子单元自己的类标签换成别的登记码与一个没登记的码（按原类版式重封）。
fn child_class_matrix(pool_name: &str, clean: &MemoryPool, knob: Option<u64>, kind: u16, target: &str, tally: &mut BTreeMap<String, u64>) {
    let Some(root) = root_slot_of_kind(clean, kind) else {
        return;
    };
    let nodes = nodes_of_the_tree(clean, root);
    for node in &nodes {
        for (index, entry) in node.entries.iter().enumerate() {
            let child = parse_node_pointer(&entry[34..120]);
            let length = if entry_type_at(entry) == 0 { NODE } else { DATA };
            let original = read_device(clean, 0, child.locations[0].slot, length)[6];
            for class in [1u8, 2, 3, 9] {
                if class == original {
                    continue;
                }
                let mut image = clean.clone();
                change_the_class_tag_of_the_unit(&mut image, child.locations[0].slot, length, class);
                let name = format!("{pool_name} childclass depth={} level={} entry={index} {original}->{class}", node.depth, node.level);
                report_shape(&name, &image, knob, target, &["I-1.6", "I-1.7", "I-1.11", "I-2.4"], tally);
            }
        }
    }
}

#[test]
fn n1_type_segment_and_child_class_matrix_over_tree_depths() {
    // OPUS_POOL 选一张起点：T1 / T2 / T3（墓碑树层级 1 / 2 / 3）、T3ring（T3 再转一圈根环）、L1 / L2 / L3（谱系树层级 1 / 2 / 3）。
    let which = std::env::var("OPUS_POOL").unwrap_or_else(|_| "T1".to_string());
    let base = SparsePool::build();
    let mut tally = BTreeMap::new();
    let knob = Some(1);
    let started = std::time::Instant::now();
    let (pool, kind, target, with_child_class) = match which.as_str() {
        "T1" => (tombstone_pool(&base, 4, 2), TREE_KIND_TOMBSTONE, "I-9.23", true),
        "T2" => (tombstone_pool(&base, 5, 3), TREE_KIND_TOMBSTONE, "I-9.23", true),
        "T3" => (tombstone_pool(&base, 7, 5), TREE_KIND_TOMBSTONE, "I-9.23", true),
        "T3ring" => {
            let mut pool = tombstone_pool(&base, 7, 5);
            turn_the_ring_over(&mut pool, 34);
            (pool, TREE_KIND_TOMBSTONE, "I-9.23", false)
        }
        "L1" => (lineage_pool(&base, 2), TREE_KIND_LINEAGE, "I-9.17", true),
        "L2" => (lineage_pool(&base, 3), TREE_KIND_LINEAGE, "I-9.17", true),
        "L3" => (lineage_pool(&base, 5), TREE_KIND_LINEAGE, "I-9.17", false),
        other => panic!("不认得的起点 {other}"),
    };
    let clean = pool.image();
    eprintln!("TIMING pool {which} built seconds={:.1}", started.elapsed().as_secs_f64());
    type_segment_matrix(&which, &clean, knob, kind, target, &mut tally);
    if with_child_class {
        child_class_matrix(&which, &clean, knob, kind, target, &mut tally);
    }
    eprintln!("TIMING pool {which} seconds={:.1}", started.elapsed().as_secs_f64());
    for (key, count) in &tally {
        println!("RESULT tally {which} {key} {count}");
    }
}

/// inode 树（根层级 1）：几片叶；干净、删除进行到一半（队列里的号住在中间那片叶）、删完几个号（有墓碑）三种。
#[test]
fn n1_inode_tree_type_segment_and_child_class_matrix() {
    let base = SparsePool::build();
    let mut tally = BTreeMap::new();
    let mut pool = base.clone();
    for _ in 0..5 {
        pool.create(100).expect("建 100 个号");
    }
    let clean = pool.image();
    let leaves = nodes_of_the_tree(&clean, root_slot_of_kind(&clean, TREE_KIND_INODE).expect("inode 树"));
    println!("RESULT inode-pool leaves_of_root={} level={}", leaves[0].entries.len(), leaves[0].level);
    type_segment_matrix("I0", &clean, None, TREE_KIND_INODE, "I-9.2", &mut tally);
    child_class_matrix("I0", &clean, None, TREE_KIND_INODE, "I-9.2", &mut tally);
    // 删除进行到一半：中间那片叶里一个号发了意图（队列非空、记录 nlink 0、墓碑已写）。
    let middle_inode = {
        let records = pool.records();
        records[records.len() / 2].0
    };
    let mut half = pool.clone();
    half.intent(middle_inode).expect("意图");
    let clean_half = half.image();
    println!("RESULT inode-pool-half queued={:?}", half.queue());
    type_segment_matrix("I1half", &clean_half, None, TREE_KIND_INODE, "I-9.2", &mut tally);
    // 删完几个号：墓碑树有记录。
    let mut done = pool.clone();
    for inode in [middle_inode, middle_inode + 1, 3] {
        done.intent(inode).expect("意图");
        done.drain().expect("删完");
    }
    let clean_done = done.image();
    type_segment_matrix("I2done", &clean_done, None, TREE_KIND_INODE, "I-9.2", &mut tally);
    for (key, count) in &tally {
        println!("RESULT tally {key} {count}");
    }
}

/// 一份副本陈旧：盘 `device` 上孩子的槽起写进 `stale`（别处一个单元的原样字节，不补任何校验和）。
fn make_one_copy_stale(image: &mut MemoryPool, device: u32, slot: u64, stale: &[u8]) {
    write_device(image, device, slot, stale);
}

fn print_all_violations(name: &str, image: &MemoryPool, knob: Option<u64>, watch: &[&str]) {
    let verdicts = verdicts_of(image, knob);
    let red: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
    let watched: Vec<String> = watch
        .iter()
        .map(|w| format!("{w}={}", verdicts.iter().find(|(n, _)| n == w).map(|(_, v)| kind_of(v)).unwrap_or("?")))
        .collect();
    println!("RESULT stale {name} violated={red:?} {}", watched.join(" "));
    for (invariant, verdict) in verdicts {
        if let InvariantVerdict::Violated(d) = verdict {
            if watch.contains(&invariant) || invariant == "I-2.1" {
                println!("DETAIL stale {name} {invariant}: {}", d.chars().take(240).collect::<String>());
            }
        }
    }
}

/// 两处故障：条目改坏（类型段与孩子的类不符）+ 孩子的一份副本陈旧、陈旧那一份头里的类标签恰好是类型段说的那一类。
/// 对照：只有陈旧副本（条目不改）、只有条目改坏（副本不陈旧）、陈旧的是另一块盘。
#[test]
fn n1_entry_type_mismatch_with_one_stale_copy_of_the_child() {
    let base = SparsePool::build();
    let knob = Some(1);
    let zero_identity = |e: &mut [u8]| {
        e[8..16].fill(0);
        e[18..34].fill(0);
    };
    // 墓碑树层级 1：条目 0 类型段 1 → 0（身份三段清零，N4-i 那一形），孩子是码 3 容器；陈旧副本是墓碑树根（码 2）的原样字节。
    {
        let clean = tombstone_pool(&base, 4, 2).image();
        let root = root_slot_of_kind(&clean, TREE_KIND_TOMBSTONE).expect("墓碑树");
        let node = &nodes_of_the_tree(&clean, root)[0];
        let child = parse_node_pointer(&node.entries[0][34..120]);
        let child_slot = child.locations[0].slot;
        let stale_node = read_device(&clean, 0, root, NODE);
        print_all_violations("T1 clean", &clean, knob, &["I-9.23"]);
        for device in [0u32, 1] {
            let mut ctrl = clean.clone();
            make_one_copy_stale(&mut ctrl, device, child_slot, &stale_node);
            print_all_violations(&format!("T1 ctrl-stale-only dev{device}"), &ctrl, knob, &["I-9.23"]);
        }
        let mut n4i = clean.clone();
        change_an_entry_of_the_node(&mut n4i, root, 0, |e| {
            e[16..18].copy_from_slice(&0u16.to_le_bytes());
            zero_identity(e);
        });
        print_all_violations("T1 ctrl-n4i-only", &n4i, knob, &["I-9.23"]);
        for device in [0u32, 1] {
            let mut image = n4i.clone();
            make_one_copy_stale(&mut image, device, child_slot, &stale_node);
            print_all_violations(&format!("T1 n4i+stale dev{device}"), &image, knob, &["I-9.23"]);
        }
        // 身份不清零（N4-a 那一形）+ 陈旧副本。
        let mut n4a = clean.clone();
        change_an_entry_of_the_node(&mut n4a, root, 0, |e| e[16..18].copy_from_slice(&0u16.to_le_bytes()));
        let mut image = n4a.clone();
        make_one_copy_stale(&mut image, 0, child_slot, &stale_node);
        print_all_violations("T1 n4a+stale dev0", &image, knob, &["I-9.23"]);
    }
    // 墓碑树层级 2：根条目 0 类型段 0 → 1（N4-f），孩子是码 2 节点；陈旧副本是一片墓碑容器的前 16 KiB（头里类标签 3）。
    {
        let clean = tombstone_pool(&base, 5, 3).image();
        let root = root_slot_of_kind(&clean, TREE_KIND_TOMBSTONE).expect("墓碑树");
        let nodes = nodes_of_the_tree(&clean, root);
        let child = parse_node_pointer(&nodes[0].entries[0][34..120]);
        let child_slot = child.locations[0].slot;
        let some_container = nodes.iter().find(|n| n.level == 1).expect("层级 1 节点");
        let container_slot = parse_node_pointer(&some_container.entries[0][34..120]).locations[0].slot;
        let stale_head = read_device(&clean, 0, container_slot, NODE);
        assert_eq!(stale_head[6], 3);
        for device in [0u32, 1] {
            let mut ctrl = clean.clone();
            make_one_copy_stale(&mut ctrl, device, child_slot, &stale_head);
            print_all_violations(&format!("T2 ctrl-stale-only dev{device}"), &ctrl, knob, &["I-9.23"]);
        }
        let mut n4f = clean.clone();
        change_an_entry_of_the_node(&mut n4f, root, 0, |e| e[16..18].copy_from_slice(&1u16.to_le_bytes()));
        print_all_violations("T2 ctrl-n4f-only", &n4f, knob, &["I-9.23"]);
        for device in [0u32, 1] {
            let mut image = n4f.clone();
            make_one_copy_stale(&mut image, device, child_slot, &stale_head);
            print_all_violations(&format!("T2 n4f+stale dev{device}"), &image, knob, &["I-9.23"]);
        }
    }
    // inode 树根层级 1：条目 1 类型段 2 → 0（身份三段清零），孩子是码 3 叶容器；陈旧副本是 inode 树根（码 2）的原样字节。
    {
        let mut pool = base.clone();
        for _ in 0..5 {
            pool.create(100).expect("建 100 个号");
        }
        let clean = pool.image();
        let root = root_slot_of_kind(&clean, TREE_KIND_INODE).expect("inode 树");
        let node = &nodes_of_the_tree(&clean, root)[0];
        let child_slot = parse_node_pointer(&node.entries[1][34..120]).locations[0].slot;
        let stale_node = read_device(&clean, 0, root, NODE);
        for device in [0u32, 1] {
            let mut ctrl = clean.clone();
            make_one_copy_stale(&mut ctrl, device, child_slot, &stale_node);
            print_all_violations(&format!("I0 ctrl-stale-only dev{device}"), &ctrl, None, &["I-9.2"]);
        }
        let mut bad = clean.clone();
        change_an_entry_of_the_node(&mut bad, root, 1, |e| {
            e[16..18].copy_from_slice(&0u16.to_le_bytes());
            zero_identity(e);
        });
        print_all_violations("I0 ctrl-type0-zero-identity-only", &bad, None, &["I-9.2"]);
        for device in [0u32, 1] {
            let mut image = bad.clone();
            make_one_copy_stale(&mut image, device, child_slot, &stale_node);
            print_all_violations(&format!("I0 type0-zero-identity+stale dev{device}"), &image, None, &["I-9.2"]);
        }
    }
}

// ---------------------------------------------------------------- N3：合法历史上新九条的判定与不适用理由（用户决定的那几步放开扫）

use singlefs_checker::walk::{
    NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE, NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER, NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
    NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB,
    NO_TOMBSTONE_IN_ANY_WALKED_VERSION, NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
    NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION, PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB,
};

/// 专门理由一句一个短名（计数用）；不认得的记「别的：前 24 字」。
fn reason_tag(reason: &str) -> String {
    let named: [(&str, &str); 11] = [
        (NO_QUEUED_INODE_IN_ANY_WALKED_VERSION, "R-no-queue"),
        (NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD, "R-no-tombstone-naming-record"),
        (NO_TOMBSTONE_IN_ANY_WALKED_VERSION, "R-no-tombstone"),
        (NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE, "R-no-number"),
        (NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER, "R-no-non-rightmost"),
        (NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, "R-old-entry"),
        (PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB, "R-knob-product-not-judged"),
        (NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB, "R-no-container-under-knob"),
        (NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION, "R-no-lineage-entry"),
        (NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION, "R-no-tombstone-entry"),
        (DEFAULT_REASON, "R-DEFAULT"),
    ];
    for (text, tag) in named {
        if reason == text {
            return tag.to_string();
        }
    }
    format!("别的：{}", reason.chars().take(24).collect::<String>())
}

#[derive(Default)]
struct N3Tally {
    checks: u64,
    kinds: BTreeMap<String, u64>,
    flags: BTreeMap<String, u64>,
    acts: BTreeMap<String, (u64, u64)>,
}

/// 判一张合法镜像：新九条每条的判定种类（不适用的带理由短名），再按现行那一版的真值核「不适用」那句话真不真。
fn n3_judge(image: &MemoryPool, knob: Option<u64>, pool: &SparsePool, tally: &mut N3Tally) -> Vec<String> {
    let verdicts = verdicts_of(image, knob);
    tally.checks += 1;
    let queue_nonempty = !pool.queue().is_empty();
    let tombstone_containers = pool.output.tombstone_containers.len();
    let lineage_containers = pool.output.lineage_containers.len();
    let lineage_records: usize = pool.output.lineage_containers.iter().map(|c| c.contents.records.len()).sum();
    let records = pool.records();
    let tombstones: Vec<(u64, u64)> = pool
        .output
        .tombstone_containers
        .iter()
        .flat_map(|c| c.contents.records.iter())
        .map(|t| (t.object.0, t.object_birth.0))
        .collect();
    let tombstone_naming_a_record = tombstones.iter().any(|(n, b)| records.iter().any(|(rn, _, rb)| rn == n && rb == b));
    let mut flags = Vec::new();
    for (invariant, verdict) in &verdicts {
        if !NINE.contains(invariant) {
            if let InvariantVerdict::Violated(d) = verdict {
                flags.push(format!("RED-other {invariant}: {}", d.chars().take(120).collect::<String>()));
            }
            continue;
        }
        let key = match verdict {
            InvariantVerdict::NotApplicable(reason) => format!("{invariant} 不适用 {}", reason_tag(reason)),
            other => format!("{invariant} {}", kind_of(other)),
        };
        *tally.kinds.entry(key).or_insert(0) += 1;
        match verdict {
            InvariantVerdict::Violated(d) => flags.push(format!("RED {invariant}: {}", d.chars().take(160).collect::<String>())),
            InvariantVerdict::NotApplicable(reason) => {
                let tag = reason_tag(reason);
                let false_claim = match (*invariant, tag.as_str()) {
                    (_, "R-DEFAULT") => Some("落到缺省理由".to_string()),
                    ("I-9.18", "R-no-queue") if queue_nonempty => Some("现行那一版队列非空".to_string()),
                    ("I-9.20", "R-no-tombstone") if tombstone_containers > 0 => Some("现行那一版墓碑树不是零根".to_string()),
                    ("I-9.23", "R-no-tombstone-entry") if tombstone_containers > 0 => Some("现行那一版墓碑树不是零根".to_string()),
                    ("I-9.17", "R-no-lineage-entry") if lineage_containers > 0 => Some("现行那一版谱系树不是零根".to_string()),
                    ("I-9.21", "R-no-non-rightmost") if tombstone_containers > 1 => Some("现行那一版墓碑容器多于一片".to_string()),
                    ("I-9.22", "R-no-container-under-knob") if tombstone_containers > 0 => Some("现行那一版墓碑树不是零根".to_string()),
                    ("I-9.29", "R-no-number") if !records.is_empty() => Some("现行那一版有 inode 记录".to_string()),
                    ("I-9.19", "R-no-tombstone-naming-record") if lineage_records == 0 && tombstone_naming_a_record => {
                        Some("没有谱系记录而现行那一版有墓碑指着同号同出生代的记录".to_string())
                    }
                    ("I-9.24", _) => Some("I-9.24 报了不适用".to_string()),
                    _ => None,
                };
                if let Some(why) = false_claim {
                    flags.push(format!("FALSE-NA {invariant} [{tag}] {why}"));
                }
            }
            _ => {}
        }
    }
    for flag in &flags {
        let head: String = flag.split(':').next().unwrap_or("").chars().take(60).collect();
        *tally.flags.entry(head).or_insert(0) += 1;
    }
    flags
}

/// 一段随机历史（七种用户动作，同 `run_history`），每做成一步判一次 N3。
fn run_history_n3(base: &SparsePool, base_versions: &[(InstanceGeneration, u64)], knob: Option<u64>, seed: u64, steps: usize, tally: &mut N3Tally) -> String {
    let mut rng = SplitMix(seed);
    let mut pool = base.clone();
    let mut versions = base_versions.to_vec();
    let mut trace = Vec::new();
    let mut flagged = Vec::new();
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
                let flags = n3_judge(&pool.image(), knob, &pool, tally);
                trace.push(format!("{label}->t{}", pool.output.root.checkpoint_txg.0));
                for flag in &flags {
                    println!("DETAIL seed={seed} step={step_index} {flag}");
                }
                if !flags.is_empty() {
                    flagged.push(step_index);
                }
            }
            Err(reason) => {
                entry.1 += 1;
                trace.push(format!("{label}:skip({})", reason.chars().take(40).collect::<String>()));
            }
        }
    }
    format!("RESULT n3history seed={seed} flagged_steps={flagged:?} trace={}", trace.join(" "))
}

/// N3 随机历史批：`OPUS_SEED_FROM`..+`OPUS_HISTORIES`，每段 `OPUS_STEPS` 步；`OPUS_KNOB` 同 `n5_n9_random_legal_histories`；`OPUS_PREFIX=reissue` 同。
#[test]
fn n3_reasons_of_the_nine_on_random_legal_histories() {
    let histories = environment_number("OPUS_HISTORIES", 4);
    let steps = environment_number("OPUS_STEPS", 8) as usize;
    let seed_from = environment_number("OPUS_SEED_FROM", 0);
    let knob_value = environment_number("OPUS_KNOB", 0);
    let switches = (knob_value > 0).then(|| knob_switches(knob_value));
    let knob = (knob_value > 0).then_some(knob_value);
    let (mut base, mut base_versions) = starting_pool(switches);
    let mut tally = N3Tally::default();
    if std::env::var("OPUS_PREFIX").as_deref() == Ok("reissue") {
        base.create(1).expect("前缀：建一个号");
        base_versions.push((base.instance(), base.output.root.checkpoint_txg.0));
        base.abandon_the_newest().expect("前缀：建号那次被崩溃恢复抛弃");
        base_versions.push((base.instance(), base.output.root.checkpoint_txg.0));
        base.create(1).expect("前缀：新线重发同号");
        base_versions.push((base.instance(), base.output.root.checkpoint_txg.0));
    }
    let flags = n3_judge(&base.image(), knob, &base, &mut tally);
    println!("RESULT n3start knob={knob:?} flags={flags:?}");
    let started = std::time::Instant::now();
    for seed in seed_from..seed_from + histories {
        println!("{}", run_history_n3(&base, &base_versions, knob, seed, steps, &mut tally));
    }
    eprintln!("TIMING n3 histories={histories} steps={steps} checks={} seconds={:.1}", tally.checks, started.elapsed().as_secs_f64());
    println!("RESULT n3sweep knob={knob:?} seeds={seed_from}..{} steps={steps} checks={}", seed_from + histories, tally.checks);
    for (act, (done, skipped)) in &tally.acts {
        println!("RESULT n3sweep act={act} done={done} skipped={skipped}");
    }
    for (key, count) in &tally.kinds {
        println!("RESULT n3sweep verdict {key} {count}");
    }
    for (flag, count) in &tally.flags {
        println!("RESULT n3sweep flag {flag} {count}");
    }
}

/// N3 一段写死前缀的历史：删完一个号之后再建 `turns` 次号（根环转过去，意图那一版、队列非空的那几版转出候选集），看新九条报什么。
#[test]
fn n3_after_the_queued_versions_leave_the_candidate_set() {
    let (base, _versions) = starting_pool(None);
    for turns in [0usize, 10, 30] {
        let mut pool = base.clone();
        pool.intent(2).expect("2 号意图");
        pool.drain().expect("删完");
        for _ in 0..turns {
            pool.create(1).expect("建号");
        }
        let image = pool.image();
        let verdicts = verdicts_of(&image, None);
        let line: Vec<String> = NINE
            .iter()
            .map(|name| {
                let verdict = &verdicts.iter().find(|(n, _)| n == name).expect("清单里有").1;
                match verdict {
                    InvariantVerdict::NotApplicable(reason) => format!("{name}=不适用[{}]", reason_tag(reason)),
                    other => format!("{name}={}", kind_of(other)),
                }
            })
            .collect();
        println!(
            "RESULT n3turn turns={turns} queue={:?} tombstones={} {}",
            pool.queue(),
            pool.output.tombstone_containers.iter().map(|c| c.contents.records.len()).sum::<usize>(),
            line.join(" ")
        );
    }
}
