# landing-audit-r2 辩方腿（Sonnet）报告：K1 第一轮判决够不够得着、是不是同样打中替代方案

日期：2026-10-23。立场：辩方。被复核：判决 `research/prompts/landing-audit-r1-main-verification.md`，依据的腿报告 `research/prompts/landing-audit-r1-sonnet-output.md`、`research/prompts/landing-audit-r1-opus-output.md`；被判的改动是第五批三份交回（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md`、`r1fix-definitions.md`、`r1fix-hooks.md`）。方案按 `crates/` 与 `scripts/` 今天的实现谈：这一批没改 `crates/`；开工快照 `sha256sum -c research/prompts/landing-audit-r2-snapshot/judged-sha256.txt` 此刻全部 OK。我没改仓里被判的任何文件，没跑门禁阶段，没编译。

## 复跑

模型目录 `research/prompts/landing-audit-r2-sonnet-model/`。命令（工作目录仓根；整条经内存包装，读约 12 GB 会话记录，峰值不到 4 GiB）：

```
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=120 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G \
  bash research/prompts/landing-audit-r2-sonnet-model/tools/rerun.sh --repo "$PWD" --frozen "$PWD/research/prompts/landing-audit-r2-snapshot" --work <新的草稿目录>
```

跑完约 25 秒，退 0，输出与 `out/` 下同名文件逐行比（会话记录随此刻变，条数会略涨，口径不变）。闭窗口结局账另跑一条（约 2 分钟，含排队），输出 sha256 要等于存档那一份：

```
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=200 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 7G \
  python3 scripts/agents/agent-cost.py outcome --since 2026-10-01 --until 2026-10-21 --summary
```

| 文件 | sha256 |
|---|---|
| tools/rerun.sh（模板只改 rerun_steps） | e5c13fd9675b6351f65713ea56f235808191827ca45abf97ba65f071f653b426 |
| tools/scan_report_names.py | 71d17feac3d0b21dff8532fbf8174a02591a4ed73494c5812b1ffac23a062780 |
| tools/rework_words.py | d4eb83ec78cc8afdb3f38beda03e37b49a3e0a324d6aed824115bf918fd74e2b |
| tools/refused_form.py | 29cbb741e05f7a901d11eac34e434fcfd68a0c2b0baf8fce791e98bfea34c00c |
| tools/claim_numbers.sh | 82954de862c62542803c230797bd4a78e08366dd0b85964c0f71abd9ee2ef6f3 |
| out/names-all-summary.txt | c8fdd36d0907ea5369a01fc6be7e55df3012ea41f40609f1d028b31619fc9611 |
| out/rework-words.txt | bb781d8e0f34b0ede380670a368d96272f24973d40dd3b1711171475d3ac5c5d |
| out/refused-form.txt | 5893da92e63852e4bb6293119eb24f0c6e3cfe073230872ff999268b97911010 |
| out/claim-vs-bin.txt | 26a4fd5739ca1edc2443abf413a23876136a4f49697a2e6d76270768481ecd93 |
| out/outcome-closed-window-rerun.txt | 6dde89a84129fa1b3b22da055372594d71bb11b262df5f2979c20db5a92e6ab0 |

模型是读会话记录的统计脚本，不是装置：它们不在 `replay.sh` 里，数是这一次真跑出来的，不进 kb。

## 判定一览

判定词：站得住（理由打中被改的那一处，且放到判决没选的改法上不翻）／够不着（理由只罩住被改处的一部分，或改出来的样子丢了缺口的一类）。同一理由放到别的臂上也成立的，写在「依据」里。

| 判决条目 | 辩方判定 | 一句话 |
|---|---|---|
| 正推 3（采，九十节 ⑥） | 站得住 | 缺口真；⑥ 登记的检查形态（核报告文字）比没选的形态（核子 agent 自己的会话记录）弱 |
| 正推 7（采，九十节 ⑦） | 站得住 | 只有文字的保护，⑦ 登了 |
| 正推 10（采，这一批改） | 够不着一半 | 6 个单元与归属办得对；格对手填表之外的盲区只有文字、九十节没登 |
| 正推 13（采，同 K4-2） | 站得住（幅度小） | 只拿掉「修订」9 条；剩下的命中里 47.6% 只靠单字「修」或 fix |
| 正推 8（不采） | 站得住（理由要改写） | 结论对；判决引的「16 次里拒 6 次」分母混进了 .py、.rs，.md 名字的数据更强地支持不采 |
| K2-1（采，用户定七处） | 站得住 | 第五批找到第八处并登 ⑪；全树再没有第九处 |
| K2-2（采） | 站得住 | 滞后表删净，活文件里零引用 |
| K2-3（采） | 站得住 | 库名与内容不再相符，不改结论 |
| K2-4（采） | 站得住 | 两条标签集合并成一份；⑰ 误拦面随标签涨，已有一个否定句误拦的实例 |
| K2-5（采） | 站得住 | 读登记位更好；扫描「子目录也算」与写范围「不跨目录」今天无差 |
| K2-6（采） | 站得住 | 看门狗变宽在 3266 条真实交回结果上与旧判法逐条同判 |
| K2-7（不采） | 站得住 | 并不并都不出错；判决的理由是「攻方自己判值不大」，更实的理由我补在下面 |
| K2-8（不采） | 站得住 | 占号目录表今天没漏任何入库装置的号；⑨ 已登 |
| K3-1（采） | 站得住，实现偏离判决原意 | 缺口关住；类型表在三处各抄一份，没写进 frontmatter，判决自己的文件划分让这条做不到 |
| K3-2（采） | 站得住 | 两处文字一致；共用约束同一行的后半句有反例 |
| K3-3（采） | 站得住 | 判决写「两份」，应是三份；第五批多改一份是对的 |
| K3-4（采） | 站得住 | 依据在现行行号上复核过；子 agent 自己建不了 worktree |
| K3-5（采） | 站得住 | 与 CLAUDE.md 一致 |
| K3-6（采） | 站得住 | 6 个单元对着 r5 登记逐个核过 |
| K3-7（采） | 够不着 | 依据是攻方自陈的读法；项目自己的第 2 步允许「算出来、列出来」；改写丢了 E185 判据退化那一类 |
| K3-8、K3-9（采，弱） | 站得住 | 文字对齐，成本低 |
| K4-1（采） | 站得住 | 闭窗口重跑与存档逐字节相同；「往后复跑得出同一组数」说过头 |
| K4-2（采） | 站得住（幅度小） | 同正推 13 |
| K4-3、K4-4（采） | 站得住 | 调度记录两处已照改 |
| 五·1、五·2（没核动的） | 不改变采 | 依据在现行文件里复核过；闭窗口重跑补上了五·2 |
| 五·3（没核动的） | 不改变不采 | 6 个名字我替它逐个看了 |

26 条判决里：24 条站得住（多数带说明或改进），2 条够不着一半以上（正推 10、K3-7）。

## 一、判决节「二、逐格判」每一条「采」

### 正推 3：命名检查挡不住「说自查过、其实没跑」（采，进九十节）

- 判决原文：「采：没有检查核报告里有没有两条命令的输出，九十节没登」（`research/prompts/landing-audit-r1-main-verification.md:31`）。登在 `records/2026-09-16-subagent拆分提案.md:1477`（九十节 ⑥）。
- 结论：站得住。缺口真（共用约束只是文字，`.claude/agent-common.md:97` 末句要贴输出，无处核）。采的做法是登记，不是现在建检查，对这件事合适。
- 同一条理由放到没选的检查形态上：⑥ 写的形态是「交回闸核执行员、实现员、工具员的报告里有没有那两条命名检查命令与它们的末行输出」。报告是文字，子 agent 想贴就能贴；而交回闸已经在开子 agent 自己的会话记录：「own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl")」（`.claude/hooks/handback-guard.sh:99`，另一处在 `:164`）。核「会话记录里有没有 Bash 调用了这两条命令」同价、更难糊弄，而且正好对着缺口原案（E187 交回原文「没有真跑」）的失败形态。判决没选这一形态，登记行（⑥）也没提。
- 推翻条件：交回闸读会话记录认不出命名检查的调用（例如被 `two-logs.sh` 包装后命令串对不上），那报告文字形态更稳。

### 正推 7：写回员「找不到的目标」只有文字（采，进九十节）

- 判决原文：「只有文字 | 采 | 九十节加一条」（`research/prompts/landing-audit-r1-main-verification.md:32`），登在 `records/2026-09-16-subagent拆分提案.md:1478`（⑦）。
- 结论：站得住。`.claude/agents/kb-scribe.md:49` 写着「没有就写 0 条」，正是这次审计整体盯着的「写的一方报了 0」的形态；⑦ 把检查放在交回闸、对着规格里点名的目标在主工作区存不存在，不依赖写的一方自报，是对的。
- 推翻条件：写回员报告里「找不到的目标」一节出现且被某个钩子核过。

### 正推 10：比对登记只读手填的 14 行（采，这一批改）

- 判决原文：「比对登记只读手填的 14 行，6 个单元没有行，没有定义要求登记」（`research/prompts/landing-audit-r1-main-verification.md:33`）；K3-6 的要求：「补上缺的 6 个单元，或逐个写明为什么不是比对对象」（同文件 `:58`）。
- 做到的（我核了）：执行员定义第 5 步与写范围点名了登记表（`.claude/agents/experiment-runner.md:44`、`:52`）；归属改成执行员（`.claude/gate.d/stage-owners.tsv:106`）；6 个单元在登记表文件头逐个写了不登记的理由（`.claude/gate.d/replay-comparison-units.tsv:15` 起）。我对着登记 `research/prompts/e165-r5-prereg.md` 第十一节末尾的比对对象表复核：19 行里 r3 档只有对抗一族（`:679`，`r3-segment1-adversarial-family.out`）与择根，没有合法一族；r5 档的 5 个单元是第三段新跑的产物（被比的一侧），不在「上一次」那一列。所以「不登记」判得对，且比补 6 行好——补了就是往表里塞不是比对对象的行。
- 够不着的一半：正推 10 的核心是格只读手填表，新登记漏填格看不见。格自己写着「管不到：登记散文里的比对对象表有没有照实抄进机读表」（`.claude/gate.d/doc-experiments.sh:2393`）。第五批的补救是执行员定义里的一句文字。对正推 3、正推 7 这类「只有文字」的保护，判决都要求九十节立检查；对这一条没有。命令：`awk '/^## 九十、/{f=1} f' records/2026-09-16-subagent拆分提案.md | grep -c 'replay-comparison\|比对对象\|管不到'`，输出 0（九十节现有 ①到⑭ 共 14 行，没有一行谈这个盲区；欠账表 C879、C880 只谈 r5 第 15 至 19 行没有单元，不是这件事）。
- 同一条理由（文字要求无检查）放到这里同样成立，判决没落到。建议的登记行：门禁 doc-experiments 的 replay-comparison-units 格加判「重跑登记里比对对象表的行数 ≤ 登记表里同一登记出处的行数 + 登记表头注点名不登记的行数」之类，形态由主 agent 定。
- 推翻条件：九十节之外已有处记了这个盲区；或比对对象表有了固定形态，格能直接解析它。

### 正推 13 与 K4-2：返工词表不认「修订」（采）

- 判决原文：「采（攻方 K4-2 同） | 这一批改」（`research/prompts/landing-audit-r1-main-verification.md:34`）；K4-2 行：「词表改成不认」（同文件 `:68`）。改在 `scripts/agents/agent-cost.py:355` 的 `REWORK_WORDS`（单字「修」后加否定前瞻）。
- 结论：打中了被改的那一处，站得住；幅度小。我用 `tools/rework_words.py` 对 2026-10-01 起（按文件修改时刻）的 2304 个子 agent 描述套新旧两张词表：旧（含修订）命中 358，新命中 349，差 9。
- 够不着的部分：新词表的 349 里，166 条（47.6%）只靠单字「修」或 fix 命中。只靠单字「修」的 72 条里，后面接补 16、红 8、空格 15（例「E183 修 cells 脚本自证一格红」「工具员：定义修补与重型闸判法」「钩子三份修补」）——这些派发本身的活就是修某样东西，不是重做前一个子 agent 的产物，和「修订」是同一类误报；另有 5 条靠 `fixture` 命中（「C531 fixture registrations excluded」之类）。判决采的理由（登记修订不是返工）放到「修补」「修红」「fix」上一样成立，判决没选，也没要求把它们记为已知误报。
- 所以「返工约 14%」读作上界对，但这一次改动只把上界收低了不到 3%（9/358）；真正的误报源留给攻方腿 K3 的词表格。
- 推翻条件：抽 30 条只靠「修」或 fix 命中的描述逐个看，多数确是重做上一个子 agent 的产物（那就不是误报）。

### K2-1：「滞后表与基准比」读法并成一份库（采，用户定七处一起并）

- 判决原文：「这一批只并同一道门禁 doc-experiments.sh 里的两处」（`research/prompts/landing-audit-r1-main-verification.md:41`），用户 2026-10-22 改定七处（`research/prompts/landing-audit-r1-main-verification.md:90` 「用户 2026-10-22 定」那一句）。
- 结论：站得住。七处当初是攻方 grep 的数；第五批交回报告写明另找到第八处没并：「listed = git('ls-tree', '--name-only', base, '--', pending_rel)」（`.claude/gate.d/doc-experiments.sh:1069`），已登九十节 ⑪（`records/2026-09-16-subagent拆分提案.md:1482`）。我在全树再 grep `ls-tree`（`.claude/gate.d`、`.claude/hooks`、`scripts` 下的 .sh、.py）：除这一处，只剩 `.claude/gate.d/doc-experiments.sh:1903`（读 HEAD 里 `research/results` 的文件清单，是另一件事）与 `scripts/kb/` 下三处不同语义的读取，没有第九处同形写法。
- 同一条理由放到没选的改法（只并两处、五处留给以后）：用户已经选了，且理由（同一件事各抄一份）在五处上一样成立，所以不翻。
- 小处：`replay_rows` 放进了 `lib-lag-baseline.py`（`.claude/gate.d/lib-lag-baseline.py:128`、`:154`），库名已不描述内容；攻方给过的另一个去处是 `lib-item-ref-status.py`，交回报告给了没放那里的理由（它管 kb 引用）。不影响结论。
- 没核到的：并后七处的行为是否一字不差。证据只有作者草稿里逐段跑样本和主 agent「对着同一棵树输出逐行相同」；后者只覆盖真仓上基准里有这张表的分支，取不到基准、基准里没有表这两个分支靠前者。四道门禁的样本档在阶段里没跑过（见本报告「第五批的偏离与遗留」G1）。这归攻方腿 K2，我只记。

### K2-2：删滞后表，「只缩不涨」改比登记表里 none: 的行（采）

- 判决原文：「采：删 replay-comparison-units-lag.tsv，」（`research/prompts/landing-audit-r1-main-verification.md:42`）。
- 结论：站得住。`grep -rln replay-comparison-units-lag . --exclude-dir=.git --exclude-dir=target` 命中的只有 `records/`、`research/prompts/` 下的材料与冻结证据，`.claude/` 与 `scripts/` 活文件零命中；kb 三语第 185 行主 agent 已改。登记表头注写明复跑写 none: 的行与基准按（登记出处、留存产物名）成对比（`.claude/gate.d/replay-comparison-units.tsv:8`）。
- 同一条理由放到没选的改法（保留滞后表、另立批准人）：攻方自己的推翻条件是「滞后表另有归属」，`.claude/gate.d/stage-owners.tsv:106` 整格一组归属，没有这种分工，所以不翻。
- 推翻条件：出现必须由登记表之外的人批准 none 行的流程。

### K2-3：复跑表读法一份（采）

- 判决原文：「放进同一份库，同一道门禁四处都调它」（`research/prompts/landing-audit-r1-main-verification.md:43`）。
- 结论：站得住。交回报告写四处各 254 行逐行相同（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md` 「K2-3　复跑表一份读法」一节）；新格原来认任意大写字母开头、第 5 列不限，并后收紧成 `E<数字>` 与 exact 或 timing，今天的 `replay.sh` 上无差，这是行为收紧，不是等价替换——`replay.sh` 将来加不属这个形状的行，新格会漏看它。我没复跑门禁格，采信交回与主 agent 记录。
- 推翻条件：`replay.sh` 加了一行形状不属 `E<数字>|…|exact 或 timing` 的复跑行而新格没报。

### K2-4：派发闸 ④ 与 ⑰ 共用一份报告行解析（采）

- 判决原文：「采：一个报告行解析，认」（`research/prompts/landing-audit-r1-main-verification.md:44`），同一行：「路径同时交 ④（写范围）与 ⑰（文件名）」（`research/prompts/landing-audit-r1-main-verification.md:44`）。
- 结论：站得住，打中了攻方量到的两个洞（④ 不判裸「报告：」、⑰ 不判「报告路径：」）。我现跑：clerk 骨架只改报告行，`dispatch-prompt.py --check` 对 `报告：<草稿目录>/result.md` 退 0；对三方辩方，报告行写 `research/prompts/x-r2-sonnet-output.md` 退 0、写草稿目录下 result.md 退 3、一行报告行都没有退 3（输出写了两条：缺必需输入「报告」，加一行报告行都没有）。
- 判法变宽带来的误拦（第五批交回报告 `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:210` 「正文里任何带扩展名、名字里带 report 的词都拒」已披露）：报告行里写否定句也拒。现跑：`报告：<草稿目录>/result.md（不要叫 report.md）` 退 3，点名「report.md」。这个行为改前就有（旧 ⑰ 也扫行里每个词），第五批把标签从一种扩到四种、允许句中，误拦面随之放大。判决没要求扩词，但「路径同时交 ④ 与 ⑰」要的是路径，不是行里每个词——⑰ 若只判 `named`（正文里第一个带 / 或扩展名的词）就不误拦。归攻方腿 K2 的「误拦」格；我只记这一个实例。
- 同一条理由放到没选的改法（只补 ⑰ 的标签、不动 ④）：④ 的洞同样留着，所以合并更好。

### K2-5：候选扫描读 `.claude/kb-directories`（采）

- 判决原文：「读登记位这一批改；新格进九十节」（`research/prompts/landing-audit-r1-main-verification.md:45`）。
- 结论：站得住。现跑 `python3 scripts/kb/stale-candidates.py --selftest` 退 0；改法不改变今天的候选集（交回报告写 `--benchmark` 4190 行不变）。判决没选的改法（保留写死的两行）在「下一次载体搬家」上一定复发，同一理由只对写死的成立。写范围一侧登九十节 ⑧（`records/2026-09-16-subagent拆分提案.md:1479`）。
- 一处不对齐：扫描一侧「子目录也算」——「return CARRIER_PATTERNS + [rf'^{re.escape(directory)}/.+\.md$' for directory in kb_directories(repository_root)]」（`scripts/kb/stale-candidates.py:179`）；写回员写范围只有 `crates/singlefs-checker/invariants/*.md`（`.claude/hooks/agent-write-scope.tsv:12`，`*` 不跨目录）。今天 `find crates/singlefs-checker/invariants -name '*.md'` 的 10 份全在一层（路径深度全是 4），无差；以后目录下建子目录，扫描会把写回员写不了的行收进候选。⑧ 要立的格得先定「子目录算不算覆盖」。
- 推翻条件：不变量清单目录下出现子目录里的 .md。

### K2-6：交回与起点一个定义，抽成会话记录库（采）

- 判决原文：「下抽一份子 agent 会话记录的库」（`research/prompts/landing-audit-r1-main-verification.md:46`）。
- 结论：站得住。最该防的是看门狗把认交回从「结果带 success:true」放宽成「结果不是错误」会不会判反。我用 `tools/refused_form.py` 扫了全部 3286 份子 agent 会话记录里 SubagentHandback 的 tool_result：3071 条 is_error 假、内容 `{"success":true}`；173 条 hook 拒（is_error 真）；20 条 classifier 拒（is_error 真）；2 条 is_error 假而内容是 `{"success":false}`（重复交回，「already delivered」）。新旧判法在前三类上逐条同判；第四类由库的内容判接住（`scripts/agents/subagent_transcript.py:56` 的 `handback_result_is_error`），而且那两条之前已经有一次送达，最近一次送达的取值不被覆盖。所以变宽今天没有判反的实例。
- 同一条理由（三份口径不一致）放到没选的改法（各自保留）：攻方量到今天差 0 个，价值是防将来；现在三处和库一同红绿（交回报告的 crosscheck 与开关），没引入新分歧。
- 推翻条件：出现 is_error 假、内容不带 success 字段而其实没送达的交回结果。

### K3-1：三方腿报告默认值（采）

- 判决原文：「每个类型的报告路径形态写进定义 frontmatter，骨架照它填默认」（`research/prompts/landing-audit-r1-main-verification.md:53`）。
- 结论：缺口关住，站得住；第五批的样子不是判决写的那一种。现跑（三方辩方骨架，别的占位填满）：报告行写 `research/prompts/x-r2-sonnet-output.md` 退 0；写草稿目录下 `result.md` 退 3，说明点名「报告行点名的文件不是这一形态」；删掉报告行退 3。骨架现在打 `报告：research/prompts/<填 轮>-<填 模型>-output.md`。派发闸放过去、交回闸拒的那条链（攻方实测）不再存在。
- 实现与判决的差：仓里没有 `report-path` 这个 frontmatter 键（`grep -rn report-path .claude/agents scripts/agents .claude/hooks | wc -l` 输出 0）。三方类型表现在在三处各抄一份：「THREE_WAY_REPORT_TYPES = ("three-way-forward", "three-way-attack", "three-way-defense")」（`.claude/hooks/runner-dispatch-guard.sh:1291`）、同一行同一内容（`scripts/agents/dispatch-prompt.py:116`）、交回闸另一份且多一个类型：「CITATION_CHECKED_AGENTS = {"three-way-forward", "three-way-attack", "three-way-defense", "three-way-verifier"}」（`.claude/hooks/handback-guard.sh:70`；`.claude/agents/` 下已没有 three-way-verifier 的定义）。
- 为什么偏离不怪工具员：判决自己把 K3-1 派给「钩子与脚本」那一份（`research/prompts/landing-audit-r1-main-verification.md:80`：「| 钩子与脚本 | K2-4、K2-6、K3-1、K3-6 的写范围表、K4-1、K4-2 |」），而这一份的文件清单不含 `.claude/agents/*.md`；定义那一份（`research/prompts/landing-audit-r1-main-verification.md:79`）列的是「K3-6 的定义一侧、K3-7、K3-8、K3-9」，不含 K3-1 的 frontmatter。frontmatter 键归谁写，判决的文件划分没分给任何人，要求本来做不到。定义那一份只给攻方、辩方补了 `required-inputs` 的「报告」（`.claude/agents/three-way-defense.md:8`），钩子那一份用常量兜。
- 同一条理由（各处各抄一份判法，将来漏改一处就矛盾）：K2-4、K2-6 就是按这条采的，放到这三份类型表上同样成立。今天三份对三种类型一致，新增第四种三方类型时要改三处；没有自证会在漏改时红。判决没选的另一条路（只在骨架里给占位）和第五批做的一致，frontmatter 那条路没人走。
- 推翻条件：下一次新增三方类型时三处同步改完且自证红过一次，说明常量表也够用。

### K3-2：「一律 result.md」没给三方腿留例外（采）

- 判决原文：见 `research/prompts/landing-audit-r1-main-verification.md` 逐格判 K3 表 K3-2 一行。两处已改一致：「派发提示里的报告路径，文件名里不带 report」（`.claude/main-agent.md:86`）与共用约束 `.claude/agent-common.md:108`。站得住。
- 没改的半句：`.claude/agent-common.md:108` 仍写「或名字里带 report 的文件，会被工具层当场拒」。数据反对这半句，见下面「正推 8」。

### K3-3：命名检查除外（采）

- 判决原文：「采：两份定义那一步各加」（`research/prompts/landing-audit-r1-main-verification.md:55`）。
- 结论：站得住，判决数错了。共用约束点名的类型有三个（`.claude/agent-common.md:97`：「implementation-writer」「experiment-runner」「tooling-writer」），第五批同时改了 `implementation-writer.md`：「共用约束「门禁」一节的命名检查除外」（`.claude/agents/implementation-writer.md:40`），是对的；只改两份，实现员仍与共用约束冲突，而「与定义冲突时以定义为准」。`grep -n '门禁阶段' .claude/agents/*.md` 里「交回前不跑」的类型：kb-scribe（不改 .rs，不适用）、implementation-writer、tooling-writer、experiment-runner，现在三份相关的全带除外。
- 连带的一处遗漏：`implementation-writer.md` 不在这一轮被判的文件清单里（`research/prompts/landing-audit-r2-snapshot/judged-sha256.txt` 无此行），而清单里的 `three-way-forward.md` 这一批没有改动（`git diff --stat -- .claude/agents/three-way-forward.md` 空）。攻方腿看不到这份改动，主 agent 要么补进被判清单，要么自己看一眼那一行。

### K3-4：命名检查在主工作区或 git worktree 副本上跑（采）

- 判决原文：「共用约束那一条改成在主工作区或 git worktree 副本上跑」（`research/prompts/landing-audit-r1-main-verification.md:56`）；依据（判决没复跑攻方的最小副本，见「五、没核动的」）：「第 316 行用 `git ls-files` 列文件」（`research/prompts/landing-audit-r1-main-verification.md:21`）。
- 依据在现行文件上复核：词表文件读取在 `.claude/gate.d/code-source-discipline.sh:297`（`WORDS_FILE`），列文件的 git 调用现在是「"ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "crates", "scripts/experiment-apparatus"」（`.claude/gate.d/code-source-discipline.sh:319`）。行号比判决写的 294、316 各挪了 3 行（暂存区里的那一版就是 294、316，第五批在文件头加了开关说明），判决当时对。`--others` 在，所以 worktree 里子 agent 新建、未跟踪的 .rs 也判得到；rsync 副本没有 `.git` 与词表，判不到——判决的取舍对。
- 够不着的一处：判决写的另一个去处「git worktree 建的副本」，子 agent 自己建不了：「不做 git 写操作」一条点名了 `worktree`（`.claude/agent-common.md:53`）。该支只在主 agent 把子 agent 派进它建的 worktree（例如 m5-dev）时成立；子 agent 平常的副本是 rsync 的，落到「写进要主 agent 跑的命令」那一支。结论不翻（rsync 一支已兜住），但共用约束 `.claude/agent-common.md:97` 写的是「或 git worktree 建的副本（写副本根）上跑两条」，读者会以为子 agent 可以自己建。
- 推翻条件：子 agent 在 rsync 副本上跑那两条也判得到（那就要在副本里带 `.git` 与词表，判决的取舍可放宽）。

### K3-5、K3-8、K3-9（采，后两条弱）

- K3-5：判决原文：「登记位加第四处」（`research/prompts/landing-audit-r1-main-verification.md:57`）。`CLAUDE.md:22` 与 `.claude/main-agent.md:21` 现在一致（后者「子 agent 与协作工具的事记进 `records/2026-09-16-subagent拆分提案.md` 的编号小节」）。站得住。
- K3-8：执行员 3c 已有「待主 agent 证红」（`.claude/agents/experiment-runner.md:40`）；K3-9：回扫员与派发表都写「进下一轮的清单」（`.claude/agents/sweep.md:52`）。攻方自己标「没找到一次真撞上的派发」，成本是两句文字，站得住。

### K3-6：比对登记归执行员（采）

- 判决原文：「登记行归执行员（它实现单元）」（`research/prompts/landing-audit-r1-main-verification.md:58`）。
- 结论：站得住。登记表第 5 列（留存产物名）与第 6 列（复跑驱动名）只有实现单元、登记复跑的人才知道，设计员写登记时这两样还不存在，所以归设计员是没选的改法，同一条理由放到它上面不成立。执行员定义（`.claude/agents/experiment-runner.md:44`、`:52`）、归属表（`.claude/gate.d/stage-owners.tsv:106`）、写范围表（`.claude/hooks/agent-write-scope.tsv:20` 的说明列）三处现在指向一致。
- 6 个单元的核对见上面「正推 10」。

### K3-7：设计员 3a 的「预期的差」与「答案不许已经存在」（采）——够不着

- 判决原文：「② 改成每个判据要分辨的差取岔路单那一行的翻面观测」（`research/prompts/landing-audit-r1-main-verification.md:59`）。依据是攻方 K3-7 的读法：规则「别处的定论、自己的直觉，一概不看。」（`.claude/singlefs-ai-sop/rules/test-discipline.md:27`）包含现算。攻方在自己的推翻条件里写明这是它的读法，判决没去核。
- 够不着一：设计员定义自己就给「自己算出了答案」留了登记位：「判问法时自己算出了答案，列进登记固定的一节」（`.claude/agents/experiment-designer.md:31`）。项目自己的读法是「可以算，算出来必须列出来」，不是「不许算」。原 3a 的冲突，一半可以靠同样的办法解掉，判决对 ③ 就是这样做的（结论列进那一节）；对 ② 却整句换掉。真正与规则冲突的是 ② 末尾那一句「分辨率分不出的，换测量手段或把判据改成分辨率分得出的形式」——它是在看到算出的预期之后改判据，而规则写的是「判据、阈值、作废条款都在跑之前写死，写完再去看已有结论。」（`.claude/singlefs-ai-sop/rules/test-discipline.md:33`）。判决没选的改法：保留预期的差、算出的数列进第 2 步那一节、分辨率分不出时交回主 agent 定判据，不由设计员当场改。
- 够不着二：改写丢了缺口的一类。缺口 K3 的 9 次续派里点名的失败形态：「E171#13 的 Q5 末段 1 207 959 552 态，E185#4 的 CPU 时钟分辨率 1 ms，E185#8 的对照判据按相对变化判，E165#31 的 r3 档没有合法一族的正式跑」（`research/prompts/subagent-landing-audit-2026-10-22/rework.md:12`）。新 3a 的 ① 罩态数类（E171#13），新 ② 罩分辨率类（E185#4）。E185#8 的原因是「CPU 臂对照按相对变化判，内存格基线中位为负，分不开 V1」（`research/prompts/subagent-landing-audit-2026-10-22/rework.md:332`）：分辨率没问题，是判据形式在这个臂的基线上退化，要有预期值或基线才发现，新 ② 的文字「写测量手段的分辨率…分不分得出它，不写被测臂上预期产生多大的差」（`.claude/agents/experiment-designer.md:33`）明文不让写预期。这一次改动对 4 个点名样本关住 2 到 3 个，不是缺口的全部。
- 站得住的部分：判决发现的冲突是真的（先算出答案再按它改判据确与规则相抵）；③ 的处理是对的。
- 推翻条件：规则作者明说「自己现算的预期也不许在设计阶段看」；或在新 3a 文字下造一份 E185#8 型登记（基线中位为负、判据按相对变化），设计员照新 ② 能发现它。

### K4-1、K4-2、K4-3、K4-4（采）

- K4-1 判决原文：「采：`outcome` 加 `--until`」（`research/prompts/landing-audit-r1-main-verification.md:67`）。我按调度记录写的命令重跑闭窗口结局账：`python3 scripts/agents/agent-cost.py outcome --since 2026-10-01 --until 2026-10-21 --summary`（经内存包装，退 0，约 2 分钟含排队），输出 sha256 `6dde89a84129fa1b3b22da055372594d71bb11b262df5f2979c20db5a92e6ab0`，与存档 `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-2026-10-01-to-2026-10-21.txt` 逐字节相同（`diff` 无输出）。站得住。
- 说过头的一句：调度记录写「往后复跑得出同一组数」（`records/2026-10-22-子agent结果落地审计.md:178`）。`--until` 只关了三个漂移源里的两个（无上界、还在跑按此刻算）。落地仍按现在的 git 历史算：「f"--since={begin}", "--branches", "HEAD"]」（`scripts/agents/agent-cost.py:907`）没有上界，将来的提交会把闭窗口里「写了」桶往上推。今天相同，是因为存档之后仓里没有新提交（`git log --since` 取存档写入之后，输出为空），所以这次逐字节相同证明不了将来提交之后还相同。能复跑得出同一组数的是存档那一份文件，不是命令。
- K4-2：同上「正推 13」。
- K4-3：判决要求把「另多 3 次」改成「4/4 复现；盲判在 E188 另归出 3 次，其中两次 rework.md 归给另一条缺口」。`records/2026-10-22-子agent结果落地审计.md:93` 已写成「盲判在 E188 另归出 3 次，其中 E188 #6、#7 rework.md 归给另一条缺口」，`:101` 写「盲判独立复现 4/4」。站得住。
- K4-4：判决要求「采为限度：输入随草稿目录删了」（`research/prompts/landing-audit-r1-main-verification.md:70`）。调度记录 `:170` 已写限度。站得住。

## 二、每一条「不采」，替被否掉的一方辩护

### 正推 8：⑰ 判名字里任意位置带 report，比实测被拒的宽（不采）

- 判决原文：「不采：宽是向安全的一边，名字里带 report 的别的文件 16 次里拒 6 次」（`research/prompts/landing-audit-r1-main-verification.md:35`）。
- 被否掉的一方（正推腿）的论点：被拒的集合是以 report 起头的名字，⑰ 宽到任意位置，会误拦写得进去的名字；共用约束写的「或名字里带 report 的文件，会被工具层当场拒」（`.claude/agent-common.md:108`）无数据。
- 我的数据（`tools/scan_report_names.py`，全部子 agent 会话记录里 Write 的目标名带 report、summary、findings、analysis 的，结果里有没有「Subagents should return findings」；输出 `out/names-all-summary.txt`）：
  - 被这句拒的：`report.md` 211 次，加 `report-part3.md`、`report-body.md`、`report-8.md`、`report-5.md`、`report-3.md`、`report-2.md` 各 1 次。7 个不同名字全以 report 起头。
  - 名字中间带 report 的 .md：`implementation-writer-c507-report.md`（2026-09-23）、`apply-report.md`（2026-10-11）写成功；`e142-r15-runner-report.md`（2026-09-25）没写成，但报错是项目自己的写范围闸（「experiment-runner 的写范围不含…」），不是这一句。三份都发生在 `report.md` 已被拒（最早约 2026-09-21，按会话记录文件的修改日期）之后。
  - 名字带 report 的非 .md（.py、.rs、.sh）13 次，全部写成功；summary、findings、analysis 起头的 .md 零次，只有 `10-summary.md`（不是前缀）与 `summary.sh` 写成功。
- 结论：不采站得住，但判决写的理由不对。「16 次里拒 6 次」的分母混进了 .py、.rs，⑰ 只看报告行里的文件名，这些与它无关；.md 名字的数据是「起头的 7 个全拒、中间带 report 的 3 个无一被这句拒」。所以「宽是向安全的一边」在数据上是空话：放宽换不来安全，只多拦了 3 个历史上写得进去的名字。不采仍对，因为代价只是改成 `result.md` 重派一次，而 SUMMARY、FINDINGS、ANALYSIS 前缀没有数据，窄了反而可能漏。
- 判决没处理的两半：`.claude/agent-common.md:108` 的后半句有三个反例（以 `report` 起头才拒，不是「名字里带」）；正推腿报告 #8 还点了 `.claude/hooks/handback-guard.sh:202` 的拒绝说明没写 result.md：「"→ 怎么办：全文写进报告文件（派发提示给的报告路径；没给就写进草稿目录）」，判决表里没有这半条的去向。
- 推翻条件：找到工具层规则的文本，写的是「名字里带」；或出现一次 `x-report.md` 这类名字被「Subagents should return findings」拒。

### K2-7：命名自查放行与广谱变异放行并成一张表（不采）

- 判决原文：「不采：攻方自己判值不大」（`research/prompts/landing-audit-r1-main-verification.md:47`）。
- 被否掉的一方：两处同形，并成「阶段 → 只读格集合」一个分支。判决的理由是攻方自己的评价，不是对改法的分析，我补一个实的：
  - 攻方说「一个是等于、一个是子集，并了要带上比法」，其实两处比法在非空集合上等价——「and checked_cells(arguments) == {BROAD_MUTATION_READ_ONLY_CELL}):」（`.claude/hooks/lib_heavy_tests.py:1603`）的右边只有一个元素，非空子集只能等于它；命名自查那一支先判了 `naming_cells` 非空：「naming_cells <= NAMING_SELF_CHECK_CELLS」（`.claude/hooks/lib_heavy_tests.py:1607`）。比法不是不并的理由。
  - 真正拦着的是第三支：同一段里 research-build 那一支（`lib_heavy_tests.py` 里 `checked_cells(arguments) == {RESEARCH_UNIT_TESTS_CELL}`）命中后落到 `kind = None` 继续往下判，另外两支直接 `return None`。三支结局不同，并成一张表就得再加一列「命中之后怎么走」，每支还各带自己的弄坏开关（`broad-mutation-read-only-counted`、`naming-self-check-counted`、`naming-self-check-any-cell`）要逐个保住自证红。收益是少两行，风险是动一个所有子 agent 命令都过的闸。
- 结论：站得住（理由换成上面这个）。推翻条件：这个闸还要再加第四、五个「只读格」放行，那时再并。

### K2-8：`git worktree list` 解析三份；占号目录表与孤儿格目录表不一致（不采，后者进九十节 ⑨）

- 判决原文：同一行「不采：攻方自己判值不大」（`research/prompts/landing-audit-r1-main-verification.md:47`）。
- 解析三份：现在 `git worktree list --porcelain` 的解析在 `scripts/kb/claim-experiment.sh:35`、`scripts/agents/agent-cost.py:895`、`scripts/hosts/clean-session-scratch.py:141`、`scripts/gate/staged-tree-worktree.sh` 两处，共五处，各两三行，三处 bash、两处 python；抽库要跨语言，仍不划算。站得住。
- 目录表不一致（攻方的推：占号不看 `crates/singlefs-checker-tier/src/bin/` 下的入库装置）：我跑 `tools/claim_numbers.sh`：bin 下 16 个号，占号脚本 `used_numbers_in_tree` 扫出的 196 个号，bin 下有而它没扫到的号：空；`claim-experiment.sh --next` 给 E198。今天没有漏。原因是占号建的跑前登记本身先落在 `research/prompts/e<号>-preregistration.md`（`scripts/kb/claim-experiment.sh:51` 起那一组 `ls` 的头一条就是 `research/prompts`），入库装置总在登记之后才有。攻方的场景要「另一棵工作树里只有装置、没有登记」，流程上先后次序不允许。站得住；⑨ 已登（`records/2026-09-16-subagent拆分提案.md:1480`），登记是对的，等流程变了再立。
- 推翻条件：出现一个号只在 bin 下有、登记与实验页都没有，且占号脚本把它又发了一遍。

## 三、判决节「五、没核动的」会不会改变采与不采

- 判决原文：「攻方 K3-4 的最小副本输出主 agent 没复跑」（`research/prompts/landing-audit-r1-main-verification.md:94`）。不改变。依据是两行代码，我在现行文件上复核过（见上面 K3-4）：没有 `.git` 和词表的副本上这两格读不到输入。
- 判决原文：「攻方 K4-1 的复算表主 agent 没复跑」（`research/prompts/landing-audit-r1-main-verification.md:95`）。不改变采。我补跑了闭窗口那一条命令，输出与存档逐字节相同；攻方复算表是只带下界那条命令的读数，采的是「命令只有下界、产物没落盘」，与它无关。
- 判决原文：「没逐个看那 6 个名字」（`research/prompts/landing-audit-r1-main-verification.md:96`）。不改变不采；改变不采的理由写法（见「正推 8」）。6 个名字是 `report-part3.md`、`report-body.md`、`report-8.md`、`report-5.md`、`report-3.md`、`report-2.md`，全以 report 起头，与正推腿的数一致。

## 四、第五批三份交回报告的偏离与遗留：合不合理，判决当初的要求是不是做不到或写错了

门禁那份「没做什么」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md` 末节）、钩子那份「判法变宽的地方」「没做什么」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md`）、定义那份「读数与限度」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md`）。

| 编号 | 偏离或遗留 | 辩方判定 | 依据 |
|---|---|---|---|
| G1 | 四道门禁的样本档没在阶段里跑：「stage-selftest 被闸拒」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:9`），样本须与暂存区相同，这一批没暂存 | 合理（做不到）；遗留要紧 | K2-1 并掉七处的行为不变，最强的证据就是这四道的样本档。目前只有作者草稿里逐段跑、主 agent 对同一棵树的逐行比（后者覆盖不到取不到基准的分支）。暂存后跑 `r1fix-gates.md` 「要主 agent 跑的命令」第 1、2 条 |
| G2 | 第八处同一写法没并：「另找到第八处同一写法，没改」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:75`） | 合理 | 不在派发清单；判决「七处」是攻方 grep 的数，少了一处；⑪ 已登 |
| G3 | 库自证没进 `selftest-registry.tsv`，没挪进 `.claude/gate.d/lib/` | 合理；判决要求写错了一半 | 判决点名路径是 `.claude/gate.d/lib-lag-baseline.py`（`research/prompts/landing-audit-r1-main-verification.md:78`），而登记表把 `.claude/gate.d/` 这一层的键一律当门禁样本档，写 light 会被当成写坏的行（交回报告同节）。判决没想到这条规则。后果：改库的人不会被钩子叫去跑自证，只有 code-tooling 的 research-script-selftests 格按表外一档跑。⑫ 已登 |
| G4 | `.claude/batch-scope` 没登记这一批的文件：「.claude/batch-scope 没登记这一批的文件」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md:271`），理由是不在写范围 | 合理；判决没安排谁登记，遗留会让提交前红 | 门禁 batch-scope 格：「② 每个触发文件都要在 .claude/batch-scope 里有一行；少一个 ⇒ 红。」（`.claude/gate.d/doc-process-records.sh:43`）。`python3 research/prompts/landing-audit-r2-sonnet-model/tools/batch_scope_gap.py . research/prompts/landing-audit-r2-snapshot/judged-sha256.txt` 的输出（`out/batch-scope-gap.txt`）：被判文件里对 HEAD 有改动的 60 份，命中触发清单 25 份，已在 batch-scope 6 份、不在 19 份；另外第五批改的 `.claude/agents/implementation-writer.md` 不在被判清单里，`grep -c implementation-writer.md .claude/batch-scope` 为 0。这是我读那一格文件头得来的判法，没跑那一格 |
| G5 | 6 个单元不登记（交回报告 K3-6 一节） | 合理 | 见「正推 10」，对着 r5 登记逐个核过 |
| H1 | ④ 现在也判裸「报告：」行 | 合理（判决 K2-4 的本意） | 三方三种类型在写范围表里没有行（`.claude/hooks/agent-write-scope.tsv` 里 `three-way` 零命中），④ 不管它们 |
| H2 | ⑰ 四种标签、可在句中、行里任何带 report 的词都拒（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:210`） | 合理，披露了；有一个误拦实例 | 否定句「（不要叫 report.md）」被拒，见 K2-4。判决要的是路径，不是行里每个词 |
| H3 | 三方腿没有报告行或不是 output.md 形态拒 | 合理（判决 K3-1 本意）；实现是常量不是 frontmatter | 见 K3-1：判决的文件划分让 frontmatter 一支没人写 |
| H4 | 看门狗认交回从「success:true」放宽到「不是错误」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:212`） | 合理，数据核过 | 3266 条真实交回结果逐条同判，见 K2-6 |
| H5 | usage、timing、dispatch 三个子命令收子 agent 从「修改时刻」改「首条记录」 | 合理（攻方 K2-6 的提议，判决采） | 口径变了：之前的数（调度记录里的 2268 个、7623.33）按修改时刻取，不能与今后的 usage 直接比；调度记录 `:178` 已注「不能原样复跑」 |
| H6 | 看门狗的「起点」没改成调库（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:239`） | 合理，小偏离 | 判决说三处「都从它取」，起点留在看门狗的逐行循环里，与库同义；是同一个口径写两份，将来改起点要改两处 |
| H7 | 看门狗 `AGENT_WATCH_BREAK=finished` 开关没跑完：「两次都超过前台 236 秒」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md:205`） | 合理（前台时限）；遗留 | 命令在交回报告「要主 agent 跑或改的」第 1 条，要后台起 |
| H8 | dispatch-prompt 文件头三条旧开关点名对不上 | 合理（改前就有） | 不在这一批 |
| D1 | 多改了 `implementation-writer.md` | 合理，判决数错（见 K3-3） | 共用约束点名三个类型 |
| D2 | K3-1 定义一侧只补了 `required-inputs` | 合理 | 见 K3-1 |
| D3 | 定义检查串有 2 项事后对齐：「这两项的检查串是事后对齐的，不是先于改动钉的，读数时按这个打折」（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md:46`） | 合理，披露了；对 K3-7 这类语义改动不够 | 检查串只证文字在不在，证不了改后缺口关住（K3-7 就是例子） |

另一处小的：`records/2026-09-16-subagent拆分提案.md:1466` 的九十节标题还写「四条与一条没想清」，表里已有 ① 到 ⑭ 共 14 行（`awk '/^## 九十、/{f=1} f' … | grep -c '^| [①-⑳] |'` 输出 14）。

## 没做什么

- 没替主 agent 撤销或维持判决，只交复核结果。
- 没跑门禁阶段、没编译、没跑 `gate.sh`；没复跑 `doc-experiments.sh` 等格，K2-1、K2-3 的「行为不变」只采信交回与主 agent 的记录。
- 跑过的轻型：`python3 scripts/kb/stale-candidates.py --selftest`、`python3 scripts/agents/subagent_transcript.py --selftest`、`python3 scripts/agents/dispatch-prompt.py --selftest`、`bash .claude/hooks/continuation-guard.sh --selftest` 都退 0；闭窗口结局账一条（经内存包装，第一次排队等到上限退 252，输出作废，隔一会儿重跑退 0 才是上面引的）。
- 没核 Claude Code 工具层对 `summary.md`、`findings.md`、`analysis.md` 起头的 .md 的行为：历史上没有一次写它们，数据空，不是「不拒」。
- 没判攻方腿的 K2、K3、K4 三格（第五批修补本身、产生结论的方法、没被看过的地方）；「正推 13」「K4-2」里的词表数、K2-4 里的误拦实例、K2-1 的行为不变，只记了我碰到的，没往下攻。
- 返工词表的数按子 agent 描述个数算，不是按花费加权；窗口按会话记录文件修改日期取，所以数与调度记录的 2287、354 差几个。
- 没改仓里被判的任何文件；写了的只有本报告、模型目录 `research/prompts/landing-audit-r2-sonnet-model/` 与草稿目录。
- 副本与编译目录：没建。草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r2-sonnet/` 留着 `tools/`、`out/`、`probe/`（派发闸探针提示）、`rerun1/`，留给主 agent 核，没有仓副本。

## 要主 agent 跑的命令

- 暂存这一批之后：`r1fix-gates.md` 「要主 agent 跑的命令」第 1、2、3 条（四道门禁样本档、库的弄坏开关、真仓单跑五格），和 `r1fix-hooks.md` 的 `AGENT_WATCH_BREAK=finished` 那一条（后台起）。这是 G1、H7。
- 提交前 batch-scope 格会红的 19 份加 `implementation-writer.md`，命令见 G4。

## 补：模型目录里后加的两个文件的 sha256

| 文件 | sha256 |
|---|---|
| tools/batch_scope_gap.py（G4 用，不在 rerun.sh 里，单独跑：`python3 research/prompts/landing-audit-r2-sonnet-model/tools/batch_scope_gap.py . research/prompts/landing-audit-r2-snapshot/judged-sha256.txt`） | 3231d6b6a85dc447aa992b375e4a0fb6b7b34ca1eb5b3b617a50b981f13350f6 |
| out/batch-scope-gap.txt | dce8dadbe8fce83e39496dbb49f79c102a4520201dae1573ca063be435aabc19 |
