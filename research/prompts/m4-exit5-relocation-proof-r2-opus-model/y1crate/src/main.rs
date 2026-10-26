// 产品构建（cargo build，不带 verus_keep_ghost）下跑三个被 Verus 验过的函数，逐个核它们的 ensures。
fn main() {
    let start = 10u64;
    let span = 2u64;
    let by_items = divergence_probe::divergence::destination_after(start, span);
    let by_macro = divergence_probe::divergence::destination_after_by_the_cfg_macro(start, span);
    println!("name=product-run fn=destination_after start={start} span={span} returned={by_items} ensures_holds={}", by_items == start + span);
    println!("name=product-run fn=destination_after_by_the_cfg_macro start={start} span={span} returned={by_macro} ensures_holds={}", by_macro == start + span);
    let slots = [2u64, 2, 2, 2, 2];
    let found = divergence_probe::std_spec_probe::first_index_of(&slots, 2);
    let holds = match found {
        Some(i) => slots[i] == 2 && slots[..i].iter().all(|&v| v != 2),
        None => true,
    };
    println!("name=product-run fn=first_index_of slice={slots:?} returned={found:?} ensures_holds={holds}");
}
