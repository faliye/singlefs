# m3-formal-code-b20-r1 正推腿的模型与脚本（Sonnet）

全部在腿自己的副本 `/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/work/`（冻结副本 `/tmp/claude-1000/m3-formal-code-b20-r1/tree/crates` 加仓根的 `Cargo.toml`、`Cargo.lock`、`.cargo/config.toml`）上做，仓内文件一个不改。
重做：先 `bash setup_work.sh`，再按下表跑；每条都经 `research/scripts/run-with-memory-cap.sh` 与 `capped.sh`，同一会话里另有一条在跑时要设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（脚本里已设）。

| 文件 | 干什么 | 档 |
|---|---|---|
| `anchor_check.py` | 核 `crates/mutations.tsv` 指定行的原文在源文件里恰好命中一次（mutate.sh 的锚点规则），可落出变异后的源文件 | 无编译 |
| `k3/probe_k3.rs`、`k3/run_k3.sh` | 18-2 两行变异：基线、行 2368、行 2369 各跑真用例与逐条断言各自求值的探针 | harness 档 |
| `f1f2/probe_f1f2.rs`、`f1f2/run_f1f2.sh` | F1 第二截三种接法、F2 第三截接对与五种接法，用 harness 与 core 的公开入口在 harness 档复做 | harness 档 |
| `pure/pure_base.rs`、`pure/run_pure.sh` | F5 函数与 `exactly_one_second_crash_point` 原样抄出的纯函数模型，加 F5 变异行 | rustc 单文件 |
| `check_mutants.sh` | 行 2362–2367 的变异源码各 `cargo check --tests`（只编不跑） | 只编 |
| `patch_vs_merged.sh` | 20-1 原补丁与合入后三份文件比 | git、patch、python |
| `outputs/` | 以上各次跑的原样输出（报告引文引的是这里） | — |
