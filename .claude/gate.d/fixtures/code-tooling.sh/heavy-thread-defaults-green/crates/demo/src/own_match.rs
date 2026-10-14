// 样本：读和回退写在同一条 match 里（Err 分支回退到登记值），读的变量 capped.sh 不设，不点名
pub fn match_threads() -> usize {
    match std::env::var("SAMPLE_MATCH_THREADS") {
        Ok(value) => value.parse().expect("正整数"),
        Err(_) => default_heavy_threads(include_str!("../../../.claude/heavy-thread-default")),
    }
}
