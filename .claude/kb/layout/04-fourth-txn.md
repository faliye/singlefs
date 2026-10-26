# 共享与回收写出哪些字节（新池新建文件、覆盖写与删除之后新出现的盘上形态）

**中文** · [English](04-fourth-txn.en.md) · [日本語](04-fourth-txn.ja.md)

**里程碑「共享与回收」一·3 与六·3 的登记表**（[04-fourth-txn.md](../milestone/04-fourth-txn.md)），2026-10-26 立。新池新建文件的每一段字节在 [layout/01-first-txn.md](01-first-txn.md)，覆盖写、释放、回退与复用新写出的形态在 [layout/02-second-txn.md](02-second-txn.md)，删除与管理员回退的谱系记录新写出的形态在 [layout/03-third-txn.md](03-third-txn.md)；这里只登记里程碑四新写出的形态：只读快照（建快照、快照下覆盖写与删文件、删快照）、整理（意图与搬迁批）、截断与变长、部分写、extent 上段叶标签 0、隔离的盘上记号。同一个字段的宽度与落点以 layout/01-first-txn.md 为准、不另抄。每一行指一条决策分项，指不到分项的写成「格式级空白」并写出处，汇总在「八、格式级空白与没找到」那一节。代码出处的行号是 2026-10-26 在工作区现查的（含别的会话还没提交的改动）；这张表不引干跑产物，字节以 `crates/` 里点名的读写函数为准。

## 一、形态

| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 |
|---|---|---|---|---|
| 树表码 14 条目（快照 inode 根） | 建快照那次发布（`crates/singlefs-core/src/transaction.rs:9436` 的 `publish_create_snapshot`：独立发布，不写用户数据、不建 inode） | 一个快照一条，200 字节，字段在「二、树表的快照条目（码 14、15）」那一节；排在十二条之后、整张树表仍按树 ID 升序；树表条目数 = 12 + 2 × 活快照数，声明长度 = 条目数 × 200；第一版至多 10 个活快照（`crates/singlefs-core/src/records.rs:559` 的 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`），第 11 个在任何读写之前拒（`crates/singlefs-core/src/transaction.rs:9452`） | D8（核心索引结构） 已定项 8 ① / 已定项 9；D5（快照 / 空间记账机制） 已定项 9 / 已定项 21 | 一·2、一·3 |
| 树表码 15 条目（快照 extent 根） | 建快照那次发布，与码 14 成对 | 一个快照一条，200 字节，字段在「二、树表的快照条目（码 14、15）」那一节；删快照的最后一批把这一对一起删掉 | D8（核心索引结构） 已定项 8 ① / 已定项 9 | 一·2、一·3 |
| 活头 inode 树条目的 `previous_snapshot_txg` 第一次不为 0 | 建快照那次发布 | 树表里 inode 树那条条目（种类 2）偏移 100 起 8 字节换成这次发布的 checkpoint_txg（S.txg）；之后每版照抄，直到下一次建快照、或删掉最新那个快照的意图把前驱传过来（`crates/singlefs-core/src/transaction.rs:6081`–`:6085` `SnapshotStepOfThePublish::CreateSnapshot` 的文档注释） | D5（快照 / 空间记账机制） 已定项 9；D8（核心索引结构） 已定项 8 ③ | 一·2 |
| 码 14 条目 flags 位 0「销毁中」 | 删快照的意图那次发布（`crates/singlefs-core/src/transaction.rs:9514` 的 `publish_snapshot_destruction_intent`） | 被删那个快照的码 14 条目 flags（条目偏移 12 起 2 字节）写 1；同一次发布把它的前驱 P（那条码 14 的 `previous_snapshot_txg`）写进下一侧：树表里按（诞生 txg, 树 ID）排在它后面的那条码 14，没有就是活头 inode 树条目（`crates/singlefs-core/src/transaction.rs:6133` 的 `next_side_of_the_snapshot`）；一条 deadlist 条目都不动；最后一批同一次发布删掉那一对码 14、15，记号随之没了；别的种类上的位 0 与别的位，读者拒收那条条目（`crates/singlefs-core/src/records.rs:736`） | D5（快照 / 空间记账机制） 已定项 12 射程；D8（核心索引结构） 已定项 8 ㊁ | 一·2 |
| deadlist 条目 | 有活快照时，换下被快照引用的用户可见单元的那次发布（覆盖写、unlink 的批、截断的批） | 43 字节、无 value，字段在「三、deadlist 条目与 deadlist 树」那一节；只有用户可见子树的六类角色（数据单元、extent 下段节点、extent 上段根之下的节点、extent 根、inode 叶容器、inode 根）问生死判定，birth 大于活头 `previous_snapshot_txg` 的照常释放，余下的进 deadlist、不释放（`crates/singlefs-core/src/transaction.rs:6495` 的 `snapshot_lifetime_verdict`）；删快照的各批按 key 升序放掉纪元落在 [lo, hi)、birth > P 的条目（`crates/singlefs-core/src/transaction.rs:6168`） | D5（快照 / 空间记账机制） 已定项 12 / 已定项 14 / 已定项 19 | 一·2 |
| deadlist 树节点（码 2） | 第一条 deadlist 条目进来那次发布（树由零根变成有根）；条目不变的发布整棵照抄 | 多层码 2 树：树 ID 65543、种类 8、头 201、叶至多 376 条、内部节点至多 125 条，字段在「三、deadlist 条目与 deadlist 树」那一节；树表 deadlist 条目的根指针从零变成指这棵树的根，一条不剩时零根、根指针置零 | D5（快照 / 空间记账机制） 已定项 11 / 已定项 12；D8（核心索引结构） 已定项 11 | 一·2 |
| 被快照钉住的单元，它的中央映射条目留存 | 「deadlist 条目」那一行同一次发布 | 进 deadlist 的单元，它那条映射条目（55 字节，形态照 [layout/01-first-txn.md](01-first-txn.md)「三·二、中央映射条目（2026-09-06 新增，D19（块指针的结构与宽度预算） 已定项 5）」那一节）照留在中央映射树里，活头已经不引用它；删快照放掉那条 deadlist 条目时经这条映射条目释放那个落点 | 格式级空白（出处：判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3「被钉单元的映射条目留在中央映射树里」） | 一·2 |
| logged_ops 整理意图条目（种类 1） | 签发那次发布（`crates/singlefs-core/src/transaction.rs` 的 `publish_compaction_step` 带 `CompactionIntentStep::Issue`） | 44 字节，字段在「四、logged_ops 条目（整理与截断）」那一节；只搬一批的发布照抄；意图达成那一次（R 按记录全空）或丢意图那一次，同一次发布删掉它 | D26（后台整理与放置回收） 已定项 2；D8（核心索引结构） 已定项 5 | 二·2 |
| logged_ops 根（码 2） | 第一条意图（整理或截断）签发那次发布（树由零根变成有根）；条目集合不变的发布照抄；意图删光的那次不写它，树表 logged_ops 条目的根指针置零（`crates/singlefs-core/src/transaction.rs:7778`） | 单元类型标签 2，层级 0，树 ID 65547，key 宽 10，头 135（86 + 2 × 10 + 29），条目宽 44，key 区间取首末两条条目的 key（`crates/singlefs-core/src/transaction.rs:13014`）；条目数至多 2：活整理意图、活截断意图全池各至多一条（`crates/singlefs-core/src/logged_operations.rs:363`）；一个根装 369 条，第一版不长层 | D26（后台整理与放置回收） 已定项 2；D8（核心索引结构） 已定项 5 / 已定项 11；D18（块里携带什么信息） 已定项 2 | 二·2、六·2 |
| 整理的搬迁批 | 签发之后的每一批（`crates/singlefs-core/src/transaction.rs:9676` 的 `publish_relocation_batch`；最后一批可以与删意图同一次发布） | 计划里的每个单元原样字节写到目的落点（单元头不改、birth 不变）；它那条中央映射条目 key 不变、两条位置条目换成目的落点；目的落点记一条已分配的分配记录，分配代写那个单元的诞生代号，不写这次发布的 txg；源落点那条分配记录改写成已释放、释放代 = 这次发布的 txg（形态同 [layout/02-second-txn.md](02-second-txn.md)「已释放的分配记录」那一行）；被快照引用的单元照样搬，留存的映射条目跟着换位置；记账树、分配记录树、中央映射树与树表照常重写；journal 点名项是搬迁副本加这次新写的节点，一个事务号；logged_ops 照抄（同一函数的文档注释，`crates/singlefs-core/src/transaction.rs:9659`–`:9661`） | D26（后台整理与放置回收） 已定项 5 第 1、2 条；D5（快照 / 空间记账机制） 已定项 15；D19（块指针的结构与宽度预算） 已定项 5 | 二·2 |
| logged_ops 截断意图条目（种类 2） | 截断的意图那次发布（`crates/singlefs-core/src/transaction.rs:10453` 的 `publish_size_change`，目标 size N 小于现在的 size S） | 44 字节，字段在「四、logged_ops 条目（整理与截断）」那一节（条目在 `crates/singlefs-core/src/transaction.rs:6028` 立）；各批照抄；最后一批同一次发布删掉它；承诺类，可写挂载交回之前走到空 | D8（核心索引结构） 已定项 5「截断意图的字段表」；D26（后台整理与放置回收） 已定项 2 的种类登记 | 六·2、六·3 |
| inode 记录偏移 120–135 的截断字段第一次不为 0 | 放掉单元的那次截断意图发布 | T 8 + N_w 8，字段与写法在「五、inode 记录的截断字段与 incompat 位 2」那一节 | D8（核心索引结构） 已定项 6；D15（格式冻结政策） 已定项 4 | 六·2、六·3 |
| 系统配置 incompat 位 2 | mkfs 起每一次写系统配置；不带位 2 的旧池在可写挂载取号那一写起带上 | 系统配置偏移 6 起的 feature bits 里，incompat 位图第一个字节从 0x02 变成 0x06，在「五、inode 记录的截断字段与 incompat 位 2」那一节 | D15（格式冻结政策） 已定项 4 | 六·2 |
| 截断与变长重写的尾单元 | 截断的意图那次（截点落在一个单元中间、那个单元的声明长度变了）；变长那次（最后一个存在的单元没装到 min(N − 起点, 净荷)） | 码 1 数据单元，字段照 [layout/01-first-txn.md](01-first-txn.md)「二、数据这一侧」那一节；单元头的 extent 声明长度改成新长度：截断取旧净荷的前这么多字节，变长在旧净荷之后补零到这么长，声明长度之后的补齐恒 0；截到 0 留一个声明长度 0 的单元（`crates/singlefs-core/src/truncate.rs:27`、`:95`、`:136`）；截点落在尾部的洞里不重写（洞前那个单元是满的）；截断不写区间墓碑，变短由读法 Z 管 | D18（块里携带什么信息） 已定项 10（部分删除不写区间墓碑）；尾单元怎么重写与声明长度 0 的单元是格式级空白（出处：判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3、K5） | 六·2 |
| 部分写重写的那一段单元 | 部分写那次发布（`crates/singlefs-core/src/transaction.rs` 的 `publish_partial_write`，会话里 `UserChange::WriteAtOffset`；任意活 inode） | 码 1 数据单元，字段照 [layout/01-first-txn.md](01-first-txn.md)「二、数据这一侧」那一节；从写的偏移所在的单元起到写之后最后一个单元一起重写，一个单元一个事务（`crates/singlefs-core/src/truncate.rs` 的 `DataUnitsRewrittenInOnePublish`）；每个单元：写盖满的不读，有旧单元的读回旧单元、没被写到的字节取原内容，没有旧单元而写到的旧内容按零、这次分到的新槽的残留字节一律不进新单元，写的起点越过末尾时中间的空隙补整单元零、不读；size = max(写之前, 写的终点)，单元数 = max(写之前, 写到的最后一个单元 + 1)，上段叶标签按写之后的单元数与 size 现取，截断字段 T、N_w 照抄；快照下只把这次真换下的单元进 deadlist | D4（校验和位置） 已定项 3 射程第一条（新单元读残留、残留字节不进新单元）；从偏移所在单元起一起重写与空隙补整单元零是格式级空白（出处：判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节写序那一格与跨末尾那一格；写序的前提在 I-1.13（扫描重建与遍历方向的索引相同） 那一行） | 待优化项「通用部分写 RMW」 |
| extent 上段叶标签 3、4 | 变长那次发布（N 越过最后一个存在单元的末端，之后到 N 是洞），任意活 inode 都写（号 ≥ 2 的记在 `FileOfAnInodeOtherThanTheFirst` 的 `tail`）；跨末尾或写进洞里的部分写同样按写之后的单元数与 size 换 | 上段叶条目 113 字节同形：标签 3 的载荷同标签 1（下段根指针），声明下段最后一个单元之后到 size 是洞；标签 4 的载荷同标签 2（单元 0 的数据指针），声明单元 0 之后到 size 是洞；在「六、extent 上段叶条目的标签 0、3、4」那一节 | 格式级空白（D8（核心索引结构） 已定项 14 只登记标签 0、1、2；出处：判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5，`crates/singlefs-core/src/extent_tree.rs:52`–`:55`） | 六·2 |
| extent 上段叶标签 0（含 size 0） | 第一个文件：现有写路径不新写出这一形（每条写路径至少落一个单元），读进来的那一版里有它时，重写上段叶的每次发布照写；号 ≥ 2 的 inode：变长到 size > 0 而一个单元都没有的那次发布（`publish_size_change`） | 第一个文件（inode 1）有 inode 记录、一个数据单元都没有（size 0，或 size > 0 而整个文件都是洞）：上段叶条目 key (0, 1, 0)、标签 0、载荷 88 字节全零；这一形上 size 改大、改成原值、截短（截到 0 也算）都只改 inode 记录、标签留 0、不加单元；号 ≥ 2 的 inode 变长到 size > 0 而一个单元都没有时写同形的标签 0 条目（key (0, N, 0)），之后缩短、缩到 0、再变长都留着，到 unlink 最后一批才去掉，`publish_new_inodes` 建出的 size 0 inode 不写（粘住的标签 0：`crates/singlefs-core/src/transaction.rs` 的 `plan_the_files_other_than_the_first_after_this_publish`；判决 m4-multi-inode-size-r1，`research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节「别的文件的洞」那一格） | D8（核心索引结构） 已定项 14（`.claude/kb/decisions/08-核心索引结构.md` 第 451 行） | 六·2 |
| 分配记录跨度段次高位 0x4000（隔离记号） | 释放之前读盘核位置条目里的校验和、一份重读一次仍对不上或读不出的那次发布 | 两块盘上那条分配记录都留在已分配、分配代不变，跨度段置 0x4000，跨度值住其余位（跨度 1、2 写成 0x4001、0x4002）；在「七、隔离的盘上记号」那一节 | D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 第 117 行） | 作业点表里没有这一步（I-3.15（未释放记录罩住的单元由映射指在这一槽） 那一组） |

## 二、树表的快照条目（码 14、15）

条目 200 字节，字段次序与宽度照 [layout/01-first-txn.md](01-first-txn.md)「七、发布（根记录与根槽）」那一节「树表条目」那一行；读写在 `crates/singlefs-core/src/records.rs:693`（`to_bytes_with_destruction_mark`）与 `:736`（`parse_with_destruction_mark`），建快照那次的一对在 `crates/singlefs-core/src/transaction.rs:14321`、`:14330` 装。整数小端。

| 偏移 | 字段 | 宽度 | 码 14（快照 inode 根）的取值 | 码 15（快照 extent 根）的取值 | 指向 |
|---|---|---|---|---|---|
| 0 | 树 ID | 8 | 建快照那次从上一版树 ID 水位起连发的第一个号 | 第二个号（码 14 的树 ID + 1） | D8（核心索引结构） 已定项 8 ② |
| 8 | 条目长度 | 2 | 200 | 200 | D8（核心索引结构） 已定项 8 ④ |
| 10 | 树的种类 | 2 | 14 | 15 | D8（核心索引结构） 已定项 9 |
| 12 | flags | 2 | 0；删快照的意图那次起 1（位 0「销毁中」） | 0（位 0 只在码 14 上有含义，别的种类上读者拒收） | D5（快照 / 空间记账机制） 已定项 12 射程；D8（核心索引结构） 已定项 8 ㊁ |
| 14 | 根指针 | 86 | 建快照那一版活头 inode 树根的指针，逐字抄 | 建快照那一版活头 extent 树根的指针，逐字抄 | D8（核心索引结构） 已定项 8 ① |
| 100 | `previous_snapshot_txg` | 8 | 捕获那一刻活头 inode 树条目里的值（换成 S.txg 之前那一个） | 0（不适用） | D5（快照 / 空间记账机制） 已定项 9 |
| 108 | 诞生 txg | 8 | S.txg：建快照那次发布的 checkpoint_txg | S.txg | D8（核心索引结构） 已定项 8 ① |
| 116 | 头 ID | 8 | 活头 inode 树的树 ID | 配对那条码 14 的树 ID | D5（快照 / 空间记账机制） 已定项 9 射程 |
| 124 | 预留 | 76 | 0 | 0 | D8（核心索引结构） 已定项 8；D5（快照 / 空间记账机制） 已定项 12「树表字节」那一行 |

树 ID 水位这次加 2（D8（核心索引结构） 已定项 8 ②）：接着第一个文件版本那一版（水位 65549）建的第一个快照取 65549、65550，水位写 65551——推的，照 `publish_create_snapshot` 的取号现算，没有用例钉这几个值。树表单节点装 81 条，扣掉十二条之后封顶 34 个快照；第一版上限 10 个（D8（核心索引结构） 已定项 8 射程）。快照按码 14 的树 ID 点名；下一侧从树表按（诞生 txg, 树 ID）排的码 14 链现取，零字节（D5（快照 / 空间记账机制） 已定项 12「下一侧怎么找」那一行）。

## 三、deadlist 条目与 deadlist 树

deadlist 条目的叶条目就是 key、value 空，整数小端；读写在 `crates/singlefs-core/src/records.rs:619`（`key_bytes`）与 `:640` 起（`parse`，宽不等于 43 拒收）。

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 0 | 头的树 ID | 8 | 活头 inode 树的树 ID | D5（快照 / 空间记账机制） 已定项 12 |
| 8 | 纪元 | 8 | 块被杀那一刻活头的 `previous_snapshot_txg` | D5（快照 / 空间记账机制） 已定项 12 |
| 16 | 中央映射 key | 27 | 被钉单元的中央映射 key，形态照 [layout/01-first-txn.md](01-first-txn.md)「三·二、中央映射条目（2026-09-06 新增，D19（块指针的结构与宽度预算） 已定项 5）」那一节（码 1 一种、码 2 / 码 3 一种） | D5（快照 / 空间记账机制） 已定项 12；D26（后台整理与放置回收） 已定项 5 第 3 条 |

key 按七段逐字段无符号整数比（8、8、1、8、8、4、6，`crates/singlefs-core/src/code_two_tree.rs:44` 的 `CodeTwoKeyFieldWidths::DEADLIST`），同一个头同一段纪元的条目挨着。快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg) 的条目；建快照只推进活头的前驱，已有条目一条不动（D5（快照 / 空间记账机制） 已定项 12）。

| 段 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| deadlist 树节点 | 码 2 头 = 86 + 2 × 43 + 29 | 201 | 树 ID 65543、key 宽 43；层级、key 区间与分裂、收缩照多层码 2 树，与记账树、中央映射树同一套（`crates/singlefs-core/src/transaction.rs:13760`）；角色名 `tdl`、`tdl@层级.同层序号`（`crates/singlefs-core/src/transaction.rs:2770`） | D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 2 |
| deadlist 树叶 | 条目 = deadlist 条目 | 43 | 一片叶至多 376 条（⌊(16384 − 201) ÷ 43⌋） | D5（快照 / 空间记账机制） 已定项 12；D8（核心索引结构） 已定项 11 |
| deadlist 树内部节点 | 条目 = 分隔 key 43 + 子指针 86 | 129 | 一个内部节点至多 125 条（⌊(16384 − 201) ÷ 129⌋；`crates/singlefs-core/src/code_two_tree.rs:1238` 的单测钉这两个数） | D8（核心索引结构） 已定项 11 |
| 树表 deadlist 条目 | 根指针 | 86 | 有条目时指这棵树的根，一条不剩时置零；种类 8、树 ID 65543、头 ID 0 不变 | D5（快照 / 空间记账机制） 已定项 11 |

## 四、logged_ops 条目（整理与截断）

条目恒 44 字节、整数小端：key 10 = 种类 2 + 签发 txg 8，value 34 按种类解；0 与未登记的种类码拒收（D26（后台整理与放置回收） 已定项 2「logged_ops 种类登记与公共编码」那一段）。读写在 `crates/singlefs-core/src/logged_operations.rs:225`（`to_bytes`）与 `:267`（`parse`），种类码常量在 `:24`、`:27`。

整理意图（种类 1）：

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 0 | 种类 | 2 | 1 | D26（后台整理与放置回收） 已定项 2 |
| 2 | 签发 txg | 8 | 签发这条意图那次发布的 txg | D26（后台整理与放置回收） 已定项 2 |
| 10 | R 的设备 | 4 | R 所在的盘 | D26（后台整理与放置回收） 已定项 2 |
| 14 | R 的起始槽 | 6 | 64 的倍数，不对齐拒收 | D26（后台整理与放置回收） 已定项 2 |
| 20 | R 的槽数 | 2 | 恒 64（一个聚簇段），别的值拒收 | D26（后台整理与放置回收） 已定项 2 |
| 22 | `c0` | 8 | 签发那一刻的分配代 | D26（后台整理与放置回收） 已定项 2 |
| 30 | `segs0` | 8 | 签发那次发布之前的记账第 11 项（全空聚簇段数）里 R 所在盘的值，只作观测量 | D26（后台整理与放置回收） 已定项 2 |
| 38 | 补零 | 6 | 0，非零拒收 | D26（后台整理与放置回收） 已定项 2 |

截断意图（种类 2）：

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 0 | 种类 | 2 | 2 | D8（核心索引结构） 已定项 5「截断意图的字段表」 |
| 2 | 签发 txg | 8 | 截断意图那次发布的 txg | D8（核心索引结构） 已定项 5「截断意图的字段表」 |
| 10 | inode 号 | 8 | 目标活对象的 inode 号（第一版只有 inode 1 真改 size，别的活 inode 只收等长请求） | D8（核心索引结构） 已定项 5「截断意图的字段表」 |
| 18 | 对象出生代 | 8 | 与目标 inode 记录偏移 8 的对象出生代一致 | D8（核心索引结构） 已定项 5「截断意图的字段表」 |
| 26 | 目标 size | 8 | N（目标字节数） | D8（核心索引结构） 已定项 5「截断意图的字段表」 |
| 34 | 补零 | 10 | 0，非零拒收 | D8（核心索引结构） 已定项 5「截断意图的字段表」 |

## 五、inode 记录的截断字段与 incompat 位 2

inode 记录 140 字节的其余字段照 [layout/01-first-txn.md](01-first-txn.md)「四、文件元数据（inode）」那一节；那一节「inode 记录（偏移 108 / 112 / 120 / 128 / 136）」那一行列了偏移 120 起的截断字段两段与 136 起 4 字节预留，取值与写法在这一节。字节在 `crates/singlefs-core/src/records.rs:163`–`:165` 写。

| 偏移 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 120 | 截断字段 T | 8 | 最近一次放掉单元的截断意图那次发布的 txg；没有过是 0 | D8（核心索引结构） 已定项 6 |
| 128 | 截断字段 N_w | 8 | 那一次截断的目标字节数；T = 0 时必须是 0 | D8（核心索引结构） 已定项 6 |
| 136 | 预留 | 4 | 恒 0 | D8（核心索引结构） 已定项 6 |

写法：截断意图那次，这次之前存在的单元数多于 ⌈N ÷ 净荷⌉（至少 1）时写成（这次的 txg, N），不放单元的截断照抄上一版（`crates/singlefs-core/src/truncate.rs:116` 的 `truncation_field_after_the_truncation_intent`）；各批、最后一批、变长、等长请求与 unlink 照抄；整份写（顺序写、覆盖写）清零；新建的 inode 写 0；回退恢复目标那一版的值（D8（核心索引结构） 已定项 6「截断字段的兼容性与写法」那一段）。读者：T = 0 而 N_w ≠ 0 判记录不可用（`crates/singlefs-core/src/records.rs:91`）；池不带 incompat 位 2 时偏移 120–135 必须全零，不是全零的那条记录 EIO（I-9.7（记录三段恒零））。

| 段 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 系统配置 | feature bits 里 incompat 位图的第一个字节（槽内偏移 6） | 1 | 0x06（位 1 布局身份加位 2 截断字段）；读者收 0x02 与 0x06 两种，只带位 2 不带位 1 的拒；不带位 2 的池里 inode 记录偏移 120–135 必须全零 | D15（格式冻结政策） 已定项 4 |

写者每次写系统配置都写位 2（`crates/singlefs-core/src/system_configuration.rs:35` 的 `INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT`，写系统配置在 `crates/singlefs-core/src/transaction.rs:627`），mkfs 也写（`crates/singlefs-core/src/make_filesystem.rs:791`），所以新池新建文件的系统配置第一个 incompat 字节也是 0x06（[layout/01-first-txn.md](01-first-txn.md)「一、mkfs 已经种下的（新池新建文件之前就在盘上）」那一节系统配置字段表 feature bits 那一行写的同一个值）。

## 六、extent 上段叶条目的标签 0、3、4

条目 113 字节 = key 24（(0, inode, 0)）+ 标签 1 + 载荷 88，字段照 [layout/01-first-txn.md](01-first-txn.md)「四·二、extent 树（2026-09-10 新增，此前这张表里没有它）」那一节；读写在 `crates/singlefs-core/src/extent_tree.rs:497`（`to_bytes`），读者把标签 3、4 换成 1、2 照同一段解，不认识的标签拒收。

| 标签 | 载荷 | 声明的尾部 | 第一次出现 | 指向 |
|---|---|---|---|---|
| 0 | 88 字节全零 | 整个文件都是洞（size 0 时没有字节） | 第一个文件有 inode 记录、一个单元都没有时；现有写路径不新写出，读进来的那一版里有它时，重写上段叶的发布照写；号 ≥ 2 的 inode 变长到 size > 0 而零单元时也写，之后留到 unlink 最后一批（判决 m4-multi-inode-size-r1 定的粘住的标签 0） | D8（核心索引结构） 已定项 14 |
| 3 | 下段根的节点指针 86 加 2 字节零（同标签 1） | 下段最后一个单元之后到 size 是洞 | 变长越过下段最后一个单元末端的那次发布 | 格式级空白（出处：判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5） |
| 4 | 单元 0 的数据指针 88（同标签 2） | 单元 0 之后到 size 是洞 | 只有一个单元的文件变长越过单元 0 末端的那次发布 | 格式级空白（出处同标签 3 那一行） |

## 七、隔离的盘上记号

分配记录的形态照 [layout/01-first-txn.md](01-first-txn.md)「五、空间记账与分配」那一节：value = 跨度段 2（小端）+ 分配代或释放代 8。

| 段 | 字段 | 宽度 | 取值 | 指向 |
|---|---|---|---|---|
| 分配记录 value | 跨度段最高位 0x8000 | 1 位 | 已释放标志 | D3（空间分配） 已定项 7 |
| 分配记录 value | 跨度段次高位 0x4000 | 1 位 | 隔离记号：隔离那次发布在两块盘上那条记录里都置上，记录仍分配、分配代不变；回退复活那一次同一次发布清掉；释放隔离记录时改写成已释放，记号随之清掉 | D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」 |
| 分配记录 value | 跨度段其余 14 位 | 14 位 | 跨度值（1 或 2）；带记号的记录读成跨度 1、2 | D3（空间分配） 已定项 7；D19（块指针的结构与宽度预算） 已定项 5 |
| 分配记录 value | 0x4000 与 0x8000 同时置 | — | 坏记录，读者拒收（`crates/singlefs-core/src/allocator.rs:57` 的 `QuarantineMarkOnAReleasedRecord`） | D19（块指针的结构与宽度预算） 已定项 5 |

记号常量在 `crates/singlefs-core/src/allocator.rs:37`，写在 `crates/singlefs-core/src/allocator.rs:2206` 的 `release_leaving_the_record_allocated_on`；一个单元任一份核不过时两份一起交回、两块盘一起留（`crates/singlefs-core/src/transaction.rs:5122`）。不认记号的旧读者读到 0x4001、0x4002 拒整棵分配记录树（E187（隔离落盘代价） 第一段）。池级 checker 只认记号：两块盘都带记号的未释放记录不按 I-3.15（未释放记录罩住的单元由映射指在这一槽） 判违例（D19（块指针的结构与宽度预算） 已定项 5）；两盘不对称还没有判定（C873（隔离记录两盘不对称没人判））。

## 八、格式级空白与没找到

| 项 | 现状 | 出处 |
|---|---|---|
| 被快照钉住的单元，它的中央映射条目留存 | 格式级空白：决策正文没写 | 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 |
| 进 deadlist 的单元不释放：分配记录照旧已分配，记账的已分配与 defer 两行不因它变 | 格式级空白：决策正文没写记账这一半 | 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3「记账」那一句 |
| extent 上段叶标签 3、4 | 格式级空白：D8（核心索引结构） 已定项 14 只登记标签 0、1、2 | 判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5；`crates/singlefs-core/src/extent_tree.rs:52`–`:55` |
| 截断与变长重写尾单元的规则、截到 0 留一个声明长度 0 的单元 | 格式级空白：决策正文没写 | 判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3、K5；`crates/singlefs-core/src/truncate.rs:27`、`:95`、`:136` |
| 部分写从偏移所在的单元起一起重写、写的起点越过末尾时空隙补整单元零 | 格式级空白：决策正文没写（写序的前提写在 I-1.13（扫描重建与遍历方向的索引相同） 那一行） | 判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节写序那一格与跨末尾那一格；`crates/singlefs-core/src/write_request_split.rs` 的 `PartialWriteOfAFile` |
| 搬迁批的 journal 点名项点名搬迁副本 | 格式级空白：点名项的字段表在 D23（journal 的角色与格式） 已定项 17，点名搬迁副本这一条决策正文没写 | 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3；`crates/singlefs-core/src/transaction.rs:9660` |
| 活整理意图、活截断意图全池各至多一条 | 格式级空白：决策正文没写 | 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、`research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1；`crates/singlefs-core/src/logged_operations.rs:363` |
| 快照条目的树 ID 取值（第一个文件版本之后第一次建快照取 65549、65550） | 推的，没有用例钉 | `crates/singlefs-core/src/transaction.rs:9436` 的 `publish_create_snapshot` |
| GPU 垃圾回收（七·x）新写出的盘上字节 | 没找到：工作区 `crates/` 下没有 `singlefs-background-gpu` 包；未跟踪的 `crates/singlefs-core/src/background_compute.rs` 里没有写盘调用；`crates/singlefs-core/src/` 未暂存的改动里新增的行没有一行带 `put_u`、`write_at`、`to_slot` | 2026-10-26 现查 |
