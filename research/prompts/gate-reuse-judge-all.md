# gate-reuse 阶段同步：逐行判（F1–F7）

基准 `896b73f`，结束是暂存区树 `3fe1b43c583d789559eb15a603e9a9a407f8e364`
（主 agent 复核：与派发提示原给的 `94e844a4e8d2f34dad575bbf7f6aa332b0c9289d` 只差
`.claude/gate.d/55-qemu-first-transaction.sh`、`59-crates-mutation-replay.sh` 两份的文件模式
100644→100755，内容逐字节相同，`git diff --name-status` 已核对）。
候选表 `/tmp/claude-1000/gate-reuse-sweep/candidates-v2.tsv`（18 行），事实表
`/tmp/claude-1000/gate-reuse-sweep/facts-v2.tsv`；两份都不重算。全部 18 行逐行读过，
读上下文一律 `git show 3fe1b43c...:路径`，没有读工作区文件。

第 10 步反向核对：`stage-must-run.sh`、`gate-staged.sh`、`stage-inputs.tsv`、`staged-green` 四个新名字
在 `.claude/kb`、`.claude/rules`、`.claude/agents`、`records/`、`CLAUDE.md`、`agent-common.md`、
`main-agent.md` 里全文零命中（脚本已核，见交回正文），只有 `C477` 命中一处（`checks-owed.md` 新增行）。
这批新东西的旧问题在 `checks-owed.md` 的 C8、C449、C454 与 `implementation-workflow.md` 里都
已经有对应的旧现状句——都在下表被判「要改」，因此没有另开 M1/M2 这类补行：这五件做成的事
各自都落在某一行的判定里，不是零记录。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F1 | .claude/kb/checks-owed.md:22 | 要改 | C8 出处列末句「**同一轮里跑第二、三次仍然重跑**：那批暂存改动相对 HEAD 一直是「碰了」，摘掉它要记每道阶段上次跑绿时的输入指纹，而指纹得落盘、会腐化、新 clone 拿不到，这一半照旧欠着」——「这一半照旧欠着」现在不真：`research/scripts/stage-must-run.sh` 正是这个指纹机制，拿 `refs/sop/staged-green` 那棵树与暂存树的 git diff 当指纹，不必另外落盘（用 git 原生内容比对，不会像散列文件那样腐化），54、55、59、74、87 五道已接上，同一轮第二、三次跑门禁只要暂存树没变就直接退 77。改后（替换该句）：「**2026-09-22 落了 `research/scripts/stage-must-run.sh`**：拿 `refs/sop/staged-green` 那棵树与这一次的暂存树做 git diff 当指纹，不必另外落盘、不腐化；54、55、59、74、87 五道已接上，同一轮第二、三次跑、暂存树没变就直接退 77。⚠️ 仍欠两样：新 clone 没有这条 ref 时只能退回全跑（不是真的『拿不到』就出错，是安全地当成要跑）；只接了这五道重阶段，56、68 这类项目本地阶段还没有」。 |
| F1 | .claude/rules/implementation-workflow.md:30 | 不相干 | 这一行只陈述判据本身——原话「判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」」——和几个阶段读什么输入的例子，是一句不依赖「今天是人工核对还是自动核对」的抽象准则；`stage-must-run.sh` 正是照这条判据实现的，没有推翻它，只是把它自动化了。这条变化落在下面的表格行（37 行，F2 已判），不落在这一句定义句上。 |
| F2 | .claude/rules/implementation-workflow.md:37 | 要改 | 本行「怎么算数」列写「逐个路径现查，输出不截断；说不全那一道读什么，就没有复用的资格」，把「说清一道阶段读什么」全压在手工现查上。现在 `.claude/gate.d/stage-inputs.tsv` 是这批路径的唯一登记位，54、55、59、74、87 五道可以直接读登记表、由 `stage-must-run.sh` 自动比对，不必再手工「现查」；照原句去读，容易让人以为这五道阶段仍要手工核对每个路径。改后（该行末尾加一句）：「已登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段（54、55、59、74、87 号），这一步由 `research/scripts/stage-must-run.sh` 读登记表自动比对，不必再手工逐个路径现查；没登记的阶段仍按本行手工核对」。 |
| F3 | .claude/agents/gate-triage.md:31 | 不相干 | 这一行列的是 gate-triage 自己直接跑 `.claude/scripts/gate.sh --staged` 时门禁本身做的 git 写，原话「`--staged` 时的临时 worktree」说的是共享 `gate.sh` 自己的行为。gate-triage 的定义（第 25 行）只调用 `.claude/scripts/gate.sh --staged`，不调用这一批新立的 `research/scripts/gate-staged.sh`；后者才会前移 `refs/sop/staged-green`。这一行没有变得不准，只是与这一批新立的东西不相干。 |
| F3 | .claude/kb/checks-owed.md:22 | 要改 | 判定与理由同上一条 F1 对这一行的判定：出处列末句「这一半照旧欠着」现在不真，`stage-must-run.sh`（用 `refs/sop/staged-green`）正是它要的指纹机制。此外这一行里另有一句提到的 `refs/sop/gate-ok`（GATE_BASE 的回退目标）与这一批新立的 `refs/sop/staged-green` 是两条不同的 ref，未受影响，不必改。改后的句子同 F1 那一条给出的替换文本。 |
| F3 | .claude/kb/checks-owed.md:390 | 不相干 | C439 说的是 `refs/singlefs/gate-ok`（共享 `gate.sh` 整轮不带 `--staged` 判绿用的 ref）前移导致门禁 88 号漏判早先写下的行，原话「该前移点之前写下的错引文，之后再跑就永远不在判定范围里了」，讲的是 88 号与 `refs/singlefs/gate-ok` 的关系，跟这一批新立的 `refs/sop/staged-green`、`stage-must-run.sh` 不是同一条 ref、同一件事。 |
| F3 | .claude/kb/checks-owed.md:397 | 要改 | C449「有意取别的基准」的豁免只举了两个例子——门禁 11 号取 HEAD、原话「68 号判据 ⑤ 同理」——而这一批之后 54、55、59、74、87 五道阶段接了 `stage-must-run.sh`，它答的是「这一道读的路径相对 `refs/sop/staged-green` 变没变」，是与共享 `diff_base`、项目 `GATE_BASE` 都不同的第三套基准算法，同样该进这张豁免表，不然 C449 这道检查会把这五道的新判据误判成「两套基准不一致」。改后（在「68 号判据 ⑤ 同理」后加一句）：「2026-09-22 起再加一种：54、55、59、74、87 号接了 `research/scripts/stage-must-run.sh`，它答的是『这一道读的路径相对 `refs/sop/staged-green`（上次整轮全绿的暂存树）变没变』，是与 `diff_base`、`GATE_BASE` 都不同的第三套算法，豁免表要把这五道也列进去」。 |
| F3 | .claude/kb/checks-owed.md:398 | 不相干 | C450 说的是门禁 91 号（归档删旧 sync 记录）与 68 号（要求触发文件都被同步记录点名）之间的时间基准差一格，原话「两道直接打架」，判据来自 `diff(HEAD~1, 工作区)` 与「相对 HEAD 改没改过」的落差，跟这一批新立的 `refs/sop/staged-green`、`stage-must-run.sh` 不是同一套基准、同一件事。 |
| F3 | .claude/kb/checks-owed.md:438 | 要改 | C454 简称「--staged 跑绿不让基准前移，而且不说」，正文限定的是上游 `scripts/gate.sh` 自己（原话「这一轮没有给 `gate-ok` 前移」），这半仍真——`refs/singlefs/gate-ok` 照旧不被它移动。但这一批之后，本仓的外壳脚本 `research/scripts/gate-staged.sh`（调 `gate.sh --staged` 之后再多做一步）在整轮全绿时会前移 `refs/sop/staged-green`；单独摘出简称读，容易读成「--staged 跑绿在这个项目里从不前移任何东西」，那句已经不对。改后（末尾补一句）：「⚠️ 2026-09-22 起这句只在指 `scripts/gate.sh` 自己、以及它前移的 `refs/singlefs/gate-ok` 时成立：本仓的外壳 `research/scripts/gate-staged.sh` 调用它之后另外前移 `refs/sop/staged-green`，`--staged` 跑绿在这个项目里不再是『什么都不前移』」。 |
| F3 | records/2026-09-16-subagent拆分提案.md:279 | 不相干 | 这一行是 2026-09-16 那份提案核对脚本用法是否「对得上」的一条清单项，原话「共享 `gate.sh` 全绿时 `update-ref refs/singlefs/gate-ok`」，说的仍是那条未变的 ref；这一批新立的 `refs/sop/staged-green` 与 `gate-staged.sh` 不在这份 2026-09-16 的清单射程里，不是同一件事。 |
| F4 | .claude/kb/checks-owed.md:367 | 不相干 | C415 说的是门禁 59 号变异表复跑时编译缓存跨轮复用导致陈旧产物被当成新的，原话「`GATE_MUTATION_TARGET_DIR` 跨轮、跨源码树复用」，是编译缓存 mtime 的问题，跟这一批给 59 号加的范围判定（该不该整表跑）不是同一件事。 |
| F4 | .claude/kb/checks-owed.md:412 | 不相干 | C465 说的是 59 号自己开的工作进程吃满核，跟同一门禁里另一道跑的按挂钟判的性能断言互相干扰，原话「开 8 个工作进程吃满核」，这是既有的并发架构细节，不是这一批改的东西；59 号的工作进程数、进程池那类改动本身也不是这一批的（那是另一个会话未提交、已撤回的东西），此行只判「不相干」。 |
| F4 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:17 | 事件句不改 | 这一行是 2026-09-17 那一段续做的收尾记录，原话「变异表复跑仍 14 条全抓」，说的是那次跑出来的具体读数，是「那一次发生的事」，不是关于 59 号有没有范围判定的现状句。 |
| F4 | .claude/kb/milestone/02-second-txn.md:380 | 事件句不改 | 这一行是里程碑收口表第 42 条，记录 2026-09-18 一次锚点腐化事故及其修复，原话「同日整表复跑 59 号，123 条变异各自红在点名的测试上」，是那一次事故与修复的记录，不是关于 59 号今天有没有范围判定的现状句。 |
| F4 | .claude/rules/implementation-workflow.md:12 | 不相干 | 这一行是「三步」表里的静态映射，把检查方式对应到门禁号，原话「59 号（crates 变异表复跑）」只是给 59 号起的既定简称，这一批加的是 59 号该不该整表跑的范围判定，不改它仍然是「crates 变异表复跑」这件事，两者不是同一层面。 |
| F4 | records/2026-08-29-复跑复核轮.md:21 | 事件句不改 | 这是 2026-08-29 那一轮复跑复核的具体读数记录，原话「48 / 0 / 1」（10 张表的被抓/盲区/无效计数），是那一次跑出来的数，不是关于 59 号有没有范围判定的现状句。 |
| F5 | .claude/kb/checks-owed.md:22 | 要改 | C8 出处列中段把 `research/scripts/change-touches-crates.sh` 描述成「接进门禁 54、55、74、87 号四道重阶段」的唯一范围闸（原话「接进门禁 54、55、74、87 号四道重阶段」），而这一批之后它降成第二道闸：先由 `stage-must-run.sh` 判定要不要跑，`change-touches-crates.sh` 只在要跑时再判一次「碰没碰前缀」。改后（在「接进门禁 54、55、74、87 号四道重阶段」后加半句）：「；2026-09-22 起这四道先问 `stage-must-run.sh`（按 `stage-inputs.tsv` 登记的具体路径与 `refs/sop/staged-green` 比对），`change-touches-crates.sh` 降为第二道闸，只在 `stage-must-run.sh` 判定要跑时才接着判『碰没碰前缀』这个更粗的信号」。（此行与 F1/F3 对同一行的判定是同一处「要改」，三个理由分别对应三个检索词命中的分句，改法可以合并进同一次编辑。） |
| F6 | .claude/kb/checks-owed.md:397 | 要改 | 与 F3 对这一行的判定相同：C449「有意取别的基准」的豁免表只举了门禁 11 号与 68 号判据 ⑤（原话「68 号判据 ⑤ 同理」）两个例子，没列这一批新立的第三种情形——54、55、59、74、87 号接了 `stage-must-run.sh`，用 `refs/sop/staged-green` 这第三套基准。改后的句子同 F3 那一条给出的替换文本。 |

## 第 11 步：--check-report

候选表第一列实际只有 F1–F6（`cut -f1 candidates-v2.tsv | sort -u` 核过）：F7 对应的事实
（C477，「新立：」开头）按 `stale-candidates.py` 的规则不生成候选行，这是预期行为，不是遗漏——
已在第 10 步反向核对里确认 C477 本身已记在 `checks-owed.md`。用 `--groups F1,F2,F3,F4,F5,F6,F7`
跑会被参数校验拒绝（退出码 2，「`--groups` 里有候选表没有的组：F7」），改用
`--groups F1,F2,F3,F4,F5,F6` 之后：

```
$ python3 research/scripts/stale-candidates.py --check-report /tmp/claude-1000/gate-reuse-sweep/candidates-v2.tsv /tmp/claude-1000/gate-reuse-sweep/judge-all.md --groups F1,F2,F3,F4,F5,F6
  ✓ 报告判全了：18 行候选都有逐行判定
```
