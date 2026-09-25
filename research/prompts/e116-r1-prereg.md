# E116 重跑登记（记录头 307 → 311）：打包容器的账在头 311 下还判不判出原来的结论

写于 2026-09-24 21:05 JST，装置改之前、这一次的任何产物之前。原判据写在装置文件头「## 跑前写死的判据与失败条款（跑完不许改）」与「## 反向接受条款（跑前写死，逐臂点名）」两节（`research/e7-index-bench/src/bin/e116_pack_settle.rs:34-65`，第二节整段抄）；E116 没有单独的跑前登记文件（`git log --all --name-only` 里没有 `e116-preregistration.md`）。

判据、门槛、作废条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**装置写在哪（写死）：`research/e7-index-bench/src/bin/e116_pack_settle.rs`（独立手写模型，不与 `crates/` 共用代码），不是入库装置。** 问的是 E116 跑前写死的判据在头 311 下判不判得出原结论，不是 `crates/` 今天这份代码的性质；`crates/` 里没有打包容器的整理与搬迁路径（第三节）。变异表 `research/mutations/e116_pack_settle.tsv`，复跑行 `research/scripts/replay.sh` 的 E116 那一行（二进制名 `e116-pack-settle`，`research/e7-index-bench/Cargo.toml:317-318`）。

文件名取 `r1`：派发提示点的；E116 在仓里（含 git 历史）没有任何 `e116-r*-prereg.md`。仓里另有「`r<n>` = 第 n 次跑」的用法（`e156-r2-prereg.md`），按那个用法这一次不是第 1 次；文件名要不要改由主 agent 定，正文不自引文件名。

## 一、问题

**主 agent 给的问题（逐字）**：记录头 `JOURNAL_HEADER_BYTES` 从 307 改成 311 之后，每个实验跑前写死的判据在 311 下还判不判出原来的结论。

**读法写死**：

| 名字 | 写死成 |
|---|---|
| 头宽 `H` | 装置里 `const JOURNAL_HEADER_BYTES`（第 84 行）。被判的取值只有 `H = 307`（今天的装置，改之前先现跑当基线）与 `H = 311`（`crates/singlefs-format/src/lib.rs:165`、D23 已定项 4） |
| `H` 进模型的唯一一处 | 第 156 行 `journal_w = n × (JOURNAL_HEADER_BYTES + MAP_ENTRY)`，只在 `journal = true` 且臂不是 `bg_ideal` 时计入搬迁写。⇒ `H` 只动 `bg_key`、`bg_fill` 两条臂 `journal=true` 那几行的 `move_write` 与回本比；稳态占用、爆炸半径、搬迁读、`bg_ideal`、`pad`、`journal=false` 那几行都不读 `H`（第三节 3.2 的 grep） |
| 「原来的结论」 | 不读实验页的结论节。读作：原判据每一格在 `H = 307` 现跑产物上判出的值 |
| 「还判不判出」 | 第六节每格在 `H = 307` 与 `H = 311` 两份产物上各判一次，相同记「不变」，不同记「变」 |
| 回本比的比较 | 反向接受条款的门槛是「回本比 ≥ 1」。一格的判定值 = `payback ≥ 1` 为真或假（`payback = inf` 记真）。回本比随 `H` 单调不减（`H` 只加在分子上），所以两次之间一格只可能从「< 1」变成「≥ 1」，不会反过来；这是模型结构推出来的，写在这里免得被当成发现 |
| 「b 取到实现可达上界」 | 原条款没写可达上界是哪一档。这一次**不替它选**：`BATCHES` 六档（1、8、64、512、4096、100000）逐档各判一行；任何一档翻面都照报，哪一档算「可达上界」由主 agent 按原实验页的读法定 |

**问题单（`research/prompts/m2-header311-rerun-questions.md`，表头与第 2 行，quote-kb 机械抄）**：

**出处 `research/prompts/m2-header311-rerun-questions.md:5-6`（整段抄，未转述）**

```markdown
| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
```

**出处 `research/prompts/m2-header311-rerun-questions.md:8-8`（整段抄，未转述）**

```markdown
| 2 | E116（打包容器的账） 的回本比与稳态占用在头 311 下结论变不变 | 不变 / 变 | 回本比或稳态占用跨过原判据的门槛 | 同上 | 开着 |
```

## 二、被测条款与它引的定义

用 `research/scripts/quote-kb.py` 抄进草稿目录、回读逐字节一致之后追加。

### 2.1 D23 已定项 4、已定项 17（被测条款）

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:139-165`（整段抄，未转述）**

```markdown
#### 已定项 4：记录头约束在单个原子单元内，头 311 字节

**定案**：**记录头完整落在一个原子单元内**（[invariants.md](../invariants.md) I-8.2（记录头不跨原子单元））。头 **311 字节**：

- **十个字段 78 字节**：magic 4 + 类型 2 + 算法类型 1 + 记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏） + 自述长度 4 + 点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32。`jsn` 的 10 字节 = 实例代号 32 位 + 计数器 48 位（已定项 9）。`tail_lsn` 不在头里：已定项 3 逐字定了「tail 的权威副本住在一个固定位置，**不内联在记录头**」，而「内联在每条记录头的 `tail_lsn`」正是被否掉的那个 XFS 形态的定义。
- **三笔已定增量 17 字节，不在那十个字段的 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节**、已定项 13 的载荷校验和 **4 字节**。
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 287、MAC 295、头末 311。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到序号 0 或一次发布之内跳号，当那条记录损坏、断链即止（用户 2026-09-24 定，不变量随 invariants 那一批另写）。
- **已定项 15 的新根段 188 字节**（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8）。
- **fsid 8 与 MAC 16，都排在新根段之后**：
  - **fsid 8**（系统配置 fsid 的低 8 字节，与单元头那 8 字节同口径，D18（块里携带什么信息） 已定项 7）。不带的话 I-1.4（块头 fsid 一致） 要给 journal 记录写一条豁免，而全环扫描就再也挡不住「上一个文件系统留在这块盘上的记录」——mkfs 整环写 0 只在 mkfs 真跑完时成立，重 mkfs 中途崩掉就不成立。带 8 字节之后不用豁免。
  - **MAC 16**（放在 fsid 之后；nonce 12 与算法类型 1 十个字段里已有）。理由与根记录那 29 字节同一条（D22（单元原子性怎么合成） 已定项 7）：journal 记录头是自证结构，没有父指针带 MAC ⇒ 不 day-1 留位，开加密那天要么整条走明文豁免（泄漏 `checkpoint_txg`、新根段里的两条指针、点名项位置），要么就是一次布局变更；D9（加密） 已定项 3 的原则逐字是「为加密留的位所有卷都留、不挪作他用」。I-6（加密不变量）不列它。
- 登记值就是 311（`JOURNAL_HEADER_BYTES`）；十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`。**说 78 时要写明指的是十个字段，连同这一句一起引。** 311 占 512 扇区的 61%，其后还余 **201 字节**（装得下 3 个点名项，每项 56 字节）；4096 上余 **3785**（67 个点名项，⌊(4096 − 311) ÷ 56⌋ = ⌊3785 / 56⌋）。
   <!-- format-const: JOURNAL_HEADER_BYTES = 311 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋ -->
   <!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **那个原子宽度不由本工程定**：D20（承重面：单元的原子性与自包含）已定自证单元依赖**运行时探测**的 `physical_block_size`，不许把 512 焊进格式。⚠️ **这个值是格式常量，改它要三处一起动**：kb 标记与正文、实验源码的 `const` 与钉死它的单测、重跑实验并更新产物；由门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/27-format-constants.sh`）判红。记录尺寸取 4 KiB 定长之后对齐浪费恒为 0（已定项 12）；变长记录的对齐代价表只对变长记录成立。

**依据**：

- E23（journal 几何）：十个字段（跑时还带 `tail_lsn`）的头装得进一个 512 单元，不对齐、多条记录挤同一个原子单元时除第一条外都判不了撕裂——头要完整落在一个原子单元内。
- E75（记录尺寸与环几何）：各个头部读法在 512 与 4096 两档 `physical_block_size` 上都装得进一个单元；定长 2 的幂尺寸下头永远不跨单元。
- fsid 8 与 MAC 16：用户弹窗定案，没有走三方论证，可推翻，原话在变更史。
- 本次发布内序号 4 字节：用户 2026-09-23 定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 的 Z6），是推的、被攻过零轮，原话在变更史。
- 用户定案 2026-09-24：头 311 落地、末条标志放在原「填充 1」那个字节、序号的读者规则（原话在变更史）。

**欠**：C94（登记的格式常量与后来的定案对不上）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:440-459`（整段抄，未转述）**

```markdown
#### 已定项 17：点名项 56 字节的字段表

**定案**：**一项一个单元（第一个事务 8 项）：位置条目 14 × 2（偏移 0）+ 单元类型标签 1（28）+ 出生树 8（29）+ 出生 txg 8（37）+ key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）+ flags 1（55）= 56；不另带载荷 CRC（位置条目各带 4 字节校验和）。重放从点名项直接凑出映射 key。**

- **记录点名的每一项自带校验和**：D4（校验和位置） 已定校验和内联进父指针，而记录点名的东西在记录持久时它的父还没发出 ⇒ 记录临时充当父，否则重放时那些单元没有任何完整性凭据；恢复重放施加前必须逐项验证点名单元（已定项 14 第四条）。
- **一次发布切成 N 个事务、N 条记录时，这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、记账树、中央映射树、树表单元）只在最后一条记录里点名**，前面几条不重复点名它们。
- **共享的提交内生块多于最后一条记录装得下（4096 记录 67 项）时，末条再跨记录**：从最后一个事务那条记录起依次多写几条，装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志；恢复按这个标志认一次发布的边界，一次发布的记录里没读到带标志的那条，整次不施加（已定项 14 第六条）。标志放在记录头十个字段里「填充 1」那个字节（改名「记录标志 1」），位 0 = 本次发布末条；每一次只有一条记录的发布（含空发布记录），那一条的位 0 也写 1——改第一个事务的字节（w1、w4、t9 三条记录的这一字节与头部校验和）。用户 2026-09-24 定。

**射程**：管点名项的字节；位置条目的构成归 D19（块指针的结构与宽度预算） 已定项 4 / 11，映射 key 的形态归 D19（块指针的结构与宽度预算） 已定项 6 / 10。⚠️ 共享的提交内生块只在最后一条点名时，崩在最后一条持久之前，那些单元没有任何记录点名——重放的回收路径要认得出这一格，层 0 的小负载要覆盖它（C491（多条记录时共享内生块在哪条点名没定））。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：记录点名的东西其父此刻还没写到盘上，记录必须临时充当父——逼出「点名的每一项自带校验和」。
- E77（发布的持久顺序）：点名项自带校验和、施加前逐项验证，是单元与记录之间那道屏障得以省掉的交换条件。
- 用户定案，原话在变更史。
- 用户定案 2026-09-23：共享的提交内生块只在最后一条记录里点名，原话在变更史。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6：维持拒绝时拒不拒取决于池的历史，按今天的字面认末条会施加半次发布，带标志位那一格 0 / 0）。

**欠**：C491（多条记录时共享内生块在哪条点名没定）；末条标志的写路径、恢复按标志认边界、checker 与层 0 流 L8 随实现：一条用例写出两条记录、断言共享单元只在最后一条里点名。

```

### 2.2 E116 的臂与原判据（装置文件头第 34–65 行）

**出处 `research/e7-index-bench/src/bin/e116_pack_settle.rs:34-65`（整段抄，未转述）**

```markdown
//! ## 四条臂（失败条款逐条点名，覆盖每一条 —— C186）
//!
//! | 臂 | 整理策略 |
//! |---|---|
//! | `pad` | 不打包（现行）|
//! | `bg_key` | 后台整理，沿中央映射 key 序走 ⇒ 映射叶顺序命中 |
//! | `bg_fill` | 后台整理，沿容器凑满序走，批量 `b` ⇒ 映射叶随机散布 |
//! | `bg_ideal` | 后台整理，元数据写记 0（**收益上界对照，不是候选**）|
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **主判据**：`bg_key` / `bg_fill` / `bg_ideal` 三条臂各自的回本比，在（对象大小 × 批量 b）各格上
//!    报绝对值与符号。**三条臂逐条点名**。
//! 2. **稳态占用**：死槽比例 `d` 从 0 扫到 1，报打包形态的占用**何时超过** `pad`。
//!    两种死亡模型都跑：均匀独立死亡、整容器一起死。
//! 3. **搬迁读**单列，不并进写账。
//! 4. **爆炸半径**：一个单元不可读，`pad` 丢 1 个，三条打包臂各丢几个。
//!
//! **阳性对照，逐臂跑**：容器容量强制为 1 ⇒ `bg_key` / `bg_fill` / `bg_ideal` 三条臂的
//! 稳态占用必须与 `pad` 逐格字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`pad` 在 512 B 与 16384 B 两档占用相同（都占一个单元）。不同 ⇒ 整轮作废。
//! **阴性对照**：N = 0 时四条臂的所有字节恰好 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **`bg_key` 的回本比 ≥ 1**（沿映射 key 序走仍然写得比省得多）⇒ 结论写
//!   「后台整理形态在最有利的策略下也不省，D27 已定项 1 该退回未定」，不许回头改口径。
//! - 若 **`bg_fill` 在 b 取到实现可达上界时回本比仍 ≥ 1** ⇒ 结论写「凑满序不可用，整理必须按映射 key 序」。
//! - 若 **`bg_ideal` 的回本比 ≥ 1**（连元数据写记 0 都回不了本）⇒ 结论写「数据侧本身不成立，提案放弃」。
//! - 若**打包形态的稳态占用在任何 `d` 上超过 `pad`** ⇒ 结论写「空间收益要等回收定案才成立」。
//! - 若**爆炸半径倍数 ≥ 占用收益倍数** ⇒ 结论写「买到的和赔上的是同一个数」。
//!
```

## 三、实现今天的样子

### 3.1 `crates/`（2026-09-24 现查）

| 处 | 文件与行 | 今天是什么样 |
|---|---|---|
| 头宽 | `crates/singlefs-format/src/lib.rs:164-165`；分项和的单测 `:303-315`；`:323` 钉 311 | 311 |
| 记录、点名项 | `lib.rs:156`（4096）、`:168-169`（56）、`:172-173`（67）；`:324-325` 钉 56、67 | E116 不用点名项，只用头宽 |
| 打包容器（码 3）头 | `lib.rs:39-40` 的 `PACKED_UNIT_HEADER_BYTES`（kb 标记 `.claude/kb/decisions/18-块里携带什么信息.md:320` 的 107）；`crates/singlefs-checker/src/lib.rs:286`、`:467`（`packed_unit_view`） | 与 E116 装置第 78 行 `PACK_HDR = 107` 同值；不读 `H` |
| 整理 / 搬迁 | `crates/singlefs-core/src/system_configuration.rs:49`、`:150`、`:174`（整理三水位，恒 0，「也没有触发路径」）；`crates/singlefs-harness/tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs:12`（「今天没有搬迁」） | `crates/` 里没有 E116 那四条臂的任何一条；`grep -rn 'settle\|整理\|compaction\|搬迁\|repack' crates/ --include=*.rs` 只命中水位字段、恢复路径注释与测试注释，没有搬迁写路径 |
| 每搬一个对象记一条 journal | `crates/` 里没有（同上） | E116 装置第 32 行把它登记成假设 `JOURNAL_PER_MOVE`、两种都跑；`H` 只在「记」那一支进账 |
| 记录标志 1 字节 | `crates/singlefs-core/src/journal.rs:151` 今天写 0 | E116 只用头宽这一个数，标志位不进任何一格 |

### 3.2 装置里读 `H` 的每一处（`grep -n 'JOURNAL_HEADER_BYTES\|307\|36_800_000\|165_840_384\|0\.051499\|2\.099192\|169_543_168\|368'` 现查）

| 行 | 是什么 | 这一次怎么动 |
|---|---|---|
| 25 | 文件头引 D23「journal 记录头 307（十个字段 78 + … + MAC 16）」 | 照 D23 已定项 4 今天的分项改成 311（加「本次发布内序号 4」） |
| 84 | `const JOURNAL_HEADER_BYTES: u64 = 307;` | 改成 311 |
| 156 | `journal_w` | 不动代码 |
| 186 | `name=config … journal_hdr=…` | 不动代码，值随常量走 |
| 337–341 | 单测 `bg_key_move_write_absolute`：钉 `journal_w = 36_800_000`、`move_write = 165_840_384` | 改钉第七节 B1、B2 |
| 346–358 | 单测 `payback_key_absolute`：钉 `0.051499` | 改钉 B3 |
| 360–376 | 单测 `payback_fill_b1_absolute`：钉 `2.099192`，注释里一串历史值「头 78 → 95 → 277 → 307 之后 …」 | 改钉 B4；注释末尾**追加**一句「头 307 → 311（已定项 4 加本次发布内序号 4）之后 <新值>」，前面的历史值一个字不删（那段注释自己写明「三个数都留在这里，不许回头改前两个」） |
| 434–435 | 单测 `fill_converges_to_key_order`：钉 `key = 165_840_384`、`fill_n = 169_543_168` | 改钉 B2、B5 |
| 497–502 | 单测 `journal_is_an_assumption`：注释「头 307 + 映射条目 61 = 368」、钉差 `36_800_000` | 注释改成 311 + 61 = 372，钉 B1 |

装置里 `JOURNAL_HEADER_BYTES` 只命中 25、84、156、186、337 五行（`grep -n` 现查），都在上表里。

## 四、跑之前已经存在的数

**这一次的答案有一部分在跑之前已经算得出来**：`H` 只加在 `journal=true` 两条臂的分子上，每格多 `100000 × 4 = 400000` 字节；我为钉单测算了两格的新值（第七节 B 类）与它们的翻面头宽，两格都离门槛 1 很远。其余各格我没有算。判定以产物为准。

| 数 | 出处 | 对判据的影响 |
|---|---|---|
| 36 800 000、165 840 384、0.051499、2.099192、169 543 168、368；跑前登记值 1.058893、第一版 3.127186、头 78 / 95 / 277 各版的 2.092080 / 2.092608 / 2.098260；容量 58 / 30 / 7 / 3 / 1、叶容量 267 / 905、容器数 1725、saved 3 220 275 200、`w / (cap − 1)` 的 0.035088 与 1.0、死亡模型的 1725 / 1637 / 29167 / 863 个容器 | 装置单测（第 265 行起的 `mod tests`） | 是 `H = 307` 的值与模型的几何，这一次改装置之前现跑复现（S1）；不读 `H` 的那些是 Q116.1 的「必须不变」 |
| 实验页小节标题：「17 单测 / 16 条变异全抓」「结论一：回本比的结构性下界是 `w / (cap − 1)`，与实现无关」「结论二：符号由整理策略定，不由对象大小定」「结论三：稳态占用在任何死槽比例上都不比 `pad` 差」「结论四：收益倍数与爆炸半径倍数是同一个数」「两条跑前登记的式子判否，三个数都留在源码里」「变异 `M14` 是真盲区」 | 看小节结构时 `grep -n '^#'` 撞见 | 是结论。第六节不照它写判据：判据是两次跑各格判出的值相同与否 |
| `.claude/kb/experiments-history.md:578` 一句：E116 的 M16 原文曾是改名回扫之前的注释写法、那时变异表跑不通 | 查 `PACKED_UNIT_HEADER_BYTES` 出处时 grep 撞见 | 今天 M16 的锚点与装置第 84 行逐字相同（我比过）；改常量之后它就不再命中，第九节写明要跟着改锚点 |
| `.claude/kb/experiments/142-第一个事务的干跑.md:157` 一整行（E142 第十一、十三次跑的装置↔`crates/` 比对结果） | 同一个 grep 撞见 | 与 E116 无关，不进任何判据；列在这里只为照实 |
| **我算出来的（第十三节命令二）**：`H = 311` 时 `journal_w = 37 200 000`、512 档 `bg_key` 的 `move_write = 166 240 384`、回本比 0.051623；512 档 `bg_fill` b = 1 的 `move_write = 6 760 374 400`、回本比 2.099316；b = 100000 的 `move_write = 169 943 168`；两格回本比 = 1 的头宽：`bg_key` 512 档 `H* = 96410463 / 3125 ≈ 30851.35`，`bg_fill` 512 档 b = 1 `H* = −4386249 / 125 ≈ −35090`（任何非负头宽都 ≥ 1） | 第十三节命令二原样输出 | ⚠️ 跑之前就存在的答案，只覆盖这两格；照纪律不删、不回改判据。它们是第七节 B 类锚点，第八节的敏感性取样点取在 512 档 `bg_key` 的 `H*` 两侧 |

## 五、臂、阳性对照、真实基线

### 5.1 臂

- **原实验的四条臂照旧，一个字不改**：`pad`、`bg_key`、`bg_fill`、`bg_ideal`（第二节 2.2 那张表）；`journal` 记与不记两支照旧都跑；两种死亡模型照旧。
- **这一次只加一个旋钮 `H`，两个被判的取值**：
  - `H = 307`：今天的装置一个字节不改，`bash research/scripts/replay.sh E116` 现跑；做完会怎样——与 `replay.sh` 登记的留存产物逐字节一致（S1 核，不是假设）。
  - `H = 311`：只按第三节 3.2 改常量、注释与钉值的单测，别的一行不动，重新出产物；做完会怎样——`bg_key`、`bg_fill` 两条臂 `journal=true` 的每一行 `move_write` 恰好多 400 000，回本比随之变；`name=config` 的 `journal_hdr=` 变；其余行逐字节不变。前一句从第 156 行的式子直接推得出（`n × 4`，`n = 100000`），后一句由 Q116.1 核。
- 两个取值都是真实发生过的格式，没有稻草人。

### 5.2 阳性对照（每一条臂都跑）

| 对照 | 对哪条臂 | 怎么做 | 该看到什么 | 看不到时 |
|---|---|---|---|---|
| **P0 原实验的三道对照** | 两个 `H` 各跑；每道对照遍历它原本覆盖的全部臂 | 不改：`name=positive_cap1`（三条打包臂 × 五个尺寸）、`name=discrimination`、`name=negative`（三条打包臂），与单测 `positive_control_cap1_matches_pad`、`discrimination_pad_pads`、`negative_control_zero` | 两次都绿，这几行两次逐字节相同 | 作废 V1 |
| **P1 头宽读没读到** | `bg_key`、`bg_fill` 两条读 `H` 的臂**各**核一次 | 比两份产物里 `name=payback_key size=512 … journal=true` 与 `name=payback_fill size=512 … b=1 journal=true` 两行的 `move_write` | 两行各自恰好差 400 000 | 任一行差不是 400 000 ⇒ 常量没读到或读错 ⇒ 作废 V2 |
| **P2 不读 `H` 的臂确实不读** | `bg_ideal`、`pad`，与两条打包臂的 `journal=false` 支 | 同一对产物里 `ideal_write=`、`ideal_payback=`、`journal=false` 的各行 | 逐字节相同 | 不同 ⇒ 停机 S2（`H` 漏进了不该进的地方） |

### 5.3 真实基线

真实基线是 `H = 311`（`crates/` 今天的头宽）。「各格两次判出同值」是正当结果，记「不变」；装置读没读到 `H` 由 P1 单独判。

### 5.4 实现量与分段

改 1 个常量、1 行文件头注释、5 个单测的钉值与 2 段注释；加第八节的敏感性输出（`pack_ledger` 加一个显式带头宽参数的版本，原函数用常量调它，约 30 行）与它的单测；变异表改 1 条锚点、加 2 条（第九节）；`replay.sh` 的 E116 一行换产物文件名。编译一次、单测一次、变异表两次（改前改后，第九节）、产物一次。**一段做完**，不分段。E116 实验页里引 307 的那几行（回扫报告列的第 41、81、89、99、118、119、125 行，我没读）由执行员在产物之后照第六节各格的判定逐句改，数先对上新产物的整行。

## 六、报哪些量与各自的判据

全部对应问题单第 2 行。每格各报各的判定，不合取。门槛全来自原判据（第二节 2.2：回本比 1、稳态占用不超过 `pad`、爆炸半径倍数不小于收益倍数），不来自 `H` 的两个取值。

| 格 | 怎么算 | 门槛 | 为什么不是同义反复 | 判定 | 问题单第 2 行：取什么值会让它翻面 |
|---|---|---|---|---|---|
| **Q116.1 头宽没漏进别的格** | `diff` 两份产物，列出不同的行 | 允许不同的只有：`name=config`（只许 `journal_hdr=` 一个字段不同）、`name=payback_key … journal=true` 五行的 `move_write=` 与 `payback=` 两个字段、`name=payback_fill … journal=true` 三十行的 `move_write=` 与 `payback=`、第八节新加的 `name=header_sensitivity` 行、`name=done` 的 `emitted=` | 装置有没有别处读 `H`，只有 diff 看得见 | 超出 ⇒ 停机 S2；在内 ⇒ 记「只动了 journal 那一项」 | 不直接翻面；它保证稳态占用（`name=death`）、爆炸半径（`name=blast`）、`bg_ideal` 各格两次同值是量到的 |
| **Q116.2 `bg_key` 回本比 ≥ 1（反向接受条款第一条）** | `name=payback_key … journal=true` 每个尺寸一行，`payback ≥ 1`（`inf` 记真） | 1 | 1 出自原条款「回本比 < 1 表示写得比省得少」，不读 `H` | 五个尺寸逐个比两次的真假；任一个不同 ⇒ 变 | 某尺寸 `payback(307) < 1 ≤ payback(311)`，即该格的 `H*` 落在 (307, 311] |
| **Q116.3 `bg_fill` 回本比 ≥ 1（反向接受条款第二条）** | `name=payback_fill … journal=true` 五个尺寸 × 六个 b，一格一行 | 1 | 同上 | 三十格逐个比；任一格不同 ⇒ 变，写明尺寸与 b | 同上，逐格 |
| **Q116.4 `bg_ideal` 回本比 ≥ 1（反向接受条款第三条）** | `ideal_payback=` 五个尺寸 | 1 | 同上 | 两次的真假逐个比 | 不读 `H`：只可能因 Q116.1 失守而变 |
| **Q116.5 稳态占用超不超过 `pad`（反向接受条款第四条）** | `name=death` 五尺寸 × 两种死亡模型 × 21 个 d 的 `pack_worse=` | 原条款「在任何 d 上超过 `pad`」 | 不读 `H` | 210 行逐行比 | 同上 |
| **Q116.6 爆炸半径倍数 ≥ 收益倍数（反向接受条款第五条）** | `name=blast` 五行的 `gain=`、`blast=` | 原条款 | 不读 `H` | 逐行比 | 同上 |
| Q116.7 两条读 `H` 的臂的回本比绝对值（只报数） | Q116.2 / Q116.3 那 35 行的 `payback=` 两次各报 | 不设门槛 | 它就是 `H` 的线性函数，由定义推得出 | 只报数；实验页里引 307 那一版回本比的句子照新值改 | — |

**够判点**：两份产物都出来、Q116.1 在允许集合之内、Q116.2–Q116.6 各判完一次，第 2 行就够判。

## 七、钉绝对值的断言

Q116.2 / Q116.3 是两次互比；两次一起错时靠下面的绝对值发现。

### 7.1 出自被测条款本身的（不符走第十节 F1「条款可能错」，不作废）

| # | 断言 | 出处 |
|---|---|---|
| A1 | 头宽 311 = 十个字段 78 + 三笔已定增量 17 + 本次发布内序号 4 + 新根段 188 + fsid 8 + MAC 16 | D23 已定项 4（第二节 2.1）。装置不从 `crates/` 引，新加一条单测钉这个和 |

### 7.2 独立算出、用命令核过的（第十三节命令二；不符先核我这边的算术，核完仍不符 ⇒ 停机 S2）

| # | 断言（`H = 311`，N = 100000，512 档 cap = 58、容器 1725） | 值 |
|---|---|---|
| B1 | `journal_w = N × (311 + 61)`；`journal_is_an_assumption` 的差 | **37 200 000** |
| B2 | `bg_key` 512 档 `move_write` = 113 049 600 + 12 288 000 + 3 702 784 + 37 200 000 | **166 240 384** |
| B3 | `bg_key` 512 档回本比 = 166 240 384 ÷ 3 220 275 200 | **0.051623**（单测容差照原来的 1e-6） |
| B4 | `bg_fill` 512 档 b = 1：113 049 600 + 100 000 × 32 768 + 101 725 × 32 768 + 37 200 000 = 6 760 374 400；回本比 | **2.099316**（容差照原来的 1e-5） |
| B5 | `bg_fill` 512 档 b = 100000：113 049 600 + 375 × 32 768 + 226 × 32 768 + 37 200 000 | **169 943 168** |
| B6 | 512 档 `bg_key` 回本比 = 1 的头宽 `H* = (3 220 275 200 − 129 040 384) ÷ 100 000 − 61`（129 040 384 = B2 减去 `journal_w`） | **96410463 / 3125 ≈ 30851.35** |

不读 `H` 的锚点（容量 58 / 30 / 7 / 3 / 1、叶容量 267 / 905、`w / (cap − 1)`、死亡模型的容器数、搬迁读 3 276 800 000）照原单测不改，两次都绿。

**互比配的绝对值**：Q116.2 配 B2、B3、B6；Q116.3 配 B4、B5；Q116.4–Q116.6 配原单测里不读 `H` 的那些绝对值。

## 八、轨迹与几何敏感性

### 8.1 轨迹

`name=death` 那 210 行本身就是稳态占用沿死槽比例 d 的轨迹（d 从 0 到 1 共 21 点），原装置就报整条，这一次照报；它不读 `H`，Q116.5 要求两次逐字节相同。没有别的被谓词消费、随轮次走的量。

### 8.2 几何敏感性（`H` 这个旋钮）

307 与 311 在我算过的两格上都落在门槛 1 的同一侧（第四节），只比这两点看不出判据有没有判别力。⇒ 执行员给 `pack_ledger` 加一个显式带头宽参数的版本，产物里加一行：

`name=header_sensitivity size=512 arm=bg_key journal=true h_star=<B6 那个分数化成的小数，保留 2 位> payback_at_30851=<…> ge1_at_30851=<0|1> payback_at_30852=<…> ge1_at_30852=<0|1>`

| 该看到 | 对应格 |
|---|---|
| `ge1_at_30851 = 0`、`ge1_at_30852 = 1`；`h_star` 与 B6 一致 | Q116.2 的判别力：同一个门槛在 `H*` 两侧判出不同的值 |

另一个方向（头变窄）不必取：回本比随 `H` 单调不减（第一节「读法写死」），头变窄只会让回本比更小，判定只可能往「< 1」那边走，已经被 30851 那一点覆盖。

**判别力自证**：单测里把门槛从 1 挪到 1.0001（比 `payback_at_30852` ≈ 1.0000202 高），`ge1_at_30852` 必须由 1 变 0；不变 ⇒ V3。（30851 与 30852 两点的回本比 ≈ 0.9999892 / 1.0000202，由 B2、B6 的式子手算。）

## 九、变异

**先后**：改装置之前，在今天的源码（`H = 307`）上跑 `bash research/scripts/mutate.sh e116-pack-settle research/e7-index-bench/src/bin/e116_pack_settle.rs research/mutations/e116_pack_settle.tsv`，留日志；改完之后再跑一遍。两次各报「抓到 / 无效 / 没红」三个数，逐条比。

- **M16（journal记录头算错）的锚点必须跟着改**：它的原文是第 84 行整行（`const JOURNAL_HEADER_BYTES: u64 = 307; // D23（journal 的角色与格式），登记名 JOURNAL_HEADER_BYTES`），常量改成 311 之后命中 0 次，`mutate.sh` 会直接报错退出（`mutation-sampling.md` 第七类）。只改原文里的 `307` 为 `311`，替换文（`= 8`）不改。改前那一次跑仍用旧锚点。M16 在 `H = 311` 上改变输出的取样点：`journal_w` 由 37 200 000 变 6 900 000，B1 / B2 的单测红。
- 除 M16 外的 15 条，两次的状态必须相同；任一条从「抓到」变成「无效」或「没红」⇒ V4。

新加两条（锚点由执行员照改完的源码取，唯一命中）：

| 条 | 改什么 | 在哪个取样点上改变输出 | 谁该红 |
|---|---|---|---|
| M17 头宽退回 307 | `const JOURNAL_HEADER_BYTES: u64 = 311;` → `307` | 512 档 `bg_key` 的 `move_write` 166 240 384 → 165 840 384 | B1 / B2 / B3 的单测 |
| M18 敏感性行的 `H*` 漏减映射条目 | `h_star` 的算式里去掉 `− 61`（`MAP_ENTRY`） | `h_star` 由 30851.35 变 30912.35 | 钉 B6 的那条单测 |

## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | **条款可能错**：A1 的分项和不等于 311 ⇒ 不作废，如实记「D23 已定项 4 的分项与它登记的头宽不符」 | 新加的那条单测红，且查过是分项本身加不到 311 |
| F2 | **结论变了（正当结果）**：Q116.2–Q116.6 任一格两次判出的值不同 ⇒ 第 2 行记「变」，写明哪一格、两边各是什么；实验页对应的结论句按新值改写，不许回头改门槛 | 两份产物里某一行 `payback=` 一次 < 1、一次 ≥ 1；或 `pack_worse=`、`ideal_payback=`、`name=blast` 任一字段两次不同 |
| F3 | **全部不变（正当结果）**：Q116.2–Q116.6 全部相同 ⇒ 记「不变」，不许写成装置坏了 | 上述比较全部相同，且 P1 看得到恰好 400 000 的差 |

## 十一、作废条款与停机条款

### 作废

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| V1 | 原实验三道对照（P0）任一次红 | `positive_control_cap1_matches_pad` / `discrimination_pad_pads` / `negative_control_zero` 失败，或那几行两次不同 |
| V2 | P1：头宽没读到或读错 | 两份产物 512 档 `bg_key` 或 `bg_fill` b = 1 那一行的 `move_write` 之差 ≠ 400 000 |
| V3 | 第八节敏感性行两侧判定相同，或挪门槛后判定不变 | `ge1_at_30851` 与 `ge1_at_30852` 相等 |
| V4 | 改常量后原有变异（M16 以外）有一条状态变差 | 两次 `mutate.sh` 日志逐条比，有一条从「抓到」变成「无效」或「没红」 |

### 停机（两边都查，写进报告交回）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| S1 | 基线没立住 | 改装置之前 `replay.sh E116` 不逐字节一致 |
| S2 | `H` 漏进了别的格，或产物与我的算术对不上 | Q116.1 超出允许集合；P2 不同；B1–B6 任一条与产物不符（核完我这边仍不符） |
| S3 | 装置与 `crates/` 对不上（`implementation-first.md` 第 4 条） | 开跑前现查 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_HEADER_BYTES` ≠ 311，或 `PACKED_UNIT_HEADER_BYTES` ≠ 107（装置 `PACK_HDR`） |

### 够判停机

第六节「够判点」一到就停，第 2 行记「变」或「不变」交回。没有第二段；没跑的量：无（Q116.7 只报数，随产物出）。

## 十二、修订

装置写完、产物产出之前，两条观测：

1. `nice -n 19 bash research/scripts/mutate.sh e116-pack-settle …`（改常量之前，跑原有 16 条表）第一次跑读数：M12、M14 各自在默认 120 秒超时内没跑完，被记成「⏱ 没跑完」；`ps` 现查（`ps -o pid,etimes,pcpu,args`）当时机器上同时有 8+ 个别的会话在跑全量 `cargo test`。按 `MUTATE_TIMEOUT=600` 用同一份未改代码重跑一次，M12、M14 均在数秒内被抓到（`tests::move_read_absolute`、`tests::death_uniform_absolute`），16/16 全抓、0 无效、0 没红——确认前一次的「没跑完」是机器超卖的计时假象，不是这两条变异真让代码不终止；取第二次（`MUTATE_TIMEOUT=600`）的 16/16 全抓当这一次的「改常量前」基线，不算「变」也不算新发现。收到主 agent 关于线程上限的消息之后（协调消息：线程上限 4），后续全部 `cargo`/`mutate.sh` 命令改用 `bash research/scripts/capped.sh 4 …`。
2. 单测跑完（20/20 绿）、产物一次没跑之前，把第八节敏感性代码写成一个显式带头宽参数、不读全局常量的独立函数 `bg_key_move_write_with_header(header_bytes)`（与原判据「专用函数，不借用全局 `JOURNAL_HEADER_BYTES` 常量」这句一致），不是文本上直接改 `pack_ledger` 内联算式；对判据、门槛、臂定义没有影响，只是实现选择。

两条都不改判据、不改门槛、不加减臂。

第三条（主 agent 转达门禁 12 号红，用户点名）：第五节 5.2 那一行「不读 `H` 的臂确实不读」原来的标签是在 `P1` 后面加一个撇号，改名 `P2`（这份登记 P 家族里下一个未用号，与 P0、P1 不撞），出现在第 200、309 行两处；全仓 `bash .claude/gate.d/12-no-prime-marks.sh` 对这份文件已转绿。不改判据、不改门槛，只改标签字面。


## 十三、读过的文件与跑过的命令

五份头 311 重跑登记（E43、E116、E155、E157、E159）是同一次派发里一起写的，这一节五份相同：列的是这一次派发里读过的全部文件，不只这一份用到的。行号是读的那一刻（2026-09-24 20:37–21:50 JST）的行号。`research/results/` 下的产物一份都没读（只在 `git log --all --name-only` 的输出里看到过文件名）。

### 13.1 规则、共用约束、门禁与脚本

- `.claude/agent-common.md`：1–71
- `.claude/singlefs-ai-sop/rules/test-discipline.md`：`grep -n '^#'` 全部标题；44–153
- `.claude/rules/three-way-inference.md`：`grep -n '^#'`；128–147
- `.claude/singlefs-ai-sop/rules/evidence-discipline.md`：`grep -n '^#'`；119–190
- `.claude/rules/mutation-sampling.md`：`grep -n '^#'`；45–53、76–95
- `.claude/rules/implementation-first.md`：`grep -n '^#'`；1–25
- `.claude/hooks/agent-write-scope.tsv`：`head -30`（全文）
- `.claude/gate.d/27-format-constants.sh`：60–125
- `.claude/gate.d/69-evidence-in-repo.sh`：`grep -n 'tmp'` 的命中行；20–37、178–200
- `.claude/gate.d/`：`ls`；`grep -ln 'preregistration\|prereg' .claude/gate.d/ -r` 零命中
- `research/scripts/replay.sh`：1–60；`grep -n 'e43\|e116\|e155\|e157\|e159'` 命中 63、77、171–174、176、192
- `research/scripts/quote-kb.py`：1–60；`research/scripts/replace-once.py`：1–30；`research/scripts/mutate.sh`：1–40
- `research/e7-index-bench/src/lib.rs`：67–88
- `research/e7-index-bench/Cargo.toml`：`grep` 命中 317–318、421–438、445–446

### 13.2 派发给的输入、被测条款与原登记

- `research/prompts/m2-header311-rerun-questions.md`：1–15（全文）
- `research/prompts/m2-header311-sweep-report.md`：`grep -n '^#'`；51–73
- `.claude/kb/decisions/23-journal的角色与格式.md`：`grep -n '^#'`；139–165、440–459
- `research/prompts/e157-preregistration.md`：`grep -n '^#'`；1–66、280–425、464–612
- `research/prompts/e159-preregistration.md`：`grep -n '^#'`；`grep -n 'smoke\|冒烟\|anchors'` 命中行；1–46、383–470、501–534、609–645
- `research/prompts/e156-r2-prereg.md`：1–12
- 归档的 E155 四份登记（`git show 3cff909^:research/prompts/e155-preregistration.md`、`git show 3cff909^:research/prompts/e155-r2-prereg.md`、`git show 8186d5b^:research/prompts/e155-r3-prereg.md`、`git show 8186d5b^:research/prompts/e155-r4-prereg.md`，拷进草稿目录读）：各自 `grep -n '^#'` 的前 60 行；`grep -n '307\|3789\|⌈[^⌉]*67\|/ *67\|÷ *67\|\b67\b\|RECORD_HEADER\|记录头宽'` 的命中行（第 1 次 659、720、727、749、769、815–824、888、899、907、912；第 2 次 25–26、33、262、563、736、782–791、826、851、857、882–891、997、1012、1262、1289；第 4 次 266–311、401、403、410、459、469、556、567、584、608、650–651、865、874、913、973、1030、1033、1053、1066）；第 3 次 `grep -n '67'` 的命中行（104、109、244、414、467、572、585、590、592、593）；四份的「六」「十」与 5.x 网格小节的起始行

### 13.3 `crates/`

- `crates/singlefs-format/src/lib.rs`：150–180、295–330、302–316；`grep` 命中 11–12、39–44、74、119、152–153、156、172、175–176、184–198、206、254、286、321–325、340
- `crates/singlefs-core/src/journal.rs`：15–35、140–200、162–170、230–250；`grep` 命中 21–27、113、151、167、169、227、259、283、294–311、360、379
- `crates/singlefs-checker/src/lib.rs`：`grep` 命中 13–15、286、454、467、478、560、590–591
- `crates/singlefs-core/src/transaction.rs`：`grep` 命中 15、2621
- `crates/singlefs-core/src/system_configuration.rs`：`grep` 命中 49、148、150、172、174、348、382、650
- `crates/singlefs-core/src/recovery.rs`：`grep` 命中 1679
- `crates/singlefs-harness/src/model.rs`：`grep` 命中 251
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`grep` 命中 3204–3205、3239、3241
- `crates/singlefs-harness/tests/second_transaction_parallel_line_three_many_inodes.rs`：`grep` 命中 28、373、391
- `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs`：`grep` 命中 7、12、230
- `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`：`grep` 命中 404

### 13.4 实验装置与变异表

- `research/e7-index-bench/src/bin/e43_extension_point_budget.rs`：1–687（全文）
- `research/e7-index-bench/src/bin/e116_pack_settle.rs`：1–504（全文）
- `research/e7-index-bench/src/bin/e155_fsync_write_volume.rs`：18–30、74–78、166；`grep` 命中（`RECORD_HEADER_BYTES\|JOURNAL_HEADER\|307\|3789\|3785\|\b67\b\|NAMED_ENTRY\|RECORD_BYTES`、`HEADER\|NAMED_ITEM_BYTES`）
- `research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs`：36–48、86–90、177；同上 `grep`
- `research/e7-index-bench/src/bin/e155_third_run_release_cascade.rs`：28–38、58–63、152；同上 `grep`
- `research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs`：37–48、87–91、178；同上 `grep`
- `research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs`：36–46、55–80、296–315、495–512；`grep` 命中
- `research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs`：40–50、44–48、90–94、3062–3102（`print_anchors` 里带 `format!` 的行）、3085–3130、3230–3260；`grep` 命中
- `research/mutations/e43_extension_point_budget.tsv`、`e116_pack_settle.tsv`、`e157_parallel_line_one_clauses.tsv`：全文；`e159_fsync_wait_group_commit.tsv`：第一列全部；四张 E155 变异表：`grep -n 'RECORD_HEADER\|307\|NAMED_ITEMS_PER_RECORD\|JOURNAL_HEADER'` 命中行（第 1 次表第 12 行、第 2 次表第 12、29 行、第 4 次表第 4、5 行）与 `wc -l`

### 13.5 kb 实验页（为判问题单第 7 行、为找原判据；结果节没读，撞见的数已列进第四节）

- `.claude/kb/experiments/43-扩展点字节上限.md`：`grep -n '^#'`；2–74、97–112
- `.claude/kb/experiments/116-打包容器的账·补元数据写与整理策略.md`、`155-每次持久化的写量三种fsync形态与反事实上界.md`、`157-并行线一两条条款的计数模型.md`、`159-fsync等待时间随并发数组提交与wal两臂.md`：`grep -n '^#'`（标题行里有结论与变异账，第四节已列）；`159-…` 另有 `grep -n 'smoke\|冒烟\|anchors'` 命中的 5、14、15、32、46、61、62、64、68、73 行
- `.claude/kb/experiments/23-journal几何.md`：63–67
- `.claude/kb/experiments/39-反向链挡不挡得住残留记录.md`：`grep -n '^#'` 的前 20 行；`grep -n '判据'` 命中行；11–27、57–60
- `.claude/kb/experiments/42-一事务几条记录.md`：54–57
- `.claude/kb/experiments/49-反向链宽度32还是64.md`：1、7、80、145
- `.claude/kb/experiments/61-反向链hash算法的均匀性.md`：93–96
- `.claude/kb/experiments/75-记录尺寸与环几何.md`：`grep -n '^#'` 的前 10 行；24–40
- 查 `PACKED_UNIT_HEADER_BYTES` 出处时 `grep -rn` 撞见：`.claude/kb/experiments-history.md:578`、`.claude/kb/experiments/142-第一个事务的干跑.md:157`、`.claude/kb/decisions/18-块里携带什么信息.md:320`

### 13.6 跑过的命令（原样；读文件的 `sed` / `awk` / `grep -n` / Read 在上面按行列过，不重列）

```
$ git log --all --name-only --format='%h %ad' --date=short -- 'research/prompts/e43*' 'research/prompts/e116*' 'research/prompts/e155*' 'research/prompts/e157*' 'research/prompts/e159*'
$ git log --all --name-only --format='' | grep -iE '(^|/)e0?43[-_]|(^|/)e116[-_]|e155|e157|e159' | sort -u
$ git log --all --format='' --name-only | grep -E 'research/prompts/e[0-9]+-r[0-9]+-prereg' | sort -u
$ git log --all --diff-filter=D --format='%h' -- research/prompts/<四份 E155 登记>   # 得 3cff909、3cff909、8186d5b、8186d5b
$ git show 3cff909^:research/prompts/e155-preregistration.md > <草稿目录>/orig/e155-preregistration.md   # 另三份同形
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/quote-d23.md '.claude/kb/decisions/23-journal的角色与格式.md@#### 已定项 4：记录头约束在单个原子单元内，头 311 字节' '.claude/kb/decisions/23-journal的角色与格式.md@#### 已定项 17：点名项 56 字节的字段表'
  ✓ 2 段整抄进 …/quote-d23.md，回读逐字节一致
$ （问题单表头 5–6 行与第 1、7 / 2 / 3 / 4 / 5 行各抄一份，五次，都回读逐字节一致）
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/orig-e43.md '.claude/kb/experiments/43-扩展点字节上限.md@### 要测什么、判据、失败条款'
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/orig-e43-src.md 'research/e7-index-bench/src/bin/e43_extension_point_budget.rs:167-206'
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/orig-e116.md 'research/e7-index-bench/src/bin/e116_pack_settle.rs:34-65'
$ grep -rn 'extension_point\|扩展点\|ExtensionPoint' crates/ --include=*.rs          # 2 行：system_configuration.rs:348、model.rs:251
$ grep -rn 'settle\|整理\|compaction\|搬迁\|repack' crates/ --include=*.rs           # 水位字段、恢复注释、测试注释，没有搬迁写路径
$ ls research/scripts | grep e159                                                    # 零命中
$ TZ=Asia/Tokyo date '+%Y-%m-%d %H:%M JST'                                          # 2026-09-24 20:47 JST（开写前）
```

`<草稿目录>` 是派发提示给的那个目录；命令二的两个脚本全文与原样输出在下面，执行员要复核时可以原样拷进 `research/` 下重跑。

#### 命令二：独立锚点（`anchors_h311.py`，全文）

```python
#!/usr/bin/env python3
# 头 311 重跑登记的独立锚点。常量从 crates/singlefs-format/src/lib.rs（156 / 165 / 168-169 / 172-173）
# 与 D23 已定项 4 / 17 手抄；不 import 任何装置，公式照各装置源码里那一行的算式手写。
from fractions import Fraction as F
from math import floor
REC, H_OLD, H_NEW, ITEM = 4096, 307, 311, 56
print("== 共用：一条记录装几个点名项")
for h in (H_OLD, H_NEW):
    print(f"  H={h}: (4096-H)={REC-h}  floor/56={(REC-h)//ITEM}  余={(REC-h)-((REC-h)//ITEM)*ITEM}  67*56+H={67*ITEM+h}  68*56+H={68*ITEM+h}")
hs67 = [h for h in range(0, REC) if (REC-h)//ITEM == 67]
print(f"  装 67 项的 H 区间：[{min(hs67)}, {max(hs67)}]；H={min(hs67)-1} 装 {(REC-min(hs67)+1)//ITEM} 项，H={max(hs67)+1} 装 {(REC-max(hs67)-1)//ITEM} 项")
print("== E43 自证单元档")
for h in (H_OLD, H_NEW):
    r512, r4096 = 512-h, 4096-h
    print(f"  H={h}: room512={r512} (点名项 {r512//ITEM})  room4096={r4096} (点名项 {r4096//ITEM})  占512={F(h,512)*100:.4f}%  bound=min(room512,255)={min(r512,255)}  864/bound={864/min(r512,255):.4f}")
side = [h for h in range(1, 512) if 512-h >= 255]
print(f"  绑定侧换成根槽 255 的 H：H <= {max(side)}")
for lb in (7, 9, 11, 13):
    ok = [h for h in range(1, 512) if min(512-h, 255) >= lb]
    print(f"  下界 {lb} 仍放得进 min(512-H,255) 的 H：H <= {max(ok)}")
ok128 = [h for h in range(1, 512) if min(512-h, 255) >= 128]
print(f"  D21 举例 128 仍放得进的 H：H <= {max(ok128)}")
# 挂载判定只在 N > 512-H（atomic 512）或 N > 4096-H（atomic 4096）处与 H 有关
for n in (0, 128, 512):
    for atomic in (512, 4096):
        flip = [h for h in range(0, atomic) if n > atomic - h]
        print(f"  mount N={n} atomic={atomic}: 自证溢出判拒的 H 从 {min(flip) if flip else '无'} 起")
print("== E116 journal 那一项（N = 100000，映射条目 61）")
N, UNIT, NODE, W = 100_000, 32768, 16384, 2
data_w = 1725 * UNIT * W
saved = N * UNIT - 1725 * UNIT
for h in (H_OLD, H_NEW):
    j = N * (h + 61)
    key = data_w + 375 * NODE * W + 113 * NODE * W + j
    fill1 = data_w + 100_000 * NODE * W + 101_725 * NODE * W + j
    filln = data_w + 375 * NODE * W + 226 * NODE * W + j
    print(f"  H={h}: journal_w={j}  key512.move_write={key}  key512.payback={key/saved:.6f}  fill512.b1.move_write={fill1}  fill512.b1.payback={fill1/saved:.6f}  fill512.bN.move_write={filln}")
base_key = data_w + 375 * NODE * W + 113 * NODE * W
base_fill1 = data_w + 100_000 * NODE * W + 101_725 * NODE * W
print(f"  saved={saved}")
print(f"  回本比 = 1 的 H*：key512 {F(saved - base_key, N) - 61}  fill512.b1 {F(saved - base_fill1, N) - 61}")
print("== E39 四档（行 7）")
for base in (303, 307):
    print(f"  base={base}: 档 {[base+k for k in (1,2,4,8)]}  增量 {[f'{k/base*100:.2f}%' for k in (1,2,4,8)]}")
print("== 十字段与增量拆分（D23 已定项 4 正文）")
print(f"  78+9+4+4+4+188+8+16={78+9+4+4+4+188+8+16}  78+17={78+17}  311-4(反向链)={311-4}")
```

```
$ cd <草稿目录> && nice -n 19 python3 anchors_h311.py
== 共用：一条记录装几个点名项
  H=307: (4096-H)=3789  floor/56=67  余=37  67*56+H=4059  68*56+H=4115
  H=311: (4096-H)=3785  floor/56=67  余=33  67*56+H=4063  68*56+H=4119
  装 67 项的 H 区间：[289, 344]；H=288 装 68 项，H=345 装 66 项
== E43 自证单元档
  H=307: room512=205 (点名项 3)  room4096=3789 (点名项 67)  占512=59.9609%  bound=min(room512,255)=205  864/bound=4.2146
  H=311: room512=201 (点名项 3)  room4096=3785 (点名项 67)  占512=60.7422%  bound=min(room512,255)=201  864/bound=4.2985
  绑定侧换成根槽 255 的 H：H <= 257
  下界 7 仍放得进 min(512-H,255) 的 H：H <= 505
  下界 9 仍放得进 min(512-H,255) 的 H：H <= 503
  下界 11 仍放得进 min(512-H,255) 的 H：H <= 501
  下界 13 仍放得进 min(512-H,255) 的 H：H <= 499
  D21 举例 128 仍放得进的 H：H <= 384
  mount N=0 atomic=512: 自证溢出判拒的 H 从 无 起
  mount N=0 atomic=4096: 自证溢出判拒的 H 从 无 起
  mount N=128 atomic=512: 自证溢出判拒的 H 从 385 起
  mount N=128 atomic=4096: 自证溢出判拒的 H 从 3969 起
  mount N=512 atomic=512: 自证溢出判拒的 H 从 1 起
  mount N=512 atomic=4096: 自证溢出判拒的 H 从 3585 起
== E116 journal 那一项（N = 100000，映射条目 61）
  H=307: journal_w=36800000  key512.move_write=165840384  key512.payback=0.051499  fill512.b1.move_write=6759974400  fill512.b1.payback=2.099192  fill512.bN.move_write=169543168
  H=311: journal_w=37200000  key512.move_write=166240384  key512.payback=0.051623  fill512.b1.move_write=6760374400  fill512.b1.payback=2.099316  fill512.bN.move_write=169943168
  saved=3220275200
  回本比 = 1 的 H*：key512 96410463/3125  fill512.b1 -4386249/125
== E39 四档（行 7）
  base=303: 档 [304, 305, 307, 311]  增量 ['0.33%', '0.66%', '1.32%', '2.64%']
  base=307: 档 [308, 309, 311, 315]  增量 ['0.33%', '0.65%', '1.30%', '2.61%']
== 十字段与增量拆分（D23 已定项 4 正文）
  78+9+4+4+4+188+8+16=311  78+17=95  311-4(反向链)=307
```

#### 命令二 b：E116 第八节敏感性两点（`anchors_h311_b.py`，全文）

```python
#!/usr/bin/env python3
# E116 第八节敏感性两点的回本比（公式照 e116_pack_settle.rs 第 156 行与 B2 手写，不 import 装置）
base_key = 1725 * 32768 * 2 + 375 * 16384 * 2 + 113 * 16384 * 2
saved = 100_000 * 32768 - 1725 * 32768
print("base_key(不含 journal) =", base_key, " saved =", saved)
for h in (307, 311, 30851, 30852):
    p = (base_key + 100_000 * (h + 61)) / saved
    print(f"  H={h}: payback={p:.7f}  ge1={int(p >= 1.0)}  ge1.0001={int(p >= 1.0001)}")
```

```
$ cd <草稿目录> && nice -n 19 python3 anchors_h311_b.py
base_key(不含 journal) = 129040384  saved = 3220275200
  H=307: payback=0.0514988  ge1=0  ge1.0001=0
  H=311: payback=0.0516230  ge1=0  ge1.0001=0
  H=30851: payback=0.9999892  ge1=0  ge1.0001=0
  H=30852: payback=1.0000202  ge1=1  ge1.0001=0
```

