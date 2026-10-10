mod card_pool {
    #[allow(non_camel_case_types)]
    struct std;
    pub fn pinned_like() -> bool { std::panic::catch_unwind(|| 1).is_ok() }
}
fn main() { println!("{}", card_pool::pinned_like()); }
