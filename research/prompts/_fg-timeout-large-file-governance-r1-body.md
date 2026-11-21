# 前台 timeout 过长与提交前大文件两道执行前拒：治理文件轮第一轮（fg-timeout-large-file-governance-r1，2026-11-18）

这一轮判的是检出钩子 `.claude/hooks/bash-command-detector.sh` 新加的 ㉑、㉒ 两道执行前拒，`scripts/gate/large-file-check.py` 新加的 `--guard`，以及 `.claude/agent-common.md` 跟着改的几句。三份都已换上主工作区。

用户原话有两句。第一句是「那条前台命令的超时设成了 600000 毫秒 这个问题 出现了很多次 拦截这个写法」，门槛由用户弹窗定为「超过 240000 就拒」。第二句是「大文件压缩的钩子在提交时候没有生效」：超过 100 MiB 的产物两次进了没推的提交，只能重做那几笔提交。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 换上的实现做的是不是文件头 ㉑、㉒ 两段与 `large-file-check.py` 文件头 `--guard` 那一段说的事（正推腿）。
2. 哪一格会错：该拒的放过、不该拒的误拒、把钩子拖慢或弄挂、拒绝说明给的出路照做不通、文档与代码对不上（攻方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；任务号后面是任务日志里的原句）：

- 这一轮没有改 `crates/` 下的代码，新加的代码也不读 `crates/` 下的任何文件。
- **㉑**：文件头第 418 到 423 行。判法是 `foreground_timeout_refusal`（第 3085 行），门槛是常量 `FOREGROUND_TIMEOUT_MILLISECONDS = 240000`（第 3083 行），拒绝说明由 `foreground_timeout_messages`（第 3098 行）写，出路按主 agent、子 agent 分开写。原来 ③ 只记子 agent 的检出、不拒，那一支连同 `subagent_timeout_findings` 已删。
- **㉒**：文件头第 424 到 443 行。
  - 参数怎么认：`git_commit_form`（第 3128 行）、`git_push_judged`（第 3169 行）、`git_add_form`（第 3186 行）。
  - 起判法：`large_file_check_run`（第 3229 行）起 `<根>/scripts/gate/large-file-check.py --guard <模式>`，时限 `LARGE_FILE_CHECK_SECONDS = 25`（第 3114 行）。
  - 汇总与拒绝说明：`large_file_findings`（第 3254 行）汇总，`large_file_messages`（第 3358 行）写拒绝说明。
  - 自证压小门限：自证经 `selftest_limit_bytes`（第 3219 行）读环境变量 `BASH_COMMAND_DETECTOR_LARGE_FILE_LIMIT_BYTES`，只在自证标记对得上时认。
- **`--guard`**：`scripts/gate/large-file-check.py` 文件头第 26 到 39 行说用法与退出码。
  - 入口是 `guard`（第 377 行）与 `main_guard`（第 395 行）。
  - 三种模式：commit-index（`guard_staged`，第 325 行，跟 `GIT_INDEX_FILE`）、commit-worktree（`guard_worktree`，第 343 行）、push（`guard_unpushed`，第 367 行）。
  - 输出与上限：标准输出只打一行 JSON，上限常量 `SINGLE_FILE_SIZE_LIMIT_BYTES` 与不带 `--guard` 时是同一份。
- **共用约束**：`.claude/agent-common.md` 第 70 行改成主 agent 与子 agent 的前台 timeout 都不超过 240000 毫秒、超过的执行前拒；第 75 行检出钩子那一条的括号里补了「② ㉑ 只在前台拒」；第 96、97 行新加 ㉑、㉒ 两项。
- **中途改过的两处**：
  - 第一处：`large_file_check_run` 原来收的是脚本路径形参，算键的库折不出，开关复跑「140 条里拒算 140 条」（任务 20578）。现在改成收根，起命令那一行写字面尾巴 `os.path.join(script_root, "scripts", "gate", "large-file-check.py")`；在快照树上现算闭包，拒算 0 行（任务 20594）。
  - 第二处：修第一处时，自证一格的格名改了、文件头点名没跟着改，开关复跑「2 处不对」（任务 20600），格名已改回。
- **验过的**：
  - 换上：`large-file-check.py` 是任务 20477；检出钩子是任务 20481、20597、20620，最后一版 sha256 b7627f14…。
  - 门禁与样本档：门禁 code-tooling 的 committed-file-size 格「甲 查了 14496 个跟踪的文件」绿（任务 20478）；code-tooling 样本档「点名的 1 道阶段判得都对（239 个用例）」（任务 20480）。
  - 自证：检出钩子自证「✓ 自检通过（查了 1420 种）」（任务 20599）。
  - 开关复跑：任务 20623「复跑了 139 条开关（1 份），自证红且点名的格与实际红的格逐格相同 139 条」「现判 140、复用 0、拒算 0」。
- **工具员报告里写明的局限**（交腿判该不该算缺口）：
  - git push 只判当前分支相对上游。
  - merge、cherry-pick、rebase、commit-tree 不判。
  - `-a` 取暂存区与工作区的并集，只会多拒。
  - LFS 与 autocrlf 转换后的大小不算。
  - 只含数字的字符串 timeout 当数拒。
  - 工具员报告在 `~/.local/share/singlefs/work/fg-timeout-large-file-guards/result.md`。
- 工具层的背景（任务服务、换上工具怎么用）在 `.claude/kb/tooling.md`。

## 范围

这一轮被判的文件（都读换上之后的那一版）：

- `.claude/hooks/bash-command-detector.sh`：只判文件头 ㉑、㉒ 两段（第 418 到 443 行）、开关表里 ㉑、㉒ 那九条（第 871 到 901 行）、第 3083 到 3370 行的函数、`main()` 里调这两道的地方、自证里「前台超时:」与「提交前大文件:」两组格（第 6530 行、第 8454 行、第 8566 行起）。这份文件里别的会话这几天改的 ⑫、⑭ 与新加的 ㉓ 不在这一轮判。
- `scripts/gate/large-file-check.py`：`--guard` 那一半（文件头第 26 到 39 行，函数第 299 到 425 行）与它和不带 `--guard` 的两半共用的上限、`exceeds`。
- `.claude/agent-common.md`：第 70、75、96、97 行。

这一轮的改动相对 HEAD 的样子：`git diff HEAD -- scripts/gate/large-file-check.py .claude/agent-common.md`；检出钩子那一份的 diff 里混着别的会话的改动，只看「范围」点名的行。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（一条 hook 输入 JSON、一串 git 操作、一条命令）或 文件:行。

正推腿：

1. ㉑ 那一段每一句，逐句对 `foreground_timeout_refusal` 与 `main()` 里的调用：哪些放行、哪些拒，主 agent 与子 agent 的出路各写什么，和别的拒绝一起列的次序。
2. ㉒ 那一段每一句，逐句对 `git_commit_form`、`git_push_judged`、`git_add_form`、`large_file_findings`：-a、路径、-o / -i、排在前面的 git add、`GIT_INDEX_FILE`、不判的那几种、判不了的那几种、出路的字面。
3. `--guard` 三种模式与退出码，逐句对文件头第 26 到 39 行；上限是不是只有 `SINGLE_FILE_SIZE_LIMIT_BYTES` 一份、`exceeds` 是不是同一个函数。
4. `.claude/agent-common.md` 那四行，与钩子今天的行为逐句对得上没有；用户要的两件各落在哪一行，有没有多做的。

攻方腿：

5. ㉑ 漏拒与误拒：timeout 写成浮点、负数、科学计数、带空格或单位的字符串、超大整数时各判成什么；`run_in_background` 写成字符串 "true" 时算前台还是后台；主 agent 照拒绝说明改成 run_in_background 之后，会不会撞上别的道（例：子 agent 的 ⑩、④ 的单独 `&`）。
6. ㉒ 漏拒：造得出「超过上限的文件进了提交或推了出去，㉒ 却放行、也不记检出」的写法吗？至少试这几形：`git commit --amend`、`git commit -F`、`git -c … commit`、`command git commit`、别名（`git ci`）、`git stash` 之后 `git commit`、`git add -u && git commit`、`git push origin HEAD:other`、`git push --all`、`git push --mirror`、子模块、`xargs git commit`。
7. ㉒ 误拒与拖慢：工作区改大了而没暂存的跟踪文件、被忽略的大文件、带空格与非 ASCII 的路径、软链接指到仓外大文件的时候各判成什么。一条命令里有几条 git commit / push 时，判法起几次、合计最长多久；`.claude/settings.json` 里这个钩子注册的时限够不够。对象库很大时 `guard_staged` 与 `guard_unpushed` 要多久。
8. 判法出错与钩子挂掉：`large-file-check.py` 抛 Python 异常、输出多行、退 3、git 不在 PATH 时，钩子是放行加记检出，还是自己抛异常、退别的码。
9. 出路照做通不通：拒绝说明给的 `gzip -n -9 -k`、`git rm --cached`、`git add <文件>.gz` 照抄执行，会不会被项目里管 Bash 的那几道闸拒（⑮ 清索引、⑱ 撤改动、⑰ 字面内容）；做完之后再 `git commit` 是不是放行。
10. 自证与开关：九条开关里有没有两条弄坏同一处、或者点名的格靠别的开关也会红；自证里真 git 仓的格过不过 `scripts/gate/selftest-process-guard.py` 盯的那几样；门限压小的环境变量在平常跑时真的不认吗。
11. 多机：钩子起判法时用的 `repository_root_of` 在仓副本、git worktree、第二台上各读哪一份 `large-file-check.py`。

## 分工

正推腿做第 1 到 4 问，攻方腿做第 5 到 11 问。两条腿都读「范围」里的三份文件；只有一条腿读到的范围，在报告里写明。要跑的探针（临时小仓里的 git 操作序列、喂钩子的 JSON）不自己跑，照「要主 agent 跑的命令」写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条。
