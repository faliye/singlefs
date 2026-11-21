# split-closure-governance-r1：判决（2026-11-21）

正文 `research/prompts/_split-closure-governance-r1-body.md`，开工快照 `research/prompts/split-closure-governance-r1-snapshot/`。治理文件轮第一轮，前面没有判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（three-way-forward） | `research/prompts/split-closure-governance-r1-sonnet-output.md` | 格 1「不成立」（三处打中）、格 3「不成立」（一处打中，另一处在被判文件里但不是这批引入的）、格 4「成立」 |
| 攻方（three-way-attack） | `research/prompts/split-closure-governance-r1-opus-output.md` | 格 2：`admission_run` 那一行「按字面算第三种排除，成立」，但「没有让任何单元出键，改 admission.py 也不会错复用」；格 5 打中五行 `STAGE_CELLS_JOBS` 与快照那一版 repository-copy-closure 表第 44 行，旁支「`keyed_family_growth` 按集合比、不比行数」；「已经量的」四句核不全、标题一句不成立 |

核查命令与它们判了什么：

- `task-server/task submit --cwd <草稿目录> -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py split-closure-governance-r1 --out <草稿目录>/split-closure-governance-r1-tree --repository /home/fy5090/code/singlefs`：type=command-failed exit=1，「清单点名 44 份，倒推出 41 份、sha256 对上 41 份；倒推不出 3 份」。倒推不出的是 `.claude/gate.d/comment-history-lag.tsv`、`.claude/gate.d/repository-copy-closure-exempt.tsv`、`.claude/memory-bounded-parallelism.tsv`（快照之后被别的会话改过，diff 里没有基底）。腿引到这三份的行逐处人工核：攻方引快照那一版 repository-copy-closure 表第 44 行，原文由同一日门禁日志那一行坐实（「第 44 行的锚串「SHARED_INPUTS = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain"」在 …code-tooling-unit-verdict-and-entry-cells.sh 里一处被判行都不命中」）；正推引滞后表的几行由下面 comment-history 格单跑的结局行坐实；`memory-bounded-parallelism.tsv` 两条腿都没引行号。
- `python3 scripts/agents/cite-check.py <两份腿报告> --root <草稿目录>/split-closure-governance-r1-tree --root /home/fy5090/code/singlefs --background research/prompts/_split-closure-governance-r1-background.md`：「✓ 核了 70 处引文、50 处裸行号，对上 70 处，没判 50 处（其中裸行号没有引文可比 39 处；逐处列在上面）」。没判的 11 处有引文、路径不在快照树里（`records/` 与 `.claude/rules/` 下的几份），主 agent 逐处在仓里现查对得上。

判决引的读数由主 agent 把两腿的模型目录拷进工作根下草稿目录 `split-closure-governance-r1-verify/` 复跑：

- 正推样本与探针（`bash <草稿目录>/sonnet-model/tools/rerun-from-a-clean-work-directory.sh …`）：type=succeeded exit=0，「name=family_probe cases=12 different=0」「name=closure_probe samples=11」；11 个样本的 exit 与腿写的 expect 逐个对上。p3（先赋字面空串、再 `read -d ''` 从 `"$1"` 读）exit=0 不拒算；p5（同一份文件里别的类的 `run(self, command)` 起折不出的命令）exit=0 不拒算；p6 三把键 a=d41418716e26eb46、b=533c497b51d5949b、helper-changed=d41418716e26eb46。
- 正推模板自证（`bash <草稿目录>/sonnet-model/tools/rerun.sh --selftest`）：type=command-failed exit=1，「自检：拷进与仓无关的临时目录、传参应当跑通…实际退 0」。腿把拷来的 `scripts/agents/three-way-rerun-template.sh` 里 `rerun_steps` 换成了自己的探针、不再拷冻结副本，模板那一格在拷贝上判红；读数在上一条，不受它影响。
- 门禁 doc-text 的 comment-history 格单跑（`bash .claude/gate.d/doc-text.sh --check comment-history`）：type=command-failed exit=1，「滞后表比 gate 基准 4319aa2b035b2d8238d64aeb68b3ceb7674cc723 那一版涨了 13 处」，其中「拆分族 .claude/gate.d/crash-point-replay.sh 里 4 行登记之和是 50 行，基准里那一行是 43 行」「拆分族 .claude/gate.d/doc-process-records.sh 里 3 行登记之和是 40 行，基准里那一行是 39 行」与 5 行「基准里没有这一行」；另「8 份文件的注释里写了历史，共 12 行」。
- 攻方探针（`bash <草稿目录>/opus-model/tools/rerun-from-a-clean-work-directory.sh …`）：type=succeeded exit=0，「name=stage_jobs_summary cases=9 disagree=8 leftover_groups=0」「name=family_growth_case case=new-member-copies-anchors base_rows=2 current_rows=4 keyed_grown=0 counted_grown=2」；admission 一族 `library_files_fetch`、`command_library_files`、`command_verdict_plan` 三行 present=yes；`def admission_run` 那一行 present=no 是探针按 def 名找，`admission_run` 写在 `.claude/gate.d/lib/code-tooling-shared-libraries.sh:727`，落在闭包里的 shell-variable 单元 `UNIT_VERDICT_ROUNDS_LIBRARY`（第 668–1230 行，闭包输出第 461 行）里，整段进键。

## 二、判决

1. **拆分族排除表比法不比行数：打中，两条腿都中，量过。** `scripts/gate/lib_lag_split_family.py:176` 的 `keyed_family_growth` 只判（族入口, 锚串）在不在基准集合里；同族新文件照抄已有锚串，攻方探针「keyed_grown=0 counted_grown=2」。三张排除表经它判「只缩不涨」，用户定案「之后照旧只缩」在拆分族上守不住。
2. **`UNKEYED_PROGRAM_WORDS` 按（文件, 函数短名, 词）认、丢了类名：打中，量过。** `TripLauncher.run` 那一行在同一份文件里也放过别的类的 `run(self, command)`，样本 p5 exit=0 不拒算。
3. **先赋字面值再 `read -d ''` 从折不出的文件读：打中，量过。** `assignment_sites` 只看到字面值，库正文折成空串、不拒算，样本 p3 exit=0；`scripts/gate/lib_function_units.py` 文件头那句「变量照旧折不出」说过了头。
4. **入口单元自己是测试服务代码时经 entry 行进键：打中，量过。** p6 改入口那一份键变（a≠b），改非入口那一份键不变（a=helper-changed）。D13（验证路线） 已定项 17「三类代码」表测试服务代码那一行写的是不进任何键，入口也在内。
5. **五行 `STAGE_CELLS_JOBS` 登进排除表：打中，量过。** 它是真定并行度的写法，不在用户定案点名的三类里；`stage_cells_default_jobs` 不认 `SINGLEFS_THREAD_CAP`、登记写坏或不在都悄悄退成 1 路，攻方探针九格里八格与 `heavy_thread_default` 不同（「disagree=8」）。照定案「真该改的用法照改代码」改代码、删这五行。`stage-cells.sh` 那一行 `${STAGE_CELLS_JOBS:-1}` 属 `${X:-1}` 那一类，照留。
6. **快照那一版 repository-copy-closure 表第 44 行 `SHARED_INPUTS` 死行：打中，开工后已被删。** 此刻表里没有这一行（攻方现查、同一日门禁日志），不再改。
7. **`admission_run` 那一行：效果上没打中，理由文字打中。** admission 一族经别的边整族在闭包里（探针三行 present=yes），这一行不让任何单元出键、不会错复用。登记理由写「与 NOT_JUDGING_FILES 同一个角色」，字面上把文件级排除的理由用到第三份文件上，与 `.claude/rules/verification.md`「一个门禁：两种跑法」只认两种排除那一句对不上：改理由文字，写它真正的作用（起的那份 admission.py 经模块级 import 与 shell 格直接起的几条已在闭包里，这一处只是命令折不出）。`command=` 参数那条潜在绕过今天没有调用方，记进下一轮攻击面。
8. **`TripLauncher.run` 与 `TAIL_READ_SCOPES` 两行的去向：不成立（攻方）。** 被复跑的自证照各条判定的入口进键；`TAIL_READ_SCOPES` 两行读的是自证自己造的输入与调用方交的根下几份。第 2 条改的是登记的认法，不改这两行的去留。
9. **复用键里的 `external` 行随主机：不是这一批引入的。** HEAD 里就有；T4（去工具链与依赖版本行那一份，与 Rust 一侧单元图补丁同一次合入）删 `--version` 那一行，这一轮不改。
10. **拆后旧路径、旧开关：成立（正推格 4）。** 滞后表 136 行路径都在盘上，没有为拆前留的读法。
11. **滞后表与注释写历史：打中（主 agent 复跑坐实）。** 滞后表比基准涨 13 处，归这一批（滞后表是这一批改成拆后路径的）；注释写历史的 12 行里 `scripts/gate/lib_function_units.py:219`「用户定」与 `scripts/gate/lib_function_units_selftest_program_registries.py:907`「改前」两处是这一批的，其余已转给各自的会话并已改。
12. **「已经量的」：攻方核出的四句照实收。** 正文是冻结证据，不改；标题「号见调度记录」不成立（任务号按规矩不进记录），X、Y 单跑各多红一格、Z 没有新开关、key 与 closure 三条只记了 P4 之前那一趟、排除表涨行数对快照是 10、4、9（heavy 表 3 行开工前已删）。下一轮正文照这几条写准。
13. **另一处算键库缺陷（不在腿的格里，别的会话报、主 agent 核实）：** `scripts/gate/lib_function_units_python_data_reads.py` 的 `path_tails` 对 `os.path.join(<折不出的前缀>, <字面名>)` 交回不带前导 `/` 的尾巴，第 435 行照字符串后缀收，`remove_stub.py` 被当成 `b.py` 收进 e171 复跑的键。并进同一批改。

## 三、改法

都在治理文件里改，不碰 `crates/`；照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」，改完再攻一轮，第二轮派辩方腿复核这一份判决。下面标「零轮」的是腿提的或主 agent 推的改法、没被攻过；它们不写进产品代码，不做 A/B，改完由第二轮攻。

1. `keyed_family_growth` 照 `family_growth` 按（族入口, 锚串）比计数之和，超了判涨；此刻表里照抄锚串的几行（admission 两行 `"nproc"`、crates 变异两行 `os.cpu_count() or 4`）合并或照实判涨。加样本与开关（判第 1 条）。零轮。
2. `UNKEYED_PROGRAM_WORDS` 与同类按（文件, 函数）登记的表认限定名（`类.方法`），登记行改成限定名；加自证格与开关（判第 2 条）。零轮。
3. `assignment_sites` 记 `read` 改写的赋值为折不出，字面值在前的照拒算；改正文件头那句（判第 3 条）。零轮。
4. `entry_key_lines` 看 `is_test_service_code`，入口单元落在测试服务代码里的不出 entry 行指纹；加自证格（判第 4 条）。零轮。
5. `stage_cells_default_jobs` 改成 source 仓根 `scripts/hosts/heavy-threads.sh` 取 `heavy_thread_default`、取不出整道判红、不吞标准错误（照 `cell_date_once` 的写法），删 `.claude/heavy-thread-exempt` 那五行；加样本与开关（判第 5 条）。零轮。
6. `admission_run` 那一行改理由文字（判第 7 条）。
7. `path_tails` 对 `os.path.join` / `Path(…)` / `/` 拼出的尾巴保留前导 `/`；加自证格与开关（判第 13 条）。零轮。
8. 滞后表涨的 13 处查清每一处多出的命中是哪一行、谁加的：这一批的改写成不写历史，别的会话的转给它；不靠往滞后表加行（判第 11 条）。`lib_function_units.py:219` 与 `lib_function_units_selftest_program_registries.py:907` 两处改成只写现状。

## 交用户的

| 事 | 依据 | 要定什么 |
|---|---|---|
| heavy 表第 71 行 crates `temporary_servers` | 攻方报告「不成立的几行」那张表第 2 行：取的是登记值、是合规写法、判法没认出，不在定案点名的三类里 | 定案「按基准登一次表」罩不罩「合规写法、判法没认出」这一类；不罩就改判法认出它、删这一行 |

## 这一轮被判过的文件

- .claude/gate.d/code-tooling.sh：判文件头开关表与拆后各格的认法，格 4 成立
- .claude/gate.d/comment-history-lag.tsv：判拆后路径没有旧路径残留（格 4 成立），复跑坐实比基准涨 13 处（第 11 条打中）
- .claude/gate.d/doc-text.sh：判 date-once 新开关与样本，开关单跑只红点名那一个用例
- .claude/gate.d/fixtures/doc-text.sh/date-once-thread-default-red/expect：判样本期望与判法的红句逐字对上
- .claude/gate.d/fixtures/doc-text.sh/date-once-thread-default-red/.gate-cells：判样本只点 date-once 一格
- .claude/gate.d/fixtures/doc-text.sh/date-once-thread-default-red/.sample-thread-cap-invalid：判样本造的线程登记取不出
- .claude/gate.d/fixtures/doc-text.sh/date-once-thread-default-red/setup.sh：判样本的造法
- .claude/gate.d/hardcoded-quota-exempt.tsv：判新登四行都是自证假值或开关坏命令文字，成立
- .claude/gate.d/lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：判拆后认文件的写法，格 4 成立
- .claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh：判 change-range 格拆后认文件，格 4 成立
- .claude/gate.d/lib/code-tooling-source-scan-cells.sh：判扫描格拆后认文件，heavy-thread-defaults 在五行 `STAGE_CELLS_JOBS` 上判红判对了（第 5 条）
- .claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh：判 quotepath 与三处 change-range 豁免标记，格 4 成立
- .claude/gate.d/lib/doc-text-notation-and-translation-cells.sh：判 `cell_date_once` 改取登记值、不再在排除表里（攻方格 5 成立）
- .claude/gate.d/repository-copy-closure-exempt.tsv：判新登九行，快照那一版第 44 行死行打中（第 6 条），其余成立
- .claude/heavy-thread-exempt：判新登行，五行 `STAGE_CELLS_JOBS` 打中（第 5 条），第 71 行交用户
- .claude/memory-bounded-parallelism.tsv：判入口改回 code-tooling.sh 与 e171 一行，格 4 成立
- scripts/gate/comment-history-check.py：判滞后表按拆分族比之和，格 4 成立
- scripts/gate/lib_function_units_base.py：判按后缀收的数据目录读法，与第 13 条同一类收宽
- scripts/gate/lib_function_units_closure_keys.py：判 entry 行与测试服务代码（第 4 条打中）、external 行随主机（第 9 条，HEAD 里就有）
- scripts/gate/lib_function_units_lexer.py：判拆后同族解析，格 4 成立
- scripts/gate/lib_function_units.py：判文件头折法说明，「变量照旧折不出」那句过头（第 3 条），第 219 行注释写历史（第 11 条）
- scripts/gate/lib_function_units_python_data_reads.py：判 `path_tails` 丢路径分隔符（第 13 条打中）
- scripts/gate/lib_function_units_python_facts.py：判 exec 进模块对象的事实收法，格 1 成立
- scripts/gate/lib_function_units_python_launches.py：判 exec 进模块对象与登记认法，登记丢类名（第 2 条打中）
- scripts/gate/lib_function_units_python_module.py：判库正文里模块级函数的参数照载入方折，格 1 成立
- scripts/gate/lib_function_units_registry.py：判 `admission_run`（第 7 条）、`TripLauncher.run` 与 `TAIL_READ_SCOPES` 两行（第 8 条）、测试服务代码登记读法（格 3 成立）
- scripts/gate/lib_function_units_script_closure.py：判载入方与模块对象的连法，模块对象交出去与出不去两形格 1 成立
- scripts/gate/lib_function_units_selftest_fixtures_closure_rules.py：判夹具碰不到「先赋字面再 read」那一形（第 3 条）
- scripts/gate/lib_function_units_selftest_fixtures.py：判 `shell-heredoc-captures-unfolded` 那一格夹具改成函数体一条 cat
- scripts/gate/lib_function_units_selftest_fixtures_real_repo.py：判真仓格与 exec 进模块对象那组夹具
- scripts/gate/lib_function_units_selftest_fixture_trees.py：判测试服务代码夹具树，碰不到入口自己是服务代码那一形（第 4 条）
- scripts/gate/lib_function_units_selftest_program_registries.py：判登记表真仓格，第 907 行注释写历史（第 11 条）
- scripts/gate/lib_function_units_selftest_real_repo.py：判真仓格的读法
- scripts/gate/lib_function_units_selftest_run.py：判开关表与 BREAK_CELLS 对得上
- scripts/gate/lib_function_units_selftest_rust_cells.py：判 Rust 夹具格，repository-copy-closure 没登记的两处在它第 1026、1027 行
- scripts/gate/lib_function_units_shell_index.py：判 `$(函数)` 与 `read -d` 折法，赋值点只看字面（第 3 条打中）
- scripts/gate/lib_lag_split_family.py：判拆分族认法，`keyed_family_growth` 不比行数（第 1 条打中）
- scripts/gate/lib_test_target_sources.py：判 `binary_target_source_files` 拆后认目标，格 4 成立
- scripts/gate/lib_verus_trait_impl_lag.py：判照拆分族比 trait impl 滞后，格 4 成立
- scripts/gate/test-service-code-paths.tsv：判 25 行与 D13（验证路线） 已定项 17 那一行逐项对上（格 3 成立），入口那一形见第 4 条
- scripts/gate/verus_proof_check_proofs.py：判拆后同族的线程取法与 verus 自证，格 4 成立
- scripts/gate/verus-proof-check.py：判照拆分族判滞后，格 4 成立
- scripts/gate/verus_proof_check_selftest.py：判自证 196 格与 122 个开关在拆后仍全对

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-21 不受影响：这一轮判的是测试服务代码不进键在算键库里落得全不全，打中的入口那一形是落实不全，定案本身不改 |
