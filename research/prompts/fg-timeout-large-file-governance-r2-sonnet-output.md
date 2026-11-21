# fg-timeout-large-file-governance-r2：辩方腿报告（2026-11-19）

读的是主工作区换上之后的那一版；这一轮没起任何探针，下面「判成什么」都是读码推的，标「推的」，复跑命令在「要主 agent 跑的命令」一节，读数由主 agent 跑出。

开工快照核哈希（开工后又核了一遍，原样）：

```
research/prompts/_fg-timeout-large-file-governance-r2-body.md: OK
.claude/hooks/bash-command-detector.sh: OK
scripts/gate/large-file-check.py: OK
.claude/gate.d/code-tooling.sh: OK
.claude/hooks/agent-write-scope.tsv: OK
scripts/agents/dispatch-prompt.py: OK
.claude/agent-common.md: FAILED
```

`.claude/agent-common.md` 此刻的哈希与快照不同（`git status` 里它是 `MM`，修改日期晚于开工），不是我改的。下面引它的行号都是现用 `grep -nF` 取的此刻的行：㉑ 在第 100 行，㉒ 在第 101 行，「Bash 检出钩子」那一条在第 79 行；背景材料写的第 70、75、96、97 行已是旧行号。

## 一、四问的回答

每问回「成立」「不成立」「推不出」，推不出的算打中，打中给能照喂的输入或 文件:行。

| 问 | 回答 | 一句话 |
|---|---|---|
| 1 坐实的每一格改完关上没有 | 成立（推的）；另有三处同形或相邻的没关，算打中 | 第一轮判决点名的每一格（按判决逐条数，约四十格），都能在现行代码里指到把它关上的那一行；没关的三处：同一条命令里先改状态再 `git push`（推送那一半）、文件头 `large-file-check.py` 一处「说明在标准错误」对不上、判法总时长没有总上限 |
| 2 「不算打中」判得站不站得住 | 部分成立 | Q01 的结论站得住（但判决引的行号是 16，造仓那一行是 14）；NaN、负数、布尔、字符串那几格的「与文件头一致」站得住；400 位整数那一格的理由不成立（文件头说数超过就拒，代码放行是 `float()` 溢出，不是文件头写的那种放行）；认不出的字符串「不是静默放过」字面不准（没记检出），harness 怎么处理字符串推不出 |
| 3 后一批五条做的是不是派发说的事 | 成立（五条各一行，见第五节） | ㉓ 出路、名单三处一致、写范围八类、第 75 行那半句（现在在第 79 行）、`commit -F` 放行格，都有对应的代码与自证格；三处一致我另用没照钩子写法的读法独立数了一遍，相等 |
| 4 「判不准就拒」会不会拒掉日常提交写法 | 不成立（文档里的日常流程没被拒）；有四种写法照拒，一种推不出 | `session-wrapup.md`、`tooling.md`、`main-agent.md` 记的暂存与提交流程都是单独起 `git commit`，不被拒；照拒的：`tee` 写提交说明再 `git commit -F`、前面排着 `xargs`、变量写的重定向目标、前面排着任何不在读不改名单里的脚本；推不出的：`git commit -m "$(cat <<'EOF' … EOF )"`（自证里没有这一形的格），要主 agent 喂 |

## 二、复跑命令与探针的内存与时间

模型目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet/model`（在草稿目录里，不进仓）。里面：

- `probe.py`、`fake_large_file_check.py`：第一轮攻方腿的探针，原样拷来，只改两处（`probe.py` 第 6 行文件头说明与第 712 行 `judged_hashes` 里的快照目录名，从第一轮快照改成这一轮的），其余一字没动；
- `probe_defense.py`：增补三组（推送前先改状态、日常写法、写范围表新行），共 41 格；
- `sonnet-probes/`：第一轮正推腿的三份探针与辅助文件，只改了 12 处期望文字、加了 1 格（第七节逐格写）；
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps` 那一段；`tools/rerun-from-a-clean-work-directory.sh`：包一层，开头删掉那一个运行目录（只删工作根下名字以 `-run` 收尾的目录，别的退 2）。

每个文件的 sha256（`sha256sum` 现算）：

```
94cda200f1c5cc02684dff08a17cf48da4a3e63bd256c36ed2e056b05bfe33b9  model/probe.py
be0b911da7a5491ee0696ee3d5d319aa009ddb5c6e5aae0b144bfab1ac0077df  model/probe_defense.py
b78c53298b49bc054a29ff992b2f35bf934046db3b7979143e92935850f47ff7  model/fake_large_file_check.py
2c672225f21fa1bf7463f179bd29b5542b26a287c2ce9d212df79ac289599145  model/tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  model/tools/rerun-from-a-clean-work-directory.sh
e63ebda7f505ace729bd5a51bfeb7cb9a7eb8c6b4b247f41ff676ef4131d3c2e  model/sonnet-probes/mkjson.py
b437070e55f6cf2903030a7ca5600ab8bd81d4d95a24e09f57a5ddb416fcfe0c  model/sonnet-probes/probe-guard.sh
6efe62601bae70e5fe9709a50870bed62c864b3d55c97979c7a2fe6302281f3e  model/sonnet-probes/probe-hook-git.sh
b0fc14d642dafba7d703fe8fada8c98bd3477fa1e2cd29994e7ef92874c1925b  model/sonnet-probes/probe-lib.sh
f0fc4697a516b68f4ecb89c50f316819f830f1f2f6615caa7df4bbc392cf7256  model/sonnet-probes/probe-timeout.sh
1cb2713a500d320cfe7b5dca1584ab87ae44096547ab563eb831251d491f3d69  tools/names-check.py
485dd957114c5cba2b44d5c39bcabb97c3131cec344827762112560a0ce59d88  tools/names-check-loose.py
```

复跑（一条，整条交服务；仓根同时当被判的树，因为这一轮被判的树就是主工作区）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet -- bash model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet-run /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet/model
```

时间与内存：

- 第一轮攻方腿探针那一份的各组秒数是量的（第一轮运行目录里 `probe.out` 的 `group-seconds-*` 行相加）：timeout 3.0、commit 20.9、push 17.3、false-reject 2.3、remedy 13.2、fake-judge 104.3、limit-variable 0.8、tree 1.6、index-refresh 0.4、main-repository-timing 4.6，合计 168.4 秒；`fake-judge` 那一组占 104 秒，是假判法睡到钩子的 25 秒时限、一条命令四次。
- 增补三组的秒数没量过，推的：造仓与真 `git commit`、`git push` 五十来格，每格秒级，合计一两分钟；第一轮正推腿三份探针第一轮的 `用时` 列每格 0.1 到 1.7 秒，三份合计十几秒（量的，第一轮 `out-*.txt`）。
- 磁盘是量的：第一轮攻方腿那份运行目录 `du -sh` 是 471M，装置里的超限文件是稀疏文件，只有不可压缩那一格写 100 MiB 随机字节与它的 `.gz`；增补三组不造超限文件之外的大东西，仍是同一个量级，推的。
- 内存没量过，推的：`git add` 一个 100 MiB 的稀疏文件时 git 把它读进内存，峰值几百 MiB 以内；整份探针串行跑，不并发。服务给的内存上限由它定，这一份不写上限。
- 跑完运行目录（`...-sonnet-run`）由主 agent 删，用前 `du -sh` 记大小；下一趟重跑时包装脚本先删它。

## 三、第 1 问：第一轮判决「二、判决」里坐实的每一格，现在判成什么

「现在」一列都是读码推的，待主 agent 复跑（第八节命令）。自证里有同形格的，写自证里那一格的区间；自证由主 agent 报「查了 1493 种」通过，我没跑。行号都是现用 `grep -nF` 取的此刻的行。

| 第一轮判决里的格 | 现在判成什么（推的） | 把它关上的行 | 自证里的同形格 |
|---|---|---|---|
| 别名：commit-repository-alias、commit-command-line-alias、正推腿 `git ci` | 别名展开成 `commit`，照暂存区判，超限拒 | 「subcommand, arguments, problem = resolved_git_subcommand(global_options, subcommand, arguments, command_at.directory, environment)」（`.claude/hooks/bash-command-detector.sh:3752`）；仓里配置的别名由 `.claude/hooks/lib_shell_words.py:4124` 的 `git_config_commands` 读 | `.claude/hooks/bash-command-detector.sh:9364-9438` 里「仓里配的别名」「命令行 -c alias.ci=commit」两格 |
| commit-command-line-shell-alias（`!git commit`） | 拒，原因类 shell-alias，出路写「写出 git 原命令」 | 「if kind == "alias":」（`.claude/hooks/bash-command-detector.sh:3780`） | 同区间「命令行 -c 写的 ! 开头的 shell 别名」 |
| commit-through-xargs、commit-through-find-exec、正推腿 `xargs git commit` | 拒，原因类 dynamic-launcher | 「if step.dynamic:」（`.claude/hooks/bash-command-detector.sh:3790`）；剥启动器在 `.claude/hooks/lib_project_code_runs.py:2098` 的 `command_under_extra_launcher`（xargs、`if name == "find":` 在 `.claude/hooks/lib_project_code_runs.py:2143`） | 同区间「echo \| xargs git commit」「find -exec git commit」 |
| commit-abbreviated-message-option、-include-option、-force-add（`--mes`、`--inc`、`--forc`） | 缩写当全名认：`--mes` 跳下一个词、`--inc` 当 `--include`、`--forc` 当 `--force` | 「def expand_long_option(word, table):」（`.claude/hooks/bash-command-detector.sh:3369`） | 同区间三格 |
| commit-after-stash-pop、-stash-apply-index、-cherry-pick-no-commit、-checkout-path-from-branch、-merge-squash、-redo-after-reset-soft、-file-created-in-same-command、-all-file-grown-in-same-command、-repository-made-in-same-command（九格） | 全拒，原因类 preceding-change，出路「先单独起前面的命令，再单独起 `git commit`」 | 「if subcommand and subcommand not in GIT_FIXED_SUBCOMMANDS and subcommand not in GIT_READ_ONLY_SUBCOMMANDS and not mutator:」（`.claude/hooks/bash-command-detector.sh:3759`）记下第一条会改内容的命令，「if step.mutator and not detector_break_is_set("preceding-mutator-unseen"):」（`.claude/hooks/bash-command-detector.sh:3793`）拒 | 同区间 `git stash pop`、`cherry-pick -n`、`checkout 分支 -- 路径`、`merge --squash`、`reset --soft`、`truncate`（两格）、新建仓各一格 |
| commit-relative-private-index-kept-at-top | 相对路径的 `GIT_INDEX_FILE` 原样交给判法，判法按顶层目录解，判到真正被 git 用的那份索引，超限拒 | 「extra_environment["GIT_INDEX_FILE"] = (os.path.normpath(os.path.join(directory, os.path.expanduser(index_file)))」（`.claude/hooks/bash-command-detector.sh:3837`，只有弄坏开关下才相对命令目录解）；判法一侧 `def effective_index_path(top):`（`scripts/gate/large-file-check.py:467`） | 同区间「相对路径的 GIT_INDEX_FILE」 |
| commit-path-from-subdirectory-diff-relative-config | 判法起的每条 git 都带 `-c diff.relative=false`，用户配置不影响 | 「configuration += ["-c", "diff.relative=false"]」（`scripts/gate/large-file-check.py:96`） | 同区间「diff.relative=true 的仓里」 |
| push-named-other-branch、push-all-branches、push-mirror、push-tags | 照推出去的引用判：点名分支取它、`--all`、`--mirror`、`--tags` 各取各的，相对 `--not --remotes=<远端>`，有超限 blob 拒 | 「push_form = git_push_form(arguments)」（`.claude/hooks/bash-command-detector.sh:3805`）；判法 `def guard_pushed(top, limit, options):`（`scripts/gate/large-file-check.py:452`） | 同区间 `push origin feature`、`--all`、`--mirror`、`--tags` 四格 |
| push-recurse-submodules | 拒，原因类 push-submodules | 「if push_form.recurse_submodules in ("on-demand", "only") and not detector_break_is_set("push-recurse-submodules-unseen"):」（`.claude/hooks/bash-command-detector.sh:3811`） | 同区间 `--recurse-submodules=on-demand` |
| 正推腿 `git add --pathspec-from-file`、变量路径、命令替换路径再 `git commit` | 拒，原因类 add-path | 「add_form.unjudged and not detector_break_is_set("add-paths-unjudged-silent")」（`.claude/hooks/bash-command-detector.sh:3873`） | 同区间三格 |
| 正推腿 `cd` 到算得出而此刻不存在的目录 | 放行并记一条检出 | 「unjudged(shown, "命令所在的目录此刻不存在")」（`.claude/hooks/bash-command-detector.sh:3825`） | `.claude/hooks/bash-command-detector.sh:9439-9449` 一带「命令所在的目录算得出而此刻不存在」 |
| 误拒：正推腿 `git add new.bin && git commit -m x small.txt` | 放行：点名路径的提交不判排在前面的 `git add` | 「chain_matters = not commit_paths or commit_form.include_staged」（`.claude/hooks/bash-command-detector.sh:3870`） | 同区间「git commit 点名的路径之前 git add 别的路径」 |
| 误拒：push-clean-side-branch-while-current-dirty、push-delete-other-ref-while-current-dirty | 放行：推的是干净的分支；只删远端引用不判 | 「if not push_form.judged or (push_form.only_deleting and not detector_break_is_set("push-current-branch-only")):」（`.claude/hooks/bash-command-detector.sh:3806`） | 同区间 `push origin side`、`push origin :stale` |
| remedy-chained-in-one-command | 仍拒（判决说拒得对），现在的原因类是 preceding-change（前面排着 `gzip`），出路写单独起 | 同 `.claude/hooks/bash-command-detector.sh:3759`、`.claude/hooks/bash-command-detector.sh:3793` | 同区间「出路的 gzip、git rm --cached、git add 与 git commit 写在同一条命令里，拒」 |
| judge-fails-keys-missing（判法 JSON 缺字段，该拒的 ㉑ 也被放过） | 放行 ㉒ 并记一条检出；同一条命令里 ㉑ 照拒（hook_exit 2、说明里有「前台命令的 timeout」） | 「if not large_file_result_valid(result):」（`.claude/hooks/bash-command-detector.sh:3724`）；入口里 ㉒ 与 ㉑ 各自包 try：「large_files = large_file_findings(tool_input.get("command") or "", command_directory, repository_root_of(hook_dir), selftest_limit_bytes())」（`.claude/hooks/bash-command-detector.sh:10369`）、「long_timeout = foreground_timeout_refusal(hook_input)」（`.claude/hooks/bash-command-detector.sh:10092`） | `.claude/hooks/bash-command-detector.sh:9449` 是函数一层的格；入口一层「超门限 + 缺字段 + 前台 timeout」三样一起的格没有，由复跑那一格补 |
| judge-fails-slow-with-child（`leftover_after_hook=4`） | 到点对判法自己的进程组发 SIGKILL，`leftover_after_hook=0` | 「start_new_session=own_group」（`.claude/hooks/bash-command-detector.sh:3701`）、「os.killpg(process.pid, signal.SIGKILL)」（`.claude/hooks/bash-command-detector.sh:3707`） | `.claude/hooks/bash-command-detector.sh:9502` 一格（判法超时被停时连它起的子进程一起停） |
| index-written-back-by-judge（含 `GIT_OPTIONAL_LOCKS=0` 那一格） | 用户索引一个字节不变：判法读索引的拷贝 | 「shutil.copyfile(source, copy)」（`scripts/gate/large-file-check.py:488`）、「with private_index(top):」（`scripts/gate/large-file-check.py:506`） | `.claude/hooks/bash-command-detector.sh:9476` 一格（判法不写用户的暂存区索引文件） |
| remedy-incompressible-gzip-size（压完仍超） | 拒绝说明另写一条路：拆成每份不超过上限的分片入库（`split -b 90M`，90 MiB 小于 100 MiB 上限）或不入库 | 「LARGE_FILE_NO_GZIP_WAY = (」（`.claude/hooks/bash-command-detector.sh:3323`），拼进出路在 `.claude/hooks/bash-command-detector.sh:3935` | 出路本身要过的别的闸那几格；这一条里的 `split`、`.gitignore` 不是 cargo 命令，别的闸不拒（推的） |
| remedy-push-message（没写重做命令） | 写整套重做命令，逐条单独起，第一条 `git reset --soft <远端已有的最新提交>` | 「if pushed:」（`.claude/hooks/bash-command-detector.sh:3937`）、`.claude/hooks/bash-command-detector.sh:3939` | `.claude/hooks/bash-command-detector.sh:9502-9540` 一带 stdin 里「git push 拒绝，出路给重做没推的提交的整套命令」 |
| 文档：退出码表漏 78 | 表里列了 78 | 「78 git 不在 PATH」（`scripts/gate/large-file-check.py:48`）；78 是 `.claude/singlefs-ai-sop/scripts/preflight.py:48` 的 `EXIT_REFUSED` | 复跑 Q16 那一格应是 `rc=78` |
| 文档：`--limit-bytes ²` 退 1、不打 JSON | 退 2、标准输出空（`--guard` 下）：值只认 ASCII 十进制 | 「return int(text) if re.fullmatch(r"[0-9]+", text) and int(text) > 0 else None」（`scripts/gate/large-file-check.py:108`）、「if argument == "--limit-bytes" and rest and positive_integer(rest[0]):」（`scripts/gate/large-file-check.py:525`） | 复跑 Q14 那一格应是 `rc=2`、`stdout 行数=0` |
| 文档：77 那半句在 `--guard` 下走不到 | 文件头改成「--guard 下只有第一种」 | `scripts/gate/large-file-check.py:47`；`--guard` 下目录不在仓里退 77、空仓退 0（`scripts/gate/large-file-check.py:501-503`） | 复跑 Q21 那一格应是 `rc=0` |
| 文档：㉑「只含数字的字符串」带空白也拒 | 文件头改写成正则，并写明 `\d` 含别的文字的数字 | 「字符串只认「前后可带空白、中间只有数字（可带一个小数点）」的那一种」（`.claude/hooks/bash-command-detector.sh:423`） | 复跑 timeout 字符串带空白那一格应是 `refused_21=yes` |
| 文档：`agent-common.md` 第 97 行没写出点名路径时暂存区不判 | 现在写了 | 「点名路径时只判那几个路径的工作区内容、暂存区不判（带 `-i` 才判）」（`.claude/agent-common.md:101`） | 读文字，没有探针 |

### 没关上的三处（算打中）

1. **同一条命令里先改状态再 `git push`，推送这一半没关。** 判决改法第 1 条写的是「提交、推送那一刻判不准内容的写法，执行前拒」，文件头总判法也写「提交、推送那一刻」（`.claude/hooks/bash-command-detector.sh:434`），但「排在前面会改工作区或暂存区的命令」只在 commit 那一支查：commit 一支有「if step.mutator and not detector_break_is_set("preceding-mutator-unseen"):」（`.claude/hooks/bash-command-detector.sh:3793`），push 一支（`.claude/hooks/bash-command-detector.sh:3805`-`.claude/hooks/bash-command-detector.sh:3819`）没有这一查：判法在起命令之前解 `HEAD` 与各引用，`merge`、`checkout` 之后 `HEAD` 指到哪里判不到。能照喂的输入（读码推的，待复跑；探针里是三格，格名在第七节）：仓里 `main` 跟着 `origin/main`、什么都没欠，分支 `feature` 有一笔没推的提交加了 100 MiB 加 1 字节的文件，停在 `main` 上，喂钩子 `git merge -q --no-edit feature && git push -q`（或 `git checkout -q feature && git push -q origin HEAD`、`git reset -q --hard feature && git push -q`）：应当 `hook_exit=0`、检出 0 条、远端里多出超限 blob。判决里这一半没有格，所以它不算「坐实的格没关上」，算同形没关。代价：把 `.claude/hooks/bash-command-detector.sh:3793` 那一查加到 push 一支，会拒 `git pull --rebase && git push`；HEAD 的 reflog 里一条 `pull`、`rebase` 都没有（`git reflog --format=%gs` 数过，见第六节），拒它的实际代价小。推翻它：那三格复跑 `verdict` 是 `rejected`。
2. **`large-file-check.py` 文件头「2 用法错……说明与出路在标准错误」对不带 `--guard` 的那一支不实。** 带 `--guard` 的那一支把说明打在标准错误（`scripts/gate/large-file-check.py:546`）；不带 `--guard` 的那一支「print(f"  ✗ 认不得的参数：{argument}")」（`scripts/gate/large-file-check.py:575`）打在标准输出。`--limit-bytes ²` 不带 `--guard` 时退 2、标准输出两行、标准错误空，与文件头不符。喂：`python3 scripts/gate/large-file-check.py <仓根> --limit-bytes ²`。小，只是文字对不上；推翻它：复跑的 Q14b 那一格 `stdout 行数` 是 0。
3. **判法总时长没有上限，钩子被外面停时判法那一组不跟着停。** 每次起判法限 25 秒（「LARGE_FILE_CHECK_SECONDS = 25」，`.claude/hooks/bash-command-detector.sh:3281`），一条命令里几次不同的判法各起一次：`git add a && git add b && git commit -a` 是四次，第一轮的 `judge-runs-two-adds-then-commit-all` 量到 `judge_runs=4`，假判法睡死时 `hook_seconds=100.23`，现在仍是四次各 25 秒，合计约 100 秒（推的）。钩子注册没有 `timeout` 字段（`.claude/settings.json:83`），超过 Claude Code 自己给 hook 的时限会怎样仓里判不出（推不出）。另：判法起在自己的会话里（`.claude/hooks/bash-command-detector.sh:3701`），到点钩子自己杀它那一组；而钩子被外面杀掉时（不是到点），那一组不在钩子的进程组里，不会跟着死（推的，没造）。现象推翻它：复跑 `judge-fails-slow-with-child` 的 `hook_seconds` 小于 30。

## 四、第 2 问：第一轮判决里「不算打中」那几条

读数取自第一轮运行目录里的 `out-*.txt` 与 `probe.out`（原样，第一轮留着，不是这一轮跑的）。

| 判决里的条目 | 辩方判定 | 证据 |
|---|---|---|
| 正推腿 Q01 的预期写错，`checked 0` 是对的 | 站得住；判决引的行号不准 | 第一轮 `probe-guard.sh` 的原文：`14:w=$(mkrepo q01); mkbig "$w/big.bin"; git -C "$w" add big.bin; echo s > "$w/ws.bin"; git -C "$w" add ws.bin; git_p -C "$w" commit -qm ws; git -C "$w" push -q 2>/dev/null` 与 `16:g Q01 "1，too_big 里 big.bin（暂存区）" $PY "$CHECK" "$w" --guard commit-index`：造仓提交并推 big.bin 的是第 14 行，判决写的第 16 行是调用 Q01 的那一行。`out-guard.txt` 第 1 行 `CASE Q01 … rc=0`、第 2 行 JSON 里 `"checked": 0, "too_big": []`，与「暂存区相对 HEAD 没有新增」一致 |
| T14：NaN 拒 | 站得住 | 文件头「含 Infinity、NaN 两个词」（`.claude/hooks/bash-command-detector.sh:421`）；代码里 NaN 不满足 `value <= 240000`，走到返回值（`.claude/hooks/bash-command-detector.sh:3261`、`.claude/hooks/bash-command-detector.sh:3263`）；`out-timeout.txt` 里 NaN 那一格 `rc=2`、说明写「nan 毫秒」 |
| T15：400 位整数放行并记检出，与文件头一致 | 结论站得住（放行、记了 1 条检出，不是静默）；理由不成立，算打中（窄） | 文件头说数超过就拒（`.claude/hooks/bash-command-detector.sh:421`、`.claude/hooks/bash-command-detector.sh:422`：「而超过 FOREGROUND_TIMEOUT_MILLISECONDS（240000 毫秒）」），400 位整数是 JSON 整数、超过 240000，应当拒；代码在「return int(value) if float(value).is_integer() else value」（`.claude/hooks/bash-command-detector.sh:3263`）里 `float()` 溢出，被入口的 try 接住（`.claude/hooks/bash-command-detector.sh:10092`），变成「没判成前台命令的 timeout（OverflowError('int too large to convert to float')）」、照常执行。判决引的文件头那句「写成布尔或认不出的字符串的……放行」说的是布尔与字符串，不含整数。能照喂：hook 输入 `{"tool_name":"Bash","tool_input":{"command":"echo hi","timeout":1` 后接 400 个 `0` `}}`；读到 `hook_exit=0`、检出 1 条即推翻文件头。实际 harness 不会发这样的整数，所以只是文字与代码对不上，不是日常会撞的洞；修法一行：先比大小再转浮点 |
| timeout-string-exponent 等认不出的字符串放行，「没有静默放过」 | 与文件头一致站得住；「没有静默放过」字面不准；harness 怎么处理字符串推不出 | 文件头把 `"3e5"`、`"300000ms"`、`"+300000"`、`"0x493e0"`、`"300000."` 列为放行（`.claude/hooks/bash-command-detector.sh:424`）；`probe.out` 里这几格是 `hook_exit=0 refused_21=no recorded=no`，即退 0、不记检出，按判决自己给 ㉒ 定的「静默放过」字义就是静默。放行是文件头写明的取舍，不是新缺陷；要不要把这几种也记一条检出，取决于 harness 会不会把字符串当数，这一点不在仓里（推不出）。喂：`"timeout":"3e5"` |
| commit-amend-keeps-head-big：大 blob 已在 HEAD，commit 不带进新内容，push 那一刻判 | 站得住 | `commit-index` 只看暂存区相对 HEAD 新增或改过的 blob（`scripts/gate/large-file-check.py:357`-`scripts/gate/large-file-check.py:372` 的 `guard_staged`），amend 不改树时没有新增；超限 blob 在没推的提交里，由 push 一支判（`scripts/gate/large-file-check.py:452` 的 `guard_pushed`，`rev-list --objects` 相对 `--not --remotes`）。分工在文件头写着：「分工是：那一格在整轮门禁里判已经进了树与暂存区的内容，这一道在提交与推送那一刻判」（`.claude/hooks/bash-command-detector.sh:433`）。前提：这笔提交没有绕过钩子（`commit-tree` 之类）进到 HEAD |
| tree-hook-from-linked-worktree、judge-without-git-on-path：真判不了、放行并记检出 | 站得住 | `probe.out` 第 111、112 行：`PROBE name=tree-hook-from-linked-worktree question=11 hook_exit=0 rejected=no recorded=yes …`、`PROBE name=judge-without-git-on-path question=8 hook_exit=0 rejected=no recorded=yes …`（行内后面是 `command_exit=not-run landed=no`，我只贴到 `recorded=yes`） |
| `run_in_background: "true"` 这类写法 harness 怎么处理，仓里判不出，钩子这一侧按前台算 | 钩子这一侧站得住；harness 一侧推不出 | 文件头「`run_in_background` 只认 JSON 的 true，字符串 `"true"` 当前台」（`.claude/hooks/bash-command-detector.sh:425`）；`probe.out` 里 `timeout-background-string-true-main` 是 `refused_21=yes`。harness 把字符串 `"true"` 当真还是当假，不在仓里 |

## 五、第 3 问：后一批五条做的是不是派发说的事

派发说的五条（调度记录里写的）：㉓ 的出路带 lint 表、开关名单与文件头开关表与代码读的名字三处一致、写范围表补只带 Read 与 Bash 的八类定义并让 code-tooling 的格跟着改、`agent-common.md` 第 75 行那半句、自证补「提交说明先重定向写到仓外再 `git commit -F`」放行那一格。五条回答都是「成立」，各自的依据：

| 条 | 回答 | 依据与现象推翻它的条件 |
|---|---|---|
| ㉓ 出路带 lint 表 | 成立 | `clippy_lint_tail` 现读 check.sh 里 `CODE_DISCIPLINE_LINTS=(` 那一段（「def clippy_lint_tail(check_script):」`.claude/hooks/bash-command-detector.sh:4047`；表在 `.claude/singlefs-ai-sop/scripts/check.sh:64`），`whole_target_build_messages` 把它接在出路的 `cargo clippy` 后面（「lint_tail = "" if lints_dropped else clippy_lint_tail(os.path.join(repository_root_of(sys.argv[1]), CLIPPY_LINT_TABLE_RELATIVE))」`.claude/hooks/bash-command-detector.sh:4082`）。要被它放行的闸 `project-code-guard.sh` 读同一张表（「def code_discipline_lints():」`.claude/hooks/project-code-guard.sh:738`），起止判据与每行正则两边是同一串（`CLIPPY_LINT_TABLE_START`、`CLIPPY_LINT_TABLE_LINE`），所以出路按构造过得了那一道；表读不到或为空时，出路不带那一段、说明里写明要照 check.sh 补全并记一条判不了，不会给出一条照样被拒的写法。两份读法各写一份（闸里是内嵌 python 导不进来），哪天一边改了另一边没改会漂，钩子头上有 `gate-similar` 注明。推翻它：自证「出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo clippy 写法喂 project-code-guard.sh 放行」红。观察（不在这五条里）：`.claude/agent-common.md` 的「执行前拒绝的写法」列到 ㉒ 为止，没有 ㉓ 的条目（`grep -c '㉓' .claude/agent-common.md` 是 0），㉓ 只在钩子文件头里 |
| 名单、文件头、代码读的名字三处一致 | 成立 | 自证从钩子源文件现读三处（「def break_switch_name_sets(source):」`.claude/hooks/bash-command-detector.sh:1691`、「def break_switch_set_mismatches(source):」`.claude/hooks/bash-command-detector.sh:1709`）。我另用没照钩子正则写的读法独立数了一遍（`tools/names-check.py`、`tools/names-check-loose.py`，文件头 sha256 见第二节），原样输出：`known 155 header 135 read 155` 与 `known 155 文件头里完全没出现的名字 []`。名单 155 与代码读的 155 逐名相等（`known-read 只在前者 [] 只在后者 []`）；文件头开关表我只认 `#   BASH_COMMAND_DETECTOR_BREAK=<项>` 起头的行时是 135，另 20 个名字写在 `#   <项>  说明` 起头的行里（钩子的正则认这两种行首），宽读法下 155 个名字在文件头里都出现。所以「三处一致」成立，但「文件头开关表」一处的判定靠这两种行首写法，行首写法变了会不会漏判，是攻方腿第 9 问的事，我没判。推翻它：自证「弄坏开关三处一致:这份钩子的名单、文件头开关表、代码里读的名字两两包含」红 |
| 写范围表补八类定义 | 成立 | 表里十四个定义每个至少一行（`grep -c "^<名>\t"` 数过：clerk 4、experiment-designer 3、experiment-runner 18、implementation-writer 3、investigator 3、kb-scribe 4、mutation-triage 1、prior-art 1、sweep 1、three-way-attack 4、three-way-defense 4、three-way-forward 4、three-way-materials 7、tooling-writer 15）。八类行与各自定义「写范围」一节逐条对过：experiment-designer 写 `e<号>-preregistration.md`、`e<号>-r<n>-prereg.md` 与草稿目录（定义第 67 行）对应表里三行；mutation-triage、prior-art、sweep 三份都写「报告文件、草稿目录」，表里各一行工作根；三方腿三份写「报告文件、模型目录、草稿目录」，表里各四行（`research/prompts/*-*-output.md`、`*-*-model/**`、`*-*-model`、工作根，如「three-way-defense	research/prompts/*-*-output.md」`.claude/hooks/agent-write-scope.tsv:65`）；材料员写小节清单、附录、背景材料、diff 材料与快照里两份清单，表里六行加工作根，正文不在里面（表里没有 `_*-body.md` 这一行）。code-tooling 的格改成每个定义都要有行（「AGENT_WRITE_SCOPE_BREAK=readers-unchecked  agent-write-scope 只要求有 Write 或 Edit」`.claude/gate.d/code-tooling.sh:773` 是它的弄坏开关，判在 `.claude/gate.d/code-tooling.sh:4062`-`4063`）。八类行只把「表里没有行就一概不判」收窄成「按定义的写范围判」，没有新放宽任何一条腿原来能写的路径。偏宽的两处主 agent 已知：`*-*-output.md` 让一条腿写得到任何一轮、任何一条腿的报告（表里这一行的文字就是这样，我这一轮的写范围也正是这几行）；`$SINGLEFS_WORK_ROOT/**` 与六个有 Write 的定义一样盖住整个工作根，包括别的 agent 的草稿目录与冻结副本，而定义里写「冻结副本只读」，表判不到。实际用过：这一轮我用 Bash 往草稿目录与 `research/prompts/` 下的报告写，一路没被拒。推翻它：自证 ⑫ 里「Bash写范围:只读定义:…」九格有一格不拒或八格放行有一格拒 |
| 第 75 行那半句 | 成立（现在在 `.claude/agent-common.md:79`，旧行号已移位） | 文字「② 前台拒、run_in_background 里只拒轮询本会话后台任务输出文件的循环，其余几处都拒」（`.claude/agent-common.md:79`）逐项对代码：㉑ 只在前台（「if tool_input.get("run_in_background") is True or detector_break_is_set("allow-long-foreground-timeout"):」`.claude/hooks/bash-command-detector.sh:3256` 返回不判）；⑩ 只在 run_in_background 里（「return bool(agent_type) and run_in_background is True」`.claude/hooks/bash-command-detector.sh:4110`）；② 在 run_in_background 里只判轮询后台任务输出的循环（「return background_task_output_wait_loops(command)」`.claude/hooks/bash-command-detector.sh:4121`）。没有配开关，只改文字，与派发一致 |
| `commit -F` 放行格 | 成立；`tee` 写提交说明仍拒，派发没要求它 | 判法没改，只加格：`harmless_write_target`（`.claude/hooks/bash-command-detector.sh:3569`）把重定向写到 `/tmp/`、`/var/tmp/`、`/dev/`（「HARMLESS_WRITE_PREFIXES = ("/tmp/", "/var/tmp/", "/dev/")」`.claude/hooks/bash-command-detector.sh:3315`）与工作根下的目标当不改仓里内容；带 `$`、反引号或相对路径的目标算不出、当会改（「if "$" in path or "`" in path or not os.path.isabs(path):」`.claude/hooks/bash-command-detector.sh:3575`），由 `command_changes_nothing` 逐条重定向判（`.claude/hooks/bash-command-detector.sh:3587`-`.claude/hooks/bash-command-detector.sh:3590`）。新加三格：`printf … > /tmp/… && git commit -F`、heredoc 写 `/tmp` 再 `git commit -F`（`.claude/hooks/bash-command-detector.sh:9455` 起的循环）、走真实入口的一格（`.claude/hooks/bash-command-detector.sh:9528` 一带），都要求 0 条拒、不记检出，开关 `commit-message-redirect-judged-as-change`（`.claude/hooks/bash-command-detector.sh:3572`）下红；已有的那一格在 `.claude/hooks/bash-command-detector.sh:9411`。`tee` 不在读不改名单里（「READ_ONLY_COMMAND_NAMES = frozenset」`.claude/hooks/bash-command-detector.sh:3311` 起那张表里没有 `tee`），所以 `printf … \| tee /tmp/… && git commit -F` 仍被拒，后一批报告写明没动 |

另：后一批多换了一份清单外的文件 `scripts/agents/dispatch-prompt.py`，`git diff HEAD -- scripts/agents/dispatch-prompt.py` 里与这件事有关的只有两处：自证拿来当「表里没有行的类型」的样本从 `sweep` 换成不存在的 `no-row-sample`（「unlisted_scope_lines = [line for line in skeleton("no-row-sample", scope_agents, {}, None).split("\n") if line.startswith(f"{WRITE_SCOPE_LABEL}：")]」`scripts/agents/dispatch-prompt.py:770`）与一句文档串；表补了 `sweep` 的行之后，不换样本那一格必红，这一处与后一批的说明一致。diff 里别的改动（引用的记录文件名）不是这一批的，不判。

## 六、第 4 问：「判不准就拒」会不会拒掉主 agent 日常的提交写法

回答：不成立（文档里记的、records 里用过的流程，没有一条被拒）；有四种写法照拒，其中一种是误拒的明确输入；有一种最常见的写法（多行提交说明）自证里没有格、读码推它放行，算推不出，要主 agent 喂。

先说证据的边界：`staged-gate.md` 里没有一条字面的 `git commit`、`git push`；`records/` 里字面的 `git commit`、`git push` 很少（下表逐条列了）；最近三笔提交的日期是 2026-11-17，检出钩子与判法脚本的修改日期是 2026-11-18，所以目前没有一笔提交是在现行的「判不准就拒」之下发生的，没有真实使用的读数，只有读码。HEAD 的 reflog 是另一类证据（本地，量的）：

```
$ git reflog --format=%gs | sed -E 's/[: ].*//' | sort | uniq -c | sort -rn | head -8
    345 commit
      9 reset
      5 merge
      3 
      2 rewrite
      2 checkout
      1 split
      1 drop
$ git reflog --format=%gs | wc -l
368
$ git reflog --format=%gs | grep -c 'commit (amend)'
0
$ git reflog --format=%gs | grep -cE '^(pull|rebase)'
0
```

368 条里 345 条是 `commit`，没有 `amend`、`pull`、`rebase`；9 条 `reset` 是 `reset: moving to HEAD`（清暂存区那一类，归 ⑮）；`merge` 五条是快进。所以日常写法的大头是单独或接在 `git add` 后面的 `git commit`。

| 写法与出处 | 现在判成什么（推的） | 依据 | 日常？ |
|---|---|---|---|
| 暂存经 `task-server/task submit -- python3 scripts/gate/stage-mine.py …`，跑完门禁，再单独起不带路径的 `git commit`（`.claude/main-agent.md:96`「主 agent 经 `task-server/task submit -- python3 scripts/gate/stage-mine.py …` 暂存」；`.claude/kb/tooling.md:211`「暂存完、提交前再核一遍名单与逐文件 cached diff，然后不带路径地 `git commit`」；`.claude/rules/session-wrapup.md:40`） | 放行：单独的 `git commit` 判暂存区，没有超限就不拒 | 前面没有命令，`mutator` 为空（`.claude/hooks/bash-command-detector.sh:3759` 不触发）；判暂存区相对 HEAD（`scripts/gate/large-file-check.py:357` 的 `guard_staged`） | 是，文档定的流程 |
| 看暂存区：`git diff --cached --name-only`、`git diff --cached -U0 -- <文件>`（`.claude/skills/gate/staged-gate.md:23`） | 不判：没有 commit、push | 判法只在命令位置上有 `git commit`、`git push` 时才起 | 是 |
| 门禁前核工作区等于暂存区：`git diff --quiet -- <这些路径>`、`git ls-files --others --exclude-standard -- crates`，与带 `\| xargs -0 -r sha256sum \| sha256sum` 的指纹（`.claude/skills/crash-test/commit-stages.md:16`、`:44`） | 不判：没有 commit、push；`xargs` 在 commit 之前才算前置改动，这里没有 commit | 同上 | 是 |
| 共用仓里的临时提交对象与挪引用：`GIT_INDEX_FILE=<草稿>/sync-index git read-tree HEAD`、`git write-tree`、`git commit-tree <树> -p HEAD`、`git update-ref`、`git read-tree`（`records/2026-10-23-subagent拆分提案.md:573`；`.claude/kb/tooling.md:214` 一行里的「主仓里 `git update-ref refs/heads/master <新提交> <旧 HEAD>` 再 `git read-tree <新提交>`」） | 不判：子命令是 `commit-tree`、`update-ref`、`read-tree`，不是 `commit`、`push` | 「GIT_FIXED_SUBCOMMANDS = ("commit", "push", "add")」（`.claude/hooks/bash-command-detector.sh:3304`）按整词认 | 是，多会话共用仓时 |
| 删远端分支：`git push origin --delete <分支>`（`records/2026-10-13-里程碑四预开发.md:425`） | 不判：删除引用不传对象 | 「if not push_form.judged or (push_form.only_deleting and not detector_break_is_set("push-current-branch-only")):」（`.claude/hooks/bash-command-detector.sh:3806`） | 偶尔 |
| 重做提交：`git reset --soft <基>` → `git rm --cached --ignore-unmatch -- <文件>` → `gzip -n -9 -k <文件>` → `git add <文件>.gz` → `git commit -m …` → 再推，逐条单独起（㉒ 自己的出路；E158 那次的 reflog 里 `drop two >100MB e158 outputs from unpushed commits`） | 每一条单独起都放行；串在一条命令里拒 | `git reset --soft` 后面没有 `git commit`，没有判的对象；串起来时 `.claude/hooks/bash-command-detector.sh:3793` 拒，出路说明里写「逐条单独起」 | 偶尔 |
| 多行提交说明：`git commit -m "$(cat <<'EOF' … EOF )"`（最近三笔提交的说明都是多行加 `Co-Authored-By` 尾行，`git log -3 --format=%B` 看到的） | 推不出，读码推放行 | `.claude/hooks/lib_shell_words.py:548` 的 `strip_data_heredocs` 把喂给非 shell 命令的 heredoc 正文当数据剥掉，结束词留成只有一个词的命令，钩子跳过它（「if not words or (len(words) == 1 and words[0] in heredoc_delimiters):」`.claude/hooks/bash-command-detector.sh:3743`）；`cat` 在读不改名单里；`-m` 的值被 `git_commit_form` 当值跳过。但 `.claude/hooks/bash-command-detector.sh:9364-9438` 的自证表里没有这一形的格（`grep` 找不到 `commit -m "$(cat`），读码不能代替那一格 | 是，最常见的一形 |
| `git add <路径> && git commit -m …`、`git add -A && git commit -m …`、`git status && git diff --cached --stat && git commit -m …`、`git commit -m … && git push` | 放行（没超限时）：`git add`、`git status`、`git diff` 不算前置改动；push 一支不查前面 | `GIT_READ_ONLY_SUBCOMMANDS`（`.claude/hooks/bash-command-detector.sh:3305`）与 `GIT_FIXED_SUBCOMMANDS`（`.claude/hooks/bash-command-detector.sh:3304`） | 是 |
| `printf … > <绝对路径，在工作根或 /tmp 下> && git commit -F <那个文件>`，或 `cat > <同上> <<'EOF' … EOF` 换行 `git commit -F <同上>` | 放行、不记检出 | 后一批新加的格，见第五节最后一行 | 是 |
| 误拒一：`printf … \| tee <文件> > /dev/null && git commit -F <文件>` | 拒，原因类 preceding-change | `tee` 不在读不改名单（`.claude/hooks/bash-command-detector.sh:3311`），且 `command_changes_nothing` 只看重定向（`.claude/hooks/bash-command-detector.sh:3582`-`.claude/hooks/bash-command-detector.sh:3596`）；主 agent 与后一批报告都写了「仍被拒」 | 不常见 |
| 误拒二：重定向目标写成 `$HOME/…`、`$WORK/…` 这类变量路径，如 `cat > "$HOME/.local/share/singlefs/work/<草稿>/msg" <<'EOF' … EOF` 再 `git commit -F "$HOME/…/msg"` | 拒（推的） | 「if "$" in path or "`" in path or not os.path.isabs(path):」（`.claude/hooks/bash-command-detector.sh:3575`）：带 `$` 的目标当会改；`output_redirections` 在 `.claude/hooks/lib_shell_words.py:2692` 一处原样交出目标，我没追到更上游有没有代入变量，所以这一条是推的，探针 `daily-redirect-home-variable-then-commit` 判准。`agent-common.md` 要求新写的脚本从参数或环境取目录、不硬写家目录，所以变量写法并不少见 | 可能 |
| 误拒三：`git diff --cached --name-only \| xargs -r <读命令> && git commit` | 拒（推的） | `xargs` 不在读不改名单（`.claude/hooks/bash-command-detector.sh:3311`），它自己被当前置改动（`.claude/hooks/bash-command-detector.sh:3746`-`.claude/hooks/bash-command-detector.sh:3747`）；里面起的命令照判，但外面这条已经把 `mutator` 记上了 | 核暂存区名单时可能写 |
| 前面排着脚本、`task-server/task submit`、`python3 …`，或提交说明用 `-m "$(python3 …)"` 命令替换生成 | 拒 | `python3`、`bash` 都不在读不改名单；文档里的流程把它们与 `git commit` 分成两条命令 | 流程上不该有；说明用脚本生成的写法推的、未见过 |

「推的」都待主 agent 复跑增补探针里的 `daily-forms` 一组（第八节命令）：放行的格应是 `verdict=passed-nothing-lands`，拒的格应是 `verdict=rejected-nothing-lands`；多行提交说明那一格（`daily-commit-heredoc-message`）读到 `rejected-nothing-lands` 就是日常写法被误拒，该先修再往下。三类误拒的改法由主 agent 定，我只列现象：`tee` 与「重定向目标带已知变量」按重定向目标判、`xargs` 之类启动器里面的命令都读不改时不记前置改动，三处各是一行判据。

## 七、探针改了哪几格，复跑后每格应有的结局（都是推的，待主 agent 复跑）

### 攻方腿第一轮的 `probe.py`（原样，只换核哈希的快照目录）

第一轮读数在 `fg-timeout-large-file-governance-r1-main-run/probe.out`。复跑后相对第一轮会变的格（第一轮 `verdict` → 应有）：

| 格 | 第一轮 | 应有 |
|---|---|---|
| commit-command-line-alias、commit-command-line-shell-alias、commit-repository-alias、commit-through-xargs、commit-through-find-exec、commit-abbreviated-message-option、-include-option、-force-add | `missed-silently` | `rejected`（命令照样执行、提交照样落地，只看钩子有没有拒） |
| commit-after-stash-pop、-stash-apply-index、-cherry-pick-no-commit、-checkout-path-from-branch、-merge-squash、-redo-after-reset-soft、-file-created-in-same-command、-all-file-grown-in-same-command、-repository-made-in-same-command | `missed-silently` | `rejected` |
| commit-relative-private-index-kept-at-top、commit-path-from-subdirectory-diff-relative-config | `missed-silently` | `rejected` |
| commit-relative-private-index-kept-in-subdirectory | `rejected-nothing-lands`（误拒） | `passed-nothing-lands` |
| push-new-branch-first-time | `passed-recorded` | `rejected` |
| push-named-other-branch、push-all-branches、push-mirror、push-tags、push-recurse-submodules | `missed-silently` | `rejected` |
| push-delete-other-ref-while-current-dirty、push-clean-side-branch-while-current-dirty | `rejected-nothing-lands`（误拒） | `passed-nothing-lands` |
| judge-fails-keys-missing | `hook_exit=0 refused_21=no refused_22=no recorded_22=no recorded_hook_layer=yes` | `hook_exit=2 refused_21=yes refused_22=yes recorded_22=yes recorded_hook_layer=no`（`refused_22` 是说明里有「超过单文件上限」，没判成的提示里也有这几个字，与别的 judge-fails 格同） |
| judge-fails-slow-with-child | `leftover_after_hook=4` | `leftover_after_hook=0`；`hook_seconds` 仍约 100（四次各 25 秒） |
| index-written-back-by-judge、…-optional-locks-off | `index_bytes_changed=yes` | `index_mtime_changed=no index_bytes_changed=no` |
| remedy-push-message | `names_a_redo_command=no` | `names_a_redo_command=yes` |
| frozen-hash 四行 | 对第一轮快照 `match=yes` | 对这一轮快照：七份里六份 `match=yes`，`.claude/agent-common.md` 一份 `match=no`（它此刻的哈希与快照不同，开工后又核过一遍） |

应与第一轮相同的格：commit-control-staged、commit-amend-with-staged-big、commit-message-from-file、commit-git-config-prefix、commit-command-builtin-prefix、commit-through-eval、-heredoc-to-bash、-sh-c、commit-add-update-chain、commit-exported-private-index、commit-after-pushd、push-control、push-head-to-other-ref、path-with-space-and-non-ascii（`rejected`）；commit-amend-keeps-head-big（`missed-silently`，设计如此）；false-unstaged-grown-plain-commit、false-ignored-big-add-all、symlink-to-outside-*（`passed-nothing-lands`）；remedy-incompressible-gzip-size（`gz_bytes=104874292 limit=104857600`，出路换了文字、压出的大小不变）；timeout 各格（含 `timeout-integer-with-400-digits` 仍是 `hook_exit=0 recorded=yes`，第 2 问那一处窄打中）、limit-variable-*、tree-*、judge-without-git-on-path。

### 第一轮正推腿的三份探针（`sonnet-probes/`）

只改期望文字、不改命令，共 12 处，另加 1 格（`Q14b`）；每格应有的读数：

| 格（第一轮探针里的编号） | 命令 | 应有读数 |
|---|---|---|
| 暂存了超限文件时 `git add new.bin && git commit -m x small.txt`（G03e） | 带路径的 commit 不判前面的 add | `rc=0`、检出 0 条（第一轮 `rc=2`） |
| `git add --pathspec-from-file=list.txt && git commit -m x`（G03f） | add 路径认不出 | `rc=2`（第一轮 `rc=0`、检出 0 条） |
| `git add "$BIGFILE" && git commit -m x`（G03g）、`git add $(ls *.bin) && git commit -m x`（G03h） | 同上 | `rc=2`（第一轮 `rc=0`） |
| `cd /nonexistent-dir-probe && git commit -m x`（G07b） | 目录算得出但不存在 | `rc=0`、检出 1 条（第一轮 0 条） |
| `git -c alias.ci=commit ci -m x`（G07g） | 别名 | `rc=2`（第一轮 `rc=0`） |
| `echo x \| xargs git commit -m`（G07m） | 经 xargs | `rc=2`（第一轮 `rc=0`） |
| 无上游仓里 `git push origin main`（G06g） | 本地没有任何远端跟踪引用 | `rc=0`、检出 1 条，原因文字换成「本地没有任何远端跟踪引用」（读数形态同第一轮） |
| `--guard commit-index` 在 big.bin 已随 ws.bin 提交并推了的仓里（Q01） | 期望改对 | `rc=0`、JSON 里 `"checked": 0` |
| `--guard push --limit-bytes ²`（Q14） | 值不是 ASCII 十进制 | `rc=2`、标准输出 0 行（第一轮 `rc=1`） |
| `PATH=/nonexistent` 起判法（Q16） | git 不在 PATH | `rc=78`（读数同第一轮，文件头现在列了 78） |
| 空仓 `--guard push`（Q21） | 没有提交 | `rc=0`（读数同第一轮，文件头现在说 `--guard` 下 77 只有目录不在仓里） |
| 不带 `--guard` 的 `--limit-bytes ²`（Q14b，新加） | 文件头与代码对不上 | `rc=2`、标准输出 2 行、标准错误空（出现 0 行说明第 3 节第 2 处我读错） |

其余格读数应与第一轮 `out-git.txt`、`out-guard.txt`、`out-timeout.txt` 相同（G01 到 G02d、G03 到 G03d、G04 到 G06f、G07 到 G08；T01 到 T18，其中 T15 仍是 `rc=0`、检出 1 条）。

### 增补三组（`probe_defense.py`）

三组共 41 格；每格一行 `PROBE`，`verdict` 取值与第一轮同形。

| 组 | 格 | 应有 |
|---|---|---|
| push-after-state-change | push-after-merge-of-dirty-branch、push-after-checkout-of-dirty-branch、push-after-reset-hard-to-dirty-branch | `hook_exit=0 recorded=no landed=push verdict=missed-silently`（第 3 节第 1 处） |
| push-after-state-change | commit-after-merge-of-dirty-branch-control | `rejected`（commit 一支会拒前面的 merge） |
| daily-forms | daily-commit-heredoc-message、daily-add-then-commit、daily-add-all-then-commit、daily-status-diff-then-commit、daily-commit-message-file、daily-redirect-absolute-then-commit-file、daily-commit-then-push、daily-push-explicit-branch、daily-push-head-to-new-ref、daily-push-force-with-lease、daily-commit-amend-small、daily-export-then-commit、daily-date-then-commit、daily-cd-then-commit | `passed-nothing-lands`；其中 daily-commit-heredoc-message 是推不出的那一格，读到 `rejected-nothing-lands` 就是误拒日常写法 |
| daily-forms | daily-tee-message-then-commit、daily-xargs-listing-then-commit、daily-script-then-commit、daily-message-from-script-substitution、daily-redirect-home-variable-then-commit | `rejected-nothing-lands`（第六节「误拒」三行与前置脚本一行）；读到 `passed-nothing-lands` 说明我读错了那一处 |
| write-scope-rows | scope-defense-writes-own-report、-other-leg-report、-other-round-model、-work-root，scope-forward-writes-own-model-file，scope-materials-writes-checklist，scope-designer-writes-preregistration | `hook_exit=0 refused=no` |
| write-scope-rows | scope-defense-writes-records、-crates、-kb、-agent-definition，scope-attack-writes-other-round-background，scope-materials-writes-body、-judged-list，scope-designer-writes-kb-page，scope-sweep-writes-records，scope-prior-art-writes-kb，scope-mutation-triage-writes-table | `hook_exit=2 refused=yes` |

## 八、要主 agent 跑的命令

我没往任务服务交任何东西。下面三条，每条一行；判决要引的读数以它们跑出来的为准，在跑出之前上面所有「判成什么」都是「推的，待主 agent 跑」。

1. 全部探针一趟（攻方腿第一轮探针原样、增补三组、正推腿三份）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet -- bash model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet-run /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet/model`
   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet`（submit 的 `--cwd`）。
   - 跑完看：`TASK_SERVER_RESULT` 的 `exit=0`（探针只在有一组装置出错时退非 0）；日志里 `name=rerun_roots`、`probe.out` 与 `defense.out` 末行的 `PROBE name=done emitted=<N>`（`tail -n 1` 打出来的两行）、三份正推腿读数各自的行数；读数在运行目录 `…-sonnet-run` 下的 `probe.out`、`defense.out`、`probe-hook-git.out`、`probe-guard.out`、`probe-timeout.out`，逐格对第七节的表；有 `PROBE name=group-error-` 起头的行说明那一组装置出了错，不算读数。运行目录 471M 量级，用完 `du -sh` 记大小再删。
2. 复跑模板自证（确认这份 `rerun.sh` 拷在别处也跑得通，换的只是 `rerun_steps`）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet -- bash model/tools/rerun.sh --selftest`
   - 在哪个目录起：同上。
   - 跑完看：`exit=0`，成功句「✓ 自检通过（查了 6 格）」；有「✗ 自检：」行就是模板被我改坏了。
3. 名单三处一致的独立数法（不经钩子的正则）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet -- python3 -I tools/names-check-loose.py /home/fy5090/code/singlefs/.claude/hooks/bash-command-detector.sh`
   - 在哪个目录起：同上。
   - 跑完看：`known 155 文件头里完全没出现的名字 []`；方括号里有名字就是文件头开关表缺了那几个。

## 九、什么现象会推翻上面的结论，没做什么

推翻条件已逐条写在各节里，汇总：

- 第 1 问「成立」：复跑里第七节「会变的格」有一格读数不是「应有」那一列。
- 没关上的三处：第 1 处看三格 `push-after-*` 的 `verdict` 是不是 `missed-silently`；第 2 处看 `Q14b` 的标准输出行数；第 3 处看 `judge-fails-slow-with-child` 的 `hook_seconds` 是不是小于 30。
- 第 2 问：T15 看 `timeout-integer-with-400-digits` 是否 `hook_exit=2`（是则文件头与代码对得上，我读错）。
- 第 3 问：自证（主 agent 报「查了 1493 种」通过）里点名的格有一格红；`names-check-loose.py` 读出文件头缺名字。
- 第 4 问：`daily-forms` 组里放行的格有 `rejected-nothing-lands`（尤其 `daily-commit-heredoc-message`），或拒的格有 `passed-nothing-lands`。

没做什么：

- 没有起任何探针、自证、门禁或钩子（按定义这些由主 agent 经任务服务跑）；「判成什么」全是读码推的，读数都待主 agent 复跑，没有一个是这一轮量出来的，唯一直接跑的是我自己写在草稿目录的两个只读文件的小脚本 `tools/names-check.py`、`tools/names-check-loose.py`（读钩子源文件、不 import 它），它们的输出贴在第五节；
- 没有读会话记录目录里任何文件的内容（共用约束「不碰 `~/.claude/`」；开头 `ls` 过一次那个目录，只看了文件名与大小，随即停手），第 4 问的「日常写法」只取 `staged-gate.md`、`commit-stages.md`、`main-agent.md`、`kb/tooling.md`、`records/`、HEAD 的 reflog 与 `git log`，没有真实的会话里的 `git commit` 命令样本，这是第 4 问最薄的一处；
- 没有读攻方腿这一轮的提示与产出（禁读清单）；
- 没有替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）；
- 没有判 ⑫、⑭ 的判法，没有判第 5 到 11 问（攻方腿的）；`.claude/gate.d/code-tooling.sh` 的 agent-write-scope 格我只读了判表那一段（`.claude/gate.d/code-tooling.sh:4020`-`4090`），没有读它的样本档；
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet` 共约 136K，没有仓副本与编译目录，里面的 `model/` 留给主 agent 复跑，复跑之后随草稿目录一起删。

引文核对（`python3 scripts/agents/cite-check.py research/prompts/fg-timeout-large-file-governance-r2-sonnet-output.md --root . --background research/prompts/_fg-timeout-large-file-governance-r2-background.md` 末行原样）：

```
  ✓ 核了 41 处引文、80 处裸行号，对上 41 处，没判 80 处（其中裸行号没有引文可比 80 处；逐处列在上面）
```
