# 落地审计第三批第 12 号任务：钩子与派发脚本（tooling-writer）

批次：records/2026-10-22-子agent结果落地审计.md#3。三条各自达到出口：自证样本红绿齐、弄坏开关打开后红在点名那一格。没跑重型，没提交。

## 改过的文件

`git diff --stat`（相对暂存区；runner-dispatch-guard.sh 里有另一个会话先改的 13 行 TaskStop 判法，下面「我自己的行数」栏是只数这一轮的，基准是开工前拷贝，diff 在草稿目录 diff-*.patch）：

```
 .claude/hooks/agent-write-scope.tsv    |   2 +
 .claude/hooks/handback-guard.sh        |   8 +--
 .claude/hooks/heavy-test-guard.sh      |  12 ++++
 .claude/hooks/lib_heavy_tests.py       |  21 +++++++
 .claude/hooks/runner-dispatch-guard.sh | 112 +++++++++++++++++++++++++++++----
 .claude/hooks/write-guard.sh           |  23 +++++--
 scripts/agents/agent-watch.py          |   8 +--
 scripts/agents/cite-check.py           |   4 +-
 scripts/agents/dispatch-prompt.py      |  33 +++++++++-
 9 files changed, 192 insertions(+), 31 deletions(-)
```

我自己的行数（+/-）：runner-dispatch-guard.sh +84/-11，write-guard.sh +18/-5，handback-guard.sh +4/-4，heavy-test-guard.sh +12/-0，lib_heavy_tests.py +21/-0，agent-write-scope.tsv +2/-0，dispatch-prompt.py +29/-3，cite-check.py +2/-2，agent-watch.py +4/-4。
钩子一律经 `scripts/gate/hook-swap.py` 换上（草稿里先验、换上后再验）；脚本经 `scripts/kb/replace-batch.py` 定点改。没动 selftest-registry.tsv：没有新脚本，各自证登记的轻重不变（全是 light）。

## 一、报告文件名（改成 result.md）

### 逐处：认 report.md 的地方

结论：五份里没有一处是「真认」report.md 这个文件名的判法；全是自证样本里拿它当报告文件名用。逐处：

| 文件 | 处 | 判 | 处理 |
|---|---|---|---|
| runner-dispatch-guard.sh | 自证样本 10 处（通用:clerk、写范围:执行员报告写进 ×3、没登记工作根的类型 ×2、快照:规格不存在、写者锁 ×3） | 样本里的报告路径；其中写者锁三处是「报告：」一行，新判法下不改会被拒 | 全改 result.md |
| runner-dispatch-guard.sh | OLD_HEAVY_TEST_REFUSALS 里 `m2-vm-raise-floor-implementer-report.md` | 逐字引用 09 月真实拒绝过的原句，不能改 | 不改 |
| runner-dispatch-guard.sh | 样本 `报告路径：research/prompts/e142-r15-runner-report.md`、`不要写进 research/prompts/x-report.md` | 测写范围（research/ 下越界）与否定词，不是测文件名 | 不改 |
| write-guard.sh | 范围三处、stdin 一处、写者锁一处 | 样本里的报告路径 | 全改 result.md |
| handback-guard.sh | 样本四处（scribe-report.md、report.md ×3） | 样本里的报告文件；REPORT_MENTION 是通配 `.md`，不认名字 | 改 result.md / scribe-result.md |
| agent-watch.py | `progress_file_of` 样本三处、`cat >> report.md` 一处 | progress_file_of 取同目录 progress.md，不认报告名；heredoc 一处只测去 heredoc 正文 | 改 result.md |
| agent-watch.py | `kept-copy-report.md`（3578） | python 自己写的样本，不经 Write 工具 | 不改 |
| cite-check.py | 样本两处 report.md | 样本 | 改 result.md |
| cite-check.py | `bare-report.md`（343）与文件头第 42 行引它的说明 | python 自己写，不经 Write 工具 | 不改 |

没改的两个 `*-report.md` 样本名是 python 写的，不碰工具层那道拦截；要统一再改，不影响判法。

### dispatch-prompt.py

「报告：」一行默认填 `<草稿目录>/result.md`（`--set 草稿目录=` 给了就拼，末尾斜杠只留一个；没给草稿目录写 `报告：<填 草稿目录，与上一行一致>/result.md`，占位仍拦着；`--set 报告=…` 照填）。开关 `DISPATCH_PROMPT_BREAK=report-default-dropped`。

### 派发闸 ⑰（落点：runner-dispatch-guard.sh）

先查了 `gate-overlap.py --list`：管「派发提示写得对不对」的钩子只有 runner-dispatch-guard.sh（PreToolUse[Agent|Task]），dispatch-prompt.py --check 也经它的 --check-prompt 判，所以并进它，不新建钩子（不需要 gate-similar / hook-events 新行）。新增 `report_file_name_verdict`：行首「报告：」（可带 - * > 与粗体）后面的路径，最后一段带扩展名而名字里带 report（不分大小写）的拒，拒绝说明给工具层原文、结论与 `result.md` 的写法。闭合围栏代码块里的行不判；目录名带 report 不碍事；「报告路径：」「报告写进」不判（见「没做什么」）。无放行行。

红绿样本（hook 自证 `runner-dispatch-guard.sh --selftest`）：

| 样本 | 应得 |
|---|---|
| 报告：…/report.md；…/final-report.md；…/Report.MD；`- **报告**：…report.md`；反引号包着的路径；一行写 result.md 与 report.md 两个；代码块之后行首的那一行 | 拒（红） |
| 报告：…/result.md；目录 reports/ 下的 result.md；代码块里的示例行；「result.md，不要叫 report」；「报告路径：…/report.md」；没有「报告：」一行 | 放行（绿） |
| 拒绝说明点名文件名、规矩、`草稿目录下的 result.md`；stdin 入口退 2 且 stderr 写 result.md 与 dispatch-prompt.py | 真 |
| `dispatch-prompt.py --check` 填完的工具员骨架（默认 result.md） | 退 0（原样见草稿目录 e2e.md） |
| 同一份把 result.md 换成 report.md | 退 3，点名 `报告文件名` |

读数：`runner-dispatch-guard.sh --selftest` 退 0，「自检通过（查了 437 种）」，✗ 行 0 条。
弄坏开关（`RUNNER_DISPATCH_GUARD_BREAK=`）：

| 开关 | 红在哪几格 |
|---|---|
| report-name-unchecked | 报告名 7 条拒的样本 + 拒绝说明格 + 两条 stdin（共 10 格） |
| report-name-case-sensitive | 「报告名:大写也拒（不分大小写）」（应当返回 2，实际 0） |
| report-name-whole-path | 「报告名:目录名带 report、文件名不带放行」（应当返回 0，实际 2） |
| report-name-codeblock-judged | 「报告名:代码块里的示例行不判放行」（应当返回 0，实际 2） |

`dispatch-prompt.py --selftest` 退 0，「自检通过（查了 78 项）」；`DISPATCH_PROMPT_BREAK=report-default-dropped` 退 1，红在「有定义的类型：骨架缺 ['报告：…/result.md']」「骨架的「报告：」一行默认应当填 <草稿目录>/result.md」两格（第三格 --check 写 report.md 的样本也跟着红）。

其余读数（样本改名，没有新判法，不另设开关）：`handback-guard.sh --selftest` 退 0，查了 22 种；`cite-check.py --selftest` 退 0；`agent-watch.py --selftest` 退 0（约 3 分钟，57 个子 agent、55 个进程、46 次看门狗）。

## 二、写范围表两行

`.claude/hooks/agent-write-scope.tsv`：`kb-scribe  crates/singlefs-checker/invariants/*.md`、`experiment-runner  .claude/gate.d/replay-comparison-units.tsv`（第三列写了出处与「定义的写范围由第 11 号任务补」）。`*` 不跨目录，所以只限这些 `.md`。

自证在 `write-guard.sh --selftest`（表副本取真表）：kb-scribe 写清单 .md 放行；清单目录里的 .rs、子目录里的 .md、checker 源码拒；experiment-runner 写比对对象登记放行、写别的门禁文件拒。读数：退 0，「自检通过（查了 228 种情形）」。
弄坏开关 `WRITE_GUARD_BREAK=scope-new-rows-dropped`（自证用的表副本去掉这两行）：退 1，红在「范围:kb-scribe 写不变量清单的 .md 应当是 0，实际 2」「范围:experiment-runner 写重跑比对对象登记 应当是 0，实际 2」，正好这两格。
Bash 一侧（⑫ 读同一张表）手喂 bash-command-detector.sh：kb-scribe `cp … invariants/cp-01.md` 退 0、`invariants/x.rs` 与 `src/lib.rs` 退 2；experiment-runner `cp … replay-comparison-units.tsv` 退 0、`cp … doc-experiments.sh` 退 2。
注意：门禁 code-tooling 的 agent-write-scope 格只比「表里的名字有定义、有 Write/Edit 的定义有登记」，不比路径，所以第 11 号任务没补定义之前它不红。

## 三、K1 放行

现查：对 implementation-writer 喂两条命令，heavy-test-guard.sh 拒 `bash .claude/gate.d/code-source-discipline.sh --check <四格> <根>`（提交时才跑的检查，退 2）；run-scope-guard.sh 要范围声明（没带退 2，带了合规退 0）；bash-command-detector.sh、two-logs-guard、timer 闸、pattern-process-guard、host-resources-reminder 都不拒。`naming-lint.sh <根>` 三道钩子本来都退 0，不用放行（自证里钉了一格防以后误加进 lint 表）。
落点：lib_heavy_tests.py 的 `classify_gate_stage`（与广谱变异 `--check gate-verdict` 那一条同形）；常量 `NAMING_SELF_CHECK_STAGE`、`NAMING_SELF_CHECK_CELLS`。`--check`（含 `--check=`，给几次取并集）点名的格非空且全在 vague-names、test-file-names、ordinal-names、data-names 之内才放行；不带 --check、点名里有别的格、点名为空的照拒。看门狗共用这份库，同一判法。拒绝说明（COMMIT_CHECK_POLICY）补了这条放行与 naming-lint.sh。范围声明仍由 run-scope-guard.sh 要，这里不判。
实跑一次（实现员口径，带声明，对主工作区根）：22 秒跑完，四格里三格红（vague-names、ordinal-names、data-names，仓里的存量，不是这一轮的改动），test-file-names 绿。
注意：这类命令的输出不能重定向到 .log（two-logs-guard 拒），要看输出管道给 tail，或用 `scripts/hosts/two-logs.sh`；第 11 号任务写进共用约束时要带这一句。

样本：lib 自证新增 8 格（四格放行、等号写法一格、两次 --check 取并集一格；夹着别的格、只点名别的格、不带 --check、点名为空各拒；naming-lint.sh 放行）；guard 自证新增 5 格（含「经范围声明与 nice 起、等号写法」）。读数：`heavy-test-guard.sh --selftest` 退 0（「查了 1386 种」，里面 lib 自检「查了 775 种」）；`lib_heavy_tests.py --selftest` 退 0。
弄坏开关（`LIB_HEAVY_TESTS_BREAK=`）：naming-self-check-counted → lib 红「命名自查：点名四格 / 点名一格（等号写法） / --check 给两次取并集 应当是 None，实际 commit-check-stage」三格，guard 红「实现员交回前自查命名：code-source-discipline.sh 点名那四格：放行」「…经范围声明与 nice 起、等号写法点名一格：放行」两格；naming-self-check-any-cell → lib 红「命名自查：点名里夹着别的格照算」「命名自查：只点名别的格照算」两格，guard 红「…点名里夹着别的格：照拒」一格。

## 要主 agent 跑的命令

无。我改到的钩子与脚本的 `--selftest` 在 selftest-registry.tsv 里都登记为 light，已在主工作区自己跑到绿（读数见上）。语法检查：四个 .sh 的 `bash -n`、四个 .py 的 `py_compile` 通过（py_compile 在 scripts/agents 下留的 __pycache__ 已删）。

## 没做什么

- 没跑重型、`gate.sh`、门禁阶段（K1 那一次实跑是该四格的 `--check`，属于被放行的自查，不是整轮）、各类 lint；没走定义三方；没提交。
- 派发闸只判「报告：」一行，没判「报告路径：」「报告写进」后面的文件名（任务点名的是「报告：」；这两种写法在 ④ 里读的是写范围）。要一并拦，往 `REPORT_LABEL_LINE` 加标签即可。
- **发现但没碰（不在文件清单里）**：`scripts/gate/apply-writer-patch.py` 第 15、218 行真认补丁目录里的 `report.md`（实现员的报告，补丁动了 checker src 时要有「受影响的层 0 流与崩溃枚举用例」一节），而实现员用 Write 建 report.md 会被工具层拒。要改成 `result.md` 并连实现员定义（第 11 号任务）一起，请主 agent 另排。另有 `scripts/hosts/clean-session-scratch.py`、`test-environment-check.py` 的样本里用 report.md，只是样本。
- `.claude/agent-common.md` 第 107 行「写草稿目录的 `report.md`」与各定义里的报告名属第 11 号任务。
- 推翻条件：若有子 agent 用 Write 写 `result.md` 也被工具层拒，则「result.md 写得进去」不成立，⑰ 的出路要换名字。
