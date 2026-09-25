# E155 重跑登记（第 5 次，记录头 307 → 311）：四份装置在头 311 下还判不判出原来的结论

写于 2026-09-24 21:20 JST，四份装置改之前、这一次的任何产物之前。

**文件名取 `r5`，不取派发提示给的 `r1`**：派发提示的前提是「仓里都还没有 `e<号>-r<n>-prereg.md`」，工作区里确实没有，但 git 历史里 E155 已有第 2、3、4 次的登记（`research/prompts/e155-r2-prereg.md`、`e155-r3-prereg.md` 删于提交 `3cff909` 与 `8186d5b`，`e155-r4-prereg.md` 删于 `8186d5b`），第 1 次是 `e155-preregistration.md`（删于 `3cff909`）。四份装置正是这四次跑各自的装置。仓里「`r<n>` = 第 n 次」（`e156-r2-prereg.md` 标题「第 2 次」），这一次是第 5 次。要不要改回 `r1` 由主 agent 定，正文不自引文件名。

**原判据在哪（这一次拿来判的尺子）**：四份原登记都已归档，读法 `git show <提交>^:<路径>`：

| 装置（`research/e7-index-bench/src/bin/`） | 那一次的登记 | 判据节 | 失败条款节 |
|---|---|---|---|
| `e155_fsync_write_volume.rs`（第 1 次） | `git show 3cff909^:research/prompts/e155-preregistration.md` | 第 868–996 行「六」 | 第 997–1036 行「十」 |
| `e155_second_run_fsync_write_volume.rs`（第 2 次） | `git show 3cff909^:research/prompts/e155-r2-prereg.md` | 第 960–1088 行 | 第 1089 行起 |
| `e155_third_run_release_cascade.rs`（第 3 次） | `git show 8186d5b^:research/prompts/e155-r3-prereg.md` | 第 372–475 行 | 第 476–515 行 |
| `e155_fourth_run_group_commit_concurrency.rs`（第 4 次） | `git show 8186d5b^:research/prompts/e155-r4-prereg.md` | 第 538–661 行 | 第 662–725 行 |

判据、门槛、作废条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**装置写在哪（写死）：四份都在 `research/e7-index-bench/src/bin/`（独立手写计数模型，不与 `crates/` 共用代码），不是入库装置。** 问的是 E155 各次跑前写死的判据在头 311 下判不判得出原结论，不是 `crates/` 今天这份代码的性质。变异表 `research/mutations/e155_fsync_write_volume.tsv`、`e155_second_run_fsync_write_volume.tsv`、`e155_third_run_release_cascade.tsv`、`e155_fourth_run_group_commit_concurrency.tsv`；复跑行 `research/scripts/replay.sh` 的 `E155`、`E155R2`、`E155R3`、`E155R4` 四行。

## 一、问题

**主 agent 给的问题（逐字）**：记录头 `JOURNAL_HEADER_BYTES` 从 307 改成 311 之后，每个实验跑前写死的判据在 311 下还判不判出原来的结论。

**先核的一件事（问题单第 3 行「先核」）：四份装置与 E159 用的 `RECORD_HEADER_BYTES` 是不是 `JOURNAL_HEADER_BYTES` 那个量——是。** 依据：

1. 四份装置里它的文档注释逐字是「记录头宽度（`journal_named_items_per_record` 用它反推容量）」（第 1 次第 25 行、第 2 次第 42 行、第 4 次第 43 行；第 3 次第 35 行没有注释）；
2. 四份装置里它只出现在一处，都是 `const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;`（第 1 次第 77 行、第 2 次第 89 行、第 3 次第 62 行、第 4 次第 90 行；`grep -n 'RECORD_HEADER_BYTES'` 每份各命中 2 行：定义与这一行），`RECORD_BYTES = 4096`、`NAMED_ITEM_BYTES = 56`；
3. `crates/` 的同一个式子是 `JOURNAL_NAMED_ENTRIES_PER_RECORD = (JOURNAL_RECORD_BYTES − JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES`（`crates/singlefs-format/src/lib.rs:172-173`），而 `crates/singlefs-core/src/journal.rs:184` 在写完 `JOURNAL_HEADER_BYTES` 个字节之后才开始写点名项——「记录里点名项数组之前的那段字节数」就是 D23 已定项 4 的头宽。

⇒ 按派发提示「是才改」：四份都改成 311。**只改值，不改名**（改名会让两张变异表里 M10 的锚点 `(RECORD_BYTES - RECORD_HEADER_BYTES)` 命中 0 次，也会动 E159 停机条款 S2 那道逐函数 diff 的范围；不改名的代价见第十一节 S3 下面那一句）。

**读法写死**：

| 名字 | 写死成 |
|---|---|
| 头宽 `H` | 四份装置里的 `RECORD_HEADER_BYTES`。被判的取值 `H = 307`（今天的装置，改之前现跑当基线）与 `H = 311` |
| `H` 进模型的唯一通道 | `NAMED_ITEMS_PER_RECORD_MAIN = ⌊(4096 − H) / 56⌋`，四份装置的记录条数（`⌈点名项 / 该值⌉`，K7 口径）全从它来 |
| 「原来的结论」 | 不读实验页的结论节。读作：四份原登记的每一条判据在 `H = 307` 现跑产物上判出的值 |
| 「还判不判出」 | 四份原登记的判据都是各自产物行的函数（判据写在登记里、算在产物上）。⇒ 一份装置在两个 `H` 下的产物逐字节相同，它那一次登记的**每一条**判据就判出同一个值，不必逐条重判；不相同时才逐条回到原登记判（第六节 Q155.3） |

**问题单（`research/prompts/m2-header311-rerun-questions.md`，表头与第 3 行，quote-kb 机械抄）**：

**出处 `research/prompts/m2-header311-rerun-questions.md:5-6`（整段抄，未转述）**

```markdown
| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
```

**出处 `research/prompts/m2-header311-rerun-questions.md:9-9`（整段抄，未转述）**

```markdown
| 3 | E155（每次持久化的写量三种 fsync 形态与反事实上界） 四份装置在头 311 下结论变不变 | 不变 / 变 | 各形态每次持久化的写量排序或与上界的关系变 | 四份装置的 `RECORD_HEADER_BYTES` 改成 311（它与 `JOURNAL_HEADER_BYTES` 是不是同一个量先核），重跑，逐格判 | 开着 |
```

## 二、被测条款与它引的定义

用 `research/scripts/quote-kb.py` 抄进草稿目录、回读逐字节一致之后追加。四份原登记的判据不整段抄进来（合计约 500 行，且归档在 git 里、读法见文件头那张表）：这一次的判定不逐条用它们，只在 Q155.3 触发时逐条回去判。原登记里提到 67 或头宽 307 的地方（`grep -n '307\|\b67\b\|RECORD_HEADER\|记录头宽'` 在四份归档文本上现查）是：第 1 次登记 K7（第 769 行）、Q1.8（第 888 行）、Q5.4（第 899 行）、A1（第 907 行）、A6（第 912 行）；第 2 次登记「读法写死」（第 25–26、33 行）、K7（第 736 行）、5.2 三臂（第 782–791、882–891 行）、A1（第 997 行）、B1（第 1012 行）；第 3 次登记没有；第 4 次登记「读法写死」（第 266–274 行）、5.2（第 302–311、459–469 行）、Q7.10（第 556 行）、A1（第 567 行）、M4 / M5（第 650–651 行）。**全部经由「一条记录 67 项」这一个数，没有一条直接用头宽。**

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

## 三、实现今天的样子

### 3.1 `crates/`（2026-09-24 现查）

| 处 | 文件与行 | 今天是什么样 |
|---|---|---|
| 头宽 | `crates/singlefs-format/src/lib.rs:164-165`；分项和单测 `:303-315`；`:323` | 311 |
| 一条记录装几项 | `lib.rs:156`（4096）、`:168-169`（56）、`:172-173`（`(4096 − 311) / 56`）、`:325` 钉 67 | 67 |
| 用这个容量的地方 | `crates/singlefs-core/src/transaction.rs:2621`（`named_unit_capacity = JOURNAL_NAMED_ENTRIES_PER_RECORD`，重写角色多于它就在动分配器之前拒掉）；`crates/singlefs-core/src/journal.rs:360-379` | 今天一次发布只写一条记录，点名项多于 67 就拒；四份装置的「⌈点名项 / 67⌉ 条记录」是模型给的，`crates/` 里没有多条记录的路径（第 2、4 次原登记第三节写过同一件事） |
| 写头与点名项的次序 | `journal.rs:184`（`assert_position(JOURNAL_HEADER_BYTES, "记录头")` 之后才写点名项）、`:241-244`（读的一侧同样从 `JOURNAL_HEADER_BYTES` 起按 56 一项读） | 「点名项数组之前的字节数」= 头宽，第一节「先核」第 3 条的依据 |
| 记录标志 1 字节 | `journal.rs:151` 今天写 0 | 四份装置不建头里的逐字节布局，标志位不进任何一格 |

### 3.2 装置里读 `H` 的每一处

| 装置 | 定义行 | 用它的行 | 钉它的单测 | 这一次怎么动 |
|---|---|---|---|---|
| 第 1 次 `e155_fsync_write_volume.rs` | 26 | 77 | 166（`assert_eq!(NAMED_ITEMS_PER_RECORD_MAIN, 67, …)`） | 26 行改 311；新加一条钉余数的单测（第七节 B2） |
| 第 2 次 `e155_second_run_fsync_write_volume.rs` | 43 | 89 | 177 | 同上 |
| 第 3 次 `e155_third_run_release_cascade.rs` | 35 | 62 | 152 | 同上 |
| 第 4 次 `e155_fourth_run_group_commit_concurrency.rs` | 44 | 90 | 178 | 同上 |

⚠️ 第 3 次装置第 32–33 行有一句「打印出去的键名还叫 `system_configuration_slot_bytes` … 下次重跑这个实验时一起改」。**这一次不改那个键名**：它会让第 3 次的产物因为与头宽无关的原因不再逐字节相同，Q155.2 就分不清差别是谁造成的。要改另派，交主 agent（第十一节末尾）。

## 四、跑之前已经存在的数

**这一次的答案在跑之前就算得出来**：`⌊(4096 − 307) / 56⌋ = ⌊(4096 − 311) / 56⌋ = 67`（第十三节命令二），而 `H` 进四份模型只有这一条通道（第一节「先核」第 2 条）。⇒ 按模型结构，四份产物在两个 `H` 下应当逐字节相同。照纪律不删、不回改判据；这一次重跑的用处是：装置、单测、变异、产物跟着格式常量一起动（D23 已定项 4 射程）；由产物而不是由我的推理确认「没有第二条通道」；新加一条能分辨 307 与 311 的单测（否则两个值在全部单测上同值，第七节 B2 与第九节 M23 管这件事）。判定以产物为准。

| 数 | 出处 | 对判据的影响 |
|---|---|---|
| 67 = ⌊3789 / 56⌋（307）与 ⌊3785 / 56⌋（311）；余数 37 / 33；装 67 项的头宽区间 `[289, 344]`，`H = 288` 装 68 项、`H = 345` 装 66 项 | 第十三节命令二原样输出 | ⚠️ 跑之前就存在的答案。第八节的敏感性点取在 288 / 289、344 / 345 两个边界上 |
| 实验页标题整句（含「第四次跑第一段 … k ≤ 1024 上，`s` 在全部 360 条判定串上都没有出现 `s ≤ 0` 的翻面点，触发失败条款 F11」「75 单测全绿」「34 条变异表 … 跑到 31/34 时因锚点在源码里不再唯一命中而中止（第七类，R2_M13）」「42 单测 / 15 条变异全抓」「47 单测 / 27 条变异 26 抓 1 等价」「52 单测 / 34 条变异 33 抓 1 等价」「22 单测 / 14 变异全抓」） | 看小节结构时 `grep -n '^#'` 撞见 | 是结论与变异账。第六节不照它写判据；第九节要求每张变异表改前改后各跑一次、比三个数，不拿这里的账当基线 |
| 回扫报告第二节第 2 小节 E155 那一行：实验页第 120 行是「第四次跑第一段（2026-09-21）」当时的 sha256 停机自检的历史事件句 | 回扫报告 | 历史句不改；与判据无关 |
| 第 2 次登记第 851 行（它的「跑之前已经存在的数」）：发布 B 344 576 字节、21 写调用、块层 26、刷写 4、FUA 1；容量 144 / 147、233 / 135、812 / 169、477 / 150、294 / 143、81、67；G23.1 份额 47.70% / 50.07%；wal_full / 乙的 fsync 与 checkpoint 字节；甲 − wal_full = 139 776；P = 145 各臂字节；环 196 608 槽、在飞上限 65 536；「wal_full 省 ≈ 37.9%、乙省 ≈ 46.8%」 | 为找原判据里哪些格用 67 而 grep 归档登记时读到 | 是第 1、2 次的产物数与锚点。这一次不引、不判；它们要么随 Q155.2 逐字节复现，要么走 Q155.3 |
| 第 3 次登记 grep 命中行里的数：966 367 641 条、996 ppm、倍数 1.88 → 1.80、830 ppm；B-r3-7 的分支比序列；M9「44.404694 变 40.672004」 | 同上 | 同上 |
| 第 4 次登记 grep 命中行里的数：B-r4-2 wal_full 各 k 的 `(字节, 写调用, 单元, 记录)`（k = 64 为 (4333568, 136, 67, 1)）；k = 64 三臂 甲 (4481536, 149, 71, 2)、wal_full (4333568, 136, 67, 1)、乙-M (4300800, 134, 66, 1)；M4「4481536 / 149 变 4473344 / 147」；M5「脏叶 113 ⇒ 2 条变 1 条，一批少 8192 字节」 | 同上 | 同上。其中「71 个点名项 ⇒ 2 条记录」「113 片脏叶 ⇒ 2 条」是 67 这条通道上离边界最近的两格：67 不变它们就不变 |

## 五、臂、阳性对照、真实基线

### 5.1 臂

- **四次原登记的臂照旧，一个字不改**（甲、wal_full、乙 / 乙-M、K9 / K10 各口径、第 3 次的三种释放写法、第 4 次的组提交 k 轴，全按各自登记第五节）。这一次不加臂、不删臂。
- **这一次只加一个旋钮 `H`，两个被判的取值**：
  - `H = 307`：四份装置一个字节不改，`bash research/scripts/replay.sh E155 E155R2 E155R3 E155R4` 现跑；做完会怎样——四份都与 `replay.sh` 登记的留存产物逐字节一致（S1 核）。
  - `H = 311`：四份各只改定义行与加一条单测，重新出四份产物；做完会怎样——`NAMED_ITEMS_PER_RECORD_MAIN` 仍是 67，四份产物与 `H = 307` 那四份逐字节相同。前半句由第十三节命令二的算术推出，后半句由前半句加「只有这一条通道」推出；两句都由 Q155.1、Q155.2 在产物上核，不当判据的前提用。
- 两个取值都是真实发生过的格式，没有稻草人。

### 5.2 阳性对照（每一条臂都跑）

| 对照 | 对哪条臂 | 怎么做 | 该看到什么 | 看不到时 |
|---|---|---|---|---|
| **P0 原登记的阳性对照** | 四份装置 × 两个 `H` | 不改：各次原登记第五节「阳性对照与真实基线」那几格随产物照出，单测照跑 | 两个 `H` 下全绿；对应产物行两次逐字节相同 | 作废 V1 |
| **P1 这些单测看得见 67 这条通道** | 四份装置**各**一次 | 变异 M24（`H` → 345）与 M25（`H` → 288），见第九节 | 四份装置在两条变异下都有单测红（至少钉 67 的那条） | 某一份在某一条下全绿 ⇒ 那份装置的单测看不见头宽通道 ⇒ 作废 V2 |
| **P2 单测分得开 307 与 311** | 四份装置各一次 | 变异 M23（`H` → 307，改完之后的源码上跑） | 四份都被新加的余数单测（B2）抓到 | 没抓到 ⇒ V2 |

### 5.3 真实基线

真实基线是 `H = 311`（`crates/` 今天的头宽）。「四份全都逐字节相同」是正当结果，记「不变」。

### 5.4 实现量与分段

四份装置各改 1 行、加 1 条单测；四张变异表各加 3 条（第九节）；`replay.sh` 四行换产物文件名。编译四个二进制、单测四次、变异表八次（改前改后各一次；第 2 次那张 34 条最重）、产物四份。**一段做完**。E155 实验页里没有引 307 的现状句要改（回扫报告：第 120 行是历史事件句，不改）；产物出来之后实验页只在「历史版本」里加一条这一次的记录。

## 六、报哪些量与各自的判据

全部对应问题单第 3 行。每格各报各的判定，不合取。

| 格 | 怎么算 | 门槛 | 为什么不是同义反复 | 判定 | 问题单第 3 行：取什么值会让它翻面 |
|---|---|---|---|---|---|
| **Q155.1 一条记录装几项（四份各一行）** | 各份 `H = 311` 产物第一行 `name=config` 的 `named_items_per_record=` | = 67 | 67 出自 D23 已定项 4 / 17 的原句「67 个点名项」，不是从 `H` 的两个取值推出的；这一格是在产物上核「只有一条通道、且这条通道没动」 | = 67 ⇒ 通道没动；≠ 67 ⇒ 先走 F1 / S2 | `⌊(4096 − H) / 56⌋ ≠ 67`，即 `H ≤ 288` 或 `H ≥ 345` |
| **Q155.2 产物逐字节相同（四份各一行）** | `cmp` 每一份 `H = 311` 产物与同一份装置 `H = 307` 现跑的那份 | 逐字节相同 | 两份产物不同的可能来源不止 `H`（构建、依赖、浮点格式），相同与否要量 | 相同 ⇒ 那一次登记的全部判据判出同值，这一份记「不变」；不同 ⇒ 进 Q155.3 | 任一格的记录条数 `⌈点名项 / 容量⌉` 因容量变化而变 |
| **Q155.3 逐条重判（只在 Q155.2 不同的那一份上做）** | 把那一份两个产物 `diff` 出来的每一行，对到它那一次原登记第六节的判据格，按原门槛各判一次 | 各格原门槛 | 原登记写死的 | 原登记任一判据格两次判出的值不同 ⇒ 第 3 行记「变」，写明哪一次、哪一格；全部相同 ⇒ 记「不变（产物有与判据无关的差）」并交停机 S2 查差从哪来 | 同原登记各格的翻面条件 |

**够判点**：四份 `H = 311` 产物都出来、Q155.1 与 Q155.2 各判完。四份都「相同」⇒ 第 3 行够判，不做 Q155.3。

## 七、钉绝对值的断言

Q155.2 是「两次比」；两次一起错（例如常量改了、容量式子同时被改坏）靠下面的绝对值。

### 7.1 出自被测条款本身的（不符走第十节 F1）

| # | 断言 | 出处 |
|---|---|---|
| A1 | 一条 4096 的记录装 **67** 个点名项；头 311 之后「4096 上余 **3785**」 | D23 已定项 4「4096 上余 **3785**（67 个点名项，⌊(4096 − 311) ÷ 56⌋ = ⌊3785 / 56⌋）」；已定项 17「4096 记录 67 项」。四份装置原有的 `assert_eq!(NAMED_ITEMS_PER_RECORD_MAIN, 67, …)` 不改 |

### 7.2 独立算出、用命令核过的（第十三节命令二；不符先核我这边，核完仍不符 ⇒ 停机 S2）

| # | 断言 | 值 |
|---|---|---|
| B1 | `4096 − 311` | **3785** |
| B2 | 余数 `4096 − 311 − 67 × 56` | **33**（`H = 307` 时是 37）。四份装置各新加一条单测钉它——这是全部单测里唯一分得开 307 与 311 的一条 |
| B3 | 装 67 项的 `H` 区间 | **[289, 344]** |

原登记里钉死的其余绝对值（各次第七节）一个不改，两个 `H` 下都要绿。

## 八、轨迹与几何敏感性

### 8.1 轨迹

原登记要求报的轨迹（第 1 次 8.1、第 3 次 8.1 的不动点迭代、第 4 次 8.1 的 T1–T5 沿 k 轴）都在各自产物里，照旧随产物出；Q155.2 要求它们两次逐字节相同。这一次不新增被谓词消费的量。

### 8.2 几何敏感性（`H` 这个旋钮）

307 与 311 落在 67 那一段的同一侧，只比这两点看不出「产物对 `H` 敏感」这件事有没有被看住。敏感性不靠再出四份产物（第 2、4 次的产物各要跑完整个网格），靠两个方向相反的取样点上的单测：`H = 345`（容量 66，头变宽方向）与 `H = 288`（容量 68，头变窄方向），即第九节 M24、M25。四份装置在两个点上都要红（P1）。

**判别力自证**：钉 67 的那条单测，把期望值从 67 改成 66，`H = 311` 上必须红、`H = 345` 上必须绿；执行员在一份装置（第 1 次）上手做一次，记进第十二节修订，不入变异表。

## 九、变异

**先后**：改装置之前，四张表在今天的源码（`H = 307`）上各跑一遍 `bash research/scripts/mutate.sh <二进制名> <源文件> <变异表>`（二进制名见 `research/e7-index-bench/Cargo.toml:421-434`），留日志；改完之后再各跑一遍。每张表两次各报「抓到 / 无效 / 没红」三个数，逐条比（`mutation-sampling.md`「改了一个格式常量之后，要看「无效」那一栏有没有变多」）：原有条目任一条状态变差 ⇒ V4。第 1、2 次那两张表里的 M10（`NAMED_ITEMS_PER_RECORD_MAIN` 改成 71）锚点是用它那一行，不含头宽的值，改常量之后照样命中。

新加三条，四张表各加一遍（锚点是各份装置自己的定义行，唯一命中）：

| 条 | 改什么 | 在哪个取样点上改变输出 | 谁该红 |
|---|---|---|---|
| M23 头宽退回 307 | `const RECORD_HEADER_BYTES: u64 = 311;` → `307` | 余数 33 → 37；容量仍 67，产物不变 | B2 那条新单测（P2） |
| M24 头宽过上边界 | 同一行 → `345` | 容量 67 → 66（`name=config` 的 `named_items_per_record=`）；点名项数落在 `(66m, 67m]` 的格记录条数加 1（例：恰好 67 项的一批 1 → 2 条；第 4 次 k = 64 甲那一格 71 项，66 与 67 下都是 2 条，这一格**不**变） | 钉 67 的单测 |
| M25 头宽过下边界 | 同一行 → `288` | 容量 67 → 68；点名项数落在 `(67m, 68m]` 的格记录条数减 1（例：恰好 68 项的一批 2 → 1 条） | 钉 67 的单测 |


## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | **条款可能错**：装置确实读到了 311（M23 被抓），而 A1 不成立 ⇒ 不作废，如实记「D23 已定项 4 / 17 的 67 与头 311 的算术不符」，交主 agent | 某份 `H = 311` 产物的 `named_items_per_record=` ≠ 67，且查过是 `⌊3785 / 56⌋` 本身不等于 67（不会发生：第十三节命令二算过是 67；触发了就先核我这边） |
| F2 | **结论变了（正当结果）**：Q155.3 判出原登记某一格两次不同 ⇒ 第 3 行记「变」，写明哪一次、哪一格、两边各是什么；实验页照新值改写，不许回头改门槛 | 某份产物两次不同，且 `diff` 出来的行落在原登记某一判据格的输入上、按原门槛判出不同的值 |
| F3 | **全部不变（正当结果）**：四份产物两次都逐字节相同 ⇒ 第 3 行记「不变」；不许写成「装置没测出差别所以坏了」（装置看不看得见头宽由 P1、P2 单独判） | 四次 `cmp` 全部无输出、退出码 0 |

## 十一、作废条款与停机条款

### 作废

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| V1 | 原登记的阳性对照（P0）在任一份、任一个 `H` 上红 | 四份装置的 `cargo test` 任一失败，或对照那几行两次不同 |
| V2 | 单测看不见头宽通道，或分不开 307 与 311（P1、P2） | M24 或 M25 在某一份装置上「没红」；M23 在某一份上「没红」 |
| V4 | 改常量后原有变异有一条状态变差 | 同一张表两次 `mutate.sh` 日志逐条比，有一条从「抓到」变成「无效」或「没红」 |

（编号跳过 V3：这份登记没有敏感性输出行，第八节的敏感性由 V2 管。）

### 停机（两边都查，写进报告交回）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| S1 | 基线没立住 | 改装置之前 `replay.sh E155 E155R2 E155R3 E155R4` 任一份不逐字节一致 |
| S2 | 头宽之外还有别的东西让产物变了，或产物与我的算术对不上 | Q155.2 某一份不同，而 Q155.1 那份仍是 67（`H` 那条通道没动却有差）；或 B1–B3 与产物、单测对不上（核完我这边仍不符） |
| S3 | 装置与 `crates/` 对不上（`implementation-first.md` 第 4 条） | 开跑前现查 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_HEADER_BYTES` ≠ 311、`JOURNAL_NAMED_ENTRY_BYTES` ≠ 56，或 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 的单测钉的不是 67 |

⚠️ 不改名的代价：门禁 27 号只认 `format-const` 标记里的名字（`JOURNAL_HEADER_BYTES`），`RECORD_HEADER_BYTES` 不在它的扫描对象里，四份装置与 E159 的这个常量以后再漂，门禁不会报。这一次按派发提示只改值；要不要改名、连同变异锚点与 E159 的 S2 范围一起动，交主 agent 另定。同样另定的：第 3 次装置第 32–33 行说好「下次重跑时一起改」的输出键名 `system_configuration_slot_bytes`（第三节 3.2 末尾）。

### 够判停机

第六节「够判点」一到就停：四份都相同 ⇒ 第 3 行记「不变」交回，Q155.3 标「够判后未跑」；有不同 ⇒ 做完那一份的 Q155.3 再交回。没有第二段。

## 十二、修订

单测跑完（四份装置各 76/43/53/23 绿）、产物一次没跑之前，一处收严：B2 那条新单测最初写成减法
`assert_eq!(RECORD_BYTES - RECORD_HEADER_BYTES - NAMED_ITEMS_PER_RECORD_MAIN * NAMED_ITEM_BYTES, 33)`；
在四份装置各自第一次「改常量后」的 `mutate.sh` 跑（`e155_fsync_write_volume` 那份的读数：M10「一条记录容量改成 71」从「抓到」变成
「编译失败」，`⏭` 而非 `✅`）里撞见——`release` профиль开着 `overflow-checks = true`（`research/Cargo.toml:15`），
M10 把 `NAMED_ITEMS_PER_RECORD_MAIN` 改成 71 之后 `4096 − 311 − 71 × 56` 在编译期可求值且下溢，
命中 `.claude/singlefs-ai-sop/rules/test-discipline.md`「常量断言写加法，别写减法」那一条。
改成加法 `assert_eq!(RECORD_HEADER_BYTES + NAMED_ITEMS_PER_RECORD_MAIN * NAMED_ITEM_BYTES + 33, RECORD_BYTES)`
之后四份装置在 `H = 311`、`M10`、`M23`–`M25` 上全部编译通过并按预期变红，重新跑一遍改常量前后的两轮
`mutate.sh`（读数见报告）。四份装置一起改，判据、门槛、臂、M23–M25 的锚点与「在哪个取样点上改变输出」一个字未改，
只改了这一条新单测自己的写法，按 evidence-discipline.md「只许收严」——加法形态在运行期给出与减法形态相同的真值表
（两边都判定「等于 33」这件事），收严在于它在更多变异下都能活到运行期被判定，不会被编译器提前拦掉。
第八节的判别力自证不是靠再建一个单测挪门槛，是靠两个方向相反的取样点单测（M24/M25，第九节），已随变异表一并跑过，无另需手做的一步。


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

