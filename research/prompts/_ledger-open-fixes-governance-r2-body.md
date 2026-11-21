# 旧账本开着项当场修的治理文件改动：治理文件轮第二轮，攻第一轮判决的修补（ledger-open-fixes-governance-r2，2026-11-19）

这一轮攻的是第一轮判决（`research/prompts/ledger-open-fixes-governance-r1-main-verification.md`）坐实的 18 条的修补，都已换上主工作区；另加两样：第一轮只读代码、没复跑的第 11 条（删旧闸），以及用户在第一轮之后定立的装置钉值门禁格。辩方腿复核第一轮判决。第一轮攻过而没打中的角度不重复：同一台判持有者的四样、出口放锁、`load_change_range` 的值与状态、挪场景之后 `s` 的赋值、三份入口只差实验号、入口脚本那一节不该守。方案按 `crates/` 今天的实现来谈。

## 问题

1. 第一轮每一处修补做的是不是判决那一条说的事，修没修到位，有没有带进新的缺口；新立的门禁格判得对不对（攻方腿）。
2. 第一轮判决够不够得着：坐实的 18 条理由成不成立，不采的 2 条撤得对不对，核不动的 2 条该不该在那一轮就核；修补里偏离判决字面的几处，偏得对不对（辩方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自各批草稿目录 `main-agent-logs/` 下的任务日志）：

- 这一轮没有改 `crates/` 下的代码。与 `crates/` 有交点的只有装置钉值的门禁格：它从起跑脚本认出 `crates/*/src/bin/` 下的装置源码。
- **换上工具**（判决第 1 至 4 条，工具员报告 `~/.local/share/singlefs/work/open-fixes-r1-fix-swap-lock/result.md`）：
  - 真换那一段占一份按仓的记录 `<结局文件目录>/singlefs-hook-swap.lock`，`--seal-outcome` 也占它，并看开始行那一趟还在不在。
  - 摘旧记录没照判决字面「改名摘走、核内容、放回」：改成先硬链接成 `<记录>.taking`（排他创建）、读它、是刚判过的那份才删记录。工具员给的理由是改名会让位置空出来。
  - `link` 报已有而读回来是自己的记录时当放成。
  - 读数：副本里自证「查了 110 种」（换上前 102 种）；11 条开关红的格与报告逐格相同；攻方腿探针在新版上 `outcome-overlap-different-scratch … b_exit=10 … next_swap_pending_refusal=9`、`seal-outcome-while-swap-running … seal_exit=10`、`gap-link-reply-lost held=True`、`gap-three-contenders-on-stale-record holding=remover`（等待点挂在 `os.rename` 上，新版不走它，这一臂退化成依次来）；真换「换上了 1 份…验证命令退 0」。
- **文档门禁与完工清理闸**（判决第 5 至 9、15、17 条，报告 `~/.local/share/singlefs/work/open-fixes-r1-fix-doc-gates/result.md` 与后续一件 `~/.local/share/singlefs/work/open-fixes-r1-fix-followups/result.md`）：
  - evidence-in-repo：同一个实验有 timing 行就不走「按复跑表判」（工具员说复跑表的行点名的是驱动名，做不成逐行映射，取的是判决里「做不到按行」那一支）；副作用是 E162、E174、E184 以后改装置而产物不比装置新就红，哪怕改的是 exact 那一段。
  - `gate_added_lines` 的 git 经 `_lib_changed_paths_run_git` 起；后续一件又让 `_lib_changed_paths_run_git` 在 git 成功时也把标准错误原样打出（不这样，`load_change_range` 取到时照打的那一份是空的）。
  - 完工清理闸只判交回的这一个 agent 自己写的那几层，带引号的路径先按引号切，符号链接的 `.deliverables` 判写法不合。
  - 七处旧账本说法改成封存；`doc-process-records.sh` 另有 367、628、776、917、2639、3035–3036、3039 行同一条说法一起改了。
  - 读数：三条探针在新版上都对（E913 的 timing 装置照红、`gate_added_lines exit=1 header_line=1`、清理闸前两臂放行第三臂拒）；开关复跑完工清理闸 76 条、changed-paths 14 条都逐格相同；doc-experiments 样本档 117 个用例全对。
- **三份虚机入口与 e56**（判决第 10 至 12 条与后续一件）：run-condition 加可读内核镜像（退 78）；删旧闸前核输出文件，不在 git 里或在 git 里而有没提交的改动都拒（比判决字面严一格）；E53 不收参数、核 `VM_DISKS` 等于 4；E159 核子命令不空；e56 照同改。读数：开关复跑 e07 14 条、e53 15 条、e159 15 条、e56 14 条都逐格相同。
- **E185 钉值**（判决第 13 条）：三份起跑脚本的钉值改回各自登记第十二节的值（第 2 次重跑 `0bd22743…`、第 3 次重跑 `4368c0e7…`、第 4 次重跑 `9e813cbd…`），三份自证绿；三语实验页与三语变更史写了撤回。
- **装置钉值的门禁格**（用户定立，零轮）：`.claude/gate.d/doc-experiments.sh` 新格 `pinned-apparatus-revision`。认 `scripts/experiment-apparatus/*.sh` 里行首写 `registered_apparatus_sha256=<64 位十六进制>` 的起跑脚本，装置从 `apparatus_source=` 或 cargo 的 `--bin` 认，登记从脚本注释里头一处 `research/prompts/…prereg….md` 认；装置在改动范围里而登记「## 十二、…」一节没有基准没有的非空行就红；登记已归档、不在树里的不判。读数：真仓上「钉了装置 sha256 的起跑脚本 3 份：装置源码这次改动里变了的 0 份…装置没变不判 3 份」；6 条开关都退 1。
- **规则与定义**（判决第 14、16、18 条，主 agent 自己改）：`.claude/main-agent.md` 第 1、15 条与派发表「测试或门禁结果与预期不符」一行；`.claude/rules/session-wrapup.md`、`.claude/rules/show-me-test-by-review.md` 的「先记欠账、按批立」补协作工具当场修；`.claude/agents/investigator.md` 第 33 行两处；`.claude/agents/implementation-writer.md` 第 4 步删 `cargo build`；`.claude/agent-common.md`「子 agent 不交任务」那一条的「clippy / build」。`.claude/agent-common.md` 另一处「clippy / build 逐条的判法」没改，主 agent 判它说的是项目代码闸判两种命令。
- 没修到的：code-tooling 的 stage-break-switch-replay 一格开跑前拒算 246 条、当场判红，拒算点在检出钩子与新加的 `scripts/gate/git-hooks-install.py` 各一处 `bash -c`，归改检出钩子那一路，不在这一轮的改动里。

## 范围

这一轮被判的文件（都读主工作区此刻那一版）：

- `scripts/gate/hook-swap.py`
- `scripts/gate/changed-paths.sh`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/doc-process-records.sh`、`.claude/hooks/completion-cleanup-guard.sh`
- `.claude/gate.d/split-proposal-ledgers.tsv`、`.claude/gate.d/stage-owners.tsv`、`scripts/kb/stale-candidates-record-carriers.tsv`、`.claude/gate.d/break-switch-replay-files.tsv`（第 10、47、89–92 行）、`.claude/hooks/selftest-registry.tsv`（第 103–106 行）
- `scripts/experiment-apparatus/e07-vm-bench-run.sh`、`e53-vm-bench-run.sh`、`e159-vm-bench-run.sh`、`e56-vm-bench-run.sh`、`e185-r2-segment-run.sh`、`e185-r3-segment-run.sh`、`e185-r4-segment-run.sh`、`scripts/experiment-apparatus/README.md`
- 新样本：`.claude/gate.d/fixtures/doc-experiments.sh/` 下 `evidence-in-repo-mixed-replay-red`、`pinned-apparatus-revision-green`、`pinned-apparatus-revision-red`、`pinned-apparatus-revision-unresolved-red`；`.claude/gate.d/fixtures/doc-process-records.sh/batch-scope-git-warning-red`
- `.claude/main-agent.md`、`.claude/rules/session-wrapup.md`、`.claude/rules/show-me-test-by-review.md`、`.claude/agents/investigator.md`、`.claude/agents/implementation-writer.md`、`.claude/agent-common.md`

要看改了哪几行，对着 HEAD 跑 `git diff HEAD -- <文件>`（新样本整份都是改动）。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 换上工具的按仓占用与硬链接摘旧记录：`.taking` 被 KILL 留下之后谁来清、清之前所有趟都退 10 这一条是不是可达的长挡；按仓的记录与按草稿目录的记录两份锁的取放次序，有没有一种交错让一趟占着草稿目录、等着仓、另一趟反过来；`--dry-run` 不占按仓的锁，演练时读到的结局文件与真换时读到的不同，会不会让演练绿而真换红在别处。
2. evidence-in-repo 改成「有 timing 行就照旧比修改时刻」：E162、E174、E184 今天在真仓上判什么；改 exact 那一段而产物不比装置新时红，出路能不能走通（给得出一条按出路做完就绿的路径，或给不出）。`_lib_changed_paths_run_git` 成功时照打标准错误之后，哪些门禁格的输出里多出了 git 的警告，会不会让样本档里以输出比对的格判错（给 文件:行 与样本名）。
3. 完工清理闸「只判自己写的那几层」：怎么认「自己写的」，认错的输入（同一会话的两个子 agent、主 agent 与子 agent 写同一份 `.deliverables`）各会怎样。
4. 删旧闸「在 git 里而有没提交的改动也拒」：复跑一条已入库的产物（同名、内容会变）时，按出路换名或挪走之后，复跑登记表与实验页引的路径还对不对得上；`git` 不在 PATH 或调用目录不在仓里时这一闸判什么。
5. 装置钉值的门禁格（零轮）：
   - 认起跑脚本、认装置、认登记三步，各给一个认错或认不出的写法；
   - 登记已归档就不判，r2、r3 两份今天恰好是这种，判它们是不是永远罩不到；
   - 「登记第十二节有基准没有的非空行」这一判据，给得出一行不是修订、却让它放行的写法（例：只改了空白或注释）。

辩方腿（逐条回「判决站得住」「判决够不着」「推不出」）：

6. 第一轮坐实的 18 条：每条的理由成不成立；第 11 条只读代码、没复跑，判决当时该不该就要求复跑。
7. 第一轮不采的 2 条（「不弹窗问」超出原话、两台 machine-id 相同时判错）与核不动的 2 条（另一台的钟差、第二台跑不跑得起虚机入口）：替「该在第一轮改或核」那一方写出最强的理由再判。
8. 修补里偏离判决字面的几处：摘旧记录用硬链接而不改名、evidence-in-repo 取「做不到按行」那一支、删旧闸比判决严一格、`agent-common.md` 另一处「clippy / build」没改、`doc-process-records.sh` 多改了七处旧账本说法、`_lib_changed_paths_run_git` 成功时照打报错。逐处判偏得对不对，偏错的写该照哪个字面。
9. 单例例外交用户之后用户定「照现在的写法」，31 处只读查表要改：判第一轮判决交用户的那一条写得够不够让用户定（依据、代价、两个选项是不是都写清了）。

## 分工

攻方腿做第 1 到 5 问，辩方腿做第 6 到 9 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
