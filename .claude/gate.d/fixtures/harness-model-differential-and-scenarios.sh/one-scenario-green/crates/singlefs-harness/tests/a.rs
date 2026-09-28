#[test]
fn fast_case() {}

#[test]
#[ignore = "harness 耗时用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn slow_case() {}

// harness-test-granularity:one-scenario 同一个池按次序覆盖写，几轮合起来是一个场景
#[test]
fn sequential_overwrites() {
    for seed in [1, 2] {
        let pool = build_pool("x");
    }
}
