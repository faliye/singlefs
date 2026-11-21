use super::*;

// ============================================================================
// 二、取样域：组、几何点与每一段历史的参数（登记第五节取样域表、第八节几何敏感性表）。一段历史 = 一个组的一个参数格，
//    从 mkfs 起自己造、自己判（登记第十四节「独立单位」）。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Group {
    ShortRing,
    SkippedRow,
    SweepCrash,
    SweepRead,
    SweepNone,
    NewestRowBad,
    RollbackSlotBad,
}

impl Group {
    /// 产物里的组名（登记第五节取样域表第一列）。
    pub(super) fn name(self) -> &'static str {
        match self {
            Group::ShortRing => "ss",
            Group::SkippedRow => "skr",
            Group::SweepCrash => "sw-crash",
            Group::SweepRead => "sw-read",
            Group::SweepNone => "sw-none",
            Group::NewestRowBad => "nrb",
            Group::RollbackSlotBad => "rs1",
        }
    }
}

/// 取样点：主取样点与第八节的四个几何敏感性点。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum SamplingPoint {
    Main,
    SweepThresholdProductDefault,
    LongJournalRing,
    SkippedRowWiderRing,
    NewestRowAndRollbackWiderRing,
}

impl SamplingPoint {
    pub(super) fn name(self) -> &'static str {
        match self {
            SamplingPoint::Main => "main",
            SamplingPoint::SweepThresholdProductDefault => "G1",
            SamplingPoint::LongJournalRing => "G2",
            SamplingPoint::SkippedRowWiderRing => "G3",
            SamplingPoint::NewestRowAndRollbackWiderRing => "G4",
        }
    }
}

/// 用户改动的写法：3000 字节覆盖写（`publish_overwrite`），或给定长度的顺序写（`publish_sequential_write`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum UserWrite {
    Overwrite,
    SequentialWrite { content_bytes: usize },
}

/// rs1 回退之后的动作（登记第五节「动作的写法照线索模型 `rollback_hole.rs`」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum RollbackAction {
    Overwrite,
    LongerSequentialWrite,
    NewInode,
    EmptyPublish,
}

impl RollbackAction {
    pub(super) fn name(self) -> &'static str {
        match self {
            RollbackAction::Overwrite => "over",
            RollbackAction::LongerSequentialWrite => "sequential",
            RollbackAction::NewInode => "inodes",
            RollbackAction::EmptyPublish => "empty",
        }
    }
}

const ROLLBACK_ACTIONS: [RollbackAction; 4] = [
    RollbackAction::Overwrite,
    RollbackAction::LongerSequentialWrite,
    RollbackAction::NewInode,
    RollbackAction::EmptyPublish,
];

/// sw 三组崩不崩：sw-crash 在 b + 3S 那一次覆盖写崩，sw-none 不崩；sw-read 不崩、在实例 3 的每一版上取镜像。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum SweepVariant {
    CrashBeforeTheNextInstance,
    NoCrash,
    ReadFaultOnTheSweepInstance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HistoryShape {
    /// ss：实例 1 覆盖写到 last → last + 1 覆盖写崩 → 实例 2 产品挂载 → 顺序写 k 次（内容 c × 524 288 字节），k = 0 … 3S 每个 k 一张。
    ShortRing { last: u64, chunks: u64 },
    /// skr：实例 1 到 last → 实例 2 产品挂载崩在写行那次根槽写之前 → 实例 3 产品挂载 → 覆盖写 k 次，k = 0 … 14。
    SkippedRow { last: u64 },
    /// sw 三组与 G1：每次挂载走带清扫配置的入口。实例 1 到 a → 实例 2 覆盖写到 b = a + 3S + d（G1：实例 2 起每次挂载只写行与暖机、
    /// 攒到清扫发生为止，b 取清扫那次挂载之前的最后一版）→ 清扫那个实例覆盖写到 b + 3S − 1 → 崩或不崩 → 下一个实例挂载 → 覆盖写 k 次。
    Sweep {
        first_instance_last: u64,
        extra_overwrites: u64,
        threshold: u64,
        accumulate_rows_by_mounts: bool,
        variant: SweepVariant,
    },
    /// nrb：实例 1 到 last → 实例 2 产品挂载（写行 r）→ 用户写 k 次 → 读坏 r … r + bad − 1 的根槽。
    NewestRowBad {
        last: u64,
        steps_after_the_mount: u64,
        bad_slots: u64,
        user_write: UserWrite,
    },
    /// rs1：实例 1 到 10 → 实例 2 产品挂载（写行 r）→ 回退到 (实例 1, target)（那一版 v）→ 动作串 → 读坏 r … R − 1 的根槽，R ∈ {v, …, v + 动作数}。
    RollbackSlotBad {
        target: u64,
        actions: Vec<RollbackAction>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HistorySpecification {
    pub(super) group: Group,
    pub(super) point: SamplingPoint,
    pub(super) slots_per_region: u64,
    pub(super) journal_ring_bytes: u64,
    pub(super) shape: HistoryShape,
}

/// rs1 的回退目标、nrb 那一组实例 1 写到哪一版：登记第五节取样域表。
pub(super) const ROLLBACK_FIRST_INSTANCE_LAST: u64 = 10;

fn user_write_name(user_write: UserWrite) -> String {
    match user_write {
        UserWrite::Overwrite => "overwrite".to_string(),
        UserWrite::SequentialWrite { content_bytes } => format!("sequential{content_bytes}"),
    }
}

pub(super) fn sweep_variant_group(variant: SweepVariant) -> Group {
    match variant {
        SweepVariant::CrashBeforeTheNextInstance => Group::SweepCrash,
        SweepVariant::NoCrash => Group::SweepNone,
        SweepVariant::ReadFaultOnTheSweepInstance => Group::SweepRead,
    }
}

/// 历史名：组、取样点、S、环长与参数格，产物里每一行带着它；按它排序就是产物的次序。
pub(super) fn history_name(specification: &HistorySpecification) -> String {
    let prefix = format!(
        "{}.{}.S{}.ring{}",
        specification.group.name(),
        specification.point.name(),
        specification.slots_per_region,
        specification.journal_ring_bytes
    );
    match &specification.shape {
        HistoryShape::ShortRing { last, chunks } => format!("{prefix}.last{last}.c{chunks}"),
        HistoryShape::SkippedRow { last } => format!("{prefix}.last{last}"),
        HistoryShape::Sweep {
            first_instance_last,
            extra_overwrites,
            threshold,
            accumulate_rows_by_mounts,
            variant: _,
        } => format!(
            "{prefix}.a{first_instance_last}.d{extra_overwrites}.threshold{threshold}.accumulate{accumulate_rows_by_mounts}"
        ),
        HistoryShape::NewestRowBad { last, steps_after_the_mount, bad_slots, user_write } => format!(
            "{prefix}.last{last}.k{steps_after_the_mount}.bad{bad_slots}.{}",
            user_write_name(*user_write)
        ),
        HistoryShape::RollbackSlotBad { target, actions } => {
            let names: Vec<&str> = actions.iter().map(|action| action.name()).collect();
            let actions_text = if names.is_empty() { "-".to_string() } else { names.join("+") };
            format!("{prefix}.target{target}.actions{actions_text}")
        }
    }
}

/// 一段历史尝试造的张数（登记第七节 B1–B6 数的是「尝试」，含写者返回 `Err` 的格）。sw-read 的张数由清扫那一版的 txg 定，
/// 跑之前不知道（推的 315 不进锚点，登记第四节第 3 条），交 `None`。
pub(super) fn attempted_images(specification: &HistorySpecification) -> Option<u64> {
    match &specification.shape {
        HistoryShape::ShortRing { .. } => Some(3 * specification.slots_per_region + 1),
        HistoryShape::SkippedRow { .. } => Some(15),
        HistoryShape::Sweep { variant, .. } => match variant {
            SweepVariant::CrashBeforeTheNextInstance | SweepVariant::NoCrash => {
                Some(3 * specification.slots_per_region + 1)
            }
            SweepVariant::ReadFaultOnTheSweepInstance => None,
        },
        HistoryShape::NewestRowBad { .. } => Some(1),
        HistoryShape::RollbackSlotBad { actions, .. } => {
            Some(1 + u64::try_from(actions.len()).expect("动作数装得进 u64"))
        }
    }
}

fn newest_row_bad_specifications(
    point: SamplingPoint,
    slots_per_region: u64,
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for bad_slots in [1, 2] {
        specifications.push(HistorySpecification {
            group: Group::NewestRowBad,
            point,
            slots_per_region,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::NewestRowBad {
                last: 10,
                steps_after_the_mount: 3,
                bad_slots,
                user_write: UserWrite::Overwrite,
            },
        });
    }
    for steps_after_the_mount in 3..=6 {
        specifications.push(HistorySpecification {
            group: Group::NewestRowBad,
            point,
            slots_per_region,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::NewestRowBad {
                last: 4,
                steps_after_the_mount,
                bad_slots: 1,
                user_write: UserWrite::SequentialWrite {
                    content_bytes: NEWEST_ROW_BAD_SEQUENTIAL_CONTENT_BYTES,
                },
            },
        });
    }
    specifications
}

/// 动作串：{不做} ∪ {一个} ∪ {两个}，按线索模型 `rollback-sweep.sh` 的次序。
fn rollback_action_strings() -> Vec<Vec<RollbackAction>> {
    let mut strings = vec![Vec::new()];
    for first in ROLLBACK_ACTIONS {
        strings.push(vec![first]);
        for second in ROLLBACK_ACTIONS {
            strings.push(vec![first, second]);
        }
    }
    strings
}

fn rollback_specifications(
    point: SamplingPoint,
    slots_per_region: u64,
    targets: &[u64],
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for target in targets {
        for actions in rollback_action_strings() {
            specifications.push(HistorySpecification {
                group: Group::RollbackSlotBad,
                point,
                slots_per_region,
                journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
                shape: HistoryShape::RollbackSlotBad {
                    target: *target,
                    actions,
                },
            });
        }
    }
    specifications
}

fn short_ring_specifications(
    point: SamplingPoint,
    slots_per_region_list: &[u64],
    journal_ring_bytes: u64,
) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for slots_per_region in slots_per_region_list {
        for last in [4, 6, 8] {
            for chunks in [2, 3, 4] {
                specifications.push(HistorySpecification {
                    group: Group::ShortRing,
                    point,
                    slots_per_region: *slots_per_region,
                    journal_ring_bytes,
                    shape: HistoryShape::ShortRing { last, chunks },
                });
            }
        }
    }
    specifications
}

fn skipped_row_specifications(
    point: SamplingPoint,
    slots_per_region: u64,
) -> Vec<HistorySpecification> {
    [6, 8, 10]
        .into_iter()
        .map(|last| HistorySpecification {
            group: Group::SkippedRow,
            point,
            slots_per_region,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::SkippedRow { last },
        })
        .collect()
}

fn sweep_specifications(variant: SweepVariant) -> Vec<HistorySpecification> {
    let mut specifications = Vec::new();
    for slots_per_region in [4, 5, 6] {
        for first_instance_last in [4, 6, 8] {
            for extra_overwrites in [0, 1, 2] {
                specifications.push(HistorySpecification {
                    group: sweep_variant_group(variant),
                    point: SamplingPoint::Main,
                    slots_per_region,
                    journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
                    shape: HistoryShape::Sweep {
                        first_instance_last,
                        extra_overwrites,
                        threshold: SWEEP_THRESHOLD_ANY_ROW,
                        accumulate_rows_by_mounts: false,
                        variant,
                    },
                });
            }
        }
    }
    specifications
}

/// G1：sw-crash 的清扫阈值从 0 改成产品默认（修订后 365），S = 4、a = 4，实例 2 起每次挂载只写行与暖机、攒到清扫发生（登记第八节 G1）。
pub(super) fn sweep_threshold_product_default_specification() -> HistorySpecification {
    HistorySpecification {
        group: Group::SweepCrash,
        point: SamplingPoint::SweepThresholdProductDefault,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::Sweep {
            first_instance_last: 4,
            extra_overwrites: 0,
            threshold: SWEEP_THRESHOLD_PRODUCT_DEFAULT,
            accumulate_rows_by_mounts: true,
            variant: SweepVariant::CrashBeforeTheNextInstance,
        },
    }
}

/// 一段（登记第五节「分段」）跑哪几组。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Segment {
    One,
    Two,
    Three,
}

impl Segment {
    pub(super) fn name(self) -> &'static str {
        match self {
            Segment::One => "segment-one",
            Segment::Two => "segment-two",
            Segment::Three => "segment-three",
        }
    }

    pub(super) fn from_name(name: &str) -> Option<Segment> {
        [Segment::One, Segment::Two, Segment::Three]
            .into_iter()
            .find(|segment| segment.name() == name)
    }

    /// 这一段的全部历史，按历史名排序（产物的次序）。
    pub(super) fn history_specifications(self) -> Vec<HistorySpecification> {
        let mut specifications = match self {
            Segment::One => {
                let mut specifications = newest_row_bad_specifications(SamplingPoint::Main, 4);
                specifications.extend(rollback_specifications(
                    SamplingPoint::Main,
                    4,
                    &[5, 6, 7, 8, 9],
                ));
                specifications.extend(newest_row_bad_specifications(
                    SamplingPoint::NewestRowAndRollbackWiderRing,
                    6,
                ));
                specifications.extend(rollback_specifications(
                    SamplingPoint::NewestRowAndRollbackWiderRing,
                    6,
                    &[5, 9],
                ));
                specifications
            }
            Segment::Two => {
                let mut specifications = short_ring_specifications(
                    SamplingPoint::Main,
                    &[4, 5, 6],
                    SHORT_JOURNAL_RING_BYTES,
                );
                specifications.extend(skipped_row_specifications(SamplingPoint::Main, 4));
                specifications.extend(sweep_specifications(
                    SweepVariant::CrashBeforeTheNextInstance,
                ));
                specifications.extend(sweep_specifications(
                    SweepVariant::ReadFaultOnTheSweepInstance,
                ));
                specifications.push(sweep_threshold_product_default_specification());
                specifications.extend(short_ring_specifications(
                    SamplingPoint::LongJournalRing,
                    &[4],
                    LONG_JOURNAL_RING_BYTES,
                ));
                specifications.extend(skipped_row_specifications(
                    SamplingPoint::SkippedRowWiderRing,
                    6,
                ));
                specifications
            }
            Segment::Three => sweep_specifications(SweepVariant::NoCrash),
        };
        specifications.sort_by_key(history_name);
        specifications
    }
}

#[cfg(test)]
mod tests;
