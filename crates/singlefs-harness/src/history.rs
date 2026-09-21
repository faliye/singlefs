//! 随机历史（里程碑「第二个事务」增补 3 第 1 件）：按种子生成一段操作序列，操作只调 `crates/` 今天的公开入口；每一步之后对镜像跑
//! 池级 checker。入口返回 `Err` 算合法结局；panic 与 checker 违例算失败——撞到「已知红」清单（`KNOWN_RED_FORMS`）里的形态照记、
//! 这段历史到此为止，清单外的算新发现，收缩到最短复现（`shrink_operations`）。
//! 第 2 件接上了理想模型（`crate::model`）：每一步调入口之前问模型该成、该拒还是区间里都行，调完拿实现的结局与它比（胶水在
//! `crate::model_comparison`）——冷启动读回的内容、回退与抬 F 该不该被拒、根的身份与 F、单元的分配代都由它判，对不上算失败。
//!
//! 生成与执行分开：一步操作只带「做什么 + 选择子」，写多长、回退到哪条根、F 抬到多少，执行时按那一刻的盘面与会话现解——
//! 删掉前面几步之后，后面的操作照样有意义，收缩靠的就是这一条。
//!
//! 第 3 至 5 件在这里生成的历史上做：录制流由调用方给（`execute_history_observing`；崩溃注入给开了内容保留的流，截断点从流里取），
//! 每一步之后的镜像经观察者交出去（坏盘输入拿它当合法镜像）；故障注入要在录制器与内存盘之间再包一层，届时把 `HistoryDevice`
//! 换成泛型——今天只有一种盘，按编码纪律「只有一个实现的 trait 不抽」不先抽。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, Once};

use singlefs_checker::image::{
    chosen_system_configurations, valid_roots, ImageReader, InvariantVerdict,
};
use singlefs_checker::walk::{allocation_record_count_under_root, check_pool_image};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{
    AllocationRecord, DeviceFreeMap, Placement, PlacementRefusal, PoolAllocator,
};
use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    instance_table_of_root, readable_roots, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_without_units, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError, ZeroUnitPublishPlan,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT};

use crate::crash::{MemoryPool, RecordCheck, SparseBlockDevice};
use crate::model::{
    IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
    ModelJudgementCounts, ModelPoolGeometry, ModelRefusalReason, ModelRootKey, ObservedEffect,
    ObservedOutcome, ObservedRefusalReason,
};
use crate::model_comparison::{
    model_root_key, observed_mount, observed_read_back, observed_root_of_file_version,
    observed_root_of_version_without_file, refusal_reason_of_block_device_error,
    refusal_reason_of_mount_error, refusal_reason_of_publish_error,
    reported_ceiling_of_mount_error,
};
use crate::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use crate::{RecordingBlockDevice, SharedStream};

/// 历史里两块内存盘多宽（取样点的参数；两块恒等大：模型的几何只有一个盘大小，`ModelPoolGeometry::device_size_in_bytes`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryDeviceWidth {
    /// 4 GiB，与 `tests/common` 的文件镜像同宽：单元区 211968 槽，几百步的历史走不到单元区墙。快档、前三个取样点与大档用它。
    FourGibibytes,
    /// 单元区 384 槽的小盘：第一个文件之后每盘占 13 槽、一次覆盖写再占 10 槽，回收之前几十次发布就写满，分配器的落点拒绝
    /// （每块盘上都没有合政策的落点）在一段历史里走得到（增补 3 第 2 件代码三方第二轮判决第三节第 2 条，照攻方副本的做法：
    /// 4 GiB 的盘上四段历史里落点拒绝一次都没有，映射把它报成什么都没人看得见）。journal 环缩到 128 MiB：mkfs 要求环不超过设备容量的
    /// 四分之一（`make_filesystem::check_geometry`），默认 768 MiB 的环放不进这块盘。
    UnitAreaOf384Slots,
}

/// 小盘的单元区槽数（`HistoryDeviceWidth::UnitAreaOf384Slots`）：六个 64 槽的聚簇段。
const SMALL_DEVICE_UNIT_AREA_SLOTS: u64 = 384;

/// 小盘上的 journal 环字节数：不超过设备容量的四分之一（设备约 790 MiB）。
const SMALL_DEVICE_JOURNAL_RING_BYTES: u64 = 128 << 20;

impl HistoryDeviceWidth {
    /// 每块盘的字节数。
    #[must_use]
    pub fn device_bytes(self) -> u64 {
        match self {
            HistoryDeviceWidth::FourGibibytes => 4 << 30,
            HistoryDeviceWidth::UnitAreaOf384Slots => {
                (UNIT_AREA_START_SLOT + SMALL_DEVICE_UNIT_AREA_SLOTS) * SLOT_BYTES
            }
        }
    }

    /// mkfs 与发布的参数：E142 装置那一份（物理块 512、io_min 512，与各步用例相同），小盘只把 journal 环缩小。
    #[must_use]
    pub fn parameters(self) -> MakeFilesystemParameters {
        let mut parameters = e142_parameters(512, 512);
        match self {
            HistoryDeviceWidth::FourGibibytes => {}
            HistoryDeviceWidth::UnitAreaOf384Slots => {
                parameters.geometry.journal_ring_bytes = SMALL_DEVICE_JOURNAL_RING_BYTES;
            }
        }
        parameters
    }

    /// 报告里的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            HistoryDeviceWidth::FourGibibytes => "两块 4 GiB 的盘",
            HistoryDeviceWidth::UnitAreaOf384Slots => {
                "两块单元区 384 槽的小盘（journal 环 128 MiB）"
            }
        }
    }
}

/// 历史里的一块盘：内存盘外面包录制器，写与屏障进调用方给的那条流。
pub type HistoryDevice = RecordingBlockDevice<SparseBlockDevice>;

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
    /// 第一版的抽法：五分之二取最近四条、五分之二在整个根环里均匀取、五分之一取根环之外。快档、大档与第 121 行的取样点用它，
    /// 那几档每个种子生成的历史因此与第一版逐项相同。
    RecentUniformOrBeyond,
    /// 先抽一次：一半取候选集的下沿（`RingRootAtTheNewestFloor`），另一半照第一版的抽法。
    FloorRootHalfTheTime,
}

/// 抬 F 的目标怎么取。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FloorTargetChoice {
    /// 新 F = 现行版本根记录的 F + `steps_above_current_floor mod (现行 txg − 现行 F + 3)`：最大到现行 txg + 2，
    /// 而上限（D16（发布语义） 已定项 1）不超过每块盘上最新的有效根 ≤ 现行 txg，所以超过上限的目标一定取得到。
    /// 生成器的前提：目标只取现行 F 及以上，不往下抬。没有条款明写，按入口的文档注释取的读法——
    /// `crates/singlefs-core/src/mount.rs` 第 590 行「抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）」，已定项 1 在
    /// `.claude/kb/decisions/16-发布语义.md` 第 107 行；往下的目标这个入口接不接、接了算不算合法没有条款（F 回落在增补 2 收口表第 ② 行里打回重议）。
    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着，2026-09-18 主 agent 定）。
    pub steps_above_current_floor: u64,
}

/// 历史里的一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HistoryOperation {
    /// `transaction::publish_first_file`，接在现行版本后面（上一条记录取现行版本的；照抄实例表指针的那条根也取现行版本的）。
    PublishFirstFile(ContentChoice),
    /// `transaction::publish_overwrite`，接在现行的带文件的版本后面。
    PublishOverwrite(ContentChoice),
    /// `transaction::publish_without_units`，接在现行的树表 0 条的版本后面。
    PublishWithoutUnits,
    /// 关掉这个进程的可写会话，`mount::mount_writable`。
    CloseAndMountWritable,
    /// 关掉这个进程的可写会话，`mount::mount_rollback`（影子账开着）。
    CloseAndMountRollback(RollbackTargetChoice),
    /// `mount::raise_rollback_floor`（影子账开着），接在现行的带文件的版本上。
    RaiseRollbackFloor(FloorTargetChoice),
    /// 关掉这个进程的可写会话，冷启动 `recovery::recover`（看 journal）。
    ColdStartRecover,
}

/// 操作的种类：统计与比重用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistoryOperationKind {
    PublishFirstFile,
    PublishOverwrite,
    PublishWithoutUnits,
    CloseAndMountWritable,
    CloseAndMountRollback,
    RaiseRollbackFloor,
    ColdStartRecover,
}

impl HistoryOperationKind {
    pub const ALL: [HistoryOperationKind; 7] = [
        HistoryOperationKind::PublishFirstFile,
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::PublishWithoutUnits,
        HistoryOperationKind::CloseAndMountWritable,
        HistoryOperationKind::CloseAndMountRollback,
        HistoryOperationKind::RaiseRollbackFloor,
        HistoryOperationKind::ColdStartRecover,
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
            HistoryOperation::CloseAndMountRollback(_) => {
                HistoryOperationKind::CloseAndMountRollback
            }
            HistoryOperation::RaiseRollbackFloor(_) => HistoryOperationKind::RaiseRollbackFloor,
            HistoryOperation::ColdStartRecover => HistoryOperationKind::ColdStartRecover,
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
}

/// 生成一段历史时各样东西的比重（每张表写份数；操作那三张按百分比写，和为 100）：起点两种各占几份，三种会话估计下各类操作各占几份。
/// 抽法是 [0, 份数之和) 里取一个数、按表的次序落到哪一格：起点那张写成 1 : 1 时就是第一版的「种子的第一个数对 2 取余」，
/// 快档与大档的每个种子生成的历史与第一版逐项相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationWeights {
    /// 报告里的名字。
    pub name: &'static str,
    pub starting_points: &'static [(HistoryStartingPoint, u64)],
    pub with_session_closed: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_without_file: &'static [(HistoryOperationKind, u64)],
    pub with_session_open_with_file: &'static [(HistoryOperationKind, u64)],
    pub rollback_targets: RollbackTargetDraw,
}

impl GenerationWeights {
    /// 快档与大档用的那一组。关着的会话只抽挂载与冷启动；树表 0 条的版本上多抽第一个文件，不然从 mkfs 起的历史多半先推零单元发布、
    /// 再也接不上第一个文件；带文件的版本上零单元发布只会记「前提不满足」，少抽。起点两种各半。
    pub const BROAD: GenerationWeights = GenerationWeights {
        name: "各类操作都抽（快档、大档）",
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 1),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 70),
            (HistoryOperationKind::CloseAndMountRollback, 22),
            (HistoryOperationKind::ColdStartRecover, 8),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 60),
            (HistoryOperationKind::PublishWithoutUnits, 20),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::CloseAndMountRollback, 4),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishOverwrite, 3),
            (HistoryOperationKind::RaiseRollbackFloor, 3),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 52),
            (HistoryOperationKind::RaiseRollbackFloor, 16),
            (HistoryOperationKind::CloseAndMountWritable, 12),
            (HistoryOperationKind::CloseAndMountRollback, 10),
            (HistoryOperationKind::ColdStartRecover, 4),
            (HistoryOperationKind::PublishFirstFile, 4),
            (HistoryOperationKind::PublishWithoutUnits, 2),
        ],
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    };

    /// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：多从第一个文件起、多覆盖写、多抬 F、多可写挂载，
    /// 少回退与冷启动。罩住别的已回收记录要：抬 F 回收一批 1 槽的固定点单元，重开之后上一次挂载的聚簇段不再挡用户数据，
    /// 下一个 2 槽的数据单元落在两条相邻的已回收记录上。
    pub const REUSE_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
        name: "偏向抬 F 之后的复用（第 121 行那一类的取样点）",
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 90),
            (HistoryOperationKind::CloseAndMountRollback, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 85),
            (HistoryOperationKind::PublishWithoutUnits, 5),
            (HistoryOperationKind::CloseAndMountWritable, 4),
            (HistoryOperationKind::CloseAndMountRollback, 2),
            (HistoryOperationKind::ColdStartRecover, 2),
            (HistoryOperationKind::PublishOverwrite, 1),
            (HistoryOperationKind::RaiseRollbackFloor, 1),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 55),
            (HistoryOperationKind::RaiseRollbackFloor, 22),
            (HistoryOperationKind::CloseAndMountWritable, 18),
            (HistoryOperationKind::CloseAndMountRollback, 2),
            (HistoryOperationKind::ColdStartRecover, 1),
            (HistoryOperationKind::PublishFirstFile, 1),
            (HistoryOperationKind::PublishWithoutUnits, 1),
        ],
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    };

    /// 理想模型那一格（B2：回退到 txg = F_生效 的根被拒）的专门取样点：多从第一个文件起、多覆盖写（攒非空根，抬 F 的上限才抬得动）、
    /// 多抬 F、多回退，少冷启动；回退的目标一半取候选集的下沿（`RollbackTargetDraw::FloorRootHalfTheTime`）。快档那组比重与抽法下
    /// 回退落到 F 那条根上一次都没有（2026-09-19 数：快档 96 段、第 121 行取样点 48 段都是 0 次；只换比重、不换抽法，192 段里 2 次）。
    pub const ROLLBACK_AFTER_RAISING_THE_FLOOR: GenerationWeights = GenerationWeights {
        name: "偏向抬 F 之后回退（B2 那一格的取样点）",
        starting_points: &[
            (HistoryStartingPoint::AfterMakeFilesystem, 1),
            (HistoryStartingPoint::AfterFirstFile, 9),
        ],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 50),
            (HistoryOperationKind::CloseAndMountRollback, 45),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 85),
            (HistoryOperationKind::PublishWithoutUnits, 3),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::CloseAndMountRollback, 2),
            (HistoryOperationKind::ColdStartRecover, 2),
            (HistoryOperationKind::PublishOverwrite, 1),
            (HistoryOperationKind::RaiseRollbackFloor, 1),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 40),
            (HistoryOperationKind::RaiseRollbackFloor, 25),
            (HistoryOperationKind::CloseAndMountRollback, 25),
            (HistoryOperationKind::CloseAndMountWritable, 7),
            (HistoryOperationKind::ColdStartRecover, 1),
            (HistoryOperationKind::PublishFirstFile, 1),
            (HistoryOperationKind::PublishWithoutUnits, 1),
        ],
        rollback_targets: RollbackTargetDraw::FloorRootHalfTheTime,
    };

    /// 分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：一律从第一个文件起，带文件的版本上多半覆盖写
    /// （每次每盘加 8 条），夹着可写挂载（写行与暖机每次加 18 或 26 条）、抬 F（回收之后复用改写记录，条数涨得慢）与少量回退，
    /// 让逼近 812 条时的条数落在不同的余数上——墙的「差一」只在某次准入之后正好 812 条时分得出。配
    /// `PerStepChecker::RunContinuingPastTheRingTurnForm` 跑（第二轮判决第三节第 3 条；第一轮配的是 `Skipped`）。
    pub const TOWARD_THE_ALLOCATION_RECORD_WALL: GenerationWeights = GenerationWeights {
        name: "逼近分配记录墙（812 条那一格的取样点）",
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 95),
            (HistoryOperationKind::CloseAndMountRollback, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 80),
            (HistoryOperationKind::CloseAndMountWritable, 12),
            (HistoryOperationKind::RaiseRollbackFloor, 5),
            (HistoryOperationKind::CloseAndMountRollback, 2),
            (HistoryOperationKind::ColdStartRecover, 1),
        ],
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    };

    /// 单元区墙那一格（增补 3 第 2 件代码三方第二轮判决第三节第 2 条）的取样点，配 `HistoryDeviceWidth::UnitAreaOf384Slots` 跑：
    /// 一律从第一个文件起，带文件的版本上绝大多数是覆盖写（每次每盘占 10 槽，回收之前三十几次写满单元区），夹着少量可写挂载与抬 F
    /// （回收之后腾出落点，写满、拒、再写满）与回退——落点拒绝落在用户数据那一处（覆盖写的数据单元）与提交内生块那一处（挂载、抬 F 的
    /// 固定点单元）都走得到。
    pub const TOWARD_THE_UNIT_AREA_WALL: GenerationWeights = GenerationWeights {
        name: "逼近单元区墙（小盘上落点拒绝那一格的取样点）",
        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
        with_session_closed: &[
            (HistoryOperationKind::CloseAndMountWritable, 90),
            (HistoryOperationKind::CloseAndMountRollback, 5),
            (HistoryOperationKind::ColdStartRecover, 5),
        ],
        with_session_open_without_file: &[
            (HistoryOperationKind::PublishFirstFile, 90),
            (HistoryOperationKind::CloseAndMountWritable, 10),
        ],
        with_session_open_with_file: &[
            (HistoryOperationKind::PublishOverwrite, 88),
            (HistoryOperationKind::CloseAndMountWritable, 6),
            (HistoryOperationKind::RaiseRollbackFloor, 4),
            (HistoryOperationKind::CloseAndMountRollback, 1),
            (HistoryOperationKind::ColdStartRecover, 1),
        ],
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    };

    fn for_session(&self, expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {
        match expected {
            ExpectedSession::Closed => self.with_session_closed,
            ExpectedSession::OpenWithoutFile => self.with_session_open_without_file,
            ExpectedSession::OpenWithFile => self.with_session_open_with_file,
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

fn draw_operation(
    source: &mut SeededRandomSource,
    kind: HistoryOperationKind,
    rollback_targets: RollbackTargetDraw,
) -> HistoryOperation {
    match kind {
        HistoryOperationKind::PublishFirstFile => {
            HistoryOperation::PublishFirstFile(draw_content(source))
        }
        HistoryOperationKind::PublishOverwrite => {
            HistoryOperation::PublishOverwrite(draw_content(source))
        }
        HistoryOperationKind::PublishWithoutUnits => HistoryOperation::PublishWithoutUnits,
        HistoryOperationKind::CloseAndMountWritable => HistoryOperation::CloseAndMountWritable,
        HistoryOperationKind::CloseAndMountRollback => {
            let takes_the_floor_root = match rollback_targets {
                RollbackTargetDraw::RecentUniformOrBeyond => false,
                RollbackTargetDraw::FloorRootHalfTheTime => source.below(2) == 0,
            };
            if takes_the_floor_root {
                return HistoryOperation::CloseAndMountRollback(
                    RollbackTargetChoice::RingRootAtTheNewestFloor,
                );
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
            HistoryOperation::CloseAndMountRollback(target)
        }
        HistoryOperationKind::RaiseRollbackFloor => {
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: source.next_word(),
            })
        }
        HistoryOperationKind::ColdStartRecover => HistoryOperation::ColdStartRecover,
    }
}

fn expected_session_after(
    expected: ExpectedSession,
    has_file_expected: bool,
    kind: HistoryOperationKind,
) -> ExpectedSession {
    match kind {
        HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::CloseAndMountRollback => {
            if has_file_expected {
                ExpectedSession::OpenWithFile
            } else {
                ExpectedSession::OpenWithoutFile
            }
        }
        HistoryOperationKind::ColdStartRecover => ExpectedSession::Closed,
        HistoryOperationKind::PublishFirstFile => match expected {
            ExpectedSession::OpenWithoutFile => ExpectedSession::OpenWithFile,
            ExpectedSession::Closed | ExpectedSession::OpenWithFile => expected,
        },
        HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::RaiseRollbackFloor => expected,
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
    let mut has_file_expected = starting_point == HistoryStartingPoint::AfterFirstFile;
    let mut operations = Vec::with_capacity(operation_count);
    for _ in 0..operation_count {
        let kind = draw_weighted(&mut source, weights.for_session(expected));
        operations.push(draw_operation(&mut source, kind, weights.rollback_targets));
        expected = expected_session_after(expected, has_file_expected, kind);
        has_file_expected = has_file_expected || expected == ExpectedSession::OpenWithFile;
    }
    GeneratedHistory {
        seed,
        starting_point,
        operations,
    }
}

/// 这个进程里开着的可写会话：分配器、现行版本、实例代号，都是挂载（或起点那个进程）交回的。
struct WritableSession {
    allocator: PoolAllocator,
    current: PoolVersion,
    instance: InstanceGeneration,
    /// 这次挂载（或起点那个进程）里写出了几条根：写行、暖机、每次发布、抬 F 的每次空发布都算。
    publishes_in_this_mount: usize,
}

/// 一段历史跑到哪了：两块盘（与它们的宽度）、这个进程的会话、挂载的次数、理想模型、录制流（判「拒绝之前写没写盘」）。
struct HistoryPool {
    device_width: HistoryDeviceWidth,
    devices: Vec<(DeviceIdentity, HistoryDevice)>,
    session: Option<WritableSession>,
    successful_mounts: usize,
    mount_attempts: usize,
    model: IdealModel,
    stream: SharedStream,
    /// mkfs 写完时录制流里有几步（崩溃注入的基线从这里起）。
    operations_written_by_make_filesystem: usize,
}

/// 模型对一步的判定：对不上的那一格（没有就是 None）与这一步比了多少格。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelVerdict {
    pub disagreement: Option<ModelDisagreement>,
    pub counts: ModelJudgementCounts,
}

/// 拿实现的结局与模型的答案比，对得上就让模型往前走（模型答不了的，答案本身就是那一格对不上）。
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

/// 入口返回 Err 时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障；理由是分配记录墙时
/// 连同从镜像上数的准入基数（`allocation_records_counted_for_the_wall`）。
fn observed_refusal(
    member: String,
    reason: ObservedRefusalReason,
    publishes_completed: usize,
    stream_length_before: usize,
    stream: &SharedStream,
    reported_ceiling: Option<ModelCheckpointTxg>,
    allocation_records_counted_on_the_image: Option<u64>,
) -> ObservedOutcome {
    ObservedOutcome::Refused {
        member,
        reason,
        publishes_completed,
        wrote_anything: stream.operation_count() != stream_length_before,
        reported_ceiling,
        allocation_records_counted_on_the_image,
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
            .map(|(identity, device)| (*identity, device.inner().image.clone()))
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

/// 分配记录墙拒时，从镜像上现数准入的基数（增补 3 第 2 件代码三方第一轮判决第三节第 1 条：用 checker 的解析读镜像，不用分配器的状态）：
/// 模型点名的那一版下有几条分配记录。理由不是分配记录墙的不数；模型答不了这一步（答案本身就是对不上的那一格）也不数。
fn allocation_records_counted_for_the_wall(
    reason: ObservedRefusalReason,
    answer: Option<&ModelAnswer>,
    publishes_completed: usize,
    devices: &[(DeviceIdentity, HistoryDevice)],
    device_width: HistoryDeviceWidth,
) -> Option<u64> {
    let ObservedRefusalReason::Explained(ModelRefusalReason::AllocationRecordNodeWall) = reason
    else {
        return None;
    };
    let root = answer?.root_whose_allocation_records_the_wall_counts(publishes_completed)?;
    allocation_records_on_the_image_under(&image_of(devices, device_width), root)
}

impl HistoryPool {
    /// 起点：mkfs（加上同一个进程里的取号、暖机、第一个文件）。模型跟着走一遍，第一个文件那次发布拿实现的输出与模型比，判定一并交回。
    fn start(
        starting_point: HistoryStartingPoint,
        device_width: HistoryDeviceWidth,
        stream: &SharedStream,
    ) -> (Self, Option<ModelVerdict>) {
        let parameters = device_width.parameters();
        let device_bytes = device_width.device_bytes();
        let mut devices: Vec<(DeviceIdentity, HistoryDevice)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                            stream.clone(),
                        ),
                    )
                })
                .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect(
            "两块全零、等大的内存盘上按 E142 参数 mkfs：两种宽度的几何都放得下（环不超过容量的四分之一），内存盘的写不报错",
        );
        let operations_written_by_make_filesystem = stream.operation_count();
        let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
            devices: devices
                .iter()
                .map(|(identity, _)| ModelDeviceIdentity(identity.0))
                .collect(),
            device_size_in_bytes: device_bytes,
        });
        let mut verdict = None;
        let session = match starting_point {
            HistoryStartingPoint::AfterMakeFilesystem => None,
            HistoryStartingPoint::AfterFirstFile => {
                let mut allocator = PoolAllocator::new(vec![
                    DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
                    DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
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
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                let instance = acquire_instance(&mut writer)
                    .expect("刚 mkfs 的池取号：内存盘的写与屏障不报错");
                let warmed = warm_up(&mut writer, &genesis.root, instance)
                    .expect("刚取号的池暖机：内存盘的写不报错");
                let content = first_file_content();
                let output = publish_first_file(
                    &mut writer,
                    &mut allocator,
                    &genesis.root,
                    FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                    },
                    instance,
                    &warmed.last_record_bytes,
                )
                .expect("暖机之后的第一个文件：与 build_pool 同一条路，各步用例都走过");
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
                    publishes_in_this_mount: 3,
                })
            }
        };
        (
            Self {
                device_width,
                devices,
                session,
                successful_mounts: 0,
                mount_attempts: 0,
                model,
                stream: stream.clone(),
                operations_written_by_make_filesystem,
            },
            verdict,
        )
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
    /// 现行版本带文件：零单元发布只给树表 0 条的一版；带文件的一版上的空发布要重写四个固定点单元，那是挂载内部的 `publish_version`，
    /// 不是这个入口。生成器的前提，没有条款明写调用约定，按入口的文档注释取的读法——`crates/singlefs-core/src/transaction.rs` 第 506 行
    /// 「零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）」，已定项 9 在 `.claude/kb/decisions/16-发布语义.md` 第 192 行。
    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着，2026-09-18 主 agent 定）。
    CurrentVersionWithFile,
    /// 按 checker 的读法，根环里一条可读根都没有：回退的目标无从取。
    NoReadableRootInRing,
}

impl MissingPrecondition {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            MissingPrecondition::NoWritableSession => "没有可写会话",
            MissingPrecondition::CurrentVersionWithoutFile => "现行版本树表 0 条",
            MissingPrecondition::CurrentVersionWithFile => "现行版本带文件",
            MissingPrecondition::NoReadableRootInRing => "根环里没有可读根",
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
/// 盘上 checker 判不出分配代没改（29 条不变量里只有 I-3.9 看代，只看已释放的；增补 2 收口表第 44 行），这里在执行器里判。
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
        }
    }
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
            | AppliedEffect::RaisedFloor { .. },
        )
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_) => None,
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
    },
    Mounted {
        instance: InstanceGeneration,
        publishes: usize,
        allocation_records_compared: MountAllocationComparison,
        /// 写行与暖机那几次发布里的复用（逐次相加）。
        reuse: RecordReuse,
    },
    RaisedFloor {
        new_floor: CheckpointTxg,
        publishes: usize,
        reclaimed_placements: usize,
        reuse: RecordReuse,
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
fn with_panic_capture<Output>(body: impl FnOnce() -> Output) -> Result<Output, CapturedPanic> {
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

impl AllocationStatisticMechanism {
    /// 环转过一圈把 F 之上的根挤出了环：环里最老的自证过的根已经高过 F，[F, 最老) 那一段的根读不到了，
    /// 而它们引用的单元还在当前账里。
    #[must_use]
    pub fn the_ring_turn_dropped_roots_above_the_floor(&self) -> bool {
        self.newest_root_txg >= self.root_ring_slot_count
            && self.oldest_readable_root_txg > self.rollback_floor
    }

    /// 回退下界把环里读得到的根挡在了遍历之外。
    #[must_use]
    pub fn the_floor_dropped_readable_roots(&self) -> bool {
        self.root_slots_dropped_below_floor >= 1
    }
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

/// 这次失败里 I-3.1 那几条（逐盘各一条）带的机理标识：取第一条读得出来的。
/// 一条 I-3.1 都没有、或那几条文字里都没有机理标识，都是 None——机理读不出来时「已知红」一条都不接。
#[must_use]
fn allocation_statistic_mechanism_of(
    observation: &FailureObservation,
) -> Option<AllocationStatisticMechanism> {
    observation
        .violations
        .iter()
        .filter(|(invariant, _)| *invariant == "I-3.1")
        .find_map(|(_, detail)| allocation_statistic_mechanism(detail))
}

/// 「已知红」清单的一条：形态与它在增补 2 收口表里的那一行。
pub struct KnownRedForm {
    /// 增补 2 收口表（`.claude/kb/milestone/02-second-txn.md`「增补 2」）里的哪一行。
    pub closeout_table_row: &'static str,
    /// 形态：哪条不变量或哪种 panic、在什么操作之后。
    pub shape: &'static str,
    pub matches: fn(&FailureObservation) -> bool,
}

/// 没有 panic、执行器没判出失败、模型没对不上、记录核对器也没判出、判红的只有 I-3.1、而且是记账的已分配大于遍历全部有效根得到的
/// （记账多算，不是少算）。
fn only_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.panic.is_none()
        && observation.harness_judgement.is_none()
        && observation.model_disagreement.is_none()
        && observation.record_check == RecordCheck::default()
        && !observation.violations.is_empty()
        && observation.violations.iter().all(|(invariant, detail)| {
            *invariant == "I-3.1"
                && allocated_and_walked_bytes(detail)
                    .is_some_and(|(allocated, walked)| allocated > walked)
        })
}

/// 第 0 条那一形：只有 I-3.1 红、记账多于遍历，而且 checker 自己报的机理是「环转过一圈把 F 之上的根挤出了环」
/// ——签名相同而机理不同（比如根一条都没掉出环、缺口出在别处）的，不接进这一条，照新发现报
/// （代码三方 m2-supp3-item3-code-r1 判决 K6 的假阴那一半，用户 2026-09-20 定案第 6 条）。
/// 执行器那一路自己从盘上读的「根环转过一圈」照样要成立：两份独立的读法都说转过了，才算这一形。
fn ring_turn_leaves_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.root_ring_has_turned()
        && only_allocated_statistic_above_walked(observation)
        && allocation_statistic_mechanism_of(observation)
            .is_some_and(|mechanism| mechanism.the_ring_turn_dropped_roots_above_the_floor())
}

/// F 落在回退留下的空档里（F 那个 txg 上的根全属于被抛弃的实例）、只有 I-3.1 红且记账多于遍历，
/// 而且 checker 报的机理是「回退下界把环里读得到的根挡在了遍历之外」
/// （代码三方第一轮判决第二节第 1 条收窄：此前不看空档，不经回退的抬 F 之后记账多算也被接走）。
///
/// 两条不再要求，都因为机理标识把它们替代掉了（用户 2026-09-20 定案第 6 条）：
/// 「这一步是抬 F」——活盘面那一路上 `raised_floor_lands_only_on_abandoned_roots` 只有抬 F 那一步算得出来（别的步恒 None），
/// 卡着操作种类只会把崩溃状态上同一机理的盘面判成新发现；
/// 「根环没转圈」——环转没转圈与这条机理正交。缺口是不是环转出来的，由第 0 条的机理（环里最老的自证过的根已经高过 F）判，
/// 不由「转没转过」判：F 之下的根本来就不走，它们掉出环一个字节都不差。两条机理同时成立时按清单次序落在第 0 条上。
fn raise_after_rollback_leaves_allocated_statistic_above_walked(
    observation: &FailureObservation,
) -> bool {
    observation.raised_floor_lands_only_on_abandoned_roots == Some(true)
        && only_allocated_statistic_above_walked(observation)
        && allocation_statistic_mechanism_of(observation)
            .is_some_and(|mechanism| mechanism.the_floor_dropped_readable_roots())
}

/// 「已知红」清单。修好一条就删一条，删掉之后那条的复现（`tests/second_transaction_supplement_three_random_history.rs` 里钉着）要转绿。
/// 第 0 条的宽度（转圈跨几次挂载也算）2026-09-18 主 agent 定案保留，记在收口表第 ② 行。
pub const KNOWN_RED_FORMS: [KnownRedForm; 2] = [
    KnownRedForm {
        closeout_table_row: "增补 2 收口表第 ② 行（一次挂载转过一整圈根环时 checker 在合法状态上判 I-3.1 红）",
        shape: "根环转过一圈之后（按 checker 的读法，盘上最新根的 txg ≥ R × S = 24）I-3.1（已分配统计对得上） 红、记账的已分配大于遍历全部有效根得到的、别的不变量都不红、没有 panic；不限哪一步操作之后（转圈可以跨几次挂载）",
        matches: ring_turn_leaves_allocated_statistic_above_walked,
    },
    KnownRedForm {
        closeout_table_row: "增补 2 收口表第 43 行",
        shape: "回退之后把 F 抬进回退目标根与新实例第一次发布之间（抬之前的镜像上，F 那个 txg 上的根全属于被抛弃的实例）：抬 F 那一步之后 I-3.1（已分配统计对得上） 红、记账的已分配大于遍历全部有效根得到的、别的不变量都不红、没有 panic、根环没转圈",
        matches: raise_after_rollback_leaves_allocated_statistic_above_walked,
    },
];

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
    pub most_successful_mounts_in_one_history: usize,
    pub most_mount_attempts_in_one_history: usize,
    pub raises_that_reclaimed: u64,
    pub reclaimed_placements: u64,
    pub records_rewritten_from_released: u64,
    pub records_rewritten_with_changed_span: u64,
    pub released_records_removed: u64,
    /// 已释放的记录被这次新分配或改写的记录罩住起点的次数（第 121 行那一类的路径跑到了；删没删不影响这个数）。
    pub released_records_covered: u64,
    /// 挂载写出的写行与暖机里，比过分配记录的发布次数；挂载之前生效根那棵分配记录树读不出、没比的挂载次数。
    pub mount_publishes_compared: u64,
    pub mounts_with_previous_records_unreadable: u64,
    pub highest_checkpoint_txg: u64,
    pub histories_that_turned_the_root_ring: u64,
    pub checker_runs: u64,
    /// 这一步一个写都没发（录制流一步没多）：镜像逐字节不变，checker 的结论沿用上一次，不重跑。
    pub checker_runs_skipped_because_nothing_was_written: u64,
    /// `PerStepChecker::RunContinuingPastTheRingTurnForm` 下 checker 判出「已知红」清单第 0 条那一形、只记不停的步数，与出现过它的历史段数。
    pub ring_turn_form_steps_noted: u64,
    pub histories_with_the_ring_turn_form_noted: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
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
        self.most_successful_mounts_in_one_history = self
            .most_successful_mounts_in_one_history
            .max(other.most_successful_mounts_in_one_history);
        self.most_mount_attempts_in_one_history = self
            .most_mount_attempts_in_one_history
            .max(other.most_mount_attempts_in_one_history);
        self.raises_that_reclaimed += other.raises_that_reclaimed;
        self.reclaimed_placements += other.reclaimed_placements;
        self.records_rewritten_from_released += other.records_rewritten_from_released;
        self.records_rewritten_with_changed_span += other.records_rewritten_with_changed_span;
        self.released_records_removed += other.released_records_removed;
        self.released_records_covered += other.released_records_covered;
        self.mount_publishes_compared += other.mount_publishes_compared;
        self.mounts_with_previous_records_unreadable +=
            other.mounts_with_previous_records_unreadable;
        self.highest_checkpoint_txg = self
            .highest_checkpoint_txg
            .max(other.highest_checkpoint_txg);
        self.histories_that_turned_the_root_ring += other.histories_that_turned_the_root_ring;
        self.checker_runs += other.checker_runs;
        self.checker_runs_skipped_because_nothing_was_written +=
            other.checker_runs_skipped_because_nothing_was_written;
        self.ring_turn_form_steps_noted += other.ring_turn_form_steps_noted;
        self.histories_with_the_ring_turn_form_noted +=
            other.histories_with_the_ring_turn_form_noted;
        for (invariant, count) in &other.invariant_holds {
            *self.invariant_holds.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
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
            StepOutcome::Applied(AppliedEffect::Published { reuse }) => self.note_reuse(*reuse),
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
            StepOutcome::Applied(AppliedEffect::Mounted {
                allocation_records_compared,
                reuse,
                ..
            }) => {
                self.note_reuse(*reuse);
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
            | HistoryOperation::CloseAndMountRollback(_)
            | HistoryOperation::RaiseRollbackFloor(_)
            | HistoryOperation::ColdStartRecover => None,
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
            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；根环转过一圈的 {} 段；最高 txg {}",
            self.histories,
            self.histories_completed,
            self.histories_ended_known_red,
            self.histories_ended_new_finding,
            self.histories_that_turned_the_root_ring,
            self.highest_checkpoint_txg
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
            "  一次挂载里最多写出 {} 条根；一段历史里最多挂载成功 {} 次、最多试挂载 {} 次",
            self.most_publishes_in_one_mount,
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
        let _ = writeln!(
            text,
            "  checker 跑了 {} 次；一个写都没发、沿用上一次结论的 {} 步；已知红第 0 条那一形只记不停 {} 步（{} 段历史）",
            self.checker_runs,
            self.checker_runs_skipped_because_nothing_was_written,
            self.ring_turn_form_steps_noted,
            self.histories_with_the_ring_turn_form_noted
        );
        let counts = &self.model_counts;
        let _ = writeln!(
            text,
            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次；分配记录墙按镜像上的真条数放行 {} 次；单元区墙按区间放行 {} 次",
            self.model_judged_steps,
            counts.required_refusals_matched,
            counts.permitted_refusals_taken,
            counts.successes_matched,
            counts.roots_compared,
            counts.allocation_records_compared,
            counts.cold_start_contents_compared,
            counts.ceilings_compared,
            counts.rollbacks_accepted_at_the_effective_floor,
            counts.allocation_record_wall_refusals_over_one_node,
            counts.unit_area_wall_refusals_in_the_interval
        );
        for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
            let _ = writeln!(
                text,
                "  {invariant}：判绿 {} 次、不适用 {} 次",
                self.invariant_holds.get(invariant).copied().unwrap_or(0),
                self.invariant_not_applicable
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
    /// mkfs 占了录制流开头的几步：崩溃注入（增补 3 第 3 件）的基线与层 0 一样取「mkfs 之后」——
    /// mkfs 不是事务，它写到一半的盘面上没有池，恢复报不出根是对的，不该拿事务的 oracle 去判
    /// （层 0 那一路同样从 `mkfs_operation_count` 之后起枚举，见 `tests/first_transaction_step_seven_layer0.rs`）。
    pub operations_written_by_make_filesystem: usize,
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
        PublishError::AllocationRecordsExceedOneNode { .. } => "AllocationRecordsExceedOneNode",
        PublishError::AccountingEntriesExceedOneNode { .. } => "AccountingEntriesExceedOneNode",
        PublishError::ReleaseNotInMapping { .. } => "ReleaseNotInMapping",
        PublishError::ReleaseTargetNotAllocated { .. } => "ReleaseTargetNotAllocated",
        PublishError::ReleaseTargetAlreadyReleased { .. } => "ReleaseTargetAlreadyReleased",
        PublishError::ReleaseSpanMismatch { .. } => "ReleaseSpanMismatch",
        PublishError::ContentExceedsDataUnit { .. } => "ContentExceedsDataUnit",
        PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp { .. } => {
            "FirstFileVersionNotRightAfterTheSecondWarmUp"
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
        RecoveryFailure::SystemConfigurationsDisagree => {
            "RecoveryFailure::SystemConfigurationsDisagree".to_string()
        }
        RecoveryFailure::NoValidRoot => "RecoveryFailure::NoValidRoot".to_string(),
        RecoveryFailure::UnitUnreadable { .. } => "RecoveryFailure::UnitUnreadable".to_string(),
        RecoveryFailure::UnitMalformed { what } => {
            format!("RecoveryFailure::UnitMalformed（{what}）")
        }
        RecoveryFailure::InvariantViolated { invariant, .. } => {
            format!("RecoveryFailure::InvariantViolated（{invariant}）")
        }
        RecoveryFailure::MappingMiss { .. } => "RecoveryFailure::MappingMiss".to_string(),
        RecoveryFailure::MappingStillUnreadable { .. } => {
            "RecoveryFailure::MappingStillUnreadable".to_string()
        }
    }
}

fn mount_error_member(error: &MountError) -> String {
    let member = match error {
        MountError::Recovery(failure) => {
            return format!("MountError::Recovery({})", recovery_failure_member(failure))
        }
        MountError::FileVersionWithoutAnyJournalRecord => "FileVersionWithoutAnyJournalRecord",
        MountError::InstanceTableMalformed => "InstanceTableMalformed",
        MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion => {
            "RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion"
        }
        MountError::Acquisition(acquisition) => {
            return format!(
                "MountError::Acquisition({})",
                block_device_error_member(&acquisition.cause)
            )
        }
        MountError::Publish(cause) => {
            return format!("MountError::Publish({})", publish_error_member(cause))
        }
        MountError::RollbackTargetNotACandidate { exclusion, .. } => {
            return format!("MountError::RollbackTargetNotACandidate({exclusion:?})")
        }
        MountError::RollbackToVersionWithoutFileUnsupported(_) => {
            "RollbackToVersionWithoutFileUnsupported"
        }
        MountError::RollbackFloorAboveCeiling { .. } => "RollbackFloorAboveCeiling",
        MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. } => {
            "InstanceRowsOnVersionWithoutFileUnsupported"
        }
        MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
            "VersionWithoutFileNotWrittenByMakeFilesystem"
        }
        MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { failure, .. } => {
            return format!(
                "MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable({})",
                recovery_failure_member(failure)
            )
        }
        MountError::InstanceGenerationChangedBeforeAcquisition { .. } => {
            "InstanceGenerationChangedBeforeAcquisition"
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
        MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported { .. } => {
            "InstanceTableRowsExceedOnePageSecondPageUnsupported"
        }
        MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction { .. } => {
            "FormattedPoolMountNotShapedLikeTheFirstTransaction"
        }
    };
    format!("MountError::{member}")
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
        device_width,
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
        Err(error) => settle_refused_file_publish(
            model,
            answer,
            &error,
            devices,
            *device_width,
            stream,
            stream_length_before,
        ),
    }
}

/// 第一个文件、覆盖写被拒：成员映射成理由、分配记录墙时从镜像上数准入基数，交模型比。
fn settle_refused_file_publish(
    model: &mut IdealModel,
    answer: Result<ModelAnswer, ModelDisagreement>,
    error: &PublishError,
    devices: &[(DeviceIdentity, HistoryDevice)],
    device_width: HistoryDeviceWidth,
    stream: &SharedStream,
    stream_length_before: usize,
) -> AppliedStep {
    let member = publish_error_member(error);
    let reason = refusal_reason_of_publish_error(error);
    let counted = allocation_records_counted_for_the_wall(
        reason,
        answer.as_ref().ok(),
        0,
        devices,
        device_width,
    );
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
            counted,
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
    session.publishes_in_this_mount += 1;
    AppliedStep {
        outcome: StepOutcome::Applied(AppliedEffect::Published { reuse }),
        harness_judgement,
        model_verdict: Some(verdict),
    }
}

fn apply_publish_overwrite(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        device_width,
        devices,
        session,
        model,
        stream,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let PoolVersion::WithFile(previous) = &session.current else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    let previous = previous.clone();
    let content = content_choice.bytes();
    let records_before = session.allocator.records().to_vec();
    let answer = model.answer_publish_overwrite(&content);
    let stream_length_before = stream.operation_count();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_overwrite(
        &mut writer,
        &mut session.allocator,
        &previous,
        FirstFile {
            content: &content,
            write_time_seconds,
        },
        session.instance,
    ) {
        Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
        Err(error) => settle_refused_file_publish(
            model,
            answer,
            &error,
            devices,
            *device_width,
            stream,
            stream_length_before,
        ),
    }
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
    };
    let previous_root = previous.root;
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    match publish_without_units(&mut writer, &previous_root, plan) {
        Ok(output) => {
            let verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed_root_of_version_without_file(&output)],
                    reported_ceiling: None,
                }),
            );
            session.current = PoolVersion::WithoutFile(output);
            session.publishes_in_this_mount += 1;
            AppliedStep::judged_by_outcome_and_model(
                StepOutcome::Applied(AppliedEffect::Published {
                    reuse: RecordReuse::default(),
                }),
                verdict,
            )
        }
        Err(error) => {
            let member = format!(
                "publish_without_units({})",
                block_device_error_member(&error)
            );
            let verdict = judge_by_model(
                model,
                answer,
                &observed_refusal(
                    member.clone(),
                    refusal_reason_of_block_device_error(&error),
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
    /// 带文件的那几次发布逐次比过：第一次比挂载之前的镜像上生效根那棵分配记录树，之后比前一次的输出。
    Compared { publishes: usize },
    /// 写行与暖机都是零单元发布（生效根那一版树表 0 条），不写分配记录，没东西可比。
    NoPublishWithFile,
    /// 挂载之前的镜像上生效根那棵分配记录树读不出（例如它是恢复施加 journal 重建出来的一版、树表单元没落盘）：没比。
    PreviousRecordsUnreadable,
}

/// 挂载写出的写行与暖机那几次（带文件的）发布：逐次拿上一版的分配记录比，改写或新增的记录代要等于那一次的 txg；顺带数复用。
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
    for version in
        std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter())
    {
        match version {
            PoolVersion::WithFile(output) => {
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
            PoolVersion::WithoutFile(_) => {}
        }
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
            let verdict = judge_by_model(
                &mut pool.model,
                Ok(answer),
                &ObservedOutcome::Succeeded(observed_mount(&mounted)),
            );
            let publishes = 1 + mounted.output.warm_up_publishes.len();
            let instance = mounted.output.instance;
            pool.successful_mounts += 1;
            pool.session = Some(WritableSession {
                allocator: mounted.allocator,
                current: mounted.current,
                instance,
                publishes_in_this_mount: publishes,
            });
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Mounted {
                    instance,
                    publishes,
                    allocation_records_compared,
                    reuse,
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            }
        }
        // 挂载半路报错（写行之后的发布失败）：第 1 件不比分配代，入口没交回写出去的那几次发布；模型按「拒之前一个字节都不写」判。
        Err(error) => {
            let member = mount_error_member(&error);
            let reason = refusal_reason_of_mount_error(&error);
            let counted = allocation_records_counted_for_the_wall(
                reason,
                Some(&answer),
                0,
                &pool.devices,
                pool.device_width,
            );
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
                    counted,
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
    let mounted = mount_writable(parameters, &mut pool.devices);
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

fn apply_mount_rollback(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: RollbackTargetChoice,
) -> AppliedStep {
    pool.session = None;
    pool.model.close_session();
    let image_before_mount = pool.image();
    let (roots, _) = ring_roots_newest_first(&image_before_mount);
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
            let floor = newest_ring_root_floor(&image_before_mount);
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
    pool.mount_attempts += 1;
    let answer = pool
        .model
        .answer_mount_rollback(model_root_key(target.instance, target.checkpoint_txg));
    let stream_length_before = pool.stream.operation_count();
    let mounted = mount_rollback(parameters, &mut pool.devices, target, ShadowLedger::On);
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

fn apply_raise_rollback_floor(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: FloorTargetChoice,
) -> AppliedStep {
    let HistoryPool {
        device_width,
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
            let counted = allocation_records_counted_for_the_wall(
                reason,
                answer.as_ref().ok(),
                publishes_completed,
                devices,
                *device_width,
            );
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
                    counted,
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
    let verdict = judge_by_model(
        &mut pool.model,
        Ok(answer),
        &ObservedOutcome::Succeeded(ObservedEffect::ColdStart {
            read_back: observed_read_back(&report.outcome),
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
            apply_publish_overwrite(pool, &parameters, content, write_time_seconds)
        }
        HistoryOperation::PublishWithoutUnits => apply_publish_without_units(pool, &parameters),
        HistoryOperation::CloseAndMountWritable => apply_mount_writable(pool, &parameters),
        HistoryOperation::CloseAndMountRollback(choice) => {
            apply_mount_rollback(pool, &parameters, *choice)
        }
        HistoryOperation::RaiseRollbackFloor(choice) => {
            apply_raise_rollback_floor(pool, &parameters, *choice)
        }
        HistoryOperation::ColdStartRecover => apply_cold_start_recover(pool),
    }
}

/// 对一份镜像跑池级 checker，记每条不变量判绿、不适用各几次，交回判红的那几条。
fn violations_on(image: &MemoryPool, tally: &mut HistoryTally) -> Vec<(&'static str, String)> {
    tally.checker_runs += 1;
    let mut violations = Vec::new();
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Holds => *tally.invariant_holds.entry(invariant).or_insert(0) += 1,
            InvariantVerdict::NotApplicable(_) => {
                *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
            }
            InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
        }
    }
    violations
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

/// 每一步之后跑不跑池级 checker、判红了停不停。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerStepChecker {
    /// 起点之后与每一步写过盘之后都跑，判红就停（第 1 件的执行器；快档、前两个取样点与大档都是这一种）。
    Run,
    /// 起点之后与每一步写过盘之后都跑；判红时只要是「已知红」清单第 0 条那一形（根环转过一圈之后只有 I-3.1 红、记账多算，
    /// 增补 2 收口表第 ② 行）就只记一笔、历史接着走，别的判红照样停、照样分类（增补 3 第 2 件代码三方第二轮判决第三节第 3 条：
    /// 逼近分配记录墙那一段要在根环转过之后接着连发几十次，第一轮给它 `Skipped`，攻方两条只有 checker 看得见的变异——根环转过之后
    /// 回收门槛多一代、分配记录过 600 条之后「已分配」少记一槽——在门禁里没有一段红）。
    RunContinuingPastTheRingTurnForm,
    /// 不跑：只由理想模型、执行器自己的判定与 panic 让历史停下。给只看准入与模型的写死用例（分配记录墙的边沿、抬 F 与回退逼近墙）：
    /// 走到 812 条要连发五十次左右，根环转过一圈之后 checker 在合法状态上判 I-3.1 红（已知红第 0 条）。
    Skipped,
}

impl PerStepChecker {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            PerStepChecker::Run => "每一步之后跑池级 checker",
            PerStepChecker::RunContinuingPastTheRingTurnForm => {
                "每一步之后跑池级 checker，已知红第 0 条那一形只记不停"
            }
            PerStepChecker::Skipped => "不跑池级 checker（只由模型、执行器的判定与 panic 判）",
        }
    }

    fn runs_the_checker(self) -> bool {
        match self {
            PerStepChecker::Run | PerStepChecker::RunContinuingPastTheRingTurnForm => true,
            PerStepChecker::Skipped => false,
        }
    }
}

/// 一段历史怎么跑：每一步之后的 checker、两块盘多宽（取样点在种子、步数、比重之外的两个参数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryExecution {
    pub per_step_checker: PerStepChecker,
    pub device_width: HistoryDeviceWidth,
}

impl HistoryExecution {
    /// 第 1 件的执行器：每一步之后跑 checker、判红就停，两块 4 GiB 的盘（快档、前两个取样点、大档）。
    pub const CHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
        per_step_checker: PerStepChecker::Run,
        device_width: HistoryDeviceWidth::FourGibibytes,
    };

    /// 报告里的名字。
    #[must_use]
    pub fn name(self) -> String {
        format!(
            "{}；{}",
            self.per_step_checker.name(),
            self.device_width.name()
        )
    }
}

/// 跑一段历史，每一步之后跑池级 checker、两块 4 GiB 的盘，录制流不留内容（第 1 件只看镜像）。
#[must_use]
pub fn execute_history(history: &GeneratedHistory) -> HistoryRun {
    execute_history_with(
        history,
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    )
}

/// 跑一段历史，每一步之后跑池级 checker、两块 4 GiB 的盘（`execute_history_with` 的 `HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES`）。
pub fn execute_history_observing(
    history: &GeneratedHistory,
    stream: &SharedStream,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    execute_history_with(
        history,
        HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        stream,
        observer,
    )
}

/// 跑一段历史：在 `execution.device_width` 那么宽的两块盘上，起点之后与每一步写过盘之后按 `execution.per_step_checker` 对镜像跑池级
/// checker，再把那一刻的镜像交给观察者。第一次失败（违例、执行器判出的、模型对不上、panic）就停——`RunContinuingPastTheRingTurnForm`
/// 下只有「已知红」清单第 0 条那一形的违例不停，记进 `HistoryTally::ring_turn_form_steps_noted` 之后接着走。盘上的写与屏障录进 `stream`
/// （崩溃注入给开了内容保留的流）。
pub fn execute_history_with(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    let per_step_checker = execution.per_step_checker;
    let mut tally = HistoryTally::default();
    let mut outcomes: Vec<StepOutcome> = Vec::new();
    let mut pool_slot: Option<HistoryPool> = None;
    let position = Cell::new(StepPosition::StartingPoint);
    let mut completed_after_the_root_ring_turned = false;
    let body_result = with_panic_capture(|| -> Option<FailureObservation> {
        let (started_pool, starting_verdict) =
            HistoryPool::start(history.starting_point, execution.device_width, stream);
        let pool = pool_slot.insert(started_pool);
        let mut image = pool.image();
        let mut checked_stream_length = stream.operation_count();
        let violations_after_the_starting_point = if per_step_checker.runs_the_checker() {
            violations_on(&image, &mut tally)
        } else {
            Vec::new()
        };
        if let Some(verdict) = &starting_verdict {
            tally.note_model_verdict(verdict);
        }
        let starting_disagreement = starting_verdict.and_then(|verdict| verdict.disagreement);
        // 起点（txg 至多 3）的根环没转过，第 0 条那一形在这里不会出现：判红一律停。
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
                if per_step_checker.runs_the_checker() {
                    violations = violations_on(&image, &mut tally);
                }
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
                // 第 0 条那一形的判定（`ring_turn_leaves_allocated_statistic_above_walked`）自己要求没有 panic、执行器没判出、模型没对不上、
                // 只有 I-3.1 记账多算、根环转过：别的失败混在同一步里就不算这一形，照样停。
                let continues_past_the_ring_turn_form = per_step_checker
                    == PerStepChecker::RunContinuingPastTheRingTurnForm
                    && ring_turn_leaves_allocated_statistic_above_walked(&observation);
                if !continues_past_the_ring_turn_form {
                    return Some(observation);
                }
                tally.ring_turn_form_steps_noted += 1;
                tally.histories_with_the_ring_turn_form_noted = 1;
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
    HistoryRun {
        ending,
        outcomes,
        tally,
        // mkfs 在 `HistoryPool::start` 里跑，它自己 panic 时（`pool_slot` 还没填）流里还没有别的东西，报 0。
        operations_written_by_make_filesystem: pool_slot
            .as_ref()
            .map_or(0, |pool| pool.operations_written_by_make_filesystem),
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
/// 有行 (i, T) 且根的 txg > T 的 i 就是被抛弃的（与 `mount::mount_rollback` 判候选集同一句）。那个 txg 上一条根都没有、
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

/// 同上，读的盘面由调用方给：崩溃注入（增补 3 第 3 件）要在崩溃镜像（`crash::CrashImage`）上也拿到这两个事实，
/// 才判得出「已知红」第 0 条那一形（根环转过一圈）。
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
        HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
            index_from_newest,
        }) => smaller_selectors(index_from_newest)
            .map(|smaller| {
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                    index_from_newest: smaller,
                })
            })
            .collect(),
        HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::BeyondNewestRoot {
            txg_beyond_newest,
        }) => smaller_selectors(txg_beyond_newest)
            .map(|smaller| {
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::BeyondNewestRoot {
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
        HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRootAtTheNewestFloor)
        | HistoryOperation::PublishWithoutUnits
        | HistoryOperation::CloseAndMountWritable
        | HistoryOperation::ColdStartRecover => Vec::new(),
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
                            "still_fails 自己不 panic：历史里的 panic 在 execute_history 里接住",
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
}

/// 新发现收不收缩：收缩一类要把那段历史反复重跑几十次，debug 下每次几秒（门禁 59 号在变异下跑快档时量到过：只收一类也要两分多钟）。
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
        let Some(shrunk) = &self.shrunk else {
            let _ = writeln!(
                text,
                "  没收缩（这一档只报种子）：SINGLEFS_RANDOM_HISTORY_SHRINK_SEED={} 跑「收缩一个种子」那条 #[ignore] 用例，或跑大档",
                self.first_seed.0
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
    pub operations_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
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
            self.operations_per_history,
            self.weights.name,
            self.execution.name()
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
/// 某个线程在历史之外 panic（历史里的 panic 在 `execute_history` 里接住，走不到这里）。
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
    let next_offset = AtomicU64::new(0);
    let finished: Mutex<Vec<(GeneratedHistory, HistoryRun)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads.max(1) {
            scope.spawn(|| loop {
                let offset = next_offset.fetch_add(1, Ordering::Relaxed);
                if offset >= seed_count {
                    break;
                }
                let history = generate_history_with_weights(
                    HistorySeed(first_seed + offset),
                    operations_per_history,
                    weights,
                );
                let run =
                    execute_history_with(&history, execution, &SharedStream::new(), &mut |_| {});
                finished
                    .lock()
                    .expect("别的线程拿着这把锁时只做一次 push，不会在锁里 panic")
                    .push((history, run));
            });
        }
    });
    let mut finished = finished
        .into_inner()
        .expect("线程都已结束；锁里只做 push，没有线程在锁里 panic");
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
                    execution,
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
            }
        })
        .collect();
    CampaignReport {
        first_seed,
        seed_count,
        operations_per_history,
        weights: *weights,
        execution,
        tally,
        known_red_hits,
        new_findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                member: "MountError::RollbackTargetNotACandidate(NotInRing)".to_string()
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

    /// 执行器判出的失败不进「已知红」：哪怕同一步 checker 也只判了 I-3.1 多算、根环也转过了，签名取执行器判的那一种。
    #[test]
    fn harness_judgement_is_never_classified_as_a_known_red_form() {
        let observation = FailureObservation {
            position: StepPosition::Operation(21),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            violations: vec![("I-3.1", allocation_statistic_detail(26, 3, 0, 0))],
            panic: None,
            newest_ring_root_txg: Some(26),
            root_ring_slot_count: Some(24),
            harness_judgement: Some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
                records: vec![record(50178, 1, 5, false)],
                first_publish_txg: CheckpointTxg(26),
                last_publish_txg: CheckpointTxg(26),
            }),
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
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
                HistoryEnding::KnownRed { form: 0, .. }
            ),
            "去掉执行器判的那一条，同一个观察是已知红第 0 条"
        );
    }

    /// 「已知红」按机理认，不按签名认（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阴那一半）：
    /// 同一个签名（只有 I-3.1 红、记账多于遍历、根环转过一圈）下，机理对得上才接进第 0 条；
    /// 机理说环转过一圈并没把 F 之上的根挤出去（环里最老的根就在 F 上）、或者说明文字里根本没有机理标识的，照新发现报。
    #[test]
    fn known_red_forms_are_matched_by_mechanism_not_by_signature() {
        let with_detail = |detail: String| FailureObservation {
            position: StepPosition::Operation(21),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            violations: vec![("I-3.1", detail)],
            panic: None,
            newest_ring_root_txg: Some(26),
            root_ring_slot_count: Some(24),
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        assert!(
            matches!(
                classify_failure(with_detail(allocation_statistic_detail(26, 3, 0, 0))),
                HistoryEnding::KnownRed { form: 0, .. }
            ),
            "机理是「环转过一圈把 F 之上的根挤出了环」：接进第 0 条"
        );
        assert!(
            matches!(
                classify_failure(with_detail(allocation_statistic_detail(26, 7, 7, 0))),
                HistoryEnding::NewFinding { .. }
            ),
            "环里最老的根正好就在 F 上（F 之上一条根都没掉出环）：签名一样，机理不同，照新发现报"
        );
        assert!(
            matches!(
                classify_failure(with_detail(
                    "盘 0：记账的已分配 Some(3817472)，遍历全部有效根得到 3801088".to_string()
                )),
                HistoryEnding::NewFinding { .. }
            ),
            "说明文字里没有机理标识：认不出机理就不接，照新发现报"
        );
        // 第 1 条那一形（收口表第 43 行）：F 落在回退留下的空档里、环没转圈、机理是「F 把环里读得到的根挡在了遍历之外」。
        let mut raised_into_the_gap = with_detail(allocation_statistic_detail(9, 0, 5, 4));
        raised_into_the_gap.newest_ring_root_txg = Some(9);
        raised_into_the_gap.raised_floor_lands_only_on_abandoned_roots = Some(true);
        // 环转没转圈与这条机理正交：环转过了、而 F 之上一条根都没掉出环（最老的自证过的根就在 F 之下），照样是第 1 条。
        let mut the_ring_also_turned = with_detail(allocation_statistic_detail(27, 4, 8, 4));
        the_ring_also_turned.newest_ring_root_txg = Some(27);
        the_ring_also_turned.raised_floor_lands_only_on_abandoned_roots = Some(true);
        assert!(
            matches!(
                classify_failure(the_ring_also_turned),
                HistoryEnding::KnownRed { form: 1, .. }
            ),
            "环也转过一圈、但缺口出在 F 上：还是第 1 条，不是新发现"
        );
        assert!(
            matches!(
                classify_failure(raised_into_the_gap.clone()),
                HistoryEnding::KnownRed { form: 1, .. }
            ),
            "F 落在空档里、环没转圈、机理对得上：接进第 1 条"
        );
        let mut nothing_dropped_by_the_floor = raised_into_the_gap.clone();
        nothing_dropped_by_the_floor.violations =
            vec![("I-3.1", allocation_statistic_detail(9, 0, 5, 0))];
        assert!(
            matches!(
                classify_failure(nothing_dropped_by_the_floor),
                HistoryEnding::NewFinding { .. }
            ),
            "F 一条读得到的根都没挡掉：缺口不出在这条机理上，照新发现报"
        );
        // 这一形不再卡「这一步是抬 F」：崩溃状态摆在抬 F 之后的别的步上，同一机理照样接得进第 1 条。
        let mut later_step = raised_into_the_gap;
        later_step.operation_kind = Some(HistoryOperationKind::PublishOverwrite);
        assert!(
            matches!(
                classify_failure(later_step),
                HistoryEnding::KnownRed { form: 1, .. }
            ),
            "抬 F 之后的别的步上同一机理照样是第 1 条"
        );
    }

    /// 记录核对器判红的不接进「已知红」：签名是 `RecordCheck`，照新发现报。
    #[test]
    fn record_check_violations_are_never_absorbed_into_known_red_forms() {
        let mut observation = FailureObservation {
            position: StepPosition::Operation(21),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            violations: vec![("I-3.1", allocation_statistic_detail(26, 3, 0, 0))],
            panic: None,
            newest_ring_root_txg: Some(26),
            root_ring_slot_count: Some(24),
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck {
                root_without_record: true,
                claimed_state_missing_unit: false,
            },
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
                HistoryEnding::KnownRed { form: 0, .. }
            ),
            "去掉记录核对器那一条，同一个观察是已知红第 0 条"
        );
    }
}
