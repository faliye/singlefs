# landing-audit-r1 正推腿（Sonnet）报告：K1 每处改法做没做到它要关的缺口

日期：2026-10-22。立场：正推。被判改动：`research/prompts/landing-audit-r1-snapshot/judged-vs-index.diff`（sha256 a68200baa5887ecb9f11bbd4afd7688ea193de89b8f544e825a038a570e34fed，我自己 `sha256sum` 复核一致）。工作区里被判文件的 sha256 与 `judged-sha256.txt` 对过（agent-common.md、main-agent.md、runner-dispatch-guard.sh、agent-cost.py 四份一致）。

方法：从 `records/2026-10-22-子agent结果落地审计.md` 节「九条缺口的判定」「第三批依赖表」「第三批交回」出发，逐条缺口找改法落在哪一份文件哪一句、哪一个判法；再按 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」第 3 步造该红的输入，看现有钩子、门禁会不会红；不会红的看 `records/2026-09-16-subagent拆分提案.md` 节「九十」有没有登。方案按 `crates/` 与 `scripts/` 今天的实现谈：这一批没有改 `crates/`，新格读 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 的单元表字面量，我只读。

我自己复跑的（子 agent 只跑轻型自证和读文本的钩子，没跑门禁阶段，没编译）：

| 复跑 | 结果 |
|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 绿，查了 437 种 |
| 同上加 `RUNNER_DISPATCH_GUARD_BREAK=report-name-unchecked` | 红 2 格（stdin 两格）；`report-name-case-sensitive` 红「报告名:大写也拒」 |
| `python3 .claude/hooks/lib_heavy_tests.py --selftest` | 绿，查了 780 种；`LIB_HEAVY_TESTS_BREAK=naming-self-check-any-cell` 红「点名里夹着别的格」「只点名别的格」，`naming-self-check-counted` 红前三格 |
| `bash .claude/hooks/write-guard.sh --selftest` | 绿，查了 228 种；`WRITE_GUARD_BREAK=scope-new-rows-dropped` 红「范围:kb-scribe 写不变量清单的 .md」「范围:experiment-runner 写重跑比对对象登记」 |
| `python3 scripts/agents/dispatch-prompt.py --selftest` | 绿，查了 78 项；`DISPATCH_PROMPT_BREAK=report-default-dropped` 红 3 处 |
| `bash scripts/kb/claim-experiment.sh --selftest` | 绿；`CLAIM_SKIP_WORKTREES=1` 红「另一棵工作树用过 E12，本棵 --next 该是 E13，拿到 E6」 |
| `python3 scripts/kb/stale-candidates.py --selftest`、`python3 scripts/gate/apply-writer-patch.py --selftest`、`python3 scripts/agents/agent-cost.py --selftest` | 绿（agent-cost 查了 12 项，其中 outcome 的 18 个子 agent）；`APPLY_WRITER_PATCH_BREAK=checker-section-ignored` 红「动 checker 而报告没那一节应当拒绝，实际 0」；`AGENT_COST_BREAK=narrow-rework-words` 红 1 项 |

判定词：做到 / 做了一半 / 越位 / 没做。「会红的输入」一栏写我造的输入与实际退出码。

## 1　K4 执行员写装置不逐条对照登记自查

- 改法落点：`.claude/agents/experiment-runner.md:40` 第 3c 步（跑产物之前把登记每条读法、量、停机与作废条款列成 `landing-checklist.md`，每行写函数文件:行与会红的用例名，写不出就停下交回）；`.claude/agents/experiment-runner.md:57` 产出加「登记落地清单」一节，句末「没有这一节，报告不算交齐」。
- 挪动核对：旧 3b 里「登记的量一次做不完…部分已跑」那半句现在在 3c 末尾，没丢（diff 里 3b 删了、3c 末尾原样出现）。
- 会红的输入：造一份执行员报告没有「登记落地清单」一节。`grep -rn '登记落地清单' .claude/hooks .claude/gate.d/*.sh scripts` 零命中；handback-guard 对执行员只查绿行有没有数（`.claude/hooks/handback-guard.sh:22` 的 ③），不查这一节。不会红。
- 九十有没有登：登了，`records/2026-09-16-subagent拆分提案.md:1472`（九十 ①，「执行员交回时，报告里「登记落地清单」一节在、每行的文件:行与用例名在仓副本里找得到」，去向「下一批工具改动」）。
- 判定：做到（定义文字侧）；检查侧已登记待做，不算漏。
- 推翻条件：拿到一份定义里 3c 的文字在、却与登记原条款对不上的执行员报告，且它的上游检查本该拦；或发现旧 3b 那半句在别处也没有（我看到它在 3c 末尾）。

## 2　K3 设计员登记前不核可跑性与判别力

- 改法落点：`.claude/agents/experiment-designer.md:33` 第 3a 步（量按定义现算态数与单态耗时得挂钟与峰值内存；每个对照点写预期差与测量分辨率；会触发的事件写触发条件并代入登记几何算；算不出的进第一节末尾「未定项：」，交回主 agent）；产出行加「未定项」。
- 会红的输入：造一份登记，第十四节预计挂钟与第六节态数乘单态耗时差一个数量级。`grep -rn '可跑性' .claude/hooks .claude/gate.d/*.sh scripts` 零命中；门禁 doc-experiments 的 preregistration-parallelism 格只查五个标签行齐不齐、冒号后字数，不对数。不会红。
- 九十有没有登：登了，`records/2026-09-16-subagent拆分提案.md:1473`（九十 ②，加判 preregistration-parallelism 格，差一个数量级红）。
- 判定：做到（定义文字侧）；检查侧已登记。
- 推翻条件：找到一份 2026-10-22 之后新写的登记，第十四节的挂钟与第六节态数明显对不上而设计员没在「未定项」里写——说明 3a 没起作用；或 preregistration-parallelism 格已在别处核了数。

## 3　K1 命名检查交回前不跑

- 改法落点三处：`.claude/agent-common.md:96`（原「子 agent 交回前不跑…各类 lint」后补「命名检查除外，见「命名检查交回前跑」」）；`.claude/agent-common.md:97` 新条（改了 `crates/`、`scripts/` 下 `.rs` 的实现员、执行员、工具员，在自己改动所在的树上跑 `code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names <根>` 与 `naming-lint.sh <根>`，只修命中在自己改动文件里的）；放行侧 `.claude/hooks/lib_heavy_tests.py:706` 的 `NAMING_SELF_CHECK_CELLS` 与 `.claude/hooks/lib_heavy_tests.py:1605` 的 `classify_gate_stage` 分支。
- 定义里的命令照抄能不能过（我用三种 agent_type 逐条喂四道钩子，PreToolUse 的 stdin JSON）：
  - 带 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=…; 范围=4 格; 为什么=交回前核自己改动里的命名'` 的四格 `--check`：run-scope-guard、heavy-test-guard、two-logs-guard、bash-command-detector 全退 0（三种 agent_type 各一遍）。「为什么」一段去掉标点恰 12 个字，刚好够 run-scope-guard 的下限，改一个字就可能掉线，记一笔。
  - `naming-lint.sh <根>`：四道全退 0。
  - 经 `SINGLEFS_RUN_SCOPE=… bash scripts/hosts/two-logs.sh naming -- bash .claude/gate.d/code-source-discipline.sh --check …`：退 0；不带声明：run-scope-guard 退 2，heavy-test-guard 退 0。
- 控制组（该红的真红）：`--check vague-names,format-constants` → heavy-test-guard 退 2；`… --check … > naming.log` 与 `| tee naming.log` → two-logs-guard 退 2；不带声明 → run-scope-guard 退 2。
- 缺口本身（rework.md K1：E187 交回原文「命名纪律我按缩写表自扫过一遍，没有真跑」，随后改 125 处）的失败形态是「说跑了、没真跑」。会红的输入：造一份实现员报告声称自扫过、没有那两条命令的输出。没有任何钩子或门禁核报告里有没有这两条命令的原样输出（`.claude/agent-common.md:97` 末句「两条的命令与末尾原样输出贴进报告」只是文字）。run-scope-guard 会把每条放行的命令追加进检出记录，是现成的核对依据，但没有人读它。不会红。
- 九十有没有登：没有。九十 ①到⑤里没有命名自查这一条。另外三份定义（`.claude/agents/implementation-writer.md`、`experiment-runner.md`、`tooling-writer.md`）里 `naming-lint`、`data-names` 仍零命中（`grep -n 'naming-lint\|data-names\|命名检查' .claude/agents/*.md` 零命中），要求只在共用约束里。
- 判定：做了一半。放行与指令做到了且命令照抄能过；挡「没真跑」的检查没有，也没登。
- 推翻条件：找到一个读检出记录或报告里命名命令输出的钩子、门禁（我 grep 零命中）；或看到改 `.rs` 的子 agent 在这条加进去之后交回里都带了真输出而且无一例外（那是自觉，不是检查）。

## 4　K9 占号不跨工作树

- 改法落点：`scripts/kb/claim-experiment.sh:30` `worktree_roots()`（读 `git worktree list --porcelain`，每棵都查；不在 git 仓只查给定的一棵；在仓里而 git 读不出清单拒绝给号，退 3）；`--next` 取全部工作树最大号加一，占号时别的工作树里用过的号被拒。
- 实现今天的样子：`git worktree list` 列两棵，主工作区与 `/home/fy5090/.local/share/singlefs/work/m5-dev`（分支 m5-dev）。分别单查（`CLAIM_SKIP_WORKTREES=1`）`--next`：主工作区 E198、m5-dev E197；不带开关，主工作区 `--next` 仍是 E198（取两棵最大加一，行为对）。缺口的原案（E185 到 E187 撞号：主工作区和开发分支各取各的号）就是这两棵，已盖住。
- 会红的输入：另一棵工作树里已有 E12，本棵占 E12。`--selftest` 在临时 git 仓加第二棵工作树造这一幕，退 1；`CLAIM_SKIP_WORKTREES=1` 下自证红（见上表）。
- 边角：子 agent 的「副本」是文件拷贝（`.claude/agent-common.md:46` 的「副本只拷要编的」），不在 `git worktree list` 里；但占号由设计员在主工作区跑，副本不占号。另一台机器上的独立克隆也不在清单里，这是原缺口之外的情形，我不算它没做到。
- 九十有没有登：不需要（有会红的自证）。
- 判定：做到。
- 推翻条件：出现一次撞号，且撞的两棵都在 `git worktree list` 里（那说明查号漏了目录，例如新增实验产物放在没查的目录）；或 `scripts/kb/claim-experiment.sh --selftest` 的跨工作树几格在不带开关时红。

## 5　延后项只写进调度记录、没进欠账表

- 改法落点：`.claude/main-agent.md:21` 第 2 条补一句「延后的当时就登记」，要求同一步在里程碑收口表、`checks-owed.md` 或决策正文登一行，调度记录里写「延后」的那一行同时写登的那一行的编号或 文件:行；「只写进调度记录不算延后，算没去向」。
- 会红的输入：造一份 `records/` 新增行，写「延后」而不带 C 号或文件:行。`grep -n '延后' .claude/gate.d/doc-process-records.sh` 零命中，没有格核这个。不会红。
- 九十有没有登：登了，`records/2026-09-16-subagent拆分提案.md:1474`（九十 ③，doc-process-records 加一格，`records/` 新增的带「延后」「下轮开」「下一批」的行，同一行写着 C 号、节号或文件:行且指得到）。
- 判定：做到（文字侧）；检查侧已登记。这条缺口的实质数据是 8/54 无去向、当时就登 6/54，文字要求本身是对症的，但缺口产生的原因（没人在派发时想起）只靠文字改不了多少，检查是真正的关。
- 推翻条件：下一批 `records/` 新增的「延后」行仍有无去向的（那说明文字无效，且检查未建就应升优先级）。

## 6　回扫判出的 `crates/` 行没人接

- 改法落点：`.claude/agents/sweep.md:52` 产出节开头新增「所有种类的报告都单列一节「交实现员」」，目标在 `crates/` 下、判「要改」「要补」的行逐条挑出，没有也写「0 行」和数行命令，`crates/singlefs-checker/invariants/` 下的 `.md` 不进这一节；`.claude/main-agent.md:99`（阶段同步那一行）把「交实现员」的行并进第 4 条的下一批交 `implementation-writer`，不交写回员。
- 与载体搬家一条自洽：不变量清单 `.md` 留在原判定表走写回员，两处（sweep.md:52、main-agent.md 派发表 kb-scribe 行的例外）说法一致。
- 会红的输入：造一份回扫报告，有目标在 `crates/` 的「要改」行而没有「交实现员」一节。`grep -n '交实现员' scripts/kb/stale-candidates.py` 零命中，`--check-report` 不核这一节。不会红。
- 九十有没有登：登了，`records/2026-09-16-subagent拆分提案.md:1475`（九十 ④，`--check-report`：目标在 `crates/`（不变量清单的 `.md` 除外）、判「要改」「要补」的行不在「交实现员」一节里就退非 0）。
- 判定：做到（文字侧）；检查侧已登记。
- 推翻条件：下一轮回扫报告里有 `crates/` 目标的「要改」行没进「交实现员」，且主 agent 也没按 main-agent.md:99 转给实现员。

## 7　载体搬家后写回落空（扫描一侧、写回一侧）

- 扫描一侧落点：`scripts/kb/stale-candidates.py:97` 的 `CARRIER_PATTERNS` 加 `^crates/singlefs-checker/invariants/[^/]+\.md$`。会红的输入：不变量清单里一句现状句，`STALE_CANDIDATES_BREAK=invariants-carrier-missing` 下自证红「不变量清单（crates/singlefs-checker/invariants/ 下）里的现状句没进候选表」；我复跑自证绿。判：做到。
- 写回一侧落点：`.claude/hooks/agent-write-scope.tsv:12` 给 kb-scribe 加 `crates/singlefs-checker/invariants/*.md`；`.claude/agents/kb-scribe.md:42` 写范围同样一行；`.claude/main-agent.md:102` 的「不改 `crates/`」后补例外。会红的输入：写回员写 `crates/singlefs-checker/invariants/cp-01.md`（放行）、`x.rs`、子目录 `sub/x.md`、`crates/singlefs-checker/src/lib.rs`（都拒）。write-guard 自证 4 格，`WRITE_GUARD_BREAK=scope-new-rows-dropped` 下红（见上表）。判：做到。
- 病根还有一半：「写的一方报了的 0」（5/9 静默跳过）。改法是 `.claude/agents/kb-scribe.md:15` 补一句「整份不写，停下：报告「找不到的目标」一节逐条写规格里的文件与旧串」，并要求报告列新位置。会红的输入：造一份写回员报告，规格里有一个文件已被搬走而报告没有「找不到的目标」一节；`grep -rn '找不到的目标' .claude/hooks .claude/gate.d/*.sh scripts` 零命中，没有检查核。不会红。
- 九十有没有登：没有。九十 ①到⑤没有这一条；`.claude/kb/checks-owed.md` 的 C879、C880 也与它无关。
- 判定：扫描一侧做到，写范围一侧做到，「找不到就报」一句做了一半（文字有、会红的检查没有、也没登记）。
- 推翻条件：找到核写回员报告「找不到的目标」一节的钩子或门禁；或下一次写回员遇到目标文件被搬时，报告里确有该节（只能证文字有效，不能证检查存在）。

## 8　报告写法矛盾（工具层拒 Write 建 `report.md`，交回闸又要写文件）

- 改法落点（文件名统一成 `result.md`）：派发闸新判法 ⑰ `.claude/hooks/runner-dispatch-guard.sh:101`（说明）、`.claude/hooks/runner-dispatch-guard.sh:1219` `report_file_name_verdict`、`.claude/hooks/runner-dispatch-guard.sh:1596` 接进 `judge_dispatch`；骨架默认填 `<草稿目录>/result.md`（`scripts/agents/dispatch-prompt.py:101`）；文字侧 `.claude/main-agent.md:86`、`.claude/agent-common.md:108`、`.claude/agents/implementation-writer.md:58`；各钩子与脚本的自证样本 `report.md` 改 `result.md`；`scripts/gate/apply-writer-patch.py:218` 读补丁目录里的 `result.md`（主 agent 自己改，见第 11 行）。
- 会红的输入：派发提示「报告：~/.local/share/singlefs/work/w/report.md」→ 派发闸退 2，`RUNNER_DISPATCH_GUARD_BREAK=report-name-unchecked` 下 stdin 两格红（见上表）；骨架默认被拿掉 → dispatch-prompt 自证红 3 处。判：对 `report.md` 这个名字做到。
- 改完之后全仓还认 `report.md` 的活代码：`grep -rnF 'report.md' .claude/hooks .claude/agents .claude/skills .claude/rules .claude/singlefs-ai-sop .claude/main-agent.md .claude/agent-common.md CLAUDE.md scripts .claude/scripts .claude/gate.d/*.sh`（排掉 `scripts/experiment-apparatus/e*.rs` 的历史产物名）命中里，要求或按名字认 `report.md` 的一处也没有：命中是 ⑰ 的说明与自证、dispatch-prompt 的自证、`scripts/agents/agent-cost.py:1163` 等自证造数据、`scripts/hosts/clean-session-scratch.py` 与 `scripts/hosts/test-environment-check.py` 的自证样本、上游 SOP 的 `handback-scratch-check.sh` 自证样本。这些是数据，不是要求。（K3 的全仓计数归攻方腿。）
- 实测证据（我从会话记录里重做）：2026-10-01 之后子 agent 会话里，被「Subagents should return findings as text」拒掉的 Write 的目标文件名是 `report.md` 211 次、`report-part3.md`、`report-body.md`、`report-5.md`、`report-8.md`、`report-3.md`、`report-2.md` 各 1 次（另有 25 次我没能把工具调用对上目标文件，标「?」）。被拒的名字全部以 `report` 起头。调度记录说的「名字里带 report 的别的文件 16 次里拒 6 次」与此相符：那 6 次就是 `report-*.md`，另 10 次按调度记录的数没被拒（名字中间带 report 的，我没逐个看这 10 个名字）。
- 越位与欠缺：
  - ⑰ 判名字里任何位置带 report（`.claude/hooks/runner-dispatch-guard.sh:105` 「目录名带 report、文件名不带的放行。没有放行行」），比被拒的集合（以 report 起头）宽，会误拦 `final-report.md` 这种实际写得进去的名字。后果小（出路就是改成 `result.md`），但「没有放行行」把误拦也锁死了。
  - `.claude/agent-common.md:108` 写「子 agent 用 Write 建文件名以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 `.md`，或名字里带 report 的文件，会被工具层当场拒，Bash 写的不拦」。审计的数据里只有 report 这一个词，SUMMARY、FINDINGS、ANALYSIS 三个前缀在 `research/prompts/subagent-landing-audit-2026-10-22/` 的各份与调度记录里找不到数据（`grep -n 'SUMMARY\|FINDINGS'` 零命中）；而 ⑰ 不判这三个前缀，派发提示写「报告：…/summary.md」⑰ 放过去。若这句话为真，派发闸漏了三个词；若为假，把没量过的写成了事实。我不能核（上限在 Claude Code 的工具层，本地没有它的规则文件），标「复核不了」。
  - `.claude/hooks/handback-guard.sh:202` 超长交回的拒绝说明写「没给就写进草稿目录」，没点名文件名；缺口原文说的正是「交回闸要写文件」与「工具层拒这个名字」两头相撞，说明里不写 `result.md`，没读共用约束的子 agent 仍可能选 `report.md`。
  - 骨架默认 `result.md` 对三方腿不对：三方腿的报告在 `research/prompts/<轮>-*-output.md`（`.claude/agent-common.md:108` 同一句前半），调度记录第四批已点名，交攻方腿判。
- 九十有没有登：不需要（⑰ 有自证）。三个前缀的覆盖和 handback-guard 说明没登。
- 判定：做到（`report.md` 与 `report-*` 的实际缺口）；有一处越位（过宽的拦）、一处欠缺（三个前缀，需核实后二选一）。
- 推翻条件：任何一次在 ⑰ 放行的派发里，子 agent 写 `result.md`、`progress.md` 被「Subagents should return findings」拒；或拿到工具层对 `summary.md` 的放行记录（则 `.claude/agent-common.md:108` 那句写过头了）。

## 9　结局没有常设的账（`agent-cost.py outcome`）

- 改法落点：`scripts/agents/agent-cost.py:10` 用法行加 `outcome`，实现在 `scripts/agents/agent-cost.py:945` `collect_outcome`、`scripts/agents/agent-cost.py:1019` `outcome_summary`；`scripts/agents/README.md` 的 agent-cost 行补子命令；`.claude/hooks/selftest-registry.tsv` 该行的理由改写。
- 缺口原文三件事：落地与否、被谁用上、返工几次。前两件：写入落地用「git 历史 ∪ 各工作树里的行」比，引用用路径、agent 号、草稿目录名，桶 12 个；自证 12 项绿，`no-path-normalize`、`no-bash-writes`、`narrow-rework-words` 各只红点名那一项。做到。
- 「返工几次」：每个子 agent 只有一个布尔「返工字样」（描述里有词表里的词），没有按链数几次；这条与 rework.md 的链判法不是一回事，是代理量。
- 「常设」：它是按需命令。`grep -rln 'agent-cost.py outcome'` 只在 README、调度记录、本轮材料与 agent-cost 自己里命中；`.claude/main-agent.md`、`.claude/agent-common.md`、任何钩子与技能里没有「什么时候跑它」。没有落盘的账本文件。缺口说的「事后从会话记录里重建，这一轮靠一次性脚本」——一次性脚本换成了可重复脚本，重建仍是每次现跑。
- 会红的输入：不适用（它是报表，不是闸）。
- 判定：做到（可重复的账）；「常设」只做到命令存在，没有调用点，也没落盘；「返工几次」只做了字样标记。
- 推翻条件：能指出一处会定期跑它或读它的流程（我 grep 不到）；或有一份落盘的账本。

## 10　丢掉的那一格：doc-experiments 加 replay-comparison-units（第三批任务 10，原延后项）

- 要关的东西：`.claude/kb/milestone/03-third-txn.md:185` 旧文「另立一条检查：每份重跑登记的比对对象表里的每条正式跑，在单元表和 `replay.sh` 里都找得到」。
- 改法落点：`.claude/gate.d/doc-experiments.sh:2408` `cell_replay_comparison_units`（读 `.claude/gate.d/replay-comparison-units.tsv` 六列，判登记出处在、单元名在单元表出处里作为带双引号字面量恰好出现一次、复跑驱动在 `scripts/experiment-apparatus/replay.sh` 复跑表里且第 4 列等于留存产物名；复跑写 `none:` 的进只缩不涨的滞后表）；三份样本 red / green / zero；`.claude/gate.d/stage-owners.tsv:106` 登记归属；`.claude/hooks/agent-write-scope.tsv:20` 放行执行员写那张 tsv。
- 我独立核了登记表读数：14 行里 13 个不同的单元名，每个在 `e165_replay_bounded_enumeration.rs` 里作为 `"名字"` 字面量各出现 1 次（循环 `grep -cF` 得 13 个 1）。单元表 `formal_run_units` 是 19 个单元；登记表里有 13 个，另 6 个（r3 合法、r5 合法、r5 对抗、r5 多盘、奇数版本的 r5 合法、奇数版本的 r5 对抗）没有行；仓里只有 `e165-r1-prereg.md` 到 `e165-r5-prereg.md`，没有它们的 r6 登记文件（单元表注释写 `driver_e165_r6_r3_legal`）。
- 会红的输入与自证：红样本各摆一种红，八个弄坏开关，主 agent 已逐个判出；我没重跑门禁阶段（子 agent 不跑），只核了格在、样本三份在（`red`、`green`、`zero` 目录），读数以主 agent 的为准，标「复核不了：门禁阶段归提交前」。
- 做到的：形态立了，格会判已登记行的三样；格文件头自己写明「管不到」。
- 没做到的（与旧文的「每份…每条…」比）：
  1. 格只读手填的 tsv。`.claude/gate.d/doc-experiments.sh:2405` 自己写「管不到：登记散文里的比对对象表有没有照实抄进机读表」。tsv 只有 E165 r5 的 14 行，r1 到 r4 不登记，r5 比对对象表第 15 至 19 行没有单元，不登记；上面那 6 个单元也没行。新写一份有比对对象表的重跑登记而不往 tsv 登记，这一格照样绿。
  2. 没有一份定义要求登记：`grep -n '比对对象' .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/rules/*.md` 零命中，`replay-comparison-units` 在 `.claude/agents`、共用约束、主 agent 文件里零命中。tsv 文件头那句「新写的重跑登记里有比对对象表的，同一次改动里往那张表登记」是唯一要求，只有读到 tsv 的人看得到。
  3. `.claude/hooks/agent-write-scope.tsv:20` 的备注写「定义的「写范围」由第 11 号任务补」，任务 11 没补：`.claude/agents/experiment-runner.md:50` 起的写范围一节里 `replay-comparison` 零命中。表有行、定义没字。门禁 code-tooling 的 agent-write-scope 格只核每个定义至少有一条表行、表里的 agent 名都有定义（`.claude/gate.d/code-tooling.sh:1040` 的判据 ③④），不逐行核定义里写没写，所以不会红。
  4. `.claude/kb/milestone/03-third-txn.md:185` 现状句写「跨实验的检查已立」，后面接着写「别的实验的重跑登记还没有比对对象表」。这句是诚实的，但「已立」读者容易理解成旧文那条检查立了；实际立的是它的子集。
- 九十有没有登：没有。`.claude/kb/checks-owed.md:560`（C879）登的是 E165 r5 第 15 至 19 行没有单元，`.claude/kb/checks-owed.md:561`（C880）登的是三条复跑行期望原产物；上面 1、2、3 与那 6 个单元没行，在 C879、C880、九十里都没有。
- 判定：做了一半（格是真的、会判、有自证；覆盖只有手填的 14 行，没有人被要求填，写范围的字没补，缺的没登）。
- 推翻条件：找到一份定义或钩子要求设计员、执行员往 tsv 登记（我 grep 零命中）；或一份门禁在重跑登记含比对对象表而 tsv 没登时红；或 `experiment-runner.md` 写范围里补了那张 tsv。

## 11　主 agent 自己改的 `scripts/gate/apply-writer-patch.py`

- 动机（调度记录「第三批交回」连带）：文件认补丁目录里的 `report.md`，实现员改交 `result.md` 后会被它拒。
- 改法落点：`scripts/gate/apply-writer-patch.py:15` 文件头说明、`scripts/gate/apply-writer-patch.py:218` 读 `result.md`、拒绝句、自证样本。
- 会红的输入：补丁动了 `crates/singlefs-checker/src/` 而 `result.md` 没有「受影响的层 0 流与崩溃枚举用例」一节 → 拒；`APPLY_WRITER_PATCH_BREAK=checker-section-ignored` 下自证红「动 checker 而报告没那一节应当拒绝，实际 0」（我复跑到了）。
- 边角：只认 `result.md`，没有读 `report.md` 的过渡。工具层本来就拒 Write 建 `report.md`，旧名的补丁目录只可能是 Bash 写的；碰到时拒绝句点名 `result.md`，可恢复。不算缺陷。
- 判定：做到。
- 推翻条件：一份交 `result.md` 的实现员补丁被它拒；或 `scripts/gate/merge-and-verify.sh` 之类别的读者仍按 `report.md` 找（我 grep `scripts/gate/` 没有）。

## 12　主 agent 自己改的 `.claude/agent-common.md` 两日志一句

- 改法落点：`.claude/agent-common.md:97` 「输出不重定向到 `.log`（`.claude/hooks/two-logs-guard.sh` 拒），要落盘就经 `bash scripts/hosts/two-logs.sh <名> -- <命令>`」。
- 与工具核对：`scripts/hosts/two-logs.sh:6` 用法 `two-logs.sh [--quiet] <名> -- <命令…>`，写法对。我喂两日志闸：`> naming.log`、`| tee naming.log` 都退 2；经 two-logs.sh 包装退 0。
- 一处说宽了：two-logs-guard 只拒门禁、cargo test、四个入口脚本与 gate-staged.sh 的输出写 `.log`（`scripts/hosts/two-logs.sh:24` 起的配套钩子说明），`naming-lint.sh <根> > n.log` 我喂过，退 0。共用约束写成「输出不重定向到 `.log`」比钩子严；对子 agent 无害（多守一条），但读者会以为钩子管着，实际管的是门禁那条。
- 判定：做到，措辞比钩子宽一圈，可留。
- 推翻条件：`naming-lint.sh` 的输出重定向到 `.log` 被某个钩子拒（我喂的是 two-logs-guard 一道，其他钩子对这条命令都退 0）。

## 13　主 agent 自己改的 `scripts/agents/agent-cost.py` 返工词表

- 动机：任务 13 只用调度记录里的五个词，数出 155；主 agent 查出「337」是宽词表数的，换成严格词表。
- 改法落点：`scripts/agents/agent-cost.py:341` `REWORK_WORDS`，文件头 `scripts/agents/agent-cost.py:30` 「「后续批」这类只说次序的不算」；自证加「返工词表」，开关 `narrow-rework-words` 红这一项（我复跑到，红 1 项）。
- 我复跑的读数：对 2026-10-01 以来的 2287 份子 agent 的 `.meta.json` 描述套这个正则，命中 354（调度记录写 353，差 1 在会话起点的取法，同一数量级；K4 归攻方腿）。
- 「严格」说过头：命中里 14 个只靠 `修订`（例「E185 换 CPU 时钟修订」「E177 跑前登记修订」）、`修改` 与 `fixture(s)`（例「C531 fixture registrations excluded」）命中，是登记修订和样本名，不是返工。正则里 `修` 单字、`\bfix` 都是松的。占 354 的 4%。自证的「不算」清单（「后续批」「第 1 次」等五条）里没有 `修订`、`fixture`。
- 判定：做了一半。方向对（从 337 的宽词表收到列全的词表、加了自证），但仍是上界，词表里的 `修`、`fix` 会误收登记修订与样本名；结果里「返工字样」应读作上界，不能读作返工数。
- 推翻条件：把 `修订`、`fixture` 加进自证的不算清单且把正则收紧后，354 掉得远不止 14（那说明我低估了误收）；或抽 30 个命中逐个看都是真返工。

## 判定一览（K1）

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 K4 执行员对照登记自查 | 做到（文字）；检查已登记九十 ① | runner 3c 步与产出一节在，无钩子核报告里有这一节 |
| 2 K3 设计员核可跑性 | 做到（文字）；检查已登记九十 ② | designer 3a 步在，preregistration-parallelism 格不对数 |
| 3 K1 命名检查交回前跑 | 做了一半 | 放行与指令做到，命令照抄四道钩子都过、控制组真红；挡「说跑了没真跑」的检查没有，九十没登，三份定义本身零命中 |
| 4 K9 占号跨工作树 | 做到 | `git worktree list` 每棵都查，自证与弄坏开关红；原案两棵都盖住 |
| 5 延后项当时就登记 | 做到（文字）；检查已登记九十 ③ | main-agent.md:21 一句，无格核 |
| 6 回扫 `crates/` 行 | 做到（文字）；检查已登记九十 ④ | sweep.md 单列「交实现员」，main-agent.md 派发表接上 |
| 7 载体搬家 | 扫描一侧、写范围一侧做到；「找不到的目标」一句做了一半 | 两侧自证真红；写回员「找不到就报」只有文字，无检查、九十没登 |
| 8 报告文件名 | 做到（`report.md`）；一处过宽、一处欠缺 | ⑰ 判任意位置带 report，比被拒的（以 report 起头）宽；agent-common.md:108 称 SUMMARY 等三前缀也被拒，无数据、⑰ 不判；handback-guard 拒绝说明没写 `result.md` |
| 9 结局账 outcome | 做到（可重复的账）；「常设」「返工几次」只做了一部分 | 无调用点、无落盘账本，返工只有字样布尔 |
| 10 丢掉的那一格 replay-comparison-units | 做了一半 | 格真、自证真；只读手填 14 行，无定义要求登记，写范围的字没补，6 个单元无行，都没登记 |
| 11 apply-writer-patch.py | 做到 | 读 `result.md`，自证与开关红 |
| 12 agent-common 两日志一句 | 做到 | 写法与 two-logs.sh 相符；措辞比两日志闸严 |
| 13 agent-cost 返工词表 | 做了一半 | 词表仍是上界（`修`、`\bfix` 误收，我数出 354 中 14 个是登记修订与样本名） |

## 没做什么

- 没判 K2（合并与重合）、K3（矛盾，含全仓 `report.md` 的完整计数）、K4（数能不能核）；`report.md` 的 grep 我只用来回答「改完有没有还要求这个名字的」。
- 没跑门禁阶段、没跑任何 lint 整仓、没编译、没重跑 `doc-experiments.sh` 的新格（子 agent 不跑；红样本、八个弄坏开关、真仓单跑读数以主 agent 的为准，标「复核不了：门禁阶段归提交前」）。
- 没核 Claude Code 工具层对 `summary.md`、`findings.md`、`analysis.md` 的行为（本地没有它的规则文件）。我想再数一遍被拒名字的全集，扫描会话记录超时（110 秒）被杀，所以第 8 节的名字清单只是我跑通的那一次（含 25 次对不上目标文件的）。
- 没判 `lib_heavy_tests.py`、`heavy-test-guard.sh`、`selftest-registry.tsv`、`write-guard.sh` 的 diff 里非这一批的段落。我看到的非这一批：lib_heavy_tests.py 与 heavy-test-guard.sh 里的 `scripts/experiment-apparatus/replay.sh` 按名字判段（`EXPERIMENT_REPLAY_SCRIPT`）、selftest-registry.tsv 里 replay.sh、doc-text、code-tooling、e197 的几行、runner-dispatch-guard.sh 的 TaskStop 段、stage-owners.tsv 里 same-origin-counter 等行，调度记录与正文都没点名，按「别的会话的未暂存改动」处理，没判。
- 不采纳、不出判决；以上都是对 K1 的正推读数。
