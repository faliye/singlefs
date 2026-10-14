fn pinned_stream() {
    let segments = [2, 1];
    assert_eq!(segments.len(), 2);
    assert_eq!(crash_states(&segments), 5);
}
