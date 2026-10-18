//! 随机历史（里程碑「覆盖写、释放、回退与复用」增补 3 第 1 件）：按种子生成一段操作序列，操作只调 `crates/` 今天的公开入口；每一步之后的镜像
//! 经每步镜像那个钩子交出去（`execute_history_handing_out_every_step_image`），池级 checker 那一遍归 checker 档两段（用户 2026-10-13 定：
//! 镜像作为给定状态写进判定库、判段从库读、判在 GPU），harness 一条不变量都不判（`PerStepChecker::Skipped`）。
//! 入口返回 `Err` 算合法结局；panic、模型对不上（与故障注入的合并入口照 checker 档判回的结论停下的那一步上的 checker 违例）算失败——撞到「已知红」清单
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
use std::sync::{Mutex, Once};

use singlefs_checker::image::{chosen_system_configurations, valid_roots, ImageReader};
use singlefs_checker::walk::allocation_record_count_under_root;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{AllocationRecord, PlacementRefusal, PoolAllocator, UnitAreaStart};
use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor, roll_back_by_a_forward_publish,
    MountError, ParametersAndDeviceTableOfTheMount, RollbackError, RollbackTarget, ShadowLedger,
    StillUnreadableAfterOneReread, SystemConfigurationUnreadableWhen,
};
use singlefs_core::mounted_session::{
    MountedSession, UserChange, UserChangePublished, UserChangeRefused,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    instance_table_of_root, readable_roots, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_without_units, warm_up, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput, ZeroUnitPublishPlan,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{
    DATA_UNIT_BYTES, EXTENT_TREE_LOWER_LEAF_DATA_UNITS, JOURNAL_RING_START_SLOT, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
use crate::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
use crate::model::{
    IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
    ModelJudgementCounts, ModelPoolGeometry, ModelRefusalReason, ModelRingPosition, ModelRootKey,
    ObservedEffect, ObservedOutcome, ObservedRefusalReason, MODEL_ROOT_RING_SLOTS,
};
use crate::model_comparison::{
    model_root_key, observed_mount_carrying_to_zero_unit_publishes,
    observed_read_back_after_a_crash, observed_root_of_file_version,
    observed_root_of_version_without_file_carrying, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error, refusal_reason_of_rollback_error,
    reported_ceiling_of_mount_error, root_ring_slot_still_bad_after_one_reread_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
    unit_copy_unreadable_or_mismatched_of_rollback_error, ContentsCarriedToZeroUnitPublishes,
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

/// 随机历史、崩溃注入与坏盘输入三场战役共用的种子基（从 checker 档的 `crash_injection` 挪到这里：harness 档的随机历史与故障注入用例也用它，
/// 放在 harness 才不让 harness 的测试依赖 checker 档，D13（验证路线） 已定项 15）。
/// 全部随机路径的种子基：随机历史那五段（增补 3 第 1 件）、崩溃注入这三档（第 3 件）、坏盘输入、故障注入与 E158 装置的种子区间都从它起。
///
/// **永久固定，不重抽**（用户 2026-09-27 定）：崩溃放量要把每个状态的过程落文件、按内容 hash 复用判定，
/// 种子一换这些产物整批作废；`crates/mutations.tsv` 的「这条变异必须红」与里程碑验收说的判红也都绑在这一批历史上，
/// 种子基一动它们就成了掷骰子。值是 2026-09-20 抽的（`python3 -c "import secrets; print(secrets.randbits(63))"`），从此不动。
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
    /// 4 GiB，与 `tests/common` 的文件镜像同宽：单元区 211968 槽，几百步的历史走不到单元区墙。快档、前三个取样点与大档用它。
    FourGibibytes,
    /// 单元区 384 槽的小盘：第一个文件之后每盘占 13 槽、一次覆盖写再占 10 槽，回收之前几十次发布就写满，分配器的落点拒绝
    /// （每块盘上都没有合政策的落点）在一段历史里走得到（增补 3 第 2 件代码三方第二轮判决第三节第 2 条，照攻方副本的做法：
    /// 4 GiB 的盘上四段历史里落点拒绝一次都没有，映射把它报成什么都没人看得见）。单元区起点随环长走（journal 环末尾的下一个槽，
    /// C475（非默认环长下单元区起点取编译期常量），实审 A3b），盘宽 = 起点 + 单元区槽数；mkfs 要求环不超过设备容量的四分之一
    /// （`make_filesystem::check_geometry`），环缩到 6 MiB（[`SMALL_DEVICE_JOURNAL_RING_BYTES`]）。
    /// 名字里的 384 槽是环窗口补足之外的那一段：环窗口（D28（挂载期承诺量） 已定项 1 末项）每块盘常驻扣，单元区放大了
    /// [`SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`]，准入算出的可用与放大之前一样多（用户 2026-10-02 定「甲」）；
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

/// 384 槽那一档小盘为环窗口补足的单元区槽数（D28（挂载期承诺量） 已定项 1 末项环窗口每块盘常驻扣；用户 2026-10-02 定「甲」：小盘放大到扣得起）。
/// 放大之后单元区 [1408, 2799) 每块盘罩分配记录树叶 3 片（812 槽一片：⌊2798 ÷ 812⌋ − ⌊1408 ÷ 812⌋ + 1）⇒ N_0 = 6、
/// W = 24 + (6 + 0) + 6 = 36（退役树那一项 6 + H_退役，H_退役 = 0）、环窗口 (3 × 8 + 1) × 36 = 900 槽。同一次补足还罩着 ckpt_cost 多出来的 8 块：
/// 叶多罩一片，分配记录树那一项 1 + 2 × 2 = 5 → 1 + 2 × 3 = 7（多 2 块），谱系那一项 3 块（同日定「乙」），退役那一项 3 块
/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2）；保留池每块盘多 8 槽、切换预留里暖机那一半多 (3 + 1) × 3 × 8 = 96 槽；
/// 标签 6 退役那一份摊到每块盘 (2 + 0 + 1) × 16384 + 35 × n_记录 字节：补整的 3 槽，35 × n_记录 字节那一截不补（同墓碑那 41 × n_live 字节当初也没补）。
/// 900 + 8 + 96 + 3 = 1007：准入算出的可用(d) 与放大之前、ckpt_cost 加谱系与退役两项之前逐盘一样多、只少 35 × n_记录 字节（推的；每次发布改哪几片叶
/// 跟着盘宽变，实测见用例）。第 49 批之前是 815（没有退役那几项：750 + 5 + 60）。
const SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;

/// 256 槽、240 槽那两档小盘为环窗口补足的单元区槽数：放大之后单元区 [1408, 2671) 与 [1408, 2655) 每块盘罩分配记录树叶 3 片
/// （⌊2670 ÷ 812⌋ = ⌊2654 ÷ 812⌋ = 3）⇒ N_0 = 6、W = 24 + (6 + 0) + 6 = 36、环窗口 25 × 36 = 900 槽；ckpt_cost 多 8 块（同上：叶多罩一片 2、
/// 谱系 3、退役 3）：保留池每块盘多 8 槽、切换预留多 12 × 8 = 96 槽；标签 6 退役那一份补整的 3 槽。900 + 8 + 96 + 3 = 1007，两档与 384 槽那一档
/// 补足一样多。第 49 批之前是 739（叶只罩 2 片：700 + 3 + 36），退役那几项把单元区推过第 3 片叶的起点 2436。
const SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;

/// 小盘上的 journal 环字节数：6 MiB。单元区从环末尾的下一个槽 1024 + 384 = 1408（= 22 × 64，落在聚簇段边界上）起，
/// 三档小盘的盘宽 = 1408 + 环窗口补足 + 单元区槽数，最窄那档（240 槽）是 1408 + 1007 + 240 = 2655 槽，环 6 MiB 不超过它的四分之一；
/// 放大之前（1648 槽），7 MiB 的环在 256 槽、240 槽那两档就超了。在飞上限 = 6 MiB ÷ 4096 ÷ 3 = 512 条。
/// 改之前是 128 MiB、单元区从编译期常量 50176 起（盘约 790 MiB）；单元区起点改成随环长走之后，128 MiB 的环把起点拉到 9216，
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

/// 历史里的一块盘：内存盘外面包录制器（写与屏障进调用方给的那条流），录制器外面再包故障注入
/// （`crate::fault_injection::FaultInjectingBlockDevice`；不注入时只数调用）。
/// 注入层在最外面：报错的写、被吞掉的写与被吞掉的屏障都不进录制流。
pub type HistoryDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

/// 一段历史的种子。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HistorySeed(pub u64);

/// 手写的伪随机源：SplitMix64（Steele、Lea、Flood，OOPSLA 2014；`java.util.SplittableRandom` 的输出函数），不加依赖（2026-09-18 用户定）。
/// 同一个种子逐位复现同一串数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeededRandomSource {
    state: u64,
}

impl SeededRandomSource {
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_word(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }

    /// [0, `exclusive_upper_bound`) 里的一个数。取模的偏差在 2^64 面前可以不计：这里要的是复现，不是均匀到最后一位。
    ///
    /// # Panics
    /// `exclusive_upper_bound` 为 0（区间是空的）。
    pub fn below(&mut self, exclusive_upper_bound: u64) -> u64 {
        assert!(
            exclusive_upper_bound > 0,
            "区间 [0, 0) 是空的：调用方要先确认至少有一个可选项"
        );
        self.next_word() % exclusive_upper_bound
    }
}

/// 一段历史从哪里起。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistoryStartingPoint {
    /// 只做过 mkfs、进程已退出：没有可写会话，要先挂载。
    AfterMakeFilesystem,
    /// mkfs 同一个进程里取号、暖机、发布第一个文件（E142 的 3000 字节），会话还开着（与 `tests/common` 的 `build_pool` 同一条路）。
    AfterFirstFile,
}

/// 发布的内容多长：执行时按数据单元的载荷容量（`data_unit_payload_capacity`）换成字节数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentLength {
    Empty,
    /// [1, 载荷容量 − 2] 里按选择子取。
    InsideOneDataUnit {
        selector: u64,
    },
    OneByteBelowDataUnitPayloadCapacity,
    ExactlyDataUnitPayloadCapacity,
    /// 装不下：载荷容量 + 1。
    OneByteAboveDataUnitPayloadCapacity,
    /// 装不下：一个数据单元的整宽（含头与预留）。
    ExactlyDataUnitBytes,
    /// 跨 `data_units`（≥ 2）个数据单元的内容：前面的单元装满净荷容量，最后一个单元装 [1, 载荷容量] 里按选择子取的字节。
    /// 这一种走顺序写入口（[`ContentLength::whole_file_write_entry`]，`UserChange::SequentialWrite`）：覆盖写的契约是一个数据单元，
    /// 多于一片下段叶（144 个单元）的文件只有顺序写长得出来（代码轮 U13）。只有偏向删除多片下段叶那一组比重抽它
    /// （[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]），别的几组比重的抽法与生成的历史不变。
    SpanningDataUnitsThroughTheSequentialWrite {
        data_units: u64,
        selector: u64,
    },
}

/// 整份重写文件内容走哪个入口（`mounted_session::UserChange` 的哪一个成员）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WholeFileWriteEntry {
    /// `UserChange::Overwrite`：契约是一个数据单元，装不下拒。
    Overwrite,
    /// `UserChange::SequentialWrite`：按切分纪律切成几个一单元事务，不按单元数拒。
    SequentialWrite,
}

impl ContentLength {
    #[must_use]
    pub fn in_bytes(self) -> usize {
        let capacity = data_unit_payload_capacity();
        match self {
            ContentLength::Empty => 0,
            ContentLength::InsideOneDataUnit { selector } => {
                let choices = u64::try_from(capacity - 2).expect("载荷容量三万多字节");
                1 + usize::try_from(selector % choices).expect("小于载荷容量")
            }
            ContentLength::OneByteBelowDataUnitPayloadCapacity => capacity - 1,
            ContentLength::ExactlyDataUnitPayloadCapacity => capacity,
            ContentLength::OneByteAboveDataUnitPayloadCapacity => capacity + 1,
            ContentLength::ExactlyDataUnitBytes => usize::try_from(DATA_UNIT_BYTES).expect("32768"),
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units,
                selector,
            } => {
                let capacity_in_bytes = u64::try_from(capacity).expect("载荷容量三万多字节");
                let bytes_of_the_full_units = (data_units - 1) * capacity_in_bytes;
                let bytes_of_the_last_unit = 1 + selector % capacity_in_bytes;
                usize::try_from(bytes_of_the_full_units + bytes_of_the_last_unit)
                    .expect("几百个单元的内容装得进 usize")
            }
        }
    }

    /// 这一种内容走哪个入口：跨几个数据单元的走顺序写，别的（含装不下一个单元的那两种：它们要钉的正是覆盖写的拒）走覆盖写。
    #[must_use]
    pub fn whole_file_write_entry(self) -> WholeFileWriteEntry {
        match self {
            ContentLength::Empty
            | ContentLength::InsideOneDataUnit { .. }
            | ContentLength::OneByteBelowDataUnitPayloadCapacity
            | ContentLength::ExactlyDataUnitPayloadCapacity
            | ContentLength::OneByteAboveDataUnitPayloadCapacity
            | ContentLength::ExactlyDataUnitBytes => WholeFileWriteEntry::Overwrite,
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite { .. } => {
                WholeFileWriteEntry::SequentialWrite
            }
        }
    }

    /// 统计表里的名字（选择子不进名字）。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ContentLength::Empty => "0 字节",
            ContentLength::InsideOneDataUnit { .. } => "[1, 载荷容量 − 2]",
            ContentLength::OneByteBelowDataUnitPayloadCapacity => "载荷容量 − 1",
            ContentLength::ExactlyDataUnitPayloadCapacity => "载荷容量",
            ContentLength::OneByteAboveDataUnitPayloadCapacity => "载荷容量 + 1（装不下）",
            ContentLength::ExactlyDataUnitBytes => "数据单元整宽 32768（装不下）",
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units, .. } => {
                if data_units <= EXTENT_TREE_LOWER_LEAF_DATA_UNITS {
                    "跨一片下段叶以内的多个单元（顺序写）"
                } else if data_units <= 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS {
                    "跨两片下段叶（顺序写）"
                } else {
                    "跨三片及以上下段叶（顺序写）"
                }
            }
        }
    }
}

/// 一次发布的内容：长度与填充的种子（字节由种子现算，同一个选择逐字节复现）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentChoice {
    pub length: ContentLength,
    pub fill_seed: u64,
}

impl ContentChoice {
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let length = self.length.in_bytes();
        let mut source = SeededRandomSource::from_seed(self.fill_seed);
        let mut bytes = Vec::with_capacity(length + 8);
        while bytes.len() < length {
            bytes.extend_from_slice(&source.next_word().to_le_bytes());
        }
        bytes.truncate(length);
        bytes
    }
}

/// 回退的目标怎么取：执行时按 checker 的读法从盘上读根环（与实现的择根不共用代码）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RollbackTargetChoice {
    /// 根环里的一条可读根：按 (txg, 实例) 从新到旧排、去重，取第 `index_from_newest mod 条数` 条。
    /// 候选集里的与候选集外的（被抛弃时间线的、F 之下的、树表 0 条的）都会落到。
    RingRoot { index_from_newest: u64 },
    /// 不在根环里的目标：最新那条根的实例、txg = 最新 txg + 1 + `txg_beyond_newest mod 3`。
    BeyondNewestRoot { txg_beyond_newest: u64 },
    /// 候选集的下沿：根环里 txg 等于最新那条根带的 F 的那条根（同一个 txg 上有几条取实例最大的）；F 那一代已被盖掉时取最新那条。
    /// 只有 `RollbackTargetDraw::FloorRootHalfTheTime` 抽它（理想模型那一格 B2：回退到 txg = F_生效 的根被拒）。
    RingRootAtTheNewestFloor,
}

/// 回退的目标按什么抽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollbackTargetDraw {
    /// 第一版的抽法：五分之二取最近四条、五分之二在整个根环里均匀取、五分之一取根环之外。快档、大档与第 121 行的取样点用它。
    RecentUniformOrBeyond,
    /// 先抽一次：一半取候选集的下沿（`RingRootAtTheNewestFloor`），另一半照第一版的抽法。
    FloorRootHalfTheTime,
    /// 先抽一次：一半取根环里第四新的那条根（`RingRoot { index_from_newest: 3 }`），另一半照第一版的抽法。unlink 一个两片下段叶的文件
    /// 一次写出三条根（意图那次、一个删除批、最后一批）；删完的那一版没有文件、回退不接（执行器记「前提不满足」），建一个 inode 之后才接，
    /// 这时第四新的根就是意图那一版（三片的是第一个删除批那一版），回退到它把号带回队列（D8（核心索引结构） 已定项 16 依据
    /// 「回退到意图那一版会把号带回队列」）、之后的用户改动先走空多步（代码轮 U13）。
    FourthNewestRootHalfTheTime,
}

/// 覆盖写（`HistoryOperation::PublishOverwrite`）的内容按什么抽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverwriteContentDraw {
    /// 第一版的六种长度（`draw_content`）：都在一个数据单元以内或刚越过它，全走覆盖写入口。
    WithinOneDataUnitOrJustBeyond,
    /// 另抽跨几个数据单元、走顺序写入口的内容（[`ContentLength::SpanningDataUnitsThroughTheSequentialWrite`]），
    /// 多半多于一片下段叶（`draw_content_spanning_several_data_units`）。
    AlsoSpanningSeveralDataUnits,
}

/// 抬 F 的目标怎么取。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FloorTargetChoice {
    /// 新 F = 现行版本根记录的 F + `steps_above_current_floor mod (现行 txg − 现行 F + 3)`：最大到现行 txg + 2，
    /// 而上限（D16（发布语义） 已定项 1）不超过每块盘上最新的有效根 ≤ 现行 txg，所以超过上限的目标一定取得到。
    /// 生成器的前提：目标只取现行 F 及以上，不往下抬。没有条款明写，按入口的文档注释取的读法——
    /// `crates/singlefs-core/src/mount.rs` 的 `raise_rollback_floor` 文档注释「抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）」，已定项 1 在
    /// `.claude/kb/decisions/16-发布语义.md`「已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读」；往下的目标这个入口接不接、接了算不算合法没有条款（F 回落在增补 2 收口表第 ② 行里打回重议）。
    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着，2026-09-18 主 agent 定）。
    pub steps_above_current_floor: u64,
}

/// 历史里的一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HistoryOperation {
    /// `transaction::publish_first_file`，接在现行版本后面（上一条记录取现行版本的；照抄实例表指针的那条根也取现行版本的）。
    PublishFirstFile(ContentChoice),
    /// 整份重写文件内容，经挂着的会话发，接在现行的带文件的版本后面：内容跨几个数据单元的
    /// （[`ContentLength::SpanningDataUnitsThroughTheSequentialWrite`]）走顺序写（`UserChange::SequentialWrite`，代码轮 U13），
    /// 别的走覆盖写（`UserChange::Overwrite`，`transaction::publish_overwrite`）。
    PublishOverwrite(ContentChoice),
    /// `transaction::publish_without_units`，接在现行的树表 0 条的版本后面。
    PublishWithoutUnits,
    /// 关掉这个进程的可写会话，`mount::mount_writable`。
    CloseAndMountWritable,
    /// 管理员回退：挂着时的一次向前发布（`mount::roll_back_by_a_forward_publish`），接在现行的带文件的版本上。
    RollBackWhileMounted(RollbackTargetChoice),
    /// `mount::raise_rollback_floor`（影子账开着），接在现行的带文件的版本上。
    RaiseRollbackFloor(FloorTargetChoice),
    /// 关掉这个进程的可写会话，冷启动 `recovery::recover`（看 journal）。
    ColdStartRecover,
    /// 崩溃恢复抛弃根（D23（journal 的角色与格式） 已定项 14：影子账与按实例表判抛弃留着，理由是崩溃恢复）：这个进程崩掉，
    /// 下一次可写挂载时根环里最新那条根（这个会话的现行版本）的根槽与它那次发布的记录点名的单元整次挂载读不出（读回全 0；
    /// `DeviceReadingZerosOverHiddenRanges`：录制流里不多出写，挂载自己写到那几处的照常落盘）。那条根系统配置见证过，
    /// C554 乙之后读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），
    /// 这一步不再造出被抛弃的根；乙罩不到的「系统配置没见证到最新那条根」那一形（`tests/common/mod.rs` 的
    /// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 另把见证它的系统配置槽清掉）这一步不造。
    CrashRecoveryAbandoningTheNewestRoot,
    /// 经挂着的会话 unlink 第一个文件（`mounted_session::UserChange::Unlink`，D8（核心索引结构） 已定项 16）：一次调用 = 意图那次 +
    /// 从尾部全部分批（文件多于一片下段叶的 144 个单元时先有不是最后一批的删除批，再最后一批），中间为空间推的抬 F 照会话交回的比。
    /// 现行那一版有第一个文件那条记录时调（删除中的那一版也调：会话先走空、再拒）。
    Unlink,
    /// 经挂着的会话建一个 size 0 的 inode（`mounted_session::UserChange::NewInodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：
    /// 删完之后只许建 inode、写行与空发布）。只在第一个文件已删完的一版上调（有树而文件集合为空的一版、或在它上面建过 inode 的一版）：
    /// 模型只罩 inode 1 一个文件，建在第一个文件旁边的 inode 会让之后那次最后一批留下别的 inode，模型的删除那一臂罩不到。
    CreateInode,
    /// 经挂着的会话整份重写文件内容（入口分法同 [`HistoryOperation::PublishOverwrite`]），两块盘在那次改动的第一个 journal 记录写
    /// 到达时掉电：之前发出的写与屏障都落了盘，从那个写起一个都不落、都报 I/O 错（这一份里的 `DeviceLosingPowerAtTheFirstJournalRecordWrite`），
    /// 这个进程随之崩掉；之后照 [`HistoryOperation::CloseAndMountWritable`] 可写挂载一次。那次改动写下的单元是这个会话那个实例的孤儿：
    /// 下一个实例写行时给它写的那一行按所选根判它们未发布，根环里这个实例的根全被轮出去之后，挂载末尾的实例表清扫抹它们的头、
    /// 再发清扫发布删那一行（D18（块里携带什么信息） 已定项 11「回收」一条：抹头那一段只有这样的孤儿走得到）。
    /// 掉电截在第一条 journal 记录之前，那次改动没有任何一样能让恢复读到它的内容：交给模型与 RefFS 时与可写挂载相同。
    OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(ContentChoice),
    /// 关掉这个进程的可写会话，可写挂载期间每块盘自证过、世代号最大的那一槽系统配置读回全 0（单故障：D22（单元原子性怎么合成）
    /// 已定项 8「容错承诺是单故障：每盘一槽系统配置坏在承诺里」；`DeviceReadingZerosOverHiddenRanges`，挂载自己写到那一槽之后照常读得出）。
    /// 读回全 0 的槽自证不过、设备不报错，可写挂载照「一槽撕裂、另一槽自证过」往下走（同条 S-辨），取号那一写落到被藏的那一槽上：
    /// 崩溃注入里唯一一种在读故障下真的往盘上发写的步（C696（崩溃注入的读故障一支在随机历史里从没触发））。
    /// 只进它自己那一组比重（[`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`]），已有各组生成的历史不变。
    WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
}

/// 操作的种类：统计与比重用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistoryOperationKind {
    PublishFirstFile,
    PublishOverwrite,
    PublishWithoutUnits,
    CloseAndMountWritable,
    RollBackWhileMounted,
    RaiseRollbackFloor,
    ColdStartRecover,
    CrashRecoveryAbandoningTheNewestRoot,
    Unlink,
    CreateInode,
    OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
    WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
}

impl HistoryOperationKind {
    pub const ALL: [HistoryOperationKind; 12] = [
        HistoryOperationKind::PublishFirstFile,
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::PublishWithoutUnits,
        HistoryOperationKind::CloseAndMountWritable,
        HistoryOperationKind::RollBackWhileMounted,
        HistoryOperationKind::RaiseRollbackFloor,
        HistoryOperationKind::ColdStartRecover,
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
        HistoryOperationKind::Unlink,
        HistoryOperationKind::CreateInode,
        HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    ];
}

impl HistoryOperation {
    #[must_use]
    pub fn kind(&self) -> HistoryOperationKind {
        match self {
            HistoryOperation::PublishFirstFile(_) => HistoryOperationKind::PublishFirstFile,
            HistoryOperation::PublishOverwrite(_) => HistoryOperationKind::PublishOverwrite,
            HistoryOperation::PublishWithoutUnits => HistoryOperationKind::PublishWithoutUnits,
            HistoryOperation::CloseAndMountWritable => HistoryOperationKind::CloseAndMountWritable,
            HistoryOperation::RollBackWhileMounted(_) => HistoryOperationKind::RollBackWhileMounted,
            HistoryOperation::RaiseRollbackFloor(_) => HistoryOperationKind::RaiseRollbackFloor,
            HistoryOperation::ColdStartRecover => HistoryOperationKind::ColdStartRecover,
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
            }
            HistoryOperation::Unlink => HistoryOperationKind::Unlink,
            HistoryOperation::CreateInode => HistoryOperationKind::CreateInode,
            HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
            }
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            }
        }
    }
}

/// 按种子生成的一段历史：起点与操作序列。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedHistory {
    pub seed: HistorySeed,
    pub starting_point: HistoryStartingPoint,
    pub operations: Vec<HistoryOperation>,
}

/// 生成时对会话的乐观估计：只用来调各类操作的比重，不进执行——执行按真实结局走，估计错了那一步就记「前提不满足」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedSession {
    Closed,
    OpenWithoutFile,
    OpenWithFile,
    /// 这段历史 unlink 过第一个文件（之后照估计一直没有它）：只在比重写了「unlink 之后那张表」时才进这一估计
    /// （[`GenerationWeights::with_session_open_after_the_file_was_deleted`]），没写的那几组照旧按「开着、没有文件」抽。
    OpenAfterTheFileWasDeleted,
}

enum_listing_every_member! {
/// 复现入口认得的每一组比重的名字：一组一个成员，[`GenerationWeights::NAMED`] 照成员表生成。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedGenerationWeights {
    /// [`GenerationWeights::BROAD`]。
    Broad,
    /// [`GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR`]。
    ReuseAfterRaisingTheFloor,
    /// [`GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR`]。
    RollbackAfterRaisingTheFloor,
    /// [`GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`]。
    TowardTheAllocationRecordWall,
    /// [`GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`]。
    TowardTheUnitAreaWall,
    /// [`GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`]。
    DeletingFilesOfSeveralLowerLeaves,
    /// [`GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP`]。
    LeavingOrphansForTheInstanceTableSweep,
    /// [`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`]。
    NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
}
}

impl NamedGenerationWeights {
    /// 这个名字指的那一组比重。
    #[must_use]
    pub const fn weights(self) -> GenerationWeights {
        match self {
            NamedGenerationWeights::Broad => GenerationWeights::BROAD,
            NamedGenerationWeights::ReuseAfterRaisingTheFloor => {
                GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR
            }
            NamedGenerationWeights::RollbackAfterRaisingTheFloor => {
                GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR
            }
            NamedGenerationWeights::TowardTheAllocationRecordWall => {
                GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL
            }
            NamedGenerationWeights::TowardTheUnitAreaWall => {
                GenerationWeights::TOWARD_THE_UNIT_AREA_WALL
            }
            NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => {
                GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES
            }
            NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => {
                GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP
            }
            NamedGenerationWeights::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS
            }
        }
    }

    /// 复现行与复现入口的环境变量里这组比重的取值（`HistorySamplingVariable::Weights`）。
    #[must_use]
    pub const fn environment_value(self) -> &'static str {
        match self {
            NamedGenerationWeights::Broad => "broad",
            NamedGenerationWeights::ReuseAfterRaisingTheFloor => "reuse",
            NamedGenerationWeights::RollbackAfterRaisingTheFloor => "rollback",
            NamedGenerationWeights::TowardTheAllocationRecordWall => "wall",
            NamedGenerationWeights::TowardTheUnitAreaWall => "unit-area-wall",
            NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => "several-leaves",
            NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => "orphans",
            NamedGenerationWeights::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                "newest-system-configuration-slot-zeros"
            }
        }
    }
}

/// 生成一段历史时各样东西的比重（每张表写份数；操作那三张按百分比写，和为 100）：起点两种各占几份，三种会话估计下各类操作各占几份。
/// 抽法是 [0, 份数之和) 里取一个数、按表的次序落到哪一格：起点那张写成 1 : 1 时就是第一版的「种子的第一个数对 2 取余」，
/// 快档与大档的每个种子生成的历史与第一版逐项相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationWeights {
    /// 报告里的名字。
    pub name: &'static str,
    /// 复现入口认这一组用的名字（[`NamedGenerationWeights`] 的哪一个成员）：每一组比重都要写，新加一组就得给它一个成员，
    /// 成员又由 [`NamedGenerationWeights::weights`] 的穷举 `match` 找回这一组，[`GenerationWeights::NAMED`] 照成员表生成。
    pub named: NamedGenerationWeights,
    pub starting_points: &'static [(HistoryStartingPoint, u64)],
    pub with_session_closed: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_without_file: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_with_file: &'static [(HistoryOperationKind, u64)],
    /// unlink 过第一个文件之后（[`ExpectedSession::OpenAfterTheFileWasDeleted`]）各类操作各占几份；`None` 的那几组 unlink 之后照旧按
    /// 「开着、没有文件」那张表抽、之后的估计也照旧，生成的历史与加「建 inode」之前逐项相同（那几组的取样点与钉的值不动）。
    pub with_session_open_after_the_file_was_deleted:
        Option<&'static [(HistoryOperationKind, u64)]>,
    pub rollback_targets: RollbackTargetDraw,
    /// 覆盖写的内容按什么抽：只有偏向删除多片下段叶那一组抽跨几个单元的，别的几组照第一版，生成的历史不变。
    pub overwrite_contents: OverwriteContentDraw,
}

impl GenerationWeights {
    /// 快档与大档用的那一组。关着的会话只抽挂载与冷启动；树表 0 条的版本上多抽第一个文件，不然从 mkfs 起的历史多半先推零单元发布、
    /// 再也接不上第一个文件；带文件的版本上零单元发布只会记「前提不满足」，少抽。起点两种各半。
    ///
    /// 崩溃恢复抛弃根（`CrashRecoveryAbandoningTheNewestRoot`，实七加）的份数从紧挨在它前面的可写挂载那一格里切出来（带文件 12 → 9 + 3，
    /// 树表 0 条 6 → 4 + 2），两张表的和照旧是 100：同一个种子抽到的数落在哪一格，只有原先落在切出去那几份里的从可写挂载换成它，
    /// 别的一格不动；它不再抽随机数、生成时对会话的估计与可写挂载相同，所以之后几步抽到的操作也不变。
    /// 关着的会话上不抽它：它要这个进程的会话交回最新那一版的记录（点名的单元）。
    /// unlink（三·2 第 8 条）同一个切法：带文件那张表里可写挂载那一格再切 3 份给它、紧挨在可写挂载之后（9 → 6 + 3），和照旧 100；
    /// 它也不抽随机数，原先落在切出去那几份里的从可写挂载换成它，之后几步按 unlink 之后的估计（有树、没有文件）抽。
    pub const BROAD: GenerationWeights = GenerationWeights {
        name: "各类操作都抽（快档、大档）",
        named: NamedGenerationWeights::Broad,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 1),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 92),
            (HistoryOperationKind::ColdStartRecover, 8),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 60),
            (HistoryOperationKind::PublishWithoutUnits, 20),
            (HistoryOperationKind::CloseAndMountWritable, 4),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                2,
            ),
            (HistoryOperationKind::RollBackWhileMounted, 4),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 3),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 52),
            (HistoryOperationKind::RaiseRollbackFloor, 16),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::Unlink, 3),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
            (HistoryOperationKind::RollBackWhileMounted, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 40),
            (HistoryOperationKind::RaiseRollbackFloor, 15),
            (HistoryOperationKind::RollBackWhileMounted, 15),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
            (HistoryOperationKind::ColdStartRecover, 5),
            (HistoryOperationKind::PublishOverwrite, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 4),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：多从第一个文件起、多覆盖写、多抬 F、多可写挂载，
    /// 少回退与冷启动。罩住别的已回收记录要：抬 F 回收一批 1 槽的固定点单元，重开之后上一次挂载的聚簇段不再挡用户数据，
    /// 下一个 2 槽的数据单元落在两条相邻的已回收记录上。
    /// unlink 从带文件那张表的可写挂载那一格切 3 份（18 → 15 + 3，紧挨在它之后；切法同 [`GenerationWeights::BROAD`]）：
    /// 小盘上删除类发布不判式子、落点那一判照判（D28（挂载期承诺量） 已定项 1），用户 2026-10-02 定 Q-T「甲」拿它去撞落点那一判，种子照旧。
    pub const REUSE_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
        name: "偏向抬 F 之后的复用（第 121 行那一类的取样点）",
        named: NamedGenerationWeights::ReuseAfterRaisingTheFloor,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 85),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::CloseAndMountWritable, 4),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 2),
            (HistoryOperationKind::PublishOverwrite, 1),
            (HistoryOperationKind::RaiseRollbackFloor, 1),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 55),
            (HistoryOperationKind::RaiseRollbackFloor, 22),
            (HistoryOperationKind::CloseAndMountWritable, 15),
            (HistoryOperationKind::Unlink, 3),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 1),
            (HistoryOperationKind::PublishFirstFile, 1),
            (HistoryOperationKind::PublishWithoutUnits, 1),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 40),
            (HistoryOperationKind::RaiseRollbackFloor, 25),
            (HistoryOperationKind::CloseAndMountWritable, 20),
            (HistoryOperationKind::RollBackWhileMounted, 5),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 2),
            (HistoryOperationKind::PublishFirstFile, 2),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ]),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 理想模型那一格（B2：回退到 txg = F_生效 的根被拒）的专门取样点：多从第一个文件起、多覆盖写（攒非空根，抬 F 的上限才抬得动）、
    /// 多抬 F、多回退，少冷启动；回退的目标一半取候选集的下沿（`RollbackTargetDraw::FloorRootHalfTheTime`）。快档那组比重与抽法下
    /// 回退落到 F 那条根上一次都没有（2026-09-19 数：快档 96 段、第 121 行取样点 48 段都是 0 次；只换比重、不换抽法，192 段里 2 次）。
    pub const ROLLBACK_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
        name: "偏向抬 F 之后回退（B2 那一格的取样点）",
        named: NamedGenerationWeights::RollbackAfterRaisingTheFloor,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 85),
            (HistoryOperationKind::PublishWithoutUnits, 3),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 2),
            (HistoryOperationKind::PublishOverwrite, 1),
            (HistoryOperationKind::RaiseRollbackFloor, 1),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 40),
            (HistoryOperationKind::RaiseRollbackFloor, 25),
            (HistoryOperationKind::RollBackWhileMounted, 25),
            (HistoryOperationKind::CloseAndMountWritable, 7),
            (HistoryOperationKind::ColdStartRecover, 1),
            (HistoryOperationKind::PublishFirstFile, 1),
            (HistoryOperationKind::PublishWithoutUnits, 1),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::FloorRootHalfTheTime,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 原分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：一律从第一个文件起，带文件的版本上多半覆盖写，
    /// 夹着可写挂载、抬 F（回收之后复用改写记录，条数涨得慢）与少量回退，把分配记录攒过 812 条。分配记录树按绝对槽号按位置寻址之后
    /// （D8（核心索引结构） 已定项 14）那道墙拆了，这一段看的是越过 812 条之后多叶的树每一步都判绿、模型对得上。每一步的池级 checker 要判
    /// （第二轮判决第三节第 3 条：第一轮一步都不判，攻方两条只有 checker 看得见的变异在它上面一段都不红）：那一遍归 checker 档两段，
    /// 每一步的镜像经钩子交出去。
    pub const TOWARD_THE_ALLOCATION_RECORD_WALL: GenerationWeights = GenerationWeights {
        name: "越过原分配记录墙（812 条那一格的取样点）",
        named: NamedGenerationWeights::TowardTheAllocationRecordWall,
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[(HistoryOperationKind::CloseAndMountWritable, 100)],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 80),
            (HistoryOperationKind::CloseAndMountWritable, 12),
            (HistoryOperationKind::RaiseRollbackFloor, 5),
            (HistoryOperationKind::RollBackWhileMounted, 2),
            (HistoryOperationKind::ColdStartRecover, 1),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 单元区墙那一格（增补 3 第 2 件代码三方第二轮判决第三节第 2 条）的取样点，配 `HistoryDeviceWidth::UnitAreaOf256Slots` 跑：
    /// 一律从第一个文件起，带文件的版本上绝大多数是覆盖写（每次每盘占 10 槽），夹着少量可写挂载与抬 F 与回退。
    /// 这组比重在 256 槽的小盘上只走得到用户数据那一处（覆盖写的数据单元）的落点拒绝：提交内生块能回落到任一空槽，
    /// 每块盘上一个空槽都不剩才拒，那一处由写死的用例钉（`commit_generated_units_fall_back_when_the_segment_runs_out.rs`）。
    /// unlink 从带文件那张表的可写挂载那一格切 3 份（6 → 3 + 3，紧挨在它之后；切法与 Q-T「甲」同 [`GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR`]）。
    pub const TOWARD_THE_UNIT_AREA_WALL: GenerationWeights = GenerationWeights {
        name: "逼近单元区墙（小盘上落点拒绝那一格的取样点）",
        named: NamedGenerationWeights::TowardTheUnitAreaWall,
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 88),
            (HistoryOperationKind::CloseAndMountWritable, 3),
            (HistoryOperationKind::Unlink, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 4),
            (HistoryOperationKind::RollBackWhileMounted, 1),
            (HistoryOperationKind::ColdStartRecover, 1),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 删除多于一片下段叶的文件那一格（代码轮 U13：模型罩多个数据单元的文件、随机历史里出现 ≥ 2 片下段叶的删除批；
    /// 代码轮第二轮判决第二节 N3「走空多步」）的取样点，配两块 4 GiB 的盘跑：多从第一个文件起；带文件的版本上覆盖写的内容另抽跨几个单元、
    /// 走顺序写入口的（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]，多半两片下段叶），夹着很多 unlink（一次 unlink
    /// = 意图那次 + 从尾部每批 144 个单元 + 最后一批）；unlink 之后只抽建 inode、回退、覆盖写与 unlink（不抽挂载、抬 F、冷启动：
    /// 它们写的根会把意图那一版在根环里的名次往后推），回退一半取第四新的根（建过一个 inode 之后落在意图那一版或第一个删除批那一版上，
    /// 号回到队列），再建 inode、覆盖写或 unlink（会话先把队列走空，多片的文件走空多步）。快档那组比重不动：它的种子钉着的取样点与
    /// checker 档、实验装置的复现都按它生成。
    pub const DELETING_FILES_OF_SEVERAL_LOWER_LEAVES: GenerationWeights = GenerationWeights {
        name: "偏向删除多于一片下段叶的文件（U13 那一格的取样点）",
        named: NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 80),
            (HistoryOperationKind::CloseAndMountWritable, 10),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 45),
            (HistoryOperationKind::Unlink, 25),
            (HistoryOperationKind::RaiseRollbackFloor, 8),
            (HistoryOperationKind::CloseAndMountWritable, 8),
            (HistoryOperationKind::RollBackWhileMounted, 8),
            (HistoryOperationKind::ColdStartRecover, 3),
            (
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
                3,
            ),
        ],
        with_session_open_after_the_file_was_deleted: Some(&[
            (HistoryOperationKind::CreateInode, 40),
            (HistoryOperationKind::RollBackWhileMounted, 40),
            (HistoryOperationKind::PublishOverwrite, 10),
            (HistoryOperationKind::Unlink, 10),
        ]),
        rollback_targets: RollbackTargetDraw::FourthNewestRootHalfTheTime,
        overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
    };

    /// 实例表清扫抹头那一段（D18（块里携带什么信息） 已定项 11「回收」一条）的取样点：一律从第一个文件起，带文件的版本上
    /// 偶尔一次覆盖写掉电在第一个 journal 记录写之前、进程崩掉再挂（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]，
    /// 留下那个实例的孤儿），其余绝大多数是覆盖写（每次一条根，把孤儿那个实例的根从 24 槽的根环里轮出去），夹着少量可写挂载；
    /// 不抽回退、抬 F、冷启动与 unlink。别的几组比重不抽这一步：它们的种子钉着的取样点照旧。
    pub const LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP: GenerationWeights = GenerationWeights {
        name: "偏向留下孤儿实例（清扫抹头那一段的取样点）",
        named: NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep,
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[(HistoryOperationKind::CloseAndMountWritable, 100)],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 88),
            (
                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
                8,
            ),
            (HistoryOperationKind::CloseAndMountWritable, 4),
        ],
        with_session_open_after_the_file_was_deleted: None,
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

    /// 崩溃注入读故障那一支的取样点（C696（崩溃注入的读故障一支在随机历史里从没触发），用户定设计调查的候选「丁」）：与
    /// [`GenerationWeights::BROAD`] 逐格相同，只把崩溃恢复抛弃根那几格（份数照旧）换成「每块盘最新那一槽系统配置读回全 0 时的可写挂载」
    /// （[`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`]）。两种步都不抽随机数、生成时对会话的
    /// 估计都与可写挂载相同，所以同一个种子在这组比重下生成的历史与快档那一组逐步相同、只有那几步换了种类。已有各组的份数表里没有新步，
    /// 它们生成的历史不变（`tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs` 钉）。
    pub const NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS: GenerationWeights =
        GenerationWeights {
            name: "每块盘最新那一槽系统配置读回全 0 的可写挂载（崩溃注入读故障那一支的取样点）",
            named: NamedGenerationWeights::NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
            starting_points: &[
                (HistoryStartingPoint::AfterMakeFilesystem, 1),
                (HistoryStartingPoint::AfterFirstFile, 1),
            ],
            with_session_closed: &[
                (HistoryOperationKind::CloseAndMountWritable, 92),
                (HistoryOperationKind::ColdStartRecover, 8),
            ],
            with_session_open_without_file: &[
                (HistoryOperationKind::PublishFirstFile, 60),
                (HistoryOperationKind::PublishWithoutUnits, 20),
                (HistoryOperationKind::CloseAndMountWritable, 4),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
                    2,
                ),
                (HistoryOperationKind::RollBackWhileMounted, 4),
                (HistoryOperationKind::ColdStartRecover, 4),
                (HistoryOperationKind::PublishOverwrite, 3),
                (HistoryOperationKind::RaiseRollbackFloor, 3),
            ],
            with_session_open_with_file: &[
                (HistoryOperationKind::PublishOverwrite, 52),
                (HistoryOperationKind::RaiseRollbackFloor, 16),
                (HistoryOperationKind::CloseAndMountWritable, 6),
                (HistoryOperationKind::Unlink, 3),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
                    3,
                ),
                (HistoryOperationKind::RollBackWhileMounted, 10),
                (HistoryOperationKind::ColdStartRecover, 4),
                (HistoryOperationKind::PublishFirstFile, 4),
                (HistoryOperationKind::PublishWithoutUnits, 2),
            ],
            with_session_open_after_the_file_was_deleted: Some(&[
                (HistoryOperationKind::CreateInode, 40),
                (HistoryOperationKind::RaiseRollbackFloor, 15),
                (HistoryOperationKind::RollBackWhileMounted, 15),
                (HistoryOperationKind::CloseAndMountWritable, 10),
                (
                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
                    3,
                ),
                (HistoryOperationKind::ColdStartRecover, 5),
                (HistoryOperationKind::PublishOverwrite, 4),
                (HistoryOperationKind::PublishFirstFile, 4),
                (HistoryOperationKind::PublishWithoutUnits, 4),
            ]),
            rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
            overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
        };

    /// 复现入口认得的每一组比重：按 `environment_value` 找回（[`HistorySampling::from_environment`]）。
    /// 照 [`NamedGenerationWeights::ALL`] 逐个经 [`NamedGenerationWeights::weights`] 生成，不手抄常量名：
    /// 新加一组比重要一个新成员，成员不在 `weights` 的穷举 `match` 里补一臂就编不过（代码三方 test-script-align-code-r1 判决 K4）。
    pub const NAMED: [GenerationWeights; NamedGenerationWeights::ALL.len()] = {
        let mut named = [GenerationWeights::BROAD; NamedGenerationWeights::ALL.len()];
        let mut position = 0;
        while position < named.len() {
            named[position] = NamedGenerationWeights::ALL[position].weights();
            position += 1;
        }
        named
    };

    /// 复现行与复现入口的环境变量里这组比重的取值（`HistorySamplingVariable::Weights`）。
    #[must_use]
    pub const fn environment_value(&self) -> &'static str {
        self.named.environment_value()
    }

    fn for_session(&self, expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {
        match expected {
            ExpectedSession::Closed => self.with_session_closed,
            ExpectedSession::OpenWithoutFile => self.with_session_open_without_file,
            ExpectedSession::OpenWithFile => self.with_session_open_with_file,
            ExpectedSession::OpenAfterTheFileWasDeleted => self
                .with_session_open_after_the_file_was_deleted
                .expect("只在这一组比重写了 unlink 之后那张表时，生成才进这一估计（`expected_session_after` 的 unlink 那一臂）"),
        }
    }
}

fn draw_weighted<Choice: Copy>(
    source: &mut SeededRandomSource,
    weights: &[(Choice, u64)],
) -> Choice {
    let total: u64 = weights.iter().map(|(_, weight)| weight).sum();
    let mut remaining = source.below(total);
    for (choice, weight) in weights {
        if remaining < *weight {
            return *choice;
        }
        remaining -= weight;
    }
    unreachable!("remaining < total = 各比重之和，循环里一定有一项接住")
}

fn draw_content(source: &mut SeededRandomSource) -> ContentChoice {
    let selector = source.next_word();
    let length = draw_weighted(
        source,
        &[
            (ContentLength::Empty, 10),
            (ContentLength::InsideOneDataUnit { selector }, 60),
            (ContentLength::OneByteBelowDataUnitPayloadCapacity, 8),
            (ContentLength::ExactlyDataUnitPayloadCapacity, 8),
            (ContentLength::OneByteAboveDataUnitPayloadCapacity, 7),
            (ContentLength::ExactlyDataUnitBytes, 7),
        ],
    );
    ContentChoice {
        length,
        fill_seed: source.next_word(),
    }
}

/// 跨几个数据单元的内容（[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]）：与 `draw_content` 抽的随机数一样多（选择子、
/// 长度种类、填充种子各一个），五分之一在一个单元以内（走覆盖写），其余走顺序写——一片下段叶以内的几个单元（2–16 个）、两片
/// （145–152 个单元，删除要一个删除批加最后一批）、三片（289–292 个单元，两个删除批加最后一批）。单元数取得窄是为了每一步的池级 checker 读得快。
fn draw_content_spanning_several_data_units(source: &mut SeededRandomSource) -> ContentChoice {
    let selector = source.next_word();
    let length = draw_weighted(
        source,
        &[
            (ContentLength::InsideOneDataUnit { selector }, 20),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 2 + selector % 15,
                    selector,
                },
                15,
            ),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 8,
                    selector,
                },
                55,
            ),
            (
                ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 4,
                    selector,
                },
                10,
            ),
        ],
    );
    ContentChoice {
        length,
        fill_seed: source.next_word(),
    }
}

fn draw_operation(
    source: &mut SeededRandomSource,
    kind: HistoryOperationKind,
    rollback_targets: RollbackTargetDraw,
    overwrite_contents: OverwriteContentDraw,
) -> HistoryOperation {
    match kind {
        HistoryOperationKind::PublishFirstFile => {
            HistoryOperation::PublishFirstFile(draw_content(source))
        }
        HistoryOperationKind::PublishOverwrite => {
            HistoryOperation::PublishOverwrite(match overwrite_contents {
                OverwriteContentDraw::WithinOneDataUnitOrJustBeyond => draw_content(source),
                OverwriteContentDraw::AlsoSpanningSeveralDataUnits => {
                    draw_content_spanning_several_data_units(source)
                }
            })
        }
        HistoryOperationKind::PublishWithoutUnits => HistoryOperation::PublishWithoutUnits,
        HistoryOperationKind::CloseAndMountWritable => HistoryOperation::CloseAndMountWritable,
        HistoryOperationKind::RollBackWhileMounted => {
            match rollback_targets {
                RollbackTargetDraw::RecentUniformOrBeyond => {}
                RollbackTargetDraw::FloorRootHalfTheTime => {
                    if source.below(2) == 0 {
                        return HistoryOperation::RollBackWhileMounted(
                            RollbackTargetChoice::RingRootAtTheNewestFloor,
                        );
                    }
                }
                RollbackTargetDraw::FourthNewestRootHalfTheTime => {
                    if source.below(2) == 0 {
                        return HistoryOperation::RollBackWhileMounted(
                            RollbackTargetChoice::RingRoot {
                                index_from_newest: 3,
                            },
                        );
                    }
                }
            }
            // 一半取最近的几条（多半在候选集里），一半在整个根环里均匀取（被抛弃的、F 之下的、暖机那两条树表 0 条的都落得到），
            // 余下的取根环之外。
            let target = match source.below(5) {
                0 | 1 => RollbackTargetChoice::RingRoot {
                    index_from_newest: source.below(4),
                },
                2 | 3 => RollbackTargetChoice::RingRoot {
                    index_from_newest: source.next_word(),
                },
                _ => RollbackTargetChoice::BeyondNewestRoot {
                    txg_beyond_newest: source.below(3),
                },
            };
            HistoryOperation::RollBackWhileMounted(target)
        }
        HistoryOperationKind::RaiseRollbackFloor => {
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: source.next_word(),
            })
        }
        HistoryOperationKind::ColdStartRecover => HistoryOperation::ColdStartRecover,
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
        }
        HistoryOperationKind::Unlink => HistoryOperation::Unlink,
        HistoryOperationKind::CreateInode => HistoryOperation::CreateInode,
        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
        }
        HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
            HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
                match overwrite_contents {
                    OverwriteContentDraw::WithinOneDataUnitOrJustBeyond => draw_content(source),
                    OverwriteContentDraw::AlsoSpanningSeveralDataUnits => {
                        draw_content_spanning_several_data_units(source)
                    }
                },
            )
        }
    }
}

/// 生成时对「这段历史的第一个文件」的估计（只调比重、不进执行）：还没写过、写过、unlink 过之后一直没有（只在比重写了 unlink 之后那张表时进）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedFirstFile {
    NotWrittenYet,
    Written,
    DeletedAndFollowedByItsOwnTable,
}

fn expected_session_after(
    expected: ExpectedSession,
    first_file: ExpectedFirstFile,
    deletion_is_followed_by_its_own_table: bool,
    kind: HistoryOperationKind,
) -> ExpectedSession {
    match kind {
        // 崩溃恢复抛弃根也是一次可写挂载：落到的前一条根带不带文件，生成时照「这段历史发过文件没有」估，与可写挂载同一个估法；
        // 掉电在 journal 记录之前的那次覆盖写之后也是一次可写挂载，落到的是那次改动之前的一版；
        // 每块盘最新那一槽系统配置读回全 0 的那一次也是可写挂载，同一个估法。
        HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            match first_file {
                ExpectedFirstFile::Written => ExpectedSession::OpenWithFile,
                ExpectedFirstFile::NotWrittenYet => ExpectedSession::OpenWithoutFile,
                ExpectedFirstFile::DeletedAndFollowedByItsOwnTable => {
                    ExpectedSession::OpenAfterTheFileWasDeleted
                }
            }
        }
        HistoryOperationKind::ColdStartRecover => ExpectedSession::Closed,
        HistoryOperationKind::PublishFirstFile => match expected {
            ExpectedSession::OpenWithoutFile => ExpectedSession::OpenWithFile,
            // 删完之后第一个文件照条款被拒（树表不是 0 条），估计不变。
            ExpectedSession::Closed
            | ExpectedSession::OpenWithFile
            | ExpectedSession::OpenAfterTheFileWasDeleted => expected,
        },
        HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::RollBackWhileMounted
        | HistoryOperationKind::RaiseRollbackFloor
        | HistoryOperationKind::CreateInode => expected,
        // unlink 之后那一版有树而文件集合为空：比重写了 unlink 之后那张表的按它抽，没写的照旧按「开着、没有文件」抽
        // （第一个文件在它上面照条款被拒）。
        HistoryOperationKind::Unlink => match expected {
            ExpectedSession::OpenWithFile => {
                if deletion_is_followed_by_its_own_table {
                    ExpectedSession::OpenAfterTheFileWasDeleted
                } else {
                    ExpectedSession::OpenWithoutFile
                }
            }
            ExpectedSession::Closed
            | ExpectedSession::OpenWithoutFile
            | ExpectedSession::OpenAfterTheFileWasDeleted => expected,
        },
    }
}

/// 这一步之后对第一个文件的估计：unlink 之后比重写了那张表的一直记「删过」，没写的回到「还没写过」（与加「建 inode」之前的估法逐项相同）；
/// 别的操作之后，写过的仍是写过，估计开着带文件的也记写过。
fn expected_first_file_after(
    first_file: ExpectedFirstFile,
    deletion_is_followed_by_its_own_table: bool,
    kind: HistoryOperationKind,
    expected_after: ExpectedSession,
) -> ExpectedFirstFile {
    if kind == HistoryOperationKind::Unlink {
        return if deletion_is_followed_by_its_own_table {
            ExpectedFirstFile::DeletedAndFollowedByItsOwnTable
        } else {
            ExpectedFirstFile::NotWrittenYet
        };
    }
    match first_file {
        ExpectedFirstFile::DeletedAndFollowedByItsOwnTable | ExpectedFirstFile::Written => {
            first_file
        }
        ExpectedFirstFile::NotWrittenYet => {
            if expected_after == ExpectedSession::OpenWithFile {
                ExpectedFirstFile::Written
            } else {
                ExpectedFirstFile::NotWrittenYet
            }
        }
    }
}

/// 按种子生成一段 `operation_count` 步的历史，比重取 `GenerationWeights::BROAD`：同一个种子、同一个步数，逐项相同。
#[must_use]
pub fn generate_history(seed: HistorySeed, operation_count: usize) -> GeneratedHistory {
    generate_history_with_weights(seed, operation_count, &GenerationWeights::BROAD)
}

/// 按种子与比重生成一段 `operation_count` 步的历史：同一个种子、同一个步数、同一组比重，逐项相同。
#[must_use]
pub fn generate_history_with_weights(
    seed: HistorySeed,
    operation_count: usize,
    weights: &GenerationWeights,
) -> GeneratedHistory {
    let mut source = SeededRandomSource::from_seed(seed.0);
    let starting_point = draw_weighted(&mut source, weights.starting_points);
    let mut expected = match starting_point {
        HistoryStartingPoint::AfterMakeFilesystem => ExpectedSession::Closed,
        HistoryStartingPoint::AfterFirstFile => ExpectedSession::OpenWithFile,
    };
    let mut first_file = match starting_point {
        HistoryStartingPoint::AfterMakeFilesystem => ExpectedFirstFile::NotWrittenYet,
        HistoryStartingPoint::AfterFirstFile => ExpectedFirstFile::Written,
    };
    let deletion_is_followed_by_its_own_table = weights
        .with_session_open_after_the_file_was_deleted
        .is_some();
    let mut operations = Vec::with_capacity(operation_count);
    for _ in 0..operation_count {
        let kind = draw_weighted(&mut source, weights.for_session(expected));
        operations.push(draw_operation(
            &mut source,
            kind,
            weights.rollback_targets,
            weights.overwrite_contents,
        ));
        expected = expected_session_after(
            expected,
            first_file,
            deletion_is_followed_by_its_own_table,
            kind,
        );
        first_file = expected_first_file_after(
            first_file,
            deletion_is_followed_by_its_own_table,
            kind,
            expected,
        );
    }
    GeneratedHistory {
        seed,
        starting_point,
        operations,
    }
}

/// [`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`] 里掉电之后的那次可写挂载（实例 2）之后覆盖写几次。
/// 根环 24 槽（`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 每区 8 槽、三区）里实例 1 的根要全被轮出去：实例 2 的写行、两次暖机与
/// 之后 24 次覆盖写一共 27 条根；第 25 次是崩溃注入摆崩溃状态的那一步。
pub const OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE: u64 = 25;

/// 留下实例 1 的孤儿、再把实例 1 的根全轮出根环的一段写死的历史：从第一个文件起（实例 1 的会话开着），一次覆盖写掉电在它的第一个
/// journal 记录写之前、进程崩掉再可写挂载成实例 2（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]：
/// 写行把实例 1 那一行写进表，那次覆盖写写下的单元按这一行判未发布），再覆盖写 [`OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE`] 次。
/// 跑完之后（或崩在最后一次覆盖写之前）的盘面上，有一行就扫的可写挂载末尾清扫删实例 1 那一行、先抹它的孤儿的头
/// （D18（块里携带什么信息） 已定项 11「回收」一条）：抹头那一段从这里走得到。
#[must_use]
pub fn history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(
) -> GeneratedHistory {
    let mut operations = vec![
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
            ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 0 },
                fill_seed: 0,
            },
        ),
    ];
    operations.extend(
        (1..=OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE).map(|overwrite| {
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: overwrite,
                },
                fill_seed: overwrite,
            })
        }),
    );
    GeneratedHistory {
        seed: HistorySeed(3),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
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
    /// 走到时交回 [`StartingPointFailure`]，不 panic——判决第二节改法二（判定五坐实起点段今天只会撞出执行器自己的 `expect`）。
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
                    current: PoolVersion::WithFile(output),
                    instance,
                    // 执行器里的抬 F 一律开着影子账（`apply_raise_rollback_floor`），mkfs 那个进程的会话同一臂。
                    shadow_ledger: ShadowLedger::On,
                    parameters_and_device_table,
                    publishes_in_this_mount: 3,
                    carried_to_zero_unit_publishes: None,
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
            },
            verdict,
        ))
    }

    /// 两块盘此刻的整份镜像（拷一份，checker 与观察者读它）。
    fn image(&self) -> MemoryPool {
        image_of(&self.devices, self.device_width)
    }
}

/// 一步操作的前提此刻不成立，入口没调。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MissingPrecondition {
    /// 这个进程里没有可写会话（还没挂载、挂载被拒、冷启动之后）。
    NoWritableSession,
    /// 现行版本树表 0 条：覆盖写与抬 F 都要一个带文件的上一版（`TransactionOutput`）。
    CurrentVersionWithoutFile,
    /// 现行版本带文件：零单元发布只给树表 0 条的一版；带文件的一版上的空发布要重写固定点单元（分配记录树要改的那几个节点 + 记账 + 映射 + 树表），那是挂载内部的 `publish_version`，
    /// 不是这个入口。生成器的前提，没有条款明写调用约定，按入口的文档注释取的读法——`crates/singlefs-core/src/transaction.rs` 的 `publish_without_units`
    /// 文档注释「零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）」，已定项 9 在 `.claude/kb/decisions/16-发布语义.md`「已定项 9：一次空发布也写单元」。
    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着，2026-09-18 主 agent 定）。
    CurrentVersionWithFile,
    /// 按 checker 的读法，根环里一条可读根都没有：回退的目标无从取。
    NoReadableRootInRing,
    /// 崩溃恢复抛弃根：按 checker 的读法，根环里最新那条根不是这个会话的现行版本（例如那一版的根槽写被吞了），
    /// 要藏起来的根槽与点名单元从会话里取不到。
    NewestRingRootIsNotTheCurrentVersionOfThisSession,
    /// 崩溃恢复抛弃根：根环里除了最新那条之外一条可读根都没有，择根落不到前一条根上。
    NoReadableRootBeforeTheNewest,
    /// 崩溃恢复抛弃根：最新那次发布的记录一个单元都没点名（树表 0 条上的零单元发布）——点名验证恒过，恢复照样施加它、
    /// 走到的就是它，抛弃不了。
    NewestPublishNamesNoUnit,
    /// 现行版本有树、第一个文件已删完（在删完的一版上建过 inode 之后，实现归带文件的一版）：覆盖写与 unlink 只写、只删第一个文件
    /// （`transaction::live_record_of_the_first_file` 拒成 `WriteToAnInodeThatIsNotInTheTree`），模型只罩 inode 1 一个文件、没有这条理由——执行器不调。
    CurrentVersionWithoutTheFirstFile,
    /// 现行版本里第一个文件还活着：建 inode 只接在第一个文件已删完的一版上（`HistoryOperation::CreateInode` 的文档）；
    /// 删除中的那一版照调，会话先走空、把它走成删完的一版（代码轮 U11）。
    CurrentVersionStillHasTheFirstFile,
    /// 每块盘最新那一槽系统配置读回全 0 的可写挂载：有一块盘两槽里本池自证过的不到两份（mkfs 之后还没取过号的盘只写过一槽）——
    /// 再藏它自证过的那一槽，这块盘一份自证过的系统配置都不剩，那是那块盘两槽都坏，不是单故障（D22（单元原子性怎么合成） 已定项 8）。
    SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots,
}

impl MissingPrecondition {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            MissingPrecondition::NoWritableSession => "没有可写会话",
            MissingPrecondition::CurrentVersionWithoutFile => "现行版本树表 0 条",
            MissingPrecondition::CurrentVersionWithFile => "现行版本带文件",
            MissingPrecondition::NoReadableRootInRing => "根环里没有可读根",
            MissingPrecondition::NewestRingRootIsNotTheCurrentVersionOfThisSession => {
                "根环里最新那条根不是这个会话的现行版本"
            }
            MissingPrecondition::NoReadableRootBeforeTheNewest => "最新那条根之前没有可读根",
            MissingPrecondition::NewestPublishNamesNoUnit => "最新那次发布一个单元都没点名",
            MissingPrecondition::CurrentVersionWithoutTheFirstFile => "现行版本里第一个文件已删完",
            MissingPrecondition::CurrentVersionStillHasTheFirstFile => "现行版本里第一个文件还在",
            MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots => {
                "有盘两槽系统配置不是都自证过"
            }
        }
    }
}

/// 一次发布前后分配记录的变化：复用已回收的记录（同盘同槽那条从已释放改回已分配）、复用时跨度变了、已释放的记录被删掉、
/// 已释放的记录被这次新分配或改写的记录罩住起点（第 121 行那一类的路径：罩住了就该删，删没删另算）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordReuse {
    pub rewritten_from_released: u64,
    pub rewritten_with_changed_span: u64,
    pub released_records_removed: u64,
    pub released_records_covered: u64,
}

impl RecordReuse {
    /// 两份逐项相加（一次挂载写出的几次发布各比一次，加起来算这次挂载的）。
    #[must_use]
    pub fn plus(self, other: RecordReuse) -> RecordReuse {
        RecordReuse {
            rewritten_from_released: self.rewritten_from_released + other.rewritten_from_released,
            rewritten_with_changed_span: self.rewritten_with_changed_span
                + other.rewritten_with_changed_span,
            released_records_removed: self.released_records_removed
                + other.released_records_removed,
            released_records_covered: self.released_records_covered
                + other.released_records_covered,
        }
    }

    fn between(before: &[AllocationRecord], after: &[AllocationRecord]) -> Self {
        let released_before: BTreeMap<(DeviceIdentity, u64), &AllocationRecord> = before
            .iter()
            .filter(|record| record.is_released)
            .map(|record| ((record.device, record.slot.0), record))
            .collect();
        let places_after: BTreeSet<(DeviceIdentity, u64)> = after
            .iter()
            .map(|record| (record.device, record.slot.0))
            .collect();
        let mut reuse = RecordReuse::default();
        for record_after in after.iter().filter(|record| !record.is_released) {
            if let Some(record_before) =
                released_before.get(&(record_after.device, record_after.slot.0))
            {
                reuse.rewritten_from_released += 1;
                if record_before.span_slots != record_after.span_slots {
                    reuse.rewritten_with_changed_span += 1;
                }
            }
        }
        reuse.released_records_removed = u64::try_from(
            released_before
                .keys()
                .filter(|place| !places_after.contains(place))
                .count(),
        )
        .expect("条数");
        let records_before: BTreeSet<&AllocationRecord> = before.iter().collect();
        let allocated_by_this_publish: Vec<&AllocationRecord> = after
            .iter()
            .filter(|record| !record.is_released && !records_before.contains(record))
            .collect();
        reuse.released_records_covered = u64::try_from(
            released_before
                .values()
                .filter(|released| {
                    allocated_by_this_publish.iter().any(|allocated| {
                        allocated.device == released.device
                            && allocated.slot.0 < released.slot.0
                            && released.slot.0 < allocated.slot.0 + u64::from(allocated.span_slots)
                    })
                })
                .count(),
        )
        .expect("条数");
        reuse
    }
}

/// 一次发布改写或新增的分配记录（发布之后有、发布之前没有一模一样的一条）里，代不在 [`first_publish_txg`, `last_publish_txg`] 里的那些。
/// 一次发布改写的记录只有三种，代都是这次发布的 txg：新分配的（分配代）、复用改写已回收的（分配代改成这次的，D3（空间分配） 的
/// 「value = 分配代」）、这次释放的（释放代，`crates/singlefs-core/src/transaction.rs` 的 `allocator.release(*placement, txg);`）。
/// 盘上 checker 判不出分配代没改（33 条不变量里只有 I-3.9 看代，只看已释放的；增补 2 收口表第 44 行），这里在执行器里判。
#[must_use]
pub fn records_changed_with_a_generation_outside_the_publish_txgs(
    before: &[AllocationRecord],
    after: &[AllocationRecord],
    first_publish_txg: CheckpointTxg,
    last_publish_txg: CheckpointTxg,
) -> Vec<AllocationRecord> {
    let records_before: BTreeSet<&AllocationRecord> = before.iter().collect();
    after
        .iter()
        .filter(|record| !records_before.contains(record))
        .filter(|record| {
            record.generation < first_publish_txg || record.generation > last_publish_txg
        })
        .copied()
        .collect()
}

/// 执行器自己判的失败：不是 checker 的不变量、不是 panic，是入口交回的东西或结局自己就说明错了。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HarnessJudgement {
    /// 这一步发布改写或新增的分配记录，代不是这次发布的 txg（`records_changed_with_a_generation_outside_the_publish_txgs`）。
    AllocationGenerationIsNotThePublishTxg {
        records: Vec<AllocationRecord>,
        first_publish_txg: CheckpointTxg,
        last_publish_txg: CheckpointTxg,
    },
    /// 冷启动恢复报错，而这一步之前的镜像 checker 判过绿（判红的话历史已经停了；冷启动不写盘，镜像没变）。
    ColdStartRecoveryFailedOnCheckerGreenImage { outcome: String },
    /// 起点段（mkfs、取号、暖机、第一个文件）里有一个入口返回了错误。不注入故障时走不到：两块内存盘的读写不报错。
    /// 故障注入（增补 3 第 4 件）把注入点摆进起点段之后走得到，判定归那一路（`fault_injection::the_fault_surfaced_as_an_error`）。
    TheStartingPointReturnedAnError {
        step: StartingPointStep,
        error: String,
    },
}

impl HarnessJudgement {
    /// 签名与报告里的名字。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            HarnessJudgement::AllocationGenerationIsNotThePublishTxg { .. } => {
                "改写或新增的分配记录的代不是这次发布的 txg"
            }
            HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage { .. } => {
                "checker 判绿的镜像上冷启动恢复报错"
            }
            HarnessJudgement::TheStartingPointReturnedAnError { .. } => "起点段的入口返回了错误",
        }
    }
}

/// 起点段是哪一步：mkfs、取号、暖机、第一个文件。故障注入把注入点摆进起点段之后这四步都可能返回块设备错，
/// 执行器把错交回判定而不是 panic（增补 3 第 4 件代码三方第一轮判决改法二）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StartingPointStep {
    MakeFilesystem,
    /// mkfs 之后、取号之前，给这个进程的会话读盘上那份参数与盘表（`mount::ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`）。
    ReadThePoolForTheSession,
    AcquireInstance,
    WarmUp,
    PublishFirstFile,
}

impl StartingPointStep {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            StartingPointStep::MakeFilesystem => "mkfs",
            StartingPointStep::ReadThePoolForTheSession => "给会话读盘上的参数与盘表",
            StartingPointStep::AcquireInstance => "取号",
            StartingPointStep::WarmUp => "暖机",
            StartingPointStep::PublishFirstFile => "第一个文件",
        }
    }

    /// 这一步报错时，盘上有没有一个 mkfs 做完了的池。mkfs 自己就报错的那一格没有：那份盘面上没有文件系统，
    /// 模型一版都没提交过，池级 checker 与「重开要恢复到模型允许的版本」都没有可判的对象。
    #[must_use]
    pub const fn finished_filesystem_is_on_the_devices(self) -> bool {
        match self {
            StartingPointStep::MakeFilesystem => false,
            StartingPointStep::ReadThePoolForTheSession
            | StartingPointStep::AcquireInstance
            | StartingPointStep::WarmUp
            | StartingPointStep::PublishFirstFile => true,
        }
    }
}

/// 起点段停在哪一步、报的什么错，加上那一刻两块盘的整份镜像（起点段失败时池还没建起来，调用方拿不到 `HistoryPool`）。
pub struct StartingPointFailure {
    pub step: StartingPointStep,
    /// 那一步交回的错误成员，`{:?}` 出来的一行。
    pub error: String,
    pub image: MemoryPool,
    /// 录制流头上 mkfs 写的步数（mkfs 自己报错时就是这一刻录制流里的全部步数）。
    pub operations_written_by_make_filesystem: usize,
}

/// 一步操作的结局本身判不判失败：冷启动读回报错就算（不看读回的内容对不对，那是第 2 件模型的事）。
#[must_use]
pub fn harness_judgement_of_outcome(outcome: &StepOutcome) -> Option<HarnessJudgement> {
    match outcome {
        StepOutcome::Applied(AppliedEffect::Recovered { outcome, read_back }) => match read_back {
            ColdStartReadBack::Failed => Some(
                HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage {
                    outcome: outcome.clone(),
                },
            ),
            ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => None,
        },
        StepOutcome::Applied(
            AppliedEffect::Published { .. }
            | AppliedEffect::Mounted { .. }
            | AppliedEffect::RaisedFloor { .. }
            | AppliedEffect::RolledBack { .. },
        )
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_) => None,
    }
}

/// 一次可写挂载的空间准入怎么判的（`singlefs_core::mount::MountSpaceAdmission` 的成员，只带写行之后推了几串抬 F）：
/// 挂载那一处的推（D16（发布语义） 已定项 1「准入」那一行）真的跑到了没有，计数里看这个。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountSpaceAdmissionOutcome {
    AdmittedBeforeAcquisition,
    AdmittedAfterTheFloorRaises { floor_raises: usize },
    StillShortAfterTheFloorRaises { floor_raises: usize },
    NotJudgedByTheTestOnlySwitch,
}

impl MountSpaceAdmissionOutcome {
    #[must_use]
    pub fn of(space_admission: &singlefs_core::mount::MountSpaceAdmission) -> Self {
        use singlefs_core::mount::MountSpaceAdmission;
        match space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition => {
                MountSpaceAdmissionOutcome::AdmittedBeforeAcquisition
            }
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { floor_raises, .. } => {
                MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises {
                    floor_raises: floor_raises.len(),
                }
            }
            MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. } => {
                MountSpaceAdmissionOutcome::StillShortAfterTheFloorRaises {
                    floor_raises: floor_raises.len(),
                }
            }
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => {
                MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch
            }
        }
    }

    /// 计数里的名字（不带推了几串）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            MountSpaceAdmissionOutcome::AdmittedBeforeAcquisition => "取号之前就够",
            MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises { .. } => {
                "写行之后推抬 F 再判够了"
            }
            MountSpaceAdmissionOutcome::StillShortAfterTheFloorRaises { .. } => {
                "写行之后推抬 F 推满仍不够"
            }
            MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch => "只供测试的开关关掉准入",
        }
    }
}

/// 冷启动读回的三种结局（`recovery::RecoveryOutcome` 的成员，不带内容）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColdStartReadBack {
    NoFile,
    FileRead,
    Failed,
}

/// 入口返回 Ok 之后这一步做成了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppliedEffect {
    Published {
        reuse: RecordReuse,
        /// 这次发布之前挂着的会话推了几串抬 F（只有经会话的覆盖写与 unlink 会推，D16（发布语义） 已定项 1「准入」那一行、C283；
        /// unlink 连同走空里删除类发布被空间拒时推的，用户 2026-10-02 定 Q-V；第一个文件与零单元发布不经会话，恒为 0）。
        floor_raises_pushed_by_the_session: usize,
    },
    Mounted {
        instance: InstanceGeneration,
        publishes: usize,
        allocation_records_compared: MountAllocationComparison,
        /// 写行与暖机那几次发布里的复用（逐次相加）。
        reuse: RecordReuse,
        /// 这次挂载的空间准入怎么判的（`singlefs_core::mount::MountSpaceAdmission` 去掉细节）。
        space_admission: MountSpaceAdmissionOutcome,
    },
    RaisedFloor {
        new_floor: CheckpointTxg,
        publishes: usize,
        reclaimed_placements: usize,
        reuse: RecordReuse,
    },
    /// 管理员回退做成：一次向前发布，释放与复活的用户可见单元各几个（复活按盘逐份数）。
    RolledBack {
        target: RollbackTarget,
        released_user_visible_units: usize,
        resurrected_user_visible_copies: usize,
    },
    Recovered {
        outcome: String,
        read_back: ColdStartReadBack,
    },
}

/// 一步操作的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    /// 入口返回 Ok。
    Applied(AppliedEffect),
    /// 入口返回 Err：合法结局，带成员名（嵌套的连里层一起写）。
    Refused { member: String },
    /// 前提此刻不成立，入口没调。
    NotApplicable(MissingPrecondition),
}

/// 走到哪一步了：起点（mkfs，或 mkfs 加第一个文件）还是第几步操作（从 0 数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StepPosition {
    StartingPoint,
    Operation(usize),
}

/// panic 钩子记下的那一次：位置（文件:行）与消息。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapturedPanic {
    pub location: String,
    pub message: String,
}

thread_local! {
    static IS_CAPTURING_PANICS: Cell<bool> = const { Cell::new(false) };
    static CAPTURED_PANIC: RefCell<Option<CapturedPanic>> = const { RefCell::new(None) };
}

static INSTALL_PANIC_CAPTURE_HOOK: Once = Once::new();

fn payload_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "（panic 的载荷不是字符串）".to_string())
}

/// 装一次全进程的 panic 钩子：本线程在捕获（`with_panic_capture` 里）就只记位置与消息、不打印；别的线程与捕获之外照旧交给原来的钩子。
fn install_panic_capture_hook() {
    INSTALL_PANIC_CAPTURE_HOOK.call_once(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_details| {
            if IS_CAPTURING_PANICS.with(Cell::get) {
                let location = panic_details.location().map_or_else(
                    || "（没有位置）".to_string(),
                    |location| format!("{}:{}", location.file(), location.line()),
                );
                let message = payload_text(panic_details.payload());
                CAPTURED_PANIC.with(|slot| {
                    *slot.borrow_mut() = Some(CapturedPanic { location, message });
                });
            } else {
                previous_hook(panic_details);
            }
        }));
    });
}

/// 跑 `body`，panic 就接住、交回钩子记下的位置与消息。
/// 坏盘输入（增补 3 第 5 件）拿它兜着恢复、可写挂载与池级 checker：panic 是被测代码的缺口，要接住记下来，不是让用例崩。
pub fn with_panic_capture<Output>(body: impl FnOnce() -> Output) -> Result<Output, CapturedPanic> {
    install_panic_capture_hook();
    let was_capturing = IS_CAPTURING_PANICS.with(|flag| flag.replace(true));
    let result = catch_unwind(AssertUnwindSafe(body));
    IS_CAPTURING_PANICS.with(|flag| flag.set(was_capturing));
    result.map_err(|payload| {
        CAPTURED_PANIC
            .with(|slot| slot.borrow_mut().take())
            .unwrap_or_else(|| CapturedPanic {
                location: "（钩子没记到位置）".to_string(),
                message: payload_text(&*payload),
            })
    })
}

/// 一次失败（违例或 panic）连同它发生时的盘面事实，拿去对「已知红」清单。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureObservation {
    pub position: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
    /// checker 判红的不变量与它报的第一处。
    pub violations: Vec<(&'static str, String)>,
    pub panic: Option<CapturedPanic>,
    /// 按 checker 的读法从盘上读：根环里最新那条根的 txg；读不到时 None。
    pub newest_ring_root_txg: Option<u64>,
    /// 根环一圈的槽数 R × S（按 checker 从系统配置读出的几何）；读不到时 None。
    pub root_ring_slot_count: Option<u64>,
    /// 执行器自己判出的失败（分配代、冷启动读回）。
    pub harness_judgement: Option<HarnessJudgement>,
    /// 理想模型与实现对不上的那一格（第 2 件）。
    pub model_disagreement: Option<ModelDisagreement>,
    /// 抬 F 那一步（入口返回 Ok）之后判红时：抬之前的镜像上，新 F 那个 txg 上的根是不是全属于被抛弃的实例
    /// （`raised_floor_lands_only_on_abandoned_roots`）；别的步、读不出、那个 txg 上没有根，都是 None。
    /// 崩溃状态上算的是同一个谓词、读的是这个崩溃镜像自己：F 取镜像里最新那条根带的回退下界
    /// （`crash_injection::raised_floor_of_the_newest_root_lands_only_on_abandoned_roots`）。
    pub raised_floor_lands_only_on_abandoned_roots: Option<bool>,
    /// 记录核对器（`crash::check_records_against`）在这个盘面上判出的两类：活盘面那一路不跑它，恒是默认值（两项都 false）。
    pub record_check: RecordCheck,
}

impl FailureObservation {
    /// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
    #[must_use]
    pub fn root_ring_has_turned(&self) -> bool {
        root_ring_has_turned(self.newest_ring_root_txg, self.root_ring_slot_count)
    }
}

/// I-3.1 的违例文字 `盘 N：记账的已分配 Some(A)，遍历全部有效根得到 B；机理：…`（`singlefs-checker` 的 `walk.rs`）里的 A 与 B；
/// 读不出就 None。B 后面跟着机理标识那一段，所以只取紧跟着的那串数字，不把整段尾巴拿去 parse。
#[must_use]
pub fn allocated_and_walked_bytes(detail: &str) -> Option<(u64, u64)> {
    let after_allocated = detail.split_once("记账的已分配 Some(")?.1;
    let (allocated_text, rest) = after_allocated.split_once(')')?;
    Some((
        allocated_text.parse().ok()?,
        leading_number_after(rest, "遍历全部有效根得到 ")?,
    ))
}

/// `label` 之后紧跟着的那串十进制数字；`label` 不在、或它后面不是数字，都是 None。
fn leading_number_after(text: &str, label: &str) -> Option<u64> {
    text.split_once(label)?
        .1
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

/// I-3.1 判红时 checker 在说明文字里带的机理标识（`singlefs-checker` 的 `walk.rs` 写的那一段）：遍历覆盖了哪些根槽、
/// 剩下的按什么理由没覆盖。「已知红」清单按它分辨机理，不按签名（只有 I-3.1 红、记账多于遍历）认人。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationStatisticMechanism {
    /// 根环一圈的槽数 R × S。
    pub root_ring_slot_count: u64,
    /// 环里最新那条根的 txg。
    pub newest_root_txg: u64,
    /// 环里自证过的根槽个数。
    pub readable_root_slot_count: u64,
    /// 环里最老的那条自证过的根的 txg。
    pub oldest_readable_root_txg: u64,
    /// 遍历真走过的候选根槽个数。
    pub walked_root_slot_count: u64,
    /// 被最新根的实例表判成「被抛弃的时间线」而没走的根槽个数。
    pub root_slots_dropped_as_abandoned: u64,
    /// 最新根带的回退下界 F。
    pub rollback_floor: u64,
    /// 低于 F 而没走的根槽个数。
    pub root_slots_dropped_below_floor: u64,
}

/// 从 I-3.1 的说明文字里读机理标识；少一项就 None（说明这条违例不是那一段格式写出来的）。
#[must_use]
pub fn allocation_statistic_mechanism(detail: &str) -> Option<AllocationStatisticMechanism> {
    Some(AllocationStatisticMechanism {
        root_ring_slot_count: leading_number_after(detail, "根环槽数 ")?,
        newest_root_txg: leading_number_after(detail, "最新根 txg ")?,
        readable_root_slot_count: leading_number_after(detail, "环里自证过的根槽 ")?,
        oldest_readable_root_txg: leading_number_after(detail, "最老的自证过的根 txg ")?,
        walked_root_slot_count: leading_number_after(detail, "遍历的候选根槽 ")?,
        root_slots_dropped_as_abandoned: leading_number_after(detail, "被实例表判抛弃的根槽 ")?,
        rollback_floor: leading_number_after(detail, "回退下界 F ")?,
        root_slots_dropped_below_floor: leading_number_after(detail, "低于 F 的根槽 ")?,
    })
}

/// 「已知红」清单的一条：形态与它在增补 2 收口表里的那一行。
pub struct KnownRedForm {
    /// 增补 2 收口表（`.claude/kb/milestone/02-second-txn.md`「增补 2」）里的哪一行。
    pub closeout_table_row: &'static str,
    /// 形态：哪条不变量或哪种 panic、在什么操作之后。
    pub shape: &'static str,
    pub matches: fn(&FailureObservation) -> bool,
}

/// 「已知红」清单。修好一条就删一条，删掉之后那条的复现（`tests/random_histories.rs` 里钉着）要转绿。
/// 增补 2 收口表第 43 行（F 抬进被抛弃实例留下的空档、抬 F 之后只有 I-3.1 红、记账多于遍历）取丁-defer 修好
/// （用户 2026-09-28 定），它的最小复现（`tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）
/// 与随机历史那边原先的复现在 2026-10-03 的副本上都跑绿，那一条从清单里删了（代码轮收口表第 43 行那一项）；清单此刻是空的。
pub const KNOWN_RED_FORMS: [KnownRedForm; 0] = [];

/// 新发现的「同一个」：panic 按位置、违例按判红的不变量集合。收缩只留签名不变的删法。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureSignature {
    Panic {
        location: String,
    },
    HarnessJudgement {
        judgement: &'static str,
    },
    ModelDisagreement {
        aspect: &'static str,
    },
    /// 记录核对器判红的那几类（`crash::RecordCheck`，只有崩溃状态上跑得到）。
    RecordCheck {
        aspects: Vec<&'static str>,
    },
    CheckerViolations {
        invariants: Vec<&'static str>,
    },
}

/// 记录核对器判红的那几类的名字，按 `RecordCheck` 的字段序；两项都 false 时是空的。
#[must_use]
pub fn record_check_aspects(check: &RecordCheck) -> Vec<&'static str> {
    let RecordCheck {
        root_without_record,
        claimed_state_missing_unit,
    } = check;
    let mut aspects = Vec::new();
    if *root_without_record {
        aspects.push("root_without_record");
    }
    if *claimed_state_missing_unit {
        aspects.push("claimed_state_missing_unit");
    }
    aspects
}

impl FailureSignature {
    /// panic 先于执行器判的失败，执行器判的失败先于模型对不上，模型对不上先于记录核对器，记录核对器先于 checker 的违例
    /// （同一步里都有时签名取前一种，别的照样在观察里）。
    fn of(observation: &FailureObservation) -> Self {
        match (
            &observation.panic,
            &observation.harness_judgement,
            &observation.model_disagreement,
        ) {
            (Some(panic), Some(_) | None, Some(_) | None) => FailureSignature::Panic {
                location: panic.location.clone(),
            },
            (None, Some(judgement), Some(_) | None) => FailureSignature::HarnessJudgement {
                judgement: judgement.name(),
            },
            (None, None, Some(disagreement)) => FailureSignature::ModelDisagreement {
                aspect: disagreement.aspect.name(),
            },
            (None, None, None) => {
                let aspects = record_check_aspects(&observation.record_check);
                if aspects.is_empty() {
                    FailureSignature::CheckerViolations {
                        invariants: observation
                            .violations
                            .iter()
                            .map(|(invariant, _)| *invariant)
                            .collect(),
                    }
                } else {
                    FailureSignature::RecordCheck { aspects }
                }
            }
        }
    }
}

/// 一段历史怎么收尾。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HistoryEnding {
    /// 每一步都跑完，checker 一条违例都没有、没有 panic。
    Completed,
    /// 撞到「已知红」清单里第 `form` 条的形态。
    KnownRed {
        form: usize,
        observation: FailureObservation,
    },
    /// 清单外的失败。
    NewFinding {
        signature: FailureSignature,
        observation: FailureObservation,
    },
}

/// 某一类操作跑了几次、各落在哪种结局。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OperationTally {
    pub applied: u64,
    pub refused: u64,
    pub not_applicable: u64,
}

/// 跑过的历史的计数：证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryTally {
    pub histories: u64,
    pub histories_completed: u64,
    /// 「已知红」清单第几条 → 几段历史以它收尾。
    pub histories_ended_known_red: BTreeMap<usize, u64>,
    pub histories_ended_new_finding: u64,
    pub operations_by_kind: BTreeMap<HistoryOperationKind, OperationTally>,
    /// 发布（第一个文件、覆盖写）的内容长度，按长度种类 → 入口被调了几次。
    pub content_lengths_attempted: BTreeMap<&'static str, u64>,
    /// 发布的内容长度，按长度种类 → 入口返回 Ok 几次（代码三方第一轮攻方的 P3：装得下的每一种至少发成一次）。
    pub content_lengths_published: BTreeMap<&'static str, u64>,
    pub refusals_by_member: BTreeMap<String, u64>,
    pub not_applicable_by_precondition: BTreeMap<&'static str, u64>,
    pub recovery_outcomes: BTreeMap<String, u64>,
    pub most_publishes_in_one_mount: usize,
    /// 一步（起点算一步）里模型写进根环的根最多几条（[`crate::model::IdealModel::roots_written_since_make_filesystem`] 的差）：
    /// 执行器每一步断言它不多于根环槽数（[`crate::model::MODEL_ROOT_RING_SLOTS`]），观察者每步并一次模型根环才不漏版本。
    pub most_roots_written_by_the_model_in_one_step: u64,
    pub most_successful_mounts_in_one_history: usize,
    pub most_mount_attempts_in_one_history: usize,
    pub raises_that_reclaimed: u64,
    pub reclaimed_placements: u64,
    /// 管理员回退复活的用户可见单元（逐盘逐份）合计：复活那一支跑到了没有。
    pub resurrected_user_visible_copies: u64,
    pub records_rewritten_from_released: u64,
    pub records_rewritten_with_changed_span: u64,
    pub released_records_removed: u64,
    /// 已释放的记录被这次新分配或改写的记录罩住起点的次数（第 121 行那一类的路径跑到了；删没删不影响这个数）。
    pub released_records_covered: u64,
    /// 挂载写出的写行与暖机里，比过分配记录的发布次数；挂载之前生效根那棵分配记录树读不出、没比的挂载次数。
    pub mount_publishes_compared: u64,
    pub mounts_with_previous_records_unreadable: u64,
    /// 做成的可写挂载按空间准入怎么判的分（[`MountSpaceAdmissionOutcome::name`]）→ 几次；写行之后推过的那几次一共推了几串抬 F。
    pub mounts_by_space_admission: BTreeMap<&'static str, u64>,
    pub floor_raises_pushed_by_mounts: u64,
    /// 经挂着的会话发成的覆盖写里，发成之前会话推过抬 F 的次数与一共推了几串（发布那一处的推，D16（发布语义） 已定项 1「准入」那一行、C283）。
    pub user_changes_published_after_the_session_raised_the_floor: u64,
    pub floor_raises_pushed_by_the_session_before_a_published_change: u64,
    pub highest_checkpoint_txg: u64,
    /// 一步之后可写会话的分配器里最多有几条分配记录（两块盘合计）：分配记录树按位置寻址之后没有一个节点 812 条那道墙
    /// （D8（核心索引结构） 已定项 14），这个数越过 812 说明历史走过了原先那道墙。
    pub most_allocation_records_in_one_version: u64,
    pub histories_that_turned_the_root_ring: u64,
    /// 模型比过至少一个不是最后一批的删除批（≥ 2 片下段叶的文件的删除批，代码轮 U13）的段数。
    pub histories_with_a_deletion_batch_keeping_data_units: u64,
    /// 会话里一次用户改动之前先走空、走了至少两步、模型逐步比过的段数（代码轮第二轮判决第二节 N3「走空多步」）。
    pub histories_with_a_walk_of_several_steps_before_a_user_change: u64,
    /// 会话里一次用户改动之前先走空、走空被空间拒推了抬 F、模型逐串逐步比过的段数（N3「走空被空间拒」）。
    pub histories_with_a_walk_before_a_user_change_refused_for_space: u64,
    /// harness 就地跑池级 checker 的次数：那一遍归 checker 档两段，harness 一次都不跑（[`PerStepChecker`]），这一格恒为 0，
    /// 用例照它核 harness 没把逐步 checker 偷回来。
    pub checker_runs: u64,
    /// 这一步一个写都没发（录制流一步没多）：镜像逐字节不变，checker 的结论沿用上一次，不重跑。
    pub checker_runs_skipped_because_nothing_was_written: u64,
    /// 经每步镜像那个钩子交出去的镜像张数（[`execute_history_handing_out_every_step_image`]）：起点之后、每一步写过盘之后各一张，
    /// 起点段报错而 mkfs 做完了的那一张也算；跑不跑池级 checker 都交。
    pub step_images_handed_out: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    /// 判成「部分没判」（`singlefs_checker::image::InvariantVerdict::PartiallyJudged`）的次数：不红，也不算判绿。
    pub invariant_partially_judged: BTreeMap<&'static str, u64>,
    /// 理想模型判过的步数（起点那次第一个文件也算一步；前提不满足、入口没调的不算）与各格计数。
    pub model_judged_steps: u64,
    pub model_counts: ModelJudgementCounts,
}

impl HistoryTally {
    /// 把另一份计数并进来。
    pub fn absorb(&mut self, other: &HistoryTally) {
        self.histories += other.histories;
        self.histories_completed += other.histories_completed;
        for (form, count) in &other.histories_ended_known_red {
            *self.histories_ended_known_red.entry(*form).or_insert(0) += count;
        }
        self.histories_ended_new_finding += other.histories_ended_new_finding;
        for (kind, tally) in &other.operations_by_kind {
            let entry = self.operations_by_kind.entry(*kind).or_default();
            entry.applied += tally.applied;
            entry.refused += tally.refused;
            entry.not_applicable += tally.not_applicable;
        }
        for (length, count) in &other.content_lengths_attempted {
            *self.content_lengths_attempted.entry(length).or_insert(0) += count;
        }
        for (length, count) in &other.content_lengths_published {
            *self.content_lengths_published.entry(length).or_insert(0) += count;
        }
        for (member, count) in &other.refusals_by_member {
            *self.refusals_by_member.entry(member.clone()).or_insert(0) += count;
        }
        for (precondition, count) in &other.not_applicable_by_precondition {
            *self
                .not_applicable_by_precondition
                .entry(precondition)
                .or_insert(0) += count;
        }
        for (outcome, count) in &other.recovery_outcomes {
            *self.recovery_outcomes.entry(outcome.clone()).or_insert(0) += count;
        }
        self.most_publishes_in_one_mount = self
            .most_publishes_in_one_mount
            .max(other.most_publishes_in_one_mount);
        self.most_roots_written_by_the_model_in_one_step = self
            .most_roots_written_by_the_model_in_one_step
            .max(other.most_roots_written_by_the_model_in_one_step);
        self.most_successful_mounts_in_one_history = self
            .most_successful_mounts_in_one_history
            .max(other.most_successful_mounts_in_one_history);
        self.most_mount_attempts_in_one_history = self
            .most_mount_attempts_in_one_history
            .max(other.most_mount_attempts_in_one_history);
        self.raises_that_reclaimed += other.raises_that_reclaimed;
        self.reclaimed_placements += other.reclaimed_placements;
        self.resurrected_user_visible_copies += other.resurrected_user_visible_copies;
        self.records_rewritten_from_released += other.records_rewritten_from_released;
        self.records_rewritten_with_changed_span += other.records_rewritten_with_changed_span;
        self.released_records_removed += other.released_records_removed;
        self.released_records_covered += other.released_records_covered;
        self.mount_publishes_compared += other.mount_publishes_compared;
        self.mounts_with_previous_records_unreadable +=
            other.mounts_with_previous_records_unreadable;
        for (outcome, count) in &other.mounts_by_space_admission {
            *self.mounts_by_space_admission.entry(outcome).or_insert(0) += count;
        }
        self.floor_raises_pushed_by_mounts += other.floor_raises_pushed_by_mounts;
        self.user_changes_published_after_the_session_raised_the_floor +=
            other.user_changes_published_after_the_session_raised_the_floor;
        self.floor_raises_pushed_by_the_session_before_a_published_change +=
            other.floor_raises_pushed_by_the_session_before_a_published_change;
        self.highest_checkpoint_txg = self
            .highest_checkpoint_txg
            .max(other.highest_checkpoint_txg);
        self.most_allocation_records_in_one_version = self
            .most_allocation_records_in_one_version
            .max(other.most_allocation_records_in_one_version);
        self.histories_that_turned_the_root_ring += other.histories_that_turned_the_root_ring;
        self.histories_with_a_deletion_batch_keeping_data_units +=
            other.histories_with_a_deletion_batch_keeping_data_units;
        self.histories_with_a_walk_of_several_steps_before_a_user_change +=
            other.histories_with_a_walk_of_several_steps_before_a_user_change;
        self.histories_with_a_walk_before_a_user_change_refused_for_space +=
            other.histories_with_a_walk_before_a_user_change_refused_for_space;
        self.checker_runs += other.checker_runs;
        self.checker_runs_skipped_because_nothing_was_written +=
            other.checker_runs_skipped_because_nothing_was_written;
        self.step_images_handed_out += other.step_images_handed_out;
        for (invariant, count) in &other.invariant_holds {
            *self.invariant_holds.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in &other.invariant_partially_judged {
            *self
                .invariant_partially_judged
                .entry(invariant)
                .or_insert(0) += count;
        }
        self.model_judged_steps += other.model_judged_steps;
        self.model_counts.add(&other.model_counts);
    }

    fn note_model_verdict(&mut self, verdict: &ModelVerdict) {
        self.model_judged_steps += 1;
        self.model_counts.add(&verdict.counts);
    }

    fn note_reuse(&mut self, reuse: RecordReuse) {
        self.records_rewritten_from_released += reuse.rewritten_from_released;
        self.records_rewritten_with_changed_span += reuse.rewritten_with_changed_span;
        self.released_records_removed += reuse.released_records_removed;
        self.released_records_covered += reuse.released_records_covered;
    }

    fn note_outcome(&mut self, operation: &HistoryOperation, outcome: &StepOutcome) {
        let entry = self.operations_by_kind.entry(operation.kind()).or_default();
        match outcome {
            StepOutcome::Applied(_) => entry.applied += 1,
            StepOutcome::Refused { .. } => entry.refused += 1,
            StepOutcome::NotApplicable(_) => entry.not_applicable += 1,
        }
        match outcome {
            StepOutcome::Applied(AppliedEffect::Published {
                reuse,
                floor_raises_pushed_by_the_session,
            }) => {
                self.note_reuse(*reuse);
                if *floor_raises_pushed_by_the_session > 0 {
                    self.user_changes_published_after_the_session_raised_the_floor += 1;
                    self.floor_raises_pushed_by_the_session_before_a_published_change +=
                        u64::try_from(*floor_raises_pushed_by_the_session)
                            .expect("一次准入推的串数装得进 u64");
                }
            }
            StepOutcome::Applied(AppliedEffect::RaisedFloor {
                reclaimed_placements,
                reuse,
                ..
            }) => {
                if *reclaimed_placements > 0 {
                    self.raises_that_reclaimed += 1;
                }
                self.reclaimed_placements += u64::try_from(*reclaimed_placements).expect("落点数");
                self.note_reuse(*reuse);
            }
            StepOutcome::Applied(AppliedEffect::Recovered { outcome, .. }) => {
                *self.recovery_outcomes.entry(outcome.clone()).or_insert(0) += 1;
            }
            StepOutcome::Applied(AppliedEffect::RolledBack {
                resurrected_user_visible_copies,
                ..
            }) => {
                self.resurrected_user_visible_copies +=
                    u64::try_from(*resurrected_user_visible_copies).expect("份数");
            }
            StepOutcome::Applied(AppliedEffect::Mounted {
                allocation_records_compared,
                reuse,
                space_admission,
                ..
            }) => {
                self.note_reuse(*reuse);
                *self
                    .mounts_by_space_admission
                    .entry(space_admission.name())
                    .or_insert(0) += 1;
                let floor_raises = match space_admission {
                    MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises { floor_raises }
                    | MountSpaceAdmissionOutcome::StillShortAfterTheFloorRaises { floor_raises } => {
                        *floor_raises
                    }
                    MountSpaceAdmissionOutcome::AdmittedBeforeAcquisition
                    | MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch => 0,
                };
                self.floor_raises_pushed_by_mounts +=
                    u64::try_from(floor_raises).expect("一次挂载推的串数装得进 u64");
                match allocation_records_compared {
                    MountAllocationComparison::Compared { publishes } => {
                        self.mount_publishes_compared +=
                            u64::try_from(*publishes).expect("一次挂载至多几次发布");
                    }
                    MountAllocationComparison::NoPublishWithFile => {}
                    MountAllocationComparison::PreviousRecordsUnreadable => {
                        self.mounts_with_previous_records_unreadable += 1;
                    }
                }
            }
            StepOutcome::Refused { member } => {
                *self.refusals_by_member.entry(member.clone()).or_insert(0) += 1;
            }
            StepOutcome::NotApplicable(precondition) => {
                *self
                    .not_applicable_by_precondition
                    .entry(precondition.name())
                    .or_insert(0) += 1;
            }
        }
        let content = match operation {
            HistoryOperation::PublishFirstFile(content)
            | HistoryOperation::PublishOverwrite(content) => Some(content),
            HistoryOperation::PublishWithoutUnits
            | HistoryOperation::CloseAndMountWritable
            | HistoryOperation::RollBackWhileMounted(_)
            | HistoryOperation::RaiseRollbackFloor(_)
            | HistoryOperation::ColdStartRecover
            | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
            | HistoryOperation::Unlink
            | HistoryOperation::CreateInode
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            // 那次改动掉电在 journal 记录之前、一次都没发布成：内容长度不记进「进过入口」「发布过」（结局是那之后可写挂载的结局）。
            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
                None
            }
        };
        let entrance_was_called = match outcome {
            StepOutcome::Applied(_) | StepOutcome::Refused { .. } => true,
            StepOutcome::NotApplicable(_) => false,
        };
        if let (Some(content), true) = (content, entrance_was_called) {
            *self
                .content_lengths_attempted
                .entry(content.length.name())
                .or_insert(0) += 1;
        }
        if let (Some(content), StepOutcome::Applied(_)) = (content, outcome) {
            *self
                .content_lengths_published
                .entry(content.length.name())
                .or_insert(0) += 1;
        }
    }

    /// 给人看的一整块：每类操作的结局、见过的 `Err` 成员、冷启动的结局、会话与挂载的最大值、复用、checker 跑了几次与每条不变量判绿几次。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；根环转过一圈的 {} 段；最高 txg {}；一版里最多 {} 条分配记录",
            self.histories,
            self.histories_completed,
            self.histories_ended_known_red,
            self.histories_ended_new_finding,
            self.histories_that_turned_the_root_ring,
            self.highest_checkpoint_txg,
            self.most_allocation_records_in_one_version
        );
        for kind in HistoryOperationKind::ALL {
            let tally = self
                .operations_by_kind
                .get(&kind)
                .copied()
                .unwrap_or_default();
            let _ = writeln!(
                text,
                "  操作 {kind:?}：Ok {}、Err {}、前提不满足没调 {}",
                tally.applied, tally.refused, tally.not_applicable
            );
        }
        for (length, count) in &self.content_lengths_attempted {
            let published = self
                .content_lengths_published
                .get(length)
                .copied()
                .unwrap_or(0);
            let _ = writeln!(
                text,
                "  内容长度 {length}：入口被调 {count} 次、发成 {published} 次"
            );
        }
        for (member, count) in &self.refusals_by_member {
            let _ = writeln!(text, "  Err 成员 {member}：{count} 次");
        }
        for (precondition, count) in &self.not_applicable_by_precondition {
            let _ = writeln!(text, "  前提不满足（{precondition}）：{count} 次");
        }
        for (outcome, count) in &self.recovery_outcomes {
            let _ = writeln!(text, "  冷启动结局 {outcome}：{count} 次");
        }
        let _ = writeln!(
            text,
            "  一次挂载里最多写出 {} 条根；模型一步里最多写进根环 {} 条根；一段历史里最多挂载成功 {} 次、最多试挂载 {} 次",
            self.most_publishes_in_one_mount,
            self.most_roots_written_by_the_model_in_one_step,
            self.most_successful_mounts_in_one_history,
            self.most_mount_attempts_in_one_history
        );
        let _ = writeln!(
            text,
            "  抬 F 回收到落点 {} 次、共回收 {} 个落点；复用已释放的记录 {} 条（其中跨度变了 {} 条）；已释放的记录被罩住起点 {} 条、被删 {} 条",
            self.raises_that_reclaimed,
            self.reclaimed_placements,
            self.records_rewritten_from_released,
            self.records_rewritten_with_changed_span,
            self.released_records_covered,
            self.released_records_removed
        );
        let _ = writeln!(
            text,
            "  挂载写出的发布比过分配记录 {} 次；挂载之前生效根的分配记录读不出、没比的挂载 {} 次",
            self.mount_publishes_compared, self.mounts_with_previous_records_unreadable
        );
        for (outcome, count) in &self.mounts_by_space_admission {
            let _ = writeln!(text, "  可写挂载的空间准入「{outcome}」：{count} 次");
        }
        let _ = writeln!(
            text,
            "  可写挂载写行之后一共推了 {} 串抬 F",
            self.floor_raises_pushed_by_mounts
        );
        let _ = writeln!(
            text,
            "  经会话的覆盖写：会话先推抬 F 再发成 {} 次、一共推了 {} 串",
            self.user_changes_published_after_the_session_raised_the_floor,
            self.floor_raises_pushed_by_the_session_before_a_published_change
        );
        let _ = writeln!(
            text,
            "  删除：模型比过不是最后一批的删除批（≥ 2 片下段叶）{} 次、在 {} 段里；会话改动之前先走空走了至少两步 {} 次、在 {} 段里；走空被空间拒推了抬 F {} 次、在 {} 段里",
            self.model_counts.deletion_batches_keeping_data_units_matched,
            self.histories_with_a_deletion_batch_keeping_data_units,
            self.model_counts
                .walks_of_several_steps_before_a_user_change_matched,
            self.histories_with_a_walk_of_several_steps_before_a_user_change,
            self.model_counts
                .walks_before_a_user_change_refused_for_space_matched,
            self.histories_with_a_walk_before_a_user_change_refused_for_space
        );
        let _ = writeln!(
            text,
            "  checker 跑了 {} 次；一个写都没发、沿用上一次结论的 {} 步；每步镜像交出去 {} 张",
            self.checker_runs,
            self.checker_runs_skipped_because_nothing_was_written,
            self.step_images_handed_out
        );
        let counts = &self.model_counts;
        let _ = writeln!(
            text,
            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次；单元区墙按区间放行 {} 次",
            self.model_judged_steps,
            counts.required_refusals_matched,
            counts.permitted_refusals_taken,
            counts.successes_matched,
            counts.roots_compared,
            counts.allocation_records_compared,
            counts.cold_start_contents_compared,
            counts.ceilings_compared,
            counts.rollbacks_accepted_at_the_effective_floor,
            counts.unit_area_wall_refusals_in_the_interval
        );
        // 每一版比没比实例表与分配代（代码审阅第 12 条）：树表 0 条的一版里「比不了」与「只比重写的角色」只剩一次挂载里第一版就是
        // 零单元发布的那几版（实审 B3c-4 往下带之后）；「比不了」另含带文件的那两格（mkfs 之后第一次重写实例表之前、从盘上重建且多于一片）。
        // 这几个数跑到了多少看这一行。
        let _ = writeln!(
            text,
            "  模型比每一版的实例表：比过 {} 张、输出不带整条链比不了 {} 张；零单元发布只比重写的角色 {} 次；比过文件内容 {} 次",
            counts.instance_tables_compared,
            counts.instance_tables_not_in_the_output,
            counts.rewritten_role_sets_compared,
            counts.file_contents_compared
        );
        for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
            let _ = writeln!(
                text,
                "  {invariant}：判绿 {} 次、不适用 {} 次、部分没判 {} 次",
                self.invariant_holds.get(invariant).copied().unwrap_or(0),
                self.invariant_not_applicable
                    .get(invariant)
                    .copied()
                    .unwrap_or(0),
                self.invariant_partially_judged
                    .get(invariant)
                    .copied()
                    .unwrap_or(0)
            );
        }
        text
    }
}

/// 一段历史跑完交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRun {
    pub ending: HistoryEnding,
    /// 每一步操作的结局，按次序；失败在第几步就到第几步为止（panic 的那一步没有结局）。
    pub outcomes: Vec<StepOutcome>,
    pub tally: HistoryTally,
    /// mkfs 占了录制流开头的几步：崩溃注入（增补 3 第 3 件）的基线与崩溃点重放一样取「mkfs 之后」——
    /// mkfs 不是事务，它写到一半的盘面上没有池，恢复报不出根是对的，不该拿事务的 oracle 去判
    /// （崩溃点重放那一路同样从 `mkfs_operation_count` 之后起枚举，见 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`）。
    pub operations_written_by_make_filesystem: usize,
    /// 这段历史停下时模型根环里的每一条根（`IdealModel::committed_versions`）：故障注入（增补 3 第 4 件）拿它当
    /// 「模型认下来的每一版」判重开走到的那一版——失败那一步的根，模型认了就在里面，没认（判定对不上、模型没往前走）就不在。
    /// 观察者那一路看不到这一份：判出失败的那一步不调观察者，而失败常常出在模型已经认下那一版之后（checker 在那一步判红）。
    /// 内容拷成 `Vec<u8>` 不留 `Rc`：`HistoryRun` 要跨线程送（`run_history_campaign` 的那张表），`Rc` 不是 `Send`。
    pub committed_versions_at_the_end: Vec<(ModelRootKey, Option<Vec<u8>>)>,
    /// 每次掉电（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]）之后录制流里第一步的下标，按次序：
    /// 掉电之前那一截与之后可写挂载另起的实例写的流在这里断开。崩溃注入把它交给记录核对器
    /// （`crate::memory_pool::RecordStreamSeam::AfterAPowerLoss`）：那次改动掉电之前落了盘的单元写没有根，
    /// 不归掉电之后的第一次发布。
    pub earliest_stream_index_after_each_power_loss: Vec<usize>,
}

/// 每一步之后交给观察者的东西：第几步、这一步是什么、结局、此刻的整份镜像、跟到这一步的理想模型。
/// 只在这一步没判出失败时调（判出失败的那一步直接停下，观察者看不到），所以模型与镜像都是判过的状态。
pub struct StepObservation<'run> {
    pub position: StepPosition,
    pub operation: Option<&'run HistoryOperation>,
    pub outcome: Option<&'run StepOutcome>,
    pub image: &'run MemoryPool,
    /// 崩溃注入（增补 3 第 3 件）每一步之后从它取一次 `committed_versions`，攒成「模型提交过的每一版」。
    pub model: &'run IdealModel,
}

fn placement_refusal_member(refusal: &PlacementRefusal) -> &'static str {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => "NoFreeSlotOnAnyDevice",
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. } => {
            "SomeDevicesFullDeviceSetSelectionUndefined"
        }
        PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. } => {
            "UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported"
        }
        PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        } => "CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined",
    }
}

fn publish_error_member(error: &PublishError) -> String {
    let member = match error {
        PublishError::PlacementRefused { refusal, .. } => {
            return format!(
                "PublishError::PlacementRefused({})",
                placement_refusal_member(refusal)
            )
        }
        PublishError::SpaceAdmissionRefused(_) => "SpaceAdmissionRefused",
        PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. } => {
            "AllocationRecordTreeRewriteSetDidNotSettle"
        }
        PublishError::MultiLevelCodeTwoTreeRefused { .. } => "MultiLevelCodeTwoTreeRefused",
        PublishError::ReleaseNotInMapping { .. } => "ReleaseNotInMapping",
        PublishError::ReleaseTargetNotAllocated { .. } => "ReleaseTargetNotAllocated",
        PublishError::ReleaseTargetAlreadyReleased { .. } => "ReleaseTargetAlreadyReleased",
        PublishError::ReleaseTargetLocationsOnDifferentSlots { .. } => {
            "ReleaseTargetLocationsOnDifferentSlots"
        }
        PublishError::ReleaseSpanMismatch { .. } => "ReleaseSpanMismatch",
        PublishError::MappingEntryNarrowerThanItsFieldTable { .. } => {
            "MappingEntryNarrowerThanItsFieldTable"
        }
        PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. } => {
            "MappingEntryLocationOnADeviceOutsideThePool"
        }
        PublishError::MappingEntryLocationsOnTheSameDevice { .. } => {
            "MappingEntryLocationsOnTheSameDevice"
        }
        PublishError::ContentExceedsDataUnit { .. } => "ContentExceedsDataUnit",
        PublishError::InodeTreeWriteRefused(_) => "InodeTreeWriteRefused",
        PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
            "FirstFileVersionDoesNotFollowTheVersionItBuildsOn"
        }
        PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. } => {
            "FirstFileVersionOnAVersionThatAlreadyHasAFile"
        }
        PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. } => {
            "TreeTableOfTheVersionToBuildOnUnreadable"
        }
        PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_) => {
            "TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees"
        }
        PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. } => {
            "PublishFrozenAfterAWriteFailureIsNotResentYet"
        }
        PublishError::JournalRecordsOfThePublishExceedTheLimit { .. } => {
            "JournalRecordsOfThePublishExceedTheLimit"
        }
        PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. } => {
            "NextCheckpointTxgPastTheTopOfItsRange"
        }
        PublishError::NextContainerNumberPastTheTopOfItsRange(_) => {
            "NextContainerNumberPastTheTopOfItsRange"
        }
        PublishError::WriteToAnInodeBeingDeleted { .. } => "WriteToAnInodeBeingDeleted",
        PublishError::WriteToAnInodeThatIsNotInTheTree { .. } => "WriteToAnInodeThatIsNotInTheTree",
        PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. } => {
            "UnlinkOfAnInodeThatIsNotALiveRecord"
        }
        PublishError::DeletedInodesQueueRefused(_) => "DeletedInodesQueueRefused",
        PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(read) => {
            return format!(
                "PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition({})",
                block_device_error_member(&read.error)
            )
        }
        PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => {
            "ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition"
        }
        PublishError::BlockDevice(cause) => {
            return format!(
                "PublishError::BlockDevice({})",
                block_device_error_member(cause)
            )
        }
    };
    format!("PublishError::{member}")
}

fn block_device_error_member(error: &BlockDeviceError) -> &'static str {
    match error {
        BlockDeviceError::OutOfRange { .. } => "BlockDeviceError::OutOfRange",
        BlockDeviceError::Unaligned { .. } => "BlockDeviceError::Unaligned",
        BlockDeviceError::InputOutput(_) => "BlockDeviceError::InputOutput",
        BlockDeviceError::ProbeFailed { .. } => "BlockDeviceError::ProbeFailed",
        BlockDeviceError::ImageFileAlreadyExists { .. } => {
            "BlockDeviceError::ImageFileAlreadyExists"
        }
        BlockDeviceError::ImageFileMissing { .. } => "BlockDeviceError::ImageFileMissing",
    }
}

fn recovery_failure_member(failure: &RecoveryFailure) -> String {
    match failure {
        RecoveryFailure::NoValidSystemConfiguration { .. } => {
            "RecoveryFailure::NoValidSystemConfiguration".to_string()
        }
        RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized { .. } => {
            "RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized".to_string()
        }
        RecoveryFailure::SystemConfigurationsDisagree => {
            "RecoveryFailure::SystemConfigurationsDisagree".to_string()
        }
        RecoveryFailure::NoDeviceInThePool => "RecoveryFailure::NoDeviceInThePool".to_string(),
        RecoveryFailure::RootRingSlotsPerRegionOutOfRange { out_of_range, .. } => {
            format!(
                "RecoveryFailure::RootRingSlotsPerRegionOutOfRange（S = {}）",
                out_of_range.declared_slots_per_region
            )
        }
        RecoveryFailure::SystemConfigurationValueRefused { value, .. } => {
            format!("RecoveryFailure::SystemConfigurationValueRefused（{value:?}）")
        }
        RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart { .. } => {
            "RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart".to_string()
        }
        RecoveryFailure::NoValidRoot => "RecoveryFailure::NoValidRoot".to_string(),
        RecoveryFailure::UnitUnreadable { .. } => "RecoveryFailure::UnitUnreadable".to_string(),
        RecoveryFailure::UnitMalformed { what } => {
            format!("RecoveryFailure::UnitMalformed（{what}）")
        }
        RecoveryFailure::InvariantViolated { invariant, .. } => {
            format!("RecoveryFailure::InvariantViolated（{invariant}）")
        }
        RecoveryFailure::EntryNarrowerThanItsFieldTable { what, .. } => {
            format!("RecoveryFailure::EntryNarrowerThanItsFieldTable（{what}）")
        }
        RecoveryFailure::AllocationRecordOutsideThePoolGeometry { what } => {
            format!("RecoveryFailure::AllocationRecordOutsideThePoolGeometry（{what}）")
        }
        RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree => {
            "RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree".to_string()
        }
        RecoveryFailure::MappingMiss { .. } => "RecoveryFailure::MappingMiss".to_string(),
        RecoveryFailure::MappingStillUnreadable { .. } => {
            "RecoveryFailure::MappingStillUnreadable".to_string()
        }
        RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            ..
        } => "RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided"
            .to_string(),
    }
}

fn mount_error_member(error: &MountError) -> String {
    let member = match error {
        MountError::Recovery(failure) => {
            return format!("MountError::Recovery({})", recovery_failure_member(failure))
        }
        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
            "NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread"
        }
        MountError::FileVersionWithoutAnyJournalRecord => "FileVersionWithoutAnyJournalRecord",
        MountError::InstanceTableMalformed => "InstanceTableMalformed",
        MountError::Acquisition(acquisition) => {
            return format!(
                "MountError::Acquisition({})",
                block_device_error_member(&acquisition.cause)
            )
        }
        MountError::Publish(failed) => {
            return format!(
                "MountError::Publish({})",
                publish_error_member(&failed.cause)
            )
        }
        MountError::RaiseFloorSequencePublishFailed(failed) => {
            return format!(
                "MountError::RaiseFloorSequencePublishFailed(publishes_persisted = {}, {})",
                failed.writes_of_persisted_publishes.len(),
                publish_error_member(&failed.cause)
            )
        }
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
            refused_publish_in_the_sequence,
            publishes_in_the_sequence,
            cause,
        } => {
            return format!(
                "MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite(publish {refused_publish_in_the_sequence} of {publishes_in_the_sequence}, {})",
                publish_error_member(cause)
            )
        }
        MountError::RollbackFloorAboveCeiling { .. } => "RollbackFloorAboveCeiling",
        MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
            "VersionWithoutFileNotWrittenByMakeFilesystem"
        }
        MountError::FormatTimeUnitLocationsOnDifferentSlots { .. } => {
            "FormatTimeUnitLocationsOnDifferentSlots"
        }
        MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { failure, .. } => {
            return format!(
                "MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable({})",
                recovery_failure_member(failure)
            )
        }
        MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot,
            first_reading,
            reread,
        } => {
            return format!(
                "MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread(区域 {} 槽 {}，{first_reading:?} 之后重读 {reread:?})",
                ring_slot.region, ring_slot.slot
            )
        }
        MountError::InstanceGenerationChangedBeforeAcquisition { .. } => {
            "InstanceGenerationChangedBeforeAcquisition"
        }
        MountError::SpaceAdmissionRefusedBeforeAcquisition { .. } => {
            "SpaceAdmissionRefusedBeforeAcquisition"
        }
        MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. } => {
            return format!(
                "MountError::RowPublishAdmissionRefusedBeforeAcquisition({})",
                publish_error_member(cause)
            )
        }
        MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            return format!(
                "MountError::WarmUpAdmissionRefusedBeforeAcquisition({})",
                publish_error_member(cause)
            )
        }
        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
            refusal, ..
        } => {
            return format!(
                "MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided({})",
                placement_refusal_member(refusal)
            )
        }
        MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. } => {
            "WritableMountRefusedByDevicesWithoutTheSelectedVersion"
        }
        MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. } => {
            "WritableDeviceCountBelowTheStripeWidthLowerBound"
        }
        MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed) => {
            return format!(
                "MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(devices_carrying_the_raised_floor = {}, {})",
                failed.devices_carrying_the_raised_floor.len(),
                block_device_error_member(&failed.cause)
            )
        }
        MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. } => {
            "RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided"
        }
        MountError::DeviceIdentitiesHandedInMoreThanOnce { .. } => {
            "DeviceIdentitiesHandedInMoreThanOnce"
        }
        MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. } => {
            "CallerParametersDisagreeWithTheSelectedSystemConfiguration"
        }
        MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. } => {
            "DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot"
        }
        MountError::SequenceNumberPastTheTopOfItsRange(_) => "SequenceNumberPastTheTopOfItsRange",
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            return format!(
                "MountError::FloorRaiseFailedAfterTheMountsPublishes({})",
                mount_error_member(&failed.cause)
            )
        }
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
            return format!(
                "MountError::NewerStateStillUnreadableAfterOneReread({})",
                still_unreadable_after_one_reread_member(still_unreadable)
            )
        }
        MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable) => {
            return format!(
                "MountError::SystemConfigurationUnreadableAfterTheReadStage({})",
                match unreadable.unreadable {
                    SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                        ..
                    } => "CheckingEachDeviceBeforeTheAcquisition",
                    SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
                        ..
                    } => "AcquisitionReadsAnUndecidableWitness",
                    SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
                        ..
                    } => "AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot",
                }
            )
        }
    };
    format!("MountError::{member}")
}

fn still_unreadable_after_one_reread_member(
    still_unreadable: &StillUnreadableAfterOneReread,
) -> &'static str {
    match still_unreadable {
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
            "PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion"
        }
        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. } => {
            "InstanceTableOfTheNewestRootForTheShadowLedger"
        }
        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. } => {
            "InstanceTableOfTheNewestRootForTheEffectiveFloor"
        }
        StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. } => {
            "RootRingSlotKnownToHoldARoot"
        }
        StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => "AccountOfAnAbandonedRoot",
        StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
            ..
        } => "AccountOfAnUnappliedPublishWithIncompleteReadableRecords",
        StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
            "PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify"
        }
    }
}

fn rollback_error_member(error: &RollbackError) -> String {
    let member = match error {
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. } => {
            "PublishFrozenAfterAWriteFailureIsNotResentYet".to_string()
        }
        RollbackError::Recovery(failure) => {
            format!("Recovery({})", recovery_failure_member(failure))
        }
        RollbackError::CurrentInstanceTableMalformed => "CurrentInstanceTableMalformed".to_string(),
        RollbackError::TargetNotACandidate { exclusion, .. } => {
            format!("TargetNotACandidate({exclusion:?})")
        }
        RollbackError::TargetVersionUnreadable { failure, .. } => {
            format!("TargetVersionUnreadable({})", recovery_failure_member(failure))
        }
        RollbackError::CurrentAccountUnreadable { failure } => {
            format!("CurrentAccountUnreadable({})", recovery_failure_member(failure))
        }
        RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        } => "TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided".to_string(),
        RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. } => {
            "UserVisibleUnitWithoutItsRecordInTheCurrentAccount".to_string()
        }
        RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. } => {
            "UserVisibleUnitStillAllocatedUnderAnotherGeneration".to_string()
        }
        RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. } => {
            "ResurrectedUnitCopyUnreadableOrMismatched".to_string()
        }
        RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. } => {
            "IsolatedUnitCopyUnreadableOrMismatched".to_string()
        }
        RollbackError::CurrentVersionUnitNotReleasable { cause } => {
            format!("CurrentVersionUnitNotReleasable({})", publish_error_member(cause))
        }
        RollbackError::Publish(failed) => format!("Publish({})", publish_error_member(&failed.cause)),
        RollbackError::CallerInputsDisagreeWithTheDisk(_) => {
            "CallerInputsDisagreeWithTheDisk".to_string()
        }
        RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. } => {
            "NextCheckpointTxgPastTheTopOfItsRange".to_string()
        }
        RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_unreadable) => {
            format!(
                "CandidateJudgementStillUnreadableAfterOneReread({})",
                still_unreadable_after_one_reread_member(still_unreadable)
            )
        }
    };
    format!("RollbackError::{member}")
}

fn recovery_outcome_member(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
        RecoveryOutcome::FileRead { .. } => "FileRead".to_string(),
        RecoveryOutcome::Failed { failure, .. } => {
            format!("Failed({})", recovery_failure_member(failure))
        }
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

/// 一次发布前后的分配记录：改写或新增的记录的代要在这次（几次）发布的 txg 里。
fn allocation_generation_judgement(
    before: &[AllocationRecord],
    after: &[AllocationRecord],
    first_publish_txg: CheckpointTxg,
    last_publish_txg: CheckpointTxg,
) -> Option<HarnessJudgement> {
    let records = records_changed_with_a_generation_outside_the_publish_txgs(
        before,
        after,
        first_publish_txg,
        last_publish_txg,
    );
    (!records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
        records,
        first_publish_txg,
        last_publish_txg,
    })
}

fn apply_publish_first_file(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let content = content_choice.bytes();
    let root_to_carry_instance_table_from = *session.current.root();
    let previous_record_bytes = session.current.record_bytes().to_vec();
    let records_before = session.allocator.records().to_vec();
    let answer = model.answer_publish_first_file(&content);
    let stream_length_before = stream.operation_count();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_first_file(
        &mut writer,
        &mut session.allocator,
        &root_to_carry_instance_table_from,
        FirstFile {
            content: &content,
            write_time_seconds,
        },
        session.instance,
        &previous_record_bytes,
    ) {
        Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
        Err(error) => {
            settle_refused_file_publish(model, answer, &error, stream, stream_length_before)
        }
    }
}

/// 第一个文件、覆盖写被拒：成员映射成理由，交模型比。
fn settle_refused_file_publish(
    model: &mut IdealModel,
    answer: Result<ModelAnswer, ModelDisagreement>,
    error: &PublishError,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let member = publish_error_member(error);
    let reason = refusal_reason_of_publish_error(error);
    let verdict = judge_by_model(
        model,
        answer,
        &observed_refusal(
            member.clone(),
            reason,
            0,
            stream_length_before,
            stream,
            None,
            None,
        ),
    );
    AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
}

/// 第一个文件、覆盖写做成之后：数复用、判分配代（第 1 件）、拿输出与模型比（第 2 件）、把现行版本换成这一版。
fn settle_file_publish(
    session: &mut WritableSession,
    model: &mut IdealModel,
    answer: Result<ModelAnswer, ModelDisagreement>,
    records_before: &[AllocationRecord],
    output: singlefs_core::transaction::TransactionOutput,
) -> AppliedStep {
    let reuse = RecordReuse::between(records_before, session.allocator.records());
    let publish_txg = output.root.checkpoint_txg;
    let harness_judgement = allocation_generation_judgement(
        records_before,
        session.allocator.records(),
        publish_txg,
        publish_txg,
    );
    let verdict = judge_by_model(
        model,
        answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: vec![observed_root_of_file_version(&output)],
            reported_ceiling: None,
        }),
    );
    session.current = PoolVersion::WithFile(output);
    session.carried_to_zero_unit_publishes = None;
    session.publishes_in_this_mount += 1;
    AppliedStep {
        outcome: StepOutcome::Applied(AppliedEffect::Published {
            reuse,
            floor_raises_pushed_by_the_session: 0,
        }),
        harness_judgement,
        model_verdict: Some(verdict),
    }
}

fn apply_publish_overwrite(
    pool: &mut HistoryPool,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let Some(current_file_version) = open_session.current.file_version() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    if current_file_version.inode_record.is_none() {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
    }
    // 现行那一版有删除中的文件（unlink 的一批被拒之后、回退到删除中的一版之后）：照调，会话先走空、把它走成有树而文件集合为空的一版、
    // 再拒成 `UserChangeRefused::NoFileVersionToChange`，那个成员带着走空的那几步与推的抬 F（代码轮 U10），逐版交模型比（代码轮 U11）。
    let queue_is_walked_first = !current_file_version.deleted_inodes.is_empty();
    let content = content_choice.bytes();
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    // 覆盖写是用户的一次改动，经挂着的会话发布（`MountedSession::publish_user_change`，实七）：发布路径报空间不够时会话先推一串抬 F、
    // 再重判（D16（发布语义） 已定项 1「准入」那一行，C283）。会话借走这个进程的分配器、现行版本与认下的池，发完原样还回来；
    // 会话只按它认下的那份参数建写入口，执行器手里的参数不交进去（实审 A1b Q2）。
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
    };
    let file = FirstFile {
        content: &content,
        write_time_seconds,
    };
    // 内容跨几个数据单元的走顺序写入口（代码轮 U13），别的走覆盖写入口；模型那一侧各按各的契约答。
    let entry = content_choice.length.whole_file_write_entry();
    let change_result = mounted_session.publish_user_change(
        devices,
        match entry {
            WholeFileWriteEntry::Overwrite => UserChange::Overwrite(file),
            WholeFileWriteEntry::SequentialWrite => UserChange::SequentialWrite(file),
        },
    );
    let answer_of_the_change = |ideal_model: &IdealModel| match entry {
        WholeFileWriteEntry::Overwrite => ideal_model.answer_publish_overwrite(&content),
        WholeFileWriteEntry::SequentialWrite => {
            ideal_model.answer_publish_sequential_write(&content)
        }
    };
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes,
    });
    if queue_is_walked_first {
        return settle_user_change_comparing_every_version_published(
            session_after_the_change,
            model,
            &answer_of_the_change,
            &records_before,
            &change_result,
            stream,
            stream_length_before,
        );
    }
    settle_user_change(
        session_after_the_change,
        model,
        &answer_of_the_change,
        &records_before,
        change_result,
        stream,
        stream_length_before,
    )
}

/// 会话推的那几串抬 F（D16（发布语义） 已定项 1「准入」那一行）：逐串照模型的抬 F 比（新 F 就是那一串的根带的、上限是实现报的），
/// 与挂载那一处（`settle_mount`）同一个比法；前面一处对不上就不再往下比。交回合起来的判定与这几串一共写出几条根。
fn judge_floor_raises_pushed_by_the_session(
    model: &mut IdealModel,
    floor_raises: &[singlefs_core::mount::RaisedFloor],
) -> (ModelVerdict, usize) {
    let mut verdict = ModelVerdict::default();
    let mut publishes = 0;
    for raised in floor_raises {
        if verdict.disagreement.is_some() {
            break;
        }
        let raise_verdict = judge_one_floor_raise_pushed_by_the_session(model, raised);
        verdict.counts.add(&raise_verdict.counts);
        verdict.disagreement = raise_verdict.disagreement;
        publishes += raised.publishes.len();
    }
    (verdict, publishes)
}

/// 会话推的一串抬 F 照模型的抬 F 比：新 F 就是那一串的根带的、上限是实现报的。
fn judge_one_floor_raise_pushed_by_the_session(
    model: &mut IdealModel,
    raised: &singlefs_core::mount::RaisedFloor,
) -> ModelVerdict {
    let raised_floor = raised
        .publishes
        .first()
        .expect("抬 F 做成至少推一次空发布：推到每块盘上都有一条带新 F 的根")
        .root
        .rollback_floor;
    let raise_answer = model.answer_raise_rollback_floor(ModelCheckpointTxg(raised_floor.0));
    judge_by_model(
        model,
        raise_answer,
        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
            roots: raised
                .publishes
                .iter()
                .map(observed_root_of_file_version)
                .collect(),
            reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),
        }),
    )
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

/// 经会话的覆盖写返回之后：先把会话推的那几串抬 F 交模型逐串比，再比这次覆盖写——做成的比根与分配代、数复用；
/// 被拒的比理由，「拒之前写没写盘」只看那几串抬 F 之外多出来的写（推了抬 F 的时候那几串的写不算这次拒绝写的）。
/// 分配代逐次比：盘上的上一版（会话开始时分配器里的记录）→ 每一次空发布的记录 → 覆盖写之后分配器里的记录，各按那一次的 txg 判。
/// 会话推抬 F 半路报错（`UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`）时，那一串已落盘的那几次空发布不在 `floor_raises` 里：模型那一侧
/// 比不到它们，接下来的历史里模型与实现的现行版本对不上，由下一步的比较报出来（与挂载那一处推满途中报错同一个现状）。
#[allow(
    clippy::too_many_arguments,
    reason = "会话、模型、内容、改动之前的记录、会话的结局、录制流与它的起点各是一样，收成结构体只会多一层没人验的名字"
)]
fn settle_user_change(
    session: &mut WritableSession,
    model: &mut IdealModel,
    answer_of_the_change: &dyn Fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
    records_before: &[AllocationRecord],
    change_result: Result<UserChangePublished, UserChangeRefused>,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let floor_raises: &[singlefs_core::mount::RaisedFloor] = match &change_result {
        Ok(published) => &published.floor_raises,
        Err(
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
            | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
            | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. },
        ) => &[],
        Err(
            UserChangeRefused::Publish { floor_raises, .. }
            | UserChangeRefused::NoFileVersionToChange { floor_raises, .. },
        ) => floor_raises,
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => &refused.floor_raises,
        Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused)) => {
            &refused.floor_raises
        }
    };
    let (mut verdict, publishes_of_the_floor_raises) =
        judge_floor_raises_pushed_by_the_session(model, floor_raises);
    session.publishes_in_this_mount += publishes_of_the_floor_raises;
    // 分配代：每一次空发布按它自己的 txg，上一版是前一次空发布（第一次是会话开始时分配器里的记录）。
    let mut records_of_the_previous_publish: &[AllocationRecord] = records_before;
    let mut harness_judgement = None;
    for publish in floor_raises
        .iter()
        .flat_map(|raised| raised.publishes.iter())
    {
        let publish_txg = publish.root.checkpoint_txg;
        harness_judgement = harness_judgement.or_else(|| {
            allocation_generation_judgement(
                records_of_the_previous_publish,
                &publish.allocation_records,
                publish_txg,
                publish_txg,
            )
        });
        records_of_the_previous_publish = &publish.allocation_records;
    }
    let refusal = match &change_result {
        Ok(_) => {
            let answer = answer_of_the_change(model);
            let output = session.current.file_version().expect(
                "会话发成一次覆盖写之后现行那一版带文件：`publish_user_change` 只接在带文件的一版后面、把新的一版放进 `current`",
            );
            let publish_txg = output.root.checkpoint_txg;
            harness_judgement = harness_judgement.or_else(|| {
                allocation_generation_judgement(
                    records_of_the_previous_publish,
                    session.allocator.records(),
                    publish_txg,
                    publish_txg,
                )
            });
            if verdict.disagreement.is_none() {
                let overwrite_verdict = judge_by_model(
                    model,
                    answer,
                    &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: vec![observed_root_of_file_version(output)],
                        reported_ceiling: None,
                    }),
                );
                verdict.counts.add(&overwrite_verdict.counts);
                verdict.disagreement = overwrite_verdict.disagreement;
            }
            session.publishes_in_this_mount += 1;
            return AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Published {
                    reuse: RecordReuse::between(records_before, session.allocator.records()),
                    floor_raises_pushed_by_the_session: floor_raises.len(),
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            };
        }
        Err(refused) => refused,
    };
    let (member, reason) = member_and_reason_of_a_refused_user_change(refusal);
    if verdict.disagreement.is_none() {
        let answer = answer_of_the_change(model);
        let wrote_anything_besides_the_floor_raises = if floor_raises.is_empty() {
            stream.operation_count() != stream_length_before
        } else {
            writes_in_the_stream_since(stream, stream_length_before)
                > writes_of_the_floor_raises(floor_raises)
        };
        let refusal_verdict = judge_by_model(
            model,
            answer,
            &ObservedOutcome::Refused {
                member: member.clone(),
                reason,
                publishes_completed: 0,
                wrote_anything: wrote_anything_besides_the_floor_raises,
                reported_ceiling: None,
                reported_root_ring_slot_still_bad_after_one_reread: None,
            },
        );
        verdict.counts.add(&refusal_verdict.counts);
        verdict.disagreement = refusal_verdict.disagreement;
    }
    AppliedStep {
        outcome: StepOutcome::Refused { member },
        harness_judgement,
        model_verdict: Some(verdict),
    }
}

/// 一次用户改动被拒：成员的名字与它说的是哪条理由。
fn member_and_reason_of_a_refused_user_change(
    refusal: &UserChangeRefused,
) -> (String, ObservedRefusalReason) {
    match refusal {
        // 执行器每次都交整池两块盘、与会话认下的那一份同次序：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. } => (
            "UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        // 有树而文件集合为空的一版上不是建 inode 的改动（会话先走空、走成这一形之后拒）：D8（核心索引结构） 已定项 16 第 5 条那一条理由。
        UserChangeRefused::NoFileVersionToChange { .. } => (
            "UserChangeRefused::NoFileVersionToChange".to_string(),
            ObservedRefusalReason::Explained(
                ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile,
            ),
        ),
        // 执行器交的是整池那两块盘、没换过盘（Z3-A 乙那一判）：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => (
            "UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        // 执行器交的盘体各在自己的身份上、没换过旧快照（代码三方 m2-closeout-code-r2 那两判）：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. } => (
            "UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. } => (
            "UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot"
                .to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        UserChangeRefused::Publish { cause, .. } => (
            publish_error_member(cause),
            refusal_reason_of_publish_error(cause),
        ),
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => (
            format!(
                "UserChangeRefused::NoSpaceAfterRaisingTheFloor({})",
                publish_error_member(&refused.last_refusal)
            ),
            refusal_reason_of_publish_error(&refused.last_refusal),
        ),
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused) => (
            format!(
                "UserChangeRefused::FloorRaiseFailedWhilePushingForSpace({})",
                mount_error_member(&refused.cause)
            ),
            refusal_reason_of_mount_error(&refused.cause),
        ),
    }
}

/// 一次用户改动交回的推的抬 F 与推的抬 F 之外发成的那几版（被拒的几种里带着的，见 `UserChangeRefused` 各成员）。
fn floor_raises_and_versions_published_of_a_user_change(
    change_result: &Result<UserChangePublished, UserChangeRefused>,
) -> (
    &[singlefs_core::mount::RaisedFloor],
    &[singlefs_core::transaction::TransactionOutput],
) {
    match change_result {
        Ok(published) => (&published.floor_raises, &published.versions_published),
        Err(UserChangeRefused::NoFileVersionToChange {
            floor_raises,
            versions_published,
            ..
        }) => (floor_raises, versions_published),
        Err(
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
            | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
            | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. },
        ) => (&[], &[]),
        Err(UserChangeRefused::Publish {
            floor_raises,
            versions_published,
            ..
        }) => (floor_raises, versions_published),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => {
            (&refused.floor_raises, &refused.versions_published)
        }
        Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused)) => {
            (&refused.floor_raises, &refused.versions_published)
        }
    }
}

/// 一次 unlink（或挂载续做）里落了盘的一样：推的一串抬 F，或推的抬 F 之外发成的一版（意图那次、走空的一步、建 inode 那次）。
enum FloorRaiseOrVersion<'change> {
    FloorRaise(&'change singlefs_core::mount::RaisedFloor),
    Version(&'change singlefs_core::transaction::TransactionOutput),
}

/// 推的那几串抬 F 与发成的那几版按 txg 排好（一串抬 F 按它第一次空发布的 txg）：两样在会话与挂载续做里按落盘的先后交错。
fn floor_raises_and_versions_in_txg_order<'change>(
    floor_raises: &'change [singlefs_core::mount::RaisedFloor],
    versions_published: &'change [singlefs_core::transaction::TransactionOutput],
) -> Vec<FloorRaiseOrVersion<'change>> {
    let mut published_in_txg_order: Vec<(CheckpointTxg, FloorRaiseOrVersion<'change>)> =
        floor_raises
            .iter()
            .map(|raised| {
                let first_publish_txg = raised
                    .publishes
                    .first()
                    .expect("抬 F 做成至少推一次空发布")
                    .root
                    .checkpoint_txg;
                (first_publish_txg, FloorRaiseOrVersion::FloorRaise(raised))
            })
            .chain(versions_published.iter().map(|version| {
                (
                    version.root.checkpoint_txg,
                    FloorRaiseOrVersion::Version(version),
                )
            }))
            .collect();
    published_in_txg_order.sort_by_key(|(txg, _)| *txg);
    published_in_txg_order
        .into_iter()
        .map(|(_, published)| published)
        .collect()
}

/// unlink 第一个文件（`HistoryOperation::Unlink`）：经挂着的会话发（意图那次 + 走空，D8（核心索引结构） 已定项 16）。
/// 意图那次带这一步的写入时间（与覆盖写、建 inode 同一个取法，用户 2026-10-03 定 Q-F 乙），各批照抄。
/// 前提：现行那一版带文件、第一个文件那条记录在。删除中的那一版照调：会话先走空、把它走成有树而文件集合为空的一版、再拒成
/// `UserChangeRefused::NoFileVersionToChange`，走空的那几步逐版交模型比（代码轮 U11）。
fn apply_unlink(pool: &mut HistoryPool, write_time_seconds: u64) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let Some(current_file_version) = open_session.current.file_version() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    if current_file_version.inode_record.is_none() {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
    }
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
    };
    let change_result = mounted_session.publish_user_change(
        devices,
        UserChange::Unlink {
            inode: singlefs_core::address::InodeNumber(
                singlefs_core::transaction::FIRST_INODE_NUMBER,
            ),
            write_time_seconds,
        },
    );
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，unlink 之后这一版有树。
        carried_to_zero_unit_publishes: None,
    });
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &IdealModel::answer_unlink_intent,
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

/// 经会话的 unlink（或建 inode）返回之后：会话推的每一串抬 F 与推的抬 F 之外发成的每一版按 txg 排好，逐样交模型比——抬 F 照模型的抬 F，
/// 发成的一版在模型的现行版本有删除中的文件时照「走空的一步」、否则照这次改动本身（`answer_of_the_change`：意图那次、建 inode 那次）；
/// 分配代逐次比（上一版是前一样落盘的）。
/// 被拒的比理由：被拒的是模型此刻的下一次发布（有删除中的文件就是走空的一步，否则是这次改动本身），「拒之前写没写盘」只看那几串抬 F
/// 与发成的那几版之外多出来的写。
fn settle_user_change_comparing_every_version_published(
    session: &mut WritableSession,
    model: &mut IdealModel,
    answer_of_the_change: &dyn Fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
    records_before: &[AllocationRecord],
    change_result: &Result<UserChangePublished, UserChangeRefused>,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let (floor_raises, versions_published) =
        floor_raises_and_versions_published_of_a_user_change(change_result);
    let published_in_txg_order =
        floor_raises_and_versions_in_txg_order(floor_raises, versions_published);
    let mut verdict = ModelVerdict::default();
    let mut harness_judgement = None;
    let mut records_of_the_previous_publish: &[AllocationRecord] = records_before;
    // 这次改动自己（意图那次、建 inode 那次、覆盖写）发成之前，会话先走空的那几步与走空时为空间推的抬 F（代码轮第二轮判决第二节 N3）。
    let mut the_change_itself_was_published = false;
    let mut steps_walked_before_the_change = 0_u64;
    let mut floor_raises_while_walking_before_the_change = 0_u64;
    // 迭代上界是这次落盘的样数；跨轮带的是上一样落盘之后的分配记录与合起来的判定；模型对不上之后不再问模型（分配代照判）。
    for published in &published_in_txg_order {
        let outputs: Vec<&singlefs_core::transaction::TransactionOutput> = match published {
            FloorRaiseOrVersion::FloorRaise(raised) => {
                if !the_change_itself_was_published
                    && model.current_version_has_a_file_being_deleted()
                {
                    floor_raises_while_walking_before_the_change += 1;
                }
                if verdict.disagreement.is_none() {
                    let raise_verdict = judge_one_floor_raise_pushed_by_the_session(model, raised);
                    verdict.counts.add(&raise_verdict.counts);
                    verdict.disagreement = raise_verdict.disagreement;
                }
                raised.publishes.iter().collect()
            }
            FloorRaiseOrVersion::Version(version) => {
                let walking = model.current_version_has_a_file_being_deleted();
                if walking && !the_change_itself_was_published {
                    steps_walked_before_the_change += 1;
                }
                if !walking {
                    the_change_itself_was_published = true;
                }
                if verdict.disagreement.is_none() {
                    let answer = if walking {
                        model.answer_deletion_step()
                    } else {
                        answer_of_the_change(model)
                    };
                    let version_verdict = judge_by_model(
                        model,
                        answer,
                        &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                            roots: vec![observed_root_of_file_version(version)],
                            reported_ceiling: None,
                        }),
                    );
                    verdict.counts.add(&version_verdict.counts);
                    verdict.disagreement = version_verdict.disagreement;
                }
                vec![*version]
            }
        };
        for output in outputs {
            let publish_txg = output.root.checkpoint_txg;
            harness_judgement = harness_judgement.or_else(|| {
                allocation_generation_judgement(
                    records_of_the_previous_publish,
                    &output.allocation_records,
                    publish_txg,
                    publish_txg,
                )
            });
            records_of_the_previous_publish = &output.allocation_records;
            session.publishes_in_this_mount += 1;
        }
    }
    if verdict.disagreement.is_none() {
        if steps_walked_before_the_change >= 2 {
            verdict
                .counts
                .walks_of_several_steps_before_a_user_change_matched += 1;
        }
        if floor_raises_while_walking_before_the_change > 0 {
            verdict
                .counts
                .walks_before_a_user_change_refused_for_space_matched += 1;
        }
    }
    let reuse = RecordReuse::between(records_before, session.allocator.records());
    let refusal = match change_result {
        Ok(_) => {
            return AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Published {
                    reuse,
                    floor_raises_pushed_by_the_session: floor_raises.len(),
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            };
        }
        Err(refused) => refused,
    };
    let (member, reason) = member_and_reason_of_a_refused_user_change(refusal);
    if verdict.disagreement.is_none() {
        let answer = if model.current_version_has_a_file_being_deleted() {
            model.answer_deletion_step()
        } else {
            answer_of_the_change(model)
        };
        let writes_of_the_versions_published: u64 = versions_published
            .iter()
            .map(|version| version.writes.total().write_calls)
            .sum();
        let wrote_anything_besides_what_was_published =
            if floor_raises.is_empty() && versions_published.is_empty() {
                stream.operation_count() != stream_length_before
            } else {
                writes_in_the_stream_since(stream, stream_length_before)
                    > writes_of_the_floor_raises(floor_raises) + writes_of_the_versions_published
            };
        let refusal_verdict = judge_by_model(
            model,
            answer,
            &ObservedOutcome::Refused {
                member: member.clone(),
                reason,
                publishes_completed: 0,
                wrote_anything: wrote_anything_besides_what_was_published,
                reported_ceiling: None,
                reported_root_ring_slot_still_bad_after_one_reread: None,
            },
        );
        verdict.counts.add(&refusal_verdict.counts);
        verdict.disagreement = refusal_verdict.disagreement;
    }
    AppliedStep {
        outcome: StepOutcome::Refused { member },
        harness_judgement,
        model_verdict: Some(verdict),
    }
}

/// 建一个 size 0 的 inode（`HistoryOperation::CreateInode`）：经挂着的会话发（`UserChange::NewInodes`，count 1）。
/// 前提：第一个文件不再活着——现行那一版有树而文件集合为空，或是在它上面建过 inode 的一版（带文件、没有第一个文件那条记录、队列空），
/// 或第一个文件删除中的那一版（会话先走空，代码轮 U11）。
fn apply_create_inode(pool: &mut HistoryPool, write_time_seconds: u64) -> AppliedStep {
    let HistoryPool {
        devices,
        session: session_slot,
        model,
        stream,
        ..
    } = pool;
    let Some(open_session) = session_slot.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    match &open_session.current {
        PoolVersion::WithoutFile(_) => {
            return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
        }
        PoolVersion::WithFile(version) => {
            // 删除中的那一版（队列不空）照调：会话先走空、把它走成删完的一版再建（代码轮 U11）。
            if version.inode_record.is_some() && version.deleted_inodes.is_empty() {
                return AppliedStep::not_applicable(
                    MissingPrecondition::CurrentVersionStillHasTheFirstFile,
                );
            }
        }
        PoolVersion::WithTreesAndNoFile(_) => {}
    }
    let records_before = open_session.allocator.records().to_vec();
    let stream_length_before = stream.operation_count();
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        publishes_in_this_mount,
        carried_to_zero_unit_publishes: _,
    } = session_slot.take().expect("上面判过这个进程的会话开着");
    let mut mounted_session = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
    };
    let change_result = mounted_session.publish_user_change(
        devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds,
        },
    );
    let session_after_the_change = session_slot.insert(WritableSession {
        allocator: mounted_session.allocator,
        current: mounted_session.current,
        instance: mounted_session.instance,
        shadow_ledger: mounted_session.shadow_ledger,
        parameters_and_device_table: mounted_session.parameters_and_device_table,
        publishes_in_this_mount,
        // 零单元发布只接在树表 0 条的一版后面，建 inode 之后这一版有树。
        carried_to_zero_unit_publishes: None,
    });
    settle_user_change_comparing_every_version_published(
        session_after_the_change,
        model,
        &IdealModel::answer_create_inode,
        &records_before,
        &change_result,
        stream,
        stream_length_before,
    )
}

fn apply_publish_without_units(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    // 前提：只在树表 0 条的一版上调（出处见 `MissingPrecondition::CurrentVersionWithFile` 的注释）。
    let PoolVersion::WithoutFile(previous) = &session.current else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithFile);
    };
    let answer = model.answer_publish_without_units();
    let stream_length_before = stream.operation_count();
    let plan = ZeroUnitPublishPlan {
        txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
        counter: previous.record.counter + 1,
        instance: session.instance,
        back_chain: back_chain_of(&previous.record_bytes),
        rollback_floor: previous.root.rollback_floor,
        // 会话里接着现行那一版：它的水位已经是根环里的 max（挂载时本实例第一次发布取过，之后只照抄或推高）。
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
    };
    let previous_root = previous.root;
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_without_units(&mut writer, &previous_root, plan) {
        Ok(output) => {
            // 零单元发布从这次挂载里前一版交回的那一份往下带着比（实审 B3c-4）；这次挂载里没有可带的照旧计数。
            let (observed, carried) = observed_root_of_version_without_file_carrying(
                &output,
                session.carried_to_zero_unit_publishes.as_ref(),
            );
            let verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed],
                    reported_ceiling: None,
                }),
            );
            session.carried_to_zero_unit_publishes = carried;
            session.current = PoolVersion::WithoutFile(output);
            session.publishes_in_this_mount += 1;
            AppliedStep::judged_by_outcome_and_model(
                StepOutcome::Applied(AppliedEffect::Published {
                    reuse: RecordReuse::default(),
                    floor_raises_pushed_by_the_session: 0,
                }),
                verdict,
            )
        }
        Err(error) => {
            let member = format!("publish_without_units({})", publish_error_member(&error));
            let verdict = judge_by_model(
                model,
                answer,
                &observed_refusal(
                    member.clone(),
                    refusal_reason_of_publish_error(&error),
                    0,
                    stream_length_before,
                    stream,
                    None,
                    None,
                ),
            );
            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
        }
    }
}

/// 挂载写出的写行与暖机那几次发布，分配记录比过没有（代码三方第二轮判决第二节第 1 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountAllocationComparison {
    /// 有树的那几次发布（带文件的、有树而文件集合为空的）逐次比过：第一次比挂载之前的镜像上生效根那棵分配记录树，之后比前一次的输出。
    Compared { publishes: usize },
    /// 写行与暖机都是零单元发布（生效根那一版树表 0 条），不写分配记录，没东西可比。
    NoPublishWithFile,
    /// 挂载之前的镜像上生效根那棵分配记录树读不出（例如它是恢复施加 journal 重建出来的一版、树表单元没落盘）：没比。
    PreviousRecordsUnreadable,
}

/// 挂载写出的写行与暖机那几次（有树的）发布、推的抬 F 与续做的每一步：逐次拿上一版的分配记录比，改写或新增的记录代要等于那一次的 txg；顺带数复用。
/// 上一版：第一次是挂载之前的镜像上生效根（`MountOutput::effective_root`，可写挂载是施加前缀之后的根、回退是回退的目标根）
/// 那棵分配记录树（`recovery::allocation_records_under_root`，读盘），之后是前一次发布的输出。照代码三方第二轮攻方的改法写，
/// 被攻过零轮。
fn mount_publish_allocation_judgement(
    image_before_mount: &MemoryPool,
    mounted: &singlefs_core::mount::Mounted,
) -> (
    MountAllocationComparison,
    RecordReuse,
    Option<HarnessJudgement>,
) {
    let Ok(mut previous_records) =
        allocation_records_under_root(image_before_mount, &mounted.output.effective_root)
    else {
        return (
            MountAllocationComparison::PreviousRecordsUnreadable,
            RecordReuse::default(),
            None,
        );
    };
    let mut publishes_compared = 0;
    let mut reuse = RecordReuse::default();
    let mut harness_judgement = None;
    // 写行、暖机，与可写挂载准入不够时写行之后推的那几串抬 F 的空发布（D16（发布语义） 已定项 1「准入」那一行），按落盘的先后。
    // 写行与暖机按「有树」挑、不按「带文件」挑：有树而文件集合为空的一版（D8（核心索引结构） 已定项 16 第 5 条）照样写分配记录，
    // 跳过它们，比较的上一版就停在挂载之前那棵树上，下一次真比的发布把这几次改写的记录全当成自己改的、代却不是自己的 txg。
    let versions_of_trees_in_publish_order = std::iter::once(&mounted.output.row_publish)
        .chain(mounted.output.warm_up_publishes.iter())
        .filter_map(PoolVersion::version_of_trees)
        .chain(
            mounted
                .output
                .space_admission
                .floor_raises()
                .iter()
                .flat_map(|raised| raised.publishes.iter()),
        )
        .chain(
            floor_raises_and_versions_in_txg_order(
                &mounted.output.floor_raises_pushed_by_the_continuation,
                &mounted.output.deletion_queue_continuation,
            )
            .into_iter()
            .flat_map(|published| match published {
                FloorRaiseOrVersion::FloorRaise(raised) => raised.publishes.iter().collect(),
                FloorRaiseOrVersion::Version(version) => vec![version],
            }),
        );
    for output in versions_of_trees_in_publish_order {
        let publish_txg = output.root.checkpoint_txg;
        harness_judgement = harness_judgement.or_else(|| {
            allocation_generation_judgement(
                &previous_records,
                &output.allocation_records,
                publish_txg,
                publish_txg,
            )
        });
        reuse = reuse.plus(RecordReuse::between(
            &previous_records,
            &output.allocation_records,
        ));
        previous_records.clone_from(&output.allocation_records);
        publishes_compared += 1;
    }
    let comparison = if publishes_compared == 0 {
        MountAllocationComparison::NoPublishWithFile
    } else {
        MountAllocationComparison::Compared {
            publishes: publishes_compared,
        }
    };
    (comparison, reuse, harness_judgement)
}

/// 挂载（可写挂载或回退）入口返回之后：做成的判分配代（第 1 件）、与模型比（第 2 件）、开会话；被拒的与模型比理由、写没写盘。
fn settle_mount(
    pool: &mut HistoryPool,
    mounted: Result<singlefs_core::mount::Mounted, MountError>,
    image_before_mount: &MemoryPool,
    answer: ModelAnswer,
    stream_length_before: usize,
) -> AppliedStep {
    match mounted {
        Ok(mounted) => {
            let (allocation_records_compared, reuse, harness_judgement) =
                mount_publish_allocation_judgement(image_before_mount, &mounted);
            // 暖机的零单元发布从这次挂载里写行那一版交回的那一份往下带着比（实审 B3c-4），带到最后的那一份交给会话。
            let (observed, carried_to_zero_unit_publishes) =
                observed_mount_carrying_to_zero_unit_publishes(&mounted);
            let mut verdict = judge_by_model(
                &mut pool.model,
                Ok(answer),
                &ObservedOutcome::Succeeded(observed),
            );
            // 可写挂载准入不够时写行之后推的那几串抬 F（D16（发布语义） 已定项 1「准入」那一行）：每一串照模型的抬 F 逐串比
            // （新 F 就是那一串的根带的、上限是实现报的），接在挂载那几条根后面；前面一处对不上就不再往下比。
            let floor_raises = mounted.output.space_admission.floor_raises();
            for raised in floor_raises {
                if verdict.disagreement.is_some() {
                    break;
                }
                let raised_floor = raised
                    .publishes
                    .first()
                    .expect("抬 F 做成至少推一次空发布：推到每块盘上都有一条带新 F 的根")
                    .root
                    .rollback_floor;
                let raise_answer = pool
                    .model
                    .answer_raise_rollback_floor(ModelCheckpointTxg(raised_floor.0));
                let raise_verdict = judge_by_model(
                    &mut pool.model,
                    raise_answer,
                    &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: raised
                            .publishes
                            .iter()
                            .map(observed_root_of_file_version)
                            .collect(),
                        reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),
                    }),
                );
                verdict.counts.add(&raise_verdict.counts);
                verdict.disagreement = raise_verdict.disagreement;
            }
            // 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）：接在推的抬 F 之后，每一步照模型的「走空的一步」比；
            // 走空里为空间推的那几串抬 F（用户 2026-10-03 定案「挂载续做被空间拒：同样推抬 F 再试」）按 txg 插在各步之间，照模型的抬 F 逐串比。
            // 推满仍放不下（`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`，用户 2026-10-03 定「照 D16 做成可写」）时挂载照样做成、
            // 队列留着：模型的现行那一版停在删除中，那一次被拒交给这次挂载之后会话里下一次用户改动的走空重判（会话先走空，`settle_user_change_comparing_every_version_published`）。
            for published in floor_raises_and_versions_in_txg_order(
                &mounted.output.floor_raises_pushed_by_the_continuation,
                &mounted.output.deletion_queue_continuation,
            ) {
                if verdict.disagreement.is_some() {
                    break;
                }
                let published_verdict = match published {
                    FloorRaiseOrVersion::FloorRaise(raised) => {
                        judge_one_floor_raise_pushed_by_the_session(&mut pool.model, raised)
                    }
                    FloorRaiseOrVersion::Version(continuation_step) => {
                        let step_answer = pool.model.answer_deletion_step();
                        judge_by_model(
                            &mut pool.model,
                            step_answer,
                            &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                                roots: vec![observed_root_of_file_version(continuation_step)],
                                reported_ceiling: None,
                            }),
                        )
                    }
                };
                verdict.counts.add(&published_verdict.counts);
                verdict.disagreement = published_verdict.disagreement;
            }
            let publishes = 1
                + mounted.output.warm_up_publishes.len()
                + floor_raises
                    .iter()
                    .map(|raised| raised.publishes.len())
                    .sum::<usize>()
                + mounted.output.deletion_queue_continuation.len()
                + mounted
                    .output
                    .floor_raises_pushed_by_the_continuation
                    .iter()
                    .map(|raised| raised.publishes.len())
                    .sum::<usize>();
            let space_admission = MountSpaceAdmissionOutcome::of(&mounted.output.space_admission);
            let instance = mounted.output.instance;
            pool.successful_mounts += 1;
            pool.session = Some(WritableSession {
                allocator: mounted.allocator,
                current: mounted.current,
                instance,
                shadow_ledger: mounted.output.shadow_ledger,
                parameters_and_device_table: mounted.output.parameters_and_device_table,
                publishes_in_this_mount: publishes,
                carried_to_zero_unit_publishes,
            });
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Mounted {
                    instance,
                    publishes,
                    allocation_records_compared,
                    reuse,
                    space_admission,
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            }
        }
        // 挂载半路报错（写行之后的发布失败）：第 1 件不比分配代，入口没交回写出去的那几次发布；模型按「拒之前一个字节都不写」判。
        Err(error) => {
            let member = mount_error_member(&error);
            let reason = refusal_reason_of_mount_error(&error);
            let verdict = judge_by_model(
                &mut pool.model,
                Ok(answer),
                &observed_refusal(
                    member.clone(),
                    reason,
                    0,
                    stream_length_before,
                    &pool.stream,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                ),
            );
            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
        }
    }
}

fn apply_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    pool.session = None;
    pool.model.close_session();
    pool.mount_attempts += 1;
    let image_before_mount = pool.image();
    let answer = pool.model.answer_mount_writable();
    let stream_length_before = pool.stream.operation_count();
    let mounted =
        mount_writable_with_space_admission(parameters, &mut pool.devices, pool.space_admission);
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

/// 盘上的一段 [起点, 起点 + 长度)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HiddenDeviceRange {
    offset_in_bytes: u64,
    length_in_bytes: u64,
}

impl HiddenDeviceRange {
    fn overlaps(self, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
        self.offset_in_bytes < offset_in_bytes + length_in_bytes
            && offset_in_bytes < self.offset_in_bytes + self.length_in_bytes
    }
}

/// 一次挂载期间点名的那几段读回全 0 的设备（崩溃恢复抛弃根的造法：那几段暂时读不出）。读照样交给里面那块盘
/// （注入计划照样数到这次读，数法与不藏时相同），交回之前把落在藏起来那几段里的字节清零；写原样交下去，
/// 写到哪一段上那一段就不再藏（挂载自己写出的字节此后照常读得出）。
struct DeviceReadingZerosOverHiddenRanges<Device: singlefs_core::block_device::BlockDevice> {
    inner: Device,
    hidden_ranges: Vec<HiddenDeviceRange>,
}

impl<Device: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
    for DeviceReadingZerosOverHiddenRanges<Device>
{
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let buffer_length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        for hidden in &self.hidden_ranges {
            if !hidden.overlaps(offset.0, buffer_length) {
                continue;
            }
            let start_in_buffer = hidden.offset_in_bytes.saturating_sub(offset.0);
            let end_in_buffer =
                (hidden.offset_in_bytes + hidden.length_in_bytes - offset.0).min(buffer_length);
            buffer[usize::try_from(start_in_buffer).expect("落在缓冲区里")
                ..usize::try_from(end_in_buffer).expect("落在缓冲区里")]
                .fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("一次写的长度装得进 u64");
        self.hidden_ranges
            .retain(|hidden| !hidden.overlaps(offset.0, length));
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.hidden_ranges
            .retain(|hidden| !hidden.overlaps(offset.0, length));
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

/// 崩溃恢复要藏起来的那几段：`current`（这个会话的现行版本、根环里最新那条根）的根槽，与它那次发布每一条记录点名的每一份单元的
/// 第一个槽（单元头在那一槽里，读回全 0 整单元校验和就对不上，点名验证不过）。交回 (盘, 那一段)；那次发布一个单元都没点名时交回 None。
fn ranges_hidden_by_the_crash_recovery(
    parameters: &MakeFilesystemParameters,
    current: &PoolVersion,
) -> Option<Vec<(DeviceIdentity, HiddenDeviceRange)>> {
    let earlier_records = match current {
        PoolVersion::WithoutFile(version) => &version.earlier_records_of_this_publish,
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
            &version.earlier_records_of_this_publish
        }
    };
    let named_units: Vec<_> = earlier_records
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
        std::iter::once((
            root_slot_device,
            HiddenDeviceRange {
                offset_in_bytes: root_slot_offset.0,
                length_in_bytes: u64::from(parameters.geometry.physical_block_size),
            },
        ))
        .chain(named_units.into_iter().map(|location| {
            (
                location.device,
                HiddenDeviceRange {
                    offset_in_bytes: location.slot.to_device_offset().0,
                    length_in_bytes: SLOT_BYTES,
                },
            )
        }))
        .collect(),
    )
}

/// 崩溃恢复抛弃根（`HistoryOperation::CrashRecoveryAbandoningTheNewestRoot`）：这个进程崩掉，下一次可写挂载时最新那条根的根槽与
/// 它那次发布点名的单元整次挂载读回全 0；挂载交回之后那几处照旧读得出。那条根是这个会话发的、系统配置见证过它，C554 乙之后
/// 读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），这一步因此造不出被抛弃的根。
/// 模型那一侧答必须拒（`IdealModel::answer_mount_writable_with_the_newest_root_unreadable`）；结局交给可写挂载那一处共用的
/// [`settle_mount`]：拒了就按成员映射出的理由与「录制流有没有多出写」同模型比，会话照旧关着（挂载之前就关了），
/// 后面的步与一次被拒的可写挂载之后一样按「没有可写会话」走。
fn apply_crash_recovery_abandoning_the_newest_root(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    let Some(session) = pool.session.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let image_before_mount = pool.image();
    let (roots, _) = ring_roots_newest_first(&image_before_mount);
    let current_root = session.current.root();
    if roots.first().copied() != Some((current_root.checkpoint_txg.0, current_root.instance.0)) {
        return AppliedStep::not_applicable(
            MissingPrecondition::NewestRingRootIsNotTheCurrentVersionOfThisSession,
        );
    }
    if roots.len() < 2 {
        return AppliedStep::not_applicable(MissingPrecondition::NoReadableRootBeforeTheNewest);
    }
    let Some(hidden) = ranges_hidden_by_the_crash_recovery(parameters, &session.current) else {
        return AppliedStep::not_applicable(MissingPrecondition::NewestPublishNamesNoUnit);
    };
    pool.session = None;
    pool.model.close_session();
    pool.mount_attempts += 1;
    let answer = match pool
        .model
        .answer_mount_writable_with_the_newest_root_unreadable()
    {
        Ok(answer) => answer,
        Err(disagreement) => {
            return AppliedStep {
                outcome: StepOutcome::NotApplicable(
                    MissingPrecondition::NoReadableRootBeforeTheNewest,
                ),
                harness_judgement: None,
                model_verdict: Some(ModelVerdict {
                    disagreement: Some(disagreement),
                    counts: ModelJudgementCounts::default(),
                }),
            }
        }
    };
    let stream_length_before = pool.stream.operation_count();
    let mut devices_with_hidden_ranges: Vec<(
        DeviceIdentity,
        DeviceReadingZerosOverHiddenRanges<HistoryDevice>,
    )> = std::mem::take(&mut pool.devices)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceReadingZerosOverHiddenRanges {
                    inner: device,
                    hidden_ranges: hidden
                        .iter()
                        .filter(|(device_of_the_range, _)| *device_of_the_range == identity)
                        .map(|(_, range)| *range)
                        .collect(),
                },
            )
        })
        .collect();
    let mounted = mount_writable_with_space_admission(
        parameters,
        &mut devices_with_hidden_ranges,
        pool.space_admission,
    );
    pool.devices = devices_with_hidden_ranges
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

/// 几块盘此刻通没通电（[`DeviceLosingPowerAtTheFirstJournalRecordWrite`] 的几块盘共用一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PowerOfTheDevices {
    On,
    /// 某块盘上第一个落在 journal 环里的写到达时掉的电：那个写与之后每一个写、写零、屏障都不落盘。
    LostAtTheFirstJournalRecordWrite,
}

/// 第一个 journal 记录写到达时整池掉电的一块盘（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`] 的造法）：
/// 之前的写与屏障原样交给里面那块盘（录制流照常记下、落盘）；落在 journal 环里的第一个写把几块盘共用的那份电源状态翻成掉电，
/// 从它起每一个写、写零与屏障都不交下去、报 I/O 错——录制流里因此只有落了盘的那一串，与崩溃注入截在那个写之前的盘面相同。
/// 读照样交下去：掉电之后读到的就是落了盘的字节。
struct DeviceLosingPowerAtTheFirstJournalRecordWrite<
    Device: singlefs_core::block_device::BlockDevice,
> {
    inner: Device,
    /// journal 环在每块盘上的字节区间（`singlefs_format::JOURNAL_RING_START_SLOT` 起、`journal_ring_bytes` 长；与
    /// `crate::segments::FixedGeometry::classify` 认 journal 记录写的那一臂同一个区间）。
    journal_ring_offsets_in_bytes: std::ops::Range<u64>,
    power: Rc<Cell<PowerOfTheDevices>>,
}

impl<Device: singlefs_core::block_device::BlockDevice>
    DeviceLosingPowerAtTheFirstJournalRecordWrite<Device>
{
    fn error_after_the_power_loss() -> BlockDeviceError {
        BlockDeviceError::InputOutput(std::io::Error::other(
            "掉电：第一个 journal 记录写到达时整池断电，这个写与之后的写、屏障都没落盘",
        ))
    }

    fn refuse_once_the_power_is_lost(&self) -> Result<(), BlockDeviceError> {
        match self.power.get() {
            PowerOfTheDevices::On => Ok(()),
            PowerOfTheDevices::LostAtTheFirstJournalRecordWrite => {
                Err(Self::error_after_the_power_loss())
            }
        }
    }
}

impl<Device: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
    for DeviceLosingPowerAtTheFirstJournalRecordWrite<Device>
{
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.refuse_once_the_power_is_lost()?;
        if self.journal_ring_offsets_in_bytes.contains(&offset.0) {
            self.power
                .set(PowerOfTheDevices::LostAtTheFirstJournalRecordWrite);
            return Err(Self::error_after_the_power_loss());
        }
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.refuse_once_the_power_is_lost()?;
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.refuse_once_the_power_is_lost()?;
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// [`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]：前提同覆盖写（[`apply_publish_overwrite`]），
/// 经挂着的会话发那次整份重写，盘在它的第一个 journal 记录写到达时掉电（[`DeviceLosingPowerAtTheFirstJournalRecordWrite`]）。
/// 会话的结局不判：进程已经在这次改动里崩掉，没有人收这个结局（掉电之前做完的那几步落了盘、录制流里有它们）。会话连同模型那一侧的会话
/// 一起丢掉，之后照可写挂载那一步（[`apply_mount_writable`]）挂一次、交模型判：模型看不见这次掉电——那次改动一样都没发布，
/// 挂载恢复落到的就是改动之前那一版，与这个进程正常退出之后的可写挂载同一个答案。
fn apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let Some(open_session) = pool.session.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let Some(current_file_version) = open_session.current.file_version() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    if current_file_version.inode_record.is_none() {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
    }
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        ..
    } = pool.session.take().expect("上面判过这个进程的会话开着");
    let mut session_that_loses_power = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
    };
    let content = content_choice.bytes();
    let file = FirstFile {
        content: &content,
        write_time_seconds,
    };
    let power = Rc::new(Cell::new(PowerOfTheDevices::On));
    let journal_ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let journal_ring_offsets_in_bytes =
        journal_ring_start..journal_ring_start + parameters.geometry.journal_ring_bytes;
    let mut devices_losing_power: Vec<(
        DeviceIdentity,
        DeviceLosingPowerAtTheFirstJournalRecordWrite<HistoryDevice>,
    )> = std::mem::take(&mut pool.devices)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceLosingPowerAtTheFirstJournalRecordWrite {
                    inner: device,
                    journal_ring_offsets_in_bytes: journal_ring_offsets_in_bytes.clone(),
                    power: Rc::clone(&power),
                },
            )
        })
        .collect();
    let _outcome_nobody_receives_after_the_crash = session_that_loses_power.publish_user_change(
        &mut devices_losing_power,
        match content_choice.length.whole_file_write_entry() {
            WholeFileWriteEntry::Overwrite => UserChange::Overwrite(file),
            WholeFileWriteEntry::SequentialWrite => UserChange::SequentialWrite(file),
        },
    );
    pool.devices = devices_losing_power
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    // 这个进程到这里就没了（掉电截在第一个 journal 记录写上；那次改动在那之前就被拒的，进程同样不再往下写），
    // 之后录下的是可写挂载另起的实例写的流。
    pool.earliest_stream_index_after_each_power_loss
        .push(pool.stream.operation_count());
    pool.model.close_session();
    apply_mount_writable(pool, parameters)
}

/// 每块盘本池自证过（整槽校验和过、fsid 与本池相同）、世代号最大的那一槽系统配置在盘上的那一段：槽号是世代号对每盘槽数取余，
/// 与发布末尾的轮换、取号那一写取槽同一个取法（`crates/singlefs-core/src/transaction.rs` 取号写系统配置那一处）。交回 (盘, 那一段)，
/// 按 `devices` 的次序；有一块盘自证过的不到两份时交回 None（[`MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots`]）。
fn newest_self_verified_system_configuration_slot_of_each_device(
    parameters: &MakeFilesystemParameters,
    image_before_mount: &MemoryPool,
    devices: &[DeviceIdentity],
) -> Option<Vec<(DeviceIdentity, HiddenDeviceRange)>> {
    let slot_spacing_in_bytes = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    devices
        .iter()
        .map(|device| {
            let self_verified = singlefs_core::recovery::verified_system_configuration_slots(
                image_before_mount,
                *device,
                slot_spacing_in_bytes,
                &parameters.filesystem_identifier,
            );
            if u64::try_from(self_verified.len()).expect("一块盘的系统配置槽数装得进 u64")
                < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
            {
                return None;
            }
            let newest_slot_generation = self_verified
                .iter()
                .map(|system_configuration| system_configuration.quantities.slot_generation)
                .max()
                .expect("上面判过这块盘自证过的系统配置有两份");
            Some((
                *device,
                HiddenDeviceRange {
                    offset_in_bytes: (newest_slot_generation
                        % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                    length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
                },
            ))
        })
        .collect()
}

/// 每块盘最新那一槽系统配置读回全 0 时的可写挂载
/// （`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`）：关掉这个进程的会话，每块盘本池自证过、
/// 世代号最大的那一槽（[`newest_self_verified_system_configuration_slot_of_each_device`]）整次挂载读回全 0
/// （[`DeviceReadingZerosOverHiddenRanges`]：取号那一写落到它上面之后照常读得出）。读回全 0 的槽自证不过、设备不报读错：读阶段的见证取另一槽，
/// R2 判不出不成立，逐盘核与取号照「一槽撕裂、另一槽自证过」往下走（D22（单元原子性怎么合成） 已定项 8 的 S-辨）。模型照一次普通可写挂载答
/// （`IdealModel::answer_mount_writable`：模型不比系统配置落在哪一槽、带的见证值）；结局交给与可写挂载共用的 [`settle_mount`]。
fn apply_writable_mount_with_the_newest_system_configuration_slot_of_each_device_reading_zeros(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    let image_before_mount = pool.image();
    let devices: Vec<DeviceIdentity> = pool.devices.iter().map(|(identity, _)| *identity).collect();
    let Some(hidden) = newest_self_verified_system_configuration_slot_of_each_device(
        parameters,
        &image_before_mount,
        &devices,
    ) else {
        return AppliedStep::not_applicable(
            MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots,
        );
    };
    pool.session = None;
    pool.model.close_session();
    pool.mount_attempts += 1;
    let answer = pool.model.answer_mount_writable();
    let stream_length_before = pool.stream.operation_count();
    let mut devices_with_hidden_ranges: Vec<(
        DeviceIdentity,
        DeviceReadingZerosOverHiddenRanges<HistoryDevice>,
    )> = std::mem::take(&mut pool.devices)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceReadingZerosOverHiddenRanges {
                    inner: device,
                    hidden_ranges: hidden
                        .iter()
                        .filter(|(device_of_the_range, _)| *device_of_the_range == identity)
                        .map(|(_, range)| *range)
                        .collect(),
                },
            )
        })
        .collect();
    let mounted = mount_writable_with_space_admission(
        parameters,
        &mut devices_with_hidden_ranges,
        pool.space_admission,
    );
    pool.devices = devices_with_hidden_ranges
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

fn apply_rollback_while_mounted(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: RollbackTargetChoice,
) -> AppliedStep {
    let image_before_rollback = pool.image();
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let WritableSession {
        allocator,
        current,
        publishes_in_this_mount,
        ..
    } = session;
    let PoolVersion::WithFile(current) = current else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    let (roots, _) = ring_roots_newest_first(&image_before_rollback);
    let Some((newest_txg, newest_instance)) = roots.first().copied() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoReadableRootInRing);
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
            let floor = newest_ring_root_floor(&image_before_rollback);
            // 根环按 (txg, 实例) 从新到旧排，同一个 txg 上先碰到的就是实例最大的那条。
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
    let answer =
        model.answer_rollback_while_mounted(model_root_key(target.instance, target.checkpoint_txg));
    let records_on_disk_before = current.allocation_records.clone();
    let stream_length_before = stream.operation_count();
    match roll_back_by_a_forward_publish(parameters, devices, allocator, current, target) {
        Ok(rolled_back) => {
            *publishes_in_this_mount += 1;
            let verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed_root_of_file_version(current)],
                    reported_ceiling: None,
                }),
            );
            // 盘上的分配记录：改写或新增的记录代是这次的 txg；复活的那几份例外——分配代写回回退目标那一版账里的
            // （D23（journal 的角色与格式） 已定项 14「复活」）。
            let rollback_txg = current.root.checkpoint_txg;
            let records = records_changed_with_a_generation_outside_the_publish_txgs(
                &records_on_disk_before,
                &current.allocation_records,
                rollback_txg,
                rollback_txg,
            )
            .into_iter()
            .filter(|record| {
                !rolled_back
                    .resurrected_user_visible_copies
                    .iter()
                    .any(|copy| {
                        copy.device == record.device
                            && copy.placement.slot == record.slot
                            && copy.allocation_generation == record.generation
                            && !record.is_released
                    })
            })
            .collect::<Vec<_>>();
            let harness_judgement = (!records.is_empty()).then_some(
                HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
                    records,
                    first_publish_txg: rollback_txg,
                    last_publish_txg: rollback_txg,
                },
            );
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::RolledBack {
                    target,
                    released_user_visible_units: rolled_back.released_user_visible_units.len(),
                    resurrected_user_visible_copies: rolled_back
                        .resurrected_user_visible_copies
                        .len(),
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            }
        }
        Err(error) => {
            let member = rollback_error_member(&error);
            let reason = refusal_reason_of_rollback_error(&error);
            let mut verdict = judge_by_model(
                model,
                answer,
                &observed_refusal(
                    member.clone(),
                    reason,
                    0,
                    stream_length_before,
                    stream,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_rollback_error(&error),
                ),
            );
            // 拒在回退目标引用的一个单元的那一份上：把实现点名的那一份记进对不上的那一格（故障注入拿它与被吞的写落在哪比，A4）。
            if let Some(disagreement) = verdict.disagreement.as_mut() {
                disagreement.note_the_unit_copy_a_rollback_refused_on(
                    unit_copy_unreadable_or_mismatched_of_rollback_error(&error),
                    stream.operation_count() == stream_length_before,
                );
            }
            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
        }
    }
}

fn apply_raise_rollback_floor(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: FloorTargetChoice,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let WritableSession {
        allocator,
        current,
        publishes_in_this_mount,
        ..
    } = session;
    let PoolVersion::WithFile(current) = current else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    let current_floor = current.root.rollback_floor.0;
    let txg_before_raise = current.root.checkpoint_txg.0;
    // 前提：目标只取现行 F 及以上（出处见 `FloorTargetChoice::steps_above_current_floor` 的注释）。
    let choices = txg_before_raise.saturating_sub(current_floor) + 3;
    let new_floor = CheckpointTxg(current_floor + choice.steps_above_current_floor % choices);
    let records_before = allocator.records().to_vec();
    let records_on_disk_before = current.allocation_records.clone();
    let answer = model.answer_raise_rollback_floor(ModelCheckpointTxg(new_floor.0));
    let stream_length_before = stream.operation_count();
    let raised = raise_rollback_floor(
        parameters,
        devices,
        allocator,
        current,
        new_floor,
        ShadowLedger::On,
    );
    // 抬 F 的空发布逐次把现行版本往前推：半路报错时已经推出去的那几次也算这次挂载写出的根。
    let publishes_completed = usize::try_from(current.root.checkpoint_txg.0 - txg_before_raise)
        .expect("一次抬 F 至多推根环区域数那么多次");
    *publishes_in_this_mount += publishes_completed;
    match raised {
        Ok(raised) => {
            let verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: raised
                        .publishes
                        .iter()
                        .map(observed_root_of_file_version)
                        .collect(),
                    reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),
                }),
            );
            // 逐次发布比盘上的分配记录：每一次改写或新增的记录，代是那一次的 txg。
            let mut records_of_previous_publish = &records_on_disk_before;
            let mut harness_judgement = None;
            for publish in &raised.publishes {
                let publish_txg = publish.root.checkpoint_txg;
                harness_judgement = harness_judgement.or_else(|| {
                    allocation_generation_judgement(
                        records_of_previous_publish,
                        &publish.allocation_records,
                        publish_txg,
                        publish_txg,
                    )
                });
                records_of_previous_publish = &publish.allocation_records;
            }
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::RaisedFloor {
                    new_floor,
                    publishes: raised.publishes.len(),
                    reclaimed_placements: raised.reclaimed.len(),
                    reuse: RecordReuse::between(&records_before, allocator.records()),
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            }
        }
        // 半路报错：推出去的那几次没有逐次的输出，现行版本是最后成功的那一次；改写或新增的记录的代要落在这几次的 txg 里。
        // 模型比理由（容量墙按做完的次数判区间）与上限。
        Err(error) => {
            let member = mount_error_member(&error);
            let reason = refusal_reason_of_mount_error(&error);
            let verdict = judge_by_model(
                model,
                answer,
                &observed_refusal(
                    member.clone(),
                    reason,
                    publishes_completed,
                    stream_length_before,
                    stream,
                    reported_ceiling_of_mount_error(&error),
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                ),
            );
            AppliedStep {
                outcome: StepOutcome::Refused { member },
                harness_judgement: allocation_generation_judgement(
                    &records_on_disk_before,
                    &current.allocation_records,
                    CheckpointTxg(txg_before_raise + 1),
                    current.root.checkpoint_txg,
                ),
                model_verdict: Some(verdict),
            }
        }
    }
}

fn apply_cold_start_recover(pool: &mut HistoryPool) -> AppliedStep {
    pool.session = None;
    pool.model.close_session();
    let answer = pool.model.answer_cold_start_recover();
    let report = recover(&pool.devices, JournalPolicy::Consult);
    // 读回的根取施加 journal 记录前缀之后实际走的那条（`observed_read_back_after_a_crash`），不取施加之前所选的：
    // 两者只在「记录已持久、根槽没落」时不等，不建崩溃的历史里说谎的设备（根槽 FUA 被吞）照样造得出这个窗口
    // （实四丙交回第二节 ④ offset 330：读回的内容正是最新确认的那一版，对不上的只是根的 key）。
    let verdict = judge_by_model(
        &mut pool.model,
        Ok(answer),
        &ObservedOutcome::Succeeded(ObservedEffect::ColdStart {
            read_back: observed_read_back_after_a_crash(&report),
        }),
    );
    let read_back = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ColdStartReadBack::NoFile,
        RecoveryOutcome::FileRead { .. } => ColdStartReadBack::FileRead,
        RecoveryOutcome::Failed { .. } => ColdStartReadBack::Failed,
    };
    AppliedStep::judged_by_outcome_and_model(
        StepOutcome::Applied(AppliedEffect::Recovered {
            outcome: recovery_outcome_member(&report.outcome),
            read_back,
        }),
        verdict,
    )
}

fn apply_operation(
    pool: &mut HistoryPool,
    operation: &HistoryOperation,
    step_index: usize,
) -> AppliedStep {
    let parameters = pool.device_width.parameters();
    // 写入时间是参数、不取系统时钟：同一段历史两次跑逐字节相同。
    let write_time_seconds =
        FIXED_WRITE_TIME_SECONDS + u64::try_from(step_index).expect("步号装得进 u64");
    // 零单元发布不写分配记录，这里不比；挂载写出的写行与暖机那几次在 `settle_mount` 里比（代码三方第二轮判决第二节第 1 条）。
    match operation {
        HistoryOperation::PublishFirstFile(content) => {
            apply_publish_first_file(pool, &parameters, content, write_time_seconds)
        }
        HistoryOperation::PublishOverwrite(content) => {
            apply_publish_overwrite(pool, content, write_time_seconds)
        }
        HistoryOperation::PublishWithoutUnits => apply_publish_without_units(pool, &parameters),
        HistoryOperation::CloseAndMountWritable => apply_mount_writable(pool, &parameters),
        HistoryOperation::RollBackWhileMounted(choice) => {
            let applied = apply_rollback_while_mounted(pool, &parameters, *choice);
            file_the_current_version_by_what_the_rollback_took(pool);
            applied
        }
        HistoryOperation::RaiseRollbackFloor(choice) => {
            apply_raise_rollback_floor(pool, &parameters, *choice)
        }
        HistoryOperation::ColdStartRecover => apply_cold_start_recover(pool),
        HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
            apply_crash_recovery_abandoning_the_newest_root(pool, &parameters)
        }
        HistoryOperation::Unlink => apply_unlink(pool, write_time_seconds),
        HistoryOperation::CreateInode => apply_create_inode(pool, write_time_seconds),
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
            apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
                pool,
                &parameters,
                content,
                write_time_seconds,
            )
        }
        HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            apply_writable_mount_with_the_newest_system_configuration_slot_of_each_device_reading_zeros(
                pool,
                &parameters,
            )
        }
    }
}

/// 回退在会话的带文件那一版上就地发（`mount::roll_back_by_a_forward_publish` 改的是 `PoolVersion::WithFile` 里那一版）：
/// 回退到删完的一版之后这一版有树而文件集合为空，按 `PoolVersion::of_a_version_of_trees` 的判法重新归成员
/// （不归的话之后的覆盖写照「带文件」调进会话、接在没有文件的一版后面）。
fn file_the_current_version_by_what_the_rollback_took(pool: &mut HistoryPool) {
    let Some(session) = pool.session.as_mut() else {
        return;
    };
    let PoolVersion::WithFile(version) = &session.current else {
        return;
    };
    if version.is_a_version_with_trees_and_no_file() {
        session.current = PoolVersion::WithTreesAndNoFile(version.clone());
    }
}

/// 下一张交出去的镜像在交出去的次序里的下标（从 0 数）：已经交了几张就是几。
fn step_image_index_of_the_next_hand_out(tally: &HistoryTally) -> usize {
    usize::try_from(tally.step_images_handed_out)
        .expect("一段历史交出去的镜像张数不超过步数加一，装得进 usize")
}

/// 把一张池级 checker 本来要判的镜像交给每步镜像那个钩子，记一张。
fn hand_out_a_step_image(
    step_images: &mut dyn FnMut(&StepImage<'_>),
    tally: &mut HistoryTally,
    step_image: &StepImage<'_>,
) {
    tally.step_images_handed_out += 1;
    step_images(step_image);
}

/// 一次失败对「已知红」清单：按清单次序第一条对得上的就是它，都对不上是新发现。
#[must_use]
pub fn classify_failure(observation: FailureObservation) -> HistoryEnding {
    match KNOWN_RED_FORMS
        .iter()
        .position(|form| (form.matches)(&observation))
    {
        Some(form) => HistoryEnding::KnownRed { form, observation },
        None => HistoryEnding::NewFinding {
            signature: FailureSignature::of(&observation),
            observation,
        },
    }
}

enum_listing_every_member! {
/// 每一步之后 harness 自己跑不跑池级 checker：一律不跑，每一步的镜像经钩子交出去（[`execute_history_handing_out_every_step_image`]），
/// 池级 checker 那一遍归 checker 档两段（镜像作为给定状态写进判定库、判段从库读、判在 GPU；
/// `crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`），
/// harness 随时跑的只剩与理想模型、RefFS 比对那一截（用户 2026-10-13 定）。原先 harness 就地判、判红就停的那一种删了（用户 2026-10-14 定）：
/// 要判红就停那一档口径的调用方（E158 的故障注入）经故障注入的合并入口收 checker 档判回的结论
/// （`crate::fault_injection::inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerStepChecker {
    /// 不跑：只由理想模型、执行器自己的判定与 panic 让历史停下，每一步的镜像经钩子交出去（harness 档的用例、崩溃注入、坏盘输入、故障注入都用它）。
    Skipped,
}
}

impl PerStepChecker {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            PerStepChecker::Skipped => "不跑池级 checker（只由模型、执行器的判定与 panic 判）",
        }
    }

    /// 复现行与复现入口的环境变量里这一种的取值（`HistorySamplingVariable::Checker`）。
    #[must_use]
    pub const fn environment_value(self) -> &'static str {
        match self {
            PerStepChecker::Skipped => "skip",
        }
    }
}

/// 一段历史怎么跑：每一步之后的 checker、两块盘多宽（取样点在种子、步数、比重之外的两个参数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryExecution {
    pub per_step_checker: PerStepChecker,
    pub device_width: HistoryDeviceWidth,
    /// 判不判空间准入（只供测试的开关，`singlefs_core::admission::SpaceAdmission`）。取样点里要测「准入放行而落点取不到」那条兜底拒绝的
    /// （小盘上式子先拒，走不到落点那一道）装 `SkippedByTheTestOnlySwitch`；别的一律判。
    pub space_admission: SpaceAdmission,
}

impl HistoryExecution {
    /// harness 档用例的执行器：harness 不跑每一步的池级 checker，每一步的镜像经钩子交出去（池级 checker 那一遍归 checker 档两段），
    /// 两块 4 GiB 的盘、判空间准入。
    pub const HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution =
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };

    /// 报告里的名字。
    #[must_use]
    pub fn name(self) -> String {
        match self.space_admission {
            SpaceAdmission::JudgedByTheFormula => format!(
                "{}；{}",
                self.per_step_checker.name(),
                self.device_width.name()
            ),
            SpaceAdmission::SkippedByTheTestOnlySwitch => format!(
                "{}；{}；空间准入关掉（只供测试的开关，{}）",
                self.per_step_checker.name(),
                self.device_width.name(),
                self.space_admission.branch_name()
            ),
        }
    }
}

/// 复现行与复现入口的环境变量里空间准入那一臂的取值（`HistorySamplingVariable::SpaceAdmission`）。复现入口认的空间准入全集是核心层生成的
/// `SpaceAdmission::ALL`（不在这里另抄一张）：核心层加一个成员，这里的穷举 `match` 不补取值就编不过，补了复现入口就认得它。
#[must_use]
pub const fn space_admission_environment_value(space_admission: SpaceAdmission) -> &'static str {
    match space_admission {
        SpaceAdmission::JudgedByTheFormula => "judged",
        SpaceAdmission::SkippedByTheTestOnlySwitch => "skipped",
    }
}

/// 复现行与复现入口里环境变量名的前缀：每个装置一个（随机历史 [`RANDOM_HISTORY_VARIABLE_PREFIX`]，崩溃注入与坏盘输入在 checker 档各起一个，
/// 故障注入在它的用例文件 `tests/fault_injection_fast_tier.rs` 里起一个），
/// 后缀由 [`HistorySamplingVariable::suffix`] 与各装置自己的几个变量定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReproductionVariablePrefix(pub &'static str);

impl ReproductionVariablePrefix {
    /// 前缀接上后缀的整个变量名。
    #[must_use]
    pub fn variable_named(self, suffix: &str) -> String {
        format!("{}{suffix}", self.0)
    }
}

/// 随机历史那个二进制的环境变量前缀。
pub const RANDOM_HISTORY_VARIABLE_PREFIX: ReproductionVariablePrefix =
    ReproductionVariablePrefix("SINGLEFS_RANDOM_HISTORY_");

/// 「收缩一个种子」那条入口读的种子变量（接在 [`RANDOM_HISTORY_VARIABLE_PREFIX`] 后面）。
pub const RANDOM_HISTORY_SHRINK_SEED_SUFFIX: &str = "SHRINK_SEED";

/// [`HistorySampling`] 的每一项在环境变量里的那一个（后缀）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistorySamplingVariable {
    Operations,
    Weights,
    Checker,
    Devices,
    SpaceAdmission,
}

impl HistorySamplingVariable {
    /// 复现行按这个次序写出 [`HistorySampling`] 的每一项。
    pub const ALL: [HistorySamplingVariable; 5] = [
        HistorySamplingVariable::Operations,
        HistorySamplingVariable::Weights,
        HistorySamplingVariable::Checker,
        HistorySamplingVariable::Devices,
        HistorySamplingVariable::SpaceAdmission,
    ];

    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            HistorySamplingVariable::Operations => "OPERATIONS",
            HistorySamplingVariable::Weights => "WEIGHTS",
            HistorySamplingVariable::Checker => "CHECKER",
            HistorySamplingVariable::Devices => "DEVICES",
            HistorySamplingVariable::SpaceAdmission => "SPACE_ADMISSION",
        }
    }
}

/// 复现入口读环境变量时认不得的输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReproductionEnvironmentError {
    /// 这个变量必给（种子这一类），没设。
    Missing { variable: String },
    /// 设了，取值不在认得的那几个里（数值变量认得的是「非负整数」）。
    Unrecognized {
        variable: String,
        value: String,
        recognized: Vec<String>,
    },
}

impl ReproductionEnvironmentError {
    /// 给人看：哪个变量、给了什么、认得什么。
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            ReproductionEnvironmentError::Missing { variable } => format!("要给 {variable}"),
            ReproductionEnvironmentError::Unrecognized {
                variable,
                value,
                recognized,
            } => format!("{variable}={value}：只认 {}", recognized.join("、")),
        }
    }
}

/// 读一个非负整数变量：没设取 `default`。前后空白先去掉，与数值环境变量共用的那一份解析
/// （`numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable`）同一口径：
/// 同一个复现入口里有的数经它、有的数经共用解析，`" 11"` 不能这边照收、那边停下。
///
/// # Errors
/// 设了但不是非负整数。
pub fn number_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    default: u64,
) -> Result<u64, ReproductionEnvironmentError> {
    Ok(
        number_given_in_reproduction_environment::<u64>(lookup, variable, "非负整数")?
            .unwrap_or(default),
    )
}

/// 读一个「跑几个」的计数变量（种子数、每段步数、每段注入次数这一类）：没设取 `default`；写 0 与写错一样认不得。
/// 0 个就是这一批一项都没跑，照样跑完、照样退 0，按只判清单外失败的那一组判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」）。
/// 前后空白照 [`number_from_reproduction_environment`] 先去掉。
///
/// # Errors
/// 设了但不是正整数。
pub fn count_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    default: u64,
) -> Result<u64, ReproductionEnvironmentError> {
    Ok(
        number_given_in_reproduction_environment::<std::num::NonZeroU64>(
            lookup,
            variable,
            COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT,
        )?
        .map_or(default, std::num::NonZeroU64::get),
    )
}

/// [`count_from_reproduction_environment`] 认不得时报的「认得的取值」。
pub const COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT: &str = "正整数（0 个就是一项都没跑）";

/// 复现入口里一个数值变量的取值换成 `Number`：没设是 `None`；前后空白先去掉；认不出交回带原值的
/// [`ReproductionEnvironmentError::Unrecognized`]，认得的取值写成 `recognized`。
fn number_given_in_reproduction_environment<Number: std::str::FromStr>(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    recognized: &str,
) -> Result<Option<Number>, ReproductionEnvironmentError> {
    let Some(text) = lookup(variable) else {
        return Ok(None);
    };
    match text.trim().parse::<Number>() {
        Ok(number) => Ok(Some(number)),
        Err(_not_a_number) => Err(ReproductionEnvironmentError::Unrecognized {
            variable: variable.to_string(),
            value: text,
            recognized: vec![recognized.to_string()],
        }),
    }
}

/// 读一个必给的非负整数变量（种子这一类）。
///
/// # Errors
/// 没设，或不是非负整数。
pub fn required_number_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
) -> Result<u64, ReproductionEnvironmentError> {
    match lookup(variable) {
        None => Err(ReproductionEnvironmentError::Missing {
            variable: variable.to_string(),
        }),
        Some(_given) => number_from_reproduction_environment(lookup, variable, 0),
    }
}

/// 读一个取值在 `choices` 里的变量：按 `environment_value_of` 认，没设取 `default`。
///
/// # Errors
/// 设了，取值不是 `choices` 里任一个的 `environment_value_of`。
pub fn choice_from_reproduction_environment<Choice: Copy>(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
    choices: &[Choice],
    environment_value_of: fn(Choice) -> &'static str,
    default: Choice,
) -> Result<Choice, ReproductionEnvironmentError> {
    let Some(value) = lookup(variable) else {
        return Ok(default);
    };
    choices
        .iter()
        .copied()
        .find(|choice| environment_value_of(*choice) == value)
        .ok_or_else(|| ReproductionEnvironmentError::Unrecognized {
            variable: variable.to_string(),
            value,
            recognized: choices
                .iter()
                .map(|choice| environment_value_of(*choice).to_string())
                .collect(),
        })
}

/// 种子之外，复现一段随机历史要的全部参数：每段步数、比重、怎么跑（每一步之后的 checker、盘宽、空间准入）。
/// 判红时打的复现行（[`HistorySampling::environment_assignments`]）与复现入口的解析（[`HistorySampling::from_environment`]）
/// 都经它、经同一张变量表（[`HistorySamplingVariable`]），格式只有这一份定义。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistorySampling {
    pub operations_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
}

impl HistorySampling {
    /// 这一项在环境变量里的取值。
    #[must_use]
    pub fn environment_value_of(&self, variable: HistorySamplingVariable) -> String {
        match variable {
            HistorySamplingVariable::Operations => self.operations_per_history.to_string(),
            HistorySamplingVariable::Weights => self.weights.environment_value().to_string(),
            HistorySamplingVariable::Checker => self
                .execution
                .per_step_checker
                .environment_value()
                .to_string(),
            HistorySamplingVariable::Devices => {
                self.execution.device_width.environment_value().to_string()
            }
            HistorySamplingVariable::SpaceAdmission => {
                space_admission_environment_value(self.execution.space_admission).to_string()
            }
        }
    }

    /// 复现行里这几项的「变量名、取值」，按 [`HistorySamplingVariable::ALL`] 的次序。
    #[must_use]
    pub fn environment_assignments(
        &self,
        prefix: ReproductionVariablePrefix,
    ) -> Vec<(String, String)> {
        HistorySamplingVariable::ALL
            .iter()
            .map(|variable| {
                (
                    prefix.variable_named(variable.suffix()),
                    self.environment_value_of(*variable),
                )
            })
            .collect()
    }

    /// 从环境读回（`lookup` 交回变量的取值，没设是 None）：没设的那一项取 `defaults` 的。
    /// 每段步数是计数（[`count_from_reproduction_environment`]）：每段 0 步的一批只跑了起点，写 0 与写错一样认不得。
    ///
    /// # Errors
    /// 某个变量的取值认不得（每段步数写 0 也算）。
    pub fn from_environment(
        prefix: ReproductionVariablePrefix,
        lookup: &dyn Fn(&str) -> Option<String>,
        defaults: &HistorySampling,
    ) -> Result<HistorySampling, ReproductionEnvironmentError> {
        let operations_variable =
            prefix.variable_named(HistorySamplingVariable::Operations.suffix());
        let operations_per_history_given = count_from_reproduction_environment(
            lookup,
            &operations_variable,
            u64::try_from(defaults.operations_per_history).expect("步数装得进 u64"),
        )?;
        let operations_per_history =
            usize::try_from(operations_per_history_given).map_err(|_too_many| {
                ReproductionEnvironmentError::Unrecognized {
                    variable: operations_variable.clone(),
                    value: operations_per_history_given.to_string(),
                    recognized: vec!["装得进 usize 的步数".to_string()],
                }
            })?;
        let weights = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::Weights.suffix()),
            &GenerationWeights::NAMED,
            |weights| weights.environment_value(),
            defaults.weights,
        )?;
        let per_step_checker = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::Checker.suffix()),
            &PerStepChecker::ALL,
            PerStepChecker::environment_value,
            defaults.execution.per_step_checker,
        )?;
        let device_width = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::Devices.suffix()),
            &HistoryDeviceWidth::ALL,
            HistoryDeviceWidth::environment_value,
            defaults.execution.device_width,
        )?;
        let space_admission = choice_from_reproduction_environment(
            lookup,
            &prefix.variable_named(HistorySamplingVariable::SpaceAdmission.suffix()),
            &SpaceAdmission::ALL,
            space_admission_environment_value,
            defaults.execution.space_admission,
        )?;
        Ok(HistorySampling {
            operations_per_history,
            weights,
            execution: HistoryExecution {
                per_step_checker,
                device_width,
                space_admission,
            },
        })
    }

    /// 按种子生成这一种取样下的那段历史。
    #[must_use]
    pub fn generate(&self, seed: HistorySeed) -> GeneratedHistory {
        generate_history_with_weights(seed, self.operations_per_history, &self.weights)
    }
}

/// 「变量名、取值」拼成一行 `名=值 名=值 …`：复现行的赋值那一截只由它拼（[`environment_assignments_in_line`] 是它的反向）。
#[must_use]
pub fn render_environment_assignments(assignments: &[(String, String)]) -> String {
    assignments
        .iter()
        .map(|(variable, value)| format!("{variable}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 从一行复现命令里取出「变量名=取值」（空白隔开，变量名 `SINGLEFS_` 起头；报告里紧贴在「复现：」这类前缀后面的那一个也认）：
/// 自证拿它把判红时打的那一行原样喂回复现入口。
#[must_use]
pub fn environment_assignments_in_line(line: &str) -> BTreeMap<String, String> {
    line.split_whitespace()
        .filter_map(|token| token.find("SINGLEFS_").map(|start| &token[start..]))
        .filter_map(|assignment| assignment.split_once('='))
        .map(|(variable, value)| (variable.to_string(), value.to_string()))
        .collect()
}

/// 随机历史判红时打的那一行复现命令：种子与这一段的全部取样参数，交「收缩一个种子」那条入口
/// （`random_histories` 的 `shrink_one_failing_seed_from_the_environment`）照抄就是原来那一段历史、原来那种跑法。
#[must_use]
pub fn random_history_reproduction_line(seed: HistorySeed, sampling: &HistorySampling) -> String {
    let mut assignments = vec![(
        RANDOM_HISTORY_VARIABLE_PREFIX.variable_named(RANDOM_HISTORY_SHRINK_SEED_SUFFIX),
        seed.0.to_string(),
    )];
    assignments.extend(sampling.environment_assignments(RANDOM_HISTORY_VARIABLE_PREFIX));
    format!(
        "{} cargo test --release -p singlefs-harness --test random_histories -- --include-ignored --exact shrink_one_failing_seed_from_the_environment",
        render_environment_assignments(&assignments)
    )
}

/// 「收缩一个种子」那条入口读环境：种子必给，取样参数没设的取 `defaults` 的。
/// 读的变量与 [`random_history_reproduction_line`] 写出的逐个相同。
///
/// # Errors
/// 种子没给，或某个变量的取值认不得。
pub fn random_history_to_reproduce_from_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    defaults: &HistorySampling,
) -> Result<(HistorySeed, HistorySampling), ReproductionEnvironmentError> {
    let seed = required_number_from_reproduction_environment(
        lookup,
        &RANDOM_HISTORY_VARIABLE_PREFIX.variable_named(RANDOM_HISTORY_SHRINK_SEED_SUFFIX),
    )?;
    let sampling =
        HistorySampling::from_environment(RANDOM_HISTORY_VARIABLE_PREFIX, lookup, defaults)?;
    Ok((HistorySeed(seed), sampling))
}

/// 跑一段历史：在 `execution.device_width` 那么宽的两块盘上，起点之后与每一步写过盘之后把那一刻的镜像交给观察者（harness 不跑池级 checker，
/// [`PerStepChecker`]）。第一次失败（执行器判出的、模型对不上、panic）就停。盘上的写与屏障录进 `stream`（崩溃注入给开了内容保留的流）。
pub fn execute_history_with(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    execute_history_with_faults(
        history,
        execution,
        stream,
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        observer,
    )
}

/// 同 [`execute_history_with`]，再按调用方给的注入计划在读 / 写 / 刷盘上注入故障（增补 3 第 4 件）。
/// 计划不开时与 [`execute_history_with`] 逐字节相同：注入层只数调用，每一次都原样交给录制器。
pub fn execute_history_with_faults(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    fault_plan: &SharedFaultPlan,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    execute_history_handing_out_every_step_image(
        history,
        execution,
        stream,
        fault_plan,
        observer,
        &mut |_| {},
    )
}

/// 经每步镜像那个钩子交出去的一张镜像：池级 checker 本来在这一刻判的那一张（起点之后、每一步写过盘之后，起点段报错而 mkfs 做完了的那一张）。
/// checker 档把它作为给定状态写进判定库：录制流开了内容保留时，这张镜像就是空盘叠上录制流头 `operations_recorded` 步（录制流只记落了盘的写）。
pub struct StepImage<'step> {
    pub position: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
    /// 这一刻录制流里有几步（写与屏障；[`SharedStream::operation_count`]）。
    pub operations_recorded: usize,
    pub image: &'step MemoryPool,
}

/// 同 [`execute_history_with_faults`]，另把池级 checker 本来要判的每一张镜像交给 `step_images`（harness 一条不变量都不判，
/// 历史只由模型、执行器的判定与 panic 停下）。交出去的次序就是历史里的次序，历史停下的那一步写过盘也交（那一步不调观察者）。
pub fn execute_history_handing_out_every_step_image(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    fault_plan: &SharedFaultPlan,
    observer: &mut dyn FnMut(&StepObservation<'_>),
    step_images: &mut dyn FnMut(&StepImage<'_>),
) -> HistoryRun {
    execute_history_stopping_at_the_given_pool_checker_violations(
        history,
        execution,
        stream,
        fault_plan,
        observer,
        step_images,
        &mut |_| Vec::new(),
    )
}

/// 同 [`execute_history_handing_out_every_step_image`]，另由调用方给每一张交出去的镜像上池级 checker 判红的那几条
/// （`given_pool_checker_violations` 收那一张在交出去的次序里的下标，从 0 数）：给的不空，历史就停在那一步、那几条违例进停下那一步的观察，
/// 与 harness 原先就地判、判红就停的那一档同一个口径（那一档删了，池级 checker 归 checker 档两段）。只给故障注入的合并入口
/// （`crate::fault_injection::inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`）用：它把 checker 档
/// 判回的每步镜像结论交进来，截短重跑那一遍。
pub(crate) fn execute_history_stopping_at_the_given_pool_checker_violations(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    fault_plan: &SharedFaultPlan,
    observer: &mut dyn FnMut(&StepObservation<'_>),
    step_images: &mut dyn FnMut(&StepImage<'_>),
    given_pool_checker_violations: &mut dyn FnMut(usize) -> Vec<(&'static str, String)>,
) -> HistoryRun {
    let mut tally = HistoryTally::default();
    let mut outcomes: Vec<StepOutcome> = Vec::new();
    let mut pool_slot: Option<HistoryPool> = None;
    let position = Cell::new(StepPosition::StartingPoint);
    // 起点段报错（会话没建起来）时 mkfs 写了几步：checker 档把交出去的镜像建成给定状态表时，mkfs 那几步进基线
    let operations_written_by_make_filesystem_before_the_start_failed: Cell<Option<usize>> =
        Cell::new(None);
    let mut completed_after_the_root_ring_turned = false;
    let body_result = with_panic_capture(|| -> Option<FailureObservation> {
        let started = HistoryPool::start(
            history.starting_point,
            execution.device_width,
            execution.space_admission,
            stream,
            fault_plan,
        );
        let (started_pool, starting_verdict) = match started {
            Ok(started) => started,
            Err(failure) => {
                operations_written_by_make_filesystem_before_the_start_failed
                    .set(Some(failure.operations_written_by_make_filesystem));
                // 起点段有一个入口返回了错误（只有故障注入把注入点摆进起点段时走得到）。mkfs 自己就报错的那一格不交镜像、
                // 也没有池级 checker 的结论：盘上没有一个做完了的文件系统，checker 判的每一条都没有对象。
                let violations = if failure.step.finished_filesystem_is_on_the_devices() {
                    let step_image_index = step_image_index_of_the_next_hand_out(&tally);
                    hand_out_a_step_image(
                        step_images,
                        &mut tally,
                        &StepImage {
                            position: StepPosition::StartingPoint,
                            operation_kind: None,
                            operations_recorded: stream.operation_count(),
                            image: &failure.image,
                        },
                    );
                    given_pool_checker_violations(step_image_index)
                } else {
                    Vec::new()
                };
                let mut observation = failure_observation(
                    &failure.image,
                    StepPosition::StartingPoint,
                    None,
                    violations,
                    None,
                );
                observation.harness_judgement =
                    Some(HarnessJudgement::TheStartingPointReturnedAnError {
                        step: failure.step,
                        error: failure.error,
                    });
                return Some(observation);
            }
        };
        let pool = pool_slot.insert(started_pool);
        let mut roots_written_by_the_model_before_this_step =
            measure_the_roots_the_model_wrote_in_one_step(
                &mut tally,
                &pool.model,
                0,
                StepPosition::StartingPoint,
            );
        let mut image = pool.image();
        let mut checked_stream_length = stream.operation_count();
        let starting_point_step_image_index = step_image_index_of_the_next_hand_out(&tally);
        hand_out_a_step_image(
            step_images,
            &mut tally,
            &StepImage {
                position: StepPosition::StartingPoint,
                operation_kind: None,
                operations_recorded: checked_stream_length,
                image: &image,
            },
        );
        let violations_after_the_starting_point =
            given_pool_checker_violations(starting_point_step_image_index);
        if let Some(verdict) = &starting_verdict {
            tally.note_model_verdict(verdict);
        }
        let starting_disagreement = starting_verdict.and_then(|verdict| verdict.disagreement);
        // 起点判红一律停。
        if !violations_after_the_starting_point.is_empty() || starting_disagreement.is_some() {
            let mut observation = failure_observation(
                &image,
                StepPosition::StartingPoint,
                None,
                violations_after_the_starting_point,
                None,
            );
            observation.model_disagreement = starting_disagreement;
            return Some(observation);
        }
        observer(&StepObservation {
            position: StepPosition::StartingPoint,
            operation: None,
            outcome: None,
            image: &image,
            model: &pool.model,
        });
        for (step_index, operation) in history.operations.iter().enumerate() {
            let step_position = StepPosition::Operation(step_index);
            position.set(step_position);
            let AppliedStep {
                outcome,
                harness_judgement,
                model_verdict,
            } = apply_operation(pool, operation, step_index);
            roots_written_by_the_model_before_this_step =
                measure_the_roots_the_model_wrote_in_one_step(
                    &mut tally,
                    &pool.model,
                    roots_written_by_the_model_before_this_step,
                    step_position,
                );
            tally.note_outcome(operation, &outcome);
            if let Some(verdict) = &model_verdict {
                tally.note_model_verdict(verdict);
            }
            let model_disagreement = model_verdict.and_then(|verdict| verdict.disagreement);
            if let Some(session) = &pool.session {
                tally.most_publishes_in_one_mount = tally
                    .most_publishes_in_one_mount
                    .max(session.publishes_in_this_mount);
                tally.highest_checkpoint_txg = tally
                    .highest_checkpoint_txg
                    .max(session.current.root().checkpoint_txg.0);
                tally.most_allocation_records_in_one_version = tally
                    .most_allocation_records_in_one_version
                    .max(u64::try_from(session.allocator.records().len()).expect("条数装得进 u64"));
            }
            let raised_floor = if let StepOutcome::Applied(AppliedEffect::RaisedFloor {
                new_floor,
                ..
            }) = &outcome
            {
                Some(*new_floor)
            } else {
                None
            };
            outcomes.push(outcome);
            let stream_length = stream.operation_count();
            let mut violations = Vec::new();
            let mut raised_floor_lands_only_on_abandoned = None;
            if stream_length == checked_stream_length {
                tally.checker_runs_skipped_because_nothing_was_written += 1;
            } else {
                let image_before_this_step = std::mem::replace(&mut image, pool.image());
                checked_stream_length = stream_length;
                let step_image_index = step_image_index_of_the_next_hand_out(&tally);
                hand_out_a_step_image(
                    step_images,
                    &mut tally,
                    &StepImage {
                        position: step_position,
                        operation_kind: Some(operation.kind()),
                        operations_recorded: stream_length,
                        image: &image,
                    },
                );
                violations = given_pool_checker_violations(step_image_index);
                if !violations.is_empty() {
                    raised_floor_lands_only_on_abandoned = raised_floor.and_then(|new_floor| {
                        raised_floor_lands_only_on_abandoned_roots(
                            &image_before_this_step,
                            new_floor,
                        )
                    });
                }
            }
            if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()
            {
                let mut observation = failure_observation(
                    &image,
                    step_position,
                    Some(operation.kind()),
                    violations,
                    None,
                );
                observation.harness_judgement = harness_judgement;
                observation.model_disagreement = model_disagreement;
                observation.raised_floor_lands_only_on_abandoned_roots =
                    raised_floor_lands_only_on_abandoned;
                return Some(observation);
            }
            observer(&StepObservation {
                position: step_position,
                operation: Some(operation),
                outcome: outcomes.last(),
                image: &image,
                model: &pool.model,
            });
        }
        let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
        completed_after_the_root_ring_turned =
            root_ring_has_turned(newest_ring_root_txg, root_ring_slot_count);
        None
    });
    let failure = match body_result {
        Ok(failure) => failure,
        Err(panic) => {
            let step_position = position.get();
            let operation_kind = match step_position {
                StepPosition::StartingPoint => None,
                StepPosition::Operation(step_index) => history
                    .operations
                    .get(step_index)
                    .map(HistoryOperation::kind),
            };
            // panic 之后的盘面照样读一遍根环（读本身也包在捕获里：坏到 checker 的解析也 panic 时只丢掉这两个事实）。
            let (newest_ring_root_txg, root_ring_slot_count) = pool_slot
                .as_ref()
                .and_then(|pool| {
                    with_panic_capture(|| newest_ring_root_and_slot_count(&pool.image())).ok()
                })
                .unwrap_or((None, None));
            Some(FailureObservation {
                position: step_position,
                operation_kind,
                violations: Vec::new(),
                panic: Some(panic),
                newest_ring_root_txg,
                root_ring_slot_count,
                harness_judgement: None,
                model_disagreement: None,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            })
        }
    };
    if let Some(pool) = &pool_slot {
        tally.most_successful_mounts_in_one_history = pool.successful_mounts;
        tally.most_mount_attempts_in_one_history = pool.mount_attempts;
    }
    let ending = match failure {
        None => {
            tally.histories_completed = 1;
            if completed_after_the_root_ring_turned {
                tally.histories_that_turned_the_root_ring = 1;
            }
            HistoryEnding::Completed
        }
        Some(observation) => {
            if observation.root_ring_has_turned() {
                tally.histories_that_turned_the_root_ring = 1;
            }
            let ending = classify_failure(observation);
            match &ending {
                HistoryEnding::KnownRed { form, .. } => {
                    tally.histories_ended_known_red.insert(*form, 1);
                }
                HistoryEnding::NewFinding { .. } => tally.histories_ended_new_finding = 1,
                HistoryEnding::Completed => {}
            }
            ending
        }
    };
    tally.histories = 1;
    tally.histories_with_a_deletion_batch_keeping_data_units = u64::from(
        tally
            .model_counts
            .deletion_batches_keeping_data_units_matched
            > 0,
    );
    tally.histories_with_a_walk_of_several_steps_before_a_user_change = u64::from(
        tally
            .model_counts
            .walks_of_several_steps_before_a_user_change_matched
            > 0,
    );
    tally.histories_with_a_walk_before_a_user_change_refused_for_space = u64::from(
        tally
            .model_counts
            .walks_before_a_user_change_refused_for_space_matched
            > 0,
    );
    HistoryRun {
        ending,
        outcomes,
        tally,
        // mkfs 在 `HistoryPool::start` 里跑：起点段报错时（`pool_slot` 没填）报那一刻 mkfs 写了的步数；它自己 panic 时流里还没有别的东西，报 0。
        operations_written_by_make_filesystem: pool_slot.as_ref().map_or_else(
            || {
                operations_written_by_make_filesystem_before_the_start_failed
                    .get()
                    .unwrap_or(0)
            },
            |pool| pool.operations_written_by_make_filesystem,
        ),
        committed_versions_at_the_end: pool_slot.as_ref().map_or_else(Vec::new, |pool| {
            pool.model
                .committed_versions()
                .into_iter()
                .map(|(key, content)| (key, content.map(|content| content.to_vec())))
                .collect()
        }),
        earliest_stream_index_after_each_power_loss: pool_slot
            .as_ref()
            .map_or_else(Vec::new, |pool| {
                pool.earliest_stream_index_after_each_power_loss.clone()
            }),
    }
}

fn failure_observation(
    image: &MemoryPool,
    position: StepPosition,
    operation_kind: Option<HistoryOperationKind>,
    violations: Vec<(&'static str, String)>,
    panic: Option<CapturedPanic>,
) -> FailureObservation {
    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(image);
    FailureObservation {
        position,
        operation_kind,
        violations,
        panic,
        newest_ring_root_txg,
        root_ring_slot_count,
        harness_judgement: None,
        model_disagreement: None,
        raised_floor_lands_only_on_abandoned_roots: None,
        record_check: RecordCheck::default(),
    }
}

/// 抬 F 之前的镜像上，txg = `new_floor` 的根是不是全属于被抛弃的实例（F 落在回退留下的空档里）：按最新根指着的实例表，
/// 有行 (i, T) 且根的 txg > T 的 i 就是被抛弃的（与 `recovery::root_is_abandoned_by_the_instance_table` 同一句）。那个 txg 上一条根都没有、
/// 系统配置或最新根或它的实例表读不出，都是 None（不算落在空档里）。
/// 读法用的是实现的 `recovery::readable_roots` / `choose_root` / `instance_table_of_root`（代码三方第一轮攻方的 P1 就这么读）：
/// checker 判候选集时解实例表的那一段不对外（`singlefs-checker` 的 `walk.rs` 里 `Walk::instance_table_rows`），这里没另写一份解析。
#[must_use]
pub fn raised_floor_lands_only_on_abandoned_roots(
    image_before_raising: &dyn PoolReader,
    new_floor: CheckpointTxg,
) -> Option<bool> {
    let system_configuration = choose_system_configuration(image_before_raising).ok()?;
    let roots_at_floor: Vec<RootRecord> = readable_roots(
        image_before_raising,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.checkpoint_txg == new_floor)
    .collect();
    if roots_at_floor.is_empty() {
        return None;
    }
    let newest_root = choose_root(image_before_raising, &system_configuration)?;
    let newest_table = instance_table_of_root(image_before_raising, &newest_root)?;
    Some(roots_at_floor.iter().all(|root| {
        newest_table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    }))
}

/// 按 checker 的读法：根环里最新那条根的 txg 与一圈的槽数 R × S。
#[must_use]
pub fn newest_ring_root_and_slot_count(image: &MemoryPool) -> (Option<u64>, Option<u64>) {
    newest_ring_root_and_slot_count_of(image)
}

/// 同上，读的盘面由调用方给：崩溃注入（增补 3 第 3 件）要在崩溃镜像（`crash::CrashImage`）上也拿到这两个事实
/// （失败观察里的「根环转过一圈」照样报出来）。
#[must_use]
pub fn newest_ring_root_and_slot_count_of(image: &dyn ImageReader) -> (Option<u64>, Option<u64>) {
    let (roots, root_ring_slot_count) = ring_roots_newest_first_of(image);
    (roots.first().map(|(txg, _)| *txg), root_ring_slot_count)
}

/// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
fn root_ring_has_turned(
    newest_ring_root_txg: Option<u64>,
    root_ring_slot_count: Option<u64>,
) -> bool {
    matches!(
        (newest_ring_root_txg, root_ring_slot_count),
        (Some(newest), Some(slot_count)) if newest >= slot_count
    )
}

/// 收缩时回退目标与抬 F 目标试的小选择子个数：根环至多 24 条可读根，F 的可选个数是现行 txg − F + 3，一段快档历史里不过几十。
const SMALL_SELECTORS_TRIED_WHEN_SHRINKING: u64 = 32;

/// 一步操作的几种更简单的写法，只往简单的方向换（换过去的不会再换回来，收缩因此有界）：内容按 0 字节 < 1 字节 < 别的排，只试排在
/// 现在这个前面的；回退目标与抬 F 目标只试比现在小的选择子（选择子执行时按条数取模：一个很大的选择子与某个小数落到同一条根、同一个 F，
/// 从小到大试，第一个还失败的就是它的小写法）。
fn simpler_variants(operation: HistoryOperation) -> Vec<HistoryOperation> {
    const EMPTY_CONTENT: ContentChoice = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    const SHORTEST_NON_EMPTY_CONTENT: ContentChoice = ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 0 },
        fill_seed: 0,
    };
    let simplicity_rank = |content: ContentChoice| {
        if content == EMPTY_CONTENT {
            0
        } else if content == SHORTEST_NON_EMPTY_CONTENT {
            1
        } else {
            2
        }
    };
    let simpler_contents = move |content: ContentChoice| {
        [EMPTY_CONTENT, SHORTEST_NON_EMPTY_CONTENT]
            .into_iter()
            .filter(move |simpler| simplicity_rank(*simpler) < simplicity_rank(content))
    };
    let smaller_selectors = |selector: u64| 0..selector.min(SMALL_SELECTORS_TRIED_WHEN_SHRINKING);
    match operation {
        HistoryOperation::PublishFirstFile(content) => simpler_contents(content)
            .map(HistoryOperation::PublishFirstFile)
            .collect(),
        HistoryOperation::PublishOverwrite(content) => simpler_contents(content)
            .map(HistoryOperation::PublishOverwrite)
            .collect(),
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
            simpler_contents(content)
                .map(HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable)
                .collect()
        }
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
            index_from_newest,
        }) => smaller_selectors(index_from_newest)
            .map(|smaller| {
                HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                    index_from_newest: smaller,
                })
            })
            .collect(),
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot {
            txg_beyond_newest,
        }) => smaller_selectors(txg_beyond_newest)
            .map(|smaller| {
                HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot {
                    txg_beyond_newest: smaller,
                })
            })
            .collect(),
        HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
            steps_above_current_floor,
        }) => smaller_selectors(steps_above_current_floor)
            .map(|smaller| {
                HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                    steps_above_current_floor: smaller,
                })
            })
            .collect(),
        // 候选集的下沿按那一刻的盘面现解，没有更简单的写法。
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRootAtTheNewestFloor)
        | HistoryOperation::PublishWithoutUnits
        | HistoryOperation::CloseAndMountWritable
        | HistoryOperation::ColdStartRecover
        | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperation::Unlink
        | HistoryOperation::CreateInode
        | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            Vec::new()
        }
    }
}

/// 收缩到最短：先按块删（块长从一半起、删不动就减半，到 1 为止），再逐步换成更简单的写法；`still_fails` 为真的删法与换法才留下。
/// 次数有界：每一轮要么少一步，要么块长减半，块长为 1 且一步都删不动就停。
///
/// 删法按窗口并行试：从当前块起连着 `worker_threads` 块各删一块、同时跑，取块号最小的那个还失败的——与一块一块按次序试的结果逐项相同
/// （按次序试时排在它前面的那几块在同一份序列上试、同样不失败），只是多跑了窗口里排在它后面的几个。
pub fn shrink_operations(
    operations: &[HistoryOperation],
    still_fails: &(dyn Fn(&[HistoryOperation]) -> bool + Sync),
    worker_threads: usize,
) -> Vec<HistoryOperation> {
    let window = worker_threads.max(1);
    let mut kept = operations.to_vec();
    let mut chunk_length = (kept.len() / 2).max(1);
    loop {
        let mut removed_this_pass = false;
        let mut chunk_start = 0;
        while chunk_start < kept.len() {
            let window_starts: Vec<usize> = (0..window)
                .map(|offset| chunk_start + offset * chunk_length)
                .take_while(|start| *start < kept.len())
                .collect();
            let candidates: Vec<Vec<HistoryOperation>> = window_starts
                .iter()
                .map(|start| {
                    let end = (start + chunk_length).min(kept.len());
                    kept[..*start].iter().chain(&kept[end..]).copied().collect()
                })
                .collect();
            let verdicts: Vec<bool> = std::thread::scope(|scope| {
                let evaluations: Vec<_> = candidates
                    .iter()
                    .map(|candidate| scope.spawn(move || still_fails(candidate)))
                    .collect();
                evaluations
                    .into_iter()
                    .map(|evaluation| {
                        evaluation.join().expect(
                            "still_fails 自己不 panic：历史里的 panic 在 execute_history_with 里接住",
                        )
                    })
                    .collect()
            });
            match verdicts.iter().position(|fails| *fails) {
                Some(first_failing) => {
                    kept = candidates[first_failing].clone();
                    chunk_start = window_starts[first_failing];
                    removed_this_pass = true;
                }
                None => {
                    chunk_start =
                        window_starts.last().expect("窗口里至少有当前这一块") + chunk_length;
                }
            }
        }
        if !removed_this_pass {
            if chunk_length == 1 {
                break;
            }
            chunk_length = (chunk_length / 2).max(1);
        }
    }
    for step_index in 0..kept.len() {
        for simpler in simpler_variants(kept[step_index]) {
            let mut candidate = kept.clone();
            candidate[step_index] = simpler;
            if still_fails(&candidate) {
                kept = candidate;
                break;
            }
        }
    }
    kept
}

/// 一段失败的历史收缩到最短：起点不变，签名不变的删法与换法才留下。先截掉失败那一步之后的操作。每一次重跑的 checker 与盘宽与发现它的
/// 那一次相同（`execution`）。
#[must_use]
pub fn shrink_failing_history(
    history: &GeneratedHistory,
    signature: &FailureSignature,
    execution: HistoryExecution,
    worker_threads: usize,
) -> GeneratedHistory {
    let run = |candidate: &GeneratedHistory| {
        execute_history_with(candidate, execution, &SharedStream::new(), &mut |_| {})
    };
    let still_fails = |operations: &[HistoryOperation]| {
        let candidate = GeneratedHistory {
            seed: history.seed,
            starting_point: history.starting_point,
            operations: operations.to_vec(),
        };
        matches!(
            run(&candidate).ending,
            HistoryEnding::NewFinding { signature: found, .. } if found == *signature
        )
    };
    let failing_length = match run(history).ending {
        HistoryEnding::NewFinding { observation, .. } => match observation.position {
            StepPosition::StartingPoint => 0,
            StepPosition::Operation(step_index) => step_index + 1,
        },
        HistoryEnding::Completed | HistoryEnding::KnownRed { .. } => history.operations.len(),
    };
    let operations = shrink_operations(
        &history.operations[..failing_length],
        &still_fails,
        worker_threads,
    );
    GeneratedHistory {
        seed: history.seed,
        starting_point: history.starting_point,
        operations,
    }
}

/// 收缩之后的最短复现：那段历史与它每一步的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShrunkReproduction {
    pub history: GeneratedHistory,
    pub outcomes: Vec<StepOutcome>,
}

/// 收缩一段失败的历史，连同最短复现每一步的结局。
#[must_use]
pub fn shrink_to_reproduction(
    history: &GeneratedHistory,
    signature: &FailureSignature,
    execution: HistoryExecution,
    worker_threads: usize,
) -> ShrunkReproduction {
    let shrunk_history = shrink_failing_history(history, signature, execution, worker_threads);
    let outcomes = execute_history_with(
        &shrunk_history,
        execution,
        &SharedStream::new(),
        &mut |_| {},
    )
    .outcomes;
    ShrunkReproduction {
        history: shrunk_history,
        outcomes,
    }
}

/// 一类新发现：第一个撞到它的种子、同一个签名的全部种子、收缩之后的最短复现（没收缩的是 None）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewFindingReport {
    pub signature: FailureSignature,
    pub first_seed: HistorySeed,
    pub observation: FailureObservation,
    pub seeds: Vec<HistorySeed>,
    pub shrunk: Option<ShrunkReproduction>,
    /// 发现它的那一段的取样参数：复现行照它写（[`random_history_reproduction_line`]）。
    pub sampling: HistorySampling,
}

/// 新发现收不收缩：收缩一类要把那段历史反复重跑几十次，debug 下每次几秒（门禁 checker-tier-crates-mutation-replay 在变异下跑快档时量到过：只收一类也要两分多钟）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingShrinking {
    /// 每一类都收缩第一个种子（大档，release 下跑）。
    EveryFindingClass,
    /// 不收缩，只报种子与失败在哪一步（快档：要的是判红与种子；收缩交给「收缩一个种子」那条 `#[ignore]` 用例或大档）。
    ReportSeedsOnly,
}

impl NewFindingReport {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "新发现 {:?}：第一个种子 {}（同签名的种子 {:?}），在 {:?}（{:?}）之后",
            self.signature,
            self.first_seed.0,
            self.seeds.iter().map(|seed| seed.0).collect::<Vec<_>>(),
            self.observation.position,
            self.observation.operation_kind
        );
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic 在 {}：{}", panic.location, panic.message);
        }
        if let Some(judgement) = &self.observation.harness_judgement {
            let _ = writeln!(text, "  执行器判出：{}：{judgement:?}", judgement.name());
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型答 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        let _ = writeln!(
            text,
            "  复现：{}",
            random_history_reproduction_line(self.first_seed, &self.sampling)
        );
        let Some(shrunk) = &self.shrunk else {
            let _ = writeln!(
                text,
                "  没收缩（这一档只报种子）：照上一行跑「收缩一个种子」那条 #[ignore] 用例"
            );
            return text;
        };
        let _ = writeln!(
            text,
            "  最短复现：起点 {:?}，{} 步",
            shrunk.history.starting_point,
            shrunk.history.operations.len()
        );
        for (step_index, operation) in shrunk.history.operations.iter().enumerate() {
            let outcome = shrunk.outcomes.get(step_index).map_or_else(
                || "（失败在这一步）".to_string(),
                |outcome| format!("{outcome:?}"),
            );
            let _ = writeln!(text, "    {step_index}. {operation:?} → {outcome}");
        }
        text
    }
}

/// 一批种子跑下来的报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignReport {
    pub first_seed: u64,
    pub seed_count: u64,
    /// 这一批每段历史的取样参数（每段步数、比重、跑法）：历史就是照它生成、照它跑的，每个新发现的复现行也照它写。
    pub sampling: HistorySampling,
    pub tally: HistoryTally,
    /// 以「已知红」收尾的种子：(种子, 清单第几条, 在哪一步)。
    pub known_red_hits: Vec<(HistorySeed, usize, StepPosition)>,
    /// 按第一个种子从小到大排。
    pub new_findings: Vec<NewFindingReport>,
}

impl CampaignReport {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "种子 [{}, {})，每段 {} 步，比重：{}；{}",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.sampling.operations_per_history,
            self.sampling.weights.name,
            self.sampling.execution.name()
        );
        text.push_str(&self.tally.render());
        for (form_index, form) in KNOWN_RED_FORMS.iter().enumerate() {
            let hits: Vec<(u64, StepPosition)> = self
                .known_red_hits
                .iter()
                .filter(|(_, hit_form, _)| *hit_form == form_index)
                .map(|(seed, _, position)| (seed.0, *position))
                .collect();
            let _ = writeln!(
                text,
                "已知红第 {form_index} 条（{}）：{} 段；前几个种子 {:?}",
                form.closeout_table_row,
                hits.len(),
                hits.iter().take(8).collect::<Vec<_>>()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render());
        }
        text
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，每段 `operations_per_history` 步、按 `execution` 跑（checker 与盘宽），分给
/// `worker_threads` 个线程（每段历史各用各的盘、各用各的录制流，互不相干）；跑完按种子排好再汇总，结论与线程数、调度次序无关。
/// 新发现按签名归类，按 `shrinking` 收缩各类的第一个种子。
///
/// # Panics
/// 某个线程在历史之外 panic（历史里的 panic 在 `execute_history_with` 里接住，走不到这里）。
#[must_use]
pub fn run_history_campaign(
    first_seed: u64,
    seed_count: u64,
    operations_per_history: usize,
    weights: &GenerationWeights,
    execution: HistoryExecution,
    worker_threads: usize,
    shrinking: FindingShrinking,
) -> CampaignReport {
    // 取样参数只在这里拼一次：历史照它生成、照它跑，报告与每个新发现的复现行也照它写——复现行写出的就是真跑的那一种
    // （代码三方 test-script-align-code-r1 判决 C607：原先生成与复现行各拼一份，复现行那一份写错时没有用例看得见）。
    let sampling = HistorySampling {
        operations_per_history,
        weights: *weights,
        execution,
    };
    let next_offset = AtomicU64::new(0);
    let finished: Mutex<Vec<(GeneratedHistory, HistoryRun)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads.max(1) {
            scope.spawn(|| loop {
                let offset = next_offset.fetch_add(1, Ordering::Relaxed);
                if offset >= seed_count {
                    break;
                }
                let history = sampling.generate(HistorySeed(first_seed + offset));
                let run = execute_history_with(
                    &history,
                    sampling.execution,
                    &SharedStream::new(),
                    &mut |_| {},
                );
                finished
                    .lock()
                    .expect("别的线程拿着这把锁时只做一次 push，不会在锁里 panic")
                    .push((history, run));
            });
        }
    });
    let finished = finished
        .into_inner()
        .expect("线程都已结束；锁里只做 push，没有线程在锁里 panic");
    campaign_report_of_finished_histories(
        first_seed,
        seed_count,
        &sampling,
        finished,
        worker_threads,
        shrinking,
    )
}

/// 一批跑完的历史（次序不限）汇总成报告：按种子排好，计数相加，「已知红」照记，新发现按签名归类、按 `shrinking` 收缩各类的第一个种子；
/// 报告与每个新发现带的取样参数都是 `sampling`（这批历史就是照它生成、照它跑的）。
fn campaign_report_of_finished_histories(
    first_seed: u64,
    seed_count: u64,
    sampling: &HistorySampling,
    mut finished: Vec<(GeneratedHistory, HistoryRun)>,
    worker_threads: usize,
    shrinking: FindingShrinking,
) -> CampaignReport {
    finished.sort_by_key(|(history, _)| history.seed);
    let mut tally = HistoryTally::default();
    let mut known_red_hits = Vec::new();
    let mut findings: BTreeMap<
        FailureSignature,
        (GeneratedHistory, FailureObservation, Vec<HistorySeed>),
    > = BTreeMap::new();
    for (history, run) in finished {
        tally.absorb(&run.tally);
        match run.ending {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, observation } => {
                known_red_hits.push((history.seed, form, observation.position));
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                findings
                    .entry(signature)
                    .or_insert_with(|| (history.clone(), observation, Vec::new()))
                    .2
                    .push(history.seed);
            }
        }
    }
    let mut by_first_seed: Vec<_> = findings.into_iter().collect();
    by_first_seed.sort_by_key(|(_, (history, _, _))| history.seed);
    let new_findings = by_first_seed
        .into_iter()
        .map(|(signature, (history, observation, seeds))| {
            let shrunk = match shrinking {
                FindingShrinking::EveryFindingClass => Some(shrink_to_reproduction(
                    &history,
                    &signature,
                    sampling.execution,
                    worker_threads,
                )),
                FindingShrinking::ReportSeedsOnly => None,
            };
            NewFindingReport {
                signature,
                first_seed: history.seed,
                observation,
                seeds,
                shrunk,
                sampling: *sampling,
            }
        })
        .collect();
    CampaignReport {
        first_seed,
        seed_count,
        sampling: *sampling,
        tally,
        known_red_hits,
        new_findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 留孤儿那一组比重抽得到「覆盖写掉电在 journal 记录之前再挂」，别的每一组比重一次都抽不到（它们的种子钉着的取样点照旧）：
    /// 32 个种子、每段 40 步。判别力：留孤儿那一组里这一步的份数是 0 时第一条红。
    #[test]
    fn the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does() {
        let kind =
            HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable;
        let steps_drawn_with = |weights: &GenerationWeights| {
            (0..32u64)
                .map(|seed_offset| {
                    generate_history_with_weights(
                        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset),
                        40,
                        weights,
                    )
                    .operations
                    .iter()
                    .filter(|operation| operation.kind() == kind)
                    .count()
                })
                .sum::<usize>()
        };
        assert!(
            steps_drawn_with(&GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP) >= 1,
            "留孤儿那一组比重 32 段里一次都没抽到 {kind:?}"
        );
        for weights in GenerationWeights::NAMED {
            if weights.named == NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep {
                continue;
            }
            assert_eq!(
                steps_drawn_with(&weights),
                0,
                "{} 那一组比重抽到了 {kind:?}",
                weights.name
            );
        }
    }

    /// 掉电在 journal 记录之前的那次覆盖写留下实例 1 的孤儿（D18（块里携带什么信息） 已定项 11「回收」一条）：这段写死的历史
    /// （[`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`]）每一步都做成，每一步之后池级 checker 与模型都不红，
    /// 掉电之后那次可写挂载取到实例 2；之后在跑完的盘面上起一次有一行就扫的可写挂载，清扫删掉实例 1 那一行、抹了孤儿的头。
    /// 判别力：掉电那一截不截写（那次覆盖写整次发布成）时模型那一判先红（挂载落到的不是它答的那一版），盘上也没有孤儿、清扫一个头都不抹。
    #[test]
    fn an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring(
    ) {
        use singlefs_core::instance_table_sweep::{
            InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
            InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
            SweepPublishPlacements,
        };
        let history =
            history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring();
        let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
        let stream = SharedStream::retaining_contents();
        let mut image_after_the_last_step = None;
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            image_after_the_last_step = Some(observation.image.clone());
        });
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "这段历史每一步都做成、模型不红（每一步的池级 checker 归 checker 档两段）：{:?}",
            run.outcomes
        );
        assert!(
            matches!(
                run.outcomes.first(),
                Some(StepOutcome::Applied(AppliedEffect::Mounted {
                    instance: InstanceGeneration(2),
                    ..
                }))
            ),
            "掉电之后那次可写挂载做成、取到实例 2：{:?}",
            run.outcomes.first()
        );
        let image = image_after_the_last_step.expect("历史跑完，观察者至少被调过一次");
        let parameters = execution.device_width.parameters();
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device = SparseBlockDevice::new(
                    image.device_size_in_bytes,
                    PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
                );
                device.image = sectors.clone();
                (*identity, device)
            })
            .collect();
        let mounted = singlefs_core::mount::mount_writable_with_instance_table_sweep_configuration(
            &parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            InstanceTableSweepConfiguration {
                switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
                row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
                sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
            },
        )
        .expect("跑完的盘面上可写挂载做得成");
        let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
            rows_deleted,
            heads_erased,
        }) = &mounted.output.instance_table_sweep
        else {
            panic!("清扫删了行：{:?}", mounted.output.instance_table_sweep);
        };
        assert_eq!(
            rows_deleted
                .iter()
                .map(|row| row.instance)
                .collect::<Vec<_>>(),
            vec![InstanceGeneration(1)],
            "清扫删的是实例 1 那一行"
        );
        assert!(
            !heads_erased.is_empty(),
            "清扫抹了实例 1 的孤儿的头：{:?}",
            mounted.output.instance_table_sweep
        );
    }

    /// SplitMix64 的参考输出（Vigna 的 splitmix64.c，种子 0）：手写的实现逐位对得上，换一个混合常数这里就红。
    #[test]
    fn seeded_random_source_reproduces_the_splitmix64_reference_sequence() {
        let mut source = SeededRandomSource::from_seed(0);
        assert_eq!(source.next_word(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(source.next_word(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(source.next_word(), 0x06c4_5d18_8009_454f);
        let mut other = SeededRandomSource::from_seed(1_234_567);
        assert_eq!(other.next_word(), 0x599e_d017_fb08_fc85);
        assert_eq!(other.next_word(), 0x2c73_f084_5854_0fa5);
    }

    /// 复现行的赋值那一截写出的每一种取样参数（每一组比重、每一种 checker（只剩不跑那一种）、四档盘宽、空间准入两臂，各配一个步数）都能经复现入口的解析原样读回：
    /// 一项的取值写了读不回（名字对不上、两项同名），或解析漏读一项，这里就红。
    #[test]
    fn every_sampling_choice_reads_back_from_its_own_reproduction_assignments() {
        let defaults = HistorySampling {
            operations_per_history: 1,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let mut combinations = 0usize;
        for weights in GenerationWeights::NAMED {
            for per_step_checker in PerStepChecker::ALL {
                for device_width in HistoryDeviceWidth::ALL {
                    for space_admission in SpaceAdmission::ALL {
                        let sampling = HistorySampling {
                            operations_per_history: 150 + combinations,
                            weights,
                            execution: HistoryExecution {
                                per_step_checker,
                                device_width,
                                space_admission,
                            },
                        };
                        let line = render_environment_assignments(
                            &sampling.environment_assignments(RANDOM_HISTORY_VARIABLE_PREFIX),
                        );
                        let assignments = environment_assignments_in_line(&line);
                        let read_back = HistorySampling::from_environment(
                            RANDOM_HISTORY_VARIABLE_PREFIX,
                            &|variable: &str| assignments.get(variable).cloned(),
                            &defaults,
                        );
                        assert_eq!(read_back, Ok(sampling), "复现行：{line}");
                        combinations += 1;
                    }
                }
            }
        }
        assert_eq!(
            combinations,
            GenerationWeights::NAMED.len()
                * PerStepChecker::ALL.len()
                * HistoryDeviceWidth::ALL.len()
                * SpaceAdmission::ALL.len(),
            "每一种组合都走过一遍"
        );
    }

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

    /// 每一组有名字的比重：名字找回的那一组自报的也是这个名字（两组比重报同一个名字时，有一组复现入口永远找不回），
    /// [`GenerationWeights::NAMED`] 里同一个位置上就是它，复现入口按它的取值读回的也是它。
    #[test]
    fn every_named_generation_weights_member_finds_its_own_weights_and_reads_back_from_its_value() {
        let defaults = HistorySampling {
            operations_per_history: 1,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        for (position, member) in NamedGenerationWeights::ALL.into_iter().enumerate() {
            assert_eq!(
                member.weights().named,
                member,
                "{member:?} 找回的那一组比重自报的名字不是它"
            );
            assert_eq!(
                GenerationWeights::NAMED[position],
                member.weights(),
                "NAMED 第 {position} 个不是 {member:?} 那一组"
            );
            let read_back = HistorySampling::from_environment(
                RANDOM_HISTORY_VARIABLE_PREFIX,
                &|variable: &str| {
                    (variable == "SINGLEFS_RANDOM_HISTORY_WEIGHTS")
                        .then(|| member.environment_value().to_string())
                },
                &defaults,
            )
            .map(|sampling| sampling.weights);
            assert_eq!(
                read_back,
                Ok(member.weights()),
                "WEIGHTS={} 读回的不是 {member:?} 那一组",
                member.environment_value()
            );
        }
    }

    /// 照 [`run_history_campaign`] 汇总的一条新发现（造一个假发现，不必真红）打出的复现行，原样喂回复现入口
    /// （[`random_history_to_reproduce_from_environment`]，「收缩一个种子」那条入口读的就是它），读回的种子与取样参数就是这一批的，
    /// 重新生成的历史与那一段逐步相同。取样参数的每一项都取与入口默认（快档那一种）不同的值（checker 只剩一种取值，那一项取不出不同的）：哪一项在汇总时装错了，读回就不同
    /// （代码三方 test-script-align-code-r1 判决 C607：这一处装配原先没有用例经过）。
    #[test]
    fn the_reproduction_line_of_a_campaign_finding_regenerates_the_history_the_campaign_ran() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let sampling = HistorySampling {
            operations_per_history: 17,
            weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
            execution: HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
                space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
            },
        };
        let first_seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
        let finding_history = sampling.generate(HistorySeed(first_seed + 1));
        let observation = FailureObservation {
            position: StepPosition::Operation(3),
            operation_kind: Some(finding_history.operations[3].kind()),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let run_ending = |ending: HistoryEnding| HistoryRun {
            ending,
            outcomes: Vec::new(),
            tally: HistoryTally::default(),
            operations_written_by_make_filesystem: 0,
            committed_versions_at_the_end: Vec::new(),
            earliest_stream_index_after_each_power_loss: Vec::new(),
        };
        let finished = vec![
            (
                finding_history.clone(),
                run_ending(HistoryEnding::NewFinding {
                    signature: FailureSignature::HarnessJudgement {
                        judgement: "自证用的假发现",
                    },
                    observation,
                }),
            ),
            (
                sampling.generate(HistorySeed(first_seed)),
                run_ending(HistoryEnding::Completed),
            ),
        ];
        let report = campaign_report_of_finished_histories(
            first_seed,
            2,
            &sampling,
            finished,
            1,
            FindingShrinking::ReportSeedsOnly,
        );
        assert_eq!(report.sampling, sampling, "报告带的取样参数不是这一批的");
        assert_eq!(report.new_findings.len(), 1, "假发现归成一类");
        let rendered = report.new_findings[0].render();
        let reproduction_lines: Vec<&str> = rendered
            .lines()
            .filter(|line| line.contains("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED="))
            .collect();
        assert_eq!(
            reproduction_lines.len(),
            1,
            "新发现里恰好一行复现命令：\n{rendered}"
        );
        let assignments = environment_assignments_in_line(reproduction_lines[0]);
        let (seed, reproduced_sampling) = random_history_to_reproduce_from_environment(
            &|variable: &str| assignments.get(variable).cloned(),
            &defaults,
        )
        .unwrap_or_else(|error| panic!("{}", error.render()));
        assert_eq!(
            (seed, reproduced_sampling),
            (finding_history.seed, sampling),
            "照复现行读回的种子与取样参数（步数、比重、checker、盘宽、空间准入）不是判红那一段的：{}",
            reproduction_lines[0]
        );
        assert_eq!(
            reproduced_sampling.generate(seed),
            finding_history,
            "照复现行重新生成的历史与判红那一段不同"
        );
    }

    /// [`run_history_campaign`] 报告带的取样参数就是调用方给的那一组，跑的就是照它生成的历史：一段 6 步的历史，计数与照取样参数
    /// 自己生成、自己跑的那一段逐项相同。判别力：同一个种子按快档的比重生成的是另一段历史，计数也不同（前提断言），
    /// 取样参数装成快档那一组时这里红。
    #[test]
    fn a_campaign_runs_and_reports_the_sampling_it_was_given() {
        let sampling = HistorySampling {
            operations_per_history: 6,
            weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
            execution: HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: HistoryDeviceWidth::FourGibibytes,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
        };
        let seed = HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE);
        let tally_of = |weights: &GenerationWeights| {
            execute_history_with(
                &generate_history_with_weights(seed, sampling.operations_per_history, weights),
                sampling.execution,
                &SharedStream::new(),
                &mut |_| {},
            )
            .tally
        };
        let tally_under_the_sampling = tally_of(&sampling.weights);
        assert_ne!(
            tally_under_the_sampling,
            tally_of(&GenerationWeights::BROAD),
            "前提：同一个种子按快档的比重跑出来的计数不同，下面那条断言才分得出跑的是哪一组"
        );
        let mut tally_of_a_one_history_campaign = HistoryTally::default();
        tally_of_a_one_history_campaign.absorb(&tally_under_the_sampling);
        let report = run_history_campaign(
            seed.0,
            1,
            sampling.operations_per_history,
            &sampling.weights,
            sampling.execution,
            1,
            FindingShrinking::ReportSeedsOnly,
        );
        assert_eq!(report.sampling, sampling, "报告带的取样参数不是调用方给的");
        assert_eq!(
            report.tally, tally_of_a_one_history_campaign,
            "跑出来的计数与照取样参数生成的那一段不同：跑的不是这一组取样参数下的历史"
        );
    }

    /// 复现入口认不得的取值报出变量名、给的值与认得的那几个，不悄悄退回默认。
    #[test]
    fn an_unrecognized_reproduction_value_is_refused_with_the_recognized_values() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let refused = HistorySampling::from_environment(
            RANDOM_HISTORY_VARIABLE_PREFIX,
            &|variable: &str| {
                (variable == "SINGLEFS_RANDOM_HISTORY_DEVICES").then(|| "small".to_string())
            },
            &defaults,
        );
        assert_eq!(
            refused,
            Err(ReproductionEnvironmentError::Unrecognized {
                variable: "SINGLEFS_RANDOM_HISTORY_DEVICES".to_string(),
                value: "small".to_string(),
                recognized: vec![
                    "4gib".to_string(),
                    "unit-area-384".to_string(),
                    "unit-area-256".to_string(),
                    "unit-area-240".to_string(),
                ],
            })
        );
    }

    /// 「收缩一个种子」那条入口里种子与每段步数前后都带空白：两个都去掉空白读回，与数值环境变量共用的那一份解析同一口径
    /// （同一个入口里一个数照收、另一个数停下，复现行手抄时多一个空格就分不清是哪个变量写错了）。
    #[test]
    fn blanks_around_the_seed_and_the_step_count_of_one_reproduction_entry_are_dropped_alike() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        let assignments = BTreeMap::from([
            (
                "SINGLEFS_RANDOM_HISTORY_SHRINK_SEED".to_string(),
                " 11".to_string(),
            ),
            (
                "SINGLEFS_RANDOM_HISTORY_OPERATIONS".to_string(),
                " 24".to_string(),
            ),
        ]);
        assert_eq!(
            random_history_to_reproduce_from_environment(
                &|variable: &str| assignments.get(variable).cloned(),
                &defaults,
            ),
            Ok((
                HistorySeed(11),
                HistorySampling {
                    operations_per_history: 24,
                    weights: GenerationWeights::BROAD,
                    execution:
                        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                }
            )),
            "种子与每段步数前后的空白都去掉读回"
        );
    }

    /// 每段步数写 0：复现入口报出变量名、给的值与认得的取值，不照收（每段 0 步的一批只跑了起点，照样判成绿）。
    #[test]
    fn a_reproduction_step_count_of_zero_is_refused_with_the_recognized_values() {
        let defaults = HistorySampling {
            operations_per_history: 30,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        };
        assert_eq!(
            HistorySampling::from_environment(
                RANDOM_HISTORY_VARIABLE_PREFIX,
                &|variable: &str| {
                    (variable == "SINGLEFS_RANDOM_HISTORY_OPERATIONS").then(|| "0".to_string())
                },
                &defaults,
            ),
            Err(ReproductionEnvironmentError::Unrecognized {
                variable: "SINGLEFS_RANDOM_HISTORY_OPERATIONS".to_string(),
                value: "0".to_string(),
                recognized: vec![COUNT_RECOGNIZED_IN_REPRODUCTION_ENVIRONMENT.to_string()],
            })
        );
    }

    /// 同一个种子、同一个步数生成的历史逐项相同；换一个种子就不同。
    #[test]
    fn the_same_seed_generates_the_same_history_and_another_seed_does_not() {
        let first = generate_history(HistorySeed(42), 40);
        assert_eq!(first, generate_history(HistorySeed(42), 40));
        assert_ne!(
            first.operations,
            generate_history(HistorySeed(43), 40).operations,
            "只比操作序列：历史里带着种子，整个比的话种子不同就恒不等"
        );
        assert_eq!(
            first.operations[..20],
            generate_history(HistorySeed(42), 20).operations[..],
            "步数只截尾，不改前面几步"
        );
    }

    /// 收缩在一个人造的失败判据上（冷启动之前有抬 F 或零单元发布就算失败）：序列是 [抬 F, 3 × 噪声, 零单元发布, 3 × 噪声, 冷启动]。
    /// 按次序删，先删掉的是前半（抬 F 连三条噪声），收到 [零单元发布, 冷启动]；按窗口并行试要收到同一个结果——窗口里若取的不是块号最小
    /// 的那个，会先删后半、收到 [抬 F, 冷启动]，两条路分得出来。
    #[test]
    fn shrinking_keeps_only_the_steps_the_failure_needs() {
        let noise = HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 7 },
            fill_seed: 9,
        });
        let raise = HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
            steps_above_current_floor: 5,
        });
        let operations: Vec<HistoryOperation> = std::iter::once(raise)
            .chain(std::iter::repeat_n(noise, 3))
            .chain(std::iter::once(HistoryOperation::PublishWithoutUnits))
            .chain(std::iter::repeat_n(noise, 3))
            .chain(std::iter::once(HistoryOperation::ColdStartRecover))
            .collect();
        let runs = AtomicU64::new(0);
        let still_fails = |candidate: &[HistoryOperation]| {
            runs.fetch_add(1, Ordering::Relaxed);
            let cold_start_at = candidate
                .iter()
                .position(|operation| operation.kind() == HistoryOperationKind::ColdStartRecover);
            cold_start_at.is_some_and(|cold_index| {
                candidate[..cold_index].iter().any(|operation| {
                    matches!(
                        operation.kind(),
                        HistoryOperationKind::RaiseRollbackFloor
                            | HistoryOperationKind::PublishWithoutUnits
                    )
                })
            })
        };
        let shrunk_one_at_a_time = shrink_operations(&operations, &still_fails, 1);
        assert_eq!(
            shrunk_one_at_a_time,
            vec![
                HistoryOperation::PublishWithoutUnits,
                HistoryOperation::ColdStartRecover
            ],
            "只剩零单元发布与冷启动两步"
        );
        let runs_one_at_a_time = runs.load(Ordering::Relaxed);
        assert!(
            runs_one_at_a_time > 2,
            "收缩真的试过删法：{runs_one_at_a_time} 次"
        );
        assert_eq!(
            shrink_operations(&operations, &still_fails, 7),
            shrunk_one_at_a_time,
            "按窗口并行试与一块一块按次序试，收到同一个结果"
        );
    }

    /// I-3.1 的违例文字读得出记账与遍历两个数；读不出（None、别的措辞）就不认。
    #[test]
    fn allocated_and_walked_bytes_are_read_from_the_checker_detail() {
        assert_eq!(
            allocated_and_walked_bytes(
                "盘 0：记账的已分配 Some(3031040)，遍历全部有效根得到 2850816"
            ),
            Some((3_031_040, 2_850_816))
        );
        assert_eq!(
            allocated_and_walked_bytes("盘 1：记账的已分配 None，遍历全部有效根得到 0"),
            None
        );
        assert_eq!(allocated_and_walked_bytes("别的措辞"), None);
    }

    fn record(slot: u64, span_slots: u16, generation: u64, is_released: bool) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(0),
            slot: singlefs_core::address::SlotNumber(slot),
            span_slots,
            generation: CheckpointTxg(generation),
            is_released,
        }
    }

    /// 一次发布（txg 12）改写或新增的记录，代都要是 12：新分配的、复用改写已回收的、这次释放的都对就一条都不报；复用改写时代没改
    /// （还写着释放代 5，代码三方第一轮攻方的 N2）就报那一条。没动过的记录（代 3、代 0）不看。
    #[test]
    fn record_rewritten_by_a_publish_that_keeps_its_old_generation_is_reported() {
        let before = vec![
            record(50176, 2, 0, false),
            record(50178, 1, 5, true),
            record(50180, 2, 3, false),
            record(50182, 1, 11, false),
        ];
        let rewritten_and_new_and_released = vec![
            record(50176, 2, 0, false),
            record(50178, 1, 12, false),
            record(50180, 2, 3, false),
            record(50182, 1, 12, true),
            record(50184, 2, 12, false),
        ];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &rewritten_and_new_and_released,
                CheckpointTxg(12),
                CheckpointTxg(12)
            ),
            Vec::new(),
            "新分配、复用改写、释放的代都是这次的 txg"
        );
        let rewritten_without_the_generation = vec![
            record(50176, 2, 0, false),
            record(50178, 1, 5, false),
            record(50180, 2, 3, false),
            record(50182, 1, 12, true),
            record(50184, 2, 12, false),
        ];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &rewritten_without_the_generation,
                CheckpointTxg(12),
                CheckpointTxg(12)
            ),
            vec![record(50178, 1, 5, false)],
            "复用改写的那一条还写着释放代 5"
        );
        assert!(
            allocation_generation_judgement(
                &before,
                &rewritten_without_the_generation,
                CheckpointTxg(12),
                CheckpointTxg(12)
            )
            .is_some(),
            "报出来就是执行器判的失败"
        );
    }

    /// 冷启动读回报错是执行器判的失败；读回文件、没有文件、别的结局都不是。
    #[test]
    fn failed_cold_start_read_back_is_a_harness_judgement_and_the_other_read_backs_are_not() {
        let recovered = |read_back: ColdStartReadBack| {
            StepOutcome::Applied(AppliedEffect::Recovered {
                outcome: format!("{read_back:?}"),
                read_back,
            })
        };
        assert_eq!(
            harness_judgement_of_outcome(&recovered(ColdStartReadBack::Failed)),
            Some(
                HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage {
                    outcome: "Failed".to_string()
                }
            )
        );
        assert_eq!(
            harness_judgement_of_outcome(&recovered(ColdStartReadBack::FileRead)),
            None
        );
        assert_eq!(
            harness_judgement_of_outcome(&recovered(ColdStartReadBack::NoFile)),
            None
        );
        assert_eq!(
            harness_judgement_of_outcome(&StepOutcome::Refused {
                member: "RollbackError::TargetNotACandidate(NotInRing)".to_string()
            }),
            None
        );
    }

    /// checker 在 I-3.1 的说明文字里带的那一段机理标识（`singlefs-checker` 的 `walk.rs` 写的格式）。
    fn allocation_statistic_detail(
        newest_root_txg: u64,
        oldest_readable_root_txg: u64,
        rollback_floor: u64,
        root_slots_dropped_below_floor: u64,
    ) -> String {
        format!(
            "盘 0：记账的已分配 Some(3817472)，遍历全部有效根得到 3801088；机理：根环槽数 24、最新根 txg {newest_root_txg}、环里自证过的根槽 24 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 20 个、被实例表判抛弃的根槽 0 个、回退下界 F {rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
        )
    }

    /// 收口表第 43 行那一形的失败观察：抬 F 那一步之后只有 I-3.1 红、记账多算，F 落在回退留下的空档里、环没转圈，
    /// checker 报的机理是「F 把环里读得到的根挡在了遍历之外」（`KNOWN_RED_FORMS` 那一条要的全都有）。
    fn raised_into_the_gap_observation() -> FailureObservation {
        FailureObservation {
            position: StepPosition::Operation(10),
            operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
            violations: vec![("I-3.1", allocation_statistic_detail(9, 0, 5, 4))],
            panic: None,
            newest_ring_root_txg: Some(9),
            root_ring_slot_count: Some(24),
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: Some(true),
            record_check: RecordCheck::default(),
        }
    }

    /// 执行器判出的失败不进「已知红」：哪怕同一步 checker 判的正是清单那一形，签名取执行器判的那一种。
    #[test]
    fn harness_judgement_is_never_classified_as_a_known_red_form() {
        let mut observation = raised_into_the_gap_observation();
        observation.harness_judgement =
            Some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
                records: vec![record(50178, 1, 5, false)],
                first_publish_txg: CheckpointTxg(10),
                last_publish_txg: CheckpointTxg(10),
            });
        let HistoryEnding::NewFinding { signature, .. } = classify_failure(observation.clone())
        else {
            panic!("要是新发现");
        };
        assert_eq!(
            signature,
            FailureSignature::HarnessJudgement {
                judgement: "改写或新增的分配记录的代不是这次发布的 txg"
            }
        );
        let mut without_the_judgement = observation;
        without_the_judgement.harness_judgement = None;
        assert!(
            matches!(
                classify_failure(without_the_judgement),
                HistoryEnding::NewFinding {
                    signature: FailureSignature::CheckerViolations { .. },
                    ..
                }
            ),
            "去掉执行器判的那一条，签名是 checker 判红的那几条；收口表第 43 行那一形已从清单里删了，照新发现报"
        );
    }

    /// 清单删空之后（收口表第 ② 行、第 43 行两形都修好、都删了），只有 I-3.1 红、记账多于遍历的观察不论 checker 报的机理是哪一种、
    /// F 落没落在被抛弃实例留下的空档里，一律照新发现报。
    #[test]
    fn with_the_known_red_list_empty_an_allocated_statistic_above_walked_is_a_new_finding_whatever_its_mechanism(
    ) {
        assert!(KNOWN_RED_FORMS.is_empty(), "清单是空的");
        let raised_into_the_gap = raised_into_the_gap_observation();
        let mut the_ring_also_turned = raised_into_the_gap.clone();
        the_ring_also_turned.violations = vec![("I-3.1", allocation_statistic_detail(27, 4, 8, 4))];
        the_ring_also_turned.newest_ring_root_txg = Some(27);
        let mut later_step = raised_into_the_gap.clone();
        later_step.operation_kind = Some(HistoryOperationKind::PublishOverwrite);
        for (observation, what) in [
            (
                raised_into_the_gap,
                "F 落在空档里、环没转圈、机理是 F 挡掉了读得到的根",
            ),
            (the_ring_also_turned, "环也转过一圈、缺口出在 F 上"),
            (later_step, "抬 F 之后的别的步上同一机理"),
        ] {
            assert!(
                matches!(
                    classify_failure(observation),
                    HistoryEnding::NewFinding {
                        signature: FailureSignature::CheckerViolations { .. },
                        ..
                    }
                ),
                "{what}：照新发现报"
            );
        }
    }

    /// 记录核对器判红的不接进「已知红」：签名是 `RecordCheck`，照新发现报。
    #[test]
    fn record_check_violations_are_never_absorbed_into_known_red_forms() {
        let mut observation = raised_into_the_gap_observation();
        observation.record_check = RecordCheck {
            root_without_record: true,
            claimed_state_missing_unit: false,
        };
        let HistoryEnding::NewFinding { signature, .. } = classify_failure(observation.clone())
        else {
            panic!("记录核对器判红了，要是新发现");
        };
        assert_eq!(
            signature,
            FailureSignature::RecordCheck {
                aspects: vec!["root_without_record"]
            }
        );
        observation.record_check = RecordCheck::default();
        assert!(
            matches!(
                classify_failure(observation),
                HistoryEnding::NewFinding {
                    signature: FailureSignature::CheckerViolations { .. },
                    ..
                }
            ),
            "去掉记录核对器那一条，签名是 checker 判红的那几条（收口表第 43 行那一形已从清单里删了）"
        );
    }
}
