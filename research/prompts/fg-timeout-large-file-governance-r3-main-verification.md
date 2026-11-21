# fg-timeout-large-file-governance-r3：判决（2026-11-19）

正文 `research/prompts/_fg-timeout-large-file-governance-r3-body.md`，开工快照 `research/prompts/fg-timeout-large-file-governance-r3-snapshot/`。这是这一题的第三轮，也是最后一轮；前两轮的判决是 `research/prompts/fg-timeout-large-file-governance-r1-main-verification.md` 与 `research/prompts/fg-timeout-large-file-governance-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推腿（Sonnet，第 1 到 4 问） | `research/prompts/fg-timeout-large-file-governance-r3-sonnet-output.md` | 第 1 问：第 1、4 条做到，第 2 条字面两处不同，第 3 条只做到一半，第 5 条待跑，第 6 条没做；第 2 问三处打中；第 3 问四处打中；第 4 问一处打中 |
| 攻方腿（Opus，第 5 到 10 问） | `research/prompts/fg-timeout-large-file-governance-r3-opus-output.md` | 第 5 到 9 问成立；第 10 问单跑一格推不出、开关轮因次序判绿不成立 |

核查：

- 开工快照：`python3 scripts/agents/snapshot-tree.py fg-timeout-large-file-governance-r3 --out <草稿目录>/fg-timeout-large-file-governance-r3-tree --repository <仓根>` 经任务服务交，结局 `type=command-failed exit=1`，末行「清单点名 16 份，倒推出 15 份、sha256 对上 15 份；倒推不出 1 份、对不上 0 份」。
  - 倒推不出的是 `.claude/agent-common.md`。腿跑着的时候，主 agent 改了它第 41 行（汇报里写会话编号那一句），别的会话也改过它；把主 agent 那一处反着做一遍，sha256 前缀是 27a8c0888ac3，不是快照的 d077b5ae41a0。
  - 两条腿都没有引这份文件的行，这一轮判它的第 102 行按主工作区此刻那一行判。
- 引文：`python3 scripts/agents/cite-check.py <两份报告> --root <倒推出的树> --background research/prompts/_fg-timeout-large-file-governance-r3-background.md` 末行「✓ 核了 37 处引文、44 处裸行号，对上 32 处，没判 49 处」。
  - 没判的里有 6 处引的文件不在快照里（`.claude/rules/session-wrapup.md`、`.claude/hooks/lib_writer_lock.py`、`.claude/rules/three-way-inference.md`、`.claude/rules/test-script-discipline.md`）。
  - 换成 `--root .` 再跑一次，末行「✓ 核了 37 处引文、44 处裸行号，对上 37 处」。
- 两份模型由主 agent 拷到 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r3-main/` 再交，下面「二、判决」只采这两趟的读数：
  - 正推腿模型：头两趟同时写同一个运行目录，读数作废。另开运行目录 `fg-timeout-large-file-governance-r3-main-sonnet-clean-run` 单独一趟，结局 `type=succeeded exit=0`，`probe.out` 末行「PROBE name=done emitted=81」，53 格 `as-expected`、8 格 `differs-from-expected`。`frozen-hash` 只有 `.claude/agent-common.md` 是 `match="no"`，原因同上。派发闸十二句的模型 `dispatch_scope_model.py` 读常量时 `NameError: name 'SENTENCE_SPLIT' is not defined`，那一问核不动。
  - 攻方腿模型：运行目录 `fg-timeout-large-file-governance-r3-main-opus-run`，结局 `type=succeeded exit=0`，`probe.out` 末行「PROBE name=done emitted=84」，`lints.out` 末行「LINTS name=done emitted=10」。`frozen-hash` 只有 `.claude/agent-common.md` 是 `match=no`。
- 腿报告里写「推的」、两份探针都没量到的，下面判成核不动，不当打中。

## 二、判决

两条腿都打中了。这是第三轮，坐实的照「三、改法」改，改完不再开第四轮。

坐实：

- 钩子目录用的是相对路径，下面几种情形 git 钩子一个都不起，大文件静默进提交、推出去：
  - 攻方 `work-tree-option-commit`、`git-dir-option-push-from-outside`、`git-dir-variable-push-from-subdirectory`、`old-commit-linked-worktree-commit-and-push`、`sparse-checkout-commit-and-push`，都是 `hooks_invoked=none verdict=missed-silently`。
  - 正推 `old-commit-worktree-big-commit`、`old-commit-worktree-push`、`main-tree-at-old-commit-big-commit`、`hooks-staged-then-reset-hard-big-commit`，都是 `verdict="differs-from-expected"`。
  - `.claude/kb/tooling.md` 第 214 行「在 worktree 上提交」那一行，在不带钩子目录的链接工作树里提交，就是同一种情形。
- 推指向 blob 或树的标签不判：攻方 `tag-to-blob-push`、`tag-to-tree-push` 是 `hooks_invoked=pre-push verdict=missed-silently`，正推 `push-tag-pointing-at-big-blob`、`push-tag-pointing-at-tree-with-big-blob` 是 `landed_remote=true`。代码在 `scripts/gate/large-file-check.py` 第 382 行 `existing_commits` 只收能解成提交的对象。
- 自证认法：任意祖先进程命令行里带 `--selftest` 就认自证门限与弄坏开关（`scripts/gate/large-file-git-hook.py` 第 108 行）。攻方 `selftest-word-in-message-break-switch` 是 `hooks_invoked=pre-commit verdict=missed-silently`。
- 改配置段绕开钩子：攻方 `config-remove-section-then-commit`、`config-rename-section-then-commit` 是 `hooks_invoked=none verdict=missed-silently`，㉒ 不认 `--remove-section core`、`--rename-section core`。
- 提交只落本地、推被拒的几种：
  - 攻方 `merge-no-verify-then-push`、`am-then-push`、`commit-tree-update-ref-then-push`、`stash-then-push-stash` 都是 `verdict=commit-lands-push-refused`。
  - 正推 `rebase-conflict-then-continue`、`rebase-i-edit-then-continue-directly` 是 `landed_commit=true landed_remote=false`。
  - 其中 `git merge --no-verify` ㉒ 不拦；`git am` 有 pre-applypatch 可接没接；rebase 逐笔重放与 `--continue` 起 prepare-commit-msg、不起 pre-commit。
- ㉒ 误拒把 `core.hooksPath` 设成正确值：攻方 `detector-set-hooks-path-to-the-right-value`、`detector-command-line-hooks-path-right-value` 都是 `detector_exit=2 detector_reason=bypass`。
- 写范围：
  - 攻方 `scope-attack-earlier-round-report`、`scope-attack-earlier-round-model`、`scope-defense-writes-forward-report-of-another-round`、`scope-forward-writes-defense-report-of-another-round` 都是 `refused=no`：腿能写别的轮的报告与模型。
  - `scope-attack-own-second-sample-report`、`scope-attack-own-second-sample-model` 是 `refused=yes by_write_scope=yes`：第二次同立场腿（`-s2-`）照 `.claude/rules/three-way-inference.md` 要写的报告与模型目录被拒。
  - `lock-owner-only-directory-of-another-session` 是 `refused=no`：只有 `.owner` 没有锁的别的会话的目录照写。
  - `lock-other-draft-unclaimed-receipt-missing` 是 `refused=no`，`.claude/hooks/lib_writer_lock.py` 文件头第 19 行要求这种情形「记一条检出」，⑫ 没记。
  - `lock-own-draft-holder-control` 是 `claims_created=own-draft/.writer-claim-…`，而 `.claude/hooks/lib_writer_lock.py` 文件头第 7 行写 ⑫「不建锁、不建认领」。
- 共用读表：
  - `lints.out` 里 `append-with-plus-equals`、`reassigned-later`、`brace-expansion`、`variable-element`、`forbid-entries` 五种表写法 `fail_open` 不空，`backslash-continuation` 是 `library_words=unreadable guard_judges=no`：两边一起读错，或一起不判。
  - `command-allow-after-deny`、`command-cap-lints-allow` 是 `guard_passes=yes`：命令里先 `-D` 再 `-A` 同一个 lint、或加 `--cap-lints allow`，闸照放。
  - `lints-switch-set-project-code-guard-full-clippy` 是 `guard_exit=2 missing=-D`，`lints-switch-set-detector-outlet` 是 `outlet="-D warnings -D -D -D -D -D -D -D"`：`LIB_CODE_DISCIPLINE_LINTS_BREAK` 平常设了，两道钩子都照认。
- 两份新脚本的自证不能单跑一格：攻方 `selftest-inner-with-cell-name-large-file-git-hook.py` 是 `cells_run=28 asked_for=commit-small-passes`，`selftest-inner-with-cell-name-git-hooks-install.py` 是 `cells_run=13`。`.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」要求单跑一格与整份跑判定相同。
- git-hooks-installed 格：
  - `.claude/gate.d/code-tooling.sh` 第 41、5796 行写两个钩子，`check` 核三个。
  - 三份样本只罩没设、指错两种红。
  - `.claude/hooks/lib_git_hooks.py` 第 16 行 `HOOK_NAMES` 只有 pre-commit、pre-push，㉒ 不核 pre-merge-commit。
- 安装脚本自证的失败句把子进程输出的换行原样拼进来（`scripts/gate/git-hooks-install.py` 第 250 行）。门禁 code-tooling 的 break-switch-replay 格复跑这份时，`install-writes-nothing` 那一趟多出一行没被点名的失败行，结局 `type=command-failed exit=1`。
- 第二轮判决「三、改法」第 6 条没做：第一轮判决第 55 行仍写「第 16 行」，第 56 行没写 `float()` 溢出。

不算打中：

- `commit-tree` 加 `update-ref`、`git stash`：git 在这两种写法上没有钩子可接。由 pre-push 兜住，推出去之前拒并给重做命令，攻方读数是 `landed_remote=no`。
- 远端写成路径、远端已有超限 blob、又被别处推进过时 pre-push 多判一次：那一格判的是宁可多判，与 `scripts/gate/large-file-check.py` 文件头一致。判全部历史的用时是攻方 `judge-unknown-full-history-elapsed` 的 `seconds=2.0 checked_commits=427`。
- 不采攻方 `path-python3-stub-commit-and-push`（PATH 上放假 python3）：这是有意替换解释器的写法，与正文「主 agent 已知、没改的」里用户自己手敲 `--no-verify`、`GIT_CONFIG_GLOBAL` 指到别处是同一类，不在这道闸的射程里。

核不动：

- 攻方第 7 问「as-is 提交在钩子运行期间放开索引锁、别的会话 `git add` 进来的内容不经判就进提交」：两份探针都没造这一格。
- 攻方第 7 问「钩子收到 TERM、HUP 时索引拷贝目录留下」：`hook-sigterm-mid-judge`、`hook-sigint-mid-judge`、`hook-sighup-mid-judge` 都是 `judge_seen=no`，探针没赶上判法在跑的那一刻，读数不能用。
- 第二轮判决第 5 条（派发闸 ④ 的五格误拒）这一轮的模型崩了，见「一、腿与核查」。mainagent(A)（它管 `.claude/hooks/runner-dispatch-guard.sh` 与全量提交门禁）把第二轮攻方的 `dispatch_scope_model.py` 拷进副本，只在载入常量那一处多取一段 `VERDICT_MENTION = ` 到 `SCOPE_PATH_START = ` 的常量，喂此刻那一份闸（sha256 前缀 7159f8eb），结局 `type=succeeded exit=0`、末行 `SCOPE name=done emitted=12`。主 agent 在那一趟日志里逐行核过：`sweep-input-lists-what-main-agent-updated`、`prior-art-says-who-writes-kb`、`attack-leg-told-where-verdict-goes`、`designer-told-what-runner-writes-later`、`triage-told-who-adds-assertions` 五格都是 `outside=[]` `verdict="correct"`。这一条已关上，不用改。

## 三、改法

都是第三轮的修补，没有被再攻过，标「零轮」。改的都是钩子、门禁与脚本，不是产品代码，不另做 A/B；改完复跑两份探针当 A/B，拿这一轮的读数当改前读数。

派两个工具员，按文件分开：

1. git 一侧（`scripts/gate/large-file-git-hook.py`、`scripts/gate/git-hooks-install.py`、`scripts/gate/large-file-check.py`、`scripts/gate/git-hooks/`、`.claude/hooks/lib_git_hooks.py`、`.claude/gate.d/code-tooling.sh` 的 git-hooks-installed 格与样本、`.claude/agent-common.md` 第 102 行）：
   1. `core.hooksPath` 装成主工作树钩子目录的绝对路径：改了仓位置的、链接工作树、稀疏检出、检出老提交的树，都起同一份钩子。零轮。
   2. 加 prepare-commit-msg 一道判内容：它在 `--no-verify` 下照起，rebase、cherry-pick、revert 的逐笔重放与 `--continue` 也起它。另接 pre-applypatch 判 `git am`。各命令起不起它、退非 0 能不能挡住提交，先在临时仓里放桩钩子量。零轮。
   3. 推送判指向 blob、树、标签的引用：取这些对象能到的全部 blob，减去远端已有的。零轮。
   4. 自证与弄坏开关只认自证自己设的标记，不认祖先命令行里的 `--selftest` 字样。零轮。
   5. 两份脚本的自证每一格自己建状态，能用 `--selftest <格名>` 单跑一格，单跑与整份判定相同。零轮。
   6. 安装脚本的失败句引子进程输出时把换行折成一行。零轮。
   7. `HOOK_NAMES` 列全全部钩子；git-hooks-installed 格的说明照实写，样本补钩子缺、没有执行位、进仓模式不对三种红。零轮。
2. 检出钩子一侧（`.claude/hooks/bash-command-detector.sh` 的 ㉒ 与 ⑫、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/lib_writer_lock.py` 文件头、`.claude/hooks/lib_code_discipline_lints.py`、`.claude/hooks/project-code-guard.sh` 的 ⑤）：
   1. ㉒ 加拦 `git merge --no-verify`、`git config --remove-section core` 与 `--rename-section core`；把 `core.hooksPath` 设成安装脚本装的那个值放行；自证里造的桩钩子照 `HOOK_NAMES` 建。零轮。
   2. 写范围：两种腿各补 `-s2-` 的报告与模型目录；⑫ 对三方腿按派发回执里「报告：」那一行的轮名收窄到这一轮；只有 `.owner` 的别的会话的目录拒；回执读不到的写记一条检出；`lib_writer_lock.py` 文件头照实写 ⑫ 会建认领。零轮。
   3. 共用读表只认 `-D clippy::<名>` 一行一条的写法，别的写法判读不成；读不成时 ⑤ 拒 clippy 并给出路，㉓ 记检出。命令里在 deny 之后又 `-A`、`-W`、`--allow`、`--warn` 表里的 lint，或带 `--cap-lints`，⑤ 拒。`LIB_CODE_DISCIPLINE_LINTS_BREAK` 平常设了退 2，进 `project-code-guard.sh` 的 `SELFTEST_ONLY_VARIABLES`。零轮。
3. 第一轮判决那两处字面，主 agent 照实改。

改后复跑（换上之后由主 agent 经任务服务跑出）：

- 换上：git 一侧 `lib_git_hooks.py`、三份脚本与三份钩子、`code-tooling.sh` 一格经 `scripts/gate/hook-swap.py` 换上，新钩子 prepare-commit-msg、pre-applypatch 进仓（100755）；检出钩子一侧 4 份与 `project-code-guard.sh` 同样换上，各在主工作区回读一致、喂法都过。`git-hooks-install.py install` 结局行「core.hooksPath 是主工作树钩子目录的绝对路径 /home/fy5090/code/singlefs/scripts/gate/git-hooks，pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push 在、有执行位、进仓的模式是 100755」，`check --both-machines` 末尾「第二台同样」。
- 量法：`measure-git-hook-invocations.py` 结局 `type=succeeded exit=0`；prepare-commit-msg 在 `commit --no-verify`、`merge --no-verify`、cherry-pick、revert、rebase 逐笔与 `--continue` 都起、退 1 挡得住；`git am` 起 pre-applypatch、退 1 挡得住，`am --no-verify` 跳过它（㉒ 因此加拦 `git am --no-verify`）。
- 自证与开关：安装脚本「28 格都对」「10 个开关逐个打开……各红在点名的格」；大文件钩子「53 格都对」「17 个开关逐个打开……各红在点名的格」；81 格逐格单跑「SINGLE_CELL_DONE total=81 ok=81」；两份脚本开关复跑「逐格相同 27 条」；检出钩子自证「查了 1543 种」、开关复跑「逐格相同 167 条」；`project-code-guard.sh` 自证「查了 1485 种」、开关复跑「逐格相同 149 条」；`code-tooling.sh` git-hooks-installed 格绿，样本档「254 个用例」都对，这一道六条开关复跑「判错的样本与文件头点名的逐条相等」。
- 两份探针（A/B，改前读数是「二、判决」那几行）：
  - 正推：`rebase-conflict-then-continue`、`rebase-i-edit-then-continue-directly`、`push-tag-pointing-at-big-blob`、`push-tag-pointing-at-tree-with-big-blob` 由 `differs-from-expected` 改成 `refused=true` 的 `as-expected`；`cherry-pick-clean-lands`、`am-clean-lands` 现在提交那一步被拒、`known-merge-no-verify-not-guarded` 现在被 ㉒ 拦，模型照旧设计预期「落地」，判成 `differs-from-expected`，属于改好。
  - 两份探针的沙盒都照第二轮装法把 `core.hooksPath` 写成相对路径（正推 `probe_hooks.py` 第 146 行、攻方 `probe.py` 第 189 行）：正推十六格日常写法被 ㉒ 以「core.hooksPath 是相对路径」拒、老提交工作树四格与攻方 `--work-tree`、`--git-dir` 三格照旧落地，测的是旧装法。新装法下这几样由大文件钩子自证 `work-tree-option-commit-refused`、`git-dir-option-push-from-outside-refused`、`git-dir-variable-push-from-subdirectory-refused`、`old-commit-linked-worktree-commit-refused`、`old-commit-linked-worktree-push-refused`、`sparse-checkout-commit-refused` 与安装脚本自证的老提交两格管，都在上面「53 格都对」「28 格都对」里。
  - 攻方写范围与读表那几格：模型喂的输入里没有派发回执、读表只取函数自算不起闸，测不到新判法；按轮收窄、`-s2-`、只有 `.owner` 的目录、读表读不成拒 clippy、deny 之后放行拒，由检出钩子与 `project-code-guard.sh` 自证的点名格与上面两条开关复跑管。

## 交用户的

- 这是最后一轮，「三、改法」各条都是零轮。改完由主 agent 复跑两份探针与各份自证、开关，不再开第四轮。

## 这一轮被判过的文件

- research/prompts/_fg-timeout-large-file-governance-r3-body.md：两条腿照它的十问作答
- scripts/gate/git-hooks/pre-commit：判了相对钩子目录下不起（坐实）
- scripts/gate/git-hooks/pre-push：判了相对钩子目录下不起（坐实）、推送非提交对象（坐实）
- scripts/gate/git-hooks/pre-merge-commit：判了合并提交判内容（成立）、`--no-verify` 下不起（坐实）
- scripts/gate/large-file-git-hook.py：判了祖先命令行认自证（坐实）、rebase 与 cherry-pick 的提交不经它（坐实）、自证不能单跑一格（坐实）
- scripts/gate/git-hooks-install.py：判了相对路径（坐实）、失败句多出一行失败行（坐实）、自证不能单跑一格（坐实）
- .claude/hooks/lib_git_hooks.py：判了 `HOOK_NAMES` 缺 pre-merge-commit（坐实）
- scripts/gate/large-file-check.py：判了 `existing_commits` 只收提交（坐实）、路径远端多判（不算打中）
- .claude/hooks/bash-command-detector.sh：判了 ㉒ 漏 `merge --no-verify` 与改配置段（坐实）、误拒设成正确值（坐实）；⑫ 写范围按轮收窄、`-s2-`、只有 `.owner` 的目录、回执读不到不记检出（坐实）
- .claude/hooks/lib_code_discipline_lints.py：判了六种表写法两边一起读错（坐实）与弄坏开关平常也认（坐实）
- .claude/hooks/project-code-guard.sh：判了 deny 之后 allow、`--cap-lints` 照放（坐实）
- .claude/hooks/agent-write-scope.tsv：判了三方腿的模式不分轮、不收 `-s2-`（坐实）
- .claude/hooks/runner-dispatch-guard.sh：判了 ④ 对八类派发提示（第二轮那五格误拒已不在，成立）
- .claude/gate.d/code-tooling.sh：判了 git-hooks-installed 格的说明与样本（坐实）
- .claude/agent-common.md：判了第 102 行 ㉒ 那一条（与代码一致，钩子集合改了要跟着改）

## 回看决策

不涉及决策：这一轮只判钩子、门禁与脚本，不推翻也不确立 `.claude/kb/decisions.md` 里任何一条。
