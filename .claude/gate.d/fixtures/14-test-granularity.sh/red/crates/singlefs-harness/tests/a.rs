#[test]
#[ignore = "harness 耗时用例：单线程 debug 下 75 秒（crates/singlefs-harness/test-timing.tsv）；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn fast_case() {}

#[test]
#[ignore = "harness 耗时用例：单线程 debug 下 75 秒（crates/singlefs-harness/test-timing.tsv）；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn slow_case() {}

#[test]
fn several_sizes() {
    for size in [1, 2] {
        let pool = build_pool("x");
    }
}
