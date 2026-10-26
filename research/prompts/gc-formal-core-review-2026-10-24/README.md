# GC/core 严格复核证据

2026-10-24 开始，2026-10-25 收尾。正式结论见 [records 审查报告](../../../records/2026-10-24-GC形式化与core改动复核.md)，详细源码说明见 [src/README.md](../../../crates/singlefs-replay-prefix/src/README.md)。

| 文件 | 入口与解释 |
| --- | --- |
| core-all-tests.log | `cargo test -p singlefs-core --lib`，最终 350 项 |
| harness-tests.log | `cargo test -p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses --test compaction_intent_is_logged_and_resumed_after_a_crash --test compaction_relocation_batches_publish_through_the_mapping --test snapshot_and_compaction_over_a_file_other_than_the_first`，36 项 |
| plan-tests.log / allocator-tests.log | 过程中的定向单测；最终结论以全 core 日志为准 |
| clippy-final.log | `cargo clippy -p singlefs-core -p singlefs-replay-prefix --all-targets -- -D warnings` |
| clippy.log | 提取私有转换函数前的过程检查，不替代最终检查 |
| rustdoc.log | `RUSTDOCFLAGS='-D warnings' cargo doc -p singlefs-replay-prefix --no-deps` |
| fmt.log | `cargo fmt -p singlefs-core -p singlefs-replay-prefix -- --check`，退出码 0、无输出 |
| verus.json / verus.stderr | 固定 Verus 0.2026.09.20，`--crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating --rlimit 10 --num-threads 2 --output-json crates/singlefs-replay-prefix/src/lib.rs` |
| coverage.log | `python3 scripts/gate/verus-proof-check.py coverage .` |
| formal-anchors.log | 核对最终 194 行均六列、原文唯一命中；本轮未重跑其全部变异 |
| review-mutations.tsv / review-mutations.log | 从表第一列构造九个 `--item`，调用 `.claude/gate.d/checker-tier-crates-mutation-replay.sh`，实际运行指定测试和基线 |
| mutation-anchors.log | 用 `scripts/mutation/crates-mutation-rows.py static-check . --table <review-mutations.tsv>` 检查九行 |
| all-changed-product-mutations.tsv / all-product-anchors.log | 前轮 14 行加本轮 9 行的最终静态核对；不是 23 行实测复跑日志 |
| check-reference.py / original-pure-function-bodies.json / reference-check.log | 从改动前固定 Git blob 冻结词法单元，再核对测试参考函数；脚本可直接复跑 |
| reference-check-rejected-staging.log / rejected-staged-function-bodies.json | 首次误用可变暂存区作旧基线，被断言拒绝的原始证据；没有据此改变参考实现 |
| source-sha256.json | 收尾源码身份；不代表 HEAD 或整个工作区 |

重型执行通过项目内存包装器运行。九个反例的成功指预期测试确实失败，不能与普通测试通过混为一谈。代码与规格的剩余信任边界以 records 和源码说明为准。
