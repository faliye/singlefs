# 事务层审阅报告：crates/singlefs-core/src/transaction.rs

结论：发布路径本身的持久次序（单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置轮换）、冻结重发、失败时分配器换回，逐行核下来是对的，没找到「引用者先于被引用者落盘」或「屏障之前就把新根当已发布」的路径，也没找到对仍被旧根引用的单元就地覆写。报出的问题集中在：发布之外的几次系统配置写与上一次轮换之间没有屏障（两槽轮换失去防撕裂的前提）、一条释放路径漏了逐盘核（盘上数据不对称时 panic）、几处过时的文档注释与死代码。没有「高」。

---

### 1. 取号、抬 F 先写系统配置、取号回卷这三种系统配置写，与上一次发布末尾的轮换写之间没有屏障：一次撕裂可以把一块盘上两个槽里较新的两代都丢掉
- 位置：transaction.rs:313-325（轮换之后不发屏障）、654-675（`write_acquired_instance` 第一个写之前不发屏障）、524-557（`write_the_raised_floor_into_every_system_configuration` 同）、426-457 与 671-673（屏障报错之后回卷，回卷之前、之后都不发屏障）
- 严重度：中
- 置信度：机制已确认（代码里没有这道屏障；`.claude/kb/layout/01-first-txn.md:412` 自己登记了「抬 F 先写系统配置、与 txg 14 那次轮换合成 4 写一段」「第 13 段是 B 的 2 个系统配置槽写与重开取号的 2 个系统配置槽写合成的 4 写一段」）；后果疑似：取决于撕裂写，而 D13（验证路线） 已定项 4 把撕裂态并进「没持久、放旧字节」，层 0 枚举不出这一格
- 代码做了什么：
  ```
  CommitStep::RotateSystemConfigurationSlots { .. } => {
      for index in 0..self.devices.len() {
          self.write_system_configuration_slot(index, journal_tail, journal_instance, ...)?;
  ```
  （313-325，之后 `persist_publish_writes` 直接返回，没有屏障）；
  ```
  for index in 0..pool.devices.len() {
      if let Err(cause) = pool.write_system_configuration_slot(index, 0, instance, ...
  ```
  （657-668，第一个写之前没有 `CommitStep::Barrier`）
- 应该做什么 / 为什么错：两槽轮换（D22（单元原子性怎么合成） 已定项 16 / 21）能扛住一次撕裂的前提是「覆写较旧那一槽时，较新那一槽已经持久」。发布与发布之间靠下一次发布开头那道屏障满足它；但轮换之后紧跟的若是取号（同一进程里卸载再挂载）、抬 F 的先写系统配置、或取号回卷，中间一道屏障都没有。世代号推演：轮换写 g 进槽 X（盖掉 g−2），接着的那次写 g+1 进槽 Y（盖掉 g−1，也就是此刻唯一已持久的最新一代）。应在这三处第一次系统配置写之前发一道 `CommitStep::Barrier`（写入口的 `has_writes_since_barrier` 会让它在前面没写时自动省掉），或者在每次轮换之后补一道。改屏障位置会改段序列登记表（门禁 52 号），归主 agent 定。
- 失败场景：抬 F 两次：第一次 SysPre 写 k+1（F1，屏障）→ 空发布，根带 F1，轮换写 k+2（没持久）→ 第二次抬 F，SysPre 写 k+3 进 k+1 那一槽。此刻断电：k+2 没落、k+3 撕裂 ⇒ 这块盘只剩世代 k（F0），而同盘根环上有带 F1 的根 ⇒ I-7.12（系统配置 F 不低于同盘根上的 F） 红；若带 F1 的根也坏了，F1 在这块盘上彻底丢失，正是 `RollbackFloorOfASystemConfigurationWrite` 文档（491-495）与 SysPre 那一步要防的那一格。取号那一格：这块盘只剩 g−2，tail 倒退两代（挂载按「落后」逐单元核，多数情况放行）。
- 附：`AcquisitionRollback::RolledBack` 的文档（478）写「已写出的那几份都回卷成了旧代号」，而回卷写之后没有屏障，返回时只是发出、没有持久。

### 2. 第一个文件版本换下 mkfs 树表与无文件那一版的分配记录树时只核盘 0、不核跨度：两盘分配记录不对称时在分配器断言上 panic，而不是报错
- 位置：transaction.rs:2433-2475（`format_time_tree_table_to_release`），经 4799 进释放清单，在 4882（`settle_the_allocation_record_tree` 的拷贝上）与 5656（真发）调 `release`；panic 点在 allocator.rs:1043-1052
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```
  let still_allocated = |placement: &Placement| {
      allocator.devices.first()
          .and_then(|device_map| allocator.record_for(device_map.device, placement.slot))
          .is_some_and(|record| !record.is_released)
  };
  ```
- 应该做什么 / 为什么错：同一个文件里经映射释放的那一路专门走 `placement_to_release_after_checking_every_device`（2913-2955），注释写明「两块盘的分配记录树对不对称是盘上读来的、不是不变量（panic 面普查 R10）」。这一路是 R10 的漏网：只看盘 0，跨度取角色的（1）不与记录比。无文件那一版的分配记录由 `mount::allocator_of_version_without_file` 整棵读进来（mount.rs:944-962），`recovery::allocation_records_fit_the_pool_geometry`（recovery.rs:995-1030）只判几何，不判两盘对称。应改成对每个候选落点调 `placement_to_release_after_checking_every_device`，核不过报 `ReleaseTarget*` / `ReleaseSpanMismatch`。
- 失败场景：写过行、树表 0 条的池重开后调 `publish_first_file`；盘 1 的分配记录里缺 mkfs 树表那一槽（或已释放、或跨度 2，而盘 0 的正常）→ `settle_the_allocation_record_tree` 在拷贝上 `rehearsal.release` → `release_leaving_the_record_allocated_on` 的 `expect("每块盘上都有这个落点的分配记录…")` 或 `assert!(!record.is_released)` / `assert_eq!(span)` panic。盘 0 那条已释放、盘 1 那条未释放时反过来：静默跳过，盘 1 那一槽永远不释放。

### 3. 一块盘两槽都读不出时世代号从 1 起：一次瞬时读错就让这次写盖掉那块盘上最新的那一槽，而且写出的新内容世代号低于残留的旧槽
- 位置：transaction.rs:356-361（`write_system_configuration_slot`），写在 395-400
- 严重度：低
- 置信度：逻辑已确认；触发要两槽同时瞬时读错（疑似，要跑故障注入才能定）
- 代码做了什么：
  ```
  let slot_generation = verified_on_this_device.iter()
      .map(|system_configuration| system_configuration.quantities.slot_generation)
      .max().unwrap_or(0) + 1;
  ```
- 应该做什么 / 为什么错：D22（单元原子性怎么合成） 已定项 16 只写了「自证过的槽里最大的世代号 + 1」，没写两槽都读不出时怎么办。代码取 1、写进槽 1。两槽其实完好、只是这一次读错时：若最新那一槽的世代号是奇数，它就在槽 1，被这次写盖掉；而新写的内容世代号 1 低于槽 0 的旧世代号，按「择槽取校验和过且世代号最大的」读的一方（例 mount.rs 的 `devices_without_the_selected_version`，`max_by_key(slot_generation)`）拿到的是旧内容，下一次轮换又去盖世代号 1 那一槽（新内容）。读不出时拒写并报错，或至少不越过另一槽的世代号，二选一要主 agent 定；同文件 `acquire_expected_instance`（636-649）已经把「两次读之间一次瞬时读错」当成真实场景处理。
- 失败场景：盘 0 槽 1 = 世代 5（实例 2、tail 40），槽 0 = 世代 4；取号时两槽各报一次读错 → 写世代 1（实例 3）进槽 1，世代 5 被盖掉 → 之后按世代号择槽的读者读到世代 4（比盖掉之前还旧一代）。

### 4. 发布入口不核「接在现行最新那一版上、同一个实例」：给一对旧而自洽的根与记录时，装了根环表会 panic，没装会盖掉同 (实例, txg) 的旧根与同计数器的旧记录
- 位置：transaction.rs:4255-4292（`publish_first_file` 只核「记录的 txg = 根的 txg」）、4298-4307、4301；`publish_overwrite` 4349-4365、`publish_sequential_write` 4402-4440、`publish_new_inodes` 4460-4500 的 `instance` 参数与 `previous.root.instance` 不比
- 严重度：低（今天产品路径的调用方都传现行那一版；属于公开 API 没守的前置条件）
- 置信度：已确认
- 代码做了什么：
  ```
  let follows_directly = previous_record
      .is_some_and(|(previous_txg, _)| previous_txg == version_to_build_on.checkpoint_txg);
  ```
- 应该做什么 / 为什么错：`FirstFileVersionDoesNotFollowTheVersionItBuildsOn` 挡的是「根与记录不是同一版」，挡不住「是同一版、但不是最新那一版」。新根 txg = 那一版 + 1、jsn = 那条记录 + 1，环里若已有更新的根，就盖在已有的 (实例, txg) 上——`FrozenPublish` 文档（863-866）要防的正是「同一 (实例代号, checkpoint_txg) 先后写出两次不同的发布」。另外 `highest_transaction_number_before_this_publish: 0`、`back_chain: back_chain_of(previous_record_bytes)` 默认上一条记录与这次同一实例；传进来的记录若是上一个实例的，本实例第一条的反向链就不是 0（I-8.6（反向链算法））。可以拿 `allocator` 上根环表记到的 txg 与实例核一遍，核不上报错。
- 失败场景：分配器根环表已记到 txg 4，调用方拿 txg 2 的暖机根与它那条记录调 `publish_first_file` → 5676 行 `record_root_written_by_this_process(CheckpointTxg(3))` 的 `assert_eq!` panic（allocator.rs:1387-1394）；分配器没装根环表时 txg 3 的根与 jsn 3 的记录被改写成另一份内容。

### 5. 几处「盘上读来的值 + 1」没有用 checked 运算，release 也开了 overflow-checks，坏镜像直接 panic
- 位置：transaction.rs:597-601（实例代号，u32）、356-361（槽世代号）、4298 / 4356 / 4409 / 4468（`checkpoint_txg.0 + 1`）、4478 与 5827-5831（inode 号水位 + N）
- 严重度：低
- 置信度：已确认（运算与来源都读过；实际要一份校验和对得上、字段取极值的镜像）
- 代码做了什么：
  ```
  InstanceGeneration(highest_system_configuration_instance(pool).max(highest_root).0 + 1)
  ```
  `inode: first_new_inode_number + offset_from_the_watermark,`
- 应该做什么 / 为什么错：同一文件里 `FileVersionTreeIdentifiers::issued_from_watermark`（2131-2140）对盘上读来的树 ID 水位用了 `checked_add` 并报 `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`，这几处同类的量没有。实例代号是系统配置与根记录里的 4 字节（system_configuration.rs:465），inode 号水位是记账树里的 8 字节，recovery.rs 里没有判它们离上界多远（只判了水位那一行在不在，recovery.rs:1539）。
- 失败场景：一块盘的系统配置槽自证过、实例代号写着 0xFFFFFFFF → 可写挂载算 `instance_generation_to_acquire` 时溢出 panic；记账行 inode 号水位是 `u64::MAX` → `publish_new_inodes` 在 4478 panic。

### 6. 一次发布的 journal 记录数没有上限：顺序写 N 个数据单元就写 N 条记录，不按系统配置里登记的在飞上限截；journal.rs 里声称存在的那道截断已经没有了
- 位置：transaction.rs:4933-4972（`roles_named_by_each_record_of_the_publish`，条数 = 数据单元数 − 1 + ⌈末事务点名项 ÷ 67⌉）、6290-6337（逐条装记录）、4397-4399（文档引的错误成员已不存在）；journal.rs:76-87（`of_record_at_offset` 的 expect 与 `# Panics`）；对照 system_configuration.rs:367-370（系统配置写进在飞上限）、recovery.rs:1958-1974（重放只取前 `in_flight_limit` 条）
- 严重度：低
- 置信度：缺截断已确认（transaction.rs 里没有一处读 `journal_in_flight_record_limit` 或环槽数）；后果疑似
- 代码做了什么：journal.rs:84-85
  ```
  u32::try_from(record_offset_in_this_publish + 1)
      .expect("一次发布的记录条数在落盘之前按 extent 叶容量截过，序号装得进 32 位"),
  ```
  而 transaction.rs:2993-2994 说 extent 树「不分裂、没有装不下这一格……此前那两个成员随之去掉」，4397 的 `# Errors` 还写着 `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`（全仓 grep 只有这一处）。
- 应该做什么 / 为什么错：I-8.3（重放前缀严格连续） 的补句要求「一次发布整体施加或整体不施加」且「这个前缀最多 N 条」（N = 环槽数 ÷ 3，默认 65536）。记录数超过 N 的发布在「记录已落、根没落」时恢复永远走不到它的末条，整体不施加；超过环槽数（默认 196608）时同一次发布自己的记录互相覆盖、并盖掉所选根那次发布的锚点记录。一致性不破（整体不施加），但写者与系统配置里登记的上限对不上，而 journal.rs 的断言消息拿一道已经不存在的截断当依据。要么在准入之前按在飞上限拒掉过大的写请求，要么把那句断言消息和 4397 的文档改成现状。
- 失败场景：默认几何（每盘单元区 211968 槽）上顺序写 65537 个数据单元（约 2 GiB）：记录 65537 条，落盘到第二道屏障之后、根槽 FUA 之前断电 → 恢复取前 65536 条，走不到带末条标志的那一条，这次发布不施加；记录数上到 196609 条时 jsn c+196609 落在 jsn c+1 那一槽。

### 7. 系统配置的「不可变」段每次写都按调用方给的 mkfs 参数重装，不照抄盘上自证过的那一份
- 位置：transaction.rs:376-383（`write_system_configuration_slot`）；另见 281、296-307（journal 记录与根槽的落位也取 `self.parameters.geometry`）
- 严重度：低（今天调用方都拿同一份参数 mkfs 与挂载；是调用方给错参数时的静默改写）
- 置信度：已确认（transaction.rs 这一半）；`mount_writable`（mount.rs:2801-2844）按盘上的系统配置扫环与重放，却不拿调用方的 `parameters` 与它比，是现查 grep 的结果
- 代码做了什么：
  ```
  immutable: SystemImmutableConfiguration {
      filesystem_identifier: self.parameters.filesystem_identifier,
      this_device: identity,
      device_count: u32::try_from(self.devices.len()).expect("设备数"),
      region_devices: self.parameters.region_devices,
      sizes: self.parameters.geometry,
  ```
- 应该做什么 / 为什么错：不可变段的来源应该是盘上那一份（或写入口在构造时核过两者相等）。现在每次取号、轮换、抬 F 都拿调用方参数与写入口里的设备数重写它；参数与盘不一致时，第一次轮换就把不可变段改掉，journal 记录也按调用方的环大小算落位，而恢复按盘上系统配置的环大小扫（recovery.rs:1833）。
- 失败场景：调用方传的 `journal_ring_bytes` 与 mkfs 时不同 → 记录落在 `(计数器 − 1) mod 调用方槽数` 的偏移上，恢复按盘上槽数找不到它们；同一次轮换把系统配置里的环大小改成调用方那个值。

### 8. 「文件变短时多出来的数据单元」按「这次重写的数据单元是 0..n 的前缀」推，没有断言钉住
- 位置：transaction.rs:2573-2599（`roles_replaced_via_mapping`）
- 严重度：低（今天走不到：`split_sequential_write_into_one_unit_transactions` 恒从单元 0 连号切，extent 计划恒整段重写下段）
- 置信度：已确认（潜在）
- 代码做了什么：
  ```
  let rewritten_data_units = roles.iter().filter(|identity| matches!(identity, TransactionUnit::Data(_))).count();
  ...
  (rewritten_data_units..previous_data_units).map(|position| TransactionUnit::Data(DataUnitIndexInFile(...)))
  ```
- 应该做什么 / 为什么错：它只数个数、不看序号。将来一旦出现只重写文件中间某一个单元的发布（部分覆盖写），数到 1 就会把单元 1..n 当成「没有继任者」经映射释放，而它们仍被新版本引用——那是一次 COW 破坏。在这里断言重写的 `Data` 角色恰好是 `0..rewritten_data_units`，或者按序号集合求差。
- 失败场景：（今天构造不出）上一版 5 个单元，一次发布只重写单元 3 ⇒ 释放清单里有单元 1、2、3、4，其中 1、2、4 这一版照抄、仍被引用。

### 9. 持久次序只由 `persist_publish_writes` 一个函数的语句次序维持；`PoolWriter::perform` 是 `pub`，取号与抬 F 的系统配置写不经 `CommitStep`
- 位置：transaction.rs:264（`pub fn perform`）、833-857（`persist_publish_writes`）、660-668 与 529-534（直接调私有的 `write_system_configuration_slot`）
- 严重度：低
- 置信度：已确认（全仓 grep：`perform` 在 transaction.rs 之外只有 `singlefs-harness/tests/instance_acquisition.rs:185` 一处调用）
- 代码做了什么：`pub fn perform(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError>`，逐步执行、不记录「上一步是什么」。
- 应该做什么 / 为什么错：`CommitStep` 的 `match` 是穷举的、没有通配臂（269-334 已核），但「状态机」只存在于 `persist_publish_writes` 的语句顺序里：任何拿得到 `PoolWriter` 的调用方都能不经屏障直接发 `WriteRootRecordForceUnitAccess`。`.claude/rules/fs-design.md`「一个事务层，所有结构共用」要的是「绕开状态机的第二条发布路径不允许」。把 `perform` 收成 `pub(crate)`（测试改走 `persist_*` 或专给测试的入口），就把这条路关上。
- 失败场景：（今天只有测试这么用）外部调用方依次发 `WriteUnitToEveryDevice`、`WriteRootRecordForceUnitAccess`：根在单元持久之前 FUA 落盘，崩溃后根指着没落的单元。

### 10. 死代码：`PublishShape` 整个类型、`ResolvedPublish::shape`、`ExtentTreePlan::roles`
- 位置：transaction.rs:3268-3346（`PublishShape` 与它的 `ROW_PUBLISH`、`EMPTY_PUBLISH`、`row_publish_rewriting_instance_table_pages`、`rewritten_roles`）、3372-3388（`ResolvedPublish::shape`）、3608-3621（`ExtentTreePlan::roles`）
- 严重度：低
- 置信度：已确认（`grep -rn "PublishShape\|EMPTY_PUBLISH\b\|ROW_PUBLISH\b" --include=*.rs crates` 只命中 transaction.rs 自身；`\.shape()` 与 `\.roles()` 在全仓零调用；都是 `pub`，编译器不报 never used）
- 代码做了什么：`pub fn rewritten_roles(self) -> Vec<TransactionUnit>`（3318）按形状排角色，文档说「可写挂载要在取号之前算写行那次发布的准入」。
- 应该做什么 / 为什么错：mount.rs 的取号前预演已改走 `prepare_the_version_publish` / `prepare_the_row_publish_on_a_version_without_file`，这套形状没人读；而且它漏掉了多层树根之下的节点（记账树、映射树、extent 下段、分配记录树根之下），真被拿去算准入会少算需求。`.claude/singlefs-ai-sop/rules/code-discipline.md`「用不到的代码删掉」。
- 失败场景：无运行时失败；风险是后来的人照 `EMPTY_PUBLISH` 的文档去找那条「钉住两者相等」的断言（见不一致清单第 5 条），或拿 `PublishShape::rewritten_roles` 算准入。

### 11. 单测名字声称的比断言多
- 位置：transaction.rs:6799-6844（`every_transaction_unit_names_its_class_tree_and_placement_rule`）
- 严重度：低
- 置信度：已确认
- 代码做了什么：名字里有 class，函数体里没有一条 `unit_class()` 断言；第一条 `assert_eq!(identity.tag().len(), 2)` 对 `IN_BUMP_ORDER` 的八个 `"t1".."t8"` 恒真（末尾那条整列比较已经覆盖它）。
- 应该做什么 / 为什么错：补一条钉 `unit_class()` 绝对值的断言（例如 `Data` → `UNIT_CLASS_DATA`、`InodeLeafContainer` / `InstanceTable` → `UNIT_CLASS_PACKED`、其余 → `UNIT_CLASS_INDEX_NODE`），或把名字改成它实际测的东西。`unit_class()` 在 harness 的 `first_transaction_step_five_publish.rs` 里有用到，但这个文件里名为 class 的这条测试一格都没钉。
- 失败场景：把 `unit_class()` 里 `InstanceTable` 那一臂改成 `UNIT_CLASS_INDEX_NODE`，这条测试照样绿。

---

## 发布路径的写出次序表（按代码逐步列）

带单元的发布（`publish_version` 一族、`publish_instance_table_on_version_without_file`）、零单元发布（`publish_without_units`）、冻结重发（`resend_the_frozen_publish`）三条都落到 `persist_publish_writes`（transaction.rs:833-857）与 `persist_the_root_then_rotate_the_system_configuration`（981-996）。每一步经 `PoolWriter::perform`（264-336）。

| 步 | 写什么 | 发到哪 | 耐久 | 之后有没有屏障 / FUA |
|---|---|---|---|---|
| 0 | （落盘之前）释放、取落点、`record_root_written_by_this_process`、装单元与记录、`note_/forget_allocation_record_tree_…`：只动内存里的分配器 | — | — | 失败时分配器由调用方换回进来时那一份（4648、1166）；落盘阶段失败时换回并冻结（920-938） |
| 1 | 这次重写的每个单元，按 bump 次序；每个单元依次写每块盘（275） | `slot.to_device_offset()`，每块盘同一槽 | Plain | 单元之间无屏障 |
| 2 | `CommitStep::Barrier`（844）：每块盘 `barrier()`（330，文件后端 = `sync_data`） | 每块盘 | — | **有**。`has_writes_since_barrier` 为假时整道省掉——只在单元为空、且上一道屏障之后这个写入口没发过写时（零单元发布紧跟取号那道屏障）才会为假 |
| 3 | 这次的每条 journal 记录，按计数器升序；每条依次写每块盘（283） | `record_offset(counter, 环字节数)`（journal.rs:178-183） | Plain | 记录之间无屏障 |
| 4 | `CommitStep::Barrier`（851） | 每块盘 | — | **有**（记录恒至少一条，这一道恒发） |
| 5 | 根槽（整槽，`root.to_slot(physical_block_size)`）（303-310） | 只写区域 `txg mod 3` 归属的那一块盘，槽 `(txg div 3) mod 8` | **FUA**（文件后端 = 写 + `sync_data`，block_device.rs:297-304） | FUA 自己关掉这一段 |
| 6 | 系统配置槽轮换（313-325 → 341-403）：每块盘现读两槽、世代号 = 最大 + 1、F = 现算整池生效值，写世代号 mod 2 那一槽 | 每块盘 | Plain | **没有**。靠下一次发布第 2 步那道屏障冲刷；若下一次系统配置写是取号 / 抬 F / 回卷，中间没有屏障（问题 1） |

发布之外的系统配置写：

| 路径 | 次序 | 屏障 |
|---|---|---|
| 取号 `write_acquired_instance`（654-675） | 逐盘写一次系统配置（tail 0、新实例代号）→ `Barrier` | 写之前无；写之后一道，报错 ⇒ 回卷 |
| 取号回卷 `roll_back_acquisition`（426-457） | 对已写出的盘逐盘再写一次（tail 0、旧实例代号），第一处报错即停 | 前后都没有 |
| 抬 F 先写系统配置 `write_the_raised_floor_into_every_system_configuration`（524-557） | 逐盘写一次系统配置（新 F）→ `Barrier` | 写之前无；写之后一道 |

结论：发布内部没有「引用者先于被引用者」的窗口——单元在第 2 步、记录在第 4 步各自持久之后，根才在第 5 步 FUA；新根在 FUA 返回之前不被当成已发布（分配器的根环表在第 0 步就推进，但失败时整个换回，下一次发布只在这次落盘成功之后才取落点）。唯一的缺口是第 6 步之后与下一次「非发布」系统配置写之间（问题 1）。

## 读过的文件与行数

| 文件 | 读了哪些行 |
|---|---|
| crates/singlefs-core/src/transaction.rs | 1-6959 全读（分 23 段 `sed -n`） |
| crates/singlefs-core/src/block_device.rs | 17-52、97-137（trait）、280-332、485-560（写 / 屏障 / FUA 的两个后端） |
| crates/singlefs-core/src/journal.rs | 70-88、167-183、185-260 |
| crates/singlefs-core/src/allocator.rs | 555-600、900-975、1015-1125、1286-1336、1370-1440 |
| crates/singlefs-core/src/write_request_split.rs | 1-120 |
| crates/singlefs-core/src/make_filesystem.rs | 125-145（`location_entries`） |
| crates/singlefs-core/src/mount.rs | 240-290、880-975、1525-1560、1660-1730、1780-1815、1870-1996、2300-2390、2801-2850 |
| crates/singlefs-core/src/recovery.rs | 919-995、995-1030、1093-1117、1804-1822、1895-2090 |
| crates/singlefs-core/src/system_configuration.rs、address.rs | grep 相关字段与类型宽度 |
| .claude/kb/decisions/17、18、22、23 | D17 已定项 2 全文；D18 已定项 11 第 313-314 行；D22 已定项 16；D23 grep「在飞」各行 |
| .claude/kb/invariants.md | I-8.1 至 I-8.9 各行 |
| .claude/kb/layout/01-first-txn.md | 第八节 390-412 |
| .claude/kb/decisions/13 | 已定项 4（撕裂态并进「没持久」） |

## 注释与代码不一致清单

| # | 位置 | 注释说的 | 代码 / 现状 |
|---|---|---|---|
| 1 | transaction.rs:3 | 「三条路径都从同一个枚举走：取号 = 逐盘一次系统配置槽写 + 一道屏障」 | 取号的写走私有的 `write_system_configuration_slot`（660-668），不经 `CommitStep`；只有那道屏障经枚举（671）。抬 F 先写系统配置（529-534）同样绕开 |
| 2 | transaction.rs:5、7 | 「第一个事务 = 单元写 × 8」「被换下的八个单元」 | 第一个事务现在写 12 个单元（`.claude/kb/layout/01-first-txn.md:83`「事务本身单元 12 × 2 盘 = 24（t1..t12）」）；覆盖写换下的角色数随多层树与按位置寻址的树变 |
| 3 | transaction.rs:1744-1745 | 「第一个事务写出的八个单元……字节表七的 t1..t8」 | 同第 2 条；`IN_BUMP_ORDER` 只列八种角色，不是八个单元 |
| 4 | transaction.rs:2249、4234、4342；2340 的 expect 消息 | 「覆盖写换下的上一版八个单元」「分配八个落点、装八个单元」「上一版的八个落点经映射释放」「八个文件 / 固定点角色每种一个」 | 同第 2 条 |
| 5 | transaction.rs:3302-3303 | `EMPTY_PUBLISH`：「`publish_empty_after` 交给 `publish_version` 的那张计划同形，那里有一条断言钉住两者相等」 | mount.rs:1956-1985 的 `publish_empty_after` 里没有这条断言；`PublishShape` 全仓无人引用（问题 10） |
| 6 | transaction.rs:4397-4399 | `publish_sequential_write` 的 `# Errors`：切出的单元多于一片 extent 叶 ⇒ `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided` | `PublishError` 没有这个成员（2993-2994 自己写了已去掉），全仓 grep 只剩这一处 |
| 7 | transaction.rs:4560 | `publish_version`：「先做准入（extent 树装得下这个文件的数据单元）」 | 准入现在是空间准入（D28（挂载期承诺量） 已定项 1，4715-4731）；extent 树不再有装不下这一格 |
| 8 | transaction.rs:4570-4572 | `publish_version` 的 `# Errors` 列了 `ContentExceedsDataUnit` | `publish_version` 自己从不报它，只有 `publish_version_holding_one_data_unit` / `…_of_trees_holding_one_data_unit` 两个包装报 |
| 9 | transaction.rs:110 | `CommitStep`：「五种步骤种类（D17（实现分层与第三方管道） 已定项 2）」 | D17 已定项 2 定的是六种（含整段清零）；整段清零只在 mkfs 里、不经这个枚举。枚举五个成员本身没错，引的条款数目对不上 |
| 10 | transaction.rs:478 | `AcquisitionRollback::RolledBack`：「已写出的那几份都回卷成了旧代号」 | 回卷写之后不发屏障（426-457），返回时只是发出、未持久（问题 1） |
| 11 | journal.rs:79-85（`of_record_at_offset`，不在审阅文件里，与本文件直接相关） | 「写者在任何落盘动作之前按一片 extent 叶的容量拒掉装不下的写」 | transaction.rs 里没有这道截断（问题 6） |
| 12 | transaction.rs:3268-3277 | `PublishShape` 文档：「可写挂载要在取号之前算写行那次发布的准入」 | 可写挂载的取号前预演走 `prepare_the_version_publish` / `prepare_the_row_publish_on_a_version_without_file`（mount.rs:2013 起），不读 `PublishShape` |

## 核过、没发现问题的几处（给核对用，不是问题）

- `CommitStep` 与 `TransactionUnit` 上的每个 `match` 都是穷举的，没有 `_ =>`（5378 那一处 `_ =>` 匹配的是整数层级，不是枚举）。
- 事务号 / 计数器 / 本次发布内序号：`transaction_offset_of_each_record_of_the_publish` 与 `roles_named_by_each_record_of_the_publish` 同一种切法，提交标记只在一个事务的最后一条（6307-6308）、末条标志只在真正的最后一条（6309-6313），序号 1..N 连号（6322-6324），tail = 末条计数器（6337、6366）；树表 0 条那一路同形（1534-1625）。`highest_transaction_number_in_this_instance` 取 max，空发布写 0 不推进。
- COW：带单元的发布只往分配器这次新取的落点写；换下的落点只进 defer，产品路径下同一次发布里取不回来；读盘核对不上的份留在已分配。冻结期间分配器是发布之前那一份，但任何发布入口第一道就拒（`refuse_while_a_publish_is_frozen`）。
- 释放不重复：映射那一路、实例表旧链那一路、第一个文件版本那一路三份清单互不相交（实例表角色在映射那一路 `continue`）。
- 失败回滚：`prepare_*` 里 `?` 之后分配器由调用方整个换回；`publish_admitted` 在 `record_root_written_by_this_process`（5676）之后再没有 `?`。
