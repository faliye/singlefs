fn usable_gpu_cards() -> u8 { 7 }
fn main() {
    let _a = cr#"x" "#; let _c = usable_gpu_cards(); let _b = cr#"y"#;
    println!("{:?} {:?} {}", _a, _b, _c);
}
