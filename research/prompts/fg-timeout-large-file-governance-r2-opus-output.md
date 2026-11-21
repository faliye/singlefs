# 攻方腿报告：fg-timeout-large-file-governance-r2（正文「要腿判的」第 5 到 11 问）

日期：2026-11-19。被判文件读的是主工作区此刻的那一版，开工时现算 sha256，与 `research/prompts/fg-timeout-large-file-governance-r2-snapshot/judged-sha256.txt` 七行逐字相同（检出钩子 02d0c3c8…、large-file-check.py 196d9ff7…、code-tooling.sh 8c097a5d…、写范围表 c72529ce…、dispatch-prompt.py d1066319…、agent-common.md 255b3b5b…、正文 8e042f45…）。

判词：「成立」= 这一问问的那种错在今天的实现上找得到（打中）；「不成立」= 找不到；「推不出」= 靠仓里读得到的东西判不出（派发提示定为打中）。
子 agent 不交任务，探针与三份模型一条都没跑：每格「钩子会判成什么」是读码推的，标「推的，待主 agent 跑」；只读命令现跑出来的标「量的」。

## 复跑命令与模型文件

模型目录 `research/prompts/fg-timeout-large-file-governance-r2-opus-model/`（从第一轮攻方模型拷来改，见下）：

| 文件 | sha256 |
|---|---|
| `probe.py` | `2a8e9f2fccf8dd41d08f057942e4e34114b7fd092a5dd4b48a30e0df19ea0883` |
| `name_sets_model.py` | `9a1bcb27fb775e25f02570088f6b776e4578b63fb02a8737317fa527458b5402` |
| `clippy_tail_model.py` | `8ebba4b2ef4535ad6d5e586122d694d438d62edef289508d4ef8d0bdbb865466` |
| `dispatch_scope_model.py` | `081e201ae36d9280e84876fd4d2513a0b234639c39e227e495c63ff6caae79a4` |
| `tools/rerun.sh` | `201612dbf82c2b5e27d8b781facd3f7ad2e9a61f9b44c92d89c47dee9bfe1616` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

复跑（整条在「要主 agent 跑的命令」第 2 条）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-opus-run /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r2-opus-model
```

- 第二个参数（冻结副本）填的是仓根：主 agent 有开工快照树就换成它。`probe.py` 开头逐份核 `judged-sha256.txt` 七行，打 `PROBE name=frozen-hash … match=yes|no`。
- 从第一轮攻方模型（`research/prompts/fg-timeout-large-file-governance-r1-opus-model/`）改了什么：`probe.py` 留下装置那一半（`emit`、`group_members`、`stop_own_group`、`run_own_group`、`git_environment`、`git`、`write_small`、`make_big`、`new_repository`、`commits_of`、`blob_sizes`、`big_blobs_brought_by`、`big_blobs_in_remote`、`feed_hook`、`payload_for`、`large_file_case`、`judged_hashes`、`main`），格全部换新；`large_file_case` 加了 `hook_dropped`（从喂钩子的环境里删掉的变量）与 `cause` 一列（拒绝说明里「判不准」那一类的原因）；`git_environment` 加 `GIT_EDITOR=true`；新加 `processes_mentioning`（只读 /proc，不发信号）。第一轮的 `fake_large_file_check.py` 没拷：这一轮不用假判法。第一轮各格复跑后应有的结局不在这一轮的模型里，由辩方腿与判决管。
- `tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`：依次起三份模型与探针，各自输出另存 `<运行目录>/name-sets.out`、`clippy-tail.out`、`dispatch-scope.out`、`probe.out`，四份都跑完才退，有一份退非 0 就退 1。外层包装是第一轮那一份原样拷来（只删工作根下以 `-run` 收尾的那一个运行目录）。
- 三份模型从被判那棵树的钩子源文件里按锚串原样取出一段代码、在模型进程里执行（取不出就打 `setup-error` 退 1）：`name_sets_model.py` 取检出钩子 `BREAK_SWITCH_KNOWN_TUPLE` 到「弄坏开关只在自证里认」那段；`clippy_tail_model.py` 取检出钩子 `CLIPPY_LINT_TABLE_RELATIVE` 起的一段与 `project-code-guard.sh` 的 `CLIPPY_LINT_TABLE_START` 起的一段；`dispatch_scope_model.py` 取 `runner-dispatch-guard.sh` 的 `SCOPE_PATH_START` 起的几条正则与 `expanded_mention`、`scope_patterns`、`out_of_scope_mentions`。
- 样本进程：喂钩子与执行命令各用 `start_new_session=True` 起，超时只 `os.killpg` 自己起的那一组；「钩子被停、判法还活着」那一格不停判法（判法在钩子起的另一个会话里，不是探针起的），只按 /proc 里的命令行等它自己退、最多 120 秒，记活了多久。git 配置隔离成运行目录里的一份。
- 内存与时间（推的，没跑过）：格数约 70；大头是三格从 100 MiB 一行的零字节读写（`sort -o`、`sed w`、`awk print >`，各一两百 MB 内存、几秒）、两格 `head -c` 写 100 MiB、「判法到点」那格 25 秒加「钩子被停」那格最多 30 来秒。估全量 6 到 10 分钟，远低于 40 分钟。盘：运行目录峰值约 700 MiB（五份真写出来的 100 MiB 文件，其余是稀疏文件与压过的零）。没有崩溃状态，第 3b 步的崩溃枚举与状态数不适用。

## 各格判定一览

结局列是 `probe.py` / 模型那一格的格名与推的结局（待主 agent 跑）。

| 问 | 格 | 判 | 推的结局 |
|---|---|---|---|
| 5 | shell 别名里不写 commit 这个词、经另一个别名起 commit（`save = !git add -A && git ci …`） | 成立：静默放过 | `alias-shell-chain-without-word` missed-silently |
| 5 | 同一条命令里先 `git config alias.ci commit` 再 `git ci` | 成立：静默放过 | `alias-defined-in-same-command` missed-silently；对照 `alias-configured-beforehand-control` rejected |
| 5 | 别名链超过 `GIT_ALIAS_DEPTH`（六层） | 成立：静默放过 | `alias-chain-deeper-than-limit` missed-silently |
| 5 | 别名值以 git 全局选项起头（`--no-pager commit`） | 推不出：钩子放过已读码坐实，git 收不收这种别名没核 | `alias-value-leading-global-option` missed-silently 或 passed-nothing-lands |
| 5 | PATH 上的外部子命令 `git-ci` | 成立：静默放过 | `external-subcommand-on-path` missed-silently |
| 5 | 别的程序里起的 git commit / push：`python3 -c`、`awk system()`、`git rebase --exec`、`git submodule foreach`、`push.recurseSubmodules` 配置 | 成立：静默放过（同一份钩子的 ⑬ 已认这几种起法，㉒ 没接） | `commit-in-python-c`、`commit-in-awk-system`、`commit-in-rebase-exec`、`push-in-submodule-foreach`、`push-recurse-from-config` 都 missed-silently；对照 `push-recurse-flag-control` rejected |
| 5 | 「读不改」名单里的命令按操作数或程序文本写文件：`sort -o`、`sed 'w 文件'`、`awk '{print > "文件"}'` | 成立：静默放过 | `read-only-name-sort-output`、`read-only-name-sed-write-command`、`read-only-name-awk-print-redirect` missed-silently |
| 5 | 重定向写进放在工作根下的仓（git worktree、仓副本都在工作根下） | 成立：静默放过（后一批为 `commit -F` 加的放行带出来的） | `redirect-into-repository-under-work-root` missed-silently；对照 `redirect-into-repository-relative-control` rejected |
| 5 | 工作根没设 `SINGLEFS_WORK_ROOT` 时照 `~` 展开：同一条共享路径上的写，换一个 HOME 判得不同 | 成立（弱）：判定随主机变，违反「多机可用」 | `harmless-work-root-home-owning-this-path` passed-nothing-lands、`harmless-work-root-another-home` rejected-nothing-lands |
| 5 | `export GIT_INDEX_FILE=… && env -u GIT_INDEX_FILE git commit` | 成立：静默放过 | `env-unset-private-index` missed-silently；对照 `env-private-index-control` passed-nothing-lands |
| 5 | `git -C "$(pwd)" commit` | 不成立：照设计判不了、记检出 | `git-C-command-substitution` passed-recorded |
| 5 | 误拒：`chmod`、`rm`、`mkdir`、`touch`、`git stash list`、`git worktree list`、`"$TMPDIR"/…` 与 `mktemp` 的提交说明、`tee` 写到工作根 | 成立（误拒）：名单外一律当会改 | 九格都 rejected-nothing-lands；对照 `daily-message-redirect-in-work-root-control`、`daily-heredoc-message-control` passed-nothing-lands |
| 6 | `origin +HEAD:refs/heads/x`、`--force-with-lease`、`--set-upstream` 已有分支、不带 refspec（`push.default=simple`） | 不成立：照拒 | 四格 rejected |
| 6 | 同一条命令里先建分支再推（`git switch -c topic && git push -u origin topic`） | 成立：静默放过（引用此刻解析不了，略过且不记） | `push-branch-created-in-same-command` missed-silently |
| 6 | 同一条命令里先快进合并再推 | 成立：静默放过（push 不看前面的改动；主 agent 已知） | `push-after-fast-forward-merge` missed-silently |
| 6 | `push.default=matching`（仓里配的、`-c` 给的）、`remote.origin.push` 配置 | 成立：静默放过 | `push-default-matching`、`push-default-matching-command-line`、`push-remote-push-refspec-config` missed-silently |
| 6 | 通配 refspec `refs/heads/*:refs/heads/*` 推带斜杠的分支 | 成立：静默放过（`for-each-ref` 的 `*` 不跨 `/`，量的） | `push-wildcard-refspec-nested-branch` missed-silently；对照 `…-top-level-control` rejected |
| 6 | `--repo=<甲> <乙>`（git 取位置参数）、推到写成路径的远端而另一个远端的跟踪引用带着那笔提交 | 成立：静默放过 | `push-repo-option-with-positional-remote`、`push-to-path-another-remote-has-it` missed-silently |
| 6 | 远端没有跟踪引用 | 不成立：照设计放行、记检出 | `push-remote-never-fetched` passed-recorded |
| 7 | 判法到点被停整组：索引拷贝留在 `$TMPDIR` | 成立（弱） | `judge-timeout-leaves-index-copy` leftover_index_copies=1 |
| 7 | 钩子本身被停（Claude Code 的钩子时限）：判法在另一个会话里接着跑 | 成立（弱）；钩子时限多长是印象 | `hook-stopped-judge-keeps-running` judges_alive_after_hook_stopped=1 |
| 7 | 别的会话拿着 `index.lock` | 不成立 | `index-locked-by-another-session` hook_exit=0、锁原样 |
| 7 | `core.splitIndex=true` 的仓：判法往用户的 `.git` 写新的 sharedindex | 推不出（git 写不写新 sharedindex 没核） | `split-index-judge-writes-git-directory` 看 shared_index_files 前后 |
| 7 | 大仓：没推的提交多时找带进大文件的提交逐笔起 `git diff-tree` | 不成立（这个仓）：每笔约 7 毫秒、整仓 427 笔（量的） | 无格 |
| 7 | `-c diff.relative=false` | 不成立：路径一律从顶层起算 | 无格（读码） |
| 8 | 三方腿的模式让一条腿写别的腿、别的轮的报告与模型 | 成立 | `scope-attack-writes-other-leg-report`、`…-earlier-round-report`、`…-other-leg-model` 都放行 |
| 8 | 八类有没有定义实际要写而表里没给的路径 | 不成立（照各定义「写范围」「产出」现读） | 无格 |
| 8 | `$SINGLEFS_WORK_ROOT/**` 盖住别的 agent 的草稿目录、冻结副本、工作根下的 worktree | 成立：八类只带 Read、Bash，写者锁对它们一处都不判 | `…-locked-draft-of-another-agent`、`…-frozen-copy`、`…-worktree-under-work-root` 都放行；对照 `…-crates-control` 拒 |
| 9 | 文件头写法变（同一行两条、反引号、续行、缩进）、说明行长得像开关行 | 成立（弱，误判方向：响红） | `name_sets_model.py` 那六种 false-mismatch |
| 9 | 单引号、经名字、bash `==` 形式读的开关；开关删了而节里「弄坏开关：」那一串还写着 | 成立（窄，漏判） | 四种 missed-silently |
| 10 | `check.sh` 读不到、表为空、写法变了 | 不成立：出路不会被 `project-code-guard.sh` 拒（两边同一种读法，读不到时闸也不判）；出路与门禁真跑的会差几条（合入后验证才红） | `clippy_tail_model.py` 全部 outlet-passes-guard；`gate_minus_outlet` 非空的八种 |
| 11 | ④ 对八类提示里「写进 / 写到 / 更新 <路径>」：转述别人要写的也当这个子 agent 要写 | 成立（误拒）：表里加了行之后才判这八类 | `dispatch_scope_model.py` 五格 false-refusal |
| 11 | ④ 对攻方腿提示里点名辩方腿的报告 | 成立（放过）：模式盖住别的腿 | `attack-leg-given-defense-leg-report` outside=[] |

## 第 5 问：执行前拒的五类，误拒与漏网 —— 成立

判据字面：误拒 = 日常写法什么超限的都带不进去而照拒；漏网 = 超限的进了提交或远端，㉒ 不拒也不记检出。每格的命令就是探针那一格的 `command=`，装置（仓里暂存了上限加一字节的 big.bin，或 big.bin 没跟踪）写在格的函数文档串里。

### 别名：只看值里有没有 commit / push 这个词，只查到第五层，只查此刻的配置

- `.claude/hooks/bash-command-detector.sh:3662`：「if detector_break_is_set("shell-alias-allowed") or not re.search(r"\bcommit\b|\bpush\b", alias):」——`!` 别名的值里没写这两个词就放过，不看它起的是不是另一个会 commit 的别名。文件头也是这么定的：`.claude/hooks/bash-command-detector.sh:444`：「#   `!` 开头而里面写着 commit 或 push 的 shell 别名拒。」
  - 格 `alias-shell-chain-without-word`：仓里配 `alias.ci = commit`、`alias.save = !git add -A && git ci -qm wip`，big.bin 没跟踪，喂 `git save`。git 起 shell，shell 起 `git ci`，`ci` 是 commit。钩子：`save` 的值不带 commit → 放过，`save` 不在 `GIT_FIXED_SUBCOMMANDS` 里，不判。推的 missed-silently。
- 别名值查的是钩子起来那一刻的配置（`git_alias_value` 经 `git_config_commands` 现读），同一条命令里排在前面的 `git config alias.…` 还没跑。格 `alias-defined-in-same-command`：`git config alias.ci commit && git ci -qm x`。`config` 在只读子命令名单里，不当会改；`ci` 查不到别名，不判。推的 missed-silently；对照 `alias-configured-beforehand-control`（先配好再 `git ci -qm x`）推的 rejected。
- `.claude/hooks/bash-command-detector.sh:3286`：「GIT_ALIAS_DEPTH = 5」：查满五层还没到内建子命令的，`resolved_git_subcommand` 交回的还是别名，`.claude/hooks/bash-command-detector.sh:3756`：「elif subcommand in GIT_FIXED_SUBCOMMANDS or (every_git_command …」不中，不判也不记。格 `alias-chain-deeper-than-limit`：`a1 → a2 → … → a6 = commit`，喂 `git a1 -qm x`。推的 missed-silently。git 自己只防别名成环，不限层数（印象，格里的 `command_exit` 核它）。
- `.claude/hooks/bash-command-detector.sh:3669`：「if not words or words[0].startswith("-"):」：别名值以选项起头的就停。git 展开别名时先处理值里的全局选项（印象，没核）。格 `alias-value-leading-global-option`：`alias.ci = --no-pager commit`。git 收这种别名就是 missed-silently，不收就是 passed-nothing-lands；推不出，算打中，等格的 `command_exit`。
- 外部子命令：`git ci` 查不到别名时 git 去 PATH 上找 `git-ci` 可执行文件。格 `external-subcommand-on-path`：`PATH=<运行目录>/extra-bin:$PATH git ci -qm x`（`git-ci` 是 `exec git commit "$@"`）。推的 missed-silently。

### 别的程序里起的 git commit / git push：㉒ 只剥 xargs、find、parallel 与启动器

㉒ 的步骤只从切词模块的「命令位置上的命令」与 `LAUNCHER_NAMES` 剥出来的那一条取（`.claude/hooks/bash-command-detector.sh:3608`：「if name not in LAUNCHER_NAMES or name == "ssh":   # ssh 起的命令跑在别的机器上…」），不读切词模块的 `embedded_launches`（python / perl / awk / sed / git 配置与别名里起的命令）。同一份钩子的 ⑬ 读它：`.claude/hooks/bash-command-detector.sh:1167`：「#   embedded-launch-claude-unread  ⑬ 不认别的程序里起的 claude、不读它们读的程序文件：…」。所以同一种起法，起 claude 拒、起 git commit 放。

| 格 | 命令 | 推的结局 |
|---|---|---|
| `commit-in-python-c` | `python3 -c 'import subprocess; subprocess.run(["git", "commit", "-qm", "x"], check=True)'` | missed-silently |
| `commit-in-awk-system` | `awk 'BEGIN { exit system("git commit -qm x") }'` | missed-silently（awk 还在「读不改」名单里） |
| `commit-in-rebase-exec` | `git rebase -q --exec 'git add big.bin && git commit -q --amend --no-edit' HEAD~1` | missed-silently：rebase 只记成「会改」，后面没有 git commit 步骤可拒 |
| `push-in-submodule-foreach` | `git submodule --quiet foreach 'git push -q origin HEAD:main'`（子模块里一笔带 big.bin 没推） | missed-silently：第一轮打中的 `--recurse-submodules` 换个写法照推 |
| `push-recurse-from-config` | `git -c push.recurseSubmodules=on-demand push -q` | missed-silently：拒的只是命令行上的 `--recurse-submodules=on-demand|only`，配置项（`-c`、仓里的 `push.recurseSubmodules`、`submodule.recurse`）不看 |
| `push-recurse-flag-control` | `git push -q --recurse-submodules=on-demand` | rejected（对照） |

### 「会改」认的是命令名，不认按操作数、程序文本写文件的；工作根下的重定向一概当不碰仓

- `.claude/hooks/bash-command-detector.sh:3312`：「"grep egrep fgrep rg sort uniq tr cut basename …」是「读不改」名单的一行，`.claude/hooks/bash-command-detector.sh:3596`：「return name in READ_ONLY_COMMAND_NAMES」只看名字；sed 只看 `-i`（`.claude/hooks/bash-command-detector.sh:3595`），重定向只看 shell 那一层的。名单里的 sort（`-o`）、sed（`w` 命令）、awk（程序里的 `print >`、`system()`）都写文件。格（仓外放一份上限加一字节的 huge.src）：`sort -o base.txt <huge.src> && git commit -qam x`、`sed -n 'w base.txt' <huge.src> && …`、`awk '{ print > "base.txt" }' <huge.src> && …`。推的三格都 missed-silently：commit -a 判的是改之前的 base.txt。
- 后一批为「提交说明写到仓外再 `git commit -F`」加的放行：`.claude/hooks/bash-command-detector.sh:3578`：「work_root = os.path.expanduser(os.environ.get("SINGLEFS_WORK_ROOT") or "~/.local/share/singlefs/work").rstrip("/") + "/"」，`.claude/hooks/bash-command-detector.sh:3579`：「return path.startswith(HARMLESS_WRITE_PREFIXES) or path.startswith(work_root)」。它不看落点是不是正要提交的那个仓。本机的 git worktree 有两棵在工作根下（量的：`git worktree list` 列出 `…/singlefs/work/doc-gate-commit-20261105/tree`、`…/singlefs/work/m5-dev`），仓副本也都在工作根下，`.claude/kb/tooling.md`「几个会话共用一个仓时怎么暂存与提交」那一节的「在 worktree 上提交」就是在这种树里提交。格 `redirect-into-repository-under-work-root`：`head -c 104857601 /dev/zero > <仓的绝对路径>/base.txt && git commit -qam x`（喂钩子时 `SINGLEFS_WORK_ROOT` 指运行目录，仓在它下面）。推的 missed-silently；对照 `redirect-into-repository-relative-control`（写成 `> base.txt`）推的 rejected。
- 同一行另一处：工作根没设 `SINGLEFS_WORK_ROOT` 时照 `~` 展开，而 `.claude/hooks/lib_work_root.py:9`：「  经 ~、$HOME 或 expanduser 拼出来的路径在第二台落进它本地那一份，两台读不到同一份。」工作根只该经 `lib_work_root.py` 取。格 `harmless-work-root-home-owning-this-path` 与 `harmless-work-root-another-home`：同一条 `printf 'x\n' > <运行目录>/message-in-shared-work-root.txt && git commit -qaF <同一路径>`，只换喂钩子的 HOME（一个是运行目录路径里 `/.local/share/singlefs/work/` 之前那一段，一个是别的目录）、删掉 `SINGLEFS_WORK_ROOT`。推的前一格 passed-nothing-lands、后一格 rejected-nothing-lands：同一条命令放到第二台判就是误拒。门禁 code-tooling 的 shared-data-root-literals 格判不判这一行，我没跑（命令在「要主 agent 跑的命令」第 3 条）；这一行不在 `.claude/gate.d/shared-data-root-literals-exempt.tsv` 里（量的：`grep -n bash-command-detector` 零命中）。

### GIT_INDEX_FILE：env -u 不认

`.claude/hooks/bash-command-detector.sh:3832`：「if index_file and not detector_break_is_set("commit-private-index-unseen") and kind == "commit":」读的是切词模块交的这一条命令的环境；切词模块认 env 的 `-u` 只为跳过它的值（`.claude/hooks/lib_shell_words.py:1164`：「ENV_SHORT_OPTIONS_WITH_VALUE = "uCS"…」），不从环境里删掉那个名字（读码，`skip_prefixes` 里只有 `-i` 清环境）。格 `env-unset-private-index`：`export GIT_INDEX_FILE=<运行目录>/private-index-never-made && env -u GIT_INDEX_FILE git commit -qm x`，默认索引里暂存了 big.bin。钩子照那份不存在的私有索引判（空），git 照默认索引提交。推的 missed-silently。对照 `env-private-index-control`（`env GIT_INDEX_FILE=… git commit`）推的 passed-nothing-lands。正文举的 `git -C "$(pwd)"`：目录带 `$` 交 None，走「目录算不出」记检出，格 `git-C-command-substitution` 推的 passed-recorded，不算打中。

### 误拒：名单外的一律当会改

`.claude/hooks/bash-command-detector.sh:3303`：「# 同一条命令里排在 git commit 前面、会改工作区或暂存区内容的命令：执行前判不到改完之后的内容。认读不改的（名单里的），名单外的一律当会改」。名单外而带不进超限内容的日常写法都拒，探针九格（仓里没有超限的文件，推的都是 rejected-nothing-lands）：

| 格 | 命令 | 为什么带不进超限内容 |
|---|---|---|
| `misreject-chmod-then-add` | `chmod +x tool.sh && git add tool.sh && git commit -qm x` | chmod 不改大小 |
| `misreject-rm-then-add-all` | `rm -f extra.txt && git add -A && git commit -qm x` | 只删 |
| `misreject-mkdir-then-commit-all` | `mkdir -p notes && git commit -qam x` | 空目录不进提交 |
| `misreject-touch-then-commit-all` | `touch base.txt && git commit -qam x` | 只动时间戳 |
| `misreject-stash-list` | `git stash list && git commit -qam x` | stash 整个子命令算会改，`list` 只读 |
| `misreject-worktree-list` | `git worktree list && git commit -qam x` | 同上 |
| `misreject-tmpdir-variable-message` | `printf "x\n" > "$TMPDIR/probe-message.txt" && git commit -qaF "$TMPDIR/probe-message.txt"` | 目标带 `$` 一律当会改（这个会话里 `TMPDIR` 没设，量的；设了它的环境里这是常见写法，推的） |
| `misreject-mktemp-message` | `f=$(mktemp) && printf "x\n" > "$f" && git commit -qaF "$f"` | mktemp 名单外 |
| `misreject-tee-message-in-work-root` | `printf 'x\n' \| tee <运行目录>/tmpdir/tee-message.txt && git commit -qaF <同一路径>` | 主 agent 已知；同一个落点用 `>` 写放行（对照 `daily-message-redirect-in-work-root-control`），用 tee 写照拒 |

对照 `daily-heredoc-message-control`（`git add base.txt && git commit -q -m "$(cat <<'EOF' … EOF)"`，Claude Code 默认的提交写法）推的 passed-nothing-lands。这九种在 `records/` 里真用过几次我没数（辩方腿第 4 问管）；拒绝说明给的出路（分两条起）照做得通，所以是误拒、不是卡死。

## 第 6 问：push 按引用判 —— 成立

装置：本地裸仓当远端（没有单文件上限，推得上去才看得出漏没漏）；「带 big.bin 的提交」是一笔加了上限加一字节文件的提交。

### 判得对的

| 格 | 命令 | 推的结局 | 依据 |
|---|---|---|---|
| `push-plus-head-to-full-ref` | `git push -q origin +HEAD:refs/heads/x` | rejected | `git_push_form` 去掉 `+`、取冒号前的 `HEAD` |
| `push-force-with-lease` | `git push -q --force-with-lease origin main` | rejected | 选项表里是 optional 值，后面的词当位置参数 |
| `push-current-default` | `git push -q`（`push.default=simple`） | rejected | 不带 refspec 取 HEAD |
| `push-set-upstream-existing-branch` | 停在 feature 上 `git push -q --set-upstream origin feature` | rejected | |
| `push-after-commit-amend-in-same-command` | `git commit -q --amend -m again && git push -q --force` | rejected | 改之前的 HEAD 已带着 big.bin，判法照它拒（对照，不算误拒也不算漏） |
| `push-remote-never-fetched` | 新加的远端 fresh 没取回过，`git push -q fresh main` | passed-recorded | 照设计：没有跟踪引用不知道远端有什么，记检出 |

`push.default` 取 `simple`、`current`、`upstream` 时推的内容都是 HEAD，判对；`nothing` 时 git 不推，钩子照 HEAD 判（有大文件就误拒一次，没量）。`matching` 与 `remote.<名>.push` 见下。

### 静默放过

- 引用此刻解析不了就略过、也不记：`scripts/gate/large-file-check.py:435`：「found = git(top, "rev-parse", "--verify", "-q", ref, allow_failure=True)」，文档串写「解析不了的引用略过：git push 自己报错、什么都不推」（`scripts/gate/large-file-check.py:420`：「"""要推出去的引用指着的对象名（去重）：…解析不了的引用略过：git push 自己报错、什么都不推。"""」）；一个都没解析到时 `scripts/gate/large-file-check.py:460`：「return {**guard_result("push", limit, 0, [], []), "remote": remote}」——`not_run` 是空串，钩子不记。「git push 自己报错」只在这条命令里没人先建那个引用时成立。格 `push-branch-created-in-same-command`：main 上一笔带 big.bin 没推，`git switch -q -c topic && git push -q -u origin topic`。推的 missed-silently。这是日常的开新分支写法。
- push 不看排在前面会改历史的命令（主 agent 已知、没改）：commit 那一支有 `.claude/hooks/bash-command-detector.sh:3793`：「if step.mutator and not detector_break_is_set("preceding-mutator-unseen"):」，push 那一支（`.claude/hooks/bash-command-detector.sh:3805` 起）没有。格 `push-after-fast-forward-merge`：main 干净、feature 带 big.bin，`git merge -q --ff-only feature && git push -q`。推的 missed-silently。上一条（新分支）用 mutator 那一招也能拦，两条各要各的改法，见「改法」。
- 不带 refspec 时只取 HEAD，不读 `push.default=matching` 与 `remote.<名>.push`：`scripts/gate/large-file-check.py:438`：「if not (refs or all_branches or tags):」。格 `push-default-matching`（仓里 `push.default=matching`，feature 先不带大文件推过一次、远端有同名分支，之后 feature 上加一笔带 big.bin，停在干净的 main 上，`git push -q`）、`push-default-matching-command-line`（同一装置，`git -c push.default=matching push -q`）、`push-remote-push-refspec-config`（`remote.origin.push = refs/heads/*:refs/heads/*`，`git push -q`）。推的三格都 missed-silently。
- 通配 refspec 交给 `for-each-ref` 按 refspec 的写法当模式：`scripts/gate/large-file-check.py:424`：「output = git(top, "for-each-ref", "--format=%(objectname)", pattern, allow_failure=True)」。`for-each-ref` 的 `*` 不跨 `/`，git push 的 refspec 通配跨。量的（本仓，只读）：`git for-each-ref --format='%(refname)' 'refs/remotes/*' | wc -l` 打 `0`，`git for-each-ref --format='%(refname)' 'refs/remotes/' | wc -l` 打 `2`。格 `push-wildcard-refspec-nested-branch`：分支 `topic/heavy` 带 big.bin，`git push -q origin 'refs/heads/*:refs/heads/*'`。推的 missed-silently；对照 `push-wildcard-refspec-top-level-control`（分支名不带斜杠）推的 rejected。
- `--repo` 与位置参数一起给时钩子取 `--repo`：`.claude/hooks/bash-command-detector.sh:3490`：「remote, specs = (remote_option, positional) if remote_option else …」。git 的 `--repo` 文档写的是两个都给时位置参数优先（印象，格的 `landed` 核它）。格 `push-repo-option-with-positional-remote`：origin（本地裸仓）已收下那笔带 big.bin 的提交，`backup` 只有基准提交，`git push -q --repo=origin backup main`。钩子把 `backup` 当一个引用（解析不了、略过）、对照 origin 的跟踪引用判 main，那笔提交 origin 已有，查不到；git 推到 backup。推的 missed-silently。
- 远端写成网址或路径时对照全部远端的跟踪引用：`scripts/gate/large-file-check.py:407`：「return remote_given if configured_remote(top, remote_given) else ""」，`scripts/gate/large-file-check.py:461`：「specs = [*positive, "--not", f"--remotes={remote}" if remote else "--remotes"]」。别的远端有、这个远端没有的提交被扣掉。格 `push-to-path-another-remote-has-it`：同上装置，`git push -q <backup 裸仓的路径> main`。推的 missed-silently。本仓只配了 origin 一个远端（量的：`git remote -v`），今天碰不上；多一个镜像远端就碰上。

### 不判、留给判决的

- `git push origin :`（推全部同名分支）钩子当成全部本地分支判，多判了远端没有的分支，可能误拒；没摆格。
- 跟踪引用过期（远端已删掉的分支本地还留着跟踪引用）：扣掉的是远端已经不认的提交；要让那笔提交带着超限 blob 进过跟踪引用，得先推上过一个没有上限的远端，本仓碰不上，没摆格。

## 第 7 问：读索引拷贝、进程组、diff.relative 带出的新毛病 —— 成立（两处弱的），一处推不出

### 到点停整组时，索引拷贝留在临时目录

拷贝建在 `scripts/gate/large-file-check.py:483`：「directory = tempfile.mkdtemp(prefix="large-file-check-index-")」（判法进程的 `$TMPDIR`，没设是 /tmp），删在 `finally` 里：`scripts/gate/large-file-check.py:496`：「shutil.rmtree(directory, ignore_errors=True)」。钩子到点发的是 SIGKILL：`.claude/hooks/bash-command-detector.sh:3707`：「os.killpg(process.pid, signal.SIGKILL)」，`finally` 不跑。每次到点留一份整份索引（本仓的索引 2555656 字节，量的：`stat -c '%s' .git/index`）。格 `judge-timeout-leaves-index-copy`：PATH 前面放一份带 `diff` 参数就先睡 40 秒的 git，`TMPDIR` 指运行目录里的一个空目录，喂 `git commit -qm x`。推的：钩子约 25 秒返回、记检出、`leftover_index_copies=1`。几个会话天天提交，碰到慢盘（第二台经共享根读仓）就攒在 /tmp。弱：只占盘、不改判定。

### 钩子自己被停时，判法在另一个会话里接着跑

`.claude/hooks/bash-command-detector.sh:3701`：「…start_new_session=own_group) as process:」把判法放进新会话，钩子停得了它；可钩子自己被 Claude Code 按钩子时限停掉时（`.claude/settings.json:83` 那一条注册没写 timeout，缺省时限多长、到点怎么停，是我的印象：约 60 秒、停进程组），判法不在钩子那一组里，接着跑到 git 自己结束，`finally` 照常删拷贝。一条命令里几条 git add 与 commit -a 各起一次判法、每次 25 秒，三次就过 60 秒（第一轮已攻过「没有总预算」，这里只说前一批改成新会话之后多出来的残留）。格 `hook-stopped-judge-keeps-running`：同样的慢 git 睡 20 秒，探针 8 秒就停掉钩子那一组。推的：`judges_alive_after_hook_stopped=1`，约 20 秒后自己退，`index_copies_after=0`。弱：判法只读，残留一会儿就走。

### 不成立的几样

- 索引上锁：判法读拷贝，git 的锁落在拷贝旁边。格 `index-locked-by-another-session`（`.git/index.lock` 先建好，喂 `git commit -qam x`）推的：`hook_exit=0`、不记、锁原样。
- 大仓：找「带进超限 blob 的那一笔」逐笔起一次 `git diff-tree`（`scripts/gate/large-file-check.py:215`），只在查到超限 blob 之后才走。本仓量的：`git rev-list --count HEAD` 打 `427`；对最近 50 笔逐笔起 `git diff-tree -r -m --root --no-commit-id --no-renames -z` 合计 `real 0m0.340s`，约 7 毫秒一笔；`git rev-list --objects --no-object-names HEAD | wc -l` 打 `73404`、`real 0m0.097s`。按这个数 25 秒够查约三千五百笔没推的提交（推的），本仓整个历史也只有 427 笔。这是本机数（本机盘、别的会话在跑），第二台没量。
- `-c diff.relative=false`：`scripts/gate/large-file-check.py:96`：「configuration += ["-c", "diff.relative=false"]」，`git ls-files` 带 `--full-name`（`scripts/gate/large-file-check.py:349`），两种输出都从顶层起算，`guard_worktree` 按顶层拼路径。读码没找到新毛病。

### 推不出：split index 的仓

开了 `core.splitIndex` 的仓，索引文件只记一部分，另一部分在 `$GIT_DIR/sharedindex.<号>`；拷贝之后 git 刷新并写回拷贝时，会不会往用户的 `.git` 里写一份新的 sharedindex，我没核（印象里会，路径按 git 目录算、不按索引文件所在目录算）。会的话「用户的索引一个字节不写」在字面上成立、对 `.git` 不成立。本仓没开（量的：`git config --get core.splitIndex` 退 1）。格 `split-index-judge-writes-git-directory`：看 `shared_index_files_before` 与 `_after`。

## 第 8 问：写范围表的新行 —— 两样成立，一样不成立

格都只喂检出钩子（agent_type 写 `three-way-attack`，cwd 是仓根，命令不执行），`SINGLEFS_WORK_ROOT` 指运行目录，拒没拒看退出码与 stderr 里有没有「写范围」。

### 三方腿的模式盖住别的轮、别的腿 —— 成立

`.claude/hooks/agent-write-scope.tsv:61`：「three-way-attack research/prompts/*-*-output.md」，`.claude/hooks/agent-write-scope.tsv:62`：「three-way-attack research/prompts/*-*-model/**」；辩方、正推两类是同样三行（第 65 到 72 行）。定义里写的是「报告文件、模型目录」（`.claude/agents/three-way-attack.md:51`：「- 报告文件、模型目录、草稿目录，除此之外不写。…」），派发提示点名的那一份；表里的模式盖住 `research/prompts/` 下每一轮、每条腿的报告与模型目录，第一轮的冻结证据也在里面。

| 格 | 命令（cwd 仓根） | 推的结局 |
|---|---|---|
| `scope-attack-writes-other-leg-report` | `printf x > research/prompts/fg-timeout-large-file-governance-r2-sonnet-output.md` | 放行（退 0） |
| `scope-attack-writes-earlier-round-report` | `printf x > research/prompts/fg-timeout-large-file-governance-r1-opus-output.md` | 放行 |
| `scope-attack-writes-other-leg-model` | `printf x > research/prompts/fg-timeout-large-file-governance-r2-sonnet-model/planted.py` | 放行 |
| `scope-attack-writes-own-report-control` | `printf x > research/prompts/fg-timeout-large-file-governance-r2-opus-output.md` | 放行（对照） |
| `scope-attack-writes-crates-control` | `printf x > crates/planted.rs` | 拒（对照） |

能照喂的 hook 输入（前一格）：`{"tool_name": "Bash", "session_id": "probe", "agent_type": "three-way-attack", "agent_id": "probe-agent", "cwd": "/home/fy5090/code/singlefs", "tool_input": {"command": "printf x > research/prompts/fg-timeout-large-file-governance-r2-sonnet-output.md", "run_in_background": false}}`。这一格同时说明：一条腿的禁读清单管不住它写另一条腿的报告。

### 八类有没有定义实际要写、表里没给的路径 —— 不成立

逐份现读「写范围」一节对表：`experiment-designer`（`.claude/agents/experiment-designer.md:67`，两种登记加草稿目录，表第 55 到 57 行）、`mutation-triage`（`.claude/agents/mutation-triage.md:39`）、`prior-art`（`.claude/agents/prior-art.md:32`）、`sweep`（`.claude/agents/sweep.md:55`，第 7 步的 `changes.md` 也写在草稿目录）、三方三类（攻方第 51 行、辩方 `.claude/agents/three-way-defense.md:37`、正推 `.claude/agents/three-way-forward.md:36`）、`three-way-materials`（`.claude/agents/three-way-materials.md:36`，四种 `_<轮>-…md` 加两份 sha256 清单，表第 73 到 79 行）。每一样都有对得上的行。只读了定义；`claim-experiment.sh` 这类脚本自己写什么，⑫ 不读脚本正文，不在这一格里。

### `$SINGLEFS_WORK_ROOT/**` 盖住别的 agent 的草稿目录、冻结副本与工作根下的 worktree —— 成立

- 写者锁只在写闸（Write / Edit）与 ⑫ 的 `mktemp` 一处判：`.claude/hooks/lib_writer_lock.py:6`：「  .claude/hooks/bash-command-detector.sh   ⑫ 判 `mktemp -d` 的模板与 `-p`、`--tmpdir=` 指的目录在不在这个子 agent 认领的草稿目录里（own_scratch_directory）：只读，」。共用约束自己写着它只是规矩：`.claude/agent-common.md:35`：「…Bash 里的写闸看不见锁，同样不许…」。八类新行的 agent 只带 Read、Bash，一次 Write / Edit 都没有，写者锁对它们一处都不判；表又给每一类整个工作根。
- 三方腿定义写冻结副本只读（`.claude/agents/three-way-attack.md:51`：「…冻结副本（主 agent 建的开工快照、`~/.local/share/singlefs/work/<轮>-frozen/` 这类）只读…」），表给的是可写。
- 工作根下有本仓的 git worktree（量的，见第 5 问），往它的 `crates/` 写等于绕过 `crates/**` 只给实现员与调查员那两行。

| 格 | 命令 | 推的结局 |
|---|---|---|
| `scope-attack-writes-locked-draft-of-another-agent` | `printf x > <运行目录>/another-agents-draft/planted.txt`（那个目录里放了一把别的派发的 `.writer-lock`） | 放行 |
| `scope-attack-writes-frozen-copy` | `printf x > <运行目录>/some-round-frozen/planted.txt` | 放行 |
| `scope-attack-writes-worktree-under-work-root` | `printf x > <运行目录>/worktree-under-work-root/crates/planted.rs` | 放行 |

主 agent 换成真路径照喂即可，例如把落点换成 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-sonnet/planted.txt`（这一轮辩方腿的草稿目录，在我的禁读清单里，我只写它的路径、没读它）。

## 第 9 问：名单三处一致那一格 —— 成立（误判方向响红；四种静默漏判，都窄）

认法三条正则：`.claude/hooks/bash-command-detector.sh:1684`：「BREAK_SWITCH_HEADER_LINE = re.compile(r"^#   (?:BASH_COMMAND_DETECTOR_BREAK=([a-z0-9-]+)|([a-z][a-z0-9]*(?:-[a-z0-9]+)+)(?:  |$))", re.M)」，`.claude/hooks/bash-command-detector.sh:1685`：「BREAK_SWITCH_DIRECT_READ = re.compile(r"detector_break_is_set\(\s*\"([a-z0-9-]+)\"")」，`.claude/hooks/bash-command-detector.sh:1688`：「BREAK_SWITCH_BASH_LAYER = re.compile(r"!= \*,([a-z0-9-]+),\*")」。今天三处一致（自证报过；模型的 `baseline` 格推的 `mismatches=0`）。`name_sets_model.py` 拿被判源文件的十一种改写喂原样取出的 `break_switch_set_mismatches`：

| 格 | 改写 | 开关是不是确实一致 | 推的结局 |
|---|---|---|---|
| `table-two-items-comma-on-one-line` | 开关表 `allow-large-file-push` 那一行写成 `=allow-large-file-push,commit-all-flag-unseen`，删掉 `commit-all-flag-unseen` 自己那一行 | 是 | false-mismatch：逗号后那一个不算进文件头 |
| `table-two-items-or-on-one-line` | 同一行写两个 `BASH_COMMAND_DETECTOR_BREAK=…`，中间「或」 | 是 | false-mismatch：每行只取开头那一个 |
| `table-name-in-backquotes` | `BASH_COMMAND_DETECTOR_BREAK=` 后面的名字套反引号 | 是 | false-mismatch |
| `table-name-on-continuation-line` | `BASH_COMMAND_DETECTOR_BREAK=` 之后换行，名字在下一行 | 是 | false-mismatch |
| `table-two-space-indent` | 那一行的 `#` 后面只有两个空格 | 是 | false-mismatch |
| `header-prose-line-looks-like-entry` | 文件头加一行说明 `#   git-commit-a  这一行是说明文字，不是开关` | 是 | false-mismatch：带连字符的词后面两个空格就当开关 |
| `code-read-single-quoted-undocumented` | 代码里加 `detector_break_is_set('probe-undocumented-switch')`，名单与文件头都不写 | 否 | missed-silently |
| `code-read-through-a-name-undocumented` | 同上，名字放进常量再传 | 否 | missed-silently |
| `bash-layer-equal-form-undocumented` | bash 那一层写 `== *,probe-undocumented-switch,*` | 否 | missed-silently |
| `switch-removed-but-still-in-section-list` | 把 `commit-all-flag-unseen` 从名单、开关表、代码里都删掉，㉒ 那一节「弄坏开关：」一串里还写着 | 否（节里写着不存在的开关） | missed-silently |

- 响红的六种：写的人照拒绝说明改一下写法就过，害处是逼着开关表只能一种写法；不让错的东西进来，算弱。
- 静默的四种：前三种是代码里有一条开关、名单与文件头都没有，三处一致那一格看不见；`break-switch-replay` 也只从文件头取开关（`.claude/gate.d/code-tooling.sh:52`：「# gate-cell: break-switch-replay 登记表里的脚本与钩子，文件头每条弄坏开关逐条设上跑它的 --selftest，点名的格与实际红的格逐格相同」），这条开关不会被复跑。第四种：各节文字里「弄坏开关：」那一串（例 ㉒ 那一节从第 469 行起）是第四处写开关名的地方，不在三处里，删开关、改名之后那一串过时没有格判。都要有人新写一种认法之外的写法才会出现，算窄。

## 第 10 问：㉓ 出路的 clippy_lint_tail —— 不成立（出路不会被 project-code-guard.sh 拒）；附带一处说明不实

两份认法逐字同形：`.claude/hooks/bash-command-detector.sh:4044`：「CLIPPY_LINT_TABLE_START = re.compile(r"^CODE_DISCIPLINE_LINTS=\(\s*$")」与 `.claude/hooks/project-code-guard.sh:732` 同一行；读不到、没读到时钩子交空串并记判不了（`.claude/hooks/bash-command-detector.sh:4048`：「…读不到文件、没有那一段、一条都没读到交空串并记一条判不了。…」），闸不判（`.claude/hooks/project-code-guard.sh:228`：「…表读不到时不判。…」）。所以出路照抄交上去，闸要么拿同样少的表比、要么不判，拒不了。`clippy_tail_model.py` 的十二种写法推的都是 outlet-passes-guard。

| 格 | check.sh 的写法 | 出路 -D 几条 | 闸判不判 | 门禁真跑的、出路里没有的 |
|---|---|---|---|---|
| `check-script-missing` | 没有这份文件 | 0 | 不判 | （门禁也跑不了） |
| `table-as-today` | 照今天的写法（样本只放今天那张表的前三条） | 3 | 判，不缺 | 无 |
| `table-empty` | `CODE_DISCIPLINE_LINTS=(` 下一行就是 `)` | 0 | 不判 | 无 |
| `table-readonly-prefix`、`table-declare-prefix`、`table-first-item-on-opening-line` | 开头一行前面加 `readonly ` / `declare -a `，或第一条写在开头那一行 | 0 | 不判 | 三条全缺 |
| `table-two-lints-on-one-line` | 一行写两条 | 3 | 判，不缺 | `clippy::cast_sign_loss` |
| `table-attached-deny`、`table-long-deny`、`table-quoted-items` | `-Dclippy::…`、`--deny clippy::…`、`"-D" "clippy::…"` | 2 | 判，不缺 | 那一条 |
| `table-rustc-lint-added` | 表里加一条不是 clippy 的 `-D unsafe_code` | 3 | 判，不缺 | `unsafe_code` |
| `table-closing-paren-indented` | 收尾的 `)` 缩进 | 3 | 判，不缺 | 无 |

- 出路比门禁少的那几种：照抄的 clippy 过得了两道闸，到合入后验证（跑 check.sh 的整张表）才红。这是两道闸共用的盲区，不是出路这一处带出来的；出路照两道闸现在的认法给，算不上打中这一问。
- 附带：表读不到时出路的说明写「project-code-guard.sh 要它写全」（`.claude/hooks/bash-command-detector.sh:4089`：「…（这一次没读到那张表），project-code-guard.sh 要它写全"」），可那一刻闸读的是同一份读不到的表，它不判，说明不实。闸那边读不到也不记检出（`code_discipline_lints` 直接交 None），表的写法一变 ⑤ 就整道静默停掉，只有检出钩子这边在出 ㉓ 拒绝说明时记一条。不归第 10 问的字面，写在这里交主 agent 判。

## 第 11 问：派发闸 ④ 对八类派发提示 —— 成立（新的误拒；攻方腿点名辩方腿报告照放）

`dispatch-prompt.py` 这一批只改了注释与自证的样本类型名（量的：`git diff HEAD --stat -- scripts/agents/dispatch-prompt.py` 打 `1 file changed, 4 insertions(+), 4 deletions(-)`），判法没动；变的是 ④ 读的表：④ 只判表里有行的类型（`.claude/hooks/runner-dispatch-guard.sh:21`：「#   ④ 写范围：类型在 `.claude/hooks/agent-write-scope.tsv` 里的，…」），八类加了行之后它们的提示第一次被 ④ 判。

④ 认「写进 / 写到 / 更新」后面紧跟的路径（`.claude/hooks/runner-dispatch-guard.sh:1089`：「SCOPE_MENTION = re.compile(r"(?:写进|写到|更新)…」），只看前面六个字以内有没有否定词（`.claude/hooks/runner-dispatch-guard.sh:23`：「#      前面六个字以内有否定词的不判；报告行里还有 <填 …> 占位的交 ⑬，这里不判。」），分不出这句话是叫这个子 agent 写，还是转述主 agent 或别的子 agent 写。`dispatch_scope_model.py` 原样取出 ④ 那几段，用被判那棵树的写范围表判十二句：

| 格 | 类型 | 句子 | 本意叫它写吗 | 推的结局 |
|---|---|---|---|---|
| `sweep-input-lists-what-main-agent-updated` | sweep | 「这一阶段做成的事」列表里一行「更新 .claude/kb/tooling.md 的任务服务一节」 | 否（定义要主 agent 在输入里列这一阶段做成的事） | false-refusal |
| `prior-art-says-who-writes-kb` | prior-art | 「结论要进 kb 的，由主 agent 交写回员写进 .claude/kb/prior-art.md。」 | 否 | false-refusal |
| `attack-leg-told-where-verdict-goes` | three-way-attack | 「主 agent 收齐之后把判决写进 research/prompts/<轮>-main-verification.md。」 | 否 | false-refusal |
| `designer-told-what-runner-writes-later` | experiment-designer | 「执行员之后把实验页写进 .claude/kb/experiments/e999.md，你只写登记。」 | 否 | false-refusal |
| `triage-told-who-adds-assertions` | mutation-triage | 「补断言由主 agent 另派实现员写进 crates/…/probe.rs。」 | 否 | false-refusal |
| `prior-art-negated-kb` | prior-art | 「不要写进 .claude/kb/prior-art.md。」 | 否 | correct（否定词） |
| `sweep-candidate-table-in-work-root`、`attack-leg-own-report`、`designer-own-preregistration` | 各类 | 写自己该写的 | 是 | correct |
| `materials-told-to-write-judged-list` | three-way-materials | 「开工快照写进 research/prompts/<轮>-snapshot/judged-sha256.txt」 | 是，而定义只给两份清单 | correct（拒） |
| `attack-leg-given-defense-leg-report` | three-way-attack | 「报告写进 research/prompts/<轮>-sonnet-output.md」 | 是，可那是辩方腿的报告 | outside=[]：④ 放过（模式盖住别的腿，第 8 问那一条在派发这一侧的样子） |
| `defense-leg-told-earlier-model-may-be-updated` | three-way-defense | 「…改完更新 research/prompts/<前一轮>-sonnet-model/probe.py 里的格名。」 | 本意含糊 | outside=[]：照放 |

误拒是响红：主 agent 改个说法就过，害处是派发要多绕一次；这几种转述句以前不判、现在判。模型只量「写进 / 写到 / 更新」那一支，报告行那一支（与 ⑰ 共用的 `report_line_mentions`）换成恒空，没量。真喂派发闸要建锁、读会话记录，探针不碰它（会在点名的草稿目录里建写者锁），这一格只有模型。

## 打中之后的四句

这一轮没有臂，「分不分辨」按「现实现中、对应的改法不中」读。

| 打中的一组 | 分不分辨 | 钩子那一刻看不看得到 | 满足的判据分句 | 改法还中不中（推的） |
|---|---|---|---|---|
| 别名链、同一条里配的别名、六层别名、外部子命令 | 分辨 | 看得到：配置与 PATH 在，同一条命令里的 `git config` 在命令文本里 | 第 5 问「照样静默放过」 | 「查不到的子命令按会起 commit 判」不中 |
| python / awk / rebase --exec / submodule foreach / push 递归配置 | 分辨 | 看得到：切词模块已交出这些起法（⑬ 在用） | 同上 | 「㉒ 接 embedded_launches」不中；递归配置要另读 `push.recurseSubmodules`、`submodule.recurse` |
| 名单里按操作数写文件的命令、工作根下的仓 | 分辨 | 看得到：选项与落点在命令里，仓的顶层在判法里算得出 | 同上 | 「名单收窄成按选项判、落点在这个仓里的不算无害」不中 |
| 工作根照 `~` 展开 | 分辨 | 看得到：`lib_work_root.py` 就在旁边 | 第 5 问误拒（在第二台） | 「改经 lib_work_root」不中 |
| env -u | 分辨 | 看得到 | 第 5 问静默放过 | 「切词模块认 env -u」不中 |
| 九种误拒 | 分辨 | 看得到：命令名与子命令名 | 第 5 问误拒 | 「名单加 chmod、rm、mkdir、touch、mktemp、tee 到工作根、stash list、worktree list」不中；`$TMPDIR` 那格要代入已知环境 |
| 新分支、快进合并再推 | 分辨 | 新分支那格看得到「引用解析不了」；合并那格看不到合并之后的 HEAD（判别子观测不到，改判据：前面有会改历史的命令就拒） | 第 6 问 | 「解析不了的引用记检出」只把新分支那格落到 passed-recorded；「push 也看 mutator」两格都拒 |
| matching、remote.push、通配跨斜杠、--repo、网址远端 | 分辨 | 看得到：配置、refspec、参数都在 | 第 6 问 | 各自的改法不中（见「改法」） |
| 索引拷贝残留、判法成孤儿 | 分辨 | 看得到 | 第 7 问「残留」 | 「拷贝建在判法自己的会话目录、钩子停时连判法一起停」不中 |
| 三方模式、工作根盖住别人 | 分辨 | 看得到：派发提示点名了报告与草稿目录 | 第 8 问 | 「写时照派发点名的路径判」不中 |
| 名单一致性的静默四种 | 分辨 | 看得到：源文件全文在 | 第 9 问漏判 | 「代码里出现 detector_break_is_set 的任何写法都要求是字面双引号」不中 |
| ④ 误拒 | 分辨 | 看不准：一句话是不是转述，只有字面 | 第 11 问 | 改判据（④ 只判报告行与写明「你写」的句子）；改完是新一轮 |

## 改法与各修哪一格

全部是我提的，只在我的模型上量过、被攻过零轮——而且一格都没量：改法都没实现，下表每格都是「推的」（主 agent 的复跑只量现实现，A/B 要另做）。

| 改法 | 改哪里 | 修的格 | 证据 |
|---|---|---|---|
| ㉒ 接切词模块的 `embedded_launches`（⑬ 已在用），里面认出的 git commit / git push 照命令位置上的判；认不出写法的按「判不准」拒 | 检出钩子 `git_command_steps` | `commit-in-python-c`、`commit-in-awk-system`；rebase --exec、submodule foreach 要切词模块另认这两种 git 子命令的命令串 | 推的 |
| 解析完仍不是内建子命令的（查不到别名、超过层数、值以选项起头、外部子命令）记一条检出；同一条命令里排在前面有 `git config alias.` 的按「判不准」拒；`!` 别名值里的 `git <名>` 接着照别名查 | `resolved_git_subcommand` 与 `large_file_findings` | 别名四格、`external-subcommand-on-path`（落到 passed-recorded 或 rejected） | 推的 |
| 读 `push.recurseSubmodules`、`submodule.recurse` 配置（`-c` 与仓里的），取值 on-demand / only 照命令行那一种拒 | `git_push_form` 之后 | `push-recurse-from-config` | 推的 |
| 「读不改」名单按选项与程序文本收窄：sort 带 `-o` / `--output`、uniq 给两个操作数、sed 脚本里有 `w` / `W` / `e`、awk 程序文本里有 `>`、`|`、`system` 的算会改 | `command_changes_nothing` | 三格 `read-only-name-*` | 推的 |
| 重定向落点在被提交的那个仓里（落点所在目录的仓顶层等于这条 git commit 的仓顶层）不算无害；工作根经 `lib_work_root.work_root()` 取，不经 `~` | `harmless_write_target` | `redirect-into-repository-under-work-root`、`harmless-work-root-another-home` | 推的 |
| 切词模块认 `env -u <名>` 从这一条的环境里删掉那个名字 | `lib_shell_words.py` 的 `skip_prefixes`（共用模块，⑫ 等一起变） | `env-unset-private-index` | 推的 |
| 名单补带不进内容的：chmod、rm、rmdir、mkdir、touch、mktemp；git 子命令按（子命令，第一个参数）认 `stash list`、`stash show`、`worktree list`；tee 的落点与重定向同一套无害判法；`$名字` 照这段命令里已知的环境代入 | `READ_ONLY_COMMAND_NAMES`、`command_changes_nothing` | 九格误拒 | 推的 |
| 引用此刻解析不了的记检出（`not_run` 写原因）；push 那一支同 commit 那一支看 `step.mutator`，前面有会改历史或引用的命令按「判不准」拒 | `push_tips`、`large_file_findings` push 那一支 | `push-branch-created-in-same-command`、`push-after-fast-forward-merge` | 推的 |
| 不带 refspec 时照 `push.default` 与 `remote.<名>.push` 取要推的引用（matching 取两边同名的分支；配置的 refspec 照它）；通配 refspec 照 git 的规则匹配（`*` 跨斜杠，列 `refs/` 全部再自己比） | `git_push_form`、`push_tips` | matching 两格、`push-remote-push-refspec-config`、`push-wildcard-refspec-nested-branch` | 推的 |
| `--repo` 与位置参数都给时取位置参数；推向网址或路径的，对照 URL 对得上的那个已配置远端，对不上的照「没有跟踪引用」记检出，不对照全部远端 | `git_push_form`、`push_remote` | `push-repo-option-with-positional-remote`、`push-to-path-another-remote-has-it` | 推的 |
| 索引拷贝放进钩子建、钩子删的目录（`--index-copy-directory`），到点停组之后钩子自己删；判法自己设一个比钩子时限短的闹钟（`signal.alarm`），钩子被停也活不过它 | 两份 | `judge-timeout-leaves-index-copy`、`hook-stopped-judge-keeps-running` | 推的 |
| 判法起 git 另带 `-c core.splitIndex=false`（拷贝写回成整份，不碰 `.git` 里的 sharedindex） | `large-file-check.py` 的 `git()` | `split-index-judge-writes-git-directory`（若打中） | 推的 |
| ⑫ 认写者锁（与写闸同一份 `writer_verdict`）；三方腿的报告与模型行改成在写时对照派发点名的那一份（锁里记派发提示点名的报告路径） | ⑫、`lib_writer_lock.py`、写范围表 | 第 8 问两组六格 | 推的 |
| 名单一致性：代码里 `detector_break_is_set(` 的参数不是字面双引号串的判红；bash 层认 `==` 与 `!=`；各节文字里「弄坏开关：」一串当第四处 | `break_switch_name_sets` | 第 9 问静默四种 | 推的 |
| ㉓ 表读不到时说明改成「闸这一刻也读不到表，不判；照 check.sh 补全」；闸那边读不到记检出 | 检出钩子 `whole_target_build_messages`、`project-code-guard.sh` 的 `code_discipline_lints` | 第 10 问附带那一条 | 推的 |
| ④ 只判报告行与写明叫这个子 agent 写的句子（「你写进」「写进你的」这类），转述别人写的不判 | `runner-dispatch-guard.sh` 的 `SCOPE_MENTION` | 第 11 问五格误拒；改判据，是新一轮 | 推的 |

## 要主 agent 跑的命令

探针与模型不起 cargo、不点验证包，没写范围声明；范围声明闸若把它们认成实验复跑而拒，照拒绝说明补一句（种类 items，跑什么写「攻方腿探针」）。

1. 先跑一小段估全量挂钟（误拒与写范围两组，最快的两组）：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-opus -- python3 /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r2-opus-model/probe.py --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-opus-estimate-run --only misreject,write-scope`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-opus`（草稿目录）。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 `PROBE name=frozen-hash` 七行都是 `match=yes`，`group-seconds-misreject`、`group-seconds-write-scope` 两行的秒数，没有 `group-error-` 行，末行 `PROBE name=done emitted=<N>`。估全量：这两组共 19 格；全量约 70 格，另加「判法到点」25 秒、「钩子被停」最多 30 来秒、五格各写一份 100 MiB；按比例乘，超过 40 分钟就别交第 2 条，交回我缩格。跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-opus-estimate-run`（里面有十几个临时 git 仓）。
2. 整份复跑：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-opus-run /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r2-opus-model`
   - 在哪起：仓根（第二个参数是冻结副本，有开工快照树就换成它）。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`（退 1 是某一份模型取不出段或探针一组装置出错：`setup-error` 或 `group-error-<组>` 行）；`task-server/task log <号> --all` 里 `NAMES`、`CLIPPY`、`SCOPE`、`PROBE` 四种行逐格对本报告各节的推的结局；四份末行 `… name=done emitted=<N>` 的 N 等于各自的行数减一；`hook_leftover=` 一列都是 0。全文另在运行目录的 `name-sets.out`、`clippy-tail.out`、`dispatch-scope.out`、`probe.out`，各格钩子与命令的 stderr 在 `probe/stderr/`。
3. 门禁 code-tooling 判不判检出钩子里经 `~` 取工作根那一行（第 5 问）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 shared-data-root-literals 格; 范围=1 格; 为什么=核检出钩子 harmless_write_target 经 ~ 取工作根的那一行这一格判不判红' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check shared-data-root-literals`
   - 在哪起：仓根。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=`；`task-server/task log <号>` 里有没有点名 `.claude/hooks/bash-command-detector.sh` 的 `✗` 行。红：这一行门禁抓得到，只是两批修补没跑这一格；绿：这一格的认法漏了 `expanduser(… or "~/…")` 这一形，另记一条。

## 没打中的形状

- push：`+HEAD:refs/heads/x`、`--force-with-lease`、`--set-upstream` 已有分支、`push.default` 取 simple / current / upstream、amend 之后再推（推的都判对）；远端没有跟踪引用照设计记检出。
- commit：`git -C "$(pwd)"` 照设计记检出；`env GIT_INDEX_FILE=…` 照私有索引判；Claude Code 默认的 `git commit -m "$(cat <<'EOF' … EOF)"` 放行（heredoc 正文当数据，读码与自证那一格 `$(cat <<EOF) 喂的是数据` 一致）。
- 判法：索引上锁、`diff.relative`、本仓的历史长度（量的）。
- 写范围：八类定义的「写范围」都对得上表。
- ㉓：十二种 check.sh 写法下出路都过得了 `project-code-guard.sh`。
- 名单一致性：今天的源文件三处一致；改写的六种都响红、不静默。
- 取样范围：第 5 问 12 + 8 + 11 + 2 格，第 6 问 15 格，第 7 问 4 格加两处只读量，第 8 问 8 格，第 9 问 11 种改写，第 10 问 12 种写法，第 11 问 12 句；大文件只取「上限加一字节」一个尺寸。

## 这条腿自己的限度

- 探针与三份模型一格都没跑，「推的」结局没有一个是量出来的。git 的几处行为是我的印象、格就是核它们：`--repo` 与位置参数谁优先、别名值能不能以全局选项起头、别名链有没有层数上限、split index 时写不写新的 sharedindex、`git rebase --exec` 不带 `-i` 时不开编辑器。
- Claude Code 的钩子缺省时限与到点怎么停（第 7 问「判法成孤儿」那一格的前提）是印象，仓里没有出处；格只模拟「停掉钩子那一组」。
- 三份模型是从钩子源文件按锚串取段执行，不是起整份钩子：`dispatch_scope_model.py` 把报告行那一支换成恒空，量的只是「写进 / 写到 / 更新」那一支；派发闸其余各道（骨架行、锁、在跑的子 agent 数）没喂，不知道同一份提示会不会先被别的道拒。
- 误拒九格「日常不日常」我没去 `records/` 数（辩方腿第 4 问管），只说它们带不进超限内容而照拒。
- 第 10 问附带那一条（闸读不到表时不记检出、说明写错）不在第 10 问字面里，判不判归主 agent。
- 被判文件之外读了：`lib_shell_words.py`（`commands_at_command_position`、`scan_git`、`skip_prefixes` 一带）、`lib_work_root.py` 文件头、`lib_writer_lock.py` 文件头、`lib_write_scope.py` 前 60 行、`project-code-guard.sh` 第 225 到 800 行一带、`runner-dispatch-guard.sh` 的 ④ ⑰ 文件头与 `out_of_scope_mentions` 一带、`.claude/gate.d/shared-data-root-literals-exempt.tsv`、八类定义的输入与写范围几节、`.claude/singlefs-ai-sop/scripts/check.sh` 第 60 到 80 行。辩方腿那一份在禁读清单里，没读。
- 只读量的几样（`git worktree list`、`git for-each-ref`、`git rev-list`、`git diff-tree` 计时、`stat` 索引大小）都在本机、本仓根上跑；第二台没量。

## 没做什么

- 不判辩方那几格（第 1 到 4 问）；不替主 agent 采纳；改法都没实现，副本上也没有数。
- 没往统一任务服务交任务，判决要引的数一个都没复跑：命令在「要主 agent 跑的命令」一节。
- 没建仓副本、没建编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-opus` 里除认领与锁之外只有两份比对开关名单的草稿文本（`code22.txt`、`inline22.txt`）与 cite-check 的输出。
