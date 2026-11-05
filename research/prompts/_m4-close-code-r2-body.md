# m4-close-code-r2：里程碑四收口修补二批代码轮正文（2026-11-03）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T3.1 T7 G7 Y5 -->

## 一、题面

这一轮攻代码轮 m4-close-code-r1（判决 `research/prompts/m4-close-code-r1-main-verification.md`）判下来之后，照判决第三节合进主工作区的产品代码修补。射程是 `crates/singlefs-core/`、`crates/singlefs-replay-prefix/` 的 `src/`；同一批合进来的只改测试代码的几份（改钉锚点那一簇、会话层补测那一簇、树表照抄那一簇）不在射程里。

| 来源（补丁目录，`~/.local/share/singlefs/work/` 下） | 关的 | 落点（产品代码） |
|---|---|---|
| m4-close-code-r1-fix-b2-writer-2 | r1 判决 T7 代价（`container_tree.rs` 按全部容器数开标记表）与用户定案「管，只管准入之前」「五处加准入现在改，根源先量」（D16（发布语义） 已定项 10 的统计量那一句、`.claude/rules/fs-design.md`「记账是事务的副产品」第一格） | core：`admission.rs`、`container_tree.rs`、`mount.rs`、`recovery.rs`、`transaction.rs` |
| m4-close-code-r1-fix-d-writer | r1 判决里的 G7（整理的发布只在现行内容与最后一个根相同时发，协议核没写这个门）与 Y5 桥（core 的根环成形条件比协议少两条） | replay-prefix：`gc_ring_interval_proofs.rs`；core：`verified_model.rs` |

改动全文：`research/prompts/_m4-close-code-r2-diff.md`（两份补丁里落在射程内的 hunk，原样抽出）。

## 二、实现今天的样子（主 agent 的观测，2026-11-03）

- 冻结副本：`~/.local/share/singlefs/work/m4-close-code-r2/repo/`，取两份补丁都打进之后的主工作区（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`prune-server`）。腿读代码读这一份；行号以它为准。
- 这一段的验证（主 agent 经服务跑的，原话记在 `records/2026-10-13-里程碑四预开发.md`）：
  - 合入后验证（任务 2364）：「✓ 测试全绿（337 个测试二进制，共 3237 条测试，其中慢档 77 条；跑了 4 个包…）」；
  - 形式化轻型（任务 4486、4802）：core 各项 verified 之和 725、登记 725，replay-prefix 155、登记 155；这一批改过与新加的 51 行必红世界全绿；
  - 变异轻型（任务 5260）：这一批四簇追加与替换的 57 行全抓到；
  - checker 档快档不带特性那一趟（任务 5172）「合计通过 858、失败 0」；带 gpu 那一格红在显卡与判定库服务一侧，与这两份补丁无关，判决不重复。
- 容器小树那一件的落点（冻结副本现查）：`container_tree.rs:1941` `children_of_every_node`、`:2078` `ancestor_suffix_starts_of_the_rightmost_containers`、`:2256` `ancestors_of_the_rightmost_containers`、`:2369` `after_a_container_is_opened_at_the_right_end`、`:2612` `node_counts_before_this_publish`、`:2729` `first_rewritten_container`、`:2874` `node_origins`；`transaction.rs:3309` `ContainerTreeRootUnitPositions`、`:4018` `pointer_below_the_root`、`:4188` `inode_record_found_by_its_leaf`、`:4299` `tombstone_tree_internal_levels_read_from_its_root_node_header`；`admission.rs:288` `InodeRecordCounts`（`:302` `counted_over_the_inode_tree`、`:326` `after_writing`、`:357` `after_removing`）；`recovery.rs:3396` 恢复时数一遍当起始值。
- 整理门那一件的落点：`gc_ring_interval_proofs.rs:243` `run_ring_step_is_enabled`、`:1090` `lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root`；`verified_model.rs:212` `root_ring_is_shaped`、`:225` 见证。
- 准入之前还剩的三处照抄（实现员报告点名，主 agent 认，归 E199（发布照抄上一版的代价））：`transaction.rs` 照抄那一支的 `shape_before.clone()` 两处与 `container_tree.rs` 追加那一支的 `shape_before_this_publish.clone()`。这三处是新一版自己的形状，不在这一轮的格里，攻方不为它们开格。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　开片原地改与上一版只记每层节点数
`after_a_container_is_opened_at_the_right_end` 按值拿形状、原地改，只碰停的那一层及其之下各层的末尾；`ContainerTreeAfterThisPublish::node_counts_before_this_publish` 只存每层节点数，规格 `every_node_absent_from_the_previous_version_is_rewritten` 改按它判。核：长高、正好填满一层、连开几片跨层这几处，新一版形状与「逐层照抄再改」那一套逐个相同；只记节点数之后，哪个判断原来要读上一版的孩子数、现在读不到了。

### T2　求祖先从最右端往左
`ancestor_suffix_starts_of_the_rightmost_containers` 每层从最右端往左减到被标过的头一个就停，`node_origins`、`first_rewritten_container` 与 `transaction.rs` 的两处 `position >= first` 成员判断跟着换。前提是「这一次重写的容器是最右端连着的那几片」（规格 `rewritten_containers_are_the_rightmost`）。核：这个前提在每一条会重写容器的发布路径上都成立吗（追加、开片、unlink 合并叶、截断、整理搬迁、回退之后第一次发布）；有一条不成立时，哪一处会把不在最右段的容器漏掉或多算。

### T3　孩子区间顺着累加与按位置取指针
`children_of_every_node` 每层从最左累加一遍，`children_of` 删掉；`pointer_below_the_root` 按「这一层之前各层节点数之和 + 同层序号」取下标、再断言一次；`ContainerTreeRootUnitPositions` 在装好 units 时记两个根单元的下标，发布、重建、管理员回退三处各记一次。核：units 的次序在这三处之外还有没有被改动的地方（搬迁、清扫、截断、挂载续做），改动之后记下的下标还对不对；断言失败时是 panic 还是返错，合不合 `.claude/singlefs-ai-sop/rules/code-discipline.md` 的规矩。

### T4　准入不遍历 inode 树
`InodeRecordCounts` 随一版带着，发布时按这一次写进与删掉的记录改，上一版那条记录按叶序折半找；恢复时 `counted_over_the_inode_tree` 数一遍当起始值。核：哪几条会改 inode 记录的路径（unlink 的 nlink 归零、截断改 size、建快照、删快照、回退、整理）没走 `after_writing` / `after_removing`；维护值与现数不等时谁会先看见（checker 有没有一条拿现数比它）；恢复数一遍是不是在准入的热路径上。

### T5　整理门与根环成形
G7：`run_ring_step_is_enabled` 整理那一支要「至少一个根、现行逻辑内容等于最后一个根」，投影引理多一条等式；前提在 core 一侧靠攒批会话排空待发之后才起整理（`mounted_session.rs` 的 `run_compaction_in_the_background`，这一轮不改它）。Y5：core 的 `root_ring_is_shaped` 补「内存 F 不高于最后一条根的号」「持久 F 之上的根装得进根环」两条。核：协议这一门与 core 的实际行为是不是同一件事（core 里整理的发布照 `previous` 的树搬单元，`rollback_floor` 取上一版的）；补的两条成形条件在 core 的值上成立吗（按 checkpoint 号写，与协议按下标写的换算对不对）；有没有一条历史让 core 发出协议这一门不许的整理发布。

## 四、第二轮分工（照 `.claude/rules/three-way-inference.md`「多轮」一节）

这一轮攻的是 r1 判决的修补本身。攻方腿：第一轮的修补本身（上面五格各自的改法有没有新破绽）/ 产生这些结论的方法（对拍取样点、钉代价的测试、必红世界行是不是真罩住了改法）/ 从未被看过的地方（这五格之外、被这两份补丁连带改到的调用点）。不许重复 r1 已攻过的角度：r1 判决第三节各格已判成立的结论不重攻，除非这一轮的改动碰到了它的前提。正推腿逐格核「代码做的是不是条款与判决说的」。
