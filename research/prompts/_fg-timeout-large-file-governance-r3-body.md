# 前台 timeout 过长与提交前大文件两道执行前拒：治理文件轮第三轮（fg-timeout-large-file-governance-r3，2026-11-19）

前两轮判决是 `research/prompts/fg-timeout-large-file-governance-r1-main-verification.md` 与 `research/prompts/fg-timeout-large-file-governance-r2-main-verification.md`。第二轮两条腿都打中了；用户在 2026-11-19 弹窗里选了「加 git 自己的钩子当主闸」，第二轮判决「三、改法」六条照这个方向改完、换上主工作区。这一轮是这一题的第三轮，也是最后一轮，攻第二轮之后的修补。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 修补做的是不是第二轮判决「三、改法」那六条（正推腿）。
2. 修补本身哪一格会错：git 钩子当主闸之后还剩的绕法与误拒、判法与安装脚本的新毛病、收窄后的 ㉒ 与写范围表（攻方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码，改动也不读 `crates/` 下的任何文件。
- **git 钩子当主闸**：
  - 进仓的钩子 `scripts/gate/git-hooks/pre-commit`、`pre-push`、`pre-merge-commit`（三份都是 bash，没有 python3 就拒，否则 `exec python3 -B scripts/gate/large-file-git-hook.py <钩子名> "$@"`），`git ls-files -s` 三份都是 `100755`。
  - 判法 `scripts/gate/large-file-git-hook.py`：pre-commit 与 pre-merge-commit 先调私有窗口闸 `$(git rev-parse --git-common-dir)/hooks/pre-commit`，再起 `scripts/gate/large-file-check.py --guard commit-index` 判钩子看到的索引；pre-push 逐行读标准输入的 `<本地引用> <本地 sha> <远端引用> <远端 sha>`，把本地 sha 相对「已有的提交」判。远端写成路径或网址时不取任何远端跟踪引用，只认行里的远端 sha。判不了的（超时、退出码不对、JSON 读不成）放行并写一行 `⊘`。文件头有一张表，写 cherry-pick、revert、rebase、am 各起哪个钩子；`git am` 的 pre-applypatch 没接，由 pre-push 兜。
  - `scripts/gate/large-file-check.py`：`--guard` 只剩 `commit-index` 与 `push` 两种；`push` 认 `--ref`、`--known`、`--judge-unknown`、`--remote`；索引拷贝放在调用方给的目录或自己建的临时目录，收到信号也删。
  - 安装与核对 `scripts/gate/git-hooks-install.py`（`install`、`check [--both-machines]`），装没装上的判法共用 `.claude/hooks/lib_git_hooks.py`。`lib_git_hooks.py` 的 `HOOK_NAMES` 只有 pre-commit、pre-push；安装脚本另设 `REQUIRED_HOOK_NAMES` 把 pre-merge-commit 加进去。
  - 私有窗口闸 `.git/hooks/pre-commit`（不进仓）：`SINGLEFS_COMMIT_OUTSIDE_WINDOW=1` 放行窗口，大文件照判，不再靠 `--no-verify`。
  - 门禁 `.claude/gate.d/code-tooling.sh` 新加 git-hooks-installed 格，三份样本在 `.claude/gate.d/fixtures/code-tooling.sh/git-hooks-installed-*/`。
- **检出钩子 `.claude/hooks/bash-command-detector.sh`**（现行 sha256 前缀 b5d024db）：
  - ㉒（文件头第 437 行起，代码第 3405 行起）收窄成两件：绕开 git 钩子的写法（`git commit` / `git push` 的 `--no-verify` 与缩写、`git commit -n`、`-c core.hooksPath=`、`--config-env`、`GIT_CONFIG_KEY_<n>`、`GIT_CONFIG_PARAMETERS`、`git config` 写 `core.hooksPath`），与 git 钩子没装上。读命令文本判内容的那一套、放行名单与经 `~` 取工作根的那一行都删了。
  - 名单一致性改成按门禁开关表的文法认；㉓ 出路的 `clippy_lint_tail` 改调共用模块 `.claude/hooks/lib_code_discipline_lints.py`，`.claude/hooks/project-code-guard.sh` 也改调它。
  - ⑫ 对只带 Read、Bash 的定义写工作根时照写者锁判（`writer_lock_refusal`）。
- **写范围表 `.claude/hooks/agent-write-scope.tsv`**：三方腿按自己那一条腿的报告名收窄（攻方 `*-opus-output.md`、`*-opus-model/**`，辩方与正推 `*-sonnet-output.md`、`*-sonnet-model/**`）。
- **派发闸 `.claude/hooks/runner-dispatch-guard.sh`** ④：第二轮读数里五格误拒，按叙述里点名的路径收窄过。
- **`.claude/agent-common.md`** 第 102 行 ㉒ 那一条照新做法重写；排除表 `.claude/gate.d/shared-data-root-literals-exempt.tsv` 失效的那一行删了；第一轮判决两处字面照实改了。
- **验过的**（都经任务服务跑出，结局退 0 或照预期退 1）：
  - `python3 scripts/gate/git-hooks-install.py check --both-machines`：「✓ /home/fy5090/code/singlefs 里进仓的 git 钩子装好了：core.hooksPath 指到 scripts/gate/git-hooks/，pre-commit、pre-push 与 pre-merge-commit 在、有执行位、进仓的模式是 100755，第二台同样」。
  - `python3 scripts/gate/large-file-git-hook.py --selftest`：「✓ --selftest：28 格都对」「✓ 开关轮：9 个开关逐个打开，各红在点名的格」；九条开关在外面逐个设上各跑一趟，红的格与文件头点名的逐格相同。
  - `python3 scripts/gate/git-hooks-install.py --selftest`：「✓ --selftest：13 格都对」「✓ 开关轮：7 个开关逐个打开，各红在点名的格」；七条开关同样逐个跑过。
  - 检出钩子自证「✓ 自检通过（查了 1486 种）」，开关复跑「复跑了 156 条开关（1 份），自证红且点名的格与实际红的格逐格相同 156 条」。
  - `project-code-guard.sh` 自证「✓ 自检通过（查了 1479 种，其中该拒 332 种）」，开关复跑 147 条逐格相同。
  - 第二轮两份探针改过的副本在 `~/.local/share/singlefs/work/fg-timeout-large-file-r2-fixes/models/`（攻方 `opus-model/probe.py`，辩方 `sonnet-model/probe_defense.py`）。复跑 push 那一组：十五格里十四格 `verdict=passed-nothing-lands`；`push-after-commit-amend-in-same-command` 是 `landed=commit verdict=missed-silently git_hook_refused=yes`，推被拒了，本地留下 amend 那一笔，与第一轮判过不算打中的 commit-amend-keeps-head-big 同一种。辩方 push-after-state-change 那一组四格都是 `verdict=passed-nothing-lands`，其中 merge 之后提交那一格是 `command_exit=1 landed=no git_hook_refused=yes`。
- **主 agent 已知、没改的**（交腿判）：
  - 检出钩子 ㉒ 不拦 `git merge --no-verify`，它照样绕开 pre-merge-commit。
  - ㉒ 判「钩子没装上」只看 `lib_git_hooks.py` 的 `HOOK_NAMES`，不看 pre-merge-commit；`.claude/agent-common.md` ㉒ 那一条、git-hooks-installed 格与 ㉒ 的拒绝说明都只写 pre-commit、pre-push。
  - 两份新脚本文件头写 `# selftest-serial`，格与格读前一格留下的仓状态；`.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」要求每一格单跑与整份跑判定相同。
  - `scripts/gate/git-hooks-install.py` 的 `check_peer` 用 `bash -c PEER_RUNNER` 起 `scripts/hosts/peer-host-lib.sh`，算键的库 `scripts/gate/lib_function_units.py` 的 `LAUNCHED_REPOSITORY_PROGRAMS` 还没登记这条边。登记行的规格草稿在 `~/.local/share/singlefs/work/fg-timeout-large-file-r2-fixes/launch-registry/spec.json`；那份库别的会话还要一起换，换完再以新底重出。
  - `JUDGE_SECONDS` 取 600 秒，没量过；没取回过的远端判全部历史，在 NFS 上可能超时放行。
  - 用户自己手敲的 `--no-verify`、`GIT_CONFIG_GLOBAL` / `GIT_CONFIG_SYSTEM` 指到别处、`GIT_DIR` 改位置，都不在检出钩子里拦。
- 工具员两份报告：`~/.local/share/singlefs/work/fg-timeout-large-file-r2-fixes/result.md`（六条改法）、`~/.local/share/singlefs/work/fg-timeout-large-file-r2-git-hook-gaps/result.md`（推到路径远端与合并提交两处）。工具层的背景（任务服务、换上工具怎么用）在 `.claude/kb/tooling.md`。

## 范围

这一轮被判的文件（都读换上之后的那一版）：

- `scripts/gate/git-hooks/pre-commit`、`pre-push`、`pre-merge-commit`。
- `scripts/gate/large-file-git-hook.py`、`scripts/gate/git-hooks-install.py`、`.claude/hooks/lib_git_hooks.py`：整份。
- `scripts/gate/large-file-check.py`：`--guard` 那一半与文件头。
- `.claude/hooks/bash-command-detector.sh`：文件头 ㉒ 段与代码第 3405 行起的 ㉒、名单一致性（`break_switch_name_sets`、`break_switch_set_mismatches`）、`clippy_lint_tail`、⑫ 的 `writer_lock_refusal` 与它在 `bash_scope_verdict` 里接的那一处，自证里这几样的格。
- `.claude/hooks/lib_code_discipline_lints.py` 整份；`.claude/hooks/project-code-guard.sh` 改调它的那几处。
- `.claude/hooks/agent-write-scope.tsv`：三方腿那几行与八类只带 Read、Bash 的定义的行。
- `.claude/hooks/runner-dispatch-guard.sh`：④。
- `.claude/gate.d/code-tooling.sh`：git-hooks-installed 格与它的三份样本。
- `.claude/agent-common.md`：第 102 行。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（一串 git 操作、一条 hook 输入 JSON、一条命令）或 文件:行。

前两轮已攻过的角度，这一轮不重复：读命令文本判内容的那一套（已删）；别名、启动器、长选项缩写在 ㉒ 里怎么认；push 的 refspec、`push.default`、`--force-with-lease` 各种写法；merge、checkout、reset --hard 之后的 push；名单一致性的十一种表写法；`clippy_lint_tail` 的十一种 lint 表写法。

正推腿（核修补做的是不是第二轮判决说的事）：

1. 第二轮判决「三、改法」六条，逐条写改到了哪一行、做的是不是那一条说的事。
2. git 钩子判的是不是「那一刻真要进提交、真要推出去的内容」：`git commit`（暂存区、`-a`、点名路径、`--amend`）、`git merge` 生成合并提交、`--no-commit` 之后的提交、冲突后 `--continue` 收尾、`git push` 每一行引用，逐种写判法读的是哪一份内容、依据哪一行代码。
3. 安装与核对：两台、链接工作树、`core.hooksPath` 相对路径怎么解、检出老提交的工作树；git-hooks-installed 格与三份样本判的是不是同一件事。
4. 提交窗口：私有闸、`SINGLEFS_COMMIT_OUTSIDE_WINDOW=1` 与 ㉒ 拒 `--no-verify` 三样放在一起，主 agent 日常的提交写法（`.claude/skills/gate/staged-gate.md` 与 `records/` 里近来真用过的）有没有被拒的、有没有绕开大文件判法的。

攻方腿（攻修补）：

5. git 钩子当主闸之后，还有哪些写法能让超限文件进提交或推出去而不被任何一道拦下：`git merge --no-verify`、`git am`、`git commit-tree` 加 `git update-ref`、`git stash` 一类、`GIT_DIR` / `GIT_WORK_TREE`、`GIT_CONFIG_GLOBAL`、子模块、链接工作树检出老提交。逐种写进没进、谁拦的。
6. 误拒：pre-push 对删除引用、只推标签、远端已有的大 blob 历史、远端没有跟踪引用（判全部历史、600 秒）各判成什么；收窄后的 ㉒ 会不会拒掉日常写法。
7. `large-file-check.py --guard push` 只认行里远端 sha 那一支（远端写成路径或网址）：`--known` 的对象本地没有、浅克隆、远端 sha 全 0，各判成什么；索引拷贝在超时、信号、并发提交时会不会留下或写错。
8. ⑫ 写者锁接到只带 Read、Bash 的定义之后，会不会误拒它们写自己的草稿目录、会不会放过写别人认领的目录；三方腿收窄之后的模式还能不能写到别的轮、别的腿的报告。
9. `lib_code_discipline_lints.py` 被检出钩子与 `project-code-guard.sh` 共用之后，有没有一种表写法让两边一起读错（门禁真传给 clippy 的与两边读到的不同）；它的弄坏开关 `LIB_CODE_DISCIPLINE_LINTS_BREAK` 平常设了会怎样。
10. 两份新脚本的自证串行、格读前一格留下的状态：单跑一格与整份跑判定是否相同；开关轮能不能因为次序把一格判绿。

## 分工

正推腿做第 1 到 4 问，攻方腿做第 5 到 10 问。两条腿都读「范围」里的文件。只有一条腿读到的范围，在报告里写明。要跑的探针不自己跑，写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条。第二轮的两份探针可以拷过去改，在报告里写明改了哪几格。
