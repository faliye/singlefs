#[test]
fn plain_case() {}

#[test]
#[ignore = "要 NVIDIA 卡"]
fn needs_gpu_case() {}

#[test]
fn several_sizes() {
    for size in [1, 2] {
        let pool = build_pool("x");
    }
}
