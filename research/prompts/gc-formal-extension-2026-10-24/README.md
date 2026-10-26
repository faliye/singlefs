# GC 形式化扩展证据

正式报告：[GC 实现、旧读者与崩溃协议形式化扩展](../../../records/2026-10-24-GC实现与旧读者及崩溃协议形式化扩展.md)。此目录为本轮原始验证输出，源码身份由 `source-sha256.json` 记录；不以 HEAD 或整个工作区代表本轮修改。

在仓库根目录运行，实际 Verus 与 cargo 调用通过 `scripts/hosts/run-with-memory-cap.sh` 设内存上限；门禁自身负责包装 Verus。没有使用未验证的求解器公理或关闭已有契约。

| 输出 | 复现入口 |
| --- | --- |
| verus.json / verus.stderr | 固定安装的 Verus，`--crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating --rlimit 10 --num-threads 2 --output-json crates/singlefs-replay-prefix/src/lib.rs` |
| proof.log | `python3 scripts/gate/verus-proof-check.py proof .`，逐项加最终无过滤整 crate 检查 |
| vacuity.log | `python3 scripts/gate/verus-proof-check.py vacuity .` |
| new-landings.log | `python3 scripts/gate/verus-proof-check.py must-red-landings . --write`；实际验证整表 194 行，仅给新增 13 行写入缺失的落点，保留已有行登记 |
| coverage.log | `python3 scripts/gate/verus-proof-check.py coverage .` |
| core-tests.log | `cargo test -p singlefs-core --lib compaction_plan::tests` |
| allocator-tests.log | `cargo test -p singlefs-core --lib allocator::tests` |
| harness-tests.log | `cargo test -p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses` |
| gc-recovery-tests.log | `cargo test -p singlefs-harness --test compaction_intent_is_logged_and_resumed_after_a_crash --test compaction_relocation_batches_publish_through_the_mapping --test snapshot_and_compaction_over_a_file_other_than_the_first` |
| clippy.log | `cargo clippy -p singlefs-core -p singlefs-replay-prefix --all-targets -- -D warnings` |
| fmt.log | `cargo fmt -p singlefs-core -p singlefs-replay-prefix -- --check`，无输出表示成功 |
| product-mutation-anchors.log | `python3 scripts/mutation/crates-mutation-rows.py static-check . --table research/prompts/gc-formal-extension-2026-10-24/product-mutations.tsv` |
| product-mutations.log | `bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <变异名> … .`，名字取归档 `product-mutations.tsv` 的 14 行第一列 |

`build.log` 是独立参考实现初次编译日志，当时有未使用导入警告；随后清理导入，最终以 clippy 与测试日志为准。原始日志保留不改写。

本轮没有运行全工作区门禁或全量设备崩溃枚举。协议定理的设备、解析和版本精化前提详见正式报告，不能把这里的结果解释为全文件系统端到端证明已经完成。
