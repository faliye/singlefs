# 主 agent：任务入口与职责

**所有任务从这里进。** 这份只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它），公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。

## 职责

主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下面那张表派。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活照旧临时写提示派 general-purpose。

## 禁止
- **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
- **禁止自行扩大任务范围，只改自己的部分**。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
- **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**

- **禁止在subagent中跑重型测试**（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准）；例外只有提交时（或用户要求时）的 `crash-verifier`、`gate-triage` 各跑自己那一份，见那一节「场合 / 跑不跑」表。

## 一轮怎么开、怎么收

1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。
2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在修；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
3. **重型测试**（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准）提交之外任务确实要跑，先弹窗问用户，同意了才跑（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」）。
4. **派实现员之前先列出它要动的 `crates/` 文件**，与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件。
5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
6. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
7. **本轮出结论之后回到第 2 步记下延后项的那几处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文）与第 8 步的收拢表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
8. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，发消息协商。

## 派出去之后

盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（各拒什么写在各钩子文件头，钩子登记在 `.claude/settings.json`，清单用 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 列）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，有告警就退出并叫醒主 agent（告警有哪几种、各自阈值与什么时候报，见 `research/scripts/agent-watch.py` 文件头与 `research/scripts/watch.conf`；其中没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程确认接着盯写 `--ack 进程:<pid>`，「跑满 N 小时要主 agent 问一次」一格多长在 `watch.conf` 的 `ask-every-minutes`）；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。

改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。

子 agent 等长活时不续提示缓存；主 agent 除了看门狗报「跑满 N 小时」时的那一条例行询问，不另外定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。

叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。收到「跑满 N 小时要主 agent 问一次」就给它发一条例行询问：在做什么、还差几步、在等哪个进程、预计多久，等的已经结束就接着做或交回，并写明把回答写进它草稿目录的 `progress.md`（子 agent 多半没有 SendMessage）；它结束这一轮之后读那份文件，没写就读它会话记录里最后一段正文；发完再起看门狗。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。

## 交回怎么读

判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。

交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）：要补的活新派一个同类 agent，派发提示指到它的报告与产物；要它会话里的进度，用 `research/scripts/agent-handover.py` 抽交接摘要给新 agent 读。还没交回、在干活或在等自己后台任务的，照旧能收整点询问与纠正。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。

## 派发提示怎么写

同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数变了，给在跑的 agent 发消息改 N。

在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。

## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派；不派的在判决里写明为什么）→ 主 agent 写判决；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：层 0 全量主 agent 跑，派 `crash-verifier` 跑 QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；平时不跑） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：层 0 全量主 agent 跑，`crash-verifier` 跑 QEMU、herd7、crates 变异表 |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键），主 agent 自己在后台跑 `.claude/gate.d/54-layer0-replay.sh` 里 `print_staged_worktree_full_commands` 打印的那三行命令，三行放进同一次 Bash 调用（它们共用一个 shell 变量；建 HEAD + 暂存区的 worktree、带 `SINGLEFS_HEAVY_TESTS=commit` 经内存包装跑那棵树里的 `--full`、删 worktree；命令只在那一处写，不在这里抄）（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯；全量判绿、全绿标记写好之后才往下，不然整轮门禁里的 54 号必红 → 派 `crash-verifier` 跑 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊：54、55、59、74 号只在这一趟里能复用上一次整轮全绿的判定，判据在 `research/scripts/stage-must-run.sh` 文件头；要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑；用户要求时各处都换成 `=user-request` |
| 改了一个数或格式常量、撤回一条结论、新立一条判据 | `sweep`（四种活与各自要给的输入见它的定义） |
| 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ 红在 `research/mutations/` 的表：`experiment-runner` 只修锚点；红在 `crates/mutations.tsv`（`relabel-item.py` 改写了 `crates/` 下的源码）：派 `implementation-writer` 修锚点，走代码轮 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 要别家文件系统的事实 | `prior-art` |
