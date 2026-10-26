use z2crate::f2_trait_impl_outside::{read_checked, BadDevice};
fn main() {
    let (start, span) = (10u64, 2u64);
    println!("name=product-run fn=f1_feature_fork::pick returned={} ensures_holds={}", z2crate::f1_feature_fork::pick(start, span), z2crate::f1_feature_fork::pick(start, span) == start + span);
    println!("name=product-run fn=f1b_debug_assertions_fork::pick returned={} ensures_holds={}", z2crate::f1b_debug_assertions_fork::pick(start, span), z2crate::f1b_debug_assertions_fork::pick(start, span) == start + span);
    println!("name=product-run fn=f5_macro_outside::pick returned={} ensures_holds={}", z2crate::f5_macro_outside::pick(start, span), z2crate::f5_macro_outside::pick(start, span) == start + span);
    let r = read_checked(&BadDevice, 0);
    println!("name=product-run fn=f2_trait_impl_outside::read_checked returned={r} ensures_holds={}", r <= 10);
}
