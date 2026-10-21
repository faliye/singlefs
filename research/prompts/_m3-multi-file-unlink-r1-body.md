# m3-multi-file-unlink-r1：第一个文件之外还有文件带数据单元时，删除各步怎么走（推论轮第一轮）正文（2026-10-19）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 Q1 Q2 Q3 Q4 -->

## 一、题面

里程碑三一·1 要的多 inode 写已经给 core 加了入口：`transaction::publish_sequential_write_of_an_inode` 与 `publish_overwrite_of_an_inode`，第一个文件之外的 inode 也能写数据单元（实现员报告 `~/.local/share/singlefs/work/m3-close-w5-multi-inode-writer/report.md`，sha256 1ed95e6f7412c1ee7e510da283491b633bc2e92121e24e6bbdaf3cce9e90e7c9）。D8（核心索引结构） 已定项 16 的 unlink 各步是照「第一版只有第一个文件写数据」写的，射程写「第一版只做单文件 unlink、按 inode 号、无目录、不带 truncate」。现在三样没有条款：删一个第一个文件之外、带数据单元的文件；第一个文件之外还有文件带数据单元时删第一个文件；删除让 extent 树上段的一片叶变空、或让上段根罩的 inode 号范围变小时，上段形状怎么变。实现员先在意图那次之前拒掉这几种 unlink（`PublishError::DeletedInodesQueueRefused(UnlinkWhileAFileOtherThanTheFirstHoldsDataUnitsIsNotDecidedByTheClauses)`），用户 2026-10-19 在弹窗里定「本轮补删除条款」。

同一次弹窗用户还定了：删完第一个文件、再建 inode 时，新入口拒写数据（「拦，K-U5 照旧欠着」，D8 已定项 16 射程那句「删完之后不再建带数据的文件（K-U5，记欠账）」照旧成立，C618（删完之后不能再建带数据的文件） 照旧挂着）。这一轮不重开 K-U5，但要答清「删完第一个文件」在多文件时指哪一版（见 U3）。

这一轮要的是：把多文件的删除各步写成能落地的条款，逐格给候选与代价（要动 `crates/` 哪几处、新写出哪些形态），找每个候选罩不住的历史；要交用户的岔路写成岔路单的形态。

被测条款（材料附录整节抄）：D8（核心索引结构） 已定项 5、6、14、16；D18（块里携带什么信息） 已定项 10、24；D16（发布语义） 已定项 1、5；D5（快照 / 空间记账机制） 已定项 19。不变量 I-4.7（删除中 inode 有账）、I-4.10（截断中间态有账）、I-3.4（可用空间扣待删占用）、I-9.5（删除队列里的号仍在树里）、I-1.13（扫描重建与遍历方向的索引相同）。欠账 C618（删完之后不能再建带数据的文件）。

## 二、实现今天的样子（主 agent 的观测，2026-10-19，行号按工作树 `~/.local/share/singlefs/work/m3-multi-file-unlink-r1-tree`：此刻的主工作区 `crates/` 加上述实现员的补丁，补丁还没打进主工作区）

- extent 树两段都按位置寻址、不分裂（`crates/singlefs-core/src/extent_tree.rs` 模块文档第 1–23 行）：上段叶 j 罩 inode 号 `[j × 143, (j + 1) × 143)`（`EXTENT_TREE_UPPER_LEAF_INODES` = 143，`crates/singlefs-format/src/lib.rs:186`），扇出 147（`EXTENT_TREE_INTERNAL_FANOUT`，同文件 176 行）；「上段的根是它最高那一层的第 0 个节点（罩从 inode 0 起的一段），层级是罩得住有条目的最大 inode 号的最低一层」；上段叶条目带标签字节，0 是「没有单元」。下段一个文件一棵，叶罩 144 个单元（`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）；只有一个数据单元的文件不建下段，上段叶条目内联那个数据指针。
- 写别的文件的上段计划：`crates/singlefs-core/src/transaction.rs:6426` `plan_the_extent_tree_after_writing_a_file_other_than_the_first`，上段节点取「各有条目 inode 那一串的并」，层级取最大；上一版只有零条目根而写的号落在第 0 片叶之外时，零条目根换下、不留空叶（实现员报告 Q3，主 agent 还没认这一读法）。
- unlink 的批：`crates/singlefs-core/src/unlink.rs:157` `plan_the_extent_tree_after_a_deletion_batch`，下段按留下的单元数重算、整个落在截掉那一段里的节点换下，上段这个文件那一串重写、别的照抄，「上段根的层级不变（这一批不改 inode 号的集合）」。最后一批只有第一个文件那一支：`unlink.rs:234` `plan_the_extent_tree_after_the_final_batch_of_the_first_file` 断言「上段只有根兼叶一个节点」，把它重写成零条目根。
- 下一步怎么取：`transaction.rs:9279` `next_unlink_step_after` 里「第一版只有第一个文件写数据：别的 inode 一个数据单元都没有，意图之后直接是最后一批」——inode ≠ 1 时剩余单元数恒取 0。
- 拒绝：`transaction.rs:9712`–`9740` `refuse_an_unlink_the_deletion_steps_do_not_cover`，第一个文件之外有文件带数据单元时，删第一个文件或删那个带数据单元的文件在意图那次之前拒；删别的没有数据单元的 inode 照做。
- `TransactionOutput.files_other_than_the_first`（补丁新加）按 inode 号记别的文件的数据指针与下段节点指针；`TransactionUnit` 新加 `DataOfAFileOtherThanTheFirst`、`ExtentLowerNodeOfAFileOtherThanTheFirst` 两个角色。恢复（`crates/singlefs-core/src/recovery.rs`）收第一个文件之外的 extent 条目，要求至少一个单元、单元连号、尾部没有洞。
- 截断与变长对别的 inode 拒（`SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses`，原有）；快照读只读 inode 1（`read_the_first_file_of_a_snapshot`），快照照样罩住别的文件的块。
- 删空判定（D8 已定项 16 第 5 条）要 extent 树零条目根；`RebuiltVersion` / `PreviousVersion` / `PoolVersion` 第三个成员是「有树、文件集合为空」。
- 池级 checker 的删除相关判法：I-4.7、I-9.5 与扫描重建对墓碑压着的对象（`crates/singlefs-checker/src/rebuild.rs` 的 `tombstoned_objects` 那一判，「删除中（墓碑压着的对象还有记录）」报不适用）。
- 方案按 `crates/` 今天的实现（上面那棵树）来谈。

## 三、格

### U1　删一个第一个文件之外、带数据单元的文件
意图、分批、最后一批三步能不能照 D8 已定项 16 第 1、2 条原样套到 inode N ≠ 1 上：批大小 K、size 怎么改、最后一批删记录与 `deleted_inodes` 那条 key 同一次发布；最后一批对 extent 上段做什么——那一格条目写成标签 0（「没有单元」）还是从叶里去掉，两种在读者、恢复（「至少一个单元」那一判）、池级 checker、扫描重建上各怎么判。逐条写出要改 `crates/` 哪几处（`next_unlink_step_after`、`unlink.rs` 的两个计划函数、`refuse_an_unlink_the_deletion_steps_do_not_cover`、恢复）。

### U2　上段形状随删除收缩
删除让一片非根上段叶没有条目、或让有条目的最大 inode 号变小时：那片叶换下还是留成零条目叶；根层级降不降；降的时候旧根下面的节点怎么处置。要求和写路径同一条规则（写路径照「各有条目 inode 那一串的并」与「罩得住有条目的最大 inode 号的最低一层」）：同一个「有条目的 inode 集合」只许有一种上段形状吗？写出那条规则的一句话；如果写与删两边形状规则不同，D8 已定项 16 第 7 条判据 ①b（同一份崩溃镜像各自续做，两份盘逐字节相同）与「逻辑内容」口径会不会出事。删到只剩第一个文件、只剩别的一个文件、全删空，三种终态各长什么样。

### U3　第一个文件之外还有文件时删第一个文件
删第一个文件的最后一批（今天断言上段只有根兼叶）改成什么；删完第一个文件而别的文件还活着，这一版是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的哪个成员（「有树、文件集合为空」不成立，第一版的成员里有没有对得上的）；第一个文件删完之后还活着的别的文件能不能接着写、能不能删（用户定的 K-U5「拦」管的是「删完之后再建的 inode 写数据」，这一版上原有的别的文件算不算）；删第一个文件与删别的文件次序不同的两段历史，终态是否逐字节相同（判据 ①b 之外，「逻辑内容」口径相同即可的写明）。

### U4　续做、队列、墓碑与 checker
`deleted_inodes` 至多一条 key（第 6 条）在多文件时还成立吗（会话每次用户改动先走空）；挂载续做对多文件；墓碑每个对象一条（D18 已定项 24）对别的文件的对象出生代怎么写；快照下删别的文件（D5 已定项 19 的 deadlist）；池级 checker 的 I-4.7、I-9.5、I-3.4、I-4.10 与扫描重建（I-1.13）对第一个文件之外的删除中 inode 各要不要改判法，改哪一句；幂等五条判据 ①–④ 在多文件历史上要加哪些取样点。

### U5　每一处要改的地方与代价
按候选列：核心层（`unlink.rs`、`transaction.rs`、`deleted_inodes.rs`、`mount.rs` 续做、`recovery.rs`）、池级 checker（`crates/singlefs-checker/src/` 与不变量清单）、GPU 判器（若有新判、恢复内核读上段那一段）、理想模型（`crates/singlefs-harness/src/model.rs`）、RefFS、崩溃流（删多文件要不要一条新流，挂在哪个里程碑三作业点）、变异与必红世界。每处给改法一句与估的行数（推的写明推的）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | U1–U5 | 逐格从条款原文与上面那棵树的代码推：每个候选要动 `crates/` 哪几个文件的哪一步、写出哪些形态、代价；把每格推成可落地的条款一句并推荐一种具体形态；要交用户的岔路写成候选、翻面观测、够判条件三列，不带倾向 |
| 云端攻方（Opus） | U1–U5 | 找反例：哪种多文件历史下（交错写与删、删一半崩溃、续做、快照、回退、上段长高之后删、删到只剩一个）删除各步会丢单元、留下孤儿上段节点、让同一逻辑内容出两种上段形状、或让池级 checker 与恢复判得不一致；能在那棵树的拷贝上写小模型量的就量（拷到自己的草稿目录改，不在那棵树里改） |
