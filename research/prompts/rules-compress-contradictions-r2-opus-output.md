# 攻方腿：rules-compress-contradictions-r2（第二轮，攻修补与没看过的地方）

日期：2026-11-16。立场：攻方，假设「第二版里改过与新补的条目都落对、标只改措辞的没动做法」是错的。这一腿没写模型，没有复跑命令与模型文件；引的数都是这一次现跑命令量的，量法写在各节。

开工核过：原件 55 份照 `research/prompts/rules-compress-contradictions-r2-snapshot/judged-sha256.txt` 逐份 `sha256sum -c`，第二版合稿树照 `merged-tree-sha256.txt` 逐份核，都没有不 OK 的行（量的）。第一版合稿树 `research/prompts/rules-compress-r1-merged/` 照第一轮的 `merged-tree-sha256.txt` 同样全 OK（量的），拿它做「第一版」的基准。

## 各格判定一览

点名的 24 条（验证-9、入口-4、常驻-7、常驻-8、常驻-11、共用-10、合稿-1、合稿-4 到合稿-12、用户定-13 到用户定-20），加格三逐条核出的两条：

| 条目 | 判定 | 一句话 |
|---|---|---|
| 验证-9 | 改得对 | 与 `scripts/gate/admission.py` 文件头和判法函数都对上 |
| 入口-4 | 改得对 | 没照第一轮判决「去掉只有」，而是把「只有」收在 `crates/` 与 kb；原件 `main-agent.md` 第 7 行本来就写「其余按下面那张表派」，收窄后的写法比去掉更贴原件 |
| 常驻-7 | 改得对 | |
| 常驻-8 | 改得对 | 弹窗几次没定，单列核不动 |
| 常驻-11 | 改得对 | 落的是用户定案第 21 条；总表这一行的说明与行号过时（见格二 B） |
| 共用-10 | 改得对 | |
| 合稿-1 | 改得对 | 与用户定案第 17 条同义 |
| 合稿-4 | 改得对 | |
| 合稿-5 | 改得对 | 留一处二义：脚本与钩子的测试不在两张表里（见格二 D） |
| 合稿-6 | 改得对 | |
| 合稿-7 | 改得对 | |
| 合稿-8 | 改得对 | 四处都对过脚本文件头 |
| 合稿-9 | 改得对 | |
| 合稿-10 | 改得对（已登「改了判据」） | 新写的「读全」读得出两种意思（见格二 D） |
| 合稿-11 | 改得对 | |
| 合稿-12 | 改得对 | 第一版树里 43 处上游节指针，第二版 0 处（量的） |
| 用户定-13 | 改得对 | 已撤回，规则侧没有剩下的句子 |
| 用户定-14 | 改得对 | 落点行号写偏一行（见格二 B） |
| 用户定-15 | 改得对 | |
| 用户定-16 | 改得对（没改） | 原件栏与合稿栏的行号对调（见格二 B） |
| 用户定-17 | 改得对 | 原件栏引的是合稿树的行（见格二 B） |
| 用户定-18 | 改得对 | 点名的五处都指过去了；用户要求的那一趟弹几次窗没定，单列核不动 |
| 用户定-19 | 改错了（落得不全） | 调查员「查到就修」写明在主工作区改，同一份定义第 1 步却写死 `--cwd` 填副本，照做是在没有修补的副本上编与测 |
| 用户定-20 | 其实改了判据 | 工具员定义第 5 步原件不许对进程组发信号，第二版许对自己起的那一组发；总表标「改回原判据」。另有一处与钩子不一致的二义，单列核不动 |
| 常驻-10（格三） | 其实改了判据 | 标「只改措辞」，规则比原件多要求 `--wait`、同一层紧跟 `名字=$!`，也比钩子 ⑥ 认的写法严 |
| 常驻-12（格三） | 改得对 | 新并到一起的两句读出一处小二义（见格二 D） |

格二另有三件不挂在单条上的（各节细说）：

- A. 用户定案第 21、22 条没进总表；第 22 条落在 `agent-common.md` 第 68 行的改动不在 `diff-from-r1.patch` 里（量的：把补丁打到第一版树的拷贝上，与第二版树比，只差这一行）。
- B. 总表里有 9 处落点行号随第二版的修补偏了一到四行，另有 6 行（用户定-2、16 到 20）把原件栏与合稿栏写反或指到第二版里已不在的原文；表头写「原话与合稿片段都由 `tools/render_contradictions.py` 逐条回原件与合稿树核过（退 0、problems=0）」，对第二版不成立。
- C. 第二版总表里 4 处〔〕不是合稿原文而是转述（合稿-1、合稿-9、合稿-10、用户定-4）。

## 格一、格二：点名条目逐条

下面引合稿树写 `research/prompts/rules-compress-r2-merged/dot-claude/…` 的全路径，原件写仓里的路径。

### 验证-9：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:146`：「这一行没有 `exhaustive=` 字段照判红」。
- 脚本文件头：`scripts/gate/admission.py:164`：「exhaustive=false 不判、成功句写缩过域、不算门禁通过，这一行没有 exhaustive= 字段照判红」。
- 判法函数也是这样（只看文件头不够，这一次读到了代码）：`scripts/gate/admission.py:3326`：「if exhaustive == "true" or (narrowing and exhaustive == "false")」，不满足就记一条问题；字段缺失时 `get` 得 None，两种分支都进不去，照判红。
- 格二：全合稿树搜 `exhaustive`，只有参考件这一行讲这条判据（量的：`grep -n exhaustive` 打在合稿树的 `verification-reference.md`、`verification.md`、`implementation-workflow.md`、两份崩溃技能上，只命中第 146 行），没有住两处。
- 什么现象会推翻：`admission.py` 在别处另有一支对缩域那一趟连 `exhaustive=true` 也判红，或对没缩域的那一趟放过 `exhaustive=false`。

### 入口-4：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:9`：「主 agent 可以自己写的只有两类」，同一句开头把射程限在「`crates/` 与 kb 里」，别的文件指共用约束「派发」一节。
- 第一轮判决写的是去掉「只有」（`research/prompts/rules-compress-contradictions-r1-main-verification.md:17`），第二版没照这句做，改成收窄射程。我核了这个取舍站不站得住：原件同一行本来就把「其余」交给派发表，`.claude/main-agent.md:7`：「收尾报进度留在主 agent，其余按下面那张表派」，两类可以自己写是这句的例外；派发表里 `crates/` 的有界改动（含「修原因已知的红」，原件第 117 行第一栏）派实现员。所以在 `crates/` 与 kb 里，原件本来就是「其余派、两类例外」，第二版的「只有两类」与它同义；第一轮攻方腿那一条只读了「可以自己写」，没读到同一行的「其余按下面那张表派」。
- 原件里与它撞的那一句仍在：`.claude/agent-common.md:8`：「简单任务不派发，自己直接改」。第二版把它的射程分开写（`crates/` 与 kb 照派发表，别的文件照这一句），是在两句原话之间取了一边，标「改了判据（已收窄）」是诚实的。
- 什么现象会推翻：原件别处有一句许主 agent 自己写 `crates/` 或 kb 里不属于这两类的东西（我搜了原件 `main-agent.md` 里「主 agent 自己」「自己写」「自己改」，命中第 7、15、33、38、48、57、94、116、119、120、121、124、129 行，逐行读过，许自己写的只有这两类与阶段同步那一行的「自己改或交 `kb-scribe`」，后者第二版的括注点名了那一行）。

### 常驻-7：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/writing-discipline-by-review.md:12`：「历史放不放、放在哪照各自的规则」，括注点 `rules-discipline.md` 第 5 条与 `changelog-format.md`「只有决策与实验有历史，别的文件没有」。
- 与登记表一致：`.claude/history-carriers:2`：「没登记的 kb 文件不留历史节」。设计文档那一类括注没点名，但同一张表第 8 行仍指 `design-doc-discipline.md`，它的第 1 条就是「历史一律放到文末」，照「各自的规则」读得到。

### 常驻-8：改得对（弹窗几次单列核不动）

- 原件那条路：`.claude/rules/session-wrapup.md:54`：「分不清的时候跑」（后接 `bash .claude/scripts/gate.sh --staged`）。
- 第二版留着这条路：`research/prompts/rules-compress-r2-merged/dot-claude/rules/session-wrapup.md:37`：「同意了才跑 `gate.sh --staged`」，起法改成经服务交 `scripts/gate/gate-staged.sh`（项目代码闸只认经 submit 起，原件那个直接起的写法今天会被拒，这一处改得对）。
- 范围声明闸不拦 `gate-staged.sh` 不带标识地交：`.claude/hooks/run-scope-guard.sh:47`：「gate-staged.sh、gate.sh、replay.sh 不带实验号的那一形」写在「不判」里。所以照第二版先弹窗、再照提交技能带 `--full-gate` 交，会弹两次；不带 `--full-gate` 交只弹一次。见「核不动」第 1 件。

### 常驻-11：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:18`：「报告的人或 agent 可以终止它」。
- 原件：`.claude/rules/command-safety-by-review.md:22`：「报告的一方（人、agent 或检出的工具）可以终止它」。
- 用户定案：`records/2026-11-16-三方豁免取消与存量补轮.md:199`：「调度的一方不响应时由报告的人或 agent 终止并报出来」。第二版与它做法相同。
- 格二：总表这一行（`research/prompts/rules-compress-r2-merged/contradictions.md:46`）说明栏末尾仍写「是否放回交用户定」，用户已在第 21 条定了；落点行号写 `:17`，第二版那一句在第 18 行。见格二 B。

### 共用-10：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:109`：「实现员一律在副本里改、交补丁（只派一个也这样」，同一句写明调查员「查到就修」那一支是例外。
- 原件是条件句：`.claude/agent-common.md:114`：「在副本里改、交补丁的，`<根>` 写 `<快照>`」。第二版把条件换成「实现员一律」，射程从「凡在副本里改、交补丁的子 agent」缩到实现员；三类要写命名检查的子 agent 里，执行员在副本里改完是整份换进主工作区、不交补丁（`research/prompts/rules-compress-r2-merged/dot-claude/agents/experiment-runner.md:38`：「先在草稿目录的副本里改，编过再整份换进主工作区」），工具员在主工作区改，缩了也没有谁落空。
- 钩子文件头那一句仍指一句共用约束里不在的话：`.claude/hooks/handback-guard.sh:45`：「共用约束写明 rsync 拷的副本 <根> 填仓根」。原件与第二版都没有这一句，不是这一版引出的；总表已写交主 agent。

### 合稿-1：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/implementation-workflow.md:10`：「打中的，主 agent 现查坐实、做成会红的测试后写回代码，改完再攻一轮，那一轮没新打中就收，最多三轮」。
- 用户定案：`records/2026-11-16-三方豁免取消与存量补轮.md:169`：「主 agent 现查坐实、做成会红的测试后写回代码，改完再攻一轮，那一轮没新打中就收，最多三轮」。逐字相同。
- 另一处指回来：`research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:43`：「代码轮的轮数照」。派发表「要推论」一行仍写「开不开第二、三轮照同一份」多轮那一节（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:90`：「开不开第二、三轮照同一份」），这一行第一栏含「实现改动的对抗」；照指针走到多轮那一节，第一句就把代码轮指回 `implementation-workflow.md`，读得到对的轮数，不算二义。

### 合稿-4：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:67`：「doc-lint:read-once verification-reference.md」，上一行写明到点名那一步才读。
- 门禁那一格只认两种登记：`.claude/gate.d/code-tooling.sh:26`：「项目规则文件与 CLAUDE.md 的 @ 引用加 read-once 登记逐项相等」。read-once 在原件里本来就不只表示「开会话读一次」（`CLAUDE.md:68`：「碰到对应的活之前先整份读一次」，紧接着下一行就是那一组的 read-once 登记），登进去不改它的读法。

### 合稿-5：改得对（留一处二义）

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/show-me-test-by-review.md:7`：「实验装置的进 `scripts/mutation/tables/` 下它那一张表」（`crates/` 的进 `crates/mutations.tsv`）。两张表都在仓里（`ls scripts/mutation/tables | wc -l` 得 209，量的）。
- 二义见格二 D 第 1 件：脚本与钩子的测试证红走弄坏开关，不进这两张表；同一棵树里 `research/prompts/rules-compress-r2-merged/dot-claude/rules/test-script-discipline.md:40`：「每一处会判红的分支配一个弄坏开关或一条变异」。

### 合稿-6：改得对

- 「两处说同一件事以上游为准」在第二版合稿树全树 0 命中（`grep -rn 以上游为准`，量的）。
- 第二版第 4 行新写了一句说明：`research/prompts/rules-compress-r2-merged/dot-claude/rules/code-discipline-by-review.md:4`：「只写上游没写的」。原件没有这句，它是对这份文件的描述、不带判据，不算改做法；读成「不许写上游写过的」也与原件逐条审过的那张表不冲突。

### 合稿-7：改得对

- 放回的几条都在第二版 `verification.md`「测试不分档，红了只重跑红的」与「门禁的结构」两节里（第 23 到 27 行、第 99 行）。模块声明那一条与脚本同义：`scripts/crash/crash-case-check.py:7`：「验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块（门禁」。
- 一处措辞：`research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:23`：「登记 `crash-case:`（登记进崩溃点重放合表」，括号里接着写「点名 `crash-case:` 行或放量分项」，前半说只登 `crash-case:`、括号说放量分项也算；原件那一行只写登记进合表、两种都算。读括号能读对，不算改做法。

### 合稿-8：改得对

四处逐处对了脚本：

| 第二版参考件 | 脚本 | 对得上 |
|---|---|---|
| `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:116`：「门禁 `crash-point-replay` 不 source 它跑格」 | `.claude/gate.d/crash-point-replay.sh:237`：「不用共用库 lib/stage-cells.sh 跑格」 | 对（文件头同一句另写只 source 它取一个函数，参考件没写这半句，不影响做法） |
| `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:130`：「与挂特性的测试文件」 | `.claude/gate.d/crash-point-replay.sh:40`：「--item 点名项。项有两类」 | 对 |
| `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:106`：「第三列不是 prune-server 的只许是」 | `.claude/gate.d/verdict-library-scripts.tsv:4`：「射程里扫到、只是提到别人的复用的写」 | 对（同一文件头第 3 行写「无判定库」那一种） |
| `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:18`：「拆成计划、各片、合并」 | `.claude/gate.d/crash-point-replay.sh:135`：「拆成计划、各片、合并」 | 对 |

### 合稿-9：改得对

- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:5`：「压缩过上下文之后再做这两件事之前同样重读」。补的是第一版漏的那一半（压缩之后技能正文不在），没有别的连带。

### 合稿-10：改得对（总表已登「改了判据」，留一处二义）

- 原件：`.claude/agent-common.md:22`：「照样读全那份文件」——规则文件点到就整份读。
- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:17`：「做那一步之前读那一节，读全」。步骤里点名一节的规则，从读整份改成读那一节，这是做法变化，总表已登。
- 二义见格二 D 第 2 件。

### 合稿-11：改得对

- 三处「开工另读」都在：第二版 `agent-common.md` 第 28、29 行，`tooling-writer.md` 第 16 行（`grep -nF 开工另读`，量的）。原件只写指针，第二版补了读取要求，总表已登「改了判据（补登）」。

### 合稿-12：改得对

- 量的：在第一版合稿树 `dot-claude/` 与 `CLAUDE-merged.md` 里搜「上游规则文件名加反引号紧跟「」的写法，43 处；第二版同一条命令 0 处；第二版写成「文件名加（要点）」的 46 处。
- 删掉的 43 处按节名数（`diff -ru` 两版 `dot-claude/` 的删行里取）：出现最多的是 `show-me-test.md`「门禁不许假装通过」与 `kb-discipline.md`「2. 每条带出处与状态」各 4 处。抽读了其中 `evidence-discipline.md` 第 32、61、81 行与 `prior-art.md` 第 16 行，括号里留着原节名当要点，项目一侧的判据句在同一行，没有丢。
- 余下的风险不是改错：括号里仍是上游此刻的节名，上游改了节名之后这些括号就只是要点、不再对得上节标题；这正是这一改的本意。

### 用户定-13：改得对

- 第二版合稿树里「逐个按进程号」「按会话号」0 命中（`grep -rn` 打在 `dot-claude/`、`CLAUDE-merged.md`、`references.md` 上，量的）。`merge-result.md` 里第一版的说法还在，文件开头加了修稿注说明已撤回，不算落点。

### 用户定-14：改得对（行号偏一行）

- 规则一侧：`research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:40`：「没给时一律取登记的每台默认重活线程数」。全树 `nproc` 只剩这一句里的否定（量的）。
- 总表这一行与用户定-3、常驻-13 都写落在第 39 行，第二版第 39 行是「不并行的三种」。见格二 B。

### 用户定-15、用户定-16：改得对

- 第 15 条只排次序，`references.md` 第三节（「挪进技能与参考件的细则：按旧节名引、最好改指的」）登了出路句随规则一起换，没有规则文字落点。
- 第 16 条脚本常量今天仍是旧节名：`scripts/gate/apply-writer-patch.py` 第 272 行 `CHECKER_SECTION` 的值含「层 0」（`grep -n` 量的），定义照旧写同一个节名是对的；这一行原件栏写的 `implementation-writer.md:51` 在原件里是第 41 行，第 51 行是合稿树的，见格二 B。

### 用户定-17：改得对

- 同合稿-1，逐字照用户定案。总表这一行原件栏引 `.claude/rules/three-way-inference.md:43`、`:70`，原件这两行一行是「凡是引 kb 的条目」、一行是空行，引的其实是合稿树的行。见格二 B。

### 用户定-18：改得对（弹窗几次单列核不动）

- 点名的五处都指过去了：合稿树 `main-agent.md:26`、`agent-common.md:59`、`skills/crash-test/SKILL.md:12`、`rules/session-wrapup.md:37`、`rules/implementation-workflow.md:51`（逐行现读，五处都写「照 … 「提交之外」那一条」）。
- 判据句：`research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:38`：「提交时那一趟的确认就是带 `--full-gate` 的那一次弹窗」——只说提交时那一趟；提交之外那一趟另有「先弹窗问用户」。
- 没指过去的三处仍只写「用户要求时」跑：`research/prompts/rules-compress-r2-merged/dot-claude/skills/gate/SKILL.md:10`：「与用户要求时」，`research/prompts/rules-compress-r2-merged/dot-claude/skills/crash-test/commit-stages.md:21`：「只在提交时（或用户要求时）跑这六道」，`staged-gate.md` 第 8、16 行同。它们没说不弹窗，与第 18 条不冲突，但照它们做的人读不到「先弹窗」那一句。加上 `--full-gate` 本身弹一次，用户要求的那一趟是一次还是两次弹窗，见「核不动」第 1 件。

### 用户定-19：改错了（落得不全）

- 用户定案：调查员「查到就修」照旧在主工作区改，写明它是例外。第二版写了：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:109`：「那一支是例外，照旧在主工作区改」；`research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:33`：「在主工作区只改」。
- 同一份定义里，这一步要跑的命令照第 1 步写：`research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:33`：「照第 1 步写进报告」；第 1 步是：`research/prompts/rules-compress-r2-merged/dot-claude/agents/investigator.md:28`：「（`--cwd` 填副本）」。
- 照字面做出来的历史：调查员在主工作区改 `crates/` 某文件 → 报告里 clippy、build、动到的测试目标、带 `--item` 的门禁都写 `--cwd <草稿目录>/copy` → 主 agent 照原样交 → 编的、测的是第 2 步复现用的副本，那里没有修补 → 绿了也不说明修补对。
- 原件第 1 步是可选的：`.claude/agents/investigator.md:28`：「写成 `task-server/task submit [--cwd <副本>] -- <命令>` 的整条」，方括号里不给 `--cwd` 就是此刻的目录（主工作区），与「查到就修在主工作区改」不撞。第一版压缩把它写死成「填副本」，第二版落第 19 条时没回头改这一句。
- 该是什么：第 1 步写成「复现、二分、探针的命令 `--cwd` 填副本；第 6 步查到就修的命令 `--cwd` 填仓根（改动在主工作区）」，或第 6 步自己写明 `--cwd` 填仓根、命名检查 `<根>` 填仓根。
- 四句：分辨得出「落对」与「落不全」（照做的产物不同：一边在有修补的树上编测、一边在没有的树上）；读定义的调查员在写命令那一刻只看得到「填副本」这一句，看不到要换；满足格二「修补之后…读不读得出两种意思」与格一「落的地方读不读得通」两句里的后者；第一轮判决没给这一处改法。
- 这一处也可能归定义那一题（第一版压缩把可选写成必填）；我只核了这一份定义的第 1、6 步与原件第 1 步，没核别的定义有没有借这一步。

### 用户定-20：其实改了判据（另有一处二义单列核不动）

- 总表标「改回原判据（第 6、13 条撤回，规则照仓里原件的做法恢复）」。规则那一节与共用约束那一条确实回到原件的「放进它自己的进程组」，但点名的落点里有一处原件是禁止的：
  - 原件：`.claude/agents/tooling-writer.md:33`：「不按名字、cgroup 或进程组发」。
  - 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/agents/tooling-writer.md:33`：「自己写的装置与脚本里只对自己用新会话起的那一个子进程的组发」。
  - 用户定案点了这一步：`records/2026-11-16-三方豁免取消与存量补轮.md:194`：「工具员定义第 5 步照这个写清」。所以这是用户定的判据变化，落得对，标签该是「改了判据（用户定 20）」，不是「改回原判据」。共用约束 ⑥ 的放行行同理：原件 ⑥ 只放行点名一个（`kill "$!"`、`kill %1`），第二版补了对自己新会话子进程的组发信号，这与钩子文件头一致（`.claude/hooks/bash-command-detector.sh:185`：「shell 里同一层先 `setsid [--wait] <命令> &`（不带 -f）」），对共用约束是新加的放行。
- 二义（格二）：第二版两处都写不在循环里发——`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:67`：「不对别的进程组发、不在循环里逐个发」，`research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:9`：「不在循环里逐个发，不先挑进程再发」。钩子对 python 这一边放行循环里给自己起的子进程的组发信号：`.claude/hooks/bash-command-detector.sh:4114`：「在循环里调也放行（发的是那一个子进程的组）」；shell 这一边循环里照拒：`.claude/hooks/bash-command-detector.sh:4009`：「在 for / while / until 循环里逐个发：这是批量停，不是点名停一个」。
  - 读出两种意思的历史：python 装置 `for 样本 in 样本表:` 里每一轮 `Popen(..., start_new_session=True)`、超时 `os.killpg(子进程.pid, SIGTERM)`。钩子放行；照共用约束第 67 行字面读是「在循环里逐个发」，不许。规则那一节同一行末尾写「以钩子 … ⑥ 为准」，照它读是许；共用约束那一条没有这半句。
  - 用户定案原话两样都说了：「维持钩子现在的判法」与「别的组、循环逐个发、先挑再发一律拒」（`records/2026-11-16-三方豁免取消与存量补轮.md:194`：「别的组、循环逐个发、先挑再发一律拒」）。每条样本在自己那一轮里停自己起的那一组，算不算「循环逐个发」，原话判不了，见「核不动」第 2 件。
- 落点栏：这一行「合稿里落在哪」写的是钩子文件头行号，第二版落点写在了原件栏，见格二 B。

## 格三：标「只改措辞」的 37 行逐条核

条数由命令数出：`grep -c '| 只改措辞 |' research/prompts/rules-compress-r2-merged/contradictions.md` 得 37（量的）。每行都把总表点名的原件行与第二版合稿行逐行现读、对了前后文；「做法动没动」只看读的人照做出来的东西变没变。

| 编号 | 判定 | 核了什么（现读） |
|---|---|---|
| 入口-2 | 没动做法 | 原件 `main-agent.md` 第 57 行「退 3 用户不同意、这一批放弃、任务被停这三支同样删」与第二版 merge-verify 技能第 19 行逐句对，连同被停那一趟照样留快照、删前先 cancel 都在 |
| 入口-3 | 没动做法 | 第二版 merge-verify 技能第 18 行写「另跑 fmt、clippy、build」，与 `scripts/gate/merge-and-verify.sh` 文件头第 9 行同 |
| 入口-5 | 没动做法 | 第二版 dispatch 技能第 26 行「读工作根所在文件系统的可用量」；`scripts/agents/watch.conf` 第 23 行 `root-free-gib-alert` 判的就是工作根所在文件系统 |
| 入口-7 | 没动做法 | 拆句；`ls -d crates/*/` 得 11（量的；`ls crates` 得 12，多的那一项是 `mutations.tsv`） |
| 入口-8 | 没动做法 | 只补劝阻句 |
| 入口-11 | 没动做法 | 清单射程句两处都在 |
| 验证-6 | 没动做法 | 原件 `verification.md` 第 55 行把门禁按需领卡那一趟另写在同一行末尾；第二版参考件第 70 到 72 行拆成三条，起跑就分卡的与按需领卡的各一条，与 `.claude/gate.d/crash-point-replay.sh` 文件头「显卡与清场」一段同 |
| 验证-8 | 没动做法 | 不写哪一方拒，做法同 |
| 常驻-9 | 没动做法 | 第二版 `timer-only-remind.md` 第 21 行把「不在拒绝范围」限定到定时闸、补一句 ② 另拒；与钩子 ② 的现行判法一致 |
| 常驻-10 | **其实改了判据** | 见下面小节 |
| 常驻-12 | 没动做法 | 原件第 30 行「长活放后台，定期回来复检」与共用约束「结束本轮等完成通知、不轮询」本来就撞，第二版取后者，与钩子 ② 一致；并句之后读出一处小二义，见格二 D 第 3 件 |
| 常驻-14 | 没动做法 | 第二版第 31 行补「主 agent 自己就是调度的一方」，指 `main-agent.md` 第 15 条，第 15 条原件就写找不到会话就自己做 |
| 常驻-15 | 没动做法 | 交回只写路径、`sha256sum` 与判定一览，两处同 |
| 常驻-17 | 没动做法 | 表头当数据的那张二格表写成一句，「能就值得，不能就不值得」与原件同义 |
| 常驻-18 | 没动做法 | 自称换文件名 |
| 共用-1 | 没动做法 | 写范围表 `.claude/hooks/agent-write-scope.tsv` 里落在 `.claude/agents|hooks|rules|settings` 下的行只有 `tooling-writer` 的四行（量的）；`kb-scribe` 改 `.claude/rules/*.md` 只经 `relabel-item.py`，与它定义第 43 行同 |
| 共用-2 | 没动做法 | 「开工先读」点名的别的文件只读点名小节、别的文件 1,000 行以内整读，两条射程分开，原话都在 |
| 共用-6 | 没动做法 | 第二版逐个列的写入脚本与 `minimal-copy.py` 都在 `.claude/hooks/lib_project_code_runs.py` 第 1076 行起的 `TEXT_TOOLS_RUN_DIRECTLY` 里（现读） |
| 共用-7 | 没动做法 | `.claude/hooks/write-guard.sh` 文件头第五道写明仓外不判，第二版「写进仓里的，写闸当场拒」与它同 |
| 共用-8 | 没动做法 | 第二版第 108 行 |
| 共用-9 | 没动做法 | 与 `.claude/hooks/completion-cleanup-guard.sh` 文件头第 104 行「只认认领目录与草稿目录里单子上列出的那几处」同；第二版顺带补的「交接」那一半与同文件第 100 行同 |
| 定义-3 | 没动做法 | 第二版执行员定义第 46 行仍列「取值是 `not_run` 的」字段，第 47 行改的是 NOT_RUN 行的写法 |
| 定义-4 | 没动做法 | 写全了引文规矩，与正推腿定义第 29 行同句 |
| 定义-5 | 没动做法 | 「书记员」换「写回员（`kb-scribe`）」 |
| 定义-6 | 没动做法 | 两项并一项，「缺它不开工」留着 |
| 定义-8 | 没动做法 | 补「逐行盘点走第 12 步」，输入一节本来就列五种活 |
| 定义-10 | 没动做法 | 位置指代换成「第一节末尾的「未定项：」一段」 |
| 按需-M1 | 没动做法 | 原件那一行处置写「留下」、写法写「按一件能单独验证的事切，不按行数」，两栏自相矛盾；第二版把处置改成「改写」，写法不变；照写法做的人做法不变 |
| 按需-M3 | 没动做法 | 第二版 `path-moves.md` 第 12 行仍写「`.claude/gate.d/` 顶层」，与 `.claude/gate.d/lib-lag-baseline.py` 第 237 行 `GATE_STAGE_DIRECTORY` 的注释「只算顶层」同 |
| 按需-M12 | 没动做法 | 「三条」改「四条」，与共用约束同 |
| 按需-M15 | 没动做法 | 错字 |
| 按需-M16 | 没动做法 | `crontab`（`-l` 除外），与 `.claude/hooks/timer-only-remind-guard.sh` 第 17 行与判法第 119 行同 |
| 按需-M17 | 没动做法 | 同一句挪了位置 |
| 合稿-2 | 没动做法 | 两处都写明服务本体与客户端 |
| 用户定-7 | 没动做法 | 第二版共用约束第 73 行「逐条以它的文件头为准」；合规写法下沉到钩子文件头，射程归另一题 |
| 用户定-8 | 没动做法 | `fs-design.md` 两段 Rust 示意删了，判据在第二版第 91、95 行写成句子（布局身份进 incompat 位、不认识就拒挂；能不能把用错变成挂不上） |
| 用户定-11 | 没动做法 | 两类说明句都删，不带判据 |

## 其实改了判据

### 常驻-10：标「只改措辞」，规则比两处原话里宽的那一处、也比钩子更严

- 两处原话：`.claude/rules/command-safety-by-review.md:8`：「shell 里用 `setsid` 起」；`.claude/agent-common.md:72`：「shell 用 `setsid --wait` 起」。
- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:8`：「shell 里用 `setsid --wait` 起（不带 `-f`），同一层紧跟着 `名字=$!`」。
- 钩子认的写法里 `--wait` 是可选的：`.claude/hooks/bash-command-detector.sh:185`：「shell 里同一层先 `setsid [--wait] <命令> &`（不带 -f）」。
- 做出来不同的历史：装置脚本里写 `setsid ./样本 & 子进程=$!` … `kill -TERM -- -"$子进程"`。原件这份规则许，钩子 ⑥ 与门禁 code-tooling 的 research-gate-lint 格（同一份判定）放，第二版这份规则字面不许（少了 `--wait`）。同一节第 9 行又写「认得出的写法与照旧拒的写法以钩子」⑥ 为准，照它读又许——两行之间读出两种意思。
- 该是什么：标签改「改了判据」，说明取的是共用约束那一边、另加了钩子 ⑥ 要的「同一层紧跟 `名字=$!`」；第 8 行照钩子写成「`setsid [--wait]`（不带 `-f`）」，让第 8、9 两行与钩子一致；要比钩子严，就得另立一条并登成改了判据。用户定案第 20 条说的是维持钩子的判法，没说规则要比钩子严。
- 四句：分辨得出（上面那段脚本两种读法下一许一不许）；写装置的人此刻读得到第 8、9 两行，读不出该照哪一行；满足格三「没登而做法变了」；第一轮判决没给改法。

### 用户定-20：标「改回原判据」，工具员第 5 步与共用约束 ⑥ 的放行行是新判据

细节见上面「用户定-20」一节：原件 `.claude/agents/tooling-writer.md:33`：「不按名字、cgroup 或进程组发」，第二版许对自己新会话起的那一组发。该是什么：总表这一行拆开写，规则那一节与共用约束样本进程那一条「改回原判据」，工具员第 5 步与共用约束 ⑥ 放行行「改了判据（用户定 20，与钩子文件头 ⑥ 一致）」。

## 改错了

### 用户定-19：调查员查到就修的命令 `--cwd` 写死成副本

见上面「用户定-19」一节。该是什么：调查员定义第 1 步写成「复现、二分与探针的命令 `--cwd` 填副本；第 6 步查到就修的命令 `--cwd` 填仓根，命名检查的 `<根>` 填仓根」。

## 格二：修补引出的、不挂在单条上的问题

### A. 用户定案第 21、22 条没进总表；第 22 条的改动不在 `diff-from-r1.patch` 里

- 用户定案：`records/2026-11-16-三方豁免取消与存量补轮.md:199`：「调度的一方不响应时由报告的人或 agent 终止并报出来」；`records/2026-11-16-三方豁免取消与存量补轮.md:200`：「上下文吃紧与卡住都可以交回、新派承接」。
- 总表「用户定」一节的标题只写到第 20 条（`research/prompts/rules-compress-r2-merged/contradictions.md:106`），表里没有用户定-21、用户定-22 两行；第 21 条只在常驻-11 的说明里出现，而那一句还写着「是否放回交用户定」。
- 第 22 条落在：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:68`：「上下文吃紧或卡住（同一步反复做不过去）时」。补丁里同一行是没改的上下文行：`research/prompts/rules-compress-r2-merged/diff-from-r1.patch:238`：「例外只有 `experiment-runner`：上下文吃紧时照它定义第 2b 步」。
- 量的：把第一版合稿树拷进草稿目录、`patch -p1` 打上 `diff-from-r1.patch`（退 0），再 `diff -rq` 比第二版树，内容不同的只有 `dot-claude/agent-common.md`，逐行 diff 只差第 68 行这一处（另三份只在第二版里：补丁本身、`dispositions.md`、`revise-result.md`）。所以正文说的「第二版相对第一版的全部改动」差这一处；格一照补丁逐处核的人看不到它，用户定-5 那一行（总表第 116 行）也没跟着改。
- 该是什么：总表补用户定-21（落点：第二版 `command-safety-by-review.md` 第 18 行；改没改做法：改了判据，删了检出工具自己终止那一支）与用户定-22（落点：第二版 `agent-common.md` 第 68 行与执行员定义第 39 行；改了判据），常驻-11 说明删「是否放回交用户定」；补丁重出一份。

### B. 总表的行号与栏位随第二版的修补走偏

量法：草稿目录 `check_cites.py` 把总表每个带行号的 文件:行〔片段〕 拿到原件与第二版合稿树的那一行里找片段，找不到的再全文件找片段落在哪一行；同样的片段在第一版树里落在哪一行另用 `grep -nF` 核了（量的）。

| 总表行 | 那一格写的 | 第二版里片段实际在 | 第一版树里 |
|---|---|---|---|
| 验证-5（第 28 行） | `verification.md:69` | 第 73 行 | 第 69 行 |
| 常驻-1（第 36 行） | `command-safety-by-review.md:45` | 第 46 行 | 第 45 行 |
| 常驻-10（第 45 行） | `command-safety-by-review.md:9` | 第 8 行 | 第 10 行 |
| 常驻-11（第 46 行） | `command-safety-by-review.md:17` | 第 18 行 | 第 17 行 |
| 常驻-12（第 47 行） | `command-safety-by-review.md:16` | 第 17 行 | 第 16 行 |
| 常驻-13（第 48 行） | `command-safety-by-review.md:39` | 第 40 行 | 第 39 行 |
| 常驻-14（第 49 行） | `command-safety-by-review.md:30` | 第 31 行 | 第 30 行 |
| 用户定-3（第 114 行） | `command-safety-by-review.md:39` | 第 40 行 | 第 39 行 |
| 用户定-14（第 125 行） | `command-safety-by-review.md:39` | 第 40 行 | 第 39 行 |

栏位写错、或指到第二版里已经没有的原文的：

| 总表行 | 问题 |
|---|---|
| 用户定-2（第 113 行） | 合稿栏引 `implementation-workflow.md:10`〔打中的写回代码，开不开下一轮照〕，这是第一版的句子，第二版这一行已改成合稿-1 的写法，片段 0 命中（`grep -c` 量的） |
| 用户定-16（第 127 行） | 原件栏写 `implementation-writer.md:51`，原件这一句在第 41 行（`grep -nF 受影响的层 0 流与崩溃枚举用例` 量的），第 51 行是合稿树的 |
| 用户定-17（第 128 行） | 原件栏引 `three-way-inference.md:43`、`:70`，原件第 43 行是「凡是引 kb 的条目」那一条、第 70 行是空行；引的是合稿树的行 |
| 用户定-18、19、20（第 129 到 131 行） | 「原件里那几处」一栏列的是第二版的落点（第 19 条那一栏的 `skills/dispatch/SKILL.md` 原件里没有这份文件；第 20 条的 `agent-common.md:67`、`:80` 在原件里是别的条目）；「合稿里落在哪」一栏分别写「同入口-1、常驻-8」「—」与钩子文件头的行号 |

表头说原话与合稿片段都经 `tools/render_contradictions.py` 核过、problems=0；对第二版不成立。这些不改做法，但第三轮或换上那一步照总表的行号去核，会核到别的句子上。

### C. 〔〕里写的是转述

合稿-1（第 93 行）、合稿-9（第 101 行）、合稿-10（第 102 行）、用户定-4（第 115 行）合稿栏〔〕里的话，在第二版那份文件里全文 0 命中（同一个脚本量的）；意思与第二版那一句相同，但不是原文。合稿-4、6、7、8、11、12 的〔〕本来就写成说明（「read-once 登记行」「无」这类），不算。

### D. 第二版新句子读得出两种意思的

1. 合稿-5：两张表只管 `crates/` 的测试与实验装置。脚本与钩子的测试证红走弄坏开关（第二版 `test-script-discipline.md` 第 40 行「弄坏开关或一条变异」），不进这两张表。照第二版 `show-me-test-by-review.md` 第 7 行「两样都要」读，新写一个门禁自证，要么得往两张表之一硬塞一行，要么读成这条只管那两类测试；原件写的是泛称「存进仓里的变异清单」，不排第三类。该是什么：括号后补「脚本与钩子的照 `.claude/rules/test-script-discipline.md`「测试脚本自己也要被测试证明有效」配弄坏开关」，或把括号写成「例：…」。
2. 合稿-10：第二版第 17 行「做那一步之前读那一节，读全」。原件同一个词是说文件：`.claude/agent-common.md:22`：「照样读全那份文件」。照原件的用法读，是读那一节、文件也读全；照新句读，是那一节读全。同一行前半又说共用约束「写」一节里写「开工另读」的规则开工整份读，而「写」一节那两句是有条件的（写测试、门禁、装置的才另读），这一条读成无条件。
3. 常驻-12 并进来的半句：第二版第 17 行「任务卡住由服务判」，紧接着第 18 行说检出卡住与空转的工具「不拦命令、不停进程与子 agent」；服务判出卡住是停的：`task-server/task_server_lib.py:2141`：「服务停了它」。第 18 行括号里只点了 hook 与看门狗，读成举例就把服务也罩进来、与服务的做法相反，读成穷举就对。原件这两句不挨着，第二版并到一起才读出来。该是什么：第 18 行写「检出卡住与空转的 hook 与看门狗」，或在第 17 行那半句后补「服务判出卡住的照它的错误类型表停」。
4. 用户定-20 的「不在循环里逐个发」与钩子对 python 的放行，见上面那一节与「核不动」第 2 件。

## 核不动（原样交用户）

1. 用户定案第 18 条「用户要求的那一趟也弹窗确认一次」：提交之外跑不指定范围的（用户要求的整轮门禁、`session-wrapup.md` 分不清是谁的红那一趟），第二版写先弹窗问用户、同意了才跑（第二版 `verification.md` 第 38 行），而照提交技能交 `gate-staged.sh` 带 `--full-gate`，提交那一下钩子还要用户在界面上再确认一次；不带 `--full-gate` 交，范围声明闸不拦（`.claude/hooks/run-scope-guard.sh:47`：「gate-staged.sh、gate.sh、replay.sh 不带实验号的那一形」写在不判里）。要查的：用户说的「确认一次」是不是指总共一次；是的话，第二版要写明提交之外那一趟的确认就是 `--full-gate` 那一次弹窗（与提交时同），或写明提交之外不带 `--full-gate`；不是的话，两次照现写法。总表补充一节最后第二条已写「修稿时没核」，这一轮我也定不了，因为差在用户原话的意思上。
2. 用户定案第 20 条「循环逐个发」的射程：每条样本在自己那一轮里起一个新会话子进程、超时只停它自己那一组的 python 装置，钩子放行（`.claude/hooks/bash-command-detector.sh:4114`：「在循环里调也放行（发的是那一个子进程的组）」），第二版共用约束第 67 行字面不许。要查的：用户原话「循环逐个发…一律拒」说的是「对一串进程号或进程组在循环里逐个发」（与钩子一致），还是连每一轮停自己起的那一组也不许（那钩子要改）。查清之前，两处规则写法应当一致地指钩子文件头为准。

## 没打中的形状

- 验证-9：找 `admission.py` 里对缩域那一趟另有判红、对没缩域放过 `exhaustive=false` 的分支——读了判法函数 `judge_crash_case_log` 与 `crash_case_narrowing_settings` 两段，没有。
- 入口-4：找原件里许主 agent 自己写 `crates/` 或 kb 的第三类——原件 `main-agent.md` 里 13 处「自己」逐行读过，没有。
- 常驻-11：找第二版里还有一处许检出工具自己停进程——第二版这一节第 18 行与原件第 22、23 行逐句对过，钩子拒写法的那一支在第二版仍写在「检出」之外，没有。
- 合稿-1 与用户定-17：找第二版里还把代码轮写成「三轮多数」的地方——全树 `grep -rn` 「三轮多数」「再攻两轮」「第二、三轮」「再攻一轮」「最多三轮」「第三轮为最终轮」，只命中三处，逐处读过，都指对了。
- 合稿-12：找改成括号之后丢了判据句的——全树计数 43 → 0 → 46，抽读了 4 处，没丢。
- 用户定-13：找残留的「逐个按进程号停」与「按会话号认」——规则、定义、技能 0 命中。
- 用户定-18：找第二版里还写「用户要求的照跑、不弹窗」的——全树 `grep -rn 用户要求` 命中 8 行（量的；一行是 halt 技能的说明，与门禁无关），都没说不弹窗。
- 格三：37 行逐行对过，动了做法的只有常驻-10。

## 这条腿自己的限度

- 只抽了一次样；「改得对」「没动做法」这类否定结论照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」要再抽一次才能拿去支撑别的结论。
- 第二版相对第一版改过、而不在点名清单里的总表行：入口-1、入口-11、共用-7、共用-8、定义-8、定义-9、常驻-15、常驻-19、按需-M1、M4、M7、M9、M11、验证-2、用户定-4、用户定-6、主 agent 处置-6（数法：`diff-from-r1.patch` 里总表那一段的增删行按编号去重）。其中标「只改措辞」的几行在格三里核了，其余没判。
- 用户定-19 那一处（调查员第 1 步写死 `--cwd` 填副本）是第一版压缩就有的，可能同时归定义那一题；我只核了调查员这一份定义的第 1、6 步与原件第 1 步，没查别的定义有没有借这一步。
- 钩子只读了 ⑥ 的文件头与判循环、判自己那一组的两段代码，没跑钩子；python 放行循环、shell 照拒，是读代码推的（`.claude/hooks/bash-command-detector.sh:4009` 那一判不看是不是自己那一组）。
- 合稿-8、验证-9 读了脚本文件头与判法函数；其余照脚本核的条目只读了文件头。

## 没做什么

- 没写模型，没有要复跑的数；没交任何任务。
- 不判辩方那几格（格四），不替主 agent 采纳。
- 没改总表、合稿树与任何仓里的文件；该怎么改都写在各节「该是什么」里。

## 要主 agent 跑的命令

无。这一腿的读数都来自只读命令与草稿目录里的两个小脚本（`check_cites.py` 对总表引文、`snip.py` 取行片段），都只读仓与合稿树，没有项目代码。

## 草稿目录

`~/.local/share/singlefs/work/rules-compress-contradictions-r2-opus/`：`check_cites.py`、`cites.txt`（总表 309 处带行号引文的逐条读数）、`actual-diff.patch`（第一版到第二版的全量 diff）、`tree-diff.txt`、`patch-dry.txt`、`snip.py`。打补丁核对用的第一版树拷贝 `r1-applied/` 已删（删前 `du -sh` 828K；它是 `research/` 下文字材料的拷贝，不是仓副本，也不是编译目录）。没有建仓副本与编译目录。
