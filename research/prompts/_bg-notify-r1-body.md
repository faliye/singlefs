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
