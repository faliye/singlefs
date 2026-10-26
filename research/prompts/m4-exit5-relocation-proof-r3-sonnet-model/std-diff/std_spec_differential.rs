// 第三轮辩方腿 Z2：差分测试能不能判出一条写错的 std 规格。
// 被试规格：第二轮攻方的 std_spec_probe（<[T]>::binary_search 的 Ok(i) 是「第一个等于 x 的下标」，看着对，std 文档只说「任一个」）。
// 做法：规格谓词写成可执行孪生，在有界域（字母表 {0,1,2}，长度 0..=7 的全部有序切片，x 取字母表全部）上逐值喂给真 std，核孪生。
fn wrong_spec_binary_search(s: &[u8], x: u8, r: Result<usize, usize>) -> bool {
    match r {
        Ok(i) => i < s.len() && s[i] == x && s[..i].iter().all(|&v| v != x),
        Err(i) => i <= s.len() && s[..i].iter().all(|&v| v < x) && s[i..].iter().all(|&v| v > x),
    }
}
fn weaker_true_spec_binary_search(s: &[u8], x: u8, r: Result<usize, usize>) -> bool {
    match r {
        Ok(i) => i < s.len() && s[i] == x,
        Err(i) => i <= s.len() && s[..i].iter().all(|&v| v < x) && s[i..].iter().all(|&v| v > x),
    }
}
fn all_sorted_slices(max_len: usize, alphabet: u8) -> Vec<Vec<u8>> {
    let mut out = vec![vec![]];
    let mut layer: Vec<Vec<u8>> = vec![vec![]];
    for _ in 0..max_len {
        let mut next = Vec::new();
        for base in &layer {
            for v in base.last().copied().unwrap_or(0)..alphabet {
                let mut e = base.clone();
                e.push(v);
                next.push(e);
            }
        }
        out.extend(next.iter().cloned());
        layer = next;
    }
    out
}
fn main() {
    let slices = all_sorted_slices(7, 3);
    let (mut cases, mut wrong_fail, mut weaker_fail) = (0u64, 0u64, 0u64);
    let mut first_counterexample: Option<(Vec<u8>, u8, Result<usize, usize>)> = None;
    for s in &slices {
        for x in 0..3u8 {
            let r = s.binary_search(&x);
            cases += 1;
            if !wrong_spec_binary_search(s, x, r) {
                wrong_fail += 1;
                if first_counterexample.is_none() { first_counterexample = Some((s.clone(), x, r)); }
            }
            if !weaker_true_spec_binary_search(s, x, r) { weaker_fail += 1; }
        }
    }
    println!("name=std_spec_differential domain=sorted slices over {{0,1,2}} len<=7 cases={cases} wrong_spec_failures={wrong_fail} weaker_true_spec_failures={weaker_fail}");
    if let Some((s, x, r)) = first_counterexample {
        println!("name=std_spec_differential first_counterexample slice={s:?} x={x} std_returned={r:?}");
    }
}
