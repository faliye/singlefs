#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""子 agent 的定时监控：卡死、空转、永久等待当场报出来。任务（卡住、过长、内存、核数、判定行、排队、撞上限、显卡与异常结局）归 task-server 判、停，经 `task wait` 叫醒提交它的 agent；这里不读服务、不判任务、不停任务。

为什么：2026-09-17 一个实验执行员把 `echo "exit=$?"` 打到标准输出、却在日志里 `until grep -q "^exit="`，
等一行永远不会出现的字，主 agent 查进度才发现。经过：records/2026-09-17-已分配口径三方与两个实验.md 第六节。
任务进程那一半原来也在这里（进程写的文件不涨与跑得过久、没经 task-server 跑的项目代码、常驻内存过线、在用核数过少、日志里的判红行、显卡坏了，以及读内存包装、
multi-host-run.sh 写的告警记录）。用户 2026-10-31 定「任务监控以后让 task-server 做」：任务卡住只按「距这条任务的日志上一次增长满 10 分钟」由服务停掉、
报错、经 `task wait` 叫醒提交它的 agent，远超记录用时、核数过少只写进服务的 `task status`；经过：records/2026-10-30-编译绕过内存总上限.md「收尾批」。
这一份不再判那些，对应的告警、自证格、弄坏开关与配置键一并删了，不留兼容。
后来加的盯任务服务那一半（起 control check 判服务通不通、读 task status 列没 clean 的编译目录与判本会话任务正不正常）同样删了：
用户 2026-11-05 定「只删盯任务那一半」（任务系统自带提醒）；经过：records/2026-10-31-task-server巡检.md「待办现状（2026-11-04）」里「看门狗删掉盯任务服务那一半」一行。

用法：
    agent-watch.py report --agents ID[,ID…]          # 看一眼：每个子 agent 的状态、最后一条命令、告警
    agent-watch.py report --session-dir DIR           # 看这个会话里最近活动过的全部子 agent
    agent-watch.py watch  --agents ID[,ID…]          # 看门狗：每隔一段时间查一次，有告警或全部结束就退出
    agent-watch.py watch  --processes-only            # 不看子 agent，只盯这个 Claude 实例底下起的时候就在跑的进程，等它们都退出：主 agent 自己放后台的长命令用它等
    agent-watch.py cost   --agents ID[,ID…]          # 用量：调用次数、起步与平均上下文、新输入、读缓存、输出、整份重写、工具结果被什么撑大
    agent-watch.py watch … --ack <键>                # 看过、判定只是慢的告警不再为它叫醒；键三种：进程:<pid>、<agent 号>:<告警名>、类:<告警名>（写法见下面「--ack 的写法」）
    agent-watch.py --selftest                         # 造假的会话记录与假进程走一遍各种告警；AGENT_WATCH_BREAK=<项> 时必须判红

主 agent 派发之后用 Bash 的 run_in_background 起 watch：它一退出，harness 就会叫醒主 agent。
它看、它报，不停子 agent、不停进程：结束不结束、停不停由主 agent 看了报告定（用户 2026-09-17）。原来「进程卡住」类告警连报到阈值次没处置就经 proc.py stop 终止进程的做法
（用户 2026-10-13 定）随进程告警一起删了：卡住的任务由 task-server 按日志停。
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
「被拒」的检出（钩子在执行前已经拒掉：写被拒、越出写范围、不经 task-server 跑项目代码被拒、认不出起什么的写法被拒……）与判不了的脚本（项目代码闸没判全）一样，
只写进报告、不叫醒主 agent（.claude/main-agent.md「派出去之后」说的「记录不处置」那一类：拒绝信息与出路在它的工具结果里，主 agent 醒了没有可处置的）；
叫醒的只有「只记」的、照常在跑的命令。2026-10-10、2026-10-11 被拒的命令让看门狗退出，主 agent 重起了十来次。弄坏开关 AGENT_WATCH_BREAK=refusedwakes。
钩子判不了而照常放行、往检出记录补一行的（findings 以「<闸>没判、照常放行」「<闸>没判全」「<脚本名>.sh 」「判不了 」这类起头，全表是 UNJUDGED_DETECTION_PREFIXES）同样只写进报告、不叫醒：
判不了不是出了事，主 agent 醒了没有可处置的（.claude/main-agent.md「派出去之后」：钩子已经拒掉的事与判不了的检出只记进调度记录）。表里没有的新起头真碰到会叫醒，
所以自证 selftest_unjudged_prefixes 从 .claude/hooks/ 的源码里抽出全部这类起头（名字以 PREFIX 收尾的常量、findings 那一行里的 f"<起头>：…"、入口自己记检出时的脚本名）与这张表比，
钩子里加了新起头而没登进表的判红；加新起头时，钩子文件头的说明与这张表一起改。弄坏开关 AGENT_WATCH_BREAK=unjudgedwakes 去掉这一组。
范围声明钩子（.claude/hooks/run-scope-guard.sh）放行的每一条也写进检出记录，会话号在 scope_session_id、带 run_scope（命令与切开的四段）：它们不是检出、不叫醒，
属于被看会话（被看子 agent 与主 agent）的逐条列进报告的「范围声明」一节，一条一行写谁、命令与四段（种类、跑什么、范围、为什么），范围合不合适由主 agent 对着看。
弄坏开关 AGENT_WATCH_BREAK=scopeunlisted。

会话记录的位置：<家目录>/.claude/projects/<项目>/<会话 id>/subagents/agent-<id>.jsonl，同名 .meta.json 里有 agentType 与 description（不给 --session-dir 按 id 找时，
<家目录> 从统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY 的路径取，没有这个变量取本机家目录，取法只在同目录 lib_session_directory.py 里）。
子 agent 在等的后台任务，报告里列开着它输出文件的最上层进程（background_task_processes：已跑多久、连同子孙此刻占几个核），只是让主 agent 不用自己翻 ps，不按这些进程判卡住。

`--processes-only`：不给子 agent，只看进程这一半——主 agent 自己放后台的长命令没有会话记录可看，只有进程。
盯的是看门狗起的时候这个 Claude 实例底下已经在跑的进程（整棵子树，不只叶子：gate.sh 这类长命令一段一段起叶子，叶子换了，命令本身还在跑；
看门狗自己那一支与 COMMAND_MARKERS_NOT_WATCHED 不算），它们都退出了（剩下没人收尸的僵尸也算退出）就退 0。
进程本身不报告警（写着的文件动不动、CPU 涨不涨、跑多久、内存、核数、判定行都归 task-server）：一直在写文件的、写着的文件不动的、后台等着的 `task-server/task wait <号>` 都在跑时一条告警都没有。
这一路的告警只有三类：交回之后后台还在跑、交回之后副本还在、根分区余量过低，报了就退 3；--ack 与定时回报照旧。它不读 hook 的检出记录：认不出是哪个会话的。
给了 --session-dir（watch.sh --processes 从 CLAUDE_CODE_SESSION_ID 找）就同时查「交回之后后台还在跑」；没给就在报告里写明这一类没查。

「task-server/task wait <号>」：只有主 agent 起（子 agent 不往服务交任务，submit、wait 由钩子在执行前拒）。主 agent 后台等着的 task wait 归 --processes-only 那一路，进程本身不报告警；
子 agent 那一侧不另放行：会话记录里的 task wait 照普通后台任务与普通工具调用判。卡不卡由服务判：距这条任务的日志上一次增长满 10 分钟，服务停掉它、报错，经 task wait 叫醒主 agent。

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

--ack 的写法（可给多次；watch.sh 也认逗号连写，逗号后面接「进程:」「类:」或「<agent 号>:」才算下一条）：
  进程:<pid>               确认那一个进程的告警（「交回之后后台还在跑」报进程号的）
  <agent 号>:<告警名>        确认那个子 agent 的那一类告警（「跑满 N 小时」写 <agent 号>:跑满 N 小时，只确认那一格）
  类:<告警名>               确认这一类告警的全部，不分子 agent、不分进程（例：类:交回之后后台还在跑）。范围最大，用之前想清楚
类: 记进状态目录的确认同样在告警还在时留着、告警没了就作废（alert_keys、pruned_acks）。弄坏开关 AGENT_WATCH_BREAK=ackscope（不认）、ackscopepersist（记进状态目录的被当成条件已消失）。
另有 ackclassfiltered（acknowledgements() 滤掉 类: 开头的键，经 --ack 类:<告警名> 确认的告警照样叫醒）：它只给 watch.sh --selftest 自检⑬用，
那一格经 watch.sh 真起看门狗子进程核 类: 确认，红在 watch.sh 的自检⑬、不在这份自证里（这份自证的 selftest_ack_scopes 不经 acknowledgements()），所以开关表里没有它这一行。

「交回之后副本还在」：扫被看的子 agent 所在会话里全部子 agent（--processes-only 带 --session-dir 时扫那个会话），交回过（交回工具成功返回）
超过 --kept-copy-hours（默认 6，watch.conf 里写着）、交回正文里——或交回正文点名的报告文件（.md）里——写了一行「没删 <全路径>」
（上游交回闸 handback-scratch-check.sh 要子 agent 写的就是这一行），而那个路径还在的，每处报一条，交主 agent 核过没人在用之后删。
只认落在工作根、/tmp（会话临时目录根 /tmp/claude-<uid> 在内）与本进程临时目录底下的路径（仓副本与编译目录只建在那里）；交回不到阈值的、已经删掉的、没交回的不报；
--ack <agent 号>:交回之后副本还在 确认（条件类，路径都删了就作废）。只报不删。会话记录按大小与修改时刻缓存，没变就不重读。
2026-10-10 草稿根下攒了 292 个交回后没删的仓副本与编译目录（records/2026-09-16-subagent拆分提案.md 第六十八节）。

「草稿根里超阈值的编译目录与仓副本」：被看的子 agent 里有交回（或被停）的，报告里带一节：工作根整个、会话临时目录根 /tmp/claude-<uid> 下只看本会话的那一份（<会话临时目录根>/<项目>/<会话号>，Claude Code 的 scratchpad 在里面）里，
编译目录（目录里有签名对得上的 CACHEDIR.TAG）与仓副本（有 .git，或同时有 Cargo.toml 与 crates/）最后修改距今超过 --scratch-idle-hours（默认 6，watch.conf 里写着，依据是推的）的，
按最后修改时刻、最久没动的在前，每处一行写种类、路径与最后修改距今多久。「最后修改」取副本自己与往里三层条目的修改时刻最大的一个（不全树走一遍）。
只进报告、不叫醒、不删（删由主 agent 核过没人在用之后做）；没有子 agent 交回时不查；写 0 不查。放这里不放交回钩子：交回钩子的话只送到交回的子 agent，主 agent 看不到。
弄坏开关 AGENT_WATCH_BREAK=scratchhog、scratchhogage、scratchhogtag、scratchhogsessionroot。

「根分区余量过低」：每次整查（--agents 与 --processes-only 两条路都查）读工作根（.claude/hooks/lib_work_root.py 的 work_root；还没建的往上取第一个在的上层）所在文件系统的可用量
（os.statvfs 的 f_bavail × f_frsize，与 df 的 Avail 同一个数），低于 --root-free-gib-alert（默认 128 GiB，watch.conf 里写着，依据是推的）就报，叫醒主 agent；不低的在报告里记一行可用多少。
告警写可用多少、阈值多少与出路：先查各会话的副本（工作根下各草稿目录），再查崩溃放量家目录（SINGLEFS_CRASH_AMPLIFICATION_HOME，没设是工作根下的 crash-amplification，
与 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_home_directory 同一个取法）
runs/ 下的各运行目录，以 du 为准，大的先看；只删没有进程在用的，别人在用的不删。
条件类告警：--ack 类:根分区余量过低 确认之后只进报告，可用量回到阈值以上这条确认就作废；只报不删、不终止。写 0 不查。
经 watch.sh 起只认类:这一种写法（它不认「本机:」这个头）；直接调 agent-watch.py 时 --ack 本机:根分区余量过低 同样认。
可用量的读法换得掉：root_free_report 收一个读可用字节数的函数（自证传假的）；环境变量 AGENT_WATCH_ROOT_FREE_BYTES 设了就当可用字节数、不读 statvfs（只给 --selftest 起的看门狗子进程用）。
2026-10-22 本机根分区写满、只剩 113M，是写文件撞上 No space left on device 才发现的，看门狗只报草稿根里闲置的编译目录、不看余量
（records/2026-10-13-里程碑四预开发.md 第 951 行）。弄坏开关 AGENT_WATCH_BREAK=rootfree。

「重启前旧进程留下的记录判已失效」：每次整查读 ~/.claude/sessions/<pid>.json（Claude Code 给每个进程登记的会话号、启动时刻与 procStart，pid 还在而且 /proc/<pid>/stat 第 22 列对得上才算活着），
没交回的子 agent 记录有一种算已失效：一、它所在的会话号不是本会话（--session-dir 的目录名，没给取 CLAUDE_CODE_SESSION_ID）、登记里也没有活着的 Claude 进程；
二、会话号还活着（重启后接着用同一个会话号），而它最后一条记录早于那个会话现在活着的最早一个进程的启动时刻。已失效的在报告里标「状态=已失效」，
不按「无动静」「工具调用过长」这类报，不算要盯的，也不拦着全部交回时退出；--discover 不再把它记成没结束的。登记读不出、或里面没有一份能解的，判不了，一律不当失效。
弄坏开关 AGENT_WATCH_BREAK=stalesession、stalesessionrestart。

弄坏开关 AGENT_WATCH_BREAK=<项>（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
（项名是那条失败落在哪个自证函数里：selftest_in、selftest_session_watch_in、selftest_scratch_hogs、selftest_stale_sessions、selftest_leftover_background、selftest_kept_copies、
selftest_root_free、selftest_ack_concurrency、selftest_unjudged_prefixes、selftest_ack_scopes、selftest_scratch_mentions，临时目录那三格写格名；括号里是那几条「✗ 自检：」的开头；红了自证退 1）：
  AGENT_WATCH_BREAK=ack  split_acknowledged() 不认 --ack，确认过的告警照样叫醒
      selftest_in（「✗ 自检：「工具调用过长」被 --ack 确认之后看门狗应当到点退出码 4 并写「已确认接着盯」」、「✗ 自检：派发 125 分钟、--ack 了「跑满 2 小时」那一格时看门狗不该叫醒」）、
      selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」）、
      selftest_session_watch_in（「✗ 自检：--discover 应当自己找出没结束的 discoveredlong、确认过的不叫醒、到点退 4」、「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、
      「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」）、selftest_root_free（「✗ 自检：根分区余量过低被 --ack 本机:根分区余量过低 确认之后」）、
      selftest_kept_copies（「✗ 自检：看门狗的报告应当带「交回之后副本还在」告警与一行汇总」）、
      selftest_ack_scopes（「✗ 自检：--ack 类:根分区余量过低 应当只确认告警」、「✗ 自检：--ack 类:交回之后后台还在跑 应当只确认告警」、「✗ 自检：--ack 进程:333 应当只确认告警」）
  AGENT_WATCH_BREAK=acksneverpruned  pruned_acks() 不作废条件已经不在的确认，全部留着
      selftest_session_watch_in（「✗ 自检：条件类告警的确认」）
      selftest_ack_scopes（「✗ 自检：记进状态目录的 类: 确认应当在告警还在时留着」）
  AGENT_WATCH_BREAK=acksunlocked  update_acks() 读改写 watch-acks.tsv 不拿 watch-acks.lock 的 flock（改前的样子）
      selftest_ack_concurrency（「✗ 自检：watch-acks.tsv 两只同时写」）
  AGENT_WATCH_BREAK=acksnotpersisted  read_acks() 不读状态目录里记下的确认
      selftest_session_watch_in（「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」、「✗ 自检：已有一个在盯时给的 --ack 应当记进状态目录」、
      「✗ 自检：条件类告警的确认」）、
      selftest_ack_concurrency（「✗ 自检：watch-acks.tsv 两只同时写」）
  AGENT_WATCH_BREAK=amplificationhome  crash_amplification_home() 没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时照改前取家目录下那一份
      selftest_root_free（「✗ 自检：没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时崩溃放量家目录应当是工作根下的 crash-amplification」）
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
  AGENT_WATCH_BREAK=cost  自检里把用量样本的 full_cache_rewrite_tokens 读成 0
      selftest_in（「✗ 自检：用量样本的 full_cache_rewrite_tokens 应当是」）
  AGENT_WATCH_BREAK=deadline  run() 不按回报周期截短复检间隔
      selftest_in（「✗ 自检：复检间隔比回报周期长时看门狗没有按回报周期退出」）
  AGENT_WATCH_BREAK=detections  read_detections() 一条检出都不读（每条都当别的会话的跳过）
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」」、「✗ 自检：检出记录属于「session」「主 agent None」」）、selftest_session_watch_in（「✗ 自检：hook 检出」）
      selftest_unjudged_prefixes（「✗ 自检：UNJUDGED_DETECTION_PREFIXES 里」）
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
  AGENT_WATCH_BREAK=heredoc  command_without_heredoc_bodies() 不去 heredoc 正文，正文里写的命令也当成在执行
      selftest_in（「✗ 自检：子 agent bannedheredoc」）
  AGENT_WATCH_BREAK=holders  background_task_processes() 一个都不列
      selftest_in（「✗ 自检：结束本轮等自己的后台任务时应当列出开着那个输出文件的进程」）
  AGENT_WATCH_BREAK=idle  agent_alerts() 不报「无动静」
      selftest_in（「✗ 自检：子 agent idle」）
  AGENT_WATCH_BREAK=infoalertwakes  split_informational() 把只记不叫醒的条件告警照旧叫醒
      selftest_in（「✗ 自检：只有「等待循环」这类只记不叫醒的告警时」）
      selftest_ack_scopes（「✗ 自检：「进度文件不涨」应当归只记不叫醒的那一类」）
  AGENT_WATCH_BREAK=infowakes  read_detections() 只记不叫醒的检出也叫醒（「写被拒」另由执行前已拒那一条挡着，红的是「项目代码闸没判全」那一条）
      selftest_session_watch_in（「✗ 自检：hook 检出：写被拒的与项目代码闸没判全的应当只记进报告不叫醒」）
      selftest_unjudged_prefixes（「✗ 自检：UNJUDGED_DETECTION_PREFIXES 里」）
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
      「✗ 自检：被看的子 agent 全部交回、它丢下的后台进程还在时」、「✗ 自检：被看的子 agent 刚交回、后台进程还在宽限里时看门狗应当等宽限过了再查、报」、「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」、「✗ 自检：--processes-only 带 --session-dir 时也应当报」
  AGENT_WATCH_BREAK=leftoverack  split_acknowledged() 不认「交回之后后台还在跑」的 --ack 进程:PID
      selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」的进程都用 --ack 进程:<pid> 确认过时不该叫醒」）、selftest_ack_scopes（「✗ 自检：--ack 进程:333 应当只确认告警」）
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
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」、检出「不经 task-server 跑项目代码被拒：…」」、「✗ 自检：检出记录属于「session」「主 agent None」、检出「认不出起什么的写法被拒：…」」）、
      selftest_session_watch_in（「✗ 自检：hook 检出：执行前已被 hook 拒了的（b1 …）应当只记进报告、不叫醒」）
  AGENT_WATCH_BREAK=ownrefusal  read_detections() 主 agent 自己的命令被 hook 拒的不单记
      selftest_in（「✗ 自检：检出记录属于「session」「主 agent None」」）
  AGENT_WATCH_BREAK=processexit  run_processes_only() 盯的进程都退出了也不退
      selftest_in（「✗ 自检：--processes-only 盯着的进程退出了」、「✗ 自检：--processes-only 起的时候一个要盯的进程都没有」）
  AGENT_WATCH_BREAK=progressanypath  progress_file_of() 照改前的取法：派发提示里点名的工作根或会话临时目录根下的路径都算候选、取改得最晚的那一份，不只认行首「草稿目录：」那一行
      selftest_scratch_mentions（「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告…」、「✗ 自检：派发提示没有行首「草稿目录：」一行、只点名别的子 agent 的报告…」、
      「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句应当写取不到 progress.md…」、「✗ 自检：派发提示点名会话临时目录根下的草稿目录不认」）
  AGENT_WATCH_BREAK=progressnote  progress_note() 告警里不附 progress.md 的末几行
      selftest_session_watch_in（「✗ 自检：整点询问应当附上 progress.md 的末几行」）
      「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句」
  AGENT_WATCH_BREAK=progressstale  agent_alerts() 不报「进度文件不涨」
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」）
  AGENT_WATCH_BREAK=refusedwakes  read_detections() 钩子执行前已拒掉的检出（outcome 被拒）也叫醒，不单记进报告
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」、检出「不经 task-server 跑项目代码被拒：…」」、「✗ 自检：检出记录属于「session」「主 agent None」、检出「认不出起什么的写法被拒：…」」）、
      selftest_session_watch_in（「✗ 自检：hook 检出：执行前已被 hook 拒了的（b1 …）应当只记进报告、不叫醒」）
  AGENT_WATCH_BREAK=repeat  agent_alerts() 不报同一命令反复且输出不变
      selftest_in（「✗ 自检：子 agent repeat」）
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
      「✗ 自检：有子 agent 交回时看门狗的报告应当列出超阈值的 …」、「✗ 自检：会话 scratchpad 里超阈值的编译目录照报」）
  AGENT_WATCH_BREAK=scratchhogage  scratch_hog_report() 不看最后修改的时刻，不到阈值的编译目录与仓副本也报
      selftest_scratch_hogs（「✗ 自检：草稿根里超阈值的编译目录与仓副本：应当只报 …」：fresh-build、fresh-copy 也报了、「✗ 自检：有子 agent 交回时看门狗的报告应当列出超阈值的 …、不列 …」）
      「✗ 自检：草稿根那一节应当按最后修改时刻、最久没动的在前」
  AGENT_WATCH_BREAK=scratchhogtag  scratch_hog_kind() 不认 CACHEDIR.TAG，编译目录一概认不出
      selftest_scratch_hogs（「✗ 自检：草稿根里超阈值的编译目录与仓副本：应当只报 …」：old-build/target 没报、「✗ 自检：有子 agent 交回时看门狗的报告应当列出超阈值的 …」）
      「✗ 自检：草稿根那一节应当按最后修改时刻、最久没动的在前」、「✗ 自检：会话 scratchpad 里超阈值的编译目录照报」
  AGENT_WATCH_BREAK=scratchhogsessionroot  scratch_hog_roots() 不看会话临时目录根下本会话的那一份（只看工作根）
      selftest_scratch_hogs（「✗ 自检：会话 scratchpad 所在的本会话目录在草稿根里」）
  AGENT_WATCH_BREAK=settle  AgentTranscript._read() 最后一条是纯文字、stop_reason 为空的一概不判结束本轮（主会话记录里有这一轮的任务通知、主会话记录不在而过了 TEXT_SETTLE_SECONDS 都不认）
      selftest_in（「✗ 自检：子 agent textsettled」、「✗ 自检：子 agent inquiryfailed」、「✗ 自检：子 agent textsettled 的状态应当是「本轮结束、没交回」」、
      「✗ 自检：子 agent textthenended」、「✗ 自检：子 agent textthenended 的状态应当是「本轮结束、没交回」」、「✗ 自检：子 agent noparenttext 的状态应当是「本轮结束、没交回」」）
  AGENT_WATCH_BREAK=settlenotification  AgentTranscript.text_turn_ended() 不看主会话记录里的任务通知，纯文字记录过了 TEXT_SETTLE_SECONDS 就判结束本轮（改前的判法）
      selftest_in（「✗ 自检：子 agent textthenwrite」、「✗ 自检：子 agent textthenwrite 的状态应当是「思考中」」）
  AGENT_WATCH_BREAK=stalepending  AgentTranscript._read() 不剔掉只落了发起、等不到结果的工具调用
      selftest_in（「✗ 自检：子 agent stalependingthenresult：应当告警 无，实际 ['工具调用过长']」）
  AGENT_WATCH_BREAK=stalesession  stale_session_reason() 重启前旧进程留下的记录一概不判已失效
      selftest_stale_sessions（「✗ 自检：重启前旧进程留下的记录（已不在的会话 deadlong …」、「✗ 自检：子 agent deadlong 应当在报告里标成「状态=已失效」」、「✗ 自检：被看的只有重启前旧进程留下的记录时应当没有告警」）
  AGENT_WATCH_BREAK=stalesessionrestart  stale_session_reason() 不判第二种（会话号还活着、最后一条记录早于进程启动）
      selftest_stale_sessions（「✗ 自检：重启前旧进程留下的记录（… beforestart …」、「✗ 自检：子 agent beforestart 应当在报告里标成「状态=已失效」」、「✗ 自检：被看的只有重启前旧进程留下的记录时应当没有告警」）
  AGENT_WATCH_BREAK=taskstop  AgentTranscript.__init__() 不认主会话记录里 TaskStop 的成功结果
      selftest_in（「✗ 自检：子 agent stoppedidle」、「✗ 自检：子 agent stoppedtwice」、「✗ 自检：子 agent stoppedidle 应当算交回或被停」、「✗ 自检：子 agent stoppedtwice 应当算交回或被停」、
      「✗ 自检：被看的子 agent 都交回或被停时看门狗应当退出码 0」）、selftest_leftover_background（「✗ 自检：「交回之后后台还在跑」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个」）
  AGENT_WATCH_BREAK=timeout  agent_alerts() 不报「工具调用过长」
      selftest_in（「✗ 自检：子 agent longtool」、「✗ 自检：子 agent inquirysentafter」、「✗ 自检：有告警（工具调用过长）时看门狗应当退出码 3」、「✗ 自检：「工具调用过长」被 --ack 确认之后看门狗应当到点退出码 4 并写「已确认接着盯」」）、
      selftest_session_watch_in（「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、「✗ 自检：条件类告警的确认」）、
      selftest_stale_sessions（「✗ 自检：重启前旧进程留下的记录」、「✗ 自检：登记目录读不了时判不了会话在不在」）
  AGENT_WATCH_BREAK=toolrevoke  AgentTranscript._read() 交回之后又调了工具就撤销交回
      selftest_in（「✗ 自检：子 agent toolafterhandback」、「✗ 自检：子 agent toolafterhandback 应当算交回或被停」）
  AGENT_WATCH_BREAK=zombie  still_running() 把僵尸当还在跑
      selftest_in（「✗ 自检：--processes-only 盯着的进程退出了」）
  AGENT_WATCH_BREAK=ackscope  ack_scope_keys() 一个键都不认：--ack 类:<告警名> 不起作用（改前的样子：只能按进程号、按 agent 号逐个确认）
      selftest_ack_scopes（「✗ 自检：--ack 类:根分区余量过低 应当只确认告警」、「✗ 自检：--ack 类:交回之后后台还在跑 应当只确认告警」、「✗ 自检：记进状态目录的 类: 确认应当在告警还在时留着」）
  AGENT_WATCH_BREAK=ackscopepersist  alert_keys() 不带 类: 这种键：记进状态目录的类确认在下一次整查被当成条件已消失而作废
      selftest_ack_scopes（「✗ 自检：记进状态目录的 类: 确认应当在告警还在时留着」）
  AGENT_WATCH_BREAK=unjudgedwakes  INFO_DETECTION_PREFIXES 不含 UNJUDGED_DETECTION_PREFIXES：钩子判不了而照常放行记下的检出真碰到都叫醒（改前的样子）
      selftest_unjudged_prefixes（「✗ 自检：钩子源码里记检出用的」、「✗ 自检：UNJUDGED_DETECTION_PREFIXES 里」）
工作根（草稿目录、检出记录与状态目录的默认位置）与会话临时目录根只在 .claude/hooks/lib_work_root.py 里定义，它的两个弄坏开关：
  SINGLEFS_WORK_ROOT_BREAK=new-root-unknown  工作根退回会话临时目录根：派发提示里工作根下的草稿目录认不出
      selftest_scratch_mentions（「✗ 自检：派发提示点名工作根下的草稿目录」两格、「✗ 自检：派发提示点名会话临时目录根下的草稿目录不认」（工作根就是会话临时目录根，那边的目录认得出了）、「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告…」）、
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」、「✗ 自检：整点询问应当附上 progress.md 的末几行」）、
  SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted  草稿根另认会话临时目录根（撤掉之前的放行）：派发提示里会话临时目录根下的草稿目录认得出
      selftest_scratch_mentions（「✗ 自检：派发提示点名会话临时目录根下的草稿目录不认」）
「交回」送没送达只在同目录 subagent_transcript.py 里认，它的开关（开关表在它的文件头）：
  SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered  被交回闸拒的交回也算送达
      selftest_in（「✗ 自检：子 agent refusedonly 应当不算交回或被停」）
会话记录目录（<家目录>/.claude/projects）只在同目录 lib_session_directory.py 里取（统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY 定家目录，不经 $HOME），它的开关（开关表在它的文件头）：
  SESSION_DIRECTORY_BREAK=home-from-environment  设了 TASK_SERVER_SOURCE_REPOSITORY 也照改前取 HOME
      selftest_projects_root（「✗ 自检：按 id 找会话记录取来源仓根的会话目录：」）
"""
import argparse
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
import lib_work_root as work_roots  # noqa: E402  工作根与会话临时目录根的唯一定义
import lib_prompt_scratch_line as prompt_scratch_line  # noqa: E402  派发提示里「本提示的草稿目录」那一行的唯一认法，派发闸、交回闸、完工清理闸调的是同一份
preflight_sys.path.insert(0, preflight_os.path.dirname(preflight_os.path.realpath(__file__)))
import subagent_transcript as transcripts  # noqa: E402  「交回」（送达）的唯一定义，续做闸与 agent-cost.py 取的是同一份
import lib_session_registry as session_registry  # noqa: E402  Claude Code 会话登记（~/.claude/sessions/<pid>.json）的唯一读法与判活
import lib_session_directory as session_directories  # noqa: E402  会话记录目录的唯一取法（来源仓根 TASK_SERVER_SOURCE_REPOSITORY，不经 $HOME；agent-cost.py、agent-handover.py、watch.sh、move-directory-in-use-check.py 同取它）

def projects_root(environment=None):
    """各项目的会话目录所在的那一层（<家目录>/.claude/projects）：统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY（两台一致）定家目录，没有这个变量（服务外手跑）取本机家目录，
    取法只在 lib_session_directory.py 里（不经 $HOME 拼）；AGENT_WATCH_PROJECTS_ROOT 设了优先，只给 --selftest 造假的项目目录用。取不到抛 session_directories.SessionDirectoryError。"""
    environment = os.environ if environment is None else environment
    return environment.get("AGENT_WATCH_PROJECTS_ROOT") or session_directories.projects_root(environment)


DEFAULT_DETECTIONS = work_roots.default_detections_path()
WAIT_LOOP_PATTERN = re.compile(r"\b(until|while)\b[\s\S]*\bsleep\b")
BANNED_COMMAND_PATTERN = re.compile(r"\bpgrep\s+-f\b|\bpkill\s+-f\b|\bkillall\b")
BROKEN_DETECTION = os.environ.get("AGENT_WATCH_BREAK", "")
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
CPU_SAMPLE_SECONDS = 2               # 子 agent 在等的后台任务，隔这么久取两次 CPU 时间，报此刻占几个核（background_task_processes）
BACKGROUND_STARTED = re.compile(r"^Command running in background with ID: (\w+)")
# 前台命令跑满超时被 harness 挪进后台，工具结果是另一种写法，之后同样会来完成通知。不认它，在等这种任务的子 agent 会被报「结束本轮却不会醒」
# （2026-09-23 门禁审计会话的 ac26abd494cda0968：doc-lint 超时转后台，看门狗报「起过的 0 个后台任务」；本机这个项目的子 agent 会话记录里 79 份有这种写法）。
BACKGROUND_MOVED = re.compile(r"^Command did not complete within its \d+s timeout and was moved to the background \(ID: (\w+)\)")
NEW_INSTRUCTION_ORIGINS = ("coordinator", "peer")   # 续做消息与别的会话发来的消息；task-notification 是自己后台任务的完成通知，不算
BACKGROUND_OUTPUT_PATH = re.compile(r"Output is being written to: (\S+?\.output)")
TASK_NOTIFICATION_ID = re.compile(r"<task-id>(\w+)</task-id>")
# --processes-only 不盯的命令：claude 本体、IDE、node（常驻的 MCP 一类）、别的看门狗
COMMAND_MARKERS_NOT_WATCHED = ("native-binary/claude", "vscode-server", "/node ", "agent-watch.py")
EXITED_PROCESS_STATES = ("Z", "X")   # /proc/<pid>/stat 的状态：僵尸（退出了、等父进程收尸）与已死，都算退出了
LEFTOVER_BACKGROUND_ALERT = "交回之后后台还在跑"
KEPT_COPY_ALERT = "交回之后副本还在"
# 交回报告里「没删 <全路径>」那一行的写法（上游交回闸 handback-scratch-check.sh 要的就是这一行）：路径取到第一个不是路径字符的字为止（后面紧跟汉字也认）
KEPT_COPY_MENTION = re.compile(r"没删(?:掉)?[：:\s]*[`「“\"']?((?:~/|/)[A-Za-z0-9_./~+@%=,-]+)")
# 交回报告里点名的报告文件（.md，绝对路径或 ~/ 起头）：里面写的「没删 <全路径>」也算
KEPT_COPY_REPORT_MENTION = re.compile(r"((?:~/|/)[A-Za-z0-9_./~+@%=,-]+\.md)")
KEPT_COPY_REPORT_MAX_BYTES = 4 << 20
# 主会话记录里任务通知的这几种状态说明子 agent 已经不在跑了：failed 失败或报错、killed 被停、stopped 上一个 Claude 进程退出时它还没跑完
ENDED_NOTIFICATION_STATUSES = ("failed", "killed", "stopped")
TASK_NOTIFICATION_BLOCK = re.compile(r"<task-notification>(.*?)</task-notification>", re.S)
TASK_NOTIFICATION_STATUS = re.compile(r"<status>([a-z_]+)</status>")
# 重启前旧进程留下的子 agent 记录判「已失效」：Claude Code 给每个进程在这个目录登记一份 <pid>.json（会话号 sessionId、启动时刻 startedAt 毫秒、
# procStart 即 /proc/<pid>/stat 第 22 列）。环境变量只给 --selftest 造假的登记目录用
SESSION_REGISTRY_ENV = "AGENT_WATCH_SESSIONS_DIR"
# 草稿根里超阈值的编译目录与仓副本：编译目录认 CACHEDIR.TAG（签名见 https://bford.info/cachedir/，cargo 的 target 都带），仓副本认 .git 或「Cargo.toml 加 crates/」。
# 环境变量（冒号分隔的根）只给 --selftest 造假的草稿根用
SCRATCH_ROOTS_ENV = "AGENT_WATCH_SCRATCH_ROOTS"
CACHEDIR_TAG_SIGNATURE = b"Signature: 8a477f597d28d172789f06886806bc55"
SCRATCH_SCAN_MAX_DEPTH = 4            # 从草稿根往下找这么深（根 → 任务目录 → 副本 → target）
SCRATCH_SCAN_MAX_DIRECTORIES = 20000  # 一次最多看这么多个目录，到顶就在报告里写明
SCRATCH_MTIME_MAX_DEPTH = 3           # 求「最后修改时刻」往副本里看这么深（target/release/deps 在内）
SCRATCH_MTIME_MAX_ENTRIES = 30000     # 每处副本最多 stat 这么多个条目
ROOT_FREE_ALERT = "根分区余量过低"
ROOT_FREE_SUBJECT = "本机"            # 这一类告警不属于哪个子 agent 或进程，主体写本机；推荐的确认写法是 --ack 类:根分区余量过低（watch.sh 不认「本机:」这个头）
CRASH_AMPLIFICATION_HOME_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_HOME"   # 崩溃放量家目录，设了用它（crash_amplification_home）
CRASH_AMPLIFICATION_ENTRY = "crash-amplification"   # 没设时工作根下那一层的目录名，与 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_home_directory 相同
ROOT_FREE_BYTES_VARIABLE = "AGENT_WATCH_ROOT_FREE_BYTES"   # 环境变量只给 --selftest 起的看门狗子进程用：设了就当根分区的可用字节数，不读 statvfs
ROOT_FREE_GIB_ALERT_DEFAULT = 128     # --root-free-gib-alert 的默认值（依据见那个选项，推的）
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))   # 从 scripts/agents/ 往上两级是仓根
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
HOOKS_DIRECTORY = os.path.join(REPOSITORY_ROOT, ".claude", "hooks")
# 自证的临时目录由同一处的共用库建（通过、判红、抛异常三条路上都删），那三格也由它判
SELFTEST_SCRATCH_LIBRARY_PATH = os.path.join(HOOKS_DIRECTORY, "lib_selftest_scratch.py")
SELFTEST_ROLE_VARIABLE = "AGENT_WATCH_SELFTEST_ROLE"   # 自证拉起自己的子进程走判红、抛异常那两条路时设的角色
SELFTEST_READY_VARIABLE = "AGENT_WATCH_SELFTEST_READY_FILE"   # 自证拉起的看门狗第一次进两次整查之间的等待前建这个文件（mark_selftest_ready），自证按它等
SELFTEST_READY_TIMEOUT_SECONDS = 120   # 自证按条件等子进程起来的上限；到点不报错，照原判据接着判
BYTES_PER_GIBIBYTE = 1024 ** 3
# 「中途停下没交回」（文件头同名一段）：结束本轮、没交回、一个后台任务都没起过的
STOPPED_MIDWAY_ALERT = "中途停下没交回"
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
STATE_ROOT_VARIABLE = "AGENT_WATCH_STATE_ROOT"   # 环境变量只给 --selftest 换成临时的状态目录
STATE_ROOT = os.environ.get(STATE_ROOT_VARIABLE) or work_roots.state_directory("watch-state")
SINGLE_INSTANCE_EXIT = 5
# 钩子已经拒掉的写与判不了的脚本：只记进报告，不叫醒主 agent（2026-09-28 量到 8 天里看门狗转报的检出大半是这两类，主 agent 醒了也没事可做）
# 钩子判不了而照常放行、往检出记录补一行（「放行但必记检出」，.claude/agent-common.md「不做」一节）的 findings 起头：同样只记不叫醒——
# 判不了的事主 agent 醒了没有可处置的（.claude/main-agent.md「派出去之后」：钩子已经拒掉的事与判不了的检出只记进调度记录）。
# 钩子里加了新的这类起头而没登进这一张表，自证 selftest_unjudged_prefixes 判红（它从 .claude/hooks/ 的源码里抽出全部这类起头与这张表比）。
# 弄坏开关 unjudgedwakes 去掉这一组（改前的样子：这些检出真碰到都叫醒）
UNJUDGED_DETECTION_PREFIXES = (
    "开会话提示没判、照常放行", "资源提醒没判、照常放行", "弹窗闸没判、照常放行", "定时闸没判、照常放行",
    "完工清理闸没判全、照常放行", "完工清理闸没判、照常放行", "交回正文闸没判全、照常放行", "派发闸没判全、照常放行",
    "快照清单读不了、照常放行", "续做闸没判全", "写闸没判全", "范围声明闸判不了",
    "循环提醒的停止钩子判不了、照常放行", "提交后的索引检查判不了、照常放行",
    "bash-command-detector.sh ", "run-scope-guard.sh ", "two-logs-guard.sh ", "判不了 ",
    "project-code-guard.sh 没判成", "项目代码闸没判全")
INFO_DETECTION_PREFIXES = ("写被拒", "项目代码闸没判全") + (() if BROKEN_DETECTION == "unjudgedwakes" else UNJUDGED_DETECTION_PREFIXES)
# 只记进报告、不叫醒主 agent 的三类条件告警（2026-09-28 量到 8 天里看门狗按告警退出 533 次，这三类醒来多半只是重起一只）
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
        2026-09-19 c381-r1 攻方腿失败中断，失败之前起的探针任务就停在这个样子，看门狗一直把它当「还在跑」。
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
        """结束本轮、没交回，手里还有没收到完成通知的后台任务：它在等，会话记录里本来就一个字都不写，醒不醒由那些任务决定，
        不按会话记录的「无动静」判——2026-09-19 c381-r1 攻方腿等两个吃满 CPU 的探针，看门狗每次重起都当场报「无动静」退出，盯不住。"""
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
    """「没删 <路径>」里只认落在这几处底下的路径（工作根、/tmp（会话临时目录根在内）与本进程的临时目录）：仓副本与编译目录只建在这里，别处的不报、不叫人删。"""
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


process_start_ticks = session_registry.process_start_ticks   # /proc/<pid>/stat 第 22 列；读法只在 lib_session_registry.py 里


def live_sessions():
    """{会话号: 此刻活着的 Claude 进程里最早的启动时刻（秒；登记里没写是 None）}。
    活着 = 登记里的 pid 还在、而且 /proc/<pid>/stat 第 22 列与登记的 procStart 一致（pid 被别的进程占了不算）。
    登记目录读不了、或里面一份能解的登记都没有，返回 None：判不了，不把所有会话当成已失效。"""
    directory = os.environ.get(SESSION_REGISTRY_ENV) or session_registry.registry_directory()
    entries = session_registry.session_registry_entries(directory)
    if entries is None:
        return None
    live = {}
    for entry in entries:
        if not entry.is_alive:
            continue
        if entry.session_id not in live:
            live[entry.session_id] = entry.started_seconds
        elif entry.started_seconds is None or live[entry.session_id] is None:
            live[entry.session_id] = None
        else:
            live[entry.session_id] = min(live[entry.session_id], entry.started_seconds)
    return live


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
    """要查的草稿根：工作根整个；会话临时目录根 /tmp/claude-<uid> 只看本会话的那一份（<会话临时目录根>/<项目>/<会话号>，Claude Code 的 scratchpad 在里面）。
    环境变量 AGENT_WATCH_SCRATCH_ROOTS 给了就只用它（自证用）。弄坏开关 scratchhogsessionroot 下只看工作根。"""
    override = os.environ.get(SCRATCH_ROOTS_ENV)
    if override is not None:
        return [root for root in override.split(os.pathsep) if root]
    roots = [work_roots.work_root()]
    if BROKEN_DETECTION != "scratchhogsessionroot":
        for session_id in sorted(session_ids):
            roots.extend(sorted(glob.glob(os.path.join(work_roots.claude_session_temporary_root(), "*", session_id))))
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


def crash_amplification_home(environment=None):
    """崩溃放量家目录（文件头「根分区余量过低」）：SINGLEFS_CRASH_AMPLIFICATION_HOME 设了用它，没设是工作根下的 crash-amplification（共享根里、两台同一份）。
    environment 不给取 os.environ。弄坏开关 amplificationhome：没设时照改前取家目录下那一份。"""
    environment = os.environ if environment is None else environment
    configured = environment.get(CRASH_AMPLIFICATION_HOME_VARIABLE, "")
    if configured:
        return configured
    if BROKEN_DETECTION == "amplificationhome":
        return os.path.join(work_roots.home_directory(), ".local", "share", "singlefs", CRASH_AMPLIFICATION_ENTRY)
    return os.path.join(work_roots.work_root(), CRASH_AMPLIFICATION_ENTRY)


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
    amplification_runs = os.path.join(crash_amplification_home(), "runs")
    explanation = f"工作根 {path} 所在的文件系统只剩 {free:.1f} GiB 可用，低于阈值 {threshold:g} GiB（--root-free-gib-alert），再写下去会撞 No space left on device"
    next_step = (f"先查各会话的副本（du -sh {work_roots.work_root()}/*，大的先看），再查 {amplification_runs}/ 下的各运行目录（du -sh {amplification_runs}/*）；以 du 为准，"
                 "只删没有进程在用的（ps 里没有进程的工作目录、命令行或 CARGO_TARGET_DIR 指到它），别人在用的不删；删完再看一次 df，提醒用户跑 sudo fstrim -v /；"
                 f"一时删不了、确认接着跑的用 --ack {ACK_CLASS_PREFIX}{ROOT_FREE_ALERT}")
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


def find_transcript(agent_id, session_dir=None, environment=None):
    if session_dir:
        candidate = os.path.join(session_dir, "subagents", f"agent-{agent_id}.jsonl")
        return candidate if os.path.isfile(candidate) else None
    matches = glob.glob(os.path.join(glob.escape(projects_root(environment)), "*", "*", "subagents", f"agent-{agent_id}.jsonl"))
    return matches[0] if matches else None


def elapsed_mark_label(mark_minutes):
    """一格写成人读的样子：整小时写「N 小时」；watch.conf 把 ask-every-minutes 改成不是整小时的，写「M 分钟」。"""
    if mark_minutes % 60 == 0:
        return f"{int(mark_minutes // 60)} 小时"
    return f"{mark_minutes:g} 分钟"


def own_scratch_directories(transcript):
    """派发提示行首「草稿目录：」那几行点名的目录（认法只在 .claude/hooks/lib_prompt_scratch_line.py 的 scratch_directories：~/… 这类写法先展开，
    只留工作根底下的），按出现次序；没有那一行返回空表。"""
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
    弄坏开关 progressanypath 下照改前的取法：提示里点名的工作根或会话临时目录根下的路径都算候选（是 progress.md 就是它，是目录取目录下的，是文件取同目录的）。"""
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
                           "主 agent 判断：这条命令起的进程还在不在动（ps 看它的子孙进程与它写的文件）、预期还要多久；动就接着盯，不动再决定怎么处理"))
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
                       "或者它等的后台任务在一次中断（失败、进程被杀）里没了"))
    elif (transcript.last_timestamp is not None and BROKEN_DETECTION != "idle"
          and not transcript.waiting_on_its_own_background_tasks()
          and (now - transcript.last_timestamp).total_seconds() >= thresholds.idle_minutes * 60):
        alerts.append(("无动静", f"最后一条记录在 {format_duration((now - transcript.last_timestamp).total_seconds())} 以前，没有工具在跑",
                       "主 agent 判断：模型调用是不是卡住或失败；看会话记录最后几条，再决定等、发消息续做、还是停掉重派"))
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
                           "主 agent 看它等的后台任务还在不在动（本报告列的开着输出文件的进程）：在动就 --ack <agent 号>:进度文件不涨 接着盯；不动再决定发消息还是停掉"))
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
    2026-09-19：批次脚本一轮轮起一分钟的探针，按进程跑了多久列的那一节一个都不够格，那一节根本不出现，主 agent 只好自己去翻 ps；
    所以这里按开着输出文件认，不按进程跑了多久认。"""
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
    """--processes-only：起的时候在跑的那些都退出了退 0，有告警（交回之后后台还在跑、副本还在、根分区余量）退 3，到定时回报退 4；
    report 只看一眼，有告警退 3、没有退 0。进程本身的卡住、过长、无输出、内存、核数、判定行都归 task-server 判，这里只盯它们还在不在。"""
    watched = processes_to_watch(process_root_pid, excluded_pids)
    report_deadline = time.time() + arguments.max_minutes * 60
    while True:
        running = still_running(watched)
        topmost_running_pids = sorted(pid for pid, info in running.items() if info["parent"] not in running)
        lines = [f"只盯进程：进程 {process_root_pid} 底下看门狗起的时候就在跑的 {len(watched)} 个进程，还没退出的 {len(running)} 个"
                 + ("，最上层的：" if topmost_running_pids else "")]
        lines += [f"  进程 {pid}：{command_for_display(running[pid]['command'])[:140]}" for pid in topmost_running_pids]
        # 根分区余量（文件头「根分区余量过低」）：告警的主体是本机，不是哪个进程
        root_lines, root_found = root_free_report(arguments)
        lines.extend(root_lines)
        found_alerts = [(ROOT_FREE_SUBJECT, name, explanation, next_step) for name, explanation, next_step in root_found]
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
        alerts, informational_lines = split_informational(found_alerts + leftover_alerts + kept_alerts)
        lines.extend(informational_lines)
        alerts, acknowledged = split_acknowledged(alerts, set(arguments.ack or []))
        for subject, name, explanation, _ in acknowledged:
            lines.append(f"已确认接着盯（--ack {subject}:{name}）：{explanation}")
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            print_report(lines, alerts)
            return 3
        # 只剩交回之后宽限里的后台进程时不退：宽限一过再查一次，免得它们刚好在退出之后才够格报
        if not running and BROKEN_DETECTION != "processexit" and (leftover_pending_until is None or BROKEN_DETECTION == "leftoverexit"):
            print_report(lines, alerts)
            if watched:
                print(f"看门狗起的时候就在跑的 {len(watched)} 个进程都退出了。")
            else:
                print("看门狗起的时候这个 Claude 实例底下就没有要盯的进程（看门狗自己那一支、claude 本体、IDE、node 与别的看门狗不算）："
                      "长命令是已经跑完了，还是还没起？")
            return 0
        if time.time() >= report_deadline:
            print_report(lines, alerts)
            print(f"定时回报：已经看了 {arguments.max_minutes:g} 分钟，没有告警，盯着的进程还有 {len(running)} 个没退出。")
            print("   → 接着盯就用 run_in_background 再起一个看门狗："
                  f"{'--processes-only' if BROKEN_DETECTION == 'restarthint' else 'bash scripts/agents/watch.sh --processes'}")
            return 4
        next_check_in = min(arguments.interval_seconds, report_deadline - time.time())
        if leftover_pending_until is not None:
            next_check_in = min(next_check_in, leftover_pending_until.timestamp() + 1 - time.time())
        mark_selftest_ready()
        time.sleep(max(0.0, next_check_in))


def build_report(agent_ids, session_dir, thresholds, watch_started):
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
    # 被看的子 agent 所在的会话：交回之后丢下的后台进程、交回之后副本还在查这些会话里全部子 agent
    session_dirs = {os.path.normpath(session_dir)} if session_dir else set()
    session_dirs |= {os.path.dirname(os.path.dirname(path)) for _, path in targets if path}
    # 根分区余量（文件头「根分区余量过低」）：每次整查都读
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
                # 钩子在执行前已经拒掉的命令（不经 task-server 跑项目代码被拒、认不出起什么的写法被拒……）：没有执行，拒绝信息与出路在它的工具结果里，
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
REFUSED_FINDING_PREFIXES = ("写被拒", "不经 task-server 跑项目代码被拒", "不经 task-server 起要经服务的外部程序被拒", "认不出起什么的写法被拒", "绕开 task-server 的写法被拒",
                            "要临时 prune-server 的自证写成了裸的 --selftest 被拒")


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


# --ack 的范围写法（文件头「--ack 的写法」）：类:<告警名> 确认这一类告警的全部（不分子 agent、不分进程）。
# 弄坏开关 ackscope 不认它（改前：只能按进程号、按 agent 号逐个确认）；ackscopepersist 下记进状态目录的这种确认在下一次整查被当成条件已消失而作废
ACK_CLASS_PREFIX = "类:"


def ack_scope_keys(alert):
    """一条告警能被哪种范围确认认：类:<告警名>。"""
    if BROKEN_DETECTION == "ackscope":
        return set()
    return {f"{ACK_CLASS_PREFIX}{alert[1]}"}


def split_acknowledged(alerts, acknowledgements):
    """主 agent 用 --ack 确认过「只是慢、接着盯」的告警：只写进报告，不叫醒主 agent。
    2026-09-19：规程写「只是慢就再起一个看门狗接着盯」，而「工具调用过长」这类条件在那次调用结束之前一直成立，
    新起的看门狗当场又报同一句退出，等于盯不住；确认只在这一次看门狗里有效，下一次起不带就失效。"""
    if BROKEN_DETECTION == "ack" or not acknowledgements:
        return alerts, []
    remaining, acknowledged = [], []
    for alert in alerts:
        subject, name, explanation, _ = alert
        keys = {f"{subject}:{name}"} | ack_scope_keys(alert)
        if name == LEFTOVER_BACKGROUND_ALERT and BROKEN_DETECTION != "leftoverack":
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
    if BROKEN_DETECTION != "ackscopepersist":
        keys |= ack_scope_keys(alert)
    process_id = re.match(r"进程 (\d+) ", explanation)
    if process_id and name == LEFTOVER_BACKGROUND_ALERT:
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
              "→ 怎么办：盯子 agent 就去掉 --processes-only（子 agent 在等的后台任务照样在报告里列出开着输出文件的进程）；只盯主 agent 自己放后台的长命令就只给 --processes-only"
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
    if arguments.processes_only:
        table = process_table()
        process_root_pid = arguments.process_root_pid if arguments.process_root_pid is not None else claude_instance_pid(table, os.getpid())
        if process_root_pid is None:
            print("✗ 往上找不到跑这个看门狗的 Claude 实例，--processes-only 没有进程可盯\n"
                  "→ 怎么办：在 Claude 的 Bash 工具里起它，或者用 --process-root-pid 指定从哪个进程往下看", file=sys.stderr)
            return 2
        return run_processes_only(arguments, process_root_pid, own_process_chain(table, os.getpid()))
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

    def acknowledgements():
        keys = set(arguments.ack or []) | (read_acks(state_dir) if discover else set())
        if BROKEN_DETECTION == "ackclassfiltered":
            # 弄坏开关（只给 watch.sh --selftest 自检⑬用，那一格经 watch.sh 真起看门狗子进程）：滤掉 类: 开头的键，--ack 类:<告警名> 确认过的告警照样叫醒
            keys = {key for key in keys if not key.startswith(ACK_CLASS_PREFIX)}
        return keys
    while True:
        if discover:
            agent_ids = discovered_agent_ids(arguments.session_dir, state_dir, discovery_cache, arguments)
        lines, alerts, all_done, active_lists, leftover_pending_until = build_report(agent_ids, arguments.session_dir, arguments, watch_started)
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
            print_report(lines, alerts)
            print_active_list(*active_lists, discover)
            return 3
        # 全部交回照旧退出，这一次 build_report 已经把「交回之后后台还在跑」查过了；只剩宽限里的，等宽限过了再查一次才退
        if all_done and (leftover_pending_until is None or BROKEN_DETECTION == "leftoverexit"):
            print_report(lines, alerts)
            print("被看的子 agent 全部交回或被停。")
            return 0
        if time.time() >= report_deadline:
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
            # 检出等不到下一次整查：读到本会话的新检出就马上整查一次，有没确认的告警就退 3
            if detection_alerts:
                lines, alerts, _, active_lists, _ = build_report(agent_ids, arguments.session_dir, arguments, watch_started)
                alerts, informational_lines = split_informational(alerts)
                lines += informational_lines
                alerts, acknowledged = split_acknowledged(alerts + detection_alerts, acknowledgements())
                if alerts:
                    lines = (lines + detection_notes + scope_declaration_section(scope_declarations)
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
        handle.write(f"# 报告\n\n## 没做什么\n\n- 没删 `{paths['report-repo']}`：主 agent 要在里面交 crates 变异那一道带 --item\n")

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
    report_lines, report_alerts, *_ = build_report(["keptold"], session, kept_thresholds, time.time())
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
    # 会话临时目录根下本会话的那一份照查（Claude Code 的 scratchpad 在里面）：不给 AGENT_WATCH_SCRATCH_ROOTS 时 scratch_hog_roots 交工作根与它；
    # 在真的会话临时目录根下造 <项目目录名>/<会话号>/scratchpad/rerun/target（项目目录名那一层是自己建的临时目录），只扫本会话那一份，不扫真的工作根
    session_temporary_root = work_roots.claude_session_temporary_root()
    os.makedirs(session_temporary_root, exist_ok=True)
    project_directory = tempfile.mkdtemp(prefix="-home-agent-watch-selftest-", dir=session_temporary_root)
    try:
        session_id = f"agent-watch-selftest-{os.getpid()}"
        session_directory = os.path.join(project_directory, session_id)
        scratchpad_build = os.path.join(session_directory, "scratchpad", "rerun", "target")
        os.makedirs(os.path.join(scratchpad_build, "release"))
        with open(os.path.join(scratchpad_build, "CACHEDIR.TAG"), "wb") as handle:
            handle.write(tag)
        moment = time.time() - 30 * 3600
        for directory, directories, files in os.walk(scratchpad_build, topdown=False):
            for name in [*files, *directories]:
                os.utime(os.path.join(directory, name), (moment, moment), follow_symlinks=False)
            os.utime(directory, (moment, moment))
        saved_override = os.environ.pop(SCRATCH_ROOTS_ENV, None)
        try:
            roots = scratch_hog_roots({session_id})
        finally:
            if saved_override is not None:
                os.environ[SCRATCH_ROOTS_ENV] = saved_override
        checks += 1
        if session_directory not in roots:
            failures.append(f"会话 scratchpad 所在的本会话目录在草稿根里：应当有 {session_directory}，实际 {roots}")
        lines = scratch_hog_report([session_directory], datetime.now(timezone.utc), 6)
        checks += 1
        if not any(line.startswith(f"  编译目录 {scratchpad_build}：") for line in lines):
            failures.append(f"会话 scratchpad 里超阈值的编译目录照报：应当报 {scratchpad_build}，实际 {lines}")
    finally:
        shutil.rmtree(project_directory, ignore_errors=True)
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
            or not all(text in found[0][2] for text in ("先查各会话的副本", "crash-amplification/runs/", "du -sh", "别人在用的不删",
                                                        f"--ack {ACK_CLASS_PREFIX}{ROOT_FREE_ALERT}"))):
        failures.append(f"根分区可用 40 GiB、阈值 128 GiB 时应当报一条「{ROOT_FREE_ALERT}」，说明写可用与阈值、出路先点名各会话的副本、再点名崩溃放量 runs/ 下的运行目录、"
                        f"以 du 为准、别人在用的不删、--ack 写法，实际 {found}")
    checks += 1
    if asked != [root_free_path()] or not os.path.exists(asked[0]):
        failures.append(f"根分区可用量应当只读一次、读的是工作根（还没建的取第一个在的上层）{root_free_path()}，实际读了 {asked}")
    # 崩溃放量家目录（crash_amplification_home）：没设变量是工作根下的 crash-amplification、不经家目录，设了照它；出路点名的是此刻环境下那一份的 runs/
    checks += 1
    expected_home = os.path.join(work_roots.work_root(), CRASH_AMPLIFICATION_ENTRY)
    configured_home = os.path.join(work, "amplification-home")
    unset_home = crash_amplification_home({})
    set_home = crash_amplification_home({CRASH_AMPLIFICATION_HOME_VARIABLE: configured_home})
    # 出路那一半只在过线那一次报了告警时核：没报归「可用 40 GiB 时应当报」那一格判，这一格只判家目录取哪一份，不跟着那一格一起红
    if (unset_home != expected_home or set_home != configured_home
            or (found and f"{crash_amplification_home()}/runs/" not in found[0][2])):
        failures.append(f"没设 {CRASH_AMPLIFICATION_HOME_VARIABLE} 时崩溃放量家目录应当是工作根下的 crash-amplification（{expected_home}）、设了照它（{configured_home}），"
                        f"报了告警时出路点名那一份的 runs/，实际没设时 {unset_home}、设了时 {set_home}、出路 {found[0][2] if found else '没报（这一半不核）'}")
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
                             "失败（最近一次任务通知是 failed）")
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


# 钩子里「判不了而照常放行」记检出用的 findings 起头，从 .claude/hooks/ 的源码里抽：五种写法各有一条抽法（UNJUDGED_DETECTION_PREFIXES 上方的说明）
UNJUDGED_SOURCE_MARKER = re.compile(r"没判全|没判|判不了|照常放行")


def hook_unjudged_prefixes(hooks_directory):
    """从钩子目录的源码里抽出记检出用的「判不了」起头：{起头: [出处文件名…]}。五种写法：
    ① 名字以 PREFIX 收尾的常量，值里有没判、判不了或照常放行的字样；② findings 那一行里 f"<起头>：…" 的起头（起头里有没判全、没判、判不了字样）；
    ③ python3 /dev/fd/3 "$HOOK_DIR" <脚本名>.sh（入口自己记检出，findings 以「<脚本名> 」起头）；④ not_judged(f"<脚本名>.sh …")；⑤ noted.append(f"判不了 …")。"""
    found = {}

    def add(prefix, name):
        found.setdefault(prefix, [])
        if name not in found[prefix]:
            found[prefix].append(name)
    for name in sorted(os.listdir(hooks_directory)):
        if not name.endswith((".sh", ".py")):
            continue
        try:
            with open(os.path.join(hooks_directory, name), encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        for line in text.splitlines():
            if line.lstrip().startswith("#"):
                continue
            constant = re.match(r'^[A-Z][A-Z_]*PREFIX\s*=\s*"([^"]+)"', line)
            if constant and UNJUDGED_SOURCE_MARKER.search(constant.group(1)):
                add(constant.group(1), name)
            if "findings" in line:
                for match in re.finditer(r'(?<![\w"])f?"(?!")([^"{}：:\n]*(?:没判全|没判|判不了)[^"{}：:\n]*)[：:]', line):
                    add(f"{name} {match.group(1)[3:]}" if match.group(1).startswith("%s ") else match.group(1), name)   # printf 的 %s 是钩子名（bash 那一层起不来时的检出）
            for pattern in (r'python3 /dev/fd/3 "\$HOOK_DIR" ([\w.-]+\.sh) ', r'not_judged\(f"([\w.-]+\.sh) '):
                match = re.search(pattern, line)
                if match:
                    add(match.group(1) + " ", name)
            guard = re.match(r'^GUARD_SCRIPT_NAME\s*=\s*"([\w.-]+\.sh)"', line)
            if guard:
                add(guard.group(1) + " ", name)
            if re.search(r'noted\.append\(f"判不了 ', line):
                add("判不了 ", name)
    return found


def selftest_unjudged_prefixes(work, failures):
    """钩子判不了而照常放行记下的检出（UNJUDGED_DETECTION_PREFIXES）只记不叫醒：① 从 .claude/hooks/ 的源码里抽出的全部这类起头都在 INFO_DETECTION_PREFIXES 里
    （钩子里加了新起头而没登进来，这一格红）；② 造一个带新起头的假钩子目录，抽法抽得出、对不上表（证明 ① 会红）；③ 表里每个起头造一条检出记录，读下来都只进报告、一条告警都没有；
    同一条记录里夹着真检出的照旧叫醒。返回查了几格。"""
    checks = 0
    hooks_directory = HOOKS_DIRECTORY
    extracted = hook_unjudged_prefixes(hooks_directory)
    checks += 1
    uncovered = {prefix: sources for prefix, sources in extracted.items() if not prefix.startswith(INFO_DETECTION_PREFIXES)}
    if not extracted or uncovered:
        failures.append(f"钩子源码里记检出用的「判不了而照常放行」起头都应当在 INFO_DETECTION_PREFIXES 里（只记不叫醒，真碰到别叫醒主 agent），"
                        f"没登进表的：{uncovered or '一个都没抽到（抽法坏了）'}；加进 UNJUDGED_DETECTION_PREFIXES")
    fake_hooks = os.path.join(work, "unjudged-prefix-hooks")
    os.makedirs(fake_hooks)
    with open(os.path.join(fake_hooks, "fake-guard.sh"), "w", encoding="utf-8") as handle:
        handle.write('NEW_UNJUDGED_PREFIX = "新闸没判、照常放行"\n')
    with open(os.path.join(fake_hooks, "fake-printf-guard.sh"), "w", encoding="utf-8") as handle:
        handle.write('  printf "{\\"findings\\": [\\"%s 没判成：钩子 bash 那一层起不来\\"]}" "$name"\n')   # bash 那一层起不来时的检出：%s 是钩子名
    checks += 1
    fake = hook_unjudged_prefixes(fake_hooks)
    if "新闸没判、照常放行" not in fake or "新闸没判、照常放行".startswith(INFO_DETECTION_PREFIXES) \
            or "fake-printf-guard.sh 没判成" not in fake or "fake-printf-guard.sh 没判成".startswith(INFO_DETECTION_PREFIXES):
        failures.append(f"造一个带新起头的假钩子（常量一份、printf 的 %s 钩子名一份），抽法应当抽出「新闸没判、照常放行」与「fake-printf-guard.sh 没判成」、而表里都没有"
                        f"（这样第一格才会红），实际抽出 {fake}")
    path = os.path.join(work, "detections-unjudged.jsonl")
    with open(path, "w", encoding="utf-8") as handle:
        for prefix in UNJUDGED_DETECTION_PREFIXES:
            separator = "" if prefix.endswith(" ") else "："
            handle.write(json.dumps({"session_id": "s1", "agent_type": "主 agent", "agent_id": None, "command": "x",
                                     "findings": [f"{prefix}{separator}读不到共用库（样本）"], "outcome": "只记"}, ensure_ascii=False) + "\n")
        handle.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1", "command": "pgrep -f x",
                                 "findings": [f"{UNJUDGED_DETECTION_PREFIXES[0]}：样本", "命令里有 pgrep -f / pkill -f / killall"], "outcome": "只记"}, ensure_ascii=False) + "\n")
    checks += 1
    _, alerts, notes = read_detections(path, 0, {"s1"}, set())
    quiet_notes = [note for note in notes if "只记不叫醒" in note]
    if len(alerts) != 1 or len(quiet_notes) != len(UNJUDGED_DETECTION_PREFIXES):
        failures.append(f"UNJUDGED_DETECTION_PREFIXES 里 {len(UNJUDGED_DETECTION_PREFIXES)} 个起头各一条检出应当都只记进报告不叫醒，夹着真检出的那一条照旧叫醒（共 1 条告警），"
                        f"实际告警 {len(alerts)} 条、只记 {len(quiet_notes)} 条")
    return checks


def selftest_ack_scopes(failures):
    """--ack 的类范围（split_acknowledged、alert_keys、pruned_acks）：类:<告警名> 确认这一类告警的全部，不分子 agent、不分进程；进程:<pid> 只确认那一个进程；
    记进状态目录的类确认在告警还在时留着、告警没了作废；「进度文件不涨」归只记不叫醒的那一类。返回查了几格。"""
    checks = 0
    alerts = [(ROOT_FREE_SUBJECT, ROOT_FREE_ALERT, "工作根所在的文件系统只剩 40.0 GiB", "清理"),
              ("a1", LEFTOVER_BACKGROUND_ALERT, "进程 333 已跑 3 分钟，sleep 900", "停下它"),
              ("a2", LEFTOVER_BACKGROUND_ALERT, "进程 444 已跑 4 分钟，sleep 800", "停下它")]
    for label, acknowledgements, wanted in (
            (f"类:{ROOT_FREE_ALERT}", {f"类:{ROOT_FREE_ALERT}"}, [ROOT_FREE_ALERT]),
            (f"类:{LEFTOVER_BACKGROUND_ALERT}", {f"类:{LEFTOVER_BACKGROUND_ALERT}"}, [LEFTOVER_BACKGROUND_ALERT, LEFTOVER_BACKGROUND_ALERT]),
            ("进程:333", {"进程:333"}, [LEFTOVER_BACKGROUND_ALERT]),
            ("类里没有的告警名", {"类:不存在的告警"}, [])):
        checks += 1
        remaining, acknowledged = split_acknowledged(list(alerts), acknowledgements)
        got = sorted(alert[1] for alert in acknowledged)
        if got != sorted(wanted) or len(remaining) + len(acknowledged) != len(alerts):
            failures.append(f"--ack {label} 应当只确认告警 {sorted(wanted) or '无'}（别的类、别的进程的照报），实际确认了 {got or '无'}")
    checks += 1
    kept = pruned_acks({f"类:{ROOT_FREE_ALERT}", "类:不存在的告警"}, alerts, set())
    if kept != {f"类:{ROOT_FREE_ALERT}"}:
        failures.append(f"记进状态目录的 类: 确认应当在告警还在时留着、告警没了就作废，实际留下 {sorted(kept)}")
    checks += 1
    remaining, informational = split_informational([("a3", PROGRESS_STALE_ALERT, "结束本轮在等自己的后台任务，progress.md 超过 30 分钟没改", "看进程"), alerts[0]])
    if len(remaining) != 1 or len(informational) != 1 or PROGRESS_STALE_ALERT not in informational[0]:
        failures.append(f"「{PROGRESS_STALE_ALERT}」应当归只记不叫醒的那一类（报告里写成「只记不叫醒：…」，不进告警），实际剩下告警 {len(remaining)} 条、只记 {informational}")
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


def selftest_projects_root(work, failures):
    """没给 --session-dir 时按 id 找会话记录（find_transcript → projects_root）：在一棵临时树里，HOME 指到别处、脚本自己所在的仓也在别处，
    TASK_SERVER_SOURCE_REPOSITORY 指到来源仓，要找到来源仓会话目录里的那一份（HOME 指到的别处有一份同 id 的，不能选它）；
    AGENT_WATCH_PROJECTS_ROOT 设了仍优先（自证造假项目目录靠它）。返回查了几格。"""
    agent_id = "afeedbeef7654321"
    tree = session_directories.build_selftest_tree(os.path.join(work, "projects-root-tree"))
    wanted = os.path.join(tree["session_directory"], "selftest-session", "subagents", f"agent-{agent_id}.jsonl")
    decoy = os.path.join(tree["other_home"], ".claude", "projects", session_directories.mangle(tree["own_repository"]), "selftest-session", "subagents", f"agent-{agent_id}.jsonl")
    for path in (wanted, decoy):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("{}\n")
    found = find_transcript(agent_id, environment=tree["environment"])
    if found != wanted:
        failures.append(f"按 id 找会话记录取来源仓根的会话目录：HOME 指到别处时应当找到 {wanted}，实际 {found}")
    overridden = find_transcript(agent_id, environment=dict(tree["environment"], AGENT_WATCH_PROJECTS_ROOT=os.path.join(tree["other_home"], ".claude", "projects")))
    if overridden != decoy:
        failures.append(f"AGENT_WATCH_PROJECTS_ROOT 设了优先（自证造假项目目录用）：应当找到 {decoy}，实际 {overridden}")
    return 2


def selftest_scratch_mentions(failures):
    """派发提示行首「草稿目录：」那一行点名的草稿目录：工作根（展开的绝对路径与 ~ 写法）下的找得到 progress.md，会话临时目录根下的不认（取不到）；
    提示里另点名的别的子 agent 的目录不取，没有那一行的取不到。建的目录自己删。返回查了几格。"""
    class Prompt:
        def __init__(self, text):
            self.dispatch_prompt = text
    home = os.path.expanduser("~")
    checks, made = 0, []
    try:
        for root in (work_roots.configured_work_root(), work_roots.claude_session_temporary_root()):
            os.makedirs(root, exist_ok=True)
            made.append(tempfile.mkdtemp(prefix="agent-watch-selftest-mention-", dir=root))
            open(os.path.join(made[-1], PROGRESS_FILE_NAME), "w", encoding="utf-8").write("第一格跑完\n")
        new_directory, session_temporary_directory = made
        home_form = "~" + new_directory[len(home):] if new_directory.startswith(home + "/") else new_directory
        for label, written, wanted in (("派发提示点名工作根下的草稿目录（绝对路径）", new_directory, new_directory),
                                       ("派发提示点名工作根下的草稿目录（~ 写法）", home_form, new_directory),
                                       ("派发提示点名会话临时目录根下的草稿目录不认", session_temporary_directory, None)):
            checks += 1
            found = progress_file_of(Prompt(f"跑前登记 x。\n草稿目录：{written}/\n报告：{written}/result.md\n"))
            expected = None if wanted is None else os.path.join(wanted, PROGRESS_FILE_NAME)
            if found != expected:
                failures.append(f"{label}：应当找到 {expected}，实际 {found}")
        # 只认行首「草稿目录：」那一行（lib_prompt_scratch_line.py）：提示里另点名别的子 agent 的目录、它的 progress.md 改得更晚，也不取它；
        # 没有那一行的取不到、告警里写取不到（AGENT_WATCH_BREAK=progressanypath 下照改前取改得最晚的那一份，会话临时目录根下的草稿目录也认得出：那一格与下面这三格共四格红）
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
                                          "command": "bash x.sh", "findings": ["项目代码闸没判全：x.sh 读不进来"]}, ensure_ascii=False) + "\n")
    _, detection_alerts, detection_notes = read_detections(detections, 0, {"s1"}, set())
    checks += 1
    if [alert[0] for alert in detection_alerts] != ["a1"] or not any("只记不叫醒" in note and "写被拒" in note for note in detection_notes) \
            or not any("只记不叫醒" in note and "项目代码闸没判全" in note for note in detection_notes):
        failures.append("hook 检出：写被拒的与项目代码闸没判全的应当只记进报告不叫醒，按模式找进程的应当告警且主体是 agent 号 a1（--ack a1:hook 检出 只压它），"
                        f"实际告警主体 {[alert[0] for alert in detection_alerts]}，记录 {detection_notes}")
    # 钩子执行前已拒掉的检出（写了结局「被拒」的 b1，没写结局而发现以「…被拒」起头的旧记录 b2）只进报告、不叫醒；写了「只记」的 b3 照常告警，下一步说命令照常在跑
    # （AGENT_WATCH_BREAK=outcomeignored 下 b1、b2 被当成只记而叫醒；refusedwakes 下它们照旧叫醒：这一格都红）
    outcome_detections = os.path.join(work, "detections-outcome.jsonl")
    with open(outcome_detections, "w", encoding="utf-8") as detections_file:
        for agent_id, findings, outcome in (("b1", ["不经 task-server 跑项目代码被拒（implementation-writer）：cargo test 带 --workspace / --all"], "被拒"),
                                            ("b2", ["认不出起什么的写法被拒：命令词由变量拼出，执行前展开不了"], None),
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
    # 范围声明钩子的放行记录（scope_session_id、run_scope）：属于这一会话的 c1 与主 agent 那一条列进「范围声明」一节（命令、四段），不叫醒、不进检出记录那几行；
    # 别的会话的 c2 不列（AGENT_WATCH_BREAK=scopeunlisted 下一条都不列：这一格红）
    scope_detections = os.path.join(work, "detections-scope.jsonl")
    with open(scope_detections, "w", encoding="utf-8") as detections_file:
        for session_id, agent_id, agent_type, command in (
                ("s1", "c1", "implementation-writer", "bash .claude/gate.d/formal-verification.sh --item proof /w"),
                ("s1", None, "主 agent", "bash .claude/gate.d/doc-text.sh --check prime-marks"),
                ("s9", "c2", "implementation-writer", "bash scripts/crash/crash-point-replay.sh --item fixed-script")):
            detections_file.write(json.dumps({"hook": "run-scope-guard.sh", "agent_id": agent_id, "agent_type": agent_type, "command": command,
                                              "scope_session_id": session_id, "outcome": "放行",
                                              "run_scope": {"where": "命令文本", "kind_of_command": "items", "what": "样本",
                                                            "declaration": "种类=items; 跑什么=样本; 范围=1 项; 为什么=只改了这一道的判法所以只跑点名的这一项",
                                                            "segments": {"种类": "items", "跑什么": f"样本 {agent_id}", "范围": "1 项",
                                                                         "为什么": "只改了这一道的判法所以只跑点名的这一项"}}}, ensure_ascii=False) + "\n")
    scope_lines = []
    _, scope_alerts, scope_notes = read_detections(scope_detections, 0, {"s1"}, set(), scope_lines)
    section = scope_declaration_section(scope_lines)
    checks += 1
    if not (not scope_alerts and not scope_notes and len(section) == 3 and section[0].startswith("范围声明（run-scope-guard.sh 放行的 2 条")
            and "implementation-writer c1：bash .claude/gate.d/formal-verification.sh --item proof /w：种类=items；跑什么=样本 c1；范围=1 项；为什么=只改了这一道的判法所以只跑点名的这一项" in section[1]
            and section[2].startswith("  主 agent：bash .claude/gate.d/doc-text.sh --check prime-marks：种类=items")
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
    os.environ[SELFTEST_MARK_VARIABLE] = SELFTEST_MARK_CONTENT   # 自证拉起的子进程（report、watch，环境原样继承）靠它认：弄坏开关只在自证里认
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
                                    not_started_minutes=5, active_minutes=600, context_tokens=600_000, leftover_background_grace_minutes=2,
                                    ask_every_minutes=60, progress_stale_minutes=30)
    session = os.path.join(work, "session")
    failures = []
    # 看门狗的状态目录（--discover 记的确认与见过的子 agent）也不落进真的：这份自证起的看门狗子进程都写临时目录里这一份；--discover 那几格照旧另设
    os.environ[STATE_ROOT_VARIABLE] = os.path.join(work, "watch-state")
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
                     {"agentType": "implementation-writer", "description": "派发 90 分钟、失败（任务通知 failed）"})
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
                                                   argparse.Namespace(**dict(vars(thresholds), kept_copy_hours=6)), time.time())
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
    probed_processes = []   # 自证自己起的假进程（Popen），交回前逐个停；这一份的收尾只对这些进程号发信号
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
            ("session", "不经 task-server 跑项目代码被拒（主 agent）：cargo test 带 --workspace / --all", "experiment-runner", "healthy", 4, "执行前已被 hook 拒了"),
            ("session", "认不出起什么的写法被拒：命令词由变量拼出，执行前展开不了", "主 agent", None, 4, "执行前已被 hook 拒了")):
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
    # --processes-only：拿自检进程自己当那个 Claude 实例。盯着一个一直在写文件的、一个写着的文件一小时没动而且不占 CPU 的、一个假的 task-server/task wait
    # （它只在结局时写、睡着等，开着输出文件）：进程卡不卡归 task-server 判，看门狗不按写着的文件报，三个都在跑时一条告警都没有（退 4 定时回报、没有 ⚠️ 行）；
    # 一个两秒就退出、退出之后没人收尸（僵尸）的，盯到它退出就退 0；一个都没有也退 0 并写明。
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
    fake_wait_directory = os.path.join(work, "fake-task-wait", "task-server")
    os.makedirs(fake_wait_directory, exist_ok=True)
    with open(os.path.join(fake_wait_directory, "task"), "w", encoding="utf-8") as handle:
        handle.write("import sys, time\nif sys.argv[1:2] == ['wait']:\n    time.sleep(900)\n")   # 假的 task wait：睡着不退，由 python3 起，不用可执行位
    fake_wait_output = os.path.join(work, "fake-task-wait.output")
    fake_wait = subprocess.Popen([sys.executable, os.path.join(fake_wait_directory, "task"), "wait", "7"], stdout=open(fake_wait_output, "ab"))
    probed_processes += [writer, silent, fake_wait]
    try:
        # 按条件等：writer 写出了第一行、silent exec 成了 sleep、假 task wait 起来了，再起看门狗
        wait_until(lambda: os.path.isfile(writing_file) and os.path.getsize(writing_file) > 0 and (process_arguments(silent.pid) or [""])[0] == "sleep"
                   and "wait" in (process_arguments(fake_wait.pid) or []))
        writer_silent_wait_run = run_process_watch(["--max-minutes", "0.05"])
        alert_lines = [line for line in writer_silent_wait_run.stdout.splitlines() if line.startswith("⚠️")]
        if writer_silent_wait_run.returncode != 4 or alert_lines or f"进程 {fake_wait.pid}" not in writer_silent_wait_run.stdout:
            failures.append(f"--processes-only 盯着一直在写文件的进程 {writer.pid}、写着的文件一小时没动的进程 {silent.pid}、假的 task-server/task wait {fake_wait.pid}："
                            f"进程卡不卡归 task-server 判，应当一条告警都没有（退出码 4、报告里列着假 task wait），"
                            f"实际退出码 {writer_silent_wait_run.returncode}，告警：{alert_lines or '无'}")
    finally:
        for process in (writer, silent, fake_wait):
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
    quiet_checks = selftest_unjudged_prefixes(work, failures) + selftest_ack_scopes(failures)
    session_watch_checks = selftest_session_watch(work, thresholds, failures, watch_runs) + session_watch_extra_checks
    session_watch_checks += selftest_scratch_mentions(failures)
    session_watch_checks += selftest_projects_root(work, failures)
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
        print("    → 看 agent_alerts() / read_detections() / run() 的判法；"
              "AGENT_WATCH_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静五种告警都报得出，"
          f"交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具送达判（只结束本轮、交回后又被续做的、被交回闸拒之后再没送达的都不算，先被拒后送达的算，忙时排进队的续做消息也算续做；"
          f"交回之后被自己后台任务的完成通知叫醒、又调了工具再结束本轮的仍算交回），前台命令超时转进后台的也算它起过的后台任务，"
          f"被停认会话记录里的打断与主会话记录里的 TaskStop（停在等后台任务时的算、停了又被续做的不算、续做之后又停的算，主会话记录不在的标出来），"
          f"结束本轮而起过的后台任务都已完成、没有通知在路上的报「结束本轮却不会醒」（通知刚到正要醒的不报），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，"
          f"本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒，用量与工具结果分类对；"
          f"结束本轮认 end_turn 与 stop_sequence、stop_reason 为空的纯文字记录认主会话记录里这一轮的任务通知（文字之后 4 分钟还在生成、没有通知的不认，主会话记录不在时两分钟没接下文也认），"
          f"结束本轮之后收到续做消息就算在想，用 --ack 确认过的告警只进报告不叫醒，主 agent 自己被拒的写只进报告不叫醒、入口已拒的命令不再按禁用命令报、别的子 agent 的检出不叫醒这一个看门狗，"
          f"等自己的后台任务时列出开着输出文件的进程，结束本轮等自己的后台任务的不报无动静（输出文件已没有进程开着、又没收到通知的后台任务不算在等，报「结束本轮却不会醒」）；"
          f"--processes-only 只盯进程：一直在写文件的、写着的文件不动的、假 task wait 都在跑时 0 条告警，盯着的进程都退出了（退出之后没人收尸的僵尸也算）退 0、起的时候一个都没有也退 0 并写明，和 --agents、cost 混用退 2，"
          f"Bash 工具包着的命令在报告里只列 eval 里那一段；"
          f"还没交回的子 agent 从派发起每跨过一个整点报「跑满 N 小时要主 agent 问一次」（报已跨过的最近那一格，--ack <agent 号>:跑满 N 小时 只确认那一格；"
          f"第一格之前收到的消息不重新计时，那一格之后主 agent 发过消息的不报——送达的 user 记录与排队附件、主会话记录里还没送达的 SendMessage 三处都认；"
          f"交回的、任务通知 failed 的不报）；"
          f"交回、被 TaskStop 停掉、最近一次任务通知是 failed 的子 agent 过了宽限还有进程开着它后台任务的输出文件报「交回之后后台还在跑」（交回不到宽限、没交回还在等、"
          f"stopped 之后又被续做的不报，进程没了不报，--ack 进程:<pid> 不叫醒），被看的全部交回时退出前也报、只剩宽限里的等宽限过了再查，--processes-only 带 --session-dir 也报、不带就写明没查；"
          f"交回超过 6 小时、交回正文或它点名的报告里写了「没删 <全路径>」而路径还在的报「{KEPT_COPY_ALERT}」（交回不到 6 小时、已经删掉的、没交回的不报，"
          f"--ack <agent 号>:{KEPT_COPY_ALERT} 不叫醒）"
          f"；钩子判不了而照常放行记下的检出只记不叫醒（{quiet_checks} 格）"
          f"；上下文过线只对实现员与执行员报「该交接」并列出吃上下文最多的工具输出、线写 0 不报，等后台任务时 progress.md 久没改报「进度文件不涨」、整点询问附 progress.md 末几行"
          f"（progress.md 只在派发提示行首「草稿目录：」那一行点名的目录下找：提示里另点名的别的子 agent 的目录不取，没有那一行的报取不到），"
          f"hook 检出的主体是 agent 号，执行前已被 hook 拒了的检出只记进报告不叫醒；重启前旧进程留下的子 agent 记录（会话已不在、或最后一条记录早于现在这个进程的启动）判已失效不报；有子 agent 交回时草稿根里超阈值的编译目录与仓副本按最后修改时刻列在报告里；"
          f"根分区可用量低于阈值报「{ROOT_FREE_ALERT}」（可用量换成假的判过线与不过线、阈值 0 不查，--agents 与 --processes-only 两条路都报、--ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT} 不叫醒）；"
          f"--discover 一个会话一个看门狗（自己找没结束的子 agent、已有一个在盯就退 5）、--ack 落文件下一次读回、条件消失的确认作废（{session_watch_checks} 格）；"
          f"没起过后台任务、结束本轮没交回的报「{STOPPED_MIDWAY_ALERT}」并在报告里另起一节带重派命令"
          f"（查了 {len(expectations) + 1 + leftover_cases_checked + 1} 个子 agent、{len(probed_processes)} 个进程、{len(watch_runs)} 次看门狗）")
    return 0


SELFTEST_MARK_VARIABLE = "AGENT_WATCH_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "agent-watch.py --selftest 起的子进程"


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认：命令行不是单独一个 --selftest、环境里也没有自证留的标记（自证起的子进程）而 AGENT_WATCH_BREAK
    （或经 subagent_transcript.py 进来的 SUBAGENT_TRANSCRIPT_BREAK）非空，参数解析之前就拒，交回退出码 2；放行交回 None。"""
    set_names = [name for name in ("AGENT_WATCH_BREAK", "SUBAGENT_TRANSCRIPT_BREAK", "SESSION_DIRECTORY_BREAK") if os.environ.get(name)]
    if not set_names or arguments == ["--selftest"] or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    for name in set_names:
        print(f"  ✗ {name}={os.environ[name]} 是弄坏开关，只在自证（agent-watch.py --selftest）里认，平常跑设了就拒，这一趟什么都没做", file=sys.stderr)
    print(f"     → 怎么办：去掉 {'、'.join(set_names)} 再跑；要证红就跑 {set_names[0]}=<项> python3 scripts/agents/agent-watch.py --selftest", file=sys.stderr)
    return 2


def main():
    refused = refuse_break_outside_selftest(sys.argv[1:])
    if refused is not None:
        return refused
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
    parser.add_argument("--not-started-minutes", type=float, default=5, help="派发之后这么久还没有会话记录告警，默认 5 分钟")
    parser.add_argument("--leftover-background-grace-minutes", type=float, default=2,
                        help="子 agent 交回、被停或失败之后过了这么久，它起的后台任务还有进程开着输出文件就报「交回之后后台还在跑」，默认 2 分钟")
    parser.add_argument("--kept-copy-hours", type=float, default=6,
                        help="子 agent 交回超过这么多小时、交回报告里写了「没删 <全路径>」而那个路径还在，就报「交回之后副本还在」交主 agent 删；默认 6 小时，写 0 不查")
    parser.add_argument("--scratch-idle-hours", type=float, default=6,
                        help="草稿根（工作根，与会话临时目录根下本会话的那一份）里的编译目录（带 CACHEDIR.TAG）与仓副本最后修改距今超过这么多小时，有子 agent 交回时列在报告里"
                             "（只报不删，交主 agent 核过没人在用再删）；默认 6 小时（推的：与 --kept-copy-hours 同一档，一个子 agent 一件活通常不到这么久，"
                             "停了这么久的副本多半没人再用），写 0 不查")
    parser.add_argument("--root-free-gib-alert", type=float, default=ROOT_FREE_GIB_ALERT_DEFAULT,
                        help="工作根所在文件系统（根分区）的可用量低于这么多 GiB 就报「根分区余量过低」、叫醒主 agent，每次整查都读；写 0 不查。"
                             "默认 128 GiB（推的，没量过）：crates 变异门禁一个工作进程有两份编译目录（普通的、插桩的），每份上限 64G"
                             "（.claude/gate.d/crates-mutation-replay.sh 的 GATE_MUTATION_TARGET_DIR_CAP），余量不到 128 GiB 就装不下再起一个工作进程；"
                             "双机驱动改前每趟留两份库拷贝（取回的与送出的，113 份合计 1159G、一份约 10G），放量段约每 8 分钟一趟，从 128 GiB 到写满约 45 分钟，够几次整查叫醒主 agent")
    parser.add_argument("--ask-every-minutes", type=float, default=60,
                        help="还没交回的子 agent 从派发起每跑满这么久（整数倍）报一次「跑满 N 小时要主 agent 问一次」，"
                             "那一格之后主 agent 给它发过消息就不报；默认 60 分钟，写 0 不报")
    parser.add_argument("--active-minutes", type=float, default=180, help="只给 --session-dir 时，只看这么久以内动过的子 agent")
    parser.add_argument("--detections-file", default=DEFAULT_DETECTIONS, help="hook 的检出记录，默认工作根下的 agent-hook-detections.jsonl（.claude/hooks/lib_work_root.py）")
    parser.add_argument("--detection-poll-seconds", type=float, default=15,
                        help="watch 读检出记录的间隔（读到本会话的新检出就马上整查一次），默认 15 秒")
    parser.add_argument("--ack", action="append", default=[],
                        help="主 agent 看过、判定只是慢接着盯的告警：子agent id:告警名，交回之后后台还在跑的进程写 进程:PID，"
                             "一类告警的全部写 类:<告警名>（例：类:交回之后后台还在跑），"
                             "「跑满 N 小时要主 agent 问一次」可写 子agent id:跑满 N 小时（只确认那一格）；可给多次；不带 --discover 时只在这一次看门狗里有效")
    parser.add_argument("--process-root-pid", type=int, help="只给 --processes-only 用：从哪个 pid 往下看；默认是跑这个脚本的 Claude 实例")
    parser.add_argument("--processes-only", action="store_true",
                        help="不看子 agent，只盯这个 Claude 实例底下起的时候就在跑的进程，它们都退出了就退 0（进程卡不卡、用多少内存与核数归 task-server 判，这里不报）；"
                             "带 --session-dir 时同时查那个会话里交回之后后台还在跑")
    arguments = parser.parse_args()
    return run(arguments)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main())
