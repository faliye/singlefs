# 冷启动恢复与三种树结构审阅报告

范围：`crates/singlefs-core/src/recovery.rs`、`extent_tree.rs`、`code_two_tree.rs`、`inode_tree.rs` 全文逐行读过；调用到的接口按需读（清单在文末）。只读，没跑 cargo、没跑脚本。
「已确认」= 读代码能把推理从输入走到结局；「疑似」= 有一步要跑才能定。行号是 2026-09-26 工作区现状。

## 发现（按严重度）

### 1. 择到 txg = u64::MAX 的根时 `replay_journal` 在加一上溢出 panic
- 位置：crates/singlefs-core/src/recovery.rs:1971（调用方：recovery.rs:2825 `recover`、mount.rs:2838 可写挂载、mounted_read.rs:554 挂载态读）
- 严重度：高
- 置信度：已确认
- 代码做了什么：
  ```rust
  let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
  ```
- 应该做什么 / 为什么错：`checkpoint_txg` 是根槽里读来的 8 字节（root_record.rs:130 `CheckpointTxg(reader.get_u64())`，`parse_slot` 不设界），只受 CRC32C 自证校验和保护。`choose_root`（recovery.rs:776-796）取 txg 最大的根，所以这样一条根一定被选中。release 开了 overflow-checks（Cargo.toml:17），这一行无条件执行（锚点读得出时也算），结局是 panic，不是 `RecoveryOutcome::Failed`。应该用 `checked_add`，溢出时按「链首接不上」处理（一条都不施加），或者报一个错误成员。
- 失败场景：根环里一条根自证过、`checkpoint_txg = 0xFFFF_FFFF_FFFF_FFFF`（写坏的实现或改出来的镜像）⇒ `recover(reader, JournalPolicy::Consult)`、`mount_writable` 都在 1971 行 panic；`JournalPolicy::Ignore` 不走这一行，不 panic。

### 2. 重建上一版不核码 2 叶的 key 次序，下一次发布在 `build_multi_level_tree` 的 assert 上 panic（可写挂载必经）
- 位置：code_two_tree.rs:1000-1011、1049-1058（两处 `if self.judges_every_header()`）；recovery.rs:497、1509（重建读映射树与记账树用 `OnlyWhatTheShapeNeeds`）；code_two_tree.rs:532（删 key 用二分）、412-424（插 key 按分隔 key 路由）；transaction.rs:6468-6475（assert）
- 严重度：高
- 置信度：已确认（推理读到 assert 为止，没跑）
- 代码做了什么：
  ```rust
  // code_two_tree.rs:1049 叶：只有 EveryHeaderAgainstItsReference 才判严格递增
  if self.judges_every_header() {
      let keys_ascend = header.entries.windows(2).all(|pair| self.key(&pair[0][..key_width]) < self.key(&pair[1][..key_width]));
  // transaction.rs:6468
  assert!(planned_keys.len() == leaf_entry_bytes_by_key.len() && planned_keys.iter().zip(leaf_entry_bytes_by_key.keys()).all(|(planned, built)| *planned == built), ...);
  ```
- 应该做什么 / 为什么错：可写挂载（mount.rs:2827-2905）只走 `replay_journal` → `rebuild_version`，从不调 `walk_to_file`（mount.rs 里没有一处 `walk_to_file`）。`rebuild_version` 读记账树、映射树用 `OnlyWhatTheShapeNeeds`，叶里 key 重复或乱序、分隔 key 不递增都照收，交进 `TransactionOutput` 的形状里（`flatten_read` 按盘上次序收 key）。规划那一步假定叶有序且唯一：`previous.keys_in_order()` 收成 `BTreeSet` 去了重，`delete_below` 对每个 key 只删一次、用 `binary_search`；`build_multi_level_tree` 要求计划里叶的 key 与这次条目的 key 逐个相等。三种坏法都走到 assert，而不是 `CodeTwoTreeRefusal`。应该在重建路径上也判叶 key 严格递增、分隔 key 严格递增（这是「拼得成一棵有序树」的一部分，不是头的自描述），判红交 `RecoveryFailure`。
- 失败场景：
  - 记账树一片叶里有两条同 key 条目 K（CRC 都对）⇒ 可写挂载重建通过 ⇒ 可写挂载在取号之前先演练写行那次发布（mount.rs:2042 `prepare_the_version_publish` → transaction.rs:4753 `publish_admitted` → 5872 `build_multi_level_tree`），规划记账树时：记账 key 带代号、每次发布整批换 key，上一版每个 key 删一次，K 只删掉一份，另一份留在叶里 ⇒ `planned_keys` 比 `leaf_entry_bytes_by_key` 多一把 K ⇒ assert panic。
  - 映射树叶里两条条目乱序 [B, A]：照抄的 key 留在原位，中序是 B、A，条目按 BTreeMap 是 A、B ⇒ zip 不等 ⇒ assert panic（二分删 key 碰巧落空时才走到 `PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt` 这一格体面拒绝）。
  - 内部节点分隔 key 不递增：`route_for_insertion` 用 `rposition(separator <= key)` 把新 key 送进错的孩子，结果全局中序乱，同一个 assert。code_two_tree.rs:766-769 的文档说这类坏法「由规划那一步交回 Refusal」，只对删 key 那一支成立。

### 3. 可写挂载路径不跑 `walk_to_file`，`rebuild_version` 自己的判定只是走读的子集，写路径依赖的几条不变量在这条路上没人判
- 位置：recovery.rs:1289-1720（`rebuild_version`），recovery.rs:1467（注释），mount.rs:2837-2860；写路径依赖处 inode_tree.rs:193、259-263、268-272
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  // recovery.rs:1467
  // 身份取容器头那一份：条目里的身份引用与它逐字相等是 I-9.2（条目身份与子头相符），走读同款在 `walk_to_file` 判。
  ```
- 应该做什么 / 为什么错：这句注释假设走读在同一条路上先跑过，可写挂载不跑它。`walk_to_file` 判而 `rebuild_version` 不判的有：树表 key 宽 8（2433）、树 ID 低于水位 I-7.8（2463）、各树根与节点头的树 ID / fsid / 诞生不晚于根 / 出生序号（extent、分配记录树用 `OnlyWhatThePositionsNeed`，记账、映射用 `OnlyWhatTheShapeNeeds`，inode 根只 `parse_index_node`）、inode 根层级 1（I-9.1）、条目身份与叶头相符及记录宽 140 以外的叶头字段（I-9.2）、记录号 ≥ 分隔 key 与容器号（I-9.4）、分配记录跨度非 0 与代不晚于根、记账条目的代与 seq、映射条目数、每落点每盘一条。其中 inode 叶的次序直接喂给写路径：`write_records_into_leaf_containers` 用二分找已有记录、拿「最后一片的最后一条」当最大号。
- 失败场景：一片容器号 1 的叶里记录按盘上次序是 [9, 1]（各自 ≥ 容器号，CRC 对）⇒ 重建照收 ⇒ 下一次发布写 inode 9：`binary_search_by_key(&9)` 在 [9, 1] 上落空（中点是 1 < 9，往右找不到），当成新号；最大号取末条 = 1，9 > 1 ⇒ 追加 ⇒ 容器变成 [9, 1, 9]，同一个 inode 两条记录写上盘，没有报错。修法：重建路径要么复用走读的判定，要么至少判写路径依赖的那几条（叶内记录严格递增、容器号沿叶序严格递增、左容器最大号 < 右容器号，即 I-9.4 全句）。

### 4. 同一种树在树表里出现两条时，五个读者四种取法；同一个 inode 号出现两条时走读取末条、重建取首条
- 位置：recovery.rs:2475 / 2513 / 2518 / 2539（`walk_to_file` 后一条覆盖前一条）；recovery.rs:1317-1325、1330-1338（`rebuild_version` 的 `find` 取首条）；recovery.rs:1052-1057（`allocation_records_under_root` 取末条）；recovery.rs:1768-1775（`user_visible_tree_root_pointers` 报错）；mounted_read.rs:409（取首条非空的）；inode 记录：recovery.rs:2703-2705（末条）对 recovery.rs:1476-1481（首条）
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  // walk_to_file
  extent_tree = Some((entry.tree, extents_of_the_first_file(read)?));      // 2513，后来的覆盖先来的
  by_kind.insert(entry.kind, read_mapped_tree_root(...)?);                  // 2539
  // rebuild_version
  let pointer_of = |kind: u16| { tree_table_entries.iter().find(|entry| entry.kind == kind) ... };  // 1317
  // allocation_records_under_root
  if entry.kind == TREE_KIND_ALLOCATION { allocation_entry = Some(entry); } // 1056
  ```
- 应该做什么 / 为什么错：`TreeTableEntry::parse`（records.rs:313-339）不判种类，走读与重建都不判树表条目按 key（树 ID）严格递增、每种至多一条。同一份盘上字节被不同读者读成不同的「这一版」。应该在树表进来的那一处判「每种至多一条」（`user_visible_tree_root_pointers` 已经这么判），所有读者共用它。种类没登记的条目也各判各的：`walk_to_file` 在根为空时放过（2469-2471 的 `continue` 先于 2536 的种类判定），重建直接忽略，`user_visible_tree_root_pointers` 报错。
- 失败场景：树表里两条 `TREE_KIND_EXTENT`（树 ID 不同、各自自洽）⇒ `recover` 读回第二棵 extent 树指的内容；可写挂载重建、挂载态读都用第一棵；影子账（`allocation_records_under_root`）用第二棵分配记录树、重建用第一棵。第一个文件那条 inode 记录在两片叶里各一条时同理：`recover` 按末条的 size 拼内容，重建交给发布路径的是首条。

### 5. 重放不核反向链，与 I-8.6「不等的记录不进重放前缀」和 D23 已定项 7 的恢复算法不一致
- 位置：recovery.rs:1931-2084（`replay_journal` 全函数不读 `back_chain`）；journal.rs:288（解出来了）；recovery.rs:7（模块文档里前缀口径没有反向链）
- 严重度：中
- 置信度：已确认（代码不读这个字段）；影响面是推的
- 代码做了什么：
  ```rust
  let back_chain = reader.get_u32();   // journal.rs:288，之后 recovery.rs 里没有一处 back_chain
  ```
- 应该做什么 / 为什么错：invariants.md I-8.6 逐字「不等的记录不进重放前缀」；D23 已定项 7「恢复算法：在『jsn 严格连续 + 校验和 + 反向链』之后再加一步」。代码按实例过滤（1950-1955 只取所选根那个实例的记录），D23 已定项 8 写明实例代号覆盖之后反向链对分辨时间线是冗余的，所以现有历史里多半不出事；但 checker（crates/singlefs-checker/src/walk.rs:4045）判 I-8.6 的前缀与恢复施加的前缀在「同实例内链值对不上」那一格口径不同：一条校验和对、链值错的记录，checker 判它违例，恢复照样施加。要么在 `replay_journal` 里断在链值不等处（本实例第一条恒 0，之后等于前一条 311 字节头的 CRC32C），要么改 I-8.6 与 D23 已定项 7 的文字，二者选一，不能两边说法不同。
- 失败场景：同一实例里 jsn 13 的记录 CRC 对、`back_chain` 不等于 jsn 12 头的 CRC32C ⇒ checker I-8.6 红；`recover` 把 jsn 13 所在那次发布施加进来，交回的 effective_root 是它的新根段。

### 6. 系统配置里的尺寸字段不设界：`journal_ring_bytes` 决定一次分配多大的偏移表，`physical_block_size` 决定每次读根槽分配多大
- 位置：recovery.rs:1837-1845（`scan_journal`）；recovery.rs:762、765 与 recovery.rs:111（读根槽）；system_configuration.rs:450、452、462（原样读入，只有 S 判了区间）
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  (0..ring_bytes / JOURNAL_RECORD_BYTES)
      .map(|record_index| DeviceOffsetInBytes(ring_start.0 + record_index * JOURNAL_RECORD_BYTES))
      .collect()                                                              // 1840-1844
  let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");   // 762
  let mut buffer = vec![0u8; length];                                         // 111
  ```
- 应该做什么 / 为什么错：两个值都是自证过的系统配置槽里的盘上字段，`parse_slot` 不判范围。环大小应当对着设备大小与格式常量判（D23 已定项 23 第 3 条写了 mount 时校验环几何，I-8.1），根槽宽应当判在格式允许的几档之内，越界报 `RecoveryFailure` 而不是往下走。
- 失败场景：`journal_ring_bytes = 2^62` ⇒ `collect` 要 2^50 个元素的 Vec，分配失败直接 abort；`2^40` 量级 ⇒ 2 GiB 偏移表加 2^28 次读，恢复实际上卡死。`physical_block_size = 0xFFFF_FFFF` ⇒ 每个根槽先分配 4 GiB 再读，3 × S 个槽。`recover`、`mount_writable`、挂载态读（mounted_read.rs:553）都先走这两处。

### 7. 交进来的盘小于 784 MiB 时，分配记录几何判定在单元区槽数的减法上下溢 panic
- 位置：recovery.rs:1014；allocator.rs:235-237
- 严重度：中
- 置信度：已确认（按代码推到 panic；没跑）
- 代码做了什么：
  ```rust
  let unit_area_end_slot = UNIT_AREA_START_SLOT + unit_area_slots_of_device(device_bytes);   // recovery.rs:1014
  absolute_slot_count_of_device(device_bytes) - UNIT_AREA_START_SLOT                           // allocator.rs:236
  ```
- 应该做什么 / 为什么错：`device_bytes` 是读者报的实际盘大小，不是 mkfs 时的大小；单元区起点 50176 槽 × 16 KiB ≈ 784 MiB。这一道判的文档说它把「盘上读来的值」挡在分配器之外，而它自己在盘太小时先 panic。应该先判 `device_bytes / SLOT_BYTES >= UNIT_AREA_START_SLOT`，否则报 `AllocationRecordOutsideThePoolGeometry` 或一个新成员。
- 失败场景：两块 4 GiB 盘的池，盘 1 的镜像文件被截到 512 MiB 交进来。系统配置与根环（1/4/7 MiB）在 512 MiB 以内读得出；journal 环（16 MiB 起、默认 768 MiB）在盘 1 上后半段读不到，记录按「任一份自证过即算在」从盘 0 那一份读；单元读盘 1 那一份落空、退到盘 0 那一份；点名单元两份不全、一条记录都不施加；分配记录树几何按读者报的大小算，根层级仍是 2（4 GiB 两格 + 512 MiB 一格 = 3 ≤ 169，一层时 323 + 41 > 169），树读得下来 ⇒ 走到 1014 行，`32768 − 50176` 下溢 ⇒ panic。可写挂载的重建（recovery.rs:1502）同样走这一道。

### 8. 池成员按「交进来的盘」算，不按系统配置算：掉了一块盘（不交进来）时冷读一律失败，报成数据坏了
- 位置：recovery.rs:1004-1008（设备身份不在读者里 ⇒ 报「不在池里」）；recovery.rs:2477（`AllocationRecordTreeGeometry::of_reader(reader)`）；recovery.rs:2572-2589（每落点每盘一条、记账条目数 3 + 6 × 读者的盘数）
- 严重度：中
- 置信度：已确认「会失败」；报出来的是哪一个成员要跑才定（疑似：多半先在分配记录树按读者几何读的那一步红）
- 代码做了什么：
  ```rust
  let device_bytes = reader.device_size_in_bytes(record.device).ok_or(
      RecoveryFailure::AllocationRecordOutsideThePoolGeometry { what: "分配记录的设备身份不在池里" })?;   // 1004
  let device_identities = reader.device_identities();                                                    // 2572
  if roots.accounting.leaf_entries_in_key_order.len() != 3 + 6 * device_count {                          // 2584
  ```
- 应该做什么 / 为什么错：`choose_system_configuration` 明写要在「掉了那块盘」时照样择得出系统配置（recovery.rs:583-585，E87），`PoolReader::read` 的契约也把「没有那块盘」当「这一份不在」（recovery.rs:80）；每个单元两盘各一份，冷读本来读得下去。但几何、记账行数、每盘一条这三处把「池里有几块盘、哪几块」取成读者交进来的那张表，缺一块盘就与盘上的记录对不上。池成员应当取系统配置的 `device_count` / `region_devices`，缺席的盘按「这一份不在」处理。
- 失败场景：两盘池只把幸存的盘 1 交给 `recover` ⇒ 分配记录树几何只按盘 1 算（`child_named_by_entry_key` 对盘 0 的条目 `has_device` 为假）或走到 1004 行 ⇒ `Failed`，成员是「位置对不上 / 设备身份不在池里」这类坏数据成员。C519 的两个用例（harness `second_transaction_supplement_two_c519_whole_device_loss_after_warm_up.rs:150-178`）都把掉了的盘换成「每次读都报错」或「全 0 空盘」交进来，身份仍在读者里，所以没碰到这一格。

### 9. 槽 0 自证不过时按 4096 找槽 1：槽距大于 4096 的池上，两盘槽 0 同时撕掉就整池择不出系统配置
- 位置：recovery.rs:607-622
- 严重度：中
- 置信度：已确认（代码路径）；可达性要槽距 > 4096（mkfs 按 io_min 取整，D22 已定项 2 那一行「槽距 = 4096 向上取整到 io_min 的整数倍」）
- 代码做了什么：
  ```rust
  SystemConfigurationSlotReading::NotSelfDescribing
  | SystemConfigurationSlotReading::IncompatBitsNotRecognized(_) => {
      FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
  }
  ```
- 应该做什么 / 为什么错：槽距是池级不变量（两盘四槽同值）。一块盘的槽 0 坏了，别的盘自证过的槽里就有槽距；逐盘独立回落到 4096，会在槽 1 真在 8192（或更远）时读错位置。应该先扫一遍各盘的槽 0，拿任一自证过的槽里的槽距去找各盘的槽 1；都没有时再按 4096 以及格式允许的几档逐个试。
- 失败场景：io_min = 8192 的两盘池，系统配置轮换正写到两盘的槽 0 时掉电（世代号 mod 2 = 0，两盘同一步写同一个槽），两盘槽 0 都撕了、槽 1 完好在偏移 8192 ⇒ 两盘都去 4096 读槽 1、自证不过 ⇒ `NoValidSystemConfiguration`，池挂不上。

### 10. `allocation_records_are_one_per_device` 在盘表为空时走到 expect panic，消息说的前提不成立
- 位置：recovery.rs:2283-2317
- 严重度：低
- 置信度：已确认；`recover` 走不到（空池在择系统配置那一步就报 `SystemConfigurationsDisagree`），`walk_to_file` 是 `pub`、harness 直接调它（second_transaction_step_four_rollback.rs:920）
- 代码做了什么：
  ```rust
  if placements_per_device.len() != device_identities.len() || device_identities.iter().any(...) { return false; }
  let first_device_placements = placement_sets.next().expect("上面核过每块盘都有记录");
  ```
- 应该做什么 / 为什么错：记录与盘表都空时 0 == 0 通过前一判，`next()` 是 `None`。判据应当显式拒空盘表（返回 false），不靠 expect。
- 失败场景：读者 `device_identities()` 交空表、树表 0 条之外的根上分配记录也为空 ⇒ 调 `allocation_records_are_one_per_device(&[], &[])` ⇒ panic。

### 11. 重放不强制同一实例内 txg 与事务号单调、也不管「提交标记之后同一事务又来一条」
- 位置：recovery.rs:1974-2016（新开一次发布时只判序号为 1，锚点读得出时不判 txg）
- 严重度：低
- 置信度：已确认；只在校验和对得上的反常记录上才有后果
- 代码做了什么：
  ```rust
  if let Some(expected_key) = expected_next {
      if (record.instance, record.counter) != expected_key { break; }
  } else if record.checkpoint_txg != chain_start_txg_without_anchor || ... { break; }
  ```
- 应该做什么 / 为什么错：锚点读得出时，新开的每一次发布只核 jsn 连号与序号 1，不核它的 txg 等于上一次发布的 txg + 1（锚点读不出那一支核了）；同一事务号在带提交标记的那条之后又出现、事务号回退，也照样接上（I-8.7、I-8.8 ③ 由 checker 判，恢复不判）。合法写者写不出这些；写坏的记录会让 `rebuilt.checkpoint_txg` 倒退、`maximum_applied_transaction`（实例表行 W）取到错的值。
- 失败场景：jsn 13（txg T+5，序号 1，末条）之后 jsn 14（txg T+2，序号 1，末条），两条校验和都对 ⇒ 两次都施加，effective_root 的 txg 是 T+2。

### 12. 死代码与只剩一半用处的分支
- 位置：extent_tree.rs:195-201（`ExtentLowerNodePosition::child_holding_data_unit`）；extent_tree.rs:515-526（`ExtentTreeVersion::upper_height` / `lower_height`）；recovery.rs:2089-2096（`key_width_for_kind` 的 EXTENT / ALLOCATION / ACCOUNTING 三臂）；inode_tree.rs:60-68（`container_number` / `smallest_inode_number` 只有本文件测试在用）
- 严重度：低
- 置信度：已确认（`grep -rn` 全仓，含 harness 与 research，除定义外零命中；mounted_read.rs:647-678 里同名的 `upper_height` / `lower_height` 是局部变量）
- 代码做了什么：
  ```rust
  pub fn child_holding_data_unit(self, data_unit: u64) -> Self {
      Self { level: self.level - 1, index: data_unit / lower_span_in_data_units(self.level - 1) }
  }
  TREE_KIND_EXTENT => Some(24), TREE_KIND_INODE => Some(8), TREE_KIND_ALLOCATION => Some(10), TREE_KIND_ACCOUNTING => Some(22),
  ```
- 应该做什么 / 为什么错：`child_holding_data_unit` 没有调用点，而且在叶（level 0）上调用会下溢，留着就是一条没人验的路径；`key_width_for_kind` 唯一的调用点（recovery.rs:2536）在分配记录树、extent 树、记账树三种上都先 `continue` 了，那三臂的值没人读，其中分配记录树 key 宽 10 与记账树 22 是 `allocation_record_tree` / `CodeTwoKeyFieldWidths::ACCOUNTING` 各自一份之外的第三份手抄。删掉，或者让唯一的调用点只处理它真会遇到的种类。
- 失败场景：无运行时后果；某天有人按名字调 `child_holding_data_unit(…)` 于层级 0 的位置 ⇒ 下溢 panic。

### 13. 文件内测试：一条恒真断言、一条注释说测三种实际测两种、读树两档没有文件内用例
- 位置：code_two_tree.rs:1346；extent_tree.rs:1174-1202；code_two_tree.rs:866-1110（`read_code_two_tree` 与 `flatten_read`）
- 严重度：低
- 置信度：已确认（只看了文件内 `#[cfg(test)]`，harness 里的用例没逐个核）
- 代码做了什么：
  ```rust
  assert_eq!(u8::MAX.checked_add(1), None);                       // code_two_tree.rs:1346：测的是标准库
  /// 上段叶条目 113 字节来回：三种标签各解回自己；...           // extent_tree.rs:1174
  for entry in [no_unit, lower] { ... }                           // 只有标签 0 与 1，标签 2（内联数据指针）没来回
  ```
- 应该做什么 / 为什么错：第一句恒真，删掉。extent 那条补上标签 2 的来回。`read_code_two_tree` 两档（`EveryHeaderAgainstItsReference` / `OnlyWhatTheShapeNeeds`）在本文件里一条读回用例都没有，第 2 条的坏法（叶里重复 key 在宽松档被收下）没有会红的量。
- 失败场景：把 `leaf_read_from_disk` 的 `judges_every_header()` 判定整段删掉，本文件的测试全绿。

## 核过、不报的
- journal 计数器是 6 字节（journal.rs:272 `get_six_byte_unsigned`），`counter + 1`（recovery.rs:1970、2015）不会溢出；槽号 6 字节、跨度 15 位，`record.slot.0 + span`（recovery.rs:1016）与 `to_device_offset`（address.rs:84）不会溢出。
- `InodeLeafContainer::free_record_slots` 的 expect（inode_tree.rs:92-96）从盘上走不到：重建要求记录恰好 140 字节（records.rs:77），码 3 单元 (32768 − 107) ÷ 140 装不下第 234 条。
- `walk_to_file` 的 `Vec::with_capacity(size)`（recovery.rs:2726）前面核过单元数等于 ⌈size ÷ 净荷⌉，size 被盘上实际读回的指针数钉住；`data_unit_payload` 的切片（unit.rs:519-523）前面核过声明长度。
- extent 树的两档读法都核位置（层级、区间、条目严格递增），宽松档只少核出生身份（extent_tree.rs:586-667 读过；分配记录树按它自己的文档也是两档都核位置，没逐行读）；extent 几何全用饱和乘，从盘上层级字节读进来的 255 层不溢出。
- I-9.15：`InodeRecord::to_bytes` 写 `size.div_ceil(512)`（records.rs:59）算式对；`parse` 跳过 blocks 不判（records.rs:87），invariants.md 那一行写着「读者要不要也判、判了报什么没有条款」，不算缺陷。
- 点名单元两份都要验过（recovery.rs:2029-2040 的 `all`）是 D23 已定项 14 第四条的定案（2026-09-25 用户定），不算缺陷。

## 注释与代码不一致清单
| # | 位置 | 注释说的 | 代码实际 |
|---|---|---|---|
| 1 | recovery.rs:1467 | I-9.2「走读同款在 `walk_to_file` 判」 | 可写挂载（mount.rs:2827-2905）只重建、不走读，这一条在那条路上没人判（第 3 条） |
| 2 | code_two_tree.rs:322 | 走到 `PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt`「说明核漏了」 | 重建按 `OnlyWhatTheShapeNeeds` 读，本来就不核区间，走到这一格是那一档的预期结局 |
| 3 | code_two_tree.rs:766-769 | 分隔 key 与孩子对不上时「由规划那一步交回 Refusal」 | 只对删 key 成立；插 key 路由不查，叶 key 乱序 / 重复最后在 transaction.rs:6468 的 assert 上 panic（第 2 条） |
| 4 | recovery.rs:1-8（模块文档）、recovery.rs:1919-1930 | 前缀口径列了五样，没有反向链 | 与 invariants.md I-8.6「不等的记录不进重放前缀」、D23 已定项 7「jsn 严格连续 + 校验和 + 反向链」不一致（第 5 条） |
| 5 | recovery.rs:2316 | expect「上面核过每块盘都有记录」 | 盘表为空时前一判放行，前提不成立（第 10 条） |
| 6 | recovery.rs:676-677 | `(None, None)` 分支注释「见报告里的设计问题」 | 指向一份不在仓里的「报告」；返回的 `SystemConfigurationsDisagree` 的成员文档（recovery.rs:172-173）只说「fsid 或设备数对不上」，没说空池也报它（函数的 `# Errors` 写了） |
| 7 | extent_tree.rs:89-93 | expect「第 8 层起罩的 inode 号已超过 u64 的上界」 | `upper_span_in_inodes` 饱和在 u64::MAX，`span > u64::MAX` 永假，`largest_inode = u64::MAX` 时这里 panic；今天唯一调用点传常量 1（transaction.rs:3662），走不到 |
| 8 | extent_tree.rs:1174 | 「三种标签各解回自己」 | 只来回了标签 0、1（第 13 条） |
| 9 | mounted_read.rs:61 | inode 树 key 宽 8「`recovery::key_width_for_kind` 同一份登记」 | 是另写的一个常量 `INODE_KEY_WIDTH_IN_BYTES = 8`，不是同一处定义 |

## 恢复算法逐步描述（按代码）
1. **择系统配置**（`choose_system_configuration`，recovery.rs:594-679）：按 `device_identities()` 次序逐盘读槽 0（偏移 0）；槽 0 自证过就按它的槽距读槽 1，否则按 4096 读槽 1。每槽分四路：读不到 / 自证不过、incompat 位不认识（记下第一处）、S 越界（整池立即拒）、可择。一盘两槽取世代号大的（相等取槽 0）；两槽都不可择就跳过这块盘。第一块有可择槽的盘的配置当选，之后各盘只比 fsid 与设备数，不一致报 `SystemConfigurationsDisagree`。一份都没有时，有不认识 incompat 的报它，否则报 `NoValidSystemConfiguration`，空池报 `SystemConfigurationsDisagree`。
2. **择根**（`choose_root`，776-796）：三个区域 × S 个槽逐个按区域归属表与槽距读 `physical_block_size` 字节，`RootRecord::parse_slot` 验 magic、整槽宽校验和、fsid、flags 未知位；自证过的里取 `(checkpoint_txg, 实例代号)` 最大的，先遇到的赢平局。不看实例表、不看 F。
3. **全环扫描**（`scan_journal`，1826-1862）：每块盘读 `[16 MiB, 16 MiB + journal_ring_bytes)` 里每个 4096 字节槽（读者有提示时只读提示的），`JournalRecord::parse` 验 magic、整条宽校验和、类型、标志、长度、提交标记 0/1、序号 ≠ 0、点名项校验和、fsid；按 `(实例代号, 计数器)` 收进 BTreeMap，同 key 先读到的留下。不核记录落在它计数器该在的槽上，不核反向链。
4. **定锚点**（`counter_of_the_last_record_the_root_covers`，1892-1916）：与所选根同实例、同 txg、带「本次发布末条」标志的记录；多于一条报 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`、一条都不施加；没有就是「锚点读不出」。
5. **取前缀并施加**（`replay_journal`，1931-2084）：候选 = 同实例且 txg > 根 txg 的记录，按计数器升序，只看前 `ring_bytes ÷ 4096 ÷ F` 条（在飞上限）。逐条：锚点读得出 ⇒ jsn 必须 = 期望值（锚点 + 1 起连号）；读不出 ⇒ 第一条要 txg = 根 txg + 1 且序号 1。新开一次发布要序号 1；同一次发布内要同 txg、序号连号、上一条没提交标记时事务号不许换。末条不带提交标记 ⇒ 停；末条之后同实例同 txg 还读得出记录 ⇒ 停。点名单元（`Consult`）两条位置条目都要读得出、整单元 CRC 等于点名项里的校验和，否则 `verification_failed += 1` 并停。一次发布走到末条才整体施加：根换成末条新根段里的树表、映射树根、树 ID 水位、F、实例、txg，实例表指针与分配记录树根照抄被施加的那条根，卸载记号写「非卸载序列」。停在一次发布中间的那几条不施加。
6. **走读**（`walk_to_file`，2388-2791，只有 `recover` 走）：实例表第 0 片（多片沿链读完）→ 树表（key 宽 8）→ 逐条树表条目：树 ID < 水位；空根跳过；分配记录树、extent 树按位置整棵读并核全部头，记账树按「每个头对着父条目」整棵读，inode 树只读根并核头；提示读不出时经中央映射回退（映射树第一次用到时整棵读、全核）。之后核：分配记录几何、每落点每盘一条、记账条目 3 + 6 × 盘数、映射条目数、分配记录跨度与代、记账代与 seq、inode 根层级 1、每片叶的身份 / 记录宽 / fsid / 出生身份、记录号 ≥ 分隔 key 与容器号；找第一个文件的 inode 记录，单元数 = ⌈size ÷ 净荷⌉，逐个数据单元核五元组、对象出生代、头与指针、声明长度与补齐零，拼出内容。
7. **可写挂载**（mount.rs:2827-2905，参照）：1、2、3、5 同上，然后 `rebuild_version` 读同一条根，不跑第 6 步；数据单元提示与映射都读不出时照抄位置项、字节为空，不算失败（D19 已定项 5 的 N2）。
8. **盘上写**：recovery.rs 全文没有一处写盘（`write_at` / `flush` 零命中）；恢复之后的写（取号、写行、暖机）全在 mount.rs，屏障次序不在本次范围。

## 读过的文件与行数
- 全读：crates/singlefs-core/src/recovery.rs 1-3113；extent_tree.rs 1-1203；code_two_tree.rs 1-1366；inode_tree.rs 1-625。
- 按需读：records.rs 38-118、274-360；unit.rs 332-524；journal.rs 249-330；root_record.rs 110-155；system_configuration.rs 427-517；allocator.rs 30-95、222-237；allocation_record_tree.rs 82-300、802-862；instance_table.rs 89-115；address.rs 71-91；pointer.rs 30-40；root_ring.rs 101-105；write_request_split.rs 56-62；mount.rs 546-560、2013-2060、2827-2905；transaction.rs 2300-2385、4010-4040、4195-4225、6440-6520；mounted_read.rs 55-62；singlefs-format/src/lib.rs 常量；Cargo.toml。
- 条款：.claude/kb/decisions/23-journal的角色与格式.md 200-240、340-372、555-580；invariants.md 中 I-1.8、I-8.6、I-8.8、I-8.9、I-9.1–I-9.15 各行；D22 槽距那几行（grep）。
