# agentdef-r2 本地攻方 逐句核转述

英文项 / 原文文件:行 / 首稿缺的 / 定稿。逐行核对英文提示 `research/prompts/agentdef-r2-local-attack.md` 里每一条转述、每一处事实数字与仓里原文，按 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节办。本轮攻击面：只攻改法一（`.claude/main-agent.md:43` 的替代句）的可执行性。

| 英文项（提示文件里的位置） | 原文文件:行 | 首稿缺的 / 首稿错的 | 定稿 |
|---|---|---|---|
| BACKGROUND 第一、二句：候选表一行是什么（现状句可能还在说旧事实，配对新事实） | `research/scripts/stale-candidates.py:2`「阶段同步的候选表：这一阶段哪些事实变了，仓里还有哪些现状句在说旧的。」 | 无遗漏；「回扫员照表逐行判，不自己挑关键词、不整组放行」半句（同一行下文）未转述——与本轮攻击面（改法一「逐行全看 + 各抽 ≥2 行重判」的算术与机械后果）无关，判定动作本身由下一条「当前规则」承担，不重复译 | `Each row of that table records one specific location in the project's own text where an existing sentence might still be stating an old, now-changed fact from before this project stage, paired with what that fact now is.` |
| 五种判定类别的名称与它们各自的字面 | `research/scripts/stale-candidates.py:69`「LINE_VERDICTS = ('要改', '要补', '事件句不改', '不相干', '要人看')」；`:32`「判定以「要改」「要补」「事件句不改」「不相干」「要人看」之一开头」 | 无遗漏，五个类别字面全部译出；提示里排布顺序（IRR/EVT/CHG/HUM/SUP）与源码元组顺序（要改/要补/事件句不改/不相干/要人看）不同，只是提示内部呈现顺序，不改变类别本身字面，因此不算漏译或错译 | CAT-IRR = 不相干（irrelevant）、CAT-EVT = 事件句不改（event-sentence, no change needed）、CAT-CHG = 要改（needs change）、CAT-HUM = 要人看（needs human review）、CAT-SUP = 要补（needs supplement） |
| CAT- 标签系统本身 | 仓里没有对应原文——本节标注为外加 | 外加：原文没有这套标签，是本稿为满足派发指令「答复不写代码行号与文件行号，按函数名、表格行号指」新造的机械标签，让模型在答案里只用 CAT-IRR 等标签指认类别，不必也不能引用文件路径或行号；手法仿照 `research/prompts/agentdef-r1-local-attack.md` 的 TAG LEGEND | 五个 CAT- 标签，见上一行 |
| 当前规则「逐行全看」那句（BACKGROUND 第三句） | `.claude/main-agent.md:43`「再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。」（该句嵌在「什么时候派哪个 agent」表的一行内） | 无遗漏：三个分句（逐行全看 / 对着载体今天的原文核一遍 / 不抽样不按判定种类挑）写作时已逐一核对原文，「今天」（today）这一限定词没有丢 | `look at every single row of the table in full, checking each row's judgment against the row's own underlying source text as that text reads today, without sampling and without picking rows by judgment category.` |
| 改法一逐字（BACKGROUND 第四句 PROPOSAL ONE） | `research/prompts/_agentdef-r2-background.md:22`「本地那句改成「逐行全看 + 另按判定种类各抽 ≥2 行重判，抽到哪几行写进同步记录的一张表」」 | 无遗漏：三个分句（逐行全看 / 另按判定种类各抽 ≥2 行重判 / 抽到哪几行写进同步记录的一张表）全部译出 | `look at every single row in full; additionally, for each of the five judgment categories, sample at least 2 rows from that category and re-judge only those sampled rows; record, in a table inside the synchronization write-up, exactly which rows were sampled.` |
| 「re-judge only those sampled rows」里的 only | 原文「另按判定种类各抽 ≥2 行重判」没有「只」字，重判对象靠上下文暗示 | 外加：为了不让 Q3（「全看」与「重判」是不是同一件事）这道题在题面里就预先替模型排除歧义，显式加了 only，把「重判」的对象限定为「被抽中的那些行」；不加的话，模型可能把「重判」读成及于全部已经全看过的行，Q3 想问的区别反而在题面自己身上就被抹平了 | 保留 only，定稿见上一行 PROPOSAL ONE 英文 |
| 138 行总数与五类各自行数（FACT TABLE） | `research/prompts/agentdef-r1-opus-output.md:82`「这一轮 138 行的判定分布」；同文件 :84-90 表格「不相干 77 / 事件句不改 44 / 要改 15 / 要人看 2 / 要补 0」 | 无遗漏：五个数字与总数逐一现查抄录，现场核算 77+44+15+2+0=138 与引文一致；另现查候选表原件 `/tmp/claude-1000/owed-sync/candidates.tsv` 今天仍在（未被归档删除），`wc -l` 139 行 = 表头 1 行 + 数据 138 行，与引文的 138 一致 | FACT TABLE 段落五行数字与「77 + 44 + 15 + 2 + 0 = 138」 |
| 第五类判定的名字：本稿用「要补」，不用派发消息的「不是这一阶段的」 | `research/scripts/stale-candidates.py:69,32`；`research/prompts/agentdef-r1-opus-output.md:90`「要补 \| 0 \| 0%」 | 首稿改正：派发消息把第五类叫「不是这一阶段的 0」，但现查源码与云端攻方报告，五种判定的官方字面只有「要改」「要补」「事件句不改」「不相干」「要人看」，没有「不是这一阶段的」这个说法；仓里查不到这个字面出现在任何判定名单里，故未采用派发消息的措辞，改用现查到的「要补」 | CAT-SUP（要补 / needs supplement）— 0 行 |
| Q2-B 里 120+1+1+1+1=124 这个假设分布 | 无原文文件:行可核——这是主 agent 派发文字自己给的极端形态之一（「某一种占 120 行而其余四种各 1 行」），仓里没有真实候选表长这个样子 | 外加：本稿把这个假设分布的算式显式写出（120+1+1+1+1=124），并显式声明这个 124 与 FACT TABLE 的 138 是两回事、不需要对上，避免模型误以为两个总数要相等而卡在算术核对上 | Q2-B 段落原文见提示文件 |

## 没做什么

- 没有转述 `.claude/main-agent.md:43` 那一整行里「交回齐了主 agent 跑 `stale-candidates.py --check-report` 候选表 各份报告，退出码 0 才往下」这个机器闸前置条件，也没有转述该行末尾「判错的那一段重派，重派交回照样全看」——本轮攻击面只问改法一（逐行全看 + 各抽 ≥2 行重判）的算术与机械可执行性，不问触发条件与重派流程，未译不算遗漏。
- 没有转述 `research/scripts/stale-candidates.py` 里「几个词要同时出现就用 && 连」「检索词命中不许超过 150 行」等事实表书写规则——与判定类别分布的算术问题无关。
- 没有把改法一另外修的 B、D、F 三格（`research/prompts/_agentdef-r2-background.md:22`「—— 修 B、D、F」）转述进提示——那是主 agent 与云端两条腿的判定射程，本地腿只攻可执行性，不需要这层归属信息。
