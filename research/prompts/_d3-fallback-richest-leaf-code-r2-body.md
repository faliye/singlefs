# d3-fallback-richest-leaf-code-r2：用户数据落点扫描换算法与第一轮判决的改法，代码轮第二轮正文（2026-11-12）

<!-- doc-lint:not-numbers R S C K -->

## 一、题面

第一轮判决 `research/prompts/d3-fallback-richest-leaf-code-r1-main-verification.md` 第三节「改法」列了五条，并进同一批由实现员一次改完、打进了主工作区。这一轮判的是这一批修补本身：

- 第 5 条是用户 2026-11-12 定的，原话「改实现 优化算法」：`DeviceFreeMap::lowest_user_data_slot_outside_the_region` 换算法，落点不许变。调查读数在 `~/.local/share/singlefs/work/e207-slow-item-investigator/result.md`。
- 第 1 到 4 条是第一轮打中的名字、裸整数、一条参数循环用例、三处过时注释。

改动全文：`research/prompts/_d3-fallback-richest-leaf-code-r2-diff.md`（产品代码三份，照三份补丁打进主工作区的次序排）。三份实现员报告：`~/.local/share/singlefs/work/user-data-placement-scan-writer/result.md`、`~/.local/share/singlefs/work/user-data-placement-scan-writer-2/result.md`、`~/.local/share/singlefs/work/user-data-placement-scan-writer-3/result.md`。

这一轮派辩方腿与攻方腿。辩方复核第一轮判决里「没采」的那几条够不够得着（攻击 1c 没采、固定点终止只记欠账），攻方攻这一批修补。

## 二、实现今天的样子（主 agent 的观测，2026-11-12）

- 被判的产品代码：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/admission.rs`（开工快照 `research/prompts/d3-fallback-richest-leaf-code-r2-snapshot/`）。`admission.rs` 在这一批里只改了两处注释。
- 快照这棵树上，这三份文件在这一批之后还有两处别的改动，写在 diff 材料末节：主 agent 给一条单测的闭包循环变量改名（归这一轮）；另一个会话把 `unblocked_slot_pairs_outside_the_cluster_segments` 的返回值与 `admission.rs` 的 `slot_pairs_for_data_units` 换成 `SlotPairCount`（归那个会话自己的代码轮，腿只攻它与这一轮 diff 的交叉）。腿引的行号以快照这棵树为准。
- 新算法（`allocator.rs`，行号是快照这棵树的）：`DeviceFreeMap` 多一项 `segments_with_an_unblocked_slot_pair: BTreeSet<usize>`（第 485 行；建空闲图时在第 660 行初始化），在 `set_blocking_bit`（第 748 行）里随每段的没挡槽对数翻过 0 增量维护；`lowest_user_data_slot_outside_the_region`（第 1320 行）按段号升序走这个集合，段起点在 `cluster_segments` 里或在 `R` 里的跳过，落进的那一段交 `lowest_unblocked_slot_pair_in_the_segment`（第 1341 行）按 2 槽看至多 32 对。原逐候选实现留成 `#[cfg(test)]` 的 `lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate`（第 1366 行），只供对拍。
- 名字与类型：`CommitGeneratedDeviceAnswer::FallBack`（第 197 行）、计数 `commit_generated_placements_that_fell_back`（第 2659 行）、newtype `AllocationRecordTreeLeafIndex`（第 203 行）与 `SlotCountInALeaf`（第 227 行）、新拒绝成员 `PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice`（第 297 行，分法在第 305 行的 `refusal_of_commit_generated_answers_that_differ`）。
- 调用点：`try_allocate_user_data`（第 2923 行）每块盘调一次新实现；产品路径的调用点三处，实现员列在第一份报告第三节。
- 验证到的数（量的；除写明的以外都在「主工作区加补丁」的副本上）：
  - 证红 23 行全抓。其中一行头一趟没红（任务 14606），实现员判成取样点不敏感、给增量维护那条单测加了固定形状的前缀后抓到。
  - 核心库单测「112 passed」。验证包 14 个测试目标过（任务 14473）。clippy 净（副本上没带 `.claude/singlefs-ai-sop/scripts/check.sh` 的 `CODE_DISCIPLINE_LINTS` 七条；打进主工作区之后带全七条跑出一处 `clippy::shadow_unrelated`，主 agent 改名后任务 14857 过）。拒算登记 12 条对 12 行。
  - 主工作区上读文本的四样都过：命名四格（任务 14782）、命名检查（14783）、崩溃枚举用例静态判据（14785）、变异行静态核（14784，没有走不到的行）。
  - 交了、等结局的：这一批新写与重写的 22 行变异在主工作区真表上入库（任务 15053）；缩到展开深度 5 的崩溃点重放两项（任务 15055：近满池一项，快照下截另一个 inode 一项）。缩过域的不算门禁通过。
- 代价（实现员推的，没量）：一次调用从「答案前面占着的候选对数 × 开过的聚簇段数」变成「对数级取集合首项 + 至多开过的聚簇段数加 2 次点查 + 32」。调查那一趟量到旧实现 `scan_candidates=2404950934`、`scan_candidates_times_segments=136618056428`（任务 14125，调查报告里的数）。
- 压着的条款：`.claude/kb/decisions/03-空间分配.md` D3（空间分配） 已定项 8（第 1 条用户数据落点的政策函数、第 2 条回落、末段）、已定项 10（聚簇段 64 槽、用户数据不进开放段）；D26（后台整理与放置回收） 已定项 1 第 3 条（保护期内的 `R`）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 第一轮的修补本身：落点真的一个不变吗。新旧答案相同靠三样对齐（实现员第一份报告第二节末段）：一对不跨段；聚簇段起点只来自 `OpenEmptySegment`，是单元区起点加 64 的整数倍；`R` 的起点是 64 的倍数。腿判：这三样在快照这棵树上各由什么保证（类型、构造函数、断言），有没有一条路径能把不对齐的起点放进 `cluster_segments` 或 `R`（挂载时从盘上重建、整理意图续做、测试以外的构造入口）；单元区槽数不是 64 的整数倍时最后那个不满的段在集合里怎么算、旧实现在那一段的尾巴上答什么；`R` 只罩一段还是可以罩几段，新实现按「段起点在 `R` 里」判与旧实现逐候选判在 `R` 跨段、`R` 不满一段时等不等。
2. 产生结论的方法：对拍与增量维护的那几条用例撑不撑得住「落点不变」。三条对拍用例的历史生成器走到的操作列在实现员第一份报告第一节第 1 条；腿判它走不到的形态有哪些（单元区不满整段的尾巴、三块以上的盘、`R` 与聚簇段相邻或重合、挂载重建之后接着走、扣住与隔离叠在同一槽上先后放开），哪一种上新旧实现可能分叉；那条头一趟没红的变异（挡上一侧不摘段）被判成取样点不敏感，判得对不对——集合里多留一个已经没有槽对的段，产品路径上会交出什么（`lowest_unblocked_slot_pair_in_the_segment` 在那一段里找不到对时走哪一支）。
3. 从未被看过的地方：集合的每一条维护路径。三张挡位图只经 `set_blocking_bit` 改（实现员给了 grep）；腿判建空闲图（第 660 行）、克隆、挂载时从盘上重建、回退、整理搬走之后，集合与每段槽对数是不是处处同源；`PoolAllocator` 的克隆（固定点演练）拷不拷这个集合、演练与真发读的是不是同一份；另一个会话那处 `SlotPairCount` 替换与集合维护里 `.0 -= 1`、`.0 += 1` 的交叉有没有改到翻过 0 的判法。
4. 新拒绝成员的分法：`refusal_of_commit_generated_answers_that_differ` 只在每块盘都答回落时换成新成员。腿判：换成员之后，上层按成员分流的每一处（发布层装进 `PublishError::PlacementRefused`、会话层抬 F 那一判、`crates/singlefs-verification/src/model_comparison.rs` 映射成 `Unexplained`）行为与换之前逐处相同吗；模型对拍把它映射成「说不清」会不会把一条真分歧吞掉。

## 四、检查单

### 检查单：走不到的理由二次查实

三份实现员报告「走不到的理由」原文都是无。第一份：「这一轮在产品代码 `crates/<包>/src/` 里没有新加 `todo!`、`unreachable!`、`assert!`，也没有新加名字以 Undecided、Unsupported 或 OutsideTheFirstVersion 收尾的错误成员。」腿在 diff 里现核；`lowest_unblocked_slot_pair_in_the_segment` 找不到对时的那一支走不走得到、走到了交什么，算在这一格里判。

### 检查单：类型与名字

新名字见第二节。diff 里新加的 `expect`（`grep -n 'expect(' research/prompts/_d3-fallback-richest-leaf-code-r2-diff.md` 里以 `+` 起头的行）有几处消息只写了「回落」「64」「个数」「下标」「段号」「盘号」这类一两个词，主 agent 没逐处分它们落在产品代码还是 `#[cfg(test)]` 模块里；腿逐处分开，产品代码里消息没写靠哪条不变量的算打中。段号用裸 `usize`（`BTreeSet<usize>`）、叶序号用 `AllocationRecordTreeLeafIndex`：腿判段号该不该也立类型，与别的裸 `usize` 下标会不会混。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支：旧名整个换掉，没有留别名；逐候选的旧实现只在 `#[cfg(test)]` 下。`grep -nE '^\+.*(legacy|old_format|migrat)' research/prompts/_d3-fallback-richest-leaf-code-r2-diff.md` 主 agent 跑过，零命中；腿另判 `#[cfg(test)]` 的旧实现算不算为旧代码留的兼容。

### 检查单：不新建判定库

本轮改动没有这一类：diff 只在 `crates/singlefs-core/src/`，新测试是分配器的内联单测，不存判定。

### 检查单：所有任务多机可用

本轮改动的产品代码没有这一类；新测试在进程里造空闲图与定种子的伪随机历史，三份报告「拒算的测试」都写没有。副本上的任务有派到第二台跑过的（clippy 那一条任务 14857 在第二台）。

### 检查单：测试优先多线程

新加的对拍三条、增量维护一条、拆开的三条回落单测、新拒绝成员一条，各自建状态、由测试框架并行跑。腿核对拍用例是不是一条一个场景，历史生成器里的循环算不算该拆的参数循环。

### 检查单：带 --item 的命令与项

本轮没有新项。实现员第一份报告写「这批目标都没登记崩溃枚举」，没列带 `--item` 的崩溃点重放命令；主 agent 另点了两项缩域重放（任务 15055）。变异行点名 22 行（任务 15053）。腿判：用户数据落点换实现之后，哪些登记的崩溃枚举用例最该点（数据单元落点随占用变化最多的那几条流），实现员一条没列算不算缺项。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没加崩溃流。腿判新算法有没有带出已有流罩不到的崩溃状态（集合是内存里的增量量，不落盘；挂载重建之后的第一次取落点是唯一与盘上状态相接的地方）。

## 五、格

1. 第三节四条攻击面。
2. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
3. 辩方腿另答：第一轮判决里没采的攻击 1c、只记欠账没改的固定点终止，在这一批修补之后是不是仍然够不着；这一批修补有没有让它们变得够得着。
