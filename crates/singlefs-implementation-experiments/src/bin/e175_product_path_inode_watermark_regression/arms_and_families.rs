//! 臂与族：施加了哪一臂的补丁、弄坏开关，每个族登记了多少段、几何点与权重组。
use super::*;

/// 被测的那一臂（登记第五节「臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Arm {
    Product,
    RollbackTakesTheTargetsWatermark,
    PublishDoesNotAddTheNewInodes,
    RollbackTakesOnlyTheRingsWatermark,
}

impl Arm {
    /// `E175_ARM` 的值：没设就拒，它说的是这棵树施加的是哪一臂的补丁。
    pub(super) fn from_variable_value(value: Option<&str>) -> Arm {
        match value.expect("E175_ARM 没设：跑的一方施加补丁之后设 P、V1、V2 或 V717")
        {
            "P" => Arm::Product,
            "V1" => Arm::RollbackTakesTheTargetsWatermark,
            "V2" => Arm::PublishDoesNotAddTheNewInodes,
            "V717" => Arm::RollbackTakesOnlyTheRingsWatermark,
            other => panic!("E175_ARM 认不得「{other}」：只认 P、V1、V2、V717"),
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Arm::Product => "P",
            Arm::RollbackTakesTheTargetsWatermark => "V1",
            Arm::PublishDoesNotAddTheNewInodes => "V2",
            Arm::RollbackTakesOnlyTheRingsWatermark => "V717",
        }
    }
}

impl Breakage {
    pub(super) const NONE: Breakage = Breakage {
        ledger_ignores_birth: false,
        every_root_abandoned: false,
        older_birth_always_still_present: false,
    };

    /// `E175_BREAK` 的值：没设或空串是不弄坏。
    pub(super) fn from_variable_value(value: Option<&str>) -> Breakage {
        match value {
            None | Some("") => Breakage::NONE,
            Some("ledger-ignores-birth") => Breakage {
                ledger_ignores_birth: true,
                ..Breakage::NONE
            },
            Some("every-root-abandoned") => Breakage {
                every_root_abandoned: true,
                ..Breakage::NONE
            },
            Some("q9-always-still-present") => Breakage {
                older_birth_always_still_present: true,
                ..Breakage::NONE
            },
            Some(other) => panic!("E175_BREAK 认不得「{other}」：见文件头那张表"),
        }
    }
}

/// 历史族（登记第五节「历史族」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Family {
    /// 定向，钉值用：D1、D2(1..=6)、D3，序号 0 是 D1、1..=6 是 D2(k)、7 是 D3。
    Directed,
    /// 定向穷举：前缀 `[Unlink, CreateInode, CreateInode]`、中段取字母表上的序列、后缀 `[CreateInode, CreateInode]`。
    DirectedExhaustive,
    /// 带回退的随机历史：登记那组比重，种子 = 种子基 + 序号。
    RandomWithRollbacks,
    /// F1 现有随机历史：一组现有命名比重，种子 = 种子基 + 序号，30 步。
    ExistingRandom(ExistingWeightGroup),
    /// 第八节几何敏感性的一个点（G0 就是 F2 本身）。
    Geometry(GeometryPoint),
}

/// F1 的四组现有命名比重（登记第五节 F1 那一行）：都会在 unlink 之后抽 `CreateInode`；另四组不抽它，不跑。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExistingWeightGroup {
    Broad,
    ReuseAfterRaisingTheFloor,
    DeletingFilesOfSeveralLowerLeaves,
    NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
}

impl ExistingWeightGroup {
    const ALL: [ExistingWeightGroup; 4] = [
        ExistingWeightGroup::Broad,
        ExistingWeightGroup::ReuseAfterRaisingTheFloor,
        ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves,
        ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    ];

    pub(super) fn weights(self) -> GenerationWeights {
        match self {
            ExistingWeightGroup::Broad => GenerationWeights::BROAD,
            ExistingWeightGroup::ReuseAfterRaisingTheFloor => {
                GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR
            }
            ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves => {
                GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES
            }
            ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS
            }
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            ExistingWeightGroup::Broad => "F1-broad",
            ExistingWeightGroup::ReuseAfterRaisingTheFloor => "F1-reuse-after-raising-the-floor",
            ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves => {
                "F1-deleting-files-of-several-lower-leaves"
            }
            ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                "F1-newest-system-configuration-slot-reading-zeros"
            }
        }
    }
}

/// 第八节几何敏感性的四个点（登记第八节那张表；G0 是第一段的 F2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GeometryPoint {
    /// G-long：F2 比重，150 步。
    LongerRandomWithRollbacks,
    /// G-small：F2 比重，60 步，`UnitAreaOf384Slots`。
    RandomWithRollbacksOnSmallDevices,
    /// G-draw：F2 比重换成 `RecentUniformOrBeyond` 回退抽法，60 步。
    RandomWithRollbacksDrawingRecentUniformOrBeyond,
    /// G-long-F1：`BROAD`，150 步。
    LongerBroad,
}

impl GeometryPoint {
    const ALL: [GeometryPoint; 4] = [
        GeometryPoint::LongerRandomWithRollbacks,
        GeometryPoint::RandomWithRollbacksOnSmallDevices,
        GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond,
        GeometryPoint::LongerBroad,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            GeometryPoint::LongerRandomWithRollbacks => "G-long",
            GeometryPoint::RandomWithRollbacksOnSmallDevices => "G-small",
            GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond => "G-draw",
            GeometryPoint::LongerBroad => "G-long-F1",
        }
    }
}

impl Family {
    /// `E175_FAMILY` 点名的那一族；不给就是这一臂登记了段的全部族。
    pub(super) fn selected_by_the_variable_value(value: Option<&str>, arm: Arm) -> Vec<Family> {
        match value {
            Some(value) => {
                vec![Family::from_label(value)
                    .unwrap_or_else(|| panic!("E175_FAMILY 认不得「{value}」：见文件头列的族名"))]
            }
            None => Family::every_family()
                .into_iter()
                .filter(|family| family.registered_history_count(arm) > 0)
                .collect(),
        }
    }

    /// 每一族，按结果行里的族名次序：D、F3a、F2、F1 四组、几何四点。
    fn every_family() -> Vec<Family> {
        [
            Family::Directed,
            Family::DirectedExhaustive,
            Family::RandomWithRollbacks,
        ]
        .into_iter()
        .chain(
            ExistingWeightGroup::ALL
                .into_iter()
                .map(Family::ExistingRandom),
        )
        .chain(GeometryPoint::ALL.into_iter().map(Family::Geometry))
        .collect()
    }

    fn from_label(label: &str) -> Option<Family> {
        Family::every_family()
            .into_iter()
            .find(|family| family.label() == label)
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Family::Directed => "D",
            Family::DirectedExhaustive => "F3a",
            Family::RandomWithRollbacks => "F2",
            Family::ExistingRandom(group) => group.label(),
            Family::Geometry(point) => point.label(),
        }
    }

    /// 这一族在这一臂上登记的段数。
    pub(super) fn registered_history_count(self, arm: Arm) -> u64 {
        match (self, arm) {
            (Family::Directed, _) => DIRECTED_HISTORY_COUNT,
            (Family::DirectedExhaustive, Arm::Product) => {
                directed_exhaustive_sequences_up_to_middle_length(
                    DIRECTED_EXHAUSTIVE_LONGEST_MIDDLE_ON_THE_PRODUCT_ARM,
                )
            }
            (
                Family::DirectedExhaustive,
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => directed_exhaustive_sequences_up_to_middle_length(
                DIRECTED_EXHAUSTIVE_LONGEST_MIDDLE_ON_THE_VARIANT_ARMS,
            ),
            (Family::RandomWithRollbacks, Arm::Product) => {
                RANDOM_WITH_ROLLBACKS_SEEDS_ON_THE_PRODUCT_ARM
            }
            (
                Family::RandomWithRollbacks,
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => RANDOM_WITH_ROLLBACKS_SEEDS_ON_THE_VARIANT_ARMS,
            (Family::ExistingRandom(_), Arm::Product) => EXISTING_RANDOM_SEEDS_ON_THE_PRODUCT_ARM,
            (
                Family::ExistingRandom(_),
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => EXISTING_RANDOM_SEEDS_ON_THE_VARIANT_ARMS,
            (Family::Geometry(GeometryPoint::LongerRandomWithRollbacks), Arm::Product) => {
                LONGER_RANDOM_SEEDS_ON_THE_PRODUCT_ARM
            }
            // V717 有条件跑（Q6 > 0 的族），规模照这一族 V1 那一格。
            (
                Family::Geometry(GeometryPoint::LongerRandomWithRollbacks),
                Arm::RollbackTakesTheTargetsWatermark | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => LONGER_RANDOM_SEEDS_ON_THE_ROLLBACK_VARIANT_ARM,
            (Family::Geometry(GeometryPoint::RandomWithRollbacksOnSmallDevices), Arm::Product) => {
                SMALL_DEVICE_RANDOM_SEEDS_ON_THE_PRODUCT_ARM
            }
            (
                Family::Geometry(GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond),
                Arm::Product,
            ) => RECENT_DRAW_RANDOM_SEEDS_ON_THE_PRODUCT_ARM,
            (Family::Geometry(GeometryPoint::LongerBroad), Arm::Product) => {
                LONGER_BROAD_SEEDS_ON_THE_PRODUCT_ARM
            }
            // 第八节那张表写「不跑」的格。
            (
                Family::Geometry(GeometryPoint::LongerRandomWithRollbacks),
                Arm::PublishDoesNotAddTheNewInodes,
            )
            | (
                Family::Geometry(
                    GeometryPoint::RandomWithRollbacksOnSmallDevices
                    | GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond
                    | GeometryPoint::LongerBroad,
                ),
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => 0,
        }
    }

    /// 这一族每段几步（D、F3a 的步数由操作表定，这里不用）。
    pub(super) fn operations_per_random_history(self) -> Option<usize> {
        match self {
            Family::Directed | Family::DirectedExhaustive => None,
            Family::RandomWithRollbacks
            | Family::Geometry(
                GeometryPoint::RandomWithRollbacksOnSmallDevices
                | GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond,
            ) => Some(RANDOM_WITH_ROLLBACKS_OPERATIONS_PER_HISTORY),
            Family::ExistingRandom(_) => Some(EXISTING_RANDOM_OPERATIONS_PER_HISTORY),
            Family::Geometry(
                GeometryPoint::LongerRandomWithRollbacks | GeometryPoint::LongerBroad,
            ) => Some(LONG_GEOMETRY_OPERATIONS_PER_HISTORY),
        }
    }

    /// 这一族跑在什么盘上（登记第五节「每一步怎么跑」；只有 G-small 换盘宽）。
    pub(super) fn execution(self) -> HistoryExecution {
        match self {
            Family::Geometry(GeometryPoint::RandomWithRollbacksOnSmallDevices) => {
                EXECUTION_ON_SMALL_DEVICES
            }
            Family::Directed
            | Family::DirectedExhaustive
            | Family::RandomWithRollbacks
            | Family::ExistingRandom(_)
            | Family::Geometry(
                GeometryPoint::LongerRandomWithRollbacks
                | GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond
                | GeometryPoint::LongerBroad,
            ) => EXECUTION,
        }
    }
}

#[cfg(test)]
mod tests;
