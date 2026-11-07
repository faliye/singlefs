# E171（三oracle合谋测试） 第 2 次重跑登记

写于 2026-11-01，装置改动之前。批次 `records/2026-10-31-删兼容旧版本代码.md#12`。
上一次登记 `research/prompts/e171-r1-prereg.md`（sha256 `5d8fb709e32e0b61df009eb5fd71d2f01285e4eb8bf5ba3dca07d50a141ebf21`，2 988 行，含修订 1–14 与各条「主 agent 认定」）只读了判据怎么定、跑法怎么写；实验页只读了 `grep -n '^#'` 的标题行（读到的结果照第四节列）；`research/results/` 下的产物没打开，只算了 sha256 与行数。
差分重跑：输入变了（坏实现名单撤项与改引，第一节读法 3），臂、判据、O1–O3 的接法、跑法照上一次登记；各节没写的照「同上一次登记」读，第一至十四节节名照抄。

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 F1 F4 F11 F12 F13 F14 Z1 Z5 Z11 Z12 Z13 S1 S16 S17 S20 S21 S22 S23 A6 A21 A22 A23 A24 A25 A26 K1 K2 K3 M1 M2 M3 M4 M5 M6 M7 M8 M9 H3 H8 H16 C936 C938 -->

## 一、问题

英文名：three_oracle_collusion_after_the_oracle_fixes

沿用上一次登记第一节第 1 行。不另起装置：入库装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 与研究一侧 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 照用；原装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` 与研究一侧 `scripts/experiment-apparatus/e171_three_oracle_collusion/` 只改名单与引名单的地方（读法 3、4）。新写的只有研究一侧两份小脚本（撤项核算、名单核对，第五节 5.3）。

主 agent 给的问题（派发提示「关的是哪几条」原文逐字）：

> 1. 问题单 Q1：两份坏实现名单（`scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/bad-implementations-expansion-164.tsv`）第 119 行（编号 236，按变异名取 `crates/mutations.tsv` 里已删的「实一…读者仍把退役的位 0 当受支持的布局身份（旧镜像挂得上）」）撤掉之后，E171 的结论变不变：登记写死撤掉什么、钉住的常量（装置二进制的 `stream-constants`、名单项数、各 oracle 的合谋计数，现查哪些是钉值）怎么重核、与 r1 怎么逐格比、什么现象算结论变了。
> 2. 问题单 Q2：两份名单里还有没有别的项按变异名取不到 `crates/mutations.tsv` 现存的行；登记写明怎么逐项核、取不到的怎么处置（这一批同口径：为旧格式、旧位造的坏实现撤掉，不换）。
> 3. 名单文件名带的项数（`-164`）随撤项变不变、引它的地方（装置、`replay.sh` 登记、实验页）怎么跟着改，写进登记。

问题单 `research/prompts/e171-r2-questions.md`（sha256 `aea682aec825df3eb2bfbf443817304ae653ece873446f09586706ed72f9034d`）第 5–8 行逐字（`sed -n '5,8p'` 原样），就是本登记的岔路单：

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| Q1：坏实现名单撤掉「实一…读者仍把退役的位 0 当受支持的布局身份（旧镜像挂得上）」那一项之后，E171 的结论变不变 | 甲：结论不变，只是名单少一项、钉值跟着变；乙：结论变 | 撤掉之后三个 oracle 的合谋计数或判定在哪一格与撤掉之前不同 | 撤掉之后整张名单在主工作区现状上跑完，钉值重核，与 r1 的结论逐格对比 | 开着 |
| Q2：名单里还有没有别的项按变异名取不到 `crates/mutations.tsv` 现存的行 | 有 / 没有 | 按名字取行命中 0 次的项 | 两份名单逐项核过 | 开着 |

### 读法写死

1. **「E171 的结论」逐格指哪几份**（行级读数都按整表编号；一行的判定只取决于这一行自己的坏实现与树，行与行之间只经补位相关，读法 6）：
   - **原登记整表**：`e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`（F4 扩表，第 1–281 行）与 `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`（加编不过 6 行的补位），比对器 `scripts/experiment-apparatus/e171_three_oracle_collusion/collusion_report.py` 的 `q1_alpha`、`q1_beta`、`q1_gamma`（每种读法的 `verdict=` 与 `row_numbers=`）、`q1_o1_and_o2_both_quiet`、`q1_denominator`、`q1_column_red_rows`、`q3_alpha`、`q3_beta`（`only_<列>=` 各行）、`untouched_but_red`、`q1_denominator_by_table`。117 行那张的 s4 整表没有第 236 行，不算。
   - **r1 第二段（整表，修订 6 树）**：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out`，比对器 `segment_two_comparator.py` 的 `name=verdict` 那一行（`q2=`、`q2_set_rows=`、`q2_o1_red_rows=`、`d_<读法>_batch_<批>=` 四个、`rows_not_located=`）、`q2_set_row`、`q3_denominator_class`（`row_numbers=`）、`q3_row_125`、`not_located_row`、`substitution`。
   - **r1 第一段（Q1–Q5 逐行，修订 6 树）**：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out`。它只跑空变异与 15 个点名行（`rows_orchestrator.py` 第 155 行 `STEP_ROWS`、`segment_one_comparator.py` 第 31–34 行），撤项效应只用第七节 A23 的结构核对判，不读它的行；现状整表段才与它逐格比。
2. **「结论变」与「钉值跟着变」怎么分**（问题单 Q1 甲的字面「结论不变，只是名单少一项、钉值跟着变」）：
   - **判定词**：上面各份里取值是词的格——`q1_alpha` / `q1_beta` / `q1_gamma` 的 `verdict=`（`yes_a_row_is_quiet_on_every_oracle`、`no_such_row`、`not_enough_rows`、`not_reported_f7_strict_o3_reaches_almost_no_state`），r1 第二段的 `q2=`（`jia`、`yi`、`bing`、`not_enough_f4`）与 `row_125_counted_registration_batch_1=`，r1 第一段 Q1–Q5 的甲乙丙。
   - **钉值**：其余取值是数或行号列表的格（`counted_D`、`d_*`、`all_quiet_rows`、`row_numbers`、`only_<列>`、`q2_set_rows`、名单行数、sha256）。
   - Q1 判**乙**：撤项之后任一份的任一判定词与撤项之前不同。判**甲**：判定词全同；变了的钉值逐格列出，一格不漏。判定词全同而某个钉值是实验页、索引或 D13（验证路线） 已定项 6 正文里写着的数，照列成「要回改的现状句」，不因此判乙。
3. **撤掉什么、改引什么**（问题单 Q2 的处置；「取不到」= 读法 5 的取法命中 0 次）。按变异名取不到的项逐项分三类，每类的判法都能用命令判：
   - **甲·旧格式旧位**：名字在 `git show HEAD:crates/mutations.tsv` 里有、在开工那一刻的主工作区表里没有，且 `records/2026-10-31-删兼容旧版本代码.md` 第 4 批记它随兼容代码删了。**撤掉，不换**：两份名单删掉那一行，整表编号不重编（读法 4）。这一类今天只有第 236 行（第四节第 2 条）。
   - **乙·同一个坏实现、名字变了**：今天的表里有一行的（文件，原文，替换文）三样与它在 r1 用的那张表里那一行逐字相同，且这样的行恰一行，或其中恰一行的名字等于把它的旧名照 `.claude/kb/term-renames.md` 登记的改名换过之后的名字。**改引**：只改名单第 5 列成今天那一行的名字，第 1–4、6 列不动（第 6 列的行文摘要照旧，逐行驱动照 `bad_implementation_table.locate_by_name` 记 `line_matches_the_registration=false`、照今天的行跑）。
   - **丙·坏实现本身变了或没了，又不是旧格式**：三样在今天的表里找不到逐字相同的一行。**撤掉，不换**：同一个坏实现在今天的表里没有对象；改引同一处锚点上替换文不同的那一行，是换了一个坏实现，等于重挑（原登记 5.2「照名字取行、不重挑」，上一次登记修订 8 第 8.3 节「重新取锚是换了一行坏实现，是新实验」）；留着取不到的名字就是这一次要修的毛病。
   - 三类都对不上的（例：三样逐字相同的行不止一行、按改名也挑不出一行）记「未分类」，不改那一行，交主 agent（F13）。
   - 「r1 用的那张表」= r1 第二段那一趟冻结树里的 `crates/mutations.tsv`；那棵树不在工作区时，取 r1 产物里那一行的 `not_located_row` / `substitution` 记载与 `git log -S'<名字>' -- crates/mutations.tsv` 定它当时取不取得到，取法写进第十二节修订。
4. **整表编号不重编**：撤掉的行留空号，两张表第一列的并集就是行集；行集不再写死成 1–281。好处是逐行与 r1、原登记按编号一一对上。
5. **Q2 的取法**：与逐行驱动同一个读法——`bad_implementation_table.read_mutation_rows`（跳过 `#` 起头的行、只收恰 6 列的行）读开工那一刻的主工作区 `crates/mutations.tsv`（HEAD 加暂存区加未暂存），按名单第 5 列逐字比第 1 列，数命中次数；命中 0 次是「取不到」，命中 2 次以上是「名字不唯一」（另列，不在 Q2 的「取不到」里）。两份名单各核一遍。表的 sha256 写进产物；核的那一刻与改名单那一刻表的 sha256 不同走 S21。
6. **补位与撤项的关系**：逐行驱动的补位只在同一文件组里换下一行，跳过两张名单里所有的名字（`substitute_for`，`names_in_the_table_of_rows`）。撤掉一行之后，同文件组里别的行在旧树上补位时可能换到被撤那一行的变异。撤项核算把被撤的行从读数里减掉，只在「与被撤行同一文件、在那一趟里有过补位或定位失败」的行都没有时成立；有的，那几行走 F12。
7. **钉值怎么重核**（问题单 Q1 够判条件「钉值重核」）：分三样，各在第六节占一行——名单的项数与 sha256（第七节 A21）；研究一侧代码里写死的行集与项数（`segment_two_comparator.py` 第 35 行 `TABLE_ROWS = 281`、`rows_orchestrator_segment_two.py` 第 108–109 行与第 218–223 行、`count_reusable_units.py` 第 11 行与第 29 行、`rows_orchestrator_segment_two_revision_11.py` 第 345 行，执行员开工时 `grep -n '\b281\b\|\b282\b\|\b164\b\|\b117\b\|\b330\b'` 现列）；装置本地常量（`stream-constants --streams 1,2,3` 在今天的主工作区上，与上一次登记修订 12 的产物比）。各 oracle 的合谋计数不是代码里的常量，是比对器读产物现算的钉值，由读法 2 那一份逐格比管，不另钉。
8. **两条比法分开**：撤项效应只在 r1 与原登记那几棵旧树的读数上算（今天的树上被撤的行没有对象，量不到「撤之前」，第八节）；现状整表段在今天的树上重跑整张名单，与 r1 逐格比，差出来的格逐行归因（被撤的行、改引的行、树变了的行），不并进 Q1 的判定词。

## 二、被测条款与它引的定义

用 `scripts/kb/quote-kb.py` 整段抄一段：D15（格式冻结政策） 已定项 4 全节（派发提示点名的被测条款：位 0 空出、读者只收这一版布局的位图、格式锁定之前位号可回收）。出口在草稿目录 `quotes.md`，回读逐字节一致；追加进来之后 `tail -n 30 登记 | cmp - quotes.md` 再比一次（命令在第十三节）。
E171 自己的被测条款 D13（验证路线） 已定项 6「三个互相独立的 oracle」与 D13 已定项 1、D23（journal 的角色与格式） 已定项 14：同上一次登记第二节（这一次的判据不引它们的新句子；它们正文里写着的 E171 数若随撤项变了，照读法 2 列成「要回改的现状句」）。实验页 `.claude/kb/experiments/171-三oracle合谋测试.md` 不抄：它的结果与结论各节这一次不读（第四节第 1 条）。

**出处 `.claude/kb/decisions/15-格式冻结政策.md:74-99`（整段抄，未转述）**

```markdown
#### 已定项 4：feature bit 的位分配登记表

**定案**：三个 bitmap 各 256 位（已定项 10 的三张位图），位号的唯一登记位是这一节的登记表。「不认识不许挂」要对默认布局也生效，所以第一条线自己占一位、mkfs 起就置上——不取「第一条线不占位」那条（那样不认识 SSD 布局的读者照样能挂）。**里程碑 17 收尾之前不存在格式锁定；格式锁定之前没有旧盘，位号可以回收、可以重排**，改位号时 `crates/` 与 checker 在同一次改动里跟着改。截断字段无条件带上：位 2 是布局身份的一部分，读者只收位 1、位 2 都带的那一种（incompat 位图第一个字节 0x06）。

| bitmap | 位 | 含义 | 出处 |
|---|---|---|---|
| incompat | 0 | 空出，可再分配 | 用户 2026-11-01 定案：格式锁定之前没有旧盘，位号可回收、可重排（原话在变更史） |
| incompat | 1 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行） | D12（目标介质） 已定项 5；用户 2026-09-26 定「换 incompat 位，挂不上」 |
| incompat | 2 | inode 记录带截断字段（偏移 120 起 T 与 N_w），截断字段无条件带上；位 2 是布局身份的一部分，mkfs 起就置上，读者只收位 1、位 2 都带的那一种 | 用户 2026-10-17 定新开一位，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节；登记的写回判决 `research/prompts/m4-validate-code-r1-main-verification.md` 第二节「截断字段与位 2」那一行；用户 2026-11-01 定截断字段无条件带上（原话在变更史） |
| incompat | 3..255 | 未分配；每开一条布局线追加一位 | D12（目标介质） 已定项 1 |
| compat_ro | 0..255 | 未分配 | — |
| compat | 0..255 | 未分配 | — |

**射程**：只定位号怎么分与第一位的归属，每一位的语义归申请那一位的决策。新池新建文件的字节：系统配置 feature bits 那 96 字节从全 0 变成 incompat 位 1 与位 2 置 1、incompat 位图第一个字节是 0x06，位图小端、位 1 与位 2 是第一个字节从最低位数起的第 2、3 位（D22（单元原子性怎么合成） 已定项 13）。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：

- E142（新池新建文件的干跑）：系统配置 feature bits 那 96 字节按这一项写出来，它的空白 G12 随之收口，[layout/01-first-txn.md](../layout/01-first-txn.md) 同步改。
- 用户定案，原话在变更史；定案之前那一格的原文也在变更史，含它当天写的「改字节：否」——那句已被这一项的定案推翻。
- 用户定案 2026-09-26：布局身份换成位 1（三方判决 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第二节 K7），原话在变更史。
- 用户定案 2026-10-17：新开 incompat 位 2（inode 记录带截断字段），三方判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节；位 2 登记的写回判决 `research/prompts/m4-validate-code-r1-main-verification.md` 第二节「截断字段与位 2」那一行；原话在变更史。
- 用户定案 2026-11-01：格式锁定之前没有旧盘，位号可回收、可重排，位 0 空出；截断字段无条件带上，读者只收位 1、位 2 都带的那一种（弹窗原句：incompat 位 2「全删，改决策」），原话在变更史。
- 用户定案 2026-11-01：里程碑 17 收尾之前不存在格式锁定（弹窗原句：「冻结时刻都改为格式锁定 但是里程碑17前不存在格式锁定」），原话在变更史。

**欠**：C183（每套布局一个 incompat 位而盘上差不出字节）。

```

## 三、实现今天的样子

2026-11-01 现查，`git rev-parse HEAD` 为 `59ce5a86550de2c2c9bc6b51b34b4f881313682c`；主工作区有别的会话的暂存与未暂存改动（`git status --porcelain` 里 `crates/mutations.tsv` 是 `MM`，`crates/singlefs-core/src/system_configuration.rs` 是 `M `，两个研究一侧目录里 44 份文件是 `M ` 或 `A `）。执行员开工时逐项再查，对不上的照实写进第十二节修订，不改判据。

- **退役位 0 在 `crates/` 里已不在**：`crates/singlefs-core/src/system_configuration.rs`（1 184 行，sha256 `2c41a1cbc2abab666832477391843105849dfeb0d91176feab18b5b6fcea4336`）第 27 行位 1 `0x02`、第 31 行位 2 `0x04`、第 34–36 行 `INCOMPAT_FIRST_BYTE_OF_THIS_LAYOUT` 是两位之并；第 857 行 `incompat_bits_are_mountable` 只收第一个字节恰为它、其余 31 字节全零（第 859 行）。`grep -rn 'INCOMPAT_RETIRED\|retired_layout_bit\|old_reader_supported_bits' crates/ | wc -l` 为 0。
- **第 236 行要的那一处改坏已没有对象**：HEAD 那张表里这个名字那一行的原文（含 `old_reader_supported_bits` 与 `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT`）在今天的 `system_configuration.rs` 里命中 0 次（草稿脚本 `tools/anchor_count.py`，第十三节）。
- **变异表**：`crates/mutations.tsv` 今天 sha256 `5e6617b21644b64acd1e6ede66144ef11002d4918b669e7d06ecd4c89ac32229`。
- **名单读法**：`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/bad_implementation_table.py`（sha256 `19d03b31…`）第 74 行 `read_mutation_rows`、第 114 行 `locate_by_name`（命中 0 次 `not_found`、2 次以上 `ambiguous_name`）、第 150 行 `group_names_of_file`、第 158 行 `substitute_for`；第 166 行 `if failed_name not in group_names: return None`——按名字找不到的行，失败的名字不在今天的组里，补位交回 None，这一行记 `not_found`、不补。原登记 5.2 的补位规则写的是「按名字找不到」也照组内下标 j + 1 换；代码与那一句不同，从 r1 起就是这样（第四节第 5 条）。
- **逐行驱动**：同目录 `row_driver.py` 第 360 行 `prepare_row`（照名字取、核锚点、编之前的失败按补位换）；第 638 行 `--selected-rows` 默认 `bad-implementations-117.tsv`、第 642 行与第 689 行另一张默认 `bad-implementations-expansion-164.tsv`。
- **写死行集与项数的地方**（读法 7）：`segment_two_comparator.py`（sha256 `3b22a1d0…`）第 35 行 `TABLE_ROWS = 281`、第 231 行 `q2_verdict`、第 377 行 `q3_lines`；`rows_orchestrator_segment_two.py` 第 108 行 `TABLE_FILES`、第 109 行 `WHOLE_TABLE_ROWS = tuple(range(1, 282))`、第 218–223 行「不是恰好 1–281 就拒」；`count_reusable_units.py` 第 11 行「单元总数 330」、第 29 行；`rows_orchestrator_segment_two_revision_11.py` 第 345 行；`reuse_library.py` 第 135 行 `SELECTED_ROW_TABLES`；`frozen_tree_lines_revision_8.py` 第 95 行 `ROW_TABLES`；`rows_orchestrator_segment_two_revision_10.py` 第 102–103 行 `INPUT_RESEARCH_FILES`；`run_segment_two_on_the_revision_6_tree_multi_host.sh` 第 31 行。原目录：`expansion_table.py` 第 26–27 行（`FIRST_EXPANSION_ROW_NUMBER = 118`、`EXPECTED_EXPANSION_ROWS = 164`，原登记挑法的事实）、`comparator.py` 第 106 行（`expected_step_4_only=164`）与第 114–131 行 `expansion_table_lines`。
- **原登记比对器的判定词**：`scripts/experiment-apparatus/e171_three_oracle_collusion/collusion_report.py`（sha256 `6ad0d810…`）第 196 行 `reading_verdict`（有一行全放行 → `yes_…`；D 不够 → `not_enough_rows`；严格读法到不了 → `not_reported_f7_…`），第 207 行 `q1_lines`、第 273 行 `q3_line`；`collusion_matrix.py` 第 279 行 `coverage_enough_to_judge`、第 284 行 `strict_reading_is_unreachable`。
- **冻结层的 sha256 钉**：`.claude/kb/checks-owed.md` 第 592 行 C936（E171 冻结层按 sha256 钉住的编排器在 HEAD 上已对不上）——`rows_orchestrator_revision_8.py` 第 53 行钉的 `rows_orchestrator.py` sha256 与 HEAD 对不上、开跑先退 3，另几份冻结层的模块顶核对同红；第 594 行 C938（两格自证负载高时会抖）。
- **入库装置**：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`（12 972 行，sha256 `22400a1bfadca19c0f1caec20fbf38501495ab60a6dea84aedf3d1adc357e983`）第 17 行 `stream-constants [--streams 1,2,3]` 模式，第 211–227 行流三本地常量与出处注释（上一次登记修订 12、13 改过的那一段）。这一次不改它；`stream-constants` 在今天的主工作区上不过时照上一次登记修订 8 第 8.2 节分类办（S20）。
- **兼容旧版本的删法**：`records/2026-10-31-删兼容旧版本代码.md` 第 96 行（「延后（等实验执行员）」那一条点名两份名单第 119 行）与第 97 行（用户原话与本批派发）。

## 四、跑之前已经存在的数

设计时读到或自己算出的，照实列，不删。

1. **实验页标题行**（`grep -n '^#'` 命中，没打开正文）：第 1 行「第 1 次重跑第一段与第二段整表（修订 6 树）已跑（Q1 甲、Q2 乙、Q3 甲且分母 D = 39、Q4 基线甲与第 242、269 行乙；Q5 告警自证已过，判甲；……）」，第 856 行「整表已跑：Q2 乙，Q3 分母 D = 39」，以及「三条全放行的 7 行」「Q3 只靠一条的 11 行」「F4 新出的只靠一条的 12 行」「编不过的 6 行补位」「没触达却红的两行（第 155、161 行）与未触达行里交回值不同的七行」几个节名。影响：这些是撤项之前那一边的判定词与钉值；撤项核算从产物自己的行现算，不从这里抄；Q1 的门槛（判定词变不变）出自问题单候选的字面，不从这些数来。第 236 行在不在这些集合里，设计时不知道。
2. **Q2 设计时已经算出来了**（判可跑性时，命令在第十三节）：在今天的 `crates/mutations.tsv`（`5e6617b2…`）上，两份名单（逐字节相同）里按名字命中 0 次的有 6 项：第 33、70、107 行（117 行那张）与第 141、168、236 行（扩表）；其余 275 项各命中 1 次，没有命中 2 次以上的。按读法 3 的三类草算（拿 `0c7fdb82b` 那一版表当「r1 用的那张」的近似，r1 冻结树里那一份没核）：第 236 行甲（HEAD 里有、工作区没有）；第 70、141 行乙（只是「层 0」照 `.claude/kb/term-renames.md` 换成「崩溃点重放」，三样逐字相同；第 141 行今天另有一行「树分裂 核心规则：……」三样也相同，按改名挑出一行）；第 168 行乙（名字在 `0c7fdb82b` 之前就改了，三样与今天一行逐字相同、恰一行）；第 33 行丙（同一处锚点，今天那一行的替换文不同、名字从「最新根」改成「目标根」）；第 107 行丙（`0c7fdb82b` 那一版就没有，`4e8b03705` 那一版有，三样今天没有）。影响：Q2 的答案（有）跑之前就在；三类的判法是看到这几项之后写的，按「是不是同一个坏实现」的语义写，不按这几行凑；执行员在开工那一刻的表上照读法 5 重核、照读法 3 重分，与这里不同照实写。
3. **名单与原登记的数**：两份名单 117 + 164 = 281 行，两个目录逐字节相同（`04ea50f5…`、`f1bd082e…`）；第 236 行在扩表第 119 行。`replay.sh` 第 2751 行与第 2779 行注释「第 33、59、127、163、206、243 行会补位」（s4 树上编不过的六行，第 33 行在里面）。影响：第 33 行在原登记整表的读数里可能是补位行，撤项核算对它照 F12 的判法先核补位，不预设。
4. **上一次登记读到的数**：第四节（s4 挂钟、K = 234、D 40 → 45 等）、修订 8 第 8.9 节与修订 10 第 10.9 节（修订 8 那一趟推约 14–17 小时，每行约 200 秒）、修订 13（`stream-constants` 产物 83 行、sha256 `aef90046…`、重钉的两个下标）。影响：只进第十四节挂钟与第七节锚点；修订 8 那一趟到今天没有跑（实验页标题「修订 8 那一趟待拍树」）。
5. **代码里读到的事实**：读法 3 第 3 节那一处「按名字找不到不补位」；`segment_two_comparator.py` 的 `q3_denominator_class` 带 `row_numbers=`、`q2_set_row` 每行一行；`collusion_report.py` 的 `q1_*` 带 `row_numbers=` 与 `verdict=`。影响：撤项核算能从产物的行里减（第五节 5.3）；不补位这一处与原登记 5.2 那一句的出入不在这一次改（改了就与 r1、原登记不可比），列进待主 agent 认的几处。
6. **工作根与产物现状**：`~/.local/share/singlefs/work/` 下 E171 只剩 `e171-repin`（修订 13 的一份报告），r1 修订 6 那一趟的导出根已不在；`~/.local/share/singlefs/reuse/e171-r1` 在（修订 11 的复用库，不进 prune-server）；原登记的 `…-s4-f4-comparator.out`、`…-s4-f4-substitutes-comparator.out` 与修订 12 的 `stream-constants` 产物在 `59ce5a865` 归档，`git show 59ce5a865^:…` 取得到。影响：撤项核算只能读产物的行，不能读导出重算。
7. **`.claude/gate.d/mutation-duplicates-lag.tsv` 第 7–16 行**：十组重复变异行等「E171 下一次修订：名单去掉同一处改坏的多条、引到被删名字的改引保留名」；`.claude/term-rename-exempt` 第 89–94 行把四份名单按路径豁免术语改名（「装置与产物里印着的照原样留」）。影响：前者不在这一次的问题单里（今天那十组的名字都取得到，Q2 不管），不顺带做；后者是第 70、141 行今天取不到的来由，改引不是术语改名、是按名字取行的键，照读法 3 乙做，名单改名时这四行跟着改路径（第五节 5.4）。
8. **自己算的**：退役位 0 的三个标识符在 `crates/` 里 0 处；第 236 行的原文在今天的源文件里 0 次；撤项之后的项数，丙两行也撤时 117 行那张 115 行、扩表 163 行、合 278 行，只撤甲时 117、163、合 280 行。进第七节当锚点，不进门槛。

## 五、臂、阳性对照、真实基线

### 5.1 臂（每一行两边，另一边写成支持它的人认的样子）

| 行 | 一边 | 另一边 |
|---|---|---|
| Q1 | **甲**：撤项之后读法 1 那几份的判定词全同，变的只有钉值（逐格列出） | **乙**：至少一个判定词变了。支持乙的人会要求：撤项核算覆盖读法 1 的每一份（原登记 F4 扩表与补位那两份、r1 第二段），不只看 r1；每种读法、两个批各算一遍（第八节）；「判定词」照比对器自己的结论字现算，不由这一次另写一套判法（撤项核算 import 原比对器的判定函数，5.3）；补位可能被撤项牵动的行不悄悄当「只减」（F12）；改引与丙类撤项的行另报，不混进第 236 行的效应；现状整表段在今天的树上真跑，差出来的格逐行列出、不因「归因到树变了」而不报。支持甲的人会要求：计数与行号列表变了算钉值（问题单甲的字面），不升格成「结论变」；现状整表段里归因到树变了的格不算撤项的效应 |
| Q2 | **有**：至少一项（第 236 行之外）按名字取不到 | **没有**。支持「没有」的人会要求：取法与逐行驱动逐字相同（读法 5：跳过注释行、只收 6 列、名字逐字比），不做模糊匹配；用开工那一刻的主工作区表，不用 HEAD 或旧树的表；命中 2 次以上另列、不算「取不到」 |

**真实基线**：撤项之前的那几份读数本身（读法 1 列的产物原样）。撤项核算拿它们与「同一份读数减掉被撤的行」逐格比。现状整表段的真实基线是 r1 修订 6 那一趟（同上一次登记修订 8 第 8.5 节「两趟之间的对照」，r2 那一趟替修订 8 那一趟的位置）。

### 5.2 阳性对照（每一条臂都跑；比对器单测里手造，不碰真产物）

| 对照 | 怎么造 | 必须看到 |
|---|---|---|
| 乙臂·原登记一份 | 手造一份原登记比对器形态的产物：`q1_alpha` 只有第 236 行全放行（`all_quiet_rows=1 row_numbers=236 verdict=yes_a_row_is_quiet_on_every_oracle`），counted 40 行 | 撤第 236 行之后 `q1_alpha` 判定词变成 `no_such_row`，撤项核算报 Q1 乙、点名那一格 |
| 乙臂·r1 第二段一份 | 手造 `q2_set_row` 两行（第 236 行 `o1_red=false`、另一行 `o1_red=true`），原判定 `yi` | 撤之后 `q2=jia`，报乙 |
| 甲臂 | 手造同形产物，第 236 行在 counted 里、不在任何全放行与集合里、D 从 40 变 39 | 判定词全同，报甲，钉值格列出 `counted_D 40 → 39` |
| 干扰（F12） | 手造产物里第 235 行有一行 `name=substitution`（同文件 `system_configuration.rs`） | 撤项核算对第 235 行报「推不出」，Q1 那一份记不够判，不报甲乙 |
| Q2·有 | 手造一张 6 列表，少掉名单里的一个名字 | 名单核对列出恰那一项、命中 0 次 |
| Q2·没有 | 手造一张表，名单每个名字各一行 | 0 项取不到 |
| Q2·注释行 | 手造一张表，名单里的一个名字只出现在 `#` 起头的行里 | 那一项命中 0 次（与逐行驱动同读法） |
| 分类三类 | 手造：一项 HEAD 有、工作区无、记录里点名（甲）；一项三样逐字相同、名字只差一处登记改名（乙）；一项同锚点替换文不同（丙）；一项三样相同的两行、按改名挑不出（未分类） | 四项各归各类 |

### 5.3 装置写在哪（写死）

**装置一律走研究一侧** `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`（Python，不与 `crates/` 共用代码）：撤项核算与名单核对问的是「名单与已有读数之间的算术」，独立模型答得了。入库装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 不改、只照用：撤项核算段里只跑它的 `stream-constants --streams 1,2,3`（钉值重核）；现状整表段照上一次登记修订 8、10、11 跑它。它要读的 `crates/` 常量同上一次登记第五节 5.3 与修订 8 第 8.2 节，这一次不加；`stream-constants` 不过时照修订 8 第 8.2 节甲乙丙三类办（S20）。

新写（每份带单测，单测进下面那张变异表）：
- `list_check.py`：读法 5 的逐项核对、读法 3 的分类、改名单之后的核对（两个目录逐字节相同、行集等于两张表第一列之并、编号不重、除「未分类」外每一项在今天的表里恰命中 1 次）。读 `crates/mutations.tsv`、`git show HEAD:crates/mutations.tsv` 落到草稿目录的那一份、r1 用的那张表（读法 3 末条）、`.claude/kb/term-renames.md`。产物 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check.out`。
- `removal_effect.py`：读读法 1 那三份产物（原登记两份从 `git show 59ce5a865^:research/results/<名>` 落到草稿目录；r1 第二段那份在工作区），按被撤的行集 R 减，判定词用原比对器的函数现算——`collusion_report.reading_verdict`、`collusion_matrix.coverage_enough_to_judge`、`collusion_matrix.strict_reading_is_unreachable`（原目录）与 `segment_two_comparator.q2_verdict`（新目录），按路径 import、不另写。R 分两种各算一遍：只第 236 行（Q1 的判定用它）；甲丙两类全部（描述）。每份产物一份输出，名字见第十一节比对对象表。产物里每一格写撤之前、撤之后、变没变；被撤行在那一份里的类别与它在各列上的红照抄；F12 那一种行单列。
- 单测 `test_list_check.py`、`test_removal_effect.py`；变异表新开 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_list_removal.tsv`，经同目录 `mutation_driver.py` 跑（`--unit-test-module` 指这两份单测，照修订 1 第 1 行的做法）。

### 5.4 名单怎么改、文件名与引它的地方（问题单第 3 条）

1. **两份名单内容同改**：照读法 3 撤甲、丙两类的行，改引乙类的名字；两个目录改完逐字节相同（A21）。
2. **文件名去掉项数，按内容起名**：`bad-implementations-117.tsv` → `bad-implementations-step-8.tsv`（原登记 5.9 步长 8 挑出的那张），`bad-implementations-expansion-164.tsv` → `bad-implementations-step-4-only.tsv`（步长 4 独有的扩表）。项数是一条命令数得出的数（`wc -l`），不写进名字：今天就是名字里的 164、117 跟着撤项变成假话的情形，按项数改成 `-163`、`-115` 下一次撤项还会再变。两张表都改名（丙类两项在 117 行那张里）。
3. **引它的地方**：执行员开工时现列 `grep -rlE 'bad-implementations-(117|expansion-164)' --exclude-dir=__pycache__ scripts .claude crates`（2026-11-01 是 13 份脚本与表、`.claude/term-rename-exempt`、`.claude/gate.d/mutation-duplicates-lag.tsv`，另有三语实验页与变更史六份、记录三份），逐处照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「把这句话里的旧值换成新值，它还是不是真话」分：
   - **说现状的，改**：两个目录里代码的默认路径、`TABLE_FILES`、`SELECTED_ROW_TABLES`、`ROW_TABLES`、`INPUT_RESEARCH_FILES` 这类常量与 docstring、`replay.sh` 里 s4 那几条驱动（第 2759、2760、2772、2791、2792、2806 行起）的路径参数与第 2748 行注释、`.claude/term-rename-exempt` 第 89–94 行四条路径、实验页「装置、研究一侧与变异表」一节说名单住哪的句子（三语同改）。行集与项数常量（读法 7）改成从两张表现算、不写死：`TABLE_ROWS`、`WHOLE_TABLE_ROWS`、「恰是 1–281」那一核改成「等于两张表第一列之并、编号不重」，`count_reusable_units.py` 的 330 改成现算。
   - **说那一次的，不改**：上一次登记与更早的 `research/prompts/`、`research/results/` 下的产物、实验页里过去各趟的节、变更史已有的块、`records/` 里的记载、`mutation-duplicates-lag.tsv` 第 7–16 行理由列里的旧文件名（那几行等的是「E171 下一次修订」，不在这一次；行里的文件名改不改归那一次）。
   - **原登记挑法的事实不改**：原目录 `expansion_table.py` 第 27 行 `EXPECTED_EXPANSION_ROWS = 164`、`comparator.py` 第 106 行 `expected_step_4_only=164` 说的是原登记挑法在原表上挑出几行，不是今天名单有几行；`expansion_table_lines` 的 `rows=` 照实报 163。
   - **冻结层**（C936 点名的、按 sha256 钉自己或被钉的那几份）：改文件名与行集常量要连着重钉。撤项核算段不碰它们：只改名单文件本身与不在冻结层里的引用，冻结层里的旧文件名照留到现状整表段开头，与 C936 的重钉一起改（S22）。撤项核算段交回时列出冻结层里还指旧名的行（`grep -n`）。
4. **改的次序**：名单核对（改之前那一份产物）→ 改两份名单内容 → 改名（`git mv` 归主 agent；执行员在草稿副本里改好、交回补丁与 `mv` 命令）→ 改引用 → 名单核对（改之后那一份）→ 撤项核算。

### 5.5 谁跑什么

| 跑什么 | 谁 | 依据 |
|---|---|---|
| 两份新脚本与单测、名单与引用的改动（草稿副本里改，交补丁） | 执行员 | 研究一侧 Python 与文本 |
| `list_check.py`、`removal_effect.py` 真跑；两份单测；新变异表经 `mutation_driver.py` | 执行员写进「要主 agent 跑的命令」，主 agent 经统一任务服务交 | 子 agent 不交任务 |
| `stream-constants --streams 1,2,3` 在今天的主工作区上 | 主 agent 经服务交（编 checker 档装置二进制） | 跑实验装置 |
| 现状整表段（修订 8 那一趟的第 0–13 步与修订 10 第二段） | 主 agent 经服务交，多机 | 同上一次登记修订 8 第 8.4 节、修订 10 第 10.2 节 |
| `replay.sh` 改过的几条复跑行、`crates/mutations.tsv` 末尾「E171R1 装置」各行 | 重型，归提交时 | 同上一次登记 5.4 |

### 5.6 实现量与分段

**实现量**（推的，没量）：`list_check.py` 约 150 行、`removal_effect.py` 约 250 行（三种产物形态的读法各约 60 行，判定词全靠 import），两份单测合约 300 行，变异表 9 行；名单改两份、改名两份、引用约 20 处（第 5.4 节第 3 条）。不要另写检查器、触发式重算或崩溃枚举。现状整表段的实现量在上一次登记修订 8、10、11 里，这一次多的是 C936 的重钉与第 5.4 节第 3 条冻结层那几处。

**分段**（每段交回之后主 agent 对着问题单两行判续不续；一行开着的都不剩就停，后面的段不跑）：

| 段 | 做什么（按能最先让某一行够判的次序） | 补哪一行 | 补完够不够判 |
|---|---|---|---|
| 撤项核算段（第一段） | 名单核对（改之前）→ 改名单与引用 → 名单核对（改之后）→ 撤项核算三份 → `stream-constants` 钉值重核 → 现状·可取行（今天的树上 278 或 280 行逐行按名字取、核锚点，不编不跑） | Q2 全部；Q1 的「撤项本身改不改判定词」；Q1 够判条件里的「钉值重核」 | Q2 够判。Q1 撤项那一格够判（F11、F12 不触发时）；问题单 Q1 的够判条件字面还要「整张名单在主工作区现状上跑完、与 r1 逐格对比」，这一格这一段补不上 |
| 现状整表段（第二段） | 上一次登记修订 8 那一趟（第一段第 0–13 步）加修订 10 第二段、修订 11 复用层，名单换成这一次的、树在主 agent 宣布那一刻拍；前置是 C936 重钉与 S20 过 | Q1 够判条件「整张名单在主工作区现状上跑完，与 r1 的结论逐格对比」 | 够判（逐格并列、逐行归因）；不改撤项核算段判的判定词 |

撤项核算段交回时，Q1 若已由撤项那一格判定、而现状整表段要的十几小时只为补够判条件那一句，续不续由主 agent 定（第四节第 4 条：修订 8 那一趟本来就待拍树，它与现状整表段是同一趟，跑一趟两边都补）。

## 六、报哪些量与各自的判据

每个量各占一行，合取留给人做。问题单只有 Q1、Q2 两行；对不上任何一行的量不登记。

| 量 | 怎么算 | 门槛与判定 | 对应问题单哪一行、取什么值翻面 |
|---|---|---|---|
| Q2-逐项 | `list_check.py`，读法 5；两份名单各报：项数、命中 0 次的项（编号、文件、名字）、命中 2 次以上的项 | 第 236 行之外命中 0 次的项 ≥ 1 判「有」，= 0 判「没有」；两份名单的结果不同走作废 Z12 | Q2：第 236 行之外有一项命中 0 次即「有」 |
| Q2-分类 | 每一项取不到的，照读法 3 归甲、乙、丙或未分类，写出判它的那几样（HEAD 表里有没有、r1 那张表里那一行的三样、今天三样逐字相同的行数与名字、改名对照） | 描述量；未分类 ≥ 1 走 F13 | Q2（处置） |
| 名单改后 | 改之后两份名单各自的行数、sha256、第一列之并、撤掉的编号、改引的编号与新旧名；每一项在今天的表里的命中次数 | 两份逐字节相同；行集 = 改之前的行集减甲丙两类；改引的每一项恰命中 1 次；未分类之外没有命中不是 1 次的项。任一不过不往下跑撤项核算（Z12） | Q2 的处置做对了没有；Q1 的「撤掉什么」 |
| 撤项·原登记 | `removal_effect.py` 读 `…-s4-f4-comparator.out` 与 `…-s4-f4-substitutes-comparator.out`，R = {236}：读法 1 列的每一格撤之前、撤之后；R = 甲丙两类全部再算一遍 | 判定词有一格不同 → 这一份判乙；全同 → 甲，变了的钉值逐格列；F11、F12 触发的那一份记不够判 | Q1：任一判定词变即翻向乙 |
| 撤项·r1 第二段 | 同上，读 `e171-r1-…-2026-10-07-s2-comparator-r6.out`；`q2` 用 `q2_verdict(撤后集合行数, 撤后红行数)` 现算；四个 `d_*` 各减被撤行里属 counted 的个数 | 同上 | Q1：同上 |
| 撤项·r1 第一段 | 结构核对：`rows_orchestrator.py` 的 `STEP_ROWS` 与 `segment_one_comparator.py` 的 `Q1_ROWS`、`Q2_NAMED_ROWS`、`Q3_ROW`、`Q4_ROWS` 里有没有 R 里的编号；这 16 行（含空变异取的表第一行）的文件与 R 里各行的文件有没有相同的（补位只在同一文件组里换） | 只按 R = {236} 判（R = 甲丙全部时同文件已知，照 F14 末句只描述）：两样都没有 → 这一份判定词按结构不变，记甲；有一样 → 这一份记「要重算」，交主 agent（不在这一次的做法里） | Q1 |
| Q1-撤项（并列） | 上面三行各自的判定并成一张表，逐份逐格列，不在脚本里合成一个判定 | 描述量；合取由主 agent 写问题单状态时做，读法 2 给的读法是「任一份乙即乙；没有乙而有一份不够判即不够判；三份都甲即甲」 | Q1 |
| 撤项·干扰 | 读法 6：三份产物里，与 R 里任一行同文件的行有没有 `substitution`、`not_located_row`、`compile_failed` 或补位记载；列编号与记载原行 | 描述量；有的那一份走 F12 | Q1 前提 |
| 钉值·名单 | 读法 7 第二样：开工时 `grep` 现列的每一处写死的行集或项数，改成现算之后各自现算出的值（行集大小、单元总数） | 现算值 = 名单改后的行数（278 或 280，A21）；单元总数 = 第 0–10 步单元数 + 第一段行数 + 整表行数，与改之前那个式子同形；不等走 Z12 | Q1 够判条件「钉值重核」；不翻判定词 |
| 钉值·装置常量 | `stream-constants --streams 1,2,3` 在开工那一刻的主工作区上跑一次：`holds=false` 的行、`name=verdict s1_failed_checks=`、退出码、`E7RESULT` 行数；与修订 12 那份（`aef90046…`，83 行）`diff` | `holds=false` 0 行、`s1_failed_checks=0`、退 0 → 装置常量在今天的树上仍成立；不是的，照修订 8 第 8.2 节三类逐项归，走 S20 | Q1 够判条件「钉值重核」；不翻判定词；是现状整表段的前提 |
| 现状·可取行 | 名单改后的每一行在开工那一刻的主工作区上：按名字取几次、锚点在源文件里命中几次（`bad_implementation_table.anchor_hit_count`）；命中不是 1 的逐行列 | 描述量（不编不跑）；告诉主 agent 现状整表段里有多少行先天要落进读法 5 不进分母的那几类 | Q1 现状那一格（续不续现状整表段的依据） |
| 现状·逐格 | 现状整表段：上一次登记修订 8 第 8.5 节、修订 10 第 10.5 节那两份对照产物的每一格，r1 修订 6 那一趟对 r2 那一趟；每一格不同的，列出差出来的行，每行归一类：被撤（R）、改引（乙）、树变了（其余） | 描述量；不并进 Q1-撤项的判定；差只来自 R 的格另标 | Q1 够判条件「与 r1 的结论逐格对比」 |
| 附带（够判后不跑） | 原登记 s4 整表（117 行那张，没有第 236 行）、`mutation-duplicates-lag.tsv` 那十组的改引 | 照数报 | 不对应任何一行 |

**写结论时**：Q1 判甲只能写成「撤掉第 236 行之后，原登记 F4 扩表、补位与 r1 第二段这三份读数上的判定词都没变（变了的钉值是……）」，不写「E171 的结论在今天的实现上不变」——今天的实现上的读数只有现状整表段给得出。Q2 判「有」照列编号与类别。

## 七、钉绝对值的断言

每一条互比（撤之前对撤之后、改之前对改之后、r1 对 r2）旁边配一条绝对值：撤项核算与名单核对一起错了，下面这几条照样会红。

**出自被测条款或问题单本身**（不符时走「条款或实现可能错」——交主 agent，两边都查，不作废）：

| # | 断言 | 出处 |
|---|---|---|
| K1 | `crates/` 里读者没有认位 0 的路：`grep -rn 'INCOMPAT_RETIRED\|retired_layout_bit\|old_reader_supported_bits' crates/ \| wc -l` 为 0；第 236 行那一行的原文（取自 `git show HEAD:crates/mutations.tsv`）在今天的 `system_configuration.rs` 里命中 0 次 | D15 已定项 4「incompat 0：空出，可再分配」与「读者只收位 1、位 2 都带的那一种」；`records/2026-10-31-删兼容旧版本代码.md` 第 96 行「这个坏实现在新格式下没有对象」 |
| K2 | 第 236 行的名字在开工那一刻的主工作区表里命中 0 次 | 问题单 Q1 起因一段（「取不到」） |
| K3 | 第 236 行归甲类：名字在 HEAD 表里恰 1 次、工作区 0 次 | 同上；读法 3 甲 |

**独立算出、用命令核过的**（不符就作废那一格；命令在第十三节）：

| # | 断言 | 怎么算的 |
|---|---|---|
| A21 | 改之前：117 行那张 117 行、扩表 164 行，两个目录各自 sha256 `04ea50f5b0c6817f09d69d5e9bddabd58c11488dcbad06088930cf7e39e2fb63`、`f1bd082e0dc933d798647e1aef761a8c5219a5eb527cf9fd627c17f5d3e6cd04`；扩表第 119 行第一列是 236。改之后：两个目录逐字节相同；甲丙两类都撤时 115 + 163 = 278 行、行集 = {1, …, 281} 减 {33, 107, 236}；只撤甲时 117 + 163 = 280 行（第四节第 2 条的分类在开工那一刻没变时） | `wc -l`、`sha256sum`、`sed -n 119p`、`cut -f1 \| sort -n` 与 `seq 1 281` 比 |
| A22 | 比对对象各份的 sha256 与行数（第十一节那张表）开工时、撤项核算读之前各核一次 | `sha256sum`、`wc -l`；归档的从 `git show 59ce5a865^:research/results/<名>` 取 |
| A23 | r1 第一段的行集是 {0, 15, 23, 42, 49, 110, 125, 155, 161, 162, 174, 242, 252, 269, 270, 271}（`rows_orchestrator.py` 第 155 行 `STEP_ROWS` 之并），不含 33、70、107、141、168、236。R = {236} 时：名单里 `crates/singlefs-core/src/system_configuration.rs` 的行是第 85、86、87、235、236、237 行，r1 第一段的 16 行（空变异取的第 1 行是 `admission.rs`）一行都不在这个文件里。R = 甲丙全部时：第 33 行（`mount.rs`）与第 107 行（`transaction.rs`）同 r1 第一段第 42、49、155、161、162、174 行与第 242、252、269、270、271 行同文件，这一种只描述、照 F14 末句办 | `awk -F'\t' '$3=="…"'` 按文件取名单行号；`grep -n STEP_ROWS` |
| A24 | 名单核对在 `5e6617b2…` 那张表上的读数：命中 0 次的恰是第 33、70、107、141、168、236 行，命中 2 次以上的 0 项；开工那一刻的表 sha256 不同时，在那张表上重算，逐项与这里比、不同照实写（不作废，表变了是事实） | 草稿脚本 `awk` 那一条（第十三节） |
| A25 | 撤项核算的自检：每一份产物、每种读法与批，撤之前各类行数之和 = 那一份的总行数；撤之后各类行数之和 = 撤之前 − R 里在那一份出现的行数；撤之后的每个行号列表 ⊆ 撤之前的 | 脚本里两份各算，单测钉 |
| A26 | `stream-constants` 那一趟 `E7RESULT` 行数等于 `name=done emitted=` 的值；修订 12 那一份是 83 行 | `grep -c '^E7RESULT '` |

## 八、轨迹与几何敏感性

**轨迹**：撤项核算与名单核对没有轮次、没有段，读的是已落盘读数的行集合，被谓词消费的是「这一格的判定词」——峰值、为正轮数、期末值三样不适用。现状整表段被谓词消费的量与它们的轨迹，同上一次登记第八节（逐段红态数的峰值、为正段数、期末值）。

**几何敏感性**（`.claude/rules/mutation-sampling.md` 第六类）：撤项效应在今天的树上量不到——今天的树上第 236 行没有对象，「撤之前」那一边跑不出来；方向相反的取样点只能在已有读数的旋钮上取：

| 判决 | 主取样点 | 方向相反的取样点 | 怎么报 |
|---|---|---|---|
| 原登记那两份的判定词 | 批 1（`q1_alpha` 等的 `verdict=`） | 批 2（同一份里的 `batch_2_all_quiet_rows` 与 `rows_that_differ_from_batch_1`） | 撤之后批 2 上的全放行行集是否同批 1 一起变；一批变、一批不变记「不稳定」并列，不变记两点 |
| r1 第二段的 D | 读法 `registration`、批 1 | 读法 `original_reading_5`、批 2（四格全报） | 四格各报撤前撤后；第 236 行在四格里的类别不同照列 |
| 撤的行集 | R = {236} | R = 甲丙两类全部 | 两种各算一遍；判定词只在大的 R 上变的，标「不是第 236 行的效应」 |
| 树 | 原登记与 r1 修订 6 那几棵旧树（撤项核算） | 今天的树（现状整表段） | 不合并，逐格并列；今天的树上差出来的格按「现状·逐格」归因 |

**判别力自证**（`test_removal_effect.py`）：把「判定词变才算乙」挪成「钉值变也算乙」，阳性对照「甲臂」那一份必须由甲转乙；把 F12 的同文件判法挪成「同一行号」，阳性对照「干扰」那一份必须由不够判转甲。

## 九、变异

研究一侧的行进新表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_list_removal.tsv`，经 `mutation_driver.py` 跑（单测模块 `test_list_check`、`test_removal_effect`）。每条写明在哪个取样点上改变输出：

| # | 改哪 | 改成什么 | 在哪个取样点上改变输出（会红的那一格） |
|---|---|---|---|
| M1 | `removal_effect.py` 从 Q2 集合里减 R | 不减 | 阳性对照「乙臂·r1 第二段」：原式 `q2=jia`，变异仍 `yi` |
| M2 | 从 counted 里减 R | 不减 | 阳性对照「甲臂」：原式 `counted_D 40 → 39`，变异 40 → 40 |
| M3 | 按行号减 | 减 `number - 1`（差一） | 手造产物第 235、236 行在不同类：原式第 236 行那一类少一，变异第 235 行那一类少一（A25 第三条也红） |
| M4 | 判定词现算 | 用撤之前的行数代进 `reading_verdict` / `q2_verdict` | 阳性对照「乙臂·原登记」：原式 `no_such_row`，变异仍 `yes_…` |
| M5 | 干扰判法 | 不看 `name=substitution` 行 | 阳性对照「干扰」：原式那一份不够判，变异报甲 |
| M6 | `list_check.py` 读表 | 不跳过 `#` 起头的行 | 阳性对照「Q2·注释行」：原式命中 0，变异 1 |
| M7 | 分类乙的判法 | 只比（文件，原文），不比替换文 | 阳性对照「分类三类」里的丙那一项：原式丙，变异乙；真表上第 33 行同样翻 |
| M8 | 分类乙的改名对照 | 不读 `.claude/kb/term-renames.md` | 阳性对照「分类三类」里的未分类那一项不变，而「三样相同的两行里有一行是改名后名字」那一项（真表第 141 行的形态）原式乙、变异未分类 |
| M9 | 改后核对的行集 | 写死成 1–281 | 「名单改后」那一行：原式行集 = 改之前减甲丙、过，变异报行集不等、不过 |

变异全抓才算这套核算有判别力；没红的照 `.claude/rules/mutation-sampling.md`「没红的三类」分；无效的照第八类另列、换一条补上。入库装置不改，不加装置一侧的变异行。

## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F11 | 撤项核算读不到要的行：那一份记「不够判」并写缺哪一种行，不判甲乙；别的份照判 | 那一份产物里没有读法 1 点名的某一种行；或 R 里的某个编号在那一份的逐行读数（`name=row` / 类别行号列表）里一处都没有 |
| F12 | 补位可能被撤项牵动：与 R 同文件、在那一趟里有过补位或定位失败的行，撤项之后的读数推不出；那一份记「不够判」并列行号，不按「只减」判 | 「撤项·干扰」那一行在那一份里列出至少一行 |
| F13 | 名单里有取不到又分不了类的项：那一项不改，交主 agent；别的项照改 | 「Q2-分类」报未分类 ≥ 1 |
| F14 | 结构核对说 r1 第一段要重算：r1 第一段那一份记「要重算」、不判；Q1 并列表里写明。R = 甲丙全部时同文件是已知的（A23），那一种不触发这一条，只在描述里写「r1 第一段那几行在 r1 里有没有补位，要读 r1 第一段比对器的补位行才知道，这一次没读」 | R = {236} 时 A23 不成立（第 236 行在 r1 第一段的行集里，或 r1 第一段有一行的文件是 `system_configuration.rs`） |

## 十一、作废条款与停机条款

**作废**（那一次的输出不算结果；修好之后重跑，不改判据）：

| # | 条款 | 触发观测 |
|---|---|---|
| Z11 | 撤项核算读的不是登记的那一份 | 比对对象表里那一行的 sha256 或行数，与撤项核算读之前现算的不同（A22） |
| Z12 | 名单没改对 | 「名单改后」或「钉值·名单」那一行不过：两个目录的名单不逐字节相同，行集不等于改之前减甲丙两类，改引的项命中不是 1 次，或代码里现算的行集大小与名单行数不等。这期间出的撤项核算与现状整表段产物全部作废 |
| Z13 | 现状整表段的作废 | 同上一次登记 Z1–Z7（Z5 含树里 `crates/mutations.tsv` 等于 H8），「登记的那棵冻结树」读作 r2 那一趟拍的树 |

**停机**（装置与 `crates/` 对不上，或拿来比的东西与登记的不是同一个：两边都查，既不作废也不当结果，交主 agent）：

| # | 条款 | 触发观测 |
|---|---|---|
| S20 | 装置本地常量在今天的树上不成立 | `stream-constants --streams 1,2,3` 编不过、退非 0、`holds=false` ≥ 1 行或 `s1_failed_checks` ≠ 0。照上一次登记修订 8 第 8.2 节逐项归类：甲类有一项不过，停，两边都查（`.claude/rules/implementation-first.md` 第 4 条）；只有乙类不过，写一条登记修订重钉（只改值与出处注释，主 agent 用 `scripts/hosts/compile-then-swap.py` 换进主工作区），重钉前现状整表段不起。撤项核算的判定不受它影响 |
| S21 | 名单核对与改名单用的不是同一张变异表 | 改之前核对、改名单、改之后核对三处记的 `crates/mutations.tsv` sha256 不全相同 |
| S22 | 现状整表段的前置没齐 | C936 没关（冻结层按 sha256 的核对仍退 3），或冻结层里仍有指旧文件名、写死 1–281 的行（撤项核算段交回列的那几行），或 S20 没过 |
| S23 | 退役位 0 的路在实现里还在 | K1 不成立：三个标识符有命中，或第 236 行的原文在今天的源文件里命中 ≥ 1 次（实现与 D15 已定项 4、删兼容第 4 批的记载对不上） |

现状整表段另沿用上一次登记的停机 S1–S3、S5–S13、S16、S17（S16 的「宣布行」写进本批调度记录 `records/2026-10-31-删兼容旧版本代码.md`，行首「- E171 第 2 次重跑拍树宣布（<日期>）：」，判法里读宣布行的那一处照改，写进第十二节修订；修订 8 第 8.1 节「三条线各自最后合入的批号」那一项不适用，改写 HEAD 与 `git status --porcelain -- crates Cargo.toml Cargo.lock .cargo` 全文）。

**够判停机**：每段交回附问题单两行的表（已够判 / 还差什么 / 剩下的量能不能让它翻面）。撤项核算段交回之后：Q2 应够判；Q1 撤项那一格够判（F11、F12、F14 不触发时），问题单字面的够判条件还差现状整表段那一格。主 agent 定不续现状整表段的，「现状·逐格」逐条标「够判后未跑」或「主 agent 定不跑」（照实写哪一种）。附带量一律够判后不跑。

**比对对象**（这一次要与之比的上一次产物；E171 没有正式跑单元表）：

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out` | `e8660ccef12a13f76409a1b6f24ebd9a75444249ad65d5617551ac59c1e3c40e` | 4698 | `replay.sh` 第 1825 行（`driver_e171_s4_f4_comparator`，在今天的 crates 上整条重出，不是这一次的比法）；工作区已没有这份，取 `git show 59ce5a865^:research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`；这一次只读它的行 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4.out` | 撤项·原登记（F4 扩表） |
| 2 | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` | `0f439cc2c86631a8d4c47a1f5da3205920b3f2bdb6e38684981a0f5b5cb5592a` | 4795 | `replay.sh` 第 1826 行（`driver_e171_s4_f4_substitutes_comparator`，同上）；取 `git show 59ce5a865^:research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4-substitutes.out` | 撤项·原登记（加补位） |
| 3 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out` | `d0bc79ca3f580774cfd4bad2fd672006b98c8a5d02063c48fb9915cb246a8549` | 451 | `replay.sh` 第 1830 行（`driver_e171_r1_s2_revision_6_comparator`；导出根已不在，复跑报本次未跑）；撤项核算只读工作区里这一份的行 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-revision-6-segment-two.out` | 撤项·r1 第二段 |
| 4 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12.out` | `aef90046e0b565776288e5051c7267f706db4ee718bfceeb66dfb0272dfdd2a1` | 83 | `replay.sh` 第 1835 行（`driver_e171_r1_revision_12_stream_constants`）；取 `git show 59ce5a865^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12.out` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants.out` | 钉值·装置常量；`diff` 比 |
| 5 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out` | `40ee800a8681caf503c05131fbd05fcdba7a9514a38178660c55195330b1cabc` | 811 | `replay.sh` 第 1827 行（`driver_e171_r1_s1_revision_6_comparator`）；现状整表段照上一次登记修订 8 第 8.5 节的对照脚本比 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-s1-comparator.out` | 现状·逐格（r1 第一段那一半） |
| 6 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out` | `d0bc79ca3f580774cfd4bad2fd672006b98c8a5d02063c48fb9915cb246a8549` | 451 | `replay.sh` 第 1830 行；现状整表段照上一次登记修订 10 第 10.5 节的对照脚本比 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-s2-comparator.out` | 现状·逐格（r1 第二段那一半） |
| 7 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out` | `59c89b96c5bef3151c8b4885d98f8fe4db680a300106a9de400af4188cf74e7c` | 565 | `replay.sh` 里没有它的行（r1 第二段编排层合成的逐行驱动产物，比对器的输入）；现状整表段的同名产物与它逐行比行集与各行类别 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-s2-rows-driver.out` | 现状·逐格（逐行归因） |

重跑产物名里的日期是写登记这一天；落产物那一天不同的，执行员在第十二节记新名，`.claude/gate.d/replay-comparison-units.tsv` 照新名登记。E171 没有正式跑单元表，七行在那份表里都只能写成「# 不登记的比对对象：research/prompts/e171-r2-prereg.md #<号>：<理由>」，理由要点名 `.claude/kb/checks-owed.md` 开着的一笔欠账；今天没有一笔欠账管「E171 没有比对单元表」（C930 管的是 E172），要主 agent 先开一笔。登记之前门禁 doc-experiments 的 replay-comparison-units 格判这七行红。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。读法 3 末条「r1 用的那张表」的取法、开工那一刻的表 sha256、改名用的补丁与 `mv` 命令、落产物那一天的新产物名、现状整表段续跑时的宣布行与跑法改动，抄在这里。）

### 修订 1（2026-11-01，执行员；装置与单测写完、单测与产物都还没跑）

依据：单测还没跑（子 agent 不交任务，单测与变异的命令交主 agent），这一条修订只记登记交给执行员定的取法与实现上的落点，不动判据；第六、七、九节每一条的去留与报告方式照原样。主 agent 认的几项照派发提示（批 12 的派发）「第 33、107 行撤、不换；第 70、141、168 行改引到改名之后的那一行；第 236 行撤；「结论变」= 判定词变；两张名单去掉文件名里的项数，按内容起名」。

1.1 **开工那一刻**：`git rev-parse HEAD` = `59ce5a86550de2c2c9bc6b51b34b4f881313682c`（名单核对 `--reference-commit` 取它：改之前的名单与「HEAD 那一版变异表」都从这个提交取，之后主 agent 提交了也不变）。工作区 `crates/mutations.tsv` sha256 `437f3351595780e18145a2c24014060d6fb36895cb7f5757d777075419496c3d`（量的，与设计时的 `5e6617b2…` 不同，A24 照「在那张表上重算」）；草稿 `awk` 在这张表上重算，命中 0 次的仍是第 33、70、107、141、168、236 行、命中 2 次以上 0 项（量的，草稿目录 `compat-removal-e171-r2-run`）。改名单用的补丁就是对着这张表造的；名单核对产物 config 行记跑那一刻的表 sha256，与这里不同走 S21。

1.2 **读法 3 末条「r1 用的那张表那一行」的取法**（那棵冻结树不在工作区）：r1 第二段逐行驱动产物（`e171-r1-…-2026-10-07-s2-rows-driver-r6.out`）里，这一行没补位、照名字取到并改坏的（`substituted_from=-` 且 `mutated_row_name` 等于 `row_name`），对照行取它记的 `mutation_line_sha256` 在变异表历史各版（`git log <开工提交> -- crates/mutations.tsv` 的每一版）里那一行（第 70、141、236 行是这一种）；别的（第 107、168 行 r1 里 `not_found`，第 33 行 r1 里编不过换了补位、`line_matches_the_registration=false`，r1 那一行的行文取不回）取登记行：名字相同、行文 sha256 前 12 位等于名单第 6 列的那一版那一行。三样（文件，原文，替换文）照对照行比。实现在 `list_check.py` 的 `reference_line`。

1.3 **甲类「记录里写着」的判法**：名字最后一个「：」之后那一截原样出现在 `records/2026-10-31-删兼容旧版本代码.md` 里（第 236 行那一截「读者仍把退役的位 0 当受支持的布局身份（旧镜像挂得上）」在记录第 96 行）。

1.4 **乙类「按改名挑一行」的改名**：照 `.claude/kb/term-renames.md` 登记表逐行换（整串照字面、词边界加 `\b`、正则照原式），切格照 `scripts/kb/sweep-term.py` 的 `table_cells`。

1.5 **改名与补丁**：`git mv` 四条（两个目录各两张：`bad-implementations-117.tsv` → `bad-implementations-step-8.tsv`，`bad-implementations-expansion-164.tsv` → `bad-implementations-step-4-only.tsv`）之后打补丁 `~/.local/share/singlefs/work/compat-removal-e171-r2-run/e171-r2-lists-and-references.patch`（sha256 `34f7a7f4b093fca86d472aa08542dc56630dfef27abd86664d0176ad21714f73`，`patch -p1`，12 个文件）：四张名单撤第 33、107、236 行、第 70、141、168 行第 5 列换成今天三样相同的那一行的名字；两个目录的 `row_driver.py`、原目录 `comparator.py` 与 `expansion_table.py` 的默认路径与说现状的说明；`segment_two_comparator.py` 的 `TABLE_ROWS` 与 `count_reusable_units.py` 的 `WHOLE_TABLE_ROWS` 改成 `list_check.whole_table_row_numbers()` 现算；`test_segment_two.py` 那一格钉的 `table_rows` 改成 278；`replay.sh` 第 2748 行注释与 s4 那几条驱动的名单路径。冻结层（第 5.4 节第 3 条）这一段不碰，旧名与写死的 1–281 留到现状整表段开头与 C936 的重钉一起改。

1.6 **第 1 次重跑那一份比对器产物的复跑照旧逐字节可比**：`segment_two_comparator.py` 加 `--table-rows`（默认照改后的名单现算），`replay.sh` 里 `driver_e171_r1_s2_revision_6_comparator` 带 `--table-rows 281`。只是实现上的落点，不动判据。

1.7 **补两条变异（收严）**：第八节的两条判别力自证写成变异表的 M10（钉值变也算乙）、M11（干扰按同一行号而不按同文件），与 M1–M9 同表跑。

1.8 **取样点**：阳性对照「Q2·注释行」的名字本身以 `#` 起头（`#甲名`），跳不跳过 `#` 行在这一格上结局才不同（M6 才会红）；名字不以 `#` 起头时 `#` 那一行第 1 列是 `#甲名`，跳不跳过都取不到。

1.9 **「撤项·r1 第一段」写在哪**：它不读产物，结构核对的行写进 r1 第二段那一份撤项核算产物（`removal_effect.py --segment-one-structure`，`name=segment_one_structure`），判定照第六节那一行。

1.10 **F11 补一种触发（收严）**：被撤的行在那一份里是计入行、而 `q1_o3_coverage` 的 `rows_with_no_strict_judgement` 大于 0 时，撤后那一格推不出（产物里没有逐行的严格覆盖数），那一格记 `unknown`、那一份记不够判。

1.11 **读法 6「有过补位或定位失败」的认法**：原登记那两份认 `name=row` 的类别是编不过、找不到、锚点不唯一、名字不唯一，或 `substituted_from` 不是 `-`，或有 `name=substitution` 行；r1 第二段认 `name=row` 的 `status` 不是 `ok`、`substituted_from` 不是 `-`，或有 `name=substitution`、`name=not_located_row` 行。

1.12 **产物名**：落产物那一天与写登记同是 2026-11-01，第十一节比对对象表里的重跑产物名照用；名单核对产物 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check.out`；变异产物另起 `…-2026-11-01-list-removal-research-mutations.out`。主 agent 哪天跑与 2026-11-01 不同，承接的执行员在这里补一条新名。

### 修订 2（2026-11-01，承接的执行员；撤项核算段的产物已落盘，现状整表段一份产物都没跑）

依据：第十一节 S20 原文「只有乙类不过，写一条登记修订重钉（只改值与出处注释，主 agent 用 `scripts/hosts/compile-then-swap.py` 换进主工作区），重钉前现状整表段不起」；派发提示（批 12 续派）「stream-constants 五项照上一次登记修订 8 第 8.2 节分类（S20），C936 照登记重钉」。这一条只记 S20 要的重钉与撤项核算段产物落盘之后查出来的几件事实；第一至十一节、修订 1 的判据一字不改，第六、七、九节每一条的去留与报告方式照原样。

2.1 **S20 读数**（量的，产物 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants.out`，83 行，sha256 `3b87b43a655b26e398d0c73183bf7a6f757b38749b67a01126a8e6b79d96c53c`）：第 82 行 `E7RESULT name=verdict s1_failed_checks=19`，`holds=false` 19 行（派发提示写的「五项」是前五行，产物现数 19）。19 行逐项按上一次登记修订 8 第 8.2 节归类，全是乙：三条流的段长表、写数、全量 / 快档 / 闭式态数（流一第 36、37、44、45、46 行，流二第 55、56、60、61、62 行，流三第 72、73、77、78、79 行），Q5 那一形删的屏障下标、改成普通写的根槽写下标、两个截断点（第 25–28 行）。甲类各项 `holds=true`：流三拆发布 `third_stream_split_publish`（第 68 行）、三条流根槽写数（第 38、57、74 行）、三条流分配器带 mkfs 的根环表（第 33、49、65 行）、流一 `new_pool_file_creation_operations_equal_build_pool`（第 41 行）、对拍末段属于第 3 次调用的写数 6 与三对镜像（`equivalence_pairing_in_flight_writes` 第 16 行、`equivalence_pairing_mirrored_pairs` 第 18 行）；Q5 那一形的已确认调用与在途调用（第 2、3 次）是装置常量 `CONFIRMED_LOSS_CONFIRMED_CALL`、`CONFIRMED_LOSS_IN_FLIGHT_CALL`，`stream-constants` 不出它们的行，没核。乙类里没变的：K 的取法数 `confirmed_loss_key_choices`（第 19 行，72）、C 的写数、类数、类的态数、照原路径判的态数、末段写数与可撕裂写数、全枚举态数（第 20–24、29–32 行）都 `holds=true`；丙类由它们推出，所以对拍态数、阳性对照「判态路 panic 单列」三件的态数不变。S20 判「只有乙类不过」，照原文重钉。

2.2 **新钉值**（取上面那份产物各行 `crates=` 一栏，旧值是 `local=` 一栏）：流一段长 11 段 → 12 段（下标 2 多一段 2 写）、写数 41 → 43、全量 16 777 260 → 16 777 268、快档 46 → 54、闭式 16 777 240 → 16 777 243；流二段长 78 段 → 80 段（下标 2、17 各多一段 2 写）、写数 481 → 485、全量 1 678 377 204 → 1 678 377 220、快档 278 → 294、闭式 1 678 377 089 → 1 678 377 095；流三段长 959 段 → 960 段（下标 2 多一段 2 写，其余后移一位）、写数 5980 → 5982、全量 362 809 947 → 362 809 955、快档 3127 → 3135、闭式 362 808 742 → 362 808 745；Q5 删的屏障 (50, 51, 55, 56) → (54, 55, 59, 60)、根槽写 52 → 56、截断 81 → 85、反向截断 57 → 61。为什么变（推的，读 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 815–826、861–866 行的注释与断言，没在装置上另核）：今天的 crates 取号写两轮（D22（单元原子性怎么合成） 已定项 8 落地，每轮两盘各一写、轮后一道池屏障），每次取号多一段 2 写；流二取号两次，所以多两段；Q5 那一形的四个下标都在第二次取号之后，后推 4（两写两屏障）。三段新表各等于旧表在上述下标插进一个 2，闭式按新表 1 + Σ(2^n − 1) 现算与 `crates=` 闭式相等（草稿脚本 `tools/new_values2.py` 的输出，第十三节之外的草稿读数）。

2.3 **改在哪**：只改 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 第 165–298 行那一段本地常量的值与出处注释，加内联单测 `local_segment_tables_reproduce_the_closed_forms_and_write_counts_copied_from_the_tests` 末两行钉的两个闭式（它们是同一组乙类常量的绝对值钉，不改就编过而单测红）。草稿副本在 `~/.local/share/singlefs/work/compat-removal-e171-r2-run-b/repin/`：改前 sha256 `22400a1bfadca19c0f1caec20fbf38501495ab60a6dea84aedf3d1adc357e983`（与第三节记的一致），改后 `173360b407d762356eb13b171987e8533fa1da4a230ab8663ac1355e99c45fb8`（12 977 行，`rustfmt --check` 过）；草稿脚本 `tools/check_repin.py` 逐个比 22 个常量与产物 `crates=` 一栏，改前 `pairs 22 differs 19`、改后 `pairs 22 differs 0`。换进主工作区由主 agent 照 S20 跑 `compile-then-swap.py`。

2.4 **重钉之后的判法**：换上之后在主工作区再跑一次 `stream-constants --streams 1,2,3`，产物名 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants-r2.out`（跑的那天不是 2026-11-01 的，照第 1.12 条换日期）：`grep -c 'holds=false'` 为 0、`name=verdict s1_failed_checks=0`、`grep -c '^E7RESULT '` 等于 `name=done emitted=` 的值（A26）、命令退 0，S20 才算过；与 2026-11-01 那一份逐行比，只许上面 19 行与第 82 行不同。拍树宣布之后、拍树之前，照上一次登记修订 8 第 8.2 节「怎么取」在宣布那一刻的草稿副本上再跑一次；crates 在这之间又变了的，照 S20 再归类，不沿用这一次的钉值。

2.5 **Z12 触发（量的）**：名单核对产物第 40 行 `E7RESULT name=verdict q2=yes directories_agree=true unclassified=0 after_lists_passed=true pinned_values_equal=false mutation_table_unchanged_during_the_run=true`，第 36 行 `count_reusable_units_whole_table_rows=import_failed:AttributeError`。原因（读代码，量的是文件内容）：`count_reusable_units.py` 模块顶 `import reuse_library`，`reuse_library.py`（工作区未暂存的别的会话改动）第 122–131 行模块载入时调 `.claude/hooks/lib_work_root.py` 的 `shared_data_path`，那份库在 HEAD、暂存区、工作区里都没有这个函数（`grep -c shared` 暂存区那一版为 0）；不是这一批补丁改的。照第十一节 Z12 原文「「钉值·名单」那一行不过 … 这期间出的撤项核算与现状整表段产物全部作废」：撤项核算三份产物（2026-11-01）作废，判定词照读、不当结果；`reuse_library.py` 那一处改好之后重跑名单核对（`pinned_values_equal=true` 才算过），再重跑撤项核算三份。重跑的产物名在旧名去掉 `.out` 之后接跑的那天的日期（例 `…-2026-11-01-list-check-2026-11-02.out`；同一天再跑加 `-r2`），旧的留着。撤项核算的算法不读名单核对的判决行，重跑预计判定词与这一次相同（推的），config 行的名单核对产物名与 sha256 会变。

2.6 **比对器两道检查的次序**：`segment_one_comparator.py` 的 `main` 改成先核「输入是不是这一趟的」（`inputs_outside_this_run`，按路径算、不碰盘），再核在不在（`report_missing_inputs`）。依据上一次登记修订 8 第 8.5 节第 1 条「给到另一个前缀就退出」与其后主 agent 认定「对不上退 2」：接错了另一趟的输入，不管在不在都退 2，不落进本次未跑的 77。只是实现上的次序，判据不动；`segment_two_comparator.py` 的同一处次序这一条没改。

2.7 **两份修订 11 入口的仓根**：`run_segment_one_revision_11_multi_host.sh`、`run_segment_two_revision_11_multi_host.sh` 文件头写死的仓根改成取脚本所在的树（`$(cd "$(dirname "$0")/../../.." && pwd)`），派到第二台也指自己那棵树。不在冻结层里（两份都没有被别的文件按 sha256 钉）。

2.8 **C936 与冻结层没做（S22 照旧不过）**：按 sha256 现量（工作区，含别的会话已暂存的改动），冻结层的钉值里只有 `rows_orchestrator.py` 对不上（此刻 `57a48df1…`，钉值 `2ab19c53…` 写在 `cross_host_o3_check.py` 第 56 行、`rows_orchestrator_revision_8.py` 第 59 行、`rows_orchestrator_segment_two.py` 第 106 行、`reuse_library.py` 第 454 行），另五个钉值（`cross_host_o3_check.py` `6476110d…`、`rows_orchestrator_segment_two.py` `2b91b18a…`、`rows_orchestrator_revision_8.py` `db66885c…`、`rows_orchestrator_segment_two_revision_10.py` `57a7d2ca…`、`segment_one_units_revision_8.py` `abf6a8c6…`）与此刻内容相同。重钉与冻结层里旧文件名、写死 1–281 的那几处要连着改，改了的冻结文件 sha256 又被别的冻结文件与 `reuse_library.py` 钉着，一串里必有 `reuse_library.py`（第 144 行 `SELECTED_ROW_TABLES` 与第 454、455、457 行三个钉值）；那份文件别的会话正在改、这一段不碰（派发提示），所以这一串整个留给 `reuse_library.py` 那件落地之后的下一段，S22 不解、现状整表段不起。

### 修订 3（2026-11-02，承接的执行员；撤项核算段的 Z12 重跑已落盘，现状整表段一份产物都没跑）

依据：派发提示（批 14）「4. 登记修订 3（接在第十二节修订 2 之后）：写进〔别的会话〕那一串改动（编排器改走分片协议、跨机对照不经服务时报 not_run、冻结层重钉）对登记哪几条的影响；两份入口 … 判完成看哪几行、产物名、`PLAN_ONLY` 输出怎么读、`revision_11_placeholders.json` 占位怎么处置；入口里的 `nice -n 19` 包装照「资源由服务定」去掉」（〔〕里原文是会话号，换成了说法）与「3. `segment_two_comparator.py` 第 469、472 行「先缺失、后这一趟」的次序：照登记修订 2 第 2.6 条与 `segment_one_comparator.py` 一样换」。读数依据：Z12 重跑四份产物（第 3.1 条）；冻结层自证八份退 0（主 agent 在派发提示里转述，没给任务号）；第二段比对器单测与重跑交主 agent，还没有读数。第一至十一节、修订 1、2 的判据一字不改，第六、七、九节每一条的去留与报告方式照原样；第 3.4 条是唯一的收严：把「读不到 ≠ 读到 0」照字面用到上一次登记的 S17 上。

3.1 **Z12 重跑（量的）**：名单核对 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check-2026-11-01.out`（41 行，sha256 `e7d8caee76e35bbf4e955a2a7dad0aed1fa2a68e962f728e77a8147a6abdaa81`）第 36 行 `E7RESULT name=pinned_values list_rows=278 segment_two_comparator_table_rows=278 count_reusable_units_whole_table_rows=278 equal=true`、第 40 行 `E7RESULT name=verdict q2=yes directories_agree=true unclassified=0 after_lists_passed=true pinned_values_equal=true mutation_table_unchanged_during_the_run=true`；去掉第 1 行（config）之后与作废那一份只差这两行。Z12 不再触发。三份撤项核算 `…-2026-11-01-removal-effect-on-the-s4-f4-2026-11-01.out`（135 行）、`…-removal-effect-on-the-s4-f4-substitutes-2026-11-01.out`（135 行）、`…-removal-effect-on-the-revision-6-segment-two-2026-11-01.out`（159 行）去掉第 1 行之后与作废那三份逐行相同（`diff` 0 行）；config 行点名新的名单核对产物与它的 sha256。产物名照修订 2 第 2.5 条；`replay.sh` 四条登记行已改指新名，作废那四份留着。Q1 撤项那一格：三份 R = {236} 都 `q1_judgement=jia`，F11、F12（R = {236} 那一种）、F14 不触发，够判（甲）；问题单 Q1 够判条件字面那一句仍差现状整表段。

3.2 **第二段比对器两道检查的次序（同修订 2 第 2.6 条那一处）**：`segment_two_comparator.py` 的 `main` 先核 `inputs_outside_this_run`（按路径，不碰盘）、再核 `segment_one.report_missing_inputs`，依据同第 2.6 条。用例 `test_segment_two.py` 的 `test_an_input_of_another_run_is_refused_before_the_missing_check_and_nothing_is_written`：另一趟的基线目录不在盘上，要退 2、不写产物；旧次序退 77。判据不动。

3.3 **别的会话（任务服务巡检批 14 乙）那一串改动对登记的影响**（报告 `~/.local/share/singlefs/work/task-server-batch-fourteen-b-e171/result.md`、`~/.local/share/singlefs/work/task-server-e171-resume-fix/result.md`；改的是编排器、新建的 `shard_relay.py`、`fake_task_server.py` 与钉值）：
   1. **别的台怎么起**：编排器不再经 `scripts/hosts/multi-host-run.sh --peer` 送树、取回，改走统一任务服务的分片协议（`task-server/task_server_lib.py` 文件头「分片」）：计划至多两片，片号 0 那一片跑整件编排，别的台那一行经 `shard_relay.py relay` 写进共享计划目录、由落在那一台的片代跑；两台照同一绝对路径读写共享根。上一次登记修订 8 第 8.4 节、修订 10 第 10.2 节与「主 agent 认定：多机跑法交回」里「送去」「取回」的落点因此不再被用：H18 三个、H24 四个落点照旧要填着、不与修订 6 那一趟同名（`revision_8_placeholders.py` 的 `check_h18`、`revision_10_placeholders.py` 照核，今天已填），值不改。H35 的「为什么」那句点名的 `build_peer_kit` 已不在，值 `false`（别的台的行只有静态拦）不变。
   2. **不经服务分片时**：`TASK_SERVER_SHARD_PHASE` 没设（两份入口 `.sh` 里 `exec` 的那一下就是这种），编排只用接到它的那一台，跨机对照那一份写 `E7RESULT name=verdict cross_host_identical=not_run row=0 reason=…`。上一次登记修订 8 第 8.6 节运行条件「不悄悄退回单机」由这一行的 `reason=` 写明，不算悄悄；跨机对照没跑，读法照第 3.4 条。计划分了两片却都落在同一台、或各片 1800 秒里没登记齐：退 78（运行条件，同第 8.6 节）。
   3. **冻结层重钉（量的，2026-11-02）**：目录里每个 64 位十六进制钉值，凡等于目录里某份 `.py` / `.sh` 此刻 sha256 的，都钉的是那一份此刻的内容：`cross_host_o3_check.py` 第 56 行、`reuse_library.py` 第 454、455、457 行、`rows_orchestrator_revision_11.py` 第 64 行、`rows_orchestrator_revision_8.py` 第 62、64 行、`rows_orchestrator_segment_two.py` 第 111 行、`rows_orchestrator_segment_two_revision_10.py` 第 99 行、`rows_orchestrator_segment_two_revision_11.py` 第 59 行、`segment_one_units_revision_11.py` 第 60 行；对不上此刻任何一份文件的钉值都是修订 6 那一趟的快照摘要、表与产物的 sha256（说那一次的）。S22「C936 没关」那一半按这个现量不再成立。
   4. **第 5.4 节第 3 条「冻结层里的旧文件名、写死 1–281」逐处判**（`grep -cE 'bad-implementations-(117|expansion-164)|range\(1, 282\)|1–281'` 在研究一侧目录里改之前命中 8 处、改之后 7 处）：`list_check.py` 第 42 行（改名之前那一侧的文件名常量）、第 246 行（文档串说的是这一次改动本身）、`test_removal_effect.py` 第 53 行（手造原登记那几份产物的整表 1–281，说那一次的）、`revision_8_research_side_of_the_revision_6_run.json` 4 处（第 5.4 节第 3 条点名不改）照原样；`bad_implementation_table.py` 第 3 行文档串说的是这一目录现在的名单名，改成 `bad-implementations-step-8.tsv`。那份文件没有被别的文件按字面钉 sha256；它在驱动行复用键的 `[research]` 节里（`reuse_library.py` 第 141 行），同一批 `row_driver.py` 已改过，键本来就变了。

3.4 **跨机对照 `not_run` 怎么读（收严）**：上一次登记修订 8 第 8.7 节作数的条件「S17 不触发」与修订 10 第 10.7 节 S17 的两处触发观测，读的是两段 `-rows-cross-host-check-r8.out` 的 verdict 行。`cross_host_identical=not_run` 是没判，不算「不触发」：那一段 Q1–Q4 的读数不满足作数的条件。第六节「现状·逐格」照原样报；它的作数与否另起一句写「S17 未判」。要作数，那一段得在两台上跑、verdict 行写 `cross_host_identical=true`。

3.5 **两份入口怎么起、怎么判完成**（`run_segment_one_revision_11_multi_host.sh`、`run_segment_two_revision_11_multi_host.sh`；这一段去掉了两份里四处 `nice -n 19`，文件头起法从 `detached-run.sh` 改成 `task-server/task submit -- bash <这一份>`，第二段文件头「整表 281 行」改成「两张名单第一列之并」）：
   - **起法**：在仓根 `task-server/task submit -- bash scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_revision_11_multi_host.sh`（第二段同形）。经入口起不分片，跨机对照 `not_run`（第 3.3 条第 2 项），照第 3.4 条不作数。第二段要两台：交编排层 `python3 -B …/rows_orchestrator_segment_two_revision_11.py run --config …/revision_11_placeholders.json --repo <仓根>`（文件头第二行 `# task-server-shards: run`，服务照它分片）。第一段今天没有经服务分两台的起法：`segment_one_units_revision_11.py` 没有分片声明，第 11–13 步的逐行编排是它起的子进程、拿不到分片环境；要两台须另改这一层，不在这一条修订里，交主 agent。
   - **判完成**：退出码照 `segment_one_units_revision_8.py`、`segment_one_units_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py` 文件头（0 全部单元与行绿；1 有单元或行红、或复用账本 N + M 不等于登记数；2 用法错；3 按路径载入的冻结层不是钉的那一份；78 运行条件不满足；143 收到停止信号），再看日志没有「✗」行、下面每份产物在且 `grep -c '^E7RESULT '` 等于它 `name=done emitted=` 的值。第一段：`<H16 前缀>-rows-driver-r8.out`、`<H16 前缀>-rows-cross-host-check-r8.out` 的 verdict 行、`<H16 前缀>-reuse-ledger-units-r8.out`；第二段：`<H23 前缀>-rows-driver-r8.out` 与它的 `-timing`、`<H23 前缀>-rows-cross-host-check-r8.out` 的 verdict 行、`<H23 前缀>-reuse-ledger-rows-r8.out`。没跑完时复用账本只写进日志目录、不进 `research/results/`。
   - **产物名**：前缀是冻结层写死的 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-<run_date>-s1`（H16）与 `…-<run_date>-s2`（H23），后缀 `-r8`（`revision_8_placeholders.py` 的 `PRODUCT_STEM`、`REVISION_8_SUFFIX`；改它要连着重钉一串，不改）。第十一节比对对象表 #5–#7 的重跑产物名照此改记：#7 是第二段编排器写的 `<H23 前缀>-rows-driver-r8.out`；#5、#6 是两段比对器的 `--output`，交比对器时写 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-<跑的那天>-s1-comparator.out`、`…-s2-comparator.out`。
   - **`PLAN_ONLY=1`**：一个装置都不起、不编、不写产物与日志。第一段打修订 8 的 plan（占位逐个 H 核、前提核对、单元表与次序、每个单元会跑的命令、逐行编排的 plan），再打每个读数单元的键与库里有没有；第二段先打一行「修订 11 占位 …；H31 …；修订 10 占位 …」，再打静态的每行永远现跑与否，再交修订 10 的 plan。读三样：H31 那一句（「与现在的文件逐个相同」才过）、各 H 报没填或不过的行、`✗ 运行条件不满足（退 78）` 行与退出码。plan 在占位没填齐时退不退 78 没跑过，以输出为准。

3.6 **`revision_11_placeholders.json` 与它点名的修订 8、10 两份占位怎么处置**：现状整表段就是上一次登记修订 8 那一趟（第五节 5.6「跑一趟两边都补」），照用这三份、不另起一份。拍树宣布之后、起跑之前由执行员照各文件头的表填、主 agent 认：
   - **H31**：点名的九份文件（含两份入口）此刻 sha256 与登记值九份都不同（2026-11-02 现量），不重填 run 就退 78；在这批文件最后一次改动之后照现 sha256 重填。
   - **H1–H9、H14–H17、H22、H23、H25、H28、H33、H34**：拍树那一刻照各自的表填；宣布行写进 `records/2026-10-31-删兼容旧版本代码.md`（第十一节末段）；H16、H23 的 `run_date` 写拍树后起跑那天，`run_suffix` 照 `-r8`。
   - **H17 填不齐**：必登清单（`revision_8_placeholders.py` 第 73–77 行）含 `<工作根>/e171-r1-segment1-rev6/s2_compare.py`（第 71 行 `S2_COMPARE`），那份文件 2026-11-02 不在（`ls` 报不存在）、`git log --all -- '*s2_compare.py'` 0 行；上一次登记修订 14 已把用它的第一段第 2 步改成显式退 2、S2 那一条读数记缺。不补，第一段 `start` 那一核不过；补法（入库、或改必登清单连着重钉）交主 agent。
   - **H30**：`library_root` 写的是 `~/.local/share/singlefs/reuse/e171-r1`，经 `~` 取，第二台的家目录是另一个；`reuse_library.py` 的存法正由别的会话接判定库服务，H30 怎么填跟那件走，这一条不定。

3.7 **现状整表段起跑之前还差的**（S22 与上面几条，缺一样不起）：S20 重钉那一趟（装置换进主工作区那一步红在 `scripts/hosts/compile-then-swap.py` 的仓副本缺 `prune-server/`，那份脚本别的会话在改）；H31 重填；H17 的 `s2_compare.py`；H30 跟判定库服务那件；拍树宣布行；第一段要作数还缺分两台的起法（第 3.4、3.5 条）。

### 修订 4（2026-11-02，承接的执行员；现状整表段一份产物都没跑，这一条修订里的单测、证红与变异也都还没跑）

依据：派发提示（批 15）原文——第 1 条「撤掉登记修订 3 第 3.4 条：用户 2026-11-02 在弹窗里选「算，单机也作数」（问句是「跨机对照没跑（cross_host_identical=not_run）时，这一段的读数算不算数」）。写成修订 4，原文第 3.4 条留着、写明撤回与依据；第一段经入口单机跑也作数。」；第 2 条「补回第一段丢了的脚本：用户 2026-11-02 选「照登记重写并入库」。」；第 3 条点名的干跑撞出的三处缺陷；第 6 条「修订 3 第 3.7 条还差的前置（H31 九份 sha256、拍树宣布行）照登记能填的填上；H30（`reuse_library.py` 存法）跟判定库服务那件走，不碰」。读数依据：干跑任务 3702、3703 的日志（主 agent 转述，草稿目录里有 `task-server/task log` 的整份）；S20 重核任务 3669（主 agent 转述）。第一至十一节、修订 1–3 的原文一字不改。第六、七、九节每一条的去留与报告方式只动第 4.1 条那一处，依据是派发提示第 1 条那一句；别的都是收严或补实现。

4.1 **撤回修订 3 第 3.4 条**（第 3.4 条原文照留）：撤回依据是用户 2026-11-02 在弹窗里选「算，单机也作数」，问句是「跨机对照没跑（cross_host_identical=not_run）时，这一段的读数算不算数」。读法改成：两段 `-rows-cross-host-check-r8.out` 的 verdict 行写 `cross_host_identical=not_run` 时，S17 记「未判（单机）」，不让那一段不作数；上一次登记修订 8 第 8.7 节、修订 10 第 10.8 节作数的其余条件照旧；跨机对照真跑出 `cross_host_identical=false` 照旧触发 S17。第六节「现状·逐格」照原样报，作数一句写「S17 未判（单机，修订 4 第 4.1 条认作数）」。修订 3 第 3.5 条「经入口起不分片，跨机对照 `not_run`（第 3.3 条第 2 项），照第 3.4 条不作数」那半句随之撤回，第 3.7 条「第一段要作数还缺分两台的起法」一项不再是前置。第 3.4 条没有落进代码（研究一侧目录的 `.py`、`.sh` 里 `grep -rn '第 3\.4\|3\.4 条'` 命中 0 行，量的；上一段登记落地清单那一行也写「读法，没有代码」），代码不改。

4.2 **第一段第 2 步的逐文件比 `s2_compare.py` 照登记重写入库**：当年那一份写在仓外草稿 `e171-r1-segment1-rev6/` 里，草稿目录不在、`git log --all --diff-filter=A -- '*s2_compare*'` 空（主 agent 现查），上一次登记修订 14 因此把这一步改成显式退 2。这一份放进 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/s2_compare.py`，判法逐字依据：原登记第十一节 S2（`git show 31128273b^:research/prompts/e171-preregistration.md` 第 561 行：八个文件 `base-sectors.bin`、`ops.tsv`、`ops-payload.bin`、`calls.tsv`、`torn.tsv`、`torn-payload.bin`、`states.tsv`、`quick-tier.tsv` 的 sha256，加 `impl-verdicts.tsv` 去掉耗时列之后逐字节比）；上一次登记第十一节 S2（「原登记 S2 原样」）与修订 8 第 8.4 节步表第 2 步（「照旧」）；单元脚本接着核的 `s2_differing_files=0 s2_compared_files=18` 与产物行形照上一次那一趟的产物（`git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-r6-r3.out`，21 行）。比流一、流二两条（E170 导出装置只导这两条），共 18 件；`impl-verdicts.tsv` 去耗时列的函数与作废 Z1 那一比是同一个（`segment_one_comparator.lines_without_the_last_column`）。收严一处：原文没写一边缺件怎么算，这里记 `absent`、算不同（读不到不当成相同）。此刻 sha256 `fa9bd9495d6da181838a70bee8b056abc133741921b7ac97d0e4a6713ec64537`，与当年记的 `8ca65474…`（上一次登记第 1307 行）不同：那一份内容丢了，对不上也核不了，两份不是同一个文件。落点：修订 11 单元脚本 `segment_one_unit_revision_11.sh` 的第 2 步照修订 8 单元脚本的第 2 步改回现跑，只换比对脚本的路径；`revision_8_placeholders.py` 的 H17 必登清单（`H17_REQUIRED_FILES`）改指研究目录里这一份（相对仓根）。修订 8 单元脚本 `segment_one_unit_revision_8.sh` 第 22 行仍指仓外那一份，不改：修订 11 单元编排起的是修订 11 单元脚本（`segment_one_units_revision_11.py` 文件头 ①）。
   另外三份不重写：`frozen_tree_lines_revision_6.py`——上一次登记修订 8 第 8.3 节「修订 6 第 0 步的判法 … 不改它。执行员另写 `frozen_tree_lines_revision_8.py`」，第一段第 0 步用后者；`resume_check.py`——修订 8 第 8.4 节「续跑（同一条命令重起就是续跑，不再要 `FROM_STEP`）」，续跑核对在单元编排里；`s15_compare.py`——修订 8 第 8.4 节步表第 1、6、7、10 步都「不接 S15 … 停机，改出对照产物」，第一段的单元里不再用它（`segment_one_units_revision_8.py` 自证格 `unit-script-commands` 另核没有一个单元打出 `s15_compare`），只剩第 8.5 节对照脚本 `contrast_revision_6_against_revision_8.py` 的 ② 调它，而那一比的另一边是修订 6 那一趟的导出根（H15 的 `dumps_root`），本登记第四节第 6 条记它已不在（`scripts/experiment-apparatus/target/` 下此刻没有 `e171-r1-dumps`，量的），重写了也比不出；那份对照怎么出（第六节「现状·逐格」第一段那一半）交主 agent。

4.3 **干跑撞出的三处缺陷（收严，都退 78 并点名）**：
   1. 逐行编排外层 `rows_orchestrator_revision_8.py` 读 H16：`run_date`、`run_suffix`、`dumps_root`、`logs_directory` 缺一项（没有这一键、null 或空串）就拒、点名缺哪几项（`require_h16`），在 `configure` 之前核（外面一层修订 11 的 `configure` 也读 H16 的导出根）；改之前缺 `logs_directory` 得到字符串 `None`，日志落进当前目录（那一趟在仓根建了 `None/sequence.log`）。
   2. 冻结树快照是空集（树不在、读不到，或没有 `.rs` 与 `Cargo.*`；摘要是空输入的 sha256 `e3b0c442…`）时不往下判 Z5，当成认不全退 78（`require_readable_tree`，在编排器原来的 Z5 核对之前）。同一次干跑第一段 plan 算键时读不到树打了 traceback：`segment_one_units_revision_11.py` 的 plan 改成照第二段编排层那一句报「算键用的树 … 认不全或读不到」。
   3. 复用库根（H30 的 `library_root`）改经新文件 `reuse_library_root.py` 取：没填取数据根下的缺省（`.claude/hooks/lib_work_root.py` 的 `shared_data_path("reuse")` 下的 `e171-r1`，两台同一个绝对路径；那份库不改）；填了绝对路径照用；经 `~` 或相对路径取的，run 退 78、plan 打出这一句。三层编排（`segment_one_units_revision_11.py`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`）都经它取，单元 `reuse-selfproof` 的 `--library <库根>/selfproof` 也就两台相同。H30 的值没动（仍是 `~/.local/share/singlefs/reuse/e171-r1`，第 3.6 条：跟判定库服务那件走），所以起跑之前 H30 要改成留空或两台同一的绝对路径，不改 run 退 78。

4.4 **冻结层按依赖次序重钉**：`rows_orchestrator_revision_8.py` 改了（第 4.3 条第 1、2 项），它的 sha256 `b4bfcec2ef128f900b8712ba4cbeeb9ea57c6a7dd66a19faf5482539094c604a` 写进钉它的 `rows_orchestrator_revision_11.py`（`ROWS_R8_SHA256`）；这之后填 H31。被钉的其余几份（`rows_orchestrator.py`、`cross_host_o3_check.py`、`rows_orchestrator_segment_two.py`、`rows_orchestrator_segment_two_revision_10.py`、`segment_one_units_revision_8.py`）这一段没改，钉值不动。
   **H31 重填**（第 3.6 条要的那一项）：原九份照此刻 sha256 重填，另加 `reuse_library_root.py`（三层编排都 import 它），共十份；值在 `revision_11_placeholders.json`，主 agent 认。`reuse_library.py`、`count_reusable_units.py` 是别的会话的暂存改动，再改一次 H31 就要再填。

4.5 **单测与研究一侧变异（补实现）**：`test_revision_8.py` 加 17 条（冻结树认不全 3 条、H16 5 条、`s2_compare.py` 8 条、修订 11 单元脚本第 2 步 PRINT_ONLY 打出的命令 1 条；26 → 43 条，`grep -c '    def test_'`），原有一条用例的假树补一个 `Cargo.toml`（空集的树在核 H8 之前就当认不全拒掉）；`test_revision_11.py` 加 4 条（库根两台同一；20 → 24 条）。变异另开两张新表，不加进修订 8、修订 11 那两张（它们登记了上一次那几份产物的 exact 复跑）：`scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_preconditions.tsv`（R2P-1 至 R2P-11，单测 `test_revision_8`）与 `…_current_state_library_root.tsv`（R2P-12 至 R2P-15，单测 `test_revision_11`）；产物 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-<跑的那天>-current-state-preconditions-research-mutations.out` 与 `…-current-state-library-root-research-mutations.out`，`replay.sh` 各登记一行（驱动 `driver_e171_second_rerun_current_state_preconditions_research_mutations`、`driver_e171_second_rerun_current_state_library_root_research_mutations`，产物名写的是 2026-11-02，跑的那天不同就照第 1.12 条改）。变异全抓才算这几处有判别力。

4.6 **S20 过了**（主 agent 转述，任务 3669）：`research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants-r2.out` 83 行、`holds=false` 0 行、`name=verdict s1_failed_checks=0`（第 2.4 条的判法）。第 3.7 条清单里 S20 那一项划掉。

4.7 **现状整表段起跑之前还差的**（替第 3.7 条那张清单，缺一样不起）：H1–H9、H14、H16、H17、H22、H23、H25、H28、H33、H34 拍树那一刻照各自的表填（H17 现在要登 `s2_compare.py` 研究目录那一份）；H30 改成留空或两台同一的绝对路径（第 4.3 条第 3 项）；拍树宣布行由主 agent 写进 `records/2026-10-31-删兼容旧版本代码.md`（行首「- E171 第 2 次重跑拍树宣布（<日期>）：」，第十一节末段）；H31 在这批文件最后一次改动之后再核一遍；这一条修订的单测、两张变异表与干跑。第一段经入口单机跑也作数（第 4.1 条），不再缺分两台的起法；第六节「现状·逐格」r1 第一段那一半要的第 8.5 节对照 ② 出不来（第 4.2 条末），交主 agent。

### 修订 5（2026-11-02，承接的执行员；现状整表段一份产物都没跑；这一条修订里的单测、证红、变异与自证都还没跑）

依据：派发提示（批 16）七条，原句——第 1 条「第一段对照（登记第 8.5 节）要整份出得来：用户 2026-11-02 在弹窗里选「重出修订 6 导出，补 s15 脚本」」；第 2 条「研究一侧变异驱动在拷出的目录里基线红 … 改驱动，让拷贝里也取得到（不改 `.claude/hooks/lib_work_root.py`），不再要仓形镜像」；第 3 条「修订 8 变异表 `R8-18` 锚点命中 0 次：修锚点」；第 4 条「H30 留空走 `reuse_library_root.py` 的共享根缺省（不经 `~`）」；第 5 条「修订 11 单元脚本里留着的 `nice`、`capped.sh` 包装删掉（资源由服务定）」；第 6 条两台对称（只出补丁、不打进主工作区）；第 7 条 H16 建议值与拍树。读数依据：主 agent 转述的任务 4023（单测 147 条绿）、4024（证红退 1）、4025（前置那张表 11 条全抓）、4026（库根那张表基线红、退 2）、4027、4028（两段干跑）。第一至十一节、修订 1–4 原文一字不改；第六、七、九节每一条的去留与报告方式不动（第 5.1 条只写做不到的事、交主 agent）；第 5.5 条的依据是派发提示第 5 条那一句。

5.1 **第一段对照 ② 要的修订 6 导出重出不来（量的，2026-11-02）**：
   - 修订 6 树（快照摘要 `7726cdd6…`，284 份）是第一段新树（`4d071c12…`）只换装置拍成的（上一次登记修订 6 第 6.3 节）；第一段新树拍自 HEAD `31e47115…` 加未提交的改动（`git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out` 记 94 行 `git_status`，其中 `.rs` 与 `Cargo.*` 约九十份）。
   - 那份产物记了 13 份文件的 sha256：拿 `git log --all -- <路径>` 的每一版（每份至多 60 版）现算比，有 9 份一版都对不上（`model.rs`、`model_comparison.rs`、`model_compares_inode_creations_across_two_inode_leaf_containers.rs`、`mount_transparency.rs`、`crash_injection.rs`、`crash_judge_gpu.rs`、`singlefs-checker` 的 `image.rs` 与 `walk.rs`、装置 `b1457bc6…`）。装置路径在 git 里只有 10 版，`67e7a549…`（修订 6 树装置）、`dbef78ed…`、`b1457bc6…` 都不在，只有 `ab35105f…` 一版（提交 `f8a9ddaab`）；P6 补丁与那几份 fixture 在 git 里也没有。
   - 所以修订 6 树逐字节造不回来，从它编的两个二进制与第 1、6、7、10 步的导出也重出不来；拿近似的树重跑，判的是另一份实现，是新实验。对照脚本 ③（两棵树的逐文件差表）同样要修订 6 树；①、④ 只读两趟的产物，修订 6 那一趟的产物在 git 里取得到。
   - `s15_compare.py` 照登记重写入库那一半没写：它只给 ② 用，② 的另一边造不回来。第六节「现状·逐格」r1 第一段那一半照现在的写法出不来；改成只出 ①④、②③ 记缺，是第六节那一行的报告方式，交主 agent 问用户。

5.2 **变异驱动的副本搭成仓形（补实现，判据不动）**：`mutation_driver.py` 的 `fresh_copy` 在 `<草稿目录>/<标签>/` 下照仓根的样子搭副本：从仓根到研究目录这一条路径上，每一层除了往下那一项与 `.git`，别的条目都链回真仓，研究目录本身是拷贝；删副本删整个 `<标签>/`（`copy_root`）。研究目录里的模块往上三层取 `.claude/hooks/lib_work_root.py`、`.claude/heavy-thread-default` 与搬迁登记表，在副本里落到真仓那一份；`--scratch` 指哪里都一样，不用调用方另搭镜像。产物行不带路径，各表的产物不因这一处变。单测 `test_mutation_driver.py`（5 条）。

5.3 **修订 8 那张表的 R8-18 锚点**：`segment_one_units_revision_8.py` 的 `fits` 在「内存额度由服务定」那一回删了内存那一维，原文 `return threads <= budget.threads and memory <= budget.memory_bytes` 命中 0 次。改成同一处还在的那一维：变异名「R8-18 放得下不算在跑的单元」，原文 `sum(item.threads for item in running) + unit.threads <= budget.threads`，替换文 `unit.threads <= budget.threads`；`test_revision_8.py` 的 `test_a_unit_does_not_fit_when_the_threads_run_out` 会红（推的）。草稿 `tools/anchor_hits.py` 照驱动的数法数，这张表 26 条都恰命中 1 次（量的）。变异名与原文都变了，这张表的产物与 `replay.sh` 登记的那一份必不同：登记行改指 `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-11-02.out`（跑的那天不同照第 1.12 条改）。

5.4 **H30 留空**：`revision_11_placeholders.json` 的 `H30.library_root` 改成 `null`（另加一句 `why_library_root`），库根照第 4.3 条第 3 项取数据根下的缺省，两台同一个绝对路径。`test_revision_11.py` 里造「经 `~` 取」那一格的字面量改成 `~/e171-reuse-library-under-the-home-directory`，判的仍是「不是绝对路径就拒」，不再写数据根下那一份的样子（门禁 code-tooling 的 shared-data-root-literals 格判红在原来那一行）。

5.5 **修订 11 单元脚本去掉 `nice` 与 `capped.sh`**：`segment_one_unit_revision_11.sh` 七处（`device_step` 打印与真跑、`driver_step` 打印与真跑、O3 阳性对照、研究一侧变异表、`reuse-selfproof`）。单元的线程不再由脚本里的 `capped.sh "$THREADS"` 设，取服务给的每台默认重活线程数；单元编排照 `THREADS` 排单元的那一套没改，几个单元同时起时各自拿到的线程可能多于编排算的额度（推的，没量），影响挂钟，不改判法与读数。修订 8 单元脚本 `segment_one_unit_revision_8.sh` 不改（修订 11 单元编排起的是修订 11 那一份）。

5.6 **宣布行（落实第十一节末段「判法里读宣布行的那一处照改」）**：`frozen_tree_lines_revision_8.py` 的 `ANNOUNCEMENT_PREFIX` 改成 `- E171 第 2 次重跑拍树宣布`、`RECORDS_FILE` 改成 `records/2026-10-31-删兼容旧版本代码.md`，`revision_8_placeholders.json` 的 `H1.records_file` 同改；上一次登记修订 8 第 8.1 节第 2 条 a 那一句的 `grep` 读作新行首、新调度记录。用例 `test_revision_8.py` 的 `test_the_announcement_line_of_the_second_rerun_is_read_from_the_compatibility_removal_record`；变异另开第三张表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_announcement.tsv`（R2P-16、R2P-17，单测 `test_revision_8`），产物 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-<跑的那天>-current-state-announcement-research-mutations.out`，`replay.sh` 登记一行（驱动 `driver_e171_second_rerun_current_state_announcement_research_mutations`）。库根那张表（第 4.5 条）重跑的产物名在旧名去掉 `.out` 之后接跑的那天，登记行已改指 `…-2026-11-02-current-state-library-root-research-mutations-2026-11-02.out`；基线红的那一份留着。

5.7 **H31**：这一段改了其中两份（`segment_one_unit_revision_11.sh`、`test_revision_11.py`），照此刻 sha256 重填；另八份没动，现算与登记值相同（量的）。

5.8 **两台对称（直接改进研究目录，12 份文件）**：派发提示第 6 条原写「只出补丁」，主 agent 同日追加「不再等别的会话同一次换上 … 改成直接打进主工作区的 E171 目录」与多机配置按台键那三件，照后者做。自证造的多机配置串里删掉 `PEER_REPOSITORY_DIRECTORY`、`PEER_CARGO_BIN_DIRECTORY`、`PEER_MEMORY_CAP`，换成 `CARGO_BIN_DIRECTORY_BY_HOST`、`BUILD_ROOT_BY_HOST`、`SLICE_MEMORY_CAP_BY_HOST`（`<地址>=<值>`，HOSTS 每台一项；`rows_orchestrator.py` 与 `rows_orchestrator_segment_two.py` 各一处）。研究一侧不认 `local`、`peer` 这类角色名，也不读 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`：分片登记照 `TASK_SERVER_SHARD_HOST` 给的地址记（`shard_relay.py`）；编排那一片所在的台是第 1 台，登记地址与它不同的是第 2 台（`rows_orchestrator.py`）；编排那一片的工作根看这一台自己有没有服务给的本地草稿目录，不看角色；假服务给假地址（`fake_task_server.py`，取文档地址段）；新单测 `test_symmetric_hosts.py`。`rows_orchestrator.py` 改了，冻结层照依赖次序重钉（`cross_host_o3_check.py`、`rows_orchestrator_revision_8.py`、`reuse_library.py` 里的钉值、`rows_orchestrator_segment_two.py`、`rows_orchestrator_segment_two_revision_10.py`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`），H31 随之重填（重钉到不动点之后，每个钉值都等于被钉那份此刻的 sha256，量的）。「这一台」这一处研究一侧用不着：编排只比各片登记的地址是不是同一个；要认这一台的地方照 `scripts/hosts/multi_host_addresses.py` 的 `resolve()`。

5.9 **现状整表段起跑之前还差的**（替第 4.7 条那张清单，缺一样不起）：两台对称那几处的单测与编排器自证；H1–H9、H14、H16、H17、H22、H23、H25、H28、H33、H34 拍树那一刻照各自的表填；拍树宣布行由主 agent 写进 `records/2026-10-31-删兼容旧版本代码.md`（行首 `- E171 第 2 次重跑拍树宣布（<日期>）：`，第 5.6 条）；H31 在最后一次改动之后再核；这一条修订的单测、证红、四张变异表与两段干跑；第 5.1 条交主 agent 定。

### 修订 6（2026-11-02，承接的执行员；现状整表段一份产物都没跑；这一条修订里的单测、证红、变异、自证与干跑都还没跑）

依据：派发提示（批 17）五条，原句——第 1 条「第一段对照照甲报：……用户选「照近似树重跑，另立新实验」：这一段第六节「现状·逐格」r1 第一段那一半只出对照 ①④（两趟比对器产物逐格、行耗时），②③ 记「修订 6 树与导出造不回来」，写进修订 6（连同上面的核法与读数）；`contrast_revision_6_against_revision_8.py` 改成 ②③ 那两项照这一条报「造不回来」、不因缺 H15 整份退 78，配单测与证红。近似树重跑另立实验，不在这一段。」；第 2 条「线程额度：三份编排器各记各的线程账……合起来每台会超过登记的每台默认重活线程数……照「线程由服务定、固定是登记值」改成一处记账、合起来不超；修订 8、10、11 各层照依赖次序重钉、H31 重填。配单测与证红。写进修订 6。」；第 3 条「拍树的编译目录：登记要「编译目录放在树里的 `target/`」，服务给 `CARGO_TARGET_DIR`……照「编译目录由服务给、不往仓里编」改登记与拍树步骤（拷编好的二进制进树里约定的位置那一步写清），写进修订 6」；第 4 条「`replay.sh` 里研究一侧变异驱动那五个函数……的 `nice -n 19` 删掉」。读数依据：主 agent 转述的任务 4305（单测 `Ran 184 tests … OK`）、4306（证红退 1）、两份编排器自证「自证 35 项，红 0」「自证 46 项，红 0」、`frozen_tree_lines_revision_8.py --selftest`「自证 33 项，红 0」，五张变异表全抓（五份产物的判决行见实验页第 2 次重跑一节第 11 条）。第一至十一节、修订 1–5 原文一字不改。第六节「现状·逐格」那一行的报告方式只动第 6.1 条那一处，依据是派发提示第 1 条那一句；第 6.2、6.3 条是跑法，不动判据。

6.1 **第一段对照只出 ①④，②③ 记「修订 6 树与导出造不回来」（撤回修订 5 第 5.1 条末句交主 agent 的那一问，问题由用户定了）**：
   - 核法与读数（主 agent 转述，量的）：用户 2026-11-02 先问「历史的版本 不是对的吗。 历史都删掉了 还存在吗」；主 agent 现查全部提交与 reflog（528 个）、3 个 stash、13300 个悬空 blob 里大于 200K 的 3700 个、第二台整盘、本机整盘 88 份同名文件，逐个算 sha256，都没有修订 6 用的 `67e7a549…`、`dbef78ed…`、`b1457bc6…`。之后用户选「照近似树重跑，另立新实验」：近似树那一趟是新实验，不在这份登记里。
   - 第六节「现状·逐格」r1 第一段那一半（比对对象表 #5）照这样报：① 两趟比对器产物逐格、④ 两趟第 11–13 步各行装置那一步的耗时照上一次登记修订 8 第 8.5 节出；② 第 1、6、7、10 步之后那几样比、③ 两棵树的逐文件差表各写 `status=not_reconstructible reason=revision_6_tree_and_exports_not_reconstructible`（② 按第 1、6、7、10-run-1、10-run-2 步各一行），收尾那一行这两格写 `not_reconstructible`、不写数（读不到不当 0）。
   - 落点：`contrast_revision_6_against_revision_8.py` 删掉调 `s15_compare.py`（那份当年在仓外草稿里，已丢）与比两棵树的代码，改写 ②③ 那几行；H15 只要 ①④ 用的四项（比对器产物、产物前缀与后缀、RESUME_TAG），导出根可以不填，缺那四项之一才退 78（`missing_h15_keys`）；`revision_8_placeholders.py` 的 `check_h15` 同改（导出根没填算过，填了就核它在）。④ 的台照编排器的账 `row-hosts.tsv` 认，修订 6 那一趟的账在它的导出根里、已不在：那一趟各行的台写 `ledger_absent`，不当本机（改前一律记本机，收严）；每份行产物在不在各写一行 `name=contrast_row_product present=…`（改前不在的那几份悄悄当没有行，收严）。
   - ④ 要读修订 6 那一趟的行产物，它们已归档出工作区：`git log --all --diff-filter=D` 在 `04ceaa904` 删的，`git show 04ceaa904^:research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-<步名>-<RESUME_TAG>-r6-r3.out` 取得到（`rows-probe-driver`、`rows-driver`、`rows-cross-host-row-0-driver` 那几份）。出对照之前主 agent 把它们取到一个目录，H15 的 `product_prefix` 写那个目录下的前缀（可以是绝对路径），`product_suffix` 写 `-r6-r3`，`rows_tag` 写 `-resume2`，`comparator_product` 写比对对象表 #5 那一份。
   - 第 2 次重跑登记第十一节比对对象表 #5 那一行的比法照这一条读；近似树那一趟出的 ②③ 不并进这一份。
   - 变异：修订 8 那张表删 R8-25（`relabel_s15_line` 删了，锚点没有对象），R8-26 的锚点改写成 `host_of_row` 里同一处（「行耗时一律记本机」，换文照旧）；那张表的产物再跑一次，名 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-11-02-r2.out`（跑的那天不同照第 1.12 条改），`replay.sh` 那一行已改指它。新加的判法进第四张表（第 6.4 条）。
   - 自证：格 `s15-relabelled` 与开关 `s15-triggered-kept` 删了；加格 `parts-not-reconstructible`（开关 `not-reconstructible-as-zero`）、`row-times-without-ledger`（开关 `absent-ledger-as-host-1`）。

6.2 **线程只记一处账，合起来不超过每台登记值（跑法，判据不动）**：
   - 新文件 `host_thread_ledger.py`：一台一本账，放在工作根下 `e171-host-thread-ledger/`（两台同一个绝对路径，各台只读写自己那一份，按这一台在多机配置 HOSTS 里的地址分：服务分片给的 `TASK_SERVER_SHARD_HOST`，没给照 `scripts/hosts/multi_host_addresses.py` 的 `resolve()`）；额度是那一台登记的每台默认重活线程数（`heavy_threads.default_heavy_threads`，服务起的任务里就是登记值）。各层起占线程的进程一律经 `host_thread_ledger.py hold --threads N --label L -- <命令>` 起：先来先得排进那一台的账，拿到 N 才起，退出就还；pid 不在了（或起始时刻对不上）的项取锁时清掉。
   - 记多少：逐行驱动记它的 `--threads`（驱动自己照它给编译、装置、执行器定线程，冻结的 `row_driver.py` 不改）；重报（carry）记 5；第 13 步单线道记每台线程；别的台的预检记它的 `--threads`；第一段的读数单元记登记值（修订 5 第 5.5 条之后单元脚本里不另设线程，装置拿服务给的那一份，所以按实际记；改前记修订 6 跑法的 10）；只比对的 Z1、Z7 记 1。
   - 套在哪：`rows_orchestrator.py` 的 `launch_local`（本机驱动、重报）与 `peer_side_invocation`（转交到别的台的命令在那一台套 hold，预检与每一行都经它）；`rows_orchestrator_segment_two.py` 的 `launch_local_two` 与 `launch_peer_two`（经第一段编排器的 `held`、`peer_command`）；修订 10 那一层干跑打的命令同样带 hold；`segment_one_units_revision_8.py` 的 `launch`（单元的命令）。各层原来的槽数与「放得下」只是同时起几个的上限，线程够不够只看这本账。
   - 后果（推的，没量）：读数单元记的就是额度，同一台上同一时刻至多一个读数单元在跑，也不与逐行驱动同时跑；上一次登记修订 8 第 8.4 节「第 9 步与第 7 步同时起」不再成立，第 7 步的进程挂钟（T）是它单独在本机、拿登记值那么多线程量的。第一段挂钟比第十四节推的长（第 7、9 步各约 4.1 小时串起来，再加别的单元，推的）。T 的量法变了这一件另交主 agent（「交主 agent 的几件」第 1 件）。
   - 单元编排那一层删掉「此刻一个单元都放不下退 78」（读数单元记的就是额度，那一支到不了）与它的自证格 `e2e-no-room`；格 `allocation-threads` 改成三个读数单元只放得下队首的第 7 步，`e2e-two-rounds` 改成第 7 步计时末行写 `units=none`；新加格 `e2e-units-wait-for-other-holders`（外来一个 hold 着整份额度的进程，第一个单元等它还了才起），开关 `ledger-skipped`。三份逐行编排的自证与单元编排的自证外壳把账本目录设到各自的临时目录（`E171_HOST_THREAD_LEDGER_DIRECTORY`），不碰真账本。
   - 冻结层照依赖次序重钉（量的，2026-11-02，每个钉值等于被钉那份此刻的 sha256）：`rows_orchestrator.py` → `08b7a55f…`（钉在 `cross_host_o3_check.py`、`rows_orchestrator_revision_8.py`、`rows_orchestrator_segment_two.py`、`reuse_library.py`）；`cross_host_o3_check.py` → `098792f8…`（`rows_orchestrator_revision_8.py`、`reuse_library.py`）；`rows_orchestrator_segment_two.py` → `ec05fdba…`（`rows_orchestrator_segment_two_revision_10.py`、`reuse_library.py`）；`rows_orchestrator_segment_two_revision_10.py` → `4ed19b99…`（`rows_orchestrator_segment_two_revision_11.py`）；`rows_orchestrator_revision_8.py` → `e33efa9c…`（`rows_orchestrator_revision_11.py`）；`segment_one_units_revision_8.py` → `e28368bb…`（`segment_one_units_revision_11.py`）。`reuse_library.py` 只换这三个钉值，存法那一块没碰。
   - H31 重填：原十份里四份变了（`reuse_library.py`、`segment_one_units_revision_11.py`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`），另加 `host_thread_ledger.py`（三层编排起的进程都经它），共十一份，值等于此刻现算（量的）。`host_thread_ledger.py` 另进 `revision_8_placeholders.py` 的 H17 必登清单、`revision_10_placeholders.py` 的 H25 必登清单与修订 10 那一层的输入清单 `INPUT_RESEARCH_FILES`（H26）。

6.3 **拍树的编译目录由服务给，不往树里编（替上一次登记修订 8 第 8.1 节「怎么拍」第二条括号里那一句）**：在树里 release 编两个 bin 照旧 `--manifest-path` 指树里的 `Cargo.toml`，整条交 `task-server/task submit`，编译目录用服务给的 `CARGO_TARGET_DIR`（命令里不写编译目录）；编完把 `task wait` 末行 `build=` 目录下 `target/release/e171_three_oracle_collusion_after_the_oracle_fixes` 与 `target/release/e170_o3_spec_executor_export` 两个文件拷进树里 `target/release/`（先建这一层目录；只拷这两个文件，不拷别的编译产物），再跑 `frozen_tree_lines_revision_8.py after-build`。快照摘要不罩 `target/`（自证格 `snapshot-table-ignores-target`），拷进去不改编前编后的摘要；H7 与单元脚本照旧从树里 `target/release/` 读这两个二进制。拍完交 `task-server/task clean <号>` 删服务的编译目录。

6.4 **第四张研究一侧变异表**：`scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_threads_and_contrast.tsv`（R2P-18 至 R2P-38，21 条；单测 `test_host_thread_ledger`、`test_revision_8`），产物 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-<跑的那天>-current-state-threads-and-contrast-research-mutations.out`，`replay.sh` 登记一行（驱动 `driver_e171_second_rerun_current_state_threads_and_contrast_research_mutations`，名里写 2026-11-02，跑的那天不同照第 1.12 条改）。账本目录取哪里（`E171_HOST_THREAD_LEDGER_DIRECTORY`）那一处没进表：改坏它，单测会写进真工作根下的账本。

6.5 **`replay.sh` 里研究一侧变异驱动的 `nice -n 19` 删了**：`driver_e171_r1_s1_research_mutations`、`driver_e171_r1_s2_research_mutations`、`driver_e171_r1_s1_revision_8_tools_research_mutations`、`driver_e171_r1_s2_revision_10_tools_research_mutations`、`driver_e171_r1_revision_11_tools_research_mutations`、`driver_e171_second_rerun_list_removal_research_mutations` 六个（派发提示写「五个」，现数六个，都删）。只去包装，产物不因此变（推的：结果行不带耗时）。

6.6 **现状整表段起跑之前还差的**（替第 5.9 条那张清单，缺一样不起）：这一条修订的单测、证红、第四张表与修订 8 那张表的重跑、三份逐行编排与单元编排的自证、两段干跑；拍树宣布行（主 agent 写，行首 `- E171 第 2 次重跑拍树宣布（<日期>）：`）；照第 6.3 条拍树、编、拷二进制；H1–H9、H14、H16、H17、H22、H23、H25、H28、H33、H34 拍树那一刻照各自的表填；H31 在最后一次改动之后再核；T 的量法（第 6.2 条后果那一句）主 agent 定。

### 修订 7（2026-11-02，承接的执行员；现状整表段一份产物都没跑；这一条修订里的单测、证红、变异、自证与干跑都还没跑）

依据：派发提示（批 17 续）四条，原句——第 1 条：上一段新建的跨任务按台 json+lock 线程账「不许留」，转述做统一任务服务的会话的判法「线程、机器、额度只由服务定；跨任务的账是服务之外的第二个排程器」，「正经写法两种：(a) 一条任务里同时起的几个单元，编排器在进程内把这一条任务的线程额度分给它们（服务给每次尝试套 `capped.sh <登记值>`，命令里读得到 `SINGLEFS_THREAD_CAP`，见 `scripts/hosts/capped.sh` 文件头），账只活在这一条任务的进程里，不落盘、不跨任务；(b) 每个单元各要吃满一台的，拆成服务的分片子任务 …；串着跑也合规」「三层编排改成 (a)（能拆成 (b) 的优先拆，你判、写依据）」；第 2 条「主 agent 定 T 的量法：照 (a) 在进程内照 r1 修订 6 那一趟的比例分——第 7 步（T 的来源）那个单元 10 线程、逐行驱动每条线道 5 线程，保住登记逐行限时 max(1800 秒, 3T) 依据的比例；读数单元记它实际分到的线程数，不记登记值。写进修订 7（连同第 1 条的改法与依据），修订 6 里读数单元「记登记值」那句照撤」；第 3 条「第二段 `launch_local_two`、`launch_peer_two` 两处的线程设法补单测与变异行」与 ④ 取产物的命令；第 4 条「冻结层照依赖次序重钉到不动点、H31 重填 …；修订 8 那张变异表与第四张表照改动重整锚点」。读数依据：没有，这一段之前的改动一条验证都没跑过（主 agent 撤了上一段的单测）。第一至十一节、修订 1–6 原文一字不改；第六、七、九节每一条的去留与报告方式不动；本条都是跑法，判据不动。

7.1 **撤修订 6 第 6.2 条整条**（第 6.2 条原文照留）：那本跨任务的按台线程账的文件与它的单测删了；三层编排套它的几处（第一段逐行编排的本机驱动与转交、第二段逐行编排的本机驱动与转交、修订 10 那一层干跑打的命令、单元编排起单元）撤回；自证外壳与两份逐行编排自证里设账本目录那几行删；H17、H25 的必登清单与修订 10 那一层的输入清单（H26）里那一项删。第 6.2 条「第一段的读数单元记登记值」与它的后果「第 7 步与第 9 步不再同时跑」照撤（依据第 2 条末句）；第 6.2 条删的「此刻一个单元都放不下退 78」与自证格 `e2e-no-room` 恢复（读数单元不再记整份额度，那一支又到得了）。第 6.4 条第四张表里锚在那本账与 hold 上的 R2P-18 至 R2P-32 一并删（第 7.5 条）。

7.2 **线程额度在一条任务的进程里分（照 (a)）**：
   - **第一段**（一条任务；经入口起不分片、只用接到它的那一台，第 3.5 条）：额度 = 这一条任务的 `SINGLEFS_THREAD_CAP`（服务套的 `capped.sh <登记值>` 设的），没设取 `.claude/heavy-thread-default` 的登记值（`segment_one_units_revision_8.py` 的 `registered_threads`，原样）。读数单元各分 10（`READING_UNIT_THREADS`，r1 修订 6 跑法「每个装置一趟 capped.sh 10」），Z1、Z7 两个小单元各分 1；在跑的合起来加上要起的那一个不超过额度才起（`fits`，原样）；余数只给放得下的单元：额度 31 时三个读数单元同时跑（30），剩的 1 线程只给得了小单元；一个都放不下退 78。
   - **第 11–13 步的逐行编排**：要跑的单元全部绿、一个都不在跑才起（`rows_may_start`），分 5 × ⌊额度 / 5⌋（`rows_threads`；额度 31 分 30，除不尽的 1 不给），它照分到的数排线道（`rows_orchestrator.py` 的 `allocate` 原样：30 → 6 道、每道 5）。这一句替上一次登记修订 8 第 8.4 节步表末行「step-0、1、2、2b、7 都绿就起、与还在跑的单元共用本机」的起跑闸（跑法，判据不动）。依据（推的，用修订 8 第 8.4 节推的用时：第 7、9 步 14 582 秒，第 3 步带流三两项 9 698 秒与 8 128 秒）：照旧闸，第 7 步绿时第 3 步参考文件系统那一项还剩约 3 244 秒没跑完、占着 10，逐行编排只分得到 20（4 道），约 9.5 小时的逐行要拉长约一半；串在后面只多等那一项收尾，约 0.9 小时。
   - **分到的数怎么落到子进程里**：单元与逐行编排各经 `bash scripts/hosts/capped.sh <分到的数> <命令>` 起（`within_share`；`capped.sh` 文件头「套在另一层 capped.sh <N> 里面的 default 取到的是 N」），里面的装置、驱动、执行器读到的 `SINGLEFS_THREAD_CAP` 与各线程变量就是分到的数。这一条替修订 5 第 5.5 条「单元的线程不再由脚本里的 `capped.sh "$THREADS"` 设，取服务给的每台默认重活线程数」那一句：第 5.5 条删掉的单元脚本里那七处包装不加回，单元整条由编排套一层；单元脚本只改文件头一句说明。
   - **第二段**（一条任务；要两台时交编排层，服务每台一片，各片是服务的另一条子任务、额度各自是登记值）：每台槽数 × 每槽线程 = ⌊额度 / 5⌋ × ⌊额度 / 槽数⌋ ≤ 额度（`allocate`，原样）；`launch_local_two` 与 `launch_peer_two` 给每个驱动 `--lanes 1 --threads <那一槽的线程>`，驱动照它给编译、装置、执行器套 `capped.sh`（冻结的 `row_driver.py` 不改）。这两处只是撤回 hold，线程设法本来如此，这一段补单测与变异（第 7.4 条）。
   - **(b) 不拆的依据**：读数单元各分 10，不吃满一台；第一段入口照第 3.5 条不分片，逐行编排在那一条任务里串在单元后面，独占它的额度；第二段已经是服务分片，每台一片就是 (b) 那一种。没有哪一个单元要吃满一台而又与别的单元并着起。

7.3 **T 的量法（主 agent 定，第 2 条）**：第 7 步单元分 10、经 `capped.sh 10` 起，T 是它的进程挂钟；逐行驱动每道 5；10 : 5 与 r1 修订 6 跑法同，逐行限时 max(1800 秒, 3T) 照原式。第 7 步与第 9 步、第 3 步带流三那一项照修订 8 第 8.4 节同时起（额度 31 放得下三个；自证格 `allocation-threads`、`e2e-two-rounds` 恢复修订 6 之前的写法）。读数单元记它实际分到的线程：单元编排日志「▶ 单元 … 起：10 线程」、修订 11 那一层入库记 `threads=10`（`item.unit.threads`）。单测钉比例：`test_thread_shares.py` 的 `test_step_7_gets_ten_threads_and_a_row_lane_five_so_the_ratio_behind_the_row_time_limit_holds`（第 7 步 10、每道 5 且等于 `rows_orchestrator.LANE_THREADS_MINIMUM`、额度 31 分给逐行编排后是 6 道 × 5）。

7.4 **单测**：新模块 `test_thread_shares.py`（12 条）：第一段——第 7 步 10 与每道 5 的比例、32 个读数单元都分 10 与两个小单元分 1、逐行编排去余数（31 → 30、14 → 10，且 14 不去余数那一道是 7）、额度照 `SINGLEFS_THREAD_CAP`（设 12 得 12）与没设照登记文件、额度 31 时整张单元表排出来合起来峰值不超 31 且链尾那一刻起的恰是第 7、9 步与第 3 步带流三那一项、余数只给小单元、逐行编排的起跑闸、单元与逐行编排各经 `capped.sh <分到的数>` 起；第二段——本机与转交到别的台的驱动各恰一个 `--lanes 1 --threads 5`（额度设成 31，驱动拿到额度而不是那一槽的 5 就红）、一台上各槽合起来不超过它的线程（5 到 64 逐个）。`test_revision_8.py` 删上一段加的两条（读数单元记登记值、单元套 hold），`test_a_unit_does_not_fit_when_the_threads_run_out` 恢复修订 6 之前的写法。上一段那份账的单测随文件删。

7.5 **第四张变异表**（`scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_threads_and_contrast.tsv`）：删 R2P-18 至 R2P-32（锚在已删的账与 hold 上，没跑过）；R2P-33 至 R2P-38 照留；加 R2P-39 至 R2P-48（读数单元照登记值分、逐行编排不去余数、逐行编排照旧闸起、单元不经 `capped.sh`、逐行编排照整份额度起、分到的数不落到子进程、余数恰好放得下小单元也不给、额度不认 `SINGLEFS_THREAD_CAP`、第二段本机驱动照额度起、第二段转交的驱动照额度起），共 16 条；单测模块换成 `test_thread_shares`、`test_revision_8`，`replay.sh` 的驱动 `driver_e171_second_rerun_current_state_threads_and_contrast_research_mutations` 同改。`rows_orchestrator.py` 里的分法（`allocate`）不进表：`test_revision_8` 载入的 `rows_orchestrator_revision_8.py` 按 sha256 钉它，改它一处整份单测退 3，红得不是因为那一处；它由 `test_thread_shares` 的两条用例与 `rows_orchestrator.py` 自证的 `allocation-*` 格盯。产物名照第 6.4 条（跑的那天不同照第 1.12 条改）。修订 8 那张表 25 条与另八张 E171 研究一侧表的锚点此刻都恰命中 1 次（草稿 `anchor_hits.py`，量的），表不改；它的产物名照第 6.1 条。

7.6 **自证（单元编排）**：开关 `rows-before-step-7` 换成 `rows-before-units-done`（逐行编排照旧闸起；红在 `e2e-order`、`e2e-threads-within-quota`）；`ledger-skipped` 删、加 `shares-not-applied`（单元与逐行编排不经 `capped.sh` 起；红在 `e2e-threads-within-quota`）；格 `e2e-units-wait-for-other-holders` 删，加 `e2e-threads-within-quota`（不设 `SINGLEFS_THREAD_CAP`，额度取登记值：每个单元里读到的上限是它分到的数、逐行编排读到 30、同一时刻合起来峰值不超登记值、逐行编排在最后一个单元收尾之后才起）；`e2e-order` 改成逐行编排不早于最后一个单元收尾；`e2e-no-room` 恢复。假单元与假逐行编排起头那一行多记一个 `cap=<SINGLEFS_THREAD_CAP>`。

7.7 **重钉与 H31**（量的，2026-11-02；草稿 `tools/repin_chain.py` 在副本里照依赖次序重钉，打 `fixpoint ok`，主工作区照它改完再现算一遍，十一处都已是不动点）：`rows_orchestrator.py` → `695fbc2b…`（钉在 `cross_host_o3_check.py`、`rows_orchestrator_revision_8.py`、`rows_orchestrator_segment_two.py`、`reuse_library.py`）；`cross_host_o3_check.py` → `3e727eab…`（`rows_orchestrator_revision_8.py`、`reuse_library.py`）；`rows_orchestrator_segment_two.py` → `e4894615…`（`rows_orchestrator_segment_two_revision_10.py`、`reuse_library.py`）；`rows_orchestrator_segment_two_revision_10.py` → `21c907ca…`（`rows_orchestrator_segment_two_revision_11.py`）；`rows_orchestrator_revision_8.py` → `4ac5505c…`（`rows_orchestrator_revision_11.py`）；`segment_one_units_revision_8.py` → `e30841dc…`（`segment_one_units_revision_11.py`）。`reuse_library.py` 只换三个钉值，存法那一块没碰。H31 回到十份（删上一段加的那一份）：五份变了（`reuse_library.py`、`segment_one_units_revision_11.py`、`segment_one_unit_revision_11.sh`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`），值等于此刻现算（`H31 files 10 mismatch []`，量的）。

7.8 **现状整表段起跑之前还差的**（替第 6.6 条那张清单，缺一样不起）：这一条修订与修订 6 的单测、证红、第四张表与修订 8 那张表的重跑、三份逐行编排与单元编排（修订 8、11）的自证、两段干跑；④ 要的修订 6 那一趟行产物照第 6.1 条从 `04ceaa904^` 取出、填 H15；拍树宣布行（主 agent 写）；照第 6.3 条拍树、编、拷二进制；H1–H9、H14、H16、H17、H22、H23、H25、H28、H33、H34 拍树那一刻照各自的表填；H31 在最后一次改动之后再核。

### 修订 8（2026-11-02，承接的执行员；现状整表段一份产物都没跑；这一条修订里的单测、证红、变异、自证与干跑都还没跑）

依据：派发提示（批 17 续）「关的是哪几条」三条，原句——第 1 条：用户定「项目中的其他脚本 其他命令 理论上应该根本不需要包装」（做统一任务服务的会话转）：「服务那一层的补丁换上之后，`run-with-memory-cap.sh`、`capped.sh`、`gpu-quiesce-run.sh`、`multi-host-run.sh --peer` 只认 task-server 起尝试时直接套的那一层，任务里面再调一律被拒；要卡的任务由服务在最外层套清场；`detached-run.sh` 整份删除」，「改法：删掉那一层直接起；线程照进程内分到的数设各装置读的线程变量（`CARGO_BUILD_JOBS`、`SINGLEFS_THREAD_CAP` 这一类，照 `scripts/hosts/capped.sh` 文件头列的那几个），内存由服务定。验收：E171 两个目录里 `grep -rnE 'capped\.sh|run-with-memory-cap|gpu-quiesce-run|detached-run|multi-host-run'` 只剩弄坏开关与自证里证它不再出现的那几处（逐处写明）；进程内分配那组单测照旧钉住合计、比例与传到子进程的线程变量」；第 2 条「冻结层照依赖次序重钉到不动点、H31 重填；改动写进登记修订 8（依据写用户那句与服务那一层的拒法）；受影响的变异表重整锚点」；第 3 条「`reuse_library.py` 那一处若在存法那一块里，写明、只动包装那一行，存法不碰」。服务那一层的拒法照那份改法清单（派发提示点名的转述）：任务里面再套一层，内存包装退 251，线程包装与清场包装退 2，多机包装退 255。读数依据（主 agent 转述，量的）：修订 7 那一段全绿——单测 4652 `Ran 205 tests … OK`；第四张表 `mutations=16 caught=16`、修订 8 那张表 `mutations=25 caught=25`，都 `restored_and_baseline_green=true`；15 份自证都「红 0」；两段干跑 4670、4671 只停在 H16 没填与冻结树不在。第一至十一节、修订 1–7 原文一字不改；第六、七、九节每一条的去留与报告方式不动；本条都是跑法，判据不动。

8.1 **任务里面不再套包装**：E171 两个研究目录里起子进程的地方一律直接起，不经线程包装、内存包装、清场包装与多机包装；文件头里写着「经脱离单元起」的起法改成整条命令交 `task-server/task submit --`。逐处：
   - 原目录 `scripts/experiment-apparatus/e171_three_oracle_collusion/`：`row_driver.py` 的编译、装置（`device_command` 不再带仓根与线程两个参数）、O3 阳性对照；`frozen_o3.py` 的执行器命令；`o3_bad_image_judge.py` 文件头一句。
   - 修订后目录 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`：`row_driver.py` 的编译、装置（`device_command` 同上）、O3 阳性对照；`frozen_o3.py` 的执行器命令；`peer_side.py` 预检里的离线编；`reuse_library.py` 的 RP2 那一趟装置（`selfproof_run`）；`segment_one_units_revision_8.py` 起单元与逐行编排；单元脚本 `segment_one_unit_revision_8.sh` 的四处；修订 6 那一趟第一段跑法 `run_segment_one_on_the_revision_6_tree_multi_host.sh` 的两处（装置一趟、O3 阳性对照）。只改文字的：`device_timing.py`、`fake_task_server.py`、`o3_bad_image_judge.py`、`shard_relay.py`、`rows_orchestrator.py`（文件头一句与配置缺项时的报错句）、`rows_orchestrator_segment_two.py`、`rows_orchestrator_segment_two_revision_11.py`、`segment_one_unit_revision_11.sh`、`segment_one_units_revision_11.py` 的文件头，与四份跑法脚本的起法一行。
   - 验收那条 `grep` 在两个目录里此刻只剩两处，都不是起子进程：两份 `freeze-path-only-changes.tsv` 第 4 行第 5 列，记的是冻结的 E170 执行器 `executor_main.py` 文件头当时那一句怎么改的（历史说明；那份执行器在 E170 的目录里，不归这一段）。两个目录里没有弄坏开关或自证点名这几样。

8.2 **线程怎么落到子进程里**（替修订 7 第 7.2 条第三点「单元与逐行编排各经 `bash scripts/hosts/capped.sh <分到的数> <命令>` 起」，与第二段那一点里「驱动照它给编译、装置、执行器套 `capped.sh`」那半句；分法、额度、比例都不动）：
   - 名单与设法各一份，在各自目录的 `frozen_o3.py`：`CHILD_THREAD_VARIABLES`（20 个，照 `scripts/hosts/capped.sh` 文件头「设的变量」与它的 `variable_names` 抄）、`thread_environment(线程, 环境)`（交回环境的拷贝，20 个都设成那个数）。放在 `frozen_o3.py` 是因为逐行驱动、代跑那一头、复用库本来就载入它，不新添模块，复用键的闭包清单与各占位的必登清单不用加文件。
   - 逐行驱动：编译、装置、O3 阳性对照各把 `--threads`（线道线程）设进环境；冻结执行器那一步把 `workers` 设进环境。代跑那一头的离线编：给了 `--threads` 设成它，不给照服务设的。RP2 那一趟装置设成 `--threads`。
   - 单元编排：`within_share(分到的线程, 环境)` 改成交回环境（20 个都设成分到的数），起单元时 `env=within_share(unit.threads, …)`；逐行编排的命令不带包装，环境另由 `rows_environment(额度)` 给（20 个都设成 `rows_threads(额度)`）。干跑那一行打成「（线程数变量都设成 N）$ nice -n 19 python3 -B …」。
   - 修订 6 那一趟的跑法脚本自己写死 `THREADS=10`，装置一趟与 O3 阳性对照经脚本里的 `with_threads` 设成它（名单从同目录 `frozen_o3.py` 读）。修订 8 那一份单元脚本由单元编排起，环境里已经是分到的数（等于 `THREADS`），只删包装。
   - 弄坏开关 `shares-not-applied` 的意思跟着变成「单元与逐行编排的环境里不设分到的线程」；红的格不变（`e2e-threads-within-quota`）。

8.3 **`reuse_library.py`**：改的那一行（`selfproof_run`，RP2 自证那一趟装置）不在存法那一块里（存法是算键、查库、入库、取用）；同一份里另有两处注释（文件头 `[env]` 那一句、`THREAD_COUNT_VARIABLES` 上面那一行）只换了说法、不再点名包装脚本，`THREAD_COUNT_VARIABLES` 那 8 个名字与 `environment_names_in` 一个字没动，键的算法不变。三个钉值照第 8.6 条重钉。复用库此刻是空的（`~/.local/share/singlefs/reuse/e171-r1` 下 0 个文件，量的），所以逐行驱动与 `frozen_o3.py` 变了带来的键变化不丢已入库的条目。

8.4 **单测**：修订后目录 `test_thread_shares.py` 12 → 16 条：起单元与起逐行编排两条改成核命令里没有包装、环境里 20 个线程数变量各是分到的数（第 7 步 10、Z1 那个小单元 1、逐行编排额度 31 时 30；这一条任务的额度设成 31，子进程拿到的若是额度就红）；新加名单本身（20 个、不重、含 `CARGO_BUILD_JOBS` 与 `SINGLEFS_THREAD_CAP`、交回的是拷贝）；新类 `RowDriverThreadsTests` 三条：编译、执行器、O3 阳性对照环境里是线道的 5；装置（一个打 `env` 的假装置经 `device_timing.py` 起）读到的是 5；RP2 那一趟读到的是给它的 10。`test_row_time_limit.py` 装置命令那一条改成核没有包装。十二个模块合计 209 条（`grep -c '    def test_'` 量的）。原目录：`test_collusion_pipeline.py` 装置命令那一条改写、加一条「假装置读到的 20 个线程数变量都是线道的 6」；`test_o3_columns.py` 执行器命令那一条改写、加一条「编译、执行器、O3 阳性对照环境里各是 5、3、4」；六个模块合计 128 条（量的）。

8.5 **第四张变异表**（`scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_threads_and_contrast.tsv`）：R2P-42、R2P-43、R2P-44 锚点改到新写法（单元的环境里不设分到的线程、逐行编排照整份额度设、`within_share` 不设）；加 R2P-49 至 R2P-55（设法不设、名单漏 `SINGLEFS_THREAD_CAP`、逐行驱动的装置与编译与 O3 阳性对照不带线道线程、冻结执行器不带、RP2 那一趟不带），共 23 条；单测模块照旧 `test_thread_shares`、`test_revision_8`。产物新名 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-threads-and-contrast-research-mutations-2026-11-02.out`。修订 8 那张表（25 条）锚点都还恰命中 1 次、表不改，但它盯的 `segment_one_units_revision_8.py` 改了，重跑，产物新名 `…-revision-8-tools-research-mutations-2026-11-02-r3.out`。`scripts/experiment-apparatus/replay.sh` 两个登记行改指这两个新名（跑的那天不同照第 1.12 条改）。原目录那张表（67 条）锚点都恰命中 1 次，不加行（它是第 1 次重跑之前那一趟的表，复跑逐字节比）。peer_side 预检里离线编那一处没有单测、不进表，写进第 8.8 条。

8.6 **重钉与 H31**（量的，2026-11-02；草稿 `repin_chain.py` 在副本里照依赖次序重钉到 `fixpoint ok`，主工作区照它改完、与副本逐字节相同，再现算一遍十一处都不变）：`rows_orchestrator.py` → `d21033a7…`（钉在 `cross_host_o3_check.py`、`rows_orchestrator_revision_8.py`、`rows_orchestrator_segment_two.py`、`reuse_library.py`）；`cross_host_o3_check.py` → `bdf03496…`（`rows_orchestrator_revision_8.py`、`reuse_library.py`）；`rows_orchestrator_segment_two.py` → `b156b7e0…`（`rows_orchestrator_segment_two_revision_10.py`、`reuse_library.py`）；`rows_orchestrator_segment_two_revision_10.py` → `24b82ae5…`（`rows_orchestrator_segment_two_revision_11.py`）；`rows_orchestrator_revision_8.py` → `3ff5bbfd…`（`rows_orchestrator_revision_11.py`）；`segment_one_units_revision_8.py` → `b2d4c89a…`（`segment_one_units_revision_11.py`）。H31 十份里五份变了（`reuse_library.py`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`、`segment_one_unit_revision_11.sh`、`segment_one_units_revision_11.py`），值等于此刻现算（`H31 files 10 mismatch []`，量的）。十张修订后目录的研究一侧表锚点每张都 `not_exactly_once=0`（量的）。

8.7 **自证**：格与开关都不加不删；`segment_one_units_revision_8.py` 的开关轮里 `shares-not-applied` 仍要红在 `e2e-threads-within-quota`。

8.8 **现状整表段起跑之前还差的**（替第 7.8 条那张清单，缺一样不起）：这一条修订的两个目录的单测、第四张表（23 条）与修订 8 那张表的重跑、15 份自证、两段干跑；④ 要的修订 6 那一趟行产物照第 6.1 条取出、填 H15；拍树宣布行（主 agent 写）；照第 6.3 条拍树、编、拷二进制；H1–H9、H14、H16、H17、H22、H23、H25、H28、H33、H34 拍树那一刻照各自的表填；H31 在最后一次改动之后再核。代跑那一头预检离线编的线程设法只由读代码核过（没有单测与变异）。

### 修订 9（2026-11-02，承接的执行员；收尾：现状整表段不跑；这一条修订写完时现状整表段一份产物都没跑，这一条没有单测、变异与自证要跑）

依据：派发提示（批 17 续，收尾）「关的是哪几条」第 1 条原句——「用户 2026-11-02 在弹窗里选「第 2 次重跑收尾，整表另立新实验」（问句：「E171 第 2 次重跑的现状整表段，起跑前的「拍树前五项核对」照现在的代码与登记会退 1（推的…）。这一段整套沿用 r1 修订 8 的冻结层，卡口六处：B1、B2 判法还认 r1 的「合后清单」与「修订 9」；B3 要宣布那一刻的草稿二进制；B4 要把仓里 ab35105f 那版装置逐块对到今天的装置（执行员量的：差 191 段、833 行）；B5 要修订 6 第二段的行输出根，已不在；B6 要修订 6 树，造不回来（E200 量的）。Q1 撤项那一格已判甲…，Q2 已判「有」；整表段只补 Q1 的逐格对比与归因。整表段怎么办？」；选项原文「E171 第 2 次重跑照已有读数收尾（Q1 撤项判甲、Q2 有，整表段记「冻结层依赖的 r1 输入造不回来，不跑」）；在今天的树上整表重跑另写一份不继承 r1 冻结层的跑前登记，先设计、后跑。」）。写登记修订 9（收尾，连同 B1–B6 原文与上一段报告出处）；问题单两行状态改用户已定（Q1 撤项那一格判甲、整表逐格对比与归因留给新实验；Q2 有）」。读数依据：上一段执行员的报告 `~/.local/share/singlefs/work/compat-removal-e171-r2-run-j/result.md`「卡口」一节第 135–153 行（逐字抄在第 9.3 条）；那份报告里「拍树前五项核对退 1」是读代码推的（那时没有宣布行、没有树，没有跑过 `before-snapshot`），B4 的 191 段、833 行与 B5 的「不在」是执行员量的，B6 是 E200（修订6近似树重跑） 页量的。第一至十一节、修订 1–8 原文一字不改；第六、七、九节每一条的去留与报告方式只动第 9.1 条点名的「现状·逐格」一行，依据是上面这句里「整表段记「冻结层依赖的 r1 输入造不回来，不跑」」与「整表逐格对比与归因留给新实验」；判据一个没动，没有收严也没有补臂：这一条只记不跑、不新增要跑的量。

9.1 **现状整表段不跑，逐条记（跑法与报告方式；判据不动）**：

| 登记里的条款 | 这一次的状态 |
|---|---|
| 第五节 5.6 分段表「现状整表段（第二段）」 | 不跑：冻结层依赖的 r1 输入造不回来（第 9.3 条） |
| 第六节「现状·可取行」 | 已跑，在撤项核算段：名单核对产物 `…-2026-11-01-list-check-2026-11-01.out` 第 37 行 `E7RESULT name=current_rows rows=278 rows_not_hit_once_by_name_or_anchor=0` |
| 第六节「现状·逐格」 | 主 agent 定不跑（第十一节「够判停机」末句两种写法里的这一种：问题单 Q1 够判条件字面的后半「与 r1 逐格对比」没满足，不是够判之后没必要）；用户定的，逐格的差与归因（被撤、改引、树变了）这一次没有读数，留给整表另立的新实验 |
| 第六节「附带（够判后不跑）」 | 照原条款，没跑 |
| 第八节「树」一行（旧树读数与今天的树读数逐格并列） | 只有旧树那一边的读数；今天的树那一边没有 |
| 第十一节 Z13、S22，与沿用的上一次登记停机 S1–S3、S5–S13、S16、S17 | 现状整表段不起，不触发也不判；S16 的宣布行不写（`records/2026-10-31-删兼容旧版本代码.md` 里没有行首「- E171 第 2 次重跑拍树宣布」的行） |
| 第十一节比对对象表 #5、#6、#7（现状整表段逐格比、逐行归因） | 不比。机读登记 `.claude/gate.d/replay-comparison-units.tsv` 第 48–50 行的「不登记的比对对象」三行与登记表逐行对，登记表原文不改，所以那三行留着 |
| 修订 5 第 5.9 条、修订 6 第 6.6 条、修订 7 第 7.8 条、修订 8 第 8.8 条「现状整表段起跑之前还差的」 | 不再执行；原文照留 |
| 第十四节现状整表段的挂钟与核数 | 不用 |

Q1 撤项那一格与 Q2 不动：Q1 撤项那一格仍是 Z12 重跑之后三份撤项核算的读数（实验页第 9 条，三份判决行都是 `q1_judgement=jia`），写结论仍照第六节「写结论时」那一句，不写成「E171 在今天的实现上结论不变」；Q2 仍是名单核对产物第 20 行 `E7RESULT name=q2 directories_agree=true not_found_rows=33,70,107,141,168,236 not_found_besides_row_236=5 verdict=yes`。

9.2 **问题单两行的最终状态**：Q1 撤项那一格判甲（用户定）；Q1 够判条件字面的后半（整张名单在今天的树上跑完，与 r1 逐格对比并归因）这一次不做，留给整表另立的新实验，新实验另写一份不继承 r1 冻结层的跑前登记，先设计、后跑；Q2 有（用户定）。两行都不再开着。

9.3 **六处卡口原文**（上一段报告「卡口」一节第 137–153 行逐字；原文里的「第 2 条 b」「第 2 条 d、e」指那份报告第 2 条（占位）的子项，「清单第 N 步」指那份报告里一份 16 步的起跑清单，那份清单只在报告里，没进仓，也不进登记；原文里各处的「量的」标注照留）：

```text
- **B1 宣布行与第 2 条 b**：`frozen_tree_lines_revision_8.py` 第 405–418 行的第 2 条 b 要三样：`H1.after_sha256_files_in_merge_order` 不空、每份 after 清单都在宣布行里点名、仓里文件等于清单里最后一份的 sha256。
  - r2 登记第十一节末段（第 287 行）写的是「三条线各自最后合入的批号那一项不适用，改写 HEAD 与 git status 全文」，判法没跟着改。
  - 照 run-g 第 5 组第 0 步那种宣布行，b 判 false，`before-snapshot` 退 1。
  - 出路二选一：(a) 写一条登记修订，判法照修订改（第 2 条 b 在 r2 下记不适用，或改核 HEAD 与 status），这连着重钉一串、H17 重填；(b) 不改判法：宣布那一刻 `sha256sum` 几份要钉的文件进一份清单，放在认领的目录里，宣布行里用 `~/.local/share/singlefs/work/…` 那种写法点名它（判法认 `~` 的写法，第 409 行）。(b) 也改了宣布行写什么，要写进登记修订。
- **B2 第 2 条 e 与 H6 要 r1 登记里有「### 修订 9」**：`frozen_tree_lines_revision_8.py` 第 89–90 行 `PREREGISTRATION = "research/prompts/e171-r1-prereg.md"`、`REVISION_9_HEADING = "### 修订 9"`。
  - r1 登记里这一行 0 处（`grep -c '^### 修订 9'` 得 0，量的）：修订 8 那一趟从没拍过，修订 9 没写过，标题从修订 8 直接到修订 10。
  - r2 下与它对应的是 r2 修订 2（S20 重钉）。不改判法，第 2 条 e 与 H6 都过不了。
  - 出路：写修订，判法改读 r2 修订 2；或在 r1 登记补一条修订 9，照抄 r2 修订 2 的常量。要不要动 r1 登记由主 agent 定。
- **B3 草稿副本**：第 2 条 d 要草稿副本里 `target/release/` 下两个二进制都在、可执行、且比副本里装置源码新。e 要那份副本上的 `stream-constants` 产物甲类全过（r1 修订 8 第 8.2 节「怎么取」：宣布之后、拍树之前，在宣布那一刻的副本上跑；r2 修订 2 第 2.4 条照这句）。这一步 run-g、run-h、run-i 的清单里都没有；我在下面清单里补成第 4 步。
- **B4 H5 块表**：S16 ③ 要 `ab35105f…` 依次打上块表各块之后等于树里装置，每块来源只认三种（r1 登记第 1480 行）。
  - 今天的装置 `cbecfceb…` 与 `ab35105f…` 差 191 段、833 行（`diff | grep -c '^[0-9]'`、`grep -c '^[<>]'`，量的）。
  - 改动来自 `f8a9ddaab` 之后 4 个提交（`ff7e45d01`、`0b926bd3f`、`df5866f67`、`d6dd6da42`），外加这一批暂存的一处（72 行加、84 行删）。
  - 这些块大多只能归「主 agent 认定点名的别的装置改动」，要主 agent 逐块认、写批号与改了什么。没有这张表，第 0 步与 start 阶段的 H5 都不过。块的 fixture 照第 1481 行用 `diff -u --label` 出、头两行不带时刻。
- **B5 H28（第二段对照）**：`check_h28` 要修订 6 那一趟第二段的行输出根是个目录。
  - `scripts/experiment-apparatus/target/e171-r1-dumps` 与 `<工作根>/e171-r1-segment2-multi-host` 都不在（`ls` 报不在，量的）。
  - 第二段对照照现在的判法出不来，要像 r2 修订 6 第 6.1 条处理第一段那样，另写一条修订。
- **B6 H33、H34**：H33 是「修订 6 树对修订 8 树」的复用计数，r1 登记第 2293 行要修订 6 树。r2 修订 6 第 6.1 条已记修订 6 树造不回来，H33 照字面出不来。H34 由主 agent 照 H33 定；库此刻是空的，r1 登记第 2468 行写「不导也不影响作数」。
```

量法（该执行员的报告；这里照量的、推的分开记）：B1–B3 是读代码与登记推的（没有宣布行，没有冻结树，`before-snapshot` 没有真跑过）；B4 的 191 段、833 行与 B5 的「不在」是执行员现查量的；B6 的「修订 6 树造不回来」出自修订 6 第 6.1 条（主 agent 转述的现查：全部提交与 reflog、stash、悬空对象、第二台整盘与本机整盘，都没有修订 6 用的文件）与 E200（修订6近似树重跑） 页（照 git 拼的两棵近似树都停在停机 S40）。六处里 B5、B6 是 r1 的输入已不在，输入补不回来，只能像修订 6 第 6.1 条处理第一段对照那样改记「造不回来」；B1–B4 改得动，但要连着改冻结层的判法、重钉一串钉值再重填占位，用户的选项把这一整套沿用 r1 冻结层的做法一并收了。

9.4 **收尾之后不再用的东西，逐项去向**（这一段做完的；判法：有活的读者就留作已冻结的装置，没有读者的过渡状态撤掉）：

| 东西 | 去向 | 依据 |
|---|---|---|
| 上一段填进 `revision_8_placeholders.json` 的 H17（15 份文件的 sha256） | 撤回，`files` 改回 `null`；文件与暂存区那一版逐字节相同（sha256 `62953b70…`，`git show :<路径> \| sha256sum` 量的） | 读它的只有 `revision_8_placeholders.py` 的 `check_h17`（第 346 行），起跑前 `--phase start` 才调，这一段不跑，所以没有读者；自证格 `h17-files`（第 515 行）用假配置，不读它；模板的约定是「值先全是 null；拍树那一刻由执行员与主 agent 照表填」（同文件第 3 行）。那 15 份文件任何一份再改，这 15 个值就过期，没有读者的过期钉值是过渡状态。15 个值留在上一段的报告里，不进仓 |
| 其余占位 JSON：`revision_8_placeholders.json` 的其余各节、`revision_10_placeholders.json`、`revision_11_placeholders.json` | 留着，作已冻结装置的模板 | 活的读者：`revision_8_placeholders.py` 第 470 行 `template()`（自证格 `template-shape`、`unfilled-named` 载入）、`rows_orchestrator_revision_8.py` 第 269、300 行、`revision_10_placeholders.py` 第 539 行 `template()`，门禁 `.claude/gate.d/code-tooling.sh` 里有这两份 `--selftest` 的命令行（`grep -n 'revision_8_placeholders.py --selftest\|revision_10_placeholders.py --selftest' .claude/gate.d/code-tooling.sh` 各命中一行）；`revision_11_placeholders.json` 是 `run_segment_one_revision_11_multi_host.sh` 第 17 行与 `run_segment_two_revision_11_multi_host.sh` 第 19 行不给配置时的默认 |
| `revision_8_research_side_of_the_revision_6_run.json` | 留着 | E200（修订6近似树重跑） 的 `build_approximate_tree.py` 第 56 行读它 |
| 那份 16 步起跑清单 | 不进仓、不进登记 | 它只在上一段的报告里；清单是为现状整表段写的执行步骤，整表另立新实验之后由新登记写自己的步骤，抄进仓就是给不跑的东西留一份可执行的样子。修订 5 第 5.9 条、修订 6 第 6.6 条、修订 7 第 7.8 条、修订 8 第 8.8 条的「还差的」清单原文照留，状态见第 9.1 条 |
| 已冻结的装置整体：三份占位检查器、冻结层判法 `frozen_tree_lines_revision_8.py`、逐行编排与单元编排、复用库、对照脚本、跑法脚本、单测模块、研究一侧变异表、自证 | 留着，不删 | 活的读者：门禁 `.claude/gate.d/code-tooling.sh` 里那两份 `--selftest` 的命令行与 `.claude/hooks/selftest-registry.tsv` 里 E171 研究目录的 14 行（`grep -c 'e171_three_oracle_collusion_after_the_oracle_fixes' .claude/hooks/selftest-registry.tsv` 得 14，都登记为 light）的 `--selftest`；`scripts/experiment-apparatus/replay.sh` 里 E171 研究一侧变异的复跑行，它们的产物是实验页第 10–12 条整行引的读数；E200 读其中一份 JSON（上一行）。删它们要连着改门禁表、自证登记、复跑登记、实验页引用与冻结层钉值，是另一件事，不在这一段，写范围也不含那些表。新实验的跑前登记沿用其中哪几件、不沿用哪几件，由它写 |
| 上一段留在工作根草稿目录的东西：`originals/`、`tools/`、`device-ab35105f.rs`（`git show f8a9ddaab:` 取出的装置，供 H5 块表用）、`rev12-stream-constants.out`、`h17-files.txt`、`h25-files.txt` 等 | 不在仓里；交主 agent 删 | H5 块表、H17、H25 都随现状整表段不跑而没有用处 |
| `scripts/experiment-apparatus/replay.sh` 里 E171 第 2 次重跑的复跑登记行（第 1923–1932 行，十行） | 不动，核对得上 | 十行的驱动函数都在 `replay.sh` 里定义，期望产物都在 `research/results/` 里；其中第 1923 行 `driver_e171_r1_revision_12_stream_constants` 指 S20 重钉之后的 `…-2026-11-01-stream-constants-r2.out`，第 1920 行指修订 8 那张表的 `-r3`；命令与原样输出在实验页第 13 条 |
| r1 登记 `research/prompts/e171-r1-prereg.md` | 没动 | B2 的出路之一是在 r1 登记补一条修订 9，这一段没做；整表另立新实验，不再需要 |

9.5 **岔路表**（第十一节「够判停机」要的那张；问题单 `research/prompts/e171-r2-questions.md` 两行）：

| 行 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|
| Q1 撤项那一格 | 已够判：甲（用户定） | 不能（读法 8：现状整表段的读数不并进撤项的判定词） | `grep -n 'name=verdict' research/results/e171-r2-*-removal-effect-on-the-*-2026-11-01.out` |
| Q1 够判条件字面（整张名单在今天的树上跑完，与 r1 逐格对比并归因） | 关了：用户定这一次不做，留给整表另立的新实验；不是够判 | 这份登记里没有剩下的量；这一格没有读数 | 没有：新实验另写跑前登记 |
| Q2 | 已够判：有（用户定） | 不能 | `grep -n 'name=q2 \|name=disposition' research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check-2026-11-01.out` |

没有开着的行。

### 修订 10（2026-11-06，实验设计员；第 2 次重跑已照修订 9 收尾、现状整表段不跑；这一条只改已冻结装置里两份单元脚本取线程的写法，写这一条时装置一处没改、单测与门禁一条没跑）

依据：
- 用户 2026-11-05 原话，`records/2026-11-05-删轻型并入重型.md` 第 338 行整行：「- 用户：「除了变异哪个 需要用线程来控制内存大小的。 其他的全部应该给 31线程」「变异给 31线程的话 内存oom ，因此限制了变异的线程。但是其他的不是这样分配的内容不应该这样做。」」
- 登记位 `.claude/heavy-thread-default` 文件头第 2 行与第 4 行：「# 取法：shell 经 scripts/hosts/heavy-threads.sh 的 heavy_thread_default，python 经 scripts/hosts/heavy_threads.py 的 default_heavy_threads，」「#   SINGLEFS_THREAD_CAP 设了照它，脚本的参数给了照参数。双机时每台各取这一个数。」
- 门禁 code-tooling 的 heavy-thread-defaults 格，任务 8521（`task-server/task log 8521 --all`）点名的两行原样：
  「scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_unit_revision_11.sh:275  读自己的线程变量 THREADS：capped.sh 不设它，读它的那条语句里也没回退到 SINGLEFS_THREAD_CAP 或登记值  「--lanes 1 --threads "$THREADS" --output-root "$REPO/$DUMPS/empty-control-$slug$TAG" "$@"」」
  「scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_unit_revision_8.sh:224  读自己的线程变量 THREADS：capped.sh 不设它，读它的那条语句里也没回退到 SINGLEFS_THREAD_CAP 或登记值  「--lanes 1 --threads "$THREADS" --output-root "$REPO/$DUMPS/empty-control-$slug$TAG" "$@"」」
  同一格另七处点名的是别的实验装置、验证包里两份测试与排除表，不归这一条。
- 派发提示（批次 `records/2026-11-05-删轻型并入重型.md#并包`）：用户 2026-11-06 弹窗定「另开登记修订」，这一件只写登记修订、不起重跑。

第一至十一节、修订 1–9 原文一字不改；判据、阈值、作废与停机条款一个没动，第六、七、九节每一条的去留与报告方式不动；没有收严判据，也没有补臂。本条是跑法（装置取线程的写法），外加一条补实现的单测（第 10.6 条）。现状整表段照修订 9 第 9.1 条仍不跑，这一条不重开它。

10.1 **这一条替掉的原句**（原文照留，只对下面两份单元脚本取线程那一处与它们的文件头失效）：
   - 修订 5 第 5.5 条末句：「修订 8 单元脚本 `segment_one_unit_revision_8.sh` 不改（修订 11 单元编排起的是修订 11 那一份）。」
   - 修订 7 第 7.2 条第三点末句：「单元脚本只改文件头一句说明。」
   - 修订 8 第 8.2 条第四点末句：「修订 8 那一份单元脚本由单元编排起，环境里已经是分到的数（等于 `THREADS`），只删包装。」
   - 研究目录 `segment_one_units_revision_11.py` 文件头第 5 行「segment_one_units_revision_8.py 与单元脚本 segment_one_unit_revision_8.sh 一个字节不改」（出自第 1 次重跑登记修订 11 第 11.10 节「修订 8、10 的文件一个字节不改」）：对单元脚本那一半，这一条之后不再成立。

10.2 **改什么**（两份同改，执行员照做）：
   - `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_unit_revision_8.sh`（此刻 376 行，sha256 `58981365aea4a056a5c5aa819db58c5a1bbde27f61daaea91b77ee8fb1b7e81e`）与同目录 `segment_one_unit_revision_11.sh`（此刻 457 行，sha256 `3fa8f3b8f85b47c69015b4fea2daa61c6490090a3933eed0176f7c9b2c800435`）：在 `UNIT=${1:?要单元名}` 与 `cd "$REPO" || exit 2` 两行之间（两份都是第 17、18 行）插两行，字面照抄：

     ```bash
     source "$(dirname "${BASH_SOURCE[0]}")/../../hosts/heavy-threads.sh" || exit 2
     THREADS="$(heavy_thread_default)" || exit 2
     ```

     第二行后面可以接一句行尾注释（写明「取每台默认重活线程数；单元编排起的单元里 SINGLEFS_THREAD_CAP 就是它分到的数」），注释不许写在这一行前面、也不许把 `|| exit 2` 换成别的写法（理由在第 10.3 条第三点）。
   - 读 `$THREADS` 的几处一个字不改：修订 8 那一份第 224 行（第 4 步十个空变异对照的逐行驱动 `--threads`）、第 264 行（第 5、8 步冻结执行器的工作线程）；修订 11 那一份第 275、321 行（同上两处）与第 431 行（RP2 那一趟的 `--threads`）。插两行之后行号各加 2。
   - 两份文件头跟着改成真话：修订 8 那一份第 5–6 行「判法、阈值、线程一个字不动（…线程数变量由单元编排设成这个单元分到的线程，等于 THREADS…）」、修订 11 那一份第 4 行「判法、阈值、线程那几行一个字不动 … 里面各进程读到的上限就是 THREADS」，改成「判法、阈值一个字不动；线程取每台默认重活线程数（`scripts/hosts/heavy-threads.sh` 的 `heavy_thread_default`，第 2 次重跑登记修订 10），单元编排起的单元里 SINGLEFS_THREAD_CAP 是它分到的数，取到的就是那个数」这一意思；两份第 13 行「环境：」里的 `THREADS` 注明「编排照旧给，这一份不读」。
   - `segment_one_units_revision_11.py` 文件头第 5 行那半句改成真话：括号里接一句「修订 10 把两份单元脚本取线程那一处改成取每台默认重活线程数」。代码不改。
   - 不改的：`segment_one_units_revision_8.py`、三份逐行编排、`reuse_library.py`、`frozen_o3.py`、`row_driver.py`、四份跑法脚本、各张变异表。

10.3 **为什么这样改**：
   - **取到的数与改前在真跑里相同**（读代码，量的是文件内容）：单元编排起单元时 `segment_one_units_revision_8.py` 第 424 行设 `THREADS=str(unit.threads)`，第 448 行 `env=within_share(unit.threads, environment)`；`within_share`（第 375–377 行）调同目录 `frozen_o3.py` 第 302–306 行 `thread_environment`，把第 51–55 行 `CHILD_THREAD_VARIABLES` 二十个变量（含 `SINGLEFS_THREAD_CAP`）都设成分到的数；`heavy_thread_default` 在 `SINGLEFS_THREAD_CAP` 设了时照打它（`scripts/hosts/heavy-threads.sh` 文件头「判法」）。所以单元里改前读到的 `THREADS` 与改后取到的值，都等于这个单元分到的数（第 4、5、8 步与 RP2 都是读数单元，额度 31 时各 10，`unit_table` 第 126 行）。这也就是用户那句里「给 31 线程」在单元这一层的样子：任务拿到登记的 31，单元里不再有脚本自己的线程变量，取的都是登记位那一条路。
   - **为什么不留 `THREADS="${THREADS:-$(heavy_thread_default)}"`**：那样照旧读脚本自己的线程变量，派发提示与门禁的出路都是取登记位；外面给了 `THREADS` 就盖过登记值，与 `.claude/heavy-thread-default` 第 4 行只认 `SINGLEFS_THREAD_CAP` 与脚本参数不合。
   - **为什么是这个字面**：门禁那一格把「同一份文件里在行首赋过值、右边不读它自己」的 shell 变量当文件内的局部变量，读它不算读环境（`.claude/gate.d/code-tooling.sh` 第 4610 行注释，正则在第 4870–4871 行 `SHELL_ASSIGNMENT`）。设计时拿那两行正则对第二行试过：认出 `name='THREADS'`、`rhs='"$(heavy_thread_default)" '`、右边不含 `THREADS`（量的，第 10.8 条第二条命令）。写成 `if ! THREADS=…`、`local THREADS=…` 以外的前缀、或把赋值放进函数里，都可能不再算行首赋值，照红。
   - **为什么照脚本自己的位置 source、并且在 `cd "$REPO"` 之前**：单测与自证给的 `REPO` 是假仓（`test_revision_8.py` 第 338 行 `REPO=self.scratch`），里面没有 `scripts/hosts/`；`heavy-threads.sh` 按它自己所在的位置往上两级找登记文件（文件头「判法」）。`BASH_SOURCE[0]` 可能是相对路径，`cd` 之后就指错。研究一侧变异驱动的副本（修订 5 第 5.2 条）里 `scripts/hosts` 链回真仓，照样取得到。
   - **为什么取不到就退 2**：`heavy_thread_default` 拒的时候自己往标准错误写 ✗ 与 →、返回 2（`heavy-threads.sh` 文件头）；单元退非 0 就是这个单元红（两份文件头第 3 行），不退回别的数。

10.4 **对已有产物、T 与逐行限时的影响**：
   - **已有产物与结论：没有影响**。`research/results/` 里没有一份产物出自这两份单元脚本：第 2 次重跑的现状整表段没跑（修订 9 第 9.1 条），第 1 次重跑修订 8 那一趟从没跑过（第四节第 4 条），修订 11 是叠在那一趟上的一层。第 1 次重跑修订 6 那一趟的产物出自 `run_segment_one_on_the_revision_6_tree_multi_host.sh`（脚本里自己写死 `THREADS=10`，修订 8 第 8.2 条第四点），这一条不碰它。第 2 次重跑的 Q1 撤项那一格（甲）与 Q2（有）出自名单核对与三份撤项核算（修订 3 第 3.1 条、修订 9 第 9.5 条），与单元脚本无关。
   - **T（修订 7 第 7.3 条）：不变**。T 是第 7 步那个单元的进程挂钟；第 7 步走 `device_step`，不读 `$THREADS`（读它的只有第 10.2 条列的第 4、5、8 步与 RP2），装置读的是编排经 `within_share` 设进环境的线程数变量（10）。这一条不碰编排，第 7 步分到 10、经同一条路落到装置，量法照旧。
   - **逐行限时 max(1800 秒, 3T)：不变**。逐行驱动每道 5 线程由逐行编排定（修订 7 第 7.2 条第二点 `rows_threads`、`allocate`），不经单元脚本；T 不变，比例 10 : 5 照旧。
   - **真跑里第 4、5、8 步与 RP2 读到的数：不变**（第 10.3 条第一点）。
   - **会变的只有不经 `within_share` 起单元脚本的地方**：
     1. 单元编排的 `plan`（`segment_one_units_revision_8.py` 第 701–705 行）只用 `unit_environment` 的环境、不套 `within_share`。改前各单元打出的命令里第 4 步驱动 `--threads`、第 5、8 步执行器的工作线程、RP2 的 `--threads` 是 10（`THREADS`）；改后是 `plan` 那一条任务自己的 `SINGLEFS_THREAD_CAP`（经服务起是登记的 31），与真跑分到的 10 不同。`plan` 第 671 行那句「每个读数单元分 10 线程」照旧。`plan` 只打命令、不出读数，不进任何判据；要它打得与真跑一样，得改 `segment_one_units_revision_8.py` 并连着重钉，不在这一条（第 10.7 条第 2 项）。
     2. 自证与单测里直接起单元脚本、设了 `THREADS` 而没设 `SINGLEFS_THREAD_CAP` 的几处（`test_revision_8.py` 第 339 行、`segment_one_units_revision_11.py` 第 523 行的 `THREADS="1"`，`segment_one_units_revision_8.py` 自证格 `unit-script-commands` 经 `unit_environment` 给的 10）：改后取的是它们所在环境的 `heavy_thread_default`。读过它们的断言（`test_revision_8.py` 第 343–345 行、`cell_unit_script_commands` 的期望表、`real_script_step_1` 只交回退出码与读数），没有一条核单元脚本打出的线程数，推的：照旧绿；由第 10.8 条的命令坐实。
   - **研究一侧变异表的产物：推的不变**。变异驱动只在研究目录的 `.py`（不含 `test_` 开头的）里找锚点（`mutation_driver.py` 第 49–51 行 `source_files`），单元脚本不是变异对象；结果行只有 `outcome` 与 `unit_test_exit`（第 151 行），第 10.6 条加的用例不改哪一条变异抓没抓到。

10.5 **要改的文件与要重钉的值**（执行员改、重钉；值都在改完之后照 `sha256sum` 现算，写进这一节的接续修订或交回报告）：

| 文件 | 改前 sha256 | 改什么 | 钉它的地方与要重钉的值 |
|---|---|---|---|
| `segment_one_unit_revision_8.sh` | `58981365aea4a056a5c5aa819db58c5a1bbde27f61daaea91b77ee8fb1b7e81e` | 第 10.2 条两行与文件头 | 没有按 sha256 钉它的地方（研究目录、`replay.sh`、`.claude/gate.d/`、`.claude/hooks/`、`scripts/mutation/` 里 `grep` 改前 sha256 前 16 位 0 处）；`revision_8_placeholders.py` 第 74 行 H17 必登清单只列它的名字，`revision_8_placeholders.json` 的 H17 `files` 是 `null`（修订 9 第 9.4 条），没有值要重钉 |
| `segment_one_unit_revision_11.sh` | `3fa8f3b8f85b47c69015b4fea2daa61c6490090a3933eed0176f7c9b2c800435` | 同上 | `revision_11_placeholders.json` 第 18 行 H31 的这一项 → 改后的 sha256 |
| `segment_one_units_revision_11.py` | `67082bfbb0b2db000111b33b5e90ada23eac8c199d2a9c57897edd7877228e18` | 文件头第 5 行半句（第 10.2 条） | `revision_11_placeholders.json` 第 17 行 H31 的这一项 → 改后的 sha256；它不在 `reuse_library.py` 第 1895–1900 行 `PIN_SITES` 的被钉文件里，没有一串要连着重钉 |
| `test_revision_8.py` | （不钉） | 第 10.6 条的用例 | 不在 H31，也没有按 sha256 钉它的地方 |
| `revision_11_placeholders.json` | （不钉） | H31 两项 | 没有按 sha256 钉它的地方 |

   H31 其余八项不动：2026-11-06 现算十项都与登记值相同（量的，第 10.8 条第三条命令）。改完照同一条命令再算一遍，十项都 `same` 才算重钉完。`segment_one_units_revision_8.py`、三份逐行编排与 `reuse_library.py` 都不改，冻结层的六个钉值（修订 8 第 8.6 条）不动。

10.6 **补一条会红的单测（补实现，判据不动）**：`test_revision_8.py` 加一个用例类，起两份真单元脚本、`PRINT_ONLY=1`，环境照 `segment_one_units_revision_8.py` 自证格 `unit-script-commands` 的造法（`fake_world`、`settings_from_config`、`unit_table`、`unit_environment`；修订 11 那一份另给它要的 `REUSE_*` 几个），再把 `THREADS` 设成 10、`SINGLEFS_THREAD_CAP` 设成 7：
   - 两份的 `step-4-o1-comparison` 打出的驱动命令里有 `--threads 7`、没有 `--threads 10`；
   - 两份的 `step-5` 打出的 `frozen_o3.run_executor(…)` 那一条，按 `show` 的 `printf %q` 打法解开（`shlex.split`）之后工作线程那一位是 7；
   - 修订 11 那一份的 `reuse-selfproof` 打出的命令里有 `--threads 7`；
   - 另一格 `SINGLEFS_THREAD_CAP=abc`：两份都退 2、标准输出里没有一行以 `  $ ` 起头、标准错误里有 ✗。
   用例名写场景（例：`test_unit_scripts_take_the_thread_count_from_the_registered_default_not_from_their_own_variable`、`test_unit_scripts_stop_with_exit_2_when_the_registered_default_is_refused`）。
   **证红**：在一份仓副本里放新用例、单元脚本留改前那两份，跑这几条必须红：改前打的是 `--threads 10`、`abc` 那一格退 0。单元脚本是 `.sh`，研究一侧变异驱动改不到它，这一组的判别力靠这一次证红，不加变异行。

10.7 **这一条没定、交主 agent 的两项**：
   1. **用户那句管不管修订 7 的进程内分法**。用户原话的后半「变异给 31线程的话 内存oom ，因此限制了变异的线程。但是其他的不是这样分配的内容不应该这样做」，可以读成也管单元编排在一条任务里分额度的那一套：读数单元各分 10（`segment_one_units_revision_8.py` 第 95 行 `READING_UNIT_THREADS`）、逐行驱动每道 5、逐行编排分 5 × ⌊额度 / 5⌋（修订 7 第 7.2、7.3 条，主 agent 定的 T 的量法）。这一条不动它，单元里取到的仍是分到的 10。要改成每个单元各拿 31，是另一条修订：改 `segment_one_units_revision_8.py`（`segment_one_units_revision_11.py` 的 `UNITS_R8_SHA256` 钉它，要连着重钉、H31 重填），T 改在 31 线程上量，max(1800 秒, 3T) 依据的 10 : 5 比例没了，三个读数单元也不能在一条 31 线程的任务里同时跑。第 2 次重跑已收尾，这件只在这套装置再被用起来时才有后果。
   2. **`plan` 打出的线程与真跑不同**（第 10.4 条第 1 项）。要一致，把 `segment_one_units_revision_8.py` 第 702–704 行起单元脚本的环境也套 `within_share(units[name].threads, …)`，连着重钉 `UNITS_R8_SHA256`、重填 H31，重跑盯这份文件的两张研究一侧变异表（修订 8 那张与第四张）。这一条不做，只记下差别。

10.8 **执行员照做的次序与交主 agent 的验证**：
   - 次序：`python3 scripts/agents/copy-originals.py copy <草稿目录> <三份要改的文件> <test_revision_8.py> <revision_11_placeholders.json>` 留改前那几份 → 照第 10.2 条改两份单元脚本与 `segment_one_units_revision_11.py` 文件头 → `bash -n` 两份单元脚本 → 加第 10.6 条用例 → 照第 10.5 条重算 sha256、填 H31 两项 → 照下面第三条命令核十项都 `same`。
   - 交主 agent 的（执行员写进「要主 agent 跑的命令」，子 agent 不交任务）：门禁 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling 的 heavy-thread-defaults 格; 范围=1 格; 为什么=三oracle合谋测试第 2 次重跑登记修订 10 改了两份单元脚本取线程的写法，只核这一格不再点名这两份' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults`，判法：日志里没有一行点名 `segment_one_unit_revision_8.sh` 或 `segment_one_unit_revision_11.sh`（这一格可能因别的装置照旧红，那几处不归这一条）；研究目录里 `python3 -B -m unittest test_revision_8 test_revision_11 test_thread_shares`（`--cwd` 给研究目录），末行 `OK`；`segment_one_units_revision_8.py --selftest` 与 `segment_one_units_revision_11.py --selftest` 末行「红 0」；第 10.6 条的证红（仓副本里单元脚本是改前那两份，新用例红在 `--threads 7` 与 `abc` 退 2 那几条断言上）。
   - 不用跑的：现状整表段、两段干跑、`stream-constants`、各张变异表（第 10.4 条末项，推的不变；提交时照准入另判）。

**设计这一条时读过的文件与跑过的命令**（2026-11-06，HEAD `ff5dca7d22b1c6f95b2ac64a8c79e0467d580d1c`，研究目录与本登记 `git status --porcelain` 为空）：
- `.claude/agent-common.md`「规则怎么读」「写」「不做」三节；规则整份：`.claude/singlefs-ai-sop/rules/test-discipline.md`、`.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`、`.claude/rules/mutation-sampling.md`、`.claude/rules/implementation-first.md`、`.claude/rules/test-script-discipline.md`。
- 本登记第 1–639 行（整份）；`research/prompts/e171-r1-prereg.md` 只有 `grep -n '一个字节不改'` 命中的第 1751、1843、1981、2032、2347、2448 行；`research/prompts/e200-preregistration.md` 第 1–8、112–120 行与 `grep -n '线程'` 命中行。实验页、`research/results/` 下产物的内容都没读（只列了文件名）。
- `.claude/heavy-thread-default`（整份）、`scripts/hosts/heavy-threads.sh`（整份）、`scripts/hosts/capped.sh` 第 1–60 行里 `grep` 命中的第 8–38 行；`.claude/gate.d/code-tooling.sh` 第 4600–4625、5090–5160 行与 `grep` 命中的第 4764、4868–4873 行；`.claude/hooks/run-scope-guard.sh` 第 1–40 行；`.claude/hooks/agent-write-scope.tsv` 第 1–27 行；`.claude/hooks/selftest-temporary-service.tsv` 第 1–12 行；`.claude/gate.d/counted-claims.tsv` 第 1–15 行；`.claude/gate.d/doc-experiments.sh` 里 `grep '修订'` 命中行；`records/2026-11-05-删轻型并入重型.md` 第 330–350 行与 `grep` 命中行；`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 862 行（`grep` 命中）。
- 研究目录：两份单元脚本第 1–40 行与第 200–280 行附近、`grep 'THREADS'` 命中行与 `case` 表；`revision_11_placeholders.json`（整份）；`segment_one_units_revision_8.py` 第 120–130、370–380、415–430、690–712、975–1010 行与 `grep` 命中行；`segment_one_units_revision_11.py` 第 1–12、425–440、505–535 行；`frozen_o3.py` 第 51–58 行与 `grep` 命中的第 302–306 行；`test_revision_8.py` 第 325–360 行；`mutation_driver.py` 第 45–75 行与 `grep` 命中行；`reuse_library.py` 第 1868–1910 行与 `grep` 命中行；`scripts/experiment-apparatus/replay.sh` 第 3215–3265 行。
- 命令：`bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose …`；`task-server/task log 8521 --all`（落进草稿目录 `task-8521.log`，46 行）；`sha256sum`、`wc -l` 两份单元脚本与占位文件；`grep -rn '<两份单元脚本改前 sha256 前 16 位>'` 与 `grep -rln 'segment_one_unit_revision_8\.sh\|segment_one_unit_revision_11\.sh'`（研究目录、`replay.sh`、`.claude/`、`scripts/mutation/`、`crates/`、`research/prompts/`）；H31 十项现算（`python3 -I -c` 读 `revision_11_placeholders.json`、逐项 `hashlib.sha256`，十项 `same`）；拿门禁第 4764、4870–4871 行的正则在 `python3 -I -c` 里试第 10.2 条第二行（`'THREADS' '"$(heavy_thread_default)" ' False`）；`TZ=Asia/Tokyo date +%F`（2026-11-06）。没跑项目代码，没往统一任务服务交任务。

### 修订 11（2026-11-06，实验设计员；第 2 次重跑已照修订 9 收尾、现状整表段不跑；这一条改已冻结装置里单元编排与两段逐行编排在一条任务里分线程的写法，让单元编排的 plan 与真跑取同一个数，并给逐行限时的「超时」一类加一道读数门槛；写这一条时装置一处没改，单测、自证、变异与门禁一条没跑）

依据：
- 用户 2026-11-05 原话，`records/2026-11-05-删轻型并入重型.md` 第 338 行整行：「- 用户：「除了变异哪个 需要用线程来控制内存大小的。 其他的全部应该给 31线程」「变异给 31线程的话 内存oom ，因此限制了变异的线程。但是其他的不是这样分配的内容不应该这样做。」」
- 用户 2026-11-06 弹窗，同一份记录第 622 行整行：「- E171 修订 10 写好（e171-r2-prereg.md 第 588–672 行，两份分段脚本改取登记线程数，要重钉 H31 两项）。弹窗定案：修订 7 的进程内分线程（读数单元各 10、每道 5）「也改成 31，另开修订」，派设计员写修订 11（e171-thread-amendment-11），同时让 plan 与真跑取同一个数。两条修订齐了再派一个执行员一起改装置、重钉。batch-scope 里这几份是旧批次，本批的范围要加。」
- 登记位 `.claude/heavy-thread-default` 第 7 行：「31  # 用户 2026-10-12 定：所有重活默认每台 31 线程，双机两台各 31、合计 62，给了参数照参数；机器忙时也固定 31、不看忙闲（records/2026-10-12-里程碑三工具与治理批.md）」。准许按内存上限定并发的入口表 `.claude/memory-bounded-parallelism.tsv` 非注释行只有 1 行（`.claude/gate.d/crates-mutation-replay.sh`）。
- 修订 7 依据里转述的服务那一层的写法（第 468 行）：「(b) 每个单元各要吃满一台的，拆成服务的分片子任务 …；串着跑也合规」。
- 修订 10 第 10.7 条交主 agent 的两项（前一件设计员的报告 `~/.local/share/singlefs/work/e171-thread-amendment/result.md`「待主 agent 认的项」第 1、2 条），这一条把两项都定了。

第一至十一节、修订 1–10 原文一字不改。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」的三步：没有失败条款打中，「记一次输」不适用；各行判甲判乙的条件、阈值、作废条款与已有的停机条款一个不动；收严两处（第 11.4 条读数门槛、第 11.5 条停机条款），各自写明收严在哪；其余是跑法（修订 7 自己把线程分法记为「本条都是跑法，判据不动」）。现状整表段照修订 9 第 9.1 条仍不跑，这一条不重开它；后果只在这套装置再被用起来时出现（第 11.9 条点名的 E200）。修订 10 与这一条由同一个执行员一次改完、一次重钉。

11.1 **替掉的原句**（原文照留，只对线程那几处失效）：
   - 修订 7 第 7.2 条：第一点从「读数单元各分 10」到「一个都放不下退 78」那几句；第二点「分 5 × ⌊额度 / 5⌋（`rows_threads`；额度 31 分 30，除不尽的 1 不给），它照分到的数排线道（`rows_orchestrator.py` 的 `allocate` 原样：30 → 6 道、每道 5）」；第四点「每台槽数 × 每槽线程 = ⌊额度 / 5⌋ × ⌊额度 / 槽数⌋ ≤ 额度（`allocate`，原样）」；第五点「读数单元各分 10，不吃满一台」那半句。
   - 修订 7 第 7.3 条整条（T 在 10 : 5 上量、第 7 步与第 9 步和第 3 步带流三那一项同时起、入库记 `threads=10`、单测钉比例）。
   - 修订 10 第 10.4 条第二点「T（修订 7 第 7.3 条）：不变」、第三点「逐行限时 max(1800 秒, 3T)：不变」、「会变的只有不经 `within_share` 起单元脚本的地方」那一点第 1 项里「`plan` 第 671 行那句「每个读数单元分 10 线程」照旧」与「要它打得与真跑一样……不在这一条」；第 10.5 条末段「`segment_one_units_revision_8.py`、三份逐行编排与 `reuse_library.py` 都不改，冻结层的六个钉值（修订 8 第 8.6 条）不动」；第 10.7 条两项。
   - 研究目录三处文件头：`segment_one_units_revision_8.py` 第 23–26 行、`rows_orchestrator.py` 第 31 行「每台槽数 = ⌊每台线程 / 5⌋；每槽线程 = ⌊每台线程 / 槽数⌋（≥ 5：原跑法每道 5，限时基线在这个量级上量的）」、`rows_orchestrator_segment_two.py` 第 31–34 行里「每槽 5 线程起，槽数 = ⌊每台线程 / 5⌋」那几句，改成第 11.2 条的真话。

11.2 **每个单元、每条线道各取多少线程**。额度 Q：这一条任务（第二段：那一片）的 `SINGLEFS_THREAD_CAP`，没设取登记值；取法原样（`segment_one_units_revision_8.py` 第 344 行 `registered_threads`、`rows_orchestrator.py` 第 260 行 `registered_host_threads`，都经 `scripts/hosts/heavy_threads.py` 的 `default_heavy_threads`），服务起的任务里 Q 就是登记的 31。
   - **第一段每个单元都取 Q**：32 个读数单元、Z1 与 Z7 两个小单元、`research-mutations`、修订 11 那一层的 RP2 自证单元（`segment_one_units_revision_11.py` 第 123 行）。`fits`（第 359 行）不改：在跑的合起来加上要起的不超过 Q，所以同一时刻本机至多一个单元在跑；队首照旧 `step-7`（`queue_order`，第 166 行），之后照推的用时从长到短一个接一个起。`research-mutations` 也取 Q：用户那句的例外是「需要用线程来控制内存大小的」变异，入口表里只有 `crates-mutation-replay.sh`，研究一侧变异驱动只跑 Python 单测、不在表里。
   - **第 11–13 步的逐行编排**照旧等要跑的单元全部绿、一个都不在跑才起（`rows_may_start` 原样），环境里线程数变量都设成 Q（不再去除不尽 5 的余数）。
   - **线道**：每台一槽，那一槽的线程 = 那一台的每台线程（`allocate` 交回槽数 1、每槽线程 = 每台线程；每台线程为 0 的记 0 槽，「线程不够一条线道」那一支照旧退 78）。第一段第 11、12 步同一个队列，在编排那一台上一次一行；第 13 步单线道照旧取每台线程（`single_lane_host` 原样）。第二段每台一片一槽，本机与转交到别的台的驱动都是 `--lanes 1 --threads <那一台的每台线程>`；修订 10 那一层本机闸没开时本机记 0 槽（`allocate_behind_the_local_gate` 原样）。
   - **重报**（第一段 `run_carry`、第二段 `run_carry_two`）取每台线程。前提：重报在那一步的队列收完之后才起（第一段 `merge_step` 在 `run_queue` 交回之后调，第二段 `merge_step_two` 在 `run_queue_two` 交回之后调）；执行员读调用处核，核出来会与一条线道同时跑的，停下交回，不自己改派法。
   - **写法（以登记值为准、不写死）**：删 `segment_one_units_revision_8.py` 第 95–98 行 `READING_UNIT_THREADS`、`SMALL_UNIT_THREADS`、`ROW_LANE_THREADS`，`rows_orchestrator.py` 第 118 行 `LANE_THREADS_MINIMUM`、第 121 行 `CARRY_THREADS`；单元的线程在 `unit_table` 里取 `registered_threads()`（与 `local_budget` 同一个数），`rows_threads` 删掉、`rows_environment` 直接用 Q。新写的代码里不出现 31、10、5 这类线程字面，只喂假世界的自证样本（`rows_orchestrator.py` 的 `SAMPLE_HOST_THREADS`、`rows_orchestrator_segment_two.py` 的 `SAMPLE_THREAD_CAP`、各格的 `thread_cap=`）除外。三处文件头（第 11.1 条末项）、`plan` 第 671–673 行与 `run` 第 558–559、605–606 行的日志句、`rows_orchestrator.py` 第 285、1162 行与 `rows_orchestrator_segment_two.py` 第 857 行「每槽要 5 线程」的报错句跟着改成真话；`rows_orchestrator_segment_two.py` 第 733 行改读重报的新取法。
   - **弄坏开关**：`rows_orchestrator.py` 的 `slots-ignore-lane-minimum`（第 72–73 行，红在 `allocation-threads`、`allocation-registered`）换成 `slots-split-by-five`（照改前按每槽 5 线程分槽；红在同两格），`BREAK_SWITCHES`（第 140 行）与 `SWITCH_CELLS`（第 1891 行）同改。单元编排的 `threads-ignored` 与 `dependency-ignored` 两条开关红的格里去掉 `e2e-no-room`（第 11.7 条删这一格）；`threads-ignored` 仍要红在 `allocation-threads`（不看额度时三个单元在零时刻一起起）。

11.3 **T 怎么重新量**：
   - T 的定义不改：第 7 步单元的进程挂钟（第 1 次重跑登记修订 2 第 2.5 节、修订 8 第 8.4 节），进程挂钟要包住各流 `elapsed_ns` 之和才算（原样）。变的是量它的条件：第 7 步取 Q（登记的 31），按第 11.2 条本机同一时刻只有它一个单元在跑。第 7 步计时产物末行 `E7TIMING name=concurrent_units_during_step_7` 应写 `units=none`（`finish_unit` 原样追加，只描述，不进判定；不是 `none` 走第 11.5 条）。
   - T₂（第二段，第 1 次重跑登记修订 10 第 10.1 节）的定义同样不改：第 1 步单元的进程挂钟；第 1 步取 Q，它本来就在第 0、1、2、2b 步那条链上依次跑。
   - 这两个数在这一条的跑法上都还没有读数：修订 8 那一趟从没跑过，第 2 次重跑的现状整表段不跑。

11.4 **逐行限时的依据怎么重写（收严一处：「超时」一类的读数门槛）**：
   - **杀进程的限时不改**：逐行驱动照旧以 max(1800, ⌈3 × T⌉) 秒杀装置那一步，第 13 步超时重跑那一次再乘 3（`device_timing.py` 第 58 行 `row_time_limit_seconds`）。`device_timing.py`、`row_driver.py` 一个字节不改：E201（三oracle合谋今树整表） 的装置按路径 import 它们、计划那一步记它们的 sha256（`scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/e171_research_side.py` 第 19–22 行），它的登记 `research/prompts/e201-preregistration.md` 第 414 行写「按路径 import 或原样起，不改、不拷」。
   - **旧依据不成立**：修订 7 第 7.3 条的依据是第 7 步 10 线程、每道 5 线程，比例 10 : 5。这一条之后第 7 步与每条线道都是 Q，比例 1 : 1，两边都独占本机。
   - **新依据**：T 那一趟与每行装置那一步线程相同、本机上都只有它一个占线程的进程，两边之比只剩坏实现行自己多出的写数、段数与取样态数——第 1 次重跑登记修订 2 第 2.5 节「3 倍不从任何一行的读数来：坏实现行会让写数、段数与取样态数跟着变，两条线道并跑会互相抢核」的前半句；后半句「两条线道并跑会互相抢核」在一台一槽下不再有。
   - **只留 3T 会放宽（推的）**：装置用时按 n^(−a) 随线程数 n 变，a 没量过，取 0 ≤ a ≤ 1。修订 7 的跑法下，一行的杀进程限时与它在同样条件下预期用时之比是 3 × (5 / 10)^a = 3 × 2^(−a)，在 1.5 与 3 之间；这一条的跑法下若只留 3T，这个比恒为 3。a > 0 时比修订 7 宽，落进超时一类的行更少；第 1 次重跑登记修订 2 第 2.5 节的修订表把限时变宽记成「对读法 5「超时」一类是放宽」。
   - **所以另加读数门槛 R = max(1800, ⌈1.5 × T⌉) 秒**（第二段把 T 换成 T₂；1.5 = 3 × 5 / 10）。读法：
     1. 第一段与第二段各行，行目录 `row-result.tsv` 里 `device_ns` 大于 R × 10⁹ 的（含被驱动以 253 杀掉的），记读法 5 的「超时」一类。
     2. 第一段第 13 步重跑的那一次（`row-result.tsv` 里 `timed_out_before` 为真），门槛是 3R；两次都超过才记超时（第 1 次重跑登记修订 2 第 2.5 节「两次都超时才记」原样）。
     3. `device_ns` 落在 R 与杀进程限时之间的行，编排不会让它进第 13 步（第 13 步只重跑 `device_exit=253` 的行，`rows_orchestrator.py` 文件头「第 13 步」），这种行第一次超过 R 就记超时。
     4. 比对器（`segment_one_comparator.py`、`segment_two_comparator.py`，E201 照用）不改。判之前先列出照 1–3 记超时、而比对器当成判过的行（下称门槛行）：门槛行是空的，各格照比对器的判定词；不空，凡判定词的输入行里有门槛行的那一格报「不够判（逐行读数门槛）」并列出门槛行，不取比对器那一格的判定词。
     列门槛行的命令（`<行输出根>` 是那一步的行输出根）：`grep -H -P '^(device_ns|device_exit|timed_out_before)\t' <行输出根>/row-*/row-result.tsv`，逐个行目录拿 `device_ns` 比 R × 10⁹（`timed_out_before` 为真时比 3R × 10⁹）。读的是驱动在进程起止取时刻算的 `device_ns`，不是转发循环里的到达时刻。
   - **收严在哪**：a ≤ 1 时这一条的比 1.5 不大于修订 7 的 3 × 2^(−a)（a = 1 时相等）：修订 7 跑法下按读数算超时的行，在这一条跑法下照样超时；另多要求第 3 项那种行一次就记超时（原规则要两次）。下限 1800 秒照旧（第 1 次重跑登记修订 2 第 2.5 节「限时只升不降」说的是不低于原来的 1800 秒，这一层不动）。各行判甲判乙的条件一个不变。
   - **前提与推翻条件（推的）**：a 在 0 与 1 之间，两边本机负载相当（修订 7 跑法下第 7 步跑时三个读数单元合 30 线程、逐行 6 道合 30 线程；这一条两边都是一个 Q 线程的进程）。量出 a > 1（多给线程超线性变快）时 1.5 比修订 7 宽，这一条的「收严」不成立，交主 agent。

11.5 **停机条款「线道线程与 T 那一趟不等」（收严一处）**：
   - 触发观测：第 7 步单元的日志行「▶ 单元 step-7 起（…）：N 线程」（`segment_one_units_revision_8.py` 第 450 行）里的 N，与第 11、12 步逐行驱动打的 `table lanes=1 threads_per_lane=M`（`row_driver.py` 第 769 行）里的 M 不等；或第 7 步计时末行不是 `units=none`。第二段同理比第 1 步单元的 N 与第二段各驱动的 M。
   - 处置：第 11.4 条的读数门槛不适用，超时一类不判，交主 agent；不作废、不当结果，照 `.claude/rules/implementation-first.md` 第 4 条两边都查。
   - 收严在哪：第 11.4 条的 1.5 只在两边同线程、同样独占本机时推得出；条件不满足就不判，不悄悄照比对器的超时一类判。

11.6 **单元编排的 plan 与真跑取同一个数**（修订 10 第 10.7 条第 2 项）：
   - `segment_one_units_revision_8.py` 第 702–704 行 `plan` 起单元脚本的环境，也经 `within_share(units[name].threads, …)`，与 `launch` 第 448 行同一个函数。这一条之后 `units[name].threads` 是 Q：`plan` 这条任务取 Q 与 `run` 起单元时设进环境的是同一个取法（`registered_threads`），单元脚本经修订 10 的 `heavy_thread_default` 读 `SINGLEFS_THREAD_CAP` 取到的也是它。
   - 单测（`test_revision_8.py` 加一条）：`SINGLEFS_THREAD_CAP` 设成 7、`THREADS` 设成 10，`plan` 打出的第 4 步驱动 `--threads`、第 5 步执行器的工作线程、修订 11 那一份 RP2 的 `--threads` 都是 7，且等于 `launch` 设进环境的 `SINGLEFS_THREAD_CAP`（`subprocess.Popen` 换成只记命令的假件，同 `test_thread_shares.py` 第 55 行 `LaunchRecorder`）。证红：仓副本里 `unit_table` 退回改前（读数单元 10）、`plan` 留改前（不套 `within_share`），这一条要红在「`launch` 设的 10 ≠ `plan` 打的 7」。
   - 等价变异留档：这一条之后「`plan` 起单元脚本不套 `within_share`」与原式在打出的线程上同值（两边都等于 `registered_threads()`），照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」第 2 条记等价、不进变异表；等价性另写一条用例留档：同一个单元，`unit_environment` 的环境直接给单元脚本与套 `within_share(units[name].threads, …)` 之后再给，`PRINT_ONLY=1` 打出的三处线程逐字相同（`SINGLEFS_THREAD_CAP` 设 7 与不设两个取样点）。

11.7 **单测、自证与变异跟着改的**（执行员照做；新用例名写场景）：
   - `test_thread_shares.py`：第 86–94 行（第 7 步 10、每道 5 的比例）换成「第 7 步取额度，逐行编排每台一槽、那一槽线程等于额度」；第 96–100 行换成「每个单元（含 Z1、Z7、`research-mutations`）都取额度」，额度取 31 与 12 两个取样点；第 102–108 行（去余数）换成「逐行编排环境里的线程数变量等于额度、不去余数」（31 得 31、14 得 14）；第 117–125 行换成「额度 31 时同一时刻至多一个单元在跑，链尾那一刻起的只有 `step-7`」；第 127–133 行（余数给小单元）删，换成「本机一个都没在跑时哪个单元都放得下，有一个在跑时别的都放不下」（这一条抓第四张表的 R2P-45）；第 146–155 行期望改成两个单元的线程数变量都是 31；第 157–164 行期望改成 31；第 257–262 行改成「5 到 64 每个额度都是一槽、那一槽线程等于额度」；第二段两条（第 241–255 行）传进去的槽线程从 5 改成 31，断言照旧核驱动拿到的是传进去的那一槽的数。
   - `test_default_threads.py` 第 52–60 行加一句：每台恰一槽、那一槽线程等于登记值。`test_revision_10.py` 第 141–147 行期望从 `([(1, 0), (2, 2)], [(1, 2), (2, 2)])` 改成 `([(1, 0), (2, 1)], [(1, 1), (2, 1)])`。
   - `segment_one_units_revision_8.py` 自证：删「此刻一个单元都放不下」那一支（第 624–626 行；每个单元取的就是额度，本机一个都没在跑时总放得下，到不了）与格 `e2e-no-room`（第 937 行起，第 1018 行登记），第 627–628 行「还有单元没做完、却没有一个依赖齐了」那一支留着；格 `allocation-threads`（第 826 行）改成三个单元只放得下队首的 `step-7`；`e2e-two-rounds`（第 859 行）改成第 7 步计时末行写 `units=none`；`e2e-threads-within-quota`（第 955 行）里 `step-7` 与 `rows-run` 读到的上限都等于额度（第 974 行的 `"10"`）。修订 6 第 6.2 条当时做过同形的三处改动（原文照留，这里只借写法）。
   - `rows_orchestrator.py` 自证：`allocation-threads`（第 1524–1525 行）期望改成两台各 `(1, 31)`；`allocation-short`（第 1526–1527 行）改成每台线程 0 时 `(0, 0)`；`allocation-registered`（第 1529–1532 行）期望改成 `(登记值, 1, 登记值)`。`rows_orchestrator_segment_two.py` 的自证假世界（第 122 行 `SAMPLE_THREAD_CAP = "10"`，注释「两槽、每槽 5 线程」）变成每台一槽 10 线程；要一台上两槽才判得出的格改到两台的假世界上判，判不出的格写进交回，不悄悄删。
   - 第四张变异表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_threads_and_contrast.tsv`（此刻 23 条）：R2P-39（原文 `READING_UNIT_THREADS = 10`）、R2P-40（`rows_threads` 去余数那一行）、R2P-43（`within_share(rows_threads(budget.threads), …)`）的原文随改动没了，换成同一意图、锚在新写法上的三条：单元不取额度（取一个固定数）、逐行编排的线程去掉除不尽 5 的余数、逐行编排不照额度起；另加一条「小单元不取额度」。R2P-45（`<=` 改 `<`）原文还在，由上面第一项新用例抓。`rows_orchestrator.py` 的分法照旧不进表（表头那一句的理由原样），由 `test_thread_shares` 与 `rows_orchestrator.py` 自证的 `allocation-*` 格盯。改完每条原文在研究目录里恰命中 1 次（修订 5 第 5.3 条 `anchor_hits.py` 那一种数法）。
   - 修订 8 那张表（25 条，`…_revision_8.tsv`）：R8-18 的原文 `sum(item.threads for item in running) + unit.threads <= budget.threads` 照旧在，表不改；它盯的 `segment_one_units_revision_8.py` 改了，重跑。

11.8 **要改的文件与要重钉的值**（修订 10 与这一条合起来一张；改前 sha256 是 2026-11-06 现算的，量的；改后的值照 `sha256sum` 现算，写进交回或接续修订）。研究目录 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`：

| 文件 | 改前 sha256 | 改什么（出处） | 钉它的地方 → 要重钉的值 |
|---|---|---|---|
| `segment_one_unit_revision_8.sh` | `58981365aea4a056a5c5aa819db58c5a1bbde27f61daaea91b77ee8fb1b7e81e` | 修订 10 第 10.2 条 | 没有（修订 10 第 10.5 条） |
| `segment_one_unit_revision_11.sh` | `3fa8f3b8f85b47c69015b4fea2daa61c6490090a3933eed0176f7c9b2c800435` | 修订 10 第 10.2 条 | H31 |
| `segment_one_units_revision_8.py` | `b2d4c89a8e1a7c63c264d5841d4f5e3bedfe405df91d7fadf525229c71f7411f` | 第 11.2、11.6、11.7 条 | `segment_one_units_revision_11.py` 第 61 行 `UNITS_R8_SHA256` |
| `segment_one_units_revision_11.py` | `67082bfbb0b2db000111b33b5e90ada23eac8c199d2a9c57897edd7877228e18` | 修订 10 第 10.2 条文件头半句；第 11.2 条第 123 行；`UNITS_R8_SHA256` | H31 |
| `rows_orchestrator.py` | `d2ebaaaf914db01f384004676d9e127593c11d1796a4694b50924db38b2255c1` | 第 11.2、11.7 条（`allocate`、两个常量、文件头、报错句、开关、自证） | `cross_host_o3_check.py` 第 56 行 `ORCHESTRATOR_SHA256`；`rows_orchestrator_revision_8.py` 第 64 行、`rows_orchestrator_segment_two.py` 第 114 行 `SEGMENT_ONE_ORCHESTRATOR_SHA256`；`reuse_library.py` 第 454 行 `ORCHESTRATOR_SHA256` |
| `cross_host_o3_check.py` | `7247276e8046551e18bbfe2a3402babfb3e4e8554f467bac56796c2e6975fc42` | 只换钉值 | `rows_orchestrator_revision_8.py` 第 66 行、`reuse_library.py` 第 457 行 `CROSS_HOST_CHECK_SHA256` |
| `rows_orchestrator_segment_two.py` | `accb1cbd108660668b2180b4f4e8b847c24140b85229c41e12c35d43d41cebd3` | 钉值；文件头、第 733 行重报、第 857 行报错句、自证假世界（第 11.2、11.7 条） | `reuse_library.py` 第 455 行、`rows_orchestrator_segment_two_revision_10.py` 第 99 行 `SEGMENT_TWO_ORCHESTRATOR_SHA256` |
| `rows_orchestrator_revision_8.py` | `d5bf6305ff1f95fb1cf2360cdb385627e9e2e989ef68d699730e4c321b8bbb80` | 只换两个钉值 | `rows_orchestrator_revision_11.py` 第 66 行 `ROWS_R8_SHA256` |
| `rows_orchestrator_segment_two_revision_10.py` | `ab4f6a78fb489e88c62dfd6070993f2c65a4dbb07c75d8805a8076d436f4703e` | 只换钉值 | `rows_orchestrator_segment_two_revision_11.py` 第 60 行 `SEGMENT_TWO_R10_SHA256` |
| `reuse_library.py` | `5c513fa59546f483bcbfe4c9e7ecdee9db52923ac6677f23372d0041f328c24d` | 只换三个钉值，存法那一块不碰 | H31 |
| `rows_orchestrator_revision_11.py` | `15224dc26382e66ba5aedaa37891ee91614f74846b1f0d6f0a64ed4dbbeb8a4f` | 只换钉值 | H31 |
| `rows_orchestrator_segment_two_revision_11.py` | `bc99c5f3565f75771f51a7703e0ce84c50667630962195b94d457b18fc626cb8` | 只换钉值 | H31 |
| `test_thread_shares.py` | `1d1ad862064d99c11565ec7b0fa0eec6a1dde3da49e5c76a52ff7beb25587be0` | 第 11.7 条 | 不钉 |
| `test_revision_8.py` | `f6f3566baf9d0af32b418329eaa40d37f117cf74112bbdf0bf34663a25ad4925` | 修订 10 第 10.6 条；第 11.6 条两条 | 不钉 |
| `test_default_threads.py` | `89ec438f0bbb689ba3cd2143d476ab25bba722dedd442915778a4f047051c8ea` | 第 11.7 条 | 不钉 |
| `test_revision_10.py` | `58814cd751bc82ce7b7558d800767096b485974620264a863451f7a6f8d59065` | 第 11.7 条 | 不钉 |
| `revision_11_placeholders.json` | `00e9776742d390622255ce00921a100f1f4d8e918af5698a0fde654fcef17116` | H31 五项 | 不钉 |
| 第四张表 `…_current_state_threads_and_contrast.tsv` | `b239116cd8dec8764d42924f0f1fe2e743c27a609137690f3a7b9a7c62e8ee49` | 第 11.7 条 | 不钉；产物新名，`replay.sh` 第 2041 行改指它 |
| 修订 8 那张表 `…_revision_8.tsv` | `d037e74e65ad105ca0acab4c135c83c2715eb6d383898f88b2d01191b4f38d03` | 不改，重跑 | 产物与 `replay.sh` 第 2029 行登记的那一份不同才另起新名、改登记行 |

   - **重钉次序**（依赖次序，到不动点：每个钉值等于被钉那份此刻的 sha256）：先改 `rows_orchestrator.py`、`segment_one_units_revision_8.py` 与别的内容改动 → `cross_host_o3_check.py` → `rows_orchestrator_segment_two.py` → `rows_orchestrator_revision_8.py`、`rows_orchestrator_segment_two_revision_10.py` → `reuse_library.py`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`、`segment_one_units_revision_11.py`。钉值共 11 处（上表第四列）；`reuse_library.py` 第 1895–1900 行 `PIN_SITES` 列的六处是其中一部分，另五处（`cross_host_o3_check.py` 一处、`rows_orchestrator_revision_8.py` 两处、`rows_orchestrator_segment_two.py` 一处、`rows_orchestrator_segment_two_revision_10.py` 一处）不在 `PIN_SITES` 里，一样要换。修订 7 第 7.7 条、修订 8 第 8.6 条的草稿 `repin_chain.py` 若已不在，照这张表手改，改完逐处现算。
   - **H31**（`revision_11_placeholders.json` 第 15–24 行，十项）：变五项——`reuse_library.py`、`segment_one_units_revision_11.py`、`segment_one_unit_revision_11.sh`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`；另五项（`count_reusable_units.py`、两份 `run_segment_*_revision_11_multi_host.sh`、`test_revision_11.py`、`reuse_library_root.py`）不动，改完逐项现算 sha256 与登记值比（修订 10 第 10.5 条末段那一种算法），十项都 `same` 才算重钉完。
   - 不改的：`device_timing.py`、`row_driver.py`、两份比对器、`frozen_o3.py`（第 11.4 条）；四份跑法脚本；另八张研究一侧变异表的内容（产物照第 11.9 条重跑核）；`revision_8_placeholders.json`（H17 照修订 9 第 9.4 条是 `null`）与 `revision_10_placeholders.json`（H25、H26 只列名字）。

11.9 **对已有产物、结论与别的实验的影响**：
   - **第 2 次重跑的结论：不受影响**。Q1 撤项那一格（甲）与 Q2（有）出自名单核对与三份撤项核算（修订 3 第 3.1 条、修订 9 第 9.5 条），不经单元编排与逐行编排。
   - **`research/results/` 里没有一份产物出自这一条改的跑法**：现状整表段没跑（修订 9 第 9.1 条），第 1 次重跑修订 8 那一趟从没跑过；第 1 次重跑修订 6 那一趟的产物出自 `run_segment_one_on_the_revision_6_tree_multi_host.sh`（脚本里写死 `THREADS=10`，修订 8 第 8.2 条第四点）与当时那一版 `rows_orchestrator.py`（`revision_8_research_side_of_the_revision_6_run.json` 钉的旧版），复跑经 `replay.sh` 的 `e171_r1_view` 起冻结的那一份，这一条一样不碰。T 与逐行限时在这一条的跑法上没有读数（第 11.3 条）。
   - **研究一侧变异表的产物**：第四张表（内容变）与修订 8 那张表（盯的文件与单测变）的产物必与 `replay.sh` 登记的不同；修订 10 那张表的单测 `test_revision_10` 变；另七张的内容不变，但它们跑的单测与被变异的源文件有变的（推的不变，没量）。十张都在改完之后重跑，逐字节比 `replay.sh` 第 2026–2041 行十个研究一侧变异驱动登记的产物；不同的照本登记修订 1 第 1.12 条的做法另起新名、改登记行，旧产物留着。实验页第 2 次重跑一节整行引这些产物的句子照新产物回对，归写回员。
   - **E200（修订6近似树重跑）**：它的登记 `research/prompts/e200-preregistration.md` 第 44 行写它「用今天研究一侧的修订 11 单元编排（`segment_one_units_revision_11.py`、`segment_one_unit_revision_11.sh`、`run_segment_one_revision_11_multi_host.sh`）」、「各步判法、阈值、限时公式照修订 6、修订 2 第 2.5 节」；修订 9 第 9.3 条末段引 E200 页，它停在停机 S40。这一条之后 E200 再起跑，单元与线道都取 Q、T 在 Q 上量；第 11.4 条的读数门槛只进 E171 这一份登记，不进 E200。E200 要不要另写修订认这件事交主 agent（第 11.12 条第 1 项）。
   - **E201（三oracle合谋今树整表）：不受影响**。它的目录里 `grep -l 'rows_orchestrator\|segment_one_units\|allocate('` 零份；它 import 的 `device_timing.py`、`row_driver.py`、两份比对器与 `frozen_o3.py` 这一条不碰（第 11.4 条）。

11.10 **并行与挂钟**（现状整表段不跑，只给这套装置再被用起来时排机器；数都是推的，没量）：
   - 独立单位：第一段各单元照依赖图独立，但每个都吃满额度，本机同一时刻至多一个；第一段逐行在编排那一台一次一行；第二段每台一槽。
   - 线程：每个单元、每条线道各取登记值 `.claude/heavy-thread-default` 的 31（每台）。
   - 串行：第一段全部单元串起来（每个都要整份额度）；不拆成服务分片子任务的理由在第 11.12 条第 3 项。
   - 预计挂钟：第一段单元——`unit_table` 第 128–148 行各单元推的用时相加约 52 400 秒（在修订 7 跑法 10 线程上推的），按第 11.4 条的 n^(−a) 缩成 52 400 × (10 / 31)^a 秒，约 4.7–14.6 小时；第一段逐行——不分片时只用一台、一次一行，16 行（带探针 2 行各约 T、不带探针 14 行各约 9 698 秒，照 `rows_orchestrator.py` 文件头「每行推的用时」；每行另加 120 秒；跨机对照那一行不分片时不跑），约 (2 × 14 582 + 14 × 9 698) × (10 / 31)^a + 16 × 120 秒，约 15.3–46.3 小时；合约 20–61 小时。修订 7 的跑法推约 15 小时（单元约 5.3 小时：链 1 200 秒 + 第 7 步 14 582 秒 + 修订 7 第 7.2 条那 3 244 秒；逐行约 9.5 小时，修订 7 第 7.2 条）。第二段——278 行 × 约 97 秒（第 1 次重跑修订 6 那一趟每行平均，那一趟每道 6 或 10 线程）÷ 两台各一槽，约 1–3.7 小时；修订 7 跑法两台各 6 槽约 0.6 小时。
   - 预计在用核数：每台约 31 核（一个单元或一条线道吃满）；第二段两台合约 62 核。
   - 超过 2 小时：分片单元与续跑入口照旧（第十四节、第 1 次重跑登记修订 8 第 8.4 节「续跑」：单元按单元续，逐行按行目录里有没有 `row-result.tsv` 续）。

11.11 **执行员照做的次序与交主 agent 的验证**：
   - 次序：`python3 scripts/agents/copy-originals.py copy <草稿目录> <第 11.8 条表里要改的文件>` 留改前那几份 → 核冻结层钉值此刻是不是不动点（第 11.12 条第 4 项；不是就停下交回）→ 修订 10 第 10.2 条两份单元脚本与 `segment_one_units_revision_11.py` 文件头 → 第 11.2 条（`segment_one_units_revision_8.py`、`segment_one_units_revision_11.py` 第 123 行、`rows_orchestrator.py`、`rows_orchestrator_segment_two.py`）→ 第 11.6 条 `plan` → 第 11.7 条单测、自证与第四张表 → `bash -n` 两份单元脚本、`python3 -m py_compile` 改过的 `.py` → 照第 11.8 条次序重钉到不动点 → 填 H31 → 现算 H31 十项都 `same`、11 处钉值都等于被钉那份此刻的 sha256。
   - 交主 agent 的（执行员写进「要主 agent 跑的命令」，子 agent 不交任务）：研究目录（`--cwd`）里 `python3 -B -m unittest test_thread_shares test_revision_8 test_revision_10 test_revision_11 test_default_threads`，末行 `OK`；`.claude/hooks/selftest-registry.tsv` 登记的 E171 研究目录 14 份 `--selftest` 与 `rows_orchestrator.py --selftest`，各末行「红 0」，开关轮里 `slots-split-by-five` 红在 `allocation-threads`；十张研究一侧变异表各经研究目录的 `mutation_driver.py` 跑一遍（表头写的 `--unit-test-module`），输出写草稿目录，与 `replay.sh` 第 2026–2041 行登记的产物 `cmp`；修订 10 第 10.8 条的门禁那一格；修订 10 第 10.6 条与第 11.6 条的证红（仓副本里）。
   - 不用跑的：现状整表段、两段干跑、`stream-constants`。

11.12 **交主 agent 的项**：
   1. **E200**（第 11.9 条）：这一条改的是 E200 登记里点名照用的单元编排；E200 再起跑之前它的登记要不要另写修订（线程、T 的条件、要不要也加读数门槛），由主 agent 定。
   2. **`.claude/batch-scope`**：第 11.8 条表里的文件多数登记在旧批次，本批（`records/2026-11-05-删轻型并入重型.md#并包`）的范围要加（修订 10 那件报告第 3 条同一件）。
   3. **不拆成服务分片子任务的理由**：第一段单元由一份冻结的编排在一条任务里排，拆成分片要重写它的派发、续跑与日志判法；第 2 次重跑已收尾，这套装置只在再被用起来时跑；串着跑合规（修订 7 依据里转述的服务那一层写法）。代价是挂钟（第 11.10 条：约 20–61 小时，修订 7 跑法约 15 小时，推的）。要压挂钟就另写修订拆分片。
   4. **冻结层钉值此刻不全是不动点**（量的，2026-11-06 `sha256sum`）：`reuse_library.py` 第 455 行与 `rows_orchestrator_segment_two_revision_10.py` 第 99 行钉 `rows_orchestrator_segment_two.py` 为 `413ce4a869667696acd62b05caba6441d3936e022d49db5fce72f437df5f0b39`，那份此刻是 `accb1cbd…`；`rows_orchestrator_revision_11.py` 第 66 行钉 `rows_orchestrator_revision_8.py` 为 `055844678056bfd032017b7a6c35671aa17b6d5f92620716b16aaed02e090526`，那份此刻是 `d5bf6305…`。`reuse_library.py` 第 466 行 `pinned_sha256_now` 按搬迁登记倒换之后认不认得上没核（要载入研究目录的模块，是跑项目代码）。执行员开工先照它核：认得上照常重钉；认不上是这一条之前就有的另一件，停下交回，不顺手改。
   5. **读数门槛（第 11.4 条）是这一条自己加的收严**：只看限时公式字面、不加门槛的写法，按第 11.4 条第四点对超时一类是放宽，不合「判据只收严或不变」。

11.13 **设计这一条时已经存在的数**（照第四节的写法，不删）：
   1. 第 1 次重跑登记里读到的：修订 6 那一趟第二段 T₂ = 75.274 秒、限时取下限 1800 秒（第 1763 行）；修订 8 第 8.4 节推的第 7、9 步各 14 582 秒、第 3 步带流三两项 9 698 与 8 128 秒（经本登记修订 7 第 7.2 条与 `unit_table` 第 128–148 行）。影响：只进第 11.10 条挂钟；1.5 出自 3 × 5 / 10，不从这些数来。
   2. E201 登记第 345 行（`grep` 命中行）引的第 1 次重跑修订 6 那一趟第二段每行平均 97.2 秒。影响：只进第 11.10 条。
   3. 修订 9 第 9.3 条末段：E200 页量的两棵近似树都停在停机 S40。影响：只进第 11.9 条。
   4. 门槛在已有读数上的值（推的）：拿第 1 项的 T₂ = 75.274 秒代进去，R = 1800 秒，与杀进程限时相同，第二段没有门槛行；第一段在这一条的跑法上没有 T。
   5. 第 11.12 条第 4 项那两处钉值与被钉文件此刻的 sha256 不同（量的）。
   6. 装置是否随线程数变快（a）：`crates/singlefs-verification/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 里 `grep -n 'thread\|rayon\|par_iter\|available_parallelism\|THREAD_CAP\|spawn'` 零命中，它经 `GivenStatesEngine` 与 `crash_state_enumeration` 判态（文件第 2、32、109 行），那两处用不用多线程这一次没读；a 照第 11.4 条当未知。

11.14 **设计这一条时读过的文件与跑过的命令**（2026-11-06，HEAD `ff5dca7d22b1c6f95b2ac64a8c79e0467d580d1c`；研究目录 `git status --porcelain` 为空，本登记是 ` M`，改前 sha256 `8aea17911c4c155c89162027bc7bf058e8c5470590c06d106be383b142b8bf77`）：
   - `.claude/agent-common.md`（整份）；规则整份：`.claude/singlefs-ai-sop/rules/test-discipline.md`、`.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`、`.claude/rules/mutation-sampling.md`、`.claude/rules/implementation-first.md`、`.claude/rules/test-script-discipline.md`。
   - 本登记第 1–184 行、第 410–726 行与 `grep -n '^#'`、`grep -n '3T\|max(1800\|1800'`、`grep -n '1\.12'` 命中行；`research/prompts/e171-r1-prereg.md` 第 690–697、790–806、1555–1562、1820–1828 行与 `grep -n '3T\|3 T\|1800 秒\|max(1800'` 命中行；`research/prompts/e200-preregistration.md` 与 `research/prompts/e201-preregistration.md` 只有 `grep` 命中行（前者第 32、44、113、116、117、370、386 行，后者第 138、143、329、345、414、420、630 行）。实验页与 `research/results/` 下产物的内容都没读。
   - 前一件设计员的报告 `~/.local/share/singlefs/work/e171-thread-amendment/result.md`（整份）；`records/2026-11-05-删轻型并入重型.md` 第 338、622 行（`grep` 命中）；`.claude/heavy-thread-default`（整份）；`.claude/memory-bounded-parallelism.tsv` 第 1–12 行；`.claude/hooks/selftest-registry.tsv` 里 `grep` E171 研究目录的 14 行；`scripts/experiment-apparatus/replay.sh` 第 1–12、3105–3175、3252–3262 行与 `grep` 命中的第 2026–2041 行；`scripts/kb/replace-once.py` 第 1–40、307–317 行。
   - 研究目录：`segment_one_units_revision_8.py` 第 1–720 行与第 820–1000 行里 `grep` 命中行、第 622–628、826–841 行；`test_thread_shares.py`（整份）；`rows_orchestrator.py` 第 25–45、105–125、255–300、1050–1080、1515–1540 行与 `grep` 命中行；`rows_orchestrator_segment_two_revision_10.py` 第 170–200 行；`device_timing.py` 第 40–62 行与 `grep` 命中行；`row_driver.py` `grep` 命中的第 6–102、568–602、653–699、769 行；`test_default_threads.py` 第 45–70 行；`test_revision_10.py` 第 138–150 行；`revision_11_placeholders.json`（整份）；`reuse_library.py` 第 466–491、1885–1905 行；另几份的 `grep` 命中行（`rows_orchestrator_segment_two.py`、`segment_one_units_revision_11.py`、`test_revision_11.py`、`cross_host_o3_check.py`、`rows_orchestrator_revision_8.py`、`rows_orchestrator_revision_11.py`、`rows_orchestrator_segment_two_revision_11.py`、`mutation_driver.py`）。第四张变异表与修订 8 那张表的 `grep`、`awk` 命中行，十张 E171 研究一侧表的条数。E201 目录 `e171_research_side.py` 第 12–40 行与 `current_tree_row_driver.py` 的 `grep` 命中行。
   - 命令：认领草稿目录；`sha256sum`、`wc -l` 第 11.8 条表里各文件；`grep -nE` 研究目录里按 64 位十六进制钉文件的常量（11 处）；`grep -rn` 线程常量与 `unit_table(` 的引用；`grep -ln` E200、E201 目录里有没有起 E171 编排的写法（零份）；`grep -n` 装置源文件的线程写法（零命中）；`git log --oneline -3` 两份被钉文件；`TZ=Asia/Tokyo date +%F`（2026-11-06）。没跑项目代码，没往统一任务服务交任务。

11.15 **第 11.12 条第 4 项核出来的原有对不上**（2026-11-06，执行员照派发提示「在登记里写明这处原来就有的对不上……然后照重钉链往下做」补写；写这一节时装置一处没改、重钉链没开始；判据、阈值、作废与停机条款不动）：
   - **哪几处、钉值与此刻值**（量的，`sha256sum` 与 `awk 'NR==行'`）：

| 钉它的地方 | 钉值 | 被钉文件 | 被钉文件此刻 sha256 |
|---|---|---|---|
| `reuse_library.py` 第 455 行 `SEGMENT_TWO_ORCHESTRATOR_SHA256` | `413ce4a869667696acd62b05caba6441d3936e022d49db5fce72f437df5f0b39` | `rows_orchestrator_segment_two.py` | `accb1cbd108660668b2180b4f4e8b847c24140b85229c41e12c35d43d41cebd3` |
| `rows_orchestrator_segment_two_revision_10.py` 第 99 行 `SEGMENT_TWO_ORCHESTRATOR_SHA256` | 同上 | 同上 | 同上 |
| `rows_orchestrator_revision_11.py` 第 66 行 `ROWS_R8_SHA256` | `055844678056bfd032017b7a6c35671aa17b6d5f92620716b16aaed02e090526` | `rows_orchestrator_revision_8.py` | `d5bf6305ff1f95fb1cf2360cdb385627e9e2e989ef68d699730e4c321b8bbb80` |

   - **从哪一次提交起就不对**：提交 ee47e22d9（2026-11-05）同时改了两份被钉文件与这三处钉值；那一次提交里两份被钉文件已经是 `accb1cbd…` 与 `d5bf6305…`（`git show ee47e22d9:<路径> | sha256sum`），三处钉值写的是 `413ce4a8…` 与 `05584467…`（`git show ee47e22d9:<路径> | grep -n '_SHA256 = "'`）。它的上一次提交 48c98a3fa 里 `reuse_library.py` 第 425 行钉 `7dda75bc…`，被钉文件那时也是 `7dda75bc…`。`git log --all` 动过两份被钉文件的各版（新旧路径都查）没有一版是 `413ce4a8…` 或 `05584467…`，这两个值对应的内容不在 git 里。
   - **怎么查出来的**：前一段执行员照第 11.11 条次序第二步核冻结层钉值，先用文本模拟（读 `.claude/gate.d/path-moves.tsv`，对两份被钉文件里搬迁登记的新前缀做每一处换不换的全组合倒换，2² 与 2⁴ 种，没有一种等于钉值；把两份里钉上游的值换回旧值的组合也等不出），报告 `~/.local/share/singlefs/work/e171-thread-landing/result.md`；主 agent 交的任务 8541 用研究目录的真函数 `reuse_library.pinned_sha256_now` 核，`task-server/task log 8541 --all` 两行原样「segment_two None」「rows_revision_8 None」，即认不上。
   - **怎么处置**：主 agent 定照第 11.8 条重钉链往下做，这三处随重钉链一并换成改后被钉文件的 sha256，换完逐处现算到不动点。换掉之前载入这两份冻结文件的那几条路（`reuse_library.py` 的 `load_repinned` 认不上就抛 RuntimeError，推的，读第 529 行起那一段）此刻会拒；这一节不量它此刻拒了哪几条单测或自证。改后的新值写进执行员的交回报告。

## 十三、读过的文件与跑过的命令

**读过的文件**（行号是读那一刻的；grep 命中行也列）：
- `.claude/agent-common.md` 第 1–121 行（整份）。
- 规则整份：`.claude/singlefs-ai-sop/rules/test-discipline.md` 第 1–138 行、`.claude/rules/three-way-inference.md` 第 1–168 行、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 第 1–182 行、`.claude/rules/mutation-sampling.md` 第 1–93 行、`.claude/rules/implementation-first.md` 第 1–19 行、`.claude/rules/test-script-discipline.md` 第 1–106 行；`.claude/rules/verification.md` 第 39 行与第 70–95 行。
- `research/prompts/e171-r2-questions.md` 第 1–10 行（整份）。
- `research/prompts/e171-r1-prereg.md`：`grep -n '^#'` 全部标题；第 1–90、289–578、1433–1672、1793–1862、1964–1971、2372–2380、2473–2512、2701–2800、2870–2989 行。
- `.claude/kb/experiments/171-三oracle合谋测试.md`：只有 `grep -n '^#'` 的 35 行标题（第四节第 1 条）；正文没读。三语实验页、变更史与 `records/2026-10-01-里程碑三形式化接入.md`、`records/2026-10-13-复用键与全仓改名.md` 只出现在 `grep -rl` 的文件名清单里，没读内容。
- `.claude/kb/decisions/15-格式冻结政策.md`：`grep -n '^#'` 标题，第 74–99 行（第二节整抄）。
- `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`：`bad_implementation_table.py` 第 1–175 行（整份）；`row_driver.py` 第 360–420 行与 `grep` 命中的第 2、17、63、66、384、403、638、642、689 行；`segment_two_comparator.py` 第 1–60、225–240、340–400、490–500 行；`reuse_library.py` 第 1–60 行与第 135 行；`segment_one_comparator.py` 第 31–36 行；`rows_orchestrator.py` 第 155 行；`rows_orchestrator_segment_two.py`、`count_reusable_units.py`、`rows_orchestrator_segment_two_revision_10.py`、`rows_orchestrator_segment_two_revision_11.py`、`frozen_tree_lines_revision_8.py`、`run_segment_*_multi_host.sh`、`check_product.py` 的 `grep` 命中行（第三节列的行号）；目录清单。
- `scripts/experiment-apparatus/e171_three_oracle_collusion/`：`comparator.py` 第 95–135 行与 `grep` 命中的第 106、115、431、442 行；`expansion_table.py` 第 20–40 行；`collusion_report.py` 第 190–300 行；`collusion_matrix.py` 第 279、284 行（`grep`）；`row_driver.py` 第 8、603 行（`grep`）；目录清单。
- `scripts/experiment-apparatus/replay.sh`：第 2700–2830 行；`grep` 命中的第 29、80、121、474、535、1244、1401–1418、1816–1835、2431–2579、2748–2806 行。
- 两份名单（四个文件）：第 1–3 行、扩表第 117–121 行；`awk` 逐行取第 1、3、5 列。
- `crates/mutations.tsv`（今天、`HEAD`、`0c7fdb82b`、`f8a9ddaab`、`73866746b`、`d6dd6da42`、`4e8b03705` 七个版本）：只经 `awk` 按名字与三样取行，命中行原样见草稿 `q2-lookup.txt`、`q2-history.txt`；第 476、612–620、954–956、2517 行（`grep` 现名）。
- `crates/singlefs-core/src/system_configuration.rs`：`grep -n 'INCOMPAT\|incompat'` 命中的第 24–36、164、598、705–724、824–860 行。
- `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`：`grep` 命中的第 17、21、211、215、227 行。
- `.claude/kb/checks-owed.md`：`grep` 命中的第 282、407、444、448、469、555、559、563、587、588、592、594、785、828 行；第 592、594 行整行。
- `.claude/kb/term-renames.md` 第 24–34 行（`grep '层 0'`）；`.claude/term-rename-exempt` 第 89–94 行；`.claude/gate.d/mutation-duplicates-lag.tsv` 第 1–16 行；`.claude/gate.d/counted-claims.tsv` 第 1–12 行；`.claude/gate.d/replay-comparison-units.tsv` 第 1–40 行；`.claude/gate.d/doc-experiments.sh` `grep` 命中的第 27–88、143–144 行。
- `records/2026-10-31-删兼容旧版本代码.md`：`grep` 命中行与第 88–97 行。
- `.claude/heavy-thread-default`（整份）。
- `research/results/`：只列了文件名，算了第十一节三份的 sha256 与行数，没打开内容；归档的三份同样只经 `git show … | sha256sum`、`| wc -l`。

**跑过的命令**（都在仓根或草稿目录 `~/.local/share/singlefs/work/compat-removal-e171-r2-design/`；输出原样在草稿目录的同名文件或上面的数里）：

```
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/compat-removal-e171-r2-design --purpose 'E171 第 2 次重跑登记：坏实现名单撤掉退役位 0 那一项'
# Q2：名单第 5 列在今天的表（跳过 # 行、只收 6 列）里命中几次，不是 1 次的列出 → q2-lookup.txt（6 行）
awk -F'\t' 'NR==FNR { if ($0 !~ /^#/ && NF==6) cnt[$1]++; next } { n=cnt[$5]+0; if (n!=1) print FILENAME"\t"$1"\t"n"\t"$3"\t"$5 }' crates/mutations.tsv <D2>/bad-implementations-117.tsv <D2>/bad-implementations-expansion-164.tsv
git log --format='%h %ad %s' --date=short -S"<名字>" -- crates/mutations.tsv            # 每个取不到的名字，→ q2-history.txt
git show <提交>:crates/mutations.tsv > mutations-<提交>.tsv                             # HEAD、0c7fdb82b、f8a9ddaab、73866746b、d6dd6da42、4e8b03705
python3 -I tools/q2_triples.py mutations-0c7fdb82b.tsv crates/mutations.tsv q2-names.txt   # 三样逐字相同、同锚点的今天的行
python3 -I tools/q2_triples.py mutations-4e8b03705.tsv crates/mutations.tsv q2-names.txt
python3 -I tools/anchor_count.py mutations-head.tsv '读者仍把退役的位 0 当受支持的布局身份（旧镜像挂得上）' crates/singlefs-core/src/system_configuration.rs   # hits_in_source=0
grep -rn 'INCOMPAT_RETIRED\|retired_layout_bit\|old_reader_supported_bits' crates/ | wc -l                                       # 0
sha256sum <两个目录的四份名单> ; wc -l <同>; cmp <两个目录的同名名单>                       # 两对都相同
sha256sum research/results/<第十一节 #3、#5、#7>; wc -l <同>
git show 59ce5a865^:research/results/<第十一节 #1、#2、#4> | sha256sum ; | wc -l
grep -rlE 'bad-implementations-(117|expansion-164)' --exclude-dir=__pycache__ --exclude-dir=results --exclude-dir=prompts .
python3 scripts/kb/quote-kb.py <草稿>/quotes.md '.claude/kb/decisions/15-格式冻结政策.md@#### 已定项 4：feature bit 的位分配登记表'   # 回读逐字节一致
tail -n 30 research/prompts/e171-r2-prereg.md | cmp - <草稿>/quotes.md                     # 追加第二节之后：appended-quotes-identical
TZ=Asia/Tokyo date +%F                                                                    # 2026-11-01
```

`<D2>` 是 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes`；草稿脚本 `tools/q2_triples.py`、`tools/anchor_count.py` 在草稿目录，只读文件、不写仓。没跑任何项目代码，没往统一任务服务交任务。

## 十四、并行与挂钟

独立单位：撤项核算段里名单核对、撤项核算三份产物、钉值·名单、现状·可取行、`stream-constants` 一趟互不读对方的输出，五件互相独立；现状整表段的单位同上一次登记修订 8 第 8.4 节（第 0–10 步按单元，第 11–13 步与修订 10 第二段按行，每行一个单线道逐行驱动）。
线程：取登记值 `.claude/heavy-thread-default` 的 31（每台）；撤项核算段只有 `stream-constants` 那一趟的 release 编译用满 31 线程，其余都是单线程 Python；现状整表段的槽数与每槽线程由统一任务服务与编排器照 31 分。
串行：撤项核算段里「名单核对（改之前）→ 改名单 → 名单核对（改之后）→ 撤项核算」必须串行，因为撤项核算的 R 与改后核对都要改之前那一份的分类与改好的名单；`stream-constants` 与现状·可取行可以与它们并行。现状整表段的串行照修订 8 第 8.4 节（第 0、1、2、2b 步依次；第 7 步的进程挂钟就是 T，不能拆）。
预计挂钟：撤项核算段约 0.5 小时（推的：`stream-constants` 那一趟 release 编译约 10–25 分钟加跑约 2 分钟（修订 12 第 12.12.4 节「约 2 分钟」），名单核对 281 项 × 每项一次字典查、撤项核算三份共约 1 万行 × 每行一次解析，都是秒级；两份单测与 9 条变异每条一次单测进程约 2–5 秒，合约 1 分钟）。现状整表段约 14–17 小时（推的，上一次登记修订 8 第 8.9 节与修订 10 第 10.9 节：整表 278 行 × 每行约 200 秒 ÷ 6 槽 ≈ 2.6 小时，关键路径上是第 7、9 步各约 4.1 小时与第一段逐行约 9.5 小时；实现变了，每行与每态耗时都可能变，只用来排机器），超过 2 小时：分片单元是第 0–10 步按单元、两段逐行按行；续跑入口是同一条命令重起，只补没做完的单元与行目录里还没有 `row-result.tsv` 的行（修订 8 第 8.4 节「续跑」、修订 10 第 10.9 节）；今天这条入口开跑先退 3（C936），所以现状整表段的续跑能不能用，要等 S22 解了才知道。
预计在用核数：撤项核算段编译那十几分钟约 31 核（一台），其余时间 1 核；现状整表段两台各约 31 核、合约 62 核（推的：槽数 × 每槽线程按登记值 31 分，修订 6 那一趟第二次起本机 5 槽 × 6 线程、第二台 3 槽 × 10 线程，合 60 线程，与此同量级）。
