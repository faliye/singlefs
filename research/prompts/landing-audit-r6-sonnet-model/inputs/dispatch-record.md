# 2026-10-22 子 agent 结果落地审计

## 课题与出口

课题：10 月以来（2026-10-01 起，按会话记录时间戳切）派出的子 agent，交回的结论与改动最后产生了什么效果：进了代码或文档、被后续动作用上、被覆盖、还是丢了。数据用来改流程。

出口：一张按类型 × 结局的分布表（个数与折算花费占比），每个结局桶有抽查核过的判定与原因分类；列出流程上的缺口，每条带数据出处。

## 机械层（主 agent 跑，草稿 `/tmp/claude-1000/-home-fy5090-code-singlefs/412f440a-3f86-449e-adc8-1563bd743223/scratchpad/audit/`）

- 子 agent 2230 个（`~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/*.meta.json`，首条记录时间不早于 2026-10-01）。
- 写入落地：子 agent 每次成功的 Edit / Write 的新增行（≥12 字、去掉 old_string 里已有的），比对 2026-09-25 以来 git 历史加进的行与当前工作树的行（哈希集合）。
- 被用上：交回之后，主会话与后来的子 agent 的工具调用里有没有点到它交回里写的路径或它的 agent 号。
- 花费：照 `scripts/agents/agent-cost.py` 的折算口径逐个子 agent 算。

## 依赖表

| 任务 | 类别 | 依赖 | 输入齐没齐 | 批次 |
|---|---|---|---|---|
| 1 写入落地率的假阳性与部分落地抽查 | clerk | 机械层 | 齐 | 1 |
| 2 写回员规格逐条落地抽查 | clerk | 机械层 | 齐 | 1 |
| 3 调查员与回扫员结论落地抽查 | clerk | 机械层 | 齐 | 1 |
| 4 没产生效果的几桶抽查原因 | clerk | 机械层 | 齐 | 1 |
| 5 返工链的原因 | clerk | 机械层 | 齐 | 1 |

派发上限 5，截止批次 1；批内冒出的发现只进下一轮清单。输入在 `~/.local/share/singlefs/work/subagent-landing-audit-2026-10-22/inputs/`。

## 结果

五份抽查报告原样存进 `research/prompts/subagent-landing-audit-2026-10-22/`（writers.md、kb-scribe.md、investigator-sweep.md、no-effect.md、rework.md），每份的关键断言主 agent 都现查核过：提交号、文件:行、仓里的原句。

### 按结局分（2230 个子 agent，花费为折算口径）

| 机械层的桶 | 个数 | 花费占比 | 抽查结论 |
|---|---|---|---|
| 写了东西、≥70% 的行进了提交或工作树 | 663 | 42.0% | 抽 22 个全部进过仓；机械落地率比实质改动的落地率平均偏高 1 到 4 个百分点；只证明「进过」，不证明「还在」 |
| 写了东西、30–70% | 61 | 3.1% | 抽 10 个全部进过仓，偏低的原因是提交前又被改写 |
| 三方腿 | 473 | 28.1% | 498 条腿里 494 条的报告在仓里、所在轮有判决；2026-10-01 以来的判决全部被 kb、records 或豁免表引用；另 4 条是 m5-dev 那条线在跑的轮 |
| 只交报告、交回后主 agent 有动作点到 | 671 | 17.5% | 调查员抽 18 个：13 个修复落了地、5 个结论记进了文档；回扫员建议改 502 行、已改 479 行；写回员可核的 1120 项里 1085 项现在在或进过提交，33 项在提交前被后续写回整块换掉 |
| 看起来没效果的几桶 | 约 240 | 约 7% | 抽 40 个：27 个其实产生了效果（机械层没认出）、10 个被后继者重做、2 个主 agent 有意弃用、1 个丢了；按桶外推（推的），真正白花约占全部花费的 1.1%（这是机械层那一版、按抽样外推的数；结局账那一版门槛换值见节「第三批交回」结局账那一条，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-1） |

结论：交回的结论与改动丢失的不多。花费大头的损失在返工：描述带返工字样的派发 337 个，占花费 13.8%（这个数是主 agent 当时的宽词表数的，含单字「续」「接着」「again」，不是这里列的五个词；更正与改后的口径见节「第三批交回」任务 13 那一条，判决 `research/prompts/landing-audit-r1-main-verification.md` K4-2）；抽的五条实验派发链 101 次里，本可避免 25 次（这几条链花费的 14.6%），部分可避免 22 次。返工字样那个数是关键词计数，误报漏报都有，不当损失的上界读（第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-4）。

### 流程缺口（每条带出处）

| 缺口 | 数据 | 出处 |
|---|---|---|
| 延后项只写进调度记录，没进欠账表，后来没人接 | 2 件：doc-experiments 跨实验比对正式跑、单测单元表、replay.sh 三处的那一格（`records/2026-10-11-里程碑三待优化上下游协调.md` 记「下轮开」，今天 `.claude/gate.d/doc-experiments.sh` 的 12 格里没有，`.claude/kb/milestone/03-third-txn.md` 那一行仍写「跨实验的检查没立」）；一条调查给的两条改法（`records/2026-10-13-里程碑四预开发.md` 记「延后」，`checks-owed.md` 里没有对应行） | no-effect.md「依据：丢了」；investigator-sweep.md |
| 回扫判出的 `crates/` 注释改动没人接：写回员不改 `crates/`，主 agent 只把文档行派给写回 | 7 行，今天仍原样在 checker 档的源码与测试里 | investigator-sweep.md #29 |
| 回扫与写回之间载体搬了家，写回员按旧路径找不到就不写，也不报 | 4 行（`.claude/kb/invariants.md` 拆进 `crates/singlefs-checker/invariants/` 的同一天） | investigator-sweep.md #29 |
| 执行员写装置不对照登记逐条自查（`experiment-runner.md` 第 3b、4b 步只管停机对拍与产物条数） | 10 次续派、42.46 百万 | rework.md K4 |
| 设计员登记前不核可跑性与判别力（`experiment-designer.md` 第 3 步没要求） | 9 次、30.24 百万 | rework.md K3 |
| 执行员写范围不含实验驱动，驱动改一次多派一次 | 7 次（多数部分归它）、18.07 百万 | rework.md K7 |
| 命名 lint 交回前不跑 | 3 次 | rework.md K1 |
| 实验占号不跨工作树 | 2 次，E185 到 E187 撞号 | rework.md K9，`scripts/kb/claim-experiment.sh` |
| 修红：前一个实现员或合入带进来、到 checker 档才露的红 | 抽 15 个修红里 7 个；前一步明确拦得住的 4 个、2.90 百万 | rework.md K11 |
| 中途停下没交回的子 agent | 截断 38 个；交接摘要接手后大半半成品被用上，白做的约 2 到 4 百万 | no-effect.md「原因归纳」 |
| 报告写文件的要求互相矛盾：工具层拒子 agent 用 Write 写报告（「Subagents should return findings as text, not write report files」），`handback-guard.sh` 又拒超过 4000 字的交回、要写文件 | 这一轮一个子 agent 撞上，改用 Bash heredoc 写成 | no-effect.md 开头 |
| 结局没有常设的账：落地与否、被谁用上、返工几次都要事后从会话记录里重建 | 这一轮靠一次性脚本重建 | 机械层 |

### 机械层的已知偏差

- 「交回后主 agent 有没有点到」漏判多：主 agent 用变量、通配、循环拼路径，交回正文本身就是结论，写回员直接写主工作区，都认不出（no-effect.md「机械层为什么把这 27 个漏判」）。
- 逐行比对漏算 Bash 与 python 写的改动（writers.md「机械层没算到的产出」）。
- 按描述关键词数的实验派发链混进了同号的另一个实验（E187、E185）。

## 第二批：用户定案之后

用户 2026-10-22 定：丢掉的那一格检查现在就派人补；九条缺口全改，改之前先评估每条的可靠性与真实性。

### 主 agent 现查（机械）

| 缺口 | 现查结果 |
|---|---|
| 报告写法矛盾 | 真，且普遍：「Subagents should return findings as text」不在仓里任何钩子里，是 Claude Code 自带的拦截；220 份子 agent 会话里出现过，被拒的是 Write，目标 210 次是 `report.md`；2026-10-14 起子 agent 往草稿目录写 `report.md` 209 次全被拒，写 `progress.md`、`halt.md` 这类别的名字都成功，名字里带 report 的别的文件 16 次里拒 6 次 |
| 执行员写范围不含驱动（K7） | 过时：7 次都在 2026-10-01 到 2026-10-04，驱动当时在 `research/scripts/`；脚本搬进 `scripts/` 之后驱动在 `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh`，`.claude/hooks/agent-write-scope.tsv` 放行执行员写 `scripts/experiment-apparatus/**`。不改 |
| 命名检查交回前不跑（K1） | 真：`.claude/agent-common.md@门禁` 把扫仓的检查都留到提交前；`.claude/agents/` 下没有一份定义提交回前跑命名检查 |
| 占号不跨工作树（K9） | 真：`scripts/kb/claim-experiment.sh` 只查 `git rev-parse --show-toplevel` 那一个仓根 |
| 结局没有常设的账 | 真：`scripts/agents/agent-cost.py` 只有 usage、timing、dispatch 三个子命令 |

### 依赖表

| 任务 | 类别 | 依赖 | 输入齐没齐 | 批次 |
|---|---|---|---|---|
| 6 doc-experiments 加一格：重跑登记里的每条正式跑在单测单元表与 `replay.sh` 都找得到 | tooling-writer | 无 | 齐 | 2 |
| 7 返工归类盲判（不给前一份结论），算一致率 | clerk | 无 | 齐 | 2 |
| 8 交接类三条缺口有多普遍：延后项去向、回扫的 `crates/` 行、载体搬家 | clerk | 无 | 齐 | 2 |
| 9 改定义、共用约束、钩子与脚本（核实为真的那几条） | tooling-writer | 7、8 | 否 | 3 |

### 第二批交回

- 任务 6（补那一格）：停在出口 1。比对对象表只在 `research/prompts/e165-r1-prereg.md` 到 `e165-r5-prereg.md` 里有，五份五种写法；单元表是 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 里 `formal_run_units` 数组；只有 `scripts/experiment-apparatus/replay.sh` 有固定形态（E165 26 行）。主 agent 现查属实。用户定：先立固定形态再立格。报告存 `research/prompts/subagent-landing-audit-2026-10-22/doc-experiments-cell-stop.md`。
- 任务 7（盲判，`research/prompts/subagent-landing-audit-2026-10-22/blind-rework.md`）：E171、E188 两条链 42 次续派，与 rework.md 比，主因一致 37 次；可避免与否（Y 或 P 对 N）一致 33 次，不一致的 9 次里 7 次是 rework.md 判 N、盲判判 P，rework.md 偏保守。K4 在这两条链里 rework.md 点 8 次、盲判点 6 次，重合 6 次；K3 rework.md 点的 4 次盲判全中；盲判在 E188 另归出 3 次，其中 E188 #6、#7 rework.md 归给另一条缺口（K10 登记判据当场报了问题仍先跑整格），E188 #8 rework.md 判计划内，不是盲判独立找出的新情况（判决 `research/prompts/landing-audit-r1-main-verification.md` K4-3）。盲判另归出一类：主 agent 写派发提示没套用已有规则与用户常设要求（E171 #19、#20、#21、#31、#33），记进下一轮清单。
- 任务 8（`research/prompts/subagent-landing-audit-2026-10-22/handoff-gaps.md`）：延后项无去向 8/54（10 月 records 32 份，固定种子 20261022 抽样），延后当时就登进欠账表、里程碑收口表或决策正文的 6/54；回扫判出的 `crates/` 行今天原样 16/129，知识腐烂回扫顺带判出的 15 行里 8 行原样、实现前盘点与代码回扫的 102 行里 4 行原样；载体搬家后写回落空 5/9，写的一方报了的 0。主 agent 现查：`scripts/kb/stale-candidates.py` 的 `CARRIER_PATTERNS` 不含 `crates/singlefs-checker/invariants/`，`.claude/hooks/agent-write-scope.tsv` 里写回员只有 `.claude/kb/**` 与工作根两行。

### 九条缺口的判定

| 缺口 | 判定 | 依据 |
|---|---|---|
| 执行员不对照登记逐条自查（K4） | 真 | 盲判独立复现 6/8（8 次全在 E171、E188 两条派发最多的链上，证的是归类一致、不证普遍，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-5） |
| 设计员不核可跑性与判别力（K3） | 真 | 盲判独立复现 4/4（同两条链，证的是归类一致、不证普遍，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-5） |
| 执行员写范围不含驱动（K7） | 过时，不改 | 主 agent 现查 |
| 命名检查交回前不跑（K1）、占号不跨工作树（K9） | 真 | 主 agent 现查定义与脚本 |
| 延后项没进欠账表 | 真，普遍 | 8/54 无去向，只有 6/54 当时就登记 |
| 回扫判出的 `crates/` 行没人接 | 真，集中在知识腐烂回扫 | 8/15 原样 |
| 载体搬家后写回落空 | 真；病根是候选扫描与写回员写范围都没跟着搬家改 | 5/9 静默跳过；主 agent 现查两处；9 行出自同一次搬家，频率上是一次事件（第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-5） |
| 报告写法矛盾 | 真，普遍 | 2026-10-14 起子 agent 写 `report.md` 209 次全被拒 |
| 结局没有常设的账 | 真 | 主 agent 现查 |

### 第三批依赖表

| 任务 | 类别 | 关的缺口 | 文件 | 批次 |
|---|---|---|---|---|
| 10 立「重跑比对对象」机读登记与 doc-experiments 一格；占号跨工作树；候选扫描认不变量清单 | tooling-writer | 丢掉的那一格、K9、载体搬家（扫描一侧） | `.claude/gate.d/doc-experiments.sh` 与 fixtures、`.claude/gate.d/replay-comparison-units.tsv`（新）与滞后表（新）、`scripts/kb/claim-experiment.sh`、`scripts/kb/stale-candidates.py` | 3 |
| 11 定义与共用约束的文字 | tooling-writer | K4、K3、K1、延后项、回扫的 `crates/` 行、载体搬家（写回一侧）、报告文件名 | `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`experiment-designer.md`、`kb-scribe.md`、`sweep.md`、`implementation-writer.md` | 3 |
| 12 钩子与派发脚本 | tooling-writer | 报告文件名、写回员写范围、K1 放行 | `.claude/hooks/agent-write-scope.tsv`、`runner-dispatch-guard.sh`、`write-guard.sh`、`handback-guard.sh`、`scripts/agents/agent-watch.py`、`cite-check.py`、`dispatch-prompt.py` | 3 |
| 13 结局账 | tooling-writer | 结局没有常设的账 | `scripts/agents/agent-cost.py` | 3 |

### 第三批派出后看门狗的检出（只记，不处置）

- 定义与共用约束那一位：想跑 `rules-lint.sh` 与 `stage-selftest.sh --item doc-experiments.sh`，钩子以「提交时才跑的检查」拒了；`s2/setup.sh` 不存在时重型测试闸没判全。
- 门禁与脚本那一位：`stage-selftest.sh` 被拒，原因是 doc-experiments.sh 的样本与 HEAD 加暂存区不同（新样本还没暂存）；`run-sample.sh` 不存在时重型测试闸没判全。
- 钩子与派发脚本那一位：`cp` 目标含命令替换，写范围算不出、没拦（目标在它自己的草稿目录 `before/` 下）。

### 第三批交回

- 任务 11（定义与共用约束，`research/prompts/subagent-landing-audit-2026-10-22/tw-definitions.md`）：七份改完，对暂存区 20 行加、12 行减。主 agent 审了 diff：执行员第 3b 步挪走的「一次做不完写部分已跑」那半句在新的 3c 步末尾，没丢。它造的输入里，K4、K3、延后项、回扫的 `crates/` 行四条现有门禁与钩子都不会红，要立的检查写在它的报告里，进欠账；K1 现在会被 heavy-test-guard 拒，等任务 12 放行。rules-lint、doc-lint 被拒，读数归提交前整轮。
- 连带：`scripts/gate/apply-writer-patch.py` 认补丁目录里的 `report.md`，改名之后实现员交 `result.md` 会被它拒。不在任何一个工具员的文件清单里、挡这一批，主 agent 自己改了四处（文件头说明、读报告的路径、拒绝句、自证样本），`--selftest` 绿（查了 15 种），`APPLY_WRITER_PATCH_BREAK=checker-section-ignored` 下自证红在「动 checker 而报告没那一节」。
- 任务 10（门禁与脚本，`research/prompts/subagent-landing-audit-2026-10-22/tw-gate-scripts.md`）：新格 replay-comparison-units、登记表 `.claude/gate.d/replay-comparison-units.tsv`（E165 r5 的 14 行）、滞后表 `.claude/gate.d/replay-comparison-units-lag.tsv`（4 行）、`.claude/gate.d/stage-owners.tsv` 加一行；`scripts/kb/claim-experiment.sh` 查 `git worktree list` 的每一棵；`scripts/kb/stale-candidates.py` 认不变量清单。工具员的 stage-selftest 与真仓单跑被闸拒（样本没暂存）。主 agent 没暂存（用户要求改完停在未暂存），改成手工重放，依据是用户 2026-09-28 的授权「别再问我授权操作了 授权你所有操作」，命令带 `SINGLEFS_HEAVY_TESTS=user-request`：
  - 真仓单跑新格：绿，「比对对象 14 行（E165）……复跑写驱动名的 10 行在 replay.sh 里都有且产物名相符；复跑写 none 的 4 行都登记在 replay-comparison-units-lag.tsv 里」。
  - 三个样本拷进临时目录、跑 setup.sh、以样本根起整道（`.gate-cells` 只点新格）：红样本退 1、绿样本退 0、零行样本退 77，`expect` 里的片段全部在。
  - 八个弄坏开关逐个打开：七个在红样本上各缺它点名的片段（缺 1、3、1、1、1、1、2 条），`comparison-unit-substring` 让绿样本退 1。全部判出。
  - `claim-experiment.sh --selftest` 绿，`CLAIM_SKIP_WORKTREES=1` 退 1；`stale-candidates.py --selftest` 绿，`STALE_CANDIDATES_BREAK=invariants-carrier-missing` 退 5，红在「不变量清单里的现状句没进候选表」。
  - 它留的两件 E165 的事（r5 比对对象表第 15 至 19 行在单元表里没有单元；`replay.sh` 里 driver_e165 那三行的期望产物仍是 2026-10-01 的原产物，单元表注释写的是已归档的 r5 产物）与定义那一路列的「要立的检查」，等第三批交齐后合成一份规格交写回员登进欠账表（三语一起），编号回填到这里。`.claude/hooks/selftest-registry.tsv` 里 claim-experiment.sh 那行的理由文字过时，等任务 12 交回后改。
- 任务 12（钩子与派发脚本，`research/prompts/subagent-landing-audit-2026-10-22/tw-hooks.md`）：五份钩子与脚本里认 `report.md` 的都是自证样本，改成 `result.md`；`scripts/agents/dispatch-prompt.py` 骨架默认 `<草稿目录>/result.md`；派发闸新判法 ⑰ 落在 `.claude/hooks/runner-dispatch-guard.sh`（「报告：」一行的文件名带 report 就拒）；写范围表加两行（写回员写 `crates/singlefs-checker/invariants/*.md`、执行员写 `.claude/gate.d/replay-comparison-units.tsv`）；K1 放行落在 `.claude/hooks/lib_heavy_tests.py` 的 `classify_gate_stage`（只放行 `--check` 全在四个命名格之内的）。主 agent 在主工作区重跑：runner-dispatch-guard 自证 437 种、heavy-test-guard 1386 种、write-guard 228 种，都绿；`RUNNER_DISPATCH_GUARD_BREAK=report-name-unchecked` 红 10 格；`LIB_HEAVY_TESTS_BREAK=naming-self-check-any-cell` 红在「点名里夹着别的格照算」「只点名别的格照算」两格。它带声明实跑那条命名检查：22 秒，vague-names、ordinal-names、data-names 三格在仓里有存量红。
- 主 agent 收尾两处：`.claude/agent-common.md`「命名检查交回前跑」一条补「输出不重定向到 `.log`，要落盘经 `scripts/hosts/two-logs.sh`」；`.claude/hooks/selftest-registry.tsv` 里 claim-experiment.sh 那行理由补「另建两棵工作树核跨工作树查号」。
- 任务 13（结局账，`research/prompts/subagent-landing-audit-2026-10-22/tw-outcome-ledger.md`）：`scripts/agents/agent-cost.py` 加 `outcome` 子命令，修了引用与 Bash 写入两处漏判。它报返工字样复现不出 337：主 agent 现查，337 是主 agent 当时用的宽词表（含单字「续」「接着」「again」）数的，多算了「后续批」这类约 3 个；它只用了调度记录里写出的五个词，数出 155 个、5.6%，漏了重派、接手、接续、fix、补写、改钉值。主 agent 把 `REWORK_WORDS` 换成严格词表（文件头列全），自证加一项「返工词表」（接续、接手、重派、redispatch、fix、第 N 次算，「后续批」「第 1 次」不算），开关 `AGENT_COST_BREAK=narrow-rework-words` 红在这一项；`--selftest` 12 项绿，四个开关各只红在点名那一项。
- 结局账在真会话目录上的正式数（`python3 scripts/agents/agent-cost.py outcome --since 2026-10-01 --summary --jobs 12`，查了 2268 个子 agent，折算合计 7623.33 百万）：写了且不低于 70% 进了提交或工作树 888 个、49.8%；30% 到 70% 90 个、4.3%；三方腿 473 个、27.5%（按类型名记有效，不核报告在不在仓、同轮有没有判决，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-2）；只交报告、主 agent 交回后点到 661 个、15.3%；看起来没效果的几桶 91 个、2.0%（70%、30%、10 行三道门槛没有出处；下沿 30% 抬到 50% 时是 3.4%（这组换值读数是在 10-01 到 10-21 闭窗口那一版上量的，不是这 2268 个；第三轮攻方在东京日期那一版上复出，只 (90%, 70%) 那一格 6.2% 变 6.1%），「写了且不低于 70%」随上沿从 70% 换到 90% 由 50.0% 掉到 35.0%，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-1）；其余 65 个、1.2%。返工字样 353 个，占个数 15.6%、花费 14.1%（关键词计数：`\bfix` 命中 fixture、单字「修」多半是新活，是误报；「补跑」「第二段」不在词表，是漏报；不是返工损失的上界，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-4）。没交回 116 个：中途停下 36、API 错 2、停掉 73、还在跑 5。这一组数取代「按结局分」那张表的机械层读数。

### 延后项的登记

- 子 agent 与协作工具的五条：`records/2026-09-16-subagent拆分提案.md` 第九十节 ①到⑤（执行员登记落地清单的检查、设计员挂钟与态数的核对格、延后项带编号的格、回扫「交实现员」一节的报告检查；主 agent 派发提示套用常设要求，还没想清做成哪种检查）。
- 文件系统一侧两条，三语同改：`.claude/kb/checks-owed.md` C879（E165 根选择与杀伤界正式跑没有单元）、C880（E165 复跑行期望的是原产物）；`.claude/kb/milestone/03-third-txn.md` 节「待优化：流程、门禁与装置上踩过的坑」第 3 行现状改成「跨实验的检查已立」。写法：`scripts/kb/insert-row.py` 与 `scripts/kb/replace-once.py`，锚点各命中一次，回读确认。
- 门禁 doc-text 的 trilingual-structure、trilingual-glossary 两格单跑都红：点名 `.claude/kb/checks-owed` 的日语版链接只有 2 个、8 条术语没译，主 agent 逐条比过，没有一条的术语出现在这次写的中文里，`03-third-txn` 一条都没点到；是存量，不在这一批修，提交前整轮再判。

## 第四批：这一批改动的三方（landing-audit-r1）

用户 2026-10-22 定「走一轮三方」，并要求看「能不能合并、有没有和之前的脚本有重合」「有没有矛盾」。正文 `research/prompts/_landing-audit-r1-body.md`（格 K1 改法做没做到缺口，K2 合并与重合，K3 矛盾，K4 数能不能核），背景材料 `research/prompts/_landing-audit-r1-background.md`，开工快照 `research/prompts/landing-audit-r1-snapshot/`（被判 51 份文件的 sha256 与对暂存区的 diff，diff sha256 a68200ba…）。

| 任务 | 类别 | 格 | 批次 |
|---|---|---|---|
| 14 正推腿 | three-way-forward | K1 | 4 |
| 15 攻方腿 | three-way-attack | K2、K3、K4 | 4 |

派之前主 agent 看到一处：`scripts/agents/dispatch-prompt.py three-way-forward` 的骨架「报告：」一行默认填 `<草稿目录>/result.md`，三方腿的报告却要写进 `research/prompts/<轮>-*-output.md`（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）；这次派发照规则写 output.md，交攻方腿在 K3 里判。
- 开工快照之后，里程碑三收尾那个会话（singlefs-18）的第七批写回改了 `.claude/kb/checks-owed` 与 `.claude/kb/milestone/03-third-txn` 的三语六份（规格 `~/.local/share/singlefs/work/m3-close-b16-text-batch-7/spec.json`，同目录 result.md 末节有逐文件前后 sha256）。主 agent 现查：这一轮被判的 C879、C880 两行与「待优化」第 3 行现状一句三语都还在、不含它第八批要动的 C397 与「全域 0 红」；回它第八批不挡，可以带「快照冲突已判」派。判决时照那份规格倒推快照时的原样。
- 攻方腿跑 `.claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 被 heavy-test-guard 当提交时才跑的检查拒了（看门狗检出）。主 agent 照用户授权自己跑，输出存 `research/prompts/landing-audit-r1-snapshot/gate-overlap-list.txt`（58 行），判决时用它核攻方 K2。
- 主 agent 另对第三批四个工具员的会话记录各跑 `gate-overlap.py --touched-by`（输出 `research/prompts/landing-audit-r1-snapshot/gate-overlap-touched.txt`）：四份都退 77「这个会话没有新加或改动门禁与钩子——无对象可判」。现查原因：门禁那一位改脚本是写 `.new` 再 `mv` 换上，钩子那一位走 `scripts/gate/hook-swap.py`，`gate-overlap.py` 的 `WRITING_TOOL_NAMES` 只认 Write、Edit、MultiEdit、NotebookEdit 与 Bash 里几种直接写的形态，认不出这两种项目规定的换文件写法。收工钩子 `gate-reuse-check.sh` 因此对这类改动无对象可判；提交前的「门禁查重」阶段按 diff 窗口判，仍罩得住。`gate-overlap.py` 在上游，照 CLAUDE.md「规范从哪来」只能在上游填内容，记进下一轮清单。

### 第一轮判决与第五批

判决 `research/prompts/landing-audit-r1-main-verification.md`：正推 K1 采四条、不采一条；攻方 K2 采六条（K2-7、K2-8 不采）、K3 采九条、K4 采四条。主 agent 复跑：派发闸「报告路径：…/report.md」退 0 放行；单元表 19 个、登记 13 个；`gate-overlap.py .` 8 段整段相同全在 `lib_two_logs_guard.py` 与 `lib_run_scope.py` 之间（不是这一批的文件，提交前整轮判）。用户定 K2-1「七处一起并」。

| 任务 | 类别 | 关的条（判决节「二、逐格判」） | 文件 | 批次 |
|---|---|---|---|---|
| 16 门禁与扫描 | tooling-writer（opus） | K2-1 七处、K2-2、K2-3、K2-5 读登记位、K3-6 登记表与归属行、正推 10 | `.claude/gate.d/doc-experiments.sh` 与 fixtures、`.claude/gate.d/lib-lag-baseline.py`（新）、`.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/replay-comparison-units-lag.tsv`（删）、`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/doc-process-records.sh`、`.claude/gate.d/code-source-discipline.sh`、`.claude/gate.d/code-tooling.sh`、`.claude/hooks/lib_long_run.py`、`scripts/kb/stale-candidates.py` | 5 |
| 17 定义文字 | tooling-writer | K3-2、K3-3、K3-4、K3-5、K3-6 定义一侧、K3-7、K3-8、K3-9 | `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`experiment-designer.md`、`sweep.md`、`tooling-writer.md`、`three-way-forward.md`、`three-way-attack.md`、`three-way-defense.md` | 5 |
| 18 钩子与协作脚本 | tooling-writer（opus） | K2-4、K2-6、K3-1、K3-6 写范围表、K4-1、K4-2 | `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/continuation-guard.sh`、`.claude/hooks/agent-write-scope.tsv`、`scripts/agents/dispatch-prompt.py`、`scripts/agents/agent-cost.py`、`scripts/agents/agent-watch.py`、`scripts/agents/` 下新的会话记录库 | 5 |
| 19 第二轮三方 | three-way | 攻第一轮修补本身与产生结论的方法 | — | 6 |
- 主 agent 照判决改的四件：第 42 行 337 加注指到更正；「另多 3 次」改写（任务 7 那一条与「九条缺口的判定」表）；结局账那一次 `--summary` 的原样输出存 `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-since-2026-10-01.txt`（就是节「第三批交回」里「结局账在真会话目录上的正式数」那一组数的出处；那条命令只有 `--since`，后来开工的子 agent 会进来，原样复跑要等第 18 号任务加 `--until`）；限度：延后项 8/54 与回扫 `crates/` 行 16/129 的输入在 `~/.local/share/singlefs/work/subagent-landing-audit-2026-10-22/`，随草稿目录一起删了，这两个数只核得了算术（攻方 K4-4 核过），复跑不了。
- 判决节「三、判决」第三条那五条进 `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑥ 到 ⑩。第一次插行时主 agent 用编号当 `--absent` 与下一行的锚点：⑥、⑦ 因别的小节已有同号的行被拒，锚点照样挪到「| ⑥ |」，把 ⑧ 到 ⑩ 插进了第八十一节（别的会话的小节）。当场用 `scripts/kb/replace-once.py` 把那三行原样拿掉，与插之前的备份逐行比只差那三行；再用整段开头文字当锚点、有一行失败就停的脚本插回第九十节。
- 任务 17（定义文字，`research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md`）：九份共 17 增 17 删。主 agent 对着它的开工前原样逐份 diff 审过：K3-2 报告文件名两处、K3-3 执行员第 6 步与「没做什么」、工具员第 7 步各加「命名检查除外」、K3-4 rsync 副本交主 agent 跑那一句在、K3-5 第 2、7 条加拆分提案、K3-6 执行员第 5 步与写范围补比对登记表、K3-7 设计员 3a 的 ② 换成翻面观测、K3-8 3c 加「待主 agent 证红」、K3-9 回扫与派发表、攻方辩方 `required-inputs` 补「报告」。它报：攻方、辩方骨架因补了「报告」会多打一行默认 `<草稿目录>/result.md`，对三方腿是错的形态，由任务 18 在 `dispatch-prompt.py` 里改。
- 任务 16（门禁与扫描，`research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md`，sha256 `dcb4ae72bdcd290489984e3a74ea84c71fc87a6be4893a2c7511be8c2dad23ff`）：15 份 223 增 278 删，新建 `.claude/gate.d/lib-lag-baseline.py`，删 `.claude/gate.d/replay-comparison-units-lag.tsv` 与两份样本里的同名表。主 agent 现查：库自证 8 项过，`LAG_BASELINE_BREAK=unreadable-as-absent` 下 3 项红；K3-6 那 6 个不登记的单元对着 `research/prompts/e165-r5-prereg.md` 第十一节比对对象表 19 行核过，没有 r3 合法一族、没有 r5 档；`stage-owners.tsv` 那一行与登记表文件头 diff 审过。真仓单跑（`SINGLEFS_HEAVY_TESTS=user-request`，依据：用户授权所有操作）：doc-experiments 的 verdict-false-named、multipath-registry、replay-comparison-units 绿（比对对象 14 行、none 4 行只缩不涨），results-cited、quoted-result-lines 红；doc-process-records 两格红；code-source-discipline 两格红。四格红与改前那一版（暂存区里的门禁拷进草稿、对着同一棵树跑）输出逐行相同（doc-process-records 只差改前副本缺钩子目录那一行 cd 报错），不是这一批带来的（这种对照只走到「基准取得到、表在」一种结局与今天表里有的行形，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-6）；其中 agent-def-adversarial-review 点名的三份定义是任务 17 改的，归第二轮判决的「## 这一轮被判过的文件」。样本档 `stage-selftest.sh` 要样本与暂存区相同，归提交前整轮。
- 任务 16 报的 kb 残留：`.claude/kb/milestone/03-third-txn.md` 与 en、ja 两份第 185 行还写着删掉的滞后表，主 agent 用 `scripts/kb/replace-once.py` 三语各改一处，改成复跑列写 `none:` 的行与门禁基准比只缩不涨。它报的第八处同一写法与库自证不在 `.claude/hooks/selftest-registry.tsv` 两条延后，登在 `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑪、⑫。
- 搬迁：results-cited 把这一轮归档的子 agent 报告当成没被实验页点名的产物判红（`research/results/` 只放实验产物，`.claude/rules/directory-layout.md`）。主 agent 把整个目录 `git mv` 到 `research/prompts/subagent-landing-audit-2026-10-22/`（15 份，其中 12 份已被别的会话暂存），全仓引用照 `.claude/rules/path-moves.md` 改：拆分提案 4 处、本记录 10 处、`records/2026-10-18-里程碑三收尾与近期欠账.md` 1 处、`landing-audit-r1` 的攻方报告 8 处、正推报告 1 处、判决 1 处、攻方模型 `tools/rerun.sh` 2 处、归档报告 `blind-rework.md` 与 `tw-gate-scripts.md` 各 1 处；改完旧路径 0 命中，新路径点名的 11 份都在。这几份文件的 sha256 因此与第一轮当时对不上，只差路径。
- 第五批派出后看门狗的检出（只记，不处置）：任务 16 起的几条脚本与 `cp`、`>` 的目标落在它草稿目录的 `mktemp -d` 里，重型测试闸与写范围闸判不了；任务 18 写 `scripts/agents/subagent_transcript.py` 时描述里写了钟点被拒一次。另报一条误报：主 agent 读后台输出的 `cat`（常驻 0.00 GiB）被报「进程常驻内存过线」，阈值取的是峰值表里这条命令的峰值 0.00 GiB 的 1.5 倍，即 0；峰值为 0 的条目当阈值用是看门狗的缺陷，登在拆分提案第九十节 ⑬。再一条：主 agent 跑门禁 code-tooling 那几格时，看门狗把 research-script-selftests 格里自证开关轮故意判红的输出报成「长跑里有判红的行」，几十条，`watch.sh --processes` 起来就退；登在 ⑭，主 agent 不再为这一趟重起它，等两条后台命令自己的完成通知。
- 任务 18（钩子与协作脚本，`research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md`，sha256 `8dd0bcb3f38c3214b78afa8102eab9b838b4c9e9b1a6755c62171486018ec8f7`）：`runner-dispatch-guard.sh`、`continuation-guard.sh`、`agent-write-scope.tsv`、`selftest-registry.tsv` 一行、`scripts/agents/` 下 `agent-cost.py`、`agent-watch.py`、`dispatch-prompt.py`、`README.md`，新建 `scripts/agents/subagent_transcript.py`。主 agent 现查：五份自证在主工作区重跑都退 0，读数与报告一致（派发闸 454 种、续做闸 20 种、`dispatch-prompt.py` 90 项、`agent-cost.py` 14 项、新库 12 项）；攻方骨架现打的报告行是 `research/prompts/<填 轮>-<填 模型>-output.md`，`--check` 对报告行填草稿目录下 `result.md` 的提示报「三方腿的报告要…」拒；`lib_run_scope.py` 与 `run-scope-guard.sh` 里 `replay.sh --list-items` 那几处不是这一批的（报告改过的文件里没有）。它要主 agent 做的两件：新库自证加进 `.claude/gate.d/code-tooling.sh` 的 `SELFTEST_RUNNERS`（主 agent 用 `replace-once.py` 加在 `agent-cost.py` 那一行之后）；`AGENT_WATCH_BREAK=finished` 下的看门狗自证放后台跑（结果见下一条）。
- K4-1 的归档：`--until` 有了之后，闭窗口重跑一次 `python3 scripts/agents/agent-cost.py outcome --since 2026-10-01 --until 2026-10-21 --summary`，原样存 `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-2026-10-01-to-2026-10-21.txt`（第七批改成按东京日期切天、三方腿分两桶、`\bfix` 不再命中 fixture 之后照第二轮判决 K2-4 重出，文件名不变，sha256 `083c9216dd4c96ee09d92d7e296629b60d07b2b001cb799d75c8bc1a911cf841`；2125 个，折算 7004.91 百万，written-landed 842 个 50.0%、三方腿 463 个 28.9% 全部报告在仓且同轮有判决、报告被主 agent 点到 591 个 14.1%、返工字样 313 个占花费 13.4%；按系统时钟日期切的第一版 sha256 `6dde89a84129fa1b3b22da055372594d71bb11b262df5f2979c20db5a92e6ab0`、2144 个），当天复跑与存档逐行相同（往后「点到」没有上界、落地比对读此刻的工作树，数会变，第二轮判决 `research/prompts/landing-audit-r2-main-verification.md` K3-3）；节「第三批交回」那组 2268 个的数出自只带 `--since` 的那一次，含 10-22 当天，不能原样复跑。
- 任务 18 要主 agent 跑的看门狗开关：`AGENT_WATCH_BREAK=finished python3 scripts/agents/agent-watch.py --selftest` 放后台跑，退 1，失败行里有「子 agent refusedthendelivered 应当算交回或被停」（1 处），另有 finished、toolafterhandback、inquiryhandedback、notifiedafterhandback 几份样本一起红；输出存草稿 `agent-watch-break-finished.txt`。
- 第二轮开轮：正文 `research/prompts/_landing-audit-r2-body.md`，材料 `three-way-materials.py landing-audit-r2`（清单抄 13 行），快照 `research/prompts/landing-audit-r2-snapshot/judged-sha256.txt`（63 份，sha256 d374441005f2…）与 `judged-vs-index.diff`（3116 行，sha256 1fbd70f11262…）；辩方腿（Sonnet，K1）与攻方腿（Opus，K2、K3、K4）同一条消息派出，派之前两份提示 `dispatch-prompt.py --check` 都退 0。
- 门禁 code-tooling 六格真仓单跑（`SINGLEFS_HEAVY_TESTS=user-request`）：research-script-selftest-coverage、stage-owners、heavy-thread-defaults、long-run-header-exempt 绿；research-script-selftests 红在 `scripts/crash/lib_verdict_library.py`、`scripts/gate/verus-proof-check.py`、`scripts/experiment-apparatus/e165-pin-red-examples.py` 三份自证（runner 表 119 条里 3 条），change-range-single-source 红在 `.claude/gate.d/doc-text.sh:1646` 少 `-c core.quotepath=false`；四份都不是这一批的文件，这一批改到的 heavy-thread-defaults、long-run-header-exempt 两格（改调 `lib-lag-baseline.py`）绿。

### 第二轮判决与第七批

判决 `research/prompts/landing-audit-r2-main-verification.md`：辩方 26 条里 24 条站得住，采它两条够不着的（正推 10 进九十节 ⑮、K3-7 交定义）与五条带说明的；攻方 K2 打中 4 条、K3 打中 6 条、K4 打中 2 条，都采。主 agent 现查：快照 63 份全对上；`cite-check.py` 两份报告 85 处引文全对上；攻方模型在草稿副本里复跑，四份输出与腿的逐字节相同；K2-3 拿本会话两份当时放行的派发提示现跑 `--check` 都退 3。正文清单把 `.claude/agents/implementation-writer.md` 漏了、列了这一批没改的 `three-way-forward.md`，判决补核、第三轮清单补上。九十节标题不改（第一、二轮正文按原标题引它）。主 agent 照判决改了本记录六处限定（三方腿桶、没效果门槛、返工字样两处、盲判样本两行与载体搬家一行、复跑与逐行相同对照各一处）、九十节 ⑥ 的形态与 ⑪ 的第九处，加 ⑮、⑯。

| 任务 | 类别 | 关的条（第二轮判决节「二、逐格判」） | 文件 | 批次 |
|---|---|---|---|---|
| 20 门禁与扫描 | tooling-writer（opus） | K2-1 甲、K2-2 甲、K4-1 与 ⑪、K4-2 | `.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/doc-experiments.sh` 与 replay-comparison-units 样本、`.claude/gate.d/doc-decisions.sh` 与样本、`scripts/kb/quote-kb.py`、`.claude/gate.d/doc-registries.sh`、`scripts/kb/stale-candidates.py` | 7 |
| 21 钩子与协作脚本 | tooling-writer（opus） | K2-3、K2-4、K3-2、K3-4 甲、正推 8 交回闸拒绝说明 | `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`、`scripts/agents/agent-cost.py`、`scripts/agents/dispatch-prompt.py` | 7 |
| 22 定义文字 | tooling-writer | K3-7、正推 8 共用约束半句 | `.claude/agents/experiment-designer.md`、`.claude/agent-common.md` | 7 |
| 23 第三轮三方 | three-way | 「已定」项撑不撑得住、未定项清单完整性、报出的数能不能核 | — | 8 |
- 任务 22（定义文字，`research/prompts/subagent-landing-audit-2026-10-22/r2fix-definitions.md`，sha256 `2406534d3661fb6e87655ebaffbf40d1b1a0e9946a60e75fb706cc959cdb71a7`）：`.claude/agents/experiment-designer.md` 第 3a 步 ② 改成现算的预期差连同各臂基线的符号与量级列进「跑之前已经存在的数」、分辨率或判据形式分不出的写进「未定项：」交回，不当场改判据；`.claude/agent-common.md` 第 108 行删「或名字里带 report 的文件」半句。主 agent 现看两处原文，② 引的节名与第 2 步、登记表第 5 节标题一致，「本条末尾」那句（「三样里算不出的，不写死判据：写进第一节末尾的「未定项：」一段」）在；全文 `名字里带 report 的文件` 0 处。两处被攻过零轮，进第三轮清单。
- 任务 20（门禁与扫描，`research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md`，sha256 `8aafe773b25f263d14af1316c3f2363498f3ccacf348867a9af8462da983f910`）：K2-1 库文件头写明四处认法变了；K2-2 只缩不涨的键改成（实验号，登记出处，单元名，留存产物名），新样本 `replay-comparison-units-rekey-red`、新开关 `comparison-none-key-pair`；K4-1 与 ⑪ 两处改调库，新样本 `.claude/gate.d/fixtures/doc-decisions.sh/evidence-ledger-base-unreadable-red`、`.claude/gate.d/fixtures/doc-experiments.sh/decision-links-pending-base-unreadable-red`；K4-2 读法规整收进 `scripts/kb/quote-kb.py` 的 `kb_directory_entries`，`doc-registries.sh` 那一份删掉改调它。派发外两处：库的 `git show` 改按当前目录取路径（与 `ls-tree` 一致，doc-decisions 原来就这样取，加自证格与开关 `show-root-relative`）；登记位写成 `.` 也拒。主 agent 现查：三份新样本都在；`lib-lag-baseline.py`（9 项）、`quote-kb.py`、`stale-candidates.py` 三份自证在主工作区退 0；派发外两处合理，主 agent 认下，被攻过零轮、进第三轮清单。样本档与开关复跑要暂存，归提交前整轮（报告「要主 agent 跑的命令」五条）。它报没查的两处同形写法登在拆分提案第九十节 ⑱；`decision-evidence-lag.tsv` 文件头没补「取不到判红」，并进 ⑱ 那一批。
- 任务 21（钩子与协作脚本，`research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md`，sha256 `ecd632b6c06848ec4d3f82fd27dda97c92323001ab8c995b13e1e0f1ab1105c2`）：派发闸报告标签改在行首与句号、问号、叹号、分号之后的句首认，引号里与逗号之后的不认，紧跟否定词的文件名不判（新开关 `report-line-unanchored`、`report-name-negation-judged`）；`agent-cost.py` 四个子命令按东京日期切天、三方腿分两桶（新桶 `three-way-report-or-verdict-missing`）、`\bfix(?:es|ed)?(?![\w-])`、文件头写门槛没出处；交回闸超长拒绝说明写明 `result.md` 与三方腿的 `-output.md`。派发外：句首也认（攻方腿那份真提示的报告行写在句中，只认行首会被三方腿形态判拒）、`--since` 只认 `YYYY-MM-DD`。主 agent 现查：四份自证在主工作区退 0（派发闸 460 种、交回闸 23 种、`agent-cost.py` 18 项、`dispatch-prompt.py` 90 项）；两份当时放行的真派发提示重跑 `--check`，第一轮攻方腿那份退 0，第五批钩子工具员那份退 3 只剩认领行一处（派它时还没有认领行这一判），报告行那两种拒都没了；照新切法重出闭窗口存档（见本记录 K4-1 归档那一条，sha256 已更新）。派发外两处主 agent 认下，被攻过零轮、进第三轮清单；它报的派发闸开关不在复跑表、19 条点名对不上等登在拆分提案第九十节 ⑲。
- 第七批派出后看门狗的检出（只记，不处置）：门禁工具员在自己草稿目录里 `$(mktemp -d …)` 建临时目录、往里 `cp`、`>` 写的每一条，写范围检出钩子都报「这一刻算不出来、没判」，看门狗随之退出，20 分钟里五次以上（登在拆分提案第九十节 ⑰）；钩子工具员在草稿副本里起 `runner-dispatch-guard.sh --selftest` 被提交时才跑的检查拒一次；`agent-cost.py --selftest` 常驻 0.03 GiB 被报过线（阈值是峰值表 0.01 GiB 的 1.5 倍，并进 ⑬）；`agent-watch.py` 起两只时第二只退 5（「已有一个看门狗在盯这个会话」），照它的话不另起。

### 第三轮（第八批）

第七批三份交回到齐、主 agent 核过（各条见节「第二轮判决与第七批」末尾）。第三轮是最后一轮：正文 `research/prompts/_landing-audit-r3-body.md`，材料 `three-way-materials.py landing-audit-r3`（清单抄 3 行），快照 `research/prompts/landing-audit-r3-snapshot/judged-sha256.txt`（74 份；本记录也在被判之列，快照的 sha256 不写进本记录）与 `judged-vs-index.diff`；辩方腿（Sonnet，K1 未定项清单的完整性）与攻方腿（Opus，K2 已定项撑不撑得住、K3 报出的数能不能核）同一条消息派出，两份提示 `dispatch-prompt.py --check` 都退 0。被判清单补了第二轮漏列的 `.claude/agents/implementation-writer.md`。

### 第三轮判决与收拢

判决 `research/prompts/landing-audit-r3-main-verification.md`：辩方把六份工具员交回、两份判决、四份腿报告里的「没做」「没查」归成 66 件，有去向 40、一半 5、没有 21；攻方 K2 打中 6 条（报告行锚法 3 条并成一条，另 3 条轻）、K3 打中 1 条轻，两版结局账的差、门槛换值、各份自证种数都复得出。主 agent 照判决改了本记录四处（第 40 行、第 138 行补出处，两处「用量撞顶」改「中途停下」）、补第七批看门狗检出一条，拆分提案第九十节改 ⑥ ⑧ ⑪ ⑯ ⑰ ⑲、加 ⑳ 到 ㉔。第三轮之后不再开轮，零轮的改法交用户。

| 线 | 判 | 去向 | 折算与挂钟 |
|---|---|---|---|
| 九条缺口（节「九条缺口的判定」） | 本轮关 | 第三批、第五批、第七批的改动，三轮三方判站住 | 本会话 465 次调用、折算 0.22 亿、挂钟 8.3 小时、24 个子 agent（`python3 scripts/agents/agent-cost.py usage --since 2026-10-22` 那一行，含 10-22 起本会话全部） |
| 三方 landing-audit-r1、r2、r3 | 本轮关 | 三份判决 | 同上 |
| 拆分提案第九十节 ⑥ 到 ㉔ | 下轮开 | 交用户定（第三轮判决节「四、交用户」） | — |
| `.claude/kb/checks-owed.md` C879、C880 | 下轮开 | E165 那一侧的实验与装置，不在这一批 | — |
| 提交前整轮（五道门禁的样本档、开关复跑格、真仓单跑、`.claude/batch-scope` 登记） | 下轮开 | 用户说提交时照 `.claude/skills/gate/staged-gate.md` 跑 | — |
| 本会话的草稿目录 | 本轮关 | 收尾清盘 | — |

用户 2026-10-23 在弹窗里定：⑳「按草稿目录判（推荐）」、㉑「先实测再定（推荐）」、㉒「每批收尾跑一次（推荐）」、⑥ 到 ㉔「下一批一起派（推荐）」、㉓ checker 档红「合入后验证跑受影响的快档（推荐）」；㉓ 中途停下那一条主 agent 判不改（第三轮判决节「用户定案」）。同日另一个会话报根分区满（df 报只剩 113M），主 agent 现查时已是 62%、可用 1.4T，照它的要求清了本会话草稿里三份副本 46M（之前收尾清盘删了工作根下本会话 12 个目录约 31M），回了它一句。

### 第九批：九十节 ⑥ 到 ㉔（用户 2026-10-23 定「下一批一起派」）

课题：九十节 ⑥ 到 ㉔ 里除 ⑪（已做）、⑩（上游脚本，填内容要先看上游三语仓有没有在发版，另派）之外的各行，照各行「形态」列改完；出口是每一行的形态列做到、各自的自证与样本红绿读数交回。派发上限 5 个同时；第一波文件互不相交，第二波依赖第一波。改完照 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」走一轮三方（landing-audit-r4 起新的三轮）。

| 任务 | 类别 | 关的行 | 文件 | 依赖 | 批次 |
|---|---|---|---|---|---|
| 24 实测三个前缀 | clerk | ㉑ 的实测 | 只在自己草稿目录里 Write | — | 9 |
| 25 派发闸 | tooling-writer（opus） | ⑳、⑲ 的派发闸开关点名对齐 | `.claude/hooks/runner-dispatch-guard.sh`、`scripts/agents/dispatch-prompt.py` | — | 9 |
| 26 交回闸与检出 | tooling-writer（opus） | ⑥、⑦、⑰ | `.claude/hooks/handback-guard.sh`、`.claude/hooks/bash-command-detector.sh` | — | 9 |
| 27 门禁甲 | tooling-writer（opus） | ⑧、⑫、⑱、㉔ 的两处门禁 | `.claude/gate.d/code-tooling.sh`、`.claude/hooks/selftest-registry.tsv` 与它的判法、`.claude/gate.d/checker-independence-and-sync.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/decision-evidence-lag.tsv`、`.claude/gate.d/doc-registries.sh`、`.claude/gate.d/replay-comparison-units.tsv` 与各自样本 | — | 9 |
| 28 门禁乙 | tooling-writer（opus） | ⑨、⑮ | `scripts/kb/claim-experiment.sh`、`.claude/gate.d/doc-experiments.sh`、`.claude/agents/experiment-designer.md` 与样本 | — | 9 |
| 29 看门狗与结局账 | tooling-writer | ⑬、⑭、⑲ 的 usage 主会话那部分、㉒ 的脚本一侧 | `scripts/agents/agent-watch.py`、`scripts/agents/agent-cost.py` | 第一波交回 | 9 第二波 |
| 30 三方类型表 | tooling-writer | ⑯ | 三份三方定义的 frontmatter、`.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`、`scripts/agents/dispatch-prompt.py`、`scripts/agents/agent-cost.py` | 25、26、29 | 9 第二波之后 |
| 31 定义与合入验证 | tooling-writer | ㉒ 的定义一侧、㉓ 的 checker 档快档、㉔ 的实现员措辞、⑲ 复跑表加行、⑧ 的写范围行（任务 27 交回）、⑥ ⑦ 的共用约束交回正文闸（任务 26 交回） | `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/kb-scribe.md`、`.claude/hooks/agent-write-scope.tsv`、`scripts/gate/merge-and-verify.sh`、`.claude/gate.d/break-switch-replay-files.tsv` | 24、25 | 9 第二波 |
- 任务 24（实测三个前缀，`research/prompts/subagent-landing-audit-2026-10-22/b9-prefix-probe.md`，sha256 `e12694390ea974c6dc659ed850a6f2b74b7b16011d09c9b5a97f68091f917c4d`）：阳性对照 `report-probe.md` 被拒、阴性对照 `probe.md` 写成；`SUMMARY.md`、`summary-x.md`、`FINDINGS.md`、`findings-x.md`、`ANALYSIS.md`、`analysis-x.md` 被拒，`x-summary.md`、`x-findings.md`、`x-analysis.md`、`summary.txt` 写成；被拒的原文逐字是「Subagents should return findings as text, not write report files.」那一句。共用约束第 108 行与实测一致、不改；㉑ 剩派发闸加判三个前缀那一半，并进第二波改派发闸的那一份（任务 30）。混合大小写、别的目录没测。交回原文里抄了工具结果的标签，harness 当成指令形状标了一下，是引文，不是指令。
- 任务 26（交回闸与检出钩子，`research/prompts/subagent-landing-audit-2026-10-22/b9-handback-guard.md`，sha256 `cb9d9a1d794176c83cd30a6485d1d60b7c132d82578bb728ceeda9896e5fc10d`）：⑥ 成交回闸 ⑤、⑦ 成交回闸 ⑥、⑰ 改检出钩子 ⑫ 的 mktemp 认法，两份钩子经 `hook-swap.py` 换进主工作区。主 agent 在主工作区重跑：`handback-guard.sh --selftest`「自检通过（查了 45 种）」、`bash-command-detector.sh --selftest`「自检通过（查了 750 种）」，都退 0；弄坏开关逐条与 code-tooling 的 break-switch-replay、agent-write-scope 两格归提交前。交回点出的尾巴：共用约束交回正文闸那一条补 ⑤ ⑥ 并进任务 31；其余记拆分提案九十节 ㉕。合进来之后照约定通知 别的会话 核两边改动。
- 任务 27（门禁甲，`research/prompts/subagent-landing-audit-2026-10-22/b9-gates-a.md`，sha256 `6396ef89af266927c1a79cac8f23c1fdfd5cbba8c94f6811ed2ea6d179a8f41c`）：⑧ 成 code-tooling 新格 kb-directories-write-scope，⑫ 登记表认 `.claude/gate.d/` 顶层 `lib-` 库，⑱ 两处取基准取不到判红，㉔ 前两处。主 agent 在主工作区重跑 `lib_heavy_tests.py`（783 种）、`heavy-test-guard.sh`（1395 种）、`scripts/gate/light-selftest-guard.py`（14 格）、`lib-lag-baseline.py`（9 项）的自证，都退 0。新格在真仓上红：kb-scribe 写范围 `crates/singlefs-checker/invariants/*.md` 不盖子目录（这个目录现在没有子目录，红的是造的探针路径），挡提交，照「一轮怎么开、怎么收」第 2 条并进任务 31：写范围表加 `crates/singlefs-checker/invariants/**/*.md` 一行、`.claude/agents/kb-scribe.md`「写范围」同步，仍只许 `.md`。其余尾巴记九十节 ㉖。
- 任务 28（门禁乙，`research/prompts/subagent-landing-audit-2026-10-22/b9-gates-b.md`，sha256 `5307b0a5d54ecfb332b7761c33813f7dc9b9ee03ee70d7ccb293fe69d6dc3611`）：⑨ 实验号目录登记放在 `.claude/gate.d/experiment-number-directories.tsv`（工具员写范围不含 `.claude/` 顶层，主 agent 认下这个位置），读法抽成 `scripts/kb/lib_experiment_number_directories.py`；⑮ 比对对象表定成 E165 r5 表头逐字的形态，`.claude/agents/experiment-designer.md` 第 1 步加一句（定义改动，进第九批收尾的定义三方）。主 agent 在主工作区重跑 `claim-experiment.sh --selftest` 退 0（自证不在登记表里，照授权带 `SINGLEFS_HEAVY_TESTS=user-request`）。检出：工具员在 Bash 里用 python 写过一次仓内样本文件（`replay-comparison-units-table-red/expect`），钩子只给 clerk、kb-scribe 判 Bash 写范围，没拦；门禁与 `stage-selftest.sh` 被 heavy-test-guard 拒之后，它照派发在草稿里拷阶段逐份喂样本，与任务 27、第七批同一种核法，正式判定归提交前的样本档。
- 协调：别的会话照用户「你不要管其他的所有的你都动」在改全部起编译、拷库、跑重活的脚本（每一趟自己的编译目录、跑完删）并删共用编译缓存，问在跑的重活；本会话回「没有」（ps 现查到的崩溃点重放与双机驱动自证不是本会话的）。任务 31 里 `scripts/gate/merge-and-verify.sh` 加 checker 档快档那一步在它的范围里，等它改完通知再派，派发提示照它改后的编译目录写法；它另有工具员在改 `.claude/main-agent.md`「一轮怎么开、怎么收」清盘那一条，任务 31 改同一份文件的另两处（第 8 条点名结局账、「写与验分两段」验证段），一并等它通知后派、只用定点替换。
- 协调（续）：别的会话 通知它的子 agent 在主工作区改 `scripts/agents/watch.sh`、`scripts/agents/agent-watch.py`、`scripts/gate/merge-and-verify.sh` 与几份多机脚本，并要把 `.claude/hooks/bash-command-detector.sh` 的 ⑫ 收严成「目标里有算不出来的部分一律执行前拒」。撞文件两处：任务 26 的 ⑰ 改同一份 `bash-command-detector.sh`，它回两边都经 `hook-swap.py` 定点换、不整份覆盖，它只动 ⑫ 的判法、文件头两段说明、自证格与开关，换之前看 ⑰ 进没进来，不用排队；任务 26 合进来之后通知它核两边的改动都在；任务 29 改 `agent-watch.py`，等它的「已合进来」通知再派。
- 第二波：别的会话 通知 `scripts/agents/watch.sh`、`scripts/agents/agent-watch.py` 已合进来（它新加根分区余量告警、键 `root-free-gib-alert`），任务 29 已派（tooling-writer、opus，草稿目录 `~/.local/share/singlefs/work/landing-audit-b9-watch-cost`），派发提示许它只往 `watch.sh` 的键表加一个键；任务 26 合进来之后已通知 别的会话。
- 别的会话 的 ⑫ 收严合进来（在任务 26 那一版上经 `hook-swap.py` 定点换，`bash-command-detector.sh` sha256 以 `03549b8c` 起头）：主 agent 在主工作区现跑 `--selftest` 退 0、查了 764 种；任务 26 的两条开关 `mktemp-scratch-template-unknown` 红 5 格、`mktemp-template-owner-unchecked` 红 3 格，与合入时相同，期望从「记 / 不记」改成「拒 / 放行」；`mktemp-directory-unknown` 红 9 格，原 7 格都在，另 2 格是它新加的 clerk 格，文件头写「共 9 格」。已回它核过。⑫ 只判有写范围的 clerk、kb-scribe，在跑的两个工具员不受影响，没给它们发消息。
- 任务 25（派发闸按草稿目录判，`research/prompts/subagent-landing-audit-2026-10-22/b9-dispatch-guard.md`，sha256 `749902f52181c3ccbc5b813212559f96074e8daf6b45a810694720193e9cc39b`）：只改 `.claude/hooks/runner-dispatch-guard.sh`（经 `hook-swap.py` 换上，sha256 `b3db189f74336fb4b626977c89a4970915f8fe110d4a6bb83fbb16bc2c6002ec`）。⑳：④ 写范围照旧只认行首句首，⑰ 文件名与三方形态两判只看本提示「草稿目录」下的路径与裸文件名，新开关 `report-line-position-anchored`；⑲：96 条开关点名都改成失败行原文。主 agent 在主工作区重跑：`--selftest` 退 0、查了 471 种；新开关下退 1、红 8 格；这一批六份派发提示（任务 24 到 29）重过 `dispatch-prompt.py --check` 都放行。它在 3317 份真提示上两版比过：④ 没变，三方形态新放 52 份、没有新拒，文件名新拒 50 份、新放 89 份（工具员的数，主 agent 没复算）。三处推的口子与 `disable-check` 只点名 70 格记九十节 ㉗，进 landing-audit-r4；复跑表加派发闸一行（限时 120 秒）并进任务 31。
- 任务 29（看门狗与结局账，`research/prompts/subagent-landing-audit-2026-10-22/b9-watch-cost.md`，sha256 `fb8664490ff050270cb279a4dd12210941a61c898d1a8d906aaf4f3f03a1f930`）：⑬ 常驻内存阈值加下限 512 MiB（推的、没量过）、⑭ 照登记表认 light 自证的临时目录不判长跑判红、⑲ 主会话改按每条记录的东京日期切天、㉒ `outcome --out` 落盘。它没用 `hook-swap.py`（只收 `.claude/hooks/` 下的），改用 `replace-batch.py` 按基线到草稿的差定点换，换前核主工作区与基线相同；别的会话 的根分区余量告警几处都还在（主 agent grep 现查）。主 agent 在主工作区重跑：`agent-cost.py --selftest` 退 0、查了 22 项；`watch.sh --selftest` 退 0；`agent-watch.py --selftest` 退 0（「agent-watch 自检通过」，临时目录三条路都删了）。登记表 `.claude/hooks/selftest-registry.tsv` 里 `agent-watch.py` 那一行的理由过时，并进任务 30。
- 任务 30 已派（tooling-writer、opus，草稿目录 `~/.local/share/singlefs/work/landing-audit-b9-leg-types`）：⑯、㉑ 派发闸那一半，加 `selftest-registry.tsv` 里 `agent-watch.py` 那一行的理由；三份三方定义只动 frontmatter，先查 Claude Code 不认的键有没有先例，没有就改用登记表。
- 任务 31 已派（tooling-writer、opus，草稿目录 `~/.local/share/singlefs/work/landing-audit-b9-defs-verify`）：别的会话 通知 `scripts/gate/merge-and-verify.sh` 的编译目录写法已合进来（每一趟编进 `<日志目录>/target`、EXIT trap 里删，自证 38 种）之后派的。关 ㉓（合入后验证加 checker 档快档那一步，工具员只写脚本与假 cargo 自证、不跑 checker 档）、㉒ 定义一侧、⑥ ⑦ 的共用约束、⑧ 写范围、⑲ 复跑表加两行、㉔ 措辞；㉑ 定义一侧照实测不改（任务 24），从这一件里去掉。头一版提示里「只跑单元键对不上的 checker 档快档测试函数」一句被派发闸 ③ 当成许可跑重型拒，改写成「你只写这一步，真的那一趟由主 agent 起」后放行。
- 任务 30（三方类型表，`research/prompts/subagent-landing-audit-2026-10-22/b9-leg-types.md`，sha256 `f13e23164c5bdac80efb29466e92a1b05b9294bef64033d519e8888f33ff6440`）：⑯ 三份定义 frontmatter 第 9 行加 `leg: three-way`（先例：14 份定义都带 Claude Code 不认的 `required-inputs:`，一直照常派），读法收进新库 `.claude/hooks/lib_three_way_legs.py`，派发闸、交回闸、`dispatch-prompt.py`、`agent-cost.py` 改读它；账表不再把 three-way-materials 与已没有定义的 three-way-verifier 算进三方腿两个桶（真会话记录上桶变多少没量）；㉑ 派发闸 ⑰ 拒三个前缀开头的 `.md` 报告名，3324 份真提示新拒 1 份，主 agent 判不按类型的工具放宽。主 agent 在主工作区重跑：派发闸 483 种、交回闸 49 种、`dispatch-prompt.py` 92 项、`agent-cost.py` 23 项，都退 0；三份定义第 9 行现查在。暂存区里这一批多份文件已有一截改动（不是本会话暂存的），已发消息问 别的会话：它回这一段没跑过 git add、提交前先来对这一批的三方与没跑的格；`bash-command-detector.sh` 暂存区那一份与工作区逐字相同（sha256 都以 `03549b8c` 起头，含它最新的 ⑫），`hook-swap.py`、`replace-batch.py`、`replace-once.py`、`insert-row.py` 里没有 git add，是谁暂存的没查清；照「只核自己的改动」不追查，提交时照 `stage-mine.py` 重新暂存，本会话不动暂存区。
- 别的会话 提醒：完工清理钩子 `.claude/hooks/completion-cleanup-guard.sh` 按 Bash 调用的时间窗认运行目录、认不出有进程开着的，会把别的会话在用的运行目录列给本会话删；它已派人改。改好之前它列出的目录，不是本会话起的先核、不删、告诉它（它随后换进修好的一版，`sha256` 以 `0ca8de76` 起头，主 agent 现查相同：只认确凿归属、认不出的只报不拦，有进程开着的整个运行目录不列）；这一批收尾清盘照「一轮怎么开、怎么收」第 11 条只删本会话认领的草稿目录。
- 别的会话 通知两处新判法生效：主工作区里直接起会编的 cargo（build、test、check、clippy、run、bench、doc）由 `.claude/hooks/heavy-test-guard.sh` 执行前拒，主 agent 也拒，出路是 `bash scripts/hosts/run-build-directory.sh <命令…>` 或仓外用完即删的 `CARGO_TARGET_DIR`（判法共用 `.claude/hooks/lib_workspace_cargo_lock.py` 的 `cargo_invocations`，`lib_heavy_tests.py` 也改了）；`run-with-memory-cap.sh` 给每条命令一个自己的 TMPDIR、跑完删，留现场设 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`。任务 31 只用假 cargo 写自证、不碰这几份钩子库，没给它发消息；它交回后 merge-and-verify 新加的 checker 档快档那一步核它起 cargo 的写法过不过这道判法。
- 任务 31（定义与合入验证，`research/prompts/subagent-landing-audit-2026-10-22/b9-defs-verify.md`，sha256 `1fb272edb0db0c867625eb1b28caf1639baafa99e34efb8098aa8f58ea3608e4`）：㉓ `scripts/gate/merge-and-verify.sh` 在编译、测试与重编核对之后起门禁 `checker-tier-crash-point-replay.sh --check crash-case-replay,feature-gated-tests <树>`（键与复用由那一道算，没带 `SINGLEFS_HEAVY_TESTS` 记红给出路，没碰 checker 档闭包报本次未跑），`.claude/main-agent.md` 第 10 条与验证段第 1 步同步；㉒ 主 agent 定义第 8 条加每批收尾的结局账命令；⑥ ⑦ 共用约束交回正文闸补两样；⑧ 写范围表加 `kb-scribe` 的 `crates/singlefs-checker/invariants/**/*.md` 一行、`kb-scribe.md` 两处同步；⑲ 复跑表加两行；㉔ 实现员第 4 步。主 agent 在主工作区重跑：`merge-and-verify.sh --selftest` 退 0、45 种。它交回时 `.claude/hooks/write-guard.sh --selftest` 红 1 格（「范围:kb-scribe 不变量清单的子目录越界」断言的是这一批改掉的旧行为），它起草了定点规格、只 `--dry-run` 过；主 agent 照规格经 `hook-swap.py` 换上（sha256 `4cd61e4a…` 换成 `a9a74bdc…`），重跑退 0、查了 229 种，`WRITE_GUARD_BREAK=scope-new-rows-dropped` 下正好红点名的 3 格。这一处是主 agent 自己改的钩子自证，并进 landing-audit-r4 的被判文件。它点出的四处口子记九十节 ㉒、㉓ 的去向，进 r4。

## 第九批收尾：landing-audit-r4 与收拢（2026-10-23）

- landing-audit-r4 开轮（2026-10-23）：正文 `research/prompts/_landing-audit-r4-body.md`；`three-way-materials.py landing-audit-r4` 读 4 份 kb 文件、清单抄 34 行不抄 15 行；快照被判 80 份、对 HEAD 的 diff 14292 行（暂存区里有一截不是本会话暂存的改动，所以对 HEAD 取）。同一条消息派正推腿（three-way-forward，K1）与攻方腿（three-way-attack，K2、K3），两条都派得出：三份三方定义 frontmatter 加 `leg: three-way` 之后照常派出，⑯ 那条「推的」由这一次派发坐实。
- 派之前主 agent 自己发现攻方提示的禁读清单写成了它自己的报告（`landing-audit-r4-opus-output.md`，sed 换名时连带换错），派发闸照样放行；改成正推腿报告后派。派发闸不判「禁读清单点到自己的报告路径」，记进九十节（与 ㉗ 三方腿报告路径认法同一处）。
- 写正文时写闸拒了正文里的会话名；查出调度记录里 11 处、拆分提案 1 处同样写了会话名（经 Bash 的 insert-row.py / replace-once.py 写进去，写闸不判 Bash），逐处经 replace-once.py 换成「别的会话」。Bash 写仓文件不过会话名那一判，记进九十节。
- landing-audit-r4 判决 `research/prompts/landing-audit-r4-main-verification.md`：两条腿交齐，快照被判 80 份 `sha256sum -c` 全 OK（kb 快照里 E165 实验页英文、日文两份被别的会话改过，两份腿报告引它们 0 处）；`cite-check.py` 核了 52 处引文全对上；攻方模型在副本里复跑四份输出逐字节相同；正推腿报的样本档读数主 agent 重跑两道（doc-registries 51 个、checker-independence-and-sync 16 个用例判对）。K1 十四行做到，⑯ 由这一轮派发坐实；采八条修补记九十节 ㉘、进第十批，派发前自己撞上的两处与 ⑬ 不单调记 ㉙，K2-5 并进 ㉗；K2-6 主 agent 自己写：`.claude/gate.d/replay-comparison-units.tsv` 文件头加一行指到 C879，`.claude/kb/checks-owed.md` 三语的 C879 补一句（那五条只在「# 不登记的比对对象」一栏点名），单跑 replay-comparison-units 格绿。
- 结局账（`.claude/main-agent.md`「一轮怎么开、怎么收」第 8 条）：`python3 scripts/agents/agent-cost.py outcome --since 2026-10-22 --until 2026-10-23 --summary --out records/landing-audit-b9-outcome-2026-10-23.txt` 退 0，查了 249 个子 agent；返工字样 38 个（个数 15.3%、花费 14.6%）；没交回 12 个（停掉 9、还在跑 3）。

收拢表（这一批开着的线）：

| 线 | 判 | 去向 |
|---|---|---|
| 九十节 ⑥ 到 ㉔（除 ⑩ ⑪） | 本轮关（landing-audit-r4 第一轮站住） | 九十节各行去向列 |
| landing-audit-r4 采的八条修补 | 下轮开 | 九十节 ㉘，第十批，改完开 landing-audit-r5 |
| ㉕ ㉖ ㉗ ㉙ | 下轮开 | 九十节各行，下一批工具改动 |
| ⑩ 上游只填内容 | 下轮开 | 九十节 ⑩，先看上游三语仓状态 |
| 提交前的样本档、开关复跑、break-switch-replay 与 kb-directories-write-scope 两格 | 下轮开 | 提交时主 agent 照 `.claude/skills/gate/staged-gate.md` 跑，等用户说提交 |
| 这一批定义改动的三方 | 本轮关第一轮；第二、三轮随第十批 | landing-audit-r5、r6 |

## 第十批：landing-audit-r4 采的修补（2026-10-23）

用户 2026-10-23 在弹窗里定「现在开第十批（推荐）」。课题：关拆分提案九十节 ㉘（landing-audit-r4 判决节「三、判决」第 2 条采的修补），加 K2-5（三方腿报告路径三种认法，判决里采、并进 ㉗ 的那一条）；K2-6 主 agent 已在第九批收尾写完。出口：下表两件各自的验收达成、主工作区自证重跑绿，再开 landing-audit-r5 攻这一批修补本身（第二轮分工）。派发上限 5，截止批次 10；批内冒出的发现只进下一轮清单。㉕、㉖、㉗ 其余几处、㉙ 不在这一批。

| 任务 | 类别 | 关的 | 要改的文件 | 依赖 | 批次 |
|---|---|---|---|---|---|
| 32 定义与规则同步 | tooling-writer | K2-1、K2-2、K2-3 | `.claude/main-agent.md`（第 3 条、调度表合入后验证那一行与写回员那一行）、`.claude/agent-common.md`（第 58 行一带）、`.claude/rules/implementation-workflow.md`（「checker 档只在提交时跑，harness 随时跑」那张表两行）、`.claude/hooks/heavy-test-guard.sh`（只改拒绝说明那一句） | 无 | 10 |
| 33 草稿目录认法、命名检查、三方报告路径 | tooling-writer | K2-4 并 K3-2、K3-1、K2-5、⑳ 自证读真提示 | `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`、`.claude/hooks/completion-cleanup-guard.sh`（只改认草稿目录那一行的读法）、`.claude/hooks/lib_three_way_legs.py`、新的共用库 | 无 | 10 |

两件文件不相交，同一条消息里派。`.claude/hooks/completion-cleanup-guard.sh` 是别的会话刚改过的，派之前发消息告诉它这一处要动。
- 任务 32、33 已派（tooling-writer、opus，同一条消息；草稿目录 `~/.local/share/singlefs/work/landing-audit-b10-defs-sync`、`landing-audit-b10-hooks`）。别的会话回：没人在改 `completion-cleanup-guard.sh`，主工作区那一份（sha256 以 `0ca8de76` 起头）就是最新的，换上之后告诉它、它再跑一遍自证。
- 任务 32（定义与规则同步，`research/prompts/subagent-landing-audit-2026-10-22/b10-defs-sync.md`，sha256 `4fd6019d6bf3ba58afa53d345269652d66f1d52f46b1fb557088172cbaca5941`）：K2-1 在 `.claude/main-agent.md` 第 22 行、`.claude/rules/implementation-workflow.md` 第 45、47 行、`.claude/agent-common.md` 第 58 行写同一个例外并指到验证段第 1 步，`.claude/hooks/heavy-test-guard.sh` 给子 agent 的拒绝说明加常量 `MERGE_VERIFY_QUICK_STEP`、开关 `merge-verify-quick-step-dropped` 与一格自证（比派发多的这三样主 agent 认下）；K2-2 第 7、29、96 行整条命令补前缀；K2-3 第 102 行两层。主 agent 在主工作区重跑 `heavy-test-guard.sh --selftest` 退 0、查了 1402 种（其中该拒 266 种），新开关下红 1 格。它点出主 agent 不带前缀被拒时看到的「照抄」那一行（第 958 行一带）没有这个例外，主 agent 照第 861 行的写法加 `merge_verify_hint`，经 `hook-swap.py` 换上（草稿与主工作区各喂 28 条），自证照旧 1402 种、退 0，喂一条不带前缀的崩溃放量命令，「照抄」里已有这一句；这一处没有单独的自证格，进 landing-audit-r5。验证段第 1 步命令本身没写前缀（前缀写在同一步正文里），留给 r5 判。
- 任务 33（草稿目录认法、命名检查、三方报告路径，`research/prompts/subagent-landing-audit-2026-10-22/b10-hooks.md`，sha256 `800ab0dd6f2102610cd1ddefc609adee1fbbfc1ade3064d678fab8b71d37b321`）：新库 `.claude/hooks/lib_prompt_scratch_line.py` 只认行首「草稿目录：」，派发闸 ⑯ ⑰、完工清理闸、交回闸 ⑥ 四处改调它（攻方探针 relay-comma、relay-list 由退 2 变退 0，own-zai 照旧 0）；交回闸 ⑤ 只认顺序执行位置、真跑完的两条命名检查（攻方五种由退 0 变退 2）；三方腿报告路径收进 `lib_three_way_legs.py` 的 `is_leg_report_path`、`leg_report_mentions`；派发闸自证读 `research/prompts/landing-audit-r3-opus-model/prompts/` 的 14 份真提示。主 agent 在主工作区重跑：派发闸 507 种、交回闸 62 种、新库 18 格、完工清理闸、`lib_three_way_legs.py`、`dispatch-prompt.py` 92 项、`agent-cost.py` 23 项，都退 0；完工清理闸换之前核过 sha256 以 `0ca8de76` 起头。它问的一处主 agent 判：交回闸 ⑤ 也认命名检查退 1，只要整条最后一条就是那条命名检查（共用约束让命中在别人文件里的照写、照样交回，退 1 正是检查跑完的结果），主 agent 认下、被攻过零轮，进 landing-audit-r5。`.claude/gate.d/` 下两处登记由主 agent 补：`code-tooling.sh` 的 `SELFTEST_RUNNERS` 加新库一条、`break-switch-replay-files.tsv` 加新库一行（60 秒）。`scripts/` 下另三种「草稿目录」认法（`dispatch-prompt.py` 的 `DRAFT_LINE`、`agent-cost.py`、`agent-watch.py`）没收进来，记九十节 ㉚。
- 主 agent 单跑门禁 code-tooling 的 break-switch-replay、break-switch-headers、break-switch-outside-selftest 三格（`SINGLEFS_HEAVY_TESTS=user-request`，经 `two-logs.sh`）：headers 绿；outside-selftest 红，点名 `scripts/agents/copy-originals.py`（别的会话的，已转给它、它已派人改）与 `scripts/agents/subagent_transcript.py`（第五批加的）；replay 那一格自己的 python 抛 `UnicodeDecodeError`、没判完。主 agent 在草稿里拷那一道、把起子进程那一句改成 `errors="surrogateescape"` 加一行打点重跑：只有 `scripts/hosts/run-build-directory.sh`（别的会话的，已转给它）在 `RUN_BUILD_DIRECTORY_BREAK=owner-left-alone` 下吐坏字节，整格其余照判、绿。两处都挡提交，照「一轮怎么开、怎么收」第 2 条并进第十批：记九十节 ㉛，任务 34 已派（tooling-writer、opus，草稿目录 `~/.local/share/singlefs/work/landing-audit-b10-replay-decode`）。
- 任务 34（复跑格容错与守卫，`research/prompts/subagent-landing-audit-2026-10-22/b10-replay-decode.md`，sha256 `6553b37cd42e6c867cbb53edc7c8a716ab038a262d9c7604a95e2593a267c2f0`）：`.claude/gate.d/code-tooling.sh` 的 `attempt()` 改按字节收、stdout 与 stderr 各自严格解，解不了的那一条记「坏字节」红并点名路径、开关、哪一路、第几字节，别的照判；新样本 `.claude/gate.d/fixtures/code-tooling.sh/break-switch-replay-undecodable-red/`、开关 `CODE_TOOLING_BREAK=replay-output-strict-decode`；`scripts/agents/subagent_transcript.py` 加 `refuse_break_outside_selftest()`、开关 `honor-break-outside-selftest`。主 agent 在主工作区重跑 `subagent_transcript.py --selftest` 绿、查了 13 项；平常跑设 `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` 退 2、点名只在自证里认。
- 主 agent 单跑门禁 code-tooling 的 break-switch-replay、break-switch-outside-selftest、break-switch-headers、research-gate-lint 四格：前三格绿；样本档 `stage-selftest.sh --item code-tooling.sh` 判得都对（97 个用例）。research-gate-lint 格红三类：`scripts/agents/agent-watch.py` 第 4682 到 4684 行（任务 29 的 ⑭ 自证里造的 ✗ 输出被当成没出路的拒绝），主 agent 把那四行 `printf` 里的 ✗ 换成 `\342\234\227`（写出的文件内容不变），重跑 `agent-watch.py --selftest` 退 0、那一格里这三处没了；暂存区里 `scripts/hosts/run-build-directory.sh`、`.claude/hooks/completion-cleanup-guard.sh` 是 100644 与五处循环里逐个 kill 归别的会话，已转给它、它已在工作区 `chmod +x` 并派人改那几处（复跑时剩执行位两份与 kill 两处）。主 agent 改 `agent-watch.py` 那四行属这一批，并进 landing-audit-r5 的被判文件。
- 进程安全扫描 `bash-command-detector.sh --scan-scripts <仓根> <目录>…` 只带一部分目录时，`.claude/process-safety-pending` 里落在没扫目录下的行都报「一处要放行的都没有（改好了就从清单里删掉这一行）」：主 agent 只带 `scripts/agents` 跑，报 `scripts/mutation/mutate.sh` 那一行；别的会话不带 `scripts/agents` 跑，报 `scripts/agents/agent-watch.py` 那一行（那一行其实还要留：第 4407、4525、4615、4758 行仍在 for 循环里逐个 `os.kill` 自己起的子树），它据此误删过 mutate.sh 那一行、又加回。记九十节新一行：清单里不在这一趟扫描目录下的行判「本次未判」，不判多余。
- 别的会话通知：`copy-originals.py`、`test-environment-check.py`、`broad-mutation.py`、`replay.sh`、`verify-citations.sh` 补了平常跑设开关就拒；`run-build-directory.sh` 自证里 `head -c 400` 按字节截中文是坏字节的根因，已改；`replay.sh` 同类 14 处与 kill 循环也改了；`run-with-memory-cap.sh` 的守卫与按字节截（第 1634、1641、1651、2435、2450、2468 行）进它的下一批。
- 别的会话报门禁 code-tooling 的 research-script-selftest-coverage 格红在 `scripts/kb/lib_experiment_number_directories.py`（任务 28 加的库）：文件头第 5 行字面写了「claim-experiment.sh --selftest」，这一格把它算成声称有自证。主 agent 用 replace-once.py 把那半句改成「claim-experiment.sh 自证里「目录登记」那几处判」（这份库不在 landing-audit-r5 快照里），重跑那一格它已不在名单里；那一格剩 4 份 E171 脚本，不是这一批的。另一个会话问 `scripts/experiment-apparatus/` 全仓改名，主 agent 回：不改、不跑引它的命令，请等 landing-audit-r5 判决写完再动手（快照 27 份里 12 份引这串）；它答应等。
- landing-audit-r5 开轮（第九、十批改动的第二轮）：正文 `research/prompts/_landing-audit-r5-body.md`，快照被判 27 份、对 HEAD 的 diff 10423 行；同一条消息派辩方腿（three-way-defense，K1，复核 landing-audit-r4 判决）与攻方腿（three-way-attack，K2、K3）。辩方跑满 2 小时，看门狗提醒，主 agent 看它一分多钟前还有动作、只是慢，`watch.sh --ack` 接着盯。
- landing-audit-r5 判决 `research/prompts/landing-audit-r5-main-verification.md`：快照里只有 `.claude/gate.d/code-tooling.sh` 被别的会话改过，主 agent 取 HEAD 加快照 diff 倒推出开工那一版（sha256 与快照相同），攻方引的第 4217 行在那一版逐字找得到，后来的改动清单存 `research/prompts/landing-audit-r5-snapshot/code-tooling-changed-after-snapshot.diff`；攻方模型复跑 18 份输出 17 份逐字节相同、`leg_handbacks` 715 变 716 是腿交回后真交回又多一次。采：交回闸 ⑤ 误拒 `timeout` 包着后接 `; echo rc=$?` 的真跑完命名检查（两条腿各自量到，2026-10-22 起 4 份）与退出码不是命名检查自己的几种照收、⑳ 真提示下一次归档会被删、交回闸 ② 仓外报告、例外措辞、看门狗 `progress_file_of`（58 份里 21 份取错），记九十节 ㉜、进第十一批；收严成只认行首「草稿目录：」保留，第一轮判决「收严不丢罩住的对象」不成立，代价是 2026-10-21 以前不照骨架写的 402 份真提示（2026-10-22 起 0 份），写进九十节 ⑳ 去向；轻的三条与进程安全扫描只扫部分目录的误报记 ㉝；㉙ 里 ⑬ 那一件辩方判没有对象，删掉。方法上的观测：第一轮两条打中在真记录里 0 次撞上，第十批收严在真记录里只带来 4 份误拒，第十一批每一处收严先拿真会话记录量新拒与新放。

## 第十一批：landing-audit-r5 采的修补（2026-10-23）

用户 2026-10-23 在弹窗里定「开第十一批加 r6（推荐）」。课题：关拆分提案九十节 ㉜（landing-audit-r5 判决节「三、判决」第 3 条采的修补）。出口：下表两件各自的验收达成、主工作区自证重跑绿，每一处收严在真会话记录上量过改前改后的新拒与新放；再开 landing-audit-r6（第三轮、最后一轮）。派发上限 5，截止批次 11；批内冒出的发现只进下一轮清单。㉝ 与 ㉕ 到 ㉛ 其余几处不在这一批。

| 任务 | 类别 | 关的 | 要改的文件 | 依赖 | 批次 |
|---|---|---|---|---|---|
| 35 交回闸与派发闸 | tooling-writer | ㉜ 的交回闸 ⑤（`timeout` 包着且跑完的、只认命名检查自己的退出码）、交回闸 ② 仓外报告、派发闸 ⑳ 真提示拷进钩子样本目录 | `.claude/hooks/handback-guard.sh`、`.claude/hooks/lib_three_way_legs.py`、`.claude/hooks/runner-dispatch-guard.sh`、钩子样本目录 | 无 | 11 |
| 36 看门狗与例外措辞 | tooling-writer | ㉜ 的看门狗 `progress_file_of`（九十节 ㉚）、例外措辞罩上 `scripts/gate/merge-and-verify.sh` 自己给的只重跑红格那条命令 | `scripts/agents/agent-watch.py`、`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md`、`.claude/hooks/heavy-test-guard.sh`（只改 `MERGE_VERIFY_QUICK_STEP` 那一句） | 无 | 11 |

两件文件不相交，同一条消息里派。另一个会话要把 `scripts/experiment-apparatus/` 全仓改名，派之前把这两件要改的文件告诉了它。
- 任务 35、36 已派（tooling-writer、opus，同一条消息；草稿目录 `~/.local/share/singlefs/work/landing-audit-b11-guards`、`landing-audit-b11-watch-wording`）。改名的会话回：这一批先照现在的路径做，交回、合进主工作区之后通知它，它再一次全仓替换（`heavy-test-guard.sh` 有 12 处按路径认重型测试，先改名会有一段时间新路径认不出）。
- 任务 36（看门狗与例外措辞，`research/prompts/subagent-landing-audit-2026-10-22/b11-watch-wording.md`，sha256 `a6d10104f7200508c98629254b409cbd3659ba3db0bb93baa02707f18f43a86e`）：`scripts/agents/agent-watch.py` 的 `progress_file_of` 改调 `.claude/hooks/lib_prompt_scratch_line.py` 的 `scratch_directories`，没有行首「草稿目录：」一行的报取不到、不猜别处，开关 `AGENT_WATCH_BREAK=progressanypath`；真会话记录上取到别人的 progress.md：2026-10-22 起 231 份里 22 → 0、全部 3358 份里 41 → 0，取到自己的不变（工具员的数，攻方清单那一行改前读数与 landing-audit-r5 攻方的 21 份相同）；五处例外改成同一句、罩上 merge-and-verify 那一步给出的只重跑那几格的命令。主 agent 在主工作区重跑：`heavy-test-guard.sh --selftest` 1402 种（其中该拒 266 种）退 0；`agent-watch.py --selftest` 退 0，`progressanypath` 下红 3 格。主 agent 补了 `lib_prompt_scratch_line.py` 文件头调用方表（四处改五处、加看门狗一行），自证 18 格照绿。单跑门禁 code-tooling 的 break-switch-headers 格红两处，都是 E171 的 `frozen_o3.py`（不是这一批的），已转给改名的那个会话。
- 任务 35（交回闸与派发闸，`research/prompts/subagent-landing-audit-2026-10-22/b11-guards.md`，sha256 `79b6b8d106ff5a6f5c54db8dab8f1330368f67f6dc92342c921b7d7c085ffddf`）：交回闸 ⑤ 只收命名检查自己的退出码认得出、又是 0 或 1 的（顶层最后一段取这条 Bash 的退出码，后面紧跟 `; echo <标签>$?` 的取输出里那一行；`timeout` 包着的照收，124、137、141、2 不收；`| head`、`| tail`、`|| true` 认不出不收），4 份真误拒放回 3 份，第 4 份 `aa68d151c2fa72bb7` 接了 `| tail` 照旧拒，真会话记录上交回层面新拒 0、新放 3；交回闸 ② 与派发闸 ⑰ 共用的 `leg_report_mentions` 只认解到仓根下 `research/prompts/` 的路径，真交回新拒 46 次（`m5-dev` 37 次改前是引文核对读不到文件退 2 放过的、`m4-predev` 9 次核的是仓里同名的另一份），⑰ 在 693 份真三方腿派发提示上改前改后不变；⑳ 的 14 份真提示逐字节拷进 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`（后缀改 `.txt`，`sources.tsv` 记来历与整份 sha256），目录不在或改一个字节逐格判红。主 agent 在主工作区重跑：交回闸 74 种、派发闸 512 种、`lib_three_way_legs.py`、`dispatch-prompt.py` 92 项、`agent-cost.py` 23 项都退 0。交回闸 ② 收严会拒仓外副本里写报告的三方腿，主 agent 给六个在跑的会话各发了通知（`m5-dev` 那个目录没有归属文件，查不出是谁的）。`| tail` 接在命名检查后面、输出看得出跑完了的那一类没收，记进 landing-audit-r6 的攻击面。
- 交回闸 ② 收严带进一处误拒（主 agent 从别的会话的答复里得知）：`~/.local/share/singlefs/work/m5-dev` 是里程碑五会话在本仓登记的 git worktree（分支 `m5-dev`，`git worktree list` 列得出），任务 35 量到的 46 次新拒里那 37 次是它在那棵树里派的三方腿，报告写在那棵树的 `research/prompts/` 下，是正当写法；另 9 次的 `m4-predev` 是早先的仓副本、目录已不在，照拒对。挡着别的会话，照「一轮怎么开、怎么收」第 2 条并进第十一批：任务 37 已派（tooling-writer、opus，草稿目录 `~/.local/share/singlefs/work/landing-audit-b11-worktree-reports`），认法的根改取 `git worktree list --porcelain` 列出的每一棵工作树。已给可能是里程碑五的两个会话发更正；其中一个回不是它、它的腿都写在主工作区。改名的会话、写回会话、里程碑四会话都回了没用这两个目录。
- 任务 37（交回闸认 worktree 报告，`research/prompts/subagent-landing-audit-2026-10-22/b11-worktree-reports.md`，sha256 `fcf7c1e0e613e0f3dbefd4f71e78f955dcfb4dde80fb0353b2d2f2c22b177f9a`）：`.claude/hooks/lib_three_way_legs.py` 新加 `registered_trees`（项目根加 `git worktree list --porcelain` 列出的各棵，取不到列表时只有项目根并交回原因）与 `leg_report_mentions_in_trees`，`leg_report_mentions` 签名不变；交回闸 ② 按报告所在那一棵当 `cite-check --root`；真会话记录上 `m5-dev` 那 37 次改回放行、`m4-predev` 那 9 次照拒、新拒 0，派发闸 ⑰ 在 693 份真三方腿派发提示上不变（工具员的数）。主 agent 在主工作区重跑：交回闸 78 种、`heavy-test-guard.sh` 1402 种、派发闸 512 种都退 0。主 agent 一度把 `lib_three_way_legs.py` 登进 `selftest-registry.tsv`（以为它有自证），现查它没有 `--selftest`（带上也只 import、不打一行就退 0，文件头写自证由各调用方各带一格），当场撤回那一行。改名后冻结真提示 `a9717cf07fb6e7c21` 随之换了路径、别的会话重钉了 sha256，主 agent 判不另管：路径不在冻结范围（`.claude/rules/path-moves.md`）。
- 别的会话 2026-10-23 把 `scripts/experiments/` 全仓 `git mv` 成 `scripts/experiment-apparatus/`（866 份文件改引用，搬迁登记在 `.claude/gate.d/path-moves.tsv`）；它报 landing-audit-r4、r5 下 16 份证据只差路径（`research/prompts/` 下 `_landing-audit-r4-appendix.md`、`_landing-audit-r4-background.md`、`landing-audit-r4-main-verification.md`、`landing-audit-r4-opus-model/probes/merge_select_probe.sh`、`landing-audit-r4-opus-output.md`、`landing-audit-r4-snapshot/judged-vs-head.diff`、`_landing-audit-r5-appendix.md`、`_landing-audit-r5-background.md`、`landing-audit-r5-opus-model/orig/handback-guard.sh.orig`、`landing-audit-r5-opus-model/orig/selftest-registry.tsv.orig`、`landing-audit-r5-opus-model/out/dump_naming.out`、`landing-audit-r5-opus-model/out/naming_real.out`、`landing-audit-r5-snapshot/code-tooling-changed-after-snapshot.diff`、`landing-audit-r5-snapshot/judged-vs-head.diff`、`landing-audit-r5-sonnet-model/inputs/pre-b10-hooks/handback-guard.sh`、`landing-audit-r5-sonnet-output.md`），照 `.claude/rules/path-moves.md` 第 6 步只差路径的不算证据被改。主 agent 核第九到十一批改法原文都在；改名带出两处钩子自证红（`heavy-test-guard.sh` 经 `lib_heavy_tests.py` 认目录那张清单还写旧名；派发闸钉的真提示 `a9717cf07fb6e7c21.txt` 改名换了路径、sha256 对不上），转给改名的会话。
- 第十一批收尾：主 agent 单跑门禁 code-tooling 的 break-switch-headers、break-switch-replay 两格都绿（交回闸新加的四条开关在里面）；改名带出的两处钩子自证红由改名的会话修好（`lib_heavy_tests.py` 的 `SCRIPT_CATEGORY_DIRECTORIES`，它顺带改了五份按段拼路径的脚本；派发闸钉的真提示重钉、`sources.tsv` 写明只差路径），主 agent 重跑 `heavy-test-guard.sh` 1402 种、派发闸 512 种退 0。什么算写完：㉜ 六处都有改法与自证格，每一处收严或放宽都在真会话记录上量过；剩下的只有三方打中才知道的。下一步开 landing-audit-r6（第三轮、最后一轮）。
