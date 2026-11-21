use super::*;

// ============================================================================
// 十一、本地常量回比（停机条款 S4、锚点 B7、B8）与不建历史的锚点（B1–B6 的闭式张数）。
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConstantCheck {
    pub(super) name: &'static str,
    pub(super) local: u64,
    pub(super) implementation: u64,
}

impl ConstantCheck {
    pub(super) fn holds(&self) -> bool {
        self.local == self.implementation
    }
}

/// 出生 txg 的回比用的显眼值：写进一条真根的实例表指针，`to_slot` 之后按本地偏移读回来要等于它。
const DISTINCTIVE_POINTER_BIRTH_TXG: u64 = 0x0102_0304_0506_0708;

/// 偏移 170 + 42 的回比：mkfs、挂载、写第一个文件得到一条真根，把它实例表指针的出生 txg 换成显眼值，`RootRecord::to_slot` 编码，
/// 按本地偏移读回的 8 字节要等于显眼值。建不出那条根时交回 `None`（照 S4 停机，不当 0）。
fn pointer_birth_offset_read_back() -> Option<u64> {
    let specification = HistorySpecification {
        group: Group::NewestRowBad,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::SkippedRow { last: 3 },
    };
    let mut builder = HistoryBuilder::new(&specification);
    let session = build_the_first_instance(
        &mut builder,
        MountConfiguration::Product,
        3,
        UserWrite::Overwrite,
    )
    .ok()?;
    let mut root = *session.current.root();
    root.instance_table.head.birth_txg = CheckpointTxg(DISTINCTIVE_POINTER_BIRTH_TXG);
    pointer_birth_txg_in_root_slot_bytes(
        &root.to_slot(usize::try_from(FIXED_STRUCTURE_SLOT_SPACING).expect("4096 装得进 usize")),
    )
}

/// 第五节本地常量表逐个对 `crates/` 那一处（S4）；B8 修订后比的是产品默认阈值。
pub(super) fn constant_checks() -> Vec<ConstantCheck> {
    vec![
        ConstantCheck {
            name: "root_record_bytes_212_to_220_hold_the_instance_table_pointer_birth_txg",
            local: DISTINCTIVE_POINTER_BIRTH_TXG,
            implementation: pointer_birth_offset_read_back().unwrap_or(0),
        },
        ConstantCheck {
            name: "root_ring_regions",
            local: ROOT_RING_REGIONS,
            implementation: singlefs_format::ROOT_RING_REGIONS,
        },
        ConstantCheck {
            name: "sweep_threshold_product_default",
            local: SWEEP_THRESHOLD_PRODUCT_DEFAULT,
            implementation: InstanceTableSweepConfiguration::PRODUCT
                .row_count_threshold
                .0,
        },
        ConstantCheck {
            name: "sweep_threshold_any_row",
            local: SWEEP_THRESHOLD_ANY_ROW,
            implementation: InstanceTableRowCountThreshold::ANY_ROW.0,
        },
        ConstantCheck {
            name: "instance_table_page_records",
            local: INSTANCE_TABLE_PAGE_RECORDS,
            implementation: singlefs_format::INSTANCE_TABLE_PAGE_RECORDS,
        },
        ConstantCheck {
            name: "genesis_checkpoint_txg",
            local: GENESIS_TXG,
            implementation: singlefs_core::make_filesystem::GENESIS_CHECKPOINT_TXG.0,
        },
    ]
}

/// 第七节「独立算出」那一类里的张数锚点（B1–B6）：(锚名, 组, 取样点, 登记写的尝试张数)。
pub(super) const ATTEMPTED_IMAGE_ANCHORS: [(&str, Group, SamplingPoint, u64); 10] = [
    ("B1", Group::ShortRing, SamplingPoint::Main, 432),
    ("B2", Group::SkippedRow, SamplingPoint::Main, 45),
    ("B3", Group::NewestRowBad, SamplingPoint::Main, 6),
    ("B4", Group::RollbackSlotBad, SamplingPoint::Main, 285),
    ("B5", Group::SweepCrash, SamplingPoint::Main, 432),
    (
        "B6",
        Group::RollbackSlotBad,
        SamplingPoint::NewestRowAndRollbackWiderRing,
        114,
    ),
    ("B6", Group::ShortRing, SamplingPoint::LongJournalRing, 117),
    (
        "B6",
        Group::SkippedRow,
        SamplingPoint::SkippedRowWiderRing,
        45,
    ),
    (
        "B6",
        Group::NewestRowBad,
        SamplingPoint::NewestRowAndRollbackWiderRing,
        6,
    ),
    (
        "B6",
        Group::SweepCrash,
        SamplingPoint::SweepThresholdProductDefault,
        13,
    ),
];

/// 第七节「出自被测条款本身」的张数锚点（A1–A4，不符走 F5，不作废）。
pub(super) const CLAUSE_IMAGE_ANCHORS: [(&str, Group, u64); 4] = [
    ("A1", Group::ShortRing, 432),
    ("A2", Group::SkippedRow, 45),
    ("A3", Group::NewestRowBad, 6),
    ("A4", Group::RollbackSlotBad, 285),
];

pub(super) fn attempted_images_of(
    specifications: &[HistorySpecification],
    group: Group,
    point: SamplingPoint,
) -> Option<u64> {
    specifications
        .iter()
        .filter(|specification| specification.group == group && specification.point == point)
        .map(attempted_images)
        .sum()
}

#[cfg(test)]
mod tests;
