# d3-fallback-richest-leaf-code-r1 判决：提交内生块回落改成先取重写集的叶、不够取空槽最多的叶，代码轮第一轮（2026-11-12）

<!-- doc-lint:not-numbers R T1 T2 T3 T4 T5 T6 -->

## 一、腿与核查

腿：正推腿 `research/prompts/d3-fallback-richest-leaf-code-r1-sonnet-output.md`，攻方腿 `research/prompts/d3-fallback-richest-leaf-code-r1-opus-output.md`（模型 `research/prompts/d3-fallback-richest-leaf-code-r1-opus-model/`）。两条腿都没跑项目代码，读数由主 agent 经任务服务跑出。

开工快照：`sha256sum -c research/prompts/d3-fallback-richest-leaf-code-r1-snapshot/crates-sha256.txt research/prompts/d3-fallback-richest-leaf-code-r1-snapshot/kb-sha256.txt` 写判决时现跑，全部 OK（对不上 0 份）。快照树倒推那一步（`task-server/task submit --cwd ~/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-verdict -- python3 scripts/agents/snapshot-tree.py d3-fallback-richest-leaf-code-r1 --out … --repository /home/fy5090/code/singlefs`，任务 14050）排了两个多小时没起（两台负载超过服务的线），取消了；快照里的每份文件此刻与快照逐字节相同，所以引文核对直接对主工作区做：`python3 scripts/agents/cite-check.py research/prompts/d3-fallback-richest-leaf-code-r1-sonnet-output.md research/prompts/d3-fallback-richest-leaf-code-r1-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_d3-fallback-richest-leaf-code-r1-background.md`，末行「核了 77 处引文、125 处裸行号，1 处对不上，125 处没判（其中裸行号没有引文可比 114 处）」。对不上的一处是正推腿报告第 184 行引 `crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:56`，原文在第 57 行，行号差一、引文无误。快照之外的文件（测试、模型、装置）没有快照可倒推，腿引到它们的行按主工作区此刻的内容核；没判的裸行号里判决用到的，主 agent 逐处现读（下面各条写了行号的都是）。

主 agent 跑出的读数：

| 任务 | 跑的什么 | 结局 |
|---|---|---|
| 13962 | `a_floor_raise_refused_for_space_counts_as_short_of_space`（正推腿 T1） | `test result: ok. 3 passed` |
| 13963 | `admission_checkpoint_cost_per_device_paths`（T2，含两条腿都点名的 `empty_publishes_of_the_histories_where_one_device_rewrites_more_than_two_leaves_fit_in_the_whole_tree_checkpoint_cost`） | `test result: ok. 4 passed` |
| 13964 | 近满池录制流的状态计数一条（T3） | `test result: ok. 1 passed; … 4 filtered out` |
| 13965 | `admission_raises_the_floor_for_the_deletion_of_another_inode`（T4） | `test result: ok. 1 passed` |
| 13966 | `random_histories` 点名的一条（T5） | `test result: ok. 1 passed; … 43 filtered out` |
| 13967 | `accept_raises_the_floor_for_space`（T6） | `test result: ok. 10 passed` |
| 13969 | `the_flow_with_only_the_region_free_drops_the_intent_with_its_placements_in_the_region` | `test result: ok. 1 passed; … 4 filtered out` |
| 13968 | 攻方模型复跑（三条臂、五个形状） | `name=exit_codes as_coded=0 prefer_then_lowest=0 lowest_only=0`、`name=probe_lines count=15 expected=15` |
| 13976、14025 | 崩溃点重放 `compaction-paused-short-of-space-dropping-the-intent`（缩到展开深度 5） | 都没判成：前一趟别的包编不过；后一趟判段报「观察者 landed_version_read_through_the_mapping … 的录入段那一块不在库里」，原因是这条用例的观察者没在 `crates/singlefs-verification/src/judging_units_entries.rs` 的 `OBSERVER_ENTRIES` 登记，补登的补丁在 `~/.local/share/singlefs/work/observer-entries-backfill-writer/patch/` |

攻方模型没按规则第 3 步拷出去在副本里复跑：它的包装脚本自己把冻结副本拷进新的运行目录再编，主 agent 交的就是那一条，读的是任务日志，没比腿报告里的读数（腿没跑过，报告里只有预测）。

核不动的：`compaction-paused-short-of-space-dropping-the-intent` 那一项的崩溃状态在新政策下红不红，等观察者补登打进主工作区之后重交；384 槽小盘那几项的全量重放归提交时的门禁。

## 二、判决

1. **代码做的是用户定的那句话：站住。** 两条腿各自逐句对到主工作区的行，结论一致：重写集取每块盘都重写的叶的交集（`RewriteSetLeaves::present_on_every_device`）；空槽最多按每块盘都空的槽数比；两步都排除活 R；只有每块盘都答回落才进池级政策，一块开段一块回落照旧在动状态之前拒；用户数据落点没动。攻方 1a、1b 没打中。
2. **各盘自己答的落点不同而池里有共同空槽时现在分配成功（改前拒）：不算越位。** 池级政策给每块盘换上同一个槽再判一致，交出的落点各盘相同，D3（空间分配） 已定项 8 末段「各盘落点答案必须相同」在结果上成立；变的是「各盘各自算出的答案不同就拒」这一层。kb 字面要跟着写清，进写回。这一形在产品路径上没有专门量过（单测 `fallback_on_two_devices_counts_the_slots_free_on_every_device_and_records_one_slot_on_both` 钉着）。
3. **固定点终止：没被打穿，也没被证明。** 攻方构造不出「重写集没变、这一版的节点来回换」的输入；模型复跑里对照形 `sparse-leaves-below-a-rich-region` 现行实现 `rounds=4 … outcome=settled-then-refused-by-the-empty-segments-floor`，另两条臂 `outcome=did-not-settle-after-64-rounds`。上界 64 留着，终止论证记欠账。
4. **攻方 1c（空槽最多的叶拆掉全池最后一对相邻空槽）：装置走不到，不采，也不算没打中。** 攻方造的四个形状，三条臂十二行全是 `outcome=other:SpaceAdmissionRefused(…)`，在空间准入就被拒、没进固定点，预测表里「现行实现 placement-refused、另两臂定下」一格都没出现。攻方自己写了这一形不分辨新旧政策（病根在准入只数槽不数对，`crates/singlefs-core/src/admission.rs:1054`）；按「打中不分辨臂」不拿它判这份补丁，另记一笔欠账。
5. **每片叶没挡槽数的增量维护：站住。** 两条腿各自全文件搜，三张挡位图的写只在 `set_blocking_bit` 里。
6. **演练与真发取同一串落点：站住**（正推腿逐个核了六个调用点）。`crates/singlefs-core/src/recovery.rs` 的预演走空重写集：攻方判产品路径上走不到分歧（合取 7 先拒），正推腿留了「单元区小于两个整段」一角没核，记欠账。
7. **正推腿推测会红的钉值（T1 到 T6）：都没红**，上表六条任务全过。补丁带出的钉值到现在只有背景材料第二节那一处（14 改 12）。
8. **检查单「类型与名字」：打中。** 现读 `crates/singlefs-core/src/allocator.rs` 第 2880 到 2881 行：池级取到的落点装进 `CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(fallback_slot_on_every_device)`，成员名与计数 `commit_generated_placements_that_fell_back_to_the_lowest_free_slot` 对它已不是实话；叶序号、槽数、槽号共用裸 `u64`（`leaf_indices: BTreeSet<u64>`、`unblocked_slots_in_allocation_record_tree_leaf(&self, leaf_index: u64) -> u64`、`candidate: u64`）；新加的 `expect` 消息没写靠哪条不变量；池级取不到时报的 `CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined` 说的是段对齐，实情是没有每块盘都空的槽。
9. **检查单「测试优先多线程」：打中。** `crates/singlefs-core/src/allocator.rs:4745` 起那条单测按参数循环、每轮重建池，后半段又换重写集再建一次，是三个场景，照 `.claude/rules/test-script-discipline.md`「用例原子化」该拆成三条。
10. **检查单「带 --item 的命令与项」「新崩溃流多罩了什么」：两条腿都回推不出，算打中。** 实现员按源码字面挑的 8 项漏了 `compaction-paused-short-of-space-dropping-the-intent`；它的形状用例过了（13969），崩溃状态那一半等观察者补登。靠关键字挑不出哪条流真走过回落，记欠账。
11. **已成假话的注释与上界文档：打中。** `crates/singlefs-verification/src/model.rs:4210`「回落取最低的空槽」、`crates/singlefs-core/src/admission.rs:981` 与 `:1188` 到 `:1189`（含「量到一块盘改 3–4 片叶」这个改前量的数）。`model.rs` 那个上界在「靠隔离或扣住撑起的非空段」一角两条腿都推不出，记欠账。
12. 检查单其余四格（走不到的理由、不兼容旧代码、不新建判定库、多机可用）两条腿都回成立。

第 8、9、11 条不改行为，第 1 到 7 条里没有要改行为的；改完开第二轮，只攻这一轮的修补与同一批并进来的改动。

## 三、改法

并进下一批的实现员一次改（`crates/singlefs-core/src/allocator.rs` 等别的会话那一批类型替换先打进主工作区，再以那棵树为基线派）：

1. 改名：`FallBackToLowestFreeSlot` 与回落计数换成说「回落」而不说「最低空槽」的名字，连同引它们的测试、`crates/singlefs-verification/src/model_comparison.rs` 与变异行原文；池级取不到落点时换一个说实情的错误成员。
2. 叶序号、每片叶的槽数各立 newtype；回落那一片里的槽号用已有的 `SlotNumber`。新加的 `expect` 消息写清靠哪条不变量。
3. `allocator.rs:4745` 那条单测拆成三条。
4. 第 11 条的三处注释改成现状；`model.rs` 上界的文档改成新政策下推得出的那一句，推不出的那一角写明并指到欠账。
5. 同一批并进用户 2026-11-12 定的「改实现 优化算法」：`DeviceFreeMap::lowest_user_data_slot_outside_the_region` 换算法、落点不变（调查报告 `~/.local/share/singlefs/work/e207-slow-item-investigator/result.md`）。

写回 kb（写回员，随这份判决一批）：D3（空间分配） 已定项 8 第 2 条改成新政策的字面，末段补一句池级政策下「各盘答案相同」指交出的落点；变更史一条；`research/prompts/d3-fallback-placement-r1-main-verification.md` 的结论由 `research/prompts/d3-fallback-placement-r1-correction.md` 之后的重量与这份判决接上，不回改那两份。欠账新登：固定点终止在新政策下没有证明；准入只数槽不数对（两槽单元的最后一对被单槽单元拆掉）；每次回落按叶走一遍的代价随单元区线性；`model.rs` 重写节点上界在隔离撑起的非空段一角；预演走空重写集在单元区小于两个整段的几何上没核；挑「真走过回落」的崩溃枚举用例没有机器的办法。

## 交用户的

| 事项 | 状态 | 说明 |
|---|---|---|
| 各盘自己答的回落落点不同、池里有共同空槽时分配成功 | 主 agent 判不算越位，写回时把已定项 8 末段的字面写清 | 用户定的那句话没管到这一格；不同意就在写回前说 |
| 攻方 1c：两槽单元的最后一对被拆 | 零轮、装置走不到 | 不分辨新旧政策，记欠账，不为它再开一轮 |
| 实现员自定的四处取法（一样多取叶序号小的；两槽单元让给下一片叶；按单槽数排「最多」；搬迁目的地也带重写集） | 随代码判过，没被打中 | 写回时逐条写进已定项 8 第 2 条 |

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs：回落那一片（`RewriteSetLeaves`、每片叶的没挡槽数与 `set_blocking_bit` 里的增量维护、`is_whole_span_open_for_commit_generated`、`try_allocate_commit_generated` 与 `try_allocate_commit_generated_preferring_the_rewrite_set`、`slots_open_on_every_device_in_the_leaf`）与它的五条单测；第二节第 1、2、5、8、9 条
- crates/singlefs-core/src/transaction.rs：`rewrite_set_leaves_of_the_roles`、`allocate_placement_for_role` 与 `allocate_relocation_destination` 的六个调用点、`settle_the_allocation_record_tree` 的终止文档；第二节第 3、6 条

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 8 | 推翻 | 2026-11-12 改了：第 2 条的回落落点由「各盘槽号最小的空槽」换成「先取重写集的叶、不够取空槽最多的叶」（用户 2026-11-11 定），末段补池级政策下各盘答案相同的含义；写回随这份判决同一次改动 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-11-12 不受影响：分配准入合取七条一条没动，回落落点仍整跨度过合取 |
| D26（后台整理与放置回收） 已定项 1 | 不影响 | 2026-11-12 不受影响：第 3 条保护期内的 R 照旧被两步回落排除 |
