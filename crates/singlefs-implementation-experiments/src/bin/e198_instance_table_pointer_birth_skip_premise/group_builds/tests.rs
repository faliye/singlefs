use super::*;
use crate::helpers_shared_by_the_tests::{
    assert_the_history_is_consistent, key, sweep_specification,
};

#[test]
fn predicted_warm_ups_follow_the_region_to_device_map() {
    assert_eq!(
        predicted_warm_up_publishes(17),
        2,
        "登记第十三节 anchors.py：b = 16 时 w₃ = 2"
    );
    assert_eq!(predicted_warm_up_publishes(18), 1, "b = 17 时 w₃ = 1");
    assert_eq!(predicted_warm_up_publishes(11), 2, "C1：写行 11、暖机 2 次");
    assert_eq!(predicted_warm_up_publishes(8), 2, "C2：写行 8、暖机 2 次");
}

fn short_ring_specification(chunks: u64) -> HistorySpecification {
    HistorySpecification {
        group: Group::ShortRing,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::ShortRing { last: 4, chunks },
    }
}

/// ss（S 4、last 4、c 2）挂载之后那一张：last + 1 = 5 那一版由记录施加出来（根槽从没落盘、记录还读得出），候选集连成一串、没有洞；
/// 它也是账本里「被施加过、根槽没落」的那一版（S5 两边相同）。
#[test]
fn short_ring_first_image_pins_the_applied_version_and_has_no_hole_pair() {
    let result = run_the_history(&short_ring_specification(2));
    assert_the_history_is_consistent(&result, 13);
    let first = &result.images[0];
    let reading = first.reading.as_ref().expect("第一张读得出");
    assert_eq!(
        reading
            .applied_only_by_records
            .iter()
            .map(|version| version.key)
            .collect::<Vec<_>>(),
        vec![key(1, 5)],
        "由记录施加出来的只有 1@5"
    );
    assert_eq!(reading.hole_pairs, Vec::new(), "0 1 2 3 4 5 6 7 连成一串");
    assert_eq!(
        first.records_applied.mismatch_count(),
        0,
        "S5：装置认的与账本推的相同"
    );
}

/// ss（S 4、last 4、c 4）：顺序写把 1 MiB 的 journal 环转一圈之后 1@5 的记录被盖掉，(4, 6) 成了洞对，中间那一版 1@5 存在过
/// （实例 2 的挂载施加了它）；今天的臂与甲都不认作取号跳过（行是 (1, 5)，不是 (1, 4)）。
#[test]
fn short_ring_hole_left_by_an_overwritten_record_is_an_existed_version_in_between() {
    let result = run_the_history(&short_ring_specification(4));
    assert_the_history_is_consistent(&result, 13);
    let found: Vec<(Truth, bool, bool)> = result
        .images
        .iter()
        .filter_map(|image| image.reading.as_ref().ok().map(|reading| (image, reading)))
        .flat_map(|(image, reading)| reading.hole_pairs.iter().zip(image.truths.iter()))
        .filter(|(pair, _)| pair.previous_txg == 4 && pair.version.key == key(2, 6))
        .map(|(pair, truth)| {
            (
                *truth,
                pair.bridged_today,
                pair.bridged_with_the_pointer_birth,
            )
        })
        .collect();
    assert!(
        !found.is_empty(),
        "(4, 6) 这个洞对在 k = 0 … 12 里一次都没出现"
    );
    for observation in found {
        assert_eq!(
            observation,
            (Truth::ExistedVersionsInBetween, false, false),
            "真值是中间有存在过的版本，两臂都不认"
        );
    }
}

#[test]
fn sweep_crash_history_sweeps_once_and_changes_the_pointer_only_there() {
    let result = run_the_history(&sweep_specification(
        SweepVariant::CrashBeforeTheNextInstance,
    ));
    assert_the_history_is_consistent(&result, 13);
    assert!(
        history_swept(&result),
        "实例 3 的挂载末尾发了清扫：{:?}",
        result.sweep
    );
    assert_eq!(result.premise.sweep_pointer_changes, 1, "清扫换指针一次");
    assert!(
        result.premise.first_version_violations.is_empty()
            && result.premise.adjacent_version_violations.is_empty()
            && result.premise.birth_after_the_txg_violations.is_empty(),
        "{:?}",
        result.premise
    );
}

#[test]
fn sweep_read_history_makes_one_image_per_version_of_the_sweep_instance_from_the_sweep_on() {
    let result = run_the_history(&sweep_specification(
        SweepVariant::ReadFaultOnTheSweepInstance,
    ));
    let sweep = result.sweep.clone().expect("sw-read 有清扫读数");
    let sweep_txg = sweep.sweep_version.expect("实例 3 的挂载末尾发了清扫").txg;
    let expected = usize::try_from(sweep.last_txg_before_the_sweep_instance + 12 - sweep_txg)
        .expect("张数装得进 usize");
    assert_the_history_is_consistent(&result, expected);
    let with_pointer_birth_e1: u64 = result
        .images
        .iter()
        .map(|image| image.counts.with_pointer_birth_e1)
        .sum();
    assert!(
        with_pointer_birth_e1 >= 1,
        "PC-甲：R 是清扫那一版、它前面同实例的版本读坏，甲按字面认错（登记第四节第 3 条）"
    );
}

#[test]
fn rollback_history_makes_one_image_per_first_readable_root() {
    let specification = HistorySpecification {
        group: Group::RollbackSlotBad,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::RollbackSlotBad {
            target: 5,
            actions: vec![
                RollbackAction::LongerSequentialWrite,
                RollbackAction::EmptyPublish,
            ],
        },
    };
    let result = run_the_history(&specification);
    assert_the_history_is_consistent(&result, 3);
    for image in &result.images {
        assert!(
            image.checker_verdicts.is_some(),
            "rs1 每张都跑今天的池级 checker"
        );
    }
}

#[test]
fn sweep_none_history_makes_one_image_per_step_after_the_next_mount() {
    let result = run_the_history(&sweep_specification(SweepVariant::NoCrash));
    assert_the_history_is_consistent(&result, 13);
    assert!(
        history_swept(&result),
        "实例 3 的挂载末尾发了清扫：{:?}",
        result.sweep
    );
}

/// G1：阈值 365 要攒到 366 行，实例 367 的挂载才扫（推的）；这一条要挂三百多次，慢，照样进单测（产物的 G1 那一段就是它）。
#[test]
fn sweep_at_the_product_default_threshold_waits_until_the_rows_exceed_it() {
    let result = run_the_history(&sweep_threshold_product_default_specification());
    assert_the_history_is_consistent(&result, 13);
    let sweep = result.sweep.clone().expect("G1 有清扫读数");
    assert!(
        sweep.sweep_version.is_some(),
        "攒满之后那次挂载发了清扫：{sweep:?}"
    );
    assert!(
        u64::from(sweep.sweep_instance) > SWEEP_THRESHOLD_PRODUCT_DEFAULT,
        "行数超过阈值才扫：清扫那个实例的代号 {} 要大于 365",
        sweep.sweep_instance
    );
}
