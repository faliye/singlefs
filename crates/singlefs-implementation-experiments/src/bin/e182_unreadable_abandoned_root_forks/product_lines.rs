use super::*;

// ============================================================================
// 十五、产物：每条历史一行摘要、一行全部计数、按登记第六节的那几种比例逐个报（`question` 写分子那个计数的名字；分子、分母、万分比），再按族与几何合并报一遍。
//    比例只报数，判定语言（Z1–Z8）留给报告；这里只在判决行里放「没过」类的计数。
// ============================================================================

fn emit_ratio(question: &str, scope: &str, label: &str, numerator: u64, denominator: u64) {
    let per_ten_thousand = if denominator == 0 {
        "none".to_string()
    } else {
        format!(
            "{}",
            u128::from(numerator) * 10_000 / u128::from(denominator)
        )
    };
    emit_result(&format!(
        "name=ratio question={question} scope={scope} label={label} numerator={numerator} denominator={denominator} per_ten_thousand={per_ten_thousand}"
    ));
}

pub(super) fn sum_over_kept_disks(counters: &Counters, suffix_of: &dyn Fn(u32) -> String) -> u64 {
    [0u32, 1u32]
        .into_iter()
        .map(|device| counters.count_named(&suffix_of(device)))
        .sum()
}

/// 一份计数（一条历史，或一族一几何合起来的）上登记第六节要的那几种比例。每一行的 `question` 就是分子那个计数的名字
/// （按留哪块盘分开的计数去掉 `kept<盘号>` 那一段、按 `label` 末尾的 `kept_<…>` 加起来），`label` 写这份计数是谁。
pub(super) fn emit_questions(scope: &str, label: &str, counters: &Counters) {
    for (kept_label, devices) in [
        ("0", vec![0u32]),
        ("1", vec![1u32]),
        ("both_disks_summed", vec![0u32, 1u32]),
    ] {
        let sum = |name_of: &dyn Fn(u32) -> String| -> u64 {
            devices
                .iter()
                .map(|device| counters.count_named(&name_of(*device)))
                .sum()
        };
        let states = sum(&|device| format!("today.kept{device}.states"));
        let question_label = format!("{label}.kept_{kept_label}");
        emit_ratio(
            "today.pick.hidden_root_covered",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.pick.hidden_root_covered")),
            states,
        );
        emit_ratio(
            "today.outcome.failed_without_root",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.outcome.failed_without_root")),
            states,
        );
        emit_ratio(
            "today.outcome.failed_with_root",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.outcome.failed_with_root")),
            states,
        );
        emit_ratio(
            "today.pick.hidden_root",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.pick.hidden_root_covered"))
                + sum(&|device| format!("today.kept{device}.pick.hidden_root_intact")),
            states,
        );
        let legal_today = sum(&|device| format!("today.kept{device}.pick.legal"));
        let covered_hidden_today =
            sum(&|device| format!("today.kept{device}.pick.hidden_root_covered"));
        for arm in [Arm::ConfigurationInstance, Arm::NewerInstanceRecord] {
            let arm_name = arm.name();
            let arm_label = format!("{question_label}.{arm_name}");
            emit_ratio(
                &format!("{arm_name}.avoided_a_legal_root"),
                scope,
                &arm_label,
                sum(&|device| format!("{arm_name}.kept{device}.avoided_a_legal_root")),
                legal_today,
            );
            emit_ratio(
                &format!("{arm_name}.avoided_a_legal_root.then_none"),
                scope,
                &arm_label,
                sum(&|device| format!("{arm_name}.kept{device}.avoided_a_legal_root.then_none")),
                legal_today,
            );
            emit_ratio(
                &format!("{arm_name}.on_a_covered_hidden_root.then_legal"),
                scope,
                &arm_label,
                sum(&|device| {
                    format!("{arm_name}.kept{device}.on_a_covered_hidden_root.then_legal")
                }),
                covered_hidden_today,
            );
            emit_ratio(
                &format!("{arm_name}.on_a_covered_hidden_root.then_hidden_root_covered"),
                scope,
                &arm_label,
                sum(&|device| {
                    format!(
                        "{arm_name}.kept{device}.on_a_covered_hidden_root.then_hidden_root_covered"
                    )
                }),
                covered_hidden_today,
            );
        }
    }
    let after_the_row_publish = counters.count_named("after_the_row_publish.states");
    for class in [
        "class_confirmed_none_covered",
        "class_some_unit_covered",
        "class_unconfirmed_none_covered",
    ] {
        let counter_name = format!("after_the_row_publish.{class}");
        emit_ratio(
            &counter_name,
            scope,
            label,
            counters.count_named(&counter_name),
            after_the_row_publish,
        );
    }
    for suffix in [
        "",
        ".class_confirmed_none_covered",
        ".class_some_unit_covered",
        ".class_unconfirmed_none_covered",
    ] {
        let counter_name = format!(
            "after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot{suffix}"
        );
        emit_ratio(
            &counter_name,
            scope,
            label,
            counters.count_named(&counter_name),
            after_the_row_publish,
        );
    }
    emit_ratio(
        "after_the_row_publish.hidden_root_slot_overwritten",
        scope,
        label,
        counters.count_named("after_the_row_publish.hidden_root_slot_overwritten"),
        after_the_row_publish,
    );
}

pub(super) fn emit_history_summary(history: &RecordedHistory) {
    let expansions: BTreeMap<&'static str, usize> =
        (0..=history.segments.len()).fold(BTreeMap::new(), |mut map, shard_index| {
            *map.entry(expansion_of_shard(history, shard_index).name())
                .or_insert(0) += 1;
            map
        });
    let total_states: u64 = (0..=history.segments.len())
        .map(|shard_index| {
            history.segments.get(shard_index).map_or(1, |segment| {
                state_count_of_segment(&history.writes, segment).unwrap_or(0)
            })
        })
        .sum();
    let ranges_by_label: BTreeMap<&str, usize> =
        history
            .fault_ranges
            .iter()
            .fold(BTreeMap::new(), |mut map, range| {
                *map.entry(range.label).or_insert(0) += 1;
                map
            });
    emit_result(&format!(
        "name=history history={} hidden_root={} fault_ranges={} segments={} writes={} states_closed_form={total_states} expansions={} mount_refused={} mount_stopped_after={} instance={} first_publish_root={} warm_up_publishes={} shadow_ledger_branch={} isolated_slots_per_device={} abandoned_roots_unreadable={} core_ring_slots_holding_an_abandoned_root={} reads_intercepted={} ranges_written_over={} consistency_problems={}",
        history_name(&history.specification),
        history.hidden_root.map_or("none".to_string(), |(instance, txg)| format!("{instance}:{txg}")),
        token_of(&format!("{ranges_by_label:?}")),
        history.segments.len(),
        history.writes.len(),
        token_of(&format!("{expansions:?}")),
        token_of(&format!("{:?}", history.mount.refused.as_deref().map(|text| text.chars().take(120).collect::<String>()))),
        token_of(&format!("{:?}", history.mount.stopped_after_the_mount)),
        history.mount.instance.map_or("none".to_string(), |instance| instance.to_string()),
        history.mount.row_publish_root.map_or("none".to_string(), |(instance, txg)| format!("{instance}:{txg}")),
        history.mount.warm_up_publishes,
        token_of(&history.mount.shadow_ledger_branch),
        token_of(&format!("{:?}", history.mount.isolated_slots_per_device)),
        history.mount.abandoned_roots_unreadable,
        token_of(&format!("{:?}", history.mount.ring_slots_recorded_as_holding_an_abandoned_root)),
        token_of(&format!("{:?}", history.mount.intercepted_reads_by_label)),
        token_of(&format!("{:?}", history.mount.ranges_written_over_by_label)),
        token_of(&format!("{:?}", history.consistency_problems)),
    ));
}
