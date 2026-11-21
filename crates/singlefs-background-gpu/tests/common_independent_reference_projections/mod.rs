//! 把产品与核心层的类型换成独立参照（`common_independent_references`）的入参与结果：只读字段，不调产品与核心层的计算函数。
//! 独立参照的函数本身不碰这些类型，三方比的时候经这里接上。要先 `mod common_independent_references;` 再 mod 进这一份。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
use singlefs_core::region_scan::{
    AllocationInRegion, CentralMappingEntry, MappedUnitClass, RegionScanInconsistency,
};
use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};

use crate::common_independent_references::{
    live_slots_per_full_segment_by_coverage_changes, region_record_class_worked_out_independently,
    AllocationRecordWords, MappingEntryWords, RegionRecordClassWorkedOutIndependently,
    UnitAreaWords,
};

#[must_use]
pub fn allocation_record_words_of(record: &AllocationRecord) -> AllocationRecordWords {
    AllocationRecordWords {
        device: record.device.0,
        first_slot: record.slot.0,
        span_slots: u64::from(record.span_slots),
        is_released: record.is_released,
    }
}

#[must_use]
pub fn unit_area_words_of(unit_area: UnitAreaOfOneDevice) -> UnitAreaWords {
    UnitAreaWords {
        device: unit_area.device.0,
        first_slot: unit_area.start.slot().0,
        slot_count: unit_area.slots,
    }
}

/// 独立参照数的每段活槽数（[`live_slots_per_full_segment_by_coverage_changes`]），入参是产品那边的记录与单元区。
#[must_use]
pub fn live_slots_worked_out_independently_on(
    records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<u64> {
    let record_words: Vec<AllocationRecordWords> =
        records.iter().map(allocation_record_words_of).collect();
    live_slots_per_full_segment_by_coverage_changes(&record_words, unit_area_words_of(unit_area))
}

/// 独立参照给一条记录的分类（[`region_record_class_worked_out_independently`]），入参是产品那边的记录、读到的字节与中央映射。
#[must_use]
pub fn region_record_class_worked_out_independently_on(
    record: &AllocationRecord,
    unit_bytes: Option<&[u8]>,
    mapping_entries: &[CentralMappingEntry],
) -> RegionRecordClassWorkedOutIndependently {
    let mapping_words: Vec<MappingEntryWords<'_>> = mapping_entries
        .iter()
        .map(|entry| MappingEntryWords {
            key: &entry.key,
            locations: entry
                .locations
                .map(|location| (location.device.0, location.slot.0)),
        })
        .collect();
    region_record_class_worked_out_independently(
        record.device.0,
        record.slot.0,
        u64::from(record.span_slots),
        unit_bytes,
        &mapping_words,
    )
}

/// 产品给的一条分类换成独立参照的那一种：可搬的带映射 key、类标签与诞生代号，不收的只留类别（读不出头的不分是哪一项头字段坏了）。
///
/// # Panics
/// 钉住、隔离：对拍交进来的记录都不钉住、不隔离，产品给出这两类就是错。
#[must_use]
pub fn class_of_the_product_classification(
    classification: &AllocationInRegion,
) -> RegionRecordClassWorkedOutIndependently {
    match classification {
        AllocationInRegion::Movable(unit) => RegionRecordClassWorkedOutIndependently::Movable {
            mapping_key: unit.mapping_key.clone(),
            unit_class_tag: match unit.unit_class {
                MappedUnitClass::Data => UNIT_CLASS_DATA,
                MappedUnitClass::IndexNode => UNIT_CLASS_INDEX_NODE,
                MappedUnitClass::Packed => UNIT_CLASS_PACKED,
            },
            birth_txg: unit.birth_txg.0,
        },
        AllocationInRegion::Dead => RegionRecordClassWorkedOutIndependently::Dead,
        AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(_)) => {
            RegionRecordClassWorkedOutIndependently::HeaderUnreadable
        }
        AllocationInRegion::Inconsistent(
            RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey { .. },
        ) => RegionRecordClassWorkedOutIndependently::UnitClassDisagreesWithTheMappingKey,
        AllocationInRegion::Inconsistent(
            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
        ) => RegionRecordClassWorkedOutIndependently::HeaderKeyDisagreesWithTheMappingLocation,
        AllocationInRegion::Pinned | AllocationInRegion::Quarantined => {
            panic!("对拍交进来的记录都不钉住、不隔离，产品给出了 {classification:?}")
        }
    }
}

/// 产品的一条分类写成 `{:?}` 的文字（库里存的 CPU 参照）只看得出类别：可搬、已死与三种不一致各是哪一种；认不出的是 `None`。
#[must_use]
pub fn class_name_of_the_debug_text(debug_text: &str) -> Option<&'static str> {
    [
        ("Movable(", "Movable"),
        ("Dead", "Dead"),
        ("Inconsistent(HeaderUnreadable(", "HeaderUnreadable"),
        (
            "Inconsistent(UnitClassDisagreesWithTheMappingKey",
            "UnitClassDisagreesWithTheMappingKey",
        ),
        (
            "Inconsistent(HeaderKeyDisagreesWithTheMappingLocation",
            "HeaderKeyDisagreesWithTheMappingLocation",
        ),
    ]
    .into_iter()
    .find(|(prefix, _name)| debug_text.starts_with(prefix))
    .map(|(_prefix, name)| name)
}

/// 独立参照那一类的类别名（与 [`class_name_of_the_debug_text`] 同一套名字）。
#[must_use]
pub fn class_name_of_the_independent_class(
    class: &RegionRecordClassWorkedOutIndependently,
) -> &'static str {
    match class {
        RegionRecordClassWorkedOutIndependently::Movable { .. } => "Movable",
        RegionRecordClassWorkedOutIndependently::Dead => "Dead",
        RegionRecordClassWorkedOutIndependently::HeaderUnreadable => "HeaderUnreadable",
        RegionRecordClassWorkedOutIndependently::UnitClassDisagreesWithTheMappingKey => {
            "UnitClassDisagreesWithTheMappingKey"
        }
        RegionRecordClassWorkedOutIndependently::HeaderKeyDisagreesWithTheMappingLocation => {
            "HeaderKeyDisagreesWithTheMappingLocation"
        }
    }
}
