# m3-formal-code-b31-r1 云端正推腿的模型与脚本

只读核对 + 草稿副本上的变异 / 探针；一行都没改仓里的 `crates/`。入口：`tools/rerun.sh <草稿目录> static|prepare|tests|mutants|rows`（每段说明在脚本头）。

| 文件 | 干什么 | 对应格 |
|---|---|---|
| `tools/check_rows.py` | 变异表新加与替换的行：锚点命中次数、点名测试是不是 `#[test]` fn、`#[ignore]`、筛选词、同改法的行对（静态，不跑测试） | N5 |
| `tools/run_mutation_rows.py` | harness 档 / core 的 5 行（2734 2735 2736 2737 570）逐行改坏、跑点名测试 | N5 |
| `tools/sim_vs_core_root_ring.rs` | 用例里模拟的根环 与 core 分配器持有的根环 在 87 个 txg 上逐个比 | N3 |
| `tools/on_device_modes_recipe_vs_core.rs` | `on_device_modes` 拼根环表的写法 与 core 在同一次新池新建文件里持有的根环 `{:?}` 逐字比 | N3 |
| `tools/mutate_core_reclaim.py` `.sh` | core 记根回收门槛 5 种错法 a–e，各跑两条新用例 | N2 N3 |
| `tools/mutant_d_existing_suite.sh` | 变异 d 在已有用例 `root_ring_turn_in_one_mount` 上红不红 | N2 |
| `tools/probe_after_first_file.rs` `probe_baseline_and_d.sh` | 第一个文件之后 mkfs 树表落点的 defer / 已分配，基线与变异 d | N3 |
| `tools/probe_after_first_overwrite.rs` `probe2_baseline_and_d.sh` | 第一次覆盖写之后槽 50178 的分配记录，基线与变异 d（复用抹平） | N3 |
| `tools/per_txg_checker_probe.rs` `per_txg_checker_baseline_and_d.sh` | C551 新用例同一段历史，每个 txg 后跑池级 checker，基线与变异 d | N2 |
| `tools/instrument_core.py` `run_binaries_chunk*.sh` | 在草稿副本 core 里加回收事件打印与 SIM_PRE31 开关，逐个 harness 二进制直接跑（`--nocapture` 版才看得到事件） | N2 |
| `out/` | 上面各段的原样输出 | |

注意：`tools/mutate_core_reclaim.py`、`tools/run_mutation_rows.py` 里写死了冻结副本路径 `/tmp/claude-1000/m3-formal-code-b31-r1/tree`；`run_binaries_chunk.sh`（不带 nocapture）看不到事件，只留着当作废的第一版，census 用 `run_binaries_chunk_nocapture.sh`。
`prepare` 段把 `crates/singlefs-harness/src/scenario.rs` 里私有的 `run_new_pool_file_creation_keeping_the_allocator` 改成 `pub`（仅草稿副本，仓里没动）。
