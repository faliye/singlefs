use super::*;

#[test]
fn closed_form_attempted_images_match_the_registered_anchors() {
    let mut every_specification = Segment::One.history_specifications();
    every_specification.extend(Segment::Two.history_specifications());
    for (anchor, group, point, expected) in ATTEMPTED_IMAGE_ANCHORS {
        assert_eq!(
            attempted_images_of(&every_specification, group, point),
            Some(expected),
            "{anchor} {} {}",
            group.name(),
            point.name()
        );
    }
    assert_eq!(
        Segment::One.history_specifications().len(),
        6 + 105 + 6 + 42,
        "第一段：nrb 6、rs1 105、G4 的 nrb 6 与 rs1 42"
    );
    assert_eq!(
        Segment::Two.history_specifications().len(),
        27 + 3 + 27 + 27 + 1 + 9 + 3,
        "第二段：ss、skr、sw-crash、sw-read、G1、G2、G3"
    );
    assert_eq!(
        rollback_action_strings().len(),
        1 + 4 + 16,
        "动作串：不做、一个、两个"
    );
    let names: BTreeSet<String> = every_specification.iter().map(history_name).collect();
    assert_eq!(names.len(), every_specification.len(), "历史名两两不同");
}
