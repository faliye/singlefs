use super::*;

/// C8e（登记 r5 5.4、停机条款 PH6）：手造一份真相，同一投影两份不同的后三项，必须停在 PH6。
/// 9.2「真相表同一投影两份后三项时不停、取先到的那一份」点名这一条。
#[test]
#[should_panic(expected = "停机条款 PH6")]
fn generated_record_truth_given_two_tails_for_one_projection_halts_on_ph6() {
    let root_fields_in_records_profile =
        RunProfile::root_fields_in_records(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let projection =
        VersionFields::of_root(&adversarial_chosen_root(root_fields_in_records_profile));
    let mut truth = GeneratedRecordTruth::new();
    truth.note(
        projection,
        root_fields_in_records_profile.record_fields_of_the_timeline_publish(0),
    );
    truth.note(
        projection,
        root_fields_in_records_profile.record_fields_of_the_timeline_publish(1),
    );
}
