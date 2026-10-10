mod card_pool {
    #[allow(non_camel_case_types, dead_code)]
    enum Fake { PollType }
    #[allow(non_camel_case_types)]
    type wgpu = Fake;
    pub fn pinned_like() -> bool { let _ = wgpu::PollType::Poll; true }
}
fn main() { println!("{}", card_pool::pinned_like()); }
