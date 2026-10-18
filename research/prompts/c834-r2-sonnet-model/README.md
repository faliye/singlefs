# C834 第二轮辩方腿的模型目录

- `patch/apply_candidates.py`：在冻结副本拷贝的 `crates/singlefs-core/src/mount.rs` 上装候选开关（环境变量 `C834_CANDIDATE`），只改 `placements_in_the_accounts_of_newer_readable_roots` 里那一判与它的调用处。候选：`today`（冻结副本的判）、`later_root_or_record`（第一轮的 K3）、`k6`（第一轮正推的 K6：K3 的凭据，或映射根槽里读出实例代号更大的索引节点，两样有一样就放过）、`mapping_root_only`（只留 K6 的映射根槽那一样）、`pointer_slots_overwritten`（第一轮攻方原型的指针槽读法，原样：读 32 KiB 再逐类解）、`pointer_slots_fixed`（同一读法，读盘长度按单元类别取：索引节点 16 KiB，打包记录单元与数据单元 32 KiB）、`always_refuse`。判决行里另带 `diag=`：R 的树表与映射根两个落点上读到什么（只读）。
- `tests/sonnet-nine-histories.rs`：第一轮正推的九段历史测试，改成读 `C834_CANDIDATE`，并多打 `JUDGE` 行（那一判看到的各项）。放进拷贝的 `crates/singlefs-harness/tests/c834_order_of_devices_decides_the_newer_root_judgement.rs` 跑。
- `src/main.rs`、`Cargo.toml`、`tools/aggregate.py`、`tools/run-batch-faults.sh`：第一轮攻方的崩溃状态全量原型，只改了判决行里多带的字段（`main.rs` 的 `second_mount` 里那一处 `filter`）。
- `tools/pieces-r2.py`、`tools/mapping-pieces.py`、`tools/nine-matrix.py`、`tools/ding-summary.py`：分件表、汇总。
- `tools/rerun.sh`：整条复跑（约 40 分钟，机器忙时更长），经 `scripts/hosts/run-with-memory-cap.sh` 跑；最后逐项比 `out/` 里的汇总，比时去掉 `example_ordinal`（多线程下举哪个状态不确定）。
- `out/`：`nine-matrix.txt` 与 `nine-*.log`（九段 × 候选 × 两种盘序）、`aggregate-all.txt`（k6、pointer_slots_fixed、later_root_or_record 的全量）、`aggregate-mapping-tail.txt`（mapping_root_only 的头尾段）、`aggregate-rollback.txt`（戊）、`ding-summary.txt`（丁的四个状态各候选的结局）、`existing-tests-by-candidate.txt`（入库两份用例逐候选）。
这些数都是拷贝上的数，不是入库装置上的数。
