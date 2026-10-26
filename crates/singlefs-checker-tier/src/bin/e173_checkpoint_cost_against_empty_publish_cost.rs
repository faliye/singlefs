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
use singlefs_harness::memory_pool::SparseBlockDevice;

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
/// `crates/singlefs-core/src/admission.rs` 的 `LINEAGE_CONTAINER_BLOCKS_OF_ONE_COW` 那四个常量）。
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

/// 分配记录树根之下层级 `level` 上一个位置罩的槽数：812 × 169^层级（D28（挂载期承诺量） 已定项 4）。
fn allocation_record_tree_span_in_slots_at_level(level: u8) -> u64 {
    let mut span = ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL;
    for _climbed in 0..level {
        span *= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL;
    }
    span
}

/// K1（D28（挂载期承诺量） 已定项 4「分配记录树这一项按 K1 算」，登记第七节 A7）：
/// 1 + Σ_盘 Σ_{L = 0 … 根层级 − 1}（⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1），单元区按 [s, e) 给。
fn local_allocation_record_tree_positions(root_level: u8, unit_areas: &[(u64, u64)]) -> u64 {
    let mut positions = 1;
    for (first_slot, end_slot) in unit_areas {
        assert!(
            end_slot > first_slot,
            "单元区至少一槽：[{first_slot}, {end_slot})"
        );
        for level in 0..root_level {
            let span = allocation_record_tree_span_in_slots_at_level(level);
            positions += (end_slot - 1) / span - first_slot / span + 1;
        }
    }
    positions
}

/// 树表 0 条、写过行的那一版上分配记录树的根层级取池几何的（D28（挂载期承诺量） 已定项 4 子条「树表 0 条、写过行的那一版取池几何的根层级」）：
/// 最小的层级 L ≥ 1，使根下那一层（层级 L − 1）每块盘按整盘槽数罩到的格数之和装得进一个内部节点（扇出 169，D8（核心索引结构） 已定项 14）。
fn local_root_level_of_the_pool_geometry(device_slot_counts: &[u64]) -> u8 {
    (1..=u8::MAX)
        .find(|candidate_root_level| {
            let child_span =
                allocation_record_tree_span_in_slots_at_level(candidate_root_level - 1);
            let cells: u64 = device_slot_counts
                .iter()
                .map(|slots| slots.div_ceil(child_span))
                .sum();
            cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL
        })
        .expect("两块盘、槽数有上界 2^48：层级至多 6 就装得下每块盘各一格")
}

/// 一层上至多切几次（D8（核心索引结构） 已定项 11 ②：装不下从中间切）：原有节点第一次切至少要加 1 条，
/// 切出来的新节点再切一次至少要加 `additions_per_split_of_a_fresh_node` 条。
fn local_splits_at_one_level_at_most(
    pre_existing_nodes: u64,
    additions: u64,
    additions_per_split_of_a_fresh_node: u64,
) -> u64 {
    let first_splits = pre_existing_nodes.min(additions);
    first_splits + (additions - first_splits) / additions_per_split_of_a_fresh_node
}

/// 中央映射树一次空发布至多重写的节点数（D28（挂载期承诺量） 已定项 4「中央映射树逐层按可能改的路径数与可能切出来的节点数」）：
/// 每层 min(这一层的节点数, 删 + 插) 加这一层切出来的；根切开就长高一层，新根一个、装不下再切。
fn local_central_mapping_tree_nodes_rewritten_at_most(
    nodes_at_each_level: &[u64],
    entries_before: u64,
    deleted_at_most: u64,
    inserted_at_most: u64,
    capacity: CodeTwoTreeNodeCapacity,
) -> u64 {
    let leaf_entries = u64::try_from(capacity.leaf_entries).expect("叶容量装得进 u64");
    let internal_entries = u64::try_from(capacity.internal_entries).expect("内部容量装得进 u64");
    let leaf_additions_per_split = leaf_entries.div_ceil(2);
    let internal_additions_per_split = internal_entries.div_ceil(2);
    let changed_paths = deleted_at_most + inserted_at_most;
    let mut rewritten = 0;
    let mut splits_at_the_level_below = 0;
    for (level, nodes_here) in nodes_at_each_level.iter().copied().enumerate() {
        let splits_here = if level == 0 {
            if entries_before + inserted_at_most <= leaf_entries {
                0
            } else {
                local_splits_at_one_level_at_most(
                    nodes_here,
                    inserted_at_most,
                    leaf_additions_per_split,
                )
            }
        } else if nodes_at_each_level[level - 1] + splits_at_the_level_below <= internal_entries {
            0
        } else {
            local_splits_at_one_level_at_most(
                nodes_here,
                splits_at_the_level_below,
                internal_additions_per_split,
            )
        };
        rewritten += nodes_here.min(changed_paths) + splits_here;
        splits_at_the_level_below = splits_here;
    }
    while splits_at_the_level_below > 0 {
        let splits_of_the_new_level = if splits_at_the_level_below < internal_entries {
            0
        } else {
            (splits_at_the_level_below - 1) / internal_additions_per_split
        };
        rewritten += 1 + splits_of_the_new_level;
        splits_at_the_level_below = splits_of_the_new_level;
    }
    rewritten
}

/// 中央映射树每层有几个节点（自叶到根），按这一版根节点头里读出的高。
fn central_mapping_tree_nodes_at_each_level(version: &TransactionOutput) -> Vec<u64> {
    let height =
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping);
    let mut counts = vec![0_u64; usize::try_from(height).expect("树高装得进 usize")];
    for node in version.central_mapping_tree.shape.nodes() {
        counts[usize::from(node.position.level)] += 1;
    }
    counts
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

impl LocalCheckpointCost {
    fn total(self) -> u64 {
        self.allocation_record_tree
            + self.central_mapping_tree
            + self.accounting_tree
            + self.lineage_container_copy_on_write
            + self.retirement_container_copy_on_write
            + self.tree_table
    }

    fn terms_text(self) -> String {
        format!(
            "{}+{}+{}+{}+{}+{}",
            self.allocation_record_tree,
            self.central_mapping_tree,
            self.accounting_tree,
            self.lineage_container_copy_on_write,
            self.retirement_container_copy_on_write,
            self.tree_table
        )
    }
}

/// 一次发布要接在后面的那一版，装置手里有的样子。
#[derive(Clone, Copy)]
enum VersionToBuildOn<'version> {
    /// 树表 0 条的一版：核心层按分配器算（分配器记没记着这一版写行时写下的分配记录树，`allocation_record_tree_of_the_version_without_file`），
    /// 本地按装置看到的「这一版上写没写过行」算（[`PublishedVersion::wrote_the_allocation_record_tree_on_a_version_without_file`]）。
    WithoutFile {
        allocation_record_tree_written: bool,
        allocator: &'version PoolAllocator,
    },
    /// 有树的一版：两边都从这一版现读。
    WithTrees(&'version TransactionOutput),
}

/// κ 是接在哪一种版本后面算的（产物与汇总只拿有树的那几版定 κ_G，登记修订）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BuiltOn {
    VersionWithoutFile,
    VersionWithTrees,
}

impl BuiltOn {
    fn label(self) -> &'static str {
        match self {
            BuiltOn::VersionWithoutFile => "version_without_file",
            BuiltOn::VersionWithTrees => "version_with_trees",
        }
    }
}

/// 一次发布之前那一版上的 κ：核心层一份，本地一份（停机条款 S2 比两份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckpointCostReading {
    built_on: BuiltOn,
    core: u64,
    local: LocalCheckpointCost,
}

/// 本地 κ（D28（挂载期承诺量） 已定项 4）：单元区与池几何的根层级按这个几何本地算，各棵树的形状从那一版读。
fn local_checkpoint_cost(geometry: Geometry, version: VersionToBuildOn<'_>) -> LocalCheckpointCost {
    let unit_area = geometry.local_unit_area();
    let unit_areas = [unit_area, unit_area];
    match version {
        VersionToBuildOn::WithoutFile {
            allocation_record_tree_written,
            ..
        } => {
            let device_slots = geometry.device_bytes() / SLOT_BYTES_LOCAL;
            let root_level = local_root_level_of_the_pool_geometry(&[device_slots, device_slots]);
            LocalCheckpointCost {
                allocation_record_tree: if allocation_record_tree_written {
                    local_allocation_record_tree_positions(root_level, &unit_areas)
                } else {
                    0
                },
                central_mapping_tree: 0,
                accounting_tree: 0,
                lineage_container_copy_on_write: 0,
                retirement_container_copy_on_write: 0,
                tree_table: TREE_TABLE_UNITS_PER_PUBLISH_LOCAL,
            }
        }
        VersionToBuildOn::WithTrees(previous) => {
            let root_level = u8::try_from(
                previous
                    .position_addressed_tree_heights_read_from_the_root_node_headers()
                    .allocation_record_tree
                    - 1,
            )
            .expect("根层级装得进 u8");
            let allocation_record_tree =
                local_allocation_record_tree_positions(root_level, &unit_areas);
            let capacities = CodeTwoTreeNodeCapacities::FromTheNodeFormat;
            let accounting_nodes_before =
                u64::try_from(previous.accounting_tree.node_count()).expect("节点数装得进 u64");
            let accounting_tree = accounting_tree_nodes_after_the_publish_built_on(
                previous,
                capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
            );
            let lineage_levels = previous.lineage_tree_internal_levels();
            let retirement_levels = previous.retirement_tree_internal_levels();
            let lineage_keys = MAPPED_KEYS_OF_ONE_CONTAINER_LOCAL
                + MAPPED_KEYS_OF_ONE_CONTAINER_TREE_ROOT_LOCAL
                + lineage_levels;
            let retirement_keys = MAPPED_KEYS_OF_ONE_CONTAINER_LOCAL
                + MAPPED_KEYS_OF_ONE_CONTAINER_TREE_ROOT_LOCAL
                + retirement_levels;
            let entries_before = u64::try_from(
                previous.mapped_units.len() + previous.mapping_entries_kept_for_the_snapshots.len(),
            )
            .expect("映射条目数装得进 u64");
            let central_mapping_tree = local_central_mapping_tree_nodes_rewritten_at_most(
                &central_mapping_tree_nodes_at_each_level(previous),
                entries_before,
                allocation_record_tree + accounting_nodes_before + lineage_keys + retirement_keys,
                allocation_record_tree + accounting_tree + lineage_keys + retirement_keys,
                capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
            );
            LocalCheckpointCost {
                allocation_record_tree,
                central_mapping_tree,
                accounting_tree,
                lineage_container_copy_on_write: LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                    + LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                    + lineage_levels,
                retirement_container_copy_on_write:
                    RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                        + RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                        + retirement_levels,
                tree_table: TREE_TABLE_UNITS_PER_PUBLISH_LOCAL,
            }
        }
    }
}

/// 核心层的 κ（`checkpoint_cost_of_the_version_to_build_on`，产品路径的节点容量）与本地那一份。
/// 有树的一版上核心层只拿分配器的几何（各盘单元区），`allocator_of_the_geometry` 取这条历史手里任何一份都一样。
fn checkpoint_cost_reading(
    geometry: Geometry,
    version: VersionToBuildOn<'_>,
    allocator_of_the_geometry: &PoolAllocator,
) -> CheckpointCostReading {
    let (built_on, core) = match version {
        VersionToBuildOn::WithoutFile { allocator, .. } => (
            BuiltOn::VersionWithoutFile,
            checkpoint_cost_of_the_version_to_build_on(None, allocator).0,
        ),
        VersionToBuildOn::WithTrees(previous) => (
            BuiltOn::VersionWithTrees,
            checkpoint_cost_of_the_version_to_build_on(Some(previous), allocator_of_the_geometry).0,
        ),
    };
    CheckpointCostReading {
        built_on,
        core,
        local: local_checkpoint_cost(geometry, version),
    }
}

/// 路一数固定点角色的次数：产品恒数一次；阳性对照 PC7 用只供测试的那一臂把每个固定点角色记两次，S3 那一判必须判红（登记第五节「阳性对照」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RouteOneCount {
    EachFixedPointRoleOnce,
    EachFixedPointRoleTwiceForThePositiveControl,
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

impl FixedPointAccount {
    /// 两条路在每块盘上相同（停机条款 S3）；路二取不出时 `None`。
    fn routes_agree_on_every_device(&self) -> Option<bool> {
        let by_roles = i64::try_from(self.by_roles).expect("槽数装得进 i64");
        self.by_records_per_device
            .as_ref()
            .map(|per_device| per_device.iter().all(|by_records| *by_records == by_roles))
    }

    fn by_records_text(&self) -> String {
        match &self.by_records_per_device {
            Some(per_device) => per_device
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(","),
            None => "records_not_handed_in".to_string(),
        }
    }
}

fn fixed_point_account(
    rewritten: &[TransactionUnit],
    records: Option<&[AllocationRecord]>,
    txg: CheckpointTxg,
    devices: &[DeviceIdentity],
    route_one_count: RouteOneCount,
) -> FixedPointAccount {
    let mut fixed_point_slots_by_roles = 0;
    let mut ordinary_slots = 0;
    let mut instance_switch_slots = 0;
    for role in rewritten {
        match space_budget_of_role(*role) {
            SpaceBudgetOfARole::CheckpointReservePool => {
                fixed_point_slots_by_roles += role.span_slots()
            }
            SpaceBudgetOfARole::OrdinaryAllocation => ordinary_slots += role.span_slots(),
            SpaceBudgetOfARole::InstanceSwitchReserve => instance_switch_slots += role.span_slots(),
        }
    }
    let by_roles = match route_one_count {
        RouteOneCount::EachFixedPointRoleOnce => fixed_point_slots_by_roles,
        RouteOneCount::EachFixedPointRoleTwiceForThePositiveControl => {
            2 * fixed_point_slots_by_roles
        }
    };
    let not_fixed_point_slots =
        i64::try_from(ordinary_slots + instance_switch_slots).expect("槽数装得进 i64");
    let by_records_per_device = records.map(|records_of_the_version| {
        devices
            .iter()
            .map(|device| {
                let allocated_in_this_publish: u64 = records_of_the_version
                    .iter()
                    .filter(|record| {
                        record.device == *device && record.generation == txg && !record.is_released
                    })
                    .map(|record| u64::from(record.span_slots))
                    .sum();
                i64::try_from(allocated_in_this_publish).expect("槽数装得进 i64")
                    - not_fixed_point_slots
            })
            .collect()
    });
    FixedPointAccount {
        by_roles,
        by_records_per_device,
        ordinary_slots,
        instance_switch_slots,
    }
}

/// 发布种类（登记第五节「每一次……的发布记一行：发布种类」），按装置从哪个入口拿到它分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishKind {
    RowOfTheMount,
    WarmUpOfTheMount,
    FloorRaiseOfTheMount,
    DeletionContinuationOfTheMount,
    FloorRaiseOfTheMountsContinuation,
    CompactionContinuationOfTheMount,
    FirstFileVersion,
    SequentialWrite,
    Overwrite,
    FloorRaiseForAUserChange,
    UnmountChain,
}

impl PublishKind {
    fn label(self) -> &'static str {
        match self {
            PublishKind::RowOfTheMount => "row_of_the_mount",
            PublishKind::WarmUpOfTheMount => "warm_up_of_the_mount",
            PublishKind::FloorRaiseOfTheMount => "floor_raise_of_the_mount",
            PublishKind::DeletionContinuationOfTheMount => "deletion_continuation_of_the_mount",
            PublishKind::FloorRaiseOfTheMountsContinuation => {
                "floor_raise_of_the_mounts_continuation"
            }
            PublishKind::CompactionContinuationOfTheMount => "compaction_continuation_of_the_mount",
            PublishKind::FirstFileVersion => "first_file_version",
            PublishKind::SequentialWrite => "sequential_write",
            PublishKind::Overwrite => "overwrite",
            PublishKind::FloorRaiseForAUserChange => "floor_raise_for_a_user_change",
            PublishKind::UnmountChain => "unmount_chain",
        }
    }

    /// 登记第六节 X2 量 c实 的那两种：暖机与卸载那一串（登记第十一节「答不了的」：准入里抬 F 那一串与推空不在里面）。
    fn counts_for_the_actual_empty_publish_cost(self) -> bool {
        match self {
            PublishKind::WarmUpOfTheMount | PublishKind::UnmountChain => true,
            PublishKind::RowOfTheMount
            | PublishKind::FloorRaiseOfTheMount
            | PublishKind::DeletionContinuationOfTheMount
            | PublishKind::FloorRaiseOfTheMountsContinuation
            | PublishKind::CompactionContinuationOfTheMount
            | PublishKind::FirstFileVersion
            | PublishKind::SequentialWrite
            | PublishKind::Overwrite
            | PublishKind::FloorRaiseForAUserChange => false,
        }
    }

    /// 没有普通分配的空发布：A6（c实 ≤ 那一次之前的 κ）与 PC7（空发布普通分配槽数 = 0）判它们。
    fn is_an_empty_publish(self) -> bool {
        match self {
            PublishKind::WarmUpOfTheMount
            | PublishKind::FloorRaiseOfTheMount
            | PublishKind::FloorRaiseOfTheMountsContinuation
            | PublishKind::FloorRaiseForAUserChange
            | PublishKind::UnmountChain => true,
            PublishKind::RowOfTheMount
            | PublishKind::DeletionContinuationOfTheMount
            | PublishKind::CompactionContinuationOfTheMount
            | PublishKind::FirstFileVersion
            | PublishKind::SequentialWrite
            | PublishKind::Overwrite => false,
        }
    }
}

/// 一次发布交回来的那一版：有树的、或树表 0 条的。
#[allow(
    clippy::large_enum_variant,
    reason = "一次调用交回的至多几十版、按值搬一次就记完，装箱只多一层解引用（核心层 PoolVersion 同一个取舍）"
)]
enum PublishedVersion {
    WithTrees(TransactionOutput),
    WithoutFile(VersionWithoutFilePublishOutput),
}

impl PublishedVersion {
    fn of_a_pool_version(version: PoolVersion) -> Self {
        match version {
            PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
                PublishedVersion::WithTrees(output)
            }
            PoolVersion::WithoutFile(output) => PublishedVersion::WithoutFile(output),
        }
    }

    fn txg(&self) -> CheckpointTxg {
        match self {
            PublishedVersion::WithTrees(output) => output.root.checkpoint_txg,
            PublishedVersion::WithoutFile(output) => output.root.checkpoint_txg,
        }
    }

    /// 树表 0 条那一版上的这次发布写下了分配记录树（写行那次）：之后接在这一版后面的发布，本地 κ 带 K1 那一项。
    fn wrote_the_allocation_record_tree_on_a_version_without_file(&self) -> bool {
        match self {
            PublishedVersion::WithTrees(_) => false,
            PublishedVersion::WithoutFile(output) => match &output.allocation_records {
                AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(_) => true,
                AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish => false,
            },
        }
    }

    fn fixed_point_account(
        &self,
        devices: &[DeviceIdentity],
        route_one_count: RouteOneCount,
    ) -> FixedPointAccount {
        match self {
            PublishedVersion::WithTrees(output) => fixed_point_account(
                &output.rewritten,
                Some(output.allocation_records.as_slice()),
                output.root.checkpoint_txg,
                devices,
                route_one_count,
            ),
            PublishedVersion::WithoutFile(output) => {
                let records = match &output.allocation_records {
                    AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(records) => Some(records.as_slice()),
                    AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish => None,
                };
                fixed_point_account(
                    &output.rewritten,
                    records,
                    output.root.checkpoint_txg,
                    devices,
                    route_one_count,
                )
            }
        }
    }
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

impl PublishObservation {
    /// 取得出的空发布的 c实（两条路都在、逐盘相同时取路一）；路二取不出或两条路不同时 `None`。
    fn actual_empty_publish_cost(&self) -> Option<u64> {
        let account = self.fixed_point.as_ref()?;
        match account.routes_agree_on_every_device() {
            Some(true) => Some(account.by_roles),
            Some(false) | None => None,
        }
    }
}

/// 接下来的发布要接在后面的那一版在不在装置手里。
#[allow(
    clippy::large_enum_variant,
    reason = "一条历史只有一个，按值换的次数就是发布次数，装箱只多一层解引用"
)]
enum PreviousVersionInHand {
    WithoutFile {
        allocation_record_tree_written: bool,
    },
    WithTrees(TransactionOutput),
    /// 中间有发布的输出没交回：接在它后面的那一次 κ 取不出。
    NotInHand,
}

/// 一条历史走下来的发布记录器：按 txg 接起每一次发布，缺了的 txg 记「取不出」。
struct HistoryRecorder {
    geometry: Geometry,
    devices: Vec<DeviceIdentity>,
    publishes: Vec<PublishObservation>,
    previous: PreviousVersionInHand,
    last_txg: u64,
    /// 每一次被准入或落点拒（推了抬 F 才发成的也算），按发生的次序：登记第五节 G-min「没有一次被准入或落点拒」。
    refusals: Vec<String>,
    /// 第一次覆盖写那一次 PC7 的读数。
    positive_control_of_the_routes: Option<PositiveControlOfTheRoutes>,
}

/// PC7（登记第五节「阳性对照」）：每个几何的第一次覆盖写上，钩子开着时 S3 判红、钩子关着时两条路相同、普通分配槽数 > 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlOfTheRoutes {
    txg: u64,
    hook_on_routes_agree: Option<bool>,
    hook_off_routes_agree: Option<bool>,
    ordinary_slots: u64,
}

impl HistoryRecorder {
    /// mkfs 刚做完：接下来的发布接在 mkfs 那一版（树表 0 条、没写过行）后面。
    fn after_make_filesystem(geometry: Geometry, make_filesystem_txg: CheckpointTxg) -> Self {
        Self {
            geometry,
            devices: vec![DeviceIdentity(0), DeviceIdentity(1)],
            publishes: Vec::new(),
            previous: PreviousVersionInHand::WithoutFile {
                allocation_record_tree_written: false,
            },
            last_txg: make_filesystem_txg.0,
            refusals: Vec::new(),
            positive_control_of_the_routes: None,
        }
    }

    fn note_refusal(&mut self, what: String) {
        self.refusals.push(what);
    }

    /// 一次调用交回的几版，按 txg 排好接起来；txg 有缺口的，缺的那几次记「取不出」、之后那一次的 κ 也取不出。
    /// `allocator_before_the_call` 只给这次调用的第一次发布在树表 0 条那一版上算 κ（那时分配器还没被这次调用动过），
    /// `allocator_after_the_call` 给同一次调用里后面那几次，也给有树那几版提供几何。调用结束时会话的现行那一版是 `current`：
    /// 它比交回的最后一版新，中间的记「取不出」，再从 `current` 接上。
    fn record_the_versions_of_a_call(
        &mut self,
        call_kind: PublishKind,
        mut versions: Vec<(PublishKind, PublishedVersion)>,
        cycle: u64,
        allocator_before_the_call: &PoolAllocator,
        allocator_after_the_call: &PoolAllocator,
        current: &PoolVersion,
    ) {
        versions.sort_by_key(|(_kind, version)| version.txg().0);
        let mut first_of_the_call = true;
        for (kind, version) in versions {
            let txg = version.txg().0;
            if txg <= self.last_txg {
                // 同一版从两个入口各交回一次：只记第一次。
                continue;
            }
            self.record_the_publishes_not_handed_back_before(txg, call_kind, cycle);
            let allocator_of_a_version_without_file = if first_of_the_call {
                allocator_before_the_call
            } else {
                allocator_after_the_call
            };
            let checkpoint_cost = match &self.previous {
                PreviousVersionInHand::WithoutFile {
                    allocation_record_tree_written,
                } => Some(checkpoint_cost_reading(
                    self.geometry,
                    VersionToBuildOn::WithoutFile {
                        allocation_record_tree_written: *allocation_record_tree_written,
                        allocator: allocator_of_a_version_without_file,
                    },
                    allocator_after_the_call,
                )),
                PreviousVersionInHand::WithTrees(previous) => Some(checkpoint_cost_reading(
                    self.geometry,
                    VersionToBuildOn::WithTrees(previous),
                    allocator_after_the_call,
                )),
                PreviousVersionInHand::NotInHand => None,
            };
            let fixed_point =
                version.fixed_point_account(&self.devices, RouteOneCount::EachFixedPointRoleOnce);
            if kind == PublishKind::Overwrite && self.positive_control_of_the_routes.is_none() {
                let hook_on = version.fixed_point_account(
                    &self.devices,
                    RouteOneCount::EachFixedPointRoleTwiceForThePositiveControl,
                );
                self.positive_control_of_the_routes = Some(PositiveControlOfTheRoutes {
                    txg,
                    hook_on_routes_agree: hook_on.routes_agree_on_every_device(),
                    hook_off_routes_agree: fixed_point.routes_agree_on_every_device(),
                    ordinary_slots: fixed_point.ordinary_slots,
                });
            }
            self.publishes.push(PublishObservation {
                kind,
                cycle,
                txg,
                checkpoint_cost,
                fixed_point: Some(fixed_point),
            });
            let wrote_the_tree =
                version.wrote_the_allocation_record_tree_on_a_version_without_file();
            let written_before = matches!(
                self.previous,
                PreviousVersionInHand::WithoutFile {
                    allocation_record_tree_written: true
                }
            );
            self.previous = match version {
                PublishedVersion::WithTrees(output) => PreviousVersionInHand::WithTrees(output),
                PublishedVersion::WithoutFile(_) => PreviousVersionInHand::WithoutFile {
                    allocation_record_tree_written: written_before || wrote_the_tree,
                },
            };
            self.last_txg = txg;
            first_of_the_call = false;
        }
        let current_txg = current.root().checkpoint_txg.0;
        if current_txg > self.last_txg {
            self.record_the_publishes_not_handed_back_before(current_txg + 1, call_kind, cycle);
            self.previous = match current {
                PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
                    PreviousVersionInHand::WithTrees(output.clone())
                }
                PoolVersion::WithoutFile(_) => PreviousVersionInHand::NotInHand,
            };
        }
    }

    /// `last_txg` 与 `next_txg` 之间没交回的发布各记一行「取不出」（κ 与两条路都取不出），之后那一次的 κ 也取不出。
    fn record_the_publishes_not_handed_back_before(
        &mut self,
        next_txg: u64,
        call_kind: PublishKind,
        cycle: u64,
    ) {
        if next_txg <= self.last_txg + 1 {
            return;
        }
        for txg in self.last_txg + 1..next_txg {
            self.publishes.push(PublishObservation {
                kind: call_kind,
                cycle,
                txg,
                checkpoint_cost: None,
                fixed_point: None,
            });
        }
        self.last_txg = next_txg - 1;
        self.previous = PreviousVersionInHand::NotInHand;
    }
}

/// 一次可写挂载交回的全部发布记下来，交回接着它的会话。取号之前被准入拒、推了抬 F 才过的，记一次被拒（G-min 的「做得完」不收它）。
fn record_the_mount(
    recorder: &mut HistoryRecorder,
    mounted: Mounted,
    cycle: u64,
    allocator_before_the_mount: &PoolAllocator,
) -> MountedSession {
    let (output, session) = MountedSession::of_the_mount(mounted);
    let MountOutput {
        row_publish,
        warm_up_publishes,
        space_admission,
        deletion_queue_continuation,
        floor_raises_pushed_by_the_continuation,
        compaction_continuation,
        ..
    } = output;
    let mount_floor_raises = match space_admission {
        MountSpaceAdmission::AdmittedBeforeAcquisition => Vec::new(),
        MountSpaceAdmission::AdmittedAfterTheFloorRaises { floor_raises, .. } => {
            recorder.note_refusal(format!(
                "mount_cycle{cycle}:refused_before_acquisition_then_admitted_after_{}_floor_raises",
                floor_raises.len()
            ));
            floor_raises
        }
        MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. } => {
            recorder.note_refusal(format!(
                "mount_cycle{cycle}:still_short_after_{}_floor_raises",
                floor_raises.len()
            ));
            floor_raises
        }
        MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => {
            recorder.note_refusal(format!("mount_cycle{cycle}:space_admission_not_judged"));
            Vec::new()
        }
    };
    if !floor_raises_pushed_by_the_continuation.is_empty() {
        recorder.note_refusal(format!(
            "mount_cycle{cycle}:continuation_pushed_{}_floor_raises",
            floor_raises_pushed_by_the_continuation.len()
        ));
    }
    let mut versions = vec![(
        PublishKind::RowOfTheMount,
        PublishedVersion::of_a_pool_version(row_publish),
    )];
    versions.extend(warm_up_publishes.into_iter().map(|version| {
        (
            PublishKind::WarmUpOfTheMount,
            PublishedVersion::of_a_pool_version(version),
        )
    }));
    for raised in mount_floor_raises {
        versions.extend(raised.publishes.into_iter().map(|published| {
            (
                PublishKind::FloorRaiseOfTheMount,
                PublishedVersion::WithTrees(published),
            )
        }));
    }
    versions.extend(deletion_queue_continuation.into_iter().map(|published| {
        (
            PublishKind::DeletionContinuationOfTheMount,
            PublishedVersion::WithTrees(published),
        )
    }));
    for raised in floor_raises_pushed_by_the_continuation {
        versions.extend(raised.publishes.into_iter().map(|published| {
            (
                PublishKind::FloorRaiseOfTheMountsContinuation,
                PublishedVersion::WithTrees(published),
            )
        }));
    }
    if let Some(run) = compaction_continuation.run {
        versions.extend(run.versions_published.into_iter().map(|published| {
            (
                PublishKind::CompactionContinuationOfTheMount,
                PublishedVersion::WithTrees(published),
            )
        }));
    }
    recorder.record_the_versions_of_a_call(
        PublishKind::RowOfTheMount,
        versions,
        cycle,
        allocator_before_the_mount,
        &session.allocator,
        &session.current,
    );
    session
}

/// 一次用户改动交回的全部发布记下来；为它推了抬 F 的，记一次被拒（推之前那一次是被准入或落点拒的）。
fn record_a_user_change(
    recorder: &mut HistoryRecorder,
    call_kind: PublishKind,
    published: UserChangePublished,
    cycle: u64,
    session: &MountedSession,
) {
    let UserChangePublished {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
        ..
    } = published;
    if let Some(first_refusal) = refusals_that_pushed_the_floor_raises.first() {
        recorder.note_refusal(format!(
            "{}_cycle{cycle}:{}_then_{}_floor_raises",
            call_kind.label(),
            variant_name_of(&format!("{first_refusal:?}")),
            floor_raises.len()
        ));
    } else if !floor_raises.is_empty() {
        recorder.note_refusal(format!(
            "{}_cycle{cycle}:{}_floor_raises",
            call_kind.label(),
            floor_raises.len()
        ));
    }
    let mut versions = Vec::new();
    for raised in floor_raises {
        versions.extend(raised.publishes.into_iter().map(|output| {
            (
                PublishKind::FloorRaiseForAUserChange,
                PublishedVersion::WithTrees(output),
            )
        }));
    }
    versions.extend(
        versions_published
            .into_iter()
            .map(|output| (call_kind, PublishedVersion::WithTrees(output))),
    );
    recorder.record_the_versions_of_a_call(
        call_kind,
        versions,
        cycle,
        &session.allocator,
        &session.allocator,
        &session.current,
    );
}

/// 一条历史走多长（登记第五节第 1–4 步）：产物照 [`HistoryPlan::registered_for`]，单测取短的那一条走同一份代码。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HistoryPlan {
    sequentially_written_data_units: u64,
    overwrites_per_cycle: u64,
    cycles: u64,
}

impl HistoryPlan {
    fn registered_for(geometry: Geometry) -> Self {
        Self {
            sequentially_written_data_units: SEQUENTIALLY_WRITTEN_DATA_UNITS,
            overwrites_per_cycle: geometry.overwrites_per_cycle(),
            cycles: CYCLES_OF_OVERWRITE_UNMOUNT_REMOUNT,
        }
    }
}

/// 一条历史停在哪：走完；哪一步没成（步名与原因，原因是错误成员名）；或核心层断言了（panic 的消息）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum HistoryEnd {
    Completed,
    StoppedAt { step: String, reason: String },
    Panicked { message: String },
}

impl HistoryEnd {
    fn text(&self) -> String {
        match self {
            HistoryEnd::Completed => "completed".to_string(),
            HistoryEnd::StoppedAt { step, reason } => format!("stopped_at:{step}:{reason}"),
            HistoryEnd::Panicked { message } => format!(
                "panicked:{}",
                message.split_whitespace().collect::<Vec<_>>().join("_")
            ),
        }
    }
}

fn stopped_at(step: String, reason: String) -> HistoryEnd {
    HistoryEnd::StoppedAt { step, reason }
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

impl HistoryReport {
    /// 登记第五节 G-min「第 1–4 步三圈全部发成（没有一次被准入或落点拒）」。
    fn completes_the_history(&self) -> bool {
        self.end == HistoryEnd::Completed && self.refusals.is_empty()
    }
}

/// 一条历史在走的时候攒下的、记录器之外的读数。
#[derive(Default)]
struct HistoryExtras {
    unit_areas_of_the_allocator: Vec<(u64, u64)>,
    allocation_record_tree_root_level: Option<u8>,
    final_checkpoint_cost: Option<CheckpointCostReading>,
}

/// 走一条历史；核心层断言了（小盘上可能）不让整个进程倒，记成 [`HistoryEnd::Panicked`]。
fn run_the_history(geometry: Geometry, plan: HistoryPlan) -> HistoryReport {
    let mut recorder_slot: Option<HistoryRecorder> = None;
    let mut extras = HistoryExtras::default();
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
        drive_the_history(geometry, plan, &mut recorder_slot, &mut extras)
    }));
    let end = match outcome {
        Ok(Ok(())) => HistoryEnd::Completed,
        Ok(Err(end)) => end,
        Err(payload) => HistoryEnd::Panicked {
            message: payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|text| (*text).to_string())
                })
                .unwrap_or_else(|| "panic_without_a_text_message".to_string()),
        },
    };
    let (refusals, publishes, positive_control_of_the_routes) = match recorder_slot {
        Some(recorder) => (
            recorder.refusals,
            recorder.publishes,
            recorder.positive_control_of_the_routes,
        ),
        None => (Vec::new(), Vec::new(), None),
    };
    HistoryReport {
        geometry,
        end,
        refusals,
        publishes,
        unit_areas_of_the_allocator: extras.unit_areas_of_the_allocator,
        allocation_record_tree_root_level: extras.allocation_record_tree_root_level,
        positive_control_of_the_routes,
        final_checkpoint_cost: extras.final_checkpoint_cost,
    }
}

/// 登记第五节第 1–4 步：mkfs（这个 S）→ 可写挂载（写行 + 暖机）→ 第一个文件版本 → 顺序写 32 个数据单元 →
/// 三圈「覆盖写 4 × 3S 次（`UserChange::Overwrite`）→ 正常卸载 → 再可写挂载」。顺序写要接在带文件的一版后面
/// （`UserChangeRefused::NoFileVersionToChange`），所以先发第一个文件版本（一个数据单元），再顺序写把它换成 32 个数据单元。
/// 哪一步没成交回 `Err`（步名、错误成员名），后面不走；读数照记到停下那一刻。
fn drive_the_history(
    geometry: Geometry,
    plan: HistoryPlan,
    recorder_slot: &mut Option<HistoryRecorder>,
    extras: &mut HistoryExtras,
) -> Result<(), HistoryEnd> {
    let parameters = make_filesystem_parameters(geometry)
        .map_err(|reason| stopped_at("make_filesystem_parameters".to_string(), reason))?;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(
                        geometry.device_bytes(),
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
                    ),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, devices.as_mut_slice()).map_err(|error| {
        stopped_at(
            "make_filesystem".to_string(),
            variant_name_of(&format!("{error:?}")),
        )
    })?;
    let allocator_after_mkfs = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    extras.unit_areas_of_the_allocator = allocator_after_mkfs
        .devices
        .iter()
        .map(|device_map| {
            let start = device_map.unit_area_start().slot().0;
            (start, start + device_map.unit_area_slots())
        })
        .collect();
    let recorder = recorder_slot.insert(HistoryRecorder::after_make_filesystem(
        geometry,
        genesis.root.checkpoint_txg,
    ));
    let mounted = mount_writable(&parameters, &mut devices).map_err(|error| {
        stopped_at(
            "first_mount".to_string(),
            variant_name_of(&format!("{error:?}")),
        )
    })?;
    let mut session = record_the_mount(recorder, mounted, 0, &allocator_after_mkfs);
    let allocator_before_the_first_file = session.allocator.clone();
    let first_file_content = one_data_unit_of_content(0);
    let first_file_version = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut session.allocator,
            session.current.root(),
            FirstFile {
                content: &first_file_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            session.instance,
            session.current.record_bytes(),
        )
    }
    .map_err(|error| {
        stopped_at(
            "first_file_version".to_string(),
            variant_name_of(&format!("{error:?}")),
        )
    })?;
    session.current = PoolVersion::of_a_version_of_trees(first_file_version.clone());
    recorder.record_the_versions_of_a_call(
        PublishKind::FirstFileVersion,
        vec![(
            PublishKind::FirstFileVersion,
            PublishedVersion::WithTrees(first_file_version),
        )],
        0,
        &allocator_before_the_first_file,
        &session.allocator,
        &session.current,
    );
    let sequential_content = sequential_write_content(plan.sequentially_written_data_units);
    let sequential = session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWrite(FirstFile {
                content: &sequential_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            }),
        )
        .map_err(|refusal| {
            stopped_at(
                "sequential_write".to_string(),
                variant_name_of(&format!("{refusal:?}")),
            )
        })?;
    record_a_user_change(
        recorder,
        PublishKind::SequentialWrite,
        sequential,
        0,
        &session,
    );
    for cycle in 1..=plan.cycles {
        for overwrite_index in 0..plan.overwrites_per_cycle {
            let content = one_data_unit_of_content(cycle * 100_000 + overwrite_index + 1);
            let overwrite = session
                .publish_user_change(
                    &mut devices,
                    UserChange::Overwrite(FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS
                            + cycle * 100_000
                            + overwrite_index
                            + 2,
                    }),
                )
                .map_err(|refusal| {
                    stopped_at(
                        format!("overwrite_cycle{cycle}_index{overwrite_index}"),
                        variant_name_of(&format!("{refusal:?}")),
                    )
                })?;
            record_a_user_change(recorder, PublishKind::Overwrite, overwrite, cycle, &session);
        }
        let unmounted = unmount(
            &parameters,
            &mut devices,
            &mut session.allocator,
            &mut session.current,
            session.shadow_ledger,
        )
        .map_err(|error| {
            stopped_at(
                format!("unmount_cycle{cycle}"),
                variant_name_of(&format!("{error:?}")),
            )
        })?;
        let unmount_chain = match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised.publishes,
            Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => {
                return Err(stopped_at(
                    format!("unmount_cycle{cycle}"),
                    "nothing_written_on_a_version_without_file".to_string(),
                ))
            }
        };
        recorder.record_the_versions_of_a_call(
            PublishKind::UnmountChain,
            unmount_chain
                .into_iter()
                .map(|output| {
                    (
                        PublishKind::UnmountChain,
                        PublishedVersion::WithTrees(output),
                    )
                })
                .collect(),
            cycle,
            &session.allocator,
            &session.allocator,
            &session.current,
        );
        let remounted = mount_writable(&parameters, &mut devices).map_err(|error| {
            stopped_at(
                format!("remount_cycle{cycle}"),
                variant_name_of(&format!("{error:?}")),
            )
        })?;
        session = record_the_mount(recorder, remounted, cycle, &allocator_after_mkfs);
    }
    if let PoolVersion::WithFile(last) | PoolVersion::WithTreesAndNoFile(last) = &session.current {
        extras.allocation_record_tree_root_level = Some(
            u8::try_from(
                last.position_addressed_tree_heights_read_from_the_root_node_headers()
                    .allocation_record_tree
                    - 1,
            )
            .expect("根层级装得进 u8"),
        );
        extras.final_checkpoint_cost = Some(checkpoint_cost_reading(
            geometry,
            VersionToBuildOn::WithTrees(last),
            &session.allocator,
        ));
    }
    Ok(())
}

/// 条款读法的保留池 P条 = 2 × 5 + (B − 1) × c_max，c_max 取现算的 κ（登记第七节 A2；D16（发布语义） 已定项 1「`df`」那一行、
/// D28（挂载期承诺量） 已定项 4 末条）。代码读法的保留池 P代 = κ。
fn clause_reading_reserve(checkpoint_cost: u64) -> u64 {
    CLAUSE_RESERVE_CONSTANT_BLOCKS + EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST * checkpoint_cost
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

impl ReserveFit {
    fn code_reserve_holds_the_longest_floor_raise_chain(self) -> bool {
        self.code_empty_publishes_held >= PUBLISHES_OF_THE_LONGEST_FLOOR_RAISE_CHAIN
    }
    fn code_reserve_holds_the_empty_publishes_of_one_admission(self) -> bool {
        self.code_empty_publishes_held >= EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST
    }
    fn clause_reserve_holds_the_longest_floor_raise_chain(self) -> bool {
        self.clause_empty_publishes_held >= PUBLISHES_OF_THE_LONGEST_FLOOR_RAISE_CHAIN
    }
    fn clause_reserve_holds_the_empty_publishes_of_one_admission(self) -> bool {
        self.clause_empty_publishes_held >= EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST
    }
}

/// 一个几何上的 X3：c实 取取得出的空发布里最大的那次（登记第六节 X3「max c实_G」）；一次都取不出、或最大的是 0，交不出（不当 0）。
fn reserve_fit(checkpoint_cost: u64, actual_empty_publish_slots: &[u64]) -> Option<ReserveFit> {
    let largest_actual_empty_publish_slots = actual_empty_publish_slots.iter().copied().max()?;
    if largest_actual_empty_publish_slots == 0 {
        return None;
    }
    let code_reserve = checkpoint_cost;
    let clause_reserve = clause_reading_reserve(checkpoint_cost);
    Some(ReserveFit {
        checkpoint_cost,
        largest_actual_empty_publish_slots,
        code_reserve,
        clause_reserve,
        code_empty_publishes_held: code_reserve / largest_actual_empty_publish_slots,
        clause_empty_publishes_held: clause_reserve / largest_actual_empty_publish_slots,
    })
}

/// 判别力自证（登记第八节）：把门槛挪到观测到的最小、最大两个取样点之间（取 ⌈(最小 + 最大) ÷ 2⌉，落在 (最小, 最大] 里），
/// 用同一个比较（n ≥ 门槛）在每个几何上重判：挪过的门槛下判定有真有假才算「翻得动」（`turns`）；与原门槛下的判定一真一假的几何数另报。
/// 观测到的最小与最大相等时没有「之间」，记翻不动（取样点同值，那一格判据不算数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThresholdDiscrimination {
    observed_minimum: u64,
    observed_maximum: u64,
    moved_threshold: Option<u64>,
    holding_at_the_moved_threshold: usize,
    failing_at_the_moved_threshold: usize,
    changed_from_the_original_threshold: usize,
}

impl ThresholdDiscrimination {
    fn turns(self) -> bool {
        self.holding_at_the_moved_threshold > 0 && self.failing_at_the_moved_threshold > 0
    }
}

fn discrimination_of_a_moved_threshold(
    observed: &[u64],
    original_threshold: u64,
) -> Option<ThresholdDiscrimination> {
    let observed_minimum = observed.iter().copied().min()?;
    let observed_maximum = observed.iter().copied().max()?;
    if observed_minimum == observed_maximum {
        return Some(ThresholdDiscrimination {
            observed_minimum,
            observed_maximum,
            moved_threshold: None,
            holding_at_the_moved_threshold: observed.len(),
            failing_at_the_moved_threshold: 0,
            changed_from_the_original_threshold: 0,
        });
    }
    let moved_threshold = observed_minimum + (observed_maximum - observed_minimum).div_ceil(2);
    let holding_at_the_moved_threshold = observed
        .iter()
        .filter(|held| **held >= moved_threshold)
        .count();
    let changed_from_the_original_threshold = observed
        .iter()
        .filter(|held| (**held >= original_threshold) != (**held >= moved_threshold))
        .count();
    Some(ThresholdDiscrimination {
        observed_minimum,
        observed_maximum,
        moved_threshold: Some(moved_threshold),
        holding_at_the_moved_threshold,
        failing_at_the_moved_threshold: observed.len() - holding_at_the_moved_threshold,
        changed_from_the_original_threshold,
    })
}

/// 一列数的最大、最小、最后一个与个数（产物里报的轨迹三样，登记第八节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SeriesStatistics {
    count: usize,
    maximum: u64,
    minimum: u64,
    last: u64,
}

fn series_statistics(series: &[u64]) -> Option<SeriesStatistics> {
    Some(SeriesStatistics {
        count: series.len(),
        maximum: series.iter().copied().max()?,
        minimum: series.iter().copied().min()?,
        last: *series.last()?,
    })
}

fn statistics_text(statistics: Option<SeriesStatistics>) -> String {
    match statistics {
        Some(statistics) => format!(
            "count={} max={} min={} last={}",
            statistics.count, statistics.maximum, statistics.minimum, statistics.last
        ),
        None => "count=0 max=unobtainable min=unobtainable last=unobtainable".to_string(),
    }
}

/// 产物里定 κ_G 的三种取法（登记修订）：带树那几版上的最小（保留池最小那一刻，主判定）、最大，与最后那一版。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckpointCostOfTheGeometry {
    MinimumOnVersionsWithTrees,
    MaximumOnVersionsWithTrees,
    FinalVersion,
}

impl CheckpointCostOfTheGeometry {
    const ALL: [CheckpointCostOfTheGeometry; 3] = [
        CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees,
        CheckpointCostOfTheGeometry::MaximumOnVersionsWithTrees,
        CheckpointCostOfTheGeometry::FinalVersion,
    ];

    fn label(self) -> &'static str {
        match self {
            CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees => {
                "minimum_on_versions_with_trees"
            }
            CheckpointCostOfTheGeometry::MaximumOnVersionsWithTrees => {
                "maximum_on_versions_with_trees"
            }
            CheckpointCostOfTheGeometry::FinalVersion => "final_version",
        }
    }
}

fn unit_area_text(unit_area: (u64, u64)) -> String {
    format!("[{},{})", unit_area.0, unit_area.1)
}

/// 一个几何的表头行：几何、环长、单元区（本地算的与分配器的）、根层级、K1、历史的结局与被拒次数。
fn geometry_row(report: &HistoryReport) -> String {
    let geometry = report.geometry;
    let local_unit_area = geometry.local_unit_area();
    let allocator_unit_areas = report
        .unit_areas_of_the_allocator
        .iter()
        .map(|unit_area| unit_area_text(*unit_area))
        .collect::<Vec<_>>()
        .join(";");
    let (root_level_text, allocation_record_tree_positions_text) = match report
        .allocation_record_tree_root_level
    {
        Some(root_level) => (
            root_level.to_string(),
            local_allocation_record_tree_positions(root_level, &[local_unit_area, local_unit_area])
                .to_string(),
        ),
        None => ("unobtainable".to_string(), "unobtainable".to_string()),
    };
    format!(
        "name=geometry geometry={} slots_per_region={} pool_size={} device_bytes={} journal_ring_bytes={} local_unit_area={} allocator_unit_areas={} allocation_record_tree_root_level={root_level_text} k1_local={allocation_record_tree_positions_text} end={} refusals={} first_refusal={} publishes={} obtained_publishes={} completes_the_history={}",
        geometry.label(),
        geometry.slots_per_region,
        geometry.pool_size.label(),
        geometry.device_bytes(),
        geometry.journal_ring_bytes(),
        unit_area_text(local_unit_area),
        if allocator_unit_areas.is_empty() { "unobtainable".to_string() } else { allocator_unit_areas },
        report.end.text(),
        report.refusals.len(),
        report.refusals.first().map_or("none", String::as_str),
        report.publishes.len(),
        report.publishes.iter().filter(|observation| observation.fixed_point.is_some()).count(),
        report.completes_the_history(),
    )
}

/// 一次发布一行（登记第五节）：种类、txg、κ（核心层、本地与本地各项）、两条路的固定点、普通分配与实例切换预留的槽数。
fn publish_row(label: &str, observation: &PublishObservation) -> String {
    let checkpoint_cost_fields = match observation.checkpoint_cost {
        Some(cost) => format!(
            "built_on={} kappa_core={} kappa_local={} kappa_local_terms={}",
            cost.built_on.label(),
            cost.core,
            cost.local.total(),
            cost.local.terms_text()
        ),
        None => "built_on=not_in_hand kappa_core=unobtainable kappa_local=unobtainable kappa_local_terms=unobtainable".to_string(),
    };
    let fixed_point_fields = match &observation.fixed_point {
        Some(account) => format!(
            "fixed_point_by_roles={} fixed_point_by_records={} ordinary_slots={} instance_switch_slots={}",
            account.by_roles,
            account.by_records_text(),
            account.ordinary_slots,
            account.instance_switch_slots
        ),
        None => "fixed_point_by_roles=unobtainable fixed_point_by_records=unobtainable ordinary_slots=unobtainable instance_switch_slots=unobtainable".to_string(),
    };
    format!(
        "name=publish geometry={label} cycle={} kind={} txg={} obtained={} {checkpoint_cost_fields} {fixed_point_fields}",
        observation.cycle,
        observation.kind.label(),
        observation.txg,
        observation.fixed_point.is_some(),
    )
}

/// 停机条款 S2（本地 κ 与核心层不同）、S3（两条路不同）、S4 的单元区那一项（本地算的单元区与分配器的不同），与 F3 的 A6
/// （取得出的空发布 c实 > 那一次之前的 κ）。每一处一行，空的就是一处都没有。
fn stop_findings_of_a_geometry(report: &HistoryReport) -> Vec<String> {
    let label = report.geometry.label();
    let mut findings = Vec::new();
    let local_unit_area = report.geometry.local_unit_area();
    if report
        .unit_areas_of_the_allocator
        .iter()
        .any(|unit_area| *unit_area != local_unit_area)
    {
        findings.push(format!(
            "name=stop clause=s4_unit_area_differs geometry={label} local={} allocator={}",
            unit_area_text(local_unit_area),
            report
                .unit_areas_of_the_allocator
                .iter()
                .map(|unit_area| unit_area_text(*unit_area))
                .collect::<Vec<_>>()
                .join(";")
        ));
    }
    let final_reading = report.final_checkpoint_cost.map(|cost| (u64::MAX, cost));
    let readings = report
        .publishes
        .iter()
        .filter_map(|observation| {
            observation
                .checkpoint_cost
                .map(|cost| (observation.txg, cost))
        })
        .chain(final_reading);
    for (txg, cost) in readings {
        if cost.core != cost.local.total() {
            findings.push(format!(
                "name=stop clause=s2_local_kappa_differs_from_core geometry={label} before_txg={} built_on={} core={} local={} local_terms={}",
                if txg == u64::MAX { "after_the_last".to_string() } else { txg.to_string() },
                cost.built_on.label(),
                cost.core,
                cost.local.total(),
                cost.local.terms_text()
            ));
        }
    }
    for observation in &report.publishes {
        let Some(account) = &observation.fixed_point else {
            continue;
        };
        if account.routes_agree_on_every_device() == Some(false) {
            findings.push(format!(
                "name=stop clause=s3_routes_differ geometry={label} kind={} txg={} fixed_point_by_roles={} fixed_point_by_records={} ordinary_slots={} instance_switch_slots={}",
                observation.kind.label(),
                observation.txg,
                account.by_roles,
                account.by_records_text(),
                account.ordinary_slots,
                account.instance_switch_slots
            ));
        }
        if let (true, Some(actual), Some(cost)) = (
            observation.kind.is_an_empty_publish(),
            observation.actual_empty_publish_cost(),
            observation.checkpoint_cost,
        ) {
            if actual > cost.core {
                findings.push(format!(
                    "name=stop clause=a6_actual_empty_publish_exceeds_kappa geometry={label} kind={} txg={} actual={actual} kappa_core={}",
                    observation.kind.label(),
                    observation.txg,
                    cost.core
                ));
            }
        }
    }
    findings
}

/// PC7 那一行（登记第五节「阳性对照」）与它判不判作废（第十节 F1 = 第十一节 V1）：钩子开着两条路必须不同、钩子关着必须相同、
/// 那一次普通分配槽数 > 0；每一次取得出的空发布普通分配槽数 = 0。
fn positive_control_row_and_holds(report: &HistoryReport) -> (String, bool) {
    let label = report.geometry.label();
    let empty_publishes_with_ordinary_slots = report
        .publishes
        .iter()
        .filter(|observation| observation.kind.is_an_empty_publish())
        .filter(|observation| {
            observation
                .fixed_point
                .as_ref()
                .is_some_and(|account| account.ordinary_slots > 0)
        })
        .count();
    match report.positive_control_of_the_routes {
        Some(control) => {
            let holds = control.hook_on_routes_agree == Some(false)
                && control.hook_off_routes_agree == Some(true)
                && control.ordinary_slots > 0
                && empty_publishes_with_ordinary_slots == 0;
            (
                format!(
                    "name=positive_control_routes geometry={label} first_overwrite_txg={} hook_on_routes_agree={} hook_off_routes_agree={} ordinary_slots={} empty_publishes_with_ordinary_slots={empty_publishes_with_ordinary_slots} holds={holds}",
                    control.txg,
                    optional_bool_text(control.hook_on_routes_agree),
                    optional_bool_text(control.hook_off_routes_agree),
                    control.ordinary_slots,
                ),
                holds,
            )
        }
        None => (
            format!(
                "name=positive_control_routes geometry={label} first_overwrite_txg=unobtainable empty_publishes_with_ordinary_slots={empty_publishes_with_ordinary_slots} holds=false"
            ),
            false,
        ),
    }
}

fn optional_bool_text(value: Option<bool>) -> String {
    value.map_or_else(|| "unobtainable".to_string(), |value| value.to_string())
}

/// 一个几何上 X3 与它的三个判定（按 κ_G 的三种取法各一份），给判别力自证与几何敏感性用。
struct GeometryVerdicts {
    geometry: Geometry,
    fits: Vec<(CheckpointCostOfTheGeometry, ReserveFit)>,
}

/// 一个几何的汇总行（登记第六节 X1–X5、第八节 κ 的轨迹）与判定行。取得出的空发布少于 6 次的，打一行作废（V6）、不出判定。
fn emit_the_summary_of_a_geometry(report: &HistoryReport) -> Option<GeometryVerdicts> {
    let label = report.geometry.label();
    let kinds_seen: Vec<PublishKind> = {
        let mut kinds = Vec::new();
        for observation in &report.publishes {
            if !kinds.contains(&observation.kind) {
                kinds.push(observation.kind);
            }
        }
        kinds
    };
    for kind in &kinds_seen {
        let kappa_series: Vec<u64> = report
            .publishes
            .iter()
            .filter(|observation| observation.kind == *kind)
            .filter_map(|observation| observation.checkpoint_cost.map(|cost| cost.core))
            .collect();
        emit(&format!(
            "name=x1_kappa geometry={label} kind={} {}",
            kind.label(),
            statistics_text(series_statistics(&kappa_series))
        ));
    }
    let kappa_on_versions_with_trees: Vec<u64> = report
        .publishes
        .iter()
        .filter_map(|observation| observation.checkpoint_cost)
        .filter(|cost| cost.built_on == BuiltOn::VersionWithTrees)
        .map(|cost| cost.core)
        .collect();
    emit(&format!(
        "name=x1_kappa geometry={label} kind=all_built_on_versions_with_trees {}",
        statistics_text(series_statistics(&kappa_on_versions_with_trees))
    ));
    let final_kappa = report.final_checkpoint_cost.map(|cost| cost.core);
    let first_file_version_kappa = report
        .publishes
        .iter()
        .skip_while(|observation| observation.kind != PublishKind::FirstFileVersion)
        .nth(1)
        .and_then(|observation| observation.checkpoint_cost)
        .map(|cost| cost.core);
    let peak = kappa_on_versions_with_trees
        .iter()
        .copied()
        .chain(final_kappa)
        .max();
    let publishes_above_the_first_file_version = first_file_version_kappa.map(|first| {
        kappa_on_versions_with_trees
            .iter()
            .filter(|kappa| **kappa > first)
            .count()
    });
    emit(&format!(
        "name=kappa_trajectory geometry={label} peak={} first_file_version_kappa={} publishes_built_on_a_version_above_it={} final={}",
        peak.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
        first_file_version_kappa.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
        publishes_above_the_first_file_version.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
        final_kappa.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
    ));
    let mut actual_empty_publish_slots = Vec::new();
    for kind in kinds_seen
        .iter()
        .copied()
        .filter(|kind| kind.counts_for_the_actual_empty_publish_cost())
    {
        let of_this_kind: Vec<&PublishObservation> = report
            .publishes
            .iter()
            .filter(|observation| observation.kind == kind)
            .collect();
        let obtained: Vec<u64> = of_this_kind
            .iter()
            .filter_map(|observation| observation.actual_empty_publish_cost())
            .collect();
        emit(&format!(
            "name=x2_actual_empty_publish_cost geometry={label} kind={} not_obtained={} {}",
            kind.label(),
            of_this_kind.len() - obtained.len(),
            statistics_text(series_statistics(&obtained))
        ));
        actual_empty_publish_slots.extend(obtained);
    }
    let overwrite_fixed_points: Vec<u64> = report
        .publishes
        .iter()
        .filter(|observation| observation.kind == PublishKind::Overwrite)
        .filter_map(|observation| {
            let account = observation.fixed_point.as_ref()?;
            (account.routes_agree_on_every_device() == Some(true)).then_some(account.by_roles)
        })
        .collect();
    let largest_actual = actual_empty_publish_slots.iter().copied().max();
    let largest_overwrite = overwrite_fixed_points.iter().copied().max();
    let overwrite_excess = match (largest_overwrite, largest_actual) {
        (Some(overwrite), Some(actual)) => (
            overwrite > actual,
            overwrite.saturating_sub(actual).to_string(),
        ),
        (None, _) | (_, None) => (false, "unobtainable".to_string()),
    };
    emit(&format!(
        "name=x5_overwrite_fixed_point geometry={label} {} exceeds_the_largest_actual_empty_publish={} excess_slots={}",
        statistics_text(series_statistics(&overwrite_fixed_points)),
        overwrite_excess.0,
        overwrite_excess.1,
    ));
    let obtained_empty_publishes = actual_empty_publish_slots.len();
    if obtained_empty_publishes < OBTAINED_EMPTY_PUBLISHES_AT_LEAST {
        emit(&format!(
            "name=geometry_void clause=v6_fewer_than_six_obtained_empty_publishes geometry={label} obtained_empty_publishes={obtained_empty_publishes}"
        ));
        return None;
    }
    let kappa_minimum = kappa_on_versions_with_trees.iter().copied().min();
    let kappa_maximum = kappa_on_versions_with_trees.iter().copied().max();
    let mut fits = Vec::new();
    for reading in CheckpointCostOfTheGeometry::ALL {
        let kappa_of_this_reading = match reading {
            CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees => kappa_minimum,
            CheckpointCostOfTheGeometry::MaximumOnVersionsWithTrees => kappa_maximum,
            CheckpointCostOfTheGeometry::FinalVersion => final_kappa,
        };
        let Some(fit) =
            kappa_of_this_reading.and_then(|kappa| reserve_fit(kappa, &actual_empty_publish_slots))
        else {
            emit(&format!(
                "name=x3_reserve_fit geometry={label} kappa_of_the_geometry={} kappa=unobtainable",
                reading.label()
            ));
            continue;
        };
        emit(&format!(
            "name=x3_reserve_fit geometry={label} kappa_of_the_geometry={} kappa={} largest_actual_empty_publish_slots={} code_reserve={} clause_reserve={} code_empty_publishes_held={} clause_empty_publishes_held={}",
            reading.label(),
            fit.checkpoint_cost,
            fit.largest_actual_empty_publish_slots,
            fit.code_reserve,
            fit.clause_reserve,
            fit.code_empty_publishes_held,
            fit.clause_empty_publishes_held,
        ));
        for (device_number, (first_slot, end_slot)) in report
            .unit_areas_of_the_allocator
            .iter()
            .copied()
            .enumerate()
        {
            let unit_area_slots = end_slot - first_slot;
            let code_millionths =
                u128::from(fit.code_reserve) * 1_000_000 / u128::from(unit_area_slots);
            let clause_millionths =
                u128::from(fit.clause_reserve) * 1_000_000 / u128::from(unit_area_slots);
            emit(&format!(
                "name=x4_reserve_share geometry={label} kappa_of_the_geometry={} device={device_number} unit_area_slots={unit_area_slots} code_reserve_millionths={code_millionths} clause_reserve_millionths={clause_millionths} difference_millionths={}",
                reading.label(),
                clause_millionths - code_millionths,
            ));
        }
        emit(&format!(
            "name=verdict geometry={label} kappa_of_the_geometry={} code_reserve_holds_the_longest_floor_raise_chain={} code_reserve_holds_the_empty_publishes_of_one_admission={} clause_reserve_holds_the_longest_floor_raise_chain={} clause_reserve_holds_the_empty_publishes_of_one_admission={} largest_actual_empty_publish_within_kappa={}",
            reading.label(),
            fit.code_reserve_holds_the_longest_floor_raise_chain(),
            fit.code_reserve_holds_the_empty_publishes_of_one_admission(),
            fit.clause_reserve_holds_the_longest_floor_raise_chain(),
            fit.clause_reserve_holds_the_empty_publishes_of_one_admission(),
            fit.largest_actual_empty_publish_slots <= fit.checkpoint_cost,
        ));
        fits.push((reading, fit));
    }
    Some(GeometryVerdicts {
        geometry: report.geometry,
        fits,
    })
}

/// 判别力自证与几何敏感性（登记第八节）：X3 两个门槛（3 与 7）在两种读法上各挪一次；每个判定在 G-min 对 G-4（同一个 S）、
/// S = 4 对 S = 16（同一档池大小）上各比一次，一真一假记 `stable=false`。都只用 κ_G 取带树那几版上的最小那一种（主判定）。
fn emit_discrimination_and_stability(verdicts: &[GeometryVerdicts]) {
    let primary: Vec<(Geometry, ReserveFit)> = verdicts
        .iter()
        .filter_map(|geometry_verdicts| {
            geometry_verdicts
                .fits
                .iter()
                .find(|(reading, _fit)| {
                    *reading == CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees
                })
                .map(|(_reading, fit)| (geometry_verdicts.geometry, *fit))
        })
        .collect();
    let code_held: Vec<u64> = primary
        .iter()
        .map(|(_geometry, fit)| fit.code_empty_publishes_held)
        .collect();
    let clause_held: Vec<u64> = primary
        .iter()
        .map(|(_geometry, fit)| fit.clause_empty_publishes_held)
        .collect();
    for (reading_name, held) in [("code", &code_held), ("clause", &clause_held)] {
        for original_threshold in [
            PUBLISHES_OF_THE_LONGEST_FLOOR_RAISE_CHAIN,
            EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST,
        ] {
            match discrimination_of_a_moved_threshold(held, original_threshold) {
                Some(discrimination) => emit(&format!(
                    "name=discrimination item=x3_{reading_name}_reserve original_threshold={original_threshold} geometries={} observed_minimum={} observed_maximum={} moved_threshold={} holding_at_the_moved_threshold={} failing_at_the_moved_threshold={} changed_from_the_original_threshold={} turns={}",
                    held.len(),
                    discrimination.observed_minimum,
                    discrimination.observed_maximum,
                    discrimination.moved_threshold.map_or_else(|| "none_sample_points_equal".to_string(), |threshold| threshold.to_string()),
                    discrimination.holding_at_the_moved_threshold,
                    discrimination.failing_at_the_moved_threshold,
                    discrimination.changed_from_the_original_threshold,
                    discrimination.turns(),
                )),
                None => emit(&format!(
                    "name=discrimination item=x3_{reading_name}_reserve original_threshold={original_threshold} geometries=0"
                )),
            }
        }
    }
    type VerdictOfAFit = fn(ReserveFit) -> bool;
    let verdict_functions: [(&str, VerdictOfAFit); 4] = [
        (
            "code_reserve_holds_the_longest_floor_raise_chain",
            ReserveFit::code_reserve_holds_the_longest_floor_raise_chain,
        ),
        (
            "code_reserve_holds_the_empty_publishes_of_one_admission",
            ReserveFit::code_reserve_holds_the_empty_publishes_of_one_admission,
        ),
        (
            "clause_reserve_holds_the_longest_floor_raise_chain",
            ReserveFit::clause_reserve_holds_the_longest_floor_raise_chain,
        ),
        (
            "clause_reserve_holds_the_empty_publishes_of_one_admission",
            ReserveFit::clause_reserve_holds_the_empty_publishes_of_one_admission,
        ),
    ];
    let fit_of = |geometry: Geometry| {
        primary
            .iter()
            .find(|(candidate, _fit)| *candidate == geometry)
            .map(|(_geometry, fit)| *fit)
    };
    let mut pairs: Vec<(Geometry, Geometry)> = Vec::new();
    for (left, _left_fit) in &primary {
        for (right, _right_fit) in &primary {
            let smallest_against_four_gibibytes = left.slots_per_region == right.slots_per_region
                && matches!(left.pool_size, PoolSize::SmallestTrial { .. })
                && right.pool_size == PoolSize::FourGibibytes;
            let four_against_sixteen = left.slots_per_region
                == ROOT_RING_SLOTS_PER_REGION_MINIMUM_LOCAL
                && right.slots_per_region == ROOT_RING_SLOTS_PER_REGION_MAXIMUM_LOCAL
                && same_pool_size_band(left.pool_size, right.pool_size);
            if smallest_against_four_gibibytes || four_against_sixteen {
                pairs.push((*left, *right));
            }
        }
    }
    for (verdict_name, verdict_of) in verdict_functions {
        for (left, right) in &pairs {
            let (Some(left_fit), Some(right_fit)) = (fit_of(*left), fit_of(*right)) else {
                continue;
            };
            let (left_verdict, right_verdict) = (verdict_of(left_fit), verdict_of(right_fit));
            emit(&format!(
                "name=stability item={verdict_name} pair={}~{} left={left_verdict} right={right_verdict} stable={}",
                left.label(),
                right.label(),
                left_verdict == right_verdict,
            ));
        }
    }
}

/// 两个几何的池大小是不是同一档（G-min 的各次试跑算一档）。
fn same_pool_size_band(left: PoolSize, right: PoolSize) -> bool {
    match (left, right) {
        (PoolSize::SmallestTrial { .. }, PoolSize::SmallestTrial { .. })
        | (PoolSize::OneGibibyte, PoolSize::OneGibibyte)
        | (PoolSize::FourGibibytes, PoolSize::FourGibibytes) => true,
        (PoolSize::SmallestTrial { .. }, PoolSize::OneGibibyte | PoolSize::FourGibibytes)
        | (PoolSize::OneGibibyte, PoolSize::SmallestTrial { .. } | PoolSize::FourGibibytes)
        | (PoolSize::FourGibibytes, PoolSize::SmallestTrial { .. } | PoolSize::OneGibibyte) => {
            false
        }
    }
}

/// 停机条款 S4：本地常量与 `crates/` 同名常量逐个回比，一行一个（全等才往下跑）。
fn check_local_constants_against_the_crates() -> bool {
    let comparisons: [(&str, u64, u64); 16] = [
        ("slot_bytes", SLOT_BYTES_LOCAL, singlefs_format::SLOT_BYTES),
        (
            "data_unit_bytes",
            DATA_UNIT_BYTES_LOCAL,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "data_unit_payload_bytes",
            DATA_UNIT_PAYLOAD_BYTES_LOCAL,
            u64::try_from(singlefs_core::unit::data_unit_payload_capacity())
                .expect("载荷字节数装得进 u64"),
        ),
        (
            "allocation_record_tree_leaf_slots",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        (
            "allocation_record_tree_internal_fanout",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        (
            "root_ring_regions",
            ROOT_RING_REGIONS_LOCAL,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        (
            "root_ring_slots_per_region_minimum",
            ROOT_RING_SLOTS_PER_REGION_MINIMUM_LOCAL,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        ),
        (
            "root_ring_slots_per_region_maximum",
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM_LOCAL,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        ),
        (
            "default_journal_ring_bytes",
            DEFAULT_JOURNAL_RING_BYTES_LOCAL,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "unit_area_start_slot_of_the_default_ring",
            UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "publishes_per_admission_at_most",
            PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL,
            u64::try_from(singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST)
                .expect("次数装得进 u64"),
        ),
        (
            "lineage_container_blocks_of_one_cow",
            LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::LINEAGE_CONTAINER_BLOCKS_OF_ONE_COW,
        ),
        (
            "lineage_tree_root_blocks_of_one_cow",
            LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COW,
        ),
        (
            "retirement_container_blocks_of_one_cow",
            RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COW,
        ),
        (
            "retirement_tree_root_blocks_of_one_cow",
            RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL,
            singlefs_core::admission::RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COW,
        ),
        (
            "unit_area_start_slot_of_the_default_ring_from_the_ring_start",
            JOURNAL_RING_START_SLOT_LOCAL + DEFAULT_JOURNAL_RING_BYTES_LOCAL / SLOT_BYTES_LOCAL,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
    ];
    let mut every_constant_equal = true;
    for (name, local, crates) in comparisons {
        let equal = local == crates;
        every_constant_equal &= equal;
        emit(&format!(
            "name=constant item={name} local={local} crates={crates} equal={equal}"
        ));
    }
    every_constant_equal
}

/// 乙类锚点里装置自己算得出的几项（登记第七节 B8 之外：B10 的 529 与 5、A2 的条款读法保留池、B11 钉 `reserve_fit.py` 的两组输出），
/// 一行一个；不等即作废（第十一节 V3）。
fn check_the_anchors() -> bool {
    let narrow_unit_area = |width: u64| {
        (
            UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL,
            UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL + width,
        )
    };
    let four_gibibyte_unit_area = (
        UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING_LOCAL,
        4 * GIBIBYTE / SLOT_BYTES_LOCAL,
    );
    let reserve_fit_with_five = reserve_fit(5, &[5]).expect("最大的空发布 5 槽");
    let reserve_fit_with_529 = reserve_fit(529, &[5]).expect("最大的空发布 5 槽");
    let anchors: [(&str, u64, u64); 12] = [
        (
            "b10_k1_two_four_gibibyte_root_level_two",
            local_allocation_record_tree_positions(
                2,
                &[four_gibibyte_unit_area, four_gibibyte_unit_area],
            ),
            529,
        ),
        (
            "b10_k1_two_narrow_240_root_level_one",
            local_allocation_record_tree_positions(
                1,
                &[narrow_unit_area(240), narrow_unit_area(240)],
            ),
            5,
        ),
        (
            "b10_k1_two_narrow_256_root_level_one",
            local_allocation_record_tree_positions(
                1,
                &[narrow_unit_area(256), narrow_unit_area(256)],
            ),
            5,
        ),
        (
            "b10_k1_two_narrow_384_root_level_one",
            local_allocation_record_tree_positions(
                1,
                &[narrow_unit_area(384), narrow_unit_area(384)],
            ),
            5,
        ),
        (
            "a2_clause_reserve_at_kappa_five",
            clause_reading_reserve(5),
            45,
        ),
        (
            "a2_clause_reserve_at_kappa_529",
            clause_reading_reserve(529),
            3713,
        ),
        (
            "b11_code_empties_held_c5_kappa5",
            reserve_fit_with_five.code_empty_publishes_held,
            1,
        ),
        (
            "b11_clause_empties_held_c5_kappa5",
            reserve_fit_with_five.clause_empty_publishes_held,
            9,
        ),
        (
            "b11_code_empties_held_c5_kappa529",
            reserve_fit_with_529.code_empty_publishes_held,
            105,
        ),
        (
            "b11_clause_empties_held_c5_kappa529",
            reserve_fit_with_529.clause_empty_publishes_held,
            742,
        ),
        (
            "a1_publishes_per_admission_at_most",
            PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL,
            8,
        ),
        (
            "local_root_level_of_two_four_gibibyte_devices",
            u64::from(local_root_level_of_the_pool_geometry(&[
                4 * GIBIBYTE / SLOT_BYTES_LOCAL,
                4 * GIBIBYTE / SLOT_BYTES_LOCAL,
            ])),
            2,
        ),
    ];
    let mut every_anchor_equal = true;
    for (name, value, expected) in anchors {
        let equal = value == expected;
        every_anchor_equal &= equal;
        emit(&format!(
            "name=anchor item={name} value={value} expected={expected} equal={equal}"
        ));
    }
    every_anchor_equal
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
        std::env::var(singlefs_harness::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE).ok();
    singlefs_harness::heavy_threads::own_variable_or_default_heavy_threads(
        "E173_THREADS",
        std::env::var("E173_THREADS").ok().as_deref(),
        thread_cap_value.as_deref(),
    )
    .get()
}

/// 往 `SINGLEFS_SERIAL_DECLARATIONS` 指的文件追加一条声明（写法在 `scripts/hosts/core_usage.py` 文件头「声明」）；变量没设不写。
fn declare(declaration: &str) {
    let Ok(path) = std::env::var("SINGLEFS_SERIAL_DECLARATIONS") else {
        return;
    };
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since_the_epoch| since_the_epoch.as_secs());
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let written =
            std::io::Write::write_all(&mut file, format!("{seconds}\t{declaration}\n").as_bytes());
        std::hint::black_box(written.is_ok());
    }
}

/// 全部几何按工作队列分到 `threads` 条线程上（几何之间互相独立；同一个几何的历史是一串接着上一版的发布，只能按次序）。
/// 读数按几何的次序收回，产物由主线程按这个次序打，所以不随线程数变。只剩最后一个几何在跑时声明串行，跑完声明并行。
fn run_every_geometry(geometries: &[Geometry], threads: usize) -> Vec<HistoryReport> {
    let next_index = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let declared_serial = Mutex::new(false);
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
                if workers > 1 && geometries.len() - finished_now == 1 {
                    let mut declared = declared_serial.lock().expect("声明的锁没被弄坏");
                    if !*declared {
                        declare("此刻串行：只剩最后一个几何的历史在跑，它是一串接着上一版的发布，只能按次序做");
                        *declared = true;
                    }
                }
            });
        }
    });
    if *declared_serial.lock().expect("声明的锁没被弄坏") {
        declare("此刻并行");
    }
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
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source =
        format!("{manifest_directory}/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
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
mod tests {
    use super::*;

    /// 单测走的短历史：G-1（两块 1 GiB）、S = 4，顺序写 3 个单元、覆盖写 2 次、卸载再挂一圈。与产物走同一份 [`run_the_history`]，
    /// 只把长度缩短（登记第九节「执行员先在单测里核这一格在原式与变异上确实不同」）。
    fn short_history() -> HistoryReport {
        run_the_history(
            Geometry {
                slots_per_region: 4,
                pool_size: PoolSize::OneGibibyte,
            },
            HistoryPlan {
                sequentially_written_data_units: 3,
                overwrites_per_cycle: 2,
                cycles: 1,
            },
        )
    }

    #[test]
    fn allocation_record_tree_positions_are_529_on_two_four_gibibyte_unit_areas_and_5_on_the_narrow_ones(
    ) {
        let four_gibibytes = (50176, 262_144);
        assert_eq!(
            local_allocation_record_tree_positions(2, &[four_gibibytes, four_gibibytes]),
            529,
            "B10：两块 4 GiB 盘、单元区 [50176, 262144)、根层级 2 是 529（k1_positions.py 的 two 4GiB 那一行）"
        );
        for width in [240, 256, 384] {
            let narrow = (50176, 50176 + width);
            assert_eq!(
                local_allocation_record_tree_positions(1, &[narrow, narrow]),
                5,
                "B10：单元区 {width} 槽的窄盘、根层级 1 是 5"
            );
        }
    }

    #[test]
    fn allocation_record_tree_positions_are_341_on_two_unit_areas_from_slot_zero_to_137001_at_root_level_two(
    ) {
        let from_slot_zero = (0, 137_001);
        assert_eq!(
            local_allocation_record_tree_positions(2, &[from_slot_zero, from_slot_zero]),
            341,
            "两块盘 [0, 137001)、根层级 2：层级 1 的跨度 812 × 169 = 137228 只罩一格（扇出取 168 时跨度 136416、罩两格，变成 343）"
        );
    }

    #[test]
    fn local_constants_equal_the_crates_constants() {
        assert!(
            check_local_constants_against_the_crates(),
            "停机条款 S4：本地常量与 crates 同名常量逐个相等（不等的那一行 equal=false）"
        );
    }

    #[test]
    fn the_anchors_computed_by_the_device_equal_the_registered_values() {
        assert!(
            check_the_anchors(),
            "作废条款 V3：B10、A2、B11 的锚点逐个相等（不等的那一行 equal=false）"
        );
    }

    #[test]
    fn the_local_root_level_is_two_on_four_gibibyte_devices_and_one_on_one_gibibyte_devices() {
        let slots_of = |device_bytes: u64| device_bytes / SLOT_BYTES_LOCAL;
        assert_eq!(
            local_root_level_of_the_pool_geometry(&[
                slots_of(4 * GIBIBYTE),
                slots_of(4 * GIBIBYTE)
            ]),
            2
        );
        assert_eq!(
            local_root_level_of_the_pool_geometry(&[slots_of(GIBIBYTE), slots_of(GIBIBYTE)]),
            1,
            "1 GiB 盘 65536 槽，每块盘 ⌈65536 ÷ 812⌉ = 81 格，两块 162 ≤ 169"
        );
    }

    #[test]
    fn the_journal_ring_is_the_default_on_four_gibibytes_and_a_quarter_of_the_device_below_three_gibibytes(
    ) {
        assert_eq!(journal_ring_bytes_of_a_device(4 * GIBIBYTE), 805_306_368);
        assert_eq!(journal_ring_bytes_of_a_device(GIBIBYTE), 256 * MEBIBYTE);
        assert_eq!(journal_ring_bytes_of_a_device(32 * MEBIBYTE), 8 * MEBIBYTE);
        let four_gibibytes = Geometry {
            slots_per_region: 8,
            pool_size: PoolSize::FourGibibytes,
        };
        assert_eq!(
            four_gibibytes.local_unit_area(),
            (50176, 262_144),
            "B10：G-4 的单元区 [50176, 262144)"
        );
    }

    #[test]
    fn the_short_history_completes_and_the_local_kappa_equals_the_core_kappa_on_every_publish() {
        let report = short_history();
        assert_eq!(report.end, HistoryEnd::Completed, "短历史走完");
        let readings: Vec<CheckpointCostReading> = report
            .publishes
            .iter()
            .filter_map(|observation| observation.checkpoint_cost)
            .chain(report.final_checkpoint_cost)
            .collect();
        assert!(
            readings
                .iter()
                .filter(|reading| reading.built_on == BuiltOn::VersionWithTrees)
                .count()
                >= 8,
            "短历史里接在有树那几版后面的发布至少 8 次：{readings:?}"
        );
        for reading in &readings {
            assert_eq!(
                reading.core,
                reading.local.total(),
                "停机条款 S2：本地 κ（{}）等于核心层 κ",
                reading.local.terms_text()
            );
        }
    }

    #[test]
    fn both_routes_agree_on_every_device_on_every_obtained_publish_of_a_short_history() {
        let report = short_history();
        let compared: Vec<&PublishObservation> = report
            .publishes
            .iter()
            .filter(|observation| {
                observation
                    .fixed_point
                    .as_ref()
                    .is_some_and(|account| account.by_records_per_device.is_some())
            })
            .collect();
        for kind in [
            PublishKind::SequentialWrite,
            PublishKind::Overwrite,
            PublishKind::UnmountChain,
            PublishKind::WarmUpOfTheMount,
        ] {
            assert!(
                compared
                    .iter()
                    .filter(|observation| observation.kind == kind)
                    .count()
                    >= 1,
                "两条路都取得出的发布里有 {} 那一种",
                kind.label()
            );
        }
        for observation in compared {
            let account = observation.fixed_point.as_ref().expect("只留了取得出的");
            assert_eq!(
                account.routes_agree_on_every_device(),
                Some(true),
                "停机条款 S3：txg {} 的 {} 两条路逐盘相同：路一 {}、路二 {}",
                observation.txg,
                observation.kind.label(),
                account.by_roles,
                account.by_records_text()
            );
        }
    }

    #[test]
    fn the_route_doubling_hook_turns_the_route_check_red_on_the_first_overwrite_of_a_short_history()
    {
        let report = short_history();
        let control = report
            .positive_control_of_the_routes
            .expect("短历史有覆盖写，PC7 有读数");
        assert_eq!(
            control.hook_on_routes_agree,
            Some(false),
            "PC7：钩子开着时 S3 判红"
        );
        assert_eq!(
            control.hook_off_routes_agree,
            Some(true),
            "PC7：钩子关着时两条路相同"
        );
        assert!(control.ordinary_slots > 0, "PC7：覆盖写的普通分配槽数 > 0");
        assert!(
            positive_control_row_and_holds(&report).1,
            "PC7 那一行 holds=true"
        );
    }

    #[test]
    fn every_empty_publish_of_a_short_history_writes_no_ordinary_slot_and_no_more_than_kappa() {
        let report = short_history();
        let mut empty_publishes_checked = 0;
        for observation in report
            .publishes
            .iter()
            .filter(|observation| observation.kind.is_an_empty_publish())
        {
            let Some(account) = &observation.fixed_point else {
                continue;
            };
            assert_eq!(
                account.ordinary_slots, 0,
                "txg {} 的空发布没有普通分配",
                observation.txg
            );
            if let (Some(actual), Some(cost)) = (
                observation.actual_empty_publish_cost(),
                observation.checkpoint_cost,
            ) {
                assert!(
                    actual <= cost.core,
                    "A6：txg {} 的空发布新占 {actual} 槽不超过之前那一版的 κ {}",
                    observation.txg,
                    cost.core
                );
                empty_publishes_checked += 1;
            }
        }
        assert!(
            empty_publishes_checked >= 3,
            "短历史里取得出的空发布至少 3 次（卸载那一串 2 或 3 次、再挂的暖机）"
        );
    }

    #[test]
    fn the_reserve_fit_divides_by_the_largest_actual_empty_publish_and_a_floor_raise_chain_needs_three(
    ) {
        let fit = reserve_fit(14, &[3, 5]).expect("取得出两次空发布");
        assert_eq!(
            fit.largest_actual_empty_publish_slots, 5,
            "X3 取 c实 的最大值"
        );
        assert_eq!(
            fit.code_empty_publishes_held, 2,
            "⌊14 ÷ 5⌋ = 2（取最小值 3 会是 4）"
        );
        assert!(
            !fit.code_reserve_holds_the_longest_floor_raise_chain(),
            "2 次装不下最长那一串抬 F 的 3 次（门槛取 2 会判装得下）"
        );
        assert_eq!(fit.clause_reserve, 108, "条款读法 10 + 7 × 14");
        assert!(
            fit.clause_reserve_holds_the_empty_publishes_of_one_admission(),
            "⌊108 ÷ 5⌋ = 21 ≥ 7"
        );
        let at_the_threshold = reserve_fit(15, &[5]).expect("取得出一次空发布");
        assert!(
            at_the_threshold.code_reserve_holds_the_longest_floor_raise_chain(),
            "⌊15 ÷ 5⌋ = 3 正好装得下"
        );
        assert!(
            !at_the_threshold.code_reserve_holds_the_empty_publishes_of_one_admission(),
            "3 < 7"
        );
    }

    #[test]
    fn reserve_fit_has_no_reading_without_an_obtained_empty_publish_that_writes_something() {
        assert_eq!(reserve_fit(14, &[]), None, "一次都取不出：读不到不当 0");
        assert_eq!(reserve_fit(14, &[0]), None, "最大的是 0：除不了");
    }

    #[test]
    fn the_smallest_pool_search_takes_32_mebibytes_when_every_trial_completes() {
        assert_eq!(
            smallest_completed_trial(&SMALLEST_POOL_TRIAL_MEBIBYTES, |_mebibytes| true),
            Some(32)
        );
    }

    #[test]
    fn the_smallest_pool_search_takes_the_first_completed_trial_in_size_order_and_none_when_none_completes(
    ) {
        assert_eq!(
            smallest_completed_trial(&SMALLEST_POOL_TRIAL_MEBIBYTES, |mebibytes| mebibytes >= 128),
            Some(128)
        );
        assert_eq!(
            smallest_completed_trial(&SMALLEST_POOL_TRIAL_MEBIBYTES, |_mebibytes| false),
            None
        );
    }

    #[test]
    fn the_threshold_moved_between_the_observed_sample_points_turns_and_equal_sample_points_do_not()
    {
        let spread = discrimination_of_a_moved_threshold(&[2, 105], 3).expect("两个取样点");
        assert_eq!(spread.moved_threshold, Some(54), "2 + ⌈103 ÷ 2⌉");
        assert_eq!(spread.holding_at_the_moved_threshold, 1);
        assert_eq!(spread.failing_at_the_moved_threshold, 1);
        assert!(spread.turns());
        let equal = discrimination_of_a_moved_threshold(&[105, 105], 3).expect("两个取样点");
        assert_eq!(equal.moved_threshold, None);
        assert!(!equal.turns(), "取样点同值时翻不动");
        assert_eq!(discrimination_of_a_moved_threshold(&[], 3), None);
    }
}
