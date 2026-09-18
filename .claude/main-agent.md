# 主 agent：任务入口与职责

**所有任务从这里进。** 这份只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`，公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。

## 职责

主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下面那张表派。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活照旧临时写提示派 general-purpose。

## 一轮怎么开、怎么收

1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。
2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在修；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
3. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
4. **本轮出结论之后回到那张表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
5. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。

## 派出去之后

盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `python3 research/scripts/agent-watch.py watch --agents <这次派出的 agent id，逗号分隔>`：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。

子 agent 的提示缓存是 5 分钟档，由它自己续（`research/scripts/cache-keepalive.sh`，共用约束「不做」一节），主 agent 不定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上这一条。

叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，由看门狗盯着进程。

## 交回怎么读

判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。

## 派发提示怎么写

在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。

## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，有腿交了模型或产物就派 `three-way-verifier` → 主 agent 写判决；三轮之后停 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep`（阶段同步；输入给改动范围与做成的事，只用过、没留改动的也写）→ 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
| 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 要别家文件系统的事实 | `prior-art` |
