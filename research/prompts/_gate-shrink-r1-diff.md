# 附录二：gate-shrink-r1 定义与规则改动（`git diff HEAD -- <C 节列出的 19 个文件>` 原样；基准 HEAD e5253e8a，取的时刻是派发时的工作区，生成日期 2026-09-28）

## 一、归属：逐份标这一轮（gate-shrink）改的是哪几块、哪几块不是

**核查结论与派发提示不完全一致**：派发提示只点名 `kb-scribe.md` 与 `format-evolution.md` 被 singlefs-0a 的 changelog-format-r1 动过；逐行核对 diff 之后，`CLAUDE.md` 与 `kb-spec-drafter.md` 也带着 changelog-format-r1 的改动（同一个信号：提到 `.claude/rules/changelog-format.md`、「当月变更史文件」「按月」这类决策变更史存储形态的措辞），一并列在下面。核对法：`grep -n "changelog-format\|当月\|按月\|decisions-history/" <diff>`，命中的 4 个文件逐处判；其余 15 个文件全篇搜不到这个信号。

**以下 15 个文件的全部改动都属于这一轮（gate-shrink）**：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/three-way-materials.md`、`.claude/agents/prior-art.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/rules/implementation-workflow.md`、`.claude/rules/implementation-first.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/mutation-sampling.md`、`.claude/rules/verification.md`、`.claude/rules/path-moves.md`。依据：这批改动的内容是 Q9（「扫仓的门禁只在提交前跑」）、Q8（doc-lint 挪到提交时）、Q15（看门狗放行样本自检）、门禁编号改名（79 道合成 23 道、去编号）与记时区/记时刻规矩的改动，与各份实现员报告（`/tmp/claude-1000/gates-commit-only/report.md`、`/tmp/claude-1000/doc-lint-commit-time/report.md`、`/tmp/claude-1000/gate-merge-a` 到 `-i`、`/tmp/claude-1000/gate59-dual-host`、`/tmp/claude-1000/gate59-memory`、`/tmp/claude-1000/gate59-parse-deletion`、`/tmp/claude-1000/gate59-speedup-a`、`/tmp/claude-1000/gate12-17-merge`、`/tmp/claude-1000/gate-decision-status-merge` 各份 `report.md`）一致：这些报告只涉及门禁阶段合并、改名、Q8/Q9/Q15 与记时规矩，不涉及决策变更史存储形态。派发提示未点名 `three-way-verifier.md` 的删除与 `three-way-local-attack.md`（会话开始前已在暂存区），本材料不审这几处。

**另外 4 个文件的改动分两半**，一半是这一轮（gate-shrink 的门禁改名 / Q9），另一半不是这一轮的（来自 singlefs-0a 的 changelog-format-r1，改的是决策变更史的存储组织形态）：

### `.claude/agents/kb-scribe.md`

| 位置（旧串前几个字） | 归属 | 依据 |
|---|---|---|
| frontmatter `description:` 里删掉「并跑 kb 门禁阶段」 | 这一轮（gate-shrink，Q9：交回前不跑门禁） | 与第 4 步「门禁阶段交回前不跑，由提交前 gate-triage 跑」一致 |
| 「开工先读：」一行新增 `.claude/rules/changelog-format.md` | **不是这一轮**，changelog-format-r1 | 这一行只引新增的变更史组织规则，不涉及门禁编号或 Q9 |
| 「标题里「（其N）」那一段由你在写之前照当月文件现取」→「照 `decisions-history.md` 对应决策节里那个日期块现取」 | **不是这一轮**，changelog-format-r1 | 变更史存储从「当月文件」改成「按决策分节」，是存储组织形态的改动 |
| 第 1 步「当月变更史文件」这一分句被删掉 | **不是这一轮**，changelog-format-r1 | 同上，不再按月份记 |
| 第 2 步整段重写（原文写进当月的 `decisions-history/<年-月>.md`、49 号 `--write` 相关 → 原文写进 `decisions-history.md` 对应决策的 `## D<n>（简称）` 节里、按 `changelog-format.md` 组织、`30-decision-history-entries.sh --check shape` / `--write`） | **不是这一轮**，changelog-format-r1（存储组织形态），但其中「49 号」删除、改叫 `30-decision-history-entries.sh` 这个**编号本身**是这一轮的改名 | 存储形态是 changelog-format-r1；门禁编号从「49 号」变成「30-decision-history-entries.sh」是 gate-shrink 的去编号改名，两件事同一段文字里各占一半 |
| 第 3 步「它改写了 `research/**/*.rs` 的，加跑 33 号」→ 删掉「加跑 33 号」 | 这一轮（gate-shrink，Q9：实现员不跑重阶段） | 与「不做」一节「重型测试…子 agent 一律不跑」一致 |
| 3b 段删掉「写完跑 75 号：它报『不对称』…」整句 | 这一轮（gate-shrink，Q9：门禁交回前不跑） | 与第 4 步改动同一逻辑 |
| 「每次写入之后，写入后钩子…」整段删掉 | 这一轮（gate-shrink，Q9：钩子按阶段跑改成交回前不跑） | 与第 4 步「门禁阶段交回前不跑」一致 |
| 第 4 步整段重写（阶段归属表逐条跑 → 「门禁阶段交回前不跑，由提交前 gate-triage 跑的 `gate-staged.sh`…」） | 这一轮（gate-shrink，Q9） | 与 `.claude/agent-common.md`「门禁」一节改动同源 |
| 写范围一节「`relabel-item.py` 与新建当月变更史文件走 Bash」→「`relabel-item.py` 走 Bash」（删掉「新建当月变更史文件」） | **不是这一轮**，changelog-format-r1 | 不再新建当月文件，是存储组织形态的改动 |
| 产出一节「各门禁阶段结果与归属」→「第 2、3 步 `--check shape` 与 `--write` 的原样末行与退出码」 | 这一轮（gate-shrink，Q9：不逐阶段跑，只跑写回流程自己的检查） | 与第 4 步改动一致 |

### `.claude/agents/kb-spec-drafter.md`

| 位置（旧串前几个字） | 归属 | 依据 |
|---|---|---|
| 「要动的 kb 文件；当月变更史文件（`decisions-history/<年-月>.md`）」→「要动的 kb 文件；`decisions-history.md`（决策变更史，组织形态见 `changelog-format.md`）」 | **不是这一轮**，changelog-format-r1 | 这是这个文件唯一的一处改动，整段都是变更史存储组织形态的改动 |

### `.claude/rules/format-evolution.md`

| 位置（旧串前几个字） | 归属 | 依据 |
|---|---|---|
| 「硬约束」一节「决策变更必须记进…（原文写进当月的 `decisions-history/<年-月>.md`…再跑 49 号 --write…）」→「原文写进 `decisions-history.md` 对应决策的节里，组织形态见 `changelog-format.md`」 | **不是这一轮**，changelog-format-r1 | 存储组织形态改动，且新引了 `changelog-format.md` |
| 同节「并把推翻依据写进当月的决策变更史」→「并把推翻依据写进决策变更史」（删「当月的」） | **不是这一轮**，changelog-format-r1 | 同上 |
| 「49 号判形态」→「34 号的 decision-links 格判形态」 | 这一轮（gate-shrink，门禁改名合并） | 纯编号 / 阶段名改动，不涉及存储组织 |
| 「门禁 31 号定位不到」→「门禁 20 号格『未定项判过改不改新池新建文件的字节』定位不到」 | 这一轮（gate-shrink，门禁改名合并） | 同上 |
| 「门禁 75 号按分项查依据」→「门禁 34 号的 decision-links 格按分项查依据」 | 这一轮（gate-shrink，门禁改名合并） | 同上 |
| 「门禁管哪一半：`.claude/gate.d/75-decision-experiment-links.sh` 判…」→「`.claude/gate.d/34-doc-experiment-pages-and-products.sh` 的 decision-links 格判…」（含「路径与结论登记」一节由 `99-multipath-registry.sh` 判 → 「同一道的 multipath-registry 格判」） | 这一轮（gate-shrink，门禁改名合并） | 同上 |
| 「`.claude/gate.d/27-format-constants.sh` 只核…」→「`.claude/gate.d/27-code-constants-enums-bits-match-kb.sh` 的 format-constants 那一格只核…」 | 这一轮（gate-shrink，门禁改名合并） | 同上 |

### `CLAUDE.md`

| 位置（旧串前几个字） | 归属 | 依据 |
|---|---|---|
| 「跑相关门禁：47、62、63、doc-lint，改了定义与共用约束的走一轮三方或由用户逐份豁免（72 号判），改了触发文件的写阶段同步记录（68 号判）…」→「只跑改过的脚本或门禁阶段自己的自证与样本；门禁归提交时…改了定义与共用约束的走一轮三方或由用户逐份豁免（11 号 agent-def-adversarial-review 那一格判），改了触发文件的写阶段同步记录（11 号 knowledge-sync 那一格判）」 | 这一轮（gate-shrink，Q9 与门禁改名合并） | 编号从 72/68 改成 11 号的两个格，且措辞改成「门禁归提交时」，与 Q9 一致 |
| 新增一行 `@.claude/rules/changelog-format.md` | **不是这一轮**，changelog-format-r1 | 新增的规则引用只服务于变更史存储组织形态，不涉及门禁 |
| 表格行「`.claude/kb/decisions-history.md` \| 决策变更史按决策的汇总，由 49 号 `--write` 从原文生成；原文按月在 `decisions-history/<年-月>.md`…」→「决策变更史，一条决策一节、节内是它的完整历史，组织形态见 `changelog-format.md`」 | **不是这一轮**，changelog-format-r1 | 存储组织形态改动 |
| 表格行「`.claude/kb/experiments-history.md` \| 全部实验的变更史」→「…一个实验一节，组织形态见 `changelog-format.md`」 | **不是这一轮**，changelog-format-r1 | 同上 |

## 二、diff（原样）

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index c6c80464..4c60a761 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -5,6 +5,8 @@
 
 ## 派发
 
+- **简单任务不派发，自己直接改**，例如修改一下、查询一下、计划一下等，任务单一流程简单的任务。
+- **派发任务要规划，尽量同类任务一起做，最大程度避免串行**。
 - 只由主 agent 点名派发；你手里没有 Agent 工具，不派 subagent，也不从 Bash 里起 `claude` 会话。
 - 轮名、产出文件路径、草稿目录、这一轮的禁读清单都由主 agent 在派发提示里给。定义里写的路径只是形态（`<轮>`），不是实际路径。
 - 输入缺一样就不开工，回复只写缺什么。
@@ -18,8 +20,8 @@
 - 「开工先读：」一行点名的别的文件，只读点名的那几个小节：先 `grep -n '^#' 文件` 找到小节的起止行，再按行读。点的是整份文件的，先看它的节标题，挑与这件活有关的节读。
   不整份读。
 - 每个定义都照守、不再写进各自「开工先读：」一行的三处：跑命令照 `.claude/singlefs-ai-sop/rules/command-safety.md`「退不回去的操作，动手前先想一遍」「`pkill -f` / `killall` 一律禁用」两节；
-  给人看的文字照 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「说人话」一节；说外部状态之前现查（`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 开头一节）。
-- 本机时钟是 UTC，人在东京（JST，UTC+9）；报告里的时刻写清是哪个时区。
+  给人看的文字照 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「文风要简单自然」一节；说外部状态之前现查（`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 开头一节）。
+- 报告与进度记录里的时间只写日期，不写时刻，也不写时区（门禁 12 号判，Write / Edit 由 `.claude/hooks/write-guard.sh` 当场拒）；日期用 `TZ=Asia/Tokyo date +%F` 取，直接 `date` 会差一天。
 - 候选、臂、方案、判据、提问编号有了变体，起一个新名字（那一族里下一个没用过的号，或一个短的描述性名字），不在原名后面加撇号类角标（U+2032、U+2033、U+2034、U+02B9、U+02BA）；全仓由门禁 12 号判，写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」。
 - 派发提示里没给、定义里也没写的项目事实（某份 kb 在哪、某条决策的原文），去仓里现查，不凭印象补。
 - **找不到历史实验的数据、提示或产物，去 `git log` 里看。** 上一轮及更早的实验记录不留在工作区：
@@ -57,7 +59,7 @@
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
 - 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。等自己起的后台任务一律结束本轮等完成通知，不写轮询它输出文件的循环（run_in_background 里也拒，见「执行前拒绝的写法」②）。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、run_in_background 里轮询本会话后台任务输出文件的循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里别的等待循环只记检出，交给主 agent 判断。跑超过 30 分钟的量，每完成一格往草稿目录的 `progress.md` 追加一行（格名、耗时、下一格预计多久），看门狗的整点询问会附上它的末几行。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令与输出写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、run_in_background 里轮询本会话后台任务输出文件的循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里别的等待循环只记检出，交给主 agent 判断。跑超过 30 分钟的量，每完成一格往草稿目录的 `progress.md` 追加一行（格名、耗时、下一格预计多久），看门狗的整点询问会附上它的末几行。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
 - 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
   - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）：② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
     ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
@@ -80,11 +82,12 @@
 
 ## 门禁
 
-- 哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/gate.d/stage-owners.tsv`（第二列是 agent 名，逗号分隔）。列出登记给你的：
+- 扫仓的门禁阶段（本地与共享）只在提交前由 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判。交回前不跑门禁阶段；改了门禁阶段或脚本的，只跑改过的那几份自己的自证与样本（`--selftest`、`.claude/gate.d/fixtures/<阶段文件名>/`）。定义写了开工前的前提检查（`--check <格名>` 只跑一格）的照定义跑；`crash-verifier` 在提交时跑登记给它的那几道，照它的定义。
+- 阶段归属表 `.claude/gate.d/stage-owners.tsv` 第二列（agent 名，逗号分隔）是提交前整轮门禁判这一道红时先派给谁修。列出登记给你的：
   `awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv`
-  逐个 `nice -n 19 bash .claude/gate.d/<文件>`，贴每个的原样末行与退出码。其中重型的那几道（54、55、57、59、87）照「不做」一节重型测试那一条跑：只在提交时或用户要求时、命令带前缀；定义另有写法的照定义（`gate-triage` 登记的阶段都在整轮门禁里跑过，不单跑）。
+- 跑了门禁阶段的，贴每个的原样末行与退出码；重型的那几道（54、55、57、59、87）照「不做」一节重型测试那一条跑：只在提交时或用户要求时、命令带前缀。
 - 退出码 77 是「本次未跑」，不是通过。贴绿行时连它报的「查了多少项」一起贴；报「查了 0 项」或根本没报数的，按没判写。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。
-- 提交前的整轮门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑。
+- 提交前的整轮门禁归 `gate-triage`，不归你。
 
 ## 报告
 
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index f92111c4..9969d256 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -27,7 +27,7 @@ omitClaudeMd: true
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 54 号，等它结束。55、57、59 在主工作区跑，判的是工作区那一份：每道开跑前先 `python3 research/scripts/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（55、59 号查 `crates`，55 号按名字读的 `research/results/` 产物这里不查，由整轮门禁里的 87 号兜；57 号查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层；要没有输出，别处的未跟踪文件不挡；57 号读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
    1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
    1c. 54 号 `--full` 开跑前在那棵 worktree 里跑 `bash research/scripts/layer0-shard-configuration-check.sh --emit-assignments <worktree>` 与 `awk -F'\t' '$1 ~ /^crash-case:/ && $3 ~ /shard=across-machines/ { print $1 }' <worktree>/.claude/gate.d/stage-inputs.tsv`，两样原样抄进报告：前一条退 0 是双机分片开着，抄它打的 `PEER_SSH_HOST=` 与 `PEER_MEMORY_CAP=` 两行（第二台那一片的内存上限：驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 用它起那一片，本机那一片与 merge 由第 1b 步的包装管），退 1 是关着、照单机跑，抄它那一句原因；后一条列出的是开着时两台各跑一片的用例。驱动只由 54 号 `--full` 调（`--merged-log`），不单独跑它。
-2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段：55、57 号与 54 号并行起（各自后台、各自内存上限，照第 1b 步），59 号等 54 号跑完再起（整表复跑与层 0 全量争 CPU），其余轻阶段一次一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认跑快档（`bash .claude/gate.d/54-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 checker 档包 `singlefs-checker-tier` 不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「54 号带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑全量（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段：55、57 号与 54 号并行起（各自后台、各自内存上限，照第 1b 步），59 号等 54 号跑完再起（整表复跑与层 0 全量争 CPU），其余轻阶段一次一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认跑快档（`bash .claude/gate.d/54-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 checker 档包 `singlefs-checker-tier` 不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「54 号带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑全量（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记耗时与退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（新池新建文件的干跑） 产物或闭式比对的那几行；54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。双机分片跑的那几条另抄驱动打的 ①（两台工具链相同）、③（两台输入指纹相同）、④（两片开跑那一行，写着第二台那一片经 `run-with-memory-cap.sh <上限>` 起）、⑤（账本拷回）、⑥（三份发现日志）各一行；判绿要这几行都在、54 号判 merge 那一趟的日志绿；第二台那一片退 250–254 是内存包装自己的结局，照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条办，那一条用例这一趟的判定不算。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index af318edc..8081f999 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -27,18 +27,17 @@ required-inputs: 草稿目录, 报告
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
    1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（例 `research/target/release/e<号>-<简称>`）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；上限先取跑前登记给的，没给再照那一条取；包装退出码 250–254 的那一次输出不算产物。
-2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`；`research/e7-index-bench` 的 `[[bin]]` 的 `name` 全写连字符 `e<号>-<英文名里的下划线换成连字符>`，入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-checker-tier/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有四条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判。④ **入库装置先在草稿目录的副本里改，编过再整份换进主工作区**：主工作区那份先 `cp` 进草稿目录（新建的装置直接建在草稿目录里），只在草稿目录那一份上改；改完跑 `bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py <草稿副本> crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进主工作区，编不过一个字节不写，照它报的错误改草稿副本再跑；主工作区里任何时候都只放编得过的版本，不用 Edit、Write 或 Bash 直接写主工作区 `crates/` 下的 `.rs`。交回之前跑 `python3 research/scripts/compile-then-swap.py --clean --scratch <草稿目录>` 删掉它建的仓副本与编译目录）（`research/e7-index-bench` 的 `name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
+2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`；`research/e7-index-bench` 的 `[[bin]]` 的 `name` 全写连字符 `e<号>-<英文名里的下划线换成连字符>`，入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-checker-tier/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有四条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 11 号 crates-adversarial-review 那一格的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一格会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判。④ **入库装置先在草稿目录的副本里改，编过再整份换进主工作区**：主工作区那份先 `cp` 进草稿目录（新建的装置直接建在草稿目录里），只在草稿目录那一份上改；改完跑 `bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py <草稿副本> crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进主工作区，编不过一个字节不写，照它报的错误改草稿副本再跑；主工作区里任何时候都只放编得过的版本，不用 Edit、Write 或 Bash 直接写主工作区 `crates/` 下的 `.rs`。交回之前跑 `python3 research/scripts/compile-then-swap.py --clean --scratch <草稿目录>` 删掉它建的仓副本与编译目录）（`research/e7-index-bench` 的 `name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 入库装置那一支不写 research 的变异表、不跑 `mutate.sh`（它只编得到 `research/` 下的 bin，整表跑 `crates/mutations.tsv` 又是重型）：变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加，报告里列出追加的变异名、三个数写「待提交时 59 号」。`research/` 那一支写变异表 `research/mutations/e<号>_<英文名>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
    3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
 4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名，拼法 `<旧名去掉 .out>-<YYYY-MM-DD>.out`（YYYY-MM-DD 是今天；旧名去掉 .out 之后末尾已经是 `-<日期>` 或 `-<日期>-rN` 的，先去掉那一截再接今天的日期，不叠两个日期；新实验没有旧名，按 `e<号>-<简称>.out` 算），同一天再跑加 `-rN`（`<旧名去掉 .out>-<YYYY-MM-DD>-r2.out`），不写 `.rN.out`；写之前 `ls` 确认没有同名的，跑完按这个新文件名认本轮才有的完成标记。这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下（入库装置那一支编在仓根的 `target/` 下），用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
    4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
    4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（输出不截断），在报告里逐个点名其中表示「没过」的字段：取值是布尔 `false` 的；取值是 `not_run` 的；取值是大于 0 的整数、而名字表示违例、不匹配、歧义、失败的计数（例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）。取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。每个点名写：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类的整数计数都是 0」。
 5. 在 `research/scripts/replay.sh` 里登记复跑（`research/` 那一支写一行登记；入库装置照 E156（alloc-basis 四条岔路的代价数）、E158（择根与修复四岔路） 的先例另写一个 `driver_e<号>` 驱动函数，登记行写 `E<号>|@driver_e<号>||<产物>|exact`；函数名先 `grep -c 'driver_e<号>' research/scripts/replay.sh` 现查没用过，同一实验的第二份产物照 E158（择根与修复四岔路） 的先例加后缀，重名的函数 bash 会静默盖掉前一个），跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。登记行改指第 4 步的新文件的同时，这份新文件要被实验页点名（逐字节一致也点一行；门禁 40 号按文件名查）；这一次不写实验页的，把「<新文件名> 还没被实验页点名」写进报告，交主 agent 派人点名。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
-6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）：它们放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
-7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。写完（连同第 7b 步）跑第 6 步留下的那几道，各贴原样末行与退出码。
+6. 门禁阶段交回前不跑，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判（共用约束 `.claude/agent-common.md`「门禁」一节）；你只跑第 5 步自己那一个实验的 `replay.sh`。整轮门禁判红派回给你修时，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。
+7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
    7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增；清单里一条都没有时 ② 不适用，① ③ 照做）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
-   7c. 写完实验页与 `experiments-history.md` 的条目，对这两份跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`，贴末行；红在这一次写的句子上的改到绿，红在别处的照写不修。
-8. 跑超过 30 分钟的量：每完成一格往草稿目录的 `progress.md` 追加一行（格名、这一格耗时、下一格预计多久，时刻写 JST），主 agent 读文件不必发消息；看门狗在它超过 30 分钟没改、而你在等后台任务时提前报。
+8. 跑超过 30 分钟的量：每完成一格往草稿目录的 `progress.md` 追加一行（格名、这一格耗时、下一格预计多久），主 agent 读文件不必发消息；看门狗在它超过 30 分钟没改、而你在等后台任务时提前报。
 
 ## 写范围
 
@@ -51,4 +50,4 @@ required-inputs: 草稿目录, 报告
 
 ## 没做什么（固定会有的）
 
-- 没判这个实验的结论能不能推翻或确立决策（那是推论，要走三方）；没跑门禁全量；没提交。
+- 没判这个实验的结论能不能推翻或确立决策（那是推论，要走三方）；没跑门禁阶段；没提交。
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 2cb10d19..a80ab047 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -13,7 +13,7 @@ required-inputs: 草稿目录, 报告, 条款, 要动的 crates 文件
 开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
 
 派发提示写「交补丁」的在草稿目录的副本里改、交补丁目录（「产出」一节），不碰主工作区；没写的在主工作区改。主 agent 同时派几个实现员时一律交补丁。你做的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」的第 1 步；第 2 步三方对抗与第 3 步 checker 由主 agent 另派。
-开工先读：`.claude/singlefs-ai-sop/rules/show-me-test.md`「新增的测试必须先证明它会红」；`.claude/singlefs-ai-sop/rules/code-discipline.md` 全篇；`.claude/rules/implementation-first.md`「规矩」；`.claude/rules/fs-design.md`；`.claude/rules/verification.md`「定义与名字」「harness 档里再分轻用例与耗时用例」「崩溃枚举用例住哪、怎么登记」（测试文件按测什么起名、一条用例一个场景、checker 档测试文件第一行声明模块、耗时用例由 `research/scripts/harness-test-timing.py` 按耗时表标，不手标）。
+开工先读：`.claude/singlefs-ai-sop/rules/show-me-test.md`「新增的测试必须先证明它会红」；`.claude/singlefs-ai-sop/rules/code-discipline.md` 全篇；`.claude/rules/implementation-first.md`「规矩」；`.claude/rules/fs-design.md`；`.claude/rules/verification.md`「定义与名字」「harness 档里再分轻用例与耗时用例」「崩溃枚举用例住哪、怎么登记」（测试文件按测什么起名、一条用例一个场景、checker 档测试文件第一行声明模块）。
 
 ## 输入（主 agent 必须给）
 
diff --git a/.claude/agents/investigator.md b/.claude/agents/investigator.md
index a7c39725..703cfb44 100644
--- a/.claude/agents/investigator.md
+++ b/.claude/agents/investigator.md
@@ -17,7 +17,7 @@ required-inputs: 现象, 草稿目录, 报告
 
 ## 输入（主 agent 必须给）
 
-- 现象原文：产物整行、门禁的 ✗ 与 → 行、种子、跑出现象的命令与时刻（写清时区）。
+- 现象原文：产物整行、门禁的 ✗ 与 → 行、种子、跑出现象的命令。
 - 要回答的问题：是真是假、机理、最小复现，哪几样。
 - 只读，还是可以在草稿目录的仓副本里改（改坏、加打印、二分）。
 - 线程上限、内存上限；报告路径与草稿目录，都在 `/tmp/claude-1000/` 下。
diff --git a/.claude/agents/kb-scribe.md b/.claude/agents/kb-scribe.md
index 1f14503b..4c8ee9a4 100644
--- a/.claude/agents/kb-scribe.md
+++ b/.claude/agents/kb-scribe.md
@@ -1,6 +1,6 @@
 ---
 name: kb-scribe
-description: 书记员：定案之后照主 agent 给的逐条规格写回 kb（决策正文、变更史、分项状态、欠账表）并跑 kb 门禁阶段。只在主 agent 点名派发、并给出逐条改动规格时用；不要自动派发。
+description: 书记员：定案之后照主 agent 给的逐条规格写回 kb（决策正文、变更史、分项状态、欠账表）。只在主 agent 点名派发、并给出逐条改动规格时用；不要自动派发。
 tools: Read, Edit, Bash
 model: sonnet
 effort: high
@@ -13,12 +13,12 @@ required-inputs: 规格, 草稿目录, 报告
 开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
 
 照规格写，不补内容、不做判断。规格里没写的句子一个字不加；规格与原文对不上（旧串不唯一、找不到）就停下报告；规格点名的文件不在你的写范围里（例如 `CLAUDE.md`），也停下报告，不按规格硬写。
-开工先读：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；`.claude/rules/format-evolution.md`「硬约束」与「决策正文只写现状，依据写成指针；决策与实验双向登记」两节；`.claude/singlefs-ai-sop/rules/writing-discipline.md`。
+开工先读：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；`.claude/rules/format-evolution.md`「硬约束」与「决策正文只写现状，依据写成指针；决策与实验双向登记」两节；`.claude/rules/changelog-format.md`；`.claude/singlefs-ai-sop/rules/writing-discipline.md`。
 
 ## 输入（主 agent 必须给）
 
 - 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
-- 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
+- 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照 `decisions-history.md` 对应决策节里那个日期块现取，其余逐字照给）。
 - 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改新池新建文件的字节、动不动格式，决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），N 写阿拉伯数字或一到十的汉字数字，20、75 号都认）；`.claude/kb/decisions.md` 状态列由第 3 步的 `21-decision-items-sync.sh --write` 重生成，规格只给预期的分项计数供回读核对，不手写。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
 - 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
 - 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按 `.claude/rules/format-evolution.md` 那一节的形态逐块给全（四块、索引行、未定项的字节判决），形态本身照那一节，这份定义不抄第二份；缺哪一块就停下报告，不自己补。
@@ -27,12 +27,12 @@ required-inputs: 规格, 草稿目录, 报告
 
 ## 做什么
 
-1. 开工时先记下规格点名的文件、当月变更史文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | AGENT_HOOK_DETECTIONS=<草稿目录>/precheck.jsonl bash .claude/hooks/write-guard.sh`；检出记录指到草稿，预检被拒不会叫醒主 agent 的看门狗），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
-2. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb/decisions-history.md` 的文件头，不另定。然后先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`：它列的「快查·… 还没写」里有不是这一轮的条目，就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；都是这一轮的才跑 `--write`。派发提示写「并行写回」（主 agent 同时派了几个书记员）的，21、49 号的 `--write` 都不跑，由主 agent 在全部交回后跑一次；变更史条目的「（其N）」照派发提示给的写。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：每条决策的卡片只留最近 3 次，被挤掉的旧行是按设计；新增的行里有这一轮之外的条目就停下交回，不自己收拾。
-3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；`--dry-run` 列出要改写 `crates/**/*.rs` 的（实现的地盘，要走代码轮），停在实跑之前，把那些文件与引用逐个列给主 agent（并进实现批），不实跑；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
-   3b. 规格里决策那几处与实验页那几行是一批，一起写、一起回读，不分两次。写完跑 75 号：它报「不对称」时看点名的那一对在不在这一份规格里——在，就是规格少给了一边，停下报告并写明缺哪一行；不在，按「不是这一轮的不修」照写。**判一个实验撑不撑一条分项不在你的活里**，规格没写的关系一个字不加。
-   每次写入之后，写入后钩子 `.claude/hooks/kb-scribe-followup.sh` 按 `.claude/hooks/kb-scribe-followups.tsv` 跑几道阶段、把红的 ✗ 与 → 交回给你：红的是这个流程下一步会消掉的（21 号在 `21-decision-items-sync.sh --write` 之前：翻状态的规格在第 3 步跑它，新立分项、改索引行这类不翻状态的规格不走第 3 步，写完正文自己跑一次 `bash .claude/gate.d/21-decision-items-sync.sh --write` 再往下；30 号在变更史条目写之前、49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走；其余的留到第 4 步一起判归属。
-4. 跑阶段归属表登记给你的阶段各一次并贴结果（共用约束 `.claude/agent-common.md`「门禁」一节）；阶段数用那条 `awk` 接 `| wc -l` 数出来贴原样，不手数。红的判它是不是这一轮写出来的（看点名的文件与行在不在这一轮改动里）；不是这一轮的不修，照写。30 号报的改动行数含别的会话没提交的改动，照抄，不据此判归属。另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。
+1. 开工时先记下规格点名的文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | AGENT_HOOK_DETECTIONS=<草稿目录>/precheck.jsonl bash .claude/hooks/write-guard.sh`；检出记录指到草稿，预检被拒不会叫醒主 agent 的看门狗），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
+2. 决策变更史：原文写进 `.claude/kb/decisions-history.md` 对应决策的 `## D<n>（简称）` 节里，按 `.claude/rules/changelog-format.md` 组织（按日期分组、日期只出现一次；新日期块插在那一节现状句正下面，同一天有几条改动就列几个 `#### ` 子标题，摘要撞了或原本没摘要才加「（其N）」，取号照那一节该日期块里已用到的最大号取下一个）；标题下两行快查照规格。一条改动点名了几条决策，就在几节里各写一份、内容相同。写完跑 `bash .claude/gate.d/30-decision-history-entries.sh --check shape`：红了看点名的节是不是这一轮改的，是就按它的 howto 改，不是就停下交回。规格改了某条决策的现状（已定/未定项数、一句话现状）时，同一次一并跑 `bash .claude/gate.d/30-decision-history-entries.sh --write` 刷新那一节顶上的「**现状**：」行（这一步只改那一行，不碰历史条目；派发提示写「并行写回」的，这个 `--write` 由主 agent 在全部交回后跑一次）。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：新增的行里有这一轮之外的条目就停下交回，不自己收拾。
+3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，把改到的实验源码逐个列给主 agent；`--dry-run` 列出要改写 `crates/**/*.rs` 的（实现的地盘，要走代码轮），停在实跑之前，把那些文件与引用逐个列给主 agent（并进实现批），不实跑；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
+   3b. 规格里决策那几处与实验页那几行是一批，一起写、一起回读，不分两次。**判一个实验撑不撑一条分项不在你的活里**，规格没写的关系一个字不加。
+   新立分项、改索引行这类不翻状态的规格不走第 3 步，写完正文自己跑一次 `bash .claude/gate.d/21-decision-items-sync.sh --write` 再往下。
+4. 门禁阶段交回前不跑，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判（共用约束 `.claude/agent-common.md`「门禁」一节）；第 2、3 步里的 `--check shape` 与 `--write` 是写回流程的一步，照跑。
 5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。
 
 ## 写范围
@@ -40,11 +40,11 @@ required-inputs: 规格, 草稿目录, 报告
 - `.claude/kb/**`。
 - `relabel-item.py` 自己会改写的引用所在文件：`records/**/*.md`、`research/**/*.md`（`research/prompts/` 除外）、`research/**/*.rs`、`.claude/rules/*.md`（`crates/**/*.rs` 不改：预演列出就停下交主 agent）。这几处只许由那个脚本改，你不手改；`.claude/rules/` 的放行只到这里。
 - `/tmp/claude-1000/` 下的报告文件与草稿目录。
-- 用 Edit 写的部分与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致；`relabel-item.py` 与新建当月变更史文件走 Bash，闸管不到，照上面几条自己守。
+- 用 Edit 写的部分与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致；`relabel-item.py` 走 Bash，闸管不到，照上面几条自己守。
 
 ## 产出
 
-- 报告：规格文件路径、`replace-batch.py` 原样输出、`relabel-item.py` 改了哪些文件与「要人看」清单、各门禁阶段结果与归属、改过的全部文件与前后 sha256。
+- 报告：规格文件路径、`replace-batch.py` 原样输出、`relabel-item.py` 改了哪些文件与「要人看」清单、第 2、3 步 `--check shape` 与 `--write` 的原样末行与退出码、改过的全部文件与前后 sha256。
 
 ## 没做什么（固定会有的）
 
diff --git a/.claude/agents/kb-spec-drafter.md b/.claude/agents/kb-spec-drafter.md
index 44f520aa..b7f1c008 100644
--- a/.claude/agents/kb-spec-drafter.md
+++ b/.claude/agents/kb-spec-drafter.md
@@ -19,7 +19,7 @@ required-inputs: 判决, 草稿目录, 条目
 
 - 判决路径与要写回的条目编号，一次不超过 8 条；超过的按 kb 文件不相交切成几份，主 agent 同时派几个起草员，不分几次串着派。
 - 用户定案原话的出处（变更史里引）。
-- 要动的 kb 文件；当月变更史文件（`.claude/kb/decisions-history/<年-月>.md`）。
+- 要动的 kb 文件；`.claude/kb/decisions-history.md`（决策变更史，组织形态见 `.claude/rules/changelog-format.md`）。
 - 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下。
 
 ## 做什么
diff --git a/.claude/agents/prior-art.md b/.claude/agents/prior-art.md
index 1138c080..e6e1f639 100644
--- a/.claude/agents/prior-art.md
+++ b/.claude/agents/prior-art.md
@@ -22,7 +22,7 @@ required-inputs: 草稿目录, 报告
 
 ## 做什么
 
-0. 要查本机源码树的，先跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）：70 号会报哪些承重引用指向的源码树或文献不在本机了，那几棵树上的事实写「本机核不动」。70 号只核 kb 里登记过的那批断言，不替你回答这一次的问题。
+0. 要查本机源码树的，先跑门禁 10 号外部引用那一格：`nice -n 19 bash .claude/gate.d/10-references-and-invariants.sh --check citations`。它会报哪些承重引用指向的源码树或文献不在本机了，那几棵树上的事实写「本机核不动」。这一格只核 kb 里登记过的那批断言，不替你回答这一次的问题。
 1. 本机有源码树的先查本机（贴 `grep -r` 命令、命中计数与文件:行）；没有的查官方文档或源码仓，贴 URL 与取得日期。
 2. 每条事实写：出处、日期、实测还是读文档、口径；标「未在本项目验证」。
 3. 每条写一处它与本工程的已知差异（负载、盘上格式、并发模型、兼容包袱其中之一）；差异写在一组事实背后的做法上，同一个做法的几个数合写一处；写不出就标「差异不明，只能当线索」。
diff --git a/.claude/agents/three-way-materials.md b/.claude/agents/three-way-materials.md
index e0b2bb15..d778ef10 100644
--- a/.claude/agents/three-way-materials.md
+++ b/.claude/agents/three-way-materials.md
@@ -23,13 +23,13 @@ required-inputs: -body.md
 
 ## 做什么
 
-0. 开工先跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）：58 号红、而且点名的就是这一轮的正文，就不开工，回复里原样抄它的 ✗ 与 →；点名的是别的轮次的正文，照写、继续。47 号红时看点名的是哪份脚本：是你要用的 `quote-kb.py`、`kb-sections.py`、`checklist-specs.py`（代码轮还有 `quote-rust-items.py`），停下报告；是别的脚本，照写、继续。
+0. 开工先跑 `bash .claude/gate.d/11-review-and-sync-records.sh --check implementation-premise`（门禁 11 号里判三方论证材料有没有「实现今天的样子」的那一格）与阶段归属表登记给你的其余阶段（共用约束 `.claude/agent-common.md`「门禁」一节）：那一格红、而且点名的就是这一轮的正文，就不开工，回复里原样抄它的 ✗ 与 →；点名的是别的轮次的正文，照写、继续。47 号红时看点名的是哪份脚本：是你要用的 `quote-kb.py`、`kb-sections.py`、`checklist-specs.py`（代码轮还有 `quote-rust-items.py`），停下报告；是别的脚本，照写、继续。
 1. 正文里提到的每个 kb 文件、以及正文每一句「已经如何」按动词全仓 grep 出来的条款所在文件，都用 `python3 research/scripts/kb-sections.py 文件…` 生成清单；清单不许再过滤。
-2. 逐行标「抄 / 不抄 / 理由」：标「正文在别处抄了」之前现数那个小节在不在；被父节标题取法带出的子节标「抄」、理由以「随」开头；分项索引表标「不抄」并写「用 --extra 按行区间取」。标题里带冒号的小节（例如带时刻的标题），`@标题` 取法会在冒号处切错：那一行标「抄」、理由以「随」开头，再用 `--extra 文件:行区间` 取同一段。
+2. 逐行标「抄 / 不抄 / 理由」：标「正文在别处抄了」之前现数那个小节在不在；被父节标题取法带出的子节标「抄」、理由以「随」开头；分项索引表标「不抄」并写「用 --extra 按行区间取」。标题里带冒号的小节，`@标题` 取法会在冒号处切错：那一行标「抄」、理由以「随」开头，再用 `--extra 文件:行区间` 取同一段。
 3. 用 `python3 research/scripts/checklist-specs.py 清单 --cited 正文 --out 附录 [--extra 文件:行区间 …]` 抽附录；退出码非 0 就按它给的下一步改清单再抽，不绕过。
-4. 代码轮先写 `_<轮>-diff.md`（附录二）：文件头写基准与生成时刻，输入给的 diff 原样放进代码块，再附新文件全文，形态照已归档的 `_m2-code-r1-diff.md`（取法见共用约束「找不到历史实验的数据」那一条）；给的是「文件::项名」清单时用 `python3 research/scripts/quote-rust-items.py 文件::项名 …` 整段抽（每段前自带文件名与行区间，回读逐字节比对），不手挑 `awk` 行区间。它不并进背景材料，各腿按正文里写的路径去读。
+4. 代码轮先写 `_<轮>-diff.md`（附录二）：文件头写基准与生成日期，输入给的 diff 原样放进代码块，再附新文件全文，形态照已归档的 `_m2-code-r1-diff.md`（取法见共用约束「找不到历史实验的数据」那一条）；给的是「文件::项名」清单时用 `python3 research/scripts/quote-rust-items.py 文件::项名 …` 整段抽（每段前自带文件名与行区间，回读逐字节比对），不手挑 `awk` 行区间。它不并进背景材料，各腿按正文里写的路径去读。
 5. 拼背景材料：正文 + 清单 + 附录，排他新建。顺序固定，主 agent 派发时写成别的顺序（例如把 diff 并进来）照定义拼、回复里写明。`.tsv`、`.sh` 这类非 markdown 文件过不了 `kb-sections.py`，整份用 `--extra 文件:1-末行` 带进附录，清单里写明。
-   5b. 写开工快照：清单里每个 kb 文件一行进 `research/prompts/<轮>-snapshot/kb-sha256.txt`，代码轮 diff 里的每个 `crates/` 文件一行进同目录的 `crates-sha256.txt`（`sha256sum` 原样输出，路径从仓根起），排他新建；这个目录已经在的不动，回复里写明。门禁 58 号查新轮次有没有这个目录、清单里的 kb 文件在不在里面。
+   5b. 写开工快照：清单里每个 kb 文件一行进 `research/prompts/<轮>-snapshot/kb-sha256.txt`，代码轮 diff 里的每个 `crates/` 文件一行进同目录的 `crates-sha256.txt`（`sha256sum` 原样输出，路径从仓根起），排他新建；这个目录已经在的不动，回复里写明。门禁 11 号的 implementation-premise 格查新轮次有没有这个目录、清单里的 kb 文件在不在里面。
 6. 标「不抄」的每一行与理由留在清单文件里，回复不列；主 agent 要过目就读清单文件。
 
 ## 写范围
diff --git a/.claude/agents/tooling-writer.md b/.claude/agents/tooling-writer.md
index 46eb9697..7e7891b0 100644
--- a/.claude/agents/tooling-writer.md
+++ b/.claude/agents/tooling-writer.md
@@ -30,8 +30,8 @@ required-inputs: 草稿目录, 报告, 出口, 要改的文件
 3. 改已存在的脚本写同目录临时文件再 `mv`，或用 `research/scripts/replace-once.py` 定点改；不在同一个 inode 上就地改。新建文件排他（Write 前先 `ls` 确认不存在）。
 4. 新门禁阶段与新钩子写 `# gate-similar:` 与 `# hook-events:`；新钩子在 `.claude/settings.json` 的 `hooks` 一节注册，自证挂进 `.claude/gate.d/63-agent-write-scope.sh`；新研究脚本写 `admission:` / `run-condition:` 文件头、开头调 `preflight`，自证挂进 `.claude/gate.d/47-research-script-selftests.sh` 的 runner 表。
 5. 自证里只对自己起的进程号发信号（`kill "$!"`、`proc.py stop <pid>`），不按名字、cgroup 或进程组发。
-6. 第二种活：只写判决或规格给的改法，不另加条款；改完在报告里逐份列出改过的定义、共用约束与规则，交主 agent 开定义三方（门禁 72 号）。
-7. 收尾跑并贴末行与退出码：`bash .claude/gate.d/47-research-script-selftests.sh`、62、63、73 号，`bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`，规则纪律项目本地那一道（`RULES_LINT_DIR=.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md" bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .`），`GATE_LINT_DIR=.claude/gate.d` 的 gate-lint 与 `SHELL_LINT_DIR=.claude/gate.d` 的 shell-lint，`python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py`，`python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py`，与阶段归属表登记给你的阶段。54、55、57、59、87 号不真跑，只跑它们的 `--selftest` 或静态分支。红了先看点名的文件在不在这一轮的改动里；不在的不修，照写。
+6. 第二种活：只写判决或规格给的改法，不另加条款；改完在报告里逐份列出改过的定义、共用约束与规则，交主 agent 开定义三方（门禁 11 号的 agent-def-adversarial-review 那一格）。
+7. 收尾只跑这一轮改过的脚本与阶段自己的自证与样本，贴末行与退出码：改过的研究脚本与钩子各跑 `--selftest`；改了钩子或 `.claude/settings.json` 的 `hooks` 一节的，另跑 63 号与 `bash .claude/singlefs-ai-sop/scripts/hooks-registered.sh .`；改过的门禁阶段只跑它们的样本——`.claude/singlefs-ai-sop/scripts/stage-selftest.sh` 整目录跑、不认单道，在仓根下搭一棵只放那几道的阶段目录交给它：`d=<草稿目录>/stage-only; rm -rf "${d:?}"; mkdir -p "$d/.claude/gate.d"; for x in * .[!.]* .claude/*; do case "$x" in .claude|.git|.claude/gate.d) ;; *) ln -s "$PWD/$x" "$d/$x";; esac; done; for x in .claude/gate.d/*; do case "$x" in *.sh) ;; *) ln -s "$PWD/$x" "$d/$x";; esac; done; for s in <改过的阶段文件名…>; do ln -s "$PWD/.claude/gate.d/$s" "$d/.claude/gate.d/$s"; done; nice -n 19 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$d/.claude/gate.d"`；改了定义、共用约束与规则的，另跑规则纪律项目本地那一道（`RULES_LINT_DIR=.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md" bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .`）。整道的 47 号、gate-lint、shell-lint、preflight-lint、gate-overlap、doc-lint 与其余门禁阶段由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判，交回前不跑。54、55、57、59、87 号不真跑，改了它们只跑 `--selftest` 或样本。红了先看点名的文件在不在这一轮的改动里；不在的不修，照写。
 8. 上下文过 600k：停在最近一个自证全绿的点，报告写做完的条、做到一半的（文件与差哪一步）、没开的，交回。
 
 ## 写范围
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index f3193665..5e9e1dc9 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -24,7 +24,7 @@
 7. **本轮出结论之后回到第 2 步记下延后项的那几处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文）与第 8 步的收拢表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
 8. **收拢要定期做**：每批交回后与每轮结束各做一次，把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
 9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，用 SendMessage 发给那个会话（`ListAgents` 列的名字）协商，送不到再弹窗让用户转达。
-10. **改、写、验都按批，不按件。** 什么时候一起改：第 4 条聚出的一簇一次改；主 agent 自己写的 `crates/` 改动算进同一批。什么时候一起写：同时待写回的全部定案（几轮判决、几次用户定案）合成一批，出一份起草规格（条目少、主 agent 自己写得清的可以自己写），条目多的按 kb 文件不相交切成几份、同时派几个起草员，起草交回后再按文件不相交切给几个书记员同时写（派发提示写「并行写回」：各自不跑 21、49 号 `--write`、变更史条目「（其N）」由主 agent 派发时分好，主 agent 在全部交回后跑一次 `--write` 与 kb 门禁）。什么时候一起验证：一批实现员全部交回后（半途交回、失败的先合已完成的部分，补派的归下一批）派一个 `implementation-writer` 做「合入后验证」：打全部补丁、编一次、harness 档的测试二进制逐个 `--test` 跑一遍，红了只重跑红的那几条（`research/scripts/rerun-failed-tests.py` 从日志取命令，不整份重跑）（checker 档包的用例不跑，归提交时 54 号；层 0 快档也归提交时），红的照各份报告的算法改钉值，只对合入时改过锚点或冲突的变异行 `prove-red.sh` 证红（实现员各自证过的不重证，整表归 59 号），交回；子 agent 各自名下的门禁阶段照定义在交回前跑，整批门禁在合入后验证交回之后跑一次；代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮。一件交回就合一件、验一件、开一轮，不许。
+10. **改、写、验都按批，不按件。** 什么时候一起改：第 4 条聚出的一簇一次改；主 agent 自己写的 `crates/` 改动算进同一批。什么时候一起写：同时待写回的全部定案（几轮判决、几次用户定案）合成一批，出一份起草规格（条目少、主 agent 自己写得清的可以自己写），条目多的按 kb 文件不相交切成几份、同时派几个起草员，起草交回后再按文件不相交切给几个书记员同时写（派发提示写「并行写回」：各自不跑 21、30 号 `--write`、变更史条目「（其N）」由主 agent 派发时分好，主 agent 在全部交回后跑一次 `--write`）。什么时候一起验证：一批实现员全部交回后（半途交回、失败的先合已完成的部分，补派的归下一批）派一个 `implementation-writer` 做「合入后验证」：打全部补丁、编一次、harness 档的测试二进制逐个 `--test` 跑一遍，红了只重跑红的那几条（`research/scripts/rerun-failed-tests.py` 从日志取命令，不整份重跑）（checker 档包的用例不跑，归提交时 54 号；层 0 快档也归提交时），红的照各份报告的算法改钉值，只对合入时改过锚点或冲突的变异行 `prove-red.sh` 证红（实现员各自证过的不重证，整表归 59 号），交回；子 agent 交回前、合入后验证都不跑门禁阶段，扫仓的门禁只在提交前由 `gate-triage` 跑的 `research/scripts/gate-staged.sh` 判（`.claude/agent-common.md`「门禁」一节）；代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮。一件交回就合一件、验一件、开一轮，不许。
 
 ## 派出去之后
 
@@ -54,16 +54,16 @@
 
 | 什么时候 | 派谁、按什么次序 |
 |---|---|
-| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条；第一轮派正推，辩方只在有前一轮判决可复核时派）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派，输入里给派腿时主 agent 记下的 `date -u`，代码轮另给开工快照；不派的在判决里写明为什么）→ 主 agent 写判决；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
+| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条；第一轮派正推，辩方只在有前一轮判决可复核时派）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，主 agent 自己核、自己写判决（`.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」：逐条核腿报告里的原文引用、产物行与复跑命令，代码轮对着开工快照核腿引的行；不派核查员）；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
 | 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff）→ 一批交回到齐派「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：派 `crash-verifier` 跑 checker 档：54 号快档、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；层 0 全量只在用户要求或夜间跑，平时不跑） |
 | 一批实现员交回到齐、合入之前 | `implementation-writer` 做「合入后验证」（输入给这一批的补丁目录与各份报告；照「一轮怎么开、怎么收」第 10 条：打全部补丁、编一次、harness 档测试二进制逐个 `--test` 跑一遍、红的照报告改钉值、改完只重跑红的那几条、只证合入时改过的变异行）→ 主 agent 审 diff、合入 → 上面那一行的三方 |
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：`crash-verifier` 跑 checker 档（54 号快档、QEMU、herd7、crates 变异表） |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧并进「一轮怎么开、怎么收」第 4 条的下一批交 `implementation-writer`（输入给分诊报告），`research/` 那侧主 agent 先写问题单 → `experiment-designer` 写重跑登记 → `experiment-runner`（续派带「这一段回答的岔路：…」与「上一段岔路表里还差：…」两句，派发闸 ⑪） |
-| 一批阶段任务结束（里程碑一步、一轮判决、一段实验、一批定义或脚本改完，同时结束的几件算一批、一批做一次），这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段约 200 行（参考值：一组超过就整组一段、不拆组），一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
+| 一批阶段任务结束（里程碑一步、一轮判决、一段实验、一批定义或脚本改完，同时结束的几件算一批、一批做一次），这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段约 200 行（参考值：一组超过就整组一段、不拆组），一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 11 号文件头 knowledge-sync 那一格，那一格判形式）→ 下一行 |
 | 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：54 号跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）；`--full` 不默认跑，用户要求或夜间才在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；判绿按输入哈希写全绿标记），55、57、59 同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（55、57 与 54 号并行，59 号与 54 号串行，内存上限照派发提示各给）→ `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（54、55、57、59 在 `gate.sh` 里只核标记与复用判定，它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-spec-drafter` 起草规格（条目少、主 agent 自己写得清的可以自己写）→ 主 agent 判 → `kb-scribe`（按「一轮怎么开、怎么收」第 10 条一批定案一份规格，按 kb 文件不相交切给几个书记员同时写）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `research/mutations/` 的锚点 `experiment-runner` 只修锚点，`crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs` 并进第 4 条下一批交 `implementation-writer`，书记员不改 `crates/` |
-| 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 72 号） |
+| 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 11 号的 agent-def-adversarial-review 那一格） |
 | 测试或门禁结果与预期不符 | 先派 `investigator` 复现、二分到文件与行（不先改方向和结论）→ 改法并进第 4 条下一批 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner`（登记分了几段、段与段不共用要改的装置与产物的：先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行跑，各写各的产物与报告）→ 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
 | 要别家文件系统的事实 | `prior-art` |
diff --git a/.claude/rules/format-evolution.md b/.claude/rules/format-evolution.md
index cf60b415..61d12c3e 100644
--- a/.claude/rules/format-evolution.md
+++ b/.claude/rules/format-evolution.md
@@ -15,17 +15,18 @@
 ## 硬约束
 
 - 改格式**必须同步更新** `.claude/kb/invariants.md` 和 checker。三者不同步的 commit 一律不收。
-- 决策变更**必须记进**决策变更史（原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下写两行快查，再跑 49 号 --write 重新生成 `.claude/kb/decisions-history.md`），含推翻依据；正文改 `.claude/kb/decisions/` 下对应文件。
+- 决策变更**必须记进**决策变更史（原文写进 `.claude/kb/decisions-history.md` 对应决策的节里，
+  组织形态见 `.claude/rules/changelog-format.md`），含推翻依据；正文改 `.claude/kb/decisions/` 下对应文件。
 - 一旦有外部用户，本文作废，改为严格兼容——那时直接改本文，
-  并把推翻依据写进当月的决策变更史。
+  并把推翻依据写进决策变更史。
 
 ## 决策正文只写现状，依据写成指针；决策与实验双向登记
 
 **决策正文长什么样**（`.claude/kb/decisions/NN-简称.md`）：
 
-- 首行 `## D<n> 简称 —— 状态`，状态后面不加括注：分项计数在索引页，日期与来历在变更史。半定 / 待定的决策例外，要在状态后写「（N 项未定）」（例：「—— 半定（三项未定）」），N 与「### 未定项」的条数一致，只许这一种括注；写了括注时 20 号判条数对不对、75 号判形态，没写括注两道都不红，靠写的人守。
+- 首行 `## D<n> 简称 —— 状态`，状态后面不加括注：分项计数在索引页，日期与来历在变更史。半定 / 待定的决策例外，要在状态后写「（N 项未定）」（例：「—— 半定（三项未定）」），N 与「### 未定项」的条数一致，只许这一种括注；写了括注时 20 号判条数对不对、34 号的 decision-links 格判形态，没写括注两道都不红，靠写的人守。
 - 开头一段写射程：管什么、不管什么。
-- `### 已定项` / `### 未定项` 的索引表每行一句话定案，不抄分项正文，不带日期，不超过 100 字；行末照旧带「**状态：已定。**」（20 号要的规范标记，不算字数）。未定项的登记行里还要带两句判定，写在表下那一段里门禁 31 号定位不到：「改新池新建文件的字节：否（YYYY-MM-DD，依据：…）」与「动不动格式：不动（YYYY-MM-DD，依据：…）」。两句都是规范标记、不算字数，那两个日期也不算「带日期」。**两把尺量的不是一个集合，不许拿一句的结论填另一句**：前一句量「这一版写不写出不同字节」，取值是 是 / 否 / 无对象；后一句量「它的答案将来会不会改动任何盘上字节，或改动已有字节的解释口径」（D15（格式冻结政策） 已定项 1），取值逐字是「**动 / 不动 / 界不定（视同动）**；**没写过视同动**」——两处都视同动，`界不定` 不是中立的第三档。
+- `### 已定项` / `### 未定项` 的索引表每行一句话定案，不抄分项正文，不带日期，不超过 100 字；行末照旧带「**状态：已定。**」（20 号要的规范标记，不算字数）。未定项的登记行里还要带两句判定，写在表下那一段里门禁 20 号格「未定项判过改不改新池新建文件的字节」定位不到：「改新池新建文件的字节：否（YYYY-MM-DD，依据：…）」与「动不动格式：不动（YYYY-MM-DD，依据：…）」。两句都是规范标记、不算字数，那两个日期也不算「带日期」。**两把尺量的不是一个集合，不许拿一句的结论填另一句**：前一句量「这一版写不写出不同字节」，取值是 是 / 否 / 无对象；后一句量「它的答案将来会不会改动任何盘上字节，或改动已有字节的解释口径」（D15（格式冻结政策） 已定项 1），取值逐字是「**动 / 不动 / 界不定（视同动）**；**没写过视同动**」——两处都视同动，`界不定` 不是中立的第三档。
 - 每个分项一节 `#### 已定项 N：名字`，标题不带日期与来历，正文四块，各以粗体标签开头：
   - `**定案**：` 现行规则，不带日期；
   - `**射程**：` 管到哪、不管什么、已知边角，各一两句；论证、推导、「那个实验量不准」这类说明不进射程——进实验页的「它答不了的」或三方判决；
@@ -36,7 +37,7 @@
 
 **实验页的「影响的决策」**：每个实验页有一节 `### 影响的决策`，一张表 `| 决策分项 | 关系 | 回看 |`：
 
-- 决策分项写 `D<n>（简称） 已定项 k` 或 `未定项 k`，没有分项的决策、或只作背景提到的，写 `D<n>（简称）`；实验正文（历史版本与这一节之外）提到的每条决策都要有一行，**一条决策撑着几条分项就写几行**——门禁 75 号按分项查依据（`decisions[decision]["basis"].get(item)`），一条决策收成一行会让它判不出哪条分项有那个实验。
+- 决策分项写 `D<n>（简称） 已定项 k` 或 `未定项 k`，没有分项的决策、或只作背景提到的，写 `D<n>（简称）`；实验正文（历史版本与这一节之外）提到的每条决策都要有一行，**一条决策撑着几条分项就写几行**——门禁 34 号的 decision-links 格按分项查依据（`decisions[decision]["basis"].get(item)`），一条决策收成一行会让它判不出哪条分项有那个实验。
 - **实验必须对应决策**：表里至少一行关系是支撑、推翻或备料；结论作废或退役的实验在标题状态里写明，不判这一条。
 - 关系是 支撑 / 推翻 / 备料 / 不影响 之一。支撑、推翻要写到分项，那条分项的 `**依据**：` 要引回这个实验；反过来，分项依据里引的每个实验，实验页里都要有这一行、关系是支撑或推翻。
 - 回看写 `YYYY-MM-DD 改了` 或 `YYYY-MM-DD 不受影响：理由`。实验出了新结论（页内历史节或 `experiments-history.md` 记了新条目、改了正文、换了产物）之后，每一行都要重新回看，日期不早于那次变动；写「改了」的，那条决策文件要在同一次改动里。
@@ -50,7 +51,7 @@
 - 跨路径的比对断言本身也要钉在源码里（`assert_eq!` 之类把两条路径的输出直接连起来比），只是各自算得出、没有一处断言连起来的，在表下写明还欠这一条。
 - 结论写成全称的（「对所有 X，A 与 B 相同 / 不同」），那一行的怎么算一栏要写明 X 的取值是**扫遍定义域**还是**几个枚举点**；写枚举点的要给理由。变异测试拦不住量词写错，它只证明「已写下的断言所覆盖的函数会红」，欠账在 C49（全称断言只在抽样点验过）。
 
-**门禁管哪一半**：`.claude/gate.d/75-decision-experiment-links.sh` 判表的形状、双向对不对得上、回看过没过期、这次改动该回看的回看了没有。还没回填的实验页与决策记在 `.claude/decision-links-pending`，只减不增。「路径与结论登记」一节由 `.claude/gate.d/99-multipath-registry.sh` 判形状、源码落点存在、共用项不空，还没补的实验页登记在 `.claude/gate.d/multipath-registry-lag.tsv`，只缩不涨。**它们管不到的**：关系判得对不对、回看的理由站不站得住、瘦身时丢没丢内容——这几样靠逐分项对照与抽查。
+**门禁管哪一半**：`.claude/gate.d/34-doc-experiment-pages-and-products.sh` 的 decision-links 格判表的形状、双向对不对得上、回看过没过期、这次改动该回看的回看了没有。还没回填的实验页与决策记在 `.claude/decision-links-pending`，只减不增。「路径与结论登记」一节由同一道的 multipath-registry 格判形状、源码落点存在、共用项不空，还没补的实验页登记在 `.claude/gate.d/multipath-registry-lag.tsv`，只缩不涨。**它们管不到的**：关系判得对不对、回看的理由站不站得住、瘦身时丢没丢内容——这几样靠逐分项对照与抽查。
 
 ## 让格式永久化的那个动作，触发点必须与拦它的闸在同一条时间线上
 
@@ -60,7 +61,7 @@
 
 ## 改一个格式常量：旧值的派生形态要逐类搜，改完登记进 `stale=`
 
-`.claude/gate.d/27-format-constants.sh` 只核 `const 名字 = 值` 那一行。同一个值在源码与 kb 里还有一串派生形态，常量改了它们不会跟着红：
+`.claude/gate.d/27-code-constants-enums-bits-match-kb.sh` 的 format-constants 那一格只核 `const 名字 = 值` 那一行。同一个值在源码与 kb 里还有一串派生形态，常量改了它们不会跟着红：
 消息串（`expect("148")`）、按预留宽写的读写（`skip(24)`、`len() - 24`）、下标（`[147]`，即值 − 1）、倍数（`7 × 148 = 1036`）、
 商与平方（`⌊16253 / 148⌋ = 109`、`109²`）、测试里钉住的产物值（随字节变的校验和），以及门禁脚本里抄过去的数。
 
diff --git a/.claude/rules/implementation-first.md b/.claude/rules/implementation-first.md
index cf754006..1b99cc16 100644
--- a/.claude/rules/implementation-first.md
+++ b/.claude/rules/implementation-first.md
@@ -16,5 +16,5 @@
 
 ## 门禁管哪一半
 
-`.claude/gate.d/58-implementation-premise.sh` 查第 3 条的形式：标题日期在 `2026-09-17` 及以后的三方论证正文（`research/prompts/_*-body.md`）必须出现 `crates/`。
+`.claude/gate.d/11-review-and-sync-records.sh` 的 implementation-premise 格（单跑：`--check implementation-premise`）查第 3 条的形式：标题日期在 `2026-09-17` 及以后的三方论证正文（`research/prompts/_*-body.md`）必须出现 `crates/`。
 **它管不到的**：读没读对、改法是不是真按实现写的、实验跑前登记里的模型该不该复用实现——这几样靠人与三方论证的攻方腿。
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 1efb7b93..d7945c62 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -8,7 +8,7 @@
 | 步 | 做什么 | 谁在判 |
 |---|---|---|
 | 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
-| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」、本地腿（`three-way-local-attack` 或 `-defense`，主 agent 按 `.claude/rules/three-way-inference.md`「各条腿必须互不重复」定）按分到的那一面找反例或辩护；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
+| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」、本地腿（`three-way-local-attack` 或 `-defense`，主 agent 按 `.claude/rules/three-way-inference.md`「各条腿必须互不重复」定）按分到的那一面找反例或辩护；打中的写回代码，再攻一轮 | 门禁 11 号的 crates-adversarial-review 格判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
 | 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（checker 档快档，全量按 `.claude/rules/verification.md`）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |
 
 **次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。
@@ -17,15 +17,15 @@
 
 **定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。
 
-**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。
+**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 11 号的 agent-def-adversarial-review 格判形式（形态照同一道的 crates-adversarial-review 格）。
 
 **三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。
 
 ## 代码轮派腿之前记一份开工快照
 
-派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），连同派腿的时刻（`date -u`）交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。
+派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），主 agent 写判决时拿它核腿引的行；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。
 
-**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、派核查员之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再交核查员——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、几点、被改了几处、腿引的行落没落在那几处。
+**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、写判决之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再核——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、被改了几处、腿引的行落没落在那几处。
 
 ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判
 
@@ -72,6 +72,6 @@ herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样
 - **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
 - **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
 - **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
-- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。54 号 `--full` 在本地配置（仓根 `layer0-shard.env`，模板 `layer0-shard.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 54 号的那一层内存包装管。驱动只由 54 号 `--full` 调。
+- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。54 号 `--full` 在本地配置（仓根 `multi-host.env`，模板 `multi-host.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 54 号的那一层内存包装管。驱动只由 54 号 `--full` 调。门禁 59 号照同一份配置按行分：配置判得过、`GATE_MUTATION_ACROSS_MACHINES` 不是 0、没设 `SINGLEFS_GATE_FULL=1` 与 `GATE_MUTATION_START_OVER=1` 时，先交 `research/scripts/mutation-shard-run.sh`：本机没有作数按条记录的行按用时表两堆均分，本机与第二台各跑一趟只判自己那一堆的 59 号，第二台那一份整份经 `research/scripts/run-with-memory-cap.sh` 起、每条变异在里面照 59 号各自经包装，第二台的按条记录拷回本机，经 `research/scripts/crates-mutation-rows.py import` 核过底座指纹与行键再导入；驱动退出之后 59 号在本机整张判一遍（命中记录的复用、缺的现跑），判定输出与单机逐字相同，全绿标记由本机写。这个驱动只由 59 号调。
 
 **门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁与提交时跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`），再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红；全量只在用户要求或夜间，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
diff --git a/.claude/rules/mutation-sampling.md b/.claude/rules/mutation-sampling.md
index d925e5ad..3dd7ef28 100644
--- a/.claude/rules/mutation-sampling.md
+++ b/.claude/rules/mutation-sampling.md
@@ -62,7 +62,7 @@
 ⇒ **判据**：读 `mutate.sh` 的输出之前，先确认它**跑完了整张表**——
 收尾没有「已还原，基线仍全绿」就是中途退出，那一轮的「全抓」一个字都不算数；有这一行而退出码是 5（跑的时候源码被改过），同样不算数，改完重跑。
 门禁形态：不跑变异，只对每张表逐条做子串计数，命中不是 1 次就判红并列出表名与条目名
-（`.claude/gate.d/33-mutation-tables.sh`，C327（变异表的锚点腐化没有会红的检查） 已还清）。
+（`.claude/gate.d/33-code-experiment-and-mutation-source-discipline.sh` 的 mutation-tables 那一格，C327（变异表的锚点腐化没有会红的检查） 已还清）。
 
 ## 第八类：变异跑了但无效——替换编不过，那条行为零覆盖
 
diff --git a/.claude/rules/path-moves.md b/.claude/rules/path-moves.md
index 8b710169..dfdccb28 100644
--- a/.claude/rules/path-moves.md
+++ b/.claude/rules/path-moves.md
@@ -23,7 +23,7 @@
 
 ## 改一个全仓术语：正文之外还有五处会红
 
-路径搬迁之外的另一类是**术语改名**（把一个概念的中文名全仓换掉）。它登记在 `.claude/kb/term-renames.md`、由门禁 90 号判全仓不再出现旧名，落地时同样要一次做完：正文改完之后还有五处会红。
+路径搬迁之外的另一类是**术语改名**（把一个概念的中文名全仓换掉）。它登记在 `.claude/kb/term-renames.md`、由门禁 12 号的 term-renames 格判全仓不再出现旧名，落地时同样要一次做完：正文改完之后还有五处会红。
 
 | 跟着要改的 | 哪道门禁会红 | 怎么做 |
 |---|---|---|
@@ -48,12 +48,12 @@
 3. 撞号先查：新名字在那一族的全部文件里零命中。
 4. 变体与原名的关系写在新名字的定义那一句里，不写进名字。
 
-门禁 12 号扫全仓（冻结证据也在内），撇号类字符出现一处就判红；ASCII 单引号当角标它判不了，靠写的人与 review。
+门禁 12 号的 prime-marks 格扫全仓（冻结证据也在内），撇号类字符出现一处就判红；ASCII 单引号当角标它判不了，靠写的人与 review。
 
 ## 门禁管哪一半
 
 **搬迁这一半没有专管它的阶段**：搬迁是低频操作、核验是一次性的，不值得挂一道常驻阶段。「链接指向」阶段与门禁 10 号第 4 段顺带罩住 Markdown 链接与治理文档里的路径，其余落点靠人。
 所以「怎么做」那七步全靠人跑（第 7 步里的自检由各道门禁、钩子与脚本自己的自检跑，改没改全靠人），第 4 步那条「旧路径搜不到了不等于新路径是对的」尤其要自己打开文件看。
 
-**门禁管的是另一半**：术语改名归 `.claude/gate.d/90-term-renames.sh`，它按 [.claude/kb/term-renames.md](../kb/term-renames.md) 的登记表查全仓不再出现旧名——
+**门禁管的是另一半**：术语改名归 `.claude/gate.d/12-doc-forbidden-notations-and-old-terms.sh` 的 term-renames 格，它按 [.claude/kb/term-renames.md](../kb/term-renames.md) 的登记表查全仓不再出现旧名——
 那一类是高频、会静静错在几千处、而且肉眼扫不出来，与搬迁不同。
diff --git a/.claude/rules/three-way-inference.md b/.claude/rules/three-way-inference.md
index 012c5c7c..8d0eafa4 100644
--- a/.claude/rules/three-way-inference.md
+++ b/.claude/rules/three-way-inference.md
@@ -117,6 +117,7 @@
 - **三方一致 → 继续。** 一致不等于对，但可以往下走。
 - **不一致 → 先核实，再讨论。** 核实指去查能分辨它们的那个事实，
   不是再加一个模型来投票。分歧点本身要写进结论里。
+- **本地腿交回的每一份样本都读，判决里逐份按路径列出**：干净样本、通读判带损坏的样本、损坏闸留下的作废副本（`-output-void<n>.md`），各标干净或参考；判决里不许只写「作废」「带损坏」「不稳定不采」而不再提它的内容。参考样本与「不稳定」格怎么用，在「一条腿只抽一次样不算一次观测——否定结论尤其不算」与「给本地腿的提示一律用英文」两节。门禁 11 号的 verdict-names-local-samples 格（`.claude/gate.d/11-review-and-sync-records.sh`，新写的判决有没有按路径点名那一轮本地腿的每一份样本）判点名这一半。
 - **一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引。** 攻方腿常拷一份装置改臂实测（允许，而且值钱），
   但副本上的数不进 kb：主 agent 把它做成入库装置的新一次跑（跑前登记写明是核实哪条腿的哪句话），逐字段坐实了才引。
 - **攻方腿自己提的收严，只在它自己的模型上量过，算「没被攻过」。**
@@ -132,7 +133,13 @@
 
 **第三轮之后停，不开第四轮。** 腿提的收严、主 agent 判决里推的组合都算被攻过零轮；第三轮只攻前两轮站住的形态，这一轮里新冒出来的零轮形态不再为它开一轮，写进判决的交用户表、标「零轮」，要么登记实验量代价，要么另立一题。
 
-**核查员按轮派。** 这一轮有腿交了模型、产物或复跑命令，就派 `three-way-verifier`；这一轮没有任何腿交模型、产物或复跑命令的可以不派，判决里写明没派、为什么。
+**核查与判决由主 agent 做，不派子 agent 去核子 agent。** 腿交齐之后、写判决之前，主 agent 自己逐条核：
+
+1. 先跑 `python3 research/scripts/cite-check.py <全部腿报告> --root <快照根> --background <背景材料>`：它判了的（对不上、指到标题行、写成背景材料的行号）照它的判定写进判决；它列成「没判」的与认不出写法的引文，逐处人工核。
+2. 腿引的每一行产物，在产物文件里逐字找。
+3. 判决要引的复跑数：把腿的模型目录拷到草稿目录，在副本里经 `research/scripts/run-with-memory-cap.sh` 跑，比输出与报告里抄的、比 sha256，不在腿的原目录里跑。
+4. 本地腿的译文核对表逐条核：行号在不在、抄的是不是原文、英文丢没丢或多没多限定词与括注。
+5. 核不动的（要虚机、要网络、内存包装起不来）在判决里写明是哪一条、为什么，不当核过。
 
 实测有效的三轮分工：第一轮过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱；第二轮第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方；第三轮「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。
 
@@ -158,8 +165,8 @@
 | **打中了**（给出反例、指出矛盾） | 一次就值得去核。它是个线索，真伪由主 agent 现查坐实，抽样次数不改变这一步 |
 | **没打中**（「构造不出反例」「没发现问题」） | **一次不算**。它与「这一轮它没想到」分不开，而两者在答复里长得一模一样 |
 
-⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」，
-两次不一致就照 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记「不稳定」，不下结论。
+⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」。
+**几份样本一不一致，按每一格答的是哪件事判，不按行号、行标签、措辞、排版与作答次序判**：先把各份样本按内容对到同一件事上再比；只有一份样本答到的那件事，照只有一份样本办。对齐之后结论不同的格记「不稳定」（`.claude/singlefs-ai-sop/rules/test-discipline.md`「单次观测不算数」）：它不支撑「没打中」，也不算几份样本一致；格里任何一份样本给出的反例、矛盾与算出的数，照「打中了」那一行当线索，由主 agent 现查，判决里逐条写去向（现查坐实而采、现查推翻而不采、核不动），不整格写「不稳定不采」。
 云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的：云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告写成 `<轮>-<模型>-s2-output.md`（例 `-opus-s2-output.md`），模型目录写成 `<轮>-<模型>-s2-model/`，不盖掉第一条腿的。
 
 ## 云端腿的报告要分段落盘
@@ -205,11 +212,9 @@
 
 ⚠️ **本地腿用 Bash 的 `run_in_background` 起、结束本轮等完成通知（单次请求最长 900 秒，超过前台上限），命令里不加 `setsid`、`&`、`disown`。**
 
-**闸判红之后**：那一轮**作废重跑**，不许记成「三方不一致」——
+**闸判红之后**：那一份不算干净样本，照样重跑取下一份，不许记成「三方不一致」——
 否则每次都会不一致，这条规则就退化成了摆设。
-⚠️ **这不是洁癖**：**损坏的不只是词。**
-确有必要采用带损坏的输出时，设 `ASK_LOCAL_ALLOW_CORRUPT=1`，
-并在结论里写明这一票带瑕疵。
+⚠️ **损坏的不只是词**，带损坏的只当参考样本：`ask-local.sh` 留下的作废副本（`-output-void<n>.md`）与通读判「带损坏」的样本都交主 agent 逐份读；损坏处所在的那一句不用，其余部分给出的反例、矛盾与算出的数，照「一条腿只抽一次样不算一次观测——否定结论尤其不算」那一节「打中了」那一行当线索现查，判决里引它时写明出自参考样本；参考样本不算进「没打中」要的两份，也不拿来与干净样本比一致不一致。
 
 ⚠️ **闸报「没跑成」或「没做」时不是通过。** 检测器自己出错（退出码 2）或找不到，与判红（退出码 1）
 是两件事，`ask-local.sh` 分开报：前者退出 6、不打正文、正文留成作废副本，判红退出 5。退出 6 就是**这一项没验**，
diff --git a/.claude/rules/verification.md b/.claude/rules/verification.md
index 699a6ca5..3e9e27f0 100644
--- a/.claude/rules/verification.md
+++ b/.claude/rules/verification.md
@@ -15,16 +15,16 @@
 
 归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。拿不准的放 checker 档，再由代码三方判要不要挪回。
 依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（它的依赖闭包里没有 `singlefs-checker-tier`，dev-dependencies 也算；源码里零处引它；门禁 94 号判）。两档都要用的一小段代码，宁可各留一份（例：`crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` 抄自 checker 档的坏盘输入），也不让 harness 依赖 checker 档。
-测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；不带里程碑、步号、增补号、并行线号、欠账号（以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`），来历写进文件头的文档注释。`research/scripts/crash-case-check.py` 判，门禁 14 号在真仓上跑。
+测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；不带里程碑、步号、增补号、并行线号、欠账号（以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`），来历写进文件头的文档注释。`research/scripts/crash-case-check.py` 的 file-names 那一样判，门禁 13 号的 test-file-names 格在真仓上跑它。
 不许用的叫法：「checker 包」（分不清是池级 checker 还是 checker 档）、「放量用例」「验证档」（说 checker 档）。
 
 ## harness 档里再分轻用例与耗时用例
 
 - 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
 - 红了只重跑红的那几条，不整份重跑，轻用例与耗时用例一样：`python3 research/scripts/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑那几条的命令（一律带 `--include-ignored`，红的是耗时用例也跑得到），经 Bash 起、加内存包装；修完代码也只重跑红的那几条；整份重跑只在上一趟的日志不全时（脚本判红、一条命令都不打），完整的一遍归提交时的整轮门禁。
-- 耗时用例：单线程 debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 耗时用例：…"]`；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。轻重由量出来的数定，不由整份全量日志里 libtest 的「has been running for over 60 seconds」定：`python3 research/scripts/harness-test-timing.py measure`（只量变了的目标加 `--targets`）把单条耗时写进 harness 档包根下的耗时表 test-timing.tsv（第一次量完才有这份文件），再 `apply` 按表统一标上或摘掉，表与标记一起提交；不手标、不手摘。门禁 14 号判标记与表对得上。
-- 调全量崩溃枚举函数（`enumerate_layer0` 一族，快档 `quick_tier` 那几个除外）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`research/scripts/crash-case-check.py` 判这一条，写在别的包里判红。
-- 一条用例一个场景：按参数循环、每一轮新建一个池（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`）的，拆成一个带参数的函数加每个取值一条 `#[test]`，红了只重跑那一条；确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`research/scripts/crash-case-check.py` 判，门禁 14 号在真仓上跑它。
+- 耗时用例：debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 耗时用例：…"]`；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。单条用时看平常跑的时候量到的，不另外单独量。
+- 调全量崩溃枚举函数（`enumerate_layer0` 一族，快档 `quick_tier` 那几个除外）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`research/scripts/crash-case-check.py` 的 placement 那一样判这一条，写在别的包里判红，门禁 54 号开跑前在真仓上跑它。
+- 一条用例一个场景：按参数循环、每一轮新建一个池（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`）的，拆成一个带参数的函数加每个取值一条 `#[test]`，红了只重跑那一条；确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`research/scripts/crash-case-check.py` 的 one-scenario 那一样判，门禁 14 号在真仓上跑它。
 
 ## checker 档自己分快档与全量
 
@@ -37,7 +37,7 @@
 
 ## 崩溃枚举用例住哪、怎么登记
 
-- checker 档每个测试文件第一行写它测哪几个模块：`//! checker 档模块：<模块，按 crash、layer0_progress、crash_injection、bad_disk_input、device_log、on_device_modes 的次序用、隔开>`，与它从 `singlefs_checker_tier::` 导入的模块逐个相同；一个都不导入的写 `无（为什么）`。按模块找用例：`grep -l '^//! checker 档模块：.*crash_injection' crates/singlefs-checker-tier/tests/*.rs`。`research/scripts/crash-case-check.py` 判。
+- checker 档每个测试文件第一行写它测哪几个模块：`//! checker 档模块：<模块，按 crash、layer0_progress、crash_injection、bad_disk_input、device_log、on_device_modes 的次序用、隔开>`，与它从 `singlefs_checker_tier::` 导入的模块逐个相同；一个都不导入的写 `无（为什么）`。按模块找用例：`grep -l '^//! checker 档模块：.*crash_injection' crates/singlefs-checker-tier/tests/*.rs`。`research/scripts/crash-case-check.py` 的 modules 那一样判，门禁 54 号开跑前在真仓上跑它。
 
 - 写在 `crates/singlefs-checker-tier/tests/<流的名字>.rs`，全量那条标 `#[ignore]`，同文件的快档用例不标。
 - 共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness 档的 `tests/common*/mod.rs`，不抄第二份。
@@ -73,12 +73,12 @@
 |---|---|
 | 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数 | 54 号；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
 | 模型对拍跑了、每段都判过 | 74 号；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
-| 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:` | `research/scripts/crash-case-check.py`（47 号跑它的自证） |
-| 函数名与类型名不只由空泛词拼成 | 13 号，词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude` |
+| 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:`；checker 档测试文件声明模块 | 54 号开跑前跑 `research/scripts/crash-case-check.py --only placement,modules`，红了不起 cargo；47 号跑那份脚本的自证 |
+| 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 13 号：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `research/scripts/crash-case-check.py --only file-names` |
 | 池级 checker 库不依赖实现；harness 档不依赖 checker 档 | 94 号 |
-| 用例住对档、一条一个场景、测试文件不按里程碑起名、checker 档测试文件声明模块、harness 耗时用例标记与耗时表对得上 | 14 号（跑 `research/scripts/crash-case-check.py` 与 `research/scripts/harness-test-timing.py check`） |
+| harness 档一条用例一个场景 | 14 号（跑 `research/scripts/crash-case-check.py --only one-scenario`） |
 | 变异行点名的测试跑得到：标了 `#[ignore]` 的带 `--include-ignored`、`--` 之后的筛选词筛得到点名的测试 | 33 号 |
 | 每道阶段认第一个参数当项目根 | 62 号 |
 | 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、55 / 57 / 59 / 87 号、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置算重型 |
 
-**它们管不到的**：耗时表是不是最近量的（表头写着量的那次提交）、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。
+**它们管不到的**：harness 耗时用例标得对不对、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。
diff --git a/CLAUDE.md b/CLAUDE.md
index 7ab31f15..d55d3308 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -19,7 +19,7 @@ agent治理与上游sop治理，是另外一个重要的任务。因此遇到问
 
 别的会话、别的贡献者、派出去的 subagent 可能撞上的坑与约定，写进项目：规则（`.claude/rules/`）、agent 定义与 `.claude/agent-common.md`、kb、`records/`。私有 memory 只放用户个人的偏好（提交时间窗、回复语言这类）：它只在本机，别的贡献者看不到，定义开了 `omitClaudeMd` 的 subagent 也读不到。往本文件加内容之前先问它属于哪个 agent，属于就写进那个定义或共用约束。
 
-subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案.md`，不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身。定义、共用约束、三方流程与配套脚本的问题：修它们本身就是这一轮的出口时，直接改，不先问；是做别的任务时撞上的，照 `.claude/main-agent.md`「禁止」一节，记下或弹窗，不自行扩大范围。改完记进那份计划，跑相关门禁：47、62、63、doc-lint，改了定义与共用约束的走一轮三方或由用户逐份豁免（72 号判），改了触发文件的写阶段同步记录（68 号判）；共享的「本地阶段判别力」「编号与简称」两道单跑 `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d` 与 `bash .claude/singlefs-ai-sop/scripts/number-name-sync.sh .`，不是重型。
+subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案.md`，不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身。定义、共用约束、三方流程与配套脚本的问题：修它们本身就是这一轮的出口时，直接改，不先问；是做别的任务时撞上的，照 `.claude/main-agent.md`「禁止」一节，记下或弹窗，不自行扩大范围。改完记进那份计划，只跑改过的脚本或门禁阶段自己的自证与样本；门禁归提交时，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh` 判，改完不跑。改了定义与共用约束的走一轮三方或由用户逐份豁免（11 号 agent-def-adversarial-review 那一格判），改了触发文件的写阶段同步记录（11 号 knowledge-sync 那一格判）。
 
 ## 规则（始终生效）
 
@@ -59,6 +59,7 @@ subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案
 @.claude/rules/verification.md
 @.claude/rules/implementation-first.md
 @.claude/rules/path-moves.md
+@.claude/rules/changelog-format.md
 
 共享 SOP 只管「项目怎么和 AI 协作」；只有本工程需要的纪律（文件系统怎么设计、压在本机资源上的流程）放 `.claude/rules/`，不往上游推。
 
@@ -70,10 +71,10 @@ subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案
 |---|---|
 | `.claude/kb/decisions.md` | **决策索引**：编号、简称、状态、指向正文的链接 |
 | `.claude/kb/decisions/` | 每个决策一个文件（`NN-简称.md`），正文与论证都在这里 |
-| `.claude/kb/decisions-history.md` | 决策变更史按决策的汇总，由 49 号 `--write` 从原文生成；原文按月在 `.claude/kb/decisions-history/<年-月>.md`，怎么写见 `.claude/rules/format-evolution.md`「硬约束」 |
+| `.claude/kb/decisions-history.md` | 决策变更史，一条决策一节、节内是它的完整历史，组织形态见 `.claude/rules/changelog-format.md` |
 | `.claude/kb/experiments.md` | **实验索引**：编号、简称、状态、指向正文的链接 |
 | `.claude/kb/experiments/` | 每个实验一个文件（`NN-简称.md`），正文与口径都在这里 |
-| `.claude/kb/experiments-history.md` | 全部实验的变更史 |
+| `.claude/kb/experiments-history.md` | 全部实验的变更史，一个实验一节，组织形态见 `.claude/rules/changelog-format.md` |
 | `.claude/kb/invariants.md` | 不变量清单，checker 是它的可执行形式 |
 | `.claude/kb/feature-bits.md` | feature bit 的记账表（位号 / 类别 / 名称 / 引入版本 / 引入 commit / 状态 / 语义一句话），形态由 D15（格式冻结政策） 已定项 10 定。**位号的唯一登记位不在这里**，在 D15（格式冻结政策） 已定项 4 那张表；门禁 93 号判两处逐位一致 |
 | `.claude/kb/term-renames.md` | 全仓术语改名的登记表，一行一条（旧名 / 新名 / 匹配），门禁 90 号按它查全仓不再出现旧名；怎么改名见 `.claude/rules/path-moves.md`「改一个全仓术语」 |
```
