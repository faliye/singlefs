//! 发布的准备与记录、断言与自检的计数、用户改动与缺口事件、模拟器的状态。

use super::*;

/// 一次发布算好、还没落盘的样子。
#[derive(Clone, Debug)]
pub(super) struct PreparedPublish {
    pub(super) next: Version,
    pub(super) new_units: Vec<Unit>,
    pub(super) written: UnitsWritten,
    pub(super) merge: MergeAction,
    pub(super) retired_identities: Vec<ContainerIdentity>,
    pub(super) stubbed_identities: Vec<ContainerIdentity>,
    pub(super) moved_objects: Vec<u64>,
    pub(super) split_checks: Vec<bool>,
    pub(super) shrink: Option<ShrinkEvent>,
}

/// 逐发布留的账：N1 / N2 配对（Q-X4-1、Q-X4-2、A5）、缺口事件（F11）、X2 的多写（Q-X2-6、Q-X2-7）、第四段的配对（V-Bs1）与桩的账用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PublishRecord {
    pub(super) publish_sequence: u64,
    pub(super) txg: u64,
    pub(super) merge: MergeAction,
    pub(super) retired_identities: Vec<ContainerIdentity>,
    pub(super) written: UnitsWritten,
    pub(super) shrink: Option<ShrinkEvent>,
    pub(super) moved_object_shrunk: bool,
    pub(super) leaf_digest: u64,
    pub(super) is_a_rollback: bool,
    pub(super) extent_digest: u64,
    pub(super) stubs_written: u64,
    /// 这一版的叶片总数（活叶 + 桩）与桩数（Q-Bs-5；原写路径桩数恒 0）。
    pub(super) leaves_in_the_leaf_order: u64,
    pub(super) stubs_in_the_leaf_order: u64,
    /// 这一版里至少有一个记录数为 0 的类型 2 容器（Q-Bs-6）。
    pub(super) holds_an_empty_inode_leaf: bool,
    /// 到这一版做完（最后一批已发布）的 unlink 数（Q-Bs-5）。
    pub(super) unlinks_completed_so_far: u64,
}

/// 每条断言判了几次、违反几次（第七节、第十一节）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct AssertionTally {
    pub(super) judged: BTreeMap<&'static str, u64>,
    violated: BTreeMap<&'static str, u64>,
}

impl AssertionTally {
    pub(super) fn judge(&mut self, name: &'static str, holds: bool) {
        *self.judged.entry(name).or_insert(0) += 1;
        if !holds {
            *self.violated.entry(name).or_insert(0) += 1;
        }
    }

    pub(super) fn merge(&mut self, other: &AssertionTally) {
        for (name, count) in &other.judged {
            *self.judged.entry(name).or_insert(0) += count;
        }
        for (name, count) in &other.violated {
            *self.violated.entry(name).or_insert(0) += count;
        }
    }

    pub(super) fn violations_of(&self, name: &str) -> u64 {
        self.violated.get(name).copied().unwrap_or(0)
    }

    pub(super) fn judged_of(&self, name: &str) -> u64 {
        self.judged.get(name).copied().unwrap_or(0)
    }
}

/// 第十一节 V7：每个发布出去的版本上判 I-9.13、今天的 I-9.4、N1 的 I-9.4、I-9.12，记红的版本数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SelfCheckTally {
    pub(super) versions: u64,
    pub(super) leaves_hold_a_record_red: u64,
    pub(super) today_container_numbers_red: u64,
    pub(super) raised_separator_container_numbers_red: u64,
    pub(super) separators_red: u64,
}

impl SelfCheckTally {
    pub(super) fn note(&mut self, shape: &TreeShape) {
        let verdicts = judge_the_shape(shape);
        self.versions += 1;
        self.leaves_hold_a_record_red += u64::from(!every_leaf_holds_a_record(shape));
        self.today_container_numbers_red += u64::from(!verdicts.today_container_numbers_hold);
        self.raised_separator_container_numbers_red += u64::from(!verdicts.raised_separator_container_numbers_hold);
        self.separators_red += u64::from(!verdicts.separators_hold);
    }

    pub(super) fn add(&mut self, other: SelfCheckTally) {
        self.versions += other.versions;
        self.leaves_hold_a_record_red += other.leaves_hold_a_record_red;
        self.today_container_numbers_red += other.today_container_numbers_red;
        self.raised_separator_container_numbers_red += other.raised_separator_container_numbers_red;
        self.separators_red += other.separators_red;
    }
}

// ---------------------------------------------------------------------------------------------
// 模拟：写路径（一个写路径格一条），可选每次发布之后扫一次
// ---------------------------------------------------------------------------------------------

/// 一次用户改动（E177 装置 `UserChange` 的推广：写文件可以写任一对象）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UserChange {
    WriteFile { object: u64, data_units: u64 },
    NewInodes { count: u64 },
    Unlink { object: u64 },
}

/// F11 的缺口事件（登记 5.2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum GapEvent {
    Absorb,
    Borrow,
    Drop,
    MovedObjectShortened,
}

pub(super) const GAP_EVENTS: [GapEvent; 4] = [GapEvent::Absorb, GapEvent::Borrow, GapEvent::Drop, GapEvent::MovedObjectShortened];

impl GapEvent {
    pub(super) const fn label(self) -> &'static str {
        match self {
            GapEvent::Absorb => "absorb",
            GapEvent::Borrow => "borrow",
            GapEvent::Drop => "drop",
            GapEvent::MovedObjectShortened => "moved_object_shortened",
        }
    }

    pub(super) const fn happened_in(self, merge: MergeAction, moved_object_shrunk: bool) -> bool {
        match self {
            GapEvent::Absorb => {
                matches!(merge, MergeAction::AbsorbedTheRightNeighbour | MergeAction::AbsorbedIntoTheLeftNeighbour)
            }
            GapEvent::Borrow => matches!(merge, MergeAction::BorrowedFromTheRightNeighbour { .. }),
            GapEvent::Drop => matches!(merge, MergeAction::EmptiedAndDropped),
            GapEvent::MovedObjectShortened => moved_object_shrunk,
        }
    }
}

/// 第一次出现缺口事件的那次发布之前那一版（R_old）。
#[derive(Clone, Debug)]
pub(super) struct GapStart {
    pub(super) version_before: Version,
    pub(super) event_publish_sequence: u64,
}

#[derive(Clone, Debug)]
pub(super) struct Simulation {
    pub(super) cell: WritePathCell,
    /// 容器身份消失那次写退役记录（原写路径）还是落桩（B-stub 写路径）。
    pub(super) container_death: ContainerDeathWritePath,
    /// 每次扫描读哪几个读者（第二、三段是 `every_reader` 的 20 个，第四段两条写路径各一张表）。
    pub(super) readers: Vec<Reader>,
    pub(super) batch_data_units: u64,
    pub(super) disk: Disk,
    pub(super) version: Version,
    /// 跨崩溃单调的发布序号（崩溃那一次没发布成的写也占一个），只给旋钮 W 用。
    pub(super) publish_sequence: u64,
    /// 实例表：旧实例 → 已施加的最大事务号。
    pub(super) instance_rows: BTreeMap<u64, u64>,
    pub(super) publishes: u64,
    pub(super) refused_steps: u64,
    /// 记录被借走或被吸收过的对象（历史上的事实，回退不清）。
    pub(super) moved_objects: BTreeSet<u64>,
    /// 被挪过、之后变短过的对象（PC-Bt2-mid 的取样，登记修订 3）。
    pub(super) moved_then_shortened_objects: BTreeSet<u64>,
    /// 变短过的对象 → 变短之前最大的 size（PC-Z）。
    pub(super) largest_size_before_a_shrink: BTreeMap<u64, u64>,
    pub(super) moved_then_shortened: u64,
    pub(super) moved_then_unlinked: u64,
    pub(super) publish_records: Vec<PublishRecord>,
    pub(super) user_changes: Vec<UserChange>,
    pub(super) self_check: SelfCheckTally,
    pub(super) assertions: AssertionTally,
    pub(super) does_not_fit_at: Option<u64>,
    pub(super) scanning: bool,
    pub(super) scans: Vec<ScanRecord>,
    pub(super) gap_watch: Option<GapEvent>,
    pub(super) gap_start: Option<GapStart>,
    /// 意图那次的 txg，按对象（PC-ELr 植入的版本要早于它）。
    pub(super) intent_txgs: BTreeMap<u64, u64>,
    /// 第二遍跑时在哪几次扫描上做阳性对照（第一遍的扫描记录定）。
    pub(super) positive_control_plan: PositiveControlPlan,
    pub(super) positive_control_outcomes: Vec<PositiveControlOutcome>,
    /// 做完（最后一批已发布）的 unlink 数。
    pub(super) unlinks_completed: u64,
    /// 最后一批让一个身份消失、而那个身份最后那一版里装着这个对象 nlink = 0 的记录（PC-Bt1 的取样）。
    pub(super) dead_versions_holding_a_finished_record: Vec<DeadVersionHoldingAFinishedRecord>,
    /// 第四段的探针与对照（第二、三段是 None）。
    pub(super) segment_four: Option<SegmentFourProbes>,
}
