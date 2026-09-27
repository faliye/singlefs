# defs-gatebatch-m2-r3 本地攻方 L3：英文提示逐句核对表

来源：正文 `_defs-gatebatch-m2-r3-body.md` L3 一格（第 15、48、50 行）与 `.claude/gate.d/stage-inputs.tsv` 第 36–42 行、`research/scripts/admission.py` 第 28–32、189、1592、1636、1667、1704、1829、200 行。

| 英文项 | 原文文件:行 | 首稿缺的限定词 | 定稿 |
|---|---|---|---|
| "Check, cell by cell, each of the seven `crash-case:` registration lines in `.claude/gate.d/stage-inputs.tsv` (today's lines 36-42)." | `_defs-gatebatch-m2-r3-body.md:15` 「逐格核：…」 | 首稿漏了「今天的」（今天文件里的样子，不是历史版本）；补上 today's | 加了 today's，与「今天」对应 |
| "For each registered judging field (count-line=, exhaustive=, threads=, shard=, etc.), state exactly what value is registered." | 同上 | 原文只列了这四种字段加「等」，首稿写全了 count-line/exhaustive/threads/shard，加 etc. 对应「等」 | 保留 etc. |
| "State which lines of output that test case's own source code can actually produce (verbatim grep results are given to you below; do not re-derive them from memory)." | 同上「那条用例的源码打得出哪些行（grep 原样）」 | 首稿漏了「grep 原样」这一限定——必须依据给定的 grep 证据，不是凭自己对 Rust 语义的猜测；补一句括注 | 加括注「evidence given below」 |
| "State what the quick tier (54号 without --full) and --full each judge for every field, citing the judging function name in admission.py (not a line number)." | 同上「54 号快档与 --full 对每个字段各判什么（引 admission.py 的判法行号）」 | 原文写「判法行号」，但派发提示第 15 行与本轮总纪律都要求答复不写代码行号，改写成「引函数名」；这是遵守派发规则做的必要偏离，不是漏译，单列说明 | 改成引函数名，不引行号（本轮总要求：答复不写代码行号） |
| "List, separately: (a) fields that are registered but the test's own source cannot produce; (b) determinations the source can produce but are not registered; (c) any of the seven cases that has no thread determination (no threads= registered) at all." | 同上「登记了而用例打不出的字段、用例打得出而没登记的判定、没有线程判定的用例，各列出来」 | 首稿把三类合并成一句列举，遗漏「各列出来」隐含的「分开列，不要混在一起」；改成 (a)(b)(c) 三个分号列表，并加 separately 对应「各」 | 加 separately 与三段式列举 |
| "The local attacker leg and the cloud attacker leg (Opus) do not overlap: Opus attacks the code semantics of the fixes and of the standing shapes; you only check the literal registered fields and what the seven test cases can literally produce, cell by cell." | `_defs-gatebatch-m2-r3-body.md:50` 「两条攻方腿不重叠：Opus 攻改法与站住形态的代码语义，本地只逐格核七条登记行的字面与用例打出的行」 | 无实质遗漏；补一句英文里的 leg 对应「腿」，把「逐格」明确译成 cell by cell 避免读成整体判断 | 保留，加 cell by cell |
| "Do not use markdown emphasis anywhere in your answer." | 三方规则「提示里不许用 markdown 强调」（本轮对模型的转述，非直引原文一句） | 这是纪律要求转述给模型的操作指令，不是引用材料原文，不需要与某一句中文原文比对；仍列此行说明来源 | 保留 |
| "Do not cite source-code line numbers or file line numbers anywhere in your answer; refer to function names, or to the row numbers of the tables below, instead." | 派发提示「提示里明令答复不写代码行号与文件行号（按函数名、表格行号指）」 | 直译，无遗漏 | 保留 |
| "For every answer, state what observation would overturn it." | 本地攻方定义「每条答复写『什么现象会推翻它』」 | 直译，无遗漏 | 保留 |
| "Answer by numbered item, matching the numbering of the questions below." | 本地攻方定义「答案按编号」 | 直译，无遗漏 | 保留 |

英文比原文多出来的限定词/括注，单列在上表「首稿缺的限定词」一列里已经写明为什么加；没有额外发现别的、原文没有而英文擅自添加的实质性内容。
