# m3-c694-r1 正推腿的模型

冻结树的副本上加候选开关（`candidates.diff`，只动 `crates/singlefs-core/src/mount.rs` 与 `mounted_read.rs`，默认全关 = 今天的实现），加 harness 探针（`tests/`），只跑 harness 档。
复跑：`SINGLEFS_REPO=<主仓> [FULL=1] bash rerun.sh`（`FULL=1` 取 60 格，缺省只取覆盖写 5 次那一排）；比对：`compare.sh compare <抽出目录> expected/`。
`expected/` 是 `FULL=1` 在空目录里从头跑通一次的抽出结果（排序、去耗时）。开关：C694 的 `w1`（废根放行）、`w2`（读不出一律放行，对照）、`w4`（点名落点预隔离）；K3 的 `k3c`（只在映射根全零时先判树表 0 条）、`SINGLEFS_K3=r`（无条件先读树表，对照）。
