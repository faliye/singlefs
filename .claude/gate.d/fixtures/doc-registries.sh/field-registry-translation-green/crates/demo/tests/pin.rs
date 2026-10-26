fn pinned_second_stream_sequence() {
    let segments = [2, 1];
    assert_eq!(1 + segments.iter().map(|length| (1u32 << length) - 1).sum::<u32>(), 5);
}
