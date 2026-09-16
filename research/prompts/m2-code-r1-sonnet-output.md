# 发布 B 三方对抗第一轮：正推腿报告（Sonnet）

立场：S1–S7 逐条与条款原文对，再判 X3、X6、X7。结论先给：**S1–S7 全部一样，未发现代码与条款不符处**；X3（恢复放开）未打中；X6 有一处接缝要记欠账（BirthSequenceAllocator 每次发布重建），其余三处写死当前判无害；X7 现状文本与代码逐句核对一致，两处轻微措辞落差记在下面，不构成「现状与代码不符」。

方法：读 `research/prompts/_m2-code-r1-background.md`（含附录一 kb 条款）与附录二 diff、两个新测试文件全文；对每条被判的 kb 条款去源文件现查行号（不从背景材料数）；对代码去 `crates/singlefs-core/src/{allocator,transaction,recovery}.rs`、`crates/singlefs-harness/src/crash.rs` 现读；跑了 `cargo test -p singlefs-core -p singlefs-harness --test second_transaction_step_one_overwrite --test second_transaction_step_zero_layer0`，4 + 2 个非 ignored 用例全绿（1 个全量用例按设计 `#[ignore]`，留给门禁 54 号 release 模式跑）。

## 一、S1：释放先于分配

**条款**（`.claude/kb/decisions/03-空间分配.md:180`，已定项 7）：「落点释放时条目不删、不点删（C113 定案 P4，2026-09-05）：改写成『已释放 + 释放代』——释放代仍写进那 8 字节，已释放标志位借跨度段的最高位，条目留到该落点被重新分配时覆盖……『释放』的口径 = 分配器把落点放进 defer 队列那一刻 = 没有任何根（含快照）引用它」。
已定项 11（`.claude/kb/decisions/03-空间分配.md:420-426`）：「进 value：key = (设备身份 4, 16 KiB 槽号 6) 共 10 字节，value = (跨度段 2, 分配代或释放代 8) 共 10 字节；已释放标志仍在跨度段最高位」。

**代码**：
- `crates/singlefs-core/src/allocator.rs:16` `pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;`——最高位借位，未占跨度值。
- `allocator.rs:21-29` `AllocationRecord` 字段 `span_slots`（纯跨度，注释「不含标志位」）+ `generation`（注释「仍分配时是分配代；已释放时是释放代」）+ `is_released: bool`；`to_bytes`（`allocator.rs:33-51`）把标志位或进 `span_field` 再写 20 字节，字段序与已定项 11 的 key(10)+value(10) 完全一致；`parse`（`allocator.rs:62-76`）反向解出 `is_released` 与去标志后的 `span_slots`。
- 条目不删：`release`（`allocator.rs:309-326`）在 `self.records` 里 `find` 到既有条目原地改字段（`record.is_released = true; record.generation = release_generation;`），没有任何 `remove`/`retain`——与「不删、不点删」一致。
- 释放先于分配：`transaction.rs:700-704`
  ```rust
  // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
  for placement in release {
      allocator.release(*placement, txg);
  }
  ```
  紧接着才是「落点先于内容」的八个单元取号循环（`transaction.rs` 注释原文「落点先于内容」那一段，`IN_BUMP_ORDER` 循环）。释放代取的是这次发布的 `txg`（`publish_file_version` 的形参 `publish.txg`），与已定项 7「释放代 = 那次发布的 checkpoint_txg」一致。
- 「槽仍占着、分配器不再把它发出去」：`release` 调用 `device.mark_released`（`allocator.rs:104-112`——`assert!(self.allocated[start..end].iter().all(|taken| *taken))` 断言这些槽此前必须是「已分配」，函数体没有把 `self.allocated[index]` 改回 `false`），槽在 `allocated` 位图里保持 `true`；`lowest_user_data_slot`（`allocator.rs:214-230`）只按 `is_free`（即 `!self.allocated[index]`）挑候选，已释放的槽因 `allocated[index]` 仍是 `true` 而不会被选中。测试 `releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue`（`allocator.rs:503-534`）钉住了「已释放的 50180 不许再发，下一个偶数空槽对是 50182」。

**结论：一样。** 条款说「不删、改写、槽仍占着、释放代=这次 txg」，代码逐句照做；释放循环在装单元之前执行，顺序也对。

## 二、S2：记账口径

**条款**（D5「已定项 4」表，`.claude/kb/decisions/05-快照-空间记账机制.md:1852-1867`）：第 1 项「已分配字节」带设备维、第 2 项「空闲字节」带设备维、第 5 项「defer 队列待释放（按代）」带设备维。已定项 8（`05-快照-空间记账机制.md:2270`）第一个事务写 15 行的规则「有值的统计量就写一行」。checker 读法甲（`.claude/kb/invariants.md:120`，I-3.1）：「已分配空间统计 == 实际遍历所有引用得到的和」，⚠️ 注（`invariants.md:1743` 背景材料引文）「实际遍历所有引用」按根环里全部有效根的引用取并集。I-5.2（`invariants.md:167`）：「空闲空间统计 == 总空间 − 已分配空间」。

**代码**：
- `allocator.rs:106-108`（`DeviceFreeMap.allocated_slots` 字段注释）：「占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间」；`release`（`allocator.rs:309-326`）不减 `allocated_slots`（只调用 `mark_released` 增 `deferred_slots`，`allocator.rs:159-168`），与「已分配含已释放」一致。
- `transaction.rs:879-885`：`STATISTIC_ALLOCATED_BYTES` 写 `device_map.allocated_slots() * SLOT_BYTES`，`STATISTIC_FREE_BYTES` 写 `device_map.free_slots() * SLOT_BYTES`（即 `unit_area_slots - allocated_slots`，`allocator.rs:151-153`），与「空闲 = 单元区 − 已分配」逐字一致。
- `transaction.rs:888-892`（对应 diff 里 `STATISTIC_DEFER_QUEUE_BYTES` 那一段）：写 `device_map.deferred_slots() * SLOT_BYTES`，即第 5 项「defer 待释放」。
- checker 侧：`crates/singlefs-checker/src/walk.rs:52` `references: BTreeMap<(u32, u64, u64), String>`——按 `(device, slot, span)` 去重的容器；`walk.rs:727-735` 先走最新根（`walk.walk_root(..., true)`）、再遍历环里其余每个自证过的根（`walk.walk_root(..., false)`），全部写进同一个 `walk.references`；`walk.rs:767-778` 对每盘把 `per_device` 里的 span 求和当 `walked`，与记账里的 `STATISTIC_ALLOCATED_BYTES` 比对（`judgements.judge("I-3.1", allocated == Some(walked), ...)`）。因为 `references` 按 `(device, slot, span)` 去重、来自根环里全部自证根的并集，这正是「读法甲：按根环里全部有效根的引用取并集」——A 的根仍在环里、仍会被 `walk_root(..., false)` 走一遍，A 的八个单元因此仍计入 `walked`，与已释放但未被抛弃的槽仍算「已分配」一致。

**结论：一样。** 测试 `release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue`（`crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs:983-1007`）钉住了「已分配 = mkfs 3 + A 10 + B 10 = 23 槽」「defer 待释放 = 10 槽」「空闲 + 已分配 = 单元区」，与 S2「B 之后每盘：已分配 23 槽、defer 10 槽」逐字相符；`cold_start_reads_the_second_content_and_the_pool_checker_stays_green` 断言 I-3.1 / I-5.1 / I-5.2 / I-7.2 真被评估过且 `Holds`（本次 `cargo test` 复核为绿）。

⚠️ **一处观察，不影响 S2 的对照结论，标出供 X2 那条腿参考**：`.claude/kb/decisions/05-快照-空间记账机制.md:398` 写「第 2 项必须独立维护，不许由 `容量 − 已分配` 现算……若空闲就是这么算出来的，那条不变量是恒真式，判别力为零」，而 `transaction.rs:879-885` 里 `STATISTIC_ALLOCATED_BYTES` 与 `STATISTIC_FREE_BYTES` 都从同一个 `device_map`（同一份 `allocated_slots` 计数）派生，`free_slots()`（`allocator.rs:149-153`）本身就是 `unit_area_slots - allocated_slots`——写盘的「空闲」值与「已分配」值不是两条互不共享的加减路径。这不改变 S2 的对照结论（S2 本身给出的定义就是「空闲 = 单元区 − 已分配」，代码照做），但它意味着 checker 的 I-5.2 目前测不出「空闲统计自己算错了」这一类 bug，只测得出「容量读错了」或「已分配读错了」。这一格不在我的判项（X1/X2/X4 由本地腿判），原样报出，不代入判定。

## 三、S3：映射

**条款**：D19（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:198`，已定项 5）「取混合——位置条目降为提示，中央映射是解引用与释放判定的唯一入口」；已定项 8（背景材料引文，全部指针进映射，「映射树自己的节点、树表单元、实例表单元自举豁免」）；已定项 12（`19-块指针的结构与宽度预算.md` 已定项索引表第 12 行，日期 2026-09-13）「维持 day-1 写出（第一个事务 6 条）」。

**代码**：`transaction.rs` 里 `mapping_entries` 数组（对应 diff 里紧邻 `mapping_key_sort_key` 那一段，即本次读到的 `crates/singlefs-core/src/transaction.rs:955-983`）只装 6 个条目——`data_pointer`、`extent_pointer`、`inode_leaf_pointer`、`inode_root_pointer`、`allocation_pointer`、`accounting_pointer`，不含 `mapping_pointer` 自己与 `TreeTable`，与「映射树 / 树表单元自举豁免」一致；`publish_overwrite` 与 `publish_first_file` 共用同一段 `publish_file_version`（`transaction.rs:686` 起），这段构造代码对两条入口完全相同——不携带上一版映射条目、每次都是从这一次的 8 个单元现建 6 条。

**结论：一样。** 测试 `release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue`（`second_transaction_step_one_overwrite.rs:1017-1023`）断言 `second.mapping_keys.len() == 6` 且「A 的映射条目一条都不留」（`!first_mapping_keys.contains(key)`），与 S3「上一版六个单元的条目一条不留，只写这次八个单元里进映射的六个」逐字相符。

## 四、S4：inode 记录

**条款**：D8「已定项 6」（`.claude/kb/decisions/08-核心索引结构.md:332`）：容器身份「一片叶活很多代、反复重写」；`08-核心索引结构.md:403`「88 偏移：改动计数（最后一次改动所在发布的 checkpoint_txg）」；`08-核心索引结构.md:371`「容器身份不需要分配器：inode 号在每一条时间线上单调……分裂只发生在最右叶」（本次覆盖写不分裂，容器号、容器出生代原样沿用）。

**代码**：`transaction.rs:291-309`（`FilePublish` 结构体）显式区分了四类不随覆盖写变化的参数：`inode_object_birth`（注释「对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改」）、`tree_birth_txg`（注释「树表条目的诞生 txg：树建起来那次发布，覆盖写不改」），与随每次发布变化的 `change_count`（注释「改动计数」）。`publish_overwrite`（`transaction.rs:660-676`）传入 `inode_object_birth: previous.inode_record.object_birth`（继承 A 的，不重算）、`change_count: txg.0`（这次发布的 txg）、`tree_birth_txg`（从 `previous.tree_table_entries.first().birth_txg` 取，`transaction.rs:655-659`，即树表第 1 版第一条的诞生 txg）。容器身份（容器号、容器出生代）在 `inode_leaf_identity` 构造处（对应 diff 里 `PackedIdentity { ..., container: FIRST_INODE_NUMBER, container_birth: publish.tree_birth_txg }` 那一段）用的正是 `publish.tree_birth_txg`，两次发布都指向树建起来那次（txg 3），不随覆盖写变化。

**结论：一样。** 测试断言 `second.inode_record.object_birth == CheckpointTxg(3)`「对象出生代不改」、`second.inode_record.change_count == 4`「改动计数 = 这次发布的 checkpoint_txg」（`second_transaction_step_one_overwrite.rs:893-901`）；`release_rewrites...` 测试断言 `entry.birth_txg == CheckpointTxg(3)`「树的诞生 txg 不随重写变」（`second_transaction_step_one_overwrite.rs:1030-1032`），与 S4「对象出生代不变（3）、容器身份不变……树表七条条目的诞生 txg 不变（3）」逐字相符。

## 五、S5：journal

**条款**：D16「已定项 6」（`.claude/kb/decisions/16-发布语义.md:246`）「每次发布把 checkpoint_txg 加一……根槽的轮转键……记账的代……恢复重放的水位都是同一个计数」；「已定项 7」（`16-发布语义.md:285`）「一次发布的持久顺序恒为：COW 单元/节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」；D23「已定项 19」②（`.claude/kb/decisions/23-journal的角色与格式.md:1261`，背景材料引文第 2 段）`previous_hash` = 「CRC32C(本实例内逻辑前一条记录的头，其中 header_csum 那 32 字节按零参与)」；D23「已定项 7」（`23-journal的角色与格式.md:285`）「一个事务在发出单元写之后失败即实例切换……事务号按实例计数、从 1 起」。

**代码**：`transaction.rs:664-665`（`publish_overwrite`）`let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);`；`counter: previous.record.counter + 1`、`transaction: previous.record.transaction + 1`（`transaction.rs:665-666`），jsn（`counter`）与事务号各在上一次发布的基础上加一，与「txg / jsn / 事务号各加一」一致。反向链：`journal.rs:98-105` 的 `back_chain_of` 取传入字节的前 307（`JOURNAL_HEADER_BYTES`）字节、把偏移 `JOURNAL_HEADER_CHECKSUM_OFFSET` 起 32 字节清零再算 CRC32C——「307 字节」「header_csum 32 字节按零参与」与已定项 19 ② 逐字对应；`publish_overwrite` 把 `&previous.record_bytes`（A 的完整记录字节）作为 `previous_record_bytes` 传进 `publish_file_version`，而 `record.back_chain = back_chain_of(previous_record_bytes)`（对应 diff 里 `record` 构造那一段），即「本实例内逻辑前一条」取的正是 A（同一实例内紧邻的上一条）。`root.rollback_floor = CheckpointTxg(0)`、`tree_identifier_watermark` 沿用第一个事务的常量、`instance_table: publish.instance_table`（覆盖写传入 `previous.root.instance_table`，即照旧不变）。

**结论：一样。** 测试断言 `second.record.counter == 4`「jsn 接着 A 的 3」、`second.record.transaction == 2`「事务号按实例计数从 1 起，B 是 2」、`second.record.back_chain == back_chain_of(&first.record_bytes)`「反向链 = A 那条记录头的 CRC32C」、`second.root.tree_identifier_watermark == 19`「没有建新树，水位不动」、`second.root.rollback_floor == CheckpointTxg(0)`、`second.root.instance_table == first.root.instance_table`「实例表指针照旧」（均见 `second_transaction_step_one_overwrite.rs:842-856`），与 S5 每一句都对得上。段序列种类串 `[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]`（`second_transaction_step_one_overwrite.rs:838-840`）与第一个事务同型，符合「段序列与第一个事务同型」。

## 六、S6：恢复走读放开

**条款**：D23「已定项 14」（`.claude/kb/decisions/23-journal的角色与格式.md:1206`）正文给的是所选根之后的重放规则，未直接约束「分配记录数应恒为多少」——那一条硬编码本来就是 `walk_to_file` 自己按 E142 的一次性镜像立的自检（recovery.rs 里明确标注 `invariant: "E142 走读同款"`），不是某条 kb 条款直接钉死的不变量；已定项 7（`.claude/kb/decisions/03-空间分配.md:175`）「释放时条目不删、不点删：改写成『已释放 + 释放代』」是「已释放记录合法」这条放宽的直接依据。

**代码**：`recovery.rs:624-631`：
```rust
// 每个落点每盘一条：第一个事务 10 × 盘数，每次覆盖写再加 8 × 盘数（换下的那些改写、不删）。
if roots.allocation.entries.len() < 10 * device_count
    || !roots.allocation.entries.len().is_multiple_of(device_count)
{
    return Err(...{ detail: "分配记录数不是盘数的整数倍或少于 10 × 盘数" });
}
```
`recovery.rs:645-654`：
```rust
// 已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
for record_bytes in &roots.allocation.entries {
    let record = AllocationRecord::parse(record_bytes);
    if record.generation > root.checkpoint_txg || record.span_slots == 0 {
        return Err(...{ detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" });
    }
}
```
旧版本（diff 里删掉的那半）额外判 `record.span_slots & 0x8000 != 0`（带已释放标志即拒），新版本去掉了这一判据，只保留「跨度为 0」与「代晚于根」两条，且这两条对 `is_released` 的记录同样成立（`AllocationRecord::generation` 字段的语义按 `is_released` 取分配代或释放代，`allocator.rs:26-27`）。`recovery.rs:639-643`「映射条目数不是 6」这一判据本次改动完全没碰，字面仍是 `roots.mapping.entries.len() != 6`。

**结论：一样。** S6 描述的两处放宽（「恒 10×盘数」→「盘数整数倍且不少于 10×盘数」、「不许带已释放标志」→「已释放记录的代按释放代判不晚于根」）与代码逐字对应；「映射条目数恒 6」确实没动——它压在的不是某条硬性不变量，而是「一次文件覆盖写恰好写出 8 个单元、其中 6 个进映射（MappingTree 与 TreeTable 自举豁免）」这个 S3 已核对过的结构性事实：只要这一形状（一个文件、8 个单元、6 条映射）在步 1/步 2 内保持不变，这条自检就仍然成立；它不是「按定义永远是 6」，是「按当前唯一场景现算出来是 6」，详见第八节 X3 的判定。

## 七、S7：层 0 多版本 oracle

**条款**：D13「已定项 4」（`.claude/kb/decisions/13-验证路线.md:326`）「崩溃点重放只枚举整写子集」；E77 判据 1（`.claude/kb/experiments/77-发布的持久顺序.md:8-11`）「违例 = 根槽已持久而恢复 ≠ 新态或走读失败……根槽未持久时新旧两态都合法」；S7 本身补的多版本读法：「实际走的根是第几代就得读出那一代写出的内容……不许走到比 T 旧的根……根下没文件的代只许报没有文件；单版本形态是它的特例」。

**代码**：`crash.rs:549-552` `PublishedVersion { checkpoint_txg, content }`；`crash.rs:557-587` `oracle_violation_for_versions`：
- 没择到根 ⇒ 违例（`crash.rs:562-563`）。
- `newest_persisted_root_txg.is_some_and(|newest| effective_txg < newest)` ⇒ 「根槽已持久而恢复到旧态」（`crash.rs:564-570`）——对应「不许走到比 T 旧的根」。
- 用 `effective_txg` 在 `versions` 里 `find` 对应版本（`crash.rs:571-573`），`(NoFile, None)` 判无违例（该代下没有文件、报没有文件，`crash.rs:576`）、`(NoFile, Some(_))`「第 N 代根下面有文件却报没有」违例（`crash.rs:577-579`）、`(FileRead, None)`「第 N 代根下面没有文件却读出了内容」违例（`crash.rs:580-583`）、`(FileRead, Some(version))` 内容不等才违例（`crash.rs:584-586`）——四种组合覆盖了「有文件读对/有文件读错/无文件报错/无文件报对」，单版本（`versions` 只有一个元素）时这段逻辑退化成旧版 `oracle_violation` 的判法，是「单版本形态是它的特例」的直接体现。
- `newest_persisted_root_txg`（`crash.rs:593-600`）：`publishes_in(writes)` 按 `StepKind::RootRecordFua` 切出每次发布，过滤 `persisted[publish.root]` 为真的，取 `checkpoint_txg` 的最大值——正是「盘上已持久的最新根槽是第 T 代」的定义。

**结论：一样。** 测试 `damage_probes_after_the_overwrite_tell_the_new_unit_from_the_released_one`（`second_transaction_step_one_overwrite.rs:1152-1179`）里「B 的根槽坏一字节 ⇒ 择回 A 的根、由 jsn 4 那条记录重建第 4 代根、内容仍是第二次的」这一格，恰好是「`effective_txg`（4，由记录重建出来）与 `newest_persisted_root_txg`（3，B 的根槽写坏了没持久）之间 `4 >= 3` 不触发『恢复到旧态』」与「`versions` 里 txg=4 那条内容仍是 `second_content()`」两条规则联合给出的判定，与 S7 逐句对得上。

## 八、X3：恢复的放开会不会放进本该拒的镜像

判两条具体形态：

1. **「带已释放标志但代晚于根」**：`recovery.rs:645-654` 的新判据对全部记录统一判 `record.generation > root.checkpoint_txg`，不区分 `is_released`；`AllocationRecord::generation` 字段本身按 `is_released` 取分配代或释放代（`allocator.rs:26-27`），所以「已释放记录的释放代晚于根」与「未释放记录的分配代晚于根」走的是同一行代码、同样被拒。**未打中**——这一形态没有被放过。

2. **「记录数是盘数整数倍但两盘条数不等」——打中。** `recovery.rs:625-631` 的判据只检查 `roots.allocation.entries.len()`（全池合并总数）`>= 10*device_count` 且 `is_multiple_of(device_count)`，没有任何一处按设备身份分组核对「每盘条数相等」；批注「每个落点每盘一条」（`recovery.rs:624`）暗示了「两盘应当各占一半」的期望，代码没有兑现这句注释。举例：两盘、`device_count=2`，一份镜像里设备 0 有 20 条分配记录、设备 1 有 16 条（合计 36，是 2 的倍数、也 ≥ 20），这份镜像会**通过**这一格判据——即便它明显违反「一次发布两盘各一条同槽记录」（`allocator.rs:248-253` `Placement` 注释「两盘同槽」；`allocator.rs:292-305` `record()` 对 `self.devices` 里每一块盘各推一条）这个结构性前提。⚠️ 这不是这次改动新引入的缺口：改动前的等值判据（`!= 10 * device_count`）同样只判总数，同样没有按设备分组核对；这次改动把等值放宽成区间，只是把同一处既有的盲区又放大了一圈（原来只能在总数固定为 20 时钻空子，现在任意 ≥20 且被 2 整除的总数都能钻）。这一格是 `walk_to_file` 自己标注为「E142 走读同款」的自检，不是 checker（`walk::check_pool_image`）的不变量——它的失守不代表 checker 层面 I-3.1 / I-5.1 也测不出（两条不变量走的是独立的树遍历 + 引用集合比对，见 S2），但它确实是「recovery 冷启动读」这条独立路径上一个本该被这句注释挡住、实际没挡住的口子。**判定：X3 打中一半**（第 1 条未打中，第 2 条打中）。按反向接受条款记欠账，不在这一轮里改代码（我的角色是正推腿，不动代码）。

## 九、X6：接缝

- **`BirthSequenceAllocator` 每次发布重建，出生序号从 0 起（`transaction.rs:552-562`，`publish_file_version` 顶部 `transaction.rs:701` `let mut sequences = BirthSequenceAllocator::default();`）——第一版可以，往后两步会埋坑。** D19「已定项 9」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:1096`）逐字「同一棵树内每写出一个码 2 或码 3 单元加 1（两类共用一个计数）；作用域换到下一个 checkpoint 时清零」——清零的时机是「换到下一个 checkpoint」，不是「每次调用一个装某个对象的函数」。今天 `publish_file_version` 恰好一次调用 = 一次发布 = 一个文件，两者重合，所以在 `(tree, txg, instance)` 这个键上重建等价于持久化后再清零，S1–S7 的对照结论不受影响。但里程碑「并行线三：多个文件」一旦落地、多个对象在**同一个 checkpoint** 里各自调用一次类似 `publish_file_version` 的入口（例如两个文件都touch 了 inode 树），每次调用都会让 `BirthSequenceAllocator::default()` 重新从 0 计数，两个对象各自的码 2/码 3 单元会在同一个 `(inode 树, 这个 txg, 这个实例)` 键上分别拿到从 0 起的出生序号，撞出重复——这正是已定项 9「同一棵树内」要求的连续计数被打断。这是格式层面的正确性问题（出生序号是 D19 已定项 8 里进映射 key 的一段），不是可以事后无痛改的实现细节。**记欠账**：`BirthSequenceAllocator` 要在「一次 checkpoint」这个作用域里跨调用共享（比如作为参数从更外层的「一次发布」入口传进去，而不是每次публиш函数自己 `default()`），留给里程碑步 3（第二个可写实例）或并行线三开工前处理。
- **`FIRST_INODE_NUMBER` 写死为 1（`transaction.rs:54`）、`walk_to_file` 只找 `inode == FIRST_INODE_NUMBER`（`recovery.rs:717`）——不埋新坑。** 两者都绑在「第一版唯一文件是 inode 1」这同一个前提上，而这条前提已经在 `invariants.md`（I-9 类正文，见背景材料 1797-1802 行区间对应的 `.claude/kb/invariants.md:247` 起）与里程碑「并行线三：多个文件」条目里被显式记成未来要解决的事，不是这批代码新引入、没人知道的空白。
- **`TransactionOutput::placements` 按 `unit.identity.placement()` 这个静态分类推跨度（`transaction.rs:516-528`）——目前不埋坑，但是耦合点。** 它没有读 `AllocationRecord.span_slots` 这个「实际记下来的跨度」，而是按单元的静态种类重新算一遍（`UserData=>2`、`TwoSlotsAligned=>2`、`OneSlot=>1`）。今天这个映射与分配器自己的行为（`allocate_user_data` 恒 span=2、`allocate_commit_generated` 按 `UnitFootprint` 给 1 或 2）完全同构，所以两条路径永远一致，不构成 S1–S7 的不一致。但它是一处「跨度靠猜种类，不靠读记录」的耦合：一旦某个单元种类的跨度不再是种类的纯函数（比如未来的打包容器按内容大小变跨度），这个函数会悄悄算错却不会有任何编译期或运行期信号——因为它现在就没有跟 `AllocationRecord` 的实际字段做过任何比对。建议往下走的步骤里，这个函数改成直接消费 `TransactionOutput.allocation_records` 里对应槽号那条记录的 `span_slots`，而不是重新分类猜一遍；今天不改不算错，先记一笔观察。

## 十、X7：`02-second-txn.md` 步 1 / 步 2 的「现状」与代码

逐句核对（见第四至七节 S4–S6 的引用与测试断言），**步 1 / 步 2 两段现状文字与代码、与两个新测试文件的断言，没有发现一处不符**。数字（50182、50249–50256、区域 1 槽 1、txg 4、jsn 4、事务号 2、世代 6、tail 4、已分配 23 槽、defer 10 槽、runs 4、全空段 3310、水位 2、映射 6 条、checker 23 条）逐项在 `cargo test` 复核为绿的两个测试文件里都能找到对应断言，已在第四至七节标出行号。「会红：四处变异……改回全绿」这一句我按测试里现有的断言逐条对了应该由哪一句抓（S1 一节已列），**但没有真的去改代码跑一遍这四处变异**——判 X5（对照够不够）不在我的分工范围，这句「已验证会红」的实测状态留给判 X5 的那条腿。

⚠️ **一处不在 X7 判项范围内、但读代码时顺带发现的落差，如实报出**：`02-second-txn.md` 步 0（`.claude/kb/milestone/02-second-txn.md:58`）「现状（2026-09-16 建档）：未开工」，而新测试文件 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 的文件头注释自陈「里程碑『第二个事务』步 0 在发布 B 上的那一半：把『取号 → 暖机 → A → B』整条录制流……枚举全部崩溃状态」，且 `crash.rs` 里 `PublishedVersion` / `oracle_violation_for_versions` / `enumerate_layer0_versions` 等一整套多版本层 0 装置已经写出并通过测试（本次 `cargo test` 复核 2 个非 ignored 用例绿、1 个全量用例按设计 ignore）。步 0 的完整设想（切换、C、回退与 D、攒根、抬 F、E 六个发布的固定脚本）确实还没做，但「未开工」这三个字与已经落地并跑通的「A→B 两次发布层 0」这部分装置不符——这句话按字面会让人以为步 0 一行代码都没有。**这条不在 X7 的判项文字里（X7 只点了步 1 / 步 2），所以不计入 X7 的判定，但按 `three-way-inference.md`「打中就是打中」的精神原样报出，请交用户或下一轮判要不要改成「部分开工：发布 B 的层 0 装置已落地」。**

## 十一、小结

S1–S7：**全部一样**。X3：**部分打中**（记录数按盘数整数倍的判据没有按设备分组核对，本该拒的非对称镜像会被这一格放过；已释放记录代晚于根这一形态没有被放过）。X6：**一处需要记欠账**（`BirthSequenceAllocator` 按调用重建，往后多对象共享一个 checkpoint 时会撞出生序号重复），其余两处（`FIRST_INODE_NUMBER`、`placements` 按种类推跨度）当前无害，后者留一句耦合观察。X7：**步 1 / 步 2 的现状与代码一致**；步 0 的「未开工」与已落地的层 0 装置不符，超出判项范围但一并报出。
