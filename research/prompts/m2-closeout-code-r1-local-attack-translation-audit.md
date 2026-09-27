# m2-closeout-code-r1 本地攻方腿 提示翻译核对表

栏位：英文项 / 原文文件:行 / 首稿缺的 / 定稿

## 分工表原句（正文 _m2-closeout-code-r1-body.md:96）

原文：「按事实表逐格填数、与仓里钉的数比：第一条流全量 150994980 与快档 54、第二条流 14960689284 与快档 390 的闭式怎么来（按段序列与每段写数，第三态记一格）；A4b 量表里 `two-4GiB`、`two-1GiB`、`two-4GiB-mapping-4-8` 三格的 ckpt_cost 各项；头宽 `86 + 2 × k + 29` 在 k = 8、24 上；一条记录装几个点名项」

1. 英文项：任务开场句「fill in every cell ... show the arithmetic step ... state whether it matches the pinned value」
   原文文件:行：_m2-closeout-code-r1-body.md:96「按事实表逐格填数、与仓里钉的数比」
   首稿缺的：无（首稿即含「show the arithmetic」「state whether it matches」两处，对应原文「填数」与「比」）
   定稿：同首稿

2. 英文项：Fact set A 任务句「how the closed forms ... are derived — by segment sequence and the write count of each segment, with the third state recorded as its own column」
   原文文件:行：_m2-closeout-code-r1-body.md:96「闭式怎么来（按段序列与每段写数，第三态记一格）」
   首稿缺的：首稿最初把「第三态记一格」译成「with the third state noted」，丢了「记一格」里「独立一格/单列」的量词——补成「recorded as its own column」，对应表格里单独一列
   定稿：「with the third state recorded as its own column」

3. 英文项：Fact set C 开场句「The ckpt_cost line items for the three A4b measurement-table cells」
   原文文件:行：_m2-closeout-code-r1-body.md:96「A4b 量表里 ... 三格的 ckpt_cost 各项」
   首稿缺的：无
   定稿：同首稿

4. 英文项：Fact set D「The header-width formula 86 + 2×k + 29 evaluated at k = 8 and k = 24」
   原文文件:行：_m2-closeout-code-r1-body.md:96「头宽 `86 + 2 × k + 29` 在 k = 8、24 上」
   首稿缺的：无（式子与两个取值原样照抄）
   定稿：同首稿

5. 英文项：Fact set E「How many named entries fit in one record」
   原文文件:行：_m2-closeout-code-r1-body.md:96「一条记录装几个点名项」
   首稿缺的：无
   定稿：同首稿

## crash.rs 与测试文件内代码注释的英译（正文分工表指向「按段序列与每段写数」，注释是算法本体）

6. 英文项：A2「closed_form_state_count(segments) = 1 + sum over each segment of (2^(segment length) - 1)」
   原文文件:行：crates/singlefs-harness/src/crash.rs:622-625（`1 + segments.iter().map(|segment| (1u64 << segment.len()) - 1).sum()`）；中文文档句在同文件 617 行「E77（发布的持久顺序） 的闭式：1 + Σ(2^|段| − 1)，每次写只取两态（没持久 / 持久）时的全量」
   首稿缺的：首稿漏了「每次写只取两态（没持久/持久）」这个限定语境，只译了公式本身——补成 A2 引言句「every write treated as landed-or-not, no torn state」
   定稿：「every write treated as landed-or-not, no torn state」+ 公式原样

7. 英文项：A3「For a segment of n writes of which m are torn in-place overwrites, the state count is 3^m * 2^(n-m) - 1 ... When m = 0 this reduces to 2^n - 1 (every proper subset).」
   原文文件:行：crates/singlefs-harness/src/crash.rs:1722-1723「段内每次写各取它的几态、任意组合，去掉整段全持久那一个... n 个写里 m 个是原地覆写时 3^m · 2^(n−m) − 1 个；m = 0 时就是任意真子集 2^n − 1 个」
   首稿缺的：无，两句都译了（组合去掉整段全持久 = "minus the one case where the whole segment is fully persisted"；m=0 退化情形也译了）
   定稿：同首稿

8. 英文项：A4「Of this stream's writes, only 8 system-configuration-slot writes take the third (torn) state. Four segments consisting purely of system-configuration-slot writes each contribute 3^2 - 1; the segment named A ... contributes 3^2 * 2^24 - 1; every other segment is as in the two-state case; then add 1 for the all-persisted state.」
   原文文件:行：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs:437-438「这条流上取三态的只有 8 次系统配置槽写，四个系统配置槽段各 3² − 1、A 那一段 3² · 2²⁴ − 1，其余同两态，再加全部持久那一个」
   首稿缺的：无
   定稿：同首稿

9. 英文项：A5「Within a segment, each in-place write ... multiplied by the unit (copy-on-write) writes taken only as either none-landed or all-landed ... With k in-place writes (of which m are torn overwrites) and c unit writes, the count is 3^m * 2^(k-m) * (2 if c > 0 else 1) - 1; when m = 0 this is 2^(k+1) - 1 if c > 0, or 2^k - 1 if c = 0.」
   原文文件:行：crates/singlefs-harness/src/crash.rs:1725-1728「段内原地写（单元写之外的写）各取它的几态、任意组合 × 单元写（COW）只取全不落或全落，去掉整段全落那一个。原地写 k 个（其中 m 个是原地覆写）、单元写 c 个：3^m · 2^(k−m) ·（c > 0 时 2，否则 1）− 1 个，m = 0 时 c > 0 是 2^(k+1) − 1、c = 0 是 2^k − 1」
   首稿缺的：无
   定稿：同首稿

10. 英文项：A6「For segment A: 2 in-place writes (system-configuration-slot writes, each three-state) times unit writes taken as none-landed-or-all-landed gives 3^2 * 2 - 1 = 17. The four system-configuration-slot-only segments each contribute 3^2 - 1 = 8. Every other segment is as in the full enumeration ... Then add 1 for the all-persisted state.」
    原文文件:行：crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs:440-441「A 那一段 2 原地写（系统配置槽写，各三态）× 单元写全不落或全落 3² · 2 − 1 = 17，四个系统配置槽段各 3² − 1 = 8，其余每段同全量（2 写段 3、1 写段 1），再加全部持久那一个（补第三态之前 29）」
    首稿缺的：首稿漏译了括注「（补第三态之前 29）」——这是「加一句多余信息」还是「原文该译」？判定：这句是给「甲二快档」旧值的对照，不影响本题要算的当前闭式，且正文分工表没有要求核对「补第三态之前」那一档，删掉不算漏译对象本身，但为了不让模型误用旧值做计算，补充说明写进任务句里的量词已经是「Then add 1 for the all-persisted state」（这是新值的收尾，不是旧值），旧值括注确认不译，因为它是「今天不判的一个历史对照值」，不是这道题要填的格
    定稿：不译「补第三态之前 29」这一括注；A6 正文照上面英文项

11. 英文项：B3「On this stream, only 46 system-configuration-slot writes take the third (torn) state. Each segment contributes 3^m * 2^(n-m) - 1 ... then add 1 for the all-persisted state. The total is about 2.25 times the two-state count. A 30-write segment contributes 3^2 * 2^28 - 1.」
    原文文件:行：crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:89-90「这条流上取三态的只有 46 次系统配置槽写，每段 3^m · 2^(n−m) − 1（m 是段里系统配置槽写的个数），再加全部持久那一个；约为两态时的 2.25 倍（30 写段 3² · 2²⁸ − 1）」
    首稿缺的：首稿漏译了「（m 是段里系统配置槽写的个数）」这个括注定义——补成「where m is the number of system-configuration-slot writes in that segment and n is the segment's total write count」（n 的定义是本轮追加的括注，因为原文只定义了 m、没重复定义 n，n 在上文 A3/A5 已经定义过，这里补一句是为了让本条独立可读，不是原文没说而我加了新限定，是把上文已定义的 n 复述一遍）
    定稿：补上「and n is the segment's total write count」，并在核对表里单列一条见下第 13 条「多出来的」

12. 英文项：B4「if every write were treated as two-state only (no torn state), this stream's quick-tier count would be 205」
    原文文件:行：crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:99「甲二快档的状态数（`crash::layer0_state_count_with_torn_in_place_overwrites` 按甲二的闭式；每次写只取两态时 205）」
    首稿缺的：无（两态对照值这里保留了，因为 B4 任务句要模型自己算的是「390」这个带三态的数，205 只是给它核对函数名指向的注解，不进它要填的答案格）
    定稿：同首稿

13. 英文项：C1「ckpt_cost for the checkpoint reserve pool: ... always taken as the worst case. Allocation record tree: ... so devices * 2 * (height - 1) + 1. Central mapping tree: ... Accounting tree: ... Tree table: 1 ... The instance-table chain does not enter ckpt_cost.」
    原文文件:行：crates/singlefs-core/src/admission.rs:476-481「checkpoint 保留池的 ckpt_cost（D28（挂载期承诺量） 已定项 4）：一次空发布至多写出的固定点单元数，按这次发布要接在后面的那一版的结构现算，一律按最坏情况计...分配记录树：每块盘两条从叶到根之下那一层的路径加共用的根，盘数 × 2 × (高 − 1) + 1...中央映射树：逐层按这次可能改的路径数与可能切出来的节点数计...记账树每次发布整批重写...这一版记账树的节点数；树表 1...实例表链不进」
    首稿缺的：首稿漏译了「（用户 2026-09-27 定：游标跨叶、中央映射树多层都不许少扣；此前按盘分路的那一版只罩「每块盘一片叶、中央映射树 1 层」）」这条历史对照括注——判定：这是给「为什么改公式」的背景，不影响本题要算的当前公式数值，本轮任务只问「今天的公式算不算得出钉的数」，不问「公式为什么这样改」，故不译入 C1；同理漏译「（它的开销归已定项 3 的切换预留）」——这句解释「为什么实例表链不进 ckpt_cost」，本题 C1 只需要「不进」这一事实（模型要用它排除掉一项、不需要用它加另一项），故为什么不进不译入正文，但在这里记录不译的理由
    定稿：C1 只保留「what」（公式与不进的事实），不译「why」（历史对照与理由）两处括注

14. 英文项：C2「Formula: devices * 2 * (tree_height - 1) + 1. Worked example given in the source: two 4 GiB devices at height 3 gives 9.」
    原文文件:行：crates/singlefs-core/src/admission.rs:579-580「⇒ 盘数 × 2 × (树高 − 1) + 1。两块 4 GiB 盘高 3 时 9 个（每块盘一片叶的那几次实写 5 个... `second_transaction_supplement_one_write_accounting` 钉的 `ALLOCATION_RECORD_TREE_NODES_REWRITTEN`；跨叶那一次实写 7 个）」
    首稿缺的：首稿漏译了括注里「实写 5 个」「跨叶那一次实写 7 个」两个对照数——判定：这两个数是「实际观测值」与「公式给的上界」的对照（公式给 9，是两种实写场景 5、7 的上界），本任务 C2 只要公式与其中一个已给的验证点（高3盘2给9）用来核对公式对不对，不需要模型再核实写值，故不译入正文，理由记在这里
    定稿：C2 只保留公式与「两块 4 GiB 盘高 3 时 9 个」这一验证点，不译两个实写对照数

## 多出来的（英文比原文多的限定词或括注）

15. Fact set A 与 B 任务句结尾的「an ambiguous row that you resolve without flagging it counts as an unflagged assumption, which is a different kind of error from a wrong sum」
    这句原文没有——是本轮新加的提示纪律，不是转述。加它的理由：任务要求模型逐行判定 m（第三态写的个数），而正文与代码注释都没有给出「每一段具体是哪种写」的逐段列表（只给了汇总说明「四个系统配置槽段」「46 次系统配置槽写」），模型要在事实不完整的情况下自己分类；不加这句，模型可能悄悄替一个有歧义的段挑一个分类而不说明，回复看起来是「填满的表格」但实际藏着未声明的假设，主 agent 事后判不出它是算错了还是猜错了输入。

16. Fact set A、B、D、E 每题结尾统一加的「what observation would show this item is wrong」
    原文分工表没有这句——按 `.claude/rules/three-way-inference.md` 与共用约束「结论写『什么现象会推翻它』」的要求，本地腿的每条结论同样要交这个，故补上，不算转述失真。
