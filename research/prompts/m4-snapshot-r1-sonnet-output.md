# m4-snapshot-r1 云端正推（Sonnet）报告

日期 2026-10-13。读的是工作树 `/home/fy5090/.local/share/singlefs/work/m4-predev`（不是主仓）。被判对象是正文第二节「实现今天的样子」与第三节 S1–S9 各格的问法。
判定词：一致 / 冲突 / 规则没说，写的是「我从条款与 crates/ 推出来的」与背景材料或条款另一处相比的结果。引文行号都用 `grep -nF` 现取；引 kb 条款多写 `文件@小节`。
模型目录 `research/prompts/m4-snapshot-r1-sonnet-model/`：D5 纪元式 deadlist 的小模型，S3、S4 引它的输出（数在「模型输出」一节）。
本报告各格的「推荐」只在格内一处；要交用户的岔路写成候选、翻面观测、够判条件三列，不带倾向。

## 〇、先说七件改变整盘方案的事实（后面各格引用）

**G1 释放现场：背景材料说的那一处不是带文件版本的释放（冲突）。**
并排：背景材料第二节第 4 条（转述）说 `:1861` 的 `for released in settled.release { allocator.release(released, txg) }` 是「删除批与覆盖写共用」的那一处；里程碑文件一·2 同样说。
代码现查：`:1861` 在 `prepare_the_row_publish_on_a_version_without_file`（树表 0 条那一版的写行），带文件的版本（覆盖写、顺序写、删除批、写行）释放在 `publish_admitted`。
- 「for released in settled.release.iter().copied() {」（`crates/singlefs-core/src/transaction.rs:1861`）所在函数是「pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(」（`crates/singlefs-core/src/transaction.rs:1823`）
- 「for placement in release {」（`crates/singlefs-core/src/transaction.rs:8141`），其下「allocator.release_leaving_the_record_allocated_on(」（`crates/singlefs-core/src/transaction.rs:8147`）
- 释放清单的来源：「fn placements_released_by_this_publish(」（`crates/singlefs-core/src/transaction.rs:6825`）→「pub fn placements_to_release_via_mapping(」（`crates/singlefs-core/src/transaction.rs:3265`）。
`publish_admitted` 只有一个调用点（`grep -nF 'publish_admitted(' crates/singlefs-core/src/transaction.rs` 命中 6799 调用、8099 定义两行）。⇒ 快照分叉的切口是 `:6825`（出清单）与 `:8141`（落释放）之间，不是 `:1861`。
另一处行号漂移：背景材料第二节第 8 条的 `crash_facts.rs:287` 现为 `:369`（「pub const MAXIMUM_ENUMERATED_WRITES: usize = 2048;」，`crates/singlefs-checker-tier/src/crash_facts.rs:369`）；`MAXIMUM_WRITES_PER_LOCATION` 现为 `:54`（「pub const MAXIMUM_WRITES_PER_LOCATION: usize = 64;」，`crates/singlefs-checker-tier/src/crash_judge_tables.rs:54`），不是 `:51`。
什么现象会推翻：另有带文件的发布路径也调 `prepare_the_row_publish_on_a_version_without_file`。`grep -rn` 它的调用点只有 `transaction.rs:1605`（树表 0 条写行）与 `mount.rs:5039`（可写挂载的预演），都是树表 0 条的一版。

**G2 钉住的单元要留映射条目：今天一释放就把它从映射里去掉（规则没说，是 C300 的前一步）。**
今天一版的中央映射树 = 这一版每个角色的映射条目：`mapped_units` 只装本版单元（「pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,」，`crates/singlefs-core/src/transaction.rs:2896`），被换下的角色在新版映射里没有条目。
- 条款：「逻辑上照样释放（映射条目去掉）」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`）
- 条款：「deadlist / livelist 按映射 key 记，搬迁只改映射一条条目」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:185`）
- 条款：「身份就是中央映射 key」（`.claude/kb/decisions/05-快照-空间记账机制.md:275`）
三句合起来要求：进 deadlist 的单元，映射条目必须一直留到它从 deadlist 被放掉。今天没有这条路，也没有条款写「进 deadlist 的单元映射条目留不留」。C300 只问了「映射丢了怎么办」（欠账表 `.claude/kb/checks-owed.md` 里 C300 那一行），没问「平时留不留」。
后果：不留，则销毁快照那一批按 key 查映射查不到落点，放不掉（空间泄漏）；读快照遇到过期的位置提示也读不回。S3 的候选以此为轴。
推翻：`plan_the_multi_level_trees`（「fn plan_the_multi_level_trees(」，`crates/singlefs-core/src/transaction.rs:5625`）里另有把被换下单元的映射条目带到下一版的路径（我读到的 `mapped_units` 分支只有「重写的现算、照抄的取上一版」）。

**G3 树表是一个节点，没有容量拒绝，只有断言（一致，补一个数）。**
树表条目 key 8 字节、条目 200 字节，一个节点 81 条，现用 12 条。数用实现的同一个式子（`unit::index_node_entry_capacity`，「pub fn index_node_entry_capacity(key_width: usize, entry_width: usize) -> usize {」，`crates/singlefs-core/src/unit.rs:151`）算，输出见「命令输出」第 1 条：81；甲或丁每个快照 2 条 ⇒ 至多 34 个，乙 1 条 ⇒ 69 个。
超过容量走断言：「条目装不进一个节点：写者要先按 index_node_entry_capacity 判、报错，不许走到这里」（`crates/singlefs-core/src/unit.rs:180`）。⇒ 建快照的入口必须自己先判、报类型化错误。

**G4 S.txg 取哪个数：D5 与 D8 两处措辞读不出同一个数（冲突读法，D16 能裁）。**
- 「| `S.txg` | **S 捕获的最后一个已发布 txg** | 不是「创建操作所在的 txg」 |」（`.claude/kb/decisions/05-快照-空间记账机制.md:321`）
- 「- **① 快照进树表**，每个快照一条条目；**诞生 txg 8 字节**，口径是**快照完成（发布）那一次 checkpoint 的 txg**」（`.claude/kb/decisions/08-核心索引结构.md:255`）
- 「快照发布点必须与 checkpoint 边界对齐，且位置钉死在「本 checkpoint 全部数据变更之后」」（`.claude/kb/decisions/16-发布语义.md:308`）
裁法：D16 把快照点放在发布末尾，「捕获的最后一个已发布 txg」就是携带这条快照条目的那次发布的 txg；独立发布（无数据变更）时它的用户可见树与上一版逐字相同。取 S.txg = 条目诞生 txg = 建快照那次发布的 checkpoint_txg，三处读成同一个数。
推翻：条款作者另有读法（S.txg = 建快照前一次发布的号）。两种读法的存储行为等价（模型里「写后同发布建快照」与「独立建快照」各 20000 史零违例），差别只在标号一格；I-9.25 的「p ≤ 这条根的 checkpoint_txg」（`crates/singlefs-checker/invariants/09-inode树结构.md:37`）两种都过。

**G5 「每次用户改动先走空」已经回答了 S2 的队列问题（一致）。**
「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的」（`.claude/kb/decisions/08-核心索引结构.md:482`）。建快照与销毁快照做成 `UserChange` 的新变体（「pub enum UserChange<」，`crates/singlefs-core/src/mounted_session.rs:67`）就继承这条：快照捕获的版本里 nlink = 0 的记录为 0 条。同一条纪律也给 S4 一个选项：销毁意图做完之前不接别的改动。

**G6 I-3.1 的遍历必须并入快照根，否则第一次钉住就红（规则没说，推的）。**
I-3.1 逐字「已分配空间统计 == 实际遍历所有引用得到的和」（`crates/singlefs-checker/invariants/05-空间记账.md:8`）。钉住的单元不进 defer、分配记录仍是「已分配」，从活头走读够不着它们；遍历不含快照根与待放的 deadlist 条目，和就比统计少。⇒ checker 改动里「走读并入快照根」不是可选项。

**G7 现有 checker 与 core 已经按「每条 kind-2 条目是一个头」写了循环（影响 S1 的甲）。**
命令输出见「命令输出」第 2 条：`walk.rs` 里 `TREE_KIND_INODE` 20 处、`TREE_KIND_EXTENT` 12 处，`recovery.rs` 10 / 7 处。`entries_of_kind(TREE_KIND_INODE)` 的逐条循环 3 处（`walk.rs:11086`、`:11439`、`:11609`），`deleted_inodes` 配对在 `walk.rs:9759` 起：每棵 inode 树必须恰有一条头 ID 指着它的 `deleted_inodes` 条目，配不上记「判不了」。
⇒ 快照做成额外的 (kind 2, kind 1) 条目，这些循环会把快照当头；做成新种类，它们按种类过滤、天然看不见。

## S1 只读快照在盘上是什么

**判定：冲突（正文的甲、乙两种写法各撞一处条款或字段表）+ 规则没说（快照条目的种类码、两根指针放哪）。**

两边并排：
- 条款：「- **① 快照进树表**，每个快照一条条目；**诞生 txg 8 字节**，口径是**快照完成（发布）那一次 checkpoint 的 txg**」（`.claude/kb/decisions/08-核心索引结构.md:255`）
- 实现：一条树表条目只有一个根指针槽，「pub root: NodePointer,」（`crates/singlefs-core/src/records.rs:442`），86 字节；一个快照要冻住两个根（inode 树、extent 树），两根共 172 字节。
- 预留只剩：「合计 48，余 28。」（`.claude/kb/decisions/08-核心索引结构.md:264`）（预留 76 字节，已认购 48 = 头条目的 24 + 快照条目的 24；一条快照条目只用自己那 24，余 52）。
所以「每个快照一条条目、条目里装两根指针」放不下；乙里写的「活头两根指针的引用」也不能是指向活头的引用——头一动，快照那两根就不是它捕获时的根了，必须存值。
不列候选：把第二根指针压进预留。一根指针的有效信息是出生树 8 + 出生 txg 8 + 位置 28 + 实例 4 + 出生序号 4 = 52 字节，恰好等于快照条目预留的余量，一个认购者都不剩，D9 开了之后 MAC 16 + nonce 12 没处放。

**候选**
| 名 | 定义 | 每快照树表条数 | 同时活快照上限（单节点 81 条，12 条已占） | 要动 crates/ 的哪几步 | 新写出的字节 |
|---|---|---|---|---|---|
| 甲 | 新发号一对 (码 2, 码 1) 条目，根指针逐字抄；用 flags 位标「快照」 | 2 | 34 | `records.rs` `TreeTableEntry`（加 `previous_snapshot_txg` 字段，`parse` 放过 flags 位）；`recovery.rs` 4 处与 `mounted_read.rs` 1 处调用的 `tree_table_entries_each_kind_at_most_once` 放行并按 flags 分头与快照；`walk.rs` 20 + 12 处按种类取条目的地方逐处分头与快照（G7）；I-9.16 ③、I-9.27、I-9.25 改写 | 2 × 200 = 400 |
| 乙1 | 新种类 14，一条条目；树表条目加宽 200 → 286 放第二根指针 | 1 | 44（(16384−131)/286 = 56，减 12） | 全部条目字节变（`singlefs-format` 的 `TREE_TABLE_ENTRY_BYTES`、mkfs、第一个文件版本的树表单元、E142 对拍），D8 已定项 8 用户 2026-09-16 定的 200 要重开 | 286，且每条旧条目多 86 字节 |
| 乙2 | 新种类 14，一条条目，根指针指向一个新的「快照根单元」（装两根指针）| 1 + 1 个 16 KiB 单元 | 69 | 新 `TransactionUnit` 角色（`transaction.rs` 约 15 处 match）、新单元类、映射、分配、journal 点名 | 200 + 16384（×2 盘）每快照 |
| 丙 | 新种类 14 的「快照树」，叶是码 3 容器（记录 204 字节：快照 ID 8 + S.txg 8 + prev 8 + 头 ID 8 + 两根指针 172），照 `tombstone_tree.rs` / `lineage_tree.rs` | 0（全池 1 条） | 不受树表限（容器 159 条，根满长高） | 新树 ID、`FileVersionTreeIdentifiers` 13 → 14 个号（新池新建文件的树表 12 → 13 条、水位 65549 → 65550）、新容器树模块、`TransactionUnit` 新角色一族、D8 已定项 9 登记 | 树表 +200（零根）；每容器 32768 |
| 丁 | 两个新种类：14 = 快照的 inode 根，15 = 快照的 extent 根，各一条 | 2 | 34 | `records.rs` 加两个种类码常量与 `previous_snapshot_txg` 字段；`recovery.rs` 的 `tree_table_entries_each_kind_at_most_once` 对 14、15 放行；I-9.16 ③、I-9.27、I-9.25 各加一格；已有 kind-1/2 的循环一处不动 | 2 × 200 = 400 |
计算：81、34、69、44、159 来自「命令输出」第 1 条。

**推荐丁。** 依据：（一）G7，它是唯一不碰已有 kind-1/2 循环的两条条目形（甲 20 + 12 处要逐处分头与快照）；（二）D8 已定项 8 的兼容档：旧读者对不认识的种类只是看不到那棵树（「新增 keyspace 对旧读者只是看不到新树」，`.claude/kb/decisions/08-核心索引结构.md:280`），甲把快照放在 flags 位上则旧读者 EIO 掉条目；（三）不加宽条目、不新增单元角色。代价写在上表：D8 已定项 8 ① 的字面「每个快照一条条目」要改成「每个快照两条」，34 个快照封顶要写成第一版的限制并报类型化错误。
什么现象会推翻：用户给的目标同时活快照数大于 34（翻到丙或乙2）；或 `field_table_width_of_tree_kind`（`crates/singlefs-checker/src/walk.rs:287`）那类按种类分发的表对新种类不是「`_ => None` 跳过」而是报错（我读到的是 `None`）。

**逐问回答（丁）**
1. 条目里写什么：码 14：树 ID 新发（水位 +2）、flags 0、根指针 = 建快照那一版 inode 树根指针逐字抄、`previous_snapshot_txg` = 建快照时活头的该字段值（D8 已定项 8 ③ 说快照条目也有）、诞生 txg = S.txg（G4）、头 ID = 被捕获的头的 inode 树 ID、预留中谱系标号 16 字节写 0（C144 仍欠，线性单头用不上，克隆再定）。码 15：树 ID 新发、根指针 = extent 树根指针、`previous_snapshot_txg` 写 0、头 ID = 配对的码 14 条目的树 ID、诞生 txg 同码 14。
2. I-3.7 的可枚举结构：树表里种类 14 的条目集合（不含带「销毁中」记号的，见 S4）。这是 D5 已定项 21 要的「盘上可枚举的持久结构」，它随根 COW，零额外单元。
3. I-9.16 ③：现文「③ 每一种种类码（登记过的与没登记的都算）至多一条」（`crates/singlefs-checker/invariants/09-inode树结构.md:28`）改成「种类 1..13 每种至多一条；种类 14、15 条数相等，每条 15 的头 ID 恰是一条 14 的树 ID」。core 一侧：「pub(crate) fn tree_table_entries_each_kind_at_most_once(」（`crates/singlefs-core/src/recovery.rs:2354`）按种类放行 14、15，C675 的两处判（core、checker）同改。GPU 判器的 ③ 同改（`crash_judge_checker.wgsl` 的 `w_judge_tree_table_entries_ordering`，C675 欠账表记着它只置一位）。
4. I-9.27：现文末尾「放宽成「等于其中某一条的树 ID」」（`crates/singlefs-checker/invariants/09-inode树结构.md:39`）。加两行：14 的头 ID 等于某条种类 2 条目的树 ID；15 的头 ID 等于某条种类 14 条目的树 ID 且一一配对。「收严留给多头那一批」不动。
5. I-9.25：现文「inode 树的条目为 0，或 1 ≤ p ≤ 这条根的 `checkpoint_txg`」（`crates/singlefs-checker/invariants/09-inode树结构.md:37`）加：种类 14 的条目同样 0 或 1 ≤ p ≤ 自己的诞生 txg（前驱在它之前）；种类 15 恒 0。
6. 「快照不许写」在写入口哪一处拒：没有入口。写路径取树号只从 `TransactionOutput.tree_identifiers`（13 个固定号，`crates/singlefs-core/src/transaction.rs:2703` 的 `FileVersionTreeIdentifiers`），快照条目不进这个结构，`UserChange` 的变体（`crates/singlefs-core/src/mounted_session.rs:67`）没有一个收快照号；读快照的句柄类型不带写方法。要测的是「构造一个指向快照树号的写计划」编译不过或类型化拒绝，而不是运行期断言。
7. 其它树持不持：`deleted_inodes` 不持（建快照前已走空，G5）；墓碑树、退役树、谱系树不持（全池一份，快照读文件内容用不到）；分配记录树、记账树、中央映射树不持（每版一棵、不是头的结构；映射条目留存见 G2、S3）；livelist、稀疏旁表、deadlist 是全池一棵，不按快照分；快照没有 inode 号水位行（D5 已定项 4 第 12 项只为可写头维护，记账行数不变，仍是 15 行）。
8. 读快照时期望的树号（今天的读法对快照会判红）：快照根指向的单元头里写的是出生树（头的树号），不是快照条目自己的新树号。今天树表指着的根按条目自己的树号比：「与树表条目里的树 ID 对树表指着的根是同一个读法」（`crates/singlefs-checker/src/walk.rs:1790`），读侧 `open_pool_for_read` 同样传条目的树号：「tree: inode_entry.tree,」（`crates/singlefs-core/src/mounted_read.rs:474`）。快照条目（码 14、15）指着的根必须改成取根指针头部的出生树（中央映射树已经这样取，`walk.rs:1789` 的注释），否则 I-1.3 当场红。这是 C110 在只读快照上的最小形态（`.claude/kb/checks-owed.md` C110 那一行）。

**要交用户的岔路（S1）**
| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 第一版同时活着的快照数上限取多少：1、34（树表单节点封顶，甲或丁）、44（乙1）、69（乙2）、不设（丙） | 出口 3 的固定脚本最多同时活几个快照；出口 4 的随机历史会不会同时留多个（数脚本里建快照减销毁快照的峰值，量一次脚本即得）；峰值超过 34 丁与甲出局，超过 69 乙2 出局 | 用户给出一个上界 |
| 快照点名：沿用码 14 条目的树号（单调、不重发、8 字节）／预留里加一个单调序号／预留里放定长名字 | 后面里程碑的 POSIX 面与 FUSE 要不要按名字访问快照；第一版有没有目录（没有目录，名字挂在哪） | 用户说第一版的调用方是按号还是按名字 |

## S2 建快照那次发布写什么、切在哪

**判定：一致（D16 已定项 14、D5 已定项 9、12、13 与 D8 已定项 16 第 6 条合起来推得出）；规则没说两处（新树号的发号路径、同一 checkpoint 两个快照在第一版怎么办）。**

**候选**
- 甲 独立发布：新增 `UserChange::CreateSnapshot`，走 `publish_user_change`（先走空，G5），发一次不带文件内容的带文件版本发布（`PublishPlan` 的 `file: None`、`instance_table: Carry`，照 `publish_overwrite` 的构造但不写数据，`crates/singlefs-core/src/transaction.rs:6033` 起），重写的只有固定点角色（分配记录树节点、记账树、映射树、树表）。
- 乙 搭便车：`PublishPlan` 加「发布末尾建快照」一项，数据变更的那次发布把快照条目一起写进树表（树表本来最后装，根指针那时都已知）。

**推荐甲。** 依据：（一）不动已有每条写路径的计划结构，崩溃流多一个短发布而不是改每个流；（二）模型里「同一发布内写完再建快照」（piggy）与独立建快照都是 20000 史零违例，所以乙不是不安全，只是更贵；（三）甲的 S.txg 就是它自己的 `checkpoint_txg`，无数据变更，`birth == S.txg` 一格在这一形里不出现（用户单元的 birth ≤ S.txg − 1），把 D5 已定项 14 的「数据丢失」那一行留给后面的数据发布去顶。什么现象会推翻：用户要求「写完立刻带快照」是原子一步（乙）。

**逐问**
1. 新树号：`FileVersionTreeIdentifiers::issued_from_watermark`（「pub fn issued_from_watermark(」，`crates/singlefs-core/src/transaction.rs:2749`）一次发 13 个，之后每一版照抄；建快照要新加一个发 2 个号的入口，计划里的水位设成上一版水位 + 2（其余发布「这次一个树 ID 都不发」，`crates/singlefs-core/src/transaction.rs:6063`）。水位装不下报与现有同形的类型化错误。
2. 活头 inode 树条目的 `previous_snapshot_txg := S.txg`：今天写死 0、读回丢掉（「writer.put_u64(0); // previous_snapshot_txg：0 = 不适用」，`crates/singlefs-core/src/records.rs:461`；「let _previous_snapshot_txg = reader.get_u64();」，`crates/singlefs-core/src/records.rs:500`）。`TreeTableEntry` 加字段，`TransactionOutput` 加一个常驻标量（D5 已定项 19 的「活头头部一个标量，常驻内存」）；I-9.25 现状「inode 树的条目为 0，或 1 ≤ p ≤ 这条根的 checkpoint_txg」，S.txg 等于这条根的 checkpoint_txg，刚好在界内。
3. deadlist 一条不动：D5 已定项 12 的「建快照只推进 prev(头)」。这次发布不写 deadlist 树；deadlist 仍是零根。
4. 建快照前要不要先走空 `deleted_inodes`：不用另写，G5 已经保证。队列非空时快照里的 nlink = 0 记录这一问在这个入口上问不出来。
5. 同一 checkpoint 两个快照：D16 已定项 14 允许，第一版做不出来——每次建快照是一次独立发布，txg 各加一。⇒ C250 的「txg → 快照不单射」在第一版不出现；但 checker 里按 (txg, 快照 id) 打破并列的那条分支只有手造镜像才进得去（`fs-design.md` 五条硬要求第 2 条：每条分支要能被测试强制进入）。要的是一张手造的合法镜像（两条码 14 同 txg）与一张判红的镜像，写进 S5。
6. 拒绝：树表装满（G3 的 34）、现行版是「有树而文件集合为空」或树表 0 条的版本，各报类型化错误，都在取落点之前。

**新写出的字节**（列照 `.claude/kb/layout/03-third-txn.md@一、形态`）
| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 |
|---|---|---|---|---|
| 快照 inode 根条目（码 14） | 建快照那次发布 | 200 字节：树 ID 新发（偏移 0）、长度 200、种类 14、flags 0、根指针 = 上一版 inode 树根指针逐字抄、偏移 100 的前驱 = 建快照时活头的该字段、诞生 txg = S.txg、头 ID = 活头 inode 树的树 ID、预留全 0 | D8 已定项 8 ①③、D5 已定项 9、13 | 一·2 |
| 快照 extent 根条目（码 15） | 同上 | 200 字节：根指针 = extent 树根指针逐字抄、前驱 0、头 ID = 配对的码 14 条目的树 ID | D8 已定项 8 ①、D5 已定项 9 | 一·2 |
| 活头 inode 树条目的前驱 | 同上 | 偏移 100 的 8 字节写成 S.txg（原来是上一个快照的 txg 或 0） | D5 已定项 9、13 | 一·2 |
| 树表条数 | 同上 | 声明长度 12 × 200 = 2400 → 14 × 200 = 2800 | D8 已定项 8 | 一·2 |
| 树 ID 水位 | 同上 | 根记录里的水位 +2 | D8 已定项 8 ② | 一·2 |
| 记账行 | 同上 | 仍 15 行（快照没有 inode 号水位行）；值照固定点单元的重写照旧变 | D5 已定项 4、8 | 一·2 |
树表那 200 字节 × 2 在已经每次发布都重写的树表单元里，不新增单元。

## S3 快照下覆盖写与删文件：释放判定改在哪

**判定：一致（D5 已定项 19 的判定函数落得下）；冲突（背景材料的切口，见 G1）；规则没说两处（哪些角色问这个判定、钉住的单元映射条目留不留，见 G2）。**

**1. 切口与受影响的角色**
切口在 `placements_released_by_this_publish`（`crates/singlefs-core/src/transaction.rs:6825`）出清单之后、`publish_admitted` 落释放之前（`transaction.rs:8141`）。清单是按角色出的，不是按落点；背景材料第三节 S3 说改法落在 `placements_to_release_via_mapping` 与 `:1861` 那处 `allocator.release` 之间（转述），到了落点上角色已经丢了，所以分流要在出清单的这一层做：对 `previous_roles_replaced` 里每个角色定一个结局（立即进 defer／进 deadlist）。
- 哪些角色问判定：只有用户可见子树的六类——Data、ExtentLowerNode、ExtentUpperNodeBelowTheRoot、ExtentRoot、InodeLeafContainer、InodeRoot。理由：快照条目只冻这两棵树的根（S1）。`placements_to_release_via_mapping` 的第一个 match 臂列了 20 类要查映射的角色（「| TransactionUnit::RetirementNodeBelowTheRoot(_) => {」，`crates/singlefs-core/src/transaction.rs:3292`），其余 14 类（分配记录树、记账树、`deleted_inodes` 根、墓碑、谱系、退役各族）快照不引用，照旧进 defer；全部问判定则这 14 类永远等不到「被某个快照引用」，成为泄漏（模型里 T-leak 一类）。映射树、树表、实例表豁免映射，另走。
- 判定输入全是内存标量：birth 取被换下角色的映射 key 里出生 txg 一段（key 27 字节 = 类 1 + 出生树 8 + 出生 txg 8 + 实例 4 + 尾 6，`CodeTwoKeyFieldWidths::CENTRAL_MAPPING`，`crates/singlefs-core/src/code_two_tree.rs:40`），在 `TransactionOutput.mapped_units`（`transaction.rs:2896`）里按角色找；`previous_snapshot_txg` 从树表条目读回常驻（S2）。零次设备 I/O，与 D5 已定项 19 一致。
- 「判据输入取不到默认进 deadlist」（D5 已定项 14）在代码里的样子：结局类型 `{立即释放, 进 deadlist}` 的 `match` 默认臂写进 deadlist；今天的 `mapped_units.find(..).expect(..)`（`transaction.rs:3493` 一带）取不到会 panic，新分流处改成取不到走 deadlist。
- 覆盖写、顺序写、删除批、最后一批共用 `publish_admitted`，一处改完四处都到。

**2. 候选**
| 名 | 定义 | 新写出的字节 |
|---|---|---|
| α | 照 D5 已定项 12：deadlist 是第三棵多层码 2 树（种类 8，key 43 = 头树号 8 + 纪元 8 + 映射 key 27，条目宽 = key 宽 43，value 空）；被钉单元的映射条目留在中央映射树里（G2） | 每个被钉单元 43 字节；映射条目不新增（少删一次），分配记录不翻「已释放」；第一次追加建根，1 个 16 KiB 单元 × 2 盘；树表里 deadlist 条目的根指针从零根改成真根（同一个已重写的单元） |
| β | deadlist 条目的 value 带映射条目的两条位置项（28 字节），不留映射条目 | 每个被钉单元 71 字节；违三处条款：D5 已定项 12 的「value 空」、D5 已定项 15 射程（带提示给实现留双重释放的路）、D19 已定项 5 硬规则 1（释放一律经映射） |
α 要动：
- `code_two_tree.rs`：`CodeTwoKeyFieldWidths` 加 `DEADLIST`，字段宽依次 8、8、1、8、8、4、6，和 43。容量 376／125（「命令输出」第 1 条）。
- `transaction.rs`：`MultiLevelCodeTwoTree`（「pub enum MultiLevelCodeTwoTree {」，`crates/singlefs-core/src/transaction.rs:2341`）加 `Deadlist`，它的 5 个方法各加一臂；`CodeTwoTreeNodeCapacities::CappedForTests` 加 deadlist 一项（分裂、收缩、长高要能被测试压进去）；`TransactionUnit` 加根与根之下节点两类角色，涉及的 match 与记账树同量级（对 `transaction.rs` 数 `AccountingTreeNodeBelowTheRoot` 的出现次数 = 15，「命令输出」第 2 条）；`resolve`（`transaction.rs:5464`）在记账树之后、删除两棵之前插 deadlist 角色（次序「记账树（树 65539）在前、删除的两棵（65544、65545）接着」，`crates/singlefs-core/src/transaction.rs:5538`，deadlist 是 65543）；`plan_the_multi_level_trees`（`transaction.rs:5625`）加 deadlist 计划与「被钉映射条目」集合；树表装配（「let tree_table_entries = vec![」，`crates/singlefs-core/src/transaction.rs:8649`）里 deadlist 一条今天写零根（`trees.deadlist,` 在 `transaction.rs:8688`），改写真根；`TransactionOutput` 加 deadlist 树版本、被钉映射条目、`previous_snapshot_txg`、快照条目列表。
- `recovery.rs`：`rebuild_version_in_the_unit_area_starting_at`（读回三个树 ID 在 `:2583` 起）加读 deadlist 树与被钉映射条目（照记账树）；`mounted_read.rs`、`walk_to_file` 里的映射读照旧（被钉条目就在映射里）。
- `admission.rs`：环窗口 W 式子（「pub fn slots_one_deletion_publish_newly_takes_on_one_device_at_most(」，`crates/singlefs-core/src/admission.rs:404`）加 deadlist 一项；一次发布追加 A 条、同一次发布的纪元相同，条目集中在一个纪元段里，新占槽至多「A 除以 376 向上取整」加 1 个叶，再加每层路径一槽（推的，没量）。
- checker：`field_table_width_of_tree_kind`（`crates/singlefs-checker/src/walk.rs:287`）加种类 8 一臂；I-3.5 ③④、I-3.6 的判见 S5。

新写出的字节（α，列照 `.claude/kb/layout/03-third-txn.md@一、形态`）
| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 |
|---|---|---|---|---|
| deadlist 条目 | 建快照之后第一次换下一个 birth ≤ 前驱的用户可见单元（覆盖写或删除批） | 43 字节，无 value：头树号 8（活头 inode 树的树 ID）、纪元 8（杀块时头的前驱）、映射 key 27（类 1、出生树 8、出生 txg 8、实例 4、尾 6）；按这个字段次序逐字段无符号整数比较 | D5 已定项 12、19；D19 已定项 6、10 | 一·2 |
| deadlist 根（码 2） | 同上（树由零根变成有根） | 码 2 节点，key 宽 43、条目宽 43，一节点叶 376、内部 125；树 ID 65543（树表里它的条目由零根指针改成这个根） | D5 已定项 11、12；D8 已定项 11 | 一·2 |
| 被钉映射条目 | 同上 | 不新增字节：被换下角色的 55 字节映射条目不从新版映射树里去掉，之后每版照抄 | D19 已定项 5、8 | 一·2 |
| 分配记录 | 同上 | 被钉单元的记录不翻「已释放」（少改 A 条）；立即释放的照旧翻 | D3 已定项 7 | 一·2 |

**3. 一次发布最多追加多少条**
被换下的用户可见角色数：D 个数据单元 + 下段叶节点（每 144 个单元一片，层数再高时加内部节点）+ extent 上段根 + 1 片 inode 叶容器 + inode 根。一单元文件（数据指针内联，无下段）是 Data、ExtentRoot、InodeLeafContainer、InodeRoot 4 条；三单元文件是 3 + 下段叶 1 + 上段根 1 + 2 = 7 条；删除批 K = 144 时至多 144 + 约 6。上界来自单元个数，实际上限由一次发布的 journal 记录数封：默认环下 805306368 ÷ 4096 ÷ 3 = 65536 条（「命令输出」第 3 条；「pub const JOURNAL_SAFETY_FACTOR: u64 = 3;」，`crates/singlefs-format/src/lib.rs:256`），一个数据单元一条记录，所以 A ≤ 65536 + 节点数。钉住的单元总数有另一个上界：只有建快照那一刻活着的单元才会被钉，每个单元死一次，总数 ≤ 快照点上用户可见单元数之和。

**4. 记账哪几个统计量变**
已分配不变（被钉单元留在「已分配」，不翻「已释放」）；defer 待释放（第 5 项）不变（钉住的不进 defer，立即释放的照旧进）；第 3 项不可回收仍恒 0（「第一版不跑 zoned ⇒ 恒 0」，`crates/singlefs-core/src/admission.rs:235`；这段注释从 `:234` 起，说活快照钉住的是 zone 粒度、只在 zoned 上非零）；第 4 项待删占用仍恒 0（注释「不销毁快照」在 `admission.rs:239`，实现销毁之后这半句要改，数不变，理由同「本来就在已分配里」）；第 12 项水位没有快照行；第 6 项已承诺预留不变，环窗口 W 式子加 deadlist 一项（不是已发布统计量）。D5 已定项 4 第 8 项的撤回依据照旧：第一版不提供每快照用量，`df` 只报全池。可观测：钉住单元数与字节数做成内存里的运行时计数（不落账），满足 `fs-design.md` 硬要求 4。

**5. 映射条目留存怎么做（G2）**
α1：留在同一棵中央映射树里。被换下的用户可见角色进 deadlist 时，从上一版映射取它的两条位置项（`mapping_locations_of_a_mapped_unit`，「fn mapping_locations_of_a_mapped_unit(」，`crates/singlefs-core/src/transaction.rs:3488`，上一版映射树整棵在内存，零 I/O），记进 `TransactionOutput` 新字段「被钉映射条目」（按 key 排），之后每次发布按 key 照抄进新版映射树，销毁快照的那一批放掉它时一并删除。挂载重建：被钉条目 = deadlist 里每个 key 在映射树里查到的那一条（查不到是 C300 的情形，报判不了）。映射树条目数 +P（P 为被钉单元数），映射树容量准入（D19 已定项 5「挂载态怎么读映射」一段）自然把 P 算进「现在的条目数」。
α2：被钉映射条目另开第四棵树。多一个树号、多一族角色，换到的只是映射树的读者不看见被钉条目；不推荐，不列入岔路。

**6. 模型对判定函数与销毁的核对**（`research/prompts/m4-snapshot-r1-sonnet-model/deadlist_model.py`，输出 `model.out`，「模型输出」一节）：照 D5 正文（纪元 = 杀块时头的前驱，销毁时 prev 传递并带上界 hi）20000 个 40 步的随机历史（含销毁中途交错建快照、写、杀）零违例；判据写成 `birth >= head_prev`（边界取错）12135 史丢数据；销毁不传 prev 11585 史泄漏或紧界不成立。这是在我自己的模型上量的，没被攻过。

**推荐 α。** 依据：条款一致（D5 已定项 12、D19 已定项 5 与 8）、β 要重开三条；α 的额外代价集中在「映射条目留存」一处，它同时是后面 GC 搬被共享的块（二·1）要的前提（D19 已定项 8 射程：搬迁只改映射一条条目）。什么现象会推翻：二·1 里 GC 另有不经映射找落点的路径（则留存的消费者少一个，但销毁快照放块仍要它）。

**要交用户的岔路（S3）**
| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| α 映射条目留存（每个被钉单元稳态 43 + 55 = 98 字节）／β deadlist 条目带位置项（71 字节，重开 D5 已定项 12、15 与 D19 已定项 5） | 被钉单元数的峰值 U：α 比 β 多 27 × U 字节；峰值取出口 4 随机历史里「快照存活期间被覆盖的单元数」（跑一遍随机历史数 deadlist 条目峰值即得） | 用户愿不愿意为这个差额重开三条条款 |

**7. 这个分支自己要过 `fs-design.md` 的硬要求**：「2. **每条分支必须能被测试强制进入。**」（`.claude/rules/fs-design.md:122`）。「立即释放／进 deadlist」是一个有显式判据（birth > 前驱）的分支：要一个只供测试的开关（放在 `DeletionSwitches` 同处）能无视判据强制走某一条，必红世界（S5、S6）靠它造；运行时计数两条各走了几次（硬要求 4）。

## S4 删快照：意图、分批、deadlist 并到更新的一侧

**判定：一致（D5 已定项 12、13、16、20 与 D8 已定项 5 硬要求第 4 条合起来给得出流程）；规则没说三处（意图住哪、下一侧的上界要不要、被销毁的快照在 checker 里怎么算活）；C195 的条款已补（D5 已定项 13「`prev` 要跟着传递」），checker 一半仍欠。**

**意图载体候选**
| 名 | 定义 | 装下 (头, lo, hi, P, 续做点) 五样要几字节 | 要动 crates/ |
|---|---|---|---|
| 甲 | 销毁中记号写在快照自己的码 14 条目上：flags 一位 + 预留里 hi 8 字节 + 续做点 27 字节 | 条目里已有头 ID（头）、诞生 txg（lo = S.txg）、前驱（P）；新增只有 hi 8 + 续做点 27 = 35 字节 + 1 个 flags 位，落在快照条目预留余量 52 字节之内（预留 76，快照条目只认购谱系标号 16 + children 8 = 24，余 52） | `records.rs` 的 `TreeTableEntry` 读写与 flags 的放行、预留认购表加一行；`recovery.rs` 重建时读回；其余同下 |
| 乙 | 住 logged_ops（树 12，今天零根注册、没有字段表、没有写者：「字段表未定」在 `crates/singlefs-core/src/transaction.rs:8694` 一带） | 8 + 8 + 8 + 8 + 27 = 59 字节加记录头 | 要等 `m4-gc-r1` 定字段表（C364），两轮在字段表上耦合 |
| 丙 | 同甲但不存续做点，每批从 lo 重扫区间 | 8 + 1 位 | 少 27 字节；区间里 birth ≤ P 的条目永远留着，每批都要重扫，总工作量对条目数是平方 |
unlink 的先例是「意图 = nlink 为 0 的记录，不立 logged_ops」（「不立 logged_ops 条目」，`.claude/kb/layout/03-third-txn.md:9`），甲是同一个做法。队列是派生态：带销毁中记号的码 14 条目集合（树表整节点常驻内存，扫 ≤ 34 条，C336 的三选一取「挂载时扫树表建派生态、零字节」，成本是 O(34) 而不是 D5 欠账里 1024 个头时的上万条）。
**推荐甲。** 依据：与 unlink 同形、不依赖另一轮的字段表、续做点保住线性工作量。什么现象会推翻：`m4-gc-r1` 定的 logged_ops 字段表恰好把「按头分的续做点」做成通用形（则乙省一份实现）。

**一次销毁的流程（甲）**
- 意图发布：S 的码 14 条目置销毁中记号、写 hi；下一侧的前驱同一次发布改成 P（下一侧是头：活头 inode 树条目偏移 100 := P；是快照：它的码 14 条目偏移 100 := P）。D5 已定项 13「不许拆到以后的发布」靠这一点成立。S 此刻起不算活引用（D5 已定项 20 的例外句「例外：被销毁的快照从写意图那一刻起不算活引用」，`.claude/kb/decisions/05-快照-空间记账机制.md:433`）。
- 一批：从续做点往后，在纪元落在 [lo, hi) 的条目里取至多 K 条 birth > P 的；每条先核还在（D8 已定项 5 硬要求第 4 条），再按映射条目读盘核校验和（D19 已定项 5 硬规则 1，每个单元每份副本读一次，一批至多 2K 次读），删 deadlist 条目、删被钉映射条目、`allocator.release_leaving_the_record_allocated_on`（进 defer），续做点前移，不改任何剩下条目的键。批大小 K 取 144（`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`，与 unlink 批同值，W 式子里的 K 不用另算）；只供测试的开关加在 `DeletionSwitches`（「pub struct DeletionSwitches {」，`crates/singlefs-core/src/unlink.rs:28`）里，形状同 `UnlinkBatchDataUnitsCappedForTests`（1 到 144）。
- 最后一批：区间里剩下的 ≤ K 条放完，同一次发布删 S 的码 14、15 两条（树表 14 → 12 条）并清记号。
- 空间不够：并进现有「走空」循环（`mounted_session.rs` 的 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`，S2 的 `UserChange` 分发处），被拒推抬 F 再试，预算同 unlink（一次准入至多 8 次发布）。放块进 defer，空间要等回收才回来，这与 unlink 同形。

**hi 要不要存（模型的结果，「模型输出」一节）**
- 下一侧是快照时 hi 不可省：上界一律取无穷的变体 17090 / 20000 史丢数据（串行纪律下也 13349 史）。原因：纪元 ≥ 下一个快照 txg 的条目是下一个快照引用的，不在这次销毁的区间里。
- 下一侧是头时：销毁做完之前允许交错建快照／写／杀，省 hi 的变体 5505 / 20000 史丢数据；销毁做完之前别的改动一律不接（串行），省 hi 的变体 20000 史零违例。⇒ D5 已定项 12 写的「是头时取写意图那个开放 checkpoint 的号」在串行纪律下可省 8 字节，不串行就不能省。
- 照 D5 正文（存 hi、传 prev）的变体，交错与串行两种都 20000 史零违例。

**崩在中间每个状态，I-3.5 ③ ④、I-3.6 成不成立**
| 状态 | ④ 头引用的块不在任何 deadlist | ③ 不被头引用的块恰在一个 deadlist，区间 = (Reach 最晚者, 下一个根] | I-3.6 紧界 birth ≤ prev(份) |
|---|---|---|---|
| 意图之前 | 成立 | 成立 | 成立 |
| 意图之后、批之间 | 成立 | **不成立**：待放块的 Reach（只算活快照）为空，算不出区间 | **不成立**：待放块 birth > P，而下一侧的前驱已改成 P |
| 最后一批之后 | 成立 | 成立 | 成立 |
⇒ ③ 与 I-3.6 在中间态要带一条例外：「落在未完成意图 (lo, hi, P) 里、birth > P 的条目」可以存在，并且续做点之前不许再有这样的条目（已放的段不回来）。例外集合从码 14 条目上的销毁中记号读，所以这些字段必须是 checker 读得到的盘上字节（甲、丙行，乙行要等 logged_ops）。C337 的「何时不算活引用」checker 侧即：带销毁中记号的码 14 条目不进 I-3.7 的活集合、不进 Reach。
崩溃恢复落到意图之前的根：S 还活着，没有任何痕迹，免费成立。落到某一批之后的根：一批是一次发布，deadlist 叶、映射条目、分配记录同在那次发布里，一致。重跑已做完的批：条目已不在，「核还在」跳过，幂等（模型 `destroy_batch` 每次重取待放集，同一批重跑零违例）。

**新写出的字节**（列照 `.claude/kb/layout/03-third-txn.md@一、形态`）
| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 |
|---|---|---|---|---|
| 销毁中记号 | 意图那次发布 | 码 14 条目 flags 位 0 置 1；预留里 hi 8 字节 + 续做点 27 字节（写意图时续做点 = 区间起点的最小 key） | D5 已定项 12、13、20 | 一·2 |
| 下一侧前驱传递 | 意图那次发布 | 下一侧条目偏移 100 的 8 字节 := P（头：inode 树条目；快照：它的码 14 条目） | D5 已定项 13 | 一·2 |
| 一批 | 每个批发布 | deadlist 叶（至多 ⌈K ÷ 376⌉ + 1 片）与路径重写、映射树叶重写（删 K 条）、K 条分配记录翻已释放、记账 defer 行、树表里续做点 27 字节更新 | D5 已定项 12、16、19；D19 已定项 5 | 一·2 |
| 最后一批 | 同上 | 另删码 14、15 两条（树表 14 → 12 条，声明长度 2800 → 2400）、清记号 | D5 已定项 12 | 一·2 |

**要交用户的岔路（S4）**
| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 销毁快照期间别的改动：先把销毁做完再接（串行，意图里可省 hi）／允许交错（意图存 hi，崩溃流多一类「销毁中穿插写、建快照」） | 一次销毁的批数 = ⌈被钉条目数 ÷ 144⌉，条目数 ≤ 快照点上用户可见单元数；批数 × 每批发布时间 = 别的改动被推迟的时长（每批发布时间要量，出口 4 的长跑能给） | 用户给出可接受的最长推迟，或要求销毁与写并发 |
意图载体（甲、乙、丙）不交用户：甲与 unlink 同形，乙要等 `m4-gc-r1` 的字段表，是两轮之间的协调项，交主 agent。
什么现象会推翻整个 S4 推荐：D5 已定项 12 的「级联之间不要次序」被攻方打成需要次序（则一批要带「更早的级联做完」检查，批数不变、每批多一次条目核对）；或预留认购表里「销毁意图」这一行与别的认购者撞（续做点 27 字节在预留里放不下）。

## S5 checker：I-3.5 ③ ④、I-3.6、I-3.7 的判法与必红世界

**判定：规则没说（I-3.6、I-3.7 在 checker 源码里零命中：对 `crates/singlefs-checker/src` 与 `crates/singlefs-checker-tier/src` 数 `I-3.6`、`I-3.7` 出现次数 = 0，「命令输出」第 5 条）；I-3.5 的 ③ ④ 现文可落，要补两样输入（销毁中的待放集、被钉单元的遍历）。**

**共同的输入与独立性**
- 一趟遍历（与 I-3.1 同一趟，G6）：对回退候选集里每一版 V，取 V 的树表、deadlist 树、映射树。引用集合 Ref_头(V) = 活头两棵树的子树单元；Ref_S(V) = 每个未带销毁中记号的码 14 条目（配对的码 15）的两棵子树单元；Reach(b) = { S : b ∈ Ref_S }。遍历方向对三条不变量判 ③ ④ 与 I-3.7；扫描方向（D13 已定项 12 的第二个方向：全盘按块头重建，两个方向对同一条不变量独立判断，共享的只有格式解析）判 I-3.6：对每条 deadlist 条目读被钉单元自己的头，birth 取头里的诞生代号，不取 key 里那一段（key 与头不一致是 C300 那一类的另一条判）。
- 输入取不到时报什么：D5 已定项 14 的「默认进 deadlist」是写侧纪律；checker 侧对应的是现有 I-3.5 的写法——「R 或 R⁻ 的引用集合数不全…时整对报不适用」（`crates/singlefs-checker/invariants/05-空间记账.md:11`）：deadlist 树读不出、被钉单元头读不出、某条 deadlist key 在映射里查不到、码 14／15 解不开，一律整条报「不适用并带理由」，不报绿也不报红；报绿会让 checker 恒真，报红会把单元丢了的盘误判成实现错。

**每条的可判定形式**
| 不变量 | 形式 | 判哪几版 | 必红世界 |
|---|---|---|---|
| I-3.5 ③ | ∀ b：b ∉ Ref_头 且 Reach(b) 非空 ⇒ b 在 deadlist 里恰一条（按映射 key 对），其纪元 e 落在「Reach(b) 中最晚者的下一侧」那一份的区间 [prev, txg) 里（下一侧是头则 [prev(头), ∞)）。反向：deadlist 的每条 b 都满足 b ∉ Ref_头，且 Reach(b) 非空或 b 在待放集里 | 候选集每一版 | deadlist 里放一条 birth > S.txg 的（题面例）：它不在 Ref_S 里，Reach 为空又不在待放集 ⇒ 红 |
| I-3.5 ④ | ∀ b ∈ Ref_头：b 不在任何 deadlist 里 | 同上 | 把仍被活头引用的块的 key 追加进 deadlist ⇒ 红 |
| I-3.6 | ∀ deadlist 条目 e，所属份 X（按纪元落进的区间）：birth(e) ≤ prev(X).txg；待放集里的除外（S4 的例外） | 同上，扫描方向 | 把 birth > prev 的块放进 deadlist（题面例：覆盖写两次，第二次的单元误进 deadlist）⇒ 红 |
| I-3.7 | （一）每个码 14 恰有一个头 ID 指着它的码 15，反之亦然；（二）把活快照按 (诞生 txg, 树 ID) 排序，每条码 14 的偏移 100 前驱 = 排在它前面那条的诞生 txg（最前一条 = 0），活头 inode 条目的偏移 100 = 最后一条的诞生 txg（没有快照 = 0）；（三）诞生 txg ≤ 这条根的 checkpoint_txg | 同上 | 树表里少一条码 14（码 15 留着）⇒ 配对红；两条都少而下一侧前驱没改 ⇒ 链红；两条都少且链也改齐 ⇒ 被它独占的单元在 deadlist 里却 Reach 为空 ⇒ I-3.5 ③ 红 |
最后一行说明 I-3.7 的「既不多也不少」靠的是链加 ③ 的合力：单靠树表本身没有第二个见证（D5 已定项 21 射程说「全部根的 txg 列表（没有）」，这就是它的落点）。
另加三条不在题面里、却是上面要成立的前提的判：C300 的「deadlist 每条 key 在映射里查得到」（必红：删掉一个被钉单元的映射条目）；销毁中记号与前驱传递一致（必红：下一侧前驱 ≠ S 的前驱，D5 已定项 13 的「不许拆到以后的发布」）；销毁中记号的续做点之前不许再有待放条目（必红：续做点往前挪而条目没删）。
独立性的弱点（给攻方）：I-3.6 用单元头的 birth 判，I-3.5 ③ 的 Reach 用遍历判，两者不共享 deadlist 的 key；但遍历与 I-3.1 共用同一趟，I-3.1 与 ③ 在「被钉单元被误释放」一格上同红，不是独立见证两次。

S2 第 5 条要的两张手造镜像（两条码 14 同诞生 txg 的合法版，按树 ID 打破并列；前驱与排在前面那条对不上的坏版）进 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 这一类用例。

**GPU 判器违例位**：现状 128 位、已用 92（`IMPLEMENTED_INVARIANTS` 长度 92，「pub const IMPLEMENTED_INVARIANTS: [&str; 92] = [」，`crates/singlefs-checker/src/image.rs:112`；容量「pub const POOL_CHECKER_VIOLATED_WORDS: usize = 4;」，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:79`）。新位：I-3.6、I-3.7、C300 映射可查、销毁意图一致共 4 位；I-3.5 ③ ④ 并进下标 65 那一位（同一条不变量，不另占）；若 ③ ④ 另占新位（出现规则留着 65 那一位）再加 2 位，共 6 位。⇒ 用到 96 至 98，剩 32 至 30；背景材料写「剩 36 位」是 128 − 92，没算这一轮新增。GPU 内核要新写 deadlist 树（多层码 2 树）的走读与 Reach 的遍历，`crash_judge_checker.wgsl` 与 `crash_judge_gpu.rs` 两处，这是 S5 里最大的一块实现量（推的，没量）。
**什么现象会推翻**：把「被钉单元」的遍历并进 I-3.1 之后 I-3.1 仍有一格对不上（比如销毁中的待放条目没算），则 ③ 的待放集例外不成立，要重拆。
补一条读法：I-3.5 现文 ③ 的区间写成死亡时刻的 (Reach 最晚者.txg, 其下一个根.txg]，D5 已定项 12 的纪元写成杀块时头的前驱；两者等价（杀块时最新的快照就是 Reach 最晚者 P，纪元 = P.txg，所属份 = 前驱为 P.txg 的那个快照，其区间 [prev, txg) 的下界正是 P.txg），表里按纪元写，不改现文。

## S6 崩溃流与态数

**判定：规则没说（态数怎么估没有现成口径，下面用闭式 1 + Σ(2^段写数 − 1) 估，推的）；一致（三条流都能接在多记录流后面，照 unlink 流的接法）。**

**今天的口径**
- 闭式：每次写取两态（没持久／持久）的全量态数 = 1 + Σ(2^|段| − 1)（「E77（发布的持久顺序） 的闭式：1 + Σ(2^|段| − 1)，每次写只取两态（没持久 / 持久）时的全量。」，`crates/singlefs-harness/src/memory_pool.rs:622`）。
- 已有流的段形状：「PROBE_SEGMENT_SIZES [2,2,1,2,2,1,2,24,2,1,2,28,4,1,2,30,6,1,2,26,2,1,2,24,2,1,2,22,2,1,2,20,2,1,2]」（`scripts/crash/crash-amplification.sh:51`）。前 19 段是多记录流（取号、暖机两次、A、B、C），其中 24、28、30 是 A、B、C 三次发布的单元段（12、14、15 个单元，每个单元两盘各一写）；后 16 段是 unlink：意图 26、两批 24、22、最后一批 20（写数；该测试文件头写的是意图 13、两批 12 与 11、最后一批 12 个单元，最后一批 12 个单元该是 24 写而不是 20，说明注释里的形状之后改过）。按这个形状代入闭式得 1 448 083 588（「命令输出」第 4 条），而该测试文件头写的是 1463812228（「每次写只取两态时的闭式 1463812228」，`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:23`）：差 15 728 640，这份段形状注释是第 32 批的，之后写表改过；我只用闭式估数量级，不引它的绝对数。
- 上限：「取值：登记分项里最长一段的写数。今天是 30 写」（`scripts/crash/crash-amplification.sh:50`）；「FULL_EXPAND_UP_TO=30」（`scripts/crash/crash-amplification.sh:319`）；超过它的段报 exhaustive=false 并要按段登记（`.claude/gate.d/crash-point-replay-segments.tsv`）。每落点 64 次写、每流 2048 次写：快照流的发布不超过十来次，远在两者之下（一次发布写数不到 40，十次不到 400）。

**三条流（接在「取号 → 暖机 × 2 → A」之后；文件取 A 的 1 个数据单元，理由见下）**
| 流 | 发布序列 | 每次发布的单元段（写数，推的） | 闭式态数（只算大段） |
|---|---|---|---|
| F1 建快照 | A，建快照 | 建快照：8 个固定点单元 = 16 写（分配记录树 5 个节点，4 GiB × 2 的池上每次发布都带着，「分配记录树按位置寻址（D8（核心索引结构） 已定项 14），4 GiB × 2 的池上一次发布带着它的五个节点」，`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:10`；加记账树 1、映射树 1、树表 1） | 65 535 |
| F2 快照下覆盖写与删文件 | A，建快照，覆盖写（1 单元文件）；另一条 A，建快照，unlink（批大小 1） | 覆盖写：1 单元文件的覆盖写是 12 个单元（24 写，A 的形状）；快照下多 deadlist 根 1 个单元 = 13 个单元 = 26 写。unlink：批把被钉单元记进 deadlist，同量级 | 67 108 863（2 的 26 次方减 1）；unlink 那条与 unlink 流同量级 |
| F3 删快照 | A，建快照，覆盖写，销毁（意图，批大小 2 的两批含最后一批） | 意图：8 个单元（同建快照）= 16 写；批：分配记录树 5 + 记账 1 + 映射 1 + 树表 1 + deadlist 1 = 9 个单元 = 18 写；deadlist 清空的最后一批是 8 个单元 = 16 写 | 65 535 + 262 143 + 65 535 = 393 213 |
结论：三条流的最大段是 F2 覆盖写的 26 写，不超过 `FULL_EXPAND_UP_TO` = 30，也不超过已有 unlink 流的最大段；三条流加起来比一条 unlink 流小。
**文件取 3 个单元就不行**：C 的覆盖写是 15 个单元 = 30 写（`PROBE_SEGMENT_SIZES` 第 16 段，已是 30 的上限）；快照下再加 deadlist 根 1 个单元 = 16 个单元 = 32 写，超出 `FULL_EXPAND_UP_TO`，要么抬到 32（2 的 32 次方减 1 = 4 294 967 295 个态，比整条 unlink 流的 1.45 × 10 的 9 次方还大约 3 倍）要么按段登记。⇒ 快照流的文件取 1 个数据单元；3 个单元的快照下覆盖写另立一条流并先登记段，不进全域。
**每次发布写几个单元、一段几次写**：已列在上表；一次发布 = 单元段（每个单元两盘各一写）+ journal 记录段 + 根槽 + 系统配置槽轮换，与 unlink 流相同。

**必红世界（每条流一个）**
- F1：建快照那次发布不推进头的前驱（只写码 14、15 两条，头 inode 条目偏移 100 仍是 0）⇒ 建快照之后的每个态 I-3.7 链红。
- F2：覆盖写把被钉的 4 个用户可见单元放进 defer 而不是 deadlist（用 S3 第 7 条的强制开关造）⇒ I-3.1（遍历到已释放的槽）与 I-3.5 ③ 同红。
- F3：销毁的一批把 birth ≤ P 的条目放掉（丢方向）⇒ 批之后的态 I-3.1 红；或销毁意图不传 prev（漏方向）⇒ 最后一批之后 I-3.6 红（模型里这两格对应 `lt` 与 `noprev` 两个变体，都判红）。
- 挂载续做观察者：每个态上可写挂载一次，续做把销毁队列走到空（照 unlink 的 `observe_the_mount_continuation`），续做步数 = 恢复落到的那一版还差几步；这一格在 F3 才有。

**要动的文件**：新三个测试文件（照 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`）；公共模块 `common_crash_points` 加建快照、销毁快照的步；登记表 `.claude/gate.d/crash-point-replay-items.tsv` 加三项（含 `crash-case:` 行，里程碑文件一·4）；`scripts/crash/crash-amplification.sh` 的分项表。后两样是门禁表与脚本，归主 agent 登记，不是 crates/。
**什么现象会推翻**：分配记录树的节点数不是 5（5 是 4 GiB × 2 的池上的数，文件头写明；池大小变它就变）；或快照下覆盖写的单元数达到 16 以上（则 F2 超 30 写，要登记段）。这些数要在实现落地后用写表真数，我这里是从 `resolve` 的角色清单与上面那份文件头推的，没编译没量。

## S7 模型、RefFS 与随机历史

**判定：规则没说（条款只在里程碑文件一·2 的验收写「模型对拍加建快照、删快照两种操作」；三处代码里快照面全无：`ModelOperationKind` 12 种、RefFS 的 `Operation` 无快照、`HistoryOperation` 12 种，背景材料第二节第 7 条的观测我逐处核了，一致）。**

**RefFS（`crates/singlefs-reference-filesystem/src/lib.rs`，「pub enum Operation {」在 `:31`）**
- 加 `CreateSnapshot`、`DestroySnapshot(号)` 两个操作，读快照做成查询（按号给回创建那一刻的内容）。状态：每个快照 (号, 内容, 是否已销毁)。
- 语义：建快照成功返回抬下界 lo（与发布点同）；快照内容是创建那一刻的 `VisibleState`，之后覆盖写不改它；销毁的意图发布是对外可见的一步（那一刻起读不到），批在后台、对外不可见 ⇒ RefFS 不需要批状态，与 unlink 同形。崩溃后的允许集合从「状态」变成「(状态, 快照集合)」对：恢复到哪个版本前缀，快照集合就是该前缀里已建未销毁的。
- 量：约 150 至 250 行（推的，参照现有 1079 行里 `RollBackToVersion` 一族的体量）。
**IdealModel（`crates/singlefs-harness/src/model.rs`，「pub enum ModelOperationKind {」在 `:978`，4622 行）**
- `ModelOperationKind` 加 `PublishCreateSnapshot`、`PublishSnapshotDestroyIntent`、`PublishSnapshotDestroyStep`；读快照加进 `ModelReadBack`（`:966`）一个成员。
- `ModelRoot`（`:528`）加：快照列表（号、txg、前驱、销毁中的 hi 与续做点）、活头前驱、deadlist 条目数、被钉映射条目数、deadlist 树节点数上界（照 `accounting_tree_nodes_upper_bound`）。模型已经按角色记「哪次发布写的」（`role_written_at`），所以「被换下单元的 birth ≤ 前驱」模型自己算得出，不需要新记 birth。
- 新增 `answer_create_snapshot`、`answer_snapshot_destroy_intent`、`answer_snapshot_destroy_step`；改 `answer_publish_overwrite`、`answer_deletion_step`（单元数多 deadlist 根或叶、映射条目数不减）；`answer_rollback_while_mounted`（`:2495`）在有快照时必须拒（S8 推荐形），并在 `ModelRefusalReason` 加一个成员。
- 量：约 500 至 700 行（推的，参照 `answer_unlink_intent`、`answer_deletion_step`、`answer_create_inode` 三个函数加 `ModelUnitRole` 的体量）。
- 波及面（命令输出第 6 条）：`ModelOperationKind::` 只在 `model.rs` 出现 18 处，各 `match` 要加臂。
**随机历史（`crates/singlefs-harness/src/history.rs`，「pub enum HistoryOperation {」在 `:475`）**
- 加 `CreateSnapshot`、`ReadSnapshot(选择)`、`DestroySnapshot(选择)` 三种（12 → 15）。波及面（命令输出第 6 条）：`HistoryOperation::` 出现 `history.rs` 93 处、`random_histories.rs` 61 处、`e158_root_choice_repair.rs` 38 处、`crash_injection_campaign.rs` 27 处、`unlink_in_histories_judged_by_the_model.rs` 26 处、`reference_filesystem_translation.rs` 15 处（历史到 RefFS 的翻译）、`fault_injection.rs` 13 处、`crash_injection.rs` 12 处。
- 生成权重：每个新操作进它自己那一组比重，已有各组生成的历史不变（`WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros` 的文档注释就是这么做的），否则现有种子的历史全被改写，旧的复现号作废。
- `KNOWN_RED_FORMS` 不新增：「pub const KNOWN_RED_FORMS: [KnownRedForm; 0] = [];」（`crates/singlefs-harness/src/history.rs:2237`）。「不新增」的判据写成：新操作进了生成器之后，N 个种子零红才算落地；红了修实现或修模型，不往清单里加。
- 有快照时 `RollBackWhileMounted` 生成器仍可生成，模型答「必须拒」，执行器的拒绝要对上，不算红。
**什么现象会推翻**：RefFS 的允许集合不能用 (状态, 快照集合) 对表达（比如快照内容要跨崩溃保留历史版本而不是前缀）；那样 RefFS 要多记「每个版本的快照集合」，量翻倍。

## S8 回退、冷启动与快照

**判定：规则没说（D23 已定项 14 的管理员回退没写有快照时怎么办）；冲突一处：背景材料的「树 ID 集合不同今天拒」覆盖不到快照；一致：崩溃恢复落到更早的根对建快照和覆盖写免费成立。**

**今天的拒绝覆盖不到快照**
- 「if target_version.tree_identifiers != current.tree_identifiers {」（`crates/singlefs-core/src/mount.rs:8083`）比的是 `FileVersionTreeIdentifiers` 那十三个固定号（「pub struct FileVersionTreeIdentifiers {」，`crates/singlefs-core/src/transaction.rs:2703`）。快照条目的新树号不在这个结构里（S1），所以回退遇到快照时这一条拒绝不会触发——它不是「处置未定」被拦住，是根本看不见。
- 回退发布释放的是「cur 引用的用户可见单元里，落点不在 R_old 引用的那几个里的」（「释放集：cur 引用的用户可见单元里，落点不在 R_old 引用的那几个里的」，`crates/singlefs-core/src/mount.rs:8108`）。快照在场而回退到快照之前的一版时，cur 的用户可见单元里有只被快照钉着的（它们此刻在活头里、也在快照里），这个释放集会把它们放进 defer，快照丢数据。⇒ 不加新规则就是数据丢失，不是泄漏。

**候选**
| 名 | 定义 | 要动 crates/ | 新写出的字节 |
|---|---|---|---|
| r1 | 拒：现行版或回退目标版的树表里有码 14／15 条目即拒（目标版有而现行版没有的情形：快照在两版之间被销毁，它的钉住单元可能已放，也拒） | `mount.rs` 的 `roll_back_by_a_forward_publish`（`:7998`）在 `:8083` 那一判旁加一判，`RollbackError` 加一个成员；`model.rs` 的 `answer_rollback_while_mounted`（`:2495`）与 `ModelRefusalReason` 各加；历史执行器对拒绝 | 0 |
| r2 | 条件放行：目标版的 txg 不低于最新活快照的 txg 且两版快照集合相同时放行；新根的 deadlist 树与被钉映射条目取目标版的，不取现行版的 | 同 r1，另改 `version_to_publish_the_rollback_after`（把 deadlist 与被钉映射条目从目标版带过来）、复活与释放两步照旧但被钉单元不进释放集；新增崩溃流 | 回退那次发布多重写 deadlist 树的节点（目标版的整棵换回），其余 0 |
| r3 | 抹掉：回退把目标版之后建的快照一并销毁 | 回退变成「意图加分批」的多次发布，D23 已定项 14 的「挂着时的一次向前发布」要重开；销毁级联在回退里再走一遍 | 同 S4 的批，批数随被抹快照的钉住条目数 |
r2 的安全条件我推了一遍（推的，没造镜像）：目标版不早于所有活快照，则任何被快照引用且活头仍引用的单元，在目标版里也被活头引用（线性历史里一个单元活到现在就一直活着），所以回退的释放集（cur 活着而目标没引用的）碰不到被钉单元；而 cur 的 deadlist 里「回退之前被杀、回退之后又活了」的条目必须消失（否则 I-3.5 ④ 红，以后销毁快照会放掉活头引用的块——数据丢失），取目标版的 deadlist 树恰好做到。
**推荐 r1（第一版）。** 依据：D6 已定项 4 已经写了「要回到某个快照的状态就从那个快照克隆一个新的可写头」，快照在场时的管理员回退不是用户路径上的主操作；r1 零字节、一处判；r2 多一条要被攻的安全条件，r3 重开 D23。什么现象会推翻：用户要求有快照时管理员回退仍可用（翻到 r2）。

**回退候选集与 I-3.5 出现规则的「按树继承下界」有快照时怎么读（推的）**：每个候选版的引用集合并入它自己的快照子树（S5 的 Ref_S）；(b) 款里「用户可见树」的列表（extent 树、inode 树、`deleted_inodes` 非全零之后加它，`crates/singlefs-checker/invariants/05-空间记账.md:11`）不加码 14／15：快照子树里的单元都是更早的头版本里就出现过的单元，在线性历史里相邻候选对 (R⁻, R) 之间它们不是「新出现」，走 (a) 款 birth > R⁻.txg 或已在 R⁻ 的引用集合里即可。回退造成的缺口（R.txg ≠ R⁻.txg + 1）照现有 gap 条款，快照条目不改它。

**崩溃恢复落到建快照那次发布之前的根：免费成立**（一致）。建快照那次发布不写 deadlist（D5 已定项 12，S2 第 3 条），快照条目随根 COW，恢复选了更早的根就没有这两条；那次发布新取的落点是孤儿，走现有的孤儿清扫。落到快照之后的任一根：deadlist 叶、被钉映射条目、分配记录同在那一根的发布里，一致。树号水位取根环里全部根的 max（含被抛弃时间线，D8 已定项 8 ②），快照树号不会被重发。
**不免费的一格**：恢复读 deadlist 树或映射树的某个节点读不出，是 C300 那类「判不了」的新入口，要在恢复的错误成员里加（`recovery.rs` 的 `RecoveryFailure`），语义是拒挂载可写、只读挂载仍可读活头。

**要交用户的岔路（S8）**
| 候选 | 翻面观测 | 够判条件 |
|---|---|---|
| 有快照时管理员回退：一律拒（r1）／只许回到最新快照之后的版本（r2）／回退时抹掉之后的快照（r3） | 用户在有快照的池上还要不要用管理员回退（D23 已定项 14 的承诺是「全池按发布回退」，没写快照在场的例外）；r3 的批数 = 被抹快照的钉住条目数 ÷ 144 | 用户答「有快照时回退要不要保留」 |

## S9 只读快照用到 D6 的哪几样

**判定：冲突一处（里程碑文件一把「只读快照」说成「第二个活引用」，与 D5 自己的两处用法不一致）；其余一致。**

**先并排那一处**
- 里程碑文件：「D5（快照 / 空间记账机制） 已定项 17「引入同一块的第二个活引用必须先重开 D5（快照 / 空间记账机制）」，只读快照就是第二个活引用。」（`.claude/kb/milestone/04-fourth-txn.md:42`）
- D5 已定项 17 的前提 3：「③ 每个块在活树里至多一个活引用」（`.claude/kb/decisions/05-快照-空间记账机制.md:368`）
- D5 已定项 14 的 death 定义：「最后一个活引用被摘掉、**且该摘除被发布**的 checkpoint 号」（`.claude/kb/decisions/05-快照-空间记账机制.md:320`）。若快照也算活引用，被快照引用的块永远不 death，引用区间 `birth ≤ S.txg < death` 自己就循环。
- D5 已定项 20 的例外句：「例外：被销毁的快照从写意图那一刻起不算活引用」（`.claude/kb/decisions/05-快照-空间记账机制.md:433`）。这里的「活引用」把未销毁的快照算在里面。
「活引用」在已定项 14、17 里指活树引用，在已定项 20 里含快照；一个词两个口径。本形态按 14 与 17 的读法：只读快照不是活树，前提 3 不被越过，D5 已定项 19 的归约证明（只用 txg 全序与「判定在死亡那一刻」）原样成立，旁表恒空。⇒ 「D5 已定项 17 的重开收口」在这一轮落成一句话：只读快照不越过四条前提，真正触发重开条款的是 D6 已定项 1 的跨头共享（可写克隆）；已定项 20 那句例外要改成「被销毁的快照从写意图那一刻起不再钉住块」以免与已定项 14 的词义打架。这是给主 agent 的条款修订，不是用户岔路。
推翻：D5 作者的「活引用」其实包括快照，而 death 另有定义——则 death 要重写，归约证明要重做。

**D6 机制逐样**（留给可写克隆的，供写进 `.claude/kb/milestone/04-fourth-txn.md` 一的「开工前要先定的」那一段的归属判断）
| 样 | 用／不用／留给可写克隆 | 依据 | 本形态里对应什么 |
|---|---|---|---|
| 每头一棵自己的树（D6 已定项 1） | 用其中的头 ID 约定（D5 已定项 9），不用「每头一棵可写树」的机制；快照不算头 | 快照只冻两个根，不新建可写树；deadlist 的 key 前缀与码 14 的头 ID 都是活头的 inode 树树号 | S1、S3 |
| 引用计数旁表（D5 已定项 6、D6 已定项 1 连带） | 留给可写克隆 | 条目恒 0 条；判定里「没有条目按 1 算」。写出这条分支则没有任何输入能让它为真，无法满足「2. **每条分支必须能被测试强制进入。**」（`.claude/rules/fs-design.md:122`）；第一版判定函数不带这一臂，旁表树保持零根注册，克隆那一轮加臂并带第一个能写进去的测试 | S3 |
| livelist（D6 已定项 3、2） | 留给可写克隆 | 它的消费者是「销毁一个可写头」；销毁快照走 deadlist 并到下一侧，不读 livelist | 树表里 livelist 零根照旧 |
| O(N) 反向索引查找（D6 已定项 5） | 不走 | 销毁快照的代价是 O(被钉条目数 + 批数)（D5 已定项 19），不需要「每个共享块找谁在引用」；D6 已定项 5 的豁免主语是销毁快照，被豁免的量这里用不上 | S4 |
| 整单元 COW（D6 已定项 7） | 用，已实现 | 快照下的部分覆盖整单元 COW，没有 sub-unit 共享，所以 deadlist 条目按单元身份记、无 extent 偏移 | S3 |
| 回退到快照态（D6 已定项 4） | 不提供，与 S8 一致 | 要回到快照的状态走克隆 | S8 |
**留给可写克隆的清单（供归属判断）**：稀疏旁表的读写分支；livelist 全部（C271、C260）；`children` 计数与分叉点闸（D5 已定项 18 第 2 条，C270）；还有自己活快照的头不许销毁的第二道闸（C272）；origin 链接与克隆时刻（预留认购表里头条目的 8 字节）；C265、C347；谱系区间标号 16 字节（C144，线性单头用不上，树状谱系才要）；`previous_snapshot_txg` 的「克隆父亲」读法（C250 的第二半）；C110 的完整形（克隆祖先表，S1 第 8 条只做了最小形）；D6 已定项 5、10 的 O(N) 与优化。
**归本形态**：C145、C300、C301、C336、C337、C195（S3、S4、S5）；C675（S1 第 3 条）；C110 的最小形（S1 第 8 条）。C301（deadlist 与 livelist 的态别没写）：按 D21 的权威态／派生态判据，deadlist 是权威态（记着谁还钉着这个单元，丢了不可重建，被钉单元会成孤儿），被钉映射条目也是权威态，销毁队列（带记号的码 14 集合）是派生态——写进 D21 是主 agent 的条款修订（推的）。

**要交用户的岔路（S9）**：无；上面的归属判断是条款整理。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| S1 | 冲突＋规则没说 | D8 已定项 8 ① 的「每个快照一条条目」与一个根指针槽对不上；推荐两个新种类（码 14 快照 inode 根、码 15 快照 extent 根）各一条，不碰已有 kind-1／2 的循环；单节点封顶 34 个快照 |
| S2 | 一致＋规则没说 | 独立发布，S.txg = 这次发布的 txg，头的前驱 := S.txg，deadlist 不动；队列问题被「每次用户改动先走空」免费回答；同 checkpoint 双快照第一版造不出 |
| S3 | 一致＋冲突（切口行号）＋规则没说（映射留存） | 切口在 `transaction.rs:6825` 与 `:8141` 之间，不是 `:1861`；只有用户可见的六类角色问判定；被钉单元的映射条目要留（G2）；deadlist 做第三棵多层码 2 树，每个被钉单元 43 字节 |
| S4 | 一致＋规则没说 | 意图住码 14 条目（flags 位＋hi 8＋续做点 27），不依赖 `m4-gc-r1`；中间态下 I-3.5 ③ 与 I-3.6 要带待放集例外；hi 在下一侧是快照时必存，是头时串行纪律下可省（模型） |
| S5 | 规则没说 | I-3.6、I-3.7 在 checker 零命中；给出可判定形式、输入取不到报「不适用」、各自必红世界；GPU 新增 4 至 6 位 |
| S6 | 规则没说＋一致 | 三条流接在多记录流 A 之后，文件取 1 个数据单元，最大段 26 写 < 30；3 单元文件快照下覆盖写 32 写超 `FULL_EXPAND_UP_TO` |
| S7 | 规则没说 | RefFS 约 150 至 250 行、IdealModel 约 500 至 700 行、`HistoryOperation` 12 → 15；新操作进自己的权重组，`KNOWN_RED_FORMS` 不新增 |
| S8 | 规则没说＋冲突（今天的拒绝覆盖不到快照） | 不加规则则回退会放掉被钉单元（丢数据）；推荐第一版有快照即拒；崩溃恢复落到更早的根对建快照免费成立 |
| S9 | 冲突（「活引用」两个口径）＋一致 | 只读快照不越过 D5 四条前提；旁表、livelist 留给可写克隆，O(N) 查找不走；给出归属判断清单 |

## 要交用户的岔路汇总（候选、翻面观测、够判条件在各格，这里只列题）
1. S1 第一版同时活着的快照数上限（1／34／44／69／不设）。
2. S1 快照怎么点名（树号／序号／名字）。
3. S3 被钉单元映射条目留存（α）还是 deadlist 条目带位置项（β，重开三条条款）。
4. S4 销毁快照期间别的改动：串行还是交错。
5. S8 有快照时管理员回退：拒／条件放行／抹掉。
被攻过零轮：全部。模型（`research/prompts/m4-snapshot-r1-sonnet-model/`）只验了 S3、S4 的判定函数与销毁流程，没验 S1、S5、S8 的盘上形与 checker 判法。

## 命令输出（这一次真跑的原样）
1. 容量，`python3 -I -c '…'` 用 `unit::index_node_entry_capacity` 的式子（头 = 115 + 2 × key 宽，16384 减头除条目宽）：
```
tree table 8/200: 81
snapshots at 2 entries each: 34 at 1 entry: 69
widened 286: 56 44
deadlist leaf/internal: 376 125
accounting leaf/internal (check vs code 477/150): 477 150
mapping leaf/internal (check vs code 294/143): 294 143
snapshot-tree record 204 bytes per 32 KiB container: 159
```
记账树、映射树的两个数与代码注释一致：「格式算出来的节点容量：记账树叶 477、内部 150，中央映射树叶 294、内部 143」（`crates/singlefs-core/src/transaction.rs:2400`），所以对 deadlist 的 376／125 用的是同一个式子。
2. 按种类取条目的地方、角色分支数，`grep -c` 每个文件：
```
singlefs-checker/src/walk.rs INODE=20 EXTENT=12
singlefs-core/src/recovery.rs INODE=10 EXTENT=7
singlefs-core/src/mounted_read.rs INODE=2 EXTENT=2
singlefs-core/src/transaction.rs INODE=2 EXTENT=2
singlefs-core/src/mount.rs INODE=2 EXTENT=2
AccountingTreeNodeBelowTheRoot in transaction.rs: 15
```
3. 默认环下一次发布的 journal 记录数上限：`python3 -I -c 'print(805306368//4096//3)'` ⇒ `65536`。
4. 已有流段形状代入闭式 1 + Σ(2^写数 − 1)，以及几个段长的 2^写数 − 1：
```
35 1448083588
16 65535
18 262143
26 67108863
28 268435455
30 1073741823
32 4294967295
```
5. I-3.6、I-3.7 在两个 checker 源目录的出现次数：`grep -rn 'I-3\.6\|I-3\.7' crates/singlefs-checker/src crates/singlefs-checker-tier/src | wc -l` ⇒ `0`。
6. 波及面：`ModelOperationKind::` 只有 `singlefs-harness/src/model.rs` 18 处；`HistoryOperation::` 前八名 `history.rs` 93、`random_histories.rs` 61、`e158_root_choice_repair.rs` 38、`crash_injection_campaign.rs` 27、`unlink_in_histories_judged_by_the_model.rs` 26、`reference_filesystem_translation.rs` 15、`fault_injection.rs` 13、`crash_injection.rs` 12。

## 模型输出（`research/prompts/m4-snapshot-r1-sonnet-model/model.out`，经内存包装 8G 跑 `tools/rerun.sh`，每个变体 20000 个 40 步的随机历史）
```
name=deadlist_model variant=paper histories=20000 steps=40 violated=0 kinds={} first=None
name=deadlist_model variant=paper_serial histories=20000 steps=40 violated=0 kinds={} first=None
name=deadlist_model variant=noprev histories=20000 steps=40 violated=11585 kinds={'T-leak': 2642, 'T-loose': 8943} first=(1, ('T-leak', 0))
name=deadlist_model variant=noprev_serial histories=20000 steps=40 violated=11614 kinds={'T-leak': 3650, 'T-loose': 7964} first=(2, ('T-loose', 6))
name=deadlist_model variant=hiinf histories=20000 steps=40 violated=17090 kinds={'L-lost': 17090} first=(0, ('L-lost', 4))
name=deadlist_model variant=hiinf_serial histories=20000 steps=40 violated=13349 kinds={'L-lost': 13349} first=(3, ('L-lost', 4))
name=deadlist_model variant=hihead histories=20000 steps=40 violated=5505 kinds={'L-lost': 5505} first=(3, ('L-lost', 10))
name=deadlist_model variant=hihead_serial histories=20000 steps=40 violated=0 kinds={} first=None
name=deadlist_model variant=lt histories=20000 steps=40 violated=12135 kinds={'L-lost': 12135} first=(0, ('L-lost', 2))
```
注：以上九行与 `model.out` 逐行相同（文件的 sha256 在模型目录的 `SHA256SUMS`）。变体含义见 `deadlist_model.py` 文件头。模型的真相（`birth ≤ S.txg < death`）与存储字段互不共享，但模型是我自己写的、没被攻过；它证的是 D5 正文这套规则自洽、省 hi 与省 prev 传递各自造出什么，不证盘上形、不证 checker。
`tools/rerun.sh --selftest` 在我改过 `rerun_steps` 的这份拷贝上是红的（模板自检把脚本单独拷走、而我的步骤要读同目录的 `deadlist_model.py`）；真跑（上面命令）是绿的。判决复跑请用真跑。

## 复核不了、要主 agent 跑的命令
- 没编译 Rust，没跑 cargo、没跑 checker 档任何东西。凡写「推的」的单元数、写数、行数、字节数，都是从 `resolve` 的角色清单、已有流的文件头与段形状推的；真数要在实现落地后取写表。
- 要主 agent 跑的命令：无。模型复跑命令（供判决复核，用副本）：`bash scripts/hosts/run-with-memory-cap.sh 8G bash <模型目录副本>/tools/rerun.sh --repo <仓根> --frozen <任一目录> --work <新目录> --model <模型目录副本>`，看 `model.out` 九行与上面逐行相同、sha256 与 `SHA256SUMS` 一致。
- 背景材料第二节我逐处核过、一致的：三棵树零根注册（`trees.deadlist,` 在 `transaction.rs:8688` 一带，livelist、稀疏旁表同形）、树表条目 200 字节与 `previous_snapshot_txg` 写 0 读回丢（`records.rs:461`、`:500`）、发号 13 个（`transaction.rs:2749`）、`mount.rs:8083` 与变体定义 `:7440`、映射 key（`records.rs:347`）、`admission.rs:240` 的待删占用恒 0、`walk.rs:633`、`image.rs:248`、GPU 违例位 128／92、`DeletionStepOfThePublish`（`transaction.rs:4178`）、`ModelOperationKind` 与 `HistoryOperation` 各 12 种、`KNOWN_RED_FORMS` 空。
- 冲突或漂移的：`:1861`（G1）、`crash_facts.rs:287`→`:369`、`crash_judge_tables.rs:51`→`:54`；`crash.rs:1548` 现在是一行 tally 计数，不是「一段的状态数装进 u64」那一处，我没找到那一处，这一条我没核。

## 没做什么
- 不替主 agent 采纳或出判决；「推荐」是我这条腿的立场，岔路表不带倾向。
- 没读禁读清单里的文件（`m4-snapshot-r1-opus-output.md`、`m4-snapshot-r1-opus-model/`、`m4-gc-r1` 的两份产出与模型、`_m4-gc-r1-*.md`）。我的草稿目录里的 `rerun1` 是模型自己的输出。
- 没量任何入库装置的数；模型在副本上写、没进 `research/results/`，不是实验，不引成 kb 数。
- 没写 kb、没改 crates/、没改任何规则与门禁表；对条款的修订意见（D8 已定项 8 ① 的字面、D5 已定项 20 的「活引用」、D21 的态别、里程碑文件一·2 的「同一处 allocator.release」）都写在格里，交主 agent。
- 没判 D5 已定项 14 边界表三行（`birth == S.txg`、`death == S.txg`、`birth == death`）在每个候选上的具体历史：那是攻方腿的分工；我只在模型里让「写后同发布建快照」（`birth == S.txg`）与独立建快照各跑了 20000 史零违例。
- 没碰并行轮 `m4-gc-r1` 的 logged_ops 字段表；S4 的意图载体选了不依赖它的一个，把耦合点写明。
- 模型目录里的 `deadlist_model.py` 与 `tools/rerun.sh` 是我写的新文件；草稿目录 `~/.local/share/singlefs/work/m4-snapshot-r1-forward/` 里只有分段稿、`rerun1/` 与一个空的 `frozen-empty/`，没有仓副本与编译目录。
