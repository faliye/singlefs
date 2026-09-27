# 工具一交回：subagent 体系分析 22 条的第一段（2026-09-27 JST）

规格 `/tmp/claude-1000/tooling-1/spec.md`；清单与每条的证据在 `research/prompts/agent-analysis-2026-09-27/report.md` 第五节。进度逐条记在同目录 `progress.md`（时刻 JST）。

## 结论

- 第一段 22 条全部落了（序 9 的门禁阶段文件、序 1 的归属行因为 `stage-owners.tsv` 冻结，写进第二段规格）；3.4、3.5、3.6（gate-triage 除外）、4.7 那几行一起做了。
- 新立：3 个定义（`tooling-writer`、`kb-spec-drafter`、`investigator`）、1 个钩子（`.claude/hooks/handback-guard.sh`）、6 个研究脚本（`cite-check.py`、`kb-spec-check.py`、`crash-case-check.py`、`prove-red.sh`、`apply-writer-patch.py`、`closeout-status.py`）。每条改法都先造了该红的输入、弄坏开关证红（逐条在下面）。
- ⚠️ 派发闸从现在起就按新判法拒：主 agent 每份派发提示要有一行「重型测试：不跑」（crash-verifier、gate-triage 除外）、通用 agent 要写「开工先读 `.claude/agent-common.md`」、按定义的 `required-inputs:` 写全输入、派实现员要有一行「要动的 crates 文件：…」、派书记员的规格要过 `kb-spec-check.py`（或写放行行「规格检查已判：…」）。opus 并发上限按用户定改成 8（真会话探针时有 7 个 opus 在跑）。
- 第二段规格：`/tmp/claude-1000/tooling-1/phase-2.md`（81 行，11 份冻结文件的逐条改法）。

## 看门狗的弄坏开关（`AGENT_WATCH_BREAK=<项> python3 research/scripts/agent-watch.py --selftest`，每项退出码与判红的格数）

`handoveranytype rc=1 2`、`progressstale rc=1 1`、`progressnote rc=1 1`、`detectiontype rc=1 1`、`nosingleinstance rc=1 1`（`✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明，实际 4`）、`acksnotpersisted rc=1 4`、`acksneverpruned rc=1 1`、`discovernone rc=1 3`、`discoverfallback rc=1 1`。
`discovernone` 第一次跑是 `rc=0 0`：`--discover` 一个都没找到时 `build_report` 退回「会话里最近动过的全部子 agent」，这个开关看不出来；改成 discover 模式不退回，加 `discoverfallback` 开关与「会话里都交回了就接着盯到点、不报全部交回」一格之后两项都红。`watch.sh --selftest` 加自检⑧（不给 agent 号走 --discover、找不到会话就拒）。

## 逐条（序号照分析报告第五节）

| 序 | 改了哪些文件 | 证红（弄坏开关 → 判红，原样摘自这一次跑的输出） |
|---|---|---|
| 1 tooling-writer | 新 `.claude/agents/tooling-writer.md`；`.claude/hooks/agent-write-scope.tsv` 加 10 行；`.claude/agent-common.md`「不做」第二条加例外；归属行进第二段规格 | 63 号 ③ 现判：`表与定义一致（4 个有 Write 或 Edit 的定义、28 条路径模式）`；③ 的红由已有 red 样本 alpha 那一格钉着（`fx.sh` 跑 63 号 red rc=1、want 全在） |
| 2 看门狗 | `research/scripts/agent-watch.py`（--discover、单例、确认落文件、该交接、hook 检出主体、整点询问附进度、进度文件不涨）、`research/scripts/watch.sh`（不给 agent 号走 --discover，自检⑧）、`research/scripts/watch.conf` | 见下「看门狗的弄坏开关」一节 |
| 3 交回限字 | 新 `.claude/hooks/handback-guard.sh`；`.claude/settings.json` 的 SubagentHandback 加一条；63 号加 ⑯ 注册与自证；`.claude/agent-common.md`「报告」一节；`three-way-materials.md` 产出 | `HANDBACK_GUARD_BREAK=no-limit` → `✗ 自检：超过上限的项目子 agent 应当返回 2，实际 0`；no-cite / cite-any-agent / no-green-count / zero-count-ok / judge-general-purpose 各 rc=1 |
| 4 派发闸 | `.claude/hooks/runner-dispatch-guard.sh`（711 → 约 930 行：逐句判重型测试那一支删掉，换成一行结构化声明；加 required-inputs、通用 agent、写范围、实现员文件与登记、模型并发与限额窗口、书记员规格、设计员准入、变异分诊要表、只补落产物）；14 个定义加 `required-inputs:` | 自证 98 种绿；29 个 `RUNNER_DISPATCH_GUARD_BREAK` 项与 3 个 DISABLE 各 rc=1（例 `inputs-ignored=1(3)`、`registry-ignored=1(1)`、`opus-no-limit`：`✗ 自检：opus:已有 8 个在跑再派 opus 应当返回 2，实际 0`）；旧判法误拦的 14 句原句（主会话记录现抽）带上那一行全放行 |
| 5 规格检查 | 新 `research/scripts/kb-spec-check.py`、新 `.claude/agents/kb-spec-drafter.md`；派发闸对 kb-scribe 跑它 | 自证 13 种；`KB_SPEC_CHECK_BREAK=any-name/any-hits/no-references/no-pointers/no-pairing/no-history/dated-time` 各 rc=1 |
| 6 实现员件数与 600k | `implementation-writer.md`（输入加「要动的 crates 文件」行、第 8 步 600k 交接）；派发闸数那一行（>8 拒）；`main-agent.md` 那一句进第二段 | `crates-no-limit` → `✗ 自检：实现员:文件超过上限 应当返回 2，实际 0` |
| 7 cite-check | 新 `research/scripts/cite-check.py`；forward / attack / defense 加交回前跑它，verifier 第 2 步先跑它；交回闸对这四类跑它 | 自证 11 种；`CITE_CHECK_BREAK=always-ok/no-heading-check/no-background-check/ignore-changed` 各 rc=1 |
| 8 补丁集成 | 新 `research/scripts/apply-writer-patch.py`；`implementation-writer.md`「产出」规定补丁目录五个文件的格式 | 自证 5 种；`APPLY_WRITER_PATCH_BREAK=checker-section-ignored/apply-unchecked/append-duplicates/no-hint` 各 rc=1 |
| 9 证红与崩溃用例 | 新 `research/scripts/prove-red.sh`、新 `research/scripts/crash-case-check.py`；`implementation-writer.md` 3a、7a；门禁阶段文件与 heavy-test-guard 那一半进第二段 | prove-red 自证 4 种、`PROVE_RED_BREAK=any-target/main-allowed/layer0-run/never-caught` 各 rc=1；crash-case-check 自证 4 种、5 个 BREAK 各 rc=1；heavy-test-guard 对实现员跑 prove-red.sh 放行（rc=0） |
| 10 opus 并发与限额窗口 | 并进序 4；上限按用户 2026-09-27 定改成 8 | `opus-no-limit`、`window-ignored`、`window-never-lifted` 各 rc=1；真会话探针：09-22 那次周限额（resets Sep 27, 4pm UTC）之后同一族又派出去没再撞，算已解开，不拒 |
| 11 investigator | 新 `.claude/agents/investigator.md` | 63 号 ⑧（omitClaudeMd、开工先读）现判绿；它只有 Read / Bash，不进写范围表 |
| 12 后台轮询拒 | `.claude/hooks/bash-command-detector.sh`（run_in_background 里轮询 `tasks/*.output` 的循环拒，加 3 个样本）；`agent-common.md` 长活那一条与拒绝写法 ② | `BASH_COMMAND_DETECTOR_ALLOW_BACKGROUND_TASK_OUTPUT_WAIT=1` → `✗ 自检：等待循环:run_in_background 里轮询后台任务的输出文件 应当…`（rc=1，2 处） |
| 13 执行员进度 | `experiment-runner.md` 第 8 步；看门狗「进度文件不涨」与整点询问附进度 | 见看门狗一节 `progressstale`、`progressnote` |
| 14 差分重跑登记 | `experiment-designer.md` 第 4 步；派发闸对设计员的重跑登记跑 `admission.py experiment`，退 77 拒 | `admission-ignored` → `✗ 自检：准入:输入没变不写重跑登记 应当返回 2，实际 0` |
| 15 崩溃验证节奏 | `apply-writer-patch.py` 到提示线（5 个补丁，推的）打一行提示；`main-agent.md` 那一句进第二段 | `no-hint` rc=1 |
| 16 sweep 盘点与宽词 | `sweep.md`（第五种活、7b、12）；`stale-candidates.py --check-facts` 加宽词抽查（>100 行要「宽词已抽查：…」） | `STALE_CANDIDATES_BREAK=wide-unchecked` → `✗ 自证失败：命中超过抽查门槛、出处列没写「宽词已抽…` rc=5；`--benchmark` 照旧 25/25 |
| 17 续做闸查主会话 | `.claude/hooks/continuation-guard.sh`（主会话记录里有 `<agent-message from="<id>">` + `[Subagent hand-back]` 就拒） | `CONTINUATION_GUARD_BREAK=ignore-main-handback` → `✗ 自检：主会话已有交回消息、子 agent 记录里还没有：拒 应当返回 2，实…` |
| 18 本地腿损坏闸 | `research/scripts/corruption-check.py`（单字母系统性替换一类；提示里的代码片段不算粘连；用法 `<输出> [<提示>]`；加 --selftest）；`three-way-local-attack.md` 5b 答案表比数 | `CORRUPTION_CHECK_BREAK=keep-prompt-code/no-substitution` 各 rc=1；`ask-local-selftest.sh` 照旧 rc=0 |
| 19 model 取值 | 63 号加 ⑮；red 样本加 `delta.md`（model: local-model） | `AGENT_WRITE_SCOPE_BREAK=any-model` → red 样本缺「delta：model: local-model」 |
| 20 冻结副本只读 | `three-way-attack.md`、`three-way-defense.md` 写范围；`main-agent.md` 那一句进第二段 | 定义文字，没有会红的检查（核查员 sha256 -c 那一条是报告 3.5 的推的写法，没做） |
| 21 执行员 doc-lint 与 Bash 就地写 | `experiment-runner.md` 7c；`bash-command-detector.sh` ⑧；`agent-common.md`「写」一节 | `BASH_COMMAND_DETECTOR_ALLOW_REPOSITORY_IN_PLACE_EDIT=1` → rc=1（3 处） |
| 22 closeout-status | 新 `research/scripts/closeout-status.py` | `CLOSEOUT_STATUS_BREAK=everything-done` rc=1；真表现算：在跑 4、等着 6、其他 2、完成 32，共 44 批 |
| 3.4 / 3.5 / 3.6 / 4.7 | 书记员三行（规格检查、汇总交回后才报——代码里已如此，没改、绿行报数）；材料员（5b 快照、不抄行不进回复、`checklist-specs.py` 核随行）；本地辩方一侧（58 号 ②③）；变异分诊不接广谱；共用约束六行 | `CHECKLIST_SPECS_BREAK=trust-follow` rc=1；58 号 red 样本 c995 两样都缺判红、green 样本 c996 判绿 |

## 改过的文件（我自己列；`git diff --stat` 对 HEAD 算，混着别的会话没提交的改动，分不出谁改的）

- 新建（未跟踪）：`.claude/agents/tooling-writer.md`、`.claude/agents/kb-spec-drafter.md`、`.claude/agents/investigator.md`、`.claude/hooks/handback-guard.sh`、`research/scripts/cite-check.py`、`research/scripts/kb-spec-check.py`、`research/scripts/crash-case-check.py`、`research/scripts/prove-red.sh`、`research/scripts/apply-writer-patch.py`、`research/scripts/closeout-status.py`；样本 `.claude/gate.d/fixtures/58-implementation-premise.sh/green/research/prompts/{_c996-r1-body.md,_c996-r1-checklist.md,_c996-r1-background.md,c996-r1-snapshot/kb-sha256.txt}`、`…/red/research/prompts/{_c995-r1-body.md,_c995-r1-checklist.md,_c995-r1-background.md}`、`.claude/gate.d/fixtures/63-agent-write-scope.sh/red/.claude/agents/delta.md`。
- 定点改（都经 `replace-once.py` 或同目录临时文件再 `mv`）：`.claude/agent-common.md`；14 份定义（implementation-writer、experiment-runner、experiment-designer、sweep、kb-scribe、prior-art、mutation-triage、three-way-forward / attack / defense / verifier / local-attack / local-defense / materials）；`.claude/hooks/{agent-write-scope.tsv,runner-dispatch-guard.sh,continuation-guard.sh,bash-command-detector.sh,ask-user-claim-guard.sh（只改 gate-similar 那一句）}`；`.claude/settings.json`（SubagentHandback 加一条）；`.claude/gate.d/{47,58,63}-*.sh`；样本 `58/{green,red}/expect`、`63/green/.claude/settings.json`、`63/red/expect`；`research/scripts/{agent-watch.py,watch.sh,watch.conf,checklist-specs.py,stale-candidates.py,corruption-check.py}`；`records/2026-09-16-subagent拆分提案.md` 第四十节插一行（第 49 行，`insert-row.py`）。
- 不是我改的、同目录里看到的：`63-agent-write-scope.sh/red/.claude/agents/{epsilon,zeta}.md`、`63/red/.claude/settings.json`、`58/red/research/prompts/_c997-r1-body.md` 是别的会话的，没碰。
- `git diff --stat`（对 HEAD，含别的会话的改动）末行：`36 files changed, 1684 insertions(+), 510 deletions(-)`，全文在 `/tmp/claude-1000/tooling-1/progress.md` 不再抄。

## 交主 agent 开定义三方的清单（门禁 72 号要判决按路径点名）

新定义 3 份：`.claude/agents/tooling-writer.md`、`.claude/agents/kb-spec-drafter.md`、`.claude/agents/investigator.md`。
改过的定义 14 份：`implementation-writer.md`、`experiment-runner.md`、`experiment-designer.md`、`sweep.md`、`kb-scribe.md`、`prior-art.md`、`mutation-triage.md`、`three-way-forward.md`、`three-way-attack.md`、`three-way-defense.md`、`three-way-verifier.md`、`three-way-local-attack.md`、`three-way-local-defense.md`、`three-way-materials.md`（都在 `.claude/agents/` 下）。
共用约束：`.claude/agent-common.md`（10 处：tooling-writer 例外、600k 交接、后台轮询拒、进度文件、拒绝写法 ②⑧、派发闸与续做闸与交回正文闸三条、绿行报数、报告进文件与 2000 字、Bash 不就地写）。
在跑的子 agent 手上要改的：派发闸新判法只影响主 agent 的派发；子 agent 这边新增的执行时拒绝是后台轮询 `tasks/*.output`（拒绝写法 ②）、有 Edit 的子 agent 在 Bash 里就地改仓内文件（⑧）、交回正文超 2000 字 / 三方腿引文对不上 / 书记员执行员绿行没数（交回正文闸）。在跑的执行员（E156、E158）、实现员（实分一、实审 A1、B1）交回时会撞交回正文闸，要提前告诉它们「全文进报告文件，交回只写结论、路径、sha256」。

## 各 lint 与 47 / 62 / 63 的末行与退出码（改完之后那一批，`lints.sh after1`，2026-09-27 JST 00:5x 起跑；基线那一批 `lints.sh base` 在开工时跑，47 / 62 / 63 / 58 / 73 / doc-lint / 规则纪律 / gate-lint / shell-lint / gate-overlap 都绿，preflight-lint 基线就红 295 处）

| 项 | 退出码 | 原样末行 |
|---|---|---|
| 47 号 | 1 | `→ 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。`（红两处，都不是这一轮的，见下） |
| 62 号 | 0 | `✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）` |
| 63 号 | 0 | `✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸与弹窗断言闸注册着、自证通过，定义的 model 取值认得，…表与定义一致（4 个有 Write 或 Edit 的定义、28 条路径模式）…` |
| 58 号 | 0 | `没查 0 份`（上一行 `标题日期 ≥ 2026-09-28、另核了本地腿一侧与开工快照的 0 份`） |
| 73 号（研究脚本、hook、.claude/scripts 的 gate-lint 与 shell-lint） | 0 | `没扫的目录 0 个（不在）：（没有）` |
| doc-lint | 1 | `✗ 文档铁律检查失败：1 个文件违规、0 处编号引用无定义、11 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 516，跳过 0）`（红全在 `.claude/kb/experiments/158-*.md`、`156-*.md`、`experiments-history.md`，UTC 15:33–15:36 别的会话的执行员写的，不是这一轮的） |
| 规则纪律（项目本地） | 0 | `✓ 规则只写怎么做（扫了 32 份文件 1985 行；没扫 0 个；…）` |
| gate-lint（`GATE_LINT_DIR=.claude/gate.d`） | 0 | `✓ 门禁自检通过：84 个脚本（.sh 与 .py）、302 条拒绝都带了出路` |
| shell-lint（`SHELL_LINT_DIR=.claude/gate.d`） | 0 | `✓ shell 纪律检查通过（共 76 个脚本）` |
| preflight-lint | 1 | `✗ 准入与运行条件：判了 99 个脚本（脚本 3、钩子 12、门禁阶段 84），291 处不合格；排除 0 个：无`（基线 295 处；少掉的是 runner-dispatch-guard.sh 补了文件头与 preflight 调用；新钩子 handback-guard.sh 0 处；剩下的都是基线就有的，门禁阶段 84 份与另 3 个钩子） |
| gate-overlap | 0 | `没判的：装进来的 SOP 副本 37 份只当对照（.claude/singlefs-ai-sop/scripts）` |

47 号红的两处：
- `check-segment-registry.py --selftest`（冻结文件，别的会话在改）：`未改动的拷贝本该判绿，实际退出码 1：… 第二条流 … 用例文件 crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs …`——红在 `crates/` 用例与 kb `layout/01` 对不上，我没碰这两处；单跑复现同样红。
- `mutate.sh --selftest`：单跑复现 `✗ 自检：MUTATE_JOBS=1 与 =4 的 stdout 应当逐字相同…实际不同` 与 `⏱ [没红] 2 秒没跑完`——它的自检用 2 秒超时，跑的时候整机负载 75（`uptime`：`load average: 75.19, 81.54, 49.85`，32 核），别的会话的测试二进制在跑；`mutate.sh` 我没改（工作区里它没有改动），基线那一批是绿的。判成环境，负载下来再跑一次。

新脚本的准入声明另在临时拷贝里登记 `research/scripts` 跑 preflight-lint：`✓ 准入与运行条件：判了 6 个脚本（登记目录 6），都在开头写明了条件并先判`（项目没有 `.claude/preflight-dirs`，仓里这一类本来就不判）。

## 报告里「推的」数，落在哪、现在是什么

| 数 | 落在哪 | 状态 |
|---|---|---|
| 实现员一件活 3 件 | `main-agent.md` 那一句（第二段规格第四节第 1 条） | 推的，没量过；派发闸只数文件，不数件 |
| 实现员一件活 8 个 crates 文件 | `runner-dispatch-guard.sh` 的 `IMPLEMENTATION_WRITER_FILE_LIMIT = 8`；`implementation-writer.md` 输入 | 推的，没量过 |
| 规格起草一次 8 条 | `kb-spec-drafter.md` 输入；第二段规格第四节第 4 条 | 推的，没量过 |
| opus 并发上限 | `runner-dispatch-guard.sh` 的 `OPUS_CONCURRENCY_LIMIT = 8` | **用户定**（2026-09-27 JST 02:2x 弹窗原话「8」，主 agent 转来的消息；报告原文的 4 已不用） |
| 上下文交接线 600k | `implementation-writer.md` 第 8 步、`tooling-writer.md` 第 8 步、`investigator.md` 第 6 步、`agent-common.md`「不做」读大文件那一条 | 推的，没量过 |
| 看门狗「该交接」700k | `agent-watch.py` `--context-tokens` 默认 700000、`watch.conf` 注释 | 推的，没量过 |
| 交回正文 2000 字 | `handback-guard.sh` 的 `HANDBACK_CHARACTER_LIMIT`、`agent-common.md`「报告」一节 | 推的，没量过（报告 p90 是 8002 字） |
| 崩溃验证提示线 5 个补丁 | `apply-writer-patch.py` 的 `CRASH_VERIFY_HINT_PATCHES` | 推的，没量过 |
| 宽检索词门槛 100 行 | `stale-candidates.py` 的 `WIDE_TERM_SPOT_CHECK_LINES` | 推的，没量过 |
| 进度文件不涨 30 分钟 | `agent-watch.py` `--progress-stale-minutes` 默认 30、`experiment-runner.md` 第 8 步 | 30 分钟来自用户 09-26 08:29「超过30分钟的任务 断点」与报告 3.2；「不涨多久报」这一用法是推的 |
| 单字母替换至少 5 次 | `corruption-check.py` 的 `SUBSTITUTION_MINIMUM_COUNT` | 推的（`UNTESFED` 那次 27 处） |
| 在跑判定的 3 小时窗口、没派出去的 10 分钟宽限、限额回看 8 天 | `runner-dispatch-guard.sh` 的 `RUNNING_RECENT_SECONDS`、`UNLAUNCHED_GRACE_SECONDS`、`LIMIT_WINDOW_LOOKBACK_SECONDS` | 推的，没量过（我起的，报告里没有） |
| inherit / 没有定义的类型按 opus 算 | `runner-dispatch-guard.sh` 的 `INHERITED_MODEL_FAMILY` | 推的：这个会话的主 agent 跑 opus |

## 做的时候撞到、改了判法的几处（给三方当线索）

- 派发闸在真会话上探了一次（主会话记录 226 MB，判一次 0.5 秒）：09-22 那次周限额写着「resets Sep 27, 4pm (UTC)」、之后 opus 照派没再撞，所以加了「失败之后同一族又派出去没再撞，算已解开」；不加的话现在派任何 opus 都会被拒到 09-28 01:00 JST。
- 书记员汇总「交回之后才报」：`agent-watch.py` 里早就是这样（`scribe_followup_report`，`AGENT_WATCH_BREAK=scribedefer` 那一格），报告写的「看门狗没改」与代码对不上，没重做。
- 58 号的新两条只管标题日期 ≥ 2026-09-28 的正文：门禁批第二轮（09-27）的正文在冻结快照里，没有「本地腿：攻 / 辩」一行，按 09-27 起判会当场红、又不许改。
- `checklist-specs.py` 的随行核对：`_m2-safety-r3-checklist.md` 用 `--extra` 取整份文件、随行没有父节，第一版把它判红了；改成「同一份文件有 --extra 取法的算带到」之后，三份真清单判绿。
- `crash-case-check.py` 第一版把层 0 二进制里的全量流也要求标 `#[ignore]`，而 54 号是整个二进制跑的，标了反而跑不到；改成层 0 二进制里的不判。仓里现查剩 1 处（`crash_enumeration_resumes_from_its_progress_file.rs:383`，写进第二段规格第二节）。

## 没做什么

- 第二段 11 份冻结文件一个字没改；改法全写进 `/tmp/claude-1000/tooling-1/phase-2.md`（`stage-owners.tsv` 归属四行与新阶段一行、新门禁阶段 `41-crash-case-registered.sh`、`heavy-test-guard.sh` / `lib_heavy_tests.py` 那几条、`main-agent.md` 十条、`gate-triage.md` 与 `crash-verifier.md` 的 required-inputs 与「谁来修」一列、`implementation-workflow.md` 第 70 行那一句）。
- 没走定义三方（门禁 72 号）；没跑重型测试（54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的目标、全量 `cargo test`、`check.sh`）；没提交。
- 没做的检查（报告里写成「推的写法」的）：核查员开工与收尾对冻结副本 `sha256sum -c`（3.5 第二行的会红检查）、判决里「采纳 / 定案」写明几轮站住的 56 / 72 号形式检查（4.6 第 8 条）、执行员写 `.claude/kb/experiments/` 之后由 `kb-scribe-followup.sh` 当场跑 doc-lint（3.2 第三行的会红检查；定义里改成了执行员自己跑）、看门狗「该交接」告警附「吃上下文最多的几笔」只按工具结果分类汇总，没到单笔。
- `bash-command-detector.sh`、`continuation-guard.sh` 基线就缺 `admission:` / `run-condition:` 与 preflight 调用（每条 Bash 都要过前者，加 preflight 会给每条命令多一次 python 启动），没补，照基线留着。
- 仓里 `.claude/kb/experiments/156`、`158` 与 `experiments-history.md` 的 doc-lint 红、`check-segment-registry.py --selftest` 的红，不是这一轮的，没碰。
- 草稿目录里没有编译目录与仓副本；`fx.sh` 起的临时目录跑完都删了。

## 最终版本的自检（改完最后一处之后跑）

- `python3 research/scripts/agent-watch.py --selftest`：退出码 0，新加的那一组查了 10 格（文件 sha256 `3a244f3ee21a416807d5b77754e99e6a55cc607790bccf1a9e56b7c7d65f64bd`）。
- `bash research/scripts/watch.sh --selftest`：`✓ watch.sh 自检通过：…`，退出码 0。
- `bash .claude/hooks/runner-dispatch-guard.sh --selftest`：`✓ 自检通过（查了 98 种）…`；`bash .claude/hooks/handback-guard.sh --selftest`：`✓ handback-guard 自检通过（查了 14 种）…`；63 号 0；58 号样本 red / green 判得对。
- 6 个新研究脚本的 `--selftest` 各退 0，已挂进 47 号的 runner 表（47 号这一批红的两处不是这一轮的，见上）。
