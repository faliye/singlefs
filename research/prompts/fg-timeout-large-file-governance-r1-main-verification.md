# fg-timeout-large-file-governance-r1：判决（2026-11-19）

正文 `research/prompts/_fg-timeout-large-file-governance-r1-body.md`，开工快照 `research/prompts/fg-timeout-large-file-governance-r1-snapshot/`。这是这一题的第一轮，没有前几轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推腿（Sonnet，第 1 到 4 问） | `research/prompts/fg-timeout-large-file-governance-r1-sonnet-output.md` | 问 1 成立，附三处小偏差；问 2 不成立，三处打中；问 3 三种模式成立，退出码一段不成立；问 4 第 70、96 行成立，第 97 行一处不精 |
| 攻方腿（Opus，第 5 到 11 问） | `research/prompts/fg-timeout-large-file-governance-r1-opus-output.md` | 第 5 问推不出；第 6、7、9、11 问成立；第 8 问四种出错不成立、两处窄的成立；第 10 问弱成立 |

核查：

- 开工快照：`sha256sum -c research/prompts/fg-timeout-large-file-governance-r1-snapshot/judged-sha256.txt` 四份都是 OK。腿交齐时被判文件没变过，所以没另起 `snapshot-tree.py` 倒推：主工作区就是开工那一刻的树。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/fg-timeout-large-file-governance-r1-sonnet-output.md research/prompts/fg-timeout-large-file-governance-r1-opus-output.md --root . --background research/prompts/_fg-timeout-large-file-governance-r1-background.md` 的末行是「✓ 核了 25 处引文、31 处裸行号，对上 25 处，没判 31 处（其中裸行号没有引文可比 31 处；逐处列在上面）」。
- 两条腿的探针都由主 agent 拷到草稿目录 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-main/` 再交：
  - 正推腿那三份是任务 20673 退 0，输出 `out-timeout.txt`、`out-git.txt`、`out-guard.txt`。
  - 攻方腿那份是任务 20715 退 0，输出 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-main-run/probe.out`（122 行，末行 `PROBE name=done emitted=122`）。它开头逐份核了冻结副本，四行 `frozen-hash … match=yes`。
- 两条腿报的「判成什么」都是读码推的，下面判决只采两份探针跑出的读数。

## 二、判决

两条腿都打中，第一轮不收。坐实的照「三、改法」改完，再攻一轮。

坐实（探针格名后面是 probe.out 或 out-*.txt 里那一格的读数）：

- 静默放过，钩子退 0、不记检出（`verdict=missed-silently`）：
  - 别名：commit-repository-alias、commit-command-line-alias、commit-command-line-shell-alias，正推腿 G07g；
  - 别的程序起的：commit-through-xargs、commit-through-find-exec，正推腿 G07m；
  - 长选项缩写：commit-abbreviated-message-option、commit-abbreviated-include-option、commit-abbreviated-force-add；
  - 同一条命令里先改状态再提交：commit-after-stash-pop、commit-after-stash-apply-index、commit-after-cherry-pick-no-commit、commit-after-checkout-path-from-branch、commit-after-merge-squash、commit-redo-after-reset-soft、commit-file-created-in-same-command、commit-all-file-grown-in-same-command、commit-repository-made-in-same-command；
  - 私有索引与用户配置：commit-relative-private-index-kept-at-top、commit-path-from-subdirectory-diff-relative-config；
  - push 推的不是当前分支：push-named-other-branch、push-all-branches、push-mirror、push-tags、push-recurse-submodules；
  - 正推腿 G03f、G03g、G03h（git add 用 `--pathspec-from-file`、变量、命令替换）、G07b（目录此刻不存在），都是 `rc=0 | 新增检出 0 条`。
- 误拒：
  - 正推腿 G03e（`git add new.bin && git commit -m x small.txt`）`rc=2`；
  - push-clean-side-branch-while-current-dirty、push-delete-other-ref-while-current-dirty 是 `verdict=rejected-nothing-lands`；
  - remedy-chained-in-one-command，照「三、改法」第 1 条的判法算拒得对，出路改写。
- 钩子出错：
  - judge-fails-keys-missing 是 `hook_exit=0 refused_21=no refused_22=no recorded_hook_layer=yes`：同一条命令里 ㉑ 该拒的也一起放过了；
  - judge-fails-slow-with-child 的 `leftover_after_hook=4`。
- 副作用：index-written-back-by-judge 与 index-written-back-by-judge-optional-locks-off 都是 `index_bytes_changed=yes`。判法改写了用户仓的索引文件。
- 出路照做不通：
  - remedy-incompressible-gzip-size 是 `gz_bytes=104874292 limit=104857600`，压完仍超；
  - remedy-push-message 是 `names_a_redo_command=no`。
- 文档与代码对不上：
  - `scripts/gate/large-file-check.py` 退出码表漏 78（Q16 `rc=78`）；
  - `--limit-bytes ²` 崩溃退 1、不打 JSON（Q14 `rc=1 | stdout 行数=0`）；
  - 77 那半句在 `--guard` 下走不到（Q21 `rc=0`）；
  - ㉑ 一段写「只含数字的字符串」，带前后空白的也拒（T08 `rc=2`）；
  - `.claude/agent-common.md` 第 97 行没写出点名路径时暂存区不判。

不算打中：

- 正推腿 Q01 的预期是错的：`probe-guard.sh` 第 14 行在造仓时已经把 big.bin 提交并推了，`checked 0` 是对的。
- T14（NaN 拒）、timeout-string-exponent 等认不出的字符串放行：都与 ㉑ 文件头「写成布尔或认不出的字符串的……放行」一致，没有静默放过。T15（400 位整数放行并记检出，`新增检出 1 条`）放行是因为 `float()` 溢出，不是认不出的字符串；放行时记了检出，同样没有静默放过。
- commit-amend-keeps-head-big：大 blob 已在 HEAD 里，commit 不带进新内容；它在 push 那一刻会被判，判法分工如此。
- tree-hook-from-linked-worktree、judge-without-git-on-path：`verdict=passed-recorded`，属于真判不了、放行并记检出。
- 第 5 问里 harness 怎么处理 `run_in_background: "true"` 这类写法，仓里判不出；钩子这一侧按前台算，探针 timeout-background-string-true-main 是 `refused_21=yes`。

## 三、改法

派工具员一件（批 3，草稿 `~/.local/share/singlefs/work/fg-timeout-large-file-r1-fixes/`），四条：

1. 提交、推送那一刻判不准内容的写法，执行前拒，出路是改写成能判的形态；真判不了的放行并记检出，不许静默放过。主 agent 定的，**零轮**。它就是改法本身；A/B 由第二轮复跑这两份探针代替，改前的读数就是这一轮的两份输出，任务 20673、20715。
2. 上面「静默放过」「误拒」各格逐个认出来判或拒；push 按推出去的引用逐个判。零轮，A/B 同上。
3. ㉒ 出错一律折成判不了；判法放进自己的进程组、到点停整组；判法不写用户的索引。零轮，A/B 同上。
4. 出路与文档照实改：压完仍超的另给出路，push 的出路给重做命令，退出码表与文件头、`.claude/agent-common.md` 第 96、97 行照实写。只改文字的部分不做 A/B。

## 交用户的

- 用户原话「大文件压缩的钩子在提交时候没有生效」：现在的做法是提交前拒，出路给出压缩命令，钩子不替用户压。要不要改成提交时自动压，交用户定；用户弹窗选了「只拒、给压缩命令」，维持现在的做法。

## 这一轮被判过的文件

- research/prompts/_fg-timeout-large-file-governance-r1-body.md：腿照它的十一问作答；正推腿的 Q01 预期写错，不归正文
- .claude/hooks/bash-command-detector.sh：判了 ㉑ 一段与 `foreground_timeout_refusal`（成立，一处文件头字面不准）；判了 ㉒ 一段与 `git_commit_form`、`git_push_judged`、`git_add_form`、`large_file_findings`、`large_file_check_run`（静默放过、误拒、出错吞掉别的道、留下子进程、改写用户索引，坐实）
- scripts/gate/large-file-check.py：判了 `--guard` 三种模式（成立）、上限只有一份（成立）、退出码表（不成立）、判法改写用户索引（坐实）
- .claude/agent-common.md：判了第 70、75、96 行（成立）与第 97 行（不精）

## 回看决策

不涉及决策：这一轮只判钩子与门禁脚本，不推翻也不确立 `.claude/kb/decisions.md` 里任何一条。
