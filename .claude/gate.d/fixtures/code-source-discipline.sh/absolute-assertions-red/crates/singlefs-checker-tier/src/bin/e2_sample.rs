//! 判别力样本：住在 crates 下的实验装置同样折叠了种子，只扫 research 那一处就看不见它。

fn main() {
    let seed: u64 = 3;
    let random_state = seed | 1;
    println!("state={}", random_state);
}
