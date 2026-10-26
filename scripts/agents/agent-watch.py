#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""子 agent 与它们起的进程的定时监控：卡死、空转、永久等待当场报出来。

为什么：2026-09-17 一个实验执行员把 `echo "exit=$?"` 打到标准输出、却在日志里 `until grep -q "^exit="`，
等一行永远不会出现的字，主 agent 查进度才发现；同一天另一个会话的层 0 全量测试跑了三个多小时没有任何输出，
没人分得清是慢还是卡。经过：records/2026-09-17-已分配口径三方与两个实验.md 第六节。

用法：
    agent-watch.py report --agents ID[,ID…]          # 看一眼：每个子 agent 的状态、最后一条命令、告警
    agent-watch.py report --session-dir DIR           # 看这个会话里最近活动过的全部子 agent
    agent-watch.py watch  --agents ID[,ID…]          # 看门狗：每隔一段时间查一次，有告警或全部结束就退出
    agent-watch.py watch  --processes-only            # 不看子 agent，只盯这个 Claude 实例底下的进程：主 agent 自己放后台的长命令用它盯
    agent-watch.py cost   --agents ID[,ID…]          # 用量：调用次数、起步与平均上下文、新输入、读缓存、输出、整份重写、工具结果被什么撑大
    agent-watch.py --selftest                         # 造假的会话记录与进程走一遍各种告警；AGENT_WATCH_BREAK=<项> 时必须判红

主 agent 派发之后用 Bash 的 run_in_background 起 watch：它一退出，harness 就会叫醒主 agent。
它看、它报，不停子 agent；终止进程只有一种情形：「进程卡住」类告警同一件事连报到 --terminate-after-unacknowledged-reports 次、主 agent 既没处置也没 --ack，
就经 proc.py stop 终止那一个进程并在报告里写明（下面「调度方不响应就终止」一节）。别的告警只报，结束不结束由主 agent 看了报告定（用户 2026-09-17）；
调度方不响应才终止是用户 2026-10-13 定的（「如果调度方没有响应，那么可以终止」，.claude/main-agent.md「派出去之后」第一段）。
退出码：0 被看的子 agent 全部交回或被停、没有告警；3 有告警；4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）；2 用法错。
带 --processes-only 时 0 是看门狗起的时候就在跑的那些进程都退出了（起的时候一个都没有也退 0，报告里写明是哪一种）。
退出时出路句印的重起命令是 watch.sh 认得的写法（主 agent 经它起看门狗，它不认这里的 --agents / --processes-only）：盯点名的子 agent 印
`bash scripts/agents/watch.sh <agent 号>,…`，--discover 印不带 agent 号的 `bash scripts/agents/watch.sh`，--processes-only 印 `bash scripts/agents/watch.sh --processes`。
AGENT_WATCH_BREAK=restarthint 印回这里自己的选项，自检里出路句那两格必须判错。
「交回」按交回工具（SubagentHandback）送达判：调用之后有它对应、不是错误的 tool_result（被交回闸拒的、{"success": false} 的不算），
认法只在同目录 subagent_transcript.py（续做闸与 agent-cost.py 取的是同一份）；只结束本轮、没交回的子 agent 还在等后台任务，不算结束。
交回之后只有新指令撤销它（续做消息，认法见 is_new_instruction）；交回之后被自己后台任务的完成通知叫醒、又调了工具再结束本轮的，仍算交回。
「被停」认两处：子 agent 会话记录里的 `[Request interrupted`（停在工具调用中途）；主会话记录（<会话 id>.jsonl）里 TaskStop 对它的成功结果，
时刻不早于它自己最后一条记录减 30 秒（停在结束本轮、等后台任务的时候，子 agent 的会话记录里一个字都不写；停了之后又被续做的不算）。
主会话记录不在那个位置时，报告里写明「被停只按打断判」，不悄悄退回。
「结束本轮却不会醒」：子 agent 结束本轮、没交回，而它起过的后台任务（工具结果「Command running in background with ID: …」，
或前台命令跑满超时被挪进后台的「… was moved to the background (ID: …)」）都已收到完成通知
（会话记录里 origin 为 task-notification 的消息，或忙时排进队的 queued_command 附件），结束本轮之后也没有新的通知进来——没有任何东西会叫醒它，
过 60 秒就报，不等 10 分钟的「无动静」。
「中途停下没交回」（.claude/kb/milestone/03-third-txn.md「待优化」第 54 行）：上面那一种里一个后台任务都没起过的单列一类——它不是在等什么而没等到，
是活干到一半结束了本轮、没调交回工具。告警写结束本轮多久、最后一句话，出路给续做与重派两条：发消息让它接着做或交回；不回应的，
`python3 scripts/agents/agent-handover.py --agent <agent 号> --out <交接摘要>` 抽交接摘要、新派一个同类 agent 接着做。
报告里另起一节「中途停下没交回的子 agent」，被看的子 agent 里这一类几个、逐个一行（agent 号、类型、描述、最后一句、重派命令），一个都没有也写 0 个。
起过后台任务、都完了而没东西叫醒的照旧报「结束本轮却不会醒」。弄坏开关 AGENT_WATCH_BREAK=midwaymerged 照改前并进「结束本轮却不会醒」、不出那一节。

hook 的检出也在这里汇给主 agent：`.claude/hooks/bash-command-detector.sh` 执行前拒绝几种写法（列在它的文件头），其余只记不拦，把检出写进检出记录；
watch 每 15 秒读一次，读到被看子 agent 所在会话的检出就退出、叫醒主 agent。
告警的下一步按检出记录的 outcome 一栏说：「被拒」是执行前已拒、没执行，「只记」是照常执行；没写这一栏的旧记录按发现的起头认（detection_outcome）。
「被拒」的检出（钩子在执行前已经拒掉：写被拒、越出写范围、重型测试被拒、提交时才跑的检查被拒、不经内存包装被拒……）与判不了的脚本（重型测试闸没判全）一样，
只写进报告、不叫醒主 agent（.claude/main-agent.md「派出去之后」说的「记录不处置」那一类：拒绝信息与出路在它的工具结果里，主 agent 醒了没有可处置的）；
叫醒的只有「只记」的、照常在跑的命令。2026-10-10、2026-10-11 被拒的命令让看门狗退出，主 agent 重起了十来次。弄坏开关 AGENT_WATCH_BREAK=refusedwakes。
钩子 ⑭ 判不了的长活（检出以「长活判不了」起头，照常执行）也只写进报告、不叫醒：判不了不是出了事，主 agent 醒了没有可处置的。
弄坏开关 AGENT_WATCH_BREAK=longrunwakes 照改前叫醒。
范围声明钩子（.claude/hooks/run-scope-guard.sh）放行的每一条也写进检出记录，会话号在 scope_session_id、带 run_scope（命令与切开的四段）：它们不是检出、不叫醒，
属于被看会话（被看子 agent 与主 agent）的逐条列进报告的「范围声明」一节，一条一行写谁、命令与四段（种类、跑什么、范围、为什么），范围合不合适由主 agent 对着看。
弄坏开关 AGENT_WATCH_BREAK=scopeunlisted。

会话记录的位置：~/.claude/projects/<项目>/<会话 id>/subagents/agent-<id>.jsonl，同名 .meta.json 里有 agentType 与 description。
进程这一半只看运行这个脚本的那个 Claude 实例的后代进程（主 agent 与它派出的子 agent 起的命令都在里面），不看别的会话。

`--processes-only`：不给子 agent，只看进程这一半——主 agent 自己放后台的长命令没有会话记录可看，只有进程。
盯的是看门狗起的时候这个实例底下已经在跑的进程（整棵子树，不只叶子：gate.sh 这类长命令一段一段起叶子，叶子换了，命令本身还在跑；
看门狗自己那一支与 COMMAND_MARKERS_NOT_WATCHED 不算），它们都退出了（剩下没人收尸的僵尸也算退出）就退 0。
告警与进程一节同一套：写着的文件 --process-stale-minutes 没动、CPU 也不涨，stdin 若是管道、上游写端连同子孙也既不算也不写，报「进程无输出」，跑过 --process-max-minutes 报「进程过长」，
报了就退 3；--ack 进程:PID 与定时回报照旧。它不读 hook 的检出记录：认不出是哪个会话的。
父进程命令行是 `remind.sh start`（scripts/agents/remind.sh 的定时提醒）的 sleep 是等到点的计时，写着的文件本来就不动、CPU 本来就不涨，不报「进程无输出」也不报「进程过长」，
只在进程一节写一行「是定时提醒的计时」；`remind.sh stop` 底下的、别的脚本底下的 sleep 照报。每次重起提醒进程号都变，不放行就得每次重新 --ack，连报还会被终止。弄坏开关 AGENT_WATCH_BREAK=remindertimer。
给了 --session-dir（watch.sh --processes 从 CLAUDE_CODE_SESSION_ID 找）就同时查「交回之后后台还在跑」；没给就在报告里写明这一类没查。

「交回之后后台还在跑」：扫会话目录里全部子 agent（不只被看的那几个）。已交回、被停、或主会话记录里最近一次任务通知是
failed / killed / stopped（stopped：上一个 Claude 进程退出时它还没跑完）的子 agent，它用 run_in_background 起过的后台任务的输出文件
还有进程开着、而且它结束已过 --leftover-background-grace-minutes（交回那一刻最后一条完成通知可能还在路上），每个最上层的进程报一条，
--ack 进程:PID 确认。2026-09-24 查到两条交回之后丢下的 `until … do sleep` 跑了 5 个多小时，看门狗只盯没交回的，没人报。
被看的子 agent 全部交回时照旧退出，但退出前查一遍这一类；只剩宽限里的，等宽限过了再查一次才退。
只认会话记录里写着输出文件路径的后台任务：命令里自己用 `&`、`nohup` 放出去又改了输出去向的进程，与子 agent 之间没有可认的联系，这里认不出。
交回那一刻由交回闸 .claude/hooks/handback-guard.sh ④ 拦（同一套认法，从这里导入 background_start_in_result 与 own_background_tasks_still_running）：
交回时自己起的后台任务还有进程开着输出文件就拒交回；这里报的是闸管不到的（被停、失败，交回闸找不到会话记录或导入失败放行的）。

「跑满 N 小时要主 agent 问一次」：还没交回、没被停（任务通知也不是 failed / killed / stopped）的子 agent，从派发（它会话记录的第一条）起
每跑满 --ask-every-minutes（默认 60 分钟，即每个整点）报一次，N 是已跨过的最近那一格；主 agent 中途发的消息不重新计时。
那一格之后主 agent 给它发过消息就算问过了、这一格不再报：看门狗每次起都从头读，按「那一格之后有没有消息」判才做得到每个整点只报一次。
主 agent 的消息取两处里晚的那个：子 agent 会话记录里送达的（origin 为 coordinator 的消息，或忙时排进队、附件自己带 origin coordinator 的
queued_command），与主会话记录里 SendMessage 发给它的成功结果（忙着跑一条长工具调用的子 agent，消息要等它下一轮才落进它自己的会话记录）。
--ack <agent 号>:跑满 N 小时 只确认那一格，下一格照报。2026-09-24 几个子 agent 结束本轮去等后台任务、一等两个多小时，
主 agent 发了一条消息它们当场醒来、很快交回；看门狗只在后台任务全结束、通知没到时报「结束本轮却不会醒」，等的还开着就不报。

「重型测试没带前缀」：每次复检扫这个 Claude 实例底下的全部进程（看门狗自己那一支、另一个看门狗连同它的自检整棵不扫），
每个进程的 /proc/<pid>/cmdline（按 NUL 切开）与 /proc/<pid>/cwd 交给 .claude/hooks/lib_heavy_tests.py 的 classify_process 判
（与执行前的闸 heavy-test-guard.sh 同一份判定，按文件路径导入），判为重型、而 /proc/<pid>/environ 里没有 SINGLEFS_HEAVY_TESTS=commit
或 =user-request 的报，叫醒主 agent。`bash -c '…'` 这一层不单判：写在代码串里的前缀不在这个 shell 自己的环境里，它起的命令各自是一个进程、各判各的。
没带前缀、底下却有带前缀的重型进程的不报（`timeout 600 env SINGLEFS_HEAVY_TESTS=commit …` 里的 timeout）；一条链上只报最上面那个，
下一步给整棵子树从最底层往上逐个停的次序。归属照「交回之后后台还在跑」那套：它或它的上层开着哪个子 agent（或主 agent）后台任务的输出文件，
认不出写认不出。读不到 cmdline、cwd、environ 的（进程刚退出、权限不够）跳过不报；lib_heavy_tests.py 导入失败时报告里写明这一类没查。
--ack 进程:PID 确认。双机分片的驱动脚本 crash-point-replay-shard-run.sh 按它参数里树根下键那一行登记判（lib_heavy_tests.py 文件头）：
驱动自检（scripts/crash/crash-point-replay-shard-run.sh --selftest）在临时仓里登记 harness 档包的替身用例；树根里那一行读不到的（lib_heavy_tests.py 照重型算）靠下一段放行。
登记成轻的自证底下不判：一个进程起的脚本（bash / sh、python 后面那个，直接执行的取它自己）照 lib_heavy_tests.py 的 classify_selftest 判
（同一份登记表 .claude/hooks/selftest-registry.tsv，命令词要解到本仓那一份），是登记成 light、或 heavy-if-set:<变量> 而它环境里那个变量没设的自证，
它自己照判，它底下整棵不判。`<名>.sh --selftest` 先 exec 进同目录 `<名>-selftest.sh` 本体的（gate-staged.sh；
exec 之后参数里没有 --selftest，下面「样本自检底下不判」那条认不出），本体不带参数时按 `<名>.sh --selftest` 那一行判，`<名>.sh` 的正文里还要写着本体的文件名。
不在它底下、而参数里的路径或当前目录落在它临时目录里的也不报：它的临时目录是它底下进程的参数与当前目录里出现过的、临时根（它环境里的 TMPDIR，没有是 /tmp）
下一层名字像 mktemp / mkdtemp 起的（tmp.XXXXXXXXXX、tmpXXXXXXXX）那一层，它还在跑时才算。仓副本里路径尾部相同的自证、设了那个变量的、登记成 heavy 的、
表外的照判；路径只是像临时目录、没有登记成轻的自证在跑着用它的照报。报告那一行写出这两类各几个。2026-10-01 layer0-shard-run.sh --selftest
exec 进本体之后，本体在 mktemp 临时仓里起的驱动每一趟报一次「重型测试没带前缀」（进程 1810646、1819322，records/2026-10-01-对接上游测试脚本规范.md 第 625、626 行）。
轻型三条底下不判：一个进程起的脚本（同上取法）照 lib_heavy_tests.py 的 light_tier_command 是 `crash-point-replay.sh --quick`、
`checker-tier-formal-verification.sh --quick`、`checker-tier-crates-mutation-replay.sh --quick`（可再带 `--item <项>`）之一，它自己照判，它底下整棵不判
（子 agent 跑轻型三条，它们底下起的 checker 档 cargo 与 Verus 进程不是没带前缀的重型测试；经 run-with-memory-cap.sh 起的，包装那一层照判、它底下起的轻型三条那一个进程算数）。
不带 `--quick` 的同一条是重型，照判。报告那一行写出轻型三条底下不判了几个。
样本自检底下不判：一个进程的参数里有一个是 stage-selftest.sh（按文件名认）或恰好是 --selftest，它自己照判，它底下整棵不判
（stage-selftest.sh 在临时目录里带 --force 拿样本起 checker-tier-crash-point-replay 与 checker-tier-lkmm.sh、checker-tier-crates-mutation-replay.sh 这些重型阶段，
--selftest 建的临时仓里起 gate.sh）；报告那一行写出不判了几个。门禁阶段按文件名认（lib_heavy_tests.py 的 STAGE_KIND：checker-tier-crash-point-replay.sh 与
另四道 checker-tier-*；checker-tier-research-build-and-replay.sh 只在跑复跑那一格时算重型，--check research-unit-tests 只跑单测那一格的不算）。
lib_heavy_tests.py 另认的一类「提交时才跑的检查」（COMMIT_CHECK_KINDS：不重型的门禁阶段、上游几道 lint，以及 .claude/hooks/selftest-registry.tsv 登记为 heavy 或没登记的样本与 --selftest）
这里不报：执行前的闸拒它们，收工钩子 gate-reuse-check.sh 每次收工起的 gate-overlap.py 也在这一类，报了就是每次收工叫醒一次。
AGENT_WATCH_BREAK=heavycommitcheck 把这一类照重型报，自检里「不重型的门禁阶段不报」那一格必须判错。
执行前的闸 heavy-test-guard.sh 不看上层进程，判法不变。射程之外：在同一个 shell 里 source 进来跑、不另起进程的。2026-09-24 实十九把重型测试写进 run-chain.sh，
执行前的闸看不进脚本（records/2026-09-16-subagent拆分提案.md 第四十节第 16 行）。

「进程常驻内存过线」：这个 Claude 实例底下整棵子树（不只跑得久的叶子；看门狗自己那一支、别的看门狗与它的自检整棵不看）任一进程的常驻内存
（/proc/<pid>/statm 第二列乘页大小）超过它的阈值就报，叫醒主 agent。阈值逐个进程取，先到先用（.claude/kb/milestone/03-third-txn.md「待优化」第 45 行：
崩溃枚举用例的测试二进制平常就 10 到 21 GiB，固定 8 GiB 每趟都报）：
  ① 进程在内存包装（scripts/hosts/run-with-memory-cap.sh）起的 scope 里：/proc/<pid>/cgroup 里有 singlefs-memory-cap-<…>.scope 那一层，
     阈值是包装账上那一笔（<状态目录>/ledger/<单元>.json 的 cap，状态目录与 scripts/hosts/memory_peaks_lib.py 的 default_state_directory 同一个取法；
     账读不到时读那个 scope 的 memory.max）登记的上限乘 --process-rss-cap-percent（默认 90）；看门狗的根进程自己也在同一个 scope 里的不算（整个会话被包着时退回下两条）；
  ② 不在包装里、峰值表（memory_peaks_lib.py 的 default_peak_table）里有这条命令（键照 command_key 由它的命令行算）的：阈值是近几次实测里最大的峰值乘 CAP_FACTOR（1.5，
     与包装 auto 的取法一致）；乘出来不到下限的取下限：--process-rss-floor-mebibytes（默认 512 MiB，watch.conf 里写着，依据见那个选项）与 --process-rss-gibibytes 里小的那个
     （峰值表里记成 0.00 GiB 的 cat、sleep 这类，乘 1.5 还是零点几 MiB，常驻几 MiB 就会报；峰值记成 0 的整条不当阈值，走 ③）；
  ③ 都没有：--process-rss-gibibytes（默认 8 GiB，依据见那个选项）。--process-rss-gibibytes 写 0 整类不查。
  峰值表的库导入不了时 ② 不查，报告那一行写明原因。弄坏开关 AGENT_WATCH_BREAK=memoryfixedline 照改前一律按 --process-rss-gibibytes 判；
  memorypeakfloor 照改前不设下限（峰值乘 1.5 多小都当阈值）；memorypeakfloorfixed 乘出来不到下限的退回 --process-rss-gibibytes（峰值很小而常驻真过了下限的不报）。
  登记成轻的自证底下不判：过线的进程，它的上层（到看门狗的根进程为止，不含根进程）里有登记成轻的自证的不报——认法与「重型测试没带前缀」同一份
  （registered_light_selftest，登记表 .claude/hooks/selftest-registry.tsv），自证那一层自己照判；报告那一行写出这样不报的几个。
  （scripts/hosts/run-with-memory-cap.sh --selftest 底下故意填满自己上限的探针只跑几秒，看门狗退 3 叫醒主 agent、连报了四次。）
  lib_heavy_tests.py 导入失败时这一条不认、照报，报告那一行写明原因。弄坏开关 AGENT_WATCH_BREAK=memoryselftest 照改前不认登记成轻的自证；
  memoryselftestanyweight 上层参数里带 --selftest 的一律当登记成轻的自证、不看登记表（登记成 heavy 的自证底下的过线进程也不报）。
报的时候
写进程号、命令、常驻内存、已跑多久、归属（照「交回之后后台还在跑」那套认开着输出文件的子 agent，认不出写认不出）与「怎么办」。
整查（--interval-seconds）之外，watch 与 --processes-only 两次整查之间每隔 --detection-poll-seconds 再单查一遍常驻内存，过线就马上整查一次、叫醒主 agent：
涨得慢的等不到下一次整查；涨得快的（E142 那条变异每秒约 5 GB、十几秒吃满整机）单查也来不及，只有内存上限（scripts/hosts/run-with-memory-cap.sh）挡得住，
这一类告警是它之外的第二道。它只报不杀（不在「进程卡住」那一类里，连报多少次都不终止）；--ack 进程:PID 确认。
2026-09-25 两次整机 OOM：E142 的变异跑道里一条变异让测试无界分配，涨到 53.6 GiB 与 56.6 GiB 才被内核杀掉，连带杀了本地模型服务与 Claude Code 会话
（records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。

「长跑里有判红的行」（.claude/kb/milestone/03-third-txn.md「待优化」第 43 行：监控轮只看进度行、不数判定行，漏了一条红）：进程一节（--agents 与 --processes-only 两条路同一份）
另数判定行。这个 Claude 实例底下跑满 --process-report-minutes 的每个进程（不只叶子；看门狗自己那一支与 COMMAND_MARKERS_NOT_WATCHED 不算）开着写的文件，去重之后逐份数
行首（可带空白）是「✓ 」或「✗ 」的行（门禁、崩溃枚举用例、自证的判定行都这样起头），报告里一份一行「<文件>（进程 <号>）：✓ N ✗ M」，✗ 的列出前 VERDICT_RED_SHOWN 行；
有 ✗ 的报「长跑里有判红的行」，叫醒主 agent，出路是当场把这一趟 ✓ / ✗ 各几行、红的是哪几条记进调度记录。文件超过 VERDICT_SCAN_BYTES 的只数末尾那么多、写明「只数了末尾」。
看门狗的根进程自己开着写的文件不数（自证时根进程的标准输出是跑它的那一份日志）。--ack 进程:<号> 确认之后只进报告、不叫醒；这之后又多出来的红照样列在报告那一行里。
登记成轻的自证在自己临时目录里写的 ✗ 不报（开关轮故意弄坏的样本本来就该红；一趟门禁 code-tooling 的 research-script-selftests 格能报几十条）：
有 ✗ 的那份文件，写它的进程或它的上层（到看门狗的根进程为止）里有登记成轻的自证——认法与「重型测试没带前缀」同一份（registered_light_selftest，
登记表 .claude/hooks/selftest-registry.tsv）——而文件落在那份自证的临时目录里。它的临时目录：
  它环境里的 TMPDIR（没有是 /tmp）是单给它的，整个 TMPDIR 都算——往上跳过只是包着它的那几层（参数以它的参数收尾的：strace … --、
  scripts/gate/light-selftest-guard.py … --、timeout、nice），第一个不是包着它的上层的 TMPDIR 与它的不同（light-selftest-guard.py 给每条自证一个私有 TMPDIR，
  .claude/hooks/lib_selftest_scratch.py 判红、抛异常两条路拉起的子进程也是）；
  TMPDIR 与上层共用的（门禁阶段的私有 TMPDIR 里直接起的自证），只算 TMPDIR 下一层名字像 mktemp / mkdtemp 起的（SELFTEST_TEMPORARY_NAME），
  或以 lib_selftest_scratch.py 的两个探查目录前缀（PASS_PROBE_PREFIX、CHILD_PROBE_PREFIX，现读那份库；通过那条路在进程里换的 TMPDIR 就是它）起头的那一层。
这样的只在报告里写一行「在登记成轻的自证 … 的临时目录里，✗ 不报」，不告警；同一份自证写在临时目录外面的、没有自证上层的进程写在临时目录里的照报。
lib_selftest_scratch.py 读不了时探查目录前缀那一种不认，报告那一行写明原因。射程之外：共用 TMPDIR 的自证用自己起的名字 mktemp（mktemp -d "$TMPDIR/<名>.XXXXXX"）
建的临时目录认不出（名字不像 tmp.XXXX），那里的 ✗ 照报。
弄坏开关 AGENT_WATCH_BREAK=verdictlines 不数判定行；verdictselftest 照改前不认自证临时目录（自证临时目录里的 ✗ 照报）；
verdictselftestanyfile 登记成轻的自证写的文件不看在不在临时目录里一律不报；verdictselftestpathonly 只看路径里有没有一层像 mktemp 临时目录，不看是不是自证写的；
verdictselftestsharedtmpdir 与上层共用的 TMPDIR 也整个算；verdictselftestprivateignored 不认单给它的 TMPDIR，只认下一层；
verdictselftestwrapperseen 不跳过只是包着它的那几层（strace、timeout 这类与它同一个 TMPDIR，单给它的 TMPDIR 认不出）。

「长跑在用核数过少」：两半，取样与判法都在 scripts/hosts/core_usage.py（窗口、阈值、「同时在算的线程」、声明的写法在它的文件头）。
一、本机：这个 Claude 实例底下整棵子树（看门狗自己那一支、别的看门狗与它的自检整棵不看）里的重活进程——lib_heavy_tests.py 的 classify_process 认出的重型用法
（提交时才跑的检查那几类不算）或 runs_compiled_code 认出的跑编译出来的代码，与 .claude/hooks/host-resources-reminder.sh 认「重活」同一个口径——一条链上只取最上面那个，
整棵树算它的。每次整查给每个都取一次样（core_usage.py 的 tree_sample），样本存状态目录的 core-usage-samples/<进程号>-<起始时刻>.jsonl（两天没写的删掉），
跑过一个窗口（600 秒）的判：在用核数与同时在算的线程都低于登记数的四分之一的报，写「进程 <pid> 重活只用了 X 核（登记 N）」、声明的串行段与理由（声明文件从那棵树
进程环境里的 SINGLEFS_SERIAL_DECLARATIONS 找）、命令与出路，叫醒主 agent；不报的逐个列一行在用几核。树里有 multi-host-run.sh --peer 那一支的判成在等第二台、不报。
  并行在几个干活进程之间的门禁（AGGREGATED_PARALLEL_STAGES：crates 变异那一道，带不带 --quick 都算）与研究脚本（AGGREGATED_PARALLEL_SCRIPTS：合入后验证测试那一步的
  scripts/gate/test-targets-parallel.py）：这道的进程当那条链上最上面的重活、整棵树按那一趟的合计判，
  报的时候写「（门禁 <名> 这一趟，各干活进程合计）」，不往下拆到各干活进程里的单行 cargo test（单行本来只占一条线，逐个判就每一行都报）；
  合计也低于四分之一的照报。2026-10-21 变异轻型里单行 cargo test 被逐个报「长跑在用核数过少」、逐个 --ack（records/2026-10-18-里程碑三收尾与近期欠账.md 第 837 行）。
  弄坏开关 AGENT_WATCH_BREAK=mutationrows 照改前往下拆、逐个单行判。
二、别处写的告警记录：core_usage.py 的告警记录里最近 660 秒的（multi-host-run.sh --peer 判第二台、detached-run.sh watch 判脱离会话的单元），会话号是被看的会话
（--processes-only 是 --session-dir 那一个）、告警进程还活着的报，说明以「进程 <告警进程号> 」起头；没有会话号的只记一行、不叫醒。
  同一份告警记录里 scripts/hosts/run-with-memory-cap.sh 排队时写的（kind=memory-queue-peer-fits，它文件头「排队时去找别的机器」）另报「本机内存排队、别的机器放得下」：
  phase=queued 的要包装还活着，phase=refused 的（退了 252）包装不在了也报，phase=admitted 的（排了队之后在本机起跑了）不报；出路是记录里改去第二台的那条命令。
  同一份告警记录里 run-with-memory-cap.sh 收尾时写的三种（它文件头「峰值表」一段）各报各的，包装退了也报：kind=memory-peak-grew 报「内存峰值比记账大了 Z 倍」
  （工作量可能变了），kind=memory-own-cap 报「撞了自己的上限：上限给小了，或工作量变了」，kind=memory-squeezed 报「被总上限挤掉：结果不算数，重跑」；出路是记录里那一句。
  memory-peak-grew 只在实测到了那一次上限的 --memory-peak-alert-cap-percent（默认 50，watch.conf 写 50）时叫醒，不到的只在报告里记一行「只记进峰值表、不叫醒」
  （峰值表包装自己已记下这一次）；上限或实测读不出的、门槛写 0 的照叫醒。2026-10-21 三次「涨十几到几十倍」都不到 8G 上限的五分之一，主 agent 只能 --ack 重起
  （records/2026-09-16-subagent拆分提案.md 第二个「七十八」节）。
两半都在 --agents 与 --processes-only 两条路的整查里；--ack 进程:<pid> 确认；只报不停（不在「进程卡住」那一类里，连报多少次都不终止）。
2026-10-20 E185 第二段在第二台只用一个核跑了几个小时，没有一处报（records/2026-09-16-subagent拆分提案.md 第七十二节）。

「交回之后副本还在」：扫被看的子 agent 所在会话里全部子 agent（--processes-only 带 --session-dir 时扫那个会话），交回过（交回工具成功返回）
超过 --kept-copy-hours（默认 6，watch.conf 里写着）、交回正文里——或交回正文点名的报告文件（.md）里——写了一行「没删 <全路径>」
（上游交回闸 handback-scratch-check.sh 要子 agent 写的就是这一行），而那个路径还在的，每处报一条，交主 agent 核过没人在用之后删。
只认落在工作根、过渡期旧根、/tmp 与本进程临时目录底下的路径（仓副本与编译目录只建在那里）；交回不到阈值的、已经删掉的、没交回的不报；
--ack <agent 号>:交回之后副本还在 确认（条件类，路径都删了就作废）。只报不删。会话记录按大小与修改时刻缓存，没变就不重读。
2026-10-10 草稿根下攒了 292 个交回后没删的仓副本与编译目录（records/2026-09-16-subagent拆分提案.md 第六十八节）。

「草稿根里超阈值的编译目录与仓副本」：被看的子 agent 里有交回（或被停）的，报告里带一节：工作根整个、过渡期旧根 /tmp/claude-<uid> 下只看本会话的那一份（<旧根>/<项目>/<会话号>）里，
编译目录（目录里有签名对得上的 CACHEDIR.TAG）与仓副本（有 .git，或同时有 Cargo.toml 与 crates/）最后修改距今超过 --scratch-idle-hours（默认 6，watch.conf 里写着，依据是推的）的，
按最后修改时刻、最久没动的在前，每处一行写种类、路径与最后修改距今多久。「最后修改」取副本自己与往里三层条目的修改时刻最大的一个（不全树走一遍）。
只进报告、不叫醒、不删（删由主 agent 核过没人在用之后做）；没有子 agent 交回时不查；写 0 不查。放这里不放交回钩子：交回钩子的话只送到交回的子 agent，主 agent 看不到。
弄坏开关 AGENT_WATCH_BREAK=scratchhog、scratchhogage、scratchhogtag。

「根分区余量过低」：每次整查（--agents 与 --processes-only 两条路都查）读工作根（.claude/hooks/lib_work_root.py 的 work_root；还没建的往上取第一个在的上层）所在文件系统的可用量
（os.statvfs 的 f_bavail × f_frsize，与 df 的 Avail 同一个数），低于 --root-free-gib-alert（默认 128 GiB，watch.conf 里写着，依据是推的）就报，叫醒主 agent；不低的在报告里记一行可用多少。
告警写可用多少、阈值多少与出路：先查崩溃放量家目录（SINGLEFS_CRASH_AMPLIFICATION_HOME，没设是 ~/.local/share/singlefs/crash-amplification）runs/ 下各运行目录里的 fetched 与
library-for-peer*（双机驱动送第二台的库快照与取回的库，一份就是整个判定库那么大），再查各会话的副本（工作根下各草稿目录），以 du 为准，大的先看；只删没有进程在用的，别人在用的不删。
条件类告警：--ack 本机:根分区余量过低 确认之后只进报告，可用量回到阈值以上这条确认就作废；只报不删、不终止（不在「进程卡住」那一类里）。写 0 不查。
可用量的读法换得掉：root_free_report 收一个读可用字节数的函数（自证传假的）；环境变量 AGENT_WATCH_ROOT_FREE_BYTES 设了就当可用字节数、不读 statvfs（只给 --selftest 起的看门狗子进程用）。
2026-10-22 本机根分区写满、只剩 113M，是写文件撞上 No space left on device 才发现的，看门狗只报草稿根里闲置的编译目录、不看余量
（records/2026-10-13-里程碑四预开发.md 第 951 行）。弄坏开关 AGENT_WATCH_BREAK=rootfree。

「重启前旧进程留下的记录判已失效」：每次整查读 ~/.claude/sessions/<pid>.json（Claude Code 给每个进程登记的会话号、启动时刻与 procStart，pid 还在而且 /proc/<pid>/stat 第 22 列对得上才算活着），
没交回的子 agent 记录有一种算已失效：一、它所在的会话号不是本会话（--session-dir 的目录名，没给取 CLAUDE_CODE_SESSION_ID）、登记里也没有活着的 Claude 进程；
二、会话号还活着（重启后接着用同一个会话号），而它最后一条记录早于那个会话现在活着的最早一个进程的启动时刻。已失效的在报告里标「状态=已失效」，
不按「无动静」「工具调用过长」这类报，不算要盯的，也不拦着全部交回时退出；--discover 不再把它记成没结束的。登记读不出、或里面没有一份能解的，判不了，一律不当失效。
弄坏开关 AGENT_WATCH_BREAK=stalesession、stalesessionrestart。

「调度方不响应就终止」：「进程卡住」类的三种告警——子 agent 的「工具调用过长」、进程的「进程无输出」（写着的文件不涨、CPU 也不涨）、「重型测试没带前缀」——
watch 每次叫醒主 agent（退 3）时，没被 --ack 的各记一次连报，按进程号加告警种类记（键 `进程:<pid>:<告警种类>`；子 agent 的工具调用过长没有进程号，键 `<agent 号>:工具调用过长`）。
这一次叫醒里没再报的（条件没了、或主 agent --ack 了）作废，连报从头数；没有告警的退出（定时回报、全部交回、盯着的进程都退出了）把记录清空。
同一个键连报到 --terminate-after-unacknowledged-reports 次（默认 0 不终止；watch.conf 写 3，主 agent 经 watch.sh 起的看门狗都照它）：
进程告警经 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>` 终止那一个进程（先 TERM、过宽限再 KILL，只停它、不带子进程），
在报告最前面写「终止了 <pid>（<告警种类>，连报 N 次没处置）」，那个键从记录里删掉；一次叫醒至多终止一个，同时到线的别的写「下一次再判」留着；
proc.py 拒了或没停成的写「想终止 <pid> …没成」留着。子 agent 的工具调用过长到线只报「没有进程号，脚本停不了子 agent，只报」（停子 agent 用停止它的工具，归主 agent）。
进程号复用：记录里同时记那个进程的起始时刻（/proc/<pid>/stat 第 22 列），这一次读到的起始时刻不同就是另一个进程，连报从 1 数；读不到起始时刻的不终止。
记录落在会话的状态目录（--session-dir 的目录名，与 watch-acks.tsv 同一处）：watch 盯子 agent 时是 watch-repeats.tsv，--processes-only 时是 watch-repeats-processes.tsv，
一行一个键，三列用制表符分开：键、连报次数、进程起始时刻（子 agent 的写 -）。没给 --session-dir 的（watch.sh <agent 号> 那种起法）没处记：报告里写明，到线也不终止。
--processes-only 只认这一次的 --ack（它不读状态目录里记下的确认）。report 只看一眼，不记也不终止。终止用的脚本可由环境变量 AGENT_WATCH_PROC_SCRIPT 换掉，只给 --selftest 换成记录调用的桩。
弄坏开关 AGENT_WATCH_BREAK=repeatsforgotten、terminateacked、terminatezero。

弄坏开关 AGENT_WATCH_BREAK=<项>（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
（项名是那条失败落在哪个自证函数里：selftest_in、selftest_session_watch_in、selftest_scratch_hogs、selftest_stale_sessions、selftest_leftover_background、selftest_pipe_upstream、
selftest_heavy_tests、selftest_heavy_selftest_tree、selftest_light_tier_tree、selftest_memory、selftest_core_usage、selftest_unacknowledged_terminations、selftest_root_free、selftest_verdict_selftest_scratch、selftest_ack_concurrency，临时目录那三格写格名；括号里是那几条「✗ 自检：」的开头；红了自证退 1）：
  AGENT_WATCH_BREAK=ack  split_acknowledged() 不认 --ack，确认过的告警照样叫醒
      selftest_in（「✗ 自检：「工具调用过长」被 --ack 确认之后看门狗应当到点退出码 4 并写「已确认接着盯」」、「✗ 自检：派发 125 分钟、--ack 了「跑满 2 小时」那一格时看门狗不该叫醒」）、
      selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」）、
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）、selftest_memory（「✗ 自检：过线进程 --ack 进程:」）、
      selftest_session_watch_in（「✗ 自检：--discover 应当自己找出没结束的 discoveredlong、确认过的不叫醒、到点退 4」、「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、
      「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」）、selftest_root_free（「✗ 自检：根分区余量过低被 --ack 本机:根分区余量过低 确认之后」）
      「✗ 自检：看门狗的报告应当带「交回之后副本还在」告警与一行汇总」
      「✗ 自检：「长跑在用核数过少」的进程都 --ack 进程:<pid> 之后看门狗不该叫醒」
  AGENT_WATCH_BREAK=acksneverpruned  pruned_acks() 不作废条件已经不在的确认，全部留着
      selftest_session_watch_in（「✗ 自检：条件类告警的确认」）
  AGENT_WATCH_BREAK=acksunlocked  update_acks() 读改写 watch-acks.tsv 不拿 watch-acks.lock 的 flock（改前的样子）
      selftest_ack_concurrency（「✗ 自检：watch-acks.tsv 两只同时写」）
  AGENT_WATCH_BREAK=acksnotpersisted  read_acks() 不读状态目录里记下的确认
      selftest_session_watch_in（「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」、「✗ 自检：已有一个在盯时给的 --ack 应当记进状态目录」、
      「✗ 自检：条件类告警的确认」）
  AGENT_WATCH_BREAK=backgroundwait  waiting_on_its_own_background_tasks() 一律交 False，结束本轮在等自己后台任务的也报「无动静」
      selftest_in（「✗ 自检：子 agent waitinglong」、「✗ 自检：子 agent textsettled」、「✗ 自检：子 agent stopsequence」、「✗ 自检：前台命令超时被挪进后台」）、
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」）
  AGENT_WATCH_BREAK=banned  agent_alerts() 不报禁用命令（pgrep -f / pkill -f / killall）
      selftest_in（「✗ 自检：子 agent banned」、「✗ 自检：子 agent bannedafterheredoc」）
  AGENT_WATCH_BREAK=bannedrefused  agent_alerts() 被钩子拒了的禁用命令也报
      selftest_in（「✗ 自检：子 agent bannedrefused」）
  AGENT_WATCH_BREAK=bannedwindow  agent_alerts() 不看禁用命令发出多久，窗口之外的旧命令也报
      selftest_in（「✗ 自检：子 agent bannedold」）
  AGENT_WATCH_BREAK=context  agent_alerts() 上下文过线也不报「该交接」
      selftest_in（「✗ 自检：子 agent hugecontext」）
  AGENT_WATCH_BREAK=contextzero  agent_alerts() 不看 --context-tokens 是不是 0，线写 0 也照报「该交接」
      selftest_in（「✗ 自检：子 agent hugecontext 在 --context-tokens 0 时」）
  AGENT_WATCH_BREAK=corerecords  core_usage_report() 不读告警记录（multi-host-run.sh --peer、detached-run.sh watch 写的那些叫不醒主 agent）
      selftest_core_usage（「✗ 自检：告警记录应当只把本会话、没过期、告警进程还活着的那一条（fresh）报出来」）
      「✗ 自检：内存包装排队时写的告警记录」
      「✗ 自检：内存包装收尾时写的峰值涨了的告警记录」
      「✗ 自检：实测涨 30 倍但不到上限一半的峰值涨了的告警记录」
      「✗ 自检：内存包装收尾时写的撞顶与被挤掉的告警记录」
  AGENT_WATCH_BREAK=mutationrows  core_usage_report() 不认 AGGREGATED_PARALLEL_STAGES 与 AGGREGATED_PARALLEL_SCRIPTS：变异门禁派出的单行 cargo test、按测试目标并行起的各条 cargo test 逐个判（改前的判法）
      selftest_core_usage_aggregated（「✗ 自检：变异门禁这一趟八个干活进程各忙一条线」、「✗ 自检：变异门禁这一趟三个干活进程都只睡」、
      「✗ 自检：按测试目标并行起的脚本这一趟三个干活进程都只睡」）
  AGENT_WATCH_BREAK=coreusage  core_usage_report() 什么都不查（本机重活不取样、告警记录也不读）
      selftest_core_usage（「✗ 自检：1 条线程忙等的重活进程」、「✗ 自检：声明了串行的重活进程」、「✗ 自检：报告里应当有一行「长跑在用核数过少：查了…」」、
      「✗ 自检：告警记录应当只把本会话」、「✗ 自检：看门狗 --processes-only 盯着 1 条线程忙等的重活进程时」、「✗ 自检：「长跑在用核数过少」的进程都 --ack」、
      「✗ 自检：内存包装排队时写的告警记录」、「✗ 自检：内存包装收尾时写的峰值涨了的告警记录」、「✗ 自检：实测涨 30 倍但不到上限一半的峰值涨了的告警记录」、
      「✗ 自检：内存包装收尾时写的撞顶与被挤掉的告警记录」）、
      selftest_core_usage_aggregated（「✗ 自检：变异门禁这一趟三个干活进程都只睡」、「✗ 自检：按测试目标并行起的脚本这一趟三个干活进程都只睡」）
  AGENT_WATCH_BREAK=cost  自检里把用量样本的 full_cache_rewrite_tokens 读成 0
      selftest_in（「✗ 自检：用量样本的 full_cache_rewrite_tokens 应当是」）
  AGENT_WATCH_BREAK=cpu  process_alerts() 不看 CPU 涨没涨，在算的进程也报「进程无输出」
      selftest_in（「✗ 自检：写着的文件一小时没动、CPU 一直在涨的进程」、「✗ 自检：CPU 在涨的进程」）
  AGENT_WATCH_BREAK=deadline  run() 不按回报周期截短复检间隔
      selftest_in（「✗ 自检：复检间隔比回报周期长时看门狗没有按回报周期退出」）
  AGENT_WATCH_BREAK=detections  read_detections() 一条检出都不读（每条都当别的会话的跳过）
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」」、「✗ 自检：检出记录属于「session」「主 agent None」」）、selftest_session_watch_in（「✗ 自检：hook 检出」）
  AGENT_WATCH_BREAK=detectiontype  read_detections() 检出的主体不写 agent 号、只写类型
      selftest_session_watch_in（「✗ 自检：hook 检出」）
  AGENT_WATCH_BREAK=discoverfallback  build_report() 带 --discover 而一个没结束的都找不到时，退回「会话里最近动过的全部子 agent」
      selftest_session_watch_in（「✗ 自检：--discover 时会话里的子 agent 都交回了应当接着盯到点退 4、不报全部交回」）
  AGENT_WATCH_BREAK=discovernone  discovered_agent_ids() 一个都找不出来
      selftest_session_watch_in（「✗ 自检：--discover 应当自己找出没结束的 discoveredlong、确认过的不叫醒、到点退 4」、
      「✗ 自检：--discover 的出路句应当是不带 agent 号的 bash scripts/agents/watch.sh」、「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、
      「✗ 自检：已有一个在盯时给的 --ack 应当记进状态目录」（在盯的那一个认不出 attackhuge 没结束，下一次整查把这一条事件类确认剪掉）、「✗ 自检：条件类告警的确认」）
  AGENT_WATCH_BREAK=display  command_for_display() 不剥 Bash 工具的那层包装，原样显示
      selftest_in（「✗ 自检：Bash 工具包着的命令在报告里应当显示成「cargo test --release」」、「✗ 自检：Bash 工具包着的命令在报告里应当显示成「grep 'x' f」」）
  AGENT_WATCH_BREAK=finished  AgentTranscript._read() 调过交回工具、没有未完的工具调用也不记成已交回
      selftest_in（「✗ 自检：子 agent finished：应当告警 无」、「✗ 自检：子 agent toolafterhandback：应当告警 无」、「✗ 自检：子 agent inquiryhandedback：应当告警 无」、
      「✗ 自检：子 agent finished 应当算交回或被停」、「✗ 自检：子 agent notifiedafterhandback 应当算交回或被停」、「✗ 自检：子 agent toolafterhandback 应当算交回或被停」、
      「✗ 自检：子 agent inquiryhandedback 应当算交回或被停」、「✗ 自检：子 agent refusedthendelivered 应当算交回或被停」、
      「✗ 自检：被看的子 agent 都交回或被停时看门狗应当退出码 0」、「✗ 自检：一个结束本轮没交回、一个已交回时看门狗应当退出码 4」、
      「✗ 自检：定时回报的出路句应当是」）、
      selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个」、「✗ 自检：交回 30 秒、宽限 2 分钟的 leftovergrace 应当记成宽限里」、
      「✗ 自检：被看的子 agent 全部交回、它丢下的后台进程还在时」、「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」、
      「✗ 自检：--processes-only 带 --session-dir 时也应当报」、「✗ 自检：被看的子 agent 刚交回、后台进程还在宽限里时」；
      run_leftover_watch 那几次看门狗等不到交回、到 60 秒没退的，退出码记成「超时（60 秒没退出）」照原判据判红，不抛异常）、
      selftest_kept_copies（「✗ 自检：「交回之后副本还在」应当只报交回超过 6 小时、交回正文或它点名的报告里写了没删而路径还在的」、「✗ 自检：看门狗的报告应当带「交回之后副本还在」告警与一行汇总」）、
      selftest_scratch_hogs（「✗ 自检：有子 agent 交回时看门狗的报告应当列出超阈值的」）；
      selftest_session_watch_in 那几格不跟着红：那一段的看门狗都带 --idle-minutes 600，交回了的被当成没交回，也到不了告警的线
  AGENT_WATCH_BREAK=handback  AgentTranscript._read() 只结束本轮、没调交回工具的也记成已交回
      selftest_in（「✗ 自检：子 agent nowake」、「✗ 自检：子 agent orphantask」、「✗ 自检：子 agent queuedresume」、「✗ 自检：子 agent inquiryfailed」、
      「✗ 自检：子 agent textsettled 的状态应当是「本轮结束、没交回」」、「✗ 自检：子 agent stopsequence 的状态应当是「本轮结束、没交回」」、「✗ 自检：子 agent waiting 应当不算交回或被停」、
      「✗ 自检：子 agent nowake 应当不算交回或被停」、「✗ 自检：子 agent wokenup 应当不算交回或被停」、「✗ 自检：子 agent waitinglong 应当不算交回或被停」、「✗ 自检：子 agent movedbackground 应当不算交回或被停」、
      「✗ 自检：子 agent queuedresume 应当不算交回或被停」、「✗ 自检：前台命令超时被挪进后台」、「✗ 自检：一个结束本轮没交回、一个已交回时看门狗应当退出码 4 并列出「还没交回、没被停的子 agent（1 个，另有还没会话记录的 0 个）：waiting」」、
      「✗ 自检：定时回报的出路句应当是「→ 接着盯就用 run_in_background 再起一个看门狗：bash scripts/agents/watch.sh waiting」」、「✗ 自检：复检间隔 240 秒、回报周期 3 秒时看门狗应当到点退出码 4」）、
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」）、selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个」、
      「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」）
      「✗ 自检：子 agent midway：应当告警 ['中途停下没交回']」
      「✗ 自检：报告里应当另起一节「中途停下没交回的子 agent：被看的 3 个里 2 个」」
      「✗ 自检：子 agent textthenended」、「✗ 自检：子 agent textthenended 的状态应当是「本轮结束、没交回」」、「✗ 自检：子 agent noparenttext 的状态应当是「本轮结束、没交回」」
  AGENT_WATCH_BREAK=handoveranytype  agent_alerts() 不分类型，所有子 agent 上下文过线都报「该交接」
      selftest_session_watch_in（「✗ 自检：子 agent attackhuge」）
  AGENT_WATCH_BREAK=heavycommitcheck  heavy_test_report() 把「提交时才跑的检查」照重型报
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、「✗ 自检：报告里应当有一行」、
      「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）
  AGENT_WATCH_BREAK=heavyexempt  heavy_test_report() 底下有带前缀重型进程的上层也报
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、「✗ 自检：报告里应当有一行」、
      「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）
  AGENT_WATCH_BREAK=heavyprefix  heavy_test_report() 不认 SINGLEFS_HEAVY_TESTS 前缀，带了前缀的也报
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、「✗ 自检：报告里应当有一行」、
      「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）
  AGENT_WATCH_BREAK=heavysampleselftest  heavy_test_report() 不放行样本自检（stage-selftest.sh、--selftest）底下的整棵
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、「✗ 自检：报告里应当有一行」、「✗ 自检：报告那一行应当写出样本自检底下不判的进程数」、
      「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）
  AGENT_WATCH_BREAK=heavyscan  heavy_test_report() 一个进程都不扫
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、
      「✗ 自检：没带前缀的层 0 进程的告警应当写类别「层 0」、「没带 SINGLEFS_HEAVY_TESTS=commit」」、「✗ 自检：带 SINGLEFS_HEAVY_TESTS=yes 的告警应当写「带的是 SINGLEFS_HEAVY_TESTS=yes」」、
      「✗ 自检：timeout 包着没带前缀的只报 timeout 那一个」、「✗ 自检：报告里应当有一行」、「✗ 自检：报告那一行应当写出样本自检底下不判的进程数」、「✗ 自检：看门狗 --processes-only 盯着没带前缀的层 0 进程时应当报」、
      「✗ 自检：看门狗 --agents 盯着没带前缀的层 0 进程时应当报」、「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）、
      selftest_heavy_selftest_tree（「✗ 自检：没有自证祖先、同一条双机分片驱动命令应当报」、「✗ 自检：设了 SAMPLE_REAL_TOOLS 的同一份自证（heavy-if-set）底下的驱动应当报」、
      「✗ 自检：仓副本里路径尾部相同的自证底下的驱动应当报」、「✗ 自检：报告那一行应当写出登记成轻的自证底下不判、它临时目录里不报的进程数」）
      「✗ 自检：同一棵树换成不带 --quick 应当只报最上面的包装那一层」
      「✗ 自检：报告那一行应当写出轻型三条底下不判的进程数」
  AGENT_WATCH_BREAK=heavyselftestancestor  registered_light_selftest() 一律交 None：登记成轻的自证认不出，它底下整棵照判、它的临时目录也不收
      selftest_heavy_selftest_tree（「✗ 自检：登记成轻的自证（exec 进本体）底下、它临时仓里起的双机分片驱动不该报」、
      「✗ 自检：不在自证底下、路径落在登记成轻的自证临时目录里的驱动不该报」、「✗ 自检：报告那一行应当写出登记成轻的自证底下不判、它临时目录里不报的进程数」）、
      selftest_memory（「✗ 自检：登记成轻的自证底下的过线进程不该报」、「✗ 自检：报告那一行应当写出登记成轻的自证底下过线不报的进程数」）、
      selftest_verdict_selftest_scratch（「✗ 自检：登记成轻的自证在自己临时目录里写的 ✗ 不该报」）
  AGENT_WATCH_BREAK=heavyselftestanycopy  registered_light_selftest() 不核命令词是不是本仓那一份：路径尾部相同的仓副本里的自证照登记表放行
      selftest_heavy_selftest_tree（「✗ 自检：仓副本里路径尾部相同的自证底下的驱动应当报」）
  AGENT_WATCH_BREAK=heavyselftestcondition  registered_light_selftest() 不看自证进程的环境：heavy-if-set:<变量> 设了也当轻的
      selftest_heavy_selftest_tree（「✗ 自检：设了 SAMPLE_REAL_TOOLS 的同一份自证（heavy-if-set）底下的驱动应当报」）
  AGENT_WATCH_BREAK=heavyselftestpathonly  heavy_test_report() 路径里有一层名字像 mktemp 临时目录的就不报，不看有没有登记成轻的自证在用它（放宽的判法）
      selftest_heavy_selftest_tree（「✗ 自检：没有自证祖先、同一条双机分片驱动命令应当报」、「✗ 自检：设了 SAMPLE_REAL_TOOLS 的同一份自证（heavy-if-set）底下的驱动应当报」、
      「✗ 自检：仓副本里路径尾部相同的自证底下的驱动应当报」）
  AGENT_WATCH_BREAK=heavyselftesttemporary  heavy_test_report() 不放行落在登记成轻的自证临时目录里、却不在它底下的进程
      selftest_heavy_selftest_tree（「✗ 自检：不在自证底下、路径落在登记成轻的自证临时目录里的驱动不该报」、「✗ 自检：报告那一行应当写出登记成轻的自证底下不判、它临时目录里不报的进程数」）
  AGENT_WATCH_BREAK=heavylighttier  light_tier_run() 一律交 None：轻型三条认不出，它底下起的 checker 档 cargo 照判
      selftest_light_tier_tree（「✗ 自检：经内存包装起的 crash-point-replay.sh --quick 底下的 cargo 不该报」、「✗ 自检：报告那一行应当写出轻型三条底下不判的进程数」）
  AGENT_WATCH_BREAK=heavylighttierargs  light_tier_run() 只看脚本名、不看参数是不是轻型那一种写法：不带 --quick 的同一条也当轻型三条
      selftest_light_tier_tree（「✗ 自检：同一棵树换成不带 --quick 应当只报最上面的包装那一层」）
  AGENT_WATCH_BREAK=heavytopmost  heavy_test_report() 一条链上每一层都报，不只报最上面那个
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、「✗ 自检：报告里应当有一行」、
      「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）
      「✗ 自检：同一棵树换成不带 --quick 应当只报最上面的包装那一层」
  AGENT_WATCH_BREAK=heavywatchdog  heavy_test_report() 不跳过另一个看门狗连同它的自检
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、「✗ 自检：报告里应当有一行」、
      「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）
  AGENT_WATCH_BREAK=heavywrapper  heavy_test_report() 把 bash -c '…' 这一层也单判
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」应当只报 ['timeout 包着没带前缀的', '不带前缀直接起的 checker-tier-cr」、「✗ 自检：报告里应当有一行」、
      「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）
  AGENT_WATCH_BREAK=heredoc  command_without_heredoc_bodies() 不去 heredoc 正文，正文里写的命令也当成在执行
      selftest_in（「✗ 自检：子 agent bannedheredoc」）
  AGENT_WATCH_BREAK=holders  background_task_processes() 一个都不列
      selftest_in（「✗ 自检：结束本轮等自己的后台任务时应当列出开着那个输出文件的进程」）
  AGENT_WATCH_BREAK=idle  agent_alerts() 不报「无动静」
      selftest_in（「✗ 自检：子 agent idle」）
  AGENT_WATCH_BREAK=infoalertwakes  split_informational() 把只记不叫醒的条件告警照旧叫醒
      selftest_in（「✗ 自检：只有「等待循环」这类只记不叫醒的告警时」）
  AGENT_WATCH_BREAK=infowakes  read_detections() 只记不叫醒的检出也叫醒（「写被拒」另由执行前已拒那一条挡着，红的是「重型测试闸没判全」那一条）
      selftest_session_watch_in（「✗ 自检：hook 检出：写被拒的与重型测试闸没判全的应当只记进报告不叫醒」、「✗ 自检：hook 检出：钩子 ⑭ 判不了的长活」）
  AGENT_WATCH_BREAK=inquiry  routine_inquiry_alert() 一律不报「跑满 N 小时要主 agent 问一次」
      selftest_in（「✗ 自检：子 agent inquiry61」、「✗ 自检：子 agent inquiry125」、「✗ 自检：子 agent inquirymessagebefore」、
      「✗ 自检：派发 125 分钟、--ack 了「跑满 1 小时」那一格时看门狗应当照报 2 小时那一格并退出码 3」、「✗ 自检：派发 125 分钟、--ack 了「跑满 2 小时」那一格时看门狗不该叫醒」）、
      selftest_session_watch_in（「✗ 自检：整点询问应当附上 progress.md 的末几行」）
  AGENT_WATCH_BREAK=inquiryack  split_acknowledged() 不认短写「<agent 号>:跑满 N 小时」的确认
      selftest_in（「✗ 自检：派发 125 分钟、--ack 了「跑满 2 小时」那一格时看门狗不该叫醒」）、selftest_session_watch_in（「✗ 自检：--discover 应当自己找出没结束的 discoveredlong、确认过的不叫醒、到点退 4」、
      「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」）
  AGENT_WATCH_BREAK=inquirydelivered  AgentTranscript._read() 不认忙时排进队、附件自带 origin coordinator 的消息
      selftest_in（「✗ 自检：子 agent inquirydeliveredafter」、「✗ 自检：子 agent inquiryqueuedafter」）
  AGENT_WATCH_BREAK=inquiryended  routine_inquiry_alert() 不看主会话里的任务通知，已结束（failed / killed / stopped）的也报
      selftest_in（「✗ 自检：子 agent inquiryfailed」）
  AGENT_WATCH_BREAK=inquiryreset  routine_inquiry_alert() 从主 agent 上一次消息起重新计时（改前的判法）
      selftest_in（「✗ 自检：子 agent inquirymessagebefore」）
  AGENT_WATCH_BREAK=inquirysent  AgentTranscript.last_main_agent_message_timestamp() 不读主会话记录里 SendMessage 的成功结果
      selftest_in（「✗ 自检：子 agent inquirysentafter」）
  AGENT_WATCH_BREAK=helpformat  argparse 的 help 文本里两处百分号不转义（改前的样子：--help 抛 ValueError: unsupported format character）
      selftest_in（「✗ 自检：--help 应当退出码 0 并打出用法」）
  AGENT_WATCH_BREAK=staleselftestscratch  remove_stale_selftest_scratch() 一个都不删（改前的样子：自证被 SIGKILL 打断留下的 agent-watch-selftest-scratch-* 一直留着）
      selftest_stale_scratch_cleanup（「✗ 自检：被杀的自证留下的 agent-watch-selftest-scratch-*」）
  AGENT_WATCH_BREAK=keepscratch  selftest() 的临时目录走回 mkdtemp 不删
      通过那条路（「✗ 自检：通过那条路上临时目录没删」）、red 那条路（「✗ 自检：red 那条路上临时目录没删」）、crash 那条路（「✗ 自检：crash 那条路上临时目录没删」）
  AGENT_WATCH_BREAK=keptcopy  kept_copy_report() 一条都不报
      selftest_kept_copies（「✗ 自检：「交回之后副本还在」应当只报交回超过 6 小时」、「✗ 自检：看门狗的报告应当带「交回之后副本还在」告警与一行汇总」）
  AGENT_WATCH_BREAK=keptcopyage  kept_copy_report() 不看交回了多久，交回不到 --kept-copy-hours 的也报
      selftest_kept_copies（「✗ 自检：「交回之后副本还在」应当只报交回超过 6 小时」：keptrecent 也报了）
  AGENT_WATCH_BREAK=keptcopyexists  kept_copy_report() 不看路径还在不在，已经删掉的也报
      selftest_kept_copies（「✗ 自检：「交回之后副本还在」应当只报交回超过 6 小时」：deleted-repo 也报了）
  AGENT_WATCH_BREAK=leftover  output_files_held_open() 一个开着输出文件的进程都不找
      selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个」、「✗ 自检：交回 30 秒、宽限 2 分钟的 leftovergrace 应当记成宽限里、给出宽限到期的时刻」、
      「✗ 自检：被看的子 agent 全部交回、它丢下的后台进程还在时」、「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」、「✗ 自检：--processes-only 带 --session-dir 时也应当报」、
      「✗ 自检：被看的子 agent 刚交回、后台进程还在宽限里时看门狗应当等宽限过了再查、报」）、selftest_heavy_tests（「✗ 自检：没带前缀的层 0 进程的告警应当写类别「层 0」、「没带 SINGLEFS_HEAVY_TESTS=commit」」）、
      selftest_memory（「✗ 自检：过线进程的告警应当写常驻内存、阈值、已跑多久与归属「子 agent memowner」」）
  AGENT_WATCH_BREAK=leftoverack  split_acknowledged() 不认「交回之后后台还在跑」的 --ack 进程:PID
      selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」）
  AGENT_WATCH_BREAK=leftoverexit  run() 只剩宽限里的后台进程时也照样退出
      selftest_leftover_background（「✗ 自检：被看的子 agent 刚交回、后台进程还在宽限里时看门狗应当等宽限过了再查、报」）
  AGENT_WATCH_BREAK=leftovergrace  leftover_background_report() 不等 --leftover-background-grace-minutes 宽限
      selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个」、「✗ 自检：交回 30 秒、宽限 2 分钟的 leftovergrace 应当记成宽限里、给出宽限到期的时刻」）
  AGENT_WATCH_BREAK=leftovernotified  AgentTranscript.ended() 不认任务通知 failed / killed / stopped 为结束
      selftest_in（「✗ 自检：子 agent inquiryfailed」）、selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个」）
  AGENT_WATCH_BREAK=loop  agent_alerts() 不报等待循环
      selftest_in（「✗ 自检：子 agent waitloop」、「✗ 自检：只有「等待循环」这类只记不叫醒的告警时」）
  AGENT_WATCH_BREAK=loopnoise  read_detections() 只有「没有 timeout 的等待循环」的检出也叫醒
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」」）
  AGENT_WATCH_BREAK=longrunwakes  INFO_DETECTION_PREFIXES 不含「长活判不了」：钩子 ⑭ 判不了的长活照改前叫醒
      selftest_session_watch_in（「✗ 自检：hook 检出：钩子 ⑭ 判不了的长活（主 agent 的、子 agent l1 的）应当只记进报告、不叫醒」）
  AGENT_WATCH_BREAK=memoryfixedline  memory_threshold() 一律按 --process-rss-gibibytes 判，不看内存包装的上限与峰值表（改前的判法）
      selftest_memory（「✗ 自检：内存包装里的进程应当照包装上限判」、「✗ 自检：峰值表里有这条命令的进程应当照峰值判」、「✗ 自检：峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报」、「✗ 自检：峰值很小、常驻过了下限的进程应当照下限报」）
  AGENT_WATCH_BREAK=memorypeakfloor  memory_threshold() 不设下限，峰值乘 1.5 多小都当阈值（改前的判法）
      selftest_memory（「✗ 自检：峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报」；「✗ 自检：峰值很小、常驻过了下限的进程应当照下限报」也红：照报了，阈值写的不是取下限）
  AGENT_WATCH_BREAK=memorypeakfloorfixed  memory_threshold() 峰值乘 1.5 不到下限的退回 --process-rss-gibibytes，不取下限
      selftest_memory（「✗ 自检：峰值很小、常驻过了下限的进程应当照下限报」；「✗ 自检：峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报」与「✗ 自检：峰值表里有这条命令的进程应当照峰值判」也红：报告里取下限的那几个记成了照 --process-rss-gibibytes）
  AGENT_WATCH_BREAK=memorypoll  memory_alarm_between_checks() 两次整查之间不单查常驻内存
      selftest_memory（「✗ 自检：看门狗 --processes-only 起了之后才冒出来的过线进程」、「✗ 自检：看门狗 --agents 起了之后才冒出来的过线进程」）
  AGENT_WATCH_BREAK=memoryqueue  core_usage_report() 把内存包装排队时写的告警记录当成核数过少的那一种（改前的读法：报成「长跑在用核数过少」、退了 252 的不报、起跑了的照报）
      selftest_core_usage（「✗ 自检：内存包装排队时写的告警记录」）
  AGENT_WATCH_BREAK=memorygrew  core_usage_report() 把内存包装收尾时写的「峰值比记账大」记录当成核数过少的那一种（改前的读法：报成「长跑在用核数过少」、包装退了就不报）
      selftest_core_usage（「✗ 自检：内存包装收尾时写的峰值涨了的告警记录」、「✗ 自检：实测涨 30 倍但不到上限一半的峰值涨了的告警记录」）
  AGENT_WATCH_BREAK=memorygrewfloor  memory_peak_below_alert_floor() 一律交 None：峰值涨了离上限多远都叫醒（改前的门槛）
      selftest_core_usage（「✗ 自检：实测涨 30 倍但不到上限一半的峰值涨了的告警记录」）
  AGENT_WATCH_BREAK=memoryoutcome  core_usage_report() 把内存包装退 250、254 时写的记录当成核数过少的那一种（改前的读法，同上）
      selftest_core_usage（「✗ 自检：内存包装收尾时写的撞顶与被挤掉的告警记录」）
  AGENT_WATCH_BREAK=memoryscan  memory_report() 一个进程都不查常驻内存
      selftest_memory（「✗ 自检：「进程常驻内存过线」应当只报写满 256 MiB 的进程」、「✗ 自检：过线进程的告警应当写常驻内存、阈值、已跑多久与归属「子 agent memowner」」、
      「✗ 自检：过线进程的「怎么办」应当写怎么看还涨不涨、按进程号停它的命令、查清之前别照原样重派、带上限重派与 --ack 进程:」、「✗ 自检：报告里应当有一行」、「✗ 自检：看门狗 --processes-only 盯着常驻内存过线的进程时应当报」、
      「✗ 自检：看门狗 --agents 盯着常驻内存过线的进程时应当报」、「✗ 自检：过线进程 --ack 进程:」、「✗ 自检：看门狗 --processes-only 起了之后才冒出来的过线进程」、「✗ 自检：看门狗 --agents 起了之后才冒出来的过线进程」、
      「✗ 自检：峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报」、「✗ 自检：峰值很小、常驻过了下限的进程应当照下限报」、
      「✗ 自检：不在登记成轻的自证底下的过线进程应当照报」、「✗ 自检：报告那一行应当写出登记成轻的自证底下过线不报的进程数」）
      「✗ 自检：内存包装里的进程应当照包装上限判」
      「✗ 自检：峰值表里有这条命令的进程应当照峰值判」
      「✗ 自检：看门狗的根进程自己也在同一个包装 scope 里时不该照包装上限判（退回 --process-rss-gibibytes）」
  AGENT_WATCH_BREAK=memoryselftest  memory_report() 不认登记成轻的自证，它底下的过线进程照报（改前的判法）
      selftest_memory（「✗ 自检：登记成轻的自证底下的过线进程不该报」、「✗ 自检：报告那一行应当写出登记成轻的自证底下过线不报的进程数」）
  AGENT_WATCH_BREAK=memoryselftestanyweight  memory_selftest_owner() 上层参数里带 --selftest 的一律当登记成轻的自证，不看登记表
      selftest_memory（「✗ 自检：不在登记成轻的自证底下的过线进程应当照报」）
  AGENT_WATCH_BREAK=memoryunit  memory_report() 的线按 MiB 算（--process-rss-gibibytes 乘 1024²）
      selftest_in（「✗ 自检：--processes-only 时写着的文件一小时没动、CPU 也不涨的进程」、「✗ 自检：--processes-only 时一直在写文件的进程」）、
      selftest_heavy_tests（「✗ 自检：「重型测试没带前缀」的 5 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒」）、selftest_memory（「✗ 自检：「进程常驻内存过线」应当只报写满 256 MiB 的进程」、「✗ 自检：报告里应当有一行」、
      「✗ 自检：过线进程 --ack 进程:」、「✗ 自检：看门狗 --processes-only 起了之后才冒出来的过线进程」、「✗ 自检：看门狗 --agents 起了之后才冒出来的过线进程」、
      「✗ 自检：峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报」（下限取 --process-rss-gibibytes 那一头、按 MiB 算成 8 MiB，只睡的 python 常驻过 8 MiB 时红））
  AGENT_WATCH_BREAK=midwaymerged  agent_alerts() 把没起过后台任务、结束本轮没交回的照改前报成「结束本轮却不会醒」，run() 不出「中途停下没交回的子 agent」一节
      selftest_in（「✗ 自检：子 agent queuedresume」、「✗ 自检：子 agent inquiryfailed」、「✗ 自检：子 agent midway：应当告警 ['中途停下没交回']」、『✗ 自检：报告里应当另起一节『中途停下没交回的子 agent』』）
      「✗ 自检：报告里应当另起一节「中途停下没交回的子 agent：被看的 3 个里 2 个」」
      「✗ 自检：子 agent textthenended」
  AGENT_WATCH_BREAK=movedbackground  background_start_in_result() 不认「… was moved to the background」
      selftest_in（「✗ 自检：子 agent movedbackground」、「✗ 自检：前台命令超时被挪进后台」）
  AGENT_WATCH_BREAK=nosingleinstance  acquire_single_instance() 不拦同一个会话的第二个看门狗
      selftest_session_watch_in（「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」）
  AGENT_WATCH_BREAK=nowake  agent_alerts() 不报「结束本轮却不会醒」
      selftest_in（「✗ 自检：子 agent nowake」、「✗ 自检：子 agent orphantask」、「✗ 自检：子 agent queuedresume」、「✗ 自检：子 agent inquiryfailed」）
      「✗ 自检：子 agent midway：应当告警 ['中途停下没交回']」
      「✗ 自检：报告里应当另起一节「中途停下没交回的子 agent：被看的 3 个里 2 个」」
      「✗ 自检：子 agent textthenended」
  AGENT_WATCH_BREAK=orphan  AgentTranscript.orphaned_background_tasks() 一律交空
      selftest_in（「✗ 自检：子 agent orphantask」）
  AGENT_WATCH_BREAK=otheragent  read_detections() 不按被看的子 agent 筛检出
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner 别的子agent」」）
  AGENT_WATCH_BREAK=outcomeignored  detection_outcome() 不看检出记录的结局，只认「写被拒」，执行前已被拒的命令也当成「只记不拦，命令照常在跑」而叫醒
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」、检出「重型测试被拒：…」」、「✗ 自检：检出记录属于「session」「主 agent None」、检出「提交时才跑的检查被拒：…」」）、
      selftest_session_watch_in（「✗ 自检：hook 检出：执行前已被 hook 拒了的（b1 …）应当只记进报告、不叫醒」）
  AGENT_WATCH_BREAK=ownrefusal  read_detections() 主 agent 自己的命令被 hook 拒的不单记
      selftest_in（「✗ 自检：检出记录属于「session」「主 agent None」」）
  AGENT_WATCH_BREAK=pipeancestor  stdin_pipe_writers() 不往上记读端自己的祖先（自己那一支也算成上游写端）
      selftest_pipe_upstream（「✗ 自检：stdin 管道的写端只在自己的祖先」）
  AGENT_WATCH_BREAK=pipeupstream  pipe_upstream_activity() 不看管道上游的写端
      selftest_pipe_upstream（「✗ 自检：stdin 是管道、上游」、「✗ 自检：上游」）
  AGENT_WATCH_BREAK=pipeupstreamalways  pipe_upstream_activity() 恒报上游在动
      selftest_in（「✗ 自检：写着的文件一小时没动、CPU 也不涨的进程」、「✗ 自检：--processes-only 时写着的文件一小时没动、CPU 也不涨的进程」）、selftest_pipe_upstream（「✗ 自检：上游」、「✗ 自检：stdin 是管道、上游」、
      「✗ 自检：stdin 管道的写端只在自己的祖先」）、
      selftest_reminder_timer（「✗ 自检：stop 那个假脚本底下的 sleep …同形却不是 remind.sh start，应当照报「进程无输出」」、「✗ 自检：other 那个假脚本底下的 sleep …同形却不是 remind.sh start，应当照报「进程无输出」」）、
      selftest_unacknowledged_terminations（「✗ 自检：看门狗真跑三次：--processes-only 连起三次都报「进程无输出」（退 3）」）
  AGENT_WATCH_BREAK=processexit  run_processes_only() 盯的进程都退出了也不退
      selftest_in（「✗ 自检：--processes-only 盯着的进程退出了」、「✗ 自检：--processes-only 起的时候一个要盯的进程都没有」）
  AGENT_WATCH_BREAK=progressanypath  progress_file_of() 照改前的取法：派发提示里点名的工作根下的路径都算候选、取改得最晚的那一份，不只认行首「草稿目录：」那一行
      selftest_scratch_mentions（「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告…」、「✗ 自检：派发提示没有行首「草稿目录：」一行、只点名别的子 agent 的报告…」、
      「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句应当写取不到 progress.md…」）
  AGENT_WATCH_BREAK=progressnote  progress_note() 告警里不附 progress.md 的末几行
      selftest_session_watch_in（「✗ 自检：整点询问应当附上 progress.md 的末几行」）
      「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句」
  AGENT_WATCH_BREAK=progressstale  agent_alerts() 不报「进度文件不涨」
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」）
  AGENT_WATCH_BREAK=refusedwakes  read_detections() 钩子执行前已拒掉的检出（outcome 被拒）也叫醒，不单记进报告
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」、检出「重型测试被拒：…」」、「✗ 自检：检出记录属于「session」「主 agent None」、检出「提交时才跑的检查被拒：…」」）、
      selftest_session_watch_in（「✗ 自检：hook 检出：执行前已被 hook 拒了的（b1 …）应当只记进报告、不叫醒」）
  AGENT_WATCH_BREAK=remindertimer  reminder_timer_note() 一律交 None：remind.sh start 等到点的 sleep 也当普通进程判
      selftest_reminder_timer（「✗ 自检：remind.sh start 底下等到点的 sleep」）
  AGENT_WATCH_BREAK=repeat  agent_alerts() 不报同一命令反复且输出不变
      selftest_in（「✗ 自检：子 agent repeat」）
  AGENT_WATCH_BREAK=repeatsforgotten  read_repeated_reports() 不读状态目录里的连报记录，每一次都从 1 数
      selftest_unacknowledged_terminations（「✗ 自检：连报到阈值：同一件「进程无输出」连报 3 次没 --ack，应当在第 3 次经 proc.py stop 终止 …」、
      「✗ 自检：连报到阈值：第 2 次叫醒之后状态文件里 …」、「✗ 自检：子 agent 的工具调用过长连报 3 次：…」、「✗ 自检：两件同时到线：…」、「✗ 自检：看门狗真跑三次 …」）
  AGENT_WATCH_BREAK=restarthint  restart_command() 与 run_processes_only() 的出路句印回本脚本自己的选项，不印 watch.sh 认得的写法
      selftest_in（「✗ 自检：定时回报的出路句应当是「→ 接着盯就用 run_in_background 再起一个看门狗：bash scripts/agents/watch.sh waiting」」）、
      selftest_session_watch_in（「✗ 自检：--discover 的出路句应当是不带 agent 号的 bash scripts/agents/watch.sh」）
  AGENT_WATCH_BREAK=resume  AgentTranscript._read() 交回之后的续做消息不撤销交回
      selftest_in（「✗ 自检：子 agent queuedresume」、「✗ 自检：子 agent continued 应当不算交回或被停」、「✗ 自检：子 agent resumedthinking 应当不算交回或被停」、
      「✗ 自检：子 agent queuedresume 应当不算交回或被停」）
      「✗ 自检：报告里应当另起一节「中途停下没交回的子 agent：被看的 3 个里 2 个」」
  AGENT_WATCH_BREAK=resumeturn  AgentTranscript._read() 结束本轮之后来的续做消息不算被叫醒
      selftest_in（「✗ 自检：子 agent resumedafterturnend」、「✗ 自检：子 agent resumedafterturnend 的状态应当是「思考中」」）
  AGENT_WATCH_BREAK=rootfree  root_free_report() 不看余量（改前的样子），根分区可用量低于 --root-free-gib-alert 也不报
      selftest_root_free（「✗ 自检：根分区可用 40 GiB、阈值 128 GiB 时应当报」、「✗ 自检：根分区可用量应当只读一次」、「✗ 自检：根分区可用 500 GiB、阈值 128 GiB 时不该报」、
      「✗ 自检：看门狗 report 在根分区可用 40 GiB 时」、「✗ 自检：根分区余量过低被 --ack 本机:根分区余量过低 确认之后」、「✗ 自检：看门狗 report 在根分区可用 500 GiB 时」、
      「✗ 自检：--processes-only 的 report 在根分区可用 40 GiB 时」）
  AGENT_WATCH_BREAK=scopeunlisted  read_detections() 不记范围声明钩子的放行记录，定时回报里没有「范围声明」一节
      selftest_session_watch_in（「✗ 自检：范围声明：这一会话放行的两条（c1、主 agent）应当列进「范围声明」一节 …」）
  AGENT_WATCH_BREAK=scratchhog  scratch_hog_report() 一条都不报
      selftest_scratch_hogs（「✗ 自检：草稿根里超阈值的编译目录与仓副本：应当只报 …」、「✗ 自检：草稿根那一节的第一行应当是汇总」、「✗ 自检：草稿根那一节应当按最后修改时刻、最久没动的在前」、
      「✗ 自检：有子 agent 交回时看门狗的报告应当列出超阈值的 …」）
  AGENT_WATCH_BREAK=scratchhogage  scratch_hog_report() 不看最后修改的时刻，不到阈值的编译目录与仓副本也报
      selftest_scratch_hogs（「✗ 自检：草稿根里超阈值的编译目录与仓副本：应当只报 …」：fresh-build、fresh-copy 也报了、「✗ 自检：有子 agent 交回时看门狗的报告应当列出超阈值的 …、不列 …」）
      「✗ 自检：草稿根那一节应当按最后修改时刻、最久没动的在前」
  AGENT_WATCH_BREAK=scratchhogtag  scratch_hog_kind() 不认 CACHEDIR.TAG，编译目录一概认不出
      selftest_scratch_hogs（「✗ 自检：草稿根里超阈值的编译目录与仓副本：应当只报 …」：old-build/target 没报、「✗ 自检：有子 agent 交回时看门狗的报告应当列出超阈值的 …」）
      「✗ 自检：草稿根那一节应当按最后修改时刻、最久没动的在前」
  AGENT_WATCH_BREAK=settle  AgentTranscript._read() 最后一条是纯文字、stop_reason 为空的一概不判结束本轮（主会话记录里有这一轮的任务通知、主会话记录不在而过了 TEXT_SETTLE_SECONDS 都不认）
      selftest_in（「✗ 自检：子 agent textsettled」、「✗ 自检：子 agent inquiryfailed」、「✗ 自检：子 agent textsettled 的状态应当是「本轮结束、没交回」」、
      「✗ 自检：子 agent textthenended」、「✗ 自检：子 agent textthenended 的状态应当是「本轮结束、没交回」」、「✗ 自检：子 agent noparenttext 的状态应当是「本轮结束、没交回」」）
  AGENT_WATCH_BREAK=settlenotification  AgentTranscript.text_turn_ended() 不看主会话记录里的任务通知，纯文字记录过了 TEXT_SETTLE_SECONDS 就判结束本轮（改前的判法）
      selftest_in（「✗ 自检：子 agent textthenwrite」、「✗ 自检：子 agent textthenwrite 的状态应当是「思考中」」）
  AGENT_WATCH_BREAK=stale  process_alerts() 不报「进程无输出」
      selftest_in（「✗ 自检：写着的文件一小时没动、CPU 也不涨的进程」、「✗ 自检：CPU 在涨的进程」、「✗ 自检：--processes-only 时写着的文件一小时没动、CPU 也不涨的进程」）、selftest_pipe_upstream（「✗ 自检：上游」、
      「✗ 自检：stdin 是管道、上游」、「✗ 自检：stdin 管道的写端只在自己的祖先」）、
      selftest_reminder_timer（「✗ 自检：stop 那个假脚本底下的 sleep …同形却不是 remind.sh start，应当照报「进程无输出」」、「✗ 自检：other 那个假脚本底下的 sleep …同形却不是 remind.sh start，应当照报「进程无输出」」）、
      selftest_unacknowledged_terminations（「✗ 自检：看门狗真跑三次：--processes-only 连起三次都报「进程无输出」（退 3）」）
  AGENT_WATCH_BREAK=stalepending  AgentTranscript._read() 不剔掉只落了发起、等不到结果的工具调用
      selftest_in（「✗ 自检：子 agent stalependingthenresult：应当告警 无，实际 ['工具调用过长']」）
  AGENT_WATCH_BREAK=stalesession  stale_session_reason() 重启前旧进程留下的记录一概不判已失效
      selftest_stale_sessions（「✗ 自检：重启前旧进程留下的记录（已不在的会话 deadlong …」、「✗ 自检：子 agent deadlong 应当在报告里标成「状态=已失效」」、「✗ 自检：被看的只有重启前旧进程留下的记录时应当没有告警」）
  AGENT_WATCH_BREAK=stalesessionrestart  stale_session_reason() 不判第二种（会话号还活着、最后一条记录早于进程启动）
      selftest_stale_sessions（「✗ 自检：重启前旧进程留下的记录（… beforestart …」、「✗ 自检：子 agent beforestart 应当在报告里标成「状态=已失效」」、「✗ 自检：被看的只有重启前旧进程留下的记录时应当没有告警」）
  AGENT_WATCH_BREAK=taskstop  AgentTranscript.__init__() 不认主会话记录里 TaskStop 的成功结果
      selftest_in（「✗ 自检：子 agent stoppedidle」、「✗ 自检：子 agent stoppedtwice」、「✗ 自检：子 agent stoppedidle 应当算交回或被停」、「✗ 自检：子 agent stoppedtwice 应当算交回或被停」、
      「✗ 自检：被看的子 agent 都交回或被停时看门狗应当退出码 0」）、selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个」）
  AGENT_WATCH_BREAK=terminateacked  terminate_unacknowledged_repeats() 把 --ack 过的告警也算进连报
      selftest_unacknowledged_terminations（「✗ 自检：有 --ack：…」）
  AGENT_WATCH_BREAK=terminatezero  terminate_unacknowledged_repeats() 不认阈值 0 是关掉，照 3 次算
      selftest_unacknowledged_terminations（「✗ 自检：阈值 0：…」）
  AGENT_WATCH_BREAK=timeout  agent_alerts() 不报「工具调用过长」
      selftest_in（「✗ 自检：子 agent longtool」、「✗ 自检：子 agent inquirysentafter」、「✗ 自检：有告警（工具调用过长）时看门狗应当退出码 3」、「✗ 自检：「工具调用过长」被 --ack 确认之后看门狗应当到点退出码 4 并写「已确认接着盯」」）、
      selftest_session_watch_in（「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、「✗ 自检：条件类告警的确认」）、
      selftest_stale_sessions（「✗ 自检：重启前旧进程留下的记录」、「✗ 自检：登记目录读不了时判不了会话在不在」）
  AGENT_WATCH_BREAK=toolrevoke  AgentTranscript._read() 交回之后又调了工具就撤销交回
      selftest_in（「✗ 自检：子 agent toolafterhandback」、「✗ 自检：子 agent toolafterhandback 应当算交回或被停」）
  AGENT_WATCH_BREAK=verdictlines  process_alerts() 不数跑得久的进程写着的文件里的判定行
      selftest_verdict_lines（「✗ 自检：判红的日志应当报「长跑里有判红的行」」、「✗ 自检：报告里应当有判定行那一行」）、
      selftest_verdict_selftest_scratch（「✗ 自检：登记成轻的自证在自己临时目录里写的 ✗ 不该报」、「✗ 自检：共用 TMPDIR 的自证直接写在 TMPDIR 底下的 ✗ 应当报」、「✗ 自检：没有自证上层的进程写在像 mktemp 的临时目录里的 ✗ 应当报」）
  AGENT_WATCH_BREAK=verdictselftest  light_selftest_scratch_owner() 一律交 None：登记成轻的自证临时目录里的 ✗ 照报（改前的判法）
      selftest_verdict_selftest_scratch（「✗ 自检：登记成轻的自证在自己临时目录里写的 ✗ 不该报」）
  AGENT_WATCH_BREAK=verdictselftestwrapperseen  private_temporary_base() 不跳过只是包着它的那几层，拿父进程（timeout）当起它的那一层
      selftest_verdict_selftest_scratch（「✗ 自检：登记成轻的自证在自己临时目录里写的 ✗ 不该报」）
  AGENT_WATCH_BREAK=verdictselftestanyfile  light_selftest_scratch_owner() 登记成轻的自证写的文件不看在不在临时目录里一律交说明
      selftest_verdict_selftest_scratch（「✗ 自检：共用 TMPDIR 的自证直接写在 TMPDIR 底下的 ✗ 应当报」）
  AGENT_WATCH_BREAK=verdictselftestpathonly  light_selftest_scratch_owner() 路径里有一层像 mktemp 临时目录就交说明，不看是不是自证写的
      selftest_verdict_selftest_scratch（「✗ 自检：没有自证上层的进程写在像 mktemp 的临时目录里的 ✗ 应当报」、「✗ 自检：登记成轻的自证在自己临时目录里写的 ✗ 不该报」）
  AGENT_WATCH_BREAK=verdictselftestsharedtmpdir  private_temporary_base() 一律交 True：与上层共用的 TMPDIR 也整个算自证的临时目录
      selftest_verdict_selftest_scratch（「✗ 自检：共用 TMPDIR 的自证直接写在 TMPDIR 底下的 ✗ 应当报」）
  AGENT_WATCH_BREAK=verdictselftestprivateignored  private_temporary_base() 一律交 False：单给自证的 TMPDIR 不认，只认下一层像 mktemp 的
      selftest_verdict_selftest_scratch（「✗ 自检：登记成轻的自证在自己临时目录里写的 ✗ 不该报」）
  AGENT_WATCH_BREAK=zombie  still_running() 把僵尸当还在跑
      selftest_in（「✗ 自检：--processes-only 盯着的进程退出了」）
工作根（草稿目录、检出记录与状态目录的默认位置）与过渡期旧根只在 .claude/hooks/lib_work_root.py 里定义，它的两个弄坏开关：
  SINGLEFS_WORK_ROOT_BREAK=new-root-unknown  工作根退回旧根：派发提示里工作根下的草稿目录认不出
      selftest_scratch_mentions（「✗ 自检：派发提示点名工作根下的草稿目录」两格、「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告…」）、
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」、「✗ 自检：整点询问应当附上 progress.md 的末几行」）
  SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown  不认过渡期旧根：派发提示里旧根下的草稿目录认不出
      selftest_scratch_mentions（「✗ 自检：派发提示点名过渡期旧根下的草稿目录」）
「交回」送没送达只在同目录 subagent_transcript.py 里认，它的开关（开关表在它的文件头）：
  SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered  被交回闸拒的交回也算送达
      selftest_in（「✗ 自检：子 agent refusedonly 应当不算交回或被停」）
"""
import argparse
import concurrent.futures
import glob
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime, timezone
from zoneinfo import ZoneInfo
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'hooks'))
import lib_work_root as work_roots  # noqa: E402  工作根与过渡期旧根的唯一定义
import lib_prompt_scratch_line as prompt_scratch_line  # noqa: E402  派发提示里「本提示的草稿目录」那一行的唯一认法，派发闸、交回闸、完工清理闸调的是同一份
preflight_sys.path.insert(0, preflight_os.path.dirname(preflight_os.path.realpath(__file__)))
import subagent_transcript as transcripts  # noqa: E402  「交回」（送达）的唯一定义，续做闸与 agent-cost.py 取的是同一份

PROJECTS_ROOT = os.environ.get("AGENT_WATCH_PROJECTS_ROOT") or os.path.expanduser("~/.claude/projects")   # 环境变量只给 --selftest 造假的项目目录用
DEFAULT_DETECTIONS = work_roots.default_detections_path()
WAIT_LOOP_PATTERN = re.compile(r"\b(until|while)\b[\s\S]*\bsleep\b")
BANNED_COMMAND_PATTERN = re.compile(r"\bpgrep\s+-f\b|\bpkill\s+-f\b|\bkillall\b")
BROKEN_DETECTION = os.environ.get("AGENT_WATCH_BREAK", "")
# argparse 的 help 文本要过一遍 % 格式化：文字里的百分号写成 %%。弄坏开关 AGENT_WATCH_BREAK=helpformat 退回没转义的写法（--help 抛 ValueError）
HELP_PERCENT = "%" if BROKEN_DETECTION == "helpformat" else "%%"
# 禁用命令只报这个窗口里发出的：看门狗每次起都从头读会话记录，窗外的旧命令会让每一次重起当场退出、再也盯不住那个 agent。
BANNED_COMMAND_WINDOW_SECONDS = 600
HEREDOC_START_PATTERN = re.compile(r"<<-?\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")


def command_without_heredoc_bodies(command):
    """去掉 heredoc 的正文，只留真会执行的那几行：往报告里写的一句「pgrep -f 会命中自己」不是在用它。"""
    if BROKEN_DETECTION == "heredoc":
        return command
    kept, terminators = [], []
    for line in command.split("\n"):
        if terminators:
            if line.strip() == terminators[0]:
                terminators.pop(0)
            continue
        kept.append(line)
        terminators.extend(match.group(2) for match in HEREDOC_START_PATTERN.finditer(line))
    return "\n".join(kept)
STOP_AFTER_LAST_RECORD_SECONDS = 30  # 停在工具中途时，被停之后还可能落几条收尾记录
NO_WAKE_GRACE_SECONDS = 60           # 结束本轮之后，完成通知可能还在路上
TURN_END_STOP_REASONS = ("end_turn", "stop_sequence")   # 会话记录里结束本轮的两种写法（2026-09-19 实测两种都有）
TEXT_SETTLE_SECONDS = 120            # 最后一条是纯文字、stop_reason 为空而主会话记录不在时：过了这么久没接下文就算结束本轮（AgentTranscript.text_turn_ended）
CPU_SAMPLE_SECONDS = 2               # 写着的文件不动的进程，隔这么久再取一次 CPU 时间，分「在算」与「不动」
BACKGROUND_STARTED = re.compile(r"^Command running in background with ID: (\w+)")
# 前台命令跑满超时被 harness 挪进后台，工具结果是另一种写法，之后同样会来完成通知。不认它，在等这种任务的子 agent 会被报「结束本轮却不会醒」
# （2026-09-23 门禁审计会话的 ac26abd494cda0968：doc-lint 超时转后台，看门狗报「起过的 0 个后台任务」；本机这个项目的子 agent 会话记录里 79 份有这种写法）。
BACKGROUND_MOVED = re.compile(r"^Command did not complete within its \d+s timeout and was moved to the background \(ID: (\w+)\)")
NEW_INSTRUCTION_ORIGINS = ("coordinator", "peer")   # 续做消息与别的会话发来的消息；task-notification 是自己后台任务的完成通知，不算
BACKGROUND_OUTPUT_PATH = re.compile(r"Output is being written to: (\S+?\.output)")
TASK_NOTIFICATION_ID = re.compile(r"<task-id>(\w+)</task-id>")
# 进程一节不看的命令：claude 本体、IDE、node（常驻的 MCP 一类）、别的看门狗。进程告警与 --processes-only 盯哪些进程读的是这同一份。
COMMAND_MARKERS_NOT_WATCHED = ("native-binary/claude", "vscode-server", "/node ", "agent-watch.py")
EXITED_PROCESS_STATES = ("Z", "X")   # /proc/<pid>/stat 的状态：僵尸（退出了、等父进程收尸）与已死，都算退出了
LEFTOVER_BACKGROUND_ALERT = "交回之后后台还在跑"
KEPT_COPY_ALERT = "交回之后副本还在"
# 交回报告里「没删 <全路径>」那一行的写法（上游交回闸 handback-scratch-check.sh 要的就是这一行）：路径取到第一个不是路径字符的字为止（后面紧跟汉字也认）
KEPT_COPY_MENTION = re.compile(r"没删(?:掉)?[：:\s]*[`「“\"']?((?:~/|/)[A-Za-z0-9_./~+@%=,-]+)")
# 交回报告里点名的报告文件（.md，绝对路径或 ~/ 起头）：里面写的「没删 <全路径>」也算
KEPT_COPY_REPORT_MENTION = re.compile(r"((?:~/|/)[A-Za-z0-9_./~+@%=,-]+\.md)")
KEPT_COPY_REPORT_MAX_BYTES = 4 << 20
# 主会话记录里任务通知的这几种状态说明子 agent 已经不在跑了：failed 撞限额或报错、killed 被停、stopped 上一个 Claude 进程退出时它还没跑完
ENDED_NOTIFICATION_STATUSES = ("failed", "killed", "stopped")
TASK_NOTIFICATION_BLOCK = re.compile(r"<task-notification>(.*?)</task-notification>", re.S)
TASK_NOTIFICATION_STATUS = re.compile(r"<status>([a-z_]+)</status>")
# 重启前旧进程留下的子 agent 记录判「已失效」：Claude Code 给每个进程在这个目录登记一份 <pid>.json（会话号 sessionId、启动时刻 startedAt 毫秒、
# procStart 即 /proc/<pid>/stat 第 22 列）。环境变量只给 --selftest 造假的登记目录用
SESSION_REGISTRY_ENV = "AGENT_WATCH_SESSIONS_DIR"
PROC_STAT_START_FIELD_AFTER_COMM = 19   # /proc/<pid>/stat 里右括号之后的字段从第 3 列起数，第 22 列是下标 19
# 草稿根里超阈值的编译目录与仓副本：编译目录认 CACHEDIR.TAG（签名见 https://bford.info/cachedir/，cargo 的 target 都带），仓副本认 .git 或「Cargo.toml 加 crates/」。
# 环境变量（冒号分隔的根）只给 --selftest 造假的草稿根用
SCRATCH_ROOTS_ENV = "AGENT_WATCH_SCRATCH_ROOTS"
CACHEDIR_TAG_SIGNATURE = b"Signature: 8a477f597d28d172789f06886806bc55"
SCRATCH_SCAN_MAX_DEPTH = 4            # 从草稿根往下找这么深（根 → 任务目录 → 副本 → target）
SCRATCH_SCAN_MAX_DIRECTORIES = 20000  # 一次最多看这么多个目录，到顶就在报告里写明
SCRATCH_MTIME_MAX_DEPTH = 3           # 求「最后修改时刻」往副本里看这么深（target/release/deps 在内）
SCRATCH_MTIME_MAX_ENTRIES = 30000     # 每处副本最多 stat 这么多个条目
ROOT_FREE_ALERT = "根分区余量过低"
ROOT_FREE_SUBJECT = "本机"            # 这一类告警不属于哪个子 agent 或进程：--ack 本机:根分区余量过低
ROOT_FREE_BYTES_VARIABLE = "AGENT_WATCH_ROOT_FREE_BYTES"   # 环境变量只给 --selftest 起的看门狗子进程用：设了就当根分区的可用字节数，不读 statvfs
ROOT_FREE_GIB_ALERT_DEFAULT = 128     # --root-free-gib-alert 的默认值（依据见那个选项，推的）
# 会话记录按字节扫后台输出文件路径：几百兆的会话记录不逐行解 JSON，先看它提没提到有进程开着的那几个文件
BACKGROUND_OUTPUT_PATH_BYTES = re.compile(rb"Output is being written to: (\S+?\.output)")
PROCESS_STOP_COMMAND = "python3 .claude/singlefs-ai-sop/scripts/proc.py stop"
# 「跑满 N 小时要主 agent 问一次」：告警名是「跑满 <已跨过的那一格> 要主 agent 问一次」，--ack 写「<agent 号>:跑满 <那一格>」
ROUTINE_INQUIRY_ALERT_PREFIX = "跑满 "
ROUTINE_INQUIRY_ALERT_SUFFIX = "要主 agent 问一次"
ROUTINE_INQUIRY_NEXT_STEP = ("主 agent 给它发一条例行询问，并写明「把回答写进它草稿目录的 progress.md」（在做什么、还差几步、在等哪个进程、预计多久；"
                             "等的已经结束就接着做或交回），发完再起看门狗")
# 主会话记录里 SendMessage 发给子 agent 的成功结果的两种写法（2026-09-24 查本机这个项目的主会话记录：157 条排队、另有续做的「Resuming agent」），
# 都带 pin，续做的另带 resumedAgentId；发给别的会话的写法不同，不认
MAIN_AGENT_MESSAGE_QUEUED = "Message queued for delivery to "
MAIN_AGENT_MESSAGE_RESUMED = "resumedAgentId"
# 「重型测试没带前缀」：判定与执行前的闸 heavy-test-guard.sh 同一份，从 scripts/ 往上两级是仓根
HEAVY_TESTS_LIBRARY_PATH = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                                        ".claude", "hooks", "lib_heavy_tests.py")
# 自证的临时目录由同一处的共用库建（通过、判红、抛异常三条路上都删），那三格也由它判
SELFTEST_SCRATCH_LIBRARY_PATH = os.path.join(os.path.dirname(HEAVY_TESTS_LIBRARY_PATH), "lib_selftest_scratch.py")
SELFTEST_ROLE_VARIABLE = "AGENT_WATCH_SELFTEST_ROLE"   # 自证拉起自己的子进程走判红、抛异常那两条路时设的角色
SELFTEST_READY_VARIABLE = "AGENT_WATCH_SELFTEST_READY_FILE"   # 自证拉起的看门狗第一次进两次整查之间的等待前建这个文件（mark_selftest_ready），自证按它等
SELFTEST_READY_TIMEOUT_SECONDS = 120   # 自证按条件等子进程起来的上限；到点不报错，照原判据接着判
SELFTEST_FAKE_PROCESS_SECONDS = 900    # 「重型测试没带前缀」那一格的假进程睡多久：活到这一格判完、由 finally 停掉；自证被 SIGKILL 打断时留下的到点自己退
HEAVY_TEST_VARIABLE = "SINGLEFS_HEAVY_TESTS"
HEAVY_TEST_OCCASIONS = ("commit", "user-request")
HEAVY_TEST_ALERT = "重型测试没带前缀"
HEAVY_TEST_STOP_ORDER_SHOWN = 24   # 下一步里列出的停止次序最多这么多个进程号，再多写「共 N 个」
# 「进程常驻内存过线」
MEMORY_ALERT = "进程常驻内存过线"
PAGE_SIZE_BYTES = os.sysconf("SC_PAGE_SIZE")
BYTES_PER_GIBIBYTE = 1024 ** 3
# 「长跑在用核数过少」：取样与判法在 scripts/hosts/core_usage.py（multi-host-run.sh --peer、detached-run.sh watch 同一份），按文件路径导入
CORE_USAGE_ALERT = "长跑在用核数过少"
# 「本机内存排队、别的机器放得下」：scripts/hosts/run-with-memory-cap.sh 排队时写进同一份告警记录的那一种（它文件头「排队时去找别的机器」）
MEMORY_QUEUE_ALERT = "本机内存排队、别的机器放得下"
MEMORY_QUEUE_RECORD_KIND = "memory-queue-peer-fits"
# run-with-memory-cap.sh 收尾时写进同一份告警记录的三种（它文件头「峰值表」一段）：种类 → 告警名
MEMORY_PEAK_GREW_ALERT = "内存峰值比记账大"
MEMORY_OWN_CAP_ALERT = "撞了自己的上限"
MEMORY_SQUEEZED_ALERT = "被总上限挤掉"
MEMORY_OUTCOME_ALERTS = {"memory-peak-grew": MEMORY_PEAK_GREW_ALERT, "memory-own-cap": MEMORY_OWN_CAP_ALERT, "memory-squeezed": MEMORY_SQUEEZED_ALERT}
MEMORY_PEAK_ALERT_CAP_PERCENT_DEFAULT = 50   # --memory-peak-alert-cap-percent 的默认值（判法见文件头「长跑在用核数过少」二）
# 阈值逐个进程取（文件头「进程常驻内存过线」①②③）：内存包装的 scope 名、包装上限的百分比默认值、峰值表的库（按文件路径导入）
MEMORY_WRAPPER_SCOPE = re.compile(r"/(singlefs-memory-cap-[^/]+)\.scope(?=/|$)")
MEMORY_WRAPPER_CAP_PERCENT_DEFAULT = 90
MEMORY_PEAK_FLOOR_MEBIBYTES_DEFAULT = 512   # --process-rss-floor-mebibytes 的默认值（依据见那个选项）：峰值表定的阈值不低于它
BYTES_PER_MEBIBYTE = 1024 ** 2
CGROUP_ROOT = "/sys/fs/cgroup"
MEMORY_PEAKS_LIBRARY_PATH = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                                         "scripts", "hosts", "memory_peaks_lib.py")
# 「长跑里有判红的行」（文件头同名一段）：判定行的认法、每份文件至多数末尾多少字节、✗ 行至多列几行
VERDICT_RED_ALERT = "长跑里有判红的行"
VERDICT_LINE = re.compile(r"^\s*([✓✗])\s")
VERDICT_SCAN_BYTES = 32 * 1024 * 1024
VERDICT_RED_SHOWN = 5
# 「中途停下没交回」（文件头同名一段）：结束本轮、没交回、一个后台任务都没起过的
STOPPED_MIDWAY_ALERT = "中途停下没交回"
MEMORY_CAP_TEXT = re.compile(r"^([0-9]+)([KMGT])$")   # 内存包装的上限写法，读法同 scripts/hosts/memory_peaks_lib.py 的 size_bytes
MEMORY_CAP_POWERS = {"K": 1, "M": 2, "G": 3, "T": 4}
CORE_USAGE_LIBRARY_PATH = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                                       "scripts", "hosts", "core_usage.py")
CORE_USAGE_SAMPLES_DIRECTORY = "core-usage-samples"   # 状态目录下：每个重活进程一份样本序列，<进程号>-<起始时刻>.jsonl
CORE_USAGE_SAMPLES_KEPT_SECONDS = 2 * 86400           # 两天没再写的样本序列删掉（进程早没了）
CORE_USAGE_SAMPLES_ROOT_VARIABLE = "AGENT_WATCH_CORE_SAMPLES_ROOT"   # 环境变量只给 --selftest 用：它起的看门狗子进程把样本写进自证的临时目录，不写真的状态目录
CORE_USAGE_SHOWN = 12                                 # 报告里逐个列在用核数的重活进程至多这么多个，再多写「另 N 个」
# 并行在几个干活进程之间、每个干活进程里的单行 cargo test 本来只占一条线的门禁：这道的进程整棵树当一个重活、按那一趟的合计判核数，不往下拆到单行
# （文件头「长跑在用核数过少」一的末段）。带 --quick 的轻型写法不算 classify_process 认的重活，不在这里点名就会往下拆
AGGREGATED_PARALLEL_STAGES = frozenset({"checker-tier-crates-mutation-replay.sh"})
# 同一类的研究脚本（相对仓根的路径，按命令词的路径尾部认）：合入后验证测试那一步按测试目标并行起的那一份，各目标各起一条 cargo test、RUST_TEST_THREADS 分摊
AGGREGATED_PARALLEL_SCRIPTS = frozenset({"scripts/gate/test-targets-parallel.py"})
# 「该交接」：只对这两类报、到 --context-tokens（默认 70 万，推的）报一次；别的类型上下文多大都不叫醒（主 agent 的规矩是上下文大小不是停它的理由）
HANDOVER_AGENT_TYPES = {"implementation-writer", "experiment-runner"}
HANDOVER_ALERT = "该交接"
HANDOVER_TOP_OUTPUTS_SHOWN = 3
# 「进度文件不涨」：结束本轮在等自己后台任务的子 agent，派发提示里草稿目录下的 progress.md 超过 --progress-stale-minutes 没改就报，不等整点
# （草稿目录只认派发提示行首「草稿目录：」那一行，认法在 .claude/hooks/lib_prompt_scratch_line.py；没有那一行的取不到，不到提示点名的别处去找）
PROGRESS_FILE_NAME = "progress.md"
PROGRESS_STALE_ALERT = "进度文件不涨"
PROGRESS_TAIL_LINES = 3
# 提示里点名的工作根下的路径：只给弄坏开关 progressanypath 照改前的取法用（平常只认行首「草稿目录：」那一行，progress_file_of）
SCRATCH_PATH_MENTION = re.compile(work_roots.mention_prefix_pattern() + r"[^\s`'\"，。；：、（）()「」<>]+")
# 一个会话一个看门狗（--discover）：锁、确认过的告警与见过没结束的子 agent 都放在这个会话的状态目录里
STATE_ROOT = os.environ.get("AGENT_WATCH_STATE_ROOT") or work_roots.state_directory("watch-state")
SINGLE_INSTANCE_EXIT = 5
# 调度方不响应就终止（文件头「调度方不响应就终止」一节）：「进程卡住」类的三种告警，连报记录与终止用的脚本
STUCK_ALERT_NAMES = ("工具调用过长", "进程无输出", HEAVY_TEST_ALERT)
REPEATED_REPORTS_FILE = "watch-repeats.tsv"
REPEATED_REPORTS_PROCESSES_FILE = "watch-repeats-processes.tsv"
PROCESS_STOP_SCRIPT = (os.environ.get("AGENT_WATCH_PROC_SCRIPT")   # 环境变量只给 --selftest 换成记录调用的桩
                       or os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                                       ".claude", "singlefs-ai-sop", "scripts", "proc.py"))
PROCESS_STOP_TIMEOUT_SECONDS = 60   # proc.py stop 默认宽限 10 秒，留足余量
# 钩子已经拒掉的写与判不了的脚本：只记进报告，不叫醒主 agent（2026-09-28 量到 8 天里看门狗转报的检出大半是这两类，主 agent 醒了也没事可做）
# 钩子 ⑭ 判不了的长活照常执行、记一条以它起头的检出（bash-command-detector.sh 的 LONG_RUN_UNDETERMINABLE_PREFIX）：只记不叫醒。
# 弄坏开关 longrunwakes 把它从只记不叫醒那一类里拿掉（改前的样子）
LONG_RUN_UNDETERMINABLE_PREFIX = "长活判不了"
INFO_DETECTION_PREFIXES = ("写被拒", "重型测试闸没判全") + (() if BROKEN_DETECTION == "longrunwakes" else (LONG_RUN_UNDETERMINABLE_PREFIX,))
# 只记进报告、不叫醒主 agent 的三类条件告警（2026-09-28 量到 8 天里看门狗按告警退出 533 次，这三类醒来多半只是重起一只）：
INFO_ALERT_NAMES = {"等待循环", "同一命令反复且输出不变", PROGRESS_STALE_ALERT}
EVENT_ALERT_NAMES = {"hook 检出"}   # 事件类告警：确认之后到那个子 agent 结束都不再叫醒；别的是条件类，条件消失就作废


def parse_timestamp(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00"))


def is_new_instruction(record):
    """会话记录里这一条是不是给子 agent 的新指令：交回之后来一条就撤销交回。
    认：续做消息（user 记录，origin 为 coordinator；别的会话发来的为 peer）；它忙时排进队的续做消息（queued_command 附件，附件自己带
    origin coordinator，没有对应的 user 记录）；不带 origin、不是 harness 自己插的一句 user 文字。
    不认：后台任务的完成通知（origin 为 task-notification，或不带 origin 的 queued_command 附件）、工具结果、harness 插的提示（isMeta：
    system-reminder、「Output token limit hit」「[handback-send-enforce]」这类）、打断、压缩摘要（isCompactSummary）。"""
    attachment = record.get("attachment")
    if isinstance(attachment, dict):
        return attachment.get("type") == "queued_command" and (attachment.get("origin") or {}).get("kind") in NEW_INSTRUCTION_ORIGINS
    origin_kind = (record.get("origin") or {}).get("kind")
    if origin_kind is not None:
        return origin_kind in NEW_INSTRUCTION_ORIGINS
    message = record.get("message")
    if record.get("isMeta") or record.get("isCompactSummary") or not isinstance(message, dict) or message.get("role") != "user":
        return False
    content = message.get("content")
    if isinstance(content, str):
        texts = [content]
    elif isinstance(content, list) and not any(isinstance(block, dict) and block.get("type") == "tool_result" for block in content):
        texts = [block.get("text", "") for block in content if isinstance(block, dict) and block.get("type") == "text"]
    else:
        return False
    return any(text.strip() for text in texts) and not any("[Request interrupted" in text for text in texts)


def parent_session_transcript(transcript_path):
    """…/<会话 id>/subagents/agent-<id>.jsonl → …/<会话 id>.jsonl（派它的那个会话的记录）。"""
    return os.path.dirname(os.path.dirname(transcript_path)) + ".jsonl"


SESSION_TASK_EVENTS_CACHE = {}   # 主会话记录路径 → ((大小, 修改时刻), 读出来的两张表)


def is_task_notification_record(record):
    """真正的任务通知：origin 为 task-notification 的消息、排队的附件、队列操作；主 agent 自己的命令与回复里写着的 <status> 不算。"""
    attachment = record.get("attachment")
    return ((record.get("origin") or {}).get("kind") == "task-notification" or record.get("type") == "queue-operation"
            or (isinstance(attachment, dict) and attachment.get("type") == "queued_command"))


def session_task_events(session_transcript_path):
    """主会话记录一遍读完，返回三张表：{子 agent id: TaskStop 对它成功结果的最晚时刻}、{任务 id: (最近一次任务通知的状态, 时刻)}、
    {子 agent id: 主 agent 用 SendMessage 发给它的最晚一次成功结果的时刻}。
    按文件大小与修改时刻缓存：主会话记录能有上百兆，逐个子 agent 各读一遍太慢。"""
    try:
        status = os.stat(session_transcript_path)
    except OSError:
        return {}, {}, {}
    key = (status.st_size, status.st_mtime_ns)
    cached = SESSION_TASK_EVENTS_CACHE.get(session_transcript_path)
    if cached is not None and cached[0] == key:
        return cached[1]
    stops, notifications, messages_sent = {}, {}, {}
    for line in open(session_transcript_path, encoding="utf-8", errors="replace"):
        mentions_stop = "Successfully stopped task" in line
        mentions_status = "<status>" in line
        mentions_message_sent = MAIN_AGENT_MESSAGE_QUEUED in line or MAIN_AGENT_MESSAGE_RESUMED in line
        if not (mentions_stop or mentions_status or mentions_message_sent):
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        if not record.get("timestamp"):
            continue
        timestamp = parse_timestamp(record["timestamp"])
        result = record.get("toolUseResult")
        if mentions_stop and isinstance(result, dict) and "Successfully stopped task" in str(result.get("message", "")):
            agent_id = result.get("task_id")
            stops[agent_id] = timestamp if agent_id not in stops else max(stops[agent_id], timestamp)
        if mentions_status and is_task_notification_record(record):
            # 一条通知可以列好几个任务 id、共用一个状态（上一个 Claude 进程退出时没跑完的那一批）
            for block in TASK_NOTIFICATION_BLOCK.findall(json.dumps(record, ensure_ascii=False)):
                found_status = TASK_NOTIFICATION_STATUS.search(block)
                if found_status:
                    for task_id in TASK_NOTIFICATION_ID.findall(block):
                        notifications[task_id] = (found_status.group(1), timestamp)
        if (mentions_message_sent and isinstance(result, dict) and result.get("success") is True
                and (str(result.get("message", "")).startswith(MAIN_AGENT_MESSAGE_QUEUED) or result.get(MAIN_AGENT_MESSAGE_RESUMED))):
            agent_id = result.get(MAIN_AGENT_MESSAGE_RESUMED) or (result.get("pin") or {}).get("id")
            if agent_id:
                messages_sent[agent_id] = timestamp if agent_id not in messages_sent else max(messages_sent[agent_id], timestamp)
    SESSION_TASK_EVENTS_CACHE[session_transcript_path] = (key, (stops, notifications, messages_sent))
    return stops, notifications, messages_sent


def task_stop_time(agent_id, session_transcript_path):
    """主会话记录里 TaskStop 停掉这个子 agent 的最晚一次成功结果的时刻；没有就 None。"""
    return session_task_events(session_transcript_path)[0].get(agent_id)


def format_duration(seconds):
    seconds = int(max(seconds, 0))
    if seconds >= 3600:
        return f"{seconds // 3600}h{(seconds % 3600) // 60:02d}m"
    if seconds >= 60:
        return f"{seconds // 60}m{seconds % 60:02d}s"
    return f"{seconds}s"


class AgentTranscript:
    """一个子 agent 会话记录的摘要：状态、最后一条命令、调用次数、上下文、缓存整份重写次数。"""

    def __init__(self, agent_id, transcript_path):
        self.agent_id = agent_id
        self.transcript_path = transcript_path
        self.agent_type = "?"
        self.description = ""
        meta_path = transcript_path[: -len(".jsonl")] + ".meta.json"
        if os.path.isfile(meta_path):
            try:
                meta = json.load(open(meta_path, encoding="utf-8"))
                self.agent_type = meta.get("agentType", "?")
                self.description = meta.get("description", "")
            except (OSError, ValueError):
                pass
        self.first_timestamp = None
        self.last_timestamp = None
        self.model_calls = 0
        self.last_context_tokens = 0
        self.full_cache_rewrites = 0
        self.full_cache_rewrite_tokens = 0
        self.first_context_tokens = 0
        self.total_context_tokens = 0
        self.fresh_input_tokens = 0
        self.cache_read_tokens = 0
        self.output_tokens = 0
        self.tool_result_characters = {}  # 类别 → [次数, 字符数]
        self.read_characters_by_file = {}  # 文件名 → [次数, 字符数]
        self.bash_commands = []          # [[时间, 命令, 输出（还没回来是 None）, 工具调用 id]]
        self.pending_tool = None         # (时间, 工具名, 命令或路径) —— 发出了还没收到结果
        self.background_started = []     # 它用 run_in_background 起过的后台任务 id
        self.background_output_paths = {}  # 后台任务 id → harness 写它输出的文件（工具结果里带着）
        self.background_finished = set() # 收到过完成通知的后台任务 id
        self.last_turn_end_timestamp = None
        self.last_notification_timestamp = None
        self.handed_back_timestamp = None    # 算交回的那一次交回工具成功返回的时刻
        self.handback_message = ""           # 算交回的那一次交回工具的正文（找「没删 <全路径>」用）
        self.interrupted_timestamp = None    # 最后一次被打断（[Request interrupted）的时刻
        self.task_stopped_timestamp = None   # 按主会话记录里的 TaskStop 判成被停时，那次停的时刻
        self.main_agent_message_delivered_timestamp = None   # 它会话记录里主 agent 的消息最晚一次送达的时刻（origin coordinator）
        self.state = "思考中"
        self.dispatch_prompt = ""            # 会话记录里第一条 user 文字：派发提示（找草稿目录里的 progress.md 用）
        self.last_text = ""                  # 它最后一句话（「中途停下没交回」告警与报告那一节用）
        self._read()
        parent_path = parent_session_transcript(transcript_path)
        self.parent_transcript_missing = not os.path.isfile(parent_path)
        self.parent_transcript_path = parent_path
        # 主会话记录里 SendMessage 发给它的最晚一次成功结果：忙着的子 agent 要等下一轮才把消息落进自己的会话记录
        self.main_agent_message_sent_timestamp = session_task_events(parent_path)[2].get(agent_id)
        stopped_at = task_stop_time(agent_id, parent_path)
        if (stopped_at is not None and BROKEN_DETECTION != "taskstop" and self.state != "已交回"
                and (self.last_timestamp is None or (self.last_timestamp - stopped_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
            self.state = "被停"
            self.task_stopped_timestamp = stopped_at

    def _read(self):
        seen_message_ids = set()
        pending_by_id = {}
        tool_inputs_by_id = {}
        handbacks = transcripts.HandbackTracker()   # 交回送没送达照 subagent_transcript.py 认（调用之后对应的 tool_result 不是错误）
        handback_message = ""
        is_handed_back = False  # 最近一次交回成功之后没有来过新指令（is_new_instruction）；交回之后它自己再调工具不撤销
        handback_timestamp = None
        previous_call_time = None
        last_kind = None
        last_text_timestamp = None
        for line in open(self.transcript_path, encoding="utf-8", errors="replace"):
            try:
                record = json.loads(line)
            except ValueError:
                continue
            timestamp_text = record.get("timestamp")
            if not timestamp_text:
                continue
            timestamp = parse_timestamp(timestamp_text)
            self.first_timestamp = self.first_timestamp or timestamp
            self.last_timestamp = timestamp
            attachment = record.get("attachment")
            is_notification = ((record.get("origin") or {}).get("kind") == "task-notification"
                               or (isinstance(attachment, dict) and attachment.get("type") == "queued_command" and "<task-notification>" in json.dumps(attachment)))
            if is_notification:
                self.background_finished.update(TASK_NOTIFICATION_ID.findall(json.dumps(record.get("message") or attachment, ensure_ascii=False)))
                self.last_notification_timestamp = timestamp
            # 主 agent 的消息送达：空闲时落成 origin 为 coordinator 的 user 记录，忙时落成附件自己带 origin coordinator 的 queued_command
            if (((record.get("origin") or {}).get("kind") == "coordinator"
                 or (isinstance(attachment, dict) and attachment.get("type") == "queued_command"
                     and (attachment.get("origin") or {}).get("kind") == "coordinator"))
                    and BROKEN_DETECTION != "inquirydelivered"):
                self.main_agent_message_delivered_timestamp = timestamp
            # 交回之后来的新指令撤销交回（认法见 is_new_instruction）：还没调工具也已经不算交回——续做的头几秒它在想，看门狗不能当场报「全部交回」退出。
            # 它忙时排进队的续做消息只落成 queued_command 附件、没有 user 记录，所以在这里判，不放进下面按 message 分的那几支。
            # 后台任务的完成通知不算，它叫醒之后又调工具也不算：交回之后调工具不撤销（见 tool_use 那一支的注释）。
            if is_new_instruction(record) and BROKEN_DETECTION != "resume":
                is_handed_back = False
            delivered = handbacks.feed(record)
            if delivered is not None:
                is_handed_back = True
                handback_timestamp = delivered["delivered_at"] or timestamp
                handback_message = delivered["message"]
            message = record.get("message")
            if not isinstance(message, dict):
                continue
            role = message.get("role")
            content = message.get("content")
            if role == "assistant":
                usage = message.get("usage") or {}
                message_id = message.get("id")
                if usage and message_id not in seen_message_ids:
                    seen_message_ids.add(message_id)
                    self.model_calls += 1
                    created = usage.get("cache_creation_input_tokens", 0) or 0
                    read_from_cache = usage.get("cache_read_input_tokens", 0) or 0
                    uncached = usage.get("input_tokens", 0) or 0
                    self.last_context_tokens = read_from_cache + created + uncached
                    self.first_context_tokens = self.first_context_tokens or self.last_context_tokens
                    self.total_context_tokens += self.last_context_tokens
                    self.fresh_input_tokens += created + uncached
                    self.cache_read_tokens += read_from_cache
                    self.output_tokens += usage.get("output_tokens", 0) or 0
                    if created > 100_000 and previous_call_time is not None and (timestamp - previous_call_time).total_seconds() >= 300:
                        self.full_cache_rewrites += 1
                        self.full_cache_rewrite_tokens += created
                    previous_call_time = timestamp
                if isinstance(content, list):
                    for block in content:
                        if block.get("type") == "tool_use":
                            tool_input = block.get("input") or {}
                            detail = tool_input.get("command") or tool_input.get("file_path") or tool_input.get("description") or ""
                            pending_by_id[block.get("id")] = (timestamp, block.get("name"), detail)
                            tool_inputs_by_id[block.get("id")] = (block.get("name"), tool_input)
                            if block.get("name") == "Bash":
                                self.bash_commands.append([timestamp, tool_input.get("command", ""), None, block.get("id")])
                            # 交回之后它自己再调工具不撤销交回，撤销只认新指令：交回之后被自己后台任务的完成通知叫醒、看一眼输出再结束本轮的，
                            # 按调工具撤销会被判成没交回、报「结束本轮却不会醒」（2026-09-23 门禁审计会话的 ac26abd494cda0968：交回、结束本轮、
                            # 超时转后台的 doc-lint 的完成通知进来、grep 一次输出、结束本轮）。
                            if block.get("name") != transcripts.HANDBACK_TOOL and BROKEN_DETECTION == "toolrevoke":
                                is_handed_back = False
                            last_kind = "tool_use"
                        elif block.get("type") == "text":
                            last_kind = "end_turn" if message.get("stop_reason") in TURN_END_STOP_REASONS else "text"
                            last_text_timestamp = timestamp
                            self.last_text = str(block.get("text") or "")
                            if last_kind == "end_turn":
                                self.last_turn_end_timestamp = timestamp
            elif role == "user":
                if not self.dispatch_prompt:
                    self.dispatch_prompt = content if isinstance(content, str) else " ".join(
                        block.get("text", "") for block in (content or []) if isinstance(block, dict) and block.get("type") == "text")
                if isinstance(content, list):
                    for block in content:
                        if block.get("type") == "tool_result":
                            pending_by_id.pop(block.get("tool_use_id"), None)
                            started = background_start_in_result(block.get("content"))
                            if started:
                                task_id, output_path = started
                                self.background_started.append(task_id)
                                if output_path:
                                    self.background_output_paths[task_id] = output_path
                            if block.get("tool_use_id") in tool_inputs_by_id:
                                self._count_tool_result(*tool_inputs_by_id[block.get("tool_use_id")], block.get("content"))
                            for entry in self.bash_commands[-20:]:
                                if entry[3] == block.get("tool_use_id"):
                                    result = block.get("content")
                                    entry[2] = result if isinstance(result, str) else json.dumps(result, ensure_ascii=False)
                            last_kind = "tool_result"
                        elif block.get("type") == "text" and "[Request interrupted" in block.get("text", ""):
                            last_kind = "interrupted"
                            self.interrupted_timestamp = timestamp
                elif isinstance(content, str) and "[Request interrupted" in content:
                    last_kind = "interrupted"
                    self.interrupted_timestamp = timestamp
                # 结束本轮之后来的续做消息：它被叫醒了，在想下一步，不再是「结束本轮」——不改的话续做头几秒它还没落下新记录，
                # 看门狗照上一轮的结束时刻报「结束本轮却不会醒」（2026-09-19 E155 执行员续派第二段，最后动作 4 秒前就被报了）。
                if (record.get("origin") or {}).get("kind") == "coordinator" and BROKEN_DETECTION != "resumeturn":
                    last_kind = "resumed"
        # 一次工具调用只有在「它之后再没有别的记录」时才算还在跑。断网 / 被停时砍掉的那一次调用
        # 只落了发起、永远等不到结果，旧写法会一直把它当成在跑，于是「工具调用过长」每一轮都假报一次
        # （2026-09-22 实测：三条腿被断网停掉、续做之后又调了 5 次，看门狗照旧报那条已死的 Bash 跑了 10 分 55 秒）。
        if BROKEN_DETECTION != "stalepending" and self.last_timestamp is not None:
            pending_by_id = {
                key: item for key, item in pending_by_id.items() if item[0] >= self.last_timestamp
            }
        self.pending_tool = max(pending_by_id.values(), key=lambda item: item[0]) if pending_by_id else None
        # 最后一条是纯文字、stop_reason 为空：流式写入时同一条消息后面还会接工具调用，一次 Write 整份生成能隔几分钟才落下来；
        # 结束本轮了的认主会话记录里的任务通知（text_turn_ended。2026-09-19 c381-r1 攻方腿写一句「等 phase B」就等后台任务，
        # 最后两条记录 stop_reason 都是空的，看门狗判成「思考中」，12 分钟后报「无动静」）。
        if (last_kind == "text" and last_text_timestamp is not None and BROKEN_DETECTION != "settle"
                and self.text_turn_ended(last_text_timestamp)):
            last_kind = "end_turn"
            self.last_turn_end_timestamp = last_text_timestamp
        if last_kind == "interrupted":
            self.state = "被停"
        elif is_handed_back and not pending_by_id and BROKEN_DETECTION != "finished":
            self.state = "已交回"
            self.handed_back_timestamp = handback_timestamp
            self.handback_message = handback_message
        elif self.pending_tool is not None:
            self.state = "执行工具中"
        elif last_kind == "end_turn" and BROKEN_DETECTION == "handback":
            self.state = "已交回"
        elif last_kind == "end_turn":
            self.state = "本轮结束、没交回"
        else:
            self.state = "思考中"

    def text_turn_ended(self, text_timestamp):
        """最后一条是纯文字、stop_reason 为空时，这一轮结束了没有：主会话记录里这个子 agent 的任务通知（completed，或 failed / killed / stopped）
        不早于这条文字才算。只按时长认，文字之后一次 Write 生成几分钟的会被当成结束本轮、报「中途停下没交回」（records/2026-10-23-subagent拆分提案.md
        第三十五节）。主会话记录不在时退回按时长认：过了 TEXT_SETTLE_SECONDS 没接下文算结束。弄坏开关 settlenotification 退回改前只按时长认。"""
        parent_path = parent_session_transcript(self.transcript_path)
        if BROKEN_DETECTION == "settlenotification" or not os.path.isfile(parent_path):
            return (datetime.now(timezone.utc) - text_timestamp).total_seconds() >= TEXT_SETTLE_SECONDS
        notified = session_task_events(parent_path)[1].get(self.agent_id)
        return notified is not None and notified[1] >= text_timestamp

    def _count_tool_result(self, tool_name, tool_input, content):
        text = content if isinstance(content, str) else json.dumps(content, ensure_ascii=False)
        category = tool_result_category(tool_name, tool_input)
        entry = self.tool_result_characters.setdefault(category, [0, 0])
        entry[0] += 1
        entry[1] += len(text)
        if tool_name == "Read":
            file_entry = self.read_characters_by_file.setdefault(os.path.basename(tool_input.get("file_path", "")), [0, 0])
            file_entry[0] += 1
            file_entry[1] += len(text)

    def is_done(self):
        return self.state in ("已交回", "被停")

    def last_main_agent_message_timestamp(self):
        """主 agent 最晚一次给它发消息的时刻：它会话记录里送达的与主会话记录里发出的，取晚的那个；都没有是 None。"""
        sent = None if BROKEN_DETECTION == "inquirysent" else self.main_agent_message_sent_timestamp
        moments = [moment for moment in (self.main_agent_message_delivered_timestamp, sent) if moment is not None]
        return max(moments) if moments else None

    def live_background_tasks(self):
        return [task_id for task_id in self.background_started if task_id not in self.background_finished]

    def orphaned_background_tasks(self):
        """起过、没收到完成通知，而 harness 写它输出的那个文件还在、却没有任何进程开着它：进程已经没了，通知不会再来。
        2026-09-19 c381-r1 攻方腿撞会话限额中断，限额之前起的探针任务就停在这个样子，看门狗一直把它当「还在跑」。
        输出文件路径不知道、或文件不在的，判不了，照旧算还在跑。"""
        if BROKEN_DETECTION == "orphan":
            return []
        candidates = [task_id for task_id in self.live_background_tasks()
                      if os.path.isfile(self.background_output_paths.get(task_id, ""))]
        if not candidates:
            return []
        held = files_held_open_by_any_process()
        return [task_id for task_id in candidates if os.path.realpath(self.background_output_paths[task_id]) not in held]

    def effectively_live_background_tasks(self):
        orphaned = set(self.orphaned_background_tasks())
        return [task_id for task_id in self.live_background_tasks() if task_id not in orphaned]

    def waiting_on_its_own_background_tasks(self):
        """结束本轮、没交回，手里还有没收到完成通知的后台任务：它在等，会话记录里本来就一个字都不写，醒不醒由那些任务决定。
        那些任务的进程动不动归进程一节判（进程无输出、进程过长），不按会话记录的「无动静」判——
        2026-09-19 c381-r1 攻方腿等两个吃满 CPU 的探针，看门狗每次重起都当场报「无动静」退出，盯不住。"""
        if BROKEN_DETECTION == "backgroundwait":
            return False
        return self.state == "本轮结束、没交回" and bool(self.effectively_live_background_tasks())

    def ended_turn_with_nothing_to_wake_it(self, now):
        """结束本轮、没交回、起过的后台任务都完了，结束之后也没有通知进来，而且已经过了宽限。"""
        if self.state != "本轮结束、没交回" or self.effectively_live_background_tasks() or self.last_turn_end_timestamp is None:
            return False
        if self.last_notification_timestamp is not None and self.last_notification_timestamp >= self.last_turn_end_timestamp:
            return False
        return (now - self.last_turn_end_timestamp).total_seconds() >= NO_WAKE_GRACE_SECONDS

    def ended(self, notifications):
        """它还在不在跑：交回、被停或失败了返回 (怎么结束的, 那一刻)，还在跑返回 None。notifications 是 session_task_events 的第二张表。
        失败认它最近一次任务通知的状态在 ENDED_NOTIFICATION_STATUSES 里，且那条通知不早于它自己最后一条记录减
        STOP_AFTER_LAST_RECORD_SECONDS（失败之后又被续做、还在跑的不算，与 TaskStop 同一个判法）。"""
        if self.state == "已交回":
            return "已交回", self.handed_back_timestamp or self.last_timestamp
        if self.state == "被停":
            return "被停", self.task_stopped_timestamp or self.interrupted_timestamp or self.last_timestamp
        status, notified_at = notifications.get(self.agent_id, (None, None))
        if (status in ENDED_NOTIFICATION_STATUSES and BROKEN_DETECTION != "leftovernotified"
                and (self.last_timestamp is None or (self.last_timestamp - notified_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
            return f"任务通知 {status}", notified_at
        return None


def background_start_in_result(content):
    """一条工具结果里起了后台任务的（run_in_background 的「Command running in background with ID: …」，或前台跑满超时被挪进后台的写法）：
    交 (任务 id, 输出文件或 None)；不是交 None。AgentTranscript 与交回闸（own_background_tasks_still_running）共用这一个认法。"""
    text = content if isinstance(content, str) else ""
    started = BACKGROUND_STARTED.match(text) or (BACKGROUND_MOVED.match(text) if BROKEN_DETECTION != "movedbackground" else None)
    if not started:
        return None
    output_path = BACKGROUND_OUTPUT_PATH.search(text)
    return started.group(1), (output_path.group(1) if output_path else None)


def own_background_tasks_still_running(transcript_path):
    """交回闸（.claude/hooks/handback-guard.sh ④）用：这份子 agent 会话记录里它自己起过的后台任务，此刻还有进程开着输出文件的
    （与「交回之后后台还在跑」同一套认法：输出文件还有进程开着就是还在跑，不看完成通知）。
    交 [(任务 id, 输出文件, [(最上层的进程号, 已跑秒数, 命令, 从最底层往上的停止次序)])]，按任务 id 排；会话记录读不了抛 OSError。"""
    outputs = {}
    with open(transcript_path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if "Output is being written to" not in line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            message = record.get("message") if isinstance(record, dict) else None
            content = message.get("content") if isinstance(message, dict) and message.get("role") == "user" else None
            for block in content if isinstance(content, list) else []:
                started = background_start_in_result(block.get("content")) if isinstance(block, dict) and block.get("type") == "tool_result" else None
                if started and started[1]:
                    outputs[started[0]] = started[1]
    if not outputs:
        return []
    holders_by_file = output_files_held_open()
    held = {task_id: holders_by_file[os.path.realpath(path)] for task_id, path in outputs.items() if os.path.realpath(path) in holders_by_file}
    if not held:
        return []
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)

    def bottom_up(pid):
        order = []
        for child in sorted(children.get(pid, [])):
            order += bottom_up(child)
        return order + ([pid] if table[pid]["state"] not in EXITED_PROCESS_STATES else [])

    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    found = []
    for task_id in sorted(held):
        holders = {pid for pid in held[task_id] if pid in table}
        tops = sorted(pid for pid in holders if table[pid]["parent"] not in holders)
        found.append((task_id, outputs[task_id], [(pid, uptime_seconds - table[pid]["start_ticks"] / ticks_per_second, command_for_display(table[pid]["command"]),
                                                   bottom_up(pid)) for pid in tops]))
    return [item for item in found if item[2]]


def files_held_open_by_any_process():
    """本机全部进程开着的文件（按真实路径）：判一个后台任务的输出文件还有没有进程在写。"""
    held = set()
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            descriptors = os.listdir(f"/proc/{entry}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                held.add(os.path.realpath(os.readlink(f"/proc/{entry}/fd/{descriptor}")))
            except OSError:
                continue
    return held


def output_files_held_open():
    """本机进程开着的后台任务输出文件（.output，按真实路径）→ 开着它的进程号集合。"""
    holders_by_file = {}
    if BROKEN_DETECTION == "leftover":
        return holders_by_file
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            descriptors = os.listdir(f"/proc/{entry}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                target = os.readlink(f"/proc/{entry}/fd/{descriptor}")
            except OSError:
                continue
            if target.endswith(".output"):
                holders_by_file.setdefault(os.path.realpath(target), set()).add(int(entry))
    return holders_by_file


TRANSCRIPT_OUTPUT_PATHS_CACHE = {}   # 子 agent 会话记录路径 → ((大小, 修改时刻), 它提到的后台输出文件的真实路径)


def output_paths_mentioned(transcript_path):
    """会话记录里提到过的后台输出文件（真实路径）：只用来挑出值得逐条解析的那几份，归属由 AgentTranscript 按工具结果认。"""
    try:
        status = os.stat(transcript_path)
    except OSError:
        return set()
    key = (status.st_size, status.st_mtime_ns)
    cached = TRANSCRIPT_OUTPUT_PATHS_CACHE.get(transcript_path)
    if cached is not None and cached[0] == key:
        return cached[1]
    with open(transcript_path, "rb") as handle:
        mentioned = {os.path.realpath(match.decode(errors="replace")) for match in BACKGROUND_OUTPUT_PATH_BYTES.findall(handle.read())}
    TRANSCRIPT_OUTPUT_PATHS_CACHE[transcript_path] = (key, mentioned)
    return mentioned


def leftover_background_report(session_dirs, now, thresholds):
    """交回、被停或失败的子 agent 起过的后台任务，它结束过了宽限还有进程开着输出文件：返回 (报告行, 告警, 宽限最早到期的时刻或 None)。
    扫这些会话目录里全部子 agent，不只被看的那几个；只检出、只报，不停进程（检出与处置分开）。"""
    holders_by_file = output_files_held_open()
    grace_seconds = thresholds.leftover_background_grace_minutes * 60
    detail_lines, alerts, pending_until = [], [], None
    scanned, agents_within_grace, table = 0, 0, None
    for session_dir in sorted(session_dirs):
        transcript_paths = sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl")))
        scanned += len(transcript_paths)
        notifications = None
        for transcript_path in transcript_paths:
            if not holders_by_file or not (output_paths_mentioned(transcript_path) & holders_by_file.keys()):
                continue
            agent_id = os.path.basename(transcript_path)[len("agent-"):-len(".jsonl")]
            transcript = AgentTranscript(agent_id, transcript_path)
            if notifications is None:
                notifications = session_task_events(parent_session_transcript(transcript_path))[1]
            ended = transcript.ended(notifications)
            if ended is None:
                continue   # 还在跑的归别的告警管
            how_it_ended, ended_at = ended
            held_tasks = sorted(task_id for task_id, path in transcript.background_output_paths.items()
                                if os.path.realpath(path) in holders_by_file)
            if not held_tasks:
                continue
            since_ended = (now - ended_at).total_seconds()
            if since_ended < grace_seconds and BROKEN_DETECTION != "leftovergrace":
                grace_ends_at = datetime.fromtimestamp(ended_at.timestamp() + grace_seconds, timezone.utc)
                pending_until = grace_ends_at if pending_until is None else min(pending_until, grace_ends_at)
                agents_within_grace += 1
                detail_lines.append(f"  子 agent {agent_id} {how_it_ended}，距今 {format_duration(since_ended)}，它起的后台任务 {', '.join(held_tasks)} "
                                    f"的输出文件还有进程开着：还在 {format_duration(grace_seconds)} 宽限里，过了再查")
                continue
            holder_pids = set().union(*(holders_by_file[os.path.realpath(transcript.background_output_paths[task_id])] for task_id in held_tasks))
            table = table if table is not None else process_table()
            topmost = sorted(pid for pid in holder_pids if pid in table and table[pid]["parent"] not in holder_pids)
            detail_lines.append(f"  子 agent {agent_id} {transcript.agent_type}「{transcript.description}」{how_it_ended}，距今 {format_duration(since_ended)}，"
                                f"它起的后台任务 {', '.join(held_tasks)} 的输出文件还有 {len(holder_pids)} 个进程开着，最上层的 {len(topmost)} 个各报一条")
            ticks_per_second = os.sysconf("SC_CLK_TCK")
            uptime_seconds = float(open("/proc/uptime").read().split()[0])
            for pid in topmost:
                running_seconds = uptime_seconds - table[pid]["start_ticks"] / ticks_per_second
                alerts.append((agent_id, LEFTOVER_BACKGROUND_ALERT,
                               f"进程 {pid} 已跑 {format_duration(running_seconds)}，{command_for_display(table[pid]['command'])[:140]}",
                               f"看它在等什么；确认没用就按进程号停：`{PROCESS_STOP_COMMAND} {pid}`；要留着它接着跑就 --ack 进程:{pid}"))
    summary = (f"交回、被停或失败之后后台还在跑：查了 {scanned} 个子 agent 的会话记录，本机有进程开着的后台输出文件 {len(holders_by_file)} 个，"
               f"报 {len(alerts)} 条，宽限里的 {agents_within_grace} 个子 agent")
    return [summary, *detail_lines], alerts, pending_until


KEPT_COPY_TRANSCRIPT_CACHE = {}   # 会话记录路径 → ((大小, 修改时刻), 交回的时刻或 None, 交回正文)


def handback_of(agent_id, transcript_path):
    """(交回的时刻或 None, 交回正文)：会话记录没变就用上一次读的（交回过的会话记录很少再变）。"""
    try:
        status = os.stat(transcript_path)
    except OSError:
        return None, ""
    key = (status.st_size, status.st_mtime_ns)
    cached = KEPT_COPY_TRANSCRIPT_CACHE.get(transcript_path)
    if cached is not None and cached[0] == key:
        return cached[1], cached[2]
    transcript = AgentTranscript(agent_id, transcript_path)
    handed_back = transcript.handed_back_timestamp if transcript.state == "已交回" else None
    KEPT_COPY_TRANSCRIPT_CACHE[transcript_path] = (key, handed_back, transcript.handback_message)
    return handed_back, transcript.handback_message


def kept_copy_roots():
    """「没删 <路径>」里只认落在这几处底下的路径（工作根、过渡期旧根、/tmp 与本进程的临时目录）：仓副本与编译目录只建在这里，别处的不报、不叫人删。"""
    roots = [*work_roots.scratch_roots(), "/tmp", tempfile.gettempdir()]
    return tuple(dict.fromkeys(os.path.normpath(root) for root in roots))


def kept_copy_paths(texts):
    """交回正文与它点名的报告文件里「没删 <全路径>」点的路径（展开 ~、去掉末尾的 / 与句读），去重、保序。"""
    found = []
    for text in texts:
        for match in KEPT_COPY_MENTION.finditer(text):
            path = os.path.normpath(os.path.expanduser(match.group(1).rstrip("/.,")))
            if path not in found:
                found.append(path)
    return found


def mentioned_report_texts(message):
    """交回正文点名的报告文件（.md，在 kept_copy_roots 底下、不超过 KEPT_COPY_REPORT_MAX_BYTES）的内容；读不了的跳过。"""
    texts = []
    for match in KEPT_COPY_REPORT_MENTION.finditer(message):
        path = os.path.normpath(os.path.expanduser(match.group(1)))
        if not any(path.startswith(root + os.sep) for root in kept_copy_roots()):
            continue
        try:
            if os.path.getsize(path) <= KEPT_COPY_REPORT_MAX_BYTES:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    texts.append(handle.read())
        except OSError:
            continue
    return texts


def kept_copy_report(session_dirs, now, thresholds):
    """交回超过 --kept-copy-hours、交回正文（或它点名的报告文件）里写了「没删 <全路径>」而那个路径还在的：返回 (报告行, 告警)。
    扫这些会话目录里全部子 agent；只报、不删（检出与处置分开），删由主 agent 核过没人在用之后做。
    弄坏开关：keptcopy 一条都不报；keptcopyage 不看交回了多久；keptcopyexists 不看路径还在不在。"""
    hours = getattr(thresholds, "kept_copy_hours", 0) or 0
    if hours <= 0:
        return ["交回之后副本还在：--kept-copy-hours 写的是 0，这一类没查"], []
    if BROKEN_DETECTION == "keptcopy":
        return ["交回之后副本还在：弄坏开关 AGENT_WATCH_BREAK=keptcopy，一条都不报"], []
    threshold_seconds = hours * 3600
    roots = kept_copy_roots()
    scanned = mentioning = within = gone = 0
    detail_lines, alerts = [], []
    for session_dir in sorted(session_dirs):
        for transcript_path in sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))):
            scanned += 1
            agent_id = os.path.basename(transcript_path)[len("agent-"):-len(".jsonl")]
            handed_back, message = handback_of(agent_id, transcript_path)
            if handed_back is None or not message:
                continue
            paths = [path for path in kept_copy_paths([message, *mentioned_report_texts(message)])
                     if any(path.startswith(root + os.sep) for root in roots)]
            if not paths:
                continue
            mentioning += 1
            since = (now - handed_back).total_seconds()
            if since < threshold_seconds and BROKEN_DETECTION != "keptcopyage":
                within += 1
                continue
            still_there = [path for path in paths if os.path.lexists(path) or BROKEN_DETECTION == "keptcopyexists"]
            gone += len(paths) - len(still_there)
            for path in still_there:
                alerts.append((agent_id, KEPT_COPY_ALERT,
                               f"副本 {path} 还在：子 agent 交回 {format_duration(since)} 前在交回报告里写了没删它",
                               f"核过没有进程在用（工作目录、命令行、CARGO_TARGET_DIR 指到它）就删：rm -rf -- '{path}'；"
                               f"还要留着就 --ack {agent_id}:{KEPT_COPY_ALERT}"))
            if still_there:
                detail_lines.append(f"  子 agent {agent_id} 交回 {format_duration(since)} 前写了没删的 {len(paths)} 处，还在 {len(still_there)} 处")
    summary = (f"交回之后副本还在：查了 {scanned} 个子 agent 的会话记录，交回报告里写了「没删 <路径>」的 {mentioning} 个"
               f"（交回不到 {hours:g} 小时、先不报的 {within} 个；已经删掉的 {gone} 处），报 {len(alerts)} 条")
    return [summary, *detail_lines], alerts


def process_start_ticks(pid):
    """/proc/<pid>/stat 第 22 列（进程启动时刻，开机起的时钟滴答数，字符串）；进程不在或读不出是 None。命令名里可以有空格与括号，从最后一个右括号往后切。"""
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    fields = text[text.rfind(")") + 2:].split()
    return fields[PROC_STAT_START_FIELD_AFTER_COMM] if len(fields) > PROC_STAT_START_FIELD_AFTER_COMM else None


def live_sessions():
    """{会话号: 此刻活着的 Claude 进程里最早的启动时刻（秒；登记里没写是 None）}。
    活着 = 登记里的 pid 还在、而且 /proc/<pid>/stat 第 22 列与登记的 procStart 一致（pid 被别的进程占了不算）。
    登记目录读不了、或里面一份能解的登记都没有，返回 None：判不了，不把所有会话当成已失效。"""
    directory = os.environ.get(SESSION_REGISTRY_ENV) or os.path.expanduser("~/.claude/sessions")
    try:
        names = [name for name in os.listdir(directory) if name.endswith(".json")]
    except OSError:
        return None
    live, parsed = {}, 0
    for name in names:
        try:
            with open(os.path.join(directory, name), encoding="utf-8") as handle:
                entry = json.load(handle)
            pid, session_id = int(entry["pid"]), str(entry["sessionId"])
        except (OSError, ValueError, KeyError, TypeError):
            continue
        parsed += 1
        started = process_start_ticks(pid)
        if started is None or (entry.get("procStart") is not None and str(entry["procStart"]) != started):
            continue
        started_at = entry.get("startedAt")
        started_seconds = started_at / 1000 if isinstance(started_at, (int, float)) else None
        if session_id not in live:
            live[session_id] = started_seconds
        elif started_seconds is None or live[session_id] is None:
            live[session_id] = None
        else:
            live[session_id] = min(live[session_id], started_seconds)
    return live if parsed else None


def stale_session_reason(transcript_path, last_timestamp, session_dir, live):
    """重启前旧进程留下的子 agent 记录判已失效，返回原因；不是返回 None。两种：
    一、它所在的会话号不是本会话（--session-dir 的目录名，没给取 CLAUDE_CODE_SESSION_ID）、登记里也没有活着的 Claude 进程；
    二、会话号还活着（重启后接着用同一个会话号），而它最后一条记录早于那个会话现在活着的最早一个进程的启动时刻——启动之前就停了的，不是现在这个进程派的。
    判不了（登记读不出、登记里没写启动时刻、没有最后一条记录）的不判失效。
    弄坏开关：stalesession 两种都不判；stalesessionrestart 不判第二种。"""
    if live is None or BROKEN_DETECTION == "stalesession":
        return None
    session_id = os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))
    current = os.path.basename(os.path.normpath(session_dir)) if session_dir else os.environ.get("CLAUDE_CODE_SESSION_ID")
    if session_id not in live:
        if session_id == current:
            return None   # 本会话自己的进程没登记：判不了，不把本会话的子 agent 全当成失效
        return f"属于已不在的会话 {session_id}（Claude 进程登记里没有活着的），是重启前旧进程留下的记录"
    started = live[session_id]
    if (started is not None and last_timestamp is not None and BROKEN_DETECTION != "stalesessionrestart"
            and last_timestamp.timestamp() < started):
        return f"最后一条记录在会话 {session_id} 现在这个 Claude 进程启动之前，是重启前旧进程留下的记录"
    return None


def scratch_hog_roots(session_ids):
    """要查的草稿根：工作根整个；过渡期旧根 /tmp/claude-<uid> 只看本会话的那一份（<旧根>/<项目>/<会话号>）。环境变量 AGENT_WATCH_SCRATCH_ROOTS 给了就只用它（自证用）。"""
    override = os.environ.get(SCRATCH_ROOTS_ENV)
    if override is not None:
        return [root for root in override.split(os.pathsep) if root]
    roots = [work_roots.work_root()]
    if work_roots.LEGACY_ROOT_IN_TRANSITION:
        for session_id in sorted(session_ids):
            roots.extend(sorted(glob.glob(os.path.join(work_roots.legacy_root(), "*", session_id))))
    return roots


def scratch_hog_kind(path, names):
    """这个目录是什么：带签名对得上的 CACHEDIR.TAG 的是「编译目录」；有 .git（目录或工作树的文件）、或同时有 Cargo.toml 与 crates 的是「仓副本」；都不是 None。
    弄坏开关 scratchhogtag：不认 CACHEDIR.TAG（编译目录一概认不出）。"""
    if "CACHEDIR.TAG" in names and BROKEN_DETECTION != "scratchhogtag":
        try:
            with open(os.path.join(path, "CACHEDIR.TAG"), "rb") as handle:
                if handle.read(len(CACHEDIR_TAG_SIGNATURE)) == CACHEDIR_TAG_SIGNATURE:
                    return "编译目录"
        except OSError:
            pass
    if ".git" in names or ("Cargo.toml" in names and "crates" in names):
        return "仓副本"
    return None


def find_scratch_hogs(roots):
    """从草稿根往下找编译目录与仓副本（认出的不再往里找）：返回 ([(路径, 种类)], 看了几个目录, 是否到顶没看完)。符号链接不跟。"""
    found, visited = [], 0
    stack = [(root, 0) for root in reversed(roots) if os.path.isdir(root)]
    while stack:
        path, depth = stack.pop()
        if visited >= SCRATCH_SCAN_MAX_DIRECTORIES:
            return found, visited, True
        visited += 1
        try:
            entries = list(os.scandir(path))
        except OSError:
            continue
        kind = scratch_hog_kind(path, {entry.name for entry in entries}) if depth > 0 else None
        if kind:
            found.append((path, kind))
            continue
        if depth < SCRATCH_SCAN_MAX_DEPTH:
            stack.extend((entry.path, depth + 1) for entry in sorted(entries, key=lambda item: item.name, reverse=True)
                         if entry.is_dir(follow_symlinks=False))
    return found, visited, False


def newest_modification(path):
    """一处副本里最近一次修改的时刻（秒）：它自己与往里 SCRATCH_MTIME_MAX_DEPTH 层的条目（目录与文件）里 st_mtime 最大的。
    只看这么深、这么多：几十万文件的编译目录不全树走一遍；cargo 重编换掉 target/release/deps 里的文件，那一层目录的修改时刻会动。"""
    newest = os.lstat(path).st_mtime
    stack, seen = [(path, 0)], 0
    while stack and seen < SCRATCH_MTIME_MAX_ENTRIES:
        directory, depth = stack.pop()
        try:
            entries = list(os.scandir(directory))
        except OSError:
            continue
        for entry in entries:
            seen += 1
            try:
                newest = max(newest, entry.stat(follow_symlinks=False).st_mtime)
            except OSError:
                continue
            if depth + 1 < SCRATCH_MTIME_MAX_DEPTH and entry.is_dir(follow_symlinks=False):
                stack.append((entry.path, depth + 1))
    return newest


def scratch_hog_report(roots, now, hours):
    """草稿根里最后修改距今超过 hours 小时的编译目录与仓副本，最久没动的排前面：返回报告行。只报不删（检出与处置分开），删由主 agent 核过没人在用之后做。
    弄坏开关：scratchhog 一条都不报；scratchhogage 不看最后修改的时刻，不到阈值的也报；scratchhogtag 见 scratch_hog_kind。"""
    if hours <= 0:
        return ["草稿根里超阈值的编译目录与仓副本：--scratch-idle-hours 写的是 0，这一类没查"]
    if BROKEN_DETECTION == "scratchhog":
        return ["草稿根里超阈值的编译目录与仓副本：弄坏开关 AGENT_WATCH_BREAK=scratchhog，一条都不报"]
    found, visited, truncated = find_scratch_hogs(roots)
    idle, recent = [], 0
    for path, kind in found:
        try:
            newest = newest_modification(path)
        except OSError:
            continue
        age = now.timestamp() - newest
        if age >= hours * 3600 or BROKEN_DETECTION == "scratchhogage":
            idle.append((age, newest, path, kind))
        else:
            recent += 1
    idle.sort(key=lambda item: -item[0])
    summary = (f"草稿根里超阈值的编译目录与仓副本：查了 {len(roots)} 个根、{visited} 个目录{'（到顶没看完）' if truncated else ''}，找到编译目录 "
               f"{sum(kind == '编译目录' for _, kind in found)} 处、仓副本 {sum(kind == '仓副本' for _, kind in found)} 处；"
               f"最后修改不到 {hours:g} 小时的 {recent} 处不报，报 {len(idle)} 处（按最后修改时刻、最久没动的在前）；只报不删")
    lines = [summary]
    for age, newest, path, kind in idle:
        modified_on = datetime.fromtimestamp(newest, ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")
        lines.append(f"  {kind} {path}：最后修改 {format_duration(age)} 前（{modified_on}）")
    if idle:
        lines.append("   → 怎么办：主 agent 核过没有进程在用（工作目录、命令行、CARGO_TARGET_DIR 指到它）、也不是哪个子 agent 还要复跑的，再逐个 rm -rf -- '<路径>'；"
                     "一批清完提醒用户跑 sudo fstrim -v /")
    return lines


def root_free_path():
    """读可用量的那个路径：工作根；还没建的往上取第一个在的上层（statvfs 要路径在）。"""
    path = work_roots.work_root()
    while not os.path.exists(path) and os.path.dirname(path) != path:
        path = os.path.dirname(path)
    return path


def root_free_bytes(path):
    """path 所在文件系统普通用户还能写的字节数（f_bavail × f_frsize，与 df 的 Avail 同一个数）；环境变量 AGENT_WATCH_ROOT_FREE_BYTES 设了照它（自证用）。"""
    override = os.environ.get(ROOT_FREE_BYTES_VARIABLE)
    if override is not None:
        return int(override)
    status = os.statvfs(path)
    return status.f_bavail * status.f_frsize


def root_free_report(thresholds, free_bytes_of=root_free_bytes):
    """根分区（工作根所在的文件系统）的可用量：返回 (报告行, [(告警名, 说明, 下一步)])，低于 --root-free-gib-alert 报「根分区余量过低」。
    free_bytes_of 是读可用字节数的函数（收路径），自证传假的。弄坏开关 rootfree：不看余量、一条都不报（改前的样子）。"""
    threshold = getattr(thresholds, "root_free_gib_alert", ROOT_FREE_GIB_ALERT_DEFAULT)
    if threshold <= 0:
        return ["根分区余量：--root-free-gib-alert 写的是 0，这一类没查"], []
    if BROKEN_DETECTION == "rootfree":
        return ["根分区余量：弄坏开关 AGENT_WATCH_BREAK=rootfree，不看余量"], []
    path = root_free_path()
    try:
        free = free_bytes_of(path) / BYTES_PER_GIBIBYTE
    except (OSError, ValueError) as error:
        return [f"根分区余量：读不出 {path} 所在文件系统的可用量（{error}），这一类没查"], []
    line = f"根分区余量：工作根 {path} 所在的文件系统可用 {free:.1f} GiB（阈值 {threshold:g} GiB，--root-free-gib-alert）"
    if free >= threshold:
        return [line], []
    amplification_runs = os.path.join(os.environ.get("SINGLEFS_CRASH_AMPLIFICATION_HOME")
                                      or os.path.expanduser("~/.local/share/singlefs/crash-amplification"), "runs")
    explanation = f"工作根 {path} 所在的文件系统只剩 {free:.1f} GiB 可用，低于阈值 {threshold:g} GiB（--root-free-gib-alert），再写下去会撞 No space left on device"
    next_step = (f"先查 {amplification_runs}/ 下各运行目录里的 fetched 与 library-for-peer*（双机驱动送第二台的库快照与取回的库：du -sh {amplification_runs}/*/fetched "
                 f"{amplification_runs}/*/library-for-peer*），再查各会话的副本（du -sh {work_roots.work_root()}/*，大的先看）；以 du 为准，"
                 "只删没有进程在用的（ps 里没有进程的工作目录、命令行或 CARGO_TARGET_DIR 指到它），别人在用的不删；删完再看一次 df，提醒用户跑 sudo fstrim -v /；"
                 f"一时删不了、确认接着跑的用 --ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT}")
    return [line], [(ROOT_FREE_ALERT, explanation, next_step)]


def tool_result_category(tool_name, tool_input):
    """工具结果按「撑大上下文的是哪一类读法」分类。"""
    if tool_name == "Read":
        file_path = tool_input.get("file_path", "")
        whole = "offset" not in tool_input and "limit" not in tool_input
        if "prereg" in file_path:
            kind = "Read 跑前登记"
        elif file_path.endswith(".rs"):
            kind = "Read 源码"
        elif file_path.endswith(".md"):
            kind = "Read 其他 md"
        else:
            kind = "Read 其他"
        return kind + ("（整份）" if whole else "（分段）")
    if tool_name == "Bash":
        command = tool_input.get("command", "")
        if "mutate.sh" in command:
            return "Bash 变异表"
        if re.search(r"\bcargo\b", command):
            return "Bash cargo"
        if re.search(r"\b(sed -n|cat|head|tail)\b", command):
            return "Bash 看文件（sed/cat/head/tail）"
        if re.search(r"\bgrep\b", command):
            return "Bash grep"
        return "Bash 其他"
    return tool_name


def format_tokens(count):
    if count >= 100_000_000:
        return f"{count / 100_000_000:.2f} 亿"
    if count >= 10_000:
        return f"{count / 10_000:.1f} 万"
    return str(count)


def cost_report(agent_ids, session_dir, thresholds):
    lines = []
    totals = {"calls": 0, "fresh": 0, "cache": 0, "output": 0, "rewrite": 0}
    if agent_ids:
        targets = [(agent_id, find_transcript(agent_id, session_dir)) for agent_id in agent_ids]
    else:
        paths = glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))
        targets = [(os.path.basename(path)[len("agent-"):-len(".jsonl")], path) for path in sorted(paths)]
    for agent_id, path in targets:
        if path is None:
            lines.append(f"子 agent {agent_id}：没有会话记录")
            continue
        transcript = AgentTranscript(agent_id, path)
        average = transcript.total_context_tokens // transcript.model_calls if transcript.model_calls else 0
        lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」调用 {transcript.model_calls} 次 "
                     f"起步上下文 {format_tokens(transcript.first_context_tokens)} 平均 {format_tokens(average)} 最后 {format_tokens(transcript.last_context_tokens)} "
                     f"新输入 {format_tokens(transcript.fresh_input_tokens)}（其中整份重写 {transcript.full_cache_rewrites} 次 {format_tokens(transcript.full_cache_rewrite_tokens)}） "
                     f"读缓存 {format_tokens(transcript.cache_read_tokens)} 输出 {format_tokens(transcript.output_tokens)}")
        all_characters = sum(characters for _, characters in transcript.tool_result_characters.values()) or 1
        ranked = sorted(transcript.tool_result_characters.items(), key=lambda item: -item[1][1])
        lines.append("  工具结果：" + "；".join(f"{category} {count} 次 {characters} 字符（{100 * characters / all_characters:.0f}%）"
                                          for category, (count, characters) in ranked[:6]))
        files = sorted(transcript.read_characters_by_file.items(), key=lambda item: -item[1][1])[:3]
        if files:
            lines.append("  读得最多的文件：" + "；".join(f"{name} {count} 次 {characters} 字符" for name, (count, characters) in files))
        totals["calls"] += transcript.model_calls
        totals["fresh"] += transcript.fresh_input_tokens
        totals["cache"] += transcript.cache_read_tokens
        totals["output"] += transcript.output_tokens
        totals["rewrite"] += transcript.full_cache_rewrite_tokens
    lines.append(f"合计：调用 {totals['calls']} 次 新输入 {format_tokens(totals['fresh'])}（整份重写 {format_tokens(totals['rewrite'])}） "
                 f"读缓存 {format_tokens(totals['cache'])} 输出 {format_tokens(totals['output'])}")
    return lines, totals


def find_transcript(agent_id, session_dir=None):
    if session_dir:
        candidate = os.path.join(session_dir, "subagents", f"agent-{agent_id}.jsonl")
        return candidate if os.path.isfile(candidate) else None
    matches = glob.glob(os.path.join(PROJECTS_ROOT, "*", "*", "subagents", f"agent-{agent_id}.jsonl"))
    return matches[0] if matches else None


def elapsed_mark_label(mark_minutes):
    """一格写成人读的样子：整小时写「N 小时」；watch.conf 把 ask-every-minutes 改成不是整小时的，写「M 分钟」。"""
    if mark_minutes % 60 == 0:
        return f"{int(mark_minutes // 60)} 小时"
    return f"{mark_minutes:g} 分钟"


def own_scratch_directories(transcript):
    """派发提示行首「草稿目录：」那几行点名的目录（认法只在 .claude/hooks/lib_prompt_scratch_line.py 的 scratch_directories：~/… 这类写法先展开，
    只留工作根与过渡期旧根底下的），按出现次序；没有那一行返回空表。"""
    return prompt_scratch_line.scratch_directories(transcript.dispatch_prompt or "")


def readable_mtime(path):
    """监控的是正在变化的文件；列出后已删除或不可读时返回 None。"""
    try:
        return os.path.getmtime(path)
    except OSError:
        return None


def progress_file_of(transcript):
    """本提示草稿目录（own_scratch_directories）下的 progress.md，几行点名几个目录的取改得最晚的那一份；没有那一行、或那几个目录下都没有，返回 None，
    不到提示里点名的别处（别的子 agent 的报告、目录）去找。
    弄坏开关 progressanypath 下照改前的取法：提示里点名的工作根或过渡期旧根下的路径都算候选（是 progress.md 就是它，是目录取目录下的，是文件取同目录的）。"""
    if BROKEN_DETECTION == "progressanypath":
        candidates = set()
        for mention in SCRATCH_PATH_MENTION.findall(transcript.dispatch_prompt or ""):
            path = work_roots.expand_written_path(mention).rstrip("/.")
            if os.path.basename(path) == PROGRESS_FILE_NAME:
                candidates.add(path)
            elif os.path.isdir(path):
                candidates.add(os.path.join(path, PROGRESS_FILE_NAME))
            else:
                candidates.add(os.path.join(os.path.dirname(path), PROGRESS_FILE_NAME))
    else:
        candidates = {os.path.join(directory, PROGRESS_FILE_NAME) for directory in own_scratch_directories(transcript)}
    existing = [(moment, path) for path in candidates if os.path.isfile(path)
                if (moment := readable_mtime(path)) is not None]
    return max(existing)[1] if existing else None


def progress_note(transcript, now):
    """告警里附的一句：progress.md 的末几行与修改时刻；派发提示里没有行首「草稿目录：」一行的写取不到，那个目录下没有的写没有。"""
    if BROKEN_DETECTION == "progressnote":
        return ""
    path = progress_file_of(transcript)
    if path is None and BROKEN_DETECTION != "progressanypath" and not own_scratch_directories(transcript):
        return "；派发提示里没有行首「草稿目录：」一行，取不到 progress.md（不到提示点名的别处去找）"
    if path is None:
        return "；派发提示点名的草稿目录下没有 progress.md"
    moment = readable_mtime(path)
    try:
        if moment is None:
            return "；progress.md 在检查期间已消失或不可读"
        modified = datetime.fromtimestamp(moment, timezone.utc)
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = [line.strip() for line in handle.read().splitlines() if line.strip()][-PROGRESS_TAIL_LINES:]
    except OSError:
        return "；progress.md 在检查期间已消失或不可读"
    return (f"；{path} 改于 {format_duration((now - modified).total_seconds())} 前，"
            f"末 {len(lines)} 行：{' ｜ '.join(line[:120] for line in lines) or '（空）'}")


def routine_inquiry_alert(transcript, now, thresholds):
    """「跑满 N 小时要主 agent 问一次」（判法见文件头）：返回 (告警名, 说明, 下一步)，不报返回 None。ask-every-minutes 写 0 就不报。"""
    if BROKEN_DETECTION == "inquiry" or transcript.first_timestamp is None or thresholds.ask_every_minutes <= 0:
        return None
    if BROKEN_DETECTION != "inquiryended" and transcript.ended(session_task_events(transcript.parent_transcript_path)[1]) is not None:
        return None
    last_message = transcript.last_main_agent_message_timestamp()
    counted_from = transcript.first_timestamp
    if BROKEN_DETECTION == "inquiryreset" and last_message is not None:
        counted_from = max(counted_from, last_message)   # 弄坏：退回「从上一次消息起重新计时」
    period_seconds = thresholds.ask_every_minutes * 60
    marks_crossed = int((now - counted_from).total_seconds() // period_seconds)
    if marks_crossed < 1:
        return None
    mark_reached_at = datetime.fromtimestamp(counted_from.timestamp() + marks_crossed * period_seconds, timezone.utc)
    if last_message is not None and last_message >= mark_reached_at:
        return None   # 这一格之后主 agent 发过消息：问过了
    label = elapsed_mark_label(marks_crossed * thresholds.ask_every_minutes)
    message_text = (f"主 agent 上一次给它发消息在 {format_duration((now - last_message).total_seconds())} 以前，早于这一格"
                    if last_message is not None else "派发之后主 agent 一次都没给它发过消息")
    return (f"{ROUTINE_INQUIRY_ALERT_PREFIX}{label}{ROUTINE_INQUIRY_ALERT_SUFFIX}",
            f"从派发起已跑 {format_duration((now - transcript.first_timestamp).total_seconds())}，"
            f"{format_duration((now - mark_reached_at).total_seconds())} 前跑满 {label}；"
            f"{message_text}（只是慢、这一格不问就 --ack '{transcript.agent_id}:{ROUTINE_INQUIRY_ALERT_PREFIX}{label}'，只确认这一格）"
            f"{progress_note(transcript, now)}",
            ROUTINE_INQUIRY_NEXT_STEP)


def stopped_midway_next_step(transcript):
    """「中途停下没交回」的出路（文件头同名一段）：续做，或抽交接摘要重派一个同类 agent。"""
    return (f"主 agent 先发消息让它接着做或交回；不回应的，抽交接摘要、新派一个 {transcript.agent_type} 接着做："
            f"python3 scripts/agents/agent-handover.py --agent {transcript.agent_id} --out <它的草稿目录>/{transcript.agent_id}-handover.md，"
            f"派发提示指到那份摘要与它的草稿目录；看过、判定它就该停在这里的，--ack {transcript.agent_id}:{STOPPED_MIDWAY_ALERT}")


def agent_alerts(transcript, now, thresholds):
    """返回 [(告警名, 说明, 下一步)]。"""
    alerts = []
    if transcript.is_done():
        return alerts
    if transcript.pending_tool is not None:
        started, tool_name, detail = transcript.pending_tool
        running_seconds = (now - started).total_seconds()
        is_wait_loop = tool_name == "Bash" and WAIT_LOOP_PATTERN.search(detail) and "timeout" not in detail
        if is_wait_loop and BROKEN_DETECTION != "loop" and running_seconds >= thresholds.wait_loop_minutes * 60:
            alerts.append(("等待循环", f"{tool_name} 已跑 {format_duration(running_seconds)}，命令是没有超时的等待循环：{detail[:160]}",
                           "主 agent 判断：它等的条件会不会成立（2026-09-17 实测等过一行永远不会写进日志的字）；会成立就接着盯，不会就决定发消息让它改、还是停掉"))
        elif BROKEN_DETECTION != "timeout" and running_seconds >= thresholds.tool_minutes * 60:
            alerts.append(("工具调用过长", f"{tool_name} 已跑 {format_duration(running_seconds)}：{detail[:160]}",
                           "主 agent 判断：这条命令起的进程还在不在动（本报告的进程一节）、预期还要多久；动就接着盯，不动再决定怎么处理"))
    elif (BROKEN_DETECTION != "nowake" and BROKEN_DETECTION != "midwaymerged" and not transcript.background_started
          and transcript.ended_turn_with_nothing_to_wake_it(now)):
        alerts.append((STOPPED_MIDWAY_ALERT, f"结束本轮、没交回已 {format_duration((now - transcript.last_turn_end_timestamp).total_seconds())}，"
                       f"一个后台任务都没起过，没有东西会叫醒它；最后一句：「{' '.join(transcript.last_text.split())[:160]}」",
                       stopped_midway_next_step(transcript)))
    elif BROKEN_DETECTION != "nowake" and transcript.ended_turn_with_nothing_to_wake_it(now):
        orphaned = transcript.orphaned_background_tasks()
        orphan_text = (f"（其中 {', '.join(orphaned)} 没收到完成通知，但它的输出文件已经没有进程开着，通知不会再来）" if orphaned else "")
        alerts.append(("结束本轮却不会醒", f"结束本轮、没交回已 {format_duration((now - transcript.last_turn_end_timestamp).total_seconds())}，"
                       f"起过的 {len(transcript.background_started)} 个后台任务都已结束{orphan_text}，没有东西会叫醒它",
                       "主 agent 看它最后在等什么：发消息让它接着做或交回，或者停掉；多半是它把活自己放了后台（run_in_background 里又加了 &）、忘了交回，"
                       "或者它等的后台任务在一次中断（会话限额、进程被杀）里没了"))
    elif (transcript.last_timestamp is not None and BROKEN_DETECTION != "idle"
          and not transcript.waiting_on_its_own_background_tasks()
          and (now - transcript.last_timestamp).total_seconds() >= thresholds.idle_minutes * 60):
        alerts.append(("无动静", f"最后一条记录在 {format_duration((now - transcript.last_timestamp).total_seconds())} 以前，没有工具在跑",
                       "主 agent 判断：模型调用是不是卡住或撞了限额；看会话记录最后几条，再决定等、发消息续做、还是停掉重派"))
    # 上下文越过一条线：每次调用都要把整段上下文从缓存再读一遍，一旦隔久了还要整份重写，越往后每一步越贵，逼近窗口上限还会被迫压缩。
    # 2026-09-19 实测：同一个执行员被续派三次（第一段、补判、第二段），上下文涨到 95 万，平均每次调用 62.6 万、累计读缓存 2.36 亿，
    # 真正的输出只有 2.9 万——没有任何告警，是用户问起才看见的。
    # 这条只叫主 agent 来看，不是停的理由（用户同日：「不应该限制 280K……应该告诉主 agent 看看任务是不是异常，如果不是就继续跑」）。
    # 只对实现员与执行员报「该交接」，别的类型不报；线写 0 不报（子 agent 上下文多大都不交回）
    if (BROKEN_DETECTION != "context" and (thresholds.context_tokens > 0 or BROKEN_DETECTION == "contextzero")
            and transcript.last_context_tokens >= thresholds.context_tokens
            and (transcript.agent_type in HANDOVER_AGENT_TYPES or BROKEN_DETECTION == "handoveranytype")):
        heaviest = sorted(transcript.tool_result_characters.items(), key=lambda item: item[1][1], reverse=True)[:HANDOVER_TOP_OUTPUTS_SHOWN]
        heaviest_text = "、".join(f"{category} {count} 次共 {characters // 1000}k 字" for category, (count, characters) in heaviest) or "没数到工具结果"
        alerts.append((HANDOVER_ALERT, f"{transcript.agent_type} 最后一次调用的上下文 {transcript.last_context_tokens / 10000:.1f} 万（线在 {thresholds.context_tokens / 10000:.0f} 万），"
                       f"已调用 {transcript.model_calls} 次；吃上下文最多的工具输出：{heaviest_text}",
                       "主 agent 发消息让它停在最近一个编得过（或自证全绿）的点、把做完的与没做的写进报告交回，再新派一个同类 agent 接着做"
                       "（研究脚本 agent-handover.py 抽交接摘要）；看过、判定这一件马上就完的，--ack <agent 号>:该交接"))
    if (BROKEN_DETECTION != "progressstale" and transcript.waiting_on_its_own_background_tasks()
            and getattr(thresholds, "progress_stale_minutes", 0) > 0):
        progress_path = progress_file_of(transcript)
        progress_moment = readable_mtime(progress_path) if progress_path is not None else None
        if progress_moment is not None and (now.timestamp() - progress_moment) >= thresholds.progress_stale_minutes * 60:
            alerts.append((PROGRESS_STALE_ALERT, f"结束本轮在等自己的后台任务，{PROGRESS_FILE_NAME} 超过 {thresholds.progress_stale_minutes:g} 分钟没改"
                           f"{progress_note(transcript, now)}",
                           "主 agent 看它等的后台任务还在不在动（本报告的进程一节）：在动就 --ack <agent 号>:进度文件不涨 接着盯；不动再决定发消息还是停掉"))
    routine_inquiry = routine_inquiry_alert(transcript, now, thresholds)
    if routine_inquiry is not None:
        alerts.append(routine_inquiry)
    recent_entries = transcript.bash_commands[-10:]
    if BROKEN_DETECTION != "repeat":
        counts = {}
        for _, command, output, _ in recent_entries:
            if output is None:
                continue
            key = (" ".join(command.split()), output)
            counts[key] = counts.get(key, 0) + 1
        for (command, _), count in counts.items():
            if count >= thresholds.repeat_count:
                alerts.append(("同一命令反复且输出不变", f"最近 10 次命令里这条出现 {count} 次，每次输出一模一样：{command[:160]}",
                               "定期复检输出会变，一模一样多半是在等一个不会变的结果；主 agent 判断要不要发消息让它换个看法，还是接着等"))
    if BROKEN_DETECTION != "banned":
        banned_window_seconds = max(BANNED_COMMAND_WINDOW_SECONDS, 2 * thresholds.interval_seconds)
        for issued_at, command, output, _ in recent_entries:
            if BROKEN_DETECTION != "bannedwindow" and (now - issued_at).total_seconds() > banned_window_seconds:
                continue
            if (output or "").startswith("PreToolUse:Bash hook error") and BROKEN_DETECTION != "bannedrefused":
                continue   # 入口已拒绝、没执行：hook 检出那一路记着，这里不再报一遍
            if BANNED_COMMAND_PATTERN.search(command_without_heredoc_bodies(command)):
                alerts.append(("禁用命令", f"用了按模式匹配的进程命令：{command[:160]}",
                               "pgrep -f / pkill -f 会命中自己所在的 shell（command-safety.md）；主 agent 判断它等的或要杀的是哪个进程，再发消息让它改用写死的 pid"))
                break
    return alerts


def background_task_processes(transcript):
    """子 agent 在等的后台任务：开着它输出文件的最上层进程，连同子孙进程跑了多久、累计用了多少 CPU、最新的子孙进程多久前起的。
    2026-09-19：报告写「进程动不动见下面进程一节」，而进程一节只列跑满 --process-report-minutes 的叶子进程——
    批次脚本一轮轮起一分钟的探针，一个都不够格，那一节根本不出现，主 agent 只好自己去翻 ps。"""
    if BROKEN_DETECTION == "holders":
        return []
    live = set(transcript.effectively_live_background_tasks())
    wanted = {os.path.realpath(path) for task_id, path in transcript.background_output_paths.items() if task_id in live}
    if not wanted:
        return []
    table = process_table()
    holders = set()
    for pid in table:
        try:
            descriptors = os.listdir(f"/proc/{pid}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                if os.path.realpath(os.readlink(f"/proc/{pid}/fd/{descriptor}")) in wanted:
                    holders.add(pid)
                    break
            except OSError:
                continue
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    def subtree_of(top):
        subtree, frontier = [top], [top]
        while frontier:
            for child in children.get(frontier.pop(), []):
                subtree.append(child)
                frontier.append(child)
        return subtree

    def cpu_ticks_of(process_ids):
        total = 0
        for pid in process_ids:
            try:
                fields = open(f"/proc/{pid}/stat").read().rsplit(")", 1)[1].split()
                total += sum(int(field) for field in fields[11:15])   # 自己的与已回收子进程的用户态、内核态
            except (OSError, ValueError, IndexError):
                continue
        return total

    # 报「此刻占几个核」（隔 CPU_SAMPLE_SECONDS 取两次样），不报累计 CPU：累计量读起来像跑了很久或重跑过，
    # 用户 2026-09-19 看到「累计 11 小时 CPU」以为是重跑的，并明说累计没有意义。
    tops = sorted(pid for pid in holders if table[pid]["parent"] not in holders)
    subtrees = {top: subtree_of(top) for top in tops}
    first_sample = {top: cpu_ticks_of(subtrees[top]) for top in tops}
    time.sleep(CPU_SAMPLE_SECONDS)
    lines = []
    for top in tops:
        cores_now = (cpu_ticks_of(subtrees[top]) - first_sample[top]) / ticks_per_second / CPU_SAMPLE_SECONDS
        youngest = max(table[pid]["start_ticks"] for pid in subtrees[top])
        lines.append(f"    进程 {top} 已跑 {format_duration(uptime_seconds - table[top]['start_ticks'] / ticks_per_second)}，"
                     f"它和 {len(subtrees[top]) - 1} 个子孙进程此刻占 {cores_now:.1f} 个核，"
                     f"最新的一个 {format_duration(uptime_seconds - youngest / ticks_per_second)} 前起：{table[top]['command'][:120]}")
    return lines


def process_table():
    table = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat") as handle:
                stat_text = handle.read()
            with open(f"/proc/{entry}/cmdline", "rb") as handle:
                command_line = handle.read().replace(b"\0", b" ").decode(errors="replace").strip()
        except OSError:
            continue
        after_name = stat_text[stat_text.rfind(")") + 2:].split()
        table[int(entry)] = {"parent": int(after_name[1]), "start_ticks": int(after_name[19]), "state": after_name[0],
                             "cpu_ticks": int(after_name[11]) + int(after_name[12]),
                             "cpu_ticks_with_reaped": sum(int(field) for field in after_name[11:15]),   # 连同已回收子进程的用户态、内核态
                             "command": command_line}
    return table


def claude_instance_pid(table, start_pid):
    pid = start_pid
    while pid in table and pid > 1:
        if "native-binary/claude" in table[pid]["command"] or os.path.basename(table[pid]["command"].split(" ")[0]) == "claude":
            return pid
        pid = table[pid]["parent"]
    return None


def written_files(pid):
    files = []
    try:
        descriptors = os.listdir(f"/proc/{pid}/fd")
    except OSError:
        return files
    for descriptor in descriptors:
        try:
            target = os.readlink(f"/proc/{pid}/fd/{descriptor}")
            with open(f"/proc/{pid}/fdinfo/{descriptor}") as handle:
                flags_line = [line for line in handle if line.startswith("flags:")][0]
        except (OSError, IndexError):
            continue
        access_mode = int(flags_line.split()[1], 8) & 0o3
        if target.startswith("/") and access_mode in (1, 2) and os.path.isfile(target):
            files.append(target)
    return sorted(set(files))


def stdin_pipe_writers(reader_pid, table):
    """这个进程的 stdin 是管道（/proc/<pid>/fd/0 指向 pipe:[inode]）时，本机别的活进程里把同一个管道开在写端的那些进程号。
    它自己与它的祖先不算：管道的上游是兄弟一类，不是祖先；祖先的子孙里就有这个进程自己那一支和不相干的兄弟，拿它的整棵子树判「在动」会把告警整个关掉。
    stdin 不是管道、读不到，或写端进程的 /proc 读不了（别的用户的进程），都当作看不到写端。"""
    try:
        stdin_target = os.readlink(f"/proc/{reader_pid}/fd/0")
    except OSError:
        return []
    if not stdin_target.startswith("pipe:["):
        return []
    ancestors, pid = set(), reader_pid
    while pid in table and pid > 1 and BROKEN_DETECTION != "pipeancestor":
        pid = table[pid]["parent"]
        ancestors.add(pid)
    writers = []
    for other in table:
        if other == reader_pid or other in ancestors or table[other]["state"] in EXITED_PROCESS_STATES:
            continue
        try:
            descriptors = os.listdir(f"/proc/{other}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                if os.readlink(f"/proc/{other}/fd/{descriptor}") != stdin_target:
                    continue
                flags_line = [line for line in open(f"/proc/{other}/fdinfo/{descriptor}") if line.startswith("flags:")][0]
            except (OSError, IndexError):
                continue
            if (int(flags_line.split()[1], 8) & 0o3) in (1, 2):   # 开在写端（只写或读写）
                writers.append(other)
                break
    return writers


def pipe_upstream_activity(reader_pid, earlier, later, thresholds):
    """写着的文件不动、CPU 也不涨的进程，stdin 是管道而上游在动：返回一句说明；看不到写端、或写端连同子孙既不算也不写，返回 None。
    窗口与「进程无输出」同一套：CPU 比 process_alerts 里 earlier、later 那两次采样（隔 CPU_SAMPLE_SECONDS），已回收的子进程与采样之间新起的子孙也算；
    写着的文件看 --process-stale-minutes 以内变没变过。"""
    if BROKEN_DETECTION == "pipeupstreamalways":
        return "（AGENT_WATCH_BREAK=pipeupstreamalways：这条判据恒真）"
    if BROKEN_DETECTION == "pipeupstream":
        return None
    writers = [pid for pid in stdin_pipe_writers(reader_pid, earlier)
               if pid in later and later[pid]["start_ticks"] == earlier[pid]["start_ticks"]]
    if not writers:
        return None
    children = {}
    for pid, info in later.items():
        children.setdefault(info["parent"], []).append(pid)
    subtree, frontier = set(), list(writers)
    while frontier:
        pid = frontier.pop()
        if pid in subtree or pid == reader_pid or later[pid]["state"] in EXITED_PROCESS_STATES:
            continue
        subtree.add(pid)
        frontier.extend(children.get(pid, []))
    cpu_ticks = 0
    for pid in subtree:
        before = earlier.get(pid)
        started_before_window = before is not None and before["start_ticks"] == later[pid]["start_ticks"]
        cpu_ticks += later[pid]["cpu_ticks_with_reaped"] - (before["cpu_ticks_with_reaped"] if started_before_window else 0)
    cpu_seconds = cpu_ticks / os.sysconf("SC_CLK_TCK")
    file_ages = []
    for pid in subtree:
        for path in written_files(pid):
            try:
                file_ages.append(time.time() - os.path.getmtime(path))
            except OSError:
                continue
    newest_age = min(file_ages, default=None)
    writers_text = f"写端进程 {', '.join(str(pid) for pid in sorted(writers))}（连同子孙 {len(subtree)} 个进程）"
    if cpu_seconds > 0:
        return f"stdin 是管道，{writers_text}{CPU_SAMPLE_SECONDS} 秒里用了 {cpu_seconds:.1f} 核秒 CPU"
    if newest_age is not None and newest_age < thresholds.process_stale_minutes * 60:
        return f"stdin 是管道，{writers_text}写着的文件最近一次变动在 {format_duration(newest_age)} 以前"
    return None


def reminder_timer_note(table, pid):
    """pid 是 `remind.sh start` 底下等到点的 sleep：返回一句说明，不是返回 None。
    只认 sleep，且父进程的命令行就是 remind.sh（本身或 bash / sh 起它）接 start：`remind.sh stop`、别的脚本底下的 sleep、remind.sh 底下别的命令都不算。"""
    if BROKEN_DETECTION == "remindertimer":
        return None
    words = table[pid]["command"].split(" ")
    parent = table.get(table[pid]["parent"])
    if os.path.basename(words[0]) != "sleep" or parent is None:
        return None
    parent_words = parent["command"].split(" ")
    for position in (0, 1):
        if position + 1 < len(parent_words) and os.path.basename(parent_words[position]) == "remind.sh" and parent_words[position + 1] == "start" \
                and (position == 0 or os.path.basename(parent_words[0]) in ("bash", "sh", "dash")):
            return f"是定时提醒（{parent_words[position]} start，进程 {table[pid]['parent']}）等到点的计时，不告警"
    return None


def process_alerts(root_pid, excluded_pids, thresholds):
    """这个 Claude 实例底下跑得久的叶子进程：报跑了多久、写的文件多久没动；过期的告警。"""
    table = process_table()
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    with open("/proc/uptime") as handle:
        uptime_seconds = float(handle.read().split()[0])
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    descendants, frontier = [], [root_pid]
    while frontier:
        pid = frontier.pop()
        for child in children.get(pid, []):
            descendants.append(child)
            frontier.append(child)
    lines, alerts, quiet = [], [], []
    for pid in sorted(descendants):
        if pid in excluded_pids or children.get(pid):
            continue
        info = table[pid]
        if any(marker in info["command"] for marker in COMMAND_MARKERS_NOT_WATCHED):
            continue
        elapsed = uptime_seconds - info["start_ticks"] / ticks_per_second
        if elapsed < thresholds.process_report_minutes * 60:
            continue
        reminder_note = reminder_timer_note(table, pid)
        if reminder_note is not None:
            lines.append(f"  进程 {pid} 已跑 {format_duration(elapsed)}，{reminder_note}")
            continue
        files = written_files(pid)
        newest_age = min((time.time() - moment for path in files
                          if (moment := readable_mtime(path)) is not None), default=None)
        age_text = "没有写着的文件" if newest_age is None else f"写着的文件最近一次变动在 {format_duration(newest_age)} 以前"
        lines.append(f"  进程 {pid} 已跑 {format_duration(elapsed)}，{age_text}：{info['command'][:140]}")
        if BROKEN_DETECTION != "stale" and newest_age is not None and elapsed >= thresholds.process_stale_minutes * 60 \
                and newest_age >= thresholds.process_stale_minutes * 60:
            quiet.append((pid, elapsed, newest_age, info))
        elif elapsed >= thresholds.process_max_minutes * 60:
            alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}：{info['command'][:140]}",
                           "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
    # 写着的文件不动的进程，过一小会儿再取一次 CPU 时间：CPU 在涨是在算、只是不报进度（有的测试只在结束时写），报一行、不告警；
    # CPU 也不涨才是可能卡住了，告警。2026-09-19 c381-r1 攻方腿两个吃满 CPU 的探针 21 分钟不写一个字，每个看门狗一起来就被它当场叫醒。
    if quiet:
        time.sleep(CPU_SAMPLE_SECONDS)
        later = process_table()
    for pid, elapsed, newest_age, info in quiet:
        if pid not in later or later[pid]["start_ticks"] != info["start_ticks"]:
            continue   # 采样那两秒里跑完了（或 pid 被别的进程接走）
        cpu_seconds = (later[pid]["cpu_ticks"] - info["cpu_ticks"]) / ticks_per_second
        if BROKEN_DETECTION != "cpu" and cpu_seconds > 0:
            lines.append(f"  进程 {pid} 在算：{CPU_SAMPLE_SECONDS} 秒里用了 {cpu_seconds:.1f} 核秒 CPU，只是写着的文件 {format_duration(newest_age)} 没动（不告警；"
                         f"跑到 {thresholds.process_max_minutes:.0f} 分钟报「进程过长」）")
            if elapsed >= thresholds.process_max_minutes * 60:
                alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}、一直在算却不写：{info['command'][:140]}",
                               "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
            continue
        # 管道尾（`cargo test … | tail -30`、`… | ugrep`）在等上游、不写也不算：上游连同子孙在算或在写就不是卡住，报一行、不告警。
        # 2026-09-24 实测这两种形态各被报过「进程无输出」，上游的测试二进制一直占满一个核。
        upstream = pipe_upstream_activity(pid, table, later, thresholds)
        if upstream is not None:
            lines.append(f"  进程 {pid} 在等上游：{upstream}，它自己写着的文件 {format_duration(newest_age)} 没动（不告警；"
                         f"跑到 {thresholds.process_max_minutes:.0f} 分钟报「进程过长」）")
            if elapsed >= thresholds.process_max_minutes * 60:
                alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}、一直在等还在动的上游：{info['command'][:140]}",
                               "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
            continue
        alerts.append(("进程无输出", f"进程 {pid} 已跑 {format_duration(elapsed)}，写着的文件 {format_duration(newest_age)} 没变，"
                                     f"{CPU_SAMPLE_SECONDS} 秒里 CPU 也没涨：{info['command'][:140]}",
                       "既不写也不算，可能卡在等什么（锁、管道、网络、另一个进程）；主 agent 看它在等什么（/proc/<pid>/wchan、它的父进程与输入），"
                       "再决定接着盯、查、还是报给用户"))
    verdict_lines, verdict_alerts = verdict_line_report(root_pid, descendants, table, excluded_pids, thresholds, uptime_seconds, ticks_per_second)
    return lines + verdict_lines, alerts + verdict_alerts


def verdict_counts(path):
    """一份文件里的判定行（文件头「长跑里有判红的行」）：交 (✓ 行数, ✗ 行数, ✗ 行（至多 VERDICT_RED_SHOWN 行）, 只数了末尾没有)；读不了交 None。"""
    try:
        size = os.path.getsize(path)
        with open(path, "rb") as handle:
            partial = size > VERDICT_SCAN_BYTES
            if partial:
                handle.seek(size - VERDICT_SCAN_BYTES)
                handle.readline()   # 切在一行中间：丢掉那半行
            data = handle.read()
    except OSError:
        return None
    green, red, red_lines = 0, 0, []
    for line in data.decode("utf-8", errors="replace").splitlines():
        match = VERDICT_LINE.match(line)
        if not match:
            continue
        if match.group(1) == "✓":
            green += 1
        else:
            red += 1
            if len(red_lines) < VERDICT_RED_SHOWN:
                red_lines.append(" ".join(line.split())[:160])
    return green, red, red_lines, partial


def verdict_line_report(root_pid, descendants, table, excluded_pids, thresholds, uptime_seconds, ticks_per_second):
    """「长跑里有判红的行」（判法见文件头）：跑满 --process-report-minutes 的进程开着写的文件里数判定行。返回 (报告行, [(告警名, 说明, 下一步)])。"""
    if BROKEN_DETECTION == "verdictlines":
        return [], []
    own_files = set(written_files(root_pid))   # 根进程自己开着写的（自证时是跑它的那一份日志）不数
    holders = {}   # 文件 → 开着它写的、跑得最久的那个进程号
    for pid in descendants:
        info = table.get(pid)
        if info is None or pid in excluded_pids or any(marker in info["command"] for marker in COMMAND_MARKERS_NOT_WATCHED):
            continue
        if uptime_seconds - info["start_ticks"] / ticks_per_second < thresholds.process_report_minutes * 60:
            continue
        for path in written_files(pid):
            if path in own_files:
                continue
            if path not in holders or info["start_ticks"] < table[holders[path]]["start_ticks"]:
                holders[path] = pid
    lines, alerts, quiet_files, in_selftest_scratch = [], [], 0, 0
    for path, pid in sorted(holders.items()):
        counted = verdict_counts(path)
        if counted is None or counted[0] + counted[1] == 0:
            quiet_files += 1
            continue
        green, red, red_lines, partial = counted
        shown = f"；✗ 的是：{' ｜ '.join(red_lines)}{f' ｜ 另 {red - len(red_lines)} 行' if red > len(red_lines) else ''}" if red else ""
        partial_text = f"（只数了末尾 {VERDICT_SCAN_BYTES // (1024 * 1024)} MiB）" if partial else ""
        owner = light_selftest_scratch_owner(pid, path, table, root_pid) if red else None
        if owner is not None:
            in_selftest_scratch += 1
            lines.append(f"  判定行 {path}（进程 {pid} 写着，在登记成轻的自证 {owner} 的临时目录里，✗ 不报）：✓ {green} ✗ {red}{partial_text}")
            continue
        lines.append(f"  判定行 {path}（进程 {pid} 写着）：✓ {green} ✗ {red}{partial_text}{shown}")
        if red:
            alerts.append((VERDICT_RED_ALERT, f"进程 {pid} 写着的 {path} 里判定行 ✓ {green} ✗ {red}{partial_text}{shown}",
                           f"主 agent 当场把这一趟记进调度记录：「这一趟 ✓ {green} ✗ {red}，✗ 的是：…」（红的逐条写名字），再决定接着跑还是停；"
                           f"记下了就 --ack 进程:{pid}（之后多出来的红照样列在报告的判定行那一行里）"))
    if holders:
        _, scratch_problem = selftest_scratch_prefixes()
        lines.insert(0, f"判定行：跑满 {thresholds.process_report_minutes:g} 分钟的进程开着写的 {len(holders)} 份文件，"
                        f"{len(holders) - quiet_files} 份里有 ✓ / ✗ 开头的行，报 {len(alerts)} 条，"
                        f"登记成轻的自证临时目录里有 ✗ 的 {in_selftest_scratch} 份不报"
                        f"{'（自证临时目录只认 mktemp 那一种：' + scratch_problem + '）' if scratch_problem else ''}")
    return lines, alerts


SELFTEST_SCRATCH_PREFIXES = None   # (探查目录前缀的元组, 读不了的原因或 None)：第一次用时按文件路径读 lib_selftest_scratch.py，之后复用


def selftest_scratch_prefixes():
    """.claude/hooks/lib_selftest_scratch.py 建探查目录用的两个前缀（PASS_PROBE_PREFIX、CHILD_PROBE_PREFIX）：交 (前缀元组, None)；读不了交 ((), 原因)。"""
    global SELFTEST_SCRATCH_PREFIXES
    if SELFTEST_SCRATCH_PREFIXES is None:
        try:
            spec = importlib.util.spec_from_file_location("lib_selftest_scratch_for_verdict_lines", SELFTEST_SCRATCH_LIBRARY_PATH)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            SELFTEST_SCRATCH_PREFIXES = ((module.PASS_PROBE_PREFIX, module.CHILD_PROBE_PREFIX), None)
        except Exception as error:   # 文件不在、语法错、少了那两个名字都算：只认 mktemp 那一种，报告里写明
            SELFTEST_SCRATCH_PREFIXES = ((), f"读不了 {SELFTEST_SCRATCH_LIBRARY_PATH}（{type(error).__name__}：{error}）")
    return SELFTEST_SCRATCH_PREFIXES


def light_selftest_scratch_owner(holder_pid, path, table, root_pid):
    """有 ✗ 的文件 path 是不是登记成轻的自证在自己临时目录里写的（文件头「长跑里有判红的行」）：从写它的进程往上到看门狗的根进程为止，
    找登记成轻的自证（registered_light_selftest，与「重型测试没带前缀」同一份认法），path 落在它的临时目录里就交那份自证的说明，不是交 None。
    它的临时目录：它环境里的 TMPDIR（没有是 /tmp）是单给它的（private_temporary_base）就是整个 TMPDIR；不是单给它的，是 TMPDIR 下一层
    名字像 mktemp / mkdtemp 起的（SELFTEST_TEMPORARY_NAME）或以 lib_selftest_scratch.py 的探查目录前缀起头的那一层。
    弄坏开关：verdictselftest 一律交 None；verdictselftestanyfile 不看路径；verdictselftestpathonly 不看自证、只看路径里有没有一层像 mktemp 临时目录；
    verdictselftestsharedtmpdir 不管 TMPDIR 是不是单给它的，一律整个算；verdictselftestprivateignored 不认单给它的 TMPDIR，只认下一层；
    verdictselftestwrapperseen 不跳过只是包着它的那几层，拿它的父进程当起它的那一层。"""
    if BROKEN_DETECTION == "verdictselftest":
        return None
    if BROKEN_DETECTION == "verdictselftestpathonly":
        return "（弄坏开关 verdictselftestpathonly：只看路径）" if any(SELFTEST_TEMPORARY_NAME.match(part) for part in path.split("/")) else None
    library, _ = heavy_tests_library()
    if library is None:
        return None
    prefixes, _ = selftest_scratch_prefixes()
    walker = holder_pid
    while walker in table and walker > 1 and walker != root_pid:
        arguments = process_arguments(walker)
        if arguments:
            try:
                environment = process_environment(walker)
                described = registered_light_selftest(library, arguments, os.readlink(f"/proc/{walker}/cwd"), environment)
            except OSError:
                described = None   # 刚退出、权限不够：不当自证
            except Exception:   # 共用判定自己的 bug：不当自证，照报（宁可多报一条）
                described = None
            if described:
                if BROKEN_DETECTION == "verdictselftestanyfile":
                    return described
                base = os.path.realpath(environment.get("TMPDIR") or "/tmp")
                if path.startswith(base.rstrip("/") + "/"):
                    top = path[len(base.rstrip("/")) + 1:].split("/", 1)[0]
                    if SELFTEST_TEMPORARY_NAME.match(top) or (prefixes and top.startswith(prefixes)) \
                            or private_temporary_base(walker, arguments, base, table, root_pid):
                        return described
        walker = table[walker]["parent"]
    return None


def private_temporary_base(pid, arguments, base, table, root_pid):
    """进程 pid（参数 arguments、TMPDIR 解开是 base）的 TMPDIR 是不是单给它的：往上跳过只是包着它的那几层（参数以它的参数收尾的：strace … --、
    light-selftest-guard.py … --、timeout、nice 这类），到第一个不是包着它的上层（到看门狗的根进程为止也算），那一层的 TMPDIR（没有是 /tmp）与 base 不同就是。
    同一个 TMPDIR 也给了它的上层的（门禁阶段的私有 TMPDIR 里直接起的自证），整个 TMPDIR 不算它的，只认下一层。读不到环境的当不是。"""
    if BROKEN_DETECTION == "verdictselftestsharedtmpdir":
        return True
    if BROKEN_DETECTION == "verdictselftestprivateignored":
        return False
    walker = table[pid]["parent"] if pid in table else None
    while walker in table and walker > 1 and walker != root_pid and BROKEN_DETECTION != "verdictselftestwrapperseen":
        upper = process_arguments(walker) or []
        if len(upper) <= len(arguments) or upper[-len(arguments):] != arguments:
            break
        walker = table[walker]["parent"]
    if walker is None or walker <= 1:
        return False
    try:
        launcher = os.path.realpath(process_environment(walker).get("TMPDIR") or "/tmp")
    except OSError:
        return False
    return launcher != base


HEAVY_TESTS_LIBRARY = None   # (模块或 None, 导入失败的原因或 None)：第一次用时按文件路径导入，之后复用


def heavy_tests_library():
    """.claude/hooks/lib_heavy_tests.py（它自己再导入同目录的 lib_shell_words.py）：返回 (模块, None)，导入失败返回 (None, 原因)。"""
    global HEAVY_TESTS_LIBRARY
    if HEAVY_TESTS_LIBRARY is None:
        try:
            spec = importlib.util.spec_from_file_location("lib_heavy_tests", HEAVY_TESTS_LIBRARY_PATH)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            HEAVY_TESTS_LIBRARY = (module, None)
        except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败都算：报告里写明这一类没查，不让看门狗整个退出
            HEAVY_TESTS_LIBRARY = (None, f"导入 {HEAVY_TESTS_LIBRARY_PATH} 失败（{type(error).__name__}：{error}）")
    return HEAVY_TESTS_LIBRARY


def process_arguments(pid):
    """/proc/<pid>/cmdline 按 NUL 切开的那一串；读不到或是空的（内核线程、僵尸）交 None。"""
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            raw = handle.read()
    except OSError:
        return None
    if not raw:
        return None
    return [part.decode("utf-8", "replace") for part in raw.rstrip(b"\0").split(b"\0")]


def heavy_test_occasion(pid):
    """这个进程环境里 SINGLEFS_HEAVY_TESTS 的值，没有是 None；读不到 /proc/<pid>/environ 抛 OSError。"""
    with open(f"/proc/{pid}/environ", "rb") as handle:
        for entry in handle.read().split(b"\0"):
            if entry.startswith(HEAVY_TEST_VARIABLE.encode() + b"="):
                return entry.split(b"=", 1)[1].decode("utf-8", "replace")
    return None


def is_watchdog_process(arguments):
    """python 起的 agent-watch.py（别的看门狗与它的自检）：它那一支整棵不扫，自检起的假重型进程不是真在跑的测试。"""
    if not os.path.basename(arguments[0]).startswith("python"):
        return False
    script = next((argument for argument in arguments[1:] if not argument.startswith("-")), "")
    return os.path.basename(script) == "agent-watch.py"


def is_sample_selftest_process(arguments):
    """样本自检：参数里有一个是 stage-selftest.sh（按文件名认），或有一个恰好是 --selftest。它自己照判，它底下整棵不判：
    stage-selftest.sh 在临时目录里带 --force 拿样本起 checker-tier-crash-point-replay 与 checker-tier-* 那几道重型阶段，`--selftest` 建的临时仓里起 gate.sh，都不是真在跑的重型测试。"""
    return any(argument == "--selftest" or os.path.basename(argument) == "stage-selftest.sh" for argument in arguments)


def is_shell_code_string_process(library, arguments):
    """`bash -c '…'` 这一层：前缀写在代码串里、不在这个 shell 自己的环境里，它起的命令各自是一个进程，各判各的。"""
    shell_words = library.shell_words
    return (os.path.basename(arguments[0]) in shell_words.SHELL_NAMES
            and shell_words.shell_invocation(arguments[1:]).code_string is not None)


def agent_meta(transcript_path):
    """子 agent 会话记录旁边 .meta.json 里的 (agentType, description)；读不到是 ("?", "")。"""
    try:
        meta = json.load(open(transcript_path[: -len(".jsonl")] + ".meta.json", encoding="utf-8"))
        return meta.get("agentType", "?"), meta.get("description", "")
    except (OSError, ValueError):
        return "?", ""


def owners_by_background_output(pids, table, session_dirs, stop_at_pid):
    """{进程号: 归属说明}：它自己或它的上层（到 stop_at_pid 为止）开着某个后台任务的输出文件（.output），
    会话记录里写着这个输出文件的那个子 agent；主会话记录里写着的算主 agent。与「交回之后后台还在跑」同一套认法。"""
    held_by = {}
    for path, holder_pids in output_files_held_open().items():
        for holder in holder_pids:
            held_by.setdefault(holder, set()).add(path)
    files_by_pid = {}
    for pid in pids:
        files, walker = set(), pid
        while walker in table and walker > 1 and walker != stop_at_pid:
            files |= held_by.get(walker, set())
            walker = table[walker]["parent"]
        files_by_pid[pid] = files
    mentioned_by = []   # [(说明, 它提到的输出文件)]
    if any(files_by_pid.values()):
        for session_dir in sorted(session_dirs):
            for transcript_path in sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))):
                agent_type, description = agent_meta(transcript_path)
                agent_id = os.path.basename(transcript_path)[len("agent-"):-len(".jsonl")]
                mentioned_by.append((f"子 agent {agent_id} {agent_type}「{description}」", output_paths_mentioned(transcript_path)))
            mentioned_by.append((f"主 agent（会话 {os.path.basename(os.path.normpath(session_dir))}）",
                                 output_paths_mentioned(os.path.normpath(session_dir) + ".jsonl")))
    owners = {}
    for pid, files in files_by_pid.items():
        found = [f"{who}（开着它后台任务的输出文件 {os.path.basename(path)}）"
                 for who, mentioned in mentioned_by for path in sorted(files & mentioned)]
        if found:
            owners[pid] = "；".join(found)
        elif not session_dirs:
            owners[pid] = "认不出（没给会话目录，不知道去哪个会话的记录里找）"
        else:
            owners[pid] = "认不出（它和它的上层都没开着哪个子 agent 或主 agent 后台任务的输出文件：前台跑的、命令里自己放后台的认不出）"
    return owners


# 「重型测试没带前缀」里登记成轻的自证（文件头那一段）：`<名>.sh --selftest` exec 进同目录 `<名>-selftest.sh` 本体的，本体按 `<名>.sh` 的登记判
SELFTEST_BODY_SUFFIX = "-selftest"
# 自证的临时目录：临时根下一层、名字像 mktemp（tmp.XXXXXXXXXX）或 python 的 mkdtemp（tmpXXXXXXXX）起的
SELFTEST_TEMPORARY_NAME = re.compile(r"^tmp\.?[A-Za-z0-9_]{6,}$")


def process_environment(pid):
    """/proc/<pid>/environ 读成 {变量: 值}；读不到抛 OSError。"""
    with open(f"/proc/{pid}/environ", "rb") as handle:
        entries = handle.read().split(b"\0")
    environment = {}
    for entry in entries:
        name, separator, value = entry.partition(b"=")
        if separator:
            environment[name.decode("utf-8", "replace")] = value.decode("utf-8", "replace")
    return environment


def launched_script(library, arguments):
    """进程起的是哪个脚本：(命令词, 它的参数)。bash / sh 这类与 python 起的取解释器后面那个脚本，直接执行的取 argv[0]；
    bash -c、python -c / -m、从标准输入读的交 (None, [])。"""
    shell_words = library.shell_words
    name = os.path.basename(arguments[0])
    if name in shell_words.SHELL_NAMES:
        invocation = shell_words.shell_invocation(arguments[1:])
        if invocation.code_string is not None or invocation.script_index is None:
            return None, []
    elif shell_words.PYTHON_NAME.match(name):
        invocation = shell_words.python_invocation(arguments[1:])
        if invocation.code_string is not None or invocation.module is not None or invocation.script_index is None:
            return None, []
    else:
        return arguments[0], arguments[1:]
    index = invocation.script_index + 1
    if index >= len(arguments):
        return None, []
    return arguments[index], arguments[index + 1:]


def registered_light_selftest(library, arguments, directory, environment):
    """这个进程是不是登记成轻的自证：起的脚本照 lib_heavy_tests.py 的 classify_selftest 判（同一份登记表，命令词要解到本仓那一份），
    登记成 light、或 heavy-if-set:<变量> 而 environment 里那个变量没设。`<名>-selftest.sh` 本体不带参数时另按同目录 `<名>.sh --selftest` 判，
    `<名>.sh` 的正文里要写着本体的文件名（它 exec 进本体）。是就交一句说明，不是交 None。"""
    if BROKEN_DETECTION == "heavyselftestancestor":
        return None
    word, rest = launched_script(library, arguments)
    if not word:
        return None
    if BROKEN_DETECTION == "heavyselftestcondition":
        environment = {}
    candidates = [(word, rest, None)]
    stem, extension = os.path.splitext(word)
    if stem.endswith(SELFTEST_BODY_SUFFIX) and not rest:
        candidates.append((stem[: -len(SELFTEST_BODY_SUFFIX)] + extension, ["--selftest"], word))
    for candidate, candidate_arguments, body in candidates:
        if BROKEN_DETECTION == "heavyselftestanycopy":
            key = library.selftest_registry_key(candidate, directory)
            if key is not None:
                candidate = os.path.join(library.SELFTEST_REGISTRY_ROOT, key)
        is_selftest, test = library.classify_selftest(candidate, candidate_arguments, directory, environment)
        if not is_selftest or test is not None:
            continue
        if body is None:
            return " ".join([word, *rest])
        front = library.shell_words.resolve_path(directory, os.path.expanduser(candidate))
        try:
            with open(front, encoding="utf-8", errors="replace") as handle:
                if os.path.basename(body) in handle.read():
                    return f"{body}（{candidate} --selftest exec 进来的本体）"
        except (OSError, TypeError):
            continue
    return None


def light_tier_run(library, arguments, directory):
    """这个进程是不是在跑轻型三条（`.claude/hooks/lib_heavy_tests.py` 的 light_tier_command 认得的）：起的脚本取法同 registered_light_selftest，
    是就交一句说明，不是交 None。弄坏开关 heavylighttier 下一律交 None，heavylighttierargs 下只看脚本名、不看参数。"""
    if BROKEN_DETECTION == "heavylighttier":
        return None
    word, rest = launched_script(library, arguments)
    if not word:
        return None
    if BROKEN_DETECTION == "heavylighttierargs":
        rest = [*rest, "--quick"]
    return library.light_tier_command([word, *rest], directory)


def process_paths(pid, arguments):
    """进程参数里的绝对路径与它的当前目录（都 normpath 过）；当前目录读不到就不算它。"""
    paths = [os.path.normpath(argument) for argument in arguments if argument.startswith("/")]
    try:
        paths.append(os.path.normpath(os.readlink(f"/proc/{pid}/cwd")))
    except OSError:
        pass
    return paths


def selftest_temporary_root(path, temporary_base):
    """path 落在临时根下一层名字像 mktemp / mkdtemp 起的目录里时交那一层（临时根/tmp.XXXX），不是交 None。"""
    if not path.startswith(temporary_base.rstrip("/") + "/"):
        return None
    top = path[len(temporary_base.rstrip("/")) + 1:].split("/", 1)[0]
    return os.path.join(temporary_base, top) if SELFTEST_TEMPORARY_NAME.match(top) else None


def heavy_test_report(root_pid, excluded_pids, session_dirs):
    """这个 Claude 实例底下在跑的重型测试没带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request（判法见文件头）：返回 (报告行, [(告警名, 说明, 下一步)])。"""
    if BROKEN_DETECTION == "heavyscan":
        return [], []
    library, problem = heavy_tests_library()
    if library is None:
        return [f"{HEAVY_TEST_ALERT}：这一类没查——{problem}"], []
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    flagged, judgement_errors = {}, []   # flagged：进程号 → (认出的重型用法, SINGLEFS_HEAVY_TESTS 的值)
    # frontier 里每项是 (进程号, 它或它的上层是不是样本自检, 它或它的上层里最近的登记成轻的自证的进程号)：这两类底下的进程不判（判法见文件头「重型测试没带前缀」）
    examined, shell_layers, under_sample_selftest, frontier = 0, 0, 0, [(root_pid, False, None)]
    under_registered_selftest, registered_selftest_members = 0, {}   # 登记成轻的自证的进程号 → 它底下的进程号
    under_light_tier, light_tier_owners = 0, set()   # 轻型三条（light_tier_run）的进程号，与它们底下不判的进程数
    while frontier:
        parent, below_sample_selftest, selftest_owner = frontier.pop()
        for child in sorted(children.get(parent, [])):
            arguments = process_arguments(child)
            if arguments and is_watchdog_process(arguments) and BROKEN_DETECTION != "heavywatchdog":
                continue   # 别的看门狗与它的自检：整棵不扫
            owner = selftest_owner
            if owner is None and arguments and child not in excluded_pids and table[child]["state"] not in EXITED_PROCESS_STATES:
                try:
                    child_directory = os.readlink(f"/proc/{child}/cwd")
                    if registered_light_selftest(library, arguments, child_directory, process_environment(child)):
                        owner = child   # 它自己照判，它底下整棵不判
                    elif light_tier_run(library, arguments, child_directory):
                        owner = child   # 轻型三条：同样它自己照判，它底下整棵不判
                        light_tier_owners.add(child)
                except OSError:
                    pass   # 刚退出、权限不够：不当自证
                except Exception as error:   # 共用判定自己的 bug：写进报告，这个进程不当自证
                    judgement_errors.append(f"  进程 {child} 没判成是不是登记成轻的自证（{type(error).__name__}：{error}）：{command_for_display(table[child]['command'])[:100]}")
            frontier.append((child, below_sample_selftest or (bool(arguments) and is_sample_selftest_process(arguments)
                                                              and BROKEN_DETECTION != "heavysampleselftest"), owner))
            if child in excluded_pids or table[child]["state"] in EXITED_PROCESS_STATES or not arguments:
                continue
            examined += 1
            if below_sample_selftest:
                under_sample_selftest += 1
                continue
            if selftest_owner in light_tier_owners:
                under_light_tier += 1
                continue
            if selftest_owner is not None:
                under_registered_selftest += 1
                registered_selftest_members.setdefault(selftest_owner, []).append((child, arguments))
                continue
            if is_shell_code_string_process(library, arguments) and BROKEN_DETECTION != "heavywrapper":
                shell_layers += 1
                continue
            try:
                directory = os.readlink(f"/proc/{child}/cwd")
                tests = library.classify_process(arguments, directory)
                # 「提交时才跑的检查」（不重型的门禁阶段、上游几道 lint、登记表外或登记为 heavy 的自证）不在进程这一层报：执行前的闸 heavy-test-guard.sh 拒，
                # 收工钩子 gate-reuse-check.sh 每次收工起的 gate-overlap.py 也是这一类，在这里报就是每次收工叫醒一次（文件头「重型测试没带前缀」）
                if BROKEN_DETECTION != "heavycommitcheck":
                    tests = [test for test in tests if test.kind not in getattr(library, "COMMIT_CHECK_KINDS", frozenset())]
                if tests:
                    flagged[child] = (tests, heavy_test_occasion(child))
            except OSError:
                continue   # 刚退出、权限不够：跳过不报
            except Exception as error:   # 共用判定自己的 bug：这一个进程没判成，写进报告，别的照判
                judgement_errors.append(f"  进程 {child} 没判成（{type(error).__name__}：{error}）：{command_for_display(table[child]['command'])[:100]}")
    # 登记成轻的自证的临时目录：它底下进程的参数与当前目录里出现过的、临时根（它环境里的 TMPDIR，没有是 /tmp）下一层像 mktemp 起的那一层；
    # 不在它底下、参数里的路径或当前目录落在里面的重型进程不报（文件头「重型测试没带前缀」）
    selftest_temporary_roots = set()
    for owner, members in registered_selftest_members.items():
        try:
            temporary_base = os.path.normpath(process_environment(owner).get("TMPDIR") or "/tmp")
        except OSError:
            continue
        for member, member_arguments in members:
            for path in process_paths(member, member_arguments):
                root = selftest_temporary_root(path, temporary_base)
                if root is not None:
                    selftest_temporary_roots.add(root)
    in_selftest_temporary = 0
    for pid in sorted(flagged):
        paths = process_paths(pid, process_arguments(pid) or [])
        if BROKEN_DETECTION == "heavyselftestpathonly":
            inside = any(SELFTEST_TEMPORARY_NAME.match(part) for path in paths for part in path.split("/"))
        else:
            inside = BROKEN_DETECTION != "heavyselftesttemporary" and any(
                path == root or path.startswith(root + "/") for path in paths for root in selftest_temporary_roots)
        if inside:
            del flagged[pid]
            in_selftest_temporary += 1

    def descendants_of(pid):
        found, stack = set(), [pid]
        while stack:
            for child in children.get(stack.pop(), []):
                if child not in found:
                    found.add(child)
                    stack.append(child)
        return found

    def ancestors_of(pid):
        found, walker = set(), table[pid]["parent"]
        while walker in table and walker > 1 and walker != root_pid:
            found.add(walker)
            walker = table[walker]["parent"]
        return found

    def bottom_up(pid):
        """整棵子树从最底层往上的次序（先子孙、后自己），只列还活着的。"""
        order = []
        for child in sorted(children.get(pid, [])):
            order += bottom_up(child)
        if table[pid]["state"] not in EXITED_PROCESS_STATES:
            order.append(pid)
        return order

    prefixed = {pid for pid, (_, occasion) in flagged.items() if occasion in HEAVY_TEST_OCCASIONS and BROKEN_DETECTION != "heavyprefix"}
    unprefixed = {pid for pid in flagged if pid not in prefixed
                  and (BROKEN_DETECTION == "heavyexempt" or not (descendants_of(pid) & prefixed))}
    reported = sorted(pid for pid in unprefixed if BROKEN_DETECTION == "heavytopmost" or not (ancestors_of(pid) & unprefixed))
    owners = owners_by_background_output(reported, table, session_dirs, root_pid) if reported else {}
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    alerts = []
    for pid in reported:
        tests, occasion = flagged[pid]
        below = sorted(descendants_of(pid) & unprefixed)
        categories = list(dict.fromkeys([test.category for test in tests] + [test.category for other in below for test in flagged[other][0]]))
        carried = "没带" if occasion is None else f"带的是 {HEAVY_TEST_VARIABLE}={occasion}"
        below_text = f"，底下另有 {len(below)} 个同样没带前缀的重型进程（{', '.join(map(str, below[:6]))}）" if below else ""
        upper = []
        walker = table[pid]["parent"]
        while walker in table and walker > 1 and walker != root_pid and len(upper) < 3:
            upper.append(f"{walker} {command_for_display(table[walker]['command'])[:60]}")
            walker = table[walker]["parent"]
        stop_order = bottom_up(pid)
        shown = " ".join(map(str, stop_order[:HEAVY_TEST_STOP_ORDER_SHOWN]))
        if len(stop_order) > HEAVY_TEST_STOP_ORDER_SHOWN:
            shown += f" …（共 {len(stop_order)} 个，最后停 {pid}）"
        alerts.append((HEAVY_TEST_ALERT,
                       f"进程 {pid} 已跑 {format_duration(uptime_seconds - table[pid]['start_ticks'] / ticks_per_second)}，"
                       f"{'、'.join(categories)}（{'；'.join(test.detail for test in tests)}），{carried} {HEAVY_TEST_VARIABLE}=commit 或 =user-request"
                       f"{below_text}：{command_for_display(table[pid]['command'])[:140]}；归属：{owners.get(pid, '认不出')}；"
                       f"上层：{' ← '.join(upper) if upper else '直接挂在 Claude 实例底下'}",
                       f"主 agent 判断：是该跑的（提交流程里、用户要求的）只是前缀漏了，还是子 agent 越界、或绕过了执行前的闸（写进脚本、python、make 起的）；"
                       f"要停就从最底层往上逐个停整棵子进程，按这个次序逐个跑 `{PROCESS_STOP_COMMAND} <进程号>`：{shown}；"
                       f"或者给归属的那个子 agent 发消息让它自己停；判定接着跑就 --ack 进程:{pid}"))
    summary = (f"{HEAVY_TEST_ALERT}：查了这个 Claude 实例底下 {examined} 个进程（其中 bash -c 那一层 {shell_layers} 个不单判，"
               f"样本自检底下 {under_sample_selftest} 个不判，轻型三条底下 {under_light_tier} 个不判，登记成轻的自证底下 {under_registered_selftest} 个不判、"
               f"不在它底下而路径落在它临时目录里的 {in_selftest_temporary} 个不报），"
               f"认出重型 {len(flagged)} 个、带前缀的 {len(prefixed)} 个，报 {len(alerts)} 条")
    return [summary, *judgement_errors], alerts


def resident_memory_bytes(pid):
    """/proc/<pid>/statm 第二列（常驻页数）乘页大小；读不到（刚退出、权限不够）交 None。"""
    try:
        with open(f"/proc/{pid}/statm") as handle:
            return int(handle.read().split()[1]) * PAGE_SIZE_BYTES
    except (OSError, ValueError, IndexError):
        return None


MEMORY_PEAKS_LIBRARY = None   # (模块或 None, 导入失败的原因或 None)：第一次用时按文件路径导入，之后复用


def memory_peaks_library():
    """scripts/hosts/memory_peaks_lib.py（峰值表、取键与包装状态目录的唯一读法）：返回 (模块, None)，导入失败返回 (None, 原因)。"""
    global MEMORY_PEAKS_LIBRARY
    if MEMORY_PEAKS_LIBRARY is None:
        try:
            spec = importlib.util.spec_from_file_location("memory_peaks_lib_for_agent_watch", MEMORY_PEAKS_LIBRARY_PATH)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            MEMORY_PEAKS_LIBRARY = (module, None)
        except Exception as error:   # 文件不在、语法错都算：峰值表那一条不查，报告里写明，不让看门狗整个退出
            MEMORY_PEAKS_LIBRARY = (None, f"导入 {MEMORY_PEAKS_LIBRARY_PATH} 失败（{type(error).__name__}：{error}）")
    return MEMORY_PEAKS_LIBRARY


def memory_wrapper_of(pid):
    """进程所在内存包装的 scope（文件头「进程常驻内存过线」①）：交 (单元名, 上限的写法, 上限字节)；不在包装里或读不到交 None。
    上限先读包装账上那一笔的 cap，账读不到再读那个 scope 的 memory.max。"""
    try:
        with open(f"/proc/{pid}/cgroup", encoding="utf-8") as handle:
            cgroup_lines = handle.read().splitlines()
    except OSError:
        return None
    for line in cgroup_lines:
        path = line.split(":", 2)[-1]
        match = None
        for match in MEMORY_WRAPPER_SCOPE.finditer(path):
            pass
        if match is None:
            continue
        unit = match.group(1)
        library, _ = memory_peaks_library()
        if library is not None:
            ledger = os.path.join(library.default_state_directory(os.environ), "ledger", unit + ".json")
            try:
                with open(ledger, encoding="utf-8") as handle:
                    cap_text = str(json.load(handle).get("cap") or "")
            except (OSError, ValueError, AttributeError):
                cap_text = ""
            if memory_cap_bytes(cap_text):
                return unit, cap_text, memory_cap_bytes(cap_text)
        try:
            with open(os.path.join(CGROUP_ROOT + path[:match.end()], "memory.max"), encoding="utf-8") as handle:
                limit = handle.read().strip()
        except OSError:
            return None
        return (unit, f"{int(limit)} 字节（scope 的 memory.max）", int(limit)) if limit.isdigit() else None
    return None


def recorded_peak_of_process(pid):
    """峰值表里这个进程的命令（键照 memory_peaks_lib.py 的 command_key 由它的命令行算）近几次实测里最大的峰值字节；表里没有、读不到交 None。"""
    library, _ = memory_peaks_library()
    arguments = process_arguments(pid)
    if library is None or not arguments:
        return None
    try:
        peak, _ = library.recorded_peak(library.default_peak_table(), library.command_key(arguments))
    except (OSError, ValueError):
        return None
    return peak


def memory_threshold(pid, root_wrapper, thresholds, fixed_bytes, wrapper_of, peak_of):
    """这个进程的常驻内存阈值（文件头「进程常驻内存过线」①②③，先到先用）：交 (阈值字节, 阈值怎么写, 来源 wrapper / peak / peakfloor / fixed)。
    peakfloor 是峰值表里有、乘 CAP_FACTOR 不到下限而取下限的（下限是 --process-rss-floor-mebibytes 与 fixed_bytes 里小的那个）。
    wrapper_of、peak_of 平常是 memory_wrapper_of、recorded_peak_of_process，自证换成造出来的。"""
    if BROKEN_DETECTION != "memoryfixedline":
        wrapper = wrapper_of(pid)
        if wrapper is not None and (root_wrapper is None or wrapper[0] != root_wrapper[0]):
            unit, cap_text, cap_bytes = wrapper
            percent = getattr(thresholds, "process_rss_cap_percent", MEMORY_WRAPPER_CAP_PERCENT_DEFAULT)
            limit = cap_bytes * percent / 100
            return limit, f"{limit / BYTES_PER_GIBIBYTE:.2f} GiB，内存包装 {unit} 登记的上限 {cap_text} 的 {percent:g}%", "wrapper"
        peak = peak_of(pid)
        if peak:
            library, _ = memory_peaks_library()
            factor = getattr(library, "CAP_FACTOR", 1.5)
            limit = peak * factor
            floor_mebibytes = getattr(thresholds, "process_rss_floor_mebibytes", MEMORY_PEAK_FLOOR_MEBIBYTES_DEFAULT)
            floor = min(floor_mebibytes * BYTES_PER_MEBIBYTE, fixed_bytes)
            if limit < floor and BROKEN_DETECTION == "memorypeakfloorfixed":
                return fixed_bytes, f"{thresholds.process_rss_gibibytes:g} GiB，--process-rss-gibibytes", "fixed"
            if limit < floor and BROKEN_DETECTION != "memorypeakfloor":
                return floor, (f"{floor / BYTES_PER_GIBIBYTE:.2f} GiB，峰值表里这条命令的峰值 {peak / BYTES_PER_GIBIBYTE:.2f} GiB 的 {factor:g} 倍不到下限，"
                               f"取下限（--process-rss-floor-mebibytes {floor_mebibytes:g} 与 --process-rss-gibibytes 里小的那个）"), "peakfloor"
            return limit, f"{limit / BYTES_PER_GIBIBYTE:.2f} GiB，峰值表里这条命令的峰值 {peak / BYTES_PER_GIBIBYTE:.2f} GiB 的 {factor:g} 倍", "peak"
    return fixed_bytes, f"{thresholds.process_rss_gibibytes:g} GiB，--process-rss-gibibytes", "fixed"


def memory_selftest_owner(library, pid, table, root_pid, judged):
    """pid 的上层（到看门狗的根进程为止，不含根进程与它自己）里最近的登记成轻的自证（registered_light_selftest，与「重型测试没带前缀」同一份认法）：
    交它的进程号，没有交 None。judged 记这一次整查里判过的上层（进程号 → 是不是）。弄坏开关 memoryselftestanyweight 下参数里带 --selftest 的一律算。"""
    walker, seen = table.get(pid, {}).get("parent"), set()
    while walker and walker != root_pid and walker in table and walker not in seen:
        seen.add(walker)
        if walker not in judged:
            arguments = process_arguments(walker)
            if not arguments:
                judged[walker] = False
            elif BROKEN_DETECTION == "memoryselftestanyweight":
                judged[walker] = "--selftest" in arguments
            else:
                try:
                    judged[walker] = bool(registered_light_selftest(library, arguments, os.readlink(f"/proc/{walker}/cwd"), process_environment(walker)))
                except OSError:
                    judged[walker] = False   # 刚退出、权限不够：不当自证
        if judged[walker]:
            return walker
        walker = table[walker]["parent"]
    return None


def memory_report(root_pid, excluded_pids, session_dirs, thresholds, attribute_owners=True, wrapper_of=None, peak_of=None):
    """这个 Claude 实例底下常驻内存超过它的阈值的进程（判法见文件头）：返回 (报告行, [(告警名, 说明, 下一步)])。
    attribute_owners 为 False 时不去认归属（两次整查之间的单查只要知道有没有过线，归属留给随后那次整查）。
    wrapper_of、peak_of 只给自证换掉阈值的两处来源（memory_threshold）。"""
    threshold_gibibytes = thresholds.process_rss_gibibytes
    if threshold_gibibytes <= 0:
        return [f"{MEMORY_ALERT}：--process-rss-gibibytes 是 0，这一类不查"], []
    if BROKEN_DETECTION == "memoryscan":
        return [], []
    threshold_bytes = threshold_gibibytes * (1024 ** 2 if BROKEN_DETECTION == "memoryunit" else BYTES_PER_GIBIBYTE)
    wrapper_of = wrapper_of or memory_wrapper_of
    peak_of = peak_of or recorded_peak_of_process
    root_wrapper = wrapper_of(root_pid)
    sources = {"wrapper": 0, "peak": 0, "peakfloor": 0, "fixed": 0}
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    examined, largest, over = 0, None, []   # over：[(进程号, 常驻字节, 阈值怎么写)]
    frontier = [root_pid]
    while frontier:
        for child in sorted(children.get(frontier.pop(), [])):
            arguments = process_arguments(child)
            if arguments and is_watchdog_process(arguments):
                continue   # 别的看门狗与它的自检：整棵不看
            frontier.append(child)
            if child in excluded_pids or table[child]["state"] in EXITED_PROCESS_STATES:
                continue
            resident = resident_memory_bytes(child)
            if resident is None:
                continue
            examined += 1
            if largest is None or resident > largest[1]:
                largest = (child, resident)
            limit, limit_text, source = memory_threshold(child, root_wrapper, thresholds, threshold_bytes, wrapper_of, peak_of)
            sources[source] += 1
            if resident > limit:
                over.append((child, resident, limit_text))
    # 登记成轻的自证底下的过线进程不报（文件头「进程常驻内存过线」末段）：只对过线的往上找，单查也照这一条
    under_selftest, judgement_notes = 0, []
    heavy_library, heavy_problem = heavy_tests_library()
    if heavy_library is None:
        judgement_notes.append(f"；登记成轻的自证认不出、照报：{heavy_problem}")
    elif over and BROKEN_DETECTION != "memoryselftest":
        judged, kept = {}, []
        for pid, resident, limit_text in over:
            try:
                owner = memory_selftest_owner(heavy_library, pid, table, root_pid, judged)
            except Exception as error:   # 共用判定自己的 bug：写进报告，这个进程照报
                judgement_notes.append(f"；进程 {pid} 没判成上层有没有登记成轻的自证（{type(error).__name__}：{error}），照报")
                owner = None
            if owner is None:
                kept.append((pid, resident, limit_text))
            else:
                under_selftest += 1
        over = kept
    owners = owners_by_background_output([pid for pid, _, _ in over], table, session_dirs, root_pid) if (over and attribute_owners) else {}
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    alerts = []
    for pid, resident, limit_text in over:
        alerts.append((MEMORY_ALERT,
                       f"进程 {pid} 常驻内存 {resident / BYTES_PER_GIBIBYTE:.2f} GiB（阈值 {limit_text}），"
                       f"已跑 {format_duration(uptime_seconds - table[pid]['start_ticks'] / ticks_per_second)}：{command_for_display(table[pid]['command'])[:140]}；"
                       f"归属：{owners.get(pid, '认不出') if attribute_owners else '（单查不认归属）'}",
                       f"先看它还涨不涨（隔几秒再读一次 `grep VmRSS /proc/{pid}/status`）：还在涨多半是无界分配，按进程号停它（`{PROCESS_STOP_COMMAND} {pid}`）"
                       f"或给归属的那个子 agent 发消息让它停；查清为什么涨之前别照原样重派同一件活，重派时让那一步经 scripts/hosts/run-with-memory-cap.sh 带上限跑；"
                       f"确实要用这么多就 --ack 进程:{pid}"))
    largest_text = (f"常驻内存最大的是进程 {largest[0]} {largest[1] / BYTES_PER_GIBIBYTE:.2f} GiB" if largest else "一个都没读到")
    peaks_problem = memory_peaks_library()[1] if BROKEN_DETECTION != "memoryfixedline" else None
    summary = (f"{MEMORY_ALERT}：查了这个 Claude 实例底下 {examined} 个进程，{largest_text}（阈值照内存包装登记的上限 {sources['wrapper']} 个、"
               f"照峰值表 {sources['peak'] + sources['peakfloor']} 个（其中峰值乘出来不到下限、取下限的 {sources['peakfloor']} 个）、"
               f"照 --process-rss-gibibytes {threshold_gibibytes:g} GiB {sources['fixed']} 个"
               f"{'；峰值表读不了：' + peaks_problem if peaks_problem else ''}），登记成轻的自证底下过线的 {under_selftest} 个不报"
               f"{''.join(judgement_notes)}，报 {len(alerts)} 条")
    return [summary], alerts


def memory_alarm_between_checks(process_root_pid, excluded_pids, arguments):
    """两次整查之间单查一遍常驻内存：有没被 --ack 确认过的过线进程就返回 True，调用方马上整查一次、叫醒主 agent。"""
    if process_root_pid is None or BROKEN_DETECTION == "memorypoll":
        return False
    _, found = memory_report(process_root_pid, excluded_pids, set(), arguments, attribute_owners=False)
    remaining, _ = split_acknowledged([("进程", name, explanation, next_step) for name, explanation, next_step in found],
                                      set(arguments.ack or []))
    return bool(remaining)


CORE_USAGE_LIBRARY = None   # (模块或 None, 导入失败的原因或 None)：第一次用时按文件路径导入，之后复用


def core_usage_library():
    """scripts/hosts/core_usage.py：返回 (模块, None)，导入失败返回 (None, 原因)。"""
    global CORE_USAGE_LIBRARY
    if CORE_USAGE_LIBRARY is None:
        try:
            spec = importlib.util.spec_from_file_location("core_usage", CORE_USAGE_LIBRARY_PATH)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            CORE_USAGE_LIBRARY = (module, None)
        except Exception as error:   # 文件不在、语法错都算：报告里写明这一类没查，不让看门狗整个退出
            CORE_USAGE_LIBRARY = (None, f"导入 {CORE_USAGE_LIBRARY_PATH} 失败（{type(error).__name__}：{error}）")
    return CORE_USAGE_LIBRARY


def is_heavy_work(library, arguments, directory):
    """重活：lib_heavy_tests.py 的 classify_process 认出的重型用法（提交时才跑的检查那几类不算），或 runs_compiled_code 认出的跑编译出来的代码
    （cargo test / run / bench、测试二进制与编出来的装置）——与 .claude/hooks/host-resources-reminder.sh 认「重活」同一个口径。"""
    tests = [test for test in library.classify_process(arguments, directory)
             if test.kind not in getattr(library, "COMMIT_CHECK_KINDS", frozenset())]
    return bool(tests) or library.runs_compiled_code(arguments, directory) is not None


def aggregated_parallel_stage(library, arguments, directory):
    """进程是 AGGREGATED_PARALLEL_STAGES 里的一道门禁（命令的前三个词里有一个是 .claude/gate.d/ 顶层的那份脚本：`bash <脚本> …` 与直接执行都认）、
    或 AGGREGATED_PARALLEL_SCRIPTS 里的一份脚本（前三个词里有一个照写的或按工作目录解开的路径以它收尾）时交回它的名字，不是交 None。
    弄坏开关 mutationrows 下一律交 None（改前：往下拆到各干活进程逐个判）。"""
    if BROKEN_DETECTION == "mutationrows":
        return None
    for word in arguments[:3]:
        stage = library.gate_stage_name(word, directory)
        if stage in AGGREGATED_PARALLEL_STAGES:
            return stage
        for candidate in (os.path.normpath(word), os.path.normpath(os.path.join(directory, word))):
            for script in AGGREGATED_PARALLEL_SCRIPTS:
                if candidate == script or candidate.endswith("/" + script):
                    return script
    return None


def serial_declarations_of_tree(pid, children):
    """这棵树里第一个设了 SINGLEFS_SERIAL_DECLARATIONS 的进程给的路径（起它的脚本 export 的），没有交 None。"""
    stack = [pid]
    while stack:
        current = stack.pop()
        try:
            path = process_environment(current).get("SINGLEFS_SERIAL_DECLARATIONS")
        except OSError:
            path = None
        if path:
            return path
        stack.extend(children.get(current, []))
    return None


def memory_outcome_alert(record):
    """内存包装收尾时写的那三种（run-with-memory-cap.sh 文件头「峰值表」一段）：交 (告警名, 说明)；别的记录交 None。
    弄坏开关 memorygrew、memoryoutcome 下对应的那几种交 None（照核数过少那一种读：包装退了就不报）。"""
    kind = record.get("kind")
    name = MEMORY_OUTCOME_ALERTS.get(kind)
    if name is None or (kind == "memory-peak-grew" and BROKEN_DETECTION == "memorygrew") \
            or (kind != "memory-peak-grew" and BROKEN_DETECTION == "memoryoutcome"):
        return None

    def size(value):
        return f"{value / 1024 ** 2:.0f} MiB" if isinstance(value, int) else "没记"
    numbers = (f"键「{str(record.get('key'))[:120]}」，上限 {record.get('cap')}、记账 {size(record.get('booked'))}（{record.get('booked_source') or '没记账'}）、"
               f"实测 {size(record.get('measured'))}")
    head = f"进程 {record.get('alert_pid')} {record.get('source')}（{record.get('label')}）："
    if kind == "memory-peak-grew":
        ratio = record.get("ratio")
        ratio_text = f"{ratio:.1f}" if isinstance(ratio, (int, float)) else "?"
        return name, f"{head}内存峰值比记账大了 {ratio_text} 倍：工作量可能变了（{numbers}）"
    admitted = "按峰值表放进来的" if record.get("by_peak_table") else "不是按峰值表放进来的"
    if kind == "memory-own-cap":
        return name, f"{head}撞了自己的上限：上限给小了，或工作量变了（{numbers}，{admitted}）"
    return name, f"{head}被总上限挤掉：结果不算数，重跑（{numbers}，{admitted}）"


def memory_cap_bytes(text):
    """内存包装的上限写法（正整数加 K / M / G / T）→ 字节；认不出交 None。"""
    match = MEMORY_CAP_TEXT.match(str(text or ""))
    return int(match.group(1)) * 1024 ** MEMORY_CAP_POWERS[match.group(2)] if match else None


def memory_peak_below_alert_floor(record, cap_percent):
    """「内存峰值比记账大」那一种、实测不到上限的 cap_percent%：交一句「实测 X 不到上限 Y 的 P%」，只记一行、不叫醒（峰值表包装自己已记下这一次）。
    要叫醒的交 None：别的种类、门槛写 0、上限或实测读不出、实测到了门槛。弄坏开关 memorygrewfloor 下一律交 None（改前：涨了就叫醒，离上限多远都一样）。"""
    if record.get("kind") != "memory-peak-grew" or not cap_percent or BROKEN_DETECTION == "memorygrewfloor":
        return None
    cap, measured = memory_cap_bytes(record.get("cap")), record.get("measured")
    if cap is None or not isinstance(measured, int) or measured * 100 >= cap * cap_percent:
        return None
    return f"实测 {measured / 1024 ** 2:.0f} MiB 不到上限 {record.get('cap')} 的 {cap_percent:g}%"


def core_usage_next_step(pid):
    return (f"装置里互相独立的单位（取样点、轮次、不计时的准备）该分到登记的线程数上却在串行：停下它（`{PROCESS_STOP_COMMAND} {pid}`，"
            f"或给归属的子 agent 发消息让它停），照 .claude/rules/test-script-discipline.md「编译与算法都要快」改装置再跑；"
            f"真要串行的段由程序当场往 $SINGLEFS_SERIAL_DECLARATIONS 写「<秒>\\t此刻串行：<理由>」（写法在 scripts/hosts/core_usage.py 文件头「声明」），"
            f"核过理由成立就 --ack 进程:{pid} 接着跑")


def core_usage_report(root_pid, excluded_pids, session_ids, samples_root=None, memory_peak_alert_cap_percent=MEMORY_PEAK_ALERT_CAP_PERCENT_DEFAULT):
    """「长跑在用核数过少」（判法见文件头）：这个 Claude 实例底下的重活进程每次整查都取一次样（存状态目录），跑过一个窗口的判；再读这些会话的告警记录
    （multi-host-run.sh --peer、detached-run.sh watch 写的）。samples_root 只给自证换掉状态目录；memory_peak_alert_cap_percent 是 --memory-peak-alert-cap-percent。
    返回 (报告行, [(告警名, 说明, 下一步)])。"""
    if BROKEN_DETECTION == "coreusage":
        return [], []
    core, problem = core_usage_library()
    if core is None:
        return [f"{CORE_USAGE_ALERT}：这一类没查——{problem}"], []
    try:
        registered = core.registered_threads()
    except Exception as error:   # 登记写坏、SINGLEFS_THREAD_CAP 不是正整数、读不到那份库
        return [f"{CORE_USAGE_ALERT}：取不了每台默认重活线程数（{error}），这一类没查"], []
    window, note = core.window_seconds()
    lines, alerts, shown = ([f"{CORE_USAGE_ALERT}：{note}"] if note else []), [], []
    examined = 0
    library, heavy_problem = heavy_tests_library()
    if library is None:
        lines.append(f"{CORE_USAGE_ALERT}：本机重活这一半没查——{heavy_problem}")
    else:
        table = process_table()
        children = {}
        for pid, info in table.items():
            children.setdefault(info["parent"], []).append(pid)
        samples_directory = os.path.join(samples_root or os.environ.get(CORE_USAGE_SAMPLES_ROOT_VARIABLE) or STATE_ROOT, CORE_USAGE_SAMPLES_DIRECTORY)
        try:
            for name in os.listdir(samples_directory):
                path = os.path.join(samples_directory, name)
                if time.time() - os.path.getmtime(path) > CORE_USAGE_SAMPLES_KEPT_SECONDS:
                    os.remove(path)
        except OSError:
            pass   # 目录还没建、刚被别的看门狗删掉：不碍这一次
        ticks_per_second = os.sysconf("SC_CLK_TCK")
        uptime_seconds = float(open("/proc/uptime").read().split()[0])
        frontier = [root_pid]
        while frontier:
            for child in sorted(children.get(frontier.pop(), [])):
                arguments = process_arguments(child)
                if arguments and is_watchdog_process(arguments):
                    continue   # 别的看门狗与它的自检：整棵不看
                info = table[child]
                heavy = aggregated_stage = False
                if arguments and child not in excluded_pids and info["state"] not in EXITED_PROCESS_STATES:
                    try:
                        directory = os.readlink(f"/proc/{child}/cwd")
                        aggregated_stage = aggregated_parallel_stage(library, arguments, directory)
                        heavy = bool(aggregated_stage) or is_heavy_work(library, arguments, directory)
                    except Exception:   # 刚退出、权限不够、共用判定自己的 bug：这一个不当重活
                        heavy = aggregated_stage = False
                if not heavy:
                    frontier.append(child)
                    continue
                # 一条链上最上面那个重活进程：整棵树算它的，不往下再找；每次整查都取样存下，跑过一个窗口才判
                sample = core.tree_sample(child)
                if sample is None:
                    continue
                series = os.path.join(samples_directory, f"{child}-{info['start_ticks']}.jsonl")
                samples = core.read_series(series) + [sample]
                try:
                    core.append_line(series, json.dumps(sample, ensure_ascii=False, separators=(",", ":")))
                except OSError:
                    pass   # 状态目录写不进：这一次照判，下一次没有这一份样本
                if uptime_seconds - info["start_ticks"] / ticks_per_second < window:
                    continue
                examined += 1
                entries, problems = core.read_declarations(serial_declarations_of_tree(child, children))
                verdict = core.judge_series(samples, entries, problems, registered, window)
                command = command_for_display(info["command"])[:140]
                subject_note = f"（门禁 {aggregated_stage} 这一趟，各干活进程合计）" if aggregated_stage else ""
                if verdict["status"] == "low":
                    alerts.append((CORE_USAGE_ALERT,
                                   f"进程 {child} 重活{subject_note}只用了 {verdict['cores']:.1f} 核（登记 {registered}）：最近 {verdict['span']:.0f} 秒平均、"
                                   f"同时在算的线程约 {verdict['parallel']:.1f} 条，都低于阈值 {verdict['threshold']:g}（登记的四分之一）"
                                   f"{core.declaration_note(verdict)}：{command}",
                                   core_usage_next_step(child)))
                else:
                    shown.append(core.verdict_lines(verdict, f"进程 {child} 重活{subject_note}")[0].strip() + f"：{command}")
    matched = 0
    if BROKEN_DETECTION != "corerecords":
        for record in core.read_alert_records(core.alerts_path()):
            alert_pid = record.get("alert_pid")
            memory_queue = record.get("kind") == MEMORY_QUEUE_RECORD_KIND and BROKEN_DETECTION != "memoryqueue"
            if memory_queue and record.get("phase") == "admitted":
                continue   # 排了队之后在本机起跑了：不再报
            alert_name = MEMORY_QUEUE_ALERT if memory_queue else CORE_USAGE_ALERT
            outcome_alert = memory_outcome_alert(record)
            if outcome_alert is not None:   # 内存包装收尾时写的三种：包装退了也报（新鲜窗口照 core_usage.py）
                below_floor = memory_peak_below_alert_floor(record, memory_peak_alert_cap_percent)
                if not record.get("session_id"):
                    lines.append(f"{outcome_alert[0]}（{record.get('source')} 写的告警记录没有会话号，只记不叫醒）：{record.get('line')}")
                elif below_floor and record.get("session_id") in session_ids:
                    lines.append(f"{outcome_alert[0]}（{below_floor}，只记进峰值表、不叫醒）：{outcome_alert[1]}")
                elif record.get("session_id") in session_ids and isinstance(alert_pid, int):
                    matched += 1
                    alerts.append((*outcome_alert, record.get("next_step") or core_usage_next_step(alert_pid)))
                continue
            if not record.get("session_id"):
                lines.append(f"{alert_name}（{record.get('source')} 写的告警记录没有会话号，只记不叫醒）：{record.get('line')}")
                continue
            # 内存包装退了 252（phase=refused）之后包装就不在了，照报；别的要告警进程还活着
            needs_live_process = not (memory_queue and record.get("phase") == "refused")
            if record.get("session_id") not in session_ids or not isinstance(alert_pid, int) or (needs_live_process and not os.path.exists(f"/proc/{alert_pid}")):
                continue
            matched += 1
            alerts.append((alert_name, f"进程 {alert_pid} {record.get('source')}（{record.get('label')}）：{record.get('line')}",
                           record.get("next_step") or core_usage_next_step(alert_pid)))
    lines.append(f"{CORE_USAGE_ALERT}：查了这个 Claude 实例底下跑过 {window} 秒的重活 {examined} 个，读到这个会话的告警记录 {matched} 条"
                 f"（multi-host-run.sh --peer、detached-run.sh watch、run-with-memory-cap.sh 排队时写的），报 {len(alerts)} 条（阈值：登记 {registered} 的四分之一）")
    lines.extend(f"  {line}" for line in shown[:CORE_USAGE_SHOWN])
    if len(shown) > CORE_USAGE_SHOWN:
        lines.append(f"  另 {len(shown) - CORE_USAGE_SHOWN} 个不报的没列")
    return lines, alerts


BASH_TOOL_WRAPPER_COMMAND = re.compile(r"&& eval '(.*)'(?: < /dev/null)? && pwd -P >\| \S+\s*$", re.S)


def command_for_display(command):
    """Claude 的 Bash 工具把命令包在「source 快照 && … && eval '<命令>' [< /dev/null] && pwd -P >| …」里：报告里只列 eval 里那一段
    （还原 eval 里转义过的单引号、换行并成空格），不然最上层每一行都是同一串包装。"""
    if BROKEN_DETECTION == "display":
        return command
    wrapped = BASH_TOOL_WRAPPER_COMMAND.search(command)
    shown = wrapped.group(1).replace("'\"'\"'", "'") if wrapped else command
    return " ".join(shown.split())


def mark_selftest_ready():
    """只给自证用：环境里有 AGENT_WATCH_SELFTEST_READY_FILE 时，看门狗第一次进两次整查之间的等待前建这个文件
    （这时已取完检出偏移、拿到单例锁、记下要盯的进程、做完第一次整查）。自证按它等看门狗起来，不按固定秒数等：
    负载高时 python 起得慢，固定等 1.5、2 秒的格会赶在看门狗起来之前动手。平时不设这个变量，什么都不做。"""
    path = os.environ.get(SELFTEST_READY_VARIABLE)
    if path and not os.path.exists(path):
        open(path, "a").close()


def processes_to_watch(root_pid, excluded_pids):
    """--processes-only 盯的进程：{进程号: 起始时刻}。这个实例底下整棵子树，不只叶子；
    看门狗自己那一支、COMMAND_MARKERS_NOT_WATCHED、已经退出的（僵尸）不算，它们的子孙照样往下找。"""
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    watched, frontier = {}, [root_pid]
    while frontier:
        for child in children.get(frontier.pop(), []):
            frontier.append(child)
            info = table[child]
            if (child in excluded_pids or info["state"] in EXITED_PROCESS_STATES
                    or any(marker in info["command"] for marker in COMMAND_MARKERS_NOT_WATCHED)):
                continue
            watched[child] = info["start_ticks"]
    return watched


def still_running(watched):
    """盯着的进程里还没退出的：{进程号: 进程表那一行}。进程号被别的进程接走（起始时刻对不上）算退出了，僵尸也算。"""
    table = process_table()
    running = {}
    for pid, start_ticks in watched.items():
        info = table.get(pid)
        if info is None or info["start_ticks"] != start_ticks:
            continue
        if info["state"] in EXITED_PROCESS_STATES and BROKEN_DETECTION != "zombie":
            continue
        running[pid] = info
    return running


def run_processes_only(arguments, process_root_pid, excluded_pids):
    """--processes-only：起的时候在跑的那些都退出了退 0，进程一节报了告警退 3，到定时回报退 4；report 只看一眼，有告警退 3、没有退 0。"""
    watched = processes_to_watch(process_root_pid, excluded_pids)
    report_deadline = time.time() + arguments.max_minutes * 60
    repeats_path = repeated_reports_path(arguments.session_dir, True)
    while True:
        running = still_running(watched)
        topmost_running_pids = sorted(pid for pid, info in running.items() if info["parent"] not in running)
        lines = [f"只盯进程：进程 {process_root_pid} 底下看门狗起的时候就在跑的 {len(watched)} 个进程，还没退出的 {len(running)} 个"
                 + ("，最上层的：" if topmost_running_pids else "")]
        lines += [f"  进程 {pid}：{command_for_display(running[pid]['command'])[:140]}" for pid in topmost_running_pids]
        process_lines, found = process_alerts(process_root_pid, excluded_pids, arguments)
        if process_lines:
            lines.append("这个 Claude 实例底下跑得久的进程：")
            lines.extend(process_lines)
        heavy_lines, heavy_found = heavy_test_report(process_root_pid, excluded_pids,
                                                     {os.path.normpath(arguments.session_dir)} if arguments.session_dir else set())
        lines.extend(heavy_lines)
        memory_lines, memory_found = memory_report(process_root_pid, excluded_pids,
                                                   {os.path.normpath(arguments.session_dir)} if arguments.session_dir else set(), arguments)
        lines.extend(memory_lines)
        core_lines, core_found = core_usage_report(process_root_pid, excluded_pids,
                                                   {os.path.basename(os.path.normpath(arguments.session_dir))} if arguments.session_dir else set(),
                                                   memory_peak_alert_cap_percent=getattr(arguments, "memory_peak_alert_cap_percent",
                                                                                         MEMORY_PEAK_ALERT_CAP_PERCENT_DEFAULT))
        lines.extend(core_lines)
        found = found + heavy_found + memory_found + core_found
        # 根分区余量（文件头「根分区余量过低」）：告警的主体是本机，不是哪个进程
        root_lines, root_found = root_free_report(arguments)
        lines.extend(root_lines)
        root_alerts = [(ROOT_FREE_SUBJECT, name, explanation, next_step) for name, explanation, next_step in root_found]
        if arguments.session_dir:
            leftover_lines, leftover_alerts, leftover_pending_until = leftover_background_report(
                {os.path.normpath(arguments.session_dir)}, datetime.now(timezone.utc), arguments)
            kept_lines, kept_alerts = kept_copy_report({os.path.normpath(arguments.session_dir)}, datetime.now(timezone.utc), arguments)
        else:
            leftover_lines, leftover_alerts, leftover_pending_until = (
                ["交回、被停或失败之后后台还在跑：没给 --session-dir，这一类没查（watch.sh --processes 从 CLAUDE_CODE_SESSION_ID 找会话目录传进来）"], [], None)
            kept_lines, kept_alerts = ["交回之后副本还在：没给 --session-dir，这一类没查"], []
        lines.extend(leftover_lines)
        lines.extend(kept_lines)
        alerts, acknowledged = split_acknowledged([("进程", name, explanation, next_step) for name, explanation, next_step in found]
                                                  + root_alerts + leftover_alerts + kept_alerts, set(arguments.ack or []))
        for subject, name, explanation, _ in acknowledged:
            lines.append(f"已确认接着盯（--ack {subject}:{name}）：{explanation}")
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            lines = terminate_unacknowledged_repeats(repeats_path, alerts, acknowledged, arguments.terminate_after_unacknowledged_reports) + lines
            print_report(lines, alerts)
            return 3
        # 只剩交回之后宽限里的后台进程时不退：宽限一过再查一次，免得它们刚好在退出之后才够格报
        if not running and BROKEN_DETECTION != "processexit" and (leftover_pending_until is None or BROKEN_DETECTION == "leftoverexit"):
            terminate_unacknowledged_repeats(repeats_path, [], [], arguments.terminate_after_unacknowledged_reports)
            print_report(lines, alerts)
            if watched:
                print(f"看门狗起的时候就在跑的 {len(watched)} 个进程都退出了。")
            else:
                print("看门狗起的时候这个 Claude 实例底下就没有要盯的进程（看门狗自己那一支、claude 本体、IDE、node 与别的看门狗不算）："
                      "长命令是已经跑完了，还是还没起？")
            return 0
        if time.time() >= report_deadline:
            terminate_unacknowledged_repeats(repeats_path, [], [], arguments.terminate_after_unacknowledged_reports)
            print_report(lines, alerts)
            print(f"定时回报：已经看了 {arguments.max_minutes:g} 分钟，没有告警，盯着的进程还有 {len(running)} 个没退出。")
            print("   → 接着盯就用 run_in_background 再起一个看门狗："
                  f"{'--processes-only' if BROKEN_DETECTION == 'restarthint' else 'bash scripts/agents/watch.sh --processes'}")
            return 4
        next_check_in = min(arguments.interval_seconds, report_deadline - time.time())
        if leftover_pending_until is not None:
            next_check_in = min(next_check_in, leftover_pending_until.timestamp() + 1 - time.time())
        # 两次整查之间每隔 --detection-poll-seconds 单查一遍常驻内存，过线就提前整查（整查那一次报告警、退 3）
        next_full_check = time.time() + max(0.0, next_check_in)
        mark_selftest_ready()
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(0.0, next_full_check - time.time())))
            if memory_alarm_between_checks(process_root_pid, excluded_pids, arguments):
                break


def build_report(agent_ids, session_dir, thresholds, watch_started, excluded_pids, process_root_pid):
    now = datetime.now(timezone.utc)
    lines, alerts, done_flags, active_ids, unstarted_ids = [], [], [], [], []
    # --discover 找出来一个没结束的都没有时就是没有要盯的，不退回「会话里最近动过的全部子 agent」
    if agent_ids or (getattr(thresholds, "discover", False) and BROKEN_DETECTION != "discoverfallback"):
        targets = [(agent_id, find_transcript(agent_id, session_dir)) for agent_id in agent_ids]
    else:
        paths = glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))
        recent = [path for path in paths if (moment := readable_mtime(path)) is not None
                  and time.time() - moment <= thresholds.active_minutes * 60]
        targets = [(os.path.basename(path)[len("agent-"):-len(".jsonl")], path) for path in sorted(recent)]
    live = live_sessions()
    any_handed_back = False
    stopped_midway = []   # 「中途停下没交回」那一节的行
    for agent_id, path in targets:
        if path is None:
            waited = time.time() - watch_started
            lines.append(f"子 agent {agent_id}：还没有会话记录（已等 {format_duration(waited)}）")
            done_flags.append(False)
            unstarted_ids.append(agent_id)
            if waited >= thresholds.not_started_minutes * 60:
                alerts.append((agent_id, "没启动", f"派发 {format_duration(waited)} 之后还没有会话记录", "查派发是否失败、agent id 是否抄对"))
            continue
        try:
            transcript = AgentTranscript(agent_id, path)
        except OSError:
            lines.append(f"子 agent {agent_id}：会话记录在检查期间已消失或不可读，下次重查")
            done_flags.append(False)
            active_ids.append(agent_id)
            continue
        stale_reason = None if transcript.is_done() else stale_session_reason(path, transcript.last_timestamp, session_dir, live)
        if stale_reason:
            # 重启前旧进程留下的记录：不当要盯的，不按「无动静」「工具调用过长」报，也不拦着看门狗在全部交回时退出
            done_flags.append(True)
            lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」状态=已失效：{stale_reason}；不报告警")
            continue
        done_flags.append(transcript.is_done())
        any_handed_back = any_handed_back or transcript.is_done()
        if not transcript.is_done():
            active_ids.append(agent_id)
        since_last = format_duration((now - transcript.last_timestamp).total_seconds()) if transcript.last_timestamp else "?"
        total = format_duration((transcript.last_timestamp - transcript.first_timestamp).total_seconds()) if transcript.first_timestamp else "?"
        lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」状态={transcript.state} 已跑 {total} "
                     f"最后动作 {since_last} 前 调用 {transcript.model_calls} 次 上下文 {transcript.last_context_tokens // 1000}k 缓存整份重写 {transcript.full_cache_rewrites} 次")
        if transcript.parent_transcript_missing and not transcript.is_done():
            lines.append(f"  主会话记录 {transcript.parent_transcript_path} 不在：被 TaskStop 停掉的认不出，被停只按会话记录里的打断判")
        if transcript.pending_tool is not None and not transcript.is_done():
            started, tool_name, detail = transcript.pending_tool
            lines.append(f"  正在跑 {tool_name}（{format_duration((now - started).total_seconds())}）：{' '.join(detail.split())[:200]}")
        if transcript.waiting_on_its_own_background_tasks():
            orphaned = transcript.orphaned_background_tasks()
            orphan_note = f"；{', '.join(orphaned)} 的输出文件已没有进程开着、通知不会再来，不算在等" if orphaned else ""
            holder_lines = background_task_processes(transcript)
            lines.append(f"  结束本轮在等自己的后台任务 {', '.join(transcript.effectively_live_background_tasks())}"
                         f"{'，开着它们输出文件的进程：' if holder_lines else '：没找到开着它们输出文件的进程'}{orphan_note}")
            lines.extend(holder_lines)
        for name, explanation, next_step in agent_alerts(transcript, now, thresholds):
            alerts.append((agent_id, name, explanation, next_step))
            if name == STOPPED_MIDWAY_ALERT:
                stopped_midway.append(f"  子 agent {agent_id} {transcript.agent_type}「{transcript.description}」：{explanation}；"
                                      f"重派：python3 scripts/agents/agent-handover.py --agent {agent_id} --out <它的草稿目录>/{agent_id}-handover.md")
    # 「中途停下没交回」另起一节（文件头同名一段）：被看的子 agent 里这一类几个，一个都没有也写 0 个
    if targets and BROKEN_DETECTION != "midwaymerged":
        lines.append(f"中途停下没交回的子 agent：被看的 {len(targets)} 个里 {len(stopped_midway)} 个"
                     f"{'（结束本轮、没交回、一个后台任务都没起过；先发消息续做，不回应的照下面的命令抽交接摘要重派）' if stopped_midway else ''}")
        lines.extend(stopped_midway)
    # 被看的子 agent 所在的会话：交回之后丢下的后台进程查这些会话里全部子 agent，重型测试的归属也在这些会话的记录里找
    session_dirs = {os.path.normpath(session_dir)} if session_dir else set()
    session_dirs |= {os.path.dirname(os.path.dirname(path)) for _, path in targets if path}
    if process_root_pid is not None:
        process_lines, found = process_alerts(process_root_pid, excluded_pids, thresholds)
        if process_lines:
            lines.append("这个 Claude 实例底下跑得久的进程：")
            lines.extend(process_lines)
        heavy_lines, heavy_found = heavy_test_report(process_root_pid, excluded_pids, session_dirs)
        lines.extend(heavy_lines)
        memory_lines, memory_found = memory_report(process_root_pid, excluded_pids, session_dirs, thresholds)
        lines.extend(memory_lines)
        core_lines, core_found = core_usage_report(process_root_pid, excluded_pids, {os.path.basename(directory) for directory in session_dirs},
                                                   memory_peak_alert_cap_percent=getattr(thresholds, "memory_peak_alert_cap_percent",
                                                                                         MEMORY_PEAK_ALERT_CAP_PERCENT_DEFAULT))
        lines.extend(core_lines)
        for name, explanation, next_step in found + heavy_found + memory_found + core_found:
            alerts.append(("进程", name, explanation, next_step))
    # 根分区余量（文件头「根分区余量过低」）：每次整查都读，不看进程一半开没开
    root_lines, root_found = root_free_report(thresholds)
    lines.extend(root_lines)
    alerts.extend((ROOT_FREE_SUBJECT, name, explanation, next_step) for name, explanation, next_step in root_found)
    leftover_pending_until = None
    if session_dirs:
        leftover_lines, leftover_alerts, leftover_pending_until = leftover_background_report(session_dirs, now, thresholds)
        lines.extend(leftover_lines)
        alerts.extend(leftover_alerts)
        kept_lines, kept_alerts = kept_copy_report(session_dirs, now, thresholds)
        lines.extend(kept_lines)
        alerts.extend(kept_alerts)
        if any_handed_back:
            # 有子 agent 交回了才查草稿根（交回时报）；只进报告，不叫醒
            lines.extend(scratch_hog_report(scratch_hog_roots({os.path.basename(directory) for directory in session_dirs}), now,
                                            getattr(thresholds, "scratch_idle_hours", 6)))
    else:
        lines.append("交回、被停或失败之后后台还在跑：被看的子 agent 都还没有会话记录，不知道是哪个会话，这一类没查")
        lines.append("交回之后副本还在：被看的子 agent 都还没有会话记录，不知道是哪个会话，这一类没查")
    return lines, alerts, bool(done_flags) and all(done_flags), (active_ids, unstarted_ids), leftover_pending_until


def session_id_of(transcript_path):
    return os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))


SCOPE_SEGMENT_KEYS = ("种类", "跑什么", "范围", "为什么")   # 与 .claude/hooks/lib_run_scope.py 的 SEGMENT_KEYS 同一组


def scope_declaration_line(entry):
    """范围声明钩子（.claude/hooks/run-scope-guard.sh）放行的一条在「范围声明」一节里的一行：谁、命令、四段（照记录里切开的 segments，没切开的写声明原文）。"""
    scope = entry.get("run_scope") if isinstance(entry.get("run_scope"), dict) else {}
    segments = scope.get("segments") if isinstance(scope.get("segments"), dict) else {}
    four = ("；".join(f"{key}={segments.get(key, '')}" for key in SCOPE_SEGMENT_KEYS) if segments
            else f"声明原文 {scope.get('declaration') or '（没有）'}")
    who = f"{entry.get('agent_type') or '?'}{' ' + entry['agent_id'] if entry.get('agent_id') else ''}"
    where = "" if scope.get("where") in (None, "命令文本") else f"（{scope['where']}）"
    return f"{who}：{(entry.get('command') or '')[:160]}{where}：{four}"


def scope_declaration_section(scope_declarations):
    """定时回报里的「范围声明」一节：放行记录一条一行，只列不叫醒；一条都没有时不出这一节。"""
    if not scope_declarations:
        return []
    return ([f"范围声明（run-scope-guard.sh 放行的 {len(scope_declarations)} 条，只列不叫醒；范围合不合适由主 agent 对着看）："]
            + [f"  {line}" for line in scope_declarations])


def read_detections(detections_path, start_offset, session_ids, watched_agent_ids=(), scope_declarations=None):
    """从 start_offset 往后读检出记录，返回 (新的偏移, [属于这些会话、被看的子 agent 或主 agent 的告警], [只进报告的记录])。
    范围声明钩子的放行记录（会话号在 scope_session_id、带 run_scope）不是检出：给了 scope_declarations 就把属于这些会话的逐条记成一行追加进去
    （scope_declaration_line），不叫醒；弄坏开关 scopeunlisted 下一条都不记。"""
    alerts, notes = [], []
    if not os.path.isfile(detections_path):
        return start_offset, alerts, notes
    position = start_offset
    with open(detections_path, "rb") as handle:
        handle.seek(start_offset)
        for raw_line in handle:
            position += len(raw_line)
            try:
                entry = json.loads(raw_line.decode("utf-8", "replace"))
            except ValueError:
                continue
            if not isinstance(entry, dict):
                continue
            if entry.get("run_scope") is not None and entry.get("scope_session_id") is not None:
                if (scope_declarations is not None and BROKEN_DETECTION != "scopeunlisted" and entry.get("scope_session_id") in session_ids
                        and not (watched_agent_ids and entry.get("agent_id") and entry.get("agent_id") not in watched_agent_ids)):
                    scope_declarations.append(scope_declaration_line(entry))
                continue
            if BROKEN_DETECTION == "detections" or entry.get("session_id") not in session_ids:
                continue
            if (watched_agent_ids and entry.get("agent_id") and entry.get("agent_id") not in watched_agent_ids
                    and BROKEN_DETECTION != "otheragent"):
                continue   # 同一会话里别的子 agent 的检出归盯它的那个看门狗（2026-09-19：盯攻方腿的看门狗被上游那个 agent 的检出叫醒）
            findings = [str(finding) for finding in (entry.get("findings") or [])]
            if findings and all(finding.startswith("没有 timeout 的等待循环") for finding in findings) and BROKEN_DETECTION != "loopnoise":
                # 只检出等待循环的不立刻叫醒：正常等编译、等全量跑也是这个形状。会话记录那一路在它跑满 --wait-loop-minutes 时再报。
                continue
            refused = any(finding.startswith("写被拒") for finding in findings)
            if entry.get("agent_type") == "主 agent" and refused and BROKEN_DETECTION != "ownrefusal":
                # 主 agent 自己的写被拒：拒绝信息已经在它的工具结果里，再叫醒一次只是打断（2026-09-19 一晚上被自己被拒的命令叫醒两次）。
                # 按模式找进程由上游钩子在执行前拒绝，不写检出记录，这里读不到它。
                notes.append(f"主 agent 自己的命令被 hook 拒了（不叫醒，拒绝信息在它的工具结果里）：{(entry.get('command') or '')[:120]}")
                continue
            if findings and all(finding.startswith(INFO_DETECTION_PREFIXES) for finding in findings) and BROKEN_DETECTION != "infowakes":
                notes.append(f"hook 检出（只记不叫醒）：{entry.get('agent_type') or '?'} {entry.get('agent_id') or ''}：{'、'.join(findings)}："
                             f"{(entry.get('command') or '')[:120]}")
                continue
            if findings and detection_outcome(entry) == DETECTION_REFUSED and BROKEN_DETECTION != "refusedwakes":
                # 钩子在执行前已经拒掉的命令（重型测试被拒、提交时才跑的检查被拒、不经内存包装被拒……）：没有执行，拒绝信息与出路在它的工具结果里，
                # 主 agent 醒了也没有可处置的；只进报告，不叫醒（「记录不处置」那一类。2026-10-10、2026-10-11 看门狗为它们被叫醒、重起了十来次）。
                notes.append(f"hook 检出（执行前已被 hook 拒了、没有执行，只记不叫醒）：{entry.get('agent_type') or '?'} {entry.get('agent_id') or ''}："
                             f"{'、'.join(findings)}：{(entry.get('command') or '')[:120]}")
                continue
            # 主体写 agent 号（有的话）：--ack <agent 号>:hook 检出 只压这一个子 agent 的检出；类型写进说明
            subject = (entry.get("agent_id") if BROKEN_DETECTION != "detectiontype" else None) or entry.get("agent_type") or "?"
            alerts.append((subject, "hook 检出",
                           f"{entry.get('agent_type') or '?'}：{'、'.join(entry.get('findings') or [])}：{(entry.get('command') or '')[:160]}",
                           detection_next_step(entry)))
    return position, alerts, notes


# 检出记录的结局（钩子写在 outcome 一栏）；没写这一栏的旧记录按发现的起头认：这几种起头是执行前拒掉的
DETECTION_REFUSED, DETECTION_RECORDED = "被拒", "只记"
REFUSED_FINDING_PREFIXES = ("写被拒", "重型测试被拒", "提交时才跑的检查被拒", "不经内存包装跑编译出来的代码被拒")


def detection_outcome(entry):
    """这一条检出的结局：「被拒」（执行前已拒、没执行）或「只记」（照常执行）。弄坏开关 outcomeignored 下退回只认「写被拒」的旧判法。"""
    findings = [str(finding) for finding in (entry.get("findings") or [])]
    if BROKEN_DETECTION == "outcomeignored":
        return DETECTION_REFUSED if any(finding.startswith("写被拒") for finding in findings) else DETECTION_RECORDED
    if entry.get("outcome") in (DETECTION_REFUSED, DETECTION_RECORDED):
        return entry["outcome"]
    return DETECTION_REFUSED if any(finding.startswith(REFUSED_FINDING_PREFIXES) for finding in findings) else DETECTION_RECORDED


def detection_next_step(entry):
    """hook 检出告警的下一步：写被拒、命令执行前被拒、只记照跑三种各说各的。"""
    if any(str(finding).startswith("写被拒") for finding in (entry.get("findings") or [])):
        return "这次写被 hook 拒了、没写进去；主 agent 判断是不是该写的，再决定发消息让它改、自己改、还是派该写的 agent"
    if detection_outcome(entry) == DETECTION_REFUSED:
        return ("这条命令执行前已被 hook 拒了、没有执行（拒绝信息与出路在它的工具结果里）；主 agent 判断它该不该跑，"
                "再决定发消息让它照出路改、交给该跑的一方、还是不管（主 agent 自己的，照拒绝信息最前面那条加好前缀的命令重跑或不跑）")
    return "检出 hook 只记不拦，命令照常在跑；主 agent 判断它会不会出问题，再决定接着盯、发消息让它改、还是处理"


def print_report(lines, alerts):
    now = datetime.now(timezone.utc)
    print(f"[agent-watch {now.astimezone(ZoneInfo('Asia/Tokyo')).strftime('%Y-%m-%d')}]")
    for line in lines:
        print(line)
    for owner, name, explanation, next_step in alerts:
        print(f"⚠️ {name}（{owner}）：{explanation}")
        print(f"   → 怎么办：{next_step}")
    if not alerts:
        print("没有告警。")


def restart_command(active_ids, unstarted_ids, discover):
    """出路句里照抄就能起的那条命令：主 agent 经 watch.sh 起看门狗，它不认 agent-watch.py 自己的 --agents / --processes-only。"""
    if BROKEN_DETECTION == "restarthint":
        return f"--agents {','.join(active_ids + unstarted_ids)}"
    if discover:
        return "bash scripts/agents/watch.sh"
    return f"bash scripts/agents/watch.sh {','.join(active_ids + unstarted_ids)}"


def print_active_list(active_ids, unstarted_ids, discover=False):
    if not active_ids and not unstarted_ids:
        return
    print(f"还没交回、没被停的子 agent（{len(active_ids)} 个，另有还没会话记录的 {len(unstarted_ids)} 个）：{','.join(active_ids + unstarted_ids)}")
    print(f"   → 接着盯就用 run_in_background 再起一个看门狗：{restart_command(active_ids, unstarted_ids, discover)}")


def own_process_chain(table, start_pid):
    chain, pid = set(), start_pid
    while pid in table and pid > 1:
        chain.add(pid)
        pid = table[pid]["parent"]
    return chain


def split_informational(alerts):
    """三类只记不叫醒的条件告警：从告警里摘出来，写成报告行；AGENT_WATCH_BREAK=infoalertwakes 时照旧叫醒（自检用）。"""
    if BROKEN_DETECTION == "infoalertwakes":
        return alerts, []
    remaining, informational = [], []
    for alert in alerts:
        (informational if alert[1] in INFO_ALERT_NAMES else remaining).append(alert)
    return remaining, [f"只记不叫醒：{name}（{subject}）：{explanation}" for subject, name, explanation, _ in informational]


def split_acknowledged(alerts, acknowledgements):
    """主 agent 用 --ack 确认过「只是慢、接着盯」的告警：只写进报告，不叫醒主 agent。
    2026-09-19：规程写「只是慢就再起一个看门狗接着盯」，而「工具调用过长」这类条件在那次调用结束之前一直成立，
    新起的看门狗当场又报同一句退出，等于盯不住；确认只在这一次看门狗里有效，下一次起不带就失效。"""
    if BROKEN_DETECTION == "ack" or not acknowledgements:
        return alerts, []
    remaining, acknowledged = [], []
    for alert in alerts:
        subject, name, explanation, _ = alert
        keys = {f"{subject}:{name}"}
        if subject == "进程" or (name == LEFTOVER_BACKGROUND_ALERT and BROKEN_DETECTION != "leftoverack"):
            process_id = re.match(r"进程 (\d+) ", explanation)
            if process_id:
                keys.add(f"进程:{process_id.group(1)}")
        # 「跑满 N 小时要主 agent 问一次」也认短写「<agent 号>:跑满 N 小时」：只确认那一格，下一格告警名不同，照报
        if name.startswith(ROUTINE_INQUIRY_ALERT_PREFIX) and name.endswith(ROUTINE_INQUIRY_ALERT_SUFFIX) and BROKEN_DETECTION != "inquiryack":
            keys.add(f"{subject}:{name[:-len(ROUTINE_INQUIRY_ALERT_SUFFIX)]}")
        (acknowledged if keys & acknowledgements else remaining).append(alert)
    return remaining, acknowledged


def session_state_directory(session_dir):
    return os.path.join(STATE_ROOT, os.path.basename(os.path.normpath(session_dir)))


def acquire_single_instance(state_dir):
    """拿这个会话的看门狗锁（flock，进程退出就放）：拿到返回打开着的锁文件，已有一个在盯返回 (None, 那一个的 pid 与起的时刻)。"""
    import fcntl
    os.makedirs(state_dir, exist_ok=True)
    handle = open(os.path.join(state_dir, "watch.lock"), "a+", encoding="utf-8")
    try:
        if BROKEN_DETECTION != "nosingleinstance":
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        handle.seek(0)
        holder = handle.read().strip()
        handle.close()
        return None, holder
    handle.seek(0)
    handle.truncate()
    handle.write(f"pid {os.getpid()} 起于 {datetime.now(ZoneInfo('Asia/Tokyo')).strftime('%Y-%m-%d')}")
    handle.flush()
    return handle, None


def read_acks(state_dir):
    path = os.path.join(state_dir, "watch-acks.tsv")
    if BROKEN_DETECTION == "acksnotpersisted" or not os.path.isfile(path):
        return set()
    return {line.rstrip("\n").split("\t", 1)[0] for line in open(path, encoding="utf-8") if line.strip()}


def write_acks(state_dir, acks):
    os.makedirs(state_dir, exist_ok=True)
    path = os.path.join(state_dir, "watch-acks.tsv")
    temporary = f"{path}.{os.getpid()}.{threading.get_ident()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        for key in sorted(acks):
            handle.write(f"{key}\t{datetime.now(timezone.utc).isoformat()}\n")
    os.replace(temporary, path)


ACK_UPDATE_PAUSE_SECONDS = 0.0   # 读与写之间停多久：只给 --selftest 拉开两只同时写的窗口用


def update_acks(state_dir, change):
    """watch-acks.tsv 的读改写：change(读到的确认集合) 交回要写下的集合。整段拿着同目录 watch-acks.lock 的 flock（进程退出就放）：
    两只看门狗同时写（在盯的那一只剪掉作废的确认、另起被拒的那一只记下新的 --ack），不加锁时后写的冲掉先写的，新给的 --ack 丢掉
    （records/2026-10-23-subagent拆分提案.md 第一百二十一节）。弄坏开关 acksunlocked 不拿锁（改前的样子）。"""
    import fcntl
    os.makedirs(state_dir, exist_ok=True)
    with open(os.path.join(state_dir, "watch-acks.lock"), "a", encoding="utf-8") as lock:
        if BROKEN_DETECTION != "acksunlocked":
            fcntl.flock(lock, fcntl.LOCK_EX)
        acks = change(read_acks(state_dir))
        if ACK_UPDATE_PAUSE_SECONDS:
            time.sleep(ACK_UPDATE_PAUSE_SECONDS)
        write_acks(state_dir, acks)
    return acks


def alert_keys(alert):
    """一条告警能被哪些 --ack 键确认（与 split_acknowledged 认的同一套）。"""
    subject, name, explanation, _ = alert
    keys = {f"{subject}:{name}"}
    process_id = re.match(r"进程 (\d+) ", explanation)
    if process_id and (subject == "进程" or name == LEFTOVER_BACKGROUND_ALERT):
        keys.add(f"进程:{process_id.group(1)}")
    if name.startswith(ROUTINE_INQUIRY_ALERT_PREFIX) and name.endswith(ROUTINE_INQUIRY_ALERT_SUFFIX):
        keys.add(f"{subject}:{name[:-len(ROUTINE_INQUIRY_ALERT_SUFFIX)]}")
    return keys


def pruned_acks(acks, alerts, unfinished_agent_ids):
    """条件类告警的确认在条件消失（这一次整查没有它）时作废；事件类（hook 检出）的确认到那个子 agent 结束才作废。"""
    present = set().union(*(alert_keys(alert) for alert in alerts)) if alerts else set()
    kept = set()
    for key in acks:
        subject, _, name = key.partition(":")
        is_event = any(name.startswith(event) for event in EVENT_ALERT_NAMES)
        if (is_event and subject in unfinished_agent_ids) or (not is_event and key in present) or BROKEN_DETECTION == "acksneverpruned":
            kept.add(key)
    return kept


def stuck_alert_key(alert):
    """「进程卡住」类告警按什么记连报：进程告警交 (进程:<pid>:<告警种类>, pid)，子 agent 的工具调用过长交 (<agent 号>:工具调用过长, None)；别的类交 None。"""
    subject, name, explanation, _ = alert
    if name not in STUCK_ALERT_NAMES:
        return None
    if subject == "进程":
        process_id = re.match(r"进程 (\d+) ", explanation)
        return (f"进程:{process_id.group(1)}:{name}", int(process_id.group(1))) if process_id else None
    return f"{subject}:{name}", None


def read_repeated_reports(path):
    """连报记录：{键: (连报次数, 进程起始时刻)}，子 agent 的告警起始时刻是 -；文件不在是空的。"""
    if BROKEN_DETECTION == "repeatsforgotten" or not os.path.isfile(path):
        return {}
    repeats = {}
    for line in open(path, encoding="utf-8"):
        fields = line.rstrip("\n").split("\t")
        if len(fields) == 3 and fields[1].isdigit():
            repeats[fields[0]] = (int(fields[1]), fields[2])
    return repeats


def write_repeated_reports(path, repeats):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    temporary = f"{path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        for key in sorted(repeats):
            count, start = repeats[key]
            handle.write(f"{key}\t{count}\t{start}\n")
    os.replace(temporary, path)


def stop_reported_process(pid):
    """经 proc.py stop 终止告警里点名的那一个进程（先 TERM、过宽限再 KILL，只停它、不带子进程）：交 (停成了没有, 它输出的末行)。"""
    try:
        completed = subprocess.run([sys.executable, PROCESS_STOP_SCRIPT, "stop", str(pid)], capture_output=True, text=True,
                                   timeout=PROCESS_STOP_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return False, f"起 {PROCESS_STOP_SCRIPT} 出错：{error!r}"
    output = (completed.stdout + completed.stderr).strip().splitlines()
    return completed.returncode == 0, (output[-1] if output else f"proc.py 退出码 {completed.returncode}")


def terminate_unacknowledged_repeats(path, alerts, acknowledged, threshold):
    """watch 每次叫醒主 agent 之前调（没有告警的退出也调，清空记录）：没被 --ack 的「进程卡住」类告警各记一次连报，这一次没再报的作废；
    同一个键连报到 threshold 次的进程告警经 proc.py stop 终止那个进程（一次至多终止一个），子 agent 的到线只报。
    path 是连报记录的路径，None 表示没给 --session-dir、没处记。threshold 为 0 不记、不终止。交回要放进报告最前面的几行。"""
    if threshold <= 0:
        if BROKEN_DETECTION != "terminatezero":
            return []
        threshold = 3
    counted = list(alerts) + (list(acknowledged) if BROKEN_DETECTION == "terminateacked" else [])
    stuck = [(alert, keyed) for alert in counted for keyed in [stuck_alert_key(alert)] if keyed is not None]
    if path is None:
        return (["连报次数：没给 --session-dir，没处记；「进程卡住」类告警连报到线也不终止（watch.sh 不带 agent 号起的看门狗才记）"]
                if stuck else [])
    previous = read_repeated_reports(path)
    current, lines, terminated = {}, [], False
    for (subject, name, _, _), (key, pid) in stuck:
        if key in current:
            continue
        start = "-" if pid is None else (process_start_ticks(pid) or "?")
        count_before, start_before = previous.get(key, (0, start))
        count = count_before + 1 if start_before == start else 1
        if count < threshold:
            current[key] = (count, start)
        elif pid is None:
            current[key] = (count, start)
            lines.append(f"{subject} 的「{name}」连报 {count} 次没处置：没有进程号，脚本停不了子 agent，只报；"
                         f"主 agent 处置（停它用停止它的工具），或判定接着等就 --ack {subject}:{name}")
        elif terminated or start == "?":
            current[key] = (count, start)
            lines.append(f"进程 {pid}（{name}）也连报 {count} 次没处置："
                         + ("这一次叫醒已终止了一个，下一次再判" if terminated else "读不到它的起始时刻，认不准还是不是同一个进程，不终止"))
        else:
            stopped, detail = stop_reported_process(pid)
            if stopped:
                terminated = True
                lines.append(f"终止了 {pid}（{name}，连报 {count} 次没处置）：{detail}")
            else:
                current[key] = (count, start)
                lines.append(f"想终止 {pid}（{name}，连报 {count} 次没处置）没成：{detail}；主 agent 照告警里的出路处置")
    write_repeated_reports(path, current)
    return lines


def repeated_reports_path(session_dir, processes_only):
    """连报记录放在会话的状态目录里；没给 --session-dir 是 None（没处记）。"""
    if not session_dir:
        return None
    return os.path.join(session_state_directory(session_dir), REPEATED_REPORTS_PROCESSES_FILE if processes_only else REPEATED_REPORTS_FILE)


def discovered_agent_ids(session_dir, state_dir, cache, thresholds):
    """这个会话里还没交回、没被停、没失败的子 agent：会话记录 --active-minutes 以内动过的，加上此前的看门狗见过、还没结束的
    （记在状态目录的 watch-agents.tsv，等后台任务等得久、会话记录好久没动的也接得上）。cache：路径 → (大小, 修改时刻, 结束了没有)。"""
    if BROKEN_DETECTION == "discovernone":
        return []
    agents_path = os.path.join(state_dir, "watch-agents.tsv")
    remembered = {line.strip() for line in open(agents_path, encoding="utf-8")} if os.path.isfile(agents_path) else set()
    notifications = session_task_events(os.path.normpath(session_dir) + ".jsonl")[1]
    live = live_sessions()
    unfinished = []
    for path in sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))):
        agent_id = os.path.basename(path)[len("agent-"):-len(".jsonl")]
        status = os.stat(path)
        key = (status.st_size, status.st_mtime)
        known = cache.get(path)
        if known is not None and known[:2] == key and known[2]:
            continue
        if known is None and agent_id not in remembered and time.time() - status.st_mtime > thresholds.active_minutes * 60:
            continue
        transcript = AgentTranscript(agent_id, path)
        ended = (transcript.is_done() or transcript.ended(notifications) is not None
                 or stale_session_reason(path, transcript.last_timestamp, session_dir, live) is not None)   # 重启前旧进程留下的记录不再当要盯的
        cache[path] = (key[0], key[1], ended)
        if not ended:
            unfinished.append(agent_id)
    os.makedirs(state_dir, exist_ok=True)
    temporary = f"{agents_path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{agent_id}\n" for agent_id in unfinished))
    os.replace(temporary, agents_path)
    return unfinished


def run(arguments):
    if arguments.processes_only and (arguments.agents or arguments.mode == "cost" or arguments.process_root_pid == 0):
        print("✗ --processes-only 只盯进程，不和 --agents、cost、--process-root-pid 0 一起用\n"
              "→ 怎么办：盯子 agent 就去掉 --processes-only（子 agent 起的进程照样在进程一节里）；只盯主 agent 自己放后台的长命令就只给 --processes-only"
              "（可带 --session-dir，同时查交回之后后台还在跑）",
              file=sys.stderr)
        return 2
    if not arguments.agents and not arguments.session_dir and not arguments.processes_only:
        print("✗ 要给 --agents 或 --session-dir\n→ 怎么办：派发返回的 agent id 用逗号连起来传 --agents；只盯主 agent 自己放后台的长命令用 --processes-only",
              file=sys.stderr)
        return 2
    if getattr(arguments, "discover", False) and (not arguments.session_dir or arguments.agents or arguments.processes_only or arguments.mode != "watch"):
        print("✗ --discover 只和 watch、--session-dir 一起用，不给 --agents、--processes-only\n"
              "→ 怎么办：bash scripts/agents/watch.sh 不带 agent 号（它从 CLAUDE_CODE_SESSION_ID 找会话目录）", file=sys.stderr)
        return 2
    agent_ids = [item for item in (arguments.agents or "").split(",") if item]
    discover = bool(getattr(arguments, "discover", False))
    state_dir = session_state_directory(arguments.session_dir) if discover else None
    if discover:
        lock_handle, holder = acquire_single_instance(state_dir)
        if lock_handle is None:
            update_acks(state_dir, lambda acks: acks | set(arguments.ack or []))
            print(f"已有一个看门狗在盯这个会话（{holder}），这一次不另起"
                  f"{'；--ack 已记进 ' + os.path.join(state_dir, 'watch-acks.tsv') + '，在盯的那一个下一次检查时读到' if arguments.ack else ''}。")
            return SINGLE_INSTANCE_EXIT
        update_acks(state_dir, lambda acks: acks | set(arguments.ack or []))
    discovery_cache = {}
    table = process_table()
    if arguments.process_root_pid == 0:
        process_root_pid = None
    elif arguments.process_root_pid is not None:
        process_root_pid = arguments.process_root_pid
    else:
        process_root_pid = claude_instance_pid(table, os.getpid())
    excluded_pids = own_process_chain(table, os.getpid())
    if arguments.processes_only:
        if process_root_pid is None:
            print("✗ 往上找不到跑这个看门狗的 Claude 实例，--processes-only 没有进程可盯\n"
                  "→ 怎么办：在 Claude 的 Bash 工具里起它，或者用 --process-root-pid 指定从哪个进程往下看", file=sys.stderr)
            return 2
        return run_processes_only(arguments, process_root_pid, excluded_pids)
    if arguments.mode == "cost":
        lines, _ = cost_report(agent_ids, arguments.session_dir, arguments)
        print("\n".join(lines))
        return 0
    watch_started = time.time()
    detections_path = arguments.detections_file
    detections_offset = os.path.getsize(detections_path) if (arguments.mode == "watch" and os.path.isfile(detections_path)) else 0
    report_deadline = watch_started + arguments.max_minutes * 60
    detection_notes = []
    scope_declarations = []   # 范围声明钩子放行的（这一个看门狗起来之后读到的），定时回报里列成「范围声明」一节
    repeats_path = repeated_reports_path(arguments.session_dir, False)
    def acknowledgements():
        return set(arguments.ack or []) | (read_acks(state_dir) if discover else set())
    while True:
        if discover:
            agent_ids = discovered_agent_ids(arguments.session_dir, state_dir, discovery_cache, arguments)
        lines, alerts, all_done, active_lists, leftover_pending_until = build_report(
            agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
        watched_transcripts = [path for path in (find_transcript(agent_id, arguments.session_dir) for agent_id in agent_ids) if path]
        session_ids = {session_id_of(path) for path in watched_transcripts}
        if arguments.session_dir:
            session_ids.add(os.path.basename(os.path.normpath(arguments.session_dir)))
        detections_offset, detection_alerts, new_notes = read_detections(
            detections_path, detections_offset, session_ids, set(agent_ids), scope_declarations)
        detection_notes += new_notes
        lines += detection_notes
        lines += scope_declaration_section(scope_declarations)
        alerts += detection_alerts
        if discover:
            update_acks(state_dir, lambda acks: pruned_acks(set(arguments.ack or []) | acks, alerts, set(agent_ids)))
        alerts, informational_lines = split_informational(alerts)
        lines += informational_lines
        alerts, acknowledged = split_acknowledged(alerts, acknowledgements())
        for subject, name, explanation, _ in acknowledged:
            lines.append(f"已确认接着盯（--ack {subject}:{name}）：{explanation}")
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            lines = terminate_unacknowledged_repeats(repeats_path, alerts, acknowledged, arguments.terminate_after_unacknowledged_reports) + lines
            print_report(lines, alerts)
            print_active_list(*active_lists, discover)
            return 3
        # 全部交回照旧退出，这一次 build_report 已经把「交回之后后台还在跑」查过了；只剩宽限里的，等宽限过了再查一次才退
        if all_done and (leftover_pending_until is None or BROKEN_DETECTION == "leftoverexit"):
            terminate_unacknowledged_repeats(repeats_path, [], [], arguments.terminate_after_unacknowledged_reports)
            print_report(lines, alerts)
            print("被看的子 agent 全部交回或被停。")
            return 0
        if time.time() >= report_deadline:
            terminate_unacknowledged_repeats(repeats_path, [], [], arguments.terminate_after_unacknowledged_reports)
            print_report(lines, alerts)
            print(f"定时回报：已经看了 {arguments.max_minutes:g} 分钟，没有告警。")
            print_active_list(*active_lists, discover)
            return 4
        next_full_check = time.time() + arguments.interval_seconds
        if BROKEN_DETECTION != "deadline":
            next_full_check = min(next_full_check, report_deadline)
        if all_done:
            next_full_check = min(next_full_check, leftover_pending_until.timestamp() + 1)
        mark_selftest_ready()
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(next_full_check - time.time(), 0)))
            detections_offset, detection_alerts, new_notes = read_detections(
                detections_path, detections_offset, session_ids, set(agent_ids), scope_declarations)
            detection_notes += new_notes
            # 常驻内存过线等不到下一次整查：单查到了就同检出一样马上整查一次（整查的报告里带着内存一节），有没确认的告警就退 3
            if detection_alerts or memory_alarm_between_checks(process_root_pid, excluded_pids, arguments):
                lines, alerts, _, active_lists, _ = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
                alerts, informational_lines = split_informational(alerts)
                lines += informational_lines
                alerts, acknowledged = split_acknowledged(alerts + detection_alerts, acknowledgements())
                if alerts:
                    lines = (terminate_unacknowledged_repeats(repeats_path, alerts, acknowledged, arguments.terminate_after_unacknowledged_reports)
                             + lines + detection_notes + scope_declaration_section(scope_declarations)
                             + [f"已确认接着盯（--ack {subject}:{name}）：{explanation}"
                                                          for subject, name, explanation, _ in acknowledged])
                    print_report(lines, alerts)
                    print_active_list(*active_lists, discover)
                    return 3


def write_transcript(directory, agent_id, records, meta=None):
    os.makedirs(os.path.join(directory, "subagents"), exist_ok=True)
    path = os.path.join(directory, "subagents", f"agent-{agent_id}.jsonl")
    with open(path, "w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")
    if meta:
        json.dump(meta, open(path[: -len(".jsonl")] + ".meta.json", "w", encoding="utf-8"), ensure_ascii=False)


def write_session_stops(session_directory, minutes_ago_by_agent):
    """主会话记录里 TaskStop 的调用与成功结果，结果的形态照 2026-09-18 主会话记录里的原样。"""
    with open(session_directory + ".jsonl", "w", encoding="utf-8") as handle:
        for index, (agent_id, minutes_ago) in enumerate(minutes_ago_by_agent.items()):
            use = record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": f"s{index}", "name": "TaskStop", "input": {"task_id": agent_id}}],
                            stop_reason="tool_use", message_id=f"ms{index}")
            message = f"Successfully stopped task: {agent_id} (样本)"
            result = record_at(minutes_ago - 0.01, "user", [{"tool_use_id": f"s{index}", "type": "tool_result",
                                                             "content": json.dumps({"message": message, "task_id": agent_id}, ensure_ascii=False)}])
            result["toolUseResult"] = {"message": message, "task_id": agent_id, "task_type": "local_agent", "command": "样本"}
            handle.write(json.dumps(use, ensure_ascii=False) + "\n")
            handle.write(json.dumps(result, ensure_ascii=False) + "\n")


def iso_minutes_ago(minutes):
    return (datetime.now(timezone.utc).timestamp() - minutes * 60)


def record_at(minutes_ago, role, content, stop_reason=None, message_id=None, usage=None, origin_kind=None):
    stamp = datetime.fromtimestamp(iso_minutes_ago(minutes_ago), timezone.utc).isoformat().replace("+00:00", "Z")
    message = {"role": role, "content": content}
    if stop_reason:
        message["stop_reason"] = stop_reason
    if message_id:
        message["id"] = message_id
    if usage:
        message["usage"] = usage
    record = {"timestamp": stamp, "type": role, "message": message}
    if origin_kind:
        record["origin"] = {"kind": origin_kind}
    return record


def bash_use(minutes_ago, tool_id, command, message_id):
    return record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": tool_id, "name": "Bash", "input": {"command": command}}],
                     stop_reason="tool_use", message_id=message_id, usage={"input_tokens": 10, "cache_read_input_tokens": 1000})


def bash_result(minutes_ago, tool_id):
    return record_at(minutes_ago, "user", [{"type": "tool_result", "tool_use_id": tool_id, "content": "ok"}])


def handback_records(minutes_ago, tool_id, message_id):
    """交回工具的调用与成功返回，返回的形态照 2026-09-18 会话记录里的原样。"""
    return [record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": tool_id, "name": "SubagentHandback", "input": {"message": "报告"}}],
                      stop_reason="tool_use", message_id=message_id, usage={"input_tokens": 10, "cache_read_input_tokens": 1000}),
            record_at(minutes_ago - 0.1, "user", [{"type": "tool_result", "tool_use_id": tool_id,
                                                   "content": [{"type": "text", "text": '{"success":true,"message":"Report delivered to your caller."}'}]}])]


SELFTEST_STOP_GRACE_SECONDS = 20   # 自证收尾发 TERM 之后等自己起的那一层退多久，到点才对它发 KILL


def term_forwarding(command):
    """自证的假脚本里起子进程的写法：子进程放后台、wait 它；收到 TERM 时只停自己起的那一个（kill "$!"），等它退了再退。
    trap 写在起子进程之前：TERM 来得早时 $! 还是空的，kill 落空、照样退。自证收尾只给 Popen 记着的那一层发 TERM（stop_own_processes），
    每一层各停各的子进程，整棵树停干净，不从进程表里挑进程逐个发信号（.claude/process-safety-pending 原来那一行）。"""
    return ("trap 'kill \"$!\" 2>/dev/null; wait \"$!\"; exit 143' TERM\n"
            f"{command} &\n"
            'wait "$!"\n')


def stop_own_processes(processes):
    """只给自证自己起的进程（Popen 记着的进程号）发信号：先各发 TERM，假脚本照 term_forwarding 停它自己起的那一个、timeout 停它自己那一组，
    整棵树由各层自己收；过 SELFTEST_STOP_GRACE_SECONDS 还没退的，对那一个发 KILL。已经退了的只收尸。"""
    for process in processes:
        if process.poll() is None:
            process.terminate()
    for process in processes:
        try:
            process.wait(timeout=SELFTEST_STOP_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def wait_until(condition, timeout_seconds=SELFTEST_READY_TIMEOUT_SECONDS):
    """自证按条件等（子进程起到要测的样子），不按固定秒数等：条件成立回 True，到点回 False，调用方照原判据接着判。"""
    deadline = time.time() + timeout_seconds
    while not condition():
        if time.time() >= deadline:
            return False
        time.sleep(0.05)
    return True


def wait_for_selftest_ready(ready_path, watcher, timeout_seconds=SELFTEST_READY_TIMEOUT_SECONDS):
    """等自证拉起的看门狗建就绪文件（mark_selftest_ready）：建了回 True；看门狗先退了或到点回 False，调用方照原判据接着判。"""
    return wait_until(lambda: os.path.exists(ready_path) or watcher.poll() is not None, timeout_seconds) and os.path.exists(ready_path)


def selftest_kept_copies(work, thresholds, failures):
    """「交回之后副本还在」的自检：另起会话目录，交回正文与它点名的报告文件里写「没删 <全路径>」的几个合成子 agent。返回查了几个合成子 agent。
    弄坏开关：keptcopy（两格红）、keptcopyage（「只报交回超过 6 小时……」那一格红：keptrecent 也报）、keptcopyexists（同一格红：已经删掉的也报）。"""
    session = os.path.join(work, "kept-copy-session")
    copies = os.path.join(work, "kept-copies")
    paths = {name: os.path.join(copies, name) for name in ("old-repo", "old-target", "recent-repo", "report-repo", "running-repo", "deleted-repo")}
    for name, path in paths.items():
        if name != "deleted-repo":
            os.makedirs(os.path.join(path, "crates"), exist_ok=True)
    report_path = os.path.join(work, "kept-copy-report.md")
    with open(report_path, "w", encoding="utf-8") as handle:
        handle.write(f"# 报告\n\n## 没做什么\n\n- 没删 `{paths['report-repo']}`：主 agent 要在里面跑轻型变异\n")

    def handed_back(agent_id, hours_ago, message):
        minutes = hours_ago * 60
        records = [bash_use(minutes + 5, "t1", "ls", "m1"), bash_result(minutes + 4, "t1"), *handback_records(minutes, "h1", "m2")]
        records[2]["message"]["content"][0]["input"]["message"] = message
        write_transcript(session, agent_id, records, {"agentType": "implementation-writer", "description": agent_id})

    handed_back("keptold", 7, f"结论：做完了。\n没删 {paths['old-repo']}/：主 agent 要复跑\n没删：{paths['old-target']}，留给主 agent\n"
                              f"没删 {paths['deleted-repo']}：已经被别人删了")
    handed_back("keptrecent", 1, f"没删 {paths['recent-repo']}：主 agent 要复跑")
    handed_back("keptreport", 8, f"报告 {report_path}（sha256 0000），交回正文里不写没删的路径")
    handed_back("keptnone", 9, "结论：做完了，草稿都删了")
    write_transcript(session, "keptrunning", [bash_use(500, "t1", f"echo 没删 {paths['running-repo']}", "m1"), bash_result(499, "t1")],
                     {"agentType": "implementation-writer", "description": "还没交回"})
    kept_thresholds = argparse.Namespace(**dict(vars(thresholds), kept_copy_hours=6))
    _lines, found = kept_copy_report({session}, datetime.now(timezone.utc), kept_thresholds)
    got = sorted((agent_id, re.match(r"副本 (\S+) 还在", explanation).group(1)) for agent_id, name, explanation, _ in found if name == KEPT_COPY_ALERT)
    wanted = sorted([("keptold", paths["old-repo"]), ("keptold", paths["old-target"]), ("keptreport", paths["report-repo"])])
    if got != wanted:
        failures.append(f"「{KEPT_COPY_ALERT}」应当只报交回超过 6 小时、交回正文或它点名的报告里写了没删而路径还在的 {wanted}，实际 {got}")
    report_lines, report_alerts, *_ = build_report(["keptold"], session, kept_thresholds, time.time(), set(), None)
    remaining, acknowledged = split_acknowledged(report_alerts, {f"keptold:{KEPT_COPY_ALERT}"})
    if not (any(name == KEPT_COPY_ALERT for _, name, _, _ in report_alerts) and any(line.startswith("交回之后副本还在：查了") for line in report_lines)
            and not any(subject == "keptold" and name == KEPT_COPY_ALERT for subject, name, _, _ in remaining) and acknowledged):
        failures.append(f"看门狗的报告应当带「{KEPT_COPY_ALERT}」告警与一行汇总、--ack keptold:{KEPT_COPY_ALERT} 之后不叫醒，"
                        f"实际告警 {[name for _, name, _, _ in report_alerts]}、确认掉 {len(acknowledged)} 条、汇总 {[line for line in report_lines if '副本' in line]}")
    return 5


def selftest_scratch_hogs(work, thresholds, failures, watch_runs):
    """「草稿根里超阈值的编译目录与仓副本」的自检：造一个草稿根，里面一个超阈值的编译目录（带 CACHEDIR.TAG）与一个不超的、
    一个超阈值的仓副本与一个不超的、一个超阈值却没有 CACHEDIR.TAG 签名的目录与一个超阈值的普通目录；交回时只报超阈值的编译目录与仓副本。返回查了几格。
    弄坏开关：scratchhog（报告里一条都没有）、scratchhogage（不超的也报）、scratchhogtag（编译目录认不出）。"""
    checks = 0
    root = os.path.join(work, "scratch-hog-root")
    tag = CACHEDIR_TAG_SIGNATURE + b"\n# sample\n"
    layout = {   # 路径 → (文件 → 内容)，空内容的键以 / 结尾是目录
        "old-build/target": {"CACHEDIR.TAG": tag, "release/deps/lib.rlib": b"x"},
        "fresh-build/target": {"CACHEDIR.TAG": tag, "release/deps/lib.rlib": b"x"},
        "old-copy": {".git/HEAD": b"ref: refs/heads/x\n", "Cargo.toml": b"[workspace]\n"},
        "fresh-copy": {".git/HEAD": b"ref: refs/heads/x\n", "Cargo.toml": b"[workspace]\n"},
        "old-minimal-copy": {"Cargo.toml": b"[workspace]\n", "crates/a/lib.rs": b"//\n"},
        "old-badtag/target": {"CACHEDIR.TAG": b"Signature: 00000000000000000000000000000000\n", "x": b"x"},
        "old-plain": {"notes.txt": b"x"},
    }
    for relative, files in layout.items():
        for name, content in files.items():
            path = os.path.join(root, relative, name)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "wb") as handle:
                handle.write(content)
    hours_idle = {"old-build": 30, "old-copy": 10, "old-minimal-copy": 20, "old-badtag": 10, "old-plain": 10}   # 排序要看得出：build 最久、minimal 其次、copy 最近
    for top, hours in hours_idle.items():
        moment = time.time() - hours * 3600
        for directory, directories, files in os.walk(os.path.join(root, top), topdown=False):
            for name in [*files, *directories]:
                os.utime(os.path.join(directory, name), (moment, moment), follow_symlinks=False)
            os.utime(directory, (moment, moment))
    wanted = {"编译目录 " + os.path.join(root, "old-build/target"), "仓副本 " + os.path.join(root, "old-copy"),
              "仓副本 " + os.path.join(root, "old-minimal-copy")}
    unwanted = [os.path.join(root, name) for name in ("fresh-build", "fresh-copy", "old-badtag", "old-plain")]
    lines = scratch_hog_report([root], datetime.now(timezone.utc), 6)
    detail_lines = [line for line in lines if line.startswith(("  编译目录 ", "  仓副本 "))]
    reported = {" ".join(re.match(r"  (编译目录|仓副本) (\S+)：", line).groups()) for line in detail_lines}
    checks += 1
    if reported != wanted:
        failures.append(f"草稿根里超阈值的编译目录与仓副本：应当只报 {sorted(wanted)}（超过 6 小时没动的编译目录、仓副本），实际 {sorted(reported)}")
    checks += 1
    if not (lines and lines[0].startswith("草稿根里超阈值的编译目录与仓副本：查了 1 个根") and "只报不删" in lines[0]):
        failures.append(f"草稿根那一节的第一行应当是汇总（查了几个根、报几处、只报不删），实际 {lines[:1]}")
    checks += 1
    order = [re.match(r"  (?:编译目录|仓副本) (\S+)：", line).group(1) for line in detail_lines]
    if order != [os.path.join(root, "old-build/target"), os.path.join(root, "old-minimal-copy"), os.path.join(root, "old-copy")]:
        failures.append(f"草稿根那一节应当按最后修改时刻、最久没动的在前（old-build 30 小时、old-minimal-copy 20 小时、old-copy 10 小时），实际次序 {order}")
    checks += 1
    if scratch_hog_report([root], datetime.now(timezone.utc), 0)[0].count("这一类没查") != 1:
        failures.append("--scratch-idle-hours 写 0 时应当写明这一类没查")
    # 端到端：有子 agent 交回时看门狗的报告里带这一节、只含超阈值的；没有子 agent 交回的报告里不带（交回时才报）
    session = os.path.join(work, "scratch-hog-session")
    write_transcript(session, "hoghanded", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18, "h1", "m2")],
                     {"agentType": "implementation-writer", "description": "交回了"})
    write_transcript(session, "hogrunning", [bash_use(1, "t1", "cargo build", "m1"), bash_result(0.5, "t1")],
                     {"agentType": "implementation-writer", "description": "还在跑"})
    environment = dict(os.environ, **{SCRATCH_ROOTS_ENV: root})

    def report_for(agent_id):
        watch_runs.append(["report", "--agents", agent_id])
        return subprocess.run([sys.executable, os.path.abspath(__file__), "report", "--agents", agent_id, "--session-dir", session,
                               "--process-root-pid", "0"], capture_output=True, text=True, timeout=120, env=environment)
    handed = report_for("hoghanded")
    checks += 1
    if (handed.returncode != 0 or not all(path.split(" ", 1)[1] in handed.stdout for path in wanted)
            or any(path in handed.stdout for path in unwanted)):
        failures.append(f"有子 agent 交回时看门狗的报告应当列出超阈值的 {sorted(wanted)}、不列 {unwanted}，实际退出码 {handed.returncode}：{handed.stdout[-900:]}")
    running = report_for("hogrunning")
    checks += 1
    if "草稿根里超阈值的编译目录与仓副本" in running.stdout:
        failures.append(f"没有子 agent 交回时看门狗的报告里不该带草稿根那一节，实际：{running.stdout[-600:]}")
    return checks


def selftest_root_free(work, failures, watch_runs):
    """「根分区余量过低」的自检：读可用量的函数换成假的（不读 os.statvfs），阈值 128 GiB 时判一次过线（可用 40 GiB）、一次不过线（可用 500 GiB），阈值 0 不查；
    再起看门狗子进程（--agents 与 --processes-only 两条路的 report，环境变量 AGENT_WATCH_ROOT_FREE_BYTES 给假的可用量）看告警、退出码与 --ack。返回查了几格。
    弄坏开关：rootfree（过线也不报）。"""
    checks = 0
    asked = []

    def fake_free(gibibytes):
        def read(path):
            asked.append(path)
            return int(gibibytes * BYTES_PER_GIBIBYTE)
        return read
    threshold = argparse.Namespace(root_free_gib_alert=128)
    lines, found = root_free_report(threshold, fake_free(40))
    checks += 1
    if ([name for name, _, _ in found] != [ROOT_FREE_ALERT] or "40.0 GiB" not in found[0][1] or "128 GiB" not in found[0][1]
            or not all(text in found[0][2] for text in ("crash-amplification/runs/", "fetched", "library-for-peer*", "du -sh", "别人在用的不删",
                                                        f"--ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT}"))):
        failures.append(f"根分区可用 40 GiB、阈值 128 GiB 时应当报一条「{ROOT_FREE_ALERT}」，说明写可用与阈值、出路先点名崩溃放量 runs/ 下的 fetched 与 library-for-peer*、"
                        f"以 du 为准、别人在用的不删、--ack 写法，实际 {found}")
    checks += 1
    if asked != [root_free_path()] or not os.path.exists(asked[0]):
        failures.append(f"根分区可用量应当只读一次、读的是工作根（还没建的取第一个在的上层）{root_free_path()}，实际读了 {asked}")
    lines, found = root_free_report(threshold, fake_free(500))
    checks += 1
    if found or not any(line.startswith("根分区余量：") and "可用 500.0 GiB" in line for line in lines):
        failures.append(f"根分区可用 500 GiB、阈值 128 GiB 时不该报，报告里应当记一行可用多少，实际告警 {found}、报告行 {lines}")
    lines, found = root_free_report(argparse.Namespace(root_free_gib_alert=0), fake_free(1))
    checks += 1
    if found or not lines or "这一类没查" not in lines[0]:
        failures.append(f"--root-free-gib-alert 写 0 时应当不报、写明这一类没查，实际告警 {found}、报告行 {lines}")
    # 端到端：看门狗子进程照环境变量读假的可用量；--agents 那条路过线退 3、--ack 之后退 0 并写「已确认接着盯」、不过线退 0
    session = os.path.join(work, "root-free-session")
    write_transcript(session, "rootfreerunning", [bash_use(1, "t1", "ls", "m1"), bash_result(0.5, "t1")],
                     {"agentType": "implementation-writer", "description": "还在跑"})

    def report_with(gibibytes, *extra):
        watch_runs.append(["report", "--agents", "rootfreerunning", f"root-free-gibibytes={gibibytes}", *extra])
        return subprocess.run([sys.executable, os.path.abspath(__file__), "report", "--agents", "rootfreerunning", "--session-dir", session,
                               "--process-root-pid", "0", "--root-free-gib-alert", "128", *extra], capture_output=True, text=True, timeout=120,
                              env=dict(os.environ, **{ROOT_FREE_BYTES_VARIABLE: str(int(gibibytes * BYTES_PER_GIBIBYTE))}))
    low = report_with(40)
    checks += 1
    if low.returncode != 3 or f"⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）：" not in low.stdout:
        failures.append(f"看门狗 report 在根分区可用 40 GiB 时应当报「⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）」、退出码 3，实际 {low.returncode}：{low.stdout[-600:]}")
    acked = report_with(40, "--ack", f"{ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT}")
    checks += 1
    if acked.returncode != 0 or f"已确认接着盯（--ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT}）" not in acked.stdout:
        failures.append(f"根分区余量过低被 --ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT} 确认之后看门狗 report 应当退 0、写「已确认接着盯」，实际 {acked.returncode}：{acked.stdout[-600:]}")
    high = report_with(500)
    checks += 1
    if high.returncode != 0 or ROOT_FREE_ALERT in high.stdout or "可用 500.0 GiB" not in high.stdout:
        failures.append(f"看门狗 report 在根分区可用 500 GiB 时不该报、应当退 0 并记一行可用多少，实际 {high.returncode}：{high.stdout[-600:]}")
    # --processes-only 那条路：盯一个自己起的 sleep（它底下没有别的进程，进程一半不报），过线同样报、退 3；查完只停自己起的这一个
    sleeper = subprocess.Popen(["sleep", "120"])
    try:
        watch_runs.append(["report", "--processes-only", "root-free-gibibytes=40"])
        processes_only = subprocess.run([sys.executable, os.path.abspath(__file__), "report", "--processes-only", "--process-root-pid", str(sleeper.pid),
                                         "--root-free-gib-alert", "128"], capture_output=True, text=True, timeout=120,
                                        env=dict(os.environ, **{ROOT_FREE_BYTES_VARIABLE: str(40 * BYTES_PER_GIBIBYTE)}))
    finally:
        sleeper.terminate()
        sleeper.wait()
    checks += 1
    if processes_only.returncode != 3 or f"⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）：" not in processes_only.stdout:
        failures.append(f"--processes-only 的 report 在根分区可用 40 GiB 时应当同样报「⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）」、退出码 3，"
                        f"实际 {processes_only.returncode}：{processes_only.stdout[-600:]}")
    return checks


def selftest_stale_sessions(work, failures, watch_runs):
    """「重启前旧进程留下的子 agent 记录判已失效」的自检：造登记目录（~/.claude/sessions 的形态）与项目目录，三类记录：
    一、已不在的会话（登记的进程已退出）里的、本会话里的 → 前者不报「工具调用过长」「无动静」，后者照报；
    二、会话号还活着、但最后一条记录早于它现在这个进程启动时刻的 → 不报，晚于的照报。返回查了几格。
    弄坏开关：stalesession（第一类红）、stalesessionrestart（第二类红）。"""
    checks = 0
    registry = os.path.join(work, "session-registry")
    projects = os.path.join(work, "stale-projects", "project")
    os.makedirs(registry, exist_ok=True)
    finished = subprocess.Popen(["true"])
    finished.wait()   # 已退出并收了尸的子进程：它的 pid 此刻不在 /proc
    own_start = process_start_ticks(os.getpid())
    for file_name, session_id, pid, proc_start, started_minutes_ago in (
            ("dead.json", "dead-session", finished.pid, "1", 600),
            ("live.json", "live-session", os.getpid(), own_start, 600),
            ("restarted.json", "restarted-session", os.getpid(), own_start, 10),
            ("reused-pid.json", "reused-pid-session", os.getpid(), "1", 600)):   # pid 活着但被别的进程占了（procStart 对不上）：也算不在
        with open(os.path.join(registry, file_name), "w", encoding="utf-8") as handle:
            json.dump({"pid": pid, "sessionId": session_id, "procStart": proc_start,
                       "startedAt": int((time.time() - started_minutes_ago * 60) * 1000)}, handle)
    long_tool = [record_at(40, "user", "派发"), bash_use(40, "t1", "cargo test --release", "m1")]
    for session_id, agent_id, records in (
            ("dead-session", "deadlong", long_tool),                                                       # 工具调用过长的形状
            ("dead-session", "deadidle", [bash_use(30, "t1", "ls", "m1"), bash_result(29, "t1")]),         # 无动静的形状
            ("reused-pid-session", "reusedpid", long_tool),
            ("live-session", "livelong", long_tool),                                                       # 本会话的：照报
            ("restarted-session", "beforestart", long_tool),                                               # 进程 10 分钟前启动，这一条最后一条记录 40 分钟前：旧进程的
            ("restarted-session", "afterstart", [record_at(12, "user", "派发"), bash_use(9, "t1", "cargo test --release", "m1")])):   # 启动之后派的：照报
        write_transcript(os.path.join(projects, session_id), agent_id, records, {"agentType": "implementation-writer", "description": agent_id})
    environment = dict(os.environ, **{SESSION_REGISTRY_ENV: registry, "AGENT_WATCH_PROJECTS_ROOT": os.path.join(work, "stale-projects"),
                                      "CLAUDE_CODE_SESSION_ID": "live-session", SCRATCH_ROOTS_ENV: os.path.join(work, "no-scratch")})

    def report_for(agent_ids):
        watch_runs.append(["report", "--agents", agent_ids, "stale"])
        return subprocess.run([sys.executable, os.path.abspath(__file__), "report", "--agents", agent_ids, "--process-root-pid", "0"],
                              capture_output=True, text=True, timeout=120, env=environment)
    everything = report_for("deadlong,deadidle,reusedpid,livelong,beforestart,afterstart")
    alerted = {match.group(1) for match in re.finditer(r"^⚠️ [^（\n]+（(\w+)）：", everything.stdout, re.M)}
    checks += 1
    if alerted != {"livelong", "afterstart"}:
        failures.append("重启前旧进程留下的记录（已不在的会话 deadlong / deadidle / reusedpid、会话号还活着但最后一条记录早于进程启动的 beforestart）不该报告警，"
                        f"本会话的 livelong 与进程启动之后的 afterstart 照报；实际告警的是 {sorted(alerted)}：{everything.stdout[-900:]}")
    checks += 1
    for agent_id in ("deadlong", "deadidle", "reusedpid", "beforestart"):
        if not re.search(rf"子 agent {agent_id} .*状态=已失效", everything.stdout):
            failures.append(f"子 agent {agent_id} 应当在报告里标成「状态=已失效」，实际：{everything.stdout[-900:]}")
            break
    only_stale = report_for("deadlong,deadidle,beforestart")
    checks += 1
    if only_stale.returncode != 0 or "⚠️" in only_stale.stdout:
        failures.append(f"被看的只有重启前旧进程留下的记录时应当没有告警、退出码 0，实际 {only_stale.returncode}：{only_stale.stdout[-600:]}")
    unreadable = dict(environment, **{SESSION_REGISTRY_ENV: os.path.join(work, "no-such-registry")})
    watch_runs.append(["report", "--agents", "deadlong", "unreadable-registry"])
    unjudged = subprocess.run([sys.executable, os.path.abspath(__file__), "report", "--agents", "deadlong", "--process-root-pid", "0"],
                              capture_output=True, text=True, timeout=120, env=unreadable)
    checks += 1
    if "⚠️ 工具调用过长（deadlong）" not in unjudged.stdout:
        failures.append(f"登记目录读不了时判不了会话在不在，不该把子 agent 当成已失效而吞掉告警，实际：{unjudged.stdout[-600:]}")
    return checks


def selftest_leftover_background(work, thresholds, failures, watch_runs, probed_processes):
    """「交回之后后台还在跑」的自检：另起会话目录，免得这些开着输出文件的进程搅到别的看门狗自检。返回查了几个合成子 agent。
    每个要看的子 agent 起过一个后台任务，自检起一个 sleep 把它的输出文件开成标准输出（harness 起的后台命令就是这个形状）。"""
    leftover_session = os.path.join(work, "leftover-session")
    holders = {}

    def write_leftover_agent(session_directory, agent_id, started_minutes_ago, tail_records, description, keep_output_open=True):
        output_path = os.path.join(work, f"{agent_id}.output")
        with open(output_path, "a") as output_handle:
            if keep_output_open:
                holders[agent_id] = subprocess.Popen(["sleep", "300"], stdout=output_handle)
                probed_processes.append(holders[agent_id])
        started_record = record_at(started_minutes_ago - 0.05, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                   "content": f"Command running in background with ID: bg{agent_id}. Output is being written to: {output_path}. "
                                              "You will be notified when it completes."}])
        write_transcript(session_directory, agent_id, [bash_use(started_minutes_ago, "t1", "until [ -f /nonexistent ]; do sleep 5; done", "m1"),
                                                       started_record, *tail_records],
                         {"agentType": "three-way-verifier", "description": description})

    def stop_holder(agent_id):
        holders[agent_id].kill()
        holders[agent_id].wait()

    def alerting_pids(found):
        """{子 agent id: 告警里报的进程号}；说明不是「进程 <pid> 已跑 …」开头的，原样放说明，比对时照样对不上。"""
        pids = {}
        for owner, name, explanation, _ in found:
            if name == LEFTOVER_BACKGROUND_ALERT:
                reported_pid = re.match(r"进程 (\d+) 已跑 ", explanation)
                pids[owner] = int(reported_pid.group(1)) if reported_pid else explanation
        return pids

    try:
        write_leftover_agent(leftover_session, "leftoverhandedback", 12, [*handback_records(10, "h1", "m2"),
                             record_at(9.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                             "交回 10 分钟、它起的等待循环还在跑")
        write_leftover_agent(leftover_session, "leftovergrace", 1, [*handback_records(0.6, "h1", "m2"),
                             record_at(0.4, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                             "交回 30 秒、还在宽限里")
        write_leftover_agent(leftover_session, "leftoverstopped", 12, [
                             record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                             "结束本轮等后台任务时被 TaskStop 停掉")
        write_leftover_agent(leftover_session, "leftoverfailed", 12, [
                             record_at(9.7, "assistant", [{"type": "text", "text": "接着跑"}], message_id="m2")],
                             "撞限额失败（最近一次任务通知是 failed）")
        write_leftover_agent(leftover_session, "leftoverrunning", 12, [
                             record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                             "没交回、结束本轮在等自己的后台任务")
        write_leftover_agent(leftover_session, "leftoverresumed", 30, [
                             record_at(20, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2"),
                             record_at(3, "user", "续做：接着跑", origin_kind="coordinator"), bash_use(2, "t2", "cargo build", "m3")],
                             "上一个 Claude 进程退出时没跑完（stopped）、之后又被续做、正在跑")
        write_session_stops(leftover_session, {"leftoverstopped": 9})
        with open(leftover_session + ".jsonl", "a", encoding="utf-8") as handle:
            failed_notification = record_at(9.5, "user", "<task-notification>\n<task-id>leftoverfailed</task-id>\n<status>failed</status>\n"
                                                         "<summary>Agent failed: API error</summary>\n</task-notification>", origin_kind="task-notification")
            stopped_notification = {"type": "queue-operation", "operation": "enqueue", "timestamp": record_at(15, "user", "")["timestamp"],
                                    "content": "<task-notification>\n<task-id>leftoverresumed</task-id>\n<task-id>aother</task-id>\n<status>stopped</status>\n"
                                               "<summary>2 background agents didn't finish before the previous session ended</summary>\n</task-notification>"}
            handle.write(json.dumps(failed_notification, ensure_ascii=False) + "\n")
            handle.write(json.dumps(stopped_notification, ensure_ascii=False) + "\n")
        wanted_alerting = {agent_id: holders[agent_id].pid for agent_id in ("leftoverhandedback", "leftoverstopped", "leftoverfailed")}
        lines, found, pending_until = leftover_background_report({leftover_session}, datetime.now(timezone.utc), thresholds)
        if alerting_pids(found) != wanted_alerting:
            failures.append(f"「{LEFTOVER_BACKGROUND_ALERT}」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个、各报开着输出文件的那个进程 {wanted_alerting}"
                            f"（交回不到宽限、没交回在等、stopped 之后又被续做的不报），实际 {alerting_pids(found) or '无'}")
        if pending_until is None or not any("宽限里" in line and "leftovergrace" in line for line in lines):
            failures.append(f"交回 30 秒、宽限 2 分钟的 leftovergrace 应当记成宽限里、给出宽限到期的时刻，实际到期时刻 {pending_until}，报告：{lines}")
        stop_holder("leftovergrace")

        def run_leftover_watch(agent_ids, session_directory, extra_arguments, timeout=60):
            """看门狗到 timeout 秒还没退：不抛出去，退出码记成「超时（N 秒没退出）」交回，那一格照原判据判红、打 ✗ 行，别的格接着跑。"""
            watch_runs.append(["leftover", agent_ids, *extra_arguments])
            try:
                return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--agents", agent_ids, "--session-dir", session_directory,
                                       "--process-root-pid", "0", "--interval-seconds", "1", "--max-minutes", "1", *extra_arguments],
                                      capture_output=True, text=True, timeout=timeout)
            except subprocess.TimeoutExpired as expired:
                partial = expired.stdout or ""
                if isinstance(partial, bytes):
                    partial = partial.decode("utf-8", errors="replace")
                return subprocess.CompletedProcess(expired.cmd, f"超时（{timeout} 秒没退出）", partial, "")

        handed_back_pid = wanted_alerting["leftoverhandedback"]
        wanted_alert_line = f"⚠️ {LEFTOVER_BACKGROUND_ALERT}（leftoverhandedback）：进程 {handed_back_pid} 已跑 "
        all_done_run = run_leftover_watch("leftoverhandedback", leftover_session, [])
        if (all_done_run.returncode != 3 or wanted_alert_line not in all_done_run.stdout
                or f"{PROCESS_STOP_COMMAND} {handed_back_pid}" not in all_done_run.stdout):
            failures.append(f"被看的子 agent 全部交回、它丢下的后台进程还在时，看门狗退出前应当报「{wanted_alert_line}…」、下一步给按进程号停的命令、退出码 3，"
                            f"实际退出码 {all_done_run.returncode}，输出：{all_done_run.stdout[-400:]}")
        acknowledged_run = run_leftover_watch("leftoverhandedback", leftover_session,
                                              [argument for pid in wanted_alerting.values() for argument in ("--ack", f"进程:{pid}")])
        if acknowledged_run.returncode != 0 or "已确认接着盯" not in acknowledged_run.stdout or "全部交回或被停" not in acknowledged_run.stdout:
            failures.append(f"「{LEFTOVER_BACKGROUND_ALERT}」的进程都用 --ack 进程:<pid> 确认过时不该叫醒：应当退出码 0、写「已确认接着盯」，"
                            f"实际退出码 {acknowledged_run.returncode}，输出：{acknowledged_run.stdout[-400:]}")
        watch_runs.append(["--processes-only", "--session-dir", "leftover"])
        processes_run = subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--processes-only", "--process-root-pid", str(os.getpid()),
                                        "--session-dir", leftover_session, "--interval-seconds", "1", "--max-minutes", "0.5"],
                                       capture_output=True, text=True, timeout=120)
        if processes_run.returncode != 3 or wanted_alert_line not in processes_run.stdout:
            failures.append(f"--processes-only 带 --session-dir 时也应当报「{wanted_alert_line}…」并退出码 3，"
                            f"实际退出码 {processes_run.returncode}，输出：{processes_run.stdout[-400:]}")
        # 被看的子 agent 刚交回、它的后台进程还在宽限里：看门狗不许当场退 0 放过，要等宽限过了再查一次、报出来
        wait_session = os.path.join(work, "leftover-wait-session")
        write_leftover_agent(wait_session, "leftoverjustnow", 0.3, [*handback_records(0.15, "h1", "m2"),
                             record_at(0.04, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                             "刚交回、后台进程还开着输出文件")
        wait_run = run_leftover_watch("leftoverjustnow", wait_session, ["--leftover-background-grace-minutes", "0.15", "--interval-seconds", "30"])
        wanted_wait_line = f"⚠️ {LEFTOVER_BACKGROUND_ALERT}（leftoverjustnow）：进程 {holders['leftoverjustnow'].pid} 已跑 "
        if wait_run.returncode != 3 or wanted_wait_line not in wait_run.stdout:
            failures.append(f"被看的子 agent 刚交回、后台进程还在宽限里时看门狗应当等宽限过了再查、报「{wanted_wait_line}…」退出码 3，"
                            f"实际退出码 {wait_run.returncode}，输出：{wait_run.stdout[-400:]}")
        stop_holder("leftoverhandedback")
        _, found_after_exit, _ = leftover_background_report({leftover_session}, datetime.now(timezone.utc), thresholds)
        if "leftoverhandedback" in alerting_pids(found_after_exit):
            failures.append(f"leftoverhandedback 丢下的进程 {handed_back_pid} 已经没了，不该再报「{LEFTOVER_BACKGROUND_ALERT}」，实际 {alerting_pids(found_after_exit)}")
    finally:
        for process in holders.values():
            process.kill()
            process.wait()
    return len(holders)


def selftest_reminder_timer(work, thresholds, failures):
    """remind.sh start 底下等到点的 sleep 不报「进程无输出」；同形的 remind.sh stop、别的脚本名 start 底下的 sleep 照报。
    三个假脚本都开着一个一小时没动的文件、起一个 sleep 等着（形状同真的 remind.sh：bash <路径>/remind.sh start …，sleep 是它的后台子进程）。返回查了几个。"""
    script_directory = os.path.join(work, "reminder-timer", "scripts", "agents")
    os.makedirs(script_directory, exist_ok=True)
    stale_file = os.path.join(work, "reminder-timer", "stale-output.log")
    open(stale_file, "w").close()
    old = time.time() - 3600
    os.utime(stale_file, (old, old))
    fake_body = "#!/usr/bin/env bash\nexec 3>>\"$2\"\ntouch -d '1 hour ago' \"$2\"\n" + term_forwarding("sleep 30")
    scripts = {}
    for label, file_name, verb in (("start", "remind.sh", "start"), ("stop", "remind.sh", "stop"), ("other", "other-timer.sh", "start")):
        path = os.path.join(script_directory, file_name)
        if not os.path.exists(path):
            with open(path, "w") as handle:
                handle.write(fake_body)
        scripts[label] = (path, verb)
    parents = {label: subprocess.Popen(["bash", path, verb, stale_file], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
               for label, (path, verb) in scripts.items()}

    def sleeper_below(parent_pid):
        return next((pid for pid, info in process_table().items() if info["parent"] == parent_pid and info["command"].startswith("sleep ")), None)

    sleepers = {}
    try:
        started = time.time()
        wait_until(lambda: all(sleeper_below(parent.pid) for parent in parents.values()))
        sleepers = {label: sleeper_below(parent.pid) for label, parent in parents.items()}
        time.sleep(max(0.0, 4 - (time.time() - started)))   # 「进程无输出」要已跑 3 秒
        process_lines, found = process_alerts(os.getpid(), set(), thresholds)
        reported = {label: any(name == "进程无输出" and f"进程 {pid} " in explanation for name, explanation, _ in found) for label, pid in sleepers.items()}
        if None in sleepers.values():
            failures.append(f"没起出三个假提醒脚本底下的 sleep：{sleepers}")
        if reported["start"] or any(f"进程 {sleepers['start']} " in explanation for _, explanation, _ in found):
            failures.append(f"remind.sh start 底下等到点的 sleep {sleepers['start']} 不该报告警（既不报「进程无输出」也不报「进程过长」）：{[e for _, e, _ in found]}")
        if not any(f"进程 {sleepers['start']} " in line and "定时提醒" in line for line in process_lines):
            failures.append(f"remind.sh start 底下等到点的 sleep {sleepers['start']} 在进程一节要有一行说它是定时提醒的计时，实际 {process_lines}")
        for label in ("stop", "other"):
            if not reported[label]:
                failures.append(f"{label} 那个假脚本底下的 sleep {sleepers[label]} 同形却不是 remind.sh start，应当照报「进程无输出」："
                                f"{[e for _, e, _ in found] or '一条告警都没有'}")
    finally:
        # 只停自己起的三个假脚本：它们收到 TERM 由 trap 停各自起的 sleep（term_forwarding）
        stop_own_processes(list(parents.values()))
    return len(parents)


def selftest_pipe_upstream(work, thresholds, failures, probed_processes):
    """管道尾在等上游：上游在算（只涨 CPU、不写管道）的不报「进程无输出」、报一行「在等上游」；上游睡着的、写端只在自己祖先手里的照报。
    管道尾的输出文件一小时没动，形状照 2026-09-24 被误报的 `cargo test … 2>&1 | tail -30`。返回查了几个管道。"""
    pipelines = {}
    busy_ready = os.path.join(work, "pipe-busy-ready")   # 在算的上游进了忙等就建它：起来之前它的 CPU 不涨
    for label, upstream_command in (("busy", [sys.executable, "-c", f"import time\nopen({busy_ready!r}, 'w').close()\n"
                                                                    "deadline = time.time() + 60\nwhile time.time() < deadline:\n    pass\n"]),
                                    ("sleeping", ["sleep", "60"])):
        tail_output = os.path.join(work, f"pipe-{label}-tail.log")
        with open(tail_output, "a") as tail_handle:
            os.utime(tail_output, (time.time() - 3600, time.time() - 3600))
            # stderr 不继承自检进程的：继承来的若是一份正被写着的日志，会被算成上游或管道尾「写着的文件在涨」
            upstream = subprocess.Popen(upstream_command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
            reader = subprocess.Popen(["tail", "-1"], stdin=upstream.stdout, stdout=tail_handle, stderr=subprocess.DEVNULL)
        upstream.stdout.close()   # 自检进程不留管道的读端：管道两端只在上游与管道尾手里，同 shell 起的管道
        pipelines[label] = (upstream, reader)
        probed_processes += [upstream, reader]
    # 写端是自己的祖先（自检进程拿着它 stdin 的写端，而自检进程的子孙里就有上面那个在算的上游）：祖先不算上游，照报
    ancestor_fed_output = os.path.join(work, "pipe-ancestor-fed-tail.log")
    with open(ancestor_fed_output, "a") as tail_handle:
        os.utime(ancestor_fed_output, (time.time() - 3600, time.time() - 3600))
        ancestor_fed = subprocess.Popen(["tail", "-1"], stdin=subprocess.PIPE, stdout=tail_handle, stderr=subprocess.DEVNULL)
    probed_processes.append(ancestor_fed)
    try:
        # 按条件等在算的上游进了忙等，再补足 4 秒（「进程无输出」要已跑 3 秒）：负载高时 python 起得慢，固定等 4 秒时它可能还没起来
        pipes_started = time.time()
        wait_until(lambda: os.path.exists(busy_ready))
        time.sleep(max(0.0, 4 - (time.time() - pipes_started)))
        process_lines, found = process_alerts(os.getpid(), set(), thresholds)
        busy_upstream, busy_reader = pipelines["busy"]
        sleeping_upstream, sleeping_reader = pipelines["sleeping"]
        if any(f"进程 {busy_reader.pid} " in explanation for _, explanation, _ in found):
            failures.append(f"stdin 是管道、上游 {busy_upstream.pid} 一直在算的管道尾 {busy_reader.pid} 被当成告警（它在等上游）："
                            f"{[explanation for _, explanation, _ in found]}")
        if not any(f"进程 {busy_reader.pid} 在等上游" in line and f"写端进程 {busy_upstream.pid}" in line for line in process_lines):
            failures.append(f"上游 {busy_upstream.pid} 在算的管道尾 {busy_reader.pid} 在报告里没有「在等上游」那一行并点名写端，实际 {process_lines}")
        if not any(name == "进程无输出" and f"进程 {sleeping_reader.pid} " in explanation for name, explanation, _ in found):
            failures.append(f"stdin 是管道、上游 {sleeping_upstream.pid} 睡着（不算也不写）的管道尾 {sleeping_reader.pid} 没报「进程无输出」："
                            f"{[explanation for _, explanation, _ in found] or '一条告警都没有'}")
        if not any(name == "进程无输出" and f"进程 {ancestor_fed.pid} " in explanation for name, explanation, _ in found):
            failures.append(f"stdin 管道的写端只在自己的祖先（自检进程 {os.getpid()}）手里的 {ancestor_fed.pid} 没报「进程无输出」："
                            f"祖先的子孙里有不相干的在算的进程，不能拿来当上游")
    finally:
        for process in [process for pair in pipelines.values() for process in pair] + [ancestor_fed]:
            process.kill()
            process.wait()
        ancestor_fed.stdin.close()
    return len(pipelines) + 1


def selftest_heavy_tests(work, failures, probed_processes, watch_runs):
    """「重型测试没带前缀」的自检：拿自检进程当 Claude 实例，起几个假的重型进程（名字照 cargo 编出来的 checker 档包 singlefs-checker-tier 的测试二进制，里面只是睡）。
    没带前缀的、带别的值的报，带 commit / user-request 的不报；bash -c 里写着带前缀的重型命令、那条命令还没起的不报；
    timeout 包着带前缀的不报，timeout 包着没带前缀的只报 timeout 那一个、下一步给整棵子树从最底层往上的停止次序；
    后台起、开着一个子 agent 后台任务输出文件的，归属报出那个子 agent；stage-selftest.sh 与带 --selftest 的脚本底下起的 checker-tier-crates-mutation-replay 不报，
    同一个 checker-tier-crates-mutation-replay 不经它们直接起照报（AGENT_WATCH_BREAK=heavysampleselftest 去掉这条放行，前一格必须判错）；checker-tier-crates-mutation-replay 用去了编号的新名字
    checker-tier-crates-mutation-replay.sh。不带前缀直接起的不重型门禁阶段（提交时才跑的检查）不报（AGENT_WATCH_BREAK=heavycommitcheck 下照报，判错）。
    返回起了几个假进程。"""
    deps = os.path.join(work, "heavy", "target", "release", "deps")
    os.makedirs(deps, exist_ok=True)
    fake_binary = os.path.join(deps, "crash_enumeration_new_pool_file_creation_stream-0123456789abcdef")   # python 顶着这个 argv[0] 睡，不用真的编
    fake_script = os.path.join(deps, "crash_enumeration_fixed_script_stream-fedcba9876543210")                    # sh 脚本：timeout、bash -c 要真的 exec 它
    with open(fake_script, "w") as handle:
        handle.write("#!/bin/sh\n" + term_forwarding(f"sleep {SELFTEST_FAKE_PROCESS_SECONDS}") + "exit 0\n")
    os.chmod(fake_script, 0o755)
    # 另一个看门狗的自检（python 起的 agent-watch.py）底下起的假重型进程：那一支整棵不扫
    fake_watchdog = os.path.join(work, "other-watchdog", "agent-watch.py")
    os.makedirs(os.path.dirname(fake_watchdog), exist_ok=True)
    with open(fake_watchdog, "w") as handle:
        # 收到 TERM 只停它自己起的那一个假重型进程（Popen 记着的），收了尸再退
        handle.write("import signal, subprocess, sys, time\n"
                     f"child = subprocess.Popen([sys.argv[1], '-c', 'import time\\ntime.sleep({SELFTEST_FAKE_PROCESS_SECONDS})\\n'], executable=sys.executable)\n"
                     "signal.signal(signal.SIGTERM, lambda *_: (child.terminate(), child.wait(), sys.exit(143)))\n"
                     f"time.sleep({SELFTEST_FAKE_PROCESS_SECONDS})\n")
    # 双机分片的驱动脚本（crash-point-replay-shard-run.sh，这里只是睡）在两棵树里各起一趟：驱动自检那样的临时仓登记的是 harness 档包的替身用例（不报），
    # 真跑的树登记的是 checker 档包、带 shard=across-machines（没带前缀照报）。lib_heavy_tests.py 按参数里树根下键那一行判
    driver_trees = {}
    for label, key, package in (("驱动自检的临时仓", "crash-case:sharded-selftest", "singlefs-harness"), ("真跑的树", "crash-case:replay-sample", "singlefs-checker-tier")):
        tree = os.path.join(work, "driver-trees", key.split(":")[1], "repository")
        os.makedirs(os.path.join(tree, ".claude", "gate.d"))
        os.makedirs(os.path.join(tree, "scripts"))
        os.makedirs(os.path.join(tree, "scripts", "crash"), exist_ok=True)
        with open(os.path.join(tree, ".claude", "gate.d", "stage-inputs.tsv"), "w") as handle:
            handle.write(f"{key}\tcrates/\ttest={package}:a_target:a_case shard=across-machines\t# 看门狗自检的样本\n")
        with open(os.path.join(tree, "scripts", "crash", "crash-point-replay-shard-run.sh"), "w") as handle:
            handle.write(term_forwarding(f"sleep {SELFTEST_FAKE_PROCESS_SECONDS}"))
        driver_trees[label] = (key, tree)
    # 样本自检：假的 stage-selftest.sh 与带 --selftest 的假脚本各在底下起一个 crates 变异表那一道（checker-tier-crates-mutation-replay，这里只是睡），都不报；
    # 同一道不经它们直接起，照报。不重型的门禁阶段（提交时才跑的检查）不带前缀直接起也不报（AGENT_WATCH_BREAK=heavycommitcheck 下照报，判错）
    sample_tree = os.path.join(work, "sample-selftest-tree")
    sample_stage = os.path.join(sample_tree, ".claude", "gate.d", "checker-tier-crates-mutation-replay.sh")
    sample_light_stage = os.path.join(sample_tree, ".claude", "gate.d", "doc-sample.sh")
    sample_runner = os.path.join(sample_tree, ".claude", "singlefs-ai-sop", "scripts", "stage-selftest.sh")
    selftest_runner = os.path.join(sample_tree, "scripts", "some-script.sh")
    for path, text in ((sample_stage, term_forwarding(f"sleep {SELFTEST_FAKE_PROCESS_SECONDS}")), (sample_light_stage, term_forwarding(f"sleep {SELFTEST_FAKE_PROCESS_SECONDS}")),
                       (sample_runner, term_forwarding('bash "$1" "$2" --force') + "exit 0\n"), (selftest_runner, term_forwarding('bash "$2"') + "exit 0\n")):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as handle:
            handle.write(text)
    heavy_session = os.path.join(work, "heavy-session")
    owned_output = os.path.join(work, "heavyowner.output")
    write_transcript(heavy_session, "heavyowner", [
        bash_use(1, "t1", f"{fake_binary} --test-threads 4", "m1"),
        record_at(0.95, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                  "content": f"Command running in background with ID: bh1. Output is being written to: {owned_output}. "
                                             "You will be notified when it completes."}]),
        record_at(0.9, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
        {"agentType": "implementation-writer", "description": "后台起了一个没带前缀的层 0 测试二进制"})
    clean_environment = {key: value for key, value in os.environ.items() if key != HEAVY_TEST_VARIABLE}
    started = {}

    def start(label, argv, occasion=None, stdout=subprocess.DEVNULL, executable=None):
        environment = dict(clean_environment, **({HEAVY_TEST_VARIABLE: occasion} if occasion else {}))
        started[label] = subprocess.Popen(argv, executable=executable, env=environment, cwd=work, stdout=stdout, stderr=subprocess.DEVNULL)
        probed_processes.append(started[label])

    # 假进程都睡 SELFTEST_FAKE_PROCESS_SECONDS：活到这一格判完（负载下两趟看门狗加 --ack 那一趟要好几分钟），在 finally 里停掉
    sleeping = ["-c", f"import time\ntime.sleep({SELFTEST_FAKE_PROCESS_SECONDS})\n"]
    try:
        with open(owned_output, "a") as output_handle:
            start("没带前缀", [fake_binary, *sleeping], stdout=output_handle, executable=sys.executable)
        start("带 commit", [fake_binary, *sleeping], "commit", executable=sys.executable)
        start("带 user-request", [fake_binary, *sleeping], "user-request", executable=sys.executable)
        start("带别的值", [fake_binary, *sleeping], "yes", executable=sys.executable)
        start("bash -c 里带前缀、还没起", ["bash", "-c", term_forwarding(f"sleep {SELFTEST_FAKE_PROCESS_SECONDS}") + f"{HEAVY_TEST_VARIABLE}=commit {fake_script}"])
        start("timeout 包着带前缀的", ["timeout", str(SELFTEST_FAKE_PROCESS_SECONDS), "env", f"{HEAVY_TEST_VARIABLE}=commit", fake_script])
        start("timeout 包着没带前缀的", ["timeout", str(SELFTEST_FAKE_PROCESS_SECONDS), fake_script])
        start("另一个看门狗的自检底下没带前缀的", [sys.executable, fake_watchdog, fake_binary])
        for label, (key, tree) in driver_trees.items():
            start(f"{label}里起的双机分片驱动", ["bash", os.path.join(tree, "scripts", "crash", "crash-point-replay-shard-run.sh"), key, tree])
        start("stage-selftest.sh 拿样本起的 checker-tier-crates-mutation-replay", ["bash", sample_runner, sample_stage, sample_tree])
        start("带 --selftest 的脚本底下起的 checker-tier-crates-mutation-replay", ["bash", selftest_runner, "--selftest", sample_stage])
        start("不带前缀直接起的 checker-tier-crates-mutation-replay", ["bash", sample_stage, sample_tree])
        start("不带前缀直接起的不重型门禁阶段", ["bash", sample_light_stage, sample_tree])
        # 按条件等孙辈起来（timeout 起 sh、sh 起 sleep，假看门狗起它的假重型进程，两个样本自检起底下那一道，bash -c 起 sleep），不按固定秒数：
        # 负载高时等 1.5 秒不够，孙辈还没起，停止次序与「样本自检底下不判」的数就对不上。
        # 「timeout 包着带前缀的」那一支要等到 timeout 底下那一层已经 exec 成带着前缀的脚本、脚本起了 sleep：env 还没 exec 时那一层的环境里没有前缀，
        # timeout 底下认不出带前缀的重型进程，它就被当成没带前缀的报出来（负载过百时并行复跑里报过 6 条）
        table = {}

        def children_of(pid):
            return sorted(child for child, info in table.items() if info["parent"] == pid)

        def occasion_of(pid):
            try:
                return heavy_test_occasion(pid)
            except OSError:   # 刚退出或刚 exec 完：这一次不算起齐
                return None

        def grandchildren_started():
            table.clear()
            table.update(process_table())
            timeout_children = children_of(started["timeout 包着没带前缀的"].pid)
            prefixed_children = children_of(started["timeout 包着带前缀的"].pid)
            return (bool(timeout_children) and bool(children_of(timeout_children[0]))
                    and bool(prefixed_children) and occasion_of(prefixed_children[0]) == "commit" and bool(children_of(prefixed_children[0]))
                    and all(children_of(started[label].pid) for label in (
                        "另一个看门狗的自检底下没带前缀的", "stage-selftest.sh 拿样本起的 checker-tier-crates-mutation-replay",
                        "带 --selftest 的脚本底下起的 checker-tier-crates-mutation-replay", "bash -c 里带前缀、还没起")))

        wait_until(grandchildren_started)

        timeout_pid = started["timeout 包着没带前缀的"].pid
        script_pid = (children_of(timeout_pid) or [None])[0]
        sleep_pid = (children_of(script_pid) or [None])[0] if script_pid else None
        lines, found = heavy_test_report(os.getpid(), set(), {heavy_session})
        alerted = {}
        for name, explanation, next_step in found:
            reported = re.match(r"进程 (\d+) ", explanation)
            if name == HEAVY_TEST_ALERT and reported:
                alerted[int(reported.group(1))] = (explanation, next_step)
        wanted = {started["没带前缀"].pid: "没带前缀", started["带别的值"].pid: "带别的值", timeout_pid: "timeout 包着没带前缀的",
                  started["真跑的树里起的双机分片驱动"].pid: "真跑的树里起的双机分片驱动",
                  started["不带前缀直接起的 checker-tier-crates-mutation-replay"].pid: "不带前缀直接起的 checker-tier-crates-mutation-replay"}
        if set(alerted) != set(wanted):
            failures.append(f"「{HEAVY_TEST_ALERT}」应当只报 {sorted(wanted.values())} 这 {len(wanted)} 个（进程号 {sorted(wanted)}），"
                            f"带 commit / user-request 的、bash -c 里带前缀还没起的、timeout 包着带前缀的、timeout 底下那个 sh、"
                            f"另一个看门狗的自检底下的、驱动自检的临时仓里起的双机分片驱动（登记的是 harness 档包）、"
                            f"stage-selftest.sh 拿样本起的与带 --selftest 的脚本底下起的 checker-tier-crates-mutation-replay、不带前缀直接起的不重型门禁阶段都不报；"
                            f"实际报了 {sorted(alerted)}（{[started_label for started_label, process in started.items() if process.pid in alerted]}），报告：{lines}")
        unprefixed_explanation = alerted.get(started["没带前缀"].pid, ("", ""))[0]
        if not all(part in unprefixed_explanation for part in ("层 0", f"没带 {HEAVY_TEST_VARIABLE}=commit", "子 agent heavyowner", "heavyowner.output")):
            failures.append(f"没带前缀的层 0 进程的告警应当写类别「层 0」、「没带 {HEAVY_TEST_VARIABLE}=commit」、归属「子 agent heavyowner」与它开着的输出文件，"
                            f"实际：{unprefixed_explanation or '没报'}")
        other_value_explanation = alerted.get(started["带别的值"].pid, ("", ""))[0]
        if f"带的是 {HEAVY_TEST_VARIABLE}=yes" not in other_value_explanation or "认不出" not in other_value_explanation:
            failures.append(f"带 {HEAVY_TEST_VARIABLE}=yes 的告警应当写「带的是 {HEAVY_TEST_VARIABLE}=yes」、归属认不出，实际：{other_value_explanation or '没报'}")
        timeout_explanation, timeout_next_step = alerted.get(timeout_pid, ("", ""))
        wanted_order = f"{sleep_pid} {script_pid} {timeout_pid}"
        if ("底下另有 1 个同样没带前缀的重型进程" not in timeout_explanation or wanted_order not in timeout_next_step
                or PROCESS_STOP_COMMAND not in timeout_next_step or f"--ack 进程:{timeout_pid}" not in timeout_next_step):
            failures.append(f"timeout 包着没带前缀的只报 timeout 那一个：说明里写「底下另有 1 个」，下一步给从最底层往上的停止次序「{wanted_order}」、"
                            f"停的命令与 --ack 进程:{timeout_pid}；实际说明：{timeout_explanation or '没报'}，下一步：{timeout_next_step}")
        if not any(line.startswith(f"{HEAVY_TEST_ALERT}：查了") and f"报 {len(wanted)} 条" in line for line in lines):
            failures.append(f"报告里应当有一行「{HEAVY_TEST_ALERT}：查了 … 报 {len(wanted)} 条」，实际 {lines}")
        if not any(re.search(r"样本自检底下 [1-9]\d* 个不判", line) for line in lines):
            failures.append(f"报告那一行应当写出样本自检底下不判的进程数（这里至少 1 个），实际 {lines}")
        # 走看门狗：--processes-only 与 --agents 两条路都报、退 3；报的那几个都 --ack 进程:<pid> 之后不叫醒，到点退 4
        wanted_alert_line = f"⚠️ {HEAVY_TEST_ALERT}（进程）：进程 {started['没带前缀'].pid} "
        base = [sys.executable, os.path.abspath(__file__), "watch", "--process-root-pid", str(os.getpid()), "--session-dir", heavy_session,
                "--interval-seconds", "1", "--max-minutes", "0.1"]
        for label, extra in (("--processes-only", ["--processes-only"]), ("--agents", ["--agents", "heavyowner"])):
            watch_runs.append(["heavy", label])
            heavy_run = subprocess.run(base + extra, capture_output=True, text=True, timeout=120)
            if heavy_run.returncode != 3 or wanted_alert_line not in heavy_run.stdout or f"   → 怎么办：主 agent 判断：是该跑的" not in heavy_run.stdout:
                failures.append(f"看门狗 {label} 盯着没带前缀的层 0 进程时应当报「{wanted_alert_line}…」连同出路、退出码 3，"
                                f"实际退出码 {heavy_run.returncode}，输出：{heavy_run.stdout[-500:]}")
        watch_runs.append(["heavy", "--processes-only", "--ack"])
        acknowledged_run = subprocess.run(base + ["--processes-only"] + [argument for pid in wanted for argument in ("--ack", f"进程:{pid}")],
                                          capture_output=True, text=True, timeout=120)
        if acknowledged_run.returncode != 4 or "已确认接着盯（--ack 进程:" not in acknowledged_run.stdout:
            failures.append(f"「{HEAVY_TEST_ALERT}」的 {len(wanted)} 个进程都 --ack 进程:<pid> 之后看门狗不该叫醒：应当到点退出码 4 并写「已确认接着盯」，"
                            f"实际退出码 {acknowledged_run.returncode}，输出：{acknowledged_run.stdout[-500:]}")
    finally:
        # 只停自己起的那一层：timeout 收到 TERM 停它自己那一组，假脚本、bash -c 照 term_forwarding 停各自起的那一个，假看门狗停它起的假重型进程
        stop_own_processes(list(started.values()))
    return len(started)


def selftest_heavy_selftest_tree(work, failures, probed_processes):
    """「重型测试没带前缀」里登记成轻的自证（文件头那一段）的自检：造一个仓根与一份登记表（lib_heavy_tests.py 的 selftest_registry_swapped 换过去），
    登记 scripts/crash/sample-shard-run.sh 为 heavy-if-set:SAMPLE_REAL_TOOLS；它带 --selftest 时 exec 进同目录 sample-shard-run-selftest.sh 本体，
    本体 mktemp 建临时仓、在里面起双机分片驱动 crash-point-replay-shard-run.sh（只是睡；树根里没有 stage-inputs.tsv，lib_heavy_tests.py 照重型算）。五格：
    自证父进程 + 它临时仓里的驱动 ⇒ 不报；没有自证祖先的同一条驱动命令（另一个临时仓）⇒ 报；设了 SAMPLE_REAL_TOOLS 的同一份自证底下 ⇒ 报；
    仓副本里路径尾部相同的自证底下 ⇒ 报；不在自证底下、路径落在第一格那份自证临时目录里的驱动 ⇒ 不报；另核报告那一行写出两类各几个。返回起了几个进程。"""
    library, problem = heavy_tests_library()
    if library is None:
        failures.append(f"登记成轻的自证那几格判不了：{problem}")
        return 0
    base = os.path.join(work, "selftest-tree")
    temporary_base = os.path.join(base, "tmp-base")
    registered_root, copy_root = os.path.join(base, "registered"), os.path.join(base, "copy")
    os.makedirs(temporary_base)
    # 驱动只是睡，照 term_forwarding 写：收到 TERM 停它自己起的 sleep。本体拷这一份进临时仓
    driver_template = os.path.join(base, "driver-template.sh")
    with open(driver_template, "w") as handle:
        handle.write(term_forwarding("sleep 60"))
    front_text = '[[ "${1:-}" != --selftest ]] || exec bash "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/sample-shard-run-selftest.sh"\nexit 0\n'
    body_text = ('work="$(mktemp -d)"\n'
                 'mkdir -p "$work/repository/scripts/crash"\n'
                 f"cp '{driver_template}' \"$work/repository/scripts/crash/crash-point-replay-shard-run.sh\"\n"
                 "printf '%s\\n' \"$work\" > \"$SAMPLE_SELFTEST_TREE_READY\"\n"
                 + term_forwarding('bash "$work/repository/scripts/crash/crash-point-replay-shard-run.sh" crash-case:sharded-selftest "$work/repository"'))
    for root in (registered_root, copy_root):
        scripts = os.path.join(root, "scripts", "crash")
        os.makedirs(scripts)
        for name, text in (("sample-shard-run.sh", front_text), ("sample-shard-run-selftest.sh", body_text)):
            with open(os.path.join(scripts, name), "w") as handle:
                handle.write(text)
    registry = os.path.join(base, "selftest-registry.tsv")
    with open(registry, "w") as handle:
        handle.write("scripts/crash/sample-shard-run.sh\theavy-if-set:SAMPLE_REAL_TOOLS\t# 看门狗自检：--selftest exec 进本体，本体在 mktemp 临时仓里起驱动\n")
    clean_environment = {key: value for key, value in os.environ.items() if key not in (HEAVY_TEST_VARIABLE, "SAMPLE_REAL_TOOLS")}
    clean_environment["TMPDIR"] = temporary_base
    started, ready_files = {}, {}

    def start(label, argv, extra=None):
        started[label] = subprocess.Popen(argv, env=dict(clean_environment, **(extra or {})), cwd=work, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        probed_processes.append(started[label])

    def start_selftest(label, root, extra=None):
        ready_files[label] = os.path.join(base, f"ready-{len(ready_files)}")
        start(label, ["bash", os.path.join(root, "scripts", "crash", "sample-shard-run.sh"), "--selftest"],
              dict(extra or {}, SAMPLE_SELFTEST_TREE_READY=ready_files[label]))

    def driver_command(tree):
        repository = os.path.join(tree, "repository")
        return ["bash", os.path.join(repository, "scripts", "crash", "crash-point-replay-shard-run.sh"), "crash-case:sharded-selftest", repository]

    table = {}

    def driver_under(pid):
        """pid 底下那个双机分片驱动（bash 起的 crash-point-replay-shard-run.sh，它底下的 sleep 也起了才算）的进程号；还没起是 None。"""
        for child, info in table.items():
            if info["parent"] == pid and "crash-point-replay-shard-run.sh" in info["command"] and any(other["parent"] == child for other in table.values()):
                return child
        return None

    def selftests_running():
        table.clear()
        table.update(process_table())
        return all(os.path.exists(ready_files[label]) and driver_under(started[label].pid) for label in ready_files)

    try:
        start_selftest("登记成轻的自证", registered_root)
        start_selftest("设了 SAMPLE_REAL_TOOLS 的自证", registered_root, {"SAMPLE_REAL_TOOLS": "1"})
        start_selftest("仓副本里的自证", copy_root)
        standalone_tree = tempfile.mkdtemp(dir=temporary_base)
        os.makedirs(os.path.join(standalone_tree, "repository", "scripts", "crash"))
        with open(os.path.join(standalone_tree, "repository", "scripts", "crash", "crash-point-replay-shard-run.sh"), "w") as handle:
            handle.write(term_forwarding("sleep 60"))
        start("没有自证祖先的驱动", driver_command(standalone_tree))
        wait_until(selftests_running)
        with open(ready_files["登记成轻的自证"]) as handle:
            selftest_temporary = handle.read().strip()
        start("落在自证临时目录里、不在它底下的驱动", driver_command(selftest_temporary))
        wait_until(lambda: any(info["parent"] == started["落在自证临时目录里、不在它底下的驱动"].pid for info in process_table().values()))
        drivers = {label: driver_under(started[label].pid) for label in ready_files}
        drivers["没有自证祖先的驱动"] = started["没有自证祖先的驱动"].pid
        drivers["落在自证临时目录里、不在它底下的驱动"] = started["落在自证临时目录里、不在它底下的驱动"].pid
        with library.selftest_registry_swapped(registry, registered_root):
            lines, found = heavy_test_report(os.getpid(), set(), set())
        alerted = {}
        for name, explanation, _ in found:
            reported = re.match(r"进程 (\d+) ", explanation)
            if name == HEAVY_TEST_ALERT and reported:
                alerted[int(reported.group(1))] = explanation
        shown = f"驱动 {drivers}，自证 {({label: started[label].pid for label in ready_files})}，实际报了 {sorted(alerted)}，报告：{lines}"
        if drivers["登记成轻的自证"] in alerted or started["登记成轻的自证"].pid in alerted or drivers["登记成轻的自证"] is None:
            failures.append(f"登记成轻的自证（exec 进本体）底下、它临时仓里起的双机分片驱动不该报，本体自己也不该报：{shown}")
        if drivers["没有自证祖先的驱动"] not in alerted:
            failures.append(f"没有自证祖先、同一条双机分片驱动命令应当报：{shown}")
        if drivers["设了 SAMPLE_REAL_TOOLS 的自证"] not in alerted:
            failures.append(f"设了 SAMPLE_REAL_TOOLS 的同一份自证（heavy-if-set）底下的驱动应当报：{shown}")
        if drivers["仓副本里的自证"] not in alerted:
            failures.append(f"仓副本里路径尾部相同的自证底下的驱动应当报：{shown}")
        if drivers["落在自证临时目录里、不在它底下的驱动"] in alerted:
            failures.append(f"不在自证底下、路径落在登记成轻的自证临时目录里的驱动不该报：{shown}")
        if not any(re.search(r"登记成轻的自证底下 [1-9]\d* 个不判、不在它底下而路径落在它临时目录里的 [1-9]\d* 个不报", line) for line in lines):
            failures.append(f"报告那一行应当写出登记成轻的自证底下不判、它临时目录里不报的进程数（这里各至少 1 个），实际 {lines}")
    finally:
        # 只停自己起的那一层（exec 之后的本体、单独起的驱动）：本体停它起的驱动、驱动停它起的 sleep，各层照 term_forwarding 停自己起的那一个
        stop_own_processes(list(started.values()))
    return len(started)


def selftest_light_tier_tree(work, failures, probed_processes):
    """「重型测试没带前缀」里轻型三条（文件头「轻型三条底下不判」）的自检：造一个仓根，scripts 里放假的内存包装 run-with-memory-cap.sh（跑它后面的命令）与
    假的 crash-point-replay.sh（起 bin/cargo test -p singlefs-checker-tier --lib，bin/cargo 是只睡的脚本）。两格：
    经内存包装起 `crash-point-replay.sh --quick`、底下是 checker 档 cargo ⇒ 整棵一个都不报；同一棵树换成不带 `--quick` ⇒ 照报（只报最上面的包装那一层，说明里写底下另有脚本那一层与 cargo 两个）；
    另核报告那一行写出轻型三条底下不判了几个。返回起了几个进程。"""
    library, problem = heavy_tests_library()
    if library is None:
        failures.append(f"轻型三条那几格判不了：{problem}")
        return 0
    repository = os.path.join(work, "light-tier-tree", "repository")
    scripts = os.path.join(repository, "scripts")
    os.makedirs(os.path.join(scripts, "hosts"))
    os.makedirs(os.path.join(scripts, "crash"))
    os.makedirs(os.path.join(repository, "bin"))
    wrapper = os.path.join(scripts, "hosts", "run-with-memory-cap.sh")
    replay = os.path.join(scripts, "crash", "crash-point-replay.sh")
    fake_cargo = os.path.join(repository, "bin", "cargo")
    member = os.path.join(repository, "crates", "singlefs-checker-tier")   # cargo test -p 要在带这个成员的工作区里才认得出是 checker 档包
    os.makedirs(os.path.join(member, "src"))
    for path, text in ((os.path.join(repository, "Cargo.toml"), '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-checker-tier"]\n'),
                       (os.path.join(member, "Cargo.toml"), '[package]\nname = "singlefs-checker-tier"\nversion = "0.1.0"\n'),
                       (os.path.join(member, "src", "lib.rs"), "pub fn judge() {}\n"),
                       (wrapper, "shift\n" + term_forwarding('"$@"') + "exit 0\n"),
                       (replay, term_forwarding(f"bash {fake_cargo} test -p singlefs-checker-tier --lib") + "exit 0\n"),
                       (fake_cargo, term_forwarding("sleep 60") + "exit 0\n")):
        with open(path, "w") as handle:
            handle.write(text)
    clean_environment = {key: value for key, value in os.environ.items() if key != HEAVY_TEST_VARIABLE}
    started = {}
    table = {}

    def tree_of(pid):
        found, stack = set(), [pid]
        while stack:
            current = stack.pop()
            found.add(current)
            stack.extend(child for child, info in table.items() if info["parent"] == current)
        return found

    def sleeping_below(pid):
        return any("sleep" in table[member]["command"] for member in tree_of(pid) if member != pid)

    def trees_running():
        table.clear()
        table.update(process_table())
        return all(sleeping_below(process.pid) for process in started.values())

    try:
        for label, replay_arguments in (("轻型三条", ["--quick"]), ("不带 --quick", [])):
            started[label] = subprocess.Popen(["bash", wrapper, "4G", "bash", replay, *replay_arguments], env=clean_environment, cwd=repository,
                                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            probed_processes.append(started[label])
        wait_until(trees_running)
        trees = {label: tree_of(process.pid) for label, process in started.items()}
        lines, found = heavy_test_report(os.getpid(), set(), set())
        alerted, explanations = set(), {}
        for name, explanation, _ in found:
            reported = re.match(r"进程 (\d+) ", explanation)
            if name == HEAVY_TEST_ALERT and reported:
                alerted.add(int(reported.group(1)))
                explanations[int(reported.group(1))] = explanation
        shown = f"两棵树 {({label: sorted(tree) for label, tree in trees.items()})}，实际报了 {sorted(alerted)}，报告：{lines}"
        if alerted & trees["轻型三条"] or not all(len(tree) >= 4 for tree in trees.values()):
            failures.append(f"经内存包装起的 crash-point-replay.sh --quick 底下的 cargo 不该报（每棵树至少 4 个进程：包装、脚本、cargo 脚本、sleep）：{shown}")
        wrapper_pid = started["不带 --quick"].pid
        if alerted & trees["不带 --quick"] != {wrapper_pid} or "底下另有 2 个同样没带前缀的重型进程" not in explanations.get(wrapper_pid, ""):
            failures.append(f"同一棵树换成不带 --quick 应当只报最上面的包装那一层 {wrapper_pid}，说明里写「底下另有 2 个同样没带前缀的重型进程」（脚本那一层与 cargo 都认出是重型）：{shown}")
        if not any(re.search(r"轻型三条底下 [1-9]\d* 个不判", line) for line in lines):
            failures.append(f"报告那一行应当写出轻型三条底下不判的进程数（这里至少 1 个），实际 {lines}")
    finally:
        # 只停自己起的两个包装：包装、脚本、cargo 脚本各照 term_forwarding 停自己起的那一个
        stop_own_processes(list(started.values()))
    return len(started)


def selftest_verdict_lines(work, thresholds, failures, probed_processes):
    """「长跑里有判红的行」的自检（文件头同名一段）：两个只睡的进程各开着一份日志写（fd 3），一份 ✓ 2 ✗ 1、一份 ✓ 2；
    判红的那份报「长跑里有判红的行」并列出 ✗ 那一行，全绿的那份只在报告里写一行、不告警。返回查了几份日志。"""
    logs = {"red": "进度 1/3\n  ✓ crash-case: alpha\n  ✓ crash-case: beta\n  ✗ crash-case: gamma 判红\n进度 3/3\n",
            "green": "进度 1/2\n  ✓ crash-case: alpha\n  ✓ crash-case: beta\n"}
    holders = {}
    try:
        for label, text in logs.items():
            path = os.path.join(work, f"verdict-{label}.log")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
            holder = subprocess.Popen(["bash", "-c", 'exec 3>>"$0"; exec sleep 60', path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            probed_processes.append(holder)
            holders[label] = (holder, path)
        started = time.time()
        wait_until(lambda: all((process_arguments(holder.pid) or [""])[0] == "sleep" for holder, _ in holders.values()))
        time.sleep(max(0.0, thresholds.process_report_minutes * 60 + 0.5 - (time.time() - started)))   # 跑满 --process-report-minutes 才数
        lines, found = process_alerts(os.getpid(), set(), thresholds)
        red_holder, red_path = holders["red"]
        green_holder, green_path = holders["green"]
        red_alerts = [explanation for name, explanation, _ in found if name == VERDICT_RED_ALERT and explanation.startswith(f"进程 {red_holder.pid} ")]
        if not (red_alerts and "✓ 2 ✗ 1" in red_alerts[0] and "✗ crash-case: gamma 判红" in red_alerts[0] and red_path in red_alerts[0]):
            failures.append(f"判红的日志应当报「{VERDICT_RED_ALERT}」：进程 {red_holder.pid} 写着的 {red_path} 里 ✓ 2 ✗ 1、列出 ✗ 那一行，实际 {found}")
        if any(name == VERDICT_RED_ALERT and f"进程 {green_holder.pid} " in explanation for name, explanation, _ in found):
            failures.append(f"全绿的日志 {green_path}（进程 {green_holder.pid}）不该报「{VERDICT_RED_ALERT}」，实际 {found}")
        if not any(line == f"  判定行 {green_path}（进程 {green_holder.pid} 写着）：✓ 2 ✗ 0" for line in lines) \
                or not any(line.startswith("判定行：跑满 ") for line in lines):
            failures.append(f"报告里应当有判定行那一行（「判定行：跑满 …」汇总与「  判定行 {green_path}（进程 {green_holder.pid} 写着）：✓ 2 ✗ 0」），实际 {lines}")
    finally:
        for holder, _ in holders.values():
            holder.kill()
            holder.wait()
    return len(holders)


def selftest_verdict_selftest_scratch(work, thresholds, failures, probed_processes):
    """「长跑里有判红的行」里登记成轻的自证临时目录（文件头同名一段）的自检：造一个仓根与一份登记表（lib_heavy_tests.py 的 selftest_registry_swapped 换过去），
    登记 scripts/crash/sample-wheel.sh 为 light。它带 --selftest 起：mktemp -d 建一个临时目录、再按 lib_selftest_scratch.py 的 PASS_PROBE_PREFIX 建一个探查目录、
    再照崩溃放量自证的写法 mktemp -d "$TMPDIR/sample-wheel-selftest.XXXXXX" 建一个，三处各写一份带 ✗ 的开关轮输出，另在 TMPDIR 底下直接写一份带 ✗ 的
    （不在哪个临时目录里），四份都开着写，然后睡。起两份：共用 TMPDIR 的一份（经 bash -c 起，上层与它同一个 TMPDIR，像门禁阶段里直接起的自证）、
    单给 TMPDIR 的一份（像 light-selftest-guard.py 给的私有 TMPDIR）。另起一个没有自证上层的进程，开着写一份落在像 mktemp 的临时目录里、带 ✗ 的文件。
    四格：共用 TMPDIR 的自证在 mktemp 与探查目录里的两份不报、单给 TMPDIR 的自证在 sample-wheel-selftest.XXXXXX 里的不报（报告里各写「✗ 不报」）、
    共用 TMPDIR 的自证直接写在 TMPDIR 底下的照报、没有自证上层的照报。返回断言过的文件份数。"""
    library, problem = heavy_tests_library()
    prefixes, scratch_problem = selftest_scratch_prefixes()
    if library is None or not prefixes:
        failures.append(f"登记成轻的自证在自己临时目录里写的 ✗ 不该报：这一段判不了（{problem or scratch_problem}）")
        return 0
    base = os.path.join(work, "verdict-selftest-scratch")
    registered_root = os.path.join(base, "registered")
    scripts = os.path.join(registered_root, "scripts", "crash")
    shared_base, private_base = os.path.join(base, "shared-tmp"), os.path.join(base, "private-tmp")
    for directory in (scripts, shared_base, private_base):
        os.makedirs(directory)
    script = os.path.join(scripts, "sample-wheel.sh")
    with open(script, "w") as handle:
        handle.write('scratch="$(mktemp -d)"\n'
                     'probe="$(mktemp -d "$TMPDIR/${SAMPLE_PROBE_PREFIX}XXXXXXXX")"\n'
                     'named="$(mktemp -d "$TMPDIR/sample-wheel-selftest.XXXXXX")"\n'
                     "printf '  ✓ 格 alpha\\n  \\342\\234\\227 开关轮 beta 红在点名那一格\\n' > \"$scratch/switch-wheel-0.out\"\n"
                     "printf '  \\342\\234\\227 开关轮 gamma 红在点名那一格\\n' > \"$probe/switch-wheel-1.out\"\n"
                     "printf '  \\342\\234\\227 开关轮 zeta 红在点名那一格\\n' > \"$named/switch-wheel-2.out\"\n"
                     "printf '  \\342\\234\\227 自证 delta 真红了\\n' > \"$TMPDIR/selftest-direct.log\"\n"
                     'exec 3>>"$scratch/switch-wheel-0.out" 4>>"$probe/switch-wheel-1.out" 5>>"$named/switch-wheel-2.out" 6>>"$TMPDIR/selftest-direct.log"\n'
                     "printf '%s\\n%s\\n%s\\n' \"$scratch/switch-wheel-0.out\" \"$probe/switch-wheel-1.out\" \"$named/switch-wheel-2.out\" > \"$SAMPLE_READY\"\n"
                     + term_forwarding("sleep 60"))
    registry = os.path.join(base, "selftest-registry.tsv")
    with open(registry, "w") as handle:
        handle.write("scripts/crash/sample-wheel.sh\tlight\t# 看门狗自检：开关轮的输出写在自己的临时目录里\n")
    plain_directory = tempfile.mkdtemp(dir=shared_base)   # 名字像 mktemp 起的（tmpXXXXXXXX），但写它的进程没有自证上层
    plain = os.path.join(plain_directory, "switch-wheel-9.out")
    with open(plain, "w", encoding="utf-8") as handle:
        handle.write("  ✗ 长跑 epsilon 判红\n")
    clean_environment = {key: value for key, value in os.environ.items() if key != HEAVY_TEST_VARIABLE}
    ready = {"shared": os.path.join(base, "ready-shared"), "private": os.path.join(base, "ready-private")}
    started = []

    def start(argv, temporary_base, ready_file=None):
        process = subprocess.Popen(argv, cwd=work, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                   env=dict(clean_environment, TMPDIR=temporary_base, SAMPLE_PROBE_PREFIX=prefixes[0], SAMPLE_READY=ready_file or ""))
        started.append(process)
        probed_processes.append(process)
        return process

    def lines_in(path):
        try:
            with open(path) as handle:
                return [line for line in handle.read().split("\n") if line]
        except OSError:
            return []

    try:
        # 共用 TMPDIR：bash -c 那一层（参数不以自证的参数收尾，不是只包着它的）与自证同一个 TMPDIR
        start(["bash", "-c", term_forwarding('bash "$0" --selftest') + ":", script], shared_base, ready["shared"])
        # 单给 TMPDIR：自检进程经 timeout 起（只是包着它的一层，与它同一个 TMPDIR，像 light-selftest-guard.py 底下的 strace），自证的 TMPDIR 与自检进程的不同
        start(["timeout", "60", "bash", script, "--selftest"], private_base, ready["private"])
        long_run = start(["bash", "-c", 'exec 3>>"$0"; exec sleep 60', plain], shared_base)
        launched = time.time()
        wait_until(lambda: all(len(lines_in(path)) >= 3 for path in ready.values()) and (process_arguments(long_run.pid) or [""])[0] == "sleep")
        time.sleep(max(0.0, thresholds.process_report_minutes * 60 + 0.5 - (time.time() - launched)))   # 跑满 --process-report-minutes 才数
        shared_files, private_files = lines_in(ready["shared"]), lines_in(ready["private"])
        with library.selftest_registry_swapped(registry, registered_root):
            lines, found = process_alerts(os.getpid(), set(), thresholds)
        reds = [explanation for name, explanation, _ in found if name == VERDICT_RED_ALERT]
        quiet_lines = [line for line in lines if "的临时目录里，✗ 不报）" in line]
        shown = f"告警 {reds}，报告 {[line for line in lines if '判定行' in line]}"

        def reported(path):
            return any(os.path.realpath(path) in explanation for explanation in reds)

        def quiet(path):
            return any(os.path.realpath(path) in line for line in quiet_lines)

        exempted = shared_files[:2] + private_files[2:3]
        if len(shared_files) < 3 or len(private_files) < 3 or any(reported(path) or not quiet(path) for path in exempted):
            failures.append(f"登记成轻的自证在自己临时目录里写的 ✗ 不该报：共用 TMPDIR 的自证 mktemp 起的与 lib_selftest_scratch.py 探查目录前缀起头的两份、"
                            f"单给 TMPDIR 的自证 sample-wheel-selftest.XXXXXX 里那一份 {exempted} 应当只在报告里各写一行「… 的临时目录里，✗ 不报」，实际 {shown}")
        direct = os.path.join(shared_base, "selftest-direct.log")
        if not any(os.path.realpath(direct) in explanation and "自证 delta 真红了" in explanation for explanation in reds):
            failures.append(f"共用 TMPDIR 的自证直接写在 TMPDIR 底下的 ✗ 应当报：{direct} 不在它的哪个临时目录里，实际 {shown}")
        if not any(os.path.realpath(plain) in explanation and "长跑 epsilon 判红" in explanation for explanation in reds):
            failures.append(f"没有自证上层的进程写在像 mktemp 的临时目录里的 ✗ 应当报：{plain}（进程 {long_run.pid} 开着写），实际 {shown}")
    finally:
        # 只停自己起的三个进程：bash -c 停它起的自证、自证停它起的 sleep（term_forwarding），timeout 停它自己那一组
        stop_own_processes(started)
    return 5   # 断言过的文件：不报的三份、照报的两份


def selftest_memory(work, failures, probed_processes, watch_runs):
    """「进程常驻内存过线」的自检：拿自检进程当 Claude 实例，阈值压到 0.125 GiB。
    一个写满 256 MiB 的进程报（归属认出开着输出文件的子 agent memowner）、一个只睡的小进程不报，阈值 0 不查；
    --processes-only 与 --agents 两条路整查都报、退 3，--ack 进程:<pid> 之后不叫醒、到点退 4；
    两次整查之间的单查：复检间隔 240 秒、看门狗起了之后才冒出来的过线进程，几秒之内就叫醒（不等到回报时刻那次整查）。返回起了几个进程。"""
    memory_session = os.path.join(work, "memory-session")
    owned_output = os.path.join(work, "memowner.output")
    write_transcript(memory_session, "memowner", [
        bash_use(1, "t1", "cargo test --release", "m1"),
        record_at(0.95, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                  "content": f"Command running in background with ID: bm1. Output is being written to: {owned_output}. "
                                             "You will be notified when it completes."}]),
        record_at(0.9, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
        {"agentType": "experiment-runner", "description": "后台起了一个吃内存的测试"})
    thresholds = argparse.Namespace(process_rss_gibibytes=0.125)
    hog_code = "import time\nheld = b'\\x01' * (256 * 1024 * 1024)\nprint('ready', flush=True)\ntime.sleep(120)\n"
    started = []

    def start(argv, stdout=subprocess.DEVNULL):
        process = subprocess.Popen(argv, stdout=stdout, stderr=subprocess.DEVNULL)
        started.append(process)
        probed_processes.append(process)
        return process

    def wait_until_resident(process, minimum_bytes, timeout_seconds=60):
        deadline = time.time() + timeout_seconds
        while time.time() < deadline:
            resident = resident_memory_bytes(process.pid)
            if resident is not None and resident >= minimum_bytes:
                return True
            time.sleep(0.1)
        return False

    def alerted_pids(found):
        return {int(match.group(1)) for name, explanation, _ in found if name == MEMORY_ALERT
                for match in [re.match(r"进程 (\d+) ", explanation)] if match}

    try:
        with open(owned_output, "a") as output_handle:
            holder = start(["sleep", "120"], stdout=output_handle)   # 一直开着 memowner 那个后台任务的输出文件：它在等的任务还活着
            hog = start([sys.executable, "-c", hog_code], stdout=output_handle)
        small = start([sys.executable, "-c", "import time\ntime.sleep(120)\n"])
        if not wait_until_resident(hog, 200 * 1024 * 1024):
            failures.append(f"写满 256 MiB 的进程 {hog.pid} 60 秒里常驻内存没到 200 MiB，这一段自检没法判")
            return len(started)
        lines, found = memory_report(os.getpid(), set(), {memory_session}, thresholds)
        alerted = alerted_pids(found)
        if alerted != {hog.pid}:
            failures.append(f"「{MEMORY_ALERT}」应当只报写满 256 MiB 的进程 {hog.pid}（阈值 0.125 GiB），只睡的 {small.pid} 与 {holder.pid} 不报；"
                            f"实际报了 {sorted(alerted)}，报告：{lines}")
        hog_explanation, hog_next_step = next(((explanation, next_step) for name, explanation, next_step in found
                                               if explanation.startswith(f"进程 {hog.pid} ")), ("", ""))
        if not all(part in hog_explanation for part in ("常驻内存", "阈值 0.125 GiB", "已跑", "子 agent memowner", "memowner.output")):
            failures.append(f"过线进程的告警应当写常驻内存、阈值、已跑多久与归属「子 agent memowner」（开着 memowner.output），实际：{hog_explanation or '没报'}")
        if not all(part in hog_next_step for part in (f"grep VmRSS /proc/{hog.pid}/status", f"{PROCESS_STOP_COMMAND} {hog.pid}",
                                                     "别照原样重派同一件活", "run-with-memory-cap.sh", f"--ack 进程:{hog.pid}")):
            failures.append(f"过线进程的「怎么办」应当写怎么看还涨不涨、按进程号停它的命令、查清之前别照原样重派、带上限重派与 --ack 进程:{hog.pid}，实际：{hog_next_step}")
        if not any(line.startswith(f"{MEMORY_ALERT}：查了") and "报 1 条" in line and f"进程 {hog.pid} " in line for line in lines):
            failures.append(f"报告里应当有一行「{MEMORY_ALERT}：查了 … 常驻内存最大的是进程 {hog.pid} … 报 1 条」，实际 {lines}")
        # 阈值逐个进程取（文件头「进程常驻内存过线」①②③）：两处来源换成造出来的，只看 hog 那一个进程
        def hog_alerted(wrapper_of, peak_of):
            found_lines, found_alerts = memory_report(os.getpid(), set(), {memory_session}, argparse.Namespace(process_rss_gibibytes=0.125, process_rss_cap_percent=90),
                                                      wrapper_of=wrapper_of, peak_of=peak_of)
            return hog.pid in alerted_pids(found_alerts), found_lines, found_alerts
        no_peak = lambda pid: None   # noqa: E731
        for cap_text, cap_bytes, wanted in (("1G", 1024 ** 3, False), ("128M", 128 * 1024 ** 2, True)):
            in_wrapper = (lambda pid, cap_text=cap_text, cap_bytes=cap_bytes:   # noqa: E731
                          ("singlefs-memory-cap-selftest", cap_text, cap_bytes) if pid == hog.pid else None)
            got, found_lines, found_alerts = hog_alerted(in_wrapper, no_peak)
            if got != wanted or not any("照内存包装登记的上限 1 个" in line for line in found_lines) \
                    or (wanted and not any(f"内存包装 singlefs-memory-cap-selftest 登记的上限 {cap_text} 的 90%" in explanation for _, explanation, _ in found_alerts)):
                failures.append(f"内存包装里的进程应当照包装上限判：上限 {cap_text} 的 90% 时写满 256 MiB 的进程 {hog.pid} {'应当' if wanted else '不该'}报，"
                                f"实际{'报了' if got else '没报'}，报告 {found_lines}，告警 {found_alerts}")
        for peak_bytes, wanted in ((512 * 1024 ** 2, False), (64 * 1024 ** 2, True)):
            got, found_lines, found_alerts = hog_alerted(lambda pid: None, lambda pid, peak_bytes=peak_bytes: peak_bytes if pid == hog.pid else None)
            if got != wanted or not any("照峰值表 1 个" in line for line in found_lines) \
                    or (wanted and not any("峰值表里这条命令的峰值" in explanation for _, explanation, _ in found_alerts)):
                failures.append(f"峰值表里有这条命令的进程应当照峰值判：峰值 {peak_bytes // 1024 ** 2} MiB 的 1.5 倍时写满 256 MiB 的进程 {hog.pid} "
                                f"{'应当' if wanted else '不该'}报，实际{'报了' if got else '没报'}，报告 {found_lines}")
        # 峰值表里不到下限的条目（文件头「进程常驻内存过线」②）：记成 0.00 GiB 的条目（4 KiB）配只睡的小进程，阈值取下限、不报；峰值记成 0 的不当阈值（走 ③）
        tiny_peaks = {small.pid: 4096, holder.pid: 0}
        floor_lines, floor_found = memory_report(os.getpid(), set(), {memory_session}, argparse.Namespace(process_rss_gibibytes=8, process_rss_cap_percent=90),
                                                 attribute_owners=False, wrapper_of=lambda pid: None, peak_of=lambda pid: tiny_peaks.get(pid))
        if alerted_pids(floor_found) & {small.pid, holder.pid} or not any("取下限的 1 个" in line for line in floor_lines):
            failures.append(f"峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报：峰值 4 KiB 的 {small.pid} 阈值应当取下限 {MEMORY_PEAK_FLOOR_MEBIBYTES_DEFAULT} MiB"
                            f"（报告写「取下限的 1 个」）、峰值记成 0 的 {holder.pid} 不当阈值，实际报了 {sorted(alerted_pids(floor_found))}，报告 {floor_lines}")
        # 峰值同样很小、常驻 256 MiB 过了下限（这里下限写 128 MiB）的照下限报
        _, hog_floor_found = memory_report(os.getpid(), set(), {memory_session},
                                           argparse.Namespace(process_rss_gibibytes=8, process_rss_cap_percent=90, process_rss_floor_mebibytes=128),
                                           attribute_owners=False, wrapper_of=lambda pid: None, peak_of=lambda pid: 4096 if pid == hog.pid else None)
        hog_floor_explanation = next((explanation for _, explanation, _ in hog_floor_found if explanation.startswith(f"进程 {hog.pid} ")), "")
        if "不到下限，取下限" not in hog_floor_explanation:
            failures.append(f"峰值很小、常驻过了下限的进程应当照下限报：峰值 4 KiB、--process-rss-floor-mebibytes 128 时写满 256 MiB 的进程 {hog.pid} 应当报、"
                            f"阈值写「… 不到下限，取下限 …」，实际 {hog_floor_explanation or '没报'}")
        same_scope = lambda pid: ("singlefs-memory-cap-whole", "1G", 1024 ** 3)   # noqa: E731  根进程自己也在这个 scope 里
        got, found_lines, _ = hog_alerted(same_scope, no_peak)
        if not got or any("照内存包装登记的上限 1 个" in line for line in found_lines):
            failures.append(f"看门狗的根进程自己也在同一个包装 scope 里时不该照包装上限判（退回 --process-rss-gibibytes），实际{'报了' if got else '没报'}，报告 {found_lines}")
        disabled_lines, disabled_found = memory_report(os.getpid(), set(), {memory_session}, argparse.Namespace(process_rss_gibibytes=0))
        if disabled_found or not any("这一类不查" in line for line in disabled_lines):
            failures.append(f"--process-rss-gibibytes 0 时应当不报、报告里写「这一类不查」，实际告警 {disabled_found}、报告 {disabled_lines}")
        # 整查：--processes-only 与 --agents 两条路都报、退 3；--ack 进程:<pid> 之后不叫醒，到点退 4
        wanted_alert_line = f"⚠️ {MEMORY_ALERT}（进程）：进程 {hog.pid} "
        base = [sys.executable, os.path.abspath(__file__), "watch", "--process-root-pid", str(os.getpid()), "--session-dir", memory_session,
                "--process-rss-gibibytes", "0.125", "--interval-seconds", "1", "--max-minutes", "0.1"]
        for label, extra in (("--processes-only", ["--processes-only"]), ("--agents", ["--agents", "memowner"])):
            watch_runs.append(["memory", label])
            memory_run = subprocess.run(base + extra, capture_output=True, text=True, timeout=120)
            if memory_run.returncode != 3 or wanted_alert_line not in memory_run.stdout or "   → 怎么办：先看它还涨不涨" not in memory_run.stdout:
                failures.append(f"看门狗 {label} 盯着常驻内存过线的进程时应当报「{wanted_alert_line}…」连同出路、退出码 3，"
                                f"实际退出码 {memory_run.returncode}，输出：{memory_run.stdout[-500:]}")
        watch_runs.append(["memory", "--processes-only", "--ack"])
        acknowledged_run = subprocess.run(base + ["--processes-only", "--ack", f"进程:{hog.pid}"], capture_output=True, text=True, timeout=120)
        if acknowledged_run.returncode != 4 or f"已确认接着盯（--ack 进程:{MEMORY_ALERT}）：进程 {hog.pid} " not in acknowledged_run.stdout:
            failures.append(f"过线进程 --ack 进程:{hog.pid} 之后看门狗不该叫醒：应当到点退出码 4 并写「已确认接着盯」，"
                            f"实际退出码 {acknowledged_run.returncode}，输出：{acknowledged_run.stdout[-500:]}")
        hog.kill()
        hog.wait()
        # 两次整查之间的单查：复检间隔 240 秒，过线进程在看门狗做完第一次整查之后才冒出来（按它建的就绪文件等，不按固定秒数：
        # 冒得早了会被第一次整查直接看见，判不出单查），应当由单查叫醒、在回报时刻那次整查之前退出（单查坏了就要等到那次整查）。
        # 回报时刻 90 秒、按就绪文件等：负载高时 python 起得慢，原先「起了 6 秒之后冒出来、20 秒内退」两头都会被起动时间吃掉
        # 两条路的看门狗同时起、都做完第一次整查之后才起那一个过线进程（单查坏了两条各要等满回报时刻，先后起就要等两倍）
        late_watch_seconds = 90
        late_watchers = {}
        for label, extra in (("--processes-only", ["--processes-only"]), ("--agents", ["--agents", "memowner"])):
            watch_runs.append(["memory", label, "单查"])
            late_ready = os.path.join(work, f"ready-memory-{label.strip('-')}")
            late_watchers[label] = (time.time(), late_ready, subprocess.Popen(
                [sys.executable, os.path.abspath(__file__), "watch", "--process-root-pid", str(os.getpid()),
                 "--session-dir", memory_session, "--process-rss-gibibytes", "0.125", "--interval-seconds", "240",
                 "--detection-poll-seconds", "0.5", "--max-minutes", str(late_watch_seconds / 60), *extra],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=dict(os.environ, **{SELFTEST_READY_VARIABLE: late_ready})))
        for _, late_ready, watcher in late_watchers.values():
            wait_for_selftest_ready(late_ready, watcher)
        # 过线进程起来之后至多等 late_alert_seconds 秒（单查每 0.5 秒一次，叫醒之前那次整查负载高时要几秒）：到点还没退的停掉它、判红，不等满回报时刻
        late_alert_seconds = 40
        late_hog = start([sys.executable, "-c", hog_code])
        late_hog_started = time.time()

        def late_outcome(item):
            watch_started, _, watcher = item
            try:
                output, _ = watcher.communicate(timeout=late_alert_seconds)
                return output, time.time() - watch_started, True
            except subprocess.TimeoutExpired:
                watcher.kill()   # 自己起的那一个看门狗（Popen 记着的进程号）
                output, _ = watcher.communicate()
                return output, time.time() - watch_started, False

        with concurrent.futures.ThreadPoolExecutor(max_workers=len(late_watchers)) as pool:
            outcomes = dict(zip(late_watchers, pool.map(late_outcome, late_watchers.values())))
        for label, (_, _, watcher) in late_watchers.items():
            output, elapsed, in_time = outcomes[label]
            if not in_time or watcher.returncode != 3 or f"⚠️ {MEMORY_ALERT}（进程）：进程 {late_hog.pid} " not in output or elapsed >= late_watch_seconds:
                failures.append(f"看门狗 {label} 起了之后才冒出来的过线进程 {late_hog.pid} 应当由两次整查之间的单查在几秒之内叫醒（退出码 3），"
                                f"实际{'退出码 ' + str(watcher.returncode) if in_time else f'过线进程起来 {time.time() - late_hog_started:.0f} 秒还没退、停了它'}、"
                                f"看门狗起了 {elapsed:.1f} 秒，输出：{output[-400:]}")
        late_hog.kill()
        late_hog.wait()
        selftest_memory_under_selftest(work, failures, started, probed_processes, alerted_pids)
    finally:
        for process in started:
            process.kill()
            process.wait()
    return len(started)


def selftest_memory_under_selftest(work, failures, started, probed_processes, alerted_pids):
    """「进程常驻内存过线」里登记成轻的自证底下不判（文件头那一段）的自检：造一个仓根与一份登记表（lib_heavy_tests.py 的 selftest_registry_swapped 换过去），
    scripts/hosts/sample-memory-cap.sh 登记成 light、sample-memory-heavy.sh 登记成 heavy，两份的 --selftest 底下各起一个写满 100 MiB 的探针（阈值压到 0.0625 GiB）：
    light 底下的不报、heavy 底下的照报，报告那一行写出不报的个数。调它时写满 256 MiB 的那几个都已停掉，这一格同时占的至多 200 MiB。"""
    library, problem = heavy_tests_library()
    if library is None:
        failures.append(f"登记成轻的自证底下的过线进程那几格判不了：{problem}")
        return
    tree = os.path.join(work, "memory-selftest-tree")
    scripts_directory = os.path.join(tree, "scripts", "hosts")
    os.makedirs(scripts_directory)
    probe_code = "import sys, time\nheld = b'\\x01' * (100 * 1024 * 1024)\nopen(sys.argv[1], 'w').close()\ntime.sleep(120)\n"
    for name in ("sample-memory-cap.sh", "sample-memory-heavy.sh"):
        with open(os.path.join(scripts_directory, name), "w") as handle:
            # 探针放后台、wait 它（term_forwarding）：bash 不把它 exec 掉，自证那一层一直在；收到 TERM 停它自己起的探针
            handle.write(term_forwarding('"$SAMPLE_PYTHON" -c "$SAMPLE_PROBE_CODE" "$SAMPLE_PROBE_READY"') + "exit 0\n")
    registry = os.path.join(tree, "selftest-registry.tsv")
    with open(registry, "w") as handle:
        handle.write("scripts/hosts/sample-memory-cap.sh\tlight\t# 看门狗自检：--selftest 底下起一个写满 100 MiB 的探针\n"
                     "scripts/hosts/sample-memory-heavy.sh\theavy\t# 看门狗自检：同样的探针，登记成 heavy\n")
    selftests, probes = {}, {}
    try:
        for label, name in (("light", "sample-memory-cap.sh"), ("heavy", "sample-memory-heavy.sh")):
            ready = os.path.join(tree, f"ready-{label}")
            process = subprocess.Popen(["bash", os.path.join(scripts_directory, name), "--selftest"], cwd=tree,
                                       env=dict(os.environ, SAMPLE_PYTHON=sys.executable, SAMPLE_PROBE_CODE=probe_code, SAMPLE_PROBE_READY=ready),
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            started.append(process)
            probed_processes.append(process)
            selftests[label] = (process, ready)
        wait_until(lambda: all(os.path.exists(ready) for _, ready in selftests.values()))
        table = process_table()
        for label, (process, _) in selftests.items():
            probes[label] = next((pid for pid, info in table.items() if info["parent"] == process.pid), None)
        if not all(probes.values()) or not all(wait_until(lambda pid=pid: (resident_memory_bytes(pid) or 0) >= 90 * 1024 * 1024, 60)
                                               for pid in probes.values()):
            failures.append(f"登记成轻的自证底下的过线进程那几格没法判：两份 --selftest 底下写满 100 MiB 的探针 {probes} 60 秒里没起来或常驻没到 90 MiB")
            return
        with library.selftest_registry_swapped(registry, tree):
            lines, found = memory_report(os.getpid(), set(), set(), argparse.Namespace(process_rss_gibibytes=0.0625), attribute_owners=False,
                                         wrapper_of=lambda pid: None, peak_of=lambda pid: None)
        alerted = alerted_pids(found)
        shown = f"探针 {probes}，自证 {({label: process.pid for label, (process, _) in selftests.items()})}，实际报了 {sorted(alerted)}，报告：{lines}"
        if probes["light"] in alerted:
            failures.append(f"登记成轻的自证底下的过线进程不该报：sample-memory-cap.sh --selftest 底下写满 100 MiB 的探针（阈值 0.0625 GiB）；{shown}")
        if probes["heavy"] not in alerted:
            failures.append(f"不在登记成轻的自证底下的过线进程应当照报：登记成 heavy 的 sample-memory-heavy.sh --selftest 底下写满 100 MiB 的探针；{shown}")
        if not any(re.search(r"登记成轻的自证底下过线的 [1-9]\d* 个不报", line) for line in lines):
            failures.append(f"报告那一行应当写出登记成轻的自证底下过线不报的进程数（这里至少 1 个），实际 {lines}")
    finally:
        # 只停自己起的两份自证（bash）：探针是它们的子进程，各由它的 trap 停（term_forwarding）
        stop_own_processes([process for process, _ in selftests.values()])


def selftest_core_usage_aggregated(work, failures, probed_processes):
    """「长跑在用核数过少」按变异门禁那一趟合计判的自检（文件头「长跑在用核数过少」一的末段）：每台登记线程数临时设成 8（SINGLEFS_THREAD_CAP，阈值 2 核），
    窗口压到 2 秒。临时目录里造一份假的 .claude/gate.d/checker-tier-crates-mutation-replay.sh（一段 python）：起 FAKE_WORKER_COUNT 个干活进程（argv[0] 照 checker 档包的测试二进制起名），
    收到 TERM 停掉自己起的那几个再退。
    两趟都照变异轻型的起法带 --quick（classify_process 不把它算重活）：busy 一趟八个干活进程各忙一条线（busy-load 1 条线）——合计约 8 核、同时在算 8 条，不低于阈值，不报；逐个单行判（弄坏开关 mutationrows）每一行都报；
    （八个不是三个：机器被别的活占着时每条线分到的不到一核，三条合计会掉到阈值 2 以下、这一格跟着负载时红时不红；八条在负载是核数两倍时合计还有 4 核上下）；
    idle 一趟三个干活进程都只睡——合计 0 核，照报，报在门禁那个进程上、写「各干活进程合计」，干活进程不单报；同一段假脚本放在 scripts/gate/test-targets-parallel.py
    （AGGREGATED_PARALLEL_SCRIPTS）再起一趟只睡的，同样报在它上面。收尾只对自己起的三个假进程发 TERM。返回起了几个进程。"""
    core, problem = core_usage_library()
    if core is None:
        failures.append(f"「{CORE_USAGE_ALERT}」按变异门禁合计判那一段自检没法判：{problem}")
        return 0
    root = os.path.join(work, "core-usage-aggregated")
    gate_directory = os.path.join(root, ".claude", "gate.d")
    deps = os.path.join(root, "target", "release", "deps")
    os.makedirs(gate_directory, exist_ok=True)
    os.makedirs(deps, exist_ok=True)
    fake_binary = os.path.join(deps, "crash_enumeration_new_pool_file_creation_stream-0123456789abcdef")
    fake_stage = os.path.join(gate_directory, sorted(AGGREGATED_PARALLEL_STAGES)[0])
    fake_script = os.path.join(root, sorted(AGGREGATED_PARALLEL_SCRIPTS)[0])   # 合入后验证测试那一步按测试目标并行起的那一份，同样合计判
    os.makedirs(os.path.dirname(fake_script), exist_ok=True)
    # 假门禁是一段 python（文件名照门禁起，命令的第二个词就是 .claude/gate.d/ 下那份脚本）：起三个干活进程，收到 TERM 就把自己起的三个停掉再退；假脚本同一段
    fake_source = ("import os, signal, subprocess, sys\n"
                   "busy = os.environ['FAKE_WORKER_KIND'] == 'busy'\n"
                   "workers = [subprocess.Popen([os.environ['FAKE_BINARY'], os.environ['CORE_USAGE_LIBRARY'], 'busy-load', '1', '60'] if busy\n"
                   "                            else [os.environ['FAKE_BINARY'], '60'], executable=sys.executable if busy else 'sleep')\n"
                   "           for _ in range(int(os.environ['FAKE_WORKER_COUNT']))]\n"
                   "def stop(*_):\n"
                   "    for worker in workers:\n"
                   "        worker.terminate()\n"
                   "    os._exit(0)   # 信号打断的是下面的 wait()：处理函数里再 wait() 会撞上 Popen 自己的锁，不收尸直接退，三个由 init 收\n"
                   "signal.signal(signal.SIGTERM, stop)\n"
                   "for worker in workers:\n"
                   "    worker.wait()\n")
    for fake_path in (fake_stage, fake_script):
        with open(fake_path, "w", encoding="utf-8") as handle:
            handle.write(fake_source)
    overrides = {"SINGLEFS_CORE_USAGE_WINDOW_SECONDS": "2", "SINGLEFS_THREAD_CAP": "8",
                 "SINGLEFS_CORE_USAGE_ALERTS": os.path.join(root, "alerts.jsonl")}
    saved = {name: os.environ.get(name) for name in overrides}
    samples_root = os.path.join(root, "state")
    started = []
    try:
        os.environ.update(overrides)
        registered = core.registered_threads()
        environment = dict(os.environ, **{HEAVY_TEST_VARIABLE: "commit", "FAKE_BINARY": fake_binary, "CORE_USAGE_LIBRARY": CORE_USAGE_LIBRARY_PATH})
        environment.pop("CORE_USAGE_BREAK", None)
        gates, worker_counts = {}, {"busy": 8, "idle": 3, "idle-script": 3}
        for kind, path in (("busy", fake_stage), ("idle", fake_stage), ("idle-script", fake_script)):
            # 照变异轻型的起法带 --quick：classify_process 不把它算重活，改前就往下拆到干活进程逐个判
            gate = subprocess.Popen([sys.executable, path, "--quick"], env=dict(environment, FAKE_WORKER_KIND=kind.split("-")[0],
                                                                                FAKE_WORKER_COUNT=str(worker_counts[kind])), cwd=root,
                                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            started.append(gate)
            probed_processes.append(gate)
            gates[kind] = gate
        # 等两趟的干活进程都起来（busy 那一趟真在算），先取一次样，隔 2.3 秒（窗口 2 秒）再整查
        deadline = time.time() + SELFTEST_READY_TIMEOUT_SECONDS
        while time.time() < deadline:
            table = process_table()
            workers = {kind: [pid for pid, info in table.items() if info["parent"] == gate.pid] for kind, gate in gates.items()}
            busy_sample = core.tree_sample(gates["busy"].pid)
            if all(len(workers[kind]) == count for kind, count in worker_counts.items()) and busy_sample and busy_sample["cpu"] >= 0.1:
                break
            time.sleep(0.1)
        core_usage_report(os.getpid(), set(), set(), samples_root)
        time.sleep(2.3)
        lines, found = core_usage_report(os.getpid(), set(), set(), samples_root)
        reported = [explanation for name, explanation, _ in found if name == CORE_USAGE_ALERT]
        busy_pids = {gates["busy"].pid, *workers["busy"]}
        busy_alerts = [explanation for explanation in reported if any(explanation.startswith(f"进程 {pid} ") for pid in busy_pids)]
        if busy_alerts:
            failures.append(f"变异门禁这一趟八个干活进程各忙一条线（合计约 8 核，登记 {registered}、阈值 {registered / 4:g}）应当按那一趟的合计判、不报，"
                            f"实际报了 {busy_alerts}（AGENT_WATCH_BREAK=mutationrows 设着的话这里本来就该红）；报告：{lines}")
        for kind, subject in (("idle", "变异门禁"), ("idle-script", "按测试目标并行起的脚本")):
            idle_gate_alert = next((explanation for explanation in reported if explanation.startswith(f"进程 {gates[kind].pid} ")), None)
            idle_worker_alerts = [explanation for explanation in reported if any(explanation.startswith(f"进程 {pid} ") for pid in workers[kind])]
            if not idle_gate_alert or "各干活进程合计" not in idle_gate_alert or idle_worker_alerts:
                failures.append(f"{subject}这一趟三个干活进程都只睡（合计 0 核）应当报在{subject}那个进程 {gates[kind].pid} 上、写「各干活进程合计」，干活进程不单报；"
                                f"实际那一条 {idle_gate_alert or '没报'}、干活进程 {idle_worker_alerts}；报告：{lines}")
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        for gate in started:
            gate.terminate()   # 假门禁收到 TERM 先停自己起的那几个干活进程再退
            try:
                gate.wait(timeout=60)
            except subprocess.TimeoutExpired:
                gate.kill()
                gate.wait()
    return len(started) + sum(worker_counts.values())


def selftest_core_usage(work, failures, probed_processes, watch_runs):
    """「长跑在用核数过少」的自检：拿自检进程当 Claude 实例，窗口压到 2 秒（SINGLEFS_CORE_USAGE_WINDOW_SECONDS），告警记录写临时目录。
    起三个假的重活进程（argv[0] 照 checker 档包的测试二进制起名，里面是 scripts/hosts/core_usage.py busy-load；带 SINGLEFS_HEAVY_TESTS=commit，「重型测试没带前缀」不报）：
    1 条线程忙等的报「重活只用了 X 核（登记 N）」；登记数那么多条线程忙等的照判器拿同一串样本判出的结论报（判成过少才报，机器不忙时就是不报，不看此刻负载）；
    环境里带 SINGLEFS_SERIAL_DECLARATIONS、声明了串行的照报并标出理由。
    先取一次样、隔 2.3 秒再整查（起动那一段的 CPU 不进窗口）。告警记录：本会话的、告警进程还活着的报；别的会话的、过期的、告警进程已不在的不报；没有会话号的只记一行。
    --processes-only 看门狗真跑：1 条线程那个进程报、退 3；都 --ack 进程:<pid> 之后到点退 4。返回起了几个进程。"""
    core, problem = core_usage_library()
    if core is None:
        failures.append(f"「{CORE_USAGE_ALERT}」这一段自检没法判：{problem}")
        return 0
    registered = core.registered_threads()
    deps = os.path.join(work, "core-usage", "target", "release", "deps")
    os.makedirs(deps, exist_ok=True)
    fake_binary = os.path.join(deps, "crash_enumeration_new_pool_file_creation_stream-0123456789abcdef")   # python 顶着这个 argv[0] 跑 busy-load
    declarations = os.path.join(work, "core-usage", "serial-declarations")
    with open(declarations, "w") as handle:
        handle.write(f"{time.time() - 1:.3f}\t此刻串行：计时窗口必须单线程跑完\n")
    alerts_file = os.path.join(work, "core-usage", "alerts.jsonl")
    samples_root = os.path.join(work, "core-usage", "state")
    core_session = os.path.join(work, "core-session")
    os.makedirs(core_session, exist_ok=True)
    session_id = os.path.basename(core_session)
    overrides = {"SINGLEFS_CORE_USAGE_WINDOW_SECONDS": "2", "SINGLEFS_CORE_USAGE_ALERTS": alerts_file}
    saved = {name: os.environ.get(name) for name in overrides}
    started = []

    def start(threads, extra=None, argv=None):
        environment = dict(os.environ, **{HEAVY_TEST_VARIABLE: "commit"}, **(extra or {}))
        environment.pop("CORE_USAGE_BREAK", None)
        process = subprocess.Popen(argv or [fake_binary, CORE_USAGE_LIBRARY_PATH, "busy-load", str(threads), "60"], executable=None if argv else sys.executable,
                                   env=environment, cwd=work, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        started.append(process)
        probed_processes.append(process)
        return process

    def alert_for(found, pid):
        return next((explanation for name, explanation, _ in found if name == CORE_USAGE_ALERT and explanation.startswith(f"进程 {pid} ")), None)

    try:
        os.environ.update(overrides)
        one = start(1)
        many = start(registered)
        declared = start(1, {"SINGLEFS_SERIAL_DECLARATIONS": declarations})
        sleeper = start(0, argv=["sleep", "120"])
        # 等三个都起来、真在算（机器忙时 python 起得慢，不按固定秒数等），先取一次样，隔 2.3 秒（窗口 2 秒）再整查
        deadline = time.time() + SELFTEST_READY_TIMEOUT_SECONDS
        while time.time() < deadline:
            samples = [core.tree_sample(process.pid) for process in (one, many, declared)]
            if all(sample and sample["cpu"] >= 0.05 for sample in samples):
                break
            time.sleep(0.1)
        core_usage_report(os.getpid(), set(), {session_id}, samples_root)
        time.sleep(2.3)
        lines, found = core_usage_report(os.getpid(), set(), {session_id}, samples_root)
        one_alert, many_alert, declared_alert = (alert_for(found, process.pid) for process in (one, many, declared))
        if not one_alert or "重活只用了" not in one_alert or f"核（登记 {registered}）" not in one_alert:
            failures.append(f"1 条线程忙等的重活进程 {one.pid} 应当报「{CORE_USAGE_ALERT}」、写「重活只用了 X 核（登记 {registered}）」，实际：{one_alert or '没报'}；报告：{lines}")
        # 登记数那么多条线程忙等的那个报不报，照判器拿这一次整查用的同一串样本判出的结论核（不看此刻负载：机器被别的活占满时它真会掉到阈值以下，那时报才对）；
        # 判器在机器忙时判得对不对，由 scripts/hosts/core_usage.py 自己的自证拿造的样本序列核。没取到样本的（这一类整个没查）照旧要求不报
        many_table = process_table()
        many_children = {}
        for pid, info in many_table.items():
            many_children.setdefault(info["parent"], []).append(pid)
        many_samples = (core.read_series(os.path.join(samples_root, CORE_USAGE_SAMPLES_DIRECTORY, f"{many.pid}-{many_table[many.pid]['start_ticks']}.jsonl"))
                        if many.pid in many_table else [])
        many_verdict = None
        if many_samples:
            many_entries, many_problems = core.read_declarations(serial_declarations_of_tree(many.pid, many_children))
            many_verdict = core.judge_series(many_samples, many_entries, many_problems, registered, core.window_seconds()[0])
        if many_verdict is not None and many_verdict["status"] == "low":
            if not many_alert or f"只用了 {many_verdict['cores']:.1f} 核（登记 {registered}）" not in many_alert:
                failures.append(f"{registered} 条线程忙等的重活进程 {many.pid} 照同一串样本判器判成过少（在用 {many_verdict['cores']:.1f} 核、"
                                f"同时在算的线程约 {many_verdict['parallel']:.1f} 条），应当报「{CORE_USAGE_ALERT}」、写「只用了 {many_verdict['cores']:.1f} 核（登记 {registered}）」，"
                                f"实际：{many_alert or '没报'}")
        elif many_alert:
            failures.append(f"{registered} 条线程忙等的重活进程 {many.pid} 不该报「{CORE_USAGE_ALERT}」（照同一串样本判器的结论是 "
                            f"{many_verdict['status'] if many_verdict else '没取到样本'}），实际：{many_alert}")
        if not declared_alert or "声明了串行" not in declared_alert or "计时窗口必须单线程跑完" not in declared_alert:
            failures.append(f"声明了串行的重活进程 {declared.pid} 应当照报并标出理由「计时窗口必须单线程跑完」，实际：{declared_alert or '没报'}")
        if not any(line.startswith(f"{CORE_USAGE_ALERT}：查了这个 Claude 实例底下跑过 2 秒的重活 3 个") for line in lines):
            failures.append(f"报告里应当有一行「{CORE_USAGE_ALERT}：查了这个 Claude 实例底下跑过 2 秒的重活 3 个…」，实际 {lines}")
        # 告警记录：本会话的、告警进程还活着的报；别的会话的、过期的、告警进程已不在的不报；没有会话号的只记一行
        now = time.time()
        gone = subprocess.Popen(["true"])
        gone.wait()
        for record in ({"time": now, "session_id": session_id, "source": "multi-host-run.sh --peer", "label": "fresh", "alert_pid": sleeper.pid,
                        "line": "✗ 第二台在用 1.0 核（登记 31）", "next_step": "停下它"},
                       {"time": now, "session_id": "another-session", "source": "multi-host-run.sh --peer", "label": "other", "alert_pid": sleeper.pid,
                        "line": "✗ 别的会话", "next_step": "停下它"},
                       {"time": now - 3600, "session_id": session_id, "source": "detached-run.sh watch", "label": "stale", "alert_pid": sleeper.pid,
                        "line": "✗ 过期的", "next_step": "停下它"},
                       {"time": now, "session_id": session_id, "source": "detached-run.sh watch", "label": "gone", "alert_pid": gone.pid,
                        "line": "✗ 告警进程已不在", "next_step": "停下它"},
                       {"time": now, "session_id": "", "source": "detached-run.sh watch", "label": "nosession", "alert_pid": sleeper.pid,
                        "line": "✗ 没有会话号", "next_step": "停下它"}):
            core.append_line(alerts_file, json.dumps(record, ensure_ascii=False))
        lines, found = core_usage_report(os.getpid(), set(), {session_id}, samples_root)
        record_alerts = [explanation for name, explanation, _ in found if name == CORE_USAGE_ALERT and explanation.startswith(f"进程 {sleeper.pid} ")]
        if record_alerts != [f"进程 {sleeper.pid} multi-host-run.sh --peer（fresh）：✗ 第二台在用 1.0 核（登记 31）"] \
                or alert_for(found, gone.pid) or not any("没有会话号，只记不叫醒" in line and "✗ 没有会话号" in line for line in lines):
            failures.append(f"告警记录应当只把本会话、没过期、告警进程还活着的那一条（fresh）报出来，没有会话号的只记一行；"
                            f"实际告警 {record_alerts}、已不在的 {alert_for(found, gone.pid)}，报告：{lines}")
        # 内存包装排队时写的（run-with-memory-cap.sh 文件头「排队时去找别的机器」）：queued 而包装活着的报、refused 而包装不在了的也报，
        # queued 之后补了 admitted 的不报、queued 而包装不在了的不报；告警名是「本机内存排队、别的机器放得下」，出路是记录里那一句
        def queue_record(label, phase, pid, offset=0.0):
            return {"time": now + offset, "session_id": session_id, "source": "run-with-memory-cap.sh", "kind": MEMORY_QUEUE_RECORD_KIND, "phase": phase,
                    "label": label, "alert_pid": pid, "line": f"→ 第二台此刻放得下（{label}）", "next_step": f"改用 multi-host-run.sh --peer（{label}）"}
        with open(alerts_file, encoding="utf-8") as handle:
            records_before_queue = handle.read()
        for record in (queue_record("queued-live", "queued", sleeper.pid), queue_record("refused-gone", "refused", gone.pid),
                       queue_record("admitted-later", "queued", sleeper.pid), queue_record("admitted-later", "admitted", sleeper.pid, 0.5),
                       queue_record("queued-gone", "queued", gone.pid)):
            core.append_line(alerts_file, json.dumps(record, ensure_ascii=False))
        lines, found = core_usage_report(os.getpid(), set(), {session_id}, samples_root)
        queue_alerts = sorted((name, explanation, next_step) for name, explanation, next_step in found if "run-with-memory-cap.sh" in explanation)
        wanted_queue_alerts = sorted([(MEMORY_QUEUE_ALERT, f"进程 {sleeper.pid} run-with-memory-cap.sh（queued-live）：→ 第二台此刻放得下（queued-live）",
                                       "改用 multi-host-run.sh --peer（queued-live）"),
                                      (MEMORY_QUEUE_ALERT, f"进程 {gone.pid} run-with-memory-cap.sh（refused-gone）：→ 第二台此刻放得下（refused-gone）",
                                       "改用 multi-host-run.sh --peer（refused-gone）")])
        if queue_alerts != wanted_queue_alerts:
            failures.append(f"内存包装排队时写的告警记录应当报成「{MEMORY_QUEUE_ALERT}」：queued 而包装活着的（queued-live）、refused 而包装不在了的（refused-gone）报，"
                            f"补了 admitted 的（admitted-later）与 queued 而包装不在了的（queued-gone）不报；实际 {queue_alerts}，报告：{lines}")
        # 下面真跑的看门狗读同一份告警记录，--ack 那一格只确认四个进程：这几条写回去之前的样子
        with open(alerts_file, "w", encoding="utf-8") as handle:
            handle.write(records_before_queue)
        # 内存包装收尾时写的三种（run-with-memory-cap.sh 文件头「峰值表」一段）：包装退了也报，三种告警名各不相同；别的会话的不报，没有会话号的只记一行
        def outcome_record(kind, label, session=session_id, **fields):
            return {"time": now, "session_id": session, "source": "run-with-memory-cap.sh", "kind": kind, "label": label, "alert_pid": gone.pid,
                    "key": f"selftest-{label}", "cap": "200M", "booked": 20 * 1024 ** 2, "booked_source": "peak-table", "measured": 60 * 1024 ** 2,
                    "by_peak_table": True, "line": f"run-with-memory-cap 写的那一行（{label}）", "next_step": f"记录里的出路（{label}）", **fields}
        # 峰值涨了的两张：grew-gone 实测 120 MiB 过了上限 200M 的一半，叫醒；grew-below-half 涨 30 倍、实测 60 MiB 不到一半，只记一行（门槛见 memory_peak_below_alert_floor）
        for record in (outcome_record("memory-peak-grew", "grew-gone", ratio=3.0, booked=40 * 1024 ** 2, measured=120 * 1024 ** 2),
                       outcome_record("memory-peak-grew", "grew-below-half", ratio=30.0, booked=2 * 1024 ** 2, measured=60 * 1024 ** 2),
                       outcome_record("memory-own-cap", "own-cap-gone"),
                       outcome_record("memory-squeezed", "squeezed-gone", by_peak_table=False), outcome_record("memory-own-cap", "own-cap-other", session="another-session"),
                       outcome_record("memory-peak-grew", "grew-nosession", session="", ratio=3.0)):
            core.append_line(alerts_file, json.dumps(record, ensure_ascii=False))
        lines, found = core_usage_report(os.getpid(), set(), {session_id}, samples_root)
        outcome_alerts = {explanation.split("（", 2)[1].split("）", 1)[0]: (name, explanation, next_step)
                          for name, explanation, next_step in found if "run-with-memory-cap.sh" in explanation and "selftest-" in explanation}
        grew = outcome_alerts.get("grew-gone")
        if not grew or grew[0] != MEMORY_PEAK_GREW_ALERT or "内存峰值比记账大了 3.0 倍" not in grew[1] or "记账 40 MiB" not in grew[1] or "实测 120 MiB" not in grew[1] \
                or grew[2] != "记录里的出路（grew-gone）" or "grew-nosession" in outcome_alerts \
                or not any("没有会话号，只记不叫醒" in line and "grew-nosession" in line for line in lines):
            failures.append(f"内存包装收尾时写的峰值涨了的告警记录（memory-peak-grew）应当在包装退了之后报成「{MEMORY_PEAK_GREW_ALERT}」、写「内存峰值比记账大了 3.0 倍」与记账、实测，"
                            f"出路是记录里那一句，没有会话号的只记一行；实际 {grew}，报告：{lines}")
        below_half_lines = [line for line in lines if "grew-below-half" in line]
        if "grew-below-half" in outcome_alerts or len(below_half_lines) != 1 or "只记进峰值表、不叫醒" not in below_half_lines[0] \
                or "实测 60 MiB 不到上限 200M 的 50%" not in below_half_lines[0]:
            failures.append(f"实测涨 30 倍但不到上限一半的峰值涨了的告警记录（memory-peak-grew，记账 2 MiB、实测 60 MiB、上限 200M）应当只在报告里记一行"
                            f"「实测 60 MiB 不到上限 200M 的 50%，只记进峰值表、不叫醒」、不报告警；实际告警 {outcome_alerts.get('grew-below-half')}，"
                            f"报告里提到它的行 {below_half_lines}（AGENT_WATCH_BREAK=memorygrewfloor 设着的话这里本来就该红）")
        own_cap, squeezed = outcome_alerts.get("own-cap-gone"), outcome_alerts.get("squeezed-gone")
        if not own_cap or own_cap[0] != MEMORY_OWN_CAP_ALERT or "撞了自己的上限：上限给小了，或工作量变了" not in own_cap[1] or "按峰值表放进来的" not in own_cap[1] \
                or not squeezed or squeezed[0] != MEMORY_SQUEEZED_ALERT or "被总上限挤掉：结果不算数，重跑" not in squeezed[1] or "不是按峰值表放进来的" not in squeezed[1] \
                or own_cap[0] == squeezed[0] or "own-cap-other" in outcome_alerts:
            failures.append(f"内存包装收尾时写的撞顶与被挤掉的告警记录应当在包装退了之后分开报：memory-own-cap 报成「{MEMORY_OWN_CAP_ALERT}」（上限给小了，或工作量变了），"
                            f"memory-squeezed 报成「{MEMORY_SQUEEZED_ALERT}」（结果不算数，重跑），别的会话的不报；实际 {own_cap}、{squeezed}，报告：{lines}")
        with open(alerts_file, "w", encoding="utf-8") as handle:
            handle.write(records_before_queue)
        # 看门狗真跑：--processes-only 报 1 条线程那个、退 3；三个都 --ack 之后到点退 4
        environment = dict(os.environ, AGENT_WATCH_STATE_ROOT=os.path.join(work, "core-watch-state"))
        base = [sys.executable, os.path.abspath(__file__), "watch", "--processes-only", "--process-root-pid", str(os.getpid()),
                "--session-dir", core_session, "--interval-seconds", "1", "--max-minutes", "0.1"]
        watch_runs.append(["core-usage", "--processes-only"])
        reported = subprocess.run(base, capture_output=True, text=True, timeout=120, env=environment)
        wanted = f"⚠️ {CORE_USAGE_ALERT}（进程）：进程 {one.pid} 重活只用了"
        if reported.returncode != 3 or wanted not in reported.stdout or "   → 怎么办：装置里互相独立的单位" not in reported.stdout:
            failures.append(f"看门狗 --processes-only 盯着 1 条线程忙等的重活进程时应当报「{wanted}…」连同出路、退出码 3，"
                            f"实际退出码 {reported.returncode}，输出：{reported.stdout[-600:]}")
        watch_runs.append(["core-usage", "--processes-only", "--ack"])
        # 登记数那么多条线程那个也确认上：看门狗子进程没有自检进程存的样本，按起跑以来平均判，机器极忙时起动那一段可能把它判低（那一半在上面的整查里判过了）
        acknowledged = subprocess.run(base + [f"--ack=进程:{process.pid}" for process in (one, many, declared, sleeper)],
                                      capture_output=True, text=True, timeout=120, env=environment)
        if acknowledged.returncode != 4 or f"已确认接着盯（--ack 进程:{CORE_USAGE_ALERT}）：进程 {one.pid} " not in acknowledged.stdout:
            failures.append(f"「{CORE_USAGE_ALERT}」的进程都 --ack 进程:<pid> 之后看门狗不该叫醒：应当到点退出码 4 并写「已确认接着盯」，"
                            f"实际退出码 {acknowledged.returncode}，输出：{acknowledged.stdout[-600:]}")
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        for process in started:
            if process.poll() is None:
                process.kill()
            process.wait()
    return len(started)


def selftest_unacknowledged_terminations(work, failures, probed_processes, watch_runs):
    """调度方不响应就终止：合成的状态目录里同一件告警连报到阈值、没 --ack，调了假的 proc.py stop（记录调用的桩）并写了那一句；
    有 --ack 的、阈值 0 的、中间断过一次的不终止；子 agent 的工具调用过长到线只报；两件同时到线一次只终止一个；
    再真跑三次 --processes-only 看门狗，第三次叫醒时终止那个不写也不算的进程。返回查了几格。"""
    global PROCESS_STOP_SCRIPT
    checks = 0
    stub_calls = os.path.join(work, "terminate-stub-calls.log")
    stub = os.path.join(work, "terminate-stub-proc.py")
    with open(stub, "w", encoding="utf-8") as handle:
        handle.write("import sys\n"
                     f"open({stub_calls!r}, 'a', encoding='utf-8').write(' '.join(sys.argv[1:]) + '\\n')\n"
                     "print(f'已停 {sys.argv[-1]}（自证的桩，没发信号）')\n")
    def stub_lines():
        return open(stub_calls, encoding="utf-8").read().splitlines() if os.path.isfile(stub_calls) else []
    # 标准输出、标准错误接到 /dev/null：继承来的输出文件（自证的日志）一直不涨，⑦ 真跑时它们会被当成「进程无输出」先终止掉
    sleeper = subprocess.Popen(["sleep", "600"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    other = subprocess.Popen(["sleep", "600"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    probed_processes += [sleeper, other]
    saved_script = PROCESS_STOP_SCRIPT
    PROCESS_STOP_SCRIPT = stub
    try:
        def stuck(pid, name="进程无输出"):
            return ("进程", name, f"进程 {pid} 已跑 1 小时，写着的文件 1 小时没变，2 秒里 CPU 也没涨：sleep 600", "主 agent 看它在等什么")
        def run_reports(label, rounds, threshold=3):
            path = os.path.join(work, "terminate-state", label, REPEATED_REPORTS_FILE)
            return path, [terminate_unacknowledged_repeats(path, alerts, acknowledged, threshold) for alerts, acknowledged in rounds]
        # ① 连报到阈值、没 --ack：第 3 次叫醒时经桩终止，写「终止了 <pid>（进程无输出，连报 3 次没处置）」，前两次不终止
        checks += 1
        open(stub_calls, "w").close()
        path, outputs = run_reports("reached", [([stuck(sleeper.pid)], [])] * 3)
        wanted = f"终止了 {sleeper.pid}（进程无输出，连报 3 次没处置）"
        if stub_lines() != [f"stop {sleeper.pid}"] or not any(line.startswith(wanted) for line in outputs[2]) or any(outputs[0] + outputs[1]):
            failures.append(f"连报到阈值：同一件「进程无输出」连报 3 次没 --ack，应当在第 3 次经 proc.py stop 终止 {sleeper.pid} 并写「{wanted}」，"
                            f"前两次不终止；实际桩收到 {stub_lines()}，三次的报告行 {outputs}")
        checks += 1
        open(stub_calls, "w").close()
        path, outputs = run_reports("counted", [([stuck(sleeper.pid)], [])] * 2)
        recorded = open(path, encoding="utf-8").read().splitlines() if os.path.isfile(path) else []   # 直接读文件，不经 read_repeated_reports（它有弄坏开关）
        if recorded != [f"进程:{sleeper.pid}:进程无输出\t2\t{process_start_ticks(sleeper.pid)}"] or stub_lines():
            failures.append(f"连报到阈值：第 2 次叫醒之后状态文件里 进程:{sleeper.pid}:进程无输出 应当记 2 次、连同它的起始时刻，还不终止；实际 {recorded}，桩收到 {stub_lines()}")
        # ② 有 --ack：告警只在确认过的那一堆里，三次都不算、不终止
        checks += 1
        open(stub_calls, "w").close()
        path, outputs = run_reports("acknowledged", [([], [stuck(sleeper.pid)])] * 3)
        if stub_lines() or any(outputs):
            failures.append(f"有 --ack：--ack 进程:{sleeper.pid} 确认过的「进程无输出」连报 3 次也不该终止，实际桩收到 {stub_lines()}，报告行 {outputs}")
        # ③ 阈值 0：关掉，不记、不终止
        checks += 1
        open(stub_calls, "w").close()
        path, outputs = run_reports("disabled", [([stuck(sleeper.pid)], [])] * 3, threshold=0)
        if stub_lines() or any(outputs) or os.path.exists(path):
            failures.append(f"阈值 0：terminate-after-unacknowledged-reports=0 是关掉，连报 3 次也不该终止、不该记，实际桩收到 {stub_lines()}，报告行 {outputs}，"
                            f"记录文件{'在' if os.path.exists(path) else '不在'}")
        # ④ 中间断过一次（条件没了、没有告警的那次退出）：连报从头数，三次叫醒里只连了两次，不终止
        checks += 1
        open(stub_calls, "w").close()
        path, outputs = run_reports("interrupted", [([stuck(sleeper.pid)], []), ([], []), ([stuck(sleeper.pid)], []), ([stuck(sleeper.pid)], [])])
        if stub_lines():
            failures.append(f"中间断过一次：报、没报、报、报只连了两次，不该终止，实际桩收到 {stub_lines()}")
        # ⑤ 子 agent 的工具调用过长到线：没有进程号，只报、不调桩
        checks += 1
        open(stub_calls, "w").close()
        tool_alert = ("a0000000000000001", "工具调用过长", "Bash 已跑 40 分钟：cargo test", "主 agent 判断")
        path, outputs = run_reports("agent-tool", [([tool_alert], [])] * 3)
        if stub_lines() or not any("脚本停不了子 agent，只报" in line for line in outputs[2]):
            failures.append(f"子 agent 的工具调用过长连报 3 次：应当只报「脚本停不了子 agent，只报」、不调 proc.py，实际桩收到 {stub_lines()}，第 3 次的报告行 {outputs[2]}")
        # ⑥ 两件同时到线（进程无输出、重型测试没带前缀）：一次叫醒只终止一个，另一个写「下一次再判」，下一次叫醒再终止它
        checks += 1
        open(stub_calls, "w").close()
        both = [stuck(sleeper.pid), stuck(other.pid, HEAVY_TEST_ALERT)]
        path, outputs = run_reports("two-at-once", [(both, [])] * 3 + [([stuck(other.pid, HEAVY_TEST_ALERT)], [])])
        if (stub_lines() != [f"stop {sleeper.pid}", f"stop {other.pid}"] or not any("下一次再判" in line for line in outputs[2])
                or not any(line.startswith(f"终止了 {other.pid}（{HEAVY_TEST_ALERT}，连报 4 次没处置）") for line in outputs[3])):
            failures.append(f"两件同时到线：第 3 次只终止 {sleeper.pid}、{other.pid} 写「下一次再判」，第 4 次再终止 {other.pid}；"
                            f"实际桩收到 {stub_lines()}，后两次的报告行 {outputs[2:]}")
    finally:
        PROCESS_STOP_SCRIPT = saved_script
        for process in (sleeper, other):
            process.kill()
            process.wait()
    # ⑦ 真跑：--processes-only 盯一个写着的文件一小时没动、不占 CPU 的进程，同一个会话目录连起三次，第三次叫醒时经桩终止它
    checks += 1
    open(stub_calls, "w").close()
    silent_file = os.path.join(work, "terminate-silent-output.log")
    open(silent_file, "w").close()
    silent = subprocess.Popen(["bash", "-c", f"exec 3>>'{silent_file}'; touch -d '1 hour ago' '{silent_file}'; exec sleep 120"])
    probed_processes.append(silent)
    session_directory = os.path.join(work, "session-terminate")
    os.makedirs(os.path.join(session_directory, "subagents"), exist_ok=True)
    environment = dict(os.environ, AGENT_WATCH_PROC_SCRIPT=stub, AGENT_WATCH_STATE_ROOT=os.path.join(work, "terminate-watch-state"))
    try:
        wait_until(lambda: (process_arguments(silent.pid) or [""])[0] == "sleep")
        runs = []
        for _ in range(3):
            watch_arguments = ["watch", "--processes-only", "--process-root-pid", str(os.getpid()), "--session-dir", session_directory,
                               "--interval-seconds", "1", "--max-minutes", "0.5", "--process-report-minutes", "0.02", "--process-stale-minutes", "0.05",
                               "--process-max-minutes", "600", "--terminate-after-unacknowledged-reports", "3"]
            watch_runs.append(watch_arguments)
            runs.append(subprocess.run([sys.executable, os.path.abspath(__file__), *watch_arguments], capture_output=True, text=True, timeout=120, env=environment))
        wanted = f"终止了 {silent.pid}（进程无输出，连报 3 次没处置）"
        if ([run.returncode for run in runs] != [3, 3, 3] or f"stop {silent.pid}" not in stub_lines()
                or wanted not in runs[2].stdout or any(wanted in run.stdout for run in runs[:2])):
            failures.append(f"看门狗真跑三次：--processes-only 连起三次都报「进程无输出」（退 3），第三次应当经 proc.py stop 终止 {silent.pid} 并写「{wanted}」；"
                            f"实际退出码 {[run.returncode for run in runs]}，桩收到 {stub_lines()}，第三次输出：{runs[2].stdout[-400:]}")
    finally:
        silent.kill()
        silent.wait()
    return checks


SELFTEST_SCRATCH_PREFIX = "agent-watch-selftest-scratch-"
SELFTEST_SCRATCH_OWNER_FILE = ".selftest-pid"
STALE_SELFTEST_SCRATCH_SECONDS = 3600


def remove_stale_selftest_scratch(parent):
    """上一次自证被 SIGKILL 打断时留在 parent 里的 agent-watch-selftest-scratch-* 目录：里面 .selftest-pid 写的进程已不在的删掉；
    没有 .selftest-pid 的（旧版留下的）最后修改超过 STALE_SELFTEST_SCRATCH_SECONDS 的删掉；进程还在的、没到时候的不动。交回删掉的名字。
    弄坏开关 AGENT_WATCH_BREAK=staleselftestscratch 下一个都不删（改前的样子）。"""
    removed = []
    if BROKEN_DETECTION == "staleselftestscratch":
        return removed
    try:
        names = sorted(os.listdir(parent))
    except OSError:
        return removed
    for name in names:
        path = os.path.join(parent, name)
        if not name.startswith(SELFTEST_SCRATCH_PREFIX) or os.path.islink(path) or not os.path.isdir(path):
            continue
        try:
            with open(os.path.join(path, SELFTEST_SCRATCH_OWNER_FILE), encoding="utf-8") as handle:
                owner = int(handle.read().strip())
        except (OSError, ValueError):
            owner = None
        if owner is not None:
            if os.path.exists(f"/proc/{owner}"):
                continue
        else:
            try:
                if time.time() - os.path.getmtime(path) < STALE_SELFTEST_SCRATCH_SECONDS:
                    continue
            except OSError:
                continue
        shutil.rmtree(path, ignore_errors=True)
        removed.append(name)
    return removed


def selftest_ack_concurrency(work, failures):
    """watch-acks.tsv 两只同时写：两个线程各记一条确认，读与写之间都停 ACK_UPDATE_PAUSE_SECONDS（拉开窗口，不加锁时两只都读到空的、后写的冲掉先写的），
    写完两条都要在。flock 按打开的文件判，同一进程里两次 open 的锁照样互斥，与两个看门狗进程同一个判法。AGENT_WATCH_BREAK=acksunlocked 下丢一条：这一格红。返回查了几格。"""
    global ACK_UPDATE_PAUSE_SECONDS
    state_dir = os.path.join(work, "ack-concurrency-state")
    wanted = {"concurrenta:工具调用过长", "进程:4242"}
    ACK_UPDATE_PAUSE_SECONDS = 1.5   # 线程起得慢（负载高）时窗口也要罩得住后起的那一只
    try:
        writers = [threading.Thread(target=update_acks, args=(state_dir, lambda acks, key=key: acks | {key})) for key in sorted(wanted)]
        for writer in writers:
            writer.start()
        for writer in writers:
            writer.join()
    finally:
        ACK_UPDATE_PAUSE_SECONDS = 0.0
    got = read_acks(state_dir)
    if got != wanted:
        failures.append(f"watch-acks.tsv 两只同时写：两条确认 {sorted(wanted)} 都应当在（读改写拿着 watch-acks.lock 的 flock），实际 {sorted(got) or '一条都没有'}")
    return 1


def selftest_stale_scratch_cleanup(work, failures):
    """被杀的自证留下的 agent-watch-selftest-scratch-*：进程已不在的、没有主人记号而超过一小时的删掉；进程还在的（这个自证自己）、没到一小时的、别的名字的不动。返回查了几格。"""
    parent = os.path.join(work, "stale-scratch-parent")
    os.makedirs(parent)
    finished = subprocess.Popen(["true"])
    finished.wait()
    for suffix, owner in (("dead-owner", finished.pid), ("live-owner", os.getpid())):
        os.makedirs(os.path.join(parent, SELFTEST_SCRATCH_PREFIX + suffix))
        with open(os.path.join(parent, SELFTEST_SCRATCH_PREFIX + suffix, SELFTEST_SCRATCH_OWNER_FILE), "w", encoding="utf-8") as handle:
            handle.write(f"{owner}\n")
    for suffix, age in (("old-no-owner", 2 * STALE_SELFTEST_SCRATCH_SECONDS), ("fresh-no-owner", 0)):
        made = os.path.join(parent, SELFTEST_SCRATCH_PREFIX + suffix)
        os.makedirs(made)
        os.utime(made, (time.time() - age, time.time() - age))
    os.makedirs(os.path.join(parent, "agent-watch-selftest-other-kept"))
    removed = remove_stale_selftest_scratch(parent)
    wanted = [SELFTEST_SCRATCH_PREFIX + "dead-owner", SELFTEST_SCRATCH_PREFIX + "old-no-owner"]
    if removed != wanted or sorted(os.listdir(parent)) != sorted([SELFTEST_SCRATCH_PREFIX + "live-owner", SELFTEST_SCRATCH_PREFIX + "fresh-no-owner",
                                                                   "agent-watch-selftest-other-kept"]):
        failures.append(f"被杀的自证留下的 {SELFTEST_SCRATCH_PREFIX}*：应当只删主人进程已不在的与没有主人记号而超过一小时的（{wanted}），"
                        f"主人还在的、没到一小时的、别的名字的不动，实际删了 {removed}，剩 {sorted(os.listdir(parent))}")
    return 1


def selftest_session_watch(work, thresholds, failures, watch_runs):
    """这一轮加的几样：「该交接」只报两类、「进度文件不涨」、整点询问附进度、hook 检出主体是 agent 号、--discover 单例与确认落文件。返回查了几格。"""
    session = os.path.join(work, "session-discover")
    # 草稿目录要在工作根下（不看弄坏开关，configured_work_root），派发提示里的路径才认得出（SCRATCH_PATH_MENTION）；它在自检的 TMPDIR 之外，自己删
    scratch_parent = work_roots.configured_work_root()
    os.makedirs(scratch_parent, exist_ok=True)
    remove_stale_selftest_scratch(scratch_parent)   # 上一次被杀的自证留下的：自证开跑先收掉
    scratch = tempfile.mkdtemp(prefix=SELFTEST_SCRATCH_PREFIX, dir=scratch_parent)
    with open(os.path.join(scratch, SELFTEST_SCRATCH_OWNER_FILE), "w", encoding="utf-8") as owner_handle:
        owner_handle.write(f"{os.getpid()}\n")
    try:
        return selftest_session_watch_in(work, session, scratch, thresholds, failures, watch_runs)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)


def selftest_scratch_mentions(failures):
    """派发提示行首「草稿目录：」那一行点名的草稿目录：工作根（展开的绝对路径与 ~ 写法）与过渡期旧根下的都找得到 progress.md；
    提示里另点名的别的子 agent 的目录不取，没有那一行的取不到。建的目录自己删。返回查了几格。"""
    class Prompt:
        def __init__(self, text):
            self.dispatch_prompt = text
    home = os.path.expanduser("~")
    checks, made = 0, []
    try:
        for root in (work_roots.configured_work_root(), work_roots.legacy_root()):
            os.makedirs(root, exist_ok=True)
            made.append(tempfile.mkdtemp(prefix="agent-watch-selftest-mention-", dir=root))
            open(os.path.join(made[-1], PROGRESS_FILE_NAME), "w", encoding="utf-8").write("第一格跑完\n")
        new_directory, old_directory = made
        home_form = "~" + new_directory[len(home):] if new_directory.startswith(home + "/") else new_directory
        for label, written, wanted in (("派发提示点名工作根下的草稿目录（绝对路径）", new_directory, new_directory),
                                       ("派发提示点名工作根下的草稿目录（~ 写法）", home_form, new_directory),
                                       ("派发提示点名过渡期旧根下的草稿目录", old_directory, old_directory)):
            checks += 1
            found = progress_file_of(Prompt(f"跑前登记 x。\n草稿目录：{written}/\n报告：{written}/result.md\n"))
            if found != os.path.join(wanted, PROGRESS_FILE_NAME):
                failures.append(f"{label}：应当找到 {os.path.join(wanted, PROGRESS_FILE_NAME)}，实际 {found}")
        # 只认行首「草稿目录：」那一行（lib_prompt_scratch_line.py）：提示里另点名别的子 agent 的目录、它的 progress.md 改得更晚，也不取它；
        # 没有那一行的取不到、告警里写取不到（AGENT_WATCH_BREAK=progressanypath 下照改前取改得最晚的那一份：这三格红）
        made.append(tempfile.mkdtemp(prefix="agent-watch-selftest-mention-other-", dir=work_roots.configured_work_root()))
        other_directory = made[-1]
        open(os.path.join(other_directory, PROGRESS_FILE_NAME), "w", encoding="utf-8").write("别的子 agent 的一格\n")
        older = time.time() - 600
        os.utime(os.path.join(new_directory, PROGRESS_FILE_NAME), (older, older))
        for label, text, wanted in (
                ("派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告（那边的 progress.md 改得更晚）：应当只取本提示草稿目录下的",
                 f"跑前登记 x。\n草稿目录：{new_directory}\n先读上一个实现员的报告 {other_directory}/report.md，它的探针在 {other_directory}/\n",
                 os.path.join(new_directory, PROGRESS_FILE_NAME)),
                ("派发提示没有行首「草稿目录：」一行、只点名别的子 agent 的报告：应当取不到",
                 f"跑前登记 x。报告写 {new_directory}/result.md，先读 {other_directory}/report.md\n", None)):
            checks += 1
            found = progress_file_of(Prompt(text))
            if found != wanted:
                failures.append(f"{label}，应当 {wanted}，实际 {found}")
        checks += 1
        note = progress_note(Prompt(f"跑前登记 x。先读 {other_directory}/report.md\n"), datetime.now(timezone.utc))
        if "没有行首「草稿目录：」一行，取不到 progress.md" not in note:
            failures.append(f"派发提示没有行首「草稿目录：」一行时，告警里附的一句应当写取不到 progress.md，实际「{note}」")
    finally:
        for directory in made:
            shutil.rmtree(directory, ignore_errors=True)
    return checks


def selftest_session_watch_in(work, session, scratch, thresholds, failures, watch_runs):
    checks = 0
    progress_path = os.path.join(scratch, PROGRESS_FILE_NAME)
    open(progress_path, "w", encoding="utf-8").write("第一格跑完\n第二格跑完\n第三格在跑，预计 40 分钟\n")
    stale_moment = time.time() - 45 * 60
    os.utime(progress_path, (stale_moment, stale_moment))
    prompt = record_at(20, "user", f"跑前登记 x。报告写 {scratch}/result.md\n草稿目录：{scratch}/\n")
    old_prompt = record_at(65, "user", f"跑前登记 x。报告写 {scratch}/result.md\n草稿目录：{scratch}/\n")
    waiting_tail = [bash_use(10, "t1", "cargo run --release", "m1"),
                    record_at(9.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: bq1."}]),
                    record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
    write_transcript(session, "progressstale", [prompt, *waiting_tail], {"agentType": "experiment-runner", "description": "等后台任务、进度 45 分钟没改"})
    write_transcript(session, "attackhuge", [record_at(0.5, "assistant", [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "ls"}}],
                                                       stop_reason="tool_use", message_id="m1", usage={"input_tokens": 10, "cache_read_input_tokens": 800_000})],
                     {"agentType": "three-way-attack", "description": "攻方上下文 80 万"})
    write_transcript(session, "inquiryprogress", [old_prompt, bash_use(64.9, "t1", "ls", "m1"), bash_result(64.8, "t1"), bash_use(0.5, "t2", "cargo build", "m2")],
                     {"agentType": "experiment-runner", "description": "跑了 65 分钟、草稿目录里有进度文件"})
    open(session + ".jsonl", "a").close()
    now = datetime.now(timezone.utc)
    for agent_id, wanted in (("progressstale", {PROGRESS_STALE_ALERT}), ("attackhuge", set())):
        got = {name for name, _, _ in agent_alerts(AgentTranscript(agent_id, find_transcript(agent_id, session)), now, thresholds)}
        checks += 1
        if got != wanted:
            failures.append(f"子 agent {agent_id}：应当告警 {sorted(wanted) or '无'}，实际 {sorted(got) or '无'}")
    inquiry = routine_inquiry_alert(AgentTranscript("inquiryprogress", find_transcript("inquiryprogress", session)), now, thresholds)
    checks += 1
    if inquiry is None or "第三格在跑，预计 40 分钟" not in inquiry[1]:
        failures.append(f"整点询问应当附上 progress.md 的末几行，实际 {inquiry}")
    detections = os.path.join(work, "detections-subject.jsonl")
    with open(detections, "w", encoding="utf-8") as detections_file:
        detections_file.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1",
                                          "command": "Write /x/.claude/kb/a.md", "findings": ["写被拒：越出写范围"]}, ensure_ascii=False) + "\n")
        detections_file.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1",
                                          "command": "pgrep -f cargo", "findings": ["命令里有 pgrep -f / pkill -f / killall"]}, ensure_ascii=False) + "\n")
        detections_file.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1",
                                          "command": "bash x.sh", "findings": ["重型测试闸没判全：x.sh 读不进来"]}, ensure_ascii=False) + "\n")
    _, detection_alerts, detection_notes = read_detections(detections, 0, {"s1"}, set())
    checks += 1
    if [alert[0] for alert in detection_alerts] != ["a1"] or not any("只记不叫醒" in note and "写被拒" in note for note in detection_notes) \
            or not any("只记不叫醒" in note and "重型测试闸没判全" in note for note in detection_notes):
        failures.append("hook 检出：写被拒的与重型测试闸没判全的应当只记进报告不叫醒，按模式找进程的应当告警且主体是 agent 号 a1（--ack a1:hook 检出 只压它），"
                        f"实际告警主体 {[alert[0] for alert in detection_alerts]}，记录 {detection_notes}")
    # 钩子执行前已拒掉的检出（写了结局「被拒」的 b1，没写结局而发现以「…被拒」起头的旧记录 b2）只进报告、不叫醒；写了「只记」的 b3 照常告警，下一步说命令照常在跑
    # （AGENT_WATCH_BREAK=outcomeignored 下 b1、b2 被当成只记而叫醒；refusedwakes 下它们照旧叫醒：这一格都红）
    outcome_detections = os.path.join(work, "detections-outcome.jsonl")
    with open(outcome_detections, "w", encoding="utf-8") as detections_file:
        for agent_id, findings, outcome in (("b1", ["重型测试被拒：cargo test 带 --workspace / --all（全量测试）：implementation-writer 不跑「全量测试」"], "被拒"),
                                            ("b2", ["提交时才跑的检查被拒：.claude/singlefs-ai-sop/scripts/doc-lint.sh（提交时才跑的检查）"], None),
                                            ("b3", ["命令里有 pgrep -f / pkill -f / killall"], "只记")):
            entry = {"session_id": "s1", "agent_type": "implementation-writer", "agent_id": agent_id, "command": "x", "findings": findings}
            if outcome:
                entry["outcome"] = outcome
            detections_file.write(json.dumps(entry, ensure_ascii=False) + "\n")
    _, outcome_alerts, outcome_notes = read_detections(outcome_detections, 0, {"s1"}, set())
    next_steps = {alert[0]: alert[3] for alert in outcome_alerts}
    checks += 1
    if not (sorted(next_steps) == ["b3"] and "照常在跑" in next_steps.get("b3", "")
            and all(any(f"implementation-writer {agent_id}：" in note and "执行前已被 hook 拒了、没有执行，只记不叫醒" in note for note in outcome_notes)
                    for agent_id in ("b1", "b2"))):
        failures.append("hook 检出：执行前已被 hook 拒了的（b1 写了结局「被拒」、b2 是没写结局的旧记录）应当只记进报告、不叫醒，"
                        f"只记的（b3）照常告警、下一步说命令照常在跑，实际告警 {sorted(next_steps)}、下一步 {next_steps}、记录 {outcome_notes}")
    # 钩子 ⑭ 判不了的长活（以「长活判不了」起头、结局「只记」，主 agent 的与带 agent 号的各一条）只进报告、不叫醒；
    # 同一条里另有一样照常要叫醒的检出（l2）照旧叫醒（AGENT_WATCH_BREAK=longrunwakes 下前两条也叫醒：这一格红）
    long_run_detections = os.path.join(work, "detections-long-run.jsonl")
    long_run_finding = f"{LONG_RUN_UNDETERMINABLE_PREFIX}：bash scripts/sample-long.sh（没有历史用时、文件头也没有长活声明），照常执行"
    with open(long_run_detections, "w", encoding="utf-8") as detections_file:
        for agent_id, agent_type, findings in ((None, "主 agent", [long_run_finding]), ("l1", "implementation-writer", [long_run_finding]),
                                               ("l2", "implementation-writer", [long_run_finding, "命令里有 pgrep -f / pkill -f / killall"])):
            entry = {"session_id": "s1", "agent_type": agent_type, "command": "bash scripts/sample-long.sh", "findings": findings, "outcome": "只记"}
            if agent_id:
                entry["agent_id"] = agent_id
            detections_file.write(json.dumps(entry, ensure_ascii=False) + "\n")
    _, long_run_alerts, long_run_notes = read_detections(long_run_detections, 0, {"s1"}, set())
    checks += 1
    long_run_quiet = [note for note in long_run_notes if "只记不叫醒" in note and LONG_RUN_UNDETERMINABLE_PREFIX in note]
    if [alert[0] for alert in long_run_alerts] != ["l2"] or len(long_run_quiet) != 2 \
            or not any(note.startswith("hook 检出（只记不叫醒）：主 agent") for note in long_run_quiet):
        failures.append("hook 检出：钩子 ⑭ 判不了的长活（主 agent 的、子 agent l1 的）应当只记进报告、不叫醒，另带一样要叫醒的检出的 l2 照旧叫醒；"
                        f"实际告警主体 {[alert[0] for alert in long_run_alerts]}，记录 {long_run_notes}")
    # 范围声明钩子的放行记录（scope_session_id、run_scope）：属于这一会话的 c1 与主 agent 那一条列进「范围声明」一节（命令、四段），不叫醒、不进检出记录那几行；
    # 别的会话的 c2 不列（AGENT_WATCH_BREAK=scopeunlisted 下一条都不列：这一格红）
    scope_detections = os.path.join(work, "detections-scope.jsonl")
    with open(scope_detections, "w", encoding="utf-8") as detections_file:
        for session_id, agent_id, agent_type, command in (
                ("s1", "c1", "implementation-writer", "bash .claude/gate.d/checker-tier-formal-verification.sh --quick /w"),
                ("s1", None, "主 agent", "bash .claude/gate.d/doc-text.sh --check prime-marks"),
                ("s9", "c2", "implementation-writer", "bash scripts/crash/crash-point-replay.sh --quick")):
            detections_file.write(json.dumps({"hook": "run-scope-guard.sh", "agent_id": agent_id, "agent_type": agent_type, "command": command,
                                              "scope_session_id": session_id, "outcome": "放行",
                                              "run_scope": {"where": "命令文本", "kind_of_command": "quick", "what": "样本",
                                                            "declaration": "种类=quick; 跑什么=样本; 范围=轻型核心部分; 为什么=只改了这一道的判法所以只跑轻型",
                                                            "segments": {"种类": "quick", "跑什么": f"样本 {agent_id}", "范围": "轻型核心部分",
                                                                         "为什么": "只改了这一道的判法所以只跑轻型"}}}, ensure_ascii=False) + "\n")
    scope_lines = []
    _, scope_alerts, scope_notes = read_detections(scope_detections, 0, {"s1"}, set(), scope_lines)
    section = scope_declaration_section(scope_lines)
    checks += 1
    if not (not scope_alerts and not scope_notes and len(section) == 3 and section[0].startswith("范围声明（run-scope-guard.sh 放行的 2 条")
            and "implementation-writer c1：bash .claude/gate.d/checker-tier-formal-verification.sh --quick /w：种类=quick；跑什么=样本 c1；范围=轻型核心部分；为什么=只改了这一道的判法所以只跑轻型" in section[1]
            and section[2].startswith("  主 agent：bash .claude/gate.d/doc-text.sh --check prime-marks：种类=quick")
            and not any("c2" in line for line in section)):
        failures.append("范围声明：这一会话放行的两条（c1、主 agent）应当列进「范围声明」一节、每条一行写命令与四段，不叫醒、不进检出记录那几行，别的会话的 c2 不列，"
                        f"实际告警 {scope_alerts}、记录 {scope_notes}、这一节 {section}")
    # --discover：状态目录放自检自己的；一个会话一个看门狗，--ack 落文件下一次读回，条件消失的确认作废
    state_root = os.path.join(work, "watch-state")
    environment = dict(os.environ, AGENT_WATCH_STATE_ROOT=state_root)
    write_transcript(session, "discoveredlong", [prompt, bash_use(19, "t1", "cargo test --release", "m1")],
                     {"agentType": "implementation-writer", "description": "工具调用 20 分钟"})
    write_transcript(session, "discovereddone", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18, "h1", "m2")],
                     {"agentType": "sweep", "description": "交回了"})
    # 这一段的看门狗都带 --idle-minutes 600：progressstale 最后一条记录写在 9.8 分钟前，默认 10 分钟的线写完 12 秒就过；
    # AGENT_WATCH_BREAK=backgroundwait 下等后台任务的也报「无动静」，负载下看门狗起得晚就当场退 3，这一段要判的锁与 --ack 判不到（判它的是上面 progressstale 那一格）
    quiet_idle = ["--idle-minutes", "600"]

    def discover_watch(extra, timeout=90):
        watch_runs.append(["--discover", *extra])
        return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--discover", "--session-dir", session, "--process-root-pid", "0",
                               "--interval-seconds", "1", "--detections-file", os.path.join(work, "no-detections.jsonl"), *quiet_idle, *extra],
                              capture_output=True, text=True, timeout=timeout, env=environment)
    acknowledged_keys = ["discoveredlong:工具调用过长", "progressstale:进度文件不涨", "inquiryprogress:跑满 1 小时", "progressstale:跑满 1 小时",
                         "attackhuge:该交接", "inquiryprogress:该交接"]
    first = discover_watch([*sum((["--ack", key] for key in acknowledged_keys), []), "--max-minutes", "0.03"])
    checks += 1
    if first.returncode != 4 or "discoveredlong" not in first.stdout or "还没交回、没被停的子 agent" not in first.stdout:
        failures.append(f"--discover 应当自己找出没结束的 discoveredlong、确认过的不叫醒、到点退 4，实际 {first.returncode}：{first.stdout[-500:]}")
    if "   → 接着盯就用 run_in_background 再起一个看门狗：bash scripts/agents/watch.sh\n" not in first.stdout:
        failures.append(f"--discover 的出路句应当是不带 agent 号的 bash scripts/agents/watch.sh（一个会话一个看门狗），实际：{first.stdout[-300:]}")
    second = discover_watch(["--max-minutes", "0.03"])
    checks += 1
    if second.returncode != 4 or "已确认接着盯（--ack discoveredlong:工具调用过长）" not in second.stdout:
        failures.append(f"--ack 应当落进状态目录、下一次起不带 --ack 也读回，实际 {second.returncode}：{second.stdout[-400:]}")
    # 在盯的那一个拿到锁、做完第一次整查（按它建的就绪文件等，不按固定秒数）之后再起第二个；第二个查完就停它（自己起的进程号），不等它到点。
    # 负载高时 python 起得慢：固定等 2 秒时它可能还没拿到锁，回报周期 9 秒时第二个起来之前它可能已经到点退了
    holder_ready = os.path.join(work, "ready-discover-holder")
    # 在盯的那一个盯 10 分钟（查完由 finally 停它）：负载下第二个起得慢，盯 1 分钟时它可能先到点退了
    holder = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--discover", "--session-dir", session, "--process-root-pid", "0",
                               "--interval-seconds", "1", "--max-minutes", "10", "--detections-file", os.path.join(work, "no-detections.jsonl"), *quiet_idle],
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=dict(environment, **{SELFTEST_READY_VARIABLE: holder_ready}))
    try:
        wait_for_selftest_ready(holder_ready, holder)
        duplicate = discover_watch(["--ack", "attackhuge:hook 检出", "--max-minutes", "0.03"])
        checks += 1
        if duplicate.returncode != SINGLE_INSTANCE_EXIT or "已有一个看门狗在盯" not in duplicate.stdout:
            failures.append(f"同一个会话已有一个看门狗在盯时应当退 {SINGLE_INSTANCE_EXIT} 并写明，实际 {duplicate.returncode}：{duplicate.stdout[-300:]}")
        # 第二个记下的 --ack 要经得起在盯的那一个下一次整查（它每次整查把确认按条件剪一遍再写回，os.replace 换新 inode）：等它写回过一次再读，
        # 不按读的早晚碰运气（AGENT_WATCH_BREAK=discovernone 下它认不出 attackhuge 没结束，这一条确认被剪掉，这一格红）
        acks_path = os.path.join(state_root, os.path.basename(session), "watch-acks.tsv")
        written_by_duplicate = os.stat(acks_path).st_ino if os.path.exists(acks_path) else None
        wait_until(lambda: holder.poll() is not None or (os.path.exists(acks_path) and os.stat(acks_path).st_ino != written_by_duplicate))
        checks += 1
        if "attackhuge:hook 检出" not in read_acks(os.path.join(state_root, os.path.basename(session))):
            failures.append("已有一个在盯时给的 --ack 应当记进状态目录，好让在盯的那一个读到")
    finally:
        holder.terminate()
        holder.communicate(timeout=60)
    # 会话里的子 agent 都交回了：--discover 没有要盯的，接着盯到点（新派的会接上），不退回「会话里最近动过的全部」、不报「全部交回」
    quiet_session = os.path.join(work, "session-all-done")
    write_transcript(quiet_session, "onlydone", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18, "h1", "m2")],
                     {"agentType": "sweep", "description": "交回了"})
    open(quiet_session + ".jsonl", "a").close()
    watch_runs.append(["--discover", "all-done"])
    quiet = subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--discover", "--session-dir", quiet_session, "--process-root-pid", "0",
                            "--interval-seconds", "1", "--max-minutes", "0.03", "--detections-file", os.path.join(work, "no-detections.jsonl"), *quiet_idle],
                           capture_output=True, text=True, timeout=90, env=environment)
    checks += 1
    if quiet.returncode != 4 or "全部交回" in quiet.stdout:
        failures.append(f"--discover 时会话里的子 agent 都交回了应当接着盯到点退 4、不报全部交回，实际 {quiet.returncode}：{quiet.stdout[-300:]}")
    state_acks = read_acks(os.path.join(state_root, os.path.basename(session)))   # 子进程的 STATE_ROOT 是环境变量给的这一个，自检进程自己的不是
    checks += 1
    if "inquiryprogress:该交接" in state_acks or "discoveredlong:工具调用过长" not in state_acks:
        failures.append(f"条件类告警的确认：条件不在（inquiryprogress 没到线）的应当作废，还在的（discoveredlong 工具过长）应当留着，实际 {sorted(state_acks)}")
    return checks


def selftest():
    """主体在 selftest_in；临时目录由共用库建（通过、判红、抛异常都删），那三格也由它判。
    弄坏开关 AGENT_WATCH_BREAK=keepscratch 走回 mkdtemp 不删，那三格必须判红。"""
    try:
        spec = importlib.util.spec_from_file_location("lib_selftest_scratch", SELFTEST_SCRATCH_LIBRARY_PATH)
        scratch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(scratch)
    except Exception as error:   # 文件不在、语法错、导入时抛的都算读不到
        print(f"  ✗ 自检：读不到共用的临时目录库 {SELFTEST_SCRATCH_LIBRARY_PATH}（{error!r}）")
        print("    → 恢复 .claude/hooks/lib_selftest_scratch.py（自证临时目录的建法与三格探查只有那一份），再跑 --selftest")
        return 1
    exit_code, problems = scratch.run_with_scratch(
        selftest_in, "agent-watch-selftest-", BROKEN_DETECTION == "keepscratch",
        SELFTEST_ROLE_VARIABLE, [sys.executable, os.path.abspath(__file__), "--selftest"])
    return scratch.merged_exit_code(exit_code, problems, "scripts/agents/agent-watch.py 的 selftest()", "AGENT_WATCH_BREAK=keepscratch")


def selftest_in(work):
    thresholds = argparse.Namespace(tool_minutes=8, wait_loop_minutes=3, idle_minutes=10, repeat_count=3, interval_seconds=240,
                                    process_report_minutes=0.02, process_stale_minutes=0.05, process_max_minutes=600,
                                    not_started_minutes=5, active_minutes=600, context_tokens=600_000, leftover_background_grace_minutes=2,
                                    ask_every_minutes=60, process_rss_gibibytes=8, progress_stale_minutes=30)
    session = os.path.join(work, "session")
    failures = []
    os.environ[CORE_USAGE_SAMPLES_ROOT_VARIABLE] = os.path.join(work, "core-usage-samples-root")   # 这份自证起的看门狗子进程都写这里，随临时目录删
    # 本机真实的会话登记与草稿根不进自证：登记目录指到不存在的地方（判不了会话在不在，一律不当失效），草稿根指到一个空目录；
    # 要用它们的两格（selftest_stale_sessions、selftest_scratch_hogs）自己给子进程另设
    os.environ[SESSION_REGISTRY_ENV] = os.path.join(work, "no-session-registry")
    os.environ[SCRATCH_ROOTS_ENV] = os.path.join(work, "no-scratch-roots")
    # 本机根分区真实的可用量也不进自证：这份自证起的看门狗子进程一律当可用 1 PiB（不报「根分区余量过低」）；要它过线的那一格（selftest_root_free）给子进程另设
    os.environ[ROOT_FREE_BYTES_VARIABLE] = str(1 << 50)
    write_transcript(session, "finished", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18.5, "h1", "m2"),
                                           record_at(18, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                     {"agentType": "experiment-runner", "description": "跑完的"})
    write_transcript(session, "waiting", [bash_use(1, "t1", "cargo test --release", "m1"),
                                          record_at(0.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: b1."}]),
                                          record_at(0.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "experiment-runner", "description": "结束本轮在等后台任务、没交回"})
    write_transcript(session, "waitinglong", [bash_use(20, "t1", "cargo test --release", "m1"),
                                              record_at(19.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: b7."}]),
                                              record_at(19.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "three-way-attack", "description": "结束本轮等自己的后台任务已 20 分钟、没交回（不是无动静）"})
    write_transcript(session, "continued", [*handback_records(3, "h1", "m1"),
                                            record_at(2.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m2"),
                                            record_at(2, "user", "续做：再补一格"), bash_use(1, "t2", "ls", "m3"), bash_result(0.9, "t2")],
                     {"agentType": "experiment-runner", "description": "交回之后被续做"})
    write_transcript(session, "resumedthinking", [*handback_records(3, "h1", "m1"),
                                                  record_at(2.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m2"),
                                                  record_at(0.2, "user", "续做：按判决改", origin_kind="coordinator")],
                     {"agentType": "implementation-writer", "description": "交回之后刚被续做、还没调工具"})
    write_transcript(session, "resumedafterturnend", [bash_use(10, "t1", "ls", "m1"), bash_result(9.9, "t1"),
                                                      record_at(9, "assistant", [{"type": "text", "text": "第一段做完了"}], stop_reason="end_turn", message_id="m2"),
                                                      record_at(0.1, "user", "续派第二段", origin_kind="coordinator")],
                     {"agentType": "experiment-runner", "description": "结束本轮没交回、刚被续派、还没落下新记录"})
    write_transcript(session, "notifiedafterhandback", [*handback_records(3, "h1", "m1"),
                                                        record_at(2.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m2"),
                                                        record_at(0.2, "user", "[SYSTEM NOTIFICATION - NOT USER INPUT] 后台任务跑完了", origin_kind="task-notification")],
                     {"agentType": "implementation-writer", "description": "交回之后收到后台任务的完成通知"})
    # 交回之后被自己后台任务的完成通知叫醒、看一眼输出再结束本轮：仍算交回，不许判成没交回、报「结束本轮却不会醒」
    # （2026-09-23 门禁审计会话 ac26abd494cda0968 实测误报，次序照它的会话记录：前台 doc-lint 超时转后台、交回、结束本轮、完成通知、grep 一次、结束本轮）。
    moved_task_started = record_at(4, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                "content": "Command did not complete within its 120s timeout and was moved to the background (ID: bl1). "
                                                           f"Output is being written to: {os.path.join(work, 'bl1.output')}. You will be notified when it completes."}])
    moved_task_finished = record_at(2.7, "user", "[SYSTEM NOTIFICATION - NOT USER INPUT]\n<task-notification>\n<task-id>bl1</task-id>\n<status>completed</status>\n</task-notification>",
                                    origin_kind="task-notification")
    write_transcript(session, "toolafterhandback", [bash_use(6, "t1", 'bash .claude/singlefs-ai-sop/scripts/doc-lint.sh "$PWD"', "m1"), moved_task_started,
                                                    *handback_records(3, "h1", "m2"),
                                                    record_at(2.8, "assistant", [{"type": "text", "text": "报告已交回主 agent。"}], stop_reason="end_turn", message_id="m3"),
                                                    moved_task_finished, bash_use(2.6, "t2", "grep -n '✗' bl1.output", "m4"), bash_result(2.5, "t2"),
                                                    record_at(2.4, "assistant", [{"type": "text", "text": "后台那次 doc-lint 跑完了，和交回里写的一致"}],
                                                              stop_reason="end_turn", message_id="m5")],
                     {"agentType": "general-purpose", "description": "交回之后被自己后台任务的完成通知叫醒、调了工具再结束本轮"})
    # 前台命令跑满超时被挪进后台之后结束本轮去等它：算在等自己的后台任务，不许报「结束本轮却不会醒」（同一份会话记录里的 doc-lint 就是这样转进后台的）。
    write_transcript(session, "movedbackground", [bash_use(4, "t1", "bash .claude/scripts/gate.sh --staged", "m1"),
                                                  record_at(2, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                                         "content": "Command did not complete within its 120s timeout and was moved to the background (ID: bm1). "
                                                                                    f"Output is being written to: {os.path.join(work, 'bm1.output')}. You will be notified when it completes."}]),
                                                  record_at(1.8, "assistant", [{"type": "text", "text": "等超时转进后台的那条跑完"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "gate-triage", "description": "前台命令超时转进后台、结束本轮等它"})
    # 交回之后、它还在忙的时候来的续做消息只落成 queued_command 附件（附件自己带 origin coordinator），同样撤销交回；
    # 续做之后干完活、没再交回就结束本轮的，要报：一个后台任务都没起过，报「中途停下没交回」。
    queued_resume = {"timestamp": record_at(3.8, "user", "")["timestamp"], "type": "attachment",
                     "attachment": {"type": "queued_command", "prompt": "再补一格", "origin": {"kind": "coordinator"}, "isMeta": True}}
    write_transcript(session, "queuedresume", [*handback_records(4, "h1", "m1"), queued_resume, bash_use(3.7, "t2", "ls", "m2"), bash_result(3.6, "t2"),
                                               record_at(3.5, "assistant", [{"type": "text", "text": "补完了"}], stop_reason="end_turn", message_id="m3")],
                     {"agentType": "experiment-runner", "description": "交回之后忙时收到续做消息、干完没再交回就结束本轮"})
    write_transcript(session, "waitloop", [bash_use(5, "t1", 'until grep -q "^exit=" log; do sleep 10; done', "m1")])
    write_transcript(session, "longtool", [bash_use(20, "t1", "cargo test --release", "m1")])
    # 断网 / 被停砍掉的那一次调用：只有发起、永远没有结果，而它之后 agent 又接着动了。
    # 那条调用已经死了，不许再按「工具调用过长」报（2026-09-22 三条腿续做后实测假报）。
    write_transcript(session, "stalepending", [bash_use(20, "t1", "cargo test --release", "m1"),
                                              bash_use(0.5, "t2", "cargo build", "m2")])
    # 同一种死掉的调用，之后接的是一次跑完的工具调用（有结果）：最后一条记录是结果，不是新的发起。
    # 上一格里最近的未完调用是 t2，挑最近那一个时 t1 显不出来，剔不剔都不报；这一格不剔掉 t1 就报「工具调用过长」。
    write_transcript(session, "stalependingthenresult", [bash_use(20, "t1", "cargo test --release", "m1"),
                                                        bash_use(1, "t2", "ls", "m2"), bash_result(0.9, "t2")])
    write_transcript(session, "healthy", [bash_use(0.5, "t1", "cargo build", "m1")])
    write_transcript(session, "hugecontext", [record_at(0.5, "assistant", [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "cargo build"}}],
                                                        stop_reason="tool_use", message_id="m1", usage={"input_tokens": 10, "cache_read_input_tokens": 700_000})],
                     {"agentType": "experiment-runner", "description": "被续派几次、上下文涨到 70 万"})
    write_transcript(session, "boundedloop", [bash_use(5, "t1", 'timeout 600 bash -c "until grep -q x log; do sleep 10; done"', "m1")])
    repeat_records = []
    for index in range(3):
        repeat_records += [bash_use(3 - index * 0.5, f"r{index}", "tail -5 /tmp/out.log", f"mr{index}"), bash_result(2.9 - index * 0.5, f"r{index}")]
    write_transcript(session, "repeat", repeat_records)
    changing_records = []
    for index in range(3):
        changing_records += [bash_use(3 - index * 0.5, f"g{index}", "tail -5 /tmp/growing.log", f"mg{index}"),
                             record_at(2.9 - index * 0.5, "user", [{"type": "tool_result", "tool_use_id": f"g{index}", "content": f"第 {index} 行"}])]
    write_transcript(session, "repeatchanging", changing_records)
    write_transcript(session, "banned", [bash_use(1, "t1", "pgrep -f e154-binary", "m1"), bash_result(0.9, "t1")])
    write_transcript(session, "bannedrefused", [bash_use(1, "t1", "pgrep -f e154-binary", "m1"),
                                                record_at(0.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "is_error": True,
                                                                         "content": "PreToolUse:Bash hook error: [bash hook]: ✗ 命令里有按模式找进程的写法"}])])
    write_transcript(session, "bannedheredoc", [bash_use(1, "t1", "cat >> result.md <<'EOF'\n等进程别用 pgrep -f，它会命中自己\nEOF\nls", "m1"),
                                                bash_result(0.9, "t1")])
    write_transcript(session, "bannedafterheredoc", [bash_use(1, "t1", "cat > note.md <<EOF\n说明\nEOF\npgrep -f e154-binary", "m1"),
                                                     bash_result(0.9, "t1")])
    # 50 分钟前：在禁用命令的 10 分钟窗口之外，又不到「跑满 1 小时要主 agent 问一次」的第一格
    write_transcript(session, "bannedold", [bash_use(50, "t1", "pgrep -f e154-binary", "m1"), bash_result(49.9, "t1"),
                                            bash_use(1, "t2", "ls", "m2"), bash_result(0.9, "t2")])
    write_transcript(session, "idle", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1")])
    write_transcript(session, "interrupted", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1"),
                                              record_at(39, "user", [{"type": "text", "text": "[Request interrupted by user]"}])])
    waiting_then_idle = [bash_use(25, "t1", "bash gate.sh --staged", "m1"), bash_result(24.9, "t1"),
                         record_at(24.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
    write_transcript(session, "stoppedidle", waiting_then_idle,
                     {"agentType": "gate-triage", "description": "结束本轮等后台任务时被 TaskStop 停掉"})
    write_transcript(session, "stoppedcontinued", [*waiting_then_idle, record_at(2, "user", "续做：再跑一遍"), bash_use(0.5, "t2", "cargo build", "m3")],
                     {"agentType": "gate-triage", "description": "被停之后又被续做"})
    write_transcript(session, "stoppedtwice", [*waiting_then_idle, record_at(18, "user", "续做：再跑一遍"), bash_use(17, "t2", "cargo build", "m3"),
                                               bash_result(16, "t2"), record_at(15.9, "assistant", [{"type": "text", "text": "又在等"}], stop_reason="end_turn", message_id="m4")],
                     {"agentType": "gate-triage", "description": "停了、续做、又停"})
    write_session_stops(session, {"stoppedidle": 20, "stoppedcontinued": 20, "stoppedtwice": 20})
    with open(session + ".jsonl", "a", encoding="utf-8") as handle:   # stoppedtwice 第二次被停：晚于续做之后的最后一条记录
        second = record_at(15, "user", [{"tool_use_id": "s9", "type": "tool_result", "content": "stopped"}])
        second["toolUseResult"] = {"message": "Successfully stopped task: stoppedtwice (样本)", "task_id": "stoppedtwice", "task_type": "local_agent"}
        handle.write(json.dumps(second, ensure_ascii=False) + "\n")
    # 「跑满 N 小时要主 agent 问一次」：派发 61 分钟、没收到过消息的报 1 小时；派发 125 分钟的报 2 小时（--ack 了 1 小时那一格照报，见下面的看门狗）；
    # 交回的、任务通知 failed 的不报；中途（第一格之前）收到消息不重新计时、照报；第一格之后收到过消息的（送达的两种写法，
    # 与还在主会话记录里、没送达的 SendMessage）不报。每个都在最后留一条 30 秒前起的工具调用，免得别的告警混进来。
    for inquiry_agent_id, minutes_since_dispatch, inquiry_middle_records, inquiry_description in (
            ("inquiry61", 61, [], "派发 61 分钟、没收到过消息"),
            ("inquiry125", 125, [], "派发 125 分钟、没收到过消息"),
            ("inquirymessagebefore", 90, [record_at(40, "user", "主 agent：线程上限 4", origin_kind="coordinator")], "派发 90 分钟、第一格之前收到过消息"),
            ("inquirydeliveredafter", 90, [record_at(20, "user", "主 agent 例行询问", origin_kind="coordinator")], "派发 90 分钟、第一格之后收到过消息"),
            ("inquiryqueuedafter", 90, [{"timestamp": record_at(20, "user", "")["timestamp"], "type": "attachment",
                                         "attachment": {"type": "queued_command", "prompt": "主 agent 例行询问", "origin": {"kind": "coordinator"}, "isMeta": True}}],
             "派发 90 分钟、第一格之后忙时收到过消息（排进队）")):
        write_transcript(session, inquiry_agent_id, [bash_use(minutes_since_dispatch, "t1", "ls", "m1"), bash_result(minutes_since_dispatch - 0.1, "t1"),
                                                     *inquiry_middle_records, bash_use(0.5, "t2", "cargo build", "m2")],
                         {"agentType": "implementation-writer", "description": inquiry_description})
    write_transcript(session, "inquiryhandedback", [bash_use(90, "t1", "ls", "m1"), bash_result(89.9, "t1"), *handback_records(10, "h1", "m2"),
                                                    record_at(9.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                     {"agentType": "implementation-writer", "description": "派发 90 分钟、10 分钟前交回"})
    # 忙着跑一条长工具调用：主 agent 20 分钟前发的消息还在主会话记录里、没落进它自己的会话记录
    write_transcript(session, "inquirysentafter", [bash_use(90, "t1", "cargo test --release", "m1")],
                     {"agentType": "implementation-writer", "description": "派发 90 分钟、一直在跑一条工具调用、主 agent 第一格之后发过消息"})
    # 中途停下没交回（文件头同名一段）：一个后台任务都没起过，干了一步就结束本轮、没调交回工具
    write_transcript(session, "midway", [bash_use(5, "t1", "ls", "m1"), bash_result(4.9, "t1"),
                                         record_at(4, "assistant", [{"type": "text", "text": "第三条做到一半，先停在这里"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "tooling-writer", "description": "做到一半结束本轮、没交回"})
    write_transcript(session, "inquiryfailed", [bash_use(90, "t1", "ls", "m1"), bash_result(89.9, "t1"),
                                                record_at(80, "assistant", [{"type": "text", "text": "接着跑"}], message_id="m2")],
                     {"agentType": "implementation-writer", "description": "派发 90 分钟、撞限额失败（任务通知 failed）"})
    with open(session + ".jsonl", "a", encoding="utf-8") as handle:   # 主会话记录里 SendMessage 结果的形态照 2026-09-24 主会话记录里的原样
        for sent_minutes_ago, sent_result in (
                (20, {"success": True, "message": "Message queued for delivery to inquirysentafter at its next tool round.",
                      "pin": {"id": "inquirysentafter", "name": "inquirysentafter", "ref": "e00001"}}),
                (5, {"success": True, "message": "Resuming agent inquiry", "resumedAgentId": "inquiryresumedsample",
                     "pin": {"id": "inquiryresumedsample", "name": "inquiryresumedsample", "ref": "e00002"}}),
                (5, {"success": False, "message": "Message queued for delivery to inquirynotsent at its next tool round.",
                     "pin": {"id": "inquirynotsent", "name": "inquirynotsent", "ref": "e00003"}})):
            sent_record = record_at(sent_minutes_ago, "user", [{"tool_use_id": "sm1", "type": "tool_result",
                                                                "content": [{"type": "text", "text": json.dumps(sent_result, ensure_ascii=False)}]}])
            sent_record["toolUseResult"] = sent_result
            handle.write(json.dumps(sent_record, ensure_ascii=False) + "\n")
        handle.write(json.dumps(record_at(79.5, "user", "<task-notification>\n<task-id>inquiryfailed</task-id>\n<status>failed</status>\n"
                                                        "<summary>Agent failed: API error</summary>\n</task-notification>", origin_kind="task-notification"),
                                ensure_ascii=False) + "\n")
        # 纯文字记录 stop_reason 为空、真结束本轮的两例（textsettled、textthenended）：主会话记录里这一轮的 completed 任务通知不早于那条文字
        for notified_agent, notified_minutes_ago in (("textsettled", 11.9), ("textthenended", 3.9)):
            handle.write(json.dumps(record_at(notified_minutes_ago, "user", f"<task-notification>\n<task-id>{notified_agent}</task-id>\n<status>completed</status>\n"
                                                                            "<summary>Agent completed</summary>\n</task-notification>", origin_kind="task-notification"),
                                    ensure_ascii=False) + "\n")
    messages_sent_to = set(session_task_events(session + ".jsonl")[2])
    if messages_sent_to != {"inquirysentafter", "inquiryresumedsample"}:
        failures.append(f"主会话记录里 SendMessage 发给子 agent 的成功结果应当认出排队与续做两种写法、success 不是 true 的不算："
                        f"应当是 inquirysentafter、inquiryresumedsample，实际 {sorted(messages_sent_to) or '无'}")
    background_started = record_at(4.9, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                  "content": "Command running in background with ID: bx1. Output is being written to: /tmp/x.output."}])
    finished_while_busy = {"timestamp": record_at(4.85, "user", "")["timestamp"], "type": "attachment",
                           "attachment": {"type": "queued_command", "prompt": "<task-notification>\n<task-id>bx1</task-id>\n<status>completed</status>\n</task-notification>"}}
    write_transcript(session, "nowake", [bash_use(5, "t1", "nohup bash gate.sh > gate.log 2>&1 &", "m1"), background_started, finished_while_busy,
                                         record_at(4.8, "assistant", [{"type": "text", "text": "等后台通知"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "gate-triage", "description": "起的后台任务当场完了、结束本轮去等"})
    woken = record_at(2, "user", "<task-notification>\n<task-id>bx1</task-id>\n<status>completed</status>\n</task-notification>")
    woken["origin"] = {"kind": "task-notification"}
    write_transcript(session, "wokenup", [bash_use(5, "t1", "bash gate.sh > gate.log 2>&1", "m1"), background_started,
                                          record_at(4.8, "assistant", [{"type": "text", "text": "等后台通知"}], stop_reason="end_turn", message_id="m2"), woken],
                     {"agentType": "gate-triage", "description": "等的后台任务完了、通知刚到、正要醒"})
    background_plain = record_at(14.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: bs1."}])
    write_transcript(session, "textsettled", [bash_use(15, "t1", "cargo test --release", "m1"), background_plain,
                                              record_at(12, "assistant", [{"type": "text", "text": "等 phase B 跑完"}], message_id="m2")],
                     {"agentType": "three-way-attack", "description": "结束本轮那条文字记录 stop_reason 为空、之后 12 分钟没接任何记录、主会话记录里有这一轮的完成通知"})
    # 先写一句文字、再一次 Write 整份生成：文字记录 stop_reason 为空，4 分钟了 Write 还没落，主会话记录里没有这一轮的通知 ⇒ 还在想，不报
    # （AGENT_WATCH_BREAK=settlenotification 退回只按时长认，判成结束本轮、报「中途停下没交回」：这一格红）
    write_transcript(session, "textthenwrite", [bash_use(10, "t1", "ls", "m1"), bash_result(9.9, "t1"),
                                                record_at(4, "assistant", [{"type": "text", "text": "下面整份写报告"}], message_id="m2")],
                     {"agentType": "tooling-writer", "description": "文字之后一次 Write 生成了 4 分钟还没落"})
    # 同样的文字记录，主会话记录里有这一轮的 completed 通知 ⇒ 真结束本轮，一个后台任务都没起过，照报「中途停下没交回」
    write_transcript(session, "textthenended", [bash_use(10, "t1", "ls", "m1"), bash_result(9.9, "t1"),
                                                record_at(4, "assistant", [{"type": "text", "text": "先停在这里"}], message_id="m2")],
                     {"agentType": "tooling-writer", "description": "文字记录 stop_reason 为空、主会话记录里有这一轮的完成通知"})
    write_transcript(session, "stopsequence", [bash_use(15, "t1", "cargo test --release", "m1"), background_plain,
                                               record_at(12, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="stop_sequence", message_id="m2")],
                     {"agentType": "three-way-attack", "description": "结束本轮写成 stop_sequence"})
    write_transcript(session, "textstreaming", [bash_use(15, "t1", "ls", "m1"), bash_result(14.9, "t1"),
                                                record_at(0.2, "assistant", [{"type": "text", "text": "接下来跑"}], message_id="m2")],
                     {"agentType": "three-way-attack", "description": "纯文字记录刚落、后面的工具调用还在路上"})
    orphan_output = os.path.join(work, "orphan-task.output")
    open(orphan_output, "w").close()
    held_output = os.path.join(work, "held-task.output")
    open(held_output, "w").close()
    holder = subprocess.Popen(["bash", "-c", f"exec 3>>'{held_output}'; exec sleep 60"])
    for sample_id, output_path in (("orphantask", orphan_output), ("heldtask", held_output)):
        started_record = record_at(9.9, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                  "content": f"Command running in background with ID: bo1. Output is being written to: {output_path}."}])
        write_transcript(session, sample_id, [bash_use(10, "t1", "cargo test --release", "m1"), started_record,
                                              record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                         {"agentType": "three-way-attack", "description": "等的后台任务：输出文件没人开着 / 还有进程开着"})
    os.makedirs(os.path.join(work, "orphan"), exist_ok=True)
    write_transcript(os.path.join(work, "orphan"), "noparent", waiting_then_idle, {"agentType": "gate-triage", "description": "主会话记录不在"})
    if not AgentTranscript("noparent", find_transcript("noparent", os.path.join(work, "orphan"))).parent_transcript_missing:
        failures.append("主会话记录不在时应当标出来（parent_transcript_missing），实际没标")
    # 主会话记录不在：纯文字记录 stop_reason 为空的退回按时长认，过了 TEXT_SETTLE_SECONDS 没接下文算结束本轮
    write_transcript(os.path.join(work, "orphan"), "noparenttext", [bash_use(10, "t1", "ls", "m1"), bash_result(9.9, "t1"),
                                                                     record_at(3, "assistant", [{"type": "text", "text": "先停在这里"}], message_id="m2")],
                     {"agentType": "tooling-writer", "description": "主会话记录不在、文字记录 stop_reason 为空、之后 3 分钟没接下文"})
    noparent_text_state = AgentTranscript("noparenttext", find_transcript("noparenttext", os.path.join(work, "orphan"))).state
    if noparent_text_state != "本轮结束、没交回":
        failures.append(f"子 agent noparenttext 的状态应当是「本轮结束、没交回」（主会话记录不在时纯文字记录按时长认），实际「{noparent_text_state}」")
    write_transcript(session, "costed", [
        record_at(30, "assistant", [{"type": "tool_use", "id": "c1", "name": "Read", "input": {"file_path": "/x/e999-preregistration.md"}}],
                  stop_reason="tool_use", message_id="k1",
                  usage={"input_tokens": 3, "cache_creation_input_tokens": 100_000, "cache_read_input_tokens": 0, "output_tokens": 50}),
        record_at(29.9, "user", [{"type": "tool_result", "tool_use_id": "c1", "content": "登" * 400}]),
        record_at(20, "assistant", [{"type": "tool_use", "id": "c2", "name": "Bash", "input": {"command": "cargo test --release"}}],
                  stop_reason="tool_use", message_id="k2",
                  usage={"input_tokens": 2, "cache_creation_input_tokens": 150_000, "cache_read_input_tokens": 0, "output_tokens": 70}),
        record_at(19.9, "user", [{"type": "tool_result", "tool_use_id": "c2", "content": "ok" * 50}]),
        record_at(19.8, "assistant", [{"type": "text", "text": "交回"}], stop_reason="end_turn", message_id="k3",
                  usage={"input_tokens": 1, "cache_creation_input_tokens": 200, "cache_read_input_tokens": 150_002, "output_tokens": 30}),
    ], {"agentType": "experiment-runner", "description": "用量样本"})
    costed = AgentTranscript("costed", find_transcript("costed", session))
    wanted_cost = {"model_calls": 3, "first_context_tokens": 100_003, "fresh_input_tokens": 250_206, "cache_read_tokens": 150_002,
                   "output_tokens": 150, "full_cache_rewrites": 1, "full_cache_rewrite_tokens": 150_000,
                   "total_context_tokens": 100_003 + 150_002 + 150_203}
    # 读到的那几样逐个点名（分派表），不按字段名 getattr：函数单元库看得见读的是哪几个属性
    got_cost = {"model_calls": costed.model_calls, "first_context_tokens": costed.first_context_tokens, "fresh_input_tokens": costed.fresh_input_tokens,
                "cache_read_tokens": costed.cache_read_tokens, "output_tokens": costed.output_tokens, "full_cache_rewrites": costed.full_cache_rewrites,
                "full_cache_rewrite_tokens": costed.full_cache_rewrite_tokens, "total_context_tokens": costed.total_context_tokens}
    for field, wanted in wanted_cost.items():
        got = got_cost[field]
        if BROKEN_DETECTION == "cost" and field == "full_cache_rewrite_tokens":
            got = 0
        if got != wanted:
            failures.append(f"用量样本的 {field} 应当是 {wanted}，实际 {got}")
    if costed.tool_result_characters.get("Read 跑前登记（整份）") != [1, 400] or costed.tool_result_characters.get("Bash cargo") != [1, 100]:
        failures.append(f"工具结果分类不对：{costed.tool_result_characters}")
    expectations = {
        "finished": set(), "healthy": set(), "boundedloop": set(), "interrupted": set(), "repeatchanging": set(), "waiting": set(), "continued": set(),
        "stoppedidle": set(), "stoppedcontinued": set(), "stoppedtwice": set(), "wokenup": set(), "resumedafterturnend": set(), "nowake": {"结束本轮却不会醒"}, "waitinglong": set(), "orphantask": {"结束本轮却不会醒"}, "heldtask": set(), "textsettled": set(), "stopsequence": set(), "textstreaming": set(),
        "textthenwrite": set(), "textthenended": {STOPPED_MIDWAY_ALERT},
        "toolafterhandback": set(), "movedbackground": set(), "queuedresume": {STOPPED_MIDWAY_ALERT}, "midway": {STOPPED_MIDWAY_ALERT},
        "hugecontext": {HANDOVER_ALERT}, "waitloop": {"等待循环"}, "longtool": {"工具调用过长"}, "stalepending": set(), "stalependingthenresult": set(), "repeat": {"同一命令反复且输出不变"}, "banned": {"禁用命令"}, "bannedrefused": set(), "bannedold": set(), "bannedheredoc": set(), "bannedafterheredoc": {"禁用命令"}, "idle": {"无动静"},
        "inquiry61": {"跑满 1 小时要主 agent 问一次"}, "inquiry125": {"跑满 2 小时要主 agent 问一次"}, "inquiryhandedback": set(),
        "inquirymessagebefore": {"跑满 1 小时要主 agent 问一次"}, "inquirydeliveredafter": set(), "inquiryqueuedafter": set(),
        "inquirysentafter": {"工具调用过长"}, "inquiryfailed": {STOPPED_MIDWAY_ALERT},
    }
    now = datetime.now(timezone.utc)
    for agent_id, wanted in expectations.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        got = {name for name, _, _ in agent_alerts(transcript, now, thresholds)}
        if got != wanted:
            failures.append(f"子 agent {agent_id}：应当告警 {sorted(wanted) or '无'}，实际 {sorted(got) or '无'}")
    # 线写 0 不报「该交接」：同一份 70 万上下文的 hugecontext，--context-tokens 0 时一条「该交接」都不该有
    zero_line = argparse.Namespace(**dict(vars(thresholds), context_tokens=0))
    got_at_zero_line = {name for name, _, _ in agent_alerts(AgentTranscript("hugecontext", find_transcript("hugecontext", session)), now, zero_line)}
    if HANDOVER_ALERT in got_at_zero_line:
        failures.append(f"子 agent hugecontext 在 --context-tokens 0 时不该报「{HANDOVER_ALERT}」，实际 {sorted(got_at_zero_line)}")
    # 「中途停下没交回」在报告里另起一节：被看的三个里两个（midway、queuedresume），逐个一行带重派命令；longtool 在跑、不算
    midway_lines, _midway_alerts, *_ = build_report(["midway", "queuedresume", "longtool"], session,
                                                   argparse.Namespace(**dict(vars(thresholds), kept_copy_hours=6)), time.time(), set(), None)
    if not (any(line.startswith("中途停下没交回的子 agent：被看的 3 个里 2 个") for line in midway_lines)
            and any(line.startswith("  子 agent midway tooling-writer") and "第三条做到一半" in line
                    and "agent-handover.py --agent midway --out" in line for line in midway_lines)):
        failures.append(f"报告里应当另起一节「中途停下没交回的子 agent：被看的 3 个里 2 个」，逐个一行写最后一句与 agent-handover.py 重派命令，实际 {midway_lines}")
    held_lines = background_task_processes(AgentTranscript("heldtask", find_transcript("heldtask", session)))
    if not any(f"进程 {holder.pid} 已跑" in line for line in held_lines):
        failures.append(f"结束本轮等自己的后台任务时应当列出开着那个输出文件的进程 {holder.pid}，实际 {held_lines or '一个都没列'}")
    holder.kill()
    holder.wait()
    # subagent_transcript.py 的两份共用样本（续做闸与 agent-cost.py 的自证判的是同两份）：先被交回闸拒、后来送达的算交回；被拒之后再没送达的不算
    shared_base = datetime.fromtimestamp(iso_minutes_ago(20), timezone.utc)
    write_transcript(session, "refusedthendelivered", transcripts.sample_refused_then_delivered(shared_base), {"agentType": "clerk", "description": "先被拒后送达"})
    write_transcript(session, "refusedonly", transcripts.sample_refused_only(shared_base), {"agentType": "clerk", "description": "被拒之后再没送达"})
    wanted_done = {"finished": True, "interrupted": True, "stoppedidle": True, "waiting": False, "continued": False, "healthy": False,
                   "stoppedcontinued": False, "stoppedtwice": True, "nowake": False, "wokenup": False, "resumedthinking": False, "notifiedafterhandback": True,
                   "waitinglong": False, "toolafterhandback": True, "movedbackground": False, "queuedresume": False, "inquiryhandedback": True,
                   "refusedthendelivered": True, "refusedonly": False}
    for agent_id, wanted_state in (("textsettled", "本轮结束、没交回"), ("stopsequence", "本轮结束、没交回"), ("textstreaming", "思考中"),
                                   ("textthenwrite", "思考中"), ("textthenended", "本轮结束、没交回"),
                                   ("resumedafterturnend", "思考中")):
        got_state = AgentTranscript(agent_id, find_transcript(agent_id, session)).state
        if got_state != wanted_state:
            failures.append(f"子 agent {agent_id} 的状态应当是「{wanted_state}」，实际「{got_state}」")
    for agent_id, wanted in wanted_done.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        if transcript.is_done() != wanted:
            failures.append(f"子 agent {agent_id} 应当{'算' if wanted else '不算'}交回或被停，实际状态「{transcript.state}」")
    moved_background_transcript = AgentTranscript("movedbackground", find_transcript("movedbackground", session))
    if moved_background_transcript.background_started != ["bm1"] or not moved_background_transcript.waiting_on_its_own_background_tasks():
        failures.append(f"前台命令超时被挪进后台（ID bm1）之后结束本轮的，应当算在等自己的后台任务，实际数到起过的后台任务 "
                        f"{moved_background_transcript.background_started}、在不在等：{moved_background_transcript.waiting_on_its_own_background_tasks()}")
    stale_file = os.path.join(work, "stale-output.log")
    open(stale_file, "w").close()
    old = time.time() - 3600
    os.utime(stale_file, (old, old))
    sleeper = subprocess.Popen(["bash", "-c", f"exec 3>>'{stale_file}'; touch -d '1 hour ago' '{stale_file}'; exec sleep 30"])
    busy_file = os.path.join(work, "busy-output.log")
    open(busy_file, "w").close()
    # 在算、只在结束时写的进程：写着的文件一小时没动，CPU 一直在涨（2026-09-19 c381-r1 攻方腿探针的形状）
    busy_ready = os.path.join(work, "busy-ready")   # busy 开了文件、进了忙等就建它
    busy = subprocess.Popen([sys.executable, "-c", "import os, time\n"
                             f"handle = open({busy_file!r}, 'a')\nos.utime({busy_file!r}, (time.time() - 3600, time.time() - 3600))\n"
                             f"open({busy_ready!r}, 'w').close()\n"
                             "deadline = time.time() + 30\nwhile time.time() < deadline:\n    pass\n"])
    probed_processes = [sleeper, busy]
    probes_started = time.time()
    try:
        # 按条件等两个进程进到要测的样子（sleeper exec 成 sleep，busy 开着文件进了忙等），再补足 4 秒（「进程无输出」要已跑 3 秒）：
        # 负载高时 python 起得慢，固定等 4 秒时 busy 可能还没开文件
        wait_until(lambda: os.path.exists(busy_ready) and (process_arguments(sleeper.pid) or [""])[0] == "sleep")
        time.sleep(max(0.0, 4 - (time.time() - probes_started)))
        process_lines, found = process_alerts(os.getpid(), set(), thresholds)
        if not any(name == "进程无输出" and str(sleeper.pid) in explanation for name, explanation, _ in found):
            failures.append(f"写着的文件一小时没动、CPU 也不涨的进程 {sleeper.pid} 没报「进程无输出」")
        if any(str(busy.pid) in explanation for _, explanation, _ in found):
            failures.append(f"写着的文件一小时没动、CPU 一直在涨的进程 {busy.pid} 被当成告警（它在算，只是不报进度）")
        if not any("在算" in line for line in process_lines):
            failures.append(f"CPU 在涨的进程 {busy.pid} 在报告里没有「在算」那一行")
    finally:
        for process in (sleeper, busy):
            process.kill()
            process.wait()
    pipelines_checked = selftest_pipe_upstream(work, thresholds, failures, probed_processes)
    verdict_logs_checked = selftest_verdict_lines(work, thresholds, failures, probed_processes)
    verdict_logs_checked += selftest_verdict_selftest_scratch(work, thresholds, failures, probed_processes)
    reminder_timers_checked = selftest_reminder_timer(work, thresholds, failures)
    watch_runs = []

    def run_watch(watch_arguments, timeout=None):
        watch_runs.append(watch_arguments)
        return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", *watch_arguments, "--session-dir", session, "--process-root-pid", "0"],
                              capture_output=True, text=True, timeout=timeout)

    watch_exit = run_watch(["--agents", "finished,interrupted,stoppedidle", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
    if watch_exit != 0:
        failures.append(f"被看的子 agent 都交回或被停时看门狗应当退出码 0，实际 {watch_exit}")
    watch_exit = run_watch(["--agents", "longtool", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
    if watch_exit != 3:
        failures.append(f"有告警（工具调用过长）时看门狗应当退出码 3，实际 {watch_exit}")
    waitloop_run = run_watch(["--agents", "waitloop", "--interval-seconds", "1", "--max-minutes", "0.03"])
    if waitloop_run.returncode != 4 or "只记不叫醒：等待循环" not in waitloop_run.stdout:
        failures.append(f"只有「等待循环」这类只记不叫醒的告警时，看门狗应当到点退 4 并在报告里写「只记不叫醒：等待循环」，"
                        f"实际退出码 {waitloop_run.returncode}：{waitloop_run.stdout[-300:]}")
    watch_exit = run_watch(["--agents", "healthy", "--interval-seconds", "1", "--max-minutes", "0.03"]).returncode
    if watch_exit != 4:
        failures.append(f"到定时回报的时刻看门狗应当退出码 4，实际 {watch_exit}")
    acknowledged_run = run_watch(["--agents", "longtool", "--ack", "longtool:工具调用过长", "--interval-seconds", "1", "--max-minutes", "0.03"])
    if acknowledged_run.returncode != 4 or "已确认接着盯" not in acknowledged_run.stdout:
        failures.append(f"「工具调用过长」被 --ack 确认之后看门狗应当到点退出码 4 并写「已确认接着盯」，实际退出码 {acknowledged_run.returncode}")
    # 派发 125 分钟、--ack 了 1 小时那一格：照报 2 小时，出路是例行询问；--ack 了 2 小时那一格才不叫醒
    first_mark_acknowledged_run = run_watch(["--agents", "inquiry125", "--ack", "inquiry125:跑满 1 小时", "--interval-seconds", "1", "--max-minutes", "1"])
    wanted_inquiry_lines = ("⚠️ 跑满 2 小时要主 agent 问一次（inquiry125）：", f"   → 怎么办：{ROUTINE_INQUIRY_NEXT_STEP}\n")
    if first_mark_acknowledged_run.returncode != 3 or not all(line in first_mark_acknowledged_run.stdout for line in wanted_inquiry_lines):
        failures.append(f"派发 125 分钟、--ack 了「跑满 1 小时」那一格时看门狗应当照报 2 小时那一格并退出码 3，要有这两行 {wanted_inquiry_lines}，"
                        f"实际退出码 {first_mark_acknowledged_run.returncode}，输出：{first_mark_acknowledged_run.stdout[-400:]}")
    second_mark_acknowledged_run = run_watch(["--agents", "inquiry125", "--ack", "inquiry125:跑满 2 小时", "--interval-seconds", "1", "--max-minutes", "0.03"])
    if second_mark_acknowledged_run.returncode != 4 or "已确认接着盯（--ack inquiry125:跑满 2 小时要主 agent 问一次）" not in second_mark_acknowledged_run.stdout:
        failures.append(f"派发 125 分钟、--ack 了「跑满 2 小时」那一格时看门狗不该叫醒：应当到点退出码 4 并写「已确认接着盯」，"
                        f"实际退出码 {second_mark_acknowledged_run.returncode}，输出：{second_mark_acknowledged_run.stdout[-400:]}")
    watched = run_watch(["--agents", "waiting,finished", "--interval-seconds", "1", "--max-minutes", "0.03"])
    wanted_active_line = "还没交回、没被停的子 agent（1 个，另有还没会话记录的 0 个）：waiting"
    if watched.returncode != 4 or wanted_active_line not in watched.stdout:
        failures.append(f"一个结束本轮没交回、一个已交回时看门狗应当退出码 4 并列出「{wanted_active_line}」，"
                        f"实际退出码 {watched.returncode}，输出：{watched.stdout[-300:]}")
    # 出路句印的是 watch.sh 认得的写法（照抄能起），不是 agent-watch.py 自己的 --agents（watch.sh 退 2「不认识的选项」）
    wanted_restart_line = "   → 接着盯就用 run_in_background 再起一个看门狗：bash scripts/agents/watch.sh waiting\n"
    if wanted_restart_line not in watched.stdout:
        failures.append(f"定时回报的出路句应当是「{wanted_restart_line.strip()}」，实际输出：{watched.stdout[-300:]}")
    try:
        started = time.time()
        watched = run_watch(["--agents", "waiting", "--interval-seconds", "240", "--max-minutes", "0.05"], timeout=90)   # 坏了要等 240 秒；90 秒留给负载高时起得慢
        if watched.returncode != 4:
            failures.append(f"复检间隔 240 秒、回报周期 3 秒时看门狗应当到点退出码 4，实际 {watched.returncode}")
    except subprocess.TimeoutExpired:
        failures.append(f"复检间隔比回报周期长时看门狗没有按回报周期退出（{time.time() - started:.0f} 秒还没退出）：回报会被拖到下一次复检")
    detections_file = os.path.join(work, "detections.jsonl")
    open(detections_file, "w").close()
    self_background = "run_in_background 里又自己放后台"
    for session_label, finding_text, agent_type, agent_id, wanted_exit, wanted_note in (
            ("session", self_background, "experiment-runner", "healthy", 3, None),
            ("session", "写被拒", "experiment-runner", "healthy", 4, "只记不叫醒"),
            ("另一个会话", self_background, "experiment-runner", "healthy", 4, None),
            ("session", "没有 timeout 的等待循环", "experiment-runner", "healthy", 4, None),
            ("session", self_background, "experiment-runner", "别的子agent", 4, None),
            ("session", "写被拒", "主 agent", None, 4, "主 agent 自己的命令被 hook 拒了"),
            ("session", "重型测试被拒：cargo test 带 --workspace / --all（全量测试）", "experiment-runner", "healthy", 4, "执行前已被 hook 拒了"),
            ("session", "提交时才跑的检查被拒：.claude/singlefs-ai-sop/scripts/doc-lint.sh（提交时才跑的检查）", "主 agent", None, 4, "执行前已被 hook 拒了")):
        watch_runs.append(["--detections-file", session_label, finding_text, agent_type, agent_id])
        # 看门狗取完检出偏移、做完第一次整查（按它建的就绪文件等，不按固定秒数）之后再写检出：负载高时 python 起得慢，
        # 固定等 1.5 秒时它可能还没取偏移，这一条检出落在偏移之前、读不到
        detection_ready = os.path.join(work, f"ready-detections-{len(watch_runs)}")
        watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--agents", "healthy", "--session-dir", session,
                                    "--interval-seconds", "2", "--detection-poll-seconds", "0.5", "--max-minutes", "0.12",
                                    "--process-root-pid", "0", "--detections-file", detections_file],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                   env=dict(os.environ, **{SELFTEST_READY_VARIABLE: detection_ready}))
        wait_for_selftest_ready(detection_ready, watcher)
        with open(detections_file, "a", encoding="utf-8") as handle:
            handle.write(json.dumps({"session_id": session_label, "agent_type": agent_type, "agent_id": agent_id,
                                     "command": "pgrep -f x", "findings": [finding_text]}, ensure_ascii=False) + "\n")
        output, _ = watcher.communicate(timeout=60)
        if watcher.returncode != wanted_exit or (wanted_exit == 3 and "hook 检出" not in output) \
                or (wanted_note is not None and wanted_note not in output):
            failures.append(f"检出记录属于「{session_label}」「{agent_type} {agent_id}」、检出「{finding_text}」时看门狗应当退出码 {wanted_exit}"
                            f"{f'并在报告里记一行「{wanted_note}」' if wanted_note else ''}，实际 {watcher.returncode}")
    # --processes-only：拿自检进程自己当那个 Claude 实例。一个一直在写文件的、一个写着的文件一小时没动而且不占 CPU 的，
    # 只许后一个报「进程无输出」；一个两秒就退出、退出之后没人收尸（僵尸）的，盯到它退出就退 0；一个都没有也退 0 并写明。
    def run_process_watch(watch_arguments, timeout=120):
        watch_runs.append(["--processes-only", *watch_arguments])
        return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--processes-only", "--process-root-pid", str(os.getpid()),
                               "--interval-seconds", "1", *watch_arguments], capture_output=True, text=True, timeout=timeout)

    writing_file = os.path.join(work, "writing-output.log")
    writer = subprocess.Popen([sys.executable, "-c", "import time\n"
                               f"handle = open({writing_file!r}, 'a')\n"
                               "deadline = time.time() + 60\nwhile time.time() < deadline:\n"
                               "    handle.write('一行\\n')\n    handle.flush()\n    time.sleep(0.3)\n"])
    silent_file = os.path.join(work, "silent-output.log")
    open(silent_file, "w").close()
    silent = subprocess.Popen(["bash", "-c", f"exec 3>>'{silent_file}'; touch -d '1 hour ago' '{silent_file}'; exec sleep 60"])
    probed_processes += [writer, silent]
    try:
        # 按条件等：writer 写出了第一行、silent exec 成了 sleep，再起看门狗（负载高时 python 起得慢，
        # writer 还没开文件时它写着的只有继承来的标准输出，会被当成不写的进程报「进程无输出」）
        wait_until(lambda: os.path.isfile(writing_file) and os.path.getsize(writing_file) > 0 and (process_arguments(silent.pid) or [""])[0] == "sleep")
        writer_and_silent_run = run_process_watch(["--max-minutes", "0.5", "--process-report-minutes", "0.02", "--process-stale-minutes", "0.05",
                                     "--process-max-minutes", "600"])
        alert_lines = [line for line in writer_and_silent_run.stdout.splitlines() if line.startswith("⚠️")]
        if writer_and_silent_run.returncode != 3 or not any("进程无输出" in line and f"进程 {silent.pid} " in line for line in alert_lines):
            failures.append(f"--processes-only 时写着的文件一小时没动、CPU 也不涨的进程 {silent.pid} 应当报「进程无输出」并退出码 3，"
                            f"实际退出码 {writer_and_silent_run.returncode}，告警：{alert_lines or '无'}")
        if any(f"进程 {writer.pid} " in line for line in alert_lines):
            failures.append(f"--processes-only 时一直在写文件的进程 {writer.pid} 被当成告警：{alert_lines}")
    finally:
        for process in (writer, silent):
            process.kill()
            process.wait()
    # 看门狗记下要盯的进程、做完第一次整查（按它建的就绪文件等）之后才停 short，停了先不收尸，就是个僵尸。
    # 不用「两秒就退」的 sleep 2：负载高时看门狗起得比两秒慢，起来时 short 已是僵尸、不算要盯的，判成「就没有要盯的进程」
    short = subprocess.Popen(["sleep", "300"])
    probed_processes.append(short)
    try:
        exit_ready = os.path.join(work, "ready-processes-exit")
        watch_runs.append(["--processes-only", "--max-minutes", "0.25"])
        exit_watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--processes-only", "--process-root-pid", str(os.getpid()),
                                         "--interval-seconds", "1", "--max-minutes", "0.25"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                        env=dict(os.environ, **{SELFTEST_READY_VARIABLE: exit_ready}))
        wait_for_selftest_ready(exit_ready, exit_watcher)
        short.kill()
        finished_output, _ = exit_watcher.communicate(timeout=120)
        if exit_watcher.returncode != 0 or "都退出了" not in finished_output:
            failures.append(f"--processes-only 盯着的进程退出了（僵尸没人收尸也算）应当退出码 0 并写「都退出了」，"
                            f"实际退出码 {exit_watcher.returncode}，输出：{finished_output[-300:]}")
    finally:
        short.kill()
        short.wait()
    nothing = run_process_watch(["--max-minutes", "0.25"])
    if nothing.returncode != 0 or "就没有要盯的进程" not in nothing.stdout:
        failures.append(f"--processes-only 起的时候一个要盯的进程都没有，应当退出码 0 并写明，实际退出码 {nothing.returncode}，输出：{nothing.stdout[-300:]}")
    if "这一类没查" not in nothing.stdout:
        failures.append(f"--processes-only 没给 --session-dir 时报告里应当写明「交回之后后台还在跑」这一类没查，实际输出：{nothing.stdout[-300:]}")
    leftover_cases_checked = selftest_leftover_background(work, thresholds, failures, watch_runs, probed_processes)
    leftover_cases_checked += selftest_kept_copies(work, thresholds, failures)
    session_watch_extra_checks = selftest_scratch_hogs(work, thresholds, failures, watch_runs) + selftest_stale_sessions(work, failures, watch_runs)
    session_watch_extra_checks += selftest_root_free(work, failures, watch_runs)
    heavy_processes_started = selftest_heavy_tests(work, failures, probed_processes, watch_runs)
    heavy_processes_started += selftest_heavy_selftest_tree(work, failures, probed_processes)
    heavy_processes_started += selftest_light_tier_tree(work, failures, probed_processes)
    memory_processes_started = selftest_memory(work, failures, probed_processes, watch_runs)
    core_processes_started = selftest_core_usage(work, failures, probed_processes, watch_runs)
    core_processes_started += selftest_core_usage_aggregated(work, failures, probed_processes)
    termination_checks = selftest_unacknowledged_terminations(work, failures, probed_processes, watch_runs)
    session_watch_checks = selftest_session_watch(work, thresholds, failures, watch_runs) + session_watch_extra_checks
    session_watch_checks += selftest_scratch_mentions(failures)
    session_watch_checks += selftest_stale_scratch_cleanup(work, failures)
    session_watch_checks += selftest_ack_concurrency(work, failures)
    help_run = subprocess.run([sys.executable, os.path.abspath(__file__), "--help"], capture_output=True, text=True, timeout=60)
    session_watch_checks += 1
    if help_run.returncode != 0 or "usage:" not in help_run.stdout:
        failures.append(f"--help 应当退出码 0 并打出用法（argparse 的 help 文本里百分号要写成 %%），实际 {help_run.returncode}：{help_run.stderr[-200:]}")
    for mixed in (["watch", "--processes-only", "--agents", "healthy"], ["cost", "--processes-only"]):
        mixed_run = subprocess.run([sys.executable, os.path.abspath(__file__), *mixed], capture_output=True, text=True, timeout=120)
        if mixed_run.returncode != 2:
            failures.append(f"{' '.join(mixed)} 应当退出码 2（--processes-only 不和子 agent、用量混用），实际 {mixed_run.returncode}")
    for wrapped_command, wanted_display in (
            ("/bin/bash -c source s.sh 2>/dev/null || true && eval 'cargo test\n  --release' < /dev/null && pwd -P >| /tmp/claude-1-cwd", "cargo test --release"),
            ("/bin/bash -c source s.sh 2>/dev/null || true && eval 'grep '\"'\"'x'\"'\"' f' && pwd -P >| /tmp/claude-2-cwd", "grep 'x' f"),
            ("sleep 60", "sleep 60")):
        if command_for_display(wrapped_command) != wanted_display:
            failures.append(f"Bash 工具包着的命令在报告里应当显示成「{wanted_display}」，实际「{command_for_display(wrapped_command)}」")
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("    → 看 agent_alerts() / process_alerts() / heavy_test_report() / memory_report() / read_detections() / run() 的判法；"
              "AGENT_WATCH_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静、进程无输出六种告警都报得出，"
          f"交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具送达判（只结束本轮、交回后又被续做的、被交回闸拒之后再没送达的都不算，先被拒后送达的算，忙时排进队的续做消息也算续做；"
          f"交回之后被自己后台任务的完成通知叫醒、又调了工具再结束本轮的仍算交回），前台命令超时转进后台的也算它起过的后台任务，"
          f"被停认会话记录里的打断与主会话记录里的 TaskStop（停在等后台任务时的算、停了又被续做的不算、续做之后又停的算，主会话记录不在的标出来），"
          f"结束本轮而起过的后台任务都已完成、没有通知在路上的报「结束本轮却不会醒」（通知刚到正要醒的不报），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒（钩子 ⑭ 判不了的长活只记不叫醒），用量与工具结果分类对；结束本轮认 end_turn 与 stop_sequence、stop_reason 为空的纯文字记录认主会话记录里这一轮的任务通知（文字之后 4 分钟还在生成、没有通知的不认，主会话记录不在时两分钟没接下文也认），结束本轮之后收到续做消息就算在想，用 --ack 确认过的告警只进报告不叫醒，主 agent 自己被拒的写只进报告不叫醒、入口已拒的命令不再按禁用命令报、别的子 agent 的检出不叫醒这一个看门狗，等自己的后台任务时列出开着输出文件的进程，结束本轮等自己的后台任务的不报无动静（输出文件已没有进程开着、又没收到通知的后台任务不算在等，报「结束本轮却不会醒」），写着的文件不动而 CPU 在涨的进程只报「在算」不告警，stdin 是管道的管道尾上游在算只报「在等上游」、上游睡着或写端只在自己祖先手里照报「进程无输出」（{pipelines_checked} 个管道），remind.sh start 底下等到点的 sleep 不报、同形的 remind.sh stop 与别的脚本名底下的 sleep 照报（{reminder_timers_checked} 个假提醒脚本）；--processes-only 只盯进程：一直在写文件的不报、写着的文件不动且 CPU 不涨的报「进程无输出」，"
          f"盯着的进程都退出了（退出之后没人收尸的僵尸也算）退 0、起的时候一个都没有也退 0 并写明，和 --agents、cost 混用退 2，Bash 工具包着的命令在报告里只列 eval 里那一段；"
          f"还没交回的子 agent 从派发起每跨过一个整点报「跑满 N 小时要主 agent 问一次」（报已跨过的最近那一格，--ack <agent 号>:跑满 N 小时 只确认那一格；"
          f"第一格之前收到的消息不重新计时，那一格之后主 agent 发过消息的不报——送达的 user 记录与排队附件、主会话记录里还没送达的 SendMessage 三处都认；"
          f"交回的、任务通知 failed 的不报）；"
          f"交回、被 TaskStop 停掉、最近一次任务通知是 failed 的子 agent 过了宽限还有进程开着它后台任务的输出文件报「交回之后后台还在跑」（交回不到宽限、没交回还在等、"
          f"stopped 之后又被续做的不报，进程没了不报，--ack 进程:<pid> 不叫醒），被看的全部交回时退出前也报、只剩宽限里的等宽限过了再查，--processes-only 带 --session-dir 也报、不带就写明没查；"
          f"交回超过 6 小时、交回正文或它点名的报告里写了「没删 <全路径>」而路径还在的报「{KEPT_COPY_ALERT}」（交回不到 6 小时、已经删掉的、没交回的不报，"
          f"--ack <agent 号>:{KEPT_COPY_ALERT} 不叫醒）；"
          f"checker 档包 singlefs-checker-tier 的测试二进制在跑而环境里没有 {HEAVY_TEST_VARIABLE}=commit / =user-request 的报「{HEAVY_TEST_ALERT}」（带别的值照报，带这两个值的、"
          f"bash -c 里写着带前缀而还没起的、timeout 包着带前缀的不报，另一个看门狗的自检那一支整棵不扫，stage-selftest.sh 与带 --selftest 的脚本底下起的 checker-tier-crates-mutation-replay 不报、直接起的照报，不重型的门禁阶段（提交时才跑的检查）不报，一条链只报最上面那个、下一步给从最底层往上的停止次序，归属认出开着输出文件的子 agent；"
          f"登记成轻的自证（--selftest exec 进本体的也认）底下与它临时目录里起的驱动不报，没有自证祖先的、设了 heavy-if-set 那个变量的、仓副本里的自证底下的照报；经内存包装起的轻型三条（crash-point-replay.sh --quick）底下的 checker 档 cargo 不报、同一棵树不带 --quick 的照报），"
          f"--processes-only 与 --agents 两条路都报、--ack 进程:<pid> 不叫醒；常驻内存过阈值的进程报「{MEMORY_ALERT}」（写常驻内存、阈值、归属与出路，"
          f"只睡的小进程不报，阈值 0 不查），两条路整查都报、--ack 进程:<pid> 不叫醒，看门狗起了之后才过线的由两次整查之间的单查几秒内叫醒，登记成轻的自证底下的过线进程不报、登记成 heavy 的照报（{memory_processes_started} 个进程）"
          f"；跑过一个窗口的重活进程 1 条线程忙等的报「{CORE_USAGE_ALERT}」、登记数那么多条线程忙等的不报、声明了串行的照报并标出理由，"
          f"变异门禁一趟八个干活进程各忙一条线按合计不报、三个都只睡的报在门禁进程上，"
          f"告警记录只报本会话、没过期、告警进程还活着的，--processes-only 真跑报、--ack 之后不叫醒（{core_processes_started} 个进程）"
          f"；上下文过线只对实现员与执行员报「该交接」并列出吃上下文最多的工具输出、线写 0 不报，等后台任务时 progress.md 久没改报「进度文件不涨」、整点询问附 progress.md 末几行"
          f"（progress.md 只在派发提示行首「草稿目录：」那一行点名的目录下找：提示里另点名的别的子 agent 的目录不取，没有那一行的报取不到），"
          f"hook 检出的主体是 agent 号，执行前已被 hook 拒了的检出只记进报告不叫醒；重启前旧进程留下的子 agent 记录（会话已不在、或最后一条记录早于现在这个进程的启动）判已失效不报；有子 agent 交回时草稿根里超阈值的编译目录与仓副本按最后修改时刻列在报告里；根分区可用量低于阈值报「{ROOT_FREE_ALERT}」（可用量换成假的判过线与不过线、阈值 0 不查，--agents 与 --processes-only 两条路都报、--ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT} 不叫醒）；--discover 一个会话一个看门狗（自己找没结束的子 agent、已有一个在盯就退 5）、--ack 落文件下一次读回、条件消失的确认作废（{session_watch_checks} 格）"
          f"；「进程卡住」类告警同一件连报到阈值、没 --ack 经 proc.py stop 终止那个进程并写「终止了 <pid>（<告警种类>，连报 N 次没处置）」，"
          f"有 --ack 的、阈值 0 的、中间断过的不终止，子 agent 的工具调用过长只报，一次至多终止一个，--processes-only 真跑三次第三次终止（{termination_checks} 格）"
          f"；常驻内存的阈值在内存包装里照包装上限的 90%、峰值表里有的照峰值的 1.5 倍（乘出来不到下限的取下限：记成 0.00 GiB 的条目配只睡的小进程不报、"
          f"峰值很小而常驻过了下限的照报）、根进程自己被包着的退回固定阈值；"
          f"没起过后台任务、结束本轮没交回的报「{STOPPED_MIDWAY_ALERT}」并在报告里另起一节带重派命令；跑得久的进程写着的文件里数判定行，"
          f"有 ✗ 的报「{VERDICT_RED_ALERT}」、全绿的只进报告，登记成轻的自证在自己临时目录里写的 ✗ 不报、写在外面的与没有自证上层的照报（{verdict_logs_checked} 份日志）"
          f"（查了 {len(expectations) + 1 + leftover_cases_checked + 1} 个子 agent、{len(probed_processes)} 个进程"
          f"（其中假的重型进程 {heavy_processes_started} 个）、{len(watch_runs)} 次看门狗）")
    return 0


def main():
    if len(sys.argv) >= 2 and sys.argv[1] == "--selftest":
        return selftest()
    parser = argparse.ArgumentParser(description="子 agent 与它们起的进程的定时监控")
    parser.add_argument("mode", choices=["report", "watch", "cost"])
    parser.add_argument("--agents", help="逗号分隔的子 agent id（派发结果里的 agentId）")
    parser.add_argument("--session-dir", help="~/.claude/projects/<项目>/<会话 id>；给了 --agents 时可省")
    parser.add_argument("--interval-seconds", type=float, default=240, help="看门狗两次检查的间隔，默认 240 秒")
    parser.add_argument("--max-minutes", type=float, default=60, help="定时回报：没有告警也每隔这么久退出一次叫醒主 agent，默认 60 分钟")
    parser.add_argument("--tool-minutes", type=float, default=8, help="一次工具调用超过这么久告警，默认 8 分钟")
    parser.add_argument("--wait-loop-minutes", type=float, default=3, help="没有超时的等待循环跑过这么久告警，默认 3 分钟")
    parser.add_argument("--idle-minutes", type=float, default=10, help="没有工具在跑、记录这么久没动告警，默认 10 分钟")
    parser.add_argument("--repeat-count", type=int, default=3, help="最近 10 次命令里同一条出现这么多次告警，默认 3")
    parser.add_argument("--context-tokens", type=int, default=700_000,
                        help="implementation-writer / experiment-runner 最后一次调用的上下文到这么多 token 报「该交接」，默认 70 万（推的）；别的类型不报")
    parser.add_argument("--progress-stale-minutes", type=float, default=30,
                        help="结束本轮在等后台任务的子 agent，草稿目录里的 progress.md 这么久没改就报「进度文件不涨」，默认 30 分钟；写 0 不报")
    parser.add_argument("--discover", action="store_true",
                        help="一个会话一个看门狗：自己从 --session-dir 找还没结束的子 agent（每次检查重找，新派的自动接上），已有一个在盯就退 5；"
                             "--ack 记进这个会话的状态目录，下一次起的时候读回，条件类告警的确认在条件消失时作废，事件类到那个子 agent 结束才作废")
    parser.add_argument("--process-report-minutes", type=float, default=5, help="进程跑过这么久才列出来，默认 5 分钟")
    parser.add_argument("--process-stale-minutes", type=float, default=20, help="进程写着的文件这么久没动告警，默认 20 分钟")
    parser.add_argument("--process-max-minutes", type=float, default=120, help="进程跑过这么久告警，默认 120 分钟")
    parser.add_argument("--not-started-minutes", type=float, default=5, help="派发之后这么久还没有会话记录告警，默认 5 分钟")
    parser.add_argument("--leftover-background-grace-minutes", type=float, default=2,
                        help="子 agent 交回、被停或失败之后过了这么久，它起的后台任务还有进程开着输出文件就报「交回之后后台还在跑」，默认 2 分钟")
    parser.add_argument("--kept-copy-hours", type=float, default=6,
                        help="子 agent 交回超过这么多小时、交回报告里写了「没删 <全路径>」而那个路径还在，就报「交回之后副本还在」交主 agent 删；默认 6 小时，写 0 不查")
    parser.add_argument("--scratch-idle-hours", type=float, default=6,
                        help="草稿根（工作根，与旧根下本会话的那一份）里的编译目录（带 CACHEDIR.TAG）与仓副本最后修改距今超过这么多小时，有子 agent 交回时列在报告里"
                             "（只报不删，交主 agent 核过没人在用再删）；默认 6 小时（推的：与 --kept-copy-hours 同一档，一个子 agent 一件活通常不到这么久，"
                             "停了这么久的副本多半没人再用），写 0 不查")
    parser.add_argument("--root-free-gib-alert", type=float, default=ROOT_FREE_GIB_ALERT_DEFAULT,
                        help="工作根所在文件系统（根分区）的可用量低于这么多 GiB 就报「根分区余量过低」、叫醒主 agent，每次整查都读；写 0 不查。"
                             "默认 128 GiB（推的，没量过）：crates 变异门禁一个工作进程有两份编译目录（普通的、插桩的），每份上限 64G"
                             "（.claude/gate.d/checker-tier-crates-mutation-replay.sh 的 GATE_MUTATION_TARGET_DIR_CAP），余量不到 128 GiB 就装不下再起一个工作进程；"
                             "双机驱动改前每趟留两份库拷贝（取回的与送出的，113 份合计 1159G、一份约 10G），放量段约每 8 分钟一趟，从 128 GiB 到写满约 45 分钟，够几次整查叫醒主 agent")
    parser.add_argument("--ask-every-minutes", type=float, default=60,
                        help="还没交回的子 agent 从派发起每跑满这么久（整数倍）报一次「跑满 N 小时要主 agent 问一次」，"
                             "那一格之后主 agent 给它发过消息就不报；默认 60 分钟，写 0 不报")
    parser.add_argument("--active-minutes", type=float, default=180, help="只给 --session-dir 时，只看这么久以内动过的子 agent")
    parser.add_argument("--detections-file", default=DEFAULT_DETECTIONS, help="hook 的检出记录，默认工作根下的 agent-hook-detections.jsonl（.claude/hooks/lib_work_root.py）")
    parser.add_argument("--detection-poll-seconds", type=float, default=15,
                        help="watch 两次读检出记录的间隔；watch 与 --processes-only 两次整查之间也按它单查常驻内存，默认 15 秒")
    parser.add_argument("--process-rss-gibibytes", type=float, default=8,
                        help="这个 Claude 实例底下不在内存包装里、峰值表里也没有它那条命令的进程，常驻内存超过这么多 GiB 就报「进程常驻内存过线」，写 0 整类不查；默认 8 GiB。"
                             "依据：本机 60 GiB 内存，本地模型服务连同会话常驻约 12 GiB（2026-09-25 free -g 的 used 列），同时可能有 3 件重活，"
                             "每件分 (60 − 12) ÷ 3 = 16 GiB，阈值取一件的一半；换机器要重算。它管涨得慢的：E142 那条无界分配的变异每秒约 5 GB、十几秒吃满整机，"
                             "快过任何复检间隔，那一类只有内存上限（scripts/hosts/run-with-memory-cap.sh）挡得住")
    parser.add_argument("--process-rss-floor-mebibytes", type=float, default=MEMORY_PEAK_FLOOR_MEBIBYTES_DEFAULT,
                        help="峰值表里有这条命令的进程，阈值是峰值乘 1.5，乘出来不到这么多 MiB 的取这么多（再大不过 --process-rss-gibibytes）；默认 512，watch.conf 写着。"
                             "依据：峰值表（~/.local/share/singlefs/memory-peaks.tsv，2026-10-22 读，30742 条）里峰值不到 8 MiB 的 2787 条、不到 512 MiB 的 19163 条，"
                             "cat、sleep 这类记成 0.00 GiB、乘 1.5 还是零点几 MiB，常驻几 MiB 就报；峰值表记的是包装 scope 的 memory.peak（含页缓存与外壳 bash），"
                             "与单个进程的常驻内存不是一个量，几十 MiB 以下比不出东西。512 是推的（没量过）：不到本机 60 GiB 的 1" + HELP_PERCENT + "、--process-rss-gibibytes 默认 8 GiB 的十六分之一，"
                             "到这里才可能是在涨的无界分配")
    parser.add_argument("--process-rss-cap-percent", type=float, default=MEMORY_WRAPPER_CAP_PERCENT_DEFAULT,
                        help="进程在内存包装（run-with-memory-cap.sh）的 scope 里时，常驻内存到它登记的上限的这么多百分比报「进程常驻内存过线」，"
                             "不再按 --process-rss-gibibytes；默认 90（推的：撞上限之前留一成给主 agent 看一眼，包装自己的上限会在 100" + HELP_PERCENT + " 处杀掉它）")
    parser.add_argument("--memory-peak-alert-cap-percent", type=float, default=MEMORY_PEAK_ALERT_CAP_PERCENT_DEFAULT,
                        help="内存包装收尾时写的「内存峰值比记账大」，实测到了它那一次上限的这么多百分比才叫醒主 agent，不到的只在报告里记一行"
                             "（峰值表包装自己已记下这一次）；上限或实测读不出的照叫醒；默认 50，写 0 一律叫醒")
    parser.add_argument("--terminate-after-unacknowledged-reports", type=int, default=0,
                        help="「进程卡住」类告警（工具调用过长、进程无输出、重型测试没带前缀）同一件事连报这么多次、主 agent 既没处置也没 --ack，"
                             "就经 proc.py stop 终止那个进程并在报告里写明（子 agent 的工具调用过长没有进程号，只报）；默认 0 不终止，watch.conf 写 3。"
                             "连报次数记在 --session-dir 那个会话的状态目录里，没给 --session-dir 不记、不终止")
    parser.add_argument("--ack", action="append", default=[],
                        help="主 agent 看过、判定只是慢接着盯的告警：子agent id:告警名，进程告警写 进程:PID，"
                             "「跑满 N 小时要主 agent 问一次」可写 子agent id:跑满 N 小时（只确认那一格）；可给多次；不带 --discover 时只在这一次看门狗里有效")
    parser.add_argument("--process-root-pid", type=int, help="进程一半从哪个 pid 往下看；默认是跑这个脚本的 Claude 实例，0 表示不看进程")
    parser.add_argument("--processes-only", action="store_true",
                        help="不看子 agent，只盯这个 Claude 实例底下起的时候就在跑的进程，它们都退出了就退 0；主 agent 自己放后台的长命令用它盯；"
                             "带 --session-dir 时同时查那个会话里交回之后后台还在跑")
    arguments = parser.parse_args()
    return run(arguments)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main())
