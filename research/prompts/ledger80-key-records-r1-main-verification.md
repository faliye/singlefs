# ledger80-key-records-r1：判决（2026-11-12）

正文 `research/prompts/_ledger80-key-records-r1-body.md`，开工快照 `research/prompts/ledger80-key-records-r1-snapshot/`。这是这一题的第一轮，没有前几轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/ledger80-key-records-r1-sonnet-output.md` | 第 1 条甲「成立，但理由要改」，乙「对治交替撞顶不成立；对治多记账推不出」，丙「推不出」；第 4 条甲「成立，理由要改」，乙、丙「推不出」；两条「一起量，分开判」 |
| 攻方（Opus） | `research/prompts/ledger80-key-records-r1-opus-output.md` | 用「80 把交替的键」证明第 1 条有现象：打中；第 1 条甲、乙、丙与第 4 条甲、乙都打中，第 4 条丙条件式打中；两条「要合」 |

核查：

- 快照树：`task-server/task submit --cwd ~/.local/share/singlefs/work/ledger80-r1-verdict -- python3 <仓根>/scripts/agents/snapshot-tree.py ledger80-key-records-r1 --out ~/.local/share/singlefs/work/ledger80-r1-verdict/ledger80-key-records-r1-tree --repository <仓根>`（任务 14205，退 0）：「清单点名 4 份，倒推出 4 份，sha256 对上 4 份」。腿交齐之后 `sha256sum -c research/prompts/ledger80-key-records-r1-snapshot/judged-sha256.txt` 三份都 OK，腿跑着的时候被判的文件没人动过。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/ledger80-key-records-r1-sonnet-output.md research/prompts/ledger80-key-records-r1-opus-output.md --root ~/.local/share/singlefs/work/ledger80-r1-verdict/ledger80-key-records-r1-tree --background research/prompts/_ledger80-key-records-r1-background.md`：「核了 90 处引文、42 处裸行号，对上 77 处，没判 55 处（其中裸行号没有引文可比 32 处）」，对不上 0 处。没判的里路径认不出的 23 处都指向快照树之外的文件（`records/2026-11-10-全量提交门禁收尾调度.md`、`scripts/hosts/memory_peaks_lib.py`、`task-server/task_server_selftest.py`、`.claude/agent-common.md`、`scripts/mutation/crates-mutation-rows.py`）；判决用到的那几处主 agent 逐处读过，写在这一节的现查清单里。
- 正文的行号有三处写早了（正文写在任务服务批 13 落盘之前，快照取在之后）：`RECORD_WINDOW` 在 `task-server/task_server.py:1071`（正文写 1059），`key_stats` 在 `task-server/task_server_lib.py:5794`（正文写 5783），第八十节两条判断在 `records/2026-10-29-subagent拆分提案.md` 第 288、291 行（正文写 287、290）。两条腿都指了出来，都对。
- 两条腿都没有交模型、没有要主 agent 复跑的读数：攻方的数例是手算的、标了「推的」；正推写了一份重放脚本（`~/.local/share/singlefs/work/ledger80-r1-forward/window-replay.py`），没跑过，它要读的尝试记录今天导不出来。所以这一轮判决里没有新量的数，只有正文第三节那份 `control key-stats` 的读数（主 agent 直接起的）与主 agent 读代码坐实的事实。

主 agent 逐条现查坐实的（读的是工作区此刻的文件，与快照相同）：

1. 统计里「成、撞、成」与「撞、成、撞」都算交替：`task-server/task_server_lib.py` 的 `key_stats` 数的是近 5 次尝试的标记翻面至少两回，不分是不是同一个任务里的重试。坐实。
2. 统计不筛峰值口径、不核片数：`key_stats` 只看 `state == "ended"` 与有命令键；定额度的 `recorded_measurements`（`task-server/task_server.py:9737` 起）另筛 `current_metric` 与 `record_fits_task`。坐实：两处读的不是同一个窗口。
3. 读数里的记录多数产生在上限乘 1.5 的那一版：`records/2026-11-10-全量提交门禁收尾调度.md` 第 77、78 行记着用户定「上限和记账都等于近五次最大」「这个不再乘以1.5」。坐实。
4. `memory-peak-grew`：全仓 grep，任务服务里只有 `task-server/control:536` 在读、`task-server/task_server_lib.py:5791` 定名字、自证里造样本，没有一处写这种告警。坐实：读数里那一行「0 条」不代表没涨。
5. 五条按项入口的现判与复用条数服务已经记着（`task-server/task_server.py:7163` 一段把 `ITEM_PLAN` 行记进尝试的 `item_plan`）。坐实有这个字段；它与峰值相不相关没量过。

## 二、判决

1. 「近 5 次里交替的键 80 个」不构成第 1 条的现象。打中，采。它按尝试数、把同一任务里撞了重试跑成的也数进去，不筛口径与片数，记录跨了几版定额度代码；它既不能说明「窗口里的记录太低」，也不能说明没有。第八十节当时的「交替 0 个」同样不再是依据。
2. 第 1 条乙（撞上限的记录按次数衰减）：不做。两条腿一致，主 agent 照代码复核成立：衰减只会让上限比甲低或相同，撞的次数只增不减；同键混了大小工作量时压住窗口的是大活跑成那一次的峰值，不是撞的那一条，衰减碰不到它。
3. 第 1 条甲（不动）与第 4 条甲（不做）：这一轮都不改定法，两条原来的理由都撤掉。第 1 条的「没有它要治的现象」、第 4 条的「近 5 次最大，偏大不偏小」都被攻方的数例 A 打中（同键大活之间隔了 5 个以上小活时，近 5 次最大比大活要的小，大活从低处一级一级撞上去；间隔不到 5 个时小活都按大活记账）。数例是手算的、照今天的代码推的，没量过它在真记录里占多少，所以判「理由撤掉、定法暂不动」，不判「甲成立」。
4. 第 1 条丙（交替的键加余量或放长窗口）、第 4 条乙（调用方带指纹）、第 4 条丙（服务自己取粗指纹）：推不出，挂起到量过。攻方对三者各给了一个会变坏的数例（都是推的）：丙的触发条件看不见数例 A 这一类；拿项数当指纹时项数少的那一次可能正好是重的；指纹没见过时退回哪里正文没写。这些是线索，要重放真记录才知道各占多少。
5. 两条合成一题。两条腿的分歧只在说法（「一起量，分开判」对「要合」），事实一致：两条量的是同一件事，同一把命令键下混着工作量不同的活。下一轮按一题开，候选重列。
6. 这一轮没攻的：`cap_hit_measurement` 的三支取法、任务内的重试阶梯，只当前提用。

## 三、改法

都是任务服务一侧的，并进任务服务下一批，照工具改动的流程走（自证格与开关）：

1. `control key-stats` 那一行「告警 memory-peak-grew：N 条」删掉，连同 `KEY_STATS_GREW_ALERT`。量过（主 agent grep，服务里没有写这种告警的地方）。
2. `key_stats` 的统计改成与定额度读同一个窗口、按任务数：筛峰值口径与片数（与 `recorded_measurements` 同一套筛法）；交替按任务数（同一任务里的重试折成一条，只看任务的第一次尝试撞不撞）；另报「只取某一时刻之后起跑的尝试」的那一份（参数给时刻）。零轮：这是攻方 1.9 节量法的前两步，没被攻过。它只改只读的统计，不改定额度，A/B 不适用。
3. 加一个只读导出：`control key-stats --dump <文件>`，把收尾了的尝试逐条写成 jsonl（攻方 1.9 节点名的那几列）。零轮，理由同第 2 条。
4. 重放：拿导出的记录照攻方 1.9 节的分类（并发、设计上的 250、口径量不全、记录退出了窗口、从没见过这么大）数一遍，并对每个候选数「多撞几次、多记多少字节」。正推那份 `window-replay.py` 没跑过、没自证，不直接当判据：主 agent 交它跑之前先照它报告里写的红绿证明跑一遍（把每个 `memory_cap` 改大，重放校验那一行应当几乎全对不上）。读数出来开第二轮。
5. 定额度的代码（`recorded_measurements`、`cap_hit_measurement`、读窗口取最大那一段）这一轮一行不改。

## 交用户的

- 没有要当场拍板的岔路。第 1 条丙的「乘余量」与用户 2026-11-10 定的「不再乘 1.5」方向相反，第二轮如果量出来它是唯一能少撞的候选，届时带着数交用户定；现在没有数，不交。
- 第八十节的两条回看到此算做过一次：结论是「读数不够判，先补量法」，不是「不做」。

## 这一轮被判过的文件

- task-server/task_server.py：判了按命令键定额度的取法（`recorded_measurements`、`cap_hit_measurement`、近 5 次取最大）在同键混了大小工作量时的行为；定法暂不动、原理由撤掉。
- task-server/task_server_lib.py：判了 `key_stats` 的统计口径，它的「交替」与告警两行撑不起结论，照第三节第 1、2、3 条改。
- records/2026-10-29-subagent拆分提案.md：判了第八十节第 1 条与第 4 条的回看，两条合成一题、挂到量过之后的第二轮。

## 回看决策

不涉及决策：这一题判的是任务服务按命令键定内存额度的做法与账本里的两条欠账，不在 `.claude/kb/decisions/` 的任何一条决策射程里。
