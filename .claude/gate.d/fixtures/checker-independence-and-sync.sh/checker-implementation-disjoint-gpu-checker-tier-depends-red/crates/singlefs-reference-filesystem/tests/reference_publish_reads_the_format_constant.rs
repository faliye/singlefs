#[test]
fn reference_publish_reads_the_format_constant() {
    assert_eq!(singlefs_reference_filesystem::reference_publish(), format_constants::SAMPLE_BYTES);
}
