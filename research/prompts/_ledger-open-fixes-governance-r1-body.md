# 旧账本开着项当场修的治理文件改动：治理文件轮第一轮（ledger-open-fixes-governance-r1，2026-11-19）

用户改了规则：子 agent 与协作工具的问题不再记账，撞上就当场修、当场走一轮三方（`records/2026-11-17-subagent账本清欠调度记录.md:377`）。这一轮攻的是上一轮（`ledger-topdown-governance-r3`）开工快照之后，主 agent 当场改并换上的治理文件，都已在主工作区；单例例外与调查员定义那两处改在改规则之前，也没被攻过，一并在内。上一轮判过、标零轮的修补，这一轮不再攻。方案按 `crates/` 今天的实现来谈。

## 问题

1. 每一处改动做的是不是它要关的那一条：修没修到位，有没有带进新的缺口（攻方腿）。
2. 规则与定义的定点改说的是不是用户定的那几句，与没改的规则、技能、定义有没有互相矛盾（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自各批草稿目录 `main-agent-logs/` 下的任务日志）：

- 这一轮没有改 `crates/` 下的代码。与 `crates/` 有交点的只有 E07、E53、E159 三份复跑入口：它们起的是 `crates/` 外的 `scripts/devices/vm-bench.sh`，构建的是 musl 目标的 release 版，二进制路径取 cargo 报的。
- **换上工具的草稿目录占用**：起因是另一个会话在同一个 `--scratch` 上并发交三份 `--dry-run`，其中一份抛 `FileNotFoundError: … hook-swap/tree` 回溯（草稿树固定是 `<scratch>/hook-swap/tree`，`scripts/gate/hook-swap.py` 换上前第 2594 行）。工具员的改法如下（报告 `~/.local/share/singlefs/work/hook-swap-scratch-concurrency/result.md`「锁的选法」一节）：
  - 一趟一开跑就用硬链接排他放 `<scratch>/hook-swap/swap.lock`，放不成退 10；
  - 记录里写持有者的机器、开机、进程号与启动刻；同一台按这四样判持有者在不在，另一台起的看记录修改时间（每 15 秒刷新，300 秒没刷新就不挡）；
  - `finally` 与 TERM、HUP 处理里放锁。
  
  读数：草稿新版自证报「查了 102 种」（换上前那一份是 90 种）；九个开关都退 1，红的格与报告「弄坏开关」表逐格相同；真换的日志报「换上了 1 份…验证命令退 0（python3 scripts/gate/hook-swap.py --selftest）」。日志在 `~/.local/share/singlefs/work/hook-swap-scratch-concurrency/main-agent-logs/`。工具员自己写明的缝：几趟同时摘同一份旧记录时有极窄的时机两趟都放成；NFS 上 `link` 回复丢失、重发报 EEXIST 时被当成「被占着」（没有自证格）。
- **文档门禁与几个钩子**（报告 `~/.local/share/singlefs/work/ledger-open-doc-gates-and-hooks/result.md` 与 `…-second/result.md`）：
  - `scripts/gate/changed-paths.sh`：取不到改动路径时，把哪一条 git、退几、git 的报错原文写到标准错误。
  - 三道文档门禁在父进程预取的地方接住这段报错，格里取不到时打出来：`doc-process-records.sh` 五个 `load_*` 并成 `load_change_range`；`doc-experiments.sh` 的 evidence-in-repo 加「复跑表 exact 行」一路，工具员标为设计判断。
  - 开关点名照复跑读数改。
  - `completion-cleanup-guard.sh`：交回时 `.deliverables` 写法不合当场拒；第二批把新场景挪到 `hand` 场景前面，修掉连带的三格红。
  - `project-code-guard.sh` 只改注释。
  - `scripts/hosts/memory_peaks_lib.py:42` 注释改成现状。
  
  读数：开关复跑各道「逐格相同」；完工清理闸补点名之后开关复跑报「复跑了 73 条开关（1 份），自证红且点名的格与实际红的格逐格相同 73 条」。
- **E07、E53、E159 复跑入口**：
  - 新建 `scripts/experiment-apparatus/e07-vm-bench-run.sh`、`e53-vm-bench-run.sh`、`e159-vm-bench-run.sh`；每份有删旧闸、条数闸、交付闸与十二个弄坏开关，条款与 文件:行 的对照在 `~/.local/share/singlefs/work/e07-e53-e159-entries/landing-checklist.md`。
  - 三份都登进了 `.claude/hooks/selftest-registry.tsv`、`.claude/gate.d/break-switch-replay-files.tsv` 与 `.claude/gate.d/code-tooling.sh` 的 runner 表。
  - 读数：三份自证退 0；36 条开关都退 1、红在点名的格；code-tooling 样本档报「239 个用例」全对。
- **E185 三份起跑脚本的装置钉值**：`scripts/experiment-apparatus/e185-r2-segment-run.sh`、`e185-r3-segment-run.sh`、`e185-r4-segment-run.sh` 钉的装置 sha256 换成现算的值（换前三份开跑都退 3）。读数：三份自证都绿。
- **规则与定义的定点改**（主 agent 自己改，没有工具员）：
  - `CLAUDE.md`「经验与约定写在哪」；
  - `.claude/main-agent.md`「禁止」一节「禁止自行扩大任务范围」、「一轮怎么开、怎么收」第 2、4 条与派发表「改门禁、钩子…」那一行；
  - `.claude/rules/implementation-workflow.md`「治理文件同规矩」；
  - `.claude/agent-common.md` 第 4 行、第 89 行（认哪些写法以检出钩子文件头 ⑭ 为准）、第 101 行（补「定点替换分几次改脚本两道都不拦，同样不许」）；
  - `.claude/agents/tooling-writer.md` 第 20 行；
  - `.claude/agents/investigator.md:33`（两处「clippy、build」只留 clippy）；
  - `.claude/rules/code-discipline-by-review.md:54`（单例补一种例外：进程级、建好之后只读的外部句柄可以用 `OnceLock`）。
  
  用户的原话与弹窗定案在 `records/2026-11-17-subagent账本清欠调度记录.md:377`；单例那一条的定案在同一份记录里，原话是「补例外」。这几份文件里也有别的会话同期改的行（例：`.claude/agent-common.md`「写」一节任务号、会话短号与批号不进仓那几行），不在这一轮的射程里。

## 范围

这一轮被判的文件（都读主工作区此刻那一版）：

- `scripts/gate/hook-swap.py`
- `scripts/gate/changed-paths.sh`、`.claude/gate.d/doc-text.sh`、`.claude/gate.d/doc-process-records.sh`、`.claude/gate.d/doc-experiments.sh`、`.claude/hooks/completion-cleanup-guard.sh`、`.claude/hooks/project-code-guard.sh`、`scripts/hosts/memory_peaks_lib.py`
- `scripts/experiment-apparatus/e07-vm-bench-run.sh`、`scripts/experiment-apparatus/e53-vm-bench-run.sh`、`scripts/experiment-apparatus/e159-vm-bench-run.sh`、`.claude/hooks/selftest-registry.tsv`、`.claude/gate.d/break-switch-replay-files.tsv`、`.claude/gate.d/code-tooling.sh`（只判第 2344 行起登记三份入口自证的那几项）
- `scripts/experiment-apparatus/e185-r2-segment-run.sh`、`scripts/experiment-apparatus/e185-r3-segment-run.sh`、`scripts/experiment-apparatus/e185-r4-segment-run.sh`
- `CLAUDE.md`、`.claude/main-agent.md`、`.claude/rules/implementation-workflow.md`、`.claude/agent-common.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/investigator.md`、`.claude/rules/code-discipline-by-review.md`

要看改了哪几行，对着 HEAD 跑 `git diff HEAD -- <文件>`；新建的三份入口整份都是改动。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 换上工具的草稿目录占用：
   - 每一条进口（`--dry-run`、真换、`--feed-only`、`--check-callers`、`--seal-outcome`、退回规格、只给脚本规格的）是不是都在建树、写文件之前占住，并在每一条出口放掉；
   - 同一台判持有者在不在的四样，在进程号复用、容器与重启之后会不会判错；
   - 另一台看修改时间那一支，在两台时钟差、NFS 属性缓存下会不会让两趟同时进同一个草稿目录；
   - 工具员写明的两条缝，各给一条能照喂的复现，或判它够不着。
2. `changed-paths.sh` 与三道文档门禁：
   - 把 git 报错写到标准错误之后，有没有哪一格的结局、哪一条成功句或拒绝句跟着变了；
   - `load_change_range` 与原来五个 `load_*` 在取不到、取到空、取到多行三种情形下是不是逐个相同；
   - evidence-in-repo 新加的「复跑表 exact 行」一路该判什么，有没有让原来该红的放行。
3. 完工清理闸「`.deliverables` 写法不合当场拒」：照 `.claude/hooks/completion-cleanup-guard.sh` 文件头「要留的交付物」那一段的写法，造多拒的与漏拒的写法各几条；第二批挪场景之后，自证里 `s` 这个名字还有没有别处被换掉、连带让格判错。
4. E07、E53、E159 三份入口：
   - 照 `.claude/rules/test-script-discipline.md`「入口脚本：不带参数就是不指定范围」与「所有任务都要多机可用」（`.claude/rules/verification.md`），逐条判它们该不该守、守没守（`--list-items`、退出码 78 的准入、派到第二台跑不跑得起、产物落在哪）；
   - 三份是不是只差实验号，有没有一份漏改了别的地方。
5. E185 三份钉值：钉的值与装置此刻现算的 sha256 是不是同一个量；装置再改一次之后，起跑脚本会不会照样只在开跑时才红（给出这一类以后怎么拦）。

正推腿（每问回「成立」「不成立」「推不出」；推不出与不成立的给 文件:行）：

6. 规则与定义的定点改：每一处说的是不是 `records/2026-11-17-subagent账本清欠调度记录.md:377` 那一行里用户的原话与两句弹窗定案；改完之后，仓里没改的规则、技能、定义、门禁文件头还有没有照旧写「记账」「登进账本」「按批走一轮三方」「一周合一批」的（全仓搜，给 文件:行），与新写法矛盾的逐处列出。
7. `.claude/agent-common.md` 第 89、101 行：与检出钩子 `.claude/hooks/bash-command-detector.sh` 文件头 ⑭ 与写闸 `.claude/hooks/write-guard.sh` 今天的判法逐项对，看多写、少写、写错的。
8. `.claude/rules/code-discipline-by-review.md:54` 的单例例外：与 `.claude/singlefs-ai-sop/rules/code-discipline.md`、`.claude/rules/machine-first.md` 有没有冲突；「进程级、建好之后只读的外部句柄」这个射程，今天 `crates/` 里用 `OnceLock` 的几处（`grep -rn OnceLock crates/`）各落不落在里面。
9. 开关点名与登记行：`doc-text.sh`、`doc-process-records.sh`、`doc-experiments.sh`、`completion-cleanup-guard.sh` 文件头改了点名的那几条开关，`selftest-registry.tsv` 第 104–106 行、`break-switch-replay-files.tsv` 第 90–92 行与 `code-tooling.sh` 第 2344 行起登记三份入口自证的那几项，各与脚本今天的开关表与格名对不对得上。
10. `.claude/agents/investigator.md:33` 与 `.claude/agents/tooling-writer.md` 第 20 行改完之后，与 `.claude/agents/implementation-writer.md` 第 1b 步、`.claude/agent-common.md`「子 agent 不交任务」是不是一个说法。

## 分工

攻方腿做第 1 到 5 问，正推腿做第 6 到 10 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
