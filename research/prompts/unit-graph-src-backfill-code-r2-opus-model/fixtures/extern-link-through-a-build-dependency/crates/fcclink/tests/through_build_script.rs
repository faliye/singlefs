#[test]
fn fdatasync_through_the_library() {
    println!("fdatasync-result {}", fcclink::entry_point());
}
