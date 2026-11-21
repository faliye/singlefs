use super::*;
use crate::helpers_shared_by_the_tests::small_history;

#[test]
fn the_configuration_instance_arm_avoids_only_a_strictly_newer_instance() {
    assert!(
        !avoided_by_configuration_instance(1, 1),
        "实例代号相等不避开"
    );
    assert!(
        avoided_by_configuration_instance(2, 1),
        "配置里的实例代号更新就避开"
    );
    assert!(
        !avoided_by_configuration_instance(1, 2),
        "配置里的实例代号更旧不避开"
    );
}

#[test]
fn counters_round_trip_through_their_text_and_merge_by_name() {
    let mut counters = Counters::default();
    counters.add("a.b", 3);
    counters.add("c", 5);
    let mut other = Counters::default();
    other.add("a.b", 4);
    other.add("d", 1);
    let mut merged = counters.clone();
    merged.merge(&other);
    assert_eq!(merged.count_named("a.b"), 7);
    assert_eq!(merged.count_named("c"), 5);
    assert_eq!(merged.count_named("d"), 1);
    assert_eq!(merged.count_named("missing"), 0);
    assert_eq!(
        Counters::from_text(&merged.to_text()).expect("文本读得回"),
        merged
    );
    let mut merged_the_other_way = other.clone();
    merged_the_other_way.merge(&counters);
    assert_eq!(merged_the_other_way, merged, "合并与次序无关");
}

/// 甲一只读剩下那块盘：新实例的记录只落在掉了的那块盘上时，不避开。拿两块盘一起读的判法对照，确有这样的状态会被两块盘一起读骗到。
#[test]
fn the_newer_instance_record_arm_reads_only_the_kept_disk() {
    let history = small_history();
    let hidden = history.hidden_root.expect("带藏根的历史");
    let (mut examined, mut would_be_fooled_by_the_other_disk) = (0u64, 0u64);
    'segments: for shard_index in 0..=history.segments.len() {
        let mut candidates: Vec<(Vec<bool>, u32)> = Vec::new();
        for_each_state_of_segment(
            &history.writes,
            &history.segments,
            shard_index,
            &mut |persisted| {
                let record_on = |device: u32| {
                    history
                        .writes
                        .iter()
                        .zip(persisted)
                        .any(|(write, is_persisted)| {
                            *is_persisted
                                && write.kind == StepKind::JournalRecord
                                && write.device.0 == device
                        })
                };
                for kept in [0u32, 1u32] {
                    if record_on(1 - kept) && !record_on(kept) && candidates.len() < 300 {
                        candidates.push((persisted.to_vec(), kept));
                    }
                }
                Ok(())
            },
        )
        .expect("枚举得完");
        for (persisted, kept) in candidates {
            let image = CrashImage {
                base: &history.base,
                writes: &history.writes,
                persisted,
            };
            let device = DeviceIdentity(kept);
            let kept_view = kept_disk(&image, device);
            let report = recover(&kept_view, JournalPolicy::Consult);
            if outcome_root(&report.outcome) != Some(hidden) {
                continue;
            }
            let result = arm_result_on_the_kept_disk(
                &image,
                device,
                &history.pool_geometry,
                Arm::NewerInstanceRecord,
                &report,
            );
            assert_eq!(
                result.avoided_roots, 0,
                "更新实例的记录只在另一块盘上，剩下这块盘上看不见，甲一不该避开"
            );
            let root =
                root_record_of(&kept_view, hidden).expect("被藏根在剩下那块盘的根环里读得出");
            if avoided_by_newer_instance_record(&image, &root) {
                would_be_fooled_by_the_other_disk += 1;
            }
            examined += 1;
            if examined >= 3000 {
                break 'segments;
            }
        }
    }
    assert!(
        examined > 0,
        "找得到被藏根被择中、新实例的记录只在另一块盘上的状态"
    );
    assert!(
        would_be_fooled_by_the_other_disk > 0,
        "这些状态里至少有一个会被两块盘一起读骗到，这条用例才分得出读几块盘"
    );
}
