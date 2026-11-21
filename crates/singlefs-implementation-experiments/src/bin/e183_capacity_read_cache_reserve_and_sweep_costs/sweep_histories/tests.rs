use super::*;
use crate::helpers_shared_by_the_tests::parsed;

#[test]
fn anchor_model_reproduces_every_registered_script_row() {
    for anchor in &REGISTERED_SWEEP_HISTORY_ANCHORS {
        let history = sweep_history_by_the_anchor_model(
            anchor.slots_per_region,
            Some(anchor.threshold_rows),
            anchor.unmount_kind,
            2 * anchor.threshold_rows + 3 * anchor.slots_per_region + 20,
        );
        assert!(
            anchor.is_reproduced_by(&history),
            "{anchor:?} 模型给的：{:?} 末 txg {}",
            history.sweeps(),
            history.final_txg
        );
    }
}

#[test]
fn anchor_model_without_a_threshold_never_sweeps_and_rows0_counts_the_mounts() {
    let history =
        sweep_history_by_the_anchor_model(8, None, UnmountKind::Normal, SWEEP_CONTROL_MOUNTS);
    assert_eq!(history.mounts.len(), 400);
    assert!(history.sweeps().is_empty());
    for (index, mount) in history.mounts.iter().enumerate() {
        assert_eq!(
            mount.rows0,
            u64::try_from(index).expect("序号") + 1,
            "第 {} 次挂载",
            index + 1
        );
    }
    assert_eq!(
        history.peak_rows0(),
        400,
        "400 次挂载 rows0 到 400，跑前登记 PC-不扫 写的是 401，修订改成这个数"
    );
    assert_eq!(
        history.mounts.last().map(|mount| mount.rows0),
        Some(SWEEP_CONTROL_REVISED_FINAL_ROWS0),
        "装置判的修订值就是锚点模型第 400 次挂载的 rows0"
    );
    assert_eq!(SWEEP_CONTROL_ORIGINALLY_REGISTERED_FINAL_ROWS0, 401);
}

#[test]
fn the_no_sweep_control_judges_its_final_rows0_against_the_revised_value_only() {
    assert!(control_final_rows0_equals_the_revised_registered_value(400));
    assert!(!control_final_rows0_equals_the_revised_registered_value(
        401
    ));
    assert!(!control_final_rows0_equals_the_revised_registered_value(
        399
    ));
}

#[test]
fn the_chain_read_is_one_unit_per_page_of_369_rows_and_never_less_than_one_page() {
    assert_eq!(instance_table_chain_read_bytes(0), 32768);
    assert_eq!(instance_table_chain_read_bytes(1), 32768);
    assert_eq!(instance_table_chain_read_bytes(369), 32768);
    assert_eq!(instance_table_chain_read_bytes(370), 2 * 32768);
    assert_eq!(instance_table_chain_read_bytes(738), 2 * 32768);
    assert_eq!(instance_table_chain_read_bytes(739), 3 * 32768);
    assert_eq!(instance_table_chain_pages_read(739), 3);
}

#[test]
fn the_chain_read_arm_holds_only_when_the_read_difference_equals_the_expected_bytes() {
    let difference = |read_bytes_difference| {
        Some(SweepDifferential {
            write_bytes_difference: 0,
            read_bytes_difference,
        })
    };
    assert!(chain_read_differential_holds(
        difference(32768),
        Some(32768)
    ));
    assert!(!chain_read_differential_holds(difference(0), Some(32768)));
    assert!(!chain_read_differential_holds(
        difference(32769),
        Some(32768)
    ));
    assert!(!chain_read_differential_holds(
        difference(32767),
        Some(32768)
    ));
    assert!(!chain_read_differential_holds(None, Some(32768)));
    assert!(!chain_read_differential_holds(difference(32768), None));
}

#[test]
fn the_chain_read_line_takes_the_expected_bytes_from_the_model_rows_and_not_from_the_reading() {
    let reading = Some(SweepDifferential {
        write_bytes_difference: 0,
        read_bytes_difference: 32768,
    });
    let line = chain_read_differential_line("cell", 1, reading, Some(1));
    assert_eq!(
        line,
        "name=h5_chain_read_differential shard=cell mount=1 read_bytes_difference=32768 model_rows0=1 expected_chain_pages=1 expected_chain_read_bytes=32768 read_bytes_difference_equals_the_chain_read=true"
    );
    let other_rows = chain_read_differential_line("cell", 1, reading, Some(370));
    assert!(
        other_rows.contains(
            "expected_chain_read_bytes=65536 read_bytes_difference_equals_the_chain_read=false"
        ),
        "{other_rows}"
    );
    let no_model = chain_read_differential_line("cell", 1, reading, None);
    assert!(
        no_model.contains(
            "expected_chain_read_bytes=none read_bytes_difference_equals_the_chain_read=false"
        ),
        "{no_model}"
    );
}

#[test]
fn sweep_is_due_only_strictly_above_the_threshold() {
    assert!(!sweep_is_due(369, 369));
    assert!(sweep_is_due(370, 369));
    assert!(!sweep_is_due(0, 0));
}

#[test]
fn the_ring_at_the_judgement_is_the_ring_roots_before_the_sweep_publish() {
    let owners: BTreeMap<u64, u32> = (0..30u64)
        .map(|txg| (txg, u32::try_from(txg / 5).expect("小")))
        .collect();
    let in_the_ring =
        instances_in_the_ring_when_the_sweep_judges(&owners, 25, 12).expect("都读到过");
    assert_eq!(in_the_ring, BTreeSet::from([2, 3, 4]), "txg 13 … 24 的实例");
    let mut missing = owners.clone();
    missing.remove(&13);
    assert_eq!(
        instances_in_the_ring_when_the_sweep_judges(&missing, 25, 12),
        None
    );
    assert_eq!(
        instances_in_the_ring_when_the_sweep_judges(&missing, 26, 12),
        Some(BTreeSet::from([2, 3, 4, 5])),
        "环是 txg 14 … 25，13 不在环里"
    );
    let should_delete = rows_the_sweep_should_delete(&[1, 2, 3, 4, 5], &in_the_ring);
    assert_eq!(should_delete, BTreeSet::from([1, 5]));
}

fn history_mount(
    rows0: u64,
    deleted: Option<Vec<u32>>,
    rows_left: Option<u64>,
    heads_erased: usize,
) -> HistoryMount {
    HistoryMount {
        number: 1,
        instance: 9,
        rows_written: 1,
        rows0,
        rows0_by_the_allocator: rows0,
        rows_before_the_sweep: (1..=u32::try_from(rows0).expect("小")).collect(),
        sweep: SweepAtTheMount {
            text: if deleted.is_some() {
                "rows_deleted".to_string()
            } else {
                "not_swept_at_or_below_the_threshold".to_string()
            },
            rows_deleted: deleted,
            heads_erased,
            handed_to_the_background: false,
        },
        rows_left,
        checkpoint_txg: 40,
        mount_nanoseconds: 1,
        written_and_read_bytes: (10, 20),
    }
}

#[test]
fn each_mount_check_turns_red_on_its_own_fault() {
    let arm = SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows: 3 };
    let ring = BTreeSet::from([3u32, 4]);
    let model = AnchorModelMount {
        rows0: 4,
        sweep: Some((2, 2)),
        checkpoint_txg_when_the_mount_returns: 40,
    };
    let good = history_mount(4, Some(vec![1, 2]), Some(2), 0);
    let checks = checks_of_a_history_mount(&good, arm, Some(&model), Some(&ring));
    assert!(checks.all_hold(), "{checks:?}");
    let wrong_rows = history_mount(4, Some(vec![1]), Some(3), 0);
    let wrong_rows_checks = checks_of_a_history_mount(&wrong_rows, arm, Some(&model), Some(&ring));
    assert_eq!(
        wrong_rows_checks.failed_names(),
        vec!["deleted_rows_match_the_ring", "a34_a35_trajectory"]
    );
    let one_row_more = history_mount(5, Some(vec![1, 2]), Some(2), 0);
    assert!(
        checks_of_a_history_mount(&one_row_more, arm, Some(&model), Some(&ring))
            .failed_names()
            .contains(&"a34_a35_trajectory"),
        "rows0 比模型多一行：轨迹对不上"
    );
    let erased = history_mount(4, Some(vec![1, 2]), Some(2), 1);
    assert_eq!(
        checks_of_a_history_mount(&erased, arm, Some(&model), Some(&ring)).failed_names(),
        vec!["no_head_erased"]
    );
    let swept_at_the_threshold = history_mount(3, Some(vec![1]), Some(2), 0);
    assert!(
        checks_of_a_history_mount(&swept_at_the_threshold, arm, None, Some(&ring))
            .failed_names()
            .contains(&"sweep_matches_the_threshold")
    );
    let not_swept_above = history_mount(4, None, None, 0);
    assert!(
        checks_of_a_history_mount(&not_swept_above, arm, Some(&model), Some(&ring))
            .failed_names()
            .contains(&"sweep_matches_the_threshold")
    );
    let unknown_ring = checks_of_a_history_mount(&good, arm, Some(&model), None);
    assert_eq!(
        unknown_ring.deleted_rows_match_the_ring,
        Some(false),
        "环里有 txg 没读到过：K34 判不了，按不符停"
    );
}

#[test]
fn history_tally_counts_rows_above_366_two_page_mounts_and_the_plus_one_run() {
    let arm = SweepArm::AtTheEndOfTheMountAboveTheThreshold {
        threshold_rows: 366,
    };
    let mut tally = HistoryTally::default();
    for rows0 in 365..=368 {
        tally.note(&history_mount(rows0, None, None, 0), arm);
    }
    assert_eq!(tally.mounts, 4);
    assert_eq!(tally.peak_rows0, 368);
    assert_eq!(tally.mounts_with_rows0_above_366, 2);
    assert_eq!(tally.mounts_with_rows0_above_the_threshold, 2);
    assert_eq!(tally.peak_chain_pages, 2);
    assert_eq!(tally.mounts_with_two_or_more_chain_pages, 2);
    assert!(!tally.rows0_plus_one_each_mount, "第一次挂载 rows0 不是 1");
    let mut counting = HistoryTally::default();
    for rows0 in 1..=5 {
        counting.note(
            &history_mount(rows0, None, None, 0),
            SweepArm::HandedToTheBackground,
        );
    }
    assert!(counting.rows0_plus_one_each_mount);
    counting.note(
        &history_mount(7, None, None, 0),
        SweepArm::HandedToTheBackground,
    );
    assert!(!counting.rows0_plus_one_each_mount, "跳了一行");
}

#[test]
fn the_differential_subtracts_the_background_mount_from_the_threshold_mount() {
    let difference = sweep_differential((5000, 300), (1000, 100));
    assert_eq!(
        difference,
        SweepDifferential {
            write_bytes_difference: 4000,
            read_bytes_difference: 200
        }
    );
    assert!(differential_control_above_holds(Some(difference)));
    assert!(!differential_control_below_holds(Some(difference)));
    let none = sweep_differential((1000, 100), (1000, 100));
    assert!(differential_control_below_holds(Some(none)));
    let read_only = sweep_differential((1000, 300), (1000, 100));
    assert!(
        !differential_control_below_holds(Some(read_only)),
        "写字节之差为 0 而读字节之差不为 0：原臂照字面不过，不放宽"
    );
    assert!(!differential_control_above_holds(Some(none)));
    assert!(!differential_control_below_holds(None), "差分没做成不算过");
}

#[test]
fn sweep_shards_are_twenty_four_cells_the_control_and_the_repeat_of_the_first() {
    let shards = sweep_shards();
    assert_eq!(shards.len(), 26);
    assert_eq!(
        shards[0].0,
        "h5-slots-per-region-4-threshold-64-normal_unmount"
    );
    assert_eq!(shards[23].0, "h5-slots-per-region-16-threshold-738-crash");
    assert_eq!(shards[23].1.mounts_at_most, 2 * 738 + 3 * 16 + 20);
    assert_eq!(shards[24].1.arm, SweepArm::HandedToTheBackground);
    assert_eq!(shards[24].1.mounts_at_most, 400);
    assert_eq!(
        shards[25].0,
        format!("{SWEEP_REPEAT_SHARD_PREFIX}{}", shards[0].0)
    );
}

/// 小盘（256 MiB、环 1 MiB）上的清扫历史走产物同一条路：θ = 4、S = 4，两种卸载方式都要走到第二次清扫、每次挂载与锚点模型对得上
/// （不对就是停机 T5 的那一种：锚点脚本的读法与 `crates/` 的实现不一致，交主 agent 两边查）。
#[test]
fn short_sweep_history_on_a_small_pool_runs_to_its_second_sweep_in_both_unmount_kinds() {
    for unmount_kind in [UnmountKind::Normal, UnmountKind::Crash] {
        let cell = SweepHistoryCell {
            slots_per_region: 4,
            arm: SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows: 4 },
            unmount_kind,
            mounts_at_most: 2 * 4 + 3 * 4 + 20,
        };
        let geometry = PoolGeometry {
            device_bytes: 256 * MEBIBYTE,
            journal_ring_bytes: MEBIBYTE,
        };
        let mut emitter = Emitter::collecting_lines();
        let end = run_sweep_history_on_a_pool("small", cell, geometry, "same", &mut emitter);
        let lines = emitter.into_collected_lines();
        assert_eq!(
            end,
            ShardEnd::Finished,
            "{}：\n{}",
            unmount_kind.name(),
            lines.join("\n")
        );
        let summary = parsed(&lines)
            .into_iter()
            .find(|line| line.name() == Some("h5_cell_summary"))
            .expect("有汇总行");
        assert_eq!(
            summary.number("sweeps_done"),
            Some(2),
            "{}",
            unmount_kind.name()
        );
        assert_eq!(
            summary.field("differential_below_the_threshold_write_bytes"),
            Some("0"),
            "没越过阈值的那一次，两次挂载写字节相同"
        );
        assert_eq!(
            summary.truth("positive_control_differential_at_the_first_sweep"),
            Some(true)
        );
        // 新臂的绝对值：第一次挂载（rows0 = 1）阈值那一次比「交给后台」那一次多读一片实例表（32768 字节），这个数另算（片数 × 一片的字节），
        // 不取自被测的读数；原臂（读字节之差 = 0）在这一格上不成立，照字面判、记一次输。
        let chain_read = parsed(&lines)
            .into_iter()
            .find(|line| line.name() == Some("h5_chain_read_differential"))
            .expect("每个阈值格有一条新臂的行");
        assert_eq!(
            chain_read.number("expected_chain_read_bytes"),
            Some(32768),
            "{}",
            unmount_kind.name()
        );
        assert_eq!(
            chain_read.truth("read_bytes_difference_equals_the_chain_read"),
            Some(true),
            "{}：{}",
            unmount_kind.name(),
            lines.join("\n")
        );
        assert_eq!(
            summary.truth("positive_control_differential_below_the_threshold"),
            Some(false),
            "原臂照字面判：读字节之差不为 0，这一格 V35 打中"
        );
        assert_eq!(
            summary.number("peak_rows0"),
            Some(5),
            "rows0 峰值 = θ + 1（A34）"
        );
    }
}

#[test]
fn short_history_handed_to_the_background_never_sweeps_and_counts_one_row_per_mount() {
    let cell = SweepHistoryCell {
        slots_per_region: 8,
        arm: SweepArm::HandedToTheBackground,
        unmount_kind: UnmountKind::Normal,
        mounts_at_most: 6,
    };
    let geometry = PoolGeometry {
        device_bytes: 256 * MEBIBYTE,
        journal_ring_bytes: MEBIBYTE,
    };
    let mut emitter = Emitter::collecting_lines();
    let end = run_sweep_history_on_a_pool("small", cell, geometry, "same", &mut emitter);
    let lines = emitter.into_collected_lines();
    assert_eq!(end, ShardEnd::Finished, "{}", lines.join("\n"));
    let summary = parsed(&lines)
        .into_iter()
        .find(|line| line.name() == Some("h5_cell_summary"))
        .expect("有汇总行");
    assert_eq!(
        summary.truth("positive_control_no_sweep_publish"),
        Some(true)
    );
    assert_eq!(
        summary.truth("positive_control_rows0_plus_one_each_mount"),
        Some(true)
    );
    assert_eq!(summary.number("final_rows0"), Some(6));
}
