//! 并行阶段、臂与变体、事务形状、格的规格、步的种类与步序列、模型本身。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ParallelStage {
    PrepareParallelCommitSerial,
    OnePublishCarriesManyTransactions,
    ManyPublishesInFlight,
}

pub(super) const ALL_STAGES: [ParallelStage; 3] = [
    ParallelStage::PrepareParallelCommitSerial,
    ParallelStage::OnePublishCarriesManyTransactions,
    ParallelStage::ManyPublishesInFlight,
];

impl ParallelStage {
    pub(super) fn label(self) -> &'static str {
        match self {
            ParallelStage::PrepareParallelCommitSerial => "Q1甲",
            ParallelStage::OnePublishCarriesManyTransactions => "Q1乙",
            ParallelStage::ManyPublishesInFlight => "Q1丙",
        }
    }

    /// 每个事务自己碰哪几组共享状态（登记 5.1「Q1 候选决定每个事务自己碰哪些资源」）。
    pub(super) fn transaction_groups(self) -> u8 {
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
    pub(super) fn pipeline_groups(self) -> u8 {
        match self {
            ParallelStage::PrepareParallelCommitSerial => GROUP_JOURNAL | GROUP_ROOT_RING,
            ParallelStage::OnePublishCarriesManyTransactions => 0,
            ParallelStage::ManyPublishesInFlight => 0,
        }
    }

    /// BASE 版本号进不进 O读 的读集（登记 5.2：Q1甲 与 Q1丙 进，Q1乙 不进）。
    pub(super) fn base_version_in_read_set(self) -> bool {
        match self {
            ParallelStage::PrepareParallelCommitSerial => true,
            ParallelStage::OnePublishCarriesManyTransactions => false,
            ParallelStage::ManyPublishesInFlight => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Arm {
    WholeLock,
    LiteralLock,
    RegionPartition,
    DevicePartition,
    OptimisticWholeVersion,
    OptimisticReadSet,
}

pub(super) const ALL_ARMS: [Arm; 6] = [
    Arm::WholeLock,
    Arm::LiteralLock,
    Arm::RegionPartition,
    Arm::DevicePartition,
    Arm::OptimisticWholeVersion,
    Arm::OptimisticReadSet,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ArmVariant {
    Normal,
    /// 登记 5.3：关掉这条臂自己那条护栏的阳性对照。
    PositiveControl,
}

impl Arm {
    pub(super) fn label(self, variant: ArmVariant) -> &'static str {
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
pub(super) enum ShapeKind {
    /// 每个事务 1 个用户数据单元、建 1 个 inode。
    SameShape,
    /// 事务 0 只建 1 个 inode；其余事务只写 1 个用户数据单元。
    Disjoint,
    /// 每个事务 1 个用户数据单元、建 2 个 inode（只用于 P区 的预留不够那一格）。
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    MultiInode,
}

impl ShapeKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            ShapeKind::SameShape => "same",
            ShapeKind::Disjoint => "disjoint",
            ShapeKind::MultiInode => "multi_inode",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct TransactionShape {
    pub(super) user_units: u8,
    pub(super) new_inodes: u8,
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
pub(super) enum InodeReservation {
    /// P区 / P设 在 CLAIM 里预留的 inode 号个数 = 本事务要建的数（默认 r_i 够用）。
    Exact,
    /// 每个要建 inode 的事务固定预留这么多个（第二段「多 inode、r_i = 1」用）。
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    Fixed(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RegionCountChoice {
    EqualToTransactionCount,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    Fixed(usize),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CellSpecification {
    pub(super) arm: Arm,
    pub(super) variant: ArmVariant,
    pub(super) stage: ParallelStage,
    pub(super) shape_kind: ShapeKind,
    pub(super) initial_has_file: bool,
    pub(super) transaction_count: usize,
    pub(super) device_count: usize,
    pub(super) region_count: RegionCountChoice,
    pub(super) inode_reservation: InodeReservation,
    pub(super) root_slots_per_region: u32,
    pub(super) journal_ring_slots: u32,
}

impl CellSpecification {
    pub(super) fn default_geometry(
        arm: Arm,
        variant: ArmVariant,
        stage: ParallelStage,
        shape_kind: ShapeKind,
        initial_has_file: bool,
        transaction_count: usize,
        device_count: usize,
    ) -> CellSpecification {
        CellSpecification {
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
pub(super) enum StepKind {
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
pub(super) struct Model {
    pub(super) arm: Arm,
    pub(super) variant: ArmVariant,
    pub(super) stage: ParallelStage,
    pub(super) shapes: Vec<TransactionShape>,
    pub(super) initial_has_file: bool,
    pub(super) transaction_count: usize,
    pub(super) device_count: usize,
    pub(super) region_count: usize,
    pub(super) inode_reservation: InodeReservation,
    pub(super) root_slots_per_region: u32,
    pub(super) journal_ring_slots: u32,
    pub(super) steps: Vec<StepKind>,
    pub(super) transaction_groups: u8,
    pub(super) pipeline_groups: u8,
}

impl Model {
    pub(super) fn new(specification: CellSpecification) -> Model {
        assert!(specification.transaction_count >= 1 && specification.transaction_count <= MAXIMUM_TRANSACTIONS);
        assert!(specification.device_count >= 1 && specification.device_count <= MAXIMUM_DEVICES);
        let region_count = match specification.region_count {
            RegionCountChoice::EqualToTransactionCount => specification.transaction_count,
            RegionCountChoice::Fixed(count) => count,
        };
        assert!((1..=MAXIMUM_REGIONS).contains(&region_count));
        Model {
            arm: specification.arm,
            variant: specification.variant,
            stage: specification.stage,
            shapes: shapes_of(specification.shape_kind, specification.transaction_count),
            initial_has_file: specification.initial_has_file,
            transaction_count: specification.transaction_count,
            device_count: specification.device_count,
            region_count,
            inode_reservation: specification.inode_reservation,
            root_slots_per_region: specification.root_slots_per_region,
            journal_ring_slots: specification.journal_ring_slots,
            steps: steps_of(specification.arm, specification.variant, specification.device_count),
            transaction_groups: specification.stage.transaction_groups(),
            pipeline_groups: specification.stage.pipeline_groups(),
        }
    }

    pub(super) fn user_pairs_counted_per_device(&self) -> bool {
        self.arm == Arm::DevicePartition
    }

    pub(super) fn optimistic_uses_read_set(&self) -> bool {
        self.arm == Arm::OptimisticReadSet && self.variant == ArmVariant::Normal
    }

    pub(super) fn region_pair_range(&self, region: usize) -> (usize, usize) {
        let region_pairs = USER_DATA_PAIR_COUNT / self.region_count;
        (region * region_pairs, (region + 1) * region_pairs)
    }

    pub(super) fn region_slot_range(&self, region: usize) -> (u8, u8) {
        let region_slots = usize::from(COMMIT_GENERATED_POOL_SLOTS) / self.region_count;
        (small(region * region_slots), small((region + 1) * region_slots))
    }
}

pub(super) fn small(value: usize) -> u8 {
    u8::try_from(value).expect("模型里的号与槽都小于 256")
}
