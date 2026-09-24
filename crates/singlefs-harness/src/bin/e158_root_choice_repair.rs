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
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_rollback, mount_writable, RollbackTarget, ShadowLedger};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::{TreeTableEntry, TREE_KIND_ALLOCATION};
use singlefs_core::recovery::{
    allocation_records_under_root, recover, tree_table_has_no_entries, JournalPolicy,
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
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::fault_injection::{
    injected_block_device_error, FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice,
    FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, NamedRootRingSlots,
    PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_harness::segments::FixedGeometry;

/// Q2-1 的穷举下界搜索，一个模板至多枚举的子集数（跑前登记 5.1 写的上限是 2¹⁶；这一次因挂钟预算改
/// 用一个更小的数——见交回报告「它答不了的」，这不是判据的修订）。
const SUBSET_ENUMERATION_CAP: u64 = 20_000;

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
/// 环境变量 `SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES` 给了就用它，没给按今天 / 甲-txg 那一份
/// 取 481——两者都没改这张字段表。装置源码各臂同一份（跑前登记「装置写在哪」），差别只在跑的时候
/// 传不传这个环境变量，不是另分支代码。
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
        Err(env::VarError::NotPresent) => 481,
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
enum FaultedOperationKind {
    MountWritable,
    MountRollbackTo(TimelineRoot),
}

impl FaultedOperationKind {
    fn label(&self) -> String {
        match self {
            FaultedOperationKind::MountWritable => "mount_writable".to_string(),
            FaultedOperationKind::MountRollbackTo(target) => format!("mount_rollback({target:?})"),
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
    let outcome = match kind {
        FaultedOperationKind::MountWritable => mount_writable(parameters, &mut wrapped)
            .map(|mounted| mounted.output.abandoned_roots_unreadable),
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
    persistence_cost_values: BTreeSet<u64>,
    rejected_zero_device_bytes: u64,
    rejected_nonzero_device_bytes: u64,
    faulted_operation_kinds_tried: BTreeSet<String>,
}

/// 跑前登记 5.1 Φ1 + 六、报哪些量：H1 全族 × 每条被抛弃可读根 × 2 个单元 × 3 个严重度（只做「瞬时」，
/// 「持续」——op1 及其后两次挂载——这一段没有实现，见交回报告）。`is_a1_arm` 只影响 Q1-4/Q1-1c 的
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
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvidenceKind {
    RootRingSlot,
    JournalRecord,
}

/// Φ2 的一个证据项：根环槽权重恒 1（根环每个区域第一版只住一块盘，D22（单元原子性怎么合成） 已定项 16
/// 第 4 条）；journal 记录的权重 = 它当下自证过的份数（`targets.len()`，不注入时通常 2）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct EvidenceItem {
    kind: EvidenceKind,
    label: String,
    targets: Vec<(DeviceIdentity, DeviceOffsetInBytes)>,
}

/// op2 那一刻不注入的孪生镜像上的证据项：根环里每个存着自证根的槽，加上 journal 环里每一条至少一份
/// 自证的记录。系统配置槽这一类跳过（见本节顶部说明）。
fn evidence_items_at(pool: &MemoryPool, geometry: &PoolGeometry) -> Vec<EvidenceItem> {
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
    items
}

/// 权重恰为 `weight` 的证据项子集：根槽（权重 1）取 `root_count` 个、记录（权重 2，见
/// `evidence_items_at` 里 `targets` 恒两份）取 `record_count` 个，`root_count + 2 * record_count
/// = weight`，逐个 `record_count` 值把两边的组合叉乘。
fn candidate_fault_sets_of_weight(
    root_slot_items: &[EvidenceItem],
    journal_record_items: &[EvidenceItem],
    weight: u64,
) -> Vec<Vec<EvidenceItem>> {
    let mut candidates = Vec::new();
    let mut record_count = 0u64;
    while record_count * 2 <= weight {
        let root_count = weight - record_count * 2;
        if let (Ok(root_count), Ok(record_count_usize)) =
            (usize::try_from(root_count), usize::try_from(record_count))
        {
            if root_count <= root_slot_items.len()
                && record_count_usize <= journal_record_items.len()
            {
                for root_combination in combinations_of_size(root_slot_items, root_count) {
                    for record_combination in
                        combinations_of_size(journal_record_items, record_count_usize)
                    {
                        let mut combination = root_combination.clone();
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
        _ => (
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
                        other => (
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
    subsets_tried: u64,
    capped_without_a_hit: bool,
    rootback_probe_failures: u64,
    overwrite_failures: u64,
    /// 这个模板 Φ2（根槽 + journal 记录两类）的证据项权重之和：`weight_at_first_hit` 为
    /// `None` 且 `capped_without_a_hit` 为假时，说明穷举已经走完全部权重都没有打中——
    /// 这个数就是「这个模板最多能凑出多大的故障集合」，交回报告解释「没打中」是不是因为
    /// 证据本来就不够多。
    maximum_weight: u64,
}

/// Q2-1 的穷举下界：按总权重从小到大枚举 Φ2 子集，同权重内全枚举，打中就停；一个模板至多枚举
/// `SUBSET_ENUMERATION_CAP` 个子集。
fn search_minimum_weight_that_triggers_rootback(
    node_before_rootback_probe: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry_view: &PoolGeometry,
    overwrites_after_the_rootback_probe: u64,
    subset_enumeration_cap: u64,
) -> RootbackToleranceSearchOutcome {
    let items = evidence_items_at(&node_before_rootback_probe.pool, geometry_view);
    let root_slot_items: Vec<EvidenceItem> = items
        .iter()
        .filter(|item| item.kind == EvidenceKind::RootRingSlot)
        .cloned()
        .collect();
    let journal_record_items: Vec<EvidenceItem> = items
        .iter()
        .filter(|item| item.kind == EvidenceKind::JournalRecord)
        .cloned()
        .collect();
    let maximum_weight = u64::try_from(root_slot_items.len()).expect("证据项数装得进 u64")
        + 2 * u64::try_from(journal_record_items.len()).expect("证据项数装得进 u64");

    let mut subsets_tried = 0u64;
    let mut rootback_probe_failures = 0u64;
    let mut overwrite_failures = 0u64;
    let mut weight = 0u64;
    loop {
        if weight > maximum_weight {
            return RootbackToleranceSearchOutcome {
                weight_at_first_hit: None,
                hit_count_at_that_weight: 0,
                first_hit_combination_labels: Vec::new(),
                subsets_tried,
                capped_without_a_hit: false,
                rootback_probe_failures,
                overwrite_failures,
                maximum_weight,
            };
        }
        let candidates =
            candidate_fault_sets_of_weight(&root_slot_items, &journal_record_items, weight);
        let mut hit_count = 0u64;
        let mut first_hit_combination_labels: Vec<String> = Vec::new();
        for combination in &candidates {
            if subsets_tried >= subset_enumeration_cap {
                return RootbackToleranceSearchOutcome {
                    weight_at_first_hit: None,
                    hit_count_at_that_weight: 0,
                    first_hit_combination_labels: Vec::new(),
                    subsets_tried,
                    capped_without_a_hit: true,
                    rootback_probe_failures,
                    overwrite_failures,
                    maximum_weight,
                };
            }
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
                capped_without_a_hit: false,
                rootback_probe_failures,
                overwrite_failures,
                maximum_weight,
            };
        }
        weight += 1;
    }
}

/// H2 主族（跑前登记 5.1）：mkfs → `mount_writable`（实例 1）→ 首个文件 → n1 次覆盖写 → 关闭 →
/// op2 = `mount_writable`（受注入）→ 成功则 n2 次覆盖写 → 关闭 → 判 P331。n1 ∈ {0,...,6}（登记要求
/// 的全范围，2026-09-24 session s5 从 {0,1,2,3} 补齐——上一段只跑到 3 是挂钟预算，不是构造上限；
/// 另加 H2-R 的 `mount_rollback` 变体与 H2c 崩溃档，这两个仍没做，见交回报告「它答不了的」）。
fn run_rootback_tolerance_family(geometry: &Geometry, overwrites_after_the_rootback_probe: u64) {
    let parameters = parameters_for(geometry);
    for overwrites_before_the_rootback_probe in 0u64..=6 {
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
            SUBSET_ENUMERATION_CAP,
        );
        match outcome.weight_at_first_hit {
            Some(weight) => emit_result(&format!(
                "name=q2_1_minimum_weight geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min={weight} hit_count_at_that_weight={} first_hit_combination={:?} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label,
                outcome.hit_count_at_that_weight,
                outcome.first_hit_combination_labels,
                outcome.subsets_tried,
                outcome.rootback_probe_failures,
                outcome.overwrite_failures
            )),
            None if outcome.capped_without_a_hit => emit_result(&format!(
                "name=q2_1_minimum_weight geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min=capped subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
            )),
            None => emit_result(&format!(
                "name=q2_1_minimum_weight geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min=none_found_within_full_evidence_space maximum_weight_available={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label, outcome.maximum_weight, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
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
        Ok(mounted) => mounted,
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

    let mut allocator = rollback_mounted.allocator;
    let PoolVersion::WithFile(previous) = &rollback_mounted.current else {
        emit_result("name=q2_2a_final_overwrite_failed reason=\"回退之后现行版本没有文件\"");
        return;
    };
    let content = deterministic_content(2000, OVERWRITE_FILE_BYTES);
    let output = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        match publish_overwrite(
            &mut writer,
            &mut allocator,
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

fn main() {
    let command_line_arguments: Vec<String> = env::args().collect();
    let mode = command_line_arguments
        .get(1)
        .map(String::as_str)
        .unwrap_or("all");

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

    // Q1（跑前登记岔路单第 1 行，C393）：H1 家族 + Φ1 故障注入 + PC1-a/PC1-b。只在 G0 上跑
    // （这一段没有做第八节的几何敏感性格），独立模式、不并进 "all"。
    if mode == "q1-g0" {
        let summary = run_ledger_fault_family(&GEOMETRY_PRIMARY);
        emit_ledger_fault_family_summary(&GEOMETRY_PRIMARY, &summary);
        run_ledger_fault_positive_controls(&GEOMETRY_PRIMARY);
    }

    // Q2-1（跑前登记岔路单第 2 行 ①，C331 修法）：H2 主族在 G0 上、n1 ∈ {0,1,2,3}、n2=1 的穷举下界。
    // 只在这一份 crates/ 上跑（同一个二进制在不同臂的副本目录里各编各的，命令与产物文件名分臂）。
    if mode == "q2-1-g0" {
        run_rootback_tolerance_family(&GEOMETRY_PRIMARY, 1);
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

    let arithmetic_checks: [(&str, i64, i64); 11] = [
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

    /// PC2：环境变量没设，本地常量按今天 / 甲-txg 那一份取 481。
    #[test]
    fn parse_local_system_configuration_bytes_defaults_to_481_when_unset() {
        let value = parse_local_system_configuration_bytes(Err(env::VarError::NotPresent));
        assert_eq!(value, 481, "没设环境变量应当取今天的 481");
    }

    /// PC2：环境变量给了十进制数字，原样解出来（乙族传 489）。
    #[test]
    fn parse_local_system_configuration_bytes_parses_the_given_decimal() {
        let value = parse_local_system_configuration_bytes(Ok("489".to_string()));
        assert_eq!(value, 489, "环境变量给的十进制数要原样解出来");
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
}
