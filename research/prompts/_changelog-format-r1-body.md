# kb 历史怎么组织：changelog-format-r1（2026-09-28）

**实现今天的样子**（`.claude/rules/implementation-first.md` 第 3 条）：这一轮改的是 kb 治理工具与文档
组织形态（`.claude/rules/`、`.claude/agents/`、`.claude/gate.d/`、`research/scripts/`、
`.claude/singlefs-ai-sop/scripts/`、`.claude/kb/`），不碰文件系统设计或实现。`crates/` 里现查
`grep -rn "changelog\|decisions-history\|history-ordinal" crates/` 零命中——这一轮与 `crates/` 无关。

本地腿：攻 —— 这一轮改的是治理机制本身（格式约定 + 门禁判据 + 一次性迁移脚本），风险集中在
「约定写得对不对」「门禁真的挡得住违规吗」「迁移脚本会不会丢内容」三处，比起需要辩护现有判决，
更需要一条腿专门找茬；没有前一轮判决要复核，辩方这一轮没有对象。

## 改了什么

`decisions-history.md`（决策变更史）与 `experiments-history.md`（实验变更史）过去是「按月存原文、
`decisions-history.md` 按决策汇总只列最近 3 次、更早的去按月文件里翻」的两级结构；每天改动一多，
`decisions-history/<年-月>.md` 里同一天要开好几个 `### 日期（其N）` 标题，日期字符串重复几十遍。
用户原话：「文档中的历史和改动等，按照条目组织，每个条目下每天的日期仅仅出现一次。现在用户一天要
说几百句话，日期就要写几百行。」

改成：一条决策一个 `## D<n>（简称）` 节，节内是这条决策的完整历史（不再只列最近 3 次）；节内按日期
分组，同一天的改动共享一个 `### 日期` 标题，各自占一个 `#### ` 子标题；按月的原文文件
（`decisions-history/2026-08.md`、`2026-09.md`）整个撤销，内容原样搬进对应决策的节里。
`experiments-history.md` 同一套规则，按 `## E<n>（简称）` 分节。

具体改动：

1. 新规则 `.claude/rules/changelog-format.md`：定这套组织形态（日期分组、编号判据、并发取号）。
2. `.claude/gate.d/30-decision-history-entries.sh` 三格重写：`entry-added`（决策正文改了要有新增条目，
   不再依赖按月文件）、`shape`（新——日期与编号必须分属 `### ` / `#### ` 两层，两份变更史的日期块必须
   挂在对的 `## D<n>` / `## E<n>` 节下，同一节里日期不许重复；只判这一轮 diff 新增的行，存量只计数），
   `status-sync`（新——两份变更史每节顶上的 `**现状**：` 行与 `decisions.md` / `experiments.md` 索引表
   同步，`--write` 只重写这一行）。旧的 `month-file`、`brief-sync`（连同 `<!-- gen:history-brief:start
   -->` 生成块与「只列最近 3 次」的截断）整套撤销。
3. 上游共享脚本 `.claude/singlefs-ai-sop/scripts/history-ordinal.sh`（撞号检测：并发会话各取号，
   取到同一个「（其 N）」判红）泛化：旧版只认 `### 日期（其N）` 融合在一行的形态，键就是这个子串；
   新版键改成「最近的 `## ` 标题 + 最近的 `### 日期` 标题 + 子标题里的点名词（已定项 N / 未定项 N /
   `E<n>（`，没有就用去掉编号的整句）+ 编号」，同时兼容旧融合形态（尚未搬迁的文件）。新增键提取器
   `history-ordinal-keys.py`。基准比对从「按文件名对文件名」改成「基准侧整个 `.claude/kb` 树的键池」，
   否则文件改名/合并会把没变的存量撞号误判成新撞号（这条踩过一次，本轮自己写重了又自己测出来）。
4. 新脚本 `research/scripts/migrate-changelog-format.py`：一次性搬迁工具。把两份按月文件的 591 条
   原条目按决策重新分组、日期分组、编号重新按新规则计算，写回 `decisions-history.md`；把
   `experiments-history.md` 的 263 条原条目同样按实验重新分组；把 22 份单个 kb 文件（`invariants.md`、
   `checks-owed.md` 等）里同一天出现 ≥2 条融合写法的日期块折叠成新形态，只有 1 条的不动。内容不丢
   由脚本自带的自检把关：每条原条目在输出里出现的次数必须恰好等于它点名的决策/实验个数，改坏一个字
   或多抄/少抄一份都会喊；没点名任何决策/实验的条目（591 条里 1 条、263 条里 3 条）不强行归类，
   写进单独的 `research/prompts/migrate-changelog-format-orphans-2026-09-28.md`。已经真跑过，两道
   门禁（30 号、history-ordinal）在真仓上都绿。
5. `kb-scribe.md`、`kb-spec-drafter.md`、`format-evolution.md`、`decisions.md`、
   `.claude/skills/decide/SKILL.md`、`research/scripts/kb-spec-check.py`（连它的样本）与若干处引用
   旧路径/旧锚点的文字（`checks-owed.md`、`milestone/02-second-txn.md`、
   `experiments/31-AAD缺快照维.md`、`.claude/term-rename-exempt`、`.claude/gate.d/20-doc-decision-
   documents.sh` 的注释与出路文字）同步改写，不再提按月文件，旧的「日期（其N）」锚点改指新位置。
6. **上游共享脚本 `doc-lint.sh` 的一个真回归，已经修**：真跑一遍 `bash .claude/singlefs-ai-sop/
   scripts/doc-lint.sh .`，`decisions-history.md` 报出几十处「本轮锚不到具体哪一轮」「不许用自指
   称呼」「不许用上下文指代」——这些历史条目在旧的按月文件里从来没被这几条检查扫到过：`body_of()`
   在第一次遇到字面「## 历史版本」时就停止扫描，旧的按月文件把这个标题放在文件头第 6 行，之后
   全部原文都落在扫描范围之外；搬迁之后 `decisions-history.md` 把 28 个 `## D<n>` 节全部放在它
   自己那个「## 历史版本」（记文件本身怎么变过）之前，于是这些历史条目第一次被当成「正文」扫描，
   而它们的写法（casual 的「本轮」「如上所述」这类词）从来没打算满足「正文」的自包含要求。
   修法：给 `doc-lint.sh` 加一张新牌 `<!-- doc-lint:history-registry -->`（只许出现在 kb 里），
   `decisions-history.md`、`experiments-history.md` 顶上各贴一张；`body_of()` 见到这张牌时把
   扫描截止点从「文件自己的『## 历史版本』」提前到「第一个『## 』标题」——效果是把这两份文件的
   每个 `## D<n>`/`## E<n>` 节都当成「历史节之后的内容」，恢复它们本来就该有的豁免。这条改在
   `.claude/singlefs-ai-sop/`（上游共享副本），已经填进上游兄弟仓 `singlefs-ai-sop-zh`（没动
   `VERSION`、没提交）并同步回项目副本。真跑 `doc-lint.sh` 全仓验证四次：改之前
   `decisions-history.md` 单独报出的位置/自指/时间指代类违规几十处；贴上 `history-registry`
   牌并把 `decisions-history.md`、`changelog-format.md` 自己写犯的三处位置指代改掉之后，
   最后一次全仓复跑，位置/自指/时间指代与历史陈述这几类里一处都不再点名这一轮碰过的文件——
   剩下的违规（`experiments/142-新池新建文件的干跑.md`、`experiments/162-崩溃放量判定块存储选型.md`、
   `experiments-history.md` 各自的编号裸引用，两份 `records/` 文件的翻译腔）现查
   `git show HEAD:<路径>` 确认都是这一轮之外别的会话新写的内容，产物见
   `/tmp/doclint-final.log`（末行「2 个文件违规、21 处编号定义/引用不合规」，都是这五处）。
7. 用户当场指出 `rules/writing-discipline.md` 的「## 说人话」这个小节标题不合适，改成
   「## 文风要简单自然」；同一份文件与 `doc-lint.sh`、`.claude/agent-common.md` 里引用这个小节名的
   几处一并改了，正文里作为普通词语使用的「说人话」（不是小节标题）没有动。同样填进上游、同步回项目副本。

## 要腿答的

1. **编号判据会不会漏判或误判撞号**：新 `shape` 格判「同一节里同一天的 `### ` 标题不许出现两次」，
   但不检查「同一节同一天下，两个 `#### ` 子标题的点名词 + 编号完全一样」——这条是
   `history-ordinal.sh` 管的，两道门禁管的范围有没有缝隙？举一个具体的输入：两个 `#### 已定项
   7（其一）：……` 挂在同一个 `### 2026-09-28` 下、内容不同，30 号的 `shape` 格会不会放过它、
   `history-ordinal.sh` 真的会抓到吗（`.claude/singlefs-ai-sop/scripts/history-ordinal-keys.py` 的
   `stem_and_ordinal` 与 `keys_of`）？
2. **`--mention-scope heading-and-quick` 会不会漏掉真正相关的决策**：`migrate-changelog-format.py`
   按「标题或 `> 快查·` 两行」里出现的 `D<n>（`/`E<n>（` 决定一条原条目进哪几节，正文（改前/改后/
   依据段）里提到的决策不算。举一个具体反例：一条原条目的「依据」段点名了某条决策、但标题与快查两行
   都没提，这条决策的读者会不会因为这条历史条目没进它的节而错过关键改动？
3. **`status-sync` 的「现状」行会不会说谎**：它只比对 `decisions.md`/`experiments.md` 索引表当下的
   「状态」「结论（简报）」两格与变更史节顶那一行是否一致，不检查这两格本身对不对。若索引表本身
   滞后（决策已经推翻但索引没更新），`status-sync` 会判「一致」、掩盖索引表的滞后——这算不算这条
   检查本身的一个设计缺口？该不该在这一轮就补，还是记欠账？
4. **`migrate-changelog-format.py` 的并发安全够不够**：脚本在读输入与真正写文件之间重新核一次
   sha256，变了就跳过那份文件。若在「核完 sha256」与「实际 `os.remove`/写文件」之间的极短窗口里另一个
   会话写入，会不会被漏判？这个窗口今天有多宽，值不值得再收紧（比如原子替换 + 文件锁）？
5. **决策与实验变更史里，历史条目自己引用别的历史条目（「本文件第 N 行」「某日（其N）」这类自引用）
   有多少条，这一轮改没改全**：已知修了两处主 agent 现查到的（D6/D22 互相引用「2026-09-06（其三十六）」
   那两处），`migrate-changelog-format.py` 的自检没有专门查这一类自引用会不会因为搬迁而失效——
   这是不是还有遗漏，值不值得再扫一遍？
6. **`<!-- doc-lint:history-registry -->` 这张牌是不是开得太宽**：它让 `decisions-history.md`、
   `experiments-history.md` 里每个 `## D<n>`/`## E<n>` 节，从第一个 `## ` 标题起，全部豁免
   D-1/D-2/D-3（上下文指代/自指称呼/时间指代）与 A（历史陈述词表）四类检查——这四类原本就是为了
   「一条事实被单独检索出来时还能不能自足」而设的，而决策变更史的条目恰恰是**最常被单独引用**的
   一类内容（`.claude/rules/three-way-inference.md` 整篇都在讲怎么引用历史条目）。这张牌是照抄
   了「文件自己的『## 历史版本』之后天然免检」这条既有豁免的边界，但既有豁免针对的是**偶发的
   历史附录**（一份文件里几条），这里放开的是**两份文件里全部的历史正文**（591 + 263 条原条目，
   以后还会一直长）。这算不算把一个「小范围可接受」的豁免搬到了「大范围可能不该接受」的地方？
   如果不该照搬，更合适的做法是什么——只放开 D-3（时间指代，理由最直接：条目本来就在日期块下面）、
   继续判 D-1/D-2？还是维持现状？
