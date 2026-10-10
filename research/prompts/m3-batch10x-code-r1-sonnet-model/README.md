# m3-batch10x-code-r1 正推腿模型

- `forward_probe.rs`：放进冻结副本的 `crates/singlefs-harness/tests/` 下编，只用 `singlefs_harness::gpu_memory_ledger` 的公开接口，假读数，不要显卡。七条：P1a、P1b、P2、P3、P4、P5、P6。
- `mutate-probe.py` + `mutants1.json`：在 `crates/singlefs-harness/src/gpu_memory_ledger.rs` 上逐个换一处，跑 `cargo test -p singlefs-harness --lib gpu_memory_ledger`，还原。仓副本路径写死在草稿目录。
- `forward_probe-run.log`、`mutate-probe-run.log`：最后一次跑的原样输出。
- 跑法（草稿目录的仓副本里）：`CARGO_TARGET_DIR=<草稿>/target bash research/scripts/capped.sh 10 bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --offline --test forward_probe -- --nocapture --test-threads=1`
- 冻结副本被测文件的 sha256：`gpu_memory_ledger.rs` 9f6fc8fd74f54b87cbc91979940003a49c661135421096e16153150ccfb47b0f（与 research/prompts/m3-batch10x-code-r1-snapshot/crates-sha256.txt 一致）。
