# 第 10z 批 a 块代码轮第一轮：正推腿的模型

都只在草稿目录的副本里跑；入库的是源码与输出，编译目录不入库。

- `probes/sonnet_probe_common.rs`：用 `GpuMemoryLedger` 的公开函数造真账，对一个「物理卡」模型（驱动实占、借据建上多少、销账之后卡上还没还的）量
  账给的量与此刻真能再借的量（保证值 = 空闲 − 活着借据还没建的 − 驱动还要长的）。改后、改前两份树各拷一份跑
  （改前 = 冻结副本 `patch -R` 掉实现员的 `crates.patch`）。
- `probes/sonnet_probe_new_only.rs`：只在改后的树上跑（`prepare_the_driver_share_before_starting_the_device` 改前没有）。
- `p2probe/`：std 里的小模型，`blocks.rs` 逐字抄自冻结副本 `crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2048-2106 行（去掉 cfg 属性）。
- `mut/run_mut3.py`：harness 档自造变异（改一处、跑 `cargo test -p singlefs-harness --lib [过滤]`、还原）。
- `logs/`：`final-new.log`（改后树，两份探针）、`final-old.log`（改前树）、`run-new-1.log` / `run-old-1.log`（头一次跑）、`p2-run.log`、`nvidia-smi-q-memory-pids.txt`（只读）。

命令（工作目录是各自的树根，`CARGO_TARGET_DIR` 各用各的）：
`scripts/run-with-memory-cap.sh 12G bash scripts/capped.sh 15 cargo test --offline -p singlefs-harness --test sonnet_probe_common --test sonnet_probe_new_only -- --nocapture --test-threads 1`
