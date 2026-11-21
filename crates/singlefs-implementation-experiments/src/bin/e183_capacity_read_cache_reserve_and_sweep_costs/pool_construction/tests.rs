use super::*;

#[test]
fn variant_name_keeps_only_the_leading_identifier() {
    assert_eq!(
        variant_name_of("SpaceAdmissionRefusedBeforeAcquisition { instance: 3 }"),
        "SpaceAdmissionRefusedBeforeAcquisition"
    );
    assert_eq!(variant_name_of("Recovery(Foo)"), "Recovery");
}
