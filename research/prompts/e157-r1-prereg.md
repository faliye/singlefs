# E157 重跑登记（记录头 307 → 311）：并行线一两条条款的计数模型第一段在头 311 下还判不判出原来的结论

写于 2026-09-24 21:35 JST，装置改之前、这一次的任何产物之前。原登记 `research/prompts/e157-preregistration.md`（工作区里就在）；E157 只跑过第一段（原登记第五·四节「第一段」：岔路单第 1 行、量 1.1a–1.3c、对照 P1 / P2a / P2b、锚点 A1–A5 / B1–B3、变异 M1–M6 与 M13），第二段（岔路单第 2 行，C491）没跑过。**这一次只重跑第一段**：第二段从未有过产物，没有「原来的结论」可比；而且 C491 已由用户定案（D23 已定项 17「只在最后一条点名」「末条再跨记录」，第二节 2.1 已整段抄），岔路单第 2 行已不再开着。

判据、门槛、作废条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**装置写在哪（写死）：`research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs`（独立手写计数模型，不与 `crates/` 共用代码），不是入库装置。** 与原登记一致（原登记第一节「这是一个纯算术计数模型」）。变异表 `research/mutations/e157_parallel_line_one_clauses.tsv`，复跑行 `research/scripts/replay.sh` 的 E157 那一行（二进制名 `e157-parallel-line-one-clauses`，`research/e7-index-bench/Cargo.toml:437-438`）。

文件名取 `r1`：派发提示点的；仓里（含 git 历史）没有任何 `e157-r*-prereg.md`。仓里另有「`r<n>` = 第 n 次跑」的用法（`e156-r2-prereg.md`），按那个用法这一次是第 2 次；要不要改由主 agent 定，正文不自引文件名。

## 一、问题

**主 agent 给的问题（逐字）**：记录头 `JOURNAL_HEADER_BYTES` 从 307 改成 311 之后，每个实验跑前写死的判据在 311 下还判不判出原来的结论。

**读法写死**：

| 名字 | 写死成 |
|---|---|
| 头宽 `H` | 装置第 70 行 `const JOURNAL_HEADER_BYTES`。被判的取值 `H = 307`（今天的装置，改之前现跑当基线）与 `H = 311` |
| `H` 进模型的通道 | 只有第 74 行 `JOURNAL_NAMED_ENTRIES_PER_RECORD = ⌊(4096 − H) / 56⌋`，它只进锚点行（第 304、310 行，A1 与 B2）；装置第 73 行逐字「本段不消费它，留给第二段」。第一段的全部判定量（1.1a–1.3c）走 extent 树几何与 key 编码，不读 `H`。`N` 的取样点 67 / 68 是第 270 行写死的数组，不从 `H` 算 |
| 「原来的结论」 | 不读实验页的结论节。读作：原登记第一段各判定格在 `H = 307` 现跑产物上判出的值 |
| 「还判不判出」 | 产物是原登记各判定格的输入。两份产物逐字节相同 ⇒ 第一段每一格判出同值；不同 ⇒ 逐格回到原登记第六·一节判（Q157.3） |

**问题单（`research/prompts/m2-header311-rerun-questions.md`，表头与第 4 行，quote-kb 机械抄）**：

**出处 `research/prompts/m2-header311-rerun-questions.md:5-6`（整段抄，未转述）**

```markdown
| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
```

**出处 `research/prompts/m2-header311-rerun-questions.md:10-10`（整段抄，未转述）**

```markdown
| 4 | E157（并行线一两条条款的计数模型） 在头 311 下结论变不变 | 不变 / 变 | 每条记录装的点名项数（⌊(4096 − 头) ÷ 56⌋）或由它推出的判据翻面 | 同上 | 开着 |
```

问题单第 4 行的翻面观测写「每条记录装的点名项数（⌊(4096 − 头) ÷ 56⌋）或由它推出的判据翻面」：第一段里由它推出的只有 A1 与 B2 两个锚点，第一段的判据格一个都不由它推出（上表第二行）。

## 二、被测条款与它引的定义

用 `research/scripts/quote-kb.py` 抄进草稿目录、回读逐字节一致之后追加。原登记第二节抄过的其余条款（D16 已定项 5 / 7、D23 已定项 7 / 12、D8 已定项 3、数据这一侧、extent 树）这一次不读 `H`，不重抄，读法见原登记第 67–279 行。

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
| 头宽 | `crates/singlefs-format/src/lib.rs:164-165`；`:303-315`；`:323` | 311。原登记第三节那一行写的是 `:148`（`JOURNAL_HEADER_BYTES = 307`）——那是 2026-09-22 的行号与值，今天已不同 |
| 一条记录装几项 | `lib.rs:172-173`；`:325` 钉 67 | 67 |
| 点名项装不下的那一边 | `crates/singlefs-core/src/transaction.rs:2621`（`named_unit_capacity = JOURNAL_NAMED_ENTRIES_PER_RECORD`） | 原登记第三节写的 `:1696-1704` 是当时的行号 |
| 两条条款的拒绝分支 | `transaction.rs` 的 `UndecidedClauseBlockingMoreThanOneDataUnit` | 第一段量的是条款定案之后才写得出的形态，与今天的拒绝分支无关（原登记第三节） |
| 记录标志 1 字节 | `crates/singlefs-core/src/journal.rs:151` 今天写 0 | 第一段不建记录的逐字节布局 |

### 3.2 装置里读 `H` 的每一处（`grep -n 'JOURNAL_HEADER_BYTES\|307\|4059\|4115'` 现查）

| 行 | 是什么 | 这一次怎么动 |
|---|---|---|
| 42 | 文件头注释「journal 记录 4096、记录头 307、点名项 56 ⇒ 67 项/条」 | 307 改 311 |
| 70 | `const JOURNAL_HEADER_BYTES: u64 = 307;` | 改 311 |
| 74 | `JOURNAL_NAMED_ENTRIES_PER_RECORD` 的式子 | 不动 |
| 304、310 | 锚点行 `a1_named_entries_per_record=`、`b2_named_entries_per_record_check=` | 不动代码，值随常量走（仍是 67） |
| 505–507 | 单测：`== 67`、`67 × 56 + H ≤ 4096`、`4096 < 68 × 56 + H` | 不改（`H = 311` 时是 4063 ≤ 4096 < 4119，第七节 B2 算过仍成立）；另加一条钉余数 33 的单测（B3） |

## 四、跑之前已经存在的数

**这一次的答案在跑之前就算得出来**：`⌊(4096 − 311) / 56⌋ = 67`，与 307 时相同（第十三节命令二），而第一段的判定格不读 `H`。按模型结构，产物应当逐字节相同。照纪律不删、不回改判据；重跑的用处：装置、单测、变异、产物跟着格式常量一起动；原登记十一·乙 V6「装置用到的任一个格式常量与 `crates/singlefs-format/src/lib.rs` 里的值不同 ⇒ 停机」今天已经被触发（装置 307、`crates/` 311），这一次改完才解除；由产物而不是推理确认没有第二条通道。

| 数 | 出处 | 对判据的影响 |
|---|---|---|
| 67 与 307 / 311 两个头宽下的 4059 / 4115、4063 / 4119；余数 37 / 33；装 67 项的 `H` 区间 [289, 344] | 第十三节命令二；原登记第七·乙节 B2（307 那一版） | 第七节锚点；第八节敏感性点取在 288 / 345 |
| 原登记第四节整张表（144、67、32634、32635、71、8 个点名项、1 条记录、「1 GiB 约 32 900 个单元 ⇒ 叶 229 片、树高 3」、量 1.3a 的 1 / 2 / 145 / 20737、叶容量 35 / 144 / 583、扇出 35 / 147 / 594 与 28 / 119 / 480、8225、p = 7） | 读原登记判据怎么定的 | 是原登记跑前就有的数；这一次不引、不判，随产物逐字节复现 |
| 实验页标题「部分已跑（2026-09-22 第一段，确定性模型，20 单测 / 7 条变异全抓；只跑岔路单第 1 行 …）」 | 看小节结构时 `grep -n '^#'` 撞见 | 是状态与变异账，不当基线；第九节改前改后各跑一次变异表 |
| 回扫报告第二节第 2 小节 E157 那一行：实验页第 12 行是「2026-09-22 那次跑」时的历史事件句 | 回扫报告 | 历史句不改 |

## 五、臂、阳性对照、真实基线

### 5.1 臂

- **原登记的四条臂照旧，第一段只跑 1甲、1乙两条**（原登记第五·一节），一个字不改。
- **这一次只加一个旋钮 `H`，两个被判的取值**：`H = 307`（装置一个字节不改，`bash research/scripts/replay.sh E157` 现跑；做完会怎样——与 `replay.sh` 登记的留存产物逐字节一致，S1 核）；`H = 311`（只改第三节 3.2 那几行、加一条单测，重出产物；做完会怎样——锚点行仍报 67，产物与 `H = 307` 那份逐字节相同。前半句由命令二的算术推出，后半句由「第一段判定格不读 `H`」推出，都由 Q157.1、Q157.2 在产物上核）。

### 5.2 阳性对照（每一条臂都跑）

| 对照 | 对哪条臂 | 怎么做 | 该看到什么 | 看不到时 |
|---|---|---|---|---|
| **P0 原登记第一段的对照** | 1甲、1乙 × 两个 `H` | 不改：P1（`NODE_BYTES` 改 4096）、P2a、P2b，随产物照出 | 两个 `H` 下都按原登记第五·二节那一列变动；对应行两次逐字节相同 | 作废 V1 |
| **P3 单测看得见 67 这条通道** | 装置一次（通道不分臂） | 变异 M15（`H` → 345）、M16（`H` → 288），第九节 | 钉 67 的单测红 | V2 |
| **P4 单测分得开 307 与 311** | 同上 | 变异 M14（`H` → 307） | 新加的余数单测（B3）红 | V2 |

### 5.3 真实基线

真实基线是 `H = 311`。原登记第五·三节那一格（第一个事务：1 条记录、8 个点名项、extent 1 条叶记录 / 1 叶 / 树高 1 / key `(0, 1, 0)`）照旧随产物出，它与 `crates/` 的对拍照原登记 F1 / V6 办。

### 5.4 实现量与分段

改 2 行、加 1 条单测；变异表加 3 条；`replay.sh` 的 E157 一行换产物文件名。编译一次、单测一次、变异表两次（改前改后）、产物一次。**一段做完**。实验页第 12 行是历史句不改；产物出来之后只在「历史版本」里加一条这一次的记录。

## 六、报哪些量与各自的判据

全部对应问题单第 4 行。每格各报各的判定，不合取。

| 格 | 怎么算 | 门槛 | 为什么不是同义反复 | 判定 | 问题单第 4 行：取什么值会让它翻面 |
|---|---|---|---|---|---|
| **Q157.1 一条记录装几项** | `H = 311` 产物 `name=anchors` 行的 `a1_named_entries_per_record=` 与 `b2_named_entries_per_record_check=` | 两个都 = 67 | 67 出自 D23 已定项 4 / 17 原句，不是从 `H` 的取值推出 | = 67 ⇒ 通道没动；≠ 67 ⇒ F1 / S2 | `H ≤ 288` 或 `H ≥ 345` |
| **Q157.2 产物逐字节相同** | `cmp` `H = 311` 产物与 `H = 307` 现跑那份 | 逐字节相同 | 两份产物不同的来源不止 `H`，要量 | 相同 ⇒ 第一段每一格判出同值，第 4 行记「不变」；不同 ⇒ Q157.3 | 第一段判定格不读 `H`，只可能因 S2 那类原因不同 |
| **Q157.3 逐格重判（只在 Q157.2 不同时做）** | `diff` 出来的每一行，对到原登记第六·一节的判定格（1.1a、1.1b、1.2a、1.2b、1.2c、1.3b、1.3c），按原门槛各判一次 | 原登记各格门槛 | 原登记写死的 | 任一格两次不同 ⇒ 第 4 行记「变」；全部相同 ⇒ 记「不变（产物有与判据无关的差）」并交 S2 | 同原登记各格 |

**够判点**：`H = 311` 产物出来、Q157.1 与 Q157.2 各判完；Q157.2 相同 ⇒ 第 4 行够判，Q157.3 标「够判后未跑」。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符走原登记 F5「条款可能错」）

| # | 断言 | 出处 |
|---|---|---|
| A1 | 一条 4 KiB 记录装 **67** 个点名项；头 311 之后 4096 上余 **3785** | D23 已定项 4「4096 上余 **3785**（67 个点名项）」、已定项 17「4096 记录 67 项」（原登记 A1 引的是已定项 12，今天的原句在已定项 4 / 17） |
| A2–A6 | 原登记第七·甲节 A2–A6 | 不读 `H`，不改 |

### 7.2 独立算出、用命令核过的（第十三节命令二；不符 ⇒ 原登记 V3 作废，先核我这边的算术）

| # | 断言 | 值 |
|---|---|---|
| B2（改） | `⌊(4096 − 311) / 56⌋ = 67`；`67 × 56 + 311 = 4063 ≤ 4096 < 68 × 56 + 311 = 4119` | 原登记 B2 那一行的 307 版本是 4059 / 4115，这一次换成这一行 |
| B7（新） | 余数 `4096 − 311 − 67 × 56` | **33**（307 时 37）。新加一条单测钉它，是全部单测里唯一分得开 307 与 311 的一条 |
| B1、B3–B6（原） | 原登记第七·乙节 | 不读 `H`，不改。⚠️ 原登记的 B3 是「净荷 32634 / 32635」，这一次的新条目为了不撞号记作「B7」，执行员在单测名与产物里照写 B7 |

## 八、轨迹与几何敏感性

### 8.1 轨迹

第一段没有被谓词消费、随轮次或崩溃点走的量（原登记第八·一节的轨迹要求落在第二段的 2.3a / 2.3b 与第一段 1.1a / 1.1b 的页集合上）。1.1a / 1.1b 按页集合逐页报的形态照旧随产物出，Q157.2 要求两次逐字节相同。

### 8.2 几何敏感性（`H` 这个旋钮）

307 与 311 落在 67 那一段同一侧。敏感性靠两个方向相反的取样点上的单测：`H = 345`（容量 66）与 `H = 288`（容量 68），即第九节 M15、M16。原登记第八·三节的几何敏感性（`NODE_BYTES`、内部条目宽、`(C₀, C₁)`）不读 `H`，照旧随产物出。

**判别力自证**：钉 67 的那条单测，把期望值改成 66，`H = 311` 上必须红、`H = 345` 上必须绿；执行员手做一次，记进第十二节修订。

## 九、变异

**先后**：改装置之前在今天的源码（`H = 307`）上跑 `bash research/scripts/mutate.sh e157-parallel-line-one-clauses research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs research/mutations/e157_parallel_line_one_clauses.tsv`，留日志；改完再跑一遍。两次各报「抓到 / 无效 / 没红」三个数，逐条比：原有 7 条（M1–M6、M13）任一条状态变差 ⇒ V7。原有 7 条的锚点都不含头宽。

新加三条（编号接原登记 M1–M13 之后，避开第二段预留的 M7–M12）：

| 条 | 改什么 | 在哪个取样点上改变输出 | 谁该红 |
|---|---|---|---|
| M14 头宽退回 307 | `const JOURNAL_HEADER_BYTES: u64 = 311;` → `307` | 余数 33 → 37；锚点行不变 | B7 的单测（P4） |
| M15 头宽过上边界 | 同一行 → `345` | 锚点行 `a1_…` 与 `b2_…` 67 → 66 | 第 505 行钉 67 的单测；第 506 行（`67 × 56 + 345 = 4097 ≤ 4096` 不成立）也红；第 507 行（`4096 < 68 × 56 + 345 = 4153` 仍成立）不红 |
| M16 头宽过下边界 | 同一行 → `288` | 锚点行 67 → 68 | 第 505 行；第 506 行（`67 × 56 + 288 = 4040 ≤ 4096` 仍成立）不红、第 507 行（`4096 < 68 × 56 + 288 = 4096` 不成立）红 |


## 十、失败条款

原登记第十节 F1–F8 照旧适用于第一段（F4 管的是第二段的量，这一次不触发）。这一次另加：

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F9 | **结论变了（正当结果）**：Q157.3 判出原登记某一格两次不同 ⇒ 第 4 行记「变」，写明哪一格、两边各是什么；不许回头改门槛 | Q157.2 不同，且 `diff` 出来的行落在 1.1a–1.3c 某格的输入上、按原门槛判出不同的值 |
| F10 | **全部不变（正当结果）**：产物逐字节相同 ⇒ 第 4 行记「不变」；不许写成装置坏了（装置看不看得见头宽由 P3、P4 单独判） | `cmp` 无输出、退出码 0 |

## 十一、作废条款与停机条款

### 作废

原登记十一·甲 V1–V5 照旧。这一次另加：

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| V1 | 原登记第一段的阳性对照（P0）在任一个 `H` 上没按预期变动 | 产物里 P1 / P2a / P2b 那几行不符原登记第五·二节，或两次不同 |
| V2 | 单测看不见头宽通道，或分不开 307 与 311 | M15 或 M16「没红」；M14「没红」 |
| V7 | 改常量后原有变异有一条状态变差 | 两次 `mutate.sh` 日志逐条比 |

### 停机（两边都查，写进报告交回）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| S1 | 基线没立住 | 改装置之前 `replay.sh E157` 不逐字节一致 |
| S2 | 头宽之外还有别的东西让产物变了，或产物与我的算术对不上 | Q157.2 不同而 Q157.1 仍是 67；或 B2 / B7 与产物、单测对不上 |
| S3（即原登记 V6） | 装置与 `crates/` 对不上 | 开跑前现查 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_HEADER_BYTES` ≠ 311、`JOURNAL_NAMED_ENTRY_BYTES` ≠ 56；或原登记 F1 的真实基线那一格对不上。⚠️ 改之前的 `H = 307` 那一次跑本身就处在 V6 的触发状态（装置 307 ≠ `crates/` 311）：那一次只当比较基线，不出结论，这是登记里写死的例外 |

### 够判停机

第六节「够判点」一到就停，第 4 行记「变」或「不变」交回。第二段不在这一次，也不因这一次而续跑：岔路单第 2 行（C491）已由 D23 已定项 17 定案。没跑的量：Q157.3（产物相同时）标「够判后未跑」；原登记第二段的全部量标「条款已定案，未跑」，这不是这一次的欠账。

## 十二、修订

单测跑完（21/21 绿）、产物一次没跑之前，B7 那条新单测按 E155 第五次重跑登记同一节撞见的坑（`research/prompts/e155-r5-prereg.md` 第十二节）预先写成加法
`JOURNAL_HEADER_BYTES + JOURNAL_NAMED_ENTRIES_PER_RECORD * JOURNAL_NAMED_ENTRY_BYTES + 33 == JOURNAL_RECORD_BYTES`，
不写减法，理由同 E155：`research/Cargo.toml:15` 的 `overflow-checks = true` 会让减法形态在某些变异（把 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 改大）下编译期溢出，被 `mutate.sh` 记成「无效」而不是「抓到」。这一条判据、门槛、臂一个字未改，只是实现写法上的收严（同一条真值在运行期而非编译期被判定）。函数名最初带 `b3_prime` 前缀，被 `naming-lint.sh` 判红（「b3」是单字母加数字），改成 `named_entries_per_record_remainder_matches_head_311`，不影响判据。

第二条（主 agent 转达门禁 12 号红，用户点名）：这份登记里四个带撇号的角标名改成各自那一族的下一个未用号——原写成「P1 加撇号」的那一项改成 `P3`（P0、P1、P2a、P2b 已占用）、原写成「P2 加撇号」的那一项改成 `P4`、原写成「B3 加撇号」的那一项改成 `B7`（原登记 B1–B6 已占用；第七节 7.2 那一行原写成「B3（新）」也一并改成「B7（新）」，免得与它下一行「原登记的 B3 是净荷…」自相矛盾）、原写成「V4 加撇号」的那一项改成 `V7`（原登记 V1–V6 已占用），出现在第 149、150、186、187、203、209、221、233、240、249 行；装置源码 `e157_parallel_line_one_clauses.rs` 里同一处文档注释的角标同步改成 `B7`。`bash .claude/gate.d/12-no-prime-marks.sh` 对这四个位置已转绿（脚本自身的 `.claude/gate.d/lib-prime-marks.py` 仍红，那是另一个未跟踪的文件、不在这一次改动范围内，写进报告交主 agent）。不改判据、不改门槛、不改锚点在哪个取样点上改变输出，只改标签字面。


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

