# checker 逐行审阅报告（crates/singlefs-checker）

口径：只读审阅，没跑 cargo / 脚本 / 门禁。行号取自 2026-09-26 14:32 UTC 的工作区快照（walk.rs sha256 `c3add14d…c47c`，5143 行）。
⚠️ 审阅中途 walk.rs 被别的会话改过一次（mtime 14:28:42 UTC，5118 → 5143 行）：只是在 2951–2975 行插入了一个 `InstanceTableOfRootRecord`（给 harness 用），其后的行整体 +25，逻辑没变；下面引用的行号都是改后的。lib.rs / image.rs / position_addressed.rs 在审阅期间没动。
checker 不从 singlefs-core 引任何东西：`Cargo.toml` 只依赖 `singlefs-format`，src 里 `singlefs_core` 只出现在 lib.rs:126 的一行注释里（已核）。
`IMPLEMENTED_INVARIANTS`（image.rs:37-43）与 kb 里标「已实现」的 45 条逐个相同（排序后 diff 为空）。

## 发现（按严重度排序）

### 1. `visited_units` 把「同一版本里的第二次引用」也当成「别的根走过」跳过：交叉链接与树内 DAG 完全判不出，I-5.1 对同起点同跨度的重叠是瞎的
- 位置：walk.rs:398-406、505-510、673-688、741、988-993、1261-1267、4983-5000；image.rs 无
- 严重度：高
- 置信度：已确认（读代码可复现推理）
- 代码做了什么：
  ```rust
  self.references.entry((location.device, location.slot, span)).or_insert_with(|| what.to_string());   // 398-401
  if !self.visited_units.insert((pointer.locations[0].device, pointer.locations[0].slot)) { return None; }  // 505-510
  if !self.visited_units.insert((pointer.locations[0].device, pointer.locations[0].slot))
      || !self.judge_unit_header(&unit, 1, "数据单元") { return; }                                       // 1261-1265
  judgements.judge("I-5.1", first_slot + first_span <= second_slot, ...)                                  // 4996
  ```
- 为什么错：`visited_units` 是整个 `Walk` 共用的一个集合，不区分「别的根已经走过」与「同一个根里第二条指针又指到它」。第二次引用时 I-2.1 / I-2.5 照判，但 I-1.1（五元组对 extent key、位置寻址树的「节点罩的就是它的位置」、多层树 ② ③）、I-1.2 / I-4.2（头对这条指针）、I-1.3、I-9.10 全部跳过；`SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot`（150-151）的名字与文档写的是「别的根」，同一根内也返回它。另一方面 `references` 以 (盘, 起点槽, 跨度) 为键去重，两条不同引用落在同一起点、同一跨度（固定大小单元最常见的重复分配形态）只剩一项，I-5.1 的相邻比较看不到它们。
- 失败场景：最新根里 inode 100 与 inode 101 的 extent 记录带着逐字节相同的数据指针（同一个数据单元 U）。第一次走 U：五元组 (100, 0) 判过；第二次：I-2.1 过（校验和就是 U 的）、`visited_units` 命中直接 return，五元组 (101, …) 从不比；`references` 只有一项，I-5.1 无对；I-3.1 两边都按一份算，对得上；I-9.10 只登记了第一份。结果：45 条全绿。同形的还有：分配记录树两条内部条目（两个不同格）指同一个孩子——第二个格从不核「孩子罩的是不是我这一格」；记账树 / 映射树里两条父条目指同一个孩子——`AlreadyWalkedFromAnotherRoot`、② ③ 跳过。
- 应该做什么：按根记「这一遍已走过」，第二次在同一根内碰到同一单元就判红（I-5.1 / I-1.1）；跨根共享时也要带回孩子头里的层级与 key 区间，让新父条目的 ② ③ 照判；I-5.1 的键里要么加上位置条目的校验和（同一落点两个不同校验和 = 两个不同单元），要么把同一版本内的重复引用单独判。

### 2. 多层码 2 树按节点头里自述的 key 宽切条目：一个重封过头校验和的节点就让 checker panic
- 位置：walk.rs:718、723-724、753、761（对照 526-539）
- 严重度：高
- 置信度：已确认
- 代码做了什么：
  ```rust
  let key_width = view.key_width;                                                    // 718
  let separator_key = entry[..key_width].to_vec();                                   // 723
  let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];     // 724
  fields(separator_key) <= fields(smallest_key),                                     // 753
  ```
- 为什么错：`read_index_node`（526-539）对「头里 key 宽 ≠ 这棵树的 key 形态」只 `judge("I-1.1", …)`，照样交回 `Some(view)`；`walk_code_two_subtree` 接着用 `view.key_width` 切条目、用 `tree.schema.fields` 解 key。条目宽的守卫只保证 `entry_width == 108`（记账）或 `≥ 113`（映射），不保证 `key_width + 86 ≤ entry_width`，也不保证孩子的 key 至少有 schema 宽。别的几处走读都按 schema 宽切并先核 `node.key_width == key_width`（note_every_node_below:2141、allocation_record_tree_without_judging:2189、extent_tree_without_judging:2255），这里没有。
- 失败场景：① 记账树一个内部节点把偏移 51 的 key 宽从 22 改成 50、重封头校验和与位置条目校验和：`entry[50..136]` 在 108 字节的条目上越界 → panic；② 记账树一片叶把 key 宽改成 10（条目宽仍 34）：叶本身走得过，回到父节点做 ② 时 `fields(smallest_key)` 在 10 字节的 key 上读偏移 10 的 u32 → panic。checker panic 等于这份镜像上 45 条都没判（C476 那一族的输入正是「盘上内容可控」）。
- 应该做什么：key 宽不等于 schema 宽时记走读失败、这一支不往下走（与 1434 / 1543 / 1629 那几处用 schema 宽的写法一致），`fields` 只在长度核过的 key 上调。

### 3. I-8.6 在计数器为 0 的记录上做 `counter - 1`：u64 下溢 panic
- 位置：walk.rs:4055-4064（对照 4397-4400）
- 严重度：高
- 置信度：已确认
- 代码做了什么：
  ```rust
  let expected_back_chain = if *counter == FIRST_JOURNAL_COUNTER {
      Some(0)
  } else {
      match records.get(&(counter - 1)) {
  ```
- 为什么错：`check_journal_record`（lib.rs:627-701）不拒计数器 0，`scanned_journal_records_of_device` 把它原样收进 `records`。计数器 0 走进 else 支，`0 - 1` 在 release（`overflow-checks = true`，Cargo.toml:17）下 panic。同文件 I-8.9 那一处写的是 `first_counter.checked_sub(1)`（4398-4399），两处口径不一。
- 失败场景：写路径把第一条记录的 jsn 计数器编成 0（从 0 起而不是从 1 起的差一错误，正是 checker 该抓的那一类），记录头校验和照算、自证过 → checker 在 I-8.6 处 panic，整份报告出不来。
- 应该做什么：计数器 0 单独成一臂（判红：计数器从 1 起，D23 已定项 9 / 注 3），或用 `checked_sub`。

### 4. 盘上字段参与的算术：I-5.2 的加法溢出、容量的减法下溢、几何偏移的乘法溢出，都会 panic
- 位置：walk.rs:5076-5080（对照 5084、5101）；image.rs:242-244；walk.rs:1756-1757、3988
- 严重度：中
- 置信度：已确认（算术路径）；可达性要一份「校验和对、数不对」的镜像
- 代码做了什么：
  ```rust
  .map(|bytes| (bytes / SLOT_BYTES - geometry.unit_area_start_slot) * SLOT_BYTES);                 // 5078
  judgements.judge("I-5.2", matches!(..., (Some(free), Some(allocated), Some(capacity)) if free + allocated == capacity), ...  // 5080
  let offset = geometry.base_slot * SLOT_BYTES + region * geometry.prime_step * geometry.chunk_bytes + slot * geometry.slot_spacing;  // image.rs:242-244
  let offset = geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;   // walk.rs:3988（1757 同式）
  ```
- 为什么错：`free`、`allocated` 是记账叶里读来的 u64；`unit_area_start_slot`、`base_slot`、`journal_ring_start_slot` 是系统配置里读来的 u64。release 开着溢出检查，这几处直接 panic。同一段里 I-3.11 特意写成 `deferred.checked_add(...)`，注释（5084「盘上读来的值大到溢出也判红、不回绕」，判定在 5101）说的正是这件事，I-5.2 却没照做。
- 失败场景：记账里「空闲」被写成接近 u64::MAX（写路径下溢回绕之类），头与载荷校验和都对 → `free + allocated` panic；系统配置单元区起点 > 盘上槽数 → 减法 panic；根环基址 ≥ 2^50 → `root_slot_positions` panic，连 I-7.1 都报不出来。
- 应该做什么：一律 `checked_*`，溢出按违例或「几何不可读」判，不 panic。

### 5. I-7.4 只判回退候选集，被抛弃时间线上还在根环里的根引用的块从不判「未被复用 / 未被抹头」
- 位置：walk.rs:4780-4798、4800-4822（候选集之外的根只在 3339-3355 的收位置项走读里走，判定丢掉）
- 严重度：中
- 置信度：已确认（代码）；条款读法见下
- 代码做了什么：
  ```rust
  let walked = *index == newest_index || (!abandoned && !below_floor);
  ```
  I-7.4 / I-4.8 只对 `candidate_indexes` 里的根与由记录施加出来的版本各判一格。
- 为什么错：kb I-7.4 那一行除候选集外还写着：被抛弃时间线的根「引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314 取影子账……）」。checker 对这部分根一格都不判；它们的单元又不在 I-3.1 的并集里（影子账隔离），于是「复用了被抛弃根引用的单元」这一类没有任何一条会红。
- 失败场景：崩溃恢复落到较旧的根，被抛弃实例 i 的根 (i, T > Ti) 还在环里；新实例把它引用的一个槽重新分配、写进新单元。旧根指针的校验和对不上，但这条根不在候选集里，I-2.1 / I-7.4 都不判 → 全绿（正是 C314 第一轮判决 H6 那一格）。
- 应该做什么：被抛弃、仍在环里的根按 I-7.4 单独判一格（只判它引用的块读回来对不对，不并进 I-3.1）。

### 6. 中央映射树的条目只核校验和与升序：映射 key 与被指单元的身份从不比，也不与树里的指针对账
- 位置：walk.rs:617-641；1994-1995（注释）
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  let length = if entry[0] == 2 { node_bytes() } else { data_unit_bytes() };
  if read_referenced_unit(self.reader, &mut self.judgements, &view.locations, length, "映射条目指的单元").is_none() { ... }
  ```
- 为什么错：映射条目本身是一条引用（key = 类标签 + 出生树 + 出生 txg + 实例 + 尾段，D19 已定项 11 / 12）。checker 不比 key 与被指单元头里的类标签、出生身份（I-1.2 遍历方向那一半判的正是「头与引用它的指针相同」、I-1.6 的类标签），不判被指单元的头（I-2.4 / I-1.4），也不 `note_reference`。1994-1995 的注释「映射条目指的单元与树里引用的是同一批，不重复数」是一句没有任何检查撑着的断言，I-3.1 / I-5.1 不数映射条目就建在它上面。类标签不是 1 / 2 / 3 时（`entry[0]` 为 0、4…255）也照按 32 KiB 读，不红。
- 失败场景：映射条目的位置条目指向另一个合法单元 V（带 V 的整单元校验和）——例如搬迁后没更新、或写路径把两个单元的映射写串了。I-2.1 过、I-2.5 过，别的一条都不看；按映射读的读者拿到的是 V。
- 应该做什么：映射条目逐条读被指单元的头，比类标签与出生身份；再与遍历里同一个单元的位置条目对账（对不上的就是一条独立引用，进 I-5.1）。

### 7. I-2.5 的「journal 点名项里的位置条目」一半没实现
- 位置：lib.rs:662-683（解出来）；walk.rs:3953-3985（`ScannedJournalRecord` 不带 `named`）
- 严重度：中
- 置信度：已确认（`grep '\.named'` 在 checker 源码里零命中）
- 代码做了什么：`check_journal_record` 把每个点名项的两条位置条目解进 `NamedEntryView.locations`，walk.rs 扫环时只取了实例、事务号、标志等几个字段，`named` 被丢掉，没有一处对它调 I-2.5。
- 为什么错：I-2.5 逐字「任一块指针、中央映射条目的 value、**journal 点名项里的位置条目数组**按设备身份严格升序」。状态列写「已实现」，射程少了三分之一。
- 失败场景：写路径把点名项的两条位置条目写反（设备 1 在前）→ 这条不红；恢复按点名项验单元时的行为无人核。
- 应该做什么：扫环时对每个点名项的 `locations` 判一次 I-2.5（整条全零的豁免照指针那一格）。

### 8. I-9.4 只判了「记录 inode ≥ 容器号」，「沿叶序容器号严格递增、左容器最大 key < 右容器号」两句没判
- 位置：walk.rs:1015（唯一的 I-9.4 判定点）；对照 1047-1069（I-9.12）
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  self.judgements.judge("I-9.4", inode >= container, || ...);
  ```
- 为什么错：I-9.4 三句：记录 inode ≥ 容器号；沿叶序容器号严格递增；左容器的最大 key < 右容器号。后两句 checker 里没有。I-9.12 比的是分隔 key 与孩子装的 key，分隔 key 与容器号之间没有任何一处比（I-9.2 只比条目身份里的容器号与子头相等），所以 I-9.12 也推不出它们。
- 失败场景：叶序 C1（容器号 5，装 5..10）、C2（容器号 8，装 20..30），分隔 key 5、15：I-9.12 两条不等式都过、I-9.4 第一句过（20 ≥ 8），而「左容器最大 key 10 < 右容器号 8」不成立——不红。
- 应该做什么：沿叶序逐对比容器号严格递增与「左最大 < 右容器号」（数据都在 `walk_inode_root` 手里）。

### 9. I-9.2 的「类型段 0」那一支被当成违例，inode 树只会走「根 → 码 3 叶」两层；根是层级 0 而有条目时整棵静默跳过
- 位置：walk.rs:929-931、945-987、994-997
- 严重度：中（今天不可达：实现拒绝让 inode 树长出第二层，`crates/singlefs-core/src/inode_tree.rs:128-134` 的 `MoreLeafContainersThanOneRootNodeHolds`）
- 置信度：已确认
- 代码做了什么：
  ```rust
  if root.level == 0 { return; }                                         // 929-931
  let entry_holds = record_type == PACKED_TYPE_INODE && unit[6] == 3 && header_identity == identity && child.birth_tree == identity.0;  // 978-981
  .judge("I-1.3", read_u64(&unit, 43) == tree, ...)                       // 994-997（在确认孩子是码 3 之前）
  ```
- 为什么错：I-9.2 逐字「类型段 ∈ {0, 2}；类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0」。checker 只写了类型段 2，类型段 0 的合法条目判 I-9.2 红，接着按码 3 偏移读码 2 孩子：I-1.3 读偏移 43（码 2 的树 ID 在 42）、`judge_packed_container` 报 I-1.6 / I-1.7，孩子之下不再走。层级 0 的根若带条目（射程没定义的形态）则 `return`，不记走读失败、不判任何一条。
- 失败场景：inode 树长到三层那一天（十万文件级，并行线三的预想），每个合法镜像 I-9.2 / I-1.3 / I-1.6 误红；更深的叶从不被走，I-9.4 / I-9.7 / I-9.12 / I-9.13 / I-9.15 对它们没有判定。
- 应该做什么：类型段 0 按条款判子单元是码 2、三段为 0，然后递归（与多层码 2 树同一套 ① ② ③）；层级 0 带条目的根记走读失败。

### 10. 树表里走不了的树（种类 6 / 7 / 8 带了非零根、或不认识的种类码）主走读静默跳过；记账树 / 映射根缺席也只报「不适用」
- 位置：walk.rs:837-839；5113-5116（I-3.1 / I-5.2 / I-3.11 的不适用分支）；611（映射根全零时 `NotWalkable` 不记失败，666-672）
- 严重度：中低
- 置信度：已确认
- 代码做了什么：
  ```rust
  let Some(schema) = key_schema_for_tree_kind(kind) else { return; };     // 837-839
  ```
- 为什么错：day-1 注册的三棵树按条款根指针全零；若写路径给它们写了非零根、或树表里出现登记表外的种类码，这棵树的根不被读（I-2.1 / I-2.5 不判）、不进 `references`（I-3.1 / I-5.1 少数）、不记走读失败（I-7.2「能走完全部权威态」照绿）。同一件事在只读不判的那一遍里是记成「数不全」的（2374 `WithoutWalkableEntryFormat(_) => references.is_complete = false`），主走读却一声不吭。同理：带文件的一版树表里没有记账树条目时，I-3.1 / I-5.2 / I-3.11 报「最新的根下面还没有记账树（第 0 代树表）」，与真正的第 0 代分不开。
- 失败场景：写路径误把 deadlist 树（种类 8）的根指针写成一个真节点的指针 → 那个节点占着的槽不进 I-3.1 并集，I-3.1 红；若记账也漏记，I-3.1 绿，而那棵「不该有根」的树没有任何一条说话。
- 应该做什么：非零根而种类不可走的，记走读失败并判一格；种类码不在登记表里的判红；带文件的一版缺记账树判红而不是不适用。

### 11. 根记录直接持有的那棵分配记录树（`record[342..428]`，C512）的记录，I-3.9、I-5.4 与 `allocation_record_count_under_root` 都不读
- 位置：walk.rs:2006-2013；2612-2646；3443-3477（对照 I-3.10 的 2729）
- 严重度：中低
- 置信度：已确认
- 代码做了什么：
  ```rust
  match allocation_record_tree_without_judging(reader, &allocation_record_tree_root, cache) {
      Some((node_pointers, _)) => { for pointer in &node_pointers { references.note(pointer); } }   // 2006-2010：记录丢掉
  ```
  `judge_allocation_records_disjoint` 与 `allocation_record_count_under_root` 只找树表里种类 3 的条目。
- 为什么错：C512 定案：树表 0 条、写过行的那一版把被换下的单元记在根记录直接持有的分配记录树里——那正是一批带已释放标志的记录。I-3.9（「任一带已释放标志的分配记录」）只看 `scanned[newest].references.allocation_records`，这批进不来；I-5.4（「任何一条有效根的分配记录树」）不读它；I-3.10 却读（2729 把 `record[342..428]` 放进读集），三条口径不一。`allocation_record_count_under_root` 给 harness 的理想模型对拍数「分配记录墙的真条数」（history.rs:824-841），这一版上少数整棵树。它的文档（3443-3448）还写着「第一版分配记录树只有一个节点」「有内部节点时交回 None」「树表 0 条的一版上没有分配记录树」，三句都与现在的代码 / 条款不符。
- 失败场景：回退到无文件那一版、写行之后，最新根是树表 0 条的版本、它的分配记录树里有一条释放代写错的已释放记录 → I-3.9 报「最新的根下面没有带已释放标志的分配记录」不适用；两条记录区间重叠 → I-5.4 不判。
- 应该做什么：三处都把根记录持有的那棵树并进来（照 I-3.10 的读集）。

### 12. 多层码 2 树（记账、映射）根之下的空叶不判；按位置寻址的树反过来连空的根也判红
- 位置：lib.rs:478-484、498-500；walk.rs:526-535、710-717
- 严重度：低
- 置信度：已确认（代码）；按位置寻址树的空根可不可达没核
- 代码做了什么：叶走 `check_index_node_keys`，条目为空时 `if let (Some(first), Some(last))` 整段跳过、交 `Ok`；按位置寻址的树每一层（含根）都走 `check_internal_node_separators`，`if view.entries.is_empty() { return Err(KeyOutsideDeclaredRange) }`。
- 为什么错：I-1.1 末句「根之下没有空节点」。记账 / 映射树里根之下的一片空叶头里区间任意，② 拿父条目的分隔 key 去比这个任意区间，挑得合适就全过。按位置寻址树的「根」不在「根之下」，空根按条款不是违例，checker 判 I-1.1 红。
- 应该做什么：多层码 2 树 `expected_level.is_some() && entries.is_empty()` 判红；按位置寻址树只在根之下判「不空」。

### 13. I-7.7：扫到读不出的槽时 ① ② 一起报不适用，② 的违例也就看不见了
- 位置：walk.rs:1812-1818（`instance_carriers` 任一槽读不出交 None，1743-1775）
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```rust
  let Some(carriers) = instance_carriers(reader, geometry, roots, filesystem_identifier_low) else {
      judgements.not_applicable("I-7.7", "根环、journal 环或单元区有读不出的槽：带号的东西数不全，判不了"); return; };
  ```
- 为什么错：I-7.7 逐字「任一根环区域、journal 记录槽、单元头读不出时**这一半**（①）报不适用」；② 的不适用条件是「镜像记不下这次挂载独占打开的集合」。代码把两半捆在一起：各盘系统配置都读得出、只是单元区有一个槽读不出时，读得出的那些载体里已经出现「较大者」也不判。
- 应该做什么：读不出只让 ① 不适用；② 在读得出的载体上照判（找到就红，找不到才不适用）。

### 14. I-9.6 拿最新根的水位去比「全部走过的版本里的最大 inode 号」，树维也丢了；不适用的理由写的是「最新的根」
- 位置：walk.rs:185、1034-1037、4946-4967
- 严重度：低
- 置信度：已确认（代码）；v1 上不会误红（水位只增不减，D23 已定项 14 管理员回退「水位取 max」）
- 代码做了什么：`largest_inode_number_in_the_inode_tree` 在同一个 `Walk` 里跨最新根、候选根、由记录施加出来的版本一路取 max；水位只取 `accounting.get(&(12, 0xFFFF_FFFF))`，key 里的树 ID 段不看。
- 为什么错：I-9.6 逐字「每个可写头的 inode 号水位 > 该树 inode 树内最大 key」。比的两边来自不同的版本集合、不分树；`(Some(_), None)` 分支的理由「最新的根下面走不到 inode 树里的任何一条记录」与实际取值口径不符。有了克隆头（多棵 inode 树）之后两棵树的水位与最大号会串。
- 应该做什么：最大号只取最新根那一遍（像 `slots_referenced_by_the_newest_root` 那样在走完最新根时截一次），按树 ID 分组比。

### 15. 记账行按 (统计量, 设备) 进表：树 ID 段、代段、seq 都丢了，重复行后写覆盖前写（与实现共用的前提）
- 位置：walk.rs:876-879
- 严重度：低
- 置信度：已确认（代码）；今天与实现一致（`crates/singlefs-core/src/transaction.rs:5851-5868` 每次发布整棵重建、每个统计量每块盘一行）
- 代码做了什么：
  ```rust
  self.accounting.insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
  ```
- 为什么错：记账 key 是 (标签 2, 树 ID 8, 设备 4, 代 8)（D5 已定项 5），D5 同一项的射程写明「四段在一个 checkpoint 内不唯一，后者胜要靠 seq 定」，第 12 项带树维。checker 取「key 序里最后一行」当值、不看 seq、不看树，也不因同一 (统计量, 设备) 出现多行而报任何东西——它依赖的是「实现只写一行」这个实现行为，而不是条款。写路径多留一行旧代的「已分配」（点删漏了）时，checker 取哪一行取决于代段的排序，可能正好拿旧值去比 I-3.1 / I-5.2 / I-3.11。
- 应该做什么：同一 (统计量, 树, 设备) 多行时按条款的合并规则（后者胜按 seq、单调量取 max）合并，或至少判红「一版里同一统计量多行」。

### 16. I-7.6 的 devs 取 reader 交出的盘，不取 mkfs 记下的设备数
- 位置：walk.rs:4623-4642
- 严重度：低
- 置信度：已确认
- 代码做了什么：`distinct.len() == regions.min(devices.len()) && … && region_devices.iter().all(|device| devices.contains(device))`，`devices = reader.devices()`；系统配置里的 `device_count`（lib.rs:219 已解出）从没用上。
- 为什么错：I-7.6 判的是「mkfs 时逐区域写下的设备身份」对 min(R, devs) 与鸽笼下界，devs 是池的设备数。少一块盘的镜像（D23 已定项 14 注里讨论过的「一块盘整块掉了」）上，这里会因为 `devices.contains` 不成立而判红，而区域身份本身没写错。
- 应该做什么：devs 取系统配置的设备数；「区域身份指向的盘在不在」另判或报不适用。

### 17. 隔离豁免的归组键不含设备，且「没有位置项可比时退回自证」让从没写成的泄漏也被豁免
- 位置：walk.rs:3391-3439
- 严重度：低（豁免本身是 2026-09-25 用户定案；这里只报读法上的两处缝）
- 置信度：已确认（代码）
- 代码做了什么：
  ```rust
  let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();   // 键 (起点槽, 跨度)
  None => check_unit(&unit).is_ok(),                                               // 3428
  ```
- 为什么错：① 键里没有设备：「第一版一个单元两盘同槽」是实现的放置行为，不是条款；两块盘同槽上的两个不同单元会被并成一组，一份坏了连另一块盘上那条也豁免。② 没有任何根（含根环里没走过的）指着这个槽时退回自证：一条已分配、无引用、而槽上从来没写过有效单元的记录（写路径漏写或写错槽）正好自证不过，于是被当成「隔离」加进 I-3.1 / I-3.11 的走读数，泄漏被抵掉。代码注释（3426-3427）自己写着「这一格条款没写」。
- 应该做什么：键带上设备（按「单元」归组要有单元身份，不能靠同槽）；退回自证那一支至少单独计数、在违例说明里报出来，交主 agent 定。

### 18. 零碎：I-9.1 只读第一份原始字节；实例表身份记在 I-1.1 名下；I-1.10 跳过不算走读失败
- 位置：walk.rs:840-853；1122-1139；792-800、1317-1324
- 严重度：低
- 置信度：已确认
- ① I-9.1：`self.reader.read(pointer.locations[0].device, …)` 读不到就一格不判（不适用），读到一份与校验和对不上的旧字节也拿它的偏移 6 判——判的是第一份原始字节而不是「对得上的那一份」。
- ② I-1.1 那一行明写「自证单元（……类型 4 实例表单元）显式豁免」，而 1129-1134 把实例表片的身份四元组 (0, 4, k, 0) 记在 I-1.1 名下；判据本身合理，但归属与条款不符，违例说明会把人引到错的条款。
- ③ 条目宽不等于字段表时 `entry_width_holds`（记账树那一支）与 `position_and_entry_width_hold` 只判 I-1.10、不记走读失败；那棵子树从此不走，而 I-7.2（「能走完」）与每条候选根的 I-7.4 / I-4.8 只看走读失败与 I-2.1，照绿。I-1.10 仍会红，所以只是归因缺一格。

### 19. 死代码与名不副实的测试
- 位置：lib.rs:20-34、707-713；lib.rs:54-55
- 严重度：低
- 置信度：已确认（`grep -rn DecisionWidth crates research` 只命中 lib.rs 自己）
- `DecisionWidth` 全仓无调用点，只有它自己的单测；它的文档「探不到时报『声明值，未探测』，不许当成探到的」没有兑现——`valid_roots`（image.rs:254）直接拿系统配置里的声明值当根槽宽。那条单测 `assert_ne!(DecisionWidth::DeclaredOnly{..}, DecisionWidth::Probed{..})` 比的是两个不同的枚举成员，恒真。
- lib.rs:54-55 说查表 CRC「与按位那一份逐字节同值（单测对拍）」，checker 里没有这条单测（全仓只有 lib.rs 同时出现两个函数名）；I-2.1 / I-2.3 用的恰是查表那一份。
- lib.rs 的另外两条单测只喂全零缓冲（`garbage_slots_are_rejected_with_the_right_reason`），ChecksumMismatch、UnknownIncompatBit、NonZeroFlags、FilesystemIdentifierMismatch 这几臂在 checker crate 内没有单测（判别力靠 harness 的坏镜像）。walk.rs 没有 `#[cfg(test)]`。

## 读过的文件与行数

| 文件 | 行数 | 读法 |
|---|---|---|
| crates/singlefs-checker/src/lib.rs | 737 | 全读 |
| crates/singlefs-checker/src/image.rs | 362 | 全读 |
| crates/singlefs-checker/src/position_addressed.rs | 338 | 全读 |
| crates/singlefs-checker/src/walk.rs | 5118 → 5143（审阅中途被别的会话插了 25 行） | 全读（1-5118 分段读；插入的 2951-2975 另读） |
| crates/singlefs-checker/Cargo.toml、Cargo.toml（overflow-checks） | — | 全读 |
| .claude/kb/invariants.md | 45 条已实现行全文 | 逐条对照 |
| 对照用：layout/01-first-txn.md 七节与系统配置 / journal 字段表；D5 已定项 4 / 5 / 10；D19 已定项 11；D23 注 1-4 与已定项 8；core 的 mount.rs:2845-2869、inode_tree.rs:110-175、transaction.rs:5840-5868；harness crash.rs:543-617、history.rs:824-841 | — | 按需 |

## 45 条已实现不变量 → checker 落点

| 编号 | 函数（walk.rs 除注明外） | 判定行 | 判定 |
|---|---|---|---|
| I-1.1 | read_index_node / walk_code_two_subtree / walk_instance_table_chain / walk_extent_data_pointer / position_and_entry_width_hold / walk_allocation_record_node / walk_extent_upper_node / walk_extent_lower_node | 536、690、752、761、773、1130、1279、1300、1313、1397、1424、1503、1533、1601、1619 | 查窄了（发现 1、6、9、12） |
| I-1.2 | judge_birth_identity_of_a_referenced_unit | 336 | 查窄了（只遍历方向，kb 写明；首次引用才判、映射条目不判：发现 1、6） |
| I-1.3 | read_index_node / walk_inode_root / walk_extent_data_pointer | 520、995、1270 | 查全了（第二次引用不判见发现 1；类型段 0 误判见发现 9） |
| I-1.4 | judge_unit_header / check_pool_image（各盘 fsid） | 470、4613 | 查全了（只判走读到的单元；映射条目指的单元不判头，发现 6） |
| I-1.6 | judge_unit_header | 417 | 查窄了（同上，映射条目的类标签不比，发现 6） |
| I-1.7 | judge_packed_container | 1227 | 查全了（走读到的码 3） |
| I-1.8 | judge_merged_version_total_order | 3830、3870（不适用 4896） | 查窄了（② 码 1 不判，kb 写明等 C464） |
| I-1.10 | entry_width_holds / walk_tree_table_entry / position_and_entry_width_hold | 795、912、1318 | 查全了（跳过不算走读失败，发现 18③） |
| I-2.1 | image.rs read_referenced_unit | image.rs:351 | 查全了（候选集射程，kb 写明） |
| I-2.3 | judge_unit_header | 461 | 查全了（走读到的单元） |
| I-2.4 | judge_unit_header | 448 | 查全了（走读到的单元） |
| I-2.5 | image.rs judge_location_order | image.rs:326 | 查窄了（journal 点名项没判，发现 7） |
| I-3.1 | check_pool_image | 5069（不适用 5108、5114） | 查全了（重复引用见发现 1，豁免读法见发现 17，记账取值见发现 15） |
| I-3.8 | judge_instance_table_rows / walk_instance_table_chain | 1198、1144 | 查全了（回收那一半 kb 写明判不了） |
| I-3.9 | judge_release_generations | 2453（不适用 2415、2427、2468、2551） | 查窄了（根记录持有的分配记录树不读，发现 11） |
| I-3.10 | judge_allocation_generations_against_unit_births | 2804 | 查全了 |
| I-3.11 | check_pool_image | 5100 | 查全了（记账取值见发现 15） |
| I-4.2 | judge_birth_identity_of_a_referenced_unit | 359 | 查窄了（首次引用、映射条目不判；「重放后」那半 kb 写明） |
| I-4.8 | check_pool_image | 4726、4819、4848 | 查全了（候选集） |
| I-5.1 | check_pool_image | 4996 | 查窄了（同起点同跨度去重，发现 1） |
| I-5.2 | check_pool_image | 5080 | 查全了，但会 panic（发现 4） |
| I-5.4 | judge_allocation_records_disjoint → judge_allocation_record_ranges | 3311、3317（不适用 2650） | 查窄了（根记录持有的那棵不读，发现 11） |
| I-7.1 | check_pool_image | 4684 | 查全了 |
| I-7.2 | check_pool_image | 4872 | 查窄了（不可走的树静默跳过，发现 10、18③） |
| I-7.3 | judge_root_ring_health | 3915 | 查全了 |
| I-7.4 | check_pool_image | 4730、4813、4842 | 查窄了（被抛弃而仍在环里的根不判，发现 5） |
| I-7.6 | check_pool_image | 4639 | 查全了（devs 口径见发现 16） |
| I-7.7 | judge_instance_carriers | 1825、1832（不适用 1802、1815） | 查窄了（发现 13） |
| I-7.8 | check_pool_image + scanned_tree_identifiers | 5139 | 查全了（按 kb 定的读法） |
| I-7.9 | judge_rollback_floor_raises_against_their_ceilings | 3193、3221 | 查全了 |
| I-7.12 | judge_system_configuration_floor_against_the_roots_on_each_device | 3274 | 查全了 |
| I-8.6 | judge_journal_back_chain | 4070 | 查全了，但计数器 0 panic（发现 3） |
| I-8.7 | judge_transaction_numbers_per_instance | 4144、4157 | 查全了 |
| I-8.8 | judge_commit_markers_per_transaction | 4288、4290 | 查全了 |
| I-8.9 | judge_publish_ordinals_and_last_record_flags | 4463、4465 | 查全了 |
| I-9.1 | walk_tree_table_entry | 848 | 查窄了（只读第一份原始字节，读不到不判，发现 18①） |
| I-9.2 | walk_inode_root | 982 | 查窄了（类型段 0 误判，发现 9） |
| I-9.4 | walk_inode_root | 1015 | 查窄了（三句只判一句，发现 8） |
| I-9.6 | check_pool_image | 4954 | 查全了（口径串版本、不分树，发现 14） |
| I-9.7 | walk_inode_root | 1011 | 查全了 |
| I-9.10 | check_pool_image | 4976 | 查窄了（同一单元第二次引用不登记，发现 1） |
| I-9.12 | walk_inode_root | 1052、1059、1063（不适用 4970） | 查窄了（跨根共享的孩子不按新父条目判，发现 1） |
| I-9.13 | walk_inode_root | 1003 | 查全了 |
| I-9.14 | judge_tree_table_birth_txg | 2518 | 查全了 |
| I-9.15 | walk_inode_root | 1023 | 查全了 |

没找到落点的：0 条。

## 注释与代码不一致清单

| 位置 | 注释说的 | 代码 / 现状 |
|---|---|---|
| lib.rs:54-55 | 查表 CRC「与按位那一份逐字节同值（单测对拍）」 | checker 里没有这条单测 |
| lib.rs:20 | `DecisionWidth`：探不到时报「声明值，未探测」 | 全仓无调用点；`valid_roots` 直接用声明值 |
| walk.rs:150-151 | `AlreadyWalkedFromAnotherRoot`：「别的根已经走过它」 | 同一根内第二次引用也返回它（发现 1） |
| walk.rs:746 | ② 跳过的孩子「那时已经按它们自己的父条目判过」 | 判过的是另一个父条目，不是这一条（发现 1） |
| walk.rs:1994-1995 | 映射条目指的单元「与树里引用的是同一批」 | 没有任何检查核这句（发现 6） |
| walk.rs:1907 | `allocation_records`「这条根的分配记录树叶里的记录」 | 根记录持有的那棵的记录在 2007 丢掉（发现 11） |
| walk.rs:3443-3448 | 「第一版分配记录树只有一个节点」「有内部节点时交回 None」「树表 0 条的一版上没有分配记录树」 | 代码按位置整棵走多层；C512 之后树表 0 条的一版可以有根记录持有的那棵（发现 11） |
| walk.rs:4964-4965 | I-9.6 不适用理由「最新的根下面走不到…」 | 最大号取的是全部走过的版本（发现 14） |
| walk.rs:4983 | I-5.1「不同的引用不许占重叠的槽」 | 同起点同跨度的不同引用在 398-401 被并成一项（发现 1） |
| walk.rs:4044 与 3930 / 4329 | I-8.6 ③「计数器 − 1 那一条实例不同 ⇒ 这一条是本实例第一条」；3930 写「从前缀末 + 1 接着写」，4329 写「从读得出的最大号 + 1 接着写」 | 两句按 D23 已定项 14 注 3（前缀末取读得出的最大号）是一回事，实现也这么做（core mount.rs:2864-2869）；但 D23 已定项 8 的正文仍写「恢复之后从前缀末 + 1 接着写，那正好接在残留序列的前面」。若按后者，残留的旧实例记录坐在新实例记录之后，I-8.6 ③ 与 I-8.9「不从 1 起」会在合法镜像上误红——checker 的这两格压在「新实例从读得出的最大号 + 1 起」这一实现行为上，条款两处说法没统一 |
| .claude/kb/invariants.md I-7.4 状态列 | 「checker 今天只读根上带的 F……欠 C556」 | walk.rs:4773-4776 已按系统配置里的 F 取 F_生效，I-3.1 那一行写 C556 已还清——kb 这一句过期 |
