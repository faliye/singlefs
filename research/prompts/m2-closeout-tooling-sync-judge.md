# m2-closeout-tooling 阶段同步：逐行判定（回扫员 sweep，段二）

阶段名：`m2-closeout-tooling`；基准 `b95e70b`，结束在工作区（主仓工作区里这一阶段自己的 17 个触发文件）。
候选表：`research/prompts/m2-closeout-tooling-sync-candidates.tsv`（57 行，分给我全部组 F1/F3/F5/F6/F8/F9/F10/F12/F14）。

判的是**主仓工作区**（`/home/fy5090/code/singlefs`）里那一行现在的样子，不是候选表冻结快照（scoped-repo）里的原文；
候选表的 file:line 只用来定位，主仓里别的会话改过的文件（`.claude/kb/checks-owed.md`、`.claude/kb/experiments.md`、
`.claude/kb/experiments/155-*.md`）行号已挪，按原文 grep 定位后在下表「载体」列括注实际行号。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F1 | `.claude/agent-common.md:42` | 要改（改定义，并进定义那一轮） | 现文「项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断」已不真：`bash-command-detector.sh` 现在只检出没超时的等待循环与 `run_in_background` 自己放后台两种（现读该脚本头部注释「按模式找进程…不在这里判：上游 SOP 的 `pattern-process-guard.sh` 在执行前拒绝」确认）。改后：「项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、`run_in_background` 起的命令又自己放后台这类命令交给主 agent 判断；按模式找进程的命令由上游 SOP 钩子 `pattern-process-guard.sh` 在执行前直接拒绝执行，不进这个 hook。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。」 |
| F1 | `.claude/main-agent.md:19` | 要改（改定义，并进定义那一轮） | 现文把「按模式找进程」列为看门狗「每 15 秒读一次 hook 的检出记录」能发现的一项；现读 `research/scripts/agent-watch.py` 第 737 行注释「按模式找进程由上游钩子在执行前拒绝，不写检出记录，这里读不到它」，说明按模式找进程已经不再走 hook 检出记录这条通道，而是看门狗另外扫这个 agent 自己会话记录里发出过的命令、命中就报「禁用命令」（`agent_alerts()` 第 483–491 行），且入口已拒绝的不再重报（第 488 行 `output.startswith("PreToolUse:Bash hook error")` 时跳过）。改后（与 F8、F9 两行合并成一句，三件事同一处改）：「…发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、用了按模式找进程这类禁用命令（这类命令现在先被上游钩子 `pattern-process-guard.sh` 在执行前拒绝，检出记录里读不到，看门狗改成直接扫这个 agent 自己会话记录里发出过的命令，入口已经拒绝的不再重报）、上下文过大（默认 60 万，只提示不叫停，可用 `--ack` 确认）、无动静超过 10 分钟（正在等自己起的后台任务、或续做消息刚落进记录的头几秒不算）、结束本轮且手里没有还在等的后台任务、进程写的文件不动，或读到本会话里被盯的这个子 agent 自己触发的 hook 检出（主 agent 自己被拒的写只进报告不叫醒，别的子 agent 的检出交给盯它的那个看门狗），就退出并叫醒主 agent；…」 |
| F1 | `records/2026-09-16-subagent拆分提案.md:526` | 事件句不改 | 原话「新脚本 `research/scripts/agent-watch.py`：读会话记录报…六种告警…自检接进门禁 47 号」是当天（2026-09-17 前后）新建 `agent-watch.py` 那一次的交付记录，说的是那一次做成了什么，不是今天有几种告警的现状句；不受本阶段把「按模式找进程」移出 hook 检出这件事影响。 |
| F1 | `records/2026-09-16-subagent拆分提案.md:527` | 事件句不改 | 原话「新 hook `.claude/hooks/bash-command-detector.sh`…检出没超时的 until/while 加 sleep 与按模式找进程的命令…当天先后做过两版…改成现在这版」逐字是当天两次改版的经过记录（"当天""改成现在这版"是叙事标记），记的是那次怎么定型，不是今天 hook 判几种。 |
| F1 | `records/2026-09-16-subagent拆分提案.md:540` | 事件句不改 | 原话「`research/scripts/agent-watch.py`：hook 只检出『没超时的等待循环』的记录不再立刻叫醒…自检加一格，`AGENT_WATCH_BREAK=loopnoise` 判红」是一次具体改动的交付记录（配一次具体的自检项），事件句。 |
| F1 | `records/2026-09-16-subagent拆分提案.md:617` | 事件句不改 | 原话「`.claude/hooks/bash-command-detector.sh` 按整条命令串匹配『按模式找进程』这一类。实测撞到：…」，"实测撞到"标明这是 2026-09-18 一次具体误报的复盘，不是现状声明。 |
| F1 | `records/2026-09-16-subagent拆分提案.md:619` | 事件句不改 | 原话「2026-09-19 取前一种：前两种检出（按模式找进程、没超时的等待循环）也与第三种一样，先剥掉喂给非 shell 命令的 heredoc 正文…自检加四格」，带日期的一次代码改动记录，事件句。 |
| F3 | `.claude/kb/checks-owed.md:356` | 要改 | 主仓工作区里 C384 这一行现在落在第 348 行（行号因别的会话编辑挪动，原文与候选表一致），仍在「欠账」表里（第 370 行才是「### 已还清」），而 C384 的前置「上游 shell-lint 先修单行函数的解析」已满足：SOP `VERSION` 现为 0.0.52，`.claude/singlefs-ai-sop/scripts/shell-lint.sh` 已认「定义与收尾写在同一行的函数，函数体就是这一行」；`research/scripts/{e129-thin-neighbour,e129-thin-rmw,e72-devtable-probe,stripe-map-probe}.sh` 的 `S() {` 已展开成多行；`.claude/gate.d/73-research-gate-lint.sh` 现读确认已对 `research/scripts/` 与 `.claude/hooks/` 都跑 shell-lint。改后：把这一行从欠账表删掉，移进「### 已还清」表：`| C384 | 研究脚本与 hook 不在 shell-lint 的射程里 | 上游 shell-lint 已修好单行函数解析假红（SOP 同步到 0.0.52）；四个研究脚本单行函数已展开成多行；门禁 73 号现同时把 research/scripts/ 与 .claude/hooks/ 交给 shell-lint | 2026-09-19 |` |
| F5 | `records/2026-09-16-subagent拆分提案.md:527` | 事件句不改 | 同上一条：这一行是 bash-command-detector.sh 两版改法的经过记录，末句「注册与自检接进门禁 63 号（样本同步）」是那一次登记的动作，不是「门禁 63 号现在有几项」的现状声明；门禁 63 号新增第⑦⑨项不推翻它当时确实被接进 63 号这件事。 |
| F5 | `records/2026-09-16-subagent拆分提案.md:532` | 事件句不改 | 原话「新 hook `.claude/hooks/runner-dispatch-guard.sh`…注册与自检接进门禁 63 号（样本同步）…当天在主会话里实派一次缺岔路行的执行员，被当场拒绝」，"当天""实派一次"是事件记录，不因 63 号后来插入新项而失真。 |
| F5 | `records/2026-09-16-subagent拆分提案.md:534` | 事件句不改 | 原话「用户看过之后定『确认然后推广』…门禁 63 号加第 ⑦ 条查每份定义有 `omitClaudeMd: true` 与『依据：』一行（样本同步）」用的是「加」（动作动词），记的是那次把 omitClaudeMd 检查加进 63 号、当时占第 ⑦ 位这件事；现读 `.claude/gate.d/63-agent-write-scope.sh` 第 12 行，omitClaudeMd 检查现在是第 ⑧ 条（本阶段插入了新的第 ⑦、⑨ 条），但这行说的是「那一次加了什么」，不是「今天第几条」，换成新编号反而不是当时发生的事。 |
| F5 | `records/2026-09-16-subagent拆分提案.md:650` | 事件句不改 | 原话「门禁 63 号第 ⑦ 条与它的两套样本跟着查『开工先读：』一行…一共 18 份文件、66 处定点替换」，同样是「跟着查」这一次改动的交付记录（清理经过），"第 ⑦ 条"记的是那次改的是哪一条，不是今天的编号。 |
| F5 | `records/2026-09-16-subagent拆分提案.md:700` | 事件句不改 | 原话「同日退回（用户定案）：收尾弹窗里用户选『退回 .claude/ 原位置』…门禁 63 号的跳过名单与绿样本那份 `agent-common.md` 去掉」，"同日退回"标题本身即事件框架，记的是那一次弹窗决定与随之的改动。 |
| F5 | `records/2026-09-17-已分配口径三方与两个实验.md:162` | 事件句不改 | 原话「已做：`.claude/hooks/runner-dispatch-guard.sh`…自检接进门禁 63 号；实派一次被当场拒绝，2026-09-18 第一次在真派发里放行」，"已做"+带日期的"第一次"是完成度记录，不受后续 63 号插入新项影响。 |
| F5 | `records/2026-09-17-已分配口径三方与两个实验.md:164` | 事件句不改 | 原话「已做：两次试点产出都与现定义逐项一致…16 份定义开了 `omitClaudeMd`，共用约束加『规则怎么读』，门禁 63 号查每份定义有 `omitClaudeMd: true`」，"已做"框定为完成报告；它只说「63 号查这一项」，不是「63 号一共几项」，与新增的第⑦⑨项不矛盾。 |
| F6 | `.claude/rules/three-way-inference.md:256` | 事件句不改 | 这一行本身现读就是「新事实」——`.claude/rules/three-way-inference.md` 是本阶段 17 个触发文件之一，正文「撞了限额、被停或报错的腿，新开一条接着做，不续原来那条」已经是现在的规则，逐句核对与新事实完全一致（含 `agent-handover.py`、`continuation-guard.sh` 都已现读代码确认存在）。整段以「用户 2026-09-19 定：『……』」起句，是把那次决定原样落成规则条文，判定按事件的锚点处理：不改。 |
| F6 | `.claude/rules/three-way-inference.md:257` | 事件句不改 | 原话「实测（2026-09-19，E155 第二次跑的执行员）：它被续派三次…停掉之后抽的交接摘要 731 行，新执行员从 1.3 万上下文起步」，带日期的"实测"是给上一条规则举的一次具体观测证据，不是现状声明。 |
| F6 | `records/2026-09-16-subagent拆分提案.md:59` | 要人看 | 原话「撞限额时主 agent 只能续到自己派出的那一层」是 2026-09-16 立项当天『调度平铺、不嵌套』三条理由之一，前提是「撞限额用续、续只够得到直接派出那一层」；本阶段把撞限额（含被停/报错）的处理换成「新开一条 + `agent-handover.py --agent <id>` 全机按 id 在 `~/.claude/projects/*/*/subagents/` 搜会话记录抽摘要」，不再依赖续，理论上不再受「只派出那一层」限制；但主 agent 能不能拿到嵌套派发出的下层 agent 的 id 本身没有变化（这与「写范围闸只挂在直接派出的那一层」是同一个可见性问题），这条理由是彻底站不住、还是换了个说法继续成立，需要人现查嵌套场景下 agent id 传递的实际路径才能判定，我判不下去。 |
| F6 | `records/2026-09-16-subagent拆分提案.md:153` | 要改 | 原话「限额：撞周限额续同一个 agent（它的上下文还在），撞单次输出上限重派，照 `three-way-inference.md` 现有两条」用「现有」直接断言另一份文件的当前内容，而 `three-way-inference.md` 第 256 行现在只有一条统一规则（撞限额、被停或报错都新开一条，不再区分周限额续/单次输出重派两条）。改后：「限额：撞了限额、被停或报错的腿一律新开一条接着做（不续原来那条），从被中断腿的会话记录里用 `agent-handover.py` 抽交接摘要给新腿，照 `three-way-inference.md` 现有规则。」 |
| F6 | `records/2026-09-16-subagent拆分提案.md:784` | 事件句不改 | 原话「这一段两次撞周限额（重置在 18:40 与 23:50 UTC），被卡住的子 agent 都等重置之后用续做（SendMessage）从原上下文接着做完，没有重派…规则照做有效，不改」是 2026-09-19 三十四节「各 agent、规则与门禁的评估」里对 2026-09-18～19 那一段具体两次撞限额事件的复盘判断，评估当时"规则照做有效"，属于那次评估的结论，不是对今天规则内容的断言；本阶段晚些时候才把规则改成新开一条，不推翻这条复盘。 |
| F8 | `.claude/main-agent.md:19` | 要改（改定义，并进定义那一轮） | 现文「10 分钟无动静…就退出并叫醒主 agent」没有例外，而现读 `research/scripts/agent-watch.py` 第 456–459 行，「无动静」判定已加 `not transcript.waiting_on_its_own_background_tasks()` 条件——正在等自己起的后台任务时不报无动静；第 262、268 行注释确认续做消息落进会话记录之后的头几秒按「思考中」算，不再误报「结束本轮却不会醒」。这两条例外现文都没有。改后句子并入 F1 行给出的同一处完整改写（见上）：「…无动静超过 10 分钟（正在等自己起的后台任务、或续做消息刚落进记录的头几秒不算）…」。 |
| F8 | `records/2026-09-16-subagent拆分提案.md:526` | 事件句不改 | 同 F1：「新脚本 `research/scripts/agent-watch.py`…六种告警（…10 分钟无动静…）」是新建 `agent-watch.py` 那次的交付记录，不是「无动静判定今天有没有例外」的现状句。 |
| F8 | `records/2026-09-16-subagent拆分提案.md:601` | 事件句不改 | 原话「看门狗在这一阶段起了 12 次以上，报警 3 次：一次『无动静』（就是上面那 17 分钟）…之后用 `--process-stale-minutes 90` 重起」是对"这一阶段"（早于本阶段）具体运行次数与报警次数的统计复盘，带具体计数与时长，是事件记录。 |
| F8 | `records/2026-09-16-subagent拆分提案.md:755` | 事件句不改 | 原话「子 agent 的会话记录在被停之后一个字都没写…`agent-watch.py` 只认子 agent 会话记录里的 `[Request interrupted`，于是一直当它还活着…报『无动静』、退出码 3…那是一次误报」，2026-09-18 一次具体误报事件的复盘（三十三节标题带日期），不是现状声明。 |
| F9 | `.claude/main-agent.md:19` | 要改（改定义，并进定义那一轮） | 现文「hook 检出本会话的问题命令，就退出并叫醒主 agent」未区分谁的检出、也没排除已拒绝的命令，而现读 `research/scripts/agent-watch.py` 第 731–743 行 `read_detections()`：① 检出属于「同一会话里别的子 agent」的（`watched_agent_ids` 不含该 `agent_id`）不产生告警，归它自己的看门狗处理（第 725–726 行注释）；② `entry.get("agent_type") == "主 agent" and refused` 时（主 agent 自己的写被拒）只进 `notes`（报告），不产生 `alerts`（叫醒）（第 738–741 行）。改后句子并入 F1 行给出的同一处完整改写（见上）：「…或读到本会话里被盯的这个子 agent 自己触发的 hook 检出（主 agent 自己被拒的写只进报告不叫醒，别的子 agent 的检出交给盯它的那个看门狗）…」。 |
| F9 | `records/2026-09-16-subagent拆分提案.md:526` | 事件句不改 | 同 F1/F8：新建 `agent-watch.py` 那次的交付记录，不是「谁的检出会叫醒谁」的现状句。 |
| F9 | `records/2026-09-16-subagent拆分提案.md:527` | 事件句不改 | 「看门狗每 15 秒读一次，读到本会话的检出就叫醒主 agent。当天先后做过两版…改成现在这版」是那天两次改版的经过记录，"当天""现在这版"标记为叙事时点，不是本阶段之后的现状。 |
| F9 | `records/2026-09-16-subagent拆分提案.md:538` | 事件句不改 | 原话「合成 `.claude/hooks/write-guard.sh`…拒绝时同时写进检出记录，看门狗读到就叫醒主 agent…主会话里实测整份覆盖一个未跟踪文件被拒、文件原样，看门狗 15 秒内读到检出并叫醒主 agent」——"实测"框定的是那次自建 hook 时跑的一次验证，记的是那次的观测结果（当时主 agent 自己的写被拒确实叫醒了它），不是「今天还会不会这样」的现状断言。 |
| F9 | `records/2026-09-16-subagent拆分提案.md:540` | 事件句不改 | 同 F1：「hook 只检出『没超时的等待循环』的记录不再立刻叫醒…自检加一格」是一次具体改动记录。 |
| F9 | `records/2026-09-16-subagent拆分提案.md:735` | 不相干 | 原话「① Bash 检出 hook 加一条『子 agent 的前台命令超时超过 240 秒』记进检出记录，让看门狗叫醒主 agent」提到「叫醒主 agent」，但说的是一条**还没做**的改法提议（前台命令超时检测），现读 `.claude/hooks/bash-command-detector.sh` 只检出两种（没超时等待循环、`run_in_background` 自放后台），确认这条提议至今仍未实现，跟 F9 要改的「谁的检出该不该叫醒」是两件事，不受这次改动影响。 |
| F9 | `records/2026-09-17-已分配口径三方与两个实验.md:167` | 事件句不改 | 原话「已做：`research/scripts/agent-watch.py`（…有告警叫醒主 agent…）…当天两次按用户纠正改过」，"已做"+"当天"框定为完成报告；它只笼统说"有告警叫醒主 agent"，这句今天仍然为真（告警确实还会叫醒主 agent），F9 只是新增了几类不算告警的排除项，不推翻这句概括。 |
| F10 | `.claude/main-agent.md:19` | 不相干 | 这一行提到「结束本轮而手里没有在跑的后台任务」，说的是「无动静/结束本轮却不会醒」这类**告警触发条件**；F10 说的是**互补情形**——子 agent 结束本轮、正在等自己的后台任务时，看门狗报告里「进程」一节怎么列这些进程（现读 `background_task_processes()` 函数）。两者是一句话里字面相邻但语义相反的两种状态，F10 改的报告措辞不在这一行里。 |
| F10 | `.claude/main-agent.md:29` | 不相干 | 原话讲「交回按子 agent 的 SubagentHandback 消息判」以及三种 completed 通知（等后台/交回/结束本轮不会醒）怎么读，说的是**主 agent 怎么解读通知**，不是看门狗报告里进程一节的列法（跑满时长门槛的叶子进程 vs 开着输出文件的最上层进程、累计 CPU vs 此刻占几个核），F10 改的是后者。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:569` | 不相干 | 原话「看门狗把『只结束本轮、还在等后台任务』的子 agent 当成已结束，会提前退出并报『全部结束』…状态按交回工具（SubagentHandback）成功返回判」，讲的是 agent **状态分类**（本轮结束没交回/交回后被续做）的一次修复，不涉及进程一节怎么报告进程的格式。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:575` | 不相干 | 原话「结束本轮在等后台任务的探针收到消息后多一次调用，读缓存 120,539、新写 93」是 `cache-keepalive.sh` 续缓存效果的一次测量，跟进程报告格式无关。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:581` | 不相干 | 原话「唤起失败（后台任务跑完而通知卡在队列里）没有检查…」讲的是通知唤起失败这条没做的检查，与 F10 的进程报告格式无关。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:751` | 不相干 | 原话讲禁用命令 heredoc 误报与「已交回」在续做消息到达头几秒被误判两个 bug 的修法，「后台任务的完成通知（origin 为 task-notification）不算」说的是判断「已交回」状态的口径，不涉及进程一节的列法。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:753` | 不相干 | 这是三十三节标题「TaskStop 停掉在等后台任务的子 agent，看门狗认不出『被停』」，讲的是被停状态识别，不是进程报告格式。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:759` | 不相干 | 原话「看门狗判『被停』只看子 agent 会话记录…已改：`research/scripts/agent-watch.py` 另读主会话记录…判被停」讲的是「被停」判定逻辑的一次修复，与进程一节的列法无关。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:760` | 不相干 | 原话讲 completed 通知里"waiting on its own background work"这句话怎么读、`.claude/main-agent.md`「交回怎么读」那一节何时写入，是通知解读，不是进程报告格式。 |
| F10 | `records/2026-09-16-subagent拆分提案.md:761` | 不相干 | 原话讲 `run_in_background` 命令里又带 `nohup … & disown` 导致后台任务当场结束、完成通知脱钩的一次修复，属于自放后台检测（F1 的②），与进程一节列法无关。 |
| F10 | `records/2026-09-17-已分配口径三方与两个实验.md:170` | 不相干 | 原话讲「起了后台任务与子 agent 要定时复检、检出与处置分开、不强制结束」这条规则写进上游 SOP 三份规则文件的完成情况，是写进上游 SOP 这件事，与看门狗报告里进程一节怎么列进程无关。 |
| F12 | `.claude/kb/checks-owed.md:382` | 不相干 | 主仓工作区里 C320 这一行现在落在第 388 行（行号因别的会话编辑挪动，已在「### 已还清」表里，原文与候选表一致）：原话「自证三种形态：带进来判红、没带进来是绿、跨文件同名不误报（`--selftest`，门禁 47 号跑）」只说这个自证归门禁 47 号跑，没有说门禁 47 号一共跑几份；现读 `.claude/gate.d/47-research-script-selftests.sh` 第 4–5 行，门禁 47 号新增第十六份 `test-environment-check.py --selftest` 之后，这条自证依旧照跑，这句话仍然为真。 |
| F12 | `records/2026-09-16-subagent拆分提案.md:526` | 事件句不改 | 同 F1/F8/F9：新建 `agent-watch.py` 那次的交付记录，末句「自检接进门禁 47 号」记的是那次登记这件事，不是「47 号今天有几份」。 |
| F12 | `records/2026-09-16-subagent拆分提案.md:568` | 事件句不改 | 原话「新脚本 `research/scripts/cache-keepalive.sh`…自检三项…接进门禁 47 号」是新建这个脚本那次的交付记录，同上，不受新增第十六份自证影响。 |
| F12 | `records/2026-09-16-subagent拆分提案.md:750` | 事件句不改 | 原话「改了：判禁用命令之前去掉 heredoc 正文…自检加 `bannedheredoc`、`bannedafterheredoc`…；门禁 47 号绿」是一次具体修复之后跑绿的记录，"改了"标记为事件，不是「47 号有几份」的现状声明。 |
| F12 | `records/2026-09-18-回扫员可靠性.md:22` | 不相干 | 原话「阳性对照：`--benchmark` 用冻结的盲写事实表…工具罩住的不许少于 20 处；门禁 47 号每次跑」只说 `--benchmark` 这一项归门禁 47 号跑，不涉及总数，新增 test-environment-check.py 之后这句依旧成立。 |
| F14 | `.claude/kb/experiments.md:173` | 不相干 | 主仓工作区里这一行现在落在第 175 行（行号因别的会话编辑挪动），现读该行，已经写着「部分已跑（第二次跑第一、二段；第一次 2026-09-17 全部四段…第二次 2026-09-19 第一段补建 K9′…第二段补建 R8 组提交、第 7 行主表，52 单测 / 34 条变异 33 抓 1 等价…）」，比 F14 新事实要求的还完整（本阶段与其他会话已把它更新到位），不含候选表冻结快照里「只到 2026-09-17」的旧状态，没有需要为 F14 再改的内容。 |
| F14 | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:1` | 不相干 | 同上，现读标题行已写「—— 部分已跑（第二次跑第一、二段…第二次 2026-09-19 第一段补建…第二段补建 R8 组提交、第 7 行主表…」，已经比 F14 的新事实更完整，无需再改。 |
| F14 | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:21` | 不相干 | 原话「`cd research && bash scripts/replay.sh E155`」是**第一次跑**（stage4）的复跑命令，`replay.sh` 里 `E155` 那一行没变、仍指向 `e155-fsync-write-volume-2026-09-17-stage4.out`（现读 `research/scripts/replay.sh` 第 171 行确认），这句话只讲第一次跑，跟新增的 `E155R2` 登记是两个不同的键，不受影响。 |
| F14 | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:24` | 不相干 | 原话「报『字节一致』（`replay.sh` 里 E155 那一行已指到 stage4）」同上，说的是 `E155`（第一次跑）这一行指到 stage4，现读 `replay.sh` 第 171 行仍然如此，不受新增 `E155R2` 行影响。 |
| F14 | `.claude/kb/milestone/02-second-txn.md:266` | 不相干 | 原话讲增补 1 第 1、2、3 件的交回情况与 E155「第一、二段」（第一次跑内部的段落）已跑，全文没有提到 `replay.sh` 的登记覆盖范围，跟 F14「replay.sh 只登记到第一次跑」这件事不是同一回事。 |
| F14 | `.claude/kb/milestone/02-second-txn.md:347` | 不相干 | 原话讲记账树「只保留最近 K 代」条款与实现不一致这笔欠账，`E155` 只是发现问题的署名，与 `replay.sh` 的登记范围无关。 |
| F14 | `.claude/rules/three-way-inference.md:257` | 事件句不改 | 同 F6：「实测（2026-09-19，E155 第二次跑的执行员）：它被续派三次…」是给撞限额新规则举的一次具体观测，不是 replay.sh 登记范围的现状句。 |
| F14 | `records/2026-09-16-subagent拆分提案.md:558` | 不相干 | 原话讲两句进度句被推翻后没人回头核、由此新立 `sweep` 第四种活（阶段同步）这件事，E155 只是举例提到（"续派闸 17:40 UTC 放行了 E155 第一、二段的执行员"），跟 replay.sh 的登记范围无关。 |
| F14 | `records/2026-09-16-subagent拆分提案.md:629` | 不相干 | 原话讲「重复核实的成本」这一类问题，E155 执行员只是四个举例之一（"同一个 agent 两次交回、数不一样"），跟 replay.sh 的登记范围无关。 |
| F14 | `records/2026-09-17-已分配口径三方与两个实验.md:162` | 不相干 | 原话讲续派闸（`runner-dispatch-guard.sh`）第一次在真派发里放行的是 E155 执行员，说的是续派闸这件事，跟 replay.sh 的登记范围无关。 |
| M1 | `.claude/kb/checks-owed.md`（该登记而未登记） | 要补 | 反向核对「这一阶段做成的事」清单第⑦项「环境检查脚本登记进门禁 47 号」：脚本本身与门禁 47 号的登记都做了（现读 `.claude/gate.d/47-research-script-selftests.sh` 第 28 行含 `test-environment-check.py --selftest`），但派发提示明说「写进里程碑出口与主 agent 收尾那一步还没做」；现搜 `grep -rn "test-environment-check" .claude/kb .claude/rules .claude/singlefs-ai-sop/rules/session-wrapup.md`，除脚本自身与门禁文件外零命中——`checks-owed.md`、任何里程碑文件、`session-wrapup.md` 都没有一处记着这个缺口。要补：在 `.claude/kb/checks-owed.md` 新立一条 C 编号，写「里程碑开工/结束与主 agent 收尾没有强制跑 `test-environment-check.py`，只在门禁 47 号自证里跑」，前置写「无（脚本已就绪，只差接入点）」。 |

## 反向核对（第 10 步）：这一阶段做成的事，各自有没有一处记着

不搜 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改）。逐项现搜结果：

- 按模式找进程改由上游钩子拒绝、bash-command-detector.sh 回到只记两种：记在脚本自身注释、`.claude/agent-common.md`/`.claude/main-agent.md`（已判「要改」，不算漏记，是记错了要更新）、`.claude/gate.d/63-agent-write-scope.sh` 第⑨条。
- `proc.py` 在上游：记在 `records/2026-09-16-subagent拆分提案.md:811`。
- SOP 0.0.52、门禁 73 号加 shell-lint：记在 `.claude/kb/checks-owed.md` C384（判「要改」，需要移进已还清）、`.claude/gate.d/73-research-gate-lint.sh` 自身注释。
- 续做闸只拒被中断过的：记在 `.claude/rules/three-way-inference.md:256`、`.claude/gate.d/63-agent-write-scope.sh` 第⑦条。
- 交接摘要脚本（`agent-handover.py`）：记在 `.claude/rules/three-way-inference.md:256`。
- 看门狗的几处改法（F7–F10）：记在 `agent-watch.py` 自身注释、`.claude/rules/three-way-inference.md`（F7 上下文过大）；`.claude/main-agent.md:19`（F8、F9 部分）现文过时，已判「要改」。
- 环境检查脚本登记进门禁 47 号：脚本与门禁登记都在，但「写进里程碑出口与主 agent 收尾」这一半没有记录——见上表 M1，判「要补」。
- 门禁 76 号：记在 `.claude/kb/milestone/02-second-txn.md`（现搜 `second-txn-hooks` 命中）。
- 四个研究脚本展开一行函数：记在 `.claude/kb/checks-owed.md` C384（同上，判「要改」）。
- `replay.sh` 登记 E155 第二次跑第二段：记在 `replay.sh` 自身（172 行）、`.claude/kb/experiments.md`、`.claude/kb/experiments/155-*.md`（均已现读确认写全）。
- 烧号「认了」：记在 `.claude/kb/checks-owed.md`、`.claude/kb/decisions/08-核心索引结构.md`、`.claude/kb/milestone/02-second-txn.md`、`.claude/kb/decisions-history/2026-09.md`（现搜均命中，未逐条展开核对内容是否准确，只核了「有没有记」这一半）。

## 没做什么

- 只判候选表里的 57 行加反向核对新增的 1 行（M1），候选表本身按事实表的检索词生成，罩不到没被检索词碰到的载体；除 M1 外没有再扩大反向核对的搜索范围（例如没有逐条核对烧号那四处记录内容是否准确，只核了「有没有记着」）。
- `.claude/main-agent.md:19`、`:29` 与 `.claude/agent-common.md:42` 判「要改」的三处只给出改后的句子，没有实际去改文件——按定义这三处属于「改定义」，要走三方并进定义那一轮，不归 sweep 写。
- `.claude/kb/checks-owed.md` C384 的表格搬迁（从欠账表移进已还清表）没有实际执行，只给出改后的行；`records/2026-09-16-subagent拆分提案.md:153` 的改法同样只给出改后的句子。
- 没有验证 F6 行（`records/2026-09-16-subagent拆分提案.md:59`）判「要人看」背后的假设——嵌套派发时主 agent 能不能拿到下层 agent 的会话 id——这需要现测一次真实的嵌套派发场景，超出这一轮回扫的范围。
- 没有编译 Rust、没有跑 `gate.sh` 全量；读到的门禁脚本内容（47、63、73 号）只是 `cat`/`grep` 现读源码确认判据，没有实跑这几个门禁阶段。
- 没有对候选表之外的文件做地毯式搜索来验证 M1 是不是唯一的「要补」项；只按派发提示给的「这一阶段做成的事」清单逐条搜了一遍。
