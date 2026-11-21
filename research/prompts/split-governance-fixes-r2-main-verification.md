# split-governance-fixes-r2：判决（2026-11-20）

正文 `research/prompts/_split-governance-fixes-r2-body.md`，开工快照 `research/prompts/split-governance-fixes-r2-snapshot/`。这是这一题的第二轮，攻第一轮两份判决（`research/prompts/split-governance-hooks-gates-r1-main-verification.md`、`research/prompts/split-governance-scripts-services-r1-main-verification.md`）的修补，辩方腿复核那两份判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（opus） | `research/prompts/split-governance-fixes-r2-opus-output.md` | 第 1 问前半不成立：部分文件编得过但不完整时照样过；后半推不出：完工清理闸在 Stop 上照退 2、不看 stop_hook_active，项目代码闸与写闸同时读不到时主 agent 只剩只读工具。第 2 问成立。第 3 问不成立：载入那一半按文本扫多认少认、门禁阶段绝对路径不认。第 4 问不成立：注释漏报、`import_module` 误报。撤回与收窄：第一轮改法第 1 条盖不住编得过而不完整的部分文件，第 4 条「两种多认都排掉」说宽了 |
| 辩方（sonnet） | `research/prompts/split-governance-fixes-r2-sonnet-output.md` | 钩子与门禁那一份：第 1 条「拆前缺依赖按拒判」推不出（完好臂退 0，派发闸与检出钩子拆前缺依赖本就放行加记检出）；第 5 条读数推不出（对照行也读 False，探针登记理由只有两个字）；第 6 条只改了 2 行，文件头还剩 7 行「推的，没复跑」。脚本与服务那一份：第 4 条结论成立但出处写错；第 6 条推不出（`show-me-test-by-review.md:7` 要弄坏开关）；第 7 条读数分辨不出入口属性那一形状。第 6 问：交集不是同样打中所有替代方案 |

核查命令：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py split-governance-fixes-r2 --out /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/split-governance-fixes-r2-tree --repository /home/fy5090/code/singlefs`：结局 type=succeeded exit=0，58 份都倒推得出。
- `python3 scripts/agents/cite-check.py research/prompts/split-governance-fixes-r2-opus-output.md research/prompts/split-governance-fixes-r2-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/split-governance-fixes-r2-tree --background research/prompts/_split-governance-fixes-r2-body.md`：「✓ 核了 71 处引文、109 处裸行号，对上 22 处，没判 158 处（其中裸行号没有引文可比 67 处；逐处列在上面）」。没判的多是裸行号、快照树外的第一轮判决与仓外探针路径；判决引的读数主 agent 都在复跑里现取，不靠腿的引文。

主 agent 复跑（两条腿的模型目录拷到 `/home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/r2-attack/`、`/home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/r2-defense/`，sha256 与腿的相同，在那里交）：

- 攻方探针（`…/r2-attack/model/tools/rerun-from-a-clean-work-directory.sh`）：结局 type=succeeded exit=0。五份钩子「arm=main-part-empty exit=0 detections=0」，自证子进程同样「exit=0 output_lines=0」；「hook=write-guard arm=first-part-empty exit=1 detections=0 stderr_first=Traceback」，派发闸、完工清理闸同样 exit=1，项目代码闸与检出钩子 exit=0 记一条检出；缺部分文件那一臂五份都照修补退 2 或记检出。共用切词模块五种路径写法里只有「form=symlink-then-dot-dot … same_core_as_absolute=False」。认族：「case=gate-stage-naming in_family_relative=True in_family_absolute=False」「case=quoted-name-in-comment family=['lib_demo_shadow.py']」「case=import-line-in-docstring family=['lib_demo_shadow.py']」「case=relative-from-import family=[]」「case=import-module-by-name family=[]」。
- 五份样本上的 agent-write-scope 格（`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash <腿的冻结副本>/.claude/gate.d/code-tooling.sh --check agent-write-scope <样本>`，在仓根交）：原样 name-like-red 结局 type=command-failed exit=1、点名「.claude/hooks/lib_shell_words_shadow.py:4  def commands_at_command_position」；注释里写影子文件名、文档串里有 import 行的两份 exit=0（漏报）；按 `import_module` 载入自己部分文件的那一份 exit=1、点名「.claude/hooks/lib_shell_words_core.py:1  def brace_expand」（误报）；按 `from … import` 载入的那一份 exit=0。
- 辩方探针（`…/r2-defense/model/tools/rerun-from-a-clean-work-directory.sh`）：结局 type=succeeded exit=0。「SUMMARY layouts intersection-exact=4-of-10 name-only-exact=6-of-10 loaded-only-exact=4-of-10」「SUMMARY real entries=316 name-only-differs=5 loaded-only-differs=209 doc-only-mentions=0 mentioned-not-loaded=7」；真仓上只按命名多认独立脚本（「entry=scripts/crash/crash-amplification.sh intersection=1 name-only-extra=['crash-amplification-item.sh', 'crash-amplification-two-hosts.sh']」），只按载入多认共用库（「entry=scripts/gate/admission.py intersection=6 … loaded-only-extra=['lib_function_units.py', …]」）。登记理由写全之后三份样本「reported_unmarked=True」。
- 仓里搜相对 import：`grep -rlE '^\s*from \. import|^\s*from \.[a-z]' scripts .claude/hooks .claude/gate.d task-server --include=*.py` 无输出。

## 二、判决

| # | 打中 | 判 | 依据 |
|---|---|---|---|
| 1 | 五份钩子引导盖不住编得过而不完整的部分文件：带 main 的那一份截空时静默退 0、不记检出；第一份截空时写闸、派发闸、完工清理闸抛错退 1 | 坐实而采 | 攻方探针 main-part-empty 五臂 exit=0、first-part-empty 三臂 exit=1 |
| 2 | 完工清理闸读不到部分文件时在 Stop、SubagentStop 上照退 2、不看 `stop_hook_active`；项目代码闸与写闸同时读不到时主 agent 只剩只读工具 | 坐实而采（读码核：`completion-cleanup-guard.sh` 引导段在读 hook 输入之前退 2；两闸同时读不到没有探针，是推的） | 攻方腿第 1 问后半 |
| 3 | `lib_split_family_loaded` 载入那一半按文本扫：注释、字符串、文档串里的 import 与引号文件名算载入；`from . import`、`import_module` 认不出；门禁阶段入口给绝对路径时命名那一半不认 | 坐实而采（`from . import` 仓里 0 处，是潜在缺陷） | 攻方探针认族五行；仓里搜相对 import 无输出 |
| 4 | agent-write-scope 格在入口注释或文档串点名影子文件时漏报、被判仓按 `import_module` 载入自己部分文件时误报；共用库文件头「族小了报得更多」对这一格查函数名那一侧不成立 | 坐实而采 | 五份样本复跑，两份漏报、一份误报 |
| 5 | 第一轮钩子与门禁那一份第 1 条「拆前缺依赖按拒判」：完好臂退 0、读数分辨不了；派发闸与检出钩子拆前缺依赖是放行加记检出 | 坐实，改那一条的措辞（打中本身照旧成立：第二轮攻方探针缺部分文件那一臂现在各照修补退 2 或记检出） | 辩方腿第 5 问；攻方探针 intact 臂写闸 exit=2 |
| 6 | 第一轮钩子与门禁那一份第 5 条的读数（`reported_unmarked=False` 三行）分辨不了：探针登记理由太短、登记核对先于族检查退了 | 坐实，读数作废、换成辩方复跑的「reported_unmarked=True」三行（修补之后） | 辩方探针 |
| 7 | 第一轮改法第 5 条只改了 2 行，`project-code-guard.sh` 文件头还剩 7 行「推的，没复跑」 | 坐实而采：那 7 条开关已在 155 条整份复跑里逐格相同 | `grep -n '推的，没复跑' .claude/hooks/project-code-guard.sh` 列出 558、560、562、564、566、579、581 行 |
| 8 | 第一轮脚本与服务那一份第 4 条出处写错：`admission_unit_key.py:359` 是 `KeyContext.workspace`，`FunctionUnitIndex` 在 `scripts/crash/lib_verdict_library.py:209` | 坐实，改出处，结论照旧 | 辩方腿原话；主 agent 现读那两处 |
| 9 | 第一轮脚本与服务那一份第 6 条（看门狗新格没挂开关判不采）：`.claude/rules/show-me-test-by-review.md` 第 7 行要每处会判红的分支配一个弄坏开关或一条变异 | 坐实，翻成采：给那一格补一条开关 | 规则原文 |
| 10 | 第一轮脚本与服务那一份第 7 条：动态核对分辨不出「换在入口属性上」那一形状，结论靠 `task_server_selftest_memory.py:40` 的 `patched_server_name` 撑着 | 坐实，补依据，结论照旧 | 辩方腿原话 |
| 11 | 认族取交集不是同样打中所有替代方案：真仓上交集比只按命名少多认 5 个入口、比只按载入少多认 209 个；三种一起错的有共用库名带入口词干、子目录部分文件；交集更差的是认不出的载入写法与绝对路径 | 坐实；交集照留，改法第 3 条补上少认的写法与路径，第一轮改法第 4 条「两种多认都排掉」收窄成「真仓上排掉」 | 辩方探针 SUMMARY 两行 |
| 12 | 第 2 问（共用切词模块路径写法） | 没打中，停 | 只有符号链接后接 `..` 一种，仓里没有这样拼路径的调用方 |

## 三、改法

1. 五份钩子与另三处引导：exec 完部分文件之后核每份部分文件声明的、入口要的名字都在（入口写一张要的名字表，部分文件里缺一个就按「读不到」那一支处置），主函数由引导在核完之后调，不靠部分文件末行 `sys.exit(main())`。自证加两格：带 main 的那一份截空、第一份截空。零轮。
2. 完工清理闸的引导读不到部分文件时先读 hook 输入：`stop_hook_active` 为真时放行并记检出，子 agent 的 SubagentStop 放行并记检出；只在主 agent 的第一回 Stop 上退 2。写闸读不到自己的部分文件时，对写回那一份部分文件本身的 Write 放行（出路只剩这一条时它走得通）。自证各加一格。零轮。
3. `lib_split_family_loaded` 的载入那一半：python 文件用 `ast` 认 `import`、`from X import`、同目录的 `from . import X` 与 `from .X import`、字面量参数的 `importlib.import_module`，引号文件名只认赋给 `*_PART_NAMES` 一类模块层元组或列表里的；shell 文件去掉注释再认 `source` 与 `.`；路径先换成仓内相对再调 `lib_lag_split_family.in_family`。文件头「族小了报得更多」那一句改成分调用方写：agent-write-scope 格查函数名那一侧族大了才报得多。三处调用方的自证各加一格注释点名与一格 `import_module`。零轮。
4. 看门狗 `agent_watch_alerts.py` 拆分族那一格补一条弄坏开关，登记进 `.claude/gate.d/break-switch-replay-files.tsv` 那一行的条数。
5. `project-code-guard.sh` 文件头剩下 7 行「推的，没复跑」改成实测。
6. 第一轮两份判决照第 5、6、8、10 条改措辞、出处与依据（只改那几格，不改结论）。
7. 改完开第三轮（最后一轮），攻这几处修补；第三轮的修补标零轮，不再开第四轮。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/gate.d/break-switch-replay-files.tsv：开关复跑登记表：几行条数照这一批复跑的现数改过
- .claude/gate.d/code-tooling.sh：agent-write-scope 格改调共用库：攻方腿第 4 问坐实入口注释里写了影子文件名、文档串里有 import 行时漏报，被判仓按 `import_module` 载入自己的部分文件时误报
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-green/expect：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/agents/alpha.md：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/agents/beta.md：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/gate.d/10-sample.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/gate.d/stage-owners.tsv：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/hooks/agent-write-scope.tsv：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/hooks/lib_project_code_runs.py：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/hooks/lib_shell_words.py：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/hooks/shared_text_tools.py：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/hooks/uses-shared-project-code.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/hooks/uses-shared-shell-words.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/settings.json：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/expect：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/.gate-cells：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-loaded-shared-red/scripts/devices/vm-bench.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/agents/alpha.md：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/agents/beta.md：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/gate.d/10-sample.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/gate.d/stage-owners.tsv：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/hooks/agent-write-scope.tsv：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/hooks/lib_project_code_runs.py：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/hooks/lib_shell_words.py：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/hooks/lib_shell_words_shadow.py：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/hooks/uses-shared-project-code.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/hooks/uses-shared-shell-words.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/settings.json：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/expect：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/.gate-cells：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-name-like-red/scripts/devices/vm-bench.sh：认族并成一份带进来的样本：攻方腿第 4 问拿它对照两族取法不同时的漏报误报，主 agent 在五份改过的样本上复跑这一格
- .claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh：agent-write-scope 格改调共用库：攻方腿第 4 问坐实入口注释里写了影子文件名、文档串里有 import 行时漏报，被判仓按 `import_module` 载入自己的部分文件时误报
- .claude/hooks/bash-command-detector-scope-and-index.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/bash-command-detector-selftest-entries-and-main.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/bash-command-detector-selftest-waits-and-scope.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/bash-command-detector.sh：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/completion-cleanup-guard-decisions.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/completion-cleanup-guard-selftest.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/completion-cleanup-guard.sh：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/lib_project_code_runs_launchers.py：`source_family` 改调共用库与引导修补的自证：攻方腿第 3 问的多认少认落在它调的共用库上，辩方腿复跑登记理由写全之后三份样本都报出源码里没有核对
- .claude/hooks/lib_project_code_runs.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/lib_project_code_runs_selftest.py：`source_family` 改调共用库与引导修补的自证：攻方腿第 3 问的多认少认落在它调的共用库上，辩方腿复跑登记理由写全之后三份样本都报出源码里没有核对
- .claude/hooks/lib_run_scope.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/lib_run_scope_selftest.py：`source_family` 改调共用库与引导修补的自证：攻方腿第 3 问的多认少认落在它调的共用库上，辩方腿复跑登记理由写全之后三份样本都报出源码里没有核对
- .claude/hooks/lib_shell_words.py：共用切词模块按目录各存一份：攻方腿第 2 问量五种路径写法，只有「符号链接后接 ..」一种让入口与部分模块出自两个目录
- .claude/hooks/project-code-guard-decisions.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active；辩方腿第 5 问核出文件头还剩 7 行「推的，没复跑」
- .claude/hooks/project-code-guard-selftest.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active；辩方腿第 5 问核出文件头还剩 7 行「推的，没复跑」
- .claude/hooks/project-code-guard.sh：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active；辩方腿第 5 问核出文件头还剩 7 行「推的，没复跑」
- .claude/hooks/runner-dispatch-guard-selftest.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/runner-dispatch-guard.sh：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/run-scope-guard.sh：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/write-guard-selftest.py：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- .claude/hooks/write-guard.sh：引导「先读进并编过才 exec」那一处修补：攻方腿第 1 问坐实带 main 的部分文件截空时静默退 0、第一份截空时抛错退 1，完工清理闸在 Stop 上照退 2 不看 stop_hook_active
- scripts/gate/lib_split_family_loaded.py：新共用库：攻方腿第 3 问坐实载入那一半按文本扫多认（注释、字符串、文档串）与少认（`from . import`、`import_module`、门禁阶段绝对路径），辩方腿第 6 问比它与三种替代定义在十种布局与真仓 316 个入口上的差
- scripts/gate/README.md：`scripts/gate/README.md` 加新共用库那一行：辩方腿第 6 问核它与共用库文件头说的调用方一致
- scripts/hosts/queue-health.py：queue-health 改调共用库与补点名：攻方腿第 3 问的多认少认同样落在它的 `entry_with_split_family` 上；开关复跑 33 条逐格相同

## 回看决策

不涉及决策：这一轮判的是治理文件按 2000 行拆分之后的行为与拆分带出的缺陷，`.claude/kb/decisions/` 下没有一条分项因此改动。
