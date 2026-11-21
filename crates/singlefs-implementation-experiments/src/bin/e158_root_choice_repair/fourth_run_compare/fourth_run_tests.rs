use super::*;

fn product_line(text: &str) -> ProductLine {
    ProductLine {
        fields: fields_of_a_result_line(text),
    }
}

/// Q5-同成（登记第六节；变异 M18）：手写三格（两格都 Ok、一格臂拒），同成栏只收两格。
#[test]
fn both_mounted_extra_reads_leave_out_the_refused_cells() {
    let today = product_line(
        "E7RESULT name=r4_cell a_class=K0 a_reads=100 a_read_bytes=409600 base_fingerprint=00ab",
    );
    let arms = [
        product_line("E7RESULT name=r4_cell a_class=K0 a_reads=104 a_read_bytes=425984 base_fingerprint=00ab"),
        product_line("E7RESULT name=r4_cell a_class=K1 a_reads=108 a_read_bytes=442368 base_fingerprint=00ab"),
        product_line("E7RESULT name=r4_cell a_class=K3 a_reads=60 a_read_bytes=245760 base_fingerprint=00ab"),
    ];
    let counted: Vec<(i64, i64)> = arms
        .iter()
        .filter_map(|arm| both_mounted_extra_reads(arm, &today))
        .collect();
    assert_eq!(counted, vec![(4, 16384), (8, 32768)]);
    let different_start = product_line(
        "E7RESULT name=r4_cell a_class=K0 a_reads=104 a_read_bytes=425984 base_fingerprint=00ac",
    );
    assert_eq!(both_mounted_extra_reads(&different_start, &today), None);
}

fn record(
    arm: &str,
    control: &str,
    name: &str,
    tag: SentenceTag,
    verdict: SentenceVerdict,
) -> SentenceRecord {
    SentenceRecord {
        arm: arm.to_string(),
        control: control.to_string(),
        name: name.to_string(),
        tag,
        verdict,
    }
}

/// V1（登记 11.1；变异 M19）：今天那一臂一句 [今] 打不中、其余全过 ⇒ 作废集合为空；[测] 在今天那一臂上打不中 ⇒ 今天那一臂那条对照管的量作废，
/// 别的臂只作废拿今天当参照的量；[臂] 打不中 ⇒ 那一臂那条对照管的量；同一句 [测] 每一臂都打不中 ⇒ 装置缺陷（全部臂一起）。
#[test]
fn the_void_scope_routes_by_the_sentence_tag() {
    let arms = ["today", "jia-cfg", "yi-cfg"];
    let passes = |control: &str, name: &str, tag: SentenceTag| {
        arms.iter()
            .map(|arm| record(arm, control, name, tag, SentenceVerdict::Pass))
            .collect::<Vec<_>>()
    };
    let mut sentences = passes("PC-N-ring", "truth_newer", SentenceTag::Measured);
    sentences.push(record(
        "today",
        "PC-N-ring",
        "hidden_in_ring_and_abandoned",
        SentenceTag::TodayDescription,
        SentenceVerdict::Fail,
    ));
    let (voided_by_a_today_sentence, defects_of_a_today_sentence) =
        fourth_run_void_scope(&sentences, &arms);
    assert!(voided_by_a_today_sentence.values().all(Vec::is_empty));
    assert!(defects_of_a_today_sentence.is_empty());

    let measured_and_arm_sentences = vec![
        record(
            "today",
            "PC-N0",
            "truth_not_newer",
            SentenceTag::Measured,
            SentenceVerdict::Fail,
        ),
        record(
            "jia-cfg",
            "PC-N0",
            "truth_not_newer",
            SentenceTag::Measured,
            SentenceVerdict::Pass,
        ),
        record(
            "yi-cfg",
            "PC-N-ring",
            "refused_after_r_rounds_k3",
            SentenceTag::ArmDefinition,
            SentenceVerdict::Fail,
        ),
    ];
    let (voided_by_measured_and_arm, defects_of_measured_and_arm) =
        fourth_run_void_scope(&measured_and_arm_sentences, &arms);
    let is_voided = |arm: &str, quantity: &'static str| {
        !voided_by_measured_and_arm[&(arm.to_string(), quantity)].is_empty()
    };
    assert!(is_voided("today", "Q4"));
    assert!(is_voided("jia-cfg", "Q4"));
    assert!(!is_voided("jia-cfg", "Q1-Q3"));
    assert!(is_voided("yi-cfg", "Q1-Q3"));
    assert!(!is_voided("today", "Q1-Q3"));
    assert!(defects_of_measured_and_arm.is_empty());

    let everyone_misses: Vec<SentenceRecord> = arms
        .iter()
        .map(|arm| {
            record(
                arm,
                "PC-lost-write",
                "overwrite_counted_and_not_last_confirmed",
                SentenceTag::Measured,
                SentenceVerdict::Fail,
            )
        })
        .collect();
    let (voided_on_every_arm, defects_on_every_arm) =
        fourth_run_void_scope(&everyone_misses, &arms);
    assert_eq!(
        defects_on_every_arm,
        vec!["PC-lost-write:overwrite_counted_and_not_last_confirmed".to_string()]
    );
    assert!(!voided_on_every_arm[&("jia-cfg".to_string(), "Q1-Q3")].is_empty());
}

/// PC-多读 (iii) 与 Q5-拒 的参照是同一种 N 的甲臂，不是今天那一臂（登记 5.8 第二件；变异 M20）。
#[test]
fn the_extra_reads_reference_is_the_jia_arm_with_the_same_witness() {
    let reference = |name: &str| extra_reads_reference_arm(fourth_run_arm_named(name).expect("臂"));
    assert_eq!(reference("yi-cfg"), Some("jia-cfg"));
    assert_eq!(reference("yi-narrow-cfg-carry"), Some("jia-cfg-carry"));
    assert_eq!(reference("yi-slot-three-rounds"), Some("jia-slot"));
    assert_eq!(reference("bing-cfg"), Some("jia-cfg"));
    assert_eq!(reference("today"), None);
}
