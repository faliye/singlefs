# split-governance-scripts-services-r1：判决（2026-11-20）

正文 `research/prompts/_split-governance-scripts-services-r1-body.md`，开工快照 `research/prompts/split-governance-scripts-services-r1-snapshot/`。这是这一题的第一轮；同一批拆分的钩子与门禁在另一题 `split-governance-hooks-gates-r1`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（opus） | `research/prompts/split-governance-scripts-services-r1-opus-output.md` | 第 1、2、3 问不成立：四处打中同一个形状，算键库拆成入口加库内模块之后，别的族在入口模块上换属性、或按路径读入口原文，够不到拆出模块按自己全局名读的调用方；第 4 问 python 与 shell 的子进程、`__file__`、`sys.path` 成立，按路径读入口原文不成立（打中三），服务没重编那一条推不出 |
| 正推（sonnet） | `research/prompts/split-governance-scripts-services-r1-sonnet-output.md` | 第 5、6、7 问都成立；线索：`scripts/hosts/queue-health.py:1137` 起的循环只读入口 `task_server.py` 原文找两种写法，自证 `attempt-forms-match-sources` 那一格应红 |

核查命令：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py split-governance-scripts-services-r1 --out /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/split-governance-scripts-services-r1-tree --repository /home/fy5090/code/singlefs`：结局 type=command-failed exit=1。202 份里 25 份倒推不出（开工之后别的会话在工作区改过，diff 里没有基底）：`scripts/crash/crash-point-replay-selftest.sh`、`scripts/crash/crash-point-replay.sh`、`scripts/gate/apply_writer_patch_plan.py`、`scripts/gate/apply-writer-patch.py`、`scripts/gate/comment-history-check.py`、`scripts/gate/lib_lag_split_family.py`、`scripts/gate/lib_verus_trait_impl_lag.py`、`scripts/gate/merge-and-verify-selftest-lib.sh`、`scripts/gate/merge-and-verify.sh`、`scripts/gate/README.md`、`scripts/gate/verus_proof_check_proofs.py`、`scripts/gate/verus-proof-check.py`、`scripts/hosts/multi-host-run.sh`、`scripts/mutation/broad_mutation_selftest_a.py`、`scripts/mutation/broad_mutation_selftest_b.py`、`scripts/mutation/crates_mutation_rows_commands.py`、`scripts/mutation/tests/test_audit_regressions.py`、`task-server/README.en.md`、`task-server/README.ja.md`、`task-server/README.md`、`task-server/task_server_lib_constants.py`、`task-server/task_server_lib.py`、`task-server/task_server_machines.py`、`task-server/task_server_scheduler_building.py`、`task-server/task_server_selftest_lifecycle.py。打中引到的行都不在这几份里（攻方腿报告点名的四处与正推腿那一条线索主 agent 都在此刻的树上复跑坐实）。
- `python3 scripts/agents/cite-check.py research/prompts/split-governance-scripts-services-r1-opus-output.md research/prompts/split-governance-scripts-services-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/split-governance-scripts-services-r1-tree --background research/prompts/_split-governance-scripts-services-r1-body.md`：「✓ 核了 102 处引文、15 处裸行号，对上 80 处，没判 37 处（其中裸行号没有引文可比 12 处；逐处列在上面）」。没判的落在倒推不出的几份、树里没有的 `records/` 与 `scripts/hosts/queue-health.py`；判决引的几处主 agent 在此刻主工作区里现读：`scripts/hosts/queue-health.py:1136-1141`、`scripts/gate/admission_unit_key.py:1562-1563`、`scripts/mutation/crates_mutation_rows_plan.py:552`、`scripts/mutation/crates_mutation_rows_units.py:1514-1517`、`scripts/gate/verus_proof_check_selftest_fixtures.py:751`。

主 agent 复跑：

- 两臂探针（从攻方腿草稿拷到 `/home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/probe/`，sha256 与腿的相同；`python3 …/probe/probe_entry_patch.py --repo /home/fy5090/code/singlefs --work …/probe-work`）：结局 type=succeeded exit=0，六行读数都落在攻方腿跑前写死的「坐实」那一栏：「PROBE_RUSTFILE arm=head files_loaded=1 patched_calls=1」「PROBE_RUSTFILE arm=split files_loaded=1 patched_calls=0」「PROBE_RUSTWORKSPACE arm=head rust_units=1 counted_loads=1」「PROBE_RUSTWORKSPACE arm=split rust_units=1 counted_loads=0」「PROBE_ROW_LINE arm=head entry=lib_function_units.py occurrences=2」「PROBE_ROW_LINE arm=split entry=lib_function_units.py occurrences=0」「PROBE_MEMO arm=split via_entry_hits=1 via_rust_closure_module_hits=0 lib_function_units_rust_closure_sees_memo=False lib_function_units_script_closure_sees_memo=False lib_function_units_closure_keys_sees_memo=False」。
- 打中二那一格的单格自证（`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/crates-mutation-rows.py --selftest --cells selftest_static_check_parallel_cell`）：结局 type=command-failed exit=1，核不动——每一趟先算判法单元，这一步拒算（「inline-script-not-literal scripts/gate/lib_function_units_selftest_real_repo.py:241；spec-path-unfolded …；subprocess-command-unfolded scripts/mutation/crates_mutation_rows_runs.py:240 等 9 处」），一格都没往下跑。拒算归 mainagent(C) 在修的算键闭包那一件；打中二由探针那一行坐实。
- `task-server/task submit -- python3 scripts/hosts/queue-health.py --selftest`：结局 type=command-failed exit=1，「✗ 自证 attempt-forms-match-sources：尝试单元、尝试标签与 scope 的写法在源码里要找得到：{"singlefs-task-{self.store.tag}-{task['id']}-{number}.service": False, "task-{self.store.tag}-{task['id']}-{number}": False, 'unit="singlefs-memory-cap-': True}」「✗ queue-health.py 自证 40 格里 1 格不对」。
- 正推腿的动态核对 `dynamic_names.py`（在它草稿目录交）：结局 type=succeeded exit=0，「== task_server_lib: 拆前顶层名字 554 个，入口 vars 里缺 0 个 []；同名可变对象在别的拆出模块里是另一份的 0 个 []」，`task_server` 331 个、`task_server_selftest` 1020 个同样缺 0、分叉 0。
- 正推腿第 5、6 问要的读数主 agent 在主工作区交过：hook-swap 换上后自证「✓ hook-swap 自检通过（查了 119 种）」，七条开关红格与文件头点名逐条相同；`agent-watch.py --selftest` 换上时的验证通过。

## 二、判决

| # | 打中 | 判 | 依据 |
|---|---|---|---|
| 1 | `admission_unit_key.py:1562-1563` 在入口上换 `library.data_file_unit`、`data_directory_unit`，拆出的 `_rust_closure`、`_script_closure`、`_closure_keys` 三份模块读不到；自证 `admission_selftest_support.py:614` 只核 `> 0` 看不出 | 坐实而采 | 探针 PROBE_MEMO 一行 |
| 2 | `crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，`RustWorkspace` 读自己模块里的 `RustFile`，并行切好的文件拆后一份都没用上 | 坐实而采 | 探针 PROBE_RUSTFILE 两行（拆前 1、拆后 0） |
| 3 | `crates_mutation_rows_units.py:1514-1517` 断言入口原文里有 `RustFile.row` 那一行，那一行已挪进拆出的模块 | 坐实而采 | 探针 PROBE_ROW_LINE 两行（拆前 2、拆后 0） |
| 4 | `verus_proof_check_selftest_fixtures.py:751` 的计数子类换在入口上，库内模块自己建的工作区计不到 | 推翻、不采：探针 PROBE_RUSTWORKSPACE 两行（拆前 1、拆后 0）量的是库内模块自己建工作区那条路；verus 自证走 `admission_unit_key.py:359` 的 `FunctionUnitIndex.workspace()`，（第二轮判决第 8 条改正：`admission_unit_key.py:359` 是 `KeyContext.workspace`，verus 自证经 `scripts/crash/lib_verdict_library.py:209` 的 `FunctionUnitIndex`），经入口模块取 `library.RustWorkspace`，入口上换的计数子类够得着 | 主 agent 交 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/verus-proof-check.py --selftest`，结局 type=command-failed exit=1，那一格「Rust 工作区载入 1 次；同一份库再判一趟：…载入 1 次」，红在证明项的键全拒算（算键闭包那一件），不在计数 |
| 5 | `scripts/hosts/queue-health.py:1137` 起的循环只读入口 `task_server.py` 原文找尝试单元与尝试标签两种写法 | 坐实而采 | queue-health 自证 40 格里红这一格 |
| 6 | 看门狗新加那一格没有挂弄坏开关 | 第二轮判决第 9 条翻成采（`.claude/rules/show-me-test-by-review.md` 第 7 行要每处会判红的分支配开关或变异）；原判不采：那一格是拆分族抽法的回归护栏，红的条件本身是一份造出来的拆分族输入，不靠开关证红；补登的两个起头由头一格盖着 | 正推腿第 6 问原话 |
| 7 | 第 2 问 task-server 五个混入类、第 4 问子进程与路径、两个 Rust 服务的 `control` 读法与 `#[path]`、正推腿第 5 到 7 问 | 没打中，这一题的这几格停 | 两条腿都判成立且没给反例；动态核对缺 0、分叉 0（它分辨不出「换在入口属性上」那一形状，那一形状靠 `task-server/task_server_selftest_memory.py:40` 的 `patched_server_name` 撑着，第二轮判决第 10 条） |
| 8 | 服务没重编重起会让哪一格与源码不符 | 推不出、核不动：攻方腿没找到比较源码与在跑二进制的格；这件归两个服务的会话重编时核 | 攻方腿第 4 问原话 |

## 三、改法

1. 四处打中改成在被测代码真读的那个绑定上换或读：替身换在拆出模块的全局名上（或改成调用方经入口取、拆出模块不留自己那一份），读源码文本的改读拆分族（入口加它 import 的同目录模块）。照 `scripts/gate/hook_swap_selftest.py` 那一处修法：换完核替身真被调到，没被调到就记「样本没造成」当场红。四份分别是 `scripts/gate/admission_unit_key.py`、`scripts/mutation/crates_mutation_rows_plan.py`、`scripts/mutation/crates_mutation_rows_units.py`、`scripts/gate/verus_proof_check_selftest_fixtures.py`；`admission_selftest_support.py:614` 改成核恰好一次。去向逐份定：`crates_mutation_rows_plan.py:552` 与 `crates_mutation_rows_units.py:1514-1517` mainagent(B) 接（后一处随它变异键那一件删掉判法单元指纹、那段断言没了对象，前一处并进它下一批）；`admission_unit_key.py:1562-1563` 连同 `admission_selftest_support.py:614` 与 mainagent(C) 在跑的算键库第二轮撞同一份文件，等它换上再以换上后为基线派；`verus_proof_check_selftest_fixtures.py:751` 随判决第 4 条推翻不改（派出去的那一份加固没换上）。零轮。
2. `scripts/hosts/queue-health.py` 的 `attempt-forms-match-sources` 一格改读 task-server 的拆分族（入口加它 import 的同目录模块），自证加一格：写法只在拆出模块里时照旧找得到。零轮。
3. 认族的写法与另一题判决第三条改法第 4 条那一份并成一份（mainagent(C) 的 `scripts/gate/lib_lag_split_family.py` 公开函数合进来之后改调它）。
4. 改完照「代码轮与治理文件轮怎么收」再攻一轮，攻这几处修补；第二轮派辩方腿复核这一份判决。

## 交用户的

无。

## 这一轮被判过的文件

- gpu-verification-server/src/library_pending_pool/claims_across_the_families.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/src/library_pending_pool/facts_verifier_pending_rows.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/src/library_pending_pool/image_judgement_pending_rows.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/src/library_pending_pool/packed_tables_pending_rows.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/src/library_pending_pool.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/cards_failing_in_the_serving_loop.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/facts_verifier_family.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/image_judgement_claims_and_write_back.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/image_judgement_serving_loop_and_claim_release.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/main.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/packed_tables_family.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/protocol/connection_level_frames.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/protocol/request_decoding.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/protocol/request_encoding.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/protocol/response_decoding.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/protocol/response_encoding.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/protocol.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/reliable_client/claims.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/reliable_client/reads_and_scans.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/reliable_client/reconnection_and_resend.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/reliable_client/response_handling.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/reliable_client.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/reliable_client/segmented_values.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/reliable_client/writes_and_acknowledgements.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/claim_lease_after_the_answer.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/claim_response_and_oversize_frame_limits.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/connect_bounded_by_reconnect_total.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/journal_high_water_and_lock.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/pending_budgets_of_responses.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/storage_failure_replay.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/unread_responses_and_incomplete_frames.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/selftest_hardening_cells/write_ahead_log_sync_failure.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/server/admission_budgets.rs：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- prune-server/src/server/connection_serving.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/server/frame_reading.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/server/request_execution.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/server.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- prune-server/src/server/service_lifecycle.rs：Rust 服务拆出的子模块：攻方腿第 4 问核 `gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 拆后指得对
- scripts/agents/agent_watch_alerts.py：看门狗拆分族（这一轮修补的一件）：正推腿第 5、6 问核补登两个起头与新加一格
- scripts/agents/agent-watch.py：看门狗拆分族（这一轮修补的一件）：正推腿第 5、6 问核补登两个起头与新加一格
- scripts/agents/agent_watch_repository.py：看门狗拆分族（这一轮修补的一件）：正推腿第 5、6 问核补登两个起头与新加一格
- scripts/agents/agent_watch_transcripts.py：看门狗拆分族（这一轮修补的一件）：正推腿第 5、6 问核补登两个起头与新加一格
- scripts/crash/crash-amplification-item.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/crash/crash-amplification-two-hosts.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/crash/crash-case-check.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/crash/crash-flow-coverage.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/crash/crash-point-replay-selftest.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/crash/crash-point-replay.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/crash/lib_crash_point_replay_items.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_driver.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_selftest.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-admission.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-claims.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-dispatch.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-finish.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-inputs-and-build.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-row-library.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-selftest.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/experiment-apparatus/replay-table-and-dispatch-limits.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/admission_base.py：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- scripts/gate/admission_commands.py：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- scripts/gate/admission_crash_case.py：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- scripts/gate/admission.py：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- scripts/gate/admission_selftest_cells.py：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- scripts/gate/admission_selftest_support.py：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- scripts/gate/admission_unit_key.py：准入模块拆分族：攻方腿打中一，`admission_unit_key.py:1562-1563` 在入口上换 `data_file_unit`、`data_directory_unit`，拆出的三份闭包模块读不到
- scripts/gate/apply_writer_patch_plan.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/apply-writer-patch.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/comment-history-check.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/crates-split-pure-move.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/hook_swap_base.py：换上工具拆分族（这一轮修补的一件）：正推腿第 5、6 问核替身换在 `hook_swap_swap` 上、两格另核 `.taking` 机器一段
- scripts/gate/hook_swap_callers.py：换上工具拆分族（这一轮修补的一件）：正推腿第 5、6 问核替身换在 `hook_swap_swap` 上、两格另核 `.taking` 机器一段
- scripts/gate/hook-swap.py：换上工具拆分族（这一轮修补的一件）：正推腿第 5、6 问核替身换在 `hook_swap_swap` 上、两格另核 `.taking` 机器一段
- scripts/gate/hook_swap_selftest_fixtures.py：换上工具拆分族（这一轮修补的一件）：正推腿第 5、6 问核替身换在 `hook_swap_swap` 上、两格另核 `.taking` 机器一段
- scripts/gate/hook_swap_selftest.py：换上工具拆分族（这一轮修补的一件）：正推腿第 5、6 问核替身换在 `hook_swap_swap` 上、两格另核 `.taking` 机器一段
- scripts/gate/hook_swap_swap.py：换上工具拆分族（这一轮修补的一件）：正推腿第 5、6 问核替身换在 `hook_swap_swap` 上、两格另核 `.taking` 机器一段
- scripts/gate/lib_crates_split_cli.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_crates_split_common.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_crates_split_layout.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_crates_split_renames.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_crates_split_selftest.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_crates_split_tables.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_crates_split_units.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_feature_gated_targets.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_function_units_base.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_closure_keys.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_lexer.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_python_data_reads.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_python_facts.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_python_launches.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_python_module.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_registry.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_rust_closure.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_rust_file.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_rust_verus.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_rust_workspace.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_script_closure.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_closure_rules.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_fixtures_closure_rules.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_fixtures.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_fixtures_real_repo.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_fixture_trees.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_program_registries.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_real_repo.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_run.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_selftest_rust_cells.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_shell_index.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_shell_text.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_unit_graph.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_function_units_wgsl.py：算键库拆分族：攻方腿第 1 至 3 问坐实四处打中的被读方（拆出模块按自己的全局名读 RustFile、data_file_unit、工作区，入口上换的替身够不着）
- scripts/gate/lib_lag_split_family.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_merge_verify_formal_tables.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_rust_test_module_files.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_test_target_sources.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/lib_verus_trait_impl_lag.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/merge-and-verify-items-lib.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/merge-and-verify-selftest-lib.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/merge-and-verify.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/merge-and-verify-snapshot-lib.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/merge-and-verify-steps-lib.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/README.md：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/gate/verus_proof_check_base.py：Verus 证明检查拆分族：攻方腿第 3 问判 `verus_proof_check_selftest_fixtures.py:751` 计数子类换在入口上计不到，主 agent 复跑此刻自证读出「Rust 工作区载入 1 次」推翻了这一条（判决第 4 条）
- scripts/gate/verus_proof_check_listing.py：Verus 证明检查拆分族：攻方腿第 3 问判 `verus_proof_check_selftest_fixtures.py:751` 计数子类换在入口上计不到，主 agent 复跑此刻自证读出「Rust 工作区载入 1 次」推翻了这一条（判决第 4 条）
- scripts/gate/verus_proof_check_proofs.py：Verus 证明检查拆分族：攻方腿第 3 问判 `verus_proof_check_selftest_fixtures.py:751` 计数子类换在入口上计不到，主 agent 复跑此刻自证读出「Rust 工作区载入 1 次」推翻了这一条（判决第 4 条）
- scripts/gate/verus-proof-check.py：Verus 证明检查拆分族：攻方腿第 3 问判 `verus_proof_check_selftest_fixtures.py:751` 计数子类换在入口上计不到，主 agent 复跑此刻自证读出「Rust 工作区载入 1 次」推翻了这一条（判决第 4 条）
- scripts/gate/verus_proof_check_selftest_fixtures.py：Verus 证明检查拆分族：攻方腿第 3 问判 `verus_proof_check_selftest_fixtures.py:751` 计数子类换在入口上计不到，主 agent 复跑此刻自证读出「Rust 工作区载入 1 次」推翻了这一条（判决第 4 条）
- scripts/gate/verus_proof_check_selftest.py：Verus 证明检查拆分族：攻方腿第 3 问判 `verus_proof_check_selftest_fixtures.py:751` 计数子类换在入口上计不到，主 agent 复跑此刻自证读出「Rust 工作区载入 1 次」推翻了这一条（判决第 4 条）
- scripts/hosts/multi-host-run-selftest.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/hosts/multi-host-run.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/hosts/run-with-memory-cap-selftest.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/hosts/run-with-memory-cap.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_base.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_gpu.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_judging.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_library.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad-mutation.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_runner.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_run.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_selftest_a.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/broad_mutation_selftest_b.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/crates_mutation_rows_baseline.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_base.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_commands.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_driver.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_judging.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_memory.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_plan.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_pool.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates-mutation-rows.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_runs.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_stage.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_static.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_support.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_targets.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/crates_mutation_rows_units.py：crates 变异判法拆分族：攻方腿打中二（`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`，拆后没用上）与打中三（`crates_mutation_rows_units.py:1514-1517` 读入口原文找已挪走的一行）
- scripts/mutation/mutate.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/prove_red_core.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/prove_red_selftest.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/prove-red.sh：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/README.md：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/mutation/tests/test_audit_regressions.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/prune-server/prune_server_client_selftest_cells.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/prune-server/prune_server_client_selftest.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/prune-server/prune_server_client_selftest_service.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- scripts/prune-server/prune_server_client_selftest_sweep.py：拆过的脚本：攻方腿第 1 至 4 问比拆前拆后的判法、子进程起法、`__file__` 与 `sys.path`
- task-server/README.en.md：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/README.ja.md：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/README.md：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_constants.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_lib_commands.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_lib_configuration.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_lib_constants.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_lib.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_lib_readings.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_lib_scopes.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_machines.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_scheduler_admission.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_scheduler_building.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_scheduler_lifecycle.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_scheduler_notifying.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_scheduler_starting.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_admission.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_client.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_fakes.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_lifecycle.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_limits.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_memory.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_notifying.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_queueing.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_refusals.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_shards.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_shared_roots.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_stalls.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_selftest_world.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/task_server_store.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题
- task-server/tests/test_cleanup_protocol.py：统一任务服务拆分族：正推腿第 7 问核三个入口再导出拆前全部顶层名字（554、331、1020 个，动态 import 复核缺 0、分叉 0），攻方腿第 2 问核五个混入类没有类属性与 super() 次序问题

## 回看决策

不涉及决策：这一轮判的是治理文件按 2000 行拆分之后的行为与拆分带出的缺陷，`.claude/kb/decisions/` 下没有一条分项因此改动。
