# m3-formal-code-b33-r1 正推腿（Sonnet）模型目录

入口：`tools/rerun.sh`（`REPO=<仓根> FROZEN=<冻结副本根> WORK=<草稿目录> bash tools/rerun.sh <段…|all>`）。全是 harness 档，经内存包装与线程上限。

- `tools/lib.sh`：环境（每份副本自己的 `CARGO_TARGET_DIR`）、`cg`（跑 cargo，打汇总与 `compiled:` 行）、`mutrun`（施加一行变异 → 跑 → 撤回）。
- `tools/mut.py`：施加 / 撤回变异（读法同 `research/scripts/crates-mutation-rows.py`：只把 `\n` 还原成换行；原文必须恰好命中一次）。
- `tools/p6_rows.py`：这一轮新增 / 删除的变异行的静态核（整行出现次数、锚点、点名的 fn 在不在、同一改法的别的行）。
- `tools/sweep.sh`、`tools/sweep-ignored.sh`：整个 harness 套件（非慢档、慢档）各跑一遍，探针行进日志。
- `probes/instrument_i54.py`：在副本的 `walk.rs` 里加三个 `eprintln!` 探针（B33F1 / B33C / B33ZERO）；`probes/make_probe_rollback_floor.py`：33-b 用例拷成 harness 档探针。
- `variants/*.spec`：本腿自己的变异变体（三行：文件、原文、替换文，`\n` 表示换行）。
- `out/`：这一趟的原始汇总（`mutation-runs-compact.txt` 每次变异后红的测试；`p6-rows.txt`；`static-check.txt`；`sweep-*-summary.txt`；`baseline-runs.txt`；抬 F 慢档那条红的原样）。
