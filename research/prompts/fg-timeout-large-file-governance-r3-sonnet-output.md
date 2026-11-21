# fg-timeout-large-file-governance-r3 正推腿报告（Sonnet，第 1 到 4 问）

日期 2026-11-19。读的是主工作区换上之后的那一版：开工快照 `research/prompts/fg-timeout-large-file-governance-r3-snapshot/judged-sha256.txt` 十五份，开工时 `sha256sum -c` 全部 OK。
读的范围：正文「范围」一节列的文件我都读了；`.claude/hooks/runner-dispatch-guard.sh` 只读了 ④ 的文件头与它的自证格，没有通读（第 1 问第 5 条另写）。
下面标「推」的是读代码推的，标「量」的是我此刻在仓里用只读命令量到的，标「待主 agent 跑」的是要起真 git 或项目代码才得到的，我没跑。

## 模型与复跑

模型目录 `research/prompts/fg-timeout-large-file-governance-r3-sonnet-model/`，没有拷第二轮的辩方探针（那一组量的是已经删掉的「读命令文本」那一套，装置对不上新做法），新写了一份真 git 的探针；派发闸 ④ 那一份是第二轮攻方的 `dispatch_scope_model.py` 原样拷来（与来源 sha256 相同，一个字没改）。

| 文件 | sha256 |
|---|---|
| `probe_hooks.py`（新写，沙盒仓里跑真 git：提交、合并、冲突后收尾、重放、推送、安装、窗口闸，另把日常提交写法逐条喂检出钩子 ㉒，不执行） | `45eda934d287e9e546dbe9fb9361b105eb464fc49d48203e46b92152e763c0c1` |
| `dispatch_scope_model.py`（第二轮攻方 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-main/opus-model/dispatch_scope_model.py` 原样拷来） | `081e201ae36d9280e84876fd4d2513a0b234639c39e227e495c63ff6caae79a4` |
| `tools/rerun.sh`（`scripts/agents/three-way-rerun-template.sh` 拷来，只改 `rerun_steps`） | `2de115513cc2a020f6cbcf380405056c6c6fe0c187c2d195b4acf6faa2e28fd6` |
| `tools/rerun-from-a-clean-work-directory.sh`（范例 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh` 原样拷来） | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

复跑命令在下面「要主 agent 跑的命令」第一条。`probe_hooks.py` 只写 `--work` 目录，读 `--repo`（主工作区）里的钩子与检出钩子；沙盒仓里的超限文件是真的 104857601 字节稀疏文件，钩子的假限额只在自证里认，探针命令行里没有 `--selftest`，所以不会误开。
探针的内存与时间我没量过（子 agent 不跑）：估内存不到 1 GiB（git add 一个 100 MiB 的稀疏文件时整份读进内存），估一两百次钩子起停加三十来个 100 MiB 文件的 add，几分钟；运行目录里留下的是稀疏文件与压成很小的对象，估不到 100 MiB，用完 `bash scripts/hosts/work-directory-ownership.sh sweep` 或点名删。
我没起过它：语法用 `ast.parse` 查过，名字用一段 `ast` 扫描查过没有未定义的；第一次真跑如果有一格装置出错，会写 `PROBE name=setup-error-<组> …` 并退 1，其余组照跑。

## 结论先写

| 问 | 回答 | 打中 |
|---|---|---|
| 1 第二轮「三、改法」六条 | 第 1、4 条做到；第 2 条做到并多做了一件、少留了一件；第 3 条只做到一半（腿名收窄了，轮没收窄）；第 5 条这批没碰、待跑；第 6 条没做 | 第 6 条；第 3 条的轮；第 2 条的字面偏差 |
| 2 钩子判的是不是那一刻真要进的内容 | 提交、合并提交、`--no-commit` 之后、点名路径、`-a` 成立；`--amend` 成立但只比 HEAD；冲突后 `--continue` 推不出；push 的每一行引用里指着非提交对象的不成立 | 三处 |
| 3 安装与核对 | 两台推不出（没连过真第二台）；相对路径怎么解成立；检出老提交的工作树不成立（三棵现成的树里没有钩子，核对只写 ⊘，检出钩子不拦）；格与三份样本判的不是同一件事 | 四处 |
| 4 窗口闸、环境变量与 ㉒ | 环境变量不绕开大文件判法成立；日常写法没有被拒的推不出（待喂）；主 agent 记在 `kb/tooling.md` 里的 worktree 提交写法绕开大文件判法不成立 | 一处 |

## 第 1 问：第二轮判决「三、改法」六条逐条

判决里的条文在 `research/prompts/fg-timeout-large-file-governance-r2-main-verification.md` 第 52 到 57 行。

### 第 1 条：钩子进仓、安装与核对、窗口闸接进去、不再靠 `--no-verify`

- 做的是这件事，量到的行：
  - 三份钩子只 `exec` 判法：`scripts/gate/git-hooks/pre-commit:10` 是 `exec python3 -B "$here/../large-file-git-hook.py" pre-commit "$@"`，`pre-merge-commit:11` 同形；`git ls-files -s` 三份都是 `100755`（量的）。
  - 窗口闸接进去：`scripts/gate/large-file-git-hook.py:248` 取 `$(git rev-parse --git-common-dir)/hooks/pre-commit`，`:252` 是 `return subprocess.run([private, *arguments]).returncode`，`:260` 之后才 `run_judge(os.getcwd(), "commit-index", [])`；私有闸退非 0 就先拒、大文件不判，退 0 照判。
  - 窗口外提交的写法换了：用户私有的 `.git/hooks/pre-commit`（不进仓）第 7 行写 `不用 --no-verify：它连大文件那一道一起绕开，检出钩子也拦它。`，第 8 行是 `if [[ "${SINGLEFS_COMMIT_OUTSIDE_WINDOW:-}" == 1 ]]; then exit 0; fi`（量的，读的是这台本机的那一份）。
  - 安装：`scripts/gate/git-hooks-install.py:184` 写 `core.hooksPath scripts/gate/git-hooks`（相对路径，写进共用的 `.git/config`，量到 `git config --show-origin --get-all core.hooksPath` 是 `file:.git/config	scripts/gate/git-hooks`）。
- 偏差两处，下面第 3 问展开：这批新文件此刻都只在暂存区、不在任何提交里（`git ls-tree HEAD scripts/gate/git-hooks` 零行，量的），所以已有的三棵链接工作树里没有钩子目录；用户私有的窗口闸只管 `git commit` 与合并提交。
- 什么现象会推翻「做到了」：沙盒里喂一笔带超限文件的 `git commit`、`git merge`，钩子没拦；或设了 `SINGLEFS_COMMIT_OUTSIDE_WINDOW=1` 的提交没判大文件（探针 `window-open-by-variable-big-commit` 应是 `verdict="as-expected"`）。

### 第 2 条：检出钩子 ㉒ 收窄

判决原文：「只拦直接写的 `git commit` / `git push` 里能直接判的那一部分，再拦 `--no-verify`、`-n`、`-c core.hooksPath=` 这类绕开 git 钩子的写法。『判不准就拒』那一套与它带的放行名单、经 `~` 取工作根的那一行一起删。」

- 删掉的做到了：`READ_ONLY_COMMAND_NAMES`、`harmless_write_target`、`判不准就拒`、`GIT_READ_SUBCOMMANDS` 在 `.claude/hooks/bash-command-detector.sh` 里各 `grep -cF` 是 0（量的）；第二轮点名的经 `~` 取工作根的那一行随之不在（剩下 `:9847` 一处 `'~/.local/share/singlefs/work'` 只是拒绝说明里取不到工作根时的字面回退，不是取目录；这一格 `shared-data-root-literals` 的结局待主 agent 跑，命令在下面）。
- 绕开写法的拦做到了：`:3649` `if subcommand == "config" and git_config_writes_hooks_path(arguments):`、`:3658` `if subcommand in ("commit", "push") and git_hook_form_judged(subcommand, arguments):`；`--no-verify`、`-n`、`-c core.hooksPath=`、`--config-env`、`GIT_CONFIG_KEY_<n>`、`GIT_CONFIG_PARAMETERS` 都在 `git_hook_bypass_reasons` 里。
- 与判决字面不同的两处：
  - 判决要留「直接写的 commit / push 里能直接判的那一部分」，现在一点内容判断都不留：`:449` `不读命令文本判 git commit / git push 要带的内容有没有超过单文件上限的文件`。这与用户选的方向（内容全交 git 钩子）一致，但不是这一条的字面；
  - 多做了一件判决没写的：「钩子没装上」的拒（`:3548` `def git_hooks_not_installed`，`:3557` `if code != 0 or not top or not library.project_checkout(top):`），这件事的判法见第 3 问。
- 推翻现象：探针 `detector` 一组里 `bypass-*` 十格有一格不是 `verdict="as-expected"`（应拒的放过）。

### 第 3 条：写范围表三方腿按腿的报告名收窄、八类写工作根照写者锁判

- 腿名收窄做到了：`.claude/hooks/agent-write-scope.tsv:61` `three-way-attack	research/prompts/*-opus-output.md`，`:65` 辩方与 `:69` 正推都是 `research/prompts/*-sonnet-output.md`，模型目录 `*-opus-model/**` 与 `*-sonnet-model/**` 同形。
- 判决第 35 行写的现象是「三方腿的模式能写别的轮、别的腿的报告」。「别的腿」（Opus 写 Sonnet 的报告）收窄了；「别的轮」没有：模式里轮名那一位仍是 `*`，`three-way-forward` 能写 `research/prompts/` 下任何一轮的 `*-sonnet-output.md` 与 `*-sonnet-model/**`。辩方与正推两个定义还共用同一条模式（`:65` 与 `:69`）。这一条只收到「按腿的报告名」，与判决第 54 行字面一致，但第 35 行那一半现象还在。推：靠表本身收不到轮（表是静态的、轮名每次不同），要靠派发闸或写闸按派发提示里的报告路径再判一道。
- 八类照写者锁判做到了：`:4446` `def writer_lock_refusal(raw_path):`，在 `bash_scope_verdict` 的 `judge` 里 `:2523` 接上，`:2651` 给只带 Read、Bash 的定义标 `read_only`；我自己就是这一类（`three-way-forward` 的 tools 是 `Read, Bash`），所以这一道对我的草稿目录写是真在判的。
- 推翻现象：定义里只带 Read、Bash 的 agent 往别的子 agent 持锁的草稿目录 `cp` 一个文件，⑫ 没拒（攻方腿第 8 问量）。

### 第 4 条：名单一致性、㉓ 出路共用读表、判法到点清索引拷贝、排除表、文件头

- 名单一致性按开关表的文法：`.claude/hooks/bash-command-detector.sh:1630` 的 `BREAK_SWITCH_TABLE_ENTRY` 与 `.claude/gate.d/code-tooling.sh:1279` 的 `SWITCH_ENTRY` 是同一个正则的两份写法（前者固定变量名），不是共用一份代码；「值后两个以上空格或（…）注，一个空格接命令的是用法示例」两边同一规则（`:1645` 起 `break_switch_name_sets`）。做的是这件事；两份抄写以后可能分叉（推）。
- ㉓ 出路与 `project-code-guard.sh` 共用读表：`.claude/hooks/project-code-guard.sh:681` `sys.path.insert(0, sys.argv[1]); import lib_code_discipline_lints as discipline_lints`，检出钩子 `clippy_lint_tail` 经 `.claude/hooks/bash-command-detector.sh:3358` 起的 `CLIPPY_LINT_LIBRARY` 导入同一个模块。做到了。表的写法两边读错与否是攻方第 9 问。
- 判法到点清索引拷贝：`scripts/gate/large-file-git-hook.py:157` `index_directory = tempfile.mkdtemp(prefix="large-file-git-hook-index-")`，`:180` `shutil.rmtree(index_directory, ignore_errors=True)` 在 `finally` 里，超时先 TERM 再 KILL 整组（`:132` 起 `stop_group`）。超时这一条做到了；本进程自己收到 TERM / HUP 时没有信号处理（该文件里 `signal.` 只出现在 `:135`、`:140` 两处，量的），索引拷贝所在目录留不留是攻方第 7 问。
- 排除表失效的那一行删了：`git diff HEAD -- .claude/gate.d/shared-data-root-literals-exempt.tsv` 只剩一行 `-scripts/agents/dispatch-prompt.py:SELFTEST_SCRATCH = …`（量的）。
- `large-file-check.py` 文件头照实写：`scripts/gate/large-file-check.py:30` 写「用法错（退出码 2）的说明与出路写到标准错误、不打 JSON；不带 --guard 的用法错写到标准输出」，`:501`、`:502` 的 `--guard` 分支确实走标准错误。

### 第 5 条：派发闸 ④ 的误拒

- 判决第 56 行写的是「归做全量提交门禁的会话那一批的 `runner-dispatch-guard.sh`，已问它并不并」。这批工具员没碰它：工具员报告 `~/.local/share/singlefs/work/fg-timeout-large-file-r2-fixes/result.md` 第 158 行写「五格 `false-refusal` 照旧（派发闸 ④ 不归这一批，没改）」。
- 此刻主工作区里那份文件已经比 HEAD 多了 553 行、少了 73 行（`git diff HEAD --stat -- .claude/hooks/runner-dispatch-guard.sh`，量的），文件头 `.claude/hooks/runner-dispatch-guard.sh:27` 起已写明「由主 agent 交写回员写进 …」「执行员之后把实验页写进 …，你只写登记」不判、「做成的事：」之下的列表不判。所以改法大概率由另一路会话的改动带进来的，这一批没有它的行。
- 五格是否真不再误拒，我读不出：规则是几十条正则与子句判定。推不出，算打中，待主 agent 把第二轮攻方量到五格误拒的那十二句原样喂换上之后的闸（`dispatch_scope_model.py`，命令在下面第一条里一起跑）。应有的结局：`sweep-input-lists-what-main-agent-updated`、`prior-art-says-who-writes-kb`、`attack-leg-told-where-verdict-goes`、`designer-told-what-runner-writes-later`、`triage-told-who-adds-assertions` 五格 `verdict="correct"`（第二轮是 `false-refusal`）；`sweep-candidate-table-in-work-root`、`attack-leg-own-report`、`materials-told-to-write-judged-list`、`designer-own-preregistration` 四格仍是 `correct`（叫它写的照拒不退）。
- 推翻现象：五格里任何一格仍是 `false-refusal`。

### 第 6 条：第一轮判决两处字面照实改——没做

- 判决第 43 行的两处：Q01 造仓的是 `probe-guard.sh` 第 14 行不是第 16 行；400 位整数那一格放行是 `float()` 溢出、不是「认不出的字符串」。
- `research/prompts/fg-timeout-large-file-governance-r1-main-verification.md` 现在（量的，`git status --short` 是 `A ` 即暂存区与工作区一致）：
  - 第 55 行还是 `正推腿 Q01 的预期是错的：`probe-guard.sh` 第 16 行在造仓时已经把 big.bin 提交并推了，`checked 0` 是对的。`；`grep -nF '第 14 行'` 零命中；
  - 第 56 行还是 `T15（400 位整数放行并记检出，`新增检出 1 条`）、timeout-string-exponent 等认不出的字符串放行：…`，没写 `float()` 溢出，`grep -nF 'float'` 零命中；
  - 我对着 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-main/sonnet-probes/probe-guard.sh` 量过：造仓、`git add big.bin`、提交并推的是第 14 行（量的，`sed -n 14p`），判决的说法「第 16 行」是错的，第二轮判决对。
- 正文「实现今天的样子」写「第一轮判决两处字面照实改了」，与这两行对不上。推翻现象：`grep -nF '第 16 行' research/prompts/fg-timeout-large-file-governance-r1-main-verification.md` 零命中且第 56 行写出 `float()` 溢出。这是纯文字，改一次两处。

### 第 1 问小结

| 条 | 判定 | 一句话 |
|---|---|---|
| 1 | 成立 | 钩子进仓、安装与核对、窗口闸先调再判、不靠 `--no-verify`；钩子目录还没进任何提交 |
| 2 | 成立，字面有两处不同 | 删的删了；内容判断全删（判决要留一部分）；多了「钩子没装上」的拒 |
| 3 | 推不出（一半成立） | 腿名收窄做到；轮名没收窄；八类写者锁做到 |
| 4 | 成立 | 五处都能指到行；名单文法是正则两份抄写；索引拷贝只处理了超时 |
| 5 | 推不出 | 这批没碰这份文件；文件头已写五种转述，待喂十二句核 |
| 6 | 不成立 | 第一轮判决第 55、56 行原样没改 |

## 第 2 问：git 钩子判的是不是「那一刻真要进提交、真要推出去的内容」

提交那一侧的判法都是同一条路：git 起钩子之前已把这一笔真要提交的东西放进钩子看到的索引（`GIT_INDEX_FILE`），`scripts/gate/large-file-git-hook.py:260` 起 `large-file-check.py --guard commit-index`（当前目录 `os.getcwd()` 是钩子的当前目录，也就是工作树顶层），`scripts/gate/large-file-check.py:419` 的 `given = os.environ.get("GIT_INDEX_FILE")` 取那份索引（相对路径从顶层起算，`:426` 的 `private_index` 拷一份再读，不写用户的索引），`:354` 判 `raw_entries(top, "--cached", "HEAD")`，即这份索引相对 HEAD 新增或改过的 blob，没有 HEAD 时（`:353` `if head_born(top):`）判整个索引。

| 提交方式 | 读的是哪一份内容 | 依据 | 自证有没有这一格 | 判 |
|---|---|---|---|---|
| `git commit`（暂存区） | 暂存区那份索引相对 HEAD 的新增与改动 | `scripts/gate/large-file-check.py:354` | 有，`large-file-git-hook.py:407` | 成立 |
| `git commit -a` | git 在起钩子前把工作区里已跟踪文件的改动放进索引，读的是放进去之后的索引 | `large-file-git-hook.py` 文件头 `:15`、`:16` | 有，`:426` | 成立 |
| `git commit <路径>` | git 给钩子的是临时索引（`next-index-*`），判的是它，不是主索引 | 同上 | 有，`:428` | 成立 |
| `git commit --amend` | 改写后的那份索引，相对被替换的那一笔 HEAD；HEAD 里已有的超限 blob 不重判，留给 pre-push | `scripts/gate/large-file-check.py:354` | 没有（自证二十八格里没有 amend） | 成立，无自证；第一轮判过「大 blob 已在 HEAD 里，由 pre-push 判」，不重复；新加的超限 blob 的 amend 由探针 `commit-amend-adds-big` 量 |
| `git merge` 生成合并提交 | pre-merge-commit：`:269` `return judge_index("merge", arguments, "pre-merge-commit-judge-skipped")`，合并结果相对第一个父提交 | `:255` 起 `judge_index` | 有，`:474`（带超限的被拒）与小文件通过 | 成立 |
| `git merge --no-commit` 之后 `git commit` | pre-commit，索引里带着被合并那边新来的 blob | 同 pre-commit | 有，`:479` | 成立 |
| `git merge --squash` 之后 `git commit` | 同上 | 同上 | 没有 | 成立（推），探针 `commit-after-merge-squash` 量 |
| 私有索引 `GIT_INDEX_FILE=… git commit` | 钩子环境里的 `GIT_INDEX_FILE`，相对路径从顶层起算 | `scripts/gate/large-file-check.py:419` | 没有 | 成立（推），探针 `commit-private-relative-index` 量 |
| 从子目录起的 `git commit` | 钩子的当前目录是顶层（git 先 chdir 到顶层再起钩子；相对 `core.hooksPath` 靠这个） | `lib_git_hooks.py:51` `resolved = os.path.realpath(value if os.path.isabs(value) else os.path.join(top, value))` | 没有 | 成立（推），探针 `commit-from-subdirectory-relative-hooks-path` 量 |
| 冲突后 `git commit` 收尾（merge） | 同 pre-commit | `large-file-git-hook.py:31` | 没有 | 成立（推），探针 `merge-conflict-then-git-commit` |
| `git merge --continue` | git 内部直接调 `cmd_commit`，起 pre-commit | git 源码（我这台没有源码，凭记忆，没量） | 没有 | 推不出，探针 `merge-conflict-then-merge-continue` |
| `git cherry-pick --continue` / `git revert --continue` | sequencer 起的 `git commit`（我记得不带 `-n`，没量） | 文件头 `:31` 写走 pre-commit | 没有 | 推不出，探针 `cherry-pick-conflict-then-continue`、`revert-conflict-then-continue` |
| `git rebase --continue`（含 `rebase -i` 停在 `edit` 后直接 `--continue`） | sequencer 自己提交；它起 `git commit` 时带不带 `-n`（跳过 pre-commit），我读不出 | 文件头 `:31` 说「`--continue` 收尾的那一笔走 pre-commit」 | 没有 | 推不出，打中；探针 `rebase-conflict-then-continue`、`rebase-i-edit-then-continue-directly`、`rebase-i-edit-then-amend` |
| `git cherry-pick`、`git revert`、`git am`、`git rebase` 无冲突 | 没有钩子；文件头 `:29` 写明不起 pre-commit，`git am` 的 pre-applypatch 没接 | `:29`、`:30` | 没有 | 钩子拦不了，由 pre-push 兜；探针 `cherry-pick-clean-lands` 与 `-then-push-refused`、`am-clean-lands` 与 `-then-push-refused` 量「落地、推送拦」 |

文件头 `:31` 那句「它们遇冲突、用 git commit 或 --continue 收尾的那一笔走 pre-commit」整句没有一格自证（`selftest_cells` 的二十八格名都在 `:401` 到 `:583` 之间，没有 cherry-pick、revert、rebase、am、amend、squash 的格）。`rebase --continue` 这一支如果真的不起 pre-commit，在冲突里自己把文件写成超限的那一笔只有 pre-push 兜；推送前不会进远端，但会进本地历史，之后要重做提交。
推翻现象：探针 `merge-conflict-then-merge-continue`、`cherry-pick-conflict-then-continue`、`revert-conflict-then-continue`、`rebase-conflict-then-continue`、`rebase-i-edit-then-continue-directly` 的 `refused` 都是 `true`、`landed_commit` 都是 `false`。其中哪一格是 `landed_commit=true`，哪一格就是文件头 `:31` 写错了。

### push：每一行引用

`pre_push` 读标准输入 `<本地引用> <本地 sha> <远端引用> <远端 sha>`，`scripts/gate/large-file-git-hook.py:295` `local = list(dict.fromkeys(row[1] for row in rows if not is_deletion(row[1])))`，`:296` `if not local:` 直接放行（只删引用的推送）；远端 sha 去重后当「远端已有」（`:298`）；`:304` 起不带 `unknown-remote-not-judged` 时总加 `--judge-unknown`（`:305`）。`scripts/gate/large-file-check.py:404` 把 `--known` 里本地读得到的对象当已有，再加远端跟踪引用（点名的是网址或路径写法时 `:400` `address_only` 不取跟踪引用），`:412` `judge_revision_lists` 判 `<引用> --not <已有的>` 之间的 blob。

| 一行引用 | 判法读的内容 | 判 |
|---|---|---|
| 分支新增、快进、强推 | 本地 sha 相对（行里远端 sha、远端跟踪引用）还没有的提交里新增或改过的 blob | 成立（自证 `:502`、`:518` 等） |
| 删除引用（本地 sha 全 0） | 不判 | 成立（`:509`） |
| 轻量标签、带说明的标签 | `:382` 的 `rev-parse --verify -q <名>^{commit}` 把它剥成提交 | 成立（`:522`） |
| 标签或别的引用指着 blob、树 | 同一行 `^{commit}` 解不出，`existing_commits` 略过；`:401`、`:402` 的 `tips = existing_commits(top, options["refs"])`、`if not tips:` 返回 `checked 0`、`too_big` 空、`not_run` 空——退 0、没有 `⊘`，超限的 blob 随引用推上去 | 不成立（推），打中；探针 `push-tag-pointing-at-big-blob`、`push-tag-pointing-at-tree-with-big-blob` 应是 `verdict="differs-from-expected"`、`landed_remote=true` |
| 新引用、远端没取回过 | 判这些 sha 能到的全部历史，`JUDGE_SECONDS = 600`（`:89`）到点放行并写 `⊘` | 成立，但 600 秒没量过（主 agent 已知，攻方第 6 问） |
| 推到路径或网址写法的远端 | 只认行里的远端 sha，不取跟踪引用 | 成立（`:535` 起） |
| 远端 sha 本地没有这个对象 | `:404` 的 `existing_commits` 略过它，退回跟踪引用或全历史 | 成立（推） |
| 从子目录起的 `git push` | 同提交，钩子当前目录是顶层 | 成立（推），探针 `push-from-subdirectory-relative-hooks-path` 量 |

推翻现象（推送那一格）：探针里指向非提交对象的两个标签推送，任何一个 `refused=true` 且 `landed_remote=false`，则「不成立」改「成立」。

## 第 3 问：安装与核对

### 两台

- `scripts/gate/git-hooks-install.py:152` 起 `check_peer` 用 `bash -c` 经 `peer-host-lib.sh` 在第二台起 `python3 scripts/gate/git-hooks-install.py check <仓> --this-machine-only`（`:152` 的 `command = …`、`:154` 的 `subprocess.run(["bash", "-c", PEER_RUNNER, …`）。仓、`.git/config` 与钩子在共享根下，两台读到同一份，这一点读得出；第二台自己的 git 够不够新（`:129` 核 `rev-parse --path-format=absolute`，git 2.31 起才有）、python3 在不在，只有真连第二台才知道。正文写 `check --both-machines` 过了，我这里没连过第二台，工具员报告第 175 到 187 行也写「第二台那一支只在自证里用假的库证过」。推不出，待主 agent 跑（命令在下面第三条）。

### 链接工作树里 `core.hooksPath` 的相对路径怎么解

- 成立。git 的规矩是从钩子运行的目录起算，非裸仓里那是工作树顶层，链接工作树各取自己的顶层；`.claude/hooks/lib_git_hooks.py:51` `resolved = os.path.realpath(value if os.path.isabs(value) else os.path.join(top, value))` 与它一致，自证 `scripts/gate/large-file-git-hook.py:453` 的 `worktree-relative-hooks-path`（链接工作树里提交超限文件被拒）与 `git-hooks-install.py` 的 `linked-worktree-green` 各有一格。探针 `commit-from-subdirectory-relative-hooks-path` 与 `push-from-subdirectory-relative-hooks-path` 再量从子目录起的两种。

### 检出老提交的工作树——不成立

- 现成的三棵链接工作树都没有钩子目录（量的，`git worktree list` 与 `ls`）：
  - `~/.local/share/singlefs/work/doc-gate-commit-20261105/tree`（`59ce5a865`，游离）：`ls …/scripts/gate/git-hooks` 是 `No such file or directory`；
  - `~/.local/share/singlefs/work/m5-dev`（`8ed2ea484`）：同；
  - `~/.local/share/singlefs/worktree-backlog-m4`（`59ce5a865`，游离）：同。
  - `git config core.hooksPath` 在共用的 `.git/config` 里是相对路径 `scripts/gate/git-hooks`，落到各树自己那一棵里，那里没有这个目录，git 不报错、什么钩子都不起。这三棵树里 `git commit`、`git push` 现在没人判大文件，私有窗口闸也不调。
- 核对只写 ⊘、不红：`scripts/gate/git-hooks-install.py:144` 的警告是 `工作树 {path} 里没有 …，在那里提交与推送没人判`，`:162`、`:163` 打 `⊘` 之后 `check` 照退 0（`:170` 起只有 `problems` 非空才退 1）。
- 检出钩子 ㉒ 也不拦：`.claude/hooks/bash-command-detector.sh:3557` `if code != 0 or not top or not library.project_checkout(top):` 在这棵树不自带钩子目录时直接不判；`lib_git_hooks.py:29` 的 `project_checkout` 就是看 `pre-commit` 文件在不在。
- 主工作树自己落到同样局面的路径（推，探针量）：`git checkout <没有钩子目录的老提交>`、`git bisect`、`git rebase` 停在老提交上；还有此刻的局面——钩子文件在暂存区、不在 HEAD（`git ls-tree HEAD scripts/gate/git-hooks` 零行），这时 `git reset --hard`、`git stash -u` 会把钩子目录整个删掉，`core.hooksPath` 指着空目录。这些局面里 `check` 对主工作树是红（`lib_git_hooks.py:56` 起逐份核「钩子 … 不在」），㉒ 却静默（同 `:3557`）。
- 探针：`old-commit-worktree-big-commit`、`old-commit-worktree-push`、`main-tree-at-old-commit-big-commit`、`hooks-staged-then-reset-hard-big-commit` 四格我预期都是 `landed_commit=true`（推送那格 `landed_remote=true`），`verdict="differs-from-expected"`；`detector-in-old-commit-worktree-commit`、`detector-in-main-tree-at-old-commit` 预期 `exit=0`。对照格 `current-commit-worktree-big-commit-control` 应是 `refused`。
- 推翻现象：上面四格里任何一格 `refused=true` 且没落地。
- 出路（推，不替主 agent定）：装绝对路径（`lib_git_hooks.py:37` 的 `accepted_directories` 已把主工作树的钩子目录算装对，老提交的链接工作树就会读到主工作树那一份）；或让 `check` 对没有钩子目录的链接工作树退 1。

### 格 git-hooks-installed 与三份样本判的是不是同一件事——不成立

- 格是转调：`.claude/gate.d/code-tooling.sh:5807` `GIT_HOOKS_INSTALL_BREAK="$break_item" python3 "$installer" check "$ROOT"`，所以格与 `check` 判的是同一份判法，这一层成立。
- 但格的描述与它实际判的不是同一件事：`.claude/gate.d/code-tooling.sh:41` 写 `core.hooksPath 指到 scripts/gate/git-hooks/、pre-commit 与 pre-push 在、有执行位、进仓的模式是 100755`，`:5796` 写 `两个钩子文件在而且有执行位`，而 `check` 核三份、加 pre-merge-commit（`scripts/gate/git-hooks-install.py:62` `REQUIRED_HOOK_NAMES = (*hooks.HOOK_NAMES, MERGE_HOOK_NAME)`）；成功句与三份样本的 `expect` 写的是三份（`git-hooks-installed-green/expect` 第 2 行 `pre-commit、pre-push 与 pre-merge-commit 在`）。`grep -c pre-merge-commit .claude/gate.d/code-tooling.sh` 是 0，`.claude/agent-common.md` 第 102 行同（量的）。
- 三份样本只覆盖 `check` 八种红里的两种：没设（`git-hooks-installed-unset-red`）、指错（`-wrong-path-red`）。钩子缺、没有执行位、pre-merge-commit 缺或没有执行位、进仓模式不是 `100755`、这台 git 太老，都只有 `git-hooks-install.py --selftest` 的格管（`.claude/gate.d/code-tooling.sh:2349` 那一行登记了它）。弄坏开关只有一个 `git-hooks-path-unchecked`。
- 格不带 `--both-machines`（`:5807` 无这个参数）：门禁判这台的配置，第二台读同一份 `.git/config` 但各自的 git 版本与执行位没人在门禁里核。
- 三样（格、`check`、㉒）不是同一件事：㉒ 用 `lib_git_hooks.py` 的 `HOOK_NAMES = ("pre-commit", "pre-push")`（`:16`）核，不看 pre-merge-commit（主 agent 已知）；而且 ㉒ 以 `pre-commit` 在不在认「这是本仓」（`:29`），目录里把 `pre-commit` 删了（别的钩子在）：`check` 红（`:56` 起）、㉒ 静默。探针 `pre-commit-file-missing-check-vs-detector` 预期 `check_rc=1`、`detector_exit=0`。

## 第 4 问：私有窗口闸、`SINGLEFS_COMMIT_OUTSIDE_WINDOW=1` 与 ㉒ 拒 `--no-verify` 放在一起

### 三样放在一起有没有绕开大文件判法——不成立（没有绕开）

- 环境变量只放行窗口：私有闸 `.git/hooks/pre-commit` 第 8 行 `if [[ "${SINGLEFS_COMMIT_OUTSIDE_WINDOW:-}" == 1 ]]; then exit 0; fi`（量的）；`scripts/gate/large-file-git-hook.py:252` 退 0 之后 `:257` 起 `judge_index` 照判大文件，退非 0 才直接返回（`code != 0`），自证 `:443` `private-hook-then-judge` 与 `:491` 的 `merge-private-hook-window` 各有一格；探针 `window-open-by-variable-big-commit` 再量。
- 私有闸不在时（新克隆、第二份克隆）：`:250` 条件不满足直接当 0，大文件照判，不拒也不绕开。
- ㉒ 拦 `--no-verify`：`.claude/hooks/bash-command-detector.sh:3411` 是 `HOOKS_BYPASS_WINDOW_VARIABLE = "SINGLEFS_COMMIT_OUTSIDE_WINDOW"`，只用在拒绝说明里指路；env 前缀写法不被拦，`SINGLEFS_COMMIT_OUTSIDE_WINDOW=1 git commit --no-verify` 照拒（探针 `bypass-outside-window-prefix-with-no-verify`）。

### 主 agent 日常的提交写法，有没有被拒的

主 agent 记着的写法：`.claude/rules/session-wrapup.md:40` 「然后不带路径地 `git commit`」；`.claude/kb/tooling.md:210` 起一张表——`stage-mine.py` 暂存、`git commit` 不带路径、「不要 `git commit -- <路径>`」、「在 worktree 上提交」一行（`:214`）。近来的提交（`git reflog`，量的）都是 `commit:` 带说明，没有 `--no-verify`；仓里写过 `--no-verify` 的只有更早的记录（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第 50、51 行、`records/2026-10-13-复用键与全仓改名.md` 第 1070 行），那是窗口外提交的旧办法，现在 ㉒ 拒、改用环境变量。
- 日常写法被拒的：推不出。读码上 `git_hook_bypass_reasons`（`:3493` 起）只认长选项全名或唯一前缀的 `--no-verify`、`git commit` 的 `-n`、`-c core.hooksPath=…` 这几种，`-m` 的值与 `--` 之后跳过。要喂的写法我写进了探针 `detector` 一组：二十一条该放的（add 加 commit、heredoc 说明里写着 `xargs git commit -n` 与 `git push --no-verify` 字样、`-F` 说明文件、`-a`、`--amend --no-edit`、`-m` 里写着 `-n` 与 `--no-verify` 字样、`--` 之后点路径、`-s --allow-empty`、环境变量前缀与 `export`、`cd … && git commit`、`git -C … commit`、`git push` 各种常用选项、`git log -n 5`、`git cherry-pick -n`、`git merge -n`、worktree 流程里的 `git update-ref` 加 `git read-tree`），应都是 `verdict="as-expected"`（放行）；十条该拒的（`--no-verify`、`-n`、`-nm`、`--no-veri`、`git push --no-verify`、`-c core.hooksPath=`、`--config-env`、`GIT_CONFIG_COUNT` 三件套、`git config core.hooksPath`、环境变量前缀加 `--no-verify`）应拒。`known-merge-no-verify-not-guarded` 是主 agent 已知的放过，期望放行。
- 绕开大文件判法的日常写法：有一条，不成立。`.claude/kb/tooling.md:214` 的 worktree 提交流程是「在 worktree（HEAD 上）重放改动、跑门禁并提交 → 主仓里 `git update-ref refs/heads/master <新提交> <旧 HEAD>` 再 `git read-tree`」。这一笔提交由 worktree 里的 pre-commit 判，而 worktree 里有没有钩子目录取决于它从哪个提交检出：现在 HEAD 里没有 `scripts/gate/git-hooks/`（量的），照这一行新开的 worktree 没有钩子，提交与之后的 `update-ref` 都没人判大文件；`update-ref` 本身不起任何钩子。钩子目录进了 HEAD 之后新开的 worktree 才有。推翻现象：此刻在 HEAD 上 `git worktree add` 一棵、在里面提交一个超限文件，被拒。
- 窗口闸只管 `git commit` 与 `git merge` 生成的合并提交：cherry-pick、revert、rebase、am 起不了 pre-commit，窗口外照样生成提交（旧就如此，不是这一轮带来的；探针 `window-closed-cherry-pick-not-covered` 预期 `commit-made`）。

## 要主 agent 跑的命令

一条一项；都是我没起过的。第一条是模型整份复跑（外面包的那一层只删 `-run` 结尾的运行目录，再调同一个 `tools/rerun.sh`）。

1. 探针与派发闸 ④ 的十二句（第 1 问第 5 条、第 2 到 4 问）

   ```
   task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-sonnet-run /home/fy5090/code/singlefs/research/prompts/fg-timeout-large-file-governance-r3-sonnet-model
   ```
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（submit 的 `--cwd`）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`（探针内部出错写 `PROBE name=setup-error-…` 与 `probe_hooks.py 退 …`，不改任务退出码，要读输出）；`<运行目录>/out/probe.out` 开头十四行 `PROBE name=frozen-hash … match=yes`（有 `match=no` 的那份被判文件换过了，整份读数作废），末行 `PROBE name=done emitted=<行数>`；`grep -c 'verdict="differs-from-expected"' <运行目录>/out/probe.out`；`<运行目录>/out/dispatch-scope.out` 末行 `SCOPE name=done emitted=12`，十二行里没有 `verdict="false-refusal"` 算第 5 条过。
   - 这条命令里没有 `cargo`、没有 `--item`，是起真 git 与项目脚本的探针，范围声明照 `.claude/hooks/run-scope-guard.sh` 的要求补（被拒了照拒绝说明加）。
   - 读完 `bash scripts/hosts/work-directory-ownership.sh sweep` 或点名删运行目录。

   复跑后每格应有的结局（推的，没量过；「预期」列是我读码后的预测，「对设计」列是格里 `expect` 字段承诺的）：

   | 格 | 我预测的结局 | 打中就是 `differs-from-expected` |
   |---|---|---|
   | `commit-amend-adds-big`、`commit-from-subdirectory-relative-hooks-path`、`commit-private-relative-index`、`commit-after-merge-squash`、`merge-conflict-then-git-commit`、`merge-conflict-then-merge-continue`、`cherry-pick-conflict-then-continue`、`revert-conflict-then-continue`、`rebase-i-edit-then-amend` | `as-expected`（refused） | 其中任何一格是 `differs`：钩子读的内容与文件头 `:15` 到 `:16` 不一致 |
   | `rebase-conflict-then-continue`、`rebase-i-edit-then-continue-directly` | 不知道（我读不出 sequencer 的 `git commit` 带不带 `-n`） | `landed_commit=true` 即文件头 `:31` 写错 |
   | `cherry-pick-clean-lands`、`am-clean-lands`、`cherry-pick-clean-then-push-refused`、`am-clean-then-push-refused` | `as-expected`（前两格落地、后两格 pre-push 拦） | 后两格 `landed_remote=true` 即 pre-push 兜不住 |
   | `push-tag-pointing-at-big-blob`、`push-tag-pointing-at-tree-with-big-blob` | `differs-from-expected`、`landed_remote=true` | 打中，见第 2 问 |
   | `push-from-subdirectory-relative-hooks-path` | `as-expected` | |
   | `check-lists-old-worktree` | `rc=0`、`lists_old_worktree=true` | |
   | `old-commit-worktree-big-commit`、`old-commit-worktree-push`、`main-tree-at-old-commit-big-commit`、`hooks-staged-then-reset-hard-big-commit` | `differs-from-expected`（落地） | 打中，见第 3 问 |
   | `detector-in-old-commit-worktree-commit`、`detector-in-old-commit-worktree-push`、`detector-in-main-tree-at-old-commit` | `exit=0`（㉒ 静默） | 打中 |
   | `current-commit-worktree-big-commit-control` | `as-expected`（refused） | |
   | `pre-commit-file-missing-check-vs-detector` | `check_rc=1`、`detector_exit=0` | 打中，见第 3 问 |
   | `detector` 一组 `daily-*`（21 格）与 `bypass-*`（10 格） | 全 `as-expected` | 哪格 `differs`，哪格就是日常写法被拒或绕开写法放过 |
   | `known-merge-no-verify-not-guarded` | `as-expected`（放行） | |
   | `window-closed-small-commit`、`window-open-by-variable-small-commit`、`window-closed-amend`、`window-closed-merge-commit`、`window-open-by-variable-big-commit` | `as-expected` | |
   | `window-closed-cherry-pick-not-covered` | `as-expected`（生成了提交，窗口闸管不到） | |
   | `leftover-index-copies` | `count=0` | 非 0 说明有判法起停没清索引拷贝 |

2. 门禁两格：git-hooks-installed 与 shared-data-root-literals

   ```
   SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 .claude/gate.d/code-tooling.sh 点名的格 git-hooks-installed; 范围=1 格; 为什么=核正推腿第 3 问：格转调 check，样本三份只覆盖其中两种红' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-tooling.sh --check git-hooks-installed
   SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 .claude/gate.d/code-tooling.sh 点名的格 shared-data-root-literals; 范围=1 格; 为什么=核第二轮判决改法第 4 条：排除表失效的一行删了、检出钩子经 ~ 取工作根的那一行不在了' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-tooling.sh --check shared-data-root-literals
   ```
   两行各是独立的一条。看 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；第一条日志里要有 `✓ /home/fy5090/code/singlefs 里进仓的 git 钩子装好了`；第二条第二轮是 `command-failed exit=1` 两处红，现在应是绿。

3. 第二台

   ```
   task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 scripts/gate/git-hooks-install.py check --both-machines
   ```
   在仓根起。看 `type=succeeded`、`exit=0` 与日志里 `第二台同样`；红的话 `✗ 第二台上的 check 退 <码>：…` 点名第二台上哪一条不对。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1-条1 钩子进仓、安装、窗口闸、不靠 `--no-verify` | 成立 | 做到；但钩子目录还只在暂存区 |
| 1-条2 ㉒ 收窄 | 成立（字面有两处不同） | 内容判断全删、多了「没装上」的拒 |
| 1-条3 写范围表 | 推不出 | 腿名收窄、八类写者锁做到；轮名那一位没收，判决第 35 行那一半还在 |
| 1-条4 名单、读表、索引拷贝、排除表、文件头 | 成立 | 名单文法是正则两份抄写；信号路径没处理（攻方第 7 问） |
| 1-条5 派发闸 ④ | 推不出 | 这批没碰，另一路会话的改动已在文件里，待喂十二句 |
| 1-条6 第一轮判决两处字面 | 不成立 | `r1-main-verification.md` 第 55、56 行原样没改 |
| 2 暂存区、`-a`、点名路径、合并提交、`--no-commit` 之后 | 成立 | 都读 git 放进钩子的那份索引，自证有格 |
| 2 `--amend`、`--squash`、私有索引、子目录 | 成立（推） | 没有自证格，探针量 |
| 2 冲突后 `--continue`（merge、cherry-pick、revert） | 推不出 | 文件头 `:31` 无格；探针量 |
| 2 冲突后 `rebase --continue` | 推不出 | 打中；sequencer 带不带 `-n` 读不出 |
| 2 push 指着非提交对象的引用 | 不成立（推） | `^{commit}` 解不出被略过，退 0 且没有 `⊘` |
| 2 push 其余各行引用 | 成立 | 删除、标签、新引用、路径远端自证有格 |
| 3 两台 | 推不出 | 没连过第二台，待跑 `check --both-machines` |
| 3 相对 `core.hooksPath` 解法与链接工作树 | 成立 | 从各自顶层起算，自证有格 |
| 3 检出老提交的工作树 | 不成立 | 三棵现成的树都没有钩子；`check` 只写 ⊘，㉒ 静默 |
| 3 格与三份样本是不是同一件事 | 不成立 | 描述写两个钩子、判的是三个；样本只覆盖八种红里的两种；㉒ 与 `check` 对「pre-commit 缺」判得不同 |
| 4 窗口闸、环境变量与 ㉒ 放一起有没有绕开大文件判法 | 不成立（没有绕开） | 环境变量只放行窗口，大文件照判 |
| 4 日常提交写法有没有被 ㉒ 拒 | 推不出 | 读码上没有；二十一条要喂 |
| 4 日常写法里有没有绕开大文件判法的 | 不成立（有一条绕开） | worktree 提交流程在没有钩子目录的 worktree 里绕开 |

## 没做什么

- 没替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数没复跑（命令在上面「要主 agent 跑的命令」一节）。
- 没判不是我这条腿的格：攻方腿的第 5 到 10 问（`git merge --no-verify`、`GIT_DIR`、子模块、误拒、索引拷贝信号、⑫ 误拒、共用读表、自证串行）只在上面提到处顺带指了行，没有独立判。
- 没跑任何探针、自证、门禁、`git` 的写操作；`probe_hooks.py` 与 `tools/rerun.sh` 没起过，只做了 `ast.parse`、`bash -n`、未定义名字扫描。第一次真跑如果装置出错，以 `PROBE name=setup-error-…` 为准。
- 没读攻方腿这一轮的提示与产出（禁读清单三条一条没碰）。读的是第二轮攻方留在 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-main/` 与 `fg-timeout-large-file-governance-r2-main-opus-run/` 的东西（`dispatch_scope_model.py` 与 `dispatch-scope.out`），不在禁读清单里。
- 草稿目录 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-sonnet` 没有写任何文件（`du -sh` 是 12K，开工认领建的目录自己的开销）；没建仓副本、没留编译目录。
- 私有值：`origin` 的地址在我读的输出里带主机名，报告里没写；没写第二台的主机名与路径。
