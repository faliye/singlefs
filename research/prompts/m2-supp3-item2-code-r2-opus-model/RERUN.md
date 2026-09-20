# m2-supp3-item2-code-r2 Opus 攻方腿：复跑

全部在仓副本上跑（`rsync -a --exclude target --exclude .git` 拷自工作区，开工时 `crates/` 与开工快照逐个同 sha256）。副本上的数不是入库装置上的数。

1. 准备三份副本：`repo`（基线，只多 `tests/opus_r2_probe.rs`）、`mut`（施加变异用）、`small`（基线 + `scripts/small-copy-history.diff`：两块等大、单元区 384 槽的小盘，journal 环 128 MiB）。三份都把 `scripts/opus_r2_probe.rs` 放进 `crates/singlefs-harness/tests/`。各用各的 `CARGO_TARGET_DIR`（脚本里写死在 `/tmp/claude-1000/m2-supp3-item2-code-r2-opus/target*`，换机器改 `D=`）。
2. 基线：门禁 74 号那个测试二进制 `cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture`（日志 `logs/base-run1.log`）；探针 `scripts/baseline-batch.sh`（日志 `logs/baseline-batch.out`）。
3. 变异：`scripts/mutant-batch.sh <变异名>...`，每条变异的定义是 `mutants/<名>/{file,old,new[,nth],extra}`，`scripts/mutate.py` 施加、跑完还原。日志 `logs/mutant-batch.out`（第一批，extra 是那时的版本）、`mutant-batch2.out`（m2h / m2i / m2j 的 extra 换成全 workspace）、`mutant-batch3.out`（m4a / m4b 的 extra 换成现在这份）。`mutants/*/extra` 是最后一次用的版本。
4. 小盘：`scripts/small-batch.sh`（日志 `logs/small-batch.out`）。
5. 所有日志原样在 `logs/all-logs.tar.gz`（不含 `logs/stale-small/`：那一批在小盘副本上跑到了上一条变异还原后没重编的核心库，作废，见报告「这条腿自己的限度」）。

注意：`scripts/mutate.py` 还原时刷新 mtime（`os.utime`）是中途补的；补之前第一批里每条变异都改 `singlefs-core` 的一个文件、会整 crate 重编，没有吃到旧产物（报告里逐条说明）。
