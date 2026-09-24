# m2-runner-in-repo-harness-r1 云端正推（Sonnet）报告

分到 W2、W3。开工快照 `sha256sum -c` 全部 OK（4 个文件，2026-09-23 现查）：
```
.claude/agents/experiment-runner.md: OK
.claude/hooks/agent-write-scope.tsv: OK
.claude/rules/implementation-first.md: OK
crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs: OK
```

判据第 2 条口径：本报告里凡标「推得出」的，都附整段原文（`grep -nF` 命中过的行号）；抄不出整段原文的一律「推不出」。不使用「E156 已经这么用了」作为推导步骤。

## 现场事实：这一支改动本身尚未提交

`git diff --stat` 现查（2026-09-23）：`.claude/agents/experiment-runner.md`、`.claude/hooks/agent-write-scope.tsv` 都是工作区里的未提交改动（`M`），`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 是未跟踪文件（`??`）。这意味着被判的这条分支还没有经过 `.claude/rules/implementation-workflow.md` 第 20 行「改它们与改 `crates/` 同规矩：写完要走一轮三方……门禁 72 号判形式（形态照 56 号）」要求的那一轮三方——本轮三方论证正是在补这一步，不是这一步之外的复核。

⚠️ 同一次 `git status` 里还看到 `crates/singlefs-harness/src/bin/first_transaction_*.rs` 三个文件被修改（`M`），但这不在这一轮开工快照点名的 4 个文件之内，也不在 experiment-runner 的写范围内（写范围只放行 `e*.rs`，不含 `first_transaction_*.rs`）：按 `.claude/agent-common.md`「本机常有别的会话在同一个仓里干活……看到别人没提交的改动不碰、不修」，本报告不分析这几个文件的改动内容，只记录其存在，供 W1（Opus）判 W1 时留意「同一目录下混着生产装置」这一事实（不作为我这两格的判据）。

## W2　写范围放宽到 `crates/`——判定：**推不出**

### 今天写的是什么（被判对象，整段原文）

`.claude/hooks/agent-write-scope.tsv:12`：
```
experiment-runner	crates/singlefs-harness/src/bin/e*.rs	实验装置要驱动真实 crates/ 代码时的只读 bin（2026-09-22 E156 撞上：岔路问的是「今天这份代码的性质」，另写一份独立模型答不了）。限定 e 开头加数字，碰不到 first_transaction_* 那几个生产装置；这个 bin 不许改 core/checker，由代码三方与门禁 56 号管
```

`.claude/agents/experiment-runner.md:27`（只抄含放宽内容的括注部分，整句未截断）：
```
（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑）
```

放宽带的三条限制（只读只驱动观测 / 不许改 core-checker / 变异表走 59 号）与 tsv 的「限定 e 开头加数字」，字面上都只出现在这两处——即被判对象自己。

### 去 `.claude/agent-common.md` 找

`grep -n "写范围"` 命中：
- `.claude/agent-common.md:32`：
```
- 只写派发提示与定义「写范围」一节给的路径。写范围闸（项目 settings 里的 hook，表在 `.claude/hooks/agent-write-scope.tsv`）只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守。
```
这一句只说写范围闸**怎么执行**（机制），没有一个字说**该往一个 agent 的写范围里放什么、放多宽**。它不能推出「放行 `crates/singlefs-harness/src/bin/e*.rs`」这个具体写法，也推不出「必须限定 e 开头加数字」这条narrow化的做法。`grep -n "放行\|放宽\|narrow\|通配符"` 在 `.claude/agent-common.md` 全篇零命中。

### 去 `.claude/rules/implementation-workflow.md` 找

`grep -n "写范围\|放行\|放宽"` 在该文件全篇**零命中**。该文件唯一相关的是「改 agent 定义与共用约束，走同一条三步」一节：
```
16:## 改 agent 定义与共用约束，走同一条三步

18:**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。门禁 71 号判这一条。

20:**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。
```
这一节管的是「改动怎么审」（程序性：要走三方），不是「该不该给这个 agent 这条写范围、该收多窄」（实体性判据）。它推得出「这条改动本身需要走一轮三方」（本轮正在补这一步），但推不出被判对象里那三条具体限制的内容。

### 去别的 agent 定义找

唯二在 `.claude/hooks/agent-write-scope.tsv` 里拿到 `crates/` 权限的 agent：

```
6:implementation-writer	crates/**	implementation-writer.md「写范围」
12:experiment-runner	crates/singlefs-harness/src/bin/e*.rs	实验装置要驱动真实 crates/ 代码时的只读 bin（2026-09-22 E156 撞上：岔路问的是「今天这份代码的性质」，另写一份独立模型答不了）。限定 e 开头加数字，碰不到 first_transaction_* 那几个生产装置；这个 bin 不许改 core/checker，由代码三方与门禁 56 号管
```

`implementation-writer.md` 开头（`grep -n "^description"` 现查）：
```
description: 实现员：按里程碑的一步或并行线的一条改 crates/，带测试并证明测试会红。只在主 agent 点名派发、并给出步号与压着的条款时用；不要自动派发。
```
它的写范围是 `crates/**`（无子路径限制），走 `implementation-workflow.md`「三步，缺一步就不算做完」全套（写代码 → 三方对抗 → checker）。这条定义本身不含「什么条件下可以把 crates/ 的一部分写权限分给另一个 agent」的判据，只是它自己拿到了写权限。

`mutation-triage.md:32`（`tools: Read, Bash`，本身不在 tsv 里）——这是判断「此前的既定分工」的关键证据，**整行抄**：
```
- 报告文件、草稿目录。不改变异表、不改源码、不改测试（补取样点、补断言由主 agent 另派：crates 那侧派 `implementation-writer`、把这份报告当输入，research 那侧派 `experiment-designer` 写重跑登记）。
```
这是在这条 W2 分支落地之前就写在仓里的分工陈述：**crates 那侧的改动派 `implementation-writer`**，没有第二条路。这句话没有被更新，也没有加一句「除了 `crates/singlefs-harness/src/bin/e*.rs` 这类只读实验 bin」这样的例外说明，与今天 `agent-write-scope.tsv:12` 造出的第二条路（`experiment-runner` 自己写）在字面上没有互相指认、也没有被合并成一句。

### 机械闸的射程覆盖了这个文件，但那是巧合式的覆盖，不是分支自带的

`.claude/gate.d/56-crates-adversarial-review.sh:20`（`grep -n` 现查）：
```
sources="$(grep -E '^crates/[^/]+/src/.*\.rs$' <<<"$changed" || true)"
```
这条正则匹配 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`（`crates/<crate>/src/*.rs`，`bin/` 只是 `src/` 下的一层子目录，符合 `.*\.rs$`）。`.claude/gate.d/stage-owners.tsv:37`：
```
56-crates-adversarial-review.sh	gate-triage	判决文件由主 agent 写，提交前核点名
```
即：这个新文件要被提交，必须先有一份本轮或后续轮次新写的 `research/prompts/*-main-verification.md` 按路径点名它——但这份职责归 `gate-triage`（主 agent 收尾时），不归 `experiment-runner`。`experiment-runner.md:27` 的括注只提了「变异表……归门禁 59 号跑」，只字未提门禁 56 号／72 号也会盯着这同一个文件。这一层覆盖能不能兜住，靠的是 56 号正则写得宽（对任何 `crates/*/src/*.rs` 一视同仁），不是这条 W2 分支自己声明的限制——分支文本对这件事沉默。

### 结论

**推不出**。`.claude/agent-common.md` 与 `.claude/rules/implementation-workflow.md` 都没有一句话规定「什么条件下可以把 `crates/` 的写权限切一小块给非 `implementation-writer` 的 agent，切多窄才算够」；`mutation-triage.md:32` 记录的既有分工（crates 侧一律派 `implementation-writer`）没有被这条改动回头改一句、也没有被指认。被判对象里那三条具体限制（glob 限定 e 开头、不许改 core/checker、变异表走 59 号）全部是这次改动自己声明的，不是从这三份文件里的既有条款推出来的。

**缺的是哪一句**：`implementation-first.md` 或 `implementation-workflow.md` 里缺一条通用规则，形如「非 `implementation-writer` 的 agent 要拿到 `crates/` 下的写路径时，必须满足：① 该路径的改动同样受门禁 56/59 号约束（与 implementation-writer 一致）；② glob 的射程不得越过实际会被建的文件族；③ `mutation-triage.md` 一类记录既有分工的文本要同步更新、点名这条新例外」——今天三份文件都没有这一句，`mutation-triage.md:32` 也没被改成承认这个例外存在。

**射程判断（有材料支持的部分）**：`e*.rs` 这个通配符相对今天 `crates/singlefs-harness/src/bin/` 目录里的实际文件（`ls` 现查）——`e156_allocation_basis_counts.rs`、`first_transaction_device_log_check.rs`、`first_transaction_on_device.rs`、`first_transaction_region_bytes.rs`——确实碰不到三个 `first_transaction_*` 文件，`tsv:12` 「碰不到 first_transaction_* 那几个生产装置」这句在今天的目录快照下成立。但这只是对**今天这四个文件**成立的一次性观测，不是规则；`e*.rs` 本身不排除未来有人把一个生产 bin 取名 `evict_stale_blocks.rs` 这类以 e 开头的名字放进同一目录——届时同一条通配符会放行到它。这一层风险，`agent-common.md`／`implementation-workflow.md` 都没有一句话去挡（例如「新建 bin 之前查一遍目标目录里已有文件是否会被同一条 glob 命中」）。

**什么现象会推翻「推不出」**：如果日后在 `.claude/agent-common.md` 或 `.claude/rules/implementation-workflow.md`（或另一份既有 agent 定义）里找到一句明写「narrow-scope 写权限授予的条件是……」并且这条 W2 分支的三条限制能对上那句话的字面要求，这个判定就要改判「推得出」。

## W3　什么时候该用哪一支——判定：**推不出**

### 触发条件写在哪、写的是什么（被判对象）

只有 `.claude/agents/experiment-runner.md:27` 写了触发条件（整段已在 W2 抄过，这里只重引触发短语）：
```
⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**
```
触发条件挂在「登记怎么写」上，但**写登记的是 `experiment-designer`，不是 `experiment-runner`**——决定权按这句话确实落在 designer 一侧（runner 只能照登记的字面分支走，登记里第 13 行「照登记做，不改判据」）。

### 去 `experiment-designer.md` 找「入库装置」怎么判——零命中

```
入库: (零命中)
harness: (零命中；grep 到的唯一一行是子串匹配 .claude/kb/vm-harness.md 文件名，与 crates/singlefs-harness 无关)
```
（`grep -n "入库" .claude/agents/experiment-designer.md`、`grep -n "harness" .claude/agents/experiment-designer.md` 现查，2026-09-23。）

即：**决定权名义上在 `experiment-designer`，但这份定义自己的正文里，「入库装置」这四个字一次都没出现**。设计员的定义没有一句话告诉它：什么情况下该把「装置放在哪」这一格写成「入库装置」，「入库装置」四个字要满足什么条件才算数（是不是任何用到 `singlefs-core`/`singlefs-format` 真类型的模型都算？是不是必须是只读 bin？是不是必须放在 `crates/singlefs-harness` 下？），以及这一格默认该写什么。

### 全仓找「入库装置」的定义——没有，只有用例

`grep -rn "入库装置" .claude/ research/` 现查（2026-09-23）命中的文件：`.claude/kb/experiments.md`、`.claude/kb/experiments-history.md`、`.claude/kb/experiments/151-用户数据落点的到达序与容器臂.md`、`research/prompts/e156-preregistration.md`——全部是**使用**（某次实验的登记或记录写了「装置建在入库装置上」），没有一处是**定义**（写「入库装置」是什么、由什么条件判定）。`.claude/main-agent.md` 同样零命中。

### 现存的唯一一次分叉处理——不是 `experiment-designer.md` 给的规则，是这份登记自己临场做的选择

`research/prompts/e156-preregistration.md:42`（整行抄，`grep -nF` 现查命中该行）：
```
| **装置放在哪** | 装置建在**入库装置**上：`crates/singlefs-harness` 新加一个只读 bin（`e156_alloc_basis_counts`），驱动真实 mkfs / 发布 / 崩溃注入 / 回退，读真实镜像、调真实 `singlefs-checker::walk::check_pool_image`；**不改 `crates/singlefs-core`、`crates/singlefs-checker` 的生产代码**，对面那条臂只作旁路的谓词评估（见「五」）。产物按 `E7RESULT` 行写，复跑进 `research/scripts/replay.sh`（先例：E142 已经从 replay.sh 里 `cargo run -p singlefs-harness --bin …`，`replay.sh` 第 370–378 行），变异进 `crates/mutations.tsv`（门禁 59 号） | 岔路单出处行逐字要求「在入库装置上量出代价」；岔路 7 问的「今天的检查判绿」与岔路 3 问的「X8-A 那一格卡在哪」都是 `crates/` 今天这份代码的性质，另写一份模型重实现答不了。岔路单「形态照 E109、E110」按**形态**读（纯算术、钉绝对值断言、变异表、进 replay.sh），不按位置读 | 另建 `research/` 下的独立计数模型。⚠️ 这一格交主 agent 认：两种读法给的装置不是同一个东西，主 agent 认下之前执行员不动手 |
```
以及 `research/prompts/e156-preregistration.md:5`：
```
**装置放在哪：主 agent 2026-09-22 认下「入库装置」这一读法**（「一、读法写死」第 1 行、「十一」S5），设计员写的那一格不动。
```
这两行合起来说明：这次唯一的实例里，`experiment-designer` 没有独立判定「该不该用入库装置」——它把「装置放在哪」写成**两种读法并列**，标注「交主 agent 认」，实际拍板的是**主 agent**，不是设计员自己。（这一段只作为「决定权今天实际落在谁手上」的事实核对，不作为「这一支该不该留」的推导依据，也不引用「E156 已经这么用了」去支撑分支本身站得住。）

### 能不能推出「说不清时默认走哪一支、谁来判」

`experiment-designer.md` 里唯一一条关于「有歧义时怎么办」的通用条款，在「输入」一节：
```
要回答的问题（一种读法；有两种读法的先交回主 agent 定问法。两种读法都报、又不改变任何判据时，可以两种都登记，文件头挂一句「问法待主 agent 在装置写之前删一种」；主 agent 删完再派执行员，执行员见到这句不开工）。
```
这一条管的是「要回答的问题」本身的读法有歧义，字面上不是「装置放在哪」这一格。E156 的登记把这条通用处理方式**挪用**到了「装置放在哪」这一格上（两种读法并列、交主 agent 认），但 `experiment-designer.md` 没有一句话说「装置放在哪」这一格也适用这条通用条款——这是登记撰写时的一次类推，不是定义里写死的规则。

### 结论

**推不出**。触发条件（「跑前登记把装置写死成『入库装置』」）本身说不清：
1. 「入库装置」没有在任何一份 kb 或 agent 定义里被赋予判定条件，`experiment-designer.md` 正文对这四个字的存在毫不知情；
2. 名义上的决定权在 `experiment-designer`，但它没有被给出任何判据去做这个决定；
3. 说不清时默认走哪一支——两份定义（`experiment-runner.md`、`experiment-designer.md`）都没有一句「不确定时默认走 `research/e7-index-bench`」或反过来的默认值；
4. 谁来判——`experiment-designer.md`「输入」一节的歧义条款字面上管的是另一个字段（问题读法），没有一句话把它接到「装置放在哪」这一格上；E156 那次由主 agent 拍板，是这条通用条款被类推套用的结果，不是这份定义自己写明的规则。

**缺的是哪一句、该写成什么**：`experiment-designer.md` 需要新增一条（建议放在「做什么」第 3 步「装置与 `crates/` 实现对不上时的停机条款」附近，或单列一条）：「『装置放在哪』这一格，满足〔具体条件，例如：岔路单里某一行的措辞要求量出的代价必须来自 `crates/` 今天这份代码本身、且另写独立模型会漏掉这份代码的某个具体性质〕时才写『入库装置』；拿不准时按『输入』一节的歧义条款处理：两种读法并列，交主 agent 认，不擅自选。」——今天两份定义里都没有这一句。

**什么现象会推翻这个判定**：如果日后在 `experiment-designer.md` 里加了一句写明「入库装置」判定条件与默认值，或者在 `.claude/kb/` 里找到一条决策明确给出这个判据、且 `experiment-designer.md` 用「开工先读」指了过去，这一格就要改判「推得出」。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W2（写范围放宽到 `crates/`） | 推不出 | `agent-common.md`、`implementation-workflow.md` 只讲写范围闸怎么执行，不讲该给多宽；三条具体限制（e 开头、不许改 core/checker、走 59 号）全是被判对象自己声明的，且没有回头改 `mutation-triage.md:32`「crates 那侧派 implementation-writer」这条既有分工的文字 |
| W3（什么时候该用哪一支） | 推不出 | 触发条件挂在写登记的一方（`experiment-designer`），但该定义正文里「入库装置」四个字零命中，没有判定条件、没有默认值、也没有把「装置放在哪」接到自己已有的歧义处理条款上 |

## 没做什么（固定会有的）

- 不判 W1（入库装置与「不与实现共用代码」）：那是云端攻方（Opus）的格，本报告只在「现场事实」一节记了一句 `first_transaction_*.rs` 三个文件正被别的会话改动、以及门禁 56 号正则覆盖 `crates/singlefs-harness/src/bin/*.rs` 的事实，供 Opus 判 W1 时参考，不构成 W1 的判定。
- 不判本地攻方那张三格形式判的表（那是本地腿的格）。
- 不采纳或出这一轮的最终判决（判决由主 agent 做，`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判决由主 agent 做，不由投票做」）。
- 没有跑门禁全量、没有编译 `crates/`（本轮任务不要求，且 `crates/singlefs-harness/src/bin/first_transaction_*.rs` 正被别的会话改动中，编译结果此刻不稳定，不强行跑）。
- 没有对 `research/prompts/e156-preregistration.md` 之外的其它实验登记做穷举式核对「入库装置」用例——已核过的三处 kb 记录 + 该登记，足以支持「全仓无定义、只有用例」这一结论；没有再扩大搜索范围到已归档的历史提交（`git log --all` 里可能还有更早的用例，但不改变「今天的 `experiment-designer.md` 正文里没有定义」这一判定）。

