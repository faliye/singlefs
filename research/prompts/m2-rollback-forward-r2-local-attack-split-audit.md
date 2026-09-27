# m2-rollback-forward-r2 本地攻方提示：拆分核对表

原提示（上一任撞退出码 3 留下的那份，原样不改）：`research/prompts/m2-rollback-forward-r2-local-attack.md`（行号见下，均为该文件自己的行号，已用 `grep -n` 现查）。
拆出两份：`research/prompts/m2-rollback-forward-r2-local-attack-part1.md`（只问 task one：16 格 F_生效 / 回退候选集表）、`research/prompts/m2-rollback-forward-r2-local-attack-part2.md`（只问 task two：B1 卸载串的空发布次数）。
措辞与事实不重新翻译，全部取自原提示已定稿的英文文本（原提示的逐句核对表 `research/prompts/m2-rollback-forward-r2-local-attack-translation-audit.md` 已核过一遍，这次只做「取哪几段」的记账，不复核译文本身）。

## 段落分配表

| 原提示片段 | 原提示行号 | 进 part1 | 进 part2 | 处理 |
|---|---|---|---|---|
| Part 1 背景段第一段（disks / root record / txg / empty publish / ring regions / F / F_effective / instance-checkpoint 免责句） | :3 | 整段照抄，行 1-3 | 只抄前三句（disks；root record/txg/empty publish；ring regions/disk 归属），行 1-3；immune-disclaimer 句挪到本段末尾一并保留 | part2 删去第4、5两句（F 的定义句、"Once raised (subject to the rule given in clause 2 below)…F_effective" 那句）——task two 全程不用 F，且该句里"clause 2 below"这个前向指针在 part2 里找不到对象（part2 的 Clause 2 是区域轮转公式，不是 F 生效规则），留着会指向错的从句，故删；immune-disclaimer 句（"instance and checkpoint 不需要"）两份都留，无删改理由，原样保留 |
| Part 1 背景段第二段（F_old=10 / F_new=30 / "if…condition…does not hold…Part 3" 那句） | :5 | 整段照抄，行 5 | 整段不取 | part2 的 task two 全程不引用 F_old / F_new，也不引用"the table in Part 3"（part2 没有这张表），整段与 task two 无关，删去 |
| Part 2 标题 | :7 | 照抄，行 7 | 照抄，行 5 | 无 |
| Clause 1（F 只住根记录、不镜像） | :9-10 | 整条照抄，行 9-10，编号不变（仍是 Clause 1） | 不取 | Clause 1 只在 task one 的读法讨论里用得上（"unreadable"这个词的语义），task two 从不引用 Clause 1 |
| Clause 2（生效门槛 + 取各盘 max 之后跨盘取 min） | :12-13 | 整条照抄，行 12-13，编号不变 | 不取 | task two 不涉及 F_effective 计算 |
| Clause 3（回退候选集规则） | :15-16 | 整条照抄，行 15-16，编号不变 | 不取 | task two 不涉及回退候选集 |
| Clause 4（候选改法：跨盘取 max） | :18-19 | 整条照抄，行 18-19，编号不变 | 不取 | task two 不涉及 C419 候选改法 |
| Clause 5（R = 3） | :21-22 | 不取 | 整条照抄，改编号为 Clause 1，行 7-8 | task one 从不引用区域数 R；part2 里改编号是因为 part2 只保留原 Clause 5-8 这连续四条，为免一份独立文件里"Clause 1"从缺、编号从 5 起跳显得像丢了前四条，改按 1-4 连续编号，条款正文一字不改 |
| Clause 6（区域 = txg mod R） | :24-25 | 不取 | 整条照抄，改编号为 Clause 2，行 10-11 | 同上；task two 正文里所有原文"clause 6"字样同步改成"clause 2" |
| Clause 7（区域-盘归属 0/1/0 + 后续计数通则） | :27-28 | 不取 | 整条照抄，改编号为 Clause 3，行 13-14 | 同上；task two 正文里所有原文"clause 7"字样同步改成"clause 3" |
| Clause 8（候选 B1 定义） | :30-31 | 不取 | 整条照抄，改编号为 Clause 4，行 16-17 | 同上；task two 正文里"clause 8"改成"clause 4" |
| Part 3 标题（task one） | :33 | 照抄，行 21 | 不取 | — |
| Part 3 正文（"Consider the moment recovery looks…"那段） | :35 | 照抄，行 23 | 不取 | — |
| State N / NU / O / OU 四态定义 | :37-40 | 整段照抄，行 25-28 | 不取 | task two 不用这四态 |
| "There are 16 rows…"起到"Write the table row by row…"止（含 16 行说明、items 1-6、书写格式） | :42-52 | 整段照抄，行 30-40 | 不取 | — |
| Part 4 标题（task two） | :54 | 不取 | 改标题为 Part 3，行 19，内容不变只是 Part 编号从 4 改到 3（part2 文件自己不含 task one 的 Part 3，顺序重排后 task two 自然落在 Part 3 的位置） | — |
| Part 4 正文第一段（B1 链定义 + t/t+1/t+2 递推） | :56 | 不取 | 照抄，行 21，文中"clause 8"→"clause 4"、"clause 6's rule"→"clause 2's rule"、"R = 3 from clause 5"→"R = 3 from clause 1"、"clause 7's region-to-disk assignment"→"clause 3's region-to-disk assignment"、"clause 7 gives for later mounts"→"clause 3 gives for later mounts"，均为编号跟随 Clause 5-8→1-4 的机械改写，语义不动 | — |
| "Because only t mod 3…"段 | :58 | 不取 | 照抄不改，行 23 | — |
| "For each of the three cases…"段（含"as given in clause 7"那句） | :60 | 不取 | 照抄，行 25，"as given in clause 7"→"as given in clause 3"（同上机械改写：这句指的是 Clause 7/3 正文里"up to R times"那个通则，不是 R=3 这个数值本身，因此对应改到 3、不是改到 1） | — |
| Part 5 标题（Closing questions） | :62 | 改标题为 Part 4，行 42，"Closing questions"不变 | 改标题为 Part 4，行 27，"Closing questions"改单数"Closing question" | part2 只剩一问，标题改单数 |
| Closing questions 引导句（"After completing both tables…Number your answers 1, 2, 3…"） | :64 | 改写：「both tables」→「the table」，「three questions」→「two questions」，「1, 2, 3」→「1, 2」，「row or case」→「row」（part1 没有 case），行 44 | 改写：「After completing both tables…」→「After completing the three cases above…」，「the following three questions. Number your answers 1, 2, 3」→「the following question. Number your answer 1」，「Each answer must point to the specific row or case in the tables above」→「The answer must point to the specific case above」，「what entry in the tables above」→「what entry in the cases above」，行 29 | 两份各自砍成只对自己那一问成立的引导句，字面从原句机械摘取、只删不增（除下条列的"three cases above"这几个词，见下） |
| Question 1（txg=20 两种取法结果不同的行） | :66 | 整句照抄不改，行 46 | 不取 | — |
| Question 2（disk1 在 O/OU 但 F_生效 仍等于 30 的行） | :68 | 整句照抄不改，行 48 | 不取 | — |
| Question 3（三种情形里最小/最大空发布次数） | :70 | 不取 | 整句照抄，只把题号从"Question 3"改成"Question 1"（part2 只有这一问，从 1 开始编号），行 31 | — |
| Part 6 标题（Formatting rules） | :72 | 改标题为 Part 5，行 50 | 改标题为 Part 5，行 33 | — |
| Formatting rules 正文 | :74 | 改写：「clause 1 through clause 8」→「clause 1 through clause 4」；删去「the case labels, t mod 3 = k,」（part1 没有 case）；「Part 1, Part 2, Part 3, or Part 4」→「Part 1, Part 2, or Part 3」（part1 只有这三节给事实，原 Part4 task two 不在这份文件里）；其余「row labels」「disk 0 and disk 1」「不编代码行号」「不许 yes/no」「UNDEFINED」等全部照抄不改，行 52 | 改写：「clause 1 through clause 8」→「clause 1 through clause 4」；删去「the row labels, Row (disk0=X, disk1=Y),」（part2 没有 row）；「Part 1, Part 2, Part 3, or Part 4」→「Part 1, Part 2, or Part 3」（part2 自己的 Part1/2/3 = 背景/条款/task two）；「Do not answer any item, row, or question」的「row」→「case」（part2 没有 row，只有 case）；其余照抄不改，行 35 | 两处「clause 1 through clause 4」字面相同但指向不同的四条条款（part1 指 F 相关四条，part2 指区域几何四条）——这是拆分带来的必然结果，不是新事实 |

## 多出 / 少了逐条列（合并小结，明细见上表「处理」列）

- **part1 少了**：Clause 5-8（区域几何、B1 定义）、Part 4 task two 全文、Question 3、原背景段第二段（F_old/F_new/Part3 指路句在 part1 里留着，因为 part1 确实有 Part 3）——均因 part1 只问 task one，与 task two 无关。
- **part2 少了**：Clause 1-4（F 生效与候选集规则）、Part 3 task one 全文（含 16 格表与 State 定义）、Question 1、Question 2、背景段第一段的第 4、5 两句（F 定义、"clause 2 below"指针）、背景段第二段整段（F_old/F_new/"table in Part 3"指路）——均因 part2 只问 task two，从不引用 F。
- **part2 多出**（相对于逐字照抄）：无新增事实句。唯一的"多出"是机械性的编号改写本身（Clause 5→1、6→2、7→3、8→4 及其在正文与格式规则里的全部引用点同步改号），这不引入新事实，只是让一份独立文件的条款编号从 1 连续起跳，不留 1-4 空缺；改动点已在上表逐条列出。
- **两份都改的引导句/格式规则**：均为原句的删减 + 数字改写（three→two/one，1,2,3→1,2 或 →1，both tables→the table / the three cases above），未添加任何原提示没有给出的事实或限定词。

## 与原提示逐句核对表的关系

`research/prompts/m2-rollback-forward-r2-local-attack-translation-audit.md` 记录的是英文译文对中文 kb 原文的忠实度，这份表不重复那一层；这份只记「哪几段进了哪份新文件、哪几段没进、进的时候动没动字面」。两份合起来才是这一轮本地攻方两份提示的完整可追溯记录。
