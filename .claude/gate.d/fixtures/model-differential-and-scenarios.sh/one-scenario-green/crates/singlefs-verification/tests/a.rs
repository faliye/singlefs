#[test]
fn plain_case() {}

#[test]
#[ignore = "要 NVIDIA 卡"]
fn needs_gpu_case() {}

// harness-test-granularity:one-scenario 同一个池按次序覆盖写，几轮合起来是一个场景
#[test]
fn sequential_overwrites() {
    for seed in [1, 2] {
        let pool = build_pool("x");
    }
}
