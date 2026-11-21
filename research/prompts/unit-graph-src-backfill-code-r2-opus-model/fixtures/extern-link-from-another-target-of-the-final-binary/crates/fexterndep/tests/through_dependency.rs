#[test]
fn fdatasync_through_the_library() {
    println!("fdatasync-result {}", fexterndep::entry_point());
}
