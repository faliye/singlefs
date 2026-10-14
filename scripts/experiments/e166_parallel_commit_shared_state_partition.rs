//! E166 并行提交的共享状态分法：整体独占、分区、乐观三个候选，在模型层枚举两到三个事务的交错，
//! 数各自让两个事务拿到同一个落点或同一个号的次数（重复分配数），以及乐观那一臂的冲突重算次数。
//!
//! 跑前登记 `research/prompts/e166-preregistration.md`（含第十二节「修订」；树 ID 改号的第 2 次重跑登记 `research/prompts/e166-r2-prereg.md`）；问题单 `research/prompts/m3-concurrency-questions.md` 的 Q4 一行。
//! 独立手写的计数模型，**不与 `crates/` 共用代码**（不是入库装置）：模型里每一条与 `crates/` 对应的规则都写成本地常量或本地函数，
//! 旁边注释写 `crates/` 的文件:行（登记第十一节「停机」逐项对拍过），单测用登记第七节甲组的值回比。
//!
//! 六条臂（A 整体独占、A字面、P区、P设、O整、O读）与它们各自的阳性对照（A去锁、A字面去锁、P区同、P设去锁、O整不、O读不），
//! 三个 Q1 候选（Q1甲 准备并行落盘串行、Q1乙 一次发布装多个事务、Q1丙 多次发布同时在飞）。
//!
//! 这个装置对登记里几处含糊的地方取的读法（逐条写进登记第十二节「修订」，不放宽任何判据）：
//! 1. 用户数据落点一列（U）：各盘空闲表同步变化的臂（A、A字面、P区、O整、O读）按一个起点槽计一次，不乘盘数
//!    （`allocator.rs:1539` 注释「两盘同槽」；登记第七节乙 1 的 U 在默认盘数 2 下就是 4 / 4，只有按槽计才成立）；
//!    P设 各盘表各自变化，按「盘 × 起点槽」计（乙 7 的 U 重复合计按这个口径）。
//! 2. Q1乙 下 G、TXG / RS、T 归批发布者一步发（登记 5.1、Q-2、Q-7 两行），不争用；5.3 末句「Q1乙、Q1丙 下 G 等于两件那一行」
//!    只对 Q1丙 成立，Q1乙 下这几列在阳性对照里也是 0。
//! 3. O读不 与 O整不 同一份代码（提交不校验、整份含 X / J / TXG 的拷贝值写回）：乙 1 要 X 一列 4 / 4，
//!    O读 的 X 在提交点发这件事被「关护栏」一并关掉。
//! 4. 记账 ACC 与 Q-10（丢更新）属第二段，这一版不建模；BASE 只建「有没有文件」位与版本号。
//!
//! 子命令：不带参数 = 第一段（停机对拍、阳性对照与乙 / 丙组锚点、六条臂在默认几何与 D = 1 上的 Q-1 至 Q-9）。
// admission: always 独立手写的纯算术计数模型，每次现算全部调度的枚举与锚点，判的是此刻装置的样子
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::HashMap;
use std::fmt::Write as _;

// ── 与 crates/ 对应的规则：本地常量，旁边写出处（登记第十一节停机条款逐项对拍过） ──────────────

/// 新池第一版的 inode 号水位 = `FIRST_INODE_NUMBER (1) + 1`。`crates/singlefs-core/src/transaction.rs:109`、`:6268`–`6269`。
const INITIAL_INODE_WATERMARK: u8 = 2;
/// 树 ID 的号与水位：树 ID 从 65536 起编，装不进 `u8`，这一族（水位、发号起点、发号个数）用 `u32`。
/// 第 2 次重跑登记 `research/prompts/e166-r2-prereg.md` 第五节臂 E2 ①，取 `u32`（登记第十二节记）。
type TreeIdentifierNumber = u32;
// 两个水位的值出自 D8（核心索引结构） 已定项 11 定案句：mkfs 时水位 65536，第一个文件版本那次之后 65548。
// 对应 `crates/singlefs-format/src/lib.rs` 的 `TREE_IDENTIFIER_WATERMARK_AT_MKFS`、`TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH`（按名字找，行号会漂）。
const TREE_IDENTIFIER_WATERMARK_AT_MKFS: TreeIdentifierNumber = 65536;
const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: TreeIdentifierNumber = 65548;
/// 第一个文件版本那次发布从水位起连号发的树 ID 个数 = 两个水位之差 = 十二个（D8 已定项 8 ②「十二棵树的号从水位起连号发」）。
/// 对应 `crates/singlefs-core/src/transaction.rs` 的 `FileVersionTreeIdentifiers::issued_from_watermark`（十二项偏移表 `OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER`）。
const TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION: TreeIdentifierNumber =
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS;
/// 根环区域数 R。`crates/singlefs-format/src/lib.rs:238`。
const ROOT_RING_REGIONS: u32 = 3;
/// journal 一条记录的定长字节数。`crates/singlefs-format/src/lib.rs:191`。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// journal 环长的 mkfs 默认值（768 MiB）。`crates/singlefs-format/src/lib.rs:213`。
const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 默认环槽数 = 默认环长 ÷ 记录长（196608）。登记第四节甲。
const DEFAULT_JOURNAL_RING_SLOTS: u32 = 196_608;
/// 用户数据落点的跨度（槽）。`crates/singlefs-core/src/allocator.rs:1410`（`Placement { slot, span: 2 }`）。
const USER_DATA_SPAN_SLOTS: u32 = 2;

// ── 模型自己的取值（登记 5.1），不抄 crates/ ─────────────────────────────────────

/// 每块盘的用户数据空闲表有 16 个「对」（起点槽 0、2、…、30）。
const USER_DATA_PAIR_COUNT: usize = 16;
/// 提交内生块池：一个 64 槽的开放段。
const COMMIT_GENERATED_POOL_SLOTS: u8 = 64;
/// 每次发布重写的树节点个数（每个节点 1 槽）。
const COMMIT_GENERATED_NODES_PER_PUBLISH: u8 = 2;
/// 初始 checkpoint_txg、jsn 计数器、事务号已用最大值。
const INITIAL_CHECKPOINT_TXG: u8 = 5;
const INITIAL_JSN_COUNTER: u8 = 5;
const INITIAL_HIGHEST_TRANSACTION_NUMBER: u8 = 1;
/// 根环每区槽数的默认值。
const DEFAULT_ROOT_SLOTS_PER_REGION: u32 = 4;
/// 默认盘数与第一个反向取样点。
const DEFAULT_DEVICE_COUNT: usize = 2;
const SAMPLE_DEVICE_COUNT: usize = 1;
/// P设 一次提交内各盘不一致的重算上限（登记乙 7 口径：超过记「截断」）。
const DEVICE_DISAGREEMENT_RETRY_CAP: u8 = 2;

const MAX_TRANSACTIONS: usize = 4;
const MAX_DEVICES: usize = 2;
const MAX_REGIONS: usize = 4;
const NO_PAIR: u8 = u8::MAX;
const NO_SLOT: u8 = u8::MAX;
const FIRST_DEVICE_STEP_INDEX: u8 = 1;

// ── 共享状态分组：事务碰哪几样、锁外锁内怎么分 ───────────────────────────────────

const GROUP_USER: u8 = 1;
const GROUP_COMMIT_SLOTS: u8 = 2;
const GROUP_INODE: u8 = 4;
const GROUP_TREE: u8 = 8;
const GROUP_TRANSACTION_NUMBER: u8 = 16;
const GROUP_JOURNAL: u8 = 32;
const GROUP_ROOT_RING: u8 = 64;
const ALL_GROUPS: u8 = 127;

/// A字面 的锁只罩候选定义点名的四样：分配器（U、G）、根环、journal 写指针、inode 号水位；锁外是事务号与树 ID 水位（记账不建模）。
const LITERAL_LOCK_INSIDE_GROUPS: u8 =
    GROUP_USER | GROUP_COMMIT_SLOTS | GROUP_ROOT_RING | GROUP_JOURNAL | GROUP_INODE;
const LITERAL_LOCK_OUTSIDE_GROUPS: u8 = GROUP_TRANSACTION_NUMBER | GROUP_TREE;
/// O读 在提交那一原子步里按当时共享值发的几样（排序器在提交点上，登记 5.2）。
const OPTIMISTIC_COMMIT_TIME_GROUPS: u8 = GROUP_TRANSACTION_NUMBER | GROUP_JOURNAL | GROUP_ROOT_RING;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ParallelStage {
    PrepareParallelCommitSerial,
    OnePublishCarriesManyTransactions,
    ManyPublishesInFlight,
}

const ALL_STAGES: [ParallelStage; 3] = [
    ParallelStage::PrepareParallelCommitSerial,
    ParallelStage::OnePublishCarriesManyTransactions,
    ParallelStage::ManyPublishesInFlight,
];

impl ParallelStage {
    fn label(self) -> &'static str {
        match self {
            ParallelStage::PrepareParallelCommitSerial => "Q1甲",
            ParallelStage::OnePublishCarriesManyTransactions => "Q1乙",
            ParallelStage::ManyPublishesInFlight => "Q1丙",
        }
    }

    /// 每个事务自己碰哪几组共享状态（登记 5.1「Q1 候选决定每个事务自己碰哪些资源」）。
    fn transaction_groups(self) -> u8 {
        match self {
            ParallelStage::PrepareParallelCommitSerial => {
                GROUP_USER | GROUP_COMMIT_SLOTS | GROUP_INODE | GROUP_TREE | GROUP_TRANSACTION_NUMBER
            }
            ParallelStage::OnePublishCarriesManyTransactions => {
                GROUP_USER | GROUP_INODE | GROUP_TRANSACTION_NUMBER | GROUP_JOURNAL
            }
            ParallelStage::ManyPublishesInFlight => ALL_GROUPS,
        }
    }

    /// 由一条串行管道在事务提交那一刻按序发的几组（只有 Q1甲：journal 计数器与 checkpoint_txg）。
    fn pipeline_groups(self) -> u8 {
        match self {
            ParallelStage::PrepareParallelCommitSerial => GROUP_JOURNAL | GROUP_ROOT_RING,
            ParallelStage::OnePublishCarriesManyTransactions => 0,
            ParallelStage::ManyPublishesInFlight => 0,
        }
    }

    /// BASE 版本号进不进 O读 的读集（登记 5.2：Q1甲 与 Q1丙 进，Q1乙 不进）。
    fn base_version_in_read_set(self) -> bool {
        match self {
            ParallelStage::PrepareParallelCommitSerial => true,
            ParallelStage::OnePublishCarriesManyTransactions => false,
            ParallelStage::ManyPublishesInFlight => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    WholeLock,
    LiteralLock,
    RegionPartition,
    DevicePartition,
    OptimisticWholeVersion,
    OptimisticReadSet,
}

const ALL_ARMS: [Arm; 6] = [
    Arm::WholeLock,
    Arm::LiteralLock,
    Arm::RegionPartition,
    Arm::DevicePartition,
    Arm::OptimisticWholeVersion,
    Arm::OptimisticReadSet,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ArmVariant {
    Normal,
    /// 登记 5.3：关掉这条臂自己那条护栏的阳性对照。
    PositiveControl,
}

impl Arm {
    fn label(self, variant: ArmVariant) -> &'static str {
        match (self, variant) {
            (Arm::WholeLock, ArmVariant::Normal) => "A",
            (Arm::WholeLock, ArmVariant::PositiveControl) => "A去锁",
            (Arm::LiteralLock, ArmVariant::Normal) => "A字面",
            (Arm::LiteralLock, ArmVariant::PositiveControl) => "A字面去锁",
            (Arm::RegionPartition, ArmVariant::Normal) => "P区",
            (Arm::RegionPartition, ArmVariant::PositiveControl) => "P区同",
            (Arm::DevicePartition, ArmVariant::Normal) => "P设",
            (Arm::DevicePartition, ArmVariant::PositiveControl) => "P设去锁",
            (Arm::OptimisticWholeVersion, ArmVariant::Normal) => "O整",
            (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl) => "O整不",
            (Arm::OptimisticReadSet, ArmVariant::Normal) => "O读",
            (Arm::OptimisticReadSet, ArmVariant::PositiveControl) => "O读不",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ShapeKind {
    /// 每个事务 1 个用户数据单元、建 1 个 inode。
    SameShape,
    /// 事务 0 只建 1 个 inode；其余事务只写 1 个用户数据单元。
    Disjoint,
    /// 每个事务 1 个用户数据单元、建 2 个 inode（只用于 P区 的预留不够那一格）。
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    MultiInode,
}

impl ShapeKind {
    fn label(self) -> &'static str {
        match self {
            ShapeKind::SameShape => "same",
            ShapeKind::Disjoint => "disjoint",
            ShapeKind::MultiInode => "multi_inode",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct TransactionShape {
    user_units: u8,
    new_inodes: u8,
}

fn shapes_of(kind: ShapeKind, transaction_count: usize) -> Vec<TransactionShape> {
    (0..transaction_count)
        .map(|transaction_index| match kind {
            ShapeKind::SameShape => TransactionShape { user_units: 1, new_inodes: 1 },
            ShapeKind::Disjoint => {
                if transaction_index == 0 {
                    TransactionShape { user_units: 0, new_inodes: 1 }
                } else {
                    TransactionShape { user_units: 1, new_inodes: 0 }
                }
            }
            ShapeKind::MultiInode => TransactionShape { user_units: 1, new_inodes: 2 },
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InodeReservation {
    /// P区 / P设 在 CLAIM 里预留的 inode 号个数 = 本事务要建的数（默认 r_i 够用）。
    Exact,
    /// 每个要建 inode 的事务固定预留这么多个（第二段「多 inode、r_i = 1」用）。
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    Fixed(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RegionCountChoice {
    EqualToTransactionCount,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    Fixed(usize),
}

#[derive(Clone, Copy, Debug)]
struct CellSpec {
    arm: Arm,
    variant: ArmVariant,
    stage: ParallelStage,
    shape_kind: ShapeKind,
    initial_has_file: bool,
    transaction_count: usize,
    device_count: usize,
    region_count: RegionCountChoice,
    inode_reservation: InodeReservation,
    root_slots_per_region: u32,
    journal_ring_slots: u32,
}

impl CellSpec {
    fn default_geometry(
        arm: Arm,
        variant: ArmVariant,
        stage: ParallelStage,
        shape_kind: ShapeKind,
        initial_has_file: bool,
        transaction_count: usize,
        device_count: usize,
    ) -> CellSpec {
        CellSpec {
            arm,
            variant,
            stage,
            shape_kind,
            initial_has_file,
            transaction_count,
            device_count,
            region_count: RegionCountChoice::EqualToTransactionCount,
            inode_reservation: InodeReservation::Exact,
            root_slots_per_region: DEFAULT_ROOT_SLOTS_PER_REGION,
            journal_ring_slots: DEFAULT_JOURNAL_RING_SLOTS,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StepKind {
    WholeLocked,
    WholeSnapshot,
    WholeWriteBack,
    OutsideSnapshot,
    InsideLocked,
    InsideSnapshot,
    InsideWriteBack,
    OutsideWriteBack,
    RegionClaim,
    RegionFallback,
    RegionCommit,
    DeviceClaim,
    DeviceTake(usize),
    DeviceRead(usize),
    DeviceWrite(usize),
    DeviceCheck,
    DeviceCommit,
    OptimisticSnapshot,
    OptimisticCommit,
}

/// 每条臂的步序列（登记 5.2 表「每个事务的步」）。
fn steps_of(arm: Arm, variant: ArmVariant, device_count: usize) -> Vec<StepKind> {
    match (arm, variant) {
        (Arm::WholeLock, ArmVariant::Normal) => vec![StepKind::WholeLocked],
        (Arm::WholeLock, ArmVariant::PositiveControl) => vec![StepKind::WholeSnapshot, StepKind::WholeWriteBack],
        (Arm::LiteralLock, ArmVariant::Normal) => {
            vec![StepKind::OutsideSnapshot, StepKind::InsideLocked, StepKind::OutsideWriteBack]
        }
        (Arm::LiteralLock, ArmVariant::PositiveControl) => vec![
            StepKind::OutsideSnapshot,
            StepKind::InsideSnapshot,
            StepKind::InsideWriteBack,
            StepKind::OutsideWriteBack,
        ],
        (Arm::RegionPartition, ArmVariant::Normal) | (Arm::RegionPartition, ArmVariant::PositiveControl) => {
            vec![StepKind::RegionClaim, StepKind::RegionFallback, StepKind::RegionCommit]
        }
        (Arm::DevicePartition, ArmVariant::Normal) => {
            let mut steps = vec![StepKind::DeviceClaim];
            for device in 0..device_count {
                steps.push(StepKind::DeviceTake(device));
            }
            steps.push(StepKind::DeviceCheck);
            steps.push(StepKind::DeviceCommit);
            steps
        }
        (Arm::DevicePartition, ArmVariant::PositiveControl) => {
            let mut steps = vec![StepKind::DeviceClaim];
            for device in 0..device_count {
                steps.push(StepKind::DeviceRead(device));
                steps.push(StepKind::DeviceWrite(device));
            }
            steps.push(StepKind::DeviceCheck);
            steps.push(StepKind::DeviceCommit);
            steps
        }
        (Arm::OptimisticWholeVersion, ArmVariant::Normal)
        | (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl)
        | (Arm::OptimisticReadSet, ArmVariant::Normal)
        | (Arm::OptimisticReadSet, ArmVariant::PositiveControl) => {
            vec![StepKind::OptimisticSnapshot, StepKind::OptimisticCommit]
        }
    }
}

/// 一个格的全部不变参数：臂、Q1 候选、事务形状、初始态与几何。
struct Model {
    arm: Arm,
    variant: ArmVariant,
    stage: ParallelStage,
    shapes: Vec<TransactionShape>,
    initial_has_file: bool,
    transaction_count: usize,
    device_count: usize,
    region_count: usize,
    inode_reservation: InodeReservation,
    root_slots_per_region: u32,
    journal_ring_slots: u32,
    steps: Vec<StepKind>,
    transaction_groups: u8,
    pipeline_groups: u8,
}

impl Model {
    fn new(spec: CellSpec) -> Model {
        assert!(spec.transaction_count >= 1 && spec.transaction_count <= MAX_TRANSACTIONS);
        assert!(spec.device_count >= 1 && spec.device_count <= MAX_DEVICES);
        let region_count = match spec.region_count {
            RegionCountChoice::EqualToTransactionCount => spec.transaction_count,
            RegionCountChoice::Fixed(count) => count,
        };
        assert!(region_count >= 1 && region_count <= MAX_REGIONS);
        Model {
            arm: spec.arm,
            variant: spec.variant,
            stage: spec.stage,
            shapes: shapes_of(spec.shape_kind, spec.transaction_count),
            initial_has_file: spec.initial_has_file,
            transaction_count: spec.transaction_count,
            device_count: spec.device_count,
            region_count,
            inode_reservation: spec.inode_reservation,
            root_slots_per_region: spec.root_slots_per_region,
            journal_ring_slots: spec.journal_ring_slots,
            steps: steps_of(spec.arm, spec.variant, spec.device_count),
            transaction_groups: spec.stage.transaction_groups(),
            pipeline_groups: spec.stage.pipeline_groups(),
        }
    }

    fn user_pairs_counted_per_device(&self) -> bool {
        self.arm == Arm::DevicePartition
    }

    fn optimistic_uses_read_set(&self) -> bool {
        self.arm == Arm::OptimisticReadSet && self.variant == ArmVariant::Normal
    }

    fn region_pair_range(&self, region: usize) -> (usize, usize) {
        let region_pairs = USER_DATA_PAIR_COUNT / self.region_count;
        (region * region_pairs, (region + 1) * region_pairs)
    }

    fn region_slot_range(&self, region: usize) -> (u8, u8) {
        let region_slots = usize::from(COMMIT_GENERATED_POOL_SLOTS) / self.region_count;
        (small(region * region_slots), small((region + 1) * region_slots))
    }
}

fn small(value: usize) -> u8 {
    u8::try_from(value).expect("模型里的号与槽都小于 256")
}

// ── 状态 ─────────────────────────────────────────────────────────────────────

/// 每个池一份、各事务共用的状态（登记第三节清单里本登记量的那几样；ACC 不建模）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct SharedState {
    /// 同步臂的用户数据空闲表：第 p 位 = 第 p 个对已被占（各盘同步变化，只记一份）。
    user_taken: u16,
    /// P设 的逐盘空闲表。
    device_taken: [u16; MAX_DEVICES],
    /// 提交内生块池的 bump 游标（非分区臂用；分区臂用 `region_cursor`）。
    commit_cursor: u8,
    region_cursor: [u8; MAX_REGIONS],
    /// 被在飞事务占着的区（按位）。
    region_busy: u8,
    inode_watermark: u8,
    tree_watermark: TreeIdentifierNumber,
    /// BASE 的「有没有文件」位：视图里没有文件的发布才发十二个树 ID。
    has_file: bool,
    highest_transaction: u8,
    counter: u8,
    txg: u8,
    /// 提交版本号：O整 整版校验用；O读 在 Q1甲 / Q1丙 下当 BASE 版本号读。
    version: u8,
}

const EMPTY_SHARED_STATE: SharedState = SharedState {
    user_taken: 0,
    device_taken: [0; MAX_DEVICES],
    commit_cursor: 0,
    region_cursor: [0; MAX_REGIONS],
    region_busy: 0,
    inode_watermark: 0,
    tree_watermark: 0,
    has_file: false,
    highest_transaction: 0,
    counter: 0,
    txg: 0,
    version: 0,
};

/// 一个事务拿到的号与槽。0（号）或 `NO_PAIR` / `NO_SLOT`（槽）表示没拿。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Holdings {
    user_pairs: [u8; MAX_DEVICES],
    commit_slot_start: u8,
    inode_start: u8,
    inode_count: u8,
    extra_inode_start: u8,
    extra_inode_count: u8,
    tree_start: TreeIdentifierNumber,
    transaction_number: u8,
    counter: u8,
    txg: u8,
}

const NOTHING_HELD: Holdings = Holdings {
    user_pairs: [NO_PAIR; MAX_DEVICES],
    commit_slot_start: NO_SLOT,
    inode_start: 0,
    inode_count: 0,
    extra_inode_start: 0,
    extra_inode_count: 0,
    tree_start: 0,
    transaction_number: 0,
    counter: 0,
    txg: 0,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct TransactionState {
    program_counter: u8,
    /// 冲突重算次数：O 臂校验失败回到第一步；P设 的 CHECK 判各盘不一致后回到第一个 DEV 步。
    retries: u8,
    fallbacks: u8,
    committed: bool,
    /// 乐观臂与「去锁」阳性对照里这个事务自己的整份拷贝（或拷贝里属于它那几组的字段）。
    view: SharedState,
    seen_version: u8,
    items: Holdings,
    claimed_regions: u8,
    device_read_value: u8,
    inode_holes: u8,
    needs_interval_fallback: bool,
}

const FRESH_TRANSACTION: TransactionState = TransactionState {
    program_counter: 0,
    retries: 0,
    fallbacks: 0,
    committed: false,
    view: EMPTY_SHARED_STATE,
    seen_version: 0,
    items: NOTHING_HELD,
    claimed_regions: 0,
    device_read_value: NO_PAIR,
    inode_holes: 0,
    needs_interval_fallback: false,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct ModelState {
    shared: SharedState,
    transactions: [TransactionState; MAX_TRANSACTIONS],
}

fn initial_state(model: &Model) -> ModelState {
    let mut region_cursor = [0; MAX_REGIONS];
    for region in 0..model.region_count {
        region_cursor[region] = model.region_slot_range(region).0;
    }
    let shared = SharedState {
        user_taken: 0,
        device_taken: [0; MAX_DEVICES],
        commit_cursor: 0,
        region_cursor,
        region_busy: 0,
        inode_watermark: INITIAL_INODE_WATERMARK,
        tree_watermark: if model.initial_has_file {
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH
        } else {
            TREE_IDENTIFIER_WATERMARK_AT_MKFS
        },
        has_file: model.initial_has_file,
        highest_transaction: INITIAL_HIGHEST_TRANSACTION_NUMBER,
        counter: INITIAL_JSN_COUNTER,
        txg: INITIAL_CHECKPOINT_TXG,
        version: 0,
    };
    let mut transactions = [FRESH_TRANSACTION; MAX_TRANSACTIONS];
    for unused in transactions.iter_mut().skip(model.transaction_count) {
        unused.committed = true;
    }
    ModelState { shared, transactions }
}

// ── 号与槽的发放规则 ─────────────────────────────────────────────────────────

/// 用户数据落点：起点槽号最小、两槽都空的对。`crates/singlefs-core/src/allocator.rs:732`（`lowest_user_data_slot`）。
fn lowest_free_pair(taken_pairs: u16, first_pair: usize, end_pair: usize) -> Option<u8> {
    (first_pair..end_pair)
        .find(|pair| taken_pairs & (1 << pair) == 0)
        .map(small)
}

/// 按分组从 `view` 里取本事务要的号与槽，推进 `view`，记进 `held`（只改它那几组对应的字段）。
/// 同一条规则既用在锁内直接取、乐观臂拷贝上取，也用在串行管道上。
fn take_resources(view: &mut SharedState, shape: TransactionShape, groups: u8, held: &mut Holdings) {
    if groups & GROUP_USER != 0 && shape.user_units > 0 {
        let pair = lowest_free_pair(view.user_taken, 0, USER_DATA_PAIR_COUNT).expect("16 个对够 n ≤ 4 个事务用");
        view.user_taken |= 1 << pair;
        held.user_pairs = [pair; MAX_DEVICES];
    }
    if groups & GROUP_COMMIT_SLOTS != 0 {
        // 提交内生块从开放段 bump，游标挪到落点之后。`crates/singlefs-core/src/allocator.rs:1494`、`:1522`。
        held.commit_slot_start = view.commit_cursor;
        view.commit_cursor += COMMIT_GENERATED_NODES_PER_PUBLISH;
        assert!(view.commit_cursor <= COMMIT_GENERATED_POOL_SLOTS);
    }
    if groups & GROUP_INODE != 0 && shape.new_inodes > 0 {
        // inode 号从上一版水位起连号，新水位 = 上一版 + 新建数。`crates/singlefs-core/src/transaction.rs:4839`、`:4848`、`:6266`–`6270`。
        held.inode_start = view.inode_watermark;
        held.inode_count = shape.new_inodes;
        view.inode_watermark = view.inode_watermark + shape.new_inodes;
    }
    if groups & GROUP_TREE != 0 {
        if !view.has_file {
            held.tree_start = view.tree_watermark;
            view.tree_watermark += TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION;
        }
        view.has_file = true;
    }
    if groups & GROUP_TRANSACTION_NUMBER != 0 {
        // 事务号从上一版加一。`crates/singlefs-core/src/transaction.rs:4733`。
        view.highest_transaction += 1;
        held.transaction_number = view.highest_transaction;
    }
    if groups & GROUP_JOURNAL != 0 {
        // jsn 计数器从上一版加一。`crates/singlefs-core/src/transaction.rs:4732`。
        view.counter += 1;
        held.counter = view.counter;
    }
    if groups & GROUP_ROOT_RING != 0 {
        // checkpoint_txg 从上一版加一。`crates/singlefs-core/src/transaction.rs:3420`。
        view.txg += 1;
        held.txg = view.txg;
    }
}

/// 把 `source` 里属于 `groups` 的字段整份写进 `destination`（覆盖，不合并：这就是丢更新发生的地方）。
fn copy_group_fields(destination: &mut SharedState, source: &SharedState, groups: u8) {
    if groups & GROUP_USER != 0 {
        destination.user_taken = source.user_taken;
    }
    if groups & GROUP_COMMIT_SLOTS != 0 {
        destination.commit_cursor = source.commit_cursor;
    }
    if groups & GROUP_INODE != 0 {
        destination.inode_watermark = source.inode_watermark;
    }
    if groups & GROUP_TREE != 0 {
        destination.tree_watermark = source.tree_watermark;
        destination.has_file = source.has_file;
    }
    if groups & GROUP_TRANSACTION_NUMBER != 0 {
        destination.highest_transaction = source.highest_transaction;
    }
    if groups & GROUP_JOURNAL != 0 {
        destination.counter = source.counter;
    }
    if groups & GROUP_ROOT_RING != 0 {
        destination.txg = source.txg;
    }
}

/// 根环槽 = (txg mod 3, (txg div 3) mod S)。`crates/singlefs-core/src/root_ring.rs:114`–`115`。
fn root_ring_slot_key(txg: u8, slots_per_region: u32) -> u64 {
    let region = u32::from(txg) % ROOT_RING_REGIONS;
    let slot = (u32::from(txg) / ROOT_RING_REGIONS) % slots_per_region;
    (u64::from(region) << 32) | u64::from(slot)
}

/// journal 环槽 = (计数器 − 1) mod 环槽数。`crates/singlefs-core/src/journal.rs:196`–`201`。
fn journal_ring_slot(counter: u8, ring_slots: u32) -> u32 {
    (u32::from(counter) - 1) % ring_slots
}

// ── 步的使能与施加 ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StepResult {
    Applied,
    /// P设 的 CHECK 判不一致而重算已到上限：这一枝记一次「截断」，不进调度数。
    Truncated,
}

fn pair_range_mask(first_pair: usize, end_pair: usize) -> u16 {
    u16::try_from((1u32 << end_pair) - (1u32 << first_pair)).expect("16 个对装得进 u16")
}

fn lowest_free_region(model: &Model, shared: &SharedState) -> Option<usize> {
    (0..model.region_count).find(|region| shared.region_busy & (1 << region) == 0)
}

/// 本事务在自己认领的区里还缺用户数据落点或提交内生块落点（区装不下）。
fn region_shortage(model: &Model, shape: TransactionShape, held: &Holdings) -> bool {
    let missing_user_pair =
        model.transaction_groups & GROUP_USER != 0 && shape.user_units > 0 && held.user_pairs[0] == NO_PAIR;
    let missing_commit_slots = model.transaction_groups & GROUP_COMMIT_SLOTS != 0 && held.commit_slot_start == NO_SLOT;
    missing_user_pair || missing_commit_slots
}

fn is_step_enabled(model: &Model, state: &ModelState, transaction_index: usize) -> bool {
    let transaction = &state.transactions[transaction_index];
    let shape = model.shapes[transaction_index];
    match model.steps[usize::from(transaction.program_counter)] {
        StepKind::RegionClaim => {
            model.variant == ArmVariant::PositiveControl || lowest_free_region(model, &state.shared).is_some()
        }
        StepKind::RegionFallback => {
            !region_shortage(model, shape, &transaction.items) || lowest_free_region(model, &state.shared).is_some()
        }
        StepKind::WholeLocked
        | StepKind::WholeSnapshot
        | StepKind::WholeWriteBack
        | StepKind::OutsideSnapshot
        | StepKind::InsideLocked
        | StepKind::InsideSnapshot
        | StepKind::InsideWriteBack
        | StepKind::OutsideWriteBack
        | StepKind::RegionCommit
        | StepKind::DeviceClaim
        | StepKind::DeviceTake(_)
        | StepKind::DeviceRead(_)
        | StepKind::DeviceWrite(_)
        | StepKind::DeviceCheck
        | StepKind::DeviceCommit
        | StepKind::OptimisticSnapshot
        | StepKind::OptimisticCommit => true,
    }
}

fn restart_transaction(transaction: &mut TransactionState) {
    transaction.program_counter = 0;
    transaction.view = EMPTY_SHARED_STATE;
    transaction.seen_version = 0;
    transaction.items = NOTHING_HELD;
    transaction.claimed_regions = 0;
    transaction.device_read_value = NO_PAIR;
    transaction.inode_holes = 0;
    transaction.needs_interval_fallback = false;
}

/// 提交那一刻：串行管道按序发它那几样，事务记为已提交，不再带拷贝与认领的区。
fn finish_commit(model: &Model, shared: &mut SharedState, transaction: &mut TransactionState, shape: TransactionShape) {
    take_resources(shared, shape, model.pipeline_groups, &mut transaction.items);
    transaction.committed = true;
    transaction.program_counter = small(model.steps.len());
    transaction.view = EMPTY_SHARED_STATE;
    transaction.seen_version = 0;
    transaction.claimed_regions = 0;
    transaction.device_read_value = NO_PAIR;
    transaction.needs_interval_fallback = false;
}

/// 在 `region` 里不加同步地取落点（P区：用户数据对与提交内生块槽）；取不到的留着，由 FALLBACK 补。
fn take_from_region(
    model: &Model,
    region: usize,
    shape: TransactionShape,
    view: &mut SharedState,
    held: &mut Holdings,
) {
    let (first_pair, end_pair) = model.region_pair_range(region);
    if model.transaction_groups & GROUP_USER != 0 && shape.user_units > 0 && held.user_pairs[0] == NO_PAIR {
        if let Some(pair) = lowest_free_pair(view.user_taken, first_pair, end_pair) {
            view.user_taken |= 1 << pair;
            held.user_pairs = [pair; MAX_DEVICES];
        }
    }
    if model.transaction_groups & GROUP_COMMIT_SLOTS != 0 && held.commit_slot_start == NO_SLOT {
        let cursor = view.region_cursor[region];
        let (_, end_slot) = model.region_slot_range(region);
        if cursor + COMMIT_GENERATED_NODES_PER_PUBLISH <= end_slot {
            held.commit_slot_start = cursor;
            view.region_cursor[region] = cursor + COMMIT_GENERATED_NODES_PER_PUBLISH;
        }
    }
}

/// 分区臂在 CLAIM 里预留号区间（`advance` = 推进共享水位；P区同 不推进，每个事务都读到同一个当前值）。
fn reserve_numbers(
    model: &Model,
    shared: &mut SharedState,
    shape: TransactionShape,
    advance: bool,
    transaction: &mut TransactionState,
    groups: u8,
) {
    if groups & GROUP_INODE != 0 && shape.new_inodes > 0 {
        let needed = shape.new_inodes;
        let reserved = match model.inode_reservation {
            InodeReservation::Exact => needed,
            InodeReservation::Fixed(count) => count,
        };
        transaction.items.inode_start = shared.inode_watermark;
        transaction.items.inode_count = reserved.min(needed);
        if advance {
            shared.inode_watermark += reserved;
        }
        transaction.inode_holes = reserved.saturating_sub(needed);
        transaction.needs_interval_fallback = reserved < needed;
    }
    if groups & GROUP_TREE != 0 && !shared.has_file {
        transaction.items.tree_start = shared.tree_watermark;
        if advance {
            shared.tree_watermark += TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION;
        }
    }
    if groups & GROUP_TRANSACTION_NUMBER != 0 {
        transaction.items.transaction_number = shared.highest_transaction + 1;
        if advance {
            shared.highest_transaction += 1;
        }
    }
    if groups & GROUP_JOURNAL != 0 {
        transaction.items.counter = shared.counter + 1;
        if advance {
            shared.counter += 1;
        }
    }
    if groups & GROUP_ROOT_RING != 0 {
        transaction.items.txg = shared.txg + 1;
        if advance {
            shared.txg += 1;
        }
    }
}

/// O读 的校验：只核本事务真读到的（登记 5.2）。
fn read_set_still_valid(model: &Model, shared: &SharedState, transaction: &TransactionState) -> bool {
    let held = &transaction.items;
    let user_pair_still_free = held.user_pairs[0] == NO_PAIR || shared.user_taken & (1 << held.user_pairs[0]) == 0;
    let commit_cursor_unmoved =
        model.transaction_groups & GROUP_COMMIT_SLOTS == 0 || shared.commit_cursor == held.commit_slot_start;
    let inode_start_unmoved = held.inode_count == 0 || shared.inode_watermark == held.inode_start;
    let tree_start_unmoved = held.tree_start == 0 || shared.tree_watermark == held.tree_start;
    let base_version_unmoved = !model.stage.base_version_in_read_set() || shared.version == transaction.seen_version;
    user_pair_still_free && commit_cursor_unmoved && inode_start_unmoved && tree_start_unmoved && base_version_unmoved
}

/// O读 校验通过之后的写回：只加自己那一份，不覆盖别人的。
fn apply_read_set_writes(shared: &mut SharedState, held: &Holdings, groups: u8) {
    if held.user_pairs[0] != NO_PAIR {
        shared.user_taken |= 1 << held.user_pairs[0];
    }
    if groups & GROUP_COMMIT_SLOTS != 0 {
        shared.commit_cursor += COMMIT_GENERATED_NODES_PER_PUBLISH;
    }
    if held.inode_count > 0 {
        shared.inode_watermark += held.inode_count;
    }
    if held.tree_start != 0 {
        shared.tree_watermark += TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION;
    }
    if groups & GROUP_TREE != 0 {
        shared.has_file = true;
    }
}

fn apply_step(model: &Model, state: &mut ModelState, transaction_index: usize) -> StepResult {
    let mut transaction = state.transactions[transaction_index];
    let shape = model.shapes[transaction_index];
    let step = model.steps[usize::from(transaction.program_counter)];
    match step {
        StepKind::WholeLocked => {
            take_resources(&mut state.shared, shape, model.transaction_groups, &mut transaction.items);
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::WholeSnapshot => {
            let mut copy = state.shared;
            take_resources(&mut copy, shape, model.transaction_groups, &mut transaction.items);
            transaction.view = copy;
            transaction.program_counter += 1;
        }
        StepKind::WholeWriteBack => {
            copy_group_fields(&mut state.shared, &transaction.view, model.transaction_groups);
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::OutsideSnapshot => {
            let mut copy = state.shared;
            take_resources(
                &mut copy,
                shape,
                LITERAL_LOCK_OUTSIDE_GROUPS & model.transaction_groups,
                &mut transaction.items,
            );
            transaction.view = copy;
            transaction.program_counter += 1;
        }
        StepKind::InsideLocked => {
            take_resources(
                &mut state.shared,
                shape,
                LITERAL_LOCK_INSIDE_GROUPS & model.transaction_groups,
                &mut transaction.items,
            );
            transaction.program_counter += 1;
        }
        StepKind::InsideSnapshot => {
            let inside_groups = LITERAL_LOCK_INSIDE_GROUPS & model.transaction_groups;
            let mut copy = state.shared;
            take_resources(&mut copy, shape, inside_groups, &mut transaction.items);
            copy_group_fields(&mut transaction.view, &copy, inside_groups);
            transaction.program_counter += 1;
        }
        StepKind::InsideWriteBack => {
            copy_group_fields(
                &mut state.shared,
                &transaction.view,
                LITERAL_LOCK_INSIDE_GROUPS & model.transaction_groups,
            );
            transaction.program_counter += 1;
        }
        StepKind::OutsideWriteBack => {
            copy_group_fields(
                &mut state.shared,
                &transaction.view,
                LITERAL_LOCK_OUTSIDE_GROUPS & model.transaction_groups,
            );
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::RegionClaim => {
            let is_normal = model.variant == ArmVariant::Normal;
            let region = if is_normal {
                lowest_free_region(model, &state.shared).expect("RegionClaim 只在有空闲区时使能")
            } else {
                0
            };
            if is_normal {
                state.shared.region_busy |= 1 << region;
            }
            transaction.claimed_regions = 1 << region;
            let mut view = EMPTY_SHARED_STATE;
            view.user_taken = state.shared.user_taken;
            view.region_cursor = state.shared.region_cursor;
            take_from_region(model, region, shape, &mut view, &mut transaction.items);
            reserve_numbers(
                model,
                &mut state.shared,
                shape,
                is_normal,
                &mut transaction,
                model.transaction_groups & !(GROUP_USER | GROUP_COMMIT_SLOTS),
            );
            transaction.view = view;
            transaction.program_counter =
                if transaction.needs_interval_fallback || region_shortage(model, shape, &transaction.items) {
                    1
                } else {
                    2
                };
        }
        StepKind::RegionFallback => {
            if transaction.needs_interval_fallback {
                let missing = shape.new_inodes - transaction.items.inode_count;
                transaction.items.extra_inode_start = state.shared.inode_watermark;
                transaction.items.extra_inode_count = missing;
                state.shared.inode_watermark += missing;
                transaction.needs_interval_fallback = false;
            }
            if region_shortage(model, shape, &transaction.items) {
                let region = lowest_free_region(model, &state.shared).expect("RegionFallback 缺区时只在有空闲区时使能");
                state.shared.region_busy |= 1 << region;
                transaction.claimed_regions |= 1 << region;
                let (first_pair, end_pair) = model.region_pair_range(region);
                let region_mask = pair_range_mask(first_pair, end_pair);
                let mut view = transaction.view;
                view.user_taken = (view.user_taken & !region_mask) | (state.shared.user_taken & region_mask);
                view.region_cursor[region] = state.shared.region_cursor[region];
                take_from_region(model, region, shape, &mut view, &mut transaction.items);
                transaction.view = view;
            }
            transaction.fallbacks += 1;
            transaction.program_counter = 2;
        }
        StepKind::RegionCommit => {
            for region in 0..model.region_count {
                if transaction.claimed_regions & (1 << region) != 0 {
                    let (first_pair, end_pair) = model.region_pair_range(region);
                    let region_mask = pair_range_mask(first_pair, end_pair);
                    state.shared.user_taken =
                        (state.shared.user_taken & !region_mask) | (transaction.view.user_taken & region_mask);
                    state.shared.region_cursor[region] = transaction.view.region_cursor[region];
                }
            }
            state.shared.region_busy &= !transaction.claimed_regions;
            if model.transaction_groups & GROUP_TREE != 0 {
                state.shared.has_file = true;
            }
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::DeviceClaim => {
            reserve_numbers(
                model,
                &mut state.shared,
                shape,
                true,
                &mut transaction,
                model.transaction_groups & !(GROUP_USER | GROUP_COMMIT_SLOTS),
            );
            transaction.program_counter += 1;
        }
        StepKind::DeviceTake(device) => {
            if shape.user_units > 0 {
                let pair = lowest_free_pair(state.shared.device_taken[device], 0, USER_DATA_PAIR_COUNT)
                    .expect("16 个对够 n ≤ 4 个事务用");
                state.shared.device_taken[device] |= 1 << pair;
                transaction.items.user_pairs[device] = pair;
            }
            transaction.program_counter += 1;
        }
        StepKind::DeviceRead(device) => {
            if shape.user_units > 0 {
                transaction.device_read_value = lowest_free_pair(state.shared.device_taken[device], 0, USER_DATA_PAIR_COUNT)
                    .expect("16 个对够 n ≤ 4 个事务用");
            }
            transaction.program_counter += 1;
        }
        StepKind::DeviceWrite(device) => {
            if shape.user_units > 0 {
                let pair = transaction.device_read_value;
                state.shared.device_taken[device] |= 1 << pair;
                transaction.items.user_pairs[device] = pair;
                transaction.device_read_value = NO_PAIR;
            }
            transaction.program_counter += 1;
        }
        StepKind::DeviceCheck => {
            // 各盘答得不一致就拒（今天的实现在 `crates/singlefs-core/src/allocator.rs:159` 拒，`:1377` 的调用点）。
            let picks_agree = transaction.items.user_pairs[..model.device_count]
                .windows(2)
                .all(|picks_of_two_devices| picks_of_two_devices[0] == picks_of_two_devices[1]);
            if picks_agree {
                transaction.program_counter += 1;
            } else if transaction.retries == DEVICE_DISAGREEMENT_RETRY_CAP {
                return StepResult::Truncated;
            } else {
                for device in 0..model.device_count {
                    let pair = transaction.items.user_pairs[device];
                    if pair == NO_PAIR {
                        continue;
                    }
                    let held_by_another_transaction = (0..model.transaction_count).any(|other| {
                        other != transaction_index && state.transactions[other].items.user_pairs[device] == pair
                    });
                    if !held_by_another_transaction {
                        state.shared.device_taken[device] &= !(1 << pair);
                    }
                }
                transaction.items.user_pairs = [NO_PAIR; MAX_DEVICES];
                transaction.retries += 1;
                transaction.program_counter = FIRST_DEVICE_STEP_INDEX;
            }
        }
        StepKind::DeviceCommit => {
            if model.transaction_groups & GROUP_COMMIT_SLOTS != 0 {
                transaction.items.commit_slot_start = state.shared.commit_cursor;
                state.shared.commit_cursor += COMMIT_GENERATED_NODES_PER_PUBLISH;
            }
            if model.transaction_groups & GROUP_TREE != 0 {
                state.shared.has_file = true;
            }
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::OptimisticSnapshot => {
            let snapshot_groups = if model.optimistic_uses_read_set() {
                model.transaction_groups & !OPTIMISTIC_COMMIT_TIME_GROUPS
            } else {
                model.transaction_groups
            };
            let mut copy = state.shared;
            take_resources(&mut copy, shape, snapshot_groups, &mut transaction.items);
            transaction.seen_version = state.shared.version;
            transaction.view = if model.optimistic_uses_read_set() { EMPTY_SHARED_STATE } else { copy };
            transaction.program_counter += 1;
        }
        StepKind::OptimisticCommit => {
            let accepted = if model.variant == ArmVariant::PositiveControl {
                true
            } else if model.optimistic_uses_read_set() {
                read_set_still_valid(model, &state.shared, &transaction)
            } else {
                transaction.seen_version == state.shared.version
            };
            if accepted {
                if model.optimistic_uses_read_set() {
                    apply_read_set_writes(&mut state.shared, &transaction.items, model.transaction_groups);
                    take_resources(
                        &mut state.shared,
                        shape,
                        model.transaction_groups & OPTIMISTIC_COMMIT_TIME_GROUPS,
                        &mut transaction.items,
                    );
                } else {
                    copy_group_fields(&mut state.shared, &transaction.view, model.transaction_groups);
                }
                state.shared.version += 1;
                finish_commit(model, &mut state.shared, &mut transaction, shape);
            } else {
                let retries_after_restart = transaction.retries + 1;
                restart_transaction(&mut transaction);
                transaction.retries = retries_after_restart;
            }
        }
    }
    state.transactions[transaction_index] = transaction;
    StepResult::Applied
}

// ── 重复数：一个调度跑完之后，每个被发出的项数「最终持有它的已提交事务个数」，记 持有数 − 1 再求和 ──────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Resource {
    UserDataSlot,
    CommitGeneratedSlot,
    InodeNumber,
    TreeIdentifier,
    TransactionNumber,
    JsnCounter,
    JournalRingSlot,
    CheckpointTxg,
    RootRingSlot,
}

const RESOURCE_COUNT: usize = 9;
const ALL_RESOURCES: [Resource; RESOURCE_COUNT] = [
    Resource::UserDataSlot,
    Resource::CommitGeneratedSlot,
    Resource::InodeNumber,
    Resource::TreeIdentifier,
    Resource::TransactionNumber,
    Resource::JsnCounter,
    Resource::JournalRingSlot,
    Resource::CheckpointTxg,
    Resource::RootRingSlot,
];

impl Resource {
    fn label(self) -> &'static str {
        match self {
            Resource::UserDataSlot => "U",
            Resource::CommitGeneratedSlot => "G",
            Resource::InodeNumber => "I",
            Resource::TreeIdentifier => "T",
            Resource::TransactionNumber => "X",
            Resource::JsnCounter => "J",
            Resource::JournalRingSlot => "JS",
            Resource::CheckpointTxg => "TXG",
            Resource::RootRingSlot => "RS",
        }
    }

    fn index(self) -> usize {
        ALL_RESOURCES
            .iter()
            .position(|candidate| *candidate == self)
            .expect("每个资源都在 ALL_RESOURCES 里")
    }
}

fn push_held_keys(model: &Model, resource: Resource, held: &Holdings, keys: &mut Vec<u64>) {
    match resource {
        Resource::UserDataSlot => {
            if model.user_pairs_counted_per_device() {
                for device in 0..model.device_count {
                    let pair = held.user_pairs[device];
                    if pair != NO_PAIR {
                        keys.push((u64::try_from(device).expect("盘号很小") << 8) | u64::from(pair));
                    }
                }
            } else if held.user_pairs[0] != NO_PAIR {
                keys.push(u64::from(held.user_pairs[0]));
            }
        }
        Resource::CommitGeneratedSlot => {
            if held.commit_slot_start != NO_SLOT {
                for node in 0..COMMIT_GENERATED_NODES_PER_PUBLISH {
                    keys.push(u64::from(held.commit_slot_start + node));
                }
            }
        }
        Resource::InodeNumber => {
            for number in held.inode_start..held.inode_start + held.inode_count {
                keys.push(u64::from(number));
            }
            for number in held.extra_inode_start..held.extra_inode_start + held.extra_inode_count {
                keys.push(u64::from(number));
            }
        }
        Resource::TreeIdentifier => {
            if held.tree_start != 0 {
                for offset in 0..TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION {
                    keys.push(u64::from(held.tree_start + offset));
                }
            }
        }
        Resource::TransactionNumber => {
            if held.transaction_number != 0 {
                keys.push(u64::from(held.transaction_number));
            }
        }
        Resource::JsnCounter => {
            if held.counter != 0 {
                keys.push(u64::from(held.counter));
            }
        }
        Resource::JournalRingSlot => {
            if held.counter != 0 {
                keys.push(u64::from(journal_ring_slot(held.counter, model.journal_ring_slots)));
            }
        }
        Resource::CheckpointTxg => {
            if held.txg != 0 {
                keys.push(u64::from(held.txg));
            }
        }
        Resource::RootRingSlot => {
            if held.txg != 0 {
                keys.push(root_ring_slot_key(held.txg, model.root_slots_per_region));
            }
        }
    }
}

/// 一批项里的重复数：每一项的持有数 − 1 求和。
fn duplicates_among(keys: &mut [u64]) -> u32 {
    keys.sort_unstable();
    let mut duplicates = 0u32;
    let mut run_start = 0;
    while run_start < keys.len() {
        let mut run_end = run_start + 1;
        while run_end < keys.len() && keys[run_end] == keys[run_start] {
            run_end += 1;
        }
        let holders = run_end - run_start;
        duplicates += u32::try_from(holders - 1).expect("持有数很小");
        run_start = run_end;
    }
    duplicates
}

fn highest_inode_number_held(held: &Holdings) -> Option<u8> {
    let first_range = (held.inode_count > 0).then(|| held.inode_start + held.inode_count - 1);
    let second_range = (held.extra_inode_count > 0).then(|| held.extra_inode_start + held.extra_inode_count - 1);
    first_range.into_iter().chain(second_range).max()
}

/// 一个状态上谓词消费的几个量（登记第八节「轨迹」）。
struct StateMetrics {
    /// 已提交事务之间各资源的重复数。
    duplicates: [u32; RESOURCE_COUNT],
    /// inode 号水位余量 = 共享水位 − 已提交事务持有的最大 inode 号 − 1（I-9.6）。
    inode_margin: i32,
    busy_regions: u32,
    unused_reserved_inodes: u32,
}

fn state_metrics(model: &Model, state: &ModelState) -> StateMetrics {
    let committed: Vec<&TransactionState> = state.transactions[..model.transaction_count]
        .iter()
        .filter(|transaction| transaction.committed)
        .collect();
    let mut duplicates = [0u32; RESOURCE_COUNT];
    let mut keys: Vec<u64> = Vec::with_capacity(64);
    if !committed.is_empty() {
        for resource in ALL_RESOURCES {
            keys.clear();
            for transaction in &committed {
                push_held_keys(model, resource, &transaction.items, &mut keys);
            }
            duplicates[resource.index()] = duplicates_among(&mut keys);
        }
    }
    let highest_held = committed
        .iter()
        .filter_map(|transaction| highest_inode_number_held(&transaction.items))
        .max()
        .unwrap_or(INITIAL_INODE_WATERMARK - 1);
    StateMetrics {
        duplicates,
        inode_margin: i32::from(state.shared.inode_watermark) - i32::from(highest_held) - 1,
        busy_regions: state.shared.region_busy.count_ones(),
        unused_reserved_inodes: state.transactions[..model.transaction_count]
            .iter()
            .map(|transaction| u32::from(transaction.inode_holes))
            .sum(),
    }
}

/// 全部调度（或一个子树）上的汇总：每个量对每个叶（完整调度）算一次，再求和 / 取最大；
/// 谓词量（轨迹）按「每个调度每一步之后」算，求峰值与为正的步数。
#[derive(Clone, PartialEq, Eq, Debug)]
struct Aggregate {
    schedules: u128,
    duplicate_total: [u128; RESOURCE_COUNT],
    duplicate_schedules: [u128; RESOURCE_COUNT],
    duplicate_max_single_schedule: [u32; RESOURCE_COUNT],
    retries_total: u128,
    retries_schedules: u128,
    retries_max_schedule: u32,
    retries_max_transaction: u32,
    fallbacks_total: u128,
    truncated: u128,
    deadlocks: u128,
    duplicate_peak: [u32; RESOURCE_COUNT],
    duplicate_positive_steps: [u128; RESOURCE_COUNT],
    inode_margin_minimum: i32,
    inode_margin_negative_steps: u128,
    inode_margin_final_minimum: i32,
    busy_regions_peak: u32,
    busy_regions_positive_steps: u128,
    busy_regions_final_maximum: u32,
    unused_reserved_peak: u32,
    unused_reserved_positive_steps: u128,
    unused_reserved_final_maximum: u32,
}

impl Aggregate {
    fn empty() -> Aggregate {
        Aggregate {
            schedules: 0,
            duplicate_total: [0; RESOURCE_COUNT],
            duplicate_schedules: [0; RESOURCE_COUNT],
            duplicate_max_single_schedule: [0; RESOURCE_COUNT],
            retries_total: 0,
            retries_schedules: 0,
            retries_max_schedule: 0,
            retries_max_transaction: 0,
            fallbacks_total: 0,
            truncated: 0,
            deadlocks: 0,
            duplicate_peak: [0; RESOURCE_COUNT],
            duplicate_positive_steps: [0; RESOURCE_COUNT],
            inode_margin_minimum: i32::MAX,
            inode_margin_negative_steps: 0,
            inode_margin_final_minimum: i32::MAX,
            busy_regions_peak: 0,
            busy_regions_positive_steps: 0,
            busy_regions_final_maximum: 0,
            unused_reserved_peak: 0,
            unused_reserved_positive_steps: 0,
            unused_reserved_final_maximum: 0,
        }
    }

    fn merge(&mut self, other: &Aggregate) {
        self.schedules += other.schedules;
        self.retries_total += other.retries_total;
        self.retries_schedules += other.retries_schedules;
        self.retries_max_schedule = self.retries_max_schedule.max(other.retries_max_schedule);
        self.retries_max_transaction = self.retries_max_transaction.max(other.retries_max_transaction);
        self.fallbacks_total += other.fallbacks_total;
        self.truncated += other.truncated;
        self.deadlocks += other.deadlocks;
        for resource in 0..RESOURCE_COUNT {
            self.duplicate_total[resource] += other.duplicate_total[resource];
            self.duplicate_schedules[resource] += other.duplicate_schedules[resource];
            self.duplicate_max_single_schedule[resource] =
                self.duplicate_max_single_schedule[resource].max(other.duplicate_max_single_schedule[resource]);
            self.duplicate_peak[resource] = self.duplicate_peak[resource].max(other.duplicate_peak[resource]);
            self.duplicate_positive_steps[resource] += other.duplicate_positive_steps[resource];
        }
        self.inode_margin_minimum = self.inode_margin_minimum.min(other.inode_margin_minimum);
        self.inode_margin_negative_steps += other.inode_margin_negative_steps;
        self.inode_margin_final_minimum = self.inode_margin_final_minimum.min(other.inode_margin_final_minimum);
        self.busy_regions_peak = self.busy_regions_peak.max(other.busy_regions_peak);
        self.busy_regions_positive_steps += other.busy_regions_positive_steps;
        self.busy_regions_final_maximum = self.busy_regions_final_maximum.max(other.busy_regions_final_maximum);
        self.unused_reserved_peak = self.unused_reserved_peak.max(other.unused_reserved_peak);
        self.unused_reserved_positive_steps += other.unused_reserved_positive_steps;
        self.unused_reserved_final_maximum = self.unused_reserved_final_maximum.max(other.unused_reserved_final_maximum);
    }

    /// 一个完整调度（叶）：期末各量。
    fn add_leaf(&mut self, metrics: &StateMetrics, retries_of_each_transaction: &[u8], fallbacks: u32) {
        self.schedules += 1;
        for resource in 0..RESOURCE_COUNT {
            let duplicates = metrics.duplicates[resource];
            self.duplicate_total[resource] += u128::from(duplicates);
            if duplicates > 0 {
                self.duplicate_schedules[resource] += 1;
            }
            self.duplicate_max_single_schedule[resource] = self.duplicate_max_single_schedule[resource].max(duplicates);
        }
        let retries_of_schedule: u32 = retries_of_each_transaction.iter().map(|retries| u32::from(*retries)).sum();
        self.retries_total += u128::from(retries_of_schedule);
        if retries_of_schedule > 0 {
            self.retries_schedules += 1;
        }
        self.retries_max_schedule = self.retries_max_schedule.max(retries_of_schedule);
        let most_retries_of_one_transaction =
            retries_of_each_transaction.iter().copied().max().map_or(0, u32::from);
        self.retries_max_transaction = self.retries_max_transaction.max(most_retries_of_one_transaction);
        self.fallbacks_total += u128::from(fallbacks);
        self.inode_margin_final_minimum = self.inode_margin_final_minimum.min(metrics.inode_margin);
        self.busy_regions_final_maximum = self.busy_regions_final_maximum.max(metrics.busy_regions);
        self.unused_reserved_final_maximum = self.unused_reserved_final_maximum.max(metrics.unused_reserved_inodes);
    }

    /// 一个状态（某个调度某一步之后）上谓词消费的量，对这个状态之下的全部 `leaves_below` 个调度各算一步。
    fn add_state_metrics(&mut self, metrics: &StateMetrics, leaves_below: u128) {
        for resource in 0..RESOURCE_COUNT {
            self.duplicate_peak[resource] = self.duplicate_peak[resource].max(metrics.duplicates[resource]);
            if metrics.duplicates[resource] > 0 {
                self.duplicate_positive_steps[resource] += leaves_below;
            }
        }
        self.inode_margin_minimum = self.inode_margin_minimum.min(metrics.inode_margin);
        if metrics.inode_margin < 0 {
            self.inode_margin_negative_steps += leaves_below;
        }
        self.busy_regions_peak = self.busy_regions_peak.max(metrics.busy_regions);
        if metrics.busy_regions > 0 {
            self.busy_regions_positive_steps += leaves_below;
        }
        self.unused_reserved_peak = self.unused_reserved_peak.max(metrics.unused_reserved_inodes);
        if metrics.unused_reserved_inodes > 0 {
            self.unused_reserved_positive_steps += leaves_below;
        }
    }
}

// ── 调度枚举：逐叶 + 按状态记忆化两种 ───────────────────────────────────────────

/// 记忆化状态数的上限。第一段全部格里最大的一格（P设去锁、D = 2、n = 3）实测 1 330 186 个状态、进程峰值 2.5 GiB；
/// 上限取它的 3 倍，约 7.7 GiB，低于变异跑道的 10G 上限。撞上限说明状态空间变了一个量级（例：变异 V10 让标占一直不撤），
/// 当场断言失败，不让进程无界吃内存。
const MEMO_STATE_LIMIT: usize = 4_000_000;

struct Explorer<'model> {
    model: &'model Model,
    memo: Option<HashMap<ModelState, Aggregate>>,
}

impl<'model> Explorer<'model> {
    fn new(model: &'model Model, use_memo: bool) -> Explorer<'model> {
        Explorer { model, memo: if use_memo { Some(HashMap::new()) } else { None } }
    }

    fn explore(&mut self, state: &ModelState) -> Aggregate {
        let model = self.model;
        // 记忆化的键 = 整个状态：各事务已重算次数也在里面（截断判据要它）。
        let memo_key = *state;
        if let Some(table) = &self.memo {
            if let Some(found) = table.get(&memo_key) {
                return found.clone();
            }
        }
        let mut aggregate = Aggregate::empty();
        let live: Vec<usize> = (0..model.transaction_count)
            .filter(|transaction_index| !state.transactions[*transaction_index].committed)
            .collect();
        if live.is_empty() {
            let metrics = state_metrics(model, state);
            let retries: Vec<u8> = state.transactions[..model.transaction_count]
                .iter()
                .map(|transaction| transaction.retries)
                .collect();
            let fallbacks: u32 = state.transactions[..model.transaction_count]
                .iter()
                .map(|transaction| u32::from(transaction.fallbacks))
                .sum();
            aggregate.add_leaf(&metrics, &retries, fallbacks);
            aggregate.add_state_metrics(&metrics, 1);
        } else {
            let enabled_indices: Vec<usize> = live
                .into_iter()
                .filter(|transaction_index| is_step_enabled(model, state, *transaction_index))
                .collect();
            if enabled_indices.is_empty() {
                aggregate.deadlocks += 1;
            }
            for transaction_index in enabled_indices {
                let mut next = *state;
                match apply_step(model, &mut next, transaction_index) {
                    StepResult::Applied => {
                        let child = self.explore(&next);
                        aggregate.merge(&child);
                    }
                    StepResult::Truncated => aggregate.truncated += 1,
                }
            }
            if aggregate.schedules > 0 {
                let metrics = state_metrics(model, state);
                aggregate.add_state_metrics(&metrics, aggregate.schedules);
            }
        }
        if let Some(table) = &mut self.memo {
            assert!(
                table.len() < MEMO_STATE_LIMIT,
                "记忆化的状态数超过上限 {MEMO_STATE_LIMIT}：模型的状态空间比登记预期的大一个量级，不再往下吃内存"
            );
            table.insert(memo_key, aggregate.clone());
        }
        aggregate
    }
}

struct Enumeration {
    aggregate: Aggregate,
}

fn enumerate_schedules(model: &Model, use_memo: bool) -> Enumeration {
    let mut explorer = Explorer::new(model, use_memo);
    let aggregate = explorer.explore(&initial_state(model));
    Enumeration { aggregate}
}

// ── Q-9：同步轮次 ────────────────────────────────────────────────────────────

fn round_order(transaction_count: usize) -> Vec<usize> {
    (0..transaction_count).collect()
}

const ROUND_LIMIT: usize = 64;

#[derive(Clone, PartialEq, Eq, Debug)]
struct RoundsOutcome {
    /// 全部提交用了几轮；死锁或截断时没有。
    rounds: Option<u32>,
    commits_per_round: Vec<u32>,
}

/// 登记第六节 Q-9：全部 n 个事务第 1 轮同时开始；每一轮里按步序号从小到大逐个相位，相位内按 `round_order` 的次序
/// 让每个停在这一相位且使能的事务走一步；失败回到前面的事务下一轮才再走。
fn simulate_rounds(model: &Model) -> RoundsOutcome {
    simulate_rounds_in_order(model, round_order)
}

fn simulate_rounds_in_order(model: &Model, order_of_transactions: fn(usize) -> Vec<usize>) -> RoundsOutcome {
    let mut state = initial_state(model);
    let mut commits_per_round: Vec<u32> = Vec::new();
    loop {
        let committed_before = state.transactions[..model.transaction_count]
            .iter()
            .filter(|transaction| transaction.committed)
            .count();
        if committed_before == model.transaction_count {
            return RoundsOutcome {
                rounds: Some(u32::try_from(commits_per_round.len()).expect("轮数很小")),
                commits_per_round,
            };
        }
        for phase in 0..model.steps.len() {
            for transaction_index in order_of_transactions(model.transaction_count) {
                let transaction = &state.transactions[transaction_index];
                if !transaction.committed
                    && usize::from(transaction.program_counter) == phase
                    && is_step_enabled(model, &state, transaction_index)
                    && apply_step(model, &mut state, transaction_index) == StepResult::Truncated
                {
                    return RoundsOutcome { rounds: None, commits_per_round };
                }
            }
        }
        let committed_after = state.transactions[..model.transaction_count]
            .iter()
            .filter(|transaction| transaction.committed)
            .count();
        if committed_after == committed_before || commits_per_round.len() >= ROUND_LIMIT {
            return RoundsOutcome { rounds: None, commits_per_round };
        }
        commits_per_round.push(u32::try_from(committed_after - committed_before).expect("提交数很小"));
    }
}

// ── 串行基线 S（`crates/` 今天）：n 个事务按下标一个接一个跑完 ───────────────────────────

fn run_serially(model: &Model) -> ModelState {
    let mut state = initial_state(model);
    for transaction_index in 0..model.transaction_count {
        while !state.transactions[transaction_index].committed {
            assert_eq!(apply_step(model, &mut state, transaction_index), StepResult::Applied);
        }
    }
    state
}

/// Q1乙 的批发布者：全部事务提交之后一步发 G、checkpoint_txg（连同根环槽）与树 ID。
fn batch_publisher_holdings(shared_after_transactions: &SharedState) -> Holdings {
    let mut view = *shared_after_transactions;
    let mut held = NOTHING_HELD;
    take_resources(
        &mut view,
        TransactionShape { user_units: 0, new_inodes: 0 },
        GROUP_COMMIT_SLOTS | GROUP_ROOT_RING | GROUP_TREE,
        &mut held,
    );
    held
}

// ── 钉绝对值的断言（登记第七节）与停机对拍 ───────────────────────────────────────

struct AnchorLine {
    identifier: String,
    observed: String,
    expected: String,
}

impl AnchorLine {
    fn new(identifier: &str, observed: String, expected: &str) -> AnchorLine {
        AnchorLine { identifier: identifier.to_string(), observed, expected: expected.to_string() }
    }

    fn matches(&self) -> bool {
        self.observed == self.expected
    }
}

fn spec_of(
    arm: Arm,
    variant: ArmVariant,
    stage: ParallelStage,
    shape_kind: ShapeKind,
    initial_has_file: bool,
    transaction_count: usize,
    device_count: usize,
) -> CellSpec {
    CellSpec::default_geometry(arm, variant, stage, shape_kind, initial_has_file, transaction_count, device_count)
}

fn should_memoize(spec: &CellSpec) -> bool {
    spec.arm == Arm::DevicePartition
}

fn enumerate_spec(spec: CellSpec) -> Enumeration {
    enumerate_schedules(&Model::new(spec), should_memoize(&spec))
}

fn duplicate_pair(aggregate: &Aggregate, resource: Resource) -> String {
    format!("{}/{}", aggregate.duplicate_total[resource.index()], aggregate.duplicate_schedules[resource.index()])
}

fn slot_text(key: u64) -> String {
    format!("({},{})", key >> 32, key & 0xffff_ffff)
}

fn join_numbers<Number: ToString>(numbers: &[Number]) -> String {
    numbers.iter().map(ToString::to_string).collect::<Vec<String>>().join(",")
}

/// 甲组：串行基线 S 与 Q1 候选的定义（登记第七节甲 1–8）。
fn anchors_from_the_clauses() -> Vec<AnchorLine> {
    let mut lines = Vec::new();
    let full_stage = ParallelStage::ManyPublishesInFlight;
    let serial_model = Model::new(spec_of(
        Arm::WholeLock,
        ArmVariant::Normal,
        full_stage,
        ShapeKind::SameShape,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    ));
    let serial_state = run_serially(&serial_model);
    let held: Vec<Holdings> = serial_state.transactions[..3].iter().map(|transaction| transaction.items).collect();
    let txgs: Vec<u8> = held.iter().map(|each| each.txg).collect();
    lines.push(AnchorLine::new("甲1", join_numbers(&txgs), "6,7,8"));
    for (slots_per_region, expected) in [(4u32, "(0,2),(1,2),(2,2)"), (1u32, "(0,0),(1,0),(2,0)")] {
        let slots: Vec<String> = txgs.iter().map(|txg| slot_text(root_ring_slot_key(*txg, slots_per_region))).collect();
        lines.push(AnchorLine::new(&format!("甲2 S={slots_per_region}"), slots.join(","), expected));
    }
    let counters: Vec<u8> = held.iter().map(|each| each.counter).collect();
    lines.push(AnchorLine::new("甲3 计数器", join_numbers(&counters), "6,7,8"));
    let ring_slots: Vec<String> =
        counters.iter().map(|counter| journal_ring_slot(*counter, DEFAULT_JOURNAL_RING_SLOTS).to_string()).collect();
    lines.push(AnchorLine::new("甲3 环槽", ring_slots.join(","), "5,6,7"));
    lines.push(AnchorLine::new(
        "甲3 默认环槽数",
        (JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES).to_string(),
        &DEFAULT_JOURNAL_RING_SLOTS.to_string(),
    ));
    let transaction_numbers: Vec<u8> = held.iter().map(|each| each.transaction_number).collect();
    lines.push(AnchorLine::new("甲4 事务号", join_numbers(&transaction_numbers), "2,3,4"));
    let counter_minus_transaction: Vec<u8> =
        held.iter().map(|each| each.counter - each.transaction_number).collect();
    lines.push(AnchorLine::new("甲4 计数器减事务号", join_numbers(&counter_minus_transaction), "4,4,4"));
    let inode_starts: Vec<u8> = held.iter().map(|each| each.inode_start).collect();
    lines.push(AnchorLine::new("甲5 inode 号", join_numbers(&inode_starts), "2,3,4"));
    lines.push(AnchorLine::new("甲5 期末水位", serial_state.shared.inode_watermark.to_string(), "5"));
    let user_start_slots: Vec<String> = held
        .iter()
        .map(|each| (u32::from(each.user_pairs[0]) * USER_DATA_SPAN_SLOTS).to_string())
        .collect();
    lines.push(AnchorLine::new("甲7 用户数据落点起点", user_start_slots.join(","), "0,2,4"));
    let commit_nodes: Vec<String> = held
        .iter()
        .map(|each| format!("{{{},{}}}", each.commit_slot_start, each.commit_slot_start + 1))
        .collect();
    lines.push(AnchorLine::new("甲7 提交内生块落点", commit_nodes.join(","), "{0,1},{2,3},{4,5}"));

    let no_file_model = Model::new(spec_of(
        Arm::WholeLock,
        ArmVariant::Normal,
        full_stage,
        ShapeKind::SameShape,
        false,
        3,
        DEFAULT_DEVICE_COUNT,
    ));
    let no_file_state = run_serially(&no_file_model);
    let tree_starts: Vec<TreeIdentifierNumber> =
        no_file_state.transactions[..3].iter().map(|transaction| transaction.items.tree_start).collect();
    lines.push(AnchorLine::new("甲6 发树 ID 的起点（0 = 没发）", join_numbers(&tree_starts), "65536,0,0"));
    // 期末水位的期望取字面量（D8 已定项 11「第一个文件版本那次之后 65548」），不取常量本身：常量错了期望不跟着错。
    lines.push(AnchorLine::new("甲6 期末树 ID 水位", no_file_state.shared.tree_watermark.to_string(), "65548"));
    let mut keys = Vec::new();
    push_held_keys(&no_file_model, Resource::TreeIdentifier, &no_file_state.transactions[0].items, &mut keys);
    lines.push(AnchorLine::new(
        "甲6 新池新建文件发的树 ID",
        keys.iter().map(u64::to_string).collect::<Vec<String>>().join(","),
        "65536,65537,65538,65539,65540,65541,65542,65543,65544,65545,65546,65547",
    ));

    let one_publish_model = Model::new(spec_of(
        Arm::WholeLock,
        ArmVariant::Normal,
        ParallelStage::OnePublishCarriesManyTransactions,
        ShapeKind::SameShape,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    ));
    let one_publish_state = run_serially(&one_publish_model);
    let batch = batch_publisher_holdings(&one_publish_state.shared);
    let transactions_with_own_txg =
        one_publish_state.transactions[..3].iter().filter(|transaction| transaction.items.txg != 0).count();
    lines.push(AnchorLine::new(
        "甲8 一个 txg 与一个根环槽",
        format!(
            "batch_txg={} batch_root_slot={} transactions_with_own_txg={}",
            batch.txg,
            slot_text(root_ring_slot_key(batch.txg, DEFAULT_ROOT_SLOTS_PER_REGION)),
            transactions_with_own_txg
        ),
        "batch_txg=6 batch_root_slot=(0,2) transactions_with_own_txg=0",
    ));
    lines
}

fn whole_copy_controls() -> [(Arm, ArmVariant); 3] {
    [
        (Arm::WholeLock, ArmVariant::PositiveControl),
        (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl),
        (Arm::OptimisticReadSet, ArmVariant::PositiveControl),
    ]
}

/// 乙组与补充锚点：设计时用命令独立算出的数（登记第七节乙 1–7；补 1 是执行员另写 Python 算的 A字面 锁外两列）。
fn anchors_from_independent_calculations() -> Vec<AnchorLine> {
    let mut lines = Vec::new();
    // 乙 1：整份写回，同形，初始态有文件，Q1甲 的资源集合。
    for (arm, variant) in whole_copy_controls() {
        for (transaction_count, expected) in [
            (2, "schedules=6 U=4/4 G=8/4 I=4/4 X=4/4"),
            (3, "schedules=90 U=120/84 G=240/84 I=120/84 X=120/84"),
        ] {
            let enumeration = enumerate_spec(spec_of(
                arm,
                variant,
                ParallelStage::PrepareParallelCommitSerial,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let aggregate = &enumeration.aggregate;
            lines.push(AnchorLine::new(
                &format!("乙1 {} n={transaction_count}", arm.label(variant)),
                format!(
                    "schedules={} U={} G={} I={} X={}",
                    aggregate.schedules,
                    duplicate_pair(aggregate, Resource::UserDataSlot),
                    duplicate_pair(aggregate, Resource::CommitGeneratedSlot),
                    duplicate_pair(aggregate, Resource::InodeNumber),
                    duplicate_pair(aggregate, Resource::TransactionNumber)
                ),
                expected,
            ));
        }
        // 乙 1 末句的结构延伸（登记 5.3 末句）：Q1丙 下单号资源各自等于「单件」那一行、G 等于「两件」那一行；
        // Q1乙 下 G、TXG、RS、T 归批发布者、不争用，所以是 0。
        for (transaction_count, single, double) in [(2usize, "4/4", "8/4"), (3, "120/84", "240/84")] {
            let full_stage = enumerate_spec(spec_of(
                arm,
                variant,
                ParallelStage::ManyPublishesInFlight,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let mut observed = Vec::new();
            for resource in [
                Resource::UserDataSlot,
                Resource::InodeNumber,
                Resource::TransactionNumber,
                Resource::JsnCounter,
                Resource::JournalRingSlot,
                Resource::CheckpointTxg,
                Resource::RootRingSlot,
            ] {
                observed.push(format!("{}={}", resource.label(), duplicate_pair(&full_stage.aggregate, resource)));
            }
            observed.push(format!("G={}", duplicate_pair(&full_stage.aggregate, Resource::CommitGeneratedSlot)));
            let expected_text = format!(
                "U={single} I={single} X={single} J={single} JS={single} TXG={single} RS={single} G={double}"
            );
            lines.push(AnchorLine::new(
                &format!("乙1 Q1丙 {} n={transaction_count}", arm.label(variant)),
                observed.join(" "),
                &expected_text,
            ));
            let one_publish = enumerate_spec(spec_of(
                arm,
                variant,
                ParallelStage::OnePublishCarriesManyTransactions,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let mut observed_one_publish = Vec::new();
            for resource in [
                Resource::UserDataSlot,
                Resource::InodeNumber,
                Resource::TransactionNumber,
                Resource::JsnCounter,
                Resource::JournalRingSlot,
                Resource::CommitGeneratedSlot,
                Resource::CheckpointTxg,
                Resource::RootRingSlot,
                Resource::TreeIdentifier,
            ] {
                observed_one_publish
                    .push(format!("{}={}", resource.label(), duplicate_pair(&one_publish.aggregate, resource)));
            }
            let expected_one_publish = format!(
                "U={single} I={single} X={single} J={single} JS={single} G=0/0 TXG=0/0 RS=0/0 T=0/0"
            );
            lines.push(AnchorLine::new(
                &format!("乙1 Q1乙 {} n={transaction_count}", arm.label(variant)),
                observed_one_publish.join(" "),
                &expected_one_publish,
            ));
        }
    }
    // 乙 2：整份写回与 P区同，初始态无文件，树 ID 一列（Q1甲）。
    let tree_controls = [
        (Arm::WholeLock, ArmVariant::PositiveControl),
        (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl),
        (Arm::OptimisticReadSet, ArmVariant::PositiveControl),
        (Arm::RegionPartition, ArmVariant::PositiveControl),
    ];
    for (arm, variant) in tree_controls {
        for (transaction_count, expected) in [(2, "T=48/4"), (3, "T=1296/72")] {
            let enumeration = enumerate_spec(spec_of(
                arm,
                variant,
                ParallelStage::PrepareParallelCommitSerial,
                ShapeKind::SameShape,
                false,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            lines.push(AnchorLine::new(
                &format!("乙2 {} n={transaction_count}", arm.label(variant)),
                format!("T={}", duplicate_pair(&enumeration.aggregate, Resource::TreeIdentifier)),
                expected,
            ));
        }
    }
    // 乙 3：P区同，同形，有文件。
    for (transaction_count, expected) in [
        (2, "schedules=6 U=4/4 G=8/4 I=6/6 X=6/6"),
        (3, "schedules=90 U=120/84 G=240/84 I=180/90 X=180/90"),
    ] {
        let enumeration = enumerate_spec(spec_of(
            Arm::RegionPartition,
            ArmVariant::PositiveControl,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            true,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let aggregate = &enumeration.aggregate;
        lines.push(AnchorLine::new(
            &format!("乙3 P区同 n={transaction_count}"),
            format!(
                "schedules={} U={} G={} I={} X={}",
                aggregate.schedules,
                duplicate_pair(aggregate, Resource::UserDataSlot),
                duplicate_pair(aggregate, Resource::CommitGeneratedSlot),
                duplicate_pair(aggregate, Resource::InodeNumber),
                duplicate_pair(aggregate, Resource::TransactionNumber)
            ),
            expected,
        ));
        // J、JS、TXG、RS 在它们争用的 Q1 候选下等于 I 那一列。
        let i_column = if transaction_count == 2 { "6/6" } else { "180/90" };
        for (stage, resources) in [
            (ParallelStage::OnePublishCarriesManyTransactions, vec![Resource::JsnCounter, Resource::JournalRingSlot]),
            (
                ParallelStage::ManyPublishesInFlight,
                vec![Resource::JsnCounter, Resource::JournalRingSlot, Resource::CheckpointTxg, Resource::RootRingSlot],
            ),
        ] {
            let contended = enumerate_spec(spec_of(
                Arm::RegionPartition,
                ArmVariant::PositiveControl,
                stage,
                ShapeKind::SameShape,
                true,
                transaction_count,
                DEFAULT_DEVICE_COUNT,
            ));
            let observed: Vec<String> = resources
                .iter()
                .map(|resource| format!("{}={}", resource.label(), duplicate_pair(&contended.aggregate, *resource)))
                .collect();
            let expected_text: Vec<String> =
                resources.iter().map(|resource| format!("{}={}", resource.label(), i_column)).collect();
            lines.push(AnchorLine::new(
                &format!("乙3 P区同 {} n={transaction_count}", stage.label()),
                observed.join(" "),
                &expected_text.join(" "),
            ));
        }
    }
    // 乙 4 / 乙 5：O整 的重复数恒 0；重算（调度数、重算合计、有重算的调度数、单调度最大重算）。
    for (transaction_count, expected) in [(2, "schedules=6 retries=4 retry_schedules=4 max_retries=1"), (3, "schedules=498 retries=1140 retry_schedules=492 max_retries=3")] {
        let mut every_cell_matches = true;
        let mut nonzero_duplicate_cells = 0;
        let mut first_observed = String::new();
        for stage in ALL_STAGES {
            for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                for initial_has_file in [true, false] {
                    let enumeration = enumerate_spec(spec_of(
                        Arm::OptimisticWholeVersion,
                        ArmVariant::Normal,
                        stage,
                        shape_kind,
                        initial_has_file,
                        transaction_count,
                        DEFAULT_DEVICE_COUNT,
                    ));
                    let aggregate = &enumeration.aggregate;
                    let observed = format!(
                        "schedules={} retries={} retry_schedules={} max_retries={}",
                        aggregate.schedules,
                        aggregate.retries_total,
                        aggregate.retries_schedules,
                        aggregate.retries_max_schedule
                    );
                    if first_observed.is_empty() {
                        first_observed = observed.clone();
                    }
                    every_cell_matches &= observed == first_observed;
                    if aggregate.duplicate_total.iter().any(|total| *total != 0) {
                        nonzero_duplicate_cells += 1;
                    }
                }
            }
        }
        lines.push(AnchorLine::new(&format!("乙4 O整 重复数全 0 的格外个数 n={transaction_count}"), nonzero_duplicate_cells.to_string(), "0"));
        lines.push(AnchorLine::new(
            &format!("乙5 O整 重算（12 格逐格相同）n={transaction_count}"),
            if every_cell_matches { first_observed } else { "12 格不全相同".to_string() },
            expected,
        ));
    }
    // 乙 6：A字面去锁（四步），同形，Q1甲，U 一列。
    for (transaction_count, expected) in [(2, "schedules=70 U_total=36 U_schedules=36"), (3, "schedules=34650 U_total=37368 U_schedules=29268")] {
        let enumeration = enumerate_spec(spec_of(
            Arm::LiteralLock,
            ArmVariant::PositiveControl,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            true,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let aggregate = &enumeration.aggregate;
        lines.push(AnchorLine::new(
            &format!("乙6 A字面去锁 n={transaction_count}"),
            format!(
                "schedules={} U_total={} U_schedules={}",
                aggregate.schedules,
                aggregate.duplicate_total[Resource::UserDataSlot.index()],
                aggregate.duplicate_schedules[Resource::UserDataSlot.index()]
            ),
            expected,
        ));
    }
    // 乙 7：P设 的 U 一列，同形，重算上限 2。
    let device_partition_expectations: [(ArmVariant, usize, usize, &str); 8] = [
        (ArmVariant::Normal, 1, 2, "70,0,0,0,0,0"),
        (ArmVariant::Normal, 1, 3, "34650,0,0,0,0,0"),
        (ArmVariant::Normal, 2, 2, "14982,53010,56376,3366,0,0"),
        (ArmVariant::Normal, 2, 3, "1029677534586,6043544394480,6199013074122,155468679642,0,0"),
        (ArmVariant::PositiveControl, 1, 2, "252,0,0,0,120,120"),
        (ArmVariant::PositiveControl, 1, 3, "756756,0,0,0,767880,616680"),
        (ArmVariant::PositiveControl, 2, 2, "2510072,9546090,11143686,1597596,1516320,758160"),
        (
            ArmVariant::PositiveControl,
            2,
            3,
            "15204801979811387676,90958422775773687714,102730730512623627846,11772307736849940132,21393243243591046560,9560362788127086480",
        ),
    ];
    for (variant, device_count, transaction_count, expected) in device_partition_expectations {
        let enumeration = enumerate_spec(spec_of(
            Arm::DevicePartition,
            variant,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            true,
            transaction_count,
            device_count,
        ));
        lines.push(AnchorLine::new(
            &format!("乙7 {} D={device_count} n={transaction_count}", Arm::DevicePartition.label(variant)),
            device_partition_tuple(&enumeration.aggregate),
            expected,
        ));
    }
    // 补 1：A字面 锁外两列（事务号、树 ID），Q1甲，同形；独立 Python（`literal_outside_anchor.py`）算出。
    for (initial_has_file, transaction_count, expected) in [
        (true, 2, "schedules=20 X=18/18 T=0/0"),
        (true, 3, "schedules=1680 X=2970/1674 T=0/0"),
        (false, 2, "schedules=20 X=18/18 T=216/18"),
        (false, 3, "schedules=1680 X=2970/1674 T=34992/1620"),
    ] {
        let enumeration = enumerate_spec(spec_of(
            Arm::LiteralLock,
            ArmVariant::Normal,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            initial_has_file,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let aggregate = &enumeration.aggregate;
        lines.push(AnchorLine::new(
            &format!("补1 A字面 锁外 file={initial_has_file} n={transaction_count}"),
            format!(
                "schedules={} X={} T={}",
                aggregate.schedules,
                duplicate_pair(aggregate, Resource::TransactionNumber),
                duplicate_pair(aggregate, Resource::TreeIdentifier)
            ),
            expected,
        ));
    }
    lines
}

/// P设 的 U 一列按乙 7 口径：（调度数，重算合计，不一致合计，截断数，重复合计，有重复的调度数）。
fn device_partition_tuple(aggregate: &Aggregate) -> String {
    format!(
        "{},{},{},{},{},{}",
        aggregate.schedules,
        aggregate.retries_total,
        aggregate.retries_total + aggregate.truncated,
        aggregate.truncated,
        aggregate.duplicate_total[Resource::UserDataSlot.index()],
        aggregate.duplicate_schedules[Resource::UserDataSlot.index()]
    )
}

fn factorial(value: u32) -> u128 {
    (1..=u128::from(value)).product()
}

/// 丙 1：没有重算、没有 FALLBACK 的臂，调度数 = (n·k)! ÷ (k!)ⁿ。
fn schedule_count_without_retries(transaction_count: u32, steps_per_transaction: u32) -> u128 {
    factorial(transaction_count * steps_per_transaction) / factorial(steps_per_transaction).pow(transaction_count)
}

/// 丙组：结构性计数。
fn anchors_from_structural_counts() -> Vec<AnchorLine> {
    let mut lines = Vec::new();
    let stage = ParallelStage::PrepareParallelCommitSerial;
    let cases: [(Arm, ArmVariant, usize, u32); 10] = [
        (Arm::WholeLock, ArmVariant::Normal, DEFAULT_DEVICE_COUNT, 1),
        (Arm::RegionPartition, ArmVariant::Normal, DEFAULT_DEVICE_COUNT, 2),
        (Arm::RegionPartition, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::WholeLock, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::OptimisticReadSet, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 2),
        (Arm::LiteralLock, ArmVariant::Normal, DEFAULT_DEVICE_COUNT, 3),
        (Arm::LiteralLock, ArmVariant::PositiveControl, DEFAULT_DEVICE_COUNT, 4),
        (Arm::DevicePartition, ArmVariant::Normal, SAMPLE_DEVICE_COUNT, 4),
        (Arm::DevicePartition, ArmVariant::PositiveControl, SAMPLE_DEVICE_COUNT, 5),
    ];
    for (arm, variant, device_count, steps_per_transaction) in cases {
        for transaction_count in [2usize, 3] {
            let enumeration = enumerate_spec(spec_of(
                arm,
                variant,
                stage,
                ShapeKind::SameShape,
                true,
                transaction_count,
                device_count,
            ));
            let expected = schedule_count_without_retries(
                u32::try_from(transaction_count).expect("n 很小"),
                steps_per_transaction,
            );
            lines.push(AnchorLine::new(
                &format!("丙1 {} k={steps_per_transaction} n={transaction_count}", arm.label(variant)),
                enumeration.aggregate.schedules.to_string(),
                &expected.to_string(),
            ));
        }
    }
    // 丙 2：串行基线 S 的调度数 1，重复数 0，重算 0。
    for transaction_count in [2usize, 3] {
        let model = Model::new(spec_of(
            Arm::WholeLock,
            ArmVariant::Normal,
            ParallelStage::ManyPublishesInFlight,
            ShapeKind::SameShape,
            true,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        ));
        let state = run_serially(&model);
        let metrics = state_metrics(&model, &state);
        let retries: u32 = state.transactions[..transaction_count].iter().map(|each| u32::from(each.retries)).sum();
        lines.push(AnchorLine::new(
            &format!("丙2 串行基线 n={transaction_count}"),
            format!("duplicates={} retries={}", metrics.duplicates.iter().sum::<u32>(), retries),
            "duplicates=0 retries=0",
        ));
    }
    // 丙 3：P设 的记忆化计数与逐叶枚举逐项相等（整份汇总，含轨迹，n = 2）。
    for variant in [ArmVariant::Normal, ArmVariant::PositiveControl] {
        for device_count in [1usize, 2] {
            let model = Model::new(spec_of(
                Arm::DevicePartition,
                variant,
                stage,
                ShapeKind::SameShape,
                true,
                2,
                device_count,
            ));
            let leafwise = enumerate_schedules(&model, false);
            let memoized = enumerate_schedules(&model, true);
            lines.push(AnchorLine::new(
                &format!("丙3 {} D={device_count} n=2 记忆化 = 逐叶", Arm::DevicePartition.label(variant)),
                (leafwise.aggregate == memoized.aggregate).to_string(),
                "true",
            ));
        }
    }
    lines
}

fn all_anchor_lines() -> Vec<AnchorLine> {
    let mut lines = anchors_from_the_clauses();
    lines.extend(anchors_from_independent_calculations());
    lines.extend(anchors_from_structural_counts());
    lines
}

// ── 产物：每格的结果行 ─────────────────────────────────────────────────────

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn initial_label(initial_has_file: bool) -> &'static str {
    if initial_has_file {
        "file"
    } else {
        "nofile"
    }
}

fn cell_prefix(name: &str, spec: &CellSpec) -> String {
    format!(
        "name={name} arm={} stage={} shape={} initial={} transactions={} devices={}",
        spec.arm.label(spec.variant),
        spec.stage.label(),
        spec.shape_kind.label(),
        initial_label(spec.initial_has_file),
        spec.transaction_count,
        spec.device_count
    )
}

fn cell_line(name: &str, spec: &CellSpec, aggregate: &Aggregate) -> String {
    let mut line = cell_prefix(name, spec);
    write!(line, " schedules={}", aggregate.schedules).expect("写进 String 不会失败");
    for resource in ALL_RESOURCES {
        let index = resource.index();
        write!(
            line,
            " {label}_total={} {label}_schedules={} {label}_max_single={}",
            aggregate.duplicate_total[index],
            aggregate.duplicate_schedules[index],
            aggregate.duplicate_max_single_schedule[index],
            label = resource.label()
        )
        .expect("写进 String 不会失败");
    }
    write!(
        line,
        " retries_total={} retries_schedules={} retries_max_schedule={} retries_max_transaction={} \
         device_disagreements={} truncated={} fallbacks_total={} deadlocks={}",
        aggregate.retries_total,
        aggregate.retries_schedules,
        aggregate.retries_max_schedule,
        aggregate.retries_max_transaction,
        if spec.arm == Arm::DevicePartition { aggregate.retries_total + aggregate.truncated } else { 0 },
        aggregate.truncated,
        aggregate.fallbacks_total,
        aggregate.deadlocks
    )
    .expect("写进 String 不会失败");
    line
}

fn margin_text(value: i32) -> String {
    if value == i32::MAX {
        "none".to_string()
    } else {
        value.to_string()
    }
}

fn trajectory_line(spec: &CellSpec, aggregate: &Aggregate) -> String {
    let mut line = cell_prefix("trajectory", spec);
    for resource in ALL_RESOURCES {
        let index = resource.index();
        write!(
            line,
            " {label}_peak={} {label}_positive_steps={}",
            aggregate.duplicate_peak[index],
            aggregate.duplicate_positive_steps[index],
            label = resource.label()
        )
        .expect("写进 String 不会失败");
    }
    write!(
        line,
        " inode_margin_minimum={} inode_margin_negative_steps={} inode_margin_final_minimum={} \
         busy_regions_peak={} busy_regions_positive_steps={} busy_regions_final_maximum={} \
         unused_reserved_peak={} unused_reserved_positive_steps={} unused_reserved_final_maximum={}",
        margin_text(aggregate.inode_margin_minimum),
        aggregate.inode_margin_negative_steps,
        margin_text(aggregate.inode_margin_final_minimum),
        aggregate.busy_regions_peak,
        aggregate.busy_regions_positive_steps,
        aggregate.busy_regions_final_maximum,
        aggregate.unused_reserved_peak,
        aggregate.unused_reserved_positive_steps,
        aggregate.unused_reserved_final_maximum
    )
    .expect("写进 String 不会失败");
    line
}

fn counts_text(counts: &[u32]) -> String {
    if counts.is_empty() {
        "none".to_string()
    } else {
        counts.iter().map(u32::to_string).collect::<Vec<String>>().join("/")
    }
}

/// A 与 A字面 的锁里装了多少计划是 Q1 的测量，登记第六节 Q-9 不对它们报 R_n。
fn reports_rounds(spec: &CellSpec) -> bool {
    match spec.arm {
        Arm::WholeLock | Arm::LiteralLock => false,
        Arm::RegionPartition | Arm::DevicePartition | Arm::OptimisticWholeVersion | Arm::OptimisticReadSet => true,
    }
}

fn rounds_line(spec: &CellSpec, outcome: &RoundsOutcome) -> String {
    let mut line = cell_prefix("rounds", spec);
    let serial_rounds = spec.transaction_count;
    write!(
        line,
        " rounds={} serial_rounds={serial_rounds} commits_per_round={} peak_commits_in_a_round={} rounds_with_commits={} last_round_commits={} \
         parallel_gain_seen_in_model={}",
        outcome.rounds.map_or("none".to_string(), |rounds| rounds.to_string()),
        counts_text(&outcome.commits_per_round),
        outcome.commits_per_round.iter().copied().max().unwrap_or(0),
        outcome.commits_per_round.iter().filter(|commits| **commits > 0).count(),
        outcome.commits_per_round.last().copied().unwrap_or(0),
        outcome
            .rounds
            .is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") < serial_rounds)
    )
    .expect("写进 String 不会失败");
    line
}

struct CellResult {
    spec: CellSpec,
    aggregate: Aggregate,
    rounds: RoundsOutcome,
}

fn run_cell(spec: CellSpec) -> CellResult {
    let model = Model::new(spec);
    let enumeration = enumerate_schedules(&model, should_memoize(&spec));
    let rounds = simulate_rounds(&model);
    CellResult { spec, aggregate: enumeration.aggregate, rounds }
}

fn arm_cell_specs(device_count: usize) -> Vec<CellSpec> {
    let mut specs = Vec::new();
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                for initial_has_file in [true, false] {
                    for transaction_count in [2usize, 3] {
                        specs.push(spec_of(
                            arm,
                            ArmVariant::Normal,
                            stage,
                            shape_kind,
                            initial_has_file,
                            transaction_count,
                            device_count,
                        ));
                    }
                }
            }
        }
    }
    specs
}

fn control_cell_specs(device_count: usize) -> Vec<CellSpec> {
    let mut specs = Vec::new();
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            for initial_has_file in [true, false] {
                for transaction_count in [2usize, 3] {
                    specs.push(spec_of(
                        arm,
                        ArmVariant::PositiveControl,
                        stage,
                        ShapeKind::SameShape,
                        initial_has_file,
                        transaction_count,
                        device_count,
                    ));
                }
            }
        }
    }
    specs
}

// ── 判定行 ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Question {
    UserDataSlots,
    CommitGeneratedSlots,
    InodeNumbers,
    TreeIdentifiers,
    TransactionNumbers,
    JsnAndJournalSlots,
    CheckpointTxgAndRootRingSlots,
    ConflictRetries,
    SynchronousRounds,
}

const DUPLICATE_QUESTIONS: [Question; 7] = [
    Question::UserDataSlots,
    Question::CommitGeneratedSlots,
    Question::InodeNumbers,
    Question::TreeIdentifiers,
    Question::TransactionNumbers,
    Question::JsnAndJournalSlots,
    Question::CheckpointTxgAndRootRingSlots,
];

impl Question {
    fn label(self) -> &'static str {
        match self {
            Question::UserDataSlots => "Q-1",
            Question::CommitGeneratedSlots => "Q-2",
            Question::InodeNumbers => "Q-3",
            Question::TreeIdentifiers => "Q-4",
            Question::TransactionNumbers => "Q-5",
            Question::JsnAndJournalSlots => "Q-6",
            Question::CheckpointTxgAndRootRingSlots => "Q-7",
            Question::ConflictRetries => "Q-8",
            Question::SynchronousRounds => "Q-9",
        }
    }

    fn resources(self) -> Vec<Resource> {
        match self {
            Question::UserDataSlots => vec![Resource::UserDataSlot],
            Question::CommitGeneratedSlots => vec![Resource::CommitGeneratedSlot],
            Question::InodeNumbers => vec![Resource::InodeNumber],
            Question::TreeIdentifiers => vec![Resource::TreeIdentifier],
            Question::TransactionNumbers => vec![Resource::TransactionNumber],
            Question::JsnAndJournalSlots => vec![Resource::JsnCounter, Resource::JournalRingSlot],
            Question::CheckpointTxgAndRootRingSlots => vec![Resource::CheckpointTxg, Resource::RootRingSlot],
            Question::ConflictRetries | Question::SynchronousRounds => Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Judgement {
    /// 登记第六节最后一列没列：这一格的结果不能从那条臂的定义直接推出，拿来判。
    Judged,
    /// 登记第六节最后一列列了：能从定义直接推出，照报、当锚点，不判。
    DefinitionImplied,
    /// 登记第四节丙：跑前已经算出整格。
    KnownBeforeRun,
    NotReported,
}

impl Judgement {
    fn label(self) -> &'static str {
        match self {
            Judgement::Judged => "judged",
            Judgement::DefinitionImplied => "definition_implied",
            Judgement::KnownBeforeRun => "known_before_run",
            Judgement::NotReported => "not_reported",
        }
    }
}

/// 登记第六节表最后一列「定义推出、不判的格」逐行落成代码。
fn judgement_of(question: Question, arm: Arm) -> Judgement {
    match (question, arm) {
        (Question::UserDataSlots, Arm::DevicePartition) => Judgement::KnownBeforeRun,
        (Question::UserDataSlots, Arm::LiteralLock) => Judgement::Judged,
        (Question::UserDataSlots, Arm::WholeLock)
        | (Question::UserDataSlots, Arm::RegionPartition)
        | (Question::UserDataSlots, Arm::OptimisticWholeVersion)
        | (Question::UserDataSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::CommitGeneratedSlots, Arm::LiteralLock) | (Question::CommitGeneratedSlots, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::CommitGeneratedSlots, Arm::WholeLock)
        | (Question::CommitGeneratedSlots, Arm::RegionPartition)
        | (Question::CommitGeneratedSlots, Arm::OptimisticWholeVersion)
        | (Question::CommitGeneratedSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::InodeNumbers, Arm::LiteralLock) | (Question::InodeNumbers, Arm::DevicePartition) => Judgement::Judged,
        (Question::InodeNumbers, Arm::WholeLock)
        | (Question::InodeNumbers, Arm::RegionPartition)
        | (Question::InodeNumbers, Arm::OptimisticWholeVersion)
        | (Question::InodeNumbers, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::TreeIdentifiers, Arm::LiteralLock)
        | (Question::TreeIdentifiers, Arm::RegionPartition)
        | (Question::TreeIdentifiers, Arm::DevicePartition) => Judgement::Judged,
        (Question::TreeIdentifiers, Arm::WholeLock)
        | (Question::TreeIdentifiers, Arm::OptimisticWholeVersion)
        | (Question::TreeIdentifiers, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::TransactionNumbers, Arm::LiteralLock) | (Question::TransactionNumbers, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::TransactionNumbers, Arm::WholeLock)
        | (Question::TransactionNumbers, Arm::RegionPartition)
        | (Question::TransactionNumbers, Arm::OptimisticWholeVersion)
        | (Question::TransactionNumbers, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::JsnAndJournalSlots, Arm::LiteralLock) | (Question::JsnAndJournalSlots, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::JsnAndJournalSlots, Arm::WholeLock)
        | (Question::JsnAndJournalSlots, Arm::RegionPartition)
        | (Question::JsnAndJournalSlots, Arm::OptimisticWholeVersion)
        | (Question::JsnAndJournalSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::CheckpointTxgAndRootRingSlots, Arm::LiteralLock)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::DevicePartition) => Judgement::Judged,
        (Question::CheckpointTxgAndRootRingSlots, Arm::WholeLock)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::RegionPartition)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::OptimisticWholeVersion)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::ConflictRetries, Arm::LiteralLock) | (Question::ConflictRetries, Arm::OptimisticReadSet) => {
            Judgement::Judged
        }
        (Question::ConflictRetries, Arm::DevicePartition) => Judgement::KnownBeforeRun,
        (Question::ConflictRetries, Arm::WholeLock)
        | (Question::ConflictRetries, Arm::RegionPartition)
        | (Question::ConflictRetries, Arm::OptimisticWholeVersion) => Judgement::DefinitionImplied,
        (Question::SynchronousRounds, Arm::OptimisticReadSet) | (Question::SynchronousRounds, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::SynchronousRounds, Arm::OptimisticWholeVersion) | (Question::SynchronousRounds, Arm::RegionPartition) => {
            Judgement::DefinitionImplied
        }
        (Question::SynchronousRounds, Arm::WholeLock) | (Question::SynchronousRounds, Arm::LiteralLock) => {
            Judgement::NotReported
        }
    }
}

fn cells_of<'results>(
    results: &'results [CellResult],
    arm: Arm,
    stage: ParallelStage,
) -> Vec<&'results CellResult> {
    results.iter().filter(|result| result.spec.arm == arm && result.spec.stage == stage).collect()
}

fn duplicates_of_question(result: &CellResult, question: Question) -> u128 {
    question
        .resources()
        .iter()
        .map(|resource| result.aggregate.duplicate_total[resource.index()])
        .sum()
}

fn emit_question_verdicts(emitter: &mut Emitter, default_cells: &[CellResult]) {
    for question in DUPLICATE_QUESTIONS {
        for arm in ALL_ARMS {
            for stage in ALL_STAGES {
                let cells = cells_of(default_cells, arm, stage);
                let cells_with_duplicates =
                    cells.iter().filter(|result| duplicates_of_question(result, question) > 0).count();
                let duplicate_total: u128 = cells.iter().map(|result| duplicates_of_question(result, question)).sum();
                emit(
                    emitter,
                    &format!(
                        "name=verdict question={} arm={} stage={} judgement={} cells={} cells_with_duplicates={} \
                         duplicate_total={} no_duplicate_in_any_cell={}",
                        question.label(),
                        arm.label(ArmVariant::Normal),
                        stage.label(),
                        judgement_of(question, arm).label(),
                        cells.len(),
                        cells_with_duplicates,
                        duplicate_total,
                        cells_with_duplicates == 0
                    ),
                );
            }
        }
    }
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            let cells = cells_of(default_cells, arm, stage);
            let retries_total: u128 = cells.iter().map(|result| result.aggregate.retries_total).sum();
            let cells_with_retries = cells.iter().filter(|result| result.aggregate.retries_total > 0).count();
            let truncated: u128 = cells.iter().map(|result| result.aggregate.truncated).sum();
            let fallbacks: u128 = cells.iter().map(|result| result.aggregate.fallbacks_total).sum();
            let deadlocks: u128 = cells.iter().map(|result| result.aggregate.deadlocks).sum();
            emit(
                emitter,
                &format!(
                    "name=verdict question=Q-8 arm={} stage={} judgement={} cells={} retries_total={} cells_with_retries={} \
                     device_truncated={} region_fallbacks={} region_deadlocks={}",
                    arm.label(ArmVariant::Normal),
                    stage.label(),
                    judgement_of(Question::ConflictRetries, arm).label(),
                    cells.len(),
                    retries_total,
                    cells_with_retries,
                    truncated,
                    fallbacks,
                    deadlocks
                ),
            );
        }
    }
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            let cells = cells_of(default_cells, arm, stage);
            let judgement = judgement_of(Question::SynchronousRounds, arm);
            if judgement == Judgement::NotReported {
                emit(
                    emitter,
                    &format!(
                        "name=verdict question=Q-9 arm={} stage={} judgement={} cells={} cells_with_fewer_rounds_than_serial=not_reported",
                        arm.label(ArmVariant::Normal),
                        stage.label(),
                        judgement.label(),
                        cells.len()
                    ),
                );
                continue;
            }
            let fewer_rounds = cells
                .iter()
                .filter(|result| {
                    result
                        .rounds
                        .rounds
                        .is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") < result.spec.transaction_count)
                })
                .count();
            let without_rounds = cells.iter().filter(|result| result.rounds.rounds.is_none()).count();
            emit(
                emitter,
                &format!(
                    "name=verdict question=Q-9 arm={} stage={} judgement={} cells={} cells_with_fewer_rounds_than_serial={} \
                     cells_without_round_count={} rounds_equal_serial_in_every_cell={}",
                    arm.label(ArmVariant::Normal),
                    stage.label(),
                    judgement.label(),
                    cells.len(),
                    fewer_rounds,
                    without_rounds,
                    fewer_rounds == 0 && without_rounds == 0
                ),
            );
        }
    }
}

fn find_cell<'results>(
    results: &'results [CellResult],
    spec: &CellSpec,
) -> Option<&'results CellResult> {
    results.iter().find(|result| {
        result.spec.arm == spec.arm
            && result.spec.variant == spec.variant
            && result.spec.stage == spec.stage
            && result.spec.shape_kind == spec.shape_kind
            && result.spec.initial_has_file == spec.initial_has_file
            && result.spec.transaction_count == spec.transaction_count
    })
}

/// 登记第十节失败条款 1–9 逐条按产物里的格算一次（`triggered` = 条款的前件满足）。
fn emit_failure_clauses(emitter: &mut Emitter, default_cells: &[CellResult]) {
    let nonzero_duplicates = |result: &CellResult| result.aggregate.duplicate_total.iter().any(|total| *total > 0);
    let whole_lock_default_duplicates = default_cells
        .iter()
        .filter(|result| result.spec.arm == Arm::WholeLock)
        .filter(|result| nonzero_duplicates(result))
        .count();
    emit(
        emitter,
        &format!("name=verdict failure=2 clause=A_default_geometry_duplicate triggered={}", whole_lock_default_duplicates > 0),
    );
    let all_duplicates_zero = default_cells.iter().all(|result| !nonzero_duplicates(result));
    let read_set_equals_whole_everywhere = default_cells
        .iter()
        .filter(|result| result.spec.arm == Arm::OptimisticReadSet)
        .all(|read_set_cell| {
            let whole_spec = CellSpec { arm: Arm::OptimisticWholeVersion, ..read_set_cell.spec };
            find_cell(default_cells, &whole_spec)
                .is_some_and(|whole_cell| whole_cell.aggregate.retries_total == read_set_cell.aggregate.retries_total)
        });
    emit(
        emitter,
        &format!(
            "name=verdict failure=4 clause=ruler_cannot_separate_candidates triggered={}",
            all_duplicates_zero && read_set_equals_whole_everywhere
        ),
    );
    let mut read_set_differs_in_stage_one_or_three = 0;
    let mut read_set_below_whole_in_stage_two_same_shape = 0;
    for read_set_cell in default_cells.iter().filter(|result| result.spec.arm == Arm::OptimisticReadSet) {
        let whole_spec = CellSpec { arm: Arm::OptimisticWholeVersion, ..read_set_cell.spec };
        let whole_cell = find_cell(default_cells, &whole_spec).expect("每个 O读 格都有同参数的 O整 格");
        let stage_uses_base_version = read_set_cell.spec.stage.base_version_in_read_set();
        if stage_uses_base_version && whole_cell.aggregate.retries_total != read_set_cell.aggregate.retries_total {
            read_set_differs_in_stage_one_or_three += 1;
        }
        if read_set_cell.spec.stage == ParallelStage::OnePublishCarriesManyTransactions
            && read_set_cell.spec.shape_kind == ShapeKind::SameShape
            && read_set_cell.aggregate.retries_total < whole_cell.aggregate.retries_total
        {
            read_set_below_whole_in_stage_two_same_shape += 1;
        }
    }
    emit(
        emitter,
        &format!(
            "name=verdict failure=5 clause=read_set_not_as_registered triggered={}",
            read_set_differs_in_stage_one_or_three > 0
        ),
    );
    let device_cells_with_truncation = default_cells
        .iter()
        .filter(|result| result.spec.arm == Arm::DevicePartition)
        .filter(|result| result.aggregate.truncated > 0)
        .count();
    emit(
        emitter,
        &format!(
            "name=verdict failure=6 clause=device_partition_does_not_converge triggered={} cells_with_truncation={}",
            device_cells_with_truncation > 0,
            device_cells_with_truncation
        ),
    );
    let literal_x_or_t = default_cells
        .iter()
        .filter(|result| result.spec.arm == Arm::LiteralLock)
        .filter(|result| {
            result.aggregate.duplicate_total[Resource::TreeIdentifier.index()] > 0
                || result.aggregate.duplicate_total[Resource::TransactionNumber.index()] > 0
        })
        .count();
    emit(
        emitter,
        &format!(
            "name=verdict failure=7 clause=literal_lock_misses_shared_state triggered={} cells_with_x_or_t_duplicates={}",
            literal_x_or_t > 0,
            literal_x_or_t
        ),
    );
    let mut stage_flipping_columns: Vec<String> = Vec::new();
    for arm in ALL_ARMS {
        for resource in ALL_RESOURCES {
            let positive_in_stage: Vec<bool> = ALL_STAGES
                .iter()
                .map(|stage| {
                    cells_of(default_cells, arm, *stage)
                        .iter()
                        .any(|result| result.aggregate.duplicate_total[resource.index()] > 0)
                })
                .collect();
            if positive_in_stage.iter().any(|positive| *positive) && positive_in_stage.iter().any(|positive| !*positive) {
                stage_flipping_columns.push(format!("{}:{}", arm.label(ArmVariant::Normal), resource.label()));
            }
        }
    }
    emit(
        emitter,
        &format!(
            "name=verdict failure=8 clause=judgement_flips_with_q1_candidate triggered={} flipping_columns={}",
            !stage_flipping_columns.is_empty(),
            if stage_flipping_columns.is_empty() { "none".to_string() } else { stage_flipping_columns.join(";") }
        ),
    );
    emit(
        emitter,
        &format!(
            "name=verdict failure=9 clause=read_set_below_whole_in_stage_two_same_shape triggered={}",
            read_set_below_whole_in_stage_two_same_shape > 0
        ),
    );
}

/// 第八节「几何敏感性」：D = 2（默认）与 D = 1（第一个反向取样点）上每格的判定逐格对一遍。
fn emit_geometry_sensitivity(emitter: &mut Emitter, default_cells: &[CellResult], sample_cells: &[CellResult]) {
    let mut unstable_groups: Vec<(String, u128, u128)> = Vec::new();
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            let default_group = cells_of(default_cells, arm, stage);
            for question in DUPLICATE_QUESTIONS.into_iter().chain([Question::SynchronousRounds]) {
                if question == Question::SynchronousRounds && !reports_rounds(&default_group[0].spec) {
                    continue;
                }
                let mut differing = 0;
                let mut strongest_default = 0u128;
                let mut strongest_sample = 0u128;
                for default_cell in &default_group {
                    let sample_cell = find_cell(sample_cells, &default_cell.spec)
                        .expect("每个默认格都有 D = 1 的同参数格");
                    let sample_aggregate = &sample_cell.aggregate;
                    let sample_rounds = &sample_cell.rounds;
                    let (default_judgement, sample_judgement, default_value, sample_value) =
                        if question == Question::SynchronousRounds {
                            let serial = default_cell.spec.transaction_count;
                            let default_equal = default_cell.rounds.rounds.is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") == serial);
                            let sample_equal = sample_rounds.rounds.is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") == serial);
                            (
                                default_equal,
                                sample_equal,
                                u128::from(default_cell.rounds.rounds.unwrap_or(0)),
                                u128::from(sample_rounds.rounds.unwrap_or(0)),
                            )
                        } else {
                            let default_total = duplicates_of_question(default_cell, question);
                            let sample_total: u128 = question
                                .resources()
                                .iter()
                                .map(|resource| sample_aggregate.duplicate_total[resource.index()])
                                .sum();
                            (default_total == 0, sample_total == 0, default_total, sample_total)
                        };
                    if default_judgement != sample_judgement {
                        differing += 1;
                        strongest_default = strongest_default.max(default_value);
                        strongest_sample = strongest_sample.max(sample_value);
                    }
                }
                if differing > 0 {
                    unstable_groups.push((
                        format!("{}:{}:{}", arm.label(ArmVariant::Normal), stage.label(), question.label()),
                        strongest_default,
                        strongest_sample,
                    ));
                }
                emit(
                    emitter,
                    &format!(
                        "name=geometry_sensitivity arm={} stage={} question={} cells={} cells_with_different_judgement={} outcome={}",
                        arm.label(ArmVariant::Normal),
                        stage.label(),
                        question.label(),
                        default_group.len(),
                        differing,
                        if differing == 0 { "two_points_same" } else { "unstable" }
                    ),
                );
            }
        }
    }
    if unstable_groups.is_empty() {
        emit(emitter, "name=discrimination_selfcheck status=not_needed_every_cell_two_points_same");
    }
    for (group, default_value, sample_value) in unstable_groups {
        // 把门槛挪到两点的值之间：一边绿一边红。
        let threshold_twice = default_value + sample_value;
        let default_is_above = default_value * 2 > threshold_twice;
        let sample_is_above = sample_value * 2 > threshold_twice;
        emit(
            emitter,
            &format!(
                "name=discrimination_selfcheck group={group} default_value={default_value} sample_value={sample_value} \
                 threshold_between_flips_judgement={}",
                default_is_above != sample_is_above
            ),
        );
    }
}

fn emit_segment_one(emitter: &mut Emitter) {
    emit(
        emitter,
        &format!(
            "name=config experiment=E166 segment=1 default_devices={DEFAULT_DEVICE_COUNT} sample_devices={SAMPLE_DEVICE_COUNT} \
             user_data_pairs={USER_DATA_PAIR_COUNT} commit_pool_slots={COMMIT_GENERATED_POOL_SLOTS} \
             root_slots_per_region={DEFAULT_ROOT_SLOTS_PER_REGION} journal_ring_slots={DEFAULT_JOURNAL_RING_SLOTS} \
             device_disagreement_retry_cap={DEVICE_DISAGREEMENT_RETRY_CAP} regions=transactions inode_reservation=exact \
             shapes=same,disjoint initial=file,nofile transactions=2,3 stages=Q1甲,Q1乙,Q1丙"
        ),
    );
    let anchor_lines = all_anchor_lines();
    let mismatched = anchor_lines.iter().filter(|line| !line.matches()).count();
    for line in &anchor_lines {
        emit(
            emitter,
            &format!(
                "name=anchor id=\"{}\" observed=\"{}\" expected=\"{}\" ok={}",
                line.identifier,
                line.observed,
                line.expected,
                line.matches()
            ),
        );
    }
    emit(
        emitter,
        &format!(
            "name=verdict anchors_checked={} anchors_mismatched={} anchors_all_ok={}",
            anchor_lines.len(),
            mismatched,
            mismatched == 0
        ),
    );
    if mismatched > 0 {
        println!("{}", emitter.finish());
        std::process::exit(1);
    }
    for device_count in [DEFAULT_DEVICE_COUNT, SAMPLE_DEVICE_COUNT] {
        for spec in control_cell_specs(device_count) {
            let result = run_cell(spec);
            emit(emitter, &cell_line("control_cell", &result.spec, &result.aggregate));
        }
    }
    let mut default_cells: Vec<CellResult> = Vec::new();
    let mut sample_cells: Vec<CellResult> = Vec::new();
    for device_count in [DEFAULT_DEVICE_COUNT, SAMPLE_DEVICE_COUNT] {
        for spec in arm_cell_specs(device_count) {
            let result = run_cell(spec);
            emit(emitter, &cell_line("cell", &result.spec, &result.aggregate));
            emit(emitter, &trajectory_line(&result.spec, &result.aggregate));
            if reports_rounds(&result.spec) {
                emit(emitter, &rounds_line(&result.spec, &result.rounds));
            }
            if device_count == DEFAULT_DEVICE_COUNT {
                default_cells.push(result);
            } else {
                sample_cells.push(result);
            }
        }
    }
    emit_question_verdicts(emitter, &default_cells);
    emit_failure_clauses(emitter, &default_cells);
    emit_geometry_sensitivity(emitter, &default_cells, &sample_cells);
    println!("{}", emitter.finish());
}

fn repository_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("仓根在 scripts/experiments 往上两层")
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("scripts/experiments/e166_parallel_commit_shared_state_partition.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
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
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    if let Some(unknown) = arguments.first() {
        eprintln!("  ✗ 认不出的参数 {unknown}：这一版只有第一段，不带参数跑");
        eprintln!("  → 怎么办：第二段（K = n − 1、多 inode、S = 1、环槽 2）登记标「够判后未跑」，没有实现");
        std::process::exit(2);
    }
    let mut emitter = Emitter::new();
    emit_segment_one(&mut emitter);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failing_anchors(lines: &[AnchorLine]) -> Vec<String> {
        lines
            .iter()
            .filter(|line| !line.matches())
            .map(|line| format!("{}: observed [{}] expected [{}]", line.identifier, line.observed, line.expected))
            .collect()
    }

    fn cell(
        arm: Arm,
        variant: ArmVariant,
        stage: ParallelStage,
        shape_kind: ShapeKind,
        initial_has_file: bool,
        transaction_count: usize,
        device_count: usize,
    ) -> Aggregate {
        enumerate_spec(spec_of(arm, variant, stage, shape_kind, initial_has_file, transaction_count, device_count))
            .aggregate
    }

    // ── 登记第七节甲 / 乙 / 丙：钉绝对值的断言（一份产物里的 `name=anchor` 行就是这几组） ─────────────

    #[test]
    fn serial_baseline_and_stage_definitions_match_the_clause_values() {
        let failing = failing_anchors(&anchors_from_the_clauses());
        assert!(failing.is_empty(), "甲组不符：{failing:#?}");
    }

    #[test]
    fn positive_controls_and_partition_cells_match_the_independently_calculated_values() {
        let failing = failing_anchors(&anchors_from_independent_calculations());
        assert!(failing.is_empty(), "乙组不符：{failing:#?}");
    }

    #[test]
    fn schedule_counts_serial_baseline_and_memoization_match_the_structural_values() {
        let failing = failing_anchors(&anchors_from_structural_counts());
        assert!(failing.is_empty(), "丙组不符：{failing:#?}");
    }

    #[test]
    fn local_constants_agree_with_the_crates_values_they_copy() {
        assert_eq!(JOURNAL_RING_DEFAULT_BYTES, u64::from(DEFAULT_JOURNAL_RING_SLOTS) * JOURNAL_RECORD_BYTES);
        assert_eq!(DEFAULT_JOURNAL_RING_SLOTS, 196_608);
        assert_eq!(ROOT_RING_REGIONS, 3);
        assert_eq!(TREE_IDENTIFIER_WATERMARK_AT_MKFS, 65536);
        assert_eq!(TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, 65548);
        assert_eq!(TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION, 12);
        assert_eq!(TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION + TREE_IDENTIFIER_WATERMARK_AT_MKFS, 65548);
        assert_eq!(INITIAL_INODE_WATERMARK, 1 + 1);
        assert_eq!(USER_DATA_SPAN_SLOTS, 2);
    }

    // ── 单个函数 ─────────────────────────────────────────────────────────────

    #[test]
    fn duplicate_count_is_holders_minus_one_for_every_item() {
        assert_eq!(duplicates_among(&mut [5, 5, 5, 7, 7, 9]), 3);
        assert_eq!(duplicates_among(&mut [1, 2, 3]), 0);
        assert_eq!(duplicates_among(&mut [4, 4]), 1);
    }

    #[test]
    fn lowest_free_pair_takes_the_smallest_untaken_pair_inside_the_range() {
        assert_eq!(lowest_free_pair(0b0000, 0, 16), Some(0));
        assert_eq!(lowest_free_pair(0b0111, 0, 16), Some(3));
        assert_eq!(lowest_free_pair(0b0101, 0, 16), Some(1));
        assert_eq!(lowest_free_pair(0b0111, 0, 3), None);
        assert_eq!(lowest_free_pair(0b0001_1111, 5, 10), Some(5));
        assert_eq!(pair_range_mask(0, 16), u16::MAX);
        assert_eq!(pair_range_mask(5, 10), 0b0000_0011_1110_0000);
    }

    #[test]
    fn schedule_count_formula_is_the_multinomial() {
        assert_eq!(schedule_count_without_retries(2, 1), 2);
        assert_eq!(schedule_count_without_retries(3, 1), 6);
        assert_eq!(schedule_count_without_retries(2, 2), 6);
        assert_eq!(schedule_count_without_retries(3, 2), 90);
        assert_eq!(schedule_count_without_retries(2, 3), 20);
        assert_eq!(schedule_count_without_retries(3, 3), 1680);
        assert_eq!(schedule_count_without_retries(3, 6), 17_153_136);
    }

    #[test]
    fn root_ring_and_journal_slot_formulas_match_the_clause_examples() {
        assert_eq!(slot_text(root_ring_slot_key(6, 4)), "(0,2)");
        assert_eq!(slot_text(root_ring_slot_key(7, 4)), "(1,2)");
        assert_eq!(slot_text(root_ring_slot_key(9, 4)), "(0,3)");
        assert_eq!(slot_text(root_ring_slot_key(12, 4)), "(0,0)");
        assert_eq!(journal_ring_slot(6, DEFAULT_JOURNAL_RING_SLOTS), 5);
        assert_eq!(journal_ring_slot(1, 2), 0);
        assert_eq!(journal_ring_slot(3, 2), 0);
    }

    // ── 各臂在默认几何上的判定（不是阳性对照，是被测臂自己） ─────────────────────────

    #[test]
    fn whole_lock_hands_out_no_duplicate_in_any_stage_shape_or_initial_state() {
        for stage in ALL_STAGES {
            for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                for initial_has_file in [true, false] {
                    for transaction_count in [2usize, 3] {
                        let aggregate = cell(
                            Arm::WholeLock,
                            ArmVariant::Normal,
                            stage,
                            shape_kind,
                            initial_has_file,
                            transaction_count,
                            DEFAULT_DEVICE_COUNT,
                        );
                        assert_eq!(
                            aggregate.duplicate_total,
                            [0; RESOURCE_COUNT],
                            "{} {shape_kind:?} {initial_has_file} n={transaction_count}",
                            stage.label()
                        );
                        let factorial_of_count: u128 = (1..=transaction_count as u128).product();
                        assert_eq!(aggregate.schedules, factorial_of_count);
                        assert_eq!(aggregate.retries_total, 0);
                    }
                }
            }
        }
    }

    #[test]
    fn region_partition_and_device_partition_hand_out_no_duplicate_in_default_geometry() {
        for arm in [Arm::RegionPartition, Arm::DevicePartition] {
            for stage in ALL_STAGES {
                for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                    for initial_has_file in [true, false] {
                        for transaction_count in [2usize, 3] {
                            for device_count in [SAMPLE_DEVICE_COUNT, DEFAULT_DEVICE_COUNT] {
                                let aggregate = cell(
                                    arm,
                                    ArmVariant::Normal,
                                    stage,
                                    shape_kind,
                                    initial_has_file,
                                    transaction_count,
                                    device_count,
                                );
                                assert_eq!(
                                    aggregate.duplicate_total,
                                    [0; RESOURCE_COUNT],
                                    "{arm:?} {} {shape_kind:?} {initial_has_file} n={transaction_count} D={device_count}",
                                    stage.label()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn region_partition_with_one_region_per_transaction_never_falls_back_or_deadlocks() {
        for stage in ALL_STAGES {
            let aggregate = cell(
                Arm::RegionPartition,
                ArmVariant::Normal,
                stage,
                ShapeKind::SameShape,
                false,
                3,
                DEFAULT_DEVICE_COUNT,
            );
            assert_eq!(aggregate.fallbacks_total, 0);
            assert_eq!(aggregate.deadlocks, 0);
            assert_eq!(aggregate.retries_total, 0);
            assert_eq!(aggregate.busy_regions_peak, 3, "三个事务各占一个区，峰值 3");
            assert_eq!(aggregate.busy_regions_final_maximum, 0, "全部提交之后区都释放了");
        }
    }

    #[test]
    fn literal_lock_leaves_transaction_numbers_and_tree_identifiers_outside_the_lock() {
        // 锁内四样（U、G、I、journal 计数器、txg）没有重复；锁外两样（X、T）有（失败 7 的前件）。
        let aggregate = cell(
            Arm::LiteralLock,
            ArmVariant::Normal,
            ParallelStage::ManyPublishesInFlight,
            ShapeKind::SameShape,
            false,
            2,
            DEFAULT_DEVICE_COUNT,
        );
        for resource in [
            Resource::UserDataSlot,
            Resource::CommitGeneratedSlot,
            Resource::InodeNumber,
            Resource::JsnCounter,
            Resource::JournalRingSlot,
            Resource::CheckpointTxg,
            Resource::RootRingSlot,
        ] {
            assert_eq!(aggregate.duplicate_total[resource.index()], 0, "{}", resource.label());
        }
        assert_eq!(aggregate.duplicate_total[Resource::TransactionNumber.index()], 18);
        // 取第 2 次重跑登记第七节 7.2 补 1「无文件 n = 2 的 T 合计 216」（独立 Python `run_literal(2, False, 65536, 12)` 算出）。
        assert_eq!(aggregate.duplicate_total[Resource::TreeIdentifier.index()], 216);
    }

    #[test]
    fn read_set_optimistic_checks_the_user_pair_and_takes_the_transaction_number_at_the_commit_point() {
        // Q1乙、不相交：事务 0 只建 inode、其余只写用户数据单元。
        let three = cell(
            Arm::OptimisticReadSet,
            ArmVariant::Normal,
            ParallelStage::OnePublishCarriesManyTransactions,
            ShapeKind::Disjoint,
            true,
            3,
            DEFAULT_DEVICE_COUNT,
        );
        assert_eq!(three.duplicate_total, [0; RESOURCE_COUNT], "校验里有「用户数据对仍空」，事务 1 与 2 争同一个对时后提交那个重算");
        assert!(three.retries_total > 0);
        let two = cell(
            Arm::OptimisticReadSet,
            ArmVariant::Normal,
            ParallelStage::OnePublishCarriesManyTransactions,
            ShapeKind::Disjoint,
            true,
            2,
            DEFAULT_DEVICE_COUNT,
        );
        assert_eq!(two.duplicate_total[Resource::TransactionNumber.index()], 0, "事务号在提交点按当时的共享值发");
        assert_eq!(two.retries_total, 0, "事务 0 的读集只有 inode 起点、事务 1 只有一个用户数据对，互不相交");
    }

    #[test]
    fn read_set_retries_equal_whole_version_retries_when_base_version_is_in_the_read_set() {
        // 失败 5 的前件：Q1甲 与 Q1丙 下 BASE 版本号在读集里，任何插进来的提交都让 O读 重算，与 O整 逐格相等。
        for stage in [ParallelStage::PrepareParallelCommitSerial, ParallelStage::ManyPublishesInFlight] {
            for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                for transaction_count in [2usize, 3] {
                    let whole = cell(
                        Arm::OptimisticWholeVersion,
                        ArmVariant::Normal,
                        stage,
                        shape_kind,
                        true,
                        transaction_count,
                        DEFAULT_DEVICE_COUNT,
                    );
                    let read_set = cell(
                        Arm::OptimisticReadSet,
                        ArmVariant::Normal,
                        stage,
                        shape_kind,
                        true,
                        transaction_count,
                        DEFAULT_DEVICE_COUNT,
                    );
                    assert_eq!(whole.retries_total, read_set.retries_total);
                    assert_eq!(whole.schedules, read_set.schedules);
                }
            }
        }
    }

    // ── 分区臂的另一端：区不够、预留不够（第二段的取样点，这里只验模型的这几条路径） ─────────────

    #[test]
    fn one_region_for_two_transactions_makes_the_second_claim_wait_for_the_first_commit() {
        let mut spec = spec_of(
            Arm::RegionPartition,
            ArmVariant::Normal,
            ParallelStage::PrepareParallelCommitSerial,
            ShapeKind::SameShape,
            true,
            2,
            DEFAULT_DEVICE_COUNT,
        );
        spec.region_count = RegionCountChoice::Fixed(1);
        let model = Model::new(spec);
        let enumeration = enumerate_schedules(&model, false);
        assert_eq!(enumeration.aggregate.schedules, 2, "先认领到区的那个事务先走完，只剩两种次序");
        assert_eq!(enumeration.aggregate.deadlocks, 0);
        assert_eq!(enumeration.aggregate.duplicate_total, [0; RESOURCE_COUNT]);
        let rounds = simulate_rounds(&model);
        assert_eq!(rounds.rounds, Some(2));
        assert_eq!(rounds.commits_per_round, vec![1, 1]);
    }

    #[test]
    fn inode_reservation_smaller_than_the_need_takes_a_fallback_interval_without_duplicates() {
        let mut spec = spec_of(
            Arm::RegionPartition,
            ArmVariant::Normal,
            ParallelStage::OnePublishCarriesManyTransactions,
            ShapeKind::MultiInode,
            true,
            2,
            DEFAULT_DEVICE_COUNT,
        );
        spec.inode_reservation = InodeReservation::Fixed(1);
        let model = Model::new(spec);
        let aggregate = enumerate_schedules(&model, false).aggregate;
        assert_eq!(aggregate.fallbacks_total, aggregate.schedules * 2, "每个事务都要补一段区间");
        assert_eq!(aggregate.duplicate_total, [0; RESOURCE_COUNT]);
        assert_eq!(aggregate.deadlocks, 0);
        let state = run_serially(&model);
        assert_eq!(state.shared.inode_watermark, INITIAL_INODE_WATERMARK + 4, "两个事务各要 2 个号，共发 4 个");
    }

    #[test]
    fn inode_reservation_larger_than_the_need_leaves_numbers_unused() {
        let mut spec = spec_of(
            Arm::RegionPartition,
            ArmVariant::Normal,
            ParallelStage::OnePublishCarriesManyTransactions,
            ShapeKind::SameShape,
            true,
            2,
            DEFAULT_DEVICE_COUNT,
        );
        spec.inode_reservation = InodeReservation::Fixed(3);
        let aggregate = enumerate_schedules(&Model::new(spec), false).aggregate;
        assert_eq!(aggregate.unused_reserved_final_maximum, 4, "每个事务多预留 2 个，两个事务共 4 个号空洞");
        assert_eq!(aggregate.duplicate_total, [0; RESOURCE_COUNT]);
    }

    // ── 轨迹量 ───────────────────────────────────────────────────────────────

    #[test]
    fn duplicate_peak_equals_the_largest_final_duplicate_because_holders_only_grow() {
        for (arm, variant, stage, transaction_count) in [
            (Arm::WholeLock, ArmVariant::PositiveControl, ParallelStage::ManyPublishesInFlight, 3),
            (Arm::RegionPartition, ArmVariant::PositiveControl, ParallelStage::ManyPublishesInFlight, 3),
            (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl, ParallelStage::PrepareParallelCommitSerial, 3),
            (Arm::LiteralLock, ArmVariant::Normal, ParallelStage::PrepareParallelCommitSerial, 3),
            (Arm::DevicePartition, ArmVariant::PositiveControl, ParallelStage::PrepareParallelCommitSerial, 2),
        ] {
            let aggregate = cell(arm, variant, stage, ShapeKind::SameShape, false, transaction_count, DEFAULT_DEVICE_COUNT);
            assert_eq!(aggregate.duplicate_peak, aggregate.duplicate_max_single_schedule, "{}", arm.label(variant));
            for resource in 0..RESOURCE_COUNT {
                assert!(
                    aggregate.duplicate_positive_steps[resource] >= aggregate.duplicate_schedules[resource],
                    "有重复的调度至少有一步为正：{}",
                    arm.label(variant)
                );
            }
        }
    }

    #[test]
    fn inode_margin_goes_negative_only_when_the_shared_watermark_falls_behind_a_held_number() {
        // 整份写回：事务 1、0 先后按水位 2 拿号并写回水位 3，事务 2 拿 3 写回水位 4，最后事务 1 用旧拷贝把水位写回 3：
        // 已提交的最大号 3，余量 = 3 − 3 − 1 = −1，共享水位退到了已发出的号之下（I-9.6 的反面）。
        let lost_update = cell(
            Arm::WholeLock,
            ArmVariant::PositiveControl,
            ParallelStage::ManyPublishesInFlight,
            ShapeKind::SameShape,
            true,
            3,
            DEFAULT_DEVICE_COUNT,
        );
        assert_eq!(lost_update.inode_margin_minimum, -1);
        assert!(lost_update.inode_margin_negative_steps > 0);
        let correct = cell(
            Arm::WholeLock,
            ArmVariant::Normal,
            ParallelStage::ManyPublishesInFlight,
            ShapeKind::SameShape,
            true,
            3,
            DEFAULT_DEVICE_COUNT,
        );
        assert_eq!(correct.inode_margin_minimum, 0);
        assert_eq!(correct.inode_margin_negative_steps, 0);
    }

    // ── 同步轮次 ─────────────────────────────────────────────────────────────

    fn rounds_of(arm: Arm, stage: ParallelStage, shape_kind: ShapeKind, transaction_count: usize) -> RoundsOutcome {
        simulate_rounds(&Model::new(spec_of(
            arm,
            ArmVariant::Normal,
            stage,
            shape_kind,
            true,
            transaction_count,
            DEFAULT_DEVICE_COUNT,
        )))
    }

    #[test]
    fn synchronous_rounds_match_the_hand_calculation_for_each_arm() {
        let one_publish = ParallelStage::OnePublishCarriesManyTransactions;
        for transaction_count in [2usize, 3] {
            let serial_rounds = u32::try_from(transaction_count).expect("n 很小");
            for stage in ALL_STAGES {
                assert_eq!(rounds_of(Arm::OptimisticWholeVersion, stage, ShapeKind::SameShape, transaction_count).rounds, Some(serial_rounds));
                let all_in_one_round = rounds_of(Arm::RegionPartition, stage, ShapeKind::SameShape, transaction_count);
                assert_eq!(all_in_one_round.rounds, Some(1));
                assert_eq!(all_in_one_round.commits_per_round, vec![serial_rounds]);
                assert_eq!(rounds_of(Arm::DevicePartition, stage, ShapeKind::SameShape, transaction_count).rounds, Some(1));
            }
            assert_eq!(rounds_of(Arm::OptimisticReadSet, one_publish, ShapeKind::SameShape, transaction_count).rounds, Some(serial_rounds));
        }
        let disjoint_three = rounds_of(Arm::OptimisticReadSet, one_publish, ShapeKind::Disjoint, 3);
        assert_eq!(disjoint_three.commits_per_round, vec![2, 1], "事务 1 与 2 争同一个用户数据对，后提交那个下一轮重来");
        assert_eq!(rounds_of(Arm::OptimisticReadSet, one_publish, ShapeKind::Disjoint, 2).commits_per_round, vec![2]);
        assert_eq!(
            rounds_of(Arm::OptimisticReadSet, ParallelStage::ManyPublishesInFlight, ShapeKind::Disjoint, 3).rounds,
            Some(3),
            "Q1丙 下 BASE 版本号在读集里，不相交也一个一个提交"
        );
    }

    fn descending_order(transaction_count: usize) -> Vec<usize> {
        (0..transaction_count).rev().collect()
    }

    #[test]
    fn commit_order_inside_a_round_changes_the_commits_per_round_for_one_inode_only_one_unit_only_and_one_of_both() {
        // 默认的「不相交」形状（事务 0 只建 inode，其余只写用户数据单元）上，升序与倒序的每轮提交数一样（[2, 1]），
        // 变异 V19 在那里不敏感；这一组形状上升序是 [2, 1]、倒序是 [1, 2]：升序先让事务 0、1 同轮提交、事务 2 与两者都冲突；
        // 倒序先让事务 2 提交，事务 0 与 1 各与它冲突，下一轮才一起提交。总轮数在这两种次序下总是相同（n ≤ 7 的穷举见报告）。
        let mut model = Model::new(spec_of(
            Arm::OptimisticReadSet,
            ArmVariant::Normal,
            ParallelStage::OnePublishCarriesManyTransactions,
            ShapeKind::SameShape,
            true,
            3,
            DEFAULT_DEVICE_COUNT,
        ));
        model.shapes = vec![
            TransactionShape { user_units: 0, new_inodes: 1 },
            TransactionShape { user_units: 1, new_inodes: 0 },
            TransactionShape { user_units: 1, new_inodes: 1 },
        ];
        let ascending = simulate_rounds(&model);
        assert_eq!(ascending.commits_per_round, vec![2, 1]);
        let descending = simulate_rounds_in_order(&model, descending_order);
        assert_eq!(descending.commits_per_round, vec![1, 2]);
        assert_eq!(ascending.rounds, descending.rounds);
    }
}
