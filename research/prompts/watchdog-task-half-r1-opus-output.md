# watchdog-task-half-r1 攻方腿（Opus）报告：K2、K3

日期：2026-11-05。立场：假设「删掉看门狗盯任务那一半不漏事」是错的，找打穿它的情形。被判对象：`research/prompts/watchdog-task-half-r1-snapshot/batch-49.patch`、`batch-49-readme.patch`；实现按此刻的 `task-server/`、`scripts/`。开工时五份被判文件的 sha256 与 `judged-sha256.txt` 逐份相同（`sha256sum` 现核）。

## 复跑命令与模型文件

模型目录 `research/prompts/watchdog-task-half-r1-opus-model/`，三份文件（sha256 是写完时 `sha256sum` 现取）：

| 文件 | sha256 | 做什么 |
|---|---|---|
| `old-watchdog-model.py` | `400724413de7a52be9895925c6bf7b1acd55b81d2ac540cf8fc144275db2ee2e` | 改前那一份 agent-watch.py（被判那一份倒打批 49 补丁得到）对 11 种任务服务状态报什么；改后那一份还剩几处读服务 |
| `overrun-probe.sh` | `ecea7aad5fcdf0ad2ed0c762900ea89b2028f1343a7fa4364ab58948966888aa` | 活探针：日志一直涨、CPU 一直占着的任务，跑到记录用时的 300 倍，看服务停不停、wait 醒不醒 |
| `tools/rerun.sh` | `e0c1a4aa2b552530bc6be9973220e771931c2ac4dd56d248dbbbaf7534f914d8` | 照 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`：核被判 agent-watch.py 的 sha256、倒打补丁、核改前那一份的 sha256（`f6ed737f…f8b3`，我在草稿目录里 `git apply -R --include=scripts/agents/agent-watch.py` 现做过一次）、跑模型 |

复跑：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-task-half-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/watchdog-task-half-r1-opus/rerun-1`（这一轮没有冻结副本；`--frozen` 给仓根时由 rerun.sh 核 agent-watch.py 仍是开工那一版，不是就退 3）。模型与探针我一次都没跑（子 agent 不交任务），下文凡引它们的输出都写「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 依据的性质 |
|---|---|---|
| K2-1 排队过久 | **打中**（漏报）：等的东西不会自己来、又不判 never-fits 的排队，没人报 | 代码 + 服务自己的自证格 `down-host-waits-not-never`（量的要主 agent 跑） |
| K2-1 跑得远超预计 | **打中**（漏报）：日志在涨、CPU 在占的死循环，服务永远不停、wait 永远不醒 | 代码；活探针待主 agent 跑 |
| K2-1 反复重试 | 不算漏报，是迟报：重试次数有上限，到顶有结局、wait 会醒 | 代码 |
| K2-1 异常结局 | **打中**（有条件）：主 agent 不在 `task wait` 上时，有了结局没人取的服务错误没人报 | 代码 |
| K2-1 服务告警 | 并进 K2-2 | — |
| K2-1 任务服务不通 | **打中**（漏报一半）：连不上的经 wait 退 78 叫醒；连得上而 `/health` 不健康（排头放得下却不起、库写不进）没人报 | 代码 + grep 零命中 |
| K2-2 服务告警有没有人读 | **打中**：生产代码里读 `alerts` 的零处；`card-fault` 换卡重排成功时末行是 `succeeded`，定义里「坏卡弹窗」那条接不到 | grep 零命中 + 服务自证格 `card-fault-replaced-on-another-card` |
| K2-3 定义里还在引已删的东西 | 定义各节零处；`.claude/hooks/selftest-registry.tsv` 一行说明过时；范围外另有 `.claude/gate.d/break-switch-replay-files.tsv` 一行注释过时 | grep 原样输出 |
| K3 删掉的开关与格是不是还管留下的一半 | **打中一处**：`--ack 类:<告警名>` 经真起的看门狗子进程那条路，删完之后零格 | 补丁行 + grep；变异未跑（推的） |
| K3 `ack`、`ackscope` 改了点名的格 | 新格核的仍是原来那件事，少了一种区分（两个都不属于子 agent 的主体之间） | 补丁行 |

只在我的模型上量过、被攻过零轮的改法在末尾「改法」一节，每格都标「推的」。

## K2 第一问：六种告警逐种看

前提（现查）：任务服务只经 `task wait` 叫醒提交它的会话。`task wait` 退出只有三种：结局有了（打末行退任务的码）、疑似卡住、连不上满 60 秒。`task-server/task:292`：「if not reply.get("finished"):」，下一行 `task-server/task:293`：「if reply.get("suspect"):」，其余一律接着等；`task-server/task:277`：「if lib.broken("wait-gives-up-on-disconnect") or gave_up」，`task-server/task:55`：「STARTUP_RETRY_SECONDS = 60.0」。服务端 `task-server/task_server.py:4981`：「return {"finished": False, "state": task.get("state")}」——没结局、没疑似卡住时 wait 只拿到这一句。主 agent 不在 `task wait` 上时，服务没有任何别的推送：客户端六个子命令里只有 wait 会阻塞到事件（`task-server/task` 文件头）。

| 告警 | 服务此刻自己报不报、报给谁 | 主 agent 不在 wait 上 | 判定 |
|---|---|---|---|
| 排队过久 | 放不下的分两种：判 never 的记 `never-fits` 结局、经 wait 叫醒（`scripts/hosts/host-dispatch.py:356`：「never = booked > memory["slice_total"] or needed > memory["mem_total"]」）；判「等」的一直排着，不记结局、不报。`/health` 只报排头放得下却没起的那一种（`task-server/task_server.py:3074`：「if judged is None or not judged[1]:」后面直接交空串） | 不醒 | 漏报，情形见下 |
| 跑得远超预计 | 服务只在日志满 600 秒不涨时停（`task-server/task_server.py:2085`：「if seen[0] != size:」日志一涨就返回、不往下判；`task-server/task_server.py:2093`：「overran = bool(estimate) and elapsed > OVERRUN_STOP_FACTOR * estimate」只在日志没涨的分支里）；超过记录用时两倍只写进 `task status`（`task-server/task:118`：「notes.append("已超过记录用时的两倍")」） | 不醒；在 wait 上也不醒 | 漏报，情形见下 |
| 反复重试 | 重排的几种都有上限：`task-server/task_server_lib.py:2555`：「if used > RETRY_LIMITS[outcome]:」，上限 `task-server/task_server_lib.py:1163`：「MEMORY_CAP_RETRIES = 16」（实际先撞到两台最大、记 `memory-cap-at-maximum`）、`task-server/task_server_lib.py:1164`：「SQUEEZE_RETRIES = 3」、`task-server/task_server_lib.py:1167`：「PEER_RUN_RETRIES = 2」；到顶记结局、经 wait 叫醒 | 同「异常结局」 | 迟报不是漏报：改前在第 3 次尝试就报，改后要等到用完（被挤掉的最多 4 次整跑才有结局，推的） |
| 异常结局 | 经 wait 末行交给提交它的会话（`task-server/task:299` 起），不在 wait 上的只在 `task status` 里列「完成了没人取」（`task-server/task:365`：「完成了没人取：」） | 不醒 | 有条件的漏报，情形见下 |
| 服务告警 | 只写库、`alerts.jsonl` 与服务自己的日志，没人读 | 不醒 | 见 K2 第二问 |
| 任务服务不通 | 连不上：wait 连续 60 秒连不上退 78；连得上而不健康（`task-server/task_server.py:4924`：「return {"ok": not problems」，problems 是库写不进与排头放得下却不起）：wait 照样拿到「没结局」、接着等；只有 `control check`、`control watch` 判它（`task-server/task_server_lib.py:2340`：「if status != 200 or not health.get("ok"):」） | 不醒 | 不健康那一半漏报，情形见下 |

「谁去跑 `control check` / `control watch`」现查：`grep -rnE 'task-server/control (watch|check)|control (watch|check)' .claude/main-agent.md .claude/agent-common.md CLAUDE.md .claude/rules .claude/agents .claude/skills .claude/kb/tooling.md | grep -c ''` 输出 `0`。`task-server/control:28`：「一直好就一直查（主 agent 用 run_in_background 起它盯着，退出即处置）」——服务那一侧写了要主 agent 起它，定义里没有一处叫主 agent 起它。

### 排队过久：等的东西不会自己来、又不判 never-fits

「第二台长时间用不了」：一条记过峰值 30G 的命令，本机 slice 20G 放不下，只有第二台放得下；第二台用不了（重启没回来、ssh 或挂载坏了）。服务把它排着、不判 never-fits——这是服务自己的自证格写死的行为：`task-server/task_server_selftest.py:4301`：「def cell_down_host_waits_not_never(world):」，判法 `task-server/task_server_selftest.py:4312`：「if waiting["task"].get("outcome") or waiting["attempts"]:」（排着时既没有结局也没有尝试才过）。第二台一直不回来，这条一直排着，`task wait` 一直拿到「没结局」。改前：等的原因里没有内存、显存、卡这类字样的判「不会自己好」，排满 10 分钟报「排队过久」；有「内存：」字样而全服务在跑 0 个的判「没人占着」也报。改后：没人报。`control watch` 会在第二台用不了满 180 秒时退 1（`task-server/control:49`：「PEER_UNAVAILABLE_WATCH_SECONDS = 180.0」），但没人起它。

「空闲被压住」（slice 外或 slice 里回收不掉的占用把空闲压住）：`needed > free` 而 `booked ≤ slice_total`、`needed ≤ mem_total`，`judge_host` 交 fits 假、never 假，一直等；全服务在跑 0 个时没有谁会腾出来。改前判「没人占着」报；改后没人报。

复现（量的要主 agent 跑）：`down-host-waits-not-never` 这一格就是「第二台长时间用不了」的服务一侧；模型 `queue-memory-nobody-running`、`queue-peer-down-other-sessions-running` 两格给出改前那一份对同样状态报「排队过久」（推的，待主 agent 跑）。反面对照 `queue-memory-other-sessions-running`：别的会话有任务在跑、等的又是内存时，改前同样不报——这是改前就有的射程（它看不到别的会话那几条正不正常），不是这一批带来的。

### 跑得远超预计：日志在涨、CPU 在占的死循环

情形：一条命令上一次 5 秒跑成（服务记成预计用时），这一次进了死循环，每几秒打一行进度、CPU 占满一个核（重试循环、打进度的自旋等待都是这个样子）。服务的三条停法与通知都不中：卡住要日志满 600 秒不涨（日志在涨）；超过记录用时 5 倍那一条同样要日志满 600 秒不涨；疑似卡住要 CPU 满 600 秒不到一个核的 1%（CPU 在占）。它一直跑，`task wait` 一直不醒。改前：已跑到预计的 3 倍、且多出 300 秒（305 秒时）报「跑得远超预计」。改后：没人报；只有主 agent 自己去跑 `task status <号>` 才看得到「已超过记录用时的两倍」。

复现（活探针，量的要主 agent 跑）：`overrun-probe.sh` 交两次、命令逐词相同（命令见「要主 agent 跑的命令」）。第二次那一条忙等 1500 秒、每 5 秒打一行。改后应当看到：1200 秒前后（宽限 600 秒 + 窗口 600 秒）没有 `TASK_SERVER_SUSPECT`、没有被停，到 1500 秒前后才打末行 `type=succeeded`；中间 `task status` 写「已超过记录用时的两倍」。模型 `overrun-log-growing` 一格给出改前那一份对已跑 1500 秒、预计 5 秒、日志 3 秒前还涨的状态报「跑得远超预计」（推的，待主 agent 跑）。推翻条件：探针在 1500 秒之前被停或 wait 提前醒，这一格就不成立。

### 异常结局：主 agent 不在 wait 上时

情形：主 agent 交了任务、起了 `task wait`，之后那一只 wait 不在了而任务还在跑——wait 报疑似卡住退 231 之后没再起（`task-server/task:293` 那一支）、连不上满 60 秒退 78 之后主 agent 照 `.claude/agent-common.md:78`：「`task wait` 退 78（连不上服务）」那一条去 `task status` 而没再起 wait、会话重启后后台的 wait 没了（这一种是推的：我没核 Claude Code 重启时会不会停后台命令）。之后任务记了 `stalled`、`card-fault`、`launcher-lost` 这类结局，服务把它放进「完成了没人取」，没有人去取。改前：会话回话里有了结局没人取、结局不是 succeeded、command-failed、cancelled 的报「异常结局」。改后：没人报，直到主 agent 自己想起来跑 `task status`。

模型 `outcome-unclaimed-stalled` 一格：改前那一份对一条结局 `stalled`、没人取的任务报「异常结局」（推的，待主 agent 跑）。限度：改前也只在看门狗正在跑时报（会话里第一次派发之后才起），没有子 agent 在跑、主 agent 也没起 `--processes` 时同样没人报；所以这一格的「漏」只覆盖「有看门狗在跑」的那一段。

### 任务服务不通：连得上而不健康

情形：服务的 HTTP 线程还在、排队线程挂了或卡住，排头放得下却一直不起；或库写不进。`/health` 回 ok 假，`control check` 退 78。`task wait` 照样每 20 秒问一次、拿到「没结局」接着等，任务一直排着。改前：每次整查起 `control check`，不退 0 报「任务服务不通」。改后：没人报；连不上那一半照旧经 wait 退 78 叫醒（在 wait 上时）。

模型 `service-reachable-but-unhealthy` 一格：假的 `control check` 退 78、标准错误写「回话了但不健康（…排头放得下却 700 秒没起）」，改前那一份报「任务服务不通」（推的，待主 agent 跑）。

## K2 第二问：服务写的告警删完之后有没有人读

服务写告警只有一处：`task-server/task_server.py:1740`：「def alert(self, kind, task_id, line, next_step):」，写三处：库（`task-server/task_server.py:1742`：「self.store.insert("alerts", dict(record))」）、`alerts.jsonl`（`task-server/task_server.py:1744`：「append_private_line(self.configuration.alerts_log」）、服务自己的标准输出即 journal（`task-server/task_server.py:1745`：「say(f"  ✗ 告警（{kind}）任务 {task_id}：{line}")」）。

没人读的证据（这一次现跑，原样）：

```
$ grep -rnE 'alerts\.jsonl|alerts_log' scripts .claude --include=*.py --include=*.sh --include=*.md -l | grep -c ''
0
$ grep -rnE 'alerts\.jsonl|alerts_log' scripts .claude task-server prune-server -l
task-server/task_server.py
task-server/task_server_selftest.py
task-server/task_server_lib.py
```

三份里：`task_server_lib.py` 只定义路径（`alerts_log` 属性），`task_server.py` 只写（1744 那一行），`task_server_selftest.py` 的 `alerts()` 读的是自证临时世界里的那一份（`task-server/task_server_selftest.py:760`：「def alerts(self):」）。库里的 `alerts` 表：`grep -n '"alerts"' task-server/task_server.py task-server/task_server_lib.py task-server/control task-server/task` 只命中 `LOG_TABLES` 的定义与 1742 的写入。服务的接口（`grep -n '@application\.\(get\|post\)' task-server/task_server.py`）是 `/health`、`/tasks`、`/tasks/{task_id}`、`/tasks/{task_id}/wait`、cancel、clean、`/pins`、`/pins/show`、`/wake`、`/sessions/self`，没有读告警的。结论：删完之后生产代码里读告警的是零处，只剩人去翻 journal。推翻条件：找到一个起 `alerts.jsonl` 或库 `alerts` 表的读者。

逐种看告警会不会经别的路到主 agent（行号是 `task-server/task_server.py` 的写告警处）：

| kind | 写在 | 别的路 | 删完之后 |
|---|---|---|---|
| card-fault（有坏卡清单） | 2524 | 坏卡停用 60 分钟、这一条换卡重排；重排跑成则结局 `succeeded` | **漏**，下面单说 |
| card-fault（取不到坏卡清单） | 2528 | 照 card-fault 收尾、不重排 → wait 末行 | 不漏（在 wait 上时） |
| record-tampered | 1751 等 | 任务那一种记结局 record-tampered，wait 打 ✗ 行（`task-server/task:302`：「if not reply.get("valid", True):」） | 不漏（在 wait 上时） |
| suspected-stall | 2164 | wait 退 231 | 不漏（在 wait 上时） |
| prune-server-unavailable | 1957 | 结局退 75 | 不漏（在 wait 上时） |
| peer-unavailable（`note_peer`） | 1920 | 调它时不带任务号（`task-server/task_server.py:1868`：「self.note_peer(available, "" if available」） | 改前也不报（认不出会话），不是这一批带来的 |
| peer-unavailable（那一趟没跑成） | 1930 | 重排至多 2 次，到顶有结局 | 迟报 |
| cleanup-pending | 2305 | 尝试保持在跑、资源照占（`task-server/task_server.py:2301`：「attempt.update(cleanup_pending=note」），没有结局 | **漏**（推的） |
| shard-advance-failed | 2808 | 分片父任务推不动就一直不收尾 | **漏**（推的） |
| settle-failed、start-uncertain、item-rates-unrecorded、build-directory-left | 2340、4328、2361、2610 | 结局照记或没有结局相关的通知；说明只在告警里 | 告警内容丢（推的） |

### card-fault 换卡重排跑成：定义里的「坏卡弹窗」接不到

改后定义只在末行是 card-fault 时弹窗：`.claude/main-agent.md:67`：「坏卡不由看门狗报：交的任务结局是 `card-fault`（`task wait` 末行 `type=card-fault`、退 237）时，当场弹窗告诉用户」。服务的做法：清场判出坏卡、清单取得到时，坏卡停用、这一条换别的卡重排（`task-server/task_server.py:2522`：「self.quarantined_cards[identity] = time.time()」，重排上限 `task-server/task_server_lib.py:1166`：「CARD_FAULT_RETRIES = 4」）。换卡跑成，末行就是 `succeeded`——服务自证格把这一点写死了：`task-server/task_server_selftest.py:4328`：「if first["task"].get("outcome") != "succeeded" or cards != [LOCAL_CARD_ZERO, LOCAL_CARD_ONE]:」（第一次派到坏卡、换卡跑完，结局要是 succeeded 才过）。这时「主 agent 当场弹窗请用户重置那张卡」只写在告警的出路里（2524 那一条的 next_step），没人读；坏卡 60 分钟后回到候选、还坏再停用，用户一直不知道。改前：这条告警的任务属于本会话、上次整查之后新增，报「服务告警」（改前自证就拿 card-fault 当服务告警的样本，补丁第 984 行）。

wait 自己的输出里其实有一处痕迹：末行之前逐次尝试一行，每行带那一次的结局（`task-server/task:107`：「（{attempt.get('outcome') or '还没结局'}）」），所以派到坏卡的那一次尝试行写着 card-fault。但定义认的是末行，不是尝试行。

复现（量的要主 agent 跑）：服务自证格 `card-fault-replaced-on-another-card`（命令见「要主 agent 跑的命令」），过了就说明「坏卡换卡跑成、结局 succeeded、写了一条 card-fault 告警」三样同时成立；模型 `card-fault-retried-on-another-card` 一格给出改前那一份报「服务告警」（推的，待主 agent 跑）。

## K2 第三问：定义里别处还在引已删的告警、阈值、`--ack` 键形

命令一（射程照派发：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/`、`.claude/hooks/`、`.claude/rules/`；词表是已删的六个告警名、三个阈值键、两种键形、「告诉用户」标记与已删的函数与变量名）：

```
$ P='任务服务不通|排队过久|跑得远超预计|反复重试|异常结局|服务告警|task-queue-minutes|task-overrun-factor|task-retry-count|任务:<号>|任务:[0-9]|任务服务:|告诉用户：|本会话任务|task_server_report|task_health_report|AGENT_WATCH_TASK_SERVER_DIRECTORY|acktaskform'
$ grep -rnE "$P" .claude/main-agent.md .claude/agent-common.md .claude/agents/ .claude/hooks/ .claude/rules/ > k2q3-scope.txt; grep -c '' k2q3-scope.txt
2
```

两行命中（行内过长，截到要点；全文在草稿目录 `k2q3-scope.txt`）：

1. `.claude/hooks/selftest-registry.tsv:32`：「任务服务与本会话任务那几格换成临时目录里假的 task-server 目录（control 与 task 是打固定回话的 python 小脚本，服务告警记录是写好的几行），不连真服务」——agent-watch.py 自证登记行的说明，那几格已删，**过时**。
2. `.claude/hooks/completion-cleanup-guard.sh:2176`：「子 agent 交回时本会话任务 13 的编译目录」——完工清理闸自证里的「本会话的任务 13」，说的是任务服务的编译目录，与看门狗无关，**不算**。

`.claude/main-agent.md` 的命中数是 0：改后第 63、67、73 行已不提这些名字；别的节没有。`.claude/agent-common.md`、`.claude/agents/*.md`、`.claude/rules/` 都是 0。

命令二（全仓，`git grep` 只扫被 git 跟踪的文件，排除 `research/` 与 `records/` 这两处当时的记录；词表去掉「异常结局」「本会话任务」两个泛词，补上已删的开关名与自证函数名），再用 `grep -rn` 把 `.claude scripts task-server crates CLAUDE.md` 里没被跟踪的也扫一遍：

```
$ P='任务服务不通|排队过久|跑得远超预计|反复重试|服务告警|task-queue-minutes|task-overrun-factor|task-retry-count|任务:<号>|任务服务:任务服务不通|→ 告诉用户|task_server_report|task_health_report|AGENT_WATCH_TASK_SERVER_DIRECTORY|acktaskform|taskserverblind|taskhealthblind|taskack|tasktelluser|selftest_task_server|selftest_task_health|write_fake_task_server'
$ git grep -nE "$P" -- . ':!research/' ':!records/' > k2q3-repo.txt; grep -c '' k2q3-repo.txt
2
$ grep -rnE "$P" .claude scripts task-server crates CLAUDE.md --exclude-dir=target --exclude-dir=__pycache__ > k2q3-fs.txt; grep -c '' k2q3-fs.txt
2
```

两条命令命中的是同两行：上面那一行 `.claude/hooks/selftest-registry.tsv:32`，加 `.claude/gate.d/break-switch-replay-files.tsv:63`：「taskhealthblind 起的十八条，taskserverblind、taskservercleanunseen、ack 三条改了点名的格」——弄坏开关复跑登记表里 agent-watch.py 那一行的注释（第二列 600 秒是限时），点名的开关已删、条数「一百零二条」也不是现数：此刻文件头 `^  AGENT_WATCH_BREAK=` 有 79 行（`grep -cE '^  AGENT_WATCH_BREAK=' scripts/agents/agent-watch.py` 现数），补丁删了 24 行（`grep -cE '^-  AGENT_WATCH_BREAK=' batch-49.patch` 现数，新加 0 行），改前是 103 条——注释在改前就差一条。这一行在派发点名的射程（`.claude/gate.d/` 不在里面）之外，列出来供主 agent 定改不改。没有翻译版：`.claude/main-agent*` 只有一份，`scripts/agents/README*` 只有一份。

## K3：自证与开关删得对不对

### 删掉的 24 + 1 条开关

24 条名字（`grep -oE '^-  AGENT_WATCH_BREAK=[a-z]+' batch-49.patch` 现取）：taskserverblind taskserverexitignored taskservercleanunseen taskservercleanexists taskservercleandeleted taskhealthblind taskqueueanywait taskqueueunrecoverable taskqueueidlemissed taskqueueoccupantack taskqueueoccupantignored taskoverrun taskoverrunfactorignored taskoverrunnoestimate taskretry taskretrythreshold taskoutcome taskoutcomeanyfailure taskalerts taskalertsothersession taskalertsrereported taskhealthrepeats taskack tasktelluser；另一条是 watch.sh 的 `acktaskform`。

逐条判：

- 23 条（除 taskack）弄坏的那一处都在已删的函数里（`task_server_report`、`build_directory_lines`、`uncleaned_build_directories`、`task_health_report`、`task_service_alerts`、`occupant_acknowledged`、`task_alert`；补丁第 236–627 行整段删），点名的格都在已删的 `selftest_task_server`、`selftest_task_health` 里（补丁第 127–186 行）。留下的判法没有一处调这几个函数：`grep -cE 'run_task_server_script|TASK_SERVER_ALERTS_NAME|alerts\.jsonl|"--json"|task_server_directory|control", \["check"\]' scripts/agents/agent-watch.py` 现数是 `0`。判定：删得对，没有一条还管着留下的那一半。
- taskack：弄坏的 `task_ack_keys` 被留下的 `split_acknowledged` 与 `alert_keys` 调（补丁第 698、707 行把那一项去掉）。去掉之后这两个函数对留下的告警算的键是 `{主体:告警名} | ack_scope_keys`，与改前对非任务告警算的一样（改前 `task_ack_keys` 对主体不以「任务 」起头的交空集，补丁第 680 行）。判定：删得对。
- acktaskform：只把 `ack_head` 里的「任务」头去掉，点名的格是已删的自检⑫（补丁第 1186–1188 行）。留下的 `ackscopeform` 照旧点名自检⑪（补丁第 1189 行：「自检⑪（「✗ 自检⑪：--ack 类:<告警名> 没原样转过去」）」）。判定：删得对。
- 别处登记这几条开关的：`grep -nE 'agent-watch|watch\.sh' .claude/gate.d/break-switch-*.tsv` 只命中复跑登记表第 63 行（注释过时，见 K2 第三问），表里没有逐条列开关名的行。

### 删掉的格里有一格其实还管着留下的一半：`--ack 类:<告警名>` 经真起的看门狗那条路

改前唯一一格真起看门狗子进程、带 `--ack 类:<告警名>`、核它到点退 4 并写「已确认接着盯」的，在已删的 `selftest_task_server` 里：补丁第 840 行：「run = watch_with(down_directory, ["--agents", "serverhealthy", "--ack", f"类:{TASK_SERVER_ALERT}", "--max-minutes", "0.03"])」。`ack` 与 `ackscope` 两条留下的开关原来都点名它（补丁第 109、121 行）。

改后留下的、真起看门狗又带 `--ack` 的调用（`grep -nE '"--ack"' scripts/agents/agent-watch.py` 现跑）一共 7 处，键是 `本机:根分区余量过低`（2636）、`进程:<pid>`（2813）、`<agent 号>:<告警名>`（3230、3249、3630、3634、3639），没有一处是 `类:`。`类:` 只剩两处在核：`selftest_ack_scopes` 在本进程里直接调 `split_acknowledged` 与 `pruned_acks`（不经命令行、不经 `acknowledgements()`），watch.sh 自检⑪只核 `--dry-run` 打出来的命令行。

会漏掉什么（推的，没跑）：把 `scripts/agents/agent-watch.py:2295`：「return set(arguments.ack or []) | (read_acks(state_dir) if discover else set())」改成先滤掉 `类:` 起头的键（或在 argparse 之后、`split_acknowledged` 之前任一处丢掉它们），改后的整套自证照绿：`selftest_ack_scopes` 不经这一行，⑪ 只看 dry-run。改前补丁第 840 行那一格会红（退 3 而不是 4）。这一格是被判对象里「`类:` 这一键形主 agent 真用得上」唯一的端到端证据；`.claude/main-agent.md:73` 还写着 `类:<告警名>`（例 `类:交回之后后台还在跑`）是三种键形之一。

### `ack`、`ackscope` 点名的格改了原文：新格还核不核原来那件事

`selftest_ack_scopes` 的告警表从四条（任务服务不通、根分区余量过低、两条交回之后后台还在跑）变成三条（补丁第 734–736 行），点名的格「`--ack 类:任务服务不通 应当只确认告警`」换成「`--ack 类:根分区余量过低 应当只确认告警`」（补丁第 112、122、741 行）。

- 核的仍是同一件事：`类:X` 只确认告警名是 X 的那几条，别的类、别的进程照报；记进状态目录的 `类:` 确认在告警还在时留着。`ack` 下 `split_acknowledged` 什么都不确认，`类:根分区余量过低` 那一格拿到的是空集、不等于 `[根分区余量过低]`，照红；`ackscope` 下 `ack_scope_keys` 交空集，两个 `类:` 格与 pruned 那一格照红（我按代码推的，没跑；命令见下）。
- 少了一种区分：改前表里有两条主体都不是子 agent 的告警（「任务服务」与「本机」），`类:任务服务不通` 只确认前一条，就同时核了「类确认按告警名、不按『主体不是子 agent』这一类」。改后只剩「本机」一个非子 agent 主体，一种「把 `类:` 当成『确认全部非子 agent 主体的告警』」的错改，在新表上与正确的实现给出同一个结果（推的）。不是这一批要修的大事，列出来交主 agent 定。
- 自检⑪的样本从 `类:任务服务不通` 换成 `类:交回之后后台还在跑`（补丁第 1222–1225 行），核的仍是「`类:` 头原样转过去、逗号连写拆开」，没有变弱。自检⑬改号成⑫，全仓 `grep -rn '自检⑬\|自检⑫'`（排除 `research/`）只命中 `scripts/agents/watch.sh` 自己，没有别处引旧号。

## 打中之后的四句

这一轮背景材料没有写跑前条款与候选改法，「臂」读成改前、改后两份看门狗。

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的是哪一个分句 | 跑前条款的改法还中不中 |
|---|---|---|---|---|
| 排队过久（第二台用不了、空闲被压住） | 分辨：改前报、改后没人报；别的会话在跑而等的是内存时两边都不报（不分辨，改前就有的射程） | 看得到：`task status <号>` 的「在等：…」与 `control check` 末行「在跑 N 个」都在，只是没人读 | K2 第一条「找出删完之后没人报的那一种，写出能复现的情形」 | 没有跑前改法 |
| 跑得远超预计（日志在涨的死循环） | 分辨 | 看得到：`task status` 的 progress 带已跑、预计、日志多久没涨 | 同上 | 没有 |
| 异常结局（不在 wait 上） | 分辨，但改前也只在看门狗在跑的那一段 | 看得到：会话回话里「完成了没人取」 | 同上，加上题面「主 agent 不在 `task wait` 上时还会不会被叫醒」 | 没有 |
| 任务服务不通（连得上而不健康） | 分辨 | 看得到：`/health` 的 ok 与 problem | 同上 | 没有 |
| card-fault 换卡跑成 | 分辨 | 看得到：告警记录与尝试行的结局 | K2 第二条「服务写的告警有没有人读」 | 没有 |
| `--ack 类:` 端到端零格 | 分辨（改前那一格红，改后不红，推的） | — | K3 第一条「删了它，留下的判法就少了一格能红的」 | 没有 |

## 改法（只在我的模型上量过、被攻过零轮；下表每一格都是「推的」，一格都没在副本上跑）

| 改法 | 排队过久 | 远超预计 | 异常结局（不在 wait 上） | 不健康 | card-fault 换卡跑成 | cleanup-pending、shard-advance-failed | 代价与冲突 |
|---|---|---|---|---|---|---|---|
| 起 control watch：定义「派出去之后」加一句：会话里有在排、在跑的任务时，用 `run_in_background` 起 `python3 task-server/control watch`，它退 1 即处置（服务那一侧早写了要主 agent 起它，`task-server/control:28`） | 第二台那一种修（满 180 秒退 1，推的）；空闲被压住那一种不修 | 不修 | 不修 | 修（推的） | 不修 | 不修 | 只改定义；`control watch` 不分会话，报的是全服务 |
| 坏卡认尝试行：定义里坏卡那一句改认「wait 输出里任何一行尝试的结局是 card-fault」，不只认末行 | 不修 | 不修 | 不修 | 不修 | 修（在 wait 上时，推的） | 不修 | 只改定义 |
| 取没人取的结局：定义加一句：wait 退 231、退 78 之后，与会话重启之后，先跑 `task-server/task status` 取「完成了没人取」 | 不修 | 不修 | 修大半（推的；靠主 agent 记得做） | 不修 | 不修 | 不修 | 只改定义，靠人记 |
| wait 带非结局叫醒：服务端让 wait 在「已跑超过记录用时 N 倍」「本任务新写了告警」时也像疑似卡住那样退一个不是结局的码 | 不修 | 修（推的） | 不修 | 不修 | 修（推的） | 修（推的） | 改 `task-server/`，超出批 49；要另走实现 |
| 看门狗只读告警：看门狗留一小段只读 `alerts.jsonl` | 不修 | 不修 | 不修 | 不修 | 修（推的） | 修（推的） | 与用户选的「只删盯任务那一半」冲突，交用户定，不替用户改 |

没有一个改法单独修全六格；「跑得远超预计」与「空闲被压住的排队」只有服务端（「wait 带非结局叫醒」一类）或留下看门狗那一半修得到。

## 要主 agent 跑的命令

都在仓根 `/home/fy5090/code/singlefs` 起（不给 `--cwd`），都不是重型。

1. 模型复跑（轻）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-task-half-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/watchdog-task-half-r1-opus/rerun-1`。看：末行 `TASK_SERVER_RESULT … type=succeeded exit=0`；`task-server/task log <号>` 里 `name=before_sha256 got=f6ed737f… want=f6ed737f…`、11 行 `name=<格> … verdict=pass`、一行 `name=after-readers functions=无 lines=无 want=无 verdict=pass`、收尾 `name=done cells=12 failed=0`。退 3 是 `--frozen` 那份 agent-watch.py 已不是开工那一版，退 4 是倒推的改前那一份对不上；任何一格 `verdict=fail`，那一格在报告里对应的「改前报」不成立。
2. 活探针快跑那一趟（轻，约 5 秒）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-task-half-r1-opus-model/overrun-probe.sh /home/fy5090/.local/share/singlefs/work/watchdog-task-half-r1-opus/overrun-state`。看：末行 `type=succeeded exit=0`，日志里有 `PROBE phase=quick end`。
3. 活探针第二次（轻，一个核占满约 1500 秒）：与第 2 条逐字相同再交一次，`task wait <号>` 照常放后台。看：① `task-server/task status <号>` 开跑后不久写「记录用时 5 秒」上下（写「没有记录」的这一趟作废：改前的看门狗也判不了，要先查为什么第 2 条的用时没记上）；② 开跑满 1300 秒以后再 `task-server/task status <号>`，在跑那一行带「已超过记录用时的两倍」、没有「疑似卡住」；③ wait 的输出里没有 `TASK_SERVER_SUSPECT` 一行，末行 `type=succeeded` 出现在开跑约 1500 秒之后；④ `task-server/task log <号>` 末行 `PROBE phase=busy end elapsed=1500` 上下。①②③ 都对上，K2「跑得远超预计」一格坐实；wait 在 1500 秒之前醒（退 231 或有结局），这一格推翻。
4. 服务自证两格（轻；`task-server/README.md:107` 写它由主 agent 直接起、不经服务排队，下面是经服务的写法，二选一）：`task-server/task submit -- env TASK_SERVER_SELFTEST_CELLS=down-host-waits-not-never,card-fault-replaced-on-another-card python3 task-server/task_server_lib.py --selftest`。看：两格都是 ✓；`down-host-waits-not-never` 那一行回的串里有第二台用不了时的排队原因原文（拿来对模型 `queue-*` 两格里写的 waiting_for 像不像）。
5. K3 证红（都是 `.claude/hooks/selftest-registry.tsv` 登记为 light 的自证）：
   - `task-server/task submit -- python3 scripts/agents/agent-watch.py --selftest`：退 0，汇总行报了查了几格。
   - `task-server/task submit -- env AGENT_WATCH_BREAK=ackscope python3 scripts/agents/agent-watch.py --selftest`：退 1，✗ 行里有「--ack 类:根分区余量过低 应当只确认告警」「--ack 类:交回之后后台还在跑 应当只确认告警」「记进状态目录的 类: 确认应当在告警还在时留着」三条，没有点名之外的 ✗ 行。
   - `task-server/task submit -- env AGENT_WATCH_BREAK=ack python3 scripts/agents/agent-watch.py --selftest`：退 1，✗ 行与文件头 `ack` 那一项点名的逐条对得上。
   - `task-server/task submit -- env AGENT_WATCH_BREAK=ackscopepersist python3 scripts/agents/agent-watch.py --selftest`：退 1，只红「记进状态目录的 类: 确认应当在告警还在时留着」。
   - `task-server/task submit -- bash scripts/agents/watch.sh --selftest`：退 0，「判了 N 条」。
   - `task-server/task submit -- env WATCH_BREAK=ackscopeform bash scripts/agents/watch.sh --selftest`：退 1，只红自检⑪。
   这几条只核「点名的格还红」；「`类:` 端到端零格」那一格要另做变异才量得到（改 `acknowledgements()` 滤掉 `类:` 之后整套自证照绿），我没建副本，留作推的。

## 没打中的形状

- 反复重试：找不到「重试一直不到顶」的路，`RETRY_LIMITS` 五种都有上限，到顶就有结局；只算迟报。
- 别的会话的任务把本会话的排队卡住：改前也看不到（服务回话里没有别的会话在跑那几条的正不正常），不是这一批带来的。
- `peer-unavailable`（`note_peer` 那一种）：改前也不报（告警记录不带任务号、认不出会话）。
- 定义里引已删东西：`.claude/main-agent.md` 其余各节、`.claude/agent-common.md`、`.claude/agents/*.md`、`.claude/rules/` 都是 0 命中；词表是上面两条命令里的，泛词「异常结局」「本会话任务」只在定义射程里扫了。
- 删掉的 24 + 1 条开关里，除 K3 那一格之外，没找到还管着留下一半的。

## 这条腿自己的限度

- 模型、活探针、证红命令一条都没跑；报告里「改前报什么」全来自读代码与模型的设计，是推的，要主 agent 照上一节跑过才算量的。
- 「会话重启之后后台的 wait 没了」没核 Claude Code 的行为，只当一种可能。
- 「cleanup-pending」「shard-advance-failed」两种只读了代码（没有结局、wait 不醒），没造出能触发它们的真情形。
- 「空闲被压住」只按 `judge_host` 的代码推，没有在真机上压过空闲。
- 只取了一次样：「没打中」的几格照 `.claude/rules/three-way-inference.md` 要再抽一次才算。

## 没做什么

- 不判正推那一格（K1）；不替主 agent 采纳；副本上量出的数这一轮没有（没建副本）。
- 没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/watchdog-task-half-r1-opus` 里没有仓副本与编译目录：倒打补丁用的两个小目录（`before/`、`check/`，各只有一两份拷来的脚本）用完已删；剩下的是 grep 的输出与拼 rerun.sh 用的草稿。
