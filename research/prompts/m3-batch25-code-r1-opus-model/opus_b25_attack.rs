//! m3-batch25-code-r1 攻方原型（只在仓副本里编、跑；不入库代码）。
//! 基础设施（`SparsePool`、用户动作、码 2 树走读与条目改写）照抄自 `research/prompts/m3-batch24-code-r1-opus-model/opus_b24_attack.rs`
//! （它又抄自 m3-batch22-code-r1 攻方原型），判的东西是这一轮自己的：
//!   A 段：I-9.29 判不了的门（主走读沿最新根的实例表链走没走到末片）在最新根实例表的各种形状上判什么；
//!   B 段：合法历史（建号、unlink、回退、再挂、崩溃恢复落到前一条根、实例表长到两片）上 I-9.29 与门、删除线九条的理由；
//!   C 段：m3-batch24-code-r1 N1 单故障的第二次抽样（换一组类型段取值与条目身份字段）。
//! 盘用内存稀疏盘（`SparseBlockDevice`），起点池建一次、之后每段从内存里那一份拷。输出一律是 `RESULT ` 打头的行。
#![allow(dead_code, clippy::all, clippy::pedantic, unused_imports, unused_variables, unused_mut)]

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::collections::{BTreeMap, BTreeSet};

use common::{file_content, named_unit_bytes, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::{parse_node_pointer, InvariantVerdict};
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_tombstone_container_capacity_knob, TombstoneContainerCapacityKnob,
    NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE, NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER, NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
    NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB,
    NO_TOMBSTONE_IN_ANY_WALKED_VERSION, NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
    NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION, PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_TOMBSTONE};
use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write, publish_unlink_intent,
    publish_unlink_step, publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const WT: u64 = 1_788_000_100;

fn new_device() -> SparseBlockDevice {
    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

/// 两块内存稀疏盘上的池：设备、现行版本、分配器、写入口要装的容器树旋钮（`None` = 产品容量）、删除开关、
/// 再挂时实例表清扫交给挂着时的后台（`true`：没有调度器，行只增不删，用来让实例表长到两片）。
pub struct SparsePool {
    pub devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
    pub capacities: Option<ContainerTreeCapacities>,
    pub switches: Option<singlefs_core::unlink::DeletionSwitches>,
    pub background_sweep: bool,
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
            background_sweep: self.background_sweep,
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
        Self { devices, output, allocator, capacities: None, switches: None, background_sweep: false }
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

    pub fn mark(&self) -> (InstanceGeneration, u64) {
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
            RollbackTarget { instance: target.0, checkpoint_txg: CheckpointTxg(target.1) },
        )
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
    }

    /// 进程退出（什么都已持久）之后再可写挂载：新实例写行、暖机，删除续做。
    pub fn remount(&mut self) -> Result<(), String> {
        let parameters = parameters();
        let mounted = mount_with(&parameters, &mut self.devices, self.switches, self.background_sweep)
            .map_err(|e| format!("{e:?}"))?;
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.version_of_trees().ok_or_else(|| "落到的那一版树表 0 条".to_string())?.clone();
        Ok(())
    }
}

impl SparsePool {
    /// 崩溃恢复落到最新那条根之前那条根，最新那条根之后又读得出（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`
    /// 那一形，换成稀疏盘、藏的是最新那次发布的根槽与它那条记录点名的单元）：见证它的系统配置轮换没落盘（改写成轮换之前那一代），
    /// 挂载时根槽与点名单元一时读不出（清零），挂载交回之后原样写回。恢复没落到前一条根的交回 `Err`。交回被抛弃那一版的 (实例, txg)。
    pub fn abandon_the_newest(&mut self) -> Result<(InstanceGeneration, u64), String> {
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
        let mounted = mount_with(&parameters, &mut self.devices, self.switches, self.background_sweep);
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
            return Err(format!("恢复没落到前一条根（落到 txg {}）", mounted.output.effective_root.checkpoint_txg.0));
        }
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.version_of_trees().ok_or("落到的那一版树表 0 条")?.clone();
        Ok((newest.root.instance, newest.root.checkpoint_txg.0))
    }
}

/// 可写挂载：装了删除开关的走带开关的入口；`background_sweep` 时实例表清扫交给挂着时的后台（行只增不删）；都没有走产品入口。
fn mount_with(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    switches: Option<singlefs_core::unlink::DeletionSwitches>,
    background_sweep: bool,
) -> Result<singlefs_core::mount::Mounted, singlefs_core::mount::MountError> {
    use singlefs_core::instance_table_sweep::{
        InstanceTableRowCountThreshold, InstanceTableSweepConfiguration, InstanceTableSweepSwitch, SweepPublishPlacements,
    };
    match (switches, background_sweep) {
        (Some(switches), _) => singlefs_core::mount::mount_writable_with_test_only_switches_and_deletion_switches(
            parameters,
            devices,
            singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
            singlefs_core::mount::ShadowLedger::On,
            switches,
        ),
        (None, true) => singlefs_core::mount::mount_writable_with_instance_table_sweep_configuration(
            parameters,
            devices,
            singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
            singlefs_core::mount::ShadowLedger::On,
            InstanceTableSweepConfiguration {
                switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
                row_count_threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690,
                sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
            },
        ),
        (None, false) => mount_writable(parameters, devices),
    }
}

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

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

fn violated_names(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect()
}

/// I-9.29 的判定写成短串：成立 / 不成立 / 部分没判[门] / 部分没判[别的] / 不适用[理由前 12 字]。
fn one_birth_tag(verdicts: &[(&'static str, InvariantVerdict)]) -> String {
    let verdict = &verdicts.iter().find(|(n, _)| *n == "I-9.29").expect("清单里有").1;
    match verdict {
        InvariantVerdict::Holds => "成立".to_string(),
        InvariantVerdict::Violated(_) => "不成立".to_string(),
        InvariantVerdict::PartiallyJudged(detail) if detail.starts_with("最新根的实例表读不出") => "部分没判[门]".to_string(),
        InvariantVerdict::PartiallyJudged(detail) => format!("部分没判[{}]", detail.chars().take(16).collect::<String>()),
        InvariantVerdict::NotApplicable(reason) => format!("不适用[{}]", reason.chars().take(12).collect::<String>()),
    }
}

const DEFAULT_REASON: &str = "这个镜像上没有它判的对象";
const NINE: [&str; 9] = ["I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29"];

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
    format!("R-other:{}", reason.chars().take(20).collect::<String>())
}

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

// ================================================================ A 段：最新根的实例表形状 × I-9.29 的门

use singlefs_core::address::DeviceOffsetInBytes;

const SLOT: u64 = 16384;
const ROOT_BYTES: usize = 512;
const IT_BYTES: usize = 32768;
const RECORD: usize = 88;
const FIRST_RECORD: usize = 136;
/// 新造的实例表片放在这一槽起（每片两槽、两盘同槽），离池里用到的槽远。
const FRESH_BASE: u64 = 200_000;

fn rd(image: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    image.devices.get(&DeviceIdentity(device)).expect("盘").read(DeviceOffsetInBytes(offset), length)
}

fn wr(image: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    image.devices.get_mut(&DeviceIdentity(device)).expect("盘").write(DeviceOffsetInBytes(offset), bytes);
}

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().expect("4"))
}
fn u64_at(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().expect("8"))
}
fn slot_at(b: &[u8], o: usize) -> u64 {
    let mut s = [0u8; 8];
    s[..6].copy_from_slice(&b[o..o + 6]);
    u64::from_le_bytes(s)
}

/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096（`checker_known_bad_images.rs` 的 `root_slots` 同一个算法）。
fn root_slots() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| ([0u32, 1, 0][region as usize], 64 * SLOT + region * 3 * (1 << 20) + slot * 4096))
        })
        .collect()
}

#[derive(Clone, Debug)]
struct RingRoot {
    device: u32,
    offset: u64,
    bytes: Vec<u8>,
    instance: u32,
    txg: u64,
}

fn ring_roots(image: &MemoryPool) -> Vec<RingRoot> {
    root_slots()
        .into_iter()
        .filter_map(|(device, offset)| {
            let bytes = rd(image, device, offset, ROOT_BYTES);
            if &bytes[..4] != b"SFSR" {
                return None;
            }
            let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&bytes, ROOT_BYTES, 138);
            if bytes[138..170] != digest[..] {
                return None;
            }
            let instance = u32_at(&bytes, 24);
            let txg = u64_at(&bytes, 28);
            Some(RingRoot { device, offset, bytes, instance, txg })
        })
        .collect()
}

fn newest_root(image: &MemoryPool) -> RingRoot {
    let mut best: Option<RingRoot> = None;
    for root in ring_roots(image) {
        if best.as_ref().map_or(true, |b| (root.txg, root.instance) > (b.txg, b.instance)) {
            best = Some(root);
        }
    }
    best.expect("根环里有根")
}

/// 最新根的实例表第 0 片：槽（两盘同槽）与盘 0 那一份的字节。
fn newest_instance_table(image: &MemoryPool) -> (u64, Vec<u8>) {
    let root = newest_root(image);
    let pointer = &root.bytes[170..256];
    let slot0 = slot_at(pointer, 50 + 4);
    let slot1 = slot_at(pointer, 64 + 4);
    assert_eq!(slot0, slot1, "实例表两盘同槽");
    (slot0, rd(image, 0, slot0 * SLOT, IT_BYTES))
}

fn records_of(page: &[u8]) -> Vec<Vec<u8>> {
    let count = usize::from(u16_at(page, 69));
    (0..count).map(|i| page[FIRST_RECORD + i * RECORD..FIRST_RECORD + (i + 1) * RECORD].to_vec()).collect()
}

fn reseal_packed(bytes: &mut [u8]) {
    let payload_crc = singlefs_core::checksum::crc32_castagnoli(&bytes[107..]);
    bytes[89..93].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, 107);
}

/// 照 `template`（一片实例表）的头造链上第 `page_index` 片：载荷换成 `records`，记录数、声明长度、容器号跟着改，重封。
fn build_page(template: &[u8], records: &[Vec<u8>], page_index: u64) -> Vec<u8> {
    let mut bytes = template.to_vec();
    for byte in &mut bytes[FIRST_RECORD..] {
        *byte = 0;
    }
    for (i, record) in records.iter().enumerate() {
        bytes[FIRST_RECORD + i * RECORD..FIRST_RECORD + (i + 1) * RECORD].copy_from_slice(record);
    }
    bytes[69..71].copy_from_slice(&(records.len() as u16).to_le_bytes());
    bytes[8..10].copy_from_slice(&((records.len() * RECORD) as u16).to_le_bytes());
    bytes[53..61].copy_from_slice(&page_index.to_le_bytes());
    reseal_packed(&mut bytes);
    bytes
}

/// 链指针记录：kind 1、有无下一片、指针 86（有下一片时照最新根的实例表指针改两条位置条目：盘 0 / 盘 1 同槽、整单元校验和）。
fn chain_record(template_pointer: &[u8], next: Option<(u64, u32)>) -> Vec<u8> {
    let mut record = vec![0u8; RECORD];
    record[0] = 1;
    if let Some((slot, checksum)) = next {
        record[1] = 1;
        let mut pointer = template_pointer.to_vec();
        for (index, device) in [(50usize, 0u32), (64, 1)] {
            pointer[index..index + 4].copy_from_slice(&device.to_le_bytes());
            pointer[index + 4..index + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
            pointer[index + 10..index + 14].copy_from_slice(&checksum.to_le_bytes());
        }
        record[2..88].copy_from_slice(&pointer);
    }
    record
}

/// 第 0 片换成 `bytes`（两盘），整单元校验和沿引用链补到根槽与 journal 新根段。
fn replace_first_page(image: &mut MemoryPool, slot: u64, bytes: &[u8]) {
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(image, SlotNumber(slot), bytes);
}

/// 把最新根的实例表改造成 `pages` 片（`rows_of_page[k]` 是第 k 片的行，每片末尾补链指针记录）；第 1 片起写在新槽上。
/// `then` 在第 1 片起各片写完、第 0 片重封之前改各片（下标 k ≥ 1 时改的是盘上那一片，交回 true 表示改过、要重写）。
fn rebuild_newest_instance_table(image: &mut MemoryPool, rows_of_page: &[Vec<Vec<u8>>], page_two_copies: &dyn Fn(usize, &mut Vec<u8>, &mut Vec<u8>)) {
    let (slot0, page0) = newest_instance_table(image);
    let root = newest_root(image);
    let template_pointer = root.bytes[170..256].to_vec();
    let pages = rows_of_page.len();
    // 从最后一片往前造：每片要知道下一片的整单元校验和。
    let mut next: Option<(u64, u32)> = None;
    for k in (1..pages).rev() {
        let mut records = rows_of_page[k].clone();
        records.push(chain_record(&template_pointer, next));
        let page = build_page(&page0, &records, k as u64);
        let slot = FRESH_BASE + 2 * k as u64;
        let mut copy0 = page.clone();
        let mut copy1 = page.clone();
        page_two_copies(k, &mut copy0, &mut copy1);
        wr(image, 0, slot * SLOT, &copy0);
        wr(image, 1, slot * SLOT, &copy1);
        next = Some((slot, singlefs_core::checksum::crc32_castagnoli(&page)));
    }
    let mut records = rows_of_page[0].clone();
    records.push(chain_record(&template_pointer, next));
    let first = build_page(&page0, &records, 0);
    replace_first_page(image, slot0, &first);
}

fn rows_of_the_newest_table(image: &MemoryPool) -> Vec<Vec<u8>> {
    let (_, page0) = newest_instance_table(image);
    records_of(&page0).into_iter().filter(|r| r[0] == 0).collect()
}

fn row_summary(rows: &[Vec<u8>]) -> String {
    rows.iter().map(|r| format!("({},{})", u32_at(r, 1), u64_at(r, 5))).collect::<Vec<_>>().join("")
}

/// core 的读法（`recovery::instance_table_of_root`，择根择到的那条根）读不读得出最新根的实例表。
fn core_reads_the_newest_table(image: &MemoryPool) -> &'static str {
    let root = common_pointer_head_corruption::newest_root(image);
    match singlefs_core::recovery::instance_table_of_root(image, &root) {
        Some(_) => "core读得出",
        None => "core读不出",
    }
}

fn report_a(name: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    println!(
        "RESULT A {name} I-9.29={} {} violated={:?}",
        one_birth_tag(&verdicts),
        core_reads_the_newest_table(image),
        violated_names(&verdicts)
    );
}

/// 起点 X：一段合法历史，被崩溃恢复抛弃的那一版与现行线同号不同出生代（不改任何字节）：
/// 新池新建文件（实例 1）→ 再挂（实例 2，写行、暖机）→ 建一个号（C）→ 崩溃恢复落到 C 之前那条根（实例 3）→ 再建一个号。
/// 交回池、被抛弃的 C 的 (实例, txg)、C 上与现行线上那几个号的出生代。
fn pool_x() -> (SparsePool, (InstanceGeneration, u64), Vec<(u64, u32, u64)>, Vec<(u64, u32, u64)>) {
    let mut pool = SparsePool::build();
    pool.remount().expect("再挂");
    pool.create(1).expect("C：建一个号");
    let on_c = pool.records();
    let abandoned = pool.abandon_the_newest().expect("崩溃恢复落到 C 之前那条根");
    pool.create(1).expect("现行线上再建一个号");
    let on_current = pool.records();
    (pool, abandoned, on_c, on_current)
}

/// 链指针记录之前插一条 `kind` 的记录（记录数加一）。
fn insert_before_chain(page: &mut Vec<u8>, kind: u8) {
    let mut records = records_of(page);
    let chain = records.pop().expect("链指针记录");
    let mut extra = vec![0u8; RECORD];
    extra[0] = kind;
    records.push(extra);
    records.push(chain);
    let rebuilt = build_page(page, &records, u64_at(page, 53));
    *page = rebuilt;
}

#[test]
fn a_gate_shapes_on_the_newest_instance_table() {
    let (pool, abandoned, on_c, on_current) = pool_x();
    println!("RESULT A pool-x abandoned=(i{},t{}) records_on_C={on_c:?} records_on_current={on_current:?}", abandoned.0 .0, abandoned.1);
    let clean = pool.image();
    let rows = rows_of_the_newest_table(&clean);
    let newest = newest_root(&clean);
    println!(
        "RESULT A pool-x newest=(i{},t{}) rows={} ring={:?}",
        newest.instance,
        newest.txg,
        row_summary(&rows),
        { let mut r: Vec<(u32, u64)> = ring_roots(&clean).iter().map(|r| (r.instance, r.txg)).collect(); r.sort(); r }
    );
    let (slot0, page0) = newest_instance_table(&clean);
    report_a("A0 干净", &clean);

    let shape = |name: &str, change: &dyn Fn(&mut Vec<u8>), reseal: bool| {
        let mut image = clean.clone();
        let mut bytes = page0.clone();
        change(&mut bytes);
        if reseal {
            reseal_packed(&mut bytes);
        }
        replace_first_page(&mut image, slot0, &bytes);
        report_a(name, &image);
    };
    // 两份里只坏一份 / 两份都坏（翻一个载荷字节、不重封，整单元校验和对不上）
    for (name, devices) in [("A1 盘0那份坏", vec![0u32]), ("A1 盘1那份坏", vec![1]), ("A2 两份都坏", vec![0, 1])] {
        let mut image = clean.clone();
        for device in devices {
            let mut bytes = rd(&image, device, slot0 * SLOT, IT_BYTES);
            bytes[IT_BYTES - 100] ^= 0xff;
            wr(&mut image, device, slot0 * SLOT, &bytes);
        }
        report_a(name, &image);
    }
    shape("A3 载荷CRC翻一位只重封头", &|b| {
        b[89] ^= 1;
        singlefs_core::unit::seal_header_checksum(b, 107);
    }, false);
    for kind in [2u8, 3, 0x80, 0xff] {
        shape(&format!("A4 链指针记录前插kind{kind}"), &|b| insert_before_chain(b, kind), false);
    }
    shape("A4 第0条前插kind2", &|b| {
        let mut records = records_of(b);
        let mut extra = vec![0u8; RECORD];
        extra[0] = 2;
        records.insert(0, extra);
        *b = build_page(b, &records, 0);
    }, false);
    for flag in [2u8, 3, 0xff] {
        shape(&format!("A5 有无下一片写{flag}"), &|b| {
            let count = usize::from(u16_at(b, 69));
            b[FIRST_RECORD + (count - 1) * RECORD + 1] = flag;
        }, true);
    }
    shape("A6 无下一片而指针不全零", &|b| {
        let count = usize::from(u16_at(b, 69));
        b[FIRST_RECORD + (count - 1) * RECORD + 2 + 50] = 1;
    }, true);
    shape("A7 去掉链指针记录", &|b| {
        let mut records = records_of(b);
        records.pop();
        *b = build_page(b, &records, 0);
    }, false);
    shape("A7 链指针记录之后多一行", &|b| {
        let mut records = records_of(b);
        let row = records[0].clone();
        records.push(row);
        *b = build_page(b, &records, 0);
    }, false);
    // 行本身：去掉被抛弃实例那一行、重复、换次序、flags 字节非 0、txg 改大
    let abandoned_instance = abandoned.0 .0;
    shape(&format!("A8 去掉实例{abandoned_instance}那一行"), &|b| {
        let records: Vec<Vec<u8>> = records_of(b).into_iter().filter(|r| !(r[0] == 0 && u32_at(r, 1) == abandoned_instance)).collect();
        *b = build_page(b, &records, 0);
    }, false);
    shape(&format!("A8 实例{abandoned_instance}那一行的txg改成被抛弃那一版的txg"), &|b| {
        let mut records = records_of(b);
        for r in &mut records {
            if r[0] == 0 && u32_at(r, 1) == abandoned_instance {
                r[5..13].copy_from_slice(&abandoned.1.to_le_bytes());
            }
        }
        *b = build_page(b, &records, 0);
    }, false);
    shape("A9 每一行重复一次", &|b| {
        let mut records = records_of(b);
        let chain = records.pop().expect("链");
        let mut doubled = Vec::new();
        for r in records {
            doubled.push(r.clone());
            doubled.push(r);
        }
        doubled.push(chain);
        *b = build_page(b, &doubled, 0);
    }, false);
    shape("A10 行倒序", &|b| {
        let mut records = records_of(b);
        let chain = records.pop().expect("链");
        records.reverse();
        records.push(chain);
        *b = build_page(b, &records, 0);
    }, false);
    shape("A11 每一行flags字节写1", &|b| {
        let mut records = records_of(b);
        for r in &mut records {
            if r[0] == 0 {
                r[21] = 1;
            }
        }
        *b = build_page(b, &records, 0);
    }, false);
    shape("A12 头里格式版本改成2", &|b| b[4] = 2, true);
    shape("A13 容器号写1", &|b| b[53] = 1, true);
    shape("A13 出生树写1", &|b| b[43] = 1, true);
    shape("A14 记录宽写89", &|b| b[71] = 89, true);
    shape("A14 声明长度多一条记录（尾巴补零）", &|b| {
        let declared = u16_at(b, 8) + RECORD as u16;
        b[8..10].copy_from_slice(&declared.to_le_bytes());
    }, true);
    shape("A14 声明长度多一条记录、尾巴写一行实例2txg0", &|b| {
        let count = usize::from(u16_at(b, 69));
        let declared = u16_at(b, 8) + RECORD as u16;
        b[8..10].copy_from_slice(&declared.to_le_bytes());
        let o = FIRST_RECORD + count * RECORD;
        b[o] = 0;
        b[o + 1..o + 5].copy_from_slice(&abandoned_instance.to_le_bytes());
    }, true);

    // 两片：第 0 片放行，第 1 片放 `split` 之后的行
    let no_change = |_: usize, _: &mut Vec<u8>, _: &mut Vec<u8>| {};
    let break_both = |k: usize, c0: &mut Vec<u8>, c1: &mut Vec<u8>| {
        if k == 1 {
            c0[IT_BYTES - 100] ^= 0xff;
            c1[IT_BYTES - 100] ^= 0xff;
        }
    };
    let break_dev0 = |k: usize, c0: &mut Vec<u8>, _c1: &mut Vec<u8>| {
        if k == 1 {
            c0[IT_BYTES - 100] ^= 0xff;
        }
    };
    let all = rows.clone();
    let (with_abandoned, without_abandoned): (Vec<Vec<u8>>, Vec<Vec<u8>>) =
        all.iter().cloned().partition(|r| u32_at(r, 1) == abandoned_instance);
    let two_page_cases: Vec<(&str, Vec<Vec<Vec<u8>>>)> = vec![
        ("P0 行全在第0片、第1片只有链指针记录", vec![all.clone(), vec![]]),
        ("P2 被抛弃实例那一行在第1片", vec![without_abandoned.clone(), with_abandoned.clone()]),
        ("P6 行全在第1片", vec![vec![], all.clone()]),
    ];
    for (name, pages) in &two_page_cases {
        for (suffix, change) in [
            ("第1片完好", &no_change as &dyn Fn(usize, &mut Vec<u8>, &mut Vec<u8>)),
            ("第1片两份都坏", &break_both),
            ("第1片盘0那份坏", &break_dev0),
        ] {
            let mut image = clean.clone();
            rebuild_newest_instance_table(&mut image, pages, change);
            report_a(&format!("{name} {suffix}"), &image);
        }
    }
    // 第 1 片的指针：位置条目第 0 条指着第 0 片自己（走读按它记「走过」），第 1 条指着真的第 1 片
    {
        let mut image = clean.clone();
        rebuild_newest_instance_table(&mut image, &[without_abandoned.clone(), with_abandoned.clone()], &no_change);
        let (slot0_now, page0_now) = newest_instance_table(&image);
        let mut records = records_of(&page0_now);
        let chain_index = records.len() - 1;
        records[chain_index][2 + 50 + 4..2 + 50 + 10].copy_from_slice(&slot0_now.to_le_bytes()[..6]);
        let rebuilt = build_page(&page0_now, &records, 0);
        replace_first_page(&mut image, slot0_now, &rebuilt);
        report_a("P4 第1片指针的盘0位置条目指回第0片（盘1那条对）", &image);
    }
    // 多片：257 片（第 256 片之后 GPU 的走读不再往下走）与 256 片，行只放第 0 片、去掉被抛弃实例那一行（C 进候选集，比就红）
    for (name, pages, rows0) in [
        ("P5 257片、去掉被抛弃实例那一行", 257usize, without_abandoned.clone()),
        ("P5 256片、去掉被抛弃实例那一行", 256, without_abandoned.clone()),
        ("P5 257片、行全在第0片", 257, all.clone()),
        ("P5 1片、去掉被抛弃实例那一行", 1, without_abandoned.clone()),
    ] {
        let mut image = clean.clone();
        let mut layout: Vec<Vec<Vec<u8>>> = vec![rows0.clone()];
        layout.extend((1..pages).map(|_| Vec::new()));
        rebuild_newest_instance_table(&mut image, &layout, &no_change);
        report_a(name, &image);
    }
}

// ================================================================ B 段：合法历史，用户动作放开扫

#[derive(Default)]
struct BTally {
    checks: u64,
    one_birth: BTreeMap<String, u64>,
    flags: BTreeMap<String, u64>,
    nine: BTreeMap<String, u64>,
    acts: BTreeMap<String, (u64, u64)>,
    other_reds: BTreeMap<String, u64>,
    /// 判的这一刻根环里还有被崩溃恢复抛弃的那一版、它上面有号的出生代与现行那一版不同（门要是放过它，I-9.29 就红）。
    exposed: u64,
    /// 最新根的实例表第 0 片说还有下一片（两片及以上）的判定次数。
    multi_page: u64,
    max_rows: u64,
}

struct Abandoned {
    instance: u32,
    txg: u64,
    births: BTreeMap<u64, u64>,
}

/// 最新根的实例表：第 0 片的行数、第 0 片说没说还有下一片。
fn first_page_shape(image: &MemoryPool) -> (u64, bool) {
    let (_, page0) = newest_instance_table(image);
    let records = records_of(&page0);
    let rows = records.iter().filter(|r| r[0] == 0).count() as u64;
    let has_next = records.last().map_or(false, |r| r[0] == 1 && r[1] == 1);
    (rows, has_next)
}

fn judge_b(image: &MemoryPool, knob: Option<u64>, pool: &SparsePool, abandoned: &[Abandoned], tally: &mut BTally) -> Vec<String> {
    let verdicts = verdicts_of(image, knob);
    tally.checks += 1;
    let mut flags = Vec::new();
    let tag = one_birth_tag(&verdicts);
    *tally.one_birth.entry(tag.clone()).or_insert(0) += 1;
    if tag != "成立" && !tag.starts_with("不适用") {
        flags.push(format!("I-9.29={tag}"));
    }
    // 「不适用」那句话按现行那一版的真值核（核法照抄 m3-batch24-code-r1 攻方原型的 `n3_judge`）。
    let queue_nonempty = !pool.queue().is_empty();
    let tombstone_containers = pool.output.tombstone_containers.len();
    let lineage_containers = pool.output.lineage_containers.len();
    let lineage_records: usize = pool.output.lineage_containers.iter().map(|c| c.contents.records.len()).sum();
    let records_now = pool.records();
    let tombstones: Vec<(u64, u64)> = pool
        .output
        .tombstone_containers
        .iter()
        .flat_map(|c| c.contents.records.iter())
        .map(|t| (t.object.0, t.object_birth.0))
        .collect();
    let tombstone_naming_a_record = tombstones.iter().any(|(n, b)| records_now.iter().any(|(rn, _, rb)| rn == n && rb == b));
    for name in NINE {
        let verdict = &verdicts.iter().find(|(n, _)| *n == name).expect("清单里有").1;
        let short = match verdict {
            InvariantVerdict::NotApplicable(reason) => format!("不适用[{}]", reason_tag(reason)),
            other => kind_of(other).to_string(),
        };
        if let InvariantVerdict::NotApplicable(reason) = verdict {
            let tag = reason_tag(reason);
            let false_claim = match (name, tag.as_str()) {
                ("I-9.18", "R-no-queue") if queue_nonempty => Some("现行那一版队列非空"),
                ("I-9.20", "R-no-tombstone") if tombstone_containers > 0 => Some("现行那一版墓碑树不是零根"),
                ("I-9.23", "R-no-tombstone-entry") if tombstone_containers > 0 => Some("现行那一版墓碑树不是零根"),
                ("I-9.17", "R-no-lineage-entry") if lineage_containers > 0 => Some("现行那一版谱系树不是零根"),
                ("I-9.21", "R-no-non-rightmost") if tombstone_containers > 1 => Some("现行那一版墓碑容器多于一片"),
                ("I-9.22", "R-no-container-under-knob") if tombstone_containers > 0 => Some("现行那一版墓碑树不是零根"),
                ("I-9.29", "R-no-number") if !records_now.is_empty() => Some("现行那一版有 inode 记录"),
                ("I-9.19", "R-no-tombstone-naming-record") if lineage_records == 0 && tombstone_naming_a_record => {
                    Some("没有谱系记录而现行那一版有墓碑指着同号同出生代的记录")
                }
                _ => None,
            };
            if let Some(why) = false_claim {
                flags.push(format!("FALSE-NA {name} [{tag}] {why}"));
            }
        }
        if short.contains("R-DEFAULT") {
            flags.push(format!("DEFAULT {name}"));
        }
        if short == "不成立" {
            flags.push(format!("RED {name}"));
        }
        if name == "I-9.24" && short.starts_with("不适用") {
            flags.push(format!("NA {name} {short}"));
        }
        *tally.nine.entry(format!("{name} {short}")).or_insert(0) += 1;
    }
    let others: Vec<&str> = violated_names(&verdicts).into_iter().filter(|n| !NINE.contains(n)).collect();
    if !others.is_empty() {
        *tally.other_reds.entry(format!("{others:?}")).or_insert(0) += 1;
    }
    let ring: BTreeSet<(u32, u64)> = ring_roots(image).iter().map(|r| (r.instance, r.txg)).collect();
    let current: BTreeMap<u64, u64> = pool.records().iter().map(|(inode, _, birth)| (*inode, *birth)).collect();
    if abandoned.iter().any(|a| {
        ring.contains(&(a.instance, a.txg))
            && a.births.iter().any(|(inode, birth)| current.get(inode).is_some_and(|b| b != birth))
    }) {
        tally.exposed += 1;
    }
    let (rows, has_next) = first_page_shape(image);
    tally.max_rows = tally.max_rows.max(rows);
    if has_next {
        tally.multi_page += 1;
    }
    flags
}

/// 一段随机历史。动作：建 1 / 2 个号、对一个活号发意图、删一批、走空、回退到前几新的一版（被抛弃的那几版也抽，该被拒）、再挂、
/// 崩溃恢复落到前一条根、先建一个号再崩溃恢复抛弃它（重发同号）、连着再挂 5 次。被拒的换回动作之前那一份，做成的判一次。
fn run_history_b(base: &SparsePool, base_versions: &[(InstanceGeneration, u64)], base_abandoned: &[Abandoned], knob: Option<u64>, seed: u64, steps: usize, tally: &mut BTally) -> String {
    let mut rng = SplitMix(seed);
    let mut pool = base.clone();
    let mut versions = base_versions.to_vec();
    let mut abandoned: Vec<Abandoned> = base_abandoned.iter().map(|a| Abandoned { instance: a.instance, txg: a.txg, births: a.births.clone() }).collect();
    let mut trace = Vec::new();
    let mut flagged = Vec::new();
    for step_index in 0..steps {
        let mut attempt = pool.clone();
        let choice = rng.below(100);
        let mut newly_abandoned: Option<Abandoned> = None;
        let (label, outcome) = if choice < 14 {
            let count = 1 + rng.below(2) as u64;
            (format!("create{count}"), attempt.create(count))
        } else if choice < 26 {
            let live: Vec<u64> = attempt.records().iter().filter(|(_, nlink, _)| *nlink >= 1).map(|(inode, _, _)| *inode).collect();
            if live.is_empty() {
                ("intent-none".to_string(), Err("没有活的号".to_string()))
            } else {
                let inode = live[rng.below(live.len())];
                (format!("intent{inode}"), attempt.intent(inode))
            }
        } else if choice < 34 {
            let outcome = if attempt.queue().is_empty() { Err("队列空".to_string()) } else { attempt.step() };
            ("step".to_string(), outcome)
        } else if choice < 40 {
            let outcome = if attempt.queue().is_empty() { Err("队列空".to_string()) } else { attempt.drain() };
            ("drain".to_string(), outcome)
        } else if choice < 60 {
            let current = attempt.mark();
            let choices: Vec<(InstanceGeneration, u64)> = versions.iter().copied().filter(|v| *v != current).collect();
            if choices.is_empty() {
                ("rollback-none".to_string(), Err("没有别的版".to_string()))
            } else {
                let back = 1 + rng.below(choices.len().min(12));
                let target = choices[choices.len() - back];
                (format!("rollback(i{},t{})", target.0 .0, target.1), attempt.rollback(target))
            }
        } else if choice < 72 {
            ("remount".to_string(), attempt.remount())
        } else if choice < 84 {
            let births: BTreeMap<u64, u64> = attempt.records().iter().map(|(i, _, b)| (*i, *b)).collect();
            let outcome = attempt.abandon_the_newest();
            let outcome = outcome.map(|(instance, txg)| {
                newly_abandoned = Some(Abandoned { instance: instance.0, txg, births });
            });
            ("abandon".to_string(), outcome)
        } else if choice < 96 {
            let mut outcome = attempt.create(1);
            let births: BTreeMap<u64, u64> = attempt.records().iter().map(|(i, _, b)| (*i, *b)).collect();
            if outcome.is_ok() {
                outcome = attempt.abandon_the_newest().map(|(instance, txg)| {
                    newly_abandoned = Some(Abandoned { instance: instance.0, txg, births });
                });
            }
            ("create-abandon".to_string(), outcome)
        } else {
            let mut outcome = Ok(());
            for _ in 0..5 {
                if outcome.is_ok() {
                    outcome = attempt.remount();
                }
            }
            ("remount5".to_string(), outcome)
        };
        let kind: String = label.chars().take_while(|c| c.is_ascii_lowercase() || *c == '-' || c.is_ascii_digit()).collect::<String>().trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        let entry = tally.acts.entry(kind).or_insert((0, 0));
        match outcome {
            Ok(()) => {
                entry.0 += 1;
                pool = attempt;
                versions.push(pool.mark());
                if let Some(a) = newly_abandoned {
                    versions.push((InstanceGeneration(a.instance), a.txg));
                    abandoned.push(a);
                }
                let flags = judge_b(&pool.image(), knob, &pool, &abandoned, tally);
                trace.push(format!("{label}->i{}t{}", pool.instance().0, pool.output.root.checkpoint_txg.0));
                for flag in &flags {
                    *tally.flags.entry(flag.clone()).or_insert(0) += 1;
                }
                if !flags.is_empty() {
                    flagged.push(format!("step{step_index}:{flags:?}"));
                }
            }
            Err(reason) => {
                entry.1 += 1;
                trace.push(format!("{label}:skip({})", reason.chars().take(30).collect::<String>()));
            }
        }
    }
    format!("RESULT B history seed={seed} flags={flagged:?} trace={}", trace.join(" "))
}

/// 起点。`S0`：新池新建文件 → 建 2、3 号；`SX`：A 段那一段（被抛弃的 C 与现行线同号不同出生代）；
/// `SR`：S0 之后实例表清扫交给后台、连着再挂 `OPUS_REMOUNTS` 次（默认 370，行数过 369 实例表长出第二片）；`SK`：S0 装删除开关（一片 1 条、节点 2 条）。
fn starting_point_b(which: &str) -> (SparsePool, Vec<(InstanceGeneration, u64)>, Vec<Abandoned>, Option<u64>) {
    match which {
        "S0" | "SR" | "SK" => {
            let mut pool = SparsePool::build();
            let mut versions = vec![pool.mark()];
            let mut knob = None;
            if which == "SK" {
                let switches = singlefs_core::unlink::DeletionSwitches {
                    batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
                    tombstone_container_capacity: TombstoneContainerCapacity::CappedForTests(
                        TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
                    ),
                    lineage_container_capacity: LineageContainerCapacity::CappedForTests(
                        LineageRecordsPerContainerCappedForTests::new(1).expect("1..=2039"),
                    ),
                    container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(
                        ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
                    ),
                };
                pool.switches = Some(switches);
                pool.allocator.set_deletion_switches(switches);
                knob = Some(1);
            }
            pool.create(2).expect("建 2、3 号");
            versions.push(pool.mark());
            if which == "SR" {
                pool.background_sweep = true;
                let remounts = environment_number("OPUS_REMOUNTS", 370);
                for _ in 0..remounts {
                    pool.remount().expect("再挂");
                }
                versions.push(pool.mark());
            }
            (pool, versions, Vec::new(), knob)
        }
        "SX" => {
            let (pool, abandoned, on_c, _) = pool_x();
            let births = on_c.iter().map(|(i, _, b)| (*i, *b)).collect();
            let versions = vec![pool.mark()];
            (pool, versions, vec![Abandoned { instance: abandoned.0 .0, txg: abandoned.1, births }], None)
        }
        other => panic!("不认得的起点 {other}"),
    }
}

#[test]
fn b_one_birth_gate_on_legal_histories_with_the_user_steps_open() {
    let which = std::env::var("OPUS_START").unwrap_or_else(|_| "S0".to_string());
    let seed_from = environment_number("OPUS_SEED_FROM", 1);
    let histories = environment_number("OPUS_HISTORIES", 2);
    let steps = environment_number("OPUS_STEPS", 6) as usize;
    let started = std::time::Instant::now();
    let (base, versions, abandoned, knob) = starting_point_b(&which);
    let mut tally = BTally::default();
    let start_flags = judge_b(&base.image(), knob, &base, &abandoned, &mut tally);
    println!("RESULT B start {which} flags={start_flags:?} first_page={:?}", first_page_shape(&base.image()));
    eprintln!("TIMING start {which} seconds={:.1}", started.elapsed().as_secs_f64());
    for seed in seed_from..seed_from + histories {
        let line = run_history_b(&base, &versions, &abandoned, knob, seed, steps, &mut tally);
        println!("{line}");
    }
    println!("RESULT B tally {which} checks={} exposed={} multi_page={} max_rows_first_page={}", tally.checks, tally.exposed, tally.multi_page, tally.max_rows);
    for (key, count) in &tally.one_birth {
        println!("RESULT B tally {which} I-9.29 {key} {count}");
    }
    for (key, count) in &tally.flags {
        println!("RESULT B tally {which} flag {key} {count}");
    }
    for (key, count) in &tally.nine {
        println!("RESULT B tally {which} nine {key} {count}");
    }
    for (key, (done, refused)) in &tally.acts {
        println!("RESULT B tally {which} act {key} done={done} refused={refused}");
    }
    for (key, count) in &tally.other_reds {
        println!("RESULT B tally {which} other-red {key} {count}");
    }
    eprintln!("TIMING {which} seconds={:.1}", started.elapsed().as_secs_f64());
}

// ================================================================ C 段：m3-batch24-code-r1 N1 单故障第二次抽样（换一组形状）
// 树的走读、条目改写、起点池的造法照抄 `research/prompts/m3-batch24-code-r1-opus-model/opus_b24_attack.rs`；
// 换掉的是取样：类型段取它没取过的值（3、4、6、8、9、10、11、12、255、256、32768），另加条目身份三段各单独改一处、
// 起点换成它没用过的几张（墓碑一片 1 条删 4 个、一片 2 条删 5 个、谱系回退 4 次、inode 树 400 个号）。

const NODE: usize = 16384;
const ENTRY: usize = 120;

fn read_device(image: &MemoryPool, device: u32, slot: u64, length: usize) -> Vec<u8> {
    image.devices.get(&DeviceIdentity(device)).expect("盘在池里").read(SlotNumber(slot).to_device_offset(), length)
}

fn entry_type_at(entry: &[u8]) -> u16 {
    u16::from_le_bytes([entry[16], entry[17]])
}

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

fn change_an_entry_of_the_node(image: &mut MemoryPool, slot: u64, index: usize, change: impl FnOnce(&mut [u8])) {
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(image, SlotNumber(slot), NODE, |unit| {
        let key_width = usize::from(unit[51]);
        let start = 115 + 2 * key_width + index * ENTRY;
        change(&mut unit[start..start + ENTRY]);
    });
}

fn report_shape(name: &str, image: &MemoryPool, knob: Option<u64>, target: &str, tally: &mut BTreeMap<String, u64>) {
    let verdicts = verdicts_of(image, knob);
    let kind = verdicts.iter().find(|(n, _)| *n == target).map(|(_, v)| kind_of(v)).unwrap_or("?");
    let extra: Vec<&'static str> = violated_names(&verdicts).into_iter().filter(|n| *n != target).collect();
    let partial: Vec<&'static str> = verdicts
        .iter()
        .filter(|(n, v)| matches!(v, InvariantVerdict::PartiallyJudged(_)) && NINE.contains(n))
        .map(|(n, _)| *n)
        .collect();
    let hit = kind != "不成立" || !extra.is_empty();
    *tally.entry(if hit { "HIT" } else { "ok" }.to_string()).or_insert(0) += 1;
    println!("RESULT C shape {} {name} target={target}:{kind} extra={extra:?} partial_nine={partial:?}", if hit { "HIT" } else { "ok" });
    if hit {
        for (invariant, verdict) in &verdicts {
            if let InvariantVerdict::Violated(d) = verdict {
                println!("DETAIL C {name} {invariant}: {}", d.chars().take(300).collect::<String>());
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

/// 旋钮（一片 `per_container` 条、一个节点 2 条）下建 `created` 个空号、删掉 2..2+`deleted`。
fn tombstone_pool(base: &SparsePool, per_container: u64, created: u64, deleted: u64) -> SparsePool {
    let mut pool = base.clone();
    pool.capacities = Some(knob_capacities(per_container, 2));
    pool.create(created).expect("建号");
    for inode in 2..2 + deleted {
        pool.intent(inode).expect("意图");
        pool.drain().expect("删完");
    }
    pool
}

/// 旋钮下回退 `rollbacks` 次：每次删 2 号、回退到这一次删之前。
fn lineage_pool(base: &SparsePool, rollbacks: usize) -> SparsePool {
    let mut pool = base.clone();
    pool.capacities = Some(knob_capacities(1, 2));
    let switches = knob_switches(1);
    pool.switches = Some(switches);
    pool.allocator.set_deletion_switches(switches);
    pool.create(1).expect("建 2 号");
    for _ in 0..rollbacks {
        let before = pool.mark();
        pool.intent(2).expect("2 号意图");
        pool.drain().expect("删完");
        pool.rollback(before).expect("回退到意图之前");
    }
    pool
}

const TYPE_VALUES_OF_THE_SECOND_SAMPLE: [u16; 11] = [3, 4, 6, 8, 9, 10, 11, 12, 255, 256, 32768];

fn second_sample_matrix(pool_name: &str, clean: &MemoryPool, knob: Option<u64>, kind: u16, target: &str, tally: &mut BTreeMap<String, u64>) {
    let Some(root) = root_slot_of_kind(clean, kind) else {
        println!("RESULT C pool {pool_name} kind={kind} 零根");
        return;
    };
    println!("RESULT C pool {pool_name} kind={kind} clean_violated={:?}", violated_names(&verdicts_of(clean, knob)));
    let nodes = nodes_of_the_tree(clean, root);
    for node in &nodes {
        for (index, entry) in node.entries.iter().enumerate() {
            let from = entry_type_at(entry);
            for to in TYPE_VALUES_OF_THE_SECOND_SAMPLE {
                if to == from {
                    continue;
                }
                let mut image = clean.clone();
                change_an_entry_of_the_node(&mut image, node.slot, index, |e| e[16..18].copy_from_slice(&to.to_le_bytes()));
                let name = format!("{pool_name} type depth={} level={} entry={index}/{} {from}->{to}", node.depth, node.level, node.entries.len());
                report_shape(&name, &image, knob, target, tally);
            }
            for (field, offset) in [("birth-tree", 8usize), ("container", 18), ("container-birth", 26)] {
                let mut image = clean.clone();
                change_an_entry_of_the_node(&mut image, node.slot, index, |e| {
                    let value = u64::from_le_bytes(e[offset..offset + 8].try_into().expect("8")).wrapping_add(1);
                    e[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
                });
                let name = format!("{pool_name} identity depth={} level={} entry={index}/{} type={from} {field}+1", node.depth, node.level, node.entries.len());
                report_shape(&name, &image, knob, target, tally);
            }
        }
    }
}

#[test]
fn c_second_sample_of_the_entry_type_and_identity_single_faults() {
    let which = std::env::var("OPUS_POOL").unwrap_or_else(|_| "T2b".to_string());
    let base = SparsePool::build();
    let mut tally = BTreeMap::new();
    let started = std::time::Instant::now();
    let (pool, kind, target, knob) = match which.as_str() {
        "T2b" => (tombstone_pool(&base, 1, 6, 4), TREE_KIND_TOMBSTONE, "I-9.23", Some(1)),
        "T2k2" => (tombstone_pool(&base, 2, 7, 5), TREE_KIND_TOMBSTONE, "I-9.23", Some(2)),
        "L2b" => (lineage_pool(&base, 4), TREE_KIND_LINEAGE, "I-9.17", Some(1)),
        "I0b" => {
            let mut pool = base.clone();
            for _ in 0..4 {
                pool.create(100).expect("建 100 个号");
            }
            (pool, TREE_KIND_INODE, "I-9.2", None)
        }
        other => panic!("不认得的起点 {other}"),
    };
    let clean = pool.image();
    second_sample_matrix(&which, &clean, knob, kind, target, &mut tally);
    for (key, count) in &tally {
        println!("RESULT C tally {which} {key} {count}");
    }
    eprintln!("TIMING C {which} seconds={:.1}", started.elapsed().as_secs_f64());
}
