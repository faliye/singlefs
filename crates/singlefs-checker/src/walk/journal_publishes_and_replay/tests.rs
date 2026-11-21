use super::*;

/// C469 ②：环里实例 1 只写过一条非 0 记录（计数器 1、事务号 1）。① ② 一对可比的号都凑不出来，
/// 「从 1 起」判过且成立：整条报不适用，理由说「从 1 起判过且成立」。环里一条非 0 记录都没有时理由照旧。
#[test]
fn single_non_empty_record_numbered_one_reports_not_applicable_saying_the_start_from_one_held() {
    let verdict_of_a_ring_holding = |transactions: &[(u64, u64)]| {
        let mut records = JournalRecordsOfADevice::new();
        for (counter, transaction) in transactions {
            let mut record = scanned_record_of(1, 0, 0);
            record.transaction = *transaction;
            records.insert(
                JournalRecordKeyOnADevice {
                    counter: *counter,
                    instance: 1,
                },
                record,
            );
        }
        let records_by_device = [(0u32, records)];
        let mut judgements = Judgements::default();
        let comparable_pairs =
            judge_transaction_numbers_per_instance(&records_by_device, &mut judgements);
        let from_one = judge_transaction_numbers_start_from_one_per_instance(
            &records_by_device,
            8,
            &mut judgements,
        );
        note_transaction_numbers_without_a_comparable_pair(
            comparable_pairs,
            from_one,
            &mut judgements,
        );
        judgements
            .into_report()
            .into_iter()
            .find(|(invariant, _)| *invariant == "I-8.7")
            .map(|(_, verdict)| verdict)
            .expect("I-8.7 在清单里")
    };
    assert_eq!(
        verdict_of_a_ring_holding(&[(1, 1)]),
        InvariantVerdict::NotApplicable(
            TRANSACTION_NUMBERS_FROM_ONE_HELD_WITHOUT_A_COMPARABLE_PAIR
        ),
        "一条非 0 记录、号是 1：「从 1 起」判过且成立，理由不许说一个号都没判"
    );
    assert_eq!(
        verdict_of_a_ring_holding(&[(1, 0)]),
        InvariantVerdict::NotApplicable(NO_COMPARABLE_PAIR_OF_TRANSACTION_NUMBERS),
        "只有空发布：「从 1 起」没有对象，理由照旧"
    );
}

/// C469（I-8.7 的两处射程今天判不了） ② 回绕那一支：
/// 环 8 槽的一块盘，实例 1 在计数器 2、3、4 写事务号 0、1、2，计数器 1 那一格被环转过一圈之后的计数器 9（实例 2 的空发布）占了。
/// ① 判过一对（1 之后是 2）、成立；实例 1 的第一条被盖掉，「从 1 起」那半句射程外、什么都不记，整条报成立，不被压成不适用。
#[test]
fn transaction_numbers_judged_in_order_hold_when_the_ring_wrapped_over_the_first_record_of_the_instance(
) {
    let ring_slots = 8;
    let mut records = JournalRecordsOfADevice::new();
    for (counter, instance, transaction) in [(2u64, 1u32, 0u64), (3, 1, 1), (4, 1, 2), (9, 2, 0)] {
        let mut record = scanned_record_of(instance, 0, 0);
        record.transaction = transaction;
        records.insert(JournalRecordKeyOnADevice { counter, instance }, record);
    }
    let records_by_device = [(0u32, records)];
    let mut judgements = Judgements::default();
    judge_transaction_numbers_per_instance(&records_by_device, &mut judgements);
    assert_eq!(
        judgements.violation_count("I-8.7"),
        0,
        "① 那一对是 1 之后 2，不红"
    );
    judge_transaction_numbers_start_from_one_per_instance(
        &records_by_device,
        ring_slots,
        &mut judgements,
    );
    let report: BTreeMap<&'static str, InvariantVerdict> =
        judgements.into_report().into_iter().collect();
    assert_eq!(
        report["I-8.7"],
        InvariantVerdict::Holds,
        "① 判过成立；回绕盖掉实例 1 的第一条，「从 1 起」射程外不记，整条报成立"
    );
}

/// 游标是盘上读来的 u64：到顶就没有「下一个」，不回绕成 0。
#[test]
fn the_counter_after_the_cursor_is_none_at_the_top_of_u64() {
    assert_eq!(counter_after(5), Some(6));
    assert_eq!(counter_after(u64::MAX), None);
}

/// 一圈窗口（I-7.9 挡洞里留下的旧根用的那一个）：N = 35、环长 24 ⇒ 窗口 = txg > 11；N 小于环长时整个环都在窗口里。
#[test]
fn the_one_revolution_window_keeps_the_txgs_above_the_newest_minus_the_ring_length() {
    let window = OneRevolutionWindow {
        anchor_txg: 35,
        root_ring_slot_count: 24,
    };
    assert!(!window.contains(11), "35 − 24 = 11 不在窗口里");
    assert!(window.contains(12), "12 在窗口里");
    assert!(window.contains(35), "N 自己在窗口里");
    let young_ring = OneRevolutionWindow {
        anchor_txg: 7,
        root_ring_slot_count: 24,
    };
    assert!(young_ring.contains(0), "N < 环长：第 0 代也在窗口里");
}
