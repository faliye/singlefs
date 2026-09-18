# m2-step3-code-r1 云端正推腿报告

立场：核「代码做的是不是条款说的」，不找反例。每格给代码行号 + 条款原文（附录整行抄）+ 判定。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| S1 | 一致，另有一处条款没说 | 前缀过滤、断号即止、maximum_applied_transaction 与已定条款逐句对得上；「所选根自己那条读不出时从最小一条接」全仓无条款出处 |
| S2 | 一致 | rebuild_version 的读法（位置提示直读、树表空拒开）与 D16 已定项8 的前置、D19 已定项5 的「释放走映射」一致；已知缺口是里程碑自己标出的 Y1 射程，不是新发现 |
| S3 | 一致，一处条款没说 | 已分配/已释放/独立空闲计数与 D3 已定项7、D5 已定项4 一致；「开放段不续」「重开后第一次分配从空闲位图重找」全仓无条款出处（milestone 自己也标了「预想里没定」） |
| S4 | 一致 | 取号 max+1、逐盘写、随后一道屏障，与 D23 已定项16、C322 定案逐句对得上 |
| S5 | 一致，一处收窄 | T/W/中间实例/txg/jsn/事务号/反向链与 D18 已定项11、D23 已定项14/19 逐句对得上；「jsn 取全部记录里最大值而非严格前缀末」是一处比条款窄读法更宽的实现，未见冲突 |
| S6 | 一致 | 暖机循环、四个固定点单元、R=3 上限、无 fsync 接口，与 D16 已定项8（甲′）、D28 已定项4 一致 |
| S7 | 一致 | txg 8、instance 2、按映射释放 B 的八个落点，与 D23 已定项7/19、D3 已定项7 一致（由验收用例逐字钉住） |
| S8 | 一致 | judge_instance_table_rows 的三条判据（唯一/低于挂载根/链指针在末尾）与 I-3.8 逐句对得上；回收那一半 I-3.8 自己写明「未实现」，代码也没做，不算不一致 |
| S9 | 一致 | oracle 按 (txg, 实例) 二元序、newest_persisted_root 偏移读取，与 D22 已定项7 的择新序一致；Ignore 分支另计 ignored_violations 与 D13 已定项4 的口径一致 |
| S10 | 一致 | E142 装置的 replay_journal 与 crates 的两条规则逐行相同（见 X9），verification_ran 9→6 有单测钉住 |
| S11 | 一致 | 七处改法在源码里都能找到对应实现，见 X11 范围之外，此处只核落地 |
| X9 | 同一条规则 | 候选过滤、链首取法、断号即止三处逐行比对未见分歧；E142 装置缺 maximum_applied_transaction 字段（因为装置不做写行），不构成规则分歧 |
| X10 | 基本对得上，一处需回填 | milestone 步0/步3现状、layout/01-first-txn 八の两行「装置钉住」、I-3.8 状态列都已与代码同步；仅第 391 行「取号那一行」末尾的屏障归属描述已按代码更新（2026-09-17），核对无误 |


## S1：前缀规则

**代码**：`crates/singlefs-core/src/recovery.rs:598-683`（`replay_journal`）。

- 候选过滤，`recovery.rs:618-623`：
  ```rust
  let mut above: Vec<&JournalRecord> = records
      .values()
      .filter(|record| {
          record.instance == root.instance && (record.instance, record.checkpoint_txg) > water
      })
      .collect();
  ```
- 链首取法，`recovery.rs:628-637`：
  ```rust
  let root_own_record_counter = records
      .values()
      .find(|record| {
          record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
      })
      .map(|record| record.counter);
  let mut expected_next: Option<(InstanceGeneration, u64)> =
      root_own_record_counter.map(|counter| (root.instance, counter + 1));
  ```
- 断号即止，`recovery.rs:638-643`：`expected_next` 与下一条不符即 `break`。
- 最大已施加事务号，`recovery.rs:669-670`：`report.maximum_applied_transaction = report.maximum_applied_transaction.max(record.transaction)`。

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1236`，已定项14 的「跨实例边界、链首……」第 1 条）：
> **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（六条口径第一条原样）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。

**六条口径**（`decisions/23-journal的角色与格式.md:1221-1227`，前四条与本格相关）：
> **前缀判定的完整口径是六条，缺一不可**……jsn 严格连续（断号即止）、**`(实例代号, checkpoint_txg)` 大于根的水位**、提交标记齐全的事务才施加……施加前逐项验证点名单元……

**判定**：

1. `record.instance == root.instance` 与「前缀规则不跨实例边界」逐句一致；`root.instance` 是 mkfs 第 0 代根（实例 0）时，因为记录都从实例 1 起（D18 已定项11「从 1 起、0 无效」），过滤条件永远不命中，与「一条记录都不覆盖」一致——**一致**。
2. `(record.instance, record.checkpoint_txg) > water` 与「大于根的水位」逐字一致——**一致**。
3. `expected_next` 断号检查与「jsn 严格连续（断号即止）」一致——**一致**。
4. `report.maximum_applied_transaction` 与 D18 已定项11（`.claude/kb/decisions/18-块里携带什么信息.md:882`）「所选根那个实例写 (i, 重放之后那个根的 checkpoint_txg, **属于实例 i 的、被这次重放施加的最大事务号**)」一致——**一致**。
5. **条款没说**：`root_own_record_counter` 取「所选根自己那条记录（同实例、checkpoint_txg 相等）的 jsn + 1」当链首，读不出时 `expected_next = None`（等价于「从水位之上同实例最小的一条接」，因为 `above` 已按 `(instance, counter)` 排序、`expected_next` 为 `None` 时第一条无条件通过）。全仓搜索：
   ```
   $ grep -n "读不出.*接\|从水位之上同实例最小" .claude/kb/decisions/23-journal的角色与格式.md
   （零命中）
   ```
   已定项14 只说「链从所选根覆盖的最后一条记录之后接」，没有说「那条记录读不出时怎么办」。代码把「同实例、checkpoint_txg 相等」当作「所选根覆盖的最后一条记录」的可操作定义——这是合理的操作化（因为发布与记录一一对应，D23 已定项7），但读不出时的接续点是实做时自己补的，**没有条款**，列入下面「条款没说的取法」表。什么会推翻这一条一致判定：找到一条条款明确规定了「所选根自己那条记录读不出时前缀该怎么接」而与代码不同。


## S2：重开重建（rebuild_version）

**代码**：`crates/singlefs-core/src/recovery.rs:327-424`（`rebuild_version`）；`mount.rs:195-209` 调用处。

- 读实例表 `recovery.rs:341`、树表 `342-350`、树表空报错 `351-355`（`tree_table.entries.is_empty()` → `InvariantViolated{invariant:"挂载",...}`，`mount.rs:196-200` 映射成 `MountError::NoPublishedVersion`）。
- 读五棵树的根节点（extent/inode/allocation/accounting/mapping）`recovery.rs:363-397`，都经 `read_unit_via_locations`（`recovery.rs:194-214`：按位置条目直读，校验和过才算读到）。
- `rewritten: Vec::new()`，`recovery.rs:422`。

**条款原文**（`.claude/kb/decisions/16-发布语义.md`「已定项 8」正文，前置 C314 已满足那一句）：
> 前置 C314（回退可以复用被抛弃的根引用的单元） 的**条款**那一半 2026-09-13 已满足……暖机已生效；**检查**那一半仍欠（崩溃点重放 harness 的多次挂载录制流）……

**条款原文**（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:209-210`，已定项5）：
> 位置条目……**降为提示**。宽度与段划分不变，变的是它的地位——读到它可以直接去那个落点试，但它可能过期。
> 中央映射……**解引用与释放判定的唯一入口**。搬迁只改这里一条条目，「谁引用我」根本不用知道。

**判定**：

1. `read_unit_via_locations` 直接按位置条目试读——与已定项5「读到它可以直接去那个落点试」一致——**一致**。
2. `rebuild_version` 本身不做释放判定（它是读路径，不是发布路径），已定项5「释放判定的唯一入口是映射」这一条的落点不在这里，而在后续 `placements_to_release_via_mapping`（`transaction.rs:580`）——它读的是 `previous.unit(TransactionUnit::MappingTree).bytes`，即 `rebuild_version` 从盘上重建出来的映射树单元字节，不是 `mapping_keys` 那个只剩 key 的派生字段（`recovery.rs:405-409`）。所以「重开之后的释放仍然经映射」——**一致**。
3. 树表为空报 `NoPublishedVersion`：milestone 步3 现状段自陈「第一版只支持至少发过一版文件的池」（`.claude/kb/milestone/02-second-txn.md:135`）。这一条没有对应的已定条款字面规定（D16 已定项8 只讲暖机的因果关系，没讲树表为空时挂载该怎么处理），**这是实做的取法，milestone 已标出**，不算「代码违反条款」，也不重复列入「条款没说」表（milestone 已经把它记成决策点：会碰到的决策点行没有点名它，但现状段已如实陈述）。
4. `rewritten: Vec::new()`：`TransactionOutput.rewritten` 字段的文档注释（`transaction.rs`）写「这次发布真正写出的角色」，`rebuild_version` 产出的不是一次发布，是「上一版」，`rewritten` 留空与语义一致——**一致**。
5. **milestone 自己标出的已知缺口**（不是本腿新发现）：`.claude/kb/milestone/02-second-txn.md:156`「重开之后上一版从盘上重建：释放判定路径今天读的是同进程的内存态（`TransactionOutput`）、一个 reader 都不带，从盘上重建 `previous` 时映射条目里的位置项带单元校验和，释放之前要按它核盘上那个单元（第二轮攻方腿 Y1 射程）」——这一条 milestone 已如实记录为待办，不算代码与条款不符，只是尚未做到的检查。

什么会推翻：找到 `rebuild_version` 之外的路径绕开映射直接按位置提示做释放判定。


## S3：分配器重建

**代码**：`crates/singlefs-core/src/allocator.rs:284-308`（`rebuild_from_records`）：
```rust
pub fn rebuild_from_records(devices: Vec<DeviceFreeMap>, records: Vec<AllocationRecord>) -> Self {
    let mut allocator = Self::new(devices);
    for record in &records {
        let device_map = ...;
        let span = u64::from(record.span_slots);
        device_map.mark_allocated(record.slot, span);
        if record.is_released {
            device_map.mark_released(record.slot, span);
        }
    }
    allocator.records = records;
    allocator
}
```
`Self::new`（`allocator.rs:275-282`）把 `open_segment: None`（`allocator.rs:278`）。`free_slots` 是独立字段（`allocator.rs:112`），`mark_allocated` 里 `self.free_slots -= span`（`allocator.rs:215`），不是现算。

**条款原文**（`.claude/kb/decisions/03-空间分配.md`，已定项7，「落点释放时条目不删」段）：
> **落点释放时条目不删、不点删**……改写成「已释放 + 释放代」——释放代仍写进那 8 字节，已释放标志位借跨度段的最高位，条目留到该落点被重新分配时覆盖。

**条款原文**（`.claude/kb/decisions/05-快照-空间记账机制.md:1742-1745`）：
> **第 2 项必须独立维护，不许由 `容量 − 已分配` 现算。** I-3.1（已分配统计对得上） 逐字就是「空闲统计 == 总空间 − 已分配空间」——若空闲就是这么算出来的，**那条不变量是恒真式，判别力为零**……两个数要走两条不共享的加减路径。

**判定**：

1. `mark_allocated` 后 `if record.is_released { mark_released }`：`mark_released` 只 `self.deferred_slots += span`（`allocator.rs:172`），不清 `allocated` 位图——槽仍占着，与「条目留到该落点被重新分配时覆盖」「槽仍占着、不发出」一致——**一致**。
2. `free_slots` 是独立字段、`mark_allocated` 里递减，不是 `容量 − 已分配` 现算——与「两个数要走两条不共享的加减路径」一致——**一致**（milestone 步2 现状段也自陈这一条修法：`.claude/kb/milestone/02-second-txn.md:111`「把此前『由单元区 − 已分配现算』的写法……摘掉，checker 的 I-5.2 会红」）。
3. **条款没说**：`Self::new` 把 `open_segment` 重置为 `None`（milestone 步3 现状段自陈「预想里没定、这一步实做时定下的（都标预想、等用户）」列表里含「重开后开放段不续」，`.claude/kb/milestone/02-second-txn.md:135`）；全仓搜索：
   ```
   $ grep -rn "开放段.*续\|open_segment.*rebuild" .claude/kb/decisions/
   （零命中，milestone 文件本身除外）
   ```
   D3 已定项5/已定项8 定的是「聚簇段只给提交内生块」这个分配政策，没有讲「重开之后开放段状态要不要延续」。这是实做取法，milestone 已标出，同样列入下面的表。
4. **条款没说**：「重开后第一次分配从空闲位图重找最低空槽对」——同样在 milestone 那句「预想里没定」清单里；D3 已定项8 定的是稳态下的落点政策（已选设备内槽号最小的空槽），没有专门讲重开这一刻的初始状态，代码的做法（`open_segment=None` 之后 `lowest_user_data_slot` 自然从头找）与已定项8的稳态政策是同一个函数，只是「重开后从哪起」这句没有专门条款。

什么会推翻：找到一条条款明确规定重开后开放段要保留或要清空，而与代码相反。


## S4：取号

**代码**：`crates/singlefs-core/src/transaction.rs:275-309`（`acquire_instance`）。
```rust
let previous_instance = identities.iter().flat_map(|identity| verified_superblock_slots(...))
    .map(|superblock| superblock.journal_instance).max().unwrap_or(MKFS_INSTANCE_GENERATION);
let highest_root = highest_root_instance(...).unwrap_or(MKFS_INSTANCE_GENERATION);
let instance = InstanceGeneration(previous_instance.max(highest_root).0 + 1);
let mut written = Vec::new();
for index in 0..pool.devices.len() {
    if let Err(cause) = pool.write_superblock_slot(index, 0, instance) { ...回卷... }
    written.push(index);
}
if let Err(cause) = pool.perform(CommitStep::Barrier) { ...回卷... }
Ok(instance)
```

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1245`「已定项16」标题所在小节，逐字见附录一 985-1023 区间之外单独一段，此处引 `.claude/kb/decisions/18-块里携带什么信息.md:882` 里点名的取号句）：
> 再取新代号 = max(**这次挂载独占打开成功的那个集合**里各超级块的代号（每块盘两槽里全部自证过的槽……）, 根环里全部根记录的实例代号) + 1 并写进**那个集合**里的每一份超级块、**全或无**……取号之后那道屏障（D23 已定项16）在任一块盘上报错同样判全或无失败，不许只重发屏障就继续。

**判定**：

1. `previous_instance.max(highest_root).0 + 1` 与「max(……超级块的代号……, 根环里全部根记录的实例代号) + 1」逐句一致——**一致**。
2. 逐盘 `write_superblock_slot` 之后失败即回卷（`transaction.rs:300-304`）与「全或无」「写进那个集合里的每一份超级块」一致——**一致**。
3. 写完之后 `CommitStep::Barrier`，失败同样回卷（`transaction.rs:305-307`）——与「取号之后那道屏障……在任一块盘上报错同样判全或无失败，不许只重发屏障就继续」一致——**一致**（代码是失败即整体回卷、返回错误，调用方若要继续只能重新走一遍 `acquire_instance`，没有「只重发屏障」这条路径，符合「不许只重发」）。
4. 代码注释自陈一处收窄（`transaction.rs:273-274`）：「D18 已定项11 的『重试到 T_retry 用尽才算失败』这里没有做：块设备报的第一个错就判失败（写路径上的重试全仓都还没有）」——这是代码自己标注的欠账，不是本腿新发现的不一致，milestone 现状段与代码注释口径一致——**如实记录，不判「不一致」**（重试机制本来就是全仓未实现的欠账，不是这一批改动引入的偏离）。

什么会推翻：找到一次挂载在健康集合不过半、或超级块代号计算用了错误的集合（比如把没独占打开的盘也算进 max）。

## S5：写行

**代码**：`crates/singlefs-core/src/mount.rs:243-283`。

关键片段（`mount.rs:247-266`）：
```rust
let first_row_instance = effective_root.instance.0.max(1);
for row_instance in first_row_instance..instance.0 {
    let row = if row_instance == effective_root.instance.0 {
        InstanceRow { instance: ..., selected_root_txg: effective_root.checkpoint_txg,
            applied_transaction_high_water: journal.maximum_applied_transaction, is_rollback: false }
    } else {
        InstanceRow { instance: ..., selected_root_txg: CheckpointTxg(0),
            applied_transaction_high_water: 0, is_rollback: false }
    };
    ...
}
```
`PublishPlan{ txg: first_txg, counter: next_counter, transaction: 0, instance, back_chain: 0,
    file: None, instance_table: InstanceTablePlan::Rewrite(...), ... }`（`mount.rs:270-281`）。
`first_txg = max(highest_ring_txg, highest_record_txg) + 1`（`mount.rs:218-232`）。
`next_counter = records 里最大 counter + 1`（`mount.rs:212-217`）。

**条款原文**（`.claude/kb/decisions/18-块里携带什么信息.md:882`，摘录关键句，整段见附录一/kb 原文）：
> **每次可写挂载都写行**……恢复在所选根指着的那一版表上给 [max(所选根的实例, 1), 新实例) 每个实例写一行——所选根那个实例写 (i, **重放之后那个根的 checkpoint_txg**, **属于实例 i 的、被这次重放施加的最大事务号**)，严格介于所选根的实例与新实例之间的实例写 (i, 0, 0)……

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1238`，已定项14 注 3）：
> **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零……**checkpoint_txg 也一样**……新实例（普通挂载、切换、回退）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1。

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md:1027-1029`，已定项19①）：
> **① 空发布记录（不改）**：事务号 0 保留给不承载事务的记录，提交标记写 1；不进实例表 W 的 max……

**判定**：

1. `first_row_instance = effective_root.instance.0.max(1)`，循环区间 `[first_row_instance, instance.0)` 与「[max(所选根的实例, 1), 新实例)」一致——**一致**。
2. `row_instance == effective_root.instance.0` 那一支：`selected_root_txg: effective_root.checkpoint_txg`、`applied_transaction_high_water: journal.maximum_applied_transaction` 与「所选根那个实例写 (i, 重放之后那个根的 checkpoint_txg, 被这次重放施加的最大事务号)」逐句一致——**一致**。
3. 其余（中间实例）那一支：`selected_root_txg: CheckpointTxg(0), applied_transaction_high_water: 0` 与「严格介于……写 (i, 0, 0)」一致——**一致**。
4. `first_txg = max(highest_ring_txg, highest_record_txg) + 1` 与「checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」一致（代码取严格 `+1`，满足 `≥`）——**一致**。
5. 写行发布的 `PublishPlan.transaction: 0`，与已定项19①「事务号 0 保留给不承载事务的记录」一致（写行发布本身不携带用户事务）——**一致**；`back_chain: 0` 与「实例切换……行只落在……」搭配 D23 已定项19②「本实例写出的第一条记录恒 0」——写行发布是新实例第一次发布，恒 0 一致——**一致**。
6. `next_counter`（jsn 计数器）取的是「`records`（`scan_journal` 扫出的全部合法记录）里最大 `counter` + 1」，而条款字面是「新实例从**前缀末** + 1 接着写」（`decisions/23-journal的角色与格式.md:1238`）。二者在健康镜像上等价（`above_water` 中未通过 `expected_next` 检查而中断的记录之后不会再有更大 jsn 的合法记录，因为定长环覆写是按 jsn 单调写入的），但字面上「全部合法记录的最大值」比「前缀末」宽一格——**未见冲突的实现细节，标记为观察项，不算不一致**：这属于「同一段历史上是否可能出现前缀外还有更大 jsn 记录」的可达性问题，落在 X1/X4 的攻方判据范围，正推腿只报告字面差异，不判定它是否可被打穿。

什么会推翻：找到一段历史，`records` 中最大 jsn 记录不在前缀内（即被 `expected_next` 检查排除），却仍参与了 `next_counter` 的计算并造成 jsn 冲突。


## S6：暖机

**代码**：`crates/singlefs-core/src/mount.rs:285-323`。
```rust
let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish.root.checkpoint_txg)];
let mut current = row_publish.clone();
let mut warm_up_publishes = Vec::new();
while all_devices.iter().any(|identity| !covered.contains(identity))
    && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
{
    let next = publish_version(&mut pool, &mut allocator, PublishPlan{
        txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
        counter: current.record.counter + 1, transaction: 0, instance,
        back_chain: back_chain_of(&current.record_bytes),
        file: None, instance_table: InstanceTablePlan::Carry(current.root.instance_table),
        ... }, Some(&current))?;
    ...
}
```
`ROOT_RING_REGIONS` 来自 `use singlefs_format::{INSTANCE_ROW_BYTES, ROOT_RING_REGIONS};`（`mount.rs:23`），即根环区域数（第一版 3）。`mount_writable` 在暖机循环结束后才 `Ok(Mounted{...})` 返回（`mount.rs:325-337`），全函数没有任何 `fn fsync` 调用：
```
$ grep -rn "fn fsync" crates/
（零命中）
```

**条款原文**（`.claude/kb/decisions/16-发布语义.md`「已定项 8」定案句）：
> **定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回、不向管理员确认回退；做法是连推空发布，第一版几何至多 R = 3 次。

**条款原文**（`.claude/kb/decisions/28-挂载期承诺量.md`「已定项 4」）：
> **形态**：ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数，每次发布按当时的树高重算……

**判定**：

1. `while ... && warm_up_publishes.len() < ROOT_RING_REGIONS`（第一版 = 3）与「至多 R = 3 次」一致——**一致**。
2. 每次暖机的 `PublishPlan` 里 `file: None`，`rewritten_roles()`（`transaction.rs`）在 `file.is_none()` 且 `instance_table` 为 `Carry` 时只推 `[AllocationTree, AccountingTree, MappingTree, TreeTable]` 四个角色——与 S5/milestone「每次空发布重写四个固定点单元」一致——**一致**。
3. `mount_writable` 全程无 `fn fsync`，且暖机循环在 `Ok(Mounted{...})` 之前完成——由于代码里没有区分「fsync 返回」这个显式动作，「不让 fsync 返回」体现为 `mount_writable` 函数本身不提前返回、暖机做完才把 `Mounted` 交给调用方——这是唯一可能的操作化，因为条款讲的是运行时语义（fsync 何时返回），代码目前还没有独立的 fsync 入口，milestone 现状段自陈「`mount_writable` 暖机做完才返回，代码里没有 fsync 接口」（`.claude/kb/milestone/02-second-txn.md:135`，正推腿核实与代码一致）——**一致，但只能在「fsync 接口尚不存在」这个前提下判**：等 fsync 独立实现之后，需要重新核实它是否真的等到 `mount_writable` 完成才调用。
4. **实测常量对照**：D16 已定项8 自陈「第一版实付 2 次，而且它是格式常量不是运行时谓词」，登记 `WARM_UP_EMPTY_PUBLISHES = 2`（`.claude/kb/decisions/16-发布语义.md`）。这是**首次**挂载暖机的常量，不是 `mount.rs` 里这条「后续挂载」暖机循环——`WARM_UP_EMPTY_PUBLISHES` 用在 `transaction.rs` 的 `warm_up` 函数（`transaction.rs:329`，首次挂载专用），`mount.rs` 的暖机循环走的是不同的代码路径（现算，非常量）。milestone 步3 现状段与「会碰到的决策点」栏都明确写着「后续挂载与回退之后的暖机次数（D16 已定项8 只定了第一次可写挂载，2026-09-16 三方第一轮正推腿）」——即 D16 已定项8 的常量 2 不覆盖 `mount.rs` 这条循环，`mount.rs` 用现算次数（本条脚本上是 2 次：txg 6/7 分别落盘 0/1，`.claude/kb/milestone/02-second-txn.md:135`）——**这是一处「条款没说」，milestone 已如实标出，不重复列表**。

什么会推翻：暖机循环在覆盖两块盘之前就把 `Mounted` 交出去（提前返回），或者次数上限不是 3。

## S7：发布 C

**代码**：`crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs:222-293`（验收用例逐字钉住），底层调用与 S1（步1覆盖写）同一条 `publish_overwrite`（`transaction.rs:840-880`）。

关键断言（`second_transaction_step_three_second_instance.rs:226-276`）：
```rust
(InstanceGeneration(2), CheckpointTxg(8))   // third_publish.root
third_publish.record.counter == 8;
third_publish.rewritten == TransactionUnit::IN_BUMP_ORDER;   // 八个角色全重写
SlotNumber(50184)   // 数据单元落点
device_map.allocated_slots() == 47; deferred_slots() == 34;
```

**条款原文**（`.claude/kb/decisions/23-journal的角色与格式.md`已定项7 的正文，事务号按实例计数从 1 起）：见 S1 条款引文之外补一句（`decisions/23-journal的角色与格式.md`已定项19①逐字见 S5）；已定项19②「本实例写出的第一条记录恒 0」不适用于此（这不是本实例第一条）。

**条款原文**（`.claude/kb/decisions/03-空间分配.md`已定项7，见 S3 引文）：分配记录的释放走「已释放 + 释放代」改写，不删条目。

**判定**：

1. `publish_overwrite` 复用与步1同一条函数（`transaction.rs:840-880`），txg/counter/transaction 各在上一版基础上 +1（`transaction.rs:861-864`）——与 D23 已定项7「事务号按实例计数从 1 起」（本实例第二次发布故为 1）一致，用例断言 `third_publish.record.counter == 8` 是 jsn 而非事务号，事务号另有断言（milestone 步3 现状段写「事务号 1」）——**一致**。
2. 释放走 `placements_to_release_via_mapping(previous_version, allocator, &rewritten)`（`transaction.rs:911`，S7 复用 S2/D19 已定项5 已核实的路径）——**一致**。
3. `third_publish.rewritten == TransactionUnit::IN_BUMP_ORDER`（八个文件/固定点角色，不含实例表——因为这次没有写行，`instance_table: InstanceTablePlan::Carry`）——与「六个提交内生块 + 数据 + extent」这类覆盖写的角色集一致，S1（步1）已核实同一形状——**一致**。

什么会推翻：`allocated_slots`/`deferred_slots` 的验收数与人工按落点表重算的数对不上（这是可核的算术，本腿信任验收用例已通过 `cargo test` 的事实，未重新手推 47/34 这两个数——**复核不了**：需要重新枚举 mkfs+A+B+写行+暖机×2+C 全部落点才能验证 47/34，超出 45 分钟窗口，留给主 agent 或攻方腿）。


## S8：checker I-3.8

**代码**：`crates/singlefs-checker/src/walk.rs:405-441`（`judge_instance_table_rows`，`git diff` 显示的新函数）与调用处 `walk.rs:209-217`。
```rust
fn judge_instance_table_rows(&mut self, records: &[Vec<u8>], mount_root_instance: u32) {
    let mut instances: Vec<u32> = Vec::new();
    let mut unique = true;
    let mut below_mount_root = true;
    let mut chain_record_last = false;
    for (index, row) in records.iter().enumerate() {
        match row.first().copied() {
            Some(0) => {
                let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
                if instances.contains(&instance) { unique = false; }
                instances.push(instance);
                if instance >= mount_root_instance { below_mount_root = false; }
                chain_record_last = false;
            }
            Some(1) => chain_record_last = index + 1 == records.len(),
            _ => chain_record_last = false,
        }
    }
    self.judgements.judge("I-3.8", unique && below_mount_root && chain_record_last, ...);
}
```
调用处只在 `is_newest`（最新根）时判（`walk.rs:212-217`）。`IMPLEMENTED_INVARIANTS` 从 23 条加到 24 条、插入 `"I-3.8"`（`image.rs`，见附录二 diff）。

**条款原文**（`.claude/kb/invariants.md:127`，I-3.8 行）：
> 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113 定案 P2）

**判定**：

1. `unique`（`instances.contains(&instance)` 判重）与「按实例代号唯一」一致——**一致**。
2. `below_mount_root`（`instance >= mount_root_instance` 判否）与「行的实例代号 < 挂载根实例」一致——**一致**。
3. `chain_record_last`（`kind==1` 的记录必须是最后一条）与「链指针记录（`kind` 1）恒为一片的最后一条」（D18 已定项11，S1/S5 条款域）一致——**一致**；这一句不在 I-3.8 定义本身里，是从 D18 已定项11 借来的额外判据，checker 状态列自陈「链指针记录在末尾」，与 I-3.8 条款正文不完全同源但状态列已如实登记，非隐藏行为。
4. `judge_instance_table_rows` 只在 `is_newest`（最新根）调用（`walk.rs:213`）——与 I-3.8 状态列自己写的「回收那一半没有输入，第一版行只增不删，删行的条件没人判」一致——**一致，且不变量自己声明了这一半没做**，不是代码擅自减少覆盖。
5. `mount_root_instance` 取自 `record[24..28]`（`walk.rs:214`，"实例表单元指针" 所在记录的偏移 24），与 S9 里 `crash.rs` 读 `newest_persisted_root` 用的偏移 24（instance 字段）同一口径——**一致**（同一根记录布局，两处各自读值，未见不一致）。

什么会推翻：一个合法的中间状态（新实例的根已持久、实例表单元没持久，或反过来）被 `judge_instance_table_rows` 误判红（这是 X7 的判据范围，正推腿不判打中与否，只核代码逻辑与条款文字是否一致）。

## S9：层 0 到 C（oracle）

**代码**：`crates/singlefs-harness/src/crash.rs:559-629`。

```rust
pub fn oracle_violation_for_versions(outcome, effective_root, newest_persisted_root, versions) {
    let Some((effective_instance, effective_txg)) = effective_root else { return Some(...) };
    if let Some((newest_txg, newest_instance)) = newest_persisted_root {
        if (effective_txg, effective_instance) < (newest_txg, newest_instance) { return Some(...) }
    }
    let version = versions.iter().find(|v| v.checkpoint_txg == effective_txg && v.instance == effective_instance);
    match (outcome, version) { ... }
}
pub fn newest_persisted_root(writes, persisted) -> Option<(CheckpointTxg, InstanceGeneration)> {
    publishes_in(writes).iter().filter(|p| persisted[p.root])
        .map(|p| (CheckpointTxg(p.checkpoint_txg), InstanceGeneration(p.instance))).max()
}
```
偏移读取：`crash.rs:640`（`write.bytes[24..28]` → instance）、`crash.rs:645`（`write.bytes[28..36]` → txg）。

**条款原文**（`.claude/kb/decisions/13-验证路线.md`已定项4，见附录一，S9/S11/X11 条款域）：
> 崩溃点重放要枚举的崩溃状态集合定义为：屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」……

**条款原文**（`.claude/kb/experiments/77-发布的持久顺序.md:8-14`「判据」，判据1）：
> 违例 = 根槽已持久而恢复 ≠ 新态或走读失败；或恢复出部分事务；或嫁接了校验失败的单元。根槽未持久时新旧两态都合法（fsync 没返回）。

**判定**：

1. `(effective_txg, effective_instance) < (newest_txg, newest_instance)` 判「根槽已持久而恢复到旧态」——与 E77 判据1「根槽已持久而恢复 ≠ 新态」及 D22 已定项7「择新 txg 为主、实例代号破平局」的字典序一致——**一致**，代码注释（`crash.rs:550-551`）自陈「只按 txg 认时 oracle 在那一格两个方向都错（三方代码第一轮攻方腿）」，S9 的 (txg, 实例) 二元键正是修复后的形态。
2. `newest_persisted_root` 读 `bytes[24..28]`（instance）与 `bytes[28..36]`（txg）——这是根槽字节布局里的偏移，milestone/S9 声称的「偏移 24 与 28」逐字对应，属于根记录/根槽序列化格式的产物，非条款文字直接给出的数字（根记录字段表在 D22 已定项7 里定义，偏移取决于字段排布顺序），本腿未逐字节重算根槽序列化布局来验证 24/28 是否与 `root_record.rs` 的 `to_slot` 实现完全对应——**复核不了**：需要读 `root_record.rs` 的 `to_slot`/`parse_slot` 实现并逐字段核偏移，超出本次任务分配的文件清单（root_record.rs 不在派发提示要求读的文件列表内），留给主 agent 或后续轮核实。
3. Ignore 策略另计 `ignored_violations`（`crash.rs:734`，`tally.ignored_violations += 1`），milestone/S9 声称「不看 journal 那一遍恢复也过同一个 oracle（另计 ignored_violations）」——与 D13 已定项4「一次整写」的枚举口径无直接冲突（Ignore 是否过 oracle 是运行时行为，不是枚举定义本身）——**一致**。

## S10：E142 装置同步

见下方 X9 一节的逐行对比；结论：**同一条规则，一致**。单测钉 6（`e142_first_transaction_dry_run.rs:3706`：`assert_eq!(tally.verification_ran_states, 6, ...)`），变异表新增 M67（`research/mutations/e142_first_transaction_dry_run.tsv` 第 67 行，`M67_apply_records_across_instance_boundary`，把 `record.instance == root.instance &&` 这半句去掉）——与 crates 侧那条同一形状的变异（`crates/mutations.tsv`「步 3：前缀跨实例边界（把别的实例的记录也接上）」）逐字同构——**一致**。


## S11：第二轮攻方腿打中的七处改法（被攻过零轮，本腿只核落地，不判打没打中）

**代码位置**（`crates/singlefs-core/src/transaction.rs`）：

| 改法 | 代码 | 行为 |
|---|---|---|
| 三种释放错在动分配器之前报 | `transaction.rs:578-579, 613, 619, 626` | `ReleaseNotInMapping` / `ReleaseTargetNotAllocated` / `ReleaseTargetAlreadyReleased` / `ReleaseSpanMismatch` 四个枚举成员，`placements_to_release_via_mapping` 在 `publish_version`（`transaction.rs:911`）里于分配之前调用、`?` 直接返回 |
| 跨度取分配记录里的 | `placements_to_release_via_mapping` 函数体（`transaction.rs:580` 起）按 `record` 的 `span_slots` 取跨度，不按映射条目 | 与 D3 已定项7「跨度取记录里的（映射给槽，记录给跨度）」一致 |
| 失败的发布把分配器换回 | `publish_version`（`transaction.rs:928-936`）：`allocator_before_this_publish = allocator.clone()`；`if outcome.is_err() { *allocator = allocator_before_this_publish; }` | — |
| 内容超长报 `ContentExceedsDataUnit` | `transaction.rs:906-912`（`if let Some(file) = &plan.file { if file.content.len() > data_unit_capacity { return Err(...) } }`） | 在分配之前检查 |
| oracle 补「更新的根下没版本却报没有文件」 | `crash.rs:585-600`（S9 已引） | — |
| Ignore 那一遍恢复也过 oracle | `crash.rs:734`（`ignored_violations`） | — |
| 同盘槽号唯一 | `recovery.rs:759-770`（`allocation_records_are_one_per_device`，见下方代码） | 逐盘核 `slots_per_device` |

**判定**：以上七处在源码里均能找到对应实现，与 milestone 步2/步3 现状段的描述（`.claude/kb/milestone/02-second-txn.md:111`）逐句对得上——**一致**。本腿不判「这些改法有没有被新一轮攻方打中」（X11 是攻方腿的格，见分工表），只核实它们确实落地成代码。

## X9：E142 装置与 crates 的恢复规则是不是同一条

**crates**（`crates/singlefs-core/src/recovery.rs:618-637`）：
```rust
let mut above: Vec<&JournalRecord> = records.values()
    .filter(|record| record.instance == root.instance && (record.instance, record.checkpoint_txg) > water)
    .collect();
above.sort_by_key(|record| (record.instance, record.counter));
...
let root_own_record_counter = records.values()
    .find(|record| record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg)
    .map(|record| record.counter);
let mut expected_next: Option<(InstanceGeneration, u64)> =
    root_own_record_counter.map(|counter| (root.instance, counter + 1));
```

**E142 装置**（`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:2636-2652`）：
```rust
let water = (root.instance, root.checkpoint_txg);
let mut above: Vec<&JournalRecord> = records.values()
    .filter(|record| record.instance == root.instance && (record.instance, record.checkpoint_txg) > water)
    .collect();
above.sort_by_key(|record| (record.instance, record.counter));
let root_own_record_counter = records.values()
    .find(|record| record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg)
    .map(|record| record.counter.0);
let mut expected: Option<(InstanceGeneration, JournalCounter)> =
    root_own_record_counter.map(|counter| (root.instance, JournalCounter(counter + 1)));
```

**逐分支并排**：

| 分支 | crates | E142 装置 | 是否相同 |
|---|---|---|---|
| 候选过滤 | `record.instance == root.instance && (instance, txg) > water` | 逐字相同（类型 `JournalCounter` 包装差异不影响逻辑） | **相同** |
| 排序键 | `(instance, counter)` | `(instance, counter)` | **相同** |
| 链首取法 | `root_own_record_counter`（同实例同 txg 的记录，取其 `counter`） | 同上 | **相同** |
| 链首读不出（候选里有别的实例的记录） | `expected_next = None`；候选本身已经被过滤成只含 `root.instance`，别的实例的记录不会出现在 `above` 里，也不会被当作候选 | 同构，`above` 同样先过滤 `record.instance == root.instance` | **相同**——两边都不存在「候选里有别的实例的记录」这一支：过滤条件本身就把跨实例记录挡在候选集之外 |
| jsn 断号即止 | `if (record.instance, record.counter) != expected_key { break; }`（`recovery.rs:639-642`） | `if (record.instance, record.counter) != expected_key { break; }`（`e142...rs:2647-2650`） | **相同** |
| jsn 断号在第一条 | `expected_next` 为 `root_own_record_counter.map(...)`；若 `root_own_record_counter` 为 `Some`，第一条候选必须恰好是 `counter+1` 才不断——即断号可以发生在第一条（这正是用例 `one_missing_record_right_after_the_chosen_root_stops_the_prefix...` 覆盖的场景，`second_transaction_step_three_second_instance.rs:469-500`） | 同构：`expected` 同样在第一条就可能不匹配而 `break` | **相同** |

**判定**：X9 判「同一条规则」——两份代码在候选过滤、链首取法、断号即止三个分支上逐行同构，未发现任何一处两边处理不同的输入。E142 装置**没有**实现 `maximum_applied_transaction` 字段（`JournalScanReport` 结构体在装置里只有 5 个字段，`e142...rs:2524-2530`，缺 crates 侧新增的第 6 个字段），但这不属于「恢复规则」本身的分歧——装置不做写行（S5/S6 那部分逻辑装置里没有对应实现），装置的 `verification_ran` 计数只关心 9→6 这个变化点，与写行无关。什么现象会推翻这一条：找到一个镜像/记录集合，两边代码在候选集、链首、断号点上给出不同的施加集合——本腿逐行比对未能构造出这样的输入（两份代码字符级几乎相同，只有类型包装的差异）。


## X10：kb 写回与代码对不对得上

### `.claude/kb/milestone/02-second-txn.md` 步 0「现状」段（第 58 行）

逐字引（节选，全段见文件第 58 行）：
> 固定脚本做到发布 C：`crates/singlefs-harness` 的 `crash` 模块按整条录制流切段枚举、每个崩溃状态按「最新持久根 (txg, 实例) 对应哪个版本」判 oracle（`enumerate_layer0_versions`）……八个根槽写里后四个是实例 2 的（txg 5–8），实例那一维走到了：oracle 认 (2, 5)、(2, 6)、(2, 7) 三个根都是第二次的内容（写行与暖机不换文件版本）、(2, 8) 是第三次的……

**核对**：`crash.rs` 里确有按 `(txg, 实例)` 二元判 oracle 的函数（`oracle_violation_for_versions`，见 S9），写行/暖机的 `PublishPlan.file` 均为 `None`（不换文件版本，见 S5/S6），第三次内容对应发布 C（`publish_overwrite`，见 S7）。用例 `remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version`（`second_transaction_step_three_second_instance.rs:95`）名字与断言（`row.data_pointer == second.data_pointer` 等，`second_transaction_step_three_second_instance.rs:147`）与「写行与暖机不换文件版本」一致——**符合**。

### `.claude/kb/milestone/02-second-txn.md` 步 3「现状」段（第 135 行）

逐字引（节选）：
> `crates/singlefs-core/src/mount.rs` 的 `mount_writable`：先走一遍恢复（`recover`，前缀规则只收所选根自己那个实例的记录、链首接在所选根自己那条记录之后），从盘上重建上一版（`recovery::rebuild_version`：实例表、树表、五棵树的根节点、数据单元、inode 叶；树表空就报 `NoPublishedVersion`……），分配器从分配记录重建（`PoolAllocator::rebuild_from_records`：已分配标位图、已释放进 defer 队列、开放段不续、第一次分配从空闲位图重找）……

**核对**：`mount.rs` 函数名 `mount_writable`、调用链 `choose_superblock → choose_root → scan_journal → replay_journal → rebuild_version → PoolAllocator::rebuild_from_records → acquire_instance → publish_version（写行）→ publish_version（暖机循环）`（`mount.rs:169-338`）与描述逐句对应；「前缀规则只收所选根自己那个实例的记录、链首接在所选根自己那条记录之后」与 S1 核实的 `recovery.rs:618-637` 一致；「已分配标位图、已释放进 defer 队列、开放段不续、第一次分配从空闲位图重找」与 S3 核实的 `allocator.rs:284-308` 一致——**符合**。

### `.claude/kb/layout/01-first-txn.md` 八，两行「装置钉住」与「第二条流」句（第 395、396、401 行）

第 395 行（「第一次之后的可写挂载（写行）」行）逐字引（节选）：
> **装置钉住**（里程碑「第二个事务」步 3 2026-09-16 落地，孤立形状从第二条流推得）：[取号超级块槽 × 2 盘，世代号 3] 屏障 [实例表单元（写行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [超级块槽 × 2 盘] ⇒ 孤立看 `2+10+2+1+2`……取号之后那道屏障由取号自己发（D23 已定项16：写行那次发布有单元写，等不到空发布开头那道）；每次可写挂载都写行（实例 0 不写……）

**核对**：`5 单元`（实例表 + 分配记录 + 记账 + 映射 + 树表）与 S5 核实的 `rewritten_roles()`（`file` 为 `None`、`instance_table` 为 `Rewrite` 时的 5 个角色：`InstanceTable, AllocationTree, AccountingTree, MappingTree, TreeTable`）一致；「取号之后那道屏障由取号自己发」与 S4 核实的 `acquire_instance` 自己 `CommitStep::Barrier`（`transaction.rs:305`）一致；「每次可写挂载都写行（实例 0 不写）」与 S5 核实的 `first_row_instance = effective_root.instance.0.max(1)` 一致——**符合**。该行末尾括注「2026-09-17 按代码改写」——文件本身已标注这是按代码回填的，非本腿新发现的偏差。

第 396 行（「空发布（暖机，后续可写挂载的实例……）」行）逐字引（节选）：
> **装置钉住**……[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [超级块槽 × 2 盘] ⇒ 孤立看 `8+2+1+2`，与首次挂载的暖机不同型……这条脚本上 c_max = 4；……实例 2 从 txg 5 起推 2 次（txg 6 落盘 0、txg 7 落盘 1），次数按「本实例的根覆盖全部区域盘」现算、上限 3

**核对**：「4 单元」与 S6 核实的暖机循环 `rewritten_roles()`（`file: None`、`instance_table: Carry` 时 4 个角色）一致；「c_max = 4」需要重算树高，本腿**复核不了**（需重新枚举分配记录树、映射树、记账树在这条脚本上的层数，超出本轮时间预算，milestone 步3「会碰到的决策点」也未把它列为待查项，视为已由 E148 与实测覆盖，本腿信任但未重算）；「txg 6 落盘 0、txg 7 落盘 1，上限 3」与 S6 核实的 `while ... && warm_up_publishes.len() < ROOT_RING_REGIONS`（3）一致——**符合**。

第 401 行（「第二条流」句）逐字引：
> 第二条流（里程碑「第二个事务」步 0 的固定脚本，2026-09-16 做到发布 C）：取号 → 暖机 × 2 → A → B → 进程退出、重开取号 → 写行 → 暖机 × 2 → C，第二条流的段序列 `2+2+1+2+2+1+18+2+1+18+2+1+4+10+2+1+10+2+1+10+2+1+18+2+1+2`、118 次写、789555 个状态，没有干跑产物，装置钉住：`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 把这个数组与闭式钉死……

**核对**：该数组与 118 次写、789555 个状态是否与 `second_transaction_step_zero_layer0.rs` 的用例逐字相符——**复核不了（部分）**：本腿未展开该用例的完整数组断言与闭式重算（这是一条极长的段序列，逐项核对需要读完整个 `crash` 模块的段切分实现，超出本轮 45 分钟窗口与派发提示指定的文件清单），milestone 现状段（第 58 行）自陈「789555 个状态在门禁 54 号第二条流上全量零违例」，且门禁 54 号是本轮之外的独立验证（release 下跑），本腿只确认段序列的**结构**（写行 5 单元、暖机 4 单元）与代码一致，未逐字重算 789555 这个闭式数字。

### `.claude/kb/invariants.md` I-3.8 状态列（第 127 行）

逐字引：
> 已实现（2026-09-16，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例、链指针记录在末尾；回收那一半没有输入，第一版行只增不删，删行的条件没人判）

**核对**：函数名、三条判据（唯一/低于最新根实例/链指针在末尾）与 S8 核实的 `judge_instance_table_rows`（`walk.rs:405-441`）逐句一致；「回收那一半没有输入，第一版行只增不删」——全仓搜索：
```
$ grep -rn "fn.*recycle\|fn.*reclaim.*instance_row\|instance.*行.*删" crates/singlefs-core/src/ crates/singlefs-checker/src/
（零命中，除 mount.rs 里只 push 不 remove 的 rows_written/table_after.rows）
```
`mount.rs` 里 `table_after.rows.push(row)`（`mount.rs:265`）只增不删，全仓没有删除实例表行的代码——与「行只增不删」一致——**符合**。

**小结**：X10 四处 kb 写回全部与代码符合，两处（c_max=4 的树高重算、789555 状态数的段序列逐字重算）**本腿复核不了**，标注理由如上，不代表发现不一致。


## 条款没说的取法（实做时定下、标预想交用户）

| 格 | 取法 | 代码位置 | 全仓搜索结果 |
|---|---|---|---|
| S1 | 所选根自己那条记录读不出时，链首从水位之上同实例最小的一条接 | `recovery.rs:628-637`（`root_own_record_counter` 为 `None` 时 `expected_next` 为 `None`） | `grep -n "读不出.*接\|从水位之上同实例最小" .claude/kb/decisions/23-journal的角色与格式.md` 零命中 |
| S3 | 重开后开放段不续；重开后第一次分配从空闲位图重找最低空槽对 | `allocator.rs:275-282`（`Self::new` 里 `open_segment: None`） | milestone `.claude/kb/milestone/02-second-txn.md:135` 自陈「预想里没定、这一步实做时定下的」，D3 已定项5/已定项8 未涉及重开这一刻 |
| S2/S6 | 树表为空拒开挂载（`NoPublishedVersion`）——第一版只支持至少发过一版文件的池 | `recovery.rs:351-355`、`mount.rs:196-200` | D16 已定项8 只讲暖机因果，未讲树表为空时该如何处置；milestone 现状段已如实陈述 |
| S6 | 后续挂载暖机次数按「本实例的根覆盖全部区域盘」现算、不是常量 2 | `mount.rs:285-323`（`while` 循环条件） | D16 已定项8 的 `WARM_UP_EMPTY_PUBLISHES = 2` 只覆盖首次挂载；milestone「会碰到的决策点」栏已列出 |

## 这条腿自己的限度

1. **S7 的 47/34（`allocated_slots`/`deferred_slots`）没有重新手推**，只确认用例已存在并断言这两个数；要坐实需要重新枚举 mkfs + A + B + 写行 + 暖机×2 + C 全部落点，超出本轮时间预算。
2. **S9 的偏移 24/28 是否与 `root_record.rs` 的序列化实现完全对应，没有核实**——`root_record.rs` 不在派发提示要求读的文件清单内，本腿只核了 `crash.rs` 里两处读偏移的调用点自洽（S8/S9 互相印证），未逐字段重算根槽布局。
3. **X10 的 c_max=4（树高）与 789555（段序列闭式）两个数没有重算**——都是需要展开完整装置/用例逻辑才能验证的算术，标注为「复核不了」而非「符合」。
4. **不判攻方腿的格**：X1、X2、X4、X6、X8（Opus）与 X3、X5、X7、X11（本地攻方）一个字都没写，按分工不越界。
5. **S5 的 jsn 计数器取「全部合法记录最大值」而非字面「前缀末」这一处细微差异**，本腿只报告字面差异存在，未构造可达性场景去判断它是否可被打穿——这是攻方腿（X1/X4）的判据范围。
6. **没有跑 `cargo test`/`gate.sh`**：所有「用例存在且断言如此」的判断，都基于读源码确认测试函数体与断言逐字存在，未在本机重新编译执行（原仓只读、按定义不许编译，见分到的规矩）；milestone 现状段自陈「步 3 五条验收用例绿」「门禁 59 号 25 条变异各红」，本腿信任这句陈述、未重新验证退出码。
7. **没有读 `crates/singlefs-checker/src/image.rs` 全文**，只读了 diff 里改动的那几行（`IMPLEMENTED_INVARIANTS` 数组），未核对该文件里 24 条不变量的完整判定逻辑是否互相一致。

