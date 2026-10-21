//! checker 档模块：无（样本：打覆盖行与计数行的崩溃枚举用例，假 cargo 照 .crash-point-replay-sample-tools/cases/ 打日志）
#[test]
fn plain_stream_quick() {
    // crates 侧接上了覆盖行（crash-flow-coverage.py 认 crates/ 下有没有这个词）
    println!("CRASH_FLOW_COVERAGE entry=publish_first_file step=Barrier segment=2 expansion=expanded domain=quick");
}

#[test]
#[ignore]
fn plain_stream_full() {}
