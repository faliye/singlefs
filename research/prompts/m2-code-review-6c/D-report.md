# 挂载与读路径审阅报告（mount.rs / mounted_read.rs / mounted_session.rs）

只读审阅，没跑 cargo、没跑脚本。每条都引了原行；「已确认」指读代码能把推理走通，「疑似」指后果要跑才能定。

### 1. 可写挂载不核调用方给的 `parameters` 与盘上系统配置是不是同一份，写全按 `parameters`、读全按盘上
- 位置：crates/singlefs-core/src/mount.rs:2828-2834、2419、2423-2424、1646、3543；写侧 crates/singlefs-core/src/transaction.rs:378-382、587-603
- 严重度：高
- 置信度：已确认
- 代码做了什么：
  ```rust
  let system_configuration = choose_system_configuration(&*devices)?;          // mount.rs:2834，读、判定都用它
  let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());           // mount.rs:2419，写入口用调用方的
  let warm_up_publishes_planned = warm_up_publish_txgs(parameters, &all_devices, start.first_txg); // 2423
  let instance_to_acquire = instance_generation_to_acquire(&pool);             // 2424，按 parameters 的 fsid/几何读
  sizes: self.parameters.geometry,                                             // transaction.rs:382，取号写的系统配置
  ```
- 应该做什么 / 为什么错：`mount.rs` 里没有一处把 `parameters.filesystem_identifier / region_devices / geometry` 与 `system_configuration.immutable` 比（`grep -n "filesystem_identifier\|region_devices" mount.rs` 除 `immutable.` 之外只剩 1494、1599、2196 三处按 `parameters` 取盘）。盘上系统配置是这几样的权威，挂载已经把它读在手里；写入口却用第二份来源。应在读第一块盘之后、任何写之前逐项比，对不上就拒（或者直接从盘上那份构造写入口要的参数）。
- 失败场景：
  - fsid 不同：`instance_generation_to_acquire` 用 `parameters.filesystem_identifier` 验槽与根，一份都认不出 ⇒ 取到号 1（池里原本是 5），`acquire_expected_instance` 重算同样是 1、照写（取号之前的预演用 `parameters` 的 fsid 装单元、不按它读盘验，所以拦不住——这一段是推的，没跑）；之后的根、记录、系统配置按错的 fsid 写出，下次挂载按盘上 fsid 读不出这批根——实例代号回退、写出的东西全部不可见。
  - 同 fsid、S 或槽距不同：根落点按 `parameters.geometry.root_ring_slots_per_region` 算（transaction.rs:294-306），S 更大时槽号越过盘上那一区的长度，写进根环区域之外的字节；取号那次写的系统配置把 `sizes` 换成调用方的几何（transaction.rs:382），盘上「不可变」配置被改写。
  - `region_devices` 不同：暖机计划（mount.rs:2196）与抬 F 那一串（1494、1599）按调用方的归属表算「落在哪块盘」，而 `rollback_floor_ceiling` 按盘上那份（1117-1128）算——两处对同一个 txg 的归属可以不同。

### 2. 同一个设备身份交两次、而不同身份仍 ≥ 2 时，准入放行，取号把系统配置写进那块重复标号的盘
- 位置：crates/singlefs-core/src/mount.rs:2374-2403（计数）、2314-2370（逐盘核）；crates/singlefs-core/src/transaction.rs:660、395（取号写）；crates/singlefs-core/src/recovery.rs:97-113（按身份读取第一项）
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  .collect::<BTreeSet<DeviceIdentity>>().len(),          // mount.rs:2384，只数不同身份
  for device in reader.device_identities() {              // mount.rs:2327，身份表不去重，读按身份取第一项
  for index in 0..pool.devices.len() {                    // transaction.rs:660，取号按下标写每一项
  self.devices[index].1.write_at(                         // transaction.rs:395
  ```
- 应该做什么 / 为什么错：`distinct_device_identities_handed_in` 的文档说「同一个身份交两次只算一块盘」，`admit_the_writable_device_count` 的文档说「取号写的就是这里数过的这几块盘」。但重复项不被拒：盘表 `[(d0, A), (d1, B), (d0, C)]` 数出 2 块、过准入；逐盘核按身份读，C 那一项永远读的是 A，从没被核过；取号按下标写，C 收到一次系统配置槽写（世代号按 A 的槽算，`device_count` 写成 3）。盘表里有重复身份应当直接拒，不是去重后放行。
- 失败场景：调用方把一块不属于池的盘误标成 d0 放在第三项 ⇒ 挂载成功，那块盘偏移 0 或槽距处 4096 字节被本池的系统配置覆盖；现有用例只测了 `[d0, d0]`（harness `mount_writable_with_device_zero_handed_in_twice_counts_one_device_and_is_refused_before_any_read`，不同身份只有 1），走不到这一格。

### 3. 盘上指针的槽号不经几何判就进分配器，坏镜像能把可写挂载打 panic
- 位置：crates/singlefs-core/src/mount.rs:983-1017（`format_time_allocator`）、756-793（`placements_referenced_by_root` 树表 0 条那一臂）、722（`isolate_abandoned`）；crates/singlefs-core/src/allocator.rs:273-277、339-342、450-460、1346-1352
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  allocator.mark_format_time_units(instance_table_placement, tree_table_placement); // mount.rs:1016
  let slot = slot_shared_by_both_location_entries(&pointer.locations).ok()?;        // mount.rs:789
  allocator.isolate_abandoned(device, slot, span_slots);                            // mount.rs:722
  let offset_in_slots = slot.0.checked_sub(UNIT_AREA_START_SLOT).expect(           // allocator.rs:274
      "槽号在单元区内：盘上读来的分配记录在 recovery::allocation_records_fit_the_pool_geometry 判过，…");
  ```
- 应该做什么 / 为什么错：盘上的分配记录都过 `recovery::allocation_records_fit_the_pool_geometry`（槽号 ≥ 单元区起点、跨度不越界、同盘不相交），allocator 的 `expect` / `assert!` 靠的就是这一判。但这两条路径的落点不是分配记录，是根记录与实例表链、分配记录树父条目里的指针，只判了「两条位置条目同槽」与「诞生 txg 为 0」，没判几何。应在进分配器之前对这些落点做同一套几何判，报错而不是 panic（同文件对别的盘上字段已经这样做，见 `FormatTimeUnitLocationsOnDifferentSlots` 的注释「panic 面普查 R5」）。
- 失败场景：
  - 一份根记录自证通过、树表 0 条、分配记录树根全零、实例表指针诞生 txg 0、两条位置条目都指槽 100（单元区起点 50176 之下），那两个位置上放着 CRC 对得上的字节 ⇒ `rebuild_previous_version` 读得出 ⇒ `format_time_allocator` → `record` → `mark_allocated` → `index()` 的 `expect` panic。实例表（跨 2 槽）与树表指到重叠的槽 ⇒ `mark_allocated` 的「跨度里有已分配的槽」断言 panic。
  - 被抛弃根的树表 0 条时，`instance_table_page_pointers_as_far_as_readable` 恒带上根记录里那条实例表指针（读不出也带），它的槽号同样不判，进 `isolate` 的 `index()` 或 `assert!(end <= self.allocated.len())` panic；抬 F（mount.rs:1617-1632）重算影子账走同一条。

### 4. 只读挂载打不开任何新建的空 inode：「长度 0 ⇒ 1 个单元」与「新 inode 没有 extent 记录」对不上
- 位置：crates/singlefs-core/src/mounted_read.rs:650、714-725；crates/singlefs-core/src/write_request_split.rs:59-62；crates/singlefs-core/src/transaction.rs:4476-4485（`publish_new_inodes`）
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  None | Some(ExtentUpperLeafTarget::NoDataUnit) => (Vec::new(), 0, 0),       // mounted_read.rs:650
  let data_units_implied_by_the_file_size =
      data_unit_count_of_a_sequential_write(inode_record.size);              // mounted_read.rs:714-715
  content_length_in_bytes.div_ceil(payload_capacity).max(1)                   // write_request_split.rs:61
  size: 0,                                                                    // transaction.rs：新建 inode 长度 0，「数据单元与 extent 树不动」
  ```
- 应该做什么 / 为什么错：`open_file` 的文档说「标签 0 与缺席一个单元都没有」，拿到 0 条记录；但紧接着的条数核用的是写侧切分的单元数，它对长度 0 返回 1（第一个文件那种「声明长度 0 的一个单元」的口径）。于是「0 条记录」这一臂在条数核上恒红，只有第一个文件走得通。`publish_new_inodes` 建的每个 inode 都是长度 0、没有 extent 条目，所以它们在只读挂载里一个都打不开。读侧该按「这个 inode 有没有 extent 条目」分两种口径：没有条目时要求 size == 0、单元数 0；有条目时才用写侧那条除法。
- 失败场景：`MountedSession::publish_user_change(UserChange::NewInodes { count: 1, .. })` 之后 `mount_read_only` → `open_file(inode = 水位里新发的号)` ⇒ `ExtentRecordCountDoesNotMatchTheFileSize { extent_records: 0, data_units_implied_by_the_file_size: 1, file_size_in_bytes: 0 }`，错误文档说的是「文件有洞或尾巴上多出记录」，而盘上没有任何坏数据。全仓 `open_file(` 的调用点全部传 `FIRST_INODE_NUMBER`，没有一个用例碰到这一格。

### 5. 抬 F 途中的块设备错被吞掉：可写挂载照样交回 `Ok`，挂着的会话把它报成空间不够
- 位置：crates/singlefs-core/src/mount.rs:1520、2674-2691、2780-2786；crates/singlefs-core/src/mounted_session.rs:225-233
- 严重度：中
- 置信度：已确认
- 代码做了什么：
  ```rust
  Err(failure) => Err(FloorRaiseStop::FloorRaiseFailed(Box::new(failure))),   // mount.rs:1520
  return MountSpaceAdmission::StillShortAfterTheFloorRaises { .., stop, }      // mount.rs:2780，随后 establish_instance 交回 Ok(Mounted)
  Err(stop) => { return Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(Box::new( // mounted_session.rs:225-226
  ```
- 应该做什么 / 为什么错：`FloorRaiseFailed` 里装的可以是 `RaiseFloorSequencePublishFailed`（落盘途中块设备报错，那次发布冻结在分配器上等原样重发）或 `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`（写系统配置那一步 I/O 错）。挂载处把它们与「预算做满 / F 已到上限」同样收成 `StillShortAfterTheFloorRaises`，`mount_writable` 交回 `Ok`；`MountSpaceAdmission` 的文档把这一成员描述成「推满仍不够」。会话处把它收进 `NoSpaceAfterRaisingTheFloor`，文档说「报给用户就是 ENOSPC」。块设备错与空间不够是调用方要做的两种不同决定（先原样重发冻结的发布 / 腾空间），`code-discipline.md`「错误成员按调用方要做的决定分」。应把 `FloorRaiseFailed` 从这两处拆出去，原样交回错误。
- 失败场景：挂载时取号前空间准入不够、写行与暖机之后推抬 F，第一串的第二次空发布写根时设备报错 ⇒ `mount_writable` 返回 `Ok`，`Mounted.allocator` 上冻结着一次发布；只看 `is_ok()` 的调用方把池当成正常可写，之后每一次 `publish_user_change` 都收到 `Publish { cause: PublishFrozenAfterAWriteFailureIsNotResentYet }`，而会话没有重发入口。会话里同样的 I/O 错被报成 ENOSPC。

### 6. 重建分配器时另选一次根、另读一次实例表，读不出就静默把影子账整个关掉
- 位置：crates/singlefs-core/src/mount.rs:852-858、859-891
- 严重度：中
- 置信度：已确认（代码缺的那一判读得出来；后果要两次读之间有一次瞬时读错才走到）
- 代码做了什么：
  ```rust
  let newest_table = choose_root(devices, system_configuration)
      .and_then(|newest| instance_table_of_root(devices, &newest));
  let is_abandoned = |root: &RootRecord| {
      newest_table.as_ref().is_some_and(|table| abandoned_by_table(root, table))
  };
  ```
- 应该做什么 / 为什么错：`mount_writable` 在这之前已经择过根（2835-2836）、并在 `rebuild_previous_version` 里把所选那一版的实例表沿链读出来了（读不出就报 `InstanceTableMalformed`，mount.rs:553-554）。`rebuilt_allocator` 不用那一份，重新 `choose_root` 再读一次；这一次读不出时 `newest_table` 是 `None`，`is_abandoned` 对每条根都判 false——被抛弃根全算成有效根：影子账一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0、根环表里全记成 `ValidRoot`，而挂载照常做成、没有任何可观测的信号。同一件事在抬 F 里是拒（mount.rs:1557-1559 读不出报 `InstanceTableMalformed`），两处处置相反。应把已经读出的那张表传进来，不重读。
- 失败场景：崩溃恢复落到旧根、环里有被抛弃时间线的根；可写挂载第二次读实例表那一刻读错一次 ⇒ 只被被抛弃根引用的槽成了空闲槽，被这次挂载之后的发布发出去——正是影子账要防的 C314（回退可以复用被抛弃的根引用的单元） 那一格，而 `MountOutput` 里 `isolated_slots_per_device` 为 0、`shadow_ledger_branch` 仍报 `shadow_ledger=on`。

### 7. 新实例第一次发布的 txg 不与已选中的根取 max，重读根环少读到一条就可能不高于它
- 位置：crates/singlefs-core/src/mount.rs:599-617、2870
- 严重度：低
- 置信度：已确认（缺的那一道 max 读代码可复现；后果要瞬时读错且环里没有那次发布的记录才走到）
- 代码做了什么：
  ```rust
  let highest_ring_txg = highest_root_txg(devices, …).unwrap_or(CheckpointTxg(0));   // 第二遍独立读根环
  CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)                     // mount.rs:616
  ```
- 应该做什么 / 为什么错：紧挨着的 `tree_identifier_watermark_of_the_ring`（mount.rs:623-641）专门为「择根那一遍读到了它、这一遍没读到」再与 `version_to_build_on` 取一次 max；`first_txg_of_new_instance` 没有这一步，也没收 `chosen_root` / `effective_root`。D23（journal 的角色与格式） 已定项 14 第 3 条要的是「max(根环里全部根记录的 txg, …) + 1」，择根时读到的那条根就在其内。应再与 `effective_root.checkpoint_txg`（≥ `chosen_root` 的）取 max。
- 失败场景：所选根 txg 100 所在的槽在第二遍读时瞬时读错、环里又没有 txg 100 那次发布的记录 ⇒ `first_txg` ≤ 100：新实例的根与所选根同槽（靠实例代号比赢）或 txg 更低；更低时下一次择根按 (txg, 实例) 会选回旧实例那条根，新实例开头几次发布不可见，崩在这个窗口里就丢掉它们。

### 8. 盘上的 txg / 实例代号不设上界，加一在 release 下溢出 panic（只读挂载也走得到）
- 位置：crates/singlefs-core/src/recovery.rs:1971（`replay_journal`，只读挂载与可写挂载都调）；crates/singlefs-core/src/mount.rs:616；crates/singlefs-core/src/transaction.rs:597-602（`instance_generation_to_acquire`）；来源 crates/singlefs-core/src/root_record.rs:129-130；Cargo.toml:17 `overflow-checks = true`
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```rust
  let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);  // recovery.rs:1971，进循环之前无条件算
  CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)                   // mount.rs:616
  highest_system_configuration_instance(pool).max(highest_root).0 + 1,           // transaction.rs:598-601，u32
  let checkpoint_txg = CheckpointTxg(reader.get_u64());                           // root_record.rs:130，不判范围
  ```
- 应该做什么 / 为什么错：根记录只判长度、magic、32 字节宽校验和、fsid、flags，txg（u64）与实例代号（u32）取任何值都算自证通过；三处 `+ 1` 在 overflow-checks 打开时 panic。按「挂载读到的每个字节都可能是坏的」，这几处应 `checked_add` 并报一个 `RecoveryFailure`，或在 `parse_slot` / 系统配置解析处给出上界。
- 失败场景：一份改出来的镜像，环里一条根 txg = `u64::MAX`、宽校验和按改后的字节重算 ⇒ `choose_root` 选中它 ⇒ `mount_read_only` 在 `replay_journal` 里 panic（只读挂载承诺一个字节都不写，也不该崩）；实例代号 `u32::MAX` 的根或系统配置槽 ⇒ 可写挂载在 `instance_generation_to_acquire` panic。随机位翻转过不了宽校验和，只有构造的镜像走得到，所以定低。

### 9. 「正常卸载之后这个写入口不再发布」没有东西强制
- 位置：crates/singlefs-core/src/mount.rs:1339-1344（`unmount` 文档）、1349-1355；crates/singlefs-core/src/mounted_session.rs:26-34、169-238
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```rust
  /// … 不写干净关闭标记，下一次挂载照旧走恢复。做完之后这个写入口不再发布。     // mount.rs 文档
  pub fn unmount<Device: BlockDevice>(…, allocator: &mut PoolAllocator, current: &mut PoolVersion, …)
  pub struct MountedSession { pub allocator: PoolAllocator, pub current: PoolVersion, … } // 字段全公开，没有「已卸载」状态
  ```
- 应该做什么 / 为什么错：`unmount` 借走分配器与现行版本、推完带卸载记号的那一串就返回，会话本身没有任何状态记下「已卸载」，也没有消费 `self` 的卸载方法；`publish_user_change` 在卸载之后照常发布。按 `code-discipline.md`「让非法状态写不出来」，应当由会话提供一个 `fn unmount(self, …)` 消费掉会话。
- 失败场景：调用方 `mount::unmount(…, &mut session.allocator, &mut session.current, …)` 之后再 `session.publish_user_change(…)` ⇒ 在带卸载记号的根后面接着写不带记号的根，文档声称的「卸载之后不再发布」不成立；没有编译期或运行期的拦截。

### 10. 死代码：三个公开取值方法全仓无调用点
- 位置：crates/singlefs-core/src/mounted_read.rs:150（`ExtentLeafRecordInMountState::inode_number`）、172（`data_unit_pointer`）、783（`OpenFileForRead::data_unit_records`）
- 严重度：低
- 置信度：已确认（`grep -rn "inode_number()\|data_unit_pointer()\|data_unit_records()" crates research` 除定义外 0 处；`mounted_read.rs` 内部直接读字段）
- 代码做了什么：
  ```rust
  pub fn inode_number(&self) -> InodeNumber {
  pub const fn data_unit_pointer(&self) -> DataPointer {
  pub fn data_unit_records(&self) -> &[ExtentLeafRecordInMountState] {
  ```
- 应该做什么 / 为什么错：`pub` 让编译器不报 `never used`，`code-discipline.md`「用不到的代码删掉」。它们不承担任何断言，删掉不丢判定；要留给装置用就补一个调用点。
- 失败场景：无运行时后果；`inode_number()` 从没被任何用例调用，它切 key 第二段的偏移错了也没人会发现。

### 11. 文件内单测的两处空档
- 位置：crates/singlefs-core/src/mount.rs:3570-3666（`non_empty_root_tests`）；crates/singlefs-core/src/mounted_read.rs:1059-1141（`tests`）
- 严重度：低
- 置信度：疑似（harness 里的用例可能抓得到，这里只说文件内单测）
- 代码做了什么：
  ```rust
  ceiling_from_newest_and_non_empty_roots(CheckpointTxg(9), distinct, Some(CheckpointTxg(1))),   // 两次都传 9
  Some(newest_on_every_device.min(fourth_newest))                                               // mount.rs:1227
  ```
- 应该做什么 / 为什么错：两次调用的 `newest_on_every_device` 都是 9、都大于交回值，`min` 的第一个参数从没胜出过：把 1227 行改成 `Some(fourth_newest)`，这两个单测照样绿。应补一格「每块盘最新有效根低于第 4 新非空根」钉住上限取前者。`mounted_read` 的单测只测跨单元的算术与多跳率，`open_file` / `read_at` 的每个错误成员都没有文件内用例，而第 4 条那一格正是因为没人用非第一个 inode 打开过才漏掉。
- 失败场景：见第 4 条；`min` 那一格的变异是否被 harness 抓到，要跑 `crates/mutations.tsv` 才能定。

## 查过、没发现问题的（不另立条目）
- 读路径每一跳：数据单元按指针或映射条目里的整单元 CRC-32C 验（mounted_read.rs:1031-1057），再核单元头的出生树、对象、对象出生代、fsid、出生 txg 与写序、锚点偏移、声明长度（945-1000）；树节点走 `read_mapped_tree_root` / `read_mapped_tree_node_via_hint_then_central_mapping`，inode 叶容器核身份、记录类型、记录宽、fsid（465-493）；`read_at` 用 `checked_add` 判越界（809-823），拷贝区间在声明长度核过之后切，不越界。
- 择根按 `(checkpoint_txg, 实例代号)` 取大，不按槽位置（recovery.rs:776-797）；根要过长度、magic、32 字节宽校验和、fsid、flags（root_record.rs:110-127），fsid 不同的根进不来。
- 取号：新号 = max(每盘两槽自证过的系统配置里的实例代号, 根环里根的实例代号) + 1，写之前重算、不等就不写（transaction.rs:587-655）；两盘都写完才过一道屏障，回卷只写一个更新世代的槽、带新号的那一槽仍在，下次取号只会更大，没看到重号的路径。
- 三个文件里没有 `_ =>` 通配臂、没有 `impl Drop`、没有 `let _ =`；`.ok()` / `unwrap_or` 六处（mount.rs:608、615、775、789、798、2868）都是文档写明的「读不出就计数 / 按 0 起」，除第 6、7 条那两处外不另报。

## 读过的文件与行数
| 文件 | 行 |
|---|---|
| crates/singlefs-core/src/mount.rs | 1-3690（全部） |
| crates/singlefs-core/src/mounted_read.rs | 1-1141（全部） |
| crates/singlefs-core/src/mounted_session.rs | 1-238（全部） |
| crates/singlefs-core/src/recovery.rs | 96-149、270-360、540-1000、1000-1740、1782-1800、1826-2090 |
| crates/singlefs-core/src/transaction.rs | 193-225、340-400、530-760、4320-4580 |
| crates/singlefs-core/src/allocator.rs | 273-292、339-364、450-473、903-1024、1346-1372 |
| crates/singlefs-core/src/extent_tree.rs | 694-800、1001-1041 |
| root_record.rs 110-155；root_ring.rs 104-137；pointer.rs 15-120；address.rs 18-26、71-91；write_request_split.rs 56-80；records.rs 76-98；checksum.rs 84-88；make_filesystem.rs 60-85；Cargo.toml 14-17 | 按需 |
| .claude/kb/invariants.md | I-3.8、I-4.8、I-7.4、I-7.9、I-7.12 那几行 |

## 根选择算法的逐步描述（按代码）
可写挂载 `mount_writable_with_test_only_switches`（mount.rs:2827-2925）：
1. 盘表里不同设备身份数 < 2 ⇒ 拒（2833）。重复身份只数一次、不拒（第 2 条）。
2. `choose_system_configuration`（recovery.rs:594-679）：逐盘读槽 0（偏移 0）与槽 1（偏移 = 槽 0 可择时它记的槽距，否则 4096）；S 越界整池拒；每盘取世代号大的那一槽；**第一块交得出的盘那一份就是所选**，后面的盘只核 fsid 与设备数相同，不比世代号、不比几何。
3. `choose_root`（recovery.rs:776-797）：三个区域 × S 个槽逐个读 `physical_block_size` 字节（盘取 `region_devices[区域]`），`parse_slot` 自证通过的里取 `(txg, 实例)` 最大的。**不看 F、不看实例表**；选中的根下面读不出时不退回更旧的根，挂载直接报错。
4. `scan_journal`：每块盘整环按记录宽逐个解，按 `(实例, 计数器)` 去重、先到先得。
5. `replay_journal`（recovery.rs:1931-2086）：只接所选根那个实例、水位之上的记录；锚点是所选根那次发布带末条标志的那一条；断号、换 txg、序号不连、提交标记缺、点名单元 CRC 不对即停；按整次发布施加，得到施加前缀之后的根（实例表指针与分配记录树根照抄所选根的）。
6. 上一版的「那条记录」：同实例、同 txg、带末条标志的那条，读不出就拿环里计数器最大的一条顶着（2849-2861）；由它算所选那一版的 jsn 位置。
7. `rebuild_previous_version`：按施加之后的根重建上一版，并沿链读它的实例表（读不出 ⇒ `InstanceTableMalformed`）。
8. `next_counter` = 环里最大计数器 + 1；`first_txg` = max(**再读一遍**根环的最大 txg, 记录最大 txg) + 1（第 7 条）；树 ID 水位另读一遍根环并与施加之后的根取 max。
9. `rebuilt_allocator`：F_生效 = max(每盘按「环里最新根的实例表」判仍有效的最新根带的 F, 各盘系统配置里的 F)；**再 `choose_root` 一次**取它的实例表判抛弃（第 6 条）；回收门槛 = max(F_生效, 环里最旧有效根)；影子账隔离只被被抛弃根引用的槽。
10. `establish_instance`：空间准入 → 整串预演 → 逐盘核带不带所选那一版 → 取号 → 写行 → 暖机 → 取号前准入不够时推抬 F。
只读挂载 `mount_read_only`（mounted_read.rs:548-570）只走第 2-5 步，再沿施加之后的根 `open_pool_for_read`。

## 注释与代码不一致清单
1. mount.rs:2389-2392 `admit_the_writable_device_count`：「取号写的就是这里数过的这几块盘」——取号按盘表下标写每一项（transaction.rs:660），重复身份那一项没被数过也被写（第 2 条）。
2. allocator.rs:274-275 `DeviceFreeMap::index` 与 allocator.rs:1351 `isolate_abandoned` 的 `expect` 消息说槽号「在 recovery::allocation_records_fit_the_pool_geometry 判过」——mount.rs:1016 的 mkfs 两个单元与 mount.rs:756-793 树表 0 条那一臂的落点都没过这一判（第 3 条）。
3. mounted_read.rs:603-606 `open_file` 文档「标签 0 与缺席一个单元都没有」是一条正常路径，而 714-725 的条数核对 0 条记录恒拒；`OpenFileFailure::ExtentRecordCountDoesNotMatchTheFileSize` 的文档「文件有洞或尾巴上多出记录」也罩不住「新建的空 inode」（第 4 条）。
4. mount.rs `MountSpaceAdmission::StillShortAfterTheFloorRaises` 文档「推满仍不够」、mounted_session.rs `UserChangeRefused::NoSpaceAfterRaisingTheFloor` 文档「空间不够（报给用户就是 ENOSPC）」——两者都还装着抬 F 途中的块设备错（第 5 条）。
5. mount.rs:808-809 `rebuilt_allocator` 文档「被抛弃的根按最新根指着的实例表判」——那张表读不出时代码一条都不判成被抛弃（852-858），不是「按表判」（第 6 条）。
6. mounted_read.rs:296-303 `ExtentTreeReadsAtOpen` 的 `height` / `leaves` 文档说「这一路的高」「读到的层级 0 的节点个数」——代码写 `leaves: 1 + lower_leaves`（679）、`height` 取上段读到的节点数；inode 落在上段某个内部节点的区间之外时 `find_upper_leaf_entry` 在非叶那一层就返回（extent_tree.rs:1033-1036），没读到叶也报 1 片叶、高也少算。只影响观测点。
7. 「每块盘上最新的有效根」两处算法不同：`rollback_floor_ceiling`（mount.rs:1117-1128）按根的 txg 用落点公式推它在哪块盘，`effective_rollback_floor`（recovery.rs:938-958）按它实际读出来的根环槽归盘。本实现写的根两者相同；同一个量两条口径没有一处断言把它们连起来。
8. mount.rs:1344 `unmount` 文档「做完之后这个写入口不再发布」——没有状态或类型拦（第 9 条）。
9. mount.rs:2844-2848 注释「上一版的记录只有 jsn 会被用到」——顶着的那条（可以是别的实例、被抛弃时间线上的）整条进了重建出来的 `TransactionOutput.record / record_bytes / highest_transaction_number_in_this_instance`（recovery.rs:1681-1716）；今天写行那张计划不读这几项（row_publish_plan_on_file_version 显式给 0），所以无害，但注释比实际窄。
