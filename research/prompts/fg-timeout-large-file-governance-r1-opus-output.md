# 攻方腿报告：fg-timeout-large-file-governance-r1（正文「要腿判的」第 5 到 11 问）

日期：2026-11-18。被判的三份读的是主工作区换上之后的那一版，开工时现算 sha256 与 `research/prompts/fg-timeout-large-file-governance-r1-snapshot/judged-sha256.txt` 四行逐字相同（检出钩子 b7627f14…、large-file-check.py 7889c968…、agent-common.md c93670a7…、正文 57b7328e…）。

判词的意思：「成立」= 这一问问的那种错在今天的实现上找得到（打中）；「不成立」= 找不到；「推不出」= 靠仓里读得到的东西判不出（派发提示定为打中）。
这一轮子 agent 不交任务，探针一条都没跑：下面每一格的「钩子会判成什么」是读码推的，标「推的，待主 agent 跑」；能用只读命令核的（git 配置、耗时、worktree 里有没有规范副本）是我现跑的，标「量的」。

## 复跑命令与模型文件

模型目录 `research/prompts/fg-timeout-large-file-governance-r1-opus-model/`：

| 文件 | sha256 |
|---|---|
| `probe.py` | `6f79e83bbd8065cdc59f9fbef3da6626bbe9563af7f0f0adc753072d23187da3` |
| `fake_large_file_check.py` | `b78c53298b49bc054a29ff992b2f35bf934046db3b7979143e92935850f47ff7` |
| `tools/rerun.sh` | `d863e8585d0cb598c79edd7e5d1859fd4c9e3b1dc9cde513497ac34281061433` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

复跑（整条在「要主 agent 跑的命令」一节，下面是同一条）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-opus-run /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r1-opus-model
```

- `tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：起 `probe.py --repo <仓根> --frozen <被判那棵树> --work <运行目录>/probe`，输出另存 `<运行目录>/probe.out`。外层包装照范例只删工作根下以 `-run` 收尾的那一个运行目录。
- 第二个参数（冻结副本）这里填的是仓根：主 agent 有开工快照树就换成它。探针开头逐份核 `judged-sha256.txt` 那四行，打 `PROBE name=frozen-hash … match=yes|no`，别的会话中途改了钩子时这一行会写 `match=no`。
- 探针做什么：在运行目录下建临时 git 仓、本地裸远端，大文件是上限加一字节（104857601 字节）的稀疏文件，只有「不可压缩」那一格写随机字节；每一格把一条 hook 输入 JSON 喂给被判的检出钩子（命令不执行），记退出码、stderr 里有没有 ㉒ / ㉑ 的拒绝句、检出记录（`AGENT_HOOK_DETECTIONS` 指到运行目录里的一份）里有没有 ㉒ 的「没判成」；再真执行那条命令一次，看超限的 blob 有没有进新提交、进远端。每格一行 `PROBE name=<格名> … verdict=<结局>`，结局是 `missed-silently`（进了、没拒、没记）、`passed-recorded`（进了、没拒、记了检出）、`rejected`、`rejected-nothing-lands`（拒了而什么都不会进：误拒）、`passed-nothing-lands`。末行 `PROBE name=done emitted=<N>`。
- 样本进程：喂钩子与执行命令各用 `start_new_session=True` 起，超时与收尾只 `os.killpg` 自己起的那一组，跑完按组号扫 `/proc` 核没有留下进程（`hook_leftover=` 一列）。git 配置隔离成运行目录里的一份（`GIT_CONFIG_GLOBAL`、`GIT_CONFIG_NOSYSTEM=1`），只有「主仓挂钟」那一组用本机配置读仓根（命令不执行，`GIT_OPTIONAL_LOCKS=0`）。
- 内存与时间（推的，没跑过）：单个进程不过几十 MB；大头是钩子起的次数（约 160 次，每次一两秒）、假判法「慢」那一格（4 次 × 25 秒）、不可压缩那一格的随机 100 MiB 写入与 gzip（十来秒）。估全量 10 到 15 分钟，远低于 40 分钟。盘：运行目录峰值约 450 MiB（不可压缩那一格的原文件、对象、.gz 与 .gz 的对象各约 100 MiB），别的都是稀疏文件与压过的零。没有崩溃状态，第 3b 步的崩溃枚举与状态数不适用。

## 各格判定一览

| 问 | 格 | 判 | 依据（证据状态） |
|---|---|---|---|
| 5 | 数、小数、负数、`3e5` 写成 JSON 数、超大整数、Infinity、NaN | 钩子侧：600000 拒，240000.5 拒，-1 放，JSON 数 3e5 拒，400 位整数抛异常放行加记检出，Infinity / NaN 拒 | 读码；推的，待主 agent 跑（`timeout-*` 那几行） |
| 5 | 字符串 `"3e5"`、`"300000ms"`、`"300000."`、`"+300000"`、`"0x493e0"` | 钩子放行；harness 收不收、收了按几毫秒跑，推不出 | 推不出（算打中）；探针只能量钩子那一侧 |
| 5 | `run_in_background` 写成字符串 `"true"` | 钩子各处都按前台算（`is True`）；harness 算前台还是后台推不出 | 推不出（算打中） |
| 5 | 主 agent 照 ㉑ 改成 run_in_background 之后撞别的道 | 不成立：撞的只会是 ②④ 自己管的写法，不是这一道带出来的 | 读码 |
| 6 | 别名（仓里配的、`-c alias.*`、`!` 起头的）、`xargs`、`find -exec` | 成立：放行、不记 | 读码；推的，待主 agent 跑 |
| 6 | 长选项缩写 `--mes`、`--inc`、`git add --forc` | 成立：放行、不记 | 读码；推的，待主 agent 跑 |
| 6 | 同一条命令里先改状态再 commit：`stash pop` / `stash apply --index`、`cherry-pick -n`、`checkout <分支> -- 路径`、`merge --squash`、`reset --soft` 重做、先造文件再 add、先长文件再 `-a` | 成立：放行、不记 | 读码；推的，待主 agent 跑 |
| 6 | 同一条命令里新建的仓（`mkdir` / `git init` / `cd` 再 commit） | 成立：目录不在就静默跳过，不记检出 | `.claude/hooks/bash-command-detector.sh:3302`；推的，待主 agent 跑 |
| 6 | `git push origin <别的分支>`、`--all`、`--mirror`、`--tags`、`--recurse-submodules=on-demand` | 成立：放行、不记 | 读码；推的，待主 agent 跑 |
| 6 | 新分支头一次推（`git push -u origin feature`，当前分支没上游） | 成立（弱）：放行，只记检出 | 读码；推的，待主 agent 跑 |
| 6 | 从子目录起、`GIT_INDEX_FILE` 写相对路径 | 推不出：钩子照命令的目录解，git 照哪一处解没核；两种摆法探针都摆了 | 推不出（算打中），待主 agent 跑 |
| 6 | `--amend`（暂存了大文件）、`-F`、`git -c … commit`、`command git commit`、`git add -u &&`、`push origin HEAD:other`、`sh -c`、`eval`、喂给 bash 的 heredoc、`pushd`、`export GIT_INDEX_FILE` | 不成立：拒（推的） | 读码；推的，待主 agent 跑 |
| 7 | 没暂存的改大文件、被忽略的大文件、软链接指到仓外大文件 | 不成立：放行而且确实不进提交 | 读码；推的，待主 agent 跑 |
| 7 | 带空格与非 ASCII 的路径 | 不成立：照拒，出路里的路径用 `shlex.quote` 引好 | 读码；推的，待主 agent 跑 |
| 7 | 全局配置 `diff.relative=true` 时从子目录 `git commit <路径>` | 成立：路径对不上，改大的文件看不见，放行不记 | 读码；推的，待主 agent 跑 |
| 7 | `git push origin :stale`、`git push origin side`（当前分支没推的提交里有大文件） | 成立（误拒）：推的不是当前分支也照当前分支拒 | 读码；推的，待主 agent 跑 |
| 7 | 判法起几次、合计多久、settings 时限 | 成立：每条 add / -a / -C 各加一次、每次 25 秒，没有总时限；settings 里这一条没写 timeout | 读码 + `.claude/settings.json:83`；次数待假判法跑 |
| 7 | 判 commit-worktree 时 `git diff` 顺手写回索引 | 成立（推的）：只动了时间戳的跟踪文件会让它拿 index.lock 写回 | 读 git 行为（推的），待主 agent 跑 |
| 7 | 本机仓根上判法的 git 耗时 | 不成立：本机量的都在 0.4 秒内 | 量的（只读 git，见第 7 问一节） |
| 8 | 抛异常、多打一行、退 3、git 不在 PATH | 不成立：都放行加记检出（推的） | 读码；推的，待主 agent 跑 |
| 8 | 判法打出缺键的 JSON | 成立（窄）：拒绝说明那一步在 try 外，钩子整个退 1，同一条命令里别的拒绝一并丢掉 | `.claude/hooks/bash-command-detector.sh:9498`；推的，待主 agent 跑 |
| 8 | 判法到点被停 | 成立（窄）：只停直接起的那一个，判法起的 git 留着 | `.claude/hooks/bash-command-detector.sh:3238`；推的，待主 agent 跑 |
| 9 | 出路三步一条一条照抄执行 | 不成立：各道闸放行、做完再 commit 放行（推的） | 读码；推的，待主 agent 跑 |
| 9 | 出路三步与 commit 串在同一条命令里 | 成立（误拒）：照改之前的暂存区判，照拒 | 读码；推的，待主 agent 跑 |
| 9 | 不可压缩的大文件 | 成立：.gz 仍超限，出路绕回自己 | 算术 + 读码；推的，待主 agent 跑 |
| 9 | push 那一支的出路 | 成立（弱）：只写「重做那几笔提交」，没给一条能照抄的重做命令 | 读码 |
| 10 | 九条开关两条弄坏同一处 | 不成立：九条各在一处判 | 读码（行号见第 10 问一节） |
| 10 | 点名的格名不副实 | 成立（弱）：「-m 的值是 -a」那一格挂在 commit-content-worktree-always 下，没有一条开关弄坏选项取值的跳过 | `.claude/hooks/bash-command-detector.sh:8644` |
| 10 | 真 git 仓的格与 selftest-process-guard | 本机不成立；第二台推不出：自证的 git 不隔全局配置 | `.claude/hooks/bash-command-detector.sh:8571`、本机全局配置量的 |
| 10 | 门限变量平常跑认不认 | 不成立（不认）；附带：标记对得上时把门限调大也认 | `.claude/hooks/bash-command-detector.sh:3227`；推的，待主 agent 跑 |
| 11 | 仓副本、git worktree、第二台各读哪一份 | 成立：会话起在新建的 git worktree 里时，那一份判法起不来，每次只记检出；第二台推不出 | 读码 + 量的（worktree 里有没有规范副本） |

## 第 5 问：㉑ 漏拒与误拒 —— 推不出

判法是这几行：`.claude/hooks/bash-command-detector.sh:3089`：「if tool_input.get("run_in_background") is True or detector_break_is_set("allow-long-foreground-timeout"):」，`.claude/hooks/bash-command-detector.sh:3092`：「if isinstance(value, str) and re.fullmatch(r"\s*\d+(?:\.\d+)?\s*", value):」，`.claude/hooks/bash-command-detector.sh:3096`：「return int(value) if float(value).is_integer() else value」。

钩子侧各写法判成什么（读码，推的，待主 agent 跑：`PROBE name=timeout-<写法>` 的 `hook_exit=`、`refused_21=`、`shown=`）：

| hook 输入里的 `tool_input.timeout` | 钩子 |
|---|---|
| `600000`、`240001` | 拒 |
| `240000.5` | 拒，说明写「240000.5 毫秒」 |
| `240000.0`、`240000`、`-1`、`true`、`"soon"` | 放 |
| JSON 数的指数写法 `3e5`（解出来是 300000.0） | 拒，说明写「300000 毫秒」 |
| `Infinity`、`NaN`（Python 的 json 收这两个词） | 拒，说明写「inf 毫秒」「nan 毫秒」 |
| 400 位的整数 | `float(value)` 抛 OverflowError，main 里接住记「没判成前台命令的 timeout」，放行 |
| `" 300000 "` | 拒 |
| `"3e5"`、`"300000ms"`、`"300000."`、`"+300000"`、`"0x493e0"` | 放 |

打中的是 harness 那一侧，仓里判不出：

- 字符串那几种：钩子只认「只含数字（可带小数）与前后空白」的字符串。harness 若照 JavaScript 的 `Number()` 把字符串转成数再起命令，`"3e5"`、`"300000."`、`"+300000"`、`"0x493e0"` 都是 300000，命令会前台挂 5 分钟而 ㉑ 放行；harness 若在钩子之前就拒掉不是数的 timeout，这几种根本到不了钩子。哪一种是真的，靠仓里的文件判不出。能分辨的观测：主 agent 自己发一次 Bash 调用，命令 `sleep 1`，`timeout` 写成字符串 `"3e5"`，看 harness 报输入校验错、还是钩子放行后命令照跑（这一步是工具调用，不是任务，子 agent 做不了）。
- `run_in_background: "true"`：钩子五处都写 `is True`（`.claude/hooks/bash-command-detector.sh:3089`、`.claude/hooks/bash-command-detector.sh:3482`：「return bool(agent_type) and run_in_background is True」，另有第 1466、3490、9286 行），字符串一律当前台。harness 若把字符串 "true" 当真起后台，主 agent 那一格是 ㉑ 误拒（说明叫它改成 run_in_background: true，它已经写了），子 agent 那一格是 ⑩ 漏拒（⑩ 不归这一轮判，这里只说两道用的是同一个认法、结局一起翻）。分辨观测同上：发一次 `run_in_background: "true"` 的调用，看是不是进后台。
- 主 agent 照拒绝说明改成 run_in_background 之后：说明只叫它去掉 timeout、改后台、结束本轮；会撞上的是 ④（单独的 `&` 后面没有 `wait`）与 ②（轮询本会话后台任务的输出文件）那两种写法自己的规矩，原命令若是前台合法的 `a & b; wait` 改后台照样合法。子 agent 的出路是写进报告交回，不碰 ⑩。这一小格不成立。

## 第 6 问：㉒ 漏拒 —— 成立

判据字面：超过上限的文件进了提交或推了出去，㉒ 放行、也不记检出。下面每一形都给了能照喂的 git 序列（探针里同名格），结局都是读码推的，待主 agent 跑（`PROBE name=<格名>` 的 `verdict=`）。

### 认不出这一条是 git commit / git push

`.claude/hooks/bash-command-detector.sh:3265`：「if not words or os.path.basename(words[0]) != "git":」，`.claude/hooks/bash-command-detector.sh:3268`：「if subcommand not in ("commit", "push", "add")」。切词模块自己写明它不剥 xargs：`.claude/hooks/lib_shell_words.py:87`：「（xargs 与名单外启动器由 lib_project_code_runs.py 的 command_under_extra_launcher 剥）；」。㉒ 没调那一个。

| 格 | 序列（仓里 big.bin 已 `git add`） | 推的结局 |
|---|---|---|
| `commit-repository-alias` | `git config alias.ci commit`，再喂 `git ci -qm x` | missed-silently |
| `commit-command-line-alias` | `git -c alias.ci=commit ci -qm x` | missed-silently |
| `commit-command-line-shell-alias` | `git -c 'alias.ci=!git commit' ci -qm x`（切词模块把 `!` 别名交在 embedded launches 里、要调用方再判，㉒ 不读） | missed-silently |
| `commit-through-xargs` | `echo x \| xargs git commit -qm` | missed-silently |
| `commit-through-find-exec` | `find . -maxdepth 0 -exec git commit -qm x ';'` | missed-silently |

### 参数认错

`git_commit_form` 只认长选项的全名：`.claude/hooks/bash-command-detector.sh:3150`：「elif name in GIT_COMMIT_OPTIONS_WITH_VALUE:」。git 的 parse-options 收唯一前缀的缩写（这一句是我对 git 的印象，没在本机核；探针那两格就是核它）。

| 格 | 序列 | 推的结局 |
|---|---|---|
| `commit-abbreviated-message-option` | big.bin 已暂存，`git commit -q --mes x`：`x` 被当成路径，判 commit-worktree `--pathspec x`，查不到东西 | missed-silently |
| `commit-abbreviated-include-option` | big.bin 已暂存、base.txt 改小，`git commit -q -m x --inc base.txt`：git 把暂存区一起提交，钩子只判 base.txt | missed-silently |
| `commit-abbreviated-force-add` | `.gitignore` 忽略 ignored.bin、ignored.bin 超限，`git add --forc ignored.bin && git commit -qm x`：`--force` 只认全名（`.claude/hooks/bash-command-detector.sh:3203`），判成不带 `--ignored-too` | missed-silently |

### 同一条命令里先改了状态，钩子判的是改之前

㉒ 只把排在前面的 `git add` 算进去，别的改暂存区、改工作区、建仓的步骤都不看；钩子在命令执行之前判，那一刻文件或暂存区还没变。

| 格 | 序列 | 推的结局 |
|---|---|---|
| `commit-after-stash-pop` | big.bin 暂存后 `git stash`，再喂 `git stash pop -q && git commit -qm x` | missed-silently（新加的文件 pop 回暂存区，这一点待跑核） |
| `commit-after-stash-apply-index` | 同上，`git stash apply -q --index && git commit -qm x` | missed-silently |
| `commit-after-cherry-pick-no-commit` | 分支 other 有一笔带 big.bin，main 上 `git cherry-pick -n other && git commit -qm x` | missed-silently（推的时候乙会拦） |
| `commit-after-checkout-path-from-branch` | 同上，`git checkout other -- big.bin && git commit -qm x` | missed-silently（同上） |
| `commit-after-merge-squash` | 同上，`git merge -q --squash other && git commit -qm x` | missed-silently（同上） |
| `commit-redo-after-reset-soft` | HEAD 那一笔加了 big.bin，`git reset -q --soft HEAD~1 && git commit -qm redo`：这正是 push 那一支出路说的「重做提交」 | missed-silently（推的时候乙会拦） |
| `commit-amend-keeps-head-big` | HEAD 那一笔加了 big.bin，`git commit -q --amend -m y`：新提交相对它的父提交带进 big.bin，钩子比的是暂存区对旧 HEAD | missed-silently（大文件本来就在没推的提交里，推的时候乙会拦） |
| `commit-file-created-in-same-command` | `truncate -s 104857601 gen.bin && git add gen.bin && git commit -qm x`：add 那一段判 `--untracked --pathspec gen.bin`，那一刻文件还不在 | missed-silently |
| `commit-all-file-grown-in-same-command` | `truncate -s 104857601 base.txt && git commit -qam x` | missed-silently |
| `commit-repository-made-in-same-command` | `mkdir <新目录> && cd <新目录> && git init -q && truncate -s 104857601 big.bin && git add big.bin && git commit -qm x` | missed-silently：`.claude/hooks/bash-command-detector.sh:3302`：「if not os.path.isdir(directory):」之后直接 `continue`，不走 `unjudged`；文件头 ㉒「判不了的」那一串里没有「目录不在」 |

### git push 只判当前分支相对上游

`--guard push` 只看当前分支的 `@{upstream}..HEAD`；没有上游时 `.claude/hooks/bash-command-detector.sh` 记检出，`scripts/gate/large-file-check.py:371`：「return guard_result("push", limit, 0, [], [], not_run=why_not)」。推的是别的引用时什么都不记：文件头写了「推的别的分支不判」（`.claude/hooks/bash-command-detector.sh:438`），但这一句落在「判不了的（放行，往检出记录追加一条）」之后，读起来像是会记一条，代码不记。

| 格 | 序列（main 跟着 origin/main、自己没欠；分支 feature 有一笔带 big.bin 的提交没推） | 推的结局 |
|---|---|---|
| `push-named-other-branch` | `git push -q origin feature` | missed-silently |
| `push-all-branches` | `git push -q --all origin` | missed-silently |
| `push-mirror` | `git push -q --mirror origin` | missed-silently |
| `push-tags` | 在 feature 那一笔上打 tag，`git push -q origin --tags` | missed-silently |
| `push-recurse-submodules` | 子模块里一笔带 big.bin 没推，超级仓记了新的 gitlink 已提交，`git push -q --recurse-submodules=on-demand` | missed-silently：超级仓那一笔只有 gitlink |
| `push-new-branch-first-time` | 停在 feature（没上游）上，`git push -q -u origin feature` | passed-recorded：新分支头一次推一律只记不拒 |
| `push-control`、`push-head-to-other-ref` | 当前分支没推的提交里加了又删 big.bin，`git push -q`、`git push -q origin HEAD:other` | rejected（对照） |

### 推不出的一格：从子目录起、相对路径的 GIT_INDEX_FILE

钩子把相对路径照命令所在的目录解：`.claude/hooks/bash-command-detector.sh:3313`：「extra_environment["GIT_INDEX_FILE"] = os.path.normpath(os.path.join(directory, os.path.expanduser(index_file)))」。git 自己在子目录里起的时候先换到工作区顶层再读索引，相对的 `GIT_INDEX_FILE` 按顶层解还是按原目录解，我没核（印象里是顶层，没把握）。探针两种都摆：私有索引放在顶层（`commit-relative-private-index-kept-at-top`）与放在子目录（`commit-relative-private-index-kept-in-subdirectory`），命令都是在 `sub/` 里 `GIT_INDEX_FILE=probe.idx git commit -qm x`。git 若按顶层解，前一格是 missed-silently：钩子去读 `sub/probe.idx`，那份不在，git 当空索引，判法只看到删除。

### 不成立的几形（推的，待主 agent 跑）

`commit-control-staged`、`commit-amend-with-staged-big`、`commit-message-from-file`（`-F`）、`commit-git-config-prefix`（`git -c … commit`）、`commit-command-builtin-prefix`（`command git commit`）、`commit-add-update-chain`（`git add -u && git commit`）、`commit-through-sh-c`、`commit-through-eval`、`commit-through-heredoc-to-bash`、`commit-after-pushd`、`commit-exported-private-index`、`push-head-to-other-ref`：照拒。eval 与喂给 bash 的 heredoc 两格我只读了切词模块的文件头（它说 eval 的参数与喂进 shell 的字面文本照切），没逐行追，待跑。

## 第 7 问：㉒ 误拒与拖慢 —— 成立

### 误拒

- 没暂存的改大文件（`false-unstaged-grown-plain-commit`）、被忽略的大文件加 `git add -A`（`false-ignored-big-add-all`）、软链接指到仓外大文件（`symlink-to-outside-big-staged`、`symlink-to-outside-big-add-all`；工作区那一半用 `lstat`、只认普通文件，暂存区那一半的 blob 是链接目标的路径）：放行，也确实不进提交。不成立。
- 带空格与非 ASCII 的路径（`path-with-space-and-non-ascii`，路径 `目录 名/大 文件.bin`）：git 用 `-c core.quotepath=false` 与 `-z` 起（`scripts/gate/large-file-check.py:80`），JSON 默认转义，拒绝说明用 `shlex.quote`。照拒，不成立。
- 推的不是当前分支而当前分支欠着大文件：`push-delete-other-ref-while-current-dirty`（`git push -q origin :stale`，只删远端一个引用）、`push-clean-side-branch-while-current-dirty`（`git push -q origin side`，side 是干净的旧提交）。钩子照当前分支拒，什么都不会传上去。成立（误拒）。
- `-a` 取暂存区与工作区的并集只会多拒：工具员已写明，不另算。

### 全局 git 配置会改判法的结局

`guard_worktree` 带路径时从命令的目录起 `git diff`（`scripts/gate/large-file-check.py:346`：「base = directory if pathspecs else top   # 路径从调用的目录起算；不给路径是整棵树」），再按顶层拼路径 `lstat`、拼不出就 `continue`（`scripts/gate/large-file-check.py:361`）。`git diff --raw` 没带 `--no-relative`（`scripts/gate/large-file-check.py:302`），全局配了 `diff.relative=true` 时输出的路径相对子目录，拼出来的路径不在，改大的文件看不见。格：`commit-path-from-subdirectory-diff-relative-config`（子目录 `sub/` 里 `git commit -qm x data.bin`，data.bin 是跟踪文件、工作区改大了，`GIT_CONFIG_GLOBAL` 指到一份写了 `diff.relative = true` 的配置），对照格 `commit-path-from-subdirectory-control` 不配它。推的结局：前一格 missed-silently、对照格 rejected。本机全局配置里没有 `diff.relative`（量的：`git config --get diff.relative` 退 1）；第二台的配置我不读，那一台判不判得对推不出。

### 起几次、合计多久、settings 的时限

- 次数（读码）：每条 commit 一次 commit-index；`-a` 再加一次 commit-worktree；每条排在前面的 `git add` 各加一次；`-C` 换一个仓就各算一套；push 一次。同键的复用（`finished_runs`）。假判法记下每次起它的参数，探针报 `PROBE name=judge-runs-<写法> judge_runs=<次数>`，推的次数：`git commit -qm x` 1、`git commit -qam x` 2、`git add a.txt && git add b.txt && git commit -qam x` 4、`git commit -qam x && git commit -qam y` 2、`git push -q` 1、三个仓各 `-C … commit -qam x` 6、`git add a.txt && git commit -qm x && git push -q` 3。
- 每次的时限是 `.claude/hooks/bash-command-detector.sh:3114`：「LARGE_FILE_CHECK_SECONDS = 25」，注释说「钩子要在整体时限内返回」，代码里没有整体的预算：次数不封顶，4 次到点就是 100 秒。`.claude/settings.json:83` 那一条注册没写 `timeout`，用的是 Claude Code 的缺省值（我印象里是 60 秒，没核）。超过注册时限时 Claude Code 停掉钩子、照常放行（印象，没核）；而这一份钩子把全部拒绝攒到 main 末尾才一起打（`.claude/hooks/bash-command-detector.sh:9498` 一带），被停在 ㉒ 里时，同一条命令里已经判出的 ⑥、⑪、㉑ 等拒绝一并丢掉，检出也来不及记。格 `judge-fails-slow-with-child`：假判法睡 60 秒，命令 `git add a.txt && git add b.txt && git commit -qam x`、`timeout` 600000，推的 `hook_seconds` 约 100、`refused_21=yes`（钩子本身没被停，因为探针不设 60 秒）。
- 本机仓根上判法用的 git（量的，`GIT_OPTIONAL_LOCKS=0`，两遍）：`git diff --cached HEAD --raw` 0.00 秒；`git diff HEAD --raw`（commit-worktree）0.23 与 0.02 秒；`git ls-files --others --exclude-standard` 0.14 与 0.13 秒（57 个）；`git ls-files --others`（`--ignored-too`）0.39 秒（12394 个）。仓里跟踪 14496 个文件，对象库 686.13 MiB 打包，当前分支相对上游没有欠的提交。这是本机数（本机盘、有别的会话在跑），第二台经共享根读同一棵树会慢，没量。整个钩子在仓根上判几种写法的耗时由 `main-repository-hook-seconds-*` 几格量，待主 agent 跑。`guard_unpushed` 的耗时按 `rev-list --objects <上游>..HEAD` 走，跟没推的历史长短走、不跟对象库总量走（读码）。

### 判 commit-worktree 时顺手写回索引

`git diff`（不带 `--cached`）遇到只动了时间戳、内容没变的跟踪文件会顺手刷新并写回索引，拿 `index.lock`（这是我对 git 的印象，没核）。判法起 git 没设 `GIT_OPTIONAL_LOCKS=0`，也没用 `--no-optional-locks`（`scripts/gate/large-file-check.py:80`）。几个会话共用一个仓时，别的会话同一刻的 `git add` 会撞 `index.lock exists`；钩子本该只读。格：`index-written-back-by-judge`（`touch` 过 base.txt 之后喂 `git commit -qam x`，看 `.git/index` 的 mtime 与内容变没变）与对照格 `index-written-back-by-judge-optional-locks-off`。推的：前一格 `index_mtime_changed=yes`、对照格 `no`。

## 第 8 问：判法出错与钩子挂掉 —— 四种不成立，另有两处窄的成立

`large_file_check_run` 只认退 0、1 且标准输出最后一行是 JSON 对象（`.claude/hooks/bash-command-detector.sh:3238` 起那一段），别的都交原因、走 `unjudged` 记检出。探针用一棵假判法树（拷被判那棵树的 `.claude/hooks`、`.claude/scripts`、规范副本的 `scripts`、判据库，把 `scripts/gate/large-file-check.py` 换成 `fake_large_file_check.py`，`PROBE_FAKE_MODE` 选行为），喂 `git commit -qm x` 且 `timeout` 600000（同一条命令里另有一类 ㉑ 拒绝，看它丢不丢）：

| 格 | 假判法做什么 | 推的结局 |
|---|---|---|
| `judge-fails-raise` | 抛 Python 异常（退 1、标准输出空） | 「输出读不成 JSON」，记检出；㉑ 照拒 |
| `judge-fails-json-then-noise` | 先打 JSON 再打一行别的 | 最后一行不是 JSON，记检出；㉑ 照拒 |
| `judge-fails-noise-then-big` | 先打一行别的、再打有超限项的 JSON、退 1 | 照最后一行拒；㉑ 也拒 |
| `judge-fails-exit-three` | 带 error 的 JSON、退 3 | 记检出；㉑ 照拒 |
| `judge-without-git-on-path` | 真判法，PATH 里除了 git 什么都有 | 判法的 preflight（`run-condition: command git`）退 78，记检出 |
| `judge-fails-keys-missing` | JSON 合法，`too_big` 那一项缺 `path` | 成立：`large_file_messages` 读 `item['path']` 抛 KeyError，这一步不在 try 里（`.claude/hooks/bash-command-detector.sh:9498`：「refusal_lines.extend(large_file_messages(large_files))」），main 抛出，bash 那一层记一条「python 那一层没起来或没判完」、退 0：这条命令照常执行，同一条命令里的 ㉑ 拒绝也丢了 |
| `judge-fails-slow-with-child` | 起一个 `sleep 200` 子进程、自己睡 60 秒 | 成立（窄）：`subprocess.run(…, timeout=…)` 到点只停直接起的那一个，判法起的子进程（真判法里是 git）留在钩子的进程组里；探针报 `leftover_after_hook=` |

缺键那一格只在判法与钩子版本对不上时出现（两份都从同一棵树起，见第 11 问），今天的判法打不出缺键的 JSON，所以算窄；它的害处是「判法有一个 bug」放大成「整条命令的全部拒绝都没了」。

## 第 9 问：出路照做通不通 —— 成立

- 一条一条照抄（`remedy-followed-step-by-step`，路径 `目录 名/大 文件.bin`）：探针从拒绝说明「→ 怎么办」那一行抠出反引号里的三条，每条先喂 `.claude/settings.json` 里挂在 Bash 上的全部 PreToolUse 钩子（主 agent 的写法），再在仓的顶层执行，最后喂 `git commit -qm x` 并执行。推的：三条都不被拒（`git rm --cached` 带了具体路径，⑮ 不认它是清整个索引；⑱ 管 checkout / restore / stash / clean，不管 rm；⑰ 只管往仓里写字面内容），最后的 commit 放行、没有超限的 blob 进提交。读 `PROBE name=remedy-followed-step-by-step-step … refused_by=` 与 `…-commit-after-remedy … verdict=`。
- 三步与 commit 串在同一条命令里（`remedy-chained-in-one-command`）：`gzip -n -9 -k big.bin && git rm -q --cached big.bin && git add big.bin.gz && git commit -qm x`。钩子判的是执行之前的暂存区，big.bin 还在，照拒；拒绝说明再给同一套出路。推的结局 rejected-nothing-lands（误拒）。说明里没写「这几步单独起、做完再另起 git commit」。
- 不可压缩的大文件（`remedy-incompressible`，上限加一字节的随机字节）：gzip 压不小随机数据，`.gz` 比原文件还大几十 KB（deflate 存储块加头尾，算术），照抄三步之后 commit 拒的是 `大 文件.bin.gz`，出路又叫它 `gzip -n -9 -k '…bin.gz'`。推的：`…-gzip-size gz_bytes=` 大于 104857600，`…-commit-after-remedy` 是 rejected。说明里没有「压完还超的怎么办」。
- push 那一支（`remedy-push-message`）：出路是「重做提交时压成 `gzip -n -9 -k …`、`git add ….gz` 入库」与「推之前重做那几笔提交」，没有一条能照抄的重做命令（`git reset --soft <上游>` 之后重提交这类）；而重做最顺手的 `git reset --soft … && git commit` 正是第 6 问 `commit-redo-after-reset-soft` 那一形，commit 那一刻不拦。成立（弱）：探针报 `names_a_redo_command=no`。

## 第 10 问：自证与开关

- 两条弄坏同一处：不成立。九条各只在一处判：allow-long-foreground-timeout 第 3089 行，allow-large-file-commit 与 allow-large-file-push 同在第 3285 行、各管一支（`.claude/hooks/bash-command-detector.sh:3285`：「if kind == "commit" and detector_break_is_set("allow-large-file-commit") or kind == "push" and detector_break_is_set("allow-large-file-push"):」），commit-all-flag-unseen 第 3167 行，large-file-every-git-command 第 3261 行，large-file-unjudged-silent 第 3275 行，commit-private-index-unseen 第 3309 行，commit-content-worktree-always 第 3325 行，commit-add-chain-unseen 第 3329 行（`grep -n` 现取，每个名字在代码里除开关名单之外只出现一次）。
- 点名的格名不副实：成立（弱）。`.claude/hooks/bash-command-detector.sh:8644`：「("git commit -m 的值是 -a，不当 -a 认，放行", "git commit -m -a", large_unstaged, 0),」挂在 commit-content-worktree-always 下；那一条开关让每个不带路径的 commit 都判工作区，这一格红是因为这个，不是因为 `-m` 的值被当成了 `-a`。`git_commit_form` 跳过选项取值那一段（`GIT_COMMIT_OPTIONS_WITH_VALUE`、`GIT_COMMIT_SHORT_OPTIONS_WITH_VALUE`）没有一条开关证它会红，第 6 问的缩写漏拒就落在这一段。
- 真 git 仓的格与 `scripts/gate/selftest-process-guard.py`：它盯的是真 cargo / rustc / qemu / herd7 / verus 与清场脚本；自证里只起 git，本机不成立。但自证给 git 的环境是整份 `os.environ` 去掉三个变量（`.claude/hooks/bash-command-detector.sh:8571`：「large_environment = {name: value for name, value in os.environ.items() if name not in ("GIT_INDEX_FILE", "GIT_DIR", "GIT_WORK_TREE")}」），不隔全局与系统配置：哪一台的全局配置有 `core.hooksPath` 或 `init.templateDir` 带了钩子，自证里的 `git commit` 就照跑那些钩子；配了 `diff.relative` 的，第 7 问那一形也会让自证的格变。本机全局配置里这三项都没有（量的：`git config --global --list`，私有值不贴）；第二台推不出。
- 门限变量平常跑认不认：不认，成立（不是打中）。`.claude/hooks/bash-command-detector.sh:3227`：「return int(value) if marked and value.isdigit() and int(value) > 0 else None」。标记是一份内容固定的文件，谁的进程环境里两个变量都设着就认；钩子的环境是 Claude Code 进程的，Bash 命令里 `export` 改不到，所以平常跑认不到。附带一处：文件头说「自证把门限压小」（`.claude/hooks/bash-command-detector.sh:442`），代码收任何正整数，标记对得上时把门限调大也认。格：`limit-variable-without-mark`（放行）、`limit-variable-with-made-mark`（拒）、`limit-raised-with-made-mark`（门限设 10¹⁵，真超限文件放行）。

## 第 11 问：仓副本、git worktree、第二台各读哪一份判法 —— 成立

- 钩子从 `.claude/settings.json:83` 的 `bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/bash-command-detector.sh` 起，`HOOK_DIR` 取脚本自己的目录（`.claude/hooks/bash-command-detector.sh:1251`：「HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"」），`repository_root_of` 往上两级（`.claude/hooks/bash-command-detector.sh:4873`），起的是 `<那一级>/scripts/gate/large-file-check.py`（`.claude/hooks/bash-command-detector.sh:3232`）。所以：会话开在主仓、命令 `cd` 进仓副本或 git worktree 再 commit 的，起主仓那一份判法、判副本那棵树的暂存区。不成立。
- 会话开在 git worktree 里（`CLAUDE_PROJECT_DIR` 是 worktree）的，起 worktree 里那一份判法。它从自己那棵树找规范副本：`scripts/gate/large-file-check.py:63`：「sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))」、`scripts/gate/large-file-check.py:64`：「from preflight import preflight  # noqa: E402」。`.claude/singlefs-ai-sop/` 被 `.gitignore` 第 8 行忽略（量的：`git check-ignore -v`），`git worktree add` 新建的树里没有它；钩子自己的 bash 那一层会退到 git 公共目录里那一份（`.claude/scripts/preflight.sh` 文件头），判法不会。结局：判法 ModuleNotFoundError、退 1、标准输出空，每一次 commit / push 都只记检出不拒。格：`tree-hook-from-linked-worktree`（推的 passed-recorded-nothing-lands）与对照 `tree-hook-from-main-tree`（rejected-nothing-lands）。本机现有的四棵 worktree 里都有人拷过规范副本（量的），今天不中；新建的 worktree 都中。
- 第二台：仓在共享根上、两台同一绝对路径，读的是同一份判法；Python 是钩子自己的 `sys.executable`、git 是那一台 PATH 上的。判法的结局还读那一台的 git 全局配置（第 7 问），那一台的配置我不读，推不出。

## 打中之后的四句

这一轮没有臂，「分不分辨」按「现实现中、对应的改法不中」读。

| 打中的一组 | 分不分辨 | 钩子那一刻看不看得到 | 满足的判据分句 | 改法还中不中 |
|---|---|---|---|---|
| 别名、xargs、find -exec | 分辨 | 看得到：别名在 `-c` 与仓的配置里，xargs / find 在命令文本里 | 第 6 问「放行、也不记检出」 | 「剥启动器、读别名」不中（推的） |
| 长选项缩写 | 分辨 | 看得到：参数原文在 | 同上 | 「认不全的选项算判不了」不中（推的） |
| 同一条命令里先改状态 | 分辨 | 看不到改之后的大小（文件还没造、暂存区还没变）：判别子观测不到，改法只能改判据，不判大小、改成认「前面有改状态的步骤」 | 同上 | 「前面有改状态的步骤就单独起」不中；只改成记检出的弱版落到 passed-recorded（推的） |
| 目录不在 | 分辨 | 看得到：目录不在 | 同上 | 「目录不在也记检出」落到 passed-recorded；配上一条一起才拒（推的） |
| push 别的引用、新分支头一次推、子模块递归 | 分辨 | 看得到：引用在参数里、远端跟踪引用在本地 | 同上；新分支那一格只满足「放行」不满足「不记」 | 「按推出去的引用判」不中；子模块递归要另判每个子模块（推的） |
| push 的两格误拒 | 分辨 | 看得到 | 第 7 问「不该拒的误拒」 | 「按推出去的引用判」不中（推的） |
| diff.relative | 分辨 | 看得到：配置在 | 第 6 问与第 7 问（不该放的放了） | 「--no-relative 与 --no-optional-locks」不中（推的） |
| 索引写回 | 分辨 | 看得到 | 第 7 问「把钩子拖慢或弄挂」里的副作用一类（字面没点名，我归进去的） | 同上不中（推的） |
| 时限没有总预算、到点留子进程 | 分辨 | 看得到：次数在钩子里数得出 | 第 7 问「合计最长多久」、第 8 问 | 「整体时限与停整组」不中（推的） |
| 缺键的 JSON | 分辨 | 看得到 | 第 8 问「钩子自己抛异常、退别的码」 | 「拒绝说明那一步放进 try」不中（推的） |
| 出路串成一条、不可压缩、push 没给命令 | 分辨 | 看得到 | 第 9 问「照做通不通」 | 「出路补两句」只改说明：串成一条的照拒但有了正确的出路，不可压缩的有了去处（推的） |
| worktree 里判法起不来 | 分辨 | 看得到：规范副本不在 | 第 11 问 | 「判法退回公共目录找规范副本」不中（推的） |
| ㉑ 的字符串与 "true" | 分不出：钩子侧照字面判得一致，结局取决于 harness | 钩子看得到原值，看不到 harness 怎么用它 | 第 5 问（推不出） | 先做分辨观测（第 5 问一节），再定 |

## 改法与各修哪一格

全部是我提的，只在我的探针里摆了格、一格都没跑过，被攻过零轮；每一格都是「推的，待主 agent 跑」（主 agent 的复跑只量现实现，改法没有实现，A/B 要另做）。

| 改法 | 改哪里 | 修的格 |
|---|---|---|
| 按推出去的引用判 | `--guard push` 收要推的引用（`--all` 取全部分支、`--mirror` 取全部引用、`--tags` 取全部标签、点名的取 refspec 冒号前那一半，不给取当前分支），基准用 `--not --remotes=<远端>` 而不是 `@{upstream}`；删引用（`:x`、`--delete`）不判 | push-named-other-branch、push-all-branches、push-mirror、push-tags、push-new-branch-first-time、两格 push 误拒；push-recurse-submodules 另要对每个子模块各判一次 |
| 剥启动器、读别名 | `large_file_findings` 照 project-code-guard 那样调 `lib_project_code_runs.py` 的 `command_under_extra_launcher`，读切词模块交出的 embedded launches；别名照 `git config --get-regexp '^alias\.'` 与 `-c alias.*` 换成子命令 | commit-repository-alias、commit-command-line-alias、commit-command-line-shell-alias、commit-through-xargs、commit-through-find-exec |
| 认不全的选项算判不了 | `git_commit_form`、`git_add_form` 遇到不是全名的 `--选项`，照 git 的唯一前缀规则展开，展不开走 `unjudged` | commit-abbreviated-message-option、commit-abbreviated-include-option、commit-abbreviated-force-add |
| 前面有改状态的步骤就单独起 | 同一条命令里排在 git commit 前面的，除了 `git add` 与只读命令，有写文件（重定向、cp、mv、truncate、gzip 这类）、改暂存区的 git 子命令（stash、cherry-pick、checkout、restore、merge、reset、read-tree、update-index、apply、mv、rm）、建仓（init、clone、worktree）或 cd 进还不在的目录的，拒，出路「git commit 单独起一条」 | 第 6 问「先改了状态」那一张表的十格、commit-repository-made-in-same-command、remedy-chained-in-one-command（误拒变成带对的出路的拒） |
| 目录不在也记检出 | `.claude/hooks/bash-command-detector.sh:3302` 那一处 `continue` 之前调 `unjudged` | commit-repository-made-in-same-command（落到 passed-recorded） |
| --no-relative 与 --no-optional-locks | `raw_entries` 的 `git diff` 加 `--no-relative`；`git()` 加 `--no-optional-locks` | commit-path-from-subdirectory-diff-relative-config、index-written-back-by-judge |
| 整体时限与停整组 | `large_file_findings` 设一份合计预算，到点剩下的记检出；判法用新会话起、到点停整组；settings 那一条写明比预算长的 timeout | judge-fails-slow-with-child |
| 拒绝说明那一步放进 try | `large_file_messages` 与它的调用包进 try，出错记检出、别的拒绝照报 | judge-fails-keys-missing |
| 出路补两句 | 说明写「这几步单独起，做完再另起 git commit」「压完还超的不入库，放仓外或拆开」，push 那一支给整条重做命令 | remedy-chained-in-one-command（说明）、remedy-incompressible、remedy-push-message |
| 判法退回公共目录找规范副本 | `large-file-check.py` 找不到自己树里的规范副本时，照 `.claude/scripts/preflight.sh` 退到 `git rev-parse --git-common-dir` 那一级 | tree-hook-from-linked-worktree |
| 自证隔开全局配置 | `large_environment` 加 `GIT_CONFIG_GLOBAL=/dev/null`、`GIT_CONFIG_NOSYSTEM=1` | 第 10 问第二台推不出那一格 |
| 给选项取值补一条开关 | 新开关弄坏 `GIT_COMMIT_OPTIONS_WITH_VALUE` 的跳过，点名「-m 的值是 -a」那一格 | 第 10 问格名不副实那一格 |
| 相对 GIT_INDEX_FILE 照 git 的解法 | 等探针那两格的结局，照 git 实际读的那一处解 | commit-relative-private-index-kept-at-top（若打中） |

## 要主 agent 跑的命令

两条都不起 cargo、不点验证包，没写范围声明；范围声明闸若把它们认成实验复跑而拒，照拒绝说明补一句（种类 items，跑什么写「攻方腿探针」）。

1. 先跑一小段估全量挂钟（两组：㉑ 的写法与 commit 那一侧）：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-opus -- python3 /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r1-opus-model/probe.py --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-opus/estimate --only timeout,commit`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-opus`（草稿目录）。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 `PROBE name=frozen-hash` 四行都是 `match=yes`，`group-seconds-timeout`、`group-seconds-commit` 两行的秒数，没有 `group-error-` 行，末行 `PROBE name=done emitted=<N>`。估全量：commit 那一组约 33 格，全量约 110 格加假判法「慢」那一格的 100 秒，按比例乘；超过 40 分钟就别交第 2 条，交回我缩格。
2. 整份复跑：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-opus-run /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r1-opus-model`
   - 在哪起：仓根（第二个参数是冻结副本，有开工快照树就换成它）。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`（探针有一组装置出错退 1，日志里是 `PROBE name=group-error-<组>`）；`task-server/task log <号> --all` 里每格一行 `PROBE name=<格名> … verdict=…`，逐格对本报告「各格判定一览」与各节里写的推的结局；`frozen-hash` 四行 `match=yes`；`hook_leftover=` 与 `leftover_after_hook=` 两列除了 `judge-fails-slow-with-child` 都是 0；末行 `PROBE name=done emitted=<N>` 的 N 等于日志里 `PROBE name=` 行数减一。全文另在 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-opus-run/probe.out`，各格钩子的 stderr 在同目录 `probe/stderr/`。

另有两样不是任务、子 agent 做不了的观测（第 5 问）：主 agent 发一次 Bash 调用，命令 `sleep 1`，`timeout` 写成字符串 `"3e5"`；再发一次 `run_in_background` 写成字符串 `"true"`、`timeout` 600000。看 harness 报输入校验错、还是钩子拒或放、命令在前台还是后台跑。

## 没打中的形状

- ㉒ 认得出的几种前缀与包装：`git -c`、`command`、`nice`、`env`、`sh -c` / `bash -c`、`eval`、喂给 bash 的 heredoc、`pushd`、`export GIT_INDEX_FILE`、`-F` 与 `-m` 的取值、`-am` 这类合写（读码，推的）。
- `--amend` 带着暂存的大文件、`git add -u &&`、`git push origin HEAD:other`（当前分支有上游）照拒（推的）。
- 判法出错的四种（异常、多一行、退 3、没有 git）都记检出（推的）。
- 九条开关没有两条弄坏同一处（读码）。
- 门限变量平常跑不认（读码）。
- 本机仓根上判法的 git 耗时（量的，都在 0.4 秒内）。
- 取样范围：commit 一侧 33 格、push 一侧 10 格、误拒 5 格、出路 3 组、㉑ 19 格、假判法 13 格、开关与门限 3 格、树 3 格、索引 2 格、主仓挂钟 5 种写法；大文件只取「上限加一字节」一个尺寸（判法是严格大于，恰好等于的已由自证 `limit-inclusive` 那一格管）。

## 这条腿自己的限度

- 探针一格都没跑，「推的」结局没有一个是量出来的；git 的几处行为（长选项缩写、`stash pop` 恢复新文件到暂存区、相对 `GIT_INDEX_FILE` 的解法、`git diff` 写回索引）是我对 git 的印象，探针那几格就是核它们。
- Claude Code 的钩子缺省时限与超时之后怎么办、harness 怎么处理字符串的 timeout 与 run_in_background，都是印象，仓里没有出处，标推不出。
- `probe.py` 第一段一次写了 156 行，超了每次 150 行的分段规矩（后面各段都在 110 行以内）；内容不受影响。
- 只读了「范围」点名的三份与它们调到的几处（切词模块文件头、`preflight.sh`、`selftest-process-guard.py` 文件头、`.claude/settings.json`）；正推腿那一份在禁读清单里，没读。
- 第二台的 git 版本与全局配置没读（私有、也不该由我读），那几格推不出。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；改法都没实现，副本上也没有数。
- 没往统一任务服务交任务，判决要引的数一个都没复跑：命令在「要主 agent 跑的命令」一节。
- 没建仓副本、没建编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-opus` 里只有认领、锁与 cite-check 的输出三份文件。
