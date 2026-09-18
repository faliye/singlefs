# 背景材料：bg-notify-r1（正文 + 清单 + 附录，三方第一轮，2026-09-18）

主 agent 写的正文、`kb-sections.py` 生成并由材料员标注的小节清单、`checklist-specs.py` 按清单抽取的附录，按固定顺序拼接，排他新建。附录二（diff）不并入本文件，见 `research/prompts/_bg-notify-r1-diff.md`，各腿按正文里给的路径去读。


---

# 正文（原文）

# 背景材料：后台任务的通知与交回——共用约束、主 agent 入口、Bash 检出 hook 与看门狗的改动，三方第一轮（2026-09-18）

<!-- doc-lint:not-numbers A1 A2 A3 A4 L1 L2 L3 L4 L5 L6 L7 L8 L9 L10 L11 L12 L13 L14 -->

主 agent 写正文、判据与跑前条款；小节清单由 `research/scripts/kb-sections.py` 全量生成，附录由 `research/scripts/checklist-specs.py` 按清单抽取。正文只写指路与主 agent 的观测，定义、规则与脚本原文在附录。
规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」——门禁 72 号要求这一轮的判决按路径点名每一份改过的定义。

## 一、被判的对象

**开工快照**：`research/prompts/bg-notify-r1-start-snapshot.sha256`（8 个文件，腿跑着的时候主 agent 不改它们）。完整改动：`git diff HEAD -- .claude/agent-common.md .claude/main-agent.md .claude/hooks/bash-command-detector.sh research/scripts/agent-watch.py research/scripts/check-staged.sh`（附录二）。

| 对象 | 路径 | 今天改了什么 |
|---|---|---|
| 共用约束 | `.claude/agent-common.md` 第 40 行「长活可以等」 | 加两句：`run_in_background` 起的命令照前台写法原样交给它，不再加 `nohup`、`setsid`、`disown`，也不在末尾加 `&`，并行起几个再 `wait` 的照常写；结束本轮就是这一条回复只写一句在等什么、不再调工具 |
| 主 agent 入口 | `.claude/main-agent.md`「交回怎么读」 | 加一段：交回按 SubagentHandback 消息判；status 为 completed、结果写「This agent has not reported yet: it is waiting on its own background work」或 note 写「the result below may be interim」的，是子 agent 结束本轮去等自己的后台任务，不当交回、不当出错 |
| Bash 检出 hook | `.claude/hooks/bash-command-detector.sh` | 加第三种检出：`tool_input.run_in_background` 为真且命令里有 `nohup`、`setsid`、`disown`，或有一个单独的 `&`（不是 `&&`、`>&`、`&>`、`\|&`、`<&`）而它后面没有 `wait`；自检加 7 种，`BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1` 判红 |
| 看门狗 | `research/scripts/agent-watch.py` | 判「被停」除了子 agent 会话记录里的 `[Request interrupted`，另读主会话记录（`<会话 id>.jsonl`）里 TaskStop 对它的成功结果，时刻不早于子 agent 最后一条记录减 30 秒就判被停；自检加两个样本，`AGENT_WATCH_BREAK=taskstop` 判红 |
| 暂存区门禁 | `research/scripts/check-staged.sh` | 阶段退出码 77（无对象可判）不再记 RED，单列并在成功行列名；点名的阶段全是 77 时整体退 77；自检加一格，`CHECK_STAGED_77_IS_RED=1` 判红 |

经过与起因写在 `records/2026-09-16-subagent拆分提案.md` 第三十三节。

## 二、现查事实（主 agent 的观测，2026-09-18，UTC）

**实现今天的样子**：这一轮判的是 agent 定义、hook 与协作脚本，不是文件系统实现；`crates/` 里没有与子 agent、后台任务通知有关的代码（`grep -rniE 'subagent|run_in_background|SubagentHandback|TaskStop' crates/ | wc -l` → 0）。

| 看了什么 | 看到什么 | 怎么看的 |
|---|---|---|
| 门禁分诊员 a339b505557ce905e 怎么起后台任务 | 13:52:19 `run_in_background` 起 `nohup nice -n 19 bash .claude/scripts/gate.sh --staged > …/gate-run.log 2>&1 & echo "started pid $!"; disown`，harness 的后台任务当场结束，`gate.sh` 的父进程变成 1；13:52:23、13:52:40、13:52:51 用同样的 `nohup … &` 形态起三个 `cache-keepalive.sh`，三份日志 13:56:14 / 13:56:31 / 13:56:43 写出，会话里没有对应的唤醒；13:52:30 另起 `until ! ps -p 2296232 …; do sleep 5; done`（不带 `&`，这一条跟着 `gate.sh` 结束）；13:53:20 改成不带 `&` 起的 `bash research/scripts/cache-keepalive.sh`，它 13:57:12 叫醒了子 agent | 子 agent 会话记录逐条列工具调用（主 agent 读，腿不碰 `~/.claude/`）；`ls -la --time-style=full-iso /tmp/claude-1000/gate-triage-commit-0918/` |
| 子 agent 怎么结束本轮 | 13:53:26–13:53:41 连调五次空命令（`sleep 1`、`echo waiting`、`true`、`sleep 0`、`# no-op, ending turn to wait for background notification`），13:53:44 只写一句文字才结束本轮 | 同上 |
| 主 agent 收到的通知原文 | 子 agent 结束本轮而后台任务还在时：status `completed`，note「This agent stopped with background work of its own still running. It may resume on its own when that work completes or reports, and the same task-id notifies again if it does; the result below may be interim.」，result「This agent has not reported yet: it is waiting on its own background work and will deliver its report through SubagentHandback when that finishes.」；这一次收到两次（13:53:44、13:58:29 各结束一次本轮）。子 agent 交回之后：status `completed`，note「A task-notification fires each time this agent stops with no live background children of its own. The user can send it another message and resume it, so the same task-id may notify more than once.」，result「This agent's report was delivered to you as a message from "<id>" (its SubagentHandback call). Read it there; it is not repeated here.」（sweep a78b10a27371fb5a5 那一次） | 主会话里收到的通知逐字抄 |
| TaskStop 之后两边的记录 | 主会话记录 14:01:49.765 有 TaskStop 的结果，`toolUseResult` = `{"message": "Successfully stopped task: a339b505557ce905e (Run gate on full staged set)", "task_id": "a339b505557ce905e", "task_type": "local_agent", "command": "Run gate on full staged set"}`；子 agent 会话记录最后一条 13:58:29.141，之后一条都没有 | 主会话记录第 355 行附近；子 agent 会话记录 `tail -3` |
| 改前的看门狗 | 14:12:14 报「状态=本轮结束、没交回」「⚠️ 无动静：最后一条记录在 13m45s 以前」，退出码 3，提示「接着盯就再起一个看门狗」；改后同一个 id 的 report 报「状态=被停」、没有告警 | 后台任务输出原文；`python3 research/scripts/agent-watch.py report --agents a339b505557ce905e` |
| hook 收不收得到 `run_in_background` | 收得到：主 agent 14:45:56 用 `run_in_background` 起 `true > /dev/null 2>&1 &`，检出记录多了一行，findings 是新加的那一种 | `tail -2 /tmp/claude-1000/agent-hook-detections.jsonl` |
| 看门狗怎么处理 hook 检出 | 本会话的检出，除了只检出「没有 timeout 的等待循环」的，读到就退出、叫醒主 agent；新加的那种因此会叫醒主 agent | `research/scripts/agent-watch.py` 的 `read_detections` |
| 别处已有的相关禁令 | `.claude/agents/three-way-local-attack.md:27`「不许 `setsid`、`&`、`disown`」、`.claude/rules/three-way-inference.md:302`「本地腿要前台跑，不要 `setsid ... & disown`」，管的是本地腿那一条 `ask-local.sh` 命令 | 回扫报告 `research/prompts/watchdog-taskstop-sweep-report.md` |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| A1 两句新指令本身 | 共用约束与主 agent 入口加的两段，执行者照字面做会不会做错：①「不加 `nohup`、`setsid`、`disown`，也不在末尾加 `&`；并行起几个再 `wait` 的照常写」有没有误禁合法写法或漏禁会脱钩的写法；②「结束本轮就是这一条回复只写一句在等什么、不再调工具」与别处要求结束本轮之前先做的事（起缓存计时器、写进度记录）冲不冲突；③ 主 agent 入口那段读法，对两种 completed 通知（等后台、已交回）判得对不对；④ 与 16 份定义、`.claude/rules/` 里现有的后台跑法指令有没有打架 | 给出一条按新指令字面会做错的具体命令或场景（会脱钩却允许的、合法却被禁的），或一处定义与新指令字面冲突（路径加行号） |
| A2 hook 第三种检出 | 判据「`run_in_background` 为真，且有 `nohup`/`setsid`/`disown`，或有单独的 `&` 而它后面没有 `wait`」对真实会写出来的命令有没有漏检与误检；误检的代价（叫醒主 agent）与漏检的代价谁大 | 一条会让等的东西与完成通知脱钩、而 hook 不记的命令；或一条不脱钩、而 hook 记了的常见命令 |
| A3 主 agent 入口与 harness 原文 | 入口那段引的两句英文与第二节的通知原文是否逐字对得上；按那段做，主 agent 在「子 agent 结束本轮等后台」「子 agent 交回」「子 agent 被 TaskStop 停掉」三种情形下各会怎么做，与看门狗的判法是否一致 | 引文与原文差一个词；或三种情形里有一种按入口做与看门狗判的相反 |
| A4 看门狗与 check-staged 的改法 | `agent-watch.py` 用「主会话记录里 TaskStop 成功结果的时刻 ≥ 子 agent 最后一条记录 − 30 秒」判被停：停在工具中途、停了又续做、续做之后又停、同一个会话停过别的 id、主会话记录不在 `<会话目录>.jsonl` 这几种情形下判得对不对；自检的两个样本与判红开关分不分得出新旧判法；`check-staged.sh` 把 77 单列之后，红、绿、全 77、77 混着红四种组合的退出码与成功行对不对 | 构造一种情形，按代码判出的状态与实际相反；或自检在旧判法下照样绿 |

**反向接受条款**：A1 打中 ⇒ 改那句指令（写成指令，不写经过），与之打架的定义一并改；A2 打中漏检 ⇒ 判据能补就补、补不了写进 hook 文件头「判不到的」；打中误检 ⇒ 常见的就收窄，少见的记下由主 agent 看了判；A3 打中 ⇒ 改入口那段的引文或读法；A4 打中 ⇒ 改代码并补一个会红的自检样本。
**失败条款**：一条腿引的定义、规则或脚本原文与附录、快照里的文件不符 ⇒ 那一格作废；「没打中」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（三条推论腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 找新指令会让人做错的地方、与现有定义打架的地方；找 hook 在附录三那张表以外的漏检与误检，判误检叫醒主 agent 的代价 | A1、A2（表外的命令） | A3、A4；附录三表里 L1–L14 那十四条 |
| Sonnet 正推（`three-way-forward`） | 逐字核入口引文与通知原文、推三种情形下主 agent 与看门狗的行为；逐行核看门狗与 check-staged 的改法与自检 | A3、A4 | 不替攻方找反例 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按附录三的事实表，对 L1–L14 每一条逐格填：共用约束那句字面禁不禁、hook 判据字面记不记、这条命令起完之后外层 shell 是不是在真正的活跑完之前就退出了；三格不一致的逐条说 | A2（只限 L1–L14） | 表外的命令；A1、A3、A4 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/bg-notify-r1-*-output*.md`）与主 agent 的判决（`research/prompts/bg-notify-r1-main-verification.md`）。本地攻方的提示里不许放 hook 对 L1–L14 的实际判定（不许先跑 hook 再把结果写进提示）。

## 附录三、本地攻方的事实表（主 agent 逐条构造的命令，只列字面特征，判定不在这里）

每条命令都假定用 `run_in_background` 起。「单独的 `&`」指不属于 `&&`、`>&`、`&>`、`|&`、`<&` 的那个 `&`。

| 编号 | 命令原文 | 出处 | 含 `nohup` / `setsid` / `disown` 这几个词 | 单独的 `&` 出现在哪（逐个） | 每个单独 `&` 之后有没有 `wait` 这个词 | `&` 或这几个词是不是在引号里 |
|---|---|---|---|---|---|---|
| L1 | `nohup nice -n 19 bash .claude/scripts/gate.sh --staged > gate-run.log 2>&1 & echo "started pid $!"; disown` | 子 agent 13:52:19 原样 | `nohup`、`disown` | `2>&1` 之后那个 | 没有 | 不在 |
| L2 | `nohup nice -n 19 bash research/scripts/cache-keepalive.sh > keepalive.log 2>&1 &` | 子 agent 13:52:23 原样（去掉路径前缀） | `nohup` | 末尾那个 | 没有 | 不在 |
| L3 | `bash research/scripts/cache-keepalive.sh` | 子 agent 13:53:20 原样 | 无 | 无 | — | — |
| L4 | `until ! ps -p 2296232 > /dev/null 2>&1; do sleep 5; done; echo "gate.sh finished"` | 子 agent 13:52:30 原样 | 无 | 无（只有 `2>&1`） | — | — |
| L5 | `cargo build 2>&1 \| tail -3 && echo ok` | 构造 | 无 | 无（`2>&1`、`&&`） | — | — |
| L6 | `bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait` | 构造 | 无 | 两个：`a.log 2>&1` 之后、`b.log 2>&1` 之后 | 两个之后都有 | 不在 |
| L7 | `wait; bash late.sh &` | 构造 | 无 | 末尾那个 | 没有（`wait` 在它前面） | 不在 |
| L8 | `make \|& tee out.log` | 构造 | 无 | 无（`\|&`） | — | — |
| L9 | `curl -s "https://example.invalid/?a=1&b=2" > page.html` | 构造 | 无 | 一个：`a=1` 与 `b=2` 之间 | 没有 | 在双引号里 |
| L10 | `setsid bash long.sh > long.log 2>&1 < /dev/null` | 构造 | `setsid` | 无（`2>&1`） | — | 不在 |
| L11 | `(sleep 300; echo done) &> timer.log` | 构造 | 无 | 无（`&>`） | — | — |
| L12 | `bash x.sh & pid=$!; wait "$pid"` | 构造 | 无 | 一个：`x.sh` 之后 | 有 | 不在 |
| L13 | `echo "remember: nohup is not needed here"` | 构造 | `nohup` | 无 | — | `nohup` 在双引号里 |
| L14 | `bash -c 'sleep 10 &'` | 构造 | 无 | 一个：`sleep 10` 之后 | 没有 | 在单引号里（内层 shell 的命令） |

---

# 小节清单（材料员标注）

# 小节清单：bg-notify-r1（三方第一轮，2026-09-18）

机器生成、未再过滤的清单见 `research/scripts/kb-sections.py` 原始产出；下表在此基础上逐行标注「抄 / 不抄 / 理由」。

### 小节清单：`.claude/agent-common.md`

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 项目 subagent 的共用约束 | 抄 | 与下一行引言合并成 1-5 行区间；引言点出门禁 71 号与「改 agent 定义与共用约束，走同一条三步」，是理解本轮共用约束改动权威来源的前提 |
| （项目 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 随（checklist-specs.py 把 H1 与紧邻的引言行自动并成 1-5 行区间） |
| ## 派发 | 不抄 | 讲谁能派发、不许起 `claude` 会话，与后台任务通知、nohup、hook 无关 |
| ## 规则怎么读 | 不抄 | 讲规则分层怎么读、本机时区；command-safety.md 那两节管的是 pkill 与退不回去的操作，不是后台跑法 |
| ## 写 | 不抄 | 讲写范围闸怎么用，与后台任务通知无关 |
| ## 不做 | 抄 | 「长活可以等」那一段（对象表第 1 行，今天改的第 40 行两句）就在这一节，是本轮被判对象本身 |
| ## 门禁 | 不抄 | 讲门禁阶段登记表怎么查、怎么跑自己该跑的阶段，与后台跑法、通知判读无关 |
| ## 报告 | 不抄 | 讲报告怎么写、分段落盘、交回只调一次，不是这一轮 A1–A4 要判的后台任务通知格式 |

### 小节清单：`.claude/main-agent.md`

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 主 agent：任务入口与职责 | 抄 | 与下一行引言合并成 1-4 行区间；引言写「为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`」，与正文第 20 行「经过与起因写在……第三十三节」的说法要对得上 |
| （主 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 随（与上一行 H1 合并） |
| ## 职责 | 不抄 | 讲主 agent 调度判断的总职责、一次性活派 general-purpose，与后台任务通知、hook 无关 |
| ## 一轮怎么开、怎么收 | 不抄 | 讲一轮工作怎么开工、怎么收拢线头，与后台任务通知无关 |
| ## 派出去之后 | 抄 | 讲看门狗怎么起、多久复检一次、子 agent 提示缓存怎么续（`cache-keepalive.sh`），直接对应判据 A1②「与别处要求结束本轮之前先做的事（起缓存计时器……）冲不冲突」与 A4 看门狗判据 |
| ## 交回怎么读 | 抄 | 今天改的那一段（对象表第 2 行），正文第 30 行引的通知原文要与这里逐字核对（判据 A3） |
| ## 派发提示怎么写 | 不抄 | 讲派发提示怎么写简练，与这一轮的技术判据无关 |
| ## 什么时候派哪个 agent | 不抄 | 是一张「什么时候派谁、按什么次序」的调度表，表里没有与后台跑法、nohup、通知判读相关的字句 |

### 小节清单：`.claude/rules/implementation-workflow.md`

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 实现改动的流程：写代码 → 三方对抗 → checker，提交前跑 herd7 与 QEMU | 抄 | 与下一行引言合并成 1-5 行区间；引言说明这是项目本地规则、压在三方论证与 herd7/QEMU 装置上，是理解「走同一条三步」适用范围的前提 |
| （实现改动的流程：写代码 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 随（与上一行 H1 合并） |
| ## 三步，缺一步就不算做完 | 抄 | 「改 agent 定义与共用约束，走同一条三步」那一节正文写「与改 `crates/` 同规矩……走同一轮三方」，这里的「三步」「三方」指的就是这张表，理解那句引用离不开它 |
| ## 改 agent 定义与共用约束，走同一条三步 | 抄 | 正文第 6 行直接点名这一节（门禁 72 号要求这一轮的判决按路径点名每一份改过的定义） |
| ## 代码轮派腿之前记一份开工快照 | 抄 | 正文「开工快照」一句与「腿跑着的时候主 agent 不改它们」逐句对应这一节内容，需要核对来源 |
| ## 提交前必跑 herd7 与 QEMU | 不抄 | 讲 `crates/` 改动提交前的 herd7/QEMU 阶段；正文第 24 行已现查 `crates/` 里没有与子 agent、后台任务通知相关的代码，这一节不适用本轮 |
| ## 为什么做成门禁而不是提醒句 | 不抄 | 举的是「发布 B」那次 `crates/` 改动没走三方的例子，与本轮（agent 定义与脚本）无关 |

### 小节清单：`.claude/rules/three-way-inference.md`

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 推论要三方独立论证 | 抄 | 与下一行引言合并成 1-7 行区间；引言说明这条规则只在本机生效、共享规则在别处，是理解后面几节判据的前提 |
| （推论要三方独立论证 标题之下、第一个下级标题之前的正文：第 2-7 行） | 抄 | 随（与上一行 H1 合并） |
| ## 适用范围 | 不抄 | 讲的是「推论」什么时候要走三方；这一轮走三方的依据是 `implementation-workflow.md`「改 agent 定义与共用约束」那一条，不是这一节 |
| ## 各条腿必须互不重复 | 不抄 | 讲三条腿怎么分工、「两条攻方腿攻击面要分开写」的写法要求；是主 agent 写分工表时要守的规矩，正文第四节的分工表已经照办，不是 A1–A4 要判的实质问题 |
| ## 引 kb 里的条目要整行抄，不许摘句——三处都管 | 不抄 | 管的是材料员怎么抄条款，是这一轮材料准备本身要守的纪律，不是三条推论腿要判的 A1–A4 内容 |
| ## 判决由主 agent 做，不由投票做 | 不抄 | 讲判决怎么由主 agent 做，是主 agent 的判决流程，不是 A1–A4 的判据 |
| ## 多轮：一次打穿不算数，三轮里多数打穿才算（2026-09-06 用户明令） | 不抄 | 讲多轮攻防怎么算数、什么时候停，是主 agent 的判决流程，不是 A1–A4 的判据 |
| ## 一条腿只抽一次样不算一次观测——否定结论尤其不算 | 抄 | 正文「失败条款」写「『没打中』一次不算（本地腿抽两次）」，正是这一节「做法」那句的具体应用，需要核对来源 |
| ## 云端腿的报告要分段落盘 | 不抄 | 讲云端腿报告怎么分段写进文件，是腿自己的写作纪律，与 A1–A4 判据无关 |
| ## 本地腿缺席时必须显式报告 | 不抄 | 讲本地腿网关不通、正文为空时怎么报错；这一轮本地腿有正文与事实表可用，不适用这一节 |
| ## 给本地腿的提示一律用英文 | 抄 | 主 agent 明写「至少『本地腿要前台跑』那一段所在小节」；正文附录三本地攻方那一列的判据字面依赖这一节「本地腿要前台跑，不要 `setsid ... & disown`」的原文 |

### 小节清单：`.claude/agents/gate-triage.md`

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 门禁分诊（gate-triage） | 抄 | 与下一行引言合并成 1-15 行区间；引言写「开工先读 `.claude/agent-common.md`……要用的规则照共用约束『规则怎么读』一节读」与「只分诊，不修」，是理解它怎么起后台任务的前提 |
| （门禁分诊（gate-triage） 标题之下、第一个下级标题之前的正文：第 10-15 行） | 抄 | 随（与上一行 H1 合并） |
| ## 输入（主 agent 必须给） | 不抄 | 列的是暂存状态、diff 范围、报告路径这三项输入，与后台跑法或通知判读无关 |
| ## 做什么 | 抄 | 第 2 条「超过 Bash 单次上限时后台跑、结束后读输出」正是正文观测里门禁分诊员（a339b505557ce905e）13:52:19 起后台 `gate.sh` 那一步的依据，是判据 A1④「与现有定义打架」的直接对象 |
| ## 写范围 | 不抄 | 讲报告文件、草稿目录与门禁自己的 git 写权限，与后台跑法无关 |
| ## 产出 | 不抄 | 讲产出一张表的格式，与后台跑法无关 |
| ## 没做什么（固定会有的） | 不抄 | 讲它不修任何红、归属只按暂存区 diff 判，与后台跑法无关 |

### 小节清单：`.claude/agents/three-way-local-attack.md`

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 本地攻方腿（three-way-local-attack） | 抄 | 与下一行引言合并成 1-15 行区间；引言点名开工先读 `three-way-inference.md` 里「各条腿必须互不重复」「一条腿只抽一次样不算一次观测」「本地腿缺席时必须显式报告」「给本地腿的提示一律用英文」四节，需要与本材料对 `three-way-inference.md` 的抄/不抄判断对得上 |
| （本地攻方腿（three-way-local-attack） 标题之下、第一个下级标题之前的正文：第 10-15 行） | 抄 | 随（与上一行 H1 合并） |
| ## 输入（主 agent 必须给） | 不抄 | 列的是轮名、正文与附录路径、攻击面、提示文件路径、草稿目录、禁读清单，与后台跑法无关 |
| ## 做什么 | 抄 | 第 3 步「前台跑 `bash research/scripts/ask-local.sh <提示文件> > ……`，不许 `setsid`、`&`、`disown`」正是正文第 35 行「别处已有的相关禁令」逐字引用的那一句，判据 A1④要核对是否与新指令打架 |
| ## 写范围 | 不抄 | 讲提示文件、核对表、样本文件的写范围，与后台跑法无关 |
| ## 产出 | 不抄 | 讲产出报告与核对表路径的格式，与后台跑法无关 |
| ## 没做什么（固定会有的） | 不抄 | 讲它不解读、不总结、不采纳本地模型答复，与后台跑法无关 |

### 小节清单：`records/2026-09-16-subagent拆分提案.md`

主 agent 只点名「第三十三节」（正文第 20 行：「经过与起因写在 `records/2026-09-16-subagent拆分提案.md` 第三十三节」）。下表逐节现查标题与内容，除第三十三节外均不抄，理由按各节自己的主题写明、可与标题核对。

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # subagent 拆分提案（2026-09-16） | 不抄 | 是整份提案 2026-09-16 立项时的总览引言（九项用户决定、彼时的进度描述），不是第三十三节那次事件；正文只点名第三十三节 |
| （subagent 标题之下、第一个下级标题之前的正文：第 2-6 行） | 不抄 | 同上，且不与 H1 合并（H1 本身已标不抄） |
| ## 一、现状：这个仓已经在派 agent，只是每次现写规矩 | 不抄 | 讲拆分之前「现写规矩」的问题现状，不是第三十三节 |
| ## 二、拆分判据 | 不抄 | 讲一件活该不该拆给独立 agent 的判据，不是第三十三节 |
| ## 三、留在主 agent 的 | 不抄 | 讲哪些事留在主 agent 自己做，不是第三十三节 |
| ## 四、agent 清单：十六个，分四族 | 不抄 | 十六个 agent 定义的总览清单，不是第三十三节 |
| ### 族一　三方论证（压在 `.claude/rules/three-way-inference.md`，只能项目本地） | 不抄 | 三方论证族 agent 清单，不是第三十三节 |
| ### 族二　实现（压在 `.claude/rules/implementation-workflow.md`） | 不抄 | 实现族 agent 清单，不是第三十三节 |
| ### 族三　门禁与回扫 | 不抄 | 门禁与回扫族 agent 清单，不是第三十三节 |
| ### 族四　kb、实验、外部资料 | 不抄 | kb/实验/外部资料族 agent 清单，不是第三十三节 |
| ## 五、写范围怎么拦：先有闸，再铺 agent | 不抄 | 讲写范围闸先于 agent 铺开建的次序，不是第三十三节 |
| ## 六、调度形态 | 不抄 | 讲调度形态设计，不是第三十三节 |
| ## 七、落地次序 | 不抄 | 讲落地次序安排，不是第三十三节 |
| ## 八、放哪一层 | 不抄 | 讲拆分内容放项目本地还是上游，不是第三十三节 |
| ## 九、用户定案 | 不抄 | 讲 2026-09-16 那批用户定案九项，不是第三十三节 |
| ## 十、这份提案没做的 | 不抄 | 讲提案当时没做的事项清单，不是第三十三节 |
| ## 十一、第一轮查出、还没做的欠账 | 不抄 | 讲第一轮查出的欠账，不是第三十三节 |
| ## 十二、第 0 步实测（2026-09-16 UTC 22:31 与 23:05–23:30，东京 09-17 07:31 与 08:05–08:30） | 不抄 | 讲定义/hook 继承关系的第 0 步实测，不是第三十三节 |
| ## 十三、定义的静态核实（2026-09-17） | 不抄 | 讲定义文件的静态核实，不是第三十三节 |
| ## 十四、CLAUDE.md 与规则做减法的次序 | 不抄 | 讲 CLAUDE.md 与规则精简的次序，不是第三十三节 |
| ## 十五、门禁阶段归属与 `crash-verifier`（2026-09-17） | 不抄 | 讲门禁阶段归属表与新增 `crash-verifier` 的经过，不是第三十三节 |
| ## 十六、写范围闸与实现员试跑（2026-09-17） | 不抄 | 讲写范围闸与实现员试跑的经过，不是第三十三节 |
| ### 写范围闸 | 不抄 | 同上细项，不是第三十三节 |
| ### 实现员试跑 | 不抄 | 同上细项，不是第三十三节 |
| ## 十七、全部定义的重推、试跑与第一轮对抗（2026-09-17） | 不抄 | 讲十六个定义的重推、试跑、第一轮对抗，不是第三十三节 |
| ### 重推 | 不抄 | 同上细项，不是第三十三节 |
| ### 试跑 | 不抄 | 同上细项，不是第三十三节 |
| ### 对抗 | 不抄 | 同上细项，不是第三十三节 |
| ### 这一轮新测到的两条事实 | 不抄 | 同上细项，不是第三十三节 |
| ## 十八、第一轮改法的重新试跑（2026-09-17） | 不抄 | 讲第一轮改法的重新试跑，不是第三十三节 |
| ### 这一轮新测到的事实 | 不抄 | 同上细项，不是第三十三节 |
| ### 顺带发现（归文件系统那边，交用户） | 不抄 | 同上细项，讲的是文件系统那边的发现，不是第三十三节 |
| ## 十九、第二轮对抗（2026-09-17） | 不抄 | 讲第二轮对抗，不是第三十三节 |
| ## 二十、按职能纸面模拟（2026-09-17） | 不抄 | 讲按职能纸面模拟，不是第三十三节 |
| ## 二十一、第一次拿真活跑的评估（2026-09-17） | 不抄 | 讲第一次拿真活跑的评估（含六批小改动），不是第三十三节 |
| ## 二十二、m2-wave1 阶段的实测（2026-09-18） | 不抄 | 讲 m2-wave1 阶段的用量实测，不是第三十三节 |
| ## 二十三、门禁 10 号的条数登记位（2026-09-18） | 不抄 | 讲门禁 10 号条数登记位改法，不是第三十三节 |
| ## 二十四、续派闸的一个边角（2026-09-18） | 不抄 | 讲续派闸的一个边角情形，不是第三十三节 |
| ## 二十五、Bash 检出 hook 的一处误报（2026-09-18） | 不抄 | 讲的是 hook 另一种误报（heredoc 正文里引用了被禁命令名），与这一轮新加的第三种检出（`run_in_background` 配 `nohup`/`setsid`/`disown`/落单 `&`）不是同一处，判 A2 不需要 |
| ## 二十六、这一阶段为什么收敛慢：两个可核的数（2026-09-18） | 不抄 | 讲这一阶段收敛慢的两个可核的数（分叉因子、单位推论墙钟），不是第三十三节 |
| ## 二十七、门禁 69 号第一次在真仓上跑的两处误判（2026-09-18） | 不抄 | 讲门禁 69 号的两处误判，不是第三十三节 |
| ## 二十八、定义只写怎么做：门禁 71、72 号与清出定义的经过（2026-09-18） | 不抄 | 讲门禁 71、72 号与清出定义的经过；正文第 6 行引的规则原文已在 `implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」那一节核过，不需要重复来源 |
| ## 二十九、共用约束与主 agent 入口搬进 `.claude/agents/`（2026-09-18） | 不抄 | 讲共用约束与主 agent 入口搬迁的经过，不是第三十三节 |
| ## 三十、子 agent 自己续提示缓存（4 分钟计时器）的效果：2026-09-18 这一个会话的数 | 不抄 | 讲缓存续期计时器效果的数，不是第三十三节 |
| ## 三十一、开了 `omitClaudeMd` 的子 agent 仍会被注入上游副本目录里的 `CLAUDE.md`（2026-09-18） | 不抄 | 讲 `omitClaudeMd` 子 agent 被注入 `CLAUDE.md` 的问题，不是第三十三节 |
| ## 三十二、崩溃验证员那一趟（门禁 54、55、57、59，2026-09-18）交回时查出的两处 | 不抄 | 讲崩溃验证员那一趟查出的两处，不是第三十三节 |
| ## 三十三、TaskStop 停掉在等后台任务的子 agent，看门狗认不出「被停」（2026-09-18） | 抄 | 正文第 20 行明写「经过与起因写在……第三十三节」，直接点名 |
| ## 历史版本 | 不抄 | 记的是这份提案文档本身各节文字的修订历史（哪句话从什么改成什么），不是 bg-notify 这一轮的事实依据，也不是第三十三节 |
| ### 2026-09-16 | 不抄 | 同上，文档修订历史，不是第三十三节 |
| ### 2026-09-17 | 不抄 | 同上，文档修订历史，不是第三十三节 |
| ### 2026-09-18 | 不抄 | 同上，文档修订历史；这一节里提到「效果见第三十节」等指路，不是第三十三节本身 |

### 脚本类（非 markdown，过不了 `kb-sections.py`，整份用 `--extra` 带进附录）

主 agent 点名四份脚本「整份进附录（不是 kb，按文件整抄）」：`.claude/hooks/bash-command-detector.sh`（127 行）、`research/scripts/agent-watch.py`（824 行）、`research/scripts/check-staged.sh`（154 行）、`research/scripts/cache-keepalive.sh`（54 行）。这四份不是 markdown、没有 `#` 级标题结构，`kb-sections.py` 生成的清单对它们没有意义，因此不生成小节清单，改用 `checklist-specs.py --extra 文件:1-末行` 整份带入附录，四份都标「抄」：

| 文件 | 抄 / 不抄 | 理由 |
|---|---|---|
| `.claude/hooks/bash-command-detector.sh` | 不抄（改用 --extra 整份带入附录） | 对象表第 3 行，今天改的那份文件（新增第三种检出），判据 A2 要判它；不是 markdown、没有标题结构，checklist-specs.py 用 `--extra .claude/hooks/bash-command-detector.sh:1-127` 整份带入附录，见附录出处行 |
| `research/scripts/agent-watch.py` | 不抄（改用 --extra 整份带入附录） | 对象表第 4 行，今天改的那份文件（TaskStop 判定改法），判据 A4 要判它；不是 markdown、没有标题结构，checklist-specs.py 用 `--extra research/scripts/agent-watch.py:1-824` 整份带入附录，见附录出处行 |
| `research/scripts/check-staged.sh` | 不抄（改用 --extra 整份带入附录） | 对象表第 5 行，今天改的那份文件（77 单列），判据 A4 要判它；不是 markdown、没有标题结构，checklist-specs.py 用 `--extra research/scripts/check-staged.sh:1-154` 整份带入附录，见附录出处行 |
| `research/scripts/cache-keepalive.sh` | 不抄（改用 --extra 整份带入附录） | 今天没改，但正文第二节两处观测（子 agent 起三个计时器、`.claude/main-agent.md`「派出去之后」引它）与共用约束「不做」一节的「结束本轮去等之前再起一次」都指着这份脚本，A1②要判它与「结束本轮只写一句、不再调工具」冲不冲突，需要看它的完整行为（230 秒后退出的机制）；不是 markdown，checklist-specs.py 用 `--extra research/scripts/cache-keepalive.sh:1-54` 整份带入附录，见附录出处行 |

---

# 附录（`checklist-specs.py` 按清单抽取）

**出处 `.claude/agent-common.md:1-5`（整段抄，未转述）**

```markdown
# 项目 subagent 的共用约束

`.claude/agents/` 下每个定义开工前先读这一份；与定义冲突时以定义为准。
这份与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进 `records/2026-09-16-subagent拆分提案.md` 或 `.claude/kb/`，这里不写、也不留解释的尾巴；要用到那些内容时写成一条读取指令（「开工先读：`文件`「小节」」）。门禁 71 号判这一条（规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。

```

**出处 `.claude/agent-common.md:32-43`（整段抄，未转述）**

```markdown
## 不做

- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
- 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
- 不读派发提示给的禁读清单里的文件。
- 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
- 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起（命令照前台的写法原样交给它，不再加 `nohup`、`setsid`、`disown`，也不在末尾加 `&`；并行起几个再 `wait` 的照常写），起完结束本轮，完成时会通知你；结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
  **结束本轮去等之前，再用 `run_in_background` 起一次 `bash research/scripts/cache-keepalive.sh`**：它 230 秒后退出，那条完成通知把你叫醒。被它叫醒就看一眼等的东西跑完没有：没跑完再起一次、结束本轮接着等，跑完了接着干、不用再起。自己已经交回或被停的不用管。预计单段要等 15 分钟以上的（层 0 全量、E152（按里程碑对比六家文件系统的文件性能） 那类），不起计时器。
  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。

```

**出处 `.claude/main-agent.md:1-4`（整段抄，未转述）**

```markdown
# 主 agent：任务入口与职责

**所有任务从这里进。** 这份只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`，公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。

```

**出处 `.claude/main-agent.md:17-24`（整段抄，未转述）**

```markdown
## 派出去之后

盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `python3 research/scripts/agent-watch.py watch --agents <这次派出的 agent id，逗号分隔>`：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。

子 agent 的提示缓存是 5 分钟档，由它自己续（`research/scripts/cache-keepalive.sh`，共用约束「不做」一节），主 agent 不定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上这一条。

叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，由看门狗盯着进程。

```

**出处 `.claude/main-agent.md:25-30`（整段抄，未转述）**

```markdown
## 交回怎么读

判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。

交回按子 agent 的 SubagentHandback 消息判。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。

```

**出处 `.claude/rules/implementation-workflow.md:1-5`（整段抄，未转述）**

```markdown
# 实现改动的流程：写代码 → 三方对抗 → checker，提交前跑 herd7 与 QEMU

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/implementation-workflow.md:6-15`（整段抄，未转述）**

```markdown
## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，正推腿核「代码做的是不是条款说的」、反推腿攻「哪一格会错」、本地腿找反例；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（层 0 全量）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

```

**出处 `.claude/rules/implementation-workflow.md:16-23`（整段抄，未转述）**

```markdown
## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**（用户 2026-09-18 定）：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。门禁 71 号判这一条。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。

**为什么**：定义是所有 subagent 每次开工都要读的东西，混进说明会让它越写越长（起步上下文直接变贵），而且改一句的影响面与改一行代码一样宽——2026-09-18 一轮里 16 份定义里有 36 行是在定义里记经过，没有任何检查拦过。数与经过在 `records/2026-09-16-subagent拆分提案.md`。

```

**出处 `.claude/rules/implementation-workflow.md:24-29`（整段抄，未转述）**

```markdown
## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），交核查员当输入；腿跑着的时候主 agent 不改这些文件。

**为什么**：腿引的行号是它读那一刻的，主 agent 轮内改一行，核查员就分不清「腿引错了」还是「文件后来变了」。实测（2026-09-18，`m2-wave1-code-r1`）：主 agent 在两条腿交回之间改了 `.claude/kb/milestone/02-second-txn.md`（正是正推腿打中的那句），核查员靠 mtime 与 `git diff` 逐处比对才判出「改动是同行数原地替换、只碰一处」，它自己在报告里写「本轮靠 mtime + git diff 补救纯属运气」。要改的等判决时一起改。

```

**出处 `.claude/rules/three-way-inference.md:1-7`（整段抄，未转述）**

```markdown
# 推论要三方独立论证

**这是 singlefs 的项目本地规则，而且只在本机生效——不进上游 SOP。**
理由很直接：其中一条腿是本机的 LLM，**别的项目、别的机器上没有**。
把一条依赖本机资源的规矩推给所有项目，那条规矩在别处只会静默失效。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/three-way-inference.md:227-247`（整段抄，未转述）**

```markdown
## 一条腿只抽一次样不算一次观测——否定结论尤其不算

模型的答复是**有变化的观测**，因此 `.claude/singlefs-ai-sop/rules/test-discipline.md`
「单次观测不算数」那条对它成立：同一份提示、同一个模型，两次可以给出方向相反的答案。

⚠️ **两个方向的门槛不一样**，与那条规则同形：

| 结论 | 采信条件 |
|---|---|
| **打中了**（给出反例、指出矛盾） | 一次就值得去核。它是个线索，真伪由主 agent 现查坐实，抽样次数不改变这一步 |
| **没打中**（「构造不出反例」「没发现问题」） | **一次不算**。它与「这一轮它没想到」分不开，而两者在答复里长得一模一样 |

实测（2026-09-06，D1（数据可移动性 / 反向索引） 已定项 5 / 6 那一轮）：本地腿同一份提示跑两次，
第一次三个 shape 全报「构造不出反例」，第二次给了两个构造——**其中一个经主 agent 现查是无效构造**
（凭空给复用者一个更小的诞生代号，而清扫准入要求它严格变大）。
两次抽样各错一半：第一次漏报，第二次误报。

⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」，
两次不一致就照 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记「不稳定」，不下结论。
云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的。

```

**出处 `.claude/rules/three-way-inference.md:271-316`（整段抄，未转述）**

```markdown
## 给本地腿的提示一律用英文

本地那条腿是 4-bit 量化模型，**中文输出会退化性复读**（「有效的有效性」「恢复恢复」），
**英文不会**。实测 23 轮：中文 12/13 判红，英文 5/5 干净、1263 个英文词零复读；
`presence_penalty` 与 `repetition_penalty` 两个采样旋钮都修不掉（前者反而 5/5 全红）。
数据与口径见 `.claude/kb/tooling.md`。

⇒ **提示用英文写，答案也让它用英文回。** 读英文答案是主 agent 的事，不构成成本；
而中文提示等于每一轮都要赌它这次坏没坏。

⚠️ **英文提示里的每一句转述，写完都要对着原文核一遍。** 本地腿读不到中文附录，条款只能译成英文转述，而转述正是「引 kb 里的条目要整行抄」那条纪律够不着的缺口：译的人觉得忠实，丢的往往是一个限定词。实测（2026-09-14，C329（写行那次发布之前推抬 F 的空发布没有检查） 第二轮）：提示把暖机规则转述成「a new instance keeps publishing empty roots until its roots cover both devices」，比 D16（发布语义） 已定项 8 原文窄——原文管的是「不让任何 fsync 返回、不向管理员确认回退」，连推空发布只是做法；本地腿据此判一个候选与暖机冲突，那一格整格作废（`research/prompts/c329-c330-r2-main-verification.md` 文件头）。⇒ 写完英文提示，把每一句转述与附录里的原文并排放一遍，缺一个限定词就补上。**多出来的也要列**：英文比原文多一个限定词、多一个括注，同样在核对表里单列一行、写明为什么加。实测（2026-09-17，alloc-basis 第三轮）：提示给「用户看到的 `df` = 第一道闸左边」多加了括注「using the currently published statistics」，核对表写成逐字照抄，本地腿据此给出的反例按原文不成立，那一格作废（`research/prompts/alloc-basis-r3-verifier-output.md` 第三节）。

**这条不靠自觉**：`ask-local.sh` 里有一道会拒绝的闸——输出判定为字词损坏时
**退出码 5**，并打印下一步。`.claude/singlefs-ai-sop/rules/show-me-test.md` 明写
「踩过的坑要做成会失败的检查，不要做成提醒句」，这一条要的就是会拒绝的闸，不是提醒句。

⚠️ **损坏有四类，签名互不相同，一个检测器只查得了一类**：复读（多吐了）、
成对标记落单（整段掉了，`**Attack P1**:it impossible.**`）、
拼接（两词粘死，`configurationing` `batchinggroup`；**后一个词还可以缺头**——
`inaccessibleisabled` 缺一个字母、`cryptographicord`（cryptographic + [Rec]ord）缺三个，
缺到尾巴不成词时前几条规则全切不开，靠「长前缀 + 既不成词也不是后缀的短尾巴」这条才抓得到；**粘上的还可以是这个词自己尾部的一截**（`reachableachable`，短到 6 个字母的 `anewew`），或者前缀是词表外的派生词（`observationalomputational`）——2026-09-17 补进 `oov-check.py`，回扫 339 份新增判红 5 份、全是真损坏、零误报，其中 `d22-item6-local-output.md` 与 `c143-r1-local-v2-output-s2.md` 当时被当成干净证据用了；闸判绿之后照样通读，闸只认得登记过的形态）。第四类是**实词自复读**
（`resetting resetting`，同一个 ≥5 字母的词连着出现两次、中间隔一个空格）——
闸只查长实词（短虚词的自复读在正常英文里合法），不收窄就误报；
收窄到长实词之后回扫 434 个文件新增判红 6 个、全是真损坏、零误报，其中 5 份当时被当成干净证据用掉了。
`ask-local.sh` 串跑 `corruption-check.py` 与 `oov-check.py`，任一判红即拒绝。
**只跑一个等于对另外三类判绿**——实测一轮含 `batchinggroup`×2 的输出被当作干净证据用了。

⚠️ **提示里不许用 markdown 强调。** 实测同一份问题、同一个模型：
标题写成 `1. **Attack P1**: ...` 时 **3/3 判损坏**，去掉粗体并加一句
「不要用任何 markdown 强调」时 **3/3 干净**；损坏点每次都落在模型回声那个模式的位置。

⚠️ **本地腿要前台跑，不要 `setsid ... & disown`。** 实测两次：后台进程消失而
stdout / stderr **都是 0 字节**，同一条命令前台 56 秒跑完。
0/0 正是「缺席必须显式报告」要禁的形态，后台跑法让它无法诊断。

**闸判红之后**：那一轮**作废重跑**，不许记成「三方不一致」——
否则每次都会不一致，这条规则就退化成了摆设。
⚠️ **这不是洁癖**：实测一题上三轮损坏输出全部给出与已定决策冲突的答案，
干净那轮给的才是对的。**损坏的不只是词。**
确有必要采用带损坏的输出时，设 `ASK_LOCAL_ALLOW_CORRUPT=1`，
并在结论里写明这一票带瑕疵。

⚠️ **闸报「没跑成」时不是通过。** 检测器自己出错（退出码 2）与判红（退出码 1）
是两件事，`ask-local.sh` 分开报。看到「没做字词损坏检查」就是**这一项没验**，
不许当成验过了。

```

**出处 `.claude/agents/gate-triage.md:1-15`（整段抄，未转述）**

```markdown
---
name: gate-triage
description: 门禁分诊：跑准入门禁，把每个红阶段判成这一轮的改动、别的会话的改动还是环境，原样抄下一步。只在主 agent 点名派发、并给出这一轮的暂存状态时用；不要自动派发。
tools: Read, Bash
model: sonnet
omitClaudeMd: true
---

# 门禁分诊（gate-triage）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只分诊，不修。
开工先读：`.claude/singlefs-ai-sop/skills/gate/SKILL.md`（阶段含义、常见假失败）；`.claude/singlefs-ai-sop/rules/session-wrapup.md`「4. 同一个仓里有没有别的会话在飞？」。

```

**出处 `.claude/agents/gate-triage.md:22-28`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
2. 暂存过的跑 `nice -n 19 bash .claude/scripts/gate.sh --staged`；主 agent 明写全量的跑不带参数。超过 Bash 单次上限时后台跑、结束后读输出。
3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。

```

**出处 `.claude/agents/three-way-local-attack.md:1-15`（整段抄，未转述）**

```markdown
---
name: three-way-local-attack
description: 三方论证的本地攻方腿：把攻方问题译成英文、驱动本机本地模型作答并过损坏闸。只在主 agent 点名派发、并给出分给本地攻方的攻击面时用；不要自动派发。
tools: Read, Bash
model: sonnet
omitClaudeMd: true
---

# 本地攻方腿（three-way-local-attack）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

推论出自本机本地模型，你负责把问题忠实地交给它、把它的答复原样收回来。不替它补推理，也不判它答得对不对。
开工先读：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」「一条腿只抽一次样不算一次观测——否定结论尤其不算」「本地腿缺席时必须显式报告」「给本地腿的提示一律用英文」。

```

**出处 `.claude/agents/three-way-local-attack.md:23-31`（整段抄，未转述）**

```markdown
## 做什么

1. 把攻击面写成英文提示：自足、不用任何 markdown 强调、答案按编号、每条答复写「什么现象会推翻它」。能落成数的问题给写死的事实表（数整行抄，核对表里写来源文件:行），要模型按表格逐格填，不许只答 yes / no。提示里明令答复不写代码行号与文件行号（按函数名、表格行号指），核对表与运行记录里把样本自带的行号一律标「模型自给、未核」。
2. 逐句核转述：每一句英文与原文并排，缺一个限定词就补上；英文比原文多出来的限定词、括注也单列一行，写明为什么加。核对表写进 `research/prompts/<轮>-local-attack-translation-audit.md`（英文项 / 原文文件:行 / 首稿缺的 / 定稿）。
3. 前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`。
4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，沿用同一个号重跑；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5、或网关不通：停下，报「本地腿缺席」。
5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本>` 看它列出的生词：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
6. 攒到至少两份干净样本为止；连续五次调用（判红作废的也算）拿不到两份干净的，停下照实报。

```

**出处 `records/2026-09-16-subagent拆分提案.md:751-761`（整段抄，未转述）**

```markdown
## 三十三、TaskStop 停掉在等后台任务的子 agent，看门狗认不出「被停」（2026-09-18）

提交前派 `gate-triage` 跑全量门禁（`--staged`），它 13:58:29 UTC 结束本轮、等后台的 `gate.sh`；主 agent 14:01:49 用 TaskStop 停掉它（层 0 崩溃点重放已由崩溃验证员在同一份 `crates/` 上跑过，用户定不重跑）。子 agent 的会话记录在被停之后一个字都没写，只有主会话记录里有 TaskStop 的成功结果；`agent-watch.py` 只认子 agent 会话记录里的 `[Request interrupted`，于是一直当它还活着，14:12:14 离它最后一条记录 13 分 45 秒，报「无动静」、退出码 3，还提示「再起一个看门狗」——那是一次误报。

| 查出什么 | 怎么核的 | 去向 |
|---|---|---|
| 看门狗判「被停」只看子 agent 会话记录 | 子 agent 会话记录最后一条 13:58:29；主会话记录里 TaskStop 的结果 14:01:49，`toolUseResult` 带 `task_id` | 已改：`research/scripts/agent-watch.py` 另读主会话记录（`<会话 id>.jsonl`）里 TaskStop 对它的成功结果，时刻不早于它最后一条记录减 30 秒就判被停；自检加「停在等后台任务时」「停了又被续做」两个样本，`AGENT_WATCH_BREAK=taskstop` 判红；对这一次的子 agent 重跑 report 报「状态=被停」、没有告警 |
| 主 agent 收到的「completed」通知里写 "This agent has not reported yet: it is waiting on its own background work…"，用户看成出错 | 那是 harness 在子 agent 结束本轮、后台任务还在跑时发的中间通知，不是交回也不是出错；这一次收到两次（子 agent 13:53:44、13:58:29 各结束一次本轮） | 没改：`.claude/main-agent.md`「交回怎么读」没写这种通知怎么读，要补一句；改定义要走一轮三方（`.claude/rules/implementation-workflow.md`），交用户定 |
| 子 agent 用 `run_in_background` 起命令时又在命令里加了 `nohup … & disown`：harness 的后台任务当场结束，完成通知跟 `gate.sh` 脱钩；三个缓存计时器也这样起，到点没叫醒它；之后连调五次空命令想结束本轮 | 子 agent 会话记录 13:52:19–13:53:44；三份计时器日志 13:56 写出、会话里没有对应的唤醒；13:53:20 改成不带 `&` 起的那一个 13:57:12 叫醒了它 | 没改：共用约束「长活可以等」一条没写「`run_in_background` 起的命令里不加 `&`、`nohup`、`disown`」，Bash 检出 hook 也不查这一形态；改共用约束要走三方，hook 可以直接加，交用户定 |
| 同一次排查里 `research/scripts/check-staged.sh` 把门禁 72 号的退出码 77（这次改动没碰定义、无对象可判）报成 RED | 同一棵树上单跑 72 号退 77、写着「本阶段无对象可判」；check-staged 的判法是「非 0 即红」 | 已改：77 单列「无对象可判，没跑」、成功行列名，点名的阶段全是 77 时整体退 77；自检加一格，`CHECK_STAGED_77_IS_RED=1` 判红 |

```

**出处 `.claude/hooks/bash-command-detector.sh:1-127`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；一律放行，不拦、不停任何命令与脚本。
#
# 为什么：2026-09-17 一个实验执行员用 `pgrep -f` 找复跑进程，接着在一个等不到的 `until` 循环里空转，
# 主 agent 查进度才发现（records/2026-09-17-已分配口径三方与两个实验.md 第六节）。用户定：hook 不能终止任务和脚本，
# 只负责检出问题，交给主 agent 去判断；子 agent 与脚本结不结束由主 agent 定。
#
# 检出三种，写进检出记录（默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
#   ① 按模式匹配的进程命令（`pgrep -f`、`pkill -f`、`killall`）：模式串会命中自己所在的 shell（command-safety.md）；
#   ② 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等；
#   ③ `run_in_background` 起的命令里又自己放后台（`nohup`、`setsid`、`disown`、后面没有 `wait` 的单独 `&`）：外层 shell 当场退出，
#      harness 的完成通知当场发出，真正在跑的东西跑完不再叫醒谁（2026-09-18 一个门禁分诊员这样起门禁与三个缓存计时器，
#      records/2026-09-16-subagent拆分提案.md 第三十三节）。
# 主 agent 的看门狗（research/scripts/agent-watch.py watch）每 15 秒读一次检出记录，读到本会话的检出就退出、叫醒主 agent。
# 只看顶层命令文本；命令里只是带着这些字（heredoc 里的测试数据）也会被记一条，主 agent 看了判断即可。
#
#   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON，永远退出 0
#   bash-command-detector.sh --selftest  # 走一遍检出与不检出；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1 或
#                                        # BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1（只关第三种）时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, subprocess, sys, tempfile
from datetime import datetime, timezone

PATTERN_PROCESS_COMMAND = re.compile(r"\bpgrep\s+(?:-\w+\s+)*-\w*f|\bpkill\s+(?:-\w+\s+)*-\w*f|\bkillall\b")
WAIT_LOOP = re.compile(r"\b(until|while)\b[^\n]*?;\s*do\b[\s\S]*?\bsleep\b")
SELF_DETACH = re.compile(r"\bnohup\b|\bsetsid\b|\bdisown\b")
# 单独的 `&`：不是 `&&`、`>&`、`&>`、`|&`、`<&`（重定向与逻辑与都不算放后台）；它后面有 `wait` 的是外层等着它，不算
LONE_AMPERSAND = re.compile(r"(?<![&>|<])&(?![&>])")

def puts_itself_in_background(command):
    if SELF_DETACH.search(command):
        return True
    return any(not re.search(r"\bwait\b", command[match.end():]) for match in LONE_AMPERSAND.finditer(command))
DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"

def findings_for(command, run_in_background=False):
    if os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1":
        return []
    findings = []
    if PATTERN_PROCESS_COMMAND.search(command):
        findings.append("按模式匹配的进程命令（pgrep -f / pkill -f / killall）")
    if WAIT_LOOP.search(command) and not re.search(r"\btimeout\b", command):
        findings.append("没有 timeout 的等待循环")
    if run_in_background and puts_itself_in_background(command) and os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND") != "1":
        findings.append("run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你")
    return findings

def record(hook_input, detections_path):
    """检出就追加一行；返回写了几行。永远不拦命令。"""
    tool_input = hook_input.get("tool_input") or {}
    command = tool_input.get("command") or ""
    findings = findings_for(command, bool(tool_input.get("run_in_background")))
    if not findings:
        return 0
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": command[:500],
        "findings": findings,
    }
    os.makedirs(os.path.dirname(detections_path), exist_ok=True)
    with open(detections_path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    return 1

def selftest(hook_dir):
    work = tempfile.mkdtemp(prefix="bash-command-detector-")
    detections = os.path.join(work, "detections.jsonl")
    cases = [
        ("普通命令", "cargo test --release", 0),
        ("带 timeout 的等待循环", "timeout 600 bash -c 'until grep -q done log; do sleep 10; done'", 0),
        ("while read 读文件", 'while read -r line; do echo "$line"; done < list.txt', 0),
        ("grep 里出现 pgrep 这个词", "grep -n 'pgrep' notes.md", 0),
        ("没有 timeout 的 until 等待", 'until grep -q "^exit=" log; do sleep 10; done', 1),
        ("pgrep -f", "pgrep -f e154-binary", 1),
        ("pkill -f", "pkill -f cargo", 1),
        ("killall", "killall cargo", 1),
        ("后台起的 nohup … & 加 disown", "nohup nice -n 19 bash gate.sh > gate.log 2>&1 & echo $!; disown", 1, True),
        ("后台起的命令末尾单独一个 &", "bash cache-keepalive.sh > keepalive.log 2>&1 &", 1, True),
        ("后台起的命令只有 2>&1、|& 与 &&", "cargo build 2>&1 | tail -3 && echo ok; make |& tee out", 0, True),
        ("后台起的普通命令", "bash cache-keepalive.sh", 0, True),
        ("后台起的并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0, True),
        ("后台起的 & 在 wait 之后", "wait; bash late.sh &", 1, True),
        ("前台命令里的 & 不算（不是 run_in_background）", "sleep 1 & wait", 0),
    ]
    results = []
    for label, command, want, *background in cases:
        tool_input = {"command": command, "run_in_background": bool(background and background[0])}
        written = record({"tool_name": "Bash", "session_id": "s", "tool_input": tool_input}, detections)
        results.append((label, want, written))
    # 走真实入口：从标准输入喂 JSON，退出码必须是 0（不拦），检出要落进文件
    script = os.path.join(hook_dir, "bash-command-detector.sh")
    before = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    completed = subprocess.run(["bash", script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "pgrep -f x"}}),
                               capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    results.append(("stdin:检出也放行（退出码 0）", 0, completed.returncode))
    results.append(("stdin:检出落进文件", 1, after - before))
    subprocess.run(["rm", "-rf", work])
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 findings_for() 与 record()；BASH_COMMAND_DETECTOR_DISABLE_CHECK 或 _DISABLE_SELF_BACKGROUND 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过：按模式找进程、没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 与前台的 & 不记，入口一律放行（查了 {len(results)} 种）")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
        record(hook_input, os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
    except Exception:
        pass
    return 0

sys.exit(main())
PY
```

**出处 `research/scripts/agent-watch.py:1-824`（整段抄，未转述）**

```markdown
#!/usr/bin/env python3
"""子 agent 与它们起的进程的定时监控：卡死、空转、永久等待当场报出来。

为什么：2026-09-17 一个实验执行员把 `echo "exit=$?"` 打到标准输出、却在日志里 `until grep -q "^exit="`，
等一行永远不会出现的字，主 agent 查进度才发现；同一天另一个会话的层 0 全量测试跑了三个多小时没有任何输出，
没人分得清是慢还是卡。经过：records/2026-09-17-已分配口径三方与两个实验.md 第六节。

用法：
    agent-watch.py report --agents ID[,ID…]          # 看一眼：每个子 agent 的状态、最后一条命令、告警
    agent-watch.py report --session-dir DIR           # 看这个会话里最近活动过的全部子 agent
    agent-watch.py watch  --agents ID[,ID…]          # 看门狗：每隔一段时间查一次，有告警或全部结束就退出
    agent-watch.py cost   --agents ID[,ID…]          # 用量：调用次数、起步与平均上下文、新输入、读缓存、输出、整份重写、工具结果被什么撑大
    agent-watch.py --selftest                         # 造假的会话记录与进程走一遍各种告警；AGENT_WATCH_BREAK=<项> 时必须判红

主 agent 派发之后用 Bash 的 run_in_background 起 watch：它一退出，harness 就会叫醒主 agent。
它只看、只报，不停任何子 agent、不杀任何进程：子 agent 与脚本不强制结束，结束不结束由主 agent 看了报告定（用户 2026-09-17）。
退出码：0 被看的子 agent 全部交回或被停、没有告警；3 有告警；4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）；2 用法错。
「交回」按交回工具（SubagentHandback）成功返回判：只结束本轮、没交回的子 agent 还在等后台任务（缓存续命见 research/scripts/cache-keepalive.sh），不算结束。
「被停」认两处：子 agent 会话记录里的 `[Request interrupted`（停在工具调用中途）；主会话记录（<会话 id>.jsonl）里 TaskStop 对它的成功结果，
时刻不早于它自己最后一条记录减 30 秒（停在结束本轮、等后台任务的时候，子 agent 的会话记录里一个字都不写；停了之后又被续做的不算）。

hook 的检出也在这里汇给主 agent：`.claude/hooks/bash-command-detector.sh` 只记不拦，把检出写进检出记录；
watch 每 15 秒读一次，读到被看子 agent 所在会话的检出就退出、叫醒主 agent。

会话记录的位置：~/.claude/projects/<项目>/<会话 id>/subagents/agent-<id>.jsonl，同名 .meta.json 里有 agentType 与 description。
进程这一半只看运行这个脚本的那个 Claude 实例的后代进程（主 agent 与它派出的子 agent 起的命令都在里面），不看别的会话。
"""
import argparse
import glob
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from zoneinfo import ZoneInfo

PROJECTS_ROOT = os.path.expanduser("~/.claude/projects")
DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"
WAIT_LOOP_PATTERN = re.compile(r"\b(until|while)\b[\s\S]*\bsleep\b")
BANNED_COMMAND_PATTERN = re.compile(r"\bpgrep\s+-f\b|\bpkill\s+-f\b|\bkillall\b")
BROKEN_DETECTION = os.environ.get("AGENT_WATCH_BREAK", "")
STOP_AFTER_LAST_RECORD_SECONDS = 30  # 停在工具中途时，被停之后还可能落几条收尾记录


def parse_timestamp(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00"))


def parent_session_transcript(transcript_path):
    """…/<会话 id>/subagents/agent-<id>.jsonl → …/<会话 id>.jsonl（派它的那个会话的记录）。"""
    return os.path.dirname(os.path.dirname(transcript_path)) + ".jsonl"


def task_stop_time(agent_id, session_transcript_path):
    """主会话记录里 TaskStop 停掉这个子 agent 的最晚一次成功结果的时刻；没有就 None。"""
    if not os.path.isfile(session_transcript_path):
        return None
    stopped_at = None
    for line in open(session_transcript_path, encoding="utf-8", errors="replace"):
        if agent_id not in line or "Successfully stopped task" not in line:
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        result = record.get("toolUseResult")
        if not isinstance(result, dict) or result.get("task_id") != agent_id or "Successfully stopped task" not in str(result.get("message", "")):
            continue
        if record.get("timestamp"):
            timestamp = parse_timestamp(record["timestamp"])
            stopped_at = timestamp if stopped_at is None else max(stopped_at, timestamp)
    return stopped_at


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
        self.state = "思考中"
        self._read()
        stopped_at = task_stop_time(agent_id, parent_session_transcript(transcript_path))
        if (stopped_at is not None and BROKEN_DETECTION != "taskstop" and self.state != "已交回"
                and (self.last_timestamp is None or (self.last_timestamp - stopped_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
            self.state = "被停"

    def _read(self):
        seen_message_ids = set()
        pending_by_id = {}
        tool_inputs_by_id = {}
        handback_tool_ids = set()
        is_handed_back = False  # 最近一次交回成功之后没有再调别的工具（续做之后又调工具就回到没交回）
        previous_call_time = None
        last_kind = None
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
                            if block.get("name") == "SubagentHandback":
                                handback_tool_ids.add(block.get("id"))
                            else:
                                is_handed_back = False
                            last_kind = "tool_use"
                        elif block.get("type") == "text":
                            last_kind = "end_turn" if message.get("stop_reason") == "end_turn" else "text"
            elif role == "user":
                if isinstance(content, list):
                    for block in content:
                        if block.get("type") == "tool_result":
                            pending_by_id.pop(block.get("tool_use_id"), None)
                            if block.get("tool_use_id") in handback_tool_ids:
                                result_text = json.dumps(block.get("content"), ensure_ascii=False).replace("\\", "").replace(" ", "")
                                if '"success":true' in result_text:
                                    is_handed_back = True
                            if block.get("tool_use_id") in tool_inputs_by_id:
                                self._count_tool_result(*tool_inputs_by_id[block.get("tool_use_id")], block.get("content"))
                            for entry in self.bash_commands[-20:]:
                                if entry[3] == block.get("tool_use_id"):
                                    result = block.get("content")
                                    entry[2] = result if isinstance(result, str) else json.dumps(result, ensure_ascii=False)
                            last_kind = "tool_result"
                        elif block.get("type") == "text" and "[Request interrupted" in block.get("text", ""):
                            last_kind = "interrupted"
                elif isinstance(content, str) and "[Request interrupted" in content:
                    last_kind = "interrupted"
        self.pending_tool = max(pending_by_id.values(), key=lambda item: item[0]) if pending_by_id else None
        if last_kind == "interrupted":
            self.state = "被停"
        elif is_handed_back and not pending_by_id and BROKEN_DETECTION != "finished":
            self.state = "已交回"
        elif self.pending_tool is not None:
            self.state = "执行工具中"
        elif last_kind == "end_turn" and BROKEN_DETECTION == "handback":
            self.state = "已交回"
        elif last_kind == "end_turn":
            self.state = "本轮结束、没交回"
        else:
            self.state = "思考中"

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
    elif (transcript.last_timestamp is not None and BROKEN_DETECTION != "idle"
          and (now - transcript.last_timestamp).total_seconds() >= thresholds.idle_minutes * 60):
        alerts.append(("无动静", f"最后一条记录在 {format_duration((now - transcript.last_timestamp).total_seconds())} 以前，没有工具在跑",
                       "主 agent 判断：模型调用是不是卡住或撞了限额；看会话记录最后几条，再决定等、发消息续做、还是停掉重派"))
    recent_entries = transcript.bash_commands[-10:]
    recent_commands = [entry[1] for entry in recent_entries]
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
        for command in recent_commands:
            if BANNED_COMMAND_PATTERN.search(command):
                alerts.append(("禁用命令", f"用了按模式匹配的进程命令：{command[:160]}",
                               "pgrep -f / pkill -f 会命中自己所在的 shell（command-safety.md）；主 agent 判断它等的或要杀的是哪个进程，再发消息让它改用写死的 pid"))
                break
    return alerts


def process_table():
    table = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            stat_text = open(f"/proc/{entry}/stat").read()
            command_line = open(f"/proc/{entry}/cmdline", "rb").read().replace(b"\0", b" ").decode(errors="replace").strip()
        except OSError:
            continue
        after_name = stat_text[stat_text.rfind(")") + 2:].split()
        table[int(entry)] = {"parent": int(after_name[1]), "start_ticks": int(after_name[19]),
                             "cpu_ticks": int(after_name[11]) + int(after_name[12]), "command": command_line}
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
            flags_line = [line for line in open(f"/proc/{pid}/fdinfo/{descriptor}") if line.startswith("flags:")][0]
        except (OSError, IndexError):
            continue
        access_mode = int(flags_line.split()[1], 8) & 0o3
        if target.startswith("/") and access_mode in (1, 2) and os.path.isfile(target):
            files.append(target)
    return sorted(set(files))


def process_alerts(root_pid, excluded_pids, thresholds):
    """这个 Claude 实例底下跑得久的叶子进程：报跑了多久、写的文件多久没动；过期的告警。"""
    table = process_table()
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    descendants, frontier = [], [root_pid]
    while frontier:
        pid = frontier.pop()
        for child in children.get(pid, []):
            descendants.append(child)
            frontier.append(child)
    lines, alerts = [], []
    for pid in sorted(descendants):
        if pid in excluded_pids or children.get(pid):
            continue
        info = table[pid]
        if any(marker in info["command"] for marker in ("native-binary/claude", "vscode-server", "/node ", "agent-watch.py")):
            continue
        elapsed = uptime_seconds - info["start_ticks"] / ticks_per_second
        if elapsed < thresholds.process_report_minutes * 60:
            continue
        files = written_files(pid)
        newest_age = min((time.time() - os.path.getmtime(path) for path in files), default=None)
        age_text = "没有写着的文件" if newest_age is None else f"写着的文件最近一次变动在 {format_duration(newest_age)} 以前"
        lines.append(f"  进程 {pid} 已跑 {format_duration(elapsed)}，{age_text}：{info['command'][:140]}")
        if BROKEN_DETECTION != "stale" and newest_age is not None and elapsed >= thresholds.process_stale_minutes * 60 \
                and newest_age >= thresholds.process_stale_minutes * 60:
            alerts.append(("进程无输出", f"进程 {pid} 已跑 {format_duration(elapsed)}，写着的文件 {format_duration(newest_age)} 没变：{info['command'][:140]}",
                           "输出不涨不一定是卡住（有的测试只在结束时写）；主 agent 看它的 CPU 在不在涨、预期多久，再决定接着盯、查、还是报给用户"))
        elif elapsed >= thresholds.process_max_minutes * 60:
            alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}：{info['command'][:140]}",
                           "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
    return lines, alerts


def build_report(agent_ids, session_dir, thresholds, watch_started, excluded_pids, process_root_pid):
    now = datetime.now(timezone.utc)
    lines, alerts, done_flags, active_ids, unstarted_ids = [], [], [], [], []
    if agent_ids:
        targets = [(agent_id, find_transcript(agent_id, session_dir)) for agent_id in agent_ids]
    else:
        paths = glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))
        recent = [path for path in paths if time.time() - os.path.getmtime(path) <= thresholds.active_minutes * 60]
        targets = [(os.path.basename(path)[len("agent-"):-len(".jsonl")], path) for path in sorted(recent)]
    for agent_id, path in targets:
        if path is None:
            waited = time.time() - watch_started
            lines.append(f"子 agent {agent_id}：还没有会话记录（已等 {format_duration(waited)}）")
            done_flags.append(False)
            unstarted_ids.append(agent_id)
            if waited >= thresholds.not_started_minutes * 60:
                alerts.append((agent_id, "没启动", f"派发 {format_duration(waited)} 之后还没有会话记录", "查派发是否失败、agent id 是否抄对"))
            continue
        transcript = AgentTranscript(agent_id, path)
        done_flags.append(transcript.is_done())
        if not transcript.is_done():
            active_ids.append(agent_id)
        since_last = format_duration((now - transcript.last_timestamp).total_seconds()) if transcript.last_timestamp else "?"
        total = format_duration((transcript.last_timestamp - transcript.first_timestamp).total_seconds()) if transcript.first_timestamp else "?"
        lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」状态={transcript.state} 已跑 {total} "
                     f"最后动作 {since_last} 前 调用 {transcript.model_calls} 次 上下文 {transcript.last_context_tokens // 1000}k 缓存整份重写 {transcript.full_cache_rewrites} 次")
        if transcript.pending_tool is not None and not transcript.is_done():
            started, tool_name, detail = transcript.pending_tool
            lines.append(f"  正在跑 {tool_name}（{format_duration((now - started).total_seconds())}）：{' '.join(detail.split())[:200]}")
        for name, explanation, next_step in agent_alerts(transcript, now, thresholds):
            alerts.append((agent_id, name, explanation, next_step))
    if process_root_pid is not None:
        process_lines, found = process_alerts(process_root_pid, excluded_pids, thresholds)
        if process_lines:
            lines.append("这个 Claude 实例底下跑得久的进程：")
            lines.extend(process_lines)
        for name, explanation, next_step in found:
            alerts.append(("进程", name, explanation, next_step))
    return lines, alerts, bool(done_flags) and all(done_flags), (active_ids, unstarted_ids)


def session_id_of(transcript_path):
    return os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))


def read_detections(detections_path, start_offset, session_ids):
    """从 start_offset 往后读检出记录，返回 (新的偏移, [属于这些会话的告警])。"""
    alerts = []
    if not os.path.isfile(detections_path):
        return start_offset, alerts
    with open(detections_path, encoding="utf-8", errors="replace") as handle:
        handle.seek(start_offset)
        for line in handle:
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            if BROKEN_DETECTION == "detections" or entry.get("session_id") not in session_ids:
                continue
            findings = [str(finding) for finding in (entry.get("findings") or [])]
            if findings and all(finding.startswith("没有 timeout 的等待循环") for finding in findings) and BROKEN_DETECTION != "loopnoise":
                # 只检出等待循环的不立刻叫醒：正常等编译、等全量跑也是这个形状。会话记录那一路在它跑满 --wait-loop-minutes 时再报。
                continue
            alerts.append((entry.get("agent_type") or "?", "hook 检出",
                           f"{'、'.join(entry.get('findings') or [])}：{(entry.get('command') or '')[:160]}",
                           ("这次写被 hook 拒了、没写进去；主 agent 判断是不是该写的，再决定发消息让它改、自己改、还是派该写的 agent"
                            if any(str(finding).startswith("写被拒") for finding in (entry.get("findings") or []))
                            else "检出 hook 只记不拦，命令照常在跑；主 agent 判断它会不会出问题，再决定接着盯、发消息让它改、还是处理")))
        return handle.tell(), alerts


def print_report(lines, alerts):
    now = datetime.now(timezone.utc)
    print(f"[agent-watch {now.strftime('%H:%M:%S')} UTC / {now.astimezone(ZoneInfo('Asia/Tokyo')).strftime('%H:%M:%S')} JST]")
    for line in lines:
        print(line)
    for owner, name, explanation, next_step in alerts:
        print(f"⚠️ {name}（{owner}）：{explanation}")
        print(f"   → 怎么办：{next_step}")
    if not alerts:
        print("没有告警。")


def print_active_list(active_ids, unstarted_ids):
    if not active_ids and not unstarted_ids:
        return
    print(f"还没交回、没被停的子 agent（{len(active_ids)} 个，另有还没会话记录的 {len(unstarted_ids)} 个）：{','.join(active_ids + unstarted_ids)}")
    print(f"   → 接着盯就再起一个看门狗 --agents {','.join(active_ids + unstarted_ids)}")


def own_process_chain(table, start_pid):
    chain, pid = set(), start_pid
    while pid in table and pid > 1:
        chain.add(pid)
        pid = table[pid]["parent"]
    return chain


def run(arguments):
    if not arguments.agents and not arguments.session_dir:
        print("✗ 要给 --agents 或 --session-dir\n→ 怎么办：派发返回的 agent id 用逗号连起来传 --agents", file=sys.stderr)
        return 2
    agent_ids = [item for item in (arguments.agents or "").split(",") if item]
    table = process_table()
    if arguments.process_root_pid == 0:
        process_root_pid = None
    elif arguments.process_root_pid is not None:
        process_root_pid = arguments.process_root_pid
    else:
        process_root_pid = claude_instance_pid(table, os.getpid())
    excluded_pids = own_process_chain(table, os.getpid())
    if arguments.mode == "cost":
        lines, _ = cost_report(agent_ids, arguments.session_dir, arguments)
        print("\n".join(lines))
        return 0
    watch_started = time.time()
    detections_path = arguments.detections_file
    detections_offset = os.path.getsize(detections_path) if (arguments.mode == "watch" and os.path.isfile(detections_path)) else 0
    report_deadline = watch_started + arguments.max_minutes * 60
    while True:
        lines, alerts, all_done, active_lists = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
        session_ids = {session_id_of(path) for path in (find_transcript(agent_id, arguments.session_dir) for agent_id in agent_ids) if path}
        if arguments.session_dir:
            session_ids.add(os.path.basename(os.path.normpath(arguments.session_dir)))
        detections_offset, detection_alerts = read_detections(detections_path, detections_offset, session_ids)
        alerts += detection_alerts
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            print_report(lines, alerts)
            print_active_list(*active_lists)
            return 3
        if all_done:
            print_report(lines, alerts)
            print("被看的子 agent 全部交回或被停。")
            return 0
        if time.time() >= report_deadline:
            print_report(lines, alerts)
            print(f"定时回报：已经看了 {arguments.max_minutes:g} 分钟，没有告警。")
            print_active_list(*active_lists)
            return 4
        next_full_check = time.time() + arguments.interval_seconds
        if BROKEN_DETECTION != "deadline":
            next_full_check = min(next_full_check, report_deadline)
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(next_full_check - time.time(), 0)))
            detections_offset, detection_alerts = read_detections(detections_path, detections_offset, session_ids)
            if detection_alerts:
                lines, alerts, _, active_lists = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
                print_report(lines, alerts + detection_alerts)
                print_active_list(*active_lists)
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


def record_at(minutes_ago, role, content, stop_reason=None, message_id=None, usage=None):
    stamp = datetime.fromtimestamp(iso_minutes_ago(minutes_ago), timezone.utc).isoformat().replace("+00:00", "Z")
    message = {"role": role, "content": content}
    if stop_reason:
        message["stop_reason"] = stop_reason
    if message_id:
        message["id"] = message_id
    if usage:
        message["usage"] = usage
    return {"timestamp": stamp, "type": role, "message": message}


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


def selftest():
    work = tempfile.mkdtemp(prefix="agent-watch-selftest-")
    thresholds = argparse.Namespace(tool_minutes=8, wait_loop_minutes=3, idle_minutes=10, repeat_count=3,
                                    process_report_minutes=0.02, process_stale_minutes=0.05, process_max_minutes=600,
                                    not_started_minutes=5, active_minutes=600)
    session = os.path.join(work, "session")
    failures = []
    write_transcript(session, "finished", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18.5, "h1", "m2"),
                                           record_at(18, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                     {"agentType": "experiment-runner", "description": "跑完的"})
    write_transcript(session, "waiting", [bash_use(1, "t1", "cargo test --release", "m1"),
                                          record_at(0.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: b1."}]),
                                          record_at(0.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "experiment-runner", "description": "结束本轮在等后台任务、没交回"})
    write_transcript(session, "continued", [*handback_records(3, "h1", "m1"),
                                            record_at(2.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m2"),
                                            record_at(2, "user", "续做：再补一格"), bash_use(1, "t2", "ls", "m3"), bash_result(0.9, "t2")],
                     {"agentType": "experiment-runner", "description": "交回之后被续做"})
    write_transcript(session, "waitloop", [bash_use(5, "t1", 'until grep -q "^exit=" log; do sleep 10; done', "m1")])
    write_transcript(session, "longtool", [bash_use(20, "t1", "cargo test --release", "m1")])
    write_transcript(session, "healthy", [bash_use(0.5, "t1", "cargo build", "m1")])
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
    write_transcript(session, "idle", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1")])
    write_transcript(session, "interrupted", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1"),
                                              record_at(39, "user", [{"type": "text", "text": "[Request interrupted by user]"}])])
    waiting_then_idle = [bash_use(25, "t1", "bash gate.sh --staged", "m1"), bash_result(24.9, "t1"),
                         record_at(24.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
    write_transcript(session, "stoppedidle", waiting_then_idle,
                     {"agentType": "gate-triage", "description": "结束本轮等后台任务时被 TaskStop 停掉"})
    write_transcript(session, "stoppedcontinued", [*waiting_then_idle, record_at(2, "user", "续做：再跑一遍"), bash_use(0.5, "t2", "cargo build", "m3")],
                     {"agentType": "gate-triage", "description": "被停之后又被续做"})
    write_session_stops(session, {"stoppedidle": 20, "stoppedcontinued": 20})
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
    for field, wanted in wanted_cost.items():
        got = getattr(costed, field)
        if BROKEN_DETECTION == "cost" and field == "full_cache_rewrite_tokens":
            got = 0
        if got != wanted:
            failures.append(f"用量样本的 {field} 应当是 {wanted}，实际 {got}")
    if costed.tool_result_characters.get("Read 跑前登记（整份）") != [1, 400] or costed.tool_result_characters.get("Bash cargo") != [1, 100]:
        failures.append(f"工具结果分类不对：{costed.tool_result_characters}")
    expectations = {
        "finished": set(), "healthy": set(), "boundedloop": set(), "interrupted": set(), "repeatchanging": set(), "waiting": set(), "continued": set(),
        "stoppedidle": set(), "stoppedcontinued": set(),
        "waitloop": {"等待循环"}, "longtool": {"工具调用过长"}, "repeat": {"同一命令反复且输出不变"}, "banned": {"禁用命令"}, "idle": {"无动静"},
    }
    now = datetime.now(timezone.utc)
    for agent_id, wanted in expectations.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        got = {name for name, _, _ in agent_alerts(transcript, now, thresholds)}
        if got != wanted:
            failures.append(f"子 agent {agent_id}：应当告警 {sorted(wanted) or '无'}，实际 {sorted(got) or '无'}")
    wanted_done = {"finished": True, "interrupted": True, "stoppedidle": True, "waiting": False, "continued": False, "healthy": False,
                   "stoppedcontinued": False}
    for agent_id, wanted in wanted_done.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        if transcript.is_done() != wanted:
            failures.append(f"子 agent {agent_id} 应当{'算' if wanted else '不算'}交回或被停，实际状态「{transcript.state}」")
    stale_file = os.path.join(work, "stale-output.log")
    open(stale_file, "w").close()
    old = time.time() - 3600
    os.utime(stale_file, (old, old))
    sleeper = subprocess.Popen(["bash", "-c", f"exec 3>>'{stale_file}'; touch -d '1 hour ago' '{stale_file}'; exec sleep 30"])
    try:
        time.sleep(4)
        _, found = process_alerts(os.getpid(), set(), thresholds)
        if not any(name == "进程无输出" and str(sleeper.pid) in explanation for name, explanation, _ in found):
            failures.append(f"写着的文件一小时没动的进程 {sleeper.pid} 没报「进程无输出」")
    finally:
        sleeper.kill()
        sleeper.wait()
    watch_runs = []

    def run_watch(watch_arguments, timeout=None):
        watch_runs.append(watch_arguments)
        return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", *watch_arguments, "--session-dir", session, "--process-root-pid", "0"],
                              capture_output=True, text=True, timeout=timeout)

    watch_exit = run_watch(["--agents", "finished,interrupted,stoppedidle", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
    if watch_exit != 0:
        failures.append(f"被看的子 agent 都交回或被停时看门狗应当退出码 0，实际 {watch_exit}")
    watch_exit = run_watch(["--agents", "waitloop", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
    if watch_exit != 3:
        failures.append(f"有告警时看门狗应当退出码 3，实际 {watch_exit}")
    watch_exit = run_watch(["--agents", "healthy", "--interval-seconds", "1", "--max-minutes", "0.03"]).returncode
    if watch_exit != 4:
        failures.append(f"到定时回报的时刻看门狗应当退出码 4，实际 {watch_exit}")
    watched = run_watch(["--agents", "waiting,finished", "--interval-seconds", "1", "--max-minutes", "0.03"])
    wanted_active_line = "还没交回、没被停的子 agent（1 个，另有还没会话记录的 0 个）：waiting"
    if watched.returncode != 4 or wanted_active_line not in watched.stdout:
        failures.append(f"一个结束本轮没交回、一个已交回时看门狗应当退出码 4 并列出「{wanted_active_line}」，"
                        f"实际退出码 {watched.returncode}，输出：{watched.stdout[-300:]}")
    try:
        started = time.time()
        watched = run_watch(["--agents", "waiting", "--interval-seconds", "240", "--max-minutes", "0.05"], timeout=30)
        if watched.returncode != 4:
            failures.append(f"复检间隔 240 秒、回报周期 3 秒时看门狗应当到点退出码 4，实际 {watched.returncode}")
    except subprocess.TimeoutExpired:
        failures.append(f"复检间隔比回报周期长时看门狗没有按回报周期退出（{time.time() - started:.0f} 秒还没退出）：回报会被拖到下一次复检")
    detections_file = os.path.join(work, "detections.jsonl")
    open(detections_file, "w").close()
    for session_label, finding_text, wanted_exit in (("session", "按模式匹配的进程命令", 3), ("另一个会话", "按模式匹配的进程命令", 4),
                                                     ("session", "没有 timeout 的等待循环", 4)):
        watch_runs.append(["--detections-file", session_label, finding_text])
        watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--agents", "healthy", "--session-dir", session,
                                    "--interval-seconds", "2", "--detection-poll-seconds", "0.5", "--max-minutes", "0.12",
                                    "--process-root-pid", "0", "--detections-file", detections_file],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        time.sleep(1.5)
        with open(detections_file, "a", encoding="utf-8") as handle:
            handle.write(json.dumps({"session_id": session_label, "agent_type": "experiment-runner",
                                     "command": "pgrep -f x", "findings": [finding_text]}, ensure_ascii=False) + "\n")
        output, _ = watcher.communicate(timeout=60)
        if watcher.returncode != wanted_exit or (wanted_exit == 3 and "hook 检出" not in output):
            failures.append(f"检出记录属于「{session_label}」、检出「{finding_text}」时看门狗应当退出码 {wanted_exit}，实际 {watcher.returncode}")
    subprocess.run(["rm", "-rf", work])
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("    → 看 agent_alerts() / process_alerts() / run() 的判法；AGENT_WATCH_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静、进程无输出六种告警都报得出，"
          f"交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具成功判（只结束本轮、交回后又被续做的都不算），"
          f"被停认会话记录里的打断与主会话记录里的 TaskStop（停在等后台任务时的算、停了又被续做的不算），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒，用量与工具结果分类对（查了 {len(expectations) + 1} 个子 agent、1 个进程、{len(watch_runs)} 次看门狗）")
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
    parser.add_argument("--process-report-minutes", type=float, default=5, help="进程跑过这么久才列出来，默认 5 分钟")
    parser.add_argument("--process-stale-minutes", type=float, default=20, help="进程写着的文件这么久没动告警，默认 20 分钟")
    parser.add_argument("--process-max-minutes", type=float, default=120, help="进程跑过这么久告警，默认 120 分钟")
    parser.add_argument("--not-started-minutes", type=float, default=5, help="派发之后这么久还没有会话记录告警，默认 5 分钟")
    parser.add_argument("--active-minutes", type=float, default=180, help="只给 --session-dir 时，只看这么久以内动过的子 agent")
    parser.add_argument("--detections-file", default=DEFAULT_DETECTIONS, help="hook 的检出记录，默认 /tmp/claude-1000/agent-hook-detections.jsonl")
    parser.add_argument("--detection-poll-seconds", type=float, default=15, help="watch 两次读检出记录的间隔，默认 15 秒")
    parser.add_argument("--process-root-pid", type=int, help="进程一半从哪个 pid 往下看；默认是跑这个脚本的 Claude 实例，0 表示不看进程")
    arguments = parser.parse_args()
    return run(arguments)


if __name__ == "__main__":
    sys.exit(main())
```

**出处 `research/scripts/check-staged.sh:1-154`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 只拿「HEAD + 暂存区」跑门禁阶段：几个会话共写一个仓时，工作区里混着别人没收尾的改动与未跟踪文件，
# 门禁在工作区上判的红分不清是谁的。这里在临时 worktree 上套 `git diff --cached`，别人的东西一样都不进来。
#
#   bash research/scripts/check-staged.sh              # 默认：doc-lint + 快的 kb 阶段（不含构建、复跑、外部引用）
#   bash research/scripts/check-staged.sh doc 34 60    # 只跑 doc-lint 与这几号阶段
#   bash research/scripts/check-staged.sh --selftest
#
# 规范副本 .claude/singlefs-ai-sop/ 不进 git，worktree 里没有它；有就原样拷进去，doc-lint 与链接检查才跑得起来。
# 自证会红：--selftest 在临时仓里放一个「文件里有 BAD 就红」的阶段，确认工作区里没暂存的 BAD 不算、暂存了的 BAD 判红；
# 再用 CHECK_STAGED_USE_WORKTREE=1 改成拿工作区原样去跑，确认没暂存的 BAD 也被算进来、selftest 判红；
# CHECK_STAGED_NO_TRAP=1 关掉打断时的清理，确认 selftest 判红（临时 worktree 留在仓里）；
# CHECK_STAGED_77_IS_RED=1 把退出码 77（无对象可判）照旧算红，确认 selftest 判红。
set -uo pipefail

DEFAULT_STAGES=(doc 10 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 42 43 44 45 50 60 61 85 86)

# 跑到一半被打断（Ctrl-C）或被杀时也要删掉临时 worktree：留下的会一直登记在仓里，下一次还得手工 git worktree prune。
cleanup_isolated() {
  if [[ -n "${ISOLATED_WORKTREE:-}" ]]; then git -C "$ISOLATED_REPO" worktree remove --force "$ISOLATED_WORKTREE" >/dev/null 2>&1; fi
  if [[ -n "${ISOLATED_BASE:-}" ]]; then rm -rf "${ISOLATED_BASE:?}"; fi
  ISOLATED_WORKTREE=""; ISOLATED_BASE=""
}

run_isolated() {
  local repo="$1"; shift
  local stages=("$@")
  local base wt patch red=0 ran=0
  local -a not_run=()
  base="$(mktemp -d)"; wt="$base/wt"; patch="$base/staged.patch"
  git -C "$repo" diff --cached --binary > "$patch" || { echo "  ✗ 取不到暂存区的 diff"; echo "    → 在仓里跑，并确认 git 可用"; rm -rf "${base:?}"; return 2; }
  git -C "$repo" worktree add --detach "$wt" HEAD >/dev/null 2>&1 || { echo "  ✗ 建临时 worktree 失败"; echo "    → git worktree prune 之后再试"; rm -rf "${base:?}"; return 2; }
  ISOLATED_REPO="$repo"; ISOLATED_BASE="$base"; ISOLATED_WORKTREE="$wt"
  if [[ "${CHECK_STAGED_NO_TRAP:-0}" != 1 ]]; then
    trap 'cleanup_isolated; exit 130' INT
    trap 'cleanup_isolated; exit 143' TERM
  fi
  if [[ "${CHECK_STAGED_USE_WORKTREE:-0}" == 1 ]]; then
    (cd "$repo" && git ls-files -z) | while IFS= read -r -d '' tracked; do
      [[ -f "$repo/$tracked" ]] && mkdir -p "$wt/$(dirname "$tracked")" && cp "$repo/$tracked" "$wt/$tracked"
    done
  elif [[ -s "$patch" ]]; then
    git -C "$wt" apply --index "$patch" || { echo "  ✗ 暂存区的 diff 套不上 HEAD"; echo "    → 先 git status 看暂存区是不是基于当前 HEAD"; trap - INT TERM; cleanup_isolated; return 2; }
  fi
  if [[ -d "$repo/.claude/singlefs-ai-sop" && ! -d "$wt/.claude/singlefs-ai-sop" ]]; then
    cp -r "$repo/.claude/singlefs-ai-sop" "$wt/.claude/"
  fi
  local stage matched script out rc
  for stage in "${stages[@]}"; do
    if [[ "$stage" == doc ]]; then
      if [[ ! -f "$wt/.claude/singlefs-ai-sop/scripts/doc-lint.sh" ]]; then
        echo "  ! doc-lint 没跑：worktree 里没有规范副本（这一项没验，不是通过）"; continue
      fi
      out="$(cd "$wt" && bash .claude/singlefs-ai-sop/scripts/doc-lint.sh 2>&1)"; rc=$?; ran=$((ran + 1))
      if [[ $rc -eq 0 ]]; then echo "  ok  doc-lint"; else red=$((red + 1)); echo "  RED doc-lint"; echo "$out" | grep -E "✗|→" | head -8; fi
      continue
    fi
    matched=0
    for script in "$wt"/.claude/gate.d/"$stage"-*.sh; do
      [[ -f "$script" ]] || continue
      matched=1; ran=$((ran + 1))
      out="$(cd "$wt" && bash "$script" 2>&1)"; rc=$?
      # 77 = 这一轮无对象可判（show-me-test.md「门禁不许假装通过」）：不算红，也不算通过，单独列名
      if [[ $rc -eq 77 && "${CHECK_STAGED_77_IS_RED:-0}" != 1 ]]; then
        not_run+=("$(basename "$script")"); echo "  77  $(basename "$script")（无对象可判，没跑）"
      elif [[ $rc -eq 0 ]]; then echo "  ok  $(basename "$script")"; else red=$((red + 1)); echo "  RED $(basename "$script")"; echo "$out" | grep -E "✗|→" | head -8; fi
    done
    [[ $matched -eq 1 ]] || echo "  ! 阶段号 $stage 没有对应的 .claude/gate.d/$stage-*.sh（没跑）"
  done
  trap - INT TERM
  cleanup_isolated
  if [[ $ran -eq 0 ]]; then
    echo "  ✗ 一个阶段都没跑到"
    echo "    → 阶段号写成 .claude/gate.d/ 下文件名的前缀（例：34），doc-lint 写 doc"
    return 1
  fi
  local not_run_note=""
  [[ ${#not_run[@]} -gt 0 ]] && not_run_note="；无对象没跑 ${#not_run[@]} 个：${not_run[*]}"
  if [[ $red -gt 0 ]]; then
    echo "  ✗ HEAD + 暂存区上跑了 $ran 个阶段，红 $red 个${not_run_note}"
    echo "    → 红的是这一次提交带进来的（别人的未提交改动不在这棵树里）；先查它在不在 HEAD 上就红：暂存区清空再跑一次"
    return 1
  fi
  if [[ ${#not_run[@]} -eq $ran ]]; then
    echo "  ! HEAD + 暂存区上点名的 $ran 个阶段全是无对象可判，一个都没判（不记通过）"
    return 77
  fi
  echo "  ✓ HEAD + 暂存区上跑了 $ran 个阶段，判了的 $((ran - ${#not_run[@]})) 个全绿${not_run_note}"
  return 0
}

selftest() {
  local repo rc_clean rc_bad
  repo="$(mktemp -d)"
  git -C "$repo" init -q && git -C "$repo" config user.email selftest@example.invalid && git -C "$repo" config user.name selftest
  mkdir -p "$repo/.claude/gate.d" "$repo/kb"
  printf '#!/usr/bin/env bash\n# gate-stage: selftest\nif grep -rq BAD kb/; then echo "  ✗ kb 里有 BAD"; echo "    → 删掉"; exit 1; fi\nexit 0\n' > "$repo/.claude/gate.d/20-selftest.sh"
  echo "干净" > "$repo/kb/a.md"; echo "干净" > "$repo/kb/b.md"
  git -C "$repo" add -A && git -C "$repo" commit -q -m base
  echo "BAD（别人没收尾的）" >> "$repo/kb/a.md"
  echo "我的" >> "$repo/kb/b.md" && git -C "$repo" add kb/b.md
  run_isolated "$repo" 20 >/dev/null; rc_clean=$?
  # 无对象可判（77）的阶段：与判得了的阶段一起点名时应当通过并在成功行里列名；只点名它时应当报 77、不报通过
  printf '#!/usr/bin/env bash\n# gate-stage: selftest-none\necho "  ! 无对象"; exit 77\n' > "$repo/.claude/gate.d/40-none.sh"
  git -C "$repo" add .claude/gate.d/40-none.sh
  local out_mixed rc_mixed rc_none
  out_mixed="$(run_isolated "$repo" 20 40 2>&1)"; rc_mixed=$?
  run_isolated "$repo" 40 >/dev/null 2>&1; rc_none=$?
  if [[ "${CHECK_STAGED_USE_WORKTREE:-0}" == 1 ]]; then
    rm -rf "${repo:?}"
    if [[ $rc_clean -eq 0 ]]; then echo "selftest: 拿工作区原样去跑，没暂存的 BAD 却没算进来 —— 检查坏了"; return 1; fi
    echo "selftest: 拿工作区原样去跑确认判红（没暂存的 BAD 被算进来）"; return 0
  fi
  echo "BAD（我的）" >> "$repo/kb/b.md" && git -C "$repo" add kb/b.md
  run_isolated "$repo" 20 >/dev/null; rc_bad=$?
  # 跑到一半被打断：一个睡 20 秒的阶段，worktree 建起来之后给整组发 INT（Ctrl-C 的形态）；
  # 打断之后仓里只许剩主工作区那一个登记。开 job control（set -m）是为了让后台那一组收得到 INT。
  printf '#!/usr/bin/env bash\n# gate-stage: selftest-slow\nsleep 20\n' > "$repo/.claude/gate.d/30-slow.sh"
  git -C "$repo" add .claude/gate.d/30-slow.sh
  local registered_after_interrupt leftover
  registered_after_interrupt="$(
    set -m
    ( run_isolated "$repo" 30 >/dev/null 2>&1 ) &
    job="$!"
    for _ in $(seq 1 100); do
      [[ "$(git -C "$repo" worktree list --porcelain | grep -c '^worktree ')" -ge 2 ]] && break
      sleep 0.1
    done
    kill -INT -- -"$job" 2>/dev/null; wait "$job" 2>/dev/null
    git -C "$repo" worktree list --porcelain | grep -c '^worktree '
  )"
  while IFS= read -r leftover; do [[ -n "$leftover" ]] && rm -rf "${leftover:?}"; done \
    < <(git -C "$repo" worktree list --porcelain | sed -n 's/^worktree //p' | tail -n +2)
  rm -rf "${repo:?}"
  if [[ "${CHECK_STAGED_NO_TRAP:-0}" == 1 ]]; then
    if [[ "$registered_after_interrupt" == 1 ]]; then echo "selftest: 关掉 trap 之后打断仍然没留下 worktree —— 检查坏了"; return 1; fi
    echo "selftest: 关掉 trap 确认判红（打断之后临时 worktree 留在仓里）"; return 0
  fi
  if [[ "$registered_after_interrupt" != 1 ]]; then echo "selftest: 跑到一半被打断，临时 worktree 留在仓里（登记 $registered_after_interrupt 个）"; return 1; fi
  if [[ $rc_clean -ne 0 ]]; then echo "selftest: 只有工作区里没暂存的 BAD，却判红了 —— 别人的改动漏进来了"; return 1; fi
  if [[ $rc_bad -ne 1 ]]; then echo "selftest: 暂存了 BAD 却没判红"; return 1; fi
  if [[ $rc_mixed -ne 0 || "$out_mixed" != *"无对象没跑 1 个：40-none.sh"* ]]; then
    echo "selftest: 无对象可判（退出码 77）的阶段被当成红、或没在成功行里列名（退出码 $rc_mixed）"; return 1
  fi
  if [[ $rc_none -ne 77 ]]; then echo "selftest: 点名的阶段全是无对象可判时应当退出码 77，实际 $rc_none"; return 1; fi
  echo "selftest: 通过（没暂存的 BAD 不算、暂存了的 BAD 判红、无对象可判的阶段不算红也不算通过、跑到一半被打断也清掉临时 worktree）"
  return 0
}

if [[ "${1:-}" == --selftest ]]; then
  selftest; exit $?
fi
repo_root="$(git rev-parse --show-toplevel)" || { echo "  ✗ 不在 git 仓里"; echo "    → cd 到仓里再跑"; exit 2; }
if [[ $# -gt 0 ]]; then run_isolated "$repo_root" "$@"; else run_isolated "$repo_root" "${DEFAULT_STAGES[@]}"; fi
```

**出处 `research/scripts/cache-keepalive.sh:1-54`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 子 agent 等长活时的缓存计时器：用 Bash 的 run_in_background 起它，默认 230 秒后退出，
# harness 的完成通知把子 agent 叫醒，那一次模型调用就把提示缓存续上了。
#
# 为什么：子 agent 的提示缓存是 5 分钟档（2026-09-18 现查会话记录：子 agent 的缓存写入全在
# `ephemeral_5m`，主会话全在 `ephemeral_1h`）。超过 5 分钟不调用模型，缓存过期、整份上下文要重写——
# 2026-09-17 实测一段执行员重写 6 次，每次 36–59 万 token。计时器由子 agent 自己起，主 agent 不参与。
# 2026-09-18 A/B 实测：起计时器的那一组 18 分钟里少花 25,807 折基础输入 token（占 5.5%）；
# 一次叫醒约 4 万、一次整份重写约 14 万，所以单段等 15 分钟以上的不起更省。
# 经过与数见 records/2026-09-16-subagent拆分提案.md 第二十一节。
#
#   cache-keepalive.sh [秒数]     # 默认 230 秒，要小于 5 分钟减去一次模型调用的时间
#   cache-keepalive.sh --selftest # 跑 1 秒的一轮，核退出码与那句下一步；CACHE_KEEPALIVE_DISABLE_HINT=1 时自检必须判红
set -uo pipefail

HINT="缓存计时器到点。看一眼你等的后台任务跑完没有：没跑完就再用 run_in_background 起一次本脚本、结束本轮接着等；跑完了就接着干，不用再起。自己已经交回或被停的，不用管。"

if [[ "${1:-}" == "--selftest" ]]; then
  output="$(bash "$0" 1 2>&1)"; rc=$?
  failures=0
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ 自检：跑一轮应当退出码 0，实际 $rc"   # gate-lint:detail
    failures=1
  fi
  if [[ "$output" != *"看一眼你等的后台任务"* ]]; then
    echo "  ✗ 自检：到点那句话里没有下一步（实际输出：$output）"   # gate-lint:detail
    failures=1
  fi
  bad_seconds_rc=0
  bash "$0" abc >/dev/null 2>&1 || bad_seconds_rc=$?
  if [[ $bad_seconds_rc -eq 0 ]]; then
    echo "  ✗ 自检：秒数不是正整数时应当判红，实际退出码 0"   # gate-lint:detail
    failures=1
  fi
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看本脚本的 HINT 与秒数校验；CACHE_KEEPALIVE_DISABLE_HINT 设着的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ 缓存计时器自检通过：跑一轮退出码 0、到点那句带下一步、秒数不是正整数判红（查了 3 项）"
  exit 0
fi

SECONDS_TO_WAIT="${1:-230}"
if [[ ! "$SECONDS_TO_WAIT" =~ ^[1-9][0-9]*$ ]]; then
  echo "✗ 秒数要是正整数，收到「$SECONDS_TO_WAIT」" >&2
  echo "→ 怎么办：不带参数（默认 230 秒），或给一个小于 300 的正整数。" >&2
  exit 2
fi

sleep "$SECONDS_TO_WAIT"
if [[ "${CACHE_KEEPALIVE_DISABLE_HINT:-}" == "1" ]]; then
  exit 0
fi
echo "$HINT"
```
