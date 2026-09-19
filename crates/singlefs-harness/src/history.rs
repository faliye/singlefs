//! 随机历史（里程碑「第二个事务」增补 3 第 1 件）：按种子生成一段操作序列，操作只调 `crates/` 今天的公开入口；每一步之后对镜像跑
//! 池级 checker。入口返回 `Err` 算合法结局；panic 与 checker 违例算失败——撞到「已知红」清单（`KNOWN_RED_FORMS`）里的形态照记、
//! 这段历史到此为止，清单外的算新发现，收缩到最短复现（`shrink_operations`）。冷启动读回的内容对不对这里不判，那是第 2 件模型的事。
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

use singlefs_checker::image::{chosen_superblocks, valid_roots, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_superblock, instance_table_of_root,
    readable_roots, recover, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_without_units, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError, ZeroUnitPublishPlan,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::DATA_UNIT_BYTES;

use crate::crash::{MemoryPool, SparseBlockDevice};
use crate::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use crate::{RecordingBlockDevice, SharedStream};

/// 每块内存盘的字节数：与 `tests/common` 的文件镜像同宽。
pub const HISTORY_DEVICE_BYTES: u64 = 4 << 30;

/// 历史里的一块盘：内存盘外面包录制器，写与屏障进调用方给的那条流。
pub type HistoryDevice = RecordingBlockDevice<SparseBlockDevice>;

/// mkfs 与发布的参数：E142 装置那一份（物理块 512、io_min 512），与各步用例相同。
#[must_use]
pub fn history_parameters() -> MakeFilesystemParameters {
    e142_parameters(512, 512)
}

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

fn draw_operation(source: &mut SeededRandomSource, kind: HistoryOperationKind) -> HistoryOperation {
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
        operations.push(draw_operation(&mut source, kind));
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

/// 一段历史跑到哪了：两块盘、这个进程的会话、挂载的次数。
struct HistoryPool {
    devices: Vec<(DeviceIdentity, HistoryDevice)>,
    session: Option<WritableSession>,
    successful_mounts: usize,
    mount_attempts: usize,
}

impl HistoryPool {
    fn start(starting_point: HistoryStartingPoint, stream: &SharedStream) -> Self {
        let parameters = history_parameters();
        let mut devices: Vec<(DeviceIdentity, HistoryDevice)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            SparseBlockDevice::new(
                                HISTORY_DEVICE_BYTES,
                                PhysicalBlockSizeInBytes(512),
                            ),
                            stream.clone(),
                        ),
                    )
                })
                .collect();
        let genesis = make_filesystem(&parameters, &mut devices)
            .expect("两块全零的 4 GiB 内存盘上按 E142 参数 mkfs：几何放得下，内存盘的写不报错");
        let session = match starting_point {
            HistoryStartingPoint::AfterMakeFilesystem => None,
            HistoryStartingPoint::AfterFirstFile => {
                let mut allocator = PoolAllocator::new(vec![
                    DeviceFreeMap::new(DeviceIdentity(0), HISTORY_DEVICE_BYTES),
                    DeviceFreeMap::new(DeviceIdentity(1), HISTORY_DEVICE_BYTES),
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
                let output = publish_first_file(
                    &mut writer,
                    &mut allocator,
                    &genesis.root,
                    FirstFile {
                        content: &first_file_content(),
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                    },
                    instance,
                    &warmed.last_record_bytes,
                )
                .expect("暖机之后的第一个文件：与 build_pool 同一条路，各步用例都走过");
                Some(WritableSession {
                    allocator,
                    current: PoolVersion::WithFile(output),
                    instance,
                    publishes_in_this_mount: 3,
                })
            }
        };
        Self {
            devices,
            session,
            successful_mounts: 0,
            mount_attempts: 0,
        }
    }

    /// 两块盘此刻的整份镜像（拷一份，checker 与观察者读它）。
    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.inner().image.clone()))
                .collect(),
            device_size_in_bytes: HISTORY_DEVICE_BYTES,
        }
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
    /// 根环一圈的槽数 R × S（按 checker 从超级块读出的几何）；读不到时 None。
    pub root_ring_slot_count: Option<u64>,
    /// 执行器自己判出的失败（分配代、冷启动读回）。
    pub harness_judgement: Option<HarnessJudgement>,
    /// 抬 F 那一步（入口返回 Ok）之后判红时：抬之前的镜像上，新 F 那个 txg 上的根是不是全属于被抛弃的实例
    /// （`raised_floor_lands_only_on_abandoned_roots`）；别的步、读不出、那个 txg 上没有根，都是 None。
    pub raised_floor_lands_only_on_abandoned_roots: Option<bool>,
}

impl FailureObservation {
    /// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
    #[must_use]
    pub fn root_ring_has_turned(&self) -> bool {
        root_ring_has_turned(self.newest_ring_root_txg, self.root_ring_slot_count)
    }
}

/// I-3.1 的违例文字 `盘 N：记账的已分配 Some(A)，遍历全部有效根得到 B`（`singlefs-checker` 的 `walk.rs`）里的 A 与 B；读不出就 None。
#[must_use]
pub fn allocated_and_walked_bytes(detail: &str) -> Option<(u64, u64)> {
    let after_allocated = detail.split_once("记账的已分配 Some(")?.1;
    let (allocated_text, rest) = after_allocated.split_once(')')?;
    let walked_text = rest.split_once("遍历全部有效根得到 ")?.1;
    Some((
        allocated_text.parse().ok()?,
        walked_text.trim().parse().ok()?,
    ))
}

/// 「已知红」清单的一条：形态与它在增补 2 收口表里的那一行。
pub struct KnownRedForm {
    /// 增补 2 收口表（`.claude/kb/milestone/02-second-txn.md`「增补 2」）里的哪一行。
    pub closeout_table_row: &'static str,
    /// 形态：哪条不变量或哪种 panic、在什么操作之后。
    pub shape: &'static str,
    pub matches: fn(&FailureObservation) -> bool,
}

/// 没有 panic、执行器没判出失败、判红的只有 I-3.1、而且是记账的已分配大于遍历全部有效根得到的（记账多算，不是少算）。
fn only_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.panic.is_none()
        && observation.harness_judgement.is_none()
        && !observation.violations.is_empty()
        && observation.violations.iter().all(|(invariant, detail)| {
            *invariant == "I-3.1"
                && allocated_and_walked_bytes(detail)
                    .is_some_and(|(allocated, walked)| allocated > walked)
        })
}

fn ring_turn_leaves_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {
    observation.root_ring_has_turned() && only_allocated_statistic_above_walked(observation)
}

/// 抬 F 那一步之后、抬之前的镜像上新 F 那个 txg 上的根全属于被抛弃的实例（F 落在回退留下的空档里）、根环没转圈、只有 I-3.1 红且
/// 记账多于遍历（代码三方第一轮判决第二节第 1 条收窄：此前不看空档，不经回退的抬 F 之后记账多算也被接走）。
fn raise_after_rollback_leaves_allocated_statistic_above_walked(
    observation: &FailureObservation,
) -> bool {
    observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)
        && observation.raised_floor_lands_only_on_abandoned_roots == Some(true)
        && !observation.root_ring_has_turned()
        && only_allocated_statistic_above_walked(observation)
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
    Panic { location: String },
    HarnessJudgement { judgement: &'static str },
    CheckerViolations { invariants: Vec<&'static str> },
}

impl FailureSignature {
    /// panic 先于执行器判的失败，执行器判的失败先于 checker 的违例（同一步里都有时签名取前一种，违例照样在观察里）。
    fn of(observation: &FailureObservation) -> Self {
        match (&observation.panic, &observation.harness_judgement) {
            (Some(panic), Some(_) | None) => FailureSignature::Panic {
                location: panic.location.clone(),
            },
            (None, Some(judgement)) => FailureSignature::HarnessJudgement {
                judgement: judgement.name(),
            },
            (None, None) => FailureSignature::CheckerViolations {
                invariants: observation
                    .violations
                    .iter()
                    .map(|(invariant, _)| *invariant)
                    .collect(),
            },
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
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
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
        for (invariant, count) in &other.invariant_holds {
            *self.invariant_holds.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
        }
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
            "  checker 跑了 {} 次；一个写都没发、沿用上一次结论的 {} 步",
            self.checker_runs, self.checker_runs_skipped_because_nothing_was_written
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
}

/// 每一步之后交给观察者的东西：第几步、这一步是什么、结局、此刻的整份镜像。
pub struct StepObservation<'run> {
    pub position: StepPosition,
    pub operation: Option<&'run HistoryOperation>,
    pub outcome: Option<&'run StepOutcome>,
    pub image: &'run MemoryPool,
}

fn publish_error_member(error: &PublishError) -> String {
    let member = match error {
        PublishError::NoSpaceFor { .. } => "NoSpaceFor",
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
    }
}

fn recovery_failure_member(failure: &RecoveryFailure) -> String {
    match failure {
        RecoveryFailure::NoValidSuperblock { .. } => {
            "RecoveryFailure::NoValidSuperblock".to_string()
        }
        RecoveryFailure::SuperblocksDisagree => "RecoveryFailure::SuperblocksDisagree".to_string(),
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
        MountError::RollbackTargetNotInRing(_) => "RollbackTargetNotInRing",
        MountError::RollbackTargetNotACandidate { reason, .. } => {
            return format!("MountError::RollbackTargetNotACandidate（{reason}）")
        }
        MountError::RollbackFloorAboveCeiling { .. } => "RollbackFloorAboveCeiling",
        MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. } => {
            "InstanceRowsOnVersionWithoutFileUnsupported"
        }
        MountError::RollbackToVersionWithoutFileUnsupported(_) => {
            "RollbackToVersionWithoutFileUnsupported"
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
    let Some(geometry) = chosen_superblocks(image)
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

/// 一步操作交回执行器的东西：结局，与执行器从这一步交回的东西里自己判出的失败（没有就是 None）。
struct AppliedStep {
    outcome: StepOutcome,
    harness_judgement: Option<HarnessJudgement>,
}

impl AppliedStep {
    /// 这一步交回的东西里没有执行器要另判的（前提不满足、入口返回 Err、挂载、零单元发布）：只看结局本身（冷启动读回报错）。
    fn judged_by_outcome_only(outcome: StepOutcome) -> Self {
        let harness_judgement = harness_judgement_of_outcome(&outcome);
        Self {
            outcome,
            harness_judgement,
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
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
            MissingPrecondition::NoWritableSession,
        ));
    };
    let content = content_choice.bytes();
    let root_to_carry_instance_table_from = *session.current.root();
    let previous_record_bytes = session.current.record_bytes().to_vec();
    let records_before = session.allocator.records().to_vec();
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
        Ok(output) => {
            let reuse = RecordReuse::between(&records_before, session.allocator.records());
            let publish_txg = output.root.checkpoint_txg;
            let harness_judgement = allocation_generation_judgement(
                &records_before,
                session.allocator.records(),
                publish_txg,
                publish_txg,
            );
            session.current = PoolVersion::WithFile(output);
            session.publishes_in_this_mount += 1;
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Published { reuse }),
                harness_judgement,
            }
        }
        Err(error) => AppliedStep::judged_by_outcome_only(StepOutcome::Refused {
            member: publish_error_member(&error),
        }),
    }
}

fn apply_publish_overwrite(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let HistoryPool {
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
            MissingPrecondition::NoWritableSession,
        ));
    };
    let PoolVersion::WithFile(previous) = &session.current else {
        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
            MissingPrecondition::CurrentVersionWithoutFile,
        ));
    };
    let previous = previous.clone();
    let content = content_choice.bytes();
    let records_before = session.allocator.records().to_vec();
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
        Ok(output) => {
            let reuse = RecordReuse::between(&records_before, session.allocator.records());
            let publish_txg = output.root.checkpoint_txg;
            let harness_judgement = allocation_generation_judgement(
                &records_before,
                session.allocator.records(),
                publish_txg,
                publish_txg,
            );
            session.current = PoolVersion::WithFile(output);
            session.publishes_in_this_mount += 1;
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Published { reuse }),
                harness_judgement,
            }
        }
        Err(error) => AppliedStep::judged_by_outcome_only(StepOutcome::Refused {
            member: publish_error_member(&error),
        }),
    }
}

fn apply_publish_without_units(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> StepOutcome {
    let HistoryPool {
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return StepOutcome::NotApplicable(MissingPrecondition::NoWritableSession);
    };
    // 前提：只在树表 0 条的一版上调（出处见 `MissingPrecondition::CurrentVersionWithFile` 的注释）。
    let PoolVersion::WithoutFile(previous) = &session.current else {
        return StepOutcome::NotApplicable(MissingPrecondition::CurrentVersionWithFile);
    };
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
            session.current = PoolVersion::WithoutFile(output);
            session.publishes_in_this_mount += 1;
            StepOutcome::Applied(AppliedEffect::Published {
                reuse: RecordReuse::default(),
            })
        }
        Err(error) => StepOutcome::Refused {
            member: format!(
                "publish_without_units({})",
                block_device_error_member(&error)
            ),
        },
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

fn settle_mount(
    pool: &mut HistoryPool,
    mounted: Result<singlefs_core::mount::Mounted, MountError>,
    image_before_mount: &MemoryPool,
) -> AppliedStep {
    match mounted {
        Ok(mounted) => {
            let (allocation_records_compared, reuse, harness_judgement) =
                mount_publish_allocation_judgement(image_before_mount, &mounted);
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
            }
        }
        // 挂载半路报错（写行之后的发布失败）：这里不比，入口没交回写出去的那几次发布。
        Err(error) => AppliedStep::judged_by_outcome_only(StepOutcome::Refused {
            member: mount_error_member(&error),
        }),
    }
}

fn apply_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    pool.session = None;
    pool.mount_attempts += 1;
    let image_before_mount = pool.image();
    let mounted = mount_writable(parameters, &mut pool.devices);
    settle_mount(pool, mounted, &image_before_mount)
}

fn apply_mount_rollback(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: RollbackTargetChoice,
) -> AppliedStep {
    pool.session = None;
    let image_before_mount = pool.image();
    let (roots, _) = ring_roots_newest_first(&image_before_mount);
    let Some((newest_txg, newest_instance)) = roots.first().copied() else {
        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
            MissingPrecondition::NoReadableRootInRing,
        ));
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
    };
    pool.mount_attempts += 1;
    let mounted = mount_rollback(parameters, &mut pool.devices, target, ShadowLedger::On);
    settle_mount(pool, mounted, &image_before_mount)
}

fn apply_raise_rollback_floor(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: FloorTargetChoice,
) -> AppliedStep {
    let HistoryPool {
        devices, session, ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
            MissingPrecondition::NoWritableSession,
        ));
    };
    let WritableSession {
        allocator,
        current,
        publishes_in_this_mount,
        ..
    } = session;
    let PoolVersion::WithFile(current) = current else {
        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(
            MissingPrecondition::CurrentVersionWithoutFile,
        ));
    };
    let current_floor = current.root.rollback_floor.0;
    let txg_before_raise = current.root.checkpoint_txg.0;
    // 前提：目标只取现行 F 及以上（出处见 `FloorTargetChoice::steps_above_current_floor` 的注释）。
    let choices = txg_before_raise.saturating_sub(current_floor) + 3;
    let new_floor = CheckpointTxg(current_floor + choice.steps_above_current_floor % choices);
    let records_before = allocator.records().to_vec();
    let records_on_disk_before = current.allocation_records.clone();
    let raised = raise_rollback_floor(
        parameters,
        devices,
        allocator,
        current,
        new_floor,
        ShadowLedger::On,
    );
    // 抬 F 的空发布逐次把现行版本往前推：半路报错时已经推出去的那几次也算这次挂载写出的根。
    *publishes_in_this_mount += usize::try_from(current.root.checkpoint_txg.0 - txg_before_raise)
        .expect("一次抬 F 至多推根环区域数那么多次");
    match raised {
        Ok(raised) => {
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
            }
        }
        // 半路报错：推出去的那几次没有逐次的输出，现行版本是最后成功的那一次；改写或新增的记录的代要落在这几次的 txg 里。
        Err(error) => AppliedStep {
            outcome: StepOutcome::Refused {
                member: mount_error_member(&error),
            },
            harness_judgement: allocation_generation_judgement(
                &records_on_disk_before,
                &current.allocation_records,
                CheckpointTxg(txg_before_raise + 1),
                current.root.checkpoint_txg,
            ),
        },
    }
}

fn apply_cold_start_recover(pool: &mut HistoryPool) -> StepOutcome {
    pool.session = None;
    let report = recover(&pool.devices, JournalPolicy::Consult);
    let read_back = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ColdStartReadBack::NoFile,
        RecoveryOutcome::FileRead { .. } => ColdStartReadBack::FileRead,
        RecoveryOutcome::Failed { .. } => ColdStartReadBack::Failed,
    };
    StepOutcome::Applied(AppliedEffect::Recovered {
        outcome: recovery_outcome_member(&report.outcome),
        read_back,
    })
}

fn apply_operation(
    pool: &mut HistoryPool,
    operation: &HistoryOperation,
    step_index: usize,
) -> AppliedStep {
    let parameters = history_parameters();
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
        HistoryOperation::PublishWithoutUnits => {
            AppliedStep::judged_by_outcome_only(apply_publish_without_units(pool, &parameters))
        }
        HistoryOperation::CloseAndMountWritable => apply_mount_writable(pool, &parameters),
        HistoryOperation::CloseAndMountRollback(choice) => {
            apply_mount_rollback(pool, &parameters, *choice)
        }
        HistoryOperation::RaiseRollbackFloor(choice) => {
            apply_raise_rollback_floor(pool, &parameters, *choice)
        }
        HistoryOperation::ColdStartRecover => {
            AppliedStep::judged_by_outcome_only(apply_cold_start_recover(pool))
        }
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

/// 跑一段历史，录制流不留内容（第 1 件只看镜像）。
#[must_use]
pub fn execute_history(history: &GeneratedHistory) -> HistoryRun {
    execute_history_observing(history, &SharedStream::new(), &mut |_| {})
}

/// 跑一段历史：起点之后与每一步操作之后都对镜像跑池级 checker，再把那一刻的镜像交给观察者。第一次失败（违例、panic）就停。
/// 盘上的写与屏障录进 `stream`（崩溃注入给开了内容保留的流）。
pub fn execute_history_observing(
    history: &GeneratedHistory,
    stream: &SharedStream,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    let mut tally = HistoryTally::default();
    let mut outcomes: Vec<StepOutcome> = Vec::new();
    let mut pool_slot: Option<HistoryPool> = None;
    let position = Cell::new(StepPosition::StartingPoint);
    let mut completed_after_the_root_ring_turned = false;
    let body_result = with_panic_capture(|| -> Option<FailureObservation> {
        let pool = pool_slot.insert(HistoryPool::start(history.starting_point, stream));
        let mut image = pool.image();
        let mut checked_stream_length = stream.operation_count();
        let violations_after_the_starting_point = violations_on(&image, &mut tally);
        if !violations_after_the_starting_point.is_empty() {
            return Some(failure_observation(
                &image,
                StepPosition::StartingPoint,
                None,
                violations_after_the_starting_point,
                None,
            ));
        }
        observer(&StepObservation {
            position: StepPosition::StartingPoint,
            operation: None,
            outcome: None,
            image: &image,
        });
        for (step_index, operation) in history.operations.iter().enumerate() {
            let step_position = StepPosition::Operation(step_index);
            position.set(step_position);
            let AppliedStep {
                outcome,
                harness_judgement,
            } = apply_operation(pool, operation, step_index);
            tally.note_outcome(operation, &outcome);
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
                violations = violations_on(&image, &mut tally);
                if !violations.is_empty() {
                    raised_floor_lands_only_on_abandoned = raised_floor.and_then(|new_floor| {
                        raised_floor_lands_only_on_abandoned_roots(
                            &image_before_this_step,
                            new_floor,
                        )
                    });
                }
            }
            if !violations.is_empty() || harness_judgement.is_some() {
                let mut observation = failure_observation(
                    &image,
                    step_position,
                    Some(operation.kind()),
                    violations,
                    None,
                );
                observation.harness_judgement = harness_judgement;
                observation.raised_floor_lands_only_on_abandoned_roots =
                    raised_floor_lands_only_on_abandoned;
                return Some(observation);
            }
            observer(&StepObservation {
                position: step_position,
                operation: Some(operation),
                outcome: outcomes.last(),
                image: &image,
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
                raised_floor_lands_only_on_abandoned_roots: None,
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
        raised_floor_lands_only_on_abandoned_roots: None,
    }
}

/// 抬 F 之前的镜像上，txg = `new_floor` 的根是不是全属于被抛弃的实例（F 落在回退留下的空档里）：按最新根指着的实例表，
/// 有行 (i, T) 且根的 txg > T 的 i 就是被抛弃的（与 `mount::mount_rollback` 判候选集同一句）。那个 txg 上一条根都没有、
/// 超级块或最新根或它的实例表读不出，都是 None（不算落在空档里）。
/// 读法用的是实现的 `recovery::readable_roots` / `choose_root` / `instance_table_of_root`（代码三方第一轮攻方的 P1 就这么读）：
/// checker 判候选集时解实例表的那一段不对外（`singlefs-checker` 的 `walk.rs` 里 `Walk::instance_table_rows`），这里没另写一份解析。
#[must_use]
pub fn raised_floor_lands_only_on_abandoned_roots(
    image_before_raising: &MemoryPool,
    new_floor: CheckpointTxg,
) -> Option<bool> {
    let superblock = choose_superblock(image_before_raising).ok()?;
    let roots_at_floor: Vec<RootRecord> = readable_roots(
        image_before_raising,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.checkpoint_txg == new_floor)
    .collect();
    if roots_at_floor.is_empty() {
        return None;
    }
    let newest_root = choose_root(image_before_raising, &superblock)?;
    let newest_table = instance_table_of_root(image_before_raising, &newest_root)?;
    Some(roots_at_floor.iter().all(|root| {
        newest_table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    }))
}

/// 按 checker 的读法：根环里最新那条根的 txg 与一圈的槽数 R × S。
fn newest_ring_root_and_slot_count(image: &MemoryPool) -> (Option<u64>, Option<u64>) {
    let (roots, root_ring_slot_count) = ring_roots_newest_first(image);
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
        HistoryOperation::PublishWithoutUnits
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

/// 一段失败的历史收缩到最短：起点不变，签名不变的删法与换法才留下。先截掉失败那一步之后的操作。
#[must_use]
pub fn shrink_failing_history(
    history: &GeneratedHistory,
    signature: &FailureSignature,
    worker_threads: usize,
) -> GeneratedHistory {
    let still_fails = |operations: &[HistoryOperation]| {
        let candidate = GeneratedHistory {
            seed: history.seed,
            starting_point: history.starting_point,
            operations: operations.to_vec(),
        };
        matches!(
            execute_history(&candidate).ending,
            HistoryEnding::NewFinding { signature: found, .. } if found == *signature
        )
    };
    let failing_length = match execute_history(history).ending {
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
    worker_threads: usize,
) -> ShrunkReproduction {
    let shrunk_history = shrink_failing_history(history, signature, worker_threads);
    let outcomes = execute_history(&shrunk_history).outcomes;
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
            "种子 [{}, {})，每段 {} 步，比重：{}",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.weights.name
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

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，每段 `operations_per_history` 步，分给 `worker_threads` 个线程（每段历史各用各的盘、
/// 各用各的录制流，互不相干）；跑完按种子排好再汇总，结论与线程数、调度次序无关。新发现按签名归类，按 `shrinking` 收缩各类的第一个种子。
///
/// # Panics
/// 某个线程在历史之外 panic（历史里的 panic 在 `execute_history` 里接住，走不到这里）。
#[must_use]
pub fn run_history_campaign(
    first_seed: u64,
    seed_count: u64,
    operations_per_history: usize,
    weights: &GenerationWeights,
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
                let run = execute_history(&history);
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
                FindingShrinking::EveryFindingClass => {
                    Some(shrink_to_reproduction(&history, &signature, worker_threads))
                }
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
                member: "MountError::RollbackTargetNotInRing".to_string()
            }),
            None
        );
    }

    /// 执行器判出的失败不进「已知红」：哪怕同一步 checker 也只判了 I-3.1 多算、根环也转过了，签名取执行器判的那一种。
    #[test]
    fn harness_judgement_is_never_classified_as_a_known_red_form() {
        let observation = FailureObservation {
            position: StepPosition::Operation(21),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            violations: vec![(
                "I-3.1",
                "盘 0：记账的已分配 Some(3817472)，遍历全部有效根得到 3801088".to_string(),
            )],
            panic: None,
            newest_ring_root_txg: Some(26),
            root_ring_slot_count: Some(24),
            harness_judgement: Some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
                records: vec![record(50178, 1, 5, false)],
                first_publish_txg: CheckpointTxg(26),
                last_publish_txg: CheckpointTxg(26),
            }),
            raised_floor_lands_only_on_abandoned_roots: None,
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
}
