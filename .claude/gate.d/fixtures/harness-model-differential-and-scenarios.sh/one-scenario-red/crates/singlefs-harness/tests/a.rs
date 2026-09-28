#[test]
fn fast_case() {}

#[test]
#[ignore = "harness 耗时用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn slow_case() {}

#[test]
fn several_sizes() {
    for size in [1, 2] {
        let pool = build_pool("x");
    }
}
