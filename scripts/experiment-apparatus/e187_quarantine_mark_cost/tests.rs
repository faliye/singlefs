use super::*;

/// 第七节「独立算出」的容量、根层级、叶界：常量断言写加法。
#[test]
fn capacities_root_levels_and_leaf_ranges_match_the_independently_computed_pins() {
    assert_eq!(ALLOCATION_RECORD_TREE_LEAF_ENTRIES, 800 + 12, "(16384 − 135) ÷ 20");
    assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, 160 + 9, "(16384 − 135) ÷ 96");
    assert_eq!(MAPPING_LEAF_ENTRIES, 290 + 4, "(16384 − 169) ÷ 55");
    assert_eq!(MAPPING_INTERNAL_ENTRIES, 140 + 3, "(16384 − 169) ÷ 113");
    assert_eq!(QUARANTINE_TREE_LEAF_ENTRIES, 800 + 12, "(16384 − 135) ÷ 20");
    assert_eq!(TREE_TABLE_UNIT_ENTRIES, 80 + 1, "(16384 − 131) ÷ 200");
    assert_eq!(EXTENT_LEAF_ENTRIES, 140 + 4, "(16384 − 163) ÷ 112");
    assert_eq!(EXTENT_INTERNAL_ENTRIES, 140 + 7, "(16384 − 163) ÷ 110");
    assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 60 + 4, "(4096 − 487) ÷ 56");
    assert_eq!(allocation_record_tree_root_level(PoolGeometry::DefaultFourGibibyteDevices.slots_per_device()), 1 + 1);
    assert_eq!(allocation_record_tree_root_level(PoolGeometry::WideOneTebibyteDevices.slots_per_device()), 2 + 1);
    assert_eq!(62 * ALLOCATION_RECORD_TREE_LEAF_SLOTS, 50000 + 344);
    assert_eq!(70 * ALLOCATION_RECORD_TREE_LEAF_SLOTS, 56000 + 840);
    assert_eq!(75 * ALLOCATION_RECORD_TREE_LEAF_SLOTS, 60000 + 900);
    assert_eq!(leaf_of_slot(140000), 170 + 2);
    assert_eq!(span_in_slots_at_level(1), 137000 + 228);
}

#[test]
fn every_format_constant_of_the_model_is_written_verbatim_in_the_crates_sources() {
    for (item, origin, line, found) in crates_source_checks() {
        assert!(found, "{item}：{origin} 里没有逐字的「{line}」");
    }
}

#[test]
fn the_borrowed_high_bit_makes_every_record_overrun_its_leaf_for_the_old_reader() {
    for record_span in FIRST_VERSION_SPANS {
        let arithmetic = old_reader_arithmetic(QUARANTINE_MARK_BIT, record_span);
        assert_eq!(arithmetic.span_read, 16384 + u64::from(record_span));
        assert_eq!(arithmetic.accepted, 0, "跨度 16385 / 16386 落不进 812 槽的叶");
    }
}

#[test]
fn the_borrowed_low_bit_keeps_most_records_in_the_leaf_with_a_wrong_span() {
    let single = old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 1);
    let double = old_reader_arithmetic(LOW_SPAN_BIT_PROBE, 2);
    assert_eq!((single.span_read, single.legal_starts, single.accepted), (4 + 1, 812, 800 + 8));
    assert_eq!((double.span_read, double.legal_starts, double.accepted), (4 + 2, 406, 400 + 4));
}

#[test]
fn the_old_reader_positive_control_span_is_judged_outside_the_leaf() {
    assert_eq!(old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD), 32767);
    assert!(!old_reader_accepts(OLD_READER_POSITIVE_CONTROL_SLOT % ALLOCATION_RECORD_TREE_LEAF_SLOTS, old_reader_span(OLD_READER_POSITIVE_CONTROL_SPAN_FIELD)));
    assert!(old_reader_accepts(0, 2), "合法的两槽记录落得进叶");
}

/// 一段微历史：甲、乙各减 `no_mark` 等于第七节手推的绝对值。
fn assert_the_micro_history_pins(micro: MicroHistory) {
    let measurements = measure_micro_history(micro);
    let (span_bit_writes, span_bit_bytes, quarantine_tree_writes, quarantine_tree_bytes) = micro.pinned_extra_writes();
    let span_bit = micro_extra(&measurements, Arm::SpanBit);
    let quarantine_tree = micro_extra(&measurements, Arm::QuarantineTree);
    assert_eq!(span_bit.device_writes, signed(span_bit_writes), "{}：甲多写的设备写", micro.label());
    assert_eq!(span_bit.bytes_on_both_devices, signed(span_bit_bytes), "{}：甲多写的字节", micro.label());
    assert_eq!(quarantine_tree.device_writes, signed(quarantine_tree_writes), "{}：乙多写的设备写", micro.label());
    assert_eq!(quarantine_tree.bytes_on_both_devices, signed(quarantine_tree_bytes), "{}：乙多写的字节", micro.label());
}

#[test]
fn micro_clean_leaf_costs_the_span_bit_two_leaves_and_the_quarantine_tree_one_new_node() {
    assert_the_micro_history_pins(MicroHistory::CleanLeaf);
}

#[test]
fn micro_clean_ancestor_costs_the_span_bit_a_leaf_and_a_level_one_node_on_each_device() {
    assert_the_micro_history_pins(MicroHistory::CleanAncestor);
}

#[test]
fn micro_dirty_leaf_costs_the_span_bit_nothing() {
    assert_the_micro_history_pins(MicroHistory::DirtyLeaf);
}

#[test]
fn micro_journal_crossing_costs_the_span_bit_one_more_journal_record() {
    assert_the_micro_history_pins(MicroHistory::JournalCrossing);
}

#[test]
fn micro_second_quarantine_with_a_cold_tree_node_costs_the_quarantine_tree_the_cold_leaf() {
    assert_the_micro_history_pins(MicroHistory::SecondQuarantineTreeCold);
}

#[test]
fn micro_revival_clean_costs_the_span_bit_the_leaf_and_clears_the_mark() {
    assert_the_micro_history_pins(MicroHistory::RevivalClean);
    let measurements = measure_micro_history(MicroHistory::RevivalClean);
    for measurement in &measurements {
        assert_eq!(measurement.mark_agreement_red_after_revival, Some(false), "{}：回退之后记号清掉了", measurement.arm.label());
    }
}

#[test]
fn micro_revival_with_a_cold_tree_node_costs_both_placements_a_leaf() {
    assert_the_micro_history_pins(MicroHistory::RevivalTreeCold);
}

#[test]
fn micro_release_of_a_quarantined_record_costs_neither_placement_anything() {
    assert_the_micro_history_pins(MicroHistory::ReleaseQuarantined);
}

#[test]
fn plain_release_rewrites_the_victim_leaf_on_both_devices_where_no_mark_does_not() {
    let measurements = measure_micro_history(MicroHistory::CleanLeaf);
    assert_eq!(micro_extra(&measurements, Arm::PlainRelease), WriteDifference { device_writes: 4, bytes_on_both_devices: 4 * 16384 });
    let no_mark = measurements.iter().find(|measurement| measurement.arm == Arm::NoMark).expect("跑了 no_mark");
    let plain = measurements.iter().find(|measurement| measurement.arm == Arm::PlainRelease).expect("跑了 plain_release");
    assert!(!no_mark.victim_leaf_rewritten && plain.victim_leaf_rewritten, "只有照常释放的那一遍改了叶 75");
}

#[test]
fn every_checker_reddens_a_leak_and_passes_a_normal_record() {
    for arm in FORK_ARMS {
        for checker in checkers_of(arm) {
            for version in JUDGED_VERSIONS {
                assert!(judge_record(*checker, &constructed_view(arm, RecordClass::Leak, MarkState::WrittenCorrectly, version)).is_red(), "{} {} 泄漏判红", arm.label(), checker.label());
                assert!(!judge_record(*checker, &constructed_view(arm, RecordClass::Normal, MarkState::WrittenCorrectly, version)).is_red(), "{} {} 正常判绿", arm.label(), checker.label());
            }
        }
    }
}

#[test]
fn today_checker_reddens_an_intact_quarantine_and_exempts_a_garbled_one() {
    let intact = judge_record(CheckerUnderTest::Today, &constructed_view(Arm::NoMark, RecordClass::QuarantineIntact, MarkState::WrittenCorrectly, JudgedVersion::TheQuarantineVersion));
    assert_eq!(intact.label(), "I-3.11+I-3.15", "隔离那一版：老根还引用它，I-3.1 不红");
    let after_window = judge_record(CheckerUnderTest::Today, &constructed_view(Arm::NoMark, RecordClass::QuarantineIntact, MarkState::WrittenCorrectly, JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow));
    assert_eq!(after_window.label(), "I-3.1+I-3.11+I-3.15");
    let garbled = judge_record(CheckerUnderTest::Today, &constructed_view(Arm::NoMark, RecordClass::QuarantineGarbled, MarkState::WrittenCorrectly, JudgedVersion::TheQuarantineVersion));
    assert_eq!(garbled.label(), "green");
}

#[test]
fn complete_mark_exempts_an_intact_quarantine_under_both_variants() {
    for arm in MARKING_ARMS {
        for checker in checkers_of(arm) {
            let verdict = judge_record(*checker, &constructed_view(arm, RecordClass::QuarantineIntact, MarkState::WrittenCorrectly, JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow));
            assert_eq!(verdict.label(), "green", "{} {}", arm.label(), checker.label());
        }
    }
}

#[test]
fn garbled_quarantine_with_its_mark_cleared_is_red_for_mark_only_and_green_for_mark_or_today() {
    let view = constructed_view(Arm::SpanBit, RecordClass::QuarantineGarbled, MarkState::WrongOnBothDevices, JudgedVersion::TheQuarantineVersion);
    assert_eq!(judge_record(CheckerUnderTest::MarkOnly, &view).label(), "I-3.11+I-3.15");
    assert_eq!(judge_record(CheckerUnderTest::MarkOrToday, &view).label(), "green");
}

#[test]
fn mark_on_a_referenced_record_reddens_mark_agreement() {
    let view = constructed_view(Arm::QuarantineTree, RecordClass::Normal, MarkState::WrongOnBothDevices, JudgedVersion::TheQuarantineVersion);
    assert_eq!(judge_record(CheckerUnderTest::MarkOnly, &view).label(), "mark_agrees_with_references");
    let one_device = constructed_view(Arm::SpanBit, RecordClass::Leak, MarkState::WrongOnOneDevice, JudgedVersion::TheQuarantineVersion);
    assert!(judge_record(CheckerUnderTest::MarkOrToday, &one_device).mark_agreement_red, "只一盘带记号");
}

#[test]
fn the_allocated_statistic_turns_red_exactly_one_window_after_the_quarantine() {
    let event = QuarantineEventRecord { txg: 100, copies: CopyCondition::Intact, marks_during_quarantine: [false, false], revival: None };
    let trajectory = trajectory_of_an_event(CheckerUnderTest::Today, &event, 150, 8);
    assert_eq!(trajectory.first_red_offset_of_the_allocated_statistic, Some(8), "候选窗口 8：事件之后第 8 版最后一个引用它的根离开窗口");
    assert_eq!((trajectory.peak, trajectory.final_value, trajectory.positive_versions), (3, 3, 50 + 1));
    let marked = QuarantineEventRecord { txg: event.txg, copies: event.copies, marks_during_quarantine: [true, true], revival: None };
    assert_eq!(trajectory_of_an_event(CheckerUnderTest::MarkOnly, &marked, 150, 8).peak, 0);
}

#[test]
fn revival_ends_the_false_red_once_the_quarantine_versions_leave_the_window() {
    let event = QuarantineEventRecord {
        txg: 100,
        copies: CopyCondition::Intact,
        marks_during_quarantine: [false, false],
        revival: Some(RevivalRecord { txg: 104, marks_after_revival: [false, false] }),
    };
    let trajectory = trajectory_of_an_event(CheckerUnderTest::Today, &event, 150, 8);
    assert_eq!(trajectory.final_value, 0);
    assert_eq!(trajectory.positive_versions, 4 + 8, "隔离 4 版，复活之后 I-3.15 还红到第 103 版离开窗口");
}

#[test]
fn the_schedule_places_ten_events_per_density_step_and_ten_rollbacks() {
    for density in [1, 3, 10] {
        let cell = HistoryCell { density, rollback_rate: 1, file_size: 200, candidate_window: 8, geometry: PoolGeometry::DefaultFourGibibyteDevices, seed: 1 };
        let schedule = schedule_of(&cell);
        assert_eq!(u64_of(schedule.events.len()), density * 10);
        assert_eq!(schedule.rollbacks.len(), 10);
        let intact = schedule.events.values().filter(|event| event.copies == CopyCondition::Intact).count();
        assert_eq!(intact * 2, schedule.events.len(), "副本状态各半");
    }
}

/// 取样点覆盖第二段会跑到的全部 320 格（`.claude/rules/mutation-sampling.md` 第四类）：每一格都排满登记第五节的事件数与回退数，
/// 每次回退排在它那次隔离之后 1 到 ⌊K ÷ 2⌋ 次发布里，几次回退的区间互不相交（作废条款 `scheduled_events_realized` 的排法那一半）。
#[test]
fn every_segment_two_cell_schedules_the_registered_events_and_rollbacks_in_disjoint_intervals() {
    for cell in all_history_cells() {
        let schedule = schedule_of(&cell);
        assert_eq!(u64_of(schedule.events.len()), cell.density * QUARANTINE_EVENTS_PER_DENSITY_STEP, "{}", cell.label());
        assert_eq!(u64_of(schedule.rollbacks.len()), cell.rollback_rate * ROLLBACKS_AT_RATE_ONE, "{}", cell.label());
        let mut end_of_the_previous_interval = 0u64;
        for (rollback_after, rollback) in &schedule.rollbacks {
            let start_of_the_interval = match rollback {
                ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish) => {
                    assert!(schedule.events.contains_key(event_publish), "{}：回退对着一次排了的隔离", cell.label());
                    assert!(
                        *rollback_after > *event_publish && *rollback_after <= event_publish + cell.candidate_window / 2,
                        "{}：回退排在隔离之后 1 到 ⌊K ÷ 2⌋ 次发布里",
                        cell.label()
                    );
                    *event_publish
                }
                ScheduledRollback::ToTheOldestVersionInTheWindow => {
                    assert_eq!(cell.density, 0, "{}：取窗口里最旧那一版的只在密度 0 上", cell.label());
                    rollback_after - cell.candidate_window
                }
            };
            assert!(start_of_the_interval > end_of_the_previous_interval, "{}：回退的区间互不相交", cell.label());
            end_of_the_previous_interval = *rollback_after;
        }
    }
}

#[test]
fn journal_records_split_after_sixty_four_named_roles() {
    assert_eq!(journal_records_of_the_publish(4, 59, 0), (3 + 1, 60));
    assert_eq!(journal_records_of_the_publish(4, 63, 0), (3 + 1, 64));
    assert_eq!(journal_records_of_the_publish(4, 64, 0), (3 + 2, 65));
    assert_eq!(journal_records_of_the_publish(0, 0, 0), (1, 0), "一个角色都不点名也写一条");
}

#[test]
fn share_tiers_follow_the_registered_bounds_and_separate_two_points_when_the_bound_moves_between_them() {
    assert_eq!(share_tier(999, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 1);
    assert_eq!(share_tier(1000, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 2);
    assert_eq!(share_tier(10_000, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 3);
    assert_eq!(share_tier(-5, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), 1);
    let moved = ShareTierBounds { tier_two_from: 300_000, tier_three_from: u64::MAX / 2 };
    assert_ne!(share_tier(200, 1_000_000, moved), share_tier(400, 1_000_000, moved));
    assert_eq!(share_tier(200, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION), share_tier(400, 1_000_000, SHARE_TIER_BOUNDS_PARTS_PER_BILLION));
}

#[test]
fn the_span_bit_changes_one_byte_per_record_and_the_quarantine_tree_two_entries() {
    assert_eq!(logical_bytes_changed_by_the_span_bit(), 1 + 1);
    assert_eq!(2 * QUARANTINE_TREE_ENTRY_BYTES + TREE_TABLE_ENTRY_BYTES, 40 + 200);
}

#[test]
fn the_new_pool_file_creation_opens_the_second_segment_and_every_arm_drops_the_victim_key() {
    assert_eq!(new_pool_file_creation_open_segment(), (50000 + 240, 50000 + 304, 1 + 1));
    assert!(mapping_entry_removed_on_quarantine_in_every_arm());
}

#[test]
fn code_two_leaf_splits_in_the_middle_and_a_root_with_one_child_collapses() {
    let capacity = CodeTwoCapacity { leaf_entries: 4, internal_entries: 4 };
    let mut fresh = FreshUnits::starting_at(0);
    let mut root: Option<CodeTwoNode<u64>> = None;
    for key in 1..=5u64 {
        insert_at_the_root(&mut root, key, capacity, &mut fresh);
    }
    assert_eq!(tree_height(&root), 2, "5 条装不进 4 条的叶：切成 3 + 2，长出根");
    let left_leaf_unit = match &*root.as_ref().expect("有根").body {
        CodeTwoBody::Internal { children, .. } => {
            assert_eq!(children.iter().map(|child| child.node.entry_count()).collect::<Vec<usize>>(), vec![3, 2]);
            children[0].node.unit
        }
        CodeTwoBody::Leaf(_) => panic!("根该是内部节点"),
    };
    let mut next_publish = FreshUnits::starting_at(fresh.next_unit);
    for key in [4u64, 5] {
        delete_at_the_root(&mut root, &key, &mut next_publish);
    }
    assert_eq!(tree_height(&root), 1, "右叶删空摘掉、根只剩一个孩子：孩子当根");
    assert_eq!(root.as_ref().expect("有根").unit, left_leaf_unit, "当根的孩子这次没改到，不重写");
}

fn small_cell(density: u64, rollback_rate: u64, seed: u64) -> HistoryCell {
    HistoryCell { density, rollback_rate, file_size: 200, candidate_window: 8, geometry: PoolGeometry::DefaultFourGibibyteDevices, seed }
}

#[test]
fn three_arms_write_the_same_on_a_history_without_events() {
    let summary = evaluate_cell(&small_cell(0, 0, 1));
    assert_eq!(summary.arms_agree_without_events, Some(true));
    assert!(summary.schedule_realized);
}

/// 回归钉，不算独立钉值（登记第七节末段、第十二节「回归钉」）：值取第二段产物
/// `research/results/e187-quarantine-mark-cost-segment-two-2026-10-21.out` 里 `no_mark` 在这一格那一行 `name=history` 的 `device_writes=`。
#[test]
fn the_no_mark_history_without_events_keeps_the_device_writes_of_the_first_segment_two_product() {
    let cell = small_cell(0, 0, 1);
    let outcome = run_history(&cell, &schedule_of(&cell), Arm::NoMark);
    assert_eq!(outcome.totals.without_root_and_system_configuration().device_writes, 453394);
}

#[test]
fn history_with_rollbacks_revives_a_quarantined_unit_at_every_rollback() {
    let summary = evaluate_cell(&small_cell(3, 1, 2));
    assert!(summary.schedule_realized, "30 次隔离、10 次回退，每次回退都让一个隔离单元重新被引用");
}

/// 两次隔离挨得近（相隔 21 次与 2 次发布）、K = 64 的格：第一遍抽到的延迟与另一次回退的区间相交，第二遍补上，
/// 10 次回退都排上、每次都让一个隔离单元重新被引用（全量里出过的红，钉回这一条）。
#[test]
fn close_quarantine_events_under_the_wide_window_still_get_ten_reviving_rollbacks() {
    for seed in [1, 2] {
        let cell = HistoryCell { density: 1, rollback_rate: 1, file_size: 200, candidate_window: 64, geometry: PoolGeometry::DefaultFourGibibyteDevices, seed };
        let summary = evaluate_cell(&cell);
        assert!(summary.schedule_realized, "{}", cell.label());
    }
}

/// 多线程与单线程对拍：产物行逐字相同（`.claude/rules/test-script-discipline.md`「编译与算法都要快」）。
#[test]
fn segment_two_lines_do_not_depend_on_the_thread_count() {
    let cells = [small_cell(1, 1, 1), small_cell(1, 0, 2)];
    let single: Vec<Vec<String>> = evaluate_cells(&cells, 1).into_iter().map(|summary| summary.lines).collect();
    let parallel: Vec<Vec<String>> = evaluate_cells(&cells, 2).into_iter().map(|summary| summary.lines).collect();
    assert_eq!(single, parallel);
}
