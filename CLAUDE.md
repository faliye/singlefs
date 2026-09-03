# singlefs

**一个从零设计的 COW 文件系统，Rust 实现。**
现有 COW 文件系统是**设计输入**（它们的病历和解法），不是移植目标。

当前里程碑：**格式设计阶段**——尚无磁盘格式、无代码。
先定决策里的待定项，再动第一行实现——索引在 `.claude/kb/decisions.md`，正文在 `.claude/kb/decisions/`。

## 规则（始终生效）

@.claude/singlefs-ai-sop/rules/show-me-test.md
@.claude/singlefs-ai-sop/rules/machine-first.md
@.claude/singlefs-ai-sop/rules/doc-discipline.md
@.claude/singlefs-ai-sop/rules/design-doc-discipline.md
@.claude/singlefs-ai-sop/rules/kb-discipline.md
@.claude/singlefs-ai-sop/rules/test-discipline.md
@.claude/singlefs-ai-sop/rules/evidence-discipline.md
@.claude/singlefs-ai-sop/rules/verify-before-claiming.md
@.claude/singlefs-ai-sop/rules/command-safety.md
@.claude/singlefs-ai-sop/rules/writing-economy.md
@.claude/singlefs-ai-sop/rules/session-wrapup.md

## 规范从哪来

| 项 | 值 |
|---|---|
| 上游仓 | `singlefs-ai-sop`，本机在兄弟目录 `../singlefs-ai-sop-zh`。同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的 |
| 项目里的副本 | `.claude/singlefs-ai-sop/`，是**拷贝，不是符号链接**；与上游同步靠重新拷贝一份 |
| 版本戳 | `.singlefs-ai-sop-version`。门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红——那是在提醒「规矩变过了，先读再跑」 |
| 怎么改 | 共享规则只能在**上游**改并抬 `VERSION`，然后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳。**不许在 `.claude/singlefs-ai-sop/` 里就地改**——下次同步就没了 |

上游管的是「项目怎么和 AI 协作」，不管文件系统怎么设计；
只有本工程需要的纪律放 `.claude/rules/`，不要往上游推。

## 项目本地规则

@.claude/rules/fs-design.md
@.claude/rules/format-evolution.md
@.claude/rules/three-way-inference.md

文件系统的设计纪律只有本工程需要，所以它不在共享 SOP 里——
共享 SOP 管的是「项目怎么和 AI 协作」，不管某一类系统怎么设计。

（上面那批是 [singlefs-ai-sop](.claude/singlefs-ai-sop/README.md) 分发的共享规则，
**改它们等于改所有项目**——要改就改上游并抬 `VERSION`，不许在项目里就地改。
上游的改动应当罕见：经常变说明规范本身没设计好。**作业在本仓，不在上游仓。**）

## 项目本地事实

| 文件 | 内容 |
|---|---|
| `.claude/kb/decisions.md` | **决策索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/decisions/` | 每个决策一个文件（`NN-简称.md`），正文与论证都在这里 |
| `.claude/kb/decisions-history.md` | 全部决策的变更史：每条写改前、改后、依据 |
| `.claude/kb/experiments.md` | **实验索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/experiments/` | 每个实验一个文件（`NN-简称.md`），正文与口径都在这里 |
| `.claude/kb/experiments-history.md` | 全部实验的变更史 |
| `.claude/kb/invariants.md` | 不变量清单，checker 是它的可执行形式 |
| `.claude/kb/prior-art.md` | 他家方案调研，含来源与口径 |
| `.claude/kb/pitfalls.md` | 避坑清单，每做设计决定回来对一遍 |
| `.claude/kb/checks-owed.md` | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| `.claude/kb/first-txn-layout.md` | 第一个事务写出哪些字节：每段每字段指向一条决策分项，指不到的就是格式级空白 |
| `.claude/kb/vm-harness.md` | 怎么把实验送进虚机在真块设备上跑：三个前置、卫生检查、虚机里才有的校验路径 |
| `.claude/kb/verification-build.md` | 三样未实现的验证手段（checker、事务层、崩溃点重放）怎么落地：消费哪些条款、被谁挡着、能复用什么、第一版范围、待定案的问题 |
| `research/scripts/replay.sh` | 复跑已入库的实验，与 `research/results/` 里那份逐字节比对；计时实验另有把 kb 里的数钉住的区间断言 |
| `research/scripts/fetch-refs.sh` | 把承重的外部文献重新固定到本机（URL + sha256 + 引用方），`pdf-text.py` 抽文本，断言在 `verify-citations.sh` |
| `.claude/rules/` | 项目本地规则（`fs-design.md` 设计纪律、`format-evolution.md` 格式演进纪律、`three-way-inference.md` 推论三方论证） |
| `records/` | 建设过程 |

## 门禁

门禁的目的是**把每一份提交抬到值得花人的时间去看那条线上**，不是把谁挡在外面。
它不按来源区分提交者，只区分带证据的和不带的。

```bash
bash .claude/scripts/gate.sh          # 准入门禁，提交前必跑
GATE_QEMU=1 bash .claude/scripts/gate.sh   # 再加 QEMU harness 自检

bash .claude/scripts/check.sh         # 快速反馈（格式/lint/构建/单测）
bash .claude/scripts/lkmm.sh          # 内存序（herd7 + litmus/）
bash .claude/scripts/qemu.sh --selftest    # QEMU harness 自检
bash .claude/scripts/gate-lint.sh     # 门禁自身：每条拒绝是否都给了下一步
bash .claude/scripts/env.sh           # 环境自检
bash .claude/gate.d/10-kb-rot.sh          # kb 腐化：引用悬空、结论悬空、条数对不上
bash .claude/gate.d/15-research-build.sh  # research 构建与单测（共享门禁只看 crates/，而证据住在 research/）
bash .claude/gate.d/20-kb-shape.sh        # kb 形状：用词、指代、链接、条数与标题相符
bash .claude/gate.d/21-decision-items-sync.sh # 决策分项清单与正文同步（--write 重新生成）
bash .claude/gate.d/22-item-ref-status.sh # 分项引用写的状态与正文的两张索引表相符
bash .claude/gate.d/23-link-targets.sh    # 相对链接与「第 N 节」指向到不到得了
bash .claude/gate.d/24-status-redundancy.sh   # 分项状态在索引表与正文里重复标注
bash .claude/gate.d/25-kb-deictic.sh      # kb 里的「本轮」锚不锚得到具体一轮
bash .claude/gate.d/26-number-name-sync.sh    # 编号简称在 doc-lint 够不到的地方也要一致
bash .claude/gate.d/27-format-constants.sh    # 格式常量在 kb 与实验源码之间同步
bash .claude/gate.d/28-cross-decision-status.sh # 说某条决策未定，而它已经定了
bash .claude/gate.d/29-settled-item-self-open.sh # 已定分项的正文里说自己还没定
bash .claude/gate.d/30-decision-history.sh # 决策变更有没有在 decisions-history.md 留条目
bash .claude/gate.d/31-blocking-verdict.sh # 每个未定项有没有判过改不改第一个事务的字节
bash .claude/gate.d/32-history-ordinal.sh # 本次新增的历史条目有没有撞号（并发会话共写一个仓）
bash .claude/gate.d/32-first-txn-fields.sh # 第一个事务的每个字段都指到一条真实存在的分项
bash .claude/gate.d/33-mutation-tables.sh # 每个实验二进制都有同名变异表（只验装置在，不跑变异）
bash .claude/gate.d/40-results-cited.sh   # 实验产物有没有写回：跑过的必须被点名，或写明未留存
bash .claude/gate.d/50-rules-manifest.sh  # 项目规则清单与本文件的 @ 引用逐项相等
bash .claude/gate.d/60-stale-open-items.sh # 未定项有没有被别处定了（跨文件 + 看历史）
bash .claude/gate.d/61-settled-same-file.sh # 定了新东西之后有没有回头看同文件的未定项（同文件 + 看 diff）
bash .claude/gate.d/70-citations.sh       # 外部引用还核得动吗（55 条承重引用，源码树不在也判红）
bash .claude/gate.d/80-absolute-assertions.sh # 每个实验都要有钉绝对值的断言（防「所有臂一起错」）却没回收
bash .claude/gate.d/85-repro-command.sh   # 点了产物的实验有没有写复跑命令
bash .claude/gate.d/86-experiment-orphans.sh # research 里的实验号在 kb 里有没有正文
bash .claude/gate.d/87-replay.sh          # 入库的实验数今天还复现得出来吗（默认只跑快的 19 个）
bash .claude/gate.d/89-stage-selftest.sh  # 上面这批阶段自己会不会红（样本在 gate.d/fixtures/）
```

**`.claude/gate.d/*.sh` 是项目本地门禁阶段**，`gate.sh` 按文件名顺序逐个跑，
每个记成一个独立阶段。放进去的检查**会红**，不是提醒句。
新增一条：写个 `.sh` 丢进去，头部写 `# gate-stage: <阶段名>`。
⚠️ **脚本存在但跑不起来（没执行位、语法错）一律判红，不许当成跳过。**

**Gate proves evidence requirements, not semantic correctness.**
绿色只说明证据要求被满足，不代表语义正确——`gate.sh` 每次都会列出未实现的阶段。

## 本项目的特殊性

1. **没有 oracle。** 从零设计意味着没有参照实现可比对——移植类项目那种
   「拿现成工具的输出当标准答案」的便利这里不存在。功能正确性只能靠模型对拍，
   这是最大的隐性成本，见 `kb/prior-art.md`「三、Rust 侧现有轮子」。
2. **格式还是软的。** 第一个外部用户出现前可以随时拆了重做。
   真正要慎重的是 `kb/decisions.md`，不是 `.rs` 文件。
3. **不进 Linux 主线**（D7）。前几年按单人项目做，准入判据是门禁——
   每个 patch 都要经过严格测试，不按提交来源区别对待。

## 一句话版本

- 先定决策，再写代码——D4/D8 未定之前写下去的实现多半要返工。
- 从事务开始，不从功能开始；第一个可运行目标是「正确提交一个事务」。
- 记账必须在提交时增量维护；任何要「事后扫一遍」的记账设计当场否决。
- 门禁全绿**不构成崩溃一致性证据**——崩溃点重放还没接进来。
