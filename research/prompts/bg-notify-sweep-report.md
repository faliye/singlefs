# bg-notify 阶段同步回扫报告

## 输入
- 阶段名：bg-notify（后台任务的通知与交回：共用约束、主 agent 入口、Bash 检出 hook、看门狗）。
- 改动范围：基准 HEAD `00c9d4f`；文件清单：`.claude/agent-common.md`（第 40 行「长活可以等」）、
  `.claude/main-agent.md`（第 21、23 行与「交回怎么读」）、`.claude/hooks/bash-command-detector.sh`、
  `research/scripts/agent-watch.py`、`research/scripts/check-staged.sh`。
- 做成的事（主 agent 给的六项，逐项已核对现状是否随之更新，见下）：
  1. 共用约束：`run_in_background` 起的命令要跑到活结束才退出，不许再自己放后台；缓存计时器单独起；结束本轮只写一句在等什么。
  2. 主 agent 入口：三种 `completed` 通知怎么读；`TaskStop` 停掉的不再交回；主 agent 自己的长命令与派 `general-purpose` 同样照写。
  3. Bash 检出 hook：第三种检出「`run_in_background` 里又自己放后台」；自检 40 种。
  4. 看门狗：新告警「结束本轮却不会醒」；主会话记录不在时报告写明被停只按打断判。
  5. `check-staged.sh` 自检补「红与 77 混着」一格。
  6. 真活证据：三方 `bg-notify-r1` 一轮；hook 实测收得到 `run_in_background`。

## 关键词（从改动范围与做成的事取）
文件/脚本名：`run_in_background`、`结束本轮`、`completed`、`TaskStop`、`交回怎么读`、`bg-notify`、`cache-keepalive`、
`bash-command-detector`、`agent-watch`、`check-staged`、`看门狗`、`60 秒`、`检出…种`/`种…检出`/`告警…种`/`种…告警`/`自检…种`（合并成一条正则）。
没有带简称的编号、没有步号：这一阶段的改动不涉及 kb 里的 D/E/C 编号或里程碑步号。

## 载体范围（按定义第 8 步）
`CLAUDE.md`、`README.md`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/`（15 个定义）、
`.claude/rules/`（7 个文件）、`.claude/kb/`（除 `decisions-history.md`、`experiments-history.md` 外，现查 198 个文件）、
`records/` 各文件「## 历史版本」以外的正文（现查：47 个 `records/*.md` 里只有本阶段命中的两份——
`records/2026-09-16-subagent拆分提案.md`〔历史版本节在第 764 行〕、`records/2026-09-17-已分配口径三方与两个实验.md`〔历史版本节在第 177 行〕
——需要在搜索后剔除历史版本节里的命中）。不搜 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改）。

## 搜索命令与计数
统一命令形态（示例，`<关键词>` 逐个替换；对正文的裁剪见下）：
```
grep -n "<关键词>" CLAUDE.md README.md .claude/agent-common.md .claude/main-agent.md \
  $(find .claude/agents -name '*.md') $(find .claude/rules -name '*.md') \
  $(find .claude/kb -name '*.md' | grep -v '^\.claude/kb/decisions-history\.md$' | grep -v '^\.claude/kb/experiments-history\.md$') \
  | cat
```
```
# records/：先按文件切掉「## 历史版本」标题之后的部分，再搜
for f in records/*.md; do
  ln=$(grep -n '^## 历史版本' "$f" | head -1 | cut -d: -f1)
  end=${ln:+$((ln-1))}; end=${end:-$(wc -l < "$f")}
  awk -v end="$end" -v fname="$f" 'NR<=end{printf "%s:%d:%s\n", fname, NR, $0}' "$f"
done | grep "<关键词>"
```
（两条命令的结果拼起来去重，即下表「计数」列；`看门狗`、`agent-watch`、`cache-keepalive` 三个关键词原始 grep（不剔除历史版本节）分别是 26、17、7，
剔除 `records/2026-09-16-subagent拆分提案.md` 第 764 行之后的 3 处命中〔第 787、790、791 行，分别含 `agent-watch`、`看门狗`+`agent-watch`、`cache-keepalive`+`看门狗`〕之后，才是下表的数。）

| 关键词 | 命中（剔除历史版本节之后）|
|---|---|
| `run_in_background` | 9 |
| `结束本轮` | 11 |
| `TaskStop` | 5 |
| `交回怎么读` | 3 |
| `bg-notify` | 1 |
| `cache-keepalive` | 6 |
| `bash-command-detector` | 4 |
| `agent-watch` | 15 |
| `check-staged` | 3 |
| `看门狗` | 24 |
| `completed` | 2 |
| `60 秒` | 2 |
| `检出…种`/`种…检出`/`告警…种`/`种…告警`/`自检…种`（合并正则）| 15 |

去重后（同一 `文件:行` 命中多个关键词只算一处）共 **57** 处，`sort -u` 现算：见下方命中清单，逐处分类。

## 命中清单（一）：这一阶段改动的 5 个文件本身

以下 4 处是这一阶段直接编辑或新增的内容（源头，不是被回扫对象，只核对地址与改动范围一致，不代入五类）：
`.claude/agent-common.md:40`（「长活可以等」新文本）、`.claude/main-agent.md:21`、`.claude/main-agent.md:23`、`.claude/main-agent.md:29`（「交回怎么读」新增段）。
已按改动范围核对：现查 `git diff 00c9d4f -- .claude/agent-common.md .claude/main-agent.md` 逐处对应，无遗漏。

同一份文件里另有 4 处命中，不是这一阶段编辑的行，按五类判：

### `.claude/agent-common.md:41`
> **结束本轮去等之前，再用 `run_in_background` 起一次 `bash research/scripts/cache-keepalive.sh`**：它 230 秒后退出，那条完成通知把你叫醒。被它叫醒就看一眼等的东西跑完没有：没跑完再起一次、结束本轮接着等，跑完了接着干、不用再起。自己已经交回或被停的不用管。预计单段要等 15 分钟以上的（层 0 全量、E152（按里程碑对比六家文件系统的文件性能） 那类），不起计时器。

判定：不相干（与这一阶段无冲突）。
理由：这一句在 diff 里是未改动的上下文行，讲的是 `cache-keepalive.sh` 单独起一次；把它换成这一阶段之后的说法仍然为真——`cache-keepalive.sh` 本身就是直接用 `run_in_background` 起、不在里面再放后台的写法，与新约束不冲突，无需改。

### `.claude/agent-common.md:42`
> 等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。

判定：不相干。
理由：「这类命令」是举例、不是穷举 hook 检出的种类（用词是「这类」不是「这两种」/「这三种」），第三种检出加进去之后这句仍然成立，不需要列出「第三种」。

### `.claude/main-agent.md:19`
> 盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `python3 research/scripts/agent-watch.py watch --agents <这次派出的 agent id，逗号分隔>`：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。

判定：**要改**。
理由：这句枚举了看门狗「退出并叫醒主 agent」的全部触发条件（用「发现 X、Y……或 Z，就退出并叫醒主 agent」的穷举语气），
但 `research/scripts/agent-watch.py` 这一阶段新加了第 7 种触发——`ended_turn_with_nothing_to_wake_it()`（`agent_alerts()` 里的
「结束本轮却不会醒」告警，见该文件第 260–266、325–328 行）：子 agent 结束本轮、没交回、起过的后台任务都已收到完成通知、
过 60 秒仍没有新通知，也会让看门狗退出并叫醒主 agent，而这句没提。把「结束本轮却不会醒」代入这句判据（换成新说法这句还是不是真话）：
不是——现在还有一种没列出的触发条件，原句作为「穷举」就不成立了。
改后的句子（在「进程写的文件 20 分钟不涨」之后插入）：
> ……10 分钟无动静、进程写的文件 20 分钟不涨、结束本轮而起过的后台任务都已收到完成通知却没有新通知会叫醒它（过 60 秒即报「结束本轮却不会醒」），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；……

### `.claude/main-agent.md:25`
> ## 交回怎么读

判定：不相干（只是小节标题，非内容句，标题本身没有随这一阶段过时）。

## 命中清单（二）：`.claude/agents/`、`CLAUDE.md`、`.claude/kb/`

### `.claude/agents/sweep.md:22`
> - 阶段同步（一个阶段任务结束、暂存之前）：阶段名；改动范围（基准提交，或文件清单）；这一阶段做成的事与第一次在真活上用上的东西，一行一件，没留改动的也写（例：第一次在真派发里用看门狗）；真活使用的证据由主 agent 在这里给，你不自己读会话记录（`~/.claude/` 照共用约束不碰）。

判定：不相干。
理由：这是 `sweep` 自己的定义正文，「看门狗」只是举例说明阶段同步这个活的形态，不描述 bg-notify 具体做了什么，不会因本阶段变假。

### `CLAUDE.md:10`
> **所有任务从 [.claude/main-agent.md](.claude/main-agent.md) 进**：主 agent 的职责、一轮怎么开怎么收（出口、判阻塞、收拢、再判）、派出去之后怎么盯、交回怎么读、派发提示怎么写、什么时候派哪个 agent 的调度表，都在那一份。这份文件只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实。

判定：不相干。
理由：只是指到 `main-agent.md`「交回怎么读」一节，不复述具体内容，main-agent.md 那一节改了它仍然成立。

### `CLAUDE.md:80`
> | `research/scripts/check-staged.sh` | 在临时 worktree 上只拿「HEAD + 暂存区」跑 doc-lint 与快的 kb 阶段 |

判定：不相干。
理由：只描述 `check-staged.sh` 的整体用途，不涉及自检内部有多少格、有没有「红与 77 混着」这一格，这一阶段的改动不会让它变假。

### `.claude/kb/tooling.md:14`
> | 单轮耗时 | 一句话问题 41 秒；一个含 54 行背景 + 两张清单的论证任务约 60 秒 | 实测 2026-08-26，两次采样，**不满足 N≥5，只作量级参考** |

判定：不相干（关键词「60 秒」误中：这里的「60 秒」是三方论证单轮耗时的实测量级，与看门狗「结束本轮却不会醒」的 60 秒宽限无关，纯属巧合）。

### `.claude/kb/tooling.md:417`
> 用户要会话到点自己开工（「确保你自己能唤醒」）时，只靠 CronCreate 不够。两条路径一起上：CronCreate 一次性任务，加一个 `run_in_background` 的 Bash（`sleep` 到目标时刻再退出，退出时 harness 会叫醒会话）。两条路径指向同一份写在 scratchpad 里的开工指令，第一步都用 `set -o noclobber` 排他建「已开工」标记，标记已在就什么都不做。

判定：不相干。
理由：讲的是「会话定时自唤醒」这个不同的机制（CronCreate + 单独一个 `run_in_background` 的 `sleep`），不是子 agent 起长活的后台命令写法，与这一阶段改的约束不是同一件事，不会被这一阶段的改动影响。

### `.claude/kb/tooling.md:580`
> | 只暂存自己的块 | `research/scripts/stage-mine.py --match 我的标记 --foreign 别人的标记 文件…`，先 `--dry-run` 看它挑了哪几块；`research/scripts/check-staged.sh` 在临时 worktree 上只拿 HEAD + 暂存区跑 kb 阶段。它会把 `.gitignore` 掉的 `.claude/singlefs-ai-sop/` 拷进 worktree；手工建的 worktree 没有这一步，`gate.sh` 退出码 127、日志只有一行 No such file，要先 `rsync -a --exclude .git` 把副本拷进去 |

判定：不相干。
理由：讲的是 worktree 里缺规范副本的踩坑，不涉及自检有没有「红与 77 混着」这一格。

### `.claude/kb/checks-owed.md:106`、`:210`、`:218`；`.claude/kb/decisions/18-块里携带什么信息.md:100`；
### `.claude/kb/experiments/35-多可写头两种形态.md:16`；`.claude/kb/experiments/98-inode记录与inode树的几何.md:104`；
### `.claude/kb/milestone/01-first-txn.md:43`

判定：不相干（正则 `检出…种`/`种…检出`/`告警…种`/`种…告警`/`自检…种` 的字面误中）。
理由：这 6 处分别讲的是 inode 容器丢失冗余条款（C100）、三方材料小节清单覆盖（C234）、格式冻结门禁触发（C225）、
扫盘重建信息（D18（块里携带什么信息）「检出误用」）、多可写头形态（「MAC 会失配…可检出的错」）、mkfs 现状（「验收五条全过」「三种槽 / 单元的解析」），
全部是文件系统设计与 kb 自身用词，和 bash-command-detector.sh 的检出种类、agent-watch.py 的告警种类、
bash-command-detector.sh 的自检种数无关，纯属字面「种」字撞上。

## 命中清单（三）：`records/2026-09-16-subagent拆分提案.md`（正文，历史版本节在第 764 行之前）

以下 33 处全部在带编号的日期小节（一至三十三节）里，逐条核对：换成「这一阶段之后」的说法，原句是不是仍然只是在叙述
「那一次发生的事 / 那一刻量到的数」——是，就是事件句，不改；下面统一给判定，理由只在与「唤起失败／看门狗种类数」这类
容易被误判成现状句的地方单独展开，其余因为形态相同（`新脚本 / 已改 / 已做 / 自检 N 种` 这类过去式陈述，前面都带日期）合并给理由。

### `:5`
> 用户同日定了九项……**进度**：`.claude/agents/` 下十六个定义……看门狗与续派闸 2026-09-17 起在真派发里用过……
判定：事件句不改。理由：带日期的进度快照（截至 2026-09-17），不是「现在看门狗有几种告警」的现状陈述。

### `:225`
> - 每个定义试跑过一次，改动最多的四个再试跑过一次（第十六至十八节）；对抗做了两轮（第十七、十九节），第二轮的改法被攻过零轮；真活上的用量读数用 `research/scripts/agent-watch.py cost` 现取（第二十一节第三批）。族一的输入与产出是照 C143（inode 号水位在回退后会退回去重发） 第三轮与里程碑「第二个事务」代码第一轮两套提示归纳的，别的轮次可能有别的形态。
判定：事件句不改。理由：讲的是 `agent-watch.py cost` 子命令用来取真活用量读数这件事，不涉及告警种类或检出种类。

### `:526`
> | E154 第四段执行员……| 新脚本 `research/scripts/agent-watch.py`：……六种告警（没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复、按模式找进程、10 分钟无动静、进程写的文件 20 分钟不涨）。……自检接进门禁 47 号 |
判定：事件句不改。理由：这是 2026-09-17（第二十一节）`agent-watch.py` **初版**做成时的记录，带日期、描述的是当时做出来的东西；「六种告警」说的是那一次做成时的数，不是「现在有几种」——现在已经是 7 种（多了「结束本轮却不会醒」），但这句没有断言「现在」，换成新说法这句本身不是在复述现状，是在叙述那一次做了什么，仍然成立，不改。

### `:527`
> | 没超时的等待循环与按模式找进程，事后才发现 | 新 hook `.claude/hooks/bash-command-detector.sh`……自检 10 种……
判定：事件句不改。理由：同上，2026-09-17 hook 初版做成时自检 10 种，是那一次的数，不是「现在自检几种」。

### `:529`
> | 主 agent 派发之后没有复检的步骤 | CLAUDE.md「什么时候派哪个 agent」一节加一段：……看门狗的「寿命上限」当天改成每 60 分钟定时回报……
判定：事件句不改。理由：带日期，讲的是那一天改了什么。

### `:532`
> | 续派之前「对着岔路单判够不够」靠自觉会被跳过 | 新 hook `.claude/hooks/runner-dispatch-guard.sh`……自检 11 种……
判定：事件句不改。理由：`runner-dispatch-guard.sh` 与这一阶段无关，自检种数说的是它自己的历史。

### `:534`
> | 子 agent 起步上下文约 11 万 token……第二次：回扫「看门狗 240 分钟寿命」这条已撤回的说法（`sweep` 与临时副本）……
判定：事件句不改。理由：讲的是当时一次 `sweep` 回扫试点的经过与用量对比，不是现状陈述。

### `:536`
> | 用量只能临时写脚本拆 | `research/scripts/agent-watch.py cost`：按 agent 报调用次数……自检钉绝对值 |
判定：不相干。理由：讲 `cost` 子命令做什么，与本阶段的告警/检出种类无关。

### `:538`
> | 两道写 hook 分开注册……| 合成 `.claude/hooks/write-guard.sh`……自检 27 种……看门狗读到就叫醒主 agent。主会话里实测整份覆盖一个未跟踪文件被拒、文件原样，看门狗 15 秒内读到检出并叫醒主 agent |
判定：事件句不改。理由：讲 `write-guard.sh` 的合并与自检，与 bg-notify 的三种检出无关。

### `:540`
> | 看门狗每遇到一个正常的等待循环……就立刻叫醒主 agent……| `research/scripts/agent-watch.py`：hook 只检出「没超时的等待循环」的记录不再立刻叫醒……自检加一格，`AGENT_WATCH_BREAK=loopnoise` 判红 |
判定：事件句不改。理由：讲的是 2026-09-17 修掉「等待循环噪音」的那次改动，带日期。

### `:542`
> **第三批没做的**：都已了结。`omitClaudeMd` 推广之后的起步上下文 2026-09-18 核过一次……已有的两道会拦的写 hook 与「hook 只检出不拦」这条是否冲突，交用户，用户答「应该考虑合并」，合并见下表。
判定：不相干。理由：讲写 hook 合并，与 bg-notify 无关。

### `:554`
> **同日第五批**（知识腐烂；用户同日指出「第三批没做的」里「看门狗与续派闸还没在真实验派发里用过」在用过之后没人改……）：
判定：事件句不改。理由：带「同日」时间锚点，讲的是知识腐烂那一批的起因。

### `:558`
> | 两句进度句在被推翻之后留着：看门狗 2026-09-17 16:51 UTC 起随真派发起过……| 回扫这份记录……主 agent 现查坐实后改了 15 处……同步记录 `research/prompts/knowledge-sync-gate-sync.md` |
判定：事件句不改。理由：带精确到分钟的时间戳，讲的是那一次回扫的经过与结果，不是本阶段。

### `:568`
> | 子 agent 的提示缓存是 5 分钟档……| 新脚本 `research/scripts/cache-keepalive.sh`：`run_in_background` 起、230 秒后退出……接进门禁 47 号。规矩写进 `.claude/agent-common.md`「不做」一节……
判定：事件句不改。理由：讲 `cache-keepalive.sh` 这个脚本 2026-09-17 建成时的经过；脚本本身「`run_in_background` 起一次、230 秒退出」的写法与这一阶段的新约束（不在 `run_in_background` 里再放后台）不冲突，仍然成立，但这句本身是叙述那一次建成的经过，不是在断言「现在共用约束长什么样」，不改。

### `:569`
> | 看门狗把「只结束本轮、还在等后台任务」的子 agent 当成已结束……——2026-09-18 派探针时实测中了一次 | `agent-watch.py`：状态按交回工具……自检加三个假子 agent……`AGENT_WATCH_BREAK=finished` / `handback` / `deadline` 各判红 |
判定：事件句不改。理由：讲的是「本轮结束、没交回」这个状态分类的引入（watchdog-taskstop 阶段的活），带日期，不是这一阶段新加的「结束本轮却不会醒」告警。

### `:575`
> - **ping 确实续上缓存**：结束本轮在等后台任务的探针收到消息后多一次调用，读缓存 120,539、新写 93。
判定：事件句不改。理由：一次探针实测的读数，带具体数字，是那一次观测。

### `:581`
> **第六批没做的**：唤起失败（后台任务跑完而通知卡在队列里）没有检查——当天在看门狗里加过一版告警，按用户「不要干涉那么多」撤了，欠账留在这里；……
判定：事件句不改（核实后仍为真，非本阶段致假）。理由：这句讲的是「通知卡在队列里、还没被送达」这类唤起失败——`agent-watch.py` 的 `ended_turn_with_nothing_to_wake_it()` 要求「起过的后台任务都已收到完成通知」（`live_background_tasks()` 为空）才会报「结束本轮却不会醒」；通知还卡在队列、任务尚未被记为收到完成通知的情形，`live_background_tasks()` 非空，函数直接返回 `False`，不会告警。把这句换成「这一阶段之后」的说法，它依然为真：唤起失败（通知卡在队列）仍然没有检查，欠账仍然成立，不改。

### `:585`
> 里程碑二收口第一波……派了 18 个子 agent，用量由 `research/scripts/agent-watch.py cost` 现取：
判定：不相干。理由：讲用量取数方式，与告警/检出种类无关。

### `:597`
> ⚠️ **0 次重写不是 `cache-keepalive.sh` 挣来的**：这一阶段只有崩溃验证员真起过它（2 次，每次 230 秒）……
判定：事件句不改。理由：一次阶段性实测复盘，带具体次数。

### `:599`
> ⚠️ **计时器只续了两轮就停**：崩溃验证员起层 0 全量……共用约束「结束本轮去等之前，再用 `run_in_background` 起一次」要的是每次被叫醒都重起一次，实测没有持续执行。
判定：事件句不改。理由：讲的是一次真活里计时器没有持续续起的实测缺口，不是在断言「现在共用约束写的是什么」（它引用共用约束是为了对照实测缺口，句子本身没有跟着这一阶段的新文本走样）。

### `:601`
> 看门狗在这一阶段起了 12 次以上，报警 3 次：一次「无动静」……另有一次把「层 0 输出不涨」报成疑似卡住……之后用 `--process-stale-minutes 90` 重起。
判定：事件句不改。理由：m2-wave1 阶段（与 bg-notify 不同阶段）的一次实测复盘，带具体次数。

### `:617`
> `.claude/hooks/bash-command-detector.sh` 按整条命令串匹配「按模式找进程」这一类。实测撞到：执行员写报告时用 `cat >> report.md <<'EOF'`……检出记录因此报「用了按模式匹配的进程命令」，看门狗把主 agent 叫醒，现查是误报。
判定：事件句不改。理由：一次误报实测记录，带具体场景，讲的是第一种检出（按模式找进程），不是第三种。

### `:706`
> 口径：会话 `32df8a2f` 的 `subagents/` 下 42 份会话记录；「整份重写」取 `python3 research/scripts/agent-watch.py cost --session-dir <会话目录>`……
判定：不相干。理由：讲一次量化口径，与告警/检出种类无关。

### `:729`
> | 设计用法（后台起、结束本轮、被它叫醒） | 4 段等待被叫醒，长于 5 分钟 2 段……
判定：事件句不改。理由：A/B 对比实测的一格数据。

### `:730`
> | 前台用法（没加 `run_in_background`，卡满 120 秒被转后台） | 22 次，全在 E142 执行员……
判定：事件句不改。理由：同上，实测计数。

### `:735`
> **三条还没做成检查的改法**：⓪ Bash 检出 hook 认出前台跑的 `cache-keepalive.sh`（没带 `run_in_background`）记进检出记录……① ……子 agent 的前台命令超时超过 240 秒……② ……
判定：事件句不改（核实后仍为真）。理由：这三条是 2026-09-18（第二十一节）分析出、当时还没做的改法；`bash-command-detector.sh` 现在的第三种检出管的是「`run_in_background` 里又自己放后台」，不是「前台跑 `cache-keepalive.sh` 没带 `run_in_background`」（前者是「起了后台却又把它藏起来」，后者是「该用后台却用了前台」，是两件事）；换成这一阶段之后的说法，这三条仍然没做，依然成立，不改。

### `:751`
> ## 三十三、TaskStop 停掉在等后台任务的子 agent，看门狗认不出「被停」（2026-09-18）
判定：事件句不改（小节标题，带日期）。

### `:753`
> 提交前派 `gate-triage` 跑全量门禁（`--staged`），它 13:58:29 UTC 结束本轮、等后台的 `gate.sh`；主 agent 14:01:49 用 TaskStop 停掉它……那是一次误报。
判定：事件句不改。理由：带精确时间戳的一次事件复盘。

### `:757`
> | 看门狗判「被停」只看子 agent 会话记录 | 子 agent 会话记录最后一条 13:58:29……| 已改：`research/scripts/agent-watch.py` 另读主会话记录……`AGENT_WATCH_BREAK=taskstop` 判红；对这一次的子 agent 重跑 report 报「状态=被停」、没有告警 |
判定：事件句不改。理由：`已改：……` 是过去式，讲的是 watchdog-taskstop 阶段做的修法，这一阶段的 sync 已明确交代过（不属于 bg-notify 新做的事）。

### `:758`
> | 主 agent 收到的「completed」通知里写……用户看成出错 | 那是 harness 在子 agent 结束本轮、后台任务还在跑时发的中间通知……| 已改（用户定修）：`.claude/main-agent.md`「交回怎么读」写明三种 completed 通知各怎么读……看门狗加「结束本轮却不会醒」告警。三方一轮 `research/prompts/bg-notify-r1-main-verification.md` |
判定：事件句不改。理由：这正是 bg-notify 这一阶段的起因与「已改」的原始记录，本身就是源头描述，与当前实现一致（已用本报告开头逐处核对）。

### `:759`
> | 子 agent 用 `run_in_background` 起命令时又在命令里加了 `nohup … & disown`……| 子 agent 会话记录 13:52:19–13:53:44；……| 已改（用户定修）：共用约束「长活可以等」写明……`.claude/main-agent.md` 第 21、23 行同样带上；Bash 检出 hook 加第三种检出……自检 40 种。同一轮三方判决 |
判定：事件句不改。理由：同上，起因与「已改」记录，与本报告开头核对的实现一致（`.claude/main-agent.md` 确实是第 21、23 行改了，hook 自检确实是 40 种，现查一致）。

### `:760`
> | 同一次排查里 `research/scripts/check-staged.sh` 把门禁 72 号的退出码 77……报成 RED | 同一棵树上单跑 72 号退 77……| 已改：77 单列「无对象可判，没跑」……自检加一格，`CHECK_STAGED_77_IS_RED=1` 判红 |
判定：事件句不改。理由：这是上一阶段 watchdog-taskstop 已经同步过的 `check-staged.sh` 77 号修法（派发提示已明确交代过），这一阶段只新加了「红与 77 混着」一格，属于同一处代码但不同的修法记录，不冲突。

### `:762`
> 第一轮三方打中的都写回了，写回的六项被攻过零轮（判决第三节）：hook 的第三种检出换成攻方腿的变体……按 `.claude/rules/implementation-workflow.md` 应再攻一轮，用户要求这一批做完就提交收尾，第二轮没开。第二轮要攻：hook 变体在真实命令上的误检率、「结束本轮却不会醒」的 60 秒宽限在通知排队时会不会误报、共用约束那句的放后台清单还漏不漏。
判定：事件句不改（核实后仍为真：第二轮确实还没开，现查工作区里没有第二轮三方材料）。理由：这是 bg-notify 第一轮三方判决的记录，「第二轮没开」在写这份报告时仍然成立——`research/prompts/` 下没有第二个 `bg-notify-r2-*` 材料。

## 命中清单（四）：`records/2026-09-17-已分配口径三方与两个实验.md`（正文，历史版本节在第 177 行之前）

### `:162`
> | 1e | 续派闸（hook）：续派执行员的提示必须点名上一段表里还差的岔路，一条不差就拒绝 | 已做：`.claude/hooks/runner-dispatch-guard.sh`，注册在 Agent 上，自检接进门禁 63 号；实派一次被当场拒绝，2026-09-18 第一次在真派发里放行（派 E155（每次持久化的写量：三种 fsync 形态与反事实上界） 执行员做第一、二段） |
判定：不相干。理由：讲 `runner-dispatch-guard.sh`，与 bg-notify 的三个改动点无关。

### `:166`
> | 3 | `research/scripts/agent-cost.py` 按 agent 报调用次数……| 已做：统计做成 `research/scripts/agent-watch.py cost`；新旧定义各跑一次同一件确定性的活，结果见第 2b 行 |
判定：不相干。理由：讲 `agent-watch.py cost` 子命令，与告警种类无关。

### `:167`
> | 4 | 子 agent 与脚本的定时监控：……| 已做：`research/scripts/agent-watch.py`（watch 每 4 分钟复检、每 15 秒读 hook 检出记录，有告警叫醒主 agent，没告警每 60 分钟定时回报；不停任何子 agent、不杀任何进程）、`.claude/hooks/bash-command-detector.sh`（只检出不拦，写检出记录）……当天两次按用户纠正改过：不是不许阻塞，是阻塞超时后严密监视；hook 不能终止任务和脚本，只检出、交主 agent 判断 |
判定：事件句不改。理由：2026-09-17 岔路单的「已做」登记，带日期，讲的是 `agent-watch.py`／hook 初版做成时的形态，不是「现在」的种类数。

## 结论：唯一需要改的地方

只有 **`.claude/main-agent.md:19`** 一处是「这一阶段之后不再是真话」的现状句：它枚举看门狗退出并叫醒主 agent 的全部触发条件，
没有把这一阶段新加的「结束本轮却不会醒」算进去。改后的句子见命中清单（一）该条。

其余 56 处，`records/` 里的 33 处（`2026-09-16-subagent拆分提案.md`）与 3 处（`2026-09-17-已分配口径三方与两个实验.md`）
全部是带日期或带精确时间戳的事件句（`新脚本 / 已改 / 已做 / 自检 N 种 / 实测` 这类过去式，且句子的主语与时态锚定在某一次改动或某一次观测上），
换成「这一阶段之后」的说法后仍然只是在叙述那一次发生的事，不改；`.claude/kb/` 与 `CLAUDE.md`、`.claude/agents/sweep.md`
里的 20 处全部是关键词字面撞上但话题不同（`check-staged.sh`／`cache-keepalive.sh`／`agent-watch.py`／`60 秒`／「种」字 的其他用法），不相干。
没有「要补」的命中：这一阶段六件做成的事逐一反查，各自的载体里都记着——共用约束在 `agent-common.md:40` 本身、
主 agent 入口三种 `completed` 通知在 `main-agent.md:29` 本身、hook 第三种检出在 hook 自己的文件头与 `findings_for()`、
看门狗新告警在 `agent-watch.py` 自己的文档字符串与 `agent_alerts()`、`check-staged.sh` 的新自检格在它自己的 `selftest()`、
真活证据在 `research/prompts/bg-notify-r1-*`（未搜，按定义排除）。没有「分不清」的命中。

## 没做什么
- 没改任何文件（按定义「写范围」只写报告与草稿目录）。
- 没有格式常量改动、没有撤回的结论、没有新立的判据，第 1–6 步不适用，只走了第 7–9 步。
- 没有搜 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改），bg-notify-r1 那一轮的产物与判决没有回扫。
- `~/.claude/` 没有碰，没有自己读会话记录；6 号「hook 实测收得到 run_in_background（14:45:56 UTC）」这条证据没有另外验证，按定义由主 agent 给。
- kb 正则「检出…种／种…检出／告警…种／种…告警／自检…种」命中的 6 处 kb 文件（`checks-owed.md` 等）只核了这一次误中的话题是什么，
  没有反过来确认这 6 处自身的「种」用词准不准——那是另一件事，不在这一阶段的回扫范围内。
- main-agent.md:19 之外，没有再往下核实 `agent-watch.py` 现在一共几种告警名（等待循环 / 工具调用过长 / 同一命令反复且输出不变 / 禁用命令 /
  无动静 / 结束本轮却不会醒 / 进程无输出 / 进程过长 / 没启动 / hook 检出，逐个数了但没有登记进任何 kb），因为这不是「这一阶段变成假话」，
  是「这份枚举从建立起就不完整」（`main-agent.md:19` 本来就没提「进程过长」「没启动」），只挑了这一阶段新加的那一种判定要不要改，
  其余缺项留给主 agent 判断要不要一并补。
