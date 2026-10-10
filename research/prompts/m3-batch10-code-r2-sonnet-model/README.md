# 正推腿模型目录（m3-batch10-code-r2）

全部是在冻结副本的拷贝（`/tmp/claude-1000/m3-batch10-code-r2-sonnet/work/`，只拷了 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates`）上量的，不是入库装置上的数。

- `mut2.py apply|restore <行号>`：把冻结副本 `crates/mutations.tsv` 的那一行（原文必须在文件里恰好命中一次）换进拷贝、跑完还原并 touch（第一版没 touch，cargo 把还原回去的旧 mtime 当没变，第一趟 2773 的日志 `mut-2773.log` 因此混进了 2772 的变异，已弃，不入库；重跑的是 `mut-2773b.log`）。
- `runrow.sh <行号>`：套 `run-with-memory-cap.sh 2G` 与 `capped.sh 7`，跑那一行第 5 段的测试参数，日志在 `logs/row-<行号>.log`。
- `logs/mut-fallback.log`：自写的变异「重读那一遍整环仍没有时退回第一遍的读数」（改的是 `mount.rs` 里重读那一段 `let Some(window_anchor) = …else { return Err(…) }`，改法见 `fallback-mutation.txt`），跑三份 harness 用例文件，22 条全绿。
- `logs/mut-fonly.log`、`logs/mut-fonly-lib.log`：自写的变异「丁-defer 门槛只取 F_生效、丢掉环里最旧有效根那一项」。
- `k1probe-…rs`：影子账那份测试文件的拷贝加一条探针，造 K1 那一形（实例 1 到 24 被抛弃、连崩 11 次、做完一次可写挂载），F 抬到 24（被拒 `RollbackFloorAboveCeiling` ceiling 20）与抬到 20 各量一次。
