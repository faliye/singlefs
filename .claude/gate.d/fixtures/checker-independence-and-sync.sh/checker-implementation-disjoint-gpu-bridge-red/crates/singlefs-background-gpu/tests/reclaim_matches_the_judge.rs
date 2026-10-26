#[test]
fn reclaim_matches_the_judge() {
    assert_eq!(judge::gpu_unit_checks::dispatch_checksum_kernel(), 1);
}
