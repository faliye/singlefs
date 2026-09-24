# m2-wave3-code-r1 本地攻方提示：转述核对表

范围说明：本表只登记提示文件里**转述自中文原文**的句子（mutations.tsv 的描述列、被这一轮 diff 删掉或新加的 Rust doc 注释）。
提示文件里大量的「结构事实」（改了哪个函数、pub 还是 private、mutations.tsv 有没有匹配行）是我自己读 diff/源码之后的观测转写，不是对某一句中文原文的转述，不登记在这张表里。

首稿：直接写成提示文件里现在这句之前的版本（第一次用 `cat > ... <<'EOF'` 写进提示文件时的原文），不是另外留了草稿文件。

| 英文项（提示文件里，Question 编号定位） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Row 1（extent 叶记录指针） | crates/mutations.tsv:331 | 「步 1 验收第 4 条」这个标签 | 补上「(this is step 1 acceptance item 4 in the project's working notes)」 |
| Row 2（inode 写入时间） | crates/mutations.tsv:332 | 同上标签 | 补上，并注明与 Row 1 同一项 |
| Row 3（反向链） | crates/mutations.tsv:333 | 同上标签 | 补上，并注明与 Row 1、2 同一项 |
| Row 4（取号之后没有屏障） | crates/mutations.tsv:334 | 「步 3 验收」标签 | 补上「(this is step 3 acceptance in the project's working notes)」 |
| Row 5（层 0 按发布分状态数） | crates/mutations.tsv:335 | 「步 6 验收第 1 条」标签 | 补上「(this is step 6 acceptance item 1 in the project's working notes)」 |
| Row 6（C481 基线抽样） | crates/mutations.tsv:336 | 「C481」这个编号标签 | 补上「(this is labeled C481 in the project's working notes)」 |
| Row 7（C378 回卷） | crates/mutations.tsv:337 | 「C378（认了）」这个编号与状态标签 | 补上「(this is labeled C378, accepted, in the project's working notes)」 |
| Question 8：model.rs 被删枚举成员 `RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion` 的 doc 注释 | 该注释已被这一轮 diff 删除，只存在于 research/prompts/_m2-wave3-code-r1-diff.md:4220-4222（删除行，`-` 前缀）；对应的 mount.rs 那一份更长的同主题注释见下一行 | 首稿把 model.rs 这条（短）注释与 mount.rs 那条（长）注释的内容混在一起转述，错误地给 model.rs 这条也加上了「在它有条款之前，这一格在任何写之前拒绝」与候选排除那半句——这两句 model.rs 的原文里没有 | 改成只译 model.rs 自己那三行原文，并加一句说明：这条短注释本身不提「拒绝」与「候选排除」，那两点只在 mount.rs 那条更长的独立注释里（指到 Question 10） |
| Question 8：model.rs `answer_mount_rollback` 里新加的替换注释 | 对应 diff research/prompts/_m2-wave3-code-r1-diff.md:4257-4259（这是这一轮新加的行，`+` 前缀；当前文件里已经是这样） | 漏了「D23（journal 的角色与格式） 已定项 14」与「C511 第 3 步」这两处引用 | 补回「(settled item 14 of decision D23)」与「(this is step 3 of C511 in the project's working notes)」 |
| Question 10：mount.rs 被删枚举成员 `RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion` 的完整 doc 注释 | 该注释已被这一轮 diff 删除，只存在于 research/prompts/_m2-wave3-code-r1-diff.md:2920-2928（删除行） | 「树 11」被泛化成了「a tree」；结尾「D23 已定项 14」「C493（回退候选集条文与实现说反话） 由此还清」两处引用漏译 | 改回「tree 11」；结尾补上「per settled item 14 of decision D23, which is why C493 counts as cleared by this」 |
| Question 10：mount.rs 新加字段 `tree_identifier_watermark_of_the_ring` 的 doc 注释 | 该注释是这一轮新加的（`+` 前缀），当前文件里已经是这样，对应 diff 里 mount.rs 那一段（研究 `research/prompts/_m2-wave3-code-r1-diff.md` 里 `crates/singlefs-core/src/mount.rs` 那一块 `@@ -295,6 +282,10 @@` 之后新增的字段注释） | 漏了「D8（核心索引结构） 已定项 8 ②」引用与被引函数名 `recovery::highest_tree_identifier_watermark_in_the_ring` | 补上「(settled item 8 clause 2 of decision D8; this is computed by the function recovery::highest_tree_identifier_watermark_in_the_ring, see question 12)」 |
| Question 14：transaction.rs 改写后的 `FirstFileVersionOnAVersionThatAlreadyHasAFile` doc 注释 | 对应 diff research/prompts/_m2-wave3-code-r1-diff.md:3559-3564（这一轮新加的行） | 漏了「（2026-09-23 崩溃注入快档打中）」这句发现来源，以及「不再是 mkfs 的 11」里的具体数字 11 | 补上「(this was hit by a 2026-09-23 fast-tier crash-injection run)」；补回「the mkfs value of 11」 |
| Question 11：mounted_read.rs 新加注释 | 对应 diff research/prompts/_m2-wave3-code-r1-diff.md:2880-2881（这一轮新加的行） | 核对未发现遗漏限定词 | 未改动，原样保留在提示文件里 |

## 提示文件里比原文多出来的限定词/括注

| 多出来的地方 | 多的是什么 | 为什么加 |
|---|---|---|
| Row 1（extent 叶记录指针） | 「instead of the new one」 | crates/mutations.tsv:331 原文只说「读回等于旧内容」，没有显式对照「新内容」；补这半句是为了让没看过测试脚手架的读者也能明确知道「旧内容」是相对谁而言的旧，不改变原意 |
| Question 8：model.rs 那条短注释译文末尾 | 一整句「This shorter comment in model.rs does not itself mention refusing the write or the candidate-exclusion question; those two points appear only in the longer, separate doc comment on the matching MountError variant in mount.rs, given in question 10.」 | model.rs 与 mount.rs 这两条注释文字相近但不是同一条；Question 8 与 Question 10 分别问的是两个不同文件的第 D 列（是不是这个文件里的没写条款的选择），不加这句本地模型容易把两条注释当成同一条来源、答错「是哪个文件」 |

## 没有转述、不登记进上表的部分（说明范围）

提示文件 Question 3（walk.rs）、Question 7（history.rs）、Question 9（model_comparison.rs 前两处）、Question 12（recovery.rs）等处的「这一round的 diff 改了什么」描述，是我自己读 diff 与当前源码之后用自己的话写的结构性事实（新增/修改了哪个函数、pub 还是 private、mutations.tsv 里有没有匹配行），不是对某一句中文原文逐句转述，因此不适用本规则、不登记进上表。
