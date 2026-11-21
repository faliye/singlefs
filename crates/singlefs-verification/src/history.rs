//! 随机历史（里程碑「覆盖写、释放、回退与复用」增补 3 第 1 件）：按种子生成一段操作序列，操作只调 `crates/` 今天的公开入口；每一步之后的镜像
//! 经每步镜像那个钩子交出去（`execute_history_handing_out_every_step_image`），池级 checker 那一遍归给定状态引擎两段（
//! 镜像作为给定状态写进判定库、判段从库读、判在 GPU），harness 一条不变量都不判（`PerStepChecker::Skipped`）。
//! 入口返回 `Err` 算合法结局；panic、模型对不上（与故障注入的合并入口照给定状态引擎判回的结论停下的那一步上的 checker 违例）算失败——撞到「已知红」清单
//! （`KNOWN_RED_FORMS`）里的形态照记、这段历史到此为止，清单外的算新发现，收缩到最短复现（`shrink_operations`）。
//! 第 2 件接上了理想模型（`crate::model`）：每一步调入口之前问模型该成、该拒还是区间里都行，调完拿实现的结局与它比（胶水在
//! `crate::model_comparison`）——冷启动读回的内容、回退与抬 F 该不该被拒、根的身份与 F、单元的分配代都由它判，对不上算失败。
//!
//! 生成与执行分开：一步操作只带「做什么 + 选择子」，写多长、回退到哪条根、F 抬到多少，执行时按那一刻的盘面与会话现解——
//! 删掉前面几步之后，后面的操作照样有意义，收缩靠的就是这一条。
//!
//! 第 3 至 5 件在这里生成的历史上做：录制流由调用方给（`execute_history_with`；崩溃注入给开了内容保留的流，截断点从流里取），
//! 每一步之后的镜像经观察者交出去（坏盘输入拿它当合法镜像）；故障注入（第 4 件）在**录制器外面**包了一层
//! `crate::fault_injection::FaultInjectingBlockDevice`，注入计划由调用方给（`execute_history_with_faults`），
//! 不注入时它只数调用、每一次都原样交给录制器——报错的写、被吞掉的写与被吞掉的屏障都不进录制流，
//! 录制流因此恒等于真正落到盘上的那一串。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Once};

use singlefs_checker::image::{chosen_system_configurations, valid_roots, ImageReader};
use singlefs_checker::walk::allocation_record_count_under_root;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{
    AllocationRecord, AllocationRecordQuarantineMark, PlacementRefusal, PoolAllocator,
    UnitAreaStart,
};
use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
use singlefs_core::compaction_plan::{
    region_is_empty_by_the_records, CompactionBatchSize, CompactionBatchUnits,
    CompactionOfThisMount, CompactionRunForcing, CompactionSwitches,
};
use singlefs_core::journal::back_chain_of;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_space_admission_and_compaction_switches, raise_rollback_floor,
    roll_back_by_a_forward_publish_in_the_session, FloorOfThePushedPublishes, MountError,
    ParametersAndDeviceTableOfTheMount, ResurrectedCopy, RollbackError, RollbackTarget,
    ShadowLedger, StillUnreadableAfterOneReread, SystemConfigurationUnreadableWhen,
};
use singlefs_core::mounted_session::{
    CompactionRun, CompactionRunFailure, CompactionRunOutcome, CompactionRunRefused,
    MountedSession, RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing, UserChange,
    UserChangePublished, UserChangeRefused,
};
use singlefs_core::records::{
    birth_txg_of_a_mapping_key, NLINK_OF_AN_INODE_BEING_DELETED, STATISTIC_FREE_BYTES,
    TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    instance_table_of_root, read_the_first_file_of_a_snapshot, readable_roots, recover,
    JournalPolicy, PoolReader, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_without_units, warm_up,
    CopyQuarantinedAfterReleaseChecksumMismatch, FirstFile, PartialWriteBytes, PoolVersion,
    PoolWriter, PublishError, RelocationBatchRefused, TransactionOutput,
    UnitRelocatedByThisPublish, ZeroUnitPublishPlan,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, EXTENT_TREE_LOWER_LEAF_DATA_UNITS, INODE_LEAF_RECORDS,
    JOURNAL_RING_START_SLOT, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
use crate::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
use crate::model::{
    IdealModel, ModelAnswer, ModelCheckpointTxg, ModelCompactionPublishSeen, ModelDeviceIdentity,
    ModelDisagreement, ModelDisagreementAspect, ModelFloorLeftByALostPower,
    ModelInstanceTableOfTheBatch, ModelJudgementCounts, ModelPoolGeometry, ModelRefusalReason,
    ModelRelocationBatchAfterAUserChangeSeen, ModelRingPosition, ModelRootKey, ObservedEffect,
    ObservedOutcome, ObservedRefusalReason, ObservedSnapshotReadBack, MODEL_ROOT_RING_SLOTS,
};
use crate::model_comparison::{
    compaction_region_of_the_model, model_compaction_region, model_root_key,
    observed_logged_compaction_intent, observed_mount_carrying_to_zero_unit_publishes,
    observed_read_back_after_a_crash, observed_root_of_file_version,
    observed_root_of_version_without_file_carrying, observed_snapshot_read_back,
    refusal_reason_of_mount_error, refusal_reason_of_publish_error,
    refusal_reason_of_rollback_error, reported_ceiling_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
    unit_copy_unreadable_or_mismatched_of_rollback_error,
    user_visible_root_units_relocated_by_this_publish, ContentsCarriedToZeroUnitPublishes,
};
use crate::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use crate::segments::FixedGeometry;
use crate::{RecordingBlockDevice, SharedStream};

/// 核心层那一份成员表生成宏（`singlefs_core::enum_listing_every_member!`）在 harness 这边的名字，给下面的转发宏用。
#[doc(hidden)]
pub use singlefs_core::enum_listing_every_member as enum_listing_every_member_defined_in_the_core_layer;

/// 声明一个封闭集合的枚举，连同它的 `ALL`（每一个成员，按声明的次序）：原样转给核心层那一份（`singlefs_core::enum_listing_every_member!`，
/// 核心层的空间准入枚举也经它生成，定义只有那一份）。生成的 `ALL` 由 `history::tests` 里的
/// `the_generated_member_table_lists_every_member_once_in_declaration_order` 盯着。
#[macro_export]
macro_rules! enum_listing_every_member {
    ($($declaration:tt)*) => {
        $crate::history::enum_listing_every_member_defined_in_the_core_layer! { $($declaration)* }
    };
}

#[cfg(test)]
mod allocation_records_for_tests;
mod applying_mounts_crash_recovery_and_rollbacks;
mod applying_operations_and_inode_changes;
mod applying_publishes_and_user_changes;
mod applying_snapshots_and_compaction;
mod batching_session;
mod execution_settings_and_reproduction;
mod failure_signatures;
mod history_execution;
mod history_generation;
mod history_tally;
mod operations_and_choices;
mod outcome_members;
mod relocation_batches;
mod shrinking_and_campaigns;
mod step_outcomes_and_record_checks;

#[cfg(test)]
use self::allocation_records_for_tests::*;
pub use self::applying_mounts_crash_recovery_and_rollbacks::*;
use self::applying_operations_and_inode_changes::*;
pub use self::applying_publishes_and_user_changes::*;
use self::applying_snapshots_and_compaction::*;
pub use self::batching_session::*;
pub use self::execution_settings_and_reproduction::*;
pub use self::failure_signatures::*;
pub use self::history_execution::*;
pub use self::history_generation::*;
pub use self::history_tally::*;
pub use self::operations_and_choices::*;
use self::outcome_members::*;
pub use self::relocation_batches::*;
pub use self::shrinking_and_campaigns::*;
pub use self::step_outcomes_and_record_checks::*;

/// 随机历史、崩溃注入与坏盘输入三场战役共用的种子基（从 `crash_injection` 挪到这里：随机历史与故障注入用例也用它）。
/// 全部随机路径的种子基：随机历史那五段（增补 3 第 1 件）、崩溃注入这三档（第 3 件）、坏盘输入、故障注入与 E158 装置的种子区间都从它起。
///
/// **永久固定，不重抽**：崩溃放量要把每个状态的过程落文件、按内容 hash 复用判定，
/// 种子一换这些产物整批作废；`crates/mutations.tsv` 的「这条变异必须红」与里程碑验收说的判红也都绑在这一批历史上，
/// 种子基一动它们就成了掷骰子。值是抽出来的（`python3 -c "import secrets; print(secrets.randbits(63))"`），从此不动。
///
/// **要多测别的历史，只往后追加新的种子区间**（新的偏移段）：已有区间与它们的判红、镜像、过程文件照旧可复用；
/// 不重抽这个基，也不改已有区间的偏移。
///
/// TODO：名字里的 `THIS_TEST_CYCLE` 与「永久固定」不符；改名要动另一个会话正在改的三份文件，
/// 等里程碑二提交之后做，登记在 `records/2026-09-24-里程碑二收尾调度.md` 第三节「种子基永久固定」那一行。
pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;

enum_listing_every_member! {
/// 历史里两块内存盘多宽（取样点的参数；两块恒等大：模型的几何只有一个盘大小，`ModelPoolGeometry::device_size_in_bytes`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryDeviceWidth {
    /// 4 GiB，与 `tests/common` 的文件镜像同宽：单元区 211968 槽，几百步的历史走不到单元区墙。随机历史的默认批、前三个取样点与设了变量的那一批用它。
    FourGibibytes,
    /// 单元区 384 槽的小盘：第一个文件之后每盘占 13 槽、一次覆盖写再占 10 槽，回收之前几十次发布就写满，分配器的落点拒绝
    /// （每块盘上都没有合政策的落点）在一段历史里走得到（增补 3 第 2 件代码三方第 2 条，照攻方副本的做法：
    /// 4 GiB 的盘上四段历史里落点拒绝一次都没有，映射把它报成什么都没人看得见）。单元区起点随环长走（journal 环末尾的下一个槽，
    /// C475（非默认环长下单元区起点取编译期常量），实审 A3b），盘宽 = 起点 + 单元区槽数；mkfs 要求环不超过设备容量的四分之一
    /// （`make_filesystem::check_geometry`），环缩到 6 MiB（[`SMALL_DEVICE_JOURNAL_RING_BYTES`]）。
    /// 名字里的 384 槽是环窗口补足之外的那一段：环窗口（D28（挂载期承诺量） 已定项 1 末项）每块盘常驻扣，单元区放大了
    /// [`SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`]，准入算出的可用与放大之前一样多；
    /// 分配器手里的空槽多了这么多，落点那一判的墙跟着往后挪。
    UnitAreaOf384Slots,
    /// 单元区 256 槽的小盘（journal 环同 384 槽那一档）：单元区墙那一格的取样点用它。mkfs 同一个进程那条会话也装上根环表之后
    /// （`make_filesystem::allocator_after_make_filesystem`），每条会话在根环转过之后都按谓词回收，稳态占用约是根环里那 24 版的账
    /// （每版每盘约 10 槽），384 槽那一档一次落点拒绝都走不到了；256 槽走得到用户数据那一处的拒绝。
    /// 名字里的 256 槽同样是环窗口补足（[`SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`]）之外的那一段。
    UnitAreaOf256Slots,
    /// 单元区 240 槽的小盘（journal 环同 384 槽那一档）：可写挂载自己那一串（写行与暖机）在这么窄的盘上拿不到落点，
    /// 走得到取号之后才被落点拒绝的那一格（增补 2 收口表第 39 行那一族：取号之前的准入不算落点）；256 槽那一档的单元区墙取样点上
    /// 一次都没走到过。名字里的 240 槽同样是环窗口补足（[`SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`]）之外的那一段。
    UnitAreaOf240Slots,
}
}

/// 小盘在环窗口补足之外的单元区槽数（`HistoryDeviceWidth::UnitAreaOf384Slots`）：六个 64 槽的聚簇段。
const SMALL_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE: u64 = 384;

/// 更小一档小盘在环窗口补足之外的单元区槽数（`HistoryDeviceWidth::UnitAreaOf256Slots`）：四个 64 槽的聚簇段。
const SMALLER_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE: u64 = 256;

/// 再窄一档小盘在环窗口补足之外的单元区槽数（`HistoryDeviceWidth::UnitAreaOf240Slots`）：三个整 64 槽的聚簇段加 48 槽的尾巴。
const NARROWEST_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE: u64 = 240;

/// 384 槽那一档小盘为环窗口补足的单元区槽数（D28（挂载期承诺量） 已定项 1 末项环窗口每块盘常驻扣「甲」：小盘放大到扣得起）。
/// 放大之后单元区 [1408, 2799) 每块盘罩分配记录树叶 3 片（812 槽一片：⌊2798 ÷ 812⌋ − ⌊1408 ÷ 812⌋ + 1）⇒ N_0 = 6、
/// W = 24 + (6 + 0) + 6 = 36（退役树那一项 6 + H_退役，H_退役 = 0）、环窗口 (3 × 8 + 1) × 36 = 900 槽。同一次补足还罩着 ckpt_cost 多出来的 8 块：
/// 叶多罩一片，分配记录树那一项 1 + 2 × 2 = 5 → 1 + 2 × 3 = 7（多 2 块），谱系那一项 3 块（同日定「乙」），退役那一项 3 块
/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2）；保留池每块盘多 8 槽、切换预留里暖机那一半多 (3 + 1) × 3 × 8 = 96 槽；
/// 标签 6 退役那一份摊到每块盘 (2 + 0 + 1) × 16384 + 35 × n_记录 字节：补整的 3 槽，35 × n_记录 字节那一截不补（同墓碑那 41 × n_live 字节当初也没补）。
/// 900 + 8 + 96 + 3 = 1007：准入算出的可用(d) 与放大之前、ckpt_cost 加谱系与退役两项之前逐盘一样多、只少 35 × n_记录 字节（推的；每次发布改哪几片叶
/// 跟着盘宽变，实测见用例）。之前是 815（没有退役那几项：750 + 5 + 60）。
const SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;

/// 256 槽、240 槽那两档小盘为环窗口补足的单元区槽数：放大之后单元区 [1408, 2671) 与 [1408, 2655) 每块盘罩分配记录树叶 3 片
/// （⌊2670 ÷ 812⌋ = ⌊2654 ÷ 812⌋ = 3）⇒ N_0 = 6、W = 24 + (6 + 0) + 6 = 36、环窗口 25 × 36 = 900 槽；ckpt_cost 多 8 块（同上：叶多罩一片 2、
/// 谱系 3、退役 3）：保留池每块盘多 8 槽、切换预留多 12 × 8 = 96 槽；标签 6 退役那一份补整的 3 槽。900 + 8 + 96 + 3 = 1007，两档与 384 槽那一档
/// 补足一样多。之前是 739（叶只罩 2 片：700 + 3 + 36），退役那几项把单元区推过第 3 片叶的起点 2436。
const SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;

/// 小盘上的 journal 环字节数：6 MiB。单元区从环末尾的下一个槽 1024 + 384 = 1408（= 22 × 64，落在聚簇段边界上）起，
/// 三档小盘的盘宽 = 1408 + 环窗口补足 + 单元区槽数，最窄那档（240 槽）是 1408 + 1007 + 240 = 2655 槽，环 6 MiB 不超过它的四分之一；
/// 环若是 7 MiB，256 槽、240 槽那两档就超了。在飞上限 = 6 MiB ÷ 4096 ÷ 3 = 512 条。
/// 128 MiB 的环把单元区起点拉到 9216，
/// 同样宽的盘上单元区有四万多槽、走不到单元区墙，四分之一那一关又不许环更长。
const SMALL_DEVICE_JOURNAL_RING_BYTES: u64 = 6 << 20;

/// 小盘的单元区起始槽号：[`SMALL_DEVICE_JOURNAL_RING_BYTES`] 那条环末尾的下一个槽（mkfs 写实例表单元、挂载建空闲图都从它起）。
fn small_device_unit_area_start_slot() -> u64 {
    UnitAreaStart::following_the_journal_ring(SMALL_DEVICE_JOURNAL_RING_BYTES)
        .expect("6 MiB 的环末尾的下一个槽 1408 = 22 × 64，落在聚簇段边界上")
        .slot()
        .0
}

impl HistoryDeviceWidth {
    /// 每块盘的字节数。
    #[must_use]
    pub fn device_bytes(self) -> u64 {
        match self {
            HistoryDeviceWidth::FourGibibytes => 4 << 30,
            HistoryDeviceWidth::UnitAreaOf384Slots => {
                (small_device_unit_area_start_slot()
                    + SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS
                    + SMALL_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE)
                    * SLOT_BYTES
            }
            HistoryDeviceWidth::UnitAreaOf256Slots => {
                (small_device_unit_area_start_slot()
                    + SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS
                    + SMALLER_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE)
                    * SLOT_BYTES
            }
            HistoryDeviceWidth::UnitAreaOf240Slots => {
                (small_device_unit_area_start_slot()
                    + SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS
                    + NARROWEST_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE)
                    * SLOT_BYTES
            }
        }
    }

    /// mkfs 与发布的参数：E142 装置那一份（物理块 512、io_min 512，与各步用例相同），小盘只把 journal 环缩小。
    #[must_use]
    pub fn parameters(self) -> MakeFilesystemParameters {
        let mut parameters = e142_parameters(512, 512);
        match self {
            HistoryDeviceWidth::FourGibibytes => {}
            HistoryDeviceWidth::UnitAreaOf384Slots
            | HistoryDeviceWidth::UnitAreaOf256Slots
            | HistoryDeviceWidth::UnitAreaOf240Slots => {
                parameters.geometry.journal_ring_bytes = SMALL_DEVICE_JOURNAL_RING_BYTES;
            }
        }
        parameters
    }

    /// 按落点分写的种类要的那份几何（`crate::segments::FixedGeometry`）：故障注入拿它报「注入落在哪一类结构上」。
    #[must_use]
    pub fn fixed_geometry(self) -> FixedGeometry {
        let parameters = self.parameters();
        FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        }
    }

    /// 报告里的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            HistoryDeviceWidth::FourGibibytes => "两块 4 GiB 的盘",
            HistoryDeviceWidth::UnitAreaOf384Slots => "两块单元区 384 槽的小盘（journal 环 6 MiB）",
            HistoryDeviceWidth::UnitAreaOf256Slots => "两块单元区 256 槽的小盘（journal 环 6 MiB）",
            HistoryDeviceWidth::UnitAreaOf240Slots => "两块单元区 240 槽的小盘（journal 环 6 MiB）",
        }
    }

    /// 复现行与复现入口的环境变量里这一档盘宽的取值（`HistorySamplingVariable::Devices`）。
    #[must_use]
    pub const fn environment_value(self) -> &'static str {
        match self {
            HistoryDeviceWidth::FourGibibytes => "4gib",
            HistoryDeviceWidth::UnitAreaOf384Slots => "unit-area-384",
            HistoryDeviceWidth::UnitAreaOf256Slots => "unit-area-256",
            HistoryDeviceWidth::UnitAreaOf240Slots => "unit-area-240",
        }
    }
}

/// 这个进程里开着的可写会话：分配器、现行版本、实例代号，都是挂载（或起点那个进程）交回的。
struct WritableSession {
    allocator: PoolAllocator,
    current: PoolVersion,
    instance: InstanceGeneration,
    /// 这次挂载走的影子账那一臂（`MountOutput::shadow_ledger`；起点那个进程开着）：覆盖写经挂着的会话发布，
    /// 会话推抬 F 时照它重算影子账（`singlefs_core::mounted_session::MountedSession`）。
    shadow_ledger: ShadowLedger,
    /// 这次挂载认下的池（`MountOutput::parameters_and_device_table`；起点那个进程读盘造）：覆盖写经挂着的会话发布，
    /// 会话只按它的参数建写入口、拿交进来的盘表与它比（实审 A1b Q2、Q3）。
    parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
    /// 这次挂载（或起点那个进程）里写出了几条根：写行、暖机、每次发布、抬 F 的每次空发布都算。
    publishes_in_this_mount: usize,
    /// 这次挂载里最近一版交回（或往下带下来）的实例表字节与分配记录：现行那一版是树表 0 条的一版、同一次挂载里写行那次交回过它们时有，
    /// 零单元发布从它往下带着比（实审 B3c-4）；一次挂载里第一版就是零单元发布（mkfs 之后第一次可写挂载）、现行那一版带文件
    /// （零单元发布只接在树表 0 条的一版后面）时是 None。
    carried_to_zero_unit_publishes: Option<ContentsCarriedToZeroUnitPublishes>,
    /// 整理在这次挂载里会话这一侧的状态（`compaction_plan::CompactionOfThisMount`：不合格段表、常驻隔离集、`B_gc` 数的发布数、观测）：
    /// 生命周期一次挂载，执行器每一步借给会话、推完原样收回，跨步带着（`B_gc` 数的是上一次用户改动之后整理发了几次布，不是这一步里）。
    /// 可写挂载交回的那一份（`MountOutput::compaction_continuation`）接着用；起点那个进程从新的一份起。
    compaction: CompactionOfThisMount,
}

/// 一段历史跑到哪了：两块盘（与它们的宽度）、这个进程的会话、挂载的次数、理想模型、录制流（判「拒绝之前写没写盘」）。
struct HistoryPool {
    device_width: HistoryDeviceWidth,
    /// 判不判空间准入（只供测试的开关，`singlefs_core::admission::SpaceAdmission`）：起点那条会话装在分配器上，挂载照它传。
    space_admission: SpaceAdmission,
    devices: Vec<(DeviceIdentity, HistoryDevice)>,
    session: Option<WritableSession>,
    successful_mounts: usize,
    mount_attempts: usize,
    model: IdealModel,
    stream: SharedStream,
    /// mkfs 写完时录制流里有几步（崩溃注入的基线从这里起）。
    operations_written_by_make_filesystem: usize,
    /// 每次掉电之后录制流里第一步的下标，按次序（交回 [`HistoryRun::earliest_stream_index_after_each_power_loss`]）。
    earliest_stream_index_after_each_power_loss: Vec<usize>,
    /// 每次截断（按步号）交给会话的目标 size（执行时按类取的，交回 [`HistoryRun::sizes_requested_by_the_truncations`]）。
    sizes_requested_by_the_truncations: BTreeMap<usize, u64>,
    /// 每次截断（按步号）点名的 inode 号（交回 [`HistoryRun::inodes_named_by_the_truncations`]）。
    inodes_named_by_the_truncations: BTreeMap<usize, u64>,
    /// 每次按偏移写（按步号）交给会话的号、偏移、长度与真落到的格（交回 [`HistoryRun::partial_writes_requested`]）。
    partial_writes_requested: BTreeMap<usize, PartialWriteRequested>,
    /// 这一步里实现交回的隔离（[`copies_quarantined_by_the_versions`] 那几处收进来），这一步跑完由执行器记上步号并进下一格。
    copies_quarantined_in_this_step: Vec<CopyQuarantinedAfterReleaseChecksumMismatch>,
    /// 到此为止实现交回的每一份隔离，跨重挂累积（池在整段历史里只有这一个，会话随挂载换）：交回
    /// [`HistoryRun::copies_quarantined_by_the_implementation`]。
    copies_quarantined_by_the_implementation: Vec<CopyQuarantinedByTheImplementation>,
}

/// 模型对一步的判定：对不上的那一格（没有就是 None）与这一步比了多少格。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelVerdict {
    pub disagreement: Option<ModelDisagreement>,
    pub counts: ModelJudgementCounts,
}

/// 拿实现的结局与模型的答案比，对得上就让模型往前走（模型答不了的，答案本身就是那一格对不上）。
// gpu-exempt: comparison-with-the-ideal-model
fn judge_by_model(
    model: &mut IdealModel,
    answer: Result<ModelAnswer, ModelDisagreement>,
    observed: &ObservedOutcome,
) -> ModelVerdict {
    match answer.and_then(|answer| model.judge_and_advance(&answer, observed)) {
        Ok(counts) => ModelVerdict {
            disagreement: None,
            counts,
        },
        Err(disagreement) => ModelVerdict {
            disagreement: Some(disagreement),
            counts: ModelJudgementCounts::default(),
        },
    }
}

/// 一步（起点算一步）里模型写进根环几条根：记进 [`HistoryTally::most_roots_written_by_the_model_in_one_step`]，交回这一步之后的累计数
/// （下一步从它量起）。多于根环槽数（[`MODEL_ROOT_RING_SLOTS`]）时这一步先写的根被同一步后写的盖掉，观察者每步并一次模型根环
/// （`IdealModel::committed_versions`）就漏了那几版：一步写几条根没有写死的常数上界（那一段注释按今天的代码数了来源），量到的必须在槽数之内。
///
/// # Panics
/// 这一步模型写出的根多于根环槽数。
fn measure_the_roots_the_model_wrote_in_one_step(
    tally: &mut HistoryTally,
    model: &IdealModel,
    roots_written_by_the_model_before_this_step: u64,
    position: StepPosition,
) -> u64 {
    let roots_written_by_the_model_after_this_step = model.roots_written_since_make_filesystem();
    let roots_written_in_this_step = roots_written_by_the_model_after_this_step
        .checked_sub(roots_written_by_the_model_before_this_step)
        .expect("模型写进根环的累计数只增不减");
    assert!(
        roots_written_in_this_step <= MODEL_ROOT_RING_SLOTS,
        "{position:?} 这一步模型写进根环 {roots_written_in_this_step} 条根，多于根环槽数 {MODEL_ROOT_RING_SLOTS}：先写的被同一步后写的盖掉，每步并一次模型根环会漏版本"
    );
    tally.most_roots_written_by_the_model_in_one_step = tally
        .most_roots_written_by_the_model_in_one_step
        .max(roots_written_in_this_step);
    roots_written_by_the_model_after_this_step
}

/// 入口返回 Err 时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障。
fn observed_refusal(
    member: String,
    reason: ObservedRefusalReason,
    publishes_completed: usize,
    stream_length_before: usize,
    stream: &SharedStream,
    reported_ceiling: Option<ModelCheckpointTxg>,
    reported_root_ring_slot_still_bad_after_one_reread: Option<ModelRingPosition>,
) -> ObservedOutcome {
    ObservedOutcome::Refused {
        member,
        reason,
        publishes_completed,
        wrote_anything: stream.operation_count() != stream_length_before,
        reported_ceiling,
        reported_root_ring_slot_still_bad_after_one_reread,
    }
}

/// 两块盘此刻的整份镜像（拷一份，checker 与观察者读它）。
fn image_of(
    devices: &[(DeviceIdentity, HistoryDevice)],
    device_width: HistoryDeviceWidth,
) -> MemoryPool {
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
        device_size_in_bytes: device_width.device_bytes(),
    }
}

/// 按 checker 的读法在镜像上数一条根下的分配记录（`singlefs_checker::walk::allocation_record_count_under_root`，与实现的分配器
/// 不共用代码）：根环里 (txg, 实例) 等于 `root` 的那条自证过的根。系统配置或那条根找不到、那棵树读不出都是 None。
#[must_use]
pub fn allocation_records_on_the_image_under(
    image: &MemoryPool,
    root: ModelRootKey,
) -> Option<u64> {
    let geometry = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
    let (_, _, view) = valid_roots(image, &geometry)
        .into_iter()
        .find(|(_, _, view)| {
            view.checkpoint_txg == root.checkpoint_txg.0
                && u64::from(view.instance) == root.instance.0
        })?;
    let records = allocation_record_count_under_root(image, &view)?;
    Some(u64::try_from(records).expect("一个节点至多几百条"))
}

/// 起点段里 mkfs 之后那三步：取号、暖机、第一个文件。任一步返回错误就交回是哪一步、报的什么。
/// 写入口借着 `devices`，交回的错里因此不带镜像——镜像由调用方在写入口放手之后自己取。
fn acquire_warm_up_and_publish_the_first_file(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, HistoryDevice)],
    allocator: &mut PoolAllocator,
    genesis_root: &RootRecord,
    content: &[u8],
) -> Result<(InstanceGeneration, TransactionOutput), (StartingPointStep, String)> {
    let mut writer = PoolWriter::new(parameters, devices);
    let instance = acquire_instance(&mut writer)
        .map_err(|error| (StartingPointStep::AcquireInstance, format!("{error:?}")))?;
    let warmed = warm_up(&mut writer, genesis_root, instance)
        .map_err(|error| (StartingPointStep::WarmUp, format!("{error:?}")))?;
    let output = publish_first_file(
        &mut writer,
        allocator,
        warmed.roots.last().expect("暖机两代根"),
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .map_err(|error| (StartingPointStep::PublishFirstFile, format!("{error:?}")))?;
    Ok((instance, output))
}

impl HistoryPool {
    /// 起点：mkfs（加上同一个进程里的取号、暖机、第一个文件）。模型跟着走一遍，第一个文件那次发布拿实现的输出与模型比，判定一并交回。
    ///
    /// 这四个入口都可能返回块设备错：不注入故障时走不到（两块内存盘的读写不报错），故障注入把注入点摆进起点段之后走得到。
    /// 走到时交回 [`StartingPointFailure`]，不 panic——改法二（判定五坐实起点段今天只会撞出执行器自己的 `expect`）。
    fn start(
        starting_point: HistoryStartingPoint,
        device_width: HistoryDeviceWidth,
        space_admission: SpaceAdmission,
        stream: &SharedStream,
        fault_plan: &SharedFaultPlan,
    ) -> Result<(Self, Option<ModelVerdict>), StartingPointFailure> {
        let parameters = device_width.parameters();
        let device_bytes = device_width.device_bytes();
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
                                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                                stream.clone(),
                            ),
                            fault_plan.clone(),
                        ),
                    )
                })
                .collect();
        let genesis = match make_filesystem(&parameters, &mut devices) {
            Ok(genesis) => genesis,
            Err(error) => {
                return Err(StartingPointFailure {
                    step: StartingPointStep::MakeFilesystem,
                    error: format!("{error:?}"),
                    image: image_of(&devices, device_width),
                    operations_written_by_make_filesystem: stream.operation_count(),
                })
            }
        };
        let operations_written_by_make_filesystem = stream.operation_count();
        let mut model = IdealModel::after_make_filesystem_on_a_journal_ring_of(
            ModelPoolGeometry {
                devices: devices
                    .iter()
                    .map(|(identity, _)| ModelDeviceIdentity(identity.0))
                    .collect(),
                device_size_in_bytes: device_bytes,
            },
            parameters.geometry.journal_ring_bytes,
        );
        let mut verdict = None;
        let session = match starting_point {
            HistoryStartingPoint::AfterMakeFilesystem => None,
            HistoryStartingPoint::AfterFirstFile => {
                // mkfs 同一个进程那条会话也装根环表（`allocator_after_make_filesystem`）：根环转过一圈之后照样按谓词回收，
                // 与做过挂载的会话同一条路（增补 2 收口表第 ② 行）。
                let mut allocator =
                    allocator_after_make_filesystem(&parameters, &devices, &genesis);
                allocator.set_space_admission(space_admission);
                // 这个进程的会话没经过可写挂载：它认下的池从 mkfs 刚写成的盘上读（`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`，
                // 与可写挂载同一套核），不拿执行器手里的参数（实审 A1b Q2）。读在取号之前：注入的读错撞上它时模型与盘面都还停在 mkfs 之后，
                // 交回起点段的错、不 panic。
                let parameters_and_device_table =
                    match ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices) {
                        Ok(parameters_and_device_table) => parameters_and_device_table,
                        Err(error) => {
                            return Err(StartingPointFailure {
                                step: StartingPointStep::ReadThePoolForTheSession,
                                error: format!("{error:?}"),
                                image: image_of(&devices, device_width),
                                operations_written_by_make_filesystem,
                            })
                        }
                    };
                let content = first_file_content();
                let started = acquire_warm_up_and_publish_the_first_file(
                    &parameters,
                    devices.as_mut_slice(),
                    &mut allocator,
                    &genesis.root,
                    &content,
                );
                let (instance, output) = match started {
                    Ok(started) => started,
                    Err((step, error)) => {
                        return Err(StartingPointFailure {
                            step,
                            error,
                            image: image_of(&devices, device_width),
                            operations_written_by_make_filesystem,
                        })
                    }
                };
                model.acquire_and_warm_up_in_the_make_filesystem_process();
                let answer = model.answer_publish_first_file(&content);
                verdict = Some(judge_by_model(
                    &mut model,
                    answer,
                    &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: vec![observed_root_of_file_version(&output)],
                        reported_ceiling: None,
                    }),
                ));
                Some(WritableSession {
                    allocator,
                    current: PoolVersion::WithFile(Arc::new(output)),
                    instance,
                    // 执行器里的抬 F 一律开着影子账（`apply_raise_rollback_floor`），mkfs 那个进程的会话同一臂。
                    shadow_ledger: ShadowLedger::On,
                    parameters_and_device_table,
                    publishes_in_this_mount: 3,
                    carried_to_zero_unit_publishes: None,
                    compaction: CompactionOfThisMount::of_a_new_mount(),
                })
            }
        };
        Ok((
            Self {
                device_width,
                space_admission,
                devices,
                session,
                successful_mounts: 0,
                mount_attempts: 0,
                model,
                stream: stream.clone(),
                operations_written_by_make_filesystem,
                earliest_stream_index_after_each_power_loss: Vec::new(),
                sizes_requested_by_the_truncations: BTreeMap::new(),
                inodes_named_by_the_truncations: BTreeMap::new(),
                partial_writes_requested: BTreeMap::new(),
                copies_quarantined_in_this_step: Vec::new(),
                copies_quarantined_by_the_implementation: Vec::new(),
            },
            verdict,
        ))
    }

    /// 这一步里实现交回的隔离记上步号、并进到此为止那一格（同一份在一步里从几处收到的只记一次）。
    fn note_the_copies_quarantined_in_the_step(&mut self, step: StepPosition) {
        for copy in std::mem::take(&mut self.copies_quarantined_in_this_step) {
            let quarantined = CopyQuarantinedByTheImplementation { step, copy };
            if !self
                .copies_quarantined_by_the_implementation
                .contains(&quarantined)
            {
                self.copies_quarantined_by_the_implementation
                    .push(quarantined);
            }
        }
    }

    /// 两块盘此刻的整份镜像（拷一份，checker 与观察者读它）。
    fn image(&self) -> MemoryPool {
        image_of(&self.devices, self.device_width)
    }
}

/// 按 checker 的读法（`singlefs_checker::image`，与实现的择根不共用代码）读根环：(txg, 实例) 从新到旧、去重，连同一圈的槽数 R × S。
fn ring_roots_newest_first(image: &MemoryPool) -> (Vec<(u64, u32)>, Option<u64>) {
    ring_roots_newest_first_of(image)
}

/// 同上，读的盘面由调用方给（崩溃镜像也走这一条）。
fn ring_roots_newest_first_of(image: &dyn ImageReader) -> (Vec<(u64, u32)>, Option<u64>) {
    let Some(geometry) = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
    else {
        return (Vec::new(), None);
    };
    let mut roots: Vec<(u64, u32)> = valid_roots(image, &geometry)
        .into_iter()
        .map(|(_, _, view)| (view.checkpoint_txg, view.instance))
        .collect();
    roots.sort_unstable_by(|left, right| right.cmp(left));
    roots.dedup();
    (roots, Some(geometry.regions * geometry.slots_per_region))
}

/// 按 checker 的读法：根环里最新那条根（(txg, 实例) 最大）带的 F；读不到时 None。
fn newest_ring_root_floor(image: &MemoryPool) -> Option<u64> {
    let geometry = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
    valid_roots(image, &geometry)
        .into_iter()
        .map(|(_, _, view)| view)
        .max_by_key(|view| (view.checkpoint_txg, view.instance))
        .map(|view| view.rollback_floor)
}

/// 一步操作交回执行器的东西：结局，执行器从这一步交回的东西里自己判出的失败（没有就是 None），模型的判定（入口没调就是 None）。
struct AppliedStep {
    outcome: StepOutcome,
    harness_judgement: Option<HarnessJudgement>,
    model_verdict: Option<ModelVerdict>,
}

impl AppliedStep {
    /// 前提不满足、入口没调：只看结局本身，模型不问。
    fn not_applicable(precondition: MissingPrecondition) -> Self {
        Self {
            outcome: StepOutcome::NotApplicable(precondition),
            harness_judgement: None,
            model_verdict: None,
        }
    }

    /// 这一步交回的东西里没有执行器要另判的（入口返回 Err、零单元发布、冷启动）：执行器只看结局本身（冷启动读回报错），连同模型的判定。
    fn judged_by_outcome_and_model(outcome: StepOutcome, model_verdict: ModelVerdict) -> Self {
        let harness_judgement = harness_judgement_of_outcome(&outcome);
        Self {
            outcome,
            harness_judgement,
            model_verdict: Some(model_verdict),
        }
    }
}

/// 一次发布前后的分配记录：改写或新增的记录的代要在这次（几次）发布的 txg 里；只改隔离记号的，置记号的只放过
/// `copies_quarantined_by_the_publishes`（这次几次发布里实现交回的隔离）点名的那几份，清记号的不放过（只有回退那一处放过，不走这里）。
fn allocation_generation_judgement(
    before: &[AllocationRecord],
    after: &[AllocationRecord],
    first_publish_txg: CheckpointTxg,
    last_publish_txg: CheckpointTxg,
    copies_quarantined_by_the_publishes: &[CopyQuarantinedAfterReleaseChecksumMismatch],
) -> Option<HarnessJudgement> {
    let records = records_changed_with_a_generation_outside_the_publish_txgs(
        before,
        after,
        first_publish_txg,
        last_publish_txg,
        QuarantineMarkChangesVouchedFor {
            copies_quarantined_by_the_publishes,
            clearing: QuarantineMarkClearing::NotDoneByThisPublish,
        },
    );
    (!records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
        records,
        first_publish_txg,
        last_publish_txg,
    })
}

/// 录制流从 `stream_length_before` 起多出的写（含 FUA 写与写零；屏障不算）。
fn writes_in_the_stream_since(stream: &SharedStream, stream_length_before: usize) -> u64 {
    let count = stream.operations()[stream_length_before..]
        .iter()
        .filter(|operation| operation.kind != crate::RecordedOperationKind::Barrier)
        .count();
    u64::try_from(count).expect("写数装得进 u64")
}

/// 那几串抬 F 交给设备的写：先写系统配置那一步加上每一次空发布（`RaisedFloor::system_configuration_writes_before_the_first_publish` 的注释）。
fn writes_of_the_floor_raises(floor_raises: &[singlefs_core::mount::RaisedFloor]) -> u64 {
    floor_raises
        .iter()
        .map(|raised| {
            raised
                .system_configuration_writes_before_the_first_publish
                .total()
                .write_calls
                + raised
                    .publishes
                    .iter()
                    .map(|publish| publish.writes.total().write_calls)
                    .sum::<u64>()
        })
        .sum()
}

#[cfg(test)]
mod tests {

    enum_listing_every_member! {
    /// 只给下面那条用例用的三个成员：看生成的 `ALL` 是不是每个成员恰好一次、按声明的次序。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ThreeDeclaredMembers {
        First,
        Second,
        Third,
    }
    }

    /// 生成的 `ALL` 照成员表逐个列出、不多不少、次序与声明相同：复现入口认的取值全集（盘宽、checker、比重的名字）都由它生成，
    /// 它漏一个、重一个，那一档就复现不出或读回成别的。
    #[test]
    fn the_generated_member_table_lists_every_member_once_in_declaration_order() {
        assert_eq!(
            ThreeDeclaredMembers::ALL,
            [
                ThreeDeclaredMembers::First,
                ThreeDeclaredMembers::Second,
                ThreeDeclaredMembers::Third
            ]
        );
    }
}
