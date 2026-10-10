// 核 K4-2 两个样本是合法的 Rust（edition 2021，rustc 1.77 起认 C 字符串字面量）：编得过、跑得出 3。
fn usable_gpu_cards() -> u32 {
    3
}
fn main() {
    let _s = cr"\"; let cards = usable_gpu_cards(); let _t = "";
    let _u = cr#"x"b"#; let more = usable_gpu_cards(); let _v = "";
    println!("cards={cards} more={more}");
}
