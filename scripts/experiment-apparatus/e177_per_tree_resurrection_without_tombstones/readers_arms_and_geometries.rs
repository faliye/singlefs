//! 读侧三种、三条臂、写路径与三个几何。

/// 扫描重建的三种读法（登记 5.4）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Reader {
    UnitScan,
    SelectCurrent,
    SelectCurrentGatedByObject,
}

pub(super) const READERS: [Reader; 3] = [
    Reader::UnitScan,
    Reader::SelectCurrent,
    Reader::SelectCurrentGatedByObject,
];

impl Reader {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Reader::UnitScan => "R-U",
            Reader::SelectCurrent => "R-S",
            Reader::SelectCurrentGatedByObject => "R-G",
        }
    }
}

/// 墓碑政策三条臂（登记 5.3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Arm {
    WithoutTombstones,
    TodaysTombstones,
    DeathRecordForEveryKey,
}

pub(super) const ARMS: [Arm; 3] = [
    Arm::WithoutTombstones,
    Arm::TodaysTombstones,
    Arm::DeathRecordForEveryKey,
];

impl Arm {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Arm::WithoutTombstones => "N",
            Arm::TodaysTombstones => "B",
            Arm::DeathRecordForEveryKey => "P",
        }
    }

    /// 臂 P 的死亡记录放在模型专用的区、不占槽（登记 5.3），写路径与臂 N 相同；臂 B 在意图那次多写墓碑。
    pub(super) const fn write_path(self) -> WritePath {
        match self {
            Arm::WithoutTombstones | Arm::DeathRecordForEveryKey => WritePath::WithoutTombstones,
            Arm::TodaysTombstones => WritePath::WithTombstones,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum WritePath {
    WithoutTombstones,
    WithTombstones,
}

impl WritePath {
    pub(super) fn arms(self) -> Vec<Arm> {
        ARMS.iter().copied().filter(|arm| arm.write_path() == self).collect()
    }
}

/// 三个几何（修订 1 第 12.3 节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Geometry {
    Ring,
    Wide,
    Tight,
}

pub(super) const GEOMETRIES: [Geometry; 3] = [Geometry::Ring, Geometry::Wide, Geometry::Tight];

impl Geometry {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Geometry::Ring => "ring",
            Geometry::Wide => "wide",
            Geometry::Tight => "tight",
        }
    }
}
