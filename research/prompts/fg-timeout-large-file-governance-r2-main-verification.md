# fg-timeout-large-file-governance-r2：判决（2026-11-19）

正文 `research/prompts/_fg-timeout-large-file-governance-r2-body.md`，开工快照 `research/prompts/fg-timeout-large-file-governance-r2-snapshot/`。这是这一题的第二轮，前一轮的判决是 `research/prompts/fg-timeout-large-file-governance-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方腿（Sonnet，第 1 到 4 问） | `research/prompts/fg-timeout-large-file-governance-r2-sonnet-output.md` | 第 1 问成立，另有三处同形没关；第 2 问部分成立；第 3 问成立；第 4 问不成立，四种写法照拒 |
| 攻方腿（Opus，第 5 到 11 问） | `research/prompts/fg-timeout-large-file-governance-r2-opus-output.md` | 第 5、6、8、9、11 问成立；第 7 问弱成立；第 10 问不成立 |

核查：

- 开工快照：`sha256sum -c research/prompts/fg-timeout-large-file-governance-r2-snapshot/judged-sha256.txt` 六份 OK，`.claude/agent-common.md: FAILED`。
  - 腿跑着的时候，改正文里不写批号那一路的会话与梳理 subagent 那一路的会话改过它：`python3 scripts/agents/who-touched.py .claude/agent-common.md --since-minutes 90` 列出两边各有 Edit/Write。
  - 这一轮判的四句内容没变，只是整体下移了四行，从第 70、75、96、97 行挪到了第 74、79、100、101 行。腿引的这几句，主 agent 按现行行号逐句核过。
- 两条腿的探针与模型都由主 agent 拷到 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-main/` 再交，结局都是 `type=succeeded exit=0`。
  - 辩方：运行目录 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-main-run/`，含复跑的第一轮两份探针与 `defense.out`。
  - 攻方：运行目录 `~/.local/share/singlefs/work/fg-timeout-large-file-governance-r2-main-opus-run/`，`probe.out` 末行 `PROBE name=done emitted=75`，另有 `name-sets.out`、`clippy-tail.out`、`dispatch-scope.out`。
  - 两份探针开头都核过冻结副本：除 `.claude/agent-common.md` 那一行写 `match=no` 外，其余都是 `match=yes`。
- 攻方点名的门禁格 `bash .claude/gate.d/code-tooling.sh --check shared-data-root-literals`：`type=command-failed exit=1`，两处红，下面「二、判决」里坐实。
- 两条腿报的「判成什么」都是读码推的，下面判决只采主 agent 跑出的读数。

## 二、判决

第一轮坐实的各格都关上了：第一轮攻方探针复跑之后，静默放过只剩 `commit-amend-keeps-head-big` 一格，第一轮判决已判它不算打中。另外，`judge-fails-keys-missing` 判成 `refused_21=yes`，`judge-fails-slow-with-child` 是 `leftover_after_hook=0`，`index-written-back-by-judge` 判成 `index_bytes_changed=no`，`remedy-push-message` 判成 `names_a_redo_command=yes`，正推腿 G03e 是 `rc=0`。

两条腿都打中了新东西，第二轮不收。坐实的：

- 静默放过：
  - 辩方 `defense.out` 里，`push-after-merge-of-dirty-branch`、`push-after-checkout-of-dirty-branch`、`push-after-reset-hard-to-dirty-branch` 都是 `verdict=missed-silently`。
  - 攻方 `probe.out` 里 22 格 `verdict=missed-silently`：别名再绕一层或当场定义、PATH 上的外部子命令、`python3 -c` 与 `awk system()` 与 `git rebase --exec` 与 `git submodule foreach` 里起的 git、`push.recurseSubmodules` 配置、`sort -o` / `sed w` / `awk print >`、写进工作根下的仓、`env -u GIT_INDEX_FILE`，以及推的十种形态（同一条命令里先建分支、快进合并之后、`push.default=matching`、`remote.origin.push`、通配 refspec、`--repo` 加位置参数、推到路径远端）。
- 误拒：攻方 `probe.out` 里十格 `misreject-*` 都是 `verdict=rejected-nothing-lands`（chmod、rm、mkdir、touch、`git stash list`、`git worktree list`、`$TMPDIR` 与 mktemp 的提交说明、tee 写到工作根）；辩方 `defense.out` 里 tee 写说明、xargs、`python3 -c`、`-m "$(python3 …)"` 四格也被拒。
- 判法到点：`leftover_index_copies=1`。
- 写范围表：三方腿的模式能写别的轮、别的腿的报告；八类只带 Read、Bash 的定义写工作根时，写者锁一处都不判。这两条是主 agent 对着 `.claude/hooks/agent-write-scope.tsv` 新行与 ⑫ 的判法现查的。
- 名单一致性：`name-sets.out` 里六格 `verdict="false-mismatch"`、四格 `verdict="missed-silently"`。
- ㉓ 出路：`clippy-tail.out` 里七种表的写法下 `gate_minus_outlet` 不空，出路比门禁真跑的少几条；没有一种写法会让出路被 `project-code-guard.sh` 拒（都是 `verdict="outlet-passes-guard"`）。
- 派发闸 ④：`dispatch-scope.out` 里五格 `verdict="false-refusal"`，是给八类补写范围行之后带出来的。
- 门禁 shared-data-root-literals 两处红：检出钩子第 3578 行经 `~` 取工作根；排除表 `.claude/gate.d/shared-data-root-literals-exempt.tsv` 第 5 行的锚串在 `scripts/agents/dispatch-prompt.py` 里已不命中。

不算打中：

- 辩方第 2 问指出第一轮判决两处字面不准：Q01 造仓的是 `probe-guard.sh` 第 14 行，不是第 16 行；400 位整数那一格的理由写错，那里放行是 `float()` 溢出，不是「认不出的字符串」。结论不变，下面改法里一并改那份判决的字面。
- 辩方推的 `large-file-check.py` 说明写到标准输出、文件头写标准错误，主 agent 没单独复跑这一格，并进第三轮修补一起核。

读命令文本判 git 提交与推送，两轮下来堵一形就有下一形，误拒也跟着多出来。所以第三轮换主闸，用户弹窗选了「加 git 自己的钩子当主闸」。

## 三、改法

派一个工具员（草稿 `~/.local/share/singlefs/work/fg-timeout-large-file-r2-fixes/`），条目照用户定的方向：

1. 提交与推送前的大文件判在 git 自己的 pre-commit、pre-push 钩子里，判的是那一刻真要进提交、真要推出去的内容。钩子文件进仓，经 `core.hooksPath` 生效，配一份安装与核对。用户本地的提交窗口闸接进去，不再靠 `--no-verify` 放行。零轮。
2. 检出钩子 ㉒ 收窄：只拦直接写的 `git commit` / `git push` 里能直接判的那一部分，再拦 `--no-verify`、`-n`、`-c core.hooksPath=` 这类绕开 git 钩子的写法。「判不准就拒」那一套与它带的放行名单、经 `~` 取工作根的那一行一起删。零轮。
3. 写范围表：三方腿按自己那一条腿的报告名收窄；八类写工作根时照写者锁判。零轮。
4. 名单一致性改成按开关表的文法认，㉓ 出路与 `project-code-guard.sh` 共用同一段读表代码，判法到点清掉索引拷贝，排除表失效的那一行删掉，`large-file-check.py` 文件头照实写。零轮。
5. 派发闸 ④ 的误拒归做全量提交门禁的会话那一批的 `.claude/hooks/runner-dispatch-guard.sh`，已问它并不并。
6. 第一轮判决那两处字面照实改。

这几条都是零轮，A/B 就是第三轮复跑这两份探针，拿这一轮的读数当改前读数。

## 交用户的

- 第三轮往哪个方向改，用户弹窗选了「加 git 自己的钩子当主闸」。弹窗里说明过代价：`.git/hooks` 不进仓，要一份安装脚本；窗口外提交用的 `--no-verify` 会一起绕过它，所以窗口闸要改成不靠 `--no-verify` 放行。

## 这一轮被判过的文件

- research/prompts/_fg-timeout-large-file-governance-r2-body.md：腿照它的十一问作答
- .claude/hooks/bash-command-detector.sh：判了 ㉒ 的执行前拒（静默放过、误拒坐实）、经 `~` 取工作根的那一行（坐实）、名单一致性那一格（误报与漏判坐实）、`clippy_lint_tail`（少几条坐实、被拒不成立）
- scripts/gate/large-file-check.py：判了 `--guard` push 按引用判（几种推法漏判坐实）与判法到点后的索引拷贝（留下一份坐实）
- .claude/gate.d/code-tooling.sh：判了 agent-write-scope 格对八类定义的要求（成立）
- .claude/hooks/agent-write-scope.tsv：判了八类新行（三方腿偏宽、写者锁不判坐实）
- scripts/agents/dispatch-prompt.py：判了换上之后与排除表的对应（排除表失效那一行坐实）
- .claude/agent-common.md：判了第 74、79、100、101 行（与代码一致）

## 回看决策

不涉及决策：这一轮只判钩子、门禁与脚本，不推翻也不确立 `.claude/kb/decisions.md` 里任何一条。
