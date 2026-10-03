//! S2：四十次发布都落在环里，期望值按 layout 的常量自洽地算。
#[test]
fn forty_publishes_land_inside_the_ring() {
    for publish in 0..40u64 {
        assert_eq!(
            model_core::ring::slot_of(publish),
            publish % model_core::layout::REGION_SLOTS,
            "publish={publish}"
        );
    }
}
