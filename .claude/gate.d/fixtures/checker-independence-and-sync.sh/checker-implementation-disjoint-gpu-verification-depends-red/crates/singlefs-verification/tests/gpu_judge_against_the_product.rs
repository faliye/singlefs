#[test]
fn gpu_judge_against_the_product() {
    assert_eq!(product::reclaim_dispatch(), 16384);
}
