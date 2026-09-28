# 验证装置核心审阅报告（录制器 / 切段 / 重放 / 理想模型 / 对拍）

只读审阅，没跑 cargo、脚本、门禁与任何测试二进制。

**行号口径**：审阅途中另一个会话在改 `crash.rs`（mtime 2026-09-26，3087 → 3120 行）、`model.rs` 与 `model_comparison.rs`（mtime）。下文五个文件的行号一律指拷下的快照（本草稿目录 `G-snap-*.rs`），sha256：

| 文件 | 行数 | sha256 前 16 位 |
|---|---|---|
| `crates/singlefs-harness/src/lib.rs` | 629 | 80449edaaedf6a4e |
| `crates/singlefs-harness/src/segments.rs` | 349 | 89ec444ca5899ed9 |
| `crates/singlefs-harness/src/crash.rs` | 3120 | 9bf8208d52b052a4 |
| `crates/singlefs-harness/src/model.rs` | 2395 | fa4cd433d608285e |
| `crates/singlefs-harness/src/model_comparison.rs` | 610 | fa50cf5c1524f767 |

快照之后 `crash.rs` 又被改到 3454 行（核的）：导入多了两行（快照第 29 行之后的行号在现行文件里整体 +2），并新加了双机分片枚举与账本合并（`enumerate_layer0_in_state_slices_or_one_shard`、`Layer0Resume::RunOneShardKeepingProgressFile` / `MergeShardLedgers`）。分片与账本合并这部分**没有审**。`diff` 的改动块全在快照 1874–2418 行（切片、计划哈希、并片与进度文件）与第 28–29 行；「问题」各条引用的快照行都在这一段之外，不受影响；「查过、没发现问题的几格」里「并行合并的确定性」与「状态评估时 panic」两格读的是快照上的这一段，要按现行文件重核。

审阅途中有一次改动在 `crash.rs` 里新加了「第二条判据不罩被抛弃时间线上的发布」（`check_records_against` 707–764、`instance_table_of_the_effective_root` 766–783）。改动前我读到的版本在这一格按 txg 比、不看实例，已经被这次改动补上，下文不再报。

## 问题（按严重度）

### 1. 屏障按池算、录制器把相邻屏障并成一道：一块盘漏发屏障在录制流里看不出来，层 0 与崩溃注入都枚举不出「另一块盘的写还没落」的状态
- 位置：`lib.rs:181-189`（`push` 丢屏障）；`crash.rs:488-492`（切段）；`segments.rs:104-109`（登记表切段）；被录的调用方 `crates/singlefs-core/src/transaction.rs:327-333`、`make_filesystem.rs:289-291`、`make_filesystem.rs:366-368`
- 严重度：高
- 置信度：已确认
- 代码做了什么：
  ```rust
  if operation.kind == RecordedOperationKind::Barrier && previous_is_barrier {
      return;
  }
  ```
  ```rust
  RecordedOperationKind::Barrier => {
      if !current.is_empty() {
          segments.push(std::mem::take(&mut current));
  ```
- 为什么错：`BlockDevice::barrier` 是一块盘的动作（`block_device.rs:132`「屏障：之前发出的写全部持久之后才返回」，两个后端各对自己的文件 `sync_data`），池屏障是实现对每块盘各发一道（`transaction.rs:329-331` 的 `for (_, device) in self.devices.iter_mut() { device.barrier()?; }`）。录制器不看设备，把紧跟在屏障后面的屏障一律丢掉；两处切段也不看屏障属于哪块盘，一道屏障就把全池的写关进上一段。于是「两块盘都屏障了」与「只有盘 0 屏障了」录出同一条流。
- 失败场景：把 `transaction.rs:329-331` 那个循环改成只对盘 0 发屏障（或 mkfs 那两处循环少一块盘）。原流里盘 1 那一道本来就被 `push` 丢了，改后的流逐字相同，层 0 与崩溃注入（后者走同一个切段函数）的状态集合与判定逐项相同，全绿。真设备上盘 1 的单元副本、journal 记录在根槽 FUA 落盘时可以还没落，「一份副本缺席而根已持久」这类状态从不生成；checker 按位置条目逐份读、逐份判 I-2.1（校验和与内容匹配）（`crates/singlefs-checker/src/image.rs:337-338` 的文档原句「每条位置条目都读、都判 I-2.1（校验和与内容匹配）」），本来会在它上面判红。`crates/mutations.tsv`（812 行）里 `device.barrier` 命中 0 次，没有变异盯这一格；带「屏障」的几条（58、298、714 行，与 `crash.rs` 那条 153 行）都是整道池屏障少掉。
- 应该做什么：屏障按设备记、按设备切段（每块盘各自的段序列，段内子集按盘算）；最少也要在录制流里保留每块盘那一道，并加一条检查「一次池屏障覆盖了池里每块盘」。

### 2. 崩溃状态上只跑只读的 `recover`，不跑可写挂载；记录核对器的「崩溃后镜像」用的是崩溃态镜像，与 D13（验证路线） 已定项 7 的定案不一致
- 位置：`crash.rs:1421-1422`（两遍 `recover`）、`crash.rs:1473`（checker 判崩溃态镜像）、`crash.rs:1494`（记录核对器）、`crash.rs:668-670`（文档）；`crash_injection.rs:683`；`.claude/kb/decisions/13-验证路线.md:131`
- 严重度：高
- 置信度：已确认
- 代码做了什么：
  ```rust
  let consulted = recover(&image, JournalPolicy::Consult);
  let ignored = recover(&image, JournalPolicy::Ignore);
  ...
  let records = check_records(&image, consulted.effective_root);
  ```
  `recover` 只拿 `&dyn PoolReader`（`crates/singlefs-core/src/recovery.rs:2794`），一个字节都不写；`check_records` 的文档写「崩溃后镜像是 `image` 本身」（`crash.rs:669`）。
- 为什么错：D13（验证路线） 已定项 7 定案原文：「⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。」代码里没有「实现恢复后的镜像」这一份：可写挂载（取号的系统配置写、写行那次发布、暖机）在任何一个枚举出的崩溃状态上都没被执行过。崩溃注入同样只调 `recover`（`crash_injection.rs:683`；该文件里 `mount` 零命中）。
- 失败场景：崩溃之后的可写挂载复用了一个在崩溃态里仍被某条候选根引用的槽，或写行那次发布把一条没被所选根覆盖、但仍在前缀里的记录盖掉——层 0 与崩溃注入都不会红，因为这段代码在崩溃状态上从没跑过。恢复中途再崩（二次崩溃）也不在枚举域里。现有的定向用例（如 `tests/second_transaction_supplement_two_tree_identifier_watermark_crash_orphans.rs:263-274`：只对一次发布的每个前缀、挂载之后只跑 checker、不跑记录核对器）罩不住这一格。
- 应该做什么：对每个崩溃状态在一份副本上跑 `mount_writable`，再对挂载后的镜像跑 checker 与记录核对器（按 D13（验证路线） 已定项 7 的双镜像入参）；做不到就在 D13（验证路线） 与 `check_records` 的文档里如实写「层 0 不跑写恢复」，并登记欠账。

### 3. 「撕裂并进没持久」对原地覆写不成立：层 0 从不生成「原地覆写的那一槽新旧都读不出」的状态
- 位置：`crash.rs:1-2`（模块头）、`crash.rs:389-416`（`CrashImage::read` 一次写要么整份叠上、要么不叠）、`crash.rs:37`（扇区 512）；`.claude/kb/decisions/13-验证路线.md:73`
- 严重度：中
- 置信度：已确认（这一类状态没被枚举）；影响是疑似（没有状态证明恢复在它上面对）
- 代码做了什么：
  ```rust
  for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
      if !is_persisted || write.device != device {
          continue;
  ```
  持久集合按整写取，没落的位置放旧字节。
- 为什么错：D13（验证路线） 已定项 4 射程原文整行（`13-验证路线.md:73`）：「**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验；设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内——撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度因此不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。」其中「撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率」这句只对 COW 写（落在空闲槽上，旧内容本来就没人要）成立。原地覆写被撕裂时，那一槽的**旧内容也没了**；「没持久」却留着完整可用的旧内容。两者在校验和眼里不是同一件事。今天的原地覆写：系统配置槽 4096 字节（`crates/singlefs-format/src/lib.rs:233`，512 字节扇区上是 8 个扇区），每次发布在每块盘写 `slot_generation % 2` 那一槽（`transaction.rs:397`），盖掉的是 gen−2；journal 记录 4096 字节在环回绕时原地覆盖。根槽写一个物理块（512），不受影响。
- 失败场景：某块盘上系统配置那一槽被撕裂（gen 没写全、gen−2 也没了），恢复只能靠另一槽的 gen−1。层 0 里这一槽恒是「gen−2 完好」或「gen 完好」，恢复在「这一槽读不出」上的行为没有一个枚举状态判过。
- 应该做什么：原地覆写的写按「没持久 / 落了 / 这一槽坏（旧新都不是）」三态枚举（坏态可以用一段不过校验和的字节代替逐扇区撕裂）；或者在 D13（验证路线） 已定项 4 的射程里把「原地覆写撕裂 = 旧内容丢失」写成承认的省略。

### 4. FUA 关段按池算，并把同段里 FUA 之前的普通写当成此后恒已持久
- 位置：`crash.rs:522-524`；`segments.rs:91`、`segments.rs:110-113`
- 严重度：中
- 置信度：已确认（今天的流里潜伏）
- 代码做了什么：
  ```rust
  if retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess {
      segments.push(std::mem::take(&mut current));
  }
  ```
  注释（`segments.rs:91`）：「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」。
- 为什么错：关段之后，后面每一段的状态都把这一段的写当作已落盘，这正好是「FUA 替前面的普通写做了持久」。对同一块盘，两个后端的 FUA 是「写完立刻 `sync_data`」（`block_device.rs:4`），那一半成立；FUA 只作用于它那块盘，同段里**别的盘**上的普通写在 FUA 之后并没有持久。
- 失败场景：一条路径在根槽 FUA 之前没有屏障、而另一块盘在同段有写。缺的状态是「FUA 落了、后面的段也落了、而另一块盘上 FUA 之前那几次写没落」。今天发布路径是「单元 → 屏障 → 记录 → 屏障 → 根槽 FUA」（`transaction.rs:837-851`、`transaction.rs:988`），FUA 那一段只有它自己（`segments.rs` 用例里的 `[root_record_fua]`），所以潜伏。
- 应该做什么：FUA 只关它所在那块盘的段；或者加一条检查「FUA 所在的段里没有别的盘的写」，违反就判红，不静默当成已持久。

### 5. `SparseBlockDevice` 不做越界检查，越界写静默成功；内存镜像的读违反两个 reader 契约
- 位置：`crash.rs:146-154`（`write_at`）、`crash.rs:72-84`（`SparseDevice::write` 只断言对齐）、`crash.rs:348-357`（`PoolReader::read`）、`crash.rs:548-550`（`ImageReader::device_bytes`）；调用方 `crates/singlefs-harness/src/history.rs:895-897`（随机历史的被录设备）、`crates/singlefs-harness/tests/common/mod.rs:258-270`（崩溃后挂载用的盘）
- 严重度：中
- 置信度：已确认（设备行为）；「实现有没有这种越界」没核
- 代码做了什么：
  ```rust
  self.image.write(offset, bytes);
  Ok(())
  ```
  ```rust
  fn device_bytes(&self, _device: u32) -> Option<u64> {
      Some(self.device_size_in_bytes)
  ```
- 为什么错：文件后端对越界与不对齐返回 `OutOfRange` / `Unaligned`（`crates/singlefs-core/src/block_device.rs:209-241`），稀疏盘照收、照读回。`PoolReader` 契约写「读不到（没有那块盘、越界）返回 None」（`crates/singlefs-core/src/recovery.rs:80-81`），checker 的 `ImageReader` 同样（`crates/singlefs-checker/src/image.rs:20`）；内存镜像越界读给 `Some(全 0)`，`device_bytes` 对池里没有的盘号也答 `Some`，不对齐读直接 panic（`crash.rs:55` 的断言）而不是 `None`。
- 失败场景：实现把一个单元落到盘尾之外（小盘墙那一类池，`model_comparison.rs` 用例里单元区只有 640 槽）：随机历史里稀疏盘写得进、读得回，对拍与冷启动读回都绿；真盘上这一步是 I/O 错。
- 应该做什么：`SparseBlockDevice::write_at` / `read_at` / `write_zeroes_at` 复用与文件后端同一个 `check_aligned_and_in_range` 判据返回错误；`MemoryPool` 与 `CrashImage` 的两个 reader 越界、没有这块盘时返回 `None`。

### 6. `index_node_header_bytes`（带算术的 const fn）被模型、checker、实现三方共用
- 位置：`crates/singlefs-format/src/lib.rs:50-54`；`model.rs:23`、`model.rs:260-262`；`crates/singlefs-checker/src/lib.rs:14`、`:314`；`crates/singlefs-core/src/unit.rs:4`、`:127`、`:155`、`:353`
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  pub const fn index_node_header_bytes(key_width_in_bytes: u64) -> u64 {
      INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE
          + 2 * key_width_in_bytes
          + NONCE_MAC_ALGORITHM_RESERVED_BYTES
  ```
- 为什么错：D13（验证路线） 已定项 5（`13-验证路线.md:88`）：原文整行：「- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。」这个函数有算术，实现写码 2 节点头、checker 解析码 2 节点头、模型算节点容量都经它。
- 失败场景：式子里「2 × key 宽」或预留位那一项错了，实现写出的节点与 checker 读的节点一起错，checker 判绿。模型那一处只进 `accounting_rows_exceed_one_node` 的断言护栏，影响小；主要风险在 checker 与实现之间。
- 应该做什么：格式 crate 只留标量（各种 key 宽对应的头宽各一个常量），checker 与实现各自写这一步算术。

### 7. 对拍只核实现交回了的角色，漏交的角色不报（单向比较）
- 位置：`model.rs:1877`（`judge_allocation_generations`）；`model_comparison.rs:116-143`、`:146-156`；用例 `model.rs:2356-2394`
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```rust
  for (role, records) in &observed.unit_allocation_records {
      let Some(written_at) = expected.role_written_at.get(role) else {
  ```
  只遍历实现交回的角色；树表 0 条的一版交回 `unit_allocation_records: Vec::new()`（`model_comparison.rs:154`）。
- 为什么错：模型这一版有哪些角色（`role_written_at` 的键）与实现交回了哪些角色从没按集合比过。用例 `carried_unit_keeps_the_generation_of_the_publish_that_wrote_it` 自己就是反例：写行那一版在模型里有九个角色，观测只给 `Data` 与 `InstanceTable` 两个，判 `Ok(())`。
- 失败场景：实现的 `TransactionOutput.units` 漏了一个角色（例如 inode 根），对拍不报；`RowsOnVersionWithoutFile` 那次发布重写的实例表与分配记录树的分配代永远不比。
- 应该做什么：先比角色集合（观测到的角色 = 这一版 `role_written_at` 的键，树表 0 条那一版也交回它重写的两个角色），再逐角色比分配代。

### 8. 对拍不比每一版的文件内容与实例表内容，只在冷启动那一步比内容、只在挂载那一步比写的行
- 位置：`model.rs:1808-1869`（`judge_roots` 比 key、jsn、F、有没有文件、分配记录）；内容只在 `model.rs:1763-1800`（`ColdStart` 分支）；行只在 `model.rs:1752`（`rows_written`）
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```rust
  if expected.file.is_some() != observed.has_file {
  ```
  `ObservedRoot` 没有内容与实例表字段（`model.rs:406-413`）。
- 为什么错：模型每条根都记着内容与整张实例表（`ModelRoot.file`、`instance_table_rows`），实现那一侧只交回「有没有文件」。
- 失败场景：一次覆盖写写错了内容，而下一次冷启动之前又覆盖写了一次，那一版的内容从没被比过；写行那次发布整链重写实例表时丢了更早的行、新写的行本身对，对拍不报，要等以后某次回退或抛弃判定分叉才可能露出来。
- 应该做什么：`ObservedRoot` 带上这一版的文件内容（或它的哈希）与实例表整张，逐步比。

### 9. 模型有几处照实现的做法写，没有标「预想」；第一个文件版本与其余发布用两条不同的 jsn 规则
- 位置：`model.rs:389`、`model.rs:1497-1499`（墙在第一次写之前一串判完）；`model.rs:821`（分配记录树根的层号「实现员的取法」）；`model.rs:276`；`model.rs:1008` 对 `model.rs:1048`、`:1080`、`:1281`、`:1343`、`:1476`
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```rust
  // 实现在任何写之前把这一串整串预演一遍（`mount::raise_rollback_floor`），哪一次撞墙都在第一次写之前拒、一次都不发。
  answer.walls_are_judged_before_the_first_write = true;
  ```
  ```rust
  ModelJournalCounter(current.journal_counter.0 + 1),        // 第一个文件版本（1008）
  ModelJournalCounter(self.highest_journal_counter.0 + 1),   // 覆盖写、零单元、回退、新实例、抬 F
  ```
- 为什么错：模型头（`model.rs:11-12`）说照代码读法写的地方都标「预想，跟收口表第 X 行」，这几处没标；它们在这一格上与实现同源，对拍证明不了条款。模型自己的类型文档（`model.rs:38`）写 jsn 是「D23（journal 的角色与格式） 已定项 14 第 3 条：全池接着走、换实例不归零」（D23（journal 的角色与格式） 原文我没现读），模型对第一个文件版本却取「现行根的 jsn + 1」（`model.rs:276` 的注释说的正是实现的做法：「jsn 从那条记录接着算」）。今天会话里现行根的 jsn 恒等于全池最大，两条规则分不开。
- 失败场景：实现的第一个文件版本在某条路径上拿到的「上一条记录」不是全池最新的那条，模型跟着实现一起接错，对拍绿。
- 应该做什么：这几处按条款另写一份，或标「预想」并登记；第一个文件版本的 jsn 也取 `highest_journal_counter + 1`。

### 10. 恢复用的 journal 提示与真盘上的全环扫描不等价
- 位置：`crash.rs:418-435`（`CrashImage::journal_record_offsets_hint`）；对照 `crash.rs:325-336`（基镜像那一份按记录槽对齐过滤）；被调方 `crates/singlefs-core/src/recovery.rs:1836-1855`（`scan_journal` 只读提示里的偏移）
- 严重度：低
- 置信度：已确认（代码路径）；影响疑似
- 代码做了什么：
  ```rust
  if !is_persisted || write.device != device || write.kind != StepKind::JournalRecord {
      continue;
  }
  offsets.push(write.offset);
  ```
- 为什么错：叠加层的偏移不按 `(offset − ring_start) % 4096 == 0` 过滤，一次写罩几条记录时只推第一条；真盘没有提示，`scan_journal` 按 4096 对齐逐槽扫全环。
- 失败场景：实现把一条记录写到不对齐的偏移：真盘恢复扫不到它；层 0 的提示里带着这个偏移，恢复读到、施加它，层 0 绿。
- 应该做什么：叠加层与基镜像用同一份「写过的扇区 → 对齐的记录槽」算法；或加一条检查：提示给出的偏移都对齐、且与全环扫描结果相同（抽几个状态对拍）。

### 11. 写失败时不录：部分落盘的写从录制流里消失
- 位置：`lib.rs:338`、`lib.rs:366`、`lib.rs:381`
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```rust
  self.inner.write_at(offset, bytes, durability)?;
  ```
  之后才 `push`。
- 为什么错：文件后端的 `write_all_at` 与分块清零（`block_device.rs:307-319`）可以写进一部分再报错；这一步不进流。池屏障在盘 0 成功、盘 1 失败时，流里照样是一道「池屏障」（与第 1 条同源）。
- 失败场景：只在设备出错时出现（故障注入那一路，不在本次范围）；层 0 的健康流不受影响。
- 应该做什么：失败的写也记一条（带「失败」标记与写到的范围未知），切段时按「可能落了一部分」处理，或至少让重放拒收带失败标记的流。

### 12. 并片单测没钉住 `failed_states` 与 `verification_failed_states` 两项相加
- 位置：`crash.rs:1137`、`crash.rs:1140`；用例 `crash.rs:2844-2966`
- 严重度：低
- 置信度：已确认
- 代码做了什么：用例里两片的 `failed_states`、`verification_failed_states` 都是 0，断言的元组（`crash.rs:2928`）不含这两项；`crates/mutations.tsv` 里 `failed_states` 零命中。
- 为什么错：删掉 `self.failed_states += failed_states;`，这条用例照绿。
- 失败场景：层 0 用例的 `assert_eq!(tally.failed_states, 0, ...)` 对第二片以后的失败状态假绿（同一个状态 oracle 仍会记进 `violations`，所以整体不至于漏判，但「走读一次都不失败」那一条断言失去判别力）。
- 应该做什么：用例里给后一片非 0 的这两项并断言相加。

## 查过、没发现问题的几格（带依据）

- **内容怎么拿回**：开了内容保留的流（`SharedStream::retaining_contents`，`lib.rs:150-158`）逐次存写的字节，重放从流里取（`crash.rs:506-526` 取 `retained.contents`，没开就 `expect` panic），不靠再跑一遍程序；清零只记偏移与长度、重放按长度铺 0（`lib.rs:355-376`、`crash.rs:493-505`）。哈希只用于流文本，不参与重放。
- **读不影响崩溃状态**：读不进流（`lib.rs:324-330`）。程序在录制那一趟里读到的是「全部写都落了」的盘，读-改-写算出来的新字节原样录下、重放时按录下的字节施加，与真设备缓存里读到未持久的写同形。
- **状态计数与序号映射**：全量每段取任意真子集（掩码 0 … 2^n − 2，含空集），最后加「全部持久」一个，等于闭式 1 + Σ(2^|段| − 1)，没有重复也没有漏（`crash.rs:1773-1825`；用例 `crash.rs` 里拿逐段走的参照走法逐个比序号 → 持久集合）。甲二的掩码拆法（最低位 = 单元写全落）与参照走法逐个比过。
- **并行合并的确定性**：已实现。片按片号从小到大并（`crash.rs` 调用线程里 `waiting_for_earlier_slices` 按 `next_slice_to_merge` 取），计数逐项相加，「第一处违例」与 checker 每条不变量的第一处都取最早那一片（`absorb_following_slice`，`crash.rs:1103` 起，字段按解构写全，新字段不并编译不过）；观察者在调用线程上按序号调用。`tests/second_transaction_step_zero_layer0.rs:683-692` 拿 1 个与 8 个线程比整份计数。
- **状态评估时 panic**：工作线程 panic → `RaiseFlagWhenPanicking` 立旗、别的线程不再领片（`crash.rs:2285`）→ `std::thread::scope` 在收尾时把 panic 抛出（`crash.rs:2273`），整条用例判红，进度文件由 `DeleteTheProgressFileWhenPanicking` 删掉。那个状态不会被记成「通过」；缺点只是 panic 信息里没有状态序号与持久集合，要从最后几行 `LAYER0_PROGRESS` 倒推是哪一片。
- **错误被吞、通配臂**：五个文件里 `let _ =` 只有 `lib.rs:291`（往 `String` 里 `writeln!`，不会失败）；没有 `.ok()`；`unwrap_or` 取的是默认计数、空区间（容量墙「没有这一次」就不放行，`model.rs:1547`）或「还没发过就用现行根」（`model.rs:1471`），都不是吞错；`_ =>` 零命中。`SystemConfiguration::parse_slot` 解不开时不抬 F 界（`crash.rs:907`），方向是更严，会假红不会假绿。
- **对拍的时机与定位**：每一步都 `judge_and_advance`（`history.rs:778`），不是只在末尾；`ModelDisagreement` 本身不带步号，步号由 `history.rs` 按 `StepPosition` 报（`history.rs:3810-3817`，那个文件不在本次范围，只核了调用点）。

## 崩溃点枚举的状态空间（按代码）

**枚举了什么**（`crash.rs:471-534` 切段，`crash.rs:1773-1825` 取状态，`crash.rs:1406-1505` 评一个状态）：

| 维度 | 代码怎么做 |
|---|---|
| 被枚举的流 | 层 0 用例取 mkfs 之后那一段（如 `tests/second_transaction_step_zero_layer0.rs:187-191` 的 `operations[pool.mkfs_operation_count..]`），基镜像 = mkfs 之后 |
| 段边界 | 任何一块盘的屏障关掉当前段（当前段非空时）；FUA 写关掉它自己所在的段；普通写与整段清零只进段 |
| 一个状态 | 前面各段全落 + 当前段一个真子集（含空）+ 后面各段全不落；最后加「全部落」一个 |
| 粒度 | 一次整写（含两块盘的两份各算一次）；没落的位置是基镜像加更早已落的写 |
| 每个状态上跑什么 | `recover` 两遍（看 / 不看 journal，只读）→ oracle（两遍都判）→ checker 判崩溃态镜像 → 记录核对器（按持久集合） |
| 平时快档（甲二） | 原地写任意子集 × 单元写全落或全不落，去掉整段全落（`crash.rs:1626-1656`） |
| 挑段 | `enumerate_layer0_selecting*` 可以让某些段不展开（只以整段落进入后面的状态） |

**省略了什么**：

| 省略的一类 | 代码依据 | 承认了没有 |
|---|---|---|
| 撕裂写（一次写只落一部分扇区），含 768 MiB 整段清零只落一部分 | `CrashImage::read` 整写叠加 | D13（验证路线） 已定项 4 承认；但它的理由不罩原地覆写（第 3 条） |
| 一块盘的屏障落了、另一块没落 | 录制器并屏障 + 切段不看设备（第 1 条） | 没承认 |
| FUA 之前、同段别的盘上的写在 FUA 之后仍没落 | FUA 关段按池算（第 4 条） | 没承认（D13（验证路线） 已定项 4 只写了 FUA 是段边界） |
| 同段里重叠的两次写按相反次序到达 | `CrashImage::read` 按写表次序叠（`crash.rs:398`），也没有「同段写不重叠」的断言 | 没承认；今天的流里同段写不重叠（按发布路径的写序推的，没逐流核） |
| 崩溃之后的可写挂载、恢复中途再崩 | 只跑只读 `recover`（第 2 条） | 与 D13（验证路线） 已定项 7 定案相反 |
| mkfs 自己那一段 | 基镜像取 mkfs 之后 | 用例层面的选择，模块里没写 |
| 设备出错、说谎设备 | 录制器只录成功的写（第 11 条） | 归故障注入（不在本次范围） |
| 写数 ≥ 64 的段 | `(1u64 << segment.len())`（`crash.rs:539`、`crash.rs:1643`）在 `overflow-checks = true`（`Cargo.toml:17`）下 panic | 响亮失败，不会静默少枚举 |
| 平时快档的单元写只落一部分 | 甲二定义 | D13（验证路线） 已定项 9 射程承认（`13-验证路线.md:188`），提交时跑全量 |

「枚举了 N 个」与「实际可达多于 N」：闭式只数上表第一张的那些状态；第二张表的每一行都是真设备上可达、而 N 里没有的状态。

## 模型与实现共用的符号清单

**`model.rs`（理想模型）**：`use` 只有 `std` 与 `singlefs_format`（`model.rs:19-28`；`model_comparison.rs` 里的用例 `the_model_module_uses_only_the_standard_library_and_the_format_constants` 逐行核这一条）。从 `singlefs_format` 取的：

| 符号 | 形态 | 实现也用吗 |
|---|---|---|
| `index_node_header_bytes` | **const fn，带算术** | 用：`singlefs-core/src/unit.rs:127/155/353`；checker 也用：`singlefs-checker/src/lib.rs:314`（第 6 条） |
| `ACCOUNTING_ENTRY_BYTES`、`ACCOUNTING_KEY_BYTES`、`ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`、`ALLOCATION_RECORD_TREE_LEAF_SLOTS`、`CLUSTER_SEGMENT_SLOTS`、`DATA_UNIT_BYTES`、`DATA_UNIT_PAYLOAD_OFFSET`、`INSTANCE_TABLE_PAGE_RECORDS`、`NODE_BYTES`、`ROOT_RING_REGIONS`、`ROOT_RING_REGION_DEVICES`、`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`、`SLOT_BYTES`、`UNIT_AREA_START_SLOT` | 标量常量 | 属 D13（验证路线） 已定项 5 允许共享的那一类 |

模型里另写了一份的算术：根环落点（`model.rs:245-252`）、区域归属（`:254-258`）、实例表片数（`:117-123`）、分配记录树重写上界（`:828-871`）、F 生效值与抬 F 上限（`:717-799`）。其中分配记录树根的层号取法照实现员的交回写（第 9 条）。

**`model_comparison.rs`（胶水）**：用 `singlefs_core` 的 `address`、`allocator::PlacementRefusal`、`block_device::BlockDeviceError`、`inode_tree`、`mount::{InstanceRow, MountError, Mounted, RollbackCandidateExclusion, RollbackError}`、`recovery::{RecoveryOutcome, RecoveryReport}`、`root_ring::RootRingSlot`、`transaction::{PoolVersion, PublishError, TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput}`（`model_comparison.rs:9-21`）。只做类型换算与错误成员映射，没有算术；按 D13（验证路线） 已定项 5 的射程它不在「模型」之列。

**`crash.rs`（重放、oracle、记录核对器）**：

| 符号 | 来自 | 用在哪 | 性质 |
|---|---|---|---|
| `recover`、`PoolReader`、`JournalPolicy`、`RecoveryOutcome`、`RecoveryReport` | core `recovery` | 每个状态跑恢复 | 被测对象本身 |
| `SystemConfiguration::parse_slot` | core `system_configuration` | 记录核对器的回收谓词取系统配置里的 F（`crash.rs:907`） | 独立审计方用实现的解析器：两边对 F 的偏移一起错时，这个界一起偏 |
| `UnmountMarker`、`ROOT_RECORD_FLAG_UNMOUNT_MARKER` | core `root_record` | C557（卸载记号只由卸载入口打没有检查） 那一格检查（`crash.rs:33`、`:1323-1331`） | 核「记号只由卸载入口打」时用实现的位定义 |
| `check_pool_image`、`ImageReader`、`InvariantVerdict`、`InstanceTableOfRootRecord` | checker | 判镜像；记录核对器判被抛弃的发布（`crash.rs:15`） | checker 那一侧，不是实现 |
| `JOURNAL_RECORD_BYTES`、`JOURNAL_RING_DEFAULT_BYTES`、`JOURNAL_RING_START_SLOT`、`NODE_POINTER_BYTES`、`SLOT_BYTES`、`UNIT_AREA_START_SLOT` | format | 标量 | 允许 |
| 根身份偏移 24 / 28、flags 偏移 20、F 偏移 `36 + NODE_POINTER_BYTES + 8` | 手写 | `crash.rs:1304-1316`、`:1323-1331`、`:854` | 另写一份，没有与 checker 的读法连起来的断言（`:853` 注释说 `singlefs_checker::check_root_slot` 读同一个偏移） |

**`segments.rs`（写的分类）**：`singlefs_core::root_ring::{region_start, ring_end, RootRingSlotsPerRegion}`（`segments.rs:16`）。哪次写算「根槽写」由实现的根环几何算术定（`segments.rs:71-78`），而 oracle 的「盘上已持久的最新根」、记录核对器的 `publishes_in`、C557（卸载记号只由卸载入口打没有检查） 那一格检查、甲二的原地写集合都骑在这一分类上；也没有「FUA 写 ⇔ 落在根环里」的交叉检查。

**`lib.rs`（录制器）**：只用 core 的 `address` 与 `block_device` 接口（被包的接口本身）。

## 注释与代码不一致清单

| # | 位置 | 注释说 | 代码 / 别处说 |
|---|---|---|---|
| 1 | `model_comparison.rs:326` | 「树表 0 条、而实例表已经不是 mkfs 那一片：零故障走得到……模型照代码今天的读法划进必须拒」 | `model.rs:1345-1353`：C512（树表 0 条的一版上被换下的单元记在哪） 定案之后「模型因此**一条都不列**……随机历史里造不出来」；`VersionWithoutFileNotWrittenByMakeFilesystem` 在模型里从不进必须拒或允许拒 |
| 2 | `model.rs:289-295`（`VersionWithoutFileNotWrittenByMakeFilesystem` 的文档） | 「第一版不支持（设计空白，交主 agent）」 | 同上，`model.rs:1345` 起写「这一格不再拒」 |
| 3 | `model.rs:9` | 「第 3 件（崩溃注入）接上时要加「这个崩溃状态恢复到的版本在不在允许集合里」」（将来时） | 已经加了：`crash_recovery_disagreement`（`model.rs:1956`） |
| 4 | `model.rs:685-687` | 「一步至多写出四条根（……抬 F 两次空发布）」 | `answer_raise_rollback_floor` 的循环上限是 `ROOT_RING_REGIONS` = 3（`model.rs:1464-1470`）；两块盘、区域归属 0 / 1 / 0 时，从 txg ≡ 1 (mod 3) 起抬 F 要三次（下一条落盘 0、再下一条仍落盘 0、第三条才落盘 1）。「至多四条」这个界仍成立 |
| 5 | `model.rs:376` | `permitted_refusals`：「照代码今天的读法划的「第一版不支持」区」 | 每个构造处都给空集（`model.rs:1017`、`:1057`、`:1089`、`:1305`、`:1449`、`:1517`，新实例那一处在 `:1411`），这一格是死的 |
| 6 | `segments.rs:91` | 「FUA 不替前面的普通写做持久，所以它们同段、任意子集」 | 关段之后后面各段都把它们当已持久（第 4 条） |
| 7 | `crash.rs:569` | 「录制流只收真的记录写，环没有整段写 0 的那一次」 | 录制器把整段清零记成一步（`lib.rs:38-40`、`:355-376`）；代码结果仍对，因为 `zero_fill` 删扇区 |
| 8 | `lib.rs:3` | 每条记「种类（普通写 / FUA 写 / 屏障）」 | 种类有四种，还有 `WriteZeroes`（`lib.rs:38-40`） |
| 9 | `crash.rs:668-670` | 「崩溃后镜像是 `image` 本身」 | D13（验证路线） 已定项 7（`13-验证路线.md:131`）：比对对象是实现恢复后的镜像（第 2 条） |
| 10 | `crash.rs:348-357`、`:551-557` 的 reader 实现 | trait 文档：越界 / 没有这块盘返回 `None`（core `recovery.rs:80-81`、checker `image.rs:20`） | 越界返回 `Some(全 0)`，不对齐 panic（第 5 条） |
| 11 | `13-验证路线.md:73`（D13（验证路线） 已定项 4 射程） | 「撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率」 | 原地覆写撕裂时旧内容也丢了（第 3 条） |

## 读过的文件与行数

审阅范围内的五个文件（行数按快照）：

| 文件 | 行数 | 怎么读的 |
|---|---|---|
| `crates/singlefs-harness/src/lib.rs` | 629 | 1–629 全读 |
| `crates/singlefs-harness/src/segments.rs` | 349 | 1–349 全读 |
| `crates/singlefs-harness/src/crash.rs` | 3120（开读时 3087） | 改动前版本 1–2700 逐段全读、2700–3087 的测试先按函数名与断言过了一遍；发现文件在变之后拷快照，在快照上重读 1–36、625–930（记录核对器改动处）、1406–1505、2735–3120，并在快照上逐条核了下文引用的行号 |
| `crates/singlefs-harness/src/model.rs` | 2395（开读时 2393） | 改动前版本 1–2083 逐段全读；快照上重读 1571–1680、1871–1910 与测试 2085–2395 全部 |
| `crates/singlefs-harness/src/model_comparison.rs` | 610（开读时 609） | 改动前版本 1–609 全读；快照上核了 326 行与 `git diff HEAD` |

顺带读的（只读了列出的段）：`crates/singlefs-core/src/block_device.rs` 1–140、209–241、277–335；`crates/singlefs-core/src/transaction.rs` 240–420、540–555、662–676、830–860、960–1010、2207–2247；`crates/singlefs-core/src/recovery.rs` 74–140、229–290、1826–1886、2794–2870；`crates/singlefs-core/src/make_filesystem.rs` 282–294、360–370；`crates/singlefs-harness/src/layer0_progress.rs` 1–140；`crates/singlefs-harness/src/crash_injection.rs` 1–30 与 grep；`crates/singlefs-harness/src/history.rs` 790–812 与 grep；`crates/singlefs-harness/tests/common/mod.rs` 90–130、240–290；`tests/second_transaction_step_zero_layer0.rs` 170–200、467–560；`tests/second_transaction_parallel_line_one_layer0.rs` 225–260；`tests/second_transaction_supplement_two_tree_identifier_watermark_crash_orphans.rs` 1–40；`.claude/kb/decisions/13-验证路线.md` 12、30–34、69–140、188；`crates/singlefs-format/src/lib.rs` 若干常量行与 44–65、212–225；`Cargo.toml`；`.claude/gate.d/54-layer0-replay.sh` 的 grep；`crates/mutations.tsv` 的 grep。

没读：`.claude/kb/verification-build.md`（派发提示列为按需，本次的判断没有用到它）。`crash_injection.rs`、`history.rs`、`fault_injection.rs` 不在审阅范围，只核了与上面几条有关的调用点。
