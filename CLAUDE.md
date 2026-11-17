# singlefs

**一个从零设计的 COW 文件系统，Rust 实现。**
现有 COW 文件系统是**设计输入**（它们的病历和解法），不是移植目标。
**并行agent治理与上游sop治理**是另一个重要任务：遇到问题优先从流程、规范和门禁着手，目的不是改一行代码，而是避免再发。

当前里程碑：**「共享与回收」**（`.claude/kb/milestone/04-fourth-txn.md`，作业点、验收与次序在那份文件）。上一个**「删除与多次 COW」**（`.claude/kb/milestone/03-third-txn.md`）主体已完结，还开着段 4「优化」（二·6、六·4）。**「覆盖写、释放、回退与复用」**（`.claude/kb/milestone/02-second-txn.md`）已结束，没做完的转进里程碑三项五（五·7 到五·11）；它的崩溃放量已接进验证包，判器整套上了 GPU，`scripts/crash/crash-amplification.sh --list-items` 列分项。「新池新建文件」（`.claude/kb/milestone/01-first-txn.md`）的出口已满足。代码在 `crates/` 下十一个 crate：格式常量、核心、池级 checker、验证包 `singlefs-verification`、恢复重放取前缀、择根、扫环三截的核 `singlefs-replay-prefix`、参照文件系统 RefFS、函数单元图 `singlefs-unit-graph`、产品 GPU 包 `singlefs-background-gpu`、它的测试包 `singlefs-background-gpu-tests`、挂载就起的后台运行库 `singlefs-mount-runtime`、实验装置读写判定库的口子 `singlefs-device-library`（验证用的几个包怎么称呼见 `.claude/rules/verification.md`「定义与名字」）。

## 任务从哪进

**所有任务从 [.claude/main-agent.md](.claude/main-agent.md) 进。** `CLAUDE.md` 只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实。

@.claude/main-agent.md

改了定义要新派才生效：续做（SendMessage）沿用第一次派发时的定义；新建的定义要等几秒才派得出去。改了 `CLAUDE.md`（连同它 `@` 的规则）要新开会话，才对临时派的 agent（general-purpose 这类）生效；`.claude/agents/` 下的定义都开了 `omitClaudeMd`、不读 `CLAUDE.md`，要它们知道就写进定义或 `.claude/agent-common.md`。

## 经验与约定写在哪

别的会话、别的贡献者、派出去的 subagent 可能撞上的坑与约定，写进项目：规则（`.claude/rules/`）、agent 定义与 `.claude/agent-common.md`、kb、`records/`。私有 memory 只放用户个人的偏好（提交时间窗、回复语言这类）。往 `CLAUDE.md` 加内容之前先问它属于哪个 agent，属于就写进那个定义或共用约束。

subagent 与协作工具的欠账与优化项记在现行账本 `records/2026-11-17-subagent账本自上而下清欠.md`，不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身；`records/2026-09-16-subagent拆分提案.md`、`records/2026-10-23-subagent拆分提案.md`、`records/2026-10-29-subagent拆分提案.md` 三版封存，不再改（除非要查来历，别读它们）。定义、共用约束、三方流程与配套脚本的问题：修它们本身就是这一轮的出口时，直接改，不先问，改完记进现行账本；做别的任务时撞上的，照 `.claude/main-agent.md`「禁止」一节「禁止自行扩大任务范围」记下或弹窗。改了定义、共用约束、规则、技能、门禁、钩子、脚本与两个服务的，按批走一轮三方（`.claude/main-agent.md`「一轮怎么开、怎么收」第 4 条）。脚本自证与门禁样本登记在 `.claude/hooks/selftest-registry.tsv`，改了就由主 agent 照钩子拒绝说明里写的起法经任务服务交。改了触发文件的，写阶段同步记录（门禁 doc-process-records 的 knowledge-sync 格判）。

## 规则（始终生效）

@.claude/singlefs-ai-sop/rules/sop-first.md
@.claude/singlefs-ai-sop/rules/show-me-test.md
@.claude/singlefs-ai-sop/rules/writing-discipline.md
@.claude/singlefs-ai-sop/rules/communication-discipline.md
@.claude/singlefs-ai-sop/rules/rules-discipline.md
@.claude/singlefs-ai-sop/rules/kb-discipline.md
@.claude/singlefs-ai-sop/rules/command-safety.md

上游四篇不 `@` 常驻：`.claude/singlefs-ai-sop/rules/design-doc-discipline.md`、`.claude/singlefs-ai-sop/rules/pushback-discipline.md` 开会话先整份读一次、之后按名字回查；`.claude/singlefs-ai-sop/rules/code-discipline.md`、`.claude/singlefs-ai-sop/rules/preflight-discipline.md` 由派出去的 agent 的定义点名读，主 agent 自己写 `crates/`、测试或脚本之前先整份读一次。
<!-- doc-lint:read-once design-doc-discipline.md pushback-discipline.md code-discipline.md preflight-discipline.md -->

## 规范从哪来

| 项 | 值 |
|---|---|
| 上游仓 | `singlefs-ai-sop`，在本机兄弟目录 `../singlefs-ai-sop-zh`。同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的 |
| 项目里的副本 | `.claude/singlefs-ai-sop/`（[README](.claude/singlefs-ai-sop/README.md)），是**拷贝，不是符号链接**。同步：`rsync -a --exclude '.git' --exclude CLAUDE.md ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/`，再 `diff -rq --exclude=.git --exclude=CLAUDE.md ../singlefs-ai-sop-zh .claude/singlefs-ai-sop` 确认一致，然后刷版本戳；不用 `cp -r` |
| 版本戳 | `.singlefs-ai-sop-version`；门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红——那是在提醒「规矩变过了，先读再跑」 |
| 怎么改 | 共享规则只能在**上游**改，**不许在 `.claude/singlefs-ai-sop/` 里就地改**；**作业在本仓，不在上游仓**。从本仓的会话改上游**只往文件里填内容**（规则正文、脚本逻辑、三语译文那几行）：不跑 `bump.sh`、不动 `VERSION`、不改 MANIFEST / SOURCE-MANIFEST 与译文首行的溯源哈希、不写带版本号的 CHANGELOG 条目、不提交，这些归做发版的会话。填之前先看上游三语仓 `git status`，有人在发版就等它提交完，填完告诉发版会话填了哪些文件哪几段。发版之后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳 |

## 项目本地规则

@.claude/rules/fs-design.md
@.claude/rules/format-evolution.md
@.claude/rules/three-way-inference.md
@.claude/rules/implementation-workflow.md
@.claude/rules/verification.md
@.claude/rules/implementation-first.md
@.claude/rules/changelog-format.md
@.claude/rules/timer-only-remind.md
@.claude/rules/evidence-discipline.md
@.claude/rules/session-wrapup.md
@.claude/rules/show-me-test-by-review.md
@.claude/rules/writing-discipline-by-review.md
@.claude/rules/command-safety-by-review.md
@.claude/rules/long-run-split.md

四篇项目本地规则不 `@` 常驻，派出去的 agent 由定义点名读，主 agent 碰到对应的活之前先整份读一次：判变异没红、无效变异之前读 `.claude/rules/mutation-sampling.md`；写测试用例、入口脚本或自证之前读 `.claude/rules/test-script-discipline.md`；搬文件或目录、改全仓术语之前读 `.claude/rules/path-moves.md`；新加脚本、往 `scripts/` 或 `research/` 放文件之前读 `.claude/rules/directory-layout.md`。
<!-- doc-lint:read-once mutation-sampling.md test-script-discipline.md path-moves.md directory-layout.md -->
另三篇不 `@` 常驻：`.claude/rules/machine-first.md` 开会话先整份读一次、之后按名字回查；`.claude/rules/code-discipline-by-review.md`、`.claude/rules/test-discipline.md` 由派出去的 agent 的定义点名读，主 agent 自己写 `crates/`、测试或脚本之前先整份读一次。
<!-- doc-lint:read-once machine-first.md code-discipline-by-review.md test-discipline.md -->
参考件 `.claude/rules/verification-reference.md` 不 `@` 常驻：各道门禁与入口怎么跑、门禁的结构、判定库的表与登记、复用与缩域的细则，到任何一份规则、技能或 `.claude/main-agent.md` 派发表点名它某一节的那一步之前，读点名的那一节。
<!-- doc-lint:read-once verification-reference.md -->

共享 SOP 只管「项目怎么和 AI 协作」；只有本工程需要的纪律（文件系统怎么设计、压在本机资源上的流程）放 `.claude/rules/`，不往上游推。规则、agent 定义、共用约束与 skill 正文照 `.claude/singlefs-ai-sop/rules/rules-discipline.md` 只写怎么做。

## 项目本地事实

| 文件 | 内容 |
|---|---|
| `.claude/kb/decisions.md` | **决策索引**：编号、简称、状态、进度、指向正文的链接 |
| `.claude/kb/decisions/` | 每个决策一个文件（`NN-简称.md`），正文与论证 |
| `.claude/kb/decisions-history.md` | 决策变更史，一条决策一节，形态照 `.claude/rules/changelog-format.md` |
| `.claude/kb/experiments.md` | **实验索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/experiments/` | 每个实验一个文件（`NN-简称.md`），正文与口径 |
| `.claude/kb/experiments-history.md` | 实验变更史，一个实验一节，形态照 `.claude/rules/changelog-format.md` |
| `crates/singlefs-checker/invariants/` | 不变量清单（按类拆开），checker 是它的可执行形式 |
| `.claude/kb/feature-bits.md` | feature bit 的记账表（位号 / 类别 / 名称 / 引入版本 / 引入 commit / 状态 / 语义一句话），形态由 D15（格式冻结政策） 已定项 10 定；**位号的唯一登记位**是 D15（格式冻结政策） 已定项 4 那张表，门禁 code-source-discipline 的 feature-bits 格判两处逐位一致 |
| `.claude/kb/term-renames.md` | 全仓术语改名登记表（旧名 / 新名 / 匹配）；`scripts/kb/sweep-term.py` 照它换全仓、`--check` 数剩几处，没有门禁复查；做法见 `.claude/rules/path-moves.md`「改一个全仓术语」 |
| `.claude/kb/freeze-layer-membership.md` | 冻结层归属登记表：一行一个结构，写它落 D15（格式冻结政策） 已定项 7 的哪一层或哪个独立冻结组件、态别、退不退出冻结、依据哪条分项；门禁 doc-decisions 的 freeze-layer-membership 格判三条（每层、每棵树、每个单元类都有行；写「退出」的必须是派生态；依据点名的分项存在且已定；三份清单怎么现读见 `.claude/kb/freeze-layer-membership.md`「门禁判哪三条」），态别判得对不对门禁管不了 |
| `.claude/kb/tooling.md` | 本机工具的现状与用法（本地模型与网关、Rust 工具链、虚机装置、造设备、并发暂存与提交、门禁常见红的写法） |
| `.claude/kb/INDEX.md` | kb 的导航表，本身不放事实；`kb-discipline.md` 第 8 条对它不判「## 历史版本」节；带历史节的 kb 文件登记在 `.claude/history-carriers` |
| `.claude/kb/prior-art.md` | 他家方案调研，含来源与口径 |
| `.claude/kb/pitfalls.md` | 避坑清单，每做设计决定回来对一遍 |
| `.claude/kb/checks-owed.md` | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| `.claude/kb/pending-rules.md` | 待议的规矩：做不成会红的检查、因而撤出规则的条款，每条带原文、原来在哪、为什么撤、还缺什么；不约束执行，要立成规则先补那道检查 |
| `.claude/kb/mount-parameters.md` | 挂载参数表：取值、默认值、存不存盘、挂着能不能切与代码落点 |
| `.claude/kb/layout/` | 每个里程碑写出哪些字节，与 `milestone/` 同号（`NN-简称.md`）；`01-first-txn.md` 每段每字段指向一条决策分项、给宽度与取值，指不到的就是格式级空白，第八节是根槽写路径的段序列登记表；之后的里程碑只登记新写出的形态，宽度与落点以 `01` 为准 |
| `.claude/kb/vm-harness.md` | 怎么把实验送进虚机在真块设备上跑 |
| `.claude/kb/verification-build.md` | 三样验证手段（checker、事务层、崩溃点重放）怎么落地、被谁挡着 |
| `.claude/kb/milestone/` | 里程碑规划，一个里程碑一个文件（`NN-简称.md`）：每步设想、验收标准、写出的字节、会碰到的决策点；每步开工前回来改 |
| `scripts/README.md` | `scripts/` 下各子目录放什么；每个子目录、`research/` 各有一份 `README.md`，规矩在 `.claude/rules/directory-layout.md` |
| `task-server/README.md` | 统一任务服务：一切跑项目代码的任务经它起，用法、判法与拒绝清单 |
| `scripts/kb/fetch-refs.sh` | 把承重的外部文献重新固定到本机（URL + sha256 + 引用方），`pdf-text.py` 抽文本，断言在 `verify-citations.sh` |
| `scripts/gate/stage-mine.py` | 几个会话共写一批文件时只把这一轮的块放进暂存区（命中 `--match` 的进，命中 `--foreign` 的拒绝） |
| `scripts/kb/replace-once.py` | 定点替换：旧串必须恰好命中一次，否则不写；几个会话共写一批文件时只许这样改，不许整份重写 |
| `scripts/kb/insert-row.py` | 往公共表里插一行：锚点必须恰好命中一次，写前比对读时记的 sha256，变了就拒绝 |
| `scripts/experiment-apparatus/perf-by-milestone.md` | singlefs 与六家文件系统按里程碑的性能对比；数来自 E152（按里程碑对比六家文件系统的文件性能），表由 `scripts/experiment-apparatus/e152-tables.py` 生成 |
| `records/` | 建设过程 |
| `briefs/` | 每次更新的简报，按日期一份：那一版能做什么、验到哪、还没罩到什么；旧的不回头改。三语：`YYYY-MM-DD.md` 中文（默认）、`YYYY-MM-DD.en.md` 英语、`YYYY-MM-DD.ja.md` 日语，仓根 README 与 `.claude/kb/` 下的文件同样三份；改一份就三份一起改，照 `.claude/skills/trilingual/SKILL.md`，用词照 `.claude/kb/translation-glossary.md`；受管文件登记在 `.claude/trilingual-documents`，门禁 doc-text 的 trilingual 六格判 |
| `.claude/kb/translation-glossary.md` | 翻译术语表：中、英、日三语并排，新术语先加进表再用 |

## 本项目的特殊性

1. **没有 oracle。** 没有参照实现可比对，功能正确性靠模型对拍（`.claude/kb/prior-art.md`「三、Rust 侧现有轮子」）。
2. **不进 Linux 主线**（D7（是否进 Linux 主线））。准入判据是门禁。

## 一句话版本

- 先定决策，再写代码。
- 从事务开始，不从功能开始；第一个可运行目标是「正确提交一个事务」（`.claude/rules/fs-design.md`「从事务开始，不从功能开始」）。
- 门禁全绿**只构成模型层的崩溃一致性证据**（真设备那一侧只有门禁 qemu-device-streams 的六个模式）；崩溃点重放（门禁 crash-point-replay）的负载是 `.claude/gate.d/stage-inputs.tsv` 登记的崩溃枚举用例（`grep -c '^crash-case:'` 数）加崩溃放量的 29 项；checker 判 114 条不变量（与代码清单 `IMPLEMENTED_INVARIANTS` 同数；I-8.4（重放幂等） 已实现在崩溃注入的二次崩溃上、不进池级 checker 清单；两个数的数法登记在 `.claude/gate.d/counted-claims.tsv`；哪几条按回退候选集判、候选集的下界怎么取，见 `crates/singlefs-checker/invariants/README.md`「## 状态」那一段与各条的状态列）。
