# split-governance-fixes-r2 辩方腿（sonnet）：复核两份第一轮判决，回正文第 5、6 问（2026-11-20）

## 复跑与文件

模型目录 `research/prompts/split-governance-fixes-r2-sonnet-model/`：两份只读探针。`probe/family_layouts.py`：造十个布局加一个门禁阶段布局，把「命名与载入的交集、只按命名、只按载入」三种认拆分族的定义各认出什么打出来，再扫仓里 scripts、task-server、.claude/hooks 的入口。`probe/source_family_registry_reason_fixed.py`：主 agent 复跑目录里那份认法探针，只把登记表每行的理由从两个字写到四个字以上（判决第 5 条的读数为什么不分辨，见第一节那一行），其余逐字相同。探针一次没跑（`python3 -m py_compile`、`bash -n` 核过语法；`out/` 是空的）；下面凡是引探针的读数都写「推的，待主 agent 跑」。探针只读仓里的文件，输出写进运行目录。推的资源：单进程 python，只读几百份文本，内存不到 200 MB、时间不到 1 分钟（推的）。

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/split-governance-fixes-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-sonnet/probe-run /home/fy5090/code/singlefs/research/prompts/split-governance-fixes-r2-sonnet-model
```

| 文件 | sha256 |
|---|---|
| `probe/family_layouts.py` | `6be71e6d61ef0fe0edfb7a4ec3df5eeff3e62ddebd62147f33c9e3c0bfc0da8d` |
| `probe/source_family_registry_reason_fixed.py` | `21982d48864615bdb21b7dddd6db4897f38aae8fcc6b6c7463e0c3a0f454d0c9` |
| `tools/rerun.sh` | `94e975a0938d50e546db21ad116c8912b1ac6a1d0bd97942607d522b5a92fe05` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 那一段（依次起两份探针）；`--frozen` 这一条没有用到，给仓根即可。

## 读了什么、没读什么

读了：两份第一轮判决；它们引的探针与模型（主 agent 复跑目录 `~/.local/share/singlefs/work/split-governance-main-rerun/` 里的那几份）；此刻主工作区里与每条判决相关的那几处（`.claude/hooks/`、`.claude/gate.d/lib/`、`scripts/`、`task-server/`、`gpu-verification-server/`）。没读：禁读清单里的五份报告、第一轮腿的草稿目录。58 份被判文件没有逐份读，只读与每条判决相关的行；读不到的写进「没做什么」。

## 记法

「回」列按派发提示：成立 = 判决这一条按它自己的依据站得住；不成立 = 依据撑不住或判错；推不出 = 现有读数分辨不了，算打中判决。「辩方判定」列：站得住 / 够不着 / 同样打中别的臂。判决原文与行号都在文件里现查过（`grep -nF` 命中）。

## 一、钩子与门禁那一份判决「二、判决」逐条（第 5 问上半）

| 判决条目（原文与行） | 回 | 辩方判定 | 证据；什么现象会推翻 |
|---|---|---|---|
| 1：「三份内嵌 python 的钩子引导读不到部分文件时抛错退 1、照常放行（拆前缺依赖按拒判）」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:30`），同行「没单跑探针」 | 打中本身成立；「拆前缺依赖按拒判」与「照常放行」两句推不出 | 站得住（打中）；够不着（归纳那一句） | ① 退 1 即放行是仓里自己的说法：「进程退 1（Claude Code 照常放行、检出记录一行不写）」（`.claude/hooks/write-guard.sh:342`）。② 探针头注自己写「完好的写闸该退 2」（`tools/probe-write-guard-part-missing.sh:3`），而完好臂退 0，判决认了原因：「副本里没有 clerk 的定义」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:19`）。所以四臂读数证明的是「缺部分文件退 1 抛异常、不记检出」，没有出现「完好臂会拒、缺文件臂放过」的一次写。③ 「按拒判」只是仓里两种已有处置之一：写闸头注把「判定自己抛异常」列为放行但必记检出（`.claude/hooks/write-guard.sh:129`），「判法读不到库、自己出错的照常放行」（`.claude/hooks/write-guard.sh:72`）；派发闸「读不到 lib_writer_lock.py 时这一条不判、不建锁」（`.claude/hooks/runner-dispatch-guard.sh:108`）、「判据库读不到时这一处没判」（`.claude/hooks/runner-dispatch-guard.sh:192`）；检出钩子「钩子自己的 python 部分文件读不到或编不过」归在放行但必记检出里（`.claude/hooks/bash-command-detector.sh:47`），「读不到、判的时候抛异常」（`.claude/hooks/bash-command-detector.sh:482`）。拆前缺依赖对这三份是放行并记检出，不是拒判；改法第 1 条仍写成四份一律退 2，实现里检出钩子已改走放行并记检出，与判决字面不同。推翻：副本补一份 clerk 定义重跑，完好臂退 2、缺部分文件臂仍退 1，则打中升级为「丢了一次拒绝」；完好臂仍退 0 则维持现在的读法。 |
| 2：「同一进程再从别的目录载入时用先载入那一份」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:31`）；「今天仓里没有从两个目录载入它的调用方，是潜在缺陷」（同行） | 成立（机制）；可达性推不出 | 站得住（机制）；够不着（「今天没有」没给出查法） | 缓存探针两臂只差拆不拆：拆分拷贝臂的部分模块指回仓里，拆前臂交「副本」（判决行 `research/prompts/split-governance-hooks-gates-r1-main-verification.md:20`），读数分辨得出。「今天没有调用方」判决没附命令；我现查：钩子各按路径载入 `lib_shell_words.py`，每个钩子一个进程一个目录（`.claude/hooks/timer-only-remind-guard.sh:91`、`.claude/hooks/find-timestamp-guard.sh:96`、`.claude/hooks/runner-dispatch-guard-prompt-checks.py:44`），同一进程载入两个目录的，在我 grep 过的这几处里只有修补时加的那一格自证（`.claude/hooks/bash-command-detector-selftest-entries-and-main.py:458-459`；只查了 `.claude/hooks/` 里按路径载入 `lib_shell_words` 的写法，别处没查）。与判决相容，但「采」的理由是潜在缺陷，修补在 `.claude/hooks/lib_shell_words.py:294-341` 加了四十八行按目录存的逻辑。推翻：找到任何会在同一进程里载入两个目录的调用方（自证拷库再进程内载入）则从潜在升级为现实。 |
| 3：「格扫描范围漏了 doc 门禁的 13 份库」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:32`） | 成立 | 站得住 | 三臂只差文件位置与名字：拆前点名 4687 行，拆后退 0，改名落进通配又点名（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:22`）；改名臂把「通配按名字认」与「内容有没有那一行」分开了，别的原因给不出同样三行。另：拆前那一份不是 HEAD 的：HEAD 的 `doc-experiments.sh` 只有 4288 行（`git show HEAD:.claude/gate.d/doc-experiments.sh` 数的，量的），4687 行取自腿的基线，与第 7 条同源。推翻：改名臂退 0。 |
| 4：「agent-write-scope 格按 `<主干>_*.py` 通配认族多认」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:33`） | 成立 | 站得住 | 同一份内容两份拷贝、只差文件名，一份被报一份不报（判决行 `research/prompts/split-governance-hooks-gates-r1-main-verification.md:23`），红因只剩名字。它在盘上的真实布局里是潜在的：两份共用模块的命名族里今天没有没载入的同名文件（要探针在真仓上的 REAL 行确认，推的，待主 agent 跑）。 |
| 5：「`source_family` 把字符串点名的文件与被 import 的共用库算进族；三份认法认出的族不一样」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:34`），依据「认法探针三行 `reported_unmarked=False`、两份认法的族不同」（同行） | 前半（族成员）成立；「三行 `reported_unmarked=False`」推不出；「三份认法的族不一样」成立但不是缺陷 | 站得住（族成员）；够不着（`reported_unmarked`）；同样打中别的臂（见第 6 问） | 族成员是量的：「source_family=['scripts/gate/sample-quotes-sibling.py', 'scripts/gate/sample-own-checker.py']」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:21`），入口只在字符串里点名的文件进了族，读数分辨得出。`reported_unmarked` 这一栏分辨不出：三行里有一行是对照（不点名任何文件，探针头注写「对照：照报」），也读成 False。原因不是「放过了」：探针写登记表时每行理由只有「样本」两个字（`tools/probe_source_family_overreach.py:34`），而登记表要求理由至少四个字（`.claude/hooks/lib_project_code_runs.py:363`、`.claude/hooks/lib_project_code_runs.py:735`），行被当写坏的行丢掉，登记表读成空；判法链里「没登记」那一支排在族检查前面（`.claude/hooks/lib_project_code_runs_launchers.py:347`、族检查在 `.claude/hooks/lib_project_code_runs_launchers.py:349`），三行都走不到族检查，探针的 `reported` 只认「登记了」起头那一句（`tools/probe_source_family_overreach.py:44`）。「三份认法的族不一样」按构造就成立（一份按命名、一份按载入），本身不是缺陷；缺陷要看它让登记核对放过一行没有核对的 required 行，这一步没被读数坐实，只被族成员读数暗示。推翻：把探针登记表理由写到四个字以上重跑，对照行读成 True、含共用库与点名字符串的两行读成 False，则门禁一层也坐实；对照行仍 False 则探针还是坏的。 |
| 6：「`project-code-guard.sh:587`、`:590` 说两条新开关…」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:35`），依据「151 条开关复跑已逐格相同」（同行） | 成立；但同一句话还打中别的行，判决只改了两处 | 同样打中别的臂 | 复跑数和头注「推的，没复跑」确实对不上，这一点成立。同一份头注里还有七行写着同一句「推的，没复跑」，现查（`grep -c`，量的）：`.claude/hooks/project-code-guard.sh:558`、`:560`、`:562`、`:564`、`:566`、`:579`、`:581`，例如「红的格（推的，没复跑）」（`.claude/hooks/project-code-guard.sh:558`）。判决的推理是「整份复跑已逐格相同，所以说没复跑不对」，对这七行同样成立；改法第 5 条只写了两行，其余没动。它们是否都在 155 条里推的，待主 agent 跑：开关复跑登记表那一行写「一百五十五条开关」、三份共 267 条逐格相同（`.claude/gate.d/break-switch-replay-files.tsv:75`），而文件头现有 157 行 `<名>_BREAK=<开关>` 起头的行（`grep -cE '^#   [A-Z_]+_BREAK=[a-z0-9-]+'`，量的，与登记表数法不一定同口径）。推翻：七行里有不在整份复跑里的开关（例如只在 lib_project_code_runs.py 自检里跑的，见 `.claude/hooks/project-code-guard.sh:581`「只在 lib_project_code_runs.py 的自检里」），则那几行的「没复跑」仍是真话。 |
| 7：「对 13 道入口不成立」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:36`） | 成立 | 站得住 | 我独立现查：`git show HEAD:.claude/gate.d/doc-experiments.sh` 4288 行、`git show HEAD:.claude/gate.d/code-tooling.sh` 12462 行（量的，`wc -l`），而判决引的拆前点名行是 4687（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:22`），超过 HEAD 的总行数，说明拆前基线不在 HEAD。只涉及正文的说法，不涉及治理文件；我只核了 doc-experiments 这一道与 code-tooling 的总行数，没逐道核 13 道。推翻：HEAD 里 doc-experiments 有 4687 行以上。 |
| 8：「第 3 问（替身换在被测命名空间上）、正推腿第 5 到 7 问」判「没打中，这一题的这几格停」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:37`） | 第 3 问部分成立；「正推腿第 5 到 7 问」我没读那条腿的报告，不判 | 站得住（第 3 问）；不判（正推腿） | 替身换在被测命名空间上的形状，现查钩子自证里改模块属性的写法（`grep`，量的）：落在拆分库上的只有两类；别的几处换的库（`.claude/hooks/runner-dispatch-guard-selftest.py:813`、`:1225`、`:1361` 的 `snapshot_library`、`prompt_scratch_line`、`three_way_legs`）在 `.claude/hooks/` 下的候选文件 `lib_snapshot_conflicts.py`、`lib_prompt_scratch_line.py`、`lib_three_way_legs.py` 各只有一份、没有同名起头的部分文件（`ls`，量的；`snapshot_library` 对应哪份文件我没核）。① `shell_words.SYSTEM_STARTUP_DIRECTORY = …`（`.claude/hooks/bash-command-detector-selftest-waits-and-scope.py:25`、`.claude/hooks/project-code-guard-selftest.py:133`），名字定义在入口自己（`.claude/hooks/lib_shell_words.py:629`）、读它的函数也在入口（`.claude/hooks/lib_shell_words.py:713`），不跨部分模块。② `project_code_runs.FAMILY_OWN_REPOSITORY`、`SAMPLE_INDEX_REPOSITORY`（`.claude/hooks/project-code-guard-selftest.py:132`、`:134`），定义在被 exec 进入口命名空间的部分文件（`.claude/hooks/lib_project_code_runs_launchers.py:1305`）与入口（`.claude/hooks/lib_project_code_runs.py:469`），exec 进同一个模块字典，换了到处看得到（部分文件的载入在 `.claude/hooks/lib_project_code_runs.py:1555`、`:1563`）。核心切词模块的别的名字没有被自证换过。推翻：找到一处自证给 `shell_words` 或别的「按模块载入」的拆分库换了入口属性、而读它的函数在拆出的部分模块里（这正是脚本与服务那一份判决四处打中的形状）。 |

## 二、脚本与服务那一份判决「二、判决」逐条（第 5 问下半）

| 判决条目（原文与行） | 回 | 辩方判定 | 证据；什么现象会推翻 |
|---|---|---|---|
| 1：「`admission_unit_key.py:1562-1563` 在入口上换 `library.data_file_unit`…」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:29`），依据「探针 PROBE_MEMO 一行」（同行） | 成立；后果只是丢了按路径只读一次的收益，不改答案 | 站得住 | 换的写法在 `scripts/gate/admission_unit_key.py:1562`；拆出模块按自己的全局名调：「data_units[file] = data_file_unit(workspace.root, file)」（`scripts/gate/lib_function_units_rust_closure.py:356`）、「units[path] = data_file_unit(self.root, path)」（`scripts/gate/lib_function_units_script_closure.py:472`）、`scripts/gate/lib_function_units_closure_keys.py:463`，三份都在拆出模块里，换在入口属性上够不着（量的，grep）。探针 PROBE_MEMO 只有拆后一臂（没有 head 臂），但同一机制的 head 臂在第 2 条：拆前 1、拆后 0，所以不是单臂读数撑起来的。自证看不出：「selftest.expect("test-plan：一趟里清单与数据文件按路径只读一次", recorded(single, "reader-memo-hits") > 0」（`scripts/gate/admission_selftest_support.py:614`）只核大于 0，经入口调的那一路就够让它为真。没读到判决说清后果的轻重：换在入口上的包装返回的是 `read(*arguments)` 的答案：「answers[key] = read(*arguments)」（`scripts/gate/admission_unit_key.py:1556`），丢的是收益不是正确性。推翻：自证改成「恰好一次」之后在拆后仍绿而探针仍读 0。 |
| 2：「`crates_mutation_rows_plan.py:552` 换 `lfu.RustFile`…」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:30`），依据「探针 PROBE_RUSTFILE 两行（拆前 1、拆后 0）」（同行） | 成立 | 站得住 | 探针两臂喂同一棵小工作区、只差库文件（拆前取 HEAD 的单文件、拆后取主工作区的入口），读数 1 对 0 分辨得出。被判代码与探针同形：「lfu.RustFile = rust_file_from_slices」（`scripts/mutation/crates_mutation_rows_plan.py:552`）之后紧接「workspace = lfu.RustWorkspace(inner.root, view)」（`scripts/mutation/crates_mutation_rows_plan.py:554`）。没被用上时回落到「return real_rust_file(root, path, text, crate_directory)」（`scripts/mutation/crates_mutation_rows_plan.py:551`），结果不变，丢的是并行预切的收益。推翻：拆后臂 `patched_calls` 读到 1。 |
| 3：「断言入口原文里有 `RustFile.row` 那一行…」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:31`），依据「探针 PROBE_ROW_LINE 两行（拆前 2、拆后 0）」（同行） | 成立 | 站得住 | 断言在「assert units_text.count(row_line) >= 1, units_file」（`scripts/mutation/crates_mutation_rows_units.py:1517`），读的是入口 `lib_function_units.py` 的原文；那一行现在在拆出的文件里：「if 0 < line <= len(self.source_rows)」（`scripts/gate/lib_function_units_rust_file.py:474`）。拆前 2 次（Rust 与 WGSL 两个类各一份，`scripts/gate/lib_function_units_wgsl.py:58` 现在也在拆出文件里）、拆后入口 0 次，读数是文本计数，别的原因给不出。推翻：拆后入口原文里数出至少一次。 |
| 4：「推翻、不采：探针 PROBE_RUSTWORKSPACE 两行」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:32`）；理由「verus 自证走 `admission_unit_key.py:359` 的 `FunctionUnitIndex.workspace()`」（同行） | 成立（推翻对）；理由里的出处写错了 | 站得住（结论）；够不着（引的位置） | 结论的核心是 verus 自证建工作区的路径读的是入口属性，我现查同意，但位置不对：`scripts/gate/admission_unit_key.py:359` 是 `KeyContext` 的方法（类起于 `scripts/gate/admission_unit_key.py:232`），不是 `FunctionUnitIndex`；`FunctionUnitIndex` 在 `scripts/crash/lib_verdict_library.py:209` 起的 `workspace`，「self._workspace = function_units().RustWorkspace(self.root)」（`scripts/crash/lib_verdict_library.py:211`）；verus 两份文件从 `lib_verdict_library` 导入它（`scripts/gate/verus_proof_check_listing.py:13`），没有任何一行引 `admission_unit_key`（`grep -rn admission_unit_key scripts/gate/verus*.py scripts/crash/lib_verdict_library.py` 无输出，量的）。自证换替身的对象是同一个库实例：「module.RustWorkspace = CountingWorkspace」（`scripts/gate/verus_proof_check_selftest_fixtures.py:751`）换在 `lib_verdict_library.function_units()` 上，建工作区时每次都经这个实例的属性取，所以够得着。「载入 1 次」的读数也分辨得出（够不着就读 0）：格里有「multi_listed_loads == 1」（`scripts/gate/verus_proof_check_selftest.py:1176`）。盲区只剩「计到的 1 次加没计到的 k 次」：库内自己建工作区的写法在 `lib_function_units_closure_keys.py:120`、`:127`、`:437`、`:560` 与 `lib_function_units_script_closure.py:977`、`:990`，verus 一路 rust 入口走 `FunctionUnitIndex.closure` 的共享工作区、python 与 shell 入口走 `closure_of` 的非 Rust 分支，没有走到它们（读代码，推的）。推翻：证明项的闭包里出现经 `rust_units`、`dump_lines` 或 WGSL 带 Rust 字符串入口的路径建出的工作区。 |
| 5：「`scripts/hosts/queue-health.py:1137` 起的循环只读入口 `task_server.py` 原文…」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:33`），依据「queue-health 自证 40 格里红这一格」（同行） | 成立 | 站得住 | 红的是自证（读不到写法就红），方向是偏严的，不会放过缺陷；读数写明两种写法在拆出模块里找不到、第三种在 `run-with-memory-cap.sh` 里找得到（判决行 `research/prompts/split-governance-scripts-services-r1-main-verification.md:21`），位置分辨得出。修补后改读拆分族：「def entry_with_split_family(entry, environment):」（`scripts/hosts/queue-health.py:169`）。推翻：拆后读数里两种写法本来就在入口文件里。 |
| 6：「看门狗新加那一格没有挂弄坏开关」判「不采：那一格是拆分族抽法的回归护栏，红的条件本身是一份造出来的拆分族输入，不靠开关证红」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:34`） | 推不出 | 够不着：被判出局的一方有规则字面撑腰，判决没引规则也没引文件内先例 | 规则字面：「没有变异表收的测试（钩子、门禁与 `scripts/` 下非装置脚本的自证这类）用弄坏开关证红」（`.claude/rules/show-me-test-by-review.md:7`），「每一处会判红的分支配一个弄坏开关或一条变异」（`.claude/rules/test-script-discipline.md:40`）。那一格有一个会判红的分支：「造一个拆成入口加同层部分文件的假钩子」起头的那一条失败说明（`scripts/agents/agent_watch_alerts.py:1104`）。造出来的输入只能证明「抽法没坏时绿」，没有证明「抽法坏了会红」；表头里点名抽法的开关只有 `unjudgedwakes`，它弄坏的是另一格（「AGENT_WATCH_BREAK=unjudgedwakes  INFO_DETECTION_PREFIXES 不含 UNJUDGED_DETECTION_PREFIXES」，`scripts/agents/agent-watch.py:554`）。文件内先例：上一格同样靠造假钩子目录证红，也没有自己的开关：「② 造一个带新起头的假钩子目录，抽法抽得出、对不上表（证明 ① 会红）」（`scripts/agents/agent_watch_alerts.py:1076`）；所以判决与这份文件的做法一致、与规则字面不一致。抽法的坏法现成：抽法只扫 `.sh` 与 `.py`（「if not name.endswith((".sh", ".py")):」，`scripts/agents/agent_watch_alerts.py:1046`），改成只扫 `.sh` 就是一个开关。推翻：规则或文件头写明「造输入的格不要开关」。 |
| 7：「没打中，这一题的这几格停」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:35`），依据「动态核对缺 0、分叉 0」（同行） | task-server 一支成立；依据里的读数分辨不出这一形状；Rust 两支成立（只核了读源码的两处与 `#[path]`） | 站得住（结论）；够不着（动态核对这一条读数） | 动态核对量的是名字在不在入口、同名可变对象是不是同一份，看不到「在入口上改绑定、拆出模块里按自己的全局名读」，四处打中的探针读数里入口上的名字都在、也没有可变对象分叉，这一项照样读 0 对 0。结论靠代码撑着：自证已经有按每份模块各换一遍的写法，「服务拆成几份之后，一个名字在每份模块里各有一个绑定」（`task-server/task_server_selftest_memory.py:41`），枚举模块的函数在 `task-server/task_server_selftest_world.py:751`，用在 `task-server/task_server_selftest_memory.py:1283`、`:1299`；被换的名字在多份调度模块里各有绑定：「from task_server_machines import MACHINE_ORDER, Placement, gibibytes_text, unit_processes, unit_states」（`task-server/task_server_scheduler_admission.py:19`）。别的换模块属性的只有一处，经入口属性取：「unittest.mock.patch.object(lib, "require_server", return_value=health)」（`task-server/task_server_selftest_admission.py:1142`），被换的 `control` 是按 `lib.require_server` 属性取的（读代码，推的）。Rust：`control` 读源码常量，定义留在入口文件：「pub const VERSION_MISMATCH_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_VERSION_MISMATCH";」（`gpu-verification-server/src/library_pending_pool.rs:135`）、读的路径「LIBRARY_PENDING_POOL_SOURCE = os.path.join(SCRIPT_DIRECTORY, "src", "library_pending_pool.rs")」（`gpu-verification-server/control:332`）；`#[path]`：「#[path = "../common/mod.rs"]」（`gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/main.rs:12`）相对 main.rs 所在目录，指到 `tests/common/mod.rs`。推翻：task-server 自证里有只换入口属性、而读它的函数在拆出模块里的写法（我的 grep 范围是 `task-server/task_server_selftest*.py` 里的 `patch.object`、`setattr` 与 `<模块别名>.<名字> =`）。 |
| 8：「推不出、核不动：攻方腿没找到比较源码与在跑二进制的格」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:36`） | 成立（判「推不出、核不动」没有越界） | 站得住 | 我找 `control` 里读源码的几处：它们读源码文本去造要核的那一行，不是读在跑进程的二进制再与源码比：「src/library_pending_pool.rs 里版本对不上那一行的开头与键」（`gpu-verification-server/control:2347`）。没有找到比较在跑二进制与源码的格，与判决「没找到」一致；我没有核 `control` 其余的读法，也没核 task-server 的 `control`。推翻：找到读在跑进程的版本串与源码常量逐字比较的格。 |

## 三、改法第 4 条的认族定义（第 6 问）

判决原句：「族的定义取两样的交集：名字落在 `lib_lag_split_family.in_family` 认的命名族里，且入口真载入它」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:44`）；另一份判决：「认族的写法与另一题判决第三条改法第 4 条那一份并成一份」（`research/prompts/split-governance-scripts-services-r1-main-verification.md:42`）。方案按 `crates/` 今天的实现来谈：`crates/` 下没有这一条的调用方，下面只涉及 `.py`、`.sh`；`.rs` 那一半只在 `lib_lag_split_family.in_family` 里，这一份共用库不认 `.rs` 载入（`scripts/gate/lib_split_family_loaded.py` 里没有 `mod x;` 的写法）。

认法本身的几处边界（量的，读代码）：载入只认三种文本写法，文件名按「引号紧挨着」取：`QUOTED_SCRIPT_NAME = re.compile(`（`scripts/gate/lib_split_family_loaded.py:50`），比较时只看文件名：「if candidate in seen or os.path.basename(candidate) not in loaded:」（`scripts/gate/lib_split_family_loaded.py:122`）；候选只列入口同目录：「入口同目录里的 .py 与 .sh 文件的路径」（`scripts/gate/lib_split_family_loaded.py:98`）；命名族里门禁阶段 lib 那一支按仓内相对路径认：「if directory == GATE_STAGE_DIRECTORY:」（`scripts/gate/lib_lag_split_family.py:97`）。

### 三种调用方各拿族做什么，方向并不一样

| 调用方 | 拿族做什么 | 族比真的大（多认）会怎样 | 族比真的小（少认）会怎样 |
|---|---|---|---|
| 临时服务登记核对 | 看 required 行的源码里有没有核对：「elif judgement == TEMPORARY_SERVICE_REQUIRED and not marked and not family_has_temporary_service_mark(」（`.claude/hooks/lib_project_code_runs_launchers.py:349`） | 别的脚本的核对被算给它：放过没核对的行（不安全） | 报「源码里没有核对」：多报（偏严、安全） |
| queue-health | 在入口加族里找任务服务的几种写法 | 别处的写法被算给它：该红的不红 | 找不到就红：多报（安全） |
| agent-write-scope 格，跳过的文件名 | 被判仓自己入口连同部分，不查手抄：「if directory == hooks_directory and file_name in family_names:」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:740`） | 名字像的死文件里手抄的函数被跳过：放过（不安全） | 部分文件里的合法定义被当手抄：多报（安全） |
| agent-write-scope 格，查的函数名 | 名字取自这一道脚本旁边那一族：「part_paths = sorted(family_library.loaded_family(module_path, family_library.sibling_files(module_path), family_defects)[1:])」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:700`），再「function_names = list(function_home)」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:721`）、「checked_names = [name for name in function_names」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:733`） | 多几个函数名：多查、多报（偏严、安全） | 少几个函数名：那几个函数在别处被手抄也没人报（不安全） |

共用库头注写的是「读不了的文件当没有载入任何东西（族只会变小，各调用方都按族小了报得更多的方向用它）」（`scripts/gate/lib_split_family_loaded.py:13`）；对前三行成立，对最后一行不成立：同一格里「查的函数名」这一处族小了报得更少。格里只有一道部分兜底：登记的排除项在函数名集合里找不到会红（「stale_exclusions = sorted(set(not_judgment) - set(function_names))」，`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:728`），只罩得住登记过的那几个名字。所以「认族定义取交集」对前三行是最小族（安全方向），对最后一行是不安全的方向；这一格同时用了两个相反方向的角色，用同一个定义取族，正是替代方案里「只按命名」在这一个角色上更好的地方。

### 各定义在哪种布局下判错

「√」认对、「多」多认、「少」少认。「真仓例」是我现查的（量的，grep）；「样本」是仓里已有的自证样本；表里 `__all__` 一栏按「再导出名单」读：`__all__` 在 scripts、task-server、.claude、prune-server 下行首定义的文件数是 0（`grep -rlE '^__all__' --include=*.py`，量的）。

| 布局 | 交集 | 只按命名 | 只按载入 | `__all__` 与再导出名单 | 真仓例或样本 |
|---|---|---|---|---|---|
| 名字像部分文件、没人载入的死文件 | √ | 多 | √ | √ | 样本 `agent-write-scope-name-like-red`（修补带的）；开关 `family-by-name` |
| 被 import 的共用库，名字不像 | √ | √ | 多 | 再导出了才多 | 样本 `agent-write-scope-loaded-shared-red`；真仓：切词部分模块的名字在检出钩子的部分文件里被点名：「SHELL_WORDS_PART_NAMES = ("lib_shell_words_core.py", "lib_shell_words_embedded.py")」（`.claude/hooks/bash-command-detector-scope-and-index.py:11`），只按载入时它们会算进检出钩子的族 |
| 共用库，名字带着入口的词干（命名与载入都满足） | 多 | 多 | 多 | 不适用（shell 没有再导出） | 真仓：`crash-amplification-lib.sh` 被三份入口各自 source：「source "$crash_amplification_script_directory/crash-amplification-lib.sh"」（`scripts/crash/crash-amplification.sh:375`）、「source "$item_script_directory/crash-amplification-lib.sh"」（`scripts/crash/crash-amplification-item.sh:148`）、「source "$two_hosts_script_directory/crash-amplification-lib.sh"」（`scripts/crash/crash-amplification-two-hosts.sh:193`）；钩子拆法按「<名>-<部分>.sh」认：「recipes.append((HOOK_SPLIT_UNRECOGNIZED, own_directory, stem + "-", SCRIPT_SUFFIXES))」（`scripts/gate/lib_lag_split_family.py:96`），所以它在 `crash-amplification.sh` 的命名族里，又被它 source，交集把一份三个入口共用的库算成它的族；它在临时服务表里自己也是一行 required（`.claude/hooks/selftest-temporary-service.tsv:20`），三份入口各自带着核对（`grep -c selftest_service_problem`，量的：`crash-amplification.sh` 2、item 3、two-hosts 4，库 5），今天不改结果；入口丢了核对时，库的核对会算给它。这正是判决要排除的「被几份脚本共用的库」，仓里自己也写过：「把它们算进族会把别的脚本判成漏登」（`.claude/hooks/lib_project_code_runs_launchers.py:331`） |
| 只在注释、文档串、说明字符串里写了名字像的文件名 | 多 | 多 | 多 | √ | 载入的认法是文本匹配、不分代码与注释，所以引号里的名字都算载入；探针造了这一布局（推的，待主 agent 跑），REAL 行 `doc-only-mention` 数仓里真有几处 |
| 真部分文件用认不出的写法载入（拼出来的名字、`from . import`、`from .名 import`、`exec bash "$目录/名.sh"`） | 少 | √ | 少 | 看写法 | 真仓：「[[ "${1:-}" != "--selftest" ]] || exec bash "$SCRIPT_DIRECTORY/gate-staged-selftest.sh"」（`scripts/gate/gate-staged.sh:34`），名字前有 `$SCRIPT_DIRECTORY/`，引号紧挨着名字的写法与 source 写法都认不出它；它是命名法里 `gate-staged-` 起头的文件，是否算「部分」看调用方，这里只当认法的边界例子 |
| 真部分文件在子目录，或点号入口的横线名部分文件 | 少 | 少 | 子目录少、横线名 √ | 少 | 探针布局 `part-in-subdirectory`、`hyphen-part-of-python-entry` |
| 门禁阶段入口按绝对路径给 | 少 | 少 | √ | 不适用 | 命名族里 lib 那一支按仓内相对路径认（`scripts/gate/lib_lag_split_family.py:97`），探针布局 `gate-stage-lib-parts-absolute`（推的，待主 agent 跑） |
| 用 `PART_NAMES` 写死名单 exec 组合的入口 | √ | √ | √ | 少 | 仓里七份：`.claude/hooks/write-guard.sh:358`、`.claude/hooks/runner-dispatch-guard.sh:1018`、`.claude/hooks/completion-cleanup-guard.sh:419`、`.claude/hooks/bash-command-detector.sh:1403`、`.claude/hooks/project-code-guard.sh:681`、`.claude/hooks/lib_run_scope.py:1904`、`.claude/hooks/lib_project_code_runs.py:1555`（量的，grep）；这些入口什么都不再导出，名字直接落在入口自己的命名空间，agent-write-scope 格要认的两份共用模块之一 `lib_project_code_runs.py` 就是这一种 |
| 用 import 组合、入口把名字再导出的入口 | √ | √ | √ | √ | `lib_shell_words.py`（「这一份把另两份的名字全部再导出」，`.claude/hooks/lib_shell_words.py:283`），`lib_function_units.py` 有 23 条 `from lib_function_units_… import`（量的，`grep -c`） |

### 第 6 问的回

- 回「不成立」：改成交集之后，打中它的几种布局并不是同样打中三种替代方案。同一种布局的归属：
  - 三种定义一起错（交集没有变坏也没有变好）：共用库名字带着入口词干；注释、文档串里点名；子目录里的真部分文件。
  - 交集比替代方案好：名字像而没载入的死文件（只按命名错）；被 import 的共用库、名字不像（只按载入错）。判决自己加的两格自证正好只覆盖这两种：「自证各加一格：名字像部分文件而入口没载入的不算族；被 import 的共用库不算族」（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:44`）。
  - 交集比替代方案差：认不出的载入写法（只按命名对）；点号入口的横线名部分文件与按绝对路径给的门禁阶段入口（只按载入对，后者不依赖命名那一支）。这三种没有样本、没有弄坏开关，格里能盖到的只有「全部部分文件都读不到」那一条：「弄坏开关 AGENT_WRITE_SCOPE_BREAK=family-parts-unread：查的函数名只取入口文件里的顶层函数」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:382`）。
- 再导出名单（`__all__`）一支：仓里 0 份 `__all__`，按它认就是处处空族；按「入口 from 导入的名字」认只罩得住 import 组合的入口，罩不住 exec 组合的七份（上表倒数第二行），而 agent-write-scope 格要认的两份共用模块里一份就是 exec 组合的。所以这一支不是可用的替代方案，判决没选它是对的；判决没写这条理由。
- 「族取交集」对三种调用方用的是同一个定义，而它们对「多认、少认」要的方向不一样（上面第一张表）：对前三种角色最小族是安全方向，对「查的函数名」这一个角色少认不安全。这是判决与修补都没有分开的一件事，不是我对改法的取舍。
- 我没有做的：不判交集该不该撤；不判「查的函数名取并集、跳过的文件名取交集」这类拆开取族的办法要不要采，只写明它存在。

### 探针的读数（推的，待主 agent 跑；每一行我都没跑过）

造的十个布局是我挑的、专找三种定义分得开的形状，恰好认对的个数不是频率，不能拿来比好坏。按代码手推（推的）：交集恰好认对 4 个、只按命名 6 个、只按载入 4 个（SUMMARY layouts 那一行）；`gate-stage-lib-parts-relative` 三种定义都认出 `stage-cells.sh`，`gate-stage-lib-parts-absolute` 交集与只按命名认不出、只按载入认出。真仓那一段（REAL 与 SUMMARY real 行）不推：它回答仓里有几处名字像而没载入、几处只在注释里被点名、几处名字像又在正文里出现却没认成载入。
第 5 条那份探针（`probe/source_family_registry_reason_fixed.py`）：推的读数是对照行 `reported_unmarked=True`、点名字符串与 import 共用库的两行 `reported_unmarked=False`；若对照行仍读 False，探针还是坏的，第 5 条那一格仍然分辨不出。

## 要主 agent 跑的命令

1. 两份探针一趟起（`tools/rerun.sh` 经清运行目录的包装起；服务撞内存上限重排同一条命令时照样跑得通）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/split-governance-fixes-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-sonnet/probe-run /home/fy5090/code/singlefs/research/prompts/split-governance-fixes-r2-sonnet-model
```

   - 在哪起：仓根；运行目录 `probe-run` 在这一腿认领的草稿目录下。范围声明闸要的话照 `.claude/hooks/run-scope-guard.sh` 文件头补。
   - 跑完看：`TASK_SERVER_RESULT` 末行 `exit=0`；`task-server/task log <号> --all` 里有 `SUMMARY layouts …`、`SUMMARY real …`、`PROBE_DONE`、三行 `PROBE entry=… reported_unmarked=…`、`name=rerun_roots …`。读数与上面「推的」不同的，以跑出的为准，我的推断作废。

2. 数头注里「推的，没复跑」还剩几行、复跑登记表那一行写的条数（无需交服务，只读命令，主 agent 直接跑）：`grep -c '推的，没复跑' .claude/hooks/project-code-guard.sh`（我现查是 7）。它们是否都在 155 条开关里要用开关复跑那一格的 `--list` 或整份复跑的读数核，我没核。

## 什么现象会推翻我的结论

- 第 1 条「够不着」：补 clerk 定义后完好臂退 2、缺部分文件臂退 1。
- 第 5 条「读数不分辨」：改探针后对照行读 True、另两行读 False（则门禁一层的放过也被坐实，原判决那一条升为站得住）。
- 第 6 条「同样打中」那一支：七行里有不在整份复跑里的开关。
- 脚本一份第 4 条「出处写错」：`admission_unit_key.py:359` 其实被 verus 自证走到（我 grep 无输出）。
- 脚本一份第 6 条：规则或文件头写明造输入的格免开关。
- 第 6 问「不成立」：探针读出的 REAL 行没有任何一处布局让三种定义分开（则交集在仓里今天恰好不输给任何一种替代）。

## 没做什么

- 没替主 agent 撤销或维持任何一条判决，只交复核结果；没往统一任务服务交任务；探针一次没跑，读数都写了「推的，待主 agent 跑」；判决要引的数没有复跑。
- 没读禁读清单里的五份报告，也没读第一轮腿的草稿目录；两份判决里引「正推腿第 5 到 7 问」的几格（钩子一份第 8 条后半、脚本一份第 7 条后半）我没判。
- 58 份被判文件没有逐份读，只读与每条判决相关的行；钩子一份第 3、4、7 条只核了与我的结论相关的那一部分，没有复跑它们的三臂。
- 第 1 到 4 问（攻方的活）没做；认族定义的第 3、4 问里的绝对路径与两族不同布局，只在上表两行里顺带提到，没单独攻。
- 脚本与服务一份第 7 条里 `control` 读法只核了读 `library_pending_pool.rs` 的两处，没核另一个 Rust 服务与 task-server 的 `control`。
- 没核函数单元库被三处各载入一次入口（`scripts/gate/admission_base.py:292`、`scripts/crash/lib_verdict_library.py:186`、`scripts/mutation/lib_mutation_units.py:155`）、而拆出的子模块按裸名字共用时，模块级状态是否被几个实例串用；这是脚本一份第 7 条「动态核对」没有覆盖的一格（那一条读数取的是一个实例），我只读到三处载入，没找到依赖它的自证，推的。
- 清理：草稿目录下没有仓副本、编译目录；认领的草稿目录里只有 `fragments.tsv`（几百字节）。
