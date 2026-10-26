# m3-close-gov-b17-r2 判决（治理轮第二轮，2026-10-24）

被判对象：第一轮判决 `research/prompts/m3-close-gov-b17-r1-main-verification.md` 之后打进主工作区的四份修补，正文 `research/prompts/_m3-close-gov-b17-r2-body.md`，开工快照 `research/prompts/m3-close-gov-b17-r2-snapshot/`（`governance-fix-1.patch`、`hooks.patch`、`mutation-judging-files.patch`、`broad-gpu-import.patch` 与 `judged-sha256.txt` 14 份）。这一轮判过的文件：`.claude/rules/implementation-workflow.md`、`.claude/rules/verification.md`、`.claude/rules/three-way-inference.md`、`.claude/skills/crash-test/commit-stages.md`、`.claude/agent-common.md`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`scripts/mutation/lib_mutation_units.py`、`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/broad-mutation.py`。腿：辩方（Sonnet）`research/prompts/m3-close-gov-b17-r2-sonnet-output.md`（33d9f3c2…），攻方（Opus）`research/prompts/m3-close-gov-b17-r2-opus-output.md`（70783efe…）。

## 一、核查

1. `python3 scripts/agents/snapshot-tree.py m3-close-gov-b17-r2 --out ~/.local/share/singlefs/work/m3-close-gov-b17-r2-verify/r2-tree`：「清单点名 14 份，倒推出 13 份、sha256 对上 13 份；倒推不出 1 份、对不上 0 份」。倒推不出的是 `scripts/mutation/crates-mutation-rows.py`：开工后另一会话又换过一版（带 gpu 的 cargo 逐格经 `gpu-quiesce-run.sh`），基底不在 diff 里。腿引它的三处人工核：`JUDGING_ENTRIES` 现文在第 510 行（攻方写 514–516，是换版前的行号，内容同），`if executed_set_is_current(keys, coverage):` 现文第 1378 行，自证新格那一句现文在（攻方写 6887）。
2. `python3 scripts/agents/cite-check.py <两份腿报告> --root <倒推树> --background research/prompts/_m3-close-gov-b17-r2-background.md`：「✓ 核了 53 处引文、55 处裸行号，对上 27 处，没判 81 处（其中裸行号没有引文可比 28 处）」，没有对不上的。没判的多是倒推树之外的文件与写短名的代码引文。判决要用的几处主 agent 在主工作区现查：
   - `.claude/hooks/lib_heavy_tests.py:3375`「return resolved is not None and os.path.realpath(resolved) == os.path.realpath(os.path.join(HOOK_REPOSITORY, location))」、`:3387`「if name in OWN_COPY_ONLY_SCRIPTS and not break_is_set("judged-by-name-path-tail"):」；
   - `scripts/experiment-apparatus/replay.sh:99`「source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"」；
   - `scripts/mutation/broad-mutation.py:4322`「if row["key"] not in current_keys:」、`:4324`「elif row["gpu_key"] != pass_key:」、`:4454`「if conflicts or gpu_plan.rejected or gpu_plan.conflicts:」；
   - `scripts/mutation/lib_mutation_units.py:759`「def not_judging_files():」、`:774`「excluded = () if break_is("judging-files-included") else not_judging_files()」；
   - `scripts/gate/lib_function_units.py:77`「今天三条，都在内存包装里」；`.claude/kb/decisions/13-验证路线.md:435` 还写「现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条」；
   - `.claude/kb/decisions/13-验证路线.md:432` 写「从 `Cargo.lock` 解析出的闭包真引到的依赖版本」，`.claude/rules/implementation-workflow.md:31` 写「解析出的依赖版本」；
   - `.claude/rules/implementation-workflow.md:49` 重型清单里没有实验复跑入口 `scripts/experiment-apparatus/replay.sh`（这一行里的 `replay.sh` 只出现在 `crash-point-replay.sh` 里）。
3. 复跑：两份模型拷进 `~/.local/share/singlefs/work/m3-close-gov-b17-r2-verify/` 再跑，`--frozen` 照快照目录：
   - 攻方：`bash scripts/hosts/run-with-memory-cap.sh 2G bash <副本>/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <仓根>/research/prompts/m3-close-gov-b17-r2-snapshot --work <副本>/opus-work`，退 0，末行「name=rerun_summary identical=6 differs=0」。
   - 辩方：同形起 `<副本>/sonnet-model/tools/rerun.sh`，退 0，八行 `result=identical`。
   - 复跑在副本里只跑 python 与钩子，没有仓副本与编译目录；输出与模型拷贝留到这一题第三轮判决写完。

## 二、逐格判定

| 格 | 辩方 | 攻方 | 主 agent 判 |
|---|---|---|---|
| H1 文本修补 | 判决要改的都改了，G2 不再与第 11 条、收工闸冲突；够不着的：`heavy-test-guard.sh` 文件头类条目与 `implementation-workflow.md:49` 重型清单没有 `replay.sh`；G1-c 论证句；依赖版本少「闭包真引到的」；第一轮「回看决策」写不受影响而定案现状句已过时；`agent-common.md` 定义级改动要这一轮点名 | 没打中 | **采辩方的够不着几处**（现查坐实，第一节第 2 条）。改法明确的进修补批：两处重型清单补实验复跑入口；依赖版本照定案补「闭包真引到的」；G1-c 那两句：「理由不同、分开记」删（论证），「管怎么存、怎么算键，不管判对错」留（判据，H3 的裁断见下）。D13 已定项 17 定案的现状句「现在只有……那一条」照判决第一轮已换成指登记位的同一口径改，交写回员（kb 三语）；第一轮回看那一格更正为「改了」，记在这一份的「回看决策」 |
| H2 只认本仓那一份 | — | 打中 a：指进本仓的入口符号链接被认作本仓那一份、不读正文，bash 却按链接所在目录 source 旁边改过的 `preflight.sh`（量的，钩子退 0）；打中 b：sha256 相同的副本里起 `--list-items` / `--quick` 照拒，与同闸 `--selftest` 比 sha256 口径不一；打中 c：门禁阶段、`mutate.sh`、包装脚本的路径尾部口子（老口子） | **a 坐实**（源码两处现查；攻方模型复跑 identical）：挂起，进第三轮；改法候选两条都是零轮：只认命令词路径本身落在本仓（不认符号链接），或认符号链接时连旁边 `preflight.sh` 一起核。**b 不采**：照 b 改成比 sha256，旁边 source 的库改过时与 a 是同一个口子；误拒在更严的一侧。b 指出的 `--selftest` 那一条判法同样只比被起脚本自己、不比它 source 的库，记欠账（拆分提案 2026-10-23 份）。**c** 是第一轮之前就有的口子，记欠账，不并进这一批 |
| H3 变异判法排除算键库 | — | a 没打中（465、0 出自那两份）；打中 b：`lib_function_units.py` 经 `executed_units` / `resolve_test` 进 crates 变异判法闭包，喂基线复用判定，排掉后改它不作废——第一轮的「多作废」翻成「少作废」（key 不变量的，改了复用判定是推的）；打中 c：自证新格改 `lib_verdict_library.py` 那一半是空测 | **b 挂起，进第三轮**：它问的是 D13 已定项 17「文件级，不管判对错」放在 crates 变异判法上成不成立，结论可能从第一轮 G1-g 的一边翻到另一边，照「结论从一边翻到另一边时，要三条互不共享前提的验证路径」，第三轮之前不改代码。**c 采**（攻方 out/crates-entries.txt 六个入口 `lib_verdict_library=0`，复跑 identical）：自证那一半写明「`lib_verdict_library.py` 对 crates 判法闭包贡献 0，改它只核指纹不变」，或换一处真在闭包里的函数，进修补批 |
| H4 GPU 判定导回 | — | 打中（小）：持久的第二台 GPU 库里旧 `gpu_key` 与不在清单的旧行使每次 `--import-results` 退 1，CPU 那一侧旧行只多一行 | **挂起，进第三轮**：「本趟该导的都导进了而旧行被拒」退 1 还是退 0，是导入的语义取舍；攻方说库持久是推的（commit-stages 第 2b 步库目录固定）。驱动自证四格前提与「被拒行照现判」攻方现跑过、成立 |
| H5 第一轮的方法 | 方法站得住；新旧两版钩子只有腿的自述、今天重喂 49 行逐字相同；「多 1」候选原因是 hook_feed 的 h11 写了改名前的旧路径；961 撑不起「不少作废」那一半 | 没打中；旁注：第一轮用 `cite-check --root .`，没走后来才有的 `snapshot-tree.py` 倒推 | **采两腿**：第一轮结论不翻；G1-g「不少作废」那一半没被证过，并进 H3-b 的第三轮。这一轮已走倒推（第一节第 1 条） |

## 三、去向

| 批 | 改什么 | 谁改 |
|---|---|---|
| 修补二批（治理文本与自证） | `implementation-workflow.md:49` 与 `heavy-test-guard.sh` 文件头类条目补实验复跑入口；`implementation-workflow.md:31` 与 `verification.md:50` 依赖版本照定案写「闭包真引到的」、删「理由不同、分开记」；H3-c 自证那一半写明或换函数 | tooling-writer，副本里改、交补丁；钩子经 `scripts/gate/hook-swap.py` |
| 写回 | D13 已定项 17 定案（中英日）现状句「现在只有……那一条」改成指登记位 `EXCLUDED_PROGRAM_EDGES` | kb-scribe |
| 第三轮 m3-close-gov-b17-r3 | 攻前两轮站住的形态：H2-a 两种零轮改法、H3-b（D13 已定项 17 文件级排除在 crates 变异判法上成不成立；结论翻面要三条互不共享前提的路径）、H4 导入退码语义；第二轮的修补二批 | 主 agent 开轮 |
| 欠账 | H2-b 指出的 `--selftest` 只比被起脚本、不比它 source 的库；H2-c 门禁阶段与包装脚本的路径尾部口子 | 记进 `records/2026-10-23-subagent拆分提案.md` |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-10-24 改了：定案现状句「现在只有……那一条」过时（`scripts/gate/lib_function_units.py:77` 今天三条），交写回员改成指登记位；第一轮写的「不受影响」更正。文件级排除在 crates 变异判法上成不成立挂起，进第三轮 |
