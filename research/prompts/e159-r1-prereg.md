# E159 重跑登记（记录头 307 → 311）：fsync 等待时间装置的锚点与冒烟在头 311 下还判不判出原来的结论

写于 2026-09-24 21:45 JST，装置改之前、这一次的任何产物之前。原登记 `research/prompts/e159-preregistration.md`（工作区里就在）。E159 到今天只跑过「装置与冒烟」：`anchors` 子命令的确定性产物（进了 `replay.sh`）与真机冒烟（三条臂各一格、S3 探针，不进 `replay.sh`）；正式计时（原登记第五节 5.7 第一段的五轮主网格、PC1–PC3、AA、G8、G1、G6）没跑过。**这一次只重跑锚点与冒烟，计时段不在这一次**（派发提示与问题单第 5 行「计时段不在这一次（机器空闲时另跑）」）。

判据、门槛、作废条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**装置写在哪（写死）：`research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs`（独立手写装置，不与 `crates/` 共用代码），不是入库装置。** 与原登记第五节 5.1 一致（`crates/` 里没有组提交）。变异表 `research/mutations/e159_fsync_wait_group_commit.tsv`，复跑行 `research/scripts/replay.sh` 的 E159 那一行（`anchors` 子命令，二进制名 `e159-fsync-wait-group-commit`，`research/e7-index-bench/Cargo.toml:445-446`）。

文件名取 `r1`：派发提示点的；仓里（含 git 历史）没有任何 `e159-r*-prereg.md`。仓里另有「`r<n>` = 第 n 次跑」的用法（`e156-r2-prereg.md`），按那个用法这一次是第 2 次；要不要改由主 agent 定，正文不自引文件名。

## 一、问题

**主 agent 给的问题（逐字）**：记录头 `JOURNAL_HEADER_BYTES` 从 307 改成 311 之后，每个实验跑前写死的判据在 311 下还判不判出原来的结论。

**先核：装置里的 `RECORD_HEADER_BYTES` 是不是 `JOURNAL_HEADER_BYTES` 那个量——是。** 依据：它的文档注释逐字「记录头宽度（`journal_named_items_per_record` 用它反推容量）」（第 46 行）；它在装置里只出现在定义（第 47 行）与 `NAMED_ITEMS_PER_RECORD_MAIN = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES`（第 93 行）两处，`RECORD_BYTES = 4096`、`NAMED_ITEM_BYTES = 56`；`crates/` 的同一个式子是 `JOURNAL_NAMED_ENTRIES_PER_RECORD`（`crates/singlefs-format/src/lib.rs:172-173`），`crates/singlefs-core/src/journal.rs:184` 写完 `JOURNAL_HEADER_BYTES` 个字节才开始写点名项。⇒ 按派发提示「是才改」：改成 311。**只改值，不改名**：这份装置的写量函数是从 E155 第 4 次装置逐字拷来的，原登记停机条款 S2 要两边逐函数 diff 为空；E155 那边同一个常量这一次也只改值不改名（E155 第 5 次的重跑登记），两边保持同名。

**读法写死**：

| 名字 | 写死成 |
|---|---|
| 头宽 `H` | 装置第 47 行 `RECORD_HEADER_BYTES`。被判的取值 `H = 307`（今天的装置，改之前现跑当基线）与 `H = 311` |
| `H` 进装置的通道 | 只有 `NAMED_ITEMS_PER_RECORD_MAIN = ⌊(4096 − H) / 56⌋`：每批记录条数 `⌈W / 该值⌉`（甲）、`⌈实写脏叶数 / 该值⌉`（乙-M-K10）、checkpoint 的 `⌈写出单元数 / 该值⌉`（原登记 5.3「记录条数不用函数交回的期望，按这一批取整之后实际写出的单元、照各臂的点名规则现算」） |
| 「原来的结论」 | E159 没有计时结论。能比的只有两样：`anchors` 产物（原登记第七节 7.2 的 B1–B4 在产物里的值）与冒烟的「三条臂各一格真跑通、S3 探针判 512」 |
| 「还判不判出」 | `anchors` 产物两个 `H` 下逐字节相同 ⇒ 写量锚点判出同值；冒烟三格在 `H = 311` 下照样跑通、探针照样 512 ⇒ 冒烟的判定同值 |
| 计时 | 冒烟里装置照原登记第一节在来宾进程里用 `std::time::Instant` 计 W，结果行由来宾进程自己打出；**这一次不拿任何计时字段做判定**，冒烟的延迟数照录不判（原登记第十一节已写明冒烟不参与逐字节比对） |

**问题单（`research/prompts/m2-header311-rerun-questions.md`，表头与第 5 行，quote-kb 机械抄）**：

**出处 `research/prompts/m2-header311-rerun-questions.md:5-6`（整段抄，未转述）**

```markdown
| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
```

**出处 `research/prompts/m2-header311-rerun-questions.md:11-11`（整段抄，未转述）**

```markdown
| 5 | E159（fsync 等待时间随并发数：组提交与 wal 两臂） 在头 311 下结论变不变 | 不变 / 变 | 两臂的等待时间或写量关系翻面 | 装置常量改成 311，冒烟与锚点产物重跑；计时段不在这一次（机器空闲时另跑） | 开着 |
```

问题单第 5 行的翻面观测写「两臂的等待时间或写量关系翻面」：等待时间 E159 从没正式量过，这一次也不量，那一半没有「原来的结论」可翻；写量那一半由 `anchors` 产物回答。

## 二、被测条款与它引的定义

用 `research/scripts/quote-kb.py` 抄进草稿目录、回读逐字节一致之后追加。原登记第二节抄的 D23 已定项 1、D16 已定项 2 / 5 / 7 / 10 / 11、D25 已定项 7 与 E155 臂定义这一次不读 `H`，不重抄。

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
| 头宽 | `crates/singlefs-format/src/lib.rs:164-165`；`:303-315`；`:323` | 311 |
| 一条记录装几项 | `lib.rs:172-173`；`:325` 钉 67 | 67 |
| 一次发布几条记录 | `crates/singlefs-core/src/transaction.rs:2621`（点名项多于 67 就在动分配器之前拒掉） | 今天一次发布一条记录；装置的 `⌈W / 67⌉` 是模型给的（原登记 5.3「已知与 `crates/` 不同的地方」第二样） |
| 锚点格对拍的测试表 | 原登记停机条款 S1 列的 `transaction.rs`、`recovery.rs` 与测试表行号（2026-09-24 09:34 JST 那一刻的行号） | 这一次不重查这些行号：锚点格（甲、F1、P = 1、k = 1）一批 8 个单元、1 条记录，记录条数在 66 / 67 / 68 任何一个容量下都是 1，不读 `H` |
| 记录标志 1 字节 | `crates/singlefs-core/src/journal.rs:151` 今天写 0 | 装置把记录当 4096 字节伪随机内容写（原登记 5.3），不建头里的逐字节布局 |

### 3.2 装置里读 `H` 的每一处

| 行 | 是什么 | 这一次怎么动 |
|---|---|---|
| 47 | `const RECORD_HEADER_BYTES: u64 = 307;` | 改 311 |
| 93 | `NAMED_ITEMS_PER_RECORD_MAIN` 的式子 | 不动 |
| 181 | 单测 `assert_eq!(NAMED_ITEMS_PER_RECORD_MAIN, 67, …)` | 不动；另加一条钉余数 33 的单测（第七节 B2） |

`grep -n 'RECORD_HEADER_BYTES\|307'` 在装置里只命中 47 一行带 307，`RECORD_HEADER_BYTES` 命中 47、93 两行。冒烟用的宿主脚本：原登记 5.6 写的 `research/scripts/e159-run.sh` 仓里没有（`ls research/scripts | grep e159` 零命中）；上一次冒烟的复跑命令，实验页第 46 行写明存在 2026-09-24 冒烟产物的文件头里（我没读那份产物），执行员照那几条命令跑。

## 四、跑之前已经存在的数

**写量那一半的答案在跑之前就算得出来**：`⌊(4096 − 311) / 56⌋ = 67`，与 307 时相同（第十三节命令二），`H` 进装置只有这一条通道。按装置结构，`anchors` 产物应当逐字节相同。照纪律不删、不回改判据；重跑的用处：装置、单测、变异、产物跟着格式常量一起动；原登记停机条款 S2（与 E155 第 4 次装置逐函数 diff）在两边都改之后仍要为空；冒烟确认改完的装置在真机上照样跑通。

| 数 | 出处 | 对判据的影响 |
|---|---|---|
| 67；余数 37 / 33；装 67 项的 `H` 区间 [289, 344] | 第十三节命令二 | 第七节锚点；第八节敏感性点取在 288 / 345 |
| 原登记第七节 B1–B4：118 784 字节 / 6 次、86 016 / 4 次、365 345.862 / 4 313 361.603 / 316 188.444 / 4 264 111.306、写调用期望 14.048942 / 134.952322 / 11.048611 / 131.946322、172 544 字节 / 11 次、多 70 144、两盘 139 776 | 读原登记判据怎么定的（第 501–534 行） | 是原登记跑前算的锚点；这一次不改它们，`anchors` 产物逐字节复现即可 |
| 原登记第一节「读法写死」：M = 2、倍数档 1.25 / 1.5 / 3 / 5 / 10、k 取样点、本机口径 | 同上（第 1–46 行） | 计时判据，这一次不用 |
| 实验页标题「装置已写、计时未跑 … 54 单测 / 14 条变异全抓」；实验页第 5、14、15、32、46、61、62、64、68、73 行（一次 `grep -n 'smoke\|冒烟\|anchors'` 的命中）：冒烟撞见 `read_at` 的 O_DIRECT 未对齐 bug 已修、三条臂各跑通一格（甲 k = 1、wal_full-K10 k = 2 duration 4 s checkpoint 0 次、乙-M-K10 k = 2 duration 6 s checkpoint 1 次）、「甲 k = 4、duration 3 s 约写 58.5 MB，折合约 17.7 MB/s」、写量预算没算清、「测量之前整盘写一遍」没实现、PC 系列没正式跑 | 为找冒烟与锚点怎么跑而 grep 实验页 | 是上一次冒烟的经过与读数。第六节冒烟那一格的判据只看「跑通、行齐、探针 512」，不看任何延迟或写量读数；「整盘写一遍没实现」是原登记 5.3 的欠账，不在这一次 |

## 五、臂、阳性对照、真实基线

### 5.1 臂

- **原登记的三条臂照旧**（甲、wal_full-K10、乙-M-K10，原登记第五节 5.2），一个字不改。
- **这一次只加一个旋钮 `H`，两个被判的取值**：`H = 307`（装置一个字节不改，`bash research/scripts/replay.sh E159` 现跑 `anchors`；做完会怎样——与留存产物逐字节一致，S1 核）；`H = 311`（只改第 47 行、加一条单测，重出 `anchors` 产物、重跑冒烟；做完会怎样——`anchors` 产物与 `H = 307` 那份逐字节相同，冒烟三格跑通。前半句由命令二推出、由 Q159.2 核；后半句由 Q159.3 核）。
- 冒烟**只在 `H = 311` 上跑**：冒烟判的是「跑不跑得通」，不是两个 `H` 之间的比较；`H = 307` 那一侧的冒烟已在 2026-09-24 跑过（实验页），这一次不为比较而在旧常量上再占一次机器。

### 5.2 阳性对照（每一条臂都跑）

| 对照 | 对哪条臂 | 怎么做 | 该看到什么 | 看不到时 |
|---|---|---|---|---|
| **P0 锚点格与写量锚点** | 三条臂 × 两个 `H` | 不改：原登记 B1–B4 在 `anchors` 产物里照出，54 条单测照跑 | 两个 `H` 下全绿；`anchors` 产物两次逐字节相同 | 作废 V1 |
| **P1 单测看得见 67 这条通道** | 装置一次（通道三臂共用） | 变异 M16（`H` → 345）、M17（`H` → 288），第九节 | 钉 67 的单测红 | V2 |
| **P2 单测分得开 307 与 311** | 同上 | 变异 M15（`H` → 307） | 新加的余数单测红 | V2 |
| **P3 冒烟每条臂都跑** | 甲、wal_full-K10、乙-M-K10 各一格 | 照上一次冒烟的三条命令（第三节 3.2 末尾） | 三格都打出完整结果行与 `name=done`，`emitted=` 等于实际行数 | 缺一格 ⇒ 那条臂没过闸，停机 S4 |

### 5.3 真实基线

真实基线是甲（原登记第五节 5.4），头宽取 `H = 311`（`crates/` 今天的值）。

### 5.4 实现量与分段

改 1 行、加 1 条单测；变异表加 3 条；`replay.sh` 的 E159 一行换产物文件名。编译一次、单测一次、变异表两次（改前改后）、`anchors` 一次、S2 的逐函数 diff 一次、冒烟：S3 探针一次 + 三条臂各一格（要起虚机，开跑前按共用约束看负载）。**一段做完**，计时段不在这一次。

## 六、报哪些量与各自的判据

全部对应问题单第 5 行。每格各报各的判定，不合取。

| 格 | 怎么算 | 门槛 | 为什么不是同义反复 | 判定 | 问题单第 5 行：取什么值会让它翻面 |
|---|---|---|---|---|---|
| **Q159.1 一条记录装几项** | `H = 311` 编出的二进制上，钉 `NAMED_ITEMS_PER_RECORD_MAIN == 67` 的单测（第 181 行）过不过 | = 67 | 67 出自 D23 已定项 4 / 17 原句 | 过 ⇒ 通道没动 | `H ≤ 288` 或 `H ≥ 345` |
| **Q159.2 写量锚点产物逐字节相同** | `cmp` `H = 311` 的 `anchors` 产物与 `H = 307` 现跑那份 | 逐字节相同 | 两份不同的来源不止 `H`，要量 | 相同 ⇒ 写量那一半「不变」；不同 ⇒ 逐行对到原登记 B1–B4，按原断言判，同时停机 S2 | 记录条数随容量变的那几格 |
| **Q159.3 冒烟跑通** | `H = 311` 上 S3 探针一次、三条臂各一格 | 探针报 `logical_block_size=512`（原登记 B9）；每格无 panic、结果行齐、`emitted=` 对得上 | 这三样是「装置在真设备上能不能跑」，不从 `H` 推出 | 三样都满足 ⇒ 冒烟「不变」（照样跑通） | 改常量让装置在真设备上出错（按通道分析不会发生；发生即停机 S4） |
| **Q159.4 与 E155 第 4 次装置的逐函数 diff（原登记 S2）** | 两边都改完常量之后，照原登记 S2 的做法把拷过来的函数逐个 diff | diff 为空 | 两边各改各的常量行，函数体不该动；有没有动要量 | 空 ⇒ 过；不空 ⇒ 停机（原登记 S2） | — |

冒烟里的延迟、吞吐、写量读数：只报数，不判（第一节「读法写死」最后一行）。

**够判点**：Q159.1–Q159.4 都判完 ⇒ 第 5 行（这一次的范围：写量那一半与冒烟）够判。等待时间那一半不在这一次，照问题单第 5 行留给计时段。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符走原登记第十节 F1「条款可能错」）

| # | 断言 | 出处 |
|---|---|---|
| A2 | 一条 4096 的记录装 **67** 个点名项；头 311 之后 4096 上余 **3785** | D23 已定项 4、已定项 17（第二节 2.1）。取下一个未用号，免得与原登记 7.1 的 A1（`T_time` 等）撞号 |

### 7.2 独立算出、用命令核过的（第十三节命令二；不符先核我这边，核完仍不符 ⇒ 原登记 V8 作废）

| # | 断言 | 值 |
|---|---|---|
| B5 | 余数 `4096 − 311 − 67 × 56` | **33**（307 时 37）。新加一条单测钉它，是全部单测里唯一分得开 307 与 311 的一条 |
| B1–B4（原） | 原登记第七节 7.2 | 不改。它们经 67 这条通道读 `H`（记录条数），67 不变它们就不变；变没变由 Q159.2 在产物上核，不靠这里的推理 |

## 八、轨迹与几何敏感性

### 8.1 轨迹

这一次不跑计时，原登记 8.1 要求的轨迹量（W 的分位数随 k）都在计时段里，不适用。

### 8.2 几何敏感性（`H` 这个旋钮）

307 与 311 落在 67 那一段同一侧。敏感性靠两个方向相反的取样点上的单测：`H = 345`（容量 66）与 `H = 288`（容量 68），即第九节 M16、M17。

**判别力自证**：钉 67 的那条单测，把期望值改成 66，`H = 311` 上必须红、`H = 345` 上必须绿；执行员手做一次，记进第十二节修订。

## 九、变异

**先后**：改装置之前在今天的源码（`H = 307`）上跑 `bash research/scripts/mutate.sh e159-fsync-wait-group-commit research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs research/mutations/e159_fsync_wait_group_commit.tsv`，留日志；改完再跑一遍。两次各报「抓到 / 无效 / 没红」三个数，逐条比：原有 14 条（M1–M14）任一条状态变差 ⇒ V4。原有 14 条的锚点都不含头宽。

新加三条：

| 条 | 改什么 | 在哪个取样点上改变输出 | 谁该红 |
|---|---|---|---|
| M15 头宽退回 307 | `const RECORD_HEADER_BYTES: u64 = 311;` → `307` | 余数 33 → 37；`anchors` 产物不变 | B5 的单测（P2） |
| M16 头宽过上边界 | 同一行 → `345` | 容量 67 → 66；点名项数落在 `(66m, 67m]` 的批记录条数加 1 | 第 181 行钉 67 的单测 |
| M17 头宽过下边界 | 同一行 → `288` | 容量 67 → 68；点名项数落在 `(67m, 68m]` 的批记录条数减 1 | 第 181 行 |

## 十、失败条款

原登记第十节里与计时有关的条款这一次不触发（没有计时段）。这一次另加：

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F-a | **写量锚点变了（正当结果）**：Q159.2 不同，且逐行对到原登记 B1–B4 判出不同 ⇒ 第 5 行写量那一半记「变」，写明哪一格 | `anchors` 两份产物的 `diff` 落在 `label=` 为 B1–B4 那几行上，值不同 |
| F-b | **全部不变（正当结果）** ⇒ 第 5 行这一次的范围记「不变」 | `anchors` 两份 `cmp` 无输出；Q159.3 三样都满足；Q159.4 diff 为空 |

## 十一、作废条款与停机条款

### 作废

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| V1 | P0：任一个 `H` 上单测红，或 `anchors` 里 B1–B4 与原登记不符 | `cargo test` 失败；`anchors` 产物的 B1–B4 行与原登记第七节 7.2 不同 |
| V2 | 单测看不见头宽通道，或分不开 307 与 311 | M16 或 M17「没红」；M15「没红」 |
| V4 | 改常量后原有变异有一条状态变差 | 两次 `mutate.sh` 日志逐条比 |

### 停机（两边都查，写进报告交回）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| S1 | 基线没立住 | 改装置之前 `replay.sh E159` 不逐字节一致 |
| S2 | 头宽之外还有别的东西让 `anchors` 变了 | Q159.2 不同而 Q159.1 仍过 |
| S3 | 装置与 `crates/` 对不上（`implementation-first.md` 第 4 条） | 开跑前现查 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_HEADER_BYTES` ≠ 311 或 `JOURNAL_NAMED_ENTRY_BYTES` ≠ 56 |
| S4 | 冒烟没跑通 | 任一格 panic、结果行缺、`emitted=` 对不上，或探针不是 512（原登记 S3） |
| S5 | 机器在忙 | 起虚机之前 `ps -o pid,args -u "$(id -u)"` 看到 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark` 或 `fio`：报「在等 pid …」停下，冒烟不在别的测量旁边跑（共用约束「不做」一节） |
| — | 原登记 S2（逐函数 diff） | 即 Q159.4 |

### 够判停机

第六节「够判点」一到就停，第 5 行这一次的范围记「变」或「不变」交回。计时段（原登记 5.7 第一段起）不因这一次而开跑，由主 agent 在机器空闲时另派。

## 十二、修订

单测跑完（54/54 绿，改常量之前的基线）、产物一次没跑之前，一条收严：第七节 A2、B5 那条新单测按 E155 第五次重跑登记同一节撞见的坑（`research/prompts/e155-r5-prereg.md` 第十二节）预先写成加法
`RECORD_HEADER_BYTES + NAMED_ITEMS_PER_RECORD_MAIN * NAMED_ITEM_BYTES + 33 == RECORD_BYTES`，不写减法，理由同 E155：`research/Cargo.toml:15` 的 `overflow-checks = true` 会让减法形态在 M16（头宽过上边界）那类变异下编译期溢出，被 `mutate.sh` 记成「无效」而不是「抓到」。判据、门槛、臂一个字未改。

第二条（主 agent 转达门禁 12 号红，用户点名）：本文里两个带撇号的角标名改成各自那一族的下一个未用号——原写成「A1 加撇号」的那一项改成 `A2`（与原登记 7.1 的 A1 撞号才加的角标，原登记 7.1 只有 A1，A2 未用）、原写成「B2 加撇号」的那一项改成 `B5`（原登记 7.2 已占 B1–B4），出现在第 186、192、215 行。`bash .claude/gate.d/12-no-prime-marks.sh` 对这三处已转绿。不改判据、不改门槛。


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

