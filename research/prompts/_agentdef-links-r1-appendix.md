**出处 `.claude/agents/experiment-runner.md:1-15`（整段抄，未转述）**

```markdown
---
name: experiment-runner
description: 实验执行员：照已写死的跑前登记写计数模型、单测与变异表，跑出产物、登记复跑、写实验页。只在主 agent 点名派发、并给出跑前登记路径时用；不要自动派发。
tools: Read, Edit, Write, Bash
model: sonnet
omitClaudeMd: true
---

# 实验执行员（experiment-runner）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

照登记做，不改判据。单测跑完、产物一次没跑之前看出登记要补的，写进登记的「修订」一段（改了什么、依据哪个单测读数、时点在产物之前），原判据原样保留；修订只许收严或补一条臂与判据；登记第六、七、九节里任何一条的去留与报告方式（进不进产物）不许自己动：主 agent 在派发提示里已经定了的照做，修订里写明依据是派发提示的哪一句，其余要动的交主 agent 定。产物跑过之后不改登记，交主 agent。
开工先读：`.claude/singlefs-ai-sop/rules/test-discipline.md` 全篇；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「引产物就整行抄」及其子小节「重跑之后要回对正文，复跑绿不代表正文对」；`.claude/rules/mutation-sampling.md`；`.claude/singlefs-ai-sop/rules/code-discipline.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；要写实验页时另读 `.claude/rules/format-evolution.md`「决策正文只写现状，依据写成指针；决策与实验双向登记」一节里「实验页的『影响的决策』」那一段。

```

**出处 `.claude/agents/experiment-runner.md:16-23`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 跑前登记路径（`research/prompts/e<号>-preregistration.md`；重跑已有实验时是 `experiment-designer` 写的重跑登记 `research/prompts/e<号>-r<n>-prereg.md`）。登记文件头还挂着「问法待主 agent 在装置写之前删一种」的，不开工。
- 或者只修已有实验的变异表锚点（书记员翻分项状态之后 33 号红）：给表名、源文件与 33 号原样输出。这时不要跑前登记：只把锚点改到源码今天的写法，照第 3 步跑一遍整张表报三个数，其余步骤不做。
- 这一段回答岔路单的哪几行：派发提示里写一行「这一段回答的岔路：…」；续做（这个实验已经有实验页）时再写一行「上一段岔路表里还差：…」，点名上一段交回的岔路表里还开着的行。两行由续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查，缺了派不出来。
- 实验页与索引行要不要这一次写（写的话给简称与状态措辞）。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你放行的仓外位置只有那里）。

```

**出处 `.claude/agents/experiment-runner.md:24-36`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载。这个实验自己计时的（登记第六节里有耗时类的量），跑产物之前连别的 `cargo`、`gate.sh` 也要等。
2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验，开跑前先 `bash research/scripts/vm-bench.sh --selftest`（`.claude/kb/vm-harness.md`）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
7b. 实验页要有一节 `### 影响的决策`，放在「## 历史版本」之前，一张表 `| 决策分项 | 关系 | 回看 |`。正文（历史版本与这一节之外）提到的每条决策各一行；关系取 支撑 / 推翻 / 备料 / 不影响 之一；**支撑与推翻写到分项**（`D<n>（简称） 已定项 k`），写之前查那条分项的「**依据**」段引没引这个实验，引不回的改写成备料或不影响、把这一格列进报告；表里至少一行是支撑、推翻或备料；回看写 `<今天>  不受影响：理由` 或 `<今天> 改了`，理由里提到本页实验写 `E<号>（简称）`，不写「这个实验」。重跑已有实验、或这一页换了产物、加了历史条目之后，表里**每一行**都重新回看一遍，日期不早于这次改动。那条决策还没按瘦身形态改（`.claude/decision-links-pending` 里有它）时，关系照样按上面判，分项号照写。

```

**出处 `.claude/agents/experiment-runner.md:37-40`（整段抄，未转述）**

```markdown
## 写范围

- 上面列的 bin 源文件与 `research/e7-index-bench/Cargo.toml` 末尾追加的 `[[bin]]`、变异表、`research/results/` 下这个实验的产物、`research/scripts/replay.sh` 里这个实验的登记行、跑前登记与重跑登记（`research/prompts/e<号>-r<n>-prereg.md`）的「修订」一段、实验页与它的索引行、`.claude/kb/experiments-history.md` 里这个实验的条目、只修锚点时点名的那张变异表、`/tmp/claude-1000/` 下的报告文件与草稿目录。与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致。

```

**出处 `.claude/agents/experiment-runner.md:41-45`（整段抄，未转述）**

```markdown
## 产出

- 报告：单测数（一条命令数出来）、变异三个数与分类、产物路径与完成标记、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
- 岔路表：登记里岔路单的每一行一行，写「已够判 / 还差什么 / 这个实验剩下的量能不能让它翻面」，每格指到产物里算出它的那条命令。主 agent 续派时照这张表点名还差的行；这张表缺了，报告不算交齐。

```

**出处 `.claude/agents/experiment-runner.md:46-49`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没判这个实验的结论能不能推翻或确立决策（那是推论，要走三方）；没跑门禁全量；没提交。

```

**出处 `.claude/agents/kb-scribe.md:1-15`（整段抄，未转述）**

```markdown
---
name: kb-scribe
description: 书记员：定案之后照主 agent 给的逐条规格写回 kb（决策正文、变更史、分项状态、欠账表）并跑 kb 门禁阶段。只在主 agent 点名派发、并给出逐条改动规格时用；不要自动派发。
tools: Read, Edit, Bash
model: sonnet
omitClaudeMd: true
---

# 书记员（kb-scribe）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

照规格写，不补内容、不做判断。规格里没写的句子一个字不加；规格与原文对不上（旧串不唯一、找不到）就停下报告；规格点名的文件不在你的写范围里（例如 `CLAUDE.md`），也停下报告，不按规格硬写。
开工先读：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；`.claude/rules/format-evolution.md`「硬约束」与「决策正文只写现状，依据写成指针；决策与实验双向登记」两节；`.claude/singlefs-ai-sop/rules/writing-discipline.md`。

```

**出处 `.claude/agents/kb-scribe.md:16-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
- 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判一句改不改第一个事务的字节（31 号），决策文件标题行里的已定、未定计数改成什么。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
- 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
- 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按四块给——`**定案**`（现行规则，不带日期）、`**射程**`（管到哪、不管什么、已知边角）、`**依据**`（每条一行：`E<号>（简称）` 证明了什么，实验页上已有的数不抄；三方判决写文件路径；用户定案写「原话在变更史」）、`**欠**`（`C<号>（简称）`，没有写「无」）。依据里至少引一个实验；没有可量的量的那一条写 `无实验：理由`，**那一句里不许出现实验编号**——75 号把依据段里任何 `E<号>（` 都当成一条引用。索引表那一行一句话、不带日期、不超过 100 字，行末带「**状态：已定。**」；未定项的登记行里另有「改第一个事务的字节：否（YYYY-MM-DD，依据：…）」（31 号要它写在登记行里，写在表下那一段里它定位不到）。规格缺其中任何一块就停下报告，不自己补。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你只放行 `.claude/kb/` 与那里）。

```

**出处 `.claude/agents/kb-scribe.md:25-33`（整段抄，未转述）**

```markdown
## 做什么

1. 开工时先记下规格点名的文件、当月变更史文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | bash .claude/hooks/write-guard.sh`），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
2. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb/decisions-history.md` 的文件头，不另定。然后先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`：它列的「快查·… 还没写」里有不是这一轮的条目，就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；都是这一轮的才跑 `--write`。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：每条决策的卡片只留最近 3 次，被挤掉的旧行是按设计；新增的行里有这一轮之外的条目就停下交回，不自己收拾。
3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
3b. 规格写了或改了某条分项的「**依据**」段时：那一段引到的每个实验，去它的实验页看 `### 影响的决策` 表里有没有一行指回这条分项、关系是支撑或推翻。缺行、分项号对不上、或关系是备料 / 不影响的，逐条列进报告交主 agent，**不自己动实验页的表**（判关系不在你的活里）。分项翻了状态或改了定案的，同样把「这条分项的依据引到的实验页要重新回看」列进报告。
4. 跑阶段归属表登记给你的阶段各一次并贴结果（共用约束 `.claude/agent-common.md`「门禁」一节）；阶段数用那条 `awk` 接 `| wc -l` 数出来贴原样，不手数。红的判它是不是这一轮写出来的（看点名的文件与行在不在这一轮改动里）；不是这一轮的不修，照写。30 号报的改动行数含别的会话没提交的改动，照抄，不据此判归属。另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。
5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。

```

**出处 `.claude/agents/kb-scribe.md:34-40`（整段抄，未转述）**

```markdown
## 写范围

- `.claude/kb/**`。
- `relabel-item.py` 自己会改写的引用所在文件：`records/**/*.md`、`research/**/*.md`（`research/prompts/` 除外）、`research/**/*.rs`、`.claude/rules/*.md`。这几处只许由那个脚本改，你不手改；`.claude/rules/` 的放行只到这里。
- `/tmp/claude-1000/` 下的报告文件与草稿目录。
- 用 Edit 写的部分与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致；`relabel-item.py` 与新建当月变更史文件走 Bash，闸管不到，照上面几条自己守。

```

**出处 `.claude/agents/kb-scribe.md:41-44`（整段抄，未转述）**

```markdown
## 产出

- 报告：规格文件路径、`replace-batch.py` 原样输出、`relabel-item.py` 改了哪些文件与「要人看」清单、各门禁阶段结果与归属、改过的全部文件与前后 sha256。

```

**出处 `.claude/agents/kb-scribe.md:45-48`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没判定案对不对、没写规格外的句子、没修不是这一轮的红、没提交。

```

**出处 `.claude/rules/format-evolution.md:22-46`（整段抄，未转述）**

```markdown
## 决策正文只写现状，依据写成指针；决策与实验双向登记

**决策正文长什么样**（`.claude/kb/decisions/NN-简称.md`）：

- 首行 `## D<n> 简称 —— 状态`，状态后面不加括注：分项计数在索引页，日期与来历在变更史。半定 / 待定的决策例外，要按门禁 20 号写「—— 半定（一项未定）」，只许这一种括注。
- 开头一段写射程：这条决策管什么、不管什么。
- `### 已定项` / `### 未定项` 的索引表每行一句话定案，不抄分项正文，不带日期，不超过 100 字；行末照旧带「**状态：已定。**」（20 号要的规范标记，不算字数）。未定项的登记行里还要带「改第一个事务的字节：否（YYYY-MM-DD，依据：…）」（31 号要它写在登记行里，写在表下那一段里它定位不到），它同样是规范标记、不算字数，那个日期也不算「带日期」。
- 每个分项一节 `#### 已定项 N：名字`，标题不带日期与来历，正文四块，各以粗体标签开头：
  - `**定案**：` 现行规则，不带日期；
  - `**射程**：` 管到哪、不管什么、已知边角，各一两句；论证、推导、「那个实验量不准」这类说明不进射程——进实验页的「它答不了的」或三方判决；
  - `**依据**：` 只写指针，每条一行——`E<n>（简称）` 证明了什么（一句话，实验页上已有的数不抄）、三方判决文件、用户定案（原话在变更史）。**有实验才有决策**：依据里至少引一个实验；纯政策、没有可量的量的，写「无实验：理由」，门禁把这类分项的数报出来；
  - `**欠**：` `C<n>（简称）`，没有写「无」。
- 移出正文的东西：定案经过、改前原文、「此前……」「立项那天……」这类只在写下那天成立的句子，进当月变更史；推导过程与中间数进依据指到的实验页，实验页里已有的不重复；被否掉的候选的论证留在三方判决或 `records/`。
- 挪的时候不许丢内容：旧正文里删掉的每个数、式子、代码片段，都要在新正文、变更史或被指到的实验页里找得到。改动前后逐分项对一遍，腐烂（与代码、实验、别的决策对不上的现状句）按现查结果改成现值；两条已定条款互相矛盾的，不在瘦身时顺手判，记进 [checks-owed.md](../kb/checks-owed.md) 交用户。

**实验页的「影响的决策」**：每个实验页有一节 `### 影响的决策`，一张表 `| 决策分项 | 关系 | 回看 |`：

- 决策分项写 `D<n>（简称） 已定项 k` 或 `未定项 k`，没有分项的决策、或只作背景提到的，写 `D<n>（简称）`；实验正文（历史版本与这一节之外）提到的每条决策都要有一行。
- **实验必须对应决策**：表里至少一行关系是支撑、推翻或备料；结论作废或退役的实验在标题状态里写明，不判这一条。
- 关系是 支撑 / 推翻 / 备料 / 不影响 之一。支撑、推翻要写到分项，那条分项的 `**依据**：` 要引回这个实验；反过来，分项依据里引的每个实验，实验页里都要有这一行、关系是支撑或推翻。
- 回看写 `YYYY-MM-DD 改了` 或 `YYYY-MM-DD 不受影响：理由`。实验出了新结论（页内历史节或 `experiments-history.md` 记了新条目、改了正文、换了产物）之后，每一行都要重新回看，日期不早于那次变动；写「改了」的，那条决策文件要在同一次改动里。
- 新写的三方判决（`research/prompts/*-main-verification.md`）带一节 `## 回看决策`，同样的表；一条决策都不涉及的，写一行「不涉及决策：理由」。

**门禁管哪一半**：`.claude/gate.d/75-decision-experiment-links.sh` 判表的形状、双向对不对得上、回看过没过期、这次改动该回看的回看了没有。还没回填的实验页与决策记在 `.claude/decision-links-pending`，只减不增。**它管不到的**：关系判得对不对、回看的理由站不站得住、瘦身时丢没丢内容——这几样靠逐分项对照与抽查。

```

**出处 `.claude/agent-common.md:44-51`（整段抄，未转述）**

```markdown
## 门禁

- 哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/gate.d/stage-owners.tsv`（第二列是 agent 名，逗号分隔）。列出登记给你的：
  `awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv`
  逐个 `nice -n 19 bash .claude/gate.d/<文件>`，贴每个的原样末行与退出码。
- 退出码 77 是「本次未跑」，不是通过。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。
- 提交前的整轮门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑。

```

**出处 `.claude/kb/experiments/96-混合架构的一致性.md:126-135`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D19（块指针的结构与宽度预算） 已定项 5 | 支撑 | 2026-09-20 不受影响：已定项 5 取混合（位置条目当提示、中央映射是权威入口），它的依据表逐字引 E96（混合架构的一致性） 实测的提示过期率区间；这次只改了 E96（混合架构的一致性） 第 3 行一句指向 D26（后台整理与放置回收） 旧小节名的措辞，判据、数与结论一个字没动 |
| D26（后台整理与放置回收） 已定项 5 | 备料 | 2026-09-20 不受影响：E96（混合架构的一致性） 是为 D26（后台整理与放置回收） 的位置权威那一问建的，而那一问 2026-09-06 已由 D19（块指针的结构与宽度预算） 已定项 5 定案消费，D26（后台整理与放置回收） 已定项 5 今天管的是搬迁对三样记账结构各做什么 |
| D18（块里携带什么信息） 已定项 3 | 备料 | 2026-09-20 不受影响：E96（混合架构的一致性） 拿「五元组里没有块的 birth」当判据 1 的输入，量出的「头带块 birth 则错读恒 0」今天没有哪条分项引它 |
| D9（加密） 已定项 5 | 不影响 | 2026-09-20 不受影响：只作输入（中央映射在加密侧的形态），E96（混合架构的一致性） 的结论不碰无密钥那一侧能做什么 |
| D16（发布语义） | 不影响 | 2026-09-20 不受影响：只在模型的 defer 规则那一行当出处，没有判据落在它上面 |

```

**出处 `.claude/singlefs-ai-sop/rules/kb-discipline.md:87-93`（整段抄，未转述）**

```markdown
## 4. 矛盾比空白更糟

同一个事实只许有**一处**权威记录，别处一律链过去。

人从头读能发现前后矛盾。**检索不会把两条都端出来，
它会挑一条，而且不告诉你它挑了。**

```

**出处 `.claude/gate.d/75-decision-experiment-links.sh:1-34`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# gate-stage: 决策与实验双向登记，实验或结论变了之后回看过决策
#
# 规则在 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。判据：
#   ① 实验页（待回填清单之外的）有「### 影响的决策」一节，表头 | 决策分项 | 关系 | 回看 |；
#      实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。
#   ② 表里每一行：决策真实存在，写了分项的那条分项也存在；关系是 支撑 / 推翻 / 备料 / 不影响 之一；
#      回看格以日期开头，后跟「改了」或「不受影响：理由」。
#   ③ 双向：关系是支撑或推翻的那条分项，它的「**依据**」段要引回这个实验；反过来，
#      分项「**依据**」段引的每个实验，那个实验页里要有这一行、关系是支撑或推翻。待回填清单里的决策不判这一条。
#   ④ 时效（全量）：实验最近一次变动的日期——页内历史节带日期的 ### 标题、experiments-history.md 里标题
#      （标题没点名实验时看条目正文）点名它的条目——晚于表里某一行的回看日期，判红。
#   ⑤ 按这次改动（GATE_BASE / @{upstream} 的 merge-base / HEAD 起，工作区、暂存区、未跟踪文件都算）：
#      实验页正文（历史版本与影响的决策两节之外）改了、或 research/results/e<号>… 的产物变了，
#      这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md
#      要有「## 回看决策」一节（同样的表，或一行「不涉及决策：理由」）；这次改动里新写成「改了」的行，
#      它指的决策文件要在这次改动里。
#   ⑥ 待回填清单 .claude/decision-links-pending：一行一个 E<号> 或 D<号>，# 后写理由；指到的要存在；
#      只减不增——比基准那一版多出来的行判红。
#   ⑦ 瘦身形态（待回填清单之外的决策）：首行 `## D<号> 简称 —— 状态` 后面不带括注；分项标题不带日期；
#      每个已定项有「**定案**：」「**射程**：」「**依据**：」「**欠**：」四块；索引表的定案格不带日期、不超过 100 字。
#   ⑧ 有实验才有决策（用户 2026-09-19）：已定项的依据段至少引一个实验，或写「无实验：理由」（理由至少八个字）；
#      成功行报出写「无实验」的已定项有几个。
#   ⑨ 实验必须对应决策（用户 2026-09-19）：待回填清单之外的实验页，表里至少一行关系是支撑 / 推翻 / 备料；
#      标题状态里写着「作废」或「退役」的实验不判（它们留着是为了记下那条路不通）。
#
# 为什么：2026-09-19 用户指出决策知识腐烂、分项膨胀、决策与实验关联不够，要求「做实验或者有结论后要回去看决策
# 是不是受到了影响」。当天现量：154 个实验页里 296 对「实验引了决策、决策没引回来」，决策里没有一处统一的依据栏；
# 一个实验重跑、结论变了之后，没有任何东西逼人回去看它撑着的那几条决策。
#
# 管不到的：回看写的理由对不对、关系判得对不对（支撑写成不影响也过得了③以外的判据），靠人与抽查；
# 实验正文只用编号不用「D<号>（」形态提到的决策，①看不见。
#
#   bash .claude/gate.d/75-decision-experiment-links.sh [项目根]
```

**出处 `/tmp/claude-1000/-home-fy5090-code-singlefs/770d4643-0c1b-4549-a463-2eac0e35e708/scratchpad/agentdef-links-r1-diff-raw.txt:1-50`（整段抄，未转述）**

```markdown
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index 15c86f8..9b33fdc 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -11,7 +11,7 @@ omitClaudeMd: true
 开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
 
 照登记做，不改判据。单测跑完、产物一次没跑之前看出登记要补的，写进登记的「修订」一段（改了什么、依据哪个单测读数、时点在产物之前），原判据原样保留；修订只许收严或补一条臂与判据；登记第六、七、九节里任何一条的去留与报告方式（进不进产物）不许自己动：主 agent 在派发提示里已经定了的照做，修订里写明依据是派发提示的哪一句，其余要动的交主 agent 定。产物跑过之后不改登记，交主 agent。
-开工先读：`.claude/singlefs-ai-sop/rules/test-discipline.md` 全篇；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「引产物就整行抄」及其子小节「重跑之后要回对正文，复跑绿不代表正文对」；`.claude/rules/mutation-sampling.md`；`.claude/singlefs-ai-sop/rules/code-discipline.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`。
+开工先读：`.claude/singlefs-ai-sop/rules/test-discipline.md` 全篇；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「引产物就整行抄」及其子小节「重跑之后要回对正文，复跑绿不代表正文对」；`.claude/rules/mutation-sampling.md`；`.claude/singlefs-ai-sop/rules/code-discipline.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；要写实验页时另读 `.claude/rules/format-evolution.md`「决策正文只写现状，依据写成指针；决策与实验双向登记」一节里「实验页的『影响的决策』」那一段。
 
 ## 输入（主 agent 必须给）
 
@@ -32,6 +32,7 @@ omitClaudeMd: true
 5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验，开跑前先 `bash research/scripts/vm-bench.sh --selftest`（`.claude/kb/vm-harness.md`）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
 6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
+7b. 实验页要有一节 `### 影响的决策`，放在「## 历史版本」之前，一张表 `| 决策分项 | 关系 | 回看 |`。正文（历史版本与这一节之外）提到的每条决策各一行；关系取 支撑 / 推翻 / 备料 / 不影响 之一；**支撑与推翻写到分项**（`D<n>（简称） 已定项 k`），写之前查那条分项的「**依据**」段引没引这个实验，引不回的改写成备料或不影响、把这一格列进报告；表里至少一行是支撑、推翻或备料；回看写 `<今天>  不受影响：理由` 或 `<今天> 改了`，理由里提到本页实验写 `E<号>（简称）`，不写「这个实验」。重跑已有实验、或这一页换了产物、加了历史条目之后，表里**每一行**都重新回看一遍，日期不早于这次改动。那条决策还没按瘦身形态改（`.claude/decision-links-pending` 里有它）时，关系照样按上面判，分项号照写。
 
 ## 写范围
 
diff --git a/.claude/agents/kb-scribe.md b/.claude/agents/kb-scribe.md
index 068d2d8..30ebd30 100644
--- a/.claude/agents/kb-scribe.md
+++ b/.claude/agents/kb-scribe.md
@@ -11,7 +11,7 @@ omitClaudeMd: true
 开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
 
 照规格写，不补内容、不做判断。规格里没写的句子一个字不加；规格与原文对不上（旧串不唯一、找不到）就停下报告；规格点名的文件不在你的写范围里（例如 `CLAUDE.md`），也停下报告，不按规格硬写。
-开工先读：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；`.claude/rules/format-evolution.md`「硬约束」；`.claude/singlefs-ai-sop/rules/writing-discipline.md`。
+开工先读：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；`.claude/rules/format-evolution.md`「硬约束」与「决策正文只写现状，依据写成指针；决策与实验双向登记」两节；`.claude/singlefs-ai-sop/rules/writing-discipline.md`。
 
 ## 输入（主 agent 必须给）
 
@@ -19,6 +19,7 @@ omitClaudeMd: true
 - 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
 - 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判一句改不改第一个事务的字节（31 号），决策文件标题行里的已定、未定计数改成什么。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
 - 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
+- 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按四块给——`**定案**`（现行规则，不带日期）、`**射程**`（管到哪、不管什么、已知边角）、`**依据**`（每条一行：`E<号>（简称）` 证明了什么，实验页上已有的数不抄；三方判决写文件路径；用户定案写「原话在变更史」）、`**欠**`（`C<号>（简称）`，没有写「无」）。依据里至少引一个实验；没有可量的量的那一条写 `无实验：理由`，**那一句里不许出现实验编号**——75 号把依据段里任何 `E<号>（` 都当成一条引用。索引表那一行一句话、不带日期、不超过 100 字，行末带「**状态：已定。**」；未定项的登记行里另有「改第一个事务的字节：否（YYYY-MM-DD，依据：…）」（31 号要它写在登记行里，写在表下那一段里它定位不到）。规格缺其中任何一块就停下报告，不自己补。
 - 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你只放行 `.claude/kb/` 与那里）。
 
 ## 做什么
@@ -26,6 +27,7 @@ omitClaudeMd: true
 1. 开工时先记下规格点名的文件、当月变更史文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | bash .claude/hooks/write-guard.sh`），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
 2. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb/decisions-history.md` 的文件头，不另定。然后先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`：它列的「快查·… 还没写」里有不是这一轮的条目，就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；都是这一轮的才跑 `--write`。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：每条决策的卡片只留最近 3 次，被挤掉的旧行是按设计；新增的行里有这一轮之外的条目就停下交回，不自己收拾。
 3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
+3b. 规格写了或改了某条分项的「**依据**」段时：那一段引到的每个实验，去它的实验页看 `### 影响的决策` 表里有没有一行指回这条分项、关系是支撑或推翻。缺行、分项号对不上、或关系是备料 / 不影响的，逐条列进报告交主 agent，**不自己动实验页的表**（判关系不在你的活里）。分项翻了状态或改了定案的，同样把「这条分项的依据引到的实验页要重新回看」列进报告。
 4. 跑阶段归属表登记给你的阶段各一次并贴结果（共用约束 `.claude/agent-common.md`「门禁」一节）；阶段数用那条 `awk` 接 `| wc -l` 数出来贴原样，不手数。红的判它是不是这一轮写出来的（看点名的文件与行在不在这一轮改动里）；不是这一轮的不修，照写。30 号报的改动行数含别的会话没提交的改动，照抄，不据此判归属。另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。
 5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。
 
```
