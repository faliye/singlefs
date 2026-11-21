# gpu-tests-placement-design-r2：判决（2026-11-19）

正文 `research/prompts/_gpu-tests-placement-design-r2-body.md`，开工快照 `research/prompts/gpu-tests-placement-design-r2-snapshot/`。这一题的第二轮，前一轮判决 `research/prompts/gpu-tests-placement-design-r1-main-verification.md`。调度记录 `records/2026-11-19-产品GPU包测试与E202装置归位.md`。被判文件快照 `judged-sha256.txt` 56 份，派腿之前记；写判决前 `sha256sum -c --quiet` 对 56 份与 kb 三份都对得上（退 0）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/gpu-tests-placement-design-r2-sonnet-output.md`（sha256 521dc2525cf196e9…） | 九格都答：录入键逐字节相同只能写成同参数下两侧写进库的行集合相同；产品测试存取代码约 830 行；实验包名 `crates/singlefs-implementation-experiments`；26 个装置留 4 进 22，E161 推不出；共用模块推乙（实验包私有一份约 820 行）；⑨ 对 15 处 `#[path]` 与 11 处跨包 `include_str!` 逐处判了 |
| 攻方（opus） | `research/prompts/gpu-tests-placement-design-r2-opus-output.md`（sha256 7eff9cad…） | 第 1 格打中：录入键拼了各侧自己的单元指纹，两侧同键做不到，两份存取代码写同一个键空间就有两份布局实现；第 3 格：D13（验证路线） 已定项 15 把装置写进验证包，E161 观察者登在验证包入口表；第 6 格：E202 带 131 条测试，含真卡对拍产品 GPU；第一轮判决两处推论不准；正文 7 处对不上 |

- 快照：`sha256sum -c --quiet research/prompts/gpu-tests-placement-design-r2-snapshot/judged-sha256.txt` 与 `kb-sha256.txt` 都退 0，开工快照之后被判文件都没改过，所以直接在主工作区核，没有另跑 `snapshot-tree.py`。
- `python3 scripts/agents/cite-check.py research/prompts/gpu-tests-placement-design-r2-opus-output.md research/prompts/gpu-tests-placement-design-r2-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_gpu-tests-placement-design-r2-background.md`：「核了 69 处引文、129 处裸行号，7 处对不上，129 处没判」。7 处对不上逐处看过：攻方引 `verdict_store.rs:1655`、`:6524` 的布局号常量实际在第 2012、7132 行，`:2778` 指到标题行，正推引 `gpu-verification-server/README.md:48` 指到标题行，都是行号漂了、原文在；攻方引第一轮判决第 24 行那句，主 agent 已照它报的改过（三份、形参名不同），所以找不到原句。
- 主 agent 现核坐实（读码与 `grep`，不起编译）：录入键 `crash_library.rs:1785-1792`；判定库布局号 6、布局行不对拒开库（`verdict_store.rs:38`、`:123`）；D13（验证路线） 已定项 15 定案原文（`.claude/kb/decisions/13-验证路线.md:405`）；E161 观察者登记（`judging_units_entries.rs:537-543`）；E202 131 条 `#[test]`、`e202_gpu_cpu_gc_comparison.rs:12381` 起是真卡上产品 GPU 一侧的对拍；`pci_bus_identifier_without_the_domain` 第三份在 `scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:488`；`e165_replay_bounded_enumeration` 由门禁 formal-verification 的红例钉表起（`.claude/gate.d/e165-pinned-red-examples.tsv`）；`git grep` 对 e161 在 `.claude/gate.d` 与 `scripts` 下零命中。
- 两条腿的数（正推约 830 行、约 820 行、30 组键会变、233 个目标上界；攻方的模块链）是腿读码推的，主 agent 没有复跑；判决只拿它们定次序，不拿它们定岔路。

## 二、判决

岔路由用户在弹窗里定（调度记录「用户定案（弹窗，第二轮）」），这一轮按定案收：

1. **产品测试的判定库存取**：写进 prune-server 那一份库，用产品测试自己的一族键与布局，与验证包的崩溃判定库（布局号 6 的那个键空间）互不牵连。攻方第 1 格打中的「两份布局实现写同一个键空间」因此不发生；「两侧同键」不再是要求，第一轮判决第 1 条里「两侧写进库的字节怎么证逐字节相同」那一问作废，换成：产品测试那一族的键照同一套组成（单元指纹加输入摘要，D13（验证路线） 已定项 17），不许特例。这一族要登进 `scripts/prune-server/prune_server_client.py` 文件头「键空间」或 `verdict_store.rs` 文件头的族表，并登进 `.claude/gate.d/verdict-library-scripts.tsv`。
2. **D13（验证路线） 已定项 15 改定案**：验证包装测试与门禁自己起的工具；实验装置住新开的实验包。要改的那一句与变更史照 `.claude/rules/format-evolution.md` 由写回员写，依据是用户这两次定案。攻方报的「装置的内联单测在门禁按条那一路里，搬走后掉出门禁」采：实验包的测试要接进门禁 crash-point-replay 按条复用那一路，改门禁随措施一起做。
3. **装置分法**：`crash_library_reader`、`new_pool_file_creation_on_device`、`new_pool_file_creation_device_log_check`、`e165_replay_bounded_enumeration` 留验证包（门禁与崩溃入口起的工具）；其余 22 份连同 E202 进实验包，E161 也进（下一条）。
4. **E161**：改崩溃引擎认观察者的接口，让实验包登记自己的观察者，E161 搬走。验证包 `judging_units_entries.rs` 里 E161 那一条与 `bin:` 目标的组随着挪到实验包的构建脚本。
5. **装置用的 `tests/common*`**：实验包自己写它用到的那一部分，验证包 `tests/common/mod.rs` 不动，验证包 340 个测试目标的键不变。攻方说这样会让 E170 的停机条款 S1 变成拿副本比副本（`e170_specification_executor_export.rs:10`），这一条交实现员时点名，写之前先读 S1 的原文判它比的是什么。
6. **E202 拆开**：E202 里测产品 GPU 一侧的测试（真卡对拍那几条，`:12381` 起）挪进产品 GPU 包 `tests/`，实验包里的 E202 只留装置自己的单测。攻方「实验包里的 E202 等于测试包形状」因此采，第一轮判决「实验包只装装置二进制」那一句照这一条落实。
7. **`pci_bus_identifier_without_the_domain`**：产品测试那一份、验证包那一份、E163 那一份各在各的包里，是各写各的；测试包那一份第 204 行「本包测试不依赖 验证包」的注释搬进产品包之后才成立，搬的时候照现状改写。
8. **单元键与现判**：正推第 8 格的分法采（产品 GPU 侧约 30 组、E202 GPU 侧 1 组的键会变，要现判；参照侧预计不变），数在实现段由主 agent 用 `--list-items --with-keys` 量。`scripts/mutation/crates-mutation-rows.py:391` 的文档串照改，不登记留旧：用户定过不拿「要重跑、代价大」当理由，那张表照判法指纹整表现判。
9. **措施**：正推第 9 格的五条检查采用，角色四种（产品、验证、测试支撑、实验装置）；⑧ 的允许共用清单加 `singlefs-unit-graph`（连同它的 `[dependencies]`：`verus_syn`、`proc-macro2`、`quote`）与 `prune-server`，另判这两个不依赖 wgpu、不含判器；另加一条「没有包依赖实验包」（正推第 3 格）。⑨ 的逐处判法以正推第 9 节的表为起点，交工具员时现扫再定。
10. **与别的会话的修补**：梳理子 agent 的那一路会话的合并补丁要改的 3 份（取卡模块、它的单测、验证包入口表的注释）不与这一轮的方案冲突，照常先落；拆包以它落完的样子为基线。

## 三、改法

- 「二、判决」第 1–9 条都是零轮（没有一条被攻过）。它们是搬家与接口改动，不改产品代码的行为，没有一条要在探针里比 A/B 两臂；产品 GPU 包里改的只有测试与构建脚本。
- 次序（照正推第 10 节，主 agent 收成三簇）：一，工具员做措施（规则、`crate-roles` 格、⑧ 加允许清单与 ⑨、检查单第九格、放置类弹窗的钩子、实验包测试接进门禁）；二，实现员拆包：产品 GPU 包 `tests/` 与 `build.rs`、产品测试的存取族、E202 拆开；三，实现员建实验包、搬 22 份装置、改引擎认观察者的接口。二、三两簇撞根 `Cargo.toml`（`crates/mutations.tsv` 只追加不算撞），排先后；写回员改 D13（验证路线） 已定项 15 与 `verification.md`「定义与名字」跟在第三簇之后。
A/B 读数：任务 <号>

## 交用户的

这一轮要交用户的四件已在弹窗里定了（调度记录）。

## 这一轮被判过的文件

- .claude/gate.d/checker-independence-and-sync.sh：判了 ⑧ 的允许共用清单加 `singlefs-unit-graph`（连同 `verus_syn`、`proc-macro2`、`quote`）与 `prune-server`，另判没有包依赖实验包，第 46 行写明的 `#[path]`、`include!` 盲区由 ⑨ 判
- crates/singlefs-background-gpu/build.rs：判了它接过对拍两侧单元键的计算（经 `singlefs-unit-graph`）
- crates/singlefs-background-gpu/Cargo.toml：判了它加 `[dev-dependencies]` `prune-server`、`singlefs-unit-graph` 与 `[build-dependencies]` `singlefs-unit-graph`，真卡测试目标挂 `tests-on-a-real-gpu`
- crates/singlefs-background-gpu-tests/build.rs：判了它算键那一段进产品 GPU 包 `build.rs`，E202 那几组进实验包的构建脚本
- crates/singlefs-background-gpu-tests/Cargo.toml：判了它随撤包删掉
- crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs：判了它拆开：测产品 GPU 的真卡对拍那几条进产品 GPU 包 `tests/`，装置与它自己的单测进实验包
- crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs：判了它随单元键那一段进产品 GPU 包，E202 那几组进实验包
- crates/singlefs-background-gpu-tests/src/lib.rs：判了它随撤包删掉
- crates/singlefs-background-gpu-tests/src/reference_units_entries.rs：判了它随单元键那一段进产品 GPU 包，E202 那几组进实验包
- crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs：判了它进产品 GPU 包 `tests/`，E202 那一份实验包自己写；梳理子 agent 的那一路会话的修补先落
- crates/singlefs-background-gpu-tests/tests/common_history_pool_through_the_library/mod.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs：判了它由产品测试自己一族键的存取代码顶替，E202 那一份实验包自己写
- crates/singlefs-background-gpu-tests/tests/common_pool_on_memory_devices/mod.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`，第 5 行「卡与清场由服务定」照「卡只归 GPU 服务，任务不拿卡」改写
- crates/singlefs-background-gpu-tests/tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs：判了它随取卡模块进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs：判了它随取卡模块进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/reference_units_entries_name_functions_that_exist.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`
- crates/singlefs-verification/Cargo.toml：判了它的 `[[bin]]` 去掉搬进实验包的 22 份，留 4 份门禁工具
- crates/singlefs-verification/src/bin/crash_library_reader.rs：判了它是门禁或崩溃入口起的工具，留验证包
- crates/singlefs-verification/src/bin/deletion_write_dump.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e142_new_pool_file_creation_write_dump.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e156_allocation_basis_counts.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e158_root_choice_repair.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs：判了它进实验包，随崩溃引擎认观察者的接口一起改
- crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs：判了它是门禁或崩溃入口起的工具，留验证包
- crates/singlefs-verification/src/bin/e168_delete_ring_window_borrow.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e170_specification_executor_export.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e171_three_oracle_collusion.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e175_product_path_inode_watermark_regression.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e182_unreadable_abandoned_root_forks.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e184_regular_layer_cost_counts.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e184_regular_layer_cost_counts_write_dump.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/e198_instance_table_pointer_birth_skip_premise.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/bin/new_pool_file_creation_device_log_check.rs：判了它是门禁或崩溃入口起的工具，留验证包
- crates/singlefs-verification/src/bin/new_pool_file_creation_on_device.rs：判了它是门禁或崩溃入口起的工具，留验证包
- crates/singlefs-verification/src/bin/new_pool_file_creation_region_bytes.rs：判了它是实验装置，进实验包
- crates/singlefs-verification/src/judging_units_entries.rs：判了 E161 的观察者条与 `bin:` 目标的组挪到实验包的构建脚本，崩溃引擎认观察者的接口改成可由实验包登记
- crates/singlefs-verification/tests/common/mod.rs：判了它留在验证包、内容不动，实验包自己写它用到的那一部分

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 15 | 推翻 | 2026-11-19 改了：用户定案验证包只装测试与门禁工具，实验装置住实验包；写回员照 `.claude/rules/format-evolution.md` 改正文与变更史 |
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-19 不受影响：产品测试那一族的键照同一套组成，包名与路径不进键 |
