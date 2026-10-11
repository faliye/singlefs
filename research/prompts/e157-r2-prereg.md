# E157 重跑登记（第 2 次，差分）：点名项 67 → 64 之后，并行线一两条条款的计数模型第一段的产物与正文引的数变不变

写于 2026-10-04，装置改之前、这一次的任何产物之前。**差分重跑**：臂、判据、门槛、作废与停机条款照上一次登记 `research/prompts/e157-r1-prereg.md`（sha256 `0d02d18083a9c5f5d295f29ce146605508c8d09abe977f1f8e5c7e84d1696c33`，第 1 次重跑，307 → 311）与它引的原登记 `research/prompts/e157-preregistration.md`。这里只写变了哪些输入、哪几格要重判、贴着旧值的锚点与变异怎么换；没写到的内容「同上一次登记」。只重跑第一段：第二段（C491）从未跑过、条款已由 D23 已定项 17 定案（同上一次登记文件头）。跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走。

**装置写在哪（写死）：`scripts/experiments/e157_parallel_line_one_clauses.rs`（独立手写计数模型，不与 `crates/` 共用代码），不是入库装置。** 变异表 `scripts/mutation/tables/e157_parallel_line_one_clauses.tsv`，复跑行 `scripts/experiments/replay.sh:193`（`E157|e157-parallel-line-one-clauses||e157-parallel-line-one-clauses-2026-09-25-h311-replay.out|exact`）。

**准入**：`python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs E157` 退 2（准入登记里没有实验键 E157），不是 77。

## 一、问题

英文名：parallel_line_one_clauses（照抄装置文件名；上一次登记没有这一行）

**问题单（`research/prompts/m3-r1-batch9-reruns-questions.md`，sha256 `d6b328eb6beebc0073200971a8dc3971db458571458748d6bd7b4f5c497616e7`，表头与第 4 行，quote-kb 机械抄）**：

**出处 `research/prompts/m3-r1-batch9-reruns-questions.md:12-13`（整段抄，未转述）**

```markdown
| # | 实验 | 问题 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
```

**出处 `research/prompts/m3-r1-batch9-reruns-questions.md:17-17`（整段抄，未转述）**

```markdown
| 4 | E116、E43、E49、E16、E157 | `scripts/experiments/` 下 e116、e157、e16、e43、e49 的源码里还有 311 / 188 / 67 旧值（删除与并发那条线跑 format-constants 的输出 `/tmp/claude-1000/m3-writeback2/format-constants.out`）：换成今天的值之后，各自的产物与正文引的数是否变 | 产物与 `scripts/experiments/replay.sh` 登记的 exact 产物不再逐字相同 | 每个都改完、复跑、与登记产物比过，变了的出新产物并回对正文 | 开着 |
```

这份登记只回答第 4 行里的 E157。每个量对应第 4 行的翻面观测（产物与登记的 exact 产物不再逐字相同）或够判条件（改完、复跑、比过、变了的出新产物并回对正文）。

**读法写死**（只有一种读法）：

| 名字 | 写死成 |
|---|---|
| 变了的输入 | `crates/singlefs-format/src/lib.rs:223` 头 311 → 487、`:229` 点名项 67 → 64（`:220` 新根段 188 → 364 只经头宽进来）。装置读它们的通道：第 70 行头宽、第 74 行每条点名项数、第 75 行编译期断言两者一致、第 311 行锚点行的独立复算 |
| `H` 的两个被比取值 | `H = 311`：`replay.sh:193` 登记的 exact 产物 `research/results/e157-parallel-line-one-clauses-2026-09-25-h311-replay.out`，改装置之前现跑复现（S1）；`H = 487` |
| 「产物变不变」 | 新产物与登记产物 `cmp`；不同就按 Q157.4 的允许集合分类 |
| 「原来的结论」 | 上一次登记第六节、原登记第六·一节第一段各判定格在登记产物上判出的值 |
| 第一段读不读点名项数 | 不读（上一次登记「读法写死」第二行，原样成立）：1.1a–1.3c 走 extent 树几何与 key 编码。⇒ 单元数取样点里的 67 / 68（装置第 271 行，原登记第八·二节「一条记录 67 个点名项（67/68）」）是为**第二段**的量 2.x 选的，第一段没有一格消费它们贴不贴着点名项上限；第二段不跑。所以这两个点**不换成 64 / 65**，第 271 行一字不动，这是写死的选择（换了只会多出与判定无关的产物行） |

## 二、被测条款与它引的定义

上一次登记 2.1 抄的是 311 那一版的 D23 已定项 4 / 17。这一次用 `scripts/kb/quote-kb.py` 重抄：D23 已定项 4（被测条款），与实验页 `### 影响的决策` 表点名分项的每一条：D23 已定项 12、17，D8 已定项 3、11。表里的 D9（加密）没有分项，不抄。产物 `/tmp/claude-1000/m3-r1/batch9-row4-design/e157-q.md`（5 段，回读逐字节一致），整份追加如下。原登记第二节抄过的其余条款（D16、数据这一侧、extent 树）不读点名项数，不重抄。

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:148-177`（整段抄，未转述）**

```markdown
#### 已定项 4：记录头约束在单个原子单元内，头 487 字节

**定案**：**记录头完整落在一个原子单元内**（[invariants.md](../invariants.md) I-8.2（记录头不跨原子单元））。头 **487 字节**：

- **十个字段 78 字节**：magic 4 + 类型 2 + 算法类型 1 + 记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏） + 自述长度 4 + 点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32。`jsn` 的 10 字节 = 实例代号 32 位 + 计数器 48 位（已定项 9）。`tail_lsn` 不在头里：已定项 3 逐字定了「tail 的权威副本住在一个固定位置，**不内联在记录头**」，而「内联在每条记录头的 `tail_lsn`」正是被否掉的那个 XFS 形态的定义。
- **三笔已定增量 17 字节，不在那十个字段的 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节**、已定项 13 的载荷校验和 **4 字节**。
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 463、MAC 471、头末 487。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到这四格，都当那条记录损坏、断链即止，那次发布整体不施加：序号 0；一次发布之内跳号；锚点读得出时，下一次发布的首条序号不是 1（断在这一条）；同一 (实例代号, checkpoint_txg) 里带末条标志的那条之后还有记录（断在带标志的那条；「之后」按计数器读、不要求紧接着：计数器大于带标志那条的、读得出的同 (实例代号, checkpoint_txg) 记录都算，中间缺号不停；前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减，有这个前提时与「表里任何位置」的全表读法给出同样的施加结果）。与 I-8.9（一次发布的记录序号连续且只有末条带标志） 判的是同一件事。
- **已定项 15 的新根段 364 字节**（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4）。
- **fsid 8 与 MAC 16，都排在新根段之后**：
  - **fsid 8**（系统配置 fsid 的低 8 字节，与单元头那 8 字节同口径，D18（块里携带什么信息） 已定项 7）。不带的话 I-1.4（块头 fsid 一致） 要给 journal 记录写一条豁免，而全环扫描就再也挡不住「上一个文件系统留在这块盘上的记录」——mkfs 整环写 0 只在 mkfs 真跑完时成立，重 mkfs 中途崩掉就不成立。带 8 字节之后不用豁免。
  - **MAC 16**（放在 fsid 之后；nonce 12 与算法类型 1 十个字段里已有）。理由与根记录那 29 字节同一条（D22（单元原子性怎么合成） 已定项 7）：journal 记录头是自证结构，没有父指针带 MAC ⇒ 不 day-1 留位，开加密那天要么整条走明文豁免（泄漏 `checkpoint_txg`、新根段里的两条指针、点名项位置），要么就是一次布局变更；D9（加密） 已定项 3 的原则逐字是「为加密留的位所有卷都留、不挪作他用」。I-6（加密不变量）不列它。
- 登记值就是 487（`JOURNAL_HEADER_BYTES`）；十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`。**说 78 时要写明指的是十个字段，连同这一句一起引。** 487 占 512 扇区的 95%，其后还余 **25 字节**（装不下一个点名项，每项 56 字节）；4096 上余 **3609**（64 个点名项，⌊(4096 − 487) ÷ 56⌋ = ⌊3609 / 56⌋）。
   <!-- format-const: JOURNAL_HEADER_BYTES = 487 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋|头 **311 字节**|余 **201 字节**|JOURNAL_HEADER_BYTES = 311|512 上余 201 -->
   <!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **那个原子宽度不由本工程定**：D20（承重面：单元的原子性与自包含）已定自证单元依赖**运行时探测**的 `physical_block_size`，不许把 512 焊进格式。⚠️ **这个值是格式常量，改它要三处一起动**：kb 标记与正文、实验源码的 `const` 与钉死它的单测、重跑实验并更新产物；由门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）判红。记录尺寸取 4 KiB 定长之后对齐浪费恒为 0（已定项 12）；变长记录的对齐代价表只对变长记录成立。

**依据**：

- E23（journal 几何）：十个字段（跑时还带 `tail_lsn`）的头装得进一个 512 单元，不对齐、多条记录挤同一个原子单元时除第一条外都判不了撕裂——头要完整落在一个原子单元内。
- E75（记录尺寸与环几何）：各个头部读法在 512 与 4096 两档 `physical_block_size` 上都装得进一个单元；定长 2 的幂尺寸下头永远不跨单元。
- fsid 8 与 MAC 16：用户弹窗定案，没有走三方论证，可推翻，原话在变更史。
- 本次发布内序号 4 字节：用户 2026-09-23 定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 的 Z6），是推的、被攻过零轮，原话在变更史。
- 用户定案 2026-09-24：头 311 落地、末条标志放在原「填充 1」那个字节、序号的读者规则（原话在变更史）。
- 用户定案 2026-09-24：锚点读得出时首条序号不是 1、末条标志之后还有同 txg 的记录，两格都当损坏断链（实二十报告 `research/prompts/m2-lastflag-implementer-report.md` 第六节 2、3），原话在变更史；无实验：两格都要 CRC-32C 碰巧对上或改出来的镜像才走得到。
- 第四格的「之后」按计数器读、缺号不停，前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减：三方判决 `research/prompts/m3-formal-r2-main-verification.md` 第二节第 8 行与 `research/prompts/m3-formal-r3-main-verification.md` 第二节第四格补句一行，两条腿同向；前提另立不变量的判决是 `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行；用户定案（等前提判完一起写回），原话在变更史；无实验：补的是读者规则的射程，没有新的实验读数。
- 新根段加实例表指针、分配记录树根指针与卸载记号三项（头 311 → 487、点名项 67 → 64）：三方判决 `research/prompts/m3-ring-r1-main-verification.md` 第二节第 2 行（今天的施加法在写行、首个文件、卸载三种发布上补出错根，推的）与第四节第 1 条，`research/prompts/m3-ring-r2-main-verification.md` 第二节第 2 行（首个文件、卸载两种发布补进去之后与当初会写的根逐字节相同、checker 绿，攻方在副本上量的），`research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行（两位放进记录标志字节的形态被专用清扫发布打穿；施加处不从新根段来的只有卸载记号、实例表指针、分配记录树根三项）。用户定案「回到甲」（原话在变更史）。卸载记号占 4 字节是第一轮正推腿 H1-b 的算法（86 + 86 + 4），判决没有单独定宽。无实验：补的是施加处的字节，数都在三方副本上量过，没立实验号。

**欠**：C94（登记的格式常量与后来的定案对不上）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:313-330`（整段抄，未转述）**

```markdown
#### 已定项 12：记录尺寸取 4 KiB

**定案**：**记录尺寸定为 4 KiB，定长，写进系统配置**（XFS 把同一个参数写进系统配置的 `h_size`）。

- 与已定项 4 相容：记录头 487 字节要完整落在一个 `physical_block_size` 内，而 4096 被 512 与 4096 两种探测值都整除。
- 定长记录取 2 的幂尺寸之后，I-8.2（记录头不跨原子单元） 的对齐浪费**恒为 0**：记录背靠背摆下去，起始偏移恒是单元的整数倍。
- 4 KiB 记录装 64 个点名项 <!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 64 stale== 67 项 -->（已定项 4）；再大只抬高 I-8.1（环几何够大） 里那个「最坏事务占用」，进而抬高环大小下界，买不到别的。
- **用户口径**：先按这个数落地，写代码时再调整。

**射程**：⚠️ 余量那一格按的是「一次 fsync 一个事务」的读法（目标负载 12 项事务恰占 1 条记录），与 D16（发布语义） 已定项 5 末段的切分纪律（一个事务最多写一个单元的用户数据）不在同一口径：按切分纪律加已定项 7 一条记录一个事务号，seq 批 1 是 8 条记录、每条只装 1 到 5 个点名项，「12 项恰占 1 条」那一格不存在；余量口径与环几何怎么改归 C310（事务切分纪律与记录数口径打架），要用户定。目标负载（D25（目标负载优先级））若长大一个量级，这个数要重推，那时另立实验。环回绕与「装成一条要先在内存里缓冲整条记录」两条代价没建模。

**依据**：

- E75（记录尺寸与环几何）：4 KiB 上目标负载恰占一条记录、跨记录数 0，几个头部读法给的容量相同；环下界与另一个二进制已发表的那一格逐格相等；定长 2 的幂尺寸下对齐浪费恒 0。
- 用户定案，原话在变更史。

**欠**：C310（事务切分纪律与记录数口径打架）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:480-499`（整段抄，未转述）**

```markdown
#### 已定项 17：点名项 56 字节的字段表

**定案**：**一项一个单元（新池新建文件 12 项）：位置条目 14 × 2（偏移 0）+ 单元类型标签 1（28）+ 出生树 8（29）+ 出生 txg 8（37）+ key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）+ flags 1（55）= 56 <!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->；不另带载荷 CRC（位置条目各带 4 字节校验和）。重放从点名项直接凑出映射 key。**

- **记录点名的每一项自带校验和**：D4（校验和位置） 已定校验和内联进父指针，而记录点名的东西在记录持久时它的父还没发出 ⇒ 记录临时充当父，否则重放时那些单元没有任何完整性凭据；恢复重放施加前必须逐项验证点名单元（已定项 14 第四条）。
- **一次发布切成 N 个事务、N 条记录时，这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、记账树、中央映射树、树表单元）只在最后一条记录里点名**，前面几条不重复点名它们。
- **共享的提交内生块多于最后一条记录装得下（4096 记录 64 项）时，末条再跨记录**：从最后一个事务那条记录起依次多写几条，装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志；恢复按这个标志认一次发布的边界，一次发布的记录里没读到带标志的那条，整次不施加（已定项 14 第五条）。标志放在记录头十个字段里「填充 1」那个字节（改名「记录标志 1」），位 0 = 本次发布末条；每一次只有一条记录的发布（含空发布记录），那一条的位 0 也写 1——改新池新建文件的字节（w1、w4、t13 三条记录的这一字节与头部校验和 [46, 50)；w4、t13 两条的反向链 [91, 95) 也跟着变，E142（新池新建文件的干跑） 空白 G26 逐字节核过）。用户 2026-09-24 定。

**射程**：管点名项的字节；位置条目的构成归 D19（块指针的结构与宽度预算） 已定项 4 / 11，映射 key 的形态归 D19（块指针的结构与宽度预算） 已定项 6 / 10。⚠️ 共享的提交内生块只在最后一条点名时，崩在最后一条持久之前，那些单元没有任何记录点名——重放的回收路径要认得出这一格，层 0 的小负载要覆盖它（C491（多条记录时共享内生块在哪条点名没定））。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：记录点名的东西其父此刻还没写到盘上，记录必须临时充当父——逼出「点名的每一项自带校验和」。
- E77（发布的持久顺序）：点名项自带校验和、施加前逐项验证，是单元与记录之间那道屏障得以省掉的交换条件。
- 用户定案，原话在变更史。
- 用户定案 2026-09-23：共享的提交内生块只在最后一条记录里点名，原话在变更史。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6（维持拒绝时拒不拒取决于池的历史），按今天的字面认末条会施加半次发布，带标志位那一格 0 / 0）。

**欠**：无。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:57-87`（整段抄，未转述）**

```markdown
#### 已定项 3：key 布局 —— 改名与索引分开处理

**定案**：**key = `(locality_id, inode, offset)`**，`locality_id` 在创建时从父目录继承，**改名时故意不更新**。offset 段是**文件字节偏移**（这个单元第一个字节在文件里的偏移，第 n 个单元从 0 起是 n × 净荷容量 32634）。

- 命名走 dirent 树（`parent, name` → inode），改名就是删一条加一条，**O(1)**
- 数据按 inode 编 key，与名字彻底解耦
- scan 局部性来自继承的 `locality_id`，改名后**局部性缓慢退化，正确性不受影响**
- 退化由 D3（空间分配） 已承诺的常驻后台整理修复，不是新增负担

**批量改名**：`mv 大目录 别处` 在这个设计下**只改一条 dirent**。三处会把它重新变成无界操作，必须一开始就堵死：

1. **`locality_id` 永远不许被改名更新。** 它是**提示，不是正确性的一部分**——可以是错的、可以过期。
2. **改名覆盖已存在的目标 = 一次删除**，必须走已定项 5（whiteout 删除）的 intent log。
3. **后台重新聚簇必须是增量、可中断、可续做的**，且随时可以停掉不做。

**推论（可当 review 判据用）**：**任何让 rename 触及后代的设计改动，一律否决。**

**inode 记录必须存一份 `locality_id`**：它既是数据 key 的首段、又永不随改名更新，而命名树的 value 只有 inode、数据单元头的五元组里也没有它，所以盘上只有 key 自己带着它——读者打开一个改过名的文件时必须有地方拿到它，否则改名之后数据只能靠扫描才找得到。落点是已定项 6（inode 记录的盘上字段表）；这个值按本项从父目录继承，而第一版没有目录，所以「没有父目录时取什么」也要一起定——第一版取 0，登记在已定项 6 的记录字段表。扫描重建时重编出来的 key 首段按 inode 记录 > 幸存 extent key > 0 给这个对象的全部 key 编同一个首段（2026-09-16 用户定案，D18（块里携带什么信息） 已定项 3 那条加字段的提议不收）。

**射程**：它管数据 key 怎么编与改名怎么不触及后代，不管命名树自己的 key（dirent 树按 `(parent_inode, name)` 编，见已定项 5）。scan 局部性不一定要住在 key 序里：可以住 key 序（全路径索引，改名是 O(子树深度)）、住分配器（把子目录的数据放到父目录附近）、住第二索引（仍然贵，且要多维护一份一致性），本工程取「出生时继承、改名不更新的 locality 前缀」（较好，改名免费）。⚠️ 「它是提示、可以是错的」准许的是**不信它的含义**，不是**不存它的位**：作为地址首段它的取值精确且定义性。⚠️ **E9（key 编码对遍历局部性的影响） 的遍历局部性数只在一个口径下成立，引用必须连口径一起引**：**交错创建**（`interleave`，老化的代理）**加上 `cache_leaves = 8`**；`locality_id` 的收益是**新树的收益**，不是稳态收益，改名强度的现实性未校准——曲线可用，绝对刻度不可用。⚠️ 全路径索引的遍历优势也量出来了，但**不许拿它去替 D9（加密） 泄漏面与 D1（数据可移动性 / 反向索引） 稳定性做判断**——E9（key 编码对遍历局部性的影响） 对那两样一个字没说。⚠️ 「每次改名平均重写约 2700 个 key」这个数**缺口径**（查不到是哪一档改名、哪个种子），标为未核实；同表里 ext4 Orlov 分配器「遍历内核树快 30%」也是未核实——kb 内无出处无口径，要用先补。⚠️ 三段 key 在 D6（快照实现模型） 取多可写头之后曾经不完整（两个可写头各写一次同一个 `(locality_id, inode, offset)` 会落在同一个 key 槽、后写覆盖先写），2026-08-30 D6（快照实现模型） 已定项 1 取「每头一棵自己的树」之后消解——每头一棵树之下三段 key 在树内唯一，**两个头不再落进同一个 key 槽**。⚠️ **nonce 那条义务仍在**：D9（加密） 已定项 4 已定 nonce 每 extent 存，**生成时仍必须保证两个可写头之间不撞**（候选形态是按头划分 nonce 空间），它不再由身份唯一性免费得到，否则 I-6.1（nonce 不重用） 从「由身份唯一性保证」变成「由生成器保证」而没人保证它；该义务是推论，未走三方论证。⚠️ 后台重新聚簇是只搬物理位置还是重编 key、重编时怎么分批、进度从哪续做，全仓没有定义。

**依据**：

- E9（key 编码对遍历局部性的影响）：同一棵目录树同一个 LRU 上四条臂对比，`locality_id` 前缀比按 inode 编 key 少发设备读，值得做；depth-first 创建时三条真实臂逐位相同，缓存装得下整棵树时四条臂全部相等、效应消失；收益随改名次数衰减到几乎为零。
- 机制论证（2026-08-25 定案时写下，原文在变更史）：全路径索引在本工程还要多付三条 BetrFS 不必付的代价——分隔 key 就是路径会把 D9（加密） 已定项 5 认定的敏感信息从一棵树扩到所有按路径编 key 的树、引用方身份是路径会让 D1（数据可移动性 / 反向索引） 的反向索引条目随改名全部失效、lifting 对 key 编码的额外要求与 D9（加密） 的 nonce 载体和 D6（快照实现模型） 的快照 ID 位置抢同一段 key 空间。
- 用户定案 2026-09-16：扫描重建时给一个对象的全部 extent key 编同一个首段（原话在变更史）。
- E157（并行线一两条条款的计数模型） 量出：定位一页的节点读取次数、树高与叶片数、换容量后字节区间还原那一格，两臂在全部取样点上逐点相等、分不开；唯一分得开的是 AAD 锚点那一格，文件字节偏移下 N 个 key 全算对、单元序号下只有 1 个算对。
- D9（加密） 已定项 6 逐字「锚点偏移 = 该 extent 首次写出时的对象内起始偏移，读者现算 `key.offset − ptr.extent_off`」，而同一条决策已定项 12 第 2 项写明 AAD 字段组成是 day-1 永久契约 ⇒ 条款本来就预设了这个单位。

**欠**：C346（后台重新聚簇没有定义）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:329-361`（整段抄，未转述）**

```markdown
#### 已定项 11：码 2 的载荷布局与 key 比较序

**定案**：码 2 节点头自描述（预留 2 之后是条目数 2 + 条目宽 2，key 宽 1 排在 key 区间之前，偏移表见 D18（块里携带什么信息） 已定项 18），基础字节数 81 → 86；含 29 字节预留位后头 = `115 + 2 × key 宽`。条目区从含预留位的头末尾（`115 + 2 × key 宽`）起，**定宽条目连续排列**，**声明长度 = 条目数 × 条目宽**；条目之后的空闲字节恒 0 且参与载荷 CRC。载荷 CRC 覆盖 `[86 + 2 × key 宽, 16384)`（起点是明文头末尾，口径与码 1 的 105、码 3 的 107 同）。**条目里 key 一律是条目的前 key 宽 个字节**——「读条目宽 → 按条目宽切条目 → 取前 key 宽 个字节当 key」是一条对全部码 2 树都成立的通用解码规则，扫描重建路径上不用另查一张「这棵树的 key 在条目里排第几」的表。key 的全序按逐字段无符号整数、字段自左向右比较，**盘上小端存储不构成 memcmp 序**。树 ID 从 65536 起编（种类字段 2 字节取不到的最小值，两个量的值域永不相交）：extent 65536、inode 65537、分配记录 65538、记账 65539、中央映射 65540、livelist 65541、稀疏旁表 65542、deadlist 65543、`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547；mkfs 时水位 65536，第一个文件版本那次之后 65548。

**改新池新建文件的字节：是**——六个码 2 单元的条目区起点（`115 + 2 × key 宽`：inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）、声明长度，以及树表条目的字段次序；树 ID 字段（树表条目里的树 ID 与头 ID、码 2 头里的树 ID、记账 key 的树 ID 段）与树 ID 水位的值也变，随树 ID 从 65536 起编。

**多层码 2 树怎么长、怎么收**（记账树、中央映射树，以及照已定项 14 仍用码 2 btree 的树）：① 一次发布里分裂新建的节点与别的单元落在同一段，整条根到叶的路径照常 COW，不加写序步骤、不加屏障；② 叶装不下时从中间切；③ 收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目；④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低，条目在兄弟间挪动之后把右边那个孩子的分隔 key 改成它新的最小 key；逐层可判：分隔 key ≤ 孩子节点头的最小 key，且 > 左邻孩子节点头的最大 key（D18（块里携带什么信息） 已定项 18 的 key 区间）；⑤ 树高 = 根节点码 2 头里的层级 + 1。

- **一次发布里先删后插**：上一版有、这一版没有的 key 先按 key 升序删，再按 key 升序插这一版新有的；只决定形状，不决定写序（写的是最终态，已定项 13）。
- **从盘上重建的树分隔 key 坏了**：重建只核拼得成树的几样（层级、节点不空、内部条目宽、不重复引用），不核分隔 key；下一次发布按分隔 key 删不掉时规划交回，可写挂载在取号之前拒，盘上不变。遇到这种盘之后怎么办（修复、只读挂载）没定。
- **叶条目宽不由读树判**：读树只切 key，叶条目窄于字段表由解条目的一方各报各的；内部条目窄于 key + 86 由读树报。

**内部节点的条目 = 本树 key + 子指针 86**：记账树 22 + 86 = 108、中央映射树 27 + 86 = 113。分配记录树与 extent 树按已定项 14 另起按 key 空间寻址的结构，结构在三方 `m2-keyspace-r1` 判；照码 2 btree 做时它们的内部条目是 96 与 110（extent 110 是用户 2026-09-24 早先定的）。

<!-- format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 -->
<!-- format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 -->

**射程**：它管码 2 节点的载荷怎么排与 key 怎么比，不管头里各字段的偏移（D18（块里携带什么信息） 已定项 18）、也不管每棵树的条目里除 key 之外还有什么（各树自己的分项）。唯一因「key 打头」需要重排的是树表条目，已按这条改成树 ID 打头（已定项 8）。「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」在中央映射条目回到定宽 55 之后不用动（D19（块指针的结构与宽度预算） 已定项 10）——条目宽字段只有一个 u16，按类两宽时这两句顶不住。两条 2026-09-14 补的定案都是用户弹窗定案，没有走三方论证，可推翻。⚠️ 分裂、收缩、分裂点、写序四样被攻过零轮，三方原型只抽了一次样；「低于一半合并」与「追加时末尾切」在崩溃历史上与选中的那两样零对零，没选的理由是用户的取舍，不是安全性。「分裂单开一次发布」出局：只改结构的发布挤掉 D16（发布语义） 已定项 1 的「最近 4 个可退到的不同状态」。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：量的正是码 2 头自描述要加的那几段与映射树 key 宽的代价——加到 86 之后扇出、树高、解头那几节都不掉档，自描述值得做。
- 用户定案 2026-09-13：码 2 头自描述、基础 86、条目区起点、载荷 CRC 覆盖范围、key 比较序（原话在变更史）。
- 用户定案 2026-09-14：条目里 key 一律是前 key 宽 个字节；「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」原样成立（原话在变更史）。
- 用户定案 2026-09-24：extent 树内部条目 110 字节（原话在变更史）；E157（并行线一两条条款的计数模型） 在 110、136 两个取样点上结论都不翻面。
- 无实验：分裂、收缩、分裂点、写序四样是三方原型（`research/prompts/m2-treesplit-r1-opus-model/`）在同一批崩溃历史上零对零之后由用户选的，没立实验号；分隔 key 跟着维护是那一轮主 agent 按打中定（原型 t1.out：沿用 D8（核心索引结构） 已定项 6 的「分隔 key 创建后不变」24 格全红、跟着维护 24 格全 0），判决 `research/prompts/m2-treesplit-r1-main-verification.md`。
- 用户定案 2026-09-24：写序同段照常 COW、只摘空节点且根剩一个孩子时降高、从中间切、内部条目 = 本树 key + 子指针 86（原话在变更史）。
- `research/prompts/m2-treesplit-implementer-report.md` 第五节 B1、B5、B6。
- 用户定案 2026-10-01：树 ID 整体挪到 65536 起，与种类码的值域不相交（岔路单 `research/prompts/m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行：新树的种类码取 9..12（两腿各自得出）。

**欠**：C306（码 2 头不自描述 key 宽）。

```

## 三、实现今天的样子

### 3.1 `crates/`（2026-10-04 现查）

| 处 | 文件与行 | 今天是什么样 |
|---|---|---|
| 头宽、点名项、一条装几项 | `crates/singlefs-format/src/lib.rs:213`（4096）、`:223`（487）、`:226`（56）、`:229`（64，注释「(4096 − 487) ÷ 56 的整数部分」） | 64 |
| 点名项装不下的那一边 | `crates/singlefs-core/src/transaction.rs:6640`（`named_unit_capacity = named_entry_capacity.named_entries_per_record()`）、`:6643`（`.chunks(named_unit_capacity)`）；测试开关 `:178`、`:184-191` | 按每条 64 项切（产品路径），第一段不建记录布局 |
| 原登记 V6（装置用到的格式常量与 `lib.rs` 不同 ⇒ 停机） | 装置第 70、74 行今天是 311 / 67，`lib.rs` 是 487 / 64 | **今天处在触发状态**；这一次改完才解除（回扫报告 3.1 末行：实验页第 12 行「全部与本装置的常量逐字相等 … V6 不触发」现在是假话） |

### 3.2 装置里读旧值的每一处（装置 760 行，sha256 `b96446b89c11fd0a95f23a01211b8acda2f0e997d1bda37c9f90bb8b25ecafb2`）

| 行 | 是什么 | 这一次怎么动 |
|---|---|---|
| 42 | 文件头「journal 记录 4096、记录头 311、点名项 56 ⇒ 67 项/条」 | 487、64 |
| 70 | `const JOURNAL_HEADER_BYTES: u64 = 311;` | 487 |
| 73–74 | 注释「67」、`const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 67;` | 64 |
| 75 | 编译期断言 `JOURNAL_NAMED_ENTRIES_PER_RECORD == (4096 − H) / 56` | 改成调第七节 B8 的那个纯函数，判的东西不变 |
| 271 | 单元数取样点 `[…, 67, 68, …]` | **不动**（第一节最后一行） |
| 305、311 | 锚点行 `a1_…`、`b2_…` | 不动代码，值随常量走（64）；第 311 行的复算改调同一个纯函数 |
| 505–508 | 单测 `锚点甲一与锚点乙二_一条记录装六十七个点名项`：`== 67`、`67 × 56 + H ≤ 4096`、`4096 < 68 × 56 + H` | 改名 `…_一条记录装六十四个点名项`；`== 64`、`64 × 56 + H ≤ 4096`、`4096 < 65 × 56 + H` |
| 511–516 | 单测 `named_entries_per_record_remainder_matches_head_311`：加法形态钉余数 33 | 改名 `…_head_487`，余数 **25**，仍写加法 |

## 四、跑之前已经存在的数

| 数 | 出处 | 对判据的影响 |
|---|---|---|
| 上一次登记的数：67、4059 / 4115、4063 / 4119、余数 37 / 33、装 67 项的 `H` 区间 [289, 344]；原登记第四节整张表 | 读上一次登记与原登记（判据怎么定的） | 不引、不判，第一段那些格随产物逐字节复现 |
| 回扫报告：实验页第 12 行是现状句且今天为假；附录表 e157 那几行的旧值 | 派发给的输入 | Q157.5 的回对清单起点 |
| 实验页 `### 影响的决策` 表下那一段（为判抄哪几条读了这一节）：D8 已定项 3 升级成支撑、D8 已定项 11 维持备料、D23 已定项 12 / 17 维持备料，「第二段一个量都没跑」 | 实验页第 110–122 行 | 是回看结论，不当判据；Q157.5 要求这几行重新回看 |
| 第二节抄进来的 D8 依据行带 E157 的结论（「两臂在全部取样点上逐点相等、分不开；唯一分得开的是 AAD 锚点」「在 110、136 两个取样点上结论都不翻面」） | 被测条款整段抄 | 不读点名项数，不进判据 |
| 实验页引产物名或含旧值的行号（只取行号）：引 `e157-parallel-line-one-clauses` 的 13、19、24、61、66、128、133；含 `name=` 的 13–18、24、29、30、36、37、45、46、66、70–83；含旧值的 12、61、107、114–118、125、127、128、133 | `grep -n … \| cut -d: -f1` | Q157.5 的待查行 |
| **我算出来的（第十三节命令一）**：`⌊(4096 − 487) / 56⌋ = 64`，余 25；`64 × 56 + 487 = 4071 ≤ 4096 < 65 × 56 + 487 = 4127`；装 64 项的 `H` 区间 [457, 512]，456 装 65、513 装 63 | 命令一原样输出 | ⚠️ 跑之前就存在的答案：锚点行两个字段由 67 变 64，第一段判定格不读它 ⇒ 产物应只在锚点行那两个字段上不同。照纪律不删、不回改判据 |
| **我推出来的（推的，没量过）**：第 75 行的编译期断言是 2026-09-27 加的（`git blame -L75,75` 现查，`7a3c8e1e9`），晚于上一次重跑；它让任何把 `H` 挪出「每条项数不变」区间、而不同时改第 74 行的变异编不过。⇒ 今天表里的 M15（`H` → 345，66 项）、M16（`H` → 288，68 项）在改装置之前那一次跑上大概率记「⏭ 无效」，不是「抓到」 | 读装置与 `git blame` | 改前那一次 `mutate.sh` 的实测为准；若实测如此，实验页里「N 条变异全抓」那句今天已经不成立，Q157.5 照实改，交回写明（`mutation-sampling.md` 第八类） |

## 五、臂、阳性对照、真实基线

同上一次登记第五节，只换：

- **两个被判的取值**：`H = 311`（装置一个字节不改，`bash scripts/experiments/replay.sh E157` 现跑；做完会怎样：与登记产物逐字节一致，S1 核）与 `H = 487`（只按第三节 3.2 改，重出产物；做完会怎样：锚点行 `a1_named_entries_per_record=` 与 `b2_named_entries_per_record_check=` 由 67 变 64，其余行逐字节不变——前半句由命令一的算术推出，后半句由「第一段判定格不读点名项数」推出，都由 Q157.1、Q157.4 在产物上核）。
- **P3（单测看得见点名项数这条通道）**改成：第七节 B8 那条纯函数单测在 456 / 457 / 512 / 513 四点上钉 65 / 64 / 64 / 63，由第九节 M17、M18 证明会红。上一次的 P3 靠 M15、M16，它们今天撞第 75 行编不过（第四节末行），不再能当阳性对照。
- **P4（单测分得开新旧两代）**改成：余数单测（B7，25）由 M19（`H` 487 → 488，同档内挪一字节）证明会红；`H` 退回 311 那一代会让每条项数变 67，由第 75 行的编译期断言与 `== 64` 那条单测挡住，不经 `mutate.sh`。
- P0（原登记第一段的对照 P1 / P2a / P2b）照旧，两个取值各跑一次。

真实基线是 `H = 487`。**5.4 实现量**：改 5 处常量与注释、加 1 个纯函数与 1 条单测、改 2 条单测；变异表删 3 条、加 3 条；`replay.sh:193` 换产物名；实验页按 Q157.5 回对。**一段做完**。

## 六、报哪些量与各自的判据

全部对应问题单第 4 行（E157）。上一次登记 Q157.1、Q157.3 的算法与判定照旧（两侧换成 311 与 487）；上一次的 Q157.2（「产物逐字节相同」）在这一次不成立为门槛——按第五节的推导锚点行必然不同——改由 Q157.4 判；新加 Q157.4、Q157.5。

| 格 | 怎么算 | 门槛与判定 | 第 4 行：取什么值让它翻面 |
|---|---|---|---|
| **Q157.1 一条记录装几项** | 新产物 `name=anchors` 行的 `a1_named_entries_per_record=` 与 `b2_named_entries_per_record_check=` | 两个都 = **64**（出自 D23 已定项 4「4096 上余 3609（64 个点名项）」、已定项 12「4 KiB 记录装 64 个点名项」、已定项 17「4096 记录 64 项」原句，不是从 `H` 的取值推出） | ≠ 64 ⇒ F1 / S2 |
| **Q157.4 产物与登记的 exact 产物逐字比** | `cmp research/results/e157-parallel-line-one-clauses-2026-09-25-h311-replay.out <新产物>`；不同就 `diff` | 允许不同的只有 `name=anchors` 那一行的 `a1_named_entries_per_record=`、`b2_named_entries_per_record_check=` 两个字段（67 → 64）；行数相同、`name=done emitted=` 相同。只差这两个字段 ⇒ 记「产物变（只在锚点行）、第一段判定不变」；超出 ⇒ Q157.3 | 这一格就是第 4 行的翻面观测本身 |
| **Q157.3 逐格重判（只在 Q157.4 超出允许集合时做）** | 同上一次登记 | 同上一次登记；超出的那几行同时交停机 S2 | 同原登记各格 |
| **Q157.5 正文回对** | 实验页 `.claude/kb/experiments/157-并行线一两条条款的计数模型.md` 第四节列的待查行逐行过：引登记产物的整行在新产物里逐字找到（锚点行那一句按新值改）；现状句换成新值仍是真话就改（第 12 行「`JOURNAL_HEADER_BYTES=311` … 全部与本装置的常量逐字相等，没有一处对不上，V6 不触发」改成今天的值与改完之后的事实）；事件句与历史节不改（判据：`evidence-discipline.md`「扫出来的清单不能一把全替换」）；「N 单测 / M 条变异全抓」照改前改后两次 `mutate.sh` 的实测数改 | 每一行报「改 / 留（事件）/ 数没变」；`### 影响的决策` 表 6 行重新回看，日期不早于新产物 | 第 4 行的够判条件「变了的出新产物并回对正文」 |

**够判点**：S1 过、新产物出来、Q157.1 与 Q157.4 判完、Q157.5 逐行判完 ⇒ 第 4 行的 E157 够判；Q157.4 在允许集合之内 ⇒ Q157.3 标「够判后未跑」。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符走原登记 F5「条款可能错」）

| # | 断言 | 出处（第二节已整段抄） |
|---|---|---|
| A1 | 一条 4 KiB 记录装 **64** 个点名项；头 487 之后 4096 上余 **3609** | D23 已定项 4、已定项 12、已定项 17 |
| A2–A6 | 原登记第七·甲节 | 不读点名项数，不改 |

### 7.2 独立算出、用命令核过的（第十三节命令一；不符 ⇒ 原登记 V3 作废，先核我这边的算术）

| # | 断言 | 值 |
|---|---|---|
| B2（改） | `⌊(4096 − 487) / 56⌋ = 64`；`64 × 56 + 487 = 4071 ≤ 4096 < 65 × 56 + 487 = 4127` | 第 505–508 行那条单测照这一行改 |
| B7（改） | 余数 `4096 − 487 − 64 × 56` | **25**（311 时 33），写成加法 `JOURNAL_HEADER_BYTES + JOURNAL_NAMED_ENTRIES_PER_RECORD * JOURNAL_NAMED_ENTRY_BYTES + 25 == JOURNAL_RECORD_BYTES` |
| B8（新） | 纯函数 `named_entries_per_record_for_header_bytes(header_bytes) = (4096 − header_bytes) / 56` 在 `header_bytes` = 456 / 457 / 512 / 513 上 | **65 / 64 / 64 / 63**；第 75 行的编译期断言与第 311 行的复算都改调它 |
| B1、B3–B6 | 原登记第七·乙节 | 不读点名项数，不改 |

## 八、轨迹与几何敏感性

8.1 同上一次登记（第一段没有被谓词消费、随轮次走的量）。

8.2 `H` 这个旋钮：上一次靠 M15 / M16 在 345 / 288 两个方向相反的点上看单测红不红，那两个点包的是 67 那一档。今天 64 那一档是 [457, 512]，**取样点换成 456 / 457 与 512 / 513 两对**，写进 B8 的单测（不经全局常量，所以不撞第 75 行的编译期断言）。原登记第八·三节的几何敏感性（`NODE_BYTES`、内部条目宽、`(C₀, C₁)`）不读点名项数，照旧随产物出。

**判别力自证**：B8 那条单测里把 512 那一点的期望改成 63，必须红；把 513 那一点改成 64，必须红；执行员手做一次，记进第十二节。

## 九、变异

**先后**：改装置之前在今天的源码上跑 `bash scripts/mutation/mutate.sh e157-parallel-line-one-clauses scripts/experiments/e157_parallel_line_one_clauses.rs scripts/mutation/tables/e157_parallel_line_one_clauses.tsv`，留日志；改完再跑一遍。两次各按七个符号分栏数。M1–M6、M13（锚点都不含头宽）两次的状态必须相同，任一条变差 ⇒ V7。

| 条 | 怎么改表 | 在哪个取样点上改变输出 | 谁该红 |
|---|---|---|---|
| M14 头宽退回307、M15 头宽过上边界、M16 头宽过下边界 | **从表里删掉**。它们的原文 `const JOURNAL_HEADER_BYTES: u64 = 311;` 改常量后命中 0 次；照改成 487，307 / 345 / 288 三个值都让每条项数不等于 64，撞第 75 行编译期断言编不过（第八类无效）。删除写进实验页「历史版本」的这一次条目 | — | — |
| **M17 每条项数多算一字节**（新） | B8 纯函数里 `JOURNAL_RECORD_BYTES - header_bytes` → `JOURNAL_RECORD_BYTES + 1 - header_bytes` | `header_bytes = 457`：64 → 65（487 上仍是 64，编得过） | B8 的单测 |
| **M18 每条项数少算一字节**（新） | 同一处 → `JOURNAL_RECORD_BYTES - 1 - header_bytes` | `header_bytes = 456`：65 → 64；`= 512`：64 → 63 | B8 的单测 |
| **M19 头宽同档内挪一字节**（新） | `const JOURNAL_HEADER_BYTES: u64 = 487;` → `488` | 每条仍 64（编得过）；余数 25 → 24 | B7 的单测（P4） |

锚点由执行员照改完的源码取，必须唯一命中（第七类）。

## 十、失败条款

原登记第十节 F1–F8 照旧适用于第一段（F4 管第二段，不触发）；上一次登记的 F9（结论变）、F10（全部不变）照旧，F10 的观测改成「Q157.4 只差锚点行两个字段」。

## 十一、作废条款与停机条款

### 作废

原登记十一·甲 V1–V5、上一次登记的 V1、V7 照旧。V2 改成：M17 或 M18「没红」，或 M19「没红」（单测看不见点名项数这条通道，或分不开同一档里的两个头宽）。另加 V8：改完之后 `bash scripts/experiments/replay.sh E157` 对新产物不逐字节一致（确定性模型有隐藏状态）。

### 停机（两边都查，写进报告交回）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| S1 | 基线没立住 | 改装置之前 `replay.sh E157` 不逐字节一致。⚠️ 这一次跑时装置 311 / 67、`crates/` 487 / 64，原登记 V6 正处在触发状态：只当比较基线，不出结论，写死的例外 |
| S2 | 点名项数之外还有东西让产物变了，或产物与我的算术对不上 | Q157.4 超出允许集合；或 B2 / B7 / B8 与产物、单测对不上 |
| S3（即原登记 V6） | 装置与 `crates/` 对不上 | 改完之后、出产物之前 `grep -nE '^pub const (JOURNAL_HEADER_BYTES\|JOURNAL_NAMED_ENTRY_BYTES\|JOURNAL_NAMED_ENTRIES_PER_RECORD):' crates/singlefs-format/src/lib.rs` 不是 `223 … 487`、`226 … 56`、`229 … 64` |

### 出新产物、改登记行

- 产物名：`research/results/e157-parallel-line-one-clauses-<跑的那天，TZ=Asia/Tokyo date +%F>-h487.out`（旧名里的 `h311-replay` 换成 `h487`）；旧产物留着，不覆盖。
- `scripts/experiments/replay.sh:193` 只改第 4 栏，成 `E157|e157-parallel-line-one-clauses||e157-parallel-line-one-clauses-<日期>-h487.out|exact`。
- Q157.4 判「产物不变」时（按推导不该发生）不出新产物、登记行不动，先查 Q157.1。

### 够判停机

第六节「够判点」一到就停。第二段不在这一次，也不因这一次而续跑。没跑的量：Q157.3（Q157.4 在允许集合之内时）标「够判后未跑」；原登记第二段的全部量标「条款已定案，未跑」。

## 十二、修订

2026-10-04 执行员：没有修订判据与臂。单测 22 条全绿之后、产物之前，按第八节「判别力自证」手做了一次（改的是单测期望，不是装置）：把 B8 那条单测里头宽 512 一点的期望改成 63，红（`assertion left == right failed`、`left: 64`、`right: 63`，单测 21 过 1 败）；把头宽 513 一点的期望改成 64，红（`left: 63`、`right: 64`，21 过 1 败）；两处都已还原，还原后 22 条全绿。变异表按第九节删 M14–M16、换 M17–M19，三条新锚点在改完的源码里各唯一命中 1 次。

## 十三、读过的文件与跑过的命令

五份差分重跑登记（E116 第 2 次、E157 第 2 次、E16 第 6 次、E43 第 2 次、E49 第 2 次）是同一次派发里一起写的，这一节的 13.1–13.6 五份相同：列的是这一次派发里读过的全部文件，不只这一份用到的；13.7 起是这一份自己的命令。行号是读的那一刻（2026-10-04）的。`research/results/` 下的产物一份都没读（只在 `replay.sh` 登记行里看到文件名）；实验页的结果节没读，只读了标题行、`### 影响的决策` 一节与按行号取的 `grep -n … | cut -d: -f1`。

### 13.1 规则、共用约束与脚本

| 文件 | 行 |
|---|---|
| `.claude/agent-common.md` | 1–100（全文） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 1–122（全文） |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 1–176（全文） |
| `.claude/rules/three-way-inference.md` | 1–174（全文） |
| `.claude/rules/mutation-sampling.md` | 1–96（全文） |
| `.claude/rules/implementation-first.md` | 1–20（全文） |
| `.claude/rules/format-evolution.md` | 1–81（全文） |
| `scripts/experiments/replay.sh` | 1–60 里的注释行；`grep -nE '^(E116\|E157\|E16\|E43\|E49)[A-Z0-9]*\|'` 命中 77、91、140、156–160、193 |
| `.claude/gate.d/stage-inputs.tsv` | 1–30 的注释行；`grep -nE '^(E116\|E157\|E16\|E16R5\|E43\|E49)\b'` 零命中 |
| `scripts/kb/quote-kb.py`、`scripts/kb/replace-once.py` | 文件头注释（1–30） |

### 13.2 派发给的输入与上一次登记

| 文件 | 行 |
|---|---|
| `research/prompts/m3-r1-batch9-reruns-questions.md` | 全文（17 行） |
| `research/prompts/m3-r1-batch9-reruns-evidence/sweep-311-188-67.md` | `grep -n '^#'`；1–224；附录表 225–418 里带 e116 / e157 / e16 / e43 / e49 的行与第 83–127、170–194 行（`awk` 按行号取）；419 起 `grep` e116 / e157 / e16 / e43 / e49 / `replay.sh` 的命中行 |
| `research/prompts/e116-r1-prereg.md` | 1–524（全文） |
| `research/prompts/e157-r1-prereg.md` | 1–260 |
| `research/prompts/e157-preregistration.md` | `grep -nwE '67\|68'` 命中 52、57、156、244、288、308、311、351、414、477、491、493、527、528、540、541、558、562、580、603、659、664 |
| `research/prompts/e16-r5-prereg.md` | 1–583（全文） |
| `research/prompts/e43-r1-prereg.md` | 1–395 |
| `research/prompts/e49-r1-prereg.md` | 1–44、158–440（读的是工具落盘的那份同内容副本，行号按原文件）、490–657 |

### 13.3 kb

| 文件 | 行 |
|---|---|
| `.claude/kb/decisions/23-journal的角色与格式.md` | `grep -n '^#'`；148–177；226–246；quote-kb 抄出的已定项 1、4、8、12、15、17 整节 |
| `.claude/kb/decisions/01-…`、`03-…`、`08-…`、`18-…`、`20-…`、`21-…`、`25-…`、`27-…` | `grep -nE '^#### 已定项 '` 的命中行；quote-kb 抄出的各节（D1-3、D3-7、D8-3 / 11、D18-7、D20-4、D21-2 / 3 / 4 / 7 / 8 / 14 / 15 / 16、D25-1–5、D27-2 / 3 / 5 / 6 / 13） |
| 五个实验页（`116-…`、`157-…`、`16-journal…`、`43-…`、`49-…`） | `grep -n '^#'`；`### 影响的决策` 一节（116：136–148，157：110–122 含表下那一段，16：360–377，43：263–282，49：219–225）只取前两栏；`grep -n` 产物名、`name=`、旧值三类只取行号 |
| `.claude/kb/decisions/*.md`、`checks-owed.md`、`invariants.md` | `grep -rnE '第五次跑\|E16R5\|e16-fifth'` 命中 23:77、23:86、checks-owed:455、:657（截前 140 字）；`grep -rnE 'E49（反向链宽度 32 还是 64）'` 命中 23:235；`grep -rnE 'E43（扩展点字节上限）.*(201\|3785\|311\|自证)'` 命中 21:98 |
| `.claude/kb/layout/01-first-txn.md` | `grep` 命中 329、357 |

### 13.4 实验装置与变异表

| 文件 | 行 |
|---|---|
| `scripts/experiments/e116_pack_settle.rs` | `grep` 命中各行；260–280；535–545 |
| `scripts/experiments/e157_parallel_line_one_clauses.rs` | 1–80；255–320；495–525；`grep` 命中各行；`git blame -L75,75` |
| `scripts/experiments/e16_fifth_run_peak_under_today_widths.rs` | `grep` 命中各行；220–236；419–445；780–800；817–880 的 `grep` 命中；880–890；1060–1090；1220–1242；`git blame -L887,887` |
| `scripts/experiments/e16_journal.rs` | `grep -nwE` 旧值，命中 915 |
| `scripts/experiments/e43_extension_point_budget.rs` | 140–182；196–252；440–475；590–641；`grep` 命中各行（含 `mount_verdict(` 的 193、439、649、651、656、671–682） |
| `scripts/experiments/e49_chain_width.rs` | 32、35、70–76、104–146；276–292；600–643 的 `grep` 命中；`grep` 命中各行 |
| `scripts/mutation/tables/e116_pack_settle.tsv`、`e157_parallel_line_one_clauses.tsv`、`e49_chain_width.tsv` | 全文（截前 160–220 字） |
| `scripts/mutation/tables/e43_extension_point_budget.tsv` | 第一栏全文；`grep` 命中 16、19、20、21 |
| `scripts/mutation/tables/e16_fifth_run_peak_under_today_widths.tsv` | 第一栏全文 |

### 13.5 `crates/`

| 文件 | 行 |
|---|---|
| `crates/singlefs-format/src/lib.rs` | 215–232；`grep -nE '^pub const (…):'` 命中 20、38、41、168、171、213、216、220、223、226、229、308 |
| `crates/singlefs-core/src/journal.rs` | `grep` 命中 158、174 |
| `crates/singlefs-core/src/transaction.rs` | `grep` 命中 178、184、190、191、290、6640、6643 |
| `crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-harness/src/model.rs` | `grep` 命中 480、478 |

### 13.6 五份共用的命令（原样；读文件的 `sed` / `awk` / `grep -n` 已按行列在上面）

```
for e in E116 E157 E16 E43 E49; do nice -n 19 timeout 60 python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs $e; echo "exit=$?"; done
  # 五个都是「✗ 准入登记里没有实验键 E…（.claude/gate.d/stage-inputs.tsv）」、exit=2
nice -n 19 python3 scripts/kb/quote-kb.py /tmp/claude-1000/m3-r1/batch9-row4-design/qrow4.md 'research/prompts/m3-r1-batch9-reruns-questions.md:12-13' 'research/prompts/m3-r1-batch9-reruns-questions.md:17-17'
  # ✓ 2 段整抄进 …/qrow4.md，回读逐字节一致（同名文件先前一次取错行区间，删掉重抄）
nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/e116-q.md / e157-q.md / e16-q.md / e43-q.md / e49-q.md '<文件>@#### 已定项 N：…' …
  # 五次都是「✓ N 段整抄进 …，回读逐字节一致」、exit=0（8 / 5 / 8 / 14 / 4 段）
git log -L75,75:scripts/experiments/e157_parallel_line_one_clauses.rs --format='%h %ad %s' --date=short
git blame -L75,75 scripts/experiments/e157_parallel_line_one_clauses.rs      # 7a3c8e1e9 2026-09-27
git blame -L887,887 scripts/experiments/e16_fifth_run_peak_under_today_widths.rs   # dbcc0d2f5 2026-09-28
sha256sum scripts/experiments/{e116_pack_settle,e157_parallel_line_one_clauses,e16_fifth_run_peak_under_today_widths,e16_journal,e43_extension_point_budget,e49_chain_width}.rs scripts/mutation/tables/{e116_pack_settle,e157_parallel_line_one_clauses,e16_fifth_run_peak_under_today_widths,e43_extension_point_budget,e49_chain_width}.tsv
  # 装置依次 9417153e… 、b96446b8… 、44bbffd5… 、95dbf822… 、db61f951… 、8eb13c71…；变异表依次 211b8b33… 、01f7f81d… 、8d9abf0c… 、6df56d56… 、ad74fb9c…
sha256sum crates/singlefs-format/src/lib.rs      # bf4f9982490a7d30a4577547779c5061d76b211096cc42a8db71d45936b4835f
TZ=Asia/Tokyo date +%F                           # 2026-10-04
```

### 13.7 命令一：独立锚点（`anchors_h487.py`，全文；五份共用同一个脚本）

脚本 sha256 `4ba238ec10072e5ed9dfdb2fcda4d6baed44ca026391a09a6f4fd2bcebd2f113`，输出 sha256 `61d46e0a418478c4b5a08f75092afd29746ca9babc610021c6925694b13283ca`；在草稿目录 `/tmp/claude-1000/m3-r1/batch9-row4-design/` 里跑，执行员要复核时可以原样拷进 `research/` 下重跑。第一节到第十一节写完之前跑过一次（判问法时），第七节的数取自这一次的输出。

```python
#!/usr/bin/env python3
# 头 487 / 新根段 364 / 点名项 64 重跑登记的独立锚点。常量手抄自 crates/singlefs-format/src/lib.rs:215-229
# 与 D23 已定项 4 / 8 / 15 / 17；不 import 任何装置，公式照各装置源码里那一行的算式手写。
from fractions import Fraction as F
from math import ceil
REC, ITEM, H_OLD, H_NEW = 4096, 56, 311, 487
print("== 共用：头宽分项、一条记录装几个点名项")
print(f"  新根段 86+86+8+8+86+86+4={86+86+8+8+86+86+4}  头 78+9+4+4+4+364+8+16={78+9+4+4+4+364+8+16}")
for h in (H_OLD, H_NEW):
    q = (REC - h) // ITEM
    print(f"  H={h}: 4096-H={REC-h} per_record={q} 余={REC-h-q*ITEM} q*56+H={q*ITEM+h} (q+1)*56+H={(q+1)*ITEM+h} 512-H={512-h} 512 上装 {(512-h)//ITEM} 项")
hs = [h for h in range(0, REC) if (REC - h) // ITEM == 64]
print(f"  装 64 项的 H 区间 [{min(hs)}, {max(hs)}]；H={min(hs)-1} 装 {(REC-min(hs)+1)//ITEM}；H={max(hs)+1} 装 {(REC-max(hs)-1)//ITEM}")
print("== E116 journal 那一项（N=100000，映射条目 61，512 档）")
N, UNIT, NODE, W = 100_000, 32768, 16384, 2
data_w = 1725 * UNIT * W
saved = N * UNIT - 1725 * UNIT
base_key = data_w + 375 * NODE * W + 113 * NODE * W
base_fill1 = data_w + 100_000 * NODE * W + 101_725 * NODE * W
base_filln = data_w + 375 * NODE * W + 226 * NODE * W
for h in (H_OLD, H_NEW):
    j = N * (h + 61)
    print(f"  H={h}: journal_w={j} per_record_bytes={h+61} key512.mw={base_key+j} key512.payback={(base_key+j)/saved:.6f} fill512.b1.mw={base_fill1+j} fill512.b1.payback={(base_fill1+j)/saved:.6f} fill512.bN.mw={base_filln+j}")
print(f"  每格 move_write 之差 = N*(487-311) = {N*(H_NEW-H_OLD)}  saved={saved}  H*(key512)={F(saved-base_key, N)-61} ≈ {float(F(saved-base_key, N)-61):.2f}")
for h in (30851, 30852):
    p = (base_key + N * (h + 61)) / saved
    print(f"  敏感性 H={h}: payback={p:.7f} ge1={int(p >= 1)}")
print("== E43 自证单元档")
for h in (H_OLD, H_NEW):
    r5, r4 = 512 - h, 4096 - h
    b = min(r5, 255)
    print(f"  H={h}: room512={r5}({r5//ITEM} 项) room4096={r4}({r4//ITEM} 项) 占512={F(h,512)*100:.4f}% bound={b} binding={'journal' if r5 < 255 else ('tie' if r5 == 255 else 'root_slot')} fits9={int(b>=9)} fits13={int(b>=13)} fits128={int(b>=128)} 864/bound={864/b:.4f}")
def slots(slot, atomic): return max(atomic // slot, 1)
for h in (H_OLD, H_NEW):
    rows = []
    for atomic in (512, 4096):
        for slot in (256, 512, 4096):
            for n in (0, 128, 512):
                if n > 4096 - 105 - 1: v = "reject_no_payload"
                elif slots(slot, atomic) != 1: v = "reject_slot_shares_atomic"
                elif n > atomic - h: v = "reject_self_witness_overflow"
                else: v = "ok"
                rows.append((atomic, slot, n, v))
    print(f"  mount_eval H={h}: " + " ".join(f"{a}/{s}/{n}={v}" for a, s, n, v in rows if v != "reject_slot_shares_atomic"))
for h in (256, 257, 258, 311, 384, 385, 487, 499, 500, 503, 504):
    b = min(512 - h, 255)
    print(f"  sens512 h={h} room={512-h} bound={b} binding={'journal' if 512-h < 255 else ('tie' if 512-h == 255 else 'root_slot')} lb9={int(b>=9)} lb13={int(b>=13)} f128={int(b>=128)} n128={'overflow' if 128 > 512-h else 'ok'}")
for h in (311, 487, 3584, 3585, 3968, 3969):
    print(f"  sens4096 h={h} room={4096-h} n128={'overflow' if 128 > 4096-h else 'ok'} n512={'overflow' if 512 > 4096-h else 'ok'}")
print("== E157")
for h in (H_OLD, H_NEW, 456, 457, 512, 513):
    q = (REC - h) // ITEM
    print(f"  H={h}: per_record={q} 余={REC-h-q*ITEM}")
print("== E49（base = 头 − 链 4）")
def units(base, c, n, u): return ceil((base + c + n * ITEM) / u)
def div_a(base, a, b, u, top=2000):
    hit = [n for n in range(top + 1) if units(base, a, n, u) != units(base, b, n, u)]
    return len(hit), (hit[0] if hit else None)
def per(base, c): return (REC - base - c) // ITEM
def recs(base, c, n): return max(1, ceil(n / per(base, c)))
def div_b(base, a, b, top=2000):
    hit = [n for n in range(top + 1) if recs(base, a, n) != recs(base, b, n)]
    return len(hit), (hit[0] if hit else None)
print("  自检（复现 r1 的旧锚点）: 93 2v4 512", div_a(93, 2, 4, 512), " 84 4v8 512", div_a(84, 4, 8, 512), " 307 4v8 512/4096", div_a(307, 4, 8, 512), div_a(307, 4, 8, 4096))
for b in (307, 483, 487):
    for u in (512, 4096):
        print(f"  甲 base={b} mod8={b%8} unit={u} 2v4={div_a(b,2,4,u)} 4v8={div_a(b,4,8,u)} cap_items={max(0, u-(b+8))//ITEM}")
for b in (307, 483):
    for u in (512, 4096):
        print(f"  甲 max_items=64 base={b} unit={u} 2v4={div_a(b,2,4,u,64)[0]} 4v8={div_a(b,4,8,u,64)[0]}  (max_items=67: 2v4={div_a(b,2,4,u,67)[0]} 4v8={div_a(b,4,8,u,67)[0]})")
for b in (307, 483, 340, 342):
    print(f"  乙 base={b} per 2/4/8={per(b,2)}/{per(b,4)}/{per(b,8)} 2v4={div_b(b,2,4)} 4v8={div_b(b,4,8)}")
print("  乙 2v4 分岔 base（440..520）:", [b for b in range(440, 521) if div_b(b, 2, 4)[0] > 0])
print("  乙 4v8 分岔 base（440..520）:", [b for b in range(440, 521) if div_b(b, 4, 8)[0] > 0])
groups = {}
for b in range(472, 496):
    cells = tuple(div_a(b, n, w, u)[0] for u in (512, 4096) for (n, w) in ((2, 4), (4, 8)))
    groups.setdefault(b % 8, set()).add(tuple(c == 0 for c in cells))
    print(f"  甲扫描 base={b} mod8={b%8} 512:2v4={cells[0]} 512:4v8={cells[1]} 4096:2v4={cells[2]} 4096:4v8={cells[3]}")
print("  472..495 mod 8 分组一致:", all(len(v) == 1 for v in groups.values()))
print("== E16R5 乙的块数（今天 64 项 / 原装置记录宽 48+56k 按 4096）与 67 项一条对照")
for bt in (63, 64, 65, 66, 67, 68, 70, 72, 73):
    print(f"  batch={bt} today64={bt+ceil(bt/64)} old67={bt+ceil(bt/67)} legacy={bt+max(1, ceil((48+ITEM*bt)/4096))} w72={bt+ceil(bt/72)}")
```

```
$ cd /tmp/claude-1000/m3-r1/batch9-row4-design && nice -n 19 python3 anchors_h487.py > anchors_h487.out
== 共用：头宽分项、一条记录装几个点名项
  新根段 86+86+8+8+86+86+4=364  头 78+9+4+4+4+364+8+16=487
  H=311: 4096-H=3785 per_record=67 余=33 q*56+H=4063 (q+1)*56+H=4119 512-H=201 512 上装 3 项
  H=487: 4096-H=3609 per_record=64 余=25 q*56+H=4071 (q+1)*56+H=4127 512-H=25 512 上装 0 项
  装 64 项的 H 区间 [457, 512]；H=456 装 65；H=513 装 63
== E116 journal 那一项（N=100000，映射条目 61，512 档）
  H=311: journal_w=37200000 per_record_bytes=372 key512.mw=166240384 key512.payback=0.051623 fill512.b1.mw=6760374400 fill512.b1.payback=2.099316 fill512.bN.mw=169943168
  H=487: journal_w=54800000 per_record_bytes=548 key512.mw=183840384 key512.payback=0.057088 fill512.b1.mw=6777974400 fill512.b1.payback=2.104781 fill512.bN.mw=187543168
  每格 move_write 之差 = N*(487-311) = 17600000  saved=3220275200  H*(key512)=96410463/3125 ≈ 30851.35
  敏感性 H=30851: payback=0.9999892 ge1=0
  敏感性 H=30852: payback=1.0000202 ge1=1
== E43 自证单元档
  H=311: room512=201(3 项) room4096=3785(67 项) 占512=60.7422% bound=201 binding=journal fits9=1 fits13=1 fits128=1 864/bound=4.2985
  H=487: room512=25(0 项) room4096=3609(64 项) 占512=95.1172% bound=25 binding=journal fits9=1 fits13=1 fits128=0 864/bound=34.5600
  mount_eval H=311: 512/512/0=ok 512/512/128=ok 512/512/512=reject_self_witness_overflow 512/4096/0=ok 512/4096/128=ok 512/4096/512=reject_self_witness_overflow 4096/4096/0=ok 4096/4096/128=ok 4096/4096/512=ok
  mount_eval H=487: 512/512/0=ok 512/512/128=reject_self_witness_overflow 512/512/512=reject_self_witness_overflow 512/4096/0=ok 512/4096/128=reject_self_witness_overflow 512/4096/512=reject_self_witness_overflow 4096/4096/0=ok 4096/4096/128=ok 4096/4096/512=ok
  sens512 h=256 room=256 bound=255 binding=root_slot lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=257 room=255 bound=255 binding=tie lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=258 room=254 bound=254 binding=journal lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=311 room=201 bound=201 binding=journal lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=384 room=128 bound=128 binding=journal lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=385 room=127 bound=127 binding=journal lb9=1 lb13=1 f128=0 n128=overflow
  sens512 h=487 room=25 bound=25 binding=journal lb9=1 lb13=1 f128=0 n128=overflow
  sens512 h=499 room=13 bound=13 binding=journal lb9=1 lb13=1 f128=0 n128=overflow
  sens512 h=500 room=12 bound=12 binding=journal lb9=1 lb13=0 f128=0 n128=overflow
  sens512 h=503 room=9 bound=9 binding=journal lb9=1 lb13=0 f128=0 n128=overflow
  sens512 h=504 room=8 bound=8 binding=journal lb9=0 lb13=0 f128=0 n128=overflow
  sens4096 h=311 room=3785 n128=ok n512=ok
  sens4096 h=487 room=3609 n128=ok n512=ok
  sens4096 h=3584 room=512 n128=ok n512=ok
  sens4096 h=3585 room=511 n128=ok n512=overflow
  sens4096 h=3968 room=128 n128=ok n512=overflow
  sens4096 h=3969 room=127 n128=overflow n512=overflow
== E157
  H=311: per_record=67 余=33
  H=487: per_record=64 余=25
  H=456: per_record=65 余=0
  H=457: per_record=64 余=55
  H=512: per_record=64 余=0
  H=513: per_record=63 余=55
== E49（base = 头 − 链 4）
  自检（复现 r1 的旧锚点）: 93 2v4 512 (31, 44)  84 4v8 512 (31, 35)  307 4v8 512/4096 (31, 31) (4, 287)
  甲 base=307 mod8=3 unit=512 2v4=(0, None) 4v8=(31, 31) cap_items=3
  甲 base=307 mod8=3 unit=4096 2v4=(0, None) 4v8=(4, 287) cap_items=67
  甲 base=483 mod8=3 unit=512 2v4=(0, None) 4v8=(31, 37) cap_items=0
  甲 base=483 mod8=3 unit=4096 2v4=(0, None) 4v8=(4, 357) cap_items=64
  甲 base=487 mod8=7 unit=512 2v4=(0, None) 4v8=(0, None) cap_items=0
  甲 base=487 mod8=7 unit=4096 2v4=(0, None) 4v8=(0, None) cap_items=64
  甲 max_items=64 base=307 unit=512 2v4=0 4v8=1  (max_items=67: 2v4=0 4v8=1)
  甲 max_items=64 base=307 unit=4096 2v4=0 4v8=0  (max_items=67: 2v4=0 4v8=0)
  甲 max_items=64 base=483 unit=512 2v4=0 4v8=1  (max_items=67: 2v4=0 4v8=1)
  甲 max_items=64 base=483 unit=4096 2v4=0 4v8=0  (max_items=67: 2v4=0 4v8=0)
  乙 base=307 per 2/4/8=67/67/67 2v4=(0, None) 4v8=(0, None)
  乙 base=483 per 2/4/8=64/64/64 2v4=(0, None) 4v8=(0, None)
  乙 base=340 per 2/4/8=67/67/66 2v4=(0, None) 4v8=(455, 67)
  乙 base=342 per 2/4/8=67/66/66 2v4=(455, 67) 4v8=(0, None)
  乙 2v4 分岔 base（440..520）: [453, 454, 509, 510]
  乙 4v8 分岔 base（440..520）: [449, 450, 451, 452, 505, 506, 507, 508]
  甲扫描 base=472 mod8=0 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=473 mod8=1 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=474 mod8=2 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=475 mod8=3 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=476 mod8=4 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=477 mod8=5 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=478 mod8=6 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=479 mod8=7 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=480 mod8=0 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=481 mod8=1 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=482 mod8=2 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=483 mod8=3 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=484 mod8=4 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=485 mod8=5 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=486 mod8=6 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=487 mod8=7 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=488 mod8=0 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=489 mod8=1 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=490 mod8=2 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=491 mod8=3 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=492 mod8=4 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=493 mod8=5 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=494 mod8=6 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=495 mod8=7 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  472..495 mod 8 分组一致: True
== E16R5 乙的块数（今天 64 项 / 原装置记录宽 48+56k 按 4096）与 67 项一条对照
  batch=63 today64=64 old67=64 legacy=64 w72=64
  batch=64 today64=65 old67=65 legacy=65 w72=65
  batch=65 today64=67 old67=66 legacy=66 w72=66
  batch=66 today64=68 old67=67 legacy=67 w72=67
  batch=67 today64=69 old67=68 legacy=68 w72=68
  batch=68 today64=70 old67=70 legacy=69 w72=69
  batch=70 today64=72 old67=72 legacy=71 w72=71
  batch=72 today64=74 old67=74 legacy=73 w72=73
  batch=73 today64=75 old67=75 legacy=75 w72=75
```

### 13.8 交主 agent 认的项

1. 英文名 `parallel_line_one_clauses`（从装置文件名取）。
2. 单元数取样点 67 / 68 不换（第一节最后一行的理由）：它们只为第二段而选，第一段不消费；要换成 64 / 65 由主 agent 改这一条，改了是新登记。
3. M14–M16 从变异表删掉（撞第 75 行编译期断言编不过），换成 M17–M19；改装置之前那一次 `mutate.sh` 若实测 M15、M16 已是「⏭ 无效」，实验页里「N 条变异全抓」那句今天已不成立，Q157.5 照实改。
4. 准入登记表里没有 E157 一行（admission 退 2）。
