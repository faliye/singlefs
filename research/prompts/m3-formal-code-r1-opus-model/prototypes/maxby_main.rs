// max_by_key 在几个元素键相同时交回哪一个：两个键相同、内容不同的元素。
fn main() {
    let roots = [(5u64, 3u32, "slot1"), (5u64, 3u32, "slot2")];
    let newest = roots.iter().max_by_key(|root| (root.0, root.1)).unwrap();
    println!("max_by_key_picks={}", newest.2);
}
