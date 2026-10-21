# 截断（truncate）的设计第一轮正文（2026-10-16）

<!-- doc-lint:not-numbers W1 W2 W3 W4 W5 W6 W7 W8 W9 G1 G3 K-U1 K-U5 K-U6 K-R3-1 -->

## 一、题面

里程碑四作业点六·1（`.claude/kb/milestone/04-fourth-txn.md`「六　截断（truncate）」）：单文件 truncate 到 N 字节。用户 2026-10-16 定两件：缩短与变长（size 改大、中间留洞）都做；真盘掉电（项五）排最后，截断先做。缩短走删除的意图与分批回收（D8（核心索引结构） 已定项 5「凡是可能修改无界数量 extent 的操作（truncate、collapse、删大目录），先把意图写进 logged_ops 树，再分批推进」），被快照引用的尾部块进 deadlist；变长留洞，「洞就是缺席」（D8（核心索引结构） 已定项 14）。punch（不改 size 的部分删除）不在这一轮：C778（不改 size 的部分删除怎么记没有条款） 挂里程碑十一。

这一轮要的是：截断的每一种形态（缩到单元边界、缩到单元中间、缩到 0、变长到单元内、变长跨过几个单元留洞、在快照下缩）各写成「意图 → 分批 → 最后一批」每一步改哪些盘上结构、读者在每个中间态答什么、checker 多判哪几条、模型与 RefFS 怎么表示，每一格给候选、代价（要动 `crates/` 哪几处、新写出哪些字节）、要交用户的岔路。

被测条款（材料附录整节抄）：D8（核心索引结构） 已定项 5、6、14、16；D18（块里携带什么信息） 已定项 10；D5（快照 / 空间记账机制） 已定项 12、13、19；D26（后台整理与放置回收） 已定项 2（logged_ops 字段表，G1 那一份）；D16（发布语义） 已定项 5；D19（块指针的结构与宽度预算） 已定项 6；D4（校验和位置） 已定项 2；D13（验证路线） 已定项 4、9。不变量 I-2.3（补齐字节为零且参与校验和）、I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-9.5（删除队列里的号仍在树里）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-1.13（扫描重建与遍历方向的索引相同）、I-1.9（清扫水位内无可读垃圾）。欠账 C1（意图续做做成常规路径）、C2（无界操作做成单事务）、C364（整理意图记录没有字段表）、C711（单元数对 size 只判了 nlink = 0 的记录）、C778（不改 size 的部分删除怎么记没有条款）、C827（扫描重建对删除中与覆盖写变短的镜像报不适用）。

## 二、实现今天的样子（主 agent 的观测，2026-10-16，事实表 `~/.local/share/singlefs/work/m4-trunc-facts/facts.md` sha256 `040f9b07…23b0`，行号按工作树 `~/.local/share/singlefs/work/m4-predev`）

- 文件级 truncate 零代码：`grep -rn -i 'truncat' --include=*.rs crates/singlefs-core/src crates/singlefs-harness/src crates/singlefs-reference-filesystem/src crates/singlefs-checker/src` 只有 `Vec::truncate` 与 `OpenOptions::truncate` 四处。测试里的「截断」是整份重写成更短的内容（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` 的 `pool.overwrite(0)`）。
- unlink 的意图与分批（样板）：意图那次 `crates/singlefs-core/src/transaction.rs:8090` `publish_unlink_intent`——记录 nlink 写 0、size 不变、`deleted_inodes` 加号、墓碑树追加整对象墓碑，不立 logged_ops；意图里没有目标态字段，下一步只由「现行那一版还剩几个数据单元」定（`crates/singlefs-core/src/unlink.rs:133` `next_unlink_step`）；一批 144 个单元（`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`，测试开关可压小），每批之后 size = 留下单元数 × 净荷（`transaction.rs:8172`–`8188`）；最后一批在 `transaction.rs:6160` `ordinary_allocation_part` 一次发布里删记录、换 extent 树、删 `deleted_inodes` 的号；会话与挂载续做共用 `transaction.rs:8236` `walk_the_deletion_queue_until_it_is_empty`（会话 `crates/singlefs-core/src/mounted_session.rs:1071`，挂载 `crates/singlefs-core/src/mount.rs:6553`）。被截那一片下段叶与祖先由 `unlink.rs:181` `plan_the_extent_tree_after_a_deletion_batch` 重写。
- logged_ops：整理已是它的第一个写者（D26 已定项 2 字段表：key = 种类 2 + 签发 txg 8，种类码只登记了 1 = 整理；I-4.4 按白名单判 value），删除那一族没用它。
- 洞：格式允许（`crates/singlefs-core/src/extent_tree.rs:9`「洞就是缺席」），结构读者按位置读、不要求连号（`extent_tree.rs:804`–`901`，读码推的）；但写路径只造从 0 起连号的文件（`extent_tree.rs:210` `lower_segment_nodes_of_a_file_without_holes`、`transaction.rs:5739` `plan_the_extent_tree_after_this_publish`），内存态 `TransactionOutput::data_pointers: Vec<DataPointer>` 按下标当单元号（`transaction.rs:2931`），冷走读拒洞（`crates/singlefs-core/src/recovery.rs:4743`–`4750`、`:5376` 单元个数要等于 ⌈size ÷ 净荷⌉.max(1)），挂载态读拒洞（`crates/singlefs-core/src/mounted_read.rs:795`–`822`）；内联指针只罩单元 0（`extent_tree.rs:1157`）；池级 checker 没有一处判文件洞（主走读 `crates/singlefs-checker/src/walk.rs:4492` 只判 I-1.1）。
- size 与尾单元：size 只住 inode 记录偏移 40（`crates/singlefs-core/src/records.rs:50`–`74`，blocks 由 size 现算）；三条写入口（`publish_first_file`、`publish_overwrite`、`publish_sequential_write`）都是整份内容从偏移 0 重写，size = 内容长度，写不到 size 之外；尾单元只装 `size − 起点` 字节、声明长度就是它，补齐为 0（`crates/singlefs-core/src/unit.rs:259`–`262`）；读者核「声明长度 = min(size − 起点, 净荷)」（`recovery.rs:5436`–`5445`、`mounted_read.rs:1089`–`1095`），越过 size 的读当场拒 `RangeBeyondEndOfFile`（`mounted_read.rs:906`–`922`）。推的：截断点落在单元中间而那个单元不重写，读者判声明长度不符；变长而旧尾单元不重写，同样不符。
- 快照下的释放判定：删除批与覆盖写都经 `transaction.rs:4841` `deadlist_changes_of_this_publish`（判定 `snapshot_lifetime_verdict` `:5073`，开关 `SnapshotLifetimeJudgement` `:5038`）；不写内容的发布靠 `DeletionStepOfThePublish::data_units_after_this_publish`（`:4557`）交「留下几个」，`previous_roles_without_a_successor`（`:6082`）把下标 ≥ 它的 `Data(i)` 算作换下；`roles_replaced_via_mapping`（`:3632`）的注释把「不含 `Data(0)` 的数据角色」当删除批截掉的尾部（`:3641`–`3646`）；判生死那一组与释放那一组在 `:9054`–`9070` 断言相等。
- 扫描重建：`crates/singlefs-checker/src/rebuild.rs:743`–`750` 落在 ⌈size ÷ 净荷⌉.max(1) 之外的数据单元报不适用 `DataUnitBeyondTheSizeOfItsRecordCoveredByNoClause`（C827），读法 Z「size 之外判死」没落成「不认领」；只认整对象墓碑（`rebuild.rs:658`–`675`）；`crates/singlefs-core/src/region_scan.rs` 不读 size 与对象出生代。
- 模型、RefFS、翻译、随机历史：理想模型 `crates/singlefs-harness/src/model.rs:567` 只有整份字节 `content: Rc<[u8]>`、只罩 inode 1；RefFS `crates/singlefs-reference-filesystem/src/lib.rs:20`–`56` 只有整份字节、操作里没有 unlink；翻译层遇写了盘的 unlink 就停（`crates/singlefs-harness/src/reference_filesystem_translation.rs:421`）；随机历史加一种操作要动的十处见事实表第六节（`crates/singlefs-harness/src/history.rs` 的 `HistoryOperation`、`HistoryOperationKind::ALL`、比重表、`draw_operation`、`note_outcome`、执行器、`apply_operation`、`simpler_variants`）；用户改动入口 `mounted_session.rs:98` `enum UserChange`。
- 第一版只有 inode 1 写数据（`transaction.rs:8166`–`8171` 注释「第一版只有第一个文件写数据」，写入口 `transaction.rs:8320` `live_record_of_the_first_file`）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### W1　意图载体与目标态
D8 已定项 5 要截断走「先写意图、再分批」，意图声明目标态（「把文件 X 截到 N 字节」）。候选：甲 logged_ops 新种类码（截断），value 装 (inode, 对象出生代, 目标 size)，与整理同一张字段表、I-4.4 白名单加一种；乙 照 unlink 的形状不立 logged_ops，意图那次就把记录的 size 写成 N、截掉的单元靠「size 之外还有单元」认出来（意图载体 = 记录本身）；丙 别的。各答：哪一种在任何一个中间态都读得出「还没做完」与目标是多少；与 unlink 队列、整理意图同一版里并存时谁先走（D8 已定项 16 第 6 条「每一次用户改动先把队列走空」）；承诺类要可写挂载交回之前走到空（D8 已定项 5 意图分类那一句），续做入口放哪；一个 inode 同时只许一条截断意图还是可叠；截到更短的第二次截断落在第一次没做完时怎么合；对已在删除中（nlink = 0）的 inode 截断拒成什么。缩到的 N 落在原 size 之内但只差不到一个单元、不放任何单元时，要不要意图（一次发布能做完的是不是就不写意图——D8 已定项 16 第 2 条「只有这一条路，不设一次做完的快路」对截断成不成立）。

### W2　分批与最后一批同事务
从尾部按单元分批，一批 K 个（与 unlink 共用 `UnlinkBatchSize` 还是另立）；每批之后的中间态：记录的 size 是多少（甲 意图那次就是 N、单元逐批放；乙 每批改成「留下单元数 × 净荷」、最后一批写 N，照 unlink；丙 别的），读者在中间态答什么（读到 N 之外报越界，还是读到旧内容）——D8 已定项 5 第 3 条「中间状态必须每一个都是合法的文件系统状态」；最后一批与删意图同一次发布（D8 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」）落在 `ordinary_allocation_part` 的哪一处；续做的幂等判据照 D8 已定项 16 第 7 条五条逐条在截断上写一遍（①b 两份盘逐字节相同对截断成不成立）；一批写入量不随文件大小增长（C2 的斜率检查，截断那一格）。

### W3　截断点落在单元中间：尾单元怎么 COW
留下的最后一个单元 k 只剩 `N − k × 净荷` 字节：甲 意图那次就重写单元 k（新声明长度、补零），单元 k 的旧版进释放判定；乙 最后一批重写；丙 不重写、改读者的声明长度核对（读到 min(声明, size − 起点)）。各答：哪一种不让任何一个中间态出现读者判声明长度不符（`recovery.rs:5436`–`5445`、`mounted_read.rs:1089`–`1095`）；D16（发布语义） 已定项 5「一个事务最多写一个单元的用户数据」与 D19 已定项 6 那一句「截断的尾单元……都算」；I-2.3 补零在重写后的单元上成立；只重写 `Data(k)` 而不从第 0 个单元写起的发布今天没有，`roles_replaced_via_mapping` 那条「不含 `Data(0)` 就是尾部」的判别怎么改、`:9054`–`9070` 那条相等断言怎么守；截到 0 时（留一个声明长度 0 的单元，还是一个不留——今天「长度 0 的内容也写一个声明长度 0 的数据单元，一个单元都没有的文件写不出来」，`extent_tree.rs:10`–`11`）；内联指针只罩单元 0 时截到一个单元以内的 extent 树形态。

### W4　快照下截断
被快照引用的尾部单元进 deadlist（D5 已定项 13、19），重写的尾单元 k 的旧版同样问判定：照 `deadlist_changes_of_this_publish` 复用要满足的（事实表第四节 1–6 条）逐条核；快照读者读截断前的内容、活头读截断后的；截断意图跨过建快照（意图那次之后、最后一批之前建快照）时，之后各批放的单元判给快照还是释放；删快照与截断意图同在队列里时的次序（`walk_the_deletion_queue_until_it_is_empty` 先走 `deleted_inodes` 再走销毁中的快照）。

### W5　变长与洞
size 从 s 改大到 N：甲 只改记录的 size，[s, N) 全是洞（不写单元）；乙 旧尾单元（若不满）重写补零到单元末尾再留洞；丙 别的。各答：旧尾单元声明长度 < min(N − 起点, 净荷) 时读者那条核对怎么改（声明长度之后、size 之内读出 0 是「洞」还是「坏单元」）；变长要不要意图（不放任何单元，一次发布写得完）；读者读洞答 0 的路径（冷走读与挂载态读今天都拒洞）；`TransactionOutput::data_pointers` 按下标当单元号的内存态怎么表达洞（改成按单元号索引的映射，还是别的），要动哪几处（事实表第二节写路径表）；洞之后又整份覆盖写（今天三条写入口都从偏移 0 写整份内容）时洞被写成 0 的单元还是保持洞；变长之后再缩回洞里（N 落在洞中）尾单元怎么办；I-9.15 blocks 按 size 算不按分到的空间（D8 已定项 6）对洞成立；单元数对 size 的检查（C711、`recovery.rs:5376`、`mounted_read.rs:812`）在有洞时换成什么。

### W6　扫描重建与读法 Z
D18 已定项 10「文件变短不写新记录，扫描重建时数据单元按重建出的 inode 记录的对象出生代与 size 判死活」：size 之外的单元今天报不适用（C827），截断落地后是不是改成「不认领」；截断中间态（意图之后、最后一批之前）size 之外还挂着的单元在扫描重建与遍历两侧各怎么算才逐项相同（I-1.13）；洞在扫描重建一侧（`rebuild.rs` 按 (inode, 偏移) 一项一项比，不要求连号）成立的理由；I-1.9 未实现与这一格的关系。

### W7　checker 多判哪几条
截断意图（若走 logged_ops）的 I-4.4 白名单与新种类码；截断中间态的「有账」（照 I-4.7 的写法：size 之外还有单元的 inode 要么有截断意图、要么一个不剩）；活 inode 的单元数对 size（C711）在有洞、有截断意图时的判法；尾单元声明长度对 size（今天只在 core 的读者里判，checker 没有）；GPU 判器的覆盖表与内核（新条目进对拍不比那张表还是内核置位）。每条给必红世界。

### W8　模型、RefFS、翻译层、随机历史
理想模型怎么表示截断与洞（整份字节补 0 能不能区分洞与写过的 0——读者答案相同，但盘上单元数不同，模型要不要知道）；`ModelPublishKind` 加哪几个成员（意图、留单元的一批、最后一批、变长）、`units_replaced_by_the_publish` 怎么算；RefFS 加 `Operation::Truncate(n)` 的允许集合（崩溃落在意图与最后一批之间时读出的内容：截前、截后，还是任一批之后的前缀）；翻译层怎么交；随机历史加 `HistoryOperation::Truncate`（比重进哪几组、`simpler_variants` 怎么缩）；`UserChange` 加哪一种、会话入口拒哪些（删除中、没有文件、N 超过上限）。

### W9　崩溃流、必红世界与只供测试的开关
六·3 要截断中间崩溃的流接进崩溃放量：接在哪条已有的流后面（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` 那条，D8 已定项 16 射程「截断成 0 另立一条流」），缩到单元中间、缩到 0、变长留洞、快照下缩各要不要一条；必红世界：改坏「最后一批与截断意图同事务」那一处，那条流的判器红；只供测试的开关（批大小、强制尾单元重写在意图那次还是最后一批）照 `.claude/rules/fs-design.md` 五条硬要求第 2 条。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | W1–W9 | 逐格从条款原文与 `crates/` 今天的代码推：每格给候选、每个候选要动 `crates/` 哪几个文件的哪一步、新写出哪些字节（照 `.claude/kb/layout/03-third-txn.md`「一、形态」那张表的列）、代价；推荐一个并说明依据；要交用户的岔路写成候选、翻面观测、够判条件三列，不带倾向 |
| 云端攻方（Opus） | W1–W9 | 找反例：每个候选在哪一种历史（写 → 截短到单元中间 → 批间崩 → 续做 → 变长 → 覆盖写 → 建快照 / 删快照 / 回退 / unlink / 整理交错）下双重释放、读到陈旧或越界内容、读者判单元不符、意图永远做不完、扫描重建与遍历不一致；能在工作树的拷贝上写小模型量的就量（拷到自己的草稿目录改，不在工作树里改） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写工作树 `~/.local/share/singlefs/work/m4-predev` 里的路径与行号。
- 每格给：判定、候选（各自定义一句）、每种候选的代价与要动的 `crates/` 文件、新写出的字节、要交用户的岔路（候选、翻面观测、够判条件，不带倾向）。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷工作树，不在工作树里改；每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo，用自己的编译目录。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
- 报告分段写：每次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
