# 逐句核转述：m2-wave2-code-r1 本地攻方腿（Z4 算术那一半）

来源：`.claude/kb/invariants.md:131`（I-3.9 整行）、`.claude/kb/invariants.md:274`（I-9.14 整行）、
`crates/singlefs-checker/src/walk.rs:1099-1117`（`judge_release_generation_and_tree_table_birth` 的文档注释与候选集 < 2 那个分支，行号本轮现查）。
提示文件：`research/prompts/m2-wave2-code-r1-local-attack.md`。

## 一、逐句对照（英文项 / 原文文件:行 / 首稿缺的 / 定稿）

| 标签 | 英文项（定稿） | 原文文件:行 | 首稿缺的 |
|---|---|---|---|
| 标题 I-3.9 | "the release generation falls within the interval where references to it have stopped" | invariants.md:131（分项名） | 无 |
| S1 | "For any allocation record that carries the released flag, its release generation must fall within the interval (checkpoint_txg …, checkpoint_txg …]. This interval is open on the left end and closed on the right end: a value equal to the left endpoint is excluded, a value equal to the right endpoint is included." | invariants.md:131 第一分句 | 无（末句「开左闭右」是把原文的数学记号 `(…, …]` 转成文字解释，不是原文没有的新限定，只是把符号讲清楚——是否要记进「添加」见下节） |
| S2 | "When the root ring has no hole, this interval contains exactly one value, equivalent to saying the release generation equals the txg of the earliest root that no longer references it." | invariants.md:131 第二分句 | 无 |
| S3 | "If every valid root still references the placement, it must not carry the released flag." | invariants.md:131 第三分句前半 | 无（括注见下节「有意删掉」） |
| S4 | "When the ring has a hole (an applied journal record's release write carries a txg that no root slot has ever written), the interval form above is used for judging; judging by plain equality instead would flag a legitimate image as red." | invariants.md:131 第四分句 | 无（括注里的实测日期与状态数见下节「有意删掉」） |
| S5 | "An already-reclaimed released record that has not yet been reused is skipped and not judged, because the root that witnessed its release is no longer in the candidate set, so that interval cannot be located; when every record is of this kind, the whole invariant reports not applicable, with a reason given." | invariants.md:131 第五分句 | 无 |
| S6 | "Discriminating power: rewriting the release generation of publish B's occurrence from 4 to 3 must turn red, given that the earliest valid root that no longer references those ten placements is B, at txg 4." | invariants.md:131 第六分句 | 无 |
| 标题 I-9.14 | "a tree-table entry's birth txg is unchanged across roots" | invariants.md:274（分项名） | 无 |
| T1 | "For the same tree, identified by tree ID, its tree-table entry must have the same birth txg across the tree tables of any two valid roots in the ring." | invariants.md:274 第一分句 | 无 |
| T2 | "This value is set only when the tree first appears in the tree table." | invariants.md:274 第二分句前半 | 无（括注见下节「有意删掉」） |
| T3 | "When the same tree's entries occur in only one tree-table unit, the invariant reports not applicable, not holds. Several roots pointing at the same tree-table unit and comparing that same set of bytes against itself is a vacuously true determination." | invariants.md:274 第三分句 | 无 |
| T4 | "Discriminating power: changing the tree-table entry's birth txg so that it tracks the txg of the current publish must turn red." | invariants.md:274 第四分句 | 无 |
| C1 | "This single pass is shared by I-3.9 and I-9.14: it walks each root in the candidate set once, to collect its reference set and its tree table." | walk.rs:1099 | 无 |
| C2 | "When the candidate set has only one root left, both invariants report not applicable. Both \"the earliest valid root that no longer references it\" (used by I-3.9) and \"the same across roots\" (used by I-9.14) need a second root before they have any content." | walk.rs:1100 | 无（括注里的用户定案日期见下节「有意删掉」；两个「(used by …)」标记见下节「添加」） |
| Br1 | "The actual code branch: if the number of entries in candidate_indexes is less than 2, do the following three things and then return immediately, evaluating neither invariant any further in this call." | walk.rs:1108、1117（`if` 条件与 `return`，非原文句子，转述代码结构） | 不适用（转述代码控制流，没有对应的中文整句） |
| Br2 | "the rollback candidate set has only one root: there is no second root that can witness that it no longer references this placement" | walk.rs:1111（`not_applicable` 的第二个参数字符串） | 无 |
| Br3 | "the rollback candidate set has only one root: there is no second root's copy of the tree-table entry to compare against" | walk.rs:1115（`not_applicable` 的第二个参数字符串） | 无 |

判据（本表自查）：把「定稿」那一列与原文文件同一行并排看，逐字比对，除下面两节列出的之外没有再发现别的缺口。

## 二、英文比原文多出来的限定词 / 括注（单列，写明为什么加）

| 位置 | 加了什么 | 原文文件:行 | 为什么加 |
|---|---|---|---|
| S1 末句 | "This interval is open on the left end and closed on the right end: a value equal to the left endpoint is excluded, a value equal to the right endpoint is included." | invariants.md:131（原文只用数学记号 `(…, …]` 表达，没有这句文字解释） | 原文靠数学记号 `(`、`]` 表达开闭区间，本地模型读的是纯文本提示（未用反引号包代码记号，怕触发损坏闸），怕它把 `(a, b]` 单纯当普通括号读丢开闭信息；这句是把符号翻译成文字，不改变区间定义本身，只是把「已经在记号里」的信息换一种媒介说一遍。若这句话本身与原记号矛盾（比如说反了开闭端）才算真正的添加限定词；现查两处一致，不构成语义收严或放松。 |
| C2 两处 | 在 "the earliest valid root that no longer references it" 后加 "(used by I-3.9)"；在 "the same across roots" 后加 "(used by I-9.14)" | walk.rs:1100 | 原文这句本身没有点名哪个短语对应哪一条不变量，靠读者认出「最早不再引用它的那条有效根」是 I-3.9 的用语、「跨根相同」是 I-9.14 的用语（分别与 S1、T1 用词相同）；本地模型一次性看完整份材料，没有反问的机会，加两个括注只是把原文隐含、但两句引文本身就能对上的归属显式标出来，不引入新事实、不改变哪一格判「不适用」。若这两个标注把归属点错（例如把「跨根相同」标成属于 I-3.9）才算真出错；现查与 S1、T1 的用词逐字对得上。 |

## 三、有意删掉的括注（不算摘句：均为纯出处引用，不改变本轮两张表要算的区间/是否评估逻辑；仍然披露）

| 位置 | 删掉了什么 | 原文文件:行 | 为什么判断可以删 |
|---|---|---|---|
| S3（I-3.9 第三分句） | "（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））" | invariants.md:131 | 这一段是指向另外两条决策/欠账条目的出处引用，说明「为什么有释放代这个字段、它长什么样」，不影响「若每一条有效根都还引用它就不许带已释放标志」这条规则本身，也不出现在本轮两张表任何一格的取值或判据里。本地攻方腿禁读附录，给它这段引用只会多两个它验不了的编号（D3、C374），没有可核对的实质。 |
| S4（I-3.9 第四分句） | "，实测 2026-09-18 残留记录那条层 0 流的 12 个状态" | invariants.md:131 | 这是给「环上有洞」这个前提举的一次实测出处（哪次跑、几个状态），不改变「环上有洞时按区间判、按等于判会判红」这条规则；本轮两张表不问「洞有多长」「12 个状态是什么」，模型算表 1、表 2 用不上这个数。 |
| T2（I-9.14 第二分句） | "（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））" | invariants.md:274 | 与 S3 同理：指向 D8、C374 的出处引用，不改变「诞生 txg 只在树第一次出现时设」这条规则，本轮两张表不需要树表条目还有哪些字段。 |
| C2（doc comment 第二句） | "（2026-09-18 用户定案随 C374 立条时定的口径）" | walk.rs:1100 | 这是「候选集 < 2 时两条都报不适用」这个口径的决策时间与决策场合，不改变分支本身怎么判；Br1/Br2/Br3 已经把这条分支的实际代码行为（`< 2` 时两条都 not_applicable 并 return）转述给了模型，这句只是多一层「谁、什么时候定的」，模型算表 2 用不上。 |

判据（本节自查）：以上四处删掉的内容，逐条核对后都只指向别的决策编号或实测记录的出处，不含区间端点、开闭、候选集门槛、树表单元数这几个本轮两张表要用的量；因此判定删掉不构成「摘句」意义上的实质删减，但仍按规则单列披露，不当作默认可省略。
