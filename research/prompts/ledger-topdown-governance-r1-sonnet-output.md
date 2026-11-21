# 治理文件轮 ledger-topdown-governance-r1：正推腿报告（2026-11-18）

腿：`three-way-forward`（Sonnet）。判的是正文「范围」一节表里的 52 份治理文件改动，只做 A（逐份核改动与账本节）、B（正文第 7 问）、C（正文第 8 问）、D（正文第 10 问）；第 1 到 6 问与第 9 问归攻方腿，没做。禁读清单里的攻方腿报告没读。

## 读法与记号

- 账本 = `records/2026-11-17-subagent账本自上而下清欠.md`，写成「账本:行号」；调度 = `records/2026-11-17-subagent账本清欠调度记录.md`，写成「调度:行号」。两个记号都是行号，行号都是读时现取。
- 逐份读的是被判文件里点到的行与 `git diff HEAD -U0`；混改的只判表里点到的几处。「验证」一栏里的任务号是调度里的转述，我没读任何一份 task 日志（没有给号），都按「转述」办。
- 开工快照 `research/prompts/ledger-topdown-governance-r1-snapshot/judged-sha256.txt` 57 份：开工时 `sha256sum -c` 57 份全 OK；读到一半再核一次，`.claude/gate.d/code-tooling.sh` 已与快照不符（别的会话在改它，修改日期 2026-11-18），其余 56 份仍相符。code-tooling.sh 的行号取它被改之后读到的：runner 表加 make-swap-spec.py 在 2207 行、加 e56 入口在 2230 行、账本指针在 9597 行（正文写 2205、2228、9595，差两行是别的会话插进的行，改动本身不变）。
- 判定只用三种：成立、不成立、推不出（推不出算打中）。
- 命令被权限分类器拒过一次：`python3 scripts/hosts/lib_repository_copy.py entries .`（我直接起它被拒，没有绕），写进末尾「要主 agent 跑的命令」。

## A 逐份核：改动做的是不是账本那一节说的

52 行各一个判定。「账本节与行号」一栏：账本里有节的写节号与行号；账本写「没写」的，写调度哪一行说了为什么改。

### 一、定义、共用约束与主 agent 流程

| 路径 | 账本节与行号 | 判定 | 差在哪 |
|---|---|---|---|
| `.claude/agent-common.md` | 第 44 行：账本没写，调度:232 说清（私有值落点加仓根四份清场脚本）；第 100 行：二百三十六 ⑥ 账本:491、二百三十一 账本:446 | 成立 | 无差。第 44 行：仓根 `multi-host-quiesce-*.sh` 恰 4 份、都未跟踪、`.gitignore:22` 命中；第 100 行与 `.claude/hooks/write-guard.sh:692-693` 的两个服务 `src/` 分支同义 |
| `.claude/agents/implementation-writer.md` | 二百五十一 ①② 账本:640、:641、:648 | 不成立（少做一半） | ② 第 32 行点名 `-p <包>` 成立（`task-server/task_server.py:7268` 在判）；① 第 44 行、第 48 行首条证红命令改成 `--cwd <仓根>` 成立，但第 48 行里缩域那一条证红仍写 `--cwd <副本>`，照写仍退 2（见 B 节对 `.claude/agents/implementation-writer.md:48` 的那一行）；账本:648 自认「改完还没拿一条新变异行走一遍」 |
| `.claude/agents/tooling-writer.md` | 二百三十一 账本:446；二百三十六 ② ④ ⑥ 账本:487、:489、:491 | 成立 | 第 35 行「键算不出的脚本登不进那张表」账本没有对应行，依据在调度:105、调度:111；第 31 行 `--verify` 的写法有账本:772 自认的缺陷（见 B 节 `--verify` 那一行） |
| `.claude/main-agent.md` | 账本没写；调度:232 说清（派发表工具员一行把换上写成经换上工具、脚本与钩子都算） | 成立 | 无差；「两个服务 `src/` 下的打补丁」一句出自账本:491（⑥）的出路，账本没点名这份文件，调度:232 点名了 |

### 二、钩子（`.claude/hooks/`）

| 路径 | 账本节与行号 | 判定 | 差在哪 |
|---|---|---|---|
| `.claude/hooks/write-guard.sh` | 二百三十六 ⑥ 账本:491；二百三十一 账本:446-449；第 88 行指针指向二百三十一 | 成立 | 两个服务 `src/` 分进 `SWAP_ONLY_SOURCE_DIRECTORIES` 在 693 行、`SWAP_ONLY_EXCLUDED` 在 692 行（正文写 693、694，差一行）；源码分支在 738-750 行；开关 `script-in-place-allowed` 现点名十三格（209-216 行），调度:242 的 19717 记「逐格相同 2 条」 |
| `.claude/hooks/bash-command-detector.sh` | 二百二十二 账本:319-320；一百九十九 账本:163；第 7500 行指针指向一百七十四 账本:95-102 | 成立 | 十处「还没复跑」改成现状的依据是调度:81（五条新开关与五条受牵连开关逐格相同）；同类句 626、635 两行「照判法推的、还没复跑」没动（那是进程组三条开关，工具员报告写明没动），仍留着 |
| `.claude/hooks/lib_project_code_runs.py` | 二百二十二 账本:320；二百三十六 尾巴 账本:496；1103 行 → 一百七十六 ⑭ 账本:121；1105 行 → 一百九十九 账本:157 | 成立 | 1107 行登 make-swap-spec.py 与账本:496「真换 19697」一致；混改 187、1127-1129、3551 行（`serves`）是别的会话的，没判 |
| `.claude/hooks/project-code-guard.sh` | 二百二十二 账本:320；指针（206、1717 行）→ 一百七十四 账本:95-102 | 成立（带一处账本没点名的多做） | 196-198 行名单抄写改成「只写在 lib_project_code_runs.py 里，这里不抄」：账本、调度:242 都没点这一行，依据只在 `~/.local/share/singlefs/work/ledger-topdown-sweep-apply-tooling/result.md` 第 7 小节（文件头抄的名单已缺多项）；新开关 `active-sessions-unlisted`、`perl-bare-options-take-next-word` 的说明写「推的，没跑」，调度:110 认开关复跑没判 |
| `.claude/hooks/handback-guard.sh` | 账本没写；调度:235、调度:238 说清（回扫判出引封存账本节号的注释） | 成立 | 第 43 行指到的一百七十四节在现行账本账本:95-102 找得到 |
| `.claude/hooks/lib_run_scope.py` | 同上，第 714 行 | 成立 | 无差 |
| `.claude/hooks/run-scope-guard.sh` | 第 31 行 → 一百七十四 账本:95；第 70 行 → 二百二十九 账本:426 | 成立 | 无差 |
| `.claude/hooks/runner-dispatch-guard.sh` | 账本没写；调度:238 | 成立 | 2839 行是拒绝说明串，人看到的出路路径已是现行账本 |
| `.claude/hooks/selftest-registry.tsv` | 一百七十六 ① 账本:131-132；二百三十六 尾巴 账本:496 | 成立 | 103 行（e56 入口）、130 行（规格工具）如表；混改第 52 行 `crash-library-totals.sh` 与第 24 行 `agent-watch.py` 行内差别是别的会话的 |

### 三、门禁（`.claude/gate.d/`）与样本

| 路径 | 账本节与行号 | 判定 | 差在哪 |
|---|---|---|---|
| `.claude/gate.d/code-tooling.sh` | 二百三十六 尾巴 账本:496；一百七十六 ① 账本:132；二百一十九 账本:258；指针 9597 行 | 成立 | runner 表加两行如表；2248 行 `crash-library-totals.sh` 是别的会话的；130-706 行一带大段别的会话的，没判 |
| `.claude/gate.d/break-switch-replay-files.tsv` | 二百三十六 尾巴 账本:496；二百三十一 账本:449；一百七十六 ① 账本:131 | 成立 | 撤掉 hook-swap.py 一行账本没写，依据调度:105、:111；账本:449 仍写「三格没点开关复跑」而调度:242 的 19717 已复跑，账本这一句过时（账本侧欠改） |
| `.claude/gate.d/comment-history-lag.tsv` | 二百零一 ③ 账本:183；二百三十七 账本:518 | 成立 | 归属栏写反：正文说 prune-server 一带七行是别的会话的，但账本:183 与调度:119 把「prune-server 五行改小四行、删一行」记作这一轮的（任务 19154），diff 里恰是这 5 行（bin/prune_server.rs 9→8、protocol.rs 6→2、reliable_client.rs 9→7、storage.rs 3→2，删 selftest_library_layout_cells.rs）。账本:183 写「35 行、869 处」，此刻现数 35 行、868 处（doc-process-records.sh 39→38 在其后） |
| `.claude/gate.d/doc-text.sh` | 二百三十七 第 7 件 账本:510、账本:518 | 成立 | 646、656-658、741、760、972、976 行俱在；开关表「十三条」与 `scripts/gate/comment-history-check.py:59` 「前十三条」相同 |
| `.claude/gate.d/crash-point-replay.sh` | 一百九十四 账本:142；调度:254、:263、:266 | 成立 | D 节逐项核；混改 387-390、769-770、1196-1209 行是别的会话的 |
| `.claude/gate.d/doc-process-records.sh` | 账本没写（回扫）；调度:235、:242 | 成立 | 457-458 行改成现状（去掉用户定的日期引文）。602、616、619 行的样本说明仍把 2026-10-29 那份叫「现行账本」：工具员判为样本世界里的话没改（result.md 第 1 小节），真仓读者会读成现行（C 节第 3 类） |
| `.claude/gate.d/stage-owners.tsv` | 账本没写（回扫）；`ledger-topdown-sweep-apply-tooling/result.md` 第 3 小节说清 | 成立 | 139 行「今天是」后路径换成现行账本，与 `.claude/gate.d/split-proposal-ledgers.tsv:6` 的 current 行相同 |
| `.claude/gate.d/replay-comparison-units.tsv` | 二百三十七 第 5 件 账本:508；调度:153、:243 | 成立 | 加 E201 比对对象（111 行）的依据是调度:153「门禁要的比对登记等 replay.sh 的改动换上之后再加」；删 18、19 行两条 e183-r1 豁免注释的依据是那份登记已随提交被清掉（调度:243），账本没写这一删；调度:243 记「删了之后没记复跑」 |
| `.claude/gate.d/fixtures/doc-text.sh/comment-history-past-red/`（未跟踪 3 份） | 二百三十七 第 7 件 账本:510、:518 | 成立 | 样本两行：「键要以前缀起头」不算、下一行「以前这里只认一种写法」判红，expect 点名样本里 `scripts/demo/run.sh` 的第 3 行，与账本:518「别的「以前」照判」同义 |
| `.claude/gate.d/fixtures/doc-text.sh/comment-history-prefix-green/`（未跟踪 3 份） | 同上 | 成立 | `.rs` 注释写「以前缀」「除以前者」「所以前面」「除以前一项」，expect 命中 0 行 |

### 四、脚本：门禁与工具

| 路径 | 账本节与行号 | 判定 | 差在哪 |
|---|---|---|---|
| `scripts/gate/hook-swap.py` | 二百三十一 账本:446-449；二百三十六 ①④⑤ 账本:486-493；二百四十（1862、1878 行补出路）账本:549 | 成立（带一处账本没有专节的多做） | 198 行六处目录、215 行退 8、216 行退 9、602 行 `neutral_feeds_failing_on_base`、26 行指规格工具，都能对到账本；796、856 行「按字符串键认调用方引到的库」账本没有一节，依据调度:178、调度:217（把另一份库的名字算到 lib_function_units.py 头上，挡住算键库换上，对应账本:626）；文件头补点名三格「补完没再复跑」（调度:217 自认）；正文账本节栏漏写二百四十 |
| `scripts/gate/make-swap-spec.py` | 二百三十六 ④ 账本:489、:493、:496 | 成立 | 文件头用法（9 行）与定义、写闸出路里写的 `--base --new --out <相对路径>…` 同形 |
| `scripts/gate/comment-history-check.py` | 二百三十七 第 7 件 账本:510、:518 | 成立 | 新增的 `BEFORE_PATTERN`（`(?<!除)以前(?![缀者面])`）与账本:518「前面紧挨「除」、或后面紧跟「缀」「者」「面」」一字不差；开关 `prefix-compounds-flagged` 与自证格 `prefix-compounds` 俱在 |
| `scripts/gate/README.md` | 二百三十六 尾巴 账本:496；调度:242 | 成立 | 无差 |
| `scripts/agents/agent-watch.py` | 二百三十六 ③ 账本:488 | 成立 | 只判 591 行 `UNJUDGED_DETECTION_PREFIXES` 里「时刻写法闸没判、照常放行」一项；695 行里其余是别的会话的，没判 |
| `scripts/agents/dispatch-prompt.py` | 账本没写；调度:235、:238 | 成立 | 300 行指针，指到的一百七十四节账本:95 找得到 |
| `scripts/kb/stale-candidates-record-carriers.tsv` | 账本没写；调度:219 说清（回扫员点出登记表仍登封存账本） | 成立 | 现在只登现行账本一行，三份封存的都没了；封存的 sha256 现算与登记表逐个相同（见第七类 split-proposal-ledgers.tsv） |
| `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` | 二百三十七 第 5 件 账本:508、:518 | 成立 | 第 57 行换成「判法 E171 目录不进 import 路径」，与账本:518「换成「判法 E171 目录不进 import 路径」」同名 |

### 五、脚本：实验装置

| 路径 | 账本节与行号 | 判定 | 差在哪 |
|---|---|---|---|
| `scripts/experiment-apparatus/replay.sh` | 二百一十四 账本:246；二百一十八 账本:255；二百一十九 账本:268；E201 账本:508、:518；OUT_DIR 二百四十 账本:549；内联 python 二百五十一 ⑦ 账本:646 | 成立（三处没有专节的多做，依据在调度） | 一，「四处调用写全路径、`python3 -B "$@"`」此刻 `git diff HEAD` 里没有（已随 HEAD 的 2026-11-17 提交，`git show HEAD:` 3852 行有），正文把它列进改动。二，OUT_DIR 拆两步正文写「没写」，账本:549 写了。三，内联 python 整段删与 host-cap：账本:268 写了 host-cap 与三条新开关，没写内联 python，原因在调度:141、:158、:161、:167（import 拒算让 61 条开关起不了，账本:646）。四，`threads-bound-kept` 样本仓取法设成只读函数账本无节，依据调度:249、:255；账本:268 还写「三条新开关没验」，调度:257 的 19775 已记「逐格相同 60 条」，账本这句过时 |
| `scripts/experiment-apparatus/replay_row_costs.py` | 二百一十九 账本:268 | 成立 | 正文账本节栏写「没写」，其实账本:268 点名了「replay_row_costs.py 的 read 多打的 host-cap 一行」与 `host-cap-unreported` 等开关 |
| `scripts/experiment-apparatus/e184-segment-one-long-run.sh` | 二百一十八 账本:255 | 成立 | 534、538 行 `clip_text 400`；`clip_text` 定义在 104 行（已在 HEAD） |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py` | 二百零一 ③ 账本:183 | 成立（少改一处） | 6、7、307 行文档串改成「基准名单」「现行名单」，但 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py:416` 的 `--reference-commit` help 字符串仍写「改之前的名单」，与改过的说法不一致（字符串不是注释，门禁不判） |
| `scripts/experiment-apparatus/e185_r4_baseline_rejudge.py` | 二百零一 ③ 账本:183 | 成立 | 无差 |
| `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh` | 二百零一 ③ 账本:183 | 成立 | 无差 |
| `scripts/experiment-apparatus/long-run-lib/binary-identity.sh` | 二百零一 ③ 账本:183 | 成立 | 无差 |
| `scripts/experiment-apparatus/README.md` | 一百七十六 ① 账本:131；二百一十九 账本:268 | 成立 | 无差 |
| `scripts/experiment-apparatus/e56-vm-bench-run.sh`（未跟踪） | 一百七十六 ① 账本:131、:132 | 成立 | 391 行（`wc -l`）；文件头讲的构建 musl 静态 `e56_epsilon`、交 `scripts/devices/vm-bench.sh`、编译目录由环境给，与账本:131 同；账本:132 欠的 runner 表一行已在 `.claude/gate.d/code-tooling.sh:2230` |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/test_full_table_run.py` | 二百三十七 第 5 件 账本:508、:518 | 成立 | 无差 |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/current_tree_row_driver.py` | 二百三十七 账本:518 | 成立 | 只改 60-63 行注释，代码不动；注释点名的算键库开关 `python-imported-sys-path-ignored` 在 `scripts/gate/lib_function_units.py:708` 存在 |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/test_current_tree_row_driver.py` | 二百三十七 账本:518 | 成立 | 无差 |

### 六、任务服务

| 路径 | 账本节与行号 | 判定 | 差在哪 |
|---|---|---|---|
| `task-server/task_server.py` | 二百零五 账本:205 | 成立（只做前半） | 读日志行首 `NOT_RUN_NO_GPU_ADAPTER`（890-895 行常量、1116 行 `gpu_not_run_test`、4440 行、6025 行）如表；账本:205 自认只罩一半（不带 `--nocapture` 读不到）；`task-server` 下全部 `.py`、README、`task` 对 `nocapture` 的 grep 命中 0，这个缺口没有写在任何一处 |
| `task-server/task_server_lib.py` | 二百零五 账本:205 | 成立 | 开关 1507-1508 行、异常表 1798 行、提醒种类 2128 行俱在 |
| `task-server/task_server_selftest.py` | 二百零五 账本:205 | 成立 | 新格 `gpu-cases-not-run-reported` 在 9638-9703 行、登记在 14695 行 |
| `task-server/README.md` | 二百零五 账本:205、账本:384 | 成立（少写限制） | 149 行写了认 `gpu-cases-not-run`、种类数十七；没写「不带 `--nocapture` 的点名测试读不到」；账本:384 另写 README.en、README.ja 没同步（它们不在这 52 份里） |

### 七、已随 2026-11-17 提交进 HEAD 的

| 路径 | 账本节与行号 | 判定 | 差在哪 |
|---|---|---|---|
| `.claude/hooks/lib_shell_words.py` | 二百二十二 账本:320 | 成立 | `git show HEAD:` 里 `perl-bare-options-take-next-word` 命中 3 处，`git diff HEAD` 空 |
| `scripts/agents/active-sessions.py` | 二百二十二 账本:319-320 | 成立 | 530 行（`wc -l`），`selftest_cells` 在 399 行；`git diff HEAD` 空 |
| `scripts/agents/README.md` | 二百二十二 账本:320 | 成立 | 16 行一行 `active-sessions.py` |
| `task-server/control` | 二百二十四 账本:384 末段「服务文件里引旧账本的 6 处换成现行账本」 | 成立 | 正文写「改了第二百二十四节两处」不准：工具员报告（`ledger-topdown-task-server-debts/result.md`）里 control 是「diff 加 2 删 2」，是 36、606 行两处账本指针，不是二百二十四节的两条；指针指向八十节 ①④ 回看，账本:87、:89 找得到 |
| `.claude/gate.d/split-proposal-ledgers.tsv` | 换账本 调度:64、调度:65 | 成立 | current 行指现行账本（6 行），三份 sealed 行的 sha256 与现算 `sha256sum` 逐个相同 |

A 小结（量的：本节 52 行）：成立 51 行（其中判定栏带括注的 6 行：多做三处、只做前半一处、少写限制一处、少改一处），不成立 1 行（`.claude/agents/implementation-writer.md`），推不出 0 行。差的几类：账本侧过时三处（账本:449、账本:268 尾巴、账本:183 的数）；正文表错四处（comment-history-lag.tsv 归属、task-server/control 的「二百二十四节两处」、write-guard.sh 行号差一、replay.sh 的「OUT_DIR 没写」与 python3 -B 在 HEAD）；账本没有专节的多做三处（hook-swap.py 字符串键、project-code-guard.sh 196-198 行、replay.sh 三处），都有调度或工具员报告说清为什么改。什么现象会推翻：任何一行的 `git diff HEAD -U0` 里出现表里没点到、账本节也不管的改动，或我引的账本行号上的字不是我写的那句。

## B 正文第 7 问：定义与共用约束说的做法，与写闸、换上工具、证红脚本今天的行为是不是一回事

逐句对到 文件:行。判定：一致、打中（照做会被拒或回滚）。

| 句 | 对到的脚本或钩子 | 判定 |
|---|---|---|
| `.claude/agents/tooling-writer.md:31` 「写闸第十一道执行前拒 Write / Edit」 | `.claude/hooks/write-guard.sh:706` 的 `decide_script_swap_only`：Write、Edit、MultiEdit 都判；目录表 692-693 行；返回退 2 的拒绝：源码分支 745-749 行、脚本分支 752-758 行 | 一致 |
| 同行 规格用 `make-swap-spec.py --base <基线根> --new <草稿目录里的新版根> --out <草稿目录>/<规格.json> <相对路径>…` 生成 | `scripts/gate/make-swap-spec.py:9` 用法同形；`scripts/gate/make-swap-spec.py:15` 「--out 要落在工作根下」，草稿目录在工作根下；子 agent 能直接起：`.claude/hooks/lib_project_code_runs.py:1107` 「"scripts/gate/make-swap-spec.py": "any"」 | 一致 |
| 同行 先 `--dry-run`、再换上，不带 `--hook` 与 `--feeds`，「`--verify` 给被换脚本的 `--selftest`」 | `scripts/gate/hook-swap.py:11` 用法行同形；验证命令由 `scripts/gate/hook-swap.py:1421` 的 `bash -c` 起。要临时判定库服务的自证（`.claude/hooks/selftest-temporary-service.tsv:38` 的 replay.sh、`.claude/hooks/selftest-temporary-service.tsv:60` 的 prove-red.sh 等登记成 required）在这条验证命令里没有包装，账本:772 自认「在这一件修好之前照写会在本机被回滚」，账本:769 量到「派到本机的那一趟退 5（任务 19767）」 | 打中（已登记在账本，不是闸拒，是验证命令退 2、换上工具回滚；另：命令位置上裸写 required 自证的 `--selftest`，项目代码闸在执行前拒，见 `.claude/hooks/selftest-temporary-service.tsv:7` 「裸写的 --selftest 在执行前被 project-code-guard.sh 拒」，写在 `--verify` 引号里的那一份闸认不认没读到，推不出） |
| 同行 两个服务的 `src/` 「在仓副本里改（建法见共用约束「建仓副本」）」，存成补丁、打补丁命令写进报告 | 拒绝说明 `.claude/hooks/write-guard.sh:748` 同一句出路；副本里有没有 `prune-server/src` 与 `gpu-verification-server/src`：仓根 `Cargo.toml` 的 18 行把 `gpu-verification-server` 列为工作区成员、`crates/singlefs-verification/Cargo.toml:33` 对 `prune-server` 有 path 依赖，`scripts/hosts/lib_repository_copy.py` 文件头写 path 依赖与工作区成员都进闭包，所以默认闭包里有（推的，我直接起 `lib_repository_copy.py entries` 被权限分类器拒了，待主 agent 跑，命令 1） | 一致（推的） |
| `.claude/agents/tooling-writer.md:35` 「范围声明的范围数是 `--check` 的格数加 `--item` 的项数」 | `.claude/hooks/run-scope-guard.sh:83` 「N = --check 点名的格（逗号隔的逐个算）与 --item 点名的项」；`.claude/gate.d/code-tooling.sh:12395` 「--item <项>：只跑点名的项（给几次取并集）」，三格分项里有 break-switch-replay | 一致 |
| 同行 这一格开跑前算键、有拒算就整格判红，「键算不出的脚本（例 `scripts/gate/hook-swap.py`）登不进那张表」 | `.claude/gate.d/code-tooling.sh:1707` 「这一格当场判红、一条都没起」、`.claude/gate.d/code-tooling.sh:1714` 「拒算没有登记表可豁免，这一格只认拒算 0」；`scripts/gate/hook-swap.py:1421` 仍是 `subprocess.run(["bash", "-c", command]`（命令是变量，键折不出）；`.claude/gate.d/break-switch-replay-files.tsv` 里现在没有 hook-swap.py 的行 | 一致（hook-swap.py 今天仍拒算是推的：读码加调度:105，没重量闭包，命令 2） |
| 同行 没登记的脚本「把要加的那一行原文写进报告、由主 agent 先加」 | `.claude/gate.d/break-switch-replay-files.tsv:7` 「加一行之前先把那份脚本的每一条开关设上跑一遍它的 --selftest」：表头要求先验后加，定义要求先加后用 `--item` 验；没有闸拒这个次序（表头是文字、门禁只判已在表里的行） | 推不出（次序说法对不上、但没有任何闸会拒；算打中，轻） |
| `.claude/main-agent.md:99` 「把草稿里的新版经 `scripts/gate/hook-swap.py` 一次换上」，已存在的脚本与钩子都这样换，两个服务 `src/` 下的打补丁 | `scripts/gate/hook-swap.py:198` 脚本规格六处目录，钩子走 `--hook`（文件头 11 行以上两种规格）；两个服务 `src/` 不收：文件头「src/（Rust 源码，在仓副本里改、存成补丁由主 agent 一次打上）」；写闸 692-693 行同分 | 一致 |
| `.claude/agent-common.md:44` 「仓根四份被忽略的私有清场脚本」 | 仓根 `multi-host-quiesce-*.sh` 恰 4 份（`ls` 数）；`git check-ignore -v` 命中 `.gitignore:22` 的 `/multi-host-quiesce-*.sh`；`git ls-files` 数 0 份 | 一致 |
| `.claude/agent-common.md:100` 「或两个服务 `src/` 下已存在的文件（不限后缀）」，两个服务 `target/` 除外，出路规格由 `make-swap-spec.py` 生成 | `.claude/hooks/write-guard.sh:738` `source = directory in SWAP_ONLY_SOURCE_DIRECTORIES`、739 行 `if not source and not relative.endswith(SWAP_ONLY_SCRIPT_SUFFIXES)` 源码不看后缀与执行位；`.claude/hooks/write-guard.sh:692` target 除外；拒绝说明里点名 make-swap-spec（753 行起） | 一致 |
| `.claude/agents/implementation-writer.md:32` 「这两条都点名这一次改到的包：`-p <包>`，不点名包的服务当场拒」 | `task-server/task_server.py:7268` 「没点名包（-p / --package），起的目录认不出」及同一函数 7253 行 `whole_workspace_cargo_refusal`；出路句 `task-server/task_server.py:973` 「点名要编的包：-p <包>」 | 一致 |
| `.claude/agents/implementation-writer.md:44` 副本上的证红照第 3a 步 `task-server/task submit --cwd <仓根> -- bash scripts/mutation/prove-red.sh --copy <副本> <crate> <变异名…>` | `scripts/mutation/prove-red.sh:271` 「MAIN_ROOT = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))」：主工作区 = 起的那份脚本所在的仓根；从仓根起主工作区那一份、`--copy` 指副本，`copy != main_root` 过 `scripts/mutation/prove-red.sh:962`；范围声明闸读 `--copy` 点名的副本里的登记（`.claude/hooks/run-scope-guard.sh:63-66`）；账本:640 量到「改成从仓根交、`--copy` 指副本的 19255、19442 退 0」 | 一致 |
| `.claude/agents/implementation-writer.md:48` 缩域证红 「task-server/task submit --cwd <副本> -- env <缩域变量>=<n> bash scripts/mutation/prove-red.sh --copy <副本>」 | 同一行前半句自己写「起副本里那一份时它把副本认成主工作区、退 2」；这一条 `--cwd <副本>` 起的正是副本里那份脚本：`scripts/mutation/prove-red.sh:271` 取 MAIN_ROOT 为副本、`scripts/mutation/prove-red.sh:962` 「if copy == main_root and BROKEN != "main-allowed":」成立，`scripts/mutation/prove-red.sh:963` 退 2 「✗ --copy 指的是主工作区」；账本:640 「在副本里起副本自己那一份脚本时副本就等于主工作区」 | 打中：照做被 prove-red.sh 自己的判断拒（退 2），判据在 962 行；账本:648 的去向只改了第 1 步与第 3a 步首条，缩域那条漏了 |

B 小结：八句里一致六句；打中两处：`.claude/agents/implementation-writer.md:48` 缩域证红命令（被 prove-red.sh 自己退 2）；`.claude/agents/tooling-writer.md:31` 的 `--verify` 写法（已登记在账本:772，写成 required 的自证没有临时服务包装，在本机被回滚）。一处推不出（`.claude/agents/tooling-writer.md:35` 先加表后验与表头先验后加的次序，没有闸拒）。同批外还见一句对不上：`.claude/agents/investigator.md:33` 「证红的 `prove-red.sh` 不认主工作区，它的 `--cwd` 与 `--copy` 填改完之后」——investigator 这句让 `--cwd` 也填副本，与 implementation-writer 第 44、48 行首条的 `--cwd <仓根>` 相反，照 investigator 写同样退 2（判据同 `scripts/mutation/prove-red.sh:962`）；它不在被判的 52 份里，只记下。另 `.claude/agents/implementation-writer.md:47` 仍写副本建时要带 `--with scripts=<为什么>`「第 3a 步的 `scripts/mutation/prove-red.sh`…同在这一份 `scripts/` 里」，3a 改成仓根起之后证红脚本不需要在副本里，这一句多余但不拒（推不出有害）。什么现象会推翻：主 agent 在副本里起缩域那条命令得到退 0（则 prove-red.sh:962 的判据我读错）；`lib_repository_copy.py entries` 的输出里没有 prune-server 或 gpu-verification-server（则 `.claude/agents/tooling-writer.md` 第 31 行的副本说法不成立）。

## C 正文第 8 问：指向封存账本的路径，十类路径里还有没有漏换

范围：`.claude/agents`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules`、`.claude/skills`、`.claude/hooks`、`.claude/gate.d`、`scripts`、`task-server`、`prune-server`。三份封存账本：`records/2026-09-16-subagent拆分提案.md`、`records/2026-10-23-subagent拆分提案.md`、`records/2026-10-29-subagent拆分提案.md`。

搜的命令（量的，输出落在 `~/.local/share/singlefs/work/governance-r1-forward/sealed-hits.txt`）：

```
grep -rnE "2026-09-16-subagent拆分提案|2026-10-23-subagent拆分提案|2026-10-29-subagent拆分提案" .claude/agents .claude/agent-common.md .claude/main-agent.md .claude/rules .claude/skills .claude/hooks .claude/gate.d scripts task-server prune-server --exclude-dir=target --exclude-dir=__pycache__
```

命中 149 行（`wc -l`）。按目录数：`.claude/agents` 0、`.claude/agent-common.md` 0、`.claude/main-agent.md` 1、`.claude/rules` 0、`.claude/skills` 0、`.claude/hooks` 31、`.claude/gate.d` 102、`scripts` 15、`task-server` 0、`prune-server` 0（加起来 149）。其中 88 行在样本目录（`.claude/gate.d/fixtures/` 83、`.claude/hooks/fixtures/` 5：样本仓里自带的账本与冻结的真提示，不是指令）；其余 61 行、28 份文件逐条判如下（行号是读时现取）。

| 类 | 行 | 判定 |
|---|---|---|
| 来历出处：为什么、依据、欠账出处，括号里写封存账本某节 | 46 行：`.claude/hooks/agent-write-scope.tsv:13`；`.claude/hooks/ask-user-claim-guard.sh:13,29`；`.claude/hooks/bash-command-detector.sh:118,236,322,326,345,1351,1875`；`.claude/hooks/completion-cleanup-guard.sh:543`；`.claude/hooks/continuation-guard.sh:78,114`；`.claude/hooks/lib_project_code_runs.py:1104`；`.claude/hooks/lib_run_scope.py:2478`；`.claude/hooks/project-code-guard.sh:15`；`.claude/hooks/runner-dispatch-guard.sh:149,2232`；`.claude/hooks/run-scope-guard.sh:301`；`.claude/hooks/write-guard.sh:11,14,30,75`；`.claude/gate.d/code-source-discipline.sh:3815`；`.claude/gate.d/code-tooling.sh:3430,4053,11191`；`.claude/gate.d/crash-point-replay.sh:265,366`；`.claude/gate.d/doc-decisions.sh:3835`；`.claude/gate.d/doc-experiments.sh:3342`；`.claude/gate.d/doc-process-records.sh:360`；`scripts/agents/agent-watch.py:910,2537`；`scripts/agents/cite-check.py:48`；`scripts/agents/snapshot-tree.py:13`；`scripts/gate/admission.py:230,8606`；`scripts/gate/merge-and-verify.sh:224`；`scripts/hosts/cache-keepalive.sh:11`；`scripts/hosts/capped.sh:7`；`scripts/hosts/run-with-memory-cap.sh:103,112,306`；`scripts/kb/stale-candidates.py:173`；`scripts/mutation/crates-mutation-rows.py:1363` | 不算漏换。这些句子说的是「这条规矩从哪来」，节在封存那份里。机器核了其中 45 处「第 N 节」写法（`~/.local/share/singlefs/work/governance-r1-forward/section-check.py` 的输出 `section-check.txt`）：43 处被引的封存账本里有 `## N、` 标题；这 43 个节号在现行账本里都没有同号的节（`current-check.txt` 末行：同号在现行账本里也有 0、只在封存那份 43），想改指现行账本也无处可指 |
| 说「封存几份、现行哪份」的登记 | `.claude/gate.d/doc-process-records.sh:457,458`；`.claude/gate.d/split-proposal-ledgers.tsv:7,8,9` | 对：登记表就该点名封存的三份；457-458 行本轮已写成现状 |
| 自证样本串、格式示例、样本说明 | `.claude/hooks/ask-user-claim-guard.sh:198,227,228`（喂钩子的样本问句）；`.claude/gate.d/doc-process-records.sh:265`（回扫记录「命中处置」表的格式示例行）；`.claude/gate.d/doc-process-records.sh:602,616,619`（样本说明里写「现行账本 records/2026-10-29-subagent拆分提案.md」，说的是样本仓里自带的登记表） | 不算漏换，但 602、616、619 行有读错风险：真仓里 2026-10-29 那份已封存，读者读到「现行账本」会读成真仓的现行。工具员判为样本世界里的话没改（`ledger-topdown-sweep-apply-tooling/result.md` 第 1 小节），我只记风险，不替它翻案 |
| 指来历、带「别读它」的指针 | `.claude/main-agent.md:3` 「records/2026-09-16-subagent拆分提案.md（除非要查来历，别读它）」 | 不算漏换，调度:232 说明不改的理由（来历确实在那一份） |
| 坏指针：被引的节在被引的封存账本里没有 | `.claude/gate.d/crash-point-replay.sh:294`（引 2026-10-23 份「第一百四十三节」）；`scripts/hosts/run-with-memory-cap.sh:73`（引 2026-10-23 份「第一百三十七节」） | 两处节号在被指的文件里找不到 `## 一百四十三、`、`## 一百三十七、`（`grep -n '^## 一百三十七\|^## 一百四十三' records/2026-10-23-subagent拆分提案.md` 0 命中）。前一处引的标题「门禁一格里前一趟的结局等整格收尾才写」在 `records/2026-10-29-subagent拆分提案.md:702` 的节标题里，号已是一百七十；后一处在 2026-10-29 份里同号 531 行是别的题（欠账清理那一轮留下的），读不出对应哪一句，推不出。算坏指针，不算漏换 |

路径类漏换：0 处（没有一句话叫读者往封存账本写、或把封存账本当现行读）。同一批里不带路径的「拆分提案第 N 节」写法另有 159 行（`grep -rnF "拆分提案" …` 去掉带日期路径的行，`broad-nonfixture.txt`），主要在 `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`：它们没带路径，读不出指哪一份，工具员也明说没改（result.md 第 1 小节末句）；不是路径，不在正文这一问的字面里，只记数。

### 已换成现行账本的句子：抽 14 句核「说的事在现行账本里找不找得到」

这十类路径里指向现行账本的共 28 行（`grep -rn 'subagent账本自上而下清欠' …`，28 行都不在样本目录），其中 18 行是这一轮 `git diff HEAD` 新加的。抽样如下（账本 = `records/2026-11-17-subagent账本自上而下清欠.md`）。

| 那一句（文件:行） | 在现行账本里 | 判定 |
|---|---|---|
| `.claude/hooks/handback-guard.sh:43`：「子 agent 一律不往统一任务服务交任务」 | `records/2026-11-17-subagent账本自上而下清欠.md:98`：「即子 agent 一律不经 `task-server/task submit` 交任务」 | 找到；一百七十四节在 95-102 行 |
| `.claude/hooks/runner-dispatch-guard.sh:2839`：「用户 2026-11-01 定」 | `records/2026-11-17-subagent账本自上而下清欠.md:98`：「用户定（弹窗）」；日期在节标题（95 行）里 | 找到 |
| `.claude/hooks/bash-command-detector.sh:7499`：「task wait 也不起」 | 一百七十四节 95-103 行里 `task wait` 0 命中（`sed -n 95,103p … \| grep -c 'task wait'` 为 0）；这半句的出处是 `.claude/agent-common.md` 的「子 agent 不交任务」 | 部分：「不往任务服务交任务」找到，「task wait 也不起」账本这一节没写 |
| `.claude/hooks/write-guard.sh:88`：「中间态对别的会话可见」 | `records/2026-11-17-subagent账本自上而下清欠.md:443`：「中间态对别的任务可见」 | 找到（二百三十一节） |
| `.claude/hooks/lib_project_code_runs.py:1103`：「出路就是它」 | `records/2026-11-17-subagent账本自上而下清欠.md:121`：「出路写经 `scripts/kb/append-record.py`」 | 找到（一百七十六节 ⑭；句里的「⑲」是检出钩子的编号，账本没写，不影响） |
| `.claude/hooks/lib_project_code_runs.py:1105`：「副本拷贝不全的问题必须根治」 | `records/2026-11-17-subagent账本自上而下清欠.md:157`：「副本拷贝不全的问题必须根治」 | 找到，逐字 |
| `.claude/hooks/run-scope-guard.sh:70`：「收口要做的」 | `records/2026-11-17-subagent账本自上而下清欠.md:426`：「收口要做的（会红的检查，不是提醒）」 | 找到（二百二十九节） |
| `.claude/gate.d/code-tooling.sh:9597`：「按核开满」 | `records/2026-11-17-subagent账本自上而下清欠.md:261`：「脚本按核开并行，服务照实测给内存」 | 找到（二百一十九节） |
| `task-server/control:36`：「八十节 ①④ 的回看数数用」 | `records/2026-11-17-subagent账本自上而下清欠.md:87`：「到期回看」 | 找到；八十节 ①④ 的去向在 87-93 行，93 行写了 `control key-stats` |
| `.claude/gate.d/stage-owners.tsv:139`：「撞号、索引缺行由主 agent 照红行改号、补索引行」 | `.claude/gate.d/split-proposal-ledgers.tsv:6`：「新欠账与优化项往这一份末尾追加」；账本 9 行起是「## 索引」 | 找到（在登记表与索引，不在某一节） |
| `.claude/main-agent.md:25`：「子 agent 与协作工具的事登进」 | `records/2026-11-17-subagent账本自上而下清欠.md:3`：「这一份是 subagent 与协作工具的现行账本」 | 找到 |
| `.claude/agent-common.md:4`：「实测、经过与带日期的记录进」 | `records/2026-11-17-subagent账本自上而下清欠.md:484`：「经过在 `records/2026-11-17-subagent账本清欠调度记录.md`」；`records/2026-11-17-subagent账本自上而下清欠.md:7`：「这一轮清欠的课题、出口与依赖表在」 | 部分：账本装现象、根子、去向，「经过」在调度记录里（这一句在 HEAD，不在本轮 diff） |
| `.claude/agents/tooling-writer.md:20`：「的节号与条号」 | 账本里有「## 二百二十四、」下的第 1 到 31 条 | 找到 |
| `scripts/kb/stale-candidates-record-carriers.tsv:3`：「现行账本，装着开着的欠账」 | 账本 9 行起的「## 索引」表，判定列里有开着的节 | 找到 |

C 小结：路径类漏换 0 处；坏指针 2 处；样本说明读错风险 3 行；已换成现行账本的 14 句抽样：找到 12、部分 2（`.claude/hooks/bash-command-detector.sh:7499`、`.claude/agent-common.md:4`）。什么现象会推翻：另有一份治理文件里出现不带日期、不带 `拆分提案` 字样的旧路径写法（例如只写 `records/subagent…`）；我的搜法对这种写法没有覆盖，`grep -rnF "subagent拆分提案"` 去掉三个带日期名字后只剩样本登记表里的假名字（2 行）。

## D 正文第 10 问：门禁注释与 crates 今天的登记、与滤日志行的代码是不是一回事

三处各给 文件:行 与原文（读时现取）。

注释（门禁文件头，本轮改的，314-318 行共五行）：

- `.claude/gate.d/crash-point-replay.sh:314`：「逐提交的行（行首是 GPU_LIFECYCLE 或 CRASH_JUDGE_ACCOUNTING_SLICE，GPU 判器每次提交、每片各打）不进日志，只留计数与末尾现场」
- `.claude/gate.d/crash-point-replay.sh:315`：「两种行在 crates 里的出处不同：GPU_LIFECYCLE 行由 gpu_lifecycle_line、gpu_lifecycle_trace、gpu_work_trace 三个单元打，它们登记成 GPU 判器那一组不沿它收的单元」
- `.claude/gate.d/crash-point-replay.sh:316`：「（crates/singlefs-verification/src/judging_units_entries.rs 的 UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW），改它们不换那一组的键；」
- `.claude/gate.d/crash-point-replay.sh:317`：「CRASH_JUDGE_ACCOUNTING_SLICE 行由 GpuCrashJudge::judge_one_dispatch 打，它是 GPU 判器那一组的入口、没登记成不沿它收，改这一行的字就换那一组的键。」
- `.claude/gate.d/crash-point-replay.sh:318`：「一行里夹着判法在行中间认的记号」，后面接「照进日志，计数里写」「夹了别的记号」（原文在同一行）

crates 今天的登记与出处：

- `crates/singlefs-verification/src/judging_units_entries.rs:1178`：「pub const UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW:」；内容是 1180、1181、1182 行三个名字 `singlefs_verification::gpu_unit_checks::gpu_lifecycle_line`、`…::gpu_lifecycle_trace`、`…::gpu_work_trace`；1213 行 「.chain(UNITS_THAT_ONLY_PRINT_THE_GPU_LIFECYCLE_LINE_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW)」 并进 `gpu_judge` 组的不沿它收清单。
- `crates/singlefs-verification/src/judging_units_entries.rs:68`：「"singlefs_verification::crash_judge_dispatch::GpuCrashJudge::judge_one_dispatch",」，在 `gpu_judge` 组的 `entry_functions` 里（入口），1178-1182 行与别的两张不沿它收的清单里都没有它（`grep -n judge_one_dispatch` 在这个文件只命中 29 行的注释与 68 行）。
- 打行的单元：`crates/singlefs-verification/src/gpu_unit_checks.rs:1052` 「"GPU_LIFECYCLE unix_microseconds={unix_microseconds}」 在 1044 行 `fn gpu_lifecycle_line(` 里拼，1007、1018 行的 `gpu_lifecycle_trace`、`gpu_work_trace` 写出去；`crates/singlefs-verification/src/crash_judge_dispatch.rs:1767` 「"CRASH_JUDGE_ACCOUNTING_SLICE card={:?}」 在 1633 行 `fn judge_one_dispatch(` 里（1633 之后到 1767 之间没有别的 fn，`awk` 数过）。全 `crates/` 里 `GPU_LIFECYCLE` 的非注释字面只有 `gpu_unit_checks.rs` 1052 与测试里 3162 两处。

滤日志行的代码：

- `.claude/gate.d/crash-point-replay.sh:1533`：「BULK_PREFIXES = (b"GPU_LIFECYCLE ", b"CRASH_JUDGE_ACCOUNTING_SLICE ")」：两个前缀都带尾随空格，所以 `CRASH_JUDGE_ACCOUNTING_SLICES`（复数，`crash_judge_dispatch.rs:1925` 打的那一行）不被滤、照进日志。
- `.claude/gate.d/crash-point-replay.sh:1616`：「if line.startswith(prefix):」，`.claude/gate.d/crash-point-replay.sh:1617`：「KEPT_MARKS.search(line, len(prefix))」：只看行首前缀，前缀之后夹着记号的行照进日志，与 318 行那句同义（搜的起点是前缀之后，所以行首 `CRASH_JUDGE_ACCOUNTING_SLICE` 里的 `CRASH_` 不被当记号）。

判定：成立。三处是一回事：登记常量的三个名字与 315 行点名的三个相同；`judge_one_dispatch` 是入口、不在任何不沿它收的清单里，与 317 行相同；滤行的前缀、「夹了别的记号」的保留规则与 314、318 行相同。两点保留：317 行「改这一行的字就换那一组的键」是读码推的（入口单元的字进键），`crash_identity.rs` 里没有钉它的测试（`grep -n ACCOUNTING_SLICE` 在这个文件 0 命中；钉登记那一半的是 `crates/singlefs-verification/src/crash_identity.rs:1548` 的 `an_edit_of_the_listed_gpu_lifecycle_trace_keeps_the_gpu_judge_key`）；315 行说三个单元都「打」，严格说 `gpu_lifecycle_line` 只拼字、写出去的是另两个，不碍结论。什么现象会推翻：把 `judge_one_dispatch` 的打印字改一个字、而 GPU 判器那一组的键不变（则 317 行后半句不成立）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A 一、定义与主 agent 流程 4 份 | 成立 3、不成立 1 | implementation-writer.md 缩域证红命令仍写 `--cwd <副本>`，照写退 2 |
| A 二、钩子 9 份 | 成立 9 | 注释与指针类，全对得上账本；project-code-guard.sh 196-198 行是账本没点名的多做 |
| A 三、门禁与样本 10 份 | 成立 10 | comment-history-lag.tsv 归属栏写反；账本:449 过时；doc-process-records.sh 样本说明有读错风险 |
| A 四、脚本：门禁与工具 8 份 | 成立 8 | hook-swap.py 字符串键一处账本没有专节、调度说清 |
| A 五、实验装置 12 份 | 成立 12 | list_check.py:416 少改一处；replay.sh 三处没有专节、账本:268 过时 |
| A 六、任务服务 4 份 | 成立 4 | 只做 205 节前半；`nocapture` 缺口哪里都没写 |
| A 七、已进 HEAD 5 份 | 成立 5 | task-server/control 正文说「二百二十四节两处」不准，实为账本指针 2 行 |
| B 八句 | 一致 6、打中 2、推不出 1（同一批） | 缩域证红 `--cwd <副本>`；`--verify` 写法在本机回滚（账本:772 已登记） |
| C 漏换 | 路径类 0 处 | 来历出处 46 行不算；坏指针 2 处；样本说明 3 行有读错风险；抽 14 句找到 12、部分 2 |
| D 门禁注释 | 成立 | 登记、入口、滤行代码三处对得上；第二半句无测试钉住 |

打中数（按「不成立」「推不出」「打中」计）：不成立 1（A）；打中 2、推不出 1（B）；C 坏指针 2、读错风险 1 类；D 0。

## 要主 agent 跑的命令

1. `task-server/task submit -- python3 scripts/hosts/lib_repository_copy.py entries .`；在仓根起；看输出里有没有 `REPOSITORY_COPY entry prune-server` 与 `REPOSITORY_COPY entry gpu-verification-server` 两行（有，则 `.claude/agents/tooling-writer.md:31` 说的「在仓副本里改」两个服务的 `src/` 成立；缺其一，该句打中）。我直接起它被权限分类器拒了，没有绕。
2. `task-server/task submit -- python3 scripts/gate/lib_function_units.py closure . scripts/gate/hook-swap.py --explain`；在仓根起；看有没有 `refused` 行、点的是不是 `scripts/gate/hook-swap.py:1421`（有，则 `.claude/agents/tooling-writer.md:35` 说「hook-swap.py 键算不出、登不进开关复跑表」今天仍成立；没有，那句过时）。

两条都不带范围声明（不起验证）；没有别的要跑的。

## 没读到的

- 调度里转述的任务号、退出码与「逐格相同」的数（19717、19775、19727 等）：没有任务号给我、没读 `task-server/task log`，A 的验证栏全按转述。
- 第 1 到 6 问、第 9 问（攻方腿）没做；`research/prompts/ledger-topdown-governance-r1-opus-output.md` 按禁读清单没读。
- A 里混改文件（13 份）只读了表点到的几处，别的会话的改动没判；`scripts/agents/agent-watch.py` 695 行、`.claude/gate.d/code-tooling.sh` 一带大段没读。
- 封存账本里被引节的正文：C 只核了「标题在不在」，没逐节核那一句话是否真出自那一节（2 处坏指针之外）。
- `.claude/kb/` 的条款：这一轮 A、B、C、D 的判据都在账本、调度、代码里，没引 kb 条款，没读 D13 等。
- 工具员报告 `~/.local/share/singlefs/work/ledger-topdown-*/result.md` 只读了 `sweep-apply-tooling` 与 `task-server-debts` 的相关小节，别的（如 `swap-followups`、`whole-script-swap`）只依赖调度转述。

## 没做什么

- 不判别的腿的格（正文第 1 到 6、9 问）；不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 草稿目录 `~/.local/share/singlefs/work/governance-r1-forward` 里只有我的只读核对输出与两个只读小脚本（`check_sections.py`、`check_current.py`），没有仓副本、编译目录；`du -sh` 量得 148K。

交回闸 `python3 scripts/agents/cite-check.py` 末行：✓ 核了 58 处引文、74 处裸行号，对上 58 处，没判 74 处（其中裸行号没有引文可比 74 处；逐处列在上面）
