//! E173（上限卡在读坏槽的三种处置）第一次重跑的第一段（重跑登记 `research/prompts/e173-r1-prereg.md` 第五节「第一段要做的」）：
//! 在真盘面上量 ckpt_cost（记 κ，核心层 `admission::checkpoint_cost_of_the_version_to_build_on`）与一次空发布实际新占的固定点槽数
//! （记 c实），答登记第一节问题单 ③乙那一行；第二段（原装置加 `checkpoint-cost-reserve` 模式）不在这个二进制里。
//!
//! 入库装置：只读、只驱动、只观测 `singlefs-core` 的公开入口，不改核心层。决定几何、工作量与判据门槛的量都是本地常量，
//! 值抄自 kb 与格式定义位并注明出处，[`check_local_constants_against_the_crates`] 逐个回比（登记第十一节停机条款 S4）。
//! 产物按 `E7RESULT` 行打印，复跑登记在 `scripts/experiment-apparatus/replay.sh`。
//!
//! 每个几何（每区槽数 S 与池大小）走同一条历史：mkfs → 可写挂载 → 第一个文件版本 → 顺序写 32 个数据单元 →
//! 三圈「覆盖写 4 × 3S 次 → 正常卸载 → 再可写挂载」。每一次交回了输出的发布记一行：κ（核心层与本地照 D28（挂载期承诺量） 已定项 4
//! 重算的各一份，停机条款 S2）、新占的固定点两条路逐盘各一份（停机条款 S3）。
//!
//! 准入写 `always`：复跑登记要在输入不变时再跑同一段、拿同一份输入核产物逐字节相同。
//! 退出码：0 正常；2 用法错；3 停机或作废（产物里有 `name=stop` 或 `name=void`，登记第十、十一节）；准入判法退 78 时照 78 退。

// admission: always 复跑登记要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::admission::{
    accounting_tree_nodes_after_the_publish_built_on, checkpoint_cost_of_the_version_to_build_on,
    space_budget_of_role, SpaceBudgetOfARole,
};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable, unmount, MountOutput, MountSpaceAdmission, Mounted, Unmounted,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangePublished};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, AllocationRecordsOfTheVersionWithoutFile, CodeTwoTreeNodeCapacities,
    FirstFile, MultiLevelCodeTwoTree, PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
    VersionWithoutFilePublishOutput,
};
use singlefs_verification::memory_pool::SparseBlockDevice;

mod checkpoint_cost_model;
mod checks_against_the_crates;
mod history_of_publishes;
mod reserve_fit_and_summary;

use checkpoint_cost_model::*;
use checks_against_the_crates::*;
use history_of_publishes::*;
use reserve_fit_and_summary::*;

/// 本地常量：槽 16 KiB（`crates/singlefs-format/src/lib.rs` 的 `SLOT_BYTES`；D3（空间分配） 已定项 7）。
const SLOT_BYTES_LOCAL: u64 = 16384;
/// 本地常量：数据单元 32 KiB（同文件 `DATA_UNIT_BYTES`；D4（校验和位置） 已定项 5）。
const DATA_UNIT_BYTES_LOCAL: u64 = 32768;
/// 本地常量：一个数据单元装得下的载荷 32634 字节 = 32768 − 头 105 − 预留 29（`singlefs_core::unit::data_unit_payload_capacity` 的文档）。
/// 它定顺序写与覆盖写的内容长度（一个数据单元正好装满）。
const DATA_UNIT_PAYLOAD_BYTES_LOCAL: u64 = 32634;
/// 本地常量：分配记录树一片叶罩 812 槽、内部扇出 169（D28（挂载期承诺量） 已定项 4「S_L = 812 × 169^L」；
/// `crates/singlefs-format/src/lib.rs` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`、`ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL: u64 = 812;
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL: u64 = 169;
/// 本地常量：根环 3 个区域（`crates/singlefs-format/src/lib.rs` 的 `ROOT_RING_REGIONS`；D22（单元原子性怎么合成） 已定项 2）。
const ROOT_RING_REGIONS_LOCAL: u64 = 3;
/// 本地常量：每区槽数 S 的区间 [4, 16]（同文件 `ROOT_RING_SLOTS_PER_REGION_MINIMUM`、`ROOT_RING_SLOTS_PER_REGION_MAXIMUM`；
/// D16（发布语义） 已定项 1「环深下限」那一行 S ≥ k_tol + 2 = 4）。
const ROOT_RING_SLOTS_PER_REGION_MINIMUM_LOCAL: u64 = 4;
const ROOT_RING_SLOTS_PER_REGION_MAXIMUM_LOCAL: u64 = 16;
/// 本地常量：默认 journal 环 768 MiB 与它之下的单元区起始槽号 50176（同文件 `JOURNAL_RING_DEFAULT_BYTES`、`UNIT_AREA_START_SLOT`；
/// D3（空间分配） 已定项 10 ④）。
const DEFAULT_JOURNAL_RING_BYTES_LOCAL: u64 = 805_306_368;
const UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL: u64 = 50176;
/// 本地常量：journal 环从槽 1024 起（`.claude/kb/layout/01-first-txn.md`「journal 环（从槽 1024 起）」；
/// `singlefs_core::make_filesystem::FixedStructure::JournalRing` 的文档）。单元区起点 = 1024 + 环槽数。
const JOURNAL_RING_START_SLOT_LOCAL: u64 = 1024;
/// 本地常量：journal 环不超过设备容量的四分之一（D23（journal 的角色与格式） 已定项 19；
/// `crates/singlefs-core/src/make_filesystem.rs` 的 `JournalRingTooLargeForDevice` 那一判）。
const JOURNAL_RING_AT_MOST_ONE_PART_OF_THE_DEVICE_IN: u64 = 4;
/// 本地常量：两块同构盘的 physical_block_size 与 io_min 都是 512、固定结构槽距 4096
/// （`.claude/kb/layout/01-first-txn.md`「两块盘、同构、physical_block_size = 512、io_min = 512」与系统配置槽 1 那一行「固定结构槽距 = 4096」）。
const PHYSICAL_BLOCK_BYTES_LOCAL: u32 = 512;
const MINIMUM_INPUT_OUTPUT_BYTES_LOCAL: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING_LOCAL: u32 = 4096;
/// 本地常量：一次准入最多 B = 4 + 2 k_tol = 8 次发布（D16（发布语义） 已定项 1「准入」那一行；`singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST`）。
const PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL: u64 = 8;
/// 本地常量：谱系容器、退役容器一次 COW 各 2 块，树根各 1 块（D28（挂载期承诺量） 已定项 4「容器 2 块 + 谱系树根 1 块」；
/// `crates/singlefs-core/src/admission.rs` 的 `LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE` 那四个常量）。
const LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL: u64 = 2;
const LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL: u64 = 1;
const RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL: u64 = 2;
const RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL: u64 = 1;
/// 本地常量：一次谱系（退役）容器 COW 在中央映射树里改的 key：容器一把、树根一把、内部层每层一把
/// （D28（挂载期承诺量） 已定项 4「谱系容器、谱系树根各一把和谱系树内部层每层一把」）。
const MAPPED_KEYS_OF_ONE_CONTAINER_LOCAL: u64 = 1;
const MAPPED_KEYS_OF_ONE_CONTAINER_TREE_ROOT_LOCAL: u64 = 1;
/// 本地常量：树表每次发布 1 个单元（D28（挂载期承诺量） 已定项 4「树表单列一项，每次发布 1 个单元」）。
/// 核心层那一项是函数里的局部常量、回比不了，由停机条款 S2（本地 κ 与核心层 κ 逐次相同）间接盯着。
const TREE_TABLE_UNITS_PER_PUBLISH_LOCAL: u64 = 1;

/// 判据门槛：抬 F 那一串最长 3 次（D16（发布语义） 已定项 1「正常卸载」那一行：现行 txg 除以 3 余 1 时 3 次）。登记第六节 X3 的 3。
const PUBLISHES_OF_THE_LONGEST_FLOOR_RAISE_CHAIN: u64 = 3;
/// 判据门槛：一次准入至多推 B − 1 = 7 次空发布（D16（发布语义） 已定项 1「`df`」那一行的 (B − 1) × c_max）。登记第六节 X3 的 7。
const EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST: u64 = PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL - 1;
/// 条款读法保留池的常数项 2 × 5（D16（发布语义） 已定项 1「`df`」那一行「保留池 = 2 × 5 + (B − 1) × c_max」）。
const CLAUSE_RESERVE_CONSTANT_BLOCKS: u64 = 2 * 5;

/// 几何旋钮：S 取 4、8、16（登记第一节「几何」）；池大小三档（G-min、G-1、G-4）。
const SLOTS_PER_REGION_SAMPLED: [u64; 3] = [4, 8, 16];
/// G-min 的找法：每块盘从 32 MiB 起依次试（登记第五节「G-min 的找法」）。
const SMALLEST_POOL_TRIAL_MEBIBYTES: [u64; 5] = [32, 64, 128, 256, 512];
const MEBIBYTE: u64 = 1 << 20;
const GIBIBYTE: u64 = 1 << 30;
/// 历史（登记第五节第 1–4 步）：顺序写 32 个数据单元（64 块）；每圈覆盖写 4 × 3S 次；第 2–4 步共走 3 圈。
const SEQUENTIALLY_WRITTEN_DATA_UNITS: u64 = 32;
const OVERWRITES_PER_ROOT_RING_SLOT_PER_CYCLE: u64 = 4;
const CYCLES_OF_OVERWRITE_UNMOUNT_REMOUNT: u64 = 3;
/// 作废条款 V6：一个几何上取得出的空发布（暖机与卸载那一串）少于 6 次，那个几何作废。
const OBTAINED_EMPTY_PUBLISHES_AT_LEAST: usize = 6;
/// 固定写入时间（只进 inode 记录，不影响落点与槽数）。
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 本装置的固定文件系统身份（与别的 eNNN 装置各自独立取一份）。
const E173_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x37, 0x33, 0x2d, 0x52, 0x31, 0x2d, 0x30, 0x31, 0x00,
];

/// 这个进程已经打出的 `E7RESULT` 行数（收尾行的 `emitted` 要等于整份产物的行数，复跑脚本的完整性闸核它）。
static EMITTED_LINES: AtomicU64 = AtomicU64::new(0);

/// 打一行产物：`E7RESULT <字段>`。只在主线程里按确定的次序调（各几何在线程里跑完之后由主线程逐个打），产物不随线程数变。
fn emit(fields: &str) {
    EMITTED_LINES.fetch_add(1, Ordering::SeqCst);
    println!("E7RESULT {fields}");
}

/// 收尾行：`name=done emitted=<行数，含这一行>`。
fn emit_the_done_line() {
    let emitted = EMITTED_LINES.load(Ordering::SeqCst) + 1;
    println!("E7RESULT name=done emitted={emitted}");
}

/// 池大小那一维的三档（登记第一节「几何」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PoolSize {
    /// G-min 的一次试跑：每块盘 `mebibytes` MiB（登记第五节「G-min 的找法」）。
    SmallestTrial { mebibytes: u64 },
    /// G-1：两块各 1 GiB。
    OneGibibyte,
    /// G-4：两块各 4 GiB。
    FourGibibytes,
}

impl PoolSize {
    fn device_bytes(self) -> u64 {
        match self {
            PoolSize::SmallestTrial { mebibytes } => mebibytes * MEBIBYTE,
            PoolSize::OneGibibyte => GIBIBYTE,
            PoolSize::FourGibibytes => 4 * GIBIBYTE,
        }
    }

    fn label(self) -> String {
        match self {
            PoolSize::SmallestTrial { mebibytes } => format!("g_min_{mebibytes}mib"),
            PoolSize::OneGibibyte => "g_1".to_string(),
            PoolSize::FourGibibytes => "g_4".to_string(),
        }
    }
}

/// 这块盘上 mkfs 取的 journal 环（登记修订：登记没写环长；默认 768 MiB 只在盘 ≥ 3 GiB 时合法）：
/// 取默认环与「盘的四分之一按 MiB 向下取整」里小的那个——mkfs 拒「环超过设备容量的四分之一」（`JournalRingTooLargeForDevice`），
/// 也拒单元区起点不在 64 槽聚簇段边界上（环长取 1 MiB 的整数倍就在边界上，`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`）。
/// G-4 取到默认环（单元区 [50176, 262144)，登记第七节 B10）。
fn journal_ring_bytes_of_a_device(device_bytes: u64) -> u64 {
    let largest_legal_in_whole_mebibytes =
        device_bytes / JOURNAL_RING_AT_MOST_ONE_PART_OF_THE_DEVICE_IN / MEBIBYTE * MEBIBYTE;
    DEFAULT_JOURNAL_RING_BYTES_LOCAL.min(largest_legal_in_whole_mebibytes)
}

/// 一个几何：每区槽数 S 与池大小（两块等大的盘）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Geometry {
    slots_per_region: u64,
    pool_size: PoolSize,
}

impl Geometry {
    fn label(self) -> String {
        format!("s{}_{}", self.slots_per_region, self.pool_size.label())
    }

    fn device_bytes(self) -> u64 {
        self.pool_size.device_bytes()
    }

    fn journal_ring_bytes(self) -> u64 {
        journal_ring_bytes_of_a_device(self.device_bytes())
    }

    /// 本地算的单元区 [起点, 终点)：起点 = journal 环从槽 1024 起之后的下一个槽，终点 = 盘的槽数（D3（空间分配） 已定项 10 ④）。
    fn local_unit_area(self) -> (u64, u64) {
        (
            JOURNAL_RING_START_SLOT_LOCAL + self.journal_ring_bytes() / SLOT_BYTES_LOCAL,
            self.device_bytes() / SLOT_BYTES_LOCAL,
        )
    }

    /// 这条历史覆盖写几次：每圈 4 × 3S（登记第五节第 2 步）。
    fn overwrites_per_cycle(self) -> u64 {
        OVERWRITES_PER_ROOT_RING_SLOT_PER_CYCLE * ROOT_RING_REGIONS_LOCAL * self.slots_per_region
    }
}

/// mkfs 参数：每区槽数取这个几何的 S（经格式的区间判，越界交不出参数），环长照 [`journal_ring_bytes_of_a_device`]。
fn make_filesystem_parameters(geometry: Geometry) -> Result<MakeFilesystemParameters, String> {
    let root_ring_slots_per_region =
        RootRingSlotsPerRegion::from_system_configuration_field(geometry.slots_per_region)
            .map_err(|_out_of_range| {
                format!(
                    "slots_per_region_out_of_range:{}",
                    geometry.slots_per_region
                )
            })?;
    Ok(MakeFilesystemParameters {
        filesystem_identifier: E173_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES_LOCAL,
            minimum_input_output_bytes: MINIMUM_INPUT_OUTPUT_BYTES_LOCAL,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_LOCAL,
            journal_ring_bytes: geometry.journal_ring_bytes(),
            root_ring_slots_per_region,
        },
    })
}

/// 一个数据单元装满的内容，`seed` 不同内容不同（覆盖写每次换一份，免得写的是同一组字节）。
fn one_data_unit_of_content(seed: u64) -> Vec<u8> {
    let payload_bytes =
        usize::try_from(DATA_UNIT_PAYLOAD_BYTES_LOCAL).expect("载荷字节数装得进 usize");
    (0..payload_bytes)
        .map(|index| {
            let mixed = (u64::try_from(index).expect("下标装得进 u64") + seed * 7919) % 251;
            u8::try_from(mixed).expect("小于 251")
        })
        .collect()
}

/// 顺序写的内容：`data_units` 个数据单元装满。
fn sequential_write_content(data_units: u64) -> Vec<u8> {
    (0..data_units)
        .flat_map(|unit_index| one_data_unit_of_content(1_000_000 + unit_index))
        .collect()
}

/// 错误的 Debug 文本里成员名那几层（`Publish(SpaceAdmissionRefused(…` 取 `Publish:SpaceAdmissionRefused`），产物里一个字段不带空白。
fn variant_name_of(debug_text: &str) -> String {
    let mut name = String::new();
    for character in debug_text.chars() {
        match character {
            '(' => name.push(':'),
            kept if kept.is_alphanumeric() || kept == '_' => name.push(kept),
            _ => break,
        }
    }
    name.trim_end_matches(':').to_string()
}

/// 本地照 D28（挂载期承诺量） 已定项 4 的 Σ 名单重算的 κ，各项分开（登记第七节 A8）。记账树那一项取核心层同名公开函数
/// （`accounting_tree_nodes_after_the_publish_built_on`：写路径的规划器，E181（落得下那一判与挂载下界） 装置同一个先例），其余各项本地算。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LocalCheckpointCost {
    allocation_record_tree: u64,
    central_mapping_tree: u64,
    accounting_tree: u64,
    lineage_container_copy_on_write: u64,
    retirement_container_copy_on_write: u64,
    tree_table: u64,
}

/// 一次发布之前那一版上的 κ：核心层一份，本地一份（停机条款 S2 比两份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckpointCostReading {
    built_on: BuiltOn,
    core: u64,
    local: LocalCheckpointCost,
}

/// 一次发布在每块盘上新占的固定点槽数，两条路各一份（登记第一节「一次空发布实际新占的块数」，口径照 E181（落得下那一判与挂载下界） 装置的
/// `fixed_point_account`）：路一按这次重写的角色里归 checkpoint 保留池的跨度相加（每个单元落每块盘，逐盘相同）；
/// 路二按这一版分配记录里分配代等于这次 txg、没释放的记录逐盘加跨度，再减普通分配与实例切换预留两类角色的槽数。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FixedPointAccount {
    by_roles: u64,
    /// 逐盘（盘 0、盘 1）；`None`：这次发布没交回这一版的分配记录（树表 0 条那一版上的零单元发布），路二取不出。
    by_records_per_device: Option<Vec<i64>>,
    ordinary_slots: u64,
    instance_switch_slots: u64,
}

/// 一次发布记一行（登记第五节）：κ 取不出（之前那一版不在手里）或输出取不出（入口没交回）的，记「取不出」，不当 0。
#[derive(Clone, Debug)]
struct PublishObservation {
    kind: PublishKind,
    cycle: u64,
    txg: u64,
    checkpoint_cost: Option<CheckpointCostReading>,
    fixed_point: Option<FixedPointAccount>,
}

/// PC7（登记第五节「阳性对照」）：每个几何的第一次覆盖写上，钩子开着时 S3 判红、钩子关着时两条路相同、普通分配槽数 > 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlOfTheRoutes {
    txg: u64,
    hook_on_routes_agree: Option<bool>,
    hook_off_routes_agree: Option<bool>,
    ordinary_slots: u64,
}

/// 一条历史的全部读数。
struct HistoryReport {
    geometry: Geometry,
    end: HistoryEnd,
    refusals: Vec<String>,
    publishes: Vec<PublishObservation>,
    /// mkfs 之后分配器上每块盘的单元区 [起点, 终点)（核心层的读数，与几何本地算的 [`Geometry::local_unit_area`] 并排报）。
    unit_areas_of_the_allocator: Vec<(u64, u64)>,
    /// 最后那一版根节点头里读出的分配记录树根层级。
    allocation_record_tree_root_level: Option<u8>,
    positive_control_of_the_routes: Option<PositiveControlOfTheRoutes>,
    /// 最后那一版上的 κ（接在它后面的那次发布的 κ，轨迹的期末值）。
    final_checkpoint_cost: Option<CheckpointCostReading>,
}

/// X3（登记第六节）：两读法的保留池各装得下几次实际的空发布，n = ⌊P ÷ max c实⌋（登记第七节 B11：等于 `reserve_fit.py` 的 `empties_held`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReserveFit {
    checkpoint_cost: u64,
    largest_actual_empty_publish_slots: u64,
    code_reserve: u64,
    clause_reserve: u64,
    code_empty_publishes_held: u64,
    clause_empty_publishes_held: u64,
}

/// 一个几何上 X3 与它的三个判定（按 κ_G 的三种取法各一份），给判别力自证与几何敏感性用。
struct GeometryVerdicts {
    geometry: Geometry,
    fits: Vec<(CheckpointCostOfTheGeometry, ReserveFit)>,
}

/// 要跑的全部几何（登记第十四节「独立单位」）：每个 S 上 G-min 的五档试跑、G-1、G-4。
fn every_geometry_to_run() -> Vec<Geometry> {
    let mut geometries = Vec::new();
    for slots_per_region in SLOTS_PER_REGION_SAMPLED {
        for mebibytes in SMALLEST_POOL_TRIAL_MEBIBYTES {
            geometries.push(Geometry {
                slots_per_region,
                pool_size: PoolSize::SmallestTrial { mebibytes },
            });
        }
        for pool_size in [PoolSize::OneGibibyte, PoolSize::FourGibibytes] {
            geometries.push(Geometry {
                slots_per_region,
                pool_size,
            });
        }
    }
    geometries
}

/// G-min（登记第五节）：按 32、64、128、256、512 MiB 的次序取第一个做得完的；五档都做不完交 `None`（「找不到」，第十节 F6）。
fn smallest_completed_trial(
    trial_sizes_in_mebibytes: &[u64],
    completes: impl Fn(u64) -> bool,
) -> Option<u64> {
    trial_sizes_in_mebibytes
        .iter()
        .copied()
        .find(|trial_mebibytes| completes(*trial_mebibytes))
}

/// 线程数取登记值（`.claude/heavy-thread-default`，经 `SINGLEFS_THREAD_CAP`），`E173_THREADS` 设了照它。
fn thread_count() -> usize {
    let thread_cap_value =
        std::env::var(singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE).ok();
    singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads(
        "E173_THREADS",
        std::env::var("E173_THREADS").ok().as_deref(),
        thread_cap_value.as_deref(),
    )
    .get()
}

/// 全部几何按工作队列分到 `threads` 条线程上（几何之间互相独立；同一个几何的历史是一串接着上一版的发布，只能按次序）。
/// 读数按几何的次序收回，产物由主线程按这个次序打，所以不随线程数变。
fn run_every_geometry(geometries: &[Geometry], threads: usize) -> Vec<HistoryReport> {
    let next_index = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<HistoryReport>>> =
        Mutex::new((0..geometries.len()).map(|_index| None).collect());
    let workers = threads.min(geometries.len()).max(1);
    std::thread::scope(|scope| {
        for _worker in 0..workers {
            scope.spawn(|| loop {
                let index = next_index.fetch_add(1, Ordering::SeqCst);
                let Some(geometry) = geometries.get(index).copied() else {
                    break;
                };
                let report = run_the_history(geometry, HistoryPlan::registered_for(geometry));
                let end_text = report.end.text();
                slots.lock().expect("收读数的锁没被别的线程 panic 弄坏")[index] = Some(report);
                let finished_now = finished.fetch_add(1, Ordering::SeqCst) + 1;
                eprintln!(
                    "[e173] 做完 {}（{finished_now} / {}）：{end_text}",
                    geometry.label(),
                    geometries.len()
                );
            });
        }
    });
    slots
        .into_inner()
        .expect("收读数的锁没被弄坏")
        .into_iter()
        .map(|slot| slot.expect("工作队列把每个几何都跑了一遍"))
        .collect()
}

const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `main` 的第一句：直接起 `python3 preflight.py check <本文件> [--force] -- <参数…>`（不经 `sh -c`）判准入与运行条件，
/// 不满足就照判法的退出码退出、一步都不做；强制跑（判法的第一行以 `forced` 起头）时交回没满足的条件摘要，要写进产物。
fn preflight() -> Option<String> {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/e173_checkpoint_cost_against_empty_publish_cost/main.rs"
    );
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.claude/singlefs-ai-sop/scripts/preflight.py"
    );
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(script).arg("check").arg(source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE)
    });
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    match output.status.code() {
        Some(0) => {}
        Some(PREFLIGHT_REFUSED_EXIT_CODE) => std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE),
        Some(_) | None => std::process::exit(1),
    }
    let verdict_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    if verdict_line.starts_with("forced") {
        let last_segment = verdict_line.rsplit('\t').next().unwrap_or("");
        Some(
            last_segment
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("_"),
        )
    } else {
        None
    }
}

/// G-min 的一次试跑一行（登记第五节「G-min 的找法」）。
fn trial_row(report: &HistoryReport) -> String {
    let geometry = report.geometry;
    format!(
        "name=g_min_trial geometry={} slots_per_region={} device_bytes={} journal_ring_bytes={} end={} refusals={} first_refusal={} completes_the_history={}",
        geometry.label(),
        geometry.slots_per_region,
        geometry.device_bytes(),
        geometry.journal_ring_bytes(),
        report.end.text(),
        report.refusals.len(),
        report.refusals.first().map_or("none", String::as_str),
        report.completes_the_history(),
    )
}

fn main() {
    if let Some(summary) = preflight() {
        emit(&format!("name=preflight forced={summary}"));
    }
    let requested: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let requested_texts: Vec<&str> = requested.iter().map(String::as_str).collect();
    match requested_texts.as_slice() {
        [] | ["segment-one"] => {}
        _ => {
            eprintln!(
                "用法：e173_checkpoint_cost_against_empty_publish_cost [segment-one] [--force]"
            );
            std::process::exit(2);
        }
    }
    let threads = thread_count();
    let geometries = every_geometry_to_run();
    emit(&format!(
        "name=config section=segment-one threads_of_the_geometries={} geometries={} slots_per_region_sampled=4,8,16 g_min_trial_mebibytes=32,64,128,256,512 sequentially_written_data_units={SEQUENTIALLY_WRITTEN_DATA_UNITS} overwrites_per_cycle=4x3S cycles={CYCLES_OF_OVERWRITE_UNMOUNT_REMOUNT} journal_ring=min(805306368,device_bytes/4_in_whole_mebibytes)",
        threads.min(geometries.len()).max(1),
        geometries.len(),
    ));
    if !check_local_constants_against_the_crates() {
        emit("name=stop clause=s4_local_constant_differs_from_the_crates");
        std::process::exit(3);
    }
    if !check_the_anchors() {
        emit("name=void clause=v3_anchor_differs");
        std::process::exit(3);
    }
    let reports = run_every_geometry(&geometries, threads);
    let report_of = |geometry: Geometry| {
        reports
            .iter()
            .find(|report| report.geometry == geometry)
            .expect("每个要跑的几何都有一份读数")
    };
    for report in &reports {
        if let PoolSize::SmallestTrial { .. } = report.geometry.pool_size {
            emit(&trial_row(report));
        }
    }
    let mut measured: Vec<&HistoryReport> = Vec::new();
    for slots_per_region in SLOTS_PER_REGION_SAMPLED {
        let trial_geometry = |mebibytes: u64| Geometry {
            slots_per_region,
            pool_size: PoolSize::SmallestTrial { mebibytes },
        };
        let chosen = smallest_completed_trial(&SMALLEST_POOL_TRIAL_MEBIBYTES, |mebibytes| {
            report_of(trial_geometry(mebibytes)).completes_the_history()
        });
        emit(&format!(
            "name=g_min slots_per_region={slots_per_region} chosen={}",
            chosen.map_or_else(
                || "not_found_up_to_512mib".to_string(),
                |mebibytes| trial_geometry(mebibytes).label()
            )
        ));
        if let Some(mebibytes) = chosen {
            measured.push(report_of(trial_geometry(mebibytes)));
        }
        for pool_size in [PoolSize::OneGibibyte, PoolSize::FourGibibytes] {
            measured.push(report_of(Geometry {
                slots_per_region,
                pool_size,
            }));
        }
    }
    let mut stops_and_voids = Vec::new();
    for report in &measured {
        let label = report.geometry.label();
        emit(&geometry_row(report));
        for observation in &report.publishes {
            emit(&publish_row(&label, observation));
        }
        let (positive_control_row, positive_control_holds) = positive_control_row_and_holds(report);
        emit(&positive_control_row);
        if !positive_control_holds {
            stops_and_voids.push(format!(
                "name=void clause=v1_positive_control_routes geometry={label}"
            ));
        }
        stops_and_voids.extend(stop_findings_of_a_geometry(report));
    }
    if !stops_and_voids.is_empty() {
        for line in &stops_and_voids {
            emit(line);
        }
        std::process::exit(3);
    }
    let verdicts: Vec<GeometryVerdicts> = measured
        .iter()
        .copied()
        .filter_map(emit_the_summary_of_a_geometry)
        .collect();
    emit_discrimination_and_stability(&verdicts);
    emit_the_done_line();
}

#[cfg(test)]
mod tests;
