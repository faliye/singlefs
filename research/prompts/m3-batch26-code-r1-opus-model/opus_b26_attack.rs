//! m3-batch26-code-r1 攻方原型（只在仓副本里编、跑；不入库代码）。
//! 基础设施（`SparsePool`、用户动作、实例表改造、码 2 树走读与条目改写、合法历史随机走）照抄自
//! `research/prompts/m3-batch25-code-r1-opus-model/opus_b25_attack.rs`（它又抄自 m3-batch24 / m3-batch22 的攻方原型），
//! 判的东西是这一轮自己的：
//!   H 段：孩子两份副本的状态（真身 / 码 2 陈旧 / 码 3 陈旧 / 全零）× 条目改法 × 树与层级，新旧两种写法各判一遍；
//!   V 段：两份副本按各自位置条目的校验和都自证得过、却是不同的两类；
//!   U 段：孩子两份都读不出时 I-9.2 / I-9.17 / I-9.23 判什么（与读得出时逐形对照）；
//!   A2 段：m3-batch25-code-r1 K2「走到末片而没滤全」第二次抽样（换一组实例表形状）；
//!   B 段：合法历史上用户动作放开扫（换种子、加 inode 多叶起点），看 I-9.2 / I-9.17 / I-9.23、I-4.8 / I-7.4 / I-7.2 与 I-9.29。
//! 盘用内存稀疏盘（`SparseBlockDevice`），起点池建一次、之后每形从内存里那一份拷。输出一律是 `RESULT ` 打头的行。
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
    flags.extend(extra_flags(&verdicts));
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


// ================================================================ 码 2 树走读与条目改写（照抄 m3-batch25 攻方原型 C 段的帮手）
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

// ================================================================ 这一轮自己的部分：H / V / U 段（孩子两份副本 × 条目改法）

use singlefs_core::block_device::BlockDevice as _;

fn code_label() -> String {
    std::env::var("OPUS_CODE").unwrap_or_else(|_| "?".to_string())
}

/// 种类 → 内部条目「孩子是容器」那一个类型段（inode 2、墓碑 1、谱系 5）。
fn record_type_of_kind(kind: u16) -> u16 {
    match kind {
        TREE_KIND_INODE => 2,
        TREE_KIND_TOMBSTONE => 1,
        TREE_KIND_LINEAGE => 5,
        other => panic!("不认得的种类 {other}"),
    }
}

/// 一张起点镜像：名字 → (干净镜像, 种类, 目标不变量, 旋钮)。I2 是手造的层级 2 inode 树（合法流里 inode 树不长高，`inode_tree.rs` 拒）。
fn clean_image_named(base: &SparsePool, which: &str) -> (MemoryPool, u16, &'static str, Option<u64>) {
    match which {
        "T1" => (tombstone_pool(base, 1, 4, 2).image(), TREE_KIND_TOMBSTONE, "I-9.23", Some(1)),
        "T2" => (tombstone_pool(base, 1, 5, 3).image(), TREE_KIND_TOMBSTONE, "I-9.23", Some(1)),
        "T3" => (tombstone_pool(base, 1, 7, 5).image(), TREE_KIND_TOMBSTONE, "I-9.23", Some(1)),
        "L1" => (lineage_pool(base, 2).image(), TREE_KIND_LINEAGE, "I-9.17", Some(1)),
        "L2" => (lineage_pool(base, 3).image(), TREE_KIND_LINEAGE, "I-9.17", Some(1)),
        "I0" => (inode_pool(base).image(), TREE_KIND_INODE, "I-9.2", None),
        "I0r" => (inode_pool_after_a_rollback(base).image(), TREE_KIND_INODE, "I-9.2", None),
        "I2" => (handcraft_level_two_inode_root(&inode_pool(base).image()), TREE_KIND_INODE, "I-9.2", None),
        other => panic!("不认得的起点 {other}"),
    }
}

/// 新池新建文件之后建 300 个号：inode 树根层级 1、两片叶。
fn inode_pool(base: &SparsePool) -> SparsePool {
    let mut pool = base.clone();
    pool.create(150).expect("建 150 个号");
    pool.create(150).expect("再建 150 个号");
    pool
}

/// inode_pool 之后对 5 号发意图、删完、回退到意图之前（一次向前发布）：回退那一版与它之前几版共享叶。
fn inode_pool_after_a_rollback(base: &SparsePool) -> SparsePool {
    let mut pool = inode_pool(base);
    let before = pool.mark();
    pool.intent(5).expect("5 号意图");
    pool.drain().expect("删完");
    pool.rollback(before).expect("回退到意图之前");
    pool
}

/// 手造层级 2 的 inode 树：原来的层级 1 根原样搬到槽 180 000（两盘同槽），原根槽写一个层级 2 的码 2 根，
/// 唯一一条条目类型段 0、身份三段 0、分隔 key = 原根的最小 key（头里的 key 区间两端都写它），子指针 = 树表里原来的根指针换两条位置条目的槽；整单元校验和沿引用链补到根槽。
const LEVEL_ONE_INODE_NODE_SLOT: u64 = 180_000;

fn handcraft_level_two_inode_root(clean: &MemoryPool) -> MemoryPool {
    let mut image = clean.clone();
    let entry = common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE);
    let root_slot = common_pointer_head_corruption::the_slot_of(&entry.root);
    let r1 = read_device(&image, 0, root_slot.0, NODE);
    let header = singlefs_core::unit::parse_index_node(&r1).expect("原根解得开");
    assert_eq!(header.level, 1, "原根层级 1");
    wr(&mut image, 0, LEVEL_ONE_INODE_NODE_SLOT * SLOT, &r1);
    wr(&mut image, 1, LEVEL_ONE_INODE_NODE_SLOT * SLOT, &r1);
    let mut pointer = entry.root;
    pointer.locations[0].slot = SlotNumber(LEVEL_ONE_INODE_NODE_SLOT);
    pointer.locations[1].slot = SlotNumber(LEVEL_ONE_INODE_NODE_SLOT);
    let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&pointer);
    let mut e = vec![0u8; ENTRY];
    e[0..header.key_width].copy_from_slice(&header.smallest_key);
    e[34..120].copy_from_slice(&pointer_bytes);
    let r2 = singlefs_core::unit::build_index_node(
        header.tree,
        2,
        header.key_width,
        &header.smallest_key,
        &header.smallest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        ENTRY as u16,
        &[e],
    );
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(&mut image, root_slot, &r2);
    image
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CopyState {
    /// 孩子原样。
    R,
    /// 前 16 KiB 换成别处一个码 2 节点的原样字节（这棵树的根）。
    S2,
    /// 前 16 KiB 换成别处一个码 3 容器的前 16 KiB（最新根的实例表第 0 片）。
    S3,
    /// 前 16 KiB 写零（类标签 0，不在登记表里）。
    Z,
}
const COPY_STATES: [CopyState; 4] = [CopyState::R, CopyState::S2, CopyState::S3, CopyState::Z];

struct StaleSources {
    stale2: Vec<u8>,
    stale3: Vec<u8>,
}

fn stale_sources(clean: &MemoryPool, root_slot: u64) -> StaleSources {
    let stale2 = read_device(clean, 0, root_slot, NODE);
    assert_eq!(stale2[6], 2);
    let (_, page0) = newest_instance_table(clean);
    assert_eq!(page0[6], 3);
    StaleSources { stale2, stale3: page0[..NODE].to_vec() }
}

fn put_copy(image: &mut MemoryPool, device: u32, slot: u64, state: CopyState, sources: &StaleSources) {
    match state {
        CopyState::R => {}
        CopyState::S2 => wr(image, device, slot * SLOT, &sources.stale2),
        CopyState::S3 => wr(image, device, slot * SLOT, &sources.stale3),
        CopyState::Z => wr(image, device, slot * SLOT, &vec![0u8; NODE]),
    }
}

/// 判一张镜像：目标那条的判定（短串）与违例集合。
fn target_and_reds(image: &MemoryPool, knob: Option<u64>, target: &str) -> (String, BTreeSet<&'static str>) {
    let verdicts = verdicts_of(image, knob);
    let kind = verdicts.iter().find(|(n, _)| *n == target).map(|(_, v)| kind_of(v)).unwrap_or("?").to_string();
    (kind, violated_names(&verdicts).into_iter().collect())
}

/// 要判的条目：每个节点的第 0 条与最后一条。
fn chosen_entries(nodes: &[NodeAt]) -> Vec<(NodeAt, usize)> {
    let mut out = Vec::new();
    for node in nodes {
        let mut indexes = vec![0usize, node.entries.len() - 1];
        indexes.dedup();
        for index in indexes {
            out.push((node.clone(), index));
        }
    }
    out
}

fn entry_key(pool: &str, node: &NodeAt, index: usize) -> String {
    format!("{pool} d{} L{} e{index}/{} t{}", node.depth, node.level, node.entries.len(), entry_type_at(&node.entries[index]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EntryMod {
    /// 条目不改。
    Keep,
    /// 类型段换成另一个登记过的值：容器类型段 → 0 并清身份三段；0 → 容器类型段（身份照旧是 0）。
    Flip,
    /// 类型段写 7（不在登记表里）。
    Unreg,
}

fn apply_entry_mod(image: &mut MemoryPool, node: &NodeAt, index: usize, m: EntryMod, record_type: u16) {
    let entry_type = entry_type_at(&node.entries[index]);
    match m {
        EntryMod::Keep => {}
        EntryMod::Flip => change_an_entry_of_the_node(image, node.slot, index, |e| {
            if entry_type == 0 {
                e[16..18].copy_from_slice(&record_type.to_le_bytes());
            } else {
                e[16..18].copy_from_slice(&0u16.to_le_bytes());
                e[8..16].fill(0);
                e[18..34].fill(0);
            }
        }),
        EntryMod::Unreg => change_an_entry_of_the_node(image, node.slot, index, |e| e[16..18].copy_from_slice(&7u16.to_le_bytes())),
    }
}

/// 照条款字面、两份都不是真身（读不出）时目标该是什么：类型段不认得 ⇒ 红；类型段 0 ⇒ 身份三段非零才红；容器类型段 ⇒ 比孩子头的那几小句判不了。
fn expected_red_without_a_readable_child(entry_after: &[u8], record_type: u16) -> bool {
    let t = entry_type_at(entry_after);
    let birth_tree = u64::from_le_bytes(entry_after[8..16].try_into().expect("8"));
    let container = u64::from_le_bytes(entry_after[18..26].try_into().expect("8"));
    let birth = u64::from_le_bytes(entry_after[26..34].try_into().expect("8"));
    let pointer_birth_tree = u64::from_le_bytes(entry_after[68..76].try_into().expect("8"));
    if t == 0 {
        birth_tree != 0 || container != 0 || birth != 0
    } else if t == record_type {
        pointer_birth_tree != birth_tree
    } else {
        true
    }
}

fn entry_after_mod(entry: &[u8], m: EntryMod, record_type: u16) -> Vec<u8> {
    let mut e = entry.to_vec();
    let entry_type = entry_type_at(entry);
    match m {
        EntryMod::Keep => {}
        EntryMod::Flip => {
            if entry_type == 0 {
                e[16..18].copy_from_slice(&record_type.to_le_bytes());
            } else {
                e[16..18].copy_from_slice(&0u16.to_le_bytes());
                e[8..16].fill(0);
                e[18..34].fill(0);
            }
        }
        EntryMod::Unreg => e[16..18].copy_from_slice(&7u16.to_le_bytes()),
    }
    e
}

/// H 段：一张起点（`OPUS_POOL`）上，每个节点第 0 条与最后一条条目 × 条目改法三种 × 孩子两份副本状态 4 × 4。
/// 每形一行 `RESULT H`；之后逐对比盘序（(a, b) 与 (b, a)）、比「只有副本状态、条目不改」那张对照 ∪ 目标，与条款字面期望比目标那一条。
#[test]
fn h_copy_states_by_entry_changes() {
    let which = std::env::var("OPUS_POOL").unwrap_or_else(|_| "T1".to_string());
    let code = code_label();
    let started = std::time::Instant::now();
    let base = SparsePool::build();
    let (clean, kind, target, knob) = clean_image_named(&base, &which);
    let record_type = record_type_of_kind(kind);
    let root = root_slot_of_kind(&clean, kind).expect("树有根");
    let sources = stale_sources(&clean, root);
    let nodes = nodes_of_the_tree(&clean, root);
    let (clean_kind, clean_reds) = target_and_reds(&clean, knob, target);
    println!("RESULT H {code} pool {which} clean target={target}:{clean_kind} red={clean_reds:?} nodes={}", nodes.len());
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for (node, index) in chosen_entries(&nodes) {
        let key = entry_key(&which, &node, index);
        let entry = node.entries[index].clone();
        let child = parse_node_pointer(&entry[34..120]);
        let child_slot = child.locations[0].slot;
        let real_class = read_device(&clean, 0, child_slot, NODE)[6];
        let mut results: BTreeMap<(EntryMod, CopyState, CopyState), (String, BTreeSet<&'static str>)> = BTreeMap::new();
        for m in [EntryMod::Keep, EntryMod::Flip, EntryMod::Unreg] {
            let mut changed = clean.clone();
            apply_entry_mod(&mut changed, &node, index, m, record_type);
            for c0 in COPY_STATES {
                for c1 in COPY_STATES {
                    let mut image = changed.clone();
                    put_copy(&mut image, 0, child_slot, c0, &sources);
                    put_copy(&mut image, 1, child_slot, c1, &sources);
                    let (kind_of_target, reds) = target_and_reds(&image, knob, target);
                    println!("RESULT H {code} {key} real{real_class} {m:?} c0={c0:?} c1={c1:?} target={kind_of_target} red={reds:?}");
                    results.insert((m, c0, c1), (kind_of_target, reds));
                }
            }
        }
        for m in [EntryMod::Keep, EntryMod::Flip, EntryMod::Unreg] {
            let after = entry_after_mod(&entry, m, record_type);
            for c0 in COPY_STATES {
                for c1 in COPY_STATES {
                    let (kind_here, reds_here) = &results[&(m, c0, c1)];
                    // 盘序：(c0, c1) 与 (c1, c0) 只差哪一份在盘 0。
                    if c0 < c1 {
                        let (kind_swapped, reds_swapped) = &results[&(m, c1, c0)];
                        let same = kind_here == kind_swapped && reds_here == reds_swapped;
                        *tally.entry(format!("order {}", if same { "same" } else { "FLIP" })).or_insert(0) += 1;
                        if !same {
                            println!(
                                "RESULT H-order {code} FLIP {key} {m:?} {c0:?}/{c1:?} target={kind_here}/{kind_swapped} red={reds_here:?} swapped={reds_swapped:?}"
                            );
                        }
                    }
                    // 目标那一条照字面：有一份真身时条目改了就红、没改不红；两份都不是真身时按读不出那几小句。
                    let any_real = c0 == CopyState::R || c1 == CopyState::R;
                    let expected_red = if any_real { m != EntryMod::Keep } else { expected_red_without_a_readable_child(&after, record_type) };
                    let target_ok = (kind_here == "不成立") == expected_red;
                    *tally.entry(format!("target {}", if target_ok { "ok" } else { "MISS" })).or_insert(0) += 1;
                    if !target_ok {
                        println!("RESULT H-target {code} MISS {key} {m:?} c0={c0:?} c1={c1:?} target={kind_here} expected_red={expected_red}");
                    }
                    // 违例集合 = 只有副本状态（条目不改）那张对照 ∪ 目标：多了的、少了的都列。
                    if m != EntryMod::Keep {
                        let (_, control) = &results[&(EntryMod::Keep, c0, c1)];
                        let mut expected: BTreeSet<&'static str> = control.iter().copied().filter(|n| *n != target).collect();
                        if expected_red {
                            expected.insert(target);
                        }
                        let missing: Vec<&str> = expected.difference(reds_here).copied().collect();
                        let extra: Vec<&str> = reds_here.difference(&expected).copied().collect();
                        let same = missing.is_empty() && extra.is_empty();
                        *tally.entry(format!("set {}", if same { "same" } else { "DIFF" })).or_insert(0) += 1;
                        if !same {
                            println!("RESULT H-set {code} DIFF {key} {m:?} c0={c0:?} c1={c1:?} missing={missing:?} extra={extra:?}");
                        }
                    }
                }
            }
        }
    }
    for (k, v) in &tally {
        println!("RESULT H-tally {code} {which} {k} {v}");
    }
    eprintln!("TIMING H {code} {which} seconds={:.1}", started.elapsed().as_secs_f64());
}

/// V 段：孩子是码 3 容器的条目，一条位置条目的整单元校验和改成别处一个码 2 节点（这棵树的根）的 CRC、那块盘上孩子的槽写那个节点：
/// 两份按各自的位置条目都自证得过、却是两类。条目不改 / 改成类型段 0（清身份）两种；陈旧的那份在盘 0 / 盘 1 两种。
#[test]
fn v_both_copies_verify_as_different_classes() {
    let code = code_label();
    let base = SparsePool::build();
    for which in ["T1", "L1", "I0", "T2"] {
        let (clean, kind, target, knob) = clean_image_named(&base, which);
        let record_type = record_type_of_kind(kind);
        let root = root_slot_of_kind(&clean, kind).expect("树有根");
        let sources = stale_sources(&clean, root);
        let nodes = nodes_of_the_tree(&clean, root);
        for (node, index) in chosen_entries(&nodes) {
            let entry = node.entries[index].clone();
            if entry_type_at(&entry) != record_type {
                continue;
            }
            let key = entry_key(which, &node, index);
            let child_slot = parse_node_pointer(&entry[34..120]).locations[0].slot;
            let stale_crc = singlefs_core::checksum::crc32_castagnoli(&sources.stale2);
            for m in [EntryMod::Keep, EntryMod::Flip] {
                for device in [0u32, 1] {
                    let mut image = clean.clone();
                    let checksum_at = if device == 0 { 34 + 60 } else { 34 + 74 };
                    change_an_entry_of_the_node(&mut image, node.slot, index, |e| {
                        e[checksum_at..checksum_at + 4].copy_from_slice(&stale_crc.to_le_bytes());
                        if m == EntryMod::Flip {
                            e[16..18].copy_from_slice(&0u16.to_le_bytes());
                            e[8..16].fill(0);
                            e[18..34].fill(0);
                        }
                    });
                    wr(&mut image, device, child_slot * SLOT, &sources.stale2);
                    let (kind_of_target, reds) = target_and_reds(&image, knob, target);
                    println!("RESULT V {code} {key} {m:?} verifying-node-on-dev{device} target={kind_of_target} red={reds:?}");
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum UnreadableMod {
    Keep,
    /// 类型段 0、身份三段清零。
    T0Z,
    /// 类型段 0、身份三段不动（原来就是 0 的写容器号 1）。
    T0N,
    U7,
    /// 子指针头部的出生树 + 1。
    BT,
    /// 身份引用的出生树 + 1。
    IDB,
    /// 身份引用的容器号 + 1。
    IDC,
}

fn apply_unreadable_mod(e: &mut [u8], m: UnreadableMod) {
    let bump = |e: &mut [u8], at: usize| {
        let value = u64::from_le_bytes(e[at..at + 8].try_into().expect("8")).wrapping_add(1);
        e[at..at + 8].copy_from_slice(&value.to_le_bytes());
    };
    match m {
        UnreadableMod::Keep => {}
        UnreadableMod::T0Z => {
            e[16..18].copy_from_slice(&0u16.to_le_bytes());
            e[8..16].fill(0);
            e[18..34].fill(0);
        }
        UnreadableMod::T0N => {
            let was_zero = entry_type_at(e) == 0;
            e[16..18].copy_from_slice(&0u16.to_le_bytes());
            if was_zero {
                e[18..26].copy_from_slice(&1u64.to_le_bytes());
            }
        }
        UnreadableMod::U7 => e[16..18].copy_from_slice(&7u16.to_le_bytes()),
        UnreadableMod::BT => bump(e, 68),
        UnreadableMod::IDB => bump(e, 8),
        UnreadableMod::IDC => bump(e, 18),
    }
}

/// U 段：每条选中的条目 × 七种改法 × 孩子 (真, 真) / (零, 真) / (真, 零) / (零, 零)。比「读得出」与「两份都读不出」时目标那一条，
/// 两份都读不出时再与条款字面里不看孩子就判得了的那几小句比（类型段、类型段 0 的身份三段、容器类型段的「子指针出生树 == 身份出生树」）。
#[test]
fn u_unreadable_child_against_the_literal_clauses() {
    let which = std::env::var("OPUS_POOL").unwrap_or_else(|_| "I0".to_string());
    let code = code_label();
    let started = std::time::Instant::now();
    let base = SparsePool::build();
    let (clean, kind, target, knob) = clean_image_named(&base, &which);
    let record_type = record_type_of_kind(kind);
    let root = root_slot_of_kind(&clean, kind).expect("树有根");
    let sources = stale_sources(&clean, root);
    let nodes = nodes_of_the_tree(&clean, root);
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    let mods = [UnreadableMod::Keep, UnreadableMod::T0Z, UnreadableMod::T0N, UnreadableMod::U7, UnreadableMod::BT, UnreadableMod::IDB, UnreadableMod::IDC];
    for (node, index) in chosen_entries(&nodes) {
        let key = entry_key(&which, &node, index);
        let entry = node.entries[index].clone();
        let child_slot = parse_node_pointer(&entry[34..120]).locations[0].slot;
        for m in mods {
            let mut after = entry.clone();
            apply_unreadable_mod(&mut after, m);
            if m != UnreadableMod::Keep && after == entry {
                continue;
            }
            let mut changed = clean.clone();
            change_an_entry_of_the_node(&mut changed, node.slot, index, |e| apply_unreadable_mod(e, m));
            let mut kinds = Vec::new();
            for (c0, c1) in [(CopyState::R, CopyState::R), (CopyState::Z, CopyState::R), (CopyState::R, CopyState::Z), (CopyState::Z, CopyState::Z)] {
                let mut image = changed.clone();
                put_copy(&mut image, 0, child_slot, c0, &sources);
                put_copy(&mut image, 1, child_slot, c1, &sources);
                let (kind_of_target, reds) = target_and_reds(&image, knob, target);
                println!("RESULT U {code} {key} {m:?} c0={c0:?} c1={c1:?} target={kind_of_target} red={reds:?}");
                kinds.push(kind_of_target);
            }
            let expected_unreadable = expected_red_without_a_readable_child(&after, record_type);
            let unreadable_red = kinds[3] == "不成立";
            let mark = match (expected_unreadable, unreadable_red) {
                (true, false) => "HIT-漏判",
                (false, true) => "HIT-误红",
                _ => "ok",
            };
            *tally.entry(format!("{mark}")).or_insert(0) += 1;
            println!(
                "RESULT U-cmp {code} {mark} {key} {m:?} readable={} one-copy-unreadable={}/{} both-unreadable={} literal-without-child={}",
                kinds[0], kinds[1], kinds[2], kinds[3], if expected_unreadable { "红" } else { "不红" }
            );
        }
    }
    for (k, v) in &tally {
        println!("RESULT U-tally {code} {which} {k} {v}");
    }
    eprintln!("TIMING U {code} {which} seconds={:.1}", started.elapsed().as_secs_f64());
}

// ================================================================ A2 段：m3-batch25-code-r1 K2「走到末片而没滤全」第二次抽样（新形状）

fn row_with(instance: u32, txg: u64, template: &[u8]) -> Vec<u8> {
    let mut row = template.to_vec();
    row[1..5].copy_from_slice(&instance.to_le_bytes());
    row[5..13].copy_from_slice(&txg.to_le_bytes());
    row
}

#[test]
fn a2_second_sample_of_the_gate_shapes() {
    let (pool, abandoned, on_c, on_current) = pool_x();
    let clean = pool.image();
    let rows = rows_of_the_newest_table(&clean);
    let abandoned_instance = abandoned.0 .0;
    println!(
        "RESULT A2 pool-x abandoned=(i{},t{}) rows={} on_C={on_c:?} on_current={on_current:?}",
        abandoned_instance,
        abandoned.1,
        row_summary(&rows)
    );
    let (with_abandoned, without_abandoned): (Vec<Vec<u8>>, Vec<Vec<u8>>) =
        rows.iter().cloned().partition(|r| u32_at(r, 1) == abandoned_instance);
    let abandoned_row = with_abandoned[0].clone();
    let no_change = |_: usize, _: &mut Vec<u8>, _: &mut Vec<u8>| {};
    let break_copy = |page: usize, device: u32| {
        move |k: usize, c0: &mut Vec<u8>, c1: &mut Vec<u8>| {
            if k == page {
                let copy = if device == 0 { c0 } else { c1 };
                copy[IT_BYTES - 100] ^= 0xff;
            }
        }
    };
    let layouts: Vec<(&str, Vec<Vec<Vec<u8>>>)> = vec![
        ("S1 三片、被抛弃实例那一行在第2片", vec![without_abandoned.clone(), vec![], with_abandoned.clone()]),
        ("S9 三片、第1片无行、其余行在第2片", vec![vec![], vec![], rows.clone()]),
        ("S5 被抛弃实例那一行在第0片txg低一份、第2片txg抬到被抛弃那一版一份", vec![
            rows.clone(),
            vec![],
            vec![row_with(abandoned_instance, abandoned.1, &abandoned_row)],
        ]),
        ("S5b 被抛弃实例那一行在第0片txg抬到被抛弃那一版、第2片原样一份", vec![
            without_abandoned.iter().cloned().chain([row_with(abandoned_instance, abandoned.1, &abandoned_row)]).collect(),
            vec![],
            with_abandoned.clone(),
        ]),
    ];
    for (name, pages) in &layouts {
        let mut image = clean.clone();
        rebuild_newest_instance_table(&mut image, pages, &no_change);
        report_a(&format!("A2 {name} 完好"), &image);
    }
    for (suffix, page, device) in [("第1片盘0那份坏", 1usize, 0u32), ("第2片盘1那份坏", 2, 1), ("第2片盘0那份坏", 2, 0)] {
        let mut image = clean.clone();
        let change = break_copy(page, device);
        rebuild_newest_instance_table(&mut image, &layouts[0].1, &change);
        report_a(&format!("A2 S1 三片、被抛弃实例那一行在第2片 {suffix}"), &image);
    }
    // 同一片里被抛弃实例两行：txg 低的在前 / 在后（I-3.8 行唯一那一半会红；看门开不开、滤不滤）
    let (slot0, page0) = newest_instance_table(&clean);
    for (name, low_first) in [("S4 同一片两行txg低的在前", true), ("S4 同一片两行txg低的在后", false)] {
        let mut image = clean.clone();
        let mut records = records_of(&page0);
        let chain = records.pop().expect("链");
        let high = row_with(abandoned_instance, abandoned.1, &abandoned_row);
        let position = records.iter().position(|r| r[0] == 0 && u32_at(r, 1) == abandoned_instance).expect("有那一行");
        if low_first {
            records.insert(position + 1, high);
        } else {
            records.insert(position, high);
        }
        records.push(chain);
        let rebuilt = build_page(&page0, &records, 0);
        replace_first_page(&mut image, slot0, &rebuilt);
        report_a(&format!("A2 {name}"), &image);
    }
    // 链指针记录在片中间（之后还有一行被抛弃实例的）、链指针记录之后多一行被抛弃实例的
    {
        let mut image = clean.clone();
        let mut records: Vec<Vec<u8>> = records_of(&page0);
        let chain = records.pop().expect("链");
        let mut kept: Vec<Vec<u8>> = records.iter().filter(|r| !(r[0] == 0 && u32_at(r, 1) == abandoned_instance)).cloned().collect();
        kept.push(chain.clone());
        kept.push(abandoned_row.clone());
        kept.push(chain);
        let rebuilt = build_page(&page0, &kept, 0);
        replace_first_page(&mut image, slot0, &rebuilt);
        report_a("A2 S6 链指针记录在片中间、被抛弃实例那一行夹在两条链指针记录之间", &image);
    }
    {
        let mut image = clean.clone();
        let mut records: Vec<Vec<u8>> = records_of(&page0);
        let chain = records.pop().expect("链");
        let mut kept: Vec<Vec<u8>> = records.iter().filter(|r| !(r[0] == 0 && u32_at(r, 1) == abandoned_instance)).cloned().collect();
        kept.push(chain);
        kept.push(abandoned_row.clone());
        let rebuilt = build_page(&page0, &kept, 0);
        replace_first_page(&mut image, slot0, &rebuilt);
        report_a("A2 S8 被抛弃实例那一行在链指针记录之后（片末）", &image);
    }
    // 第 1 片是第 0 片行的整份拷贝（行全重复一遍）
    {
        let mut image = clean.clone();
        rebuild_newest_instance_table(&mut image, &[rows.clone(), rows.clone()], &no_change);
        report_a("A2 S7 两片、第1片是第0片行的整份拷贝", &image);
    }
    // 第 1 片的链指针两条位置条目盘序倒过来（盘 1 在前）
    {
        let mut image = clean.clone();
        rebuild_newest_instance_table(&mut image, &[without_abandoned.clone(), with_abandoned.clone()], &no_change);
        let (slot0_now, page0_now) = newest_instance_table(&image);
        let mut records = records_of(&page0_now);
        let last = records.len() - 1;
        let pointer = records[last][2..88].to_vec();
        let mut swapped = pointer.clone();
        swapped[50..64].copy_from_slice(&pointer[64..78]);
        swapped[64..78].copy_from_slice(&pointer[50..64]);
        records[last][2..88].copy_from_slice(&swapped);
        let rebuilt = build_page(&page0_now, &records, 0);
        replace_first_page(&mut image, slot0_now, &rebuilt);
        report_a("A2 S10 两片、第1片指针的两条位置条目盘序倒过来", &image);
    }
}

// ================================================================ B 段：合法历史（第二次抽样，换种子；加 inode 多叶起点 SI）

/// judge_b 之外再看的几条：I-9.2 红、I-4.8 / I-7.4 / I-7.2 红（读不出孩子那一支在合法历史上有没有被走到）。
fn extra_flags(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<String> {
    let mut flags = Vec::new();
    for name in ["I-9.2", "I-4.8", "I-7.4", "I-7.2", "I-2.1"] {
        if verdicts.iter().any(|(n, v)| *n == name && matches!(v, InvariantVerdict::Violated(_))) {
            flags.push(format!("RED {name}"));
        }
    }
    flags
}

#[test]
fn b2_legal_histories_with_the_user_steps_open() {
    let which = std::env::var("OPUS_START").unwrap_or_else(|_| "S0".to_string());
    let seed_from = environment_number("OPUS_SEED_FROM", 1);
    let histories = environment_number("OPUS_HISTORIES", 2);
    let steps = environment_number("OPUS_STEPS", 6) as usize;
    let started = std::time::Instant::now();
    let (base, versions, abandoned, knob) = if which == "SI" {
        let mut pool = SparsePool::build();
        let mut versions = vec![pool.mark()];
        pool.create(150).expect("建 150 个号");
        versions.push(pool.mark());
        pool.create(150).expect("再建 150 个号");
        versions.push(pool.mark());
        (pool, versions, Vec::new(), None)
    } else {
        starting_point_b(&which)
    };
    let mut tally = BTally::default();
    let start_flags = judge_b(&base.image(), knob, &base, &abandoned, &mut tally);
    println!("RESULT B start {which} flags={start_flags:?}");
    eprintln!("TIMING start {which} seconds={:.1}", started.elapsed().as_secs_f64());
    for seed in seed_from..seed_from + histories {
        let line = run_history_b(&base, &versions, &abandoned, knob, seed, steps, &mut tally);
        println!("{line}");
    }
    println!("RESULT B tally {which} checks={} exposed={}", tally.checks, tally.exposed);
    for (key, count) in &tally.one_birth {
        println!("RESULT B tally {which} I-9.29 {key} {count}");
    }
    for (key, count) in &tally.flags {
        println!("RESULT B tally {which} flag {key} {count}");
    }
    for (key, (done, refused)) in &tally.acts {
        println!("RESULT B tally {which} act {key} done={done} refused={refused}");
    }
    for (key, count) in &tally.other_reds {
        println!("RESULT B tally {which} other-red {key} {count}");
    }
    eprintln!("TIMING B {which} seconds={:.1}", started.elapsed().as_secs_f64());
}

// ================================================================ 崩溃状态数（只数，不枚举）：几段与读不出孩子相关的合法流

use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

#[test]
fn k_count_crash_states_of_small_flows() {
    let base = SparsePool::build();
    let si = inode_pool(&base);
    let flows: Vec<(&str, SparsePool, &str)> = vec![
        ("SI 上建 1 个号", si.clone(), "create1"),
        ("SI 上对 5 号发意图", si.clone(), "intent5"),
        ("S0 上建 1 个号", base.clone(), "create1"),
    ];
    for (name, pool, act) in flows {
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = pool
            .devices
            .iter()
            .map(|(identity, device)| {
                let mut copy = new_device();
                copy.image = device.image.clone();
                (*identity, RecordingBlockDevice::with_shared_stream(*identity, copy, stream.clone()))
            })
            .collect();
        let parameters = parameters();
        let mut allocator = pool.allocator.clone();
        {
            let mut writer = PoolWriter::new(&parameters, &mut devices);
            let outcome = match act {
                "create1" => publish_new_inodes(&mut writer, &mut allocator, &pool.output, 1, WT, pool.instance()).map(|_| ()),
                "intent5" => publish_unlink_intent(&mut writer, &mut allocator, &pool.output, InodeNumber(5), WT, pool.instance()).map(|_| ()),
                other => panic!("{other}"),
            };
            outcome.expect("发布做成");
        }
        let operations = stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations, &common::geometry());
        println!(
            "RESULT K {name} writes={} segments={} segment_lengths={:?} closed_form_states={}",
            writes.len(),
            segments.len(),
            segments.iter().map(Vec::len).collect::<Vec<_>>(),
            closed_form_state_count(&segments)
        );
    }
}

/// 单形细节：`OPUS_POOL` 起点、`OPUS_NODE`（nodes_of_the_tree 里的下标）、`OPUS_ENTRY`、`OPUS_MOD`（Keep / Flip / Unreg）、`OPUS_C0` / `OPUS_C1`（R / S2 / S3 / Z），
/// 印每条违例的细节（前 260 字）。
#[test]
fn p_details_of_one_shape() {
    let which = std::env::var("OPUS_POOL").unwrap_or_else(|_| "I0".to_string());
    let code = code_label();
    let node_index = environment_number("OPUS_NODE", 0) as usize;
    let index = environment_number("OPUS_ENTRY", 0) as usize;
    let m = match std::env::var("OPUS_MOD").unwrap_or_default().as_str() {
        "Flip" => EntryMod::Flip,
        "Unreg" => EntryMod::Unreg,
        _ => EntryMod::Keep,
    };
    let state = |name: &str| match std::env::var(name).unwrap_or_default().as_str() {
        "S2" => CopyState::S2,
        "S3" => CopyState::S3,
        "Z" => CopyState::Z,
        _ => CopyState::R,
    };
    let (c0, c1) = (state("OPUS_C0"), state("OPUS_C1"));
    let base = SparsePool::build();
    let (clean, kind, target, knob) = clean_image_named(&base, &which);
    let record_type = record_type_of_kind(kind);
    let root = root_slot_of_kind(&clean, kind).expect("树有根");
    let sources = stale_sources(&clean, root);
    let nodes = nodes_of_the_tree(&clean, root);
    let node = nodes[node_index].clone();
    let child_slot = parse_node_pointer(&node.entries[index][34..120]).locations[0].slot;
    let mut image = clean.clone();
    apply_entry_mod(&mut image, &node, index, m, record_type);
    put_copy(&mut image, 0, child_slot, c0, &sources);
    put_copy(&mut image, 1, child_slot, c1, &sources);
    let key = entry_key(&which, &node, index);
    for (invariant, verdict) in verdicts_of(&image, knob) {
        if let InvariantVerdict::Violated(detail) = verdict {
            println!("RESULT P {code} {key} child_slot={child_slot} {m:?} c0={c0:?} c1={c1:?} {invariant}: {}", detail.chars().take(260).collect::<String>());
        }
    }
    let ring: Vec<(u32, u64)> = { let mut r: Vec<(u32, u64)> = ring_roots(&image).iter().map(|r| (r.instance, r.txg)).collect(); r.sort(); r };
    println!("RESULT P {code} {key} ring={ring:?}");
}
