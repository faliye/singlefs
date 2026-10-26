# landing-audit-r1 修补：钩子与脚本那一份（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#5；判决 research/prompts/landing-audit-r1-main-verification.md「三、判决」第一张表「钩子与脚本」那一行（K2-4、K2-6、K3-1、K3-6 写范围表、K4-1、K4-2）。

## 结论

六条出口都达成：

| 条 | 出口 | 读数 |
|---|---|---|
| 1 K2-4 报告行一份解析 | `dispatch-prompt.py --check` 上「报告：…/report.md」「报告路径：…/report.md」退非 0，「报告：（草稿目录）/result.md」退 0；开关打开后「报告路径：」那一格红 | 3 / 3 / 0；`RUNNER_DISPATCH_GUARD_BREAK=report-line-labels-split` 下「报告路径：…/report.md」退 0（红），自证 7 格红 |
| 2 K3-1 三方腿报告默认值 | 三种类型照骨架只填别的占位退非 0；填 `research/prompts/x-r1-opus-output.md` 退 0；填草稿目录下 result.md 退非 0 | 真定义上三种类型各 3 / 0 / 3；自证 12 格，开关见下 |
| 3 K2-6 交回一个定义 | 三处在「先被拒、后来送达」「被拒之后再没送达」两份样本上判得一样；库有自证与开关；三处自证仍绿 | 三处：送达 / 没送达，一致；`SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` 下三处一起把后一份判成交回 |
| 4 K4-1 `--until` | `outcome --until YYYY-MM-DD`，成功句报 `--since`、`--until` 的取值 | 真数据窗口 2026-10-21 到 2026-10-21：128 个；自证「交回与起点」格 |
| 5 K4-2 返工词表 | 「修订」不算；自证加「E177 跑前登记修订」；`narrow-rework-words` 照旧 | 开关下判错清单里有「E177 跑前登记修订」 |
| 6 写范围表 | 执行员那一行说明列指到改后的执行员定义 | 表照 lib_write_scope.py 读得出，别的行不动 |

推翻条件：主工作区里这几份的 `--selftest` 有一份不绿；或拿下面「样本」一节的提示文本经 `python3 scripts/agents/dispatch-prompt.py --check` 判出与表里不同的退出码；或 `python3 scripts/agents/subagent_transcript.py --selftest` 在 `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` 下退 0。

## 改过的文件

新建：`scripts/agents/subagent_transcript.py`（313 行，未跟踪）。

`git diff --stat -- <这些文件>` 原样（只看未暂存的，即这一轮的改动）：

```
 .claude/hooks/agent-write-scope.tsv    |   2 +-
 .claude/hooks/continuation-guard.sh    |  95 +++++++----------
 .claude/hooks/runner-dispatch-guard.sh | 184 +++++++++++++++++++++++++++------
 .claude/hooks/selftest-registry.tsv    |   1 +
 scripts/agents/README.md               |   3 +-
 scripts/agents/agent-cost.py           | 172 ++++++++++++++++++++++--------
 scripts/agents/agent-watch.py          |  39 ++++---
 scripts/agents/dispatch-prompt.py      |  59 ++++++++++-
 8 files changed, 398 insertions(+), 157 deletions(-)
```

sha256（交回时）：

```
637fb1c975a183f0eb5e7fdc52bceb0e163b1c39fe378f7848eb3b5d5a7e2a8a  .claude/hooks/runner-dispatch-guard.sh
f0288933cfac6a7c6684572048e8b4570d4a2d09385dc899ff39bd6705a09913  .claude/hooks/continuation-guard.sh
774dd0b9991c733c9741e842334d94853c672dfdddce4f9d1085163ec6b30fd7  scripts/agents/dispatch-prompt.py
f1c672582761a7b91920fe682dcbb2fcbf4d1bf30aa800ebb45f2d584d7d1243  scripts/agents/agent-cost.py
8898d7d4de47956a606f40d2bfcddfd62ca26e752acb8bf8365f8ae9dc5e8f55  scripts/agents/agent-watch.py
65cd850c047191e29a8945fe3676c622833bf5bf46c1e508656a8c3119a1db10  scripts/agents/subagent_transcript.py
14494769762367b8ffcf47e38869d46d3ce7b9c1fb7897244c55871ac898722b  scripts/agents/README.md
5164248e3b0aadc131f9578b8f6dbe8696eddfac646d39c383699986acc13de7  .claude/hooks/agent-write-scope.tsv
122b81f622504c0b387f1285a50bd921a20f0ba51f993b869651fa84ae12f92b  .claude/hooks/selftest-registry.tsv
```

写法：两个钩子经 `scripts/gate/hook-swap.py` 换上（规格由草稿里改好的全文与原文逐段对出、每处唯一；草稿里语法检查、内嵌 python 编译、喂法全过才换，换上后主工作区再验），各两次（一次正文、一次只改文件头开关表）。四次都退 0。反向规格与喂法输出在草稿目录 `hook-swap/` 下。脚本用 Edit 工具或「同目录临时文件再 mv」换 inode。开工与每次写之前核过：`runner-dispatch-guard.sh` 写之前的 sha256 与开工时一致（c70f1683…），别的会话的 TaskStop 一段没动（原文与改后逐行 diff 里 TaskStop 出现 0 次）。

## 1. K2-4：报告行一份解析（`.claude/hooks/runner-dispatch-guard.sh`）

改了什么：
- 新 `report_line_mentions(prompt)`：④ 与 ⑰ 共用。标签认「报告」「报告路径」「报告写进」「报告写到」；前面不连着字（「交回报告：」「把报告写进」不算）；「报告」要接冒号，或接空白再接路径（~、/、$、research/ 等起头）；另三种冒号可有可无、可在句中；闭合了的围栏代码块里的行不认。标签之后到行尾是这一行的正文；正文里第一个带 / 或带扩展名的词是这一行点名的路径。
- ④（`out_of_scope_mentions`）：「写进 / 写到 / 更新」照旧用 `SCOPE_MENTION` 认（删掉了里面的三个报告标签），报告行点名的路径从共用解析取；报告行还有 `<填 …>` 占位的交 ⑬，④ 不判。
- ⑰（`report_file_name_verdict`）：从共用解析取正文里的每个词判文件名；删 `REPORT_LABEL_LINE`。拒绝说明改成「报告行点名的文件名里带 report」，并写明四种标签。
- `REPORT_NAME_TOKEN` 不再按尖括号切词（「<轮>-sonnet-output.md」这类形态写法要整段算一个词）。
- 自证样本：「报告名:「报告路径：」一行不是这一种标签，不判」（应得 0）改成「报告名:「报告路径：」一行写 report.md 也拒」（应得 2）。新加「报告写进」「报告写到」「句中「，报告 <路径>」」三格拒、「交回报告：」放行，「写范围:「报告：」一行点名写范围外的路径也拒」「…工作根下的路径放行」「报告行里还有占位的交骨架占位那一条、写范围不判」三格。
- 「写范围:执行员报告写进 research/prompts（09-26 那一次）」样本的文件名从 `…-runner-report.md` 改成 `…-runner-result.md`：并成一份之后 ⑰ 也会拒原名，`scope-ignored` 开关下这一格就红不出来了。

新开关 `RUNNER_DISPATCH_GUARD_BREAK=report-line-labels-split`（退回改前：⑰ 只认行首「报告：」，④ 用改前带三个报告标签的 `SCOPE_MENTION_BEFORE_MERGE`）。打开后自证红的格（原样）：

```
写范围:「报告：」一行点名写范围外的路径也拒 应当返回 2，实际 0
报告名:「报告路径：」一行写 report.md 也拒 应当返回 2，实际 0
报告名:「报告写进」后面写 report.md 也拒 应当返回 2，实际 0
报告名:「报告写到」后面写 report.md 也拒 应当返回 2，实际 0
报告名:句中「，报告 <路径>」也认 应当返回 2，实际 0
三方报告:报告路径写成仓根下的绝对路径放行 应当返回 0，实际 2
三方报告:报告路径一行写在括注里的形态、后面接真路径放行 应当返回 0，实际 2
```

出口的 `--check` 读数（clerk 骨架，草稿目录 `<草稿目录>/exit/probe-clerk`，只改「报告」那一行）：

```
[报告：…/probe-clerk/report.md] rc=3   ✗ 派 clerk 的提示报告行点名的文件名里带 report：「…」
[报告路径：…/probe-clerk/report.md] rc=3   ✗ 派 clerk 的提示报告行点名的文件名里带 report：「…」
[报告：…/probe-clerk/result.md] rc=0   ✓ 派发闸照这份提示放行（clerk，查了 18 行提示）
[report-line-labels-split: 报告路径：…/probe-clerk/report.md] rc=0
```

（「…」是本草稿目录 `exit/probe-clerk` 的绝对路径，这里截短；提示文本在草稿目录 `exit/` 下，命令在下面「复跑命令」一节。）

## 2. K3-1：三方腿的报告默认值与派发闸的形态判

`scripts/agents/dispatch-prompt.py`：
- `THREE_WAY_REPORT_TYPES`（three-way-forward、three-way-attack、three-way-defense）的「报告：」一行填占位 `research/prompts/<填 轮>-<填 模型>-output.md`（`THREE_WAY_REPORT_PLACEHOLDER`）。定义的 required-inputs 要了「报告」的照定义打一行；没要的经 `EXTRA_LINES` 也打一行；只打一行。`--set 报告=…` 照填。别的类型照旧 `<草稿目录>/result.md`。
- 第 17 号任务已经给攻方、辩方定义的 required-inputs 补了「报告」（现查：三份都是 `…, 报告`），骨架照它打；补之前那条 `EXTRA_LINES` 兜着。

`.claude/hooks/runner-dispatch-guard.sh` ⑰ 另加 `three_way_report_verdict`：三方三种类型，报告行点名的文件（带扩展名的那个，去掉仓根前缀或 `./`）要形如 `research/prompts/<名>-output.md`；有一行点名的是别的文件、或一行报告行都没有，拒；报告行还有 `<填 …>` 占位的交 ⑬。拒绝说明给形态、交回闸为什么只认它、骨架命令。

出口读数（真定义，`dispatch-prompt.py <类型> --set 草稿目录=…` 打骨架，别的占位填「已填」、预计挂钟 2 小时内，只动「报告」那一行）：

```
报告：research/prompts/<填 轮>-<填 模型>-output.md
  [three-way-forward KEEP] rc=3  ✗ 派 three-way-forward 的提示骨架还有 1 处占位没填：
  [three-way-forward 报告：research/prompts/x-r1-opus-output.md] rc=0    ✓ 派发闸照这份提示放行（three-way-forward，查了 20 行提示）
  [three-way-forward 报告：…/probe-three-way-forward/result.md] rc=3  ✗ 派 three-way-forward 的提示报告行点名的文件不是这一形态：「…」
  [three-way-attack KEEP] rc=3  ✗ 派 three-way-attack 的提示骨架还有 1 处占位没填：
  [three-way-attack 报告：research/prompts/x-r1-opus-output.md] rc=0    ✓ 派发闸照这份提示放行（three-way-attack，查了 21 行提示）
  [three-way-attack 报告：…/probe-three-way-attack/result.md] rc=3  ✗ 派 three-way-attack 的提示报告行点名的文件不是这一形态：「…」
  [three-way-defense KEEP] rc=3  ✗ 派 three-way-defense 的提示骨架还有 1 处占位没填：
  [three-way-defense 报告：research/prompts/x-r1-opus-output.md] rc=0    ✓ 派发闸照这份提示放行（three-way-defense，查了 21 行提示）
  [three-way-defense 报告：…/probe-three-way-defense/result.md] rc=3  ✗ 派 three-way-defense 的提示报告行点名的文件不是这一形态：「…」
```

自证格与开关：

| 自证格（dispatch-prompt.py，三种类型各一句） | 红在 |
|---|---|
| 三方腿的骨架应当只打一行「报告：research/prompts/<填 轮>-<填 模型>-output.md」 | `DISPATCH_PROMPT_BREAK=three-way-report-default-dropped`（新） |
| 三方腿骨架只填别的占位、不动报告行（退 3 并点名这一行） | `DISPATCH_PROMPT_BREAK=three-way-report-default-dropped`；`RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored` |
| 三方腿骨架报告填成草稿目录下的 result.md（退 3 并给形态） | `RUNNER_DISPATCH_GUARD_BREAK=three-way-report-unchecked`（新） |
| 三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（退 0，对照格） | `RUNNER_DISPATCH_GUARD_BREAK=thread-limit-any-refused`（通用开关，只证这一格会红） |

派发闸自证新加「三方报告:」10 格；`RUNNER_DISPATCH_GUARD_BREAK=three-way-report-unchecked` 下原样红：

```
三方报告:攻方腿报告写草稿目录下的 result.md 拒 应当返回 2，实际 0
三方报告:正推腿报告写草稿目录下的 result.md 拒 应当返回 2，实际 0
三方报告:辩方腿没写报告行拒 应当返回 2，实际 0
三方报告:报告行写别的仓内文件拒 应当返回 2，实际 0
三方报告:拒绝说明给出形态与骨架 应当返回 True，实际 False
```

自证里 `case_in` 新加 `report=True`：派三方三种类型的样本默认补一行合形态的报告行（原有的长活、线程上限那几格派这三种类型、不写报告行，不补会被这一条拦下）；要判缺行的传 `report=False`。

## 3. K2-6：「交回」与「起点」一个定义（`scripts/agents/subagent_transcript.py`，新库）

库给的：`handback_result_is_error(block)`、`HandbackTracker`（`wants(line)` 原文快筛、`feed(record)` 逐条认，交出这一条里认出的送达；`delivered` 是最近一次送达）、`delivered_handback(path)`、`handback_delivered(path)`、`record_moment`、`first_record_moment(path)`、`started_between(path, since, until)`；三处自证共用的两份样本 `sample_refused_then_delivered()`、`sample_refused_only()`。送达的认法与续做闸原来的一样：调过 SubagentHandback，后面有它对应、不是错误的 tool_result（is_error 不为真、内容不是 `{"success": false}`）。

删了的、改成从库取的：
- `continuation-guard.sh`：删 `handback_result_is_error` 与整份 `handback_delivered` 的重写（只留调库的三行）；删开关 `handback-grep`、`handback-anyresult`（代码随库搬走，它们管的那几格由库的开关管）。读不到库时子 agent 那一侧不判、放行，stderr 写「读不到共用模块」（`hook-swap.py` 认这句）；新开关 `CONTINUATION_GUARD_BREAK=library-unread` 证这条路。
- `agent-cost.py`：`scan_subagent` 删「调过就算交回」那一支，交回正文取最近一次送达的那一次（快筛留下的行另喂库）；起点改取 `first_record_moment`；`collect_outcome` 删 `scan["start"] < since` 那一段与按修改时刻的筛选，改 `started_between`；`subagent_files`（usage、timing、dispatch 三个子命令）也改按起点筛；`read_transcript` 的交回时刻改取送达的那一刻（timing 的「没交回」一列因此与续做闸同义）。主会话的筛选照旧按修改时刻。
- `agent-watch.py`：`AgentTranscript._read` 删交回调用号、正文表与「结果里带 "success":true 才算」那一段，改逐条喂 `HandbackTracker`、按 `feed` 交出的送达置交回（续做消息撤销交回那一套照旧）。认法从「success:true」变成「不是错误」：结果是纯文字、不带 success 字段的，现在也算送达（续做闸原来就这样认）。
- 看门狗的「跑满 N 小时」用的起点仍是它逐行读时的首条带时间戳的记录，与库同义，没改成调库（那几行与它别的逐行统计在同一个循环里）。

三处在同两份会话记录上的判定（`tools/crosscheck.py`，原样）：

```
crossdelivered01：库 handback_delivered=True；续做闸退 2（2 = 交回过不续）；agent-cost scan_subagent 交回正文='短的报告'、read_transcript 交回时刻=有；agent-watch 状态=已交回、is_done=True
crossrefused0001：库 handback_delivered=False；续做闸退 0（2 = 交回过不续）；agent-cost scan_subagent 交回正文=None、read_transcript 交回时刻=无；agent-watch 状态=思考中、is_done=False
[refused-counts-delivered] crossdelivered01：库 handback_delivered=True；续做闸退 2（2 = 交回过不续）；agent-cost scan_subagent 交回正文='短的报告'、read_transcript 交回时刻=有；agent-watch 状态=已交回、is_done=True
[refused-counts-delivered] crossrefused0001：库 handback_delivered=True；续做闸退 2（2 = 交回过不续）；agent-cost scan_subagent 交回正文='长的报告'、read_transcript 交回时刻=有；agent-watch 状态=已交回、is_done=True
```

库的开关（文件头开关表，两条都照 code-tooling break-switch-replay 格的读法核过点名的项都在失败行里）：

| 开关 | 库自证红的格 | 三处调用方红的格 |
|---|---|---|
| `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered`（被拒的也算送达） | 被拒之后再没送达不算交回；success:false 不算交回；被拒之后再没送达的样本 | 续做闸：调过 SubagentHandback 但 tool_result 是错误的放行、…success:false…的放行、共用样本：被拒之后再没送达的放行；agent-cost：交回与起点（outcome 与 timing 两句）；看门狗：子 agent refusedonly 应当不算交回或被停 |
| `SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime`（起点取修改时刻） | 起点按首条记录；窗口按起点筛；一条带时间戳的记录都没有 | agent-cost：交回与起点（outcome 那一句） |

续做闸新开关 `CONTINUATION_GUARD_BREAK=library-unread` 红：已交回（SubagentHandback 送达）的不续、共用样本：先被拒、后来送达的不续、stdin:已交回的续做。

## 4. K4-1：`agent-cost.py outcome --until`

- `--until YYYY-MM-DD`（只 outcome 认）：上界是那一天的第二天零点（与 `--since` 同一种切法），含那一天；`--until` 早于 `--since` 退 2。
- 成功句与两个汇总标题报窗口。真数据读数（`outcome --since 2026-10-21 --until 2026-10-21 --summary --jobs 4`，6 秒）：

```
## 按结局桶（2026-10-21 起、到 2026-10-21 为止，128 个，折算合计 337.11 百万）
## 没交回的 16 个，按截断原因：用量撞顶 3、停掉 13
  ✓ 查了 128 个子 agent（--since 2026-10-21、--until 2026-10-21（含那一天），按首条记录的时刻筛），逐个归了结局桶
```

  `--since 2026-10-21 --until 2026-10-20`：`✗ --until 2026-10-20 早于 --since 2026-10-21，窗口是空的`，退 2。
- 自证新格「交回与起点」（`selftest_handback_and_start`）：三份样本（先被拒后送达、被拒之后再没送达，开工在 `--until` 那一天；第三份在两天后开工），outcome 要收前两份、判前一份交回后一份没交回、不收第三份；timing 收三份、「没交回」只数 1 个。
- 原 outcome 样本的交回调用后面补了一条成功的 tool_result（送达才算交回），用量样本同补；数不变（outcome 18 个子 agent、各桶照旧）。

## 5. K4-2：返工词表

`REWORK_WORDS` 的单字「修」改成 `修(?!订)`；文件头词表说明同改。自证「返工词表」的不算返工清单加「E177 跑前登记修订」。`AGENT_COST_BREAK=narrow-rework-words` 下判错清单原样：

```
判错的 ['七·6 代码轮第二轮攻方腿（接续）', '实验执行：E181 第一段（接续）', 'm3-batch28-code-r1 正推腿（续）', '7-2b 接手：M3 在核上证', '准入条款簇三方攻方腿（重派）', 'Runner E187 r2 (redispatch)', 'Fix E193 guard flush bug and rerun', 'E156 第 7 次收尾', 'E177 跑前登记修订']
```

## 6. 写范围表

`.claude/hooks/agent-write-scope.tsv` 执行员 `.claude/gate.d/replay-comparison-units.tsv` 那一行说明列改成「experiment-runner.md 第 5 步与「写范围」：重跑登记有比对对象表的，实现单元时往这张表每个比对对象登一行（只写这个实验的行）」；现查执行员定义第 5 步与「写范围」一节都已点名这张表。别的行不动。`lib_write_scope.load_scopes` 读出 6 个类型、执行员那一条模式照旧；三列以外的非注释行 0 行。

## 自证读数（主工作区，交回时这一版）

全部登记为 light，自己跑：

| 命令 | 结果 |
|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 退 0，「✓ 自检通过（查了 454 种）」（开工时 437 种） |
| `bash .claude/hooks/continuation-guard.sh --selftest` | 退 0，「✓ 自检通过（查了 20 种）」（开工时 18 种） |
| `python3 scripts/agents/dispatch-prompt.py --selftest` | 退 0，「✓ 自检通过（查了 90 项）」（开工时 78 项） |
| `python3 scripts/agents/agent-cost.py --selftest` | 退 0，「✓ 自检通过（查了 14 项，其中 outcome 的 18 个子 agent）」（开工时 12 项） |
| `python3 scripts/agents/agent-watch.py --selftest` | 退 0，180 秒，「查了 57 个子 agent、55 个进程（其中假的重型进程 21 个）、46 次看门狗」 |
| `python3 scripts/agents/subagent_transcript.py --selftest` | 退 0，「✓ subagent_transcript 自证通过（查了 12 项）」 |

开关逐个打开（每条都退 1、红在点名的格；照 code-tooling break-switch-replay 格的读法用草稿里的 `tools/replay-emulate.py` 核点名的项）：
- 派发闸：`report-line-labels-split`、`three-way-report-unchecked`、`report-name-unchecked`、`report-name-case-sensitive`、`report-name-whole-path`、`report-name-codeblock-judged`、`scope-ignored`、`scope-library-unread`、`scope-negation-ignored`、`scope-token-unexpanded` 逐条跑过，红的格与文件头点名的一致（`scope-token-unexpanded` 另红新格「写范围:「报告：」一行点名工作根下的路径放行」，多红不碍）。
- 续做闸：五条全对（disable-check、anyrecord、ignore-main-handback、library-unread、SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered）。
- agent-cost：六条全对（四条原有 + 库的两条）。
- 库：两条全对。
- dispatch-prompt：17 条里新加与改的五条全对（three-way-report-default-dropped、report-default-dropped、派发闸的 three-way-report-unchecked、placeholders-ignored、thread-limit-any-refused）；另三条（long-run-lines-dropped、long-run-types-narrowed、live-words-narrowed）照那一格的读法对不上，是改前就有的点名写法（「<类型>」「nproc …」与没写「」），开关本身照样退 1；dispatch-prompt 不在复跑登记表里，这三条没动。
- 看门狗：`SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` 退 1，失败行只有「✗ 自检：子 agent refusedonly 应当不算交回或被停，实际状态「已交回」」。`AGENT_WATCH_BREAK=finished` 两次都超过前台 236 秒的上限没跑完（这条开关下有几次看门狗要等 60 秒才退），交主 agent。

## 判法变宽的地方（主 agent 派发时会碰到）

- ④ 现在也判报告行点名的路径：写范围表里有行的类型，「报告：research/prompts/…」这类落在写范围外的会被拒（改前只有「报告路径 / 报告写进 / 报告写到」被判，裸「报告：」放过）。
- ⑰ 现在四种标签都判文件名，标签可在句中（前面不连着字）；正文里任何带扩展名、名字里带 report 的词都拒（改前只判行首「报告：」）。
- 三方三种类型的提示没有报告行、或报告行点名的文件不是 `research/prompts/<名>-output.md`，派发闸拒。
- 交回：看门狗原来只认结果里带 `"success":true` 的，现在认「不是错误」（与续做闸同）；agent-cost 原来调过就算，现在要送达。usage、timing、dispatch 三个子命令收哪些子 agent 从「文件修改时刻不早于 --since」改成「首条记录不早于 --since」（攻方量过 2026-10-01 起差 3 个）。

## 要主 agent 跑或改的

1. 看门狗 `finished` 开关（前台跑不完，用 run_in_background 起；工作目录仓根；不经内存包装、不起重活）：
   `AGENT_WATCH_BREAK=finished python3 scripts/agents/agent-watch.py --selftest`
   判法：退 1，失败行里要有「✗ 自检：子 agent refusedthendelivered 应当算交回或被停」（这一轮新加的点名项）与文件头那一条原有的各项。
2. 门禁那一侧（`.claude/gate.d/` 不归这一份）：新库的自证要进 `.claude/gate.d/code-tooling.sh` research-script-selftests 格的 `SELFTEST_RUNNERS`，加一项 `"python3 scripts/agents/subagent_transcript.py --selftest"`；不加，research-script-selftest-coverage 格可能报它没人跑。要它进开关复跑的，`.claude/gate.d/break-switch-replay-files.tsv` 加一行 `scripts/agents/subagent_transcript.py<TAB>60<TAB># 理由`（两条开关的点名项照那一格读法核过都对）。
3. 两份钩子与四份脚本的改动是这一轮的定义三方之外的工具改动；要开第二轮攻修补的，被判文件清单照上面「改过的文件」。

## 复跑命令（工作目录仓根，`W=~/.local/share/singlefs/work/landing-audit-r1fix-hooks`）

- K2-4 出口：`python3 scripts/agents/dispatch-prompt.py clerk --set 草稿目录=$W/exit/probe-clerk --set 认领用途=出口核对 --set 关的是哪几条=核一件事 --set 出口=交回 --batch records/x.md#1`，把预计挂钟改成 2 小时内、删分片续跑多机三行，「报告：」一行逐个换成三种写法，各跑 `python3 scripts/agents/dispatch-prompt.py --check <文件>`；开关那一次前面加 `RUNNER_DISPATCH_GUARD_BREAK=report-line-labels-split`。
- K3-1 出口：同上换成三种三方类型，别的占位填满、报告行照三种写法。
- K2-6 三处一致：`python3 $W/tools/crosscheck.py <仓根> <空目录，里面先建 session/subagents 与空的 session.jsonl>`，再带 `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` 跑一次。
- 开关复跑：`python3 $W/tools/replay-emulate.py . <脚本> <起法，{script} 代脚本路径>`，例 `python3 $W/tools/replay-emulate.py . .claude/hooks/continuation-guard.sh bash {script} --selftest`。
- 钩子撤回：草稿目录 `hook-swap/` 下四份 `reverse-*.json` 是四次换上的反向规格（时间次序倒着用），`python3 scripts/gate/hook-swap.py <反向规格> --hook <钩子> --scratch $W --feeds <不含新判法的喂法>`；`feeds-continuation.json` 引的两份样本会话记录已删，要用先照 `tools/crosscheck.py` 的写法重造。

## 草稿目录里留着的

`$W` 下：`logs/`（自证输出）、`tools/`（make-spec.py、replay-emulate.py、crosscheck.py）、`hook-swap/`（喂法输出与反向规格）、`spec-*.json`、`feeds-*.json`、`exit/`（出口核对用的提示文本）、`orig.sha256`。没有仓副本与编译目录；改前的文件拷贝与样本会话目录已删。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没跑门禁阶段与各类 lint（gate-lint、shell-lint、preflight-lint、gate-overlap、doc-lint、hooks-registered）；没改 `.rs`，没跑命名检查。
- `AGENT_WATCH_BREAK=finished` 那一条开关没跑完（见上）；看门狗别的开关没逐条重跑（改的只有交回那一段，跑了它的整份自证与库的开关）。
- 看门狗的「起点」没改成调库（逐行循环里与别的统计一起取，判法与库同义）。
- 没碰 `.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/gate.d/`、`.claude/hooks/lib_long_run.py`、`scripts/kb/`；没新建钩子，没改 `.claude/settings.json`。
- dispatch-prompt 文件头里三条改前就有、照复跑格读法对不上的开关点名项（long-run-lines-dropped、long-run-types-narrowed、live-words-narrowed）没改，不在这一批。
- 调度记录第 42 行的注、`--summary` 输出存档归主 agent（判决「三、判决」二）。
