# m3-close-i718-oldslot-r2 辩方腿（Sonnet）模型

- 报告：`research/prompts/m3-close-i718-oldslot-r2-sonnet-output.md`。
- 装置：`tests/i718_defense_probe.rs`（放进 `crates/singlefs-harness/tests/`，只在副本里跑）；`tools/walk-defense-switch.py` 在副本的 `crates/singlefs-checker/src/walk.rs` 里，I-7.18 真判不动，同时把十一个候选的判定一次算出放进线程局部变量；`tools/mod-slots-from-env.py` 让环洞池的每区槽数 S 读环境变量 `I718_S`。
- 复跑：`bash scripts/hosts/run-with-memory-cap.sh 16G bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <仓根或冻结树> --work <新的草稿目录> --model <模型目录>`；全跑 15 到 20 分钟（放后台），`RERUN_QUICK=1` 只验装置（约两分钟）。末尾两行 `name=rows same_as=…`、`name=scans same_as=…`。
- 产物：`out/summary-rows.txt`（按 S 与格汇总的逐镜像判定）、`out/summary-scans.txt`（错写者扫描）、`out/raw/`（各格原始输出）。
- 旋钮：`I718_S`（每区槽数，4 到 16）、`I718_FEW`（少取点）、`I718_NOSCAN`、`I718_NOROWS`、`I718_ROWS_FROM`、`I718_B_BASE`、`I718_B_REPEATS`。
