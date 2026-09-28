# E16 重跑登记（第 5 次）：今天的单元宽度下，多流模型里甲 / 乙比值的峰值落在哪、预测式还对不对、适用范围句还是不是真话

写于 2026-09-28，新装置写之前、这一次的任何产物之前。

**第几次**：第 5 次。现查依据：实验页标题行（`.claude/kb/experiments/16-journal的角色WALvs意图日志.md:1`）写「2026-08-28；08-31 补祖先计数；09-03 补字节口径」，`.claude/kb/experiments-history.md` 的 E16 节（第 967 行起）另有 2026-08-29「审计轮复跑」一条；四次与派发提示一致。仓里（含 git 历史，`git log --all --name-only -- 'research/prompts/e16*'`）没有任何 `e16-r*-prereg.md`，也没有 `e16-preregistration.md`：E16 早于跑前登记这一套，原判据只在实验页与装置 `sweep()` 的文档注释里（第二节整段抄）。

判据、门槛、作废条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**装置写在哪（写死）：新装置 `research/e7-index-bench/src/bin/e16_fifth_run_peak_under_today_widths.rs`（独立手写计数模型，不与 `crates/` 共用代码），不是入库装置。** 这条问题问的是「E16 那个多流记账模型换上今天的宽度之后判出什么」，不是「`crates/` 今天这份代码的性质」。原装置 `research/e7-index-bench/src/bin/e16_journal.rs`、它的变异表 `research/mutations/e16_journal.tsv` 与 `research/scripts/replay.sh` 第 156、157 行两条 E16 复跑行一字不动；原装置的 `sweep` 模式只在这一次被现跑一遍当阳性对照的参照（第五节 PC1），不改它的代码。另起新装置的理由：原 `sweep()` 把几何（`FANOUT = 128`、`HEIGHT = 4`、叶数 = 128⁴）、记录宽（头 48、每项 24 + 32）与两轴取值写死在代码里，改它就丢了阳性对照的参照（E155R2–R4 的先例）。新装置的变异表 `research/mutations/e16_fifth_run_peak_under_today_widths.tsv`，复跑行 `replay.sh` 新加一行 `E16R5`。

## 一、问题

英文名：fifth_run_peak_under_today_widths

**主 agent 给的问题（逐字）**：按今天的单元宽度（扇出、树高、记录宽，以 crates/singlefs-format/src/lib.rs 与 .claude/kb/layout/01-first-txn.md 为准现查），E16 多流模型里甲 / 乙比值在流数 × 批二维上的峰值落在哪、预测式还对不对得上、实验页那句适用范围句还是不是真话。只有这一种读法。

**读法写死**：

| 名字 | 写死成 |
|---|---|
| 甲、乙 | 甲 = 原装置的 `intent` 臂（fsync 即一次完整 checkpoint：脏叶 + 去重后的全部祖先 + 一条游标记录 + 根槽），乙 = 原装置的 `wal_leaf` 臂（fsync 只落还没落过的脏叶 + 点名这些叶的记录，祖先延到 checkpoint）。定义逐字见第二节（装置第 16–29 行）。两条臂的语义这一次一个字不改，只换宽度 |
| 甲 / 乙比值 | 每一格里「甲每次 fsync 当场付出的块数的平均」÷「乙每次 fsync 当场付出的块数的平均」，与原装置 `sweep()` 同一口径（第二节装置第 468–469 行）：块数口径，数据单元、索引节点、journal 记录、根槽各记 1 块，不按字节加权。checkpoint 那一路的代价不进比值 |
| 今天的扇出 | E16 模型的树是「叶就是用户数据单元、按位置寻址的一棵完全树」，今天与它同形的是一个文件的 extent 树下段（`.claude/kb/decisions/08-核心索引结构.md:386` 那一行与第 405 行，第二节整行抄）。所以最底一层节点罩 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144` 个数据单元，往上每层扇出 `EXTENT_TREE_INTERNAL_FANOUT = 147`（`crates/singlefs-format/src/lib.rs:153`、`:156`）。原装置是上下各层一律 128 |
| 今天的树高 | 按 `crates/singlefs-core/src/extent_tree.rs:97-102` 的定义：文件有 N 个数据单元时，根层级 r 是满足 144 × 147^r ≥ N 的最小 r，祖先节点层数 L = r + 1（最底一层是层级 0）。N = ⌈文件字节数 ÷ 32634⌉，32634 = `DATA_UNIT_BYTES` 32768 − `DATA_UNIT_PAYLOAD_OFFSET` 134（`lib.rs:20`、`:38`；D8 已定项 14 第 405 行，第二节整行抄）。**文件多大没有条款定**（第三节末尾的 grep），所以树高是一个旋钮，不是一个数：第五节列的几何点 G1–G5 各判一次。L 就是原装置的 `HEIGHT`（原装置叶数 = 128⁴、祖先层数 4） |
| 今天的记录宽 | 一条 journal 记录定长 4096、头 311、每个点名项 56、一条最多装 67 项（`lib.rs:191`、`:200`、`:203`、`:206`）。乙一次 fsync 点名 k 片叶，写 ⌈k ÷ 67⌉ 条记录、各记 1 块（`crates/singlefs-core/src/transaction.rs:5368-5374` 按 67 项一条切）；甲的记录只装游标、不点名，今天是 1 条只有头的记录 = 1 块。原装置是乙 ⌈(48 + 56k) ÷ 4096⌉ 块、甲 48 字节 1 块 |
| 流、流的落点 | 与原装置同一个生成器（装置第 142–149 行）：第 i 个操作归第 i mod S 条流，第 j 条流占单元号 `[j × ⌊N ÷ S⌋, (j + 1) × ⌊N ÷ S⌋)`，流内顺序推进，一个操作弄脏 1 个数据单元。原装置里 N = 128⁴ |
| 批 B | 每 B 个操作 fsync 一次（原装置 `fsync_every`）。checkpoint 间隔取 `B × ⌈2000 ÷ B⌉`、每格操作数取间隔的 10 倍：原 8 档 B 都整除 2000，这两个数落回原装置的 2000 与 20000；间隔是 B 的整数倍，区间 checkpoint 只在 fsync 之后触发，不改任何一次 fsync 的代价 |
| 流数轴 S | {2, 3, 4, 6, 7, 8, 11, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256}；含原 7 档 {2, 4, …, 128}，另补今天根节点孩子数附近的点（第七节 G1 根下 7 个孩子、G2 根下 11 个） |
| 批轴 B | {1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 14, 16, 20, 24, 28, 32, 40, 48, 50, 56, 64, 67, 68, 70, 72, 73, 80, 96, 100, 112, 128, 160, 192, 200, 224, 256, 320, 384, 448, 512, 640, 768, 1024, 1280, 1536, 2048, 3072, 4096}，再并上每个 S 的 {S − 1, S, S + 1}；含原 8 档 {1, 2, 5, 10, 20, 50, 100, 200}。B = 1 是定义域下界（每个操作都 fsync），不是扫描截断 |
| 原 56 格子集 | S ∈ {2, 4, 8, 16, 32, 64, 128} × B ∈ {1, 2, 5, 10, 20, 50, 100, 200}，与实验页那张表同一组格 |
| 相对误差 | 每格 e = (预测比值 − 模型比值) ÷ 模型比值，带符号；中位与最大都取 \|e\|。预测比值 = (B + min(S, B) × L + 2) ÷ (B + 1)，就是实验页的式子把「树高」换成今天几何的 L。**这个分母是否就是实验页那两个数（中位 9.6%、最大 20.5%）的算法，由第五节 PC2 在原装置现跑的产物上核**：复现得出就用它；复现不出、而换成以预测比值为分母复现得出，就改用那个并在报告里写明；两个都复现不出，照 e 的原定义算、第十节 F2 触发 |

**问题单（`research/prompts/e16-r5-questions.md`，表头与三行，quote-kb 机械抄）**：

**出处 `research/prompts/e16-r5-questions.md:7-11`（整段抄，未转述）**

```markdown
| # | 要回答的判断 | 候选（各自的定义） | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 按今天的宽度（扇出、树高、记录宽），甲 / 乙比值在流数 × 批的二维上峰值落在哪 | 甲：峰值仍逐行落在「批 ≈ 流数」；乙：峰值移到别的对角（写明移到哪条）；丙：不再有逐行峰值（单调） | 每个流数取值那一行峰值所在的批与流数之比 | 流数、批两轴都扫到峰值落在扫描区间内部（不在端点），并在至少一个方向相反的几何取样点上重跑判那一格 | 开着 |
| 2 | 今天宽度下的预测式与模型逐格对不对得上 | 甲：相对误差不超过实验页原来报的那一档；乙：超出（写明哪几格、差多少） | 逐格相对误差的中位与最大 | 全部格都报了，误差按格列出 | 开着 |
| 3 | 实验页那句适用范围句（「只在批大小 ≤ 并发流数时成立；批一旦超过流数，比值单调塌向 1」）今天还是不是真话 | 甲：原句成立；乙：要改写（写出按今天的数该怎么说） | 第 1、2 行的结果 | 第 1、2 行都够判 | 开着 |
```

问题单三行各自对应第六节哪几个量：第 1 行 ← Q1、Q2；第 2 行 ← Q3、Q4；第 3 行 ← Q5、Q6，并以第 1、2 行的判定为前提。没有岔路单：这是核一条适用范围句，不是为岔路建的实验。

## 二、被测条款与它引的定义

以下由 `research/scripts/quote-kb.py` 机械抄出（8 段，回读逐字节一致），依次是：被测条款整节（实验页第 58–90 行）；它的式子与比值的出处——原装置的臂定义（第 16–29 行）、`sweep()` 的文档注释与函数体（第 446–481 行）；「今天的扇出」的定义（D8 已定项 14 的 extent 树两行）；「今天的记录宽」的定义（布局 kb 第六节点名项那一行与「合计 311 字节」那一行）。

**出处 `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:58-90`（整段抄，未转述）**

```markdown
#### ⚠️ 「攒批吃不掉延后的收益」这句话的适用范围有界，扫描出来才发现

「16 条流 × 批 10」那一格恰好落在峰值前，单看它会把一条有界的结论读成普遍陈述。
补做流数 × 批大小的二维扫描（7 × 8 = 56 格）之后，形状是**非单调**的：

| 流数＼批 | 1 | 2 | 5 | 10 | 20 | 50 | 100 | 200 | 峰值位置 |
|---|---|---|---|---|---|---|---|---|---|
| 2 | 3.50 | **3.67** | 2.34 | 1.73 | 1.39 | 1.16 | 1.08 | 1.04 | 批=2 |
| 4 | 3.50 | **3.67** | 3.33 | 2.28 | 1.67 | 1.28 | 1.14 | 1.07 | 批=2 |
| 8 | 3.50 | 3.67 | **3.83** | 3.37 | 2.24 | 1.52 | 1.25 | 1.13 | 批=5 |
| 16 | 3.50 | 3.67 | 3.83 | **3.91** | 3.38 | 1.99 | 1.49 | 1.24 | 批=10 |
| 32 | 3.50 | 3.67 | 3.83 | 3.91 | **3.95** | 2.92 | 1.95 | 1.48 | 批=20 |
| 64 | 3.50 | 3.67 | 3.83 | 3.91 | 3.95 | **3.98** | 2.89 | 1.95 | 批=50 |
| 128 | 3.50 | 3.67 | 3.83 | 3.91 | 3.95 | **3.98** | 3.95 | 2.89 | 批=50 |

**峰值逐行落在「批 ≈ 流数」上。** ⇒ 正确的说法是：

> **攒批吃不掉延后祖先的收益，只在「批大小 ≤ 并发流数」时成立；
> 批一旦超过流数，比值单调塌向 1。**

**这条形状是先算后测的**，不是事后拟合：甲每批 ≈ `批 + min(流数,批)×树高 + 2`，
乙 ≈ `批 + 1` ⇒ 比值应当在批 ≈ 流数处见顶。56 格与预测的相对误差**中位 9.6%、最大 20.5%**，
且预测**系统性偏高**（它没算顶层祖先的共享）。⇒ 形状对得上，绝对值不要直接引用预测式。

⚠️ **确定性实验要扫参数，不是重跑。**
`.claude/singlefs-ai-sop/rules/test-discipline.md`「单次观测不算数」管的是**重复测量**，
而确定性记账模型没有测量噪声——**「跑两遍逐字节相同」一点也不能证明结论稳**。
它的对应风险是**只测了参数空间里的一个点**，而这只有扫参数才拦得住。

⚠️ **这不重开 [decisions.md](../decisions.md) D23（journal 的角色与格式） 的轴二。** 轴二是被轴一**结构性**排除的
（延后祖先 ⇒ 新根不可能自洽 ⇒ 发不出根），与收益多少无关。
E16（journal 的角色：WAL vs 意图日志）改的是那条决策里「③ 收益形状」这一依据的强度，不是结论。

```

**出处 `research/e7-index-bench/src/bin/e16_journal.rs:16-29`（整段抄，未转述）**

```markdown
//! ## 三条臂，不是两条 —— 这是本实验最先得到的东西
//!
//! D23 已定「**分配先于 journal**：journal 记录是**已经写到盘上的东西的发布指令**」。
//! ⇒ **「WAL 的 fsync = 追加一条记录」在本工程不可实现**：记录里点名的块必须已经在盘上。
//! 于是 WAL 这一侧裂成两条本质不同的臂，它们的差别不在快慢，在**重放要不要分配器**：
//!
//! | 臂 | fsync 时写什么 | 重放要不要分配器 | 与 D23 已定项 |
//! |---|---|---|---|
//! | `intent` 意图日志 | 脏叶 + 全部祖先 + 根槽 + 一条游标记录 | 不要 | 相容 |
//! | `wal_full` WAL / 记录点名到根下 | 脏叶 + 全部祖先 + 一条记录（**不发根**） | 不要 | 相容 |
//! | `wal_leaf` WAL / 记录只点名叶 | **只写脏叶** + 一条记录；祖先延到 checkpoint | **要**（祖先在重放时才生成，要分配） | **冲突** |
//!
//! ⚠️ **`wal_leaf` 是唯一真正买到东西的那条**，而它踩的正是 D23 用来否掉「逻辑意图日志」
//! 的那条判据。本实验的任务因此变成：**量出它买到多少**，好判断那条判据值不值这个价。
```

**出处 `research/e7-index-bench/src/bin/e16_journal.rs:446-451`（整段抄，未转述）**

```markdown
/// 扫描模式：`e16-journal sweep` —— 流数 × 批大小的二维扫描。
///
/// 它要证伪的是一条**先算后测**的预测：甲每批 ≈ `批 + min(流数,批)×树高 + 2`，
/// 乙 ≈ `批 + 1` ⇒ **比值随批增长到「批 ≈ 流数」时见顶，再往下掉**。
/// 若扫出来是这条非单调曲线，那 multistream 那条结论是结构性的；
/// 若效应只在某一档流数上冒出来，它就是建模伪影。
```

**出处 `research/e7-index-bench/src/bin/e16_journal.rs:452-481`（整段抄，未转述）**

```markdown
fn sweep() {
    // ⚠️ checkpoint 间隔必须**有界且远小于 n**：乙臂的 fsync 不清空脏集合，
    // 间隔比 n 还大的话脏集合无界增长，`copy_on_write_blocks` 每次 fsync 都扫全集 ⇒ O(n²·树高)。
    // 第一版正是如此，跑不完。间隔取最大批的 10 倍，既不让 checkpoint 主导，也把脏集合封住。
    let operation_count = 20_000usize;
    let checkpoint_interval = 2_000usize;
    let mut emitter = Emitter::new();
    let mut output = String::new();
    println!("E7RESULT name=sweep_config ops={operation_count} ckpt_interval={checkpoint_interval} height={HEIGHT} fanout={FANOUT}");
    for stream_count in [2u64, 4, 8, 16, 32, 64, 128] {
        CURRENT_STREAM_COUNT.store(stream_count, std::sync::atomic::Ordering::Relaxed);
        for batch_size in [1usize, 2, 5, 10, 20, 50, 100, 200] {
            let operations = generate_operations(operation_count, Workload::MultiStream, 0xBEEF ^ stream_count ^ (batch_size as u64) << 8);
            // ckpt 间隔取得远大于 n，使区间 checkpoint 不介入，只看 fsync 那一侧
            let intent_outcome = run(&operations, Arm::Intent, batch_size, checkpoint_interval, false);
            let write_ahead_log_leaf_outcome = run(&operations, Arm::WriteAheadLogLeaf, batch_size, checkpoint_interval, false);
            let (intent_blocks_per_fsync, write_ahead_log_leaf_blocks_per_fsync) = (intent_outcome.fsync_cost.iter().sum::<u64>() as f64 / intent_outcome.fsyncs as f64,
                            write_ahead_log_leaf_outcome.fsync_cost.iter().sum::<u64>() as f64 / write_ahead_log_leaf_outcome.fsyncs as f64);
            // 先算的预测值
            let predicted_intent_blocks = batch_size as f64 + (stream_count.min(batch_size as u64) as f64) * HEIGHT as f64 + 2.0;
            let predicted_write_ahead_log_leaf_blocks = batch_size as f64 + 1.0;
            output.push_str(&emitter.emit_raw(&format!(
                "name=sweep streams={stream_count} batch={batch_size} a_blocks={intent_blocks_per_fsync:.2} l_blocks={write_ahead_log_leaf_blocks_per_fsync:.2}                  ratio={:.3} pred_ratio={:.3}", intent_blocks_per_fsync / write_ahead_log_leaf_blocks_per_fsync, predicted_intent_blocks / predicted_write_ahead_log_leaf_blocks)));
            output.push('\n');
        }
    }
    output.push_str(&emitter.finish());
    print!("{output}");
}

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:386-386`（整段抄，未转述）**

```markdown
- **extent 树**：两段，都按位置寻址。上段按 inode 号的位置寻址；下段一个文件一棵，按数据单元号的位置寻址（叶罩 144 个单元，内部扇出 147），洞就是缺席。**只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针；条目带一个标签字节区分「没有单元 / 下段根指针 / 内联数据指针」（用户 K2）。
```

**出处 `.claude/kb/decisions/08-核心索引结构.md:405-406`（整段抄，未转述）**

```markdown
    - 下段叶罩 144 个单元，下段 key 的字节偏移 = 单元号 × 32634。上段与下段的内部条目，key 都取这个孩子按位置罩的那一段的起点（`crates/singlefs-core/src/extent_tree.rs` 的 `build_extent_internal_entry`）。
    <!-- format-const: EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144 -->
```

**出处 `.claude/kb/layout/01-first-txn.md:323-323`（整段抄，未转述）**

```markdown
| 点名项 | 56 | 12 项（t1..t12 各一项，一项含两盘的位置条目）；构成见「点名项字段表」；4096 − 311 = 3785 ⇒ 一条记录最多装 67 项 | D23（journal 的角色与格式） 已定项 17 | 已定 |
```

**出处 `.claude/kb/layout/01-first-txn.md:351-351`（整段抄，未转述）**

```markdown
合计 **311 字节**，4096 记录余 3785，装 67 个点名项。
```

## 三、实现今天的样子

2026-09-28 现查，行号是那一刻的。

| 事实 | 出处（文件:行） |
|---|---|
| 索引节点恒 16 KiB，数据单元恒 32 KiB 含头，数据单元载荷从偏移 134 起 | `crates/singlefs-format/src/lib.rs:17`（`NODE_BYTES = 16384`）、`:20`（`DATA_UNIT_BYTES = 32768`）、`:38`（`DATA_UNIT_PAYLOAD_OFFSET = 134`） |
| extent 树内部扇出 147 = (16384 − 163) ÷ 110；下段叶罩 144 个数据单元 = (16384 − 163) ÷ 112 | `lib.rs:153`（`EXTENT_TREE_INTERNAL_FANOUT = 147`）、`:156`（`EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144`） |
| 下段按数据单元号的位置寻址，层级 L 的节点罩 144 × 147^L 个单元；根层级取罩得住全部单元的最低一层 | `crates/singlefs-core/src/extent_tree.rs:7-9`（模块文档）、`:81-86`（`lower_span_in_data_units`）、`:97-102`（`lower_root_level_for`：`lower_span_in_data_units(level) >= data_units` 的最小 level）、`:196-203`（`child_holding_data_unit`：孩子序号 = 单元号 ÷ 下一层跨度） |
| journal 记录定长 4096、头 311、点名项 56、一条 67 项 | `lib.rs:191`、`:200`、`:203`、`:206`；`lib.rs:491` 的单测把 67 与 (4096 − 311) ÷ 56 比 |
| 一次发布点名多于 67 项时按 67 项一条切成几条记录（末条再跨记录，D23 已定项 17） | `crates/singlefs-core/src/transaction.rs:5338-5378`（`roles_named_by_each_record_of_the_publish`，第 5368–5374 行 `chunks(named_unit_capacity)`）；容量开关 `:140-165`（产品路径恒 67，测试可压到 1..=67） |
| 测试镜像默认 4 GiB，是占位值、没有条款 | `lib.rs:283-285`（`// placeholder: C323（镜像大小全仓没有条款）…`、`TEST_IMAGE_DEFAULT_BYTES = 4_294_967_296`） |

**实现与 E16 模型对不上的两处，这一次不建模（写明，不当宽度处理）**：

1. `crates/` 今天的发布点名**每一个重写的单元**：`transaction.rs:5338-5378` 把重写集里的数据单元与树节点都列成点名项（数据单元除最后一个外各占一条记录，其余按 67 项一条切）。E16 的甲只记游标、乙只点名叶，都不是这个形状。按 `crates/` 的点名方式重定义甲，是换臂，不是换宽度，不在这一次的问法里；要问，另起一题（第十三节交主 agent 认的项）。
2. 一次写数据单元，`crates/` 今天除了 extent 树下段还要动上段叶、树表、中央映射树、分配记录树、记账树（`.claude/kb/layout/01-first-txn.md` 第零节写清单）。E16 模型只有一棵树、根上面只有一个根槽。多出来的这几棵树在每次 fsync 上加的是一个与 B、S 都有关的附加量，同样是结构、不是宽度，这一次不建模。

**文件多大没有条款**：

```
grep -rn '最大文件\|文件大小上限\|单文件上限\|单个文件.*TiB\|文件.*多大' .claude/kb/decisions/ .claude/kb/layout/ .claude/kb/milestone/ | wc -l
```

输出 `1`，唯一命中是 `.claude/kb/decisions/14-双轨大小文件-持久临时.md:3`（那一段说的是小对象写路径按一个数据单元补齐，不是文件大小的上限）。镜像大小同样没有条款（C323（镜像大小全仓没有条款））。所以树高照第一节做成旋钮，几何点见第五节。

## 四、跑之前已经存在的数

照实列。凡是在下面出现过的数，判据都没有拿它调门槛；各条写了它对判据的影响。

| 数 | 在哪读到 / 怎么来的 | 对判据的影响 |
|---|---|---|
| 实验页那张 7 × 8 表的 56 个比值与「峰值位置」一列（批 = 2、2、5、10、20、50、50）；「中位 9.6%、最大 20.5%」；「预测系统性偏高」；预测式本身 | 被测条款正文（第二节第一段抄录），开工必读 | 这是被测对象，不是参照答案。56 个比值与 9.6% / 20.5% 当「出自条款本身的锚点」（第七节 7.1），复现不出走 F1 / F2。第 2 行「那一档」的门槛就取这两个数，由问题单的候选定义给定，不是我挑的 |
| 实验页标题行「2026-08-28；08-31 补祖先计数；09-03 补字节口径」与 `experiments-history.md` E16 节的五个日期标题、其中一行「14 个确定性实验，12 个逐字节相同」 | 为核「第几次」读了标题行 | 只用来定「第 5 次」，与判据无关 |
| I6 那一行：树高 h = 4、扇出 135、2⁴⁰ 数据 ⇒ 叶 33 554 432、135³ < 叶数 < 135⁴、流数 16 | `research/prompts/m2-s1-r2-main-verification.md:22`（派发提示点名的欠账来历） | 这一次不用 135，也不用 2²⁵：扇出按第三节现查取 144 / 147，单元数按 32634 字节载荷算（第七节 G2 是 33 692 212 个单元，不是 33 554 432）。2⁴⁰ 字节这个文件大小被我取作几何点 G2 之一，理由是 `.claude/kb/decisions/28-挂载期承诺量.md:107` 也用 1 TiB 当取样容量；135 那一档不另设几何点 |
| 我用独立脚本算出的锚点：五个几何点的单元数、根层级、根下孩子数；每格 S = 2 时 B = 1、B = 2 两格的甲、乙与比值；乙在 B = 67、68、70、72、73 上今天与旧记录宽的块数 | 第十三节命令一（`anchors_e16_r5.py`），输出抄在第七节 7.2 | 这些是钉绝对值的锚点。它们透露了 S = 2 那一行前两格的形状（G1 两格都是 3.000，G4 两格都是 4.000，G5 从 2.500 降到 2.333），**是第 1 行在 S = 2 那一行上的部分答案**，照实列在这里。判据的门槛（第六节）不依赖这几格：分类规则对每一行一样，写规则时没有按这几格去挪带宽或容差 |
| 设计时从实验页的预测式手推的形状（推的，没量过）：预测式在 B ≤ S 段给出比值 (B × L + 3) ÷ (B + 1)，L > 3 时随 B 升、L = 3 时恒为 3、L < 3 时随 B 降 | 我判问法时自己推的 | 这是预测式的性质，不是模型的结果；它正是第 2 行要核的那个式子。它让我把「平台」与「B_lo = 1」两类写进第 1 行的分类（第六节 Q1），并选了 L 两侧的几何点（第八节）。结果反过来（模型在 L = 3 上仍逐行在 B ≈ S 见顶）我照判甲，不改分类 |
| 原装置文档注释里的历史数：`wal_full` 环峰值虚报的 192048 字节（47 块）、第一版 `root=0 ckpts=0`、`FANOUT` 注释里的 291 | 读原装置 `research/e7-index-bench/src/bin/e16_journal.rs:53`、`:191`、`:380` | 是主网格的建模史，不进这一次任何一格 |
| 布局 kb 第三·二节读到的 `map_height=4 … map_bytes=66048`、扇出 267 → 239、映射叶扇出 296 | `.claude/kb/layout/01-first-txn.md:237` | 296 只被我取作第八节扇出旋钮的一个取样点（「更大的扇出」那一侧），不用于判定 |
| grep 顺带命中的 D28 第 107 行（两块 4 GiB 盘 529 个节点、2.7%、3.2%）、C538、C567、D26 第 108 行的数 | 第十三节 grep 命中行 | 与本实验无关，不进判据 |
| 设计时手推的另一件事（推的，没量过）：批很大时乙每次 fsync 的记录块数 ⌈B ÷ 67⌉ 随 B 线性涨，而甲多出来的祖先里最底一层大约是 B ÷ 144 片再加每条流几层，所以比值在 B 很大时可能跌到 1 以下，不只是「塌向 1」 | 我写阳性对照时自己推的 | 因此阳性对照不写成「每一格比值都 > 1」（那条会在大 B 上误判装置坏了），只钉 B ≤ 67 的格（第五节 PC4）；另加一个量报每行比值的最小值与它跌没跌破 1（第六节 Q6 第二行），因为第 3 行问的原句写的是「塌向 1」 |

## 五、臂、阳性对照、真实基线

### 5.1 臂

| 臂 | 怎么做（每次 fsync 当场写什么） | 做完会怎样（由前一句推得出） |
|---|---|---|
| 甲（`intent`） | 自上次 fsync 以来的脏数据单元各 1 块；这些单元在今天几何下的全部祖先节点去重后各 1 块（层级 0 到根层级 r，共 L = r + 1 层，同一个节点被几片脏单元共享只算一次）；1 条只装游标的记录 1 块；根槽 1 块。写完清空脏集合 | 一次 fsync 的块数 = B + 去重祖先数 + 2。B 个脏单元分属 min(S, B) 条流时，各流在根下哪一层汇合决定了去重祖先数：各流在根的孩子那一层就分开时每条流各付 r 层、根只付一次；根下孩子少于流数时，几条流共用上面几层 |
| 乙（`wal_leaf`） | 自上次 fsync 以来还没落过的脏数据单元各 1 块；点名这 k 片单元的记录 ⌈k ÷ 67⌉ 条，各 1 块。祖先一概不写，延到 checkpoint | 一次 fsync 的块数 = k + ⌈k ÷ 67⌉；多流顺序追加时每次 fsync 的 k = B（同一个区间里一个单元不会被写第二次，第七节的 B = 1、2 锚点核这一点） |

两条臂的语义与原装置逐字相同（第二节装置第 16–29 行、`run()` 第 318–397 行的 fsync 那一段），只把几何与记录宽换成第一节写死的今天的值。对面那条臂按支持它的人认的样子取：乙的记录用今天的格式（点名项 56 字节自带位置条目的 CRC，不用原装置那 32 字节的额外校验和；一条 67 项，不是按 4096 字节整除的 72 项），这是今天格式能给乙的最省的点名方式；甲的记录只有头、不点名，今天格式里 1 条 = 1 块，与原装置同。甲的祖先去重、根只付一次，照原装置。

### 5.2 几何点

| 点 | 扇出（最底层 / 往上各层） | 单元数 N | 根层级 r / 祖先层数 L / 根下孩子数 | 记录宽 | 在哪一段 | 为什么取它 |
|---|---|---|---|---|---|---|
| G0 | 128 / 128 | 128⁴ = 268 435 456 | 3 / 4 / 128 | 原装置：乙 ⌈(48 + 56k) ÷ 4096⌉ 块，甲 1 块 | 第一段 | 原装置的几何，只用来做 PC3（新装置在这里要逐格复现原装置） |
| G1 | 144 / 147 | ⌈4 GiB ÷ 32634⌉ = 131 611 | 2 / 3 / 7 | 今天 | 第一段 | 树高比 G2 低一层的方向相反的取样点；4 GiB 是 `crates/` 里唯一的容量字面量（测试镜像，占位值）。一个文件填不满 4 GiB 的镜像，这里只把它当 L = 3 的取样点，不当现实负载 |
| G2 | 144 / 147 | ⌈2⁴⁰ ÷ 32634⌉ = 33 692 212 | 3 / 4 / 11 | 今天 | 第一段 | 真实基线：L = 4 与原装置同高，但根下只有 11 个孩子；1 TiB 取自 D28 第 107 行与 I6 用的容量 |
| G4 | 144 / 147 | ⌈2⁴⁴ ÷ 32634⌉ = 539 075 383 | 4 / 5 / 2 | 今天 | 第一段 | 树高比 G2 高一层的方向相反的取样点 |
| G3 | 144 / 147 | 144 × 147³ = 457 419 312 | 3 / 4 / 147 | 今天 | 第二段 | 与 G2 同高、根下孩子数满的点，把「根下孩子数」与「树高」两件事分开 |
| G5 | 144 / 147 | ⌈256 MiB ÷ 32634⌉ = 8 226 | 1 / 2 / 58 | 今天 | 第二段 | L = 2，比 G1 再低一层 |
| F128、F296 | 128 / 128；296 / 296 | ⌈2⁴⁰ ÷ 32634⌉ | 装置照第一节的定义算，产物配置行报 | 今天 | 第二段 | 扇出旋钮方向相反的两点（比 144 / 147 小、比它大）；296 取自布局 kb 第 237 行的映射叶扇出，只当取样值 |
| R1、R∞ | 144 / 147 | 同 G2 | 同 G2 | 乙一条记录装 1 项；装 10⁶ 项 | 第二段 | 记录宽旋钮方向相反的两点（1 是 `transaction.rs:148-149` 测试开关的下界） |

N、r、L、根下孩子数在 G0–G5 上都已由独立脚本算过（第七节 7.2），装置的配置行必须逐个对上（停机 S3）。

### 5.3 阳性对照（每一条臂都跑）

| 编号 | 做什么 | 对哪条臂 | 不过怎么办 |
|---|---|---|---|
| PC1 | 现跑原装置 `e16-journal sweep`（不改代码），56 格的 `a_blocks`、`l_blocks`、`ratio`、`pred_ratio` 落进 `research/results/`；56 个 `ratio` 按两位小数与实验页那张表逐格比 | 甲、乙都比（`a_blocks` 是甲、`l_blocks` 是乙） | 对不上走 F1（条款的表可能错），不作废：之后第 2 行的比对同时用页上的数与现跑的数各判一次 |
| PC2 | 在 PC1 的 56 格上按第一节的相对误差定义算中位与最大 \|e\|，四舍五入到 0.1% 与「9.6%、20.5%」比；复现不出就换以预测比值为分母再算一次 | 比值（两臂合成的量） | 两种都复现不出走 F2 |
| PC3 | 新装置在 G0 上跑原 56 格，`a_blocks`、`l_blocks`、`ratio`、`pred_ratio` 与 PC1 的产物逐字段相同 | 甲、乙逐格各比 | 不同就停机 S2（新装置与原装置两边都查） |
| PC4 | 延后祖先的收益本身测不测得出：每个几何点上 B = 1 的每一格甲 − 乙 = L + 1 恰好成立；B ≤ 67 的每一格比值 > 1 | 甲、乙各一个绝对值（甲 = L + 3、乙 = 2） | 不成立整轮作废 V4 |

### 5.4 真实基线

G2（今天的扇出、今天的记录宽、1 TiB 文件）。G1、G4 是它两侧的树高取样点。文件多大没有条款定（第三节），所以真实基线不是唯一的：第 1、3 行的判定按几何点分别报，G1、G2、G4 判定不同时，第 3 行的改写要带上树高或根下孩子数的条件（第六节 Q5、Q6）。

### 5.5 实现量与分段

历史族、崩溃支线、坏镜像都没有：这是纯计数模型。量 = 几何点 × 流数 17 档 × 批约 60 档 × 两臂。要另写的只有三样：按层给扇出的祖先去重、峰值与单调性的分类、相对误差的汇总。一个执行员一次做得完第一段。

- **第一段**（补问题单第 1、2、3 行，补完三行都能判）：停机 S1 的常量核对；PC1–PC4；第七节全部锚点写成单测；G1、G2、G4 三个几何点的全网格；原 56 格子集上的误差；变异表整表。第一段交回时对着问题单三行逐行写「够判 / 还差什么」。
- **第二段**（只在第一段交回之后主 agent 判第 1 或第 3 行还差时才跑）：G3、G5、F128、F296、R1、R∞。它补的是第 1 行的「几何敏感性」多几个点、第 3 行改写里「条件」那半句的边界（根下孩子数与树高哪一个在起作用）。第一段里 G1、G2、G4 已经给了树高两侧方向相反的点，第 1 行的够判条件在第一段就满足，第二段不是欠账；不跑就在实验页记「够判后未跑」。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，不合取；「几行合起来对应问题单哪个候选」写在最后一列，合取由主 agent 做。计时类的量没有（纯计数模型，不计时）。容差与带宽都是在读任何产物之前定的：峰值集合容差 0.1%、单调容差 1%、「批 ≈ 流数」的带宽 [S ÷ 2, 2S]、「塌向 1」取 B ≥ 16S 处的超额不超过峰值超额的四分之一。它们不能从候选的定义推出——候选甲只说「≈」，没说几倍。

**名词**：一「行」= 一个几何点上一个流数 S 的全部批。行内比值 ρ(B)。峰值集合 P = {B：ρ(B) ≥ max ρ × (1 − 0.001)}，B_lo = min P，B_hi = max P。B_max = 批轴最大值（4096）。

| 量 | 怎么算 | 门槛与判定 | 对应问题单哪一行、取什么值让它翻面 |
|---|---|---|---|
| Q1 每行的峰值落点分类 | 每个几何点、每个 S 算 P、B_lo、B_hi，按下面的次序归类：B_hi = B_max 记「端点」（批轴往上加 6144、8192、12288、16384 重跑这一行，仍是端点就记「≤ 端点，没量到」）；B_lo = 1 记「丙行」；P ⊆ [S ÷ 2, 2S] 记「甲行」；P 与 [S ÷ 2, 2S] 不相交记「乙行」并报 B_lo ÷ S；其余记「宽平台」并报 [B_lo, B_hi]。B_lo 或 B_hi 换成它在批轴上的相邻取值会改变归类的，在两个相邻取值之间逐个整数加密重判，报「加密过」 | 每个几何点给一个判定：全部行同一类就是那一类；乙行全体再报 B_lo ÷ S 的中位与范围、以及 B_lo 与根下孩子数的关系（逐行列出）；混着几类就记「分段」，列出每一类覆盖的 S 区间 | 第 1 行。G2 上每一行都是甲行 ⇒ 甲；G2 上有任何一行不是甲行 ⇒ 按那几行的类别记乙、丙或分段，写明移到哪。G1、G4 的判定与 G2 不同 ⇒ 第 1 行记「随树高变」，两边的数都写（第八节） |
| Q2 二维最大值的位置 | 每个几何点在整张 S × B 网格上取比值最大的格，连同比值不低于最大值 × (1 − 0.001) 的全部格，报它们的 S 范围与 B 范围 | 这组格里有 S < 256 的格、且没有 B = B_max 的格 ⇒ 两轴的最大值都落在扫描区间里面（沿 S 方向是平台也算）；否则流数轴往上加 384、512、768、1024 重跑（加出来的行批轴同时加长到不小于 16S，Q1、Q5、Q6 对这些行照判），仍在端点就记「≤ 端点」 | 第 1 行的够判条件「两轴都扫到峰值落在扫描区间内部」。落在端点 ⇒ 第 1 行不够判 |
| Q3 原 56 格子集上预测式的误差 | G1、G2、G4 各自在原 56 格上算每格 e（第一节定义，分母以 PC2 核定的为准），报中位 \|e\| 与最大 \|e\| 两行 | 第一行：中位 \|e\| ≤ 9.6% 记「不超」，否则「超」；第二行：最大 \|e\| ≤ 20.5% 记「不超」，否则「超」。PC1 或 PC2 触发 F1 / F2 时，页上的数与现跑原装置算出的数各当一次门槛、各判一次 | 第 2 行。G2 上两行都「不超」对应甲；任一行「超」对应乙，乙要的「哪几格、差多少」由 Q4 给 |
| Q4 逐格误差清单 | 第一段三个几何点的全部格（不只 56 格）逐格列 S、B、模型比值、预测比值、e；另报每个几何点 e > 0 的格占几成 | 不设门槛，是 Q3 判「超」时交代「哪几格、差多少」的依据；\|e\| > 20.5% 的格单独列表 | 第 2 行的够判条件「全部格都报了，误差按格列出」 |
| Q5 原句前半「批 ≤ 流数时攒批吃不掉延后祖先的收益」 | 每行在 B ≤ S 的取值上，相邻两点之间 ρ 不下降超过 1%（ρ(后) ≥ ρ(前) × 0.99） | 每行「成立 / 不成立」，报不成立的行与跌得最多的那一步；每个几何点报不成立的行数 | 第 3 行。G2 上有任何一行不成立 ⇒ 原句前半要改（乙） |
| Q6 原句后半「批一旦超过流数，比值单调塌向 1」 | 每行三件事各占一行：① B ≥ S 的取值上相邻两点之间 ρ 不上升超过 1%；② 取批轴上不小于 16S 的最小 B，ρ(B) − 1 ≤ (max ρ − 1) ÷ 4；③ 这一行 ρ 的最小值与它在哪个 B，是否 < 1 | ①②各报「成立 / 不成立」；③报数，并记「跌破 1 / 没跌破」 | 第 3 行。G2 上①或②有一行不成立 ⇒ 原句后半要改；③在 G2 上任何一行跌破 1 ⇒ 「塌向 1」要改写成「在 B = … 处跌破 1」，这同样对应乙 |

**第 3 行怎么由这几个量合成（写给主 agent，判不由执行员做）**：G2 上 Q1 为甲、Q5 全行成立、Q6 ①② 全行成立且 ③ 没有跌破 1 ⇒ 原句成立（甲）；任何一项不是 ⇒ 乙，改写照这个模板填数：「攒批吃不掉延后祖先的收益，在 [Q1、Q5 给的条件] 时成立；批超过 [Q1 给的峰值落点] 之后，比值 [Q6 给的形状]」。G1、G4 与 G2 判得不同，改写里要带上树高（L）或根下孩子数的条件，两者哪一个在起作用由第二段的 G3 分开；第二段没跑就写「两者没分开」。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符走第十节 F1 / F2「条款可能错」，不作废）

| 断言 | 出处 | 在哪核 |
|---|---|---|
| G0 上 56 格的比值逐格等于实验页那张表（两位小数）：例如 S = 2 行 3.50、3.67、2.34、1.73、1.39、1.16、1.08、1.04；S = 128 行 3.50、3.67、3.83、3.91、3.95、3.98、3.95、2.89 | 第二节抄录的表 | PC1（原装置现跑）、PC3（新装置在 G0） |
| 56 格相对误差中位 9.6%、最大 20.5% | 第二节抄录的正文 | PC2 |
| 预测比值 = (B + min(S, B) × L + 2) ÷ (B + 1)；G0 上 S = 2、B = 1 为 3.500，S = 16、B = 10 为 (10 + 40 + 2) ÷ 11 = 4.727 | 第二节抄录的式子，L = 4 | 新装置单测；PC1 的 `pred_ratio` 列 |

### 7.2 独立算出、用命令核过的（第十三节命令一；不符先核我这边的算术与 `crates/` 的定义，核完仍不符 ⇒ 停机 S3）

命令一的原样输出（`anchors_e16_r5.py`，sha256 见第十三节）：

```
payload=32634 per_record_today=67 per_record_old_model=72
G5_256MiB units=8226 root_level=1 node_levels=2 root_children=58 S2B1_a=5 S2B1_l=2 S2B1_ratio=2.500 S2B2_a=7.00 S2B2_l=3 S2B2_ratio=2.333
G1_4GiB units=131611 root_level=2 node_levels=3 root_children=7 S2B1_a=6 S2B1_l=2 S2B1_ratio=3.000 S2B2_a=9.00 S2B2_l=3 S2B2_ratio=3.000
G2_1TiB units=33692212 root_level=3 node_levels=4 root_children=11 S2B1_a=7 S2B1_l=2 S2B1_ratio=3.500 S2B2_a=11.00 S2B2_l=3 S2B2_ratio=3.667
G3_full_root_L4 units=457419312 root_level=3 node_levels=4 root_children=147 S2B1_a=7 S2B1_l=2 S2B1_ratio=3.500 S2B2_a=11.00 S2B2_l=3 S2B2_ratio=3.667
G4_16TiB units=539075383 root_level=4 node_levels=5 root_children=2 S2B1_a=8 S2B1_l=2 S2B1_ratio=4.000 S2B2_a=12.00 S2B2_l=3 S2B2_ratio=4.000
l_blocks batch=67 today=68 old_model=68
l_blocks batch=68 today=70 old_model=69
l_blocks batch=70 today=72 old_model=71
l_blocks batch=72 today=74 old_model=73
l_blocks batch=73 today=75 old_model=75
```

逐条的断言：

| 断言 | 值 |
|---|---|
| 数据单元载荷 | 32634 = 32768 − 134 |
| 今天一条记录装的点名项 / 原装置记录宽下一块装的项 | 67 / 72 |
| G5、G1、G2、G3、G4 的单元数 | 8226、131611、33692212、457419312、539075383 |
| 同上的根层级 / 祖先层数 L / 根下孩子数 | 1 / 2 / 58；2 / 3 / 7；3 / 4 / 11；3 / 4 / 147；4 / 5 / 2 |
| 每个几何点 S = 2、B = 1：甲 = L + 3，乙 = 2 | G5 5 / 2，G1 6 / 2，G2 7 / 2，G3 7 / 2，G4 8 / 2 |
| 每个几何点 S = 2、B = 2（10000 次 fsync 的平均，蛮力逐次数）：甲、乙 | G5 7.00 / 3，G1 9.00 / 3，G2 11.00 / 3，G3 11.00 / 3，G4 12.00 / 3 |
| 乙在 B = 67、68、70、72、73 的块数（今天 / 原装置记录宽） | 68 / 68、70 / 69、72 / 71、74 / 73、75 / 75 |
| G2 上 S = 3 时第 1 条流的首个单元号 ⌊N ÷ 3⌋ | 11230737（33692212 ÷ 3 = 11230737.33） |
| 单元 143、144 的最底层节点序号 | 0、1（143 ÷ 144、144 ÷ 144 取整） |
| checkpoint 间隔与每格操作数：B = 3 时 2001 与 20010；B = 200 时 2000 与 20000；B = 4096 时 4096 与 40960 | 第一节「批 B」那一行的定义 |
| `crates/singlefs-format/src/lib.rs` 里的字面量 | `DATA_UNIT_BYTES` 32768、`DATA_UNIT_PAYLOAD_OFFSET` 134、`EXTENT_TREE_INTERNAL_FANOUT` 147、`EXTENT_TREE_LOWER_LEAF_DATA_UNITS` 144、`JOURNAL_RECORD_BYTES` 4096、`JOURNAL_HEADER_BYTES` 311、`JOURNAL_NAMED_ENTRY_BYTES` 56、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 67、`TEST_IMAGE_DEFAULT_BYTES` 4294967296（停机 S1 的命令逐个核） |

装置里的这些常量写成本地常量，并各带一条加法形式的回比断言（`assert_eq!(PAYLOAD + 134, 32768)` 这一类，不写减法），单测把上表每一行钉成绝对值。

## 八、轨迹与几何敏感性

### 8.1 轨迹

被谓词消费的量是每一行的 ρ(B)（Q1、Q5、Q6 都读它），它沿批轴的「轨迹」每行报三样：**峰值**（max ρ、P、B_lo、B_hi）；**上升与下降的步数**（B < B_lo 段相邻两点 ρ 上升超过 1% 的步数，B > B_hi 段下降超过 1% 的步数，以及两段里方向相反的步数）；**期末值**（ρ(B_max)，加密或加长过批轴的报最后一个取值）。

每一格的比值是许多次 fsync 的平均，所以格内还有一条按 fsync 次序的轨迹：每格另报甲、乙每次 fsync 块数的最小、最大、平均（多流顺序追加时一批跨不跨最底层节点的边界会让甲逐次不同）。实验页的表只有平均；这一次三样都进产物行，正文不许只凭平均说「每次都」。

### 8.2 几何敏感性

| 旋钮 | 第一段的取样点 | 方向相反的点 | 第二段加的点 |
|---|---|---|---|
| 树高（由文件大小定） | G2（L = 4） | G1（L = 3，低一层）、G4（L = 5，高一层） | G5（L = 2）；G3（L = 4、根下孩子满），把「树高」与「根下孩子数」分开 |
| 扇出 | 144 / 147 | — | F128（小）、F296（大），文件同 1 TiB |
| 记录宽（一条几项） | 67，另有 B = 67–73 这几格对照原装置的 72 | — | R1（1 项）、R∞（10⁶ 项），几何同 G2 |

每个旋钮上 Q1 的每个几何点判定各报一行；取样点之间判定翻面记「不稳定（随该旋钮变）」并写两边的数，不翻就记「两点（或几点）同判」。**判别力自证**（`.claude/rules/mutation-sampling.md` 第六类）：在分类代码的单测里，拿第一段产物里两个判定不同的相邻取样点（没有判定不同的，就拿 Q1 甲行里 B_lo ÷ S 最大与最小的两行），把带宽的上沿或下沿挪到两点的 B_lo ÷ S 之间，「两点同判」那条检查必须由绿转红；写进执行员报告，贴那条单测的名字与红绿。

## 九、变异

变异表 `research/mutations/e16_fifth_run_peak_under_today_widths.tsv`，执行员按装置写成的源码定锚点原文。每一条写明在哪个取样点上改变输出；所列取样点都是第七节已钉的单测，变异不需要跑网格就该红。

| 编号 | 改什么 | 在哪个取样点上改变输出 |
|---|---|---|
| M1 | 最底一层扇出 144 换成内部扇出 147 | 单元 143 与 144 的最底层节点序号单测：144 时分属节点 0 与 1，147 时同属节点 0；配置行的各层跨度 144、21168、3111696 也跟着变。G1 的根下孩子数在这条变异下仍是 7（⌈131611 ÷ 21609⌉），**那一点不敏感**，不拿它当取样点 |
| M2 | 内部扇出 147 换成 146 | G3 根层级单测：N = 144 × 147³ 在扇出 146 下要 r = 4，L 从 4 变 5，B = 1 的甲从 7 变 8 |
| M3 | 数据单元载荷用 32768（不扣 134） | G2 单元数单测：33692212 变 33554432 |
| M4 | 一条记录装 72 项（旧记录宽） | 乙在 B = 68 的单测：70 变 69 |
| M5 | 甲的游标记录不计块 | G2 S = 2、B = 1 单测：甲 7 变 6 |
| M6 | 甲的根槽不计块 | 同 M5 那一格：甲 7 变 6 |
| M7 | 祖先只走到根层级的下一层（漏根） | 同 M5 那一格：甲 7 变 6；G4 S = 2、B = 2 单测：12 变 11 |
| M8 | 根层级取「跨度 > N」的最小层（边界取错） | G3 根层级单测：N 恰等于 144 × 147³，r 从 3 变 4 |
| M9 | 流的落点用 ⌈N ÷ S⌉ | G2 S = 3 第 1 条流首个单元号单测：11230737 变 11230738 |
| M10 | 预测式里用 L − 1 | 预测比值单测：G0 S = 2、B = 1 从 3.500 变 3.000 |
| M11 | 相对误差的分母换成预测比值 | 合成格单测：模型 2.0、预测 2.5 ⇒ e 从 0.25 变 0.20 |
| M12 | 峰值集合容差 0.1% 换成 10% | 合成行单测：B = 1、2、3 上 ρ = 3.00、3.20、3.10，S = 2 ⇒ P 从 {2} 变 {1, 2, 3}，归类从甲行变丙行 |
| M13 | 带宽 [S ÷ 2, 2S] 换成 [S ÷ 4, 4S] | 合成行单测：S = 10、峰值只在 B = 30 ⇒ 从乙行变甲行 |
| M14 | 单调容差 1% 换成 0 | 合成行单测：B ≤ S 段有一步下跌 0.5% ⇒ Q5 从成立变不成立 |
| M15 | 乙每次 fsync 落全部脏单元（不扣已落过的） | G2 S = 2、B = 2 单测：乙从 3 变大（同一区间里脏集合越积越多） |
| M16 | checkpoint 间隔恒取 2000（不取 B 的整数倍） | 间隔单测：B = 3 时 2001 变 2000 |

变异跑完按 `.claude/rules/mutation-sampling.md`「改了一个格式常量之后，要看「无效」那一栏有没有变多」一节的七个符号分栏数，「无效」「没红」都不算进「全抓」。

## 十、失败条款

| 编号 | 条款 | 什么观测会让它触发 | 触发之后 |
|---|---|---|---|
| F1 | 被测条款那张表复现不出（条款可能错） | PC1：原装置现跑的 56 个 `ratio` 里有任何一格按两位小数与实验页的表不同 | 不作废。第 3 行交回时写明「页上的表与现跑不符」，列出不符的格；Q3 用页上的数与现跑的数各判一次 |
| F2 | 「中位 9.6%、最大 20.5%」复现不出（条款可能错） | PC2：两种分母都算不出这两个数（四舍五入到 0.1%） | 不作废。Q3 的门槛页上的数、现跑的数各当一次，各判一次；e 照第一节原定义算 |
| F3 | 峰值落在扫描端点，第 1 行不够判 | Q1 有行、或 Q2 在加长之后仍是「端点」 | 那几行、那个几何点在第 1 行记「≤ 端点，没量到」，交回写明；不许把端点写成峰值 |
| F4 | 被测句子的射程随树高变 | Q1、Q5 或 Q6 在 G1、G2、G4 之间判定不同 | 照第八节记「不稳定（随树高变）」；第 3 行的改写必须带条件，不许只按 G2 写成一句无条件的话 |
| F5 | 归类取决于容差 | 把 0.1%、1% 两个容差各减半、各加倍重跑归类，有行的类别变了 | 那几行记「容差敏感」，四种容差下的类别都列出；原容差下的判定照报，不挪容差 |
| F6 | 预测式的「形状对得上」在今天宽度下不成立 | Q1 在 G2 上不是甲，而预测式在 G2 各行给出的峰值落点（预测比值的 P）在 [S ÷ 2, 2S] 里 | 第 2 行除了误差，还要写明「式子预测的峰值落点与模型不符」，列出不符的行 |

## 十一、作废条款与停机条款

### 作废（这一次不出结论，修好重跑）

| 编号 | 条件 |
|---|---|
| V1 | 产物行数与 Emitter 收尾报的条数对不上（完整性闸红），或装置中途退出 |
| V2 | 任何一格 fsync 次数为 0、或比值是 NaN / 无穷（读不到 ≠ 读到 0，这一格不许当 0 进统计） |
| V3 | 同一格两臂的操作流指纹不同（比的是不同的工作量） |
| V4 | PC4 不成立：某个几何点 B = 1 的格上甲 − 乙 ≠ L + 1，或 B ≤ 67 的格里有比值 ≤ 1 |
| V5 | 同一份装置连跑两遍，产物不逐字节相同（确定性模型有隐藏状态） |
| V6 | 变异表没跑完（`mutate.sh` 收尾没有「已还原，基线仍全绿」、或退出码 3 / 5）：只作废「N 条全抓」那句话，不作废产物 |

### 停机（既不作废，也不当结果；两边都查，写进报告交回）

| 编号 | 条件 | 两边查什么 |
|---|---|---|
| S1 | 装置的本地常量与 `crates/singlefs-format/src/lib.rs` 的字面量有一个对不上。核对命令见第十三节命令二，输出须与那里抄的九行一致 | 装置抄错了，还是 `crates/` 在登记之后改了值；后者要主 agent 判这份登记还作不作数 |
| S2 | PC3：新装置在 G0 上与原装置现跑的产物有任何一格任何一个字段不同 | 新装置的生成器、去重、记录块数；原装置的 `sweep()`。不许默认哪一边对 |
| S3 | 装置配置行里某个几何点的单元数、根层级、L、根下孩子数与第七节 7.2 不同 | 装置、我的脚本、`crates/singlefs-core/src/extent_tree.rs:81-102` 的定义三边对 |

### 够判停机

第一段交回附一张表，按问题单三行逐行写「已够判 / 还差什么 / 剩下的量能不能让它翻面」。三行都够判就停，第二段的点逐个在实验页记「够判后未跑」。第 1 行的够判条件（两轴峰值在区间里、至少一个方向相反的几何点）第一段就能满足；第 3 行只在 F4 触发、而改写里的条件要分清「树高」与「根下孩子数」时才需要第二段的 G3，这由主 agent 判。

## 十二、修订

**改了什么**：第一节「相对误差」原判据的默认分母是模型比值，PC2（第五节）在原装置现跑的产物（`research/results/e16-journal-2026-09-28.out`）上核实：以模型比值为分母算出的中位/最大误差是 10.63%/25.79%，复现不出「9.6%、20.5%」；以**预测比值为分母**算出的是 9.61%/20.50%，四舍五入到 0.1% 与登记的数一致。按登记第一节原文「复现得出就用它」的指示，Q3/Q4 的判据改为以预测比值为分母的那一路为准；模型比值分母那一路**没有删**，两种分母的数都跑了、都报进产物（`error_model_denominator`、`error_predicted_denominator` 两个字段），是加一条更精确的判据、不是丢弃原判据（只许收严）。

**依据是哪个单测读数**：不是单测，是 PC2 这一步现跑的阳性对照——`/tmp/claude-1000/e16-r5-run/pc2.py` 读 `research/results/e16-journal-2026-09-28.out`（PC1 产物）算出的两组数字（见上一段）。

**时点在产物之前**：这一条修订是在跑第一段全网格产物（`research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out`）之前作出的——PC1、PC2 本身就是登记第五节写死要先跑的阳性对照步骤，先于第一段网格。

## 十三、读过的文件与跑过的命令

行号是 2026-09-28 读的那一刻的。grep 命中行也列。实验页的结果节（第 5–57 行、第 91–349 行）没读，只读了标题行与第 58–90 行被测条款、第 350–352 行「历史版本」指路；`research/results/` 一份没读。

### 13.1 规则、共用约束与脚本

| 文件 | 行 |
|---|---|
| `.claude/agent-common.md` | 全文 |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 1–121（全文） |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 1–176（全文） |
| `.claude/rules/three-way-inference.md` | 1–221（全文） |
| `.claude/rules/mutation-sampling.md` | 1–96（全文） |
| `.claude/rules/implementation-first.md` | 1–20（全文） |
| `.claude/singlefs-ai-sop/rules/command-safety.md` | 1–159（全文） |
| `.claude/singlefs-ai-sop/rules/writing-discipline.md` | 1–131（全文） |
| `.claude/singlefs-ai-sop/rules/verify-before-claiming.md` | 1–53（全文） |
| `research/scripts/claim-experiment.sh` | 1–60 |
| `research/scripts/quote-kb.py` | 1–50 |
| `research/scripts/replace-once.py` | 1–30 |

### 13.2 派发给的输入、被测条款、格式参照

| 文件 | 行 |
|---|---|
| `research/prompts/e16-r5-questions.md` | 1–11（全文） |
| `research/prompts/m2-s1-r2-main-verification.md` | 22 |
| `.claude/kb/milestone/02-second-txn.md` | grep 命中 284、285、374 |
| `.claude/kb/experiments/16-journal的角色WALvs意图日志.md` | 标题行（`grep -n '^#'`：1、5、17、33、58、91、107、154、168、178、207、223、247、286、327、332、350）；58–90；350–352 |
| `.claude/kb/experiments-history.md` | grep 命中 183、185、465、467、498、919、967、973、1004、1006、1007、1012、1018、1020、1032、1039、1041、1074、1076、1106；967–1130 之间的标题行；1146、1250 两个节标题 |
| `research/prompts/e43-r1-prereg.md` | 1–30 与标题行（只看格式） |
| `research/prompts/e155-r5-prereg.md` | 1–25（只看格式） |
| `research/mutations/e16_journal.tsv` | 全文（打印出的各行） |
| `research/scripts/replay.sh` | grep 命中 156、157、191、211–214、499、502、503、677 |
| `research/e7-index-bench/Cargo.toml` | 1–30 |

### 13.3 `crates/`

| 文件 | 行 |
|---|---|
| `crates/singlefs-format/src/lib.rs` | 1–20；138–165；186–215；280–290；`grep -n 'pub const\|const '` 命中 9–285 各行；grep 命中 385、404–408、485、490–491 |
| `crates/singlefs-core/src/extent_tree.rs` | 1–12；78–106；165–206；`grep -n 'fn \|EXTENT_TREE_…'` 命中各行（4–1176） |
| `crates/singlefs-core/src/transaction.rs` | 140–170；147–150；1760–1800；5330–5400；grep 命中 149、155、161、162、171、174、195、211、251、257、260、265、266、671、923、1647、1776、1778、1780、5110、5338、5340、5368 |
| `crates/singlefs-checker/src/lib.rs`、`image.rs`，`crates/singlefs-core/src/recovery.rs`、`make_filesystem.rs`、`journal.rs`、`address.rs`、`system_configuration.rs`、`allocation_record_tree.rs`、`mount.rs`、`allocator.rs` | 只有 grep 命中行（journal 常量与 journal 相关函数名），没读正文 |

### 13.4 原装置

| 文件 | 行 |
|---|---|
| `research/e7-index-bench/src/bin/e16_journal.rs` | 1–560；`grep -n 'fn \|const \|甲\|乙\|流\|stream\|sweep\|batch\|批'` 命中各行（5–971）；另核 141、142、149、318、395–400、468、469、471、472 |

### 13.5 kb 其余

| 文件 | 行 |
|---|---|
| `.claude/kb/layout/01-first-txn.md` | 标题行；grep 命中 75、237、240、281；192–282；307–354 |
| `.claude/kb/decisions/08-核心索引结构.md` | grep 命中 386、405；346–444 之间的标题行；380–415 |
| `.claude/kb/decisions/28-挂载期承诺量.md` | 107（grep 命中） |
| `.claude/kb/decisions/26-后台整理与放置回收.md` | 108（grep 命中） |
| `.claude/kb/decisions/14-双轨大小文件-持久临时.md` | 3（grep 命中） |
| `.claude/kb/checks-owed.md` | grep 命中 274、433、450、748、754 |
| `.claude/kb/decisions-history.md` | grep 命中 5764、15665、36530、39405、40978、42896、42906 |

### 13.6 跑过的命令（原样；读文件的 `sed` / `awk` / `grep -n` 已按行在上面列过，不重列）

```
git log --all --diff-filter=D --name-only --format='%h %ad' --date=short -- '*e16-*' '*e16_*'
git log --all --name-only --format='%h %ad' --date=short -- 'research/prompts/e16*'
git log --all --name-only --format='' -- 'research/prompts/*-r[0-9]*-prereg.md' | sort -u
grep -rn '2⁴⁰\|2\^40\|1 TiB' .claude/kb --exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude=experiments.md --exclude=experiments-history.md
grep -rn '最大文件\|文件大小上限\|单文件上限\|单个文件.*TiB\|文件.*多大' .claude/kb/decisions/ .claude/kb/layout/ .claude/kb/milestone/ | wc -l
python3 research/scripts/quote-kb.py /tmp/claude-1000/e16-r5-design/q1.md 'research/prompts/e16-r5-questions.md:7-11'
python3 research/scripts/quote-kb.py /tmp/claude-1000/e16-r5-design/q2.md '.claude/kb/experiments/16-journal的角色WALvs意图日志.md@#### ⚠️ 「攒批吃不掉延后的收益」这句话的适用范围有界，扫描出来才发现' 'research/e7-index-bench/src/bin/e16_journal.rs:16-29' 'research/e7-index-bench/src/bin/e16_journal.rs:446-451' 'research/e7-index-bench/src/bin/e16_journal.rs:452-481' '.claude/kb/decisions/08-核心索引结构.md:386-386' '.claude/kb/decisions/08-核心索引结构.md:405-406' '.claude/kb/layout/01-first-txn.md~^\| 点名项 \| 56' '.claude/kb/layout/01-first-txn.md~^合计 \*\*311 字节\*\*'
```

两次 quote-kb 都报「回读逐字节一致」、退出码 0。

#### 命令一：独立锚点

`nice -n 19 python3 /tmp/claude-1000/e16-r5-design/anchors_e16_r5.py`，脚本 sha256 `fe345003edb832733ac4b8855d09415ead5a732f06bdad0875728db060393c8f`，输出 sha256 `c98778d5585b98cbe167503b44a5c28ec106b70c868fb3db60fa4d0bf65c3ffd`，输出原样抄在第七节 7.2。脚本全文：

```python
#!/usr/bin/env python3
# E16 第五次跑的独立锚点。常量手抄自 crates/singlefs-format/src/lib.rs（20 / 38 / 153 / 156 / 191 / 200 / 203 / 206 / 285 行），
# 不 import 任何装置；根层级照 crates/singlefs-core/src/extent_tree.rs 81-102 行的定义手写（层级 L 罩 144 x 147^L 个单元，根取罩得住的最低一层）。
import math

DATA_UNIT_BYTES = 32768
DATA_UNIT_PAYLOAD_OFFSET = 134
LEAF_UNITS = 144          # EXTENT_TREE_LOWER_LEAF_DATA_UNITS
INTERNAL_FANOUT = 147     # EXTENT_TREE_INTERNAL_FANOUT
RECORD = 4096             # JOURNAL_RECORD_BYTES
HEADER = 311              # JOURNAL_HEADER_BYTES
ENTRY = 56                # JOURNAL_NAMED_ENTRY_BYTES
PAYLOAD = DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET

def span(level):
    return LEAF_UNITS * INTERNAL_FANOUT ** level

def root_level(units):
    level = 0
    while span(level) < units:
        level += 1
    return level

def ancestors(unit, top):
    return [(level, unit // span(level)) for level in range(top + 1)]

geometries = [
    ("G5_256MiB", math.ceil((256 << 20) / PAYLOAD)),
    ("G1_4GiB", math.ceil((4 << 30) / PAYLOAD)),
    ("G2_1TiB", math.ceil((1 << 40) / PAYLOAD)),
    ("G3_full_root_L4", span(3)),
    ("G4_16TiB", math.ceil((16 << 40) / PAYLOAD)),
]
print(f"payload={PAYLOAD} per_record_today={(RECORD - HEADER) // ENTRY} per_record_old_model={(RECORD - 48) // ENTRY}")
for name, units in geometries:
    top = root_level(units)
    levels = top + 1
    children = math.ceil(units / span(top - 1)) if top >= 1 else 1
    a1, l1 = 1 + levels + 2, 1 + 1
    # S=2, B=2 蛮力：两条流各占 floor(N/2)，每次 fsync 两个操作各落一条流；取样 2000 个操作的区间、20000 个操作
    per = units // 2
    costs = []
    for k in range(10000):
        leaves = {0 * per + k % per, 1 * per + k % per}
        anc = set()
        for leaf in leaves:
            anc.update(ancestors(leaf, top))
        costs.append(len(leaves) + len(anc) + 2)
    a2 = sum(costs) / len(costs)
    print(f"{name} units={units} root_level={top} node_levels={levels} root_children={children} "
          f"S2B1_a={a1} S2B1_l={l1} S2B1_ratio={a1 / l1:.3f} S2B2_a={a2:.2f} S2B2_l=3 S2B2_ratio={a2 / 3:.3f}")
for batch in (67, 68, 70, 72, 73):
    today = batch + math.ceil(batch / ((RECORD - HEADER) // ENTRY))
    old = batch + max(1, math.ceil((48 + ENTRY * batch) / 4096))
    print(f"l_blocks batch={batch} today={today} old_model={old}")
```

#### 命令二：停机 S1 的常量核对（执行员开工先跑一次，交回前再跑一次，输出须与下面九行逐字相同）

```
grep -nE '^pub const (DATA_UNIT_BYTES|DATA_UNIT_PAYLOAD_OFFSET|EXTENT_TREE_INTERNAL_FANOUT|EXTENT_TREE_LOWER_LEAF_DATA_UNITS|JOURNAL_RECORD_BYTES|JOURNAL_HEADER_BYTES|JOURNAL_NAMED_ENTRY_BYTES|JOURNAL_NAMED_ENTRIES_PER_RECORD|TEST_IMAGE_DEFAULT_BYTES):' crates/singlefs-format/src/lib.rs
```

2026-09-28 的输出：

```
20:pub const DATA_UNIT_BYTES: u64 = 32768;
38:pub const DATA_UNIT_PAYLOAD_OFFSET: u64 = 134;
153:pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
156:pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
191:pub const JOURNAL_RECORD_BYTES: u64 = 4096;
200:pub const JOURNAL_HEADER_BYTES: u64 = 311;
203:pub const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
206:pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 67;
285:pub const TEST_IMAGE_DEFAULT_BYTES: u64 = 4_294_967_296;
```

### 13.7 交主 agent 认的项

1. 英文名 `fifth_run_peak_under_today_widths`、新装置文件名与复跑行名 `E16R5`；另起新装置而不改原装置 `sweep()`（理由在文件头）。
2. 文件名取 `r5`（第 5 次）。
3. 文件大小没有条款，树高因此做成旋钮；G2（1 TiB）当真实基线、G1（4 GiB）与 G4（16 TiB）当两侧取样点，这三个取值由我定。
4. 不建模的两处结构差异（第三节末尾）：`crates/` 的发布点名每个重写单元、一次写还动别的几棵树。它们会改每次 fsync 的块数，但不是宽度；要不要另起一题由主 agent 定。
5. 块数口径沿用原装置（数据单元 32 KiB、节点 16 KiB、记录 4 KiB 各记 1 块）；字节口径这一次不登记。
