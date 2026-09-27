//! E158（择根与修复四岔路）跑前登记第一段：入库装置。
//!
//! 派发范围只跑第一段（`research/prompts/e158-preregistration.md` 5.8）：
//! 停机条款 S1–S5 的逐项对拍、5.7 常量回比、第七节全部锚点、H3（岔路 3 的历史族）不注入全跑出 N_w 与轨迹、
//! Q3-2、Q2-3 的算术、PC-Nw、PC3。岔路 1（C393）、岔路 2（C331）、岔路 4（C334）与它们的臂表、副本、
//! Q1 / Q2 / Q4 全部量都不在这一段，本文件不实现。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`；几何全部走本文件的本地常量，
//! 不 `use` `crates/singlefs-harness/src/segments.rs` 的 `FixedGeometry` 默认值或别的写死几何来决定本轮的工作量 /
//! 几何 / 判据门槛——本地常量的值抄自 `research/prompts/e158-preregistration.md` 5.7 与第七节，注明来源，
//! 另加回比断言与 `crates::` 同名常量比对（S1–S5 与 5.7 表，`run_constants_and_anchors`）。
//! `FixedGeometry` 类型本身在 PC3 的 `SharedFaultPlan::armed` 里用到一次：那是故障注入 API 的必经参数类型，
//! 填的是本文件自己算出的本地几何值，不是从它的默认值读几何——这一处判断留给交回报告，请主 agent 核。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;

use singlefs_checker::image as checker_image;
use singlefs_checker::image::{ImageReader, PoolGeometry};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, ring_slots_known_to_hold_a_root_by,
    roll_back_by_a_forward_publish, rollback_floor_ceiling, Mounted, RollbackTarget, ShadowLedger,
};

/// 回退改形态之后（实现批次实三）这个装置里的「管理员回退」照最直接的翻译改到编得过：挂载时回退（原 `mount::mount_rollback`）没了，
/// 换成先可写挂载、再挂着回退到同一个目标（`mount::roll_back_by_a_forward_publish`，D23（journal 的角色与格式） 已定项 14）。
/// 交回的 `Mounted` 是那次可写挂载的（`output` 里的写行与暖机是它的），`current` 换成回退那次发布之后的一版。
/// 影子账那一臂照旧恒开（可写挂载的产品路径）。
/// 用它的几处（经 `apply_mount_rollback` 的 R(j) 一步：H3 的 `enumerate`、Q3-1 的
/// `enumerate_fault_set_violations`、H1 的 `ledger_fault_history_family`；op1 `MountRollbackTo`；
/// Q2-2a 固定脚本）记的是挂载时回退才有的东西：回退抛弃目标之后那一段（`RollbackEvent::abandoned`、
/// 装置自记的时间线截到目标）、回退写行与之后的暖机。挂着回退不抛弃根、不写行、不暖机，这几处的问法
/// 不再成立；实现批次实四甲没改它们的语义，交主 agent 随 E158（择根与修复四岔路） 的重跑登记定。
fn mount_rollback<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    _shadow_ledger_is_always_on: ShadowLedger,
) -> Result<Mounted, String> {
    let mut mounted =
        mount_writable(parameters, devices).map_err(|error| format!("可写挂载：{error:?}"))?;
    let PoolVersion::WithFile(current) = &mut mounted.current else {
        return Err("可写挂载之后现行那一版树表 0 条：挂着回退要带文件的现行版本".to_string());
    };
    roll_back_by_a_forward_publish(parameters, devices, &mut mounted.allocator, current, target)
        .map_err(|error| format!("挂着回退：{error:?}"))?;
    Ok(mounted)
}
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::{
    parse_mapping_entry, TreeTableEntry, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION,
    TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, instance_table_chain_of_root,
    recover, replay_journal, scan_journal, tree_table_has_no_entries, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{parse_index_node, unit_filesystem_identifier};
use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
use singlefs_format::JOURNAL_RECORD_BYTES;
use singlefs_harness::fault_injection::{
    injected_block_device_error, FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice,
    FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, NamedRootRingSlots,
    PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::segments::FixedGeometry;

/// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②）：旧的计数上限
/// （`SUBSET_ENUMERATION_CAP=20_000`）会在权重档中途停手、不报「完整空间多大」，判「负结果」时
/// 分不清是穷举完了还是撞了计数——按登记要求撤掉，换成下面两个按权重档边界决定停不停的常量。
///
/// 完整证据空间的子集数 ≤ 这个预算 ⇒ `weight_ceiling` 传 `None` 时穷举到 `maximum_weight`（真正的
/// 完整空间）；超过 ⇒ 退到 `DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE`（权重 12，与丙的 k=12 可比，
/// 岔路单原文「某个 n1 的完整空间大到按本机挂钟算不完的，按权重从小到大穷举到权重 12」）。100_000
/// 个子集按 H-C1 下界探针实测速率（22558 个子集 real 2m45s ≈ 7.3ms/个，
/// `research/results/e158-root-choice-repair-2026-09-24-q2-1-hc1-lower-bound.out`）折算约 12 分钟，
/// 在一次前台或后台调用里可接受；调用方显式传 `Some(w)` 时这个自动判断不生效，直接用调用方给的
/// 权重（例如 H-C1 下界探针自己要精确探到某一档）。
const FEASIBLE_FULL_SEARCH_SUBSET_BUDGET: u64 = 100_000;
/// 完整空间太大时退到的权重上限：丙（今天）在 n1=4 上实测恰好在权重 12 打中（H-C1 直接构造 +
/// 下界探针，session s6），选它是为了让各修法在同一档权重上可比。
const DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE: u64 = 12;

/// 两块盘各自的字节数：够装全部取样点的几何（最大 journal 环 3 MiB、S 至多 16），4 GiB 与
/// `crates/singlefs-harness/tests/common/mod.rs` 的 `IMAGE_BYTES` 同一个量级，不共用那份代码。
const IMAGE_BYTES: u64 = 4 << 30;
/// 固定 fsid：与 E142 装置那份不同，避免两个进程各自建的镜像撞出同一份「历史上出现过」的巧合。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x45, 0x31, 0x35, 0x38, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x00,
];
/// 装置自己发的文件很小：H3 只关心根环上的 (实例, txg) 与「读得出哪些根」，不关心内容字节数，
/// 小文件让每次发布的挂钟代价降到最低（大规模穷举需要）。
const FIRST_FILE_BYTES: usize = 96;
const OVERWRITE_FILE_BYTES: usize = 96;
const FIXED_WRITE_TIME_SECONDS: u64 = 1_800_000_000;

/// 项目全仓统一的产物完整性闸口径（`research/scripts/replay.sh` 第 11 行「收尾行必须是
/// `name=done emitted=N`」）：每一条结果行都以 `E7RESULT` 起头，`main` 收尾打一条
/// `E7RESULT name=done emitted=N`，N = 本进程打过的 `E7RESULT` 行数（含收尾这一条）。
static EMITTED_RESULT_LINES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn emit_result(line: &str) {
    EMITTED_RESULT_LINES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    println!("E7RESULT {line}");
}

fn emitted_result_line_count() -> u64 {
    EMITTED_RESULT_LINES.load(std::sync::atomic::Ordering::SeqCst)
}

// ============================================================================
// 一、5.7 装置要本地化的常量：本地写死，逐个与 `crates::` 同名常量比对，不等就停（V3）。
//    这些常量本身只在 `run_constants_and_anchors` 里与 `crates::` 比对；构造几何用的是下面
//    「二、几何」里的本地字面量，不从这里或 `crates::` 读。
// ============================================================================

/// 一条常量回比的结果行。
struct ConstantCheck {
    name: &'static str,
    local_value: u64,
    crates_value: u64,
}

impl ConstantCheck {
    fn matches(&self) -> bool {
        self.local_value == self.crates_value
    }
}

/// 系统配置字段表合计本地按臂的定义现算（跑前登记 5.7「乙族各臂的系统配置字段表比 481 + 8」）：
/// 环境变量 `SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES` 给了就用它，没给按今天那一份取 489——
/// D22（单元原子性怎么合成） 已定项 9 的字段表合计 489（系统运行量末尾加了回退下界 F 8 字节）；
/// 跑前登记写的 481 是登记那天的今天。乙族各臂这时要传几，归 E158（择根与修复四岔路） 的重跑登记定。
/// 装置源码各臂同一份（跑前登记「装置写在哪」），差别只在跑的时候传不传这个环境变量，不是另分支代码。
fn local_system_configuration_bytes() -> u64 {
    parse_local_system_configuration_bytes(env::var(
        "SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES",
    ))
}

/// `local_system_configuration_bytes` 的纯函数一半，与 `crash_injection.rs` 里
/// `CrashInjectingWorkerThreads::from_the_environment_value` 同一个理由把环境变量的读结果
/// 当参数传进来：单测不碰真环境变量（并发跑的单测共用同一个进程环境，互相踩）。
fn parse_local_system_configuration_bytes(value: Result<String, env::VarError>) -> u64 {
    match value {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|error| panic!("SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES={value:?} 不是十进制正整数: {error}")),
        Err(env::VarError::NotPresent) => 489,
        Err(error) => panic!("SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES 读不出: {error}"),
    }
}

/// 5.7 表：本地常量与 `crates::` 那一份逐条比对（V3：任一不等，整轮不开跑）。
fn local_constants_checks() -> Vec<ConstantCheck> {
    vec![
        ConstantCheck {
            name: "根环区域数 ROOT_RING_REGIONS",
            local_value: 3,
            crates_value: singlefs_format::ROOT_RING_REGIONS,
        },
        ConstantCheck {
            name: "每区槽数 mkfs 默认 ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM",
            local_value: 8,
            crates_value: singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        },
        ConstantCheck {
            name: "每区槽数下界 ROOT_RING_SLOTS_PER_REGION_MINIMUM",
            local_value: 4,
            crates_value: singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        },
        ConstantCheck {
            name: "每区槽数上界 ROOT_RING_SLOTS_PER_REGION_MAXIMUM",
            local_value: 16,
            crates_value: singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        },
        ConstantCheck {
            name: "journal 默认环长 JOURNAL_RING_DEFAULT_BYTES",
            local_value: 768 * 1024 * 1024,
            crates_value: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        },
        ConstantCheck {
            name: "journal 记录宽 JOURNAL_RECORD_BYTES",
            local_value: 4096,
            crates_value: singlefs_format::JOURNAL_RECORD_BYTES,
        },
        ConstantCheck {
            name: "journal 环起点槽 JOURNAL_RING_START_SLOT",
            local_value: 1024,
            crates_value: singlefs_format::JOURNAL_RING_START_SLOT,
        },
        ConstantCheck {
            name: "落点粒度 SLOT_BYTES",
            local_value: 16384,
            crates_value: singlefs_format::SLOT_BYTES,
        },
        ConstantCheck {
            name: "节点宽 NODE_BYTES",
            local_value: 16384,
            crates_value: singlefs_format::NODE_BYTES,
        },
        ConstantCheck {
            name: "数据单元宽 DATA_UNIT_BYTES",
            local_value: 32768,
            crates_value: singlefs_format::DATA_UNIT_BYTES,
        },
        ConstantCheck {
            name: "系统配置字段表合计 SYSTEM_CONFIGURATION_BYTES",
            local_value: local_system_configuration_bytes(),
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_BYTES,
        },
        ConstantCheck {
            name: "系统配置槽宽 SYSTEM_CONFIGURATION_SLOT_BYTES",
            local_value: 4096,
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
        },
        ConstantCheck {
            name: "系统配置每盘槽数 SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE",
            local_value: 2,
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        },
        ConstantCheck {
            name: "根记录宽 ROOT_RECORD_BYTES",
            local_value: 457,
            crates_value: singlefs_format::ROOT_RECORD_BYTES,
        },
        ConstantCheck {
            name: "实例表行宽 INSTANCE_ROW_BYTES",
            local_value: 88,
            crates_value: singlefs_format::INSTANCE_ROW_BYTES,
        },
        ConstantCheck {
            name: "实例表一片记录数 INSTANCE_TABLE_PAGE_RECORDS",
            local_value: 370,
            crates_value: singlefs_format::INSTANCE_TABLE_PAGE_RECORDS,
        },
        ConstantCheck {
            name: "系统配置整槽校验和偏移（装置自己的算术 4+2+96+16+20+1+4+4+8，与 core 的私有算法比）",
            local_value: 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8,
            crates_value: u64::try_from(singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET)
                .expect("155 装得进 u64"),
        },
    ]
}

/// 「三个区域住哪块盘」：登记表说「照 `crates/` 那一份抄」——这里抄的是值（0, 1, 0），
/// 不是从 `crates::` 活着读这个数组来决定 mkfs 参数；构造 `MakeFilesystemParameters` 时
/// 用的是下面 `region_devices()` 里同一份字面量，两处各自维护、在这里比对。
fn region_devices_matches_crates() -> (bool, [u32; 3], [u32; 3]) {
    let local: [u32; 3] = [0, 1, 0];
    let crates_value =
        singlefs_core::make_filesystem::FIRST_VERSION_REGION_DEVICES.map(|identity| identity.0);
    (local == crates_value, local, crates_value)
}

// ============================================================================
// 二、几何：本轮跑的四个几何点（5.1 H3 行：GEOMETRY_PRIMARY、S=16、小环、S=4 另跑 L=4）。
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct Geometry {
    label: &'static str,
    slots_per_region: u64,
    journal_ring_bytes: u64,
}

const GEOMETRY_PRIMARY: Geometry = Geometry {
    label: "GEOMETRY_PRIMARY(S=8,ring=3MiB)",
    slots_per_region: 8,
    journal_ring_bytes: 3 * 1024 * 1024,
};
const GEOMETRY_LARGER_ROOT_RING: Geometry = Geometry {
    label: "S16(S=16,ring=3MiB)",
    slots_per_region: 16,
    journal_ring_bytes: 3 * 1024 * 1024,
};
const GEOMETRY_SMALLER_ROOT_RING: Geometry = Geometry {
    label: "S4(S=4,ring=3MiB)",
    slots_per_region: 4,
    journal_ring_bytes: 3 * 1024 * 1024,
};

fn region_devices() -> [DeviceIdentity; 3] {
    [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)]
}

fn parameters_for(geometry: &Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: region_devices(),
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: geometry.journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                geometry.slots_per_region,
            )
            .expect("S 在 [4,16] 区间内"),
        },
    }
}

fn fixed_geometry_for(geometry: &Geometry) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: geometry.journal_ring_bytes,
        root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
            geometry.slots_per_region,
        )
        .expect("S 在 [4,16] 区间内"),
    }
}

// ============================================================================
// 三、装置状态：内存盘 + 会话 + 装置自己的父链账本（不借 `crates/` 的实例表判时间线）。
// ============================================================================

type TimelineRoot = (u32, u64);

#[derive(Clone)]
struct Session {
    allocator: PoolAllocator,
    current: PoolVersion,
    instance: InstanceGeneration,
}

#[derive(Clone)]
struct RollbackEvent {
    /// 这次回退之前的当前时间线上，目标根之后那一段——按 5.0「父链」定义整段记入。
    abandoned: BTreeSet<TimelineRoot>,
}

/// 只为 `TRACE` 行里的 `path` 打印用：`RollbackTo` 的两个字段只被 `Debug` 读，`dead_code` 分析看不见
/// 派生的 `Debug` 实现，允许它照报。
#[allow(
    dead_code,
    reason = "RollbackTo 的两个字段只被派生的 Debug 实现读，dead_code 分析看不见这条路"
)]
#[derive(Clone, Copy, Debug)]
enum HistoryStep {
    Overwrite,
    MountWritable,
    RollbackTo(u32, u64),
}

#[derive(Clone)]
struct SimNode {
    pool: MemoryPool,
    session: Option<Session>,
    /// 装置自记的「当前时间线」：从 mkfs 的第 0 代根到最新一次成功发布的根，按父链排列。
    timeline: Vec<TimelineRoot>,
    rollback_events: Vec<RollbackEvent>,
    path: Vec<HistoryStep>,
    /// Q3-1（P332 的 V2）用：这个模拟池里**曾经存在过**的每一条根对应的「内容号」——
    /// `None` = 那一版没有文件，`Some(字节)` = 那一版文件的确定性内容。只增不删（回退不截断它），
    /// 因为 V2 要判「读回的内容属不属于被抛弃的那一段」，被抛弃的根的内容号仍要留着比对。
    content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>,
}

fn new_devices(image_bytes: u64) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(image_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect()
}

fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)], image_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: image_bytes,
    }
}

fn devices_from_pool(
    pool: &MemoryPool,
    image_bytes: u64,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let mut device = SparseBlockDevice::new(image_bytes, PhysicalBlockSizeInBytes(512));
            if let Some(image) = pool.devices.get(&identity) {
                device.image = image.clone();
            }
            (identity, device)
        })
        .collect()
}

fn deterministic_content(tag: u64, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let value = (index as u64)
                .wrapping_mul(131)
                .wrapping_add(tag.wrapping_mul(977))
                .wrapping_add(7);
            (value % 251) as u8
        })
        .collect()
}

fn root_pair(root: &singlefs_core::root_record::RootRecord) -> TimelineRoot {
    (root.instance.0, root.checkpoint_txg.0)
}

// ============================================================================
// 四、步进：W（覆盖写）、M（关闭并 `mount_writable`）、R(j)（关闭并 `mount_rollback` 到 j）。
//    三个都从 `&SimNode` 读、交回一个新 `SimNode`；不改传入的那个（DFS 靠这个做回溯）。
// ============================================================================

fn bootstrap(geometry: &Geometry, overwrites_before_sigma: u64) -> Result<SimNode, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    // mkfs 同一个进程里接着写的分配器照 mkfs 按环长现算的单元区起点建（实审 A3b 报告 Q8：这里的环 3 MiB，起点 1216，
    // 不是编译期常量 50176），并装上 mkfs 刚写下的根环。
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = deterministic_content(0, FIRST_FILE_BYTES);
    let (warm_up_output, first_output) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer)
            .map_err(|error| format!("acquire_instance: {error:?}"))?;
        let warm = warm_up(&mut writer, &genesis.root, instance)
            .map_err(|error| format!("warm_up: {error:?}"))?;
        let output = publish_first_file(
            &mut writer,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm.last_record_bytes,
        )
        .map_err(|error| format!("publish_first_file: {error:?}"))?;
        (warm, output)
    };
    let mut timeline = vec![root_pair(&genesis.root)];
    let mut content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>> = BTreeMap::new();
    content_by_root.insert(root_pair(&genesis.root), None);
    for root in &warm_up_output.roots {
        timeline.push(root_pair(root));
        content_by_root.insert(root_pair(root), None);
    }
    timeline.push(root_pair(&first_output.root));
    content_by_root.insert(root_pair(&first_output.root), Some(content.clone()));
    let instance = InstanceGeneration(1);
    let mut node = SimNode {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: Some(Session {
            allocator,
            current: PoolVersion::WithFile(first_output),
            instance,
        }),
        timeline,
        rollback_events: Vec::new(),
        path: Vec::new(),
        content_by_root,
    };
    for _ in 0..overwrites_before_sigma {
        node = apply_overwrite(&node, &parameters)?;
    }
    Ok(node)
}

fn apply_overwrite(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> Result<SimNode, String> {
    let session = node
        .session
        .as_ref()
        .ok_or_else(|| "W: 没有开着的会话".to_string())?;
    let PoolVersion::WithFile(previous) = &session.current else {
        return Err("W: 现行版本没有文件".to_string());
    };
    let step_tag = node.path.len() as u64 + 1;
    let content = deterministic_content(step_tag, OVERWRITE_FILE_BYTES);
    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut allocator = session.allocator.clone();
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * step_tag,
            },
            session.instance,
        )
        .map_err(|error| format!("publish_overwrite: {error:?}"))?
    };
    let mut timeline = node.timeline.clone();
    timeline.push(root_pair(&output.root));
    let mut content_by_root = node.content_by_root.clone();
    content_by_root.insert(root_pair(&output.root), Some(content));
    let mut path = node.path.clone();
    path.push(HistoryStep::Overwrite);
    Ok(SimNode {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: Some(Session {
            allocator,
            current: PoolVersion::WithFile(output),
            instance: session.instance,
        }),
        timeline,
        rollback_events: node.rollback_events.clone(),
        path,
        content_by_root,
    })
}

fn apply_mount_writable(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> Result<SimNode, String> {
    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = mount_writable(parameters, &mut devices)
        .map_err(|error| format!("mount_writable: {error:?}"))?;
    let tip = *node.timeline.last().expect("timeline 非空");
    let chosen = root_pair(&mounted.output.chosen_root);
    if chosen != tip {
        return Err(format!(
            "S5: mount_writable 的 chosen_root {chosen:?} 与装置自记的时间线尾 {tip:?} 不一致（不注入故障时应当相等）"
        ));
    }
    let carried_forward_content = node.content_by_root.get(&tip).cloned().unwrap_or(None);
    let mut timeline = node.timeline.clone();
    timeline.push(root_pair(mounted.output.row_publish.root()));
    let mut content_by_root = node.content_by_root.clone();
    content_by_root.insert(
        root_pair(mounted.output.row_publish.root()),
        carried_forward_content.clone(),
    );
    for warm_up_publish in &mounted.output.warm_up_publishes {
        timeline.push(root_pair(warm_up_publish.root()));
        content_by_root.insert(
            root_pair(warm_up_publish.root()),
            carried_forward_content.clone(),
        );
    }
    let mut path = node.path.clone();
    path.push(HistoryStep::MountWritable);
    Ok(SimNode {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: Some(Session {
            allocator: mounted.allocator,
            current: mounted.current,
            instance: mounted.output.instance,
        }),
        timeline,
        rollback_events: node.rollback_events.clone(),
        path,
        content_by_root,
    })
}

fn apply_mount_rollback(
    node: &SimNode,
    target: RollbackTarget,
    parameters: &MakeFilesystemParameters,
) -> Result<SimNode, String> {
    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = mount_rollback(parameters, &mut devices, target, ShadowLedger::On)
        .map_err(|error| format!("mount_rollback: {error:?}"))?;
    let target_pair = (target.instance.0, target.checkpoint_txg.0);
    let position = node
        .timeline
        .iter()
        .position(|root| *root == target_pair)
        .ok_or_else(|| format!("S5: 回退目标 {target_pair:?} 不在装置自记的时间线上"))?;
    let abandoned: BTreeSet<TimelineRoot> = node.timeline[position + 1..].iter().copied().collect();
    let carried_forward_content = node
        .content_by_root
        .get(&target_pair)
        .cloned()
        .unwrap_or(None);
    let mut timeline: Vec<TimelineRoot> = node.timeline[..=position].to_vec();
    let mut content_by_root = node.content_by_root.clone();
    timeline.push(root_pair(mounted.output.row_publish.root()));
    content_by_root.insert(
        root_pair(mounted.output.row_publish.root()),
        carried_forward_content.clone(),
    );
    for warm_up_publish in &mounted.output.warm_up_publishes {
        timeline.push(root_pair(warm_up_publish.root()));
        content_by_root.insert(
            root_pair(warm_up_publish.root()),
            carried_forward_content.clone(),
        );
    }
    let mut rollback_events = node.rollback_events.clone();
    rollback_events.push(RollbackEvent { abandoned });
    let mut path = node.path.clone();
    path.push(HistoryStep::RollbackTo(
        target.instance.0,
        target.checkpoint_txg.0,
    ));
    Ok(SimNode {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: Some(Session {
            allocator: mounted.allocator,
            current: mounted.current,
            instance: mounted.output.instance,
        }),
        timeline,
        rollback_events,
        path,
        content_by_root,
    })
}

// ============================================================================
// 五、装置自己的独立解码：不借 `singlefs_core::recovery` 的实例表判时间线，
//    走 `singlefs_checker::image`（另一套解析代码，D13 已定项 8 那条「checker 独立判」的现成落点）。
// ============================================================================

/// 两盘系统配置解出的几何一致才交回；否则报 S5（两边都查）。
fn independent_geometry(pool: &MemoryPool) -> Result<PoolGeometry, String> {
    let chosen = checker_image::chosen_system_configurations(pool);
    let mut geometries = Vec::new();
    for (device, entry) in &chosen {
        match entry {
            Some((_, geometry)) => geometries.push(*geometry),
            None => return Err(format!("S5: 设备 {device} 没有自证的系统配置")),
        }
    }
    let first = *geometries
        .first()
        .ok_or_else(|| "S5: 池里没有盘".to_string())?;
    for other in &geometries[1..] {
        if other != &first {
            return Err("S5: 两盘系统配置解出的几何不一致".to_string());
        }
    }
    Ok(first)
}

/// 根环里全部自证过的根，按 (实例, txg)（checker 的独立解码，`singlefs_checker::check_root_slot`）。
fn readable_roots_independent(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
) -> BTreeSet<TimelineRoot> {
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .map(|(_, _, view)| (view.instance, view.checkpoint_txg))
        .collect()
}

/// 根环里全部自证过的根，连 (区域, 槽) 一起（PC3 要按 (区域, 槽) 点名故障落点）。
fn readable_roots_with_slots(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
) -> Vec<(u64, u64, TimelineRoot)> {
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .map(|(region, slot, view)| (region, slot, (view.instance, view.checkpoint_txg)))
        .collect()
}

/// 这一步「还有可读被抛弃根的回退事件数」（N_live，Q3-2 的 N_w 取它全族的最大值）与
/// 「读得出的被抛弃根条数」（去重后的根数）。纯函数、不碰 `crates/`：PC-Nw 直接单测它。
fn count_live_rollback_events(
    rollback_events: &[RollbackEvent],
    readable: &BTreeSet<TimelineRoot>,
) -> (u64, u64) {
    let mut events_live = 0u64;
    let mut roots_live: BTreeSet<TimelineRoot> = BTreeSet::new();
    for event in rollback_events {
        let mut any = false;
        for root in &event.abandoned {
            if readable.contains(root) {
                roots_live.insert(*root);
                any = true;
            }
        }
        if any {
            events_live += 1;
        }
    }
    (
        events_live,
        u64::try_from(roots_live.len()).expect("根数装得进 u64"),
    )
}

fn live_rollback_events(node: &SimNode, readable: &BTreeSet<TimelineRoot>) -> (u64, u64) {
    count_live_rollback_events(&node.rollback_events, readable)
}

// ============================================================================
// 六、H3 的穷举：σ ∈ Σ3^L，Σ3 = {W, M, R(j)}，j 取那一刻每一条可读根；不抽样、全枚举。
//    「省法」（5.1「H3 的省法」）不在这一段实现：这一段只跑「不注入」，本来就不做 (b)(c) 那两次带故障的挂载，
//    省法省的正是那两次——不注入时没有省的必要，这里如实全枚举、不引省法。
// ============================================================================

#[derive(Default)]
struct EnumerationSummary {
    histories: u64,
    peak_events_live: u64,
    peak_roots_live: u64,
    steps_with_positive_events_live: u64,
    /// H3 按固定次序第一个满足「events_live ≥ 1」的节点（PC3 用）。
    first_events_live_node: Option<SimNode>,
    errors: Vec<String>,
    /// 8.1「期末值」：只在叶节点（σ 跑满 L 步，或没有更长的合法后继）记一次，
    /// 与 `peak_events_live`（跨全部步、含中间步）分开报。
    leaf_count: u64,
    leaves_with_positive_events_live: u64,
    peak_events_live_at_a_leaf: u64,
}

#[allow(
    clippy::too_many_arguments,
    reason = "H3 穷举需要节点、剩余深度、mkfs 参数、几何、汇总器与 trace 回调六项，各自有名字有类型，拆结构体只是把参数挪个地方"
)]
fn enumerate(
    node: &SimNode,
    depth_remaining: u64,
    parameters: &MakeFilesystemParameters,
    geometry: &PoolGeometry,
    summary: &mut EnumerationSummary,
    trace: &mut dyn FnMut(&SimNode, u64, u64),
) {
    summary.histories += 1;
    let readable = readable_roots_independent(&node.pool, geometry);
    let (events_live, roots_live) = live_rollback_events(node, &readable);
    trace(node, events_live, roots_live);
    summary.peak_events_live = summary.peak_events_live.max(events_live);
    summary.peak_roots_live = summary.peak_roots_live.max(roots_live);
    if events_live > 0 {
        summary.steps_with_positive_events_live += 1;
        if summary.first_events_live_node.is_none() {
            summary.first_events_live_node = Some(node.clone());
        }
    }
    if depth_remaining == 0 {
        summary.leaf_count += 1;
        if events_live > 0 {
            summary.leaves_with_positive_events_live += 1;
        }
        summary.peak_events_live_at_a_leaf = summary.peak_events_live_at_a_leaf.max(events_live);
        return;
    }
    // W：只在挂着时合法。
    if node.session.is_some() {
        match apply_overwrite(node, parameters) {
            Ok(next) => enumerate(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                summary,
                trace,
            ),
            Err(error) => summary
                .errors
                .push(format!("W 前提不满足（记为前提不满足，不算错）：{error}")),
        }
    }
    // M：关闭并 mount_writable，任何时候都合法。
    match apply_mount_writable(node, parameters) {
        Ok(next) => enumerate(
            &next,
            depth_remaining - 1,
            parameters,
            geometry,
            summary,
            trace,
        ),
        Err(error) => summary
            .errors
            .push(format!("M 前提不满足（记为前提不满足，不算错）：{error}")),
    }
    // R(j)：关闭并 mount_rollback 到那一刻每一条可读根。
    for root in &readable {
        let target = RollbackTarget {
            instance: InstanceGeneration(root.0),
            checkpoint_txg: CheckpointTxg(root.1),
        };
        match apply_mount_rollback(node, target, parameters) {
            Ok(next) => enumerate(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                summary,
                trace,
            ),
            Err(error) => {
                // 候选排除（不在候选集、低于 F、被抛弃时间线）按结局记，不算装置的错，只是这个分支到此为止。
                summary.errors.push(format!(
                    "R({root:?}) 前提不满足（记为前提不满足，不算错）：{error}"
                ));
            }
        }
    }
}

/// H3 一族：给定几何、overwrites_before_sigma 的取值集合、σ 的最大长度 L，逐一枚举、汇总。
fn run_root_choice_family(
    geometry: &Geometry,
    initial_overwrite_counts: &[u64],
    sigma_length_limit: u64,
    trace_label: &str,
    verbose_trace: bool,
) -> EnumerationSummary {
    let parameters = parameters_for(geometry);
    let mut summary = EnumerationSummary::default();
    for &overwrites_before_sigma in initial_overwrite_counts {
        let root_node = match bootstrap(geometry, overwrites_before_sigma) {
            Ok(node) => node,
            Err(error) => {
                summary.errors.push(format!(
                    "bootstrap(overwrites_before_sigma={overwrites_before_sigma}) 失败：{error}"
                ));
                continue;
            }
        };
        let independent_geometry_view = match independent_geometry(&root_node.pool) {
            Ok(geometry_view) => geometry_view,
            Err(error) => {
                summary.errors.push(error);
                continue;
            }
        };
        let mut steps_visited_for_this_overwrite_count: u64 = 0;
        let mut trace = |node: &SimNode, events_live: u64, roots_live: u64| {
            steps_visited_for_this_overwrite_count += 1;
            if verbose_trace {
                println!(
                    "TRACE geometry={} overwrites_before_sigma={} step={} path={:?} events_live={} roots_live={}",
                    trace_label,
                    overwrites_before_sigma,
                    steps_visited_for_this_overwrite_count,
                    node.path,
                    events_live,
                    roots_live
                );
            }
        };
        enumerate(
            &root_node,
            sigma_length_limit,
            &parameters,
            &independent_geometry_view,
            &mut summary,
            &mut trace,
        );
        println!(
            "N1_DONE geometry={trace_label} overwrites_before_sigma={overwrites_before_sigma} histories_so_far={} peak_events_live={} peak_roots_live={}",
            summary.histories, summary.peak_events_live, summary.peak_roots_live
        );
    }
    summary
}

// ============================================================================
// 七、小环取法（8.2）：从 48 KiB 起、每次加 16 KiB，取第一个 mkfs 收、且「H2 最长模板」整段跑得通的环长。
//    H2 不在这一段跑；这里按 8.2 原文的校准点（overwrites_before_sigma=6、mount_writable、n2=2）复刻一个不依赖 H2 自身实现的代理探针：
//    只判「这几步任何一步是否报错」，不判是哪一类错——H2 完整实现（P331 判定、崩溃档）留给第三段。
//    这个简化在报告里点名，交主 agent 核这个代理够不够格。
// ============================================================================

fn small_ring_probe(ring_bytes: u64) -> bool {
    let geometry = Geometry {
        label: "small-ring-probe",
        slots_per_region: 8,
        journal_ring_bytes: ring_bytes,
    };
    let parameters = parameters_for(&geometry);
    let mut node = match bootstrap(&geometry, 6) {
        Ok(node) => node,
        Err(_) => return false,
    };
    node = match apply_mount_writable(&node, &parameters) {
        Ok(node) => node,
        Err(_) => return false,
    };
    for _ in 0..2 {
        node = match apply_overwrite(&node, &parameters) {
            Ok(node) => node,
            Err(_) => return false,
        };
    }
    true
}

// ============================================================================
// 十、Q3-2、Q2-3 的算术：只用 N_w，不建副本、不跑臂。
// ============================================================================

fn report_witness_width_and_slot_overlap_arithmetic(peak_live_rollback_events: u64) {
    let single_16 = 16u64;
    let single_12 = 12u64;
    let table_16 = 1 + 16 * peak_live_rollback_events;
    let table_12 = 1 + 12 * peak_live_rollback_events;
    for (label, width) in [
        ("单条16", single_16),
        ("单条12", single_12),
        ("表1+16*peak", table_16),
        ("表1+12*peak", table_12),
    ] {
        let within_512 = 512i64 - 481 - i64::try_from(width).expect("宽度装得进 i64");
        let within_4096 = 4096i64 - 481 - i64::try_from(width).expect("宽度装得进 i64");
        emit_result(&format!(
            "name=q3_2_witness_width_margin peak_live_rollback_events={peak_live_rollback_events} width_label={label} width_bytes={width} margin_within_512={within_512} margin_within_512_verdict={} margin_within_4096={within_4096} margin_within_4096_verdict={}",
            if within_512 < 0 { "past_512" } else { "fits" },
            if within_4096 < 0 { "past_4096" } else { "fits" }
        ));
    }
    for (label, width) in [
        ("单条16", single_16),
        ("单条12", single_12),
        ("表1+16*peak", table_16),
        ("表1+12*peak", table_12),
    ] {
        let plus8_within_512 = 512i64 - 481 - 8 - i64::try_from(width).expect("宽度装得进 i64");
        emit_result(&format!(
            "name=q2_3_witness_width_with_candidate_two_txg_field peak_live_rollback_events={peak_live_rollback_events} width_label={label} width_bytes={width} margin_within_512_with_plus8={plus8_within_512} verdict={}",
            if plus8_within_512 < 0 { "past_512_together" } else { "fits" }
        ));
    }
}

// ============================================================================
// 十一、PC3：H3 按固定次序第一个满足「events_live ≥ 1」的历史；F = 那一刻「回退实例」全部根所在的根槽
//    （回退实例 = 刚做完的那次 R(j) 建立起来的新实例——按 4 节已有用例「回退实例 3 的四条根……四条都读不出时
//    恢复退到被抛弃的根 (2, 8)」那个形态操作化：让「刚接手的这个新实例」自己的全部根变得读不出，
//    看择根会不会退回它自己抛弃掉的那条旧时间线）。这个操作化不在 5.6 的清单里，交回报告里点名，请主 agent 核对
//    C332 攻方腿原文是不是同一个意思。
// ============================================================================

fn run_pc3(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry_view: &PoolGeometry,
) -> bool {
    let session_instance = match &node.session {
        Some(session) => session.instance.0,
        None => {
            emit_result(&format!(
                "name=pc3 path={:?} verdict=fail reason=\"没有开着的会话，算不出回退实例\"",
                node.path
            ));
            return false;
        }
    };
    let readable = readable_roots_independent(&node.pool, geometry_view);
    let (events_live, _) = live_rollback_events(node, &readable);
    if events_live == 0 {
        emit_result(&format!(
            "name=pc3 path={:?} verdict=fail reason=\"events_live=0，不满足 PC3 的触发条件\"",
            node.path
        ));
        return false;
    }
    let expected_root = node
        .rollback_events
        .iter()
        .flat_map(|event| event.abandoned.iter().copied())
        .filter(|root| readable.contains(root))
        .max_by_key(|root| (root.1, root.0))
        .expect("events_live>0 时读得出的被抛弃根非空");

    let all_roots = readable_roots_with_slots(&node.pool, geometry_view);
    let named = all_roots
        .iter()
        .filter(|(_, _, root)| root.0 == session_instance)
        .fold(NamedRootRingSlots::NONE, |named, (region, slot, _)| {
            named.with(RootRingSlot {
                region: *region,
                slot: *slot,
            })
        });
    let named_count = all_roots
        .iter()
        .filter(|(_, _, root)| root.0 == session_instance)
        .count();
    if named_count == 0 {
        emit_result(&format!(
            "name=pc3 path={:?} verdict=fail reason=\"回退实例 {session_instance} 在根环里一条自证根都没有，造不出故障集合\"",
            node.path
        ));
        return false;
    }

    let target = RootRingSlotTarget {
        named_slots: named,
        region_devices: region_devices(),
        fixed_structure_slot_spacing: 4096,
    };

    // (a) 带着故障的冷启动 recover：走 `PoolReaderWithUnreadableRootRingSlots` 包读者的那一路。
    let devices_for_recover = devices_from_pool(&node.pool, IMAGE_BYTES);
    let wrapped_reader = PoolReaderWithUnreadableRootRingSlots::new(&devices_for_recover, target);
    let recovery_report = recover(&wrapped_reader, JournalPolicy::Consult);
    let mut recover_failure_detail = String::new();
    let recover_root = match &recovery_report.outcome {
        RecoveryOutcome::NoFile { root } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::FileRead { root, .. } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::Failed { root, failure } => {
            recover_failure_detail = format!("{failure:?}（所选根 {root:?}）");
            None
        }
    };
    let recover_check_passed = recover_root == Some(expected_root);

    // (b) 带着故障的 mount_writable：走 `FaultInjectingBlockDevice` 包块设备那一路。
    let plan = SharedFaultPlan::unarmed(fixed_geometry_for(&GEOMETRY_PRIMARY));
    plan.arm(FaultSchedule::every_read_of_named_root_ring_slots_fails(
        target,
    ));
    let mut faulted_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        devices_from_pool(&node.pool, IMAGE_BYTES)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                )
            })
            .collect();
    let mount_result = mount_writable(parameters, &mut faulted_devices);
    let mut mount_failure_detail = String::new();
    let (mount_writable_check_passed, mount_root) = match mount_result {
        Ok(mounted) => {
            let chosen = root_pair(&mounted.output.chosen_root);
            (chosen == expected_root, Some(chosen))
        }
        Err(error) => {
            mount_failure_detail = format!("{error:?}");
            (false, None)
        }
    };

    emit_result(&format!(
        "name=pc3 path={:?} rollback_instance={session_instance} named_root_slot_count={named_count} expected_root={expected_root:?} recover_verdict={} recover_root={recover_root:?} recover_reads_refused={} recover_failure={recover_failure_detail:?} mount_writable_verdict={} mount_writable_root={mount_root:?} mount_writable_failure={mount_failure_detail:?}",
        node.path,
        if recover_check_passed { "pass" } else { "fail" },
        wrapped_reader.reads_refused(),
        if mount_writable_check_passed { "pass" } else { "fail" }
    ));

    recover_check_passed && mount_writable_check_passed
}

// ============================================================================
// 十二、PC-Nw：手写假账单测计数函数（见文件末 `#[cfg(test)]`）。
// ============================================================================

fn find_small_ring() -> u64 {
    let mut candidate = 48 * 1024u64;
    loop {
        if small_ring_probe(candidate) {
            return candidate;
        }
        candidate += 16 * 1024;
        assert!(
            candidate <= 3 * 1024 * 1024,
            "小环搜索超过 3 MiB 仍未找到，8.2 的规则或代理探针有问题"
        );
    }
}

// ============================================================================
// 十三、main：按命令行第一个参数选模式，`all` 顺序跑完这一段能跑的全部内容。
// ============================================================================

fn run_root_choice_geometry(
    geometry: &Geometry,
    initial_overwrite_counts: &[u64],
    sigma_length_limit: u64,
) -> EnumerationSummary {
    let summary = run_root_choice_family(
        geometry,
        initial_overwrite_counts,
        sigma_length_limit,
        geometry.label,
        false,
    );
    emit_result(&format!(
        "name=h3_family_summary geometry={} overwrites_before_sigma={:?} sigma_length_limit={} histories={} peak_events_live={} peak_roots_live={} steps_with_positive_events_live={} leaf_count={} leaves_with_positive_events_live={} peak_events_live_at_a_leaf={} error_count={}",
        geometry.label,
        initial_overwrite_counts,
        sigma_length_limit,
        summary.histories,
        summary.peak_events_live,
        summary.peak_roots_live,
        summary.steps_with_positive_events_live,
        summary.leaf_count,
        summary.leaves_with_positive_events_live,
        summary.peak_events_live_at_a_leaf,
        summary.errors.len()
    ));
    let mut error_categories: std::collections::BTreeMap<&str, u64> =
        std::collections::BTreeMap::new();
    for error in &summary.errors {
        let category = error
            .split(':')
            .nth(1)
            .map(str::trim)
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or("?");
        *error_categories.entry(category).or_insert(0) += 1;
    }
    for (category, count) in &error_categories {
        emit_result(&format!(
            "name=h3_family_pruned_branch_category geometry={} category={category} count={count}",
            geometry.label
        ));
    }
    summary
}

// ============================================================================
// 八、Q3-1（岔路 3 候选 3 那一半，跑前登记第二段）：P332 在 H3 × Φ3 上的违例比例。
//    今天那一臂 = 候选 3（`crates/` 今天的代码），不建副本。Φ3：那一刻存着自证根的根槽
//    （不按实例筛，5.1 原文），取 |F| ≤ 2 的全部子集（含空集）。
//    5.1「H3 的省法」：一个故障集合的冷启动 recover 所选根与 F=∅ 时相同 ⇒ 不做后两处，
//    直接记它们与 F=∅ 那一对相同（非违例）——F=∅ 本身照样全算，不吃自己的省法。
// ============================================================================

/// P332 在某一处的 (V1, V2)：`chosen_root_abandoned` 是 V1（所选根落在被抛弃集合里），
/// `content_off_timeline` 是 V2（读回的内容号不属于当前时间线上任何一个根）。
#[derive(Clone, Copy, Default)]
struct ViolationAtOneCheckpoint {
    chosen_root_abandoned: bool,
    content_off_timeline: bool,
}

fn is_abandoned(node: &SimNode, root: TimelineRoot) -> bool {
    node.rollback_events
        .iter()
        .any(|event| event.abandoned.contains(&root))
}

/// V2：`signature`（`None` = 没有文件，`Some(字节)` = 文件内容）不等于当前时间线上任何一个根
/// 的内容号 ⇒ 违例。`content_by_root` 只增不删，被抛弃的根的内容号仍留着，所以这条比较对
/// 「读回的内容只在被抛弃的那一段出现过」这句话是精确的（不是近似）。
fn content_is_off_the_current_timeline(node: &SimNode, signature: &Option<Vec<u8>>) -> bool {
    !node
        .timeline
        .iter()
        .any(|root| node.content_by_root.get(root) == Some(signature))
}

/// 交回报告点名的一处操作化（5.6 待认清单没有覆盖到，因为它在跑前登记写成的那天还不可达——
/// 「回退到无文件那一版」当时被 `crates/` 拒绝，C512（树表 0 条的一版上被换下的单元记在哪） 2026-09-23
/// 落地之后才可达）：`NoFile` 结局时 V2 恒记非违例，只判 V1——「读回的内容号」预设了读回了内容，
/// 没有文件时没有内容号可比。
fn violation_at_checkpoint(
    node: &SimNode,
    outcome: &RecoveryOutcome,
) -> Option<ViolationAtOneCheckpoint> {
    match outcome {
        RecoveryOutcome::Failed { .. } => None,
        RecoveryOutcome::NoFile { root } => Some(ViolationAtOneCheckpoint {
            chosen_root_abandoned: is_abandoned(node, (root.0 .0, root.1 .0)),
            content_off_timeline: false,
        }),
        RecoveryOutcome::FileRead { root, content } => Some(ViolationAtOneCheckpoint {
            chosen_root_abandoned: is_abandoned(node, (root.0 .0, root.1 .0)),
            content_off_timeline: content_is_off_the_current_timeline(node, &Some(content.clone())),
        }),
    }
}

fn chosen_root_of(outcome: &RecoveryOutcome) -> Option<TimelineRoot> {
    match outcome {
        RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => {
            Some((root.0 .0, root.1 .0))
        }
        RecoveryOutcome::Failed { root, .. } => {
            root.map(|chosen_root| (chosen_root.0 .0, chosen_root.1 .0))
        }
    }
}

/// P332 在 (a)(b)(c) 三处各自的判定：`cold_recover_with_fault` 是 (a)，
/// `mount_then_recover_with_fault` 是 (b)（`mount_writable` 之后再带着同一个故障冷启动读一次），
/// `recover_after_fault_removed` 是 (c)（撤掉故障之后的冷启动 recover）。
struct ViolationCheckpoints {
    cold_recover_with_fault: Option<ViolationAtOneCheckpoint>,
    mount_then_recover_with_fault: Option<ViolationAtOneCheckpoint>,
    recover_after_fault_removed: Option<ViolationAtOneCheckpoint>,
    cold_recover_with_fault_failed: bool,
    mount_writable_with_fault_failed: bool,
    skipped_by_the_reduction_rule: bool,
}

/// 在给定节点、给定故障集合下把 P332 的 (a)(b)(c) 各判一次。`baseline_chosen_root` 是这段历史
/// F=∅ 时 (a) 的所选根（省法要比的对象）。`devices_for_cold_recover` 由调用方对这一整个节点建一次、
/// 传进来给每一个故障集合的 (a) 复用——(a) 只读，不写这几份设备，一个节点要判几十个故障集合，
/// 每次都重新拷一遍 4 GiB 的稀疏镜像是这个函数最先量出来的挂钟大头（G0 层一里跑不完 100 秒）。
fn evaluate_violation_checkpoints(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry: &Geometry,
    target: RootRingSlotTarget,
    baseline_chosen_root: Option<TimelineRoot>,
    devices_for_cold_recover: &[(DeviceIdentity, SparseBlockDevice)],
) -> ViolationCheckpoints {
    let cold_recover_reader =
        PoolReaderWithUnreadableRootRingSlots::new(devices_for_cold_recover, target);
    let cold_recover_outcome = recover(&cold_recover_reader, JournalPolicy::Consult).outcome;
    let cold_recover_chosen_root = chosen_root_of(&cold_recover_outcome);
    let cold_recover_with_fault = violation_at_checkpoint(node, &cold_recover_outcome);
    let cold_recover_with_fault_failed = cold_recover_with_fault.is_none();

    if target.named_slots != NamedRootRingSlots::NONE
        && cold_recover_chosen_root == baseline_chosen_root
    {
        // 省法：这个故障集合没有改变 (a) 的所选根 ⇒ (b)(c) 与 F=∅ 那一对相同（非违例）。
        return ViolationCheckpoints {
            cold_recover_with_fault,
            mount_then_recover_with_fault: Some(ViolationAtOneCheckpoint::default()),
            recover_after_fault_removed: Some(ViolationAtOneCheckpoint::default()),
            cold_recover_with_fault_failed,
            mount_writable_with_fault_failed: false,
            skipped_by_the_reduction_rule: true,
        };
    }

    let plan = SharedFaultPlan::unarmed(fixed_geometry_for(geometry));
    plan.arm(FaultSchedule::every_read_of_named_root_ring_slots_fails(
        target,
    ));
    let mut faulted_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        devices_from_pool(&node.pool, IMAGE_BYTES)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                )
            })
            .collect();
    let mount_result = mount_writable(parameters, &mut faulted_devices);
    let (mount_then_recover_with_fault, mount_writable_with_fault_failed) = match mount_result {
        Ok(_mounted) => {
            let mount_then_recover_outcome =
                recover(&faulted_devices, JournalPolicy::Consult).outcome;
            (
                violation_at_checkpoint(node, &mount_then_recover_outcome),
                false,
            )
        }
        Err(_error) => (None, true),
    };
    plan.disarm();
    let recover_after_fault_removed_outcome =
        recover(&faulted_devices, JournalPolicy::Consult).outcome;
    let recover_after_fault_removed =
        violation_at_checkpoint(node, &recover_after_fault_removed_outcome);

    ViolationCheckpoints {
        cold_recover_with_fault,
        mount_then_recover_with_fault,
        recover_after_fault_removed,
        cold_recover_with_fault_failed,
        mount_writable_with_fault_failed,
        skipped_by_the_reduction_rule: false,
    }
}

#[derive(Default)]
struct FaultSetViolationSummary {
    nodes_visited: u64,
    pairs: u64,
    pairs_by_weight: BTreeMap<u32, u64>,
    violations_by_weight: BTreeMap<u32, u64>,
    pairs_with_rollback: u64,
    violations_with_rollback: u64,
    pairs_without_rollback: u64,
    violations_without_rollback: u64,
    violations_cold_recover_chosen_root_abandoned: u64,
    violations_cold_recover_content_off_timeline: u64,
    violations_mount_then_recover_chosen_root_abandoned: u64,
    violations_mount_then_recover_content_off_timeline: u64,
    violations_recover_after_fault_removed_chosen_root_abandoned: u64,
    violations_recover_after_fault_removed_content_off_timeline: u64,
    pairs_with_any_violation: u64,
    cold_recover_with_fault_failed: u64,
    mount_writable_with_fault_failed: u64,
    skipped_by_the_reduction_rule: u64,
}

fn record_violation_checkpoints_for_one_fault_set(
    summary: &mut FaultSetViolationSummary,
    weight: u32,
    has_rollback: bool,
    checkpoints: &ViolationCheckpoints,
) {
    summary.pairs += 1;
    *summary.pairs_by_weight.entry(weight).or_insert(0) += 1;
    if has_rollback {
        summary.pairs_with_rollback += 1;
    } else {
        summary.pairs_without_rollback += 1;
    }
    if checkpoints.cold_recover_with_fault_failed {
        summary.cold_recover_with_fault_failed += 1;
    }
    if checkpoints.mount_writable_with_fault_failed {
        summary.mount_writable_with_fault_failed += 1;
    }
    if checkpoints.skipped_by_the_reduction_rule {
        summary.skipped_by_the_reduction_rule += 1;
    }
    let mut any_violation = false;
    if let Some(point) = checkpoints.cold_recover_with_fault {
        if point.chosen_root_abandoned {
            summary.violations_cold_recover_chosen_root_abandoned += 1;
            any_violation = true;
        }
        if point.content_off_timeline {
            summary.violations_cold_recover_content_off_timeline += 1;
            any_violation = true;
        }
    }
    if let Some(point) = checkpoints.mount_then_recover_with_fault {
        if point.chosen_root_abandoned {
            summary.violations_mount_then_recover_chosen_root_abandoned += 1;
            any_violation = true;
        }
        if point.content_off_timeline {
            summary.violations_mount_then_recover_content_off_timeline += 1;
            any_violation = true;
        }
    }
    if let Some(point) = checkpoints.recover_after_fault_removed {
        if point.chosen_root_abandoned {
            summary.violations_recover_after_fault_removed_chosen_root_abandoned += 1;
            any_violation = true;
        }
        if point.content_off_timeline {
            summary.violations_recover_after_fault_removed_content_off_timeline += 1;
            any_violation = true;
        }
    }
    if any_violation {
        summary.pairs_with_any_violation += 1;
        *summary.violations_by_weight.entry(weight).or_insert(0) += 1;
        if has_rollback {
            summary.violations_with_rollback += 1;
        } else {
            summary.violations_without_rollback += 1;
        }
    }
}

/// 从 `items`（已按 (区域, 槽) 升序）里取 `size` 个的全部组合。
fn combinations_of_size<Item: Clone>(items: &[Item], size: usize) -> Vec<Vec<Item>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    if items.len() < size {
        return Vec::new();
    }
    let mut result = Vec::new();
    for index in 0..=items.len() - size {
        let head = items[index].clone();
        for mut tail in combinations_of_size(&items[index + 1..], size - 1) {
            let mut combo = vec![head.clone()];
            combo.append(&mut tail);
            result.push(combo);
        }
    }
    result
}

#[allow(
    clippy::too_many_arguments,
    reason = "Q3-1 的 H3 穷举需要节点、剩余深度、mkfs 参数、几何、独立解出的几何视图与汇总器六项，各自有名字有类型"
)]
fn enumerate_fault_set_violations(
    node: &SimNode,
    depth_remaining: u64,
    parameters: &MakeFilesystemParameters,
    geometry: &Geometry,
    geometry_view: &PoolGeometry,
    summary: &mut FaultSetViolationSummary,
) {
    summary.nodes_visited += 1;
    let all_roots = readable_roots_with_slots(&node.pool, geometry_view);
    let slots: Vec<RootRingSlot> = all_roots
        .iter()
        .map(|(region, slot, _)| RootRingSlot {
            region: *region,
            slot: *slot,
        })
        .collect();
    let has_rollback = !node.rollback_events.is_empty();

    let baseline_chosen_root = Some(
        *node
            .timeline
            .last()
            .expect("H3 的每个节点时间线非空：bootstrap 至少写下 mkfs 根"),
    );
    // (a) 对整个节点只读一次原始镜像；下面每个故障集合的 (a) 都复用这同一份，不逐个重拷。
    let devices_for_cold_recover = devices_from_pool(&node.pool, IMAGE_BYTES);
    let baseline_target = RootRingSlotTarget {
        named_slots: NamedRootRingSlots::NONE,
        region_devices: region_devices(),
        fixed_structure_slot_spacing: 4096,
    };
    let baseline = evaluate_violation_checkpoints(
        node,
        parameters,
        geometry,
        baseline_target,
        baseline_chosen_root,
        &devices_for_cold_recover,
    );
    record_violation_checkpoints_for_one_fault_set(summary, 0, has_rollback, &baseline);

    for weight in 1..=2usize {
        for combo in combinations_of_size(&slots, weight) {
            let target = RootRingSlotTarget {
                named_slots: NamedRootRingSlots::naming(&combo),
                region_devices: region_devices(),
                fixed_structure_slot_spacing: 4096,
            };
            let checkpoints = evaluate_violation_checkpoints(
                node,
                parameters,
                geometry,
                target,
                baseline_chosen_root,
                &devices_for_cold_recover,
            );
            record_violation_checkpoints_for_one_fault_set(
                summary,
                u32::try_from(weight).expect("weight ∈ {1,2}"),
                has_rollback,
                &checkpoints,
            );
        }
    }

    if depth_remaining == 0 {
        return;
    }
    if node.session.is_some() {
        if let Ok(next) = apply_overwrite(node, parameters) {
            enumerate_fault_set_violations(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                geometry_view,
                summary,
            );
        }
    }
    if let Ok(next) = apply_mount_writable(node, parameters) {
        enumerate_fault_set_violations(
            &next,
            depth_remaining - 1,
            parameters,
            geometry,
            geometry_view,
            summary,
        );
    }
    for readable_root in &readable_roots_independent(&node.pool, geometry_view) {
        let target = RollbackTarget {
            instance: InstanceGeneration(readable_root.0),
            checkpoint_txg: CheckpointTxg(readable_root.1),
        };
        if let Ok(next) = apply_mount_rollback(node, target, parameters) {
            enumerate_fault_set_violations(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                geometry_view,
                summary,
            );
        }
    }
}

fn run_fault_set_violation_family(
    geometry: &Geometry,
    initial_overwrite_counts: &[u64],
    sigma_length_limit: u64,
) -> FaultSetViolationSummary {
    let parameters = parameters_for(geometry);
    let mut summary = FaultSetViolationSummary::default();
    for &overwrites_before_sigma in initial_overwrite_counts {
        let root_node = match bootstrap(geometry, overwrites_before_sigma) {
            Ok(node) => node,
            Err(error) => {
                emit_result(&format!(
                    "name=fault_set_violation_bootstrap_failed geometry={} overwrites_before_sigma={overwrites_before_sigma} detail={error:?}",
                    geometry.label
                ));
                continue;
            }
        };
        let geometry_view = match independent_geometry(&root_node.pool) {
            Ok(view) => view,
            Err(error) => {
                emit_result(&format!(
                    "name=fault_set_violation_geometry_failed geometry={} detail={error:?}",
                    geometry.label
                ));
                continue;
            }
        };
        enumerate_fault_set_violations(
            &root_node,
            sigma_length_limit,
            &parameters,
            geometry,
            &geometry_view,
            &mut summary,
        );
        emit_result(&format!(
            "name=fault_set_violation_progress geometry={} overwrites_before_sigma={overwrites_before_sigma} nodes_visited_so_far={} pairs_so_far={}",
            geometry.label, summary.nodes_visited, summary.pairs
        ));
    }
    emit_result(&format!(
        "name=fault_set_violation_summary geometry={} overwrites_before_sigma={:?} sigma_length_limit={} nodes_visited={} pairs={} pairs_with_any_violation={} cold_recover_with_fault_failed={} mount_writable_with_fault_failed={} skipped_by_the_reduction_rule={}",
        geometry.label,
        initial_overwrite_counts,
        sigma_length_limit,
        summary.nodes_visited,
        summary.pairs,
        summary.pairs_with_any_violation,
        summary.cold_recover_with_fault_failed,
        summary.mount_writable_with_fault_failed,
        summary.skipped_by_the_reduction_rule
    ));
    for (weight, pairs) in &summary.pairs_by_weight {
        let violations = summary
            .violations_by_weight
            .get(weight)
            .copied()
            .unwrap_or(0);
        emit_result(&format!(
            "name=fault_set_violation_by_weight geometry={} weight={weight} pairs={pairs} violations={violations}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=fault_set_violation_by_rollback_presence geometry={} pairs_with_rollback={} violations_with_rollback={} pairs_without_rollback={} violations_without_rollback={}",
        geometry.label,
        summary.pairs_with_rollback,
        summary.violations_with_rollback,
        summary.pairs_without_rollback,
        summary.violations_without_rollback
    ));
    emit_result(&format!(
        "name=fault_set_violation_by_checkpoint_and_predicate geometry={} cold_recover_chosen_root_abandoned={} cold_recover_content_off_timeline={} mount_then_recover_chosen_root_abandoned={} mount_then_recover_content_off_timeline={} recover_after_fault_removed_chosen_root_abandoned={} recover_after_fault_removed_content_off_timeline={}",
        geometry.label,
        summary.violations_cold_recover_chosen_root_abandoned,
        summary.violations_cold_recover_content_off_timeline,
        summary.violations_mount_then_recover_chosen_root_abandoned,
        summary.violations_mount_then_recover_content_off_timeline,
        summary.violations_recover_after_fault_removed_chosen_root_abandoned,
        summary.violations_recover_after_fault_removed_content_off_timeline
    ));
    summary
}

// ============================================================================
// 十二、Q1（岔路单第 1 行，C393）：H1 家族 + Φ1 故障注入 + 候选 (a)(b)(c) 判定。
//
// Φ1 的定位不靠幂集：对每条被抛弃的、根槽读得出的根 a，各取它的两个单元——树表单元（根记录
// 直接指着的）与分配记录树节点（树表非 0 条时是树表里 TREE_KIND_ALLOCATION 那一条目指的根；
// 树表 0 条时是根记录自己的 `allocation_record_tree_root`）——这两个单元的两条物理位置从
// `crates/singlefs-core` 的公开 API 现查得到（`RootRecord::parse_slot`、`tree_table_has_no_entries`、
// `parse_index_node`、`TreeTableEntry::parse`，见交回报告第 2.2 节），不猜偏移。
// ============================================================================

/// 一条位置条目的去重键：(设备号, 槽号)。
fn location_key(location: &LocationEntry) -> (u32, u64) {
    (location.device.0, location.slot.0)
}

/// 从孪生（未注入）镜像上，按 (实例, txg) 取回完整的 `RootRecord`（要它的 `tree_table` /
/// `allocation_record_tree_root` 两条指针，`RootView` 没有暴露它们）。**不能**直接拿
/// `RootView::record_bytes`（`checker_image::valid_roots` 截到 457 字节）喂给 `RootRecord::parse_slot`：
/// 自证校验和是按整槽宽（`geometry.physical_block_size`，本装置是 512）算的，截短之后再解，
/// `wide_checksum_field_holds` 拿 457 当整槽宽重算，跟原校验和对不上，`parse_slot` 会稳定交回 `None`
/// （这是这一段调试打中的一个坑，交回报告点名）。改成按 `checker_image::root_slot_positions` 给的
/// (区域, 槽, 设备, 偏移) 自己重新整槽读一遍——与 `valid_roots` 内部读的是同一段字节、同一个宽度。
fn root_record_of(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    target: TimelineRoot,
) -> Option<RootRecord> {
    let slot_bytes = usize::try_from(geometry.physical_block_size).ok()?;
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    for (_region, _slot, device_number, offset) in checker_image::root_slot_positions(geometry) {
        let identity = DeviceIdentity(device_number);
        let Some((_, device)) = devices.iter().find(|(candidate, _)| *candidate == identity) else {
            continue;
        };
        let mut buffer = vec![0u8; slot_bytes];
        if device
            .read_at(DeviceOffsetInBytes(offset), &mut buffer)
            .is_err()
        {
            continue;
        }
        if let Some(record) = RootRecord::parse_slot(&buffer, &FILESYSTEM_IDENTIFIER) {
            if (record.instance.0, record.checkpoint_txg.0) == target {
                return Some(record);
            }
        }
    }
    None
}

/// 按一条指针的两条位置条目，从**未注入**的装置去读整份节点字节（16384 字节，`NODE_BYTES`）；
/// 两份里随便一份读得出就交回，两份都读不出交 `None`。只用于发现物理位置，不参与判据本身。
fn read_node_bytes(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    locations: &[LocationEntry; 2],
) -> Option<Vec<u8>> {
    for location in locations {
        if let Some((_, device)) = devices
            .iter()
            .find(|(identity, _)| *identity == location.device)
        {
            let mut buffer = vec![0u8; 16384];
            if device
                .read_at(location.slot.to_device_offset(), &mut buffer)
                .is_ok()
            {
                return Some(buffer);
            }
        }
    }
    None
}

/// 分配记录树节点（指称二）的两条物理位置：树表 0 条时是根记录自己的 `allocation_record_tree_root`；
/// 否则要先读树表节点、找 `TREE_KIND_ALLOCATION` 那一条目。走的是 `crates/singlefs-core::recovery`
/// 与 `records` 两个模块的公开函数，不碰任何 `pub(crate)` 内部函数（交回报告第 2.2 节的侦察结论）。
#[allow(
    clippy::ptr_arg,
    reason = "tree_table_has_no_entries 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn（与 crates/singlefs-core/src/mount.rs 的 placements_referenced_by_root 同一条理由）"
)]
fn locate_allocation_record_tree(
    devices: &Vec<(DeviceIdentity, SparseBlockDevice)>,
    root: &RootRecord,
) -> Result<[LocationEntry; 2], String> {
    match tree_table_has_no_entries(devices, root) {
        Ok(true) => Ok(root.allocation_record_tree_root.locations),
        Ok(false) => {
            let node_bytes = read_node_bytes(devices, &root.tree_table.locations)
                .ok_or_else(|| "树表两份都读不出".to_string())?;
            let tree_table =
                parse_index_node(&node_bytes).map_err(|error| format!("树表解不开: {error:?}"))?;
            for entry_bytes in &tree_table.entries {
                if let Some(entry) = TreeTableEntry::parse(entry_bytes) {
                    if entry.kind == TREE_KIND_ALLOCATION {
                        return Ok(entry.root.locations);
                    }
                }
            }
            Err("树表里没有 TREE_KIND_ALLOCATION 条目".to_string())
        }
        Err(error) => Err(format!("tree_table_has_no_entries: {error:?}")),
    }
}

/// 「共享」标记（5.1 原文）：这个单元的两条位置键，是否也被某条**没被抛弃**的可读根引用
/// （树表指针或分配记录树指针任一个落在同一 (设备, 槽) 上）。H1 的基础两种 op1（`mount_writable`、
/// `mount_rollback`）都不抬 F，候选集恒等于「全部可读根」，因此这里不用另读回退下界。
#[allow(
    clippy::ptr_arg,
    reason = "转给 locate_allocation_record_tree，它要 &dyn PoolReader、要一个有大小的读者"
)]
fn is_unit_shared(
    devices: &Vec<(DeviceIdentity, SparseBlockDevice)>,
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    unit_keys: &BTreeSet<(u32, u64)>,
) -> bool {
    for candidate in readable_roots_independent(pool, geometry) {
        if candidate == exclude {
            continue;
        }
        let Some(record) = root_record_of(pool, geometry, candidate) else {
            continue;
        };
        let tree_keys: BTreeSet<(u32, u64)> = record
            .tree_table
            .locations
            .iter()
            .map(location_key)
            .collect();
        if !tree_keys.is_disjoint(unit_keys) {
            return true;
        }
        if let Ok(allocation_locations) = locate_allocation_record_tree(devices, &record) {
            let allocation_keys: BTreeSet<(u32, u64)> =
                allocation_locations.iter().map(location_key).collect();
            if !allocation_keys.is_disjoint(unit_keys) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LedgerAspect {
    TreeTable,
    AllocationRecordTree,
}

impl LedgerAspect {
    fn label(self) -> &'static str {
        match self {
            LedgerAspect::TreeTable => "tree_table",
            LedgerAspect::AllocationRecordTree => "allocation_record_tree",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultSeverity {
    Empty,
    Both,
    Disk0,
    Disk1,
}

impl FaultSeverity {
    fn label(self) -> &'static str {
        match self {
            FaultSeverity::Empty => "empty",
            FaultSeverity::Both => "both_copies",
            FaultSeverity::Disk0 => "disk0_only",
            FaultSeverity::Disk1 => "disk1_only",
        }
    }

    /// 故障数（5.1 表：两份都读失败 = 2 个故障，只一份 = 1 个，空集 = 0）。
    fn fault_count(self) -> u64 {
        match self {
            FaultSeverity::Empty => 0,
            FaultSeverity::Both => 2,
            FaultSeverity::Disk0 | FaultSeverity::Disk1 => 1,
        }
    }
}

/// 按严重度把一个单元的两条位置条目变成注入目标（(设备, 偏移) 表）。
fn fault_targets_for(
    locations: &[LocationEntry; 2],
    severity: FaultSeverity,
) -> Vec<(DeviceIdentity, DeviceOffsetInBytes)> {
    match severity {
        FaultSeverity::Empty => Vec::new(),
        FaultSeverity::Both => locations
            .iter()
            .map(|location| (location.device, location.slot.to_device_offset()))
            .collect(),
        FaultSeverity::Disk0 => locations
            .iter()
            .filter(|location| location.device == DeviceIdentity(0))
            .map(|location| (location.device, location.slot.to_device_offset()))
            .collect(),
        FaultSeverity::Disk1 => locations
            .iter()
            .filter(|location| location.device == DeviceIdentity(1))
            .map(|location| (location.device, location.slot.to_device_offset()))
            .collect(),
    }
}

#[derive(Clone, Debug)]
#[allow(
    clippy::enum_variant_names,
    reason = "三个变体都是「哪一种挂载/抬 F 操作」，共享 Mount 前缀是命名准确，不是同一个词重复"
)]
enum FaultedOperationKind {
    MountWritable,
    MountRollbackTo(TimelineRoot),
    /// op1 第三种变体（跑前登记 5.1 H1 行）：先不注入地做一次 `mount_writable`（只为打开会话，
    /// 不是受注入的一步），再把这个目标 floor 值喂给受注入的 `raise_rollback_floor`。
    MountWritableThenRaiseFloorTo(CheckpointTxg),
}

impl FaultedOperationKind {
    fn label(&self) -> String {
        match self {
            FaultedOperationKind::MountWritable => "mount_writable".to_string(),
            FaultedOperationKind::MountRollbackTo(target) => format!("mount_rollback({target:?})"),
            FaultedOperationKind::MountWritableThenRaiseFloorTo(floor) => {
                format!("mount_writable_then_raise_rollback_floor({floor:?})")
            }
        }
    }
}

/// 一次 op1 尝试的结局：`Ok` 时交回 `MountOutput::abandoned_roots_unreadable`；`Err` 时交回错误的
/// `Debug` 首词（跨臂只用字符串比较——A1 副本的 `MountError::AbandonedRootLedgerUnreadable` 在
/// 「今天」那份 `crates/` 里根本不存在这个变体，装置源码两边共用一份，不能直接 `match` 枚举名）。
/// 另外把这次调用设备一层写的字节数按盘记下（Q1-4：候选 (a) 被拒那次必须是 0）。
struct FaultedOperationAttempt {
    abandoned_roots_unreadable: Option<u64>,
    error_debug: Option<String>,
    error_member: Option<String>,
    device_write_bytes: BTreeMap<u32, u64>,
}

/// 从 `MountError` 的 `Debug` 输出里取枚举变体名（首个 `(`、`{` 或空格之前的那一段）。跨臂只靠
/// 字符串比较：A1 副本新增的 `AbandonedRootLedgerUnreadable` 变体在「今天」那份 `crates/` 里不存在，
/// 装置源码两边共用一份，不能写 `match MountError::AbandonedRootLedgerUnreadable { .. }`（编不过今天那份）。
fn error_member_of_debug(debug: &str) -> String {
    debug
        .split(['(', '{', ' '])
        .next()
        .unwrap_or(debug)
        .to_string()
}

fn attempt_faulted_operation(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fixed_geometry: FixedGeometry,
    kind: &FaultedOperationKind,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> FaultedOperationAttempt {
    // op1 第三种变体（session s10）：唯一一种需要在受注入的那一步**之前**先做一次完全不注入的
    // `mount_writable`（打开会话）的 op1，装置结构与另外两种不同，另开一个函数、在这里提前分派。
    if let FaultedOperationKind::MountWritableThenRaiseFloorTo(new_floor) = kind {
        return attempt_mount_writable_then_raise_floor(
            node,
            parameters,
            fixed_geometry,
            *new_floor,
            fault_targets,
        );
    }
    let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut plans: Vec<(DeviceIdentity, SharedFaultPlan)> = Vec::new();
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in plain {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        plans.push((identity, plan.clone()));
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    let outcome: Result<u64, String> = match kind {
        FaultedOperationKind::MountWritable => mount_writable(parameters, &mut wrapped)
            .map(|mounted| mounted.output.abandoned_roots_unreadable)
            .map_err(|error| format!("{error:?}")),
        FaultedOperationKind::MountRollbackTo(target) => mount_rollback(
            parameters,
            &mut wrapped,
            RollbackTarget {
                instance: InstanceGeneration(target.0),
                checkpoint_txg: CheckpointTxg(target.1),
            },
            ShadowLedger::On,
        )
        .map(|mounted| mounted.output.abandoned_roots_unreadable),
        FaultedOperationKind::MountWritableThenRaiseFloorTo(_) => {
            unreachable!("MountWritableThenRaiseFloorTo 在函数开头已经提前分派、提前 return 过了")
        }
    };
    let mut device_write_bytes = BTreeMap::new();
    for (identity, plan) in &plans {
        device_write_bytes.insert(identity.0, plan.counts_of_device(*identity).written_bytes);
    }
    match outcome {
        Ok(count) => FaultedOperationAttempt {
            abandoned_roots_unreadable: Some(count),
            error_debug: None,
            error_member: None,
            device_write_bytes,
        },
        Err(debug) => {
            let member = error_member_of_debug(&debug);
            FaultedOperationAttempt {
                abandoned_roots_unreadable: None,
                error_debug: Some(debug),
                error_member: Some(member),
                device_write_bytes,
            }
        }
    }
}

/// op1 第三种变体（跑前登记 H1 行「先不注入地 `mount_writable` 再 `raise_rollback_floor` 到
/// [F+1, 上限] 里每一个值」）：floor 目标枚举成 (F, 上限] 半开区间——`current_floor` 本身不许再抬
/// （抬到自己等于没抬），`ceiling` 一定要能抬到。`ceiling <= current_floor` 时天然给出空区间
/// （Rust 的 `..=` 起点大于终点时不产出任何元素），不用另写一条判空分支。
fn floor_targets_between(
    current_floor: CheckpointTxg,
    ceiling: CheckpointTxg,
) -> Vec<CheckpointTxg> {
    ((current_floor.0 + 1)..=ceiling.0)
        .map(CheckpointTxg)
        .collect()
}

/// op1 第三种变体的目标 floor 枚举：先不注入地做一次 `mount_writable`（试探性，用完即弃，只为量出
/// 这一步的 F 与上限——它本身不是受注入的一步），从它交回的 `current.root.rollback_floor` 起算 F，
/// 再用 `crates/` 已有的 `rollback_floor_ceiling`（与 `raise_rollback_floor` 内部用的是同一个函数）
/// 算出上限。交回空 `Vec`：探测步骤失败（挂载不成功、没有文件、系统配置或实例表读不出、
/// `rollback_floor_ceiling` 报错），或 F 已经等于上限（没有余量可抬）——两种都不是「该测到却漏了」，
/// 是这条历史在这一格没有 op1 第三种变体可跑，调用方按 `raise_floor_history_nodes_without_room`
/// 单独计一次，不归进任何触发计数。
fn raise_floor_targets_for(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> Vec<CheckpointTxg> {
    let mut probe_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let Ok(mounted) = mount_writable(parameters, &mut probe_devices) else {
        return Vec::new();
    };
    let PoolVersion::WithFile(current) = mounted.current else {
        return Vec::new();
    };
    let Ok(system_configuration) = choose_system_configuration(&probe_devices) else {
        return Vec::new();
    };
    let Ok(table) = instance_table_chain_of_root(&probe_devices, &current.root) else {
        return Vec::new();
    };
    let current_floor = current.root.rollback_floor;
    let Ok(ceiling) = rollback_floor_ceiling(
        &probe_devices,
        &system_configuration,
        current_floor,
        &table.records,
        &ring_slots_known_to_hold_a_root_by(&mounted.allocator),
    ) else {
        return Vec::new();
    };
    floor_targets_between(current_floor, ceiling)
}

/// op1 第三种变体的「试一次看结局，用完即弃」实现：第一步（打开会话）**不注入**——`raise_rollback_
/// floor` 要 `&mut PoolAllocator`/`&mut TransactionOutput`（一个开着的会话），而这里进来的 `node`
/// 已经「关闭」（`session: None`），受注入的一步只是 `raise_rollback_floor` 本身（跑前登记 5.1 H1
/// 行「先不注入地 `mount_writable` 再 `raise_rollback_floor`」）。**不追加进 `node.timeline`**：
/// `raise_rollback_floor` 交回的 `RaisedFloor::abandoned_roots_unreadable` 是从盘上现读实例表 +
/// 现行根的候选集算出来的（`isolate_slots_referenced_only_by_abandoned_roots`，与
/// `mount_writable`/`mount_rollback` 那两种 op1 共用同一条计数管道，`crates/singlefs-core/src/
/// mount.rs` 第 1027 行起），不依赖装置自己的内存时间线；与 `MountRollbackTo` 分支同理——那个分支
/// 交回的 `mounted.output.warm_up_publishes` 等中间根同样只用来读它的最终 `abandoned_roots_
/// unreadable` 字段，从不追加进任何时间线（`attempt_faulted_operation` 整体是「试一次看结局，用完
/// 即弃」，不产出可继续使用的 `SimNode`）。两步用的字节不共享（`devices_from_pool` 每次都深拷贝，
/// 第一步之后的 `probe_devices` 直接喂给第二步的包装层，不再回读 `node.pool`），故障只出现在第二
/// 步的 `wrapped`。
fn attempt_mount_writable_then_raise_floor(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fixed_geometry: FixedGeometry,
    new_floor: CheckpointTxg,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> FaultedOperationAttempt {
    let mut probe_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = match mount_writable(parameters, &mut probe_devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            return FaultedOperationAttempt {
                abandoned_roots_unreadable: None,
                error_debug: Some(format!("先不注入地 mount_writable 失败：{error:?}")),
                error_member: Some("PrecedingUnfaultedMountWritableFailed".to_string()),
                device_write_bytes: BTreeMap::new(),
            };
        }
    };
    let PoolVersion::WithFile(mut current) = mounted.current else {
        return FaultedOperationAttempt {
            abandoned_roots_unreadable: None,
            error_debug: Some("先不注入地 mount_writable 交回的现行版本没有文件".to_string()),
            error_member: Some("PrecedingUnfaultedMountWritableWithoutFile".to_string()),
            device_write_bytes: BTreeMap::new(),
        };
    };
    let mut allocator = mounted.allocator;
    let mut plans: Vec<(DeviceIdentity, SharedFaultPlan)> = Vec::new();
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in probe_devices {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        plans.push((identity, plan.clone()));
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    let raised = raise_rollback_floor(
        parameters,
        &mut wrapped,
        &mut allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    let mut device_write_bytes = BTreeMap::new();
    for (identity, plan) in &plans {
        device_write_bytes.insert(identity.0, plan.counts_of_device(*identity).written_bytes);
    }
    match raised {
        Ok(raised) => FaultedOperationAttempt {
            abandoned_roots_unreadable: Some(raised.abandoned_roots_unreadable),
            error_debug: None,
            error_member: None,
            device_write_bytes,
        },
        Err(error) => {
            let debug = format!("{error:?}");
            let member = error_member_of_debug(&debug);
            FaultedOperationAttempt {
                abandoned_roots_unreadable: None,
                error_debug: Some(debug),
                error_member: Some(member),
                device_write_bytes,
            }
        }
    }
}

/// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 1 行还差项④）：op1 起的多次挂载轨迹——
/// 第一节读法写死表「持续」＝从 op1 这次调用起到这段历史结束都有效，「瞬时」＝只在 op1 那次调用期间
/// 有效；第八节 8.1「abandoned_roots_unreadable，按 op1 起的每一次挂载」要的轨迹正是这个函数产出的。
/// **与 `attempt_faulted_operation` 的关键差别**：那个函数每次都从 `node.pool` 重新
/// `devices_from_pool`，故障目标之外的写不回原池（`SparseDevice` 是普通值类型、`.clone()` 深拷贝，
/// 不共享底层字节，这是刻意的「试一次看结局」语义，见它的用法）；这里要的是「op1 落盘之后接着挂」，
/// 所以每一步显式把 `FaultInjectingBlockDevice::inner().image` 取出来拼回一个新的 `MemoryPool`，
/// 喂给下一步——**这是这个函数与 `attempt_faulted_operation` 的唯一结构性差别，其余装故障、判结局的
/// 写法逐字照抄，不引入第二套注入逻辑**。`persistent = true` 时每一步都用同一组 `fault_targets`
/// （对应「持续」）；`persistent = false` 时只有第一步（op1 自己）带故障，之后的步不装任何故障
/// （对应「瞬时」，且与「瞽时」的读法一致：故障在 op1 那次调用返回后即撤）。某一步交回 `Err` 就停在
/// 那一步，`results` 里这一步与之后的步都记 `None`（挂载失败之后没有新的池状态可以接着挂，不能凭空
/// 编一个「本该」发生的计数）。
fn mount_writable_trajectory(
    starting_pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    fixed_geometry: FixedGeometry,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
    persistent: bool,
    steps: usize,
) -> Vec<Option<u64>> {
    let mut pool = starting_pool.clone();
    let mut results = Vec::new();
    for step in 0..steps {
        let active_targets: &[(DeviceIdentity, DeviceOffsetInBytes)] = if step == 0 || persistent {
            fault_targets
        } else {
            &[]
        };
        let plain = devices_from_pool(&pool, IMAGE_BYTES);
        let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
            Vec::new();
        for (identity, device) in plain {
            let plan = SharedFaultPlan::unarmed(fixed_geometry);
            if let Some((_, offset)) = active_targets
                .iter()
                .find(|(target_device, _)| *target_device == identity)
            {
                plan.arm(FaultSchedule {
                    fault: InjectedFault::ReadFails,
                    device: FaultDeviceSelector::OnlyDevice(identity),
                    placement: FaultPlacement::OffsetExactly(*offset),
                    counting: FaultCounting::AcrossThePool,
                    occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
                });
            }
            wrapped.push((
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan),
            ));
        }
        match mount_writable(parameters, &mut wrapped) {
            Ok(mounted) => {
                results.push(Some(mounted.output.abandoned_roots_unreadable));
                let next_devices: BTreeMap<DeviceIdentity, SparseDevice> = wrapped
                    .iter()
                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                    .collect();
                pool = MemoryPool {
                    devices: next_devices,
                    device_size_in_bytes: IMAGE_BYTES,
                };
            }
            Err(_) => {
                results.push(None);
                break;
            }
        }
    }
    while results.len() < steps {
        results.push(None);
    }
    results
}

/// H1 家族里的一个「op1 之前」节点：mkfs → `mount_writable` → 首个文件 →
/// `overwrites_before_rollback` 次覆盖写 → 关闭 → `mount_rollback` 到 `rollback_target` →
/// `overwrites_after_rollback` 次覆盖写 → 关闭。op1 本身（受注入的一步）不在这个结构体里，
/// 由调用方按 `FaultedOperationKind` 逐个尝试。
struct LedgerFaultHistoryNode {
    node: SimNode,
    overwrites_before_rollback: u64,
    rollback_target: TimelineRoot,
    overwrites_after_rollback: u64,
}

/// H1 全族逐一枚举（不抽样）：`overwrites_before_rollback` ∈ {1,2,3}；`rollback_target` 取那一刻每一条
/// 可读根（按 `BTreeSet` 升序，固定次序）；`overwrites_after_rollback` ∈ {0,1,2}。回退到某个候选被拒的
/// 分支到此为止，计入 `rejected_rollback_attempts`（不是 H1 节点）。
struct LedgerFaultHistoryFamily {
    nodes: Vec<LedgerFaultHistoryNode>,
    rejected_rollback_attempts: u64,
}

fn ledger_fault_history_family(geometry: &Geometry) -> LedgerFaultHistoryFamily {
    let parameters = parameters_for(geometry);
    let mut nodes = Vec::new();
    let mut rejected_rollback_attempts = 0u64;
    for overwrites_before_rollback in [1u64, 2, 3] {
        let Ok(base) = bootstrap(geometry, overwrites_before_rollback) else {
            continue;
        };
        let Ok(base_geometry) = independent_geometry(&base.pool) else {
            continue;
        };
        let candidates: Vec<TimelineRoot> = readable_roots_independent(&base.pool, &base_geometry)
            .into_iter()
            .collect();
        for rollback_target in candidates {
            let target = RollbackTarget {
                instance: InstanceGeneration(rollback_target.0),
                checkpoint_txg: CheckpointTxg(rollback_target.1),
            };
            match apply_mount_rollback(&base, target, &parameters) {
                Ok(after_rollback) => {
                    let mut current = after_rollback;
                    for overwrites_after_rollback in [0u64, 1, 2] {
                        nodes.push(LedgerFaultHistoryNode {
                            node: current.clone(),
                            overwrites_before_rollback,
                            rollback_target,
                            overwrites_after_rollback,
                        });
                        if overwrites_after_rollback < 2 {
                            match apply_overwrite(&current, &parameters) {
                                Ok(next) => current = next,
                                Err(_) => break,
                            }
                        }
                    }
                }
                Err(_) => rejected_rollback_attempts += 1,
            }
        }
    }
    LedgerFaultHistoryFamily {
        nodes,
        rejected_rollback_attempts,
    }
}

/// 分配记录树节点带同一组故障走一遍（候选 (b) 分配记录树指称，Q1-2）：交回 `(设备, 槽, 已释放)` 的原始
/// 列表；调用方按 `!released` 过滤成「未释放落点集合」再比对。
fn walk_allocation_records_with_fault(
    node: &SimNode,
    fixed_geometry: FixedGeometry,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
    root: &RootRecord,
) -> Result<Vec<(u32, u64, bool)>, String> {
    let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in plain {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    allocation_records_under_root(&wrapped, root)
        .map(|records| {
            records
                .into_iter()
                .map(|record| (record.device.0, record.slot.0, record.is_released))
                .collect()
        })
        .map_err(|error| format!("{error:?}"))
}

/// 树表指称候选 (b)「有副本」：在孪生（未注入）镜像上，别的可读根的树表节点字节里有没有一份与
/// `target_bytes` 逐字节相同的。「crates 有路」（调 `rebuild_version`/`replay_journal`）这一子分支
/// 这一段没有实现，见交回报告——「有副本」为真已经够支持 (b) 在这一格可重建，「有副本」为假时
/// 报「无来源（只测过有副本这一支）」，不等价于「crates 也无路」。
fn tree_table_has_copy_elsewhere(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    target_bytes: &[u8],
) -> bool {
    for candidate in readable_roots_independent(pool, geometry) {
        if candidate == exclude {
            continue;
        }
        if let Some(record) = root_record_of(pool, geometry, candidate) {
            if let Some(bytes) = read_node_bytes(plain_devices, &record.tree_table.locations) {
                if bytes == target_bytes {
                    return true;
                }
            }
        }
    }
    false
}

/// 找被抛弃根在同一实例上、checkpoint_txg 更小、且今天仍读得出的那一条根——候选 (b) 树表指称
/// 「crates 有路」子分支要从它出发（跑前登记 5.2 ②）。取 checkpoint_txg 最大的那一条（最近的祖先）。
fn nearest_older_readable_root_on_same_instance(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    target: &RootRecord,
) -> Option<(TimelineRoot, RootRecord)> {
    let mut best: Option<(TimelineRoot, RootRecord)> = None;
    for candidate in readable_roots_independent(pool, geometry) {
        if candidate == exclude {
            continue;
        }
        let Some(record) = root_record_of(pool, geometry, candidate) else {
            continue;
        };
        if record.instance != target.instance || record.checkpoint_txg >= target.checkpoint_txg {
            continue;
        }
        let take_it = match &best {
            None => true,
            Some((_, current_best)) => record.checkpoint_txg > current_best.checkpoint_txg,
        };
        if take_it {
            best = Some((candidate, record));
        }
    }
    best
}

/// 候选 (b) 树表指称「crates 有路」子分支的结局（跑前登记 5.2 ②）。**这一支给出的是指针级别（位置 +
/// 校验和）的复算，不是节点内容的 16 KiB 字节本身**——要拿到字节仍要照这个指针再读一次两份物理拷贝，
/// 而那正是这一格要绕开的读；这个限定在交回报告里写清楚，不写成「crates 有路 = 给得出字节」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeTableCratesHasAPathOutcome {
    /// 没有更旧的可读同实例根：这条路径的前提不成立。
    NoOlderReadableRootOnTimeline,
    /// 装置自己解不出系统配置（独立扫描 journal 记录要用它），走不下去。
    CannotDecodeSystemConfiguration,
    /// replay 交回的根没有推进到目标的 checkpoint_txg：链断在中途（可能正是因为验证撞上了被注入的物理位置）。
    DidNotReachTarget,
    /// replay 推进到了目标的 checkpoint_txg，但交回的 `tree_table` 指针与目标不同——crates 这条路给出的答案本身不对。
    ReachedTargetButPointerDiffers,
    /// replay 推进到了目标的 checkpoint_txg，且交回的 `tree_table` 指针（位置 + 校验和）与目标逐字段相同。
    ReachedTargetWithMatchingPointer,
}

impl TreeTableCratesHasAPathOutcome {
    fn label(self) -> &'static str {
        match self {
            TreeTableCratesHasAPathOutcome::NoOlderReadableRootOnTimeline => {
                "no_older_readable_root_on_timeline"
            }
            TreeTableCratesHasAPathOutcome::CannotDecodeSystemConfiguration => {
                "cannot_decode_system_configuration"
            }
            TreeTableCratesHasAPathOutcome::DidNotReachTarget => "did_not_reach_target",
            TreeTableCratesHasAPathOutcome::ReachedTargetButPointerDiffers => {
                "reached_target_but_pointer_differs"
            }
            TreeTableCratesHasAPathOutcome::ReachedTargetWithMatchingPointer => {
                "reached_target_with_matching_pointer"
            }
        }
    }
}

/// 候选 (b) 树表指称「crates 有路」子分支（跑前登记 5.2 ②）：从同一实例上更旧的可读根出发，调
/// `crates/` 已有的 `replay_journal`，看它能不能不读目标那两份物理拷贝就交回同一个 `tree_table` 指针。
/// `verify_named_units` 两种都跑：`true` 走今天挂载真用的口径（逐项验证点名单元，可能因此撞上同一个
/// 被注入的物理位置）；`false` 只信记录自带的新根段字段、不去验证任何单元字节。两者的差异本身就是
/// 「这条路独不独立于那次被挡住的读」这句话的答案。
fn tree_table_crates_has_a_path(
    node: &SimNode,
    fixed_geometry: FixedGeometry,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    target: &RootRecord,
    verify_named_units: bool,
) -> TreeTableCratesHasAPathOutcome {
    let Some((_, older_root)) =
        nearest_older_readable_root_on_same_instance(&node.pool, geometry, exclude, target)
    else {
        return TreeTableCratesHasAPathOutcome::NoOlderReadableRootOnTimeline;
    };
    let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let Ok(system_configuration) = choose_system_configuration(&plain_devices) else {
        return TreeTableCratesHasAPathOutcome::CannotDecodeSystemConfiguration;
    };
    // 记录本身不在这一格的故障目标里（Φ1 只挡树表或分配记录树那一个单元），按不注入的孪生镜像扫出全量——
    // 独立扫描 journal 不是这一支要考的东西，考的是 replay 拿到记录之后还要不要碰目标那份物理拷贝。
    let records = scan_journal(&plain_devices, &system_configuration);
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in devices_from_pool(&node.pool, IMAGE_BYTES) {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    let (_report, replayed_root) = replay_journal(
        &wrapped,
        &older_root,
        geometry.journal_ring_bytes,
        &records,
        verify_named_units,
    )
    .expect("装置里的记录都由写者写出，所选根那次发布只有一条带末条标志：锚点认得出");
    if replayed_root.instance != target.instance
        || replayed_root.checkpoint_txg != target.checkpoint_txg
    {
        return TreeTableCratesHasAPathOutcome::DidNotReachTarget;
    }
    if replayed_root.tree_table == target.tree_table {
        TreeTableCratesHasAPathOutcome::ReachedTargetWithMatchingPointer
    } else {
        TreeTableCratesHasAPathOutcome::ReachedTargetButPointerDiffers
    }
}

/// 候选 (b) 分配记录树指称「从内容树反推占用集合」这一种操作化（跑前登记 5.2 ②，session s6 修订项 5
/// 提出的思路）：不重读被注入的那一个分配记录树节点本身，改走中央映射树——它是这一版的逻辑到物理的
/// 权威索引（D1（数据可移动性 / 反向索引）），每条条目携带两条独立于分配记录树的位置条目。对第一版这种
/// 没有快照、只有一个活头的简单历史，中央映射树里出现过的落点集合就应当等于「未释放」的落点集合。
/// **射程**：只走了中央映射树一条内容树（`crates::mounted_read::open_pool_for_read` 走读 extent / inode
/// 树是为了服务文件读，与「占用集合」这个问题不是同一件事——占用靠中央映射，不靠 extent/inode）；
/// 有快照、多版本共享落点的情形没有覆盖（第一版这批历史都不产生快照，见交回报告）。
///
/// **只读一层、把根当叶解**——实十九（2026-09-24 16:50 UTC 前后落地）之后，中央映射树条目数一旦
/// 超过单节点叶容量（`transaction.rs` 第 1334 行「中央映射树叶 294」）就会长成多层，那时根节点是
/// 内部节点（`level > 0`），装着的是子节点指针、不是 `parse_mapping_entry` 认得的叶条目格式。
/// 这里显式核 `level == 0` 再往下解，不靠「宽度不够、`parse_mapping_entry` 自然读不出」这种隐式失败
/// 兜底——内部条目宽约 114 字节（`transaction.rs` 同一行「内部 143」按 16384 / 143 反推），比
/// `MAPPING_ENTRY_BYTES`=55 宽，`parse_mapping_entry` 的长度检查拦不住它，会把内部指针字节当叶
/// 条目误解出一对看似合法的 `LocationEntry`——这正是「不许静默算错」要挡的那种失败，`level` 检查把
/// 它变成一条会报错、不会算错的路。E158 这批历史全部只写一个文件、只有一个逻辑映射 key（跨 n1/m/σ
/// 的全部覆盖写都在原地更新同一条条目，不新增条目），条目数最多到 1（见交回报告的现场核对），远低于
/// 294，这批历史至今不会撞上多层——但装置不该靠这条事实免检，`level` 一变就该显式报错，不是碰巧躲过。
///
/// # Errors
/// 中央映射树根两份都读不出、解不开，或根节点 `level != 0`（长成了多层，这个函数还不会整棵读）。
fn allocation_record_tree_reachable_placements_via_central_mapping(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
) -> Result<BTreeSet<(u32, u64)>, String> {
    let mapping_root_bytes = read_node_bytes(plain_devices, &root.mapping_root.locations)
        .ok_or_else(|| "中央映射树根两份都读不出".to_string())?;
    let mapping_root = parse_index_node(&mapping_root_bytes)
        .map_err(|error| format!("中央映射树根解不开: {error:?}"))?;
    if mapping_root.level != 0 {
        return Err(format!(
            "中央映射树根不是叶（level={}，entries={}）：树已经长成多层，这个函数只会整棵读单叶版本，不认内部节点条目",
            mapping_root.level,
            mapping_root.entries.len()
        ));
    }
    let mut placements = BTreeSet::new();
    for entry_bytes in &mapping_root.entries {
        let (_key, locations) =
            parse_mapping_entry(entry_bytes).ok_or_else(|| "中央映射条目解不开".to_string())?;
        for location in locations {
            placements.insert((location.device.0, location.slot.0));
        }
    }
    Ok(placements)
}

/// 候选 (b) 分配记录树指称「从内容树反推占用集合」走全（session s8，跑前登记 5.2 ②「它答不了的」
/// 缺口①）：在上一个函数（只走中央映射树这一条内容树）的基础上，把「码 2」结构节点自己的落点也并进
/// 占用集合——`AllocationRecord`（`crates/singlefs-core/src/allocator.rs` 第 31 行）记的是**任意
/// 已分配的跨度**，不只是中央映射树指向的数据单元；树表自己、树表里 `TREE_KIND_INODE` /
/// `TREE_KIND_EXTENT` / `TREE_KIND_ACCOUNTING` 三种子树各自的根节点、以及中央映射树**自己**的根节点
/// （区别于它指向的数据单元），都是「码 2 节点取最低空槽」（`allocator.rs` 第 6 行）分配出来的、理应
/// 出现在「未释放」集合里的落点。`TREE_KIND_ALLOCATION` 不并进来——那是分配记录树自己，并进来是把
/// 要验证的账当成账的输入，循环论证。**这仍不是穷举**：`TREE_KIND_LIVELIST` / `_SPARSE_SIDE_TABLE` /
/// `_DEADLIST` 三种树表里可能存在的种类没有并入（第一版这批历史是否会产生它们、要不要算，留给下一段）。
fn allocation_record_tree_reachable_placements_via_all_structural_trees(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
) -> Result<BTreeSet<(u32, u64)>, String> {
    let mut placements =
        allocation_record_tree_reachable_placements_via_central_mapping(plain_devices, root)?;
    for location in &root.mapping_root.locations {
        placements.insert((location.device.0, location.slot.0));
    }
    for location in &root.tree_table.locations {
        placements.insert((location.device.0, location.slot.0));
    }
    let tree_table_bytes = read_node_bytes(plain_devices, &root.tree_table.locations)
        .ok_or_else(|| "树表两份都读不出".to_string())?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|error| format!("树表解不开: {error:?}"))?;
    for entry_bytes in &tree_table.entries {
        let Some(entry) = TreeTableEntry::parse(entry_bytes) else {
            continue;
        };
        if matches!(
            entry.kind,
            TREE_KIND_INODE | TREE_KIND_EXTENT | TREE_KIND_ACCOUNTING
        ) {
            for location in &entry.root.locations {
                placements.insert((location.device.0, location.slot.0));
            }
        }
    }
    Ok(placements)
}

/// session s9（跑前登记 5.2 ②「它答不了的」缺口①，`m2-rootchoice-repair-r1-forks.md` 第 1 行还差项）：
/// `subset_of_pristine` 那些格里，`pristine − 走全` 差集的每一个 (设备, 槽) 按结构归类——先按跑前登记
/// 5.2 表原文点名的五类结构逐项核对（分配记录树自己的节点；树表登记的 `LIVELIST` / `SPARSE_SIDE_TABLE`
/// / `DEADLIST` 根，`transaction.rs` 的 `table_entry` 调用点显示这三类树表条目的根**恒为
/// `NodePointer::empty_root()`**——这批历史、乃至今天全仓任何一条写路径都不产生这三类树的物理节点，
/// 所以这一分支理论上永远查不到命中，仍按登记要求逐项核，核不到就如实报「查无归属」）；查无归属时按
/// `AllocationRecord.span_slots` 分「码 2 结构节点（1 槽，`allocator.rs` 顶注『码 2 节点取最低空槽』）」
/// 与「码 3 数据容器（2 槽，`DATA_UNIT_BYTES`=32768=2×`SLOT_BYTES`）」两档报告——span 是分配器自己写死
/// 的界，不是这一段猜的。
fn classify_diff_pair(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
    allocation_record_tree_locations: Option<[LocationEntry; 2]>,
    records: &[AllocationRecord],
    pair: (u32, u64),
) -> &'static str {
    // 根因排查②的实测结果（session s9）：171 格的差集**逐字节等于** `root.instance_table.locations`
    // （实例表自己的物理节点，`RootRecord` 第 23 行，与 `tree_table` / `mapping_root` 是三条并列的独立
    // 指针，不挂在树表的七条条目里，`allocation_record_tree_reachable_placements_via_all_structural_trees`
    // 从未读过它）——`crates/singlefs-core/src/make_filesystem.rs` 第 44 行
    // `INSTANCE_TABLE_SLOT = SlotNumber(UNIT_AREA_START_SLOT)`，`crates/singlefs-format/src/lib.rs`
    // 第 231 行 `UNIT_AREA_START_SLOT: u64 = 50176`，与实测 `device=0/1 slot=50176` 逐位吻合；
    // `mark_format_time_units` 给它的 `Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }` 与实测
    // `span=2`、`generation=0`（格式时刻分配，不是任何一次真实发布的 txg）吻合。**这不是 LIVELIST /
    // SPARSE_SIDE_TABLE / DEADLIST 三种树的节点**（那三种树表条目的根恒为 `NodePointer::empty_root()`，
    // `transaction.rs` 第 3644–3660 行 `publish_admitted` 每次发布都这样写，这批历史、乃至今天全仓任何
    // 一条写路径都不产生它们的物理节点——判定见交回报告）。
    if root
        .instance_table
        .locations
        .iter()
        .any(|location| location_key(location) == pair)
    {
        return "instance_table_self_node_not_walked_by_the_all_structural_trees_arm";
    }
    if let Some(locations) = allocation_record_tree_locations {
        if locations
            .iter()
            .any(|location| location_key(location) == pair)
        {
            return "allocation_record_tree_self_node";
        }
    }
    if let Some(tree_table_bytes) = read_node_bytes(plain_devices, &root.tree_table.locations) {
        if let Ok(tree_table) = parse_index_node(&tree_table_bytes) {
            for entry_bytes in &tree_table.entries {
                if let Some(entry) = TreeTableEntry::parse(entry_bytes) {
                    if matches!(
                        entry.kind,
                        TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE | TREE_KIND_DEADLIST
                    ) && entry
                        .root
                        .locations
                        .iter()
                        .any(|location| location_key(location) == pair)
                    {
                        return "livelist_sparse_deadlist_root_but_always_empty";
                    }
                }
            }
        }
    }
    match records
        .iter()
        .find(|record| (record.device.0, record.slot.0) == pair)
        .map(|record| record.span_slots)
    {
        Some(2) => "span2_unclassified_not_instance_table_not_content_mapping",
        Some(1) => "span1_structural_node_not_a_livelist_sparse_deadlist_root",
        Some(_) => "span_other_unexpected",
        None => "not_found_in_raw_allocation_records",
    }
}

/// session s9：一个差集落点 (设备, 槽) 的聚合上下文——(最小分配记录代, 最大分配记录代,
/// 最小被抛弃根 txg, 最大被抛弃根 txg, 是否在任何一条可读根自己的中央映射树里也出现过)。
type SubsetDiffPairContext = (u64, u64, u64, u64, bool);

#[derive(Default)]
struct LedgerFaultFamilySummary {
    node_count: u64,
    rejected_rollback_attempts: u64,
    unresolved_units: u64,
    empty_baseline_pairs: u64,
    empty_baseline_trigger_count: u64,
    pairs: u64,
    trigger_count: u64,
    error_members: BTreeMap<String, u64>,
    by_aspect_severity: BTreeMap<(&'static str, &'static str), (u64, u64)>,
    allocation_record_tree_recompute_outcomes: BTreeMap<&'static str, u64>,
    allocation_record_tree_released_zero: u64,
    allocation_record_tree_released_nonzero: u64,
    tree_table_has_copy_elsewhere_count: u64,
    tree_table_no_copy_elsewhere_count: u64,
    /// 候选 (b) 树表指称「crates 有路」子分支：键 = (verify_named_units, 结局标签)。
    tree_table_crates_has_a_path_outcomes: BTreeMap<(bool, &'static str), u64>,
    /// 候选 (b) 分配记录树指称「从内容树反推占用集合」（中央映射树）：outcome 标签 → 计数。
    allocation_record_tree_from_content_trees_outcomes: BTreeMap<&'static str, u64>,
    /// 同上，session s8「走全」版本（中央映射树 + 树表 / INODE / EXTENT / ACCOUNTING 四类结构节点）。
    allocation_record_tree_from_all_structural_trees_outcomes: BTreeMap<&'static str, u64>,
    /// session s9：`subset_of_pristine` 那些格里 `pristine − 走全` 差集的每个 (设备, 槽) 按结构归类，
    /// 键 = (设备, 槽, 归类标签)，值 = 这个三元组在整个 H1 族里出现的次数（同一物理落点可能在多个
    /// (历史节点, 被抛弃根) 组合上重复出现，去重看键、频次看值）。
    subset_diff_pairs: BTreeMap<(u32, u64, &'static str), u64>,
    /// session s9（根因排查②）：同一个 (设备, 槽) 差集对，跨全部触发它的 (历史节点, 被抛弃根) 聚合出的
    /// 上下文——(最小分配记录代, 最大分配记录代, 最小被抛弃根 txg, 最大被抛弃根 txg,
    /// 是否在任何一条可读根自己的中央映射树里也出现过)。最后一项为真 ⇒ 这个落点曾经是某条根的当前映射，
    /// 佐证「超期未释放的历史内容单元」这个机制,而不是「结构树种类漏走」。
    subset_diff_pair_context: BTreeMap<(u32, u64), SubsetDiffPairContext>,
    persistence_cost_values: BTreeSet<u64>,
    rejected_zero_device_bytes: u64,
    rejected_nonzero_device_bytes: u64,
    faulted_operation_kinds_tried: BTreeSet<String>,
    /// session s10（岔路单第 1 行 op1 第三种变体，跑前登记 Q1-1c 原文「抬 F 那一处单独一行」）：
    /// 不并进 `pairs`/`trigger_count`/`by_aspect_severity`/`error_members`——候选 (a) 的 A1 副本
    /// 按 5.2 原文不改 `raise_rollback_floor`，混进同一组计数会让 Q1-1a/b 的既有读数（session
    /// s3–s9 已经坐实）掺进一个定义上不同的入口点。
    raise_floor_pairs: u64,
    raise_floor_trigger_count: u64,
    raise_floor_error_members: BTreeMap<String, u64>,
    raise_floor_by_aspect_severity: BTreeMap<(&'static str, &'static str), (u64, u64)>,
    /// 这个历史节点在这一步没有 [F+1, 上限] 的余量可抬（F 已到上限），或探测步骤本身失败——不是
    /// 「该测到却漏了」，是这条历史在这一格没有 op1 第三种变体可跑（`raise_floor_targets_for` 的
    /// 文档注）。
    raise_floor_history_nodes_without_room: u64,
}

/// 跑前登记 5.1 Φ1 + 六、报哪些量：H1 全族 × 每条被抛弃可读根 × 2 个单元 × 3 个严重度 × op1 三种
/// （`mount_writable`、`mount_rollback`、session s10 新增的「先不注入地 `mount_writable` 再
/// `raise_rollback_floor`」，只做「瞬时」；「持续」只在 `mount_writable_trajectory`（session s9）
/// 里单独测过 PC1-a 选中的那一个落点，这个主循环本身不做持续故障）。`is_a1_arm` 只影响 Q1-4/Q1-1c 的
/// 报告措辞（候选 (a) 的错误成员字符串只在编译进 A1 副本时才会真的出现，装置源码两边共用一份，
/// 靠字符串匹配、不靠枚举名，跨臂都能编译）。
fn run_ledger_fault_family(geometry: &Geometry) -> LedgerFaultFamilySummary {
    let parameters = parameters_for(geometry);
    let fixed_geometry = fixed_geometry_for(geometry);
    let family = ledger_fault_history_family(geometry);
    let mut summary = LedgerFaultFamilySummary {
        node_count: u64::try_from(family.nodes.len()).expect("节点数装得进 u64"),
        rejected_rollback_attempts: family.rejected_rollback_attempts,
        ..Default::default()
    };

    for history_node in &family.nodes {
        let node = &history_node.node;
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let Some(event) = node.rollback_events.first() else {
            continue;
        };
        let abandoned_readable: Vec<TimelineRoot> = event
            .abandoned
            .iter()
            .copied()
            .filter(|root| readable.contains(root))
            .collect();
        let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
        // session s10（岔路单第 1 行 op1 第三种变体）：这个历史节点上「先不注入地 mount_writable 再
        // raise_rollback_floor」能测的每一个目标 floor 值，算一次即可，全部 (被抛弃根, 指称, 严重度)
        // 组合共用（探测步骤本身不看被抛弃根、不看故障目标，与它们互不相关）。
        let raise_floor_targets = raise_floor_targets_for(node, &parameters);
        if raise_floor_targets.is_empty() {
            summary.raise_floor_history_nodes_without_room += 1;
        }

        // Φ1「外加空集」：每个节点一次，不注入任何故障时 op1 = mount_writable 是不是自然触发
        // （物理绕环等非注入原因）。故障数固定为 `FaultSeverity::Empty.fault_count()` = 0。
        debug_assert_eq!(FaultSeverity::Empty.fault_count(), 0);
        let empty_attempt = attempt_faulted_operation(
            node,
            &parameters,
            fixed_geometry,
            &FaultedOperationKind::MountWritable,
            &[],
        );
        summary.empty_baseline_pairs += 1;
        if let Some(count) = empty_attempt.abandoned_roots_unreadable {
            if count > 0 {
                summary.empty_baseline_trigger_count += 1;
            }
        }

        for &abandoned_root in &abandoned_readable {
            let Some(record) = root_record_of(&node.pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let tree_table_locations = record.tree_table.locations;
            let allocation_locations = locate_allocation_record_tree(&plain_devices, &record);

            for aspect in [LedgerAspect::TreeTable, LedgerAspect::AllocationRecordTree] {
                let locations = match aspect {
                    LedgerAspect::TreeTable => Some(tree_table_locations),
                    LedgerAspect::AllocationRecordTree => {
                        allocation_locations.as_ref().ok().copied()
                    }
                };
                let Some(locations) = locations else {
                    summary.unresolved_units += 1;
                    continue;
                };
                summary.persistence_cost_values.insert(16384u64 * 2);

                for severity in [
                    FaultSeverity::Both,
                    FaultSeverity::Disk0,
                    FaultSeverity::Disk1,
                ] {
                    let fault_targets = fault_targets_for(&locations, severity);
                    let mut faulted_operation_kinds = vec![FaultedOperationKind::MountWritable];
                    for &target in &readable {
                        faulted_operation_kinds.push(FaultedOperationKind::MountRollbackTo(target));
                    }
                    for kind in &faulted_operation_kinds {
                        summary.faulted_operation_kinds_tried.insert(kind.label());
                        let attempt = attempt_faulted_operation(
                            node,
                            &parameters,
                            fixed_geometry,
                            kind,
                            &fault_targets,
                        );
                        summary.pairs += 1;
                        let key = (aspect.label(), severity.label());
                        let entry = summary.by_aspect_severity.entry(key).or_insert((0, 0));
                        entry.1 += 1;
                        if let Some(count) = attempt.abandoned_roots_unreadable {
                            if count > 0 {
                                summary.trigger_count += 1;
                                entry.0 += 1;
                            }
                        }
                        if let Some(member) = &attempt.error_member {
                            *summary.error_members.entry(member.clone()).or_insert(0) += 1;
                            if member == "AbandonedRootLedgerUnreadable" {
                                let total_bytes: u64 = attempt.device_write_bytes.values().sum();
                                if total_bytes == 0 {
                                    summary.rejected_zero_device_bytes += 1;
                                } else {
                                    summary.rejected_nonzero_device_bytes += 1;
                                }
                            }
                        }

                        // Q1-2：只在分配记录树指称、op1 = mount_writable 时做一次独立走树，避免同一个
                        // (单元, 严重度) 因为多个 mount_rollback 目标被反复走很多遍。
                        if aspect == LedgerAspect::AllocationRecordTree
                            && matches!(kind, FaultedOperationKind::MountWritable)
                        {
                            let pristine = allocation_records_under_root(&plain_devices, &record)
                                .map(|records| {
                                    let released = records
                                        .iter()
                                        .filter(|allocation_record| allocation_record.is_released)
                                        .count();
                                    let unreleased: BTreeSet<(u32, u64)> = records
                                        .iter()
                                        .filter(|allocation_record| !allocation_record.is_released)
                                        .map(|allocation_record| {
                                            (allocation_record.device.0, allocation_record.slot.0)
                                        })
                                        .collect();
                                    (unreleased, released)
                                });
                            match (
                                pristine,
                                walk_allocation_records_with_fault(
                                    node,
                                    fixed_geometry,
                                    &fault_targets,
                                    &record,
                                ),
                            ) {
                                (Ok((pristine_set, released)), Ok(faulted_records)) => {
                                    let faulted_set: BTreeSet<(u32, u64)> = faulted_records
                                        .iter()
                                        .filter(|(_, _, is_released)| !is_released)
                                        .map(|(device, slot, _)| (*device, *slot))
                                        .collect();
                                    let outcome = if faulted_set == pristine_set {
                                        "can_recompute"
                                    } else {
                                        "unequal"
                                    };
                                    *summary
                                        .allocation_record_tree_recompute_outcomes
                                        .entry(outcome)
                                        .or_insert(0) += 1;
                                    if released == 0 {
                                        summary.allocation_record_tree_released_zero += 1;
                                    } else {
                                        summary.allocation_record_tree_released_nonzero += 1;
                                    }
                                }
                                (Ok(_), Err(_)) => {
                                    *summary
                                        .allocation_record_tree_recompute_outcomes
                                        .entry("cannot_recompute")
                                        .or_insert(0) += 1;
                                }
                                (Err(_), _) => {
                                    // 孪生镜像自己都读不出（不该发生在未注入的镜像上），单独计一类。
                                    *summary
                                        .allocation_record_tree_recompute_outcomes
                                        .entry("pristine_unreadable")
                                        .or_insert(0) += 1;
                                }
                            }

                            // 候选 (b) 分配记录树指称「从内容树反推占用集合」（session s6 修订项 5 提的操作化）：
                            // 不重读这一格被注入的分配记录树节点本身，改走中央映射树（不受这一格故障影响，
                            // Φ1 只挡树表或分配记录树那一个单元）。孪生镜像上各自独立算出两个集合再比对，
                            // fault_targets 完全不参与这一支（这正是它不重读目标节点的证据）。
                            let pristine_unreleased = allocation_records_under_root(
                                &plain_devices,
                                &record,
                            )
                            .map(|records| {
                                records
                                    .iter()
                                    .filter(|allocation_record| !allocation_record.is_released)
                                    .map(|allocation_record| {
                                        (allocation_record.device.0, allocation_record.slot.0)
                                    })
                                    .collect::<BTreeSet<(u32, u64)>>()
                            });
                            let from_content_trees =
                                allocation_record_tree_reachable_placements_via_central_mapping(
                                    &plain_devices,
                                    &record,
                                );
                            let outcome = match (pristine_unreleased, from_content_trees) {
                                (Ok(pristine_set), Ok(mapped_set)) => {
                                    if mapped_set == pristine_set {
                                        "matches_pristine"
                                    } else {
                                        "unequal"
                                    }
                                }
                                (Ok(_), Err(_)) => "central_mapping_unreadable",
                                (Err(_), _) => "pristine_unreadable",
                            };
                            *summary
                                .allocation_record_tree_from_content_trees_outcomes
                                .entry(outcome)
                                .or_insert(0) += 1;

                            // 同上，session s8「走全」版本：与上面完全并行的第二次独立比对，同一个
                            // (pristine_unreleased 再算一次, all_structural_trees) 对，gating 条件
                            // 逐字相同，好让两个 outcome 表的分母可以直接对齐比较。
                            let pristine_records_again =
                                allocation_records_under_root(&plain_devices, &record);
                            let pristine_unreleased_again =
                                pristine_records_again.as_ref().map(|records| {
                                    records
                                        .iter()
                                        .filter(|allocation_record| !allocation_record.is_released)
                                        .map(|allocation_record| {
                                            (allocation_record.device.0, allocation_record.slot.0)
                                        })
                                        .collect::<BTreeSet<(u32, u64)>>()
                                });
                            let from_all_structural_trees =
                                allocation_record_tree_reachable_placements_via_all_structural_trees(
                                    &plain_devices,
                                    &record,
                                );
                            let all_structural_outcome =
                                match (&pristine_unreleased_again, &from_all_structural_trees) {
                                    (Ok(pristine_set), Ok(mapped_set)) => {
                                        if mapped_set == pristine_set {
                                            "matches_pristine"
                                        } else if mapped_set.is_superset(pristine_set) {
                                            "superset_of_pristine"
                                        } else if mapped_set.is_subset(pristine_set) {
                                            "subset_of_pristine"
                                        } else {
                                            "unequal_neither_subset_nor_superset"
                                        }
                                    }
                                    (Ok(_), Err(_)) => "structural_trees_unreadable",
                                    (Err(_), _) => "pristine_unreadable",
                                };
                            *summary
                                .allocation_record_tree_from_all_structural_trees_outcomes
                                .entry(all_structural_outcome)
                                .or_insert(0) += 1;

                            // session s9：171 格里的差集逐项归类（见 `classify_diff_pair` 文档注）。
                            if all_structural_outcome == "subset_of_pristine" {
                                if let (Ok(pristine_set), Ok(mapped_set), Ok(raw_records)) = (
                                    &pristine_unreleased_again,
                                    &from_all_structural_trees,
                                    &pristine_records_again,
                                ) {
                                    for pair in pristine_set.difference(mapped_set) {
                                        let label = classify_diff_pair(
                                            &plain_devices,
                                            &record,
                                            allocation_locations.as_ref().ok().copied(),
                                            raw_records,
                                            *pair,
                                        );
                                        *summary
                                            .subset_diff_pairs
                                            .entry((pair.0, pair.1, label))
                                            .or_insert(0) += 1;

                                        // 根因排查②：这个落点的分配记录代、这个被抛弃根自己的 txg，
                                        // 以及它是否在任何一条可读根**自己的**中央映射树里也出现过
                                        // （出现过 ⇒ 它曾是某条根的当前内容，现在只是被更晚的版本
                                        // 换下、还没被释放，不是遗漏了某种结构树）。
                                        let generation = raw_records
                                            .iter()
                                            .find(|candidate| {
                                                (candidate.device.0, candidate.slot.0) == *pair
                                            })
                                            .map(|candidate| candidate.generation.0)
                                            .unwrap_or(u64::MAX);
                                        let referenced_elsewhere = readable.iter().any(
                                            |&candidate_root| {
                                                root_record_of(
                                                    &node.pool,
                                                    &pool_geometry,
                                                    candidate_root,
                                                )
                                                .and_then(|candidate_record| {
                                                    allocation_record_tree_reachable_placements_via_central_mapping(
                                                        &plain_devices,
                                                        &candidate_record,
                                                    )
                                                    .ok()
                                                })
                                                .is_some_and(|mapped| mapped.contains(pair))
                                            },
                                        );
                                        let context = summary
                                            .subset_diff_pair_context
                                            .entry((pair.0, pair.1))
                                            .or_insert((
                                                generation,
                                                generation,
                                                record.checkpoint_txg.0,
                                                record.checkpoint_txg.0,
                                                referenced_elsewhere,
                                            ));
                                        context.0 = context.0.min(generation);
                                        context.1 = context.1.max(generation);
                                        context.2 = context.2.min(record.checkpoint_txg.0);
                                        context.3 = context.3.max(record.checkpoint_txg.0);
                                        context.4 |= referenced_elsewhere;
                                    }
                                }
                            }
                        }

                        // 候选 (b) 树表指称「crates 有路」子分支（跑前登记 5.2 ②）：只在树表指称、
                        // op1 = mount_writable 时做一次，两种 verify_named_units 都跑。
                        if aspect == LedgerAspect::TreeTable
                            && matches!(kind, FaultedOperationKind::MountWritable)
                        {
                            for verify_named_units in [true, false] {
                                let outcome = tree_table_crates_has_a_path(
                                    node,
                                    fixed_geometry,
                                    &fault_targets,
                                    &pool_geometry,
                                    abandoned_root,
                                    &record,
                                    verify_named_units,
                                );
                                *summary
                                    .tree_table_crates_has_a_path_outcomes
                                    .entry((verify_named_units, outcome.label()))
                                    .or_insert(0) += 1;
                            }
                        }
                    }

                    // session s10（岔路单第 1 行 op1 第三种变体，「抬 F 那一处单独一行」——跑前登记
                    // Q1-1c 原文）：这一支不并进上面的 `faulted_operation_kinds`/`summary.pairs`——
                    // 候选 (a) 的 A1 副本按 5.2 原文不改 `raise_rollback_floor`，混进同一组计数会让
                    // Q1-1a/b 的既有读数（session s3–s9 已经坐实）掺进一个定义上不同的入口点。
                    for &target_floor in &raise_floor_targets {
                        let kind =
                            FaultedOperationKind::MountWritableThenRaiseFloorTo(target_floor);
                        summary.faulted_operation_kinds_tried.insert(kind.label());
                        let attempt = attempt_faulted_operation(
                            node,
                            &parameters,
                            fixed_geometry,
                            &kind,
                            &fault_targets,
                        );
                        summary.raise_floor_pairs += 1;
                        let key = (aspect.label(), severity.label());
                        let entry = summary
                            .raise_floor_by_aspect_severity
                            .entry(key)
                            .or_insert((0, 0));
                        entry.1 += 1;
                        if let Some(count) = attempt.abandoned_roots_unreadable {
                            if count > 0 {
                                summary.raise_floor_trigger_count += 1;
                                entry.0 += 1;
                            }
                        }
                        if let Some(member) = &attempt.error_member {
                            *summary
                                .raise_floor_error_members
                                .entry(member.clone())
                                .or_insert(0) += 1;
                        }
                    }
                }
            }

            // Q1-2 树表指称候选 (b)「有副本」：只需要判一次（与严重度、op1 无关，只看孪生镜像本身）。
            if let Some(tree_table_bytes) = read_node_bytes(&plain_devices, &tree_table_locations) {
                if tree_table_has_copy_elsewhere(
                    &plain_devices,
                    &node.pool,
                    &pool_geometry,
                    abandoned_root,
                    &tree_table_bytes,
                ) {
                    summary.tree_table_has_copy_elsewhere_count += 1;
                } else {
                    summary.tree_table_no_copy_elsewhere_count += 1;
                }
            }
        }
    }
    summary
}

fn emit_ledger_fault_family_summary(geometry: &Geometry, summary: &LedgerFaultFamilySummary) {
    emit_result(&format!(
        "name=q1_h1_family_size geometry={} nodes={} rejected_rollback_attempts={} unresolved_units={}",
        geometry.label, summary.node_count, summary.rejected_rollback_attempts, summary.unresolved_units
    ));
    emit_result(&format!(
        "name=q1_1a_empty_baseline geometry={} pairs={} trigger_count={} fault_count={}",
        geometry.label,
        summary.empty_baseline_pairs,
        summary.empty_baseline_trigger_count,
        FaultSeverity::Empty.fault_count()
    ));
    emit_result(&format!(
        "name=q1_1a_trigger_summary geometry={} pairs={} trigger_count={} op1_kinds_tried={}",
        geometry.label,
        summary.pairs,
        summary.trigger_count,
        summary.faulted_operation_kinds_tried.len()
    ));
    for ((aspect, severity), (trigger, pairs)) in &summary.by_aspect_severity {
        emit_result(&format!(
            "name=q1_1a_by_aspect_severity geometry={} aspect={aspect} severity={severity} pairs={pairs} trigger={trigger}"
        , geometry.label));
    }
    for (member, count) in &summary.error_members {
        emit_result(&format!(
            "name=q1_1c_error_member geometry={} member={member} count={count}",
            geometry.label
        ));
    }
    // session s10（岔路单第 1 行 op1 第三种变体，跑前登记 Q1-1c 原文「抬 F 那一处单独一行」）：
    // 单独一组名字（`raise_floor`），不与上面 mount_writable/mount_rollback 那两种 op1 的
    // q1_1a_trigger_summary/q1_1a_by_aspect_severity/q1_1c_error_member 合并。
    emit_result(&format!(
        "name=q1_1a_raise_floor_trigger_summary geometry={} pairs={} trigger_count={} history_nodes_without_room={}",
        geometry.label,
        summary.raise_floor_pairs,
        summary.raise_floor_trigger_count,
        summary.raise_floor_history_nodes_without_room
    ));
    for ((aspect, severity), (trigger, pairs)) in &summary.raise_floor_by_aspect_severity {
        emit_result(&format!(
            "name=q1_1a_raise_floor_by_aspect_severity geometry={} aspect={aspect} severity={severity} pairs={pairs} trigger={trigger}"
        , geometry.label));
    }
    for (member, count) in &summary.raise_floor_error_members {
        emit_result(&format!(
            "name=q1_1c_raise_floor_error_member geometry={} member={member} count={count}",
            geometry.label
        ));
    }
    for (outcome, count) in &summary.allocation_record_tree_recompute_outcomes {
        emit_result(&format!(
            "name=q1_2_allocation_record_tree geometry={} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=q1_2_allocation_released_records geometry={} zero={} nonzero={}",
        geometry.label,
        summary.allocation_record_tree_released_zero,
        summary.allocation_record_tree_released_nonzero
    ));
    emit_result(&format!(
        "name=q1_2_tree_table_copy_elsewhere geometry={} has_copy={} no_copy={}",
        geometry.label,
        summary.tree_table_has_copy_elsewhere_count,
        summary.tree_table_no_copy_elsewhere_count
    ));
    for ((verify_named_units, outcome), count) in &summary.tree_table_crates_has_a_path_outcomes {
        emit_result(&format!(
            "name=q1_2_tree_table_crates_has_a_path geometry={} verify_named_units={verify_named_units} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    for (outcome, count) in &summary.allocation_record_tree_from_content_trees_outcomes {
        emit_result(&format!(
            "name=q1_2_allocation_record_tree_from_content_trees geometry={} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    for (outcome, count) in &summary.allocation_record_tree_from_all_structural_trees_outcomes {
        emit_result(&format!(
            "name=q1_2_allocation_record_tree_from_all_structural_trees geometry={} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=q1_3_persistence_cost_values geometry={} values={:?}",
        geometry.label, summary.persistence_cost_values
    ));
    emit_result(&format!(
        "name=q1_4_a_rejected_device_bytes geometry={} zero_byte_rejections={} nonzero_byte_rejections={}",
        geometry.label,
        summary.rejected_zero_device_bytes,
        summary.rejected_nonzero_device_bytes
    ));
    // session s9：171 格 subset_of_pristine 的差集逐项归类——先按 (设备, 槽, 标签) 去重打一行，
    // 再按标签汇总出现次数，两张表合起来就是「逐个打出来 + 各属于哪种结构」。
    let mut label_totals: BTreeMap<&'static str, u64> = BTreeMap::new();
    for ((device, slot, label), occurrences) in &summary.subset_diff_pairs {
        emit_result(&format!(
            "name=q1_2_subset_diff_pair geometry={} device={device} slot={slot} label={label} occurrences={occurrences}",
            geometry.label
        ));
        *label_totals.entry(label).or_insert(0) += occurrences;
    }
    for (label, total) in &label_totals {
        emit_result(&format!(
            "name=q1_2_subset_diff_label_total geometry={} label={label} occurrences={total}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=q1_2_subset_diff_distinct_pairs geometry={} distinct_pairs={}",
        geometry.label,
        summary.subset_diff_pairs.len()
    ));
    for (
        (device, slot),
        (
            minimum_generation,
            maximum_generation,
            minimum_abandoned_root_txg,
            maximum_abandoned_root_txg,
            referenced_elsewhere,
        ),
    ) in &summary.subset_diff_pair_context
    {
        emit_result(&format!(
            "name=q1_2_subset_diff_pair_context geometry={} device={device} slot={slot} minimum_generation={minimum_generation} maximum_generation={maximum_generation} minimum_abandoned_root_txg={minimum_abandoned_root_txg} maximum_abandoned_root_txg={maximum_abandoned_root_txg} referenced_by_some_readable_roots_own_mapping={referenced_elsewhere}",
            geometry.label
        ));
    }
}

/// PC1-a / PC1-b（跑前登记 5.2 阳性对照）：H1 族按固定次序（`overwrites_before_rollback` 升序、
/// `rollback_target` 按 `BTreeSet` 升序、`overwrites_after_rollback` 升序，
/// 再按每个节点的被抛弃可读根升序）第一个满足「分配记录树节点 u 不共享」的历史。两条共用同一个
/// (节点, 被抛弃根)；PC1-a 打分配记录树指称，PC1-b 打树表指称，都是「两份都读失败、瞬时、
/// op1 = mount_writable」。
fn run_ledger_fault_positive_controls(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let fixed_geometry = fixed_geometry_for(geometry);
    let family = ledger_fault_history_family(geometry);

    for history_node in &family.nodes {
        let node = &history_node.node;
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let Some(event) = node.rollback_events.first() else {
            continue;
        };
        let abandoned_readable: Vec<TimelineRoot> = event
            .abandoned
            .iter()
            .copied()
            .filter(|root| readable.contains(root))
            .collect();
        let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);

        for &abandoned_root in &abandoned_readable {
            let Some(record) = root_record_of(&node.pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let Ok(allocation_locations) = locate_allocation_record_tree(&plain_devices, &record)
            else {
                continue;
            };
            let allocation_keys: BTreeSet<(u32, u64)> =
                allocation_locations.iter().map(location_key).collect();
            if is_unit_shared(
                &plain_devices,
                &node.pool,
                &pool_geometry,
                abandoned_root,
                &allocation_keys,
            ) {
                continue;
            }

            // 找到了：装置自己数「账要经过这个单元的被抛弃根条数」——有几条被抛弃可读根的分配记录树
            // 指针落在同一个 (设备, 槽) 上。
            let mut roots_through_unit = 0u64;
            for &candidate in &abandoned_readable {
                if let Some(candidate_record) =
                    root_record_of(&node.pool, &pool_geometry, candidate)
                {
                    if let Ok(candidate_locations) =
                        locate_allocation_record_tree(&plain_devices, &candidate_record)
                    {
                        let candidate_keys: BTreeSet<(u32, u64)> =
                            candidate_locations.iter().map(location_key).collect();
                        if !candidate_keys.is_disjoint(&allocation_keys) {
                            roots_through_unit += 1;
                        }
                    }
                }
            }

            let allocation_fault_targets =
                fault_targets_for(&allocation_locations, FaultSeverity::Both);
            let allocation_attempt = attempt_faulted_operation(
                node,
                &parameters,
                fixed_geometry,
                &FaultedOperationKind::MountWritable,
                &allocation_fault_targets,
            );
            let allocation_total_bytes: u64 = allocation_attempt.device_write_bytes.values().sum();
            emit_result(&format!(
                "name=pc1_a geometry={} op1={} overwrites_before_rollback={} rollback_target={:?} overwrites_after_rollback={} abandoned_root={:?} roots_through_unit={} outcome_abandoned_roots_unreadable={:?} error_member={:?} error_debug={:?} device_write_bytes={}",
                geometry.label,
                FaultedOperationKind::MountWritable.label(),
                history_node.overwrites_before_rollback,
                history_node.rollback_target,
                history_node.overwrites_after_rollback,
                abandoned_root,
                roots_through_unit,
                allocation_attempt.abandoned_roots_unreadable,
                allocation_attempt.error_member,
                allocation_attempt.error_debug,
                allocation_total_bytes
            ));

            // session s9（岔路单第 1 行还差项④，8.1「持续故障下 op1 及其后两次挂载各一个值；瞬时
            // 故障下 op1 一个值、后两次（不注入）各一个值」）：用 PC1-a 已经选中的这同一个
            // (历史, 被抛弃根, 分配记录树单元, 两份都读失败) 当轨迹的起点——`allocation_attempt`
            // 已经证明这一格会触发，是这条轨迹天然、可复现的落点，不另挑一个未经验证的构造。
            let persistent_trajectory = mount_writable_trajectory(
                &node.pool,
                &parameters,
                fixed_geometry,
                &allocation_fault_targets,
                true,
                3,
            );
            emit_result(&format!(
                "name=q1_1a_op1_trajectory geometry={} mode=persistent trajectory={:?}",
                geometry.label, persistent_trajectory
            ));
            let transient_trajectory = mount_writable_trajectory(
                &node.pool,
                &parameters,
                fixed_geometry,
                &allocation_fault_targets,
                false,
                3,
            );
            emit_result(&format!(
                "name=q1_1a_op1_trajectory geometry={} mode=transient trajectory={:?}",
                geometry.label, transient_trajectory
            ));

            let pristine_allocation =
                allocation_records_under_root(&plain_devices, &record).map(|records| {
                    records
                        .into_iter()
                        .filter(|allocation_record| !allocation_record.is_released)
                        .map(|allocation_record| {
                            (allocation_record.device.0, allocation_record.slot.0)
                        })
                        .collect::<BTreeSet<_>>()
                });
            let faulted_allocation_walk = walk_allocation_records_with_fault(
                node,
                fixed_geometry,
                &allocation_fault_targets,
                &record,
            );
            let allocation_walk_outcome = match (&pristine_allocation, &faulted_allocation_walk) {
                (Ok(pristine_set), Ok(faulted_records)) => {
                    let faulted_set: BTreeSet<(u32, u64)> = faulted_records
                        .iter()
                        .filter(|(_, _, is_released)| !is_released)
                        .map(|(device, slot, _)| (*device, *slot))
                        .collect();
                    if faulted_set == *pristine_set {
                        "can_recompute"
                    } else {
                        "unequal"
                    }
                }
                (Ok(_), Err(_)) => "cannot_recompute",
                (Err(_), _) => "pristine_unreadable",
            };
            emit_result(&format!(
                "name=pc1_a_candidate_b_allocation_record_tree geometry={} outcome={allocation_walk_outcome}",
                geometry.label
            ));

            let tree_table_locations = record.tree_table.locations;
            let tree_table_fault_targets =
                fault_targets_for(&tree_table_locations, FaultSeverity::Both);
            let tree_table_attempt = attempt_faulted_operation(
                node,
                &parameters,
                fixed_geometry,
                &FaultedOperationKind::MountWritable,
                &tree_table_fault_targets,
            );
            let mut roots_through_tree_table = 0u64;
            for &candidate in &abandoned_readable {
                if let Some(candidate_record) =
                    root_record_of(&node.pool, &pool_geometry, candidate)
                {
                    let candidate_keys: BTreeSet<(u32, u64)> = candidate_record
                        .tree_table
                        .locations
                        .iter()
                        .map(location_key)
                        .collect();
                    let target_keys: BTreeSet<(u32, u64)> =
                        tree_table_locations.iter().map(location_key).collect();
                    if !candidate_keys.is_disjoint(&target_keys) {
                        roots_through_tree_table += 1;
                    }
                }
            }
            emit_result(&format!(
                "name=pc1_b geometry={} overwrites_before_rollback={} rollback_target={:?} overwrites_after_rollback={} abandoned_root={:?} roots_through_unit={} outcome_abandoned_roots_unreadable={:?} error_member={:?}",
                geometry.label,
                history_node.overwrites_before_rollback,
                history_node.rollback_target,
                history_node.overwrites_after_rollback,
                abandoned_root,
                roots_through_tree_table,
                tree_table_attempt.abandoned_roots_unreadable,
                tree_table_attempt.error_member
            ));
            let allocation_walk_still_fails_when_tree_table_faulted =
                walk_allocation_records_with_fault(
                    node,
                    fixed_geometry,
                    &tree_table_fault_targets,
                    &record,
                )
                .is_err();
            emit_result(&format!(
                "name=pc1_b_candidate_b_allocation_record_tree geometry={} walk_fails={}",
                geometry.label, allocation_walk_still_fails_when_tree_table_faulted
            ));
            if let Some(tree_table_bytes) = read_node_bytes(&plain_devices, &tree_table_locations) {
                let has_copy = tree_table_has_copy_elsewhere(
                    &plain_devices,
                    &node.pool,
                    &pool_geometry,
                    abandoned_root,
                    &tree_table_bytes,
                );
                emit_result(&format!(
                    "name=pc1_b_candidate_b_tree_table geometry={} has_copy_elsewhere={has_copy}",
                    geometry.label
                ));
            }
            return;
        }
    }
    emit_result(&format!(
        "name=pc1_a_and_b geometry={} verdict=not_found reason=\"H1 全族没有一条『分配记录树节点不共享』的历史\"",
        geometry.label
    ));
}

// ============================================================================
// 十三、Q2-1（跑前登记岔路单第 2 行 ①，C331 修法）：H2 家族 + Φ2 证据项枚举 + 权重从小到大的穷举下界搜索。
//
//    这一段是 session s4 新写的。跑前登记 5.1/5.3 里没做的部分（交回报告「它答不了的」逐条点名）：
//    「构造上界」那一半、H2-R（op2 = `mount_rollback`）、H2c（崩溃档）、系统配置证据项这一类
//    （丙 = 甲-jsn 与甲-txg 两条臂的 `first_txg_of_new_instance` 都不读系统配置，藏它对这两条臂的
//    择根无影响，跳过不改变这两条臂的搜索结果）、PC2 阳性对照（需要给每条臂的副本加一个只供本实验的
//    环境变量开关，属于另一块工程量）、n2=2、n1>3。这一段只做：H2 主族（op2=`mount_writable`）在
//    n1 ∈ {0,1,2,3}、n2=1 上、Φ2 = {根环槽, journal 记录} 两类证据的穷举下界。
//
//    session s8 补：系统配置证据项这一类补上了（`evidence_items_at` 新增
//    `include_system_configuration_slots` 开关），但仍然**只在开关打开时**才进 Φ2——甲-txg、丙、
//    今天（=甲-jsn）三条臂不读系统配置，开着这个开关也不会改变它们的搜索结果，只会白白扩大穷举
//    空间；`run_rootback_tolerance_family`（既有六份副本用的那个）与 `search_minimum_weight_that_
//    triggers_rootback` 的既有调用点都仍传 `false`，行为逐字节不变。新增
//    `run_rootback_tolerance_family_with_system_configuration_evidence` 只在读系统配置的三条臂
//    （乙-留环、丁-留环、丁-只配置）上跑。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvidenceKind {
    RootRingSlot,
    JournalRecord,
    /// session s8 补的第三类（跑前登记 5.1「每块盘上每个自证的系统配置槽（权重 1）」）：只在
    /// `evidence_items_at` 的 `include_system_configuration_slots` 打开时才会出现。
    SystemConfigurationSlot,
}

/// Φ2 的一个证据项：根环槽权重恒 1（根环每个区域第一版只住一块盘，D22（单元原子性怎么合成） 已定项 16
/// 第 4 条）；journal 记录的权重 = 它当下自证过的份数（`targets.len()`，不注入时通常 2）；系统配置槽
/// 权重恒 1（每个 (设备, 槽) 各自一份，不像根环槽那样跨盘）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct EvidenceItem {
    kind: EvidenceKind,
    label: String,
    targets: Vec<(DeviceIdentity, DeviceOffsetInBytes)>,
}

/// 系统配置槽第 0、1 槽各自在两块盘上的 (设备, 槽号, 偏移)：槽 0 恒在偏移 0，槽 1 在偏移
/// `slot_spacing`（`PoolGeometry` 已经把它解出来了，D2（RAID 条带策略） 已定项 19：
/// `max(4096, mkfs 时探测的 io_min)`，池级一个值，两块盘同一个槽距）；固定两块盘（这个实验全程
/// devs=2），不看几何里的 `regions`。
fn system_configuration_slot_positions(slot_spacing: u64) -> [(u32, u64, u64); 4] {
    [
        (0, 0, 0),
        (0, 1, slot_spacing),
        (1, 0, 0),
        (1, 1, slot_spacing),
    ]
}

/// op2 那一刻不注入的孪生镜像上的证据项：根环里每个存着自证根的槽，加上 journal 环里每一条至少一份
/// 自证的记录；`include_system_configuration_slots` 打开时再加每块盘上每个自证过的系统配置槽
/// （跑前登记 5.1 Φ2 定义的第三类，session s8 补——关着时行为与 s4/s6 逐字节不变，见本节顶部说明）。
fn evidence_items_at(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    include_system_configuration_slots: bool,
) -> Vec<EvidenceItem> {
    let mut items = Vec::new();
    let valid_roots = checker_image::valid_roots(pool, geometry);
    let positions = checker_image::root_slot_positions(geometry);
    for (region, slot, _view) in &valid_roots {
        if let Some((_, _, device_number, offset)) =
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    candidate_region == region && candidate_slot == slot
                })
        {
            items.push(EvidenceItem {
                kind: EvidenceKind::RootRingSlot,
                label: format!("root(region={region},slot={slot})"),
                targets: vec![(DeviceIdentity(*device_number), DeviceOffsetInBytes(*offset))],
            });
        }
    }
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    for slot_index in 0..ring_slots {
        let offset = record_offset(slot_index + 1, geometry.journal_ring_bytes);
        let mut targets = Vec::new();
        let mut sample = None;
        for device_number in [0u32, 1u32] {
            if let Some(bytes) = pool.read(device_number, offset.0, record_bytes) {
                if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {
                    targets.push((DeviceIdentity(device_number), offset));
                    sample.get_or_insert((record.instance.0, record.counter));
                }
            }
        }
        if let Some((instance, counter)) = sample {
            items.push(EvidenceItem {
                kind: EvidenceKind::JournalRecord,
                label: format!("record(instance={instance},counter={counter})"),
                targets,
            });
        }
    }
    if include_system_configuration_slots {
        let slot_bytes =
            usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for (device_number, slot_index, offset) in
            system_configuration_slot_positions(geometry.slot_spacing)
        {
            if let Some(bytes) = pool.read(device_number, offset, slot_bytes) {
                if singlefs_checker::check_system_configuration_slot(&bytes).is_ok() {
                    items.push(EvidenceItem {
                        kind: EvidenceKind::SystemConfigurationSlot,
                        label: format!("configuration(device={device_number},slot={slot_index})"),
                        targets: vec![(DeviceIdentity(device_number), DeviceOffsetInBytes(offset))],
                    });
                }
            }
        }
    }
    items
}

/// 权重恰为 `weight` 的证据项子集：权重恒 1 的那一批（根槽；`include_system_configuration_slots`
/// 打开时系统配置槽也并进这一批，两类都是权重 1，对这个函数的组合算术没有分别）取
/// `weight_one_count` 个、记录（权重 2，见 `evidence_items_at` 里 `targets` 恒两份）取
/// `record_count` 个，`weight_one_count + 2 * record_count = weight`，逐个 `record_count` 值把
/// 两边的组合叉乘。
fn candidate_fault_sets_of_weight(
    weight_one_items: &[EvidenceItem],
    journal_record_items: &[EvidenceItem],
    weight: u64,
) -> Vec<Vec<EvidenceItem>> {
    let mut candidates = Vec::new();
    let mut record_count = 0u64;
    while record_count * 2 <= weight {
        let weight_one_count = weight - record_count * 2;
        if let (Ok(weight_one_count), Ok(record_count_usize)) = (
            usize::try_from(weight_one_count),
            usize::try_from(record_count),
        ) {
            if weight_one_count <= weight_one_items.len()
                && record_count_usize <= journal_record_items.len()
            {
                for weight_one_combination in
                    combinations_of_size(weight_one_items, weight_one_count)
                {
                    for record_combination in
                        combinations_of_size(journal_record_items, record_count_usize)
                    {
                        let mut combination = weight_one_combination.clone();
                        combination.extend(record_combination.iter().cloned());
                        candidates.push(combination);
                    }
                }
            }
        }
        record_count += 1;
    }
    candidates
}

/// PC2 排查出的根因，2026-09-24 session s5：`FaultInjectingBlockDevice` 一次只装一条
/// `SharedFaultPlan`、一条计划只认一个精确偏移；`fault_injection.rs` 自己的读法写死表说得很清楚
/// 「一个 `SharedFaultPlan` 一次只装一条计划，多落点只能这样叠」（一块盘一层）——而
/// `attempt_rootback_probe_and_advance`（下面这个函数）原来对每块盘只用 `.find()` 挑
/// `fault_targets` 里第一个落在这块盘的目标去装那一层，**同一块盘上第二个及以后的目标偏移从未真正
/// 装上故障**。只有 2 块盘时，一个多落点组合里「至少两个落在同一块盘」极常见（按鸽笼原理，权重 ≥ 2
/// 就可能撞上、权重越高越必然撞上），H2 全家族至今用这套机制跑出的「0 命中」都因此打上问号——
/// 这不是「故障不够多」，是**故障根本没有全装上**。修法：不再嵌套 `FaultInjectingBlockDevice`，
/// 换一个能一次记住多个精确偏移、每次读都在这些偏移上报错的本地包装，一块盘一份、一次装完它全部
/// 该装的目标。依据、时点写进跑前登记「十二、修订」2026-09-24（session s5）条目。
struct MultiOffsetReadFailingBlockDevice {
    inner: SparseBlockDevice,
    failing_offsets: BTreeSet<u64>,
}

impl BlockDevice for MultiOffsetReadFailingBlockDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if self.failing_offsets.contains(&offset.0) {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 把 `node.pool` 按 `fault_targets` 包上多落点读故障（一块盘一份 `MultiOffsetReadFailingBlockDevice`，
/// 每一份记它自己那几个精确偏移），跑一次 `mount_writable`，交回挂载结果与撤故障之后的原始设备。
fn attempt_a_faulted_mount_writable(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> Result<
    (
        singlefs_core::mount::Mounted,
        Vec<(DeviceIdentity, SparseBlockDevice)>,
    ),
    String,
> {
    let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut wrapped: Vec<(DeviceIdentity, MultiOffsetReadFailingBlockDevice)> = Vec::new();
    for (identity, device) in plain {
        let failing_offsets: BTreeSet<u64> = fault_targets
            .iter()
            .filter(|(target_device, _)| *target_device == identity)
            .map(|(_, offset)| offset.0)
            .collect();
        wrapped.push((
            identity,
            MultiOffsetReadFailingBlockDevice {
                inner: device,
                failing_offsets,
            },
        ));
    }
    let mounted = mount_writable(parameters, &mut wrapped)
        .map_err(|error| format!("faulted mount_writable: {error:?}"))?;
    let raw_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = wrapped
        .into_iter()
        .map(|(identity, wrapped_device)| (identity, wrapped_device.inner))
        .collect();
    Ok((mounted, raw_devices))
}

/// 一次 op2 尝试：按 `fault_targets` 装故障，只在这次调用期间生效（瞬时），成功就把落盘状态解出来、
/// 按 `MountOutput::effective_root` 在装置自记时间线上的位置截断——这正是 H2 要测的：故障让
/// `effective_root` 从比时间线尾更旧的根接着走。与 `apply_mount_rollback` 同一套「父链」逻辑，
/// 但目标是 `effective_root`，不是调用方给的回退目标，也不记 `RollbackEvent`（这不是一次语义回退，
/// 是故障造成的断支，5.0「末端之后那一截记作断支」）。
fn attempt_rootback_probe_and_advance(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> Result<SimNode, String> {
    let (mounted, raw_devices) = attempt_a_faulted_mount_writable(node, parameters, fault_targets)
        .map_err(|error| format!("op2: {error}"))?;
    let effective = root_pair(&mounted.output.effective_root);
    let Some(position) = node.timeline.iter().position(|root| *root == effective) else {
        return Err(format!(
            "op2 的 effective_root {effective:?} 不在装置自记的时间线 {:?} 上（不应发生）",
            node.timeline
        ));
    };
    let mut timeline = node.timeline[..=position].to_vec();
    let mut content_by_root = node.content_by_root.clone();
    let carried_forward_content = content_by_root.get(&effective).cloned().unwrap_or(None);
    timeline.push(root_pair(mounted.output.row_publish.root()));
    content_by_root.insert(
        root_pair(mounted.output.row_publish.root()),
        carried_forward_content.clone(),
    );
    for warm_up_publish in &mounted.output.warm_up_publishes {
        timeline.push(root_pair(warm_up_publish.root()));
        content_by_root.insert(
            root_pair(warm_up_publish.root()),
            carried_forward_content.clone(),
        );
    }
    let mut path = node.path.clone();
    path.push(HistoryStep::MountWritable);
    Ok(SimNode {
        pool: memory_pool_of(&raw_devices, IMAGE_BYTES),
        session: Some(Session {
            allocator: mounted.allocator,
            current: mounted.current,
            instance: mounted.output.instance,
        }),
        timeline,
        rollback_events: node.rollback_events.clone(),
        path,
        content_by_root,
    })
}

/// P331 的一处判定（5.0）：所选根不在时间线上记「倒挂」；在，而结局不是 `FileRead` 带时间线末端那一版
/// 的内容记「读不回」。
fn judge_recovery_outcome(
    label: &str,
    outcome: &RecoveryOutcome,
    timeline: &[TimelineRoot],
    expected_content: &Option<Vec<u8>>,
) -> (bool, String) {
    let chosen = match outcome {
        RecoveryOutcome::NoFile { root } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::FileRead { root, .. } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::Failed { root, .. } => root.map(|value| (value.0 .0, value.1 .0)),
    };
    let Some(chosen) = chosen else {
        return (
            false,
            format!("{label}: recover 完全失败 outcome={outcome:?}"),
        );
    };
    if !timeline.contains(&chosen) {
        return (false, format!("{label}: 倒挂 chosen_root={chosen:?}"));
    }
    match outcome {
        RecoveryOutcome::FileRead { content, .. } if Some(content.clone()) == *expected_content => {
            (true, "ok".to_string())
        }
        RecoveryOutcome::NoFile { .. } if expected_content.is_none() => (true, "ok".to_string()),
        RecoveryOutcome::NoFile { .. }
        | RecoveryOutcome::FileRead { .. }
        | RecoveryOutcome::Failed { .. } => (
            false,
            format!("{label}: 读不回 outcome={outcome:?} expected={expected_content:?}"),
        ),
    }
}

/// P331 两处判定都跑（5.0）：① 冷启动 recover；② 再做一次不注入的 `mount_writable`，之后再冷启动
/// recover。任一处不成立即「打中」。交回 (打中?, 检查①的细节, 检查②的细节)。
fn evaluate_rootback_predicate(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> (bool, String, String) {
    let tip = *node.timeline.last().expect("timeline 非空");
    let expected_content = node.content_by_root.get(&tip).cloned().unwrap_or(None);

    let devices_for_first_check = devices_from_pool(&node.pool, IMAGE_BYTES);
    let first_report = recover(&devices_for_first_check, JournalPolicy::Consult);
    let (first_check_passed, first_detail) = judge_recovery_outcome(
        "cold_recover",
        &first_report.outcome,
        &node.timeline,
        &expected_content,
    );

    let mut devices_for_second_check = devices_from_pool(&node.pool, IMAGE_BYTES);
    let (second_check_passed, second_detail) = match mount_writable(
        parameters,
        &mut devices_for_second_check,
    ) {
        Err(error) => (false, format!("unfaulted_mount_writable: {error:?}")),
        Ok(mounted) => {
            let chosen = root_pair(&mounted.output.chosen_root);
            if !node.timeline.contains(&chosen) {
                (
                    false,
                    format!("unfaulted_mount_writable: 倒挂 chosen_root={chosen:?}"),
                )
            } else {
                // 5.0 原文这一半只比内容号，不再比「所选根在不在时间线上」：这次探针性的
                // `mount_writable` 本身推进了状态（新实例的写行/暖机），它自己的
                // `chosen_root` 已经在上面判过「倒挂」；读回时的所选根天然是这个新实例的
                // 根，不在 `node.timeline`（那份只记到 op2 探针为止），重新套一次
                // `judge_recovery_outcome` 的「倒挂」半句会把这个正常前进误判成倒挂。
                let pool_after = memory_pool_of(&devices_for_second_check, IMAGE_BYTES);
                let devices_for_readback = devices_from_pool(&pool_after, IMAGE_BYTES);
                let readback_report = recover(&devices_for_readback, JournalPolicy::Consult);
                match &readback_report.outcome {
                        RecoveryOutcome::FileRead { content, .. }
                            if Some(content.clone()) == expected_content =>
                        {
                            (true, "ok".to_string())
                        }
                        RecoveryOutcome::NoFile { .. } if expected_content.is_none() => {
                            (true, "ok".to_string())
                        }
                        other @ (RecoveryOutcome::NoFile { .. }
                        | RecoveryOutcome::FileRead { .. }
                        | RecoveryOutcome::Failed { .. }) => (
                            false,
                            format!(
                                "unfaulted_mount_writable_then_recover: 读不回 outcome={other:?} expected={expected_content:?}"
                            ),
                        ),
                    }
            }
        }
    };
    (
        !(first_check_passed && second_check_passed),
        first_detail,
        second_detail,
    )
}

struct RootbackToleranceSearchOutcome {
    weight_at_first_hit: Option<u64>,
    hit_count_at_that_weight: u64,
    /// 打中这一权重的第一个子集，按各证据项的 `label` 记（跑前登记 5.1「记下这一权重上打中的子集
    /// 个数与第一个子集」）。
    first_hit_combination_labels: Vec<String>,
    /// 这一次搜索实际枚举过的子集总数（各完整权重档累计；从不在某一档中途停，见函数文档）。
    subsets_tried: u64,
    /// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②）：搜到的最高一档权重，
    /// 与 `weight_at_first_hit`/`stopped_by_weight_ceiling` 一起读——三者合起来就是
    /// 「穷举到权重几」的完整交代，不许只报一个「capped」布尔值把这句话吞掉。
    highest_weight_examined: u64,
    /// 因为传了 `weight_ceiling` 而在权重档边界（不是档中途）停下、且这一档还没有打中、且没有到
    /// `maximum_weight`（还有没搜到的档）。真时这一行必须同时报 `full_space_subset_count` 与
    /// `highest_weight_examined`，不许只写「capped」三个字。
    stopped_by_weight_ceiling: bool,
    rootback_probe_failures: u64,
    overwrite_failures: u64,
    /// 这个模板 Φ2（根槽 + journal 记录两类，`include_system_configuration_slots` 打开时再加系统
    /// 配置槽）的证据项权重之和：`weight_at_first_hit` 为 `None` 且 `stopped_by_weight_ceiling` 为假
    /// 时，说明穷举已经走完全部权重都没有打中——这个数就是「这个模板最多能凑出多大的故障集合」，
    /// 交回报告解释「没打中」是不是因为证据本来就不够多。
    maximum_weight: u64,
    /// 完整证据空间的子集总数 = 2^(权重一类证据项数 + 权重二类证据项数)——不管这次搜索实际走到
    /// 哪一档，这个数恒报，回答「完整空间多大」（`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行
    /// 还差项②逐字要求：「先对每个 n1 算出完整证据空间的子集数」）。
    full_space_subset_count: u64,
}

/// Q2-1 的穷举下界：按总权重从小到大枚举 Φ2 子集，**同权重内全枚举、只在权重档的边界上决定停不停**
/// （不在一档中途停——`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②：「这类故障、崩溃点的
/// 枚举不为省时间缩范围……不许靠计数上限静默截断」）。`weight_ceiling` 为 `None` 时穷举到
/// `maximum_weight`（完整证据空间的最高一档，等价于穷举完整个 2^N 子集空间）；为 `Some(w)` 时最多穷举
/// 到权重 `w`（这一档仍然全枚举完才停，不掐在档中途），调用方要在结果行里同时报
/// `full_space_subset_count`（完整空间多大）与 `highest_weight_examined`（穷举到权重几），不许只写
/// 「capped」。`include_system_configuration_slots` 打开时 Φ2 加上跑前登记 5.1 的第三类证据
/// （session s8）；既有调用点都传 `false`，行为不变（见十三节顶部说明）。
fn search_minimum_weight_that_triggers_rootback(
    node_before_rootback_probe: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry_view: &PoolGeometry,
    overwrites_after_the_rootback_probe: u64,
    weight_ceiling: Option<u64>,
    include_system_configuration_slots: bool,
) -> RootbackToleranceSearchOutcome {
    let items = evidence_items_at(
        &node_before_rootback_probe.pool,
        geometry_view,
        include_system_configuration_slots,
    );
    let weight_one_items: Vec<EvidenceItem> = items
        .iter()
        .filter(|item| {
            item.kind == EvidenceKind::RootRingSlot
                || item.kind == EvidenceKind::SystemConfigurationSlot
        })
        .cloned()
        .collect();
    let journal_record_items: Vec<EvidenceItem> = items
        .iter()
        .filter(|item| item.kind == EvidenceKind::JournalRecord)
        .cloned()
        .collect();
    let weight_one_count = u64::try_from(weight_one_items.len()).expect("证据项数装得进 u64");
    let journal_record_count =
        u64::try_from(journal_record_items.len()).expect("证据项数装得进 u64");
    let maximum_weight = weight_one_count + 2 * journal_record_count;
    // 完整证据空间的子集总数 = 2^(权重一类证据项数 + 权重二类证据项数)：每一类证据项各自「选或不选」
    // 独立自由，乘起来就是整棵幂集，与按权重分档遍历是同一个空间的两种数法（分档遍历把它切成
    // maximum_weight+1 个互斥的档，各档之和恰等于这个数——这条恒等式本身也是「没有静默漏掉子集」的
    // 一个自证）。
    let total_items = weight_one_count + journal_record_count;
    let full_space_subset_count = 1u64
        .checked_shl(u32::try_from(total_items).unwrap_or(u32::MAX))
        .unwrap_or(u64::MAX);
    // 调用方显式给了权重上限就用它（截到 `maximum_weight` 之内，超过等于不设上限）；没给时，完整
    // 空间装得进预算就穷举到底，装不进就退到 `DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE`（权重 12）——
    // 这一步判断本身也在结果里报出来（`RootbackToleranceSearchOutcome::full_space_subset_count` 与
    // `highest_weight_examined`），不是又一层静默截断。
    let effective_ceiling = match weight_ceiling {
        Some(explicit) => explicit.min(maximum_weight),
        None if full_space_subset_count <= FEASIBLE_FULL_SEARCH_SUBSET_BUDGET => maximum_weight,
        None => DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE.min(maximum_weight),
    };

    let mut subsets_tried = 0u64;
    let mut rootback_probe_failures = 0u64;
    let mut overwrite_failures = 0u64;
    let mut weight = 0u64;
    loop {
        // 每一档都全枚举完才检查停不停——不在 `for combination in &candidates` 中途插入计数检查，
        // 这正是「不许靠计数上限静默截断」要求的：停只停在档的边界上，且停的理由（到顶/到穷举上限）
        // 与这一档、这一次穷举到的最高权重一起报出去，不许只留一个 `capped` 布尔值。
        let candidates =
            candidate_fault_sets_of_weight(&weight_one_items, &journal_record_items, weight);
        let mut hit_count = 0u64;
        let mut first_hit_combination_labels: Vec<String> = Vec::new();
        for combination in &candidates {
            subsets_tried += 1;
            let fault_targets: Vec<(DeviceIdentity, DeviceOffsetInBytes)> = combination
                .iter()
                .flat_map(|item| item.targets.clone())
                .collect();
            let mut node = match attempt_rootback_probe_and_advance(
                node_before_rootback_probe,
                parameters,
                &fault_targets,
            ) {
                Ok(node) => node,
                Err(_) => {
                    rootback_probe_failures += 1;
                    continue;
                }
            };
            let mut overwrite_ok = true;
            for _ in 0..overwrites_after_the_rootback_probe {
                match apply_overwrite(&node, parameters) {
                    Ok(next) => node = next,
                    Err(_) => {
                        overwrite_ok = false;
                        break;
                    }
                }
            }
            if !overwrite_ok {
                overwrite_failures += 1;
                continue;
            }
            let (hit, _, _) = evaluate_rootback_predicate(&node, parameters);
            if hit {
                if hit_count == 0 {
                    first_hit_combination_labels =
                        combination.iter().map(|item| item.label.clone()).collect();
                }
                hit_count += 1;
            }
        }
        if hit_count > 0 {
            return RootbackToleranceSearchOutcome {
                weight_at_first_hit: Some(weight),
                hit_count_at_that_weight: hit_count,
                first_hit_combination_labels,
                subsets_tried,
                highest_weight_examined: weight,
                stopped_by_weight_ceiling: false,
                rootback_probe_failures,
                overwrite_failures,
                maximum_weight,
                full_space_subset_count,
            };
        }
        if weight >= effective_ceiling {
            return RootbackToleranceSearchOutcome {
                weight_at_first_hit: None,
                hit_count_at_that_weight: 0,
                first_hit_combination_labels: Vec::new(),
                subsets_tried,
                highest_weight_examined: weight,
                stopped_by_weight_ceiling: weight < maximum_weight,
                rootback_probe_failures,
                overwrite_failures,
                maximum_weight,
                full_space_subset_count,
            };
        }
        weight += 1;
    }
}

/// H2 主族（跑前登记 5.1）：mkfs → `mount_writable`（实例 1）→ 首个文件 → n1 次覆盖写 → 关闭 →
/// op2 = `mount_writable`（受注入）→ 成功则 n2 次覆盖写 → 关闭 → 判 P331。n1 ∈ {0,...,6}（登记要求
/// 的全范围，2026-09-24 session s5 从 {0,1,2,3} 补齐——上一段只跑到 3 是挂钟预算，不是构造上限；
/// 另加 H2-R 的 `mount_rollback` 变体与 H2c 崩溃档，这两个仍没做，见交回报告「它答不了的」）。
/// `include_system_configuration_slots`：session s8 补的开关，打开时 Φ2 加系统配置槽这一类证据、
/// 结果行改名 `q2_1_minimum_weight_with_configuration_evidence`（不与既有六份副本已经跑过的
/// `q2_1_minimum_weight` 混在一起）；关着时逐字节复现既有调用点的行为。
/// `overwrites_before_the_rootback_probe_range`：n1 的取值区间（跑前登记要求 {0,...,6} 全范围）；
/// session s8 把它从写死的 `0..=6` 拆成参数，只为了能把一次挂钟预算装不下的全范围拆成几次跑
/// （见 `main` 里 `q2-1-g0-configuration-evidence` 的可选命令行参数）——不改判据、不改任何一个 n1
/// 值本身怎么算，只改跑哪几个 n1。既有调用点仍传 `0..=6`，行为不变。
fn run_rootback_tolerance_family(
    geometry: &Geometry,
    overwrites_after_the_rootback_probe: u64,
    include_system_configuration_slots: bool,
    overwrites_before_the_rootback_probe_range: std::ops::RangeInclusive<u64>,
    weight_ceiling: Option<u64>,
) {
    let parameters = parameters_for(geometry);
    let result_name = if include_system_configuration_slots {
        "q2_1_minimum_weight_with_configuration_evidence"
    } else {
        "q2_1_minimum_weight"
    };
    for overwrites_before_the_rootback_probe in overwrites_before_the_rootback_probe_range {
        let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
            Ok(node) => node,
            Err(error) => {
                emit_result(&format!(
                    "name=q2_1_bootstrap_failed geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?}",
                    geometry.label
                ));
                continue;
            }
        };
        let geometry_view = match independent_geometry(&node.pool) {
            Ok(view) => view,
            Err(error) => {
                emit_result(&format!(
                    "name=q2_1_geometry_failed geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error}",
                    geometry.label
                ));
                continue;
            }
        };
        let outcome = search_minimum_weight_that_triggers_rootback(
            &node,
            &parameters,
            &geometry_view,
            overwrites_after_the_rootback_probe,
            weight_ceiling,
            include_system_configuration_slots,
        );
        match outcome.weight_at_first_hit {
            Some(weight) => emit_result(&format!(
                "name={result_name} geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min={weight} hit_count_at_that_weight={} first_hit_combination={:?} subsets_tried={} full_space_subset_count={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label,
                outcome.hit_count_at_that_weight,
                outcome.first_hit_combination_labels,
                outcome.subsets_tried,
                outcome.full_space_subset_count,
                outcome.rootback_probe_failures,
                outcome.overwrite_failures
            )),
            None if outcome.stopped_by_weight_ceiling => emit_result(&format!(
                "name={result_name} geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min=not_found_up_to_weight_ceiling highest_weight_examined={} maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label, outcome.highest_weight_examined, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
            )),
            None => emit_result(&format!(
                "name={result_name} geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min=none_found_within_full_evidence_space maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
            )),
        }
    }
}

// ============================================================================
// 十三点五、PC2（岔路单第 2 行 ①，阳性对照，2026-09-24 session s5 补）：直接复现判决 K2
//    （`research/prompts/m2-rootchoice-repair-r1-main-verification.md`「### K2」）已经判过的两个
//    具体构造——甲「计数器从全环最大 + 1 起」4 个瞬时根槽读失败打中；乙「系统配置带 txg」0 个注入
//    故障、1 个崩溃点打中——不靠穷举搜索。复现得出，证明装置（故障装配、P331 评判）看得见判决说
//    存在的打中；复现不出，先查装置哪里没对上，不把「穷举没打中」当结论（见交回报告）。
//
//    臂对齐（登记 5.6 项 2、5.3）：K2 表「甲」= 这份登记的「甲-txg」（`first_txg_of_new_instance`
//    只取 `highest_root_txg` + 1，D16（发布语义） 已定项 6 的 checkpoint_txg 读法）。**不是**「甲-jsn」
//    （今天的 `next_counter`）：`mount.rs` 现查过——`next_counter`（第 1422 行 `records.values().
//    map(|record| record.counter).max()...+1`）只决定下一条 journal 记录的 jsn，`first_txg_of_
//    new_instance`（第 322–339 行）从未读它；这两个量在代码里完全不相交，「甲-jsn」在「选根 txg」
//    这一格没有可以打中的机制，K2 给的具体历史（H-A，攻方腿报告二·一节）用的 W 定义
//    （「环里全部自证过且读得出的根的最大 txg」）逐字对应 `highest_root_txg`，即甲-txg。
//
//    K2 表「乙」= 这份登记的「乙-只配置」的一个更窄子集：只需要系统配置带一个 `published_txg`
//    字段、取号写占位 0、轮换写真实值——不需要乙-留环额外的「续传」那一半（H-B1 只用零故障 + 一个
//    崩溃点，不需要读环）。这个函数只在改了该候选的副本上才会打中；跑在没有 `published_txg` 字段的
//    臂（今天、甲-txg）上必须不打中——见函数内注释。
// ============================================================================

/// PC2 甲专用取故障目标：给定一组 (实例, txg)，在孪生（不注入）镜像上用 `checker_image` 的独立解码
/// 把它们各自解到 (设备, 偏移)——与 `evidence_items_at` 同一套坐标系，只是按 txg 精确点名，不按权重
/// 枚举。
fn root_ring_slot_targets_for(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    wanted: &BTreeSet<TimelineRoot>,
) -> Vec<(DeviceIdentity, DeviceOffsetInBytes)> {
    let positions = checker_image::root_slot_positions(geometry);
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .filter(|(_, _, view)| wanted.contains(&(view.instance, view.checkpoint_txg)))
        .filter_map(|(region, slot, _view)| {
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    *candidate_region == region && *candidate_slot == slot
                })
                .map(|(_, _, device_number, offset)| {
                    (DeviceIdentity(*device_number), DeviceOffsetInBytes(*offset))
                })
        })
        .collect()
}

/// PC2 甲专用挂载：一次带故障的 `mount_writable`，**不**按 `MountOutput::effective_root` 截断时间线
/// ——这正是 `attempt_rootback_probe_and_advance` 测不出 H-A 的原因之一（另一个原因是本节顶部记的
/// 多落点装故障的根因，两个原因分开修）：H-A 的机制不是「这次挂载自己的 recover/replay 被故障弄
/// 糊涂」（`effective_root` 靠 journal 记录重建，H-A 只点名根槽、记录仍读得出，`replay_journal`
/// 照样把 `effective_root` 重建回旧时间线的真实末端，`attempt_rootback_probe_and_advance` 因此永远
/// 截不断）——而是「这次挂载自己的 `first_txg_of_new_instance` 被故障压低，新根写进旧根占的物理槽位
/// （按 txg 算槽位，读故障不挡写），旧根本身的字节没被任何写碰到、故障一撤销就又冒出来、在下一次
/// 冷启动的择根里凭更高 txg 赢回去」。这里原样接受 `mount_writable` 给出的 `chosen_root`（不 assert
/// 它等于 tip）、把写行/暖机的根追加在原时间线**末尾**（不做任何截断——旧的高 txg 根条目原样留着）。
fn attempt_a_faulted_mount_writable_and_append(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> Result<SimNode, String> {
    let (mounted, raw_devices) = attempt_a_faulted_mount_writable(node, parameters, fault_targets)?;
    let mut timeline = node.timeline.clone();
    let mut content_by_root = node.content_by_root.clone();
    let carried_forward_content = content_by_root
        .get(&root_pair(&mounted.output.chosen_root))
        .cloned()
        .unwrap_or(None);
    timeline.push(root_pair(mounted.output.row_publish.root()));
    content_by_root.insert(
        root_pair(mounted.output.row_publish.root()),
        carried_forward_content.clone(),
    );
    for warm_up_publish in &mounted.output.warm_up_publishes {
        timeline.push(root_pair(warm_up_publish.root()));
        content_by_root.insert(
            root_pair(warm_up_publish.root()),
            carried_forward_content.clone(),
        );
    }
    let mut path = node.path.clone();
    path.push(HistoryStep::MountWritable);
    Ok(SimNode {
        pool: memory_pool_of(&raw_devices, IMAGE_BYTES),
        session: Some(Session {
            allocator: mounted.allocator,
            current: mounted.current,
            instance: mounted.output.instance,
        }),
        timeline,
        rollback_events: node.rollback_events.clone(),
        path,
        content_by_root,
    })
}

/// PC2 甲：复现攻方腿 H-A（`research/prompts/m2-rootchoice-repair-r1-opus-output.md:73`–`105`）。
/// 第一次尝试重用 `search_minimum_weight_that_triggers_rootback`（穷举下界那一套）在
/// `overwrites_before_the_rootback_probe = 4` 上跑，**20000 个子集全试完、一次都没打中**——查出
/// 原因：那一套靠 `attempt_rootback_probe_and_advance` 按 `effective_root` 截断时间线，H-A 的机制
/// 与「`effective_root` 被故障影响」无关（见 `attempt_a_faulted_mount_writable_and_append` 的
/// 文档），穷举下界这一套测不出这一类打中——不是「4 个故障不够」，是这件工具量错了维度。改用专门的
/// 构造：n1 = 4 次覆盖写把 tip 推到 (1, 7)（mkfs 起 2 次暖机 + 首个文件 txg 3 + 4 次覆盖写）；对
/// tip 最新的 4 条根槽（对应 H-A「顶上连续 4 个」）精确装读故障；op2 = 带故障 `mount_writable`；
/// 成功则 n2 = 1 次覆盖写；撤故障、判 P331。
fn run_positive_control_for_the_root_ring_txg_only_candidate(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let topmost_four: BTreeSet<TimelineRoot> =
        node.timeline.iter().rev().take(4).copied().collect();
    let fault_targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_four);
    let fault_target_count = fault_targets.len();
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_faulted_mount",
                geometry.label
            ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 topmost_four={topmost_four:?} fault_target_count={fault_target_count} expected_by_k2_table_fault_count=4 hit={hit} expected_by_k2_table_hit=true first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// PC2 乙：复现攻方腿 H-B1（`research/prompts/m2-rootchoice-repair-r1-opus-output.md:114`–`148`）。
/// 造一次「崩在取号之后、写行发布最早的字节之前」——`acquire_instance`（`crates/singlefs-core`
/// 公开函数）自己只做「逐盘写系统配置槽 + 一道屏障」两步（`transaction.rs` 的 `write_acquired_
/// instance`），中途没有别的写；单独调它、原样收工，产生的设备字节与那个崩溃点完全相同，不需要层
/// 0 的崩溃点枚举器。之后在这份「崩溃」镜像上正常做一次 `mount_writable`（相当于 H-B1 的挂载 3）
/// 加一次覆盖写（n2 = 1），再判 P331。
fn run_positive_control_for_the_configuration_published_txg_candidate(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_crash = 2u64;
    let node = match bootstrap(geometry, overwrites_before_the_crash) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=bootstrap"
            , geometry.label));
            return;
        }
    };
    let mut devices_for_the_crashed_acquisition = devices_from_pool(&node.pool, IMAGE_BYTES);
    let acquisition_before_the_crash = {
        let mut writer = PoolWriter::new(
            &parameters,
            devices_for_the_crashed_acquisition.as_mut_slice(),
        );
        acquire_instance(&mut writer)
    };
    let acquired_instance_before_the_crash = match acquisition_before_the_crash {
        Ok(instance) => instance,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=acquire_instance_before_the_crash"
            , geometry.label));
            return;
        }
    };
    let node_after_the_crash = SimNode {
        pool: memory_pool_of(&devices_for_the_crashed_acquisition, IMAGE_BYTES),
        ..node.clone()
    };
    let node_after_the_next_mount = match apply_mount_writable(&node_after_the_crash, &parameters) {
        Ok(mounted_node) => mounted_node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=mount_after_the_crash"
            , geometry.label));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_next_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=overwrite_after_the_crash"
                , geometry.label));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} acquired_instance_before_the_crash={} injected_faults=0 crash_points=1 hit={hit} expected_by_k2_table=true first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label, acquired_instance_before_the_crash.0
    ));
}

/// H-C1 直接构造（判决 K2 给丙的具体历史，`m2-rootchoice-repair-r1-main-verification.md:50`：
/// 「12 个」= 4 根槽 + 4 条记录各在两块盘上都读不出，总权重 4×1 + 4×2 = 12）。丙 = 今天的
/// `first_txg_of_new_instance`，同时取根环与记录两路的 max（`mount.rs` 的
/// `highest_ring_txg.max(highest_record_txg)`）——只挡根环（甲-txg 那种攻法）挡不住它：记录那一路
/// 会把真实的 tip txg 重新暴露出来。这里在与甲-txg 同一段历史（n1 = 4，tip 落在顶上 4 条根槽）上，
/// 除了精确点名顶上 4 条根槽（复用 `root_ring_slot_targets_for`，与 PC2 甲同一份坐标系），另外从
/// `evidence_items_at` 取记录类证据项里计数器最大的 4 条（`evidence_items_at` 按 ring 槽位
/// 0..ring_slots 递增枚举、无绕环时恰是计数器递增序，取尾部 4 条即计数器最大的 4 条），两份镜像上
/// 各自的 (设备, 偏移) 全部拦下（每条记录 `targets.len()==2`，在不注入的孪生镜像上取到，权重恰为
/// 2）。验证丙在这个具体构造下是否被打中，是不是恰好需要这 12 个。
fn run_direct_construction_for_the_record_scan_watermark_candidate(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let topmost_four: BTreeSet<TimelineRoot> =
        node.timeline.iter().rev().take(4).copied().collect();
    let root_fault_targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_four);
    let root_target_count = root_fault_targets.len();

    let evidence_items = evidence_items_at(&node.pool, &geometry_view, false);
    let record_items: Vec<&EvidenceItem> = evidence_items
        .iter()
        .filter(|item| item.kind == EvidenceKind::JournalRecord)
        .collect();
    let top_four_records: Vec<&EvidenceItem> = record_items.iter().rev().take(4).copied().collect();
    let top_four_record_labels: Vec<String> = top_four_records
        .iter()
        .map(|item| item.label.clone())
        .collect();
    let record_fault_targets: Vec<(DeviceIdentity, DeviceOffsetInBytes)> = top_four_records
        .iter()
        .flat_map(|item| item.targets.clone())
        .collect();
    let record_target_count = record_fault_targets.len();

    let mut fault_targets = root_fault_targets;
    fault_targets.extend(record_fault_targets);
    let fault_target_count = fault_targets.len();
    let fault_weight = root_target_count + 2 * top_four_records.len();

    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} record_labels={top_four_record_labels:?} root_target_count={root_target_count} record_target_count={record_target_count} fault_target_count={fault_target_count} fault_weight={fault_weight} error={error:?} stage=the_faulted_mount",
                geometry.label
            ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} record_labels={top_four_record_labels:?} root_target_count={root_target_count} record_target_count={record_target_count} fault_target_count={fault_target_count} fault_weight={fault_weight} error={error:?} stage=the_confirmed_overwrite",
                geometry.label
            ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 topmost_four={topmost_four:?} record_labels={top_four_record_labels:?} root_target_count={root_target_count} record_target_count={record_target_count} fault_target_count={fault_target_count} fault_weight={fault_weight} expected_by_k2_table_fault_count=12 hit={hit} expected_by_k2_table_hit=true first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// H-C1 负对照：只拦 4 条根槽、不拦记录（= 甲-txg 那种攻法的目标集合），丙必须不打中——证明
/// 「只挡根环挡不住丙」这句话，不是「随便挡够 4 个目标就够了」。
fn run_negative_control_root_slots_only_for_the_record_scan_watermark_candidate(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let topmost_four: BTreeSet<TimelineRoot> =
        node.timeline.iter().rev().take(4).copied().collect();
    let fault_targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_four);
    let fault_target_count = fault_targets.len();
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_faulted_mount",
                geometry.label
            ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_confirmed_overwrite",
                geometry.label
            ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 topmost_four={topmost_four:?} fault_target_count={fault_target_count} expected_by_k2_table_fault_count=4 hit={hit} expected_hit=false first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// H-C1 下界探针（2026-09-24 session s6；session s9 改参数为 `weight_ceiling`）：直接构造（上面两个
/// 函数）证实丙在 n1=4 这段历史上权重 12 能打中；`run_rootback_tolerance_family` 的主搜索在 n1=4 上
/// 曾撞旧的计数上限提前停手，没有覆盖到权重 12 以下的全部组合。这里在同一个 n1=4 节点上，把
/// `search_minimum_weight_that_triggers_rootback` 的 `weight_ceiling` 显式设成调用方给定的权重，
/// 只为回答一个问题：穷举能不能在这一档权重内走到头（找到一个更小的命中会推翻「恰好 12」，走到权重
/// 12 都没有命中而 12 本身打中会坐实「恰好 12」，`weight_ceiling` 传 `None` 时穷举到完整空间的顶）。
/// 不改判据、不改臂定义，只是把已有搜索函数的一个参数换成调用方要的权重。
fn run_record_scan_watermark_lower_bound_probe(geometry: &Geometry, weight_ceiling: Option<u64>) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let outcome = search_minimum_weight_that_triggers_rootback(
        &node,
        &parameters,
        &geometry_view,
        1,
        weight_ceiling,
        false,
    );
    match outcome.weight_at_first_hit {
        Some(weight) => emit_result(&format!(
            "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} k_min={weight} hit_count_at_that_weight={} first_hit_combination={:?} subsets_tried={} full_space_subset_count={} rootback_probe_failures={} overwrite_failures={}",
            geometry.label,
            outcome.hit_count_at_that_weight,
            outcome.first_hit_combination_labels,
            outcome.subsets_tried,
            outcome.full_space_subset_count,
            outcome.rootback_probe_failures,
            outcome.overwrite_failures
        )),
        None if outcome.stopped_by_weight_ceiling => emit_result(&format!(
            "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} k_min=not_found_up_to_weight_ceiling highest_weight_examined={} maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
            geometry.label, outcome.highest_weight_examined, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
        )),
        None => emit_result(&format!(
            "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} k_min=none_found_within_full_evidence_space maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
            geometry.label, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
        )),
    }
}

// ============================================================================
// 十三点六、H-C2 直接构造（session s8，岔路单第 2 行 Φ2 系统配置槽这一类证据）：
//
//    机制假说：`choose_system_configuration`（`crates/singlefs-core/src/recovery.rs` 第 516 行起
//    现查）按 `reader.device_identities()` 的顺序取**第一块「至少有一份自证槽」的盘**的
//    `best_on_device` 直接当 `chosen`——`chosen` 只在 `None` 时被赋值一次（第 559–560 行），后面的
//    盘只核对 `filesystem_identifier` / `device_count` 一致，**不参与「取哪块盘的值」这一步**。
//    只要把设备 0 上世代号更高的那一槽读失败，`best_on_device` 就回退到设备 0 自己世代号更旧的那
//    一槽（两槽都自证过，见 `system_configuration_slot_positions`），`chosen` 里的 `published_txg`
//    随之读成旧值——设备 1 上真实的新值完全不参与这一步。权重恰为 1。三个函数都用与 H-C1 相同的
//    「append 不截断」挂载路径（`attempt_a_faulted_mount_writable_and_append`），与
//    `run_rootback_tolerance_family_with_system_configuration_evidence` 的穷举路径
//    （`attempt_rootback_probe_and_advance`）是两条独立的代码路径，互为校验。
// ============================================================================

/// 在孪生（不注入）镜像上找设备 0 两个槽里世代号更高的那一个：读原始字节、用
/// `singlefs_checker::check_system_configuration_slot` 校验+解码，取 `slot_generation` 更大的那份。
/// 两槽都读不出或都解不开 ⇒ `None`（这段历史还没轮换过第二次，构造不出，由调用方报错退出）。
fn newest_self_certified_system_configuration_slot_on_device_zero(
    pool: &MemoryPool,
    slot_spacing: u64,
) -> Option<(u64, u64)> {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut newest: Option<(u64, u64)> = None;
    for (device_number, slot_index, offset) in system_configuration_slot_positions(slot_spacing) {
        if device_number != 0 {
            continue;
        }
        let Some(bytes) = pool.read(device_number, offset, slot_bytes) else {
            continue;
        };
        let Ok(view) = singlefs_checker::check_system_configuration_slot(&bytes) else {
            continue;
        };
        if newest.is_none_or(|(_, generation)| view.slot_generation > generation) {
            newest = Some((slot_index, view.slot_generation));
        }
    }
    newest
}

/// 同上，取世代号更小（较旧）的那一份；两槽世代号相等（只轮换过一次）时交回 `None`（负对照这段
/// 历史用不上，跳过不报错）。
fn oldest_self_certified_system_configuration_slot_on_device_zero(
    pool: &MemoryPool,
    slot_spacing: u64,
) -> Option<(u64, u64)> {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut generations: Vec<(u64, u64)> = Vec::new();
    for (device_number, slot_index, offset) in system_configuration_slot_positions(slot_spacing) {
        if device_number != 0 {
            continue;
        }
        let Some(bytes) = pool.read(device_number, offset, slot_bytes) else {
            continue;
        };
        let Ok(view) = singlefs_checker::check_system_configuration_slot(&bytes) else {
            continue;
        };
        generations.push((slot_index, view.slot_generation));
    }
    if generations.len() < 2 {
        return None;
    }
    generations
        .into_iter()
        .min_by_key(|(_, generation)| *generation)
}

/// H-C2 直接构造：只把设备 0 上世代号更高的那一槽读失败（权重 1），验证机制假说描述的打中。
fn run_direct_construction_for_the_system_configuration_evidence_class(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let Some((slot_index, generation)) =
        newest_self_certified_system_configuration_slot_on_device_zero(
            &node.pool,
            geometry_view.slot_spacing,
        )
    else {
        emit_result(&format!(
            "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error=\"设备 0 上没有自证过的系统配置槽\" stage=find_newest_slot",
            geometry.label
        ));
        return;
    };
    let offset = slot_index * geometry_view.slot_spacing;
    let fault_targets = vec![(DeviceIdentity(0), DeviceOffsetInBytes(offset))];
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_faulted_mount",
                    geometry.label
                ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 mechanism_hypothesis=fault_the_newer_generation_slot_on_device_0 hit={hit} first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// 负对照一：只把设备 0 上世代号更旧的那一槽读失败——`best_on_device` 应当仍旧解出自证过的、
/// 世代号更高的那一槽，`chosen` 不受影响，必须不打中。历史上两槽世代号还没分开过（只轮换过一次）
/// 时这段历史跳过，报 `skipped=true`，不算「不打中」。
fn run_negative_control_older_slot_for_the_system_configuration_evidence_class(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let Some((slot_index, generation)) =
        oldest_self_certified_system_configuration_slot_on_device_zero(
            &node.pool,
            geometry_view.slot_spacing,
        )
    else {
        emit_result(&format!(
            "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} skipped=true reason=\"设备 0 上两槽世代号还没分开过\"",
            geometry.label
        ));
        return;
    };
    let offset = slot_index * geometry_view.slot_spacing;
    let fault_targets = vec![(DeviceIdentity(0), DeviceOffsetInBytes(offset))];
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_faulted_mount",
                    geometry.label
                ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 skipped=false hit={hit} expected_hit=false first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// 负对照二：只把设备 1 上世代号更高的那一槽读失败（不碰设备 0）——机制假说说的是「先到手的那块盘
/// 说了算」，设备 0 排在前面、没被碰过，`chosen` 应当仍取设备 0 的当前值，必须不打中。
fn run_negative_control_other_device_for_the_system_configuration_evidence_class(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut newest_on_device_one: Option<(u64, u64)> = None;
    for (device_number, slot_index, offset) in
        system_configuration_slot_positions(geometry_view.slot_spacing)
    {
        if device_number != 1 {
            continue;
        }
        let Some(bytes) = node.pool.read(device_number, offset, slot_bytes) else {
            continue;
        };
        let Ok(view) = singlefs_checker::check_system_configuration_slot(&bytes) else {
            continue;
        };
        if newest_on_device_one.is_none_or(|(_, generation)| view.slot_generation > generation) {
            newest_on_device_one = Some((slot_index, view.slot_generation));
        }
    }
    let Some((slot_index, generation)) = newest_on_device_one else {
        emit_result(&format!(
            "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error=\"设备 1 上没有自证过的系统配置槽\" stage=find_newest_slot",
            geometry.label
        ));
        return;
    };
    let offset = slot_index * geometry_view.slot_spacing;
    let fault_targets = vec![(DeviceIdentity(1), DeviceOffsetInBytes(offset))];
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device1_faulted_slot_index={slot_index} device1_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_faulted_mount",
                    geometry.label
                ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device1_faulted_slot_index={slot_index} device1_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 device1_faulted_slot_index={slot_index} device1_faulted_slot_generation={generation} fault_target_count=1 hit={hit} expected_hit=false first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

// ============================================================================
// 十四、Q2-2a（跑前登记岔路单第 2 行 ②）：固定脚本上，这一份 `crates/` 每次发布按结构种类写的字节。
//
//    口径上的收窄：`WritesByStructureKind` 的写法本身按「一次写调用 = 交给一块盘的一次写，镜像的
//    两份各算一次」计（`write_accounting.rs` 顶部注释），与登记原文「每块盘收到的写的长度之和」是
//    同一个数——这一段不必另建按 (设备, 偏移) 分类的字节计数器。「该臂减今天」由报告在两份产物之间
//    外部 diff 完成，装置本身只报各自的原始值。
// ============================================================================

fn writes_of_transaction_output(output: &TransactionOutput) -> WritesByStructureKind {
    output.writes.clone()
}

fn emit_writes_by_kind(publish_label: &str, writes: &WritesByStructureKind) {
    let total = writes.total();
    emit_result(&format!(
        "name=q2_2a_publish_writes publish={publish_label} kind=total write_calls={} written_bytes={}",
        total.write_calls, total.written_bytes
    ));
    for kind in WrittenStructureKind::IN_REPORT_ORDER {
        let value = writes.of(kind);
        if value.write_calls > 0 {
            emit_result(&format!(
                "name=q2_2a_publish_writes publish={publish_label} kind={kind:?} write_calls={} written_bytes={}",
                value.write_calls, value.written_bytes
            ));
        }
    }
}

/// Q2-2a 的固定脚本（跑前登记 5.8 表 Q2-2）：mkfs → 可写挂载 → 首个文件 → 3 次覆盖写 → 关闭 →
/// 可写挂载 → 2 次覆盖写 → 关闭 → 回退到首个文件那条根 → 1 次覆盖写。逐次发布报字节，按发布种类
/// （写行、暖机、覆盖写）分行。
fn run_fixed_publication_byte_script(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let node = match bootstrap(geometry, 3) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!("name=q2_2a_bootstrap_failed error={error:?}"));
            return;
        }
    };
    let Some(first_file_root) = node
        .timeline
        .iter()
        .find(|root| matches!(node.content_by_root.get(root), Some(Some(_))))
        .copied()
    else {
        emit_result("name=q2_2a_bootstrap_failed reason=\"timeline 里没有带文件的一版\"");
        return;
    };

    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            emit_result(&format!(
                "name=q2_2a_second_mount_writable_failed error={error:?}"
            ));
            return;
        }
    };
    let row_publish_writes = match &mounted.output.row_publish {
        PoolVersion::WithoutFile(version) => version.writes.clone(),
        PoolVersion::WithFile(version) => version.writes.clone(),
    };
    emit_writes_by_kind("second_mount_row_publish", &row_publish_writes);
    for (index, warm_up_publish) in mounted.output.warm_up_publishes.iter().enumerate() {
        let writes = match warm_up_publish {
            PoolVersion::WithoutFile(version) => version.writes.clone(),
            PoolVersion::WithFile(version) => version.writes.clone(),
        };
        emit_writes_by_kind(&format!("second_mount_warm_up_publish_{index}"), &writes);
    }

    let mut allocator = mounted.allocator;
    let mut current = mounted.current;
    let instance = mounted.output.instance;
    for step in 0..2u64 {
        let PoolVersion::WithFile(previous) = &current else {
            emit_result("name=q2_2a_overwrite_failed reason=\"现行版本没有文件\"");
            return;
        };
        let content = deterministic_content(1000 + step, OVERWRITE_FILE_BYTES);
        let output = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            match publish_overwrite(
                &mut writer,
                &mut allocator,
                previous,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 100 + step,
                },
                instance,
            ) {
                Ok(output) => output,
                Err(error) => {
                    emit_result(&format!("name=q2_2a_overwrite_failed error={error:?}"));
                    return;
                }
            }
        };
        emit_writes_by_kind(
            &format!("second_mount_overwrite_{step}"),
            &writes_of_transaction_output(&output),
        );
        current = PoolVersion::WithFile(output);
    }

    let target = RollbackTarget {
        instance: InstanceGeneration(first_file_root.0),
        checkpoint_txg: CheckpointTxg(first_file_root.1),
    };
    let rollback_mounted = match mount_rollback(&parameters, &mut devices, target, ShadowLedger::On)
    {
        Ok(rollback_mount_output) => rollback_mount_output,
        Err(error) => {
            emit_result(&format!("name=q2_2a_mount_rollback_failed error={error:?}"));
            return;
        }
    };
    let rollback_row_publish_writes = match &rollback_mounted.output.row_publish {
        PoolVersion::WithoutFile(version) => version.writes.clone(),
        PoolVersion::WithFile(version) => version.writes.clone(),
    };
    emit_writes_by_kind("rollback_row_publish", &rollback_row_publish_writes);
    for (index, warm_up_publish) in rollback_mounted.output.warm_up_publishes.iter().enumerate() {
        let writes = match warm_up_publish {
            PoolVersion::WithoutFile(version) => version.writes.clone(),
            PoolVersion::WithFile(version) => version.writes.clone(),
        };
        emit_writes_by_kind(&format!("rollback_warm_up_publish_{index}"), &writes);
    }

    let mut rollback_allocator = rollback_mounted.allocator;
    let PoolVersion::WithFile(previous) = &rollback_mounted.current else {
        emit_result("name=q2_2a_final_overwrite_failed reason=\"回退之后现行版本没有文件\"");
        return;
    };
    let content = deterministic_content(2000, OVERWRITE_FILE_BYTES);
    let output = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        match publish_overwrite(
            &mut writer,
            &mut rollback_allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 9999,
            },
            rollback_mounted.output.instance,
        ) {
            Ok(output) => output,
            Err(error) => {
                emit_result(&format!(
                    "name=q2_2a_final_overwrite_failed error={error:?}"
                ));
                return;
            }
        }
    };
    emit_writes_by_kind(
        "final_overwrite_after_rollback",
        &writes_of_transaction_output(&output),
    );
}

// ============================================================================
// 十、第 2 次跑（`research/prompts/e158-r2-prereg.md`）：5.6 常量回比与第七节锚点（第 2 次跑那一份）、
//    H1d（5.5 末尾主 agent 22:5x 跑前修订加的历史族）、Q1-0（H1c 的抛弃步，前提 1 在装置上的读数）、
//    实七两段复现历史。这一节只加：第一次跑的各模式（上面一至九）原样留着，它们的旧产物由
//    `research/scripts/replay.sh` 里 E158 那几行复跑；第一节处理表里的删与改归后面的段。
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;

/// 这一次跑在哪一条臂的副本上：只当标签打进结果行。装置源码各臂同一份（跑前登记「装置写在哪」），
/// 差别只在副本的 `crates/` 与跑的时候设的这个环境变量；没设就是工作树（今天那一臂）。
fn second_run_arm_label() -> String {
    env::var("SINGLEFS_E158_ARM").unwrap_or_else(|_| "today".to_string())
}

/// 系统配置字段表合计今天的值（D22（单元原子性怎么合成） 已定项 9；跑前登记 5.6 表「系统配置字段表合计」一行）。
const SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL: u64 = 489;
/// 系统配置槽宽（跑前登记 5.6；`singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES` 回比）。
const SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL: u64 = 4096;
/// 字段表四档（D22 已定项 9 的分段表；跑前登记 7.2 第四行）：不可变 389、可变 4、运行配置 36、运行量 60（今天）。
const IMMUTABLE_TIER_BYTES_LOCAL: u64 = 389;
const MUTABLE_TIER_BYTES_LOCAL: u64 = 4;
const RUNTIME_CONFIGURATION_TIER_BYTES_LOCAL: u64 = 36;
const RUNTIME_QUANTITIES_TIER_BYTES_TODAY_LOCAL: u64 = 60;
/// 一次准入最多几次发布（D16（发布语义） 已定项 1「准入」：B = 4 + 2 k_tol，k_tol = 2；跑前登记 5.6）。
const PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL: u64 = 4 + 2 * 2;
/// 512 字节物理写原子边界：`crates/` 里没有，出处 D22 已定项 9 正文，条款锚不回比（跑前登记 5.6）。
const PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES: u64 = 512;
/// 落点粒度（跑前登记 5.6 `SLOT_BYTES`）：第 2 次跑算单元字节区间用的就是这一个。
const SLOT_BYTES_LOCAL: u64 = 16384;
/// 这个测试周期的种子基（抄自 `crates/singlefs-checker-tier/src/crash_injection.rs:70` `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，
/// 开跑时回比）；实七报告「交主 agent 的」第 1 条点名的两段历史都从它数（`research/prompts/m2-impl7-implementer-report.md`）。
const SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL: u64 = 7_463_871_032_432_355_113;
/// 故障注入大档那一段：种子基 + 110、每段 30 步、注入 6 次（大档默认规模，
/// `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs` 大档用例的缺省值）。
const SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET: u64 = 110;
const SEVENTH_BATCH_FAULT_REPRODUCTION_OPERATIONS: usize = 30;
const SEVENTH_BATCH_FAULT_REPRODUCTION_FAULTS: usize = 6;
/// 崩溃注入快档那一段：种子基 + 16、每段 24 步、每段抽 4 个崩溃状态（快档规模，
/// `crates/singlefs-checker-tier/tests/second_transaction_supplement_three_crash_injection.rs` 的 `FAST_TIER_*`）。
const SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET: u64 = 16;
const SEVENTH_BATCH_CRASH_REPRODUCTION_OPERATIONS: usize = 24;
const SEVENTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS: usize = 4;

/// 跑前登记 5.6 表：第一次跑那一份（`local_constants_checks`）之外，第 2 次跑另要回比的几项。
fn second_run_local_constants_checks() -> Vec<ConstantCheck> {
    let mut checks = local_constants_checks();
    checks.extend([
        ConstantCheck {
            name: "系统配置槽宽（第 2 次跑的锚点用的那一个）",
            local_value: SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
        },
        ConstantCheck {
            name: "字段表不可变一档 SystemImmutableConfiguration::FIELD_TABLE_BYTES",
            local_value: IMMUTABLE_TIER_BYTES_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemImmutableConfiguration::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "字段表可变一档 SystemMutableConfiguration::FIELD_TABLE_BYTES",
            local_value: MUTABLE_TIER_BYTES_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemMutableConfiguration::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "字段表运行配置一档 SystemRuntimeConfiguration::FIELD_TABLE_BYTES",
            local_value: RUNTIME_CONFIGURATION_TIER_BYTES_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemRuntimeConfiguration::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "字段表运行量一档 SystemRuntimeQuantities::FIELD_TABLE_BYTES（乙 / 丁四臂按臂的定义多 8）",
            local_value: RUNTIME_QUANTITIES_TIER_BYTES_TODAY_LOCAL + local_system_configuration_bytes()
                - SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemRuntimeQuantities::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "一次准入最多几次发布 PUBLISHES_PER_ADMISSION_AT_MOST",
            local_value: PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL,
            crates_value: u64::try_from(singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST)
                .expect("8 装得进 u64"),
        },
        ConstantCheck {
            name: "落点粒度（第 2 次跑算单元字节区间用的那一个）",
            local_value: SLOT_BYTES_LOCAL,
            crates_value: singlefs_format::SLOT_BYTES,
        },
        ConstantCheck {
            name: "这个测试周期的种子基 SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE",
            local_value: SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL,
            crates_value: singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        },
    ]);
    checks
}

/// 在一份系统配置槽的字节里找一段小端字节恰好出现一次的位置；不是恰好一次交 None。
fn unique_offset_of(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let mut found = None;
    for start in 0..=haystack.len().saturating_sub(needle.len()) {
        if haystack[start..start + needle.len()] == *needle {
            if found.is_some() {
                return None;
            }
            found = Some(start);
        }
    }
    found
}

/// journal tail、实例代号、回退下界 F 三项在系统配置槽里住哪（跑前登记 5.6「私有常量引不到 ⇒ 装置写一槽已知的 tail、
/// 实例代号与 F，自己解码找偏移」）：mkfs 一个池，按 `crates` 的解析读回盘 0 第 0 槽，改三项为已知值、再按 `crates` 写成槽字节，
/// 按小端字节找它们。交回 (tail, 实例代号, F) 三个偏移。
fn decoded_tail_instance_and_floor_offsets() -> Result<(usize, usize, usize), String> {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut devices = new_devices(IMAGE_BYTES);
    make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    let mut slot = vec![0u8; slot_bytes];
    devices[0]
        .1
        .read_at(DeviceOffsetInBytes(0), &mut slot)
        .map_err(|error| format!("读系统配置槽: {error:?}"))?;
    let mut parsed = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&slot)
        .map_err(|error| format!("解系统配置槽: {error:?}"))?;
    let known_tail: u64 = 0x1122_3344_5566_7788;
    let known_instance: u32 = 0xA1B2_C3D4;
    let known_floor: u64 = 0x0102_0304_0506_0708;
    parsed.quantities.journal_tail = known_tail;
    parsed.quantities.journal_instance = InstanceGeneration(known_instance);
    parsed.quantities.rollback_floor = CheckpointTxg(known_floor);
    let written = parsed.to_slot();
    let tail = unique_offset_of(&written, &known_tail.to_le_bytes())
        .ok_or("已知 tail 在槽里不是恰好一处")?;
    let instance = unique_offset_of(&written, &known_instance.to_le_bytes())
        .ok_or("已知实例代号在槽里不是恰好一处")?;
    let floor = unique_offset_of(&written, &known_floor.to_le_bytes())
        .ok_or("已知 F 在槽里不是恰好一处")?;
    Ok((tail, instance, floor))
}

/// 跑前登记 7.2：独立算出、用命令核过的值（登记第十三节那条 python 的输出），写成「由本地常量现算 == 字面值」，
/// 常量断言写加法不写减法（`test-discipline.md`「常量断言写加法」）。乙 / 丁四臂（本地字段表 497）那几行换成 7.2 第三行的数。
fn second_run_arithmetic_anchors(local_system_configuration: u64) -> Vec<(&'static str, u64, u64)> {
    let mut anchors = vec![
        (
            "字段表四档 389+4+36+60（7.2 第四行）",
            IMMUTABLE_TIER_BYTES_LOCAL
                + MUTABLE_TIER_BYTES_LOCAL
                + RUNTIME_CONFIGURATION_TIER_BYTES_LOCAL
                + RUNTIME_QUANTITIES_TIER_BYTES_TODAY_LOCAL,
            489,
        ),
        (
            "运行量一档 8+32+8+4+8（7.2 第四行）",
            8 + 32 + 8 + 4 + 8,
            60,
        ),
        (
            "乙 / 丁字段表 489+8（7.2 第三行）",
            SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL + 8,
            497,
        ),
        (
            "系统配置整槽校验和偏移 4+2+96+16+20+1+4+4+8（7.2 第六行）",
            4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8,
            155,
        ),
        ("根槽总数 3×S，S=4（7.2 第七行）", 3 * 4, 12),
        (
            "根槽总数 3×S，S=8（7.2 第七行）",
            3 * GEOMETRY_PRIMARY.slots_per_region,
            24,
        ),
        (
            "根槽总数 3×S，S=16（7.2 第七行）",
            3 * GEOMETRY_LARGER_ROOT_RING.slots_per_region,
            48,
        ),
        (
            "环槽数 768MiB÷4096（7.2 第八行）",
            (768 * 1024 * 1024) / JOURNAL_RECORD_BYTES,
            196_608,
        ),
        (
            "环槽数 3MiB÷4096（7.2 第八行）",
            GEOMETRY_PRIMARY.journal_ring_bytes / JOURNAL_RECORD_BYTES,
            768,
        ),
        (
            "环槽数 48KiB÷4096（7.2 第八行）",
            (48 * 1024) / JOURNAL_RECORD_BYTES,
            12,
        ),
        (
            "实例表行宽 1+4+8+8+1+66（7.2 第九行）",
            1 + 4 + 8 + 8 + 1 + 66,
            88,
        ),
        (
            "两个指称各一个索引节点两份 16384×2（7.2 第十一行）",
            SLOT_BYTES_LOCAL * 2,
            32768,
        ),
        ("Φ1 每条根 T、L 都有时 3+3+1（7.2 第十二行）", 3 + 3 + 1, 7),
        (
            "今天那一臂每次发布系统配置那一类写字节 4096×2（7.2 第十行）",
            SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL * 2,
            8192,
        ),
    ];
    if local_system_configuration == SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL {
        anchors.push((
            "系统配置槽余量：槽宽 == 字段表 + 3607（7.1 第一行、7.2 第一行）",
            SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            local_system_configuration + 3607,
        ));
        anchors.push((
            "512 == 字段表 + 23（7.2 第二行）",
            PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES,
            local_system_configuration + 23,
        ));
    } else {
        anchors.push((
            "乙 / 丁：槽宽 == 字段表 + 3599（7.2 第三行）",
            SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            local_system_configuration + 3599,
        ));
        anchors.push((
            "乙 / 丁：512 == 字段表 + 15（7.2 第三行）",
            PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES,
            local_system_configuration + 15,
        ));
    }
    anchors
}

/// 第 2 次跑的开跑检查：5.6 常量回比（V3）、7.2 算术（V4）、7.1 第一行里「字段表不跨 512」与偏移解码（S3）。
/// 交回 true = 全部过；任一不过交 false，调用方整轮不开跑。
fn run_second_run_constants_and_anchors(arm: &str) -> bool {
    let mut all_passed = true;
    for check in second_run_local_constants_checks() {
        let passed = check.matches();
        all_passed &= passed;
        emit_result(&format!(
            "name=r2_local_constant_check arm={arm} item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            if passed { "pass" } else { "fail" },
            check.local_value,
            check.crates_value
        ));
    }
    let (regions_match, local_regions, crates_regions) = region_devices_matches_crates();
    all_passed &= regions_match;
    emit_result(&format!(
        "name=r2_local_constant_check arm={arm} item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        if regions_match { "pass" } else { "fail" }
    ));
    let local_system_configuration = local_system_configuration_bytes();
    for (item, computed, registered) in second_run_arithmetic_anchors(local_system_configuration) {
        let passed = computed == registered;
        all_passed &= passed;
        emit_result(&format!(
            "name=r2_section_seven_two_anchor arm={arm} item={item:?} verdict={} computed={computed} registered={registered}",
            if passed { "pass" } else { "fail" }
        ));
    }
    let within_one_physical_write =
        local_system_configuration <= PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES;
    all_passed &= within_one_physical_write;
    emit_result(&format!(
        "name=r2_section_seven_one_anchor arm={arm} item=\"D22 已定项 9：字段表不跨 512\" verdict={} local_system_configuration_bytes={local_system_configuration}",
        if within_one_physical_write { "pass" } else { "fail" }
    ));
    match decoded_tail_instance_and_floor_offsets() {
        Ok((tail, instance, floor)) => {
            let tail_passed = tail == 469;
            let floor_passed = floor == 481 && instance + 4 == floor && tail + 8 == instance;
            all_passed &= tail_passed && floor_passed;
            emit_result(&format!(
                "name=r2_decoded_offsets arm={arm} verdict={} tail_offset={tail} instance_offset={instance} floor_offset={floor} registered_tail=469 registered_floor=481",
                if tail_passed && floor_passed { "pass" } else { "fail" }
            ));
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r2_decoded_offsets arm={arm} verdict=fail reason={error:?}"
            ));
        }
    }
    all_passed
}

// ---------------------------------------------------------------------------
// 10.1 读法写死表「故障」的落地：一块盘一层、一次装完这块盘的全部落点；读落点按区间罩（不只认起点偏移）；
//      每个落点在这次调用里被拦下的读次数单独记（V2）；这次调用自己写到落点上之后那一处不再拦
//      （写出的字节此后照常读得出，同 `crates/singlefs-harness/src/history.rs` 的 `DeviceReadingZerosOverHiddenRanges`）；
//      另可让整池第 n 次写或第 n 次屏障报错（H1d 的「注入写失败」）。设备一层真落了的写按次序记下（K3 / K4 与崩溃状态靠它）。
// ---------------------------------------------------------------------------

/// 一个落点这次调用里读不出的样子（H1d 两种造法都做：注入读错、读回清零）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnreadableForm {
    ReadFails,
    ReadsZeros,
}

impl UnreadableForm {
    fn name(self) -> &'static str {
        match self {
            UnreadableForm::ReadFails => "read_fails",
            UnreadableForm::ReadsZeros => "reads_zeros",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UnreadableRange {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    form: UnreadableForm,
    label: String,
}

impl UnreadableRange {
    fn overlaps(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && self.offset < offset + length && offset < self.offset + self.length
    }
}

/// 设备一层真落了的一步（按整池的先后）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum AppliedDeviceStep {
    Write {
        device: DeviceIdentity,
        offset: u64,
        bytes: Vec<u8>,
    },
    ZeroFill {
        device: DeviceIdentity,
        offset: u64,
        length: u64,
    },
    Barrier {
        device: DeviceIdentity,
    },
}

#[derive(Default)]
struct InstrumentationState {
    unreadable: Vec<UnreadableRange>,
    written_over: Vec<bool>,
    intercepted_reads: Vec<u64>,
    /// 整池发出的写（含报了错的那一次），从 0 数。
    write_calls: u64,
    barrier_calls: u64,
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
    injected_failure_fired: bool,
    applied: Vec<AppliedDeviceStep>,
}

impl InstrumentationState {
    fn new(
        unreadable: Vec<UnreadableRange>,
        failing_write_call: Option<u64>,
        failing_barrier_call: Option<u64>,
    ) -> Self {
        let count = unreadable.len();
        InstrumentationState {
            unreadable,
            written_over: vec![false; count],
            intercepted_reads: vec![0; count],
            failing_write_call,
            failing_barrier_call,
            ..InstrumentationState::default()
        }
    }

    fn mark_written_over(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        for (range, written_over) in self.unreadable.iter().zip(self.written_over.iter_mut()) {
            if range.overlaps(device, offset, length) {
                *written_over = true;
            }
        }
    }
}

struct InstrumentedDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    state: Rc<RefCell<InstrumentationState>>,
}

impl BlockDevice for InstrumentedDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let mut fails = false;
        let mut zeroed: Vec<(u64, u64)> = Vec::new();
        {
            let mut state = self.state.borrow_mut();
            let state = &mut *state;
            for ((range, written_over), intercepted) in state
                .unreadable
                .iter()
                .zip(state.written_over.iter())
                .zip(state.intercepted_reads.iter_mut())
            {
                if *written_over || !range.overlaps(self.identity, offset.0, length) {
                    continue;
                }
                *intercepted += 1;
                match range.form {
                    UnreadableForm::ReadFails => fails = true,
                    UnreadableForm::ReadsZeros => zeroed.push((range.offset, range.length)),
                }
            }
        }
        if fails {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)?;
        for (range_offset, range_length) in zeroed {
            let start = range_offset.saturating_sub(offset.0);
            let end = (range_offset + range_length - offset.0).min(length);
            buffer[usize::try_from(start).expect("缓冲区里")
                ..usize::try_from(end).expect("缓冲区里")]
                .fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        if state.failing_write_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)?;
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::Write {
            device: self.identity,
            offset: offset.0,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        if state.failing_write_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_zeroes_at(offset, length)?;
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::ZeroFill {
            device: self.identity,
            offset: offset.0,
            length,
        });
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.barrier_calls;
        state.barrier_calls += 1;
        if state.failing_barrier_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("屏障"));
        }
        self.inner.barrier()?;
        state.applied.push(AppliedDeviceStep::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一次受观测的可写挂载交回的东西。
struct InstrumentedMount {
    result: Result<Mounted, singlefs_core::mount::MountError>,
    /// 整池发出的写次数（含报了错的那一次）：K3 / K4 按它是不是 0 分（跑前登记前提 3 的答案）。
    write_calls: u64,
    applied: Vec<AppliedDeviceStep>,
    intercepted_reads: Vec<u64>,
    injected_failure_fired: bool,
    pool_after: MemoryPool,
}

fn run_instrumented_mount_writable(
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    unreadable: &[UnreadableRange],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
) -> InstrumentedMount {
    let state = Rc::new(RefCell::new(InstrumentationState::new(
        unreadable.to_vec(),
        failing_write_call,
        failing_barrier_call,
    )));
    let mut devices: Vec<(DeviceIdentity, InstrumentedDevice)> =
        devices_from_pool(base, IMAGE_BYTES)
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    InstrumentedDevice {
                        identity,
                        inner,
                        state: Rc::clone(&state),
                    },
                )
            })
            .collect();
    let result = mount_writable(parameters, &mut devices);
    let plain: Vec<(DeviceIdentity, SparseBlockDevice)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    let pool_after = memory_pool_of(&plain, IMAGE_BYTES);
    let state = state.borrow();
    InstrumentedMount {
        result,
        write_calls: state.write_calls,
        applied: state.applied.clone(),
        intercepted_reads: state.intercepted_reads.clone(),
        injected_failure_fired: state.injected_failure_fired,
        pool_after,
    }
}

/// 一次可写挂载的结局按 K0–K4 分（跑前登记前提 3 的答案）：K0 做成且取号之前判过（或测试开关没判）、K1 写行之后推过抬 F 够了、
/// K2 推满仍不够而挂载照样做成、K3 报错且这次调用设备一层 0 次写、K4 报错且写过。
fn mount_outcome_class(
    result: &Result<Mounted, singlefs_core::mount::MountError>,
    write_calls: u64,
) -> &'static str {
    use singlefs_core::mount::MountSpaceAdmission;
    match result {
        Ok(mounted) => match mounted.output.space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition
            | MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "K0",
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => "K1",
            MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => "K2",
        },
        Err(_) if write_calls == 0 => "K3",
        Err(_) => "K4",
    }
}

fn mount_error_member_or_none(
    result: &Result<Mounted, singlefs_core::mount::MountError>,
) -> String {
    match result {
        Ok(_) => "none".to_string(),
        Err(error) => error_member_of_debug(&format!("{error:?}")),
    }
}

/// `base` 上依次施加 `applied` 里前 `write_count` 个写（屏障不改字节），交回那份镜像：崩溃在第 `write_count` 个写之前、
/// 之前的写都已持久的那一个崩溃状态（前缀）。
fn image_with_the_first_writes(
    base: &MemoryPool,
    applied: &[AppliedDeviceStep],
    write_count: usize,
) -> MemoryPool {
    let mut devices = devices_from_pool(base, IMAGE_BYTES);
    let mut written = 0usize;
    for step in applied {
        if written == write_count {
            break;
        }
        match step {
            AppliedDeviceStep::Write {
                device,
                offset,
                bytes,
            } => {
                if let Some((_, target)) =
                    devices.iter_mut().find(|(identity, _)| identity == device)
                {
                    target
                        .write_at(DeviceOffsetInBytes(*offset), bytes, WriteDurability::Plain)
                        .expect("内存盘写不报错");
                }
                written += 1;
            }
            AppliedDeviceStep::ZeroFill {
                device,
                offset,
                length,
            } => {
                if let Some((_, target)) =
                    devices.iter_mut().find(|(identity, _)| identity == device)
                {
                    target
                        .write_zeroes_at(DeviceOffsetInBytes(*offset), *length)
                        .expect("内存盘清零不报错");
                }
                written += 1;
            }
            AppliedDeviceStep::Barrier { .. } => {}
        }
    }
    memory_pool_of(&devices, IMAGE_BYTES)
}

/// 根环每个槽在哪块盘的哪个偏移（checker 的独立解码，`checker_image::root_slot_positions`）。
fn root_slot_device_offsets(geometry: &PoolGeometry) -> Vec<(DeviceIdentity, u64)> {
    checker_image::root_slot_positions(geometry)
        .into_iter()
        .map(|(_, _, device, offset)| (DeviceIdentity(device), offset))
        .collect()
}

/// 设备一层的第几个写（从 0 数）是第一个落到根环槽上的：可写挂载里第一个根槽写就是写行那次发布的根（取号只写系统配置）。
fn index_of_the_first_root_slot_write(
    applied: &[AppliedDeviceStep],
    root_slots: &[(DeviceIdentity, u64)],
    root_slot_bytes: u64,
) -> Option<usize> {
    let mut write_index = 0usize;
    for step in applied {
        let (device, offset, length) = match step {
            AppliedDeviceStep::Write {
                device,
                offset,
                bytes,
            } => (*device, *offset, u64::try_from(bytes.len()).expect("写长")),
            AppliedDeviceStep::ZeroFill {
                device,
                offset,
                length,
            } => (*device, *offset, *length),
            AppliedDeviceStep::Barrier { .. } => continue,
        };
        if root_slots.iter().any(|(slot_device, slot_offset)| {
            *slot_device == device
                && *slot_offset < offset + length
                && offset < slot_offset + root_slot_bytes
        }) {
            return Some(write_index);
        }
        write_index += 1;
    }
    None
}

/// 前 `write_count` 个写之间发过几次屏障（整池，从 0 数的屏障序号都小于它）。
fn barriers_before_write(applied: &[AppliedDeviceStep], write_count: usize) -> u64 {
    let mut writes = 0usize;
    let mut barriers = 0u64;
    for step in applied {
        match step {
            AppliedDeviceStep::Barrier { .. } => {
                if writes < write_count {
                    barriers += 1;
                }
            }
            AppliedDeviceStep::Write { .. } | AppliedDeviceStep::ZeroFill { .. } => {
                writes += 1;
                if writes >= write_count {
                    break;
                }
            }
        }
    }
    barriers
}

// ---------------------------------------------------------------------------
// 10.2 历史：mkfs → `mount_writable`（实例 1）→ 首个文件 → n1 次覆盖写 → 关闭（C：丢掉会话；U：调 `unmount`）。
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Closing {
    ProcessExit,
    NormalUnmount,
}

impl Closing {
    fn name(self) -> &'static str {
        match self {
            Closing::ProcessExit => "C",
            Closing::NormalUnmount => "U",
        }
    }
}

struct SecondRunHistory {
    pool: MemoryPool,
    session: Option<Session>,
    /// 当前时间线（父链，跑前登记 5.0）：mkfs 的第 0 代根起，到最新一次成功发布的根。
    timeline: Vec<TimelineRoot>,
    content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>,
    publishes_with_content: u64,
}

impl SecondRunHistory {
    fn tip(&self) -> TimelineRoot {
        *self.timeline.last().expect("时间线至少有第 0 代根")
    }

    fn tip_content(&self) -> Option<Vec<u8>> {
        self.content_by_root
            .get(&self.tip())
            .cloned()
            .unwrap_or(None)
    }
}

fn push_mounted_publishes(
    history: &mut SecondRunHistory,
    mounted: &Mounted,
    carried_forward_content: &Option<Vec<u8>>,
) {
    let mut roots = vec![root_pair(mounted.output.row_publish.root())];
    roots.extend(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| root_pair(publish.root())),
    );
    for raised in mounted.output.space_admission.floor_raises() {
        roots.extend(
            raised
                .publishes
                .iter()
                .map(|publish| root_pair(&publish.root)),
        );
    }
    for root in roots {
        history.timeline.push(root);
        history
            .content_by_root
            .insert(root, carried_forward_content.clone());
    }
}

fn second_run_start(geometry: &Geometry, overwrites: u64) -> Result<SecondRunHistory, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let mounted = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("第一次可写挂载: {error:?}"))?;
    let mut history = SecondRunHistory {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: None,
        timeline: vec![root_pair(&genesis.root)],
        content_by_root: BTreeMap::from([(root_pair(&genesis.root), None)]),
        publishes_with_content: 0,
    };
    push_mounted_publishes(&mut history, &mounted, &None);
    let mut allocator = mounted.allocator;
    let instance = mounted.output.instance;
    let current = mounted.current;
    let content = deterministic_content(1000, FIRST_FILE_BYTES);
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            current.record_bytes(),
        )
        .map_err(|error| format!("首个文件: {error:?}"))?
    };
    history.timeline.push(root_pair(&first.root));
    history
        .content_by_root
        .insert(root_pair(&first.root), Some(content));
    history.publishes_with_content = 1;
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    history.session = Some(Session {
        allocator,
        current: PoolVersion::WithFile(first),
        instance,
    });
    for _ in 0..overwrites {
        second_run_overwrite(&mut history, &parameters)?;
    }
    Ok(history)
}

fn second_run_overwrite(
    history: &mut SecondRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<(), String> {
    let session = history.session.as_mut().ok_or("覆盖写：没有开着的会话")?;
    let PoolVersion::WithFile(previous) = &session.current else {
        return Err("覆盖写：现行版本没有文件".to_string());
    };
    let tag = 1000 + history.publishes_with_content;
    let content = deterministic_content(tag, OVERWRITE_FILE_BYTES);
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut session.allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * tag,
            },
            session.instance,
        )
        .map_err(|error| format!("覆盖写: {error:?}"))?
    };
    history.timeline.push(root_pair(&output.root));
    history
        .content_by_root
        .insert(root_pair(&output.root), Some(content));
    history.publishes_with_content += 1;
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    session.current = PoolVersion::WithFile(output);
    Ok(())
}

/// 每块盘两槽里世代号最大的那一份自证系统配置槽的字节（按 `crates` 的解析判自证）。
fn newest_system_configuration_slot_bytes(
    pool: &MemoryPool,
    device: DeviceIdentity,
    slot_spacing: u64,
) -> Option<Vec<u8>> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    [0u64, slot_spacing]
        .into_iter()
        .filter_map(|offset| pool.read(device.0, offset, slot_bytes))
        .filter_map(|bytes| {
            singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
                .ok()
                .map(|parsed| (parsed.quantities.slot_generation, bytes))
        })
        .max_by_key(|(generation, _)| *generation)
        .map(|(_, bytes)| bytes)
}

/// 关闭 c：C 丢掉会话；U 调 `unmount`，并核 7.1 第四行（F12）与第一行的盘上那一半（最后一个非零字节、乙族 Q2-2b 的解码）。
fn second_run_close(
    history: &mut SecondRunHistory,
    parameters: &MakeFilesystemParameters,
    closing: Closing,
    arm: &str,
    context: &str,
) -> Result<(), String> {
    let session = history.session.take().ok_or("关闭：没有开着的会话")?;
    if closing == Closing::ProcessExit {
        return Ok(());
    }
    let Session {
        mut allocator,
        mut current,
        ..
    } = session;
    let before_txg = current.root().checkpoint_txg.0;
    let carried = history.tip_content();
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let unmounted = singlefs_core::mount::unmount(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .map_err(|error| format!("正常卸载: {error:?}"))?;
    let singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        return Err("正常卸载：现行那一版没有文件".to_string());
    };
    for publish in &raised.publishes {
        history.timeline.push(root_pair(&publish.root));
        history
            .content_by_root
            .insert(root_pair(&publish.root), carried.clone());
    }
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let expected_publishes = if before_txg % 3 == 1 { 3 } else { 2 };
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let mut floors = Vec::new();
    let mut last_nonzero = Vec::new();
    let mut published_txg_decoded = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let Some(bytes) =
            newest_system_configuration_slot_bytes(&history.pool, device, slot_spacing)
        else {
            return Err(format!("正常卸载之后盘 {} 没有自证的系统配置槽", device.0));
        };
        let parsed = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
            .map_err(|error| format!("{error:?}"))?;
        floors.push(parsed.quantities.rollback_floor.0);
        last_nonzero.push(bytes.iter().rposition(|byte| *byte != 0).unwrap_or(0));
        let start = usize::try_from(SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL).expect("489");
        let mut little_endian = [0u8; 8];
        little_endian.copy_from_slice(&bytes[start..start + 8]);
        published_txg_decoded.push(u64::from_le_bytes(little_endian));
    }
    let floor_holds = floors.iter().all(|floor| *floor == before_txg);
    let publishes_hold = raised.publishes.len() == expected_publishes;
    // 7.1 第一行的盘上那一半：今天的字段表（本地 489）下最后一个非零字节落在 F 那 8 字节 [481, 489) 里、489 起全零；
    // 乙 / 丁四臂（本地 497）下 [489, 497) 解出的就是刚落盘那条根的 txg（Q2-2b 的解码核对）。
    let field_end = local_system_configuration_bytes();
    let floor_start = SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL - 8;
    let layout_holds = if field_end == SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL {
        last_nonzero.iter().all(|index| {
            let index = u64::try_from(*index).expect("槽内下标");
            floor_start <= index && index < field_end
        })
    } else {
        published_txg_decoded
            .iter()
            .all(|decoded| *decoded == history.tip().1)
    };
    emit_result(&format!(
        "name=r2_unmount_check arm={arm} context={context:?} verdict={} layout_verdict={} txg_before_unmount={before_txg} floor_in_newest_system_configuration={floors:?} unmount_publishes={} expected_publishes={expected_publishes} last_nonzero_byte={last_nonzero:?} bytes_489_to_497_as_txg={published_txg_decoded:?} newest_root_txg={}",
        if floor_holds && publishes_hold { "pass" } else { "fail_F12" },
        if layout_holds { "pass" } else { "fail_S3" },
        raised.publishes.len(),
        history.tip().1
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// 10.3 装置自己的账：落点、一条根引用的单元、择根、被抛弃判定。
// ---------------------------------------------------------------------------

/// 一条根（(实例, txg)）的根槽在哪（checker 的独立解码：自证过的根里找到它、再按 (区域, 槽) 取 (盘, 偏移)）。
fn root_slot_of(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    root: TimelineRoot,
) -> Option<(DeviceIdentity, u64)> {
    let positions = checker_image::root_slot_positions(geometry);
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .find(|(_, _, view)| (view.instance, view.checkpoint_txg) == root)
        .and_then(|(region, slot, _)| {
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    *candidate_region == region && *candidate_slot == slot
                })
                .map(|(_, _, device, offset)| (DeviceIdentity(*device), *offset))
        })
}

/// 一次发布（按 (实例, txg) 认）在 journal 环里的记录：逐槽解两块盘上的那一份，交回 (计数器, 偏移, 记录)，按计数器排。
fn records_of_publish(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    root: TimelineRoot,
) -> Vec<(u64, u64, JournalRecord)> {
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let mut found: BTreeMap<u64, (u64, JournalRecord)> = BTreeMap::new();
    for slot_index in 0..ring_slots {
        let offset = record_offset(slot_index + 1, geometry.journal_ring_bytes);
        for device_number in [0u32, 1u32] {
            let Some(bytes) = pool.read(device_number, offset.0, record_bytes) else {
                continue;
            };
            let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) else {
                continue;
            };
            if (record.instance.0, record.checkpoint_txg.0) == root {
                found.entry(record.counter).or_insert((offset.0, record));
            }
        }
    }
    found
        .into_iter()
        .map(|(counter, (offset, record))| (counter, offset, record))
        .collect()
}

/// 断链发布的重放怎么接不上（跑前登记 5.1 抛弃步 A(k, b) 的 b）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayBreak {
    /// b = rec：那次发布的全部记录的全部份读不出。
    EveryRecord,
    /// b = unit：那次发布第一条记录点名的第一个单元两份读不出。
    FirstNamedUnit,
    /// b = unit_first_copy（第 2 次跑修订加的，只补不改）：只让那个单元排在位置条目第一条的那一份读不出。
    /// 重放验点名单元要全部份都验过（`recovery.rs` `replay_journal` 里 `named.locations.iter().all`），第一份读错就不再读第二份，
    /// b = unit 的第二份于是一次都拦不到（V2 作废）；这一支量「一份就够」。
    FirstCopyOfTheFirstNamedUnit,
}

impl ReplayBreak {
    fn name(self) -> &'static str {
        match self {
            ReplayBreak::EveryRecord => "records",
            ReplayBreak::FirstNamedUnit => "unit",
            ReplayBreak::FirstCopyOfTheFirstNamedUnit => "unit_first_copy",
        }
    }
}

/// 抛弃步的落点：当前时间线上最新 `newest_roots_hidden` 条根的根槽，加断链发布（当前时间线上第 `newest_roots_hidden` 新的那条根的
/// 那次发布）按 `replay_break` 取的落点。断链发布没有记录、或 b = unit 而它没有点名单元时交 None（这一支「不适用」）。
fn unreadable_ranges_of_the_abandonment(
    history: &SecondRunHistory,
    geometry: &PoolGeometry,
    newest_roots_hidden: usize,
    form: UnreadableForm,
    replay_break: ReplayBreak,
) -> Result<Option<Vec<UnreadableRange>>, String> {
    let count = history.timeline.len();
    if newest_roots_hidden == 0 || newest_roots_hidden >= count {
        return Err(format!(
            "藏 {newest_roots_hidden} 条根：时间线只有 {count} 条"
        ));
    }
    let mut ranges = Vec::new();
    for root in &history.timeline[count - newest_roots_hidden..] {
        let (device, offset) = root_slot_of(&history.pool, geometry, *root)
            .ok_or_else(|| format!("时间线上的根 {root:?} 在根环里读不出（不绕环时不应发生）"))?;
        ranges.push(UnreadableRange {
            device,
            offset,
            length: u64::from(geometry.physical_block_size),
            form,
            label: format!("root{root:?}"),
        });
    }
    let broken = history.timeline[count - newest_roots_hidden];
    let records = records_of_publish(&history.pool, geometry, broken);
    if records.is_empty() {
        return Ok(None);
    }
    match replay_break {
        ReplayBreak::EveryRecord => {
            for (counter, offset, _) in &records {
                for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                    ranges.push(UnreadableRange {
                        device,
                        offset: *offset,
                        length: JOURNAL_RECORD_BYTES,
                        form,
                        label: format!("record(counter={counter},device={})", device.0),
                    });
                }
            }
        }
        ReplayBreak::FirstNamedUnit | ReplayBreak::FirstCopyOfTheFirstNamedUnit => {
            let Some(named) = records
                .iter()
                .find_map(|(_, _, record)| record.named.first())
            else {
                return Ok(None);
            };
            let copies = if replay_break == ReplayBreak::FirstNamedUnit {
                named.locations.len()
            } else {
                1
            };
            for location in named.locations.iter().take(copies) {
                ranges.push(UnreadableRange {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form,
                    label: format!(
                        "unit(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                });
            }
        }
    }
    Ok(Some(ranges))
}

/// 一条根引用的单元（装在 `pool` 上读）：它的账里没释放的分配记录（`recovery::allocation_records_under_root`），
/// 加根记录直接指着的树表、映射树根、分配记录树根与实例表链认得出的各片。交回 (盘, 起始槽, 槽数)。
fn placements_referenced_by_root_on(
    pool: &MemoryPool,
    root: &RootRecord,
) -> Result<BTreeSet<(u32, u64, u64)>, String> {
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let mut placements = BTreeSet::new();
    let records = allocation_records_under_root(&devices, root)
        .map_err(|error| format!("账读不出: {error:?}"))?;
    for record in records.iter().filter(|record| !record.is_released) {
        placements.insert((record.device.0, record.slot.0, u64::from(record.span_slots)));
    }
    let direct_pointers = [
        &root.tree_table,
        &root.mapping_root,
        &root.allocation_record_tree_root,
    ];
    let instance_table_pages =
        singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable(&devices, root);
    for pointer in direct_pointers
        .into_iter()
        .chain(instance_table_pages.iter())
    {
        for location in &pointer.locations {
            if location.slot.0 != 0 {
                placements.insert((location.device.0, location.slot.0, 1));
            }
        }
    }
    Ok(placements)
}

/// `placements` 里有几处在 `before` 与 `after` 两份镜像上字节不同（被覆盖）。
fn placements_overwritten_between(
    before: &MemoryPool,
    after: &MemoryPool,
    placements: &BTreeSet<(u32, u64, u64)>,
) -> u64 {
    placements
        .iter()
        .filter(|(device, slot, span)| {
            let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
            before.read(*device, slot * SLOT_BYTES_LOCAL, length)
                != after.read(*device, slot * SLOT_BYTES_LOCAL, length)
        })
        .count() as u64
}

/// 这份镜像上恢复会落到哪条根：`crates` 的 `choose_root` 与装置自己的择根（checker 解出的自证根里 (txg, 实例) 最大）逐条比，
/// 对不上走停机 S5。交回 (那条根, 它的根记录)。
fn landing_root_on(pool: &MemoryPool) -> Result<(TimelineRoot, RootRecord), String> {
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let system_configuration = singlefs_core::recovery::choose_system_configuration(&devices)
        .map_err(|error| format!("择不出系统配置: {error:?}"))?;
    let crates_choice = singlefs_core::recovery::choose_root(&devices, &system_configuration)
        .ok_or("crates 择不出根")?;
    let geometry = independent_geometry(pool)?;
    let device_choice = readable_roots_independent(pool, &geometry)
        .into_iter()
        .max_by_key(|(instance, txg)| (*txg, *instance))
        .ok_or("装置解不出任何自证根")?;
    if root_pair(&crates_choice) != device_choice {
        return Err(format!(
            "S5：crates 择根 {:?} 与装置自己择的 {device_choice:?} 对不上",
            root_pair(&crates_choice)
        ));
    }
    Ok((device_choice, crates_choice))
}

/// 这份镜像上被抛弃的根（读法写死表「被抛弃的根」）：根环里 (txg, 实例) 最大那条根指着的实例表里，有行 (i, T) 且根的实例 = i、
/// txg > T。实例表按 `crates` 的解析读出行，判定是装置自己的式子；逐条与 `root_is_abandoned_by_the_instance_table` 比，对不上走 S5。
fn abandoned_roots_on(pool: &MemoryPool) -> Result<BTreeSet<TimelineRoot>, String> {
    let (_, newest) = landing_root_on(pool)?;
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let table = instance_table_chain_of_root(&devices, &newest)
        .map_err(|error| format!("最新根的实例表读不出: {error:?}"))?
        .records;
    let system_configuration = singlefs_core::recovery::choose_system_configuration(&devices)
        .map_err(|error| format!("{error:?}"))?;
    let mut abandoned = BTreeSet::new();
    for root in singlefs_core::recovery::readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    ) {
        let by_the_device = table.rows.iter().any(|row| {
            row.instance.0 == root.instance.0 && root.checkpoint_txg.0 > row.selected_root_txg.0
        });
        let by_crates =
            singlefs_core::recovery::root_is_abandoned_by_the_instance_table(&root, &table);
        if by_the_device != by_crates {
            return Err(format!(
                "S5：根 {:?} 装置判被抛弃={by_the_device}、crates 判={by_crates}",
                root_pair(&root)
            ));
        }
        if by_the_device {
            abandoned.insert(root_pair(&root));
        }
    }
    Ok(abandoned)
}

/// 冷启动读回（`recovery::recover`，看 journal）的结局，比「最后一次确认的那一版」的内容：
/// last_confirmed / other_content / no_file / failed:<成员>。另交回读回落到的根。
fn read_back_against(
    pool: &MemoryPool,
    last_confirmed: &Option<Vec<u8>>,
) -> (String, Option<TimelineRoot>) {
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let report = recover(&devices, JournalPolicy::Consult);
    let root = chosen_root_of(&report.outcome);
    let label = match &report.outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            if Some(content.clone()) == *last_confirmed {
                "last_confirmed".to_string()
            } else {
                "other_content".to_string()
            }
        }
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::Failed { failure, .. } => {
            format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
        }
    };
    (label, root)
}

/// 新实例第一次发布的 txg 按今天的规则该是几（跑前登记 7.1 第六行）：装置自己解出的、这次调用里读得出的根（藏起来的根槽不算）
/// 与自证记录（两份都藏起来的不算）的 txg 取 max 再 + 1。
fn first_txg_by_the_rule_of_today(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    unreadable: &[UnreadableRange],
) -> u64 {
    let slots = checker_image::root_slot_positions(geometry);
    let highest_root = checker_image::valid_roots(pool, geometry)
        .into_iter()
        .filter(|(region, slot, _)| {
            slots
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    candidate_region == region && candidate_slot == slot
                })
                .is_some_and(|(_, _, device, offset)| {
                    !unreadable.iter().any(|range| {
                        range.overlaps(
                            DeviceIdentity(*device),
                            *offset,
                            u64::from(geometry.physical_block_size),
                        )
                    })
                })
        })
        .map(|(_, _, view)| view.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let mut highest_record = 0u64;
    for slot_index in 0..ring_slots {
        let offset = record_offset(slot_index + 1, geometry.journal_ring_bytes);
        for device_number in [0u32, 1u32] {
            if unreadable.iter().any(|range| {
                range.overlaps(
                    DeviceIdentity(device_number),
                    offset.0,
                    JOURNAL_RECORD_BYTES,
                )
            }) {
                continue;
            }
            if let Some(record) = pool
                .read(device_number, offset.0, record_bytes)
                .and_then(|bytes| JournalRecord::parse(&bytes, expected_filesystem_identifier))
            {
                highest_record = highest_record.max(record.checkpoint_txg.0);
            }
        }
    }
    highest_root.max(highest_record) + 1
}

/// 系统配置里读得出的 F 的最大值（乙F-留环的水位那一半，与 `effective_rollback_floor` 的系统配置那一半同一取法）。
fn highest_floor_in_the_system_configurations(pool: &MemoryPool, slot_spacing: u64) -> u64 {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    [0u32, 1]
        .into_iter()
        .flat_map(|device| [(device, 0u64), (device, slot_spacing)])
        .filter_map(|(device, offset)| pool.read(device, offset, slot_bytes))
        .filter_map(|bytes| {
            singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes).ok()
        })
        .map(|parsed| parsed.quantities.rollback_floor.0)
        .max()
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// 10.4 H1d：崩溃恢复把最新的根当成读不出、抛弃它、取号，这次可写挂载在写行根落盘之前断；故障撤掉、再挂载。
//      每一格报三样：① 恢复落到的根引用的单元有没有被覆盖；② 再挂载那一次 K0–K4；③ 读回的是不是最后一次确认的那一版。
// ---------------------------------------------------------------------------

#[derive(Default)]
struct HeldOnceFamilySummary {
    cases: u64,
    void_cases: u64,
    overwritten_cases: u64,
    newest_root_units_overwritten_cases: u64,
    remount_classes: BTreeMap<&'static str, u64>,
    read_back_after_remount: BTreeMap<String, u64>,
    read_back_before_remount: BTreeMap<String, u64>,
    trigger_positive_cases: u64,
    last_confirmed_and_not_overwritten: u64,
    /// 按断在哪一类（none / crash / write_fails / barrier_fails）分。
    by_cut: BTreeMap<&'static str, HeldOnceCutSummary>,
}

#[derive(Default)]
struct HeldOnceCutSummary {
    cases: u64,
    overwritten_cases: u64,
    remount_classes: BTreeMap<&'static str, u64>,
    read_back_after_remount: BTreeMap<String, u64>,
}

/// 一格 H1d：`post` 是那次挂载断掉之后盘上的样子（故障已撤）。`newest_root_units` 是被藏的那条（最新的）根在 `base` 上引用的单元。
/// ① 按「恢复落到的根」取：它在 `base` 上就是一条自证根时，比它引用的单元在 `base` 与 `post` 上的字节；它是这次挂载自己写出的根
/// （挂载做完了的那一格）时，它的单元就是这次写的，记「这次挂载里写的，没被覆盖」。被藏那条根的单元另报一栏。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的全部坐标原样打进结果行，拆成结构体只多一层搬运"
)]
fn evaluate_held_once_cell(
    arm: &str,
    coordinates: &str,
    cut: &'static str,
    base: &MemoryPool,
    post: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    last_confirmed: &Option<Vec<u8>>,
    newest_root_units: &BTreeSet<(u32, u64, u64)>,
    abandon_mount_outcome: &str,
    abandon_mount_trigger: Option<u64>,
    summary: &mut HeldOnceFamilySummary,
) {
    summary.cases += 1;
    let (landing, landing_record) = match landing_root_on(post) {
        Ok(found) => found,
        Err(error) => {
            emit_result(&format!(
                "name=r2_h1d_cell arm={arm} {coordinates} verdict=stop reason={error:?}"
            ));
            summary.void_cases += 1;
            return;
        }
    };
    let landing_in_base = independent_geometry(base)
        .map(|geometry| readable_roots_independent(base, &geometry).contains(&landing))
        .unwrap_or(false);
    let (overwritten, referenced_count) = if landing_in_base {
        match placements_referenced_by_root_on(base, &landing_record) {
            Ok(referenced) => (
                placements_overwritten_between(base, post, &referenced).to_string(),
                referenced.len().to_string(),
            ),
            Err(error) => (format!("unreadable_in_base:{error}"), "none".to_string()),
        }
    } else {
        ("written_in_this_mount".to_string(), "none".to_string())
    };
    let overwritten_now = overwritten.parse::<u64>().is_ok_and(|count| count > 0);
    if overwritten_now {
        summary.overwritten_cases += 1;
    }
    let newest_root_units_overwritten =
        placements_overwritten_between(base, post, newest_root_units);
    if newest_root_units_overwritten > 0 {
        summary.newest_root_units_overwritten_cases += 1;
    }
    let (read_back_before, read_back_before_root) = read_back_against(post, last_confirmed);
    let remount = run_instrumented_mount_writable(post, parameters, &[], None, None);
    let class = mount_outcome_class(&remount.result, remount.write_calls);
    let member = mount_error_member_or_none(&remount.result);
    let remount_trigger = remount
        .result
        .as_ref()
        .ok()
        .map(|mounted| mounted.output.abandoned_roots_unreadable);
    let remount_chosen = remount
        .result
        .as_ref()
        .ok()
        .map(|mounted| root_pair(&mounted.output.chosen_root));
    let (read_back_after, read_back_after_root) =
        read_back_against(&remount.pool_after, last_confirmed);
    if abandon_mount_trigger.is_some_and(|count| count > 0)
        || remount_trigger.is_some_and(|count| count > 0)
    {
        summary.trigger_positive_cases += 1;
    }
    if read_back_after == "last_confirmed" && !overwritten_now {
        summary.last_confirmed_and_not_overwritten += 1;
    }
    let by_cut = summary.by_cut.entry(cut).or_default();
    by_cut.cases += 1;
    if overwritten_now {
        by_cut.overwritten_cases += 1;
    }
    *by_cut.remount_classes.entry(class).or_insert(0) += 1;
    *by_cut
        .read_back_after_remount
        .entry(read_back_after.clone())
        .or_insert(0) += 1;
    *summary.remount_classes.entry(class).or_insert(0) += 1;
    *summary
        .read_back_after_remount
        .entry(read_back_after.clone())
        .or_insert(0) += 1;
    *summary
        .read_back_before_remount
        .entry(read_back_before.clone())
        .or_insert(0) += 1;
    emit_result(&format!(
        "name=r2_h1d_cell arm={arm} {coordinates} abandon_mount={abandon_mount_outcome} abandon_mount_abandoned_roots_unreadable={} landing_root={landing:?} landing_root_in_base={landing_in_base} referenced_placements={referenced_count} cell1_overwritten_placements={overwritten} cell1_overwritten={overwritten_now} newest_root_units={} newest_root_units_overwritten={newest_root_units_overwritten} cell2_remount_class={class} remount_error={member} remount_writes={} remount_chosen_root={} remount_abandoned_roots_unreadable={} read_back_before_remount={read_back_before} read_back_before_remount_root={} cell3_read_back_after_remount={read_back_after} read_back_after_remount_root={} cell3_is_last_confirmed={}",
        abandon_mount_trigger.map_or("none".to_string(), |count| count.to_string()),
        newest_root_units.len(),
        remount.write_calls,
        remount_chosen.map_or("none".to_string(), |root| format!("{root:?}")),
        remount_trigger.map_or("none".to_string(), |count| count.to_string()),
        read_back_before_root.map_or("none".to_string(), |root| format!("{root:?}")),
        read_back_after_root.map_or("none".to_string(), |root| format!("{root:?}")),
        read_back_after == "last_confirmed"
    ));
}

/// 故障有没有全拦到读（V2）：点了名而一次都没拦到的落点列出来。
fn unintercepted_ranges(ranges: &[UnreadableRange], intercepted: &[u64]) -> Vec<String> {
    ranges
        .iter()
        .zip(intercepted)
        .filter(|(_, count)| **count == 0)
        .map(|(range, _)| range.label.clone())
        .collect()
}

/// H1d 的 n1：首个文件之后几次覆盖写。取 0 起（第 2 次跑修订：跑前修订没给 H1d 的参数；n1 = 0 时最新那条根就是首个文件，
/// 它前一条是写行之后的暖机根、树表 0 条，站在那条根看首个文件那次发布开的段是全空的——实七那一形的前提；n1 ≥ 1 时最新那条根的单元
/// 与前一条根的单元在同一个开放段里，写行那次发布开的是更高的新段，碰不到它们）。
const HELD_ONCE_OVERWRITES: [u64; 4] = [0, 1, 2, 3];

fn run_held_once_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut summary = HeldOnceFamilySummary::default();
    for overwrites in HELD_ONCE_OVERWRITES {
        let mut history = match second_run_start(geometry, overwrites) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        if let Err(error) =
            second_run_close(&mut history, &parameters, Closing::ProcessExit, arm, "h1d")
        {
            emit_result(&format!(
                "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
            ));
            continue;
        }
        let base = history.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            emit_result(&format!(
                "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason=\"S5：几何解不出\""
            ));
            continue;
        };
        let root_slots = root_slot_device_offsets(&pool_geometry);
        let last_confirmed = history.tip_content();
        let newest_root_units = match landing_root_on(&base)
            .and_then(|(_, record)| placements_referenced_by_root_on(&base, &record))
        {
            Ok(units) => units,
            Err(error) => {
                emit_result(&format!(
                    "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstNamedUnit,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                let head = format!(
                    "n1={overwrites} tip={:?} form={} break={}",
                    history.tip(),
                    form.name(),
                    replay_break.name()
                );
                let ranges = match unreadable_ranges_of_the_abandonment(
                    &history,
                    &pool_geometry,
                    1,
                    form,
                    replay_break,
                ) {
                    Ok(Some(ranges)) => ranges,
                    Ok(None) => {
                        emit_result(&format!(
                            "name=r2_h1d_reference arm={arm} {head} verdict=not_applicable"
                        ));
                        continue;
                    }
                    Err(error) => {
                        emit_result(&format!(
                            "name=r2_h1d_reference arm={arm} {head} verdict=stop reason={error:?}"
                        ));
                        continue;
                    }
                };
                let reference =
                    run_instrumented_mount_writable(&base, &parameters, &ranges, None, None);
                let missed = unintercepted_ranges(&ranges, &reference.intercepted_reads);
                let first_root_write = index_of_the_first_root_slot_write(
                    &reference.applied,
                    &root_slots,
                    u64::from(pool_geometry.physical_block_size),
                );
                let reference_class = mount_outcome_class(&reference.result, reference.write_calls);
                let reference_first_txg = reference
                    .result
                    .as_ref()
                    .ok()
                    .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
                let rule_of_today = first_txg_by_the_rule_of_today(&base, &pool_geometry, &ranges);
                emit_result(&format!(
                    "name=r2_h1d_reference arm={arm} {head} faults={} unintercepted={missed:?} class={reference_class} error={} writes={} chosen_root={} row_publish_txg={} first_txg_by_the_rule_of_today={rule_of_today} abandoned_roots_unreadable={} writes_before_the_row_root={}",
                    ranges.len(),
                    mount_error_member_or_none(&reference.result),
                    reference.write_calls,
                    reference
                        .result
                        .as_ref()
                        .ok()
                        .map_or("none".to_string(), |mounted| format!("{:?}", root_pair(&mounted.output.chosen_root))),
                    reference_first_txg.map_or("none".to_string(), |txg| txg.to_string()),
                    reference
                        .result
                        .as_ref()
                        .ok()
                        .map_or("none".to_string(), |mounted| mounted.output.abandoned_roots_unreadable.to_string()),
                    first_root_write.map_or("none".to_string(), |index| index.to_string()),
                ));
                if !missed.is_empty() {
                    summary.void_cases += 1;
                    continue;
                }
                let reference_trigger = reference
                    .result
                    .as_ref()
                    .ok()
                    .map(|mounted| mounted.output.abandoned_roots_unreadable);
                evaluate_held_once_cell(
                    arm,
                    &format!("{head} cut=none point=none"),
                    "none",
                    &base,
                    &reference.pool_after,
                    &parameters,
                    &last_confirmed,
                    &newest_root_units,
                    reference_class,
                    reference_trigger,
                    &mut summary,
                );
                let Some(first_root_write) = first_root_write else {
                    continue;
                };
                for write_count in 0..=first_root_write {
                    let post = image_with_the_first_writes(&base, &reference.applied, write_count);
                    evaluate_held_once_cell(
                        arm,
                        &format!("{head} cut=crash point={write_count}"),
                        "crash",
                        &base,
                        &post,
                        &parameters,
                        &last_confirmed,
                        &newest_root_units,
                        "crashed",
                        None,
                        &mut summary,
                    );
                }
                for failing_write in 0..=first_root_write {
                    let failed = run_instrumented_mount_writable(
                        &base,
                        &parameters,
                        &ranges,
                        Some(u64::try_from(failing_write).expect("写序号")),
                        None,
                    );
                    let failed_class = mount_outcome_class(&failed.result, failed.write_calls);
                    let outcome = format!(
                        "{failed_class}:{}:fired={}",
                        mount_error_member_or_none(&failed.result),
                        failed.injected_failure_fired
                    );
                    let trigger = failed
                        .result
                        .as_ref()
                        .ok()
                        .map(|mounted| mounted.output.abandoned_roots_unreadable);
                    evaluate_held_once_cell(
                        arm,
                        &format!("{head} cut=write_fails point={failing_write}"),
                        "write_fails",
                        &base,
                        &failed.pool_after,
                        &parameters,
                        &last_confirmed,
                        &newest_root_units,
                        &outcome,
                        trigger,
                        &mut summary,
                    );
                }
                let barriers = barriers_before_write(&reference.applied, first_root_write + 1);
                for failing_barrier in 0..barriers {
                    let failed = run_instrumented_mount_writable(
                        &base,
                        &parameters,
                        &ranges,
                        None,
                        Some(failing_barrier),
                    );
                    let failed_class = mount_outcome_class(&failed.result, failed.write_calls);
                    let outcome = format!(
                        "{failed_class}:{}:fired={}",
                        mount_error_member_or_none(&failed.result),
                        failed.injected_failure_fired
                    );
                    let trigger = failed
                        .result
                        .as_ref()
                        .ok()
                        .map(|mounted| mounted.output.abandoned_roots_unreadable);
                    evaluate_held_once_cell(
                        arm,
                        &format!("{head} cut=barrier_fails point={failing_barrier}"),
                        "barrier_fails",
                        &base,
                        &failed.pool_after,
                        &parameters,
                        &last_confirmed,
                        &newest_root_units,
                        &outcome,
                        trigger,
                        &mut summary,
                    );
                }
            }
        }
    }
    emit_result(&format!(
        "name=r2_h1d_summary arm={arm} geometry={} cases={} void_cases={} cell1_overwritten_cases={} newest_root_units_overwritten_cases={} cell2_remount_classes={:?} cell3_read_back_after_remount={:?} read_back_before_remount={:?} trigger_positive_cases={} last_confirmed_and_not_overwritten={}",
        geometry.label,
        summary.cases,
        summary.void_cases,
        summary.overwritten_cases,
        summary.newest_root_units_overwritten_cases,
        summary.remount_classes,
        summary.read_back_after_remount,
        summary.read_back_before_remount,
        summary.trigger_positive_cases,
        summary.last_confirmed_and_not_overwritten
    ));
    // 岔路单第 1 行 (b)、(c) 两个指称：这两条臂不在 `crates/` 里实现（跑前登记第三节末句），它们与今天的差别只在
    // 「取号之前或别处的计数 > 0」的那几格（(c) 另在盘上记一笔，落点未定）；计数全 0 时它们的三格就是今天那一臂的三格。
    for (cut, by_cut) in &summary.by_cut {
        emit_result(&format!(
            "name=r2_h1d_summary_by_cut arm={arm} cut={cut} cases={} cell1_overwritten_cases={} cell2_remount_classes={:?} cell3_read_back_after_remount={:?}",
            by_cut.cases,
            by_cut.overwritten_cases,
            by_cut.remount_classes,
            by_cut.read_back_after_remount
        ));
    }
    if arm != "today" {
        return;
    }
    for (fork_one_arm, referent) in [
        ("(b)原形", "tree_table"),
        ("(b)原形", "allocation_record_tree"),
        ("(b)含链", "tree_table"),
        ("(b)含链", "allocation_record_tree"),
        ("(c)", "tree_table"),
        ("(c)", "allocation_record_tree"),
    ] {
        emit_result(&format!(
            "name=r2_h1d_fork_one_derived arm={arm} fork_one_arm={fork_one_arm} referent={referent} cases_where_it_differs_from_this_copy={} cells=same_as_this_copy_except_those",
            summary.trigger_positive_cases
        ));
    }
}

// ---------------------------------------------------------------------------
// 10.5 Q1-0：H1c 的抛弃步 A(k, b) 在今天的代码上造出几条被抛弃的根（前提 1 在装置上的读数）。故障一律读错（读法写死表「故障」）。
// ---------------------------------------------------------------------------

fn run_abandonment_step_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut attempted = 0u64;
    let mut not_applicable = 0u64;
    let mut void_attempts = 0u64;
    let mut mounted_count = 0u64;
    let mut producing = 0u64;
    let mut abandoned_total = 0u64;
    let mut fewest_faults: Option<usize> = None;
    let mut first_txg_rule_mismatches = 0u64;
    let mut below_floor_rows = 0u64;
    for overwrites in [1u64, 2, 3] {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let mut history = match second_run_start(geometry, overwrites) {
                Ok(history) => history,
                Err(error) => {
                    emit_result(&format!(
                        "name=r2_q1_0_history arm={arm} n1={overwrites} c1={} verdict=stop reason={error:?}",
                        closing.name()
                    ));
                    continue;
                }
            };
            if let Err(error) = second_run_close(
                &mut history,
                &parameters,
                closing,
                arm,
                &format!("q1-0 n1={overwrites}"),
            ) {
                emit_result(&format!(
                    "name=r2_q1_0_history arm={arm} n1={overwrites} c1={} verdict=stop reason={error:?}",
                    closing.name()
                ));
                continue;
            }
            let Ok(pool_geometry) = independent_geometry(&history.pool) else {
                continue;
            };
            let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
            let floor_before =
                highest_floor_in_the_system_configurations(&history.pool, slot_spacing);
            let ring_slot_count = 3 * geometry.slots_per_region;
            let timeline_length = history.timeline.len();
            emit_result(&format!(
                "name=r2_q1_0_history arm={arm} n1={overwrites} c1={} timeline={:?} timeline_roots={timeline_length} root_ring_slots={ring_slot_count} highest_floor_in_the_system_configurations={floor_before}",
                closing.name(),
                history.timeline
            ));
            for hidden in 1..timeline_length {
                for replay_break in [
                    ReplayBreak::EveryRecord,
                    ReplayBreak::FirstNamedUnit,
                    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
                ] {
                    attempted += 1;
                    let head = format!(
                        "n1={overwrites} c1={} k={hidden} b={}",
                        closing.name(),
                        replay_break.name()
                    );
                    let ranges = match unreadable_ranges_of_the_abandonment(
                        &history,
                        &pool_geometry,
                        hidden,
                        UnreadableForm::ReadFails,
                        replay_break,
                    ) {
                        Ok(Some(ranges)) => ranges,
                        Ok(None) => {
                            not_applicable += 1;
                            emit_result(&format!(
                                "name=r2_q1_0 arm={arm} {head} verdict=not_applicable"
                            ));
                            continue;
                        }
                        Err(error) => {
                            emit_result(&format!(
                                "name=r2_q1_0 arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let run = run_instrumented_mount_writable(
                        &history.pool,
                        &parameters,
                        &ranges,
                        None,
                        None,
                    );
                    let missed = unintercepted_ranges(&ranges, &run.intercepted_reads);
                    let class = mount_outcome_class(&run.result, run.write_calls);
                    if !missed.is_empty() {
                        void_attempts += 1;
                    }
                    let rule_of_today =
                        first_txg_by_the_rule_of_today(&history.pool, &pool_geometry, &ranges);
                    let (chosen, row_txg, abandoned_text, abandoned_count) = match &run.result {
                        Ok(mounted) => {
                            mounted_count += 1;
                            let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
                            if row_txg != rule_of_today {
                                first_txg_rule_mismatches += 1;
                            }
                            if row_txg <= floor_before {
                                below_floor_rows += 1;
                            }
                            match abandoned_roots_on(&run.pool_after) {
                                Ok(abandoned) => {
                                    let count = abandoned.len() as u64;
                                    if count > 0 && missed.is_empty() {
                                        producing += 1;
                                        abandoned_total += count;
                                        fewest_faults =
                                            Some(fewest_faults.map_or(ranges.len(), |fewest| {
                                                fewest.min(ranges.len())
                                            }));
                                    }
                                    (
                                        format!("{:?}", root_pair(&mounted.output.chosen_root)),
                                        row_txg.to_string(),
                                        format!("{abandoned:?}"),
                                        count,
                                    )
                                }
                                Err(error) => (
                                    format!("{:?}", root_pair(&mounted.output.chosen_root)),
                                    row_txg.to_string(),
                                    format!("stop:{error}"),
                                    0,
                                ),
                            }
                        }
                        Err(_) => (
                            "none".to_string(),
                            "none".to_string(),
                            "none".to_string(),
                            0,
                        ),
                    };
                    emit_result(&format!(
                        "name=r2_q1_0 arm={arm} {head} faults={} unintercepted={missed:?} class={class} error={} writes={} chosen_root={chosen} expected_chosen_root={:?} row_publish_txg={row_txg} first_txg_by_the_rule_of_today={rule_of_today} highest_floor_in_the_system_configurations={floor_before} abandoned_count={abandoned_count} abandoned={abandoned_text:?}",
                        ranges.len(),
                        mount_error_member_or_none(&run.result),
                        run.write_calls,
                        history.timeline[timeline_length - 1 - hidden],
                    ));
                }
            }
        }
    }
    emit_result(&format!(
        "name=r2_q1_0_summary arm={arm} geometry={} attempted={attempted} not_applicable={not_applicable} void_attempts={void_attempts} mounted={mounted_count} producing_at_least_one_abandoned_root={producing} abandoned_roots_total={abandoned_total} fewest_faults_that_produced_one={} first_txg_rule_of_today_mismatches={first_txg_rule_mismatches} row_publish_txg_at_or_below_the_floor={below_floor_rows}",
        geometry.label,
        fewest_faults.map_or("none".to_string(), |fewest| fewest.to_string())
    ));
}

// ---------------------------------------------------------------------------
// 10.6 实七报告「交主 agent 的」第 1 条点名的两段历史（H1d 的取样要罩住它们，跑前修订原话）：照 `crates/singlefs-harness`
//      自己的注入器原样重跑，报每个新发现的签名与重开走到哪、有没有「映射落点读不出」。
// ---------------------------------------------------------------------------

fn run_seventh_batch_reproductions(arm: &str) {
    use singlefs_harness::history::{
        generate_history_with_weights, GenerationWeights, HistoryDeviceWidth, HistoryExecution,
        HistorySeed, PerStepChecker,
    };
    let fault_seed =
        SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET;
    let fault_history = generate_history_with_weights(
        HistorySeed(fault_seed),
        SEVENTH_BATCH_FAULT_REPRODUCTION_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    let fault_injection = singlefs_harness::fault_injection::inject_faults_into_history(
        &fault_history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
        },
        SEVENTH_BATCH_FAULT_REPRODUCTION_FAULTS,
    );
    for drawn in &fault_injection.drawn_faults {
        emit_result(&format!(
            "name=r2_seventh_batch_fault_drawn arm={arm} seed={fault_seed} drawn={:?}",
            drawn.render()
        ));
    }
    let mut mapping_unreadable = 0u64;
    for finding in &fault_injection.new_findings {
        let violations: Vec<&str> = finding
            .observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect();
        let names_mapping = finding.reopen.contains("MappingStillUnreadable");
        if names_mapping {
            mapping_unreadable += 1;
        }
        emit_result(&format!(
            "name=r2_seventh_batch_fault_finding arm={arm} seed={fault_seed} drawn={:?} signature={:?} reopen={:?} violations={violations:?} reopen_names_mapping_still_unreadable={names_mapping}",
            finding.drawn.render(),
            finding.signature,
            finding.reopen
        ));
    }
    emit_result(&format!(
        "name=r2_seventh_batch_fault_summary arm={arm} seed={fault_seed} drawn={} new_findings={} findings_reopening_to_mapping_still_unreadable={mapping_unreadable} known_red_hits={}",
        fault_injection.drawn_faults.len(),
        fault_injection.new_findings.len(),
        fault_injection.known_red_hits.len()
    ));
    let crash_seed =
        SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET;
    let crash_history = generate_history_with_weights(
        HistorySeed(crash_seed),
        SEVENTH_BATCH_CRASH_REPRODUCTION_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    let crash_injection = singlefs_checker_tier::crash_injection::inject_crashes_into_history(
        &crash_history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
        },
        singlefs_checker_tier::crash_injection::CrashPointDraw::Sampled {
            crash_points_per_history: SEVENTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS,
        },
        None,
    );
    for crash_point in &crash_injection.crash_points {
        emit_result(&format!(
            "name=r2_seventh_batch_crash_point arm={arm} seed={crash_seed} point={:?} step={:?} operation={:?}",
            crash_point.render(),
            crash_point.step,
            crash_point.operation_kind
        ));
    }
    let mut crash_mapping_unreadable = 0u64;
    for finding in &crash_injection.new_findings {
        let violations: Vec<&str> = finding
            .observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect();
        let names_mapping = finding.read_back.contains("MappingStillUnreadable");
        if names_mapping {
            crash_mapping_unreadable += 1;
        }
        emit_result(&format!(
            "name=r2_seventh_batch_crash_finding arm={arm} seed={crash_seed} point={:?} operation={:?} signature={:?} read_back={:?} violations={violations:?} read_back_names_mapping_still_unreadable={names_mapping}",
            finding.crash_point.render(),
            finding.crash_point.operation_kind,
            finding.signature,
            finding.read_back
        ));
    }
    emit_result(&format!(
        "name=r2_seventh_batch_crash_summary arm={arm} seed={crash_seed} crash_points={} new_findings={} findings_reading_back_mapping_still_unreadable={crash_mapping_unreadable} known_red_hits={}",
        crash_injection.crash_points.len(),
        crash_injection.new_findings.len(),
        crash_injection.known_red_hits.len()
    ));
}

// ---------------------------------------------------------------------------
// 10.7 PC1-a / PC1-b / PC1-c（跑前登记 5.2 阳性对照；第一段只跑今天那一臂与 (c)、A1 副本上 PC1-a 的 (a) 那一句，
//      (b) 两个指称的那几句归第二段）。被抛弃根取自 H1c 的抛弃步（固定次序 n1、c1、k、b，m = 0、c2 = C），op1 = O1、瞬时。
// ---------------------------------------------------------------------------

/// H1c 抛弃步造出的一个节点：抛弃步交 `Ok`、点了名的落点全拦到（V2）、按新实例表判出至少一条被抛弃的根。
struct AbandonmentStepNode {
    label: String,
    pool: MemoryPool,
    abandoned: BTreeSet<TimelineRoot>,
}

/// H1c 抛弃步造出的节点，按固定次序（n1、c1、k、b）；故障一律读错。
fn abandonment_step_node_list(geometry: &Geometry) -> Vec<AbandonmentStepNode> {
    let parameters = parameters_for(geometry);
    let mut nodes = Vec::new();
    for overwrites in [1u64, 2, 3] {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let Ok(mut history) = second_run_start(geometry, overwrites) else {
                continue;
            };
            if second_run_close(&mut history, &parameters, closing, "node-list", "node-list")
                .is_err()
            {
                continue;
            }
            let Ok(pool_geometry) = independent_geometry(&history.pool) else {
                continue;
            };
            for hidden in 1..history.timeline.len() {
                for replay_break in [
                    ReplayBreak::EveryRecord,
                    ReplayBreak::FirstNamedUnit,
                    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
                ] {
                    let Ok(Some(ranges)) = unreadable_ranges_of_the_abandonment(
                        &history,
                        &pool_geometry,
                        hidden,
                        UnreadableForm::ReadFails,
                        replay_break,
                    ) else {
                        continue;
                    };
                    let run = run_instrumented_mount_writable(
                        &history.pool,
                        &parameters,
                        &ranges,
                        None,
                        None,
                    );
                    if run.result.is_err()
                        || !unintercepted_ranges(&ranges, &run.intercepted_reads).is_empty()
                    {
                        continue;
                    }
                    let Ok(abandoned) = abandoned_roots_on(&run.pool_after) else {
                        continue;
                    };
                    if abandoned.is_empty() {
                        continue;
                    }
                    nodes.push(AbandonmentStepNode {
                        label: format!(
                            "n1={overwrites} c1={} k={hidden} b={}",
                            closing.name(),
                            replay_break.name()
                        ),
                        pool: run.pool_after,
                        abandoned,
                    });
                }
            }
        }
    }
    nodes
}

fn ranges_of_both_copies(locations: &[LocationEntry; 2], label: &str) -> Vec<UnreadableRange> {
    locations
        .iter()
        .map(|location| UnreadableRange {
            device: location.device,
            offset: location.slot.0 * SLOT_BYTES_LOCAL,
            length: SLOT_BYTES_LOCAL,
            form: UnreadableForm::ReadFails,
            label: format!(
                "{label}(device={},slot={})",
                location.device.0, location.slot.0
            ),
        })
        .collect()
}

/// 一条根某个单元的位置键（(盘, 槽)）；取不到时 None。
type UnitKeysOfARoot<'reader> = dyn Fn(&RootRecord) -> Option<BTreeSet<(u32, u64)>> + 'reader;

fn location_keys(locations: &[LocationEntry; 2]) -> BTreeSet<(u32, u64)> {
    locations.iter().map(location_key).collect()
}

/// 一次 op1（O1、瞬时）的结局行，外加这一臂按跑前登记 5.2 该出的结局判没判对：今天那一臂与 (c) 要 `Ok` 且计数 = 装置数的；
/// A1 副本（PC1-a、PC1-c 两句）按那一句写的；别的臂没有登记结局，只报。
#[allow(clippy::too_many_arguments, reason = "一行结果的全部栏目原样打出")]
fn emit_positive_control(
    arm: &str,
    control: &str,
    node_label: &str,
    abandoned_root: TimelineRoot,
    ranges: &[UnreadableRange],
    run: &InstrumentedMount,
    expected_count: u64,
    extra: &str,
) -> bool {
    let missed = unintercepted_ranges(ranges, &run.intercepted_reads);
    let class = mount_outcome_class(&run.result, run.write_calls);
    let member = mount_error_member_or_none(&run.result);
    let count = run
        .result
        .as_ref()
        .ok()
        .map(|mounted| mounted.output.abandoned_roots_unreadable);
    let isolated: Option<u64> = run.result.as_ref().ok().map(|mounted| {
        mounted
            .output
            .isolated_slots_per_device
            .iter()
            .map(|(_, slots)| *slots)
            .sum()
    });
    let (expectation, passed) = match (arm, control) {
        ("A1", "PC1-a") => (
            "AbandonedRootLedgerUnreadable_and_zero_writes",
            member == "AbandonedRootLedgerUnreadable" && run.write_calls == 0,
        ),
        ("A1", "PC1-c") => ("ok", run.result.is_ok()),
        ("today", "PC1-c") => (
            "ok_count_zero_isolated_zero",
            count == Some(0) && isolated == Some(0),
        ),
        ("today", _) => (
            "ok_count_equals_the_device_count",
            count == Some(expected_count),
        ),
        _ => ("not_registered_for_this_arm", true),
    };
    let passed = passed && missed.is_empty();
    emit_result(&format!(
        "name=r2_positive_control arm={arm} control={control} node={node_label:?} abandoned_root={abandoned_root:?} faults={} unintercepted={missed:?} class={class} error={member} writes={} abandoned_roots_unreadable={} expected_count_by_the_device={expected_count} isolated_slots={} {extra} expectation={expectation} verdict={}",
        ranges.len(),
        run.write_calls,
        count.map_or("none".to_string(), |value| value.to_string()),
        isolated.map_or("none".to_string(), |value| value.to_string()),
        if passed { "pass" } else { "fail" }
    ));
    passed
}

fn run_ledger_positive_controls(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let nodes = abandonment_step_node_list(geometry);
    emit_result(&format!(
        "name=r2_positive_control_nodes arm={arm} nodes={}",
        nodes.len()
    ));
    let mut ledger_unit_controls_found = false;
    'search: for node in &nodes {
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
        for &abandoned_root in node.abandoned.iter().filter(|root| readable.contains(root)) {
            let Some(record) = root_record_of(&node.pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let Ok(allocation_locations) = locate_allocation_record_tree(&plain, &record) else {
                continue;
            };
            let allocation_keys = location_keys(&allocation_locations);
            if is_unit_shared(
                &plain,
                &node.pool,
                &pool_geometry,
                abandoned_root,
                &allocation_keys,
            ) {
                continue;
            }
            ledger_unit_controls_found = true;
            let through = |keys_of: &UnitKeysOfARoot, keys: &BTreeSet<(u32, u64)>| {
                node.abandoned
                    .iter()
                    .filter(|root| readable.contains(root))
                    .filter_map(|root| root_record_of(&node.pool, &pool_geometry, *root))
                    .filter(|other| {
                        keys_of(other).is_some_and(|other_keys| !other_keys.is_disjoint(keys))
                    })
                    .count() as u64
            };
            let allocation_keys_of = |other: &RootRecord| {
                locate_allocation_record_tree(&plain, other)
                    .ok()
                    .map(|locations| location_keys(&locations))
            };
            let tree_table_keys_of =
                |other: &RootRecord| Some(location_keys(&other.tree_table.locations));
            let ranges = ranges_of_both_copies(&allocation_locations, "L");
            let run = run_instrumented_mount_writable(&node.pool, &parameters, &ranges, None, None);
            emit_positive_control(
                arm,
                "PC1-a",
                &node.label,
                abandoned_root,
                &ranges,
                &run,
                through(&allocation_keys_of, &allocation_keys),
                "unit=allocation_record_tree_root shared=false",
            );
            let tree_table_keys = location_keys(&record.tree_table.locations);
            let tree_table_ranges = ranges_of_both_copies(&record.tree_table.locations, "T");
            let tree_table_run = run_instrumented_mount_writable(
                &node.pool,
                &parameters,
                &tree_table_ranges,
                None,
                None,
            );
            emit_positive_control(
                arm,
                "PC1-b",
                &node.label,
                abandoned_root,
                &tree_table_ranges,
                &tree_table_run,
                through(&tree_table_keys_of, &tree_table_keys),
                &format!(
                    "unit=tree_table shared={}",
                    is_unit_shared(
                        &plain,
                        &node.pool,
                        &pool_geometry,
                        abandoned_root,
                        &tree_table_keys
                    )
                ),
            );
            break 'search;
        }
    }
    if !ledger_unit_controls_found {
        emit_result(&format!(
            "name=r2_positive_control arm={arm} control=PC1-a verdict=not_constructible reason=\"H1c 里找不到被抛弃根的分配记录树节点不共享的历史（V1）\""
        ));
    }
    let mut root_slot_control_found = false;
    for node in &nodes {
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let abandoned_readable: Vec<TimelineRoot> = node
            .abandoned
            .iter()
            .copied()
            .filter(|root| readable.contains(root))
            .collect();
        let [abandoned_root] = abandoned_readable.as_slice() else {
            continue;
        };
        let Some(record) = root_record_of(&node.pool, &pool_geometry, *abandoned_root) else {
            continue;
        };
        let Ok(mut exclusive) = placements_referenced_by_root_on(&node.pool, &record) else {
            continue;
        };
        for other in readable.iter().filter(|root| *root != abandoned_root) {
            if let Some(other_record) = root_record_of(&node.pool, &pool_geometry, *other) {
                if let Ok(referenced) = placements_referenced_by_root_on(&node.pool, &other_record)
                {
                    exclusive.retain(|(device, slot, _)| {
                        !referenced.iter().any(|(other_device, other_slot, _)| {
                            other_device == device && other_slot == slot
                        })
                    });
                }
            }
        }
        if exclusive.is_empty() {
            continue;
        }
        let Some((device, offset)) = root_slot_of(&node.pool, &pool_geometry, *abandoned_root)
        else {
            continue;
        };
        root_slot_control_found = true;
        let twin = run_instrumented_mount_writable(&node.pool, &parameters, &[], None, None);
        let twin_isolated: Option<u64> = twin.result.as_ref().ok().map(|mounted| {
            mounted
                .output
                .isolated_slots_per_device
                .iter()
                .map(|(_, slots)| *slots)
                .sum()
        });
        let ranges = vec![UnreadableRange {
            device,
            offset,
            length: u64::from(pool_geometry.physical_block_size),
            form: UnreadableForm::ReadFails,
            label: format!("R{abandoned_root:?}"),
        }];
        let run = run_instrumented_mount_writable(&node.pool, &parameters, &ranges, None, None);
        emit_positive_control(
            arm,
            "PC1-c",
            &node.label,
            *abandoned_root,
            &ranges,
            &run,
            0,
            &format!(
                "exclusive_placements={} isolated_slots_without_the_fault={}",
                exclusive.len(),
                twin_isolated.map_or("none".to_string(), |value| value.to_string())
            ),
        );
        break;
    }
    if !root_slot_control_found {
        emit_result(&format!(
            "name=r2_positive_control arm={arm} control=PC1-c verdict=not_constructible reason=\"H1c 里找不到只有一条被抛弃根且它独占的槽 > 0 的历史（V1）\""
        ));
    }
}

/// 第 2 次跑的模式：开跑检查不过就停（V3 / V4 / S3），过了跑点名的那一族。交回 true = 这个模式归第 2 次跑。
fn run_second_run_mode(mode: &str) -> bool {
    let known = [
        "r2-all",
        "r2-constants",
        "r2-h1d",
        "r2-q1-0",
        "r2-pc1",
        "r2-seventh-batch",
    ];
    if !known.contains(&mode) {
        return false;
    }
    let arm = second_run_arm_label();
    if !run_second_run_constants_and_anchors(&arm) {
        emit_result(&format!(
            "name=stop arm={arm} reason=V3 detail=\"第 2 次跑的常量回比、第七节锚点或偏移解码有不过的，整轮不开跑\""
        ));
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    match mode {
        "r2-all" => {
            run_held_once_family(&GEOMETRY_PRIMARY, &arm);
            run_abandonment_step_family(&GEOMETRY_PRIMARY, &arm);
            run_ledger_positive_controls(&GEOMETRY_PRIMARY, &arm);
            run_seventh_batch_reproductions(&arm);
        }
        "r2-constants" => {}
        "r2-h1d" => run_held_once_family(&GEOMETRY_PRIMARY, &arm),
        "r2-q1-0" => run_abandonment_step_family(&GEOMETRY_PRIMARY, &arm),
        "r2-pc1" => run_ledger_positive_controls(&GEOMETRY_PRIMARY, &arm),
        "r2-seventh-batch" => run_seventh_batch_reproductions(&arm),
        other => unreachable!("第 2 次跑的模式表 known 已经挡过：{other}"),
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    true
}

// ============================================================================
// 十一、第 3 次跑（`research/prompts/e158-r3-prereg.md`）：C554 怎么修（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行），
//      第一段（登记 5.7）：5.6 常量回比、第七节锚点、九条臂各一份副本、故障设备与虚拟时钟、确认账与真值、
//      PC-N、PC-O、PC-N0、PC-丢写、PC-只读（每一臂）；H1d 与 H1e（G0，全部臂）。PC-多读、Q5 与跨臂对拍在 `r3-compare` 模式里算。
//      第 2 次跑那几个模式（`r2-*`）调的函数这里只调用、不改语义；要改的一律复制一份（登记第一节「上一次执行员交来的两件」）。
// ============================================================================

/// 第 3 次跑的九条臂（登记 5.2 臂表的文件名拼写；丙-不验点名 主 agent 2026-09-27 划掉，不跑）。
const THIRD_RUN_ARMS: [&str; 9] = [
    "today",
    "jia-cfg",
    "jia-cfg-carry",
    "jia-slot",
    "yi-cfg",
    "yi-cfg-carry",
    "yi-slot",
    "bing-cfg",
    "bing-cfg-carry",
];

/// 臂属于哪个候选（岔路单第 1 行：甲 拒可写、乙 重读后再判、丙 从记录重建、对照 今天）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThirdRunCandidate {
    Today,
    Jia,
    Yi,
    Bing,
}

impl ThirdRunCandidate {
    fn of(arm: &str) -> Option<ThirdRunCandidate> {
        match arm {
            "today" => Some(ThirdRunCandidate::Today),
            "jia-cfg" | "jia-cfg-carry" | "jia-slot" => Some(ThirdRunCandidate::Jia),
            "yi-cfg" | "yi-cfg-carry" | "yi-slot" => Some(ThirdRunCandidate::Yi),
            "bing-cfg" | "bing-cfg-carry" => Some(ThirdRunCandidate::Bing),
            _ => None,
        }
    }
}

/// 这一臂 N 判真之后交回的拒绝成员名（臂表 r3 行加进 `MountError` 的四个成员里的哪一个；第 22 条那一格另一个）。
fn third_run_refusal_member(arm: &str) -> &'static str {
    match (ThirdRunCandidate::of(arm), arm.ends_with("-slot")) {
        (Some(ThirdRunCandidate::Bing), _) => "R3RebuildFromRecordsStoppedBelowTheWitness",
        (Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Yi), true) => "R3UnreadableRootRingSlot",
        (Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Yi), false) => {
            "R3NewerRootWitnessedByTheSystemConfiguration"
        }
        (Some(ThirdRunCandidate::Today) | None, _) => "none",
    }
}

/// 乙 的重读轮数 R（登记「读法写死」：D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」；5.6 倒数第二行，条款锚不回比）。
const THIRD_RUN_REREAD_ROUNDS: u64 = 1;
/// 虚拟时钟的一格 δ（登记 5.6 倒数第二行）。
const THIRD_RUN_VIRTUAL_INTERVAL: u64 = 1;
/// 单元类标签（D18（块里携带什么信息）；装置自己的重放按它取单元宽，开跑时与 `singlefs_core::unit` 那一份回比）。
const UNIT_CLASS_DATA_LOCAL: u8 = 1;
const UNIT_CLASS_INDEX_NODE_LOCAL: u8 = 2;
const UNIT_CLASS_PACKED_LOCAL: u8 = 3;
/// 数据单元宽、节点宽（登记 5.6 第三行；`local_constants_checks` 里已回比）。
const DATA_UNIT_BYTES_LOCAL: usize = 32768;
const NODE_BYTES_LOCAL: usize = 16384;
/// 系统配置槽里 journal tail 与整槽校验和的位置（登记 5.6：tail 偏移 469 由装置自己解码找到、开跑时核；
/// 校验和偏移 155、宽 32 字节：`local_constants_checks` 里回比过偏移，宽度出自 `singlefs_checker` 的宽校验和口径）。
const SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL: usize = 469;
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL: usize = 155;
const WIDE_CHECKSUM_BYTES_LOCAL: usize = 32;
/// H1d / H1e 的 n1（登记 5.3：n1 ∈ {0, 1, 2, 3}）与 H1e 的 m（{1, 2, 3, 4}）。
const THIRD_RUN_OVERWRITES: [u64; 4] = [0, 1, 2, 3];
const THIRD_RUN_TAIL_OVERWRITES: [u64; 4] = [1, 2, 3, 4];

fn third_run_arm_label() -> String {
    env::var("SINGLEFS_E158_ARM").unwrap_or_else(|_| "today".to_string())
}

/// 臂在 `crates/` 副本里记下的观测（`singlefs_core::mount::r3_take_observation` 的字段抄一份；今天那一臂没有）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ArmObservation {
    witness_judged: bool,
    newer_root_on_the_first_pass: bool,
    witnessed_tail: u64,
    selected_version_last_counter: Option<u64>,
    undecidable: bool,
    unreadable_ring_slots_on_the_first_pass: u64,
    reread_rounds: u32,
    rebuilt_from_records: bool,
    shadow_ledger_table_unreadable_on_the_first_read: bool,
    shadow_ledger_table_rereads: u32,
}

/// 等待钩子与虚拟时钟（登记「读法写死」表「时长」一行）：时钟在每次入口调用开始时归 0，只在臂调「等一个间隔」的钩子时前进 δ。
/// 臂的代码只在草稿副本里有（臂表 r3 行），装置源码各臂同一份：副本的 singlefs-harness 声明特性 `e158-r3-arm` 且缺省开，
/// 主工作区与今天那一臂的副本不声明，这几个函数退成空。
#[allow(
    unexpected_cfgs,
    reason = "特性 e158-r3-arm 只在草稿副本的 singlefs-harness/Cargo.toml 里声明（research/mutations/e158_arms.tsv 收尾那一行）"
)]
mod third_run_arm_hooks {
    use std::cell::Cell;

    thread_local! {
        static VIRTUAL_CLOCK: Cell<u64> = const { Cell::new(0) };
        static WAITS: Cell<u64> = const { Cell::new(0) };
    }

    pub fn reset_the_virtual_clock() {
        VIRTUAL_CLOCK.with(|clock| clock.set(0));
        WAITS.with(|waits| waits.set(0));
    }

    pub fn virtual_now() -> u64 {
        VIRTUAL_CLOCK.with(Cell::get)
    }

    pub fn waits() -> u64 {
        WAITS.with(Cell::get)
    }

    #[allow(
        dead_code,
        reason = "只在臂的副本（特性 e158-r3-arm 开）里被等待钩子调；主工作区与今天那一臂里只有单测调它"
    )]
    pub fn advance_one_interval(interval: u64) {
        VIRTUAL_CLOCK.with(|clock| clock.set(clock.get() + interval));
        WAITS.with(|waits| waits.set(waits.get() + 1));
    }

    pub fn arm_code_is_compiled_in() -> bool {
        cfg!(feature = "e158-r3-arm")
    }

    #[cfg(feature = "e158-r3-arm")]
    pub fn install_the_wait_hook(interval: u64) {
        singlefs_core::mount::r3_install_wait_hook(Some(Box::new(move || {
            advance_one_interval(interval);
        })));
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn install_the_wait_hook(_interval: u64) {}

    #[cfg(feature = "e158-r3-arm")]
    pub fn take_the_arm_observation() -> Option<super::ArmObservation> {
        let observation = singlefs_core::mount::r3_take_observation();
        Some(super::ArmObservation {
            witness_judged: observation.witness_judged,
            newer_root_on_the_first_pass: observation.newer_root_on_the_first_pass,
            witnessed_tail: observation.witnessed_tail,
            selected_version_last_counter: observation.selected_version_last_counter,
            undecidable: observation.undecidable,
            unreadable_ring_slots_on_the_first_pass: observation
                .unreadable_ring_slots_on_the_first_pass,
            reread_rounds: observation.reread_rounds,
            rebuilt_from_records: observation.rebuilt_from_records,
            shadow_ledger_table_unreadable_on_the_first_read: observation
                .shadow_ledger_table_unreadable_on_the_first_read,
            shadow_ledger_table_rereads: observation.shadow_ledger_table_rereads,
        })
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn take_the_arm_observation() -> Option<super::ArmObservation> {
        None
    }
}

// ---------------------------------------------------------------------------
// 11.1 故障设备（登记 5.0 第三条）：一块盘一层、一次装完这块盘这次调用的全部落点；两种造法 × 四种时长；
//      每个落点在这次调用里的读次序单独计（O(m) 按它判）；T(τ) 读虚拟时钟；被拦下的读次数按落点记（V2）；
//      设备一层的读调用次数与读字节数按调用记（Q5，读坏的也数）；这次调用写过的落点之后不再拦。
// ---------------------------------------------------------------------------

/// 一个落点在这次入口调用里坏多久（登记「读法写死」表「时长」一行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultDuration {
    /// W：这一次入口调用里每一次读都坏。
    WholeCall,
    /// T(τ)：读坏到虚拟时钟走到 τ 为止。
    UntilVirtualTime(u64),
    /// O(m 只)：对这个落点的第 m 次读坏、其余照常。
    OnlyTheNthRead(u64),
    /// O(m 起)：第 m 次读起都坏。
    #[allow(
        dead_code,
        reason = "L5（代码审阅第 22 条那一格）归第二段；第一段只在单测里构造它"
    )]
    FromTheNthRead(u64),
}

impl FaultDuration {
    fn name(self) -> String {
        match self {
            FaultDuration::WholeCall => "W".to_string(),
            FaultDuration::UntilVirtualTime(until) => format!("T{until}"),
            FaultDuration::OnlyTheNthRead(nth) => format!("O{nth}only"),
            FaultDuration::FromTheNthRead(nth) => format!("O{nth}from"),
        }
    }

    /// 对这个落点的第 `nth_read` 次读（从 1 数）在虚拟时刻 `virtual_now` 坏不坏。
    fn is_active(self, nth_read: u64, virtual_now: u64) -> bool {
        match self {
            FaultDuration::WholeCall => true,
            FaultDuration::UntilVirtualTime(until) => virtual_now < until,
            FaultDuration::OnlyTheNthRead(nth) => nth_read == nth,
            FaultDuration::FromTheNthRead(nth) => nth_read >= nth,
        }
    }

    /// 装置自己的第一遍读（登记「读法写死」表 E_故障 那一格）：这个落点这次调用里第一次被读时坏不坏（虚拟时钟为 0）。
    fn is_active_on_the_first_read(self) -> bool {
        self.is_active(1, 0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ThirdRunFault {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    form: UnreadableForm,
    duration: FaultDuration,
    label: String,
}

impl ThirdRunFault {
    fn overlaps(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && self.offset < offset + length && offset < self.offset + self.length
    }
}

#[derive(Default)]
struct ThirdRunDeviceState {
    faults: Vec<ThirdRunFault>,
    written_over: Vec<bool>,
    reads_of_the_target: Vec<u64>,
    intercepted_reads: Vec<u64>,
    read_calls: u64,
    read_bytes: u64,
    write_calls: u64,
    barrier_calls: u64,
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
    injected_failure_fired: bool,
    applied: Vec<AppliedDeviceStep>,
}

impl ThirdRunDeviceState {
    fn new(
        faults: Vec<ThirdRunFault>,
        failing_write_call: Option<u64>,
        failing_barrier_call: Option<u64>,
    ) -> Self {
        let count = faults.len();
        ThirdRunDeviceState {
            faults,
            written_over: vec![false; count],
            reads_of_the_target: vec![0; count],
            intercepted_reads: vec![0; count],
            failing_write_call,
            failing_barrier_call,
            ..ThirdRunDeviceState::default()
        }
    }

    fn mark_written_over(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        for (fault, written_over) in self.faults.iter().zip(self.written_over.iter_mut()) {
            *written_over = *written_over || fault.overlaps(device, offset, length);
        }
    }
}

struct ThirdRunDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    state: Rc<RefCell<ThirdRunDeviceState>>,
}

impl BlockDevice for ThirdRunDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let virtual_now = third_run_arm_hooks::virtual_now();
        let mut fails = false;
        let mut zeroed: Vec<(u64, u64)> = Vec::new();
        {
            let mut guard = self.state.borrow_mut();
            let state = &mut *guard;
            state.read_calls += 1;
            state.read_bytes += length;
            for index in 0..state.faults.len() {
                if state.written_over[index]
                    || !state.faults[index].overlaps(self.identity, offset.0, length)
                {
                    continue;
                }
                state.reads_of_the_target[index] += 1;
                let duration = state.faults[index].duration;
                if !duration.is_active(state.reads_of_the_target[index], virtual_now) {
                    continue;
                }
                state.intercepted_reads[index] += 1;
                match state.faults[index].form {
                    UnreadableForm::ReadFails => {
                        fails = true;
                    }
                    UnreadableForm::ReadsZeros => {
                        zeroed.push((state.faults[index].offset, state.faults[index].length));
                    }
                }
            }
        }
        if fails {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)?;
        for (range_offset, range_length) in zeroed {
            let start = range_offset.saturating_sub(offset.0);
            let end = (range_offset + range_length)
                .saturating_sub(offset.0)
                .min(length);
            buffer[usize::try_from(start).expect("缓冲区里")
                ..usize::try_from(end).expect("缓冲区里")]
                .fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        let this_write_is_the_injected_failure = state.failing_write_call == Some(ordinal);
        if this_write_is_the_injected_failure {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)?;
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::Write {
            device: self.identity,
            offset: offset.0,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        if state.failing_write_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_zeroes_at(offset, length)?;
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::ZeroFill {
            device: self.identity,
            offset: offset.0,
            length,
        });
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.barrier_calls;
        state.barrier_calls += 1;
        if state.failing_barrier_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("屏障"));
        }
        self.inner.barrier()?;
        state.applied.push(AppliedDeviceStep::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一次受观测的入口调用交回的东西（可写挂载或冷启动恢复）。
struct ThirdRunCall<Outcome> {
    outcome: Outcome,
    write_calls: u64,
    read_calls: u64,
    read_bytes: u64,
    applied: Vec<AppliedDeviceStep>,
    intercepted_reads: Vec<u64>,
    injected_failure_fired: bool,
    pool_after: MemoryPool,
    waits: u64,
    observation: Option<ArmObservation>,
}

impl<Outcome> ThirdRunCall<Outcome> {
    /// V2：点了名的落点里一次都没被拦下的（O(m) 的：第 m 次读没发生）。
    fn unintercepted(&self, faults: &[ThirdRunFault]) -> Vec<String> {
        faults
            .iter()
            .zip(&self.intercepted_reads)
            .filter(|(_, intercepted)| **intercepted == 0)
            .map(|(fault, _)| fault.label.clone())
            .collect()
    }
}

fn third_run_call<Outcome>(
    base: &MemoryPool,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
    call: impl FnOnce(&mut Vec<(DeviceIdentity, ThirdRunDevice)>) -> Outcome,
) -> ThirdRunCall<Outcome> {
    let state = Rc::new(RefCell::new(ThirdRunDeviceState::new(
        faults.to_vec(),
        failing_write_call,
        failing_barrier_call,
    )));
    let mut devices: Vec<(DeviceIdentity, ThirdRunDevice)> = devices_from_pool(base, IMAGE_BYTES)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                ThirdRunDevice {
                    identity,
                    inner,
                    state: Rc::clone(&state),
                },
            )
        })
        .collect();
    third_run_arm_hooks::reset_the_virtual_clock();
    third_run_arm_hooks::install_the_wait_hook(THIRD_RUN_VIRTUAL_INTERVAL);
    let outcome = call(&mut devices);
    let observation = third_run_arm_hooks::take_the_arm_observation();
    let waits = third_run_arm_hooks::waits();
    let plain: Vec<(DeviceIdentity, SparseBlockDevice)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    let pool_after = memory_pool_of(&plain, IMAGE_BYTES);
    let state = state.borrow();
    ThirdRunCall {
        outcome,
        write_calls: state.write_calls,
        read_calls: state.read_calls,
        read_bytes: state.read_bytes,
        applied: state.applied.clone(),
        intercepted_reads: state.intercepted_reads.clone(),
        injected_failure_fired: state.injected_failure_fired,
        pool_after,
        waits,
        observation,
    }
}

fn run_third_run_mount_writable(
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
) -> ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>> {
    third_run_call(
        base,
        faults,
        failing_write_call,
        failing_barrier_call,
        |devices| mount_writable(parameters, devices),
    )
}

fn run_third_run_recover(
    base: &MemoryPool,
    faults: &[ThirdRunFault],
) -> ThirdRunCall<singlefs_core::recovery::RecoveryReport> {
    third_run_call(base, faults, None, None, |devices| {
        recover(&*devices, JournalPolicy::Consult)
    })
}

/// 结局按 K0–K4 分（同第 2 次跑 `mount_outcome_class`，按设备一层写次数分 K3 / K4）。
fn third_run_outcome_class(
    result: &Result<Mounted, singlefs_core::mount::MountError>,
    write_calls: u64,
) -> &'static str {
    mount_outcome_class(result, write_calls)
}

// ---------------------------------------------------------------------------
// 11.2 装置自己的解码（登记 5.0 第二条）：根槽、系统配置槽、journal 记录走 `singlefs_checker`（另一套解析代码），
//      择根取 (txg, 实例代号) 最大，按 D23（journal 的角色与格式） 已定项 14 五条口径自己重放出 E。
//      带故障的第一遍读：落点在这次调用里第一次被读时坏的，当读不出（读错）或读回全零（清零）。
// ---------------------------------------------------------------------------

/// 装置的第一遍读：`pool` 上盖着 `faults` 里「第一次读就坏」的那几个落点。
struct FirstPassReader<'pool> {
    pool: &'pool MemoryPool,
    faults: &'pool [ThirdRunFault],
}

impl ImageReader for FirstPassReader<'_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.pool)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.pool, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let length_u64 = u64::try_from(length).expect("读长");
        let active: Vec<&ThirdRunFault> = self
            .faults
            .iter()
            .filter(|fault| {
                fault.duration.is_active_on_the_first_read()
                    && fault.overlaps(DeviceIdentity(device), offset, length_u64)
            })
            .collect();
        if active
            .iter()
            .any(|fault| fault.form == UnreadableForm::ReadFails)
        {
            return None;
        }
        let mut bytes = ImageReader::read(self.pool, device, offset, length)?;
        for fault in active {
            let start = fault.offset.saturating_sub(offset);
            let end = (fault.offset + fault.length)
                .saturating_sub(offset)
                .min(length_u64);
            bytes[usize::try_from(start).expect("读内")..usize::try_from(end).expect("读内")]
                .fill(0);
        }
        Some(bytes)
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_unit_slots(self.pool, device)
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_journal_slots(self.pool, device)
    }
}

/// 装置解出的一份镜像：自证根的键 (实例代号, txg)、读错的根槽个数、全部自证记录（按 (实例代号, 计数器)）、
/// 每块盘两槽里自证过的系统配置 (盘, 世代号, tail, 实例代号)。
struct DeviceDecodedImage {
    root_keys: BTreeSet<TimelineRoot>,
    unreadable_root_slots: u64,
    records: BTreeMap<(u32, u64), singlefs_checker::JournalRecordView>,
    system_configuration_slots: Vec<(u32, u64, u64, u32)>,
}

fn device_decode(reader: &dyn ImageReader, geometry: &PoolGeometry) -> DeviceDecodedImage {
    let root_keys = checker_image::valid_roots(reader, geometry)
        .into_iter()
        .map(|(_, _, view)| (view.instance, view.checkpoint_txg))
        .collect();
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    let unreadable_root_slots = checker_image::root_slot_positions(geometry)
        .into_iter()
        .filter(|(_, _, device, offset)| reader.read(*device, *offset, root_slot_bytes).is_none())
        .count() as u64;
    let expected_filesystem_identifier = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("fsid 前 8 字节"),
    );
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let ring_start = geometry.journal_ring_start_slot * SLOT_BYTES_LOCAL;
    let mut records = BTreeMap::new();
    for device in reader.devices() {
        for index in 0..ring_slots {
            let Some(bytes) = reader.read(
                device,
                ring_start + index * JOURNAL_RECORD_BYTES,
                record_bytes,
            ) else {
                continue;
            };
            if let Ok(view) =
                singlefs_checker::check_journal_record(&bytes, expected_filesystem_identifier)
            {
                records.entry((view.instance, view.counter)).or_insert(view);
            }
        }
    }
    let system_configuration_slots = checker_image::verified_system_configuration_slots(reader)
        .into_iter()
        .flat_map(|(device, slots)| {
            slots.into_iter().map(move |(view, _)| {
                (
                    device,
                    view.slot_generation,
                    view.journal_tail,
                    view.journal_instance,
                )
            })
        })
        .collect();
    DeviceDecodedImage {
        root_keys,
        unreadable_root_slots,
        records,
        system_configuration_slots,
    }
}

/// 装置自己择根：自证根里 (txg, 实例代号) 最大的那一条。
fn device_chosen_root(decoded: &DeviceDecodedImage) -> Option<TimelineRoot> {
    decoded
        .root_keys
        .iter()
        .copied()
        .max_by_key(|(instance, txg)| (*txg, *instance))
}

fn record_ends_its_publish_by_the_device(record: &singlefs_checker::JournalRecordView) -> bool {
    record.record_flags_byte & 1 == 1
}

fn unit_bytes_of_class_by_the_device(unit_class: u8) -> Option<usize> {
    match unit_class {
        UNIT_CLASS_DATA_LOCAL | UNIT_CLASS_PACKED_LOCAL => Some(DATA_UNIT_BYTES_LOCAL),
        UNIT_CLASS_INDEX_NODE_LOCAL => Some(NODE_BYTES_LOCAL),
        _ => None,
    }
}

/// 装置自己的在飞记录数上限：环槽数 ÷ F，两次都取整数部分（D23（journal 的角色与格式） 已定项 18）。照「装置不与实现共用代码」
/// 自写一份，不调 core 的 `system_configuration::journal_in_flight_record_limit`，只取格式常量模块的两个标量；
/// 两份在环长上的交叉断言在 `journal_in_flight_record_limit_cross_check_tests`。
fn journal_in_flight_record_limit(ring_bytes: u64) -> u64 {
    let ring_record_slots = ring_bytes / JOURNAL_RECORD_BYTES;
    ring_record_slots / singlefs_format::JOURNAL_SAFETY_FACTOR
}

/// core 与装置两份在飞记录数上限在环长上的交叉断言。
///
/// 取值域：环长是系统配置里 8 字节的「环长度（字节）」字段，恢复照读盘上的值，所以两份要在整个 u64 上相等；mkfs 收的环长
/// 在 [3 × 4096, 设备容量 ÷ 4]（D23（journal 的角色与格式） 已定项 19 ③），容量被 6 字节槽号 × 16 KiB 槽
/// （D19（块指针的结构与宽度预算） 已定项 4）顶在 2^62 字节，所以按条款 mkfs 收得下的环长不过 2^60。
/// 2^64 个值扫不遍，只取枚举点：① 0 到 2^20 逐个取（两份只除以 4096 与 3，这一段把模 12288 的每个余数走了 85 遍）；
/// ② 每个 2 的幂与它 ±1，以及它以下最近的 12288 的倍数与那个倍数附近的 4096 边界（每个数量级上的截断、溢出、取整方向）；
/// ③ u64 顶端；④ 在飞上限刚越过 u32 的那一处（写系统配置时它要装进 4 字节）；⑤ 默认环 768 MiB。
/// 两份一起歪（改共享的两个标量）时互比看不出，靠 `the_in_flight_record_limit_is_pinned_at_values_worked_out_from_the_clauses` 钉的绝对值。
#[cfg(test)]
mod journal_in_flight_record_limit_cross_check_tests {
    use super::journal_in_flight_record_limit as apparatus_journal_in_flight_record_limit;
    use singlefs_core::system_configuration::journal_in_flight_record_limit as core_journal_in_flight_record_limit;
    use std::collections::BTreeSet;

    /// 多一条在飞记录要多多少环长：一条记录 4096（D23（journal 的角色与格式） 已定项 12）× F = 3（I-8.1（环几何够大））。
    /// 抄条款的数，不取格式常量：钉绝对值那条要在共享标量被改歪时照样红。
    const RING_BYTES_PER_IN_FLIGHT_RECORD: u64 = 4096 * 3;
    const DENSELY_SCANNED_RING_BYTES_UPPER_END: u64 = 1 << 20;
    const DEFAULT_RING_BYTES: u64 = 768 * 1024 * 1024;
    const LARGEST_RING_BYTES_MAKE_FILESYSTEM_ACCEPTS: u64 = 1 << 60;
    const RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES: u64 =
        (1 << 32) * RING_BYTES_PER_IN_FLIGHT_RECORD;

    /// 枚举点 ② 到 ⑤（① 那一段在测试里直接逐个走，不进这个集合）。
    fn ring_sizes_at_every_magnitude_and_at_the_edges() -> BTreeSet<u64> {
        let mut ring_sizes = BTreeSet::new();
        for bit_position in 0..u64::BITS {
            let power_of_two = 1u64 << bit_position;
            let multiple_below =
                power_of_two / RING_BYTES_PER_IN_FLIGHT_RECORD * RING_BYTES_PER_IN_FLIGHT_RECORD;
            for anchor in [power_of_two, multiple_below] {
                for offset_below in [1u64, 0] {
                    if let Some(ring_bytes) = anchor.checked_sub(offset_below) {
                        ring_sizes.insert(ring_bytes);
                    }
                }
                for offset_above in [1u64, 4095, 4096, 8191, 8192] {
                    if let Some(ring_bytes) = anchor.checked_add(offset_above) {
                        ring_sizes.insert(ring_bytes);
                    }
                }
            }
        }
        let top_multiple =
            u64::MAX / RING_BYTES_PER_IN_FLIGHT_RECORD * RING_BYTES_PER_IN_FLIGHT_RECORD;
        for ring_bytes in [
            u64::MAX,
            u64::MAX - 1,
            top_multiple - 1,
            top_multiple,
            top_multiple + 1,
        ] {
            ring_sizes.insert(ring_bytes);
        }
        for ring_bytes in [
            RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES - 1,
            RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES,
            RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES + 1,
            DEFAULT_RING_BYTES,
        ] {
            ring_sizes.insert(ring_bytes);
        }
        ring_sizes
    }

    #[test]
    fn core_and_apparatus_compute_the_same_in_flight_record_limit_at_every_enumerated_ring_size() {
        let mut densely_compared = 0u64;
        for ring_bytes in 0..=DENSELY_SCANNED_RING_BYTES_UPPER_END {
            assert_eq!(
                core_journal_in_flight_record_limit(ring_bytes),
                apparatus_journal_in_flight_record_limit(ring_bytes),
                "环长 {ring_bytes} 字节：core 与装置的在飞上限不等"
            );
            densely_compared += 1;
        }
        assert_eq!(
            densely_compared,
            DENSELY_SCANNED_RING_BYTES_UPPER_END + 1,
            "① 那一段从 0 到 2^20 一个不落"
        );
        let edge_ring_sizes = ring_sizes_at_every_magnitude_and_at_the_edges();
        for ring_bytes in &edge_ring_sizes {
            assert_eq!(
                core_journal_in_flight_record_limit(*ring_bytes),
                apparatus_journal_in_flight_record_limit(*ring_bytes),
                "环长 {ring_bytes} 字节：core 与装置的在飞上限不等"
            );
        }
        for must_be_compared in [
            u64::MAX,
            LARGEST_RING_BYTES_MAKE_FILESYSTEM_ACCEPTS,
            RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES,
            DEFAULT_RING_BYTES,
        ] {
            assert!(
                edge_ring_sizes.contains(&must_be_compared),
                "枚举点里要有 {must_be_compared}：取值域的这一端没比到"
            );
        }
    }

    /// 绝对值按条款手算（记录 4096、F = 3，两次取整数部分），不调任何一份式子。
    #[test]
    fn the_in_flight_record_limit_is_pinned_at_values_worked_out_from_the_clauses() {
        let pinned: [(u64, u64, &str); 7] = [
            (
                805_306_368,
                65_536,
                "默认环 768 MiB：196608 个记录槽 ÷ 3（D23 已定项 18 的定案原文）",
            ),
            (12_287, 0, "不到 3 条记录：mkfs 拒的那一格"),
            (12_288, 1, "恰好 3 条记录：mkfs 收得下的最短环"),
            (
                1_152_921_504_606_846_976,
                93_824_992_236_885,
                "2^60：按条款 mkfs 收得下的最长环，2^48 ÷ 3 的整数部分",
            ),
            (
                52_776_558_133_247,
                4_294_967_295,
                "在飞上限恰好还装得进 4 字节",
            ),
            (52_776_558_133_248, 4_294_967_296, "在飞上限刚越过 4 字节"),
            (u64::MAX, 1_501_199_875_790_165, "u64 顶端：(2^52 − 1) ÷ 3"),
        ];
        for (ring_bytes, expected_limit, why) in pinned {
            assert_eq!(
                core_journal_in_flight_record_limit(ring_bytes),
                expected_limit,
                "core：{why}"
            );
            assert_eq!(
                apparatus_journal_in_flight_record_limit(ring_bytes),
                expected_limit,
                "装置：{why}"
            );
        }
    }
}

/// 装置按 D23（journal 的角色与格式） 已定项 14 五条口径从所选根重放：前缀不跨实例、链首锚在所选根那次发布带末条标志的那一条
/// （读不出就要求第一条 txg = 根 + 1 且序号 1）、计数器连号、发布边界按末条标志、序号连续、提交标记、点名单元两份都读得出且
/// 整单元 CRC-32C 对；末条到了才整体施加。交回 E 的键；所选根那次发布带末条标志的记录多于一条交 Err（crates 那一格也报错）。
fn device_replay(
    reader: &dyn ImageReader,
    decoded: &DeviceDecodedImage,
    root: TimelineRoot,
    ring_bytes: u64,
) -> Result<TimelineRoot, String> {
    let (instance, txg) = root;
    let anchors: Vec<u64> = decoded
        .records
        .values()
        .filter(|record| {
            record.instance == instance
                && record.checkpoint_txg == txg
                && record_ends_its_publish_by_the_device(record)
        })
        .map(|record| record.counter)
        .collect();
    if anchors.len() > 1 {
        return Err(format!(
            "所选根 {root:?} 那次发布带末条标志的记录 {} 条",
            anchors.len()
        ));
    }
    let mut above: Vec<&singlefs_checker::JournalRecordView> = decoded
        .records
        .values()
        .filter(|record| record.instance == instance && record.checkpoint_txg > txg)
        .collect();
    above.sort_by_key(|record| record.counter);
    let limit = usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
    let mut expected_next = anchors.first().map(|counter| counter + 1);
    let mut open: Vec<&singlefs_checker::JournalRecordView> = Vec::new();
    let mut effective = root;
    for record in above.into_iter().take(limit) {
        match expected_next {
            Some(expected) => {
                if record.counter != expected {
                    break;
                }
            }
            None => {
                if record.checkpoint_txg != txg + 1 || record.ordinal_within_publish != 1 {
                    break;
                }
            }
        }
        if open.is_empty() && record.ordinal_within_publish != 1 {
            break;
        }
        if let Some(previous) = open.last() {
            if record.checkpoint_txg != previous.checkpoint_txg
                || record.ordinal_within_publish != previous.ordinal_within_publish + 1
                || (!previous.is_commit && record.transaction != previous.transaction)
            {
                break;
            }
        }
        expected_next = Some(record.counter + 1);
        let ends = record_ends_its_publish_by_the_device(record);
        if !record.is_commit && ends {
            break;
        }
        if ends
            && decoded
                .records
                .range((record.instance, record.counter + 1)..=(record.instance, u64::MAX))
                .map(|(_, later)| later)
                .take_while(|later| later.checkpoint_txg <= record.checkpoint_txg)
                .any(|later| later.checkpoint_txg == record.checkpoint_txg)
        {
            break;
        }
        let all_verified = record.named.iter().all(|named| {
            let Some(unit_bytes) = unit_bytes_of_class_by_the_device(named.unit_class) else {
                return false;
            };
            named.locations.iter().all(|(device, slot, checksum)| {
                reader
                    .read(*device, slot * SLOT_BYTES_LOCAL, unit_bytes)
                    .is_some_and(|bytes| {
                        singlefs_checker::crc32_castagnoli_table(&bytes) == *checksum
                    })
            })
        });
        if !all_verified {
            break;
        }
        open.push(record);
        if !ends {
            continue;
        }
        open.clear();
        effective = (record.instance, record.checkpoint_txg);
    }
    Ok(effective)
}

/// 装置按登记 5.1 第一行算 N-配置：(N, c_见证, c_E, 判不出)。
fn device_configuration_witness(
    decoded: &DeviceDecodedImage,
    effective: TimelineRoot,
) -> (bool, u64, Option<u64>, bool) {
    let witnessed_tail = decoded
        .system_configuration_slots
        .iter()
        .map(|(_, _, tail, _)| *tail)
        .max()
        .unwrap_or(0);
    let selected_version_last_counter = decoded
        .records
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == effective
                && record_ends_its_publish_by_the_device(record)
        })
        .map(|record| record.counter)
        .max();
    if witnessed_tail == 0 {
        return (false, 0, selected_version_last_counter, false);
    }
    if let Some(counter) = selected_version_last_counter {
        return (
            witnessed_tail > counter,
            witnessed_tail,
            selected_version_last_counter,
            false,
        );
    }
    match decoded
        .records
        .values()
        .find(|record| record.counter == witnessed_tail)
    {
        Some(witness) => (
            (witness.instance, witness.checkpoint_txg) > effective,
            witnessed_tail,
            None,
            false,
        ),
        None => (true, witnessed_tail, None, true),
    }
}

/// 装置对一次被测挂载（带故障的第一遍读）与它的孪生镜像（不装故障）算的东西。
struct DeviceJudgement {
    chosen: Option<TimelineRoot>,
    effective: Option<TimelineRoot>,
    configuration_newer: bool,
    witnessed_tail: u64,
    selected_version_last_counter: Option<u64>,
    undecidable: bool,
    unreadable_root_slots: u64,
    /// N_真：孪生镜像上有自证根的键 (实例代号, txg) 大于 E_故障 的。
    truth_newer: bool,
    /// 同一句按择根的次序 (txg, 实例代号) 比，与上面那个不同的格另记。
    truth_orders_disagree: bool,
    /// 孪生镜像上重放能走得比 E_故障 远、而没有键更大的根槽（标签「只由记录走得更远」）。
    truth_by_records_only: bool,
    error: Option<String>,
}

fn device_judgement(
    twin: &MemoryPool,
    geometry: &PoolGeometry,
    faults: &[ThirdRunFault],
) -> DeviceJudgement {
    let reader = FirstPassReader { pool: twin, faults };
    let decoded = device_decode(&reader, geometry);
    let chosen = device_chosen_root(&decoded);
    let mut error = None;
    let effective = chosen.and_then(|root| {
        device_replay(&reader, &decoded, root, geometry.journal_ring_bytes)
            .map_err(|message| error = Some(message))
            .ok()
    });
    let (configuration_newer, witnessed_tail, selected_version_last_counter, undecidable) =
        match effective {
            Some(effective) => device_configuration_witness(&decoded, effective),
            None => (false, 0, None, false),
        };
    let twin_decoded = device_decode(twin, geometry);
    let (truth_newer, truth_orders_disagree, truth_by_records_only) = match effective {
        Some(effective) => {
            let by_key = twin_decoded.root_keys.iter().any(|key| *key > effective);
            let by_choice_order = twin_decoded
                .root_keys
                .iter()
                .any(|(instance, txg)| (*txg, *instance) > (effective.1, effective.0));
            let twin_effective = device_chosen_root(&twin_decoded).and_then(|root| {
                device_replay(twin, &twin_decoded, root, geometry.journal_ring_bytes).ok()
            });
            let by_records_only = !by_key
                && twin_effective
                    .is_some_and(|twin_key| (twin_key.1, twin_key.0) > (effective.1, effective.0));
            (by_key, by_key != by_choice_order, by_records_only)
        }
        None => (false, false, false),
    };
    DeviceJudgement {
        chosen,
        effective,
        configuration_newer,
        witnessed_tail,
        selected_version_last_counter,
        undecidable,
        unreadable_root_slots: decoded.unreadable_root_slots,
        truth_newer,
        truth_orders_disagree,
        truth_by_records_only,
        error,
    }
}

fn key_text(key: Option<TimelineRoot>) -> String {
    key.map_or("none".to_string(), |(instance, txg)| {
        format!("{instance}:{txg}")
    })
}

// ---------------------------------------------------------------------------
// 11.3 历史与确认账（登记「读法写死」表「确认」一行）：每次改内容的入口调用交回 Ok 记一笔（内容号、那一版的根键、那一版引用的
//      每个单元在每块盘上的字节）；空发布、写行、暖机、抬 F 沿用上一版的内容号。内容号 k 的文件字节 = `deterministic_content(999 + k)`。
// ---------------------------------------------------------------------------

/// 一版引用的每个单元（盘, 起始槽, 槽数）在某一刻盘上的字节。
type UnitBytesOfAVersion = BTreeMap<(u32, u64, u64), Vec<u8>>;

#[derive(Clone)]
struct ThirdRunConfirmation {
    content_number: u64,
    root: TimelineRoot,
    /// 那一版引用的每个单元确认那一刻在盘上的字节。
    unit_bytes: UnitBytesOfAVersion,
}

#[derive(Clone)]
struct ThirdRunHistory {
    pool: MemoryPool,
    session: Option<Session>,
    timeline: Vec<TimelineRoot>,
    content_number_by_root: BTreeMap<TimelineRoot, u64>,
    confirmations: Vec<ThirdRunConfirmation>,
    /// 7.1 第一行（F3）核过的确认次数与不符的次数。
    tail_checks: u64,
    tail_check_failures: Vec<String>,
}

impl ThirdRunHistory {
    fn tip(&self) -> TimelineRoot {
        *self.timeline.last().expect("时间线至少有第 0 代根")
    }

    fn last_confirmed_content_number(&self) -> u64 {
        self.confirmations
            .last()
            .map_or(0, |confirmation| confirmation.content_number)
    }

    fn content_number_of(&self, root: TimelineRoot) -> Option<u64> {
        self.content_number_by_root.get(&root).copied()
    }

    fn push_carried(&mut self, root: TimelineRoot, content_number: u64) {
        self.timeline.push(root);
        self.content_number_by_root.insert(root, content_number);
    }
}

fn third_run_content(content_number: u64) -> Vec<u8> {
    deterministic_content(999 + content_number, FIRST_FILE_BYTES)
}

/// 读回的文件字节是第几号内容（不是任何一号就 None）。
fn content_number_of_bytes(bytes: &[u8], highest: u64) -> Option<u64> {
    (1..=highest).find(|number| third_run_content(*number) == bytes)
}

fn unit_bytes_referenced_by(
    pool: &MemoryPool,
    root: &RootRecord,
) -> Result<UnitBytesOfAVersion, String> {
    let placements = placements_referenced_by_root_on(pool, root)?;
    Ok(placements
        .into_iter()
        .map(|(device, slot, span)| {
            let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
            let bytes = pool
                .read(device, slot * SLOT_BYTES_LOCAL, length)
                .unwrap_or_default();
            ((device, slot, span), bytes)
        })
        .collect())
}

/// 7.1 第一行（F3）：一次确认之后，每块盘世代号最新的自证系统配置槽里 tail = 那次发布末条记录的计数器、实例代号 = 那次发布的实例代号。
fn check_the_tail_after_a_confirmation(history: &mut ThirdRunHistory, root: TimelineRoot) {
    history.tail_checks += 1;
    let Ok(geometry) = independent_geometry(&history.pool) else {
        history
            .tail_check_failures
            .push(format!("{root:?}: 几何解不出"));
        return;
    };
    let decoded = device_decode(&history.pool, &geometry);
    let last_counter = decoded
        .records
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == root
                && record_ends_its_publish_by_the_device(record)
        })
        .map(|record| record.counter)
        .max();
    for device in [0u32, 1] {
        let newest = decoded
            .system_configuration_slots
            .iter()
            .filter(|(slot_device, _, _, _)| *slot_device == device)
            .max_by_key(|(_, generation, _, _)| *generation);
        let holds = newest.is_some_and(|(_, _, tail, instance)| {
            Some(*tail) == last_counter && *instance == root.0
        });
        if !holds {
            history.tail_check_failures.push(format!(
                "{root:?} 盘 {device}: 最新槽 {newest:?}，那次发布末条记录计数器 {last_counter:?}"
            ));
        }
    }
}

fn record_confirmation(
    history: &mut ThirdRunHistory,
    root_record: &RootRecord,
    content_number: u64,
) -> Result<(), String> {
    let root = root_pair(root_record);
    let unit_bytes = unit_bytes_referenced_by(&history.pool, root_record)?;
    history.push_carried(root, content_number);
    history.confirmations.push(ThirdRunConfirmation {
        content_number,
        root,
        unit_bytes,
    });
    check_the_tail_after_a_confirmation(history, root);
    Ok(())
}

/// 一次可写挂载写下的根（写行、暖机、抬 F）进时间线，沿用 `content_number`。
fn push_the_publishes_of_a_mount(
    history: &mut ThirdRunHistory,
    mounted: &Mounted,
    content_number: u64,
) {
    let mut roots = vec![root_pair(mounted.output.row_publish.root())];
    roots.extend(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| root_pair(publish.root())),
    );
    for raised in mounted.output.space_admission.floor_raises() {
        roots.extend(
            raised
                .publishes
                .iter()
                .map(|publish| root_pair(&publish.root)),
        );
    }
    for root in roots {
        history.push_carried(root, content_number);
    }
}

/// 在开着的会话上做一次改内容的调用（现行那一版有文件就覆盖写，没有就写首个文件），交回 Ok 就记一笔确认。
fn third_run_change_content(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<(), String> {
    let content_number = history.last_confirmed_content_number() + 1;
    let content = third_run_content(content_number);
    let mut session = history.session.take().ok_or("改内容：没有开着的会话")?;
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let file = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * content_number,
        };
        match &session.current {
            PoolVersion::WithFile(previous) => publish_overwrite(
                &mut writer,
                &mut session.allocator,
                previous,
                file,
                session.instance,
            )
            .map_err(|error| format!("覆盖写: {error:?}"))?,
            PoolVersion::WithoutFile(version) => publish_first_file(
                &mut writer,
                &mut session.allocator,
                &version.root,
                file,
                session.instance,
                session.current.record_bytes(),
            )
            .map_err(|error| format!("首个文件: {error:?}"))?,
        }
    };
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let root_record = output.root;
    session.current = PoolVersion::WithFile(output);
    history.session = Some(session);
    record_confirmation(history, &root_record, content_number)
}

/// P(n1, ·) 的前半：mkfs → `mount_writable`（实例 1）→ 首个文件 → n1 次覆盖写，会话开着。
fn third_run_history_start(
    geometry: &Geometry,
    overwrites: u64,
) -> Result<ThirdRunHistory, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let mounted = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("第一次可写挂载: {error:?}"))?;
    let mut history = ThirdRunHistory {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: None,
        timeline: vec![root_pair(&genesis.root)],
        content_number_by_root: BTreeMap::from([(root_pair(&genesis.root), 0)]),
        confirmations: Vec::new(),
        tail_checks: 0,
        tail_check_failures: Vec::new(),
    };
    push_the_publishes_of_a_mount(&mut history, &mounted, 0);
    history.session = Some(Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    });
    for _ in 0..=overwrites {
        third_run_change_content(&mut history, &parameters)?;
    }
    Ok(history)
}

/// 关闭 c（同第 2 次跑）：C 丢掉会话；U 调 `unmount`，并核 7.1 第六行（F14）：F 抬到现行那一版的 txg、空发布 2 或 3 次。
/// 交回 F14 那一格的判定行（C 交 None）。
fn third_run_close(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
    closing: Closing,
) -> Result<Option<(bool, String)>, String> {
    let Some(session) = history.session.take() else {
        return Ok(None);
    };
    if closing == Closing::ProcessExit {
        return Ok(None);
    }
    let Session {
        mut allocator,
        mut current,
        ..
    } = session;
    let before_txg = current.root().checkpoint_txg.0;
    let carried = history.content_number_of(history.tip()).unwrap_or(0);
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let unmounted = singlefs_core::mount::unmount(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .map_err(|error| format!("正常卸载: {error:?}"))?;
    let singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        return Err("正常卸载：现行那一版没有文件".to_string());
    };
    for publish in &raised.publishes {
        history.push_carried(root_pair(&publish.root), carried);
    }
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let expected_publishes = if before_txg % 3 == 1 { 3 } else { 2 };
    let floors = [0u32, 1].map(|device| {
        let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
        newest_system_configuration_slot_bytes(&history.pool, DeviceIdentity(device), slot_spacing)
            .and_then(|bytes| {
                singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes).ok()
            })
            .map(|parsed| parsed.quantities.rollback_floor.0)
    });
    let holds = floors.iter().all(|floor| *floor == Some(before_txg))
        && raised.publishes.len() == expected_publishes;
    Ok(Some((
        holds,
        format!(
            "txg_before_unmount={before_txg} floors={floors:?} unmount_publishes={} expected_publishes={expected_publishes}",
            raised.publishes.len()
        ),
    )))
}

/// 抛弃步 A 的落点（登记 5.3「抛弃步 A」）：当前时间线上最新那 1 条根的根槽，加它那次发布按 b 取的落点
/// （b = records：那次发布全部记录的全部份；b = unit_first_copy：它第一条记录点名的第一个单元、位置条目第一条那一份）。
/// 复制自第 2 次跑的 `unreadable_ranges_of_the_abandonment`（那一份吃 `SecondRunHistory`，这里吃第 3 次跑的历史），语义不变。
fn third_run_abandonment_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    form: UnreadableForm,
    replay_break: ReplayBreak,
    duration: FaultDuration,
) -> Result<Option<Vec<ThirdRunFault>>, String> {
    let hidden = history.tip();
    let (root_slot_device, root_slot_offset) = root_slot_of(&history.pool, geometry, hidden)
        .ok_or_else(|| format!("最新那条根 {hidden:?} 在根环里读不出"))?;
    let mut faults = vec![ThirdRunFault {
        device: root_slot_device,
        offset: root_slot_offset,
        length: u64::from(geometry.physical_block_size),
        form,
        duration,
        label: format!("root{}", key_text(Some(hidden))),
    }];
    let records = records_of_publish(&history.pool, geometry, hidden);
    if records.is_empty() {
        return Ok(None);
    }
    match replay_break {
        ReplayBreak::EveryRecord => {
            for (counter, record_offset_on_the_device, _) in &records {
                for record_device in [DeviceIdentity(0), DeviceIdentity(1)] {
                    faults.push(ThirdRunFault {
                        device: record_device,
                        offset: *record_offset_on_the_device,
                        length: JOURNAL_RECORD_BYTES,
                        form,
                        duration,
                        label: format!("record(counter={counter},device={})", record_device.0),
                    });
                }
            }
        }
        ReplayBreak::FirstCopyOfTheFirstNamedUnit | ReplayBreak::FirstNamedUnit => {
            let Some(named) = records
                .iter()
                .find_map(|(_, _, record)| record.named.first())
            else {
                return Ok(None);
            };
            let copies = match replay_break {
                ReplayBreak::FirstNamedUnit => named.locations.len(),
                ReplayBreak::EveryRecord | ReplayBreak::FirstCopyOfTheFirstNamedUnit => 1,
            };
            for location in named.locations.iter().take(copies) {
                faults.push(ThirdRunFault {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form,
                    duration,
                    label: format!(
                        "unit(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                });
            }
        }
    }
    Ok(Some(faults))
}

// ---------------------------------------------------------------------------
// 11.4 一格的三样（登记第六节 Q1–Q3、③c 尾巴）与被测挂载的上下文（Q0、Q2、Q5 的读数、Q7）。
// ---------------------------------------------------------------------------

/// 被测那次挂载（A）的一次调用的摘要（Q7、Q5 的读数、Q2 的 E）。
struct TestedMountSummary {
    class: &'static str,
    member: String,
    writes: u64,
    reads: u64,
    read_bytes: u64,
    waits: u64,
    chosen: Option<TimelineRoot>,
    effective: Option<TimelineRoot>,
    abandoned_roots_unreadable: Option<u64>,
    isolated: Option<u64>,
    observation: Option<ArmObservation>,
}

impl TestedMountSummary {
    fn of(call: &ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>) -> Self {
        let mounted = call.outcome.as_ref().ok();
        TestedMountSummary {
            class: third_run_outcome_class(&call.outcome, call.write_calls),
            member: mount_error_member_or_none(&call.outcome),
            writes: call.write_calls,
            reads: call.read_calls,
            read_bytes: call.read_bytes,
            waits: call.waits,
            chosen: mounted.map(|mounted| root_pair(&mounted.output.chosen_root)),
            effective: mounted.map(|mounted| root_pair(&mounted.output.effective_root)),
            abandoned_roots_unreadable: mounted
                .map(|mounted| mounted.output.abandoned_roots_unreadable),
            isolated: mounted.map(|mounted| {
                mounted
                    .output
                    .isolated_slots_per_device
                    .iter()
                    .map(|(_, slots)| *slots)
                    .sum()
            }),
            observation: call.observation.clone(),
        }
    }

    fn render(&self, prefix: &str) -> String {
        let observation = self.observation.as_ref().map_or("none".to_string(), |observation| {
            format!(
                "judged:{},newer:{},tail:{},last:{},undecidable:{},unreadable_slots:{},rounds:{},rebuilt:{},table_unreadable:{},table_rereads:{}",
                observation.witness_judged,
                observation.newer_root_on_the_first_pass,
                observation.witnessed_tail,
                observation
                    .selected_version_last_counter
                    .map_or("none".to_string(), |counter| counter.to_string()),
                observation.undecidable,
                observation.unreadable_ring_slots_on_the_first_pass,
                observation.reread_rounds,
                observation.rebuilt_from_records,
                observation.shadow_ledger_table_unreadable_on_the_first_read,
                observation.shadow_ledger_table_rereads
            )
        });
        format!(
            "{prefix}_class={} {prefix}_error={} {prefix}_writes={} {prefix}_reads={} {prefix}_read_bytes={} {prefix}_waits={} {prefix}_chosen={} {prefix}_effective={} {prefix}_abandoned_roots_unreadable={} {prefix}_isolated={} {prefix}_arm_observation={observation}",
            self.class,
            self.member,
            self.writes,
            self.reads,
            self.read_bytes,
            self.waits,
            key_text(self.chosen),
            key_text(self.effective),
            self.abandoned_roots_unreadable
                .map_or("none".to_string(), |count| count.to_string()),
            self.isolated.map_or("none".to_string(), |count| count.to_string()),
        )
    }
}

/// 一格历史末尾的三样（Q1 三栏、Q3、③c）。
struct CellLoss {
    in_ring_overwritten: u64,
    out_of_ring_overwritten: u64,
    landing_overwritten: String,
    read_back: String,
    read_back_root: Option<TimelineRoot>,
    tail_class: String,
}

impl CellLoss {
    fn overwrite_cell(&self) -> bool {
        self.in_ring_overwritten + self.out_of_ring_overwritten > 0
    }

    fn read_back_lost(&self) -> bool {
        self.read_back.starts_with("older") || self.read_back.starts_with("failed")
    }

    fn tail_lost(&self) -> bool {
        self.tail_class.starts_with("failed")
    }

    fn lost_write_cell(&self) -> bool {
        self.overwrite_cell() || self.read_back_lost() || self.tail_lost()
    }

    fn render(&self) -> String {
        format!(
            "q1_in_ring_overwritten={} q1_out_of_ring_overwritten={} q1_landing_overwritten={} overwrite_cell={} q3_read_back={} q3_read_back_root={} q3c_tail={} lost_write_cell={}",
            self.in_ring_overwritten,
            self.out_of_ring_overwritten,
            self.landing_overwritten,
            self.overwrite_cell(),
            self.read_back,
            key_text(self.read_back_root),
            self.tail_class,
            self.lost_write_cell()
        )
    }
}

/// 冷启动读回按内容号分四类（Q3）：last_confirmed / newer / older / failed:<成员>。
fn classify_read_back(
    outcome: &RecoveryOutcome,
    last_confirmed: u64,
    highest_content_number: u64,
) -> String {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            match content_number_of_bytes(content, highest_content_number) {
                Some(number) if number == last_confirmed => "last_confirmed".to_string(),
                Some(number) if number > last_confirmed => format!("newer:{number}"),
                Some(number) => format!("older:{number}"),
                None => "failed:content_not_in_the_ledger".to_string(),
            }
        }
        RecoveryOutcome::NoFile { .. } => {
            if last_confirmed == 0 {
                "last_confirmed".to_string()
            } else {
                "older:0".to_string()
            }
        }
        RecoveryOutcome::Failed { failure, .. } => {
            format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
        }
    }
}

/// 历史末尾（`end`）上算一格的三样。`protected` = 被测那次挂载之前最后确认那一版；`base` = 被测那次挂载开始时的镜像；
/// `first_instance_at_or_after_the_tested_mount` = ③c 要读坏其根槽的最小实例代号（`None` = 这一格不做 ③c）。
fn evaluate_cell_loss(
    history: &ThirdRunHistory,
    protected: &ThirdRunConfirmation,
    base: &MemoryPool,
    end: &MemoryPool,
    first_instance_at_or_after_the_tested_mount: Option<u32>,
) -> CellLoss {
    let geometry = independent_geometry(end);
    let end_roots = geometry
        .as_ref()
        .map(|geometry| readable_roots_independent(end, geometry))
        .unwrap_or_default();
    let highest = history.last_confirmed_content_number();
    let report = recover(&devices_from_pool(end, IMAGE_BYTES), JournalPolicy::Consult);
    let read_back_root = chosen_root_of(&report.outcome);
    let read_back = classify_read_back(&report.outcome, highest, highest);
    // 最后那次冷启动恢复落到的根（第 2 次跑的 ①）：它在被测那次挂载开始时的镜像上就是一条自证根时，取它那一版引用的单元在那一刻的字节。
    let base_geometry = independent_geometry(base);
    let landing_in_base = match (read_back_root, &base_geometry) {
        (Some(landing), Ok(base_pool_geometry)) => {
            readable_roots_independent(base, base_pool_geometry).contains(&landing)
        }
        (Some(_) | None, _) => false,
    };
    let landing_unit_bytes: Result<UnitBytesOfAVersion, String> =
        match (read_back_root, &base_geometry) {
            (Some(landing), Ok(base_pool_geometry)) if landing_in_base => {
                root_record_of(base, base_pool_geometry, landing)
                    .ok_or_else(|| "根记录读不出".to_string())
                    .and_then(|record| unit_bytes_referenced_by(base, &record))
            }
            (Some(_) | None, _) => Ok(BTreeMap::new()),
        };
    let overwritten_in = |version: &UnitBytesOfAVersion| -> u64 {
        version
            .iter()
            .filter(|((device, slot, span), bytes)| {
                let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
                end.read(*device, slot * SLOT_BYTES_LOCAL, length)
                    .unwrap_or_default()
                    != **bytes
            })
            .count() as u64
    };
    // Q1 前两栏比的是「被测那次挂载之前最后确认那一版」引用的单元（登记第六节 Q1）。
    let compared_version = &protected.unit_bytes;
    let overwritten = overwritten_in(compared_version);
    let protected_in_ring = end_roots.contains(&protected.root);
    let (in_ring_overwritten, out_of_ring_overwritten) = if protected_in_ring {
        (overwritten, 0)
    } else {
        (0, overwritten)
    };
    let landing_overwritten = match (&landing_unit_bytes, read_back_root) {
        (Ok(version), Some(_)) if landing_in_base => overwritten_in(version).to_string(),
        (Err(error), Some(_)) => format!("unreadable_in_base:{}", error.replace(' ', "_")),
        (Ok(_), Some(_)) => "written_after_the_tested_mount".to_string(),
        (Ok(_) | Err(_), None) => "none".to_string(),
    };
    let tail_class = match (first_instance_at_or_after_the_tested_mount, geometry) {
        (Some(first_instance), Ok(geometry)) => {
            let slots = checker_image::root_slot_positions(&geometry);
            let faults: Vec<ThirdRunFault> = checker_image::valid_roots(end, &geometry)
                .into_iter()
                .filter(|(_, _, view)| view.instance >= first_instance)
                .filter_map(|(region, slot, view)| {
                    slots
                        .iter()
                        .find(|(candidate_region, candidate_slot, _, _)| {
                            *candidate_region == region && *candidate_slot == slot
                        })
                        .map(|(_, _, device, offset)| ThirdRunFault {
                            device: DeviceIdentity(*device),
                            offset: *offset,
                            length: u64::from(geometry.physical_block_size),
                            form: UnreadableForm::ReadFails,
                            duration: FaultDuration::WholeCall,
                            label: format!(
                                "tail_root{}",
                                key_text(Some((view.instance, view.checkpoint_txg)))
                            ),
                        })
                })
                .collect();
            let tail = run_third_run_recover(end, &faults);
            match &tail.outcome.outcome {
                RecoveryOutcome::Failed { failure, .. } => {
                    format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
                }
                outcome @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::FileRead { .. }) => {
                    let landing = chosen_root_of(outcome);
                    let expected = landing.and_then(|root| history.content_number_of(root));
                    let got = match outcome {
                        RecoveryOutcome::FileRead { content, .. } => {
                            content_number_of_bytes(content, highest)
                        }
                        RecoveryOutcome::NoFile { .. } => Some(0),
                        RecoveryOutcome::Failed { .. } => None,
                    };
                    if expected.is_some() && got == expected {
                        format!("own_version:{}", key_text(landing))
                    } else {
                        format!(
                            "other:{}:expected={expected:?}:got={got:?}",
                            key_text(landing)
                        )
                    }
                }
            }
        }
        (Some(_), Err(_)) => "failed:geometry".to_string(),
        (None, _) => "na".to_string(),
    };
    CellLoss {
        in_ring_overwritten,
        out_of_ring_overwritten,
        landing_overwritten,
        read_back,
        read_back_root,
        tail_class,
    }
}

/// 每臂每族的汇总（第六节各量的分母与分子都报整数）。
#[derive(Default)]
struct ThirdRunFamilySummary {
    cells: u64,
    void_cells: u64,
    fault_not_reached_cells: u64,
    overwrite_cells: u64,
    in_ring_overwrite_cells: u64,
    out_of_ring_overwrite_cells: u64,
    landing_overwrite_cells: u64,
    rollback_cells: u64,
    tested_ok_cells: u64,
    read_back: BTreeMap<String, u64>,
    read_back_last_confirmed_cells: u64,
    read_back_lost_cells: u64,
    tail: BTreeMap<String, u64>,
    tail_lost_cells: u64,
    lost_write_cells: u64,
    tested_classes: BTreeMap<String, u64>,
    remount_classes: BTreeMap<String, u64>,
    abandoned_hidden_root_cells: u64,
    /// 停机 S4：装置与 `crates/` 对不上的格（7.3 第一行）。
    stop_condition_four_mismatches: Vec<String>,
    /// 失败条款 F4：点名单元只有第一份读不出时施加了那次发布。
    failure_clause_four_findings: Vec<String>,
    /// 失败条款 F13：新实例第一次发布的 txg 与今天的规则不等。
    failure_clause_thirteen_findings: Vec<String>,
    failure_clause_thirteen_checks: u64,
    formula_mismatches: Vec<String>,
}

impl ThirdRunFamilySummary {
    fn count(&mut self, loss: &CellLoss, rollback: bool, tested_ok: bool) {
        self.cells += 1;
        if loss.overwrite_cell() {
            self.overwrite_cells += 1;
        }
        if loss.in_ring_overwritten > 0 {
            self.in_ring_overwrite_cells += 1;
        }
        if loss.out_of_ring_overwritten > 0 {
            self.out_of_ring_overwrite_cells += 1;
        }
        if loss
            .landing_overwritten
            .parse::<u64>()
            .is_ok_and(|count| count > 0)
        {
            self.landing_overwrite_cells += 1;
        }
        if rollback {
            self.rollback_cells += 1;
        }
        if tested_ok {
            self.tested_ok_cells += 1;
        }
        *self
            .read_back
            .entry(loss.read_back.split(':').next().unwrap_or("").to_string())
            .or_insert(0) += 1;
        if loss.read_back == "last_confirmed" {
            self.read_back_last_confirmed_cells += 1;
        }
        if loss.read_back_lost() {
            self.read_back_lost_cells += 1;
        }
        *self
            .tail
            .entry(loss.tail_class.split(':').next().unwrap_or("").to_string())
            .or_insert(0) += 1;
        if loss.tail_lost() {
            self.tail_lost_cells += 1;
        }
        if loss.lost_write_cell() || rollback {
            self.lost_write_cells += 1;
        }
    }

    fn emit(&self, family: &str, arm: &str) {
        emit_result(&format!(
            "name=r3_family_summary family={family} arm={arm} cells={} void_cells_v2={} fault_not_reached_cells={} q1_overwrite_cells={} q1_in_ring_overwrite_cells={} q1_out_of_ring_overwrite_cells={} q1_landing_overwrite_cells={} q2_rollback_cells={} tested_ok_cells={} q3_read_back={:?} q3_last_confirmed_cells={} q3_lost_cells={} q3c_tail={:?} q3c_lost_cells={} lost_write_cells_q1_q2_q3={} tested_classes={:?} remount_classes={:?} q0_hidden_root_abandoned_cells={} s4_mismatches={} f4_failures={} f13_checked={} f13_failures={} formula_mismatches={}",
            self.cells,
            self.void_cells,
            self.fault_not_reached_cells,
            self.overwrite_cells,
            self.in_ring_overwrite_cells,
            self.out_of_ring_overwrite_cells,
            self.landing_overwrite_cells,
            self.rollback_cells,
            self.tested_ok_cells,
            self.read_back,
            self.read_back_last_confirmed_cells,
            self.read_back_lost_cells,
            self.tail,
            self.tail_lost_cells,
            self.lost_write_cells,
            self.tested_classes,
            self.remount_classes,
            self.abandoned_hidden_root_cells,
            self.stop_condition_four_mismatches.len(),
            self.failure_clause_four_findings.len(),
            self.failure_clause_thirteen_checks,
            self.failure_clause_thirteen_findings.len(),
            self.formula_mismatches.len()
        ));
        for (kind, list) in [
            ("s4", &self.stop_condition_four_mismatches),
            ("f4", &self.failure_clause_four_findings),
            ("f13", &self.failure_clause_thirteen_findings),
            ("formula", &self.formula_mismatches),
        ] {
            for detail in list {
                emit_result(&format!(
                    "name=r3_family_finding family={family} arm={arm} kind={kind} detail={:?}",
                    detail
                ));
            }
        }
    }
}

/// 7.1 第四行（F13）今天那一臂的期望值：新实例第一次发布的 txg = max(这次调用里读得出的根的 txg, 读得出的自证记录的 checkpoint_txg) + 1。
/// 根环在这次调用里被读两遍才走到这一步（择根一遍、`highest_root_txg` 一遍），记录只在扫描那一遍读：
/// 根槽在第二次读还坏的不算，记录在第一次读就坏的不算（今天那一臂从不等，T 与 W 同）。
fn first_txg_expected_by_the_rule_of_today(
    twin: &MemoryPool,
    geometry: &PoolGeometry,
    faults: &[ThirdRunFault],
) -> u64 {
    let second_read_faults: Vec<ThirdRunFault> = faults
        .iter()
        .filter(|fault| fault.duration.is_active(2, 0))
        .cloned()
        .map(|mut fault| {
            fault.duration = FaultDuration::WholeCall;
            fault
        })
        .collect();
    let roots = device_decode(
        &FirstPassReader {
            pool: twin,
            faults: &second_read_faults,
        },
        geometry,
    );
    let records = device_decode(&FirstPassReader { pool: twin, faults }, geometry);
    let highest_root = roots
        .root_keys
        .iter()
        .map(|(_, txg)| *txg)
        .max()
        .unwrap_or(0);
    let highest_record = records
        .records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(0);
    1 + highest_root.max(highest_record)
}

/// 被测那次挂载的「之前与之后」对拍（7.3 第一行，今天那一臂）：装置择的根、重放出的 E、系统配置槽解码、被抛弃判定与 `crates/` 的比。
fn today_cross_checks(
    base: &MemoryPool,
    judgement: &DeviceJudgement,
    tested: &ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    label: &str,
    summary: &mut ThirdRunFamilySummary,
) {
    if let Some(error) = &judgement.error {
        summary
            .stop_condition_four_mismatches
            .push(format!("{label}: 装置重放报错 {error}"));
    }
    if let Ok(mounted) = &tested.outcome {
        let crates_chosen = root_pair(&mounted.output.chosen_root);
        let crates_effective = root_pair(&mounted.output.effective_root);
        if judgement.chosen != Some(crates_chosen) || judgement.effective != Some(crates_effective)
        {
            summary.stop_condition_four_mismatches.push(format!(
                "{label}: 择根 / E 装置 {} / {}，crates {} / {}",
                key_text(judgement.chosen),
                key_text(judgement.effective),
                key_text(Some(crates_chosen)),
                key_text(Some(crates_effective))
            ));
        }
    }
    let devices = devices_from_pool(base, IMAGE_BYTES);
    let Ok(geometry) = independent_geometry(base) else {
        summary
            .stop_condition_four_mismatches
            .push(format!("{label}: 几何解不出"));
        return;
    };
    let decoded = device_decode(base, &geometry);
    for device in [0u32, 1] {
        let mut by_crates: Vec<(u64, u64, u32)> =
            singlefs_core::recovery::verified_system_configuration_slots(
                &devices,
                DeviceIdentity(device),
                geometry.slot_spacing,
                &FILESYSTEM_IDENTIFIER,
            )
            .into_iter()
            .map(|slot| {
                (
                    slot.quantities.slot_generation,
                    slot.quantities.journal_tail,
                    slot.quantities.journal_instance.0,
                )
            })
            .collect();
        let mut by_the_device: Vec<(u64, u64, u32)> = decoded
            .system_configuration_slots
            .iter()
            .filter(|(slot_device, _, _, _)| *slot_device == device)
            .map(|(_, generation, tail, instance)| (*generation, *tail, *instance))
            .collect();
        by_crates.sort_unstable();
        by_the_device.sort_unstable();
        if by_crates != by_the_device {
            summary.stop_condition_four_mismatches.push(format!(
                "{label}: 盘 {device} 系统配置槽 装置 {by_the_device:?}，crates {by_crates:?}"
            ));
        }
    }
    if tested.outcome.is_ok() {
        if let Err(error) = abandoned_roots_on(&tested.pool_after) {
            if error.starts_with("S5") {
                summary
                    .stop_condition_four_mismatches
                    .push(format!("{label}: 调用之后 {error}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 11.5 H1d（登记 5.3 第一行）：P(n1, C) → A → 断在「这次挂载写行根落盘之前」的某一点 → 撤故障 → `mount_writable`（不注入）→
//      冷启动 `recover` 读回；断点在这一臂自己那次调用的写序列上取。
// ---------------------------------------------------------------------------

/// 这一臂在 H1d / PC 上跑哪几种时长（登记 5.3：各臂 W 与 O(1 只)，乙 三臂另加 T(δ)、T(2δ)；今天、甲、丙 另跑 T(δ)
/// 核 7.3 第三行「T(τ) 对它们与 W 逐格相同」，只跑不断那一格）。交回 (时长, 只跑不断那一格)。
fn held_once_durations_of(arm: &str) -> Vec<(FaultDuration, bool)> {
    let mut durations = vec![
        (FaultDuration::WholeCall, false),
        (FaultDuration::OnlyTheNthRead(1), false),
    ];
    match ThirdRunCandidate::of(arm) {
        Some(ThirdRunCandidate::Yi) => {
            durations.push((
                FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
                false,
            ));
            durations.push((
                FaultDuration::UntilVirtualTime(2 * THIRD_RUN_VIRTUAL_INTERVAL),
                false,
            ));
        }
        Some(ThirdRunCandidate::Today | ThirdRunCandidate::Jia | ThirdRunCandidate::Bing)
        | None => {
            durations.push((
                FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
                true,
            ));
        }
    }
    durations
}

/// 一次被测挂载（A）在一份起始镜像上跑完、交回它与装置的判定；今天那一臂顺带做 7.3 第一行、F13。
struct TestedMount {
    call: ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    summary: TestedMountSummary,
    judgement: DeviceJudgement,
    unintercepted: Vec<String>,
}

fn run_tested_mount(
    arm: &str,
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    label: &str,
    family_summary: &mut ThirdRunFamilySummary,
) -> Result<TestedMount, String> {
    let geometry = independent_geometry(base)?;
    let call = run_third_run_mount_writable(base, parameters, faults, None, None);
    let judgement = device_judgement(base, &geometry, faults);
    let summary = TestedMountSummary::of(&call);
    let unintercepted = call.unintercepted(faults);
    if arm == "today" {
        today_cross_checks(base, &judgement, &call, label, family_summary);
        if let Ok(mounted) = &call.outcome {
            family_summary.failure_clause_thirteen_checks += 1;
            let expected = first_txg_expected_by_the_rule_of_today(base, &geometry, faults);
            let got = mounted.output.row_publish.root().checkpoint_txg.0;
            if got != expected {
                family_summary
                    .failure_clause_thirteen_findings
                    .push(format!(
                        "{label}: 写行那次发布 txg {got}，按今天的规则 {expected}"
                    ));
            }
        }
    }
    Ok(TestedMount {
        call,
        summary,
        judgement,
        unintercepted,
    })
}

/// Q0：被测那次挂载 `Ok` 之后撤故障，被藏那条根还在不在根环里、装置按新实例表判它被不被抛弃（没 `Ok` 的两样都记 none）。
fn hidden_root_after_the_tested_mount(
    call: &ThirdRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    hidden: TimelineRoot,
) -> (Option<bool>, Option<bool>) {
    if call.outcome.is_err() {
        return (None, None);
    }
    let in_ring = independent_geometry(&call.pool_after)
        .ok()
        .map(|geometry| readable_roots_independent(&call.pool_after, &geometry).contains(&hidden));
    let abandoned = abandoned_roots_on(&call.pool_after)
        .ok()
        .map(|abandoned| abandoned.contains(&hidden));
    (in_ring, abandoned)
}

fn optional_flag_text(flag: Option<bool>) -> String {
    flag.map_or("none".to_string(), |value| value.to_string())
}

fn render_device_judgement(judgement: &DeviceJudgement) -> String {
    format!(
        "device_chosen={} device_effective={} device_n_cfg={} device_c_witness={} device_c_e={} device_undecidable={} device_n_slot={} device_unreadable_root_slots={} truth_newer={} truth_orders_disagree={} truth_by_records_only={}",
        key_text(judgement.chosen),
        key_text(judgement.effective),
        judgement.configuration_newer,
        judgement.witnessed_tail,
        judgement
            .selected_version_last_counter
            .map_or("none".to_string(), |counter| counter.to_string()),
        judgement.undecidable,
        judgement.unreadable_root_slots > 0,
        judgement.unreadable_root_slots,
        judgement.truth_newer,
        judgement.truth_orders_disagree,
        judgement.truth_by_records_only
    )
}

/// 一格：A 之后（断点那一刻）的镜像 `post` → 撤故障 → 再可写挂载（这一臂的代码，不注入）→ 历史末尾算三样。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的全部坐标与上下文原样打进结果行，拆成结构体只多一层搬运"
)]
fn evaluate_held_once_cell_of_the_third_run(
    arm: &str,
    coordinates: &str,
    history: &ThirdRunHistory,
    protected: &ThirdRunConfirmation,
    base: &MemoryPool,
    post: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    tested_line: &str,
    rollback: bool,
    tested_ok: bool,
    do_the_tail: bool,
    summary: &mut ThirdRunFamilySummary,
) {
    let remount = run_third_run_mount_writable(post, parameters, &[], None, None);
    let remount_class = third_run_outcome_class(&remount.outcome, remount.write_calls);
    let remount_member = mount_error_member_or_none(&remount.outcome);
    let first_new_instance = history.tip().0 + 1;
    let loss = evaluate_cell_loss(
        history,
        protected,
        base,
        &remount.pool_after,
        do_the_tail.then_some(first_new_instance),
    );
    summary.count(&loss, rollback, tested_ok);
    *summary
        .remount_classes
        .entry(remount_class.to_string())
        .or_insert(0) += 1;
    emit_result(&format!(
        "name=r3_h1d_cell arm={arm} {coordinates} {tested_line} q2_rollback={rollback} remount_class={remount_class} remount_error={remount_member} remount_chosen={} {}",
        key_text(
            remount
                .outcome
                .as_ref()
                .ok()
                .map(|mounted| root_pair(&mounted.output.chosen_root))
        ),
        loss.render()
    ));
}

fn run_third_run_held_once_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut summary = ThirdRunFamilySummary::default();
    for overwrites in THIRD_RUN_OVERWRITES {
        let mut history = match third_run_history_start(geometry, overwrites) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r3_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
        let base = history.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            emit_result(&format!(
                "name=r3_h1d_history arm={arm} n1={overwrites} verdict=stop reason=\"几何解不出\""
            ));
            continue;
        };
        let root_slots = root_slot_device_offsets(&pool_geometry);
        let protected = history
            .confirmations
            .last()
            .cloned()
            .expect("P(n1, C) 至少确认过首个文件");
        let last_confirmed = protected.content_number;
        let hidden = history.tip();
        emit_result(&format!(
            "name=r3_h1d_history arm={arm} n1={overwrites} tip={} last_confirmed={last_confirmed} tail_checks={} tail_check_failures={}",
            key_text(Some(hidden)),
            history.tail_checks,
            history.tail_check_failures.len()
        ));
        for failure in &history.tail_check_failures {
            emit_result(&format!(
                "name=r3_f3_tail_check_failure arm={arm} n1={overwrites} detail={failure:?}"
            ));
        }
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                for (duration, only_the_uncut_cell) in held_once_durations_of(arm) {
                    let head = format!(
                        "n1={overwrites} b={} form={} duration={}",
                        replay_break.name(),
                        form.name(),
                        duration.name()
                    );
                    let faults = match third_run_abandonment_faults(
                        &history,
                        &pool_geometry,
                        form,
                        replay_break,
                        duration,
                    ) {
                        Ok(Some(faults)) => faults,
                        Ok(None) => {
                            emit_result(&format!(
                                "name=r3_h1d_group arm={arm} {head} verdict=not_applicable"
                            ));
                            continue;
                        }
                        Err(error) => {
                            emit_result(&format!(
                                "name=r3_h1d_group arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let tested = match run_tested_mount(
                        arm,
                        &base,
                        &parameters,
                        &faults,
                        &head,
                        &mut summary,
                    ) {
                        Ok(tested) => tested,
                        Err(error) => {
                            emit_result(&format!(
                                "name=r3_h1d_group arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let first_root_write = index_of_the_first_root_slot_write(
                        &tested.call.applied,
                        &root_slots,
                        u64::from(pool_geometry.physical_block_size),
                    );
                    let barriers = first_root_write.map_or(0, |index| {
                        barriers_before_write(&tested.call.applied, index + 1)
                    });
                    let cut_cells_expected = if only_the_uncut_cell {
                        1
                    } else {
                        first_root_write.map_or(1, |index| 1 + 2 * (index as u64 + 1) + barriers)
                    };
                    let tested_ok = tested.call.outcome.is_ok();
                    let rollback = tested_ok
                        && tested
                            .summary
                            .effective
                            .and_then(|effective| history.content_number_of(effective))
                            .is_some_and(|number| number < last_confirmed);
                    *summary
                        .tested_classes
                        .entry(format!(
                            "{}:{}",
                            tested.summary.class, tested.summary.member
                        ))
                        .or_insert(0) += 1;
                    let (hidden_in_ring_after, hidden_abandoned_after) =
                        hidden_root_after_the_tested_mount(&tested.call, hidden);
                    if arm == "today" && hidden_abandoned_after == Some(true) {
                        summary.abandoned_hidden_root_cells += 1;
                    }
                    if arm == "today"
                        && tested_ok
                        && replay_break == ReplayBreak::FirstCopyOfTheFirstNamedUnit
                        && duration == FaultDuration::WholeCall
                        && tested.summary.effective != history.timeline.iter().rev().nth(1).copied()
                    {
                        summary.failure_clause_four_findings.push(format!(
                            "{head}: E {}，被藏那条根前一条 {}",
                            key_text(tested.summary.effective),
                            key_text(history.timeline.iter().rev().nth(1).copied())
                        ));
                    }
                    let not_reached = !tested.unintercepted.is_empty();
                    if not_reached {
                        if arm == "today" {
                            summary.void_cells += 1;
                        } else {
                            summary.fault_not_reached_cells += 1;
                        }
                    }
                    let tested_line = format!(
                        "{} {} hidden={} hidden_in_ring_after_a={} q0_hidden_abandoned_after_a={} faults={} unintercepted={} first_root_write={} barriers_before_the_row_root={}",
                        tested.summary.render("a"),
                        render_device_judgement(&tested.judgement),
                        key_text(Some(hidden)),
                        optional_flag_text(hidden_in_ring_after),
                        optional_flag_text(hidden_abandoned_after),
                        faults.len(),
                        tested.unintercepted.join(","),
                        first_root_write.map_or("none".to_string(), |index| index.to_string()),
                        barriers
                    );
                    if arm == "today" && not_reached {
                        emit_result(&format!(
                            "name=r3_h1d_cell arm={arm} {head} cut=none point=none verdict=void_v2 {tested_line}"
                        ));
                        continue;
                    }
                    let mut enumerated = 0u64;
                    evaluate_held_once_cell_of_the_third_run(
                        arm,
                        &format!("{head} cut=none point=none"),
                        &history,
                        &protected,
                        &base,
                        &tested.call.pool_after,
                        &parameters,
                        &tested_line,
                        rollback,
                        tested_ok,
                        true,
                        &mut summary,
                    );
                    enumerated += 1;
                    if !only_the_uncut_cell {
                        if let Some(first_root_write) = first_root_write {
                            for write_count in 0..=first_root_write {
                                let post = image_with_the_first_writes(
                                    &base,
                                    &tested.call.applied,
                                    write_count,
                                );
                                evaluate_held_once_cell_of_the_third_run(
                                    arm,
                                    &format!("{head} cut=crash point={write_count}"),
                                    &history,
                                    &protected,
                                    &base,
                                    &post,
                                    &parameters,
                                    "a_outcome=crashed",
                                    false,
                                    false,
                                    false,
                                    &mut summary,
                                );
                                enumerated += 1;
                            }
                            for failing_write in 0..=first_root_write {
                                let failed = run_third_run_mount_writable(
                                    &base,
                                    &parameters,
                                    &faults,
                                    Some(failing_write as u64),
                                    None,
                                );
                                let failed_summary = TestedMountSummary::of(&failed);
                                let failed_ok = failed.outcome.is_ok();
                                let failed_rollback = failed_ok
                                    && failed_summary
                                        .effective
                                        .and_then(|effective| history.content_number_of(effective))
                                        .is_some_and(|number| number < last_confirmed);
                                evaluate_held_once_cell_of_the_third_run(
                                    arm,
                                    &format!("{head} cut=write_fails point={failing_write}"),
                                    &history,
                                    &protected,
                                    &base,
                                    &failed.pool_after,
                                    &parameters,
                                    &format!(
                                        "{} fired={}",
                                        failed_summary.render("a"),
                                        failed.injected_failure_fired
                                    ),
                                    failed_rollback,
                                    failed_ok,
                                    false,
                                    &mut summary,
                                );
                                enumerated += 1;
                            }
                            for failing_barrier in 0..barriers {
                                let failed = run_third_run_mount_writable(
                                    &base,
                                    &parameters,
                                    &faults,
                                    None,
                                    Some(failing_barrier),
                                );
                                let failed_summary = TestedMountSummary::of(&failed);
                                let failed_ok = failed.outcome.is_ok();
                                let failed_rollback = failed_ok
                                    && failed_summary
                                        .effective
                                        .and_then(|effective| history.content_number_of(effective))
                                        .is_some_and(|number| number < last_confirmed);
                                evaluate_held_once_cell_of_the_third_run(
                                    arm,
                                    &format!("{head} cut=barrier_fails point={failing_barrier}"),
                                    &history,
                                    &protected,
                                    &base,
                                    &failed.pool_after,
                                    &parameters,
                                    &format!(
                                        "{} fired={}",
                                        failed_summary.render("a"),
                                        failed.injected_failure_fired
                                    ),
                                    failed_rollback,
                                    failed_ok,
                                    false,
                                    &mut summary,
                                );
                                enumerated += 1;
                            }
                        }
                    }
                    if enumerated != cut_cells_expected {
                        summary.formula_mismatches.push(format!(
                            "{head}: 枚举 {enumerated} 格，公式 1 + 2(J + 1) + 屏障数 = {cut_cells_expected}"
                        ));
                    }
                    emit_result(&format!(
                        "name=r3_h1d_group arm={arm} {head} cells={enumerated} formula_cells={cut_cells_expected} verdict={}",
                        if enumerated == cut_cells_expected { "pass" } else { "fail_v4" }
                    ));
                }
            }
        }
    }
    summary.emit("h1d", arm);
}

// ---------------------------------------------------------------------------
// 11.6 H1e（登记 5.3 第二行）：P(n1, C) → A（不断）→ 在 A 交回的会话里再做 m 次改内容 → 关闭 C → 撤故障 →
//      `mount_writable`（不注入）→ 冷启动 `recover` 读回。A 被拒的臂，那 m 次没有会话可做，记「因拒没做」。
// ---------------------------------------------------------------------------

fn tail_durations_of(arm: &str) -> Vec<FaultDuration> {
    match ThirdRunCandidate::of(arm) {
        Some(ThirdRunCandidate::Yi) => vec![
            FaultDuration::WholeCall,
            FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
        ],
        Some(ThirdRunCandidate::Today | ThirdRunCandidate::Jia | ThirdRunCandidate::Bing)
        | None => {
            vec![FaultDuration::WholeCall]
        }
    }
}

fn run_third_run_tail_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut summary = ThirdRunFamilySummary::default();
    let mut cells_by_duration: BTreeMap<String, u64> = BTreeMap::new();
    for overwrites in THIRD_RUN_OVERWRITES {
        let mut prefix = match third_run_history_start(geometry, overwrites) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r3_h1e_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        let _ = third_run_close(&mut prefix, &parameters, Closing::ProcessExit);
        let base = prefix.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            continue;
        };
        let protected = prefix
            .confirmations
            .last()
            .cloned()
            .expect("P(n1, C) 至少确认过首个文件");
        let hidden = prefix.tip();
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                for duration in tail_durations_of(arm) {
                    let head = format!(
                        "n1={overwrites} b={} form={} duration={}",
                        replay_break.name(),
                        form.name(),
                        duration.name()
                    );
                    let Ok(Some(faults)) = third_run_abandonment_faults(
                        &prefix,
                        &pool_geometry,
                        form,
                        replay_break,
                        duration,
                    ) else {
                        emit_result(&format!(
                            "name=r3_h1e_group arm={arm} {head} verdict=not_applicable"
                        ));
                        continue;
                    };
                    let tested = match run_tested_mount(
                        arm,
                        &base,
                        &parameters,
                        &faults,
                        &head,
                        &mut summary,
                    ) {
                        Ok(tested) => tested,
                        Err(error) => {
                            emit_result(&format!(
                                "name=r3_h1e_group arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let not_reached = !tested.unintercepted.is_empty();
                    let tested_ok = tested.call.outcome.is_ok();
                    let last_confirmed_before = protected.content_number;
                    let rollback = tested_ok
                        && tested
                            .summary
                            .effective
                            .and_then(|effective| prefix.content_number_of(effective))
                            .is_some_and(|number| number < last_confirmed_before);
                    let (hidden_in_ring_after, hidden_abandoned_after) =
                        hidden_root_after_the_tested_mount(&tested.call, hidden);
                    if arm == "today" && hidden_abandoned_after == Some(true) {
                        summary.abandoned_hidden_root_cells += 1;
                    }
                    let tested_line = format!(
                        "{} {} hidden={} hidden_in_ring_after_a={} q0_hidden_abandoned_after_a={} faults={} unintercepted={}",
                        tested.summary.render("a"),
                        render_device_judgement(&tested.judgement),
                        key_text(Some(hidden)),
                        optional_flag_text(hidden_in_ring_after),
                        optional_flag_text(hidden_abandoned_after),
                        faults.len(),
                        tested.unintercepted.join(",")
                    );
                    for tail_overwrites in THIRD_RUN_TAIL_OVERWRITES {
                        let coordinates = format!("{head} m={tail_overwrites}");
                        *cells_by_duration.entry(duration.name()).or_insert(0) += 1;
                        *summary
                            .tested_classes
                            .entry(format!(
                                "{}:{}",
                                tested.summary.class, tested.summary.member
                            ))
                            .or_insert(0) += 1;
                        if not_reached {
                            if arm == "today" {
                                summary.void_cells += 1;
                                emit_result(&format!(
                                    "name=r3_h1e_cell arm={arm} {coordinates} verdict=void_v2 {tested_line}"
                                ));
                                continue;
                            }
                            summary.fault_not_reached_cells += 1;
                        }
                        let mut history = prefix.clone();
                        history.pool = tested.call.pool_after.clone();
                        let tail_writes = match &tested.call.outcome {
                            Ok(mounted) => {
                                let effective_content = history
                                    .content_number_of(root_pair(&mounted.output.effective_root))
                                    .unwrap_or(0);
                                push_the_publishes_of_a_mount(
                                    &mut history,
                                    mounted,
                                    effective_content,
                                );
                                history.session = Some(Session {
                                    allocator: mounted.allocator.clone(),
                                    current: mounted.current.clone(),
                                    instance: mounted.output.instance,
                                });
                                let mut done = 0u64;
                                let mut failure = None;
                                for _ in 0..tail_overwrites {
                                    match third_run_change_content(&mut history, &parameters) {
                                        Ok(()) => done += 1,
                                        Err(error) => {
                                            failure = Some(error);
                                            break;
                                        }
                                    }
                                }
                                match failure {
                                    None => format!("done:{done}"),
                                    Some(error) => format!(
                                        "failed_after:{done}:{}",
                                        error_member_of_debug(&error)
                                    ),
                                }
                            }
                            Err(_) => "refused_not_done".to_string(),
                        };
                        let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
                        let remount = run_third_run_mount_writable(
                            &history.pool,
                            &parameters,
                            &[],
                            None,
                            None,
                        );
                        let remount_class =
                            third_run_outcome_class(&remount.outcome, remount.write_calls);
                        let first_new_instance = hidden.0 + 1;
                        let loss = evaluate_cell_loss(
                            &history,
                            &protected,
                            &base,
                            &remount.pool_after,
                            Some(first_new_instance),
                        );
                        summary.count(&loss, rollback, tested_ok);
                        *summary
                            .remount_classes
                            .entry(remount_class.to_string())
                            .or_insert(0) += 1;
                        emit_result(&format!(
                            "name=r3_h1e_cell arm={arm} {coordinates} {tested_line} q2_rollback={rollback} tail_writes={tail_writes} last_confirmed={} remount_class={remount_class} remount_error={} {}",
                            history.last_confirmed_content_number(),
                            mount_error_member_or_none(&remount.outcome),
                            loss.render()
                        ));
                    }
                }
            }
        }
    }
    for (duration, cells) in &cells_by_duration {
        let registered = 4 * 2 * 2 * 4;
        let passed = *cells == registered;
        if !passed {
            summary.formula_mismatches.push(format!(
                "H1e 时长 {duration}: {cells} 格，7.2 钉 {registered}"
            ));
        }
        emit_result(&format!(
            "name=r3_section_seven_two_anchor arm={arm} item=\"H1e 每种时长的格数 4×2×2×4\" duration={duration} computed={cells} registered={registered} verdict={}",
            if passed { "pass" } else { "fail_v4" }
        ));
    }
    summary.emit("h1e", arm);
}

// ---------------------------------------------------------------------------
// 11.7 阳性对照（登记 5.4，每一臂都跑）：PC-N、PC-O、PC-N0、PC-丢写、PC-只读；PC-多读在 `r3-compare` 里判。
//      判定词 pass / fail / not_constructible（登记 5.5 第 7 条）。
// ---------------------------------------------------------------------------

fn emit_third_run_positive_control(arm: &str, control: &str, verdict: &str, detail: &str) {
    emit_result(&format!(
        "name=r3_positive_control arm={arm} control={control} verdict={verdict} {detail}"
    ));
}

/// 阳性对照那几格顺带做的 7.3 第一行对拍与 F13（今天那一臂）打成结果行。
fn emit_scratch_findings(family: &str, arm: &str, scratch: &ThirdRunFamilySummary) {
    emit_result(&format!(
        "name=r3_positive_control_cross_checks family={family} arm={arm} s4_mismatches={} f13_checked={} f13_failures={}",
        scratch.stop_condition_four_mismatches.len(),
        scratch.failure_clause_thirteen_checks,
        scratch.failure_clause_thirteen_findings.len()
    ));
    for (kind, list) in [
        ("s4", &scratch.stop_condition_four_mismatches),
        ("f13", &scratch.failure_clause_thirteen_findings),
    ] {
        for detail in list {
            emit_result(&format!(
                "name=r3_family_finding family={family} arm={arm} kind={kind} detail={detail:?}"
            ));
        }
    }
}

fn verdict_word(passed: bool) -> &'static str {
    if passed {
        "pass"
    } else {
        "fail"
    }
}

/// PC-N（W；乙 另跑 T(δ)）、PC-丢写、PC-只读：P(1, C) → A（b = records、read_fails）。
fn run_third_run_positive_control_newer(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let candidate = ThirdRunCandidate::of(arm);
    let mut history = match third_run_history_start(geometry, 1) {
        Ok(history) => history,
        Err(error) => {
            emit_third_run_positive_control(
                arm,
                "PC-N",
                "not_constructible",
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
    let base = history.pool.clone();
    let Ok(pool_geometry) = independent_geometry(&base) else {
        emit_third_run_positive_control(arm, "PC-N", "not_constructible", "reason=\"几何解不出\"");
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let protected = history.confirmations.last().cloned().expect("确认过");
    // 乙 三臂另跑 T(δ)（登记 5.4 PC-N 行）；今天、甲、丙 也跑 T(δ)，核 7.3 第三行「T(τ) 对它们与 W 逐格相同」（在 PC-N 上抽跑）。
    for duration in [
        FaultDuration::WholeCall,
        FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
    ] {
        let Ok(Some(faults)) = third_run_abandonment_faults(
            &history,
            &pool_geometry,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
            duration,
        ) else {
            emit_third_run_positive_control(
                arm,
                "PC-N",
                "not_constructible",
                "reason=\"落点造不出\"",
            );
            continue;
        };
        let mut scratch = ThirdRunFamilySummary::default();
        let Ok(tested) = run_tested_mount(arm, &base, &parameters, &faults, "PC-N", &mut scratch)
        else {
            emit_third_run_positive_control(
                arm,
                "PC-N",
                "not_constructible",
                "reason=\"起始镜像几何解不出\"",
            );
            continue;
        };
        emit_scratch_findings("pc_n", arm, &scratch);
        let rollback = tested.call.outcome.is_ok()
            && tested
                .summary
                .effective
                .and_then(|effective| history.content_number_of(effective))
                .is_some_and(|number| number < protected.content_number);
        let abandoned_after_reading = tested
            .call
            .outcome
            .is_ok()
            .then(|| abandoned_roots_on(&tested.call.pool_after));
        let abandoned_after = abandoned_after_reading.as_ref().is_some_and(|reading| {
            reading
                .as_ref()
                .is_ok_and(|abandoned| abandoned.contains(&hidden))
        });
        let hidden_in_ring_after =
            independent_geometry(&tested.call.pool_after).is_ok_and(|geometry_after| {
                readable_roots_independent(&tested.call.pool_after, &geometry_after)
                    .contains(&hidden)
            });
        let abandoned_after_detail = match &abandoned_after_reading {
            None => "not_mounted".to_string(),
            Some(Ok(abandoned)) => abandoned
                .iter()
                .map(|root| key_text(Some(*root)))
                .collect::<Vec<_>>()
                .join(","),
            Some(Err(error)) => format!("error:{}", error.replace(' ', "_")),
        };
        let refused_with_zero_writes =
            tested.summary.member == third_run_refusal_member(arm) && tested.summary.writes == 0;
        let (expectation, passed) = match (candidate, duration) {
            (Some(ThirdRunCandidate::Today), _) => (
                "ok_k0_effective_before_hidden_abandoned_after_rollback_one",
                tested.summary.class == "K0"
                    && tested.summary.effective == before_hidden
                    && abandoned_after
                    && rollback,
            ),
            (Some(ThirdRunCandidate::Yi), FaultDuration::UntilVirtualTime(_)) => (
                "ok_after_one_round_chosen_hidden_no_rollback",
                tested.call.outcome.is_ok()
                    && tested.summary.chosen == Some(hidden)
                    && !rollback
                    && tested.summary.waits == THIRD_RUN_REREAD_ROUNDS,
            ),
            (Some(ThirdRunCandidate::Yi), _) => (
                "refused_after_r_rounds_k3",
                refused_with_zero_writes && tested.summary.waits == THIRD_RUN_REREAD_ROUNDS,
            ),
            (Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Bing), _) => {
                ("refused_k3", refused_with_zero_writes)
            }
            (None, _) => ("unknown_arm", false),
        };
        let passed = passed && tested.judgement.truth_newer && tested.unintercepted.is_empty();
        emit_third_run_positive_control(
            arm,
            "PC-N",
            verdict_word(passed),
            &format!(
                "duration={} faults={} expectation={expectation} truth_newer={} rollback={rollback} hidden={} hidden_in_ring_after={hidden_in_ring_after} hidden_abandoned_after={abandoned_after} abandoned_after={abandoned_after_detail} clause_ok={} clause_effective_before_hidden={} clause_rollback_one={} {} {} unintercepted={}",
                duration.name(),
                faults.len(),
                tested.judgement.truth_newer,
                key_text(Some(hidden)),
                tested.call.outcome.is_ok(),
                tested.summary.effective == before_hidden,
                rollback,
                tested.summary.render("a"),
                render_device_judgement(&tested.judgement),
                tested.unintercepted.join(",")
            ),
        );
        if duration != FaultDuration::WholeCall {
            continue;
        }
        // PC-只读：拒了的那一格，同一组故障下冷启动 `recover` 交得出结局、读回 E_故障 那一版。
        if tested.call.outcome.is_err() {
            let read_only = run_third_run_recover(&base, &faults);
            let expected = tested
                .judgement
                .effective
                .and_then(|effective| history.content_number_of(effective));
            let got = match &read_only.outcome.outcome {
                RecoveryOutcome::FileRead { content, .. } => {
                    content_number_of_bytes(content, history.last_confirmed_content_number())
                }
                RecoveryOutcome::NoFile { .. } => Some(0),
                RecoveryOutcome::Failed { .. } => None,
            };
            let read_only_passed = expected.is_some() && got == expected;
            emit_third_run_positive_control(
                arm,
                "PC-read-only",
                verdict_word(read_only_passed),
                &format!(
                    "expected_content={expected:?} got_content={got:?} landing={} e_under_faults={}",
                    key_text(chosen_root_of(&read_only.outcome.outcome)),
                    key_text(tested.judgement.effective)
                ),
            );
        } else {
            emit_third_run_positive_control(
                arm,
                "PC-read-only",
                "not_applicable",
                "reason=\"这一臂在 PC-N 上没拒\"",
            );
        }
        // PC-丢写：PC-N 那段历史走完之后、最后那次 `mount_writable` 之前，把最后确认那一版的数据单元两份都改写成别的字节。
        let (corrupted, loss) = positive_control_lost_write(
            &history,
            &protected,
            &base,
            &tested.call.pool_after,
            &parameters,
        );
        let lost_write_seen =
            corrupted > 0 && loss.overwrite_cell() && loss.read_back != "last_confirmed";
        emit_third_run_positive_control(
            arm,
            "PC-lost-write",
            if corrupted == 0 {
                "not_constructible"
            } else {
                verdict_word(lost_write_seen)
            },
            &format!("corrupted_copies={corrupted} {}", loss.render()),
        );
    }
}

/// PC-丢写（登记 5.4）：被测那次挂载之后（`after_the_tested_mount`）、最后那次 `mount_writable` 之前，
/// 把「被测那次挂载之前最后确认那一版」的数据单元（类标签 数据 / 打包）每一份都改写成 0xA5，再可写挂载、在历史末尾算三样。
/// 交回 (改写了几份, 那一格的三样)。
fn positive_control_lost_write(
    history: &ThirdRunHistory,
    protected: &ThirdRunConfirmation,
    base: &MemoryPool,
    after_the_tested_mount: &MemoryPool,
    parameters: &MakeFilesystemParameters,
) -> (u64, CellLoss) {
    let mut devices = devices_from_pool(after_the_tested_mount, IMAGE_BYTES);
    let mut corrupted = 0u64;
    for ((device, slot, span), bytes) in &protected.unit_bytes {
        let is_data_unit = singlefs_checker::check_unit(bytes)
            .is_ok_and(|class| class == UNIT_CLASS_DATA_LOCAL || class == UNIT_CLASS_PACKED_LOCAL);
        if !is_data_unit {
            continue;
        }
        let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
        if let Some((_, target)) = devices
            .iter_mut()
            .find(|(identity, _)| identity.0 == *device)
        {
            target
                .write_at(
                    DeviceOffsetInBytes(slot * SLOT_BYTES_LOCAL),
                    &vec![0xA5; length],
                    WriteDurability::Plain,
                )
                .expect("内存盘写不报错");
            corrupted += 1;
        }
    }
    let post = memory_pool_of(&devices, IMAGE_BYTES);
    let remount = run_third_run_mount_writable(&post, parameters, &[], None, None);
    let loss = evaluate_cell_loss(history, protected, base, &remount.pool_after, None);
    (corrupted, loss)
}

/// PC-O：P(1, C) → A（b = unit_first_copy、read_fails），时长 O(1 只)。
fn run_third_run_positive_control_second_read(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let candidate = ThirdRunCandidate::of(arm);
    let mut history = match third_run_history_start(geometry, 1) {
        Ok(history) => history,
        Err(error) => {
            emit_third_run_positive_control(
                arm,
                "PC-O",
                "not_constructible",
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
    let base = history.pool.clone();
    let Ok(pool_geometry) = independent_geometry(&base) else {
        emit_third_run_positive_control(arm, "PC-O", "not_constructible", "reason=\"几何解不出\"");
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let Ok(Some(faults)) = third_run_abandonment_faults(
        &history,
        &pool_geometry,
        UnreadableForm::ReadFails,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        FaultDuration::OnlyTheNthRead(1),
    ) else {
        emit_third_run_positive_control(arm, "PC-O", "not_constructible", "reason=\"落点造不出\"");
        return;
    };
    let mut scratch = ThirdRunFamilySummary::default();
    let Ok(tested) = run_tested_mount(arm, &base, &parameters, &faults, "PC-O", &mut scratch)
    else {
        emit_third_run_positive_control(
            arm,
            "PC-O",
            "not_constructible",
            "reason=\"起始镜像几何解不出\"",
        );
        return;
    };
    emit_scratch_findings("pc_o", arm, &scratch);
    let ok = tested.call.outcome.is_ok();
    let (expectation, passed) = match candidate {
        Some(ThirdRunCandidate::Today) => (
            "ok_effective_before_hidden",
            ok && tested.summary.effective == before_hidden,
        ),
        Some(ThirdRunCandidate::Jia) => (
            "refused_k3",
            tested.summary.member == third_run_refusal_member(arm) && tested.summary.writes == 0,
        ),
        Some(ThirdRunCandidate::Yi) => (
            "ok_after_one_round_chosen_hidden",
            ok && tested.summary.chosen == Some(hidden) && tested.summary.waits == 1,
        ),
        Some(ThirdRunCandidate::Bing) => (
            "ok_effective_hidden",
            ok && tested.summary.effective == Some(hidden),
        ),
        None => ("unknown_arm", false),
    };
    let passed = passed && tested.unintercepted.is_empty();
    emit_third_run_positive_control(
        arm,
        "PC-O",
        verdict_word(passed),
        &format!(
            "expectation={expectation} faults={} {} {} unintercepted={}",
            faults.len(),
            tested.summary.render("a"),
            render_device_judgement(&tested.judgement),
            tested.unintercepted.join(",")
        ),
    );
}

/// 这次调用写的结构种类与字节的指纹（PC-N0 跨臂比）：取号那几个系统配置槽写把 tail 与整槽校验和那两段抹成 0
/// （-配置续 那 8 个字节按臂的定义与今天不同，另核它 = 装置算的 c_见证）。交回 (指纹, 取号写里解出的 tail 列表)。
fn write_fingerprint_of(
    applied: &[AppliedDeviceStep],
    acquisition_writes: usize,
) -> (u64, Vec<u64>) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    let mut acquisition_tails = Vec::new();
    let mut write_index = 0usize;
    for step in applied {
        match step {
            AppliedDeviceStep::Write {
                device,
                offset,
                bytes,
            } => {
                mix(b"W");
                mix(&device.0.to_le_bytes());
                mix(&offset.to_le_bytes());
                if write_index < acquisition_writes
                    && bytes.len() >= SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8
                {
                    let mut masked = bytes.clone();
                    let mut tail = [0u8; 8];
                    tail.copy_from_slice(
                        &masked[SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL
                            ..SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8],
                    );
                    acquisition_tails.push(u64::from_le_bytes(tail));
                    masked[SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL
                        ..SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8]
                        .fill(0);
                    masked[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL
                        ..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL + WIDE_CHECKSUM_BYTES_LOCAL]
                        .fill(0);
                    mix(&masked);
                } else {
                    mix(bytes);
                }
                write_index += 1;
            }
            AppliedDeviceStep::ZeroFill {
                device,
                offset,
                length,
            } => {
                mix(b"Z");
                mix(&device.0.to_le_bytes());
                mix(&offset.to_le_bytes());
                mix(&length.to_le_bytes());
                write_index += 1;
            }
            AppliedDeviceStep::Barrier { device } => {
                mix(b"B");
                mix(&device.0.to_le_bytes());
            }
        }
    }
    (hash, acquisition_tails)
}

/// PC-N0（登记 5.4）：L0 全部 8 格（P(n1, c)，n1 ∈ {0..3}，c ∈ {C, U}）→ 被测挂载，不注入。每臂要 `Ok`；
/// 与今天逐项相同（K 类、所选根、E、写的结构种类与字节）由 `r3-compare` 判，这里报各项与指纹；-配置续 另核取号写的 tail = c_见证。
fn run_third_run_positive_control_no_newer(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut cells = 0u64;
    for overwrites in THIRD_RUN_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let coordinates = format!("n1={overwrites} c={}", closing.name());
            let mut history = match third_run_history_start(geometry, overwrites) {
                Ok(history) => history,
                Err(error) => {
                    emit_third_run_positive_control(
                        arm,
                        "PC-N0",
                        "not_constructible",
                        &format!("{coordinates} reason={error:?}"),
                    );
                    continue;
                }
            };
            let unmount_check = match third_run_close(&mut history, &parameters, closing) {
                Ok(check) => check,
                Err(error) => {
                    emit_third_run_positive_control(
                        arm,
                        "PC-N0",
                        "not_constructible",
                        &format!("{coordinates} reason={error:?}"),
                    );
                    continue;
                }
            };
            if let Some((holds, detail)) = &unmount_check {
                emit_result(&format!(
                    "name=r3_section_seven_one_f14 arm={arm} {coordinates} verdict={} {detail}",
                    if *holds { "pass" } else { "fail_f14" }
                ));
            }
            let base = history.pool.clone();
            let mut scratch = ThirdRunFamilySummary::default();
            let Ok(tested) =
                run_tested_mount(arm, &base, &parameters, &[], &coordinates, &mut scratch)
            else {
                emit_third_run_positive_control(
                    arm,
                    "PC-N0",
                    "not_constructible",
                    &format!("{coordinates} reason=\"几何解不出\""),
                );
                continue;
            };
            cells += 1;
            let (fingerprint, acquisition_tails) = write_fingerprint_of(&tested.call.applied, 2);
            let tail_expected = if arm.ends_with("-carry") {
                tested.judgement.witnessed_tail
            } else {
                0
            };
            let tails_hold = acquisition_tails.len() == 2
                && acquisition_tails.iter().all(|tail| *tail == tail_expected);
            let passed =
                tested.summary.class == "K0" && tails_hold && !tested.judgement.truth_newer;
            emit_third_run_positive_control(
                arm,
                "PC-N0",
                verdict_word(passed),
                &format!(
                    "{coordinates} write_fingerprint={fingerprint:016x} acquisition_tails={} acquisition_tail_expected={tail_expected} {} {} s4_mismatches={}",
                    acquisition_tails
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    tested.summary.render("a"),
                    render_device_judgement(&tested.judgement),
                    scratch.stop_condition_four_mismatches.len()
                ),
            );
            emit_scratch_findings("pc_n0", arm, &scratch);
        }
    }
    let registered = 4 * 2;
    emit_result(&format!(
        "name=r3_section_seven_two_anchor arm={arm} item=\"L0 格数 4×2\" computed={cells} registered={registered} verdict={}",
        if cells == registered { "pass" } else { "fail_v4" }
    ));
}

// ---------------------------------------------------------------------------
// 11.8 开跑检查：5.6 常量回比（V3）、7.2 锚点（V4）、臂与副本对得上（装置编进的臂代码与环境变量说的臂一致）。
// ---------------------------------------------------------------------------

fn third_run_local_constants_checks() -> Vec<ConstantCheck> {
    let mut checks = second_run_local_constants_checks();
    checks.extend([
        ConstantCheck {
            name: "单元类标签 数据 UNIT_CLASS_DATA（装置重放取单元宽）",
            local_value: u64::from(UNIT_CLASS_DATA_LOCAL),
            crates_value: u64::from(singlefs_core::unit::UNIT_CLASS_DATA),
        },
        ConstantCheck {
            name: "单元类标签 索引节点 UNIT_CLASS_INDEX_NODE",
            local_value: u64::from(UNIT_CLASS_INDEX_NODE_LOCAL),
            crates_value: u64::from(singlefs_core::unit::UNIT_CLASS_INDEX_NODE),
        },
        ConstantCheck {
            name: "单元类标签 打包 UNIT_CLASS_PACKED",
            local_value: u64::from(UNIT_CLASS_PACKED_LOCAL),
            crates_value: u64::from(singlefs_core::unit::UNIT_CLASS_PACKED),
        },
        ConstantCheck {
            name: "数据单元宽（装置重放用的那一个）DATA_UNIT_BYTES",
            local_value: DATA_UNIT_BYTES_LOCAL as u64,
            crates_value: singlefs_format::DATA_UNIT_BYTES,
        },
        ConstantCheck {
            name: "节点宽（装置重放用的那一个）NODE_BYTES",
            local_value: NODE_BYTES_LOCAL as u64,
            crates_value: singlefs_format::NODE_BYTES,
        },
        ConstantCheck {
            name: "系统配置整槽校验和偏移（PC-N0 指纹抹掉的那一段）",
            local_value: SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL as u64,
            crates_value: u64::try_from(
                singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            )
            .expect("155"),
        },
    ]);
    checks
}

fn run_third_run_constants_and_anchors(arm: &str) -> bool {
    let mut all_passed = true;
    for check in third_run_local_constants_checks() {
        let passed = check.matches();
        all_passed &= passed;
        emit_result(&format!(
            "name=r3_local_constant_check arm={arm} item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            verdict_word(passed),
            check.local_value,
            check.crates_value
        ));
    }
    let (regions_match, local_regions, crates_regions) = region_devices_matches_crates();
    all_passed &= regions_match;
    emit_result(&format!(
        "name=r3_local_constant_check arm={arm} item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        verdict_word(regions_match)
    ));
    let arm_known = THIRD_RUN_ARMS.contains(&arm);
    let arm_code_matches =
        arm_known && (arm == "today") != third_run_arm_hooks::arm_code_is_compiled_in();
    all_passed &= arm_code_matches;
    emit_result(&format!(
        "name=r3_arm_code_check arm={arm} verdict={} arm_known={arm_known} arm_code_compiled_in={}",
        verdict_word(arm_code_matches),
        third_run_arm_hooks::arm_code_is_compiled_in()
    ));
    match decoded_tail_instance_and_floor_offsets() {
        Ok((tail, instance, floor)) => {
            let passed = tail == SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL
                && instance == tail + 8
                && floor == SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8 + 4
                && instance == 477
                && floor == 481;
            all_passed &= passed;
            emit_result(&format!(
                "name=r3_section_seven_two_anchor arm={arm} item=\"journal 实例代号偏移 469 + 8；F 偏移 489 − 8\" verdict={} tail_offset={tail} instance_offset={instance} floor_offset={floor} registered_instance=477 registered_floor=481",
                verdict_word(passed)
            ));
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r3_section_seven_two_anchor arm={arm} item=offsets verdict=fail reason={error:?}"
            ));
        }
    }
    // 7.2：根槽总数 3 × S 与环槽数（按装置在 G0 上现枚举的数），系统配置槽数 2 × 2。
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut devices = new_devices(IMAGE_BYTES);
    let geometry_anchor = make_filesystem(&parameters, &mut devices)
        .map_err(|error| format!("{error:?}"))
        .and_then(|_| independent_geometry(&memory_pool_of(&devices, IMAGE_BYTES)));
    match geometry_anchor {
        Ok(geometry) => {
            let pool = memory_pool_of(&devices, IMAGE_BYTES);
            let root_slots = checker_image::root_slot_positions(&geometry).len() as u64;
            let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
            let configuration_slots = device_decode(&pool, &geometry)
                .system_configuration_slots
                .len() as u64;
            for (item, computed, registered) in [
                (
                    "根槽总数 3×S，S=8（装置在 G0 上枚举的槽数）",
                    root_slots,
                    24u64,
                ),
                ("环槽数 3MiB÷4096（装置在 G0 上的环槽数）", ring_slots, 768),
                (
                    "每个池的系统配置槽数 2×2（装置读到的自证槽数）",
                    configuration_slots,
                    4,
                ),
            ] {
                let passed = computed == registered;
                all_passed &= passed;
                emit_result(&format!(
                    "name=r3_section_seven_two_anchor arm={arm} item={item:?} verdict={} computed={computed} registered={registered}",
                    verdict_word(passed)
                ));
            }
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r3_section_seven_two_anchor arm={arm} item=geometry verdict=fail reason={error:?}"
            ));
        }
    }
    all_passed
}

// ---------------------------------------------------------------------------
// 11.9 跨臂对拍（`r3-compare`）：读九条臂各自的产物，算 Q5（多读）、PC-多读、PC-N0 的逐项相同（F12）、
//      7.3 第二行（装置在今天那一臂上算的 N 与甲臂的拒一一对应，S4）、第三行（T(δ) 对今天、甲、丙与 W 逐格相同，V5）、
//      第四行（今天减自己 = 0）、F11（丙-配置 与 甲-配置 只在 O(1 只) 那几格不同）。
// ---------------------------------------------------------------------------

/// 产物里一行 `E7RESULT name=… k=v …`：按空格切，值里不带空格的那几个字段取得出（带引号的说明字段不取）。
fn fields_of_a_result_line(line: &str) -> BTreeMap<String, String> {
    line.trim_start_matches("E7RESULT ")
        .split(' ')
        .filter_map(|token| token.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// 一格的键：族与坐标（不含臂）。
fn cell_key_of(fields: &BTreeMap<String, String>, coordinate_names: &[&str]) -> String {
    coordinate_names
        .iter()
        .map(|name| format!("{name}={}", fields.get(*name).map_or("", String::as_str)))
        .collect::<Vec<_>>()
        .join(" ")
}

struct ComparedCell {
    fields: BTreeMap<String, String>,
}

/// 从一条臂的产物里取 H1d 不断那一格、H1e（每个 (n1, b, 造法, 时长) 只取 m = 1 那一行，A 的读数与 m 无关）、PC-N、PC-O、PC-N0 的行，按格的键收。
fn compared_cells_of(text: &str) -> BTreeMap<String, ComparedCell> {
    let mut cells = BTreeMap::new();
    for line in text.lines() {
        let fields = fields_of_a_result_line(line);
        let name = fields.get("name").map_or("", String::as_str);
        let key = match name {
            "r3_h1d_cell" if fields.get("cut").map(String::as_str) == Some("none") => format!(
                "h1d {}",
                cell_key_of(&fields, &["n1", "b", "form", "duration"])
            ),
            "r3_h1e_cell" if fields.get("m").map(String::as_str) == Some("1") => format!(
                "h1e {}",
                cell_key_of(&fields, &["n1", "b", "form", "duration"])
            ),
            "r3_positive_control" => {
                let control = fields.get("control").map_or("", String::as_str);
                match control {
                    "PC-N" | "PC-O" => format!("{control} {}", cell_key_of(&fields, &["duration"])),
                    "PC-N0" => format!("{control} {}", cell_key_of(&fields, &["n1", "c"])),
                    _ => continue,
                }
            }
            _ => continue,
        };
        cells.insert(key, ComparedCell { fields });
    }
    cells
}

fn field_u64(cell: &ComparedCell, name: &str) -> Option<u64> {
    cell.fields.get(name).and_then(|value| value.parse().ok())
}

fn field_text<'cell>(cell: &'cell ComparedCell, name: &str) -> &'cell str {
    cell.fields.get(name).map_or("", String::as_str)
}

#[derive(Default)]
struct ExtraReads {
    cells: u64,
    positive_cells: u64,
    total_calls: i64,
    total_bytes: i64,
    minimum_calls: Option<i64>,
    maximum_calls: Option<i64>,
}

impl ExtraReads {
    fn add(&mut self, calls: i64, bytes: i64) {
        self.cells += 1;
        if calls > 0 {
            self.positive_cells += 1;
        }
        self.total_calls += calls;
        self.total_bytes += bytes;
        self.minimum_calls = Some(
            self.minimum_calls
                .map_or(calls, |current| current.min(calls)),
        );
        self.maximum_calls = Some(
            self.maximum_calls
                .map_or(calls, |current| current.max(calls)),
        );
    }
}

fn run_third_run_compare(prefix: &str) -> bool {
    let mut texts = BTreeMap::new();
    for arm in THIRD_RUN_ARMS {
        let path = format!("{prefix}-{arm}.out");
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                texts.insert(arm, text);
            }
            Err(error) => {
                emit_result(&format!(
                    "name=r3_compare_input arm={arm} verdict=stop reason={:?}",
                    format!("{path}: {error}")
                ));
                return false;
            }
        }
    }
    for (arm, text) in &texts {
        let finished = text
            .lines()
            .last()
            .is_some_and(|line| line.starts_with("E7RESULT name=done emitted="));
        emit_result(&format!(
            "name=r3_compare_input arm={arm} verdict={} lines={}",
            if finished { "pass" } else { "stop_unfinished" },
            text.lines().count()
        ));
        if !finished {
            return false;
        }
    }
    let cells: BTreeMap<&str, BTreeMap<String, ComparedCell>> = texts
        .iter()
        .map(|(arm, text)| (*arm, compared_cells_of(text)))
        .collect();
    let today = &cells["today"];
    // 7.3 第四行：今天减自己 = 0（每一格，装置自检）。
    let self_difference_cells = today
        .values()
        .filter(|cell| field_u64(cell, "a_reads").is_some())
        .count();
    emit_result(&format!(
        "name=r3_section_seven_three arm=today row=4 item=\"今天那一臂减自己的多读 = 0\" cells={self_difference_cells} nonzero_cells=0 verdict=pass"
    ));
    for arm in THIRD_RUN_ARMS.into_iter().filter(|arm| *arm != "today") {
        let arm_cells = &cells[arm];
        // Q5 多读：H1d 不断那一格、H1e、PC-N（不断），臂减今天，同一格、同一份起始镜像。
        let mut by_family: BTreeMap<&str, ExtraReads> = BTreeMap::new();
        for (key, cell) in arm_cells {
            let Some(today_cell) = today.get(key) else {
                continue;
            };
            let family = key.split(' ').next().unwrap_or("");
            if !matches!(family, "h1d" | "h1e" | "PC-N") {
                continue;
            }
            let (Some(calls), Some(today_calls), Some(bytes), Some(today_bytes)) = (
                field_u64(cell, "a_reads"),
                field_u64(today_cell, "a_reads"),
                field_u64(cell, "a_read_bytes"),
                field_u64(today_cell, "a_read_bytes"),
            ) else {
                continue;
            };
            by_family.entry(family).or_default().add(
                i64::try_from(calls).expect("读次数") - i64::try_from(today_calls).expect("读次数"),
                i64::try_from(bytes).expect("读字节") - i64::try_from(today_bytes).expect("读字节"),
            );
        }
        for (family, extra) in &by_family {
            emit_result(&format!(
                "name=r3_q5_extra_reads arm={arm} family={family} cells={} positive_cells={} total_extra_read_calls={} total_extra_read_bytes={} minimum_extra_read_calls={} maximum_extra_read_calls={}",
                extra.cells,
                extra.positive_cells,
                extra.total_calls,
                extra.total_bytes,
                extra.minimum_calls.map_or("none".to_string(), |value| value.to_string()),
                extra.maximum_calls.map_or("none".to_string(), |value| value.to_string())
            ));
        }
        // PC-多读（PC-N，W）：乙 ≥ R ×（第一遍里读失败的落点个数）且 > 0；丙 > 0。
        let whole_call_key = "PC-N duration=W".to_string();
        if let (Some(cell), Some(today_cell)) =
            (arm_cells.get(&whole_call_key), today.get(&whole_call_key))
        {
            let extra = i64::try_from(field_u64(cell, "a_reads").unwrap_or(0)).expect("读次数")
                - i64::try_from(field_u64(today_cell, "a_reads").unwrap_or(0)).expect("读次数");
            let faults = i64::try_from(field_u64(cell, "faults").unwrap_or(0)).expect("落点数");
            let (expectation, passed) = match ThirdRunCandidate::of(arm) {
                Some(ThirdRunCandidate::Yi) => (
                    "extra_reads_at_least_r_times_faults_and_positive",
                    extra >= i64::try_from(THIRD_RUN_REREAD_ROUNDS).expect("R") * faults
                        && extra > 0,
                ),
                Some(ThirdRunCandidate::Bing) => ("extra_reads_positive", extra > 0),
                Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Today) | None => {
                    ("not_registered_for_this_arm", true)
                }
            };
            emit_result(&format!(
                "name=r3_positive_control arm={arm} control=PC-extra-reads verdict={} expectation={expectation} extra_read_calls={extra} faults={faults}",
                verdict_word(passed)
            ));
        } else {
            emit_result(&format!(
                "name=r3_positive_control arm={arm} control=PC-extra-reads verdict=not_constructible reason=\"PC-N W 那一行缺\""
            ));
        }
        // PC-N0 与今天逐项相同（F12）：K 类、所选根、E、写的指纹（-配置续 取号那 8 个字节已抹）。
        let mut no_newer_root_cells = 0u64;
        let mut no_newer_root_differences = Vec::new();
        for (key, cell) in arm_cells
            .iter()
            .filter(|(cell_key, _)| cell_key.starts_with("PC-N0"))
        {
            no_newer_root_cells += 1;
            let Some(today_cell) = today.get(key) else {
                no_newer_root_differences.push(format!("{key}: 今天那一臂没有这一格"));
                continue;
            };
            for field in ["a_class", "a_chosen", "a_effective", "write_fingerprint"] {
                if field_text(cell, field) != field_text(today_cell, field) {
                    no_newer_root_differences.push(format!(
                        "{key}: {field} 臂 {} 今天 {}",
                        field_text(cell, field),
                        field_text(today_cell, field)
                    ));
                }
            }
        }
        emit_result(&format!(
            "name=r3_positive_control arm={arm} control=PC-N0-same-as-today verdict={} cells={no_newer_root_cells} differences={}",
            verdict_word(no_newer_root_differences.is_empty() && no_newer_root_cells == 8),
            no_newer_root_differences.len()
        ));
        for difference in &no_newer_root_differences {
            emit_result(&format!(
                "name=r3_compare_finding arm={arm} kind=f12 detail={difference:?}"
            ));
        }
    }
    // 7.3 第二行：今天那一臂上装置算的 N-配置 / N-槽 为真的格 = 甲-配置 / 甲-槽 交回「N 判真」拒绝成员的格。
    for (witness_field, jia_arm) in [("device_n_cfg", "jia-cfg"), ("device_n_slot", "jia-slot")] {
        let mut compared = 0u64;
        let mut mismatches = Vec::new();
        for (key, today_cell) in today {
            let Some(jia_cell) = cells[jia_arm].get(key) else {
                continue;
            };
            if field_text(today_cell, witness_field).is_empty() {
                continue;
            }
            compared += 1;
            let device_says = field_text(today_cell, witness_field) == "true";
            let jia_refused = field_text(jia_cell, "a_error") == third_run_refusal_member(jia_arm);
            if device_says != jia_refused {
                mismatches.push(format!(
                    "{key}: 装置 {witness_field}={device_says}，{jia_arm} 拒={jia_refused}"
                ));
            }
        }
        emit_result(&format!(
            "name=r3_section_seven_three row=2 witness={witness_field} jia_arm={jia_arm} cells={compared} mismatches={} verdict={}",
            mismatches.len(),
            if mismatches.is_empty() { "pass" } else { "stop_s4" }
        ));
        for mismatch in &mismatches {
            emit_result(&format!(
                "name=r3_compare_finding kind=s4 detail={mismatch:?}"
            ));
        }
    }
    // 7.3 第三行：T(δ) 对今天、甲、丙各臂与 W 逐格相同（H1d 不断那一格与 PC-N）。
    for arm in THIRD_RUN_ARMS.into_iter().filter(|arm| {
        matches!(
            ThirdRunCandidate::of(arm),
            Some(ThirdRunCandidate::Today | ThirdRunCandidate::Jia | ThirdRunCandidate::Bing)
        )
    }) {
        let mut compared = 0u64;
        let mut differences = Vec::new();
        for (key, cell) in cells[arm]
            .iter()
            .filter(|(key, _)| key.contains("duration=T1"))
        {
            let whole_key = key.replace("duration=T1", "duration=W");
            let Some(whole_cell) = cells[arm].get(&whole_key) else {
                continue;
            };
            compared += 1;
            for field in [
                "a_class",
                "a_error",
                "a_writes",
                "a_reads",
                "a_chosen",
                "a_effective",
                "q3_read_back",
                "overwrite_cell",
            ] {
                if field_text(cell, field) != field_text(whole_cell, field) {
                    differences.push(format!(
                        "{key}: {field} T {} W {}",
                        field_text(cell, field),
                        field_text(whole_cell, field)
                    ));
                }
            }
        }
        emit_result(&format!(
            "name=r3_section_seven_three row=3 arm={arm} cells={compared} differences={} verdict={}",
            differences.len(),
            if differences.is_empty() { "pass" } else { "void_v5" }
        ));
        for difference in &differences {
            emit_result(&format!(
                "name=r3_compare_finding arm={arm} kind=v5 detail={difference:?}"
            ));
        }
    }
    // F11：丙-配置 与 甲-配置 只在第 22 条那几格与 O(1 只) 那几格不同（第四节推的 ③）。
    let mut failure_clause_eleven_cells = 0u64;
    let mut failure_clause_eleven_differences = Vec::new();
    for (key, bing_cell) in &cells["bing-cfg"] {
        if key.contains("duration=O1only") || key.starts_with("PC-O") {
            continue;
        }
        let Some(jia_cell) = cells["jia-cfg"].get(key) else {
            continue;
        };
        failure_clause_eleven_cells += 1;
        let jia_refused = field_text(jia_cell, "a_error") == third_run_refusal_member("jia-cfg");
        let bing_refused = field_text(bing_cell, "a_error") == third_run_refusal_member("bing-cfg");
        let same = jia_refused == bing_refused
            && field_text(bing_cell, "a_class") == field_text(jia_cell, "a_class")
            && field_text(bing_cell, "a_chosen") == field_text(jia_cell, "a_chosen")
            && field_text(bing_cell, "a_effective") == field_text(jia_cell, "a_effective")
            && field_text(bing_cell, "q3_read_back") == field_text(jia_cell, "q3_read_back")
            && field_text(bing_cell, "overwrite_cell") == field_text(jia_cell, "overwrite_cell");
        if !same {
            failure_clause_eleven_differences.push(key.clone());
        }
    }
    emit_result(&format!(
        "name=r3_f11 cells={failure_clause_eleven_cells} cells_differing_outside_o1_and_cell_22={} verdict={}",
        failure_clause_eleven_differences.len(),
        if failure_clause_eleven_differences.is_empty() { "inference_holds" } else { "inference_fails_f11" }
    ));
    for difference in &failure_clause_eleven_differences {
        emit_result(&format!(
            "name=r3_compare_finding kind=f11 detail={difference:?}"
        ));
    }
    true
}

/// 第 3 次跑的模式：开跑检查不过就停（V3 / V4），过了跑点名的那一段。交回 true = 这个模式归第 3 次跑。
fn run_third_run_mode(mode: &str, command_line_arguments: &[String]) -> bool {
    let known = [
        "r3-seg1",
        "r3-constants",
        "r3-pc",
        "r3-h1d",
        "r3-h1e",
        "r3-compare",
    ];
    if !known.contains(&mode) {
        return false;
    }
    emit_input_header();
    if mode == "r3-compare" {
        let prefix = command_line_arguments
            .get(2)
            .cloned()
            .unwrap_or_else(|| "research/results/e158-root-choice-repair-r3-seg1".to_string());
        run_third_run_compare(&prefix);
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    let arm = third_run_arm_label();
    if !run_third_run_constants_and_anchors(&arm) {
        emit_result(&format!(
            "name=stop arm={arm} reason=V3 detail=\"第 3 次跑的常量回比、第七节锚点或臂与副本的对应有不过的，整轮不开跑\""
        ));
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    match mode {
        "r3-seg1" => {
            run_third_run_positive_control_newer(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_second_read(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_no_newer(&GEOMETRY_PRIMARY, &arm);
            run_third_run_held_once_family(&GEOMETRY_PRIMARY, &arm);
            run_third_run_tail_family(&GEOMETRY_PRIMARY, &arm);
        }
        "r3-constants" => {}
        "r3-pc" => {
            run_third_run_positive_control_newer(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_second_read(&GEOMETRY_PRIMARY, &arm);
            run_third_run_positive_control_no_newer(&GEOMETRY_PRIMARY, &arm);
        }
        "r3-h1d" => run_third_run_held_once_family(&GEOMETRY_PRIMARY, &arm),
        "r3-h1e" => run_third_run_tail_family(&GEOMETRY_PRIMARY, &arm),
        other => unreachable!("第 3 次跑的模式表 known 已经挡过：{other}"),
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    true
}

/// 产物文件头（登记开头「产物文件头第一行写这个汇总 sha256」）：快照的汇总 sha256 由跑的人经环境变量交进来；
/// `E7INPUT` 开头，`research/scripts/replay.sh` 比对前删掉，`name=done` 不数它。
fn emit_input_header() {
    println!(
        "E7INPUT name=crates_snapshot key=E158 sha256={}",
        env::var("SINGLEFS_E158_SNAPSHOT_SHA256").unwrap_or_else(|_| "unset".to_string())
    );
}

#[cfg(test)]
mod third_run_tests {
    use super::*;

    fn whole_call_read_failure_at(offset: u64, length: u64) -> ThirdRunFault {
        ThirdRunFault {
            device: DeviceIdentity(0),
            offset,
            length,
            form: UnreadableForm::ReadFails,
            duration: FaultDuration::WholeCall,
            label: format!("probe({offset})"),
        }
    }

    /// 登记 5.6 与 7.2：第 3 次跑的常量回比、偏移解码、根槽 / 环槽 / 系统配置槽数全过（V3、V4；变异 M12 要它红）。
    #[test]
    fn third_run_constants_and_anchors_all_pass() {
        assert!(run_third_run_constants_and_anchors("today"));
    }

    /// 「读法写死」表「时长」一行：W 每次都坏；T(τ) 坏到虚拟时钟走到 τ；O(m 只) 只坏第 m 次；O(m 起) 第 m 次起都坏（变异 M5、M6 要它红）。
    #[test]
    fn fault_durations_follow_the_registered_table() {
        assert!(FaultDuration::WholeCall.is_active(1, 0));
        assert!(FaultDuration::WholeCall.is_active(7, 5));
        assert!(FaultDuration::UntilVirtualTime(1).is_active(1, 0));
        assert!(!FaultDuration::UntilVirtualTime(1).is_active(2, 1));
        assert!(FaultDuration::UntilVirtualTime(2).is_active(2, 1));
        assert!(FaultDuration::OnlyTheNthRead(1).is_active(1, 0));
        assert!(!FaultDuration::OnlyTheNthRead(1).is_active(2, 0));
        assert!(!FaultDuration::OnlyTheNthRead(2).is_active(1, 0));
        assert!(!FaultDuration::FromTheNthRead(2).is_active(1, 0));
        assert!(FaultDuration::FromTheNthRead(2).is_active(3, 0));
        assert!(FaultDuration::OnlyTheNthRead(1).is_active_on_the_first_read());
        assert!(!FaultDuration::OnlyTheNthRead(2).is_active_on_the_first_read());
    }

    /// 虚拟时钟只由等待钩子推进：推一格之后 T(1) 的落点读得出。
    #[test]
    fn fault_until_one_interval_clears_after_one_wait() {
        let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
        let fault = ThirdRunFault {
            duration: FaultDuration::UntilVirtualTime(1),
            ..whole_call_read_failure_at(0, 4096)
        };
        let call = third_run_call(&pool, &[fault], None, None, |devices| {
            let first = singlefs_core::recovery::PoolReader::read(
                &*devices,
                DeviceIdentity(0),
                DeviceOffsetInBytes(0),
                4096,
            );
            third_run_arm_hooks::advance_one_interval(THIRD_RUN_VIRTUAL_INTERVAL);
            let second = singlefs_core::recovery::PoolReader::read(
                &*devices,
                DeviceIdentity(0),
                DeviceOffsetInBytes(0),
                4096,
            );
            (first.is_some(), second.is_some())
        });
        assert_eq!(call.outcome, (false, true));
        assert_eq!(call.intercepted_reads, vec![1]);
    }

    /// Q5 的读数：读坏的读也数进读调用次数与读字节数（变异 M14 要它红）。
    #[test]
    fn failed_reads_are_counted_as_reads() {
        let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
        let call = third_run_call(
            &pool,
            &[whole_call_read_failure_at(0, 4096)],
            None,
            None,
            |devices| {
                singlefs_core::recovery::PoolReader::read(
                    &*devices,
                    DeviceIdentity(0),
                    DeviceOffsetInBytes(0),
                    4096,
                )
                .is_some()
            },
        );
        assert!(!call.outcome);
        assert_eq!(call.read_calls, 1);
        assert_eq!(call.read_bytes, 4096);
    }

    /// V2：点了名而这次调用一次都没读到的落点要报出来（变异 M13 要它红）。
    #[test]
    fn named_target_never_read_is_reported_as_not_intercepted() {
        let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
        let faults = [
            whole_call_read_failure_at(0, 4096),
            whole_call_read_failure_at(1 << 20, 4096),
        ];
        let call = third_run_call(&pool, &faults, None, None, |devices| {
            singlefs_core::recovery::PoolReader::read(
                &*devices,
                DeviceIdentity(0),
                DeviceOffsetInBytes(0),
                4096,
            )
            .is_some()
        });
        assert_eq!(call.unintercepted(&faults), vec![faults[1].label.clone()]);
    }

    /// K3 / K4 按这次调用设备一层写了几次分，不按错误成员分（变异 M10 要它红）。
    #[test]
    fn refusal_after_one_write_is_classed_by_its_write_count() {
        let refused: Result<Mounted, singlefs_core::mount::MountError> =
            Err(singlefs_core::mount::MountError::InstanceTableMalformed);
        assert_eq!(third_run_outcome_class(&refused, 1), "K4");
        assert_eq!(third_run_outcome_class(&refused, 0), "K3");
    }

    /// N-配置 的 c_见证 取每块盘两槽的最大值，不只取世代号最新那一槽（手写两盘两槽：新槽 tail 0 实例 2、旧槽 tail c 实例 1；变异 M3 的装置一半）。
    #[test]
    fn the_configuration_witness_takes_both_slots_of_every_device() {
        let witnessed = 9u64;
        let decoded = DeviceDecodedImage {
            root_keys: BTreeSet::new(),
            unreadable_root_slots: 0,
            records: BTreeMap::new(),
            system_configuration_slots: vec![
                (0, 2, 0, 2),
                (0, 1, witnessed, 1),
                (1, 2, 0, 2),
                (1, 1, witnessed, 1),
            ],
        };
        let (newer, tail, last, undecidable) = device_configuration_witness(&decoded, (1, 3));
        assert_eq!(tail, witnessed);
        assert_eq!(last, None);
        assert!(newer);
        assert!(undecidable);
    }

    /// 装置自己的择根与重放在一段 P(1, C) 上与 `crates/` 的 `choose_root`、`replay_journal` 交回的同一条根（7.3 第一行的不注入那一格）。
    #[test]
    fn the_device_replay_agrees_with_crates_on_an_untouched_history() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
        third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
        let geometry = independent_geometry(&history.pool).expect("几何");
        let judgement = device_judgement(&history.pool, &geometry, &[]);
        let devices = devices_from_pool(&history.pool, IMAGE_BYTES);
        let system_configuration = choose_system_configuration(&devices).expect("系统配置");
        let chosen =
            singlefs_core::recovery::choose_root(&devices, &system_configuration).expect("根");
        let records = scan_journal(&devices, &system_configuration);
        let (_, effective) = replay_journal(
            &devices,
            &chosen,
            system_configuration.immutable.sizes.journal_ring_bytes,
            &records,
            true,
        )
        .expect("重放");
        assert_eq!(judgement.chosen, Some(root_pair(&chosen)));
        assert_eq!(judgement.effective, Some(root_pair(&effective)));
        assert_eq!(judgement.effective, Some(history.tip()));
        assert!(!judgement.truth_newer);
        assert_eq!(history.tail_check_failures, Vec::<String>::new());
        assert_eq!(history.tail_checks, 2);
    }

    /// PC-N 的真值一半：被藏那条根在孪生镜像上读得出，N_真 为真；今天那一臂回滚到被藏那条根的前一条（变异 M1 要它红）。
    #[test]
    fn the_truth_of_a_newer_root_is_read_on_the_twin_image() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
        third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
        let geometry = independent_geometry(&history.pool).expect("几何");
        let faults = third_run_abandonment_faults(
            &history,
            &geometry,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
            FaultDuration::WholeCall,
        )
        .expect("落点")
        .expect("有记录");
        let judgement = device_judgement(&history.pool, &geometry, &faults);
        assert!(judgement.truth_newer);
        assert!(judgement.configuration_newer);
        assert_eq!(
            judgement.effective,
            history.timeline.iter().rev().nth(1).copied()
        );
    }

    /// PC-丢写（今天那一臂）：P(1, C) → A（PC-N 那组故障，另加系统配置没见证到被藏的根：每块盘世代号最大的那一槽读回全 0；
    /// C554 乙合进 `crates` 之后见证在时可写挂载拒，造不出被抛弃的根）→ 最后确认那一版的数据单元被改写 → 再可写挂载：Q1 前两栏数得出、
    /// 读回不是最后确认那一版（变异 M8 要它红：那时恢复落到再挂载写下的根，它不在被测那次挂载开始时的镜像上）。
    #[test]
    fn an_overwritten_last_confirmed_version_counts_as_a_lost_write() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
        third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
        let geometry = independent_geometry(&history.pool).expect("几何");
        let mut faults = third_run_abandonment_faults(
            &history,
            &geometry,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
            FaultDuration::WholeCall,
        )
        .expect("落点")
        .expect("有记录");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            faults.push(ThirdRunFault {
                device,
                offset: crate::tests::newest_system_configuration_slot_offset(
                    &history.pool,
                    device,
                ),
                length: SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
                form: UnreadableForm::ReadsZeros,
                duration: FaultDuration::WholeCall,
                label: format!("newest_system_configuration(device={})", device.0),
            });
        }
        let tested = run_third_run_mount_writable(&history.pool, &parameters, &faults, None, None);
        assert!(tested.outcome.is_ok());
        let protected = history.confirmations.last().cloned().expect("确认过");
        let (corrupted, loss) = positive_control_lost_write(
            &history,
            &protected,
            &history.pool,
            &tested.pool_after,
            &parameters,
        );
        assert!(corrupted > 0);
        assert!(loss.overwrite_cell());
        assert_ne!(loss.read_back, "last_confirmed");
    }

    /// 跨臂对拍读产物行：值里不带空格的字段按 `k=v` 取得出。
    #[test]
    fn result_line_fields_are_split_on_spaces() {
        let fields = fields_of_a_result_line(
            "E7RESULT name=r3_h1d_cell arm=today n1=2 cut=none a_reads=12 q3c_tail=own_version:2:9",
        );
        assert_eq!(fields.get("n1").map(String::as_str), Some("2"));
        assert_eq!(
            fields.get("q3c_tail").map(String::as_str),
            Some("own_version:2:9")
        );
    }
}

// ============================================================================
// 十二、第 4 次跑（`research/prompts/e158-r4-prereg.md`）：C554 怎么修（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行）。
//      装置按登记 5.7 的五段写，模式 `r4-seg1` … `r4-seg5`，跨臂的量在 `r4-compare`。第 2、3 次跑的函数只调用、不改语义；
//      要改的一律复制一份给 `r4-*` 用（登记文件头「装置写在哪」）。
// ============================================================================

/// 候选（岔路单第 1 行：甲 拒可写、乙 重读后再判、丙 从记录重建、对照 今天；乙-窄读 是主 agent 认定第 1 项另立的那一条乙）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FourthRunCandidate {
    Today,
    Jia,
    Yi,
    YiNarrow,
    Bing,
}

/// 臂的判据 N 取哪一样（登记 5.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FourthRunWitness {
    NotJudged,
    SystemConfiguration,
    SystemConfigurationCarried,
    UnreadableRootRingSlot,
}

#[derive(Clone, Copy, Debug)]
struct FourthRunArm {
    name: &'static str,
    candidate: FourthRunCandidate,
    witness: FourthRunWitness,
    /// 乙、乙-窄读 的重读轮数 R（别的臂 0）。
    reread_rounds: u64,
}

/// 第 4 次跑的臂（文件名拼写 = 臂表 r4 行里的名字）：前十二条是 G0 主族的臂；后六条是第五段敏感性点 R = 3 的 乙 与 乙-窄读。
const FOURTH_RUN_ARMS: [FourthRunArm; 18] = [
    FourthRunArm {
        name: "today",
        candidate: FourthRunCandidate::Today,
        witness: FourthRunWitness::NotJudged,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "jia-cfg",
        candidate: FourthRunCandidate::Jia,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "jia-cfg-carry",
        candidate: FourthRunCandidate::Jia,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "jia-slot",
        candidate: FourthRunCandidate::Jia,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "yi-cfg",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-cfg-carry",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-slot",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-narrow-cfg",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-narrow-cfg-carry",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-narrow-slot",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "bing-cfg",
        candidate: FourthRunCandidate::Bing,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "bing-cfg-carry",
        candidate: FourthRunCandidate::Bing,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "yi-cfg-three-rounds",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-cfg-carry-three-rounds",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-slot-three-rounds",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-narrow-cfg-three-rounds",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-narrow-cfg-carry-three-rounds",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-narrow-slot-three-rounds",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 3,
    },
];

fn fourth_run_arm_named(name: &str) -> Option<FourthRunArm> {
    FOURTH_RUN_ARMS.iter().copied().find(|arm| arm.name == name)
}

impl FourthRunArm {
    fn is_today(self) -> bool {
        self.candidate == FourthRunCandidate::Today
    }

    fn rereads(self) -> bool {
        match self.candidate {
            FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => true,
            FourthRunCandidate::Today | FourthRunCandidate::Jia | FourthRunCandidate::Bing => false,
        }
    }

    /// N 判真之后这一臂交回的拒绝成员名（臂表 r4 行加进 `MountError` 的成员）；今天那一臂没有。
    fn refusal_member(self) -> &'static str {
        match (self.candidate, self.witness) {
            (FourthRunCandidate::Today, _) | (_, FourthRunWitness::NotJudged) => "none",
            (FourthRunCandidate::Bing, _) => "R3RebuildFromRecordsStoppedBelowTheWitness",
            (
                FourthRunCandidate::Jia | FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow,
                FourthRunWitness::UnreadableRootRingSlot,
            ) => "R3UnreadableRootRingSlot",
            (
                FourthRunCandidate::Jia | FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow,
                FourthRunWitness::SystemConfiguration
                | FourthRunWitness::SystemConfigurationCarried,
            ) => "R3NewerRootWitnessedByTheSystemConfiguration",
        }
    }

    /// 同一种 N 的甲臂（PC-多读 (iii) 与 Q5-拒 的参照）。
    fn jia_arm_with_the_same_witness(self) -> Option<&'static str> {
        match self.witness {
            FourthRunWitness::NotJudged => None,
            FourthRunWitness::SystemConfiguration => Some("jia-cfg"),
            FourthRunWitness::SystemConfigurationCarried => Some("jia-cfg-carry"),
            FourthRunWitness::UnreadableRootRingSlot => Some("jia-slot"),
        }
    }

    fn witness_name(self) -> &'static str {
        match self.witness {
            FourthRunWitness::NotJudged => "none",
            FourthRunWitness::SystemConfiguration => "cfg",
            FourthRunWitness::SystemConfigurationCarried => "cfg-carry",
            FourthRunWitness::UnreadableRootRingSlot => "slot",
        }
    }

    fn candidate_name(self) -> &'static str {
        match self.candidate {
            FourthRunCandidate::Today => "today",
            FourthRunCandidate::Jia => "jia",
            FourthRunCandidate::Yi => "yi",
            FourthRunCandidate::YiNarrow => "yi-narrow",
            FourthRunCandidate::Bing => "bing",
        }
    }
}

/// H1d 与各族 A 的时长（登记 5.3：各臂 W 与 O(1 只)；乙、乙-窄读 另加 T(δ)、T(2δ)，R = 3 的另加到 T(4δ)（8.2）；
/// 今天、甲、丙 另跑 T(δ) 只跑不断那一格，给 7.3 第三行 V5 对拍）。交回 (时长, 只跑不断那一格)。
fn fourth_run_held_once_durations(arm: FourthRunArm) -> Vec<(FaultDuration, bool)> {
    let mut durations = vec![
        (FaultDuration::WholeCall, false),
        (FaultDuration::OnlyTheNthRead(1), false),
    ];
    if arm.rereads() {
        for intervals in 1..=(arm.reread_rounds + 1) {
            durations.push((
                FaultDuration::UntilVirtualTime(intervals * THIRD_RUN_VIRTUAL_INTERVAL),
                false,
            ));
        }
    } else {
        durations.push((
            FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
            true,
        ));
    }
    durations
}

/// H1e 与接 H1e 尾巴的族的时长（登记 5.3：W；乙、乙-窄读 另加 T(δ)，R = 3 的另加到 T(4δ)（8.2））。
fn fourth_run_tail_durations(arm: FourthRunArm) -> Vec<FaultDuration> {
    let mut durations = vec![FaultDuration::WholeCall];
    if arm.rereads() {
        let longest = if arm.reread_rounds > 1 {
            arm.reread_rounds + 1
        } else {
            1
        };
        for intervals in 1..=longest {
            durations.push(FaultDuration::UntilVirtualTime(
                intervals * THIRD_RUN_VIRTUAL_INTERVAL,
            ));
        }
    }
    durations
}

// ---------------------------------------------------------------------------
// 12.1 受观测的入口调用（第 4 次跑）：第 3 次跑的故障设备外面再包一层，数「第一个等待标记之前 / 之后」打到盘上的读、
//      标记之后读回全零的读（登记 5.0 第三条、8.1 末），与第一个标记那一刻已被拦下过的落点个数 f（5.4 PC-多读 (iii)）。
// ---------------------------------------------------------------------------

#[derive(Default)]
struct FourthRunReadMarker {
    read_calls: u64,
    reads_before_the_first_wait: Option<u64>,
    faults_intercepted_before_the_first_wait: Option<u64>,
    all_zero_reads_after_the_first_wait: u64,
}

struct FourthRunDevice {
    device: ThirdRunDevice,
    marker: Rc<RefCell<FourthRunReadMarker>>,
}

impl BlockDevice for FourthRunDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let result = self.device.read_at(offset, buffer);
        let mut marker = self.marker.borrow_mut();
        marker.read_calls += 1;
        if marker.reads_before_the_first_wait.is_some()
            && result.is_ok()
            && buffer.iter().all(|byte| *byte == 0)
        {
            marker.all_zero_reads_after_the_first_wait += 1;
        }
        result
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.device.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.device.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.device.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.device.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.device.size_in_bytes()
    }
}

/// 臂的观测里第 4 次跑新加的两样（臂表 r4 行的观测字段）：重读轮数上限 R、读缓存收不收 journal 空槽（乙-窄读 收）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FourthRunArmExtras {
    reread_rounds_limit: u32,
    zeroed_journal_ring_slots_are_cached: bool,
}

/// 等待钩子（第 4 次跑）：推进虚拟时钟之前先调 `on_wait`（记第一个等待标记）。臂的代码只在副本里有，见 `third_run_arm_hooks`。
#[allow(
    unexpected_cfgs,
    reason = "特性 e158-r3-arm 只在草稿副本的 singlefs-harness/Cargo.toml 里声明（research/mutations/e158_arms.tsv 的 r4 收尾那一行）"
)]
mod fourth_run_arm_hooks {
    #[cfg(feature = "e158-r3-arm")]
    pub fn install_the_marking_wait_hook(interval: u64, on_wait: Box<dyn Fn()>) {
        singlefs_core::mount::r3_install_wait_hook(Some(Box::new(move || {
            on_wait();
            super::third_run_arm_hooks::advance_one_interval(interval);
        })));
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn install_the_marking_wait_hook(_interval: u64, _on_wait: Box<dyn Fn()>) {}

    #[cfg(feature = "e158-r3-arm")]
    pub fn take_the_arm_observation() -> Option<(super::ArmObservation, super::FourthRunArmExtras)>
    {
        let observation = singlefs_core::mount::r3_take_observation();
        Some((
            super::ArmObservation {
                witness_judged: observation.witness_judged,
                newer_root_on_the_first_pass: observation.newer_root_on_the_first_pass,
                witnessed_tail: observation.witnessed_tail,
                selected_version_last_counter: observation.selected_version_last_counter,
                undecidable: observation.undecidable,
                unreadable_ring_slots_on_the_first_pass: observation
                    .unreadable_ring_slots_on_the_first_pass,
                reread_rounds: observation.reread_rounds,
                rebuilt_from_records: observation.rebuilt_from_records,
                shadow_ledger_table_unreadable_on_the_first_read: observation
                    .shadow_ledger_table_unreadable_on_the_first_read,
                shadow_ledger_table_rereads: observation.shadow_ledger_table_rereads,
            },
            super::FourthRunArmExtras {
                reread_rounds_limit: observation.reread_rounds_limit,
                zeroed_journal_ring_slots_are_cached: observation
                    .zeroed_journal_ring_slots_are_cached,
            },
        ))
    }

    #[cfg(not(feature = "e158-r3-arm"))]
    pub fn take_the_arm_observation() -> Option<(super::ArmObservation, super::FourthRunArmExtras)>
    {
        None
    }
}

/// 一次受观测的入口调用（第 4 次跑）：第 3 次跑那一份交回的东西，加标记前后的读。
struct FourthRunCall<Outcome> {
    call: ThirdRunCall<Outcome>,
    extras: Option<FourthRunArmExtras>,
    reads_before_the_first_wait: Option<u64>,
    faults_intercepted_before_the_first_wait: Option<u64>,
    all_zero_reads_after_the_first_wait: u64,
    /// 每个点了名的落点在这次调用里被读了几次（坏没坏都数；L5 的 M 取今天那一臂孪生镜像上这个数）。
    reads_of_the_targets: Vec<u64>,
}

impl<Outcome> FourthRunCall<Outcome> {
    /// 第一个等待标记之后打到盘上的读调用数（没有标记 = 0）。
    fn reads_after_the_first_wait(&self) -> u64 {
        self.reads_before_the_first_wait
            .map_or(0, |before| self.call.read_calls - before)
    }

    fn render_marker(&self, prefix: &str) -> String {
        format!(
            "{prefix}_reads_before_first_wait={} {prefix}_reads_after_first_wait={} {prefix}_faults_intercepted_before_first_wait={} {prefix}_zero_reads_after_first_wait={} {prefix}_arm_rounds_limit={} {prefix}_arm_caches_zeroed_journal_slots={}",
            self.reads_before_the_first_wait
                .map_or("none".to_string(), |reads| reads.to_string()),
            self.reads_after_the_first_wait(),
            self.faults_intercepted_before_the_first_wait
                .map_or("none".to_string(), |count| count.to_string()),
            self.all_zero_reads_after_the_first_wait,
            self.extras
                .map_or("none".to_string(), |extras| extras.reread_rounds_limit.to_string()),
            self.extras.map_or("none".to_string(), |extras| {
                extras.zeroed_journal_ring_slots_are_cached.to_string()
            })
        )
    }
}

/// 等待钩子被调时记第一个等待标记：那一刻打到盘上的读调用数，与那一刻已被拦下过的落点个数 f（之后的标记不改它们）。
fn mark_the_first_wait(
    marker: &RefCell<FourthRunReadMarker>,
    state: &RefCell<ThirdRunDeviceState>,
) {
    let mut marker = marker.borrow_mut();
    if marker.reads_before_the_first_wait.is_none() {
        marker.reads_before_the_first_wait = Some(marker.read_calls);
        marker.faults_intercepted_before_the_first_wait = Some(
            state
                .borrow()
                .intercepted_reads
                .iter()
                .filter(|intercepted| **intercepted > 0)
                .count() as u64,
        );
    }
}

fn fourth_run_call<Outcome>(
    base: &MemoryPool,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
    call: impl FnOnce(&mut Vec<(DeviceIdentity, FourthRunDevice)>) -> Outcome,
) -> FourthRunCall<Outcome> {
    let state = Rc::new(RefCell::new(ThirdRunDeviceState::new(
        faults.to_vec(),
        failing_write_call,
        failing_barrier_call,
    )));
    let marker = Rc::new(RefCell::new(FourthRunReadMarker::default()));
    let mut devices: Vec<(DeviceIdentity, FourthRunDevice)> = devices_from_pool(base, IMAGE_BYTES)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FourthRunDevice {
                    device: ThirdRunDevice {
                        identity,
                        inner,
                        state: Rc::clone(&state),
                    },
                    marker: Rc::clone(&marker),
                },
            )
        })
        .collect();
    third_run_arm_hooks::reset_the_virtual_clock();
    let marker_for_the_hook = Rc::clone(&marker);
    let state_for_the_hook = Rc::clone(&state);
    fourth_run_arm_hooks::install_the_marking_wait_hook(
        THIRD_RUN_VIRTUAL_INTERVAL,
        Box::new(move || mark_the_first_wait(&marker_for_the_hook, &state_for_the_hook)),
    );
    let outcome = call(&mut devices);
    let taken = fourth_run_arm_hooks::take_the_arm_observation();
    let waits = third_run_arm_hooks::waits();
    let plain: Vec<(DeviceIdentity, SparseBlockDevice)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, device.device.inner))
        .collect();
    let pool_after = memory_pool_of(&plain, IMAGE_BYTES);
    let state = state.borrow();
    let marker = marker.borrow();
    FourthRunCall {
        call: ThirdRunCall {
            outcome,
            write_calls: state.write_calls,
            read_calls: state.read_calls,
            read_bytes: state.read_bytes,
            applied: state.applied.clone(),
            intercepted_reads: state.intercepted_reads.clone(),
            injected_failure_fired: state.injected_failure_fired,
            pool_after,
            waits,
            observation: taken.as_ref().map(|(observation, _)| observation.clone()),
        },
        extras: taken.map(|(_, extras)| extras),
        reads_before_the_first_wait: marker.reads_before_the_first_wait,
        faults_intercepted_before_the_first_wait: marker.faults_intercepted_before_the_first_wait,
        all_zero_reads_after_the_first_wait: marker.all_zero_reads_after_the_first_wait,
        reads_of_the_targets: state.reads_of_the_target.clone(),
    }
}

fn run_fourth_run_mount_writable(
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
) -> FourthRunCall<Result<Mounted, singlefs_core::mount::MountError>> {
    fourth_run_call(
        base,
        faults,
        failing_write_call,
        failing_barrier_call,
        |devices| mount_writable(parameters, devices),
    )
}

/// 一次被测挂载（第 4 次跑）：同第 3 次跑 `run_tested_mount`，调用走 `fourth_run_call`；今天那一臂照做 7.3 第一行与 F13。
struct FourthRunTestedMount {
    call: FourthRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    summary: TestedMountSummary,
    judgement: DeviceJudgement,
    unintercepted: Vec<String>,
}

fn run_fourth_run_tested_mount(
    arm: FourthRunArm,
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    label: &str,
    family_summary: &mut ThirdRunFamilySummary,
) -> Result<FourthRunTestedMount, String> {
    let geometry = independent_geometry(base)?;
    let call = run_fourth_run_mount_writable(base, parameters, faults, None, None);
    let judgement = device_judgement(base, &geometry, faults);
    let summary = TestedMountSummary::of(&call.call);
    let unintercepted = call.call.unintercepted(faults);
    if arm.is_today() {
        today_cross_checks(base, &judgement, &call.call, label, family_summary);
        if let Ok(mounted) = &call.call.outcome {
            family_summary.failure_clause_thirteen_checks += 1;
            let expected = first_txg_expected_by_the_rule_of_today(base, &geometry, faults);
            let got = mounted.output.row_publish.root().checkpoint_txg.0;
            if got != expected {
                family_summary
                    .failure_clause_thirteen_findings
                    .push(format!(
                        "{label}: 写行那次发布 txg {got}，按今天的规则 {expected}"
                    ));
            }
        }
    }
    Ok(FourthRunTestedMount {
        call,
        summary,
        judgement,
        unintercepted,
    })
}

fn render_fourth_run_tested_mount(tested: &FourthRunTestedMount) -> String {
    format!(
        "{} {} {}",
        tested.summary.render("a"),
        tested.call.render_marker("a"),
        render_device_judgement(&tested.judgement)
    )
}

// ---------------------------------------------------------------------------
// 12.2 Q0 的去向栏（登记「读法写死」表「被藏那条根的去向」一行）：被测挂载交回（或崩在中途）之后、撤故障之后，按孪生读法解。
//      判的次序（执行员修订第 4 条写死）：被择或被施加 → 环在且被抛弃 → 根槽被盖 → 原样留着 → 其它。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum HiddenRootDestination {
    InRingAndAbandoned,
    RootSlotOverwritten,
    ChosenOrApplied,
    LeftAsItWas,
    Other(String),
}

impl HiddenRootDestination {
    fn name(&self) -> String {
        match self {
            HiddenRootDestination::InRingAndAbandoned => "in_ring_and_abandoned".to_string(),
            HiddenRootDestination::RootSlotOverwritten => "root_slot_overwritten".to_string(),
            HiddenRootDestination::ChosenOrApplied => "chosen_or_applied".to_string(),
            HiddenRootDestination::LeftAsItWas => "left_as_it_was".to_string(),
            HiddenRootDestination::Other(detail) => format!("other:{detail}"),
        }
    }
}

/// 被藏那条根：键 K_h 与它在起始镜像上住的根槽 s_h = (区域, 区域内槽)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HiddenRootPosition {
    key: TimelineRoot,
    region: u64,
    slot: u64,
}

fn hidden_root_position(base: &MemoryPool, key: TimelineRoot) -> Option<HiddenRootPosition> {
    let geometry = independent_geometry(base).ok()?;
    checker_image::valid_roots(base, &geometry)
        .into_iter()
        .find(|(_, _, view)| (view.instance, view.checkpoint_txg) == key)
        .map(|(region, slot, _)| HiddenRootPosition { key, region, slot })
}

/// 登记 D22（单元原子性怎么合成） 已定项 2「槽序」：txg 的根落在 (txg mod 3, (txg div 3) mod S)（PC-N-盖 的点预言，变异 M21）。
fn predicted_root_ring_slot_of_a_publish(checkpoint_txg: u64, slots_per_region: u64) -> (u64, u64) {
    (
        checkpoint_txg % ROOT_RING_REGIONS_LOCAL,
        (checkpoint_txg / ROOT_RING_REGIONS_LOCAL) % slots_per_region,
    )
}

/// 按内容号表（装置的账，外加被测挂载写下的根沿用 E 的内容号）判去向；`tested_ok_effective` = 被测挂载交回 `Ok` 时 E 的键。
fn hidden_root_destination(
    position: &HiddenRootPosition,
    content_numbers: &BTreeMap<TimelineRoot, u64>,
    hidden_content_number: u64,
    after: &MemoryPool,
    tested_ok_effective: Option<TimelineRoot>,
) -> HiddenRootDestination {
    let Ok(geometry) = independent_geometry(after) else {
        return HiddenRootDestination::Other("geometry".to_string());
    };
    if tested_ok_effective
        .and_then(|effective| content_numbers.get(&effective))
        .is_some_and(|number| *number >= hidden_content_number)
    {
        return HiddenRootDestination::ChosenOrApplied;
    }
    let roots = checker_image::valid_roots(after, &geometry);
    let at_the_hidden_slot = roots
        .iter()
        .find(|(region, slot, _)| (*region, *slot) == (position.region, position.slot))
        .map(|(_, _, view)| (view.instance, view.checkpoint_txg));
    let abandoned = abandoned_roots_on(after);
    let hidden_abandoned = abandoned
        .as_ref()
        .is_ok_and(|set| set.contains(&position.key));
    let slot_holds_the_hidden_root = at_the_hidden_slot == Some(position.key);
    if slot_holds_the_hidden_root && hidden_abandoned {
        return HiddenRootDestination::InRingAndAbandoned;
    }
    let ring_has_the_hidden_content_or_newer = roots.iter().any(|(_, _, view)| {
        content_numbers
            .get(&(view.instance, view.checkpoint_txg))
            .is_some_and(|number| *number >= hidden_content_number)
    });
    if at_the_hidden_slot.is_some()
        && !slot_holds_the_hidden_root
        && !ring_has_the_hidden_content_or_newer
    {
        return HiddenRootDestination::RootSlotOverwritten;
    }
    if tested_ok_effective.is_none() && slot_holds_the_hidden_root && !hidden_abandoned {
        return HiddenRootDestination::LeftAsItWas;
    }
    HiddenRootDestination::Other(format!(
        "slot_holds={}:abandoned={}:abandoned_reading={}:ring_has_content_at_least_hidden={ring_has_the_hidden_content_or_newer}:tested_ok={}",
        key_text(at_the_hidden_slot),
        hidden_abandoned,
        abandoned.as_ref().map_or("error", |_| "ok"),
        tested_ok_effective.is_some()
    ))
}

/// 装置的内容号表，外加一次可写挂载写下的根（写行、暖机、抬 F）沿用 E 的内容号（「读法写死」表「内容号跟根走」一行）。
fn content_numbers_with_the_publishes_of(
    history: &ThirdRunHistory,
    mounted: Option<&Mounted>,
) -> BTreeMap<TimelineRoot, u64> {
    let mut numbers = history.content_number_by_root.clone();
    if let Some(mounted) = mounted {
        let carried = numbers
            .get(&root_pair(&mounted.output.effective_root))
            .copied()
            .unwrap_or(0);
        let mut roots = vec![root_pair(mounted.output.row_publish.root())];
        roots.extend(
            mounted
                .output
                .warm_up_publishes
                .iter()
                .map(|publish| root_pair(publish.root())),
        );
        for raised in mounted.output.space_admission.floor_raises() {
            roots.extend(
                raised
                    .publishes
                    .iter()
                    .map(|publish| root_pair(&publish.root)),
            );
        }
        for root in roots {
            numbers.insert(root, carried);
        }
    }
    numbers
}

/// 一份镜像的指纹（Q5-同成 要「起始镜像逐字节相同」；也给 H1f 的前缀跨臂核同一份）：每块盘写过的扇区按序号与字节混进 FNV-1a。
fn image_fingerprint(pool: &MemoryPool) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for (identity, device) in &pool.devices {
        mix(&identity.0.to_le_bytes());
        for sector in device.written_sectors_in(DeviceOffsetInBytes(0), pool.device_size_in_bytes) {
            mix(&sector.to_le_bytes());
            mix(&device.read(DeviceOffsetInBytes(sector * SECTOR_BYTES_LOCAL), 512));
        }
    }
    hash
}

// ---------------------------------------------------------------------------
// 12.3 阳性对照的句子（登记 5.4：每一句带标签、各句单独打一个字段；判定词 pass / fail / not_constructible，
//      条件式的 [测] 句前件不成立记 vacuous）。V1 按标签路由（11.1），由 `r4-compare` 机械算。
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SentenceTag {
    /// [测]：只由装置自己的账决定真假。
    Measured,
    /// [臂]：由臂的定义推出的这一臂的结局。
    ArmDefinition,
    /// [今]：对今天那一臂的底座怎么表现的描述。
    TodayDescription,
}

impl SentenceTag {
    fn name(self) -> &'static str {
        match self {
            SentenceTag::Measured => "measure",
            SentenceTag::ArmDefinition => "arm",
            SentenceTag::TodayDescription => "today",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SentenceVerdict {
    Pass,
    Fail,
    NotConstructible,
    Vacuous,
}

impl SentenceVerdict {
    fn name(self) -> &'static str {
        match self {
            SentenceVerdict::Pass => "pass",
            SentenceVerdict::Fail => "fail",
            SentenceVerdict::NotConstructible => "not_constructible",
            SentenceVerdict::Vacuous => "vacuous",
        }
    }

    fn of(holds: bool) -> SentenceVerdict {
        if holds {
            SentenceVerdict::Pass
        } else {
            SentenceVerdict::Fail
        }
    }

    /// 条件式：前件不成立 ⇒ vacuous，成立 ⇒ 后件定 pass / fail。
    fn conditional(antecedent: bool, consequent: bool) -> SentenceVerdict {
        if antecedent {
            SentenceVerdict::of(consequent)
        } else {
            SentenceVerdict::Vacuous
        }
    }
}

struct ControlSentence {
    tag: SentenceTag,
    name: &'static str,
    verdict: SentenceVerdict,
    /// 打不中走哪一条（[臂] 句：F9 / F10 / F12 / F16 / F18；[今] 句：F1 / F2 / F4 / F15 / F17 / F19；[测] 句：V1 作废）。
    clause: &'static str,
}

fn sentence(
    tag: SentenceTag,
    name: &'static str,
    verdict: SentenceVerdict,
    clause: &'static str,
) -> ControlSentence {
    ControlSentence {
        tag,
        name,
        verdict,
        clause,
    }
}

/// 一条对照的总判定：有 fail ⇒ fail；否则有 not_constructible ⇒ not_constructible；否则 pass。
fn overall_verdict(sentences: &[ControlSentence]) -> &'static str {
    if sentences
        .iter()
        .any(|sentence| sentence.verdict == SentenceVerdict::Fail)
    {
        "fail"
    } else if sentences
        .iter()
        .any(|sentence| sentence.verdict == SentenceVerdict::NotConstructible)
    {
        "not_constructible"
    } else {
        "pass"
    }
}

fn emit_fourth_run_positive_control(
    arm: &str,
    control: &str,
    coordinates: &str,
    sentences: &[ControlSentence],
    detail: &str,
) {
    let fields: Vec<String> = sentences
        .iter()
        .map(|sentence| {
            format!(
                "sentence_{}_{}={}",
                sentence.tag.name(),
                sentence.name,
                sentence.verdict.name()
            )
        })
        .collect();
    let routes: Vec<&str> = sentences
        .iter()
        .filter(|sentence| {
            matches!(
                sentence.verdict,
                SentenceVerdict::Fail | SentenceVerdict::NotConstructible
            )
        })
        .map(|sentence| sentence.clause)
        .collect();
    emit_result(&format!(
        "name=r4_positive_control arm={arm} control={control} {coordinates} verdict={} sentences={} {} missed_routes={} {detail}",
        overall_verdict(sentences),
        sentences.len(),
        fields.join(" "),
        if routes.is_empty() { "none".to_string() } else { routes.join(",") }
    ));
}

/// 阳性对照与 L 族那几格顺带做的 7.3 第一行对拍与 F13（今天那一臂）打成结果行（第 3 次跑 `emit_scratch_findings` 的第 4 次跑那一份）。
fn emit_fourth_run_scratch_findings(family: &str, arm: &str, scratch: &ThirdRunFamilySummary) {
    emit_result(&format!(
        "name=r4_cross_checks family={family} arm={arm} s4_mismatches={} f13_checked={} f13_failures={}",
        scratch.stop_condition_four_mismatches.len(),
        scratch.failure_clause_thirteen_checks,
        scratch.failure_clause_thirteen_findings.len()
    ));
    for (kind, list) in [
        ("s4", &scratch.stop_condition_four_mismatches),
        ("f13", &scratch.failure_clause_thirteen_findings),
    ] {
        for detail in list {
            emit_result(&format!(
                "name=r4_family_finding family={family} arm={arm} kind={kind} detail={detail:?}"
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// 12.4 起始镜像（被测那次挂载之前）与抛弃步 A（第 4 次跑）：P(n1, c)（第 3 次跑 5.3）、H1f 的「中间夹一次重挂载」、
//      H1g 的「a 次挂载崩在取号之后」（这 a 次在这一臂自己的副本上做，登记 5.3 H1g 行）；A 藏最新 k 条根（k = 2 是 8.2 的敏感性点）。
// ---------------------------------------------------------------------------

/// 本地常量（登记 5.6；开跑时回比 `crates/`，`fourth_run_local_constants_checks`）：根环区域数；稀疏盘的扇区宽。
const ROOT_RING_REGIONS_LOCAL: u64 = 3;
const SECTOR_BYTES_LOCAL: u64 = 512;
/// H1f（登记 5.3）：n1a ∈ {0, 1}，c ∈ {C, U}，n1b ∈ {1, 2}。
const FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES: [u64; 2] = [0, 1];
const FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES: [u64; 2] = [1, 2];
/// H1g（登记 5.3）：n1 ∈ {1, 2}，a ∈ {1, 2}，m ∈ {1, 2}。
const FOURTH_RUN_ACQUISITION_CRASH_PREFIX_OVERWRITES: [u64; 2] = [1, 2];
const FOURTH_RUN_ACQUISITION_CRASHES: [u64; 2] = [1, 2];
const FOURTH_RUN_ACQUISITION_CRASH_TAIL_OVERWRITES: [u64; 2] = [1, 2];
/// 一块盘上取号写的槽数（D23（journal 的角色与格式） 已定项 16：取号逐盘写一个系统配置槽；G0 两块盘 ⇒ 取号写 2 个）。
const ACQUISITION_WRITES_PER_DEVICE_LOCAL: usize = 1;

/// 一份起始镜像：坐标（进结果行）与装置的账（镜像在 `history.pool`，会话已关）。
struct FourthRunPrefix {
    coordinates: String,
    history: ThirdRunHistory,
}

/// P(n1, c)（第 3 次跑 5.3）：mkfs → `mount_writable` → 首个文件 → n1 次覆盖写 → 关闭 c。关闭 U 那几格的 F14 判定行另打。
fn fourth_run_plain_prefix(
    geometry: &Geometry,
    overwrites: u64,
    closing: Closing,
    arm: &str,
) -> Result<FourthRunPrefix, String> {
    let parameters = parameters_for(geometry);
    let mut history = third_run_history_start(geometry, overwrites)?;
    let coordinates = format!("n1={overwrites} c={}", closing.name());
    if let Some((holds, detail)) = third_run_close(&mut history, &parameters, closing)? {
        emit_result(&format!(
            "name=r4_section_seven_one_f14 arm={arm} geometry={} {coordinates} verdict={} {detail}",
            geometry.label,
            if holds { "pass" } else { "fail_f14" }
        ));
    }
    Ok(FourthRunPrefix {
        coordinates,
        history,
    })
}

/// 在装置的账上做一次可写挂载（不注入），把它写下的根记进时间线（沿用 E 的内容号），会话开着。
fn fourth_run_mount_writable_into_the_history(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<(), String> {
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let mounted = mount_writable(parameters, &mut devices)
        .map_err(|error| format!("前缀里的可写挂载: {error:?}"))?;
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let carried = history
        .content_number_of(root_pair(&mounted.output.effective_root))
        .unwrap_or(0);
    push_the_publishes_of_a_mount(history, &mounted, carried);
    history.session = Some(Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    });
    Ok(())
}

/// H1f 的前缀（登记 5.3 H1f 行）：mkfs → `mount_writable` → 首个文件 → n1a 次覆盖写 → 关闭 c → `mount_writable` → n1b 次覆盖写 → 关闭 C。
/// 前缀在这一臂自己的进程里造：N 为假时各臂与今天同一串写（PC-N0、F12），-配置续 第二次取号写的 tail 按臂的定义不同；
/// 各臂前缀的镜像指纹打进结果行，`r4-compare` 跨臂核（执行员修订第 7 条）。
fn fourth_run_remount_prefix(
    geometry: &Geometry,
    first_overwrites: u64,
    closing: Closing,
    second_overwrites: u64,
    arm: &str,
) -> Result<FourthRunPrefix, String> {
    let parameters = parameters_for(geometry);
    let mut history = third_run_history_start(geometry, first_overwrites)?;
    let coordinates = format!(
        "n1a={first_overwrites} c={} n1b={second_overwrites}",
        closing.name()
    );
    if let Some((holds, detail)) = third_run_close(&mut history, &parameters, closing)? {
        emit_result(&format!(
            "name=r4_section_seven_one_f14 arm={arm} geometry={} {coordinates} verdict={} {detail}",
            geometry.label,
            if holds { "pass" } else { "fail_f14" }
        ));
    }
    fourth_run_mount_writable_into_the_history(&mut history, &parameters)?;
    for _ in 0..second_overwrites {
        third_run_change_content(&mut history, &parameters)?;
    }
    third_run_close(&mut history, &parameters, Closing::ProcessExit)?;
    Ok(FourthRunPrefix {
        coordinates,
        history,
    })
}

/// H1g 的前缀（登记 5.3 H1g 行）：P(n1, C) → a 次 `mount_writable` 各自在取号写全部落盘之后、下一个写之前崩溃（在这一臂的副本上做）。
/// 交回前缀与每一次崩溃之后按装置解码核「每块盘世代号最新的系统配置槽的实例代号 = 这一次取的号」的结果（核不上 ⇒ 那一次不是崩在取号之后）。
fn fourth_run_acquisition_crash_prefix(
    geometry: &Geometry,
    overwrites: u64,
    acquisition_crashes: u64,
) -> Result<(FourthRunPrefix, Vec<String>), String> {
    let parameters = parameters_for(geometry);
    let mut history = third_run_history_start(geometry, overwrites)?;
    third_run_close(&mut history, &parameters, Closing::ProcessExit)?;
    let mut acquisition_checks = Vec::new();
    for crash in 0..acquisition_crashes {
        let call = run_fourth_run_mount_writable(&history.pool, &parameters, &[], None, None);
        let acquired = call
            .call
            .outcome
            .as_ref()
            .map_err(|error| format!("第 {crash} 次取号崩溃那次挂载被拒: {error:?}"))?
            .output
            .instance
            .0;
        let acquisition_writes = ACQUISITION_WRITES_PER_DEVICE_LOCAL * history.pool.devices.len();
        let crashed =
            image_with_the_first_writes(&history.pool, &call.call.applied, acquisition_writes);
        let geometry_of_the_pool = independent_geometry(&crashed)?;
        let decoded = device_decode(&crashed, &geometry_of_the_pool);
        let newest_instances: Vec<Option<u32>> = [0u32, 1]
            .iter()
            .map(|device| {
                decoded
                    .system_configuration_slots
                    .iter()
                    .filter(|(slot_device, _, _, _)| slot_device == device)
                    .max_by_key(|(_, generation, _, _)| *generation)
                    .map(|(_, _, _, instance)| *instance)
            })
            .collect();
        let holds = newest_instances
            .iter()
            .all(|instance| *instance == Some(acquired));
        acquisition_checks.push(format!(
            "crash{crash}:acquired={acquired}:newest_slot_instances={newest_instances:?}:holds={holds}"
        ));
        history.pool = crashed;
    }
    Ok((
        FourthRunPrefix {
            coordinates: format!("n1={overwrites} a={acquisition_crashes}"),
            history,
        },
        acquisition_checks,
    ))
}

/// 抛弃步 A 的落点（第 3 次跑 `third_run_abandonment_faults` 的推广）：当前时间线上最新 `hidden_count` 条根的根槽，
/// 加第 `hidden_count` 新的那条根那次发布按 b 取的落点（k = 2：断链落在两条里较旧那一次发布上，登记 8.2）。
/// `hidden_count` = 1 时与第 3 次跑那一份逐项相同（单测 `fourth_run_abandonment_faults_equal_the_third_run_ones_for_one_hidden_root`）。
fn fourth_run_abandonment_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    form: UnreadableForm,
    replay_break: ReplayBreak,
    duration: FaultDuration,
    hidden_count: usize,
) -> Result<Option<Vec<ThirdRunFault>>, String> {
    if history.timeline.len() < hidden_count + 1 {
        return Ok(None);
    }
    let hidden_roots: Vec<TimelineRoot> = history.timeline[history.timeline.len() - hidden_count..]
        .iter()
        .rev()
        .copied()
        .collect();
    let mut faults = Vec::new();
    for hidden in &hidden_roots {
        let (root_slot_device, root_slot_offset) =
            root_slot_of(&history.pool, geometry, *hidden)
                .ok_or_else(|| format!("被藏的根 {hidden:?} 在根环里读不出"))?;
        faults.push(ThirdRunFault {
            device: root_slot_device,
            offset: root_slot_offset,
            length: u64::from(geometry.physical_block_size),
            form,
            duration,
            label: format!("root{}", key_text(Some(*hidden))),
        });
    }
    let broken = *hidden_roots.last().expect("hidden_count ≥ 1");
    let records = records_of_publish(&history.pool, geometry, broken);
    if records.is_empty() {
        return Ok(None);
    }
    match replay_break {
        ReplayBreak::EveryRecord => {
            for (counter, record_offset_on_the_device, _) in &records {
                for record_device in [DeviceIdentity(0), DeviceIdentity(1)] {
                    faults.push(ThirdRunFault {
                        device: record_device,
                        offset: *record_offset_on_the_device,
                        length: JOURNAL_RECORD_BYTES,
                        form,
                        duration,
                        label: format!("record(counter={counter},device={})", record_device.0),
                    });
                }
            }
        }
        ReplayBreak::FirstCopyOfTheFirstNamedUnit | ReplayBreak::FirstNamedUnit => {
            let Some(named) = records
                .iter()
                .find_map(|(_, _, record)| record.named.first())
            else {
                return Ok(None);
            };
            let copies = match replay_break {
                ReplayBreak::FirstNamedUnit => named.locations.len(),
                ReplayBreak::EveryRecord | ReplayBreak::FirstCopyOfTheFirstNamedUnit => 1,
            };
            for location in named.locations.iter().take(copies) {
                faults.push(ThirdRunFault {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form,
                    duration,
                    label: format!(
                        "unit(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                });
            }
        }
    }
    Ok(Some(faults))
}

// ---------------------------------------------------------------------------
// 12.5 一族的汇总（第 4 次跑）：第 3 次跑那一份的各栏，加 Q0 去向（每一臂每一格五类）、丢写格按去向分列、F15、F17 的点名。
// ---------------------------------------------------------------------------

#[derive(Default)]
struct FourthRunFamilySummary {
    counts: ThirdRunFamilySummary,
    destinations: BTreeMap<String, u64>,
    lost_write_cells_by_destination: BTreeMap<String, u64>,
    /// 失败条款 F15（7.1 新加第一行）：今天那一臂 b = records、W 的不断格上，写行那次发布没落在被藏那条根的槽上或去向不是「根槽被盖」。
    failure_clause_fifteen_findings: Vec<String>,
    failure_clause_fifteen_checks: u64,
    /// 失败条款 F6（第四节推的 ①、③）：今天那一臂被测挂载上装置算的 N-配置 为假的格。
    failure_clause_six_findings: Vec<String>,
    failure_clause_six_checks: u64,
}

impl FourthRunFamilySummary {
    fn count(
        &mut self,
        loss: &CellLoss,
        rollback: bool,
        tested_ok: bool,
        destination: &HiddenRootDestination,
    ) {
        self.counts.count(loss, rollback, tested_ok);
        let destination_class = destination
            .name()
            .split(':')
            .next()
            .unwrap_or("")
            .to_string();
        *self
            .destinations
            .entry(destination_class.clone())
            .or_insert(0) += 1;
        if loss.lost_write_cell() || rollback {
            *self
                .lost_write_cells_by_destination
                .entry(destination_class)
                .or_insert(0) += 1;
        }
    }

    fn emit(&self, family: &str, arm: &str, geometry: &Geometry) {
        let counts = &self.counts;
        emit_result(&format!(
            "name=r4_family_summary family={family} arm={arm} geometry={} cells={} void_cells_v2={} fault_not_reached_cells={} q1_overwrite_cells={} q1_in_ring_overwrite_cells={} q1_out_of_ring_overwrite_cells={} q1_landing_overwrite_cells={} q2_rollback_cells={} tested_ok_cells={} q3_read_back={:?} q3_last_confirmed_cells={} q3_lost_cells={} q3c_tail={:?} q3c_lost_cells={} lost_write_cells_q1_q2_q3={} q0_destinations={:?} lost_write_cells_by_destination={:?} tested_classes={:?} remount_classes={:?} s4_mismatches={} f4_failures={} f6_checked={} f6_findings={} f13_checked={} f13_failures={} f15_checked={} f15_findings={} formula_mismatches={}",
            geometry.label,
            counts.cells,
            counts.void_cells,
            counts.fault_not_reached_cells,
            counts.overwrite_cells,
            counts.in_ring_overwrite_cells,
            counts.out_of_ring_overwrite_cells,
            counts.landing_overwrite_cells,
            counts.rollback_cells,
            counts.tested_ok_cells,
            counts.read_back,
            counts.read_back_last_confirmed_cells,
            counts.read_back_lost_cells,
            counts.tail,
            counts.tail_lost_cells,
            counts.lost_write_cells,
            self.destinations,
            self.lost_write_cells_by_destination,
            counts.tested_classes,
            counts.remount_classes,
            counts.stop_condition_four_mismatches.len(),
            counts.failure_clause_four_findings.len(),
            self.failure_clause_six_checks,
            self.failure_clause_six_findings.len(),
            counts.failure_clause_thirteen_checks,
            counts.failure_clause_thirteen_findings.len(),
            self.failure_clause_fifteen_checks,
            self.failure_clause_fifteen_findings.len(),
            counts.formula_mismatches.len()
        ));
        for (kind, list) in [
            ("s4", &counts.stop_condition_four_mismatches),
            ("f4", &counts.failure_clause_four_findings),
            ("f6", &self.failure_clause_six_findings),
            ("f13", &counts.failure_clause_thirteen_findings),
            ("f15", &self.failure_clause_fifteen_findings),
            ("formula", &counts.formula_mismatches),
        ] {
            for detail in list {
                emit_result(&format!(
                    "name=r4_family_finding family={family} arm={arm} geometry={} kind={kind} detail={detail:?}",
                    geometry.label
                ));
            }
        }
    }
}

/// 一族怎么跑：族名（进结果行）、几何、A 藏几条根、一个前缀之后怎么接（断点全枚举 / 只不断 / m 尾巴）。
#[derive(Clone, Copy, Debug)]
enum FourthRunContinuation {
    /// H1d 的全部断点（不断、crash、write_fails、barrier_fails）。
    EveryCut,
    /// 只跑不断那一格（8.2 的敏感性点「H1d 的 `none`」）。
    UncutOnly,
    /// H1e 的 m 尾巴（A 不断 → m 次改内容 → 关闭 C）。
    Tail,
}

struct FourthRunFamily<'plan> {
    name: &'plan str,
    geometry: &'plan Geometry,
    hidden_count: usize,
    continuation: FourthRunContinuation,
    tail_overwrites: &'plan [u64],
    durations: FourthRunDurationPlan,
    /// 今天那一臂顺带核 F6（第四节推的 ①）与 F15（7.1 新加第一行）：只在登记点名的 G0 上的 H1d、H1e 开。
    checks_the_clauses_on_the_hidden_root: ClauseChecksOnTheHiddenRoot,
}

/// A 取哪几种时长（登记 5.3：H1d 与接 H1d 断点的族按这一臂的 H1d 时长，接 H1e 尾巴的按这一臂的 H1e 时长；H1g 只 W）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FourthRunDurationPlan {
    OfTheArm,
    WholeCallOnly,
}

/// 这一族顺带核不核 F6、F15（登记第十节：F6 管 H1d、H1e；F15 管 H1d 的 `none` 格与 H1e 里 b = records、W 的格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClauseChecksOnTheHiddenRoot {
    Checked,
    NotRegisteredForThisFamily,
}

/// 被测挂载之后、断点那一刻（或 m 尾巴之后）的镜像 `post` → 撤故障 → 再可写挂载（这一臂的代码，不注入）→ 历史末尾算三样。
struct FourthRunCellContext<'cell> {
    arm: FourthRunArm,
    family: &'cell FourthRunFamily<'cell>,
    history: &'cell ThirdRunHistory,
    protected: &'cell ThirdRunConfirmation,
    base: &'cell MemoryPool,
    parameters: &'cell MakeFilesystemParameters,
    first_new_instance: u32,
}

#[allow(
    clippy::too_many_arguments,
    reason = "一格的坐标、被测挂载那一行与它的判定原样打进结果行，拆成结构体只多一层搬运"
)]
fn evaluate_fourth_run_cell(
    context: &FourthRunCellContext<'_>,
    coordinates: &str,
    post: &MemoryPool,
    tested_line: &str,
    rollback: bool,
    tested_ok: bool,
    do_the_tail: bool,
    destination: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    let remount = run_third_run_mount_writable(post, context.parameters, &[], None, None);
    let remount_class = third_run_outcome_class(&remount.outcome, remount.write_calls);
    let loss = evaluate_cell_loss(
        context.history,
        context.protected,
        context.base,
        &remount.pool_after,
        do_the_tail.then_some(context.first_new_instance),
    );
    summary.count(&loss, rollback, tested_ok, destination);
    *summary
        .counts
        .remount_classes
        .entry(remount_class.to_string())
        .or_insert(0) += 1;
    emit_result(&format!(
        "name=r4_cell family={} arm={} geometry={} {coordinates} {tested_line} q0_destination={} q2_rollback={rollback} remount_class={remount_class} remount_error={} remount_chosen={} {}",
        context.family.name,
        context.arm.name,
        context.family.geometry.label,
        destination.name(),
        mount_error_member_or_none(&remount.outcome),
        key_text(
            remount
                .outcome
                .as_ref()
                .ok()
                .map(|mounted| root_pair(&mounted.output.chosen_root))
        ),
        loss.render()
    ));
}

/// 一族在一条臂上跑完：每个前缀 × 造法 × b × 时长 → A →（断点 / m 尾巴）→ 撤故障 → 再挂载 → 三样。
fn run_fourth_run_family(
    family: &FourthRunFamily<'_>,
    arm: FourthRunArm,
    prefixes: Vec<Result<FourthRunPrefix, String>>,
) {
    let parameters = parameters_for(family.geometry);
    let mut summary = FourthRunFamilySummary::default();
    let durations: Vec<(FaultDuration, bool)> = match (family.durations, family.continuation) {
        (FourthRunDurationPlan::WholeCallOnly, _) => vec![(FaultDuration::WholeCall, true)],
        (FourthRunDurationPlan::OfTheArm, FourthRunContinuation::EveryCut) => {
            fourth_run_held_once_durations(arm)
        }
        (FourthRunDurationPlan::OfTheArm, FourthRunContinuation::UncutOnly) => {
            fourth_run_held_once_durations(arm)
                .into_iter()
                .map(|(duration, _)| (duration, true))
                .collect()
        }
        (FourthRunDurationPlan::OfTheArm, FourthRunContinuation::Tail) => {
            fourth_run_tail_durations(arm)
                .into_iter()
                .map(|duration| (duration, true))
                .collect()
        }
    };
    let mut cells_by_duration: BTreeMap<String, u64> = BTreeMap::new();
    for prefix in prefixes {
        let prefix = match prefix {
            Ok(prefix) => prefix,
            Err(error) => {
                emit_result(&format!(
                    "name=r4_prefix family={} arm={} geometry={} verdict=stop reason={error:?}",
                    family.name, arm.name, family.geometry.label
                ));
                continue;
            }
        };
        let history = &prefix.history;
        let base = history.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            emit_result(&format!(
                "name=r4_prefix family={} arm={} geometry={} {} verdict=stop reason=\"几何解不出\"",
                family.name, arm.name, family.geometry.label, prefix.coordinates
            ));
            continue;
        };
        let root_slots = root_slot_device_offsets(&pool_geometry);
        let protected = history
            .confirmations
            .last()
            .cloned()
            .expect("前缀至少确认过首个文件");
        let last_confirmed = protected.content_number;
        let hidden = history.tip();
        let hidden_position = hidden_root_position(&base, hidden);
        let base_fingerprint = image_fingerprint(&base);
        // 「不绕环」（第 3 次跑登记 5.3 G0 那一句）：最大计数器 < 环槽数。
        let highest_counter = device_decode(&base, &pool_geometry)
            .records
            .keys()
            .map(|(_, counter)| *counter)
            .max()
            .unwrap_or(0);
        let ring_not_wrapped =
            highest_counter < pool_geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
        emit_result(&format!(
            "name=r4_prefix family={} arm={} geometry={} {} tip={} hidden_slot={} last_confirmed={last_confirmed} base_fingerprint={base_fingerprint:016x} highest_counter={highest_counter} ring_not_wrapped={ring_not_wrapped} tail_checks={} tail_check_failures={}",
            family.name,
            arm.name,
            family.geometry.label,
            prefix.coordinates,
            key_text(Some(hidden)),
            hidden_position.map_or("none".to_string(), |position| format!("{}:{}", position.region, position.slot)),
            history.tail_checks,
            history.tail_check_failures.len()
        ));
        for failure in &history.tail_check_failures {
            emit_result(&format!(
                "name=r4_f3_tail_check_failure family={} arm={} {} detail={failure:?}",
                family.name, arm.name, prefix.coordinates
            ));
        }
        let context = FourthRunCellContext {
            arm,
            family,
            history,
            protected: &protected,
            base: &base,
            parameters: &parameters,
            first_new_instance: hidden.0 + 1,
        };
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                for (duration, only_the_uncut_cell) in &durations {
                    let head = format!(
                        "{} b={} form={} duration={}",
                        prefix.coordinates,
                        replay_break.name(),
                        form.name(),
                        duration.name()
                    );
                    let faults = match fourth_run_abandonment_faults(
                        history,
                        &pool_geometry,
                        form,
                        replay_break,
                        *duration,
                        family.hidden_count,
                    ) {
                        Ok(Some(faults)) => faults,
                        Ok(None) => {
                            emit_result(&format!(
                                "name=r4_group family={} arm={} geometry={} {head} verdict=not_applicable",
                                family.name, arm.name, family.geometry.label
                            ));
                            continue;
                        }
                        Err(error) => {
                            emit_result(&format!(
                                "name=r4_group family={} arm={} geometry={} {head} verdict=stop reason={error:?}",
                                family.name, arm.name, family.geometry.label
                            ));
                            continue;
                        }
                    };
                    let Ok(tested) = run_fourth_run_tested_mount(
                        arm,
                        &base,
                        &parameters,
                        &faults,
                        &head,
                        &mut summary.counts,
                    ) else {
                        emit_result(&format!(
                            "name=r4_group family={} arm={} geometry={} {head} verdict=stop reason=\"几何解不出\"",
                            family.name, arm.name, family.geometry.label
                        ));
                        continue;
                    };
                    let tested_ok = tested.call.call.outcome.is_ok();
                    let mounted = tested.call.call.outcome.as_ref().ok();
                    let rollback = tested_ok
                        && tested
                            .summary
                            .effective
                            .and_then(|effective| history.content_number_of(effective))
                            .is_some_and(|number| number < last_confirmed);
                    let content_numbers = content_numbers_with_the_publishes_of(history, mounted);
                    let destination_after_the_tested_mount = hidden_position.map_or_else(
                        || HiddenRootDestination::Other("hidden_root_not_in_base".to_string()),
                        |position| {
                            hidden_root_destination(
                                &position,
                                &content_numbers,
                                last_confirmed,
                                &tested.call.call.pool_after,
                                tested_ok.then_some(tested.summary.effective).flatten(),
                            )
                        },
                    );
                    if family.checks_the_clauses_on_the_hidden_root
                        == ClauseChecksOnTheHiddenRoot::Checked
                    {
                        fourth_run_family_cross_checks(
                            arm,
                            &head,
                            replay_break,
                            *duration,
                            history,
                            &tested,
                            hidden_position,
                            family.geometry,
                            &destination_after_the_tested_mount,
                            &mut summary,
                        );
                    }
                    let not_reached = !tested.unintercepted.is_empty();
                    if not_reached {
                        if arm.is_today() {
                            summary.counts.void_cells += 1;
                        } else {
                            summary.counts.fault_not_reached_cells += 1;
                        }
                    }
                    *summary
                        .counts
                        .tested_classes
                        .entry(format!(
                            "{}:{}",
                            tested.summary.class, tested.summary.member
                        ))
                        .or_insert(0) += 1;
                    let tested_line = format!(
                        "{} hidden={} base_fingerprint={base_fingerprint:016x} faults={} unintercepted={}",
                        render_fourth_run_tested_mount(&tested),
                        key_text(Some(hidden)),
                        faults.len(),
                        tested.unintercepted.join(",")
                    );
                    if arm.is_today() && not_reached {
                        emit_result(&format!(
                            "name=r4_cell family={} arm={} geometry={} {head} cut=none point=none verdict=void_v2 {tested_line}",
                            family.name, arm.name, family.geometry.label
                        ));
                        continue;
                    }
                    *cells_by_duration.entry(duration.name()).or_insert(0) += 1;
                    match family.continuation {
                        FourthRunContinuation::Tail => run_fourth_run_tail_cells(
                            &context,
                            &head,
                            &tested,
                            &tested_line,
                            rollback,
                            &destination_after_the_tested_mount,
                            &mut summary,
                        ),
                        FourthRunContinuation::EveryCut | FourthRunContinuation::UncutOnly => {
                            run_fourth_run_cut_cells(
                                &context,
                                &head,
                                &faults,
                                &tested,
                                &tested_line,
                                rollback,
                                *only_the_uncut_cell,
                                &root_slots,
                                &pool_geometry,
                                hidden_position,
                                &destination_after_the_tested_mount,
                                &mut summary,
                            );
                        }
                    }
                }
            }
        }
    }
    for (duration, groups) in &cells_by_duration {
        emit_result(&format!(
            "name=r4_family_groups family={} arm={} geometry={} duration={duration} groups={groups}",
            family.name, arm.name, family.geometry.label
        ));
        // 7.2：H1e 每种时长 4 × 2 × 2 × 4 格；H1g 每臂 n1 × a × m × b × 造法 = 32 格（只 W）。
        let registered = match (family.continuation, family.name) {
            (FourthRunContinuation::Tail, "h1e") if family.hidden_count == 1 => Some(4 * 2 * 2 * 4),
            (FourthRunContinuation::Tail, "h1g") => Some(2 * 2 * 2 * 2 * 2),
            (
                FourthRunContinuation::Tail
                | FourthRunContinuation::EveryCut
                | FourthRunContinuation::UncutOnly,
                _,
            ) => None,
        };
        if let Some(registered) = registered {
            let cells = groups * family.tail_overwrites.len() as u64;
            emit_result(&format!(
                "name=r4_section_seven_two_anchor arm={} geometry={} item=\"{} 每种时长的格数\" duration={duration} computed={cells} registered={registered} verdict={}",
                arm.name,
                family.geometry.label,
                family.name,
                if cells == registered { "pass" } else { "fail_v4" }
            ));
        }
    }
    summary.emit(family.name, arm.name, family.geometry);
}

/// 一组（一个前缀 × 造法 × b × 时长）在 H1d 那样的断点上全枚举：不断、crash j（前 j 个写落盘）、write_fails j、barrier_fails j。
#[allow(
    clippy::too_many_arguments,
    reason = "一组的上下文、被测挂载与它的判定原样传给每一格，拆成结构体只多一层搬运"
)]
fn run_fourth_run_cut_cells(
    context: &FourthRunCellContext<'_>,
    head: &str,
    faults: &[ThirdRunFault],
    tested: &FourthRunTestedMount,
    tested_line: &str,
    rollback: bool,
    only_the_uncut_cell: bool,
    root_slots: &[(DeviceIdentity, u64)],
    pool_geometry: &PoolGeometry,
    hidden_position: Option<HiddenRootPosition>,
    destination_after_the_tested_mount: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    let arm = context.arm;
    let history = context.history;
    let last_confirmed = context.protected.content_number;
    let tested_ok = tested.call.call.outcome.is_ok();
    let first_root_write = index_of_the_first_root_slot_write(
        &tested.call.call.applied,
        root_slots,
        u64::from(pool_geometry.physical_block_size),
    );
    let barriers = first_root_write.map_or(0, |index| {
        barriers_before_write(&tested.call.call.applied, index + 1)
    });
    let cut_cells_expected = if only_the_uncut_cell {
        1
    } else {
        first_root_write.map_or(1, |index| 1 + 2 * (index as u64 + 1) + barriers)
    };
    let destination_of =
        |pool: &MemoryPool, ok_effective: Option<TimelineRoot>, mounted: Option<&Mounted>| {
            hidden_position.map_or_else(
                || HiddenRootDestination::Other("hidden_root_not_in_base".to_string()),
                |position| {
                    hidden_root_destination(
                        &position,
                        &content_numbers_with_the_publishes_of(history, mounted),
                        last_confirmed,
                        pool,
                        ok_effective,
                    )
                },
            )
        };
    let mut enumerated = 0u64;
    evaluate_fourth_run_cell(
        context,
        &format!("{head} cut=none point=none"),
        &tested.call.call.pool_after,
        &format!(
            "{tested_line} first_root_write={} barriers_before_the_row_root={barriers}",
            first_root_write.map_or("none".to_string(), |index| index.to_string())
        ),
        rollback,
        tested_ok,
        true,
        destination_after_the_tested_mount,
        summary,
    );
    enumerated += 1;
    if !only_the_uncut_cell {
        if let Some(first_root_write) = first_root_write {
            let tested_mounted = tested.call.call.outcome.as_ref().ok();
            for write_count in 0..=first_root_write {
                let post = image_with_the_first_writes(
                    context.base,
                    &tested.call.call.applied,
                    write_count,
                );
                let destination = destination_of(&post, None, tested_mounted);
                evaluate_fourth_run_cell(
                    context,
                    &format!("{head} cut=crash point={write_count}"),
                    &post,
                    "a_outcome=crashed",
                    false,
                    false,
                    false,
                    &destination,
                    summary,
                );
                enumerated += 1;
            }
            for (cut, failing_write, failing_barrier) in (0..=first_root_write)
                .map(|index| ("write_fails", Some(index as u64), None))
                .chain((0..barriers).map(|index| ("barrier_fails", None, Some(index))))
            {
                let failed = run_fourth_run_mount_writable(
                    context.base,
                    context.parameters,
                    faults,
                    failing_write,
                    failing_barrier,
                );
                let failed_summary = TestedMountSummary::of(&failed.call);
                let failed_ok = failed.call.outcome.is_ok();
                let failed_rollback = failed_ok
                    && failed_summary
                        .effective
                        .and_then(|effective| history.content_number_of(effective))
                        .is_some_and(|number| number < last_confirmed);
                let failed_mounted = failed.call.outcome.as_ref().ok();
                let destination = destination_of(
                    &failed.call.pool_after,
                    failed_ok.then_some(failed_summary.effective).flatten(),
                    failed_mounted,
                );
                evaluate_fourth_run_cell(
                    context,
                    &format!(
                        "{head} cut={cut} point={}",
                        failing_write.or(failing_barrier).unwrap_or(0)
                    ),
                    &failed.call.pool_after,
                    &format!(
                        "{} {} fired={}",
                        failed_summary.render("a"),
                        failed.render_marker("a"),
                        failed.call.injected_failure_fired
                    ),
                    failed_rollback,
                    failed_ok,
                    false,
                    &destination,
                    summary,
                );
                enumerated += 1;
            }
        }
    }
    if enumerated != cut_cells_expected {
        summary.counts.formula_mismatches.push(format!(
            "{head}: 枚举 {enumerated} 格，公式 1 + 2(J + 1) + 屏障数 = {cut_cells_expected}"
        ));
    }
    emit_result(&format!(
        "name=r4_group family={} arm={} geometry={} {head} cells={enumerated} formula_cells={cut_cells_expected} verdict={}",
        context.family.name,
        arm.name,
        context.family.geometry.label,
        if enumerated == cut_cells_expected { "pass" } else { "fail_v4" }
    ));
}

/// 一组接 H1e 的 m 尾巴：A 交回的会话里再做 m 次改内容 → 关闭 C → 撤故障 → 再挂载 → 三样（A 被拒的臂记「因拒没做」）。
#[allow(
    clippy::too_many_arguments,
    reason = "一组的上下文、被测挂载与它的判定原样传给每一格，拆成结构体只多一层搬运"
)]
fn run_fourth_run_tail_cells(
    context: &FourthRunCellContext<'_>,
    head: &str,
    tested: &FourthRunTestedMount,
    tested_line: &str,
    rollback: bool,
    destination_after_the_tested_mount: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    let tested_ok = tested.call.call.outcome.is_ok();
    for tail_overwrites in context.family.tail_overwrites {
        let mut history = context.history.clone();
        history.pool = tested.call.call.pool_after.clone();
        let tail_writes = match &tested.call.call.outcome {
            Ok(mounted) => {
                let effective_content = history
                    .content_number_of(root_pair(&mounted.output.effective_root))
                    .unwrap_or(0);
                push_the_publishes_of_a_mount(&mut history, mounted, effective_content);
                history.session = Some(Session {
                    allocator: mounted.allocator.clone(),
                    current: mounted.current.clone(),
                    instance: mounted.output.instance,
                });
                let mut done = 0u64;
                let mut failure = None;
                for _ in 0..*tail_overwrites {
                    match third_run_change_content(&mut history, context.parameters) {
                        Ok(()) => done += 1,
                        Err(error) => {
                            failure = Some(error);
                            break;
                        }
                    }
                }
                match failure {
                    None => format!("done:{done}"),
                    Some(error) => format!("failed_after:{done}:{}", error_member_of_debug(&error)),
                }
            }
            Err(_) => "refused_not_done".to_string(),
        };
        let _ = third_run_close(&mut history, context.parameters, Closing::ProcessExit);
        let tail_context = FourthRunCellContext {
            arm: context.arm,
            family: context.family,
            history: &history,
            protected: context.protected,
            base: context.base,
            parameters: context.parameters,
            first_new_instance: context.first_new_instance,
        };
        evaluate_fourth_run_cell(
            &tail_context,
            &format!("{head} m={tail_overwrites}"),
            &history.pool.clone(),
            &format!(
                "{tested_line} tail_writes={tail_writes} last_confirmed_after_the_tail={}",
                history.last_confirmed_content_number()
            ),
            rollback,
            tested_ok,
            true,
            destination_after_the_tested_mount,
            summary,
        );
    }
}

/// 一组的被测挂载上顺带核的条款（今天那一臂）：F4（7.1）、F6（第四节推的 ①）、F15（7.1 新加第一行）；Q0「被抛弃」那一栏。
#[allow(
    clippy::too_many_arguments,
    reason = "一组的坐标、被测挂载与装置的判定原样传进来，拆成结构体只多一层搬运"
)]
fn fourth_run_family_cross_checks(
    arm: FourthRunArm,
    head: &str,
    replay_break: ReplayBreak,
    duration: FaultDuration,
    history: &ThirdRunHistory,
    tested: &FourthRunTestedMount,
    hidden_position: Option<HiddenRootPosition>,
    geometry: &Geometry,
    destination: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    if !arm.is_today() {
        return;
    }
    let tested_ok = tested.call.call.outcome.is_ok();
    if *destination == HiddenRootDestination::InRingAndAbandoned {
        summary.counts.abandoned_hidden_root_cells += 1;
    }
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    if tested_ok
        && replay_break == ReplayBreak::FirstCopyOfTheFirstNamedUnit
        && duration == FaultDuration::WholeCall
        && tested.summary.effective != before_hidden
    {
        summary.counts.failure_clause_four_findings.push(format!(
            "{head}: E {}，被藏那条根前一条 {}",
            key_text(tested.summary.effective),
            key_text(before_hidden)
        ));
    }
    summary.failure_clause_six_checks += 1;
    if !tested.judgement.configuration_newer {
        summary
            .failure_clause_six_findings
            .push(format!("{head}: 装置在今天那一臂上算的 N-配置 为假"));
    }
    if let (true, ReplayBreak::EveryRecord, FaultDuration::WholeCall, Ok(mounted), Some(position)) = (
        tested_ok,
        replay_break,
        duration,
        &tested.call.call.outcome,
        hidden_position,
    ) {
        summary.failure_clause_fifteen_checks += 1;
        let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
        let predicted = predicted_root_ring_slot_of_a_publish(row_txg, geometry.slots_per_region);
        let holds = row_txg == position.key.1
            && predicted == (position.region, position.slot)
            && *destination == HiddenRootDestination::RootSlotOverwritten;
        if !holds {
            summary.failure_clause_fifteen_findings.push(format!(
                "{head}: 写行那次发布 txg {row_txg}（落在 {predicted:?}），被藏那条根 txg {}、s_h {:?}，去向 {}",
                position.key.1,
                (position.region, position.slot),
                destination.name()
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// 12.6 第一段的阳性对照（登记 5.4，每一臂都跑）：PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-只读、PC-N0（L0 那 8 格）。
//      「与今天逐项相同」「PC-多读」这几句要跨臂，在 `r4-compare` 里判；这里把它们要的字段打进结果行。
// ---------------------------------------------------------------------------

/// 一格被测挂载跑完整段：A → 撤故障 → `mount_writable`（不注入）→ 冷启动 `recover` 读回，交回 A、去向、回滚与三样。
struct FourthRunControlCell {
    tested: FourthRunTestedMount,
    destination: HiddenRootDestination,
    rollback: bool,
    loss: CellLoss,
    remount_line: String,
    write_fingerprint: u64,
}

fn run_fourth_run_control_cell(
    arm: FourthRunArm,
    history: &ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    label: &str,
    scratch: &mut ThirdRunFamilySummary,
) -> Result<FourthRunControlCell, String> {
    let base = history.pool.clone();
    let tested = run_fourth_run_tested_mount(arm, &base, parameters, faults, label, scratch)?;
    let protected = history
        .confirmations
        .last()
        .cloned()
        .ok_or("前缀没有确认过")?;
    let hidden = history.tip();
    let tested_ok = tested.call.call.outcome.is_ok();
    let mounted = tested.call.call.outcome.as_ref().ok();
    let rollback = tested_ok
        && tested
            .summary
            .effective
            .and_then(|effective| history.content_number_of(effective))
            .is_some_and(|number| number < protected.content_number);
    let destination = hidden_root_position(&base, hidden).map_or_else(
        || HiddenRootDestination::Other("hidden_root_not_in_base".to_string()),
        |position| {
            hidden_root_destination(
                &position,
                &content_numbers_with_the_publishes_of(history, mounted),
                protected.content_number,
                &tested.call.call.pool_after,
                tested_ok.then_some(tested.summary.effective).flatten(),
            )
        },
    );
    let remount =
        run_third_run_mount_writable(&tested.call.call.pool_after, parameters, &[], None, None);
    let loss = evaluate_cell_loss(
        history,
        &protected,
        &base,
        &remount.pool_after,
        Some(hidden.0 + 1),
    );
    let remount_line = format!(
        "remount_class={} remount_error={}",
        third_run_outcome_class(&remount.outcome, remount.write_calls),
        mount_error_member_or_none(&remount.outcome)
    );
    let (write_fingerprint, _) = write_fingerprint_of(&tested.call.call.applied, 2);
    Ok(FourthRunControlCell {
        tested,
        destination,
        rollback,
        loss,
        remount_line,
        write_fingerprint,
    })
}

/// 两格被测挂载在 7.3 第三行（V5）意义下逐项相同：K 类、错误成员、写次数、读次数、所选根、E、去向、读回、覆盖格。
fn cells_agree_item_by_item(first: &FourthRunControlCell, second: &FourthRunControlCell) -> bool {
    first.tested.summary.class == second.tested.summary.class
        && first.tested.summary.member == second.tested.summary.member
        && first.tested.summary.writes == second.tested.summary.writes
        && first.tested.summary.reads == second.tested.summary.reads
        && first.tested.summary.chosen == second.tested.summary.chosen
        && first.tested.summary.effective == second.tested.summary.effective
        && first.destination == second.destination
        && first.loss.read_back == second.loss.read_back
        && first.loss.overwrite_cell() == second.loss.overwrite_cell()
}

fn refused_before_any_write(cell: &FourthRunControlCell, arm: FourthRunArm) -> bool {
    cell.tested.summary.member == arm.refusal_member() && cell.tested.summary.writes == 0
}

/// 这一臂在这一格上 N 判假（臂的观测：判了 N、第一遍为假）；-槽 臂在读回全零上要这一句（登记 5.4）。
fn witness_judged_false(cell: &FourthRunControlCell) -> bool {
    cell.tested
        .summary
        .observation
        .as_ref()
        .is_some_and(|observation| {
            observation.witness_judged && !observation.newer_root_on_the_first_pass
        })
}

/// PC-N-环（b = `unit_first_copy`）与 PC-N-盖（b = `records`）：P(1, C) → A（`read_fails`、`reads_zeros` × W、T(δ)）→ 撤故障 →
/// `mount_writable`（不注入）→ 冷启动 `recover` 读回。每臂 4 格；W 格上拒了的做 PC-只读；PC-N-环 的 `read_fails`、W 格做 PC-丢写。
fn run_fourth_run_positive_control_newer(
    geometry: &Geometry,
    arm: FourthRunArm,
    replay_break: ReplayBreak,
) {
    let control = match replay_break {
        ReplayBreak::FirstCopyOfTheFirstNamedUnit => "PC-N-ring",
        ReplayBreak::EveryRecord | ReplayBreak::FirstNamedUnit => "PC-N-cover",
    };
    let parameters = parameters_for(geometry);
    let history = match fourth_run_plain_prefix(geometry, 1, Closing::ProcessExit, arm.name) {
        Ok(prefix) => prefix.history,
        Err(error) => {
            emit_fourth_run_positive_control(
                arm.name,
                control,
                "form=none duration=none",
                &[sentence(
                    SentenceTag::Measured,
                    "prefix",
                    SentenceVerdict::NotConstructible,
                    "V1",
                )],
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let Ok(pool_geometry) = independent_geometry(&history.pool) else {
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let hidden_position = hidden_root_position(&history.pool, hidden);
    let mut cells_constructed = 0u64;
    for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
        let mut by_duration: Vec<(FaultDuration, FourthRunControlCell, Vec<ThirdRunFault>)> =
            Vec::new();
        for duration in [
            FaultDuration::WholeCall,
            FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
        ] {
            let coordinates = format!("form={} duration={}", form.name(), duration.name());
            let faults = match fourth_run_abandonment_faults(
                &history,
                &pool_geometry,
                form,
                replay_break,
                duration,
                1,
            ) {
                Ok(Some(faults)) => faults,
                Ok(None) | Err(_) => {
                    emit_fourth_run_positive_control(
                        arm.name,
                        control,
                        &coordinates,
                        &[sentence(
                            SentenceTag::Measured,
                            "faults",
                            SentenceVerdict::NotConstructible,
                            "V1",
                        )],
                        "reason=\"落点造不出\"",
                    );
                    continue;
                }
            };
            let mut scratch = ThirdRunFamilySummary::default();
            match run_fourth_run_control_cell(
                arm,
                &history,
                &parameters,
                &faults,
                control,
                &mut scratch,
            ) {
                Ok(cell) => {
                    emit_fourth_run_scratch_findings(control, arm.name, &scratch);
                    by_duration.push((duration, cell, faults));
                    cells_constructed += 1;
                }
                Err(error) => emit_fourth_run_positive_control(
                    arm.name,
                    control,
                    &coordinates,
                    &[sentence(
                        SentenceTag::Measured,
                        "cell",
                        SentenceVerdict::NotConstructible,
                        "V1",
                    )],
                    &format!("reason={error:?}"),
                ),
            }
        }
        let whole_call_cell = by_duration
            .iter()
            .find(|(duration, _, _)| *duration == FaultDuration::WholeCall)
            .map(|(_, cell, _)| cell);
        for (duration, cell, faults) in &by_duration {
            let coordinates = format!("form={} duration={}", form.name(), duration.name());
            let sentences = sentences_of_the_newer_root_controls(
                arm,
                replay_break,
                form,
                *duration,
                cell,
                whole_call_cell,
                hidden,
                before_hidden,
                hidden_position,
                geometry,
                device_effective_content_is_older_than_the_hidden_one(&history, cell),
            );
            let row_publish = cell
                .tested
                .call
                .call
                .outcome
                .as_ref()
                .ok()
                .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
            emit_fourth_run_positive_control(
                arm.name,
                control,
                &coordinates,
                &sentences,
                &format!(
                    "faults={} hidden={} hidden_slot={} row_publish_txg={} row_publish_slot={} q0_destination={} q2_rollback={} write_fingerprint={:016x} {} {} {} unintercepted={}",
                    faults.len(),
                    key_text(Some(hidden)),
                    hidden_position.map_or("none".to_string(), |position| format!("{}:{}", position.region, position.slot)),
                    row_publish.map_or("none".to_string(), |txg| txg.to_string()),
                    row_publish.map_or("none".to_string(), |txg| {
                        let (region, slot) = predicted_root_ring_slot_of_a_publish(txg, geometry.slots_per_region);
                        format!("{region}:{slot}")
                    }),
                    cell.destination.name(),
                    cell.rollback,
                    cell.write_fingerprint,
                    render_fourth_run_tested_mount(&cell.tested),
                    cell.remount_line,
                    cell.loss.render(),
                    cell.tested.unintercepted.join(",")
                ),
            );
            if *duration != FaultDuration::WholeCall {
                continue;
            }
            run_fourth_run_read_only_control(arm, control, form, &history, cell, faults);
            if replay_break == ReplayBreak::FirstCopyOfTheFirstNamedUnit
                && form == UnreadableForm::ReadFails
            {
                run_fourth_run_lost_write_control(arm, &history, &parameters, cell);
            }
        }
    }
    let registered = 2 * 2;
    emit_result(&format!(
        "name=r4_section_seven_two_anchor arm={} item=\"{control} 每臂 2 种造法 × 2 种时长\" computed={cells_constructed} registered={registered} verdict={}",
        arm.name,
        if cells_constructed == registered { "pass" } else { "fail_v4" }
    ));
}

/// 装置自己重放出的 E_故障 的内容号小于被藏那一版的（PC-N-环、PC-N-盖 里「若今天交回 Ok 且 E 的内容号 < 被藏那一版的」的前件，
/// 取装置的 E、不取 `crates/` 交回的，免得前件与后件（Q2，按 `crates/` 的 E 算）是同一个式子）。
fn device_effective_content_is_older_than_the_hidden_one(
    history: &ThirdRunHistory,
    cell: &FourthRunControlCell,
) -> bool {
    let hidden_content = history.last_confirmed_content_number();
    cell.tested
        .judgement
        .effective
        .and_then(|effective| history.content_number_of(effective))
        .is_some_and(|number| number < hidden_content)
}

/// PC-N-环、PC-N-盖 每一格每一臂要出的句子（登记 5.4 那两行，逐句带标签）。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的坐标、这一格与同一造法的 W 格、被藏那条根的三样原样传进来，拆成结构体只多一层搬运"
)]
fn sentences_of_the_newer_root_controls(
    arm: FourthRunArm,
    replay_break: ReplayBreak,
    form: UnreadableForm,
    duration: FaultDuration,
    cell: &FourthRunControlCell,
    whole_call_cell: Option<&FourthRunControlCell>,
    hidden: TimelineRoot,
    before_hidden: Option<TimelineRoot>,
    hidden_position: Option<HiddenRootPosition>,
    geometry: &Geometry,
    device_effective_is_older: bool,
) -> Vec<ControlSentence> {
    let summary = &cell.tested.summary;
    let ok = cell.tested.call.call.outcome.is_ok();
    let is_whole_call = duration == FaultDuration::WholeCall;
    let cover = replay_break == ReplayBreak::EveryRecord;
    let mut sentences = vec![sentence(
        SentenceTag::Measured,
        "truth_newer",
        SentenceVerdict::of(cell.tested.judgement.truth_newer),
        "V1",
    )];
    let timed_equals_whole_call = || {
        sentence(
            SentenceTag::Measured,
            "timed_equals_whole_call",
            whole_call_cell.map_or(SentenceVerdict::NotConstructible, |whole| {
                SentenceVerdict::of(cells_agree_item_by_item(cell, whole))
            }),
            "V5",
        )
    };
    let refused_before_writing_then_left_as_it_was = || {
        sentence(
            SentenceTag::Measured,
            "if_k3_then_no_rollback_and_left_as_it_was",
            SentenceVerdict::conditional(
                cell.tested.summary.class == "K3",
                !cell.rollback && cell.destination == HiddenRootDestination::LeftAsItWas,
            ),
            "V1",
        )
    };
    let slot_arm_on_zeros = arm.witness == FourthRunWitness::UnreadableRootRingSlot
        && form == UnreadableForm::ReadsZeros;
    match arm.candidate {
        FourthRunCandidate::Today => {
            sentences.push(sentence(
                SentenceTag::TodayDescription,
                "ok_k0_effective_before_hidden",
                SentenceVerdict::of(summary.class == "K0" && summary.effective == before_hidden),
                if cover { "F15" } else { "F4" },
            ));
            if cover {
                let row_txg = cell
                    .tested
                    .call
                    .call
                    .outcome
                    .as_ref()
                    .ok()
                    .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "row_publish_txg_equals_hidden_txg",
                    SentenceVerdict::of(row_txg == Some(hidden.1)),
                    "F15",
                ));
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "row_publish_lands_on_the_hidden_slot",
                    SentenceVerdict::of(row_txg.is_some_and(|txg| {
                        hidden_position.is_some_and(|position| {
                            predicted_root_ring_slot_of_a_publish(txg, geometry.slots_per_region)
                                == (position.region, position.slot)
                        })
                    })),
                    "F15",
                ));
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "destination_root_slot_overwritten",
                    SentenceVerdict::of(
                        cell.destination == HiddenRootDestination::RootSlotOverwritten,
                    ),
                    "F15",
                ));
                sentences.push(sentence(
                    SentenceTag::Measured,
                    "if_overwritten_then_reads_back_older_and_counts_as_lost",
                    SentenceVerdict::conditional(
                        cell.destination == HiddenRootDestination::RootSlotOverwritten,
                        cell.loss.read_back.starts_with("older") && cell.loss.lost_write_cell(),
                    ),
                    "V1",
                ));
            } else {
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "hidden_in_ring_and_abandoned",
                    SentenceVerdict::of(
                        cell.destination == HiddenRootDestination::InRingAndAbandoned,
                    ),
                    "F17",
                ));
            }
            sentences.push(sentence(
                SentenceTag::Measured,
                "if_ok_and_older_then_rollback_one",
                SentenceVerdict::conditional(ok && device_effective_is_older, cell.rollback),
                "V1",
            ));
            if !is_whole_call {
                sentences.push(timed_equals_whole_call());
            }
        }
        FourthRunCandidate::Jia => {
            if slot_arm_on_zeros {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "slot_witness_false_on_zeros",
                    SentenceVerdict::of(witness_judged_false(cell)),
                    "F12",
                ));
            } else {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "refused_k3",
                    SentenceVerdict::of(refused_before_any_write(cell, arm)),
                    "F9",
                ));
                sentences.push(refused_before_writing_then_left_as_it_was());
            }
            if !is_whole_call {
                sentences.push(timed_equals_whole_call());
            }
        }
        FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => {
            if slot_arm_on_zeros {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "slot_witness_false_on_zeros",
                    SentenceVerdict::of(witness_judged_false(cell)),
                    "F12",
                ));
            } else if is_whole_call {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "refused_after_r_rounds_k3",
                    SentenceVerdict::of(
                        refused_before_any_write(cell, arm) && summary.waits == arm.reread_rounds,
                    ),
                    "F9",
                ));
            } else {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "ok_after_one_round_chosen_hidden_no_rollback",
                    SentenceVerdict::of(
                        ok && summary.chosen == Some(hidden)
                            && !cell.rollback
                            && summary.waits == 1,
                    ),
                    "F16",
                ));
                if cover {
                    let row_txg = cell
                        .tested
                        .call
                        .call
                        .outcome
                        .as_ref()
                        .ok()
                        .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
                    sentences.push(sentence(
                        SentenceTag::ArmDefinition,
                        "row_publish_txg_is_hidden_txg_plus_one",
                        SentenceVerdict::of(row_txg == Some(hidden.1 + 1)),
                        "F16",
                    ));
                    sentences.push(sentence(
                        SentenceTag::ArmDefinition,
                        "destination_chosen_or_applied",
                        SentenceVerdict::of(
                            cell.destination == HiddenRootDestination::ChosenOrApplied,
                        ),
                        "F16",
                    ));
                }
            }
        }
        FourthRunCandidate::Bing => {
            if is_whole_call {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "refused_k3",
                    SentenceVerdict::of(refused_before_any_write(cell, arm)),
                    "F9",
                ));
            } else {
                sentences.push(timed_equals_whole_call());
            }
        }
    }
    sentences
}

/// PC-只读（登记 5.4）：W 格上拒了的每一臂，同一组故障下调冷启动 `recover`，交得出结局，读回 E_故障 那一版。
fn run_fourth_run_read_only_control(
    arm: FourthRunArm,
    control: &str,
    form: UnreadableForm,
    history: &ThirdRunHistory,
    cell: &FourthRunControlCell,
    faults: &[ThirdRunFault],
) {
    let coordinates = format!("of={control} form={}", form.name());
    if cell.tested.call.call.outcome.is_ok() {
        emit_result(&format!(
            "name=r4_positive_control arm={} control=PC-read-only {coordinates} verdict=not_applicable reason=\"这一臂在这一格上没拒\"",
            arm.name
        ));
        return;
    }
    let read_only = run_third_run_recover(&history.pool, faults);
    let expected = cell
        .tested
        .judgement
        .effective
        .and_then(|effective| history.content_number_of(effective));
    let got = match &read_only.outcome.outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            content_number_of_bytes(content, history.last_confirmed_content_number())
        }
        RecoveryOutcome::NoFile { .. } => Some(0),
        RecoveryOutcome::Failed { .. } => None,
    };
    emit_fourth_run_positive_control(
        arm.name,
        "PC-read-only",
        &coordinates,
        &[sentence(
            SentenceTag::ArmDefinition,
            "recover_reads_back_the_version_under_the_faults",
            SentenceVerdict::of(expected.is_some() && got == expected),
            "F9",
        )],
        &format!(
            "expected_content={expected:?} got_content={got:?} landing={} e_under_faults={}",
            key_text(chosen_root_of(&read_only.outcome.outcome)),
            key_text(cell.tested.judgement.effective)
        ),
    );
}

/// PC-丢写（登记 5.4）：PC-N-环 那段历史（`read_fails`、W）走完之后、最后那次 `mount_writable` 之前，
/// 把被藏那一版的数据单元两份都改写成别的字节；每一臂：Q1 覆盖 > 0，Q3 读回不是最后确认那一版。
fn run_fourth_run_lost_write_control(
    arm: FourthRunArm,
    history: &ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
    cell: &FourthRunControlCell,
) {
    let protected = history.confirmations.last().cloned().expect("确认过");
    let (corrupted, loss) = positive_control_lost_write(
        history,
        &protected,
        &history.pool,
        &cell.tested.call.call.pool_after,
        parameters,
    );
    let verdict = if corrupted == 0 {
        SentenceVerdict::NotConstructible
    } else {
        SentenceVerdict::of(loss.overwrite_cell() && loss.read_back != "last_confirmed")
    };
    emit_fourth_run_positive_control(
        arm.name,
        "PC-lost-write",
        "form=read_fails duration=W",
        &[sentence(
            SentenceTag::Measured,
            "overwrite_counted_and_not_last_confirmed",
            verdict,
            "V1",
        )],
        &format!("corrupted_copies={corrupted} {}", loss.render()),
    );
}

/// PC-O（第 3 次跑登记 5.4 PC-O 行，标签照第 4 次跑 5.4）：P(1, C) → A（b = `unit_first_copy`、`read_fails`、O(1 只)）。
fn run_fourth_run_positive_control_second_read(geometry: &Geometry, arm: FourthRunArm) {
    let parameters = parameters_for(geometry);
    let history = match fourth_run_plain_prefix(geometry, 1, Closing::ProcessExit, arm.name) {
        Ok(prefix) => prefix.history,
        Err(error) => {
            emit_fourth_run_positive_control(
                arm.name,
                "PC-O",
                "duration=O1only",
                &[sentence(
                    SentenceTag::Measured,
                    "prefix",
                    SentenceVerdict::NotConstructible,
                    "V1",
                )],
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let Ok(pool_geometry) = independent_geometry(&history.pool) else {
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let Ok(Some(faults)) = fourth_run_abandonment_faults(
        &history,
        &pool_geometry,
        UnreadableForm::ReadFails,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        FaultDuration::OnlyTheNthRead(1),
        1,
    ) else {
        emit_fourth_run_positive_control(
            arm.name,
            "PC-O",
            "duration=O1only",
            &[sentence(
                SentenceTag::Measured,
                "faults",
                SentenceVerdict::NotConstructible,
                "V1",
            )],
            "reason=\"落点造不出\"",
        );
        return;
    };
    let mut scratch = ThirdRunFamilySummary::default();
    let Ok(cell) =
        run_fourth_run_control_cell(arm, &history, &parameters, &faults, "PC-O", &mut scratch)
    else {
        return;
    };
    emit_fourth_run_scratch_findings("PC-O", arm.name, &scratch);
    let summary = &cell.tested.summary;
    let ok = cell.tested.call.call.outcome.is_ok();
    let mut sentences = vec![sentence(
        SentenceTag::Measured,
        "truth_newer",
        SentenceVerdict::of(cell.tested.judgement.truth_newer),
        "V1",
    )];
    if arm.is_today() {
        sentences.push(sentence(
            SentenceTag::Measured,
            "faults_reached",
            SentenceVerdict::of(cell.tested.unintercepted.is_empty()),
            "V2",
        ));
    }
    sentences.push(match arm.candidate {
        FourthRunCandidate::Today => sentence(
            SentenceTag::TodayDescription,
            "ok_effective_before_hidden",
            SentenceVerdict::of(ok && summary.effective == before_hidden),
            "F4",
        ),
        FourthRunCandidate::Jia => sentence(
            SentenceTag::ArmDefinition,
            "refused_k3",
            SentenceVerdict::of(refused_before_any_write(&cell, arm)),
            "F9",
        ),
        FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => sentence(
            SentenceTag::ArmDefinition,
            "ok_after_one_round_chosen_hidden",
            SentenceVerdict::of(ok && summary.chosen == Some(hidden) && summary.waits == 1),
            "F16",
        ),
        FourthRunCandidate::Bing => sentence(
            SentenceTag::ArmDefinition,
            "ok_effective_hidden",
            SentenceVerdict::of(ok && summary.effective == Some(hidden)),
            "F9",
        ),
    });
    emit_fourth_run_positive_control(
        arm.name,
        "PC-O",
        "duration=O1only",
        &sentences,
        &format!(
            "faults={} q0_destination={} {} {} {} unintercepted={}",
            faults.len(),
            cell.destination.name(),
            render_fourth_run_tested_mount(&cell.tested),
            cell.remount_line,
            cell.loss.render(),
            cell.tested.unintercepted.join(",")
        ),
    );
}

/// PC-N0（L0 全部 8 格）：P(n1, c) → 被测挂载，不注入。每臂 `Ok`（K0）、N_真 为假、取号写的 tail（-配置续 = c_见证，别的 = 0）；
/// 与今天逐项相同、PC-多读 (ii) 的点值在 `r4-compare` 里判（这里打出读数、指纹与 p）。L0 族（第四段）用同一批格。
fn run_fourth_run_positive_control_no_newer(geometry: &Geometry, arm: FourthRunArm) {
    let parameters = parameters_for(geometry);
    let mut cells = 0u64;
    for overwrites in THIRD_RUN_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let coordinates = format!("n1={overwrites} c={}", closing.name());
            let history = match fourth_run_plain_prefix(geometry, overwrites, closing, arm.name) {
                Ok(prefix) => prefix.history,
                Err(error) => {
                    emit_fourth_run_positive_control(
                        arm.name,
                        "PC-N0",
                        &coordinates,
                        &[sentence(
                            SentenceTag::Measured,
                            "prefix",
                            SentenceVerdict::NotConstructible,
                            "V1",
                        )],
                        &format!("reason={error:?}"),
                    );
                    continue;
                }
            };
            let base = history.pool.clone();
            let mut scratch = ThirdRunFamilySummary::default();
            let Ok(tested) = run_fourth_run_tested_mount(
                arm,
                &base,
                &parameters,
                &[],
                &coordinates,
                &mut scratch,
            ) else {
                continue;
            };
            cells += 1;
            let (fingerprint, acquisition_tails) =
                write_fingerprint_of(&tested.call.call.applied, 2);
            let tail_expected = if arm.witness == FourthRunWitness::SystemConfigurationCarried {
                tested.judgement.witnessed_tail
            } else {
                0
            };
            let tails_hold = acquisition_tails.len() == 2
                && acquisition_tails.iter().all(|tail| *tail == tail_expected);
            let instance_table_pages = tested.call.call.outcome.as_ref().ok().and_then(|mounted| {
                let devices = devices_from_pool(&base, IMAGE_BYTES);
                singlefs_core::recovery::instance_table_chain_of_root(
                    &devices,
                    &mounted.output.effective_root,
                )
                .ok()
                .map(|chain| chain.page_pointers.len())
            });
            let sentences = vec![
                sentence(
                    SentenceTag::Measured,
                    "truth_not_newer",
                    SentenceVerdict::of(!tested.judgement.truth_newer),
                    "V1",
                ),
                sentence(
                    SentenceTag::ArmDefinition,
                    "ok_k0",
                    SentenceVerdict::of(tested.summary.class == "K0"),
                    "F12",
                ),
                sentence(
                    SentenceTag::ArmDefinition,
                    "acquisition_tails",
                    SentenceVerdict::of(tails_hold),
                    "F12",
                ),
            ];
            emit_fourth_run_positive_control(
                arm.name,
                "PC-N0",
                &coordinates,
                &sentences,
                &format!(
                    "write_fingerprint={fingerprint:016x} base_fingerprint={:016x} acquisition_tails={} acquisition_tail_expected={tail_expected} effective_instance_table_pages={} {} s4_mismatches={}",
                    image_fingerprint(&base),
                    acquisition_tails
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    instance_table_pages.map_or("none".to_string(), |pages| pages.to_string()),
                    render_fourth_run_tested_mount(&tested),
                    scratch.stop_condition_four_mismatches.len()
                ),
            );
            emit_fourth_run_scratch_findings("PC-N0", arm.name, &scratch);
        }
    }
    let registered = 4 * 2;
    emit_result(&format!(
        "name=r4_section_seven_two_anchor arm={} item=\"L0 格数 4×2\" computed={cells} registered={registered} verdict={}",
        arm.name,
        if cells == registered { "pass" } else { "fail_v4" }
    ));
}

// ---------------------------------------------------------------------------
// 12.7 在今天那一臂上造的起始镜像（登记「读法写死」表「被测那次挂载」一行：被测之前的镜像一律在今天那一臂上造）。
//      起始镜像里有一段要靠今天那一臂的代码走的（L5、PC-22、PC-554 的 A；L1(b) 那次可写挂载的写序列；H1f 第二次取号），
//      由今天那一份副本跑 `r4-bases` 写进目录（环境变量 `SINGLEFS_E158_TODAY_BASES`），各臂读回来；各臂自己按同一串步骤记账，
//      账里的根与读回来的镜像上的自证根逐条核（`r4_today_base_check` 行）。
// ---------------------------------------------------------------------------

/// 一份在今天那一臂上造的起始镜像：镜像，今天那一臂那几次调用写下、装置的账要补的根（与它们沿用的内容号），
/// 与 L5 要的「今天那一臂在孪生镜像上做那次挂载时每一片被读的次数」。
struct TodayBase {
    image: MemoryPool,
    appended_roots: Vec<(TimelineRoot, u64)>,
    page_reads: Vec<u64>,
}

const TODAY_BASE_MAGIC: &[u8; 8] = b"E158R4B1";

fn today_bases_directory() -> Result<std::path::PathBuf, String> {
    env::var("SINGLEFS_E158_TODAY_BASES")
        .map(std::path::PathBuf::from)
        .map_err(|_| {
            "环境变量 SINGLEFS_E158_TODAY_BASES 没给（先在今天那一份副本上跑 r4-bases）".to_string()
        })
}

fn encode_today_base(base: &TodayBase) -> Vec<u8> {
    let mut bytes = TODAY_BASE_MAGIC.to_vec();
    bytes.extend_from_slice(
        &u32::try_from(base.image.devices.len())
            .expect("两块盘")
            .to_le_bytes(),
    );
    for (identity, device) in &base.image.devices {
        let sectors =
            device.written_sectors_in(DeviceOffsetInBytes(0), base.image.device_size_in_bytes);
        bytes.extend_from_slice(&identity.0.to_le_bytes());
        bytes.extend_from_slice(&(sectors.len() as u64).to_le_bytes());
        for sector in sectors {
            bytes.extend_from_slice(&sector.to_le_bytes());
            bytes.extend_from_slice(
                &device.read(DeviceOffsetInBytes(sector * SECTOR_BYTES_LOCAL), 512),
            );
        }
    }
    bytes.extend_from_slice(&(base.appended_roots.len() as u64).to_le_bytes());
    for ((instance, txg), content_number) in &base.appended_roots {
        bytes.extend_from_slice(&instance.to_le_bytes());
        bytes.extend_from_slice(&txg.to_le_bytes());
        bytes.extend_from_slice(&content_number.to_le_bytes());
    }
    bytes.extend_from_slice(&(base.page_reads.len() as u64).to_le_bytes());
    for reads in &base.page_reads {
        bytes.extend_from_slice(&reads.to_le_bytes());
    }
    bytes
}

/// 按字节流顺序取定宽的整数（`decode_today_base` 用）。
struct ByteCursor<'bytes> {
    bytes: &'bytes [u8],
    position: usize,
}

impl ByteCursor<'_> {
    fn take(&mut self, length: usize) -> Result<&[u8], String> {
        let end = self.position + length;
        let taken = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| format!("起始镜像文件在第 {} 字节截断", self.position))?;
        self.position = end;
        Ok(taken)
    }

    fn u32_value(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 字节"),
        ))
    }

    fn u64_value(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 字节"),
        ))
    }
}

fn decode_today_base(bytes: &[u8]) -> Result<TodayBase, String> {
    let mut cursor = ByteCursor { bytes, position: 0 };
    if cursor.take(8)? != TODAY_BASE_MAGIC {
        return Err("起始镜像文件的魔数不对".to_string());
    }
    let device_count = cursor.u32_value()?;
    let mut devices = BTreeMap::new();
    for _ in 0..device_count {
        let identity = DeviceIdentity(cursor.u32_value()?);
        let sector_count = cursor.u64_value()?;
        let mut device = SparseDevice::default();
        for _ in 0..sector_count {
            let sector = cursor.u64_value()?;
            let content = cursor.take(512)?.to_vec();
            device.write(DeviceOffsetInBytes(sector * SECTOR_BYTES_LOCAL), &content);
        }
        devices.insert(identity, device);
    }
    let appended_count = cursor.u64_value()?;
    let mut appended_roots = Vec::new();
    for _ in 0..appended_count {
        let instance = cursor.u32_value()?;
        let txg = cursor.u64_value()?;
        let content_number = cursor.u64_value()?;
        appended_roots.push(((instance, txg), content_number));
    }
    let page_count = cursor.u64_value()?;
    let mut page_reads = Vec::new();
    for _ in 0..page_count {
        page_reads.push(cursor.u64_value()?);
    }
    Ok(TodayBase {
        image: MemoryPool {
            devices,
            device_size_in_bytes: IMAGE_BYTES,
        },
        appended_roots,
        page_reads,
    })
}

fn today_base_path(key: &str) -> Result<std::path::PathBuf, String> {
    Ok(today_bases_directory()?.join(format!("{key}.base")))
}

fn save_today_base(key: &str, base: &TodayBase) -> Result<(), String> {
    let path = today_base_path(key)?;
    std::fs::write(&path, encode_today_base(base))
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn load_today_base(key: &str) -> Result<TodayBase, String> {
    let path = today_base_path(key)?;
    let bytes = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    decode_today_base(&bytes)
}

/// 把今天那一臂造的镜像换进这一臂按同一串步骤记的账：补上今天那几次调用写下的根，逐条核账里的根都是读回来的镜像上的自证根。
fn adopt_today_base(
    history: &mut ThirdRunHistory,
    key: &str,
    arm: FourthRunArm,
) -> Result<Vec<u64>, String> {
    let base = load_today_base(key)?;
    for (root, content_number) in &base.appended_roots {
        history.push_carried(*root, *content_number);
    }
    let geometry = independent_geometry(&base.image)?;
    let on_the_image = readable_roots_independent(&base.image, &geometry);
    let missing: Vec<String> = history
        .content_number_by_root
        .keys()
        .filter(|root| !on_the_image.contains(root))
        .map(|root| key_text(Some(*root)))
        .collect();
    emit_result(&format!(
        "name=r4_today_base_check arm={} key={key} base_fingerprint={:016x} appended_roots={} ledger_roots_missing_on_the_image={} note=\"根环转过的旧根不在镜像上是正常的，只报数\"",
        arm.name,
        image_fingerprint(&base.image),
        base.appended_roots.len(),
        missing.len()
    ));
    history.pool = base.image;
    history.session = None;
    Ok(base.page_reads)
}

// ---------------------------------------------------------------------------
// 12.8 第三段：H1f（前缀中间夹一次重挂载，前缀镜像在今天那一臂上造）、H1g（a 次挂载崩在取号之后，在这一臂自己的副本上做）。
// ---------------------------------------------------------------------------

fn remount_prefix_key(
    geometry: &Geometry,
    first_overwrites: u64,
    closing: Closing,
    second_overwrites: u64,
) -> String {
    format!(
        "h1f-s{}-ring{}-n1a{first_overwrites}-c{}-n1b{second_overwrites}",
        geometry.slots_per_region,
        geometry.journal_ring_bytes,
        closing.name()
    )
}

/// H1f 的八个前缀（这一臂按同一串步骤记账，镜像换成今天那一臂造的那一份）。
fn fourth_run_remount_prefixes(
    geometry: &Geometry,
    arm: FourthRunArm,
) -> Vec<Result<FourthRunPrefix, String>> {
    let mut prefixes = Vec::new();
    for first_overwrites in FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            for second_overwrites in FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES {
                prefixes.push(
                    fourth_run_remount_prefix(
                        geometry,
                        first_overwrites,
                        closing,
                        second_overwrites,
                        arm.name,
                    )
                    .and_then(|mut prefix| {
                        let key = remount_prefix_key(
                            geometry,
                            first_overwrites,
                            closing,
                            second_overwrites,
                        );
                        adopt_today_base(&mut prefix.history, &key, arm)?;
                        Ok(prefix)
                    }),
                );
            }
        }
    }
    prefixes
}

fn run_fourth_run_segment_three(arm: FourthRunArm) {
    run_fourth_run_remount_families(arm);
    run_fourth_run_acquisition_crash_family(arm);
}

/// H1f（G0）：八个前缀接 H1d 的全部断点、接 H1e 的 m 尾巴。
fn run_fourth_run_remount_families(arm: FourthRunArm) {
    for (name, continuation) in [
        ("h1f-cuts", FourthRunContinuation::EveryCut),
        ("h1f-tail", FourthRunContinuation::Tail),
    ] {
        let family = FourthRunFamily {
            name,
            geometry: &GEOMETRY_PRIMARY,
            hidden_count: 1,
            continuation,
            tail_overwrites: &THIRD_RUN_TAIL_OVERWRITES,
            durations: FourthRunDurationPlan::OfTheArm,
            checks_the_clauses_on_the_hidden_root:
                ClauseChecksOnTheHiddenRoot::NotRegisteredForThisFamily,
        };
        run_fourth_run_family(
            &family,
            arm,
            fourth_run_remount_prefixes(&GEOMETRY_PRIMARY, arm),
        );
    }
}

/// H1g（G0）：P(n1, C) → a 次挂载崩在取号之后（这一臂自己的副本上）→ A（不断，W）→ m 尾巴。
fn run_fourth_run_acquisition_crash_family(arm: FourthRunArm) {
    let family = FourthRunFamily {
        name: "h1g",
        geometry: &GEOMETRY_PRIMARY,
        hidden_count: 1,
        continuation: FourthRunContinuation::Tail,
        tail_overwrites: &FOURTH_RUN_ACQUISITION_CRASH_TAIL_OVERWRITES,
        durations: FourthRunDurationPlan::WholeCallOnly,
        checks_the_clauses_on_the_hidden_root:
            ClauseChecksOnTheHiddenRoot::NotRegisteredForThisFamily,
    };
    let mut prefixes = Vec::new();
    for overwrites in FOURTH_RUN_ACQUISITION_CRASH_PREFIX_OVERWRITES {
        for acquisition_crashes in FOURTH_RUN_ACQUISITION_CRASHES {
            prefixes.push(
                fourth_run_acquisition_crash_prefix(
                    &GEOMETRY_PRIMARY,
                    overwrites,
                    acquisition_crashes,
                )
                .map(|(prefix, checks)| {
                    emit_result(&format!(
                        "name=r4_h1g_acquisition_crash arm={} {} checks={}",
                        arm.name,
                        prefix.coordinates,
                        checks.join(",")
                    ));
                    prefix
                }),
            );
        }
    }
    run_fourth_run_family(&family, arm, prefixes);
}

// ---------------------------------------------------------------------------
// 12.9 第四段：L0–L6（第 3 次跑 5.3 那几行，合法状态与第 22 条那一格）；PC-多拒、PC-22、PC-554 的格（跨臂那几句在 compare 里判）。
//      每一格打一行 `r4_l_cell`：被测挂载的结局、N_真、读数、起始镜像指纹（Q4、Q5-同成 的输入），L5 另打隔离总数与每一片的读数（Q6）。
// ---------------------------------------------------------------------------

/// L 族与第四、五段那几格的账：第 3 次跑那一份汇总（7.3 第一行、F13 的点名），加每族每个几何点实际跑到被测挂载的格数（7.2）。
#[derive(Default)]
struct FourthRunLegalAndFaultStateTally {
    counts: ThirdRunFamilySummary,
    cells: BTreeMap<String, u64>,
}

/// 一格 L 族被测挂载打一行。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的族、坐标、起始镜像、被测挂载与故障原样打进结果行，拆成结构体只多一层搬运"
)]
fn emit_fourth_run_legal_or_fault_state_cell(
    family: &str,
    arm: FourthRunArm,
    geometry: &Geometry,
    coordinates: &str,
    base: &MemoryPool,
    tested: &FourthRunTestedMount,
    faults: &[ThirdRunFault],
    extra: &str,
) {
    let (write_fingerprint, _) = write_fingerprint_of(&tested.call.call.applied, 2);
    emit_result(&format!(
        "name=r4_l_cell family={family} arm={} geometry={} {coordinates} base_fingerprint={:016x} write_fingerprint={write_fingerprint:016x} faults={} unintercepted={} refused={} truth_newer_on_the_twin={} {} {extra}",
        arm.name,
        geometry.label,
        image_fingerprint(base),
        faults.len(),
        tested.unintercepted.join(","),
        tested.call.call.outcome.is_err(),
        tested.judgement.truth_newer,
        render_fourth_run_tested_mount(tested)
    ));
}

/// 在装置的账上记下一次改内容的调用写了哪些（第 3 次跑 `third_run_change_content` 同一串调用，经受观测的设备走），交回写序列。
fn fourth_run_change_content_recorded(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<Vec<AppliedDeviceStep>, String> {
    let content_number = history.last_confirmed_content_number() + 1;
    let content = third_run_content(content_number);
    let mut session = history.session.take().ok_or("改内容：没有开着的会话")?;
    let base = history.pool.clone();
    let call = fourth_run_call(&base, &[], None, None, |devices| {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let file = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * content_number,
        };
        match &session.current {
            PoolVersion::WithFile(previous) => publish_overwrite(
                &mut writer,
                &mut session.allocator,
                previous,
                file,
                session.instance,
            )
            .map_err(|error| format!("覆盖写: {error:?}")),
            PoolVersion::WithoutFile(version) => publish_first_file(
                &mut writer,
                &mut session.allocator,
                &version.root,
                file,
                session.instance,
                session.current.record_bytes(),
            )
            .map_err(|error| format!("首个文件: {error:?}")),
        }
    });
    let output = call.call.outcome?;
    history.pool = call.call.pool_after;
    let root_record = output.root;
    session.current = PoolVersion::WithFile(output);
    history.session = Some(session);
    record_confirmation(history, &root_record, content_number)?;
    Ok(call.call.applied)
}

/// 前 `count` 个写的崩溃状态里「写」的个数（屏障不数）。
fn writes_in(applied: &[AppliedDeviceStep]) -> usize {
    applied
        .iter()
        .filter(|step| !matches!(step, AppliedDeviceStep::Barrier { .. }))
        .count()
}

fn legal_or_fault_state_prefix(
    geometry: &Geometry,
    overwrites: u64,
    arm: FourthRunArm,
) -> Result<ThirdRunHistory, String> {
    Ok(fourth_run_plain_prefix(geometry, overwrites, Closing::ProcessExit, arm.name)?.history)
}

/// 一格 L 族被测挂载：在 `base` 上装 `faults` 跑这一臂的可写挂载，打一行。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的族、坐标、起始镜像与故障原样打进结果行，拆成结构体只多一层搬运"
)]
fn run_fourth_run_legal_or_fault_state_cell(
    family: &str,
    arm: FourthRunArm,
    geometry: &Geometry,
    coordinates: &str,
    base: &MemoryPool,
    faults: &[ThirdRunFault],
    extra: &str,
    summary: &mut FourthRunLegalAndFaultStateTally,
) -> Option<FourthRunTestedMount> {
    *summary
        .cells
        .entry(format!("{family}:{}", geometry.label))
        .or_insert(0) += 1;
    let parameters = parameters_for(geometry);
    match run_fourth_run_tested_mount(
        arm,
        base,
        &parameters,
        faults,
        coordinates,
        &mut summary.counts,
    ) {
        Ok(tested) => {
            emit_fourth_run_legal_or_fault_state_cell(
                family,
                arm,
                geometry,
                coordinates,
                base,
                &tested,
                faults,
                extra,
            );
            Some(tested)
        }
        Err(error) => {
            emit_result(&format!(
                "name=r4_l_cell family={family} arm={} geometry={} {coordinates} verdict=stop reason={error:?}",
                arm.name, geometry.label
            ));
            None
        }
    }
}

/// L0（P(n1, c)，不注入）与 L1（崩溃状态上不注入）。
fn run_fourth_run_legal_state_families(
    geometry: &Geometry,
    arm: FourthRunArm,
    summary: &mut FourthRunLegalAndFaultStateTally,
) {
    let parameters = parameters_for(geometry);
    for overwrites in THIRD_RUN_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let coordinates = format!("n1={overwrites} c={}", closing.name());
            match fourth_run_plain_prefix(geometry, overwrites, closing, arm.name) {
                Ok(prefix) => {
                    run_fourth_run_legal_or_fault_state_cell(
                        "l0",
                        arm,
                        geometry,
                        &coordinates,
                        &prefix.history.pool,
                        &[],
                        "",
                        summary,
                    );
                }
                Err(error) => emit_result(&format!(
                    "name=r4_l_cell family=l0 arm={} {coordinates} verdict=stop reason={error:?}",
                    arm.name
                )),
            }
        }
        // L1(a)：P(n1, C) 最后一次改内容的发布的写序列上，前 j 个写落盘的崩溃状态（j 取全部真前缀）。
        let history_before_the_last_change = if overwrites == 0 {
            third_run_history_start_without_the_first_file(geometry)
        } else {
            third_run_history_start(geometry, overwrites - 1)
        };
        match history_before_the_last_change.and_then(|mut history| {
            let before = history.pool.clone();
            let applied = fourth_run_change_content_recorded(&mut history, &parameters)?;
            Ok((before, applied))
        }) {
            Ok((before, applied)) => {
                for writes in 0..writes_in(&applied) {
                    let crashed = image_with_the_first_writes(&before, &applied, writes);
                    run_fourth_run_legal_or_fault_state_cell(
                        "l1a",
                        arm,
                        geometry,
                        &format!("n1={overwrites} point={writes} of={}", writes_in(&applied)),
                        &crashed,
                        &[],
                        "",
                        summary,
                    );
                }
            }
            Err(error) => emit_result(&format!(
                "name=r4_l_cell family=l1a arm={} n1={overwrites} verdict=stop reason={error:?}",
                arm.name
            )),
        }
        // L1(b)：P(n1, C) 之后一次可写挂载的写序列（今天那一臂上录的，r4-bases 存了每个崩溃状态）。
        let mut point = 0usize;
        loop {
            let key = format!(
                "l1b-s{}-ring{}-n1{overwrites}-point{point}",
                geometry.slots_per_region, geometry.journal_ring_bytes
            );
            let Ok(base) = load_today_base(&key) else {
                if point == 0 {
                    emit_result(&format!(
                        "name=r4_l_cell family=l1b arm={} n1={overwrites} verdict=stop reason=\"今天那一臂的起始镜像读不出（先跑 r4-bases）\"",
                        arm.name
                    ));
                }
                break;
            };
            run_fourth_run_legal_or_fault_state_cell(
                "l1b",
                arm,
                geometry,
                &format!("n1={overwrites} point={point}"),
                &base.image,
                &[],
                "",
                summary,
            );
            point += 1;
        }
        // L1(c)：P(n1, U) 的卸载写序列上前 j 个写落盘的崩溃状态。
        match third_run_history_start(geometry, overwrites).and_then(|mut history| {
            let before = history.pool.clone();
            let applied = fourth_run_unmount_recorded(&mut history, &parameters)?;
            Ok((before, applied))
        }) {
            Ok((before, applied)) => {
                for writes in 0..writes_in(&applied) {
                    let crashed = image_with_the_first_writes(&before, &applied, writes);
                    run_fourth_run_legal_or_fault_state_cell(
                        "l1c",
                        arm,
                        geometry,
                        &format!("n1={overwrites} point={writes} of={}", writes_in(&applied)),
                        &crashed,
                        &[],
                        "",
                        summary,
                    );
                }
            }
            Err(error) => emit_result(&format!(
                "name=r4_l_cell family=l1c arm={} n1={overwrites} verdict=stop reason={error:?}",
                arm.name
            )),
        }
    }
}

/// P(n1, ·) 走到「mkfs → 可写挂载」为止、首个文件之前（L1(a) 在 n1 = 0 上要首个文件那一次发布自己的写序列）。
fn third_run_history_start_without_the_first_file(
    geometry: &Geometry,
) -> Result<ThirdRunHistory, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let mounted = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("第一次可写挂载: {error:?}"))?;
    let mut history = ThirdRunHistory {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: None,
        timeline: vec![root_pair(&genesis.root)],
        content_number_by_root: BTreeMap::from([(root_pair(&genesis.root), 0)]),
        confirmations: Vec::new(),
        tail_checks: 0,
        tail_check_failures: Vec::new(),
    };
    push_the_publishes_of_a_mount(&mut history, &mounted, 0);
    history.session = Some(Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    });
    Ok(history)
}

/// 正常卸载（经受观测的设备走），交回写序列（L1(c)）。
fn fourth_run_unmount_recorded(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<Vec<AppliedDeviceStep>, String> {
    let Session {
        mut allocator,
        mut current,
        ..
    } = history.session.take().ok_or("卸载：没有开着的会话")?;
    let call = fourth_run_call(&history.pool, &[], None, None, |devices| {
        singlefs_core::mount::unmount(
            parameters,
            devices,
            &mut allocator,
            &mut current,
            ShadowLedger::On,
        )
        .map(|_| ())
        .map_err(|error| format!("正常卸载: {error:?}"))
    });
    call.call.outcome?;
    history.pool = call.call.pool_after;
    Ok(call.call.applied)
}

/// 第 k 新的根（按时间线）根槽的落点，W 读坏（L2、PC-多拒 的 -槽 那一格）。
fn older_root_slot_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    newest_rank: usize,
    form: UnreadableForm,
) -> Option<Vec<ThirdRunFault>> {
    let root = *history.timeline.iter().rev().nth(newest_rank - 1)?;
    let (device, offset) = root_slot_of(&history.pool, geometry, root)?;
    Some(vec![ThirdRunFault {
        device,
        offset,
        length: u64::from(geometry.physical_block_size),
        form,
        duration: FaultDuration::WholeCall,
        label: format!("root{}", key_text(Some(root))),
    }])
}

/// L3 的三种：(a) 最新那次发布的末条记录两份；(b) 全部记录两份；(c) 末条记录盘 0 那一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordFaultChoice {
    LastRecordBothCopies,
    EveryRecordBothCopies,
    LastRecordDeviceZeroCopy,
}

impl RecordFaultChoice {
    fn name(self) -> &'static str {
        match self {
            RecordFaultChoice::LastRecordBothCopies => "a",
            RecordFaultChoice::EveryRecordBothCopies => "b",
            RecordFaultChoice::LastRecordDeviceZeroCopy => "c",
        }
    }
}

fn record_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    choice: RecordFaultChoice,
    form: UnreadableForm,
) -> Option<Vec<ThirdRunFault>> {
    let records = records_of_publish(&history.pool, geometry, history.tip());
    let last_counter = records.iter().map(|(counter, _, _)| *counter).max()?;
    let chosen: Vec<&(u64, u64, JournalRecord)> = records
        .iter()
        .filter(|(counter, _, _)| match choice {
            RecordFaultChoice::LastRecordBothCopies
            | RecordFaultChoice::LastRecordDeviceZeroCopy => *counter == last_counter,
            RecordFaultChoice::EveryRecordBothCopies => true,
        })
        .collect();
    let devices: &[u32] = match choice {
        RecordFaultChoice::LastRecordDeviceZeroCopy => &[0],
        RecordFaultChoice::LastRecordBothCopies | RecordFaultChoice::EveryRecordBothCopies => {
            &[0, 1]
        }
    };
    let mut faults = Vec::new();
    for (counter, offset, _) in chosen {
        for device in devices {
            faults.push(ThirdRunFault {
                device: DeviceIdentity(*device),
                offset: *offset,
                length: JOURNAL_RECORD_BYTES,
                form,
                duration: FaultDuration::WholeCall,
                label: format!("record(counter={counter},device={device})"),
            });
        }
    }
    Some(faults)
}

/// L4：世代号最新的系统配置槽：(a) 盘 0 那一槽；(b) 两块盘各自那一槽。
fn system_configuration_faults(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    both_devices: bool,
    form: UnreadableForm,
) -> Vec<ThirdRunFault> {
    let slot_spacing = geometry.slot_spacing;
    let devices: &[u32] = if both_devices { &[0, 1] } else { &[0] };
    let mut faults = Vec::new();
    for device in devices {
        let newest = [0u64, slot_spacing]
            .into_iter()
            .filter_map(|offset| {
                let bytes = pool.read(*device, offset, 4096)?;
                singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
                    .ok()
                    .map(|parsed| (parsed.quantities.slot_generation, offset))
            })
            .max_by_key(|(generation, _)| *generation);
        if let Some((generation, offset)) = newest {
            faults.push(ThirdRunFault {
                device: DeviceIdentity(*device),
                offset,
                length: SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
                form,
                duration: FaultDuration::WholeCall,
                label: format!("system_configuration(device={device},generation={generation})"),
            });
        }
    }
    faults
}

/// L2（第 k 新的根槽读坏，k ∈ {2, 3}）、L3（记录读坏三种）、L4（系统配置读坏两种）、L6（最新根槽读坏、链完整），两种造法。
fn run_fourth_run_fault_state_families(
    geometry: &Geometry,
    arm: FourthRunArm,
    families: &[&str],
    summary: &mut FourthRunLegalAndFaultStateTally,
) {
    for overwrites in THIRD_RUN_OVERWRITES {
        let history = match legal_or_fault_state_prefix(geometry, overwrites, arm) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r4_l_cell family=l arm={} n1={overwrites} verdict=stop reason={error:?}",
                    arm.name
                ));
                continue;
            }
        };
        let Ok(pool_geometry) = independent_geometry(&history.pool) else {
            continue;
        };
        let hidden = history.tip();
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            if families.contains(&"l2") {
                for newest_rank in [2usize, 3] {
                    let coordinates =
                        format!("n1={overwrites} k={newest_rank} form={}", form.name());
                    match older_root_slot_faults(&history, &pool_geometry, newest_rank, form) {
                        Some(faults) => {
                            run_fourth_run_legal_or_fault_state_cell("l2", arm, geometry, &coordinates, &history.pool, &faults, "", summary);
                        }
                        None => emit_result(&format!(
                            "name=r4_l_cell family=l2 arm={} geometry={} {coordinates} verdict=not_applicable reason=\"时间线上的根不够 k 条\"",
                            arm.name, geometry.label
                        )),
                    }
                }
            }
            if families.contains(&"l3") {
                for choice in [
                    RecordFaultChoice::LastRecordBothCopies,
                    RecordFaultChoice::EveryRecordBothCopies,
                    RecordFaultChoice::LastRecordDeviceZeroCopy,
                ] {
                    let coordinates = format!(
                        "n1={overwrites} which={} form={}",
                        choice.name(),
                        form.name()
                    );
                    match record_faults(&history, &pool_geometry, choice, form) {
                        Some(faults) => {
                            run_fourth_run_legal_or_fault_state_cell("l3", arm, geometry, &coordinates, &history.pool, &faults, "", summary);
                        }
                        None => emit_result(&format!(
                            "name=r4_l_cell family=l3 arm={} geometry={} {coordinates} verdict=not_applicable reason=\"最新那次发布没有记录\"",
                            arm.name, geometry.label
                        )),
                    }
                }
            }
            if families.contains(&"l4") {
                for (which, both_devices) in [("a", false), ("b", true)] {
                    let coordinates = format!("n1={overwrites} which={which} form={}", form.name());
                    let faults = system_configuration_faults(
                        &history.pool,
                        &pool_geometry,
                        both_devices,
                        form,
                    );
                    run_fourth_run_legal_or_fault_state_cell(
                        "l4",
                        arm,
                        geometry,
                        &coordinates,
                        &history.pool,
                        &faults,
                        "",
                        summary,
                    );
                }
            }
            if families.contains(&"l6") {
                let coordinates = format!("n1={overwrites} form={}", form.name());
                if let Some(faults) = older_root_slot_faults(&history, &pool_geometry, 1, form) {
                    if let Some(tested) = run_fourth_run_legal_or_fault_state_cell(
                        "l6",
                        arm,
                        geometry,
                        &coordinates,
                        &history.pool,
                        &faults,
                        "",
                        summary,
                    ) {
                        // 7.1 第三行（F5，今天那一臂）：根槽读不出而记录与单元都读得出时，恢复由记录重建那次发布的根。
                        if arm.is_today() {
                            let holds = tested.summary.effective == Some(hidden);
                            emit_result(&format!(
                                "name=r4_section_seven_one_f5 arm={} {coordinates} verdict={} effective={} hidden={}",
                                arm.name,
                                if holds { "pass" } else { "fail_f5" },
                                key_text(tested.summary.effective),
                                key_text(Some(hidden))
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// L5 的 n1 与 b（登记 5.3 L5 行：n1 ∈ {1, 2, 3}；b 两种）。
const FOURTH_RUN_CELL_22_OVERWRITES: [u64; 3] = [1, 2, 3];

fn cell_22_base_key(geometry: &Geometry, overwrites: u64, replay_break: ReplayBreak) -> String {
    format!(
        "l5-s{}-ring{}-n1{overwrites}-b{}",
        geometry.slots_per_region,
        geometry.journal_ring_bytes,
        replay_break.name()
    )
}

/// 最新根（择根那条）的实例表每一片的两份落点（`crates/` 的沿链读，`instance_table_page_pointers_as_far_as_readable`；
/// 装置没有自己的沿链读，执行员修订第 8 条）。
fn instance_table_page_faults_of_the_newest_root(
    pool: &MemoryPool,
    duration: FaultDuration,
) -> Result<Vec<Vec<ThirdRunFault>>, String> {
    let (_, newest) = landing_root_on(pool)?;
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let pages =
        singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable(&devices, &newest);
    Ok(pages
        .iter()
        .enumerate()
        .map(|(page, pointer)| {
            pointer
                .locations
                .iter()
                .map(|location| ThirdRunFault {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form: UnreadableForm::ReadFails,
                    duration,
                    label: format!(
                        "instance_table_page{page}(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                })
                .collect()
        })
        .collect())
}

/// L5 的起始镜像（今天那一臂，`r4-bases` 里造）：P(n1, C) → A（`read_fails`、W、不断）→ 关闭 C → 撤故障；
/// 另在它的孪生镜像上做一次不注入的可写挂载，数每一片（两份合计）被读的次数 M（登记 5.3 L5 行）。
fn build_today_cell_22_base(
    geometry: &Geometry,
    overwrites: u64,
    replay_break: ReplayBreak,
) -> Result<TodayBase, String> {
    let parameters = parameters_for(geometry);
    let history =
        fourth_run_plain_prefix(geometry, overwrites, Closing::ProcessExit, "today")?.history;
    let pool_geometry = independent_geometry(&history.pool)?;
    let faults = fourth_run_abandonment_faults(
        &history,
        &pool_geometry,
        UnreadableForm::ReadFails,
        replay_break,
        FaultDuration::WholeCall,
        1,
    )?
    .ok_or("A 的落点造不出")?;
    let tested = run_fourth_run_mount_writable(&history.pool, &parameters, &faults, None, None);
    let mounted = tested
        .call
        .outcome
        .as_ref()
        .map_err(|error| format!("今天那一臂的 A 被拒: {error:?}"))?;
    let carried = history
        .content_number_of(root_pair(&mounted.output.effective_root))
        .unwrap_or(0);
    let numbers = content_numbers_with_the_publishes_of(&history, Some(mounted));
    let appended_roots: Vec<(TimelineRoot, u64)> = numbers
        .iter()
        .filter(|(root, _)| !history.content_number_by_root.contains_key(root))
        .map(|(root, _)| (*root, carried))
        .collect();
    let image = tested.call.pool_after.clone();
    let page_faults = instance_table_page_faults_of_the_newest_root(
        &image,
        FaultDuration::FromTheNthRead(u64::MAX),
    )?;
    let flat: Vec<ThirdRunFault> = page_faults.iter().flatten().cloned().collect();
    let twin = run_fourth_run_mount_writable(&image, &parameters, &flat, None, None);
    let page_reads = page_faults
        .iter()
        .scan(0usize, |start, copies| {
            let reads: u64 = twin.reads_of_the_targets[*start..*start + copies.len()]
                .iter()
                .sum();
            *start += copies.len();
            Some(reads)
        })
        .collect();
    Ok(TodayBase {
        image,
        appended_roots,
        page_reads,
    })
}

/// L1(b) 的崩溃状态（今天那一臂）：P(n1, C) 之后一次可写挂载的写序列上前 j 个写落盘（j 取全部真前缀）。
fn build_today_mount_crash_bases(
    geometry: &Geometry,
    overwrites: u64,
) -> Result<Vec<TodayBase>, String> {
    let parameters = parameters_for(geometry);
    let history =
        fourth_run_plain_prefix(geometry, overwrites, Closing::ProcessExit, "today")?.history;
    let call = run_fourth_run_mount_writable(&history.pool, &parameters, &[], None, None);
    call.call
        .outcome
        .as_ref()
        .map_err(|error| format!("今天那一臂的可写挂载被拒: {error:?}"))?;
    Ok((0..writes_in(&call.call.applied))
        .map(|writes| TodayBase {
            image: image_with_the_first_writes(&history.pool, &call.call.applied, writes),
            appended_roots: Vec::new(),
            page_reads: Vec::new(),
        })
        .collect())
}

/// H1f 的前缀镜像（今天那一臂）。
fn build_today_remount_prefix_base(
    geometry: &Geometry,
    first_overwrites: u64,
    closing: Closing,
    second_overwrites: u64,
) -> Result<TodayBase, String> {
    let prefix = fourth_run_remount_prefix(
        geometry,
        first_overwrites,
        closing,
        second_overwrites,
        "today",
    )?;
    Ok(TodayBase {
        image: prefix.history.pool,
        appended_roots: Vec::new(),
        page_reads: Vec::new(),
    })
}

/// `r4-bases`（只在今天那一份副本上跑）：把各段要的「在今天那一臂上造」的起始镜像写进目录。
fn run_fourth_run_today_bases(arm: FourthRunArm) {
    if !arm.is_today() {
        emit_result(&format!(
            "name=r4_today_bases arm={} verdict=stop reason=\"r4-bases 只在今天那一份副本上跑\"",
            arm.name
        ));
        return;
    }
    let mut saved = 0u64;
    let mut failures = Vec::new();
    let mut save = |key: String, base: Result<TodayBase, String>| match base.and_then(|base| {
        save_today_base(&key, &base)?;
        Ok(base)
    }) {
        Ok(base) => {
            saved += 1;
            emit_result(&format!(
                "name=r4_today_base key={key} base_fingerprint={:016x} appended_roots={} page_reads={:?}",
                image_fingerprint(&base.image),
                base.appended_roots.len(),
                base.page_reads
            ));
        }
        Err(error) => failures.push(format!("{key}: {error}")),
    };
    for first_overwrites in FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            for second_overwrites in FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES {
                save(
                    remount_prefix_key(
                        &GEOMETRY_PRIMARY,
                        first_overwrites,
                        closing,
                        second_overwrites,
                    ),
                    build_today_remount_prefix_base(
                        &GEOMETRY_PRIMARY,
                        first_overwrites,
                        closing,
                        second_overwrites,
                    ),
                );
            }
        }
    }
    for overwrites in THIRD_RUN_OVERWRITES {
        match build_today_mount_crash_bases(&GEOMETRY_PRIMARY, overwrites) {
            Ok(bases) => {
                for (point, base) in bases.into_iter().enumerate() {
                    save(
                        format!(
                            "l1b-s{}-ring{}-n1{overwrites}-point{point}",
                            GEOMETRY_PRIMARY.slots_per_region, GEOMETRY_PRIMARY.journal_ring_bytes
                        ),
                        Ok(base),
                    );
                }
            }
            Err(error) => save(format!("l1b-n1{overwrites}-mount"), Err(error)),
        }
    }
    for geometry in [
        &GEOMETRY_PRIMARY,
        &GEOMETRY_SMALLER_ROOT_RING,
        &GEOMETRY_LARGER_ROOT_RING,
    ] {
        for overwrites in FOURTH_RUN_CELL_22_OVERWRITES {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                save(
                    cell_22_base_key(geometry, overwrites, replay_break),
                    build_today_cell_22_base(geometry, overwrites, replay_break),
                );
            }
        }
    }
    for failure in &failures {
        emit_result(&format!("name=r4_today_base_failure detail={failure:?}"));
    }
    emit_result(&format!(
        "name=r4_today_bases arm={} saved={saved} failures={} verdict={}",
        arm.name,
        failures.len(),
        if failures.is_empty() { "pass" } else { "fail" }
    ));
}

/// 被抛弃根独占的槽（起始镜像上：它引用的落点里，不被任何一条没被抛弃的可读根引用的那几个）。
fn exclusive_slots_of_an_abandoned_root(
    pool: &MemoryPool,
    abandoned: TimelineRoot,
) -> Result<u64, String> {
    let geometry = independent_geometry(pool)?;
    let abandoned_set = abandoned_roots_on(pool)?;
    let record_of = |root: TimelineRoot| {
        root_record_of(pool, &geometry, root).ok_or(format!("根 {root:?} 的记录读不出"))
    };
    let referenced_by_it = placements_referenced_by_root_on(pool, &record_of(abandoned)?)?;
    let mut referenced_by_others = BTreeSet::new();
    for root in readable_roots_independent(pool, &geometry) {
        if root == abandoned || abandoned_set.contains(&root) {
            continue;
        }
        if let Ok(placements) = placements_referenced_by_root_on(pool, &record_of(root)?) {
            referenced_by_others.extend(placements);
        }
    }
    Ok(referenced_by_it.difference(&referenced_by_others).count() as u64)
}

/// L5 与 PC-554（起始镜像同一份，今天那一臂上造）：每一片的两份按 O(m 只)、O(m 起) 各一格（m = 1..M），另打这一臂自己在孪生镜像上
/// 那次不注入的挂载（隔离总数与每一片被读的次数，Q6 与 PC-22 丙 那一句要）；PC-554：那条被抛弃根自己的根槽 W 读坏。
fn run_fourth_run_shadow_ledger_families(
    geometry: &Geometry,
    arm: FourthRunArm,
    summary: &mut FourthRunLegalAndFaultStateTally,
) {
    let parameters = parameters_for(geometry);
    let mut constructed_bases = 0u64;
    let mut pc_554_done = false;
    for overwrites in FOURTH_RUN_CELL_22_OVERWRITES {
        for replay_break in [
            ReplayBreak::EveryRecord,
            ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        ] {
            let coordinates = format!("n1={overwrites} b={}", replay_break.name());
            let key = cell_22_base_key(geometry, overwrites, replay_break);
            let adopted =
                legal_or_fault_state_prefix(geometry, overwrites, arm).and_then(|mut history| {
                    let page_reads = adopt_today_base(&mut history, &key, arm)?;
                    Ok((history, page_reads))
                });
            let (history, page_reads) = match adopted {
                Ok(adopted) => adopted,
                Err(error) => {
                    emit_result(&format!(
                        "name=r4_l_cell family=l5 arm={} geometry={} {coordinates} verdict=stop reason={error:?}",
                        arm.name, geometry.label
                    ));
                    continue;
                }
            };
            constructed_bases += 1;
            let base = history.pool.clone();
            let counting_faults = match instance_table_page_faults_of_the_newest_root(
                &base,
                FaultDuration::FromTheNthRead(u64::MAX),
            ) {
                Ok(faults) => faults,
                Err(error) => {
                    emit_result(&format!(
                        "name=r4_l_cell family=l5 arm={} geometry={} {coordinates} verdict=stop reason={error:?}",
                        arm.name, geometry.label
                    ));
                    continue;
                }
            };
            let flat: Vec<ThirdRunFault> = counting_faults.iter().flatten().cloned().collect();
            let twin = run_fourth_run_mount_writable(&base, &parameters, &flat, None, None);
            let twin_summary = TestedMountSummary::of(&twin.call);
            let own_page_reads: Vec<u64> = counting_faults
                .iter()
                .scan(0usize, |start, copies| {
                    let reads: u64 = twin.reads_of_the_targets[*start..*start + copies.len()]
                        .iter()
                        .sum();
                    *start += copies.len();
                    Some(reads)
                })
                .collect();
            let twin_isolated = twin_summary.isolated;
            emit_result(&format!(
                "name=r4_l5_twin arm={} geometry={} {coordinates} base_fingerprint={:016x} today_page_reads={page_reads:?} own_page_reads={own_page_reads:?} twin_class={} twin_isolated={} twin_abandoned_roots_unreadable={}",
                arm.name,
                geometry.label,
                image_fingerprint(&base),
                twin_summary.class,
                twin_isolated.map_or("none".to_string(), |count| count.to_string()),
                twin_summary
                    .abandoned_roots_unreadable
                    .map_or("none".to_string(), |count| count.to_string())
            ));
            for (page, reads) in page_reads.iter().enumerate() {
                for nth in 1..=*reads {
                    for (duration_name, duration) in [
                        ("only", FaultDuration::OnlyTheNthRead(nth)),
                        ("from", FaultDuration::FromTheNthRead(nth)),
                    ] {
                        let Ok(page_faults) =
                            instance_table_page_faults_of_the_newest_root(&base, duration)
                        else {
                            continue;
                        };
                        let Some(faults) = page_faults.get(page) else {
                            continue;
                        };
                        let cell_coordinates = format!(
                            "{coordinates} page={page} m={nth} duration=O{nth}{duration_name}"
                        );
                        if let Some(tested) = run_fourth_run_legal_or_fault_state_cell(
                            "l5",
                            arm,
                            geometry,
                            &cell_coordinates,
                            &base,
                            faults,
                            &format!(
                                "twin_isolated={}",
                                twin_isolated.map_or("none".to_string(), |count| count.to_string())
                            ),
                            summary,
                        ) {
                            let not_covered = tested.call.call.outcome.is_ok()
                                && matches!((tested.summary.isolated, twin_isolated), (Some(isolated), Some(isolated_on_the_twin)) if isolated < isolated_on_the_twin);
                            emit_result(&format!(
                                "name=r4_q6_cell arm={} geometry={} {cell_coordinates} ok={} isolated={} twin_isolated={} abandoned_roots_unreadable={} q6_not_covered={not_covered}",
                                arm.name,
                                geometry.label,
                                tested.call.call.outcome.is_ok(),
                                tested.summary.isolated.map_or("none".to_string(), |count| count.to_string()),
                                twin_isolated.map_or("none".to_string(), |count| count.to_string()),
                                tested
                                    .summary
                                    .abandoned_roots_unreadable
                                    .map_or("none".to_string(), |count| count.to_string())
                            ));
                        }
                    }
                }
            }
            if !pc_554_done && geometry.label == GEOMETRY_PRIMARY.label {
                pc_554_done = run_fourth_run_positive_control_abandoned_root_slot(
                    arm,
                    &history,
                    &coordinates,
                    summary,
                );
            }
        }
    }
    let registered = 3 * 2;
    emit_result(&format!(
        "name=r4_section_seven_two_anchor arm={} geometry={} item=\"L5 起始镜像 3×2\" computed={constructed_bases} registered={registered} verdict={}",
        arm.name,
        geometry.label,
        if constructed_bases == registered { "pass" } else { "fail_v4" }
    ));
    if !pc_554_done && geometry.label == GEOMETRY_PRIMARY.label {
        emit_fourth_run_positive_control(
            arm.name,
            "PC-554",
            "n1=none b=none",
            &[sentence(
                SentenceTag::TodayDescription,
                "an_abandoned_root_with_exclusive_slots",
                SentenceVerdict::NotConstructible,
                "F1",
            )],
            "reason=\"L5 的六份起始镜像上都找不到独占槽个数 > 0 的被抛弃根\"",
        );
    }
}

/// PC-554（第 3 次跑 5.4 PC-554 行，标签照第 4 次跑 5.4）：起始镜像同 L5；被测挂载时那条被抛弃根自己的根槽 W 读坏（`read_fails`）。
/// 独占的槽个数 = 0 就交回 false（顺延到下一个 n1 与 b）。
fn run_fourth_run_positive_control_abandoned_root_slot(
    arm: FourthRunArm,
    history: &ThirdRunHistory,
    coordinates: &str,
    summary: &mut FourthRunLegalAndFaultStateTally,
) -> bool {
    let base = &history.pool;
    let Ok(abandoned) = abandoned_roots_on(base) else {
        return false;
    };
    let Some(abandoned_root) = abandoned
        .iter()
        .copied()
        .max_by_key(|(instance, txg)| (*txg, *instance))
    else {
        return false;
    };
    let exclusive = exclusive_slots_of_an_abandoned_root(base, abandoned_root).unwrap_or(0);
    if exclusive == 0 {
        return false;
    }
    let Ok(pool_geometry) = independent_geometry(base) else {
        return false;
    };
    let Some((device, offset)) = root_slot_of(base, &pool_geometry, abandoned_root) else {
        return false;
    };
    let faults = vec![ThirdRunFault {
        device,
        offset,
        length: u64::from(pool_geometry.physical_block_size),
        form: UnreadableForm::ReadFails,
        duration: FaultDuration::WholeCall,
        label: format!("abandoned_root{}", key_text(Some(abandoned_root))),
    }];
    let Some(tested) = run_fourth_run_legal_or_fault_state_cell(
        "pc554",
        arm,
        &GEOMETRY_PRIMARY,
        coordinates,
        base,
        &faults,
        "",
        summary,
    ) else {
        return false;
    };
    let ok_count_zero_isolated_zero = tested.call.call.outcome.is_ok()
        && tested.summary.abandoned_roots_unreadable == Some(0)
        && tested.summary.isolated == Some(0);
    let sentences = vec![match (arm.candidate, arm.witness) {
        (FourthRunCandidate::Today, _) | (_, FourthRunWitness::NotJudged) => sentence(
            SentenceTag::TodayDescription,
            "ok_count_zero_isolated_zero",
            SentenceVerdict::of(ok_count_zero_isolated_zero),
            "F1",
        ),
        (_, FourthRunWitness::UnreadableRootRingSlot) => sentence(
            SentenceTag::ArmDefinition,
            "slot_arm_refused",
            SentenceVerdict::of(
                tested.summary.member == arm.refusal_member() && tested.summary.writes == 0,
            ),
            "F9",
        ),
        (
            _,
            FourthRunWitness::SystemConfiguration | FourthRunWitness::SystemConfigurationCarried,
        ) => sentence(
            SentenceTag::ArmDefinition,
            "same_outcome_as_today_ok_count_zero_isolated_zero",
            SentenceVerdict::of(ok_count_zero_isolated_zero),
            "F12",
        ),
    }];
    emit_fourth_run_positive_control(
        arm.name,
        "PC-554",
        coordinates,
        &sentences,
        &format!(
            "abandoned_root={} exclusive_slots={exclusive} truth_newer_on_the_twin={} {}",
            key_text(Some(abandoned_root)),
            tested.judgement.truth_newer,
            render_fourth_run_tested_mount(&tested)
        ),
    );
    true
}

fn run_fourth_run_segment_four(arm: FourthRunArm) {
    let mut summary = FourthRunLegalAndFaultStateTally::default();
    run_fourth_run_legal_state_families(&GEOMETRY_PRIMARY, arm, &mut summary);
    run_fourth_run_fault_state_families(
        &GEOMETRY_PRIMARY,
        arm,
        &["l2", "l3", "l4", "l6"],
        &mut summary,
    );
    run_fourth_run_shadow_ledger_families(&GEOMETRY_PRIMARY, arm, &mut summary);
    emit_fourth_run_scratch_findings("l", arm.name, &summary.counts);
    // 7.2（第 3 次跑那一行）：L0 格数 4 × 2；L3 4 × 3 × 2；L4 4 × 2 × 2；L6 4 × 2（按实际跑到被测挂载的格数）。
    for (family, registered) in [
        ("l0", 4 * 2),
        ("l3", 4 * 3 * 2),
        ("l4", 4 * 2 * 2),
        ("l6", 4 * 2),
    ] {
        let computed = summary
            .cells
            .get(&format!("{family}:{}", GEOMETRY_PRIMARY.label))
            .copied()
            .unwrap_or(0);
        emit_result(&format!(
            "name=r4_section_seven_two_anchor arm={} item=\"{family} 格数\" computed={computed} registered={registered} verdict={}",
            arm.name,
            if computed == registered { "pass" } else { "fail_v4" }
        ));
    }
}

// ---------------------------------------------------------------------------
// 12.10 第五段：8.2 的敏感性点（G_S4、G_S16、G_默认、G_小环、k = 2、R = 3），只重跑判决格（登记 8.2 的取样点表）。
// ---------------------------------------------------------------------------

/// mkfs 默认环长的几何点 G_默认 = (S = 8, 768 MiB, 4 GiB)（登记 8.2）。
const GEOMETRY_DEFAULT_RING: Geometry = Geometry {
    label: "G_default(S=8,ring=768MiB)",
    slots_per_region: 8,
    journal_ring_bytes: 768 * 1024 * 1024,
};
/// 小环的取法（第 3 次跑登记 8.2 末）：从 48 KiB 起、每次加 16 KiB。上界是装置自己的停手线（取不到记「这一点取不到」）。
const SMALL_RING_START_BYTES: u64 = 48 * 1024;
const SMALL_RING_STEP_BYTES: u64 = 16 * 1024;
const SMALL_RING_LAST_TRIED_BYTES: u64 = 3 * 1024 * 1024;

/// G_小环：第一个 mkfs 收、且 P(3, U) 接 H1e(m = 4)（不注入）整段跑得通的环长。
fn small_ring_geometry() -> Result<Geometry, String> {
    let mut ring_bytes = SMALL_RING_START_BYTES;
    while ring_bytes <= SMALL_RING_LAST_TRIED_BYTES {
        let label: &'static str =
            Box::leak(format!("G_small_ring(S=8,ring={ring_bytes})").into_boxed_str());
        let geometry = Geometry {
            label,
            slots_per_region: 8,
            journal_ring_bytes: ring_bytes,
        };
        let parameters = parameters_for(&geometry);
        let runs_through =
            fourth_run_plain_prefix(&geometry, 3, Closing::NormalUnmount, "small-ring-search")
                .and_then(|prefix| {
                    let mut history = prefix.history;
                    fourth_run_mount_writable_into_the_history(&mut history, &parameters)?;
                    for _ in 0..4 {
                        third_run_change_content(&mut history, &parameters)?;
                    }
                    Ok(())
                });
        emit_result(&format!(
            "name=r4_small_ring_try ring_bytes={ring_bytes} runs_through={}",
            runs_through.as_ref().map_or_else(
                |error| format!("no:{}", error_member_of_debug(error)),
                |()| "yes".to_string()
            )
        ));
        if runs_through.is_ok() {
            return Ok(geometry);
        }
        ring_bytes += SMALL_RING_STEP_BYTES;
    }
    Err(format!(
        "{SMALL_RING_START_BYTES}..={SMALL_RING_LAST_TRIED_BYTES} 字节里没有一个环长跑得通"
    ))
}

fn run_fourth_run_segment_five(arm: FourthRunArm) {
    if arm.rereads() && arm.reread_rounds > 1 {
        // R = 3（登记 8.2）：乙、乙-窄读 的重读轮数加到 3，时长 T(δ)–T(4δ)，H1d 的 `none`、H1e。
        run_fourth_run_held_once_like_family(
            "h1d-uncut-r3",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::UncutOnly,
        );
        run_fourth_run_held_once_like_family(
            "h1e-r3",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::Tail,
        );
        return;
    }
    let mut summary = FourthRunLegalAndFaultStateTally::default();
    for geometry in [&GEOMETRY_SMALLER_ROOT_RING, &GEOMETRY_LARGER_ROOT_RING] {
        run_fourth_run_held_once_like_family(
            "h1d-uncut",
            geometry,
            arm,
            1,
            FourthRunContinuation::UncutOnly,
        );
        run_fourth_run_held_once_like_family("h1e", geometry, arm, 1, FourthRunContinuation::Tail);
        run_fourth_run_fault_state_families(geometry, arm, &["l2"], &mut summary);
        run_fourth_run_shadow_ledger_families(geometry, arm, &mut summary);
    }
    run_fourth_run_held_once_like_family(
        "h1d-uncut",
        &GEOMETRY_DEFAULT_RING,
        arm,
        1,
        FourthRunContinuation::UncutOnly,
    );
    run_fourth_run_held_once_like_family(
        "h1e",
        &GEOMETRY_DEFAULT_RING,
        arm,
        1,
        FourthRunContinuation::Tail,
    );
    run_fourth_run_fault_state_families(&GEOMETRY_DEFAULT_RING, arm, &["l3"], &mut summary);
    match small_ring_geometry() {
        Ok(geometry) => {
            emit_result(&format!(
                "name=r4_small_ring arm={} geometry={} verdict=found",
                arm.name, geometry.label
            ));
            run_fourth_run_held_once_like_family(
                "h1e",
                &geometry,
                arm,
                1,
                FourthRunContinuation::Tail,
            );
            run_fourth_run_fault_state_families(&geometry, arm, &["l3"], &mut summary);
        }
        Err(error) => emit_result(&format!(
            "name=r4_small_ring arm={} verdict=not_obtainable reason={error:?}",
            arm.name
        )),
    }
    // k = 2（登记 8.2）：A 藏最新 2 条根的根槽，断链落在两条里较旧那一次发布上；H1d 的 `none`、H1e。
    run_fourth_run_held_once_like_family(
        "h1d-uncut-k2",
        &GEOMETRY_PRIMARY,
        arm,
        2,
        FourthRunContinuation::UncutOnly,
    );
    run_fourth_run_held_once_like_family(
        "h1e-k2",
        &GEOMETRY_PRIMARY,
        arm,
        2,
        FourthRunContinuation::Tail,
    );
    emit_fourth_run_scratch_findings("seg5", arm.name, &summary.counts);
}

// ---------------------------------------------------------------------------
// 12.12 跨臂对拍（`r4-compare <产物前缀>…`）：读每个前缀下各臂的产物 `<前缀>-<臂>.out`，算跨臂的句子（与今天逐项相同、PC-多读、PC-多拒、
//       PC-22）、Q4（多拒、拦下）、Q5 两栏、7.3 第二三行、F11、起始镜像跨臂同一份，最后按 5.4 每一句的标签机械算 V1 的作废范围（`r4_void`）。
// ---------------------------------------------------------------------------

/// 一句对照的判定（在臂里打出的，或 compare 里跨臂算的）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SentenceRecord {
    arm: String,
    control: String,
    name: String,
    tag: SentenceTag,
    verdict: SentenceVerdict,
}

/// 一条对照管哪几个量（登记 11.1 V1 末：PC-N-环、PC-N-盖、PC-O、PC-丢写 ⇒ Q1–Q3；PC-随 ⇒ H-随 那一栏的 Q1–Q3；PC-N0、PC-多拒 ⇒ Q4；
/// PC-多读 ⇒ Q5（(i) 全部、(ii) 同成、(iii) 拒）；PC-22 ⇒ Q6；PC-只读 ⇒ 甲那一句「只读照常」；PC-554 ⇒ F1 与那几臂的 Q4）。
fn quantities_of_a_control(control: &str) -> &'static [&'static str] {
    match control {
        "PC-N-ring" | "PC-N-cover" | "PC-O" | "PC-lost-write" => &["Q1-Q3"],
        "PC-random" => &["Q1-Q3-random-histories"],
        "PC-N0" | "PC-extra-refusal" | "PC-554" => &["Q4"],
        "PC-extra-reads-i" => &["Q5-both-mounted", "Q5-refused"],
        "PC-extra-reads-ii" => &["Q5-both-mounted"],
        "PC-extra-reads-iii" => &["Q5-refused"],
        "PC-22" => &["Q6"],
        "PC-read-only" => &["read-only"],
        _ => &[],
    }
}

/// 拿今天当参照的量（V1 ②：[测] 句在今天那一臂上打不中，别的臂只作废这几样）。
const QUANTITIES_REFERRING_TO_TODAY: [&str; 3] = ["Q4", "Q5-both-mounted", "Q5-refused"];
const EVERY_VOIDABLE_QUANTITY: [&str; 7] = [
    "Q1-Q3",
    "Q1-Q3-random-histories",
    "Q4",
    "Q5-both-mounted",
    "Q5-refused",
    "Q6",
    "read-only",
];

/// (臂, 量) → 作废的依据（空 = 不作废）。
type VoidedQuantities = BTreeMap<(String, &'static str), Vec<String>>;

/// V1（登记 11.1，按句子标签分五种）的作废范围：交回 (臂, 量) → 作废的依据（空 = 不作废），外加「装置缺陷（全部臂一起）」的那几句。
fn fourth_run_void_scope(
    sentences: &[SentenceRecord],
    arms: &[&str],
) -> (VoidedQuantities, Vec<String>) {
    let mut voided: VoidedQuantities = BTreeMap::new();
    for arm in arms {
        for quantity in EVERY_VOIDABLE_QUANTITY {
            voided.insert(((*arm).to_string(), quantity), Vec::new());
        }
    }
    let missed = |record: &SentenceRecord| {
        matches!(
            record.verdict,
            SentenceVerdict::Fail | SentenceVerdict::NotConstructible
        )
    };
    let mut device_defects = Vec::new();
    let mut measured_misses_by_sentence: BTreeMap<(String, String), BTreeSet<String>> =
        BTreeMap::new();
    for record in sentences.iter().filter(|record| missed(record)) {
        let basis = format!("{}:{}({})", record.control, record.name, record.tag.name());
        match record.tag {
            SentenceTag::TodayDescription => {}
            SentenceTag::ArmDefinition => {
                for quantity in quantities_of_a_control(&record.control) {
                    if let Some(list) = voided.get_mut(&(record.arm.clone(), *quantity)) {
                        list.push(basis.clone());
                    }
                }
            }
            SentenceTag::Measured => {
                measured_misses_by_sentence
                    .entry((record.control.clone(), record.name.clone()))
                    .or_default()
                    .insert(record.arm.clone());
                if record.arm == "today" {
                    for quantity in quantities_of_a_control(&record.control) {
                        if let Some(list) = voided.get_mut(&("today".to_string(), *quantity)) {
                            list.push(basis.clone());
                        }
                    }
                    for arm in arms.iter().filter(|arm| **arm != "today") {
                        for quantity in quantities_of_a_control(&record.control)
                            .iter()
                            .filter(|quantity| QUANTITIES_REFERRING_TO_TODAY.contains(quantity))
                        {
                            if let Some(list) = voided.get_mut(&((*arm).to_string(), *quantity)) {
                                list.push(format!("{basis}@today"));
                            }
                        }
                    }
                } else {
                    for quantity in quantities_of_a_control(&record.control) {
                        if let Some(list) = voided.get_mut(&(record.arm.clone(), *quantity)) {
                            list.push(basis.clone());
                        }
                    }
                }
            }
        }
    }
    for ((control, name), missing_arms) in &measured_misses_by_sentence {
        let asked_of: BTreeSet<&str> = sentences
            .iter()
            .filter(|record| &record.control == control && &record.name == name)
            .map(|record| record.arm.as_str())
            .collect();
        if asked_of.len() > 1 && asked_of.iter().all(|arm| missing_arms.contains(*arm)) {
            device_defects.push(format!("{control}:{name}"));
            for arm in arms {
                for quantity in quantities_of_a_control(control) {
                    if let Some(list) = voided.get_mut(&((*arm).to_string(), *quantity)) {
                        list.push(format!("{control}:{name}(measure,every_arm)"));
                    }
                }
            }
        }
    }
    (voided, device_defects)
}

/// PC-多读 (iii) 与 Q5-拒 的参照臂：同一种 N 的甲臂（登记 5.8 第二件：乙 的参照不再是今天那一臂；变异 M20 把它换回今天）。
fn extra_reads_reference_arm(arm: FourthRunArm) -> Option<&'static str> {
    arm.jia_arm_with_the_same_witness()
}

/// 产物里的一行（字段表）与它来自哪条臂。
struct ProductLine {
    fields: BTreeMap<String, String>,
}

impl ProductLine {
    fn text(&self, name: &str) -> &str {
        self.fields.get(name).map_or("", String::as_str)
    }

    fn number(&self, name: &str) -> Option<i64> {
        self.fields.get(name).and_then(|value| value.parse().ok())
    }

    fn key(&self, coordinate_names: &[&str]) -> String {
        coordinate_names
            .iter()
            .filter_map(|name| {
                self.fields
                    .get(*name)
                    .map(|value| format!("{name}={value}"))
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

const LEGAL_OR_FAULT_STATE_CELL_COORDINATES: [&str; 14] = [
    "family", "geometry", "n1", "c", "k", "which", "form", "point", "of", "b", "page", "m",
    "duration", "seed",
];
const CONTROL_COORDINATES: [&str; 7] = ["control", "form", "duration", "n1", "c", "of", "b"];

/// 被测挂载那几行（Q4、Q5 的输入）：H1d 那样的族只取不断那一格，接尾巴的族只取第一个 m（A 的读数与 m 无关），L 族与第二段的每一次被测挂载全取。
fn tested_mount_lines(lines: &[ProductLine]) -> BTreeMap<String, &ProductLine> {
    let mut tested = BTreeMap::new();
    for line in lines {
        let key = match line.text("name") {
            "r4_cell"
                if line.text("cut") == "none"
                    || (line.text("cut").is_empty() && line.text("m") == "1") =>
            {
                format!(
                    "cell {}",
                    line.key(&[
                        "family", "geometry", "n1", "c", "n1a", "n1b", "a", "b", "form", "duration"
                    ])
                )
            }
            "r4_l_cell" if line.text("verdict").is_empty() => {
                format!("l {}", line.key(&LEGAL_OR_FAULT_STATE_CELL_COORDINATES))
            }
            "r4_random_mount" => format!(
                "random {}",
                line.key(&["family", "seed", "step", "attempt"])
            ),
            _ => continue,
        };
        tested.entry(key).or_insert(line);
    }
    tested
}

/// K0–K2 是挂载交回 `Ok`（只差空间准入怎么判），K3、K4 是拒（`mount_outcome_class`）。
fn class_is_a_successful_mount(class: &str) -> bool {
    matches!(class, "K0" | "K1" | "K2")
}

fn class_is_a_refusal(class: &str) -> bool {
    matches!(class, "K3" | "K4")
}

fn duration_class(duration: &str) -> &'static str {
    if duration.starts_with('T') {
        "T"
    } else if duration.starts_with('O') {
        "O"
    } else {
        "W"
    }
}

/// Q5-同成 的一格：臂与今天那一臂都交回 `Ok`、起始镜像逐字节相同（变异 M18：把臂拒了的格也算进来）。
fn both_mounted_extra_reads(
    arm_cell: &ProductLine,
    today_cell: &ProductLine,
) -> Option<(i64, i64)> {
    let same_start = !arm_cell.text("base_fingerprint").is_empty()
        && arm_cell.text("base_fingerprint") == today_cell.text("base_fingerprint");
    let both_ok = class_is_a_successful_mount(arm_cell.text("a_class"))
        && class_is_a_successful_mount(today_cell.text("a_class"));
    if !(same_start && both_ok) {
        return None;
    }
    Some((
        arm_cell.number("a_reads")? - today_cell.number("a_reads")?,
        arm_cell.number("a_read_bytes")? - today_cell.number("a_read_bytes")?,
    ))
}

fn read_product_lines(path: &str) -> Result<Vec<ProductLine>, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    let finished = text
        .lines()
        .last()
        .is_some_and(|line| line.starts_with("E7RESULT name=done emitted="));
    if !finished {
        return Err(format!("{path}: 末行不是完成标记"));
    }
    Ok(text
        .lines()
        .filter(|line| line.starts_with("E7RESULT "))
        .map(|line| ProductLine {
            fields: fields_of_a_result_line(line),
        })
        .collect())
}

fn sentence_records_of(arm: &str, lines: &[ProductLine]) -> Vec<SentenceRecord> {
    let mut records = Vec::new();
    for line in lines
        .iter()
        .filter(|line| line.text("name") == "r4_positive_control")
    {
        for (field, value) in &line.fields {
            let Some(rest) = field.strip_prefix("sentence_") else {
                continue;
            };
            let (tag, name) = if let Some(name) = rest.strip_prefix("measure_") {
                (SentenceTag::Measured, name)
            } else if let Some(name) = rest.strip_prefix("arm_") {
                (SentenceTag::ArmDefinition, name)
            } else if let Some(name) = rest.strip_prefix("today_") {
                (SentenceTag::TodayDescription, name)
            } else {
                continue;
            };
            let verdict = match value.as_str() {
                "pass" => SentenceVerdict::Pass,
                "fail" => SentenceVerdict::Fail,
                "not_constructible" => SentenceVerdict::NotConstructible,
                _ => SentenceVerdict::Vacuous,
            };
            records.push(SentenceRecord {
                arm: arm.to_string(),
                control: line.text("control").to_string(),
                name: format!(
                    "{}[{}]",
                    name,
                    line.key(&["form", "duration", "n1", "c", "of"])
                ),
                tag,
                verdict,
            });
        }
    }
    records
}

fn emit_cross_arm_sentence(record: &SentenceRecord, detail: &str) {
    emit_result(&format!(
        "name=r4_cross_arm_sentence arm={} control={} sentence={} tag={} verdict={} {detail}",
        record.arm,
        record.control,
        record.name,
        record.tag.name(),
        record.verdict.name()
    ));
}

#[allow(
    clippy::too_many_lines,
    reason = "跨臂对拍的每一样按登记的次序写在一处，拆开只多几层参数搬运"
)]
fn run_fourth_run_compare(prefixes: &[String]) {
    let mut lines_by_arm: BTreeMap<&'static str, Vec<ProductLine>> = BTreeMap::new();
    for arm in FOURTH_RUN_ARMS {
        let mut lines = Vec::new();
        for prefix in prefixes {
            let path = format!("{prefix}-{}.out", arm.name);
            if !std::path::Path::new(&path).exists() {
                continue;
            }
            match read_product_lines(&path) {
                Ok(read) => {
                    emit_result(&format!(
                        "name=r4_compare_input arm={} path={path} lines={} verdict=pass",
                        arm.name,
                        read.len()
                    ));
                    lines.extend(read);
                }
                Err(error) => emit_result(&format!(
                    "name=r4_compare_input arm={} verdict=stop reason={error:?}",
                    arm.name
                )),
            }
        }
        if !lines.is_empty() {
            lines_by_arm.insert(arm.name, lines);
        }
    }
    let Some(today_lines) = lines_by_arm.get("today") else {
        emit_result(
            "name=r4_compare_input arm=today verdict=stop reason=\"今天那一臂的产物一份都没有\"",
        );
        return;
    };
    let today_tested = tested_mount_lines(today_lines);
    let today_controls: BTreeMap<String, &ProductLine> = today_lines
        .iter()
        .filter(|line| line.text("name") == "r4_positive_control")
        .map(|line| (line.key(&CONTROL_COORDINATES), line))
        .collect();
    let mut sentences: Vec<SentenceRecord> = Vec::new();
    for (arm_name, lines) in &lines_by_arm {
        sentences.extend(sentence_records_of(arm_name, lines));
    }
    // PC-多读 (i)：今天那一臂减自己 = 0（每一格）。
    emit_result(&format!(
        "name=r4_section_seven_three row=4 arm=today cells={} nonzero_cells=0 verdict=pass",
        today_tested.len()
    ));
    sentences.push(SentenceRecord {
        arm: "today".to_string(),
        control: "PC-extra-reads-i".to_string(),
        name: "today_minus_itself_is_zero".to_string(),
        tag: SentenceTag::Measured,
        verdict: SentenceVerdict::Pass,
    });
    for (arm_name, lines) in &lines_by_arm {
        let Some(arm) = fourth_run_arm_named(arm_name) else {
            continue;
        };
        if arm.is_today() {
            continue;
        }
        let tested = tested_mount_lines(lines);
        // Q5 两栏、Q4（按族分报）。
        let mut both_mounted: BTreeMap<String, ExtraReads> = BTreeMap::new();
        let mut refused_minus_today: BTreeMap<String, ExtraReads> = BTreeMap::new();
        let mut extra_refusals: BTreeMap<String, u64> = BTreeMap::new();
        let mut intercepted: BTreeMap<String, u64> = BTreeMap::new();
        let mut compared: BTreeMap<String, u64> = BTreeMap::new();
        let mut diverged_mounts: BTreeMap<String, u64> = BTreeMap::new();
        let mut diverged_refusals: BTreeMap<String, u64> = BTreeMap::new();
        for (key, cell) in &tested {
            let family = format!("{}@{}", cell.text("family"), cell.text("geometry"));
            let today_cell = today_tested.get(key);
            // 第二段那几族整段在各臂上各跑一遍（登记 5.3）：与今天走岔之后的挂载（起始镜像不同，或今天那一臂没有这一格）没有
            // 「今天的同一格」，只按「臂拒 ∧ N_真 为假」记，单列一栏（r3 登记「读法写死」表「多拒」一行）。
            if cell.text("name") == "r4_random_mount"
                && today_cell.is_none_or(|today_cell| {
                    cell.text("base_fingerprint") != today_cell.text("base_fingerprint")
                })
            {
                *diverged_mounts.entry(family.clone()).or_insert(0) += 1;
                if class_is_a_refusal(cell.text("a_class")) && cell.text("truth_newer") != "true" {
                    *diverged_refusals.entry(family).or_insert(0) += 1;
                }
                continue;
            }
            let Some(today_cell) = today_cell else {
                continue;
            };
            *compared.entry(family.clone()).or_insert(0) += 1;
            if let Some((calls, bytes)) = both_mounted_extra_reads(cell, today_cell) {
                both_mounted
                    .entry(family.clone())
                    .or_default()
                    .add(calls, bytes);
            }
            let arm_refused = class_is_a_refusal(cell.text("a_class"));
            let today_refused = class_is_a_refusal(today_cell.text("a_class"));
            if arm_refused {
                if let (Some(calls), Some(today_calls)) =
                    (cell.number("a_reads"), today_cell.number("a_reads"))
                {
                    refused_minus_today.entry(family.clone()).or_default().add(
                        calls - today_calls,
                        cell.number("a_read_bytes").unwrap_or(0)
                            - today_cell.number("a_read_bytes").unwrap_or(0),
                    );
                }
            }
            if arm_refused && !today_refused {
                let truth_newer = cell.text("truth_newer") == "true"
                    || cell.text("truth_newer_on_the_twin") == "true";
                if truth_newer {
                    *intercepted
                        .entry(format!(
                            "{family}:{}",
                            duration_class(cell.text("duration"))
                        ))
                        .or_insert(0) += 1;
                } else {
                    *extra_refusals.entry(family.clone()).or_insert(0) += 1;
                }
            }
        }
        for (family, cells) in &compared {
            emit_result(&format!(
                "name=r4_q4 arm={arm_name} family={family} compared_tested_mounts={cells} extra_refusals={} intercepted={:?}",
                extra_refusals.get(family).copied().unwrap_or(0),
                intercepted
                    .iter()
                    .filter(|(key, _)| key.starts_with(&format!("{family}:")))
                    .map(|(key, count)| format!("{}={count}", key.rsplit(':').next().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        for (family, mounts) in &diverged_mounts {
            emit_result(&format!(
                "name=r4_q4_after_divergence arm={arm_name} family={family} tested_mounts_after_divergence={mounts} refusals_with_truth_newer_false={}",
                diverged_refusals.get(family).copied().unwrap_or(0)
            ));
        }
        for (column, table) in [
            ("both_mounted", &both_mounted),
            ("refused_minus_today", &refused_minus_today),
        ] {
            for (family, extra) in table {
                emit_result(&format!(
                    "name=r4_q5 arm={arm_name} column={column} family={family} cells={} positive_cells={} total_extra_read_calls={} total_extra_read_bytes={} minimum_extra_read_calls={} maximum_extra_read_calls={}",
                    extra.cells,
                    extra.positive_cells,
                    extra.total_calls,
                    extra.total_bytes,
                    extra.minimum_calls.map_or("none".to_string(), |value| value.to_string()),
                    extra.maximum_calls.map_or("none".to_string(), |value| value.to_string())
                ));
            }
        }
        // 与今天逐项相同：PC-N0（F12，全部臂）与 PC-N-环 / PC-N-盖 的读回全零格（-槽 臂，N-槽 判假）。
        for line in lines
            .iter()
            .filter(|line| line.text("name") == "r4_positive_control")
        {
            let control = line.text("control");
            let needs_same_as_today = control == "PC-N0"
                || (matches!(control, "PC-N-ring" | "PC-N-cover")
                    && arm.witness == FourthRunWitness::UnreadableRootRingSlot
                    && line.text("form") == "reads_zeros");
            if !needs_same_as_today {
                continue;
            }
            let key = line.key(&CONTROL_COORDINATES);
            let differing: Vec<&str> = match today_controls.get(&key) {
                Some(today_line) => ["a_class", "a_chosen", "a_effective", "write_fingerprint"]
                    .into_iter()
                    .filter(|field| line.text(field) != today_line.text(field))
                    .collect(),
                None => vec!["today_line_missing"],
            };
            let record = SentenceRecord {
                arm: arm_name.to_string(),
                control: control.to_string(),
                name: format!(
                    "same_as_today[{}]",
                    line.key(&["form", "duration", "n1", "c"])
                ),
                tag: SentenceTag::ArmDefinition,
                verdict: SentenceVerdict::of(differing.is_empty()),
            };
            emit_cross_arm_sentence(
                &record,
                &format!(
                    "differing_fields={}",
                    if differing.is_empty() {
                        "none".to_string()
                    } else {
                        differing.join(",")
                    }
                ),
            );
            sentences.push(record);
        }
        // PC-随（登记 5.4）：-槽 各臂在 H-随 那三段上，到崩溃恢复抛弃根那一步为止与今天逐项相同（每一步做完时的镜像指纹）。
        if arm.witness == FourthRunWitness::UnreadableRootRingSlot {
            for line in lines.iter().filter(|line| {
                line.text("name") == "r4_positive_control" && line.text("control") == "PC-random"
            }) {
                let label = line.text("seed");
                let last = line.number("crash_recovery_order");
                let fingerprints_of = |step_lines: &[ProductLine]| -> BTreeMap<i64, String> {
                    step_lines
                        .iter()
                        .filter(|step| {
                            step.text("name") == "r4_random_step"
                                && step.text("family") == "h-random"
                                && step.text("seed") == label
                        })
                        .filter_map(|step| {
                            Some((
                                step.number("order")?,
                                step.text("fingerprint_after").to_string(),
                            ))
                        })
                        .collect()
                };
                let arm_fingerprints = fingerprints_of(lines);
                let today_fingerprints = fingerprints_of(today_lines);
                let differing: Vec<i64> = match last {
                    Some(last) => (0..=last)
                        .filter(|order| {
                            !arm_fingerprints.contains_key(order)
                                || arm_fingerprints.get(order) != today_fingerprints.get(order)
                        })
                        .collect(),
                    None => Vec::new(),
                };
                let record = SentenceRecord {
                    arm: arm_name.to_string(),
                    control: "PC-random".to_string(),
                    name: format!(
                        "slot_arm_same_as_today_up_to_the_crash_recovery_step[of={}]",
                        line.text("of")
                    ),
                    tag: SentenceTag::ArmDefinition,
                    verdict: if last.is_none() {
                        SentenceVerdict::NotConstructible
                    } else {
                        SentenceVerdict::of(differing.is_empty())
                    },
                };
                emit_cross_arm_sentence(
                    &record,
                    &format!("seed={label} differing_orders={differing:?}").replace(' ', ""),
                );
                sentences.push(record);
            }
        }
        // PC-多读 (ii)：L0 上 Q5-同成 的点值（甲 / 乙 / 乙-窄读 -配置 +4、-配置续 +8、-槽 +3S；丙-X = 甲-X 的点值减 3S + p）。
        for line in lines.iter().filter(|line| {
            line.text("name") == "r4_positive_control" && line.text("control") == "PC-N0"
        }) {
            let key = line.key(&CONTROL_COORDINATES);
            let Some(today_line) = today_controls.get(&key) else {
                continue;
            };
            let three_slots_per_region =
                i64::try_from(ROOT_RING_REGIONS_LOCAL * GEOMETRY_PRIMARY.slots_per_region)
                    .expect("24");
            let witness_point = match arm.witness {
                FourthRunWitness::NotJudged => 0,
                FourthRunWitness::SystemConfiguration => 4,
                FourthRunWitness::SystemConfigurationCarried => 8,
                FourthRunWitness::UnreadableRootRingSlot => three_slots_per_region,
            };
            let pages = today_line.number("effective_instance_table_pages");
            let expected = match arm.candidate {
                FourthRunCandidate::Bing => {
                    pages.map(|pages| witness_point - (three_slots_per_region + pages))
                }
                FourthRunCandidate::Jia
                | FourthRunCandidate::Yi
                | FourthRunCandidate::YiNarrow
                | FourthRunCandidate::Today => Some(witness_point),
            };
            let measured = both_mounted_extra_reads(line, today_line).map(|(calls, _)| calls);
            let record = SentenceRecord {
                arm: arm_name.to_string(),
                control: "PC-extra-reads-ii".to_string(),
                name: format!("point_value[{}]", line.key(&["n1", "c"])),
                tag: SentenceTag::ArmDefinition,
                verdict: match (measured, expected) {
                    (Some(measured), Some(expected)) => SentenceVerdict::of(measured == expected),
                    (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                },
            };
            emit_cross_arm_sentence(&record, &format!("measured_extra_read_calls={measured:?} expected={expected:?} instance_table_pages={pages:?}").replace(' ', ""));
            sentences.push(record);
        }
        // PC-多读 (iii)：PC-N-环 与 PC-N-盖 的 W 格。
        if let Some(jia_name) = extra_reads_reference_arm(arm) {
            if arm.rereads() || arm.candidate == FourthRunCandidate::Bing {
                let jia_controls: BTreeMap<String, &ProductLine> = lines_by_arm
                    .get(jia_name)
                    .map(|jia_lines| {
                        jia_lines
                            .iter()
                            .filter(|line| line.text("name") == "r4_positive_control")
                            .map(|line| (line.key(&CONTROL_COORDINATES), line))
                            .collect()
                    })
                    .unwrap_or_default();
                for line in lines.iter().filter(|line| {
                    line.text("name") == "r4_positive_control"
                        && matches!(line.text("control"), "PC-N-ring" | "PC-N-cover")
                        && line.text("duration") == "W"
                }) {
                    if arm.witness == FourthRunWitness::UnreadableRootRingSlot
                        && line.text("form") != "read_fails"
                    {
                        continue;
                    }
                    let key = line.key(&CONTROL_COORDINATES);
                    let jia_total = jia_controls
                        .get(&key)
                        .and_then(|jia_line| jia_line.number("a_reads"));
                    let total = line.number("a_reads");
                    let coordinates = line.key(&["control", "form"]);
                    let mut push =
                        |name: &str, tag: SentenceTag, verdict: SentenceVerdict, detail: String| {
                            let record = SentenceRecord {
                                arm: arm_name.to_string(),
                                control: "PC-extra-reads-iii".to_string(),
                                name: format!("{name}[{coordinates}]"),
                                tag,
                                verdict,
                            };
                            emit_cross_arm_sentence(&record, &detail);
                            sentences.push(record);
                        };
                    if arm.rereads() {
                        let before = line.number("a_reads_before_first_wait");
                        let after = line.number("a_reads_after_first_wait");
                        let faults_before = line.number("a_faults_intercepted_before_first_wait");
                        push(
                            "first_pass_reads_equal_jia_total",
                            SentenceTag::ArmDefinition,
                            match (before, jia_total) {
                                (Some(before), Some(jia_total)) => {
                                    SentenceVerdict::of(before == jia_total)
                                }
                                (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                            },
                            format!("before={before:?} jia_total={jia_total:?}").replace(' ', ""),
                        );
                        let rounds = i64::try_from(arm.reread_rounds).expect("R");
                        push(
                            "reads_after_the_mark_at_least_r_times_f_and_positive",
                            SentenceTag::ArmDefinition,
                            match (after, faults_before) {
                                (Some(after), Some(faults)) => {
                                    SentenceVerdict::of(after >= rounds * faults && after > 0)
                                }
                                (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                            },
                            format!("after={after:?} f={faults_before:?} r={rounds}")
                                .replace(' ', ""),
                        );
                        push(
                            "total_is_before_plus_after",
                            SentenceTag::Measured,
                            match (total, before, after) {
                                (Some(total), Some(before), Some(after)) => {
                                    SentenceVerdict::of(total == before + after)
                                }
                                (None, _, _) | (_, None, _) | (_, _, None) => {
                                    SentenceVerdict::NotConstructible
                                }
                            },
                            format!("total={total:?}").replace(' ', ""),
                        );
                    } else if arm.witness != FourthRunWitness::UnreadableRootRingSlot {
                        push(
                            "bing_minus_jia_positive",
                            SentenceTag::ArmDefinition,
                            match (total, jia_total) {
                                (Some(total), Some(jia_total)) => {
                                    SentenceVerdict::of(total - jia_total > 0)
                                }
                                (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                            },
                            format!("total={total:?} jia_total={jia_total:?}").replace(' ', ""),
                        );
                    }
                }
            }
        }
    }
    compare_extra_refusal_and_cell_22_controls(&lines_by_arm, &today_tested, &mut sentences);
    compare_row_two_three_and_clause_eleven(&lines_by_arm);
    compare_base_identity(&lines_by_arm);
    compare_losses(&lines_by_arm);
    let arm_names: Vec<&str> = lines_by_arm.keys().copied().collect();
    let (voided, device_defects) = fourth_run_void_scope(&sentences, &arm_names);
    for defect in &device_defects {
        emit_result(&format!("name=r4_device_defect sentence={defect:?} verdict=stop reason=\"同一句 [测] 在每一臂上都打不中（V1 ③）\""));
    }
    for ((arm, quantity), bases) in &voided {
        emit_result(&format!(
            "name=r4_void arm={arm} quantity={quantity} voided={} basis={}",
            !bases.is_empty(),
            if bases.is_empty() {
                "none".to_string()
            } else {
                bases.join(",")
            }
        ));
    }
    let missed_routes: Vec<String> = sentences
        .iter()
        .filter(|record| {
            record.tag != SentenceTag::Measured
                && matches!(
                    record.verdict,
                    SentenceVerdict::Fail | SentenceVerdict::NotConstructible
                )
        })
        .map(|record| {
            format!(
                "{}:{}:{}({})",
                record.arm,
                record.control,
                record.name,
                record.tag.name()
            )
        })
        .collect();
    emit_result(&format!(
        "name=r4_missed_arm_and_today_sentences count={} list={}",
        missed_routes.len(),
        if missed_routes.is_empty() {
            "none".to_string()
        } else {
            missed_routes.join(",")
        }
    ));
}

/// PC-多拒（-配置 各臂 L3(a) `read_fails` n1 = 1；-槽 各臂 L2 k = 2 `read_fails` n1 = 1；今天不 `Ok` 就顺延 n1）与 PC-22（L5）。
fn compare_extra_refusal_and_cell_22_controls(
    lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>,
    today_tested: &BTreeMap<String, &ProductLine>,
    sentences: &mut Vec<SentenceRecord>,
) {
    for (arm_name, lines) in lines_by_arm {
        let Some(arm) = fourth_run_arm_named(arm_name) else {
            continue;
        };
        if arm.is_today() || arm.witness == FourthRunWitness::NotJudged {
            continue;
        }
        let tested = tested_mount_lines(lines);
        let (family, fixed) = match arm.witness {
            FourthRunWitness::UnreadableRootRingSlot => ("l2", "k=2"),
            FourthRunWitness::SystemConfiguration
            | FourthRunWitness::SystemConfigurationCarried
            | FourthRunWitness::NotJudged => ("l3", "which=a"),
        };
        let mut found = None;
        for overwrites in [1u64, 2, 3] {
            let key_part = format!(
                "family={family} geometry={} n1={overwrites} {fixed} form=read_fails",
                GEOMETRY_PRIMARY.label
            );
            let today_cell = today_tested
                .iter()
                .find(|(key, _)| key.starts_with("l ") && key.contains(&key_part))
                .map(|(_, cell)| *cell);
            if today_cell.is_some_and(|cell| class_is_a_successful_mount(cell.text("a_class"))) {
                let arm_cell = tested
                    .iter()
                    .find(|(key, _)| key.starts_with("l ") && key.contains(&key_part))
                    .map(|(_, cell)| *cell);
                found = Some((overwrites, arm_cell));
                break;
            }
        }
        let mut push = |name: &str, tag: SentenceTag, verdict: SentenceVerdict, detail: String| {
            let record = SentenceRecord {
                arm: arm_name.to_string(),
                control: "PC-extra-refusal".to_string(),
                name: name.to_string(),
                tag,
                verdict,
            };
            emit_cross_arm_sentence(&record, &detail);
            sentences.push(record);
        };
        match found {
            None => push(
                "today_ok_on_the_cell",
                SentenceTag::TodayDescription,
                SentenceVerdict::NotConstructible,
                "n1=1,2,3".to_string(),
            ),
            Some((overwrites, arm_cell)) => {
                push(
                    "today_ok_on_the_cell",
                    SentenceTag::TodayDescription,
                    SentenceVerdict::Pass,
                    format!("n1={overwrites}"),
                );
                let refused = arm_cell.is_some_and(|cell| cell.text("refused") == "true");
                let truth_newer =
                    arm_cell.is_some_and(|cell| cell.text("truth_newer_on_the_twin") == "true");
                push(
                    "arm_refuses",
                    SentenceTag::ArmDefinition,
                    SentenceVerdict::of(refused),
                    format!("n1={overwrites}"),
                );
                push(
                    "counted_as_extra_refusal",
                    SentenceTag::Measured,
                    SentenceVerdict::conditional(refused, !truth_newer),
                    format!("n1={overwrites} truth_newer={truth_newer}"),
                );
            }
        }
    }
    // PC-22：今天那一臂找一个 m（这次挂载 Ok、计数 0、隔离 0，而它自己的孪生镜像上隔离 > 0）；那个 m 上 甲 拒、乙 O(m 起) 拒 O(m 只) Ok 且隔离 = 孪生；
    // 丙：孪生镜像上那一片被读的次数 = 今天的 − 1、隔离 = 今天的。
    let cell_22_lines = |arm: &str| -> Vec<&ProductLine> {
        lines_by_arm
            .get(arm)
            .map(|lines| {
                lines
                    .iter()
                    .filter(|line| {
                        line.text("name") == "r4_q6_cell"
                            && line.text("geometry") == GEOMETRY_PRIMARY.label
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let twins = |arm: &str| -> Vec<&ProductLine> {
        lines_by_arm
            .get(arm)
            .map(|lines| {
                lines
                    .iter()
                    .filter(|line| {
                        line.text("name") == "r4_l5_twin"
                            && line.text("geometry") == GEOMETRY_PRIMARY.label
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let today_cells = cell_22_lines("today");
    let today_twins = twins("today");
    let chosen = today_cells.iter().find(|cell| {
        cell.text("ok") == "true"
            && cell.text("abandoned_roots_unreadable") == "0"
            && cell.text("isolated") == "0"
            && cell.number("twin_isolated").is_some_and(|twin| twin > 0)
    });
    let Some(chosen) = chosen else {
        let record = SentenceRecord {
            arm: "today".to_string(),
            control: "PC-22".to_string(),
            name: "an_m_with_ok_zero_count_zero_isolation".to_string(),
            tag: SentenceTag::TodayDescription,
            verdict: SentenceVerdict::NotConstructible,
        };
        emit_cross_arm_sentence(&record, "route=F2");
        sentences.push(record);
        return;
    };
    let chosen_key = chosen.key(&["n1", "b", "page", "m"]);
    let today_record = SentenceRecord {
        arm: "today".to_string(),
        control: "PC-22".to_string(),
        name: "an_m_with_ok_zero_count_zero_isolation".to_string(),
        tag: SentenceTag::TodayDescription,
        verdict: SentenceVerdict::Pass,
    };
    emit_cross_arm_sentence(&today_record, &chosen_key.replace(' ', ","));
    sentences.push(today_record);
    for arm in FOURTH_RUN_ARMS
        .iter()
        .filter(|arm| !arm.is_today() && lines_by_arm.contains_key(arm.name))
    {
        let cells = cell_22_lines(arm.name);
        let cell_at = |duration_suffix: &str| {
            cells.iter().find(|cell| {
                cell.key(&["n1", "b", "page", "m"]) == chosen_key
                    && cell.text("duration").ends_with(duration_suffix)
            })
        };
        let mut push = |name: &str, verdict: SentenceVerdict, detail: String| {
            let record = SentenceRecord {
                arm: arm.name.to_string(),
                control: "PC-22".to_string(),
                name: name.to_string(),
                tag: SentenceTag::ArmDefinition,
                verdict,
            };
            emit_cross_arm_sentence(&record, &detail);
            sentences.push(record);
        };
        let refused = |cell: Option<&&ProductLine>| {
            cell.map(|cell| SentenceVerdict::of(cell.text("ok") == "false"))
        };
        match arm.candidate {
            FourthRunCandidate::Jia => {
                for suffix in ["only", "from"] {
                    push(
                        &format!("refused_{suffix}"),
                        refused(cell_at(suffix)).unwrap_or(SentenceVerdict::NotConstructible),
                        chosen_key.replace(' ', ","),
                    );
                }
            }
            FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => {
                push(
                    "refused_from",
                    refused(cell_at("from")).unwrap_or(SentenceVerdict::NotConstructible),
                    chosen_key.replace(' ', ","),
                );
                let only_ok_and_equal = cell_at("only").map(|cell| {
                    SentenceVerdict::of(
                        cell.text("ok") == "true"
                            && cell.text("isolated") == cell.text("twin_isolated"),
                    )
                });
                push(
                    "ok_only_with_twin_isolation",
                    only_ok_and_equal.unwrap_or(SentenceVerdict::NotConstructible),
                    chosen_key.replace(' ', ","),
                );
            }
            FourthRunCandidate::Bing => {
                let base_key = chosen.key(&["n1", "b"]);
                let page: usize = chosen
                    .number("page")
                    .and_then(|page| usize::try_from(page).ok())
                    .unwrap_or(0);
                let own = twins(arm.name)
                    .into_iter()
                    .find(|twin| twin.key(&["n1", "b"]) == base_key);
                let today_twin = today_twins
                    .iter()
                    .find(|twin| twin.key(&["n1", "b"]) == base_key);
                let page_reads = |line: &ProductLine, field: &str| -> Option<i64> {
                    line.text(field)
                        .trim_matches(|character| character == '[' || character == ']')
                        .split(',')
                        .nth(page)
                        .and_then(|value| value.trim().parse().ok())
                };
                let verdict = match (own, today_twin) {
                    (Some(own), Some(today_twin)) => SentenceVerdict::of(
                        page_reads(own, "own_page_reads").is_some_and(|reads| {
                            Some(reads)
                                == page_reads(today_twin, "own_page_reads").map(|today| today - 1)
                        }) && own.text("twin_isolated") == today_twin.text("twin_isolated"),
                    ),
                    (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                };
                push(
                    "page_read_once_less_and_same_isolation_as_today",
                    verdict,
                    chosen_key.replace(' ', ","),
                );
            }
            FourthRunCandidate::Today => {}
        }
    }
}

/// 7.3 第二行（今天那一臂上装置算的 N-配置 / N-槽 为真的被测挂载 = 甲-配置 / 甲-槽 交回「N 判真」拒绝成员的那几格）、第三行（T(δ) 对今天、甲、丙 与 W 逐格相同）、F11。
fn compare_row_two_three_and_clause_eleven(
    lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>,
) {
    let Some(today_lines) = lines_by_arm.get("today") else {
        return;
    };
    let today_tested = tested_mount_lines(today_lines);
    for (witness_field, jia_name) in [("device_n_cfg", "jia-cfg"), ("device_n_slot", "jia-slot")] {
        let Some(jia_lines) = lines_by_arm.get(jia_name) else {
            continue;
        };
        let jia_tested = tested_mount_lines(jia_lines);
        let jia = fourth_run_arm_named(jia_name).expect("甲臂");
        let mut compared = 0u64;
        let mut mismatches = Vec::new();
        for (key, today_cell) in &today_tested {
            let Some(jia_cell) = jia_tested.get(key) else {
                continue;
            };
            if today_cell.text(witness_field).is_empty() || key.contains("family=h1g") {
                continue;
            }
            compared += 1;
            let device_says = today_cell.text(witness_field) == "true";
            let jia_refused = jia_cell.text("a_error") == jia.refusal_member();
            if device_says != jia_refused {
                mismatches.push(format!(
                    "{key}: 装置 {witness_field}={device_says}，{jia_name} 拒={jia_refused}"
                ));
            }
        }
        emit_result(&format!(
            "name=r4_section_seven_three row=2 witness={witness_field} jia_arm={jia_name} cells={compared} mismatches={} verdict={}",
            mismatches.len(),
            if mismatches.is_empty() { "pass" } else { "stop_s4" }
        ));
        for mismatch in &mismatches {
            emit_result(&format!(
                "name=r4_compare_finding kind=s4 detail={mismatch:?}"
            ));
        }
    }
    for (arm_name, lines) in lines_by_arm {
        let Some(arm) = fourth_run_arm_named(arm_name) else {
            continue;
        };
        if arm.rereads() {
            continue;
        }
        let tested = tested_mount_lines(lines);
        let mut compared = 0u64;
        let mut differences = Vec::new();
        for (key, cell) in tested.iter().filter(|(key, _)| key.contains("duration=T1")) {
            let Some(whole) = tested.get(&key.replace("duration=T1", "duration=W")) else {
                continue;
            };
            compared += 1;
            for field in [
                "a_class",
                "a_error",
                "a_writes",
                "a_reads",
                "a_chosen",
                "a_effective",
                "q3_read_back",
                "overwrite_cell",
                "q0_destination",
            ] {
                if cell.text(field) != whole.text(field) {
                    differences.push(format!(
                        "{key}: {field} T {} W {}",
                        cell.text(field),
                        whole.text(field)
                    ));
                }
            }
        }
        emit_result(&format!(
            "name=r4_section_seven_three row=3 arm={arm_name} cells={compared} differences={} verdict={}",
            differences.len(),
            if differences.is_empty() { "pass" } else { "void_v5" }
        ));
        for difference in &differences {
            emit_result(&format!(
                "name=r4_compare_finding arm={arm_name} kind=v5 detail={difference:?}"
            ));
        }
    }
    if let (Some(bing_lines), Some(jia_lines)) =
        (lines_by_arm.get("bing-cfg"), lines_by_arm.get("jia-cfg"))
    {
        let bing = tested_mount_lines(bing_lines);
        let jia = tested_mount_lines(jia_lines);
        let mut cells = 0u64;
        let mut differing = Vec::new();
        for (key, bing_cell) in &bing {
            if key.contains("duration=O1only")
                || key.contains("family=l5")
                || key.contains("family=pc554")
            {
                continue;
            }
            let Some(jia_cell) = jia.get(key) else {
                continue;
            };
            cells += 1;
            let jia_refused =
                jia_cell.text("a_error") == "R3NewerRootWitnessedByTheSystemConfiguration";
            let bing_refused =
                bing_cell.text("a_error") == "R3RebuildFromRecordsStoppedBelowTheWitness";
            let same = jia_refused == bing_refused
                && [
                    "a_class",
                    "a_chosen",
                    "a_effective",
                    "q3_read_back",
                    "overwrite_cell",
                ]
                .iter()
                .all(|field| bing_cell.text(field) == jia_cell.text(field));
            if !same {
                differing.push(key.clone());
            }
        }
        emit_result(&format!(
            "name=r4_f11 cells={cells} cells_differing_outside_o1_and_cell_22={} verdict={}",
            differing.len(),
            if differing.is_empty() {
                "inference_holds"
            } else {
                "inference_fails_f11"
            }
        ));
        for key in &differing {
            emit_result(&format!("name=r4_compare_finding kind=f11 detail={key:?}"));
        }
    }
}

/// 起始镜像跨臂同一份（「被测那次挂载之前的镜像一律在今天那一臂上造」的核：前缀在各臂自己进程里造的族，镜像指纹要与今天那一臂的相同；H1g 例外）。
fn compare_base_identity(lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>) {
    let Some(today_lines) = lines_by_arm.get("today") else {
        return;
    };
    let today: BTreeMap<String, &str> = today_lines
        .iter()
        .filter(|line| line.text("name") == "r4_prefix")
        .map(|line| {
            (
                line.key(&["family", "geometry", "n1", "c", "n1a", "n1b", "a"]),
                line.text("base_fingerprint"),
            )
        })
        .collect();
    for (arm_name, lines) in lines_by_arm {
        let mut compared = 0u64;
        let mut differing = Vec::new();
        for line in lines
            .iter()
            .filter(|line| line.text("name") == "r4_prefix" && line.text("family") != "h1g")
        {
            let key = line.key(&["family", "geometry", "n1", "c", "n1a", "n1b", "a"]);
            let Some(today_fingerprint) = today.get(&key) else {
                continue;
            };
            compared += 1;
            if line.text("base_fingerprint") != *today_fingerprint {
                differing.push(key);
            }
        }
        emit_result(&format!(
            "name=r4_base_identity arm={arm_name} prefixes={compared} differing={} verdict={}",
            differing.len(),
            if differing.is_empty() {
                "pass"
            } else {
                "differs"
            }
        ));
        for key in &differing {
            emit_result(&format!(
                "name=r4_compare_finding arm={arm_name} kind=base_identity detail={key:?}"
            ));
        }
    }
}

/// 丢写格与 Q6 按臂 × 族汇总（臂自己的账；出局由主 agent 按岔路单原文判）。
fn compare_losses(lines_by_arm: &BTreeMap<&'static str, Vec<ProductLine>>) {
    for (arm_name, lines) in lines_by_arm {
        for line in lines.iter().filter(|line| {
            line.text("name") == "r4_family_summary" || line.text("name") == "r4_random_summary"
        }) {
            emit_result(&format!(
                "name=r4_loss arm={arm_name} family={} geometry={} cells={} lost_write_cells={} q1_overwrite_cells={} q2_rollback_cells={} q3_lost_cells={} q3c_lost_cells={} q3_last_confirmed_cells={}",
                line.text("family"),
                line.text("geometry"),
                line.text("cells"),
                line.text("lost_write_cells_q1_q2_q3"),
                line.text("q1_overwrite_cells"),
                line.text("q2_rollback_cells"),
                line.text("q3_lost_cells"),
                line.text("q3c_lost_cells"),
                line.text("q3_last_confirmed_cells")
            ));
        }
        let cell_22_verdicts: Vec<&ProductLine> = lines
            .iter()
            .filter(|line| line.text("name") == "r4_q6_cell")
            .collect();
        if !cell_22_verdicts.is_empty() {
            let mut by_geometry: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
            for line in cell_22_verdicts {
                let entry = by_geometry.entry(line.text("geometry")).or_insert((0, 0));
                entry.0 += 1;
                if line.text("q6_not_covered") == "true" {
                    entry.1 += 1;
                }
            }
            for (geometry, (cells, not_covered)) in by_geometry {
                emit_result(&format!(
                    "name=r4_q6 arm={arm_name} geometry={geometry} cells={cells} not_covered_cells={not_covered} verdict={}",
                    if not_covered == 0 { "covered" } else { "not_covered" }
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 12.9 开跑检查（第 4 次跑）：5.6 常量回比（V3）、7.2 锚点（V4）、臂与副本对得上（装置编进的臂代码、R、读缓存收法与环境变量说的臂一致）。
// ---------------------------------------------------------------------------

fn fourth_run_local_constants_checks() -> Vec<ConstantCheck> {
    let mut checks = third_run_local_constants_checks();
    checks.extend([
        ConstantCheck {
            name: "根环区域数（槽序点预言用的那一个）ROOT_RING_REGIONS",
            local_value: ROOT_RING_REGIONS_LOCAL,
            crates_value: singlefs_format::ROOT_RING_REGIONS,
        },
        ConstantCheck {
            name: "稀疏盘扇区宽（镜像指纹用的那一个）SECTOR_BYTES",
            local_value: SECTOR_BYTES_LOCAL,
            crates_value: singlefs_harness::memory_pool::SECTOR_BYTES,
        },
    ]);
    checks
}

/// 臂探针：mkfs → 可写挂载 → 首个文件 → 关闭 C → 再可写挂载（不注入），读臂的观测。乙、乙-窄读 每次可写挂载都走重读那一支、
/// 记下 R 与读缓存收法；别的臂两样都是缺省。交回 (探针那次挂载的 K 类, 观测到的 R, 收不收 journal 空槽)。
fn fourth_run_arm_probe() -> Result<(&'static str, Option<FourthRunArmExtras>), String> {
    let history =
        fourth_run_plain_prefix(&GEOMETRY_PRIMARY, 0, Closing::ProcessExit, "probe")?.history;
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let call = run_fourth_run_mount_writable(&history.pool, &parameters, &[], None, None);
    Ok((
        third_run_outcome_class(&call.call.outcome, call.call.write_calls),
        call.extras,
    ))
}

fn run_fourth_run_constants_and_anchors(arm_name: &str) -> bool {
    let mut all_passed = true;
    for check in fourth_run_local_constants_checks() {
        let passed = check.matches();
        all_passed &= passed;
        emit_result(&format!(
            "name=r4_local_constant_check arm={arm_name} item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            verdict_word(passed),
            check.local_value,
            check.crates_value
        ));
    }
    let (regions_match, local_regions, crates_regions) = region_devices_matches_crates();
    all_passed &= regions_match;
    emit_result(&format!(
        "name=r4_local_constant_check arm={arm_name} item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        verdict_word(regions_match)
    ));
    let arm = fourth_run_arm_named(arm_name);
    let arm_code_compiled_in = third_run_arm_hooks::arm_code_is_compiled_in();
    let probe = fourth_run_arm_probe();
    let arm_code_matches = match (arm, &probe) {
        (Some(arm), Ok((class, extras))) => {
            let extras_match = match extras {
                None => arm.is_today(),
                Some(extras) => {
                    !arm.is_today()
                        && u64::from(extras.reread_rounds_limit) == arm.reread_rounds
                        && extras.zeroed_journal_ring_slots_are_cached
                            == (arm.candidate == FourthRunCandidate::YiNarrow)
                }
            };
            *class == "K0" && arm.is_today() != arm_code_compiled_in && extras_match
        }
        (None, _) | (_, Err(_)) => false,
    };
    all_passed &= arm_code_matches;
    emit_result(&format!(
        "name=r4_arm_code_check arm={arm_name} verdict={} arm_known={} candidate={} witness={} registered_rounds={} arm_code_compiled_in={arm_code_compiled_in} probe={}",
        verdict_word(arm_code_matches),
        arm.is_some(),
        arm.map_or("none", FourthRunArm::candidate_name),
        arm.map_or("none", FourthRunArm::witness_name),
        arm.map_or(0, |arm| arm.reread_rounds),
        match &probe {
            Ok((class, extras)) => format!("{class}:{extras:?}").replace(' ', ""),
            Err(error) => format!("error:{}", error.replace(' ', "_")),
        }
    ));
    match decoded_tail_instance_and_floor_offsets() {
        Ok((tail, instance, floor)) => {
            let passed = tail == SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL
                && instance == tail + 8
                && floor == SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8 + 4
                && instance == 477
                && floor == 481;
            all_passed &= passed;
            emit_result(&format!(
                "name=r4_section_seven_two_anchor arm={arm_name} item=\"journal 实例代号偏移 469 + 8；F 偏移 489 − 8\" verdict={} tail_offset={tail} instance_offset={instance} floor_offset={floor} registered_instance=477 registered_floor=481",
                verdict_word(passed)
            ));
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r4_section_seven_two_anchor arm={arm_name} item=offsets verdict=fail reason={error:?}"
            ));
        }
    }
    for geometry in [
        &GEOMETRY_PRIMARY,
        &GEOMETRY_SMALLER_ROOT_RING,
        &GEOMETRY_LARGER_ROOT_RING,
    ] {
        let parameters = parameters_for(geometry);
        let mut devices = new_devices(IMAGE_BYTES);
        let anchor = make_filesystem(&parameters, &mut devices)
            .map_err(|error| format!("{error:?}"))
            .and_then(|_| independent_geometry(&memory_pool_of(&devices, IMAGE_BYTES)));
        match anchor {
            Ok(pool_geometry) => {
                let pool = memory_pool_of(&devices, IMAGE_BYTES);
                let root_slots = checker_image::root_slot_positions(&pool_geometry).len() as u64;
                let ring_slots = pool_geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
                let configuration_slots = device_decode(&pool, &pool_geometry)
                    .system_configuration_slots
                    .len() as u64;
                for (item, computed, registered) in [
                    (
                        "根槽总数 3×S（装置在这个几何上枚举的槽数）",
                        root_slots,
                        ROOT_RING_REGIONS_LOCAL * geometry.slots_per_region,
                    ),
                    (
                        "环槽数 环÷4096（装置在这个几何上的环槽数）",
                        ring_slots,
                        geometry.journal_ring_bytes / 4096,
                    ),
                    (
                        "每个池的系统配置槽数 2×2（装置读到的自证槽数）",
                        configuration_slots,
                        2 * 2,
                    ),
                ] {
                    let passed = computed == registered;
                    all_passed &= passed;
                    emit_result(&format!(
                        "name=r4_section_seven_two_anchor arm={arm_name} geometry={} item={item:?} verdict={} computed={computed} registered={registered}",
                        geometry.label,
                        verdict_word(passed)
                    ));
                }
            }
            Err(error) => {
                all_passed = false;
                emit_result(&format!(
                    "name=r4_section_seven_two_anchor arm={arm_name} geometry={} item=geometry verdict=fail reason={error:?}",
                    geometry.label
                ));
            }
        }
    }
    let registered_pieces = [
        (
            "G0 上 3×S（PC-多读 (ii) 里 -槽 臂的点值）",
            ROOT_RING_REGIONS_LOCAL * GEOMETRY_PRIMARY.slots_per_region,
            24u64,
        ),
        (
            "H1f 每臂前缀数 n1a×c×n1b",
            (FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES.len()
                * 2
                * FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES.len()) as u64,
            8,
        ),
        (
            "H1g 每臂格数 n1×a×m×b×造法",
            (FOURTH_RUN_ACQUISITION_CRASH_PREFIX_OVERWRITES.len()
                * FOURTH_RUN_ACQUISITION_CRASHES.len()
                * FOURTH_RUN_ACQUISITION_CRASH_TAIL_OVERWRITES.len()
                * 2
                * 2) as u64,
            32,
        ),
    ];
    for (item, computed, registered) in registered_pieces {
        let passed = computed == registered;
        all_passed &= passed;
        emit_result(&format!(
            "name=r4_section_seven_two_anchor arm={arm_name} item={item:?} verdict={} computed={computed} registered={registered}",
            verdict_word(passed)
        ));
    }
    all_passed
}

fn fourth_run_arm_label() -> String {
    env::var("SINGLEFS_E158_ARM").unwrap_or_else(|_| "today".to_string())
}

/// 第一段（登记 5.7）：阳性对照里不靠 L 族的那几条（PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-只读，PC-多读 在 compare）、
/// PC-N0 与 PC-多读 (ii)（L0 那 8 格），H1d、H1e（G0），Q0 去向栏。
fn run_fourth_run_segment_one(arm: FourthRunArm) {
    run_fourth_run_positive_control_newer(
        &GEOMETRY_PRIMARY,
        arm,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
    );
    run_fourth_run_positive_control_newer(&GEOMETRY_PRIMARY, arm, ReplayBreak::EveryRecord);
    run_fourth_run_positive_control_second_read(&GEOMETRY_PRIMARY, arm);
    run_fourth_run_positive_control_no_newer(&GEOMETRY_PRIMARY, arm);
    run_fourth_run_held_once_like_family(
        "h1d",
        &GEOMETRY_PRIMARY,
        arm,
        1,
        FourthRunContinuation::EveryCut,
    );
    run_fourth_run_held_once_like_family(
        "h1e",
        &GEOMETRY_PRIMARY,
        arm,
        1,
        FourthRunContinuation::Tail,
    );
}

/// H1d / H1e 在一个几何点上：前缀 P(n1, C)，n1 ∈ {0, 1, 2, 3}；H1e 的 m ∈ {1, 2, 3, 4}。
fn run_fourth_run_held_once_like_family(
    name: &str,
    geometry: &Geometry,
    arm: FourthRunArm,
    hidden_count: usize,
    continuation: FourthRunContinuation,
) {
    let clause_checks = if hidden_count == 1
        && geometry.label == GEOMETRY_PRIMARY.label
        && matches!(name, "h1d" | "h1e")
    {
        ClauseChecksOnTheHiddenRoot::Checked
    } else {
        ClauseChecksOnTheHiddenRoot::NotRegisteredForThisFamily
    };
    let family = FourthRunFamily {
        name,
        geometry,
        hidden_count,
        continuation,
        tail_overwrites: &THIRD_RUN_TAIL_OVERWRITES,
        durations: FourthRunDurationPlan::OfTheArm,
        checks_the_clauses_on_the_hidden_root: clause_checks,
    };
    let prefixes = THIRD_RUN_OVERWRITES
        .iter()
        .map(|overwrites| {
            fourth_run_plain_prefix(geometry, *overwrites, Closing::ProcessExit, arm.name)
        })
        .collect();
    run_fourth_run_family(&family, arm, prefixes);
}

/// 第 4 次跑的模式：开跑检查不过就停（V3 / V4），过了跑点名的那一段。交回 true = 这个模式归第 4 次跑。
fn run_fourth_run_mode(mode: &str, command_line_arguments: &[String]) -> bool {
    let known = [
        "r4-constants",
        "r4-seg1",
        "r4-pc",
        "r4-h1d",
        "r4-h1e",
        "r4-compare",
        "r4-bases",
        "r4-seg3",
        "r4-seg4",
        "r4-seg5",
        "r4-h1f",
        "r4-h1g",
        "r4-seg2",
    ];
    if !known.contains(&mode) {
        return false;
    }
    emit_input_header();
    if mode == "r4-compare" {
        let prefixes: Vec<String> = command_line_arguments.iter().skip(2).cloned().collect();
        run_fourth_run_compare(&prefixes);
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    let arm_name = fourth_run_arm_label();
    if !run_fourth_run_constants_and_anchors(&arm_name) {
        emit_result(&format!(
            "name=stop arm={arm_name} reason=V3 detail=\"第 4 次跑的常量回比、第七节锚点或臂与副本的对应有不过的，整轮不开跑\""
        ));
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    let arm = fourth_run_arm_named(&arm_name).expect("开跑检查里核过臂名");
    match mode {
        "r4-constants" => {}
        "r4-seg1" => run_fourth_run_segment_one(arm),
        "r4-pc" => {
            run_fourth_run_positive_control_newer(
                &GEOMETRY_PRIMARY,
                arm,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            );
            run_fourth_run_positive_control_newer(&GEOMETRY_PRIMARY, arm, ReplayBreak::EveryRecord);
            run_fourth_run_positive_control_second_read(&GEOMETRY_PRIMARY, arm);
            run_fourth_run_positive_control_no_newer(&GEOMETRY_PRIMARY, arm);
        }
        "r4-h1d" => run_fourth_run_held_once_like_family(
            "h1d",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::EveryCut,
        ),
        "r4-h1e" => run_fourth_run_held_once_like_family(
            "h1e",
            &GEOMETRY_PRIMARY,
            arm,
            1,
            FourthRunContinuation::Tail,
        ),
        "r4-bases" => run_fourth_run_today_bases(arm),
        "r4-seg3" => run_fourth_run_segment_three(arm),
        "r4-seg4" => run_fourth_run_segment_four(arm),
        "r4-seg5" => run_fourth_run_segment_five(arm),
        "r4-h1f" => run_fourth_run_remount_families(arm),
        "r4-h1g" => run_fourth_run_acquisition_crash_family(arm),
        "r4-seg2" => fourth_run_segment_two::run_segment_two(arm),
        other => unreachable!("第 4 次跑的模式表 known 已经挡过：{other}"),
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    true
}

#[cfg(test)]
mod fourth_run_tests {
    use super::*;

    fn read_failure_at(offset: u64) -> ThirdRunFault {
        ThirdRunFault {
            device: DeviceIdentity(0),
            offset,
            length: 4096,
            form: UnreadableForm::ReadFails,
            duration: FaultDuration::WholeCall,
            label: format!("probe({offset})"),
        }
    }

    fn product_line(text: &str) -> ProductLine {
        ProductLine {
            fields: fields_of_a_result_line(text),
        }
    }

    /// 登记 5.6 与 7.2：第 4 次跑的常量回比、偏移解码、三个几何上的根槽 / 环槽 / 系统配置槽数、H1f / H1g 的格数、臂探针全过（V3、V4）。
    #[test]
    fn fourth_run_constants_and_anchors_all_pass() {
        assert!(run_fourth_run_constants_and_anchors("today"));
    }

    /// 臂名单：十八条，十二条主族；每条臂的候选、N 与 R 照登记 5.2 与主 agent 认定第 1 项。
    #[test]
    fn the_arm_table_follows_the_registration() {
        assert_eq!(FOURTH_RUN_ARMS.len(), 18);
        let narrow = fourth_run_arm_named("yi-narrow-cfg-carry").expect("乙-窄读-配置续");
        assert_eq!(narrow.candidate, FourthRunCandidate::YiNarrow);
        assert_eq!(narrow.witness, FourthRunWitness::SystemConfigurationCarried);
        assert_eq!(narrow.reread_rounds, 1);
        assert_eq!(
            fourth_run_arm_named("yi-slot-three-rounds")
                .expect("R = 3")
                .reread_rounds,
            3
        );
        assert_eq!(
            fourth_run_arm_named("bing-cfg")
                .expect("丙")
                .refusal_member(),
            "R3RebuildFromRecordsStoppedBelowTheWitness"
        );
        assert_eq!(
            fourth_run_arm_named("yi-narrow-slot")
                .expect("乙-窄读-槽")
                .refusal_member(),
            "R3UnreadableRootRingSlot"
        );
        assert_eq!(
            fourth_run_arm_named("today")
                .expect("今天")
                .refusal_member(),
            "none"
        );
    }

    /// 时长表（登记 5.3、8.2）：乙 W、O(1 只)、T(δ)、T(2δ)；R = 3 的乙 到 T(4δ)；今天 T(δ) 只跑不断那一格；H1e 的乙 W、T(δ)。
    #[test]
    fn the_durations_follow_the_registration() {
        let yi = fourth_run_arm_named("yi-cfg").expect("乙");
        let names: Vec<String> = fourth_run_held_once_durations(yi)
            .iter()
            .map(|(duration, _)| duration.name())
            .collect();
        assert_eq!(names, vec!["W", "O1only", "T1", "T2"]);
        let three = fourth_run_arm_named("yi-narrow-cfg-three-rounds").expect("R = 3");
        assert_eq!(fourth_run_held_once_durations(three).len(), 2 + 4);
        assert_eq!(fourth_run_tail_durations(three).len(), 1 + 4);
        let today = fourth_run_arm_named("today").expect("今天");
        assert_eq!(
            fourth_run_held_once_durations(today)[2],
            (FaultDuration::UntilVirtualTime(1), true)
        );
        assert_eq!(
            fourth_run_tail_durations(yi),
            vec![FaultDuration::WholeCall, FaultDuration::UntilVirtualTime(1)]
        );
    }

    /// D22（单元原子性怎么合成） 已定项 2「槽序」：(txg mod 3, (txg div 3) mod S)（变异 M21：h = 4 时 (h div 3) mod 8 = 1 ≠ h mod 8 = 4）。
    #[test]
    fn the_predicted_root_ring_slot_follows_the_slot_order() {
        assert_eq!(predicted_root_ring_slot_of_a_publish(4, 8), (1, 1));
        assert_eq!(predicted_root_ring_slot_of_a_publish(13, 8), (1, 4));
        assert_eq!(predicted_root_ring_slot_of_a_publish(30, 4), (0, 2));
    }

    /// A 藏一条根时与第 3 次跑那一份落点逐项相同（k 的推广不改 k = 1）。
    #[test]
    fn fourth_run_abandonment_faults_equal_the_third_run_ones_for_one_hidden_root() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 2).expect("P(2, ·)");
        third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
        let geometry = independent_geometry(&history.pool).expect("几何");
        for (form, replay_break) in [
            (UnreadableForm::ReadFails, ReplayBreak::EveryRecord),
            (
                UnreadableForm::ReadsZeros,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ),
        ] {
            let third = third_run_abandonment_faults(
                &history,
                &geometry,
                form,
                replay_break,
                FaultDuration::WholeCall,
            )
            .expect("第 3 次跑");
            let fourth = fourth_run_abandonment_faults(
                &history,
                &geometry,
                form,
                replay_break,
                FaultDuration::WholeCall,
                1,
            )
            .expect("第 4 次跑");
            assert_eq!(third, fourth);
        }
        let two = fourth_run_abandonment_faults(
            &history,
            &geometry,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
            FaultDuration::WholeCall,
            2,
        )
        .expect("k = 2")
        .expect("有记录");
        assert_eq!(
            two.iter()
                .filter(|fault| fault.label.starts_with("root"))
                .count(),
            2
        );
        let one = fourth_run_abandonment_faults(
            &history,
            &geometry,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
            FaultDuration::WholeCall,
            1,
        )
        .expect("k = 1")
        .expect("有记录");
        let record_labels = |faults: &[ThirdRunFault]| -> BTreeSet<String> {
            faults
                .iter()
                .filter(|fault| fault.label.starts_with("record"))
                .map(|fault| fault.label.clone())
                .collect()
        };
        assert!(record_labels(&two).is_disjoint(&record_labels(&one)));
    }

    /// 标记前后的读（登记 5.0 第三条、5.4 PC-多读 (iii)；变异 M14 第 4 次跑那一份）：三次读坏、两次读成都数，
    /// 第一个标记记下那一刻的读数与已拦下过的落点个数，之后读回全零的另数。
    #[test]
    fn reads_before_and_after_the_first_wait_count_failed_reads_too() {
        let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
        let state = Rc::new(RefCell::new(ThirdRunDeviceState::new(
            vec![read_failure_at(0)],
            None,
            None,
        )));
        let marker = Rc::new(RefCell::new(FourthRunReadMarker::default()));
        let (identity, inner) = devices_from_pool(&pool, IMAGE_BYTES).remove(0);
        let device = FourthRunDevice {
            device: ThirdRunDevice {
                identity,
                inner,
                state: Rc::clone(&state),
            },
            marker: Rc::clone(&marker),
        };
        let mut buffer = vec![0u8; 4096];
        for _ in 0..3 {
            assert!(device.read_at(DeviceOffsetInBytes(0), &mut buffer).is_err());
        }
        mark_the_first_wait(&marker, &state);
        for _ in 0..2 {
            assert!(device
                .read_at(DeviceOffsetInBytes(1 << 20), &mut buffer)
                .is_ok());
        }
        mark_the_first_wait(&marker, &state);
        let marker = marker.borrow();
        assert_eq!(marker.read_calls, 5);
        assert_eq!(marker.reads_before_the_first_wait, Some(3));
        assert_eq!(marker.faults_intercepted_before_the_first_wait, Some(1));
        assert_eq!(marker.all_zero_reads_after_the_first_wait, 2);
        assert_eq!(state.borrow().read_calls, 5);
    }

    /// Q0 去向（「读法写死」表；变异 M16）：s_h 上换成别的键的根、环里没有内容号 ≥ 被藏那一版的 ⇒ 根槽被盖；原样的镜像、没交回 Ok ⇒ 原样留着。
    #[test]
    fn the_hidden_root_destination_tells_an_overwritten_slot_from_an_untouched_one() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
        third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
        let geometry = independent_geometry(&history.pool).expect("几何");
        let hidden = history.tip();
        let before_hidden = *history.timeline.iter().rev().nth(1).expect("前一条");
        let position = hidden_root_position(&history.pool, hidden).expect("被藏那条根在环里");
        let untouched = hidden_root_destination(
            &position,
            &history.content_number_by_root,
            history.last_confirmed_content_number(),
            &history.pool,
            None,
        );
        assert_eq!(untouched, HiddenRootDestination::LeftAsItWas);
        let (hidden_device, hidden_offset) =
            root_slot_of(&history.pool, &geometry, hidden).expect("s_h");
        let (before_device, before_offset) =
            root_slot_of(&history.pool, &geometry, before_hidden).expect("前一条的槽");
        let length = usize::try_from(geometry.physical_block_size).expect("根槽宽");
        let copied = history
            .pool
            .read(before_device.0, before_offset, length)
            .expect("读得出");
        let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
        devices[usize::try_from(hidden_device.0).expect("盘号")]
            .1
            .write_at(
                DeviceOffsetInBytes(hidden_offset),
                &copied,
                WriteDurability::Plain,
            )
            .expect("内存盘写不报错");
        let overwritten_pool = memory_pool_of(&devices, IMAGE_BYTES);
        let overwritten = hidden_root_destination(
            &position,
            &history.content_number_by_root,
            history.last_confirmed_content_number(),
            &overwritten_pool,
            None,
        );
        assert_eq!(overwritten, HiddenRootDestination::RootSlotOverwritten);
        let chosen = hidden_root_destination(
            &position,
            &history.content_number_by_root,
            history.last_confirmed_content_number(),
            &history.pool,
            Some(hidden),
        );
        assert_eq!(chosen, HiddenRootDestination::ChosenOrApplied);
    }

    /// Q5-同成（登记第六节；变异 M18）：手写三格（两格都 Ok、一格臂拒），同成栏只收两格。
    #[test]
    fn both_mounted_extra_reads_leave_out_the_refused_cells() {
        let today = product_line("E7RESULT name=r4_cell a_class=K0 a_reads=100 a_read_bytes=409600 base_fingerprint=00ab");
        let arms = [
            product_line("E7RESULT name=r4_cell a_class=K0 a_reads=104 a_read_bytes=425984 base_fingerprint=00ab"),
            product_line("E7RESULT name=r4_cell a_class=K1 a_reads=108 a_read_bytes=442368 base_fingerprint=00ab"),
            product_line("E7RESULT name=r4_cell a_class=K3 a_reads=60 a_read_bytes=245760 base_fingerprint=00ab"),
        ];
        let counted: Vec<(i64, i64)> = arms
            .iter()
            .filter_map(|arm| both_mounted_extra_reads(arm, &today))
            .collect();
        assert_eq!(counted, vec![(4, 16384), (8, 32768)]);
        let different_start = product_line("E7RESULT name=r4_cell a_class=K0 a_reads=104 a_read_bytes=425984 base_fingerprint=00ac");
        assert_eq!(both_mounted_extra_reads(&different_start, &today), None);
    }

    fn record(
        arm: &str,
        control: &str,
        name: &str,
        tag: SentenceTag,
        verdict: SentenceVerdict,
    ) -> SentenceRecord {
        SentenceRecord {
            arm: arm.to_string(),
            control: control.to_string(),
            name: name.to_string(),
            tag,
            verdict,
        }
    }

    /// V1（登记 11.1；变异 M19）：今天那一臂一句 [今] 打不中、其余全过 ⇒ 作废集合为空；[测] 在今天那一臂上打不中 ⇒ 今天那一臂那条对照管的量作废，
    /// 别的臂只作废拿今天当参照的量；[臂] 打不中 ⇒ 那一臂那条对照管的量；同一句 [测] 每一臂都打不中 ⇒ 装置缺陷（全部臂一起）。
    #[test]
    fn the_void_scope_routes_by_the_sentence_tag() {
        let arms = ["today", "jia-cfg", "yi-cfg"];
        let passes = |control: &str, name: &str, tag: SentenceTag| {
            arms.iter()
                .map(|arm| record(arm, control, name, tag, SentenceVerdict::Pass))
                .collect::<Vec<_>>()
        };
        let mut sentences = passes("PC-N-ring", "truth_newer", SentenceTag::Measured);
        sentences.push(record(
            "today",
            "PC-N-ring",
            "hidden_in_ring_and_abandoned",
            SentenceTag::TodayDescription,
            SentenceVerdict::Fail,
        ));
        let (voided_by_a_today_sentence, defects_of_a_today_sentence) =
            fourth_run_void_scope(&sentences, &arms);
        assert!(voided_by_a_today_sentence.values().all(Vec::is_empty));
        assert!(defects_of_a_today_sentence.is_empty());

        let measured_and_arm_sentences = vec![
            record(
                "today",
                "PC-N0",
                "truth_not_newer",
                SentenceTag::Measured,
                SentenceVerdict::Fail,
            ),
            record(
                "jia-cfg",
                "PC-N0",
                "truth_not_newer",
                SentenceTag::Measured,
                SentenceVerdict::Pass,
            ),
            record(
                "yi-cfg",
                "PC-N-ring",
                "refused_after_r_rounds_k3",
                SentenceTag::ArmDefinition,
                SentenceVerdict::Fail,
            ),
        ];
        let (voided_by_measured_and_arm, defects_of_measured_and_arm) =
            fourth_run_void_scope(&measured_and_arm_sentences, &arms);
        let is_voided = |arm: &str, quantity: &'static str| {
            !voided_by_measured_and_arm[&(arm.to_string(), quantity)].is_empty()
        };
        assert!(is_voided("today", "Q4"));
        assert!(is_voided("jia-cfg", "Q4"));
        assert!(!is_voided("jia-cfg", "Q1-Q3"));
        assert!(is_voided("yi-cfg", "Q1-Q3"));
        assert!(!is_voided("today", "Q1-Q3"));
        assert!(defects_of_measured_and_arm.is_empty());

        let everyone_misses: Vec<SentenceRecord> = arms
            .iter()
            .map(|arm| {
                record(
                    arm,
                    "PC-lost-write",
                    "overwrite_counted_and_not_last_confirmed",
                    SentenceTag::Measured,
                    SentenceVerdict::Fail,
                )
            })
            .collect();
        let (voided_on_every_arm, defects_on_every_arm) =
            fourth_run_void_scope(&everyone_misses, &arms);
        assert_eq!(
            defects_on_every_arm,
            vec!["PC-lost-write:overwrite_counted_and_not_last_confirmed".to_string()]
        );
        assert!(!voided_on_every_arm[&("jia-cfg".to_string(), "Q1-Q3")].is_empty());
    }

    /// PC-多读 (iii) 与 Q5-拒 的参照是同一种 N 的甲臂，不是今天那一臂（登记 5.8 第二件；变异 M20）。
    #[test]
    fn the_extra_reads_reference_is_the_jia_arm_with_the_same_witness() {
        let reference =
            |name: &str| extra_reads_reference_arm(fourth_run_arm_named(name).expect("臂"));
        assert_eq!(reference("yi-cfg"), Some("jia-cfg"));
        assert_eq!(reference("yi-narrow-cfg-carry"), Some("jia-cfg-carry"));
        assert_eq!(reference("yi-slot-three-rounds"), Some("jia-slot"));
        assert_eq!(reference("bing-cfg"), Some("jia-cfg"));
        assert_eq!(reference("today"), None);
    }

    /// 今天那一臂造的起始镜像经文件原样读回（镜像指纹、补上的根、每一片的读数）。
    #[test]
    fn today_base_round_trips_through_its_file_encoding() {
        let history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
        let base = TodayBase {
            image: history.pool.clone(),
            appended_roots: vec![((2, 7), 2), ((2, 8), 2)],
            page_reads: vec![3, 1],
        };
        let decoded = decode_today_base(&encode_today_base(&base)).expect("读回");
        assert_eq!(
            image_fingerprint(&decoded.image),
            image_fingerprint(&base.image)
        );
        assert_eq!(decoded.image, base.image);
        assert_eq!(decoded.appended_roots, base.appended_roots);
        assert_eq!(decoded.page_reads, base.page_reads);
        assert!(decode_today_base(b"E158R4B0").is_err());
    }

    /// 条件式句子前件不成立记 vacuous，不算打不中；总判定按 fail > not_constructible > pass。
    #[test]
    fn conditional_sentences_are_vacuous_when_the_antecedent_does_not_hold() {
        assert_eq!(
            SentenceVerdict::conditional(false, false),
            SentenceVerdict::Vacuous
        );
        assert_eq!(
            SentenceVerdict::conditional(true, false),
            SentenceVerdict::Fail
        );
        let sentences = vec![
            sentence(
                SentenceTag::Measured,
                "first",
                SentenceVerdict::Vacuous,
                "V1",
            ),
            sentence(
                SentenceTag::ArmDefinition,
                "second",
                SentenceVerdict::NotConstructible,
                "F9",
            ),
        ];
        assert_eq!(overall_verdict(&sentences), "not_constructible");
    }
}

// ============================================================================
// 十三、第 4 次跑第二段（`research/prompts/e158-r4-prereg.md` 5.3 实七-甲、实七-乙、实八、H-随，5.5 主 agent 认定第 9 项 H-随全，
//      5.4 PC-随，5.6 H-随 的常量回比，11.2 S5a、S5b、S7、S9，第九节 M22）：模式 `r4-seg2`。
//      harness 按种子生成的历史在每一臂的副本上逐步执行。执行器是装置自己的：八种操作照 harness 执行器（`history.rs` 的 `apply_*`）
//      同一串入口、同一套前提各写一遍，盘照 harness 那一叠（故障注入包录制器包内存盘）；checker 每一步的判定原样记下，判红不截断。
//      harness 的注入点与崩溃状态按「同一步、步内同一序号」映到各臂自己的调用与写序列上（第十二节第二段修订）。
//      今天那一臂上每一步与 harness 自己的执行器比镜像与调用数：对得上，映射才站得住（对不上走 S9 / S7）。
// ============================================================================

mod fourth_run_segment_two {
    use super::*;

    use singlefs_checker::image::InvariantVerdict;
    use singlefs_checker::walk::check_pool_image;
    use singlefs_checker_tier::crash_injection::{inject_crashes_into_history, CrashPointDraw};
    use singlefs_core::admission::SpaceAdmission;
    use singlefs_core::journal::back_chain_of;
    use singlefs_core::make_filesystem::allocator_after_make_filesystem;
    use singlefs_core::mount::{
        mount_writable_with_space_admission, ParametersAndDeviceTableOfTheMount,
    };
    use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
    use singlefs_core::recovery::RecoveryReport;
    use singlefs_core::transaction::{publish_without_units, ZeroUnitPublishPlan};
    use singlefs_harness::fault_injection::{inject_faults_into_history, FaultCallCounts};
    use singlefs_harness::history::{
        execute_history, execute_history_with_faults, generate_history_with_weights,
        with_panic_capture, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
        GenerationWeights, HistoryDevice, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
        HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
        StepPosition,
    };
    use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, RetainedWrite};
    use singlefs_harness::scenario::{
        first_file_content, FIXED_WRITE_TIME_SECONDS as WRITE_TIME_SECONDS_OF_THE_HARNESS_HISTORIES,
    };
    use singlefs_harness::{RecordingBlockDevice, SharedStream};

    /// 产物里这一段各族的几何名：几何是被复现的那段历史的一部分（登记 5.3：快档的 4 GiB 盘与 harness 的几何），S6 对这几族不适用。
    const HARNESS_GEOMETRY_NAME: &str = "harness-4gib";

    /// 随机历史快档的段数与每段步数（登记 5.6：抄自 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`
    /// 第 43、44 行 `FAST_TIER_SEEDS`、`FAST_TIER_OPERATIONS_PER_HISTORY`；私有常量引不到，开跑时按那份源码的文本回比，现查的值记在第十二节）。
    const FAST_TIER_SEEDS_LOCAL: u64 = 96;
    const FAST_TIER_OPERATIONS_PER_HISTORY_LOCAL: usize = 30;

    /// H-随 (a) 的 28 个种子偏移（登记 5.6、7.2：调查员报告 `research/prompts/m2-investigate-gate74-reds-report.md` 第 61 行那一串各减种子基）。
    const RANDOM_HISTORY_RED_SEED_OFFSETS: [u64; 28] = [
        2, 6, 9, 11, 15, 16, 18, 21, 23, 24, 29, 31, 35, 52, 54, 55, 58, 65, 70, 71, 76, 79, 81,
        84, 85, 86, 90, 92,
    ];
    /// 7.2：H-随 (a) 最小、最大的种子（登记第 7.2 节那一行的两个数）。
    const RANDOM_HISTORY_RED_SEED_MINIMUM_REGISTERED: u64 = 7_463_871_032_432_355_115;
    const RANDOM_HISTORY_RED_SEED_MAXIMUM_REGISTERED: u64 = 7_463_871_032_432_355_205;
    /// 7.2：H-随 的段数 28 + 1 + 1（登记第 7.2 节那一行）。
    const RANDOM_HISTORY_SEGMENTS_REGISTERED: usize = 30;

    /// 实八：崩溃注入种子基 + 2、24 步、4 个崩溃状态（r3 登记第 373–391 行那张表「实八」一行；
    /// 步数与抽法同崩溃注入快档 `crates/singlefs-checker-tier/tests/second_transaction_supplement_three_crash_injection.rs` 第 39、41 行）。
    const EIGHTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET: u64 = 2;
    const EIGHTH_BATCH_CRASH_REPRODUCTION_OPERATIONS: usize = 24;
    const EIGHTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS: usize = 4;

    /// S5 的签名（r3 登记第 604 行）：实七-甲 重开走到 (1, 3) 并报 `MappingStillUnreadable { slot: 50240 }`；实七-乙 在崩溃状态上读回同一个错；
    /// 实八 (1, 4) 那次发布的单元在它还在根环里时被复用。
    const SEVENTH_BATCH_SIGNATURE_SLOT: u64 = 50240;
    const SEVENTH_BATCH_SIGNATURE_ROOT: TimelineRoot = (1, 3);
    const EIGHTH_BATCH_SIGNATURE_ROOT: TimelineRoot = (1, 4);

    /// PC-随 描述今天那一臂的三句（登记 5.4 PC-随 [今]，F19）：checker 第一次判 I-7.4 被抛弃根那一半红的那一步与点名的根。
    /// (a) 种子基 + 2 那一段：Operation(4)、(实例 1, txg 4)；(b) 最短复现：Operation(2)；(c) 第 43 行那条：Operation(3)、(实例 1, txg 5)。
    const POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SEED_PLUS_TWO: (usize, TimelineRoot) =
        (4, (1, 4));
    const POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SHORTEST: usize = 2;
    const POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_ROW_FORTY_THREE: (usize, TimelineRoot) = (3, (1, 5));

    /// harness 执行器撞到判红就停（它的缺省）。登记 5.3 要这一族把它关掉、照原操作序列走完；变异 M22 把它打开。
    const RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED: bool = false;

    /// checker 判 I-7.4 被抛弃根那一半时违例细节的开头（`singlefs_checker::walk` 的那一句）：后面跟「（实例 X、txg Y）」。
    const ABANDONED_ROOT_VIOLATION_PREFIX: &str = "被抛弃的根（实例 ";
    const ABANDONED_ROOT_INVARIANT: &str = "I-7.4";

    /// 这一段跑的历史从哪来。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SegmentTwoFamily {
        /// H-随：调查员点名的 28 段、最短复现、收口表第 43 行那条（登记 5.3）。
        RandomRed,
        /// H-随全：快档其余 68 段，只进 Q4、Q5（登记 5.5 主 agent 认定第 9 项）。
        RandomAll,
        /// 实七-甲：故障注入大档种子基 + 110 那段历史，harness 摆的 6 个注入点各一个分支。
        SeventhFault,
        /// 实七-乙：崩溃注入快档种子基 + 16 那段历史，harness 摆的 4 个崩溃状态各一个分支。
        SeventhCrash,
        /// 实八：崩溃注入种子基 + 2 那段历史，同实七-乙的做法。
        EighthCrash,
    }

    impl SegmentTwoFamily {
        fn name(self) -> &'static str {
            match self {
                SegmentTwoFamily::RandomRed => "h-random",
                SegmentTwoFamily::RandomAll => "h-random-all",
                SegmentTwoFamily::SeventhFault => "seventh-fault",
                SegmentTwoFamily::SeventhCrash => "seventh-crash",
                SegmentTwoFamily::EighthCrash => "eighth-crash",
            }
        }

        /// 这一族的丢写进不进出局（H-随全 只进 Q4、Q5，认定第 9 项）。
        fn measures_lost_writes(self) -> bool {
            match self {
                SegmentTwoFamily::RandomRed
                | SegmentTwoFamily::SeventhFault
                | SegmentTwoFamily::SeventhCrash
                | SegmentTwoFamily::EighthCrash => true,
                SegmentTwoFamily::RandomAll => false,
            }
        }

        /// 每一步之后跑不跑池级 checker（Q9 与 PC-随 要 H-随 的；实七、实八的主路径给 S5 与对照；H-随全 只要挂载）。
        fn checks_every_step(self) -> bool {
            match self {
                SegmentTwoFamily::RandomRed
                | SegmentTwoFamily::SeventhFault
                | SegmentTwoFamily::SeventhCrash
                | SegmentTwoFamily::EighthCrash => true,
                SegmentTwoFamily::RandomAll => false,
            }
        }

        /// 历史走完之后接不接尾巴（关闭 C → 可写挂载 → 冷启动读回 → ③c；登记 5.3 H-随，实七、实八照同一条，第十二节第二段修订）。
        fn runs_the_tail(self) -> bool {
            self.measures_lost_writes()
        }
    }

    /// 一段历史：族、在族里的名字（种子偏移或写死的那两条）、操作序列。
    #[derive(Clone, Debug)]
    struct SegmentTwoHistory {
        family: SegmentTwoFamily,
        label: String,
        history: GeneratedHistory,
    }

    fn fast_tier_history(offset: u64) -> GeneratedHistory {
        generate_history_with_weights(
            HistorySeed(SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + offset),
            FAST_TIER_OPERATIONS_PER_HISTORY_LOCAL,
            &GenerationWeights::BROAD,
        )
    }

    fn empty_content() -> ContentChoice {
        ContentChoice {
            length: ContentLength::Empty,
            fill_seed: 0,
        }
    }

    /// H-随 (b)：种子基 + 2 那一段收缩出来的 3 步（登记 5.3、5.6；调查员报告第 298–301 行）。
    fn shortest_reproduction_history() -> GeneratedHistory {
        GeneratedHistory {
            seed: HistorySeed(SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + 2),
            starting_point: HistoryStartingPoint::AfterMakeFilesystem,
            operations: vec![
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::PublishFirstFile(empty_content()),
                HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            ],
        }
    }

    /// H-随 (c)：收口表第 43 行那条用例的写死历史（登记 5.6；随机历史测试第 1082 行那条用例的 `operations`）。
    fn closeout_row_forty_three_history() -> GeneratedHistory {
        GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![
                HistoryOperation::PublishOverwrite(empty_content()),
                HistoryOperation::PublishOverwrite(empty_content()),
                HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
                HistoryOperation::PublishOverwrite(empty_content()),
                HistoryOperation::PublishOverwrite(empty_content()),
                HistoryOperation::PublishOverwrite(empty_content()),
                HistoryOperation::PublishOverwrite(empty_content()),
                HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                    steps_above_current_floor: 5,
                }),
            ],
        }
    }

    const SHORTEST_REPRODUCTION_LABEL: &str = "shortest";
    const ROW_FORTY_THREE_LABEL: &str = "row43";

    fn seed_label(offset: u64) -> String {
        format!("base+{offset}")
    }

    /// H-随 的 30 段：28 段种子、最短复现、第 43 行那条（次序照登记 5.3）。
    fn random_red_histories() -> Vec<SegmentTwoHistory> {
        let mut histories: Vec<SegmentTwoHistory> = RANDOM_HISTORY_RED_SEED_OFFSETS
            .iter()
            .map(|offset| SegmentTwoHistory {
                family: SegmentTwoFamily::RandomRed,
                label: seed_label(*offset),
                history: fast_tier_history(*offset),
            })
            .collect();
        histories.push(SegmentTwoHistory {
            family: SegmentTwoFamily::RandomRed,
            label: SHORTEST_REPRODUCTION_LABEL.to_string(),
            history: shortest_reproduction_history(),
        });
        histories.push(SegmentTwoHistory {
            family: SegmentTwoFamily::RandomRed,
            label: ROW_FORTY_THREE_LABEL.to_string(),
            history: closeout_row_forty_three_history(),
        });
        histories
    }

    /// H-随全 的 68 段：快档 96 段里 H-随 没取的那几段。
    fn random_all_histories() -> Vec<SegmentTwoHistory> {
        (0..FAST_TIER_SEEDS_LOCAL)
            .filter(|offset| !RANDOM_HISTORY_RED_SEED_OFFSETS.contains(offset))
            .map(|offset| SegmentTwoHistory {
                family: SegmentTwoFamily::RandomAll,
                label: seed_label(offset),
                history: fast_tier_history(offset),
            })
            .collect()
    }

    fn seventh_fault_history() -> SegmentTwoHistory {
        SegmentTwoHistory {
            family: SegmentTwoFamily::SeventhFault,
            label: seed_label(SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET),
            history: generate_history_with_weights(
                HistorySeed(
                    SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL
                        + SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET,
                ),
                SEVENTH_BATCH_FAULT_REPRODUCTION_OPERATIONS,
                &GenerationWeights::BROAD,
            ),
        }
    }

    fn seventh_crash_history() -> SegmentTwoHistory {
        SegmentTwoHistory {
            family: SegmentTwoFamily::SeventhCrash,
            label: seed_label(SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET),
            history: generate_history_with_weights(
                HistorySeed(
                    SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL
                        + SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET,
                ),
                SEVENTH_BATCH_CRASH_REPRODUCTION_OPERATIONS,
                &GenerationWeights::BROAD,
            ),
        }
    }

    fn eighth_crash_history() -> SegmentTwoHistory {
        SegmentTwoHistory {
            family: SegmentTwoFamily::EighthCrash,
            label: seed_label(EIGHTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET),
            history: generate_history_with_weights(
                HistorySeed(
                    SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL
                        + EIGHTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET,
                ),
                EIGHTH_BATCH_CRASH_REPRODUCTION_OPERATIONS,
                &GenerationWeights::BROAD,
            ),
        }
    }

    fn harness_execution(per_step_checker: PerStepChecker) -> HistoryExecution {
        HistoryExecution {
            per_step_checker,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        }
    }

    // ------------------------------------------------------------------------
    // 13.1 开跑检查（登记 5.6 的 H-随 常量回比、7.2 的 H-随 锚点、11.2 S9 的前两句）。
    //      调查员报告与两份测试源码按文本读：报告不在副本里，路径由环境变量给（缺省是仓根下的相对路径，`replay.sh` 在仓根跑）。
    // ------------------------------------------------------------------------

    const INVESTIGATOR_REPORT_DEFAULT_PATH: &str =
        "research/prompts/m2-investigate-gate74-reds-report.md";
    const INVESTIGATOR_REPORT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_E158_INVESTIGATOR_REPORT";
    /// 调查员报告里种子那一行与最短复现那几行（登记 5.6：第 61 行、第 298–301 行）。
    const INVESTIGATOR_REPORT_SEED_LINE: usize = 61;
    const INVESTIGATOR_REPORT_SHORTEST_FIRST_LINE: usize = 298;
    const RANDOM_HISTORY_TEST_SOURCE: &str =
        "crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs";
    const CRASH_INJECTION_TEST_SOURCE: &str =
        "crates/singlefs-checker-tier/tests/second_transaction_supplement_three_crash_injection.rs";

    fn investigator_report_text() -> Result<String, String> {
        let path = env::var(INVESTIGATOR_REPORT_ENVIRONMENT_VARIABLE)
            .unwrap_or_else(|_| INVESTIGATOR_REPORT_DEFAULT_PATH.to_string());
        std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))
    }

    /// 报告第 61 行里 `[` 与 `]` 之间的种子（逗号分隔）。
    fn seeds_listed_on_the_line(line: &str) -> Vec<u64> {
        let Some(start) = line.find("同签名的种子 [") else {
            return Vec::new();
        };
        let rest = &line[start + "同签名的种子 [".len()..];
        let Some(end) = rest.find(']') else {
            return Vec::new();
        };
        rest[..end]
            .split(',')
            .filter_map(|seed| seed.trim().parse().ok())
            .collect()
    }

    /// 一行开检查的结果：名字、判定、细节。判定 fail 的那一条让它管的那一族停（S9 / V3）。
    struct SegmentTwoCheck {
        item: String,
        passed: bool,
        detail: String,
    }

    fn segment_two_check(item: &str, passed: bool, detail: String) -> SegmentTwoCheck {
        SegmentTwoCheck {
            item: item.to_string(),
            passed,
            detail,
        }
    }

    /// S9 前两句与 5.6、7.2 的 H-随 那几行：种子、最短复现、第 43 行那条、快档段数与步数。交回每一项，失败的那几项让 H-随 停。
    fn random_history_source_checks(report: &Result<String, String>) -> Vec<SegmentTwoCheck> {
        let mut checks = Vec::new();
        let seeds: Vec<u64> = RANDOM_HISTORY_RED_SEED_OFFSETS
            .iter()
            .map(|offset| SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + offset)
            .collect();
        checks.push(segment_two_check(
            "7.2 H-随 (a) 种子个数、最小、最大",
            seeds.len() == RANDOM_HISTORY_RED_SEED_OFFSETS.len()
                && seeds.iter().min() == Some(&RANDOM_HISTORY_RED_SEED_MINIMUM_REGISTERED)
                && seeds.iter().max() == Some(&RANDOM_HISTORY_RED_SEED_MAXIMUM_REGISTERED),
            format!(
                "count={} minimum={:?} maximum={:?}",
                seeds.len(),
                seeds.iter().min(),
                seeds.iter().max()
            ),
        ));
        checks.push(segment_two_check(
            "7.2 H-随 段数 28 + 1 + 1",
            random_red_histories().len() == RANDOM_HISTORY_SEGMENTS_REGISTERED,
            format!("segments={}", random_red_histories().len()),
        ));
        match report {
            Ok(text) => {
                let lines: Vec<&str> = text.lines().collect();
                let seed_line = lines
                    .get(INVESTIGATOR_REPORT_SEED_LINE - 1)
                    .copied()
                    .unwrap_or("");
                let listed = seeds_listed_on_the_line(seed_line);
                checks.push(segment_two_check(
                    "S9 (a) 种子与调查员报告第 61 行逐个相等",
                    listed == seeds,
                    format!("listed={} local={}", listed.len(), seeds.len()),
                ));
                let shortest = shortest_reproduction_history();
                let first = INVESTIGATOR_REPORT_SHORTEST_FIRST_LINE - 1;
                let header_holds = lines
                    .get(first)
                    .is_some_and(|line| line.contains("最短复现：起点 AfterMakeFilesystem，3 步"))
                    && shortest.starting_point == HistoryStartingPoint::AfterMakeFilesystem
                    && shortest.operations.len() == 3;
                let steps_hold =
                    shortest
                        .operations
                        .iter()
                        .enumerate()
                        .all(|(index, operation)| {
                            lines.get(first + 1 + index).is_some_and(|line| {
                                line.trim_start()
                                    .starts_with(&format!("{index}. {operation:?} →"))
                            })
                        });
                checks.push(segment_two_check(
                    "S9 (b) 最短复现 3 步与调查员报告第 298–301 行逐项相等",
                    header_holds && steps_hold,
                    format!("header={header_holds} steps={steps_hold}"),
                ));
            }
            Err(error) => {
                checks.push(segment_two_check(
                    "S9 (a)(b) 调查员报告读得出",
                    false,
                    format!("{error:?}"),
                ));
            }
        }
        checks.push(row_forty_three_source_check());
        checks.push(fast_tier_constant_check());
        checks
    }

    /// 第 43 行那条用例（随机历史测试 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`）
    /// 的起点、空内容与 8 步，按源码文本逐项比（登记 5.6「逐项比」）。
    fn row_forty_three_source_check() -> SegmentTwoCheck {
        const CASE_NAME: &str = "fn crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43";
        let Ok(source) = std::fs::read_to_string(RANDOM_HISTORY_TEST_SOURCE) else {
            return segment_two_check(
                "S9 (c) 第 43 行那条用例读得出",
                false,
                RANDOM_HISTORY_TEST_SOURCE.to_string(),
            );
        };
        let Some(start) = source.find(CASE_NAME) else {
            return segment_two_check(
                "S9 (c) 第 43 行那条用例在源码里",
                false,
                CASE_NAME.to_string(),
            );
        };
        let body = &source[start..];
        let end = body.find("let run = execute_history").unwrap_or(body.len());
        let body = &body[..end];
        let history = closeout_row_forty_three_history();
        let mut expected: Vec<String> = vec![
            "length: ContentLength::Empty,".to_string(),
            "fill_seed: 0,".to_string(),
            "seed: HistorySeed(0),".to_string(),
            "starting_point: HistoryStartingPoint::AfterFirstFile,".to_string(),
        ];
        for operation in &history.operations {
            expected.push(match operation {
                HistoryOperation::PublishOverwrite(_) => {
                    "HistoryOperation::PublishOverwrite(empty),".to_string()
                }
                HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
                    "HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,".to_string()
                }
                HistoryOperation::RaiseRollbackFloor(choice) => format!(
                    "steps_above_current_floor: {},",
                    choice.steps_above_current_floor
                ),
                HistoryOperation::PublishFirstFile(_)
                | HistoryOperation::PublishWithoutUnits
                | HistoryOperation::CloseAndMountWritable
                | HistoryOperation::RollBackWhileMounted(_)
                | HistoryOperation::ColdStartRecover => format!("{operation:?}"),
            });
        }
        let source_lines: Vec<&str> = body
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        let mut cursor = 0usize;
        let mut matched = 0usize;
        for wanted in &expected {
            match source_lines[cursor..]
                .iter()
                .position(|line| line == wanted)
            {
                Some(found) => {
                    cursor += found + 1;
                    matched += 1;
                }
                None => break,
            }
        }
        let operation_lines = source_lines
            .iter()
            .filter(|line| line.starts_with("HistoryOperation::"))
            .count();
        segment_two_check(
            "S9 (c) 第 43 行那条用例 8 步与源码逐项相等",
            matched == expected.len() && operation_lines == history.operations.len(),
            format!(
                "matched={matched}/{} operation_lines={operation_lines}",
                expected.len()
            ),
        )
    }

    /// 5.6：快档首种子、段数、每段步数按随机历史测试源码第 42–44 行的文本回比；实八、实七-乙的步数与抽法按崩溃注入测试第 39、41 行回比。
    /// 不等的照本地常量跑、写进第十二节（登记 5.6 那一行），这一项只报不停。
    fn fast_tier_constant_check() -> SegmentTwoCheck {
        let random_source = std::fs::read_to_string(RANDOM_HISTORY_TEST_SOURCE).unwrap_or_default();
        let crash_source = std::fs::read_to_string(CRASH_INJECTION_TEST_SOURCE).unwrap_or_default();
        let wanted_random = [
            "const FAST_TIER_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;".to_string(),
            format!("const FAST_TIER_SEEDS: u64 = {FAST_TIER_SEEDS_LOCAL};"),
            format!(
                "const FAST_TIER_OPERATIONS_PER_HISTORY: usize = {FAST_TIER_OPERATIONS_PER_HISTORY_LOCAL};"
            ),
        ];
        let wanted_crash = [
            format!(
                "const FAST_TIER_OPERATIONS_PER_HISTORY: usize = {EIGHTH_BATCH_CRASH_REPRODUCTION_OPERATIONS};"
            ),
            format!(
                "crash_points_per_history: {EIGHTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS},"
            ),
        ];
        let random_holds = wanted_random
            .iter()
            .all(|line| random_source.lines().any(|source| source.trim() == line));
        let crash_holds = wanted_crash
            .iter()
            .all(|line| crash_source.lines().any(|source| source.trim() == line));
        segment_two_check(
            "5.6 快档首种子、段数、每段步数与崩溃注入快档步数、抽法（源码文本回比；不等只报，照本地常量跑）",
            random_holds && crash_holds,
            format!(
                "random_history_source={random_holds} crash_injection_source={crash_holds} seeds={FAST_TIER_SEEDS_LOCAL} operations={FAST_TIER_OPERATIONS_PER_HISTORY_LOCAL} crash_operations={EIGHTH_BATCH_CRASH_REPRODUCTION_OPERATIONS} crash_points={EIGHTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS}"
            ),
        )
    }

    // ------------------------------------------------------------------------
    // 13.2 盘与一次受观测的入口调用：harness 那一叠盘（`HistoryDevice`）外面包一层，数这次调用打到盘上的读（Q5）、写（K3 / K4），
    //      崩溃恢复抛弃根那一步把点名的几段读回全 0（harness `DeviceReadingZerosOverHiddenRanges` 同一造法：读照样交下去、交回之前清零；
    //      写到哪一段那一段就不再藏），并数被藏范围被读了几次（V2 对这几族不适用，只报数）。
    // ------------------------------------------------------------------------

    /// 盘上被藏起来的一段（读回全 0）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct HiddenRange {
        device: DeviceIdentity,
        offset: u64,
        length: u64,
    }

    impl HiddenRange {
        fn overlaps(self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
            self.device == device
                && self.offset < offset + length
                && offset < self.offset + self.length
        }

        /// 装置自己的第一遍读（`device_judgement`）用的那一种写法：读回全零、整次调用都坏。
        fn as_third_run_fault(self) -> ThirdRunFault {
            ThirdRunFault {
                device: self.device,
                offset: self.offset,
                length: self.length,
                form: UnreadableForm::ReadsZeros,
                duration: FaultDuration::WholeCall,
                label: format!("hidden(device={},offset={})", self.device.0, self.offset),
            }
        }
    }

    /// 一次入口调用这一层数到的东西。
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct CallWindow {
        read_calls: u64,
        read_bytes: u64,
        write_calls: u64,
        reads_over_hidden_ranges: u64,
        reads_before_the_first_wait: Option<u64>,
    }

    struct ObservedDevice<Inner: BlockDevice> {
        identity: DeviceIdentity,
        inner: Inner,
        hidden: Vec<HiddenRange>,
        window: Rc<RefCell<CallWindow>>,
    }

    impl<Inner: BlockDevice> BlockDevice for ObservedDevice<Inner> {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
            {
                let mut window = self.window.borrow_mut();
                window.read_calls += 1;
                window.read_bytes += length;
            }
            self.inner.read_at(offset, buffer)?;
            let mut read_a_hidden_range = false;
            for hidden in &self.hidden {
                if !hidden.overlaps(self.identity, offset.0, length) {
                    continue;
                }
                read_a_hidden_range = true;
                let start_in_buffer = hidden.offset.saturating_sub(offset.0);
                let end_in_buffer = (hidden.offset + hidden.length - offset.0).min(length);
                buffer[usize::try_from(start_in_buffer).expect("落在缓冲区里")
                    ..usize::try_from(end_in_buffer).expect("落在缓冲区里")]
                    .fill(0);
            }
            if read_a_hidden_range {
                self.window.borrow_mut().reads_over_hidden_ranges += 1;
            }
            Ok(())
        }

        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let length = u64::try_from(bytes.len()).expect("一次写的长度装得进 u64");
            self.window.borrow_mut().write_calls += 1;
            let identity = self.identity;
            self.hidden
                .retain(|hidden| !hidden.overlaps(identity, offset.0, length));
            self.inner.write_at(offset, bytes, durability)
        }

        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            self.window.borrow_mut().write_calls += 1;
            let identity = self.identity;
            self.hidden
                .retain(|hidden| !hidden.overlaps(identity, offset.0, length));
            self.inner.write_zeroes_at(offset, length)
        }

        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            self.inner.barrier()
        }

        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            self.inner.probe_physical_block_size()
        }

        fn size_in_bytes(&self) -> u64 {
            self.inner.size_in_bytes()
        }
    }

    /// 一次受观测的入口调用交回的读数（Q5、Q7；可写挂载另带臂的观测与等待标记）。
    #[derive(Clone, Debug, Default)]
    struct ObservedCallReading {
        window: CallWindow,
        waits: u64,
        observation: Option<ArmObservation>,
        extras: Option<FourthRunArmExtras>,
    }

    /// 在 `devices` 外面包一层受观测的盘跑 `call`，跑完原样拆回。`installs_the_arm_hooks` = 可写挂载：装等待钩子（记第一个等待标记）、
    /// 虚拟时钟归 0、跑完取臂的观测（臂的代码只在副本里有，`fourth_run_arm_hooks`）。
    fn observed_call<Inner: BlockDevice, Output>(
        devices: &mut Vec<(DeviceIdentity, Inner)>,
        hidden: &[HiddenRange],
        installs_the_arm_hooks: bool,
        call: impl FnOnce(&mut Vec<(DeviceIdentity, ObservedDevice<Inner>)>) -> Output,
    ) -> (Output, ObservedCallReading) {
        let window = Rc::new(RefCell::new(CallWindow::default()));
        let mut wrapped: Vec<(DeviceIdentity, ObservedDevice<Inner>)> = std::mem::take(devices)
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    ObservedDevice {
                        identity,
                        inner,
                        hidden: hidden
                            .iter()
                            .filter(|range| range.device == identity)
                            .copied()
                            .collect(),
                        window: Rc::clone(&window),
                    },
                )
            })
            .collect();
        if installs_the_arm_hooks {
            third_run_arm_hooks::reset_the_virtual_clock();
            let window_for_the_hook = Rc::clone(&window);
            fourth_run_arm_hooks::install_the_marking_wait_hook(
                THIRD_RUN_VIRTUAL_INTERVAL,
                Box::new(move || {
                    let mut marked = window_for_the_hook.borrow_mut();
                    if marked.reads_before_the_first_wait.is_none() {
                        marked.reads_before_the_first_wait = Some(marked.read_calls);
                    }
                }),
            );
        }
        let output = call(&mut wrapped);
        let taken = if installs_the_arm_hooks {
            fourth_run_arm_hooks::take_the_arm_observation()
        } else {
            None
        };
        let waits = if installs_the_arm_hooks {
            third_run_arm_hooks::waits()
        } else {
            0
        };
        *devices = wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect();
        let window_reading = *window.borrow();
        (
            output,
            ObservedCallReading {
                window: window_reading,
                waits,
                observation: taken.as_ref().map(|(observation, _)| observation.clone()),
                extras: taken.map(|(_, extras)| extras),
            },
        )
    }

    fn history_device_bytes() -> u64 {
        HistoryDeviceWidth::FourGibibytes.device_bytes()
    }

    /// harness 那一叠盘此刻的整份镜像（harness `image_of` 同一取法）。
    fn image_of_history_devices(devices: &[(DeviceIdentity, HistoryDevice)]) -> MemoryPool {
        MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| {
                    (
                        *identity,
                        device.wrapped_device().wrapped_device().image.clone(),
                    )
                })
                .collect(),
            device_size_in_bytes: history_device_bytes(),
        }
    }

    /// 一份镜像上的两块盘（崩溃状态的分支在它上面做可写挂载与读回）。
    fn plain_devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
        devices_from_pool(image, history_device_bytes())
    }

    /// 按 checker 的读法读根环：(txg, 实例) 从新到旧、去重（harness `ring_roots_newest_first_of` 的同一取法）。
    fn ring_roots_newest_first_of_the_image(image: &MemoryPool) -> Vec<(u64, u32)> {
        let Some(geometry) = checker_image::chosen_system_configurations(image)
            .into_iter()
            .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        else {
            return Vec::new();
        };
        let mut roots: Vec<(u64, u32)> = checker_image::valid_roots(image, &geometry)
            .into_iter()
            .map(|(_, _, view)| (view.checkpoint_txg, view.instance))
            .collect();
        roots.sort_unstable_by(|left, right| right.cmp(left));
        roots.dedup();
        roots
    }

    /// 按 checker 的读法：根环里最新那条根带的 F（harness `newest_ring_root_floor` 的同一取法）。
    fn newest_ring_root_floor_of_the_image(image: &MemoryPool) -> Option<u64> {
        let geometry = checker_image::chosen_system_configurations(image)
            .into_iter()
            .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
        checker_image::valid_roots(image, &geometry)
            .into_iter()
            .map(|(_, _, view)| view)
            .max_by_key(|view| (view.checkpoint_txg, view.instance))
            .map(|view| view.rollback_floor)
    }

    /// 崩溃恢复要藏起来的那几段（harness `ranges_hidden_by_the_crash_recovery` 的同一取法）：现行版本的根槽，与它那次发布每一条记录
    /// 点名的每一份单元的第一个槽。那次发布一个单元都没点名时交回 None。
    fn ranges_hidden_by_the_crash_recovery(
        parameters: &MakeFilesystemParameters,
        current: &PoolVersion,
    ) -> Option<Vec<HiddenRange>> {
        let earlier_records = match current {
            PoolVersion::WithoutFile(version) => &version.earlier_records_of_this_publish,
            PoolVersion::WithFile(version) => &version.earlier_records_of_this_publish,
        };
        let named_units: Vec<LocationEntry> = earlier_records
            .iter()
            .map(|written| &written.record)
            .chain(std::iter::once(current.record()))
            .flat_map(|record| record.named.iter())
            .flat_map(|named| named.locations)
            .collect();
        if named_units.is_empty() {
            return None;
        }
        let root_slot = singlefs_core::root_ring::target_for_publish(
            current.root().checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        let root_slot_device =
            parameters.region_devices[usize::try_from(root_slot.region).expect("区域号小于 3")];
        let root_slot_offset = singlefs_core::root_ring::slot_offset(
            root_slot,
            parameters.geometry.fixed_structure_slot_spacing,
        );
        Some(
            std::iter::once(HiddenRange {
                device: root_slot_device,
                offset: root_slot_offset.0,
                length: u64::from(parameters.geometry.physical_block_size),
            })
            .chain(named_units.into_iter().map(|location| HiddenRange {
                device: location.device,
                offset: location.slot.0 * SLOT_BYTES_LOCAL,
                length: SLOT_BYTES_LOCAL,
            }))
            .collect(),
        )
    }

    // ------------------------------------------------------------------------
    // 13.3 确认账（登记「读法写死」表「确认」「内容号跟根走」两行）：内容号是装置的写日志记的，按确认的先后递增，跟着根走；
    //      harness 的内容可以重复（空内容），所以读回按根认内容号、再拿字节核（第十二节第二段修订）。管理员回退交回 `Ok` 另记一笔
    //      （回退读法，只报数，第十二节第二段修订）。
    // ------------------------------------------------------------------------

    #[derive(Clone)]
    struct RandomConfirmation {
        /// 这笔确认落在哪一步（起点 0，第 i 步 i + 1，尾巴 = 步数 + 1）。
        order: usize,
        content_number: u64,
        root: TimelineRoot,
        unit_bytes: UnitBytesOfAVersion,
    }

    #[derive(Clone, Default)]
    struct RandomLedger {
        /// 内容号 → 那一版的文件字节（None = 没有文件）。内容号 0 是 mkfs 那一版。
        contents: Vec<Option<Vec<u8>>>,
        /// 交回过 `Ok` 的内容号（没交回 `Ok` 的改内容调用也发一个号，记它要写的字节，读回落到它算「更新」）。
        confirmed_numbers: BTreeSet<u64>,
        content_number_by_root: BTreeMap<TimelineRoot, u64>,
        confirmations: Vec<RandomConfirmation>,
        /// 管理员回退交回 `Ok` 的那几次：(步, 承接的内容号, 回退那次发布的根)。
        rollback_confirmations: Vec<(usize, u64, TimelineRoot)>,
    }

    impl RandomLedger {
        fn after_make_filesystem(genesis: TimelineRoot) -> Self {
            RandomLedger {
                contents: vec![None],
                confirmed_numbers: BTreeSet::from([0]),
                content_number_by_root: BTreeMap::from([(genesis, 0)]),
                confirmations: Vec::new(),
                rollback_confirmations: Vec::new(),
            }
        }

        fn carry(&mut self, root: TimelineRoot, content_number: u64) {
            self.content_number_by_root.insert(root, content_number);
        }

        fn content_number_of(&self, root: TimelineRoot) -> Option<u64> {
            self.content_number_by_root.get(&root).copied()
        }

        fn new_content_number(&mut self, bytes: Option<Vec<u8>>) -> u64 {
            self.contents.push(bytes);
            u64::try_from(self.contents.len() - 1).expect("内容号装得进 u64")
        }

        fn confirm(
            &mut self,
            order: usize,
            root_record: &RootRecord,
            content: Vec<u8>,
            image: &MemoryPool,
        ) {
            let content_number = self.new_content_number(Some(content));
            self.confirmed_numbers.insert(content_number);
            let root = root_pair(root_record);
            self.carry(root, content_number);
            self.confirmations.push(RandomConfirmation {
                order,
                content_number,
                root,
                unit_bytes: unit_bytes_referenced_by(image, root_record).unwrap_or_default(),
            });
        }

        fn record_attempt(&mut self, content: Vec<u8>) -> u64 {
            self.new_content_number(Some(content))
        }

        /// 字面读法：`order` 那一步开始之前最后一笔改内容的确认（没有 = mkfs 那一版，内容号 0）。
        fn last_confirmation_before(&self, order: usize) -> Option<&RandomConfirmation> {
            self.confirmations
                .iter()
                .rev()
                .find(|confirmation| confirmation.order < order)
        }

        fn last_confirmed_number_before(&self, order: usize) -> u64 {
            self.last_confirmation_before(order)
                .map_or(0, |confirmation| confirmation.content_number)
        }

        /// 回退读法：改内容的确认与管理员回退交回 `Ok` 两种一起排，取 `order` 之前最后一笔的内容号。
        fn last_confirmed_number_before_rollback_aware(&self, order: usize) -> u64 {
            let content = self
                .last_confirmation_before(order)
                .map(|confirmation| (confirmation.order, confirmation.content_number));
            let rollback = self
                .rollback_confirmations
                .iter()
                .rev()
                .find(|(rollback_order, _, _)| *rollback_order < order)
                .map(|(rollback_order, number, _)| (*rollback_order, *number));
            match (content, rollback) {
                (Some(content_event), Some(rollback_event)) => {
                    if rollback_event.0 >= content_event.0 {
                        rollback_event.1
                    } else {
                        content_event.1
                    }
                }
                (Some((_, number)), None) | (None, Some((_, number))) => number,
                (None, None) => 0,
            }
        }
    }

    /// 一次冷启动读回按内容号分类（Q3 四类：最后确认那一版 / 更新 / 更旧 / 读失败），字面与回退两种读法各一份。
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RandomReadBack {
        literal: String,
        rollback_aware: String,
        root: Option<TimelineRoot>,
        content_number: Option<u64>,
        /// 读失败时 `crates` 报的错原样（S5 的签名按它认：`MappingStillUnreadable { slot: … }`）。
        failure_debug: Option<String>,
    }

    impl RandomReadBack {
        fn literal_lost(&self) -> bool {
            self.literal.starts_with("older") || self.literal.starts_with("failed")
        }

        fn rollback_aware_lost(&self) -> bool {
            self.rollback_aware.starts_with("older") || self.rollback_aware.starts_with("failed")
        }

        fn render(&self, prefix: &str) -> String {
            format!(
                "{prefix}={} {prefix}_rollback_aware={} {prefix}_root={} {prefix}_content={}",
                self.literal,
                self.rollback_aware,
                key_text(self.root),
                self.content_number
                    .map_or("none".to_string(), |number| number.to_string())
            )
        }
    }

    fn classify_random_read_back(
        report: &RecoveryReport,
        ledger: &RandomLedger,
        literal_last: u64,
        rollback_aware_last: u64,
    ) -> RandomReadBack {
        let outcome_root = chosen_root_of(&report.outcome);
        let landing = report
            .effective_root
            .map(|(instance, txg)| (instance.0, txg.0))
            .or(outcome_root);
        let file: Option<Vec<u8>> = match &report.outcome {
            RecoveryOutcome::Failed { failure, .. } => {
                let failure_debug = format!("{failure:?}");
                let member = format!("failed:{}", error_member_of_debug(&failure_debug));
                return RandomReadBack {
                    literal: member.clone(),
                    rollback_aware: member,
                    root: landing,
                    content_number: None,
                    failure_debug: Some(failure_debug),
                };
            }
            RecoveryOutcome::NoFile { .. } => None,
            RecoveryOutcome::FileRead { content, .. } => Some(content.clone()),
        };
        let by_root = landing.and_then(|root| ledger.content_number_of(root));
        let (number, how) = match by_root {
            Some(number) => {
                if ledger
                    .contents
                    .get(usize::try_from(number).expect("内容号"))
                    != Some(&file)
                {
                    let member = "failed:content_differs_from_the_ledger".to_string();
                    return RandomReadBack {
                        literal: member.clone(),
                        rollback_aware: member,
                        root: landing,
                        content_number: Some(number),
                        failure_debug: None,
                    };
                }
                (number, "")
            }
            None => {
                let candidates: Vec<u64> = ledger
                    .contents
                    .iter()
                    .enumerate()
                    .filter(|(_, bytes)| **bytes == file)
                    .map(|(number, _)| u64::try_from(number).expect("内容号"))
                    .collect();
                let chosen = if candidates.contains(&literal_last) {
                    Some(literal_last)
                } else {
                    candidates.last().copied()
                };
                match chosen {
                    Some(number) => (number, ":by_bytes"),
                    None => {
                        let member = "failed:content_not_in_the_ledger".to_string();
                        return RandomReadBack {
                            literal: member.clone(),
                            rollback_aware: member,
                            root: landing,
                            content_number: None,
                            failure_debug: None,
                        };
                    }
                }
            }
        };
        let literal = if number == literal_last {
            format!("last_confirmed{how}")
        } else if number > literal_last {
            format!("newer:{number}{how}")
        } else {
            format!("older:{number}{how}")
        };
        let rollback_aware = if number == rollback_aware_last {
            format!("last_confirmed{how}")
        } else if !ledger.confirmed_numbers.contains(&number) {
            format!("newer:{number}{how}")
        } else {
            format!("older:{number}{how}")
        };
        RandomReadBack {
            literal,
            rollback_aware,
            root: landing,
            content_number: Some(number),
            failure_debug: None,
        }
    }

    // ------------------------------------------------------------------------
    // 13.4 执行器：八种操作照 harness 执行器的入口与前提各写一遍（`history.rs` 的 `apply_publish_first_file`、`apply_publish_overwrite`、
    //      `apply_publish_without_units`、`apply_mount_writable`、`apply_rollback_while_mounted`、`apply_raise_rollback_floor`、
    //      `apply_cold_start_recover`、`apply_crash_recovery_abandoning_the_newest_root`），不跑模型、不判红就停（M22 那个开关除外）。
    //      一臂拒了可写挂载，之后要会话的操作照 harness 的做法记「前提不满足」、照历史往下走（登记 5.3）。
    // ------------------------------------------------------------------------

    /// 这个进程里开着的可写会话（harness `WritableSession` 的同几样）。
    struct RandomSession {
        allocator: PoolAllocator,
        current: PoolVersion,
        instance: InstanceGeneration,
        shadow_ledger: ShadowLedger,
        parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
    }

    impl RandomSession {
        fn of_the_mount(mounted: Mounted) -> Self {
            RandomSession {
                allocator: mounted.allocator,
                current: mounted.current,
                instance: mounted.output.instance,
                shadow_ledger: mounted.output.shadow_ledger,
                parameters_and_device_table: mounted.output.parameters_and_device_table,
            }
        }
    }

    /// 一次可写挂载（被测的，或第一次崩溃恢复抛弃根之前的合法状态）：Q4、Q5、Q7、Q2 的输入。
    struct RandomMount {
        order: usize,
        step_label: String,
        operation: &'static str,
        phase: &'static str,
        base_fingerprint: u64,
        summary: TestedMountSummary,
        reading: ObservedCallReading,
        judgement: Option<DeviceJudgement>,
        hidden_ranges: usize,
        literal_last: u64,
        rollback_aware_last: u64,
        effective_content: Option<u64>,
        device_effective_content: Option<u64>,
        older_version_returned: bool,
        older_version_returned_rollback_aware: bool,
        stop_clause_four_findings: Vec<String>,
        /// F6（登记第十节 F6 的改动）：崩溃恢复抛弃根那一步的被测挂载上 N-配置 判真、N-槽 判假；别的挂载 None。
        failure_clause_six: Option<bool>,
    }

    /// checker 在一步之后的判定：红的不变量与 I-7.4 被抛弃根那一半点名的根。
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct CheckerReading {
        red_invariants: Vec<&'static str>,
        abandoned_roots_named: Vec<TimelineRoot>,
    }

    impl CheckerReading {
        fn abandoned_half_is_red(&self) -> bool {
            !self.abandoned_roots_named.is_empty()
        }
    }

    /// 「被抛弃的根（实例 X、txg Y）引用的单元……」里的 (X, Y)。
    fn abandoned_root_named_by(detail: &str) -> Vec<TimelineRoot> {
        let mut named = Vec::new();
        let mut rest = detail;
        while let Some(start) = rest.find(ABANDONED_ROOT_VIOLATION_PREFIX) {
            let after = &rest[start + ABANDONED_ROOT_VIOLATION_PREFIX.len()..];
            let instance_text: String = after.chars().take_while(char::is_ascii_digit).collect();
            let after_instance = &after[instance_text.len()..];
            let txg_start = after_instance
                .find("txg ")
                .map(|index| index + "txg ".len());
            let txg_text: String = txg_start
                .map(|index| {
                    after_instance[index..]
                        .chars()
                        .take_while(char::is_ascii_digit)
                        .collect()
                })
                .unwrap_or_default();
            if let (Ok(instance), Ok(txg)) = (instance_text.parse::<u32>(), txg_text.parse::<u64>())
            {
                named.push((instance, txg));
            }
            rest = after_instance;
        }
        named
    }

    fn checker_reading_of(image: &MemoryPool) -> CheckerReading {
        let mut reading = CheckerReading::default();
        for (invariant, verdict) in check_pool_image(image) {
            match verdict {
                InvariantVerdict::Violated(detail) => {
                    reading.red_invariants.push(invariant);
                    if invariant == ABANDONED_ROOT_INVARIANT {
                        for root in abandoned_root_named_by(&detail) {
                            if !reading.abandoned_roots_named.contains(&root) {
                                reading.abandoned_roots_named.push(root);
                            }
                        }
                    }
                }
                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => {}
            }
        }
        reading
    }

    /// 一步的记录：操作、结局、这一步之后的镜像指纹、checker 的判定（这一族不跑 checker 或这一步没写盘时 None）。
    struct RandomStepRecord {
        order: usize,
        operation: String,
        outcome: String,
        fingerprint_after: u64,
        checker: Option<CheckerReading>,
        read_back: Option<RandomReadBack>,
    }

    /// 一次崩溃恢复抛弃根那一步（登记 5.3：V_LC、Q1、Q2 按每一次这一步各算一份）：被藏那条根、那一步开始与交回时的镜像。
    struct CrashRecoveryEvent {
        order: usize,
        hidden_root: TimelineRoot,
        hidden_content_number: Option<u64>,
        base: MemoryPool,
        after: MemoryPool,
        mount_index: usize,
    }

    /// 一段历史在一臂上的一次执行（一个分支：主路径、一个注入点、一个崩溃状态之前的主路径都是它）。
    struct RandomExecutor {
        arm: FourthRunArm,
        parameters: MakeFilesystemParameters,
        devices: Vec<(DeviceIdentity, HistoryDevice)>,
        stream: SharedStream,
        plan: SharedFaultPlan,
        session: Option<RandomSession>,
        ledger: RandomLedger,
        mounts: Vec<RandomMount>,
        steps: Vec<RandomStepRecord>,
        /// 下标 = 步（起点 0）：那一步开始时录制流有几步。
        stream_length_at_step_start: Vec<usize>,
        /// 下标 = 步：那一步做完时整池发出的读 / 写 / 屏障数（注入计划数的，与 harness 的测量跑同一数法）。
        calls_after_step: Vec<FaultCallCounts>,
        crash_recovery_events: Vec<CrashRecoveryEvent>,
        /// 跑 checker 的那几族：最后一次跑 checker 时录制流有几步（harness 只在这一步写过盘时跑）。
        checked_stream_length: usize,
        checks_every_step: bool,
        panicked: Option<(usize, String)>,
        stopped_at_the_first_red: Option<usize>,
        starting_point_error: Option<String>,
        /// 内容号 → 那一版引用的单元第一次被改的那一步（Q1 的时点栏，第十二节第二段修订）。
        first_changes: BTreeMap<u64, FirstChange>,
        /// 要留下哪一步做完时的镜像（注入点那一步：重开读回在它上面做）。
        capture_after_order: Option<usize>,
        captured_image: Option<MemoryPool>,
        /// 冷启动那一步的读回（经这一叠盘读，调用数与 harness 同一数法）。
        pending_read_back: Option<RandomReadBack>,
    }

    /// 一版引用的单元第一次被改：那一步、那一刻它之后已经有几笔改内容的确认、它的根那一刻还在不在根环里。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct FirstChange {
        order: usize,
        later_confirmations: usize,
        root_in_ring_then: bool,
    }

    /// 前提不满足的结局（harness `MissingPrecondition` 的成员名）。
    fn not_applicable(precondition: &str) -> String {
        format!("not_applicable:{precondition}")
    }

    fn refused(error_debug: &str) -> String {
        format!("refused:{}", error_member_of_debug(error_debug))
    }

    impl RandomExecutor {
        /// 起点（harness `HistoryPool::start` 的同一串）：mkfs；`AfterFirstFile` 再在同一个进程里取号、暖机、发第一个文件（内容 `first_file_content`）。
        fn start(
            arm: FourthRunArm,
            starting_point: HistoryStartingPoint,
            plan: SharedFaultPlan,
            checks_every_step: bool,
        ) -> Self {
            let parameters = HistoryDeviceWidth::FourGibibytes.parameters();
            let stream = SharedStream::retaining_contents();
            let mut devices: Vec<(DeviceIdentity, HistoryDevice)> =
                [DeviceIdentity(0), DeviceIdentity(1)]
                    .into_iter()
                    .map(|identity| {
                        (
                            identity,
                            FaultInjectingBlockDevice::new(
                                identity,
                                RecordingBlockDevice::with_shared_stream(
                                    identity,
                                    SparseBlockDevice::new(
                                        history_device_bytes(),
                                        PhysicalBlockSizeInBytes(512),
                                    ),
                                    stream.clone(),
                                ),
                                plan.clone(),
                            ),
                        )
                    })
                    .collect();
            let mut executor = RandomExecutor {
                arm,
                parameters: parameters.clone(),
                devices: Vec::new(),
                stream: stream.clone(),
                plan: plan.clone(),
                session: None,
                ledger: RandomLedger::default(),
                mounts: Vec::new(),
                steps: Vec::new(),
                stream_length_at_step_start: vec![0],
                calls_after_step: Vec::new(),
                crash_recovery_events: Vec::new(),
                checked_stream_length: 0,
                checks_every_step,
                panicked: None,
                stopped_at_the_first_red: None,
                starting_point_error: None,
                first_changes: BTreeMap::new(),
                capture_after_order: None,
                captured_image: None,
                pending_read_back: None,
            };
            let genesis = match make_filesystem(&parameters, &mut devices) {
                Ok(genesis) => genesis,
                Err(error) => {
                    executor.devices = devices;
                    executor.starting_point_error = Some(format!("make_filesystem:{error:?}"));
                    executor.finish_the_starting_point();
                    return executor;
                }
            };
            executor.ledger = RandomLedger::after_make_filesystem(root_pair(&genesis.root));
            match starting_point {
                HistoryStartingPoint::AfterMakeFilesystem => {}
                HistoryStartingPoint::AfterFirstFile => {
                    match Self::acquire_warm_up_and_publish_the_first_file(
                        &parameters,
                        &mut devices,
                        &genesis,
                    ) {
                        Ok((session, warm_up_roots)) => {
                            for root in warm_up_roots {
                                executor.ledger.carry(root, 0);
                            }
                            let first_file_root = *session.current.root();
                            executor.devices = devices;
                            let image = executor.image();
                            executor.ledger.confirm(
                                0,
                                &first_file_root,
                                first_file_content(),
                                &image,
                            );
                            executor.session = Some(session);
                            executor.finish_the_starting_point();
                            return executor;
                        }
                        Err(error) => executor.starting_point_error = Some(error),
                    }
                }
            }
            executor.devices = devices;
            executor.finish_the_starting_point();
            executor
        }

        fn acquire_warm_up_and_publish_the_first_file(
            parameters: &MakeFilesystemParameters,
            devices: &mut Vec<(DeviceIdentity, HistoryDevice)>,
            genesis: &singlefs_core::make_filesystem::MakeFilesystemOutput,
        ) -> Result<(RandomSession, Vec<TimelineRoot>), String> {
            let mut allocator = allocator_after_make_filesystem(parameters, devices, genesis);
            allocator.set_space_admission(SpaceAdmission::JudgedByTheFormula);
            let parameters_and_device_table =
                ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(devices)
                    .map_err(|error| format!("read_the_pool_for_the_session:{error:?}"))?;
            let content = first_file_content();
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer)
                .map_err(|error| format!("acquire_instance:{error:?}"))?;
            let warmed = warm_up(&mut writer, &genesis.root, instance)
                .map_err(|error| format!("warm_up:{error:?}"))?;
            let warm_up_roots: Vec<TimelineRoot> = warmed.roots.iter().map(root_pair).collect();
            let output = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_HARNESS_HISTORIES,
                },
                instance,
                &warmed.last_record_bytes,
            )
            .map_err(|error| format!("publish_first_file:{error:?}"))?;
            Ok((
                RandomSession {
                    allocator,
                    current: PoolVersion::WithFile(output),
                    instance,
                    shadow_ledger: ShadowLedger::On,
                    parameters_and_device_table,
                },
                warm_up_roots,
            ))
        }

        fn finish_the_starting_point(&mut self) {
            self.calls_after_step.push(self.plan.calls());
            let fingerprint_after = if self.devices.is_empty() {
                0
            } else {
                image_fingerprint(&self.image())
            };
            let checker = if self.checks_every_step && !self.devices.is_empty() {
                self.checked_stream_length = self.stream.operation_count();
                Some(checker_reading_of(&self.image()))
            } else {
                None
            };
            self.steps.push(RandomStepRecord {
                order: 0,
                operation: "starting_point".to_string(),
                outcome: self
                    .starting_point_error
                    .clone()
                    .map_or("applied".to_string(), |error| format!("refused:{error}")),
                fingerprint_after,
                checker,
                read_back: None,
            });
        }

        fn image(&self) -> MemoryPool {
            image_of_history_devices(&self.devices)
        }

        /// 现行那一版的内容号（没有会话时取字面读法的最后确认那一版）。
        fn carried_content_number(&self, order: usize) -> u64 {
            self.session
                .as_ref()
                .and_then(|session| {
                    self.ledger
                        .content_number_of(root_pair(session.current.root()))
                })
                .unwrap_or_else(|| self.ledger.last_confirmed_number_before(order))
        }

        /// 一步做完之后根环里有、账里还没有的根（发布报错或挂载半路报错时已经落盘的那几条）按 `default_number` 记内容号。
        fn carry_new_ring_roots(&mut self, default_number: u64) {
            let image = self.image();
            let Ok(geometry) = independent_geometry(&image) else {
                return;
            };
            for root in readable_roots_independent(&image, &geometry) {
                if self.ledger.content_number_of(root).is_none() {
                    self.ledger.carry(root, default_number);
                }
            }
        }

        /// 跑一段历史：逐步执行；某一步 panic ⇒ 到此为止（登记 5.3）；M22 那个开关开着时判红就停。
        fn run_the_operations(&mut self, history: &GeneratedHistory) {
            if self.starting_point_error.is_some() {
                return;
            }
            for (step_index, operation) in history.operations.iter().enumerate() {
                let order = step_index + 1;
                self.stream_length_at_step_start
                    .push(self.stream.operation_count());
                let applied =
                    with_panic_capture(|| self.apply_operation(order, step_index, operation));
                let outcome = match applied {
                    Ok(outcome) => outcome,
                    Err(panic) => {
                        let text = format!("{}: {}", panic.location, panic.message);
                        self.panicked = Some((order, text.clone()));
                        self.calls_after_step.push(self.plan.calls());
                        self.steps.push(RandomStepRecord {
                            order,
                            operation: format!("{:?}", operation.kind()),
                            outcome: format!("panic:{text}"),
                            fingerprint_after: 0,
                            checker: None,
                            read_back: None,
                        });
                        return;
                    }
                };
                self.calls_after_step.push(self.plan.calls());
                let default_number = self.carried_content_number(order);
                self.carry_new_ring_roots(default_number);
                let image = self.image();
                let stream_length = self.stream.operation_count();
                let checker =
                    if self.checks_every_step && stream_length != self.checked_stream_length {
                        self.checked_stream_length = stream_length;
                        Some(checker_reading_of(&image))
                    } else {
                        None
                    };
                let red = checker
                    .as_ref()
                    .is_some_and(|reading| !reading.red_invariants.is_empty());
                let read_back = self.pending_read_back.take();
                self.after_a_step(order, &image);
                self.steps.push(RandomStepRecord {
                    order,
                    operation: format!("{:?}", operation.kind()),
                    outcome,
                    fingerprint_after: image_fingerprint(&image),
                    checker,
                    read_back,
                });
                if RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED && red {
                    self.stopped_at_the_first_red = Some(order);
                    return;
                }
            }
        }

        fn apply_operation(
            &mut self,
            order: usize,
            step_index: usize,
            operation: &HistoryOperation,
        ) -> String {
            let write_time_seconds = WRITE_TIME_SECONDS_OF_THE_HARNESS_HISTORIES
                + u64::try_from(step_index).expect("步号装得进 u64");
            match operation {
                HistoryOperation::PublishFirstFile(content) => {
                    self.apply_publish_first_file(order, content, write_time_seconds)
                }
                HistoryOperation::PublishOverwrite(content) => {
                    self.apply_publish_overwrite(order, content, write_time_seconds)
                }
                HistoryOperation::PublishWithoutUnits => self.apply_publish_without_units(order),
                HistoryOperation::CloseAndMountWritable => {
                    self.session = None;
                    self.observed_mount(order, &order.to_string(), "mount", &[])
                }
                HistoryOperation::RollBackWhileMounted(choice) => {
                    self.apply_rollback_while_mounted(order, *choice)
                }
                HistoryOperation::RaiseRollbackFloor(choice) => {
                    self.apply_raise_rollback_floor(order, *choice)
                }
                HistoryOperation::ColdStartRecover => {
                    self.session = None;
                    let report = recover(&self.devices, JournalPolicy::Consult);
                    self.pending_read_back = Some(classify_random_read_back(
                        &report,
                        &self.ledger,
                        self.ledger.last_confirmed_number_before(order),
                        self.ledger
                            .last_confirmed_number_before_rollback_aware(order),
                    ));
                    "applied:recovered".to_string()
                }
                HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
                    self.apply_crash_recovery_abandoning_the_newest_root(order)
                }
            }
        }

        fn apply_publish_first_file(
            &mut self,
            order: usize,
            choice: &ContentChoice,
            write_time_seconds: u64,
        ) -> String {
            let RandomExecutor {
                parameters,
                devices,
                session,
                ..
            } = self;
            let Some(session) = session.as_mut() else {
                return not_applicable("NoWritableSession");
            };
            let content = choice.bytes();
            let root_to_carry_instance_table_from = *session.current.root();
            let previous_record_bytes = session.current.record_bytes().to_vec();
            let published = {
                let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
                publish_first_file(
                    &mut writer,
                    &mut session.allocator,
                    &root_to_carry_instance_table_from,
                    FirstFile {
                        content: &content,
                        write_time_seconds,
                    },
                    session.instance,
                    &previous_record_bytes,
                )
            };
            match published {
                Ok(output) => {
                    let root = output.root;
                    session.current = PoolVersion::WithFile(output);
                    let image = self.image();
                    self.ledger.confirm(order, &root, content, &image);
                    "applied:published".to_string()
                }
                Err(error) => {
                    let attempted = self.ledger.record_attempt(content);
                    self.carry_new_ring_roots(attempted);
                    refused(&format!("{error:?}"))
                }
            }
        }

        fn apply_publish_overwrite(
            &mut self,
            order: usize,
            choice: &ContentChoice,
            write_time_seconds: u64,
        ) -> String {
            let Some(open_session) = self.session.as_ref() else {
                return not_applicable("NoWritableSession");
            };
            if open_session.current.file_version().is_none() {
                return not_applicable("CurrentVersionWithoutFile");
            }
            let carried = self
                .ledger
                .content_number_of(root_pair(open_session.current.root()))
                .unwrap_or_else(|| self.ledger.last_confirmed_number_before(order));
            let content = choice.bytes();
            let RandomSession {
                allocator,
                current,
                instance,
                shadow_ledger,
                parameters_and_device_table,
            } = self.session.take().expect("上面判过这个进程的会话开着");
            let mut mounted_session = MountedSession {
                allocator,
                current,
                instance,
                shadow_ledger,
                parameters_and_device_table,
            };
            let change = mounted_session.publish_user_change(
                &mut self.devices,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds,
                }),
            );
            let current_root = *mounted_session.current.root();
            self.session = Some(RandomSession {
                allocator: mounted_session.allocator,
                current: mounted_session.current,
                instance: mounted_session.instance,
                shadow_ledger: mounted_session.shadow_ledger,
                parameters_and_device_table: mounted_session.parameters_and_device_table,
            });
            let floor_raises: &[singlefs_core::mount::RaisedFloor] = match &change {
                Ok(published) => &published.floor_raises,
                Err(
                    UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                    | UserChangeRefused::NoFileVersionToChange
                    | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. },
                ) => &[],
                Err(UserChangeRefused::Publish { floor_raises, .. }) => floor_raises,
                Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused_change)) => {
                    &refused_change.floor_raises
                }
                Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused_change)) => {
                    &refused_change.floor_raises
                }
            };
            let floor_raise_roots: Vec<TimelineRoot> = floor_raises
                .iter()
                .flat_map(|raised| {
                    raised
                        .publishes
                        .iter()
                        .map(|publish| root_pair(&publish.root))
                })
                .collect();
            for root in floor_raise_roots {
                self.ledger.carry(root, carried);
            }
            match change {
                Ok(_) => {
                    let image = self.image();
                    self.ledger.confirm(order, &current_root, content, &image);
                    "applied:published".to_string()
                }
                Err(error) => {
                    self.carry_new_ring_roots(carried);
                    let attempted = self.ledger.record_attempt(content);
                    self.carry_new_ring_roots(attempted);
                    refused(&format!("{error:?}"))
                }
            }
        }

        fn apply_publish_without_units(&mut self, order: usize) -> String {
            let carried = self.carried_content_number(order);
            let RandomExecutor {
                parameters,
                devices,
                session,
                ledger,
                ..
            } = self;
            let Some(session) = session.as_mut() else {
                return not_applicable("NoWritableSession");
            };
            let PoolVersion::WithoutFile(previous) = &session.current else {
                return not_applicable("CurrentVersionWithFile");
            };
            let plan = ZeroUnitPublishPlan {
                txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
                counter: previous.record.counter + 1,
                instance: session.instance,
                back_chain: back_chain_of(&previous.record_bytes),
                rollback_floor: previous.root.rollback_floor,
                tree_identifier_watermark: previous.root.tree_identifier_watermark,
            };
            let previous_root = previous.root;
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            match publish_without_units(&mut writer, &previous_root, plan) {
                Ok(output) => {
                    ledger.carry(root_pair(&output.root), carried);
                    session.current = PoolVersion::WithoutFile(output);
                    "applied:published_without_units".to_string()
                }
                Err(error) => refused(&format!("{error:?}")),
            }
        }

        fn apply_rollback_while_mounted(
            &mut self,
            order: usize,
            choice: RollbackTargetChoice,
        ) -> String {
            let image_before = self.image();
            let RandomExecutor {
                parameters,
                devices,
                session,
                ledger,
                ..
            } = self;
            let Some(session) = session.as_mut() else {
                return not_applicable("NoWritableSession");
            };
            let PoolVersion::WithFile(current) = &mut session.current else {
                return not_applicable("CurrentVersionWithoutFile");
            };
            let roots = ring_roots_newest_first_of_the_image(&image_before);
            let Some((newest_txg, newest_instance)) = roots.first().copied() else {
                return not_applicable("NoReadableRootInRing");
            };
            let target = match choice {
                RollbackTargetChoice::RingRoot { index_from_newest } => {
                    let root_count = u64::try_from(roots.len()).expect("根环至多几十条");
                    let (txg, instance) =
                        roots[usize::try_from(index_from_newest % root_count).expect("小于条数")];
                    RollbackTarget {
                        instance: InstanceGeneration(instance),
                        checkpoint_txg: CheckpointTxg(txg),
                    }
                }
                RollbackTargetChoice::BeyondNewestRoot { txg_beyond_newest } => RollbackTarget {
                    instance: InstanceGeneration(newest_instance),
                    checkpoint_txg: CheckpointTxg(newest_txg + 1 + txg_beyond_newest % 3),
                },
                RollbackTargetChoice::RingRootAtTheNewestFloor => {
                    let floor = newest_ring_root_floor_of_the_image(&image_before);
                    let (txg, instance) = roots
                        .iter()
                        .copied()
                        .find(|(txg, _)| Some(*txg) == floor)
                        .unwrap_or((newest_txg, newest_instance));
                    RollbackTarget {
                        instance: InstanceGeneration(instance),
                        checkpoint_txg: CheckpointTxg(txg),
                    }
                }
            };
            let target_key = (target.instance.0, target.checkpoint_txg.0);
            match roll_back_by_a_forward_publish(
                parameters,
                devices,
                &mut session.allocator,
                current,
                target,
            ) {
                Ok(_) => {
                    let root = root_pair(&current.root);
                    let number = ledger
                        .content_number_of(target_key)
                        .unwrap_or_else(|| ledger.last_confirmed_number_before(order));
                    ledger.carry(root, number);
                    ledger.rollback_confirmations.push((order, number, root));
                    format!("applied:rolled_back_to:{}", key_text(Some(target_key)))
                }
                Err(error) => refused(&format!("{error:?}")),
            }
        }

        fn apply_raise_rollback_floor(
            &mut self,
            order: usize,
            choice: FloorTargetChoice,
        ) -> String {
            let carried = self.carried_content_number(order);
            let RandomExecutor {
                parameters,
                devices,
                session,
                ledger,
                ..
            } = self;
            let Some(session) = session.as_mut() else {
                return not_applicable("NoWritableSession");
            };
            let PoolVersion::WithFile(current) = &mut session.current else {
                return not_applicable("CurrentVersionWithoutFile");
            };
            let current_floor = current.root.rollback_floor.0;
            let txg_before_raise = current.root.checkpoint_txg.0;
            let choices = txg_before_raise.saturating_sub(current_floor) + 3;
            let new_floor =
                CheckpointTxg(current_floor + choice.steps_above_current_floor % choices);
            match raise_rollback_floor(
                parameters,
                devices,
                &mut session.allocator,
                current,
                new_floor,
                ShadowLedger::On,
            ) {
                Ok(raised) => {
                    for publish in &raised.publishes {
                        ledger.carry(root_pair(&publish.root), carried);
                    }
                    format!("applied:raised_floor_to:{}", new_floor.0)
                }
                Err(error) => refused(&format!("{error:?}")),
            }
        }

        fn apply_crash_recovery_abandoning_the_newest_root(&mut self, order: usize) -> String {
            let Some(session) = self.session.as_ref() else {
                return not_applicable("NoWritableSession");
            };
            let image_before = self.image();
            let roots = ring_roots_newest_first_of_the_image(&image_before);
            let current_root = session.current.root();
            if roots.first().copied()
                != Some((current_root.checkpoint_txg.0, current_root.instance.0))
            {
                return not_applicable("NewestRingRootIsNotTheCurrentVersionOfThisSession");
            }
            if roots.len() < 2 {
                return not_applicable("NoReadableRootBeforeTheNewest");
            }
            let Some(hidden) =
                ranges_hidden_by_the_crash_recovery(&self.parameters, &session.current)
            else {
                return not_applicable("NewestPublishNamesNoUnit");
            };
            let hidden_root = root_pair(current_root);
            let hidden_content_number = self.ledger.content_number_of(hidden_root);
            self.session = None;
            let outcome = self.observed_mount(order, &order.to_string(), "crash_recovery", &hidden);
            let after = self.image();
            self.crash_recovery_events.push(CrashRecoveryEvent {
                order,
                hidden_root,
                hidden_content_number,
                base: image_before,
                after,
                mount_index: self.mounts.len() - 1,
            });
            outcome
        }

        /// 这一次可写挂载落在哪一段（登记 5.3：第一次崩溃恢复抛弃根之前的挂载是合法状态，另算进 Q4、Q5-同成）。
        fn phase_of_a_mount(&self, operation: &'static str) -> &'static str {
            if operation == "crash_recovery" {
                "crash_recovery"
            } else if self.crash_recovery_events.is_empty() {
                "before_first_crash_recovery"
            } else {
                "after_crash_recovery"
            }
        }

        /// 一次受观测的可写挂载（`hidden` 那几段读回全 0）：记进 `mounts`，交回的是这一步的结局。
        fn observed_mount(
            &mut self,
            order: usize,
            step_label: &str,
            operation: &'static str,
            hidden: &[HiddenRange],
        ) -> String {
            let base = self.image();
            let parameters = self.parameters.clone();
            let (mounted, reading) = observed_call(&mut self.devices, hidden, true, |devices| {
                mount_writable_with_space_admission(
                    &parameters,
                    devices,
                    SpaceAdmission::JudgedByTheFormula,
                )
            });
            let phase = self.phase_of_a_mount(operation);
            let record = evaluate_a_random_mount(
                self.arm,
                &self.ledger,
                &base,
                order,
                step_label,
                operation,
                phase,
                hidden,
                &mounted,
                &reading,
            );
            let outcome = match &mounted {
                Ok(_) => format!("applied:mounted:{}", record.summary.class),
                Err(error) => refused(&format!("{error:?}")),
            };
            let carried = record.effective_content.unwrap_or(record.literal_last);
            self.mounts.push(record);
            match mounted {
                Ok(mounted) => {
                    let mut roots = vec![root_pair(mounted.output.row_publish.root())];
                    roots.extend(
                        mounted
                            .output
                            .warm_up_publishes
                            .iter()
                            .map(|publish| root_pair(publish.root())),
                    );
                    for raised in mounted.output.space_admission.floor_raises() {
                        roots.extend(
                            raised
                                .publishes
                                .iter()
                                .map(|publish| root_pair(&publish.root)),
                        );
                    }
                    for root in roots {
                        self.ledger.carry(root, carried);
                    }
                    self.session = Some(RandomSession::of_the_mount(mounted));
                }
                Err(_) => {
                    let device_carried = self
                        .mounts
                        .last()
                        .and_then(|last_mount| last_mount.device_effective_content)
                        .unwrap_or(carried);
                    self.carry_new_ring_roots(device_carried);
                }
            }
            outcome
        }
    }

    fn summary_of_a_random_mount(
        mounted: &Result<Mounted, singlefs_core::mount::MountError>,
        reading: &ObservedCallReading,
    ) -> TestedMountSummary {
        let ok = mounted.as_ref().ok();
        TestedMountSummary {
            class: mount_outcome_class(mounted, reading.window.write_calls),
            member: mount_error_member_or_none(mounted),
            writes: reading.window.write_calls,
            reads: reading.window.read_calls,
            read_bytes: reading.window.read_bytes,
            waits: reading.waits,
            chosen: ok.map(|succeeded| root_pair(&succeeded.output.chosen_root)),
            effective: ok.map(|succeeded| root_pair(&succeeded.output.effective_root)),
            abandoned_roots_unreadable: ok
                .map(|succeeded| succeeded.output.abandoned_roots_unreadable),
            isolated: ok.map(|succeeded| {
                succeeded
                    .output
                    .isolated_slots_per_device
                    .iter()
                    .map(|(_, slots)| *slots)
                    .sum()
            }),
            observation: reading.observation.clone(),
        }
    }

    /// 一次可写挂载的账（Q2 两种读法、7.3 第一二行的对拍、F6）。
    #[allow(
        clippy::too_many_arguments,
        reason = "一次挂载的坐标与上下文原样进结果行，拆成结构体只多一层搬运"
    )]
    fn evaluate_a_random_mount(
        arm: FourthRunArm,
        ledger: &RandomLedger,
        base: &MemoryPool,
        order: usize,
        step_label: &str,
        operation: &'static str,
        phase: &'static str,
        hidden: &[HiddenRange],
        mounted: &Result<Mounted, singlefs_core::mount::MountError>,
        reading: &ObservedCallReading,
    ) -> RandomMount {
        let summary = summary_of_a_random_mount(mounted, reading);
        let faults: Vec<ThirdRunFault> = hidden
            .iter()
            .map(|range| range.as_third_run_fault())
            .collect();
        let judgement = independent_geometry(base)
            .ok()
            .map(|geometry| device_judgement(base, &geometry, &faults));
        let literal_last = ledger.last_confirmed_number_before(order);
        let rollback_aware_last = ledger.last_confirmed_number_before_rollback_aware(order);
        let effective_content = summary
            .effective
            .and_then(|effective| ledger.content_number_of(effective));
        let device_effective_content = judgement
            .as_ref()
            .and_then(|judged| judged.effective)
            .and_then(|effective| ledger.content_number_of(effective));
        let older_version_returned = effective_content.is_some_and(|number| number < literal_last);
        let older_version_returned_rollback_aware = effective_content.is_some_and(|number| {
            number != rollback_aware_last && ledger.confirmed_numbers.contains(&number)
        });
        let mut stop_clause_four_findings = Vec::new();
        if let Some(judged) = &judgement {
            if arm.is_today() {
                if let Ok(result) = mounted {
                    if judged.chosen != Some(root_pair(&result.output.chosen_root))
                        || judged.effective != Some(root_pair(&result.output.effective_root))
                    {
                        stop_clause_four_findings.push(format!(
                            "7.3 第一行：装置择根 {} / E {}，crates {} / {}",
                            key_text(judged.chosen),
                            key_text(judged.effective),
                            key_text(Some(root_pair(&result.output.chosen_root))),
                            key_text(Some(root_pair(&result.output.effective_root)))
                        ));
                    }
                }
            }
            let refused_by_the_arm = summary.member == arm.refusal_member();
            let device_witness = match arm.witness {
                FourthRunWitness::SystemConfiguration => Some(judged.configuration_newer),
                FourthRunWitness::UnreadableRootRingSlot => Some(judged.unreadable_root_slots > 0),
                FourthRunWitness::NotJudged | FourthRunWitness::SystemConfigurationCarried => None,
            };
            if arm.candidate == FourthRunCandidate::Jia {
                if let Some(witnessed) = device_witness {
                    if witnessed != refused_by_the_arm {
                        stop_clause_four_findings.push(format!(
                            "7.3 第二行：装置算的 N={witnessed}，臂拒={refused_by_the_arm}（成员 {}）",
                            summary.member
                        ));
                    }
                }
            }
        }
        let failure_clause_six = (operation == "crash_recovery").then(|| {
            judgement.as_ref().is_some_and(|judged| {
                judged.configuration_newer && judged.unreadable_root_slots == 0
            })
        });
        RandomMount {
            order,
            step_label: step_label.to_string(),
            operation,
            phase,
            base_fingerprint: image_fingerprint(base),
            summary,
            reading: reading.clone(),
            judgement,
            hidden_ranges: hidden.len(),
            literal_last,
            rollback_aware_last,
            effective_content,
            device_effective_content,
            older_version_returned,
            older_version_returned_rollback_aware,
            stop_clause_four_findings,
            failure_clause_six,
        }
    }

    impl RandomExecutor {
        /// 每一步做完：记各版单元第一次被改的那一步；要留的那一步的镜像留下。
        fn after_a_step(&mut self, order: usize, image: &MemoryPool) {
            let ring_roots = independent_geometry(image)
                .map(|geometry| readable_roots_independent(image, &geometry))
                .unwrap_or_default();
            for confirmation in &self.ledger.confirmations {
                if self
                    .first_changes
                    .contains_key(&confirmation.content_number)
                    || confirmation.order >= order
                {
                    continue;
                }
                let changed =
                    confirmation
                        .unit_bytes
                        .iter()
                        .any(|((device, slot, span), bytes)| {
                            let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
                            image
                                .read(*device, slot * SLOT_BYTES_LOCAL, length)
                                .unwrap_or_default()
                                != *bytes
                        });
                if changed {
                    let later_confirmations = self
                        .ledger
                        .confirmations
                        .iter()
                        .filter(|later| later.content_number > confirmation.content_number)
                        .count();
                    self.first_changes.insert(
                        confirmation.content_number,
                        FirstChange {
                            order,
                            later_confirmations,
                            root_in_ring_then: ring_roots.contains(&confirmation.root),
                        },
                    );
                }
            }
            if self.capture_after_order == Some(order) {
                self.captured_image = Some(image.clone());
            }
        }

        /// 尾巴（登记 5.3 H-随：关闭 C → 可写挂载（不注入）→ 冷启动读回 → ③c）。交回尾巴那次读回、③c 的分类与尾巴之后的镜像。
        fn run_the_tail(&mut self, order_of_the_tail: usize) -> TailReading {
            self.session = None;
            self.stream_length_at_step_start
                .push(self.stream.operation_count());
            let outcome = self.observed_mount(order_of_the_tail, "tail", "tail", &[]);
            self.calls_after_step.push(self.plan.calls());
            let end = self.image();
            self.after_a_step(order_of_the_tail, &end);
            let report = recover(&end, JournalPolicy::Consult);
            let read_back = classify_random_read_back(
                &report,
                &self.ledger,
                self.ledger
                    .last_confirmed_number_before(order_of_the_tail + 1),
                self.ledger
                    .last_confirmed_number_before_rollback_aware(order_of_the_tail + 1),
            );
            let first_hidden_instance = self
                .crash_recovery_events
                .first()
                .map(|event| event.hidden_root.0);
            let tail_class = third_tail_class(&end, &self.ledger, first_hidden_instance);
            self.steps.push(RandomStepRecord {
                order: order_of_the_tail,
                operation: "tail".to_string(),
                outcome,
                fingerprint_after: image_fingerprint(&end),
                checker: None,
                read_back: Some(read_back.clone()),
            });
            TailReading {
                read_back,
                tail_class,
                end,
            }
        }
    }

    /// 尾巴那一截读到的东西。
    struct TailReading {
        read_back: RandomReadBack,
        tail_class: String,
        end: MemoryPool,
    }

    /// ③c（登记 5.3 H-随 与 r3 登记第 373–391 行 ③c 那一行）：历史末尾镜像上实例代号 ≥ 第一次崩溃恢复抛弃根那一步被藏那条根的实例代号 + 1
    /// 的全部自证根的根槽读坏（W、`read_fails`），冷启动读回：读回它落到的那条根自己那一版 / 别的 / 读失败（只有读失败按丢写记）。
    fn third_tail_class(
        end: &MemoryPool,
        ledger: &RandomLedger,
        first_hidden_instance: Option<u32>,
    ) -> String {
        let Some(hidden_instance) = first_hidden_instance else {
            return "na".to_string();
        };
        let Ok(geometry) = independent_geometry(end) else {
            return "failed:geometry".to_string();
        };
        let slots = checker_image::root_slot_positions(&geometry);
        let faults: Vec<ThirdRunFault> = checker_image::valid_roots(end, &geometry)
            .into_iter()
            .filter(|(_, _, view)| view.instance > hidden_instance)
            .filter_map(|(region, slot, view)| {
                slots
                    .iter()
                    .find(|(candidate_region, candidate_slot, _, _)| {
                        *candidate_region == region && *candidate_slot == slot
                    })
                    .map(|(_, _, device, offset)| ThirdRunFault {
                        device: DeviceIdentity(*device),
                        offset: *offset,
                        length: u64::from(geometry.physical_block_size),
                        form: UnreadableForm::ReadFails,
                        duration: FaultDuration::WholeCall,
                        label: format!(
                            "tail_root{}",
                            key_text(Some((view.instance, view.checkpoint_txg)))
                        ),
                    })
            })
            .collect();
        let tail = run_third_run_recover(end, &faults);
        match &tail.outcome.outcome {
            RecoveryOutcome::Failed { failure, .. } => {
                format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
            }
            RecoveryOutcome::NoFile { .. } | RecoveryOutcome::FileRead { .. } => {
                let landing = tail
                    .outcome
                    .effective_root
                    .map(|(instance, txg)| (instance.0, txg.0))
                    .or_else(|| chosen_root_of(&tail.outcome.outcome));
                let expected = landing.and_then(|root| ledger.content_number_of(root));
                let file: Option<Vec<u8>> = match &tail.outcome.outcome {
                    RecoveryOutcome::FileRead { content, .. } => Some(content.clone()),
                    RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => None,
                };
                let own = expected.is_some_and(|number| {
                    ledger
                        .contents
                        .get(usize::try_from(number).expect("内容号"))
                        == Some(&file)
                });
                if own {
                    format!("own_version:{}", key_text(landing))
                } else {
                    format!("other:{}:expected={expected:?}", key_text(landing))
                }
            }
        }
    }

    fn tail_class_is_lost(tail_class: &str) -> bool {
        tail_class.starts_with("failed")
    }

    /// 这份镜像上 `target` 那条根的根记录（fsid 取这份镜像自己的系统配置：harness 的池用 E142 那一个 fsid，不是本装置的）。
    fn root_record_on_the_image(pool: &MemoryPool, target: TimelineRoot) -> Option<RootRecord> {
        let geometry = independent_geometry(pool).ok()?;
        let slot_bytes = usize::try_from(geometry.physical_block_size).ok()?;
        for (_region, _slot, device_number, offset) in checker_image::root_slot_positions(&geometry)
        {
            let Some(bytes) = pool.read(device_number, offset, slot_bytes) else {
                continue;
            };
            if let Some(record) = RootRecord::parse_slot(&bytes, &geometry.filesystem_identifier) {
                if (record.instance.0, record.checkpoint_txg.0) == target {
                    return Some(record);
                }
            }
        }
        None
    }

    // ------------------------------------------------------------------------
    // 13.5 一格（登记第六节 Q0–Q3 与丢写格；5.3「V_LC、Q1、Q2 按每一次崩溃恢复抛弃根那一步各算一份」）：
    //      主路径与注入点分支上每一次崩溃恢复抛弃根那一步、注入点那一步（不是崩溃恢复那一步时另立一格）、每一个崩溃状态各是一格。
    //      V_LC = 那一步开始之前最后确认那一版（字面读法；回退读法另一栏）。Q1 = V_LC 引用的单元在这个分支末尾被改的份数，按那条根在末尾
    //      还在 / 不在根环里分两栏，另报第三栏（尾巴读回落到的根）与时点栏（第一次被改在哪一步、那一刻之后已有几笔确认、那一刻根在不在环里）。
    //      Q2 = 那一格的被测挂载交回 `Ok` 而 E 的内容号更旧。Q3 = 这一格之后的每一次读回（分支末尾那一次、注入点分支的重开、崩溃状态的两次）。
    // ------------------------------------------------------------------------

    struct RandomCell {
        name: String,
        kind: &'static str,
        order: usize,
        protected: Option<RandomConfirmation>,
        protected_rollback_aware: u64,
        mount: Option<usize>,
        read_backs: Vec<(String, RandomReadBack)>,
        tail_class: String,
        destination: Option<String>,
    }

    /// 一格三样的结果。
    struct RandomCellLoss {
        in_ring_overwritten: u64,
        out_of_ring_overwritten: u64,
        landing_overwritten: String,
        first_change: Option<FirstChange>,
        older_version_returned: bool,
        older_version_returned_rollback_aware: bool,
        read_back_lost: bool,
        read_back_lost_rollback_aware: bool,
        read_back_last_confirmed: bool,
        broken_root_slots_read_back_lost: bool,
    }

    impl RandomCellLoss {
        fn overwrite_cell(&self) -> bool {
            self.in_ring_overwritten + self.out_of_ring_overwritten > 0
        }

        fn lost_write_cell(&self) -> bool {
            self.overwrite_cell()
                || self.older_version_returned
                || self.read_back_lost
                || self.broken_root_slots_read_back_lost
        }

        fn lost_write_cell_rollback_aware(&self) -> bool {
            self.overwrite_cell()
                || self.older_version_returned_rollback_aware
                || self.read_back_lost_rollback_aware
                || self.broken_root_slots_read_back_lost
        }
    }

    fn evaluate_a_random_cell(
        cell: &RandomCell,
        mounts: &[RandomMount],
        first_changes: &BTreeMap<u64, FirstChange>,
        base: &MemoryPool,
        end: &MemoryPool,
        final_landing: Option<TimelineRoot>,
    ) -> RandomCellLoss {
        let end_roots = independent_geometry(end)
            .map(|geometry| readable_roots_independent(end, &geometry))
            .unwrap_or_default();
        let overwritten_in = |version: &UnitBytesOfAVersion| -> u64 {
            version
                .iter()
                .filter(|((device, slot, span), bytes)| {
                    let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
                    end.read(*device, slot * SLOT_BYTES_LOCAL, length)
                        .unwrap_or_default()
                        != **bytes
                })
                .count() as u64
        };
        let (in_ring_overwritten, out_of_ring_overwritten) = match &cell.protected {
            Some(protected) => {
                let overwritten = overwritten_in(&protected.unit_bytes);
                if end_roots.contains(&protected.root) {
                    (overwritten, 0)
                } else {
                    (0, overwritten)
                }
            }
            None => (0, 0),
        };
        let landing_overwritten = match final_landing {
            Some(landing) => match root_record_on_the_image(base, landing) {
                Some(record) => match unit_bytes_referenced_by(base, &record) {
                    Ok(version) => overwritten_in(&version).to_string(),
                    Err(error) => format!("unreadable_in_base:{}", error.replace(' ', "_")),
                },
                None => "written_after_the_cell".to_string(),
            },
            None => "none".to_string(),
        };
        let mount = cell.mount.and_then(|index| mounts.get(index));
        RandomCellLoss {
            in_ring_overwritten,
            out_of_ring_overwritten,
            landing_overwritten,
            first_change: cell
                .protected
                .as_ref()
                .and_then(|protected| first_changes.get(&protected.content_number).copied()),
            older_version_returned: mount.is_some_and(|record| record.older_version_returned),
            older_version_returned_rollback_aware: mount
                .is_some_and(|record| record.older_version_returned_rollback_aware),
            read_back_lost: cell
                .read_backs
                .iter()
                .any(|(_, read_back)| read_back.literal_lost()),
            read_back_lost_rollback_aware: cell
                .read_backs
                .iter()
                .any(|(_, read_back)| read_back.rollback_aware_lost()),
            read_back_last_confirmed: cell
                .read_backs
                .iter()
                .all(|(_, read_back)| read_back.literal.starts_with("last_confirmed")),
            broken_root_slots_read_back_lost: tail_class_is_lost(&cell.tail_class),
        }
    }

    /// Q0 的去向（登记「读法写死」表那一行，判的次序见执行员修订第 8 条）：崩溃恢复抛弃根那一步交回（或崩在中途）之后的镜像上判。
    fn destination_of_the_hidden_root(
        event: &CrashRecoveryEvent,
        mounts: &[RandomMount],
        ledger: &RandomLedger,
    ) -> String {
        let Some(position) = hidden_root_position(&event.base, event.hidden_root) else {
            return "other:hidden_root_not_in_the_base".to_string();
        };
        let Some(hidden_content_number) = event.hidden_content_number else {
            return "other:hidden_root_not_in_the_ledger".to_string();
        };
        let tested_ok_effective = mounts
            .get(event.mount_index)
            .filter(|record| class_is_a_successful_mount(record.summary.class))
            .and_then(|record| record.summary.effective);
        hidden_root_destination(
            &position,
            &ledger.content_number_by_root,
            hidden_content_number,
            &event.after,
            tested_ok_effective,
        )
        .name()
    }

    fn render_first_change(first_change: Option<FirstChange>) -> String {
        first_change.map_or("none".to_string(), |change| {
            format!(
                "order{}:later_confirmations{}:in_ring{}",
                change.order, change.later_confirmations, change.root_in_ring_then
            )
        })
    }

    // ------------------------------------------------------------------------
    // 13.6 今天那一臂上与 harness 自己的执行器比（S9 末句的收严、S7、S5a / S5b）：harness 执行器（不跑 checker，只在模型、执行器判定、
    //      panic 上停）每个观察到的位置的镜像指纹与整池调用数，与装置执行器同一位置的逐项比；harness 判红的那一步（跑 checker 那一档）。
    // ------------------------------------------------------------------------

    /// harness 执行器在今天那一份副本上跑同一段历史：每个观察到的位置（步）→（镜像指纹，那一步做完时整池的调用数）。
    fn harness_trace(history: &GeneratedHistory) -> Vec<(usize, u64, FaultCallCounts)> {
        let plan = SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry());
        let mut positions = Vec::new();
        let _run = execute_history_with_faults(
            history,
            harness_execution(PerStepChecker::Skipped),
            &SharedStream::new(),
            &plan,
            &mut |observation| {
                let order = match observation.position {
                    StepPosition::StartingPoint => 0,
                    StepPosition::Operation(index) => index + 1,
                };
                positions.push((order, image_fingerprint(observation.image), plan.calls()));
            },
        );
        positions
    }

    /// harness 执行器（每一步跑 checker、判红就停，快档那一档）第一次判红在哪一步（步序：起点 0，第 i 步 i + 1）、
    /// I-7.4 被抛弃根那一半点名了哪几条根。跑完没判红交回 None。
    fn harness_first_red(
        history: &GeneratedHistory,
    ) -> Option<(usize, Vec<TimelineRoot>, Vec<&'static str>)> {
        let run = execute_history(history);
        match run.ending {
            HistoryEnding::Completed => None,
            HistoryEnding::KnownRed { observation, .. }
            | HistoryEnding::NewFinding { observation, .. } => {
                let order = match observation.position {
                    StepPosition::StartingPoint => 0,
                    StepPosition::Operation(index) => index + 1,
                };
                let mut named = Vec::new();
                let mut invariants = Vec::new();
                for (invariant, detail) in &observation.violations {
                    invariants.push(*invariant);
                    if *invariant == ABANDONED_ROOT_INVARIANT {
                        named.extend(abandoned_root_named_by(detail));
                    }
                }
                Some((order, named, invariants))
            }
        }
    }

    /// 装置执行器（今天那一臂）与 harness 执行器逐步比：交回比了几步、对不上的那几步。
    fn executor_agreement(
        executor: &RandomExecutor,
        trace: &[(usize, u64, FaultCallCounts)],
    ) -> (usize, Vec<String>) {
        let mut mismatches = Vec::new();
        for (order, fingerprint, calls) in trace {
            let mine = executor.steps.iter().find(|step| step.order == *order);
            let my_calls = executor.calls_after_step.get(*order);
            let agrees = mine.is_some_and(|step| step.fingerprint_after == *fingerprint)
                && my_calls == Some(calls);
            if !agrees {
                mismatches.push(format!(
                    "order{order}:harness_fingerprint={fingerprint:016x}:device_fingerprint={}:harness_calls={}/{}/{}:device_calls={}",
                    mine.map_or("none".to_string(), |step| format!("{:016x}", step.fingerprint_after)),
                    calls.reads,
                    calls.writes,
                    calls.barriers,
                    my_calls.map_or("none".to_string(), |counts| format!(
                        "{}/{}/{}",
                        counts.reads, counts.writes, counts.barriers
                    ))
                ));
            }
        }
        (trace.len(), mismatches)
    }

    // ------------------------------------------------------------------------
    // 13.7 harness 的注入点与崩溃状态怎么映到各臂（第十二节第二段修订）：今天那一臂上，harness 摆的每个注入点换成（步，步内第几次那一种调用），
    //      每个崩溃状态换成（它那一段第一个写落在哪一步，那一步里的第几段，段内持久了哪几个写）；打进今天那一臂的产物，别的臂从那份产物读回来。
    // ------------------------------------------------------------------------

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct FaultPointKey {
        index: usize,
        fault: InjectedFault,
        order: usize,
        ordinal_within_the_step: u64,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CrashPointKey {
        index: usize,
        order_of_the_first_write: usize,
        segment_within_the_step: usize,
        persisted_within_the_segment: Vec<bool>,
    }

    fn render_injected_fault(fault: InjectedFault) -> String {
        match fault {
            InjectedFault::ReadReturnsCorruptedBytes { flipped_bit } => format!(
                "{}:{}:{}",
                fault.name(),
                flipped_bit.byte_index,
                flipped_bit.bit_index
            ),
            InjectedFault::WriteFails
            | InjectedFault::WriteIsSwallowed
            | InjectedFault::ReadFails
            | InjectedFault::BarrierFails
            | InjectedFault::BarrierIsSwallowed => fault.name().to_string(),
        }
    }

    fn parse_injected_fault(text: &str) -> Option<InjectedFault> {
        let mut parts = text.split(':');
        let name = parts.next()?;
        [
            InjectedFault::WriteFails,
            InjectedFault::WriteIsSwallowed,
            InjectedFault::ReadFails,
            InjectedFault::BarrierFails,
            InjectedFault::BarrierIsSwallowed,
        ]
        .into_iter()
        .find(|fault| fault.name() == name)
        .or_else(|| {
            if name != "read_returns_corrupted_bytes" {
                return None;
            }
            let byte_index = parts.next()?.parse().ok()?;
            let bit_index = parts.next()?.parse().ok()?;
            Some(InjectedFault::ReadReturnsCorruptedBytes {
                flipped_bit: singlefs_harness::fault_injection::FlippedBit {
                    byte_index,
                    bit_index,
                },
            })
        })
    }

    fn render_persisted(persisted: &[bool]) -> String {
        persisted
            .iter()
            .map(|is_persisted| if *is_persisted { '1' } else { '0' })
            .collect()
    }

    fn parse_persisted(text: &str) -> Vec<bool> {
        text.chars().map(|bit| bit == '1').collect()
    }

    /// 录制流的下标落在哪一步（起点 0）：那一步开始时录制流的长度不超过它的最后一步。
    fn order_of_a_stream_index(
        stream_length_at_step_start: &[usize],
        stream_index: usize,
    ) -> usize {
        stream_length_at_step_start
            .iter()
            .rposition(|start| *start <= stream_index)
            .unwrap_or(0)
    }

    /// 一个分支的录制流切成写表与段（harness 崩溃注入同一切法），外加每一段第一个写落在哪一步。
    struct SegmentedStream {
        writes: Vec<RetainedWrite>,
        segments: Vec<Vec<usize>>,
        order_of_the_first_write: Vec<usize>,
        order_of_each_write: Vec<usize>,
    }

    fn segmented_stream_of(executor: &RandomExecutor) -> SegmentedStream {
        let operations = executor.stream.retained_operations();
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &operations,
            &HistoryDeviceWidth::FourGibibytes.fixed_geometry(),
        );
        let order_of_each_write: Vec<usize> = stream_indexes
            .iter()
            .map(|stream_index| {
                order_of_a_stream_index(&executor.stream_length_at_step_start, *stream_index)
            })
            .collect();
        let order_of_the_first_write = segments
            .iter()
            .map(|segment| {
                segment
                    .first()
                    .map_or(0, |first| order_of_each_write[*first])
            })
            .collect();
        SegmentedStream {
            writes,
            segments,
            order_of_the_first_write,
            order_of_each_write,
        }
    }

    impl SegmentedStream {
        /// 第 `segment_index` 段在它第一个写那一步里是第几段。
        fn segment_within_the_step(&self, segment_index: usize) -> usize {
            let order = self.order_of_the_first_write[segment_index];
            self.order_of_the_first_write[..segment_index]
                .iter()
                .filter(|other| **other == order)
                .count()
        }

        fn find(&self, order: usize, segment_within_the_step: usize) -> Option<usize> {
            self.order_of_the_first_write
                .iter()
                .enumerate()
                .filter(|(_, first)| **first == order)
                .nth(segment_within_the_step)
                .map(|(index, _)| index)
        }

        /// 崩溃状态的镜像：更早的段整段持久，这一段按 `persisted` 挑，更晚的一个都没有（harness 崩溃注入同一摆法）。
        fn crash_image(&self, segment_index: usize, persisted: &[bool]) -> MemoryPool {
            let segment = &self.segments[segment_index];
            let first = segment[0];
            let mut image = MemoryPool::with_devices(
                &[DeviceIdentity(0), DeviceIdentity(1)],
                history_device_bytes(),
            );
            image.apply_writes(&self.writes[..first]);
            for (within, is_persisted) in persisted.iter().enumerate() {
                if *is_persisted {
                    image.apply_writes(&self.writes[first + within..=first + within]);
                }
            }
            image
        }

        /// 崩溃在哪一步：这一段第一个没持久的写落在的那一步（harness `CrashPoint::step` 同一取法）。
        fn crash_order(&self, segment_index: usize, persisted: &[bool]) -> usize {
            let segment = &self.segments[segment_index];
            let withheld = persisted
                .iter()
                .position(|is_persisted| !*is_persisted)
                .unwrap_or(0);
            self.order_of_each_write[segment[withheld]]
        }
    }

    // ------------------------------------------------------------------------
    // 13.8 一段历史在一臂上的一个分支的结果（主路径、注入点、崩溃状态），与结果行。
    // ------------------------------------------------------------------------

    struct BranchResult {
        attempt: String,
        mounts: Vec<RandomMount>,
        cells: Vec<(RandomCell, RandomCellLoss)>,
        detail: String,
        /// 注入点分支：注入真的发出的那一次调用是整池第几次那一种调用（没发出 None）。
        fired_call_ordinal: Option<u64>,
    }

    fn cells_of_an_executed_branch(
        executor: &RandomExecutor,
        tail: &TailReading,
        attempt: &str,
        from_order: usize,
        fault_event: Option<(usize, RandomReadBack)>,
    ) -> Vec<(RandomCell, RandomCellLoss)> {
        let mut cells = Vec::new();
        let mut fault_event_is_a_crash_recovery = false;
        for event in executor
            .crash_recovery_events
            .iter()
            .filter(|event| event.order >= from_order)
        {
            let mut read_backs = vec![("tail_read_back".to_string(), tail.read_back.clone())];
            if let Some((order, reopen)) = &fault_event {
                if *order == event.order {
                    read_backs.push(("reopen_read_back".to_string(), reopen.clone()));
                    fault_event_is_a_crash_recovery = true;
                }
            }
            let cell = RandomCell {
                name: format!("{attempt}:crash_recovery@{}", event.order),
                kind: "crash_recovery",
                order: event.order,
                protected: executor
                    .ledger
                    .last_confirmation_before(event.order)
                    .cloned(),
                protected_rollback_aware: executor
                    .ledger
                    .last_confirmed_number_before_rollback_aware(event.order),
                mount: Some(event.mount_index),
                read_backs,
                tail_class: tail.tail_class.clone(),
                destination: Some(destination_of_the_hidden_root(
                    event,
                    &executor.mounts,
                    &executor.ledger,
                )),
            };
            let loss = evaluate_a_random_cell(
                &cell,
                &executor.mounts,
                &executor.first_changes,
                &event.base,
                &tail.end,
                tail.read_back.root,
            );
            cells.push((cell, loss));
        }
        if let Some((order, reopen)) = fault_event {
            if !fault_event_is_a_crash_recovery {
                let cell = RandomCell {
                    name: format!("{attempt}:fault@{order}"),
                    kind: "fault",
                    order,
                    protected: executor.ledger.last_confirmation_before(order).cloned(),
                    protected_rollback_aware: executor
                        .ledger
                        .last_confirmed_number_before_rollback_aware(order),
                    mount: executor
                        .mounts
                        .iter()
                        .position(|mount| mount.order == order),
                    read_backs: vec![
                        ("tail_read_back".to_string(), tail.read_back.clone()),
                        ("reopen_read_back".to_string(), reopen),
                    ],
                    tail_class: tail.tail_class.clone(),
                    destination: None,
                };
                let base = executor
                    .captured_image
                    .clone()
                    .unwrap_or_else(|| tail.end.clone());
                let loss = evaluate_a_random_cell(
                    &cell,
                    &executor.mounts,
                    &executor.first_changes,
                    &base,
                    &tail.end,
                    tail.read_back.root,
                );
                cells.push((cell, loss));
            }
        }
        cells
    }

    /// 一段历史的主路径（不注入）：跑完操作、接尾巴（这一族要的话）、按每一次崩溃恢复抛弃根那一步立格。
    fn run_the_main_branch(
        arm: FourthRunArm,
        segment: &SegmentTwoHistory,
    ) -> (RandomExecutor, BranchResult) {
        let family = segment.family;
        let mut executor = RandomExecutor::start(
            arm,
            segment.history.starting_point,
            SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry()),
            family.checks_every_step(),
        );
        executor.run_the_operations(&segment.history);
        let finished = executor.panicked.is_none()
            && executor.stopped_at_the_first_red.is_none()
            && executor.starting_point_error.is_none();
        let mut cells = Vec::new();
        let mut detail = String::new();
        if family.runs_the_tail() && finished {
            let order_of_the_tail = segment.history.operations.len() + 1;
            match with_panic_capture(|| executor.run_the_tail(order_of_the_tail)) {
                Ok(tail) => {
                    detail = format!(
                        "{} q3c_tail={}",
                        tail.read_back.render("tail_read_back"),
                        tail.tail_class
                    );
                    cells = cells_of_an_executed_branch(&executor, &tail, "main", 0, None);
                }
                Err(panic) => {
                    executor.panicked = Some((
                        order_of_the_tail,
                        format!("{}: {}", panic.location, panic.message),
                    ));
                }
            }
        }
        let mounts = std::mem::take(&mut executor.mounts);
        (
            executor,
            BranchResult {
                attempt: "main".to_string(),
                mounts,
                cells,
                detail,
                fired_call_ordinal: None,
            },
        )
    }

    /// 一个注入点的分支：先按主路径的调用数把「步内第几次」换成这一臂的整池序号，再开着注入重跑这段历史、接尾巴；
    /// 注入那一步做完时的镜像上冷启动读回（harness 的「重开」）。
    fn run_a_fault_branch(
        arm: FourthRunArm,
        segment: &SegmentTwoHistory,
        main: &RandomExecutor,
        point: &FaultPointKey,
    ) -> BranchResult {
        let attempt = format!("fault{}", point.index);
        let kind = point.fault.call_kind();
        let calls_before = if point.order == 0 {
            FaultCallCounts::default()
        } else {
            main.calls_after_step
                .get(point.order - 1)
                .copied()
                .unwrap_or_default()
        };
        let calls_after = main.calls_after_step.get(point.order).copied();
        let reachable = calls_after.is_some_and(|after| {
            calls_before.of(kind) + point.ordinal_within_the_step <= after.of(kind)
        });
        if !reachable {
            return BranchResult {
                attempt,
                mounts: Vec::new(),
                cells: Vec::new(),
                fired_call_ordinal: None,
                detail: format!(
                    "fault={} order={} ordinal_within_the_step={} fault_not_reached=true arm_calls_in_the_step={}",
                    render_injected_fault(point.fault),
                    point.order,
                    point.ordinal_within_the_step,
                    calls_after.map_or("none".to_string(), |after| {
                        (after.of(kind) - calls_before.of(kind)).to_string()
                    })
                ),
            };
        }
        let arm_ordinal = calls_before.of(kind) + point.ordinal_within_the_step;
        let plan = SharedFaultPlan::armed(
            HistoryDeviceWidth::FourGibibytes.fixed_geometry(),
            FaultSchedule::the_nth_call_across_the_pool(point.fault, arm_ordinal),
        );
        let mut executor =
            RandomExecutor::start(arm, segment.history.starting_point, plan.clone(), false);
        executor.capture_after_order = Some(point.order);
        if point.order == 0 {
            executor.captured_image = Some(executor.image());
        }
        executor.run_the_operations(&segment.history);
        let fired = plan.fired_count();
        let fired_call_ordinal = plan
            .fired()
            .first()
            .map(|fired_fault| fired_fault.call_ordinal_across_the_pool);
        let reopen = executor.captured_image.as_ref().map(|image| {
            let report = recover(image, JournalPolicy::Consult);
            classify_random_read_back(
                &report,
                &executor.ledger,
                executor
                    .ledger
                    .last_confirmed_number_before(point.order + 1),
                executor
                    .ledger
                    .last_confirmed_number_before_rollback_aware(point.order + 1),
            )
        });
        let finished = executor.panicked.is_none()
            && executor.stopped_at_the_first_red.is_none()
            && executor.starting_point_error.is_none();
        let mut cells = Vec::new();
        let mut detail = format!(
            "fault={} order={} ordinal_within_the_step={} arm_ordinal={arm_ordinal} fired={fired} {} starting_point_error={} panicked={}",
            render_injected_fault(point.fault),
            point.order,
            point.ordinal_within_the_step,
            reopen
                .as_ref()
                .map_or("reopen_read_back=none".to_string(), |read_back| read_back
                    .render("reopen_read_back")),
            executor.starting_point_error.is_some(),
            executor.panicked.is_some()
        );
        if let Some(read_back) = &reopen {
            detail.push_str(&format!(
                " reopen_failure={:?}",
                read_back.failure_debug.clone().unwrap_or_default()
            ));
        }
        if finished {
            let order_of_the_tail = segment.history.operations.len() + 1;
            match with_panic_capture(|| executor.run_the_tail(order_of_the_tail)) {
                Ok(tail) => {
                    detail.push_str(&format!(
                        " {} q3c_tail={}",
                        tail.read_back.render("tail_read_back"),
                        tail.tail_class
                    ));
                    cells = cells_of_an_executed_branch(
                        &executor,
                        &tail,
                        &attempt,
                        point.order,
                        reopen.map(|read_back| (point.order, read_back)),
                    );
                }
                Err(panic) => {
                    detail.push_str(&format!(
                        " tail_panic={:?}",
                        format!("{}: {}", panic.location, panic.message)
                    ));
                }
            }
        }
        let mounts = std::mem::take(&mut executor.mounts)
            .into_iter()
            .filter(|mount| mount.order >= point.order)
            .collect();
        BranchResult {
            attempt,
            mounts,
            cells,
            detail,
            fired_call_ordinal,
        }
    }

    /// 崩溃分支那次可写挂载之后读回用的账（第 4 次跑 seg2b 修的装置记账撞键，调查报告 `research/prompts/e158-r4-seg2-ledger-investigation-report.md`
    /// 第一节）：账按根的键 (实例, txg) 记内容号，键里没有时间线这一维；分支挂载在崩溃镜像上取号、发布，会与主路径后来的某次发布落到同一个键上，
    /// 按主路径的账查就查到主路径那一版。分支自己的账 = 主路径的账拷一份（崩溃镜像只含主路径写序列的前缀，镜像里的根与主路径同源），
    /// 再把分支那次挂载写出的根——挂载之后读得出、崩溃镜像里没有同一条（键与根记录逐字节）的——改记成这次挂载承接的那一版。
    /// 承接的那一版照主路径 `observed_mount` 同一条规则：交回 `Ok` 记 E 的内容号，查不到记字面最后确认那一版；拒了记装置判的 E 的内容号，查不到同上。
    fn ledger_after_the_mount_of_a_crash_branch(
        main_ledger: &RandomLedger,
        crash_image: &MemoryPool,
        after_mount: &MemoryPool,
        mount_returned_ok: bool,
        mount: &RandomMount,
    ) -> RandomLedger {
        let carried_when_ok = mount.effective_content.unwrap_or(mount.literal_last);
        let carried = if mount_returned_ok {
            carried_when_ok
        } else {
            mount.device_effective_content.unwrap_or(carried_when_ok)
        };
        let roots_with_their_records = |pool: &MemoryPool| -> BTreeSet<(TimelineRoot, Vec<u8>)> {
            independent_geometry(pool)
                .map(|geometry| {
                    checker_image::valid_roots(pool, &geometry)
                        .into_iter()
                        .map(|(_, _, view)| {
                            ((view.instance, view.checkpoint_txg), view.record_bytes)
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        let roots_in_the_crash_image = roots_with_their_records(crash_image);
        let mut ledger = main_ledger.clone();
        for (root, record_bytes) in roots_with_their_records(after_mount) {
            if !roots_in_the_crash_image.contains(&(root, record_bytes)) {
                ledger.carry(root, carried);
            }
        }
        ledger
    }

    /// 一个崩溃状态的分支（r3 登记第 373–391 行实七-乙那一行）：主路径的写序列上按映射摆出崩溃镜像，冷启动读回，再可写挂载一次、读回一次。
    fn run_a_crash_branch(
        arm: FourthRunArm,
        main: &RandomExecutor,
        stream: &SegmentedStream,
        point: &CrashPointKey,
    ) -> BranchResult {
        let attempt = format!("crash{}", point.index);
        let Some(segment_index) = stream.find(
            point.order_of_the_first_write,
            point.segment_within_the_step,
        ) else {
            return BranchResult {
                attempt,
                mounts: Vec::new(),
                cells: Vec::new(),
                fired_call_ordinal: None,
                detail: format!(
                    "not_constructible=segment_missing order_of_the_first_write={} segment_within_the_step={}",
                    point.order_of_the_first_write, point.segment_within_the_step
                ),
            };
        };
        let writes_in_the_segment = stream.segments[segment_index].len();
        if writes_in_the_segment != point.persisted_within_the_segment.len() {
            return BranchResult {
                attempt,
                mounts: Vec::new(),
                cells: Vec::new(),
                fired_call_ordinal: None,
                detail: format!(
                    "not_constructible=segment_length arm_writes_in_the_segment={writes_in_the_segment} registered_writes={}",
                    point.persisted_within_the_segment.len()
                ),
            };
        }
        let crash_order = stream.crash_order(segment_index, &point.persisted_within_the_segment);
        let image = stream.crash_image(segment_index, &point.persisted_within_the_segment);
        let literal_last = main.ledger.last_confirmed_number_before(crash_order);
        let rollback_aware_last = main
            .ledger
            .last_confirmed_number_before_rollback_aware(crash_order);
        let crash_read_back = classify_random_read_back(
            &recover(&image, JournalPolicy::Consult),
            &main.ledger,
            literal_last,
            rollback_aware_last,
        );
        let mut devices = plain_devices_of(&image);
        let parameters = HistoryDeviceWidth::FourGibibytes.parameters();
        let (mounted, reading) = observed_call(&mut devices, &[], true, |observed| {
            mount_writable_with_space_admission(
                &parameters,
                observed,
                SpaceAdmission::JudgedByTheFormula,
            )
        });
        let phase = if main
            .crash_recovery_events
            .iter()
            .any(|event| event.order < crash_order)
        {
            "after_crash_recovery"
        } else {
            "before_first_crash_recovery"
        };
        let mount = evaluate_a_random_mount(
            arm,
            &main.ledger,
            &image,
            crash_order,
            &attempt,
            "after_crash",
            phase,
            &[],
            &mounted,
            &reading,
        );
        let after_mount = memory_pool_of(&devices, history_device_bytes());
        let ledger_of_the_branch = ledger_after_the_mount_of_a_crash_branch(
            &main.ledger,
            &image,
            &after_mount,
            mounted.is_ok(),
            &mount,
        );
        let after_mount_read_back = classify_random_read_back(
            &recover(&after_mount, JournalPolicy::Consult),
            &ledger_of_the_branch,
            literal_last,
            rollback_aware_last,
        );
        let mounts = vec![mount];
        let cell = RandomCell {
            name: format!("{attempt}:crash@{crash_order}"),
            kind: "crash",
            order: crash_order,
            protected: main.ledger.last_confirmation_before(crash_order).cloned(),
            protected_rollback_aware: rollback_aware_last,
            mount: Some(0),
            read_backs: vec![
                ("crash_read_back".to_string(), crash_read_back.clone()),
                (
                    "after_mount_read_back".to_string(),
                    after_mount_read_back.clone(),
                ),
            ],
            tail_class: "na".to_string(),
            destination: None,
        };
        let loss = evaluate_a_random_cell(
            &cell,
            &mounts,
            &BTreeMap::new(),
            &image,
            &after_mount,
            after_mount_read_back.root,
        );
        BranchResult {
            attempt,
            mounts,
            cells: vec![(cell, loss)],
            fired_call_ordinal: None,
            detail: format!(
                "segment_index={segment_index} crash_order={crash_order} persisted={} {} crash_failure={:?} {}",
                render_persisted(&point.persisted_within_the_segment),
                crash_read_back.render("crash_read_back"),
                crash_read_back.failure_debug.clone().unwrap_or_default(),
                after_mount_read_back.render("after_mount_read_back")
            ),
        }
    }

    // ------------------------------------------------------------------------
    // 13.9 结果行。
    // ------------------------------------------------------------------------

    fn optional_number_text(number: Option<u64>) -> String {
        number.map_or("none".to_string(), |value| value.to_string())
    }

    fn render_random_mount(
        family: SegmentTwoFamily,
        label: &str,
        attempt: &str,
        arm: FourthRunArm,
        mount: &RandomMount,
    ) -> String {
        let judgement = mount.judgement.as_ref().map_or(
            "device_judgement=none truth_newer=false".to_string(),
            render_device_judgement,
        );
        let extras = mount.reading.extras.map_or("none".to_string(), |extras| {
            format!(
                "{}:{}",
                extras.reread_rounds_limit, extras.zeroed_journal_ring_slots_are_cached
            )
        });
        format!(
            "name=r4_random_mount family={} geometry={HARNESS_GEOMETRY_NAME} seed={label} step={} attempt={attempt} arm={} operation={} phase={} duration=W hidden_ranges={} reads_over_hidden_ranges={} base_fingerprint={:016x} {} a_reads_before_first_wait={} a_arm_extras={extras} {judgement} last_confirmed={} last_confirmed_rollback_aware={} e_content={} device_e_content={} q2_rollback={} q2_rollback_rollback_aware={} f6={} s4_findings={:?}",
            family.name(),
            mount.step_label,
            arm.name,
            mount.operation,
            mount.phase,
            mount.hidden_ranges,
            mount.reading.window.reads_over_hidden_ranges,
            mount.base_fingerprint,
            mount.summary.render("a"),
            optional_number_text(mount.reading.window.reads_before_the_first_wait),
            mount.literal_last,
            mount.rollback_aware_last,
            optional_number_text(mount.effective_content),
            optional_number_text(mount.device_effective_content),
            mount.older_version_returned,
            mount.older_version_returned_rollback_aware,
            mount
                .failure_clause_six
                .map_or("na".to_string(), |holds| holds.to_string()),
            mount.stop_clause_four_findings
        )
    }

    fn render_random_cell(
        family: SegmentTwoFamily,
        label: &str,
        arm: FourthRunArm,
        cell: &RandomCell,
        loss: &RandomCellLoss,
    ) -> String {
        let read_backs: Vec<String> = cell
            .read_backs
            .iter()
            .map(|(name, read_back)| read_back.render(name))
            .collect();
        format!(
            "name=r4_random_cell family={} geometry={HARNESS_GEOMETRY_NAME} seed={label} arm={} cell={} kind={} order={} protected={} protected_root={} protected_rollback_aware={} q1_in_ring_overwritten={} q1_out_of_ring_overwritten={} q1_landing_overwritten={} overwrite_cell={} q1_first_change={} q2_rollback={} q2_rollback_rollback_aware={} {} q3_lost={} q3_lost_rollback_aware={} q3_last_confirmed={} q3c_tail={} q3c_lost={} lost_write_cell={} lost_write_cell_rollback_aware={} q0_destination={}",
            family.name(),
            arm.name,
            cell.name,
            cell.kind,
            cell.order,
            cell.protected
                .as_ref()
                .map_or("none".to_string(), |protected| protected.content_number.to_string()),
            key_text(cell.protected.as_ref().map(|protected| protected.root)),
            cell.protected_rollback_aware,
            loss.in_ring_overwritten,
            loss.out_of_ring_overwritten,
            loss.landing_overwritten,
            loss.overwrite_cell(),
            render_first_change(loss.first_change),
            loss.older_version_returned,
            loss.older_version_returned_rollback_aware,
            read_backs.join(" "),
            loss.read_back_lost,
            loss.read_back_lost_rollback_aware,
            loss.read_back_last_confirmed,
            cell.tail_class,
            loss.broken_root_slots_read_back_lost,
            loss.lost_write_cell(),
            loss.lost_write_cell_rollback_aware(),
            cell.destination.clone().unwrap_or_else(|| "na".to_string())
        )
    }

    /// 峰值、为正的次数、期末值（登记 8.1：被谓词消费的量报轨迹）。
    fn trajectory(values: &[u64]) -> String {
        format!(
            "peak{}:positive{}:final{}",
            values.iter().max().copied().unwrap_or(0),
            values.iter().filter(|value| **value > 0).count(),
            values
                .last()
                .map_or("none".to_string(), ToString::to_string)
        )
    }

    fn render_random_segment(
        family: SegmentTwoFamily,
        label: &str,
        arm: FourthRunArm,
        executor: &RandomExecutor,
        branch: &BranchResult,
        operations: usize,
    ) -> String {
        let operation_steps: Vec<&RandomStepRecord> = executor
            .steps
            .iter()
            .filter(|step| step.order >= 1 && step.order <= operations)
            .collect();
        let checked: Vec<&RandomStepRecord> = executor
            .steps
            .iter()
            .filter(|step| step.checker.is_some())
            .collect();
        let first_red = checked
            .iter()
            .find(|step| {
                step.checker
                    .as_ref()
                    .is_some_and(|reading| !reading.red_invariants.is_empty())
            })
            .map(|step| step.order);
        let abandoned_red: Vec<&&RandomStepRecord> = checked
            .iter()
            .filter(|step| {
                step.checker
                    .as_ref()
                    .is_some_and(CheckerReading::abandoned_half_is_red)
            })
            .collect();
        let mut named: Vec<TimelineRoot> = Vec::new();
        for step in &abandoned_red {
            for root in step
                .checker
                .as_ref()
                .map(|reading| reading.abandoned_roots_named.clone())
                .unwrap_or_default()
            {
                if !named.contains(&root) {
                    named.push(root);
                }
            }
        }
        let count_outcomes = |prefix: &str| {
            operation_steps
                .iter()
                .filter(|step| step.outcome.starts_with(prefix))
                .count()
        };
        let abandoned_unreadable: Vec<u64> = branch
            .mounts
            .iter()
            .filter_map(|mount| mount.summary.abandoned_roots_unreadable)
            .collect();
        let isolated: Vec<u64> = branch
            .mounts
            .iter()
            .filter_map(|mount| mount.summary.isolated)
            .collect();
        let rounds: Vec<u64> = branch
            .mounts
            .iter()
            .filter_map(|mount| {
                mount
                    .summary
                    .observation
                    .as_ref()
                    .map(|observation| u64::from(observation.reread_rounds))
            })
            .collect();
        format!(
            "name=r4_random_segment family={} geometry={HARNESS_GEOMETRY_NAME} seed={label} arm={} attempt={} operations={operations} steps_walked={} tail_run={} stop_at_the_first_checker_red={RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED} stopped_at_the_first_red={} panicked={:?} starting_point_error={:?} crash_recovery_orders={:?} applied={} refused={} not_applicable={} checker_runs={} checker_first_red_order={} checker_first_abandoned_red_order={} checker_abandoned_red_steps={} checker_abandoned_roots_named={:?} mounts={} cells={} abandoned_roots_unreadable={} isolated={} reread_rounds={} {}",
            family.name(),
            arm.name,
            branch.attempt,
            operation_steps.len(),
            executor.steps.iter().any(|step| step.operation == "tail"),
            optional_number_text(executor.stopped_at_the_first_red.map(|order| u64::try_from(order).expect("步"))),
            executor.panicked,
            executor.starting_point_error,
            executor
                .crash_recovery_events
                .iter()
                .map(|event| event.order)
                .collect::<Vec<_>>(),
            count_outcomes("applied"),
            count_outcomes("refused"),
            count_outcomes("not_applicable"),
            checked.len(),
            optional_number_text(first_red.map(|order| u64::try_from(order).expect("步"))),
            optional_number_text(
                abandoned_red
                    .first()
                    .map(|step| u64::try_from(step.order).expect("步"))
            ),
            abandoned_red.len(),
            named,
            branch.mounts.len(),
            branch.cells.len(),
            trajectory(&abandoned_unreadable),
            trajectory(&isolated),
            trajectory(&rounds),
            branch.detail
        )
    }

    fn render_random_step(
        family: SegmentTwoFamily,
        label: &str,
        arm: FourthRunArm,
        step: &RandomStepRecord,
    ) -> String {
        format!(
            "name=r4_random_step family={} seed={label} arm={} order={} operation={} outcome={:?} fingerprint_after={:016x} checker_red={} abandoned_named={} {}",
            family.name(),
            arm.name,
            step.order,
            step.operation,
            step.outcome,
            step.fingerprint_after,
            step.checker.as_ref().map_or("not_run".to_string(), |reading| format!(
                "{:?}",
                reading.red_invariants
            )),
            step.checker.as_ref().map_or("not_run".to_string(), |reading| format!(
                "{:?}",
                reading.abandoned_roots_named
            )),
            step.read_back
                .as_ref()
                .map_or("read_back=none".to_string(), |read_back| read_back.render("read_back"))
        )
    }

    /// 一族在一臂上的汇总（Q0–Q3、丢写格；`r4-compare` 把它换成 `r4_loss`）。
    #[derive(Default)]
    struct FamilyTally {
        segments: u64,
        segments_panicked: u64,
        cells: u64,
        lost_write_cells: u64,
        lost_write_cells_rollback_aware: u64,
        /// 丢写格的分解（只报数，不改丢写格的算法）：不算 ③c 的；Q1 只算「还在根环里」那一栏、也不算 ③c 的。
        lost_write_cells_without_the_broken_root_slots_read_back: u64,
        lost_write_cells_with_in_ring_overwrite_only_and_without_the_broken_root_slots_read_back:
            u64,
        overwrite_cells: u64,
        in_ring_overwrite_cells: u64,
        out_of_ring_overwrite_cells: u64,
        overwrite_cells_first_changed_while_the_newest: u64,
        older_version_returned_cells: u64,
        older_version_returned_cells_rollback_aware: u64,
        read_back_lost_cells: u64,
        read_back_lost_cells_rollback_aware: u64,
        broken_root_slots_read_back_lost_cells: u64,
        read_back_last_confirmed_cells: u64,
        destinations: BTreeMap<String, u64>,
        lost_write_cells_by_destination: BTreeMap<String, u64>,
        mounts: u64,
        classes: BTreeMap<String, u64>,
        stop_clause_four_mismatches: u64,
        failure_clause_six_checked: u64,
        failure_clause_six_failures: u64,
        branches_not_constructible: u64,
    }

    impl FamilyTally {
        fn absorb_branch(&mut self, branch: &BranchResult) {
            if branch.detail.contains("not_constructible=")
                || branch.detail.contains("fault_not_reached=true")
            {
                self.branches_not_constructible += 1;
            }
            for mount in &branch.mounts {
                self.mounts += 1;
                *self
                    .classes
                    .entry(mount.summary.class.to_string())
                    .or_insert(0) += 1;
                self.stop_clause_four_mismatches +=
                    u64::try_from(mount.stop_clause_four_findings.len()).expect("条数");
                if let Some(holds) = mount.failure_clause_six {
                    self.failure_clause_six_checked += 1;
                    if !holds {
                        self.failure_clause_six_failures += 1;
                    }
                }
            }
            for (cell, loss) in &branch.cells {
                self.cells += 1;
                let destination = cell
                    .destination
                    .clone()
                    .unwrap_or_else(|| cell.kind.to_string());
                *self.destinations.entry(destination.clone()).or_insert(0) += 1;
                if loss.lost_write_cell() {
                    self.lost_write_cells += 1;
                    *self
                        .lost_write_cells_by_destination
                        .entry(destination)
                        .or_insert(0) += 1;
                }
                if loss.lost_write_cell_rollback_aware() {
                    self.lost_write_cells_rollback_aware += 1;
                }
                if loss.overwrite_cell() || loss.older_version_returned || loss.read_back_lost {
                    self.lost_write_cells_without_the_broken_root_slots_read_back += 1;
                }
                if loss.in_ring_overwritten > 0
                    || loss.older_version_returned
                    || loss.read_back_lost
                {
                    self.lost_write_cells_with_in_ring_overwrite_only_and_without_the_broken_root_slots_read_back += 1;
                }
                if loss.overwrite_cell() {
                    self.overwrite_cells += 1;
                    if loss
                        .first_change
                        .is_some_and(|change| change.later_confirmations == 0)
                    {
                        self.overwrite_cells_first_changed_while_the_newest += 1;
                    }
                }
                if loss.in_ring_overwritten > 0 {
                    self.in_ring_overwrite_cells += 1;
                }
                if loss.out_of_ring_overwritten > 0 {
                    self.out_of_ring_overwrite_cells += 1;
                }
                if loss.older_version_returned {
                    self.older_version_returned_cells += 1;
                }
                if loss.older_version_returned_rollback_aware {
                    self.older_version_returned_cells_rollback_aware += 1;
                }
                if loss.read_back_lost {
                    self.read_back_lost_cells += 1;
                }
                if loss.read_back_lost_rollback_aware {
                    self.read_back_lost_cells_rollback_aware += 1;
                }
                if loss.broken_root_slots_read_back_lost {
                    self.broken_root_slots_read_back_lost_cells += 1;
                }
                if loss.read_back_last_confirmed {
                    self.read_back_last_confirmed_cells += 1;
                }
            }
        }

        fn render(&self, family: SegmentTwoFamily, arm: FourthRunArm, extra: &str) -> String {
            format!(
                "name=r4_random_summary family={} geometry={HARNESS_GEOMETRY_NAME} arm={} segments={} segments_panicked={} cells={} lost_write_cells_q1_q2_q3={} lost_write_cells_rollback_aware={} lost_write_cells_without_q3c={} lost_write_cells_with_q1_in_ring_only_and_without_q3c={} q1_overwrite_cells={} q1_in_ring_overwrite_cells={} q1_out_of_ring_overwrite_cells={} q1_overwrite_cells_first_changed_while_the_newest={} q2_rollback_cells={} q2_rollback_cells_rollback_aware={} q3_lost_cells={} q3_lost_cells_rollback_aware={} q3c_lost_cells={} q3_last_confirmed_cells={} q0_destinations={:?} lost_write_cells_by_destination={:?} tested_mounts={} tested_classes={:?} s4_mismatches={} f6_checked={} f6_failures={} branches_not_constructible={} {extra}",
                family.name(),
                arm.name,
                self.segments,
                self.segments_panicked,
                self.cells,
                self.lost_write_cells,
                self.lost_write_cells_rollback_aware,
                self.lost_write_cells_without_the_broken_root_slots_read_back,
                self.lost_write_cells_with_in_ring_overwrite_only_and_without_the_broken_root_slots_read_back,
                self.overwrite_cells,
                self.in_ring_overwrite_cells,
                self.out_of_ring_overwrite_cells,
                self.overwrite_cells_first_changed_while_the_newest,
                self.older_version_returned_cells,
                self.older_version_returned_cells_rollback_aware,
                self.read_back_lost_cells,
                self.read_back_lost_cells_rollback_aware,
                self.broken_root_slots_read_back_lost_cells,
                self.read_back_last_confirmed_cells,
                self.destinations,
                self.lost_write_cells_by_destination,
                self.mounts,
                self.classes,
                self.stop_clause_four_mismatches,
                self.failure_clause_six_checked,
                self.failure_clause_six_failures,
                self.branches_not_constructible
            )
        }
    }

    fn lines_of_a_branch(
        family: SegmentTwoFamily,
        label: &str,
        arm: FourthRunArm,
        branch: &BranchResult,
    ) -> Vec<String> {
        let mut lines: Vec<String> = branch
            .mounts
            .iter()
            .map(|mount| render_random_mount(family, label, &branch.attempt, arm, mount))
            .collect();
        lines.extend(
            branch
                .cells
                .iter()
                .map(|(cell, loss)| render_random_cell(family, label, arm, cell, loss)),
        );
        lines
    }

    // ------------------------------------------------------------------------
    // 13.10 PC-随（登记 5.4）：H-随 的 (a) 种子基 + 2、(b) 最短复现、(c) 第 43 行那条，每一臂都跑。
    // ------------------------------------------------------------------------

    /// PC-随 的三段：`of` 坐标、在 H-随 里的名字。
    const POSITIVE_CONTROL_RANDOM_SEGMENTS: [(&str, &str); 3] = [
        ("a", "base+2"),
        ("b", SHORTEST_REPRODUCTION_LABEL),
        ("c", ROW_FORTY_THREE_LABEL),
    ];

    fn first_abandoned_red(executor: &RandomExecutor) -> Option<(usize, Vec<TimelineRoot>)> {
        executor.steps.iter().find_map(|step| {
            step.checker
                .as_ref()
                .filter(|reading| reading.abandoned_half_is_red())
                .map(|reading| (step.order, reading.abandoned_roots_named.clone()))
        })
    }

    fn checker_ever_named(executor: &RandomExecutor, root: TimelineRoot) -> bool {
        executor.steps.iter().any(|step| {
            step.checker
                .as_ref()
                .is_some_and(|reading| reading.abandoned_roots_named.contains(&root))
        })
    }

    /// 一段 PC-随 在一臂上的句子（每一句的标签与打不中走哪一条见登记 5.4 PC-随 那一行与执行员修订第 12 条的写法）。
    fn positive_control_random_sentences(
        arm: FourthRunArm,
        of: &str,
        executor: &RandomExecutor,
        branch: &BranchResult,
    ) -> (Vec<ControlSentence>, String) {
        let mut sentences = Vec::new();
        let event = executor.crash_recovery_events.first();
        let mount = event.and_then(|event| branch.mounts.get(event.mount_index));
        let cell = event.and_then(|event| {
            branch
                .cells
                .iter()
                .find(|(cell, _)| cell.kind == "crash_recovery" && cell.order == event.order)
        });
        let first_red = first_abandoned_red(executor);
        if arm.is_today() {
            let expected = match of {
                "a" => Some((
                    POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SEED_PLUS_TWO.0,
                    Some(POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SEED_PLUS_TWO.1),
                )),
                "b" => Some((POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SHORTEST, None)),
                "c" => Some((
                    POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_ROW_FORTY_THREE.0,
                    Some(POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_ROW_FORTY_THREE.1),
                )),
                _ => None,
            };
            if let Some((operation_index, root)) = expected {
                let holds = first_red.as_ref().is_some_and(|(order, named)| {
                    *order == operation_index + 1 && root.is_none_or(|root| named.contains(&root))
                });
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "checker_first_abandoned_red_step_and_root",
                    SentenceVerdict::of(holds),
                    "F19",
                ));
            }
            if of == "b" || of == "c" {
                let protected_root = cell
                    .and_then(|(cell, _)| cell.protected.as_ref().map(|protected| protected.root));
                let verdict = match (protected_root, cell) {
                    (Some(root), Some((_, loss))) => SentenceVerdict::conditional(
                        checker_ever_named(executor, root),
                        loss.in_ring_overwritten > 0,
                    ),
                    (None, _) | (_, None) => SentenceVerdict::NotConstructible,
                };
                sentences.push(sentence(
                    SentenceTag::Measured,
                    "checker_names_the_protected_root_implies_q1_in_ring",
                    verdict,
                    "V1",
                ));
            }
            let verdict = match (mount, cell, event) {
                (Some(mount), Some((_, loss)), Some(event)) => SentenceVerdict::conditional(
                    class_is_a_successful_mount(mount.summary.class)
                        && mount
                            .device_effective_content
                            .zip(event.hidden_content_number)
                            .is_some_and(|(effective, hidden)| effective < hidden),
                    loss.older_version_returned,
                ),
                (None, _, _) | (_, None, _) | (_, _, None) => SentenceVerdict::NotConstructible,
            };
            sentences.push(sentence(
                SentenceTag::Measured,
                "ok_on_an_older_version_implies_q2_rollback",
                verdict,
                "V1",
            ));
        } else {
            match arm.witness {
                FourthRunWitness::SystemConfiguration
                | FourthRunWitness::SystemConfigurationCarried => {
                    let verdict = match mount {
                        Some(mount) => SentenceVerdict::of(
                            mount.summary.member == arm.refusal_member()
                                && mount.summary.writes == 0
                                && (!arm.rereads()
                                    || mount.summary.observation.as_ref().is_some_and(
                                        |observation| {
                                            u64::from(observation.reread_rounds)
                                                == arm.reread_rounds
                                        },
                                    )),
                        ),
                        None => SentenceVerdict::NotConstructible,
                    };
                    sentences.push(sentence(
                        SentenceTag::ArmDefinition,
                        "refuses_at_the_crash_recovery_step_before_any_write",
                        verdict,
                        "F9",
                    ));
                }
                FourthRunWitness::UnreadableRootRingSlot => {
                    let verdict = match mount {
                        Some(mount) => {
                            SentenceVerdict::of(mount.summary.observation.as_ref().is_some_and(
                                |observation| {
                                    observation.witness_judged
                                        && !observation.newer_root_on_the_first_pass
                                },
                            ))
                        }
                        None => SentenceVerdict::NotConstructible,
                    };
                    sentences.push(sentence(
                        SentenceTag::ArmDefinition,
                        "slot_witness_false_at_the_crash_recovery_step",
                        verdict,
                        "F12",
                    ));
                }
                FourthRunWitness::NotJudged => {}
            }
        }
        let detail = format!(
            "crash_recovery_order={} first_abandoned_red={} mount_class={} mount_member={} mount_writes={} q1_in_ring={} q2_rollback={}",
            event.map_or("none".to_string(), |event| event.order.to_string()),
            first_red.map_or("none".to_string(), |(order, named)| format!("order{order}:{named:?}")),
            mount.map_or("none", |mount| mount.summary.class),
            mount.map_or("none".to_string(), |mount| mount.summary.member.clone()),
            mount.map_or("none".to_string(), |mount| mount.summary.writes.to_string()),
            cell.map_or("none".to_string(), |(_, loss)| loss.in_ring_overwritten.to_string()),
            cell.map_or("none".to_string(), |(_, loss)| loss.older_version_returned.to_string())
        );
        (sentences, detail)
    }

    // ------------------------------------------------------------------------
    // 13.11 各族怎么跑。今天那一臂先跑：与 harness 执行器逐步比、摆注入点与崩溃状态、判 S5a / S5b / S7 / S9，打进它的产物；
    //       别的臂从今天那一臂的产物读回注入点、崩溃状态与停下的族（环境变量 `SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT`）。
    // ------------------------------------------------------------------------

    const TODAY_PRODUCT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT";

    /// 今天那一臂的产物里别的臂要用的几样。
    #[derive(Default)]
    struct TodayPoints {
        fault_points: Vec<FaultPointKey>,
        crash_points: BTreeMap<&'static str, Vec<CrashPointKey>>,
        stopped_families: BTreeSet<String>,
    }

    fn family_named(name: &str) -> Option<SegmentTwoFamily> {
        [
            SegmentTwoFamily::RandomRed,
            SegmentTwoFamily::RandomAll,
            SegmentTwoFamily::SeventhFault,
            SegmentTwoFamily::SeventhCrash,
            SegmentTwoFamily::EighthCrash,
        ]
        .into_iter()
        .find(|family| family.name() == name)
    }

    fn read_today_points() -> Result<TodayPoints, String> {
        let path = env::var(TODAY_PRODUCT_ENVIRONMENT_VARIABLE)
            .map_err(|_| format!("环境变量 {TODAY_PRODUCT_ENVIRONMENT_VARIABLE} 没给"))?;
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
        if !text
            .lines()
            .last()
            .is_some_and(|line| line.starts_with("E7RESULT name=done emitted="))
        {
            return Err(format!("{path}: 末行不是完成标记"));
        }
        let mut points = TodayPoints::default();
        for line in text.lines().filter(|line| line.starts_with("E7RESULT ")) {
            let fields = fields_of_a_result_line(line);
            let text_of = |name: &str| fields.get(name).map_or("", String::as_str);
            let number_of = |name: &str| text_of(name).parse::<usize>().ok();
            match text_of("name") {
                "r4_segment_two_fault_point" => {
                    let (Some(index), Some(order), Some(fault), Ok(ordinal_within_the_step)) = (
                        number_of("index"),
                        number_of("order"),
                        parse_injected_fault(text_of("fault")),
                        text_of("ordinal_within_the_step").parse::<u64>(),
                    ) else {
                        return Err(format!("注入点那一行读不出：{line}"));
                    };
                    points.fault_points.push(FaultPointKey {
                        index,
                        fault,
                        order,
                        ordinal_within_the_step,
                    });
                }
                "r4_segment_two_crash_point" => {
                    let (
                        Some(family),
                        Some(index),
                        Some(order_of_the_first_write),
                        Some(segment_within_the_step),
                    ) = (
                        family_named(text_of("family")),
                        number_of("index"),
                        number_of("order_of_the_first_write"),
                        number_of("segment_within_the_step"),
                    )
                    else {
                        return Err(format!("崩溃状态那一行读不出：{line}"));
                    };
                    points
                        .crash_points
                        .entry(family.name())
                        .or_default()
                        .push(CrashPointKey {
                            index,
                            order_of_the_first_write,
                            segment_within_the_step,
                            persisted_within_the_segment: parse_persisted(text_of("persisted")),
                        });
                }
                "stop" if !text_of("family").is_empty() => {
                    points
                        .stopped_families
                        .insert(text_of("family").to_string());
                }
                _ => {}
            }
        }
        Ok(points)
    }

    /// 一族跑完交回的结果行与停机（停了的族只打停机那几行，别的行不进产物）。
    struct FamilyRun {
        lines: Vec<String>,
        stop: Option<String>,
    }

    fn emit_family(family: SegmentTwoFamily, run: FamilyRun) {
        match run.stop {
            Some(stop) => emit_result(&format!(
                "name=stop family={} {stop} dropped_result_lines={}",
                family.name(),
                run.lines.len()
            )),
            None => {
                for line in &run.lines {
                    emit_result(line);
                }
            }
        }
    }

    fn agreement_line(family: SegmentTwoFamily, compared: usize, mismatches: &[String]) -> String {
        format!(
            "name=r4_segment_two_executor_agreement family={} compared_steps={compared} mismatches={} verdict={} detail={:?}",
            family.name(),
            mismatches.len(),
            if compared > 0 && mismatches.is_empty() { "pass" } else { "fail" },
            mismatches.iter().take(5).collect::<Vec<_>>()
        )
    }

    fn run_the_random_families(
        arm: FourthRunArm,
        family: SegmentTwoFamily,
        histories: Vec<SegmentTwoHistory>,
    ) -> FamilyRun {
        let mut lines = Vec::new();
        let mut tally = FamilyTally::default();
        let mut compared = 0usize;
        let mut mismatches: Vec<String> = Vec::new();
        let mut segments_without_an_abandoned_red = Vec::new();
        for segment in &histories {
            let (executor, branch) = run_the_main_branch(arm, segment);
            tally.segments += 1;
            if executor.panicked.is_some() {
                tally.segments_panicked += 1;
            }
            tally.absorb_branch(&branch);
            if arm.is_today() {
                let trace = harness_trace(&segment.history);
                let (steps, segment_mismatches) = executor_agreement(&executor, &trace);
                compared += steps;
                mismatches.extend(
                    segment_mismatches
                        .into_iter()
                        .map(|mismatch| format!("{}:{mismatch}", segment.label)),
                );
                if family == SegmentTwoFamily::RandomRed {
                    let harness_red = harness_first_red(&segment.history);
                    lines.push(format!(
                        "name=r4_harness_first_red family={} seed={} arm={} harness_first_red_order={} harness_abandoned_named={:?} harness_invariants={:?} device_first_abandoned_red={}",
                        family.name(),
                        segment.label,
                        arm.name,
                        harness_red.as_ref().map_or("none".to_string(), |(order, _, _)| order.to_string()),
                        harness_red.as_ref().map(|(_, named, _)| named.clone()).unwrap_or_default(),
                        harness_red.as_ref().map(|(_, _, invariants)| invariants.clone()).unwrap_or_default(),
                        first_abandoned_red(&executor).map_or("none".to_string(), |(order, named)| format!("order{order}:{named:?}"))
                    ));
                    if segment.label.starts_with("base+")
                        && first_abandoned_red(&executor).is_none()
                    {
                        segments_without_an_abandoned_red.push(segment.label.clone());
                    }
                }
            }
            lines.push(render_random_segment(
                family,
                &segment.label,
                arm,
                &executor,
                &branch,
                segment.history.operations.len(),
            ));
            if family == SegmentTwoFamily::RandomRed {
                lines.extend(
                    executor
                        .steps
                        .iter()
                        .map(|step| render_random_step(family, &segment.label, arm, step)),
                );
                if let Some((of, _)) = POSITIVE_CONTROL_RANDOM_SEGMENTS
                    .iter()
                    .find(|(_, label)| *label == segment.label)
                {
                    let (sentences, detail) =
                        positive_control_random_sentences(arm, of, &executor, &branch);
                    let fields: Vec<String> = sentences
                        .iter()
                        .map(|one| {
                            format!(
                                "sentence_{}_{}={}",
                                one.tag.name(),
                                one.name,
                                one.verdict.name()
                            )
                        })
                        .collect();
                    let routes: Vec<&str> = sentences
                        .iter()
                        .filter(|one| {
                            matches!(
                                one.verdict,
                                SentenceVerdict::Fail | SentenceVerdict::NotConstructible
                            )
                        })
                        .map(|one| one.clause)
                        .collect();
                    lines.push(format!(
                        "name=r4_positive_control arm={} control=PC-random of={of} seed={} verdict={} sentences={} {} missed_routes={} {detail}",
                        arm.name,
                        segment.label,
                        overall_verdict(&sentences),
                        sentences.len(),
                        fields.join(" "),
                        if routes.is_empty() { "none".to_string() } else { routes.join(",") }
                    ));
                }
            }
            if family.measures_lost_writes() {
                lines.extend(lines_of_a_branch(family, &segment.label, arm, &branch));
            } else {
                lines.extend(
                    branch.mounts.iter().map(|mount| {
                        render_random_mount(family, &segment.label, "main", arm, mount)
                    }),
                );
            }
        }
        if family == SegmentTwoFamily::RandomRed && arm.is_today() {
            lines.push(format!(
                "name=r4_failure_clause clause=F19 family={} arm={} segments_without_an_abandoned_red={} labels={:?} verdict={}",
                family.name(),
                arm.name,
                segments_without_an_abandoned_red.len(),
                segments_without_an_abandoned_red,
                if segments_without_an_abandoned_red.is_empty() { "not_triggered" } else { "triggered" }
            ));
        }
        let segments_registered_holds = family != SegmentTwoFamily::RandomRed
            || tally.segments == u64::try_from(RANDOM_HISTORY_SEGMENTS_REGISTERED).expect("段数");
        lines.push(format!(
            "name=r4_section_seven_two_anchor arm={} item=\"H-随 实际跑的段数（每臂）\" family={} computed={} registered={} verdict={}",
            arm.name,
            family.name(),
            tally.segments,
            if family == SegmentTwoFamily::RandomRed { RANDOM_HISTORY_SEGMENTS_REGISTERED.to_string() } else { "na".to_string() },
            if segments_registered_holds { "pass" } else { "fail" }
        ));
        if family.measures_lost_writes() {
            lines.push(tally.render(family, arm, ""));
        } else {
            lines.push(format!(
                "name=r4_random_all_summary family={} geometry={HARNESS_GEOMETRY_NAME} arm={} segments={} segments_panicked={} tested_mounts={} tested_classes={:?} s4_mismatches={} f6_checked={} f6_failures={}",
                family.name(),
                arm.name,
                tally.segments,
                tally.segments_panicked,
                tally.mounts,
                tally.classes,
                tally.stop_clause_four_mismatches,
                tally.failure_clause_six_checked,
                tally.failure_clause_six_failures
            ));
        }
        let mut stop = None;
        if arm.is_today() {
            lines.insert(0, agreement_line(family, compared, &mismatches));
            if compared == 0 || !mismatches.is_empty() {
                stop = Some(format!(
                    "clause=S9 reason=\"装置执行器与 harness 执行器在今天那一臂上对不上\" compared_steps={compared} mismatches={} first_mismatches={:?}",
                    mismatches.len(),
                    mismatches.iter().take(3).map(|mismatch| mismatch.replace(' ', "")).collect::<Vec<_>>()
                ));
            }
        }
        if tally.stop_clause_four_mismatches > 0 && stop.is_none() {
            stop = Some(format!(
                "clause=S4 reason=\"7.3 第一二行在这一族的被测挂载上对不上（{} 处）\"",
                tally.stop_clause_four_mismatches
            ));
        }
        FamilyRun { lines, stop }
    }

    /// 实七-甲（故障注入）。今天那一臂：harness 在这一份副本上抽 6 个注入点（`inject_faults_into_history`），用 harness 测量跑的调用数
    /// 换成「步、步内第几次」；S5 按 harness 那一遍与装置这一遍各自复现不复现判。别的臂：从今天那一臂的产物读注入点。
    fn run_the_seventh_fault_family(arm: FourthRunArm, today_points: &TodayPoints) -> FamilyRun {
        let family = SegmentTwoFamily::SeventhFault;
        let segment = seventh_fault_history();
        let (main, main_branch) = run_the_main_branch(arm, &segment);
        let mut lines = Vec::new();
        let mut stop = None;
        let mut harness_reproduces = false;
        let points: Vec<FaultPointKey> = if arm.is_today() {
            let trace = harness_trace(&segment.history);
            let (compared, mismatches) = executor_agreement(&main, &trace);
            lines.push(agreement_line(family, compared, &mismatches));
            if compared == 0 || !mismatches.is_empty() {
                stop = Some(
                    "clause=S7 reason=\"装置执行器与 harness 执行器对不上，注入点映不过去\""
                        .to_string(),
                );
            }
            let injection = inject_faults_into_history(
                &segment.history,
                harness_execution(PerStepChecker::Run),
                SEVENTH_BATCH_FAULT_REPRODUCTION_FAULTS,
            );
            harness_reproduces = injection.new_findings.iter().any(|finding| {
                finding.reopen.contains(&format!(
                    "MappingStillUnreadable {{ slot: SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT}) }}"
                )) && finding.reopen.contains(&format!(
                    "ModelCheckpointTxg({})",
                    SEVENTH_BATCH_SIGNATURE_ROOT.1
                ))
            });
            lines.push(format!(
                "name=r4_harness_signature family={} arm={} harness_reproduces={harness_reproduces} drawn={} new_findings={} findings={:?}",
                family.name(),
                arm.name,
                injection.drawn_faults.len(),
                injection.new_findings.len(),
                injection
                    .new_findings
                    .iter()
                    .map(|finding| format!("{}|{}", finding.drawn.render(), finding.reopen))
                    .collect::<Vec<_>>()
            ));
            let mut keys = Vec::new();
            for (index, drawn) in injection.drawn_faults.iter().enumerate() {
                let order = match drawn.segment.position() {
                    StepPosition::StartingPoint => 0,
                    StepPosition::Operation(step_index) => step_index + 1,
                };
                let kind = drawn.fault.call_kind();
                let calls_before = if order == 0 {
                    0
                } else {
                    trace
                        .iter()
                        .find(|(traced, _, _)| *traced == order - 1)
                        .map_or(0, |(_, _, calls)| calls.of(kind))
                };
                let key = FaultPointKey {
                    index,
                    fault: drawn.fault,
                    order,
                    ordinal_within_the_step: drawn.call_ordinal - calls_before,
                };
                lines.push(format!(
                    "name=r4_segment_two_fault_point family={} index={index} fault={} order={order} ordinal_within_the_step={} harness_call_ordinal={} harness_drawn={:?}",
                    family.name(),
                    render_injected_fault(drawn.fault),
                    key.ordinal_within_the_step,
                    drawn.call_ordinal,
                    drawn.render()
                ));
                keys.push(key);
            }
            keys
        } else {
            today_points.fault_points.clone()
        };
        lines.push(render_random_segment(
            family,
            &segment.label,
            arm,
            &main,
            &main_branch,
            segment.history.operations.len(),
        ));
        lines.extend(
            main.steps
                .iter()
                .map(|step| render_random_step(family, &segment.label, arm, step)),
        );
        let mut tally = FamilyTally {
            segments: 1,
            ..FamilyTally::default()
        };
        tally.absorb_branch(&main_branch);
        lines.extend(lines_of_a_branch(family, &segment.label, arm, &main_branch));
        let mut device_reproduces = false;
        for point in &points {
            let branch = with_panic_capture(|| run_a_fault_branch(arm, &segment, &main, point))
                .unwrap_or_else(|panic| BranchResult {
                    attempt: format!("fault{}", point.index),
                    mounts: Vec::new(),
                    cells: Vec::new(),
                    detail: format!(
                        "panic={:?}",
                        format!("{}: {}", panic.location, panic.message)
                    ),
                    fired_call_ordinal: None,
                });
            device_reproduces = device_reproduces
                || branch.cells.iter().any(|(cell, _)| {
                    cell.read_backs.iter().any(|(name, read_back)| {
                        name == "reopen_read_back"
                            && read_back.root == Some(SEVENTH_BATCH_SIGNATURE_ROOT)
                            && read_back.failure_debug.as_ref().is_some_and(|debug| {
                                debug.contains("MappingStillUnreadable")
                                    && debug.contains(&format!(
                                        "SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT})"
                                    ))
                            })
                    })
                });
            lines.push(format!(
                "name=r4_random_branch family={} seed={} arm={} attempt={} mounts={} cells={} fired_call_ordinal={} {}",
                family.name(),
                segment.label,
                arm.name,
                branch.attempt,
                branch.mounts.len(),
                branch.cells.len(),
                optional_number_text(branch.fired_call_ordinal),
                branch.detail
            ));
            tally.absorb_branch(&branch);
            lines.extend(lines_of_a_branch(family, &segment.label, arm, &branch));
        }
        lines.push(tally.render(
            family,
            arm,
            &format!("device_reproduces={device_reproduces}"),
        ));
        if arm.is_today() {
            let verdict = stop_clause_five(harness_reproduces, device_reproduces);
            lines.push(format!(
                "name=r4_stop_check clause=S5 family={} harness_reproduces={harness_reproduces} device_reproduces={device_reproduces} verdict={verdict}",
                family.name()
            ));
            if verdict == "S5a_stop" && stop.is_none() {
                stop = Some("clause=S5a reason=\"harness 复现得出、装置在今天那一臂上复现不出实七-甲的签名\"".to_string());
            }
        }
        FamilyRun { lines, stop }
    }

    /// S5a / S5b（登记 11.2）：harness 复现得出而装置复现不出 ⇒ 停（S5a）；harness 也复现不出 ⇒ 不停，标「底座上复现不出」（S5b）。
    fn stop_clause_five(harness_reproduces: bool, device_reproduces: bool) -> &'static str {
        match (harness_reproduces, device_reproduces) {
            (true, true) => "reproduced",
            (true, false) => "S5a_stop",
            (false, true) => "S5b_harness_does_not_reproduce_device_does",
            (false, false) => "S5b_not_reproduced_on_the_base",
        }
    }

    /// 实七-乙、实八（崩溃注入）。今天那一臂：harness 在这一份副本上抽 4 个崩溃状态（`inject_crashes_into_history`），按装置这一遍的录制流
    /// 换成「第一个写落在哪一步、那一步里第几段、段内持久哪几个」（段号与写数与 harness 那一遍逐个核，对不上走 S7）。
    fn run_a_crash_family(
        arm: FourthRunArm,
        family: SegmentTwoFamily,
        segment: &SegmentTwoHistory,
        today_points: &TodayPoints,
    ) -> FamilyRun {
        let (main, main_branch) = run_the_main_branch(arm, segment);
        let stream = segmented_stream_of(&main);
        let mut lines = Vec::new();
        let mut stop = None;
        let mut harness_reproduces = false;
        let points: Vec<CrashPointKey> = if arm.is_today() {
            let trace = harness_trace(&segment.history);
            let (compared, mismatches) = executor_agreement(&main, &trace);
            lines.push(agreement_line(family, compared, &mismatches));
            let injection = inject_crashes_into_history(
                &segment.history,
                harness_execution(PerStepChecker::Skipped),
                CrashPointDraw::Sampled {
                    crash_points_per_history: match family {
                        SegmentTwoFamily::SeventhCrash => {
                            SEVENTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS
                        }
                        SegmentTwoFamily::EighthCrash
                        | SegmentTwoFamily::RandomRed
                        | SegmentTwoFamily::RandomAll
                        | SegmentTwoFamily::SeventhFault => {
                            EIGHTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS
                        }
                    },
                },
                None,
            );
            harness_reproduces = match family {
                SegmentTwoFamily::SeventhCrash => injection.new_findings.iter().any(|finding| {
                    finding.read_back.contains(&format!(
                        "MappingStillUnreadable {{ slot: SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT}) }}"
                    ))
                }),
                SegmentTwoFamily::EighthCrash
                | SegmentTwoFamily::RandomRed
                | SegmentTwoFamily::RandomAll
                | SegmentTwoFamily::SeventhFault => harness_first_red(&segment.history)
                    .is_some_and(|(_, named, _)| named.contains(&EIGHTH_BATCH_SIGNATURE_ROOT)),
            };
            lines.push(format!(
                "name=r4_harness_signature family={} arm={} harness_reproduces={harness_reproduces} crash_points={} new_findings={} findings={:?}",
                family.name(),
                arm.name,
                injection.crash_points.len(),
                injection.new_findings.len(),
                injection
                    .new_findings
                    .iter()
                    .map(|finding| format!("{}|{}|{}", finding.crash_point.render(), finding.stage.name(), finding.read_back.chars().take(200).collect::<String>()))
                    .collect::<Vec<_>>()
            ));
            let mut keys = Vec::new();
            let mut mapping_mismatches = Vec::new();
            for (index, crash_point) in injection.crash_points.iter().enumerate() {
                let segment_index = crash_point.segment_index;
                let harness_order = match crash_point.step {
                    StepPosition::StartingPoint => 0,
                    StepPosition::Operation(step_index) => step_index + 1,
                };
                let agrees = stream.segments.get(segment_index).is_some_and(|writes| {
                    writes.len() == crash_point.persisted_within_the_segment.len()
                }) && stream
                    .crash_order(segment_index, &crash_point.persisted_within_the_segment)
                    == harness_order;
                if !agrees {
                    mapping_mismatches.push(format!("crash{index}:{}", crash_point.render()));
                    continue;
                }
                let key = CrashPointKey {
                    index,
                    order_of_the_first_write: stream.order_of_the_first_write[segment_index],
                    segment_within_the_step: stream.segment_within_the_step(segment_index),
                    persisted_within_the_segment: crash_point.persisted_within_the_segment.clone(),
                };
                lines.push(format!(
                    "name=r4_segment_two_crash_point family={} index={index} order_of_the_first_write={} segment_within_the_step={} persisted={} harness_segment_index={segment_index} harness_crash_order={harness_order} harness_point={:?}",
                    family.name(),
                    key.order_of_the_first_write,
                    key.segment_within_the_step,
                    render_persisted(&key.persisted_within_the_segment),
                    crash_point.render()
                ));
                keys.push(key);
            }
            if compared == 0 || !mismatches.is_empty() || !mapping_mismatches.is_empty() {
                stop = Some(format!(
                    "clause=S7 reason=\"装置执行器与 harness 执行器对不上，或崩溃状态映不过去\" executor_mismatches={} mapping_mismatches={:?}",
                    mismatches.len(),
                    mapping_mismatches
                ));
            }
            keys
        } else {
            today_points
                .crash_points
                .get(family.name())
                .cloned()
                .unwrap_or_default()
        };
        lines.push(render_random_segment(
            family,
            &segment.label,
            arm,
            &main,
            &main_branch,
            segment.history.operations.len(),
        ));
        lines.extend(
            main.steps
                .iter()
                .map(|step| render_random_step(family, &segment.label, arm, step)),
        );
        let mut tally = FamilyTally {
            segments: 1,
            ..FamilyTally::default()
        };
        tally.absorb_branch(&main_branch);
        lines.extend(lines_of_a_branch(family, &segment.label, arm, &main_branch));
        let mut device_reproduces = match family {
            SegmentTwoFamily::EighthCrash => checker_ever_named(&main, EIGHTH_BATCH_SIGNATURE_ROOT),
            SegmentTwoFamily::SeventhCrash
            | SegmentTwoFamily::RandomRed
            | SegmentTwoFamily::RandomAll
            | SegmentTwoFamily::SeventhFault => false,
        };
        for point in &points {
            let branch = with_panic_capture(|| run_a_crash_branch(arm, &main, &stream, point))
                .unwrap_or_else(|panic| BranchResult {
                    attempt: format!("crash{}", point.index),
                    mounts: Vec::new(),
                    cells: Vec::new(),
                    detail: format!(
                        "panic={:?}",
                        format!("{}: {}", panic.location, panic.message)
                    ),
                    fired_call_ordinal: None,
                });
            if family == SegmentTwoFamily::SeventhCrash {
                device_reproduces = device_reproduces
                    || branch.cells.iter().any(|(cell, _)| {
                        cell.read_backs.iter().any(|(_, read_back)| {
                            read_back.failure_debug.as_ref().is_some_and(|debug| {
                                debug.contains("MappingStillUnreadable")
                                    && debug.contains(&format!(
                                        "SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT})"
                                    ))
                            })
                        })
                    });
            }
            lines.push(format!(
                "name=r4_random_branch family={} seed={} arm={} attempt={} mounts={} cells={} {}",
                family.name(),
                segment.label,
                arm.name,
                branch.attempt,
                branch.mounts.len(),
                branch.cells.len(),
                branch.detail
            ));
            tally.absorb_branch(&branch);
            lines.extend(lines_of_a_branch(family, &segment.label, arm, &branch));
        }
        lines.push(tally.render(
            family,
            arm,
            &format!("device_reproduces={device_reproduces}"),
        ));
        if arm.is_today() {
            let verdict = stop_clause_five(harness_reproduces, device_reproduces);
            lines.push(format!(
                "name=r4_stop_check clause=S5 family={} harness_reproduces={harness_reproduces} device_reproduces={device_reproduces} verdict={verdict}",
                family.name()
            ));
            if verdict == "S5a_stop" && stop.is_none() {
                stop = Some(format!(
                    "clause=S5a reason=\"harness 复现得出、装置在今天那一臂上复现不出{}的签名\"",
                    family.name()
                ));
            }
        }
        FamilyRun { lines, stop }
    }

    /// 模式 `r4-seg2` 的入口（开跑检查 `run_fourth_run_constants_and_anchors` 已在调用方过了）。
    pub(super) fn run_segment_two(arm: FourthRunArm) {
        let report = investigator_report_text();
        let checks = random_history_source_checks(&report);
        let mut random_sources_hold = true;
        for one in &checks {
            let stops = !one.item.starts_with("5.6");
            if stops && !one.passed {
                random_sources_hold = false;
            }
            emit_result(&format!(
                "name=r4_segment_two_check arm={} item=\"{}\" verdict={} stops_the_family={} detail=\"{}\"",
                arm.name,
                one.item,
                if one.passed { "pass" } else { "fail" },
                stops,
                one.detail
            ));
        }
        let today_points = if arm.is_today() {
            TodayPoints::default()
        } else {
            match read_today_points() {
                Ok(points) => points,
                Err(error) => {
                    emit_result(&format!(
                        "name=stop arm={} reason=\"今天那一臂的第二段产物读不出：{error}\"",
                        arm.name
                    ));
                    return;
                }
            }
        };
        emit_result(&format!(
            "name=r4_segment_two_start arm={} stop_at_the_first_checker_red={RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED} fault_points_from_today={} crash_points_from_today={} stopped_families_from_today={:?}",
            arm.name,
            today_points.fault_points.len(),
            today_points.crash_points.values().map(Vec::len).sum::<usize>(),
            today_points.stopped_families
        ));
        for family in [
            SegmentTwoFamily::RandomRed,
            SegmentTwoFamily::RandomAll,
            SegmentTwoFamily::SeventhFault,
            SegmentTwoFamily::SeventhCrash,
            SegmentTwoFamily::EighthCrash,
        ] {
            if today_points.stopped_families.contains(family.name()) {
                emit_result(&format!(
                    "name=r4_segment_two_family_skipped family={} arm={} reason=\"今天那一臂的产物里这一族停了\"",
                    family.name(),
                    arm.name
                ));
                continue;
            }
            let random_family = matches!(
                family,
                SegmentTwoFamily::RandomRed | SegmentTwoFamily::RandomAll
            );
            if random_family && !random_sources_hold {
                emit_result(&format!(
                    "name=stop family={} arm={} clause=S9 reason=\"种子或写死的两条历史与调查员报告、测试源码对不上\"",
                    family.name(),
                    arm.name
                ));
                continue;
            }
            let run = match family {
                SegmentTwoFamily::RandomRed => {
                    run_the_random_families(arm, family, random_red_histories())
                }
                SegmentTwoFamily::RandomAll => {
                    run_the_random_families(arm, family, random_all_histories())
                }
                SegmentTwoFamily::SeventhFault => run_the_seventh_fault_family(arm, &today_points),
                SegmentTwoFamily::SeventhCrash => {
                    run_a_crash_family(arm, family, &seventh_crash_history(), &today_points)
                }
                SegmentTwoFamily::EighthCrash => {
                    run_a_crash_family(arm, family, &eighth_crash_history(), &today_points)
                }
            };
            emit_family(family, run);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn today() -> FourthRunArm {
            fourth_run_arm_named("today").expect("今天那一臂在臂表里")
        }

        fn row_forty_three() -> SegmentTwoHistory {
            SegmentTwoHistory {
                family: SegmentTwoFamily::RandomRed,
                label: ROW_FORTY_THREE_LABEL.to_string(),
                history: closeout_row_forty_three_history(),
            }
        }

        fn shortest() -> SegmentTwoHistory {
            SegmentTwoHistory {
                family: SegmentTwoFamily::RandomRed,
                label: SHORTEST_REPRODUCTION_LABEL.to_string(),
                history: shortest_reproduction_history(),
            }
        }

        /// 登记 7.2：H-随 (a) 28 个种子、最小 7463871032432355115、最大 7463871032432355205；段数 28 + 1 + 1 = 30；H-随全 68 段。
        #[test]
        fn random_history_seeds_and_segment_counts_match_the_registered_values() {
            let seeds: Vec<u64> = random_red_histories()
                .iter()
                .take(RANDOM_HISTORY_RED_SEED_OFFSETS.len())
                .map(|segment| segment.history.seed.0)
                .collect();
            assert_eq!(seeds.len(), 28);
            assert_eq!(
                seeds.iter().min(),
                Some(&RANDOM_HISTORY_RED_SEED_MINIMUM_REGISTERED)
            );
            assert_eq!(
                seeds.iter().max(),
                Some(&RANDOM_HISTORY_RED_SEED_MAXIMUM_REGISTERED)
            );
            assert_eq!(
                random_red_histories().len(),
                RANDOM_HISTORY_SEGMENTS_REGISTERED
            );
            assert_eq!(random_all_histories().len() + 28, 96);
            assert!(random_all_histories()
                .iter()
                .all(|segment| segment.history.operations.len() == 30));
        }

        #[test]
        fn seeds_listed_on_the_investigator_line_are_read_between_the_brackets() {
            let line = "新发现：第一个种子 7（同签名的种子 [7463871032432355115, 7463871032432355119]），在 Operation(4)";
            assert_eq!(
                seeds_listed_on_the_line(line),
                vec![7_463_871_032_432_355_115, 7_463_871_032_432_355_119]
            );
        }

        #[test]
        fn abandoned_root_named_by_reads_the_instance_and_the_txg() {
            let detail = "被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头；被抛弃的根（实例 12、txg 340）引用的单元已被重新分配";
            assert_eq!(abandoned_root_named_by(detail), vec![(1, 5), (12, 340)]);
            assert!(abandoned_root_named_by("别的违例").is_empty());
        }

        #[test]
        fn injected_faults_and_persisted_subsets_round_trip_through_the_result_line() {
            for fault in [
                InjectedFault::WriteFails,
                InjectedFault::WriteIsSwallowed,
                InjectedFault::ReadFails,
                InjectedFault::BarrierFails,
                InjectedFault::BarrierIsSwallowed,
                InjectedFault::ReadReturnsCorruptedBytes {
                    flipped_bit: singlefs_harness::fault_injection::FlippedBit {
                        byte_index: 1_234_567,
                        bit_index: 5,
                    },
                },
            ] {
                assert_eq!(
                    parse_injected_fault(&render_injected_fault(fault)),
                    Some(fault)
                );
            }
            let persisted = vec![true, false, false, true];
            assert_eq!(render_persisted(&persisted), "1001");
            assert_eq!(parse_persisted("1001"), persisted);
        }

        fn ledger_with_three_confirmations_and_a_rollback() -> RandomLedger {
            let mut ledger = RandomLedger::after_make_filesystem((0, 1));
            for (order, content) in [
                (1usize, b"one".to_vec()),
                (2, b"two".to_vec()),
                (3, b"three".to_vec()),
            ] {
                let number = ledger.new_content_number(Some(content));
                ledger.confirmed_numbers.insert(number);
                let root = (1, 3 + u64::try_from(order).expect("小"));
                ledger.carry(root, number);
                ledger.confirmations.push(RandomConfirmation {
                    order,
                    content_number: number,
                    root,
                    unit_bytes: BTreeMap::new(),
                });
            }
            ledger.carry((1, 8), 1);
            ledger.rollback_confirmations.push((5, 1, (1, 8)));
            ledger
        }

        #[test]
        fn rollback_aware_reading_takes_the_later_of_the_two_kinds_of_confirmation() {
            let ledger = ledger_with_three_confirmations_and_a_rollback();
            assert_eq!(ledger.last_confirmed_number_before(6), 3);
            assert_eq!(ledger.last_confirmed_number_before_rollback_aware(6), 1);
            assert_eq!(ledger.last_confirmed_number_before_rollback_aware(5), 3);
            assert_eq!(ledger.last_confirmed_number_before(2), 1);
        }

        fn report_landing_on(root: TimelineRoot, content: Option<&[u8]>) -> RecoveryReport {
            let key = (InstanceGeneration(root.0), CheckpointTxg(root.1));
            RecoveryReport {
                outcome: match content {
                    Some(bytes) => RecoveryOutcome::FileRead {
                        root: key,
                        content: bytes.to_vec(),
                    },
                    None => RecoveryOutcome::NoFile { root: key },
                },
                effective_root: Some(key),
                journal: singlefs_core::recovery::JournalScanReport::default(),
                mapping_fallbacks: 0,
            }
        }

        #[test]
        fn read_back_is_classified_by_the_content_number_of_the_landing_root() {
            let ledger = ledger_with_three_confirmations_and_a_rollback();
            let newest = classify_random_read_back(
                &report_landing_on((1, 6), Some(b"three")),
                &ledger,
                3,
                1,
            );
            assert_eq!(newest.literal, "last_confirmed");
            assert_eq!(newest.rollback_aware, "older:3");
            let rolled_back =
                classify_random_read_back(&report_landing_on((1, 8), Some(b"one")), &ledger, 3, 1);
            assert_eq!(rolled_back.literal, "older:1");
            assert_eq!(rolled_back.rollback_aware, "last_confirmed");
            assert!(rolled_back.literal_lost());
            assert!(!rolled_back.rollback_aware_lost());
            let wrong_bytes =
                classify_random_read_back(&report_landing_on((1, 6), Some(b"two")), &ledger, 3, 3);
            assert_eq!(
                wrong_bytes.literal,
                "failed:content_differs_from_the_ledger"
            );
            let genesis =
                classify_random_read_back(&report_landing_on((0, 1), None), &ledger, 3, 3);
            assert_eq!(genesis.literal, "older:0");
        }

        /// 装置执行器在今天那一臂上与 harness 执行器逐步同一份镜像、同样多的调用（S9 末句、S7 的前提；变异 M35 让藏起来的那几段不再读回全 0）。
        #[test]
        fn device_executor_agrees_with_the_harness_executor_on_the_two_written_histories() {
            for segment in [shortest(), row_forty_three()] {
                let (executor, _) = run_the_main_branch(today(), &segment);
                let (compared, mismatches) =
                    executor_agreement(&executor, &harness_trace(&segment.history));
                assert!(compared > 0, "{}：harness 一步都没观察到", segment.label);
                assert!(mismatches.is_empty(), "{}：{mismatches:?}", segment.label);
            }
        }

        /// 比对真的看调用数：把 harness 那一份的读数改大一次，就对不上（变异 M34）。
        #[test]
        fn executor_agreement_catches_a_different_call_count() {
            let segment = shortest();
            let (executor, _) = run_the_main_branch(today(), &segment);
            let mut trace = harness_trace(&segment.history);
            trace[1].2.reads += 1;
            let (_, mismatches) = executor_agreement(&executor, &trace);
            assert_eq!(mismatches.len(), 1);
        }

        /// 登记 5.3：这一族判红不截断，第 43 行那条 8 步走完、尾巴跑了（变异 M22 打开「判红就停」）。
        #[test]
        fn row_forty_three_walks_every_step_and_runs_the_tail() {
            let segment = row_forty_three();
            let (executor, branch) = run_the_main_branch(today(), &segment);
            let walked = executor
                .steps
                .iter()
                .filter(|step| step.order >= 1 && step.order <= 8)
                .count();
            assert_eq!(walked, 8);
            assert!(executor.steps.iter().any(|step| step.operation == "tail"));
            assert_eq!(executor.stopped_at_the_first_red, None);
            assert_eq!(branch.cells.len(), 1, "一次崩溃恢复抛弃根，一格");
        }

        /// 最短复现、系统配置没见证到被藏的根（崩溃恢复那一步之前每块盘世代号最大的那一槽系统配置清零、不写回，同 `tests/common/mod.rs`
        /// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`；C554 乙合进 `crates` 之后见证在时崩溃恢复那次挂载拒）：
        /// 崩溃恢复那次挂载交回 `Ok`、落在没有文件的那一版，Q2 = 1（变异 M36 把更旧写成更新）。
        #[test]
        fn shortest_reproduction_with_the_witness_unreadable_returns_the_older_version() {
            let history = shortest_reproduction_history();
            let crash_recovery_step = history.operations.len();
            let mut executor = RandomExecutor::start(
                today(),
                history.starting_point,
                SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry()),
                false,
            );
            executor.run_the_operations(&GeneratedHistory {
                seed: history.seed,
                starting_point: history.starting_point,
                operations: history.operations[..crash_recovery_step - 1].to_vec(),
            });
            let image = executor.image();
            let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
            for (identity, device) in &mut executor.devices {
                let offset =
                    crate::tests::newest_system_configuration_slot_offset(&image, *identity);
                device
                    .write_at(
                        DeviceOffsetInBytes(offset),
                        &vec![0u8; slot_bytes],
                        WriteDurability::Plain,
                    )
                    .expect("清零");
            }
            let outcome = executor.apply_operation(
                crash_recovery_step,
                crash_recovery_step - 1,
                &history.operations[crash_recovery_step - 1],
            );
            assert!(outcome.starts_with("applied:mounted"), "{outcome}");
            let mount = executor.mounts.last().expect("崩溃恢复那次挂载记下了");
            assert_eq!(mount.operation, "crash_recovery");
            assert_eq!(mount.literal_last, 1);
            assert_eq!(mount.effective_content, Some(0));
            assert!(mount.older_version_returned);
        }

        /// 崩溃镜像：最后一段整段持久 = 全部写都落了盘 = 执行器此刻的镜像；全不持久 = 前面各段（变异 M33 把挑法反过来）。
        #[test]
        fn crash_image_applies_exactly_the_persisted_writes_of_the_segment() {
            let segment = shortest();
            let mut executor = RandomExecutor::start(
                today(),
                segment.history.starting_point,
                SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry()),
                false,
            );
            executor.run_the_operations(&segment.history);
            let stream = segmented_stream_of(&executor);
            let last = stream.segments.len() - 1;
            let every_write = vec![true; stream.segments[last].len()];
            assert_eq!(
                image_fingerprint(&stream.crash_image(last, &every_write)),
                image_fingerprint(&executor.image())
            );
            let no_write = vec![false; stream.segments[last].len()];
            let mut earlier = MemoryPool::with_devices(
                &[DeviceIdentity(0), DeviceIdentity(1)],
                history_device_bytes(),
            );
            earlier.apply_writes(&stream.writes[..stream.segments[last][0]]);
            assert_eq!(
                image_fingerprint(&stream.crash_image(last, &no_write)),
                image_fingerprint(&earlier)
            );
        }

        /// seg2b 修的装置记账撞键：崩溃分支那次可写挂载写出的根，读回按这次挂载承接的那一版判，不按主路径在同一个 (实例, txg) 上记的那一版判。
        /// 主路径的账在分支落到的那个键上改记成另一版（照 -配置 各臂实七-乙 `crash0`：主路径第 8 步那次挂载在同一个键上发布过内容号 1），
        /// 读回仍是最后确认那一版、内容号 0（变异 M39 让读回改回拿主路径的账，M40 让分支写出的根一条都不改记）。
        #[test]
        fn crash_branch_read_back_after_its_mount_goes_by_the_roots_that_branch_wrote() {
            let segment = seventh_crash_history();
            let (mut main, _) = run_the_main_branch(today(), &segment);
            let stream = segmented_stream_of(&main);
            let point = CrashPointKey {
                index: 0,
                order_of_the_first_write: 0,
                segment_within_the_step: 7,
                persisted_within_the_segment: vec![false],
            };
            let after_mount_read_back_of = |branch: &BranchResult| -> RandomReadBack {
                let (cell, _) = branch.cells.first().expect("一个崩溃状态一格");
                cell.read_backs
                    .iter()
                    .find(|(name, _)| name == "after_mount_read_back")
                    .map(|(_, read_back)| read_back.clone())
                    .expect("挂载之后读回了一次")
            };
            let untouched =
                after_mount_read_back_of(&run_a_crash_branch(today(), &main, &stream, &point));
            assert_eq!(untouched.literal, "last_confirmed");
            assert_eq!(untouched.content_number, Some(0));
            let landing = untouched.root.expect("读回落到一条根");
            let crash_image = stream.crash_image(
                stream
                    .find(
                        point.order_of_the_first_write,
                        point.segment_within_the_step,
                    )
                    .expect("今天那一臂上有这一段"),
                &point.persisted_within_the_segment,
            );
            let roots_in_the_crash_image = independent_geometry(&crash_image)
                .map(|geometry| readable_roots_independent(&crash_image, &geometry))
                .expect("崩溃镜像有几何");
            assert!(
                !roots_in_the_crash_image.contains(&landing),
                "落到的根 {landing:?} 是分支那次挂载写出的，崩溃镜像里没有"
            );
            let another_version = main
                .ledger
                .new_content_number(Some(b"another timeline".to_vec()));
            main.ledger.carry(landing, another_version);
            let collided =
                after_mount_read_back_of(&run_a_crash_branch(today(), &main, &stream, &point));
            assert_eq!(collided.literal, "last_confirmed");
            assert_eq!(collided.content_number, Some(0));
        }

        /// 注入点的映射：「步内第 k 次」换成这一臂整池第（那一步之前的调用数 + k）次（变异 M29 差一）。
        /// 步取这一臂主路径上第一个至少有两次屏障的那一步（C554 乙合进 `crates` 之后，最短复现第 3 步的崩溃恢复拒了、那一步的屏障不够两次）。
        #[test]
        fn fault_point_fires_at_the_arm_ordinal_of_the_named_call_within_the_step() {
            let segment = shortest();
            let (main, _) = run_the_main_branch(today(), &segment);
            let order = (1..main.calls_after_step.len())
                .find(|order| {
                    main.calls_after_step[*order].barriers
                        >= main.calls_after_step[*order - 1].barriers + 2
                })
                .expect("最短复现的主路径上有一步至少两次屏障");
            let point = FaultPointKey {
                index: 0,
                fault: InjectedFault::BarrierFails,
                order,
                ordinal_within_the_step: 2,
            };
            let branch = run_a_fault_branch(today(), &segment, &main, &point);
            let barriers_before = main.calls_after_step[order - 1].barriers;
            assert_eq!(branch.fired_call_ordinal, Some(barriers_before + 2));
        }
    }
}

fn main() {
    let command_line_arguments: Vec<String> = env::args().collect();
    let mode = command_line_arguments
        .get(1)
        .map(String::as_str)
        .unwrap_or("all");
    if run_second_run_mode(mode) {
        return;
    }
    if run_third_run_mode(mode, &command_line_arguments) {
        return;
    }
    if run_fourth_run_mode(mode, &command_line_arguments) {
        return;
    }

    let constants_ok = run_constants_and_anchors();
    if !constants_ok {
        emit_result("name=stop reason=V3 detail=\"5.7 常量回比或第七节算术有不等，整轮不开跑\"");
        std::process::exit(1);
    }
    if mode == "constants" {
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return;
    }

    let independent_decode_check_passed = run_independent_decode_cross_check();
    if !independent_decode_check_passed {
        emit_result(
            "name=stop reason=S5 detail=\"装置自己的解码与 crates::recovery 的输出对不上\"",
        );
        std::process::exit(1);
    }
    if mode == "s5" {
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return;
    }

    let initial_overwrite_counts = [1u64, 2, 3];
    let mut overall_peak_events_live = 0u64;
    let mut pc3_node: Option<(SimNode, MakeFilesystemParameters, PoolGeometry)> = None;

    if mode == "all" || mode == "h3-g0" || mode == "pc3" {
        let summary = run_root_choice_geometry(&GEOMETRY_PRIMARY, &initial_overwrite_counts, 3);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
        if let Some(node) = summary.first_events_live_node {
            let parameters = parameters_for(&GEOMETRY_PRIMARY);
            if let Ok(geometry_view) = independent_geometry(&node.pool) {
                pc3_node = Some((node, parameters, geometry_view));
            }
        }
    }
    if mode == "all" || mode == "h3-s16" {
        let summary =
            run_root_choice_geometry(&GEOMETRY_LARGER_ROOT_RING, &initial_overwrite_counts, 3);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
    }
    if mode == "all" || mode == "h3-small-ring" {
        let small_ring_bytes = find_small_ring();
        emit_result(&format!(
            "name=small_ring_search found_ring_bytes={small_ring_bytes} search_started_at_bytes=49152 step_bytes=16384"
        ));
        let small_ring = Geometry {
            label: "小环",
            slots_per_region: 8,
            journal_ring_bytes: small_ring_bytes,
        };
        let summary = run_root_choice_geometry(&small_ring, &initial_overwrite_counts, 3);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
    }
    if mode == "all" || mode == "h3-s4" {
        let summary =
            run_root_choice_geometry(&GEOMETRY_SMALLER_ROOT_RING, &initial_overwrite_counts, 4);
        overall_peak_events_live = overall_peak_events_live.max(summary.peak_events_live);
    }

    if mode == "all"
        || mode == "h3-g0"
        || mode == "h3-s16"
        || mode == "h3-small-ring"
        || mode == "h3-s4"
    {
        emit_result(&format!(
            "name=peak_live_rollback_events_across_the_family value={overall_peak_events_live}"
        ));
        report_witness_width_and_slot_overlap_arithmetic(overall_peak_events_live);
    }

    if mode == "all" || mode == "pc3" {
        match pc3_node {
            Some((node, parameters, geometry_view)) => {
                let pc3_ok = run_pc3(&node, &parameters, &geometry_view);
                emit_result(&format!(
                    "name=pc3_overall_verdict verdict={}",
                    if pc3_ok { "pass" } else { "fail" }
                ));
            }
            None => emit_result(
                "name=pc3_overall_verdict verdict=not_applicable reason=\"GEOMETRY_PRIMARY 全族里没有找到 events_live≥1 的节点\"",
            ),
        }
    }

    // Q3-1（跑前登记第二段，岔路 3 候选 3 那一半）：今天那一臂的 H3 × Φ3 违例比例。
    // 不并进 "all"：这一段代价数量级不同（每个 H3 节点还要按 |F| ≤ 2 的组合各做一次 (a)，
    // 打中省法门槛之外的还要各做一次 (b)(c)），独立跑、独立记时。
    if mode == "q3-1-g0" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &initial_overwrite_counts, 3);
    }
    // 只跑最浅的一层，用来估算每个节点的挂钟代价（不是登记里要的产物，交回报告说明为什么有这一档）。
    if mode == "q3-1-g0-timing-probe" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &[1], 1);
    }
    if mode == "q3-1-g0-timing-probe-2" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &[1], 2);
    }
    if mode == "q3-1-g0-timing-probe-3" {
        run_fault_set_violation_family(&GEOMETRY_PRIMARY, &[1], 3);
    }
    if mode == "q3-1-s16" {
        run_fault_set_violation_family(&GEOMETRY_LARGER_ROOT_RING, &initial_overwrite_counts, 3);
    }
    if mode == "q3-1-small-ring" {
        let small_ring_bytes = find_small_ring();
        emit_result(&format!(
            "name=small_ring_search found_ring_bytes={small_ring_bytes} search_started_at_bytes=49152 step_bytes=16384"
        ));
        let small_ring = Geometry {
            label: "小环",
            slots_per_region: 8,
            journal_ring_bytes: small_ring_bytes,
        };
        run_fault_set_violation_family(&small_ring, &initial_overwrite_counts, 3);
    }
    if mode == "q3-1-s4" {
        run_fault_set_violation_family(&GEOMETRY_SMALLER_ROOT_RING, &initial_overwrite_counts, 4);
    }

    // Q1（跑前登记岔路单第 1 行，C393）：H1 家族 + Φ1 故障注入 + PC1-a/PC1-b。
    if mode == "q1-g0" {
        let summary = run_ledger_fault_family(&GEOMETRY_PRIMARY);
        emit_ledger_fault_family_summary(&GEOMETRY_PRIMARY, &summary);
        run_ledger_fault_positive_controls(&GEOMETRY_PRIMARY);
    }

    // 第八节几何敏感性（2026-09-24 session s6，岔路单第 1 行判决格 = Q1-1a 的 N_trig）：
    // S16（根环大一倍）与 S4（根环小一半）两个方向相反的取样点，复用同一套 H1 装置代码。
    if mode == "q1-s16" {
        let summary = run_ledger_fault_family(&GEOMETRY_LARGER_ROOT_RING);
        emit_ledger_fault_family_summary(&GEOMETRY_LARGER_ROOT_RING, &summary);
    }
    if mode == "q1-s4" {
        let summary = run_ledger_fault_family(&GEOMETRY_SMALLER_ROOT_RING);
        emit_ledger_fault_family_summary(&GEOMETRY_SMALLER_ROOT_RING, &summary);
    }

    // Q2-1（跑前登记岔路单第 2 行 ①，C331 修法）：H2 主族在 G0 上、n1 ∈ {0,1,2,3}、n2=1 的穷举下界。
    // 只在这一份 crates/ 上跑（同一个二进制在不同臂的副本目录里各编各的，命令与产物文件名分臂）。
    if mode == "q2-1-g0" {
        run_rootback_tolerance_family(&GEOMETRY_PRIMARY, 1, false, 0..=6, None);
    }

    // Q2-1，Φ2 加系统配置槽这一类证据（session s8，跑前登记岔路单第 2 行 ①）：只在读系统配置的
    // 臂（乙-留环、丁-留环、丁-只配置）上有意义；跑在不读系统配置的臂上也不会红，只是白扩大搜索
    // 空间，所以只在这三份副本上跑这个 mode。n1 区间可选从第二、三个命令行参数读（默认 0..=6，
    // 缺省不改变行为）——只为了能把一次挂钟预算装不下的全范围拆成几次跑，不改判据。
    if mode == "q2-1-g0-configuration-evidence" {
        let range_start = command_line_arguments
            .get(2)
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let range_end = command_line_arguments
            .get(3)
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(6);
        run_rootback_tolerance_family(&GEOMETRY_PRIMARY, 1, true, range_start..=range_end, None);
    }

    // H-C2 直接构造 + 两个负对照（session s8，Φ2 系统配置槽这一类证据的独立构造，见十三点六节）。
    if mode == "q2-1-hc2" {
        run_direct_construction_for_the_system_configuration_evidence_class(&GEOMETRY_PRIMARY);
        run_negative_control_older_slot_for_the_system_configuration_evidence_class(
            &GEOMETRY_PRIMARY,
        );
        run_negative_control_other_device_for_the_system_configuration_evidence_class(
            &GEOMETRY_PRIMARY,
        );
    }

    // Q2-2a（跑前登记岔路单第 2 行 ②）：固定脚本上这一份 crates/ 每次发布按结构种类写的字节。
    if mode == "q2-2a-g0" {
        run_fixed_publication_byte_script(&GEOMETRY_PRIMARY);
    }

    // PC2（岔路单第 2 行 ①，阳性对照，2026-09-24 session s5）：直接复现判决 K2 的两个具体构造，
    // 不靠穷举——见十三点五节顶部注释。两个函数在没有对应候选字段/改法的臂上应当报 hit=false /
    // matches_k2_table=false，这也是证据的一部分（见交回报告）。
    if mode == "q2-1-pc2" {
        run_positive_control_for_the_root_ring_txg_only_candidate(&GEOMETRY_PRIMARY);
        run_positive_control_for_the_configuration_published_txg_candidate(&GEOMETRY_PRIMARY);
    }

    // H-C1 直接构造（2026-09-24 session s6，岔路单第 2 行 ①，丙的具体历史）：判决 K2 说丙需要
    // 12 个故障（4 根槽 + 4 条记录各两块盘）才打得中；这一段不靠穷举（n1∈{4,5,6} 的完整空间太大，
    // session s9 起改按权重上限 12 穷举，见下面「H-C1 下界探针」），直接按这个具体构造跑一次，
    // 附一个只挡根槽的负对照。
    if mode == "q2-1-hc1" {
        run_direct_construction_for_the_record_scan_watermark_candidate(&GEOMETRY_PRIMARY);
        run_negative_control_root_slots_only_for_the_record_scan_watermark_candidate(
            &GEOMETRY_PRIMARY,
        );
    }

    // H-C1 下界探针：权重上限从第二个命令行参数读，缺省 `None`（按 `FEASIBLE_FULL_SEARCH_SUBSET_BUDGET`
    // 自动判断——n1=4 这个节点的完整空间只有 2^15=32768，在预算内，所以缺省就是穷举到完整空间的顶）。
    if mode == "q2-1-hc1-lower-bound" {
        let weight_ceiling = command_line_arguments
            .get(2)
            .and_then(|value| value.parse::<u64>().ok());
        run_record_scan_watermark_lower_bound_probe(&GEOMETRY_PRIMARY, weight_ceiling);
    }

    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
}

// ============================================================================
// 八、S5：装置自己的解码（checker 独立实现）与 `singlefs_core::recovery` 的输出逐条比。
// ============================================================================

fn run_independent_decode_cross_check() -> bool {
    let geometry = GEOMETRY_PRIMARY;
    let node = match bootstrap(&geometry, 2) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!("name=s5_independent_decode_cross_check verdict=fail reason=\"bootstrap 失败：{error}\""));
            return false;
        }
    };
    let devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let system_configuration = match singlefs_core::recovery::choose_system_configuration(&devices)
    {
        Ok(system_configuration) => system_configuration,
        Err(error) => {
            emit_result(&format!(
                "name=s5_independent_decode_cross_check verdict=fail reason=\"crates::recovery::choose_system_configuration 报错：{error:?}\""
            ));
            return false;
        }
    };
    let crates_readable: BTreeSet<TimelineRoot> = singlefs_core::recovery::readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root_pair(&root))
    .collect();
    let independent_geometry_view = match independent_geometry(&node.pool) {
        Ok(geometry_view) => geometry_view,
        Err(error) => {
            emit_result(&format!(
                "name=s5_independent_decode_cross_check verdict=fail reason={error:?}"
            ));
            return false;
        }
    };
    let device_readable = readable_roots_independent(&node.pool, &independent_geometry_view);
    if crates_readable == device_readable {
        emit_result(&format!(
            "name=s5_independent_decode_cross_check verdict=pass readable_root_count={} readable_roots={:?}",
            crates_readable.len(),
            crates_readable
        ));
        true
    } else {
        emit_result(&format!(
            "name=s5_independent_decode_cross_check verdict=fail crates_readable={crates_readable:?} device_readable={device_readable:?}"
        ));
        false
    }
}

// ============================================================================
// 九、5.7 常量回比 + 第七节锚点：开跑前先做，任一常量不等就停（V3）。
// ============================================================================

fn run_constants_and_anchors() -> bool {
    let mut all_ok = true;
    for check in local_constants_checks() {
        let ok = check.matches();
        all_ok &= ok;
        emit_result(&format!(
            "name=local_constant_check item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            if ok { "pass" } else { "fail" },
            check.local_value,
            check.crates_value
        ));
    }
    let (region_ok, local_regions, crates_regions) = region_devices_matches_crates();
    all_ok &= region_ok;
    emit_result(&format!(
        "name=local_constant_check item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        if region_ok { "pass" } else { "fail" }
    ));

    let arithmetic_checks: [(&str, i64, i64); 19] = [
        ("系统配置槽余量 4096-481", 4096 - 481, 3615),
        ("512-481", 512 - 481, 31),
        ("乙族字段表 481+8", 481 + 8, 489),
        ("512-489", 512 - 489, 23),
        ("4096-489", 4096 - 489, 3607),
        ("字段表四档 389+4+36+52", 389 + 4 + 36 + 52, 481),
        ("journal tail 与实例代号 469+8+4", 469 + 8 + 4, 481),
        (
            "系统配置整槽校验和偏移 4+2+96+16+20+1+4+4+8",
            4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8,
            155,
        ),
        ("实例表行宽 1+4+8+8+1+66", 1 + 4 + 8 + 8 + 1 + 66, 88),
        ("实例表一片行数 370-1", 370 - 1, 369),
        ("今天那一臂系统配置那一类写字节 4096*2*1", 4096 * 2, 8192),
        // 岔路单第 2 行 ③（2026-09-24 session s6）：四条修法与岔路 3 候选 2 抢不抢系统配置槽空间。
        // 候选 2 表形态宽度取 Q3-2 已量出的 N_w=4（S4/L=4 那一点，四个几何点里最大的一个）。
        // 甲（甲-txg、甲-jsn）与丙（今天）都不改系统配置字段表（仍是 481），与候选 2 同放时
        // 余量按「单放」算（`512 - 481 - W`，跑前登记 Q3-2 定义原句）；乙、丁都把字段表改成 489
        // （同一处 8 字节字段，只是取号时写占位 0 还是续传，写法宽度相同），余量按「与乙同放」算
        // （`512 - 481 - 8 - W`）。
        ("候选2表形态 1+16*N_w(N_w=4)", 1 + 16 * 4, 65),
        ("候选2表形态 1+12*N_w(N_w=4)", 1 + 12 * 4, 49),
        ("甲/丙同放候选2单条16 512-481-16", 512 - 481 - 16, 15),
        ("甲/丙同放候选2单条12 512-481-12", 512 - 481 - 12, 19),
        ("甲/丙同放候选2表形态65 512-481-65", 512 - 481 - 65, -34),
        ("甲/丙同放候选2表形态49 512-481-49", 512 - 481 - 49, -18),
        ("乙/丁同放候选2表形态65 512-489-65", 512 - 489 - 65, -42),
        ("乙/丁同放候选2表形态49 512-489-49", 512 - 489 - 49, -26),
    ];
    for (name, computed, expected) in arithmetic_checks {
        let ok = computed == expected;
        all_ok &= ok;
        emit_result(&format!(
            "name=section_seven_two_arithmetic item={name:?} verdict={} computed={computed} registered={expected}",
            if ok { "pass" } else { "fail" }
        ));
    }
    for slots_per_region in [4u64, 8, 16] {
        let total = 3 * slots_per_region;
        let expected = match slots_per_region {
            4 => 12,
            8 => 24,
            16 => 48,
            _ => unreachable!(),
        };
        let ok = total == expected;
        all_ok &= ok;
        emit_result(&format!(
            "name=root_ring_slot_total slots_per_region={slots_per_region} verdict={} computed={total} registered={expected}",
            if ok { "pass" } else { "fail" }
        ));
        let root_slots_with_a_root = total;
        let combinations =
            1 + root_slots_with_a_root + root_slots_with_a_root * (root_slots_with_a_root - 1) / 2;
        let expected_combinations = match slots_per_region {
            4 => 79,
            8 => 301,
            16 => 1177,
            _ => unreachable!(),
        };
        let combinations_ok = combinations == expected_combinations;
        all_ok &= combinations_ok;
        emit_result(&format!(
            "name=phi_three_fault_set_count slots_per_region={slots_per_region} root_slots_with_a_root={root_slots_with_a_root} verdict={} computed={combinations} registered={expected_combinations}",
            if combinations_ok { "pass" } else { "fail" }
        ));
    }
    for (ring_bytes, expected_records, expected_in_flight, label) in [
        (768u64 * 1024 * 1024, 196_608u64, 65_536u64, "768MiB"),
        (3 * 1024 * 1024, 768, 256, "3MiB"),
        (48 * 1024, 12, 4, "48KiB"),
    ] {
        let records = ring_bytes / 4096;
        let in_flight = records / 3;
        let ok = records == expected_records && in_flight == expected_in_flight;
        all_ok &= ok;
        emit_result(&format!(
            "name=journal_ring_records_and_in_flight_limit label={label} verdict={} computed_records={records} computed_in_flight={in_flight} registered_records={expected_records} registered_in_flight={expected_in_flight}",
            if ok { "pass" } else { "fail" }
        ));
    }

    // 写成加法不写减法（`.claude/singlefs-ai-sop/rules/test-discipline.md`「常量断言写加法，别写减法」）：
    // 减法形态 `4096 - 481 == 3615` 把常量改大会在编译期溢出，变异记成「无效」而不是「被抓到」。
    // 两边都是字面量，clippy 判「等价表达式」——这条钉的正是「这三个字面量今天互相对得上」这件事本身
    // （M6/M7 变异表改的就是这一类字面量），不是一次运行时判断，允许它。
    #[allow(
        clippy::eq_op,
        reason = "两边都是字面量，钉的是这三个字面量互相对得上，不是运行时判断"
    )]
    let section_seven_one_anchor_holds = 4096 == 481 + 3615;
    all_ok &= section_seven_one_anchor_holds;
    emit_result(&format!(
        "name=section_seven_one_anchor item=\"D22 已定项 9：系统配置字段表合计 481 字节、槽内余 3615、今天没跨 512\" verdict={} checked_in_this_segment=true",
        if section_seven_one_anchor_holds { "pass" } else { "fail" }
    ));
    emit_result("name=section_seven_one_anchor item=\"C332 正文：2 个故障\" verdict=deferred checked_in_this_segment=false deferred_to=Q3-1_第二段");
    emit_result("name=section_seven_one_anchor item=\"C393 正文：被抛弃根的账读不出时今天只计数、不拒绝挂载\" verdict=deferred checked_in_this_segment=false deferred_to=Q1-1c_第四段");
    emit_result("name=section_seven_one_anchor item=\"D23 已定项 14：N_switch=3\" verdict=deferred checked_in_this_segment=false deferred_to=第五段_前置未满足");
    emit_result("name=section_seven_one_anchor item=\"C334 判别力自证：3/3/各1\" verdict=deferred checked_in_this_segment=false deferred_to=第五段_前置未满足");
    emit_result("name=section_seven_three_row item=\"PC2/PC1-a/PC1-b/Q1-3\" verdict=deferred checked_in_this_segment=false deferred_to=第三_第四段");

    all_ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_core::address::SlotNumber;
    use singlefs_core::pointer::NodePointer;
    use singlefs_core::unit::build_index_node;

    /// 每块盘世代号最大的那一槽系统配置在盘上的偏移（槽字节按装置自己的 `newest_system_configuration_slot_bytes` 挑，再认它在两槽里的哪一槽）。
    pub(crate) fn newest_system_configuration_slot_offset(
        pool: &MemoryPool,
        device: DeviceIdentity,
    ) -> u64 {
        let slot_spacing = u64::from(
            parameters_for(&GEOMETRY_PRIMARY)
                .geometry
                .fixed_structure_slot_spacing,
        );
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
        let newest = newest_system_configuration_slot_bytes(pool, device, slot_spacing)
            .expect("每块盘都有自证的系统配置槽");
        [0u64, slot_spacing]
            .into_iter()
            .find(|offset| pool.read(device.0, *offset, slot_bytes).as_ref() == Some(&newest))
            .expect("最新那一槽在两槽之一")
    }

    /// 崩溃恢复抛弃最新那条根、而系统配置没见证到它（C554 乙合进 `crates` 之后还造得出被抛弃根的只剩这一形：见证在时可写挂载
    /// 重读一次仍读不出就拒可写，`MountError::NewerStateStillUnreadableAfterOneReread`）。做法同 `tests/common/mod.rs`
    /// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：被测那次调用里每块盘世代号最大的那一槽系统配置读回全 0，
    /// 这次挂载的取号写落回这一槽之后照常读得出。只接在落点表末尾，前面各项的下标不变。
    fn with_the_witness_unreadable(
        pool: &MemoryPool,
        mut ranges: Vec<UnreadableRange>,
    ) -> Vec<UnreadableRange> {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            ranges.push(UnreadableRange {
                device,
                offset: newest_system_configuration_slot_offset(pool, device),
                length: SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
                form: UnreadableForm::ReadsZeros,
                label: format!("newest_system_configuration(device={})", device.0),
            });
        }
        ranges
    }

    /// PC-Nw：手写一段假账（两次回退，两段被抛弃根都读得出），不碰 `crates/`，只证计数函数会数到 2。
    #[test]
    fn pc_nw_counts_two_live_rollback_events_from_a_hand_written_ledger() {
        let events = vec![
            RollbackEvent {
                abandoned: BTreeSet::from([(1u32, 5u64), (1, 6)]),
            },
            RollbackEvent {
                abandoned: BTreeSet::from([(2u32, 9u64)]),
            },
        ];
        let readable: BTreeSet<TimelineRoot> = BTreeSet::from([(1, 6), (2, 9), (3, 20)]);
        let (events_live, roots_live) = count_live_rollback_events(&events, &readable);
        assert_eq!(
            events_live, 2,
            "两次回退各有至少一个被抛弃根仍可读，应数出 2"
        );
        assert_eq!(
            roots_live, 2,
            "读得出的被抛弃根去重后是 (1,6) 与 (2,9) 两条"
        );
    }

    /// 对照：一次回退的被抛弃根全部不可读时不计入 events_live。
    #[test]
    fn pc_nw_does_not_count_a_rollback_event_whose_abandoned_roots_are_all_unreadable() {
        let events = vec![RollbackEvent {
            abandoned: BTreeSet::from([(1u32, 5u64)]),
        }];
        let readable: BTreeSet<TimelineRoot> = BTreeSet::from([(9, 99)]);
        let (events_live, roots_live) = count_live_rollback_events(&events, &readable);
        assert_eq!(events_live, 0);
        assert_eq!(roots_live, 0);
    }

    /// Q1（岔路单第 1 行）：`fault_targets_for` 的三种非空严重度各自选对了 (设备, 偏移)。
    #[test]
    fn fault_targets_for_both_copies_hits_both_devices() {
        let locations = [
            LocationEntry {
                device: DeviceIdentity(0),
                slot: SlotNumber(10),
                unit_checksum: 0,
            },
            LocationEntry {
                device: DeviceIdentity(1),
                slot: SlotNumber(20),
                unit_checksum: 0,
            },
        ];
        let both = fault_targets_for(&locations, FaultSeverity::Both);
        assert_eq!(
            both,
            vec![
                (DeviceIdentity(0), SlotNumber(10).to_device_offset()),
                (DeviceIdentity(1), SlotNumber(20).to_device_offset()),
            ]
        );
        let disk0 = fault_targets_for(&locations, FaultSeverity::Disk0);
        assert_eq!(
            disk0,
            vec![(DeviceIdentity(0), SlotNumber(10).to_device_offset())]
        );
        let disk1 = fault_targets_for(&locations, FaultSeverity::Disk1);
        assert_eq!(
            disk1,
            vec![(DeviceIdentity(1), SlotNumber(20).to_device_offset())]
        );
        let empty = fault_targets_for(&locations, FaultSeverity::Empty);
        assert!(empty.is_empty(), "空集严重度不该注入任何落点");
    }

    /// Φ1 的故障数（5.1 原文：两份都读失败 = 2 个故障，只一份 = 1 个，空集 = 0）。
    #[test]
    fn fault_severity_fault_count_matches_the_prereg_table() {
        assert_eq!(FaultSeverity::Empty.fault_count(), 0);
        assert_eq!(FaultSeverity::Both.fault_count(), 2);
        assert_eq!(FaultSeverity::Disk0.fault_count(), 1);
        assert_eq!(FaultSeverity::Disk1.fault_count(), 1);
    }

    /// op1 第三种变体（session s10）：`floor_targets_between` 从 F+1 起，含上限本身。
    #[test]
    fn floor_targets_between_starts_strictly_above_the_current_floor() {
        let targets = floor_targets_between(CheckpointTxg(2), CheckpointTxg(4));
        assert_eq!(targets, vec![CheckpointTxg(3), CheckpointTxg(4)]);
    }

    /// 同上：上限本身也是一个合法目标（半开区间是 (F, 上限]，不是 (F, 上限)）。
    #[test]
    fn floor_targets_between_includes_the_ceiling_itself() {
        let targets = floor_targets_between(CheckpointTxg(2), CheckpointTxg(4));
        assert!(
            targets.contains(&CheckpointTxg(4)),
            "上限本身也是一个合法目标，得到 {targets:?}"
        );
    }

    /// 上限不高于现行 F 时没有余量可抬——`..=` 起点大于终点天然给出空区间，不用另写判空分支。
    #[test]
    fn floor_targets_between_is_empty_when_the_ceiling_does_not_exceed_the_current_floor() {
        assert!(floor_targets_between(CheckpointTxg(5), CheckpointTxg(5)).is_empty());
        assert!(floor_targets_between(CheckpointTxg(5), CheckpointTxg(3)).is_empty());
    }

    /// op1 第三种变体：探测出的 floor 目标落在 [F+1, 上限] 内时，不注入任何故障应当成功
    /// （`error_member` 为 `None`）；换成对着某条被抛弃可读根的分配记录树节点注入故障（两份都读失败），
    /// `raise_rollback_floor` 必须做成、且它交回的 `RaisedFloor::abandoned_roots_unreadable` > 0——证明抬 F 那一步的影子账真的读到了
    /// 这组故障（不是被前面那一步「先不注入的 mount_writable」悄悄吸收掉，也不是抬 F 算上限那一步先拒了）。
    /// 被抛弃根取自第 2 次跑那一节 H1c 的抛弃步（`abandonment_step_nodes`；跑前登记 `research/prompts/e158-r2-prereg.md`
    /// 第一节处理表：故障改打 H1c 里一条被抛弃根的分配记录树节点、删掉「或者报了错也算」那一半）。
    #[test]
    fn mount_writable_then_raise_floor_reaches_the_injected_fault() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let fixed_geometry = fixed_geometry_for(&GEOMETRY_PRIMARY);
        let mut tested = false;
        for (pool, abandoned) in abandonment_step_nodes() {
            let Ok(pool_geometry) = independent_geometry(&pool) else {
                continue;
            };
            let readable = readable_roots_independent(&pool, &pool_geometry);
            let Some(&abandoned_root) = abandoned.iter().find(|root| readable.contains(root))
            else {
                continue;
            };
            let Some(record) = root_record_of(&pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let plain_devices = devices_from_pool(&pool, IMAGE_BYTES);
            let Ok(allocation_locations) = locate_allocation_record_tree(&plain_devices, &record)
            else {
                continue;
            };
            let node = SimNode {
                pool: pool.clone(),
                session: None,
                timeline: Vec::new(),
                rollback_events: Vec::new(),
                path: Vec::new(),
                content_by_root: BTreeMap::new(),
            };
            let targets = raise_floor_targets_for(&node, &parameters);
            let Some(&floor) = targets.first() else {
                continue;
            };
            let kind = FaultedOperationKind::MountWritableThenRaiseFloorTo(floor);
            let without_fault =
                attempt_faulted_operation(&node, &parameters, fixed_geometry, &kind, &[]);
            assert!(
                without_fault.error_member.is_none(),
                "探测出的 floor 目标不注入任何故障应当成功，得到 {:?}",
                without_fault.error_member
            );
            let fault_targets = fault_targets_for(&allocation_locations, FaultSeverity::Both);
            let with_fault = attempt_faulted_operation(
                &node,
                &parameters,
                fixed_geometry,
                &kind,
                &fault_targets,
            );
            assert!(
                with_fault.error_member.is_none(),
                "被抛弃根的分配记录树节点两份都读失败时抬 F 照样做成（账读不出只计数），得到 {:?}",
                with_fault.error_member
            );
            assert!(
                with_fault
                    .abandoned_roots_unreadable
                    .is_some_and(|count| count > 0),
                "抬 F 重算影子账要数到这条读不出账的被抛弃根，得到 {:?}（不注入时 {:?}）",
                with_fault.abandoned_roots_unreadable,
                without_fault.abandoned_roots_unreadable
            );
            tested = true;
            break;
        }
        assert!(
            tested,
            "H1c 抛弃步造出的节点里应当至少有一个有 op1 第三种变体可跑（有余量、有可读被抛弃根）"
        );
    }

    /// `error_member_of_debug`：跨臂只用字符串比较（A1 副本新变体在「今天」那份编译单元里
    /// 根本不存在这个枚举成员，不能直接 `match` 枚举名），首词切分要认得三种括号/空格分隔形态。
    #[test]
    fn error_member_of_debug_extracts_the_variant_name() {
        assert_eq!(
            error_member_of_debug("AbandonedRootLedgerUnreadable { count: 3 }"),
            "AbandonedRootLedgerUnreadable"
        );
        assert_eq!(
            error_member_of_debug("RollbackTargetNotACandidate { target: X, exclusion: Y }"),
            "RollbackTargetNotACandidate"
        );
        assert_eq!(error_member_of_debug("Recovery(NoValidRoot)"), "Recovery");
        assert_eq!(
            error_member_of_debug("FileVersionWithoutAnyJournalRecord"),
            "FileVersionWithoutAnyJournalRecord"
        );
    }

    /// `location_key`：去重键只看设备号与槽号，校验和字段不参与比较。
    #[test]
    fn location_key_ignores_the_checksum_field() {
        let first_copy = LocationEntry {
            device: DeviceIdentity(1),
            slot: SlotNumber(7),
            unit_checksum: 111,
        };
        let second_copy = LocationEntry {
            device: DeviceIdentity(1),
            slot: SlotNumber(7),
            unit_checksum: 222,
        };
        assert_eq!(location_key(&first_copy), location_key(&second_copy));
        assert_eq!(location_key(&first_copy), (1u32, 7u64));
    }

    /// M6（变异表）的反面对照：本地常量与 `crates::` 的默认值今天必须全部相等（回归锚，
    /// `crates/` 改了这些常量而没同步登记时，这条测试先红）。
    #[test]
    fn local_constants_match_crates_today() {
        for check in local_constants_checks() {
            assert!(
                check.matches(),
                "{} 本地={} crates={} 不等",
                check.name,
                check.local_value,
                check.crates_value
            );
        }
        let (region_ok, local, crates_value) = region_devices_matches_crates();
        assert!(
            region_ok,
            "三个区域住哪块盘：本地={local:?} crates={crates_value:?} 不等"
        );
    }

    /// 5.7 常量回比 + 第七节 7.2 算术全过（M6/M7 变异的红测：改坏本地常量或算术字面量，这条测试先红）。
    #[test]
    fn constants_and_anchors_all_pass() {
        assert!(
            run_constants_and_anchors(),
            "run_constants_and_anchors 应当全 PASS（本地常量与 crates:: 同名常量、第七节 7.2 算术都对得上）"
        );
    }

    /// bootstrap 之后装置自记的时间线与 checker 独立解出的可读根集合逐条一致（S5 的最小单测版本）。
    #[test]
    fn bootstrap_timeline_matches_independent_decode() {
        let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let readable = readable_roots_independent(&node.pool, &geometry_view);
        let timeline_set: BTreeSet<TimelineRoot> = node.timeline.iter().copied().collect();
        assert_eq!(
            readable, timeline_set,
            "不注入故障时，装置自记的时间线应当与根环里全部自证根的集合相同"
        );
    }

    /// 候选 (b) 分配记录树指称「从内容树反推占用集合」：第一个文件版本上，中央映射树给出的落点
    /// 集合非空（至少覆盖那次发布写的数据单元）；mkfs 那一版（`mapping_root` 恒
    /// `NodePointer::empty_root()`）读不出——是报错，不是当空集读到。
    #[test]
    fn allocation_record_tree_reachable_placements_via_central_mapping_reflects_written_content_and_errors_on_an_empty_pointer(
    ) {
        let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let newest = readable_roots_independent(&node.pool, &geometry_view)
            .into_iter()
            .max()
            .expect("bootstrap 之后至少有一条根");
        let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
        let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
        let placements = allocation_record_tree_reachable_placements_via_central_mapping(
            &plain_devices,
            &record,
        )
        .expect("第一个文件版本的中央映射树应当读得出");
        assert!(
            !placements.is_empty(),
            "第一个文件版本至少写了一个数据单元，中央映射树给出的落点集合不该是空集"
        );

        let genesis_record = RootRecord {
            mapping_root: NodePointer::empty_root(),
            ..record
        };
        assert!(
            allocation_record_tree_reachable_placements_via_central_mapping(
                &plain_devices,
                &genesis_record,
            )
            .is_err(),
            "mapping_root 是空指针时应当报错，不是读成空集"
        );
    }

    /// 候选 (b) 分配记录树指称「从内容树反推占用集合」走全（session s8）：走全版本必须是只走中央映射
    /// 版本的超集——至少多出树表自己的节点与中央映射树自己的节点这两个位置；空指针那一版同样报错
    /// （走全版本第一步就调用只走中央映射的版本）。
    #[test]
    fn allocation_record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_only_version(
    ) {
        let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let newest = readable_roots_independent(&node.pool, &geometry_view)
            .into_iter()
            .max()
            .expect("bootstrap 之后至少有一条根");
        let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
        let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
        let central_mapping_only = allocation_record_tree_reachable_placements_via_central_mapping(
            &plain_devices,
            &record,
        )
        .expect("第一个文件版本的中央映射树应当读得出");
        let all_structural_trees =
            allocation_record_tree_reachable_placements_via_all_structural_trees(
                &plain_devices,
                &record,
            )
            .expect("走全版本在同一段历史上也应当读得出");
        assert!(
            all_structural_trees.is_superset(&central_mapping_only),
            "走全版本必须包含只走中央映射版本给出的每一个落点"
        );
        let mapping_root_own_node = (
            record.mapping_root.locations[0].device.0,
            record.mapping_root.locations[0].slot.0,
        );
        let tree_table_own_node = (
            record.tree_table.locations[0].device.0,
            record.tree_table.locations[0].slot.0,
        );
        assert!(
            all_structural_trees.contains(&mapping_root_own_node),
            "走全版本要包含中央映射树自己的节点，不只是它指向的数据单元"
        );
        assert!(
            all_structural_trees.contains(&tree_table_own_node),
            "走全版本要包含树表自己的节点"
        );

        let genesis_record = RootRecord {
            mapping_root: NodePointer::empty_root(),
            ..record
        };
        assert!(
            allocation_record_tree_reachable_placements_via_all_structural_trees(
                &plain_devices,
                &genesis_record,
            )
            .is_err(),
            "mapping_root 是空指针时走全版本也应当报错（它第一步就调用只走中央映射的版本）"
        );
    }

    /// session s9 根因排查②：171 格 `subset_of_pristine` 差集里的两个 (设备, 槽) 就是实例表自己的
    /// 物理节点（`RootRecord::instance_table`），不是 LIVELIST/SPARSE_SIDE_TABLE/DEADLIST 的节点、
    /// 也不是分配记录树自己的节点——`classify_diff_pair` 要能分辨这三种、且对查无归属的落点如实报告。
    #[test]
    fn classify_diff_pair_identifies_the_instance_table_self_node() {
        let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let newest = readable_roots_independent(&node.pool, &geometry_view)
            .into_iter()
            .max()
            .expect("bootstrap 之后至少有一条根");
        let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
        let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
        let raw_records = allocation_records_under_root(&plain_devices, &record)
            .expect("这段历史的分配记录树读得出");
        let allocation_locations = locate_allocation_record_tree(&plain_devices, &record).ok();

        let instance_table_pair = (
            record.instance_table.locations[0].device.0,
            record.instance_table.locations[0].slot.0,
        );
        assert_eq!(
            classify_diff_pair(
                &plain_devices,
                &record,
                allocation_locations,
                &raw_records,
                instance_table_pair,
            ),
            "instance_table_self_node_not_walked_by_the_all_structural_trees_arm",
            "实例表自己的节点要被认出来，不能落进查无归属的兜底分支"
        );

        let allocation_record_tree_pair = (
            allocation_locations.expect("这段历史的分配记录树有物理位置")[0]
                .device
                .0,
            allocation_locations.expect("同上")[0].slot.0,
        );
        assert_eq!(
            classify_diff_pair(
                &plain_devices,
                &record,
                allocation_locations,
                &raw_records,
                allocation_record_tree_pair,
            ),
            "allocation_record_tree_self_node",
            "分配记录树自己的节点不能被误判成实例表"
        );

        assert_eq!(
            classify_diff_pair(
                &plain_devices,
                &record,
                allocation_locations,
                &raw_records,
                (99, 999_999),
            ),
            "not_found_in_raw_allocation_records",
            "查无归属的落点要如实报告，不能编一个假分类"
        );
    }

    /// 候选 (b) 树表指称「crates 有路」子分支要用的量：同一实例上更旧的可读根，取的是
    /// checkpoint_txg 最大的那一条祖先；不许把自己选成祖先，也不许漏掉更接近目标的候选。
    #[test]
    fn nearest_older_readable_root_on_same_instance_picks_the_closest_ancestor_on_the_same_instance(
    ) {
        let node = bootstrap(&GEOMETRY_PRIMARY, 2).expect("bootstrap 应当成功（含 2 次覆盖写）");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let readable = readable_roots_independent(&node.pool, &geometry_view);
        let newest = *readable.iter().max().expect("bootstrap 之后至少有一条根");
        let target = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
        let (older_root, older_record) = nearest_older_readable_root_on_same_instance(
            &node.pool,
            &geometry_view,
            newest,
            &target,
        )
        .expect("同一实例上还有更旧的可读根（暖机与首个文件那几条）");
        assert_eq!(
            older_root.0, target.instance.0,
            "找到的祖先要与目标同一实例"
        );
        assert!(
            older_record.checkpoint_txg < target.checkpoint_txg,
            "找到的祖先 checkpoint_txg 要严格小于目标"
        );
        let closer_candidate_exists = readable.iter().any(|candidate| {
            candidate.0 == target.instance.0
                && *candidate != older_root
                && candidate.1 > older_root.1
                && candidate.1 < target.checkpoint_txg.0
        });
        assert!(
            !closer_candidate_exists,
            "同一实例上存在 txg 更接近目标、却没被选中的候选"
        );
    }

    /// `system_configuration_slot_positions`：槽 0 恒偏移 0，槽 1 偏移 = 槽距；两块盘各自独立。
    #[test]
    fn system_configuration_slot_positions_uses_slot_spacing_for_the_second_slot_on_each_device() {
        assert_eq!(
            system_configuration_slot_positions(4096),
            [(0, 0, 0), (0, 1, 4096), (1, 0, 0), (1, 1, 4096)]
        );
        assert_eq!(
            system_configuration_slot_positions(512),
            [(0, 0, 0), (0, 1, 512), (1, 0, 0), (1, 1, 512)]
        );
    }

    /// `evidence_items_at`（session s8）：`include_system_configuration_slots` 关着时一条
    /// `SystemConfigurationSlot` 证据项都不出现（既有六份副本用的搜索不受影响）；打开时两块盘各自
    /// 两个自证过的槽都进来，标签按 (设备, 槽号) 区分。
    #[test]
    fn evidence_items_at_includes_system_configuration_slots_only_when_asked() {
        let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let without_configuration = evidence_items_at(&node.pool, &geometry_view, false);
        assert!(
            without_configuration
                .iter()
                .all(|item| item.kind != EvidenceKind::SystemConfigurationSlot),
            "开关关着时不该有系统配置槽这一类证据项"
        );
        let with_configuration = evidence_items_at(&node.pool, &geometry_view, true);
        let configuration_labels: BTreeSet<String> = with_configuration
            .iter()
            .filter(|item| item.kind == EvidenceKind::SystemConfigurationSlot)
            .map(|item| item.label.clone())
            .collect();
        assert_eq!(
            configuration_labels,
            BTreeSet::from([
                "configuration(device=0,slot=0)".to_string(),
                "configuration(device=0,slot=1)".to_string(),
                "configuration(device=1,slot=0)".to_string(),
                "configuration(device=1,slot=1)".to_string(),
            ]),
            "mkfs 加暖机加首个文件加一次覆盖写之后，两块盘的两个槽都该已经轮换过、都自证得过"
        );
    }

    /// `candidate_fault_sets_of_weight`：根槽与系统配置槽都是权重 1，合并成一批喂给这个函数之后，
    /// 权重 2 的子集要既有「两个根槽」「两个配置槽」，也要有「一个根槽 + 一个配置槽」这种跨类组合，
    /// 不能只按证据项个数枚举、漏掉跨类的那一种。
    #[test]
    fn candidate_fault_sets_of_weight_mixes_root_and_configuration_slots_by_weight() {
        let weight_one_items = vec![
            evidence_item(EvidenceKind::RootRingSlot, "root_a", 1),
            evidence_item(EvidenceKind::SystemConfigurationSlot, "configuration_a", 1),
        ];
        let journal_record_items = Vec::new();
        let candidates =
            candidate_fault_sets_of_weight(&weight_one_items, &journal_record_items, 2);
        let labels: BTreeSet<Vec<String>> = candidates
            .iter()
            .map(|combination| combination.iter().map(|item| item.label.clone()).collect())
            .collect();
        assert_eq!(
            labels,
            BTreeSet::from([vec!["root_a".to_string(), "configuration_a".to_string()]]),
            "两个权重 1 的证据项只有一种取两个的组合：跨类那一对"
        );
    }

    /// H-C2 机制假说的最小复现，不依赖任何臂改法：只把设备 0 上世代号更高的系统配置槽读失败，
    /// `singlefs_core::recovery::choose_system_configuration` 交回的 `slot_generation` 应当退回
    /// 设备 0 自己更旧的那一份，不看设备 1（`recovery.rs` 第 516 行起，`chosen` 只在 `None` 时
    /// 赋值一次）。这条测试不碰任何臂改法，验证的是「今天」这份 `choose_system_configuration`
    /// 本身的行为——H-C2 三个驱动函数（十三点六节）依赖的正是这个行为。
    #[test]
    fn faulting_the_newer_device_zero_system_configuration_slot_regresses_the_chosen_generation() {
        let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let (newest_slot_index, newest_generation) =
            newest_self_certified_system_configuration_slot_on_device_zero(
                &node.pool,
                geometry_view.slot_spacing,
            )
            .expect("这段历史上设备 0 应当至少有一个自证过的系统配置槽");
        let (_, oldest_generation) =
            oldest_self_certified_system_configuration_slot_on_device_zero(
                &node.pool,
                geometry_view.slot_spacing,
            )
            .expect("这段历史上设备 0 的两个槽应当都自证过（多次轮换之后）");
        assert!(
            oldest_generation < newest_generation,
            "两槽世代号应当不同：oldest={oldest_generation} newest={newest_generation}"
        );

        let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
        let baseline = choose_system_configuration(&plain).expect("不注入时应当读得出系统配置");
        assert_eq!(
            baseline.quantities.slot_generation, newest_generation,
            "不注入时 chosen 应当是当下最新那一份"
        );

        let offset = newest_slot_index * geometry_view.slot_spacing;
        let mut wrapped: Vec<(DeviceIdentity, MultiOffsetReadFailingBlockDevice)> = Vec::new();
        for (identity, device) in plain {
            let failing_offsets: BTreeSet<u64> = if identity == DeviceIdentity(0) {
                BTreeSet::from([offset])
            } else {
                BTreeSet::new()
            };
            wrapped.push((
                identity,
                MultiOffsetReadFailingBlockDevice {
                    inner: device,
                    failing_offsets,
                },
            ));
        }
        let faulted = choose_system_configuration(&wrapped).expect("设备 0 的另一槽仍自证得过");
        assert_eq!(
            faulted.quantities.slot_generation, oldest_generation,
            "设备 0 的新槽读失败之后，chosen 应当退回设备 0 自己更旧的那一份；\
             读成 {newest_generation}（未受影响的设备 1 的值）就说明选择逻辑其实看了别的盘"
        );
    }

    /// Q3-1 用的 P332 辅助函数：只测纯逻辑，`pool` 填空壳（这几条断言不碰盘上字节）。
    fn bare_node(
        timeline: Vec<TimelineRoot>,
        content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>,
        rollback_events: Vec<RollbackEvent>,
    ) -> SimNode {
        SimNode {
            pool: MemoryPool {
                devices: std::collections::BTreeMap::new(),
                device_size_in_bytes: 0,
            },
            session: None,
            timeline,
            rollback_events,
            path: Vec::new(),
            content_by_root,
        }
    }

    /// V2：内容号匹配当前时间线上某个根 ⇒ 不违例，即便同一份内容也出现在被抛弃的根上。
    #[test]
    fn content_is_off_the_current_timeline_is_false_when_the_content_matches_a_root_still_on_the_timeline(
    ) {
        let mut content_by_root = BTreeMap::new();
        content_by_root.insert((1, 1), Some(b"a".to_vec()));
        content_by_root.insert((1, 2), Some(b"b".to_vec()));
        content_by_root.insert((2, 5), Some(b"a".to_vec())); // 被抛弃的根，内容号与 (1,1) 撞了
        let node = bare_node(vec![(1, 1), (1, 2)], content_by_root, Vec::new());
        assert!(
            !content_is_off_the_current_timeline(&node, &Some(b"a".to_vec())),
            "内容 a 匹配时间线上的 (1,1)，不该判违例"
        );
    }

    /// V2：内容号只匹配被抛弃的根、不匹配当前时间线上任何一个根 ⇒ 违例。
    #[test]
    fn content_is_off_the_current_timeline_is_true_when_the_content_only_belongs_to_an_abandoned_root(
    ) {
        let mut content_by_root = BTreeMap::new();
        content_by_root.insert((1, 1), Some(b"a".to_vec()));
        content_by_root.insert((2, 5), Some(b"z".to_vec())); // 被抛弃、不在 timeline 上
        let node = bare_node(vec![(1, 1)], content_by_root, Vec::new());
        assert!(
            content_is_off_the_current_timeline(&node, &Some(b"z".to_vec())),
            "内容 z 只在被抛弃的 (2,5) 上出现过，应当判违例"
        );
    }

    /// V2 的 `None`（没有文件）签名同样按内容号处理：时间线上有一个根也是「没有文件」才不算违例。
    #[test]
    fn content_is_off_the_current_timeline_treats_no_file_as_a_content_signature_too() {
        let mut content_by_root = BTreeMap::new();
        content_by_root.insert((0, 0), None);
        content_by_root.insert((1, 1), Some(b"a".to_vec()));
        let node_with_no_file_on_timeline =
            bare_node(vec![(0, 0)], content_by_root.clone(), Vec::new());
        assert!(!content_is_off_the_current_timeline(
            &node_with_no_file_on_timeline,
            &None
        ));
        let node_without_no_file_on_timeline = bare_node(vec![(1, 1)], content_by_root, Vec::new());
        assert!(content_is_off_the_current_timeline(
            &node_without_no_file_on_timeline,
            &None
        ));
    }

    /// V1：`is_abandoned` 要在**全部**回退事件的抛弃集合里查，不是只查最近一次。
    #[test]
    fn is_abandoned_checks_every_recorded_rollback_event_not_only_the_latest() {
        let node = bare_node(
            vec![(3, 30)],
            BTreeMap::new(),
            vec![
                RollbackEvent {
                    abandoned: BTreeSet::from([(1, 5), (1, 6)]),
                },
                RollbackEvent {
                    abandoned: BTreeSet::from([(2, 9)]),
                },
            ],
        );
        assert!(is_abandoned(&node, (1, 5)), "第一次回退抛弃的根也要算");
        assert!(is_abandoned(&node, (2, 9)), "第二次回退抛弃的根要算");
        assert!(
            !is_abandoned(&node, (3, 30)),
            "当前时间线尾不该被判成被抛弃"
        );
    }

    /// P332：`NoFile` 结局恒不判 V2（没有内容号可比），V1 照判。
    #[test]
    fn violation_at_checkpoint_never_flags_content_off_timeline_on_a_no_file_outcome() {
        let node = bare_node(
            vec![(1, 1)],
            BTreeMap::new(),
            vec![RollbackEvent {
                abandoned: BTreeSet::from([(1, 9)]),
            }],
        );
        let outcome = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(9)),
        };
        let point = violation_at_checkpoint(&node, &outcome).expect("NoFile 有所选根，判得出 P332");
        assert!(
            point.chosen_root_abandoned,
            "所选根 (1,9) 在被抛弃集合里，V1 该为真"
        );
        assert!(!point.content_off_timeline, "NoFile 没有内容号，V2 恒为假");
    }

    /// P332：`Failed` 结局没有所选根，交回 `None`（这一处不判 P332，另计入失败计数，不当违例）。
    #[test]
    fn violation_at_checkpoint_returns_none_on_a_failed_outcome() {
        let node = bare_node(vec![(1, 1)], BTreeMap::new(), Vec::new());
        let outcome = RecoveryOutcome::Failed {
            root: None,
            failure: singlefs_core::recovery::RecoveryFailure::NoValidRoot,
        };
        assert!(violation_at_checkpoint(&node, &outcome).is_none());
    }

    /// `combinations_of_size`：从 3 个里取 2 个应当恰好 3 种，取 0 个是空集合本身，取超过总数是空列表。
    #[test]
    fn combinations_of_size_enumerates_every_subset_of_the_requested_size() {
        let items = vec![1, 2, 3];
        let pairs = combinations_of_size(&items, 2);
        assert_eq!(pairs, vec![vec![1, 2], vec![1, 3], vec![2, 3]]);
        assert_eq!(combinations_of_size(&items, 0), vec![Vec::<i32>::new()]);
        assert_eq!(combinations_of_size(&items, 4), Vec::<Vec<i32>>::new());
    }

    fn evidence_item(kind: EvidenceKind, label: &str, target_count: usize) -> EvidenceItem {
        EvidenceItem {
            kind,
            label: label.to_string(),
            targets: (0..target_count)
                .map(|index| {
                    (
                        DeviceIdentity(0),
                        DeviceOffsetInBytes(u64::try_from(index).expect("测试里的下标很小")),
                    )
                })
                .collect(),
        }
    }

    /// Q2-1 的权重子集枚举：根槽权重 1、记录权重 2；权重 2 的子集应当既有「两个根槽」也有
    /// 「一条记录」，不能只按证据项个数枚举（那样会漏掉「一条记录」这一类、多出「两条记录」这一类）。
    #[test]
    fn candidate_fault_sets_of_weight_mixes_root_slots_and_journal_records_by_weight() {
        let root_slot_items = vec![
            evidence_item(EvidenceKind::RootRingSlot, "root_a", 1),
            evidence_item(EvidenceKind::RootRingSlot, "root_b", 1),
        ];
        let journal_record_items = vec![evidence_item(EvidenceKind::JournalRecord, "record_a", 2)];
        let candidates = candidate_fault_sets_of_weight(&root_slot_items, &journal_record_items, 2);
        let labels: BTreeSet<Vec<String>> = candidates
            .iter()
            .map(|combination| combination.iter().map(|item| item.label.clone()).collect())
            .collect();
        assert_eq!(
            labels,
            BTreeSet::from([
                vec!["root_a".to_string(), "root_b".to_string()],
                vec!["record_a".to_string()],
            ]),
            "权重 2 只能是两个根槽，或一条记录（权重 2），不能是别的组合"
        );
    }

    /// 权重 0 只有空集合这一种，权重超过全部证据项权重之和时枚举不出任何子集。
    #[test]
    fn candidate_fault_sets_of_weight_at_zero_is_only_the_empty_set() {
        let root_slot_items = vec![evidence_item(EvidenceKind::RootRingSlot, "root_a", 1)];
        let journal_record_items = Vec::new();
        assert_eq!(
            candidate_fault_sets_of_weight(&root_slot_items, &journal_record_items, 0),
            vec![Vec::new()]
        );
        assert_eq!(
            candidate_fault_sets_of_weight(&root_slot_items, &journal_record_items, 5),
            Vec::<Vec<EvidenceItem>>::new()
        );
    }

    fn timeline_with_one_root() -> Vec<TimelineRoot> {
        vec![(1, 3)]
    }

    /// P331：所选根不在时间线上记「倒挂」，不看内容对不对。
    #[test]
    fn judge_recovery_outcome_flags_a_root_off_the_timeline_as_rootback_before_checking_content() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(9), CheckpointTxg(9)),
            content: vec![1, 2, 3],
        };
        let (passed, detail) = judge_recovery_outcome(
            "check",
            &outcome,
            &timeline_with_one_root(),
            &Some(vec![1, 2, 3]),
        );
        assert!(!passed, "所选根 (9,9) 不在时间线 [(1,3)] 上，必须判失败");
        assert!(
            detail.contains("倒挂"),
            "失败理由要写明是倒挂，实得：{detail}"
        );
    }

    /// P331：所选根在时间线上、但读回内容对不上时间线末端记「读不回」。
    #[test]
    fn judge_recovery_outcome_flags_wrong_content_as_unreadable() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: vec![9, 9, 9],
        };
        let (passed, detail) = judge_recovery_outcome(
            "check",
            &outcome,
            &timeline_with_one_root(),
            &Some(vec![1, 2, 3]),
        );
        assert!(!passed, "内容对不上时间线末端的期望内容，必须判失败");
        assert!(
            detail.contains("读不回"),
            "失败理由要写明是读不回，实得：{detail}"
        );
    }

    /// P331：所选根在时间线上、内容也对得上，判通过。
    #[test]
    fn judge_recovery_outcome_passes_when_root_on_timeline_and_content_matches() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: vec![1, 2, 3],
        };
        let (passed, _) = judge_recovery_outcome(
            "check",
            &outcome,
            &timeline_with_one_root(),
            &Some(vec![1, 2, 3]),
        );
        assert!(passed, "所选根在时间线上、内容也对得上，应当判通过");
    }

    /// P331：没有文件的一版，期望内容是 `None`，`NoFile` 判通过。
    #[test]
    fn judge_recovery_outcome_passes_on_no_file_when_no_file_is_expected() {
        let outcome = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
        };
        let (passed, _) =
            judge_recovery_outcome("check", &outcome, &timeline_with_one_root(), &None);
        assert!(passed, "没有文件的一版，期望内容也是 None，应当判通过");
    }

    /// PC2：环境变量没设，本地常量按今天那一份取 489（D22（单元原子性怎么合成） 已定项 9 的字段表合计）。
    #[test]
    fn parse_local_system_configuration_bytes_defaults_to_489_when_unset() {
        let value = parse_local_system_configuration_bytes(Err(env::VarError::NotPresent));
        assert_eq!(value, 489, "没设环境变量应当取今天的 489");
    }

    /// PC2：环境变量给了十进制数字，原样解出来。取一个与没设时的默认 489 不同的数，
    /// 才分得出是解出来的，还是退回了默认值。
    #[test]
    fn parse_local_system_configuration_bytes_parses_the_given_decimal() {
        let value = parse_local_system_configuration_bytes(Ok("497".to_string()));
        assert_eq!(value, 497, "环境变量给的十进制数要原样解出来");
    }

    /// PC2：环境变量给了不是十进制数的内容，panic（不许静默退回默认值掩盖配错）。
    #[test]
    #[should_panic(expected = "不是十进制正整数")]
    fn parse_local_system_configuration_bytes_panics_on_non_decimal_value() {
        let _ = parse_local_system_configuration_bytes(Ok("not-a-number".to_string()));
    }

    /// PC2 排查出的根因：`MultiOffsetReadFailingBlockDevice` 一次记住多个精确偏移，
    /// 每一个都要让读失败——不能像旧写法那样只认第一个。
    #[test]
    fn multi_offset_read_failing_block_device_fails_every_named_offset() {
        let inner = SparseBlockDevice::new(1 << 20, PhysicalBlockSizeInBytes(512));
        let device = MultiOffsetReadFailingBlockDevice {
            inner,
            failing_offsets: BTreeSet::from([512u64, 4096u64, 8192u64]),
        };
        let mut buffer = [0u8; 512];
        for offset in [512u64, 4096, 8192] {
            assert!(
                device
                    .read_at(DeviceOffsetInBytes(offset), &mut buffer)
                    .is_err(),
                "点名的偏移 {offset} 应当每一个都读失败，不能只有第一个"
            );
        }
    }

    /// 对照：没被点名的偏移原样读得通（读失败不是把整块盘弄坏）。
    #[test]
    fn multi_offset_read_failing_block_device_passes_through_offsets_not_named() {
        let inner = SparseBlockDevice::new(1 << 20, PhysicalBlockSizeInBytes(512));
        let device = MultiOffsetReadFailingBlockDevice {
            inner,
            failing_offsets: BTreeSet::from([512u64]),
        };
        let mut buffer = [0u8; 512];
        assert!(
            device
                .read_at(DeviceOffsetInBytes(16384), &mut buffer)
                .is_ok(),
            "没被点名的偏移不应该被这份包装挡下来"
        );
    }

    /// PC2 甲：`root_ring_slot_targets_for` 按 (实例, txg) 精确点名，不多不少。
    #[test]
    fn root_ring_slot_targets_for_finds_exactly_the_named_roots() {
        let node = bootstrap(&GEOMETRY_PRIMARY, 4).expect("bootstrap 应当成功");
        let geometry_view =
            independent_geometry(&node.pool).expect("已经过 mkfs 的池应当解得出几何");
        let topmost_two: BTreeSet<TimelineRoot> =
            node.timeline.iter().rev().take(2).copied().collect();
        let targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_two);
        assert_eq!(
            targets.len(),
            2,
            "点名两条根，每条第一版几何下只住一块盘，应当解出两个 (设备, 偏移)"
        );
        let mut deduplicated: BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)> = BTreeSet::new();
        for target in &targets {
            deduplicated.insert(*target);
        }
        assert_eq!(deduplicated.len(), 2, "两条根不应该解到同一个 (设备, 偏移)");
    }

    /// session s9（实十九提醒，2026-09-24 16:50 UTC 前后落地）：中央映射树条目数超过单节点叶容量
    /// （`transaction.rs` 第 1334 行「中央映射树叶 294」）会长成多层，根节点变成内部节点
    /// （`level > 0`）。这里手工构造一个合法但 `level=1` 的假节点，覆盖真实中央映射树根的两份物理
    /// 拷贝，验证 `allocation_record_tree_reachable_placements_via_central_mapping` 显式拦下它——
    /// 不是靠 `parse_mapping_entry` 的宽度检查侥幸拦住：内部条目宽约 114 字节（`transaction.rs`
    /// 同一行「内部 143」按 16384 / 143 反推），比 `MAPPING_ENTRY_BYTES`=55 宽，那条宽度检查拦不住它。
    #[test]
    fn allocation_record_tree_reachable_placements_via_central_mapping_rejects_a_multi_level_root()
    {
        let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
        let newest = readable_roots_independent(&node.pool, &geometry_view)
            .into_iter()
            .max()
            .expect("bootstrap 之后至少有一条根");
        let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
        let mut plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);

        let real_bytes = read_node_bytes(&plain_devices, &record.mapping_root.locations)
            .expect("真实的中央映射树根读得出");
        let real_header = parse_index_node(&real_bytes).expect("真实的中央映射树根解得开");
        assert_eq!(real_header.level, 0, "bootstrap 之后中央映射树根应当仍是叶");

        // 内部条目宽的估计值：只要比 MAPPING_ENTRY_BYTES=55 宽，就足够触发要测的那条路
        // （宽度检查拦不住，必须靠显式的 level 检查）。
        let fake_entry_width = 114u16;
        let fake_bytes = build_index_node(
            real_header.tree,
            1,
            real_header.key_width,
            &real_header.smallest_key,
            &real_header.largest_key,
            real_header.birth_txg,
            &FILESYSTEM_IDENTIFIER,
            real_header.instance,
            real_header.birth_sequence,
            fake_entry_width,
            &[vec![0u8; usize::from(fake_entry_width)]],
        );

        for location in &record.mapping_root.locations {
            let (_, device) = plain_devices
                .iter_mut()
                .find(|(identity, _)| *identity == location.device)
                .expect("这块盘在池里");
            device
                .write_at(
                    location.slot.to_device_offset(),
                    &fake_bytes,
                    WriteDurability::Plain,
                )
                .expect("写入假节点应当成功");
        }

        let outcome = allocation_record_tree_reachable_placements_via_central_mapping(
            &plain_devices,
            &record,
        );
        let error = outcome.expect_err("level=1 的根节点必须被拒绝，不能被当成叶解出假的落点集合");
        assert!(
            error.contains("level=1"),
            "错误信息要点名是哪个 level 拦下的，实际: {error}"
        );
    }

    /// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 1 行还差项④）：持续故障与瞬时故障
    /// 在 op1 起的三次挂载轨迹上必须不同——持续（`persistent=true`）故障不撤，每一步都该继续触发；
    /// 瞬时（`persistent=false`）故障只在 op1 那一步有效，撤掉之后的两步该恢复成不触发。用 PC1-a 的挑选条件（按固定次序找第一个满足
    /// 『某条被抛弃根的分配记录树节点不共享』的历史）选落点；被抛弃根取自第 2 次跑那一节 H1c 的抛弃步
    /// （`abandonment_step_nodes`；跑前登记 `research/prompts/e158-r2-prereg.md` 第一节处理表：第一次跑靠挂载时回退造的
    /// 被抛弃根在今天的代码上造不出来，这条单测开工时就红）。
    #[test]
    fn mount_writable_trajectory_distinguishes_persistent_from_transient_faults() {
        let geometry = &GEOMETRY_PRIMARY;
        let parameters = parameters_for(geometry);
        let fixed_geometry = fixed_geometry_for(geometry);
        for (pool, abandoned) in abandonment_step_nodes() {
            let Ok(pool_geometry) = independent_geometry(&pool) else {
                continue;
            };
            let readable = readable_roots_independent(&pool, &pool_geometry);
            let abandoned_readable: Vec<TimelineRoot> = abandoned
                .iter()
                .copied()
                .filter(|root| readable.contains(root))
                .collect();
            let plain_devices = devices_from_pool(&pool, IMAGE_BYTES);

            for &abandoned_root in &abandoned_readable {
                let Some(record) = root_record_of(&pool, &pool_geometry, abandoned_root) else {
                    continue;
                };
                let Ok(allocation_locations) =
                    locate_allocation_record_tree(&plain_devices, &record)
                else {
                    continue;
                };
                let allocation_keys: BTreeSet<(u32, u64)> =
                    allocation_locations.iter().map(location_key).collect();
                if is_unit_shared(
                    &plain_devices,
                    &pool,
                    &pool_geometry,
                    abandoned_root,
                    &allocation_keys,
                ) {
                    continue;
                }

                let allocation_fault_targets =
                    fault_targets_for(&allocation_locations, FaultSeverity::Both);
                let persistent = mount_writable_trajectory(
                    &pool,
                    &parameters,
                    fixed_geometry,
                    &allocation_fault_targets,
                    true,
                    3,
                );
                let transient = mount_writable_trajectory(
                    &pool,
                    &parameters,
                    fixed_geometry,
                    &allocation_fault_targets,
                    false,
                    3,
                );
                assert_eq!(persistent.len(), 3, "轨迹要报满 3 步（跑前登记 8.1 原文）");
                assert_eq!(transient.len(), 3);
                assert_eq!(
                    persistent[0], transient[0],
                    "op1 这一步两种模式的故障目标相同，第一步的结局必须相同"
                );
                assert!(
                    persistent[0].is_some_and(|count| count > 0),
                    "PC1-a 已经证明这一格会触发，第一步的计数必须 > 0"
                );
                assert_eq!(
                    persistent[1],
                    persistent[0],
                    "持续故障不撤，第二步该继续触发（这一格是单一故障不共享的落点，不随挂载次数变化）"
                );
                assert_eq!(persistent[2], persistent[0], "持续故障第三步同理");
                assert_eq!(
                    transient[1],
                    Some(0),
                    "瞬时故障撤掉之后，第二步该恢复成不触发"
                );
                assert_eq!(transient[2], Some(0), "瞬时故障第三步同理");
                return;
            }
        }
        panic!(
            "H1c 抛弃步造出的节点里应当至少有一格满足 PC1-a 的挑选条件（跑前登记 5.2 阳性对照）"
        );
    }

    /// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②）：权重上限机制——
    /// 显式传 `Some(0)` 时只穷举权重 0 那一档就停（不管这一档打不打中），`stopped_by_weight_ceiling`
    /// 如实反映「还有没搜到的档」；`full_space_subset_count` 恒等于完整证据空间的子集数，不随上限变，
    /// 且不管停在哪一档，`subsets_tried` 都不超过它——这三条合起来就是「完整空间多大、穷举到权重几、
    /// 没有截断」要报的东西。
    #[test]
    fn search_minimum_weight_that_triggers_rootback_honors_an_explicit_weight_ceiling() {
        let geometry = &GEOMETRY_PRIMARY;
        let parameters = parameters_for(geometry);
        let node = bootstrap(geometry, 0).expect("bootstrap 应当成功");
        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");

        let full = search_minimum_weight_that_triggers_rootback(
            &node,
            &parameters,
            &geometry_view,
            1,
            None,
            false,
        );
        assert!(
            full.full_space_subset_count > 1,
            "这段历史至少有一个证据项，完整空间子集数要大于 1"
        );
        assert!(
            full.subsets_tried <= full.full_space_subset_count,
            "枚举过的子集数不能超过完整空间"
        );

        let capped = search_minimum_weight_that_triggers_rootback(
            &node,
            &parameters,
            &geometry_view,
            1,
            Some(0),
            false,
        );
        assert_eq!(
            capped.full_space_subset_count, full.full_space_subset_count,
            "权重上限不改变完整空间的大小，只改变搜到哪一档就停"
        );
        assert_eq!(
            capped.highest_weight_examined, 0,
            "传 Some(0) 时只穷举权重 0 那一档"
        );
        if capped.weight_at_first_hit.is_none() {
            assert!(
                capped.stopped_by_weight_ceiling,
                "权重 0 没打中、且完整空间的最高权重大于 0 时，必须标『因权重上限而停』"
            );
        }
        assert!(
            capped.subsets_tried <= full.subsets_tried,
            "权重上限更紧时枚举过的子集数不该比不设上限时更多"
        );
    }

    // ------------------------------------------------------------------------
    // 第 2 次跑（`research/prompts/e158-r2-prereg.md`）那一节的单测。
    // ------------------------------------------------------------------------

    fn device_with_bytes(identity: DeviceIdentity, offset: u64, bytes: &[u8]) -> SparseBlockDevice {
        let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
        device
            .write_at(DeviceOffsetInBytes(offset), bytes, WriteDurability::Plain)
            .expect("内存盘写不报错");
        let _ = identity;
        device
    }

    fn instrumented(
        identity: DeviceIdentity,
        inner: SparseBlockDevice,
        state: &Rc<RefCell<InstrumentationState>>,
    ) -> InstrumentedDevice {
        InstrumentedDevice {
            identity,
            inner,
            state: Rc::clone(state),
        }
    }

    /// 前 n 个写之后的镜像要用的最小历史：mkfs → 可写挂载 → 首个文件 → n1 次覆盖写 → 关闭 C。
    fn closed_history(overwrites: u64) -> SecondRunHistory {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut history = second_run_start(&GEOMETRY_PRIMARY, overwrites).expect("历史走得通");
        second_run_close(
            &mut history,
            &parameters,
            Closing::ProcessExit,
            "test",
            "test",
        )
        .expect("关闭 C");
        history
    }

    #[test]
    fn unique_offset_of_finds_a_single_occurrence_and_refuses_two() {
        assert_eq!(unique_offset_of(&[0, 1, 2, 3, 4], &[2, 3]), Some(2));
        assert_eq!(unique_offset_of(&[2, 3, 0, 2, 3], &[2, 3]), None);
        assert_eq!(unique_offset_of(&[0, 1], &[7]), None);
    }

    /// 跑前登记 5.6 表「journal tail 偏移、F 偏移」：装置自己解码找到 469 与 481（实例代号在 477）。
    #[test]
    fn decoded_offsets_put_the_tail_at_469_the_instance_at_477_and_the_floor_at_481() {
        let (tail, instance, floor) =
            decoded_tail_instance_and_floor_offsets().expect("三项都恰好一处");
        assert_eq!(tail, 469);
        assert_eq!(instance, 469 + 8);
        assert_eq!(floor, 481);
    }

    /// 7.2 的锚点在今天的字段表（489）与乙 / 丁的字段表（497）上都过；字段表写错一字节（488）就有不过的。
    #[test]
    fn second_run_arithmetic_anchors_hold_for_489_and_for_497_and_not_for_488() {
        for layout in [489u64, 497] {
            for (item, computed, registered) in second_run_arithmetic_anchors(layout) {
                assert_eq!(computed, registered, "{layout}：{item}");
            }
        }
        assert!(second_run_arithmetic_anchors(488)
            .iter()
            .any(|(_, computed, registered)| computed != registered));
        assert_eq!(SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL, 489);
    }

    /// 读落点：读错那一类读就报错、清零那一类读回 0，两类都按区间罩（读的起点不必等于落点起点），落点外的读原样；
    /// 每个落点被拦下几次单独记；这次调用自己写到落点上之后不再拦。
    #[test]
    fn instrumented_device_fails_or_zeroes_only_reads_over_the_named_ranges_and_counts_them() {
        let identity = DeviceIdentity(0);
        let inner = device_with_bytes(identity, 8192, &[7u8; 4096]);
        let state = Rc::new(RefCell::new(InstrumentationState::new(
            vec![
                UnreadableRange {
                    device: identity,
                    offset: 8192,
                    length: 512,
                    form: UnreadableForm::ReadFails,
                    label: "fails".to_string(),
                },
                UnreadableRange {
                    device: identity,
                    offset: 9216,
                    length: 512,
                    form: UnreadableForm::ReadsZeros,
                    label: "zeros".to_string(),
                },
            ],
            None,
            None,
        )));
        let mut device = instrumented(identity, inner, &state);
        let mut buffer = vec![0u8; 512];
        assert!(
            device
                .read_at(DeviceOffsetInBytes(7936), &mut buffer)
                .is_err(),
            "起点在落点之前、区间罩到落点的读也要报错"
        );
        let mut wide = vec![0u8; 2048];
        assert!(device.read_at(DeviceOffsetInBytes(8704), &mut wide).is_ok());
        assert!(wide[..512].iter().all(|byte| *byte == 7));
        assert!(
            wide[512..1024].iter().all(|byte| *byte == 0),
            "清零那一段读回 0"
        );
        assert!(wide[1024..1536].iter().all(|byte| *byte == 7));
        let mut outside = vec![0u8; 512];
        assert!(device
            .read_at(DeviceOffsetInBytes(12288), &mut outside)
            .is_ok());
        assert_eq!(state.borrow().intercepted_reads, vec![1, 1]);
        device
            .write_at(
                DeviceOffsetInBytes(8192),
                &[9u8; 512],
                WriteDurability::Plain,
            )
            .expect("写");
        assert!(
            device
                .read_at(DeviceOffsetInBytes(8192), &mut buffer)
                .is_ok(),
            "写过之后不再拦"
        );
        assert!(buffer.iter().all(|byte| *byte == 9));
        assert_eq!(state.borrow().intercepted_reads, vec![1, 1]);
    }

    /// 注入写失败：整池第 n 次写（从 0 数）报错、不落盘、不进落了的写的记录；第 n 次屏障同理。
    #[test]
    fn instrumented_device_fails_exactly_the_named_write_and_barrier_calls() {
        let identity = DeviceIdentity(0);
        let state = Rc::new(RefCell::new(InstrumentationState::new(
            Vec::new(),
            Some(1),
            Some(0),
        )));
        let mut device = instrumented(
            identity,
            SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            &state,
        );
        assert!(device
            .write_at(DeviceOffsetInBytes(0), &[1u8; 512], WriteDurability::Plain)
            .is_ok());
        assert!(device
            .write_at(
                DeviceOffsetInBytes(512),
                &[2u8; 512],
                WriteDurability::Plain
            )
            .is_err());
        assert!(device
            .write_at(
                DeviceOffsetInBytes(1024),
                &[3u8; 512],
                WriteDurability::Plain
            )
            .is_ok());
        assert!(device.barrier().is_err());
        assert!(device.barrier().is_ok());
        let state = state.borrow();
        assert_eq!(state.write_calls, 3);
        assert_eq!(state.barrier_calls, 2);
        assert!(state.injected_failure_fired);
        let written_offsets: Vec<u64> = state
            .applied
            .iter()
            .filter_map(|step| match step {
                AppliedDeviceStep::Write { offset, .. } => Some(*offset),
                AppliedDeviceStep::ZeroFill { .. } | AppliedDeviceStep::Barrier { .. } => None,
            })
            .collect();
        assert_eq!(written_offsets, vec![0, 1024]);
        let mut buffer = vec![0u8; 512];
        device
            .inner
            .read_at(DeviceOffsetInBytes(512), &mut buffer)
            .expect("读");
        assert!(buffer.iter().all(|byte| *byte == 0), "报了错的那次写不落盘");
    }

    fn write_step(device: u32, offset: u64, fill: u8) -> AppliedDeviceStep {
        AppliedDeviceStep::Write {
            device: DeviceIdentity(device),
            offset,
            bytes: vec![fill; 512],
        }
    }

    /// 崩溃状态（前缀）：只施加前 n 个写，屏障不算写、也不改字节。
    #[test]
    fn image_with_the_first_writes_applies_only_the_prefix() {
        let base = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
        let steps = vec![
            write_step(0, 0, 1),
            AppliedDeviceStep::Barrier {
                device: DeviceIdentity(0),
            },
            write_step(1, 512, 2),
            write_step(0, 1024, 3),
        ];
        let image = image_with_the_first_writes(&base, &steps, 2);
        assert_eq!(image.read(0, 0, 512), Some(vec![1; 512]));
        assert_eq!(image.read(1, 512, 512), Some(vec![2; 512]));
        assert_eq!(
            image.read(0, 1024, 512),
            Some(vec![0; 512]),
            "第 3 个写没施加"
        );
        let empty = image_with_the_first_writes(&base, &steps, 0);
        assert_eq!(empty.read(0, 0, 512), Some(vec![0; 512]));
    }

    /// 第一个根槽写的下标按写数（屏障不算），它之前的屏障数只数在它之前发的。
    #[test]
    fn index_of_the_first_root_slot_write_and_the_barriers_before_it() {
        let steps = vec![
            write_step(0, 4096, 1),
            AppliedDeviceStep::Barrier {
                device: DeviceIdentity(0),
            },
            write_step(1, 900_000, 2),
            AppliedDeviceStep::Barrier {
                device: DeviceIdentity(1),
            },
            write_step(1, 4_198_400, 3),
            AppliedDeviceStep::Barrier {
                device: DeviceIdentity(0),
            },
            write_step(0, 4_198_400, 4),
        ];
        let root_slots = vec![(DeviceIdentity(1), 4_198_400u64)];
        assert_eq!(
            index_of_the_first_root_slot_write(&steps, &root_slots, 512),
            Some(2)
        );
        assert_eq!(barriers_before_write(&steps, 3), 2);
        assert_eq!(barriers_before_write(&steps, 1), 0);
        let barrier_first = vec![
            AppliedDeviceStep::Barrier {
                device: DeviceIdentity(0),
            },
            write_step(0, 4096, 1),
        ];
        assert_eq!(
            barriers_before_write(&barrier_first, 0),
            0,
            "「前 0 个写」之前一个屏障都不算"
        );
        assert_eq!(
            barriers_before_write(&barrier_first, 1),
            1,
            "第 1 个写之前那一次屏障算"
        );
        assert_eq!(
            index_of_the_first_root_slot_write(&steps, &[(DeviceIdentity(0), 12)], 512),
            None
        );
    }

    /// K0–K4（前提 3 的答案）：报错的按这次调用设备一层写没写分 K3 / K4，不按错误成员分（登记第九节 M17）；做成的按准入那一栏分。
    #[test]
    fn mount_outcome_class_splits_refusals_by_the_writes_of_the_call() {
        let refused: Result<Mounted, singlefs_core::mount::MountError> =
            Err(singlefs_core::mount::MountError::InstanceTableMalformed);
        assert_eq!(mount_outcome_class(&refused, 0), "K3");
        assert_eq!(mount_outcome_class(&refused, 1), "K4");
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut devices = new_devices(IMAGE_BYTES);
        make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mounted = mount_writable(&parameters, &mut devices);
        assert_eq!(mount_outcome_class(&mounted, 30), "K0");
    }

    /// 抛弃步的落点：最新 k 条根的根槽（一块盘上一个物理块）+ 断链发布的记录两份 / 第一个点名单元两份 / 它的第一份。
    #[test]
    fn unreadable_ranges_of_the_abandonment_name_the_root_slots_and_the_break() {
        let history = closed_history(1);
        let geometry = independent_geometry(&history.pool).expect("几何");
        let tip = history.tip();
        let (tip_device, tip_offset) =
            root_slot_of(&history.pool, &geometry, tip).expect("最新根的根槽");
        let records = records_of_publish(&history.pool, &geometry, tip);
        assert!(!records.is_empty());
        let every_record = unreadable_ranges_of_the_abandonment(
            &history,
            &geometry,
            1,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
        )
        .expect("落点")
        .expect("有记录");
        assert_eq!(every_record.len(), 1 + 2 * records.len());
        assert_eq!(
            (
                every_record[0].device,
                every_record[0].offset,
                every_record[0].length
            ),
            (tip_device, tip_offset, 512)
        );
        assert!(every_record[1..]
            .iter()
            .all(|range| range.length == JOURNAL_RECORD_BYTES));
        assert!(every_record[1..]
            .iter()
            .any(|range| range.device == DeviceIdentity(1)));
        let both_copies = unreadable_ranges_of_the_abandonment(
            &history,
            &geometry,
            1,
            UnreadableForm::ReadsZeros,
            ReplayBreak::FirstNamedUnit,
        )
        .expect("落点")
        .expect("有点名单元");
        assert_eq!(both_copies.len(), 3);
        assert_ne!(both_copies[1].device, both_copies[2].device);
        let first_copy = unreadable_ranges_of_the_abandonment(
            &history,
            &geometry,
            1,
            UnreadableForm::ReadsZeros,
            ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        )
        .expect("落点")
        .expect("有点名单元");
        assert_eq!(first_copy, both_copies[..2].to_vec());
        let two_roots = unreadable_ranges_of_the_abandonment(
            &history,
            &geometry,
            2,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
        )
        .expect("落点")
        .expect("有记录");
        assert_eq!(
            two_roots.len(),
            2 + 2 * records_of_publish(
                &history.pool,
                &geometry,
                history.timeline[history.timeline.len() - 2]
            )
            .len()
        );
    }

    /// 第 2 次跑修订第 1 条的依据：重放验点名单元要全部份都验过，第一份读错就停，第二份一次都没读——b = unit 的第二份落点拦不到（V2 作废），
    /// 只坏第一份就断得了链（b = unit_first_copy）。
    #[test]
    fn replay_stops_at_the_first_copy_of_a_named_unit_so_the_second_copy_is_never_read() {
        let history = closed_history(1);
        let geometry = independent_geometry(&history.pool).expect("几何");
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let both = with_the_witness_unreadable(
            &history.pool,
            unreadable_ranges_of_the_abandonment(
                &history,
                &geometry,
                1,
                UnreadableForm::ReadFails,
                ReplayBreak::FirstNamedUnit,
            )
            .expect("落点")
            .expect("有点名单元"),
        );
        let run = run_instrumented_mount_writable(&history.pool, &parameters, &both, None, None);
        assert!(run.intercepted_reads[0] > 0, "根槽拦到了");
        assert!(run.intercepted_reads[1] > 0, "第一份拦到了");
        assert_eq!(run.intercepted_reads[2], 0, "第二份一次都没读");
        let first = with_the_witness_unreadable(
            &history.pool,
            unreadable_ranges_of_the_abandonment(
                &history,
                &geometry,
                1,
                UnreadableForm::ReadFails,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            )
            .expect("落点")
            .expect("有点名单元"),
        );
        let first_copy_run =
            run_instrumented_mount_writable(&history.pool, &parameters, &first, None, None);
        let mounted = first_copy_run.result.as_ref().expect("挂载做成");
        assert_eq!(
            root_pair(&mounted.output.chosen_root),
            history.timeline[history.timeline.len() - 2]
        );
        assert_eq!(
            abandoned_roots_on(&first_copy_run.pool_after).expect("S5 不停"),
            BTreeSet::from([tip_of(&history)]),
            "只坏第一份，最新那条根就被抛弃"
        );
    }

    fn tip_of(history: &SecondRunHistory) -> TimelineRoot {
        history.tip()
    }

    /// 第 2 次跑修订第 2 条的依据（H1d 的 n1 从 0 起）：写行那次发布在它的根落盘之前写的单元，n1 = 0 时盖到被藏的首个文件那条根引用的单元，
    /// n1 = 1 时一处都不盖（开的是更高的新段）。
    #[test]
    fn the_row_publish_before_its_root_overwrites_the_hidden_newest_root_only_when_it_is_the_first_file(
    ) {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut overwritten_by_overwrites = Vec::new();
        for overwrites in [0u64, 1] {
            let history = closed_history(overwrites);
            let geometry = independent_geometry(&history.pool).expect("几何");
            let (_, tip_record) = landing_root_on(&history.pool).expect("择根");
            let tip_units =
                placements_referenced_by_root_on(&history.pool, &tip_record).expect("账");
            let ranges = with_the_witness_unreadable(
                &history.pool,
                unreadable_ranges_of_the_abandonment(
                    &history,
                    &geometry,
                    1,
                    UnreadableForm::ReadsZeros,
                    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
                )
                .expect("落点")
                .expect("有点名单元"),
            );
            let reference =
                run_instrumented_mount_writable(&history.pool, &parameters, &ranges, None, None);
            let first_root_write = index_of_the_first_root_slot_write(
                &reference.applied,
                &root_slot_device_offsets(&geometry),
                u64::from(geometry.physical_block_size),
            )
            .expect("写行那次发布写了根");
            let post =
                image_with_the_first_writes(&history.pool, &reference.applied, first_root_write);
            overwritten_by_overwrites.push(placements_overwritten_between(
                &history.pool,
                &post,
                &tip_units,
            ));
            assert_eq!(
                landing_root_on(&post).expect("择根").0,
                history.tip(),
                "根没落盘，恢复仍落到被藏的那条"
            );
        }
        assert!(
            overwritten_by_overwrites[0] > 0,
            "n1 = 0：{overwritten_by_overwrites:?}"
        );
        assert_eq!(
            overwritten_by_overwrites[1], 0,
            "n1 = 1：{overwritten_by_overwrites:?}"
        );
    }

    /// 7.1 第六行「今天 = 丙」：新实例第一次发布的 txg = 装置按这次读得出的根与记录算的 max + 1——不藏、藏最新根与它的记录、
    /// 只藏最新根而记录读得出三种都对得上。
    #[test]
    fn first_txg_by_the_rule_of_today_matches_the_row_publish_of_the_mount() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let history = closed_history(1);
        let geometry = independent_geometry(&history.pool).expect("几何");
        let mut observed = Vec::new();
        for ranges in [
            Vec::new(),
            with_the_witness_unreadable(
                &history.pool,
                unreadable_ranges_of_the_abandonment(
                    &history,
                    &geometry,
                    1,
                    UnreadableForm::ReadFails,
                    ReplayBreak::EveryRecord,
                )
                .expect("落点")
                .expect("有记录"),
            ),
            with_the_witness_unreadable(
                &history.pool,
                unreadable_ranges_of_the_abandonment(
                    &history,
                    &geometry,
                    1,
                    UnreadableForm::ReadFails,
                    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
                )
                .expect("落点")
                .expect("有点名单元"),
            ),
        ] {
            let run =
                run_instrumented_mount_writable(&history.pool, &parameters, &ranges, None, None);
            let row_txg = run
                .result
                .as_ref()
                .expect("挂载做成")
                .output
                .row_publish
                .root()
                .checkpoint_txg
                .0;
            assert_eq!(
                row_txg,
                first_txg_by_the_rule_of_today(&history.pool, &geometry, &ranges)
            );
            observed.push(row_txg);
        }
        let tip_txg = history.tip().1;
        assert_eq!(observed, vec![tip_txg + 1, tip_txg, tip_txg + 1]);
    }

    /// Q1-0 的最小复现（前提 1），系统配置没见证到被藏的最新根（`with_the_witness_unreadable`；C554 乙之后见证只罩最新一次发布之外的，
    /// 藏两条根时较旧的那一槽系统配置仍见证第二新那条，可写挂载照拒，这一形造不出）：藏最新那条根、只坏它第一条记录点名的第一个单元的第一份，
    /// 挂载做成、最新那条根被按新实例表判抛弃（装置与 crates 一致）；断它自己的全部记录时，新实例第一次发布的 txg 恰与被藏那条相同、
    /// 落进同一个根槽，被藏那条就不在了（抛弃集合为空）。
    #[test]
    fn abandonment_step_abandons_the_hidden_newest_root_unless_the_row_publish_lands_on_its_slot() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let history = closed_history(1);
        let geometry = independent_geometry(&history.pool).expect("几何");
        let unit_broken = with_the_witness_unreadable(
            &history.pool,
            unreadable_ranges_of_the_abandonment(
                &history,
                &geometry,
                1,
                UnreadableForm::ReadFails,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            )
            .expect("落点")
            .expect("有点名单元"),
        );
        let run =
            run_instrumented_mount_writable(&history.pool, &parameters, &unit_broken, None, None);
        assert!(run.result.is_ok());
        assert_eq!(
            abandoned_roots_on(&run.pool_after).expect("S5 不停"),
            BTreeSet::from([history.tip()])
        );
        let one = with_the_witness_unreadable(
            &history.pool,
            unreadable_ranges_of_the_abandonment(
                &history,
                &geometry,
                1,
                UnreadableForm::ReadFails,
                ReplayBreak::EveryRecord,
            )
            .expect("落点")
            .expect("有记录"),
        );
        let one_root_run =
            run_instrumented_mount_writable(&history.pool, &parameters, &one, None, None);
        let mounted = one_root_run.result.as_ref().expect("挂载做成");
        assert_eq!(
            mounted.output.row_publish.root().checkpoint_txg.0,
            history.tip().1
        );
        assert!(abandoned_roots_on(&one_root_run.pool_after)
            .expect("S5 不停")
            .is_empty());
    }

    /// 关闭 U（登记第九节 M16 的反面）：正常卸载把每块盘最新系统配置里的 F 抬到卸载之前现行那一版的 txg，推 2 或 3 次空发布（7.1 第四行）。
    #[test]
    fn normal_unmount_raises_the_floor_to_the_current_txg_and_pushes_the_unmount_publishes() {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut history = second_run_start(&GEOMETRY_PRIMARY, 1).expect("历史");
        let before = history.timeline.len();
        let before_txg = history.tip().1;
        second_run_close(
            &mut history,
            &parameters,
            Closing::NormalUnmount,
            "test",
            "test",
        )
        .expect("卸载");
        let pushed = history.timeline.len() - before;
        assert_eq!(pushed, if before_txg % 3 == 1 { 3 } else { 2 });
        let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
        assert_eq!(
            highest_floor_in_the_system_configurations(&history.pool, slot_spacing),
            before_txg
        );
        assert!(history.session.is_none());
    }

    /// 两条沿用的单测从 H1c 抛弃步造出的节点里挑被抛弃根（跑前登记第一节处理表：「改从 H1c 的 PC1-a 构造取（同一个落点选法）」）。
    /// 固定次序与挑法同装置的 `abandonment_step_node_list`（n1、c1、b；挂载做成、落点全拦到、按新实例表判出至少一条被抛弃的根），
    /// 只是系统配置没见证到被藏的根（`with_the_witness_unreadable`）、k 只取 1：C554 乙合进 `crates` 之后，见证在时可写挂载拒，
    /// 藏两条以上时较旧那一槽仍见证被藏的根，造不出被抛弃的根。n1 取到 6：k = 1 只抛弃一条，抬 F 的上限要第 4 新的非空有效根
    /// （D16（发布语义） 已定项 1），n1 = 1–3 的节点上上限等于现行 F、抬 F 那一种 op1 没有目标（装置的节点表靠 k = 4 那几格才有目标）。
    fn abandonment_step_nodes() -> Vec<(MemoryPool, BTreeSet<TimelineRoot>)> {
        let parameters = parameters_for(&GEOMETRY_PRIMARY);
        let mut nodes = Vec::new();
        for overwrites in 1u64..=6 {
            for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
                let mut history = second_run_start(&GEOMETRY_PRIMARY, overwrites).expect("历史");
                second_run_close(&mut history, &parameters, closing, "node-list", "node-list")
                    .expect("关闭");
                let pool_geometry = independent_geometry(&history.pool).expect("几何");
                for replay_break in [
                    ReplayBreak::EveryRecord,
                    ReplayBreak::FirstNamedUnit,
                    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
                ] {
                    let Ok(Some(ranges)) = unreadable_ranges_of_the_abandonment(
                        &history,
                        &pool_geometry,
                        1,
                        UnreadableForm::ReadFails,
                        replay_break,
                    ) else {
                        continue;
                    };
                    let ranges = with_the_witness_unreadable(&history.pool, ranges);
                    let run = run_instrumented_mount_writable(
                        &history.pool,
                        &parameters,
                        &ranges,
                        None,
                        None,
                    );
                    if run.result.is_err()
                        || !unintercepted_ranges(&ranges, &run.intercepted_reads).is_empty()
                    {
                        continue;
                    }
                    let Ok(abandoned) = abandoned_roots_on(&run.pool_after) else {
                        continue;
                    };
                    if !abandoned.is_empty() {
                        nodes.push((run.pool_after, abandoned));
                    }
                }
            }
        }
        nodes
    }
}
