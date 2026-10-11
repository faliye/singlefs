# m3-leaf-death-r2 辩方腿（Sonnet）模型

第一轮攻方腿（`research/prompts/m3-leaf-death-r1-opus-model/`）的 E177 装置拷贝，加第二轮辩方的改动。回归：对第一轮已有的读法，E177 110 行聚合、合并 280 行聚合逐行相同。

- `src/leaf_death.rs`：改动都在这里（零条目版本配合并；读法 `C2Hold` `C2HoldZ` `T` `TS` `TSZ` `FSZ`；模式 `r2-e177` `r2-merges` `r2-sweep` `r2-rollback` `r2-indistinguishable`）。`src/main.rs` 只多 `borrowed_records` 字段与 4 个模式名。
- `tools/`：`agg.awk`（第一轮攻方腿的）、`counts2.py`、`window.py`、`sweep.py`。
- `rerun.sh`、`compare.sh`、`expected/`：复跑与期望；用法见 `rerun.sh` 文件头。需要 `SINGLEFS_REPO`（只用来找 `scripts/run-with-memory-cap.sh`），不需要 `crates/` 与 `scripts/experiments/Cargo.toml`。
- 完整模型输出（`e177.out` 约 2.3 MB、每批 `merges-*.out` 约 1.4 MB）不入库：`rerun.sh` 逐字节复出，`expected/sorted-output-sha256.txt` 钉着排序后的 sha256。
