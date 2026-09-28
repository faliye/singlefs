# singlefs

**一个从零设计的 COW 文件系统，Rust 实现。**
现有 COW 文件系统是**设计输入**（它们的病历和解法），不是移植目标。
**并行agent治理与上游sop治理**
agent治理与上游sop治理，是另外一个重要的任务。因此遇到问题优先从流程、规范和门禁等方面着手解决问题，目的不是改一行代码，而是避免再发。

当前里程碑：**「覆盖写、释放、回退与复用」**（`.claude/kb/milestone/02-second-txn.md`，做到哪一步看那份文件）。收尾九步（`records/2026-09-24-里程碑二收尾调度.md`）做到第 9 步；崩溃放量（增补 4：身份、KV、双机、GPU 核对、前缀共享、三段流）已接进 `crates/singlefs-checker-tier`，固定脚本到 D 全域 51 773 558 态零违例，到 E 全域在跑。上一个里程碑「新池新建文件」（`.claude/kb/milestone/01-first-txn.md`）的出口已经满足，代码在 `crates/` 下五个 crate（格式常量、核心、池级 checker、harness 档、checker 档；两档怎么分见 `.claude/rules/verification.md`）。

## 任务从哪进

**所有任务从 [.claude/main-agent.md](.claude/main-agent.md) 进**：主 agent 的职责、一轮怎么开怎么收（出口、判阻塞、收拢、再判）、派出去之后怎么盯、交回怎么读、派发提示怎么写、什么时候派哪个 agent 的调度表，都在那一份。这份文件只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实。

@.claude/main-agent.md

改了定义要新派才生效：续做（SendMessage）沿用第一次派发时的定义；新建的定义要等几秒才派得出去。改了本文件（连同它 `@` 的规则）要新开会话才对临时派的 agent（general-purpose 这类）生效：同一会话里派的继承会话开始时那一份；`.claude/agents/` 下的定义都开了 `omitClaudeMd`，根本不读本文件，要它们知道就写进定义或 `.claude/agent-common.md`。计划、实测与现状在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它）。

## 经验与约定写在哪

别的会话、别的贡献者、派出去的 subagent 可能撞上的坑与约定，写进项目：规则（`.claude/rules/`）、agent 定义与 `.claude/agent-common.md`、kb、`records/`。私有 memory 只放用户个人的偏好（提交时间窗、回复语言这类）：它只在本机，别的贡献者看不到，定义开了 `omitClaudeMd` 的 subagent 也读不到。往本文件加内容之前先问它属于哪个 agent，属于就写进那个定义或共用约束。

subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案.md`，不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身。定义、共用约束、三方流程与配套脚本的问题：修它们本身就是这一轮的出口时，直接改，不先问；是做别的任务时撞上的，照 `.claude/main-agent.md`「禁止」一节，记下或弹窗，不自行扩大范围。改完记进那份计划；门禁、样本与脚本自证归提交时，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh` 判，改完一样都不跑。改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 doc-process-records 的 agent-def-adversarial-review 格判），改了触发文件的写阶段同步记录（门禁 doc-process-records 的 knowledge-sync 格判）。

## 规则（始终生效）

@.claude/singlefs-ai-sop/rules/engineering-philosophy.md
@.claude/singlefs-ai-sop/rules/sop-first.md
@.claude/singlefs-ai-sop/rules/show-me-test.md
@.claude/singlefs-ai-sop/rules/machine-first.md
@.claude/singlefs-ai-sop/rules/code-discipline.md
@.claude/singlefs-ai-sop/rules/writing-discipline.md
@.claude/singlefs-ai-sop/rules/rules-discipline.md
@.claude/singlefs-ai-sop/rules/design-doc-discipline.md
@.claude/singlefs-ai-sop/rules/kb-discipline.md
@.claude/singlefs-ai-sop/rules/test-discipline.md
@.claude/singlefs-ai-sop/rules/evidence-discipline.md
@.claude/singlefs-ai-sop/rules/verify-before-claiming.md
@.claude/singlefs-ai-sop/rules/pushback-discipline.md
@.claude/singlefs-ai-sop/rules/command-safety.md
@.claude/singlefs-ai-sop/rules/session-wrapup.md
@.claude/singlefs-ai-sop/rules/preflight-discipline.md

## 规范从哪来

| 项 | 值 |
|---|---|
| 上游仓 | `singlefs-ai-sop`，在本机兄弟目录 `../singlefs-ai-sop-zh`。同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的 |
| 项目里的副本 | `.claude/singlefs-ai-sop/`（[README](.claude/singlefs-ai-sop/README.md)），是**拷贝，不是符号链接**；与上游同步靠重新拷贝一份：`rsync -a --exclude '.git' ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/`，拷完 `diff -rq --exclude=.git ../singlefs-ai-sop-zh .claude/singlefs-ai-sop` 确认一致再刷版本戳（`cp -r` 会把上游的 `.git` 一起拷进副本，第二次同步时那些只读 object 报一屏 Permission denied） |
| 版本戳 | `.singlefs-ai-sop-version`。门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红——那是在提醒「规矩变过了，先读再跑」 |
| 怎么改 | 共享规则只能在**上游**改。从本仓的会话改上游**只往文件里填内容**（规则正文、脚本逻辑、三语译文那几行）：不跑 `bump.sh`、不动 `VERSION`、不改 MANIFEST / SOURCE-MANIFEST 与译文首行的溯源哈希、不写带版本号的 CHANGELOG 条目、不提交，这些归做发版的会话；填之前先看上游三语仓 `git status`，有人在发版就等它提交完，填完告诉发版会话填了哪些文件哪几段。发版之后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳。**不许在 `.claude/singlefs-ai-sop/` 里就地改**——下次同步就没了，而且改它们等于改所有项目。上游的改动应当罕见：经常变说明规范本身没设计好；**作业在本仓，不在上游仓** |

## 项目本地规则

@.claude/rules/fs-design.md
@.claude/rules/format-evolution.md
@.claude/rules/three-way-inference.md
@.claude/rules/mutation-sampling.md
@.claude/rules/implementation-workflow.md
@.claude/rules/verification.md
@.claude/rules/implementation-first.md
@.claude/rules/path-moves.md
@.claude/rules/changelog-format.md

共享 SOP 只管「项目怎么和 AI 协作」；只有本工程需要的纪律（文件系统怎么设计、压在本机资源上的流程）放 `.claude/rules/`，不往上游推。

规则正文只写怎么做（`.claude/singlefs-ai-sop/rules/rules-discipline.md`）：实测、论证、定案日期不写进规则，经过要留就留在 `records/` 与 `.claude/kb/` 里已有的那一份。

## 项目本地事实

| 文件 | 内容 |
|---|---|
| `.claude/kb/decisions.md` | **决策索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/decisions/` | 每个决策一个文件（`NN-简称.md`），正文与论证都在这里 |
| `.claude/kb/decisions-history.md` | 决策变更史，一条决策一节、节内是它的完整历史，组织形态见 `.claude/rules/changelog-format.md` |
| `.claude/kb/experiments.md` | **实验索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/experiments/` | 每个实验一个文件（`NN-简称.md`），正文与口径都在这里 |
| `.claude/kb/experiments-history.md` | 全部实验的变更史，一个实验一节，组织形态见 `.claude/rules/changelog-format.md` |
| `.claude/kb/invariants.md` | 不变量清单，checker 是它的可执行形式 |
| `.claude/kb/feature-bits.md` | feature bit 的记账表（位号 / 类别 / 名称 / 引入版本 / 引入 commit / 状态 / 语义一句话），形态由 D15（格式冻结政策） 已定项 10 定。**位号的唯一登记位不在这里**，在 D15（格式冻结政策） 已定项 4 那张表；门禁 code-source-discipline 的 feature-bits 格判两处逐位一致 |
| `.claude/kb/term-renames.md` | 全仓术语改名的登记表，一行一条（旧名 / 新名 / 匹配），门禁 doc-text 的 term-renames 格按它查全仓不再出现旧名；怎么改名见 `.claude/rules/path-moves.md`「改一个全仓术语」 |
| `.claude/kb/freeze-layer-membership.md` | 冻结层归属登记表：一行一个结构，写它落 D15（格式冻结政策） 已定项 7 的哪一层或哪个独立冻结组件、态别、退不退出冻结、依据哪条分项；门禁 doc-decisions 的 freeze-layer-membership 格按它判三条（每层、每棵树、每个单元类都有行；写「退出」的必须是派生态；依据点名的分项存在且已定）。态别判得对不对门禁管不了 |
| `.claude/kb/tooling.md` | 本机工具与模型的事实（本地腿用哪个模型、网关怎么调、量化到几位、中文为什么会退化性复读） |
| `.claude/kb/INDEX.md` | kb 的导航表，本身不放事实；它是唯一不留「## 历史版本」节的 kb 文件（`kb-discipline.md` 第 8 条显式豁免） |
| `.claude/kb/prior-art.md` | 他家方案调研，含来源与口径 |
| `.claude/kb/pitfalls.md` | 避坑清单，每做设计决定回来对一遍 |
| `.claude/kb/checks-owed.md` | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| `.claude/kb/layout/` | 每个里程碑写出哪些字节，一个里程碑一个文件，与 `milestone/` 同号（`NN-简称.md`）：`01-first-txn.md` 每段每字段指向一条决策分项、给宽度与取值，指不到的就是格式级空白，第八节是根槽写路径的段序列登记表；之后的里程碑只登记新写出的形态，宽度与落点仍以 `01` 为准 |
| `.claude/kb/vm-harness.md` | 怎么把实验送进虚机在真块设备上跑 |
| `.claude/kb/verification-build.md` | 三样验证手段（checker、事务层、崩溃点重放）怎么落地、被谁挡着 |
| `.claude/kb/milestone/` | 里程碑规划，一个里程碑一个文件（`NN-简称.md`）：每步设想、验收标准、写出的字节、会碰到的决策点；每步开工前回来改 |
| `research/scripts/fetch-refs.sh` | 把承重的外部文献重新固定到本机（URL + sha256 + 引用方），`pdf-text.py` 抽文本，断言在 `verify-citations.sh` |
| `research/scripts/stage-mine.py` | 几个会话共写一批文件时，只把这一轮的块放进暂存区（命中 `--match` 的进，命中 `--foreign` 的拒绝） |
| `research/scripts/replace-once.py` | 定点替换：旧串在文件里必须恰好命中一次，否则不写；几个会话共写一批文件时只许这样改，不许整份重写 |
| `research/scripts/insert-row.py` | 往公共表里插一行：按锚点定位（锚点必须恰好命中一次），写之前复核文件没被别人改过（读时记 sha256，写前再比一次，变了就拒绝）。`replace-once.py` 只解决「改一处已有的文字」，插入没有旧串可替、只能读整份写回，两个会话一前一后会互相覆盖 |
| `research/perf-by-milestone.md` | singlefs 与六家文件系统按里程碑的性能对比；数来自 E152（按里程碑对比六家文件系统的文件性能），表由 `research/scripts/e152-tables.py` 生成 |
| `records/` | 建设过程 |
| `briefs/` | 每次更新的简报，按日期一份（`YYYY-MM-DD.md`）：那一版能做什么、验到哪、还没罩到什么；旧的不回头改 |

## 本项目的特殊性

1. **没有 oracle。** 从零设计意味着没有参照实现可比对——移植类项目那种
   「拿现成工具的输出当标准答案」的便利这里不存在。功能正确性只能靠模型对拍，
   这是最大的隐性成本，见 `.claude/kb/prior-art.md`「三、Rust 侧现有轮子」。
2. **不进 Linux 主线**（D7（是否进 Linux 主线））。前几年按单人项目做，准入判据是门禁。

## 一句话版本

- 先定决策，再写代码——未定项还开着就写下去的实现多半要返工。
- 从事务开始，不从功能开始；第一个可运行目标是「正确提交一个事务」（`.claude/rules/fs-design.md`「从事务开始，不从功能开始」）。
- 门禁全绿**只构成新池新建文件在模型层的崩溃一致性证据**；里程碑「覆盖写、释放、回退与复用」步 0 那条固定脚本（覆盖写、释放、重开写行、暖机、回退、抬 F、复用各一次）在管理员回退改成挂着时的向前发布之后，层 0 用例只改到编得过、下游钉的值没有重核，重写与重跑归层 0 规模那一轮与实六，这之前它的层 0 结果不作数（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实三交回」那一行）——层 0 崩溃点重放（门禁 54-layer0-replay）的负载是两条流：新池新建文件，以及固定脚本到 E；checker 判 48 条不变量（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`；第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）、I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变）、I-5.4（分配记录罩住的槽互不相交）、I-1.8（归并后版本全序）、I-7.3（环健康性） 与 I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开）、I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读）、I-8.9（一次发布的记录序号连续且只有末条带标志） 、I-7.12（系统配置 F 不低于同盘根上的 F） 、I-1.11（映射 key 与单元头相符）、I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序）；I-3.1（已分配统计对得上）、I-2.1（校验和与内容匹配） 与后加的六条里除 I-3.8（实例表行唯一且低于挂载根） 之外的五条按回退候选集判，候选集的下界取 F_生效（各幸存盘最新持久有效根带的 F 与系统配置里的 F 取大）；I-3.1（已分配统计对得上） 在最新根带的 F 低于 F_生效 时报不适用）。
