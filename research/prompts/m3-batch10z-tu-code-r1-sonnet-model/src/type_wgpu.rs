mod card_pool {
    #[allow(non_camel_case_types)]
    type wgpu = u8;
    pub fn pinned_like() -> bool { let _ = wgpu::MAX; true }
}
fn main() { println!("{}", card_pool::pinned_like()); }
