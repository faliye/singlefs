use super::*;

pub(super) fn key(instance: u32, txg: u64) -> VersionKey {
    VersionKey { instance, txg }
}

pub(super) fn ring_version(instance: u32, txg: u64, birth: Option<u64>) -> VersionOnTheImage {
    VersionOnTheImage {
        key: key(instance, txg),
        pointer_birth: birth.map_or(PointerBirth::AllZeroOrUnreadable, PointerBirth::Readable),
    }
}

pub(super) fn ledger_version(
    parent: Option<VersionKey>,
    kind: PublishKind,
    pointer_birth_txg: u64,
    root_landed: bool,
    applied_by_a_mount: bool,
    publish_order: usize,
) -> LedgerVersion {
    LedgerVersion {
        parent,
        kind,
        pointer_birth_txg,
        root_landed,
        applied_by_a_mount,
        publish_order,
        rows: Arc::default(),
    }
}

/// 造历史的几条共用的核对：写者一步没停、每张都读得出，停机条款 S1、S2、S3 与账本自检都没触发（S5 另报，见登记修订）。
pub(super) fn assert_the_history_is_consistent(result: &HistoryResult, expected_images: usize) {
    assert_eq!(result.stopped, None, "{} 的写者没停", result.name);
    assert_eq!(
        result.images.len(),
        expected_images,
        "{} 的张数",
        result.name
    );
    assert!(
        result.ledger.consistency_problems.is_empty(),
        "{:?}",
        result.ledger.consistency_problems
    );
    assert!(
        result.ledger.ring_formula_mismatches.is_empty(),
        "{:?}",
        result.ledger.ring_formula_mismatches
    );
    assert!(
        result.ledger.root_slot_write_mismatches.is_empty(),
        "{:?}",
        result.ledger.root_slot_write_mismatches
    );
    assert!(
        result.ledger.root_slot_writes_compared > 0,
        "S2 一条都没比到"
    );
    for image in &result.images {
        assert!(
            image.reading.is_ok(),
            "{}/{}：{:?}",
            result.name,
            image.label,
            image.reading.as_ref().err()
        );
        assert!(
            image.ring_slot_mismatches.is_empty(),
            "{}/{}：{:?}",
            result.name,
            image.label,
            image.ring_slot_mismatches
        );
        assert_eq!(image.row_mismatch, None, "{}/{}", result.name, image.label);
    }
}

pub(super) fn sweep_specification(variant: SweepVariant) -> HistorySpecification {
    HistorySpecification {
        group: sweep_variant_group(variant),
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::Sweep {
            first_instance_last: 4,
            extra_overwrites: 0,
            threshold: SWEEP_THRESHOLD_ANY_ROW,
            accumulate_rows_by_mounts: false,
            variant,
        },
    }
}

pub(super) fn with_pointer_birth_e1(count: u64) -> ArmCounts {
    ArmCounts {
        today_e1: 0,
        today_e2: 0,
        with_pointer_birth_e1: count,
        with_pointer_birth_e2: 0,
    }
}
