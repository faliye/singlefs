//! 几何取样点与第七节 B 类锚点：最小可用盘的搜索与独立算术的锚点的单测。

use super::*;

const LEDGER_OF_THE_FIRST_VERSION: LedgerForTheFill = LedgerForTheFill {
    ledger: None,
    set_ledger: leave_the_ledger_as_it_is,
};

#[test]
fn smallest_usable_disk_of_the_steps_one_to_three_reading_fills_while_the_steps_one_and_two_disk_and_the_disk_two_slots_smaller_do_not(
) {
    for slots_per_region in [
        ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
    ] {
        let search = smallest_usable_geometry(slots_per_region, 1, LEDGER_OF_THE_FIRST_VERSION);
        let written = search
            .written
            .expect("一个单元的文件在原定义下找得到最小可用盘");
        let fillable = search
            .written_and_fillable
            .expect("读法 C 的最小盘在搜索上沿之内找得到");
        assert!(
            free_slots_after_the_fill(fillable, 1, LEDGER_OF_THE_FIRST_VERSION).is_some(),
            "S = {slots_per_region}：读法 C 找到的盘（单元区 {} 槽）第 3 步压得满",
            fillable.unit_area_slots()
        );
        assert!(
            !pool_is_written_and_fillable(written, 1, LEDGER_OF_THE_FIRST_VERSION),
            "S = {slots_per_region}：原定义的 P1（单元区 {} 槽）压不满（2026-10-01 第一段 r4 原定义下 P1 的 40 格全 not_fillable）——读法 C 真把盘往大挪了",
            written.unit_area_slots()
        );
        let two_slots_smaller = smallest_usable_candidate(
            slots_per_region,
            fillable.journal_ring_bytes,
            fillable.unit_area_slots() - 2,
        );
        assert!(
            !pool_is_written_and_fillable(two_slots_smaller, 1, LEDGER_OF_THE_FIRST_VERSION),
            "S = {slots_per_region}：小 2 槽那一格第 1–3 步做不成（登记 5.7「找到之后再核小 2 槽那一格做不成」）"
        );
    }
}

#[test]
fn smallest_disk_journal_ring_for_a_file_of_88_data_units_is_two_mebibytes_and_one_mebibyte_refuses_the_write(
) {
    for (records_of_the_publish, expected_ring_bytes) in [
        (0, 1_048_576),
        (1, 1_048_576),
        (85, 1_048_576),
        (86, 2_097_152),
        (88, 2_097_152),
        (170, 2_097_152),
        (171, 3_145_728),
    ] {
        assert_eq!(
            smallest_legal_journal_ring_holding(records_of_the_publish),
            expected_ring_bytes,
            "{records_of_the_publish} 条记录：1 MiB 的上限 ⌊1048576 ÷ 12288⌋ = 85、2 MiB 的 170，环按 1 MiB 一档取装得下的最小一档、不小于 1 MiB（修订第 15 条）"
        );
    }
    for journal_ring_bytes in [1 << 20, 2 << 20, 3 << 20, JOURNAL_RING_DEFAULT_BYTES] {
        assert_eq!(
            journal_record_limit_by_the_format(journal_ring_bytes),
            singlefs_core::system_configuration::journal_in_flight_record_limit(
                journal_ring_bytes
            ),
            "本地的记录数上限与 core 的 journal_in_flight_record_limit 在环 {journal_ring_bytes} 字节上相同"
        );
    }
    let ring = journal_ring_of_the_smallest_usable_disk(
        ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        88,
        JOURNAL_RING_ENLARGEMENTS_AT_MOST,
    );
    assert_eq!(
        ring,
        JournalRingOfTheSmallestUsableDisk {
            journal_ring_bytes: 2_097_152,
            tried: vec![
                (1_048_576, RingProbeOutcome::RecordsOverTheLimit(88)),
                (2_097_152, RingProbeOutcome::Written),
            ],
            not_settled: None,
        },
        "P1-S16、N = 88：1 MiB 的环上那次顺序写切 88 条记录、被上限 85 拒（第九个执行员 probe2.out 那几行），换成 2 MiB 写得进"
    );
    let search = smallest_usable_geometry(
        ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        88,
        LEDGER_OF_THE_FIRST_VERSION,
    );
    assert_eq!(
        search
            .written_and_fillable
            .map(|geometry| geometry.journal_ring_bytes),
        Some(2_097_152),
        "读法 C 的 P1 在 2 MiB 的环上找得到"
    );
    assert_eq!(
        search.smaller_journal_ring_refused_by_the_record_limit,
        Some(true),
        "小一档（1 MiB）的环在同一单元区上被记录数上限拒：2 MiB 是最小合法环"
    );
    assert_eq!(search.not_found_reason, None);
}

#[test]
fn smallest_usable_line_names_the_journal_record_limit_when_the_ring_may_not_grow() {
    let search = smallest_usable_geometry_with_the_ring_enlarged_at_most(
        ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        88,
        LEDGER_OF_THE_FIRST_VERSION,
        0,
    );
    assert_eq!(
        search.not_found_reason,
        Some(
            SmallestUsableNotFoundReason::JournalRecordsOfOnePublishExceedTheLimit(
                JournalRecordsOverTheLimit {
                    records_of_the_publish: 88,
                    record_limit: 85,
                }
            )
        ),
        "环不许放大（改之前的形态）时 N = 88 找不到盘，原因是记录数上限，不是盘的大小"
    );
    let line = smallest_usable_line(ROOT_RING_SLOTS_PER_REGION_MAXIMUM, 88, &search);
    for expected_field in [
        "unit_area_slots=not_found",
        "above_the_narrow_point=true",
        "journal_ring_bytes=1048576",
        "smaller_journal_ring_refused_by_the_record_limit=not_checked",
        "not_found_reason=journal_records_of_one_publish_exceed_the_limit",
    ] {
        assert!(
            line.split(' ').any(|field| field == expected_field),
            "`E name=smallest_usable` 那一行有 `{expected_field}`：{line}"
        );
    }
}
