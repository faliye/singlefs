//! 几何、两维的政策、取样点与全部取样点。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Geometry {
    /// E113 的几何。
    UnknownTreeFullArms,
    /// `crates/` 今天的两盘池。
    ImplementationTwoDevices,
}

impl Geometry {
    pub(super) fn label(self) -> &'static str {
        match self {
            Geometry::UnknownTreeFullArms => "e113",
            Geometry::ImplementationTwoDevices => "implementation_two_devices",
        }
    }

    pub(super) fn rows_without_tree_dimension(self) -> u64 {
        match self {
            Geometry::UnknownTreeFullArms => E113_ROWS_WITHOUT_TREE_DIMENSION,
            Geometry::ImplementationTwoDevices => {
                CRATES_POOL_WIDE_ROWS - CRATES_POOL_WIDE_ROWS_WITH_TREE_DIMENSION + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT
            }
        }
    }

    pub(super) fn known_writable_head_count(self) -> u64 {
        match self {
            Geometry::UnknownTreeFullArms => E113_KNOWN_WRITABLE_HEAD_COUNT,
            Geometry::ImplementationTwoDevices => CRATES_KNOWN_WRITABLE_HEAD_COUNT,
        }
    }

    pub(super) fn known_head_step_rows(self) -> u64 {
        match self {
            Geometry::UnknownTreeFullArms => E113_KNOWN_HEAD_STEP_ROWS,
            Geometry::ImplementationTwoDevices => CRATES_KNOWN_HEAD_STEP_ROWS,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum UnknownFlagsPolicy {
    Ignore,
    Skip,
    RejectEntry,
    ReadOnly,
    PerBit,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum AccountingPolicy {
    Carry,
    NoCarry,
    /// 随根留着：不认识的树的行原样留在记账树里，不重写、不换代号。
    KeepWithRoot,
}

pub(super) const FLAGS_ARMS: [UnknownFlagsPolicy; 5] = [
    UnknownFlagsPolicy::Ignore,
    UnknownFlagsPolicy::Skip,
    UnknownFlagsPolicy::RejectEntry,
    UnknownFlagsPolicy::ReadOnly,
    UnknownFlagsPolicy::PerBit,
];
pub(super) const ACCOUNTING_ARMS: [AccountingPolicy; 3] = [AccountingPolicy::Carry, AccountingPolicy::NoCarry, AccountingPolicy::KeepWithRoot];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct SamplePoint {
    pub(super) geometry: Geometry,
    /// 已撤回的「只留最近 K 代」的 K：模型不读它（阴性对照丙），只进产物。
    pub(super) retired_kept_generation_count: u64,
    pub(super) invisible_window_generations: u64,
    pub(super) unknown_writable_head_count: u64,
    pub(super) trees_with_registered_bit: u64,
    pub(super) published_generations: u64,
    pub(super) earlier_step_count: u64,
    pub(super) initial_publish_changed_the_watermark: bool,
    pub(super) tree_dimension_statistic_count: u64,
}

impl SamplePoint {
    /// (`s`, `lpc`) = (0, 0) 是 E113 口径：头一级台阶行都没有，按今天 D5 已定项 2 射程「建克隆那次给新头留一行」不合法。
    pub(super) fn is_legal_under_todays_step_rows(&self) -> bool {
        self.earlier_step_count > 0 || self.initial_publish_changed_the_watermark
    }

    pub(super) fn describe(&self) -> String {
        format!(
            "geometry={} k_keep={} w_gens={} h_new={} registered={} g_gens={} s={} lpc={} r_tree_dim={}",
            self.geometry.label(),
            self.retired_kept_generation_count,
            self.invisible_window_generations,
            self.unknown_writable_head_count,
            self.trees_with_registered_bit,
            self.published_generations,
            self.earlier_step_count,
            u8::from(self.initial_publish_changed_the_watermark),
            self.tree_dimension_statistic_count
        )
    }
}

/// 全部取样点，次序固定（几何、k_keep、w_gens、h_new、registered、G、(s, lpc)、r_tree_dim 由外到内）。
pub(super) fn all_sample_points() -> Vec<SamplePoint> {
    let mut points = Vec::new();
    for geometry in GEOMETRIES {
        for retired_kept_generation_count in RETIRED_KEPT_GENERATION_COUNT_VALUES {
            for invisible_window_generations in INVISIBLE_WINDOW_GENERATION_VALUES {
                for unknown_writable_head_count in UNKNOWN_WRITABLE_HEAD_COUNT_VALUES {
                    for trees_with_registered_bit in REGISTERED_TREE_COUNT_VALUES {
                        for published_generations in PUBLISHED_GENERATION_VALUES {
                            for (earlier_step_count, initial_publish_changed_the_watermark) in STEP_HISTORY_VALUES {
                                for tree_dimension_statistic_count in TREE_DIMENSION_STATISTIC_COUNT_VALUES {
                                    points.push(SamplePoint {
                                        geometry,
                                        retired_kept_generation_count,
                                        invisible_window_generations,
                                        unknown_writable_head_count,
                                        trees_with_registered_bit,
                                        published_generations,
                                        earlier_step_count,
                                        initial_publish_changed_the_watermark,
                                        tree_dimension_statistic_count,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    points
}
