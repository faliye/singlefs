# 只读快照的形态第一轮正文（2026-10-13）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 S7 S8 S9 I-3.5 I-3.6 I-3.7 I-9.25 I-9.27 I-9.16 I-3.1 I-1.3 -->
<!-- 不变量编号声明为非编号是绕 scripts/kb/quote-kb.py 第 326 行还指已搬走的 kb 根下那份不变量单文件：清单改用 --files 把 crates/singlefs-checker/invariants/ 下的类文件整份带上 -->

## 一、题面

里程碑四作业点一·1（`.claude/kb/milestone/04-fourth-txn.md`「一　共享最小形态：一个只读快照」）：D5（快照 / 空间记账机制） 已定项 17 的重开收口到「一个只读快照」这一形态；只读快照用到 D6（快照实现模型） 已定机制的哪几样（每头一棵树、引用计数旁表、livelist），哪几样留给可写克隆。里程碑四按串行提交设计，「快照在并行提交下切在哪」归里程碑六一·1，这一轮不答。

这一轮要的是：把「建一个只读快照、在快照下覆盖写与删文件、删快照」三种操作各自写成一次（或几次）发布改哪些盘上结构、运行时在哪一处判生死、checker 多判哪几条，每一格给候选、代价（要动 `crates/` 哪几处、新写出哪些字节）、要交用户的岔路。不定可写克隆、不定多头、不定并行提交。

被测条款（材料附录整节抄）：D5（快照 / 空间记账机制） 已定项 9、12、13、14、16、17、19、20、21；D6（快照实现模型） 已定项 1、3、5；D8（核心索引结构） 已定项 5、8、9、16；D16（发布语义） 已定项 14；D19（块指针的结构与宽度预算） 已定项 5、7、8；D21（权威态与派生态的分界） 已定项 6。不变量 I-3.5（引用区间的精确性）、I-3.6（deadlist 紧界）、I-3.7（活快照可枚举）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.27（树表条目的头 ID 取值）。欠账 C144（快照谱系的区间标号没有载体）、C145（deadlist 在盘上没有落点）、C300（deadlist 条目按 key 查映射查不到没有条款）、C301（deadlist 与 livelist 的态别没写）、C336（写意图那一刻找不到快照的下一侧）、C337（被销毁的头何时不算活引用没有条款）、C195（销毁级联时 prev 传递只写在实验的模型修订记录里）、C250（`previous_snapshot_txg` 的父链解析非单射）、C110（跨头共享与加密后元数据类的块头一致性读法）、C675（同一种树在树表里至多一条没有不变量）。

## 二、实现今天的样子（主 agent 的观测，2026-10-12 现查，行号按工作树 `~/.local/share/singlefs/work/m4-predev`，与主仓暂存区相同）

- 三棵树已注册、没有写者：`crates/singlefs-core/src/transaction.rs` 的 `FileVersionTreeIdentifiers`（`:2709`–`:2711` 是 `livelist`、`sparse_side_table`、`deadlist` 三个字段），树表条目三棵都只写零根（`:8677`、`:8683`、`:8689`）；`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version_in_the_unit_area_starting_at`（`:2521`）在 `:2583`–`:2585` 读回三个树 ID。全 core 没有往这三棵树装条目的代码。`grep -rn "克隆" crates/singlefs-core/src` 只命中 `transaction.rs:3073`–`:3074` 一条 TODO（inode 号水位台阶行不按树段分）。
- 树表条目 200 字节（`crates/singlefs-core/src/records.rs` 的 `TreeTableEntry`，`to_bytes` 在 `:437` 起）：树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + 头 ID 8 + 预留 76。`previous_snapshot_txg` 一律写 0（`:461`，「0 = 不适用」），`parse` 读回时丢掉（`:500` 的 `let _previous_snapshot_txg = reader.get_u64();`）。头 ID 的期望值表在 `crates/singlefs-checker/src/walk.rs:4291` 起（I-9.27 的判法在 `image.rs` 的 `HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES` 文档）：inode 树（种类 2）等于自己的树 ID，extent、`deleted_inodes`、墓碑树等于某一条种类 2 条目的树 ID，种类 3、4、6、7、8、11、12 等于 0。树的种类码 0..13（`records.rs:430` 起，13 是退役树）。
- 树 ID 只在第一个文件版本那次从水位连号发十三个（`transaction.rs:2698`、`:2749`，`FileVersionTreeIdentifiers::issued_from_watermark`），之后每一版照抄、不再发号；mkfs 水位 65536，发出来 65536..65548。建快照要给新树发 ID，今天没有这条路。`crates/singlefs-core/src/mount.rs:8083`–`:8085`（变体定义 `:7440`）：管理员回退遇到目标版树 ID 集合与现行版不同就拒，写着「处置未定」；core 读树表遇同一种树两条就拒（C675（同一种树在树表里至多一条没有不变量）），checker I-9.16 ③ 判每种至多一条。
- 释放判定今天只有一条路：`transaction.rs:3265` 的 `placements_to_release_via_mapping` 把换下的角色经中央映射查落点，`:1861` 的 `for released in settled.release { allocator.release(released, txg) }` 一律进 defer（`crates/singlefs-core/src/allocator.rs:1850` 的 `release`）；删除批与覆盖写共用这一处，没有按快照分支、没有引用计数、没有往 deadlist 写的路径。指针头部带出生 (树, txg)（`crates/singlefs-core/src/pointer.rs:100`–`:103`），所以 D5（快照 / 空间记账机制） 已定项 19 的 `birth` 零次 I/O 读得到；`head.previous_snapshot_txg` 今天没有常驻内存的载体（树表条目读回丢掉）。
- 中央映射 key：数据单元按指针头 + 写序，节点按单元类 + 指针（`records.rs:347`、`:359`）；按版查映射位置 `transaction.rs:3504`；位置提示读不出时经映射回退读（`recovery.rs:448`，D19（块指针的结构与宽度预算） 已定项 8）。
- 准入式子里「待删占用」恒 0（`crates/singlefs-core/src/admission.rs:238`–`:240` 的 `PENDING_DELETE_OF_THE_FIRST_VERSION`，注释「不销毁快照」）。记账统计量第 12 项 inode 号水位只为可写头维护（D5（快照 / 空间记账机制） 已定项 4）。
- checker：I-3.5 的出现规则已实现（`crates/singlefs-checker/src/walk.rs:633` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`），③ ④（deadlist）这一版不判（`crates/singlefs-checker/src/image.rs:248` 的注释）；I-3.6、I-3.7 在 checker 源码里零命中；I-9.25 判 `previous_snapshot_txg`（inode 树那条条目写的非 0 值不高于引用这张树表的根的 txg）。GPU 判器违例位 128 位、已用 92（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:79`–`:81`、`:110`–`:116`）。
- 删除的意图机制只认 unlink：`transaction.rs:4178` 的 `DeletionStepOfThePublish`（`NotADeletion` / `Intent` / `Batch` / `FinalBatch`），载体是 nlink = 0 的记录、`deleted_inodes` 的 key 与墓碑；logged_ops 树零根注册、种类码 12（`records.rs:430`）、没有字段表、没有写者（`transaction.rs:8694` 写 0，注释「字段表未定」）。logged_ops 的字段表归并行的 `m4-gc-r1` 那一轮（C364（整理意图记录没有字段表）），这一轮里销毁快照的意图只写「要装下哪几样」，不定字段表。
- 模型与 RefFS 的操作面：`crates/singlefs-harness/src/model.rs`（`ModelOperationKind` `:978`）与 `crates/singlefs-reference-filesystem/src/lib.rs:31`，都没有快照操作；随机历史 `crates/singlefs-harness/src/history.rs` 的 `HistoryOperation`（`:475`）12 种，没有快照。
- 崩溃放量的上限（GC 流与快照流都要对着估）：一个落点至多 64 次写（`crates/singlefs-checker-tier/src/crash_judge_tables.rs:51` 的 `MAXIMUM_WRITES_PER_LOCATION`），一条流至多 2048 次写（`crash_facts.rs:287`），全量展开深度 30（`scripts/crash/crash-amplification.sh:222` 的 `FULL_EXPAND_UP_TO`），一段的状态数装进 u64（`crash.rs:1548`）。
- 方案按 `crates/` 今天的实现来谈。别家的 deadlist 与 livelist 做法在 `.claude/kb/prior-art.md`「十、快照下的死块记账、搬被共享的块与 discard 时机」（未在本项目验证，只当线索）。

## 三、格

### S1　只读快照在盘上是什么
D8（核心索引结构） 已定项 8 写「快照进树表」，D5（快照 / 空间记账机制） 已定项 9 写头 ID = 该头 inode 树的树 ID，D6（快照实现模型） 已定项 1 取「每头一棵自己的树」。一个只读快照在树表里是几条条目、各什么种类、根指针指什么、头 ID 与 `previous_snapshot_txg` 各写什么：候选至少两种——甲：快照 = 新发号的一对（inode 树、extent 树）条目，根指针逐字抄建快照那一刻活头的两个根，头 ID = 快照自己的 inode 树 ID，快照条目的 `previous_snapshot_txg` = 它捕获那一版活头的值；乙：树表加一种新种类「快照」（码 14 起），一条条目装 (S.txg, 被捕获的头 ID, 活头两根指针的引用)。每种答：I-3.7（活快照可枚举） 的那个可枚举结构是什么、I-9.27 与 C675（同一种树在树表里至多一条没有不变量）（core 与 checker 都拒同一种树两条）要改成什么、「快照不许写」在写入口哪一处拒、`deleted_inodes` / 墓碑树 / 谱系树 / 退役树 / 三棵记账类的树快照各持不持（只读快照没有自己的 inode 号水位行：D5 已定项 4 第 12 项只为可写头维护）。

### S2　建快照那次发布写什么、切在哪
D16（发布语义） 已定项 14 把快照点钉在「本 checkpoint 全部数据变更之后」。建快照是不是一次独立的发布（像空发布那样只换根与树表，不带文件版本）；那次发布要写：新树 ID（从水位发，`issued_from_watermark` 之外要一条新路）、活头 inode 树条目的 `previous_snapshot_txg` := S.txg（D5 已定项 9、13「建快照只推进 prev(头)」、I-9.25）、deadlist 一条不动（D5 已定项 12）。建快照之前要不要先把 `deleted_inodes` 队列走空（D8 已定项 16 第 6 条每次用户改动先走空；队列非空时快照里的 nlink = 0 记录怎么算）。同一 checkpoint 两个快照（D16 已定项 14 允许、C250（`previous_snapshot_txg` 的父链解析非单射））第一版许不许。

### S3　快照下覆盖写与删文件：释放判定改在哪
D5（快照 / 空间记账机制） 已定项 19 的判定：旁表计数（第一版无克隆，恒按 1）→ `birth > head.previous_snapshot_txg` 立即释放进 defer → 否则追加进 head.deadlist。改法落在 `transaction.rs:3265` 的 `placements_to_release_via_mapping` 与 `:1861` 那处 `allocator.release` 之间，覆盖写、顺序写、删除批与最后一批共用；`head.previous_snapshot_txg` 的常驻载体从树表条目读回。deadlist 条目 key = 头的树 ID 8 + 纪元 8 + 中央映射 key 27、value 空（D5 已定项 12）；纪元 = 块被杀那一刻头的 `previous_snapshot_txg`。各答：deadlist 树走不走 `crates/singlefs-core/src/code_two_tree.rs` 的多层码 2 树规则、种类 8 的条目宽登记在哪（D8 已定项 11 的种类到条目宽表）、一次发布最多追加多少条（受 D3 已定项 2 已承诺预留与一次发布的记录上限约束）、记账哪几个统计量变（已分配不变、defer 待释放不变、第 3 项不可回收仍恒 0 还是开始算）、I-3.6（deadlist 紧界） 的判法与必红世界（把该直接释放的块放进 deadlist）。

### S4　删快照：意图、分批、deadlist 并到更新的一侧
D5（快照 / 空间记账机制） 已定项 12、13、16：写意图 (头, lo = S.txg, hi, P = prev(S).txg)，prev(S) 在写意图的那个事务里传给下一侧（活头的 `previous_snapshot_txg` := P，或下一个快照条目的）；之后按批放掉纪元落在 [lo, hi) 里 birth > P 的条目（释放进 defer）、不改任何条目的键；级联之间不要次序。下一侧从树表派生（C336（写意图那一刻找不到快照的下一侧））。各答：意图装下 (头, lo, hi, P, 续做点) 五样要几字节、住 logged_ops（字段表归 `m4-gc-r1`）还是像 unlink 那样用现有载体（快照条目自己写成「销毁中」）；分批的批大小与只供测试的开关；最后一批与删树表条目同一次发布；崩在中间每个状态 I-3.5 ③ ④、I-3.6 成不成立；删快照被空间拒推抬 F 再试（照 unlink 那条）；C337（被销毁的头何时不算活引用没有条款）：从写意图那一刻起不算活引用，checker 怎么判。

### S5　checker：I-3.5 ③ ④、I-3.6、I-3.7 的判法与必红世界
三条各写可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟遍历，D13 已定项 12 两个方向独立判）、判哪几版、输入取不到时报什么（D5 已定项 14「判据的任何一个输入取不到时默认进 deadlist」是写侧纪律，checker 侧对应什么）、每条一个必红世界（例：I-3.6 把 birth > prev 的块放进 deadlist；I-3.7 树表里的快照条目少一条；I-3.5 ③ deadlist 里有一条 birth > S.txg 的）。GPU 判器违例位各占一位（剩 36 位）。

### S6　崩溃流与态数
一·4 要三条流：建快照、快照下覆盖写（与删文件）、删快照，接在哪条已有的流后面（多记录流「取号 → 暖机 × 2 → A → B → C」之后，照 unlink 流的接法），每条流每次发布写几个单元、一段几次写，对着每落点 64 次写、每流 2048 次写、展开深度 30 估态数；每条流一个必红世界。

### S7　模型、RefFS 与随机历史
`crates/singlefs-harness/src/model.rs` 与 `crates/singlefs-reference-filesystem/src/lib.rs` 加建快照、读快照（按快照读文件内容）、删快照三种操作各要加什么状态；`HistoryOperation` 加哪几种、`KNOWN_RED_FORMS` 不新增的判据。

### S8　回退、冷启动与快照
管理员回退（`mount.rs:7998` 的 `roll_back_by_a_forward_publish`）回到建快照之前的一版：树 ID 集合不同（`mount.rs:8083` 今天拒）怎么处置——拒、或回退把快照一起抹掉（快照条目不在目标版的树表里，被快照钉住的 deadlist 条目随根 COW 一起消失，块经回退候选集判释放）；回退候选集与 I-3.5 出现规则的「按树继承下界」在有快照时怎么读；崩溃恢复落到建快照那次发布之前的根，快照不存在，deadlist 没写过——这两条是不是免费成立。

### S9　只读快照用到 D6 的哪几样
每头一棵树（D6 已定项 1）：只读快照算不算一个「头」；引用计数旁表（D5 已定项 6）：第一版无克隆，条目恒 0 条，判定里「没有条目按 1 算」照走；livelist（D6 已定项 3）：只给可写克隆，只读快照一条不写；D6 已定项 5 的 O(N) 反向索引查找：只读快照删快照走不走——逐样写「用 / 不用 / 留给可写克隆」，留给克隆的列进 `.claude/kb/milestone/04-fourth-txn.md`「开工前要先定的」那一段的归属判断。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | S1–S9 | 逐格从条款原文与 `crates/` 今天的代码推：每格给候选、每个候选要动 `crates/` 哪几个文件的哪一步、新写出哪些字节（照 `.claude/kb/layout/03-third-txn.md`「一、形态」那张表的列）、代价；推荐一个并说明依据；要交用户的岔路写成候选、翻面观测、够判条件三列，不带倾向 |
| 云端攻方（Opus） | S1–S9 | 找反例：每个候选在哪一种历史（建快照 → 覆盖写 → 删文件 → 删快照 → 回退 / 崩溃恢复交错）下丢数据、漏空间、归属错或让 checker 恒真；D5 已定项 14 边界表三行（`birth == S.txg`、`death == S.txg`、`birth == death`）在每个候选上各取一个具体历史核；能在工作树的拷贝上写小模型量的就量（拷到自己的草稿目录改，不在工作树里改） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写工作树 `~/.local/share/singlefs/work/m4-predev` 里的路径与行号。
- 每格给：判定、候选（各自定义一句）、每种候选的代价与要动的 `crates/` 文件、新写出的字节、要交用户的岔路（候选、翻面观测、够判条件，不带倾向）。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷工作树，不在工作树里改。
- 报告分段写：每次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
