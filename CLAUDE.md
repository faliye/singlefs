# singlefs

**一个从零设计的 COW 文件系统，Rust 实现。**
现有 COW 文件系统是**设计输入**（它们的病历和解法），不是移植目标。

当前里程碑：**「第二个事务」**（`.claude/kb/milestone/02-second-txn.md`，2026-09-16 建档，做到哪一步看那份文件）。上一个里程碑「第一个事务」（`.claude/kb/milestone/01-first-txn.md`）出口 2026-09-14 满足，代码在 `crates/` 下四个 crate（格式常量、核心、验证装置、checker）。

## 什么时候派哪个 agent

主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下表派。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活照旧临时写提示派 general-purpose。改了定义要新派才生效：续做（SendMessage）沿用第一次派发时的定义；新建的定义要等几秒才派得出去。改了本文件（连同它 `@` 的规则）要新开会话才对派出去的 agent 生效：同一会话里派的继承会话开始时那一份。计划、实测与现状在 `records/2026-09-16-subagent拆分提案.md`。

派出去之后要盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本（2026-09-17 用户定）。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `python3 research/scripts/agent-watch.py watch --agents <这次派出的 agent id，逗号分隔>`：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部结束也退出，没有告警时每 60 分钟定时回报一次。叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，由看门狗盯着进程。经过见 `records/2026-09-17-已分配口径三方与两个实验.md` 第六节。

派发提示与发给别的会话的消息，在不影响正确性的前提下写简练（用户 2026-09-17 定）：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，有腿交了模型或产物就派 `three-way-verifier` → 主 agent 写判决；三轮之后停 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
| 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 要别家文件系统的事实 | `prior-art` |

## 经验与约定写在哪

别的会话、别的贡献者、派出去的 subagent 可能撞上的坑与约定，写进项目：规则（`.claude/rules/`）、agent 定义与 `.claude/agent-common.md`、kb、`records/`。私有 memory 只放用户个人的偏好（提交时间窗、回复语言这类）：它只在本机，别的贡献者看不到，定义开了 `omitClaudeMd` 的 subagent 也读不到（2026-09-17 用户定）。往本文件加内容之前先问它属于哪个 agent，属于就写进那个定义或共用约束。

subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案.md`，不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身（2026-09-16 用户定）。定义、共用约束、三方流程与配套脚本用着发现问题就直接改，改完记进那份计划、跑相关门禁（26、47、62、63、89 与 doc-lint），不先问（2026-09-17 用户授权）。

## 规则（始终生效）

@.claude/singlefs-ai-sop/rules/engineering-philosophy.md
@.claude/singlefs-ai-sop/rules/sop-first.md
@.claude/singlefs-ai-sop/rules/show-me-test.md
@.claude/singlefs-ai-sop/rules/machine-first.md
@.claude/singlefs-ai-sop/rules/code-discipline.md
@.claude/singlefs-ai-sop/rules/writing-discipline.md
@.claude/singlefs-ai-sop/rules/design-doc-discipline.md
@.claude/singlefs-ai-sop/rules/kb-discipline.md
@.claude/singlefs-ai-sop/rules/test-discipline.md
@.claude/singlefs-ai-sop/rules/evidence-discipline.md
@.claude/singlefs-ai-sop/rules/verify-before-claiming.md
@.claude/singlefs-ai-sop/rules/pushback-discipline.md
@.claude/singlefs-ai-sop/rules/command-safety.md
@.claude/singlefs-ai-sop/rules/session-wrapup.md

## 规范从哪来

| 项 | 值 |
|---|---|
| 上游仓 | `singlefs-ai-sop`，本机在兄弟目录 `../singlefs-ai-sop-zh`。同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的 |
| 项目里的副本 | `.claude/singlefs-ai-sop/`（[README](.claude/singlefs-ai-sop/README.md)），是**拷贝，不是符号链接**；与上游同步靠重新拷贝一份：`rsync -a --exclude '.git' ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/`，拷完 `diff -rq --exclude=.git ../singlefs-ai-sop-zh .claude/singlefs-ai-sop` 确认一致再刷版本戳（`cp -r` 会把上游的 `.git` 一起拷进副本，第二次同步时那些只读 object 报一屏 Permission denied） |
| 版本戳 | `.singlefs-ai-sop-version`。门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红——那是在提醒「规矩变过了，先读再跑」 |
| 怎么改 | 共享规则只能在**上游**改。从本仓的会话改上游**只往文件里填内容**（规则正文、脚本逻辑、三语译文那几行）：不跑 `bump.sh`、不动 `VERSION`、不改 MANIFEST / SOURCE-MANIFEST 与译文首行的溯源哈希、不写带版本号的 CHANGELOG 条目、不提交，这些归做发版的会话；填之前先看上游三语仓 `git status`，有人在发版就等它提交完，填完告诉发版会话填了哪些文件哪几段（2026-09-12 用户定）。发版之后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳。**不许在 `.claude/singlefs-ai-sop/` 里就地改**——下次同步就没了，而且改它们等于改所有项目。上游的改动应当罕见：经常变说明规范本身没设计好；**作业在本仓，不在上游仓** |

## 项目本地规则

@.claude/rules/fs-design.md
@.claude/rules/format-evolution.md
@.claude/rules/three-way-inference.md
@.claude/rules/mutation-sampling.md
@.claude/rules/implementation-workflow.md
@.claude/rules/implementation-first.md

共享 SOP 只管「项目怎么和 AI 协作」；只有本工程需要的纪律（文件系统怎么设计、压在本机资源上的流程）放 `.claude/rules/`，不往上游推。

## 项目本地事实

| 文件 | 内容 |
|---|---|
| `.claude/kb/decisions.md` | **决策索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/decisions/` | 每个决策一个文件（`NN-简称.md`），正文与论证都在这里 |
| `.claude/kb/decisions-history.md` | 决策变更史按决策的汇总，由 49 号 `--write` 从原文生成；原文按月在 `.claude/kb/decisions-history/<年-月>.md`，怎么写见 `.claude/rules/format-evolution.md`「硬约束」 |
| `.claude/kb/experiments.md` | **实验索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/experiments/` | 每个实验一个文件（`NN-简称.md`），正文与口径都在这里 |
| `.claude/kb/experiments-history.md` | 全部实验的变更史 |
| `.claude/kb/invariants.md` | 不变量清单，checker 是它的可执行形式 |
| `.claude/kb/prior-art.md` | 他家方案调研，含来源与口径 |
| `.claude/kb/pitfalls.md` | 避坑清单，每做设计决定回来对一遍 |
| `.claude/kb/checks-owed.md` | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| `.claude/kb/first-txn-layout.md` | 第一个事务写出哪些字节：每段每字段指向一条决策分项、给宽度与取值，指不到的就是格式级空白 |
| `.claude/kb/vm-harness.md` | 怎么把实验送进虚机在真块设备上跑 |
| `.claude/kb/verification-build.md` | 三样验证手段（checker、事务层、崩溃点重放）怎么落地、被谁挡着 |
| `.claude/kb/milestone/` | 里程碑规划，一个里程碑一个文件（`NN-简称.md`）：每步设想、验收标准、写出的字节、会碰到的决策点；每步开工前回来改 |
| `research/scripts/fetch-refs.sh` | 把承重的外部文献重新固定到本机（URL + sha256 + 引用方），`pdf-text.py` 抽文本，断言在 `verify-citations.sh` |
| `research/scripts/stage-mine.py` | 几个会话共写一批文件时，只把这一轮的块放进暂存区（命中 `--match` 的进，命中 `--foreign` 的拒绝） |
| `research/scripts/check-staged.sh` | 在临时 worktree 上只拿「HEAD + 暂存区」跑 doc-lint 与快的 kb 阶段 |
| `research/scripts/replace-once.py` | 定点替换：旧串在文件里必须恰好命中一次，否则不写；几个会话共写一批文件时只许这样改，不许整份重写 |
| `research/perf-by-milestone.md` | singlefs 与六家文件系统按里程碑的性能对比；数来自 E152（按里程碑对比六家文件系统的文件性能），表由 `research/scripts/e152-tables.py` 生成 |
| `records/` | 建设过程 |
| `briefs/` | 每次更新的简报，按日期一份（`YYYY-MM-DD.md`）：那一版能做什么、验到哪、还没罩到什么；旧的不回头改 |

## 本项目的特殊性

1. **没有 oracle。** 从零设计意味着没有参照实现可比对——移植类项目那种
   「拿现成工具的输出当标准答案」的便利这里不存在。功能正确性只能靠模型对拍，
   这是最大的隐性成本，见 `.claude/kb/prior-art.md`「三、Rust 侧现有轮子」。
2. **不进 Linux 主线**（D7（是否进 Linux 主线））。前几年按单人项目做，准入判据是门禁。

## 一句话版本

- 先定决策，再写代码——未定项还开着就写下去的实现多半要返工。第一个事务的代码是 2026-09-13 总审核把未定项集中交用户定案之后才开工的（`records/2026-09-13-总审核.md`）。
- 从事务开始，不从功能开始；第一个可运行目标是「正确提交一个事务」（`.claude/rules/fs-design.md`「从事务开始，不从功能开始」）。
- 门禁全绿**只构成第一个事务与一次覆盖写在模型层的崩溃一致性证据**——层 0 崩溃点重放（门禁 54 号）的负载是两条流：第一个事务，以及同一实例里的覆盖写 + 释放（发布 B）；多次挂载、回退、已释放落点的复用都没进来，checker 也只判第一版那部分不变量。
