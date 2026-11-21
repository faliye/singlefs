#[link(name = "fakesync")]
extern "C" {}

#[test]
fn fdatasync_through_the_library() {
    println!("fdatasync-result {}", fexternfinal::entry_point());
}
