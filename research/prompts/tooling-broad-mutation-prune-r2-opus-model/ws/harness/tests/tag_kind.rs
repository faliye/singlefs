use model_core::kind::Kind;
use model_core::tag::tag;

#[test]
fn tag_scales_by_the_kind_code() {
    assert_eq!(tag(Kind::Data, 2), Kind::Data as u64 * 2);
}
