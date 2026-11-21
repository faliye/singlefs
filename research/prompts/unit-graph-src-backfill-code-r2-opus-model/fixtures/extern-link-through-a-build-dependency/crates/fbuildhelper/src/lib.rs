pub fn link_the_fake_library() {
    println!("cargo:rustc-link-lib=fakesync");
}
