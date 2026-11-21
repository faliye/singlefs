//! 环几何与档（登记 r2 5.1）：`RingGeometry`、卸载记号的奇偶、投影之外字段的生成器、判据集、建表来源与盘次序、`RunProfile` 与它给每条生成记录带的投影之外字段。

use super::*;

/// 环几何：只有槽数一个旋钮（登记第八节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RingGeometry {
    pub(super) ring_slots: u64,
}

impl RingGeometry {
    pub(super) const BIG: Self = Self {
        ring_slots: BIG_RING_SLOTS,
    };
    pub(super) const SMALL_SIX: Self = Self {
        ring_slots: SMALL_RING_SIX_SLOTS,
    };
    pub(super) const SMALL_NINE: Self = Self {
        ring_slots: SMALL_RING_NINE_SLOTS,
    };

    pub(super) fn ring_bytes(self) -> u64 {
        self.ring_slots * LOCAL_JOURNAL_RECORD_BYTES
    }

    /// 在飞上限，装置自己算：环槽数 ÷ F 取整（I-8.3 补句「N = 环槽数 ÷ F」）。
    pub(super) fn local_in_flight_limit(self) -> usize {
        usize::try_from(self.ring_slots / LOCAL_JOURNAL_SAFETY_FACTOR)
            .expect("环槽数不超过 64，商装得进 usize")
    }

    pub(super) fn ring_start_offset_in_bytes() -> u64 {
        LOCAL_JOURNAL_RING_START_SLOT * LOCAL_SLOT_BYTES
    }

    /// 记录落点，装置自己算：环起点 + ((计数器 − 第一个计数器) mod 环槽数) × 记录字节数。
    pub(super) fn local_record_offset_in_bytes(self, counter: u64) -> u64 {
        Self::ring_start_offset_in_bytes()
            + ((counter - LOCAL_FIRST_JOURNAL_COUNTER) % self.ring_slots)
                * LOCAL_JOURNAL_RECORD_BYTES
    }

    /// 历史的第一个计数器 c_0（登记 5.1）。
    pub(super) fn first_history_counter(self) -> u64 {
        if self.ring_slots == BIG_RING_SLOTS {
            BIG_RING_FIRST_HISTORY_COUNTER
        } else {
            self.ring_slots
        }
    }
}

// ── 档（登记 r2 5.1）：r1 档的生成器、判据与输出行同 r1 时的装置；r2 档投影之外的根字段随版本变，另判 J6Qn、J7

/// r2 档卸载记号的奇偶（登记 r2 第一节、第八节；`--marker-parity even|odd`，不给取 even）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MarkerParity {
    /// 偶数版 `WrittenByTheUnmountSequence`、奇数版 `NotWrittenByTheUnmountSequence`（R_0 是 `Written…`）。
    EvenVersionsWrittenByTheUnmountSequence,
    /// 反过来：奇数版 `Written…`、偶数版 `NotWritten…`（R_0 是 `NotWritten…`）。
    OddVersionsWrittenByTheUnmountSequence,
}

impl MarkerParity {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::EvenVersionsWrittenByTheUnmountSequence => "even",
            Self::OddVersionsWrittenByTheUnmountSequence => "odd",
        }
    }
}

/// 时间线上每一版根记录投影（六个字段）之外的四个字段怎么取。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FieldsBeyondTheProjectionGenerator {
    /// r1 档：全历史相同（fsid 常量、卸载记号 `NotWritten…`、实例表槽 69 000、分配记录树根空）。
    SameForEveryVersion,
    /// r2 档（登记 r2 第一节）：第 j 版实例表指针取槽 69 000 + j、分配记录树根取槽 68 000 + j、fsid 末字节取 j，卸载记号按奇偶。
    VaryWithTheVersion(MarkerParity),
}

/// 判哪几条判据：r1 档只判原登记与 r1 登记的；r2 档另判 J6Qn 两行与 J7 五行（登记 r2 5.4）；r3 档在 r2 档之上，对抗一族的 J6Qn 按 I-8.12 再拆。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CriteriaSet {
    RegisteredBeforeTheVaryingFieldsProfile,
    WithQnFixedPointAndCopiedFields,
    /// r3 档（登记 `research/prompts/e165-r3-prereg.md` 5.1、5.6）：r2 档全部判据原样，对抗一族的 J6Qn 再按 I-8.12 拆行。
    WithQnFixedPointAndCopiedFieldsSplitByTxgOrder,
    /// r5 档（登记 `research/prompts/e165-r5-prereg.md` 5.1、5.3）：r3 档的判据原样，只把 J7 五行换成 J8 五行与「无出处」一行；
    /// 这一档生成的记录带它那次发布的根的后三项（D23（journal 的角色与格式） 已定项 15 今天的原句）。
    WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder,
}

/// 判据一侧拿哪张记录表去判（登记 r2 5.4 T-表、T-行；`--table direct|scan|both`，不给取 `direct`，只在 r2 档有）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordTableSource {
    /// 装置自己建的表：生成每条记录时解一次 `parse`，按盘次序逐槽收，`(实例, 计数器)` 先到先得（登记 5.1）。
    Direct,
    /// 把状态摆上内存盘，调 `scan_journal` 从盘上字节建表（登记 r2 第三节「由 scan_journal 建表」）。
    Scanned,
    /// 两张表各判一遍逐态比；汇总行取直接建表那一遍，另打 `name=table_comparison` 行。
    Both,
}

impl RecordTableSource {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Scanned => "scan",
            Self::Both => "both",
        }
    }
}

/// 内存盘的 `device_identities()` 报哪块盘在前（登记 r2 第八节；`--disk-order normal|reversed`，不给取 `normal`，只在 r2 档有）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReportedDiskOrder {
    FirstDeviceBeforeSecond,
    SecondDeviceBeforeFirst,
}

impl ReportedDiskOrder {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::FirstDeviceBeforeSecond => "normal",
            Self::SecondDeviceBeforeFirst => "reversed",
        }
    }

    pub(super) const fn device_numbers(self) -> [u32; 2] {
        match self {
            Self::FirstDeviceBeforeSecond => [0, 1],
            Self::SecondDeviceBeforeFirst => [1, 0],
        }
    }
}

/// 一次跑的档：生成器与判据两样，另有 r2 档第四段加的两个旋钮（记录表怎么建、盘次序）。`r1` 与 `r2` 是命令行给得出的两种；
/// 「r1 档生成器 + r2 判据」只给 J7 的阳性对照用（登记 r2 5.5「同一个对照在 r1 档生成器上 = 0」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RunProfile {
    pub(super) generator: FieldsBeyondTheProjectionGenerator,
    pub(super) criteria: CriteriaSet,
    pub(super) table_source: RecordTableSource,
    pub(super) reported_disk_order: ReportedDiskOrder,
}

/// 不给 `--profile` 时取的档（登记 r2 5.1）。
pub(super) const DEFAULT_PROFILE_LABEL: &str = "r1";
/// r1 档时间线根的实例表单元指针住的槽；r2 档第 j 版取它加 j。
pub(super) const INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION: u64 = 69_000;
/// r2 档第 j 版分配记录树根住的槽：它加 j。
const ALLOCATION_RECORD_TREE_ROOT_SLOT_OF_THE_FIRST_VERSION: u64 = 68_000;
/// 16 字节 fsid 里 r2 档随版本变的那一个字节：末字节（低 8 字节不动，登记 r2 第一节）。
const FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION: usize = 15;

impl RunProfile {
    pub(super) const CONSTANT_FIELDS: Self = Self {
        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
        criteria: CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile,
        table_source: RecordTableSource::Direct,
        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
    };

    pub(super) const fn varying_fields(marker_parity: MarkerParity) -> Self {
        Self {
            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            criteria: CriteriaSet::WithQnFixedPointAndCopiedFields,
            table_source: RecordTableSource::Direct,
            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
        }
    }

    /// r3 档（登记 r3 5.1）：生成器与 r2 档相同，判据是 r2 档的全部再加对抗一族的 I-8.12 拆行。
    pub(super) const fn txg_order_split(marker_parity: MarkerParity) -> Self {
        Self {
            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            criteria: CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder,
            table_source: RecordTableSource::Direct,
            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
        }
    }

    /// r5 档（登记 r5 5.1）：时间线根的投影之外四个字段同 r2 / r3 档，生成的记录带它那次发布的根的后三项，判 J8 不判 J7。
    pub(super) const fn root_fields_in_records(marker_parity: MarkerParity) -> Self {
        Self {
            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            criteria: CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder,
            table_source: RecordTableSource::Direct,
            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
        }
    }

    /// 只给 J8 的阳性对照用（登记 r5 5.4 末列「r1 档那种生成器」）：生成器照 r1 档（时间线根与记录的后三项恒为 r1 常量），判据照 r5 档。
    pub(super) const CONSTANT_FIELDS_GENERATOR_WITH_ROOT_FIELDS_IN_RECORDS_CRITERIA: Self = Self {
        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
        criteria: CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder,
        table_source: RecordTableSource::Direct,
        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
    };

    /// 只给 J7 的阳性对照用：生成器照 r1 档，判据照 r2 档。
    pub(super) const CONSTANT_FIELDS_GENERATOR_WITH_VARYING_FIELDS_CRITERIA: Self = Self {
        generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
        criteria: CriteriaSet::WithQnFixedPointAndCopiedFields,
        table_source: RecordTableSource::Direct,
        reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
    };

    pub(super) const fn with_table_source(self, table_source: RecordTableSource) -> Self {
        Self {
            generator: self.generator,
            criteria: self.criteria,
            table_source,
            reported_disk_order: self.reported_disk_order,
        }
    }

    pub(super) const fn with_reported_disk_order(
        self,
        reported_disk_order: ReportedDiskOrder,
    ) -> Self {
        Self {
            generator: self.generator,
            criteria: self.criteria,
            table_source: self.table_source,
            reported_disk_order,
        }
    }

    /// J6Qn 两行与多盘一族：r2、r3、r5 档判。
    pub(super) const fn judges_j6qn(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile => false,
            CriteriaSet::WithQnFixedPointAndCopiedFields
            | CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder
            | CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder => true,
        }
    }

    /// J7 五行（照「投影之外抄所选根」登记的旧规则）：只在 r2、r3 档判（r5 登记 5.1：旧档一个字不改）。
    pub(super) const fn judges_copied_fields(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile
            | CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder => false,
            CriteriaSet::WithQnFixedPointAndCopiedFields
            | CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder => true,
        }
    }

    /// J8 五行与「无出处」一行（登记 r5 5.3）：只在 r5 档判；这一档生成的记录同时带它那次发布的根的后三项。
    pub(super) const fn judges_new_root_segment_fields(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile
            | CriteriaSet::WithQnFixedPointAndCopiedFields
            | CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder => false,
            CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder => true,
        }
    }

    /// 对抗一族的 J6Qn 按 I-8.12 再拆（登记 r3 5.6）：r3 档与 r5 档（r5 档 = r3 档原样，只换 J7 → J8）。
    pub(super) const fn splits_j6qn_by_the_txg_order(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile
            | CriteriaSet::WithQnFixedPointAndCopiedFields => false,
            CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder
            | CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder => true,
        }
    }

    /// `--family root_choice` 认不认这一档：只认 r3 档（`--profile r3`、`--profile r4` 都解成它）；r5 档报用法错（登记 r5 5.1）。
    pub(super) const fn admits_the_root_choice_family(self) -> bool {
        match self.criteria {
            CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile
            | CriteriaSet::WithQnFixedPointAndCopiedFields
            | CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder => false,
            CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder => true,
        }
    }

    /// 两种建表法各判一遍逐态比（`--table both`）。
    pub(super) const fn compares_the_two_record_tables(self) -> bool {
        matches!(self.table_source, RecordTableSource::Both)
    }

    /// 时间线第 `version` 版 R_j 投影之外的四个字段。
    pub(super) fn fields_beyond_the_projection_of_version(
        self,
        version: usize,
    ) -> FieldsBeyondTheProjection {
        match self.generator {
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion => FieldsBeyondTheProjection {
                filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
                instance_table: node_pointer_at_slot(INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION, 1),
                allocation_record_tree_root: NodePointer::empty_root(),
            },
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity) => {
                let version_in_the_varying_fields = version;
                let version_number =
                    u64::try_from(version_in_the_varying_fields).expect("版本数不超过 12");
                let mut filesystem_identifier = ROOT_FILESYSTEM_IDENTIFIER;
                filesystem_identifier[FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION] =
                    u8::try_from(version_number).expect("版本数不超过 12，装得进一个字节");
                let version_is_even = version_number % 2 == 0;
                let written_by_the_unmount_sequence = match marker_parity {
                    MarkerParity::EvenVersionsWrittenByTheUnmountSequence => version_is_even,
                    MarkerParity::OddVersionsWrittenByTheUnmountSequence => !version_is_even,
                };
                FieldsBeyondTheProjection {
                    filesystem_identifier,
                    unmount_marker: if written_by_the_unmount_sequence {
                        UnmountMarker::WrittenByTheUnmountSequence
                    } else {
                        UnmountMarker::NotWrittenByTheUnmountSequence
                    },
                    instance_table: node_pointer_at_slot(
                        INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION + version_number,
                        1,
                    ),
                    allocation_record_tree_root: node_pointer_at_slot(
                        ALLOCATION_RECORD_TREE_ROOT_SLOT_OF_THE_FIRST_VERSION + version_number,
                        1,
                    ),
                }
            }
        }
    }
}

/// r5 档时间线之外那条记录（对抗一族字母表里的、R 域的残留）的实例表指针住的槽：这个数加区分号 d（登记 r5 5.3 第 2 条 (i)：
/// 对 d 单射，与时间线第 0..12 版的槽 69 000 + j 两两不同）。
const OFF_TIMELINE_RECORD_INSTANCE_TABLE_SLOT_BASE: u64 = 200_000;
/// 同上，分配记录树根住的槽：这个数加 d（与时间线第 0..12 版的槽 68 000 + j 两两不同）。
const OFF_TIMELINE_RECORD_ALLOCATION_RECORD_TREE_ROOT_SLOT_BASE: u64 = 300_000;

/// 一条记录新根段的后三项（D23（journal 的角色与格式） 已定项 15：实例表指针、分配记录树根、卸载记号；fsid 不在新根段里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordFieldsBeyondTheProjection {
    pub(super) instance_table: NodePointer,
    pub(super) allocation_record_tree_root: NodePointer,
    pub(super) unmount_marker: UnmountMarker,
}

impl RecordFieldsBeyondTheProjection {
    fn of_root_fields(fields: FieldsBeyondTheProjection) -> Self {
        Self {
            instance_table: fields.instance_table,
            allocation_record_tree_root: fields.allocation_record_tree_root,
            unmount_marker: fields.unmount_marker,
        }
    }

    pub(super) fn of_record(record: &JournalRecord) -> Self {
        Self {
            instance_table: record.new_instance_table,
            allocation_record_tree_root: record.new_allocation_record_tree_root,
            unmount_marker: record.new_unmount_marker,
        }
    }

    pub(super) fn of_root(root: &RootRecord) -> Self {
        Self {
            instance_table: root.instance_table,
            allocation_record_tree_root: root.allocation_record_tree_root,
            unmount_marker: root.unmount_marker,
        }
    }

    /// 见证里的写法：卸载记号、实例表与分配记录树根住的槽（空根是槽 0）。
    pub(super) fn describe(self) -> String {
        format!(
            "marker_{}:instance_table_slot{}:allocation_record_tree_root_slot{}",
            match self.unmount_marker {
                UnmountMarker::WrittenByTheUnmountSequence => "written",
                UnmountMarker::NotWrittenByTheUnmountSequence => "not_written",
            },
            self.instance_table.locations[0].slot.0,
            self.allocation_record_tree_root.locations[0].slot.0
        )
    }
}

/// 卸载记号的另一个值（登记 r5 5.3 第 2 条 (ii)：时间线之外的记录取与所选根 R_0 相反的那一值）。
const fn the_other_unmount_marker(marker: UnmountMarker) -> UnmountMarker {
    match marker {
        UnmountMarker::WrittenByTheUnmountSequence => UnmountMarker::NotWrittenByTheUnmountSequence,
        UnmountMarker::NotWrittenByTheUnmountSequence => UnmountMarker::WrittenByTheUnmountSequence,
    }
}

impl RunProfile {
    /// 时间线上第 `publish_ordinal` 次发布（`NewRootSegment::of_timeline_publish` 那一个序号）的每条记录带的后三项（登记 r5 5.3 第 1 条）：
    /// r5 档是时间线第 j 版 R_j 的那三项；别的档一律是 r1 档时间线那一份常量（r1–r4 档一个字不改）。
    pub(super) fn record_fields_of_the_timeline_publish(
        self,
        publish_ordinal: usize,
    ) -> RecordFieldsBeyondTheProjection {
        if self.judges_new_root_segment_fields() {
            RecordFieldsBeyondTheProjection::of_root_fields(
                self.fields_beyond_the_projection_of_version(publish_ordinal),
            )
        } else {
            RecordFieldsBeyondTheProjection::of_root_fields(
                fields_beyond_the_projection_carried_by_every_generated_record(),
            )
        }
    }

    /// 时间线之外区分号为 `distinguishing_number` 的那条记录带的后三项 h(d)（登记 r5 5.3 第 2 条）：r5 档且生成器随版本变时，
    /// 实例表指针、分配记录树根各住 d 的单射槽，卸载记号取与 R_0 相反的那一值；别的档与 r1 档那种生成器上是 r1 常量。
    pub(super) fn record_fields_off_the_timeline(
        self,
        distinguishing_number: u64,
    ) -> RecordFieldsBeyondTheProjection {
        let constant_record_fields = RecordFieldsBeyondTheProjection::of_root_fields(
            fields_beyond_the_projection_carried_by_every_generated_record(),
        );
        if !self.judges_new_root_segment_fields() {
            return constant_record_fields;
        }
        match self.generator {
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion => constant_record_fields,
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(_marker_parity) => {
                let marker_of_the_first_version = self
                    .fields_beyond_the_projection_of_version(0)
                    .unmount_marker;
                RecordFieldsBeyondTheProjection {
                    instance_table: node_pointer_at_slot(
                        OFF_TIMELINE_RECORD_INSTANCE_TABLE_SLOT_BASE + distinguishing_number,
                        1,
                    ),
                    allocation_record_tree_root: node_pointer_at_slot(
                        OFF_TIMELINE_RECORD_ALLOCATION_RECORD_TREE_ROOT_SLOT_BASE
                            + distinguishing_number,
                        1,
                    ),
                    unmount_marker: the_other_unmount_marker(marker_of_the_first_version),
                }
            }
        }
    }
}

/// 装置生成的每条记录的新根段后三项（实例表指针、分配记录树根、卸载记号；D23（journal 的角色与格式） 已定项 15
/// 「回到甲」之后施加一条记录时这三项取记录里的，不再抄所选根）：r1–r4 档一律带 r1 档时间线那一份常量。r1 档的所选根投影之外的字段就是这几个常量，
/// 施加结果与抄所选根逐态相同；r2 / r3 档的投影之外字段随版本变，J7 那几行判据照「投影之外抄所选根」登记，与新的施加规则对不上
/// （旧档一个字不改，r5 登记 5.1）。照今天原句重新登记的是 r5 档：记录带它那次发布的根的后三项（[`RunProfile::record_fields_of_the_timeline_publish`]），判 J8。
pub(super) fn fields_beyond_the_projection_carried_by_every_generated_record(
) -> FieldsBeyondTheProjection {
    RunProfile::CONSTANT_FIELDS.fields_beyond_the_projection_of_version(0)
}

/// 根记录里投影之外的四个字段（登记 r2 第一节「投影之外的根字段」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FieldsBeyondTheProjection {
    pub(super) filesystem_identifier: [u8; 16],
    pub(super) unmount_marker: UnmountMarker,
    pub(super) instance_table: NodePointer,
    pub(super) allocation_record_tree_root: NodePointer,
}

impl FieldsBeyondTheProjection {
    pub(super) fn of_root(root: &RootRecord) -> Self {
        Self {
            filesystem_identifier: root.filesystem_identifier,
            unmount_marker: root.unmount_marker,
            instance_table: root.instance_table,
            allocation_record_tree_root: root.allocation_record_tree_root,
        }
    }

    /// 见证里的写法：卸载记号、实例表与分配记录树根住的槽（空根是槽 0）、fsid 末字节。
    pub(super) fn describe(self) -> String {
        format!(
            "marker_{}:instance_table_slot{}:allocation_record_tree_root_slot{}:fsid_last_byte{}",
            match self.unmount_marker {
                UnmountMarker::WrittenByTheUnmountSequence => "written",
                UnmountMarker::NotWrittenByTheUnmountSequence => "not_written",
            },
            self.instance_table.locations[0].slot.0,
            self.allocation_record_tree_root.locations[0].slot.0,
            self.filesystem_identifier[FILESYSTEM_IDENTIFIER_BYTE_THAT_VARIES_WITH_THE_VERSION]
        )
    }
}

/// 系统配置 fsid 低 8 字节的本地取法：16 字节 fsid 的前 8 字节按小端读（登记 r2 第三节回比表；D22 已定项 13「盘上定长字段一律小端」）。
/// [`check_the_fields_beyond_the_projection_of_the_varying_fields_generator`] 拿它与 `singlefs_core::unit::unit_filesystem_identifier` 回比。
pub(super) fn local_filesystem_identifier_low_eight_bytes(filesystem_identifier: &[u8; 16]) -> u64 {
    let mut low_eight_bytes = [0u8; 8];
    low_eight_bytes.copy_from_slice(&filesystem_identifier[..8]);
    u64::from_le_bytes(low_eight_bytes)
}

#[cfg(test)]
mod root_choice_fault_tests;
