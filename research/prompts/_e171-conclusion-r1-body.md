# E171（三oracle合谋测试） 的结论撑不撑 D13（验证路线） 已定项 6：第一轮正文（2026-10-03）

<!-- doc-lint:not-numbers O1 O2 O3 Q1 Q2 Q3 Q4 G1 G2 G3 G4 G5 F3 F4 F5 P8 -->

## 一、题面

E171（三oracle合谋测试） 照跑前登记 `research/prompts/e171-preregistration.md`（修订 1–5）与问题单 `research/prompts/three-oracle-collusion-questions.md` 跑完了干净整表（117 行，s4）与 F4 扩表（164 行），编不过的 6 行补位那一趟（产物与读数见第三节末）。D13（验证路线） 已定项 6 要求「独立性要被验证，不许自称」：「若三者全部放行，说明存在共同盲区，回头补 oracle」。这一轮判的是推论，不是读数：

1. 执行员三分出来的「真盲区候选」各自是不是真盲区——坏实现写出的状态在哪一句已定条款下非法、条款读得对不对、这一处算不算三条 oracle 的**共同**盲区（D13（验证路线） 已定项 6 表最后一列「验不到什么」是不是已经承认了它）；
2. 判为「对这一批等价」「取样点不敏感」的几行站不站得住；
3. O3（独立规约执行器） 在全部计入分母的行上 0 红、它的允许集合 A(c) 由实现自己写出的根构成——这削不削 D13（验证路线） 已定项 6 写给 O3 的「独立性来自哪里」；
4. 第 155、161 行照登记读法 4 是「未触达」，O1-对拍却红（成功调用的交回值不同）——触达定义的缺口影不影响 Q1、Q3 的判；
5. 由 1–4，D13（验证路线） 已定项 6 与 C407（三 oracle 合谋测试没有代码） 该怎么改：依据能不能引 E171（三oracle合谋测试）、关系写支撑还是推翻、「回头补 oracle」补在哪一条、C407 还清还是改写。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- O1（内存模型对拍（HashMap 理想文件系统））：`crates/singlefs-harness/src/model.rs` 第 594 行 `ObservedRoot`（根的 key、journal 计数器、根带的 F、文件内容、实例表、分配记录）；`write_time` 在 `model.rs`、`model_comparison.rs` 零命中（`grep -n write_time crates/singlefs-harness/src/model.rs crates/singlefs-harness/src/model_comparison.rs` 无输出）。E171 装置里 O1 分两列：O1-对拍（逐调用比交回值）与 O1-崩溃判官（实现恢复落到的那一版合不合法）。
- O2（独立解析器 + checker）：`crates/singlefs-checker`，只依赖 `singlefs-format`；系统配置槽的 `journal_tail` 在 `crates/singlefs-checker/src/lib.rs` 第 286 行读出（执行员报告说读了没有判它的不变量，主 agent 没逐条核 `invariants.md`）。
- O3（独立规约执行器）：`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/`（冻结种子 85441ae491419411）；`version_table.py` 文件头：版本表 = 基镜像上的现行那一版加「录制流里每一次根槽写按流序各一版」，A(c) 由这张表算。
- 坏实现：E171 装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` 逐行把 `crates/mutations.tsv` 里 `crates/singlefs-core/src/` 与 `crates/singlefs-replay-prefix/src/lib.rs` 的变异打进实现（挑法登记 5.2：按文件分组、步长 8 取 117 行，F4 步长 4 多取 164 行），在新池新建文件与固定脚本到 E 再卸载两条流上录写流，层 0 切段取样两批，O1、O2、O3 各判。
- 方案按 `crates/` 今天的实现来谈。

## 三、读数（主 agent 现查产物，整行抄）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`：

```
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=232 only_in_the_ignored_pass=0 compile_failed=6 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
E7RESULT name=q1_o1_and_o2_both_quiet counted_rows=37 rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 o3=run
E7RESULT name=q1_gamma counted_rows=37 all_quiet_rows=13 row_numbers=15,23,27,34,42,92,112,233,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=37 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q3_beta caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=untouched_but_red rows=2 row_numbers=155,161
```

逐行三分（执行员做的，主 agent 只核了引的条款行在不在）在实验页 `.claude/kb/experiments/171-三oracle合谋测试.md` 的「三条全放行的 7 行」「F4 新出的全放行行」「F4 新出的只靠一条的 12 行」「没触达却红的两行（第 155、161 行）」几节，执行员报告 `/tmp/claude-1000/e171-segment7/report.md` 第二、三节与 `/tmp/claude-1000/e171-segment8/report.md` 第二至四节。真盲区候选与它们依的条款：

| 行 | 坏实现 | 执行员依的条款句 | 状态在哪 |
|---|---|---|---|
| 23 | inode 叶容器号 1 → 0 | D8（核心索引结构） 已定项 6「一个码 3 叶（类型 2、容器号 1……）」与射程「每个容器号都是某条记录首次插入时的 inode 号」 | 择到新版的态 |
| 42 | 正常卸载把 F 抬到现行那一版带的 F，不抬到它的 txg | D16（发布语义） 已定项 1 字段表「正常卸载」「推一串空发布把 F 抬到现行那一版的 txg」 | 卸载写的系统配置与空发布的根 |
| 92 | 覆盖写照抄上一版的写入时间 | 「写入时间是发布参数」只在 `.claude/kb/milestone/01-first-txn.md` 与 `.claude/kb/milestone/02-second-txn.md` 第 103 行；决策正文只有 D8（核心索引结构） 已定项 6 字段表的位置 | 覆盖写的 inode 记录 |
| 242 | 系统配置槽轮换挪到根槽 FUA 之前 | D16（发布语义） 已定项 7「根槽（FUA）→ 系统配置槽 → 屏障」「系统配置槽在根槽持久之后再更新」 | 系统配置先落、它那一段根没落的态（层 0 切段模型上数） |
| 269 | 取号见证值取成槽世代号 | D23（journal的角色与格式） 已定项 14「mkfs 之后第一次取号读到 0、写 0」 | 取号写的系统配置槽 |
| 270 | 系统配置轮换之后不过屏障就返回 | D16（发布语义） 已定项 7「fsync 等系统配置轮换持久之后才返回」与射程「屏障口径」 | 后一次调用的写先于前一次的系统配置落盘的态 |
| 271 | `deleted_inodes` 树表条目头 ID 写 0 | D8（核心索引结构） 已定项 8「`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」 | 择到新版的态 |
| 233（只在读法丙） | 由记录重建的根不照抄所选根的 fsid | D13（验证路线） 已定项 1 M2 与「其余字段等于所选根」 | 施加了记录的态；甲乙里 O1-崩溃判官红，批 2 没有这种态 |

判为对这一批等价：15、27、34、112、277；取样点不敏感（两批红列不同）：102、233、265。

补位那一趟（6 行）跑完：`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`「E7RESULT name=substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0 row_numbers=33,59,127,163,206,243」，六行换上的都是 `category=untouched`；补位之后「q1_denominator rows_run=281 counted_D=37 untouched=238 … compile_failed=0 …」，全放行、只靠一条、没触达却红三张名单与补位之前逐字相同。

## 四、格

### G1　真盲区候选逐行（23、42、92、242、269、270、271；丙里 233）
每一行：条款句读得对不对（整节读，不只读结论句）；执行员数的「不合法的态」是不是真的不合法（242、270 是在层 0 切段模型上数的，FUA 写只放行它自己那块盘）；这一处是不是**三条的共同**盲区——D13（验证路线） 已定项 6 表「验不到什么」一列是不是已经写明某条 oracle 验不到它（写明了的，算不算「共同盲区」）；要补，补在哪一条 oracle、补什么（O1 的 `ObservedRoot` 加字段、O2 加一条不变量、O3 投影加字段）。92 那一句只在里程碑页：里程碑页的验收句算不算「条款」。

### G2　等价与取样点不敏感
15、27、34、112、277 的「每个态连同实现的恢复都在条款允许的集合里」站不站得住；执行员推的「会分开的历史」是不是真会分开；102、233、265 记「不稳定」对不对。

### G3　O3 的 A(c) 取自实现写出的根
O3 在 37 行上 0 红、每行都有严格判定（`q1_o3_coverage rows_with_no_strict_judgement=0`）。A(c) 由「录制流里每一次根槽写」构成，这是不是让 O3 对「实现写了一个错的根」整类无感；D13（验证路线） 已定项 6 写 O3「只照 decisions 与 invariants 的文字重新推导」，这一读法下它推导的是什么、不推导的是什么；这一点是 O3 的设计边界（该写进「验不到什么」）还是缺陷（该改）。

### G4　触达定义与测量方法
155、161 照读法 4 未触达而 O1-对拍红：若算触达，D = 39、Q3 只靠 O1 多两行，Q1 不变（执行员推的）——这一判对不对。坏实现表取自 `crates/mutations.tsv`（每一行本来就被某条测试抓过）：这对 Q1「有共同盲区」的判与对 Q3 的判各偏向哪一边；两条流、两批层 0 取样的域外推到哪里为止。

### G5　D13（验证路线） 已定项 6 与 C407（三 oracle 合谋测试没有代码） 怎么改
由 G1–G4：已定项 6 的依据能不能引 E171（三oracle合谋测试）、实验页「影响的决策」那一行写支撑、推翻还是仍是备料；「回头补 oracle」这一句今天落成什么（逐条补哪几处，还是登记欠账）；O3 的「验不到什么」要不要加句；C407 还清（它的改法要求做成门禁阶段：已知坏实现至少一条红、关一条仍有别的腿红）还是改写成什么。

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | G1–G5 | 从条款原文与产物推：每个候选是不是真盲区、O3 的边界在哪、D13 已定项 6 与 C407 该怎么写；给出每一格的推导与它依的原文行 |
| 云端攻方（Opus） | G1–G5，重点 G1、G3 | 攻执行员的三分与主 agent 读数：哪个「真盲区」其实合法或其实有一条 oracle 判得到、哪个「等价」其实有非法态、O3 的 A(c) 是不是让「共同盲区」的数被低估或高估；可以在 `crates/` 与 E171 导出的拷贝上造反例（例：只让 O3 红的坏实现、让 O2 判出 242 的那一句不变量），不重跑整表 |

## 六、跑前写死的判据

- 两腿对一个候选同判且主 agent 现查坐实 ⇒ 按那一判写；不一致 ⇒ 先核能分辨它们的那个事实，再判；核不动的写明、该候选挂起、进第二轮。
- 一个候选判为真盲区，必须写出：非法依的条款原文行（kb 文件自己的行号）、三条各自为什么放行（指到代码行）、补在哪。
- 腿提的新改法（新不变量、新投影字段、改登记）算零轮。

交付：腿的报告写 `research/prompts/e171-conclusion-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/e171-conclusion-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
