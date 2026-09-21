# C48/C49「多条校验路径」登记位——普查（sweep，2026-09-21）

只普查、不改文件、不提议登记形态。范围：`.claude/kb/experiments/*.md` 里
`grep -rlE '互不共享|独立的.*路径|两条路径|三条路径|两条独立|三条独立'` 命中的全部 15 份。

## 普查范围（完整名单，现查）

```
$ grep -rlE '互不共享|独立的.*路径|两条路径|三条路径|两条独立|三条独立' .claude/kb/experiments/*.md | sort
.claude/kb/experiments/07-离线索引harness.md
.claude/kb/experiments/110-条带表在连续发布下的期望写放大.md
.claude/kb/experiments/113-不认识的树-补齐臂集.md
.claude/kb/experiments/12-攒批的顺序追加vs不攒批的随机页读改写.md
.claude/kb/experiments/14-I-3.1的判别力幂等完整值vs增量Δ.md
.claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md
.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md
.claude/kb/experiments/21-GPU卸载的净收益.md
.claude/kb/experiments/32-断号即止会不会重放上一条时间线.md
.claude/kb/experiments/45-事务跨多条记录的环占用.md
.claude/kb/experiments/47-根环失败域的损失.md
.claude/kb/experiments/56-消息缓冲的收益vsε.md
.claude/kb/experiments/58-校验和粒度的端到端代价.md
.claude/kb/experiments/86-扫描步进与两种单元大小.md
.claude/kb/experiments/93-老化下的放置与碎片度.md
```

派发提示点名的 10 份（07/12/14/21/32/45/56/58/93/110）之外，其余 5 份是
110（已在点名单里）之外的 **113、154、155、47、86**。共 15 份，与命令输出条数一致（`wc -l` = 15）。

## 方法说明（先说清怎么判的，免得表看着像自动生成）

第 1 列没有机械判据——C48 自己的 2026-08-30 结论就是「按函数名认是猜的」。
这张表是**逐份读正文 + 逐份读对应的 `research/e7-index-bench/src/bin/*.rs`** 判出来的，
不是跑一条命令筛出来的。第 3 列的 file:line 全部现查，命令与输出见下面各份小节。

**重要提醒**：**grep 命中的那一行本身，往往不是真结构所在的地方。**
15 份里有 5 份（07、56、58、110、155）grep 命中的原句是别的意思（三方论证的「腿」、
两条设计臂本身、B 树结构路径、WAL 建模里的「不共享路径」……），
而页面别处另有一段真正的「同一个量两条路径算一遍再比对」结构。
这张表两栏都写：**grep 命中的那句说的是什么**，以及**页面里真实结构（如果有）在哪。**

## 总表

| 实验页 | ① 是不是真的多条路径互证 | ② 路径在正文里有没有名字 | ③ 源码那一侧认不认得出 |
|---|---|---|---|
| E07 离线索引 harness | 是（但不在 grep 命中处；见下） | 有名字，无 E45 式统一表 | 部分：block 计数器/程序计数器/sorted_bplus 闭式都在源码；logstruct_wb 闭式（≈1.56）全仓无代码 |
| E12 攒批的顺序追加 vs 不攒批 | 是（grep 命中处即真结构） | 有名字（块层读数/实测本身/推算），有对照表 | 部分：块层读数与实测本身在源码；「推算」（首性原理闭式）全仓无代码 |
| E14 I-3.1 判别力 | 是 | 有名字，源码注释自陈「三方，不是两方」 | 认得出，且比对断言也在源码里 |
| E21 GPU 卸载净收益 | 否——三种不同性质的证据证「GPU 跑过」，不是同一个量算两遍 | 不适用 | 不适用 |
| E32 断号即止 | 否——「两条独立的腿」是三方论证的推理腿，不是代码路径 | 不适用 | 不适用 |
| E45 事务跨多条记录的环占用 | 是（C48 的原案例；且已被实验自己推翻「独立」这个说法） | 有名字+专门一节`\| # \| 路径 \| 怎么算 \|`表 | 认得出，比对与故障注入都在源码里 |
| E47 根环失败域的损失 | 否——「两条独立路径」说的是 E46/E47 两个实验别共享代码，不是 E47 内部互证 | 不适用 | 不适用 |
| E56 消息缓冲的收益 vs ε | 是（但不在 grep 命中处；见下） | 有名字（臂 A/B/C），无统一表 | 部分：闭式与实测臂函数都在源码，但断言比的是闭式对字面常量，不是闭式对同次实测输出 |
| E58 校验和粒度的端到端代价 | 是（第 54 行是真结构；第 110 行是已撤回的假互证，见下） | 有名字（内核 read_bytes / 程序自算 dev_bytes） | 认得出各路径，但比值本身只在 main() 打印，没有单测断言 |
| E86 扫描步进与两种单元大小 | 是 | 有名字（扫描重建漏数 / 独立统计奇槽节点数） | 认得出，比对断言在源码里 |
| E93 老化下的放置与碎片度 | 是 | 有名字（增量 O(1) 维护的 runs / 全扫审计） | 认得出，比对与故障注入都在源码里 |
| E110 条带表连续发布写放大 | 是（但不在 grep 命中处；见下） | 有名字（单次发布算术 / E107 已钉的两个端点） | 认得出，比对断言在源码里 |
| E113 不认识的树·补齐臂集 | 否——「两条独立保证」指只读安全/可写安全两类不同保证要分开判 | 不适用 | 不适用 |
| E154 两道闸串 | 否——「两条独立验证过的路径」是在说 H4a/H4b 本该分测却退化成同一条代码路径（覆盖面缺口，不是互证） | 不适用 | 不适用 |
| E155 每次持久化的写量 | 是（但不在 grep 命中处；见下） | 无「路径」命名，称「手算格」vs「通用实现」 | 认得出，比对断言在源码里 |

## 逐份详情

### E07 离线索引 harness（`.claude/kb/experiments/07-离线索引harness.md`）

grep 命中 2 处，行 69、287：
```
69:#### 两条解析对照（第三条独立路径）
287:E7（离线索引 harness）列的第三条独立校验路径是「两条解析式与实测吻合（1.9375 vs 1.9378、约 1.56 vs 1.5755）」，
```
真结构有两层：① 三条硬要求第 3 条「块层独立读数 + 该校验路径本身有判别力」——程序自己的 I/O 计数器
与 `/sys/block/vda/stat` 五轮九格相符，摘掉 `O_DIRECT` 后块层读数归零而程序计数器不变（60/60 格相符）；
② 「两条解析对照（第三条独立路径）」——`sorted_bplus` 闭式 1.9375 对实测 1.9378、`logstruct_wb` 闭式约 1.56 对实测 1.5755。
2026-08-29 对抗验证那一节自己承认：「**它钉的是解析式与几何常量，不是实测值**——实测要设备与虚机，单测里跑不了，两者的比对仍然只能靠复跑」。

源码现查：
```
$ grep -n "fn read_block_layer_counters\|the_no_batching_control_arm_has_an_analytic_io_per_op_of_1_9375\|1\.56\|logstruct_wb" research/e7-index-bench/src/bin/e7_index.rs
314:fn read_block_layer_counters(device_path: &str) -> Option<BlockLayerCounters> {
377:    for (name, which) in [("sorted_bplus", 0), ("logstruct_wb", 1), ("betree", 2)] {
448:    fn the_no_batching_control_arm_has_an_analytic_io_per_op_of_1_9375() {
```
- 块层计数器路径：`read_block_layer_counters`（research/e7-index-bench/src/bin/e7_index.rs:314），
  程序自身计数是 `device.reads`/`device.writes` 字段；两者只在 `main()` 378–404 行打印比对（`block_counter_delta`），无单测断言。
- `sorted_bplus` 闭式路径：`the_no_batching_control_arm_has_an_analytic_io_per_op_of_1_9375`（e7_index.rs:448），
  内联算出 `analytic = 1 + (1 − DEFAULT_CACHE_PAGES/LEAF_COUNT)`，但断言对象是几何常量算出的 1.9375 本身，
  不是「这个值等于同一次跑里的实测 1.9378」——跨路径比对没有被断言钉住。
- `logstruct_wb` 闭式路径（约 1.56，来自「512 个随机 key 落到约 400 个不同叶子」的手算）：
  **认不出**——`research/e7-index-bench/src/bin/e7_index.rs` 全文搜不到任何函数或测试实现这条推导，
  它只活在 kb 正文的散文里，卡在「路径不在源码里，在正文的算术里」。

### E12 攒批的顺序追加 vs 不攒批的随机页读改写（`.claude/kb/experiments/12-攒批的顺序追加vs不攒批的随机页读改写.md`）

grep 命中 1 处，行 175：
```
175:并且**推算与实测吻合到三位有效数字**——这构成第三条独立校验路径
    （另两条是块层读数、以及实测本身）。
```
这处就是真结构：三条路径——**块层读数**（内核块设备计数器）、**实测本身**（程序内计数器/臂函数输出）、
**推算**（首性原理闭式：臂 A `io = ⌈进 deadlist 总字节 / 缓冲区⌉`、臂 B `io_per_op = 2×(1 − 缓存页数/表页数)`）。
臂 A 例：推算 1,600,000 字节 vs 实测 `bytes_w=1597440`，差 0.16%；臂 B 例：解析 io/op 与实测 io/op 在 fits/2×/8× 三档逐格接近
（0.000/0.003、1.000/0.998、1.750/1.751）。

源码现查：
```
$ grep -n "fn read_block_layer_statistics\|fn arm_time_order_deadlist\|fn arm_reference_count\|fn arm_hybrid" research/e7-index-bench/src/bin/e12_lifecycle.rs
25:fn read_block_layer_statistics(device_path: &str) -> Option<BlockLayerStatistics> {
134:fn arm_time_order_deadlist(device: &mut Device, operation_count: u64, seed: u64, deadlist_base: u64) -> IoCounters {
162:fn arm_reference_count(
202:fn arm_hybrid(
$ grep -n "1597440\|analytic\|io_per_op\|#\[test\]" research/e7-index-bench/src/bin/e12_lifecycle.rs | grep -c test
5
```
- 块层读数：`read_block_layer_statistics`（research/e7-index-bench/src/bin/e12_lifecycle.rs:25），
  与实测本身（`arm_time_order_deadlist`:134、`arm_reference_count`:162、`arm_hybrid`:202）在 `main()` 里打印比对（`block_layer_delta`），无单测断言。
- 「推算」路径：**认不出**——`e12_lifecycle.rs` 现有 5 个 `#[test]`，没有一个计算
  `2 × (1 − 缓存页数/表页数)` 或校验 `1,600,000` 这个手算字节数；这条推导完全只活在 kb 正文，
  卡在「名字对不上：kb 说的『推算』这个动作，源码里根本不存在对应函数」。

### E14 I-3.1 的判别力：幂等完整值 vs 增量 Δ（`.claude/kb/experiments/14-I-3.1的判别力幂等完整值vs增量Δ.md`）

grep 命中 1 处，行 46：
```
46:  checker 遍历内容求和，与运行时那条「入缓冲时增量维护」的代码**不共享任何东西**——
```
真结构：运行时计数器（`runtime_total`）、checker 独立重算（`checker_total`）、真值（`truth`）三路，
「幂等完整值」形态下 checker 遍历内容求和、不碰运行时的合并函数；「增量 Δ」形态下账本身就是权威内容，
checker 只能重放同一份 Δ 日志、共用同一个累加函数——这正是 E14 的核心发现（判别力归零 vs 判别力健全）。

源码现查：
```
$ grep -n "三方，不是两方\|struct Reading\|fn run_idempotent\|fn flush_idempotent\|fn run_delta\|fn accumulate\|fn ground_truth" research/e7-index-bench/src/bin/e14_discrimination.rs
6://! 三方，不是两方：运行时计数器 / checker 重算 / 真值。
43:fn ground_truth(operations: &[Operation]) -> i64 {
52:struct Reading { runtime_total: i64, checker_total: i64, truth: i64, entries: usize }
57:fn flush_idempotent(buffered_entries: &[(u64, Option<i64>)], tree: &mut BTreeMap<u64, i64>, bug: Bug) {
72:fn run_idempotent(operations: &[Operation], bug: Bug) -> Reading {
99:fn accumulate(accumulator: &mut i64, delta: i64, bug: Bug) {
103:fn run_delta(operations: &[Operation], bug: Bug) -> Reading {
```
比对本身也在源码里：`main()` 第 143 行 `if reading.runtime_total != reading.checker_total { invariant_fired_count += 1; }`，
测试 `both_forms_are_correct_without_any_bug`（:172）两次 `assert_eq!(reading.runtime_total/checker_total, reading.truth, ...)`。
**认得出，全链路。**

### E21 GPU 卸载的净收益（`.claude/kb/experiments/21-GPU卸载的净收益.md`）

grep 命中 2 处，行 76、87，同一段：
```
76:### GPU 确实在工作：三条独立证据（2026-08-28 补证）
87:三条独立证据，任意一条单独都够：
```
**判「否」**：这三条是「占用率对照（0%→100%）」「功耗（11.67W→352W）」「物理不可能（CPU 35.6 GB/s、GPU 1037.3 GB/s，
本机内存带宽封顶 64.91 GB/s，CPU 造不出这个数）」——三种**不同性质**的证据共同证明「GPU 确实被使用过」这一**存在性命题**，
不是「同一个量（比如某个吞吐数字）用两条以上互不共享代码的路径各算一遍、再比对」。C48 管的是后者。

### E32 断号即止会不会重放上一条时间线（`.claude/kb/experiments/32-断号即止会不会重放上一条时间线.md`）

grep 命中 1 处，行 141：
```
141:  ⚠️ **但这个空缺已由两条独立的腿各自推完，结论是不影响**：
```
**判「否」**：「两条独立的腿」是 `.claude/rules/three-way-inference.md` 意义上的三方论证推理腿
（前一节「三方论证的腿况」表里「云端（正推）」「本地（找反例）」两行），不是校验路径。
紧接着那句「把 tail 推到 `hole + 新写条数` 重算，被重放的残留条数与固定 tail 那一栏逐格相同」是一段**文字推理**
（`tail ≤ min(残留 jsn) 恒成立`），不是另一套独立实现的代码在跑；全文再搜不到第二处「独立」/「路径」。

### E45 事务跨多条记录的环占用（`.claude/kb/experiments/45-事务跨多条记录的环占用.md`）

grep 命中多处（8/30/40/56/75/82/115 行），核心是专门一节：
```
30:### 三条互不共享代码的验证路径
```
| # | 路径 | 怎么算 |（正文表，逐字抄）|
|---|---|---|
| 1 | 正推：字节级摆放 | 真的把记录一条条摆进一个按字节寻址的环，走游标 |
| 2 | 校验 A：闭式算术 | `记录条数 × ceil((头 + 每条项数 × 项宽) / 单元) × 单元`，独立写，不调用路径 1 |
| 3 | 校验 B：对上 E23 已发表的表 | 断言本模型复现 140→512、644→1024、5684→6144 三格 |

这是 C48 本身援引的案例，且已被 2026-08-30 攻击轮自己推翻「独立」这个说法：路径 1、2 共享同一条**错的**对齐规则
（会一起犯错、一起逐格相等）；路径 3 不是真的跨二进制复现——140/644/5684 不在任何 `.out` 产物里，
是从 E23 正文**手算表抄成字面量**，只锚定两个常量，对被验的倍数判别力为零。

源码现查：
```
$ grep -n "fn laid_out_bytes\|fn closed_form_bytes\|fn e23_unpadded_record_bytes\|fn e23_padded_record_bytes\|#\[test\]\|fn all_three_paths_agree\|fn reproduces_the_published\|fn cells_match_independently\|fn the_cross_check_catches" research/e7-index-bench/src/bin/e45_span_ring_cost.rs
51:fn laid_out_bytes(total_items: u64, items_per_record: u64, atomic_unit_bytes: u64) -> u64 {
63:fn closed_form_bytes(total_items: u64, items_per_record: u64, atomic_unit_bytes: u64) -> u64 {
78:fn e23_unpadded_record_bytes(items: u64) -> u64 { ... }
79:fn e23_padded_record_bytes(items: u64, atomic_unit_bytes: u64) -> u64 { ... }
147:    fn all_three_paths_agree_on_every_cell() {
159:    fn reproduces_the_published_e24_alignment_table() {
173:    fn cells_match_independently_computed_arithmetic() {
253:    fn the_cross_check_catches_an_injected_bias() {
```
**认得出，全链路**：三条路径各自的函数与比对断言都在源码里，且有故障注入测试
（`the_cross_check_catches_an_injected_bias`，往路径 1 注入已知偏差验证路径 2/3 会红）。
路径 3 对应的函数 `e23_unpadded_record_bytes`/`e23_padded_record_bytes` 也在源码里，
但它锚定的只是两个常量（`BASE_RECORD_HEADER_BYTES`、`NAMED_ITEM_BYTES`）——**能定位到代码，不代表判别力真的够**，
这正是 kb 正文自己推翻它的理由。

### E47 根环失败域的损失（`.claude/kb/experiments/47-根环失败域的损失.md`）

grep 命中 2 处，行 73、162（同一句复述两次）：
```
73:但**E47（根环失败域的损失）自己重写一份并各自单测**——两处若共用同一段代码，
   E46（根环区域间距与失败域） 与E47（根环失败域的损失）就不再是两条独立路径。
```
**判「否」**：这是在提醒 **E46 和 E47 是两个不同的实验**，测的是不同的东西（失败域间距 vs 环损失），
但都用了同一条落盘归属算术（`device_of`），如果 E47 直接调用 E46 的实现，两个实验的结论就共押同一段代码；
E47 因此自己重写一份并各自单测，**目的是让 E46/E47 互相独立**，不是「E47 内部用两条路径算同一个量再比对」。
全文另一处「独立算术给出的绝对值断言」（行 52）是通用方法论要求（每条互比断言旁要有绝对值断言），
不是已执行的互证结构。

### E56 消息缓冲的收益 vs ε（`.claude/kb/experiments/56-消息缓冲的收益vsε.md`）

grep 命中 1 处，行 171：
```
171:#### 树长大 8 倍，缓存预算不变（这是两条独立论证腿都点名的那个外推威胁）
```
这句是三方论证的推理腿语言（判「否」的那类用词），**不是真结构本身**。
真结构在「跑前写死的解析预测」节：闭式 `touch_per_op = H(ε) × min(1, F(ε)/B(ε))`，
被 2026-08-31 修正为 `io/op = 2(1 − e^(−x))/x`；臂 A（x→0，极限 2）、臂 B（x=512/1024=0.5，闭式 1.5739，实测 1.5737）、
臂 C（x=B/F=3ε/(1−ε)）。正文明写「这一条钉在单测 `all_three_arms_lie_on_one_flush_curve` 里，不许只活在散文里」。

源码现查：
```
$ grep -n "fn all_three_arms_lie_on_one_flush_curve\|fn arm_logstruct_wb\|fn analytic_touches_per_operation" research/e7-index-bench/src/bin/e56_epsilon.rs
145:    fn analytic_touches_per_operation(&self) -> f64 {
504:fn arm_logstruct_wb(device: &mut Device, geometry: &Geometry, operation_count: u64, seed: u64, cache_node_count: usize, write_buffer_capacity: usize, touched: &mut [bool]) -> Run {
1256:    fn all_three_arms_lie_on_one_flush_curve() {
```
`all_three_arms_lie_on_one_flush_curve`（e56_epsilon.rs:1256）内联定义闭式 `curve`，
断言 `(write_buffer_arm_cost - 1.5739).abs() < 1e-3`——**这个断言比的是闭式值对字面常量 1.5739**，
不是「闭式值等于同一次跑里 `arm_logstruct_wb`（:504）的实测输出 1.5737」；两者仍要人读 kb 正文核对。
**部分认得出**：三条路径各自的计算都在源码里，跨路径的比对断言没有把「闭式」与「实测」直接连在一起。

### E58 校验和粒度的端到端代价（`.claude/kb/experiments/58-校验和粒度的端到端代价.md`）

grep 命中 2 处，行 54、110：
```
54:⚠️ **阴性对照是 E58（校验和粒度的端到端代价） 里唯一能证明「校验路径不是回声」的东西**：直连档两条路径逐格一致，
110:⚠️ **「两条路径落到同一处」这句作废**（2026-08-31 反推腿，主 agent 复算确认）。
```
第 54 行是真结构：内核 `/proc/self/io` 的 `read_bytes`（独立观测）vs 程序自算的 `dev_bytes`（记账，等于 `ops×G`），
阴性对照（摘掉 `O_DIRECT` 后 `read_bytes` 塌到 0、`dev_bytes` 纹丝不动）验证这条校验路径不是回声。
第 110 行是**已撤回的假互证**：字节口径临界占比（0.0404%）与时间口径临界占比（QD=1 时 0.0961%）曾被认为
「两条路径落到同一处」，2026-08-31 查出它们是两种不同机制（字节侧是指针摊薄、时间侧是 AES 分块）巧合接近，
正文写明「不构成互证」——这正是本次普查要找的反面案例：措辞像互证，结构不是。

源码现查：
```
$ grep -n "fn proc_read_bytes\|dev_bytes: ops\|互不共享代码" research/e7-index-bench/src/bin/e58_csum_grain.rs
140://! 与程序自己数的 `dev_bytes` 互不共享代码，也不共享采样。
141:fn proc_read_bytes() -> Option<u64> {
224:        dev_bytes: ops * g as u64,
```
两条路径各自都在源码里（`proc_read_bytes`:141 vs `dev_bytes`:224 的记账公式），源码注释自陈「互不共享代码」；
但比值 `pr_over_devbytes` 只在 `main()` 第 363–367 行算出、381 行打印，没有 `#[test]` 断言它 ≈ 1（该路径要真设备，单测跑不了）。

### E86 扫描步进与两种单元大小（`.claude/kb/experiments/86-扫描步进与两种单元大小.md`）

grep 命中 1 处，行 9：
```
9:1. 16 KiB 步进收全率恒 100%；2. 32 KiB 步进 + 节点任意 16K 槽的漏数**恒等于**独立统计的
   奇槽节点数（两条路径不共享步进逻辑）；
```
真结构：扫描重建（内部探针按步进走）算出的漏数 vs 独立统计的奇槽节点数，判据明写「两条路径不共享步进逻辑」。

源码现查：
```
$ grep -n "fn scan\|fn census\|fn step32k_packed_misses_exactly_odd_nodes" research/e7-index-bench/src/bin/e86_scan_step.rs
152:fn scan(disk: &[Slot], step_slots: usize) -> (u64, u64, u64) {
168:fn census(disk: &[Slot]) -> (u64, u64, u64) {
233:    fn step32k_packed_misses_exactly_odd_nodes() {
```
`step32k_packed_misses_exactly_odd_nodes`（:233）直接 `assert_eq!(census_nodes - found_nodes, odd_nodes, "seed={seed}")`——
**认得出，比对断言也在源码里**，且带一条「场景里必须真的有奇槽节点，否则这格什么也没证」的判别力自证
（`assert!(odd_nodes > 0, ...)`）。

### E93 老化下的放置与碎片度（`.claude/kb/experiments/93-老化下的放置与碎片度.md`）

grep 命中 1 处，行 107：
```
107:增量 O(1) 维护的 runs 与全扫审计在**全部臂 × 负载 × 种子 × 采样点**逐点相等；
    审计的判别力单测验证（绕过增量路径直接踩坏布局，两条路径必须分叉）。
```
真结构：增量 O(1) 维护的 runs 计数 vs 全扫审计重算，逐点相等；专门有故障注入测试证明两条路径分辨得出坏布局。

源码现查：
```
$ grep -n "fn audit_runs\|fn runs_incremental\|assert_eq!(simulation.runs_incremental\|fn audit_has_teeth" research/e7-index-bench/src/bin/e93_aging_placement.rs
147:    fn audit_runs(&self) -> u64 {
157:    fn runs_incremental(&self) -> u64 {
437:            assert_eq!(simulation.runs_incremental(), simulation.audit_runs(), "t={checkpoint_index} 增量与审计分叉");
598:        assert_eq!(simulation.runs_incremental(), 1);
604:        assert_eq!(simulation.runs_incremental(), 3);
622:        assert_eq!(simulation.runs_incremental(), 3);
684:    fn audit_has_teeth() {
```
**认得出，全链路**：比对断言在 :437，故障注入测试 `audit_has_teeth`（:684）`assert_ne!(simulation.runs_incremental(), simulation.audit_runs())`
（人为踩坏布局后两条路径必须分叉）。

### E110 条带表在连续发布下的期望写放大（`.claude/kb/experiments/110-条带表在连续发布下的期望写放大.md`）

grep 命中 1 处，行 49：
```
49:**对照与作废条款**：阴性对照（0 轮、0 叶两条路径都测）pass；
```
这里「两条路径」指甲（条带表）/丙（全镜像）**两条设计臂**在 0 轮 0 叶的边界取值一致，
是设计臂本身的对照，不是同一个量的两条计算路径。**真结构在别处**：
「单次发布的算术与 E107 逐字节相同」——E110 自己独立写的发布函数复现 E107 已发表的三个端点，
正文写「作废条款 2 就是拿它已钉的两个端点（627 200 / 692 736 / 659 968）当交叉校验，对不上整轮作废」。

源码现查：
```
$ grep -n "fn publish_on_stripe_table_arm\|fn publish_on_full_mirror_arm\|fn single_publish_matches_e107_pinned_values" research/e7-index-bench/src/bin/e110_stripe_table_steady.rs
58:fn publish_on_stripe_table_arm(leaves: u64, devices: u64, occupied_container_slots: u64) -> (u64, u64, u64) {
76:fn publish_on_full_mirror_arm(leaves: u64) -> u64 {
186:    fn single_publish_matches_e107_pinned_values() {
```
`single_publish_matches_e107_pinned_values`（:186）三次 `assert_eq!` 直接比对 E110 自己的函数输出与 E107 的已钉值。
**认得出，全链路**，且这条跨实验交叉校验比 E45 的「校验 B」更扎实——它是真的另一份独立代码复现，
不是从正文抄字面量。

### E113 不认识的树·补齐臂集（`.claude/kb/experiments/113-不认识的树-补齐臂集.md`）

grep 命中 1 处，行 12：
```
12:| 臂集漏了判定表里明写的那一档 | D15（格式冻结政策）「整理 / 写可能误删看不懂的记录 ⇒ compat_ro」，
    并明写「**「只读安全」与「可写安全」是两条独立保证，必须分开判**」|
```
**判「否」**：「两条独立保证」指只读安全与可写安全是**两类不同的安全属性**，要求分开判定，
不是「同一个量用两条路径算一遍再比对」。全文再无第二处「独立」/「路径」命中
（`grep -n "独立\|互证\|校验路径\|逐格相等\|closed.form\|解析" ...` 只有这一行）。

### E154 两道闸串-重判与回收时点的代价（`.claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md`）

grep 命中 1 处，行 154：
```
154:4. **H3a/H3b（并发请求）仍未实现**……**H4a/H4b（推空期间崩溃）已实现，但在本模型里是同一条代码路径**
    （见「这一段做了什么」第 8 条），不是两条独立验证过的路径——真要分开，需要把 `attempt_publish` 拆成
    「分配」与「写根持久」两个能单独崩溃的步骤，本段没有做到。
```
**判「否」**：H4a（崩在根持久之后）与 H4b（崩在根还没持久之前）本该是两个不同的**崩溃时点场景**，
各自单独验证；正文承认它们在本模型里退化成了同一条代码路径——这是**测试覆盖面缺口**（该分测的没分测），
不是「同一个量两条路径互证」。全文再无第二处命中。

### E155 每次持久化的写量：三种 fsync 形态与反事实上界（`.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`）

grep 命中 2 处，行 40、66，两处都是别的意思：
```
40:extent / inode 长高一层的 e 值恰好是 1.0（钉进单测），是「一层一条路径」；分配记录 / 映射长高一层的 e 值
   恰好是 2.0——长一层要多写两条路径的份额，不是一条，这是它们的上界比 extent/inode 长得快的一个直接原因。
66:……不共享用 `UnsharedPaths`——k 条互不共享路径、组间空隙 ≥ 这一层的覆盖……
```
第 40 行「两条路径」是 **B 树结构路径**（root-to-leaf 意义上的树路径），第 66 行「互不共享路径」是
**WAL 建模里的一个数据结构/参数名**（`UnsharedPaths`，表示 k 条不共享日志空间的写流）——
两处都是这份实验的领域概念，与 C48 的「校验路径」无关。**真结构在别处**：「7.3 手算格」一节，
多处单测把通用参数化模型函数的输出与手算绝对值字面量断言相等。

源码现查：
```
$ grep -n "fn whole_unit_rewrite_general_implementation_matches_the_hand_calculated_cell_at_file_count_1" research/e7-index-bench/src/bin/e155_fsync_write_volume.rs research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs
research/e7-index-bench/src/bin/e155_fsync_write_volume.rs:1753:    fn whole_unit_rewrite_general_implementation_matches_the_hand_calculated_cell_at_file_count_1() {
research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs:2674:    fn whole_unit_rewrite_general_implementation_matches_the_hand_calculated_cell_at_file_count_1() {
```
该测试体：`share = whole_unit_rewrite_counterfactual_share(shape, ...); assert!((share - 250_880.0/344_576.0).abs() < 1e-9)`——
**认得出，比对断言也在源码里**：通用模型函数的输出直接对手算字面量断言相等。
同型的还有 `q3_1_general_implementation_matches_the_hand_calculated_cell_at_file_count_1`（正文 26 行提到）、
`in_flight_records_and_ring_bytes_match_the_hand_calculation`（:1373）、
`tree_bytes_at_file_count_1_matches_the_hand_calculation`（:1881）等一批同名模式的测试，
第二次跑的 `research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs` 里也有对应的批级版本
（B12–B14，`batch_anchors_at_file_count_100_concurrent_fsync_count_2_match_the_hand_calculation` 等）。

## 两个数

**这两个数不是靠一条 grep 筛出来的**——第 1 列判据本身就没有机械形态（C48 2026-08-30 的评估原话），
每一份都是逐份读正文 + 读对应 `.rs` 源码判出来的（见上面「逐份详情」）。这里给的是**对已判定结果的现查计数**，
不是判定过程本身的自动化。

### 数 1：15 份里第 1 列判「是」的有几份

```
$ printf '%s\n' 07 12 14 45 56 58 86 93 110 155 | wc -l
10
```
10 份：E07、E12、E14、E45、E56、E58、E86、E93、E110、E155。
其余 5 份（E21、E32、E47、E113、E154）判「否」——详见上面各自小节，共同点是：
grep 命中的「独立」「两条/三条路径」说的都是别的东西（存在性证据的种类、三方论证的推理腿、
两个不同实验间的代码隔离、两类不同的安全保证、测试场景覆盖面缺口、树结构路径、建模参数名），
不是「同一个量用两条以上互不共享代码的路径各算一遍、再比对」。

### 数 2：判「是」的 10 份里，源码那一侧认得出的有几份

这里要分两档，一刀切会把「认得出但比对本身没断言」和「有一条路径根本没代码」混在一起：

**档 A——正文命名的每一条路径都能在源码里找到对应实现**（不要求跨路径比较本身被断言）：
```
$ printf '%s\n' 14 45 56 58 86 93 110 155 | wc -l
8
```
8 份：E14、E45、E56、E58、E86、E93、E110、E155。
E07、E12 **不算**——各自有一条命名路径（E07 的 `logstruct_wb` 闭式估算、E12 的「推算」首性原理公式）
全仓搜不到任何对应的函数或测试，只活在 kb 正文的散文推导里。

**档 B——档 A 之中，跨路径的比对断言本身也钉在源码里**（`assert_eq!`/`assert_ne!`/`assert!` 直接比两条路径的输出）：
```
$ printf '%s\n' 14 45 86 93 110 155 | wc -l
6
```
6 份：E14、E45、E86、E93、E110、E155。E56、E58 停在档 A——两条路径各自的计算都能定位到代码，
但源码里没有一处断言把「闭式/内核计数器」与「同一次跑的实测输出」直接连起来比较，
这一步目前只能靠人读 kb 正文核对两个数字。

## 没做什么

- 没改任何文件；没提议登记形态该长什么样（那是主 agent 的活）；没判 C48/C49 该怎么还。
- 第 1 列判据没有机械形态，全靠逐份读正文判定；换一个人读，「E56/E58/E110/E155 算不算『是』」
  这类边界情况（grep 命中处不是真结构、真结构在页面别处）可能判得不一样——这几份在上面「逐份详情」
  里把 grep 命中的原句和真结构分开写了，供复核。
- 源码侧的「认得出」只查了 `research/e7-index-bench/src/bin/` 下与该实验编号对应的那一个 `.rs` 文件
  （`e07_*`/`e12_lifecycle.rs`/`e14_discrimination.rs`/`e45_span_ring_cost.rs`/`e56_epsilon.rs`/
  `e58_csum_grain.rs`/`e86_scan_step.rs`/`e93_aging_placement.rs`/`e110_stripe_table_steady.rs`/
  `e155_fsync_write_volume.rs`+`e155_second_run_fsync_write_volume.rs`），没有扫描 `crates/` 下的生产代码——
  E14 对应的生产不变量 I-3.1 的真实实现在 `crates/singlefs-checker/src/walk.rs`，
  这份普查没有去查生产代码里是否也有类似的「两条互不共享路径」结构（不在派发范围内）。
- E45 的判「是」带一个特殊情况：它是唯一一份「结构上真的存在、但实验自己后来推翻了『独立』这个说法」的页面；
  按第 1 列判据字面（「正文里是不是真有……这件事」），这件事**发生过**（三条路径真的被写出来比对过），
  所以判「是」；但登记设计如果打算过滤掉「独立性已被证伪」的情况，E45 需要单独考虑（这一句不是提议，只是提醒范围）。
- E12、E07 各有一条命名路径在源码里完全找不到实现（「推算」「logstruct_wb 闭式」）——
  这两条路径的「怎么算」目前只能靠人读 kb 正文核对，普查没有去反推它们在源码里"应该"对应哪个函数，
  只报告了现状（认不出）。

---
交付路径：`research/prompts/c48-c49-multipath-survey.md`
