# 形式化核导读与 GC 证明边界

本目录不是整个文件系统的形式化实现。Verus 验证这个独立 crate 中的可执行纯函数、规格和引理；普通 Rust 构建擦除 ghost 代码后，core 调用其中一部分可执行函数。**某个核验证通过，不自动证明输入转换、设备实现或调用方满足其前提。**

本次 GC 扩展的审查记录在 [records](../../../records/2026-10-24-GC形式化与core改动复核.md)。原始验证记录在 [扩展报告](../../../records/2026-10-24-GC实现与旧读者及崩溃协议形式化扩展.md)。以下说明当前源码，而非冻结历史。

## 四份源文件各负责什么

| 文件 | 可执行产品入口 | 验证内容与主要边界 |
| --- | --- | --- |
| [lib.rs](lib.rs) | `replay_core`、`choose_root_slot`、`first_copy_of_each_record_key` | 前缀规则、择根和记录去重，以及在明确前提下的重放唯一性/不动点；回调的盘面判断正确性不由核保证 |
| [compaction_plan_proofs.rs](compaction_plan_proofs.rs) | `choose_compaction_segment`、`plan_compaction_batch` | 选段资格、贪心和平局规则；计划顺序、数量、互异性和进展；另有按槽区间的搬迁保持引理（P4），前提含回退候选与读者的区间，它不是产品 I/O 代码 |
| [gc_ring_interval_proofs.rs](gc_ring_interval_proofs.rs) | 无 | GC 搬迁协议的根环版：映射值是槽区间，F 分内存与持久两层，根按版本号点名；任意有限历史的读取保持、三个反例被拒、G7 |
| [gc_crash_proofs.rs](gc_crash_proofs.rs) | **只有** `released_record_can_be_reclaimed` | 回收比较核，与「门槛蕴涵没被引用」那一条引理（两条前提显式写在 requires 里） |

核的入口文件为 `lib.rs`，另三份文件通过模块引入。`.claude/gate.d/verus-pins.tsv` 登记核文件和固定工具版本；`verus-must-red-worlds.tsv` 登记反例。完整 proof 门禁同时做逐项验证和无过滤的整 crate 验证。

## core 到 compaction 核的转换

### 选段

`singlefs-core::compaction_plan::choose_region` 先统计目标设备各完整段的活槽数，忽略已释放记录、异盘记录和不落在完整段内的槽；这些循环仍在 core，未进入 Verus。扫描表先按设备过滤，同一起点保留第一份扫描。

转换逐段保持顺序：第 i 项对应 `unit_area_start + i * CLUSTER_SEGMENT_SLOTS`。传入四个排除条件和三态扫描结果：未扫描、扫描后有可搬项、扫描后无可搬项。核中的候选恰好是四个排除条件均为 false 且不是“扫描后无可搬项”。

核按 `live_slots` 最小选候选，只有严格更小时更新 best。输入顺序与段起点顺序相同，因此同分保留最早下标等价于原算法取最小槽号。返回下标再由相同公式转换为 CompactionRegion：有可搬项返回 Chosen，未扫描返回 ScanBeforeChoosing，无候选返回 NoEligibleSegment。

这段论证依赖输入地址算术有效。core 已有的面积、地址宽度、段对齐和记录跨度约束不因接核而获得新证明。新转换还增加一个按完整段数分配的输入向量；并未证明其资源消耗上界适合所有池规模。

### 批计划

1. `CompactionBatchUnits` 包住 `NonZeroUsize`，所以传给核的 K 至少为一。
2. 适配层在整份扫描上检查可搬 key 互异，即使重复项位于本批 K 项以外也拒绝。
3. 每条扫描记录转换为同下标的一条核记录，不排序、不过滤。五种分类逐一对应；源槽号保持不变。
4. 以 `BTreeMap<&[u8], u64>` 给完整 key 字节分配身份。比较按字节内容，不按借用地址；共同前缀、空 key、内嵌零都不会混为一把 key。第一次出现分配当前表长，重复出现沿用旧身份。输入不可变，借用不会悬空，也不用克隆整份扫描的 key。
5. 核返回前 K 个可搬记录的下标。产品从原扫描取 key 和**完整 Placement**，保留源槽和 span；key 身份值不成为产品映射 key。
6. 没有计划项时，原扫描为空才是 RegionEmptied；有记录但全不可搬则是 NothingMovableButTheRegionIsNotEmpty。

这建立了人工可审阅的转换论证，并有独立参考和边界用例。适配代码自身仍是普通 Rust，不应称为已由 Verus 证明的一般精化。`plan_batch_skipping_the_units_this_publish_rewrites_anyway` 的特殊分支也仍在 core，不能直接套“输出最多 K 项”：当区域仅剩本次本来就重写的节点时，它可以返回全部节点，这是原有契约。

## GC 搬迁协议的数学对象

出处：判决 `research/prompts/m4-exit5-relocation-proof-r3-main-verification.md` 第二节 Z1 三行与第三节，D13（验证路线）已定项 18。

- `disk: Map<int, Seq<u8>>`：已持久的抽象槽，每个槽一段字节。没有设备号、副本与根环槽。
- `SlotRun { start, span }`：一个单元占的槽区间。读一个 key 是读它区间里每个槽，按槽序排成一串（`run_bytes_on_disk`）。槽号与跨度是数学整数；接 core 的 u64 时要另证起点加跨度不溢出。
- `RunView { mapping, logical }`：key 到区间的映射，加上这一版的逻辑内容。现行、每个根、每个读者各带自己的一份，所以不同逻辑版本（用户在旧根之后改过、删过 key）可以并存。
- `roots: Seq<RingRoot>`：根环里的持久根，旧到新；每个根带名字：用户发布的带版本号，整理的发布不带用户点得到的版本号。
- `in_memory_floor` / `persisted_floor`：内存 F 与持久 F。回退候选是内存 F 之上的根；写与回收的门要避开持久 F 之上的每个根。抬 F 先只改内存 F，下一次发布落成根时持久 F 跟上；崩溃让内存 F 回到持久 F。
- `readers`：进程内的留存读者，崩溃后全部消失。

`every_protected_view_reads_right` 是不变量：现行、持久 F 之上的每个根、每个读者都读到自己的逻辑内容，且根环形状成立（持久 F 不高于内存 F、两者低于根数、持久 F 之上的根数不超过根环容量）。

`slot_is_unreferenced` 是写与回收的门：现行、持久 F 之上的每个根、每个读者，映射里都没有区间罩住这个槽。按区间比，两槽单元的第二槽也护得住；按持久 F 比，只在内存里抬过的 F 不放开任何槽。

## 十一个步骤为什么保持读取

| 步骤 | 前提 | 推导 |
| --- | --- | --- |
| Write / Reclaim | 槽没被引用 | 改的槽不在任何受保护视图的区间里，每个区间的字节不变 |
| Relocate | key 在现行映射里；源、目的区间都在盘上、跨度相同、字节相同 | 被搬 key 改区间不改内容；其余 key 不变；根与读者继续指旧区间 |
| UserPublish | 新视图在此刻的盘上读对 | 只换现行 |
| UserCheckpoint / CompactionCheckpoint | 无 | 现行落成新根；根数超过容量时最旧的候选被挤出（F 加一），持久 F 跟上新的 F；新 F 之上的旧根原本就受保护 |
| Rollback（按版本号） | 版本号是一个回退候选的 | 现行换成那个根的视图，它在持久 F 之上、读对 |
| RaiseFloorInMemory（按版本号） | 同上 | 只改内存 F；保护范围看持久 F，不变 |
| AcquireReader / ReleaseReader | 释放时下标有效 | 追加现行，或删去一个读者 |
| Crash | 无 | 现行取最后一个根（在持久 F 之上），读者清空，内存 F 回到持久 F |

`run_ring_next` 自己不查前提；一步保持定理同时要 `run_ring_step_is_enabled`。整理的写目的地与回收源槽走的就是 Write、Reclaim 这两道门。

## 证明义务如何组合

- `lemma_writing_an_avoided_slot_keeps_the_view` 是帧引理；`lemma_every_run_ring_step_keeps_every_protected_view` 按十一种步骤分类；`theorem_every_finite_run_ring_history_keeps_every_protected_view` 对前缀长度归纳，任意有限长度，不是固定深度枚举。
- `theorem_a_crash_recovers_the_newest_root_and_is_idempotent`：恢复的是最后一个持久的根，再崩一次结果相同；内存里领先的现行不当恢复根。
- `lemma_the_in_memory_floor_and_the_persisted_floor`：只在内存里抬 F 不改任何槽的保护；崩溃让内存 F 回到持久 F；发布落成根时两者相等。
- 三个反例在这一版上被拒：`lemma_both_slots_of_a_two_slot_unit_of_a_rollback_target_are_refused`（第三轮攻方的两槽单元第二槽，连同回退目标还在读的起点槽）、`lemma_reclaiming_the_source_after_a_compaction_publish_is_refused_while_the_old_root_is_a_candidate`（第一轮攻方的整理发布之后回收源槽）、`lemma_a_floor_raised_only_in_memory_keeps_the_old_root_protected_until_a_publish_carries_it`（只看内存 F 就放开回收）。另有两条非空见证：根环只容一个根时旧根被挤出、源槽可回收；两个逻辑版本与一个旧读者同时满足不变量。
- G7（`theorem_compaction_keeps_what_the_user_sees`）在用户看得见的那一层（根的内容与名字、内存 F、现行内容）上说：同一串用户历史两份都接受了每一步、整理那份另插 k 次整理的发布，之后现行内容相同；整理那份的回退候选（按版本号）是不整理那份候选的后缀、至多少 k 个；抬 F 的上限恰多 k。`lemma_run_ring_steps_project_onto_the_observed_operations` 说根环的每一步在这一层上是哪一步（不发布的步骤不改这一层）；`lemma_compaction_never_admits_an_operation_the_plain_history_refuses` 说整理那份接受的点名不整理那份也接受。G7 不说崩溃，也没用到「整理只在现行内容与最后一个根相同时发布」。
- `compaction_plan_proofs` 的 P4 按区间：目的区间两两不重叠、不与现行映射的区间重叠、不压住回退候选与读者的区间、跨度等于源区间时，一批与一批批搬到 R 空都不改现行映射与回退候选、读者每个 key 读到的单元。第三轮攻方的「目的地压住回退目标两槽单元的第二槽」在这个前提下不合格（`lemma_a_destination_on_the_second_slot_of_a_rollback_target_unit_breaks_the_premise`）。
- `lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced`：门槛 `released && generation <= floor` 放行、释放代的含义与 floor 的下界两条前提成立时，记录区间里每个槽都没被引用。两条前提是分配器与 floor 来历那一侧的义务，还没证。

## 产品回收比较为什么没有改变语义

core 的 `CheckpointTxg` 是单字段 u64 newtype，派生 Ord。将 `record.generation <= floor` 改为两者 `.0` 的 u64 比较保持全宽和顺序，保留“等于也可回收”的边界，released 为 false 时仍拒绝。generation 在未释放记录上是分配代，只有 released 为 true 时才表示释放代。

核没有计算 floor；产品根据根环、回退与隔离等状态算 floor 的逻辑仍在 allocator。门槛到「没被引用」的那一步由 `lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced` 接上，但它的两条前提（版本不早于释放代的受保护视图不碰这条记录的区间；floor 不大于每个受保护视图的版本）要在 allocator 与抬 F、挂载重建分配器那几处另证，才能组合成全路径安全性。

## 验证结果应该怎样阅读

- 整 crate 的 verified 数以 `.claude/gate.d/verus-pins.tsv` 的 crate 行为准，0 errors 说明登记的这些义务在工具和规格信任边界内成立。
- 必红表中注入 assert(false) 检查证明函数确实进入验证；按起点比、门里去掉回退候选、门只看内存 F、回收门恒真、P4 前提不看回退候选、G7 的 k 改成 k − 1 或改成逐项相同、门槛引理去掉一条前提这些变异检查关键前提对结论有作用。锚点覆盖不等于每条语义都穷尽。
- requires 入口注入 false 失败只是未检出入口空真；不是一般可满足性证明。构造见证提供具体非空实例。
- 独立参考及边界测试验证 core 适配层的具体输入；有限测试不能把适配层提升为已证的一般转换。
- 协议到 core 的精化（设备屏障/FUA、根与 journal 编解码、校验、真实恢复选择、双盘与隔离、失败重试、u64 的溢出边界）与门槛引理的两条前提仍未证。不能称为整个文件系统的完整崩溃恢复证明。
