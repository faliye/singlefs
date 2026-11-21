//! 一次模拟跑的世界、活化变体、块与池的状态、根记录与待抬的底线。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum World {
    Steady,
    TornNewest,
    DiskLoss,
    SlotWriteFail,
    AdminRollback,
    RollbackFirstPublish,
}

pub(super) const WORLDS: [World; 6] = [World::Steady, World::TornNewest, World::DiskLoss, World::SlotWriteFail, World::AdminRollback, World::RollbackFirstPublish];

impl World {
    pub(super) fn name(self) -> &'static str {
        match self {
            World::Steady => "steady",
            World::TornNewest => "torn_newest",
            World::DiskLoss => "disk_loss",
            World::SlotWriteFail => "slot_write_fail",
            World::AdminRollback => "admin_rollback",
            World::RollbackFirstPublish => "rollback_first_publish",
        }
    }
    /// E135 跑前补登第 1 条：这四个世界给 F 臂在故障之前安排一次强制盘紧。
    pub(super) fn forces_pressure_before_event(self) -> bool {
        match self {
            World::TornNewest | World::DiskLoss | World::SlotWriteFail | World::AdminRollback => true,
            World::Steady | World::RollbackFirstPublish => false,
        }
    }
}

/// 活化区间一族世界的三种形态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ActivationVariant {
    /// E138 原样：抬一次 F，写失败 × 崩溃窗口 × 丢盘。
    Plain,
    /// 第 j 个窗口崩溃不丢盘、恢复，再跑 2 个窗口，然后再崩一次并丢盘。
    CrashWithoutLossFirst,
    /// 抬 F 之前先让区域 1 的根槽连续写失败 r 次（跳号），再照 Plain 枚举。
    SkipThenRaise { repeats: u64 },
}

impl ActivationVariant {
    pub(super) fn name(self) -> String {
        match self {
            ActivationVariant::Plain => "activation_faults".to_string(),
            ActivationVariant::CrashWithoutLossFirst => "activation_crash_no_loss".to_string(),
            ActivationVariant::SkipThenRaise { repeats } => format!("skip_then_raise_r{repeats}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum BlockState {
    Unused,
    Live { allocated_at: u64 },
    Freed { allocated_at: u64, freed_at: u64 },
}

#[derive(Clone)]
pub(super) struct PoolState {
    pub(super) blocks: Vec<BlockState>,
    pub(super) objects: Vec<[usize; USER_BLOCKS_PER_OBJECT]>,
    pub(super) metadata: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct RootRecord {
    pub(super) txg: u64,
    pub(super) instance: u64,
    pub(super) floor: u64,
    pub(super) is_self_verified: bool,
}

#[derive(Clone, Copy)]
pub(super) struct PendingFloor {
    pub(super) value: u64,
    pub(super) carrier_txg: Option<u64>,
    pub(super) durable_publications_since_carrier: u64,
}

/// m 取不到（有一块幸存盘一条有效根都没有）时回收门槛 F 那一项当场取的值：`crates/singlefs-core/src/mount.rs` 的
/// `reclaim_the_released_slots_of_the_rebuilt_allocator` 那一支只放到最旧那一项，等于 F 那一项取 0（第 3 次重跑登记第一节读法第 1 条）。
pub(super) const FLOOR_TERM_WHEN_A_SURVIVING_DISK_HAS_NO_VALID_ROOT: u64 = 0;
