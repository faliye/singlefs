# agentdef-r3 本地攻方 逐句核转述

英文项 / 原文文件:行 / 首稿缺的 / 定稿。逐行核对英文提示 `research/prompts/agentdef-r3-local-attack.md` 里每一条转述、每一处事实数字与仓里原文，按 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节办。本轮攻击面：只攻 H 格（候选表漏召回）。

| 英文项（提示文件里的位置） | 原文文件:行 | 首稿缺的 / 首稿错的 | 定稿 |
|---|---|---|---|
| FACT-1 第一句：管不到的（三方 sweep-rel-r1 第一轮攻方腿打中、机械闸补不上） | `research/scripts/stale-candidates.py:35` | 无遗漏；「攻方腿」译作 leg（不译 side），与仓内三方术语「本地攻方腿」一致；「补不上」译作 cannot cover it，不译 cannot close this gap（避免添入原文没有的 gap 一词） | `What this tool cannot handle: caught by the attacker leg in round one of the three-way review called sweep-rel-r1; the mechanical gate cannot cover it.` |
| FACT-1 第二句：写表人给一件真变了的事实选一个基准里从没出现过的检索词、再标「新立：」，新事实里又避开落地/实现这类字，过得了闸 | `research/scripts/stale-candidates.py:36` | 无遗漏，四个分句（真变了的事实/选基准零命中的检索词/标新立/新事实避开落地实现类字）与「过得了闸」全部译出；「再标」原文省略主语，英文补了 that row 作显式主语，加了不改原意，单列说明 | `The person who writes the fact table, for a fact that has genuinely changed, picks a search term that never appears anywhere in the baseline version, and labels that row with the prefix "Newly established:", and, in that row's new-fact text, also avoids words of the kind "landed" or "implemented"; this passes the gate.` |
| FACT-1 第三句：这件事实的旧说法一行都进不了候选 | `research/scripts/stale-candidates.py:37` 前半 | 无遗漏 | `Not a single row stating this fact's old wording ever enters the candidate table.` |
| FACT-1 第四句：判别子是写表人自己选的词，闸看不到；靠主 agent 读事实表 | `research/scripts/stale-candidates.py:37` 后半 | 无遗漏 | `The discriminant is a word the fact-table writer chose personally; the gate cannot see it. It relies on the main agent reading the fact table.` |
| FACT-2：候选表 138 行、21 组；组即产生该行的事实 ID | 138/21 数字见 `research/prompts/agentdef-r2-opus-output.md:190`「量到的是 138 行候选 / 21 组」；「组=事实 ID」现读 `research/scripts/stale-candidates.py:267`（docstring「[(组号,…)]」）与 `:277`（`candidates.append((fact['编号'], …))`） | 138/21 数字无遗漏；「组即事实 ID」半句是本稿现读源码得出的机械说明，:190 原文本身没有解释「组」的定义，外加但已现查坐实 | 见提示文件 FACT-2 全文 |
| FACT-3：那一轮的事实表本身 21 行 | `research/prompts/agentdef-r2-opus-output.md:301`「主 agent 读事实表」（21 行）」 | 无遗漏 | `The fact table that produced the candidate table in FACT-2 had 21 rows.` |
| FACT-4：五种判定字面 | `research/scripts/stale-candidates.py:69`「LINE_VERDICTS = ('要改', '要补', '事件句不改', '不相干', '要人看')」；`:32` 判定字面同名单 | 无遗漏，五个类别字面全部译出；CAT- 标签是外加的机械标签（仿照 `research/prompts/agentdef-r2-local-attack.md` 已用过的同一套手法），让答复不必引代码行号 | `CAT-IRR (irrelevant), CAT-EVT (event sentence, no change needed), CAT-CHG (needs change), CAT-HUM (needs human review), CAT-SUP (needs supplement)` |
| FACT-5：事实表五列 | `research/scripts/stale-candidates.py:18-19`「事实表（制表符分隔，首行是表头）：编号 旧事实 新事实 检索词 出处」；SOURCE 一句译自 `:24`「出处写这条事实来自哪几条变更记录（H 编号，--changes 给的）或哪个提交」 | 无遗漏，五列字面与出处列的说明全部译出 | `ID, OLD-FACT, NEW-FACT, SEARCH-TERM, SOURCE.` |
| FACT-6：候选表五列 | `research/scripts/stale-candidates.py:282`（`write_candidates` 表头 `组\t旧事实\t新事实\t载体\t原文`） | 无遗漏五列字面；CARRIER/ORIGINAL-TEXT 的补充说明（文件路径与行号 / 该位置原文）是本稿现读 `:277`（`f'{path}:{line_number}'` 与 `line.strip()`）得出，源码表头本身没有这句解释，外加但已现查坐实 | `GROUP, OLD-FACT, NEW-FACT, CARRIER, ORIGINAL-TEXT.` |
| FACT-7：检索词写法规则 | `research/scripts/stale-candidates.py:21-22` | 无遗漏，「概念名词」「新旧两种说法都会提到」「不写状态词」「不照抄新说法原句」「宁宽勿窄」与三个例词（层0/录制流/多次挂载）全部译出；`the tool's documentation gives an example:` 是外加的引导语，原文括注本身没有这句框架性说明，加了是让读者知道例子出自文档原文而非本稿编造 | 见提示文件 FACT-7 全文 |
| FACT-8：「新立：」前缀的定义与「事实变了」的区分 | `research/scripts/stale-candidates.py:26-27` | 无遗漏，两句全部分句（仓里一个字都没提过 / 新立的欠账新门禁阶段 / 也不出候选 / 实现落地实验跑完欠账还清不是新立 / 旧事实写没有还欠 / 检索词写概念名词）全部译出 | 见提示文件 FACT-8 全文 |
| FACT-9：候选表构建时的机械跳过规则 | `research/scripts/stale-candidates.py:271-272` | 这是现读代码得出的机械说明，不是翻译现成中文散文——核对方式是核代码逻辑：271 行条件判断两个前缀常量 `REWORDING_ONLY_OLD_FACT_PREFIX`（字面 `只改措辞`，无冒号，见 `:74`）与 `NEWLY_ADDED_OLD_FACT_PREFIX`（字面 `新立：`，带冒号，见 `:72`），272 行 `continue` 跳过；英文译文里 "Wording only" 特意不加冒号（与常量字面一致），"Newly established:" 带冒号（与常量字面一致），这处标点差异已现查坐实非笔误 | 见提示文件 FACT-9 全文 |
| FACT-10：`main-agent.md:43` 该行整个单元格 | `.claude/main-agent.md:43`（表格行「一个阶段任务结束…」对应的派发单元格全文，从「`sweep` 写事实表」到「重派交回照样全看」） | 无遗漏，整个单元格全文译出，不摘句；箭头结构保留；「退出码 0 才往下」「不抽样、不按判定种类挑」两处限定词均已译出 | 见提示文件 FACT-10 全文 |
| TASK 段落：不用 markdown 强调 | `.claude/rules/three-way-inference.md:202`「⚠️ 提示里不许用 markdown 强调。」 | 无遗漏，规则本身照办，不是转述一句被引用的散文，是本稿遵守的写作纪律 | 见提示文件 TASK 段落「Do not use any markdown emphasis…」 |
| TASK 段落：不写代码行号与文件行号 | 派发提示原文「提示里明令答复不写代码行号与文件行号（按函数名、表格行号指）」 | 无遗漏；因本题不涉及函数名，改用 FACT-# 标签与表格列名两种指代方式，均已在 TASK 段落写清 | 见提示文件 TASK 段落「Never cite, invent, or guess at a code line number…」 |
| TASK 段落：不许只答 yes/no，按表格逐格填 | `.claude/rules/three-way-inference.md:36`「本地腿只问能落成数、能逐格判的题…要模型按表格逐格填，不许只答 yes / no。」 | 无遗漏 | 见提示文件 Q2 表格设计（LEAVES-TRACE/NO-TRACE 必须配一句说明，不许裸答） |
| 每条答复都要写「什么现象会推翻它」 | 派发提示原文「每条答复都要写『什么现象会推翻它』」 | 无遗漏，Q1/Q2/Q3 各自要求一句显式标注 "Qn overturn condition:" 的推翻条件 | 见提示文件 Q1/Q2/Q3 末尾各一句 |

## 没做什么

- 没有转述 `research/scripts/stale-candidates.py` 里「几个词要同时出现就用 && 连」「一行事实的检索词在结束那一版命中不许超过 150 行」等事实表书写规则（docstring 第 23 行）——与 H 格（候选表漏召回的最少动作数、留不留痕迹、判据）无关，只改窄检索词命中范围，不涉及「新立：」伪装机制。
- 没有转述 `research/scripts/stale-candidates.py` 里 `check_facts` 函数对「新立：」行的具体机械判定逻辑（第 314-318 行，判 `base_hits` 与 `NOT_NEWLY_ADDED_WORDS` 正则）——那是「机械闸补不上」这句话本身在源码里的实现细节，FACT-1 已经明说闸补不上、判别子闸看不到，若把闸的判定逻辑喂给模型，等于替它把 Q1（最少要做几件事）与 Q3（判据）的推理做了一半；只给 FACT-1 的自然语言转述与检索词书写规则（FACT-7），不给这段代码。
- 没有转述 `agentdef-r2-opus-output.md` 里 F3 组 19 行、三种判定共存那一段（:190 后半）——那是候选表内部同一组多种判定共存的问题，不是候选表漏召回，与 H 格无关。
