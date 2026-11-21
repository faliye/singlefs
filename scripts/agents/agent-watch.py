#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""子 agent 的定时监控：卡死、空转、永久等待当场报出来。任务（卡住、过长、内存、核数、判定行、排队、撞上限、显卡与异常结局）归 task-server 判、停，经 `task wait` 叫醒提交它的 agent；这里不读服务、不判任务、不停任务。

为什么：子 agent 会等一行永远不会出现的字（例：把 `echo "exit=$?"` 打到标准输出、却在日志里 `until grep -q "^exit="`），主 agent 查进度才发现；这里把它当场报出来。
任务监控归 task-server：任务卡住只按「距这条任务的日志上一次增长满 10 分钟」由服务停掉、
报错、经 `task wait` 叫醒提交它的 agent，远超记录用时、核数过少只写进服务的 `task status`。
这一份不判任务进程（进程写的文件不涨与跑得过久、没经 task-server 跑的项目代码、常驻内存过线、在用核数过少、日志里的判红行、显卡坏了，以及读内存包装、
multi-host-run.sh 写的告警记录），也不盯任务服务（起 control check 判服务通不通、读 task status 列没 clean 的编译目录与判本会话任务正不正常）：任务系统自带提醒。

用法：
    agent-watch.py report --agents ID[,ID…]          # 看一眼：每个子 agent 的状态、最后一条命令、告警
    agent-watch.py report --session-dir DIR           # 看这个会话里最近活动过的全部子 agent
    agent-watch.py watch  --agents ID[,ID…]          # 看门狗：每隔一段时间查一次，有告警或全部结束就退出
    agent-watch.py watch  --processes-only            # 不看子 agent，只盯这个 Claude 实例底下起的时候就在跑的进程，等它们都退出：主 agent 自己放后台的长命令用它等
    agent-watch.py cost   --agents ID[,ID…]          # 用量：调用次数、起步与平均上下文、新输入、读缓存、输出、整份重写、工具结果被什么撑大
    agent-watch.py watch … --ack <键>                # 看过、判定只是慢的告警不再为它叫醒；键三种：进程:<pid>、<agent 号>:<告警名>、类:<告警名>（写法见下面「--ack 的写法」）
    agent-watch.py --selftest                         # 造假的会话记录与假进程走一遍各种告警；AGENT_WATCH_BREAK=<项> 时必须判红

主 agent 派发之后用 Bash 的 run_in_background 起 watch：它一退出，harness 就会叫醒主 agent。
它看、它报，不停子 agent、不停进程：结束不结束、停不停由主 agent 看了报告定；卡住的任务由 task-server 按日志停。
退出码：0 被看的子 agent 全部交回或被停、没有告警；3 有告警；4 定时回报（没有告警、子 agent 还在跑；起来之后满 --max-minutes，或离上次知识同步提醒满 --max-minutes 先到，
到点叫醒主 agent 看一眼）；2 用法错。3 与 4 离上次提醒满 --max-minutes 时带一节「知识同步提醒」（见下面）。
带 --processes-only 时 0 是看门狗起的时候就在跑的那些进程都退出了（起的时候一个都没有也退 0，报告里写明是哪一种）。
退出时出路句印的重起命令是 watch.sh 认得的写法（主 agent 经它起看门狗，它不认这里的 --agents / --processes-only）：盯点名的子 agent 印
`bash scripts/agents/watch.sh <agent 号>,…`，--discover 印不带 agent 号的 `bash scripts/agents/watch.sh`，--processes-only 印 `bash scripts/agents/watch.sh --processes`。
AGENT_WATCH_BREAK=restarthint 印回这里自己的选项，自检里出路句的几格必须判错。

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
起过后台任务、都完了而没东西叫醒的照旧报「结束本轮却不会醒」。弄坏开关 AGENT_WATCH_BREAK=midwaymerged 把「中途停下没交回」并进「结束本轮却不会醒」、不出那一节。

hook 的检出也在这里汇给主 agent：`.claude/hooks/bash-command-detector.sh` 执行前拒绝几种写法（列在它的文件头），其余只记不拦，把检出写进检出记录；
watch 每 15 秒读一次，读到被看子 agent 所在会话的检出就退出、叫醒主 agent。
告警的下一步按检出记录的 outcome 一栏说：「被拒」是执行前已拒、没执行，「只记」是照常执行；没写这一栏的旧记录按发现的起头认（detection_outcome）。
「被拒」的检出（钩子在执行前已经拒掉：写被拒、越出写范围、不经 task-server 跑项目代码被拒、认不出起什么的写法被拒……）与判不了的脚本（项目代码闸没判全）一样，
只写进报告、不叫醒主 agent（.claude/skills/dispatch/SKILL.md「派出去之后」「读到告警的处置」第一项说的只记不处置那一类：拒绝信息与出路在它的工具结果里，主 agent 醒了没有可处置的）；
叫醒的只有「只记」的、照常在跑的命令；被拒的命令叫醒看门狗，主 agent 只会反复重起它。弄坏开关 AGENT_WATCH_BREAK=refusedwakes。
钩子判不了而照常放行、往检出记录补一行的（findings 以「<闸>没判、照常放行」「<闸>没判全」「<脚本名>.sh 」「判不了 」这类起头，全表是 UNJUDGED_DETECTION_PREFIXES）同样只写进报告、不叫醒：
判不了不是出了事，主 agent 醒了没有可处置的（.claude/skills/dispatch/SKILL.md「派出去之后」「读到告警的处置」第一项：钩子已经拒掉的事与判不了的检出只记进调度记录）。表里没有的新起头真碰到会叫醒，
所以自证 selftest_unjudged_prefixes 从 .claude/hooks/ 的源码里抽出全部这类起头（名字以 PREFIX 收尾的常量、findings 那一行里的 f"<起头>：…"、入口自己记检出时的脚本名）与这张表比，
钩子里加了新起头而没登进表的判红；加新起头时，钩子文件头的说明与这张表一起改。弄坏开关 AGENT_WATCH_BREAK=unjudgedwakes 去掉这一组；
钩子拆成入口 .sh 加同层 .py 部分文件时，起头常写在部分文件里，抽法两种后缀都读，弄坏开关 AGENT_WATCH_BREAK=unjudgedpartsunread 只读 .sh。
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
--ack 进程:PID 确认。交回之后丢下的 `until … do sleep` 会一直跑下去，只盯没交回的子 agent 就没人报它。
被看的子 agent 全部交回时照旧退出，但退出前查一遍这一类；只剩宽限里的，等宽限过了再查一次才退。
只认会话记录里写着输出文件路径的后台任务：命令里自己用 `&`、`nohup` 放出去又改了输出去向的进程，与子 agent 之间没有可认的联系，这里认不出。
交回那一刻由交回闸 .claude/hooks/handback-guard.sh ④ 拦（同一套认法，从这里导入 background_start_in_result 与 own_background_tasks_still_running）：
交回时自己起的后台任务还有进程开着输出文件就拒交回；这里报的是闸管不到的（被停、失败，交回闸找不到会话记录或导入失败放行的）。

「跑满 N 小时要主 agent 问一次」：还没交回、没被停（任务通知也不是 failed / killed / stopped）的子 agent，从派发（它会话记录的第一条）起
每跑满 --ask-every-minutes（默认 60 分钟，即每个整点）报一次，N 是已跨过的最近那一格；主 agent 中途发的消息不重新计时。
那一格之后主 agent 给它发过消息就算问过了、这一格不再报：看门狗每次起都从头读，按「那一格之后有没有消息」判才做得到每个整点只报一次。
主 agent 的消息取两处里晚的那个：子 agent 会话记录里送达的（origin 为 coordinator 的消息，或忙时排进队、附件自己带 origin coordinator 的
queued_command），与主会话记录里 SendMessage 发给它的成功结果（忙着跑一条长工具调用的子 agent，消息要等它下一轮才落进它自己的会话记录）。
--ack <agent 号>:跑满 N 小时 只确认那一格，下一格照报。子 agent 结束本轮去等后台任务、一等很久的，
主 agent 发一条消息它就当场醒来；看门狗只在后台任务全结束、通知没到时报「结束本轮却不会醒」，等的还开着就不报。

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
不看余量的话，根分区写满只会在写文件撞上 No space left on device 时才发现。弄坏开关 AGENT_WATCH_BREAK=rootfree。

「知识同步提醒」：告警退 3 与定时回报退 4（--agents 与 --processes-only 两路都是）时，离上次提醒满 --max-minutes 就带一节，免得主对话做久了、项目的知识不同步；
没满的退出不带，全部交回退 0、report 那几路不打，退出码不因这一节改变。退 4 的打在「定时回报：」那一行之后，退 3 的打在告警之后，都在重起看门狗的出路句之前
（--processes-only 退 3、退 4 都以重起看门狗那一行收尾；--agents 那一路还有子 agent 在跑时同样）。
定时回报的时刻（report_deadline_of）取「起来时刻 + --max-minutes」与「上次提醒 + --max-minutes」里早的那个，起来时就已经过了的第一次整查之后当场退 4：
两次提醒之间最长是 --max-minutes 加主 agent 重起看门狗的那一点时间；离上次提醒满了先到的，「定时回报：」那一行写明是提前回报。
emit_knowledge_sync_section 判带不带、先打出这一节（flush）再落盘，repository_status_section 出这一节，次序：
以一句写给主对话的提醒句起头（到点该同步项目最新变动与状态、照下面点名的文件重读；写出离上次提醒多少分钟，变过知识类文件的写出几份，一份都没有的写明没变）；
随后逐份点名变过的知识类文件、各标来源；再是「仓的现状」（分支名与 HEAD 的短哈希、标题；「基线：」一行写这份基线从哪来；基线之后的新提交；git status 此刻的计数；基线之后变了的别的路径）；
末尾一行「→ 同步：」出路。
基线落盘：看门狗起来时读会话目录（<家目录>/.claude/projects/<仓路径改写>/<会话号>，取法在同目录 lib_session_directory.py 文件头；这里用 --session-dir，没给时取被看的子 agent
会话记录所在的那一层）底下的 agent-watch-knowledge-sync.json（KNOWLEDGE_STATE_FILE_NAME）：上一只落盘的基线（HEAD、git status、范围内各路径与范围里被 git 忽略的各文件的内容摘要）
与上次提醒的时刻。起来时先删会话目录里这个文件名前缀、超过一小时没动的临时文件（落盘时被杀留下的）。读得出、记的是这个仓：上次提醒的时刻照它记的；快照在、起点 HEAD 核得到的
接着它比，快照是 null（上一只起来时没查成）或起点 HEAD 核不成的此刻重新记一份快照。没有、读不出（上次提醒或记下的时刻不是有限数、或晚于此刻，也算读不出）、对不上仓的，
此刻重新记一份、上次提醒的时刻记成这一只起来的时刻。记了的落盘；此刻那一份也查不成的照样落盘（快照照读回的那一份留着，没有就只落上次提醒的时刻）。「基线：」那一行写明从哪来。
退出时先重读落盘的那一份，上次提醒的时刻取它与这一只内存里晚的那个，满没满与提醒句里的分钟数都照它算：几只看门狗（--discover 与 --processes）盯同一个会话时，
一只刚提醒过，另一只就不再提醒。带了这一节：读到了此刻的仓就把基线换成此刻；没查成的快照不换；两种都把上次提醒的时刻记成此刻、落盘。落盘都是在同目录排他新建名字带随机段的
临时文件、写完改名换上；写不成（含编码出错）删掉那份临时文件，在打出的这一节之后补一行「基线没换成此刻」。摘要里的链接目标与路径不是合法 UTF-8 的，按 surrogateescape
照原字节写、照原字节读回。取不到会话目录的（--processes-only 没给 --session-dir，或 --agents 而被看的子 agent 都还没有会话记录）只记在这一只里、下一只接不上，「基线：」那一行写明。
知识类文件的范围写在表 KNOWLEDGE_SCOPE 里：CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、.claude/rules/、.claude/skills/、.claude/agents/、.claude/kb/、.claude/singlefs-ai-sop/rules/；
点名的先后写在表 KNOWLEDGE_PRIORITY 里：.claude/kb/milestone/、.claude/kb/decisions.md、.claude/kb/checks-owed.md 排最前，其次 CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md，其余按路径排。
变过的判法取三处的并集，不分提交没提交：基线之后的新提交动到的路径（基线的 HEAD 到此刻 HEAD 的 git diff --name-only -z）；工作区里相对基线变了的路径（两份 git status 按路径比）；
范围里被 git 忽略的文件（git status 与 git diff 都看不见，例：本仓 .gitignore 整个忽略 .claude/singlefs-ai-sop/）：git ls-files 列出范围底下被忽略的未跟踪文件，内容不经 git、
按摘要逐个比，改了、新加、删掉都点名。
git status 读 --porcelain=v1 -z --untracked-files=normal（GIT_STATUS_ARGUMENTS）：NUL 分隔、路径不加引号（带空格、双引号的照认），不受 status.branch、status.showUntrackedFiles、
color.status 这些配置影响；改名、复制那一条按两段读，原路径与新路径各算一份；同一路径出了几条的（git rm --cached 之后「D 」与「??」）几个状态码都留着，输出里用 + 连写。
每条 git 都带 -c core.quotepath=off -c core.fileMode=false（GIT_READ_OPTIONS）：只改执行位的不算改了，git status 不列它。git 的标准输出按 surrogateescape 读：
不是合法 UTF-8 的文件名照原字节留着，算摘要时按原字节找得到文件。
工作区那一半：范围里的路径基线与此刻都记了内容摘要的只比摘要，状态码变了而内容没变的不点名（只暂存）；有一头没记摘要的（基线时不在 git status 里）看状态码变没变；
基线时在 git status 里、此刻不在了的另标一种。git reset --soft 这类只挪 HEAD 的，由新提交那一半照 HEAD 之间的差点名。
内容摘要：文件取内容的 sha256，git status 写成一条的整个未跟踪目录取底下每个目录项的相对路径与摘要合起来，不存在写「不在」；只读普通文件（不跟随符号链接、非阻塞地打开，
链接记指向的路径，命名管道、套接字、设备只记类型、不打开）；一次快照里算全部摘要带总时限（DIGEST_TIMEOUT_SECONDS_DEFAULT，30 秒），超时算查不成。范围外的路径只比状态码。
来源标签七种：「新提交动到」「工作区改了没提交」「两样都有」「被 git 忽略、内容摘要变了」（两头都被忽略而内容变了、新加、删掉）、「新进 git 忽略」（基线时在 git status 里、
此刻被忽略了）、「移出 git 忽略」（基线时被忽略、此刻出现在 git status 里或被它列的未跟踪目录盖住）、「起来时未提交的改动已不在 git status 里」（基线里在 git status 里、此刻不在了、
新提交又没动到它：撤掉了）。输出里的路径、分支名与提交标题过一遍转义（visible_text）：换行、制表、回车写成 \\n、\\t、\\r，别的控制字符与行、段分隔符写成 \\x.. 或 \\u....，
不是合法 UTF-8 的字节写成 \\x..，反斜杠写成 \\\\：一份点名只占一行，造不出像「→」出路的一行。
「仓的现状」里：新提交逐条「短哈希 标题」，一条没有写「没有新提交」，HEAD 往回挪了的写明「HEAD 从 … 移到了 …，不是往前走」；git status 此刻的计数是已暂存、未暂存、未跟踪各几条
（从输出的原始条目现数，同一路径出了几条的各算各的，一条两边都有改动的两边各算一次）；变了的别的路径只列范围外的（范围里的上面已逐份点名），一条没变写「没变」。
末尾一行「→ 同步：」出路：还有子 agent 在跑的，先照这份报告最后一行「→ 接着盯」起下一只看门狗；再重读上面点名的知识类文件（一份没变的写不用重读）；
再核手上正在改或要改的文件有没有被别的会话改过、提交过，查谁改过、提到过在仓根跑 python3 scripts/agents/who-touched.py <路径>…（WHO_TOUCHED_COMMAND）。
--repository-report-lines（watch.conf 的同名键，默认 20）是每张清单（点名的知识类文件、新提交、变了的别的路径）最多列几条，余下的报条数、再一行一条看全部的命令；写 0 不打这一节、也不读不记基线。
给人抄的命令（「没查成」与「余 … 没列」之后的）每条单独一行、行首六个空格（COMMAND_INDENT），写成 git -c core.quotepath=off -C <仓> …，每一段按 shell 的写法加了引号，原样交给 shell 就能跑。
读哪个仓：--repository-root；缺省取统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY（与 lib_session_directory.py 取会话目录同一个来源，两台同一个值），没有这个变量取本脚本所在的仓。
git 查不成（不是仓、git 起不来、一条命令超过 30 秒、算摘要超时），与记基线、带这一节这两处外层接住的别的异常（写成「<异常名>：<原因>」）：提醒句照写，并写明变过哪些知识类文件
这一趟没查成，随后一行「没查成：<卡在哪、什么原因>」，再一行一条手动看的命令：git status、最近五条提交、范围里被 git 忽略的条目（git ls-files --others --ignored --exclude-standard，
git status 不列它们）。git 命令带时限、标准错误的第一行写进原因，只读（GIT_OPTIONAL_LOCKS=0，不刷新索引）；不继承的变量：起来时现取一次 git rev-parse --local-env-vars 列的仓内变量
（取不成用写死的 GIT_LOCAL_ENVIRONMENT_FALLBACK 那 15 个：GIT_ALTERNATE_OBJECT_DIRECTORIES、GIT_CONFIG、GIT_CONFIG_PARAMETERS、GIT_CONFIG_COUNT、GIT_OBJECT_DIRECTORY、GIT_DIR、
GIT_WORK_TREE、GIT_IMPLICIT_WORK_TREE、GIT_GRAFT_FILE、GIT_INDEX_FILE、GIT_NO_REPLACE_OBJECTS、GIT_REPLACE_REF_BASE、GIT_PREFIX、GIT_SHALLOW_FILE、GIT_COMMON_DIR），另加 GIT_NAMESPACE。
环境变量只给 --selftest 用：AGENT_WATCH_REPOSITORY_ROOT（--repository-root 的缺省，自证不碰真仓）、AGENT_WATCH_GIT_TIMEOUT_SECONDS（每条 git 的时限）、AGENT_WATCH_GIT_EXECUTABLE（换掉起的 git）、
AGENT_WATCH_DIGEST_TIMEOUT_SECONDS（算摘要的总时限）。
弄坏开关见下面开关表里 repository 起头与 knowledge 起头的那几条，与 processesalertrestartomitted。

「重启前旧进程留下的记录判已失效」：每次整查读 ~/.claude/sessions/<pid>.json（Claude Code 给每个进程登记的会话号、启动时刻与 procStart，pid 还在而且 /proc/<pid>/stat 第 22 列对得上才算活着），
没交回的子 agent 记录有一种算已失效：一、它所在的会话号不是本会话（--session-dir 的目录名，没给取 CLAUDE_CODE_SESSION_ID）、登记里也没有活着的 Claude 进程；
二、会话号还活着（重启后接着用同一个会话号），而它最后一条记录早于那个会话现在活着的最早一个进程的启动时刻。已失效的在报告里标「状态=已失效」，
不按「无动静」「工具调用过长」这类报，不算要盯的，也不拦着全部交回时退出；--discover 不再把它记成没结束的。登记读不出、或里面没有一份能解的，判不了，一律不当失效。
弄坏开关 AGENT_WATCH_BREAK=stalesession、stalesessionrestart。

弄坏开关 AGENT_WATCH_BREAK=<项>（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
（项名是那条失败落在哪个自证函数里：selftest_in、selftest_session_watch_in、selftest_scratch_hogs、selftest_stale_sessions、selftest_leftover_background、selftest_kept_copies、
selftest_root_free、selftest_ack_concurrency、selftest_unjudged_prefixes、selftest_ack_scopes、selftest_scratch_mentions、selftest_repository_report，临时目录那三格写格名；括号里是那几条「✗ 自检：」的开头；红了自证退 1）：
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
  AGENT_WATCH_BREAK=acksunlocked  update_acks() 读改写 watch-acks.tsv 不拿 watch-acks.lock 的 flock
      selftest_ack_concurrency（「✗ 自检：watch-acks.tsv 两只同时写」）
  AGENT_WATCH_BREAK=acksnotpersisted  read_acks() 不读状态目录里记下的确认
      selftest_session_watch_in（「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」、「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」、「✗ 自检：已有一个在盯时给的 --ack 应当记进状态目录」、
      「✗ 自检：条件类告警的确认」）、
      selftest_ack_concurrency（「✗ 自检：watch-acks.tsv 两只同时写」）
  AGENT_WATCH_BREAK=amplificationhome  crash_amplification_home() 没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时取家目录下那一份
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
      selftest_repository_report（「✗ 自检：知识同步提醒（全部交回与 report 不打）：」）、
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
  AGENT_WATCH_BREAK=inquiryreset  routine_inquiry_alert() 从主 agent 上一次消息起重新计时
      selftest_in（「✗ 自检：子 agent inquirymessagebefore」）
  AGENT_WATCH_BREAK=inquirysent  AgentTranscript.last_main_agent_message_timestamp() 不读主会话记录里 SendMessage 的成功结果
      selftest_in（「✗ 自检：子 agent inquirysentafter」）
  AGENT_WATCH_BREAK=staleselftestscratch  remove_stale_selftest_scratch() 一个都不删（自证被 SIGKILL 打断留下的 agent-watch-selftest-scratch-* 一直留着）
      selftest_stale_scratch_cleanup（「✗ 自检：被杀的自证留下的 agent-watch-selftest-scratch-*」）
  AGENT_WATCH_BREAK=keepscratch  selftest() 的临时目录走回 mkdtemp 不删
      通过那条路（「✗ 自检：通过那条路上临时目录没删」）、red 那条路（「✗ 自检：red 那条路上临时目录没删」）、crash 那条路（「✗ 自检：crash 那条路上临时目录没删」）
  AGENT_WATCH_BREAK=keptcopy  kept_copy_report() 一条都不报
      selftest_kept_copies（「✗ 自检：「交回之后副本还在」应当只报交回超过 6 小时」、「✗ 自检：看门狗的报告应当带「交回之后副本还在」告警与一行汇总」）
  AGENT_WATCH_BREAK=keptcopyage  kept_copy_report() 不看交回了多久，交回不到 --kept-copy-hours 的也报
      selftest_kept_copies（「✗ 自检：「交回之后副本还在」应当只报交回超过 6 小时」：keptrecent 也报了）
  AGENT_WATCH_BREAK=keptcopyexists  kept_copy_report() 不看路径还在不在，已经删掉的也报
      selftest_kept_copies（「✗ 自检：「交回之后副本还在」应当只报交回超过 6 小时」：deleted-repo 也报了）
  AGENT_WATCH_BREAK=knowledgecheckcommandomitted  repository_status_section() 的「→ 同步：」那一行不给查谁改过文件的命令（who-touched.py）
      selftest_repository_report（「✗ 自检：知识同步提醒（核手上的文件给命令）：」）
  AGENT_WATCH_BREAK=knowledgebaselineexceptionescapes  take_repository_baseline() 外层只接查不成，别的异常往外抛（每条 git 的时限写成不是数时看门狗起不来）
      selftest_repository_report（「✗ 自检：知识同步提醒（记基线时的异常照没查成记）：」）
  AGENT_WATCH_BREAK=knowledgecodechangenamed  worktree_path_changed() 状态码变了也算变了：基线时记了摘要、内容一字没变而只暂存的知识类文件也点名
      selftest_repository_report（「✗ 自检：知识同步提醒（只暂存不点名）：」）
  AGENT_WATCH_BREAK=knowledgedeadlinefromstart  report_deadline_of() 只看起来时刻加 --max-minutes，不看上次提醒：一次告警退出贴着满之前，两次提醒的间隔拖到约两倍 --max-minutes
      selftest_repository_report（「✗ 自检：知识同步提醒（定时回报离上次提醒满了就退）：」）
  AGENT_WATCH_BREAK=knowledgedigestblind  worktree_path_changed() 不比内容摘要、只看状态码：状态码没变而内容又改了的不点名，状态码变了而内容没变的照点
      selftest_repository_report（「✗ 自检：知识同步提醒（起来时已改内容又变）：」、「✗ 自检：知识同步提醒（只暂存不点名）：」、「✗ 自检：知识同步提醒（未跟踪目录的摘要）：」、
      「✗ 自检：知识同步提醒（文件名不是 UTF-8）：」）
  AGENT_WATCH_BREAK=knowledgedigestoutofscope  digest_wanted() 与 knowledge_files_changed() 范围外的路径也记内容摘要、状态码没变而摘要变了也点名（范围外的路径只比状态码）
      selftest_repository_report（「✗ 自检：知识同步提醒（范围外不点名）：」）
  AGENT_WATCH_BREAK=knowledgedigestuntimed  check_digest_deadline() 不看截止时刻：一次快照里算内容摘要的总时限不起作用
      selftest_repository_report（「✗ 自检：知识同步提醒（整次摘要超时走没查成）：」、「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」、
      「✗ 自检：知识同步提醒（没查成给列被忽略条目的命令）：」）
  AGENT_WATCH_BREAK=knowledgedirectorydigestblind  content_digest() 目录一律记「目录」：git status 写成一条的未跟踪目录底下变了也看不出
      selftest_repository_report（「✗ 自检：知识同步提醒（未跟踪目录的摘要）：」）
  AGENT_WATCH_BREAK=knowledgeduplicatepathmerged  parse_status_porcelain() 同一路径出两条时后一条盖掉前一条，只留最后那一码（git rm --cached 之后「D 」丢了、已暂存少数一个）
      selftest_repository_report（「✗ 自检：知识同步提醒（同一路径出两条）：」）
  AGENT_WATCH_BREAK=knowledgefilemodecounted  run_git() 不带 -c core.fileMode=false：只改执行位的文件出现在 git status 里，起来时干净、之后只改执行位的照状态码点名
      selftest_repository_report（「✗ 自检：知识同步提醒（起来时干净、之后只改执行位不点名）：」、「✗ 自检：知识同步提醒（只改执行位不点名）：」）
  AGENT_WATCH_BREAK=knowledgefiletypeunchecked  file_digest() 不判文件类型，照普通文件阻塞着打开：未跟踪的知识类目录里有没有写端的命名管道时记基线卡住、看门狗不退
      selftest_repository_report（「✗ 自检：知识同步提醒（未跟踪知识类目录里有命名管道）：」、「✗ 自检：知识同步提醒（链接目标不是 UTF-8 也落得了盘）：」）
  AGENT_WATCH_BREAK=knowledgegitoutputreplaced  run_git() 标准输出按 replace 读：不是合法 UTF-8 的字节换成 U+FFFD，那个文件按换过的名字找不到、改了不点名
      selftest_repository_report（「✗ 自检：知识同步提醒（文件名不是 UTF-8）：」）
  AGENT_WATCH_BREAK=knowledgeignoredblind  ignored_knowledge_paths() 一份都不列：范围里被 git 忽略的文件改了、新加、删掉都不点名
      selftest_repository_report（「✗ 自检：知识同步提醒（范围里被 git 忽略的条目）：」、「✗ 自检：知识同步提醒（被忽略条目的标签按发生的事分）：」）
  AGENT_WATCH_BREAK=knowledgeignoredcommandomitted  unchecked_section_lines() 不给列被忽略条目的那一条命令（「没查成」下面只有 git status 与最近五条提交）
      selftest_repository_report（「✗ 自检：知识同步提醒（没查成给列被忽略条目的命令）：」、「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」）
  AGENT_WATCH_BREAK=knowledgeignoredtagflat  ignored_knowledge_tag() 一律写「被 git 忽略、内容摘要变了」，不分新进忽略、移出忽略
      selftest_repository_report（「✗ 自检：知识同步提醒（被忽略条目的标签按发生的事分）：」）
  AGENT_WATCH_BREAK=knowledgeorderflat  knowledge_sort_key() 一律只按路径排，不先排里程碑、决策索引、欠账表与 CLAUDE.md、主 agent 规矩、共用约束
      selftest_repository_report（「✗ 自检：知识同步提醒（点名次序）：」）
  AGENT_WATCH_BREAK=knowledgepathunescaped  visible_text() 原样交回：路径里的换行原样打出，一份点名拆成两行、能造出像「→ 同步：」的一行
      selftest_repository_report（「✗ 自检：知识同步提醒（路径带换行与控制字符）：」）
  AGENT_WATCH_BREAK=knowledgeremindedatunchecked  knowledge_time_from_record() 照 float() 转得了就收：落盘的上次提醒被改成 NaN、无穷、将来的时刻或字符串也接着用
      selftest_repository_report（「✗ 自检：知识同步提醒（落盘的时刻不是有限数或晚于此刻就重新记）：」）
  AGENT_WATCH_BREAK=knowledgereminderlast  repository_status_section() 把提醒句放在这一节最后，不排第一行
      selftest_repository_report（「✗ 自检：知识同步提醒（提醒句排第一）：」、「✗ 自检：知识同步提醒（--processes-only 告警退出且已满带这一节）：」）
  AGENT_WATCH_BREAK=knowledgerenamesplitoff  parse_status_porcelain() 改名、复制那一条只认新路径，原路径那一段丢掉
      selftest_repository_report（「✗ 自检：知识同步提醒（状态码计数）：」、「✗ 自检：知识同步提醒（知识类文件改名）：」）
  AGENT_WATCH_BREAK=knowledgesavedbeforeprint  emit_knowledge_sync_section() 先落盘、再打出：打这一节时被停，落盘的上次提醒已换成此刻，这一节点名的那几份之后谁都不再点
      selftest_repository_report（「✗ 自检：知识同步提醒（先打出这一节再落盘）：」）
  AGENT_WATCH_BREAK=knowledgesectionexceptionescapes  emit_knowledge_sync_section() 外层只接查不成，别的异常往外抛（到点时每条 git 的时限写成不是数，看门狗退出码不再是 3 或 4）
      selftest_repository_report（「✗ 自检：知识同步提醒（带这一节时的异常照没查成写）：」）
  AGENT_WATCH_BREAK=knowledgestaletemporarykept  remove_stale_knowledge_state_temporaries() 一份都不删：落盘时被杀留下的临时文件一直留在会话目录里
      selftest_repository_report（「✗ 自检：知识同步提醒（落盘留下的旧临时文件起来时删掉）：」）
  AGENT_WATCH_BREAK=knowledgestatenotreread  emit_knowledge_sync_section() 退出时不重读落盘的那一份：两只看门狗读到同一份旧的上次提醒，前一只提醒过、后一只照样提醒且分钟数写错
      selftest_repository_report（「✗ 自检：知识同步提醒（两只读同一份只提醒一次）：」）
  AGENT_WATCH_BREAK=knowledgestaterepositoryunchecked  load_knowledge_state() 不核落盘的基线记的是哪个仓，别的仓的基线也照用
      selftest_repository_report（「✗ 自检：知识同步提醒（落盘的基线对不上仓或读不出就重新记）：」）
  AGENT_WATCH_BREAK=knowledgestatestrictencoding  knowledge_state_encoding_errors() 落盘那一份按严格的 UTF-8 读写：摘要里带着不是 UTF-8 的链接目标时写不了、落盘不成
      selftest_repository_report（「✗ 自检：知识同步提醒（链接目标不是 UTF-8 也落得了盘）：」）
  AGENT_WATCH_BREAK=knowledgetagsource  knowledge_files_changed() 来源标签恒写「新提交动到」，不分工作区改了没提交、两样都有、被 git 忽略、已不在
      selftest_repository_report（「✗ 自检：知识同步提醒（工作区改了没提交）：」、「✗ 自检：知识同步提醒（起来时已改内容又变）：」、「✗ 自检：知识同步提醒（范围里被 git 忽略的条目）：」、
      「✗ 自检：知识同步提醒（继承了带配置的 git 变量）：」、「✗ 自检：知识同步提醒（被忽略条目的标签按发生的事分）：」）
  AGENT_WATCH_BREAK=knowledgeunreadablenotadvanced  emit_knowledge_sync_section() 带了「没查成」的那一次不落盘：上次提醒的时刻不往前推，一直查不成时之后每次退出都带
      selftest_repository_report（「✗ 自检：知识同步提醒（起来时就查不成也按时提醒）：」、「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」）
  AGENT_WATCH_BREAK=knowledgeunreadablenotsaved  unreadable_baseline() 起来时查不成不落盘：上次提醒的时刻每只都从起来时重新算，告警比 --max-minutes 密时一次都不提醒
      selftest_repository_report（「✗ 自检：知识同步提醒（起来时就查不成也按时提醒）：」）
  AGENT_WATCH_BREAK=knowledgeworktreeblind  knowledge_files_changed() 工作区那一半不看（新出现、状态变了、摘要变了、不在了都不认），只认新提交动到的与被 git 忽略的
      selftest_repository_report（「✗ 自检：知识同步提醒（工作区改了没提交）：」、「✗ 自检：知识同步提醒（起来时已改内容又变）：」、「✗ 自检：知识同步提醒（工作区路径带空格与双引号）：」、
      「✗ 自检：知识同步提醒（本机 git 配置不改读数）：」、「✗ 自检：知识同步提醒（知识类文件改名）：」、「✗ 自检：知识同步提醒（未跟踪目录的摘要）：」、
      「✗ 自检：知识同步提醒（两只之间变了的被后一只点名）：」、「✗ 自检：知识同步提醒（路径带换行与控制字符）：」、「✗ 自检：知识同步提醒（文件名不是 UTF-8）：」、
      「✗ 自检：知识同步提醒（继承了带配置的 git 变量）：」）
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
  AGENT_WATCH_BREAK=midwaymerged  agent_alerts() 把没起过后台任务、结束本轮没交回的也报成「结束本轮却不会醒」，run() 不出「中途停下没交回的子 agent」一节
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
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」、检出「不经 task-server 跑项目代码被拒…」」、「✗ 自检：检出记录属于「session」「主 agent None」、检出「认不出起什么的写法被拒：…」」）、
      selftest_session_watch_in（「✗ 自检：hook 检出：执行前已被 hook 拒了的（b1 …）应当只记进报告、不叫醒」）
  AGENT_WATCH_BREAK=ownrefusal  read_detections() 主 agent 自己的命令被 hook 拒的不单记
      selftest_in（「✗ 自检：检出记录属于「session」「主 agent None」」）
  AGENT_WATCH_BREAK=processexit  run_processes_only() 盯的进程都退出了也不退
      selftest_in（「✗ 自检：--processes-only 盯着的进程退出了」、「✗ 自检：--processes-only 起的时候一个要盯的进程都没有」）
  AGENT_WATCH_BREAK=processesalertrestartomitted  run_processes_only() 告警退 3 不打重起看门狗那一行：带了这一节时「→ 同步：」叫主 agent 照的「→ 接着盯」不在
      selftest_repository_report（「✗ 自检：知识同步提醒（--processes-only 告警退出且已满带这一节）：」）
  AGENT_WATCH_BREAK=progressanypath  progress_file_of() 放宽取法：派发提示里点名的工作根或会话临时目录根下的路径都算候选、取改得最晚的那一份，不只认行首「草稿目录：」那一行
      selftest_scratch_mentions（「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告…」、「✗ 自检：派发提示没有行首「草稿目录：」一行、只点名别的子 agent 的报告…」、
      「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句应当写取不到 progress.md…」、「✗ 自检：派发提示点名会话临时目录根下的草稿目录不认」）
  AGENT_WATCH_BREAK=progressnote  progress_note() 告警里不附 progress.md 的末几行
      selftest_session_watch_in（「✗ 自检：整点询问应当附上 progress.md 的末几行」）
      「✗ 自检：派发提示没有行首「草稿目录：」一行时，告警里附的一句」、「✗ 自检：进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」
  AGENT_WATCH_BREAK=progressstale  agent_alerts() 不报「进度文件不涨」
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」）
  AGENT_WATCH_BREAK=progresstailhead  progress_note() 取 progress.md 的头几行、不取末几行
      selftest_scratch_mentions（「✗ 自检：进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」）
  AGENT_WATCH_BREAK=progresstailreversed  progress_note() 附的末几行倒着排（最后一行在前）
      selftest_scratch_mentions（「✗ 自检：进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」）
  AGENT_WATCH_BREAK=progressblankkept  progress_note() 不滤空行：空行与只有空白的行也算一行，末几行里混进空行
      selftest_scratch_mentions（「✗ 自检：进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」）
  AGENT_WATCH_BREAK=refusedwakes  read_detections() 钩子执行前已拒掉的检出（outcome 被拒）也叫醒，不单记进报告
      selftest_in（「✗ 自检：检出记录属于「session」「experiment-runner healthy」、检出「不经 task-server 跑项目代码被拒…」」、「✗ 自检：检出记录属于「session」「主 agent None」、检出「认不出起什么的写法被拒：…」」）、
      selftest_session_watch_in（「✗ 自检：hook 检出：执行前已被 hook 拒了的（b1 …）应当只记进报告、不叫醒」）
  AGENT_WATCH_BREAK=repeat  agent_alerts() 不报同一命令反复且输出不变
      selftest_in（「✗ 自检：子 agent repeat」）
  AGENT_WATCH_BREAK=repositorybackwardsunsaid  repository_status_section() HEAD 往回挪时不写「HEAD 从 … 移到了 …，不是往前走」
      selftest_repository_report（「✗ 自检：知识同步提醒（HEAD 往回挪）：」）
  AGENT_WATCH_BREAK=repositorybaselinekept  emit_knowledge_sync_section() 带了这一节也不换基线、不记上次提醒的时刻（落盘的那一份原样留着）
      selftest_repository_report（「✗ 自检：知识同步提醒（打过把基线换成此刻）：」、「✗ 自检：知识同步提醒（定时回报离上次提醒满了就退）：」、
      「✗ 自检：知识同步提醒（起来时就查不成也按时提醒）：」、「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」、「✗ 自检：知识同步提醒（两只读同一份只提醒一次）：」）
  AGENT_WATCH_BREAK=repositorychangesblind  paths_changed_since() 一概当没变：基线之后新出现、状态变了、不在了的别的路径都不列，只写「变了的别的路径：没变」
      selftest_repository_report（「✗ 自检：知识同步提醒（有新提交与改动）：」、「✗ 自检：知识同步提醒（清单超过上限）：」、「✗ 自检：知识同步提醒（--processes-only 有新提交与改动）：」、
      「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」）
  AGENT_WATCH_BREAK=repositorycommitsblind  new_commits_since() 一概当没有新提交：不比基线的 HEAD 与此刻的，新提交与它们带来的改动路径都是空
      selftest_repository_report（「✗ 自检：知识同步提醒（有新提交与改动）：」、「✗ 自检：知识同步提醒（新提交动了知识类文件）：」、「✗ 自检：知识同步提醒（点名次序）：」、
      「✗ 自检：知识同步提醒（工作区改了没提交）：」、「✗ 自检：知识同步提醒（清单超过上限）：」、「✗ 自检：知识同步提醒（--processes-only 有新提交与改动）：」、
      「✗ 自检：知识同步提醒（新提交文件名带双引号）：」、「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」）
  AGENT_WATCH_BREAK=repositorydiffnewline  new_commits_since() 改动过的路径改读不带 -z 的 git diff --name-only、按换行切：文件名带双引号的被 git 加了引号、对不上范围
      selftest_repository_report（「✗ 自检：知识同步提醒（新提交文件名带双引号）：」）
  AGENT_WATCH_BREAK=repositorydueonalertignored  emit_knowledge_sync_section() 告警退出一律不带这一节，离上次提醒满了也不带
      selftest_repository_report（「✗ 自检：知识同步提醒（告警退出且已满带这一节）：」、「✗ 自检：知识同步提醒（打过把基线换成此刻）：」、
      「✗ 自检：知识同步提醒（--processes-only 告警退出且已满带这一节）：」）
  AGENT_WATCH_BREAK=repositoryfailuresilent  repository_status_section() 查不成时也不写「没查成」，这一节是空的
      selftest_repository_report（「✗ 自检：知识同步提醒（仓根不是仓）：」、「✗ 自检：知识同步提醒（git 超时）：」、「✗ 自检：知识同步提醒（git 起不来）：」、
      「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」、「✗ 自检：知识同步提醒（整次摘要超时走没查成）：」、「✗ 自检：知识同步提醒（起来时就查不成也按时提醒）：」、
      「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」、「✗ 自检：知识同步提醒（记基线时的异常照没查成记）：」、「✗ 自检：知识同步提醒（没查成给列被忽略条目的命令）：」）
  AGENT_WATCH_BREAK=repositoryheadomitted  repository_status_section() 不写「分支 …，HEAD …」那一行
      selftest_repository_report（「✗ 自检：知识同步提醒（有新提交与改动）：」）
  AGENT_WATCH_BREAK=repositoryinheritedcommondir  dropped_git_environment() 只去掉 GIT_DIR、GIT_WORK_TREE、GIT_INDEX_FILE，GIT_COMMON_DIR、GIT_CONFIG_PARAMETERS 这些照继承
      selftest_repository_report（「✗ 自检：知识同步提醒（继承了指到别的仓的 git 变量）：」、「✗ 自检：知识同步提醒（继承了带配置的 git 变量）：」）
  AGENT_WATCH_BREAK=repositorylimitignored  visible_count() 不看 --repository-report-lines，清单全列、不报余下条数
      selftest_repository_report（「✗ 自检：知识同步提醒（清单超过上限）：」、「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」）
  AGENT_WATCH_BREAK=repositorylineszero  take_repository_baseline() 在 --repository-report-lines 写 0 时照读照记基线，退出时照打这一节
      selftest_repository_report（「✗ 自检：知识同步提醒（写 0 不打）：」）
  AGENT_WATCH_BREAK=repositorylocalenvironmentfallbackshort  git_local_environment_variables() 取不成时交只有八个的短表（不含 GIT_CONFIG 这一组）
      selftest_repository_report（「✗ 自检：知识同步提醒（取不到 git 现列的变量时用写死的）：」）
  AGENT_WATCH_BREAK=repositorylocalenvironmentunread  dropped_git_environment() 不现取 git rev-parse --local-env-vars，只去掉写死的八个：GIT_CONFIG_PARAMETERS 这一组照继承
      selftest_repository_report（「✗ 自检：知识同步提醒（继承了带配置的 git 变量）：」）
  AGENT_WATCH_BREAK=repositorymanualcommandsjoined  command_rows() 给人抄的几条命令接在说明后面、用全角分号连成一行（原样交给 shell 跑不起来）
      selftest_repository_report（「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」、「✗ 自检：知识同步提醒（没查成给列被忽略条目的命令）：」）
  AGENT_WATCH_BREAK=repositorymanualcommandsquotedloosely  git_command_line() 每一段只在两头套一对单引号、里面的单引号不转义：仓的路径带单引号时照抄的命令跑不起来
      selftest_repository_report（「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」）
  AGENT_WATCH_BREAK=repositorymanualcommandssemicolon  command_rows() 每条命令照旧单独一行、行首照旧，行尾接一个全角分号（条数与「git -c core.quotepath=off -C 」都对得上，
      原样交给 shell 时 git 把分号当成末一个参数的一截，status --short、diff、log --oneline 那几条退非 0；只有逐条真跑才看得出）
      selftest_repository_report（「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」、「✗ 自检：知识同步提醒（没查成给列被忽略条目的命令）：」）
  AGENT_WATCH_BREAK=repositoryonalert  emit_knowledge_sync_section() 告警退出不看离上次提醒满没满，一律带这一节
      selftest_repository_report（「✗ 自检：知识同步提醒（告警退出而没满不带）：」、「✗ 自检：知识同步提醒（起来时就查不成也按时提醒）：」、
      「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」）
  AGENT_WATCH_BREAK=repositoryonfinish  run() 被看的子 agent 全部交回退 0 之前也打这一节
      selftest_repository_report（「✗ 自检：知识同步提醒（全部交回与 report 不打）：」）
  AGENT_WATCH_BREAK=repositoryrootorder  repository_root_of() 后两级对调：没给 --repository-root 时脚本所在的仓先于来源仓根 TASK_SERVER_SOURCE_REPOSITORY
      selftest_repository_report（「✗ 自检：知识同步提醒（读哪个仓的三级取法）：」）
  AGENT_WATCH_BREAK=repositoryruleblind  in_knowledge_scope() 一份知识类文件都不认：范围表一概当空，新提交、工作区里变了的与被 git 忽略的都不点名、不记摘要
      selftest_repository_report（「✗ 自检：知识同步提醒（新提交动了知识类文件）：」、「✗ 自检：知识同步提醒（点名次序）：」、「✗ 自检：知识同步提醒（工作区改了没提交）：」、
      「✗ 自检：知识同步提醒（起来时已改内容又变）：」、「✗ 自检：知识同步提醒（清单超过上限）：」、「✗ 自检：知识同步提醒（工作区路径带空格与双引号）：」、
      「✗ 自检：知识同步提醒（新提交文件名带双引号）：」、「✗ 自检：知识同步提醒（本机 git 配置不改读数）：」、「✗ 自检：知识同步提醒（范围里被 git 忽略的条目）：」、
      「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」、「✗ 自检：知识同步提醒（知识类文件改名）：」、「✗ 自检：知识同步提醒（未跟踪目录的摘要）：」、
      「✗ 自检：知识同步提醒（整次摘要超时走没查成）：」、「✗ 自检：知识同步提醒（两只之间变了的被后一只点名）：」、「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」、
      「✗ 自检：知识同步提醒（链接目标不是 UTF-8 也落得了盘）：」、「✗ 自检：知识同步提醒（路径带换行与控制字符）：」、「✗ 自检：知识同步提醒（文件名不是 UTF-8）：」、
      「✗ 自检：知识同步提醒（继承了带配置的 git 变量）：」、「✗ 自检：知识同步提醒（没查成给列被忽略条目的命令）：」、「✗ 自检：知识同步提醒（被忽略条目的标签按发生的事分）：」）
  AGENT_WATCH_BREAK=repositorysectionoff  repository_status_section() 一概是空：告警退出与定时回报都不打这一节、也不换基线
      selftest_repository_report（「✗ 自检：知识同步提醒（有新提交与改动）：」、「✗ 自检：知识同步提醒（新提交动了知识类文件）：」、「✗ 自检：知识同步提醒（点名次序）：」、
      「✗ 自检：知识同步提醒（什么都没变）：」、「✗ 自检：知识同步提醒（知识类文件没变）：」、「✗ 自检：知识同步提醒（提醒句排第一）：」、
      「✗ 自检：知识同步提醒（工作区改了没提交）：」、「✗ 自检：知识同步提醒（起来时已改内容又变）：」、「✗ 自检：知识同步提醒（范围外不点名）：」、
      「✗ 自检：知识同步提醒（清单超过上限）：」、「✗ 自检：知识同步提醒（--processes-only 有新提交与改动）：」、「✗ 自检：知识同步提醒（仓根不是仓）：」、
      「✗ 自检：知识同步提醒（git 超时）：」、「✗ 自检：知识同步提醒（git 起不来）：」、「✗ 自检：知识同步提醒（工作区路径带空格与双引号）：」、
      「✗ 自检：知识同步提醒（新提交文件名带双引号）：」、「✗ 自检：知识同步提醒（本机 git 配置不改读数）：」、「✗ 自检：知识同步提醒（范围里被 git 忽略的条目）：」、
      「✗ 自检：知识同步提醒（给人抄的命令照原样跑得起来）：」、「✗ 自检：知识同步提醒（核手上的文件给命令）：」、「✗ 自检：知识同步提醒（只暂存不点名）：」、
      「✗ 自检：知识同步提醒（只改执行位不点名）：」、「✗ 自检：知识同步提醒（知识类文件改名）：」、「✗ 自检：知识同步提醒（HEAD 往回挪）：」、
      「✗ 自检：知识同步提醒（未跟踪目录的摘要）：」、「✗ 自检：知识同步提醒（整次摘要超时走没查成）：」、「✗ 自检：知识同步提醒（落盘的基线对不上仓或读不出就重新记）：」、
      「✗ 自检：知识同步提醒（告警退出且已满带这一节）：」、「✗ 自检：知识同步提醒（打过把基线换成此刻）：」、「✗ 自检：知识同步提醒（两只之间变了的被后一只点名）：」、
      「✗ 自检：知识同步提醒（未跟踪知识类目录里有命名管道）：」、「✗ 自检：知识同步提醒（定时回报离上次提醒满了就退）：」、「✗ 自检：知识同步提醒（起来时就查不成也按时提醒）：」、
      「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」、「✗ 自检：知识同步提醒（两只读同一份只提醒一次）：」、「✗ 自检：知识同步提醒（先打出这一节再落盘）：」、
      「✗ 自检：知识同步提醒（记基线时的异常照没查成记）：」、「✗ 自检：知识同步提醒（带这一节时的异常照没查成写）：」、「✗ 自检：知识同步提醒（同一路径出两条）：」、
      「✗ 自检：知识同步提醒（路径带换行与控制字符）：」、「✗ 自检：知识同步提醒（文件名不是 UTF-8）：」、「✗ 自检：知识同步提醒（继承了带配置的 git 变量）：」、
      「✗ 自检：知识同步提醒（起来时干净、之后只改执行位不点名）：」、「✗ 自检：知识同步提醒（--processes-only 告警退出且已满带这一节）：」、
      「✗ 自检：知识同步提醒（没查成给列被忽略条目的命令）：」、「✗ 自检：知识同步提醒（被忽略条目的标签按发生的事分）：」）
  AGENT_WATCH_BREAK=repositoryshortstatus  repository_snapshot() 状态改读 git status --short（按换行切、引号原样，受 status.branch、status.showUntrackedFiles、color.status 配置影响）
      selftest_repository_report（「✗ 自检：知识同步提醒（工作区路径带空格与双引号）：」、「✗ 自检：知识同步提醒（本机 git 配置不改读数）：」、「✗ 自检：知识同步提醒（同一路径出两条）：」、
      「✗ 自检：知识同步提醒（路径带换行与控制字符）：」）
  AGENT_WATCH_BREAK=repositorystagedunread  status_counts() 不数状态码的第一位，已暂存恒为 0
      selftest_repository_report（「✗ 自检：知识同步提醒（状态码计数）：」、「✗ 自检：知识同步提醒（有新提交与改动）：」、「✗ 自检：知识同步提醒（什么都没变）：」、
      「✗ 自检：知识同步提醒（同一路径出两条）：」）
  AGENT_WATCH_BREAK=repositorystatenotread  take_repository_baseline() 不读会话目录里落盘的基线，一律此刻重新记（前一只记下的与两只之间的变动都丢了）
      selftest_repository_report（「✗ 自检：知识同步提醒（有新提交与改动）：」、「✗ 自检：知识同步提醒（新提交动了知识类文件）：」、「✗ 自检：知识同步提醒（点名次序）：」、
      「✗ 自检：知识同步提醒（工作区改了没提交）：」、「✗ 自检：知识同步提醒（起来时已改内容又变）：」、「✗ 自检：知识同步提醒（清单超过上限）：」、
      「✗ 自检：知识同步提醒（--processes-only 有新提交与改动）：」、「✗ 自检：知识同步提醒（告警退出且已满带这一节）：」、「✗ 自检：知识同步提醒（两只之间变了的被后一只点名）：」、
      「✗ 自检：知识同步提醒（落盘的基线对不上仓或读不出就重新记）：」、「✗ 自检：知识同步提醒（定时回报离上次提醒满了就退）：」、「✗ 自检：知识同步提醒（起来时就查不成也按时提醒）：」、
      「✗ 自检：知识同步提醒（之后查不成每满一次才提醒）：」、「✗ 自检：知识同步提醒（两只读同一份只提醒一次）：」、「✗ 自检：知识同步提醒（先打出这一节再落盘）：」、
      「✗ 自检：知识同步提醒（落盘的时刻不是有限数或晚于此刻就重新记）：」、「✗ 自检：知识同步提醒（--processes-only 告警退出且已满带这一节）：」）
  AGENT_WATCH_BREAK=restarthint  restart_command() 与 processes_restart_line() 的出路句印回本脚本自己的选项，不印 watch.sh 认得的写法
      selftest_in（「✗ 自检：定时回报的出路句应当是「→ 接着盯就用 run_in_background 再起一个看门狗：bash scripts/agents/watch.sh waiting」」）、
      selftest_session_watch_in（「✗ 自检：--discover 的出路句应当是不带 agent 号的 bash scripts/agents/watch.sh」）、
      selftest_repository_report（「✗ 自检：知识同步提醒（--processes-only 有新提交与改动）：」、「✗ 自检：知识同步提醒（--processes-only 告警退出且已满带这一节）：」）
  AGENT_WATCH_BREAK=resume  AgentTranscript._read() 交回之后的续做消息不撤销交回
      selftest_in（「✗ 自检：子 agent queuedresume」、「✗ 自检：子 agent continued 应当不算交回或被停」、「✗ 自检：子 agent resumedthinking 应当不算交回或被停」、
      「✗ 自检：子 agent queuedresume 应当不算交回或被停」）
      「✗ 自检：报告里应当另起一节「中途停下没交回的子 agent：被看的 3 个里 2 个」」
  AGENT_WATCH_BREAK=resumeturn  AgentTranscript._read() 结束本轮之后来的续做消息不算被叫醒
      selftest_in（「✗ 自检：子 agent resumedafterturnend」、「✗ 自检：子 agent resumedafterturnend 的状态应当是「思考中」」）
  AGENT_WATCH_BREAK=rootfree  root_free_report() 不看余量，根分区可用量低于 --root-free-gib-alert 也不报
      selftest_root_free（「✗ 自检：根分区可用 40 GiB、阈值 128 GiB 时应当报」、「✗ 自检：根分区可用量应当只读一次」、「✗ 自检：根分区可用 500 GiB、阈值 128 GiB 时不该报」、
      「✗ 自检：看门狗 report 在根分区可用 40 GiB 时」、「✗ 自检：根分区余量过低被 --ack 本机:根分区余量过低 确认之后」、「✗ 自检：看门狗 report 在根分区可用 500 GiB 时」、
      「✗ 自检：--processes-only 的 report 在根分区可用 40 GiB 时」）、
      selftest_repository_report（「✗ 自检：知识同步提醒（告警退出而没满不带）：…（--processes-only）…」、「✗ 自检：知识同步提醒（--processes-only 告警退出且已满带这一节）：」：
      这两格靠根分区余量过低造出告警叫醒）
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
  AGENT_WATCH_BREAK=settlenotification  AgentTranscript.text_turn_ended() 不看主会话记录里的任务通知，纯文字记录过了 TEXT_SETTLE_SECONDS 就判结束本轮
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
      selftest_stale_sessions（「✗ 自检：重启前旧进程留下的记录」、「✗ 自检：登记目录读不了时判不了会话在不在」）、
      selftest_repository_report（「✗ 自检：知识同步提醒（告警退出而没满不带）：…（--agents）…」、「✗ 自检：知识同步提醒（告警退出且已满带这一节）：」：这两格靠工具调用过长造出告警叫醒）
  AGENT_WATCH_BREAK=toolrevoke  AgentTranscript._read() 交回之后又调了工具就撤销交回
      selftest_in（「✗ 自检：子 agent toolafterhandback」、「✗ 自检：子 agent toolafterhandback 应当算交回或被停」）
  AGENT_WATCH_BREAK=zombie  still_running() 把僵尸当还在跑
      selftest_in（「✗ 自检：--processes-only 盯着的进程退出了」）
  AGENT_WATCH_BREAK=ackscope  ack_scope_keys() 一个键都不认：--ack 类:<告警名> 不起作用（只能按进程号、按 agent 号逐个确认）
      selftest_ack_scopes（「✗ 自检：--ack 类:根分区余量过低 应当只确认告警」、「✗ 自检：--ack 类:交回之后后台还在跑 应当只确认告警」、「✗ 自检：记进状态目录的 类: 确认应当在告警还在时留着」）
  AGENT_WATCH_BREAK=ackscopepersist  alert_keys() 不带 类: 这种键：记进状态目录的类确认在下一次整查被当成条件已消失而作废
      selftest_ack_scopes（「✗ 自检：记进状态目录的 类: 确认应当在告警还在时留着」）
  AGENT_WATCH_BREAK=unjudgedwakes  INFO_DETECTION_PREFIXES 不含 UNJUDGED_DETECTION_PREFIXES：钩子判不了而照常放行记下的检出真碰到都叫醒
      selftest_unjudged_prefixes（「✗ 自检：钩子源码里记检出用的」、「✗ 自检：UNJUDGED_DETECTION_PREFIXES 里」）
  AGENT_WATCH_BREAK=unjudgedpartsunread  hook_unjudged_prefixes() 只读钩子目录里的 .sh、不读同层的 .py 部分文件：拆成入口加部分文件的钩子，起头只写在部分文件里的抽不出来
      selftest_unjudged_prefixes（「✗ 自检：造一个拆成入口加同层部分文件的假钩子」）
工作根（草稿目录、检出记录与状态目录的默认位置）与会话临时目录根只在 .claude/hooks/lib_work_root.py 里定义，它的两个弄坏开关：
  SINGLEFS_WORK_ROOT_BREAK=new-root-unknown  工作根退回会话临时目录根：派发提示里工作根下的草稿目录认不出
      selftest_scratch_mentions（「✗ 自检：派发提示点名工作根下的草稿目录」两格、「✗ 自检：派发提示点名会话临时目录根下的草稿目录不认」（工作根就是会话临时目录根，那边的目录认得出了）、「✗ 自检：派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告…」）、
      selftest_session_watch_in（「✗ 自检：子 agent progressstale」、「✗ 自检：整点询问应当附上 progress.md 的末几行」）、
      「✗ 自检：进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」、
  SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted  草稿根另认会话临时目录根：派发提示里会话临时目录根下的草稿目录认得出
      selftest_scratch_mentions（「✗ 自检：派发提示点名会话临时目录根下的草稿目录不认」）
「交回」送没送达只在同目录 subagent_transcript.py 里认，它的开关（开关表在它的文件头）：
  SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered  被交回闸拒的交回也算送达
      selftest_in（「✗ 自检：子 agent refusedonly 应当不算交回或被停」）
会话记录目录（<家目录>/.claude/projects）只在同目录 lib_session_directory.py 里取（统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY 定家目录，不经 $HOME），它的开关（开关表在它的文件头）：
  SESSION_DIRECTORY_BREAK=home-from-environment  设了 TASK_SERVER_SOURCE_REPOSITORY 也仍取 HOME
      selftest_projects_root（「✗ 自检：按 id 找会话记录取来源仓根的会话目录：」）

分在几份文件里：本文件留命令行、整份自证的编排（selftest、selftest_in）与最外层的开关守卫（refuse_break_outside_selftest），其余按块放在同目录三份模块里，测哪一块的自证用例跟着那一块：
  agent_watch_transcripts.py  会话记录与后台任务的读法、交回之后后台还在跑、副本还在、草稿根里的编译目录与仓副本、根分区余量、进程表，测它们的自证用例与造会话记录的工具
  agent_watch_repository.py   知识同步提醒的仓现状（git 状态、内容摘要、基线落盘与读回、定时回报一节的写法），测它的自证用例与造小仓的工具
  agent_watch_alerts.py       单个子 agent 的告警、只看进程的一路、整份报告、hook 检出的读法、--ack 的读写与剪除、整查一轮 run，测它们的自证用例
本文件把三份导入并再导出（交回闸、完工清理闸、move-directory-in-use-check.py、clean-session-scratch.py 按路径载入本文件之后按名字取的函数与常量都还在这里取得到）；
测子进程那几格要起的入口是本文件，拆出的模块里读它的路径是 agent_watch_transcripts.py 的 ENTRY_SCRIPT_PATH。
弄坏开关表仍写在本文件。AGENT_WATCH_BREAK 各条开关的代码在哪份（拿开关名带引号的字面在那份里搜）：
  本文件：keepscratch（selftest）、cost（selftest_in）
  agent_watch_transcripts.py：
    amplificationhome、backgroundwait、display、finished、handback、heredoc、inquirydelivered、inquirysent、keptcopy、keptcopyage、keptcopyexists、leftover、leftovergrace、leftovernotified、movedbackground、orphan、resume、resumeturn、rootfree、scratchhog、scratchhogage、scratchhogtag、scratchhogsessionroot、settle、settlenotification、stalepending、stalesession、stalesessionrestart、taskstop、toolrevoke
  agent_watch_repository.py：
    knowledgecheckcommandomitted、knowledgebaselineexceptionescapes、knowledgecodechangenamed、knowledgedeadlinefromstart、knowledgedigestblind、knowledgedigestoutofscope、knowledgedigestuntimed、knowledgedirectorydigestblind、knowledgeduplicatepathmerged、knowledgefilemodecounted、knowledgefiletypeunchecked、knowledgegitoutputreplaced、knowledgeignoredblind、knowledgeignoredcommandomitted、knowledgeignoredtagflat、knowledgeorderflat、knowledgepathunescaped、knowledgeremindedatunchecked、knowledgereminderlast、knowledgerenamesplitoff、knowledgesavedbeforeprint、knowledgesectionexceptionescapes、knowledgestaletemporarykept、knowledgestatenotreread、knowledgestaterepositoryunchecked、knowledgestatestrictencoding、knowledgetagsource、knowledgeunreadablenotadvanced、knowledgeunreadablenotsaved、knowledgeworktreeblind、repositorybackwardsunsaid、repositorybaselinekept、repositorychangesblind、repositorycommitsblind、repositorydiffnewline、repositorydueonalertignored、repositoryfailuresilent、repositoryheadomitted、repositoryinheritedcommondir、repositorylimitignored、repositorylineszero、repositorylocalenvironmentfallbackshort、repositorylocalenvironmentunread、repositorymanualcommandsjoined、repositorymanualcommandsquotedloosely、repositorymanualcommandssemicolon、repositoryonalert、repositoryrootorder、repositoryruleblind、repositorysectionoff、repositoryshortstatus、repositorystagedunread、repositorystatenotread
  agent_watch_alerts.py：
    ack、acksneverpruned、acksunlocked、acksnotpersisted、banned、bannedrefused、bannedwindow、context、contextzero、deadline、detections、detectiontype、discoverfallback、discovernone、handoveranytype、holders、idle、infoalertwakes、infowakes、inquiry、inquiryack、inquiryended、inquiryreset、staleselftestscratch、leftoverack、leftoverexit、loop、loopnoise、midwaymerged、nosingleinstance、nowake、otheragent、outcomeignored、ownrefusal、processexit、processesalertrestartomitted、progressanypath、progressblankkept、progressnote、progressstale、progresstailhead、progresstailreversed、refusedwakes、repeat、repositoryonfinish、restarthint、scopeunlisted、timeout、zombie、ackscope、ackscopepersist、unjudgedwakes、unjudgedpartsunread
SINGLEFS_WORK_ROOT_BREAK、SUBAGENT_TRANSCRIPT_BREAK、SESSION_DIRECTORY_BREAK 三族的开关代码在各自的库里（lib_work_root.py、subagent_transcript.py、lib_session_directory.py），这里只有测它们的自证用例。
"""
import argparse
import importlib.util
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'hooks'))
preflight_sys.path.insert(0, preflight_os.path.dirname(preflight_os.path.realpath(__file__)))
import lib_work_root as work_roots  # noqa: E402  工作根与会话临时目录根的唯一定义
import subagent_transcript as transcripts  # noqa: E402  「交回」（送达）的唯一定义，续做闸与 agent-cost.py 取的是同一份
from agent_watch_transcripts import (
    AgentTranscript, BROKEN_DETECTION, KEPT_COPY_ALERT, ROOT_FREE_ALERT, ROOT_FREE_BYTES_VARIABLE, ROOT_FREE_GIB_ALERT_DEFAULT,
    ROOT_FREE_SUBJECT, SCRATCH_ROOTS_ENV, SELFTEST_SECTION_OFF, SESSION_REGISTRY_ENV, bash_result, bash_use, command_for_display,
    find_transcript, handback_records, iso_minutes_ago, process_arguments, record_at, selftest_leftover_background, selftest_root_free,
    selftest_scratch_hogs, selftest_stale_sessions, session_task_events, wait_for_selftest_ready, wait_until, write_session_stops,
    write_transcript
)
from agent_watch_repository import REPOSITORY_REPORT_LINES_DEFAULT, build_selftest_repository, selftest_repository_report
from agent_watch_alerts import (
    HANDOVER_ALERT, HOOKS_DIRECTORY, ROUTINE_INQUIRY_NEXT_STEP, SELFTEST_READY_VARIABLE, STATE_ROOT_VARIABLE, STOPPED_MIDWAY_ALERT,
    agent_alerts, background_task_processes, build_report, run, selftest_ack_concurrency, selftest_ack_scopes, selftest_kept_copies,
    selftest_projects_root, selftest_scratch_mentions, selftest_session_watch, selftest_stale_scratch_cleanup, selftest_unjudged_prefixes
)
# 再导出：别处按名字取这份脚本里的函数与常量（交回闸、完工清理闸、move-directory-in-use-check.py、clean-session-scratch.py、测试），三份模块里定义的名字一个不少
from agent_watch_transcripts import *  # noqa: F401,F403
from agent_watch_repository import *  # noqa: F401,F403
from agent_watch_alerts import *  # noqa: F401,F403


DEFAULT_DETECTIONS = work_roots.default_detections_path()

# 定时回报里「知识同步提醒」一节（文件头「知识同步提醒」）
REPOSITORY_ROOT_VARIABLE = "AGENT_WATCH_REPOSITORY_ROOT"   # 环境变量只给 --selftest 起的看门狗子进程用：--repository-root 的缺省，自证不碰真仓

# 自证的临时目录由同一处的共用库建（通过、判红、抛异常三条路上都删），那三格也由它判
SELFTEST_SCRATCH_LIBRARY_PATH = os.path.join(HOOKS_DIRECTORY, "lib_selftest_scratch.py")
SELFTEST_ROLE_VARIABLE = "AGENT_WATCH_SELFTEST_ROLE"   # 自证拉起自己的子进程走判红、抛异常那两条路时设的角色


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
    # 本仓真实的 git 状态也不进自证：这份自证起的看门狗子进程，「知识同步提醒」那一节一律读临时目录里造的小仓（要改仓的几格在 selftest_repository_report 里另给）
    os.environ[REPOSITORY_ROOT_VARIABLE] = build_selftest_repository(os.path.join(work, "default-repository"))
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
    # （样本次序：前台命令超时转后台、交回、结束本轮、完成通知、grep 一次、结束本轮）。
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
    # 那条调用已经死了，不许再按「工具调用过长」报。
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
    with open(session + ".jsonl", "a", encoding="utf-8") as handle:   # 主会话记录里 SendMessage 结果的形态照主会话记录里的原样
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
        return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", *watch_arguments, *SELFTEST_SECTION_OFF, "--session-dir", session, "--process-root-pid", "0"],
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
        watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", *SELFTEST_SECTION_OFF, "--agents", "healthy", "--session-dir", session,
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
    repository_report_checks = selftest_repository_report(work, failures, watch_runs, probed_processes)
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
          f"没起过后台任务、结束本轮没交回的报「{STOPPED_MIDWAY_ALERT}」并在报告里另起一节带重派命令；"
          f"告警退 3 与定时回报退 4（--agents 与 --processes-only 两路）离上次提醒满 --max-minutes 时带一节「知识同步提醒」（提醒句排第一、逐份点名变过的知识类文件与来源——"
          f"新提交动到的、工作区改了没提交的与被 git 忽略而摘要变了的取并集，路径带空格与引号的照认、不受本机 git 配置影响，范围内基线时已记摘要的只比摘要、"
          f"里程碑与决策索引与欠账表排最前，之后是分支与 HEAD、基线从哪来、基线之后的新提交、git status 的计数、基线之后变了的别的路径、末尾的出路；"
          f"清单超过上限的截并报余下条数、给人抄的命令一行一条照原样跑得起来，查不成与算摘要超时写「没查成」、命名管道不卡住，写 0 不打；"
          f"基线与上次提醒的时刻落盘到会话目录，下一只接着比，退出时重读、几只只提醒一次，先打出再落盘，打过换成此刻、没查成也推进时刻，定时回报离上次提醒满了就退；"
          f"同一路径两条、带换行与不是 UTF-8 的路径、带配置的 git 变量、只改执行位、进出 git 忽略都照对，异常照没查成写、退出码不变），"
          f"没满的告警退出、全部交回退 0、report 不打（{repository_report_checks} 格）"
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
    parser.add_argument("--max-minutes", type=float, default=60,
                        help="定时回报：没有告警也每隔这么久退出一次叫醒主 agent（离上次知识同步提醒满这么久的提前退），默认 60 分钟")
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
    parser.add_argument("--repository-report-lines", type=int, default=REPOSITORY_REPORT_LINES_DEFAULT,
                        help="定时回报里「知识同步提醒」一节：点名的知识类文件、新提交、起来之后变了的别的路径这几张清单各最多列这么多条，余下的报条数；默认 20，写 0 不打这一节、也不记起来时的仓")
    parser.add_argument("--repository-root", default=os.environ.get(REPOSITORY_ROOT_VARIABLE),
                        help="「知识同步提醒」读哪一个仓；缺省是统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY，没有这个变量取本脚本所在的仓（给自证指到临时仓用）")
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
