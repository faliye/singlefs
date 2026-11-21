# 前台 timeout 过长与提交前大文件两道执行前拒：治理文件轮第二轮（fg-timeout-large-file-governance-r2，2026-11-19）

第一轮判决是 `research/prompts/fg-timeout-large-file-governance-r1-main-verification.md`。那一轮两条腿都打中了，坐实的改法分两批改完、已经换上主工作区。这一轮攻这两批修补。方案按 `crates/` 与这些治理文件今天的实现来谈。

用户在 2026-11-19 弹窗里选了「照规矩开第二轮」。

## 问题

1. 两批修补做的，是不是第一轮判决「三、改法」那四条，和后一批关的那五条（辩方腿）。
2. 修补本身哪一格会错：新加的拒绝误拒、还剩静默放过、判法改法出了新毛病、写范围表新行写错（攻方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码，改动也不读 `crates/` 下的任何文件。
- **前一批（修第一轮坐实的四条）**：工具员报告在 `~/.local/share/singlefs/work/fg-timeout-large-file-r1-fixes/result.md`。
  - 换上：`scripts/gate/large-file-check.py` 与检出钩子都经 `scripts/gate/hook-swap.py` 换上，结局行都是「✓ 换上了 1 份（改名换上新 inode、回读一致）」；检出钩子后一批又换过。
  - `--guard push` 改成按推出去的引用判，选项见 `scripts/gate/large-file-check.py` 第 328 行的 `GUARD_PUSH_OPTIONS`，函数是 `guard_pushed`（第 452 行）。
  - 判法读索引文件的拷贝，不写用户的索引；起 git 带 `-c diff.relative=false`。
  - 钩子 ㉒ 认别名、剥启动器、认长选项缩写；判不准的五类写法在执行前拒。
  - 判法放进自己的进程组，到点停整组；出错一律折成「判不了，放行并记检出」。
- **后一批（名单一致、写范围八类、㉓ 出路、第 75 行、`commit -F` 那一格）**：工具员报告在 `~/.local/share/singlefs/work/fg-timeout-large-file-batch4/result.md`。
  - 换上：检出钩子、`.claude/gate.d/code-tooling.sh`、`scripts/agents/dispatch-prompt.py` 三份一份一份经 `scripts/gate/hook-swap.py` 换上，`dispatch-prompt.py` 那一份带 `--verify 'python3 scripts/agents/dispatch-prompt.py --selftest'`，都回读一致。
  - 之后别的会话为 ⑰ 换过一次检出钩子；主 agent 又换了一次，只挪文件头三行开关点名：只读定义那九格从 `scope-known-scripts-unseen` 挪到 `allow-bash-scope-write`。检出钩子现行 sha256 前缀是 02d0c3c8。
  - ㉓ 的出路带上 clippy lint 表：`clippy_lint_tail`（检出钩子第 4042 行）现读 `.claude/singlefs-ai-sop/scripts/check.sh` 的 `CODE_DISCIPLINE_LINTS`。
  - 开关名单 `KNOWN_DETECTOR_BREAK_ITEMS`（第 1623 行）、文件头开关表、代码里读的名字三处一致，由 `break_switch_name_sets`、`break_switch_set_mismatches`（第 1686、1704 行）判。
  - `.claude/hooks/agent-write-scope.tsv` 补了只带 Read、Bash 的八类定义，门禁 code-tooling 的 agent-write-scope 格改成每个定义都要有行（弄坏开关 `readers-unchecked`，第 773 行）。
  - `.claude/agent-common.md` 第 75 行 ② 那半句改成了实话。
  - 自证补了「先把提交说明写到仓外，再 `git commit -F`」放行那一格。
- **验过的**：
  - 检出钩子自证 `bash .claude/hooks/bash-command-detector.sh --selftest`：「✓ 自检通过（查了 1493 种）」。
  - code-tooling 样本档：「点名的 1 道阶段判得都对（240 个用例）」。
  - `bash .claude/gate.d/code-tooling.sh --check agent-write-scope`：「6 个有 Write 或 Edit 的定义与 8 个只带 Read、Bash 这类的定义都登记了、72 条路径模式」。
  - `bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/bash-command-detector.sh`（挪完头注之后）：「✓ 脚本与钩子的弄坏开关复跑：复跑了 162 条开关（1 份），自证红且点名的格与实际红的格逐格相同 162 条」。
- **主 agent 已知、没改的**（交腿判）：
  - 三方腿那几行写范围是 `research/prompts/*-*-output.md`、`research/prompts/*-*-model/**`，允许一条腿写任何一轮、任何一条腿的报告。
  - 工具员报告写「`echo … | tee /tmp/… && git commit -F` 仍被拒，没动」。
  - `git push` 前面排着改历史的命令，没拦。
- 第一轮两份探针还在：`~/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-main/sonnet-probes/`、`~/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-main/opus-model/`。第一轮读数在同一目录的 `out-*.txt` 与 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-main-run/probe.out`。复跑结局写进判决。
- 工具层的背景（任务服务、换上工具怎么用）在 `.claude/kb/tooling.md`。

## 范围

这一轮被判的文件（都读换上之后的那一版）：

- `.claude/hooks/bash-command-detector.sh`：文件头 ㉑、㉒、㉓ 三段（第 419 行起），开关表里两批新加与补了点名的那几条，第 1623 到 1720 行附近的名单一致性，第 3244 行起 ㉑、第 3274 行起 ㉒、第 4042 行 `clippy_lint_tail` 一带，自证里「前台超时:」「提交前大文件:」「出路过别的闸:」「写范围:」几组格。⑫、⑭ 的判法不在这一轮判；⑫ 只判它读写范围表新行的那一面。
- `scripts/gate/large-file-check.py`：`--guard` 那一半与文件头退出码表。
- `.claude/gate.d/code-tooling.sh`：agent-write-scope 格与它的样本。
- `.claude/hooks/agent-write-scope.tsv`：新加的八类定义的行与文件头。
- `scripts/agents/dispatch-prompt.py`：后一批改的那几处，见 `git diff HEAD -- scripts/agents/dispatch-prompt.py`。
- `.claude/agent-common.md`：第 70、75、96、97 行。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（一条 hook 输入 JSON、一串 git 操作、一条命令）或 文件:行。

辩方腿（复核第一轮判决与两批修补够不够得着）：

1. 第一轮判决「二、判决」里坐实的每一格，改完是不是都关上了。逐格写它现在判成什么、依据是哪一行代码。
2. 第一轮判决里「不算打中」那几条，判得站不站得住。
3. 后一批那五条做的是不是派发说的事：㉓ 出路、名单三处一致、写范围八类、第 75 行、`commit -F` 那一格。
4. 前一批定的「判不准就拒、出路是改写成能判的形态」，会不会把项目里主 agent 日常的提交写法也拒掉。看 `.claude/skills/gate/staged-gate.md` 与 `records/` 里近来真实用过的 git commit、git push 写法，逐条判。

攻方腿（攻修补，不重复第一轮已攻过的角度）：

5. 前一批新加的「执行前拒」五类：造得出误拒日常写法的输入吗？造得出照样静默放过的输入吗？例如别名链、`git -c alias.x='!git commit'`、`env GIT_INDEX_FILE=…`、`git -C "$(pwd)"`。
6. push 按引用判：`origin +HEAD:refs/heads/x`、`--force-with-lease`、`--set-upstream` 新分支、远端没有跟踪引用、`push.default` 的几种取值，各判成什么。
7. 判法读索引拷贝、进程组、`-c diff.relative=false`：大仓、索引上锁、拷贝写进哪里、到点停组时的残留，各会不会出新毛病。
8. 写范围表的新行：三方腿的模式会不会让一条腿写到别的轮、别的腿的报告；八类里有没有定义实际要写、而表里没给的路径（照各定义的「写范围」「产出」现读）；`$SINGLEFS_WORK_ROOT/**` 会不会盖住别的 agent 的草稿目录。
9. 名单一致性那一格：文件头开关表的写法变一下（续行、反引号、同一行两条），会不会漏判或误判。
10. ㉓ 出路的 `clippy_lint_tail`：`check.sh` 读不到、表为空、表的写法变了时，出路给成什么样，会不会给出一条照样被 `project-code-guard.sh` 拒的写法。
11. `dispatch-prompt.py` 改的那一处，派发闸 ④ 读同一张写范围表，现在对八类派发提示里点名的路径判得对不对。

## 分工

辩方腿做第 1 到 4 问，攻方腿做第 5 到 11 问。两条腿都读「范围」里的文件。只有一条腿读到的范围，在报告里写明。要跑的探针不自己跑，写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条。第一轮的两份探针可以拷过去改，在报告里写明改了哪几格。
