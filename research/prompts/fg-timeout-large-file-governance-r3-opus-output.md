# 攻方腿报告：fg-timeout-large-file-governance-r3（正文「要腿判的」第 5 到 10 问）

日期：2026-11-19。被判文件读的是主工作区此刻的那一版，开工时 `sha256sum -c research/prompts/fg-timeout-large-file-governance-r3-snapshot/judged-sha256.txt` 十五行都是 `OK`（量的）。

判词：「成立」= 这一问问的那种错在今天的实现上找得到（打中）；「不成立」= 找不到；「推不出」= 靠仓里读得到的东西判不出（派发提示定为打中）。
子 agent 不交任务，探针与模型一格都没跑：每格「会判成什么」是读码推的，标「推的，待主 agent 跑」；只读命令现跑出来的标「量的」。git 自己的几处行为（相对 core.hooksPath 从哪解、as-is 提交放不放索引锁、起过钩子之后重读索引）是我按 git 源码的印象加本机 git 2.43 的 man 页推的，探针的格就是核它们。

## 复跑命令与模型文件

模型目录 `research/prompts/fg-timeout-large-file-governance-r3-opus-model/`：

| 文件 | sha256 |
|---|---|
| `probe.py` | `476db527d32d27cf047664ab6047f2a3b1c0ab4bb45b9c1e8a4bdd549160ab01` |
| `lints_model.py` | `12d49d36f16db513da1b306b9ad9a22f975adfc4e6cc6308e8ac1842ec7383a6` |
| `tools/rerun.sh` | `5a5096e4eba47120022f89e6001ffa2c62d0eb96c74491305976f21b2918c6d9` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

复跑（整条在「要主 agent 跑的命令」第 2 条）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-opus-run /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r3-opus-model
```

- 第二个参数（冻结副本）填的是仓根（这一轮没有另建冻结树）；`probe.py` 开头逐份核 `judged-sha256.txt` 十五行，打 `PROBE name=frozen-hash … match=yes|no`，仓根之后改了的格照跑、行里写 `match=no`。
- 从第二轮攻方探针改过的副本（`/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-r2-fixes/models/opus-model/probe.py`）拷来改了什么：留下装置函数的写法（`emit`、`group_members`、`processes_mentioning`、`stop_own_group`、`run_own_group`、`git_environment`、`git`、`write_small`、`make_big`、`bare_remote`、`blob_sizes`、`big_blobs_brought_by`、`payload_for`、`keep_stderr`、`judged_hashes`、`main`），第二轮的格一格都没留（那一组十五格 push 复跑的结局归第二轮判决管，这一份不复跑它们）。改的几处：临时仓不再经全局配置指钩子，改成仓里放三份转发桩、`core.hooksPath` 写成相对路径（与本仓的装法一样，第 5 问的绕法靠它）；转发桩每起一次往 `hook-calls.log` 记一行，格里多一列 `hooks_invoked`；`commits_of` 换成 `all_commits`（`rev-list --all`，stash 的提交也算）；远端统计换成 `big_blobs_in`（含指着 blob、tree 的标签）；新加 `wait_until_gone`、`slow_git_directory`、`wait_for_judge`、`write_lock`、`write_transcript`。
- `lints_model.py` 是新写的：被判的 `lib_code_discipline_lints.py` 原样导入，表的几种写法交给 bash 自己展开当「门禁真传的」，逐词比。
- `tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`：先跑 `lints_model.py`（输出 `<运行目录>/lints.out`），再跑 `probe.py`（`<运行目录>/probe.out`），两份都跑完才退，有一份退非 0 就退 1。外层包装从 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh` 原样拷（只删工作根下以 `-run` 收尾的那一个运行目录，别的退 2）。
- 样本进程：每条命令、每次喂钩子都 `start_new_session=True` 起，超时只 `os.killpg` 自己起的那一组；信号那三格只对自己起的那一组发 TERM / INT / HUP，钩子起的判法在它自己的会话里，探针不停它，按 /proc 里的命令行等它自己退（最多 60 秒）。
- 内存与时间（推的，没跑过）：约 50 格。大头是约三十次 `git add` 一份上限加一字节的稀疏文件（git 读 100 MiB 零字节、压成约百 KB 的 blob，每次一秒上下）、浅克隆那格真写出一份 100 MiB 的工作区文件、约四十五次喂检出钩子与项目代码闸（每次一两秒）、两份新脚本各整份跑一次自证（第 10 问，一两分钟）、慢判法的五格（6 到 8 秒的睡眠各一次）、在被判的仓上判一次全部历史（git 那两步本机量的约 1.7 秒，见第 6 问）。估全量 5 到 10 分钟，远低于 40 分钟；内存几百 MB 以内；运行目录峰值约 300 MiB。没有崩溃状态，第 3b 步的崩溃枚举与状态数不适用。
- 探针、模型与包装都没跑过；`python3 -B -c 'import ast…'` 与 `bash -n` 的语法检查过了（量的，退 0）。

## 各格判定一览

结局列是探针那一格的格名与推的结局（待主 agent 跑）；第 9 问表的写法那几格是 `lints_model.py` 的行。

| 问 | 格 | 判 | 推的结局 |
|---|---|---|---|
| 5 | `git merge --no-verify`、`git am`、`commit-tree` 加 `update-ref`、`git stash` | 不成立（推不出去）：提交进了、推送被 pre-push 拦 | `merge-no-verify-then-push`、`am-then-push`、`commit-tree-update-ref-then-push`、`stash-then-push-stash` 都 commit-lands-push-refused |
| 5 | `--work-tree` / `--git-dir` / `GIT_DIR` 让 git 的「钩子运行的目录」不是这棵树的顶层：相对的 core.hooksPath 解到别处，钩子一个不起；㉒ 对改了仓位置的一概不判 | 成立：提交与推送都静默进 | `work-tree-option-commit`、`git-dir-option-push-from-outside`、`git-dir-variable-push-from-subdirectory` 都 missed-silently、`hooks_invoked=none` |
| 5 | 检出不带钩子目录的提交的树（链接工作树、`sparse-checkout`）：相对路径解到没有钩子的目录；㉒ 的「钩子没装上」看同一个目录，判成「不是本项目的树」 | 成立：提交与推送都静默进。今天本仓三棵链接工作树都是这样（量的） | `old-commit-linked-worktree-commit-and-push`、`sparse-checkout-commit-and-push` missed-silently |
| 5 | 推指着 blob 或 tree 的标签 | 成立：pre-push 起了、判法只认能解成提交的 sha，一个都没有就当判过，连 ⊘ 都不写 | `tag-to-blob-push`、`tag-to-tree-push` missed-silently、`hooks_invoked=pre-push` |
| 5 | 命令行里任一个祖先进程带一个 `--selftest` 词（`git commit -m --selftest`、`bash -c '…' --selftest`），钩子就认自证用的门限、时限、PATH 与弄坏开关 | 成立：静默进 | `selftest-word-in-message-break-switch`、`selftest-word-as-shell-name-limit-raised-push` missed-silently |
| 5 | PATH 前面放一个假 `python3`（GIT_EXEC_PATH 同理） | 成立（有意绕开，与 `--no-verify` 同一类，㉒ 不认） | `path-python3-stub-commit-and-push` missed-silently |
| 5 | 同一条命令里 `git config --remove-section core` / `--rename-section core …` 再提交 | 成立：㉒ 只认点名 core.hooksPath 的 git config | `config-remove-section-then-commit`、`config-rename-section-then-commit` missed-silently |
| 5 | `GIT_CONFIG_GLOBAL` / `GIT_CONFIG_SYSTEM` 指到设了 core.hooksPath 的文件 | 不成立：仓里的配置盖过全局与系统的 | `global-config-hooks-path-control` refused-by-git-hook |
| 5 | 子模块 | 不成立（对本仓的远端）：子模块的提交推到它自己的远端；见正文 | 无格 |
| 6 | 只删远端分支、只推指着已推提交的标签 | 不成立：放行 | `push-delete-remote-branch-only`、`push-tag-on-pushed-commit-only` passed-nothing-lands |
| 6 | 远端（写成路径）已有带超限 blob 的历史、又被别处推进了一笔，本地 `--force` 推 | 成立（误拒，窄）：判全部历史，拦下远端早就有的 blob | `push-path-remote-advanced-elsewhere-force` refused-by-git-hook、`remote_had_big_before=yes` |
| 6 | 推向配置过的远端名、跟踪引用带着那笔提交 | 不成立：放行 | `push-configured-remote-already-has-big-control` passed-nothing-lands |
| 6 | 远端没有跟踪引用、判全部历史 | 不成立（本仓本机）：git 那两步约 1.7 秒（量的）；600 秒在 Claude Code 前台够不着（Bash 时限至多 240 秒，到那里整条命令先被停） | `judge-unknown-full-history-elapsed` |
| 6 | 收窄后的 ㉒ 对日常写法 | 成立（弱）：只有「把 core.hooksPath 设成正确值」这一种被拒，日常提交写法都放 | `detector-*` 六格 |
| 7 | `--known` 的对象本地没有（远端被别处推进过） | 不成立（不漏）：配置过的远端名有跟踪引用兜着；路径远端改判全部历史（偏严） | `force-push-remote-sha-unknown-locally-*` refused-by-git-hook |
| 7 | 浅克隆推新分支 | 推向配置过的远端名不成立；推向路径成立（误拒，窄）：浅克隆的边界提交整棵树都算没推过 | `shallow-clone-new-branch-by-path` refused-by-git-hook、`…-by-configured-name-control` passed-nothing-lands |
| 7 | 远端 sha 全 0 | 不成立 | `new-branch-remote-sha-all-zero-control` refused-by-git-hook（该拒） |
| 7 | 推的本地 sha 不是提交（标签指着 blob、tree） | 成立：静默放过（同第 5 问那两格） | 同上 |
| 7 | 并发提交：as-is 的 `git commit` 在钩子跑着时放开索引锁，git 起过钩子之后重读索引；别的会话这时 `git add` 的超限文件不经判就进提交 | 成立（竞态）：判的是拷贝那一刻，进的是重读那一刻 | `race-add-during-as-is-commit` missed-silently；对照 `race-add-during-commit-all-control` 加不进去 |
| 7 | 钩子收到 TERM、HUP（Bash 到点、终端断开） | 成立（弱）：索引拷贝的目录留在 TMPDIR，判法成孤儿跑到自己结束；INT 时钩子删了目录、判法照样成孤儿 | `hook-sigterm-mid-judge`、`hook-sighup-mid-judge`、`hook-sigint-mid-judge` |
| 8 | 只带 Read、Bash 的定义写自己的草稿目录 | 不成立（今天的写法）：放行并认领；草稿目录里拷进了别人的锁才误拒（窄） | `lock-own-draft-holder-control` 放行；`lock-copied-lock-inside-own-draft` 拒 |
| 8 | 写别人的目录：别人持锁的拒；只有 `.owner` 没有锁的、什么都没有的（本仓的 git 链接工作树）、派发回执读不到的、带 Write / Edit 的定义在 Bash 里写的，都放 | 成立 | `lock-owner-only-directory-of-another-session`、`lock-unowned-worktree-like-directory`、`lock-other-draft-unclaimed-receipt-missing`、`lock-write-edit-definition-into-claimed-draft` 放行 |
| 8 | 三方腿模式：别的轮同一模型的报告与模型照写得进；第二次抽样（`-s2-`）自己的报告与模型写不进 | 成立（两头都有：放过与误拒） | `scope-attack-earlier-round-*`、`scope-defense-…`、`scope-forward-…` 放行；`scope-attack-own-second-sample-*` 拒 |
| 9 | 表的写法：`+=` 追加、后面再赋值、`\` 续行、花括号展开、元素是另一个数组、`-F` / `--forbid` | 成立：两边一起读错（漏要求的四种、读不到而 ⑤ 整道不判的一种、多要求而误拒的一种） | `lints_model.py` 七行 |
| 9 | 命令那一侧：`-D` 之后再 `-A` 同名、`--cap-lints allow` | 成立：⑤ 照放，lint 实际关了 | `command-allow-after-deny`、`command-cap-lints-allow` guard_passes=yes |
| 9 | `LIB_CODE_DISCIPLINE_LINTS_BREAK` 平常设了 | 成立：两道钩子都不当它是弄坏开关拒，照认；⑤ 拒掉每一条写全了的 clippy，㉓ 出路成了 `-D warnings -D -D …` | `lints-switch-set-*` 两格；对照 `lints-switch-unset-control-*` |
| 10 | 单跑一格 | 推不出（算打中）：两份脚本都没有单跑一格的入口，`--selftest-inner` 后面点格名照跑整份；七组格读前一格留下的仓、一格读整份共用的 TMPDIR | `selftest-inner-with-cell-name-*` cells_run=28 / 13 |
| 10 | 开关轮因为次序把一格判绿 | 不成立（读码）：每个开关一个新进程、一个新临时目录，点名的格都是开关的直接后果 | 无格 |

## 第 5 问：git 钩子当主闸之后剩下的绕法 —— 成立

判据字面：超限文件进了提交或推出去，而检出钩子 ㉒、pre-commit、pre-merge-commit、pre-push 都没拦。每格的装置：临时仓里进仓的钩子目录 `scripts/gate/git-hooks/` 放三份转发桩（exec 被判那棵树里的同名钩子），`core.hooksPath` 写相对路径 `scripts/gate/git-hooks`（本仓就是这样装的，量的：`git config --show-origin --get-all core.hooksPath` 打 `file:.git/config	scripts/gate/git-hooks`）；远端是本地裸仓（没有单文件上限，推得上去才看得出漏没漏）；超限文件是上限加一字节的稀疏文件。

### 逐种写进没进、谁拦的

| 写法 | 进提交 | 推出去 | 谁拦 | 格 |
|---|---|---|---|---|
| `git merge --no-verify` | 进（pre-merge-commit 不起；㉒ 不看 merge） | 不进 | pre-push | `merge-no-verify-then-push` |
| `git am` | 进（pre-applypatch 没接，文件头自己写了） | 不进 | pre-push | `am-then-push` |
| `git commit-tree` 加 `git update-ref` | 进（不起任何钩子） | 不进 | pre-push | `commit-tree-update-ref-then-push` |
| `git stash push -- big.bin` 再推 `refs/stash` | 进（stash 的两笔提交不起钩子） | 不进 | pre-push | `stash-then-push-stash` |
| `git --work-tree=<别的目录> commit` | 进 | — | 没人 | `work-tree-option-commit` |
| 在这棵树外面 `git --git-dir=<仓>/.git push` | — | 进 | 没人 | `git-dir-option-push-from-outside` |
| 在子目录 `GIT_DIR=<仓>/.git git push` | — | 进 | 没人 | `git-dir-variable-push-from-subdirectory` |
| 链接工作树检出不带钩子目录的提交，在里面提交、推 | 进 | 进 | 没人 | `old-commit-linked-worktree-commit-and-push` |
| `git sparse-checkout set` 把 `scripts/` 剔出工作区再提交、推 | 进 | 进 | 没人 | `sparse-checkout-commit-and-push` |
| 标签指着 blob / tree 再推 | — | 进 | 没人（pre-push 起了、判成判过） | `tag-to-blob-push`、`tag-to-tree-push` |
| `git commit -qm --selftest` 加 `LARGE_FILE_GIT_HOOK_BREAK=pre-commit-judge-skipped` | 进 | — | 没人 | `selftest-word-in-message-break-switch` |
| `bash -c 'git push …' --selftest` 加 `LARGE_FILE_GIT_HOOK_LIMIT_BYTES=999999999999` | — | 进 | 没人 | `selftest-word-as-shell-name-limit-raised-push` |
| `PATH=<放了假 python3 的目录>:"$PATH" git commit / git push` | 进 | 进 | 没人 | `path-python3-stub-commit-and-push` |
| `git config --remove-section core && git commit`（`--rename-section` 同） | 进 | — | 没人（同一条命令里钩子判的是改之前的配置） | `config-remove-section-then-commit`、`config-rename-section-then-commit` |
| `GIT_CONFIG_GLOBAL=<设了 hooksPath 的文件> git commit` | 不进 | — | pre-commit（仓里的配置盖过全局的） | `global-config-hooks-path-control` |
| 对照：普通 `git commit`、普通 `git push` | 不进 | 不进 | pre-commit / pre-push | `control-commit-staged-big`、`control-push-unpushed-big` |

前四行是「进了提交、推不出去」。前两轮判过「本地留下一笔、推被拒」不算打中（背景材料「验过的」最后一条），这四行照同一口径记不成立；`git am` 的 pre-applypatch 没接是文件头写明的：「git am 起 pre-applypatch（能判索引，没接）」（`scripts/gate/large-file-git-hook.py:30`）。

### 相对的 core.hooksPath：git 换了「钩子运行的目录」，钩子就没了

本机 git 2.43 的 `git-config(1)` 写 core.hooksPath 的相对路径「taken as relative to the directory where the hooks are run」，`githooks(5)` 写钩子起之前 git 换到「the root of the working tree in a non-bare repository」（转述，man 页不在仓里）。显式给了 `GIT_DIR`（`--git-dir`）而没给工作树时，git 把当前目录当工作树的顶层；给了 `--work-tree` 的，`git commit` 先换到那个目录（印象，按 git 的 setup.c）。所以：

- `git --work-tree=/某个空目录 commit -qm x`：暂存区里已经 `git add` 过的超限文件照进提交，git 在那个空目录里找 `scripts/gate/git-hooks/pre-commit`，找不到，不报错、不起钩子。
- `git -C /tmp --git-dir=<仓>/.git push origin main`、在仓的子目录里 `GIT_DIR=<仓>/.git git push origin main`：同理，pre-push 不起。

㉒ 对这几种一概放：「--git-dir / --work-tree / GIT_DIR / GIT_WORK_TREE」（`.claude/hooks/bash-command-detector.sh:446`）改了仓位置的不判，代码是「or location or {"GIT_DIR", "GIT_WORK_TREE"} & set(environment):」（`.claude/hooks/bash-command-detector.sh:3551`）。它管的是「钩子没装上」那一件，「绕开钩子的写法」那一件里没有这几种，可这几种恰恰是让钩子不起的写法。主 agent 已知的那一条写的是「用户自己手敲的 … `GIT_DIR` 改位置」；子 agent 与主 agent 在 Bash 里这样写同样放过，而且连 pre-push 一起绕开，不是只差 pre-commit。

能照喂的 hook 输入（主 agent 的 Bash，仓根起）：`{"tool_name": "Bash", "session_id": "probe", "cwd": "/home/fy5090/code/singlefs", "tool_input": {"command": "git -C /tmp --git-dir=/home/fy5090/code/singlefs/.git push origin master", "run_in_background": false}}`，推的 ㉒ 退 0。只喂、别执行。

### 检出不带钩子目录的提交：本仓今天就有三棵

`core.hooksPath` 写在共用的 `.git/config` 里，每棵链接工作树都解到自己那棵树里的 `scripts/gate/git-hooks/`。量的（只读）：

```
$ git worktree list
/home/fy5090/code/singlefs                                             4319aa2b0 [master]
/home/fy5090/.local/share/singlefs/work/doc-gate-commit-20261105/tree  59ce5a865 (detached HEAD)
/home/fy5090/.local/share/singlefs/work/m5-dev                         8ed2ea484 [m5-dev]
/home/fy5090/.local/share/singlefs/worktree-backlog-m4                 59ce5a865 (detached HEAD)
```

三棵别的工作树里 `ls <树>/scripts/gate/git-hooks` 都是 `No such file or directory`，`git -C <树> config --get core.hooksPath` 都打 `scripts/gate/git-hooks`（量的）。原因：三份钩子与判法这一轮还只在暂存区（`git status --short` 打 `A  scripts/gate/git-hooks/pre-commit` 这几行，量的），任何一笔已有的提交都不带这个目录；提交之后，凡是还在老提交上的树（`m5-dev` 这类长期分支、检出老提交的树、`git bisect` 途中、`git sparse-checkout` 把 `scripts/` 剔掉的树）照旧没有。

- 这几棵树里 git 一个钩子都不起，连私有的提交窗口闸也不起：设了 core.hooksPath 之后 git 不读 `.git/hooks`，私有闸自己写着「设了 core.hooksPath 之后 git 不再读 .git/hooks，由进仓的 scripts/gate/git-hooks/pre-commit 先调这一份、再判大文件。」（`.git/hooks/pre-commit:5`）。设 core.hooksPath 之前，私有闸对每棵链接工作树都起（`.git/hooks` 是共用的）；这是这一轮带出来的退步，不止大文件一件。
- ㉒ 的「钩子没装上」用同一个目录认「是不是本项目的树」：「return os.path.isfile(os.path.join(top, HOOKS_DIRECTORY, HOOK_NAMES[0]))」（`.claude/hooks/lib_git_hooks.py:29`）。目录不在就当别的仓，放行。
- 安装脚本知道这件事，只记 ⊘、不算红：「链接工作树检出的是不带这个目录的老提交时，那棵树里没有钩子（git 不报错、什么都不判）：check 对每棵别的工作树都列出来。」（`scripts/gate/git-hooks-install.py:20`）。背景材料引的 `check --both-machines` 绿行之外，那一趟应当另有三行 ⊘（推的，主 agent 读那一趟的日志可核）。

能照做的序列（在 `m5-dev` 里，只说明、别执行）：`cd /home/fy5090/.local/share/singlefs/work/m5-dev && git add <超限文件> && git commit -m x && git push origin m5-dev`，推的：两个 git 钩子都不起，㉒ 退 0。探针那一格在临时仓里摆同样的形状。

### 标签指着 blob、tree：pre-push 判成「判过、没有超限的」

判法把每个 `--ref` 解成提交：「resolved = git(top, "rev-parse", "--verify", "-q", f"{name}^{{commit}}", allow_failure=True)」（`scripts/gate/large-file-check.py:382`），解不成的略过，文档串的理由是「读不到的略过：git push 自己会报错、什么都不推。」（`scripts/gate/large-file-check.py:379`）。指着 blob 或 tree 的标签 git push 照推（`refs/tags/` 下不限对象类型），可 `^{commit}` 解不成；一个都没解成时直接交「return {**guard_result("push", limit, 0, [], []), "remote": remote}」（`scripts/gate/large-file-check.py:403`）：`not_run` 是空串，钩子不写 ⊘、退 0。

序列（格 `tag-to-blob-push`）：`git tag big-blob "$(git hash-object -w big.bin)" && git push -q origin refs/tags/big-blob`。`big.bin` 从没进过暂存区，pre-commit 碰不到；pre-push 起了（`hooks_invoked=pre-push`），推的退 0，远端多一个超限 blob。tree 那一格用 `git mktree` 造一棵只放 `big.bin` 的树、打附注标签再推。这是日常不会写的形状，但它是静默的：判法自己的「判不了就写 ⊘」在这里没接上。

### 命令行里有一个 `--selftest` 词的祖先，钩子就认自证用的开关

两份新脚本认「在不在自证里」的办法是往上数祖先进程的命令行：「if b"--selftest" in words or b"--selftest-inner" in words:」（`scripts/gate/large-file-git-hook.py:114`）。认了之后门限、时限与判法的 PATH 都从环境变量取（「return os.environ.get(name, "") if inside_selftest() else ""」（`scripts/gate/large-file-git-hook.py:125`）），弄坏开关也照认（`scripts/gate/large-file-git-hook.py:660`：「broken.update(item for item in (os.environ.get(BREAK_VARIABLE, "") if inside_selftest() else "").split(",") if item in BREAKS)」）。git 自己就是钩子的祖先，它的参数里有一个 `--selftest` 词就够：

- `LARGE_FILE_GIT_HOOK_BREAK=pre-commit-judge-skipped git commit -qm --selftest`：提交说明是 `--selftest`，钩子认作自证、跳过判法，超限文件进提交（格 `selftest-word-in-message-break-switch`）。
- `LARGE_FILE_GIT_HOOK_LIMIT_BYTES=999999999999 bash -c 'git push -q origin main' --selftest`：`bash -c` 的 `$0` 是 `--selftest`，门限抬到近 1 TB，推出去（格 `selftest-word-as-shell-name-limit-raised-push`）。

㉒ 不看这几个环境变量；开关「平常设了退 2」那一道也是同一个祖先判法，同样被骗过。`git-hooks-install.py` 用的是同一份 `inside_selftest`（`GIT_HOOKS_INSTALL_PEER_LIBRARY` 换成假的第二台库，`check --both-machines` 就判绿），我没摆格。检出钩子与项目代码闸认自证用的是标记文件加环境变量（`.claude/hooks/bash-command-detector.sh:1739` 起），两份新脚本没照它写。

### 有意绕开、㉒ 不认的另几种

- 假 `python3`：三份进仓钩子起判法前只查 `command -v python3`，然后 `exec python3 -B …`；git 往 PATH 最前面放的是它自己的 exec-path（那里没有 python3），再往后才是调用方的 PATH。`PATH=<目录>:"$PATH" git commit`（目录里放 `#!/bin/sh` 加 `exit 0` 的 `python3`）两个钩子都照「通过」退 0（格 `path-python3-stub-commit-and-push`）。`git --exec-path=<目录>`、`GIT_EXEC_PATH` 同一类，我没摆格。
- 同一条命令里先改配置：㉒ 认 git config 改 core.hooksPath 时只看点名这个键的写法（「if flags & {"--unset", "--unset-all", "--replace-all", "--add"}:」这一段，`.claude/hooks/bash-command-detector.sh:3529`），`git config --remove-section core`、`--rename-section core moved` 里没有这个键，放；随后的 `git commit` 判「钩子装没装上」读的是命令跑之前的配置，也放（两格 `config-*-then-commit`）。直接改 `.git/config`（`sed -i`、重定向）同理。
- 这三种都要人有意去写，与 `--no-verify` 同一类；㉒ 的第一件就是拦有意的绕法，所以记打中，分量比上面两节轻。

### 不成立的两种

- `GIT_CONFIG_GLOBAL` / `GIT_CONFIG_SYSTEM`：本仓的 core.hooksPath 写在仓里的 `.git/config`，仓里的配置盖过全局与系统的，指到别处的全局配置改不动它（格 `global-config-hooks-path-control` 推的 refused-by-git-hook）。它们能改判法自己起的 git 的行为（判法不隔用户配置），我逐项想了 diff、rev-list、cat-file、for-each-ref 用到的配置，没找到能让判法少报的一项（没摆格）。
- 子模块：子模块的仓有自己的配置，没有 core.hooksPath，在里面提交、推都不起钩子；但它推到的是子模块自己的远端，不是本仓的。超级项目这边，`commit-index` 判法跳过 gitlink（`GITLINK_MODE`），pre-push 的 `rev-list --objects` 不进子模块的提交，不会误拒也不会把子模块的大文件带上本仓的远端。要进本仓的远端得把子模块的远端也指到本仓，本仓今天没有子模块（量的：`ls .gitmodules` 打 `No such file or directory`；没摆格）。不成立。

## 第 6 问：pre-push 与收窄后 ㉒ 的误拒 —— 成立（窄）

判据字面：误拒 = 这一次推送什么超限的新内容都不会带上去，钩子照拒。

| 情形 | 判法读到的 | 推的结局 | 格 |
|---|---|---|---|
| 只删远端分支（`git push origin :stale`），当前分支还欠着一笔超限提交 | 本地 sha 全 0 的行不判，一行都不剩就退 0 | 放行 | `push-delete-remote-branch-only` |
| 只推指着已推提交的附注标签 | 标签解到提交，`--not --remotes=origin` 扣掉 | 放行 | `push-tag-on-pushed-commit-only` |
| 远端（配置过的名字）已有带超限 blob 的提交，跟踪引用带着它，上面加一笔小提交再推 | 行里的远端 sha 本地有、跟踪引用也扣 | 放行 | `push-configured-remote-already-has-big-control` |
| 远端写成路径，已有带超限 blob 的提交，之后被另一份克隆推进了一笔；本地加一笔小提交 `git push --force <路径> main` | 行里的远端 sha 本地没有 → 略过；路径写法不取任何跟踪引用 → 「已有的」为空 → 判全部历史，超限 blob 算进来 | 拒（误拒：那个 blob 远端早就有） | `push-path-remote-advanced-elsewhere-force` |
| 远端没有跟踪引用（新加的远端、路径写法的新引用） | 判全部历史 | 本仓本机约 2 秒，放行 | `judge-unknown-full-history-elapsed` |

- 误拒那一格的根在判法只认本地读得到的远端 sha，读不到的不当「远端已有」也不当「判不了」，直接退到全部历史；文件头写的就是这样：「两样都没有（远端没取回过、远端上也没有这些引用）时不知道远端已有什么」（`scripts/gate/large-file-check.py:37`）。只在推向路径或网址、远端带着超限 blob（GitHub 收不下，只可能是没有上限的远端，比如共享根上的备份裸仓）、而且远端被别处推进过时才碰上，窄。出路照拒绝说明重做不了（那个 blob 在远端已有的历史里，重做本地提交拿不掉它），只能 `git fetch` 那个路径再推、或用户自己 `--no-verify`；拒绝说明不提这两条（推的）。浅克隆推向路径是同一个根，见第 7 问。
- 全部历史要多久：本仓本机量的（只读）：`git rev-list --objects --no-object-names HEAD` 打 `73404` 行、`real 0m0.112s`；对这 73404 个对象跑一次 `git cat-file --batch-check` 找大于上限的 blob，打 `0`、`real 0m1.635s`；`git rev-list --count HEAD` 打 `427`、`--all` 打 `490`；`git count-objects -vH` 打 `size-pack: 686.13 MiB`。判法另起 `for-each-ref`、`rev-parse` 几次，只有查到超限 blob 才逐笔起 `diff-tree`。所以 600 秒（「JUDGE_SECONDS = 600」（`scripts/gate/large-file-git-hook.py:89`））在本仓本机碰不上；第二台经共享根读对象库没量。另一面：这个 600 秒在 Claude Code 前台够不着——前台 Bash 的时限至多 240000 毫秒（检出钩子 ㉑），判法慢过它时整条 `git push` 先被 Claude Code 停掉，推送不发生（不漏），留下的东西见第 7 问的信号三格。到点「放行并写 ⊘」只在后台与用户自己的终端里走得到。
- 收窄后的 ㉒（六格只喂、不执行，临时仓装好了钩子）：`git commit -q -m 'drop the -n flag'`、`git commit -q -F - <<'EOF' … --no-verify … EOF`、`git config --show-origin --get core.hooksPath`、普通 `git commit -qm x` 推的都放；拒的只有两种「把 core.hooksPath 设成正确值」：`git config core.hooksPath scripts/gate/git-hooks`、`git -c core.hooksPath=scripts/gate/git-hooks commit -qm x`。前者是手装钩子的写法，拒绝说明给了经任务服务装的出路，算弱的误拒；㉒ 的 `-c`、`GIT_CONFIG_*` 与 git config 三处都只看键、不看值。日常的提交写法（Claude Code 默认的 `-m "$(cat <<'EOF' … EOF)"`、`staged-gate.md` 那一套）没有被拒的形状，我只读了 ㉒ 的代码，没去 `records/` 数（正推腿第 4 问管）。

## 第 7 问：`--guard push` 只认行里的远端 sha；索引拷贝在超时、信号、并发提交下 —— 成立

### 行里的远端 sha 那一支

| 情形 | 判法怎么走 | 推的结局 | 格 |
|---|---|---|---|
| `--known` 的对象本地没有（远端被别处推进过，本地 `--force` 推），推向配置过的名字 | `existing_commits` 略过它；`--remotes=origin` 的跟踪引用（旧的）还在，照它扣 | 该拒的拒（不漏） | `force-push-remote-sha-unknown-locally-configured-remote` |
| 同上，推向路径 | 「已有的」为空 → 判全部历史 | 该拒的拒（偏严：远端已有超限 blob 时误拒，第 6 问） | `force-push-remote-sha-unknown-locally-path-remote` |
| 浅克隆（`--depth 1`，边界提交带着远端已有的超限 blob）加一笔小提交推新分支，推向配置过的名字 | 跟踪引用就是边界提交，扣掉 | 放行 | `shallow-clone-new-branch-by-configured-name-control` |
| 同上，推向路径 | 远端 sha 全 0、路径不取跟踪引用 → 判「这些对象能到的全部历史」：边界提交整棵树都算 | 拒（误拒，窄） | `shallow-clone-new-branch-by-path` |
| 远端 sha 全 0、推向配置过的名字、新分支带着超限提交 | 跟踪引用扣掉已推的那几笔 | 该拒的拒 | `new-branch-remote-sha-all-zero-control` |
| 本地 sha 是 blob 或 tree（标签指着它们） | `^{commit}` 解不成 → 略过 → 一个都不剩 → 当判过 | 静默放过 | 第 5 问 `tag-to-*` 两格 |

这一支只在两处出事：解不成提交的本地 sha（漏，静默）、本地读不到的远端 sha 落到全部历史（误拒，窄）。前者的改法是 `--ref` 那一边认 `^{object}`、让 `rev-list --objects` 从标签、tree、blob 起；后者是读不到的远端 sha 写 ⊘、不退到全部历史（都是推的，见「改法」）。

### 索引拷贝与判法进程：超时那一支干净，信号那一支留东西

拷贝的目录由钩子建：「index_directory = tempfile.mkdtemp(prefix="large-file-git-hook-index-")」（`scripts/gate/large-file-git-hook.py:157`），判法放进新会话（「start_new_session=True) as process:」（`scripts/gate/large-file-git-hook.py:170`）），删在 `finally` 里（「shutil.rmtree(index_directory, ignore_errors=True)」（`scripts/gate/large-file-git-hook.py:180`））。超时那一支（`communicate` 到点 → `stop_group` 停整组 → `finally` 删）自证有格（`judge-timeout-leaves-nothing`），我不再摆。钩子自己没装信号处理：

- TERM、HUP 落到钩子身上（Claude Code 的 Bash 到点停整条命令、终端断开；钩子与 git 在同一组），Python 照缺省处理当场退出，`finally` 不跑：`large-file-git-hook-index-*` 留在 TMPDIR。判法在它自己的会话里收不到信号，接着跑到自己结束（最长是判法自己的时长，600 秒那道闹钟在钩子里、跟着钩子一起没了）；它退出时只删拷贝那一个文件（调用方给了目录时它只删自己放的那一份），空目录留下。格 `hook-sigterm-mid-judge`、`hook-sighup-mid-judge` 推的：`index_copy_directories_right_after=1`、`judge_alive_after_command=yes`、`index_copy_directories_at_end=1`、`files_in_them_at_end=0`。
- INT（Ctrl-C 给整组）：Python 的 `communicate` 与 `Popen.__exit__` 遇 KeyboardInterrupt 只再等子进程约 0.25 秒就把中断抛出去（印象，按 CPython 的 subprocess），`finally` 删目录；判法照样成孤儿。格 `hook-sigint-mid-judge` 推的：`index_copy_directories_at_end=0`、`judge_alive_after_command=yes`。
- 弱：只占盘（一份索引拷贝；本仓此刻 `stat -c %s .git/index` 打 `2644559`，量的）与一个只读的孤儿进程，不改判定。第二轮在检出钩子那一侧报过同形（判法放进新会话、钩子被停时它活着）；这一份是新脚本照搬了那个形状。

三格的慢判法靠 `LARGE_FILE_GIT_HOOK_JUDGE_PATH`（只有祖先命令行带 `--selftest` 词才认，见第 5 问），格里用 `bash -c 'git commit -qm x' --selftest` 起；这只是把判法放慢、好对准时刻，信号落到钩子上的路径与平常一样。

### 并发提交：判的是拷贝那一刻的索引，进提交的是 git 重读的那一份

不带 `-a`、不点名路径的 `git commit`（as-is 那一支）在起 pre-commit 之前把刷新过的索引写回 `.git/index` 并放开索引锁，钩子拿到的 `GIT_INDEX_FILE` 是真索引；起过钩子之后 git 重读这份索引再建树（印象，按 git 的 builtin/commit.c：prepare_index 的 as-is 一支带 COMMIT_LOCK 写回，prepare_to_commit 在起过钩子之后重读索引；转述，本机没有 git 源码，原文没核）。判法读的是拷贝那一刻；拷贝之后、git 重读之前，同一个仓里别的会话 `git add` 的文件不经判就进这一笔。

- 格 `race-add-during-as-is-commit`：暂存一个小文件，`bash -c 'git commit -qm as-is' --selftest` 起（判法放慢 6 秒），看到判法进程起来之后再等 1 秒，另起 `git add big.bin`。推的：`concurrent_add_exit=0`、`commit_exit=0`、`landed_commit=yes`、`git_hook_refused=no`。
- 对照 `race-add-during-commit-all-control`：`git commit -qam all` 那一支钩子跑着时索引锁一直握着，另起的 `git add` 拿不到锁退非 0，推的 `landed_commit=no`。
- 窗口在平常就是判法跑的那一段（本仓本机一次 commit-index 判法零点几秒，没量），NFS 上与大索引时变长。共用约束写着「本机常有别的会话同仓干活」，`.claude/kb/tooling.md`「几个会话共用一个仓时怎么暂存与提交」整节讲的就是共用一个暂存区，所以这不是造出来的历史。被别的会话混进来的内容本来就是另一种错，可它带着超限文件时三道闸都不出声。
- 索引拷贝本身不会写错：每一次判法各建一个 `mkdtemp` 目录，拷的是 `GIT_INDEX_FILE` 指的整份文件；git 写索引是锁文件写完再改名，拷到的不会是半份（推的）。

## 第 8 问：⑫ 写者锁接到只带 Read、Bash 的定义；三方腿收窄后的模式 —— 成立

格都只喂检出钩子（不执行），cwd 是仓根，`SINGLEFS_WORK_ROOT` 指运行目录里的假工作根；锁与派发回执照 `lib_writer_lock.py` 文件头的格式造（`.writer-lock` 里的 `prompt_sha256` 与 `dispatched_at`，假的主会话记录里一行 `async_launched` 回执），agent_id 是 `probe-agent`。

### 写者锁那一半

只带 Read、Bash 的定义由 `definition_can_write` 认出来，设上 `SCRATCH_OWNER_IDENTITY["read_only"] = "1"`（`.claude/hooks/bash-command-detector.sh:2651`），之后每一处写的落点照最近一层的锁判，持锁的不是它就拒；回执读不到的照放：「if verdict in ("holder", "unclaimed"):」（`.claude/hooks/bash-command-detector.sh:4470`）。

| 格 | 定义 | 写到哪 | 推的结局 | 算什么 |
|---|---|---|---|---|
| `lock-own-draft-holder-control` | sweep | 自己的草稿目录（锁的提示对得上回执） | 放行，并在那个目录里建一份 `.writer-claim-*` | 对照 |
| `lock-copied-lock-inside-own-draft` | sweep | 自己草稿目录里拷进来的别人的草稿目录（带着别人的锁与认领） | 拒 | 误拒（窄）：最近一层的锁是拷来的那一把 |
| `lock-other-draft-claimed` | sweep | 别人认领过的草稿目录 | 拒 | 对 |
| `lock-other-draft-unclaimed-receipt-mismatch` | sweep | 别人的、还没认领的草稿目录，自己的回执在、提示对不上 | 拒 | 对 |
| `lock-other-draft-unclaimed-receipt-missing` | sweep | 同上，自己的回执在主会话记录里找不到 | 放行，不记检出 | 放过 |
| `lock-owner-only-directory-of-another-session` | sweep | 只有 `.owner`（别的会话认领）、没有锁的目录 | 放行 | 放过 |
| `lock-unowned-worktree-like-directory` | sweep | 什么都没有的目录（形状照工作根下本仓的 git 链接工作树） | 放行 | 放过 |
| `lock-write-edit-definition-into-claimed-draft` | clerk（带 Write、Edit） | 别人认领过的草稿目录 | 放行 | 放过：只判只带 Read、Bash 的定义 |

- 放过的三类在工作根里有多少（量的，只读）：工作根下 2887 项，带 `.owner` 的 2442，带 `.writer-lock` 的 2363，带 `.owner` 而没有 `.writer-lock` 的 134；`m5-dev`（本仓的链接工作树，分支 `m5-dev`）两样都没有。写者锁只罩派发闸建过锁的草稿目录，别的会话认领的目录、主 agent 的运行目录与冻结副本、本仓的链接工作树都不在里面。第二轮报过「工作根下的 git worktree 写得进 `crates/`」，这一轮的修补没碰到它：`m5-dev/crates/` 照写得进。
- 回执读不到的放行：写闸那一侧同一种情形「放行但必记检出」（「找不到（回执还没落盘、记录读不了）：这一次写放行（"unclaimed"），不认领；写闸照「放行但必记检出」stderr 打一行」（`.claude/hooks/lib_writer_lock.py:19`）），⑫ 这一侧 `continue` 了事、不记。本机检出记录里写闸记的「回执还没落盘」有 548 条（量的：`grep -c '回执还没落盘' ~/.local/share/singlefs/work/agent-hook-detections.jsonl`），这种情形真会发生；它在 ⑫ 这一侧不留痕。
- 带 Write、Edit 的定义在 Bash 里写别人的草稿目录：⑫ 不判锁（只判 `read_only` 的），写闸只看 Write / Edit。共用约束写的是「Bash 里的写闸看不见锁，同样不许」，仍是靠自觉。
- 误拒的那一格要子 agent 把别人的整个草稿目录（连 `.writer-lock`、`.writer-claim-*`）拷进自己的草稿目录再往里写；派发提示叫腿「可以拷过去改」第二轮的探针副本，那一份副本所在的目录连上层都没有锁（量的：`ls -la` 那一层只见 `.owner` 与 `.deliverables`，`models/opus-model/` 里只有四份 `.py` 与 `tools/`），碰不上；拷的是一份派发过、带着锁的草稿目录才会碰上。
- 附带：⑫ 判的时候真去认领。`writer_verdict` 认作持锁者时排他建认领文件（「owner = try_claim(scratch, lock, agent_id)」（`.claude/hooks/lib_writer_lock.py:347`）），而 `lib_writer_lock.py` 文件头写检出钩子这一侧「用 find_lock_directory、read_lock、read_claim、launch_receipt 与 LAUNCH_SLACK_SECONDS，不建锁、不建认领」（`.claude/hooks/lib_writer_lock.py:7`）。说明与行为不一致；后果只是认领早一步落盘（被拒的整条命令也会先认领），不改谁是持锁者。格 `lock-own-draft-holder-control` 的 `claims_created` 一列核它。这条腿就是只带 Read、Bash 的定义，我的草稿目录里已有一份 `.writer-claim-<锁里的 nonce>`，建在我第一次用 Bash 往草稿目录写的那一刻（量的：`ls -la` 列出它，文件名里的 nonce 与 `.writer-lock` 里的 `nonce` 相同）。

### 三方腿收窄之后的模式

表里是「three-way-attack	research/prompts/*-opus-output.md」（`.claude/hooks/agent-write-scope.tsv:61`）、「three-way-defense	research/prompts/*-sonnet-output.md」（`.claude/hooks/agent-write-scope.tsv:65`）、「three-way-forward	research/prompts/*-sonnet-output.md」（`.claude/hooks/agent-write-scope.tsv:69`），模型目录同形。`*` 不跨 `/`、能配任何轮名：

| 格 | 定义 | 写到哪 | 推的结局 | 算什么 |
|---|---|---|---|---|
| `scope-attack-own-report-control` | three-way-attack | 这一轮自己的报告 | 放行 | 对照 |
| `scope-attack-earlier-round-report`、`scope-attack-earlier-round-model` | three-way-attack | 第一轮攻方的报告、第二轮攻方的模型目录（上一轮的冻结证据） | 放行 | 放过（第二轮报过，收窄只收了腿、没收轮） |
| `scope-defense-writes-forward-report-of-another-round` | three-way-defense | 第一轮正推腿的报告 | 放行 | 放过：正推与辩方都是 sonnet，后缀相同 |
| `scope-forward-writes-defense-report-of-another-round` | three-way-forward | 第二轮辩方腿的报告 | 放行 | 放过 |
| `scope-attack-own-second-sample-report`、`scope-attack-own-second-sample-model` | three-way-attack | 同一份提示再派的第二条腿自己的报告 `<轮>-opus-s2-output.md` 与模型目录 | 拒 | 误拒 |

第二次抽样的命名是规则定的：「报告写成 `<轮>-<模型>-s2-output.md`、模型目录写成 `<轮>-<模型>-s2-model/`」（`.claude/rules/three-way-inference.md:72`）。`*-opus-output.md` 要求名字以 `-opus-output.md` 收尾，`<轮>-opus-s2-output.md` 以 `-s2-output.md` 收尾，配不上；模型目录同理。`research/prompts/` 里今天有 27 份名字带 `-s2-` 的文件（量的：`ls research/prompts | grep -c -- '-s2-'`），这种腿真会再派。派发闸 ④ 读同一张表判报告行，报告行写 `-s2-output.md` 的派发提示在派发那一刻就被拒（推的，读码：④ 照写范围表判报告行点名的路径，`.claude/hooks/runner-dispatch-guard.sh:21` 那一段；没喂派发闸，它会在点名的草稿目录里建锁）。

## 第 9 问：`lib_code_discipline_lints.py` 两边共用之后 —— 成立

共用模块自己写着「两边读到的表逐条相同，也与整轮门禁真传给 clippy 的那张数组相同」（`.claude/hooks/lib_code_discipline_lints.py:2`）。前半句因为是同一份函数，恒真；后半句只在表照今天的写法写时真。门禁真传给 clippy 的是 bash 展开 `"${CODE_DISCIPLINE_LINTS[@]}"` 的结果，模块逐行用 `shlex` 切、只认第一段 `CODE_DISCIPLINE_LINTS=(` 到 `)`。`lints_model.py` 把下面几种 bash 认的写法交给 bash 展开当「门禁真传的」，与模块读到的逐词比（行 `LINTS name=<写法>`；本机用 bash 与 `python3 -I` 的 shlex 量过其中三种的切法，见表下）：

| 写法 | 门禁真传的 | 两边读到的 | ⑤（项目代码闸）怎么判 | ㉓ 出路 | 推的结局 |
|---|---|---|---|---|---|
| 今天的写法（对照） | 7 条 `-D` | 同 | 要求 7 条加 `warnings` | 逐词相同 | 全对 |
| 后面再 `CODE_DISCIPLINE_LINTS+=( -D clippy::needless_return )` | 8 条 | 7 条 | 不要求第 8 条 | 少第 8 条 | fail_open=clippy::needless_return |
| 后面整个重新赋值 | 新的那几条 | 旧的 7 条 | 要旧的、不要新的 | 是旧的 | misreject 旧 7 条、fail_open 新 2 条 |
| 某一行末尾 `\` 续行 | 7 条 | 读不到（shlex 遇行末反斜杠报 `No escaped character`） | 整道不判（「if lints is None:」（`.claude/hooks/project-code-guard.sh:762`）） | 空，记一条判不了 | guard_judges=no、fail_open=全部 7 条 |
| `-Dclippy::cast_{possible_truncation,sign_loss}` | 展开成两条 | 一条名字带花括号的 | 要求那个带花括号的名字，照门禁写全的 clippy 被拒 | 带花括号的字面、clippy 不认 | misreject、fail_open 两条 |
| 表里一个元素是 `"${BASE_LINTS[@]}"` | 另一张表的几条 | 字面 `${BASE_LINTS[@]}` | 不要求那几条 | 带一个字面词 | fail_open=那几条 |
| `-F clippy::todo`、`--forbid clippy::dbg_macro` | 两条当错误判的 | 读到了词，`table_names` 只认 `-D` / `--deny` | 不要求它们 | 逐词相同 | fail_open=两条 |
| `LIB_CODE_DISCIPLINE_LINTS_BREAK=first-word-per-line` 设着、表照今天 | 7 条 | 7 个 `-D` | 要求名字 `-D`，照门禁写全的 clippy 被拒 | `-D warnings -D -D -D -D -D -D -D` | misreject=-D |

量的（本机，草稿目录里只跑 bash 与 `python3 -I` 的 shlex，不碰被判文件）：`shlex.split('  -D clippy::a \\', comments=True)` 抛 `ValueError No escaped character`；`'  -Dclippy::cast_{possible_truncation,sign_loss}'` 切成一个词；同样的数组交给 bash，`printf "[%s]\n" "${LINTS[@]}"` 打出 `[-D]` `[clippy::a]` `[-D]` `[clippy::b]` `[-Dclippy::cast_possible_truncation]` `[-Dclippy::cast_sign_loss]` `[-D]` `[clippy::appended]` 八个词（续行、花括号、`+=` 都照 bash 的规矩展开）。

- 两边一起读错、门禁真传的与两边读到的不同：成立。方向分三种：少要求（`+=`、重新赋值的新条、数组元素、`-F`），照抄出路或少写这几条的 clippy 过得了两道钩子，到合入后验证才红；读不到（续行），⑤ 整道不判、闸那边不记检出（第二轮报过闸那边读不到不记，这一处没改）；多要求（花括号、重新赋值的旧条），误拒。都要有人改一回 check.sh 的写法才出现；`check.sh` 在 `.claude/singlefs-ai-sop/` 下（上游的包），本仓不改它，上游改写法时两道钩子不会跟着红。
- 命令那一侧（同一份 `denied_names`）：`-D clippy::cast_sign_loss` 之后再写 `-A clippy::cast_sign_loss`、或者 `--cap-lints allow`，rustc 照后写的、或照封顶的算，这一条 lint 实际关了；⑤ 只收集 `-D` / `--deny` 点名的名字，照放（`lints_model.py` 的 `command-allow-after-deny`、`command-cap-lints-allow` 两行推的 guard_passes=yes）。这是有意写的，不是表的写法，附在这里。

### `LIB_CODE_DISCIPLINE_LINTS_BREAK` 平常设了

模块读开关不看在不在自证里：「return item in os.environ.get("LIB_CODE_DISCIPLINE_LINTS_BREAK", "").split(",")」（`.claude/hooks/lib_code_discipline_lints.py:29`），文件头说它「只给导入它的钩子的自证判红用」（`.claude/hooks/lib_code_discipline_lints.py:14`），把「平常不认」交给导入它的钩子。两道钩子都没接：

- 项目代码闸在读 hook 输入之前拒「它导入的」库的开关，名单是「SELFTEST_ONLY_VARIABLES = (GUARD_BREAK_VARIABLE, "LIB_PROJECT_CODE_RUNS_BREAK", "LIB_SHELL_WORDS_BREAK", "WORKSPACE_CARGO_LOCK_BREAK")」（`.claude/hooks/project-code-guard.sh:3531`），上面一行的理由是「（本钩子的，加上它导入的 lib_project_code_runs.py、lib_shell_words.py、lib_workspace_cargo_lock.py 的：它们的开关一样能放宽这道闸）」（`.claude/hooks/project-code-guard.sh:3528`）。这一轮它新导入了这份模块（「import lib_code_discipline_lints as discipline_lints」（`.claude/hooks/project-code-guard.sh:681`）），名单没跟上。
- 检出钩子平常只拒自己的 `BASH_COMMAND_DETECTOR_BREAK`（「if not value or (len(sys.argv) > 2 and sys.argv[2] == "--selftest"):」那一段，`.claude/hooks/bash-command-detector.sh:1739`），共用库的开关都不看。

所以设在 Claude Code 进程环境里（钩子的环境就是它；Bash 命令里的 `export` 改不到钩子），两道钩子都照认、不点名它是弄坏开关：⑤ 要求 clippy 的 `--` 之后有名字 `-D`，照门禁写全的 `task-server/task submit -- cargo clippy --all-targets -- -D warnings <表>` 每一条都被拒；㉓ 的出路给 `-D warnings -D -D -D -D -D -D -D`，照抄交上去 clippy 自己报错。方向是拒（不放宽），没有放过的形状；害处是查不出原因：拒绝说明写「缺 -D」，不说是哪个变量。格 `lints-switch-set-project-code-guard-full-clippy` 推的 `guard_exit=2`、`refused_at_entry=no`、`missing=-D`，`lints-switch-set-detector-outlet` 推的 `outlet_equals_gate=no`；对照的两格 `lints-switch-unset-control-*` 推的放行、出路与门禁逐词相同。

## 第 10 问：两份新脚本的串行自证 —— 单跑：推不出（算打中）；开关轮次序判绿：不成立

规则的判据是「**判据：点名单跑一条用例，与它在不指定范围那一趟里跑，判定相同。** 自证分了格、能点名单跑的，每一格同样守这一条。」（`.claude/rules/test-script-discipline.md:8`）。

### 单跑一格：没有入口，判定同不同无从比

- 两份脚本的 `--selftest-inner` 都整份跑：`large-file-git-hook.py` 是「if arguments[0] == "--selftest-inner":」（`scripts/gate/large-file-git-hook.py:655`）之后直接 `return run_selftest()`，`git-hooks-install.py` 同形（`scripts/gate/git-hooks-install.py:356`）；后面多给的词不看。格 `selftest-inner-with-cell-name-*` 点一个格名起，推的 `cells_run=28`、`13`（整份）。规则要的「能点名单跑」这一前提就没有，所以「单跑与整份跑判定是否相同」判不出：推不出，照派发提示算打中。
- 两份文件头的 `selftest-serial` 一行说的是「不能并行」，没说「不能单跑」：「# selftest-serial: 各格按次序在同一个临时仓里装、改配置、改执行位、加链接工作树，后一格读前一格留下的仓状态，次序本身是自证的一部分」（`scripts/gate/git-hooks-install.py:4`）。「次序本身是自证的一部分」与「每一条都能单独跑」正面相反；`.claude/gate.d/selftest-parallelism-exempt.tsv` 那一格只管并行，管不到这一条。
- 读前一格留下的状态的格（读码数的）：
  - `large-file-git-hook.py`：`commit-judge-reads-index-copy` 用 `commit-staged-big-refused` 那一格被拒之后还暂存着 big.bin 的仓（`scripts/gate/large-file-git-hook.py:415`）；`commit-all-grown-refused`、`commit-paths-grown-refused` 接着 `commit-unstaged-growth-passes` 的仓（第 427 行那一格要前一格的 `-a` 被拒、grown.txt 还没进提交）；`private-hook-then-judge` 接着 `private-hook-refusal-first`；`push-clean-branch-passes`、`push-delete-passes` 接着 `push-unpushed-big-refused` 的仓与远端；`push-all-refused` 到 `push-small-branch-passes` 六格共用 `push-branches` 一个仓与远端（「done = box.git(repository, "push", "-q", "--all", "origin", check=False)」（`scripts/gate/large-file-git-hook.py:519`），要前面 `push-other-branch-refused` 没把 feature 推上去）；`push-to-url-refused` 接着 `push-never-fetched-refused`；`judge-timeout-leaves-nothing` 数的是整份自证共用的 TMPDIR 里的残留（「leftovers = [entry for entry in os.listdir(box.temporary) if entry.startswith("large-file-git-hook-index-")]」（`scripts/gate/large-file-git-hook.py:581`）），前面任何一格漏下的拷贝都算到它头上。
  - `git-hooks-install.py`：`not-installed-red` 到 `linked-worktree-green` 九格与三格 `peer-*` 都在一个 `plain` 仓上依次改配置与执行位（例「box.git(repository, "config", "core.hooksPath", ".git/hooks")」（`scripts/gate/git-hooks-install.py:251`））。
- 判定会不会因单跑而变：按读码逐格推，每一格的「准备」都在它前面那几行，挪进它自己的格里判定照旧相同（推的）；唯一一处会变的是 `judge-timeout-leaves-nothing`：它判「不留索引拷贝」时把前面各格的残留一起算，别的格的漏在整份跑时红在这一格，单跑这一格时不红。今天前面各格不漏（自证绿），所以今天相同。

### 开关轮会不会因为次序把一格判绿

逐个开关读了一遍：开关轮对每个开关新起一个 `--selftest-inner` 进程、新建一个临时目录（`switch_wheel` 那一段），一个开关的残留带不到下一个；每个开关点名的格都是那一处被弄坏的直接后果，没有哪一格是靠前一格先坏、后一格跟着坏才红的（例 `push-reads-current-branch-only` 点名的八格，每一格自己的推送都只判 HEAD；`remote-sha-ignored` 点名的两格，各自的仓里跟踪引用已删、只剩行里的远端 sha）。也没找到一格在开关打开时该红、却因为前面某一格先改了仓而判绿的：读得到的同仓格里，前一格在开关下的结局只会让后一格更红（例 `push-other-branch-refused` 在 `push-reads-current-branch-only` 下把 feature 推上去之后，`push-all-refused` 本来就在点名里）。不成立；读码的，没造开关逐格跑（背景材料「验过的」第二、三条的复跑结局是主 agent 的数）。

附带（不归这一问的字面）：`judge-timeout-leaves-nothing` 认残留进程只认命令行带「<仓> --guard」的那一个（判法自己），判法起的慢 git 与 `sleep` 的命令行里没有这一串；若 `stop_group` 改坏成只停直接起的那一个，那一格看不出来，开关表里也没有这一条开关（推的）。

## 打中之后的四句

这一轮没有臂，「分不分辨」按「今天的实现中、对应的改法不中」读；「跑前条款」读作正文第 5 到 10 问的字面。

| 打中的一组 | 分不分辨 | 被判的那一道当时看不看得到判别它的东西 | 满足的是哪一个分句 | 改法在打中的格上还中不中（推的） |
|---|---|---|---|---|
| 改了仓位置（`--work-tree`、`--git-dir`、`GIT_DIR`） | 分辨 | ㉒ 看得到：选项与变量都在命令里，它现在是看到了故意不判 | 第 5 问「能让超限文件进提交或推出去而不被任何一道拦下」 | core.hooksPath 改绝对路径：三格都不中；㉒ 把这几种算「绕开」：三格都拒 |
| 不带钩子目录的树（链接工作树、sparse） | 分辨 | git 钩子看不到（它根本不起）；㉒ 看得到：git 命令所在的树与 core.hooksPath 解出来的目录都在本机 | 同上 | 绝对路径：链接工作树那格不中；sparse 在主工作树里那格仍中（绝对路径指进同一棵被剔掉的树）；㉒ 改认「core.hooksPath 解出来的目录里没有钩子」：两格都拒 |
| 标签指着 blob / tree | 分辨 | 看得到：标准输入里就是那个对象名 | 第 5 问；第 7 问「各判成什么」 | `--ref` 认 `^{object}`：不中 |
| `--selftest` 词的祖先 | 分辨 | 看得到：钩子知道自己是被 git 起的 | 第 5 问 | 改认自证标记（标记文件加环境变量，照检出钩子）：不中 |
| 假 python3、改配置节 | 分辨 | ㉒ 看得到：PATH、GIT_EXEC_PATH 与 git config 的子命令都在命令里 | 第 5 问（有意绕开） | ㉒ 认这几种：不中；钩子起判法用绝对路径的 python：假 python3 那格不中 |
| 路径远端、浅克隆的误拒 | 分辨 | 看得到：行里的远端 sha 读不到 | 第 6 问「误拒」、第 7 问 | 读不到的远端 sha 写 ⊘ 放行而不是判全部历史：不中；但这一改让「真没取回过」的远端也放行，要另想（见改法表） |
| 并发 as-is 提交 | 分辨 | 钩子看得到：判之前、判之后索引文件的内容可以各量一次 | 第 7 问「并发提交时会不会…写错」：拷贝没写错，判的与进的不是同一份 | 判完再核一次索引没变、变了就拒：窗口缩到「钩子退出到 git 重读」那一段，不到零 |
| 信号下留拷贝、判法成孤儿 | 分辨 | 看得到 | 第 7 问「会不会留下」 | 钩子接 TERM / HUP 转成退出、`finally` 里停判法那一组再删：不中 |
| ⑫ 放过与误拒 | 分辨 | 看得到：落点、`.owner`、锁、定义的工具都在 | 第 8 问 | 见改法表；`.owner` 那一类要改判据（只认锁的话判别子不在） |
| 三方 `-s2-` 误拒、别的轮放过 | 分辨 | 看得到：派发提示点名了报告路径 | 第 8 问 | 表加 `-s2-` 两行：误拒那格不中；别的轮那格要按派发点名的路径判（第二轮提过） |
| 表的写法、命令的 `-A` | 分辨 | 看得到：check.sh 全文与命令全文都在 | 第 9 问「两边一起读错（门禁真传给 clippy 的与两边读到的不同）」 | 见改法表；不交给 bash 展开就总有一种写法读错 |
| 开关平常设了 | 分辨 | 看得到：环境变量就在钩子进程里 | 第 9 问后半 | 名单加这一个：不中 |
| 单跑一格 | 判别子观测不到：没有单跑的入口，「判定相同」这个判据在这两份脚本上无从判 | — | 第 10 问前半 | 加点名单跑的入口、每格自己建仓：之后才判得了 |

## 改法与各修哪一格

全部是我提的，只在我的模型上量过、被攻过零轮——而且一格都没量：改法都没实现，副本上也没有数，下表每格都是「推的」（主 agent 的复跑只量今天的实现，A/B 要另做）。

| 改法 | 改哪里 | 修的格 | 对哪几格不起作用 | 证据 |
|---|---|---|---|---|
| core.hooksPath 装成主工作树钩子目录的绝对路径（两台挂在同一绝对路径，`accepted_directories` 已认绝对写法） | `git-hooks-install.py` 的 `run_install` | `work-tree-option-commit`、`git-dir-*` 两格、`old-commit-linked-worktree-commit-and-push`；链接工作树里私有窗口闸也回来 | 主工作树自己被 `sparse-checkout` 剔掉 `scripts/` 的那一格；链接工作树里跑的是主工作树那一版钩子（不再跟着各自的树走） | 推的 |
| ㉒ 第一件（绕开）加：git commit / git push 带 `--git-dir`、`--work-tree`、`GIT_DIR`、`GIT_WORK_TREE`、`--exec-path`、`GIT_EXEC_PATH`、`PATH=` 前缀、`LARGE_FILE_GIT_HOOK_*`；git config 的 `--remove-section`、`--rename-section`、`--edit` 碰到 core 节 | 检出钩子 `git_hook_bypass_reasons`、`git_config_writes_hooks_path` | 第 5 问改仓位置三格、假 python3、改配置节两格、`--selftest` 两格（看变量名） | 不经 Bash 的（用户手敲、脚本里起的） | 推的 |
| ㉒ 第二件（没装上）不再用「树里有没有钩子目录」认是不是本项目：用 core.hooksPath 解出来的目录（在本仓的共用 .git/config 里设了它，就要它解开之后有钩子） | `lib_git_hooks.py` 的 `project_checkout` 与 `hooks_path_problems` | 链接工作树、sparse 两格（拒，出路写「在主工作树提交」或「装绝对路径」） | — | 推的 |
| `--guard push` 的 `--ref` 解成 `^{object}`，`rev-list --objects` 从标签、tree、blob 起 | `large-file-check.py` 的 `existing_commits`（`--ref` 那一边另写一个） | `tag-to-blob-push`、`tag-to-tree-push` | — | 推的 |
| 两份新脚本认自证改成标记文件加环境变量（照检出钩子的 `SELFTEST_MARK`），不看祖先的命令行 | `inside_selftest` 两处 | `selftest-word-*` 两格 | — | 推的 |
| 本地读不到的远端 sha：写 ⊘、只判这些本地 sha 相对「本地有的远端 sha 与跟踪引用」的部分；全都读不到才判全部历史 | `guard_pushed` | `push-path-remote-advanced-elsewhere-force` | 浅克隆推向路径那一格（远端 sha 全 0，没得读）；另要给「远端 sha 全 0 的路径远端」一条 ⊘ 出路（`git fetch <路径>` 再推） | 推的 |
| 钩子接 TERM、HUP（转成 SystemExit），`finally` 里先停判法那一组再删目录 | `large-file-git-hook.py` 的 `run_judge` | 信号三格 | KILL | 推的 |
| 钩子判之前、判之后各算一次 `GIT_INDEX_FILE` 指的文件的 sha256，变了就拒（「判的时候索引被改了，重来」） | `judge_index` | `race-add-during-as-is-commit` | 钩子退出到 git 重读之间那一小段 | 推的 |
| ⑫：unclaimed 记一条检出（同写闸）；带 `.owner` 而会话不是这一个的目录、工作根下本仓的链接工作树按「别人的」拒；带 Write、Edit 的定义在 Bash 里写也照锁判 | 检出钩子 `writer_lock_refusal`、`bash_scope_refusal` | 第 8 问放过的四格 | `lock-copied-lock-inside-own-draft`（要改判据：拷进来的锁认不出是拷的） | 推的 |
| 写范围表三方腿各加 `research/prompts/*-<模型>-s2-output.md`、`*-<模型>-s2-model/**`、`*-<模型>-s2-model` | `agent-write-scope.tsv` | `scope-attack-own-second-sample-*` 两格与派发闸 ④ 的同一形 | 别的轮那几格 | 推的 |
| 表读法：认不出的写法（行末 `\`、`$`、花括号、第二处 `CODE_DISCIPLINE_LINTS=` 或 `+=`）一律当「读不到」并在 ⑤ 那边也记检出；`-F` / `--forbid` 也算要求；`denied_names` 看后写的 `-A` / `-W` / `--cap-lints` | `lib_code_discipline_lints.py`、`project-code-guard.sh` | 第 9 问七行与命令那一侧两行 | 读不到时 ⑤ 照旧不判（只是不再静默） | 推的 |
| `SELFTEST_ONLY_VARIABLES` 加 `LIB_CODE_DISCIPLINE_LINTS_BREAK`；检出钩子平常也拒共用库的开关 | 两道钩子 | `lints-switch-set-*` 两格 | — | 推的 |
| 两份自证各加「点名单跑一格」的入口，每格自己建仓、自己的 TMPDIR | 两份新脚本 | 第 10 问前半 | — | 推的 |

## 要主 agent 跑的命令

探针与模型不起 cargo、不点验证包，没写范围声明；范围声明闸若把它们认成实验复跑而拒，照拒绝说明补一句（种类 items，跑什么写「攻方腿探针」）。

1. 先跑一小段估全量挂钟（只喂钩子、不建大文件的两组）：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-opus -- python3 /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r3-opus-model/probe.py --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-opus-estimate-run --only scope,lint-switch`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-opus`（草稿目录）。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 `PROBE name=frozen-hash` 十五行都是 `match=yes`，`group-seconds-scope`、`group-seconds-lint-switch` 两行的秒数，没有 `group-error-` 行，末行 `PROBE name=done emitted=<N>`。估全量：这两组共 19 格、约 19 次喂钩子；全量另有约 34 次喂钩子、约 30 次 `git add` 超限文件、两份自证整份各一次、五格各睡 6 到 8 秒；照第一条的秒数按比例乘，超过 40 分钟就别交第 2 条，交回我缩格。跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-opus-estimate-run`。
2. 整份复跑：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-opus-run /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r3-opus-model`
   - 在哪起：仓根（第二个参数是冻结副本，有开工快照树就换成它）。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`（退 1 是 `lints_model.py` 某一种写法装置出错（`error-` 行）或探针某一组装置出错（`group-error-<组>` 行））；`task-server/task log <号> --all` 里 `LINTS`、`PROBE` 两种行逐格对本报告「各格判定一览」与各节的推的结局；两份末行 `… name=done emitted=<N>` 的 N 等于各自的行数减一。全文另在运行目录的 `lints.out`、`probe.out`，各格钩子与命令的 stderr 在 `probe/stderr/`，转发桩记的钩子调用在 `probe/hook-calls.log`。推的结局与读数对不上的格，以读数为准，我那一格的判断照读数改。

内存与时间见开头「复跑命令与模型文件」最后两条（推的：5 到 10 分钟、几百 MB）。

## 没打中的形状

- 第 5 问：`git merge --no-verify`、`git am`、`commit-tree` 加 `update-ref`、`git stash` 都推不出去（pre-push 拦）；`GIT_CONFIG_GLOBAL` / `GIT_CONFIG_SYSTEM` 改不动仓里的 core.hooksPath；子模块碰不到本仓的远端。
- 第 6 问：只删分支、只推指着已推提交的标签、推向配置过的名字而远端已有超限历史，都放行；判全部历史在本仓本机两秒上下（git 那两步量的）。
- 第 7 问：`--known` 读不到时配置过的远端名有跟踪引用兜着，不漏；远端 sha 全 0 照跟踪引用判；索引拷贝在超时那一支删得干净（自证有格）；拷贝本身不会拷成半份。
- 第 8 问：只带 Read、Bash 的定义写自己的草稿目录放行并认领；别人认领过的、回执对不上的拒。
- 第 10 问：开关轮没找到靠次序判绿的格。
- 取样范围：第 5 问 19 格，第 6 问 5 格加 6 种 ㉒ 写法，第 7 问 5 格加并发 2 格、信号 3 格，第 8 问 15 格，第 9 问 8 种表的写法、2 种命令写法、4 格喂钩子，第 10 问 2 格加读码；超限文件只取「上限加一字节」一个尺寸。

## 这条腿自己的限度

- 探针与模型一格都没跑，「推的」结局没有一个是量出来的。git 与 Python 的几处行为是我的印象，格就是核它们：相对 core.hooksPath 跟着「钩子运行的目录」走（`--work-tree` 时 commit 先换目录、`--git-dir` 不给工作树时把当前目录当顶层）、as-is 提交在钩子跑着时放开索引锁并在之后重读索引、`git am` 不起 pre-commit、`sparse-checkout set` 会把 `scripts/` 从工作区拿掉、`Popen.__exit__` 遇 Ctrl-C 只等约 0.25 秒。
- 慢判法那五格（并发两格、信号三格）借了钩子自证用的 `LARGE_FILE_GIT_HOOK_JUDGE_PATH`，要命令行里有 `--selftest` 词的祖先才认；这一招本身是第 5 问的一格。平常判法快，窗口与「Bash 到点时判法还在跑」都要慢盘或大仓才碰得上，我没量第二台。
- 第 8 问的锁、认领与派发回执是按 `lib_writer_lock.py` 文件头的格式造的假文件，不是派发闸真建的；派发闸 ④ 对 `-s2-` 报告行的误拒是读码推的，没喂派发闸（喂它会在点名的草稿目录里建锁）。
- 第 9 问的「门禁真传的」是 bash 展开表的那一段，不是真起 `check.sh`；表的写法都是我造的，上游今天没有这么写。
- 被判文件之外读了：`.claude/hooks/lib_writer_lock.py` 前 360 行、`lib_work_root.py` 的几个函数、`project-code-guard.sh` 第 680 到 790 行与第 3525 到 3560 行、检出钩子第 262 到 297 行、第 1734 到 1757 行、第 2408 到 2420 行、第 2489 到 2560 行、第 2620 到 2660 行、第 2947 到 2990 行、第 4396 到 4475 行、`.claude/singlefs-ai-sop/scripts/check.sh` 第 64 到 104 行、`scripts/agents/cite-check.py` 文件头、`scripts/kb/replace-once.py` 文件头、第二轮攻方报告与第一轮攻方报告的节标题、第二轮攻方探针副本 `probe.py` 的装置部分、`.git/hooks/pre-commit`、`git help config` 与 `man githooks` 里 core.hooksPath 与钩子工作目录两段。正推腿那一份在禁读清单里，没读。
- 只读量的几样（`git worktree list`、`git status --short`、`git rev-list`、`git cat-file --batch-check`、`git count-objects`、`stat`、工作根的 `.owner` / `.writer-lock` 计数、检出记录的 `grep -c`）都在本机、本仓根上跑；第二台没量。

## 没做什么

- 不判正推那几格（第 1 到 4 问）；不替主 agent 采纳；改法都没实现，副本上也没有数。
- 没往统一任务服务交任务，判决要引的数一个都没复跑：命令在「要主 agent 跑的命令」一节。
- 没建仓副本、没建编译目录。草稿目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-opus` 里除认领文件与锁之外，只有量全部历史时写的一份对象清单 `objects.txt`（交回前删了，2.9M）与 cite-check 的输出 `cite-check.txt`，没有交接的副本。
