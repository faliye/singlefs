# 形式化核导读与 GC 证明边界

本目录不是整个文件系统的形式化实现。Verus 验证这个独立 crate 中的可执行纯函数、规格和引理；普通 Rust 构建擦除 ghost 代码后，core 调用其中一部分可执行函数。**某个核验证通过，不自动证明输入转换、设备实现或调用方满足其前提。**

GC 扩展的审查记录在 [records](../../../records/2026-10-24-GC形式化与core改动复核.md)。原始验证记录在 [扩展报告](../../../records/2026-10-24-GC实现与旧读者及崩溃协议形式化扩展.md)。以下说明当前源码，而非冻结历史。

## 十份源文件各负责什么

| 文件 | 可执行产品入口 | 验证内容与主要边界 |
| --- | --- | --- |
| [lib.rs](lib.rs) | `replay_core`、`choose_root_slot`、`first_copy_of_each_record_key` | 前缀规则、择根和记录去重，以及在明确前提下的重放唯一性/不动点；回调的盘面判断正确性不由核保证 |
| [compaction_plan_proofs.rs](compaction_plan_proofs.rs) | `choose_compaction_segment`、`plan_compaction_batch` | 选段资格、贪心和平局规则；计划顺序、数量、互异性和进展；另有按槽区间的搬迁保持引理（P4），前提含回退候选与读者的区间，它不是产品 I/O 代码 |
| [gc_ring_interval_proofs.rs](gc_ring_interval_proofs.rs) | 无 | GC 搬迁协议的根环版：映射值是槽区间，F 分内存与持久两层，根按版本号点名；任意有限历史的读取保持、三个反例被拒、G7 |
| [gc_crash_proofs.rs](gc_crash_proofs.rs) | **只有** `released_record_can_be_reclaimed` | 回收比较核，与「门槛蕴涵没被引用」那一条引理（两条前提显式写在 requires 里） |
| [gpu_gc_proofs.rs](gpu_gc_proofs.rs) | 双核仲裁、预算、分片、区域准入和回读的八个纯函数入口 | [GPU GC 的 G01–G20 与边界](GPU_GC_PROOFS.md)：主机端纯核直验，抽象计数协议、区间打分与 WGSL 双字算术模型另列边界 |
| [gpu_gc_capacity_proofs.rs](gpu_gc_capacity_proofs.rs) | `gpu_chunk_fits`、`largest_gpu_piece_input`、`tightened_gpu_memory_limit`、`checked_gpu_memory_hold`、`checked_gpu_memory_release`、`gpu_bytes_after_submission_wait`、`gpu_memory_cap_after_allocation_failure`、`gpu_workgroups_for_entries`、`checked_gpu_mapping_table_bytes`；`extended_gpu_chunk_input` 产品此刻没有调用点 | [G21–G30](GPU_GC_PROOFS.md)：切批准入、最大分片、显存账本、派发尺寸与映射表占用；显存界比的是交进来的上限，产品扣输入暂存那一层在核外 |
| [gpu_gc_scoring_proofs.rs](gpu_gc_scoring_proofs.rs) | 无 | 两份 GPU 活槽打分算法（逐条走与二分查）的模型：二分下界、候选窗口、逐槽计数与排列等价，任意长度不超过 `u32::MAX` 的记录表；WGSL 到模型的翻译、主机排序与 GPU 执行在边界外（[GPU_GC_PROOFS.md](GPU_GC_PROOFS.md)「两份打分算法的归纳证明」） |
| [file_range_proofs.rs](file_range_proofs.rs) | `next_tail_batch`、`bytes_in_unit`、`checked_write_end`、`payload_length_from_start`、`units_covering_size`、`units_for_sequential_write`、`rounded_size_fits`、`checked_unit_byte_range`、`checked_bytes_in_indexed_unit` | 截断/删除的批量释放边界、进展与尾区间分区；部分写交集的精确覆盖、精确交集长度（`range_intersection_length`，单元分界与请求分界两侧长度相加等于整体）、两侧偏移与容量上界；相邻单元无重叠无遗漏；按单元号算单元端点的乘加与溢出拒绝 |
| [unlink_proofs.rs](unlink_proofs.rs) | `next_unlink_batch`、`size_after_an_unlink_batch` | unlink 的分批计划：整串计划的形状与步数、每个数据单元恰好释放一次、从任一中间版续做的计划等于一路做下来的后半截；删除批的下段节点每层至多重写截口那一个（按跨度模型）；size 乘法不回绕 |
| [partial_write_proofs.rs](partial_write_proofs.rs) | `classify_partial_write_unit`、`first_unit_to_rewrite`、`units_after_partial_write`、`unit_containing_byte` | 部分写五类单元来源、重写起点、写后单元数；端点覆盖/相交判定与逐字节语义等价；抽象字节模型的覆盖规律与整单元覆盖不依赖旧内容 |

核的入口文件为 `lib.rs`。九份文件由 `lib.rs` 的九个 `pub mod` 引入，其中 `gc_ring_interval_proofs.rs` 又引入 `gc_ring_interval_proofs/` 下的两份；M3 的引理在 `lib.rs` 的私有模块 `idempotence_and_monotonicity_lemmas.rs`，纯函数核 `replay_core`、`choose_root_slot`、`first_copy_of_each_record_key` 在它的子模块 `idempotence_and_monotonicity_lemmas/pure_function_kernels.rs`，由 `lib.rs` 再导出。`.claude/gate.d/verus-pins.tsv` 登记核文件和固定工具版本；`verus-must-red-worlds.tsv` 登记反例。完整 proof 门禁同时做逐项验证和无过滤的整 crate 验证。

## core 到 compaction 核的转换

### 选段

`singlefs-core::compaction_plan::choose_region` 先统计目标设备各完整段的活槽数，忽略已释放记录、异盘记录和不落在完整段内的槽；这些循环仍在 core，未进入 Verus。扫描表先按设备过滤，同一起点保留第一份扫描。

转换逐段保持顺序：第 i 项对应 `unit_area_start + i * CLUSTER_SEGMENT_SLOTS`。传入四个排除条件和三态扫描结果：未扫描、扫描后有可搬项、扫描后无可搬项。核中的候选恰好是四个排除条件均为 false 且不是“扫描后无可搬项”。

核按 `live_slots` 最小选候选，只有严格更小时更新 best。输入顺序与段起点顺序相同，因此同分保留最早下标等价于原算法取最小槽号。返回下标再由相同公式转换为 CompactionRegion：有可搬项返回 Chosen，未扫描返回 ScanBeforeChoosing，无候选返回 NoEligibleSegment。

这段论证依赖输入地址算术有效。core 的面积、地址宽度、段对齐和记录跨度约束不由核证明。转换另分配一个按完整段数的输入向量；并未证明其资源消耗上界适合所有池规模。

### 批计划

1. `CompactionBatchUnits` 包住 `NonZeroUsize`，所以传给核的 K 至少为一。
2. 适配层在整份扫描上检查可搬 key 互异，即使重复项位于本批 K 项以外也拒绝。
3. 每条扫描记录转换为同下标的一条核记录，不排序、不过滤。五种分类逐一对应；源槽号保持不变。
4. 以 `BTreeMap<&[u8], u64>` 给完整 key 字节分配身份。比较按字节内容，不按借用地址；共同前缀、空 key、内嵌零都不会混为一把 key。第一次出现分配当前表长，重复出现沿用旧身份。输入不可变，借用不会悬空，也不用克隆整份扫描的 key。
5. 核返回前 K 个可搬记录的下标。产品从原扫描取 key 和**完整 Placement**，保留源槽和 span；key 身份值不成为产品映射 key。
6. 没有计划项时，原扫描为空才是 RegionEmptied；有记录但全不可搬则是 NothingMovableButTheRegionIsNotEmpty。

这建立了人工可审阅的转换论证，并有独立参考和边界用例。适配代码自身仍是普通 Rust，不应称为已由 Verus 证明的一般精化。`plan_batch_skipping_the_units_this_publish_rewrites_anyway` 的特殊分支也仍在 core，不能直接套“输出最多 K 项”：当区域仅剩本次本来就重写的节点时，它可以返回全部节点，这是它自己的契约。

## GC 搬迁协议的数学对象

出处：判决 `research/prompts/m4-exit5-relocation-proof-r3-main-verification.md` 第二节 Z1 三行与第三节，D13（验证路线）已定项 18。

- `disk: Map<int, Seq<u8>>`：已持久的抽象槽，每个槽一段字节。没有设备号、副本与根环槽。
- `SlotRun { start, span }`：一个单元占的槽区间。读一个 key 是读它区间里每个槽，按槽序排成一串（`run_bytes_on_disk`）。槽号与跨度是数学整数；接 core 的 u64 时要另证起点加跨度不溢出。
- `RunView { mapping, logical }`：key 到区间的映射，加上这一版的逻辑内容。现行、每个根、每个读者各带自己的一份，所以不同逻辑版本（用户在旧根之后改过、删过 key）可以并存。
- `roots: Seq<RingRoot>`：根环里的持久根，旧到新；每个根带名字：用户发布的带版本号，整理的发布不带用户点得到的版本号。
- `in_memory_floor` / `persisted_floor`：内存 F 与持久 F。回退候选是内存 F 之上的根；写与回收的门要避开持久 F 之上的每个根。抬 F 先只改内存 F，下一次用户那一侧的发布落成根时持久 F 跟上；整理的发布只带持久 F、不带内存里抬过的，落成根时持久 F 只在根数超过容量时挤上一格；崩溃让内存 F 回到持久 F。
- `readers`：进程内的留存读者，崩溃后全部消失。

`every_protected_view_reads_right` 是不变量：现行、持久 F 之上的每个根、每个读者都读到自己的逻辑内容，且根环形状成立（持久 F 不高于内存 F、两者低于根数、持久 F 之上的根数不超过根环容量）。

`slot_is_unreferenced` 是写与回收的门：现行、持久 F 之上的每个根、每个读者，映射里都没有区间罩住这个槽。按区间比，两槽单元的第二槽也护得住；按持久 F 比，只在内存里抬过的 F 不放开任何槽。

## 十一个步骤为什么保持读取

| 步骤 | 前提 | 推导 |
| --- | --- | --- |
| Write / Reclaim | 槽没被引用 | 改的槽不在任何受保护视图的区间里，每个区间的字节不变 |
| Relocate | key 在现行映射里；源、目的区间都在盘上、跨度相同、字节相同 | 被搬 key 改区间不改内容；其余 key 不变；根与读者继续指旧区间 |
| UserPublish | 新视图在此刻的盘上读对 | 只换现行 |
| UserCheckpoint | 无 | 现行落成新根；根数超过容量时最旧的候选被挤出（内存 F 加一），持久 F 跟上新的内存 F；新 F 之上的旧根原本就受保护 |
| CompactionCheckpoint | 现行的逻辑内容与最后一个根相同（映射可以不同） | 现行落成新根；只带持久 F：根数超过容量时持久 F 之上最旧的根被挤出（持久 F 加一），内存 F 不低于它；持久 F 只升不降，新持久 F 之上的旧根原本就受保护 |
| Rollback（按版本号） | 版本号是一个回退候选的 | 现行换成那个根的视图，它在持久 F 之上、读对 |
| RaiseFloorInMemory（按版本号） | 同上 | 只改内存 F；保护范围看持久 F，不变 |
| AcquireReader / ReleaseReader | 释放时下标有效 | 追加现行，或删去一个读者 |
| Crash | 无 | 现行取最后一个根（在持久 F 之上），读者清空，内存 F 回到持久 F |

`run_ring_next` 自己不查前提；一步保持定理同时要 `run_ring_step_is_enabled`。整理的写目的地与回收源槽走的就是 Write、Reclaim 这两道门。

## 证明义务如何组合

- `lemma_writing_an_avoided_slot_keeps_the_view` 是帧引理；`lemma_every_run_ring_step_keeps_every_protected_view` 按十一种步骤分类；`theorem_every_finite_run_ring_history_keeps_every_protected_view` 对前缀长度归纳，任意有限长度，不是固定深度枚举。
- `theorem_a_crash_recovers_the_newest_root_and_is_idempotent`：恢复的是最后一个持久的根，再崩一次结果相同；内存里领先的现行不当恢复根。
- `lemma_the_in_memory_floor_and_the_persisted_floor`：只在内存里抬 F 不改任何槽的保护；崩溃让内存 F 回到持久 F；用户那一侧的发布落成根时两者相等；整理的发布落成根时持久 F 至多挤上一格、内存 F 不降。
- `lemma_a_compaction_publish_over_a_floor_raised_only_in_memory_loses_one_candidate_across_a_crash`：协议核实现员推的那个例子（根环容 3 个根、F 只在内存里抬到版本 2、插一次整理的发布再崩溃）：整理那份崩溃之后的候选是版本 1、2，只比不整理那份少 1 个；整理的发布改回带内存 F 就只剩版本 2（必红世界表里那一行）。
- 三个反例被拒：`lemma_both_slots_of_a_two_slot_unit_of_a_rollback_target_are_refused`（两槽单元第二槽，连同回退目标还在读的起点槽）、`lemma_reclaiming_the_source_after_a_compaction_publish_is_refused_while_the_old_root_is_a_candidate`（整理发布之后回收源槽）、`lemma_a_floor_raised_only_in_memory_keeps_the_old_root_protected_until_a_publish_carries_it`（只看内存 F 就放开回收）。另有两条非空见证：根环只容一个根时旧根被挤出、源槽可回收；两个逻辑版本与一个旧读者同时满足不变量。
- G7（`theorem_compaction_keeps_what_the_user_sees`）在用户看得见的那一层（根的内容与名字、内存 F、持久 F、现行内容）上说：同一串用户历史两份都接受了每一步、整理那份另插 k 次整理的发布，之后现行内容相同；整理那份的回退候选（按版本号）是不整理那份候选的后缀、至多少 k 个；抬 F 的上限恰多 k。`lemma_run_ring_steps_project_onto_the_observed_operations` 说根环的每一步在这一层上是哪一步（不发布的步骤不改这一层）；`lemma_compaction_never_admits_an_operation_the_plain_history_refuses` 说整理那份接受的点名不整理那份也接受。历史里可以有崩溃（两份同时崩溃，`lemma_a_crash_of_both_keeps_the_relation`）：对应关系另带持久 F 那一层（持久 F 之上的根同样是后缀、至多少 k 个）与「两份最后一个根的内容相同」，崩溃之后回退候选照样是后缀、至多少 k 个，现行内容照样相同。为此整理的发布只在现行内容与最后一个根相同时发（`compaction_publish_is_accepted`，穷举同样只在这时插整理的发布）。这一条是根环协议里整理那一步的门：投影引理说根环允许的整理的发布恰是这一层接受的，G7 要的「整理那份也接受每一步」由此从协议推出；`lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root` 是门拦下的那一段历史（用户改了现行、还没落成根时整理的发布不发，搬迁之后、改动落成根之后照发）。core 里整理的发布接在上一版后面只搬单元，攒批的会话在整理起跑之前先把待发排空。
- `compaction_plan_proofs` 的 P4 按区间：目的区间两两不重叠、不与现行映射的区间重叠、不压住回退候选与读者的区间、跨度等于源区间时，一批与一批批搬到 R 空都不改现行映射与回退候选、读者每个 key 读到的单元。「目的地压住回退目标两槽单元的第二槽」在这个前提下不合格（`lemma_a_destination_on_the_second_slot_of_a_rollback_target_unit_breaks_the_premise`）。
- `lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced`：门槛 `released && generation <= floor` 放行、释放代的含义与 floor 的下界两条前提成立时，记录区间里每个槽都没被引用。两条前提是分配器与 floor 来历那一侧的义务，还没证。

## 产品回收比较为什么没有改变语义

core 的 `CheckpointTxg` 是单字段 u64 newtype，派生 Ord。将 `record.generation <= floor` 改为两者 `.0` 的 u64 比较保持全宽和顺序，保留“等于也可回收”的边界，released 为 false 时仍拒绝。generation 在未释放记录上是分配代，只有 released 为 true 时才表示释放代。

核没有计算 floor；产品根据根环、回退与隔离等状态算 floor 的逻辑仍在 allocator。门槛到「没被引用」的那一步由 `lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced` 接上，但它的两条前提（版本不早于释放代的受保护视图不碰这条记录的区间；floor 不大于每个受保护视图的版本）要在 allocator 与抬 F、挂载重建分配器那几处另证，才能组合成全路径安全性。

## 验证结果应该怎样阅读

- 整 crate 的 verified 数以 `.claude/gate.d/verus-pins.tsv` 的 crate 行为准，0 errors 说明登记的这些义务在工具和规格信任边界内成立。
- 必红表中注入 assert(false) 检查证明函数确实进入验证；按起点比、门里去掉回退候选、门只看内存 F、回收门恒真、P4 前提不看回退候选、G7 的 k 改成 k − 1 或改成逐项相同、整理的发布改回带内存 F、整理的发布不要求现行内容与最后一个根相同、根环里整理的发布不设门或要映射也相同、G7 的对应去掉持久 F 那一层或去掉最后一个根相同、门槛引理去掉一条前提这些变异检查关键前提对结论有作用。锚点覆盖不等于每条语义都穷尽。
- requires 入口注入 false 失败只是未检出入口空真；不是一般可满足性证明。构造见证提供具体非空实例。
- 独立参考及边界测试验证 core 适配层的具体输入；有限测试不能把适配层提升为已证的一般转换。
- 协议到 core 的精化（设备屏障/FUA、根与 journal 编解码、校验、真实恢复选择、双盘与隔离、失败重试、u64 的溢出边界）与门槛引理的两条前提仍未证。不能称为整个文件系统的完整崩溃恢复证明。

## 文件区间核到 core 的边界

`truncate::next_truncation_step` 把目标 size 换成单元数后调用 `next_tail_batch`；`unlink::next_unlink_step` 经 `unlink_proofs::next_unlink_batch` 以目标 0 调同一核（见下一节）。核只要求批大小大于 0，调用方的 `UnlinkBatchSize` 产品值为 144，测试值限制在 1 到 144。证明覆盖现有单元数小于目标（尾部有洞）、恰好一批、零单元与 u64 极值；size 到单元数的转换现由下述取整核证明；单元地址计算与事务发布仍不在这里证明。unlink 的剩余 size 乘法由下一节的独立核证明。

`bytes_in_unit` 是求交那一半：核无前提，接受空或倒置区间：返回 None 当且仅当没有共同字节；返回 Some 时长度严格为正，两侧偏移指向同一文件字节，且范围恰好是交集。core 的 `PartialWriteOfAFile::bytes_written_into` 不直接调它，调的是 `checked_bytes_in_indexed_unit`（本页「按单元号计算部分写区间」一节），单元端点的乘法与加法连同求交在同一证明里，地址溢出显式拒绝。相邻单元分区引理证明交集不重不漏，不等于证明完整写入流程或 I/O 正确。

文件区间模块共 19 项 Verus 义务（按门禁 formal-verification 的 verus-proof 格成项：10 个 exec fn、9 个 proof fn，没有带 decreases 的 spec fn 与带值的 const）：9 个产品入口（文件表那一行列的）、8 条引理、2 个无前提见证（`witness_file_range_boundaries`、`witness_intersection_length_partitions`）。必须被拒绝的变异是必红表里第二段为 `crates/singlefs-replay-prefix/src/file_range_proofs.rs` 的各行。普通 Rust 测试另外枚举小区间的实际字节集、逐批释放的单元集，并检查全宽端点。

文件区间模块的 19 项义务里，管 u64 边界与声明长度的 4 项：

- `PartialWriteOfAFile::new` 直接调用 `checked_write_end`，返回 Some 当且仅当数学和装得进 u64；等于 u64::MAX 仍允许，溢出拒绝，不作饱和截断。
- `truncate::declared_length_within_a_size` 与 `write_request_split::payload_length_within_a_size` 在取得单元起点后调用 `payload_length_from_start`。后置条件证明返回值不超过容量，且 `[0, length)` 恰好包含单元中位于 EOF 之前的字节；空文件、EOF 后的起点和零容量均有定义。
- `lemma_payload_length_grows_by_at_most_the_size_change` 证明声明长度单调且增长量不超过 size 的增长量；`lemma_truncation_keeps_exactly_the_payload_prefix` 证明截短后保留的恰好是原载荷在新 EOF 以内的前缀。

这两处声明长度调用之前的单元起点乘法仍由调用方负责，没有移入证明。另有 7 个必红变异，包含删除两个引理各自的大小顺序前提。无前提边界见证覆盖恰好到 u64::MAX、溢出、部分载荷和 EOF 后起点，Rust 测试用 u128 数学和及枚举字节集合独立核对。

单元取整占这 19 项里的 4 项：3 个产品函数（exec fn）与 1 条单调引理（proof fn）；取整边界的 nonlinear_arith 子证明写在 `units_covering_size` 的函数体里，不单成一项。

- `units_covering_size` 以 `(size - 1) / capacity + 1` 处理非零 size，零 size 返回 0。前提为 capacity > 0；后置条件同时给出 `size <= count × capacity` 和非零 size 时 `(count - 1) × capacity < size`，证明覆盖足够且没有多算。乘积在规格中是数学整数，返回 count 本身不溢出。
- `units_for_sequential_write` 在此基础上使空文件保留一个单元，由 core 的 `data_unit_count_of_a_sequential_write` 直接调用，读侧数量检查、顺序写和截断目标计数因此共用同一核。
- `rounded_size_fits` 用覆盖计数与 checked_mul 判断整单元取整后的数学值是否装得进 u64；core 的 `target_size_rounded_up_to_a_whole_data_unit_fits` 直接调用。零大小的准入仍按 0 个单元，与空文件保留 1 个单元的语义有意区分。
- `lemma_covering_units_are_monotone` 证明在正容量与有序 size 前提下，所需单元数不倒退。

容量从产品格式取值，仍由调用方保证正数。单元取整这一块在必红表里有 6 行（`unit-count-` 开头三行、`sequential-write-empty-keeps-one-unit`、`rounded-size-refuses-overflow`、`unit-count-monotonicity-needs-ordered-sizes`），另有 3 个 Rust 回归测试（`file_range_proofs::tests` 的 `unit_count_matches_the_distinct_units_touched_by_each_byte`、`full_width_rounding_matches_wide_arithmetic_without_an_intermediate_overflow`，core `write_request_split` 的 `unit_count_and_truncation_admission_agree_at_the_last_representable_whole_unit`）；测试以逐字节触及的单元集合和 u128 向上取整作独立参照，并检查 core 使用真实净荷容量时的最大整倍数及后一字节。部分写那一路（`bytes_written_into`、`data_unit`）的单元起点与结束地址乘加已由 `checked_bytes_in_indexed_unit`、`checked_unit_byte_range` 证明（见「按单元号计算部分写区间」）；别的调用方各自的单元地址换算、调用方状态满足核的前提（计划公开字段的不变量）与完整 I/O 仍未由本模块证明。

## unlink 分批核到 core 的边界

`unlink::next_unlink_step` 调 `next_unlink_batch(剩下的单元数, 批大小)` 判这一步是不是最后一批、留下几个，调 `size_after_an_unlink_batch(留下的, 净荷容量)` 算非最后一批之后 inode 记录的 size；乘积装不进 u64 时核交回 None，core 停下（`expect`），不回绕。`next_unlink_batch` 的后置条件把它的返回值钉成规格 `unlink_plan(剩下的, 批大小)` 的第一步，所以下面几条关于整串计划的引理说的就是 core 一步一步走出来的那一串（每一步都只看现行那一版剩下的单元数，第 6 条）：

- `lemma_unlink_plan_shape`：至少一步；最后一步是最后一批、释放不多于一批；前面每步截掉恰好一批、至少留一个，下一步从上一步留下的起。
- `lemma_unlink_plan_length`：没有数据单元的文件只有一步，否则恰好 ⌈剩下 ÷ 批⌉ 步。
- `lemma_each_data_unit_released_exactly_once`：单元号小于起点的恰好被一步释放，不小于的没有一步释放。
- `lemma_continued_plan_equals_the_rest`：做完第 i 个非最后一批之后从留下的单元数续做，整串与一路做下来的后半截逐步相同（第 7 条 ①「计划相等」）。
- `lemma_deletion_batch_rewrites_only_the_cut_path`：按「一层第 index 个节点罩 [index × span, index × span + span)」的模型，截到 kept 个时这一层重写的节点恰好是下标 kept ÷ span 那一个且只在 kept 不是 span 倍数时出现，换下的恰好是整个落在被截掉那一段里的（K-R3-1）。

不证明：剩下的单元数从 extent 树怎么数出来、净荷容量取值、`plan_the_lower_segment_after_a_deletion_batch` 的 Vec 构造与它用的 `data_unit_range`（饱和乘加）与上面模型一致、事务发布与崩溃恢复。下段那条引理是模型上的结论，产品函数不调用它。必红世界表里第二段是 `crates/singlefs-replay-prefix/src/unlink_proofs.rs` 的有 12 行：计划的边界、截多一个、续做起点、最后一批留一个、size 换成加法、截口叶误判整片留下、节点存在的边界差一，形状与步数两条引理各跳过归纳步、续做引理断言多要一批、逐单元释放引理断言最后一批的计划没有步、截口引理里节点存在的蕴涵反向。

## 部分写计划核到 core 的边界

`PartialWriteOfAFile::data_unit` 将已有单元标记、写区间、单元区间传给 `classify_partial_write_unit`，再用穷尽 match 转成 core 的公开枚举；状态名称与两个读回标记方法在 core。五种来源的契约分别规定：旧单元仅整块被覆盖时免读，新单元只有被部分覆盖时读残留，未写到的新单元补零。两个引理证明在非空写区间、非空单元区间前提下，端点“整块覆盖”当且仅当每个单元字节都被写，端点“相交”当且仅当存在共同字节。

`first_rewritten_unit` 把写入起点、已有单元数和旧尾单元是否不满传给 `first_unit_to_rewrite`。契约给出精确起点，保证不晚于写入起点、不越过已有尾部；从旧尾之后写且旧尾未满时，必须把旧尾纳入重写。该布尔量仍由 core 用已验证的声明长度核求得，其输入单元起点换算不在本模块的证明内。

`data_units_after` 调 `units_after_partial_write`，证明结果恰好为已有单元数与写入终点所需单元数的最大值；既不丢旧后缀，也能容纳本次写入。写入终点必须非零、容量必须为正；前者来自非零长度和构造时的溢出检查，后者来自产品净荷容量。

部分写计划这一块有 6 项义务（按门禁 formal-verification 的 verus-proof 格成项：`unit_containing_byte`、`classify_partial_write_unit`、`first_unit_to_rewrite`、`units_after_partial_write` 4 个 exec fn 与上面两条引理 2 个 proof fn；`unit_containing_byte` 函数体里的 nonlinear_arith 子证明不单成一项）；`partial_write_proofs` 模块连同下一节覆盖写的 8 个 proof fn 共 14 项。必红表里第二段是 `crates/singlefs-replay-prefix/src/partial_write_proofs.rs` 的有 35 行，部分写计划这一块是 `partial-` 开头的 12 行，其余 23 行是下一节覆盖写的。证明依赖输入区间有效：构造后的计划字段须保持一致，单元端点的乘加不溢出；不宣称已经证明公开字段被任意改写后的计划、区间地址转换、五类来源对应的真实 I/O 或整个发布事务。核测试枚举实际覆盖字节并确认五类都出现，core 包装测试单独核对公开枚举的读旧数据/读残留/补零标记。

`first_written_unit` 与 `last_written_unit` 另调用 `unit_containing_byte`，分别定位写入首字节与末字节（end - 1）。核要求正容量，证明 `index × capacity <= byte < (index + 1) × capacity`；区间端点是数学整数，因此也覆盖 byte = u64::MAX，并不保证实际构造下一个单元地址时不会溢出。core 包装测试通过枚举写到的字节集合核对首尾单元，核测试以 u128 端点检查极值。

## 固定长度覆盖写的字节规律

`partial_write_proofs` 的 `overwritten_bytes` 用 `Seq<u8>` 表示固定长度文件内的覆盖写，`overwrite_fits` 要求写入起点非负、整个请求位于文件内。证明义务包括：

- `lemma_overwrite_changes_exactly_the_requested_bytes`：长度不变，写内字节来自请求，写外字节保留旧值，空请求不改变文件；另按段说写入那一段恰是请求、前后两段恰是原值，这三条要 `overwrite_fits`（请求越出文件时那几段的 `subrange` 落在序列之外）。逐字节那一条是 `overwritten_bytes` 定义的展开，它钉住的是定义本身。
- `lemma_overwriting_the_same_range_keeps_only_the_last_write`：同起点、同长度的连续覆盖只保留最后一次的内容。
- `lemma_disjoint_overwrites_commute`：不相交的两次覆盖可以交换顺序，两种顺序下第二次写仍满足范围前提；允许相邻区间和空请求。
- `lemma_whole_unit_overwrite_does_not_depend_on_old_bytes`：利用现有整单元覆盖引理，证明分类核的覆盖条件足以使单元结果不依赖旧内容。
- `witness_overwrite_laws_and_overlapping_write_order`：给出满足各引理前提的具体非空调用、EOF 处的空请求，以及重叠写交换后结果不同的反例。
- `lemma_splitting_an_overwrite_preserves_the_result`：任意合法分界点把请求分成相邻的前缀与后缀，依序写入与整段一次写入相同，两段均满足范围前提；包含空前缀和空后缀。
- `lemma_a_contained_overwrite_folds_into_the_payload`：后写完全位于前次请求内时，依序写入与先改请求内容、再整段写回相同，两条路径均满足范围前提。
- `witness_split_and_contained_overwrites`：构造非空分段写入、严格包含的后写、两端分界与空后写，钉住组合后的具体字节。

这些性质仅针对抽象字节模型，不代表 core 的缓冲区拷贝、扩展文件、补洞、设备 I/O 或持久化顺序已经精化到此模型。规格使用数学整数和有限序列，不带运行时代码或可信假设。必红表里这一块是 `overwrite-` 开头的 23 行：各函数的契约反例（`overwrite-contract-*` 五条、`overwrite-composition-contract-*` 三条，每条结论改成 false）、六条删除关键前提的反例（第一条引理的范围前提、同长度、不相交、整单元覆盖、合法分界、后写包含于请求）、两条改坏 `overwritten_bytes` 定义的反例（写入段整段右移一格、写入段少一格）、两条改坏另两条引理结论的反例（`overwrite-last-write-wins`、`overwrite-disjoint-writes-keep-both`）与两条改坏第一条引理另两条结论的反例（`overwrite-exact-keeps-the-length` 长度不变改成多一字节、`overwrite-exact-empty-request-keeps-the-file` 空请求不改文件放宽到长度不超过 1）；组合性质另有后缀偏移错位、后写偏移错位和见证字节错误的反例。

复跑本模块（命令由主 agent 在仓根经统一任务服务交，`.claude/agent-common.md`「子 agent 不交任务」；证明逐项的项名是 `proof:<全路径>`，照门禁 formal-verification 的 `--list-items` 列出的写）：

```sh
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 formal-verification.sh 带 --item; 范围=点名的 8 项; 为什么=复判覆盖写字节模型的范围前提、同长度、不相交、整单元覆盖、后缀偏移、后写偏移、合法分界与后写包含于请求八行反例' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item must-red:overwrite-exact-needs-fit --item must-red:overwrite-last-needs-equal-lengths --item must-red:overwrite-commute-needs-disjoint-ranges --item must-red:overwrite-whole-needs-coverage --item must-red:overwrite-split-needs-suffix-offset --item must-red:overwrite-contained-needs-file-offset --item must-red:overwrite-split-needs-valid-boundary --item must-red:overwrite-contained-needs-payload-bounds .

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的 --lib 里 partial_write_proofs 的 4 条测试; 范围=4 个测试; 为什么=复核部分写计划核的四个 Rust 测试' task-server/task submit -- cargo test -p singlefs-replay-prefix --lib -- partial_write_proofs::tests::located_unit_contains_the_byte_even_when_its_end_exceeds_u64 partial_write_proofs::tests::unit_sources_match_the_bytes_actually_overwritten partial_write_proofs::tests::rewrite_start_includes_the_gap_and_the_partial_old_tail partial_write_proofs::tests::after_count_preserves_old_units_and_covers_the_write_at_full_width --exact
```

## 按单元号计算部分写区间

`file_range_proofs::checked_unit_byte_range` 验证单元起点乘法与终点加法，返回精确端点，或在完整区间超出 u64 时返回 None。零容量得到空区间，恰好到 u64::MAX 的终点可以表示。`PartialWriteOfAFile::data_unit` 已使用此核；无法表示的地址显式拒绝，不依赖 debug 构建的算术溢出检查。

`checked_bytes_in_indexed_unit` 把地址换算与求交组合在同一证明中：外层 None 表示地址溢出，Some(None) 表示可表示区间内没有写入字节；Some(Some(...)) 给出两个相对偏移及正长度，并与数学上的逐字节交集等价。core 的 `bytes_written_into` 直接调用它，地址溢出时显式 panic，其余返回行为保持不变。

`range_intersection_length` 用数学整数端点给出精确交集长度：最大起点到最小终点之间的正差，否则为零。`bytes_in_unit` 的空结果对应长度零，非空结果的 `length` 直接等于此规格；`checked_bytes_in_indexed_unit` 在地址可表示的外层 `Some` 内保留同样关联，单元端点使用数学乘法与加法。外层 `None` 仍只表示地址溢出，不能当作零长度交集。core 直接消费返回的 `length` 字段，因此该契约关联的是执行核给出的长度。

`lemma_adjacent_unit_intersection_lengths_add_up` 证明在单元内合法分界时两侧交集长度相加等于整体，允许写区间为空或倒置。`lemma_split_write_intersection_lengths_add_up` 证明在写请求内合法分界时两段请求的交集长度相加等于整体，允许单元区间为空或倒置。无前提见证 `witness_intersection_length_partitions` 覆盖内部与两端分界、具体的 `4 = 1 + 3` 和 `4 = 2 + 2`、空与倒置区间及 u64 上界附近的单字节交集。规格与组合引理只涉及字节区间和返回长度，不代表缓冲区拷贝、设备 I/O 或持久化顺序已经精化到这些性质。

`lemma_rounded_size_bounds_written_unit_addresses` 将取整准入条件连接到单元地址安全性：正容量、取整后长度可表示且单元号小于 size 所需单元数时，该单元终点不溢出。size 之外保留的已有单元需要文件状态另给界限。无前提边界见证覆盖该引理及 `checked_unit_byte_range`、`checked_bytes_in_indexed_unit` 两个入口。

这一节的核没有封闭计划公开字段的不变量，也没有验证五类来源到 core 枚举的转换或实际 I/O。别的单元地址换算各由调用方负责。
