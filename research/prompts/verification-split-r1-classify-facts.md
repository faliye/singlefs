# 验证两档拆分 第一轮：重型判定事实表（主 agent 2026-09-27 在仓根用 .claude/hooks/lib_heavy_tests.py 的 heavy_tests_in_command_text 现跑）

| # | 命令（在仓根跑） | classify 返回 kind | 类别 |
|---|---|---|---|
| 1 | `cargo test -p singlefs-harness` | None | 不重型 |
| 2 | `cargo test -p singlefs-harness --test second_transaction_step_one_overwrite` | None | 不重型 |
| 3 | `cargo test -p singlefs-checker` | checker-tier-cargo | checker 档 |
| 4 | `cargo test --release -p singlefs-checker --test first_transaction_step_seven_layer0` | checker-tier-cargo | checker 档 |
| 5 | `cargo test -p singlefs-checker --lib` | None | 不重型 |
| 6 | `cargo test -p singlefs-checker --doc` | None | 不重型 |
| 7 | `cargo test -p singlefs-checker --tests` | checker-tier-cargo | checker 档 |
| 8 | `cargo test --package=singlefs-checker` | checker-tier-cargo | checker 档 |
| 9 | `cd crates/singlefs-checker && cargo test` | checker-tier-cargo | checker 档 |
| 10 | `cd crates/singlefs-harness && cargo test` | None | 不重型 |
| 11 | `cargo test` | full-cargo | 全量测试 |
| 12 | `cargo test --workspace` | full-cargo | 全量测试 |
| 13 | `cargo test -p singlefs-core -p singlefs-checker` | checker-tier-cargo | checker 档 |
| 14 | `cargo test -p singlefs-checker --test '*'` | checker-tier-cargo | checker 档 |
| 15 | `cargo nextest run -p singlefs-checker` | checker-tier-cargo | checker 档 |
| 16 | `cargo test --release -p singlefs-checker -- --include-ignored --exact full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean` | checker-tier-cargo | checker 档 |
| 17 | `./target/release/deps/first_transaction_step_seven_layer0-0123456789abcdef` | checker-tier-binary | checker 档 |
| 18 | `./target/debug/deps/second_transaction_step_one_overwrite-0123456789abcdef` | None | 不重型 |
| 19 | `bash .claude/gate.d/54-layer0-replay.sh` | layer0-stage | 层 0 |
| 20 | `bash .claude/gate.d/74-model-differential.sh` | None | 不重型 |
| 21 | `cargo test -p singlefs-checker -- --list` | None | 不重型 |
| 22 | `cargo test -p singlefs-format` | None | 不重型 |
| 23 | `cargo clippy --all-targets` | None | 不重型 |
| 24 | `bash research/scripts/mutate.sh x y crates/mutations.tsv` | crates-mutation-mutate | crates 变异整表 |
| 25 | `nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker` | checker-tier-cargo | checker 档 |
| 26 | `bash .claude/gate.d/59-crates-mutation-replay.sh` | crates-mutation-stage | crates 变异整表 |
