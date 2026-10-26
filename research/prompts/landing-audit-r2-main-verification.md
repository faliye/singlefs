# landing-audit-r2 判决：子 agent 落地审计这一批改动第二轮（主 agent，2026-10-23）

<!-- doc-lint:not-numbers K1 K2 K3 K4 -->

正文 `research/prompts/_landing-audit-r2-body.md`；辩方腿 `research/prompts/landing-audit-r2-sonnet-output.md`（sha256 10c9501c…），模型 `research/prompts/landing-audit-r2-sonnet-model/`；攻方腿 `research/prompts/landing-audit-r2-opus-output.md`（sha256 aeeee5ee…），模型 `research/prompts/landing-audit-r2-opus-model/`。前一轮判决 `research/prompts/landing-audit-r1-main-verification.md`。正文标题写的 2026-10-22 是起草那天，正文与两条腿的报告落盘时东京日期已是 2026-10-23；正文是冻结证据，不回改。

## 这一轮被判过的文件

- `.claude/agent-common.md`：第五批 K3-2、K3-3、K3-4、K3-5 改的几句；辩方点出第 108 行「名字里带 report 的文件，会被工具层当场拒」有三个反例。
- `.claude/main-agent.md`：第五批 K3-2、K3-5、K3-9 改的几句。
- `.claude/agents/experiment-runner.md`：第 5 步登记比对对象行、「命名检查除外」。
- `.claude/agents/experiment-designer.md`：第 3a 步 ② 改成翻面观测、3c 加「待主 agent 证红」；辩方判 ② 够不着 E185#8 一类。
- `.claude/agents/sweep.md`：「交实现员」一节。
- `.claude/agents/tooling-writer.md`：第 7 步「命名检查除外」。
- `.claude/agents/implementation-writer.md`：第 4 步「共用约束「门禁」一节的命名检查除外」（第五批改了、正文清单漏列，主 agent 自己核，见「一、核查」）。
- `.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`：`required-inputs` 补「报告」。
- `.claude/hooks/runner-dispatch-guard.sh`：④ 与 ⑰ 共用的报告行解析、三方腿报告形态判法；攻方打中报告行不锚行首。
- `.claude/hooks/continuation-guard.sh`：改从 `scripts/agents/subagent_transcript.py` 读交回。
- `.claude/hooks/lib_long_run.py`：`exempt-check` 改收 `--base`、改调 `lib-lag-baseline.py`。
- `.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/selftest-registry.tsv`：执行员那一行说明列、新库一行。
- `scripts/agents/subagent_transcript.py`：「交回」「起点」两个定义；攻方在 3286 份真会话记录上核新旧判法逐个相同。
- `scripts/agents/agent-cost.py`：`outcome --until`、词表、分桶门槛、三方腿按类型名记有效；攻方打中窗口不按东京日期切天、门槛无出处、三方桶不核报告。
- `scripts/agents/agent-watch.py`、`scripts/agents/dispatch-prompt.py`、`scripts/agents/README.md`：改从新库读、三方骨架报告行。
- `scripts/kb/stale-candidates.py`：读 `.claude/kb-directories`；攻方打中 `./<目录>` 与绝对路径静默不认。
- `.claude/gate.d/lib-lag-baseline.py`：七处并成的库；攻方打中四处注释行认法变了。
- `.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/doc-process-records.sh`、`.claude/gate.d/code-source-discipline.sh`、`.claude/gate.d/code-tooling.sh`：改调库；replay-comparison-units 格「只缩不涨」的键，攻方打中借键。
- `.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/`、`.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/`：登记表文件头 6 个单元不登记的理由、归属行、样本。
- `.claude/kb/milestone/03-third-txn.md`（与 `.en.md`、`.ja.md`）第 185 行；`records/2026-10-22-子agent结果落地审计.md`；`records/2026-09-16-subagent拆分提案.md` 第九十节。

## 一、核查

- 快照：`sha256sum -c research/prompts/landing-audit-r2-snapshot/judged-sha256.txt` 63 份全对上，腿跑着的时候被判文件没动；所以下一条 `--root .` 判的就是快照时的样子（第一轮快照对不上七份、用活仓判，攻方 K3-6 点的是那一次）。
- `python3 scripts/agents/cite-check.py research/prompts/landing-audit-r2-sonnet-output.md research/prompts/landing-audit-r2-opus-output.md --root . --background research/prompts/_landing-audit-r2-background.md` 末行：「✓ 核了 85 处引文、49 处裸行号，对上 85 处，没判 49 处（其中裸行号没有引文可比 48 处；逐处列在上面）」。下面采的每一条，主 agent 打开那一行现看过（列在各条）。
- 攻方模型拷进草稿、经 `scripts/hosts/run-with-memory-cap.sh 4G` 跑 `tools/rerun.sh --repo <仓根> --frozen <仓根> --work <新目录>`，退 0；`threshold-sweep.txt`、`dispatch.txt`、`lstrip.txt`、`rekey.txt` 与腿的 `out/` 逐字节相同；`handback-shapes.txt` 差两行（「files 3287」对「files 3286」、送达 3075 对 3071），是腿交回之后又有四个子 agent 交回，形状仍是三种。
- 攻方 K2-3 另用主 agent 自己的路子核：从本会话会话记录取第一轮攻方腿与第五批钩子工具员的首条派发提示原样，`python3 scripts/agents/dispatch-prompt.py --check` 两份都退 3，拒的是正文里「」引着的「报告：」与「…/report.md」；两份当时都放行、都干完了活。
- 攻方 K4-1：`.claude/gate.d/doc-decisions.sh:3354` 起的 `base_ledger_text` 现看，`git show` 退非 0 返回 `None`、调用方 `ledger_growth` 拿 `None` 当「基准里没有这张表」不比，三处调用在第 1268、3449、3542 行。
- 攻方 K4-2：`scripts/kb/stale-candidates.py` 的 `carrier_patterns` 拿 `kb_directories` 的原串拼 `^<目录>/.+\.md$` 去配 git 路径，`scripts/kb/quote-kb.py:440` 的 `kb_candidate_roots` 不规整 `./` 与绝对路径；读代码坐实，没另造仓。
- 辩方：正推 10 那条命令（`awk '/^## 九十、/{f=1} f' records/2026-09-16-subagent拆分提案.md | grep -c 'replay-comparison\|比对对象\|管不到'`）主 agent 现跑输出 0；`.claude/gate.d/doc-experiments.sh:2393` 那句「管不到」现看在；`.claude/agents/experiment-designer.md` 第 2 步「判问法时自己算出了答案，列进登记固定的一节「跑之前已经存在的数」」与第 3a 步 ②「不写被测臂上预期产生多大的差；分辨率分不出的，换测量手段或把判据改成分辨率分得出的形式」现看在；`.claude/agent-common.md:108`「或名字里带 report 的文件，会被工具层当场拒」现看在；`scripts/agents/agent-cost.py:905` 起的 `git log` 只有 `--since`。
- `.claude/agents/implementation-writer.md`：第五批只改第 4 步一处（`research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md` 第 13、30 行），改法与执行员、工具员两份同句；主 agent 对照 `.claude/agent-common.md` 节「门禁」点名的三种类型看过，没有新矛盾。两条腿都没读它，这一份算被攻过零轮，第三轮清单要列。正文清单里的 `.claude/agents/three-way-forward.md` 这一批没改（`git diff --stat` 空），不算被判。

## 二、逐格判

### K1（辩方：第一轮判决够不够得着）

辩方 26 条里 24 条判站得住，2 条够不着；采纳与否：

| 条 | 辩方 | 判 |
|---|---|---|
| 正推 10 | 「不登记」判对；但格只读手填表这一盲区只补了一句文字，九十节没有一行给它立检查，与正推 3、正推 7 的处理不一致 | 采：九十节加一行（「三、判决」第三条） |
| K3-7 | 依据是攻方自陈的读法、判决没核；设计员第 2 步本来允许算出答案、列进固定一节；新 ② 明文不让写预期，丢了 E185#8 一类（基线为负时相对判据退化） | 采：② 改成「算出的预期差列进『跑之前已经存在的数』，分辨率或判据形式分不出的交回主 agent 定，不由设计员当场改判据」，交下一批定义 |
| 正推 8（不采） | 不采对，理由错：「16 次里拒 6 次」分母混进了 .py、.rs；.md 里以 report 起头的 7 个名字全拒、中间带的 3 个无一被拒；`.claude/agent-common.md:108` 后半句因此有三个反例；`.claude/hooks/handback-guard.sh:202` 的拒绝说明没写 `result.md` | 采：不采照旧（⑰ 宽着），共用约束那半句改成实测的形态，交回闸拒绝说明补 `result.md`，交下一批 |
| K3-1 | 关住；三方类型表在 `runner-dispatch-guard.sh:1291`、`dispatch-prompt.py:116`、`handback-guard.sh:70` 各抄一份 | 采：进九十节 |
| K3-3 | 判决写「两份」，应是三份；第五批多改 `implementation-writer.md` 是对的 | 采：本判决「一、核查」最后一条补核 |
| K3-4 | 子 agent 自己建不了 worktree，那一支只对主 agent 派进的 worktree 有效 | 不改：定义写的就是「主 agent 建的工作树」，与「不做」一节不矛盾（攻方 K2-5 同判） |
| K4-1 | 闭窗口复跑逐字节相同；「往后复跑得出」说过头 | 采：并进攻方 K3-3 改记录 |
| 正推 13、K4-2 | 只拿掉「修订」9 条，剩下 349 条里 47.6% 只靠单字「修」或 fix | 采：并进攻方 K3-4 |
| 正推 3 | ⑥ 登的是核报告文字，更强的是核子 agent 自己的会话记录 | 采：改九十节 ⑥ 的形态 |
| K2-6 | 3266 条真实交回上新旧判法逐条同判 | 站得住（攻方 K2-4 同） |
| K2-4 | 报告行写「（不要叫 report.md）」被 ⑰ 拒 | 采：并进攻方 K2-3 |
| 其余 | 站得住 | — |

辩方遗留三件：样本档没在阶段里跑过、提交前 batch-scope 格会红、九十节标题还写「四条与一条」——前两件归提交前（主 agent 暂存时登 `.claude/batch-scope`、跑样本档），第三件不改：第一、二轮正文按标题原文引这一节，改了标题冻结证据里的引用就对不上，行数以表为准。

### K2（攻方：第一轮的修补本身）

| 条 | 攻方 | 判 |
|---|---|---|
| K2-1 | 库 `content_rows` 用 `lstrip` 认注释，四处改前只认行首 `#`：缩进的 `#` 行、首列为空第二列 `#` 起头的行，改前判形状错、并后静默跳过；方向更严，今天七张表一行都没有 | 采甲：库文件头写明这四处语义变了；不改判法 |
| K2-2 | 「只缩不涨」的键是（登记出处，留存产物名），换单元名、换实验号借同一对键照绿；改前滞后表同一对键，洞改前就有 | 采甲：键改成（实验号，登记出处，单元名，留存产物名），样本加借键一份红 |
| K2-3 | 报告行不锚行首，正文「」里引着的「报告：」「报告路径：」当报告行；本会话两份合法派发改前退 0、改后退 2 | 采：这一批带进来的回归。改成报告标签只在行首认（列表记号、粗体之后也算），「」『』“”与反引号里的不认；自证「句中「，报告 <路径>」也认」那一格一并删；验收是上面两份派发提示与 14 种骨架都放行、骨架里填 `report.md` 照拒 |
| K2-4 | 交回定义变宽没打中；起点按系统时钟的日期切天（不是东京日期），`--until 2026-10-21` 收进东京 10-22 开工的 45 个、花费 1.66% | 采：`--since`、`--until` 按东京日期切，存档那一份照新切法重出、文件名不变 |
| K2-5 | 没打中 | — |

### K3（攻方：产生结论的方法）

| 条 | 攻方 | 判 |
|---|---|---|
| K3-1 | 70% / 30% / 10 行三个门槛没出处；「没效果」只随 30% 与写入下限走，30%→50% 是 3.4%，(0.9, 0.7) 是 6.2%；「写了、不低于 70%」随上沿从 50.0% 掉到 35.0%（70%→90%） | 采：调度记录「真浪费约 1% 到 2%」加限定（下沿 ≤ 30% 时；下沿 50% 时 3.4%），`agent-cost.py` 文件头写门槛没出处、换值表指到这份判决；「丢失的不多」不翻 |
| K3-2 | 三方腿交回就按类型名记有效（28.6%），不核报告在不在、所在轮有没有判决 | 采：结局账三方腿分两桶（交回点名的 `-output.md` 在仓且同轮有 `-main-verification.md` / 其余），交下一批；调度记录那一行加限定 |
| K3-3 | 「往后复跑得出同一组数」说过头：「点到」没有上界、落地比对读此刻的工作树 | 采：调度记录改成「当天复跑逐行相同；往后点到与工作树会变」 |
| K3-4 | `\bfix` 命中 fixture；单字「修」多半是新活；「补跑」「第二段」漏；14% 不是上界 | 采：调度记录「返工约 14%」改成关键词计数、误报漏报都有；`\bfix` 加尾词界交下一批；不再扩词表（判别子观测不到，攻方引 evidence-discipline 那一形） |
| K3-5 | K4、K3 两条缺口的样本是两条最长链里 8 次、4 次，盲判证归类一致不证普遍；载体搬家是一次事件 | 采：调度记录判定表那两行与载体搬家一行加样本来源的限定 |
| K3-6 | 第一轮 `--root .`；「与改前门禁逐行相同」只走到「基准里有表」一种结局、只走到今天的行形 | 采：调度记录那一句加限定；第二轮这次快照全对上，`--root .` 与快照根同义，写在「一、核查」 |

### K4（攻方：从未被看过的地方）

| 条 | 攻方 | 判 |
|---|---|---|
| K4-1 | `doc-decisions.sh` 的 `base_ledger_text` 是第九处同一写法，把取不到当成没有表，三格调它 | 采：与九十节 ⑪（第八处）一起并进 `lib-lag-baseline.py`，交下一批 |
| K4-2 | `.claude/kb-directories` 写成 `./<目录>` 或绝对路径，`stale-candidates.py` 静默不认 | 采：读法在 `quote-kb.py` 的 `kb_candidate_roots` 规整（去 `./`、拒绝绝对路径与出仓路径并报出来），`doc-registries.sh` 那一份同改或改调它，交下一批 |
| K4-3 | 归档目录搬进 `research/prompts/` 没打中 | — |

## 三、判决

一、下一批（第七批）派工具员，文件不相交分三份同时派：

| 份 | 关的条 | 文件 |
|---|---|---|
| 门禁与扫描 | K2-1 甲、K2-2 甲、K4-1 与九十节 ⑪、K4-2 | `.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/doc-experiments.sh` 与 replay-comparison-units 的样本、`.claude/gate.d/doc-decisions.sh` 与它挂账表格的样本、`scripts/kb/quote-kb.py`、`.claude/gate.d/doc-registries.sh`、`scripts/kb/stale-candidates.py` |
| 钩子与协作脚本 | K2-3、K2-4、K3-2、K3-4 的 `\bfix\b`、正推 8 的交回闸拒绝说明 | `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`、`scripts/agents/agent-cost.py`（含文件头门槛一句）、`scripts/agents/dispatch-prompt.py`（自证若读报告行） |
| 定义文字 | K3-7（辩方）、正推 8 的共用约束半句 | `.claude/agents/experiment-designer.md`、`.claude/agent-common.md` |

二、主 agent 自己改：调度记录 K3-1、K3-2、K3-3、K3-4、K3-5、K3-6 六处限定；九十节 ⑥ 的形态（核会话记录）；⑪ 补「第九处 `doc-decisions.sh` 的 `base_ledger_text`」。

三、九十节加的：正推 10 的盲区（比对对象表散文没有固定形态、格看不见漏登记）；K3-1 的三方类型表三处各抄一份。

四、改完开第三轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核），第三轮被判清单补 `.claude/agents/implementation-writer.md`。上面每一条改法都是腿或主 agent 推的、被攻过零轮。

## 四、交用户

无：这一轮采的都是工具与定义的改法和记录措辞，没有要用户在几条路之间选的。

## 五、没核动的

- 攻方 K2-4 的「45 个、1.66%」是腿在草稿里按 `out/closed-agents.jsonl` 现算的，没进它的模型，主 agent 没复算；采的是「不按东京日期切」这一机制（`scripts/agents/agent-cost.py:1430` 的注释现看在），不依赖那两个数。
- 辩方正推 8 的名字分布（`tools/scan_report_names.py`）主 agent 没复跑；采的是「共用约束那半句有反例」，三个反例名字在辩方报告里逐个点了日期与结局，主 agent 没逐个去会话记录里找。
- 攻方 K3-5 对九条缺口样本来源的读法，主 agent 只现看了它引的 `rework.md:14` 与调度记录第 100、101、106 行，没重做抽样。

## 回看决策

不涉及决策：被判的都是 agent 定义、共用约束、钩子、门禁与协作脚本的工作流写法，加上里程碑三「待优化」表一句现状与两份调度记录；不碰 `.claude/kb/decisions/` 的任何分项。
