# d3-fallback-placement-r1 正推腿报告（2026-11-09）

批次：`records/2026-10-13-里程碑四预开发.md#收口修补二批`。立场：正推（从已定条款与 `crates/` 实现推「应该是什么」，再与条款逐格比）。
用户已定「回落时优先取已在重写集里的叶」，这一轮不再问要不要改，只答怎么改。本报告里的数每个标「量的」或「推的」；我没有跑任何命令（编译、测试、模型都交主 agent，命令在第七节），所以凡是模型读数一律「待主 agent 跑」。
下文三条回落臂的名字：`lowest_free_slot`（今天的政策）、`rewrite_set_first`（用户定的政策，本报告给的定义）、`rewrite_set_then_densest_leaf`（在上一条上多一步「重写集的叶里没有空槽时去哪」的候选，用户没要求，作为问题 1 第二问的备选交出）。
「重写集」指 `crates/singlefs-core/src/transaction.rs` 里分配记录树这一次发布要重写的节点集合；条款里的 `R` 指活整理意图的区域，两者无关，下文 `R` 只指后者。

## 模型目录与复跑

模型目录 `research/prompts/d3-fallback-placement-r1-sonnet-model/`（补丁、两个测试目标、复跑脚本）。复跑命令见第七节命令 2，整条是：
`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/d3-fallback-placement-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录>`（范围声明与冻结副本的建法写在命令 1、2）。每个文件的 sha256（量的：`sha256sum`）：

| 文件 | sha256 |
|---|---|
| `research/prompts/d3-fallback-placement-r1-sonnet-model/patch/b0-rewrite-set-leaves.patch` | `822eb140c4a6512fff86829e65a23a1d287e3851ca65454bcc834f789e8bce20` |
| `research/prompts/d3-fallback-placement-r1-sonnet-model/tests/d3_fallback_placement_forged_sweep.rs` | `2b490a70c24cc24300aeee33144ecde26d856ed7ff70b8b1f1b19aa808f97aa6` |
| `research/prompts/d3-fallback-placement-r1-sonnet-model/tests/d3_fallback_placement_leaf_chain.rs` | `12678bcc821dc2256c1b9645c59601828f80ee21c06d56eaf7a31d2306223b71` |
| `research/prompts/d3-fallback-placement-r1-sonnet-model/tools/rerun.sh` | `b99637b7cba27445ddd23dab3cd275197b70970d9deaacc647093f10c8b2a4aa` |

`tools/rerun.sh` 是 `scripts/agents/three-way-rerun-template.sh` 拷来、只换了 `rerun_steps` 那一段；`--selftest` 没有跑（换了步骤之后模板自带的自证格不再适用，攻方腿的 `rerun.sh` 同样）。

## 〇、结论

1. 定义（`rewrite_set_first`）：按叶，不按段；叶 = 分配记录树的叶（812 槽，按绝对槽号 k × 812 对齐），重写集的叶 = 这次发布重写的角色里层级 0 的节点，取「每块盘上都在重写集里」的那几片叶的交集；回落时先取这些叶里槽号最小、整跨度在每块盘上都不被挡、不在活 `R` 里的空槽；这些叶里一个都没有，才取今天的答案（该设备内槽号最小的空槽）。各盘先照今天的规则各答一个、全答回落且一致，才看这条偏好；不一致、有的盘答不出，照旧整笔拒。
2. 它与条款字面冲突一处：D3（空间分配） 已定项 8 第 2 条的回落句与已定项 8 摘要行的「段耗尽回落同一规则」要改写；已定项 5、已定项 10、D26（后台整理与放置回收） 已定项 3 的底额、已定项 1 第 3 条的 `R` 排他都不冲突；checker 的不变量里没有一条判回落落点，没有要动的（推的，grep 过，第三节）。
3. 它修不了那 5 次探：攻方模型 185 次探里撞 64 的 5 次，重写集的叶里一个空槽都没有，`rewrite_set_first` 在那几个形态上取的槽与 `lowest_free_slot` 逐个相同（推的，推导在第四节；读数待主 agent 跑模型坐实）。要修那 5 次，得另加「重写集的叶里没有时去空槽最多的叶」（`rewrite_set_then_densest_leaf`），那要维护每片叶的空槽数索引，产品代码里没有，没有被攻过。
4. `rewrite_set_first` 改的是「重写集的叶之内」的落点，作用在「现行叶有空槽、而全池最低空槽在重写集之外」的形态上（今天的政策会多弄脏几片叶、每片叶多 2 个角色）；这类形态在攻方自然历史那一段没有走到（量的：三段历史每次探轮数只有 2 或 3），所以这一条政策今天买到的是「少弄脏叶」的下界保证，不是已经量到的收益。
5. 补丁（可打在今天的工作树上，我没编译）在 `research/prompts/d3-fallback-placement-r1-sonnet-model/patch/b0-rewrite-set-leaves.patch`；模型与复跑脚本在同一个目录。

## 一、实现今天的样子（读过的路径与看到的事实；除标「量的」的都是读源码，我的观测）

- 提交内生块的落点入口：`crates/singlefs-core/src/allocator.rs:2574`「pub fn try_allocate_commit_generated(」。开放段 bump 装不下时，每块盘按自己的空闲图各答一个去处，答案一致才分配（`allocator.rs:2608`「DeviceAgreement::AnswersDiffer(answer_per_device) => {」是不一致的拒绝）；全空段有就开新段，没有就回落。
- 回落的取法：`allocator.rs:2626`「CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(slot) => {」这一臂把开放段置空（`allocator.rs:2629`「self.open_segment = None;」）、计数加一，落点就是各盘一致的那个槽。每块盘的那个槽由 `allocator.rs:1037`「pub fn lowest_commit_generated_fallback_slot_outside_the_region(」从单元区起点往上扫：每次步进 `allocator.rs:1052`「candidate += footprint.alignment_in_slots();」，第一个整跨度都不被挡（`allocator.rs:808`「pub fn is_blocked_for_commit_generated」：已分配、影子账隔离、抬 F 扣住三种位）且不在活 `R` 里的槽。全池线性扫、扫到第一个空槽为止。
- 分配记录树的叶：`crates/singlefs-format/src/lib.rs:163`「pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;」；一个槽所在的叶 `crates/singlefs-core/src/allocation_record_tree.rs:194`「index_in_device: slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,」（盘 d 上第 k 片叶罩槽 [k × 812, (k + 1) × 812)，按绝对槽号，不按单元区起点）；内部节点扇出 `crates/singlefs-format/src/lib.rs:169`「pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;」。聚簇段 64 槽按单元区起点（50176 起）对齐，叶按绝对槽号 812 对齐，两个网格不重合：50176 − 61 × 812 = 644（推的：算术），一个段常常跨两片叶。
- 重写集的固定点：`transaction.rs:13825`「fn settle_the_allocation_record_tree(」。每一轮先 `plan_the_tree_after_this_publish`（`allocation_record_tree.rs:438`「if rewritten.contains(node) || !previous_nodes.contains(node) {」：在重写集里的、或上一版没有的节点，这次重写），按计划算出这次重写的全部角色（`resolved.rewritten_roles`），在分配器拷贝上先释放、再取落点（`transaction.rs:13931`「allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;」），按走出来的记录算「变了的节点」，`transaction.rs:13936`「if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {」才定下，否则并进重写集（`transaction.rs:13962`「fn merge_into_the_rewrite_set_walking_the_release_chain(」顺着释放链走到闭包）再走一轮，上限 `transaction.rs:13994`「pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;」。
- 真发与演练取同一串落点：`transaction.rs:13593`「&settled.resolved,」把固定点定下来的那一份 `resolved` 交给 `publish_admitted`，它在 `transaction.rs:15672`「let placement = allocate_placement_for_role(allocator, *identity, txg)?;」按 `resolved.rewritten_roles` 取落点。所以「演练用的重写集」与「真发用的重写集」是同一串角色，补丁从这一串角色算偏好集合，两边就得同一个集合。
- 这些回落发生的场合：合取 7（全空聚簇段数 ≥ N₀）只拦前台写入类发布（`transaction.rs:13328`「EmptyClusterSegmentsConjunctReach::ForegroundWrite,」），删除类、整理批、挂载、回退、抬 F 的空发布免（`transaction.rs:13330`「(SpaceAdmission::JudgedByTheFormula, EmptyClusterSegmentsConjunctReach::Exempt)」）。也就是说「池里没有全空段、提交内生块回落」的发布主要是这几类免判的发布；固定点没定下来对它们是整笔失败。
- 已有用例把回落落点钉在今天的政策上：`crates/singlefs-verification/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs:166`「fn publish_on_pool_without_empty_cluster_segment_falls_back_to_lowest_free_slot_on_every_device() {」，期望的落点 50179、50184…50194，分配记录树的叶是 `allocation_record_tree_node(0, 0, 61)`（`crates/singlefs-verification/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs:197`「(allocation_record_tree_node(0, 0, 61), 50187),」）；叶 61 罩 [49532, 50344)，这些落点全在叶 61 里。用例里重写集 = 叶 61，所以 `rewrite_set_first` 取到同一串槽，这条用例不用改（推的，补丁树上的复跑在第七节命令 2 的「现有用例目标」）。
- 观测（量的，主 agent 的副本装置，`/home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r3-opus/rerun-1/settling-fallback.log`，sha256 d8bb339cef294952d363ee8e801ed0c014388e7c050a62d8c81f073ec60051c9）：`grep -c 'fallback sweep=' → 185`；`grep -c 'outcome=did_not_settle' → 5`；5 次是前缀 14、15（每片叶 2 个空槽、120 片）与前缀 14 的形状扫里 2 个空槽、64 / 120 / 200 片；每片 1 个空槽 8–10 轮，每片 ≥ 3 个空槽 4–5 轮；自然历史每次探 `natural … rounds=` 只有 2（13 次）与 3（46 次）。

## 二、问题 1：「优先已在重写集里的叶」的精确定义

### 2.1 按叶，不按段

- 要避免的代价是「弄脏一片原本不在重写集里的叶」：一个落点落进某片叶，那片叶的记录清单就变了，叶连同祖先都要重写、并且每块盘一个角色（`allocation_record_tree.rs:194`「index_in_device: slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,」）。重写集的成员是叶，不是段。
- 叶按绝对槽号 812 对齐、段按单元区起点 64 对齐，两个网格不重合（第一节：偏差 644 槽）；「在重写集里的一个段」里的槽会跨进别的叶，按段取偏好不能保证不弄脏新叶。按叶取才能保证。
- 比叶更粗的一档（同一第 1 层祖先下的叶，一个第 1 层节点罩 169 片叶 × 812 槽）省不下叶的角色：它只省第 1 层节点，不省叶，不采。

### 2.2 定义

输入：`footprint`（一槽或两槽）、`generation`、`rewrite_set_leaves`（叶序号的集合）。叶序号集合由发布路径算：这次发布 `resolved.rewritten_roles` 里分配记录树层级 0 的角色，每块盘各取一份序号集合，取交集（一片叶只有池里每块盘上都重写才进集合）。

1. 开放段还装得下：bump，不变。
2. 装不下：每块盘照今天的规则各答一个（开全空段，或回落到该设备内槽号最小的空槽）；各盘答案不一致、有的盘答不出：照旧整笔拒（`NoFreeSlotOnAnyDevice` 等原因不变）。
3. 一致的答案是开全空段：照旧开段，偏好不看。
4. 一致的答案是回落：按叶序号升序、叶内按 `footprint` 对齐步进，取第一个满足下面三条的起点槽：整跨度落在池里每块盘的单元区内；整跨度每一槽在每块盘上三种位（已分配、影子账隔离、抬 F 扣住）一个都没置；整跨度不在活整理意图的 `R` 里（D26（后台整理与放置回收） 已定项 1 第 3 条）。找到就取它，同时回落计数加一、另一个「落进重写集的叶」的计数加一；一个都找不到，取第 2 步各盘一致的那个槽。
5. 第 4 步取到槽之后与今天一样：开放段置空、回落计数加一、记一条分配记录。

两槽的单元起在偶数槽，叶宽 812 是偶数，一个两槽的单元不跨叶：「记录的末槽不越过它所在叶的末槽」（`crates/singlefs-core/src/allocation_record_tree.rs:4`，文件头也写了这一点）。

### 2.3 重写集的叶里空槽不够，退到哪里

- `rewrite_set_first` 的答案：退到第 2 步各盘一致的那个槽（今天的答案：该设备内槽号最小的空槽）。理由：不加新结构、不加新的拒绝路径（第 4 步没找到时与今天逐字节相同）、条款原句「槽号最小」仍是兜底。
- 备选 `rewrite_set_then_densest_leaf`：第 4 步没找到时，取重写集之外空槽最多的那片叶（并列取序号小的），在那片叶里取槽号最小的空槽。它回答的是另一个问题：一片新叶进重写集要多 2 个角色（每块盘各一个），今天的政策每进一片叶只换来这片叶里的空槽数；空槽数 F 只有 2 的一串叶永远追不平（第四节推导）。取空槽最多的叶，每进一片叶净收益最大。代价：每块盘要维护「每片叶的未被挡槽数」的增量索引（形态同 `allocator.rs:317`「used_per_segment: Vec<u64>,」那一类、改动点在 `allocator.rs:533`「fn set_blocking_bit(」三种位每变一次维护 `allocator.rs:351`「unblocked_slots: u64,」的地方）；扫全池取最大是 O(单元区槽数)，不增量维护就不能进产品。补丁里没有这一条，只在模型里（测试里选叶、经「重写集 ∪ 选中的叶」传给同一个入口）。没有被攻过。
- 不采：「退到与重写集的叶相邻的叶」没有依据；「重写集的叶里取最高槽」与条款「槽号最小」更远。

### 2.4 多盘时每盘答案要不要相同

- 要相同，且不放宽：`Placement` 两盘同槽（`allocator.rs:1120`「一次发布里分配的一个落点，两盘同槽」），已定项 8 末段「第一版盘不等大时，各盘落点答案必须相同」（`03-空间分配.md:173`「第一版盘不等大时，各盘落点答案必须相同」）原样成立。
- 偏好只在「各盘都答回落且答案一致」之后才看，所以没有新增任何一种拒绝，也没有把今天拒的（小盘没有全空段而大盘有、各盘回落的槽不同）放过去：各盘回落槽不同时即使两盘有公共的重写集内空槽也照旧拒，这一格归已定项 8 待办 ①（各盘聚簇段要不要对齐），条款没写，不借这条政策悄悄定。
- 偏好集合取交集、候选槽要在每块盘上都不被挡：等大盘上两块盘的记录同槽，重写集本来对称，交集等于每块盘各自的集合；盘不等大或某块盘上有影子账隔离、抬 F 扣住的槽时，候选自然落在两块盘都空的槽里，不产生两盘不同槽的落点（推的，由补丁的单元测试 `fallback_in_the_rewrite_set_leaves_skips_slots_blocked_on_any_device_and_gives_way_when_the_leaves_are_full` 钉：一块盘上隔离叶首槽，取到的是下一个槽）。

### 2.5 补丁接口（对 `allocator.rs:2570` 一带的改动）

- `allocator.rs` 新增 `RewriteSetLeaves`（叶序号集合，`none()` 是空集、`present_on_every_device(...)` 取交集）；`try_allocate_commit_generated` 变成调 `try_allocate_commit_generated_preferring_the_rewrite_set(footprint, generation, &RewriteSetLeaves::none())` 的薄壳，其余调用方（`recovery.rs:6577` 的新池新建文件演练、`admission.rs:2342` 与各用例）不动、仍是今天的政策；回落那一臂在取到各盘一致的槽之后调新私有函数 `lowest_slot_in_the_rewrite_set_leaves`，认一个落点的判法抽成 `DeviceFreeMap::is_whole_span_unblocked_for_commit_generated`，旧的 `lowest_commit_generated_fallback_slot_outside_the_region` 与它共用同一处判法；新增读口 `commit_generated_placements_that_fell_back_into_the_rewrite_set_leaves`（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
- `transaction.rs`：`allocate_placement_for_role` 与 `allocate_relocation_destination` 各多一个 `&RewriteSetLeaves` 参数；新私有函数 `rewrite_set_leaves_of_the_roles(rewritten_roles, allocator)`；在 `settle_the_allocation_record_tree`（`transaction.rs:13825`）、行发布的固定点（`transaction.rs:2023` 的演练）、`publish_admitted`（`transaction.rs:15672`）与行发布真发（`transaction.rs:2178`）四处，都从同一串 `rewritten_roles` 算集合。

## 三、问题 2：它碰到的条款、记录树自身落点的环、崩溃状态与 checker

### 3.1 冲突的一格（两句并排，整句抄）

| 条款现句 | 实现按用户定的政策应该是什么 |
|---|---|
| 「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」（`.claude/kb/decisions/03-空间分配.md:168`） | 提交内生块的段耗尽时回落：先取这次发布分配记录树重写集的叶里槽号最小的空槽（每块盘上都在重写集里的叶；整跨度每块盘都不被挡、同样排除 `R`），那些叶里没有，才取该设备内槽号最小的空槽 |
| 「第一版不进聚簇段，取已选设备内起点最小、整跨度过准入合取的空槽；段耗尽回落同一规则」（`.claude/kb/decisions/03-空间分配.md:20`，已定项 8 摘要行） | 摘要行里「段耗尽回落同一规则」不再成立：回落与用户数据落点是两条规则（用户数据落点不变，仍是起点最小的空槽，I-5.5 判它） |

另有同一句的代码侧引用要同步：`allocator.rs` 文件头第 7 行与 `allocator.rs:1024` 一带的文档仍写「该设备内槽号最小的空槽」（补丁里已改文件头那一句；`lowest_commit_generated_fallback_slot_outside_the_region` 本身仍是「各盘各答」的那一半，字面不错）。

### 3.2 逐条比对

| 条款 | 判定 | 依据 |
|---|---|---|
| D3（空间分配） 已定项 8 第 2 条 回落句 | 冲突（上表） | 改写后第 2 条还得带上「重写集」的定义（叶、取交集、排除 `R`） |
| D3（空间分配） 已定项 8 末段 盘不等大 | 一致 | 偏好只在各盘都答回落且一致之后才看（第二节 2.4）；「各盘落点答案必须相同」「小盘没有全空段而大盘有时，在动任何状态之前拒绝」原样（`03-空间分配.md:173`「第一版盘不等大时，各盘落点答案必须相同」） |
| D3（空间分配） 已定项 5 | 一致 | 「提交内生块在固定点分配时找不到空槽（`NoFreeSlotOnAnyDevice`），在任何写之前拒绝这次发布」（`03-空间分配.md:100`）：偏好没找到时取今天的答案，不增加任何一种拒绝；「违反它的实现写出的镜像完全合法，只是每次发布贵几百倍」（`03-空间分配.md:102`）：本条政策同样是分配器政策、不是格式位，能拦它的只有运行时读数，补丁加了一个读口 |
| D3（空间分配） 已定项 10 ③ | 一致 | 「「最低的空槽」只在开放段之内说，不下探到段外」（`03-空间分配.md:212`）：偏好只在回落一臂，bump 与开段不看 |
| D3（空间分配） 已定项 10 ⑤ bump 次序 | 一致 | 取落点的角色次序不变（实例表最前、树 ID 升序、「树内先叶后根、同层按 key 升序」，`03-空间分配.md:212`）；出生序号按角色次序发，不按槽号。补丁之后落点槽号不保证随角色次序递增（今天的回落一般递增，两槽容器取对齐对子时也可能不递增）：我在条款里没有找到哪一句要求落点槽号随次序递增，在 `crates/singlefs-checker/src` 里 `grep -n bump` 只命中一处：「I-5.5 拿哪一段当这一版在这块盘上的开放段」（`crates/singlefs-checker/src/walk.rs:7735`，与回落无关） |
| D26（后台整理与放置回收） 已定项 3 底额 | 一致，一个边角没量 | 「`(可用 ≥ 需求) ∧ (全空聚簇段数 ≥ N₀)`」（`26-后台整理与放置回收.md:104`）；回落只发生在每块盘都没有可开的全空段时，回落不开段、本来就不增加全空聚簇段数；两条臂会不会各自毁掉一个「记录口径全空、但隔离位或扣住位非零」的段（记账算它全空、分配器不能开它），这一格没量 |
| D26 已定项 1 第 3 条 `R` 排他 | 一致 | 「`R` 在意图期内不可作分配目的地」（`26-后台整理与放置回收.md:30`）：新函数逐槽过 `excluded_region`，与旧回落函数共用 `is_whole_span_unblocked_for_commit_generated` 一处判法；补丁里没写「重写集的叶里有 `R`」的专门用例，列入第六节的待补用例 |
| D3（空间分配） 已定项 7 登记表第 5 行 | 一致 | 「不在 D26（后台整理与放置回收） 已定项 1 保护期内的 `R`」（`03-空间分配.md:146`）：回落只认三种位加 `R` 这一条，不认七条合取里别的几条（今天也如此），偏好不改这一点 |
| C146 ① | 一致 | 「整理的下一个目的地落回自己正在清空的段」（`.claude/kb/checks-owed.md:137`）：`R` 排他保留 |
| C146 ③ | 补丁让它好转一格 | 「改写量从 `⌈N/扇出⌉` 变成 `min(N, 叶总数)`」（`.claude/kb/checks-owed.md:137`）：整理搬迁里提交内生块类的目的地（`allocate_relocation_destination`，`transaction.rs:15590`）取偏好；用户数据类的目的地仍走 `try_allocate_user_data`，不受影响，所以整理批的大头（数据单元）这一格没有改善 |
| D26 已定项 6 回落计数 | 一致 | 计数口径不变，另加一个读口（「回落计数、一批整理的搬迁量」是运行时计数器，`26-后台整理与放置回收.md:177`，只供观测） |
| D3（空间分配） 已定项 9、I-5.3 | 一致 | 候选集是今天「未被挡的槽」的子集，找不到再退到今天的答案，可用空槽数不变，没有新的假性 ENOSPC |

### 3.3 分配记录树自身的落点会不会形成新的环

- 环本来就在：分配记录树的节点要给自己记分配记录，「落点先于内容」（`transaction.rs:15669`「落点先于内容：分配记录树要装自己那条」），所以才有固定点。补丁没有加新的依赖方向：落点仍只依赖「这一轮的重写角色 + 分配器状态」，不依赖节点内容；偏好集合是这一轮重写角色的函数，演练与真发读同一串角色（第一节 `transaction.rs:13593` 与 `transaction.rs:15672`）。
- 新节点放进重写集里的叶：这片叶的记录清单多一条，但它本来就要重写（在重写集里），不多弄脏任何叶，也不多要祖先；这正是偏好要的结果。放进节点自己所在的叶（叶 k 的新副本落在叶 k 里）也合法：记录先定、内容后装。
- 失去的是今天「走得完」说明里那条性质：「取落点时分配记录树的节点之后只剩单槽的提交内生块」（`crates/singlefs-core/src/transaction.rs:13811`）写的是「按同一串槽位往后取，多一个角色只把这一串拉长」，即每多一个角色，同一串槽位只是多往后取一个。偏好之后重写集变大会改变前面角色的落点（挪进新进来的叶），这条「同一串槽位」不再成立，那一段文档要重写。新的收敛论证（推的，没证）：重写集只增（`merge_into_the_rewrite_set_walking_the_release_chain` 只 `insert`），一个角色要么落进当前重写集的叶（不多弄脏任何叶），要么落在今天的全池最低空槽（只有这一种会让下一轮的重写集多一片叶）；`nodes_after = holding_records`（`transaction.rs:13951`）是整体换成这一轮装着记录的节点，不是累加，我没找到振荡的反例但没有证明。兜底仍是 64 轮上限与 `AllocationRecordTreeRewriteSetDidNotSettle`（`transaction.rs:13953`），条款没写这一格怎么办的状态不变。

### 3.4 崩溃状态与 checker

- checker 的不变量里没有一条判回落落点（推的：`crates/singlefs-checker/invariants/07-分配器.md` 的 I-5.1 至 I-5.7 里，与落点位置有关的是 I-5.5（从 mkfs 水位建树那一版的数据单元落点 t1，该版开放段没满、不走回落）与 I-5.6（活整理意图的区域里没有签发之后分配的记录，补丁保留 `R` 排他）；I-5.1 / I-5.4 判重叠，与槽号取哪一个无关）。没有要动的不变量。
- 发布内单元写的次序是角色的 bump 次序，补丁不改；崩溃点重放判的是每个崩溃态上的不变量，不看某个节点落在哪个槽。改的是回落形态下一次发布重写多少个角色（重写集更小时角色变少），崩溃点枚举的写数会跟着变；要照提交阶段跑崩溃点重放（第七节命令 3、4 之后的那一句）。
- 「在重写集的叶里放新节点」没有新的崩溃态：叶里多一条记录是这片叶 COW 新副本的内容，新副本整体落盘，与今天一个落点落进新叶时叶重写的情形同形。

## 四、问题 3：修得了哪一格

### 4.1 那 5 次撞 64 的探：`rewrite_set_first` 不起作用（推的）

- 攻方造的空闲图（`research/prompts/m4-close-fixes-code-r3-opus-model/tests/attack_fixes_r3_settling_fallback.rs:82`「if leaf < FIRST_LEAF_OF_THE_FREE_LIST {」，`:34`「const FIRST_LEAF_OF_THE_FREE_LIST: u64 = 12;」）：叶序号小于 12 的槽一个都不留空；第 12 片叶开头留 `prefix` 个空槽；其后每片叶只在叶内偏移 400 起留 `free_slots_per_leaf` 个空槽（`:87`「(400..400 + shape.free_slots_per_leaf).contains(&offset)」）；再往后每个 64 槽段只占段首一槽（`:89`「(slot - start) % 64 != 0」）。
- 探开始时的重写集：单元区起点是环末尾的下一个槽，6 MiB 环 = 1024 + 384 = 1408（推的：`crates/singlefs-format/src/lib.rs:261` 的 `JOURNAL_RING_START_SLOT` 为 1024，环 6 MiB ÷ 16 KiB = 384 槽），在叶 1 里；第一个文件与四次覆盖写的单元都在最前面的几个段里，所以现行树的叶就是叶 1（推的）。叶 1 < 12，造图时一个空槽都没留。
- 第 1 轮：重写集里没有空槽，每个角色取到的槽 = 今天的答案（全池最低空槽，在叶 12 的前缀里）；叶 12 因此进「这一版的节点」，下一轮起是新节点、自动在重写集里。之后每一轮重写集 = 叶 1 + 叶 12 起的一段连续的叶，而全池最低的空槽永远落在这一段里或紧接在它后面：偏好集合里的最低空槽 = 全池最低空槽，两条政策取到的槽逐个相同，固定点逐轮相同。
- 所以：前缀 14、15 与形状扫里每片 2 个空槽的那 5 次，`rewrite_set_first` 仍然撞 64（推的）。**这是对用户这个决定的一条硬事实**：它不修那 5 次。验证：第七节命令 2 的「今天树扫描目标」「补丁树扫描目标」在未打补丁与打了补丁的树上跑同一份扫描，`forged-sweep-today-vs-patched.diff` 里不一致的行数应为 0（`differing_lines=0`）；不为 0 则这一节推错了。
- 为什么会撞：一个单元的两份落两块盘同一槽，取落点每碰到一片新叶，下一轮多重写两片叶（两块盘各一）、多要两个槽（第一节 `allocation_record_tree.rs:438`）；每片叶只给 `free_slots_per_leaf` 个槽。每片 2 个空槽时净收益为 0，链永远追不平，直到走到最后那片稠密段才定下；每片 1 个空槽时欠额翻倍、很快走到稠密段（8–10 轮，量的）；每片 ≥ 3 个空槽欠额每轮缩小（4–5 轮，量的）。这个递推（推的）与量的轮数表一致，与量的门槛也对得上：一次覆盖写取落点的槽数 = 数据单元 2 + 不是分配记录树节点的提交内生块 7（extent 树根 1、inode 叶容器 2、inode 树根 1、记账树根 1、映射树根 1、树表 1）+ 2m + 2g + 1（m 片重写的叶、g 个重写的第 1 层节点），m = 2、g = 1 时是 16；攻方扫稠密前缀，前缀 14、15 撞上界、前缀 16 起 3 轮定下（量的，`settling-fallback.log` 里 `prefix=16` 至 `prefix=19` 的行 `rounds=3`），门槛正是 16。

### 4.2 `rewrite_set_first` 起作用的形态（推的，待模型读数）

起作用要同时有两件事：现行叶（重写集里的叶）里有空槽，而全池最低空槽不在重写集里（在更低的、没有进重写集的叶里）。此时今天的政策先去更低的叶、每进一片叶多 2 个角色，`rewrite_set_first` 先用现行叶里的槽。模型 `d3_fallback_placement_leaf_chain.rs` 第二段 `current_leaves_above_the_chain` 把现行叶放在链叶上面、扫现行叶的空槽数 0 到 32、链叶每片 2 或 3 个空槽，三条臂并排量。

自然历史里这类形态有没有：量的两个读数——攻方自然历史每次探的轮数只有 2、3（第一节），也没走进过「重写集之外的低叶有空槽而现行叶也有空槽」的形态的读数；我没有量过自然历史里现行叶的空槽数，所以「自然形态下买到多少」是空白。

### 4.3 有没有反过来变差的形状

- 单次固定点里新弄脏的叶数：`rewrite_set_first` 每个角色要么落进当前重写集的叶（不多弄脏叶），要么落在今天的全池最低空槽（与今天同）。所以一次取落点里新弄脏的叶数不会比今天多（推的，没有形式证明；模型有一个自检 `claim_rewrite_set_first_equals_lowest_free_slot_without_slack` 与现行叶无空槽那一段对读）。
- 聚集写放大：节点新副本倾向落在上一版同一片叶（旧副本在那里，旧副本的叶本来就在重写集里），重写集会随版本稳定在一小撮叶上；今天的政策每次取全池最低，节点位置随全池空槽漂。这是好方向，没量。
- 局部碎片：一槽的提交内生块被放进重写集的叶里最低空槽，可能拆掉一对对齐的空槽（用户数据要两槽对子，`allocator.rs:1052` 的步进对两槽单元是 2）。今天的政策也拆全池最低处的对子，两者都不是「保对子」的政策；拆在哪里不同。`runs`、可用的空槽对数在两条臂下怎么走，我没量（模型里没有对子），要量得另做（第七节没有命令，列在「没做什么」）。
- 回落分配的扫描代价：新函数每次回落分配从重写集的最小叶起逐槽扫，最坏 O(重写集叶数 × 812 × 盘数)；今天的函数从单元区起点扫到第一个空槽。一次发布里角色数约为叶数的 2 倍，大批删除类发布（重写集的叶可达上千片）下新函数一轮是叶数的平方量级（推的，约 1000 片叶时单个角色 1.6 × 10⁶ 次位判，约 2000 个角色），今天的函数在老化池里同样按「第一个空槽的下标」逐角色扫，量级相当，我没量。补丁里没有加「已扫完的叶不重扫」的游标；要加是下一步，不是这一步。

## 五、问题 4：代价数

「步数」读 `PoolWriter::rounds_of_the_last_allocation_record_tree_settling`（真实现轮数，含第一轮把释放弄脏的叶并进重写集那一轮）；「写放大」读一次发布重写的角色数（`published` 时是 `version.rewritten.len()`，模型里是 `roles`）。下表每格标状态：量的（读过产物）、推的（从代码推）、待跑（命令 2，我没有读数）。

| 候选 | 一句话定义 | 固定点轮数 | 一次发布重写的角色数（写放大） | 每次回落分配的扫描 | 新结构 |
|---|---|---|---|---|---|
| `lowest_free_slot` | 今天：回落取该设备内槽号最小的空槽 | 起点 3 轮（量的：`prefix=16` 起各行 `rounds=3`）；每片叶 2 个空槽的链走到稠密段才定、前缀 14、15 撞 64（量的：185 次探里 5 次） | 重写集 m = 1、g = 1 时 12 个（数据 1 + 提交内生块 11，量的：`…runs_out.rs` 用例期望表 12 项，`:197` 一带）；每多弄脏一片叶 +2，每多一个第 1 层节点 +2（推的） | 从单元区起点扫到第一个空槽 | 无 |
| `rewrite_set_first` | 先取重写集的叶里最小空槽，没有再取今天的答案 | 攻方造的空闲图上与上一行相同（推的，4.1；待命令 2 的 `differing_lines`）；现行叶空槽 ≥ 14 且含对齐空槽对时 3 轮（推的：m = 1、g = 1 时取落点的槽数 9 + 2 + 2 + 1 = 14）；模型 `current_leaves_above_the_chain` 逐形态读数待跑 | 不高于上一行（每个角色要么不弄脏新叶、要么与今天同，推的）；形态不同的行待跑 | 从重写集最小叶起逐槽扫，最坏 O(重写集叶数 × 812 × 盘数)（推的） | 无 |
| `rewrite_set_then_densest_leaf` | 上一行，重写集的叶里没有时取空槽最多的重写集外叶 | 攻方造的图上走到稠密段那一片叶、欠额一次补平（推的：欠额 d、稠密叶几百个空槽）；模型 `forged_like_the_attack_model` 各行逐前缀读数待跑 | 进一片稠密叶 +2（推的） | 若有每叶空槽数索引，取最大 O(叶数)；没有索引则每次回落扫全池，不能进产品 | 每块盘每片叶的未被挡槽数，增量维护 |

岔路里的翻面观测（量到什么会让「要不要再加 `rewrite_set_then_densest_leaf`」从一边翻到另一边）：

- 命令 2 的 `differing_lines=0` 且模型 `forged_like_the_attack_model` 上 `rewrite_set_first` 与 `lowest_free_slot` 逐行相同 ⇒ 4.1 的推导成立，用户定的政策不修那 5 次；要修就得加第三条候选。`differing_lines` 不为 0 ⇒ 4.1 推错，先看是哪几行。
- 模型 `current_leaves_above_the_chain` 里 `rewrite_set_first` 在现行叶空槽 ≥ 14 的行上比 `lowest_free_slot` 少 `new_leaves`、少 `roles` ⇒ 这条政策的收益在模型里是真的；读数相同 ⇒ 我推错了（偏好没起作用，先查 `rewrite_set_leaves_of_the_roles` 取到的集合是不是空集）。
- 模型的 `lowest_free_slot` 在 `forged_like_the_attack_model` 上轮数随前缀怎么走，与攻方真实现的读数（前缀 16 起 3 轮、14、15 撞 64）不对拍 ⇒ 模型不能拿去给另两条臂定数，两边都查（`.claude/rules/implementation-first.md`「规矩」第 4 条）。

## 六、改动清单与要同步的东西

### 6.1 补丁（`research/prompts/d3-fallback-placement-r1-sonnet-model/patch/b0-rewrite-set-leaves.patch`）

- 对象：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`；基于今天工作树里这两份的 sha256：allocator.rs `36fe6af3e544c9bd56126e525393c749c9dd0cdc81fb01297e8623895e3e7499`、transaction.rs `19c50519ed1ede3a96298feb646ef7aac38a810596b9f80797f0cb28ccd016a1`（量的：`sha256sum`）。
- 我做过的核：在草稿目录里把补丁打在这两份的拷贝上，打出来的文件与我改好的文件 `cmp` 一致；`rustfmt --edition 2021 --check` 两份都过（量的：只读语法与格式检查）。没有编译：类型检查是主 agent 交 `cargo` 的事（命令 2 里「单元测试目标」先编这棵树）。
- 带两个单元测试：`fallback_takes_the_lowest_free_slot_in_the_rewrite_set_leaves_first`、`fallback_in_the_rewrite_set_leaves_skips_slots_blocked_on_any_device_and_gives_way_when_the_leaves_are_full`（钉：偏好取重写集的叶里最低空槽、没有重写集取全池最低、只在一块盘上重写的叶不算、一块盘上被隔离的槽跳过、两槽单元取偶数起点、重写集的叶全满时退回今天的答案且不计偏好计数）。

### 6.2 采纳之后还要补的（我没做）

1. 用例：重写集的叶里有活 `R`（`CompactionRegion`）的槽时被跳过——补丁的单元测试没有造 `R`；发布层的一条用例：偏好改变了落点的形态上发布、冷启动读回、池级 checker 判过（形态同 `commit_generated_units_fall_back_when_the_segment_runs_out.rs:166` 那条，但要让重写集的叶不是全池最低槽所在的叶，需要自己造空闲图）。
2. `crates/mutations.tsv` 的新行（弄坏开关）：偏好被忽略（回落臂恒取 `lowest_free_slot`，应被 `fallback_takes_the_lowest_free_slot_in_the_rewrite_set_leaves_first` 抓到）；交集写成并集（`present_on_every_device` 里 `all` 改 `any`，被同一条用例的「只在一块盘上重写的叶不算」抓到）；候选槽只看第一块盘（新函数里 `all` 改 `any`，被第二条单元测试抓到）；新函数不看 `excluded_region`（要上面第 1 条的 `R` 用例）；真发路径传空集（演练与真发取不同槽，要上面第 1 条的发布层用例）。现有 `增补 2：` 的九行里 79、80 行的锚点文本（`!self.is_blocked_for_commit_generated(SlotNumber(slot))` 换行 `&& !excluded_region`）补丁后仍恰好在 `is_whole_span_unblocked_for_commit_generated` 里命中一次（缩进相同，已对过：补丁里该段缩进 16 与 20 个空格），77、78 行的锚点在 `commit_generated_answer_without_open_segment_outside_the_region` 里，补丁不碰；这四行的判别力不变，但要由 `crates-mutation-replay` 复核一次。
3. kb：已定项 8 第 2 条与摘要行改写（3.1 表）、C146 第 ② 条的「已还」说明、D26 已定项 6 的观测量里加「落进重写集的叶」的计数、E206 的取样点里没有回落这一列（本轮背景材料第二节）。
4. 代码文档：`transaction.rs:13811` 起「走得完」那一段按 3.3 重写；`allocator.rs` 里 `lowest_commit_generated_fallback_slot` 的文档指着条款原句，条款改了要跟着改。
5. 命名检查与崩溃点重放：照 `.claude/agent-common.md`「门禁」一节，补丁采纳进主工作区之后的两条命名检查与提交阶段的崩溃点重放归主 agent（命令 3、4）。

## 七、要主 agent 跑的命令

命令 1（直接跑，不经服务；在仓根起）：建冻结副本。
`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs <工作根下的新目录>/d3-fallback-placement-r1-frozen --with-test-inputs`
判结果：退出码 0；再 `sha256sum <冻结副本>/crates/singlefs-core/src/allocator.rs <冻结副本>/crates/singlefs-core/src/transaction.rs`，两行必须是 `36fe6af3e544c9bd56126e525393c749c9dd0cdc81fb01297e8623895e3e7499` 与 `19c50519ed1ede3a96298feb646ef7aac38a810596b9f80797f0cb28ccd016a1`（补丁基于的那两份；不是这两份，`rerun.sh` 的 `base_check` 会退 3，补丁要重做）。

命令 2（经服务；在仓根 `/home/fy5090/code/singlefs` 起）：复跑模型，五个测试目标：「单元测试目标」是补丁树 `singlefs-core` 的 lib 里两个新单元测试；「现有用例目标」是补丁树上现有回落落点用例目标 `commit_generated_units_fall_back_when_the_segment_runs_out`；「叶链模型目标」是补丁树叶链模型目标 `d3_fallback_placement_leaf_chain`；「今天树扫描目标」是未打补丁的树上伪造空闲图扫描目标 `d3_fallback_placement_forged_sweep`；「补丁树扫描目标」是补丁树上同一个扫描目标。
`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=d3-fallback-placement-r1 模型目录 tools/rerun.sh 里的 5 个测试目标; 范围=5 个测试目标; 为什么=量三条回落臂在固定点轮数与写放大上的差，核补丁树上现有回落用例仍绿' task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/d3-fallback-placement-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work /home/fy5090/.local/share/singlefs/work/d3-fallback-placement-r1-sonnet/rerun-1`
判结果：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里要有这几行——`name=base_check match=yes`、`name=patch_apply rc=0 expected=0`、五行 `name=patched_unit_target rc=0 expected=0`、`name=patched_regression_target rc=0 expected=0`、`name=patched_leaf_chain_target rc=0 expected=0`、`name=today_forged_sweep_target rc=0 expected=0`、`name=patched_forged_sweep_target rc=0 expected=0`、`name=d3_fallback_placement_summary target_as_expected=yes`；再看 `forged_sweep_lines today=… patched=… differing_lines=0`（第四节 4.1 的判据）。读数在 `--work` 目录里：`observations-today-forged-sweep.txt`、`observations-patched-forged-sweep.txt`、`observations-patched-leaf-chain.txt`、`forged-sweep-today-vs-patched.diff`。叶链模型自检行 `claim_rewrite_set_first_equals_lowest_free_slot_without_slack prefix=0..20` 应全是 `same=true`（4.3 的自检）。`rerun.sh` 要新的 `--work`，再跑给 `rerun-2`。
注意：「单元测试目标」起编补丁树，我没有编译过补丁，类型错会先在这里出；编不过时把编译错误交回，新派同立场的腿（带「承接：」）改补丁，不在这里改。

命令 3、4（经服务；补丁采纳进主工作区之后在仓根起；命名检查）：
`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=交回前核自己改动里的命名' task-server/task submit -- bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names /home/fy5090/code/singlefs`
`task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/naming-lint.sh /home/fy5090/code/singlefs`
判结果：末行 `TASK_SERVER_RESULT` 的 `exit=0` 为过，`exit=1` 为判红，别的码不算跑完。采纳之后的崩溃点重放按提交阶段照 `.claude/skills/crash-test/commit-stages.md` 跑，不单列。

命令 5（我已跑，直接跑的文本工具）：`python3 scripts/agents/cite-check.py research/prompts/d3-fallback-placement-r1-sonnet-output.md`，末行贴在文末。

## 八、什么现象会推翻这些结论

| 结论 | 推翻它的现象 |
|---|---|
| 按叶不按段 | 重写集里的成员不是叶（`allocation_record_tree.rs` 的 `plan_the_tree_after_this_publish` 不按位置节点算）；或叶宽不是 812、落点会跨叶（那两槽单元起点要不是偶数） |
| `rewrite_set_first` 不修那 5 次 | 命令 2 的 `differing_lines` 不为 0，或补丁树扫描目标的 `did_not_settle` 行数少于今天树扫描目标的 |
| 它对条款只有一处字面冲突 | 在已定项 8 以外找到另一句写「回落取槽号最小」或要求落点槽号随次序递增的条款或不变量（我读了 D3 已定项 1–10 全文与摘要表、D26 已定项 1、3、6、C146 与 `07-分配器.md` 的 I-5.5、I-5.6，没读 D3 已定项 11–17，没有读全 checker 源码，只 grep 了 `bump`） |
| 不引入新拒绝、不放宽盘不等大 | 补丁树上 `unequal_devices.rs`、`core_review_unit_area_start_and_publish_limits.rs` 里的用例变红（它们断言各盘回落答案与拒绝原因，我没有逐条看用例，只看了断言的是 `CommitGeneratedDeviceAnswer`） |
| 一次取落点新弄脏的叶数不比今天多 | 叶链模型任一行 `rewrite_set_first` 的 `new_leaves` 或 `rounds_taking_placements` 比 `lowest_free_slot` 大 |
| 演练与真发得同一个偏好集合 | 补丁树上现有用例目标或任何走真发布的用例在固定点之后 panic 或记录对不上（演练取的槽与真发取的槽不同）；补丁树扫描目标里 `outcome=published` 的行 panic |
| 偏好之后的收敛没有振荡 | 叶链模型出现 `did_not_settle_limit_64` 而 `lowest_free_slot` 在同一形态上定下了 |

## 九、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问题 1：按叶还是按段 | 按叶 | 叶（812）与段（64）网格不重合，只有叶能保证不弄脏新叶（2.1） |
| 问题 1：空槽不够退到哪里 | `rewrite_set_first` 退到今天的答案；备选 `rewrite_set_then_densest_leaf` | 前者不加新结构；后者要每叶空槽数索引，没被攻过（2.3） |
| 问题 1：多盘答案 | 要相同，不放宽 | 偏好只在各盘都答回落且一致后看，候选取每块盘都不被挡的槽（2.4） |
| 问题 2：D3 已定项 8 第 2 条与摘要行 | 冲突 | 回落句与「段耗尽回落同一规则」要改写（3.1） |
| 问题 2：D3 已定项 8 末段、已定项 5、已定项 10 | 一致 | 偏好只在回落一臂，不增拒绝（3.2） |
| 问题 2：D26 已定项 3 底额、已定项 1 第 3 条、C146 ① | 一致 | 回落不开段；`R` 排他保留；底额的一个边角没量（3.2） |
| 问题 2：C146 ③ | 好转一格（仅提交内生块类搬迁） | 整理批的大头是数据单元，不受影响（3.2） |
| 问题 2：分配记录树自身落点的环 | 没有新的环；「走得完」文档里「同一串槽位」那一句失效 | 新收敛论证是推的、没证（3.3） |
| 问题 2：崩溃状态与 checker | 规则没说（checker 里没有判回落落点的不变量） | 没有要动的不变量；崩溃点枚举的写数会随重写集变（3.4） |
| 问题 3：那 5 次探 | 不修 | 重写集的叶里没有空槽，取到的槽与今天逐个相同（推的，待 `differing_lines`）（4.1） |
| 问题 3：反过来变差的形状 | 没发现；聚集写放大与局部碎片没量；扫描代价量级推了没量 | 4.3 |
| 问题 4：代价数 | 三条候选的轮数、写放大、扫描代价、新结构见第五节；只有今天政策的轮数是量的，其余待跑 | 第五节 |

## 十、没做什么

- 没跑任何命令：没编译补丁（只做了 `rustfmt --check` 与 `patch` 对拍），没跑任何测试与模型，所以补丁能不能编过、模型能不能编过、两个单元测试绿不绿、模型读数是多少，全是「待主 agent 跑」。
- 没证收敛：「偏好之后固定点不振荡、走得完」只有推理，没有形式证明，也没有跑出读数。
- 没量局部碎片（`runs`、对齐空槽对数）与回落分配的扫描代价；没给扫描加游标。
- 没读全 checker 源码，也没逐条看 `unequal_devices.rs` 等用例的断言；改 kb、改 mutations、补发布层用例与 `R` 用例都不归我，列在 6.2。
- 没判别 `rewrite_set_then_densest_leaf` 要不要采纳：它没有被攻过，不替主 agent 判决；不替用户在三条候选里选。
- 自然历史那一段（用户操作写到盘满）没重跑，用的是主 agent 已有的 `settling-fallback.log`（量的，副本装置，不进 kb）。
- 清理：草稿目录 `/home/fy5090/.local/share/singlefs/work/d3-fallback-placement-r1-sonnet` 里我建的两份源文件拷贝（补丁的改前与改后各一份，删前 `du -sh` 各 1.3M）已 `rm -rf`；补丁本身在模型目录里，打补丁的对拍已做完。目录里剩下的是小的辅助脚本与日志（编辑脚本、`cite1.txt`、`cite2.txt`、`model-sha256.txt`、`steps.txt`、`sweep-check.rs`），没有仓副本、编译目录。

## 十一、cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/d3-fallback-placement-r1-sonnet-output.md` 的末行（原样）：

      ✓ 核了 44 处引文、21 处裸行号，对上 44 处，没判 21 处（其中裸行号没有引文可比 21 处；逐处列在上面）
