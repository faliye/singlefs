# 旧账本开着项当场修的治理文件改动：治理文件轮第三轮（最后一轮），攻第二轮判决的修补（ledger-open-fixes-governance-r3，2026-11-19）

这一轮攻的是第二轮判决（`research/prompts/ledger-open-fixes-governance-r2-main-verification.md`）「三、改法」六条的修补，都已换上主工作区。这一轮是最后一轮，这一轮的修补在判决里标零轮，不再开第四轮。方案按 `crates/` 今天的实现来谈。前两轮攻过而没打中的角度不重复：同一台判持有者的四样、出口放锁、`load_change_range` 的值与状态、挪场景之后 `s` 的赋值、三份入口只差实验号、入口脚本那一节不该守、两份锁反序互等、`--dry-run` 不占按仓的锁、`_lib_changed_paths_run_git` 成功时照打警告让样本档判错、承接的子 agent 被前一个写坏的行拦。

## 问题

1. 第二轮每一处修补做的是不是判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 第二轮修补里偏离判决字面的三处，偏得对不对；修补的自证、样本与开关证明的是不是名字说的事（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自各批草稿目录 `main-agent-logs/` 下的任务日志）：

- 这一轮没有改 `crates/` 下的代码。与 `crates/` 有交点的是 doc-experiments 两格：evidence-in-repo 开始认 `crates/*/src/bin/` 下以 `e<号>_` 起头的装置，pinned-apparatus-revision 从起跑脚本认出 `crates/*/src/bin/` 下的装置源码。
- **换上工具 `.taking`**（判决第 1 条，工具员报告 `~/.local/share/singlefs/work/open-fixes-r2-fix-taking/result.md`）：`.taking` 里写持有者身份（与占用记录同一套写法），判它挡不挡照记录那一套（同一台看进程、另一台看修改时间，过期 360 秒）；摘的那一趟登记进 `HELD_REMOVAL_CLAIMS`，被 TERM 或 HUP 杀掉时由 `release_then_die` 放掉；建 `.taking` 的 `link` 报已有而读回来是自己写的，当放成。新加四格自证（KILL 留下、TERM 当中、link 回复丢了、还占着时点名），四条新开关、七条旧开关的红格名单加了新格。读数：副本里换上前那一版加新自证恰好红新四格；18 条开关减去 16 格副本缺文件的环境格之后逐条红在文件头点名的格；攻方腿探针在新版上 `taking-left-by-kill-repository-lock … next_swap_exit=0 seal_exit=2 … next_killed_holder_then_swap_exit=0`、TERM 臂 `taking_left=False`、回复丢了臂 `first_held=True taking_left=False`，基线版上照旧 `next_swap_exit=10 seal_exit=10`、`taking_left=True`；换上时验证命令 `python3 scripts/gate/hook-swap.py --selftest` 退 0。
- **evidence-in-repo**（判决第 2 条，报告 `~/.local/share/singlefs/work/open-fixes-r2-fix-experiments-gate/result.md`）：有 timing 行的实验，比修改时刻的对象改成每一条 timing 行点名的产物（只存着 `.gz` 的也认），每一份都要在盘、不比装置旧；exact 行的产物与别的 `e<号>` 文件不再作保；`crates/*/src/bin/e<号>_*.rs` 与目录形装置一并认。判决写「那一份」，工具员取「每一条都要」。
- **pinned-apparatus-revision**（判决第 4 条，同一份报告）：触发改成钉值变了或装置变了；登记认脚本里写「跑前登记 <路径>」那一处（三份 E185 起跑脚本是文件头注释那一行）；钉值行允许行尾注释；登记不在树里时取基准那一版，再取 git 历史里它被删前的那一版；装置变了的，第十二节要新加写着新钉值前 12 位的行，且这 12 位基准那一版里没有，旧钉值的行一行不能少。**偏离判决字面**：装置没变、只改钉值的，第十二节里写着新钉值前 12 位即可，不要求是新加的行（主 agent 收了：r4 钉值改回登记第十二节本来写着的值，照字面要为它补一行没有内容的修订）。登记路径写错与历史里从没有过这份文件的，与「已归档」分不开，照旧不判。
- doc-experiments 两格的读数：19 份新样本（`.claude/gate.d/fixtures/doc-experiments.sh/` 下 `evidence-in-repo-crates-bin-*`、`-exact-only-rerun-red`、`-timing-rows-*`、`-unrelated-file-red`，`pinned-apparatus-revision-` 起头的十三份）打上并暂存；关着开关样本档 136 份全对；18 条开关（新加 11 条、名单变了 5 条、共用库两条）逐条判错的样本与文件头点名的相等；攻方腿两份探针九个样本全红；真仓 pinned-apparatus-revision 绿（三份 E185 起跑脚本：「装置没变而钉值改了的 3 份，登记第十二节都新加了写着新钉值的行」，r2、r3 取自 git 历史 `a24379563928` 里被删之前的那一版），evidence-in-repo 红在 E158、E180、E202（装置今天改过、产物没有或实验页缺那一句），三处归改那三份装置的会话，提交前它弄绿。
- **删旧闸**（判决第 3 条）：四份入口 `e07`、`e53`、`e159`、`e56-vm-bench-run.sh` 两条 git 的路径加 `:(literal)`；攻方腿探针在新版上八臂都 `exit=2 untracked_output_kept=true refused_by_delete_gate=true reached_build=false`、提交了的那份原样；开关 `existing-output-deleted-unchecked` 下多红的方括号那一格补进了点名，四份开关复跑 15、16、16、15 条逐格相同。
- **完工清理闸**（判决第 5 条）：下层 `.deliverables` 已声明交接的副本，上层子 agent 交回时算已声明；新加三格 `nested-lower-handover-main-listed`、`nested-undeclared-copy-blocks`、`nested-lower-handover-passes`，补进三条开关的点名。读数：攻方腿探针五臂里上层不再被下层交接副本拦（`nested-upper-blocked-by-lower-handover-copy action=pass`），承接者被前一个写坏的行拦照旧（第二轮不采那一条）；换上后自证「判了 106 格；77 个弄坏开关各红在点名的格」，开关复跑 77 条逐格相同。
- **stage-owners**（判决第 6 条与跟着新判法改的两行）：第 143 行改成封存的说法；第 117、130 行的「为什么」照 doc-experiments 两格新判法改；单格核 stage-owners「17 道门禁、195 格」绿。
- 没修到、不归这一轮的：doc-experiments 真仓 evidence-in-repo 的三处红（归改装置的会话）。

## 范围

这一轮被判的文件（都读主工作区此刻那一版；要看改了哪几行，对着 HEAD 跑 `git diff HEAD -- <文件>`，新样本整份都是改动）：

- `scripts/gate/hook-swap.py`
- `.claude/gate.d/doc-experiments.sh`，与 `.claude/gate.d/fixtures/doc-experiments.sh/` 下这一轮新加的 19 份样本
- `.claude/hooks/completion-cleanup-guard.sh`
- `scripts/experiment-apparatus/e07-vm-bench-run.sh`、`e53-vm-bench-run.sh`、`e159-vm-bench-run.sh`、`e56-vm-bench-run.sh`
- `.claude/gate.d/stage-owners.tsv`（第 117、130、143 行）

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 换上工具 `.taking`：两趟同时摘同一份已死的 `.taking`（工具员自己列进文件头「留下的缝」那一种）后果多大，有没有一种交错让两趟都以为自己摘成、都去删记录；`release_then_die` 放 `.taking` 与放草稿目录记录的次序，被 TERM 在两者之间时留下什么；另一台起的 `.taking` 过期 360 秒，与记录的过期判法是不是同一个数、同一个时钟。
2. evidence-in-repo「每一条 timing 行都要」：一个实验几条 timing 行、只重跑其中改到的那一段时，出路能不能走通（给得出一条按出路做完就绿的路径，或给不出）；`crates/*/src/bin/` 下目录形装置与 `e<号>_` 起头而不是装置的二进制各判成什么。
3. pinned-apparatus-revision 的放宽：只改钉值时「第十二节里写着新钉值前 12 位即可」，给得出一种钉回一个已被后续修订作废的旧值也放行的写法吗；「登记路径写错」与「已归档」分不开，给一个今天就会被当成「不判」的真实写错；「旧钉值的行一行不能少」与「整行抄一行已有的不算」两条判据之间有没有一种写法两条都过而其实没修订。
4. 删旧闸 `:(literal)`：`git` 的别的路径解法（大小写不敏感的文件系统、`core.ignorecase`、带 `:` 起头的文件名）还能不能让一份没进 git 的输出被当成在 git 里；四份入口这两条之外有没有第三处按 pathspec 解输出路径。
5. 完工清理闸「下层已声明交接的副本，上层交回时算已声明」：下层是别的会话、别的 agent 的目录时上层也认它的声明吗；下层 `.deliverables` 写坏、被删、换成符号链接时上层判什么。

正推腿（逐条回「成立」「不成立」「推不出」）：

6. 第二轮「三、改法」六条，逐条核今天的代码做的是不是那一条说的事，写出代码里的 文件:行 与判决那一句的原文。
7. 偏离判决字面的三处（evidence-in-repo 取「每一条都要」、钉值格只改钉值时的放宽、换上工具的新格名单是按读码推的）：逐处判偏得对不对，偏错的写该照哪个字面。
8. 新加的自证格、样本与开关：每一样红的地方是不是它名字说的那件事（给一份样本或一格自证，说清它红在哪一行判法上）。

## 分工

攻方腿做第 1 到 5 问，正推腿做第 6 到 8 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
