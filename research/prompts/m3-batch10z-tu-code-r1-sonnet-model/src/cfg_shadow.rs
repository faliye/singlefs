mod outer {
    pub fn device_found_idle_by_a_poll(_x: &u8) -> bool { true }
    mod card_pool {
        #[cfg(any())]
        use super::device_found_idle_by_a_poll;
        fn device_found_idle_by_a_poll(_x: &u8) -> bool { false }
        pub fn pinned_like() -> bool { device_found_idle_by_a_poll(&1) }
    }
    pub fn run() -> bool { card_pool::pinned_like() }
}
fn main() { println!("pinned-like call returned {} (outer fn returns true)", outer::run()); }
