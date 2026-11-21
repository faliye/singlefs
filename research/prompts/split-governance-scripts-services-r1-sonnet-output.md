# 拆分换上的脚本与服务治理文件轮第一轮：正推腿报告（第 5 到 7 问）

日期：2026-11-20。腿：`three-way-forward`（Sonnet）。草稿目录：`/home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-sonnet/`。
方案按 `crates/` 今天的实现来谈：这一轮第 5 到 7 问的被判文件里没有 `crates/` 下的代码，与 `crates/` 的交点不在这三问的射程里；这三问读的是 `scripts/gate`、`scripts/agents`、`task-server/` 与它们的调用方。

## 读到的范围（只有这一条腿读到的写明）

- 第 5、6 问：`scripts/gate/hook_swap_selftest.py`、`scripts/gate/hook_swap_swap.py`、`scripts/gate/hook-swap.py`（头注）、`scripts/agents/agent_watch_alerts.py`、`scripts/agents/agent-watch.py`（头注与入口）、`.claude/hooks/ask-user-placement-guard.sh`、`.claude/hooks/session-number-stop-check.sh`，另扫 `.claude/hooks/` 下全部 `.sh`、`.py` 里记检出的起头。
- 第 7 问：`task-server/` 下三个入口与 25 份拆出的模块、`task-server/task`、`task-server/control`、`task-server/tests/test_cleanup_protocol.py`，`.claude/hooks/` 里按路径载入服务库的三处，`scripts/` 下提到 task-server 三个文件名的脚本。只有这条腿读到；`crates/`、`prune-server/`、`gpu-verification-server/` 这一腿没读。
- 拆前原文：`task-server` 三份的 HEAD 版早于拆分前就已落后（HEAD 的 `task_server.py` 10117 行，拆分合入前的工作区版 10728 行），所以第 7 问的「拆前」取拆分合入前的备份 `~/.local/share/singlefs/work/crates-split-task-server/backup-before-apply/`（三份 sha256 与调度记录写的基线一致，见下表），已拷进草稿目录 `pre/`。hook-swap 与看门狗那两件的拆前版取 `~/.local/share/singlefs/work/crates-split-gate-scripts/originals/scripts/gate/hook-swap.py`。
- 没起任何项目代码：第 7 问的名字比对用 ast 读语法树（不 import 项目代码，项目代码闸也不让我 import 服务库）；要真 import 的动态核对写进「要主 agent 跑的命令」。

## 复跑（只读、不起项目代码）

静态比对三条，都是直接跑的 ast 脚本（不碰服务）：

```
cd /home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-sonnet
python3 -I compare.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-sonnet/pre
python3 -I external.py /home/fy5090/code/singlefs
python3 -I server_alias.py /home/fy5090/code/singlefs
```

文件 sha256（量的）：

| 文件 | sha256 |
|---|---|
| names.py | 74707b06585fab2fe60f2b4634fbfb1b88c5a99f701f7dba008c1a62a7ac8918 |
| compare.py | d9aa745bd385aaa8211a8b6bbb47d35aec7d26efeb0f0070ca8e2a850e867555 |
| external.py | 2e6d8cf93f42c724a7276932aa584aa1cb100077a02708582865fd60b737ffc2 |
| server_alias.py | 0c8ee9372ec4f8ccb6438b01391bec1cefce7bbee09d740be832bf5978e95013 |
| globals_rebound.py | cc5432c95f558fd22cba4fa3832d8905ef63fb289ed034d5c30236f9af7effe4 |
| rebinds.py | 93c8e02d0a414f0f232fd376d95d1522fd16ab00764cc748bb4505b004733b15 |
| hook_names.py | 7667333370ce4f1965df54b3c9110135ea482068bec2645d0f75fce2da68a96d |
| dynamic_names.py（只编过，没跑） | 4ea3110dc07b4155416fb7bf6f7127450d2ffda79d16eecd4d7c1df4a551dbce |
| pre/task_server_lib.py | 454df3cd689e371cf0c4171c6b4d84936c5493529d3b92d95bf49aefdf99ac90 |
| pre/task_server.py | 7d0f7454735df270c0daf2630e6dd210ff2180d16a63da3cb17e3cba3ddceabc |
| pre/task_server_selftest.py | ed68a213e886082c94905545280ecc5f16261cc760dba844b36f1d59283874c8 |

没有写模型，没有 `rerun.sh`；探针只是读语法树，没起进程，内存与时间可以忽略（每条几秒、几十 MB）。草稿目录 2.9M，里面没有仓副本、编译目录。

## 第 5 问：两件修补做的是不是「实现今天的样子」那一条说的事

### 5a hook-swap：自证换替身的模块——成立

「实现今天的样子」那一条（转述）：自证把替身写在 `globals()["scratch_lock_content"]`，拆后被测的 `claim_record_removal` 按 `hook_swap_swap` 的全局名读，改成换在 `hook_swap_swap.scratch_lock_content` 上，两格另核 `.taking` 机器一段确是替身写的。

逐条核今天的代码：

1. 被测代码按 `hook_swap_swap` 的全局名读（成立）：
   - `scripts/gate/hook_swap_swap.py:664`「def claim_record_removal(taking, mode, heartbeat_seconds=SCRATCH_LOCK_HEARTBEAT_SECONDS):」
   - `scripts/gate/hook_swap_swap.py:673`「content = scratch_lock_content()」（这一行读的是 `hook_swap_swap` 模块自己的全局名；函数本身在 `scripts/gate/hook_swap_swap.py:474`「def scratch_lock_content(directory=None):」）
   - 同一模块里还有两处读同一个全局名：`scripts/gate/hook_swap_swap.py:721`「content = scratch_lock_content()」（`reap_dead_file`）、`scripts/gate/hook_swap_swap.py:875`「held = ScratchLock(path, scratch_lock_content(), mode)」（`hold_scratch`）。
2. 修补前的写法确实换在别的模块上：拆前原文（工作根下 `crates-split-gate-scripts/originals/scripts/gate/hook-swap.py`，仓外，引文核对脚本找不到，转述）第 5226 行写 `globals()["scratch_lock_content"] = content_elsewhere_for_remover`，同一份文件里 `claim_record_removal` 在第 2938 行、读 `scratch_lock_content()` 在第 2947 行，同一个模块，`globals()` 就是被测读的那个；拆后自证在 `hook_swap_selftest` 模块、被测在 `hook_swap_swap` 模块，`globals()` 换在前者，后者照读真函数。这一条与「实现今天的样子」写的因果一致（拆前同模块、拆后跨模块）。
3. 今天的代码换在 `hook_swap_swap` 上（成立）：
   - `scripts/gate/hook_swap_selftest.py:16`「import hook_swap_swap」
   - `scripts/gate/hook_swap_selftest.py:1492`「hook_swap_swap.scratch_lock_content = replacement」（在 `use_scratch_lock_content` 里，`:1491`）
   - 四处调用：`scripts/gate/hook_swap_selftest.py:1535`「use_scratch_lock_content(content_elsewhere_for_remover)」与 `:1562`「use_scratch_lock_content(real_scratch_lock_content)」，`:1597`、`:1615` 同样一进一出；还原放在 `finally` 里。
   - 本文件里已不剩 `globals()["scratch_lock_content"]`（`grep -nE 'globals\(\)\["scratch' scripts/gate/hook_swap_selftest.py` 零命中，已跑）。
4. 「两格另核 `.taking` 机器一段确是替身写的」成立：`scripts/gate/hook_swap_selftest.py:1494`「def taking_machine(path):」读 `.taking` 的第 `SCRATCH_LOCK_COLUMNS.index("机器")` 段；`scripts/gate/hook_swap_selftest.py:1543`「elif taking_machine(beat_taking) != elsewhere_machine:」与 `scripts/gate/hook_swap_selftest.py:1604`「elif taking_machine(over_taking) != elsewhere_machine:」各在一格里；`scripts/gate/hook_swap_selftest.py:1487`「elsewhere_machine = "f" * 16」；机器是第一段：`scripts/gate/hook_swap_swap.py:431`「SCRATCH_LOCK_COLUMNS = ("机器", "开机", "进程号", "进程启动刻", "开始秒数", "随机串", "主机名")」。
5. 别处有没有同一类的换错（只核这一个模块，不替攻方腿报）：`grep -nE '^\s+(os|time|hook_swap_\w+|threading|shutil|subprocess|socket|signal)\.\w+ = ' scripts/gate/hook_swap_selftest.py` 只命中 `os.link`（`:1323`）、`os.unlink`（`:1596`）与新写的这一处；`hook_swap_swap.py` 里用的是 `os.link(`、`os.unlink(` 的模块属性写法（`scripts/gate/hook_swap_swap.py:641`「os.unlink(path)」），换在 `os` 模块上被测照读，不是换错。

不能由读代码坐实的两句（「副本里拆前绿、拆后恰好红那一格、修后绿，七条开关红格与文件头点名逐条相同」「换上后主工作区自证「查了 119 种」通过」）：待主 agent 跑（命令见「要主 agent 跑的命令」第 1、2 条）；没跑之前当「推的」。

什么现象会推翻 5a：主工作区跑 `hook-swap.py --selftest` 红在这两格以外的格，或者把 `use_scratch_lock_content` 的函数体改回 `globals()["scratch_lock_content"] = replacement` 之后自证仍绿（说明新加的两处核对没把换错的替身抓出来）。

### 5b 看门狗：检出前缀表补登——成立

「实现今天的样子」那一条（转述）：`UNJUDGED_DETECTION_PREFIXES` 补登 `ask-user-placement-guard.sh`、`session-number-stop-check.sh` 两份新钩子的起头，自证加一格拆分族的该红输入。

1. 补登的两个起头与钩子里的字面逐字相同：
   - `scripts/agents/agent_watch_alerts.py:81`「"放置类弹窗闸没判、照常放行", "会话编号收工检查没判、照常放行",」
   - `.claude/hooks/ask-user-placement-guard.sh:86`「UNJUDGED_PREFIX = "放置类弹窗闸没判、照常放行"」
   - `.claude/hooks/session-number-stop-check.sh:113`「UNJUDGED_PREFIX = "会话编号收工检查没判、照常放行"」
2. 钩子真拿这个字面起头写检出（所以表里登的起头会被真检出碰到）：`.claude/hooks/ask-user-placement-guard.sh:423`「_, said = lib_work_root.record_unjudged_pass(HOOK_NAME, hook_input, f"AskUserQuestion：{str(header)[:300]}", [f"{UNJUDGED_PREFIX}：{problem}"])」；`.claude/hooks/session-number-stop-check.sh:566`「_, said = lib_work_root.record_unjudged_pass(HOOK_NAME, hook_input, "收工回复的会话编号检查", [f"{UNJUDGED_PREFIX}：{problem}"])」。
3. 表确实管着「只记不叫醒」：`scripts/agents/agent_watch_alerts.py:84`「INFO_DETECTION_PREFIXES = ("写被拒", "项目代码闸没判全") + (() if BROKEN_DETECTION == "unjudgedwakes" else UNJUDGED_DETECTION_PREFIXES)」把它并进 `INFO_DETECTION_PREFIXES`，`scripts/agents/agent_watch_alerts.py:630`「if findings and all(finding.startswith(INFO_DETECTION_PREFIXES) for finding in findings) and BROKEN_DETECTION != "infowakes":」按它判。
4. 全表现在把真钩子里抽得到的起头都盖住（自己再用 grep 独立列了一遍，没经被测的抽法）：`grep -nE '^[A-Z][A-Z_]*PREFIX\s*=\s*"[^"]+"' .claude/hooks/*.sh .claude/hooks/*.py` 里带「没判、没判全、判不了、照常放行」字样的 10 行（放置类弹窗闸、定时闸、时刻写法闸、弹窗闸、会话编号收工检查、完工清理闸两条、快照清单读不了、派发闸、交回正文闸）都在 `UNJUDGED_DETECTION_PREFIXES` 里，`findings` 行里的 f"续做闸没判全：…"、f"循环提醒的停止钩子判不了、照常放行：…"、f"提交后的索引检查判不了、照常放行：…"、f"写闸没判全：…"、f"范围声明闸判不了：…"、f"项目代码闸没判全：…"，与 `bash-command-detector.sh`、`project-code-guard.sh` 的 `%s 没判成` 也在。
5. 「抽法本来就扫拆分族」成立：抽法在 `scripts/agents/agent_watch_alerts.py:1045`「for name in sorted(os.listdir(hooks_directory)):」与 `scripts/agents/agent_watch_alerts.py:1046`「if not name.endswith((".sh", ".py")):」按目录读全部 `.sh`、`.py`，不按入口名；拆前 HEAD 的 `scripts/agents/agent-watch.py` 同样是这两行（`git show HEAD:scripts/agents/agent-watch.py` 的 2856、2857 行），抽法没改。所以补登是「新钩子带进来的两个起头」，不是「抽法漏读部分文件」；这与调度记录「丙交回」写的一致（`records/2026-11-17-subagent账本清欠调度记录.md:508`「- 丙交回：两个漏登起头是新钩子带进来的」）。
6. 「换上后自证绿；开关复跑红在门禁那一格…没跑成」：自证绿待主 agent 跑（命令第 3 条）；开关复跑那一格归 mainagent(B)，这一腿不判。

什么现象会推翻 5b：`agent-watch.py --selftest` 的 `selftest_unjudged_prefixes` 第一格（抽到的起头对表）红在新起头之外的起头；或把这两个起头从表里删掉后自证仍绿。

## 第 6 问：两件修补新加的自证格，红的地方是不是它名字说的那件事

### 6a hook-swap 两格——成立（读代码；真红待主 agent 跑）

新加的是两处核对，各嵌在一格已有的格里（格名没改，头注 `scripts/gate/hook-swap.py:329`「草稿目录的占用：摘的那一趟放着 .taking 期间刷新它的修改时间，另一台上还活着的摘的那一趟放了超过过期秒数也照挡后来的一趟」与 `:330` 点名的那两句都还在）：

- 「摘的那一趟放着 .taking 期间刷新它的修改时间」那一格：`scripts/gate/hook_swap_selftest.py:1543`「elif taking_machine(beat_taking) != elsewhere_machine:」，`:1544`「problems.append(f"摘的那一趟放的 {REMOVAL_SUFFIX} 身份不是另一台起的（样本没造成：替身没换到占用记录读的那个模块上）")」。
- 「摘的那一趟停住、.taking 被当成已死接手之后」那一格：`scripts/gate/hook_swap_selftest.py:1604`「elif taking_machine(over_taking) != elsewhere_machine:」，`:1605` 同一句。

红的地方：
1. 它只在「`.taking` 放成了、但第一段不是 `f`×16」时红。这正是替身没换到 `hook_swap_swap` 上的样子：`claim_record_removal` 读的 `scratch_lock_content`（`scripts/gate/hook_swap_swap.py:673`「content = scratch_lock_content()」）是真函数，写出来的机器一段是这一台自己的（`scripts/gate/hook_swap_swap.py:477`「fields = (this_machine(), identity_file_value(BOOT_ID_PATH),」，第一个字段是 `this_machine()`）。所以红的说法（「样本没造成：替身没换到占用记录读的那个模块上」）和红的条件一致，不是别的原因红了借这句话说。
2. 先有 `elif`、后有 `else`：红了之后不再往下跑那一格真正要判的东西（过期、刷新、被接手），避免把「样本没造成」报成「来的一趟没被挡住」之类别的症状。头一格 `taking_text is None`（没走到放着 `.taking` 的当中）仍走原来的分支（`scripts/gate/hook_swap_selftest.py:1542`「problems.append(f"摘的那一趟没走到放着 {REMOVAL_SUFFIX} 的当中（样本没造成）")」），两种「样本没造成」分得开。
3. 这一格要判的本职红（弄坏开关 `taking-no-heartbeat`、`taken-over-unchecked` 下红）没动：头注 `scripts/gate/hook-swap.py:388`「HOOK_SWAP_BREAK=taking-no-heartbeat  摘的那一趟放着 .taking 的期间不刷新它的修改时间」到 `:391` 写的点名格没变，新加的 `elif` 都夹在它们之前，只在样本没造成时短路，开关下替身换对了，机器一段是 `f`×16，不进新分支，原来的红照出。
4. 一处小的不对称（不算打中）：`taking_machine` 读不到文件或段数不够时交 `None`（`scripts/gate/hook_swap_selftest.py:1498`「except (OSError, IndexError):」），`None != elsewhere_machine` 也走红；说法是「身份不是另一台起的」，读不到时说的略偏，但红仍红在同一件事（占用记录里的身份没按替身写）上。

什么现象会推翻 6a：把 `use_scratch_lock_content` 的函数体改回只换在 `hook_swap_selftest` 自己的全局名上，自证在这两格上不红在「样本没造成：替身没换到…」那一句；或开关 `taking-no-heartbeat`（`task-server/task submit -- env HOOK_SWAP_BREAK=taking-no-heartbeat python3 scripts/gate/hook-swap.py --selftest`）下红的格不止头注点名的那一格。

### 6b 看门狗一格——成立，但读代码看有一处只管得住一半

新加的一格：`scripts/agents/agent_watch_alerts.py:1092`「with open(os.path.join(fake_hooks, "fake-split-guard.sh"), "w", encoding="utf-8") as handle:」写一个只有注释的入口，`:1094`「with open(os.path.join(fake_hooks, "fake-split-guard-decisions.py"), "w", encoding="utf-8") as handle:」写部分文件，里面 `scripts/agents/agent_watch_alerts.py:1095`「handle.write('SPLIT_UNJUDGED_PREFIX = "拆分闸没判、照常放行"\n')」；判在 `scripts/agents/agent_watch_alerts.py:1103`「if fake.get("拆分闸没判、照常放行") != ["fake-split-guard-decisions.py"] or "拆分闸没判、照常放行".startswith(INFO_DETECTION_PREFIXES):」。

红的地方：
1. 红在「抽法对只有部分文件里才有起头常量的钩子抽不出」：入口只有注释、常量只在 `-decisions.py` 里，抽法若只读 `.sh` 或只读与入口同名主干的文件就抽不到，`fake.get(...)` 不是 `["fake-split-guard-decisions.py"]`，红。红的说法（「拆分族的部分文件读不到，漏登就不会红」）和红的条件一致。
2. 一半管得住：今天抽法本来就按目录读全部 `.sh`、`.py`（5b 第 5 点），这一格是给抽法写了一道回归护栏，不是补登的那两个起头的直接护栏。补登的那两个起头由头一格（`scripts/agents/agent_watch_alerts.py:1083`「if not extracted or uncovered:」，真钩子目录抽出的起头都要在表里）盖着：把任一个从表里删掉，头一格红，不靠这一新格。
3. `or` 后面的那一半（`"拆分闸没判、照常放行".startswith(INFO_DETECTION_PREFIXES)`）对假起头永远为假（表里没有这一句），在弄坏开关 `unjudgedwakes`（`scripts/agents/agent-watch.py:554`「AGENT_WATCH_BREAK=unjudgedwakes  INFO_DETECTION_PREFIXES 不含 UNJUDGED_DETECTION_PREFIXES：钩子判不了而照常放行记下的检出真碰到都叫醒」）下也不会变真，所以这一格不挂这个开关；头注里 `scripts/agents/agent-watch.py:555`「selftest_unjudged_prefixes」点名的两格（「钩子源码里记检出用的」「UNJUDGED_DETECTION_PREFIXES 里」）也不含这一格。这一格没有挂任何弄坏开关，证红只能靠改抽法（例：抽法里只认 `.sh`），没有现成的开关。
4. 对表里每个起头造检出记录的那一格（`scripts/agents/agent_watch_alerts.py:1117`「if len(alerts) != 1 or len(quiet_notes) != len(UNJUDGED_DETECTION_PREFIXES):」）现在多算了两个起头，它是按表长算的，不写死数，所以补登之后它仍是对的。

什么现象会推翻 6b：把抽法改成只读 `.sh`（草稿副本里）后这一新格仍绿；或把两个新起头之一从 `UNJUDGED_DETECTION_PREFIXES` 删掉后头一格仍绿。待主 agent 跑（命令第 3 条是整份自证绿，证红要在副本里改抽法，写在下面「要主 agent 跑的命令」之后的备注里）。

## 第 7 问：task-server 三个入口再导出的名字，覆盖不覆盖拆前被别的文件引到的每一个名字——成立（名字层面零缺；绑定是不是同一个对象这一腿没真 import，待主 agent 跑）

做法：把拆前三份（拆分合入前的备份，不是 HEAD）的顶层名字（`def`、`class`、赋值、`import`、`from … import` 引入的名字，含 `if`、`try`、`with` 里的）用 ast 列出，与拆后入口自己定义的加显式 `from … import (…)` 带进来的名字比；再取各引用方引到的名字，核它们都在入口里。入口再导出全是显式名单，没有星号导入（`grep -nE '^from .* import \*' task-server/task_server*.py` 零命中，已跑），所以不存在「星号导入漏带下划线名字与 `__all__` 之外的名字」这一类漏网。

### 7.1 拆前顶层名字全覆盖（量的，`compare.py`，源文件语法树）

```
== task_server_lib: 拆前顶层名字 554 个；拆后入口给得出 554 个；拆前有、拆后入口没有 0 个
== task_server: 拆前顶层名字 331 个；拆后入口给得出 336 个；拆前有、拆后入口没有 0 个
== task_server_selftest: 拆前顶层名字 1020 个；拆后入口给得出 1022 个；拆前有、拆后入口没有 0 个
```

- 比对脚本自己会红的证明：在拆前拷贝末尾加两个不存在的名字（`ZZ_ABSENT_NAME`、`zz_absent_function`）再跑，它点名这两个「拆前有、拆后入口没有 2 个」（已跑，拷贝用完已删）。
- 入口再导出的写法：`task-server/task_server_lib.py:1801`「from task_server_lib_constants import (」起，后面还有 `configuration`、`commands`、`readings`、`scopes` 四份，各一条显式名单；`task-server/task_server.py:855`「from task_server_scheduler_starting import SchedulerStarting」前面有 `task_server_constants`、`task_server_store`、`task_server_machines` 与另四份混入类；`task-server/task_server_selftest.py:255`「from task_server_selftest_lifecycle import (」前面有另十二份。
- 函数里用 `global` 重新绑定的模块名字：拆前三份各 0 个（`globals_rebound.py`，量的）。所以不会有「某个名字在子模块里被函数改绑、入口那一份还是旧值」这一类（这一类是攻方腿第 2 问的，这里只报零命中）。

### 7.2 各引用方引到的名字（量的，`external.py`、`server_alias.py`、`hook_names.py`）

| 引用方 | 经什么引 | 引到几个名字 | 入口里没有的 |
|---|---|---|---|
| `task-server/task` | `task-server/task:86`「task_server_lib as lib  # noqa: E402」，`lib.X` | 35 | 0 |
| `task-server/control` | `task-server/control:95`「task_server_lib as lib  # noqa: E402」，`lib.X` | 48 | 0 |
| `task-server/tests/test_cleanup_protocol.py` | `task-server/tests/test_cleanup_protocol.py:36`「task_server as server  # noqa: E402」，`server.X`（`Scheduler`、`Store`、`cgroup_processes`、`lib`、`subprocess`），另有 `patch_server("cgroup_processes"/"LEFTOVER_STOP_SECONDS"/"say"/"unit_processes")` 四个字符串名 | 5（加 4 个字符串名） | 0 |
| 拆出的 13 份自证模块 | `task-server/task_server_selftest_fakes.py:500`「SERVER = lib.load_module("task_server_constants_for_selftest", lib.SERVER_SCRIPT)」，`SERVER.X` | 79 | 0 |
| 拆出的各份模块经 `lib.X` 引库入口 | 各份自己 `task_server_lib as lib` | 全部 | 0（`external.py` 逐份核，输出「拆出各份模块经 lib. 引、库入口里没有的名字： 无」） |
| `.claude/hooks/lib_project_code_runs.py`、`lib_run_scope.py`、`completion-cleanup-guard-decisions.py` | 按文件路径载入库后 `library.X`（`.claude/hooks/lib_project_code_runs.py:1355`「"task_server_lib_for_hooks", TASK_SERVER_LIBRARY_PATH)」） | `read_configuration`、`default_values`、`DATABASE_NAME`、`SOCKET_NAME`、`UNIT_NAME`、`SERVER_SCRIPT`、`RUNNER_SCRIPT`、`ADMITTED_VARIABLE`、`unnamed_run_problem`、`sharded_entry`（另 `COMMIT_GATE_ENTRIES`、`TEST_STARTING_GATE_CELLS`、`SHARDED_ENTRIES`、`DECLARED_HEADER_LINES`、`load_module`） | 0（`hook_names.py`）；`completion-cleanup-guard-decisions.py` 里 `home_directory`、`shared_data_path`、`DATA_ROOT_HOWTO` 这几个不在服务库里，因为它们读的是 `lib_work_root` 模块（`.claude/hooks/completion-cleanup-guard-decisions.py:890`「library = module("lib_work_root")」） |
| `scripts/` 下提到 task-server 三个文件名的脚本 | 读到的都是命令行起它们（`scripts/hosts/memory_peaks_lib.py:1352`「lone_server = ["python3", "task-server/task_server_lib」）或注释，没有把三个入口当模块 import 取名字的；唯一读入口源码文本的见 7.4 | — | — |

`task_server_selftest` 入口被别处引到的只有 `run_all`、`run_cell`（`task-server/task_server_lib.py:1902`「return task_server_selftest.run_all(arguments)」），两个都在入口里定义（`task-server/task_server_selftest.py:643`「def run_all(arguments):」、`task-server/task_server_selftest.py:589`「def run_cell(name, directory):」）。

结论：没有缺的名字，给不出「缺的那一个」。

### 7.3 按路径载入库的调用方找得到拆出的模块

库入口自己把同目录放进 `sys.path`：`task-server/task_server_lib.py:1799`「if os.path.dirname(os.path.realpath(__file__)) not in sys.path:」与 `task-server/task_server_lib.py:1800`「sys.path.append(os.path.dirname(os.path.realpath(__file__)))」，在第一条 `task-server/task_server_lib.py:1801`「from task_server_lib_constants import (」之前。所以钩子里 `spec_from_file_location` 按路径载入库（上表）不靠调用方设 `sys.path`；拆前是单文件不需要这一步，拆后补了。

### 7.4 同一类的一条线索（不是名字缺）：`scripts/hosts/queue-health.py` 的一格读 `task-server/task_server.py` 原文

- `scripts/hosts/queue-health.py:1140`「forms_read[form] = looked_for in handle.read()」所在的循环（`scripts/hosts/queue-health.py:1137`「for relative, form in (("task-server/task_server.py", ATTEMPT_UNIT_SOURCE_FORM)」起）只读入口 `task_server.py` 的文本，找 `scripts/hosts/queue-health.py:142`「ATTEMPT_UNIT_SOURCE_FORM = "singlefs-task-{self.store.tag}-{task['id']}-{number}.service"」；格在 `scripts/hosts/queue-health.py:1141`「verify("attempt-forms-match-sources", all(forms_read.values())」。
- 拆前这两种写法在 `task_server.py` 里（拆前备份第 9345、9272 行）；拆后在 `task-server/task_server_scheduler_starting.py:119`「unit = f"singlefs-task-{self.store.tag}-{task['id']}-{number}.service" if detached else ""」与 `task-server/task_server_scheduler_starting.py:46`「label = f"task-{self.store.tag}-{task['id']}-{number}"」，入口 `task-server/task_server.py` 里 `grep -nF` 这两句零命中（已跑）。
- 推：`queue-health.py --selftest` 的 `attempt-forms-match-sources` 这一格拆后红（读入口原文、找不到）。是判法只读入口原文那一类（攻方腿第 1、2 问的射程），这一腿只报线索；没起自证，待主 agent 跑（命令第 4 条）。它是否在被判清单里这一腿没核。

什么现象会推翻第 7 问：`dynamic_names.py`（命令第 5 条）真 import 三个入口之后，拆前顶层名字有一个不在入口的 `vars` 里；或 `task-server/task_server_lib.py --selftest` 整份红在 `AttributeError: module 'task_server_lib' has no attribute …`。

## 附带（不属于第 5 到 7 问，读到顺手记）

- 调度记录 写 task-server 三份拆完「最长 1718 行」（`records/2026-11-19-crates代码拆分.md:293`「最长 1718 行」），今天 `wc -l task-server/task_server_selftest_notifying.py` 是 1932 行（最长的一份），且 25 份拆出的模块与拆分合入时的补丁来源逐份 `cmp` 相同（工作根 `crates-split-task-server/new/task-server/` 对比主工作区，已跑，28 份「same」），不是合入后又改长的。2000 行线内，只是记录里的数不对。
- `.claude/hooks/ask-user-placement-guard.sh` 在我读的过程中被别的会话改过（`UNJUDGED_PREFIX` 常量从第 68 行挪到第 86 行，`git status` 是 `AM`）。报告里的行号取改后的，常量的字面没变。

## 要主 agent 跑的命令

每一条一行、一个范围；`--selftest` 不在「起验证的命令」那几种里，不带范围声明。命令在仓根起，除非写了 `--cwd`。

1. hook-swap 自证（5a、6a：新格是否绿、「查了 119 种」）：
   `task-server/task submit -- python3 scripts/gate/hook-swap.py --selftest`
   在哪起：仓根。算过：`TASK_SERVER_RESULT` 行 `type=succeeded exit=0`，`task-server/task log <号>` 里有「✓ hook-swap 自检通过（查了 119 种）」。种数不是 119 时按判决逐条对（推的：新加的两处核对没加「种」，应不变）。
2. hook-swap 两条开关（6a：红红在点名的那一格、没红在新加的核对上）：
   `task-server/task submit -- env HOOK_SWAP_BREAK=taking-no-heartbeat python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=taken-over-unchecked python3 scripts/gate/hook-swap.py --selftest`
   （两条各自一行交）。在哪起：仓根。算过：每条 `exit=1`，`✗` 行里只有头注点名的那一格（头注 `scripts/gate/hook-swap.py:388`「HOOK_SWAP_BREAK=taking-no-heartbeat」那一条与其后 `taken-over-unchecked` 那一条各点名的一格），`✗` 行里没有「样本没造成：替身没换到占用记录读的那个模块上」。
3. 看门狗自证（5b、6b）：
   `task-server/task submit -- python3 scripts/agents/agent-watch.py --selftest`
   在哪起：仓根。算过：`type=succeeded exit=0`，日志里 `✓ agent-watch 自检通过`，其中「钩子判不了而照常放行记下的检出只记不叫醒（N 格）」的 N 比补登之前多一格（推的）。
4. queue-health 自证（7.4：拆后读入口原文的那一格）：
   `task-server/task submit -- python3 scripts/hosts/queue-health.py --selftest`
   在哪起：仓根。预期（推的）：红，`✗ 自证 attempt-forms-match-sources` 一行；绿了就推翻 7.4。
5. 三个入口真 import 之后的名字与绑定动态核对（第 7 问的动态半边；脚本里模块名由几段拼出，不触发项目代码闸的字面判法；只读，不起服务）：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-sonnet -- python3 -I dynamic_names.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-sonnet/pre`
   在哪起：草稿目录（上面 `--cwd`）。算过：`exit=0`，末行「缺或分叉合计 0」；非 0 时先看是「入口 vars 里缺」（推翻第 7 问）还是「同名可变对象是另一份」（线索，它按类型判，可能有假阳性，要对着名字看）。脚本只编过（`python3 -m py_compile`），没跑过，第一次跑出的任何红先查脚本。
6. 6b 证红（可选；5b 的「该红输入」是否抓得到只读不到的部分文件）：在仓副本里把 `scripts/agents/agent_watch_alerts.py:1046`「if not name.endswith((".sh", ".py")):」改成只认 `.sh`，副本里交第 3 条那一条，预期 `✗ 自检：` 行里出现「造一个拆成入口加同层部分文件的假钩子」；副本用 `python3 scripts/hosts/minimal-copy.py` 建，我没建。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 5a hook-swap 换替身的模块 | 成立 | 被测在 `scripts/gate/hook_swap_swap.py:673`「content = scratch_lock_content()」读自己模块的全局名，今天的替身换在 `hook_swap_swap.scratch_lock_content` 上（`scripts/gate/hook_swap_selftest.py:1492`「hook_swap_swap.scratch_lock_content = replacement」），两格另核 `.taking` 机器一段（`:1543`、`:1604`）；自证真绿、「119 种」待主 agent 跑 |
| 5b 看门狗前缀表补登 | 成立 | 两个起头与两份钩子的 `UNJUDGED_PREFIX` 字面逐字相同，钩子真用它写检出；抽法按目录读全部 `.sh`、`.py`，拆前就是这样，补登的是新钩子带进来的起头；自证绿待主 agent 跑，开关复跑那格归 mainagent(B) |
| 6a 两格的红 | 成立（读代码） | 只在 `.taking` 机器一段不是替身写的才红，红的说法与条件一致，弄坏开关下原来的红不变；真红待主 agent 跑 |
| 6b 看门狗新格的红 | 成立，管一半 | 红在抽法读不到只有部分文件里才有的起头常量；抽法今天本就读全，所以它是回归护栏；补登的两个起头由头一格盖着；这一格没挂任何弄坏开关 |
| 7 三个入口再导出的名字 | 成立（名字层面） | 拆前 554、331、1020 个顶层名字在入口里 0 缺；`task`、`control`、测试、钩子三处按路径载入、`SERVER.X` 79 个名字都在；给不出缺的那一个；同一对象与否待主 agent 跑 |
| 7.4 `queue-health.py` 读入口原文 | 线索（推的） | 两种尝试单元写法拆后在 `task_server_scheduler_starting.py`，入口里零命中，`attempt-forms-match-sources` 一格应红，待主 agent 跑 |

## 没做什么

- 没替主 agent 采纳或出判决；没往统一任务服务交任务（命令在上面一节）；判决要引的数（自证绿、「查了 119 种」、第 7 问的动态对象一致）都没复跑，当「推的」。
- 没起任何项目代码：没 import 服务库、没起自证、没跑弄坏开关；第 7 问只比了名字，没有核每个名字绑定的是不是同一个对象（`dynamic_names.py` 编过、没跑）。
- 没读 `crates/`、`prune-server/`、`gpu-verification-server/` 与两个 Rust 服务；没判第 1 到 4 问（攻方腿的）；没判「开关复跑红在门禁那一格」（归 mainagent(B)）。
- 第 5 问拆前对照：hook-swap 的拆前版取工作根下的备份（仓外，引文核对找不到，写成转述）。
- 草稿目录里留了脚本与拆前备份（共 2.9M，没有仓副本、编译目录）；脚本是上面命令要用的，由主 agent 用完删（`du -sh` 已记：2.9M）。
