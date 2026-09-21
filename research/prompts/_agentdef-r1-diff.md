# 附录二：agent 定义两处改动（`git diff HEAD -- .claude/main-agent.md .claude/agents/kb-scribe.md` 原样；基准 HEAD 11a551b23864cf4620cea7c65fd48da5045d1e4b，生成时刻 2026-09-21T19:05:32Z UTC）

## 一、diff（`.claude/main-agent.md`、`.claude/agents/kb-scribe.md` 相对 HEAD 11a551b 的工作区改动）

无新增文件：两处改动都是对已有文件的定点替换（各一行），没有新文件需要附全文。

```diff
diff --git a/.claude/agents/kb-scribe.md b/.claude/agents/kb-scribe.md
index 17fb324..d3c8084 100644
--- a/.claude/agents/kb-scribe.md
+++ b/.claude/agents/kb-scribe.md
@@ -17,7 +17,7 @@ omitClaudeMd: true
 
 - 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
 - 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
-- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判一句改不改第一个事务的字节（31 号），决策文件标题行里的已定、未定计数改成什么。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
+- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行里的已定、未定计数改成什么。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
 - 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
 - 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按 `.claude/rules/format-evolution.md` 那一节的形态逐块给全（四块、索引行、未定项的字节判决），形态本身照那一节，这份定义不抄第二份；缺哪一块就停下报告，不自己补。
 - **规格改了某条分项的「**依据**」段时，同一份规格还要带上那个实验页 `### 影响的决策` 表里对应那几行的改动**（旧串、新串，一条分项一行）：门禁 75 号那条双向检查两侧要同时到位，而判关系是主 agent 的活、不是你的。规格里没带那几行就停下报告，列出缺哪几行。
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index f633441..af40d1a 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -40,7 +40,7 @@
 | 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
-| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再从全部判定里随机抽 20 行（五种判定各至少 2 行，不够的全抽）自己复判，抽到判错的那一段重派，重派交回后对那一段再抽 20 行，抽不出判错为止 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
+| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
 | 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
```
