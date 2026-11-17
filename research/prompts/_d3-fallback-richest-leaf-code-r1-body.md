# d3-fallback-richest-leaf-code-r1：提交内生块回落改成先取重写集的叶、不够取空槽最多的叶，代码轮第一轮正文（2026-11-12）

<!-- doc-lint:not-numbers R L1 L9 -->

## 一、题面

D3（空间分配） 已定项 8 第 2 条管提交内生块在聚簇段耗尽时的回落落点。用户 2026-11-11 弹窗定案，选项原文「改成先取重写集、不够取空槽最多的叶」：每块盘都答回落时，池里先取这次发布分配记录树重写集里每块盘都重写的叶中、每块盘都空的最低落点；没有再取每块盘都空的槽最多的那片叶里的最低落点；两步都排除活整理意图的 R；固定点的轮数上限 64 不变。kb 里已定项 8 第 2 条的字面还是改前的（各盘槽号最小的空槽），写回随这一轮判决一批做；这一轮判的是代码做的是不是用户定的那句话。

这是零轮改法（判决 `research/prompts/d3-fallback-placement-r1-main-verification.md` 与更正 `research/prompts/d3-fallback-placement-r1-correction.md` 之后由重量得出），A/B 读数在任务 13216、13264、13265、13266（今天臂与新臂喂同一批输入），判读报告 `~/.local/share/singlefs/work/d3-fallback-remeasure-4/result.md`。

改动全文：`research/prompts/_d3-fallback-richest-leaf-code-r1-diff.md`（产品代码两份）。实现员报告：`~/.local/share/singlefs/work/d3-fallback-richest-leaf-writer/result.md`。测试改动在补丁 `~/.local/share/singlefs/work/d3-fallback-richest-leaf-writer/patch/crates.patch` 第 941 行起。

## 二、实现今天的样子（主 agent 的观测，2026-11-12）

- 被判的产品代码：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`（开工快照 `research/prompts/d3-fallback-richest-leaf-code-r1-snapshot/`）。主工作区的 `allocator.rs` 在这份补丁之后又被整理预检那一题的补丁改过，那些块归它自己的代码轮；腿只攻这一轮 diff 里的块与它们和周围代码的交叉。
- 新入口 `PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`；`try_allocate_commit_generated` 成了交空重写集的薄包装；`DeviceFreeMap` 多一个每片分配记录树叶的没挡槽数，在 `set_blocking_bit` 里增量维护；`transaction.rs` 新函数 `rewrite_set_leaves_of_the_roles`，`allocate_placement_for_role` 与 `allocate_relocation_destination` 各多一个参数，调用点 6 处。
- 非测试路径上取回落落点的每一处，实现员列在报告「验收第 1 条」一节；`crates/singlefs-core/src/recovery.rs` 新池新建文件的预演走空重写集。
- 验证到的数（量的）：副本上核心库单测任务 13432「49 passed」；回落目标任务 13428「8 passed」；锻造盘收敛用例任务 13429 退 0；盘不等大目标任务 13430 退 0；clippy 任务 13502 退 0（主 agent 手改两处变量遮蔽与一处取余写法之后）；build 任务 13434 退 0；证红任务 13482 抓到 5 条、13483 抓到 2 条；拒算登记任务 13431「拒算 12 条、登记表 12 行」。打进主工作区任务 13525。主工作区上十一行变异分两片各抓到 5 条与 6 条（任务 13806 的两片），合并核账那一步因跑的当中树被别的补丁改过、键对不上而退 1，整表留给提交时的门禁。崩溃点重放缩到展开深度 5：近满池一项 89 态 red=0（任务 13626）；另七项三项绿、一项本次未跑、三项在合并那一步没判成（任务 13868），都不算门禁通过。
- 补丁带出一处实现员没点到的钉值：`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/admission_raises_the_floor_before_refusing.rs` 里卸载那一串的单元数 14 变 12，A/B 任务 13860（现行政策，读到 12）与 13861（还原成槽号最小，读到 14）坐实，改后整个目标任务 13862「26 passed」。
- 压着的条款：`.claude/kb/decisions/03-空间分配.md` D3（空间分配） 已定项 8（第 2 条与末段「第一版盘不等大时，各盘落点答案必须相同；小盘没有全空段而大盘有时，在动任何状态之前拒绝」）、已定项 7（分配准入合取）；D26（后台整理与放置回收） 已定项 1 第 3 条（保护期内的 R）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 过度外推与越位：代码做的是不是用户定的那一句。逐句对：重写集取的是不是每块盘都重写的叶的交集；「空槽最多」按每块盘都空的槽数比；两步都排除活 R；只有每块盘都答回落时才进池级政策，一块开段一块回落照旧在动任何状态之前拒（已定项 8 末段）；用户数据落点一行没动。各盘自己答的回落落点不同而池里有共同空槽时，现在分配成功、改前拒：这一形越没越出用户定的那句话。
2. 内部矛盾与循环依赖：固定点的终止。`settle_the_allocation_record_tree` 的旧论证靠落点按同一串槽位往后取，新政策下先取的叶跟着这一轮的计划变；实现员报告「固定点终止」一节写明有一类轮（重写集没变、这一版的节点来回换）没有上界论证，只有量到的轮数（锻造盘 41 探最多 5 轮、老化盘 212 格最多 6 轮）。腿判：构造得出走满 64 轮或来回换的输入吗；演练与真发读的是不是同一串重写角色、取不取同一串落点；`recovery.rs` 的预演走空重写集、真发走带重写集的，两边判的结果会不会不同。
3. 哪一条最脆弱：每片叶的没挡槽数是增量维护的。三种挡位（已分配、隔离、扣住）的每一条置位与清位路径是不是都经 `set_blocking_bit`；有没有绕开它直接改位的路径（挂载时从盘上建空闲图、回收、整理搬走、抬 F、回退）；上界与逐槽数不一致时取到的叶对不对。
4. 钉着落点与单元数的测试。这份补丁改了回落落点，实现员只跑了自己点到的目标，主 agent 事后撞上一处 14 变 12。腿判：仓里还有哪些测试、装置、模型的上界或注释钉着「回落取槽号最小的空槽」推出的落点、单元数、轮数（实现员报告「交主 agent 的问题」第 1、3、4、6 条点了名字、`admission.rs` 两处注释、`model.rs` 的上界、没有行盯的那一句），哪几处是会红而还没跑到的。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」原文：「这一轮在产品代码 `crates/<包>/src/` 里新加的 `todo!`、`unreachable!`、`assert!` 与以 `Undecided`、`Unsupported`、`OutsideTheFirstVersion` 收尾的错误成员：无。」新加的 `expect` 两处都是整型换算（`allocation_record_tree_leaf_position_of_index`、`slots_open_on_every_device_in_the_leaf`）。腿核 diff 里新加的分支有没有走不到的，两处 `expect` 依赖的范围成不成立。

### 检查单：类型与名字

新类型 `RewriteSetLeaves`（`none`、`present_on_every_device`、`leaf_indices`）；新函数 `try_allocate_commit_generated_preferring_the_rewrite_set`、`rewrite_set_leaves_of_the_roles`、`is_whole_span_open_for_commit_generated`、`slots_open_on_every_device_in_the_leaf`、`unblocked_slots_in_allocation_record_tree_leaf`。没改的旧名 `CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot` 与计数 `commit_generated_placements_that_fell_back_to_the_lowest_free_slot` 对池里真取的落点已不是实话（实现员报告「交主 agent 的问题」第 1 条）；腿判这算不算打中、叶序号与槽数有没有共用裸整数。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支：旧政策整条换掉，没有臂开关；`try_allocate_commit_generated` 留着当空重写集的包装，腿判它是调用方要的入口还是为旧调用方留的兼容。`grep -nE 'legacy|old_format|migrat' research/prompts/_d3-fallback-richest-leaf-code-r1-diff.md` 由腿现跑。

### 检查单：不新建判定库

本轮改动没有这一类：diff 只在 `crates/singlefs-core/src/`，产品代码不写判定；新测试不存判定。

### 检查单：所有任务多机可用

本轮改动的产品代码没有这一类；新测试在进程里造分配器、在内存稀疏盘上建池，实现员报告「拒算的测试」写无，拒算登记任务 13431 没有未登记。副本上的测试任务有派到第二台跑绿的（任务 13428 的日志）。

### 检查单：测试优先多线程

新加 5 条分配器单测与 1 个验证包测试目标（`allocation_record_tree_settling_after_a_fallback_on_a_disk_with_two_free_slots_per_leaf`，一条用例一个场景），各自建状态、由测试框架并行跑；没有切片循环。腿核有没有该拆而没拆的参数循环。

### 检查单：带 --item 的命令与项

本轮没有新项。实现员报告「受影响的崩溃枚举用例与崩溃点重放项」列 8 项带 `--item` 的命令，主 agent 交过（任务 13626、13868，缩过域）；crates 变异表点名 11 行（任务 13806）。腿核那 8 项列得全不全：回落会走到的崩溃枚举用例还有没有没点到的。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没加崩溃流，只加一条固定点收敛的小用例。腿判回落落点换了之后，崩溃态上分配记录树的形状有没有已有流罩不到的新形（回落进空槽最多的叶之后那片叶头一次被重写的那一版）。

## 五、格

1. 第三节四条攻击面。
2. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
