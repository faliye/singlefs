# agent-defs-r2 本地辩方英文提示：逐句核对表（2026-09-17）

依据 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」：英文里的每一句转述写完都要与原文并排核，缺限定词就补。
下表逐项对到源文件自己的行号（不从背景材料 `_agent-defs-r2-body.md` 或 `_agent-defs-r2-appendix.md` 里数，除非该行号本身就在正文里）。
定稿即 `research/prompts/agent-defs-r2-local-defense.md` 现在的内容，发给本地模型的就是它。

## 主题一：A1（Bash + replace-once/replace-batch 改成 Edit）对应 F1–F9

| 英文项 | 原文（文件:行） | 首稿缺的 / 需要修的 | 定稿 |
|---|---|---|---|
| F1 写范围闸只看 Write/Edit | `.claude/agent-common.md:14` | 首句「只写派发提示与定义『写范围』一节给的路径」未直接译入（不影响 Q1 的论证核心，只是框架句），其余逐句对应 | 保留现状，未回补首句 |
| F2 有 Write/Edit 的用 Edit/Write，Bash 只许定义点名脚本写 | `.claude/agent-common.md:15` | 无 | 逐句对应，含「这一轮自己新建的文件可以 replace_all」「草稿目录用什么写都行」两处限定 |
| F3 只有 Read/Bash 的仍用 replace-once/replace-batch | `.claude/agent-common.md:18` | 无 | 逐句对应 |
| F4 旧规则：不分工具集，一律用 replace-once/replace-batch | `research/prompts/_agent-defs-r1-appendix.md:1122-1129`（对应旧 `.claude/agent-common.md` 第 12–19 行） | 无 | 合并了旧文两条 bullet（写范围闸描述 + 定点替换规则），保留「Bash 里的写这道闸拦不住，那一半全靠你守」这句限定 |
| F5 replace-once.py 全有全无、写完回读 | `research/scripts/replace-once.py:1-9,17-32` | 无（描述而非逐字引用，标为读源码所得） | 命中 0 次或 >1 次都拒绝且不写一个字节；命中 1 次才写并回读确认 |
| F6 replace-batch.py 两阶段设计与 2026-09-14 54 处事故 | `research/scripts/replace-batch.py:1-12`（docstring） | **首稿有误**：把「没被」→「没有被」的一字之差描述成「a couple of characters」（约两个字符），现查两串长度差恰好 1（`没被` 2 字，`没有被` 3 字，插入了「有」这一个字）；已用 `replace-once.py` 定点改为「differed ... by exactly one inserted character in the middle of a two-character word」 | 已修正，回读确认 |
| F7 plan_in_memory / run 的具体算法 | `research/scripts/replace-batch.py:19-31,54-62` | 无（读源码得出的描述，非引用） | 逐步对应：先在内存里核对全部编辑，任何一处不中就整批失败且未写盘；全部核过才真正写盘，写完逐个回读 |
| F8 kb-scribe 今天的实际做法：dry-run 核对 + Edit 逐条写 | `.claude/agents/kb-scribe.md:25` | 无 | 逐句对应，含「先 --dry-run 核全部恰好命中一次」「实写用 Edit 逐条改」「改完回读」三个环节 |
| F9 中途一条 Edit 失败时没有任何指引 | 缺失事实，核对范围：`.claude/agents/kb-scribe.md:12`（只覆盖 dry-run 之前规格与原文对不上的停法）、`.claude/agent-common.md` 全篇（无相关条款） | 无（负面事实，现查 grep 确认：`grep -n "半套\|回滚\|失败\|中途\|停下\|已经落盘" .claude/agents/kb-scribe.md .claude/agent-common.md` 命中的三行都不覆盖「dry-run 通过之后、单条 Edit 中途失败」这一情形） | 如实写「没有任何指引覆盖这个情形」，并列出已核过的两处来源 |

## 主题二：报告放进交回内容对应 F10–F17

| 英文项 | 原文（文件:行） | 首稿缺的 / 需要修的 | 定稿 |
|---|---|---|---|
| F10 新规则：定义点名的产出文件照写，报告放交回内容，报告路径给主 agent 存档 | `.claude/agent-common.md:44` | 无 | 逐句对应，含「只调一次（第二次会被拒）」与「系统说明写『不要写报告文件』时报告可以不写文件」两处限定 |
| F11 旧规则：报告先写文件、交回内容要带路径、贴不贴全文随意，没写文件交回不算交了 | `research/prompts/_agent-defs-r1-appendix.md:1168` | 无 | 逐句对应，含「没写文件就交回不算交了」这句硬限定 |
| F12 判决表最后一行（报告不写文件那一行） | `research/prompts/agent-defs-r1-main-verification.md:49` | 无 | 五段式（标题/出处/判决/依据/改法）逐段对应原行的五个字段 |
| F13 SubagentHandback 工具自身说明 | 本会话自己的工具定义（不是仓库文件），标注来源为「本 agent 自己的工具描述」 | 无 | 「只调一次」「未经此工具交付的纯文本不会送达」「没有收件人参数」三点逐句对应工具描述原文 |
| F14 实测：交回只收第一份，第二次被拒 | `records/2026-09-16-subagent拆分提案.md:246` | 无 | 逐句对应「转录里只有两次交回调用（第二次被拒：已交过一次）」 |
| F15 长上下文中段记不牢，本项目没测过多大规模开始碍事 | `.claude/singlefs-ai-sop/rules/machine-first.md:29-30` | 无 | 逐句对应；「unconditionally」是对原文语气的合理概括（原文本身没有任何让步/限定词），未添加原文没有的限定 |
| F16 转述会漂移，四轮连错、每步都像忠实 | `.claude/singlefs-ai-sop/rules/evidence-discipline.md:197-199` | 具体四次转述内容（「相同」→「逐格相同」→「略省」→「正好 2 个节点」）未逐字译出，改成概括的「each restatement different from the one before it」——这四个具体措辞本身是中文特定表述，逐字译出对论证没有增量信息（论证只依赖「四轮、连错、当时都没被发现」这几个事实），故未回补 | 保留概括，未回补四个具体措辞 |
| F17 新旧规则下「谁在什么时候做归档抄写」的对比 | 由 F10、F11 推出，非独立引用来源 | 无（标注为推论，不是直接引用） | 明确写「under the new rule … under the old rule …」对照句式，来源即 F10/F11 |

## 主题三：D3（修订只许收严或补臂）对应 F18–F26

| 英文项 | 原文（文件:行） | 首稿缺的 / 需要修的 | 定稿 |
|---|---|---|---|
| F18 新规则：修订只许收严或补臂，不许少报，少报交主 agent | `.claude/agents/experiment-runner.md:12`（当前文件） | 无 | 逐句对应，含「产物跑过之后不改登记，交主 agent」这句边界条件 |
| F19 旧规则：没有「少报交主 agent」那句，其余逐字相同 | `research/prompts/_agent-defs-r1-appendix.md:155` | 无 | 明确指出唯一差异是新增的那一句，其余原样保留 |
| F20 判决表 D3 行 | `research/prompts/agent-defs-r1-main-verification.md:39` | 无 | 五段式逐段对应，含「Opus（归格有保留）」这个限定词 |
| F21 执行员自陈：把依赖两种读法的 A10 挪出报告流，自己判断，写明需要人复核 | `research/prompts/agent-defs-r1-trial-experiment-runner.md:36` | 无 | 逐句对应，含「这个判断合不合理需要看的人再确认一遍」这句自我标注 |
| F22 主 agent 已定的两项决定写进登记第十二节，原判据一字未改 | `research/prompts/agent-defs-r1-trial-experiment-runner.md:9` | 无 | 逐句对应 |
| F23 设计员登记时两种读法都报、明确交主 agent 选；A10 的定义 | `research/prompts/agent-defs-r1-trial-experiment-designer.md:17,22` | 无 | 逐句对应，含「两种都报没有任何挑选效应」这句设计员自己的判断 |
| F24 综合 F21–F23：少报是主 agent 已定决策的逻辑后果，不是执行员新发明的判断 | 由 F21、F22、F23 推出，非独立引用来源 | 无（标注为推论） | 明确写「putting F21 through F23 together」 |
| F25 旧规则的披露要求正是让这次事件被抓到、写进 D3 的机制，全程未中断实验 | 由 F20、F21 推出，非独立引用来源 | 无（标注为推论） | 明确写来源是 F20 引用的那一行判决表 + F21 那句自我标注 |
| F26 Opus 自己的判决带保留，不是无条件采纳 | `research/prompts/agent-defs-r1-main-verification.md:39`（与 F20 同一行，单独摘出这个限定词） | 无（与 F20 重复摘出，用于强化论证；不是新来源） | 逐字对应「Opus（归格有保留）」 |

## 攻方问题（Q1–Q3）核对

Q1、Q2、Q3 的攻方论据分别整行抄自 `research/prompts/agent-defs-r1-main-verification.md` 第 30、49、39 行（见正文第一节「二、被判的对象」表『第一轮判决』一行指到的同一份文件），逐字段（标题/出处/判决/依据/改法）翻译核对，未摘句、未省略判决表任何一栏。

## 未回补的两处（现查后判定不影响论证，如实记录）

1. F1 首句「只写派发提示与定义『写范围』一节给的路径」未直接译入：这是通用框架句，不是 Q1 论证链的一部分（Q1 只依赖闸怎么看 Write/Edit、旧规则怎么写、新规则怎么写、replace-batch.py 的原子性这几点），漏译不影响四条腿可判定性。
2. F16 四次转述的具体措辞（「相同」→「逐格相同」→「略省」→「正好 2 个节点」）未逐字译出：这四个中文短语本身是那次事故的具体记录，对 Q2 的论证只需要「确有四轮、每轮都错、每轮当时都像忠实、没人当场发现」这几个抽象事实，具体措辞不增加可判定性，故未回补。

## 一处首稿错误的修正记录（已在上表 F6 行注明，此处汇总）

F6：首稿把「没被」→「没有被」这一字之差写成「a couple of characters」，现查两串字数差恰好 1（插入了「有」这一个字），已用 `research/scripts/replace-once.py` 在 `research/prompts/agent-defs-r2-local-defense.md` 里定点改为「differed ... by exactly one inserted character in the middle of a two-character word」，回读确认。
