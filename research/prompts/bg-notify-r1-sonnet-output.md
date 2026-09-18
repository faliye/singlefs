# bg-notify-r1 正推腿报告（Sonnet）

分到的格：A3（主 agent 入口引文与三种情形的读法，是否与看门狗判法一致）、A4（agent-watch.py 的 TaskStop 判法、check-staged.sh 的 77 单列）。

## 核实方法

1. 开工快照核对：`research/prompts/bg-notify-r1-start-snapshot.sha256` 里 `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/hooks/bash-command-detector.sh`、`research/scripts/agent-watch.py`、`research/scripts/check-staged.sh`、`.claude/rules/implementation-workflow.md` 六份文件与仓里当前内容 `sha256sum` 逐字节一致（现跑）；`records/2026-09-16-subagent拆分提案.md`、`research/scripts/cache-keepalive.sh` 快照里没有单独列，未核对（不在我这两格要判的对象里）。
2. 每一句要写进报告的引文，先在被引文件里 `grep -nF` 一次，命中的行号才写进报告；命中 0 次的改写「转述」。
3. 能跑的都跑了：`agent-watch.py --selftest`、`AGENT_WATCH_BREAK=taskstop`、`check-staged.sh --selftest` 及三个判红开关，命令与原样输出见下文。
4. 仓里的自检没有覆盖到的几种情形（续做之后又停、同一会话停过别的 id、主会话记录缺失、停在工具中途且没有打断标记），另写了四段独立 Python 脚本，只 `importlib` 现成的 `research/scripts/agent-watch.py`（不改它一个字），在临时目录里另造会话记录跑；`check-staged.sh` 的「红 + 77 混合」组合同理，另起一个临时 git 仓直接调用脚本（脚本本身未改）。产物在 `research/prompts/bg-notify-r1-sonnet-model/`：
   - `resumed-then-stopped-again-test.py`
   - `other-id-stopped-test.py`
   - `missing-parent-session-test.py`
   - `mid-tool-taskstop-test.py`
   - `red-plus-77-fixture/`（临时 git 仓，跑完没删，保留证据）

## A3：主 agent 入口与 harness 原文

### 逐字核对两句英文

`.claude/main-agent.md:29`（grep -n 现查命中的行）：

> 交回按子 agent 的 SubagentHandback 消息判。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。

正文第二节「主 agent 收到的通知原文」行（`research/prompts/_bg-notify-r1-background.md:39`，与 `_bg-notify-r1-body.md:30` 内容相同）：

> 子 agent 结束本轮而后台任务还在时：status `completed`，note「This agent stopped with background work of its own still running. It may resume on its own when that work completes or reports, and the same task-id notifies again if it does; the result below may be interim.」，result「This agent has not reported yet: it is waiting on its own background work and will deliver its report through SubagentHandback when that finishes.」

`grep -nF` 逐字核对：

```
$ grep -nF 'This agent has not reported yet: it is waiting on its own background work' .claude/main-agent.md
29:交回按子 agent 的 SubagentHandback 消息判。……
$ grep -nF 'This agent has not reported yet: it is waiting on its own background work' research/prompts/_bg-notify-r1-body.md
15:……
30:| 主 agent 收到的通知原文 | ……
$ grep -nF 'the result below may be interim' .claude/main-agent.md
29:……
$ grep -nF 'the result below may be interim' research/prompts/_bg-notify-r1-body.md
15:……
30:……
```

两句都在两份文件里精确命中，且各是通知原文里连续的逐字子串（`result` 字段那句的前段、`note` 字段那句的尾段），截到句子的自然边界，没有改写、没有漏词、没有多字。

**判定：一致。**
推翻条件：若这两句在 `.claude/main-agent.md` 或背景材料正文里少一个词、多一个词，或断在了原句中间某个改变了语义的位置，就算打中——现查两处逐字相同，没有出现。

### 三种情形逐一核：入口怎么做，与看门狗判法是否一致

**情形一：子 agent 结束本轮等后台**

入口（`.claude/main-agent.md:29`，上面已引）：不当交回、不当出错，照旧等它的交回消息与看门狗。

看门狗 `research/scripts/agent-watch.py`：这类记录（最后一条是 `stop_reason="end_turn"` 的文本，没有 `SubagentHandback` 成功返回，也没有 `[Request interrupted`）落进 `_read()` 的 `elif last_kind == "end_turn": self.state = "本轮结束、没交回"`（214 行），`is_done()`（229-230 行）对这个状态返回 `False`；`agent_alerts()`（312 行）在 `pending_tool is None` 时才会因为 `last_timestamp` 闲置超过 `idle-minutes`（默认 10 分钟）给一条「无动静」告警（329 行），措辞是「主 agent 判断：模型调用是不是卡住或撞了限额……」——是提醒主 agent 自己判断，不是自动判定出错或已完成。

**判定：一致。** 两边都不把这种通知当交回、也不当错误；看门狗会在闲置太久时提醒主 agent 自己去看，这和入口「不当出错，照旧等」不冲突——它仍然是「等」，只是多一条「该看一眼了」的提示。
推翻条件：若 `agent_alerts()` 在这个状态下把「无动静」当成需要自动处理的硬失败（而不是留给主 agent 判断），或者 `is_done()` 在这个状态下返回 `True`（被当成已完成），就是冲突——现查代码都不是。

**情形二：子 agent 交回**

入口：交回按子 agent 的 SubagentHandback 消息判（`.claude/main-agent.md:29` 首句）。

看门狗：`SubagentHandback` 工具调用之后若对应 `tool_result` 内容里出现 `"success":true`（且之后没有再调别的工具），`_read()` 置 `is_handed_back = True`（192 行），随后 `self.state = "已交回"`（208 行），`is_done()` 对它返回 `True`。

**判定：一致。**

**情形三：子 agent 被 TaskStop 停掉**

先现查入口文本里有没有出现过「TaskStop」这个词：

```
$ grep -n "TaskStop" .claude/main-agent.md .claude/agent-common.md
（0 命中）
```

`.claude/main-agent.md`「交回怎么读」（25-29 行）与 `.claude/agent-common.md`「不做」一节（32-43 行）全文都没有一处写「被 TaskStop 停掉时怎么读」。主 agent 知道自己停了某个子 agent，靠的是 `TaskStop` 工具调用自身的返回结果（正文第二节：「主会话记录 14:01:49.765 有 TaskStop 的结果……`{"message": "Successfully stopped task: …"}`」，`_bg-notify-r1-background.md:40`）——这条路径不经过「交回怎么读」段要判读的 harness completed 通知，所以入口没写不一定是漏了，但严格按 A3 判据字面（「按那段做，主 agent……会怎么做」），入口那段本身对这第三种情形确实一个字都没说。

看门狗（A4 改动后）：`task_stop_time()`（57-75 行）读主会话记录，`__init__`（120-122 行）据此把 `state` 改判「被停」，`is_done()` 返回 `True`（除非又被续做，见 A4 一节）。

行为结果上不冲突：主 agent 自己已经认定「停了」（TaskStop 工具调用的返回值就是证据），看门狗现在也认定「被停」——不会出现主 agent 已经知道停了、看门狗还在报警要求处理的情形。这正是改之前真实撞上的那次（记录第三十三节，正文 `_bg-notify-r1-background.md:32`：「改前的看门狗 14:12:14 报『状态=本轮结束、没交回』『⚠️ 无动静……』，退出码 3……；改后同一个 id 的 report 报『状态=被停』、没有告警」）。我用 `AGENT_WATCH_BREAK=taskstop` 复现了这次冲突（见 A4 一节的命令与原样输出）：关掉这条改动之后，`stoppedidle` 那一格立刻从「无告警、`is_done()=True`」变成「告警『无动静』、`is_done()=False`」——证明这个冲突在改之前是真实存在的，改之后没有了。

**判定：入口文本本身「规则没说」；实际行为结果上「一致」（不冲突）。** 这份一致建立在一个入口没写明的前提上：只有当前这个主 agent 自己会调用 `TaskStop` 停自己的子 agent。
推翻条件：出现一种情形——`TaskStop` 由不是当前主 agent 会话的另一方发起（另一个并发主 agent 会话、或 harness 自动超时停用），主 agent 读不到那次调用的返回结果，只能靠看门狗报告——这时入口一个字都没写该怎么读就是真正的缺口。材料里没有出现过这种情形的证据（复核不了：我够不着主会话记录本身，只能核正文抄录下来的那几条）。

## A4：看门狗与 check-staged 的改法

### agent-watch.py 的 TaskStop 判法：判据代码

`research/scripts/agent-watch.py`（行号现查）：

```
45:STOP_AFTER_LAST_RECORD_SECONDS = 30  # 停在工具中途时，被停之后还可能落几条收尾记录
52:def parent_session_transcript(transcript_path):
57:def task_stop_time(agent_id, session_transcript_path):
59:    if not os.path.isfile(session_transcript_path):
60:        return None
70:        if not isinstance(result, dict) or result.get("task_id") != agent_id or ...
74:            stopped_at = timestamp if stopped_at is None else max(stopped_at, timestamp)
120:        stopped_at = task_stop_time(agent_id, parent_session_transcript(transcript_path))
121:        if (stopped_at is not None and BROKEN_DETECTION != "taskstop" and self.state != "已交回"
122:                and (self.last_timestamp is None or (self.last_timestamp - stopped_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
123:            self.state = "被停"
```

45 行注释「停在工具中途时，被停之后还可能落几条收尾记录」说明这条 30 秒宽限本身就是为「工具中途」这一路设计的，这一点与文件头 19 行文档字面「『被停』认两处……（停在工具调用中途）；……TaskStop……（停在结束本轮、等后台任务的时候）」暗示的「两处各管一段互不相干的情形」不完全一致——我在下表用构造的样本核了这一点。

### 五种情形逐一核（判据 A4 点名的五种）

| 情形 | 现查/实测 | 判定 |
|---|---|---|
| 停在工具中途 | 另写 `mid-tool-taskstop-test.py`：只有一条 `Bash` 的 `tool_use`、没有对应 `tool_result`（工具还没回来），也没有 `[Request interrupted` 标记，主会话记录里有一条对它的 TaskStop 成功结果。实测：`state = 被停, is_done() = True, pending_tool = (…, 'Bash', 'cargo test --release')`。 | 对，而且比文件头 19 行「认两处」字面暗示的范围更宽：没有 `[Request interrupted` 标记时，TaskStop 那条判法照样接得住工具中途的停 |
| 停了又续做 | 仓里自带样本 `stoppedcontinued`：`write_session_stops` 记一次 TaskStop（20 分钟前），随后又给这个 id 追加了续做的新记录（2 分钟前起）。`selftest()` 里 `wanted_done["stoppedcontinued"] = False`，实测通过（见下方 baseline 输出） | 对 |
| 续做之后又停（第二次 TaskStop） | 仓里没有样本覆盖这一格。另写 `resumed-then-stopped-again-test.py`：主会话记录里对同一个 id 写两次 TaskStop 成功结果（18 分钟前、8.83 分钟前），中间插一段续做（10 分钟前跑了个 `cargo test`，9 分钟前又结束本轮等后台）。实测：`task_stop_time` 返回的是离现在 529.9 秒（≈8.83 分钟）那次，不是 1080 秒（18 分钟）那次；`state = 被停, is_done() = True`。这条测试真正验的是 `task_stop_time`（74 行）用 `max()` 取最晚一次——如果代码误取了第一次，529.9 秒会变成约 1080 秒，超过 30 秒宽限就不会判被停，测试会 FAIL；实际 PASS | 对，但**没有自检样本覆盖，是我现补的验证**，仓里 `selftest()` 本身答不出这一格 |
| 同一个会话停过别的 id | 另写 `other-id-stopped-test.py`：主会话记录只停了 `otheragentabc`，没停 `targetagentxyz`；对 `targetagentxyz` 调 `task_stop_time`。实测：`task_stop_time(target) = None`，`state = 本轮结束、没交回, is_done() = False` | 对——70 行 `result.get("task_id") != agent_id` 精确匹配，实测印证没有被别的 id 的停止记录污染 |
| 主会话记录不在 `<会话目录>.jsonl` | 另写 `missing-parent-session-test.py`：故意不创建 `session + ".jsonl"` 这份文件。实测：不崩溃，但 `state = 本轮结束、没交回, is_done() = False` | **判不对（静默退化）**：文件缺失时 `task_stop_time` 的 `os.path.isfile` 检查（59-60 行）直接返回 `None`，`__init__` 里的覆盖条件因此不成立，看门狗悄悄退回到改之前的老判法——没有任何提示说「这次 TaskStop 检查没跑成」。会复现记录第三十三节那次「无动静」误报，只是触发条件从「没做这个改动」变成「文件路径对不上」 |

### 自检的两个样本分不分得出新旧判法

命令与原样输出（现跑）：

```
$ python3 research/scripts/agent-watch.py --selftest
  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静、进程无输出六种告警都报得出，交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具成功判（只结束本轮、交回后又被续做的都不算），被停认会话记录里的打断与主会话记录里的 TaskStop（停在等后台任务时的算、停了又被续做的不算），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒，用量与工具结果分类对（查了 15 个子 agent、1 个进程、8 次看门狗）
EXIT=0

$ AGENT_WATCH_BREAK=taskstop python3 research/scripts/agent-watch.py --selftest
  ✗ 自检：子 agent stoppedidle：应当告警 无，实际 ['无动静']
  ✗ 自检：子 agent stoppedidle 应当算交回或被停，实际状态「本轮结束、没交回」
  ✗ 自检：被看的子 agent 都交回或被停时看门狗应当退出码 0，实际 3
    → 看 agent_alerts() / process_alerts() / run() 的判法；AGENT_WATCH_BREAK 设着的话这里本来就该红
EXIT=1
```

**判定：分得出，但两个样本的作用不对称。** `stoppedidle` 这一格在旧判法（`AGENT_WATCH_BREAK=taskstop` 关掉 TaskStop 检查）下会告警、`is_done()` 判不完——这正是这条改动要修的真实场景（记录第三十三节）；`stoppedcontinued` 这一格在同一次判红开关下**没有**被打红（上面的输出里只列了 `stoppedidle` 的三条失败，没有 `stoppedcontinued` 的），因为它的作用是防止「停了又续做」被新判法误伤，而旧判法根本不看 TaskStop、本来就不会把它判成「被停」——所以它对「新旧判法分不分得出」这件事本身没有分辨力，是护栏（防止新判法反向出错），不是判别力来源。真正能分辨新旧判法的只有 `stoppedidle` 一个样本。

### check-staged.sh 四种组合：红、绿、全 77、77 混着红

`research/scripts/check-staged.sh` 判据代码（行号现查）：

```
75:    return 1        # 一个阶段都没跑到
77:  local not_run_note=""
78:  [[ ${#not_run[@]} -gt 0 ]] && not_run_note="；无对象没跑 ${#not_run[@]} 个：${not_run[*]}"
80:    echo "  ✗ HEAD + 暂存区上跑了 $ran 个阶段，红 $red 个${not_run_note}"
82:    return 1         # 红（不论混不混 77）
85:    echo "  ! HEAD + 暂存区上点名的 $ran 个阶段全是无对象可判，一个都没判（不记通过）"
86:    return 77        # 全 77
88:  echo "  ✓ HEAD + 暂存区上跑了 $ran 个阶段，判了的 $((ran - ${#not_run[@]})) 个全绿${not_run_note}"
89:  return 0            # 绿（不论混不混 77）
```

命令与原样输出（现跑；红、绿、全 77 三种由仓里内置的 `selftest()` 覆盖，我又补了一次判红开关）：

```
$ bash research/scripts/check-staged.sh --selftest
selftest: 通过（没暂存的 BAD 不算、暂存了的 BAD 判红、无对象可判的阶段不算红也不算通过、跑到一半被打断也清掉临时 worktree）
EXIT=0

$ CHECK_STAGED_77_IS_RED=1 bash research/scripts/check-staged.sh --selftest
selftest: 无对象可判（退出码 77）的阶段被当成红、或没在成功行里列名（退出码 1）
EXIT=1

$ CHECK_STAGED_USE_WORKTREE=1 bash research/scripts/check-staged.sh --selftest
selftest: 拿工作区原样去跑确认判红（没暂存的 BAD 被算进来）
EXIT=0

$ CHECK_STAGED_NO_TRAP=1 bash research/scripts/check-staged.sh --selftest
selftest: 关掉 trap 确认判红（打断之后临时 worktree 留在仓里）
EXIT=0
```

内置 `selftest()` 里含「绿 + 77 混合」（`run_isolated "$repo" 20 40`，此时阶段 20 还是干净的），但**没有**「红 + 77 混合」这一格（阶段 20 判红、同时阶段 40 是 77 的组合）。另起一个临时仓补了这一格（产物在 `research/prompts/bg-notify-r1-sonnet-model/red-plus-77-fixture/`，脚本本身未改）：

```
$ cd <临时仓>；git status --short
M  kb/a.md
$ bash <仓里的> research/scripts/check-staged.sh 20 40
  RED 20-red.sh
  ✗ kb 里有 BAD
    → 删掉
  77  40-none.sh（无对象可判，没跑）
  ✗ HEAD + 暂存区上跑了 2 个阶段，红 1 个；无对象没跑 1 个：40-none.sh
    → 红的是这一次提交带进来的（别人的未提交改动不在这棵树里）；先查它在不在 HEAD 上就红：暂存区清空再跑一次
EXIT=1
```

| 组合 | 退出码 | 失败/成功行是否列出 77 的阶段名 | 判定 |
|---|---|---|---|
| 绿（含「绿 + 77」混合，内置 selftest 的 `rc_mixed`） | 0 | 是（「；无对象没跑 1 个：40-none.sh」） | 对 |
| 红（无 77 混入，内置 selftest 的 `rc_bad`） | 1 | 不适用（没有 77 阶段） | 对 |
| 全 77（内置 selftest 的 `rc_none`） | 77 | 「点名的 N 个阶段全是无对象可判，一个都没判（不记通过）」 | 对——77 既不记成功也不记失败，符合 `show-me-test.md`「退出码 77 是『本次未跑』，不是通过」 |
| 红 + 77 混合（内置 selftest 没测；现补） | 1 | 是（「红 1 个；无对象没跑 1 个：40-none.sh」） | 对，但**仓里的 `selftest()` 没有直接测这一格**，是我现补的验证 |

**判定：四种组合的退出码与成功/失败行都对。**
推翻条件：任一组合下退出码与「红/绿/77」的实际情形不符，或成功/失败行没有把 77 的阶段名列出来——现跑都没有出现。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A3-引文核对 | 一致 | `.claude/main-agent.md:29` 两句英文都是通知原文（`_bg-notify-r1-background.md:39`）的逐字连续子串，`grep -nF` 全部命中，没有摘句、没有改写 |
| A3-情形一：结束本轮等后台 | 一致 | 入口「不当出错、照旧等」与看门狗 `is_done()=False`、只在闲置超时后给一条留给主 agent 判断的「无动静」告警，不冲突 |
| A3-情形二：交回 | 一致 | 入口「按 SubagentHandback 消息判」与看门狗 `is_handed_back` → 「已交回」一致 |
| A3-情形三：被 TaskStop 停掉 | 入口文本「规则没说」；实际行为「一致」 | `.claude/main-agent.md`、`.claude/agent-common.md` 全文 0 处提及 TaskStop；主 agent 靠工具自身返回值知道，行为上与看门狗新判法不冲突，前提是只有主 agent 自己会调用 TaskStop |
| A4-停在工具中途 | 对，且比文档描述更宽 | 没有 `[Request interrupted` 标记时 TaskStop 机制照样接得住，19 行「认两处」字面暗示的互斥不准确（45 行注释其实早说明了这一点） |
| A4-停了又续做 | 对 | 仓里自带 `stoppedcontinued` 样本，实测通过 |
| A4-续做之后又停 | 对，但无自检样本 | 现补测试证明 `task_stop_time` 的 `max()`（74 行）取到的是最晚一次，不是第一次 |
| A4-同一会话停别的 id | 对 | `task_id` 精确匹配（70 行），现补测试证实没有串号 |
| A4-主会话记录缺失 | 判不对（静默退化） | 现补测试证实：不崩溃，但悄悄退回改前的老判法，没有任何提示说这次检查没跑成 |
| A4-自检两样本分辨力 | 部分：`stoppedidle` 有分辨力，`stoppedcontinued` 是护栏 | `AGENT_WATCH_BREAK=taskstop` 只把 `stoppedidle` 打红，`stoppedcontinued` 不受影响 |
| A4-check-staged 四种组合 | 全对，但「红 + 77 混合」无自检覆盖 | 绿、红、全 77 由内置 `selftest()` 验证；「红 + 77 混合」现补验证 |

## 没做什么

- A1（两句新指令本身）、A2（hook 第三种检出，含表外命令）：不归我判，没碰这两格。
- 附录三 L1–L14 的逐格填表：本地攻方的格，没碰。
- 没有单独核 `.claude/hooks/bash-command-detector.sh` 本身的判法对不对——那是 A2；`agent-watch.py` 的 `read_detections()` 转发 hook 检出的部分我读了、`--selftest` 里带过（`hook 检出会叫醒主 agent` 那三次判红开关），但没有单独验证 hook 判据本身的字面对不对。
- 没有反过来核正文第二节「主 agent 收到的通知原文」这张表本身是不是真实 harness 输出的逐字抄录——那是主 agent 自己的观测（正文自陈「主会话里收到的通知逐字抄」），我只核了入口引文与这张表是否对得上，够不着主会话记录本身，无法反向核实这张表对不对（复核不了）。
- A3 情形三「规则没说」这一格判定为「行为不冲突」，但没有进一步判断这算不算 A3 判据字面要求的「打中」——那是主 agent 的判决职责，我只如实报告代码与文本在这个输入下的真实行为，以及支撑这个判断的前提（只有当前主 agent 自己会调用 TaskStop）没有写进文本。
- 没有跑 doc-lint 或别的门禁阶段检查本报告或产物本身的格式；没有编译 Rust，没有碰 `crates/`（这一轮判的是 agent 定义、hook 与协作脚本，正文已现查 `crates/` 里没有相关代码）。
