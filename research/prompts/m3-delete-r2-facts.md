# 写入时预付（unlink 空间口径）：实现今天的事实表

批次：records/2026-10-01-删除与并发事务.md#2。读的是冻结副本 `/tmp/claude-1000/m3-delete-r2/tree/crates/`，下文文件名一律写成 `crates/…`（对应冻结副本里同一路径）；行号都是这一次用 awk 现取。
只列事实，不推荐、不论证；凡是「推的」都单独标出。kb 两份（invariants、decisions）读的是主仓 `.claude/kb/` 当前工作区。

## (a) 准入今天怎么算

### a1 可用式子的实现（逐项来源）

式子函数 `AdmissionReading::available_on_each_device`，原文：
crates/singlefs-core/src/admission.rs:357:     /// 可用(d)，逐盘，按读数里盘的次序：
crates/singlefs-core/src/admission.rs:358:     /// 容量 − 已分配 − 不可回收 − 挂载期承诺量 − 被抛弃根独占量
crates/singlefs-core/src/admission.rs:359:     /// − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数（D28（挂载期承诺量） 已定项 1）。
crates/singlefs-core/src/admission.rs:360:     #[must_use]
crates/singlefs-core/src/admission.rs:361:     pub fn available_on_each_device(&self) -> Vec<(DeviceIdentity, AvailableBytesOnOneDevice)> {
crates/singlefs-core/src/admission.rs:362:         let pool_wide_share_of_one_device = [
crates/singlefs-core/src/admission.rs:363:             self.pool_wide.pending_delete,
crates/singlefs-core/src/admission.rs:364:             self.pool_wide.committed_reservation,
crates/singlefs-core/src/admission.rs:365:             self.pool_wide.checkpoint_reserve_pool,
crates/singlefs-core/src/admission.rs:366:         ]
crates/singlefs-core/src/admission.rs:367:         .map(|commitment| i128::from(commitment.share_of_one_device(self.replicas).0));
crates/singlefs-core/src/admission.rs:368:         self.per_device
crates/singlefs-core/src/admission.rs:369:             .iter()
crates/singlefs-core/src/admission.rs:370:             .map(|terms| {
crates/singlefs-core/src/admission.rs:371:                 let own_terms = [
crates/singlefs-core/src/admission.rs:372:                     terms.allocated,
crates/singlefs-core/src/admission.rs:373:                     terms.unreclaimable,
crates/singlefs-core/src/admission.rs:374:                     terms.mount_time_commitment,
crates/singlefs-core/src/admission.rs:375:                     terms.abandoned_root_exclusive,
crates/singlefs-core/src/admission.rs:376:                 ]
crates/singlefs-core/src/admission.rs:377:                 .map(|term| i128::from(term.0));
crates/singlefs-core/src/admission.rs:378:                 let deducted: i128 = own_terms.iter().sum::<i128>()
crates/singlefs-core/src/admission.rs:379:                     + pool_wide_share_of_one_device.iter().sum::<i128>();
crates/singlefs-core/src/admission.rs:380:                 (
crates/singlefs-core/src/admission.rs:381:                     terms.device,
crates/singlefs-core/src/admission.rs:382:                     AvailableBytesOnOneDevice(i128::from(terms.capacity.0) - deducted),
crates/singlefs-core/src/admission.rs:383:                 )
crates/singlefs-core/src/admission.rs:384:             })
crates/singlefs-core/src/admission.rs:385:             .collect()
crates/singlefs-core/src/admission.rs:386:     }

逐项从哪来（读数由 `AdmissionReading::of_allocator` 取，原文）：
crates/singlefs-core/src/admission.rs:308:     /// 从分配器此刻的计数取逐盘各项：容量 = 单元区槽数、已分配 = 占着的槽数（含已释放还在窗口里的）、
crates/singlefs-core/src/admission.rs:309:     /// 被抛弃根独占量 = 影子账隔离的槽数，不可回收取 [`UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`]；副本数按
crates/singlefs-core/src/admission.rs:310:     /// [`ReplicaCount::of_every_device_in_the_pool`]。挂载期承诺量（每块盘同一个数）与全池三项由调用方给。
crates/singlefs-core/src/admission.rs:311:     ///
crates/singlefs-core/src/admission.rs:323:         let per_device = allocator
crates/singlefs-core/src/admission.rs:324:             .devices
crates/singlefs-core/src/admission.rs:325:             .iter()
crates/singlefs-core/src/admission.rs:326:             .map(|device_map| DeviceAdmissionTerms {
crates/singlefs-core/src/admission.rs:327:                 device: device_map.device,
crates/singlefs-core/src/admission.rs:328:                 capacity: BytesOnOneDevice::of_slots(device_map.unit_area_slots()),
crates/singlefs-core/src/admission.rs:329:                 allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),
crates/singlefs-core/src/admission.rs:330:                 unreclaimable: UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED,
crates/singlefs-core/src/admission.rs:331:                 mount_time_commitment: mount_time_commitment_on_each_device,
crates/singlefs-core/src/admission.rs:332:                 abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),
crates/singlefs-core/src/admission.rs:333:             })
crates/singlefs-core/src/admission.rs:334:             .collect();
crates/singlefs-core/src/admission.rs:335:         Self::new(
crates/singlefs-core/src/admission.rs:336:             per_device,
crates/singlefs-core/src/admission.rs:337:             pool_wide,
crates/singlefs-core/src/admission.rs:338:             ReplicaCount::of_every_device_in_the_pool(allocator),
crates/singlefs-core/src/admission.rs:339:         )

| 式子项 | 今天的来源 | 出处 |
|---|---|---|
| 容量 | `device_map.unit_area_slots()` 乘 16384 | admission.rs:328 |
| 已分配 | `device_map.allocated_slots()`（含已释放、仍在 defer 窗口的槽） | admission.rs:329；allocator.rs:256-258 |
| 不可回收 | 常量 `UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED` = 0 | admission.rs:330、147 |
| 挂载期承诺量 | `instance_switch_reserve_on_one_device(rows, c_max)` 每盘同一个数，由调用方给 | admission.rs:331、194-222 |
| 被抛弃根独占量 | `device_map.isolated_slots()` | admission.rs:332 |
| 待删占用 ÷ 副本数 | `PoolWideCommitments.pending_delete`，今天恒来自常量 0 | admission.rs:249、259-265 |
| 已承诺预留 ÷ 副本数 | `PoolWideCommitments.committed_reservation`，今天恒来自常量 0 | admission.rs:251、259-265 |
| checkpoint 保留池 ÷ 副本数 | `checkpoint_reserve_pool(ckpt_cost, 副本数)` | admission.rs:173-181、1021-1024 |

后三项「按全部副本之和记、每盘扣 副本之和 ÷ 副本数（向上取整）」的原文：
crates/singlefs-core/src/admission.rs:70: /// 全池的承诺量：按全部副本之和记的物理字节（C375（待删占用与已承诺预留按份还是按和记没有条款） 第三轮写回：
crates/singlefs-core/src/admission.rs:71: /// 与「已分配」同一口径；按一份副本记会让 D28（挂载期承诺量） 已定项 1 的式子与 I-3.4（可用空间扣待删占用） 当场为假）。
crates/singlefs-core/src/admission.rs:72: #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
crates/singlefs-core/src/admission.rs:73: pub struct BytesSummedOverAllReplicas(pub u64);
crates/singlefs-core/src/admission.rs:95:     /// 摊到每块盘扣的那一份：副本之和 ÷ 副本数（D28（挂载期承诺量） 已定项 1）。除不尽时向上取整——扣多不扣少，
crates/singlefs-core/src/admission.rs:96:     /// 可用只会少报、不会多报（式子是按字节的实数除法，条款没写取整；用 `of_one_copy_on_every_replica` 记下的量总除得尽）。
crates/singlefs-core/src/admission.rs:97:     #[must_use]
crates/singlefs-core/src/admission.rs:98:     pub fn share_of_one_device(self, replicas: ReplicaCount) -> BytesOnOneDevice {
crates/singlefs-core/src/admission.rs:99:         BytesOnOneDevice(self.0.div_ceil(replicas.replicas()))
crates/singlefs-core/src/admission.rs:100:     }

### a2 两个常量与 `PoolWideCommitments::of_the_first_version` 原文
crates/singlefs-core/src/admission.rs:149: /// 式子里的「待删占用」（记账第 4 项）：第一版不删对象、不销毁快照，没有「待删除但意图未完成」的结构 ⇒ 恒 0。
crates/singlefs-core/src/admission.rs:150: /// 记账树那一行池级「待删占用」写的就是它。
crates/singlefs-core/src/admission.rs:151: pub const PENDING_DELETE_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =
crates/singlefs-core/src/admission.rs:152:     BytesSummedOverAllReplicas::ZERO;
crates/singlefs-core/src/admission.rs:153: 
crates/singlefs-core/src/admission.rs:154: /// 式子里的「已承诺预留」（记账第 6 项，含墓碑）：第一版没有删除路径，墓碑的预付（D3（空间分配） 已定项 2，量是 C84（墓碑单元的粒度没人定）
crates/singlefs-core/src/admission.rs:155: /// 那一格）与别的预留都不发生 ⇒ 恒 0。记账树那一行池级「已承诺预留」写的就是它。
crates/singlefs-core/src/admission.rs:156: pub const COMMITTED_RESERVATION_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =
crates/singlefs-core/src/admission.rs:157:     BytesSummedOverAllReplicas::ZERO;
crates/singlefs-core/src/admission.rs:245: /// 式子后三项：全池的承诺量，按全部副本之和记，每块盘各扣「副本之和 ÷ 副本数」。
crates/singlefs-core/src/admission.rs:246: #[derive(Clone, Copy, Debug, PartialEq, Eq)]
crates/singlefs-core/src/admission.rs:247: pub struct PoolWideCommitments {
crates/singlefs-core/src/admission.rs:248:     /// 式子里的「待删占用」（记账第 4 项）。
crates/singlefs-core/src/admission.rs:249:     pub pending_delete: BytesSummedOverAllReplicas,
crates/singlefs-core/src/admission.rs:250:     /// 式子里的「已承诺预留」（记账第 6 项，含墓碑）。
crates/singlefs-core/src/admission.rs:251:     pub committed_reservation: BytesSummedOverAllReplicas,
crates/singlefs-core/src/admission.rs:252:     /// 式子里的「checkpoint 保留池」（`checkpoint_reserve_pool`）。
crates/singlefs-core/src/admission.rs:253:     pub checkpoint_reserve_pool: BytesSummedOverAllReplicas,
crates/singlefs-core/src/admission.rs:254: }
crates/singlefs-core/src/admission.rs:255: 
crates/singlefs-core/src/admission.rs:256: impl PoolWideCommitments {
crates/singlefs-core/src/admission.rs:257:     /// 第一版的待删占用与已承诺预留（两个 0，记账树那两行写的就是它们）；checkpoint 保留池由调用方给（C363）。
crates/singlefs-core/src/admission.rs:258:     #[must_use]
crates/singlefs-core/src/admission.rs:259:     pub fn of_the_first_version(checkpoint_reserve_pool: BytesSummedOverAllReplicas) -> Self {
crates/singlefs-core/src/admission.rs:260:         Self {
crates/singlefs-core/src/admission.rs:261:             pending_delete: PENDING_DELETE_OF_THE_FIRST_VERSION,
crates/singlefs-core/src/admission.rs:262:             committed_reservation: COMMITTED_RESERVATION_OF_THE_FIRST_VERSION,
crates/singlefs-core/src/admission.rs:263:             checkpoint_reserve_pool,
crates/singlefs-core/src/admission.rs:264:         }
crates/singlefs-core/src/admission.rs:265:     }
crates/singlefs-core/src/admission.rs:266: }

### a3 全部引用处（全量 grep，输出未截断）

命令一：`cd /tmp/claude-1000/m3-delete-r2/tree/crates; grep -rn 'PENDING_DELETE_OF_THE_FIRST_VERSION\|COMMITTED_RESERVATION_OF_THE_FIRST_VERSION' .`
原样输出，共 6 行：
    singlefs-core/src/admission.rs:151:pub const PENDING_DELETE_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =
    singlefs-core/src/admission.rs:156:pub const COMMITTED_RESERVATION_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =
    singlefs-core/src/admission.rs:261:            pending_delete: PENDING_DELETE_OF_THE_FIRST_VERSION,
    singlefs-core/src/admission.rs:262:            committed_reservation: COMMITTED_RESERVATION_OF_THE_FIRST_VERSION,
    singlefs-core/src/transaction.rs:6284:                crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0
    singlefs-core/src/transaction.rs:6287:                crate::admission::COMMITTED_RESERVATION_OF_THE_FIRST_VERSION.0

命令二：`grep -rn 'PoolWideCommitments::of_the_first_version\|fn of_the_first_version' .`
原样输出，共 6 行：
    singlefs-core/src/admission.rs:259:    pub fn of_the_first_version(checkpoint_reserve_pool: BytesSummedOverAllReplicas) -> Self {
    singlefs-core/src/admission.rs:1021:        PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(
    singlefs-core/src/admission.rs:1215:            PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(
    singlefs-core/src/admission.rs:1416:            PoolWideCommitments::of_the_first_version(BytesSummedOverAllReplicas::ZERO),
    singlefs-harness/tests/release_checksum_quarantine.rs:805:            PoolWideCommitments::of_the_first_version(BytesSummedOverAllReplicas::ZERO),
    singlefs-harness/tests/admission_formula_on_the_mounted_state.rs:172:        PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(

命令三（原问法，两个常量加小写的 `of_the_first_version` 一起）：`grep -rn 'PENDING_DELETE_OF_THE_FIRST_VERSION\|COMMITTED_RESERVATION_OF_THE_FIRST_VERSION\|of_the_first_version' . | wc -l` 数出 36 行。
其中 6 行是命令一、6 行是命令二（两组不重叠，大小写不同），余下 24 行是撞名的无关项（`data_slot_of_the_first_version` 变量与函数、`rollback_by_a_forward_publish.rs` 的用例名、`crates/mutations.tsv:733` 一条变异行），全量输出存在 `/tmp/claude-1000/m3-prepay-facts/refs.txt`（草稿，未入库）。

分类（行号见上面两组输出）：
- 产品路径（非测试）引用：`admission.rs:261-262`（`of_the_first_version` 函数体里读两个常量）、`admission.rs:1021`（`admission_reading_with_the_switch_reserve_of` 里调 `of_the_first_version`，今天产品路径上唯一的调用）、`transaction.rs:6284`、`transaction.rs:6287`（记账行池级两行的取值闭包读两个常量）。
- 测试引用：`admission.rs:1215`、`admission.rs:1416`（在 `mod tests`，始于 `admission.rs:1075`）；`crates/singlefs-harness/tests/release_checksum_quarantine.rs:805`；`crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs:172`。
- `crates/singlefs-checker`、`crates/singlefs-checker-tier`、`crates/singlefs-format`、`crates/singlefs-replay-prefix` 里这几个名字零命中（命令一、二的输出里没有这些目录）。
- `PoolWideCommitments` 本身的其余引用（命令：`grep -rn 'PoolWideCommitments' .`，共 15 行）里，`admission.rs:1140`、`admission.rs:1176` 是测试里直接写字面结构体 `PoolWideCommitments { … }`，即结构体的三个字段是 `pub` 的、调用方可以直接给非零值（`admission.rs:247-254`）。

### a4 一次发布的需求怎么算与判不判

函数原文（`demand_of_the_roles_on_each_device`、`publish_has_an_ordinary_allocation`）：
crates/singlefs-core/src/admission.rs:940: /// 这次发布判不判空间准入（D28（挂载期承诺量） 已定项 1 接线「判不判照旧」）：重写的角色里至少一个是普通分配
crates/singlefs-core/src/admission.rs:941: /// （[`space_budget_of_role`] 是 [`SpaceBudgetOfARole::OrdinaryAllocation`]）才判；没有普通分配的发布（空发布、写行、暖机、抬 F）不判。
crates/singlefs-core/src/admission.rs:942: #[must_use]
crates/singlefs-core/src/admission.rs:943: pub fn publish_has_an_ordinary_allocation(rewritten_roles: &[TransactionUnit]) -> bool {
crates/singlefs-core/src/admission.rs:944:     rewritten_roles
crates/singlefs-core/src/admission.rs:945:         .iter()
crates/singlefs-core/src/admission.rs:946:         .any(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation)
crates/singlefs-core/src/admission.rs:947: }
crates/singlefs-core/src/admission.rs:948: 
crates/singlefs-core/src/admission.rs:949: /// 一次发布的需求：这次新写的全部角色（普通分配加这次写出的固定点，不加这次换下的槽，D28（挂载期承诺量） 已定项 1 接线，
crates/singlefs-core/src/admission.rs:950: /// 用户 2026-09-25 定 ND+A1n）要在每块盘上新占的物理字节：
crates/singlefs-core/src/admission.rs:951: /// 第一版每个单元落池里每一块盘（`ReplicaCount::of_every_device_in_the_pool` 同一条理由），每块盘的需求相同；
crates/singlefs-core/src/admission.rs:952: /// 按盘字节记（C370（需求、可用与 df 没有共同单位） 2026-09-17 收窄：需求要与逐盘物理字节的「已分配」同口径）。
crates/singlefs-core/src/admission.rs:953: /// 按 `devices` 的次序每块盘一条，需求为零也写 0（`admit_on_every_device` 要逐盘写全）。
crates/singlefs-core/src/admission.rs:954: ///
crates/singlefs-core/src/admission.rs:955: /// # Panics
crates/singlefs-core/src/admission.rs:956: /// 字节数装不进 u64：一次发布的角色数有上界（一版的节点数），装不下说明调用方给的不是一次发布的角色清单。
crates/singlefs-core/src/admission.rs:957: #[must_use]
crates/singlefs-core/src/admission.rs:958: pub fn demand_of_the_roles_on_each_device(
crates/singlefs-core/src/admission.rs:959:     rewritten_roles: &[TransactionUnit],
crates/singlefs-core/src/admission.rs:960:     devices: &[DeviceIdentity],
crates/singlefs-core/src/admission.rs:961: ) -> Vec<DemandOnDevice> {
crates/singlefs-core/src/admission.rs:962:     let slots_of_the_demand: u64 = rewritten_roles.iter().map(|role| role.span_slots()).sum();
crates/singlefs-core/src/admission.rs:963:     let bytes = BytesOnOneDevice::of_slots(slots_of_the_demand);
crates/singlefs-core/src/admission.rs:964:     devices
crates/singlefs-core/src/admission.rs:965:         .iter()
crates/singlefs-core/src/admission.rs:966:         .map(|device| DemandOnDevice {
crates/singlefs-core/src/admission.rs:967:             device: *device,
crates/singlefs-core/src/admission.rs:968:             bytes,
crates/singlefs-core/src/admission.rs:969:         })
crates/singlefs-core/src/admission.rs:970:         .collect()
crates/singlefs-core/src/admission.rs:971: }

每个角色占几个槽（`TransactionUnit::span_slots`，需求就是它的和乘 16384）：
crates/singlefs-core/src/transaction.rs:2183:     pub fn span_slots(self) -> u64 {
crates/singlefs-core/src/transaction.rs:2184:         match self.placement() {
crates/singlefs-core/src/transaction.rs:2185:             PlacementRule::UserData => 2,
crates/singlefs-core/src/transaction.rs:2186:             PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned) => 2,
crates/singlefs-core/src/transaction.rs:2187:             PlacementRule::CommitGenerated(UnitFootprint::OneSlot) => 1,
crates/singlefs-core/src/transaction.rs:2188:         }
crates/singlefs-core/src/transaction.rs:2189:     }

### a5 哪些角色走普通分配、哪些走保留池、哪些走切换预留（`space_budget_of_role`，原文）
crates/singlefs-core/src/admission.rs:900: /// 一次发布里一个角色的空间归哪一格（依据见各成员）：一次发布判不判空间准入，看它重写的角色里有没有
crates/singlefs-core/src/admission.rs:901: /// [`SpaceBudgetOfARole::OrdinaryAllocation`]（[`publish_has_an_ordinary_allocation`]）；判的时候需求是这次新写的全部角色，
crates/singlefs-core/src/admission.rs:902: /// 三格都算（[`demand_of_the_roles_on_each_device`]，D28（挂载期承诺量） 已定项 1 接线）。
crates/singlefs-core/src/admission.rs:903: #[derive(Clone, Copy, Debug, PartialEq, Eq)]
crates/singlefs-core/src/admission.rs:904: pub enum SpaceBudgetOfARole {
crates/singlefs-core/src/admission.rs:905:     /// 普通分配：用户数据单元，与因为用户这次改了内容才重写的 extent 树、inode 树的节点。保留池对它是纯税
crates/singlefs-core/src/admission.rs:906:     /// （D23（journal 的角色与格式） 已定项 24「保留池是给 checkpoint 开的一道地板，对普通分配是纯税」）。
crates/singlefs-core/src/admission.rs:907:     OrdinaryAllocation,
crates/singlefs-core/src/admission.rs:908:     /// checkpoint 自己的固定点：ckpt_cost 的 Σ 名单里那几棵（分配记录树、中央映射树、记账树）与树表。
crates/singlefs-core/src/admission.rs:909:     /// 没有普通分配的发布（空发布、暖机、抬 F）只写它们，空间从式子第八项 checkpoint 保留池里出
crates/singlefs-core/src/admission.rs:910:     /// （D28（挂载期承诺量） 已定项 1「它保证 checkpoint 自己的固定点写得出去」），不判。
crates/singlefs-core/src/admission.rs:911:     CheckpointReservePool,
crates/singlefs-core/src/admission.rs:912:     /// 实例表链：写行那次整条重写，从挂载期承诺量里的实例切换预留出（D28（挂载期承诺量） 已定项 3「多的一份给写行那次发布的元数据」、
crates/singlefs-core/src/admission.rs:913:     /// 已定项 4「实例表链不进 ckpt_cost，它的开销归已定项 3 的切换预留」），不判。
crates/singlefs-core/src/admission.rs:914:     InstanceSwitchReserve,
crates/singlefs-core/src/admission.rs:915: }
crates/singlefs-core/src/admission.rs:916: 
crates/singlefs-core/src/admission.rs:917: /// 这个角色的空间归哪一格（[`SpaceBudgetOfARole`]）。
crates/singlefs-core/src/admission.rs:918: #[must_use]
crates/singlefs-core/src/admission.rs:919: pub const fn space_budget_of_role(role: TransactionUnit) -> SpaceBudgetOfARole {
crates/singlefs-core/src/admission.rs:920:     match role {
crates/singlefs-core/src/admission.rs:921:         TransactionUnit::Data(_)
crates/singlefs-core/src/admission.rs:922:         | TransactionUnit::ExtentLowerNode(_)
crates/singlefs-core/src/admission.rs:923:         | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
crates/singlefs-core/src/admission.rs:924:         | TransactionUnit::ExtentRoot
crates/singlefs-core/src/admission.rs:925:         | TransactionUnit::InodeLeafContainer(_)
crates/singlefs-core/src/admission.rs:926:         | TransactionUnit::InodeRoot => SpaceBudgetOfARole::OrdinaryAllocation,
crates/singlefs-core/src/admission.rs:927:         TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
crates/singlefs-core/src/admission.rs:928:         | TransactionUnit::AllocationTree
crates/singlefs-core/src/admission.rs:929:         | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
crates/singlefs-core/src/admission.rs:930:         | TransactionUnit::AccountingTree
crates/singlefs-core/src/admission.rs:931:         | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
crates/singlefs-core/src/admission.rs:932:         | TransactionUnit::MappingTree
crates/singlefs-core/src/admission.rs:933:         | TransactionUnit::TreeTable => SpaceBudgetOfARole::CheckpointReservePool,
crates/singlefs-core/src/admission.rs:934:         TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
crates/singlefs-core/src/admission.rs:935:             SpaceBudgetOfARole::InstanceSwitchReserve
crates/singlefs-core/src/admission.rs:936:         }
crates/singlefs-core/src/admission.rs:937:     }
crates/singlefs-core/src/admission.rs:938: }

普通分配那一半实际重写的角色清单（`OrdinaryAllocationPartOfThePublish::rewritten_roles`，原文）：
crates/singlefs-core/src/transaction.rs:4192: impl OrdinaryAllocationPartOfThePublish {
crates/singlefs-core/src/transaction.rs:4193:     /// 这一半重写的角色（`admission::space_budget_of_role` 都是普通分配）：数据单元、extent 树的节点、inode 叶容器与 inode 根，
crates/singlefs-core/src/transaction.rs:4194:     /// 与 `PublishPlan::resolve` 排进 bump 次序的是同一批（实例表的几片夹在数据单元与 extent 树之间，不在这一半里）。
crates/singlefs-core/src/transaction.rs:4195:     fn rewritten_roles(&self) -> Vec<TransactionUnit> {
crates/singlefs-core/src/transaction.rs:4196:         let mut roles: Vec<TransactionUnit> = self
crates/singlefs-core/src/transaction.rs:4197:             .file_content_transactions
crates/singlefs-core/src/transaction.rs:4198:             .iter()
crates/singlefs-core/src/transaction.rs:4199:             .map(|transaction| TransactionUnit::Data(transaction.unit_index_in_file))
crates/singlefs-core/src/transaction.rs:4200:             .collect();
crates/singlefs-core/src/transaction.rs:4201:         roles.extend(self.extent_tree.rewritten_roles());
crates/singlefs-core/src/transaction.rs:4202:         roles.extend(
crates/singlefs-core/src/transaction.rs:4203:             self.inode_tree
crates/singlefs-core/src/transaction.rs:4204:                 .rewritten
crates/singlefs-core/src/transaction.rs:4205:                 .iter()
crates/singlefs-core/src/transaction.rs:4206:                 .map(|index| TransactionUnit::InodeLeafContainer(*index)),
crates/singlefs-core/src/transaction.rs:4207:         );
crates/singlefs-core/src/transaction.rs:4208:         if !self.inode_tree.rewritten.is_empty() {
crates/singlefs-core/src/transaction.rs:4209:             roles.push(TransactionUnit::InodeRoot);
crates/singlefs-core/src/transaction.rs:4210:         }
crates/singlefs-core/src/transaction.rs:4211:         roles
crates/singlefs-core/src/transaction.rs:4212:     }

由此（推的，依据是上面两段原文）：建 inode 的发布（`publish_new_inodes`）重写 `InodeLeafContainer` 与 `InodeRoot`，两个都是普通分配，所以它被判；
不带文件内容、也不新建 inode 的发布（写行、暖机、抬 F、回退）这一半为空，不判。

## (b) 写文件的入口今天在哪判准入

### b1 四个入口到准入的调用链

四个 `publish_*` 公开入口都收进同一条路：`publish_version_of_trees` → `prepare_the_version_publish`。入口各自的行：
crates/singlefs-core/src/transaction.rs:4624: pub fn publish_first_file<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4625:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4626:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4627:     version_to_build_on: &RootRecord,
crates/singlefs-core/src/transaction.rs:4628:     file: FirstFile<'_>,
crates/singlefs-core/src/transaction.rs:4629:     instance: InstanceGeneration,
crates/singlefs-core/src/transaction.rs:4630:     previous_record_bytes: &[u8],
crates/singlefs-core/src/transaction.rs:4631: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:4671:     allocator.record_zero_unit_roots_leading_to(version_to_build_on.checkpoint_txg);
crates/singlefs-core/src/transaction.rs:4672:     publish_version_of_trees_holding_one_data_unit(
crates/singlefs-core/src/transaction.rs:4673:         pool,
crates/singlefs-core/src/transaction.rs:4719: pub fn publish_overwrite<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4720:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4721:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4722:     previous: &TransactionOutput,
crates/singlefs-core/src/transaction.rs:4723:     file: FirstFile<'_>,
crates/singlefs-core/src/transaction.rs:4724:     instance: InstanceGeneration,
crates/singlefs-core/src/transaction.rs:4725: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:4726:     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
crates/singlefs-core/src/transaction.rs:4727:     publish_version_holding_one_data_unit(
crates/singlefs-core/src/transaction.rs:4728:         pool,
crates/singlefs-core/src/transaction.rs:4772: pub fn publish_sequential_write<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4773:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4774:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4775:     previous: &TransactionOutput,
crates/singlefs-core/src/transaction.rs:4776:     file: FirstFile<'_>,
crates/singlefs-core/src/transaction.rs:4777:     instance: InstanceGeneration,
crates/singlefs-core/src/transaction.rs:4778: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:4779:     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
crates/singlefs-core/src/transaction.rs:4780:     // 改动计数与新池新建文件号先取成局部量、字段也换了次序：`crates/mutations.tsv` 有几条变异按 `publish_overwrite` 里
crates/singlefs-core/src/transaction.rs:4781:     // `change_count: txg.0,`、`transaction: previous.highest_transaction_number_in_this_instance + 1,` 与那几行字段的原样
crates/singlefs-core/src/transaction.rs:4782:     // 锚着，同一份文件里再出现一次它们就腐化（门禁 59 号；`publish_new_inodes` 里那句注释是同一件事）。
crates/singlefs-core/src/transaction.rs:4783:     let change_count = txg.0;
crates/singlefs-core/src/transaction.rs:4784:     let new_pool_file_creation_of_this_request =
crates/singlefs-core/src/transaction.rs:4785:         previous.highest_transaction_number_in_this_instance + 1;
crates/singlefs-core/src/transaction.rs:4786:     publish_version(
crates/singlefs-core/src/transaction.rs:4787:         pool,
crates/singlefs-core/src/transaction.rs:4830: pub fn publish_new_inodes<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4831:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4832:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4833:     previous: &TransactionOutput,
crates/singlefs-core/src/transaction.rs:4834:     new_inode_count: u64,
crates/singlefs-core/src/transaction.rs:4835:     write_time_seconds: u64,
crates/singlefs-core/src/transaction.rs:4836:     instance: InstanceGeneration,
crates/singlefs-core/src/transaction.rs:4837: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:4838:     let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
crates/singlefs-core/src/transaction.rs:4839:     let first_new_inode_number = previous.inode_number_watermark();
crates/singlefs-core/src/transaction.rs:4840:     // 改动计数与事务号都先取成局部量：`crates/mutations.tsv` 有几条变异按 `change_count: txg.0,` 与
crates/singlefs-core/src/transaction.rs:4841:     // `transaction: previous.highest_transaction_number_in_this_instance + 1,` 这两串字面锚在 `publish_overwrite` 上，
crates/singlefs-core/src/transaction.rs:4842:     // 同一份文件里再出现一次它们就腐化（门禁 59 号；`publish_first_file` 里那句注释是同一件事）。
crates/singlefs-core/src/transaction.rs:4843:     // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）；建出来那次就是这次发布。
crates/singlefs-core/src/transaction.rs:4844:     let change_count = txg.0;
crates/singlefs-core/src/transaction.rs:4845:     let transaction_number = previous.highest_transaction_number_in_this_instance + 1;
crates/singlefs-core/src/transaction.rs:4846:     let new_inode_records: Vec<InodeRecord> = (0..new_inode_count)
crates/singlefs-core/src/transaction.rs:4847:         .map(|offset_from_the_watermark| InodeRecord {
crates/singlefs-core/src/transaction.rs:4848:             inode: first_new_inode_number + offset_from_the_watermark,
crates/singlefs-core/src/transaction.rs:4849:             // 对象出生代 = 创建那次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 8）。
crates/singlefs-core/src/transaction.rs:4850:             object_birth: txg,
crates/singlefs-core/src/transaction.rs:4851:             // 建出来还没写过数据：长度 0，blocks 跟着是 0。
crates/singlefs-core/src/transaction.rs:4852:             size: 0,
crates/singlefs-core/src/transaction.rs:4853:             change_count,
crates/singlefs-core/src/transaction.rs:4854:             write_time_seconds,
crates/singlefs-core/src/transaction.rs:4855:         })
crates/singlefs-core/src/transaction.rs:4856:         .collect();
crates/singlefs-core/src/transaction.rs:4857:     publish_version(
crates/singlefs-core/src/transaction.rs:4858:         pool,

收拢的几层（原文）：
crates/singlefs-core/src/transaction.rs:4900: /// `publish_overwrite` 那一路：先按一个数据单元的契约判内容，再走 `publish_version`。
crates/singlefs-core/src/transaction.rs:4901: fn publish_version_holding_one_data_unit<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4902:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4903:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4904:     plan: PublishPlan<'_>,
crates/singlefs-core/src/transaction.rs:4905:     previous: Option<&TransactionOutput>,
crates/singlefs-core/src/transaction.rs:4906: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:4907:     refuse_a_file_that_does_not_fit_one_data_unit(&plan)?;
crates/singlefs-core/src/transaction.rs:4908:     publish_version(pool, allocator, plan, previous)
crates/singlefs-core/src/transaction.rs:4909: }
crates/singlefs-core/src/transaction.rs:4910: 
crates/singlefs-core/src/transaction.rs:4911: /// `publish_first_file` 那一路：先按一个数据单元的契约判内容，再走 `publish_version_of_trees`（八棵树的号由它给）。
crates/singlefs-core/src/transaction.rs:4912: fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4913:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4914:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4915:     plan: PublishPlan<'_>,
crates/singlefs-core/src/transaction.rs:4916:     previous: Option<&TransactionOutput>,
crates/singlefs-core/src/transaction.rs:4917:     trees: FileVersionTreeIdentifiers,
crates/singlefs-core/src/transaction.rs:4918: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:4919:     refuse_a_file_that_does_not_fit_one_data_unit(&plan)?;
crates/singlefs-core/src/transaction.rs:4920:     publish_version_of_trees(
crates/singlefs-core/src/transaction.rs:4921:         pool,
crates/singlefs-core/src/transaction.rs:4922:         allocator,
crates/singlefs-core/src/transaction.rs:4923:         plan,
crates/singlefs-core/src/transaction.rs:4924:         previous,
crates/singlefs-core/src/transaction.rs:4925:         trees,
crates/singlefs-core/src/transaction.rs:4926:         UnmountMarker::NotWrittenByTheUnmountSequence,
crates/singlefs-core/src/transaction.rs:4927:     )
crates/singlefs-core/src/transaction.rs:4928: }
crates/singlefs-core/src/transaction.rs:4947: pub fn publish_version<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4948:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4949:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4950:     plan: PublishPlan<'_>,
crates/singlefs-core/src/transaction.rs:4951:     previous: Option<&TransactionOutput>,
crates/singlefs-core/src/transaction.rs:4952: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:4953:     publish_version_with_unmount_marker(
crates/singlefs-core/src/transaction.rs:4954:         pool,
crates/singlefs-core/src/transaction.rs:4955:         allocator,
crates/singlefs-core/src/transaction.rs:4956:         plan,
crates/singlefs-core/src/transaction.rs:4957:         previous,
crates/singlefs-core/src/transaction.rs:4958:         UnmountMarker::NotWrittenByTheUnmountSequence,
crates/singlefs-core/src/transaction.rs:4959:     )
crates/singlefs-core/src/transaction.rs:4994: fn publish_version_of_trees<Device: BlockDevice>(
crates/singlefs-core/src/transaction.rs:4995:     pool: &mut PoolWriter<'_, Device>,
crates/singlefs-core/src/transaction.rs:4996:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/transaction.rs:4997:     plan: PublishPlan<'_>,
crates/singlefs-core/src/transaction.rs:4998:     previous: Option<&TransactionOutput>,
crates/singlefs-core/src/transaction.rs:4999:     trees: FileVersionTreeIdentifiers,
crates/singlefs-core/src/transaction.rs:5000:     unmount_marker: UnmountMarker,
crates/singlefs-core/src/transaction.rs:5001: ) -> Result<TransactionOutput, PublishError> {
crates/singlefs-core/src/transaction.rs:5002:     // 第一道：冻结着一次没重发的发布就在任何读写之前拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
crates/singlefs-core/src/transaction.rs:5003:     refuse_while_a_publish_is_frozen(allocator)?;
crates/singlefs-core/src/transaction.rs:5004:     // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
crates/singlefs-core/src/transaction.rs:5005:     let allocator_before_this_publish = allocator.clone();
crates/singlefs-core/src/transaction.rs:5006:     let built = prepare_the_version_publish(
crates/singlefs-core/src/transaction.rs:5007:         pool,
crates/singlefs-core/src/transaction.rs:5008:         allocator,
crates/singlefs-core/src/transaction.rs:5009:         &plan,
crates/singlefs-core/src/transaction.rs:5010:         previous,
crates/singlefs-core/src/transaction.rs:5011:         trees,
crates/singlefs-core/src/transaction.rs:5012:         ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
crates/singlefs-core/src/transaction.rs:5013:         unmount_marker,
crates/singlefs-core/src/transaction.rs:5014:     );

### b2 `prepare_the_version_publish` 里读准入的两处（每次发布各一次，都在动分配器与任何写之前）

第一处，「这次的单元落不落得下」（`admit_the_units_landing_on_every_device`），原文：
crates/singlefs-core/src/transaction.rs:5077:     let ordinary_allocation_roles = plan
crates/singlefs-core/src/transaction.rs:5078:         .ordinary_allocation_part(previous, &trees)?
crates/singlefs-core/src/transaction.rs:5079:         .rewritten_roles();
crates/singlefs-core/src/transaction.rs:5080:     let units_landing_is_judged = match allocator.space_admission() {
crates/singlefs-core/src/transaction.rs:5081:         SpaceAdmission::JudgedByTheFormula => {
crates/singlefs-core/src/transaction.rs:5082:             publish_has_an_ordinary_allocation(&ordinary_allocation_roles)
crates/singlefs-core/src/transaction.rs:5083:         }
crates/singlefs-core/src/transaction.rs:5084:         SpaceAdmission::SkippedByTheTestOnlySwitch => {
crates/singlefs-core/src/transaction.rs:5085:             // 关掉准入的那一档这一判与式子那一判都不判，留给落点那一道在任何写之前拒。
crates/singlefs-core/src/transaction.rs:5086:             false
crates/singlefs-core/src/transaction.rs:5087:         }
crates/singlefs-core/src/transaction.rs:5088:     };
crates/singlefs-core/src/transaction.rs:5089:     if units_landing_is_judged {
crates/singlefs-core/src/transaction.rs:5090:         admit_the_units_landing_on_every_device(
crates/singlefs-core/src/transaction.rs:5091:             &PlacementRoomOnOneDevice::of_every_device_of_the_allocator(allocator),
crates/singlefs-core/src/transaction.rs:5092:             units_of_a_publish_to_land(
crates/singlefs-core/src/transaction.rs:5093:                 &ordinary_allocation_roles,
crates/singlefs-core/src/transaction.rs:5094:                 checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
crates/singlefs-core/src/transaction.rs:5095:                     previous,
crates/singlefs-core/src/transaction.rs:5096:                     allocator,
crates/singlefs-core/src/transaction.rs:5097:                     pool.code_two_tree_node_capacities(),
crates/singlefs-core/src/transaction.rs:5098:                 ),
crates/singlefs-core/src/transaction.rs:5099:             ),
crates/singlefs-core/src/transaction.rs:5100:         )
crates/singlefs-core/src/transaction.rs:5101:         .map_err(PublishError::SpaceAdmissionRefused)?;
crates/singlefs-core/src/transaction.rs:5102:     }

第二处，式子那一判，需求在这里算，读数在这里取，原文：
crates/singlefs-core/src/transaction.rs:5125:     let demand = demand_of_the_roles_on_each_device(
crates/singlefs-core/src/transaction.rs:5126:         &settled.resolved.rewritten_roles,
crates/singlefs-core/src/transaction.rs:5127:         &device_identities_of_the_accounting_rows,
crates/singlefs-core/src/transaction.rs:5128:     );
crates/singlefs-core/src/transaction.rs:5129:     let demand_is_judged = match allocator.space_admission() {
crates/singlefs-core/src/transaction.rs:5130:         SpaceAdmission::JudgedByTheFormula => {
crates/singlefs-core/src/transaction.rs:5131:             publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)
crates/singlefs-core/src/transaction.rs:5132:         }
crates/singlefs-core/src/transaction.rs:5133:         SpaceAdmission::SkippedByTheTestOnlySwitch => false,
crates/singlefs-core/src/transaction.rs:5134:     };
crates/singlefs-core/src/transaction.rs:5135:     if demand_is_judged {
crates/singlefs-core/src/transaction.rs:5136:         admit_on_every_device(
crates/singlefs-core/src/transaction.rs:5137:             &admission_reading_before_a_publish(allocator, previous),
crates/singlefs-core/src/transaction.rs:5138:             &demand,
crates/singlefs-core/src/transaction.rs:5139:         )
crates/singlefs-core/src/transaction.rs:5140:         .map_err(PublishError::SpaceAdmissionRefused)?;
crates/singlefs-core/src/transaction.rs:5141:     }

两处之间是分配记录树固定点（`settle_the_allocation_record_tree`，`transaction.rs:5103-5110`）与记录条数上限（`transaction.rs:5112-5119`）。
之后才是读盘核、`publish_admitted`（`transaction.rs:5163`）；释放与取落点在 `publish_admitted` 里（`transaction.rs:6086-6112`），所以准入读数取的是释放与分配之前那一刻的分配器计数。
覆盖写的入口 `publish_overwrite`（`transaction.rs:4719`）与 `publish_sequential_write`（`transaction.rs:4772`）本身一行准入都不判，同一条路径。

### b3 需求怎么算（每个入口的需求）

需求只由 `settled.resolved.rewritten_roles`（这次新写的全部角色）定，取 `demand_of_the_roles_on_each_device`（见 a4）：每个角色的 `span_slots` 之和乘 16384，每块盘同一个数；不加这次换下的槽。
文件内容切成几个数据单元由 `write_request_split` 定（`PublishPlan::file_content_transactions`，`transaction.rs:4220-4228`）；第一版契约：
- `publish_first_file`、`publish_overwrite`：内容装不进一个数据单元就在动分配器前报 `ContentExceedsDataUnit`（`transaction.rs:4885-4898`）。
- `publish_sequential_write`：按切分纪律切成 N 个一单元事务，N 个数据单元（`transaction.rs:4756-4766` 的文档）。
- `publish_new_inodes`：不写数据单元，重写 inode 叶容器与根（`transaction.rs:4816-4823`）。

### b4 要在这里加「将来删除的预付」，需求那一边今天有没有现成的挂点（以下除原文外都是「推的」）

今天的事实：
crates/singlefs-core/src/transaction.rs:3551: pub struct PublishPlan<'content> {
crates/singlefs-core/src/transaction.rs:3566:     pub highest_transaction_number_before_this_publish: u64,
crates/singlefs-core/src/transaction.rs:3567:     pub instance: InstanceGeneration,
crates/singlefs-core/src/transaction.rs:3568:     /// 反向链：上一条记录头的 CRC；本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
crates/singlefs-core/src/transaction.rs:3569:     pub back_chain: u32,
crates/singlefs-core/src/transaction.rs:3570:     pub file: Option<FileVersionPlan<'content>>,
crates/singlefs-core/src/transaction.rs:3571:     /// 这次发布往 inode 树里新建的 inode，按 inode 号严格升序（建 N 个文件那一路，里程碑「覆盖写、释放、回退与复用」并行线三）。
crates/singlefs-core/src/transaction.rs:3572:     /// 号由调用方按记账里的水位发（D5（快照 / 空间记账机制） 已定项 4 第 12 项），落在哪一片叶、要不要在末尾分裂
crates/singlefs-core/src/transaction.rs:3573:     /// 由 `crate::inode_tree` 在任何落盘动作之前算。别的发布给空的：文件版本那次只换它自己那条记录，写行与暖机不碰 inode 树。
crates/singlefs-core/src/transaction.rs:3574:     pub new_inode_records: &'content [InodeRecord],
crates/singlefs-core/src/transaction.rs:3575:     pub instance_table: InstanceTablePlan,
crates/singlefs-core/src/transaction.rs:3576:     /// 树表条目的诞生 txg：树建起来那次发布，之后每一版重写都不改。
crates/singlefs-core/src/transaction.rs:3577:     pub tree_birth_txg: CheckpointTxg,
crates/singlefs-core/src/transaction.rs:3578:     /// 这次发布写进根记录与记录新根段的树 ID 水位（D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)）。
crates/singlefs-core/src/transaction.rs:3579:     pub tree_identifier_watermark: u64,
crates/singlefs-core/src/transaction.rs:3580:     pub rollback_floor: CheckpointTxg,
crates/singlefs-core/src/transaction.rs:3581: }
- `PublishPlan` 的全部 `pub` 字段声明行（`awk 'NR>=3551 && NR<=3581 && /^    pub /'` 原样输出，共 12 个字段）：
    transaction.rs:3552 pub txg、3555 pub counter、3559 pub transaction、3566 pub highest_transaction_number_before_this_publish、3567 pub instance、3569 pub back_chain、3570 pub file、3574 pub new_inode_records、3575 pub instance_table、3577 pub tree_birth_txg、3579 pub tree_identifier_watermark、3580 pub rollback_floor（各行原文见 `crates/singlefs-core/src/transaction.rs` 同行号；3552 是 `    pub txg: CheckpointTxg,` 这样的形式）。
  其中没有「预付量」「将来删除的量」这类字段。
- `demand_of_the_roles_on_each_device(rewritten_roles, devices)`（`admission.rs:958-961`）只收角色清单与盘表，返回 `Vec<DemandOnDevice>`；调用点只有 `transaction.rs:5125`（产品路径）。
- 读数一侧：`admission_reading_before_a_publish(allocator, previous)`（`admission.rs:1031-1034`）→ `admission_reading_with_the_switch_reserve_of`（`admission.rs:1005`）在 `admission.rs:1021` 写死 `PoolWideCommitments::of_the_first_version(…)`；
  `AdmissionReading::of_allocator` 本身收一个 `pool_wide: PoolWideCommitments` 参数（`admission.rs:318-322`），所以读数构造器有入参、只是上面那一层没往里传。
- `PoolAllocator` 今天没有存放「已承诺预留」「待删占用」的字段（`grep -rn 'committed_reservation\|pending_delete' crates/singlefs-core/src` 只命中 `admission.rs` 的 `PoolWideCommitments` 与测试）；
  它有两个只住内存的挂载级字段可作类比：`instance_rows_after_this_mounts_row_publish`、`space_admission`（`allocator.rs:1064`、`1067`，注释说「只住内存、每次挂载重算」）。分配器在每次发布前整份 `clone`、失败换回（`transaction.rs:5005`、`5018`）。
- 推的：需求一侧今天有两个接口形的位置——`transaction.rs:5125` 的调用点（需求是 `Vec<DemandOnDevice>`，加一项要在这里并进去）与 `admission.rs:1021` 的读数构造；
  没有现成的字段或参数在带一个「预付量」。
- 推的：判不判由 `publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)`（`transaction.rs:5131`）定，只看角色里有没有普通分配；今天没有任何一类发布因为别的理由被判。

## (c) 记账行

### c1 每次发布写哪几行池级记账

带文件的发布（所有 `publish_version` 一族：写文件、建 inode、写行、暖机、抬 F、回退）在 `publish_admitted`（`transaction.rs:6048`）里装记账行，条目数每次相同：池级 3 行 + 每盘 6 行（共 3 + 6 × 盘数，恢复时判这个数，`recovery.rs:3497-3508`）。行表原文：
crates/singlefs-core/src/transaction.rs:4029: /// 记账树里不带设备维的三行（D5（快照 / 空间记账机制） 已定项 8）。
crates/singlefs-core/src/transaction.rs:4030: #[derive(Clone, Copy, Debug, PartialEq, Eq)]
crates/singlefs-core/src/transaction.rs:4031: enum PoolWideAccountingRow {
crates/singlefs-core/src/transaction.rs:4032:     InodeNumberWatermark,
crates/singlefs-core/src/transaction.rs:4033:     PendingDeleteBytes,
crates/singlefs-core/src/transaction.rs:4034:     CommittedReservationBytes,
crates/singlefs-core/src/transaction.rs:4035: }
crates/singlefs-core/src/transaction.rs:4036: 
crates/singlefs-core/src/transaction.rs:4037: impl PoolWideAccountingRow {
crates/singlefs-core/src/transaction.rs:4038:     const ALL: [Self; 3] = [
crates/singlefs-core/src/transaction.rs:4039:         PoolWideAccountingRow::InodeNumberWatermark,
crates/singlefs-core/src/transaction.rs:4040:         PoolWideAccountingRow::PendingDeleteBytes,
crates/singlefs-core/src/transaction.rs:4041:         PoolWideAccountingRow::CommittedReservationBytes,
crates/singlefs-core/src/transaction.rs:4042:     ];
crates/singlefs-core/src/transaction.rs:4043: 
crates/singlefs-core/src/transaction.rs:4044:     const fn statistic(self) -> u16 {
crates/singlefs-core/src/transaction.rs:4045:         match self {
crates/singlefs-core/src/transaction.rs:4046:             PoolWideAccountingRow::InodeNumberWatermark => STATISTIC_INODE_WATERMARK,
crates/singlefs-core/src/transaction.rs:4047:             PoolWideAccountingRow::PendingDeleteBytes => STATISTIC_PENDING_DELETE_BYTES,
crates/singlefs-core/src/transaction.rs:4048:             PoolWideAccountingRow::CommittedReservationBytes => {
crates/singlefs-core/src/transaction.rs:4049:                 STATISTIC_COMMITTED_RESERVATION_BYTES
crates/singlefs-core/src/transaction.rs:4050:             }
crates/singlefs-core/src/transaction.rs:4051:         }
crates/singlefs-core/src/transaction.rs:4052:     }
crates/singlefs-core/src/transaction.rs:4053: 
crates/singlefs-core/src/transaction.rs:4054:     /// 这一行记在哪棵树上：inode 号水位记在 inode 树上，另两行不属于任何一棵树。
crates/singlefs-core/src/transaction.rs:4055:     const fn tree(self, inode_tree: TreeIdentifier) -> TreeIdentifier {
crates/singlefs-core/src/transaction.rs:4056:         match self {
crates/singlefs-core/src/transaction.rs:4057:             PoolWideAccountingRow::InodeNumberWatermark => inode_tree,
crates/singlefs-core/src/transaction.rs:4058:             PoolWideAccountingRow::PendingDeleteBytes
crates/singlefs-core/src/transaction.rs:4059:             | PoolWideAccountingRow::CommittedReservationBytes => {
crates/singlefs-core/src/transaction.rs:4060:                 TreeIdentifier(TREE_IDENTIFIER_NONE)
crates/singlefs-core/src/transaction.rs:4061:             }
crates/singlefs-core/src/transaction.rs:4062:         }
crates/singlefs-core/src/transaction.rs:4063:     }
crates/singlefs-core/src/transaction.rs:4064: }
crates/singlefs-core/src/transaction.rs:4099: /// 一次发布写的记账行（池级三行加每块盘六行，代 = 这次的 txg、seq 一律 1，D8（核心索引结构） 已定项 10），按 key 升序。
crates/singlefs-core/src/transaction.rs:4100: /// 准入之前算记账树形状（只要 key，值给 0）与装记账树（值从分配器取）读的是这一份行表。
crates/singlefs-core/src/transaction.rs:4101: fn accounting_entries_of_this_publish(
crates/singlefs-core/src/transaction.rs:4102:     txg: CheckpointTxg,
crates/singlefs-core/src/transaction.rs:4103:     inode_tree: TreeIdentifier,
crates/singlefs-core/src/transaction.rs:4104:     device_identities: &[DeviceIdentity],
crates/singlefs-core/src/transaction.rs:4105:     pool_wide_value: &dyn Fn(PoolWideAccountingRow) -> u64,
crates/singlefs-core/src/transaction.rs:4106:     per_device_value: &dyn Fn(PerDeviceAccountingRow, DeviceIdentity) -> u64,
crates/singlefs-core/src/transaction.rs:4107: ) -> Vec<AccountingEntry> {
crates/singlefs-core/src/transaction.rs:4108:     let mut entries: Vec<AccountingEntry> = PoolWideAccountingRow::ALL
crates/singlefs-core/src/transaction.rs:4109:         .into_iter()
crates/singlefs-core/src/transaction.rs:4110:         .map(|row| AccountingEntry {
crates/singlefs-core/src/transaction.rs:4111:             statistic: row.statistic(),
crates/singlefs-core/src/transaction.rs:4112:             tree: row.tree(inode_tree),
crates/singlefs-core/src/transaction.rs:4113:             device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
crates/singlefs-core/src/transaction.rs:4114:             generation: txg,
crates/singlefs-core/src/transaction.rs:4115:             value: pool_wide_value(row),
crates/singlefs-core/src/transaction.rs:4116:             sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
crates/singlefs-core/src/transaction.rs:4117:         })
crates/singlefs-core/src/transaction.rs:4118:         .collect();
crates/singlefs-core/src/transaction.rs:4119:     for device in device_identities {
crates/singlefs-core/src/transaction.rs:4120:         for row in PerDeviceAccountingRow::ALL {
crates/singlefs-core/src/transaction.rs:4121:             entries.push(AccountingEntry {
crates/singlefs-core/src/transaction.rs:4122:                 statistic: row.statistic(),
crates/singlefs-core/src/transaction.rs:4123:                 tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
crates/singlefs-core/src/transaction.rs:4124:                 device: *device,
crates/singlefs-core/src/transaction.rs:4125:                 generation: txg,
crates/singlefs-core/src/transaction.rs:4126:                 value: per_device_value(row, *device),
crates/singlefs-core/src/transaction.rs:4127:                 sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
crates/singlefs-core/src/transaction.rs:4128:             });
crates/singlefs-core/src/transaction.rs:4129:         }
crates/singlefs-core/src/transaction.rs:4130:     }
crates/singlefs-core/src/transaction.rs:4131:     entries.sort_by_key(AccountingEntry::sort_key);
crates/singlefs-core/src/transaction.rs:4132:     entries
crates/singlefs-core/src/transaction.rs:4133: }

### c2 值从哪来（`publish_admitted` 里装记账行那一段，原文）
crates/singlefs-core/src/transaction.rs:6259:     // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
crates/singlefs-core/src/transaction.rs:6260:     // 全部来自分配器在分配那一刻增量维护的数，不扫盘（`.claude/rules/fs-design.md` 第一格）。seq 一律 1（D8（核心索引结构） 已定项 10）。
crates/singlefs-core/src/transaction.rs:6261:     // inode 号水位 = 下一个可用号（D5（快照 / 空间记账机制） 已定项 4 第 12 项）：每次发布重写这一行，
crates/singlefs-core/src/transaction.rs:6262:     // 这次建了几个 inode 就在上一版的水位上加几。树里最大的 key 恒小于它（I-9.6（水位大于两处最大号））——
crates/singlefs-core/src/transaction.rs:6263:     // 这里按「已经发出去的号」算，不按「树里现在有什么」算：号一旦发出去就不再复用（D8（核心索引结构） 已定项 6）。
crates/singlefs-core/src/transaction.rs:6264:     let new_inodes_of_this_publish =
crates/singlefs-core/src/transaction.rs:6265:         u64::try_from(plan.new_inode_records.len()).expect("这次新建的 inode 数");
crates/singlefs-core/src/transaction.rs:6266:     let inode_number_watermark = match previous {
crates/singlefs-core/src/transaction.rs:6267:         Some(previous_version) => previous_version.inode_number_watermark(),
crates/singlefs-core/src/transaction.rs:6268:         // 新池新建文件：树里只有 inode 1，下一个可用号是 2（字节表六那一行）。
crates/singlefs-core/src/transaction.rs:6269:         None => FIRST_INODE_NUMBER + 1,
crates/singlefs-core/src/transaction.rs:6270:     } + new_inodes_of_this_publish;
crates/singlefs-core/src/transaction.rs:6271:     let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
crates/singlefs-core/src/transaction.rs:6272:         .devices
crates/singlefs-core/src/transaction.rs:6273:         .iter()
crates/singlefs-core/src/transaction.rs:6274:         .map(|device_map| device_map.device)
crates/singlefs-core/src/transaction.rs:6275:         .collect();
crates/singlefs-core/src/transaction.rs:6276:     let accounting_entries = accounting_entries_of_this_publish(
crates/singlefs-core/src/transaction.rs:6277:         txg,
crates/singlefs-core/src/transaction.rs:6278:         trees.inode,
crates/singlefs-core/src/transaction.rs:6279:         &device_identities_of_the_accounting_rows,
crates/singlefs-core/src/transaction.rs:6280:         &|row| match row {
crates/singlefs-core/src/transaction.rs:6281:             PoolWideAccountingRow::InodeNumberWatermark => inode_number_watermark,
crates/singlefs-core/src/transaction.rs:6282:             // 这两行与准入读数读的是同一个常量（`admission`）：一处定义，两处各抄一个 0 会分叉。
crates/singlefs-core/src/transaction.rs:6283:             PoolWideAccountingRow::PendingDeleteBytes => {
crates/singlefs-core/src/transaction.rs:6284:                 crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0
crates/singlefs-core/src/transaction.rs:6285:             }
crates/singlefs-core/src/transaction.rs:6286:             PoolWideAccountingRow::CommittedReservationBytes => {
crates/singlefs-core/src/transaction.rs:6287:                 crate::admission::COMMITTED_RESERVATION_OF_THE_FIRST_VERSION.0
crates/singlefs-core/src/transaction.rs:6288:             }
crates/singlefs-core/src/transaction.rs:6289:         },
crates/singlefs-core/src/transaction.rs:6290:         &|row, device| {
crates/singlefs-core/src/transaction.rs:6291:             let device_map = allocator
crates/singlefs-core/src/transaction.rs:6292:                 .devices
crates/singlefs-core/src/transaction.rs:6293:                 .iter()
crates/singlefs-core/src/transaction.rs:6294:                 .find(|device_map| device_map.device == device)
crates/singlefs-core/src/transaction.rs:6295:                 .expect("行按分配器里的那几块盘展开");
crates/singlefs-core/src/transaction.rs:6296:             match row {
crates/singlefs-core/src/transaction.rs:6297:                 PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,
crates/singlefs-core/src/transaction.rs:6298:                 // 第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）。
crates/singlefs-core/src/transaction.rs:6299:                 PerDeviceAccountingRow::FreeBytes => device_map.free_slots() * SLOT_BYTES,
crates/singlefs-core/src/transaction.rs:6300:                 PerDeviceAccountingRow::UnreclaimableBytes => {
crates/singlefs-core/src/transaction.rs:6301:                     crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0
crates/singlefs-core/src/transaction.rs:6302:                 }
crates/singlefs-core/src/transaction.rs:6303:                 // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
crates/singlefs-core/src/transaction.rs:6304:                 PerDeviceAccountingRow::DeferQueueBytes => device_map.deferred_slots() * SLOT_BYTES,
crates/singlefs-core/src/transaction.rs:6305:                 PerDeviceAccountingRow::FragmentationRuns => device_map.free_runs(),
crates/singlefs-core/src/transaction.rs:6306:                 PerDeviceAccountingRow::EmptyClusterSegments => device_map.empty_segments(),
crates/singlefs-core/src/transaction.rs:6307:             }
crates/singlefs-core/src/transaction.rs:6308:         },
crates/singlefs-core/src/transaction.rs:6309:     );

### c3 `records.rs` 的统计量标签登记（原文）
crates/singlefs-core/src/records.rs:14: /// 统计量标签 = D5（快照 / 空间记账机制） 已定项 4 的行号（已定项 10 的登记表）。
crates/singlefs-core/src/records.rs:15: pub const STATISTIC_ALLOCATED_BYTES: u16 = 1;
crates/singlefs-core/src/records.rs:16: pub const STATISTIC_FREE_BYTES: u16 = 2;
crates/singlefs-core/src/records.rs:17: pub const STATISTIC_UNRECLAIMABLE_BYTES: u16 = 3;
crates/singlefs-core/src/records.rs:18: pub const STATISTIC_PENDING_DELETE_BYTES: u16 = 4;
crates/singlefs-core/src/records.rs:19: pub const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
crates/singlefs-core/src/records.rs:20: pub const STATISTIC_COMMITTED_RESERVATION_BYTES: u16 = 6;
crates/singlefs-core/src/records.rs:21: pub const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;
crates/singlefs-core/src/records.rs:22: pub const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u16 = 11;
crates/singlefs-core/src/records.rs:23: pub const STATISTIC_INODE_WATERMARK: u16 = 12;
crates/singlefs-core/src/records.rs:24: /// 不带设备维的统计量，设备段写保留值（D5（快照 / 空间记账机制） 已定项 10）。
crates/singlefs-core/src/records.rs:25: pub const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
crates/singlefs-core/src/records.rs:26: /// 第一版直落叶，seq 一律 1（D8（核心索引结构） 已定项 10）。
crates/singlefs-core/src/records.rs:27: pub const ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF: u32 = 1;
crates/singlefs-core/src/records.rs:131: /// 记账条目 34：key (标签 2, 树 ID 8, 设备 4, 代 8) + value 8 + seq 4。

### c4 记账行的其他事实

- 记账行的两种用途：装树时取值（`transaction.rs:6276-6309`）与准入前算记账树形状（只要 key、值给 0，`transaction.rs:4382-4391`，原文 `&|_| 0,` 与 `&|_, _| 0,` 在 4386-4387）。形状不依赖标签 4、6 的值。
- 只有 `publish_admitted` 一处用真值装行；没有文件的零单元发布（`publish_without_units`）不写记账行（`mount.rs:3889-3890` 的注释：「这一版没有记账树，已定项 9 的那五样一样都不写」）。
- 盘上的标签 4、6 两行在挂载时被读回内存，但没有消费者：`recovery.rs:2584-2590` 把整棵记账树的叶条目逐条解成 `Vec<AccountingEntry>`，`recovery.rs:2593-2598` 只判「inode 号水位那一行在」，条目数只比 `3 + 6 × 盘数`（`recovery.rs:3502`）。
  回退发布拷贝上一版记账行时只改水位那一行，其余条目值原样（`mount.rs:4981-4996`，原文 `value: if entry.statistic == STATISTIC_INODE_WATERMARK {`在 4989）。
- 整个 `crates/` 里 `STATISTIC_PENDING_DELETE_BYTES`、`STATISTIC_COMMITTED_RESERVATION_BYTES` 的引用：`records.rs:18`、`records.rs:20`；`transaction.rs:81`、`83`（use）、`4033`、`4034`（枚举成员）、`4040`、`4041`（`ALL`）、`4047`、`4048`、`4049`（`statistic()`）、`4058`、`4059`（`tree()`）、`6283`、`6286`（取值闭包）；
  `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:35`、`37`、`1153`、`1154`、`1174`、`1175`（用例断言「准入三项 day-1 写 0 行」，`new_pool_file_creation_publish.rs:1156`）。
  命令：`grep -rn 'STATISTIC_PENDING_DELETE_BYTES\|STATISTIC_COMMITTED_RESERVATION_BYTES\|PendingDeleteBytes\|CommittedReservationBytes' --include=*.rs .`（在 `crates/` 下），输出 21 行（records.rs 2、transaction.rs 13、harness 用例 6），没有任何一行在 `singlefs-checker`、`singlefs-checker-tier`。

## (d) 一次准入最多 8 次发布

### d1 `PUBLISHES_PER_ADMISSION_AT_MOST` 的定义与注释原文
crates/singlefs-core/src/mount.rs:2120: /// 一次准入最多推几次发布：B = 4 + 2 k_tol，k_tol = 2 ⇒ 8（D16（发布语义） 已定项 1「准入」那一行）。数的是这次准入推的空发布
crates/singlefs-core/src/mount.rs:2121: /// 加上这次准入为之判的那一次发布（发布路径是那次用户发布，可写挂载那一处是写行那次）：`df` 的保留池按 B − 1 次空发布留
crates/singlefs-core/src/mount.rs:2122: /// （同一张表「`df`」那一行：保留池 = 2 × 5 + (B − 1) × c_max）。
crates/singlefs-core/src/mount.rs:2123: pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;

### d2 被谁用（命令 `cd crates; grep -rn 'PUBLISHES_PER_ADMISSION_AT_MOST' .`，共 13 行：产品代码 1 处读值，其余是定义、文档引用、E158 装置的对账、一个用例、一条变异）

- 另有（上面清单之外的）：`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:5039`（本地常量 `PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL: u64 = 4 + 2 * 2;`）、`:5089`、`:5090`、`:5091`（拿 `singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST` 与本地值对账）；`crates/mutations.tsv:759`（一条变异：8 改成 10）。

- 读值（产品路径唯一一处）：`crates/singlefs-core/src/mount.rs:2312`：`    if publishes_of_this_admission > PUBLISHES_PER_ADMISSION_AT_MOST {`
- 文档引用：`mount.rs:2207`、`mount.rs:2277`、`mount.rs:4068`、`crates/singlefs-core/src/mounted_session.rs:5`。
- 用例：`crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write.rs:26`（use）、`:399`（`PUBLISHES_PER_ADMISSION_AT_MOST - 1,`）。
- 另外 `mount.rs:2123` 是定义本身，`mounted_session.rs:234` 的文档说：
crates/singlefs-core/src/mounted_session.rs:233:     /// 循环：每一轮发一次；被拒是空间不够就推一串（至少一次空发布、推的次数增加）进下一轮，不推了就交回。
crates/singlefs-core/src/mounted_session.rs:234:     /// 预算把推的次数压在 B − 1 以内 ⇒ 至多 8 轮；跨轮携带的是现行那一版（抬 F 就地推进它）与已推的几串。
crates/singlefs-core/src/mounted_session.rs:235:     ///

### d3 `push_one_floor_raise_within_the_admission_budget` 的流程（原文）
crates/singlefs-core/src/mount.rs:2275: /// 一次准入里再推一串抬 F 的空发布：先核调用方交进来的参数与盘表（照可写挂载同一套，实审 A1b Q2、Q3；之后只用盘上那一份参数），
crates/singlefs-core/src/mount.rs:2276: /// 再按预算判（这次准入已经推的 `publishes_pushed` 次，加这一串要推的次数，
crates/singlefs-core/src/mount.rs:2277: /// 加这次准入为之判的那一次发布，不超过 [`PUBLISHES_PER_ADMISSION_AT_MOST`]），再抬到上限。这一串要推几次与抬 F 那一串按同一个
crates/singlefs-core/src/mount.rs:2278: /// 纯算的函数数（`txgs_of_the_publishes_carrying_the_floor_to_every_device`）。推满仍不够（预算用完、F 已在上限、抬 F 那一串自己也被
crates/singlefs-core/src/mount.rs:2279: /// 空间不够拒）交回 [`FloorRaisePushedWithinTheAdmissionBudget::Stopped`]。
crates/singlefs-core/src/mount.rs:2280: ///
crates/singlefs-core/src/mount.rs:2281: /// # Errors
crates/singlefs-core/src/mount.rs:2282: /// 盘表里有身份交了不止一次、调用方的参数或盘表与盘上系统配置不一致（在判预算与任何写之前）；
crates/singlefs-core/src/mount.rs:2283: /// 抬 F 自己报的别的错（[`raise_rollback_floor_to_the_admission_ceiling`] 那一半的错，原样；代码审阅第 23 条：块设备错不是空间不够）：
crates/singlefs-core/src/mount.rs:2284: /// 在任何写之前被拒的，盘上与分配器不动；真发时落盘途中失败的（`MountError::RaiseFloorSequencePublishFailed`），
crates/singlefs-core/src/mount.rs:2285: /// 前面已落盘的那几次在调用方的现行版本里。
crates/singlefs-core/src/mount.rs:2286: pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
crates/singlefs-core/src/mount.rs:2287:     parameters: &MakeFilesystemParameters,
crates/singlefs-core/src/mount.rs:2288:     devices: &mut Vec<(DeviceIdentity, Device)>,
crates/singlefs-core/src/mount.rs:2289:     allocator: &mut PoolAllocator,
crates/singlefs-core/src/mount.rs:2290:     current: &mut TransactionOutput,
crates/singlefs-core/src/mount.rs:2291:     shadow_ledger: ShadowLedger,
crates/singlefs-core/src/mount.rs:2292:     publishes_pushed: usize,
crates/singlefs-core/src/mount.rs:2293: ) -> Result<FloorRaisePushedWithinTheAdmissionBudget, MountError> {
crates/singlefs-core/src/mount.rs:2294:     let caller_inputs_agreeing_with_the_disk =
crates/singlefs-core/src/mount.rs:2295:         caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
crates/singlefs-core/src/mount.rs:2296:     let parameters_of_the_pool = &caller_inputs_agreeing_with_the_disk.parameters_of_the_pool;
crates/singlefs-core/src/mount.rs:2297:     let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
crates/singlefs-core/src/mount.rs:2298:     let device_of_txg = |txg: CheckpointTxg| {
crates/singlefs-core/src/mount.rs:2299:         let target = target_for_publish(
crates/singlefs-core/src/mount.rs:2300:             txg,
crates/singlefs-core/src/mount.rs:2301:             parameters_of_the_pool.geometry.root_ring_slots_per_region,
crates/singlefs-core/src/mount.rs:2302:         );
crates/singlefs-core/src/mount.rs:2303:         parameters_of_the_pool.region_devices[usize::try_from(target.region).expect("区域号")]
crates/singlefs-core/src/mount.rs:2304:     };
crates/singlefs-core/src/mount.rs:2305:     let publishes_of_the_next_raise = txgs_of_the_publishes_carrying_the_floor_to_every_device(
crates/singlefs-core/src/mount.rs:2306:         current.root.checkpoint_txg,
crates/singlefs-core/src/mount.rs:2307:         &all_devices,
crates/singlefs-core/src/mount.rs:2308:         &device_of_txg,
crates/singlefs-core/src/mount.rs:2309:     )?
crates/singlefs-core/src/mount.rs:2310:     .len();
crates/singlefs-core/src/mount.rs:2311:     let publishes_of_this_admission = publishes_pushed + publishes_of_the_next_raise + 1;
crates/singlefs-core/src/mount.rs:2312:     if publishes_of_this_admission > PUBLISHES_PER_ADMISSION_AT_MOST {
crates/singlefs-core/src/mount.rs:2313:         return Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
crates/singlefs-core/src/mount.rs:2314:             FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded {
crates/singlefs-core/src/mount.rs:2315:                 publishes_pushed,
crates/singlefs-core/src/mount.rs:2316:                 publishes_of_the_next_raise,
crates/singlefs-core/src/mount.rs:2317:             },
crates/singlefs-core/src/mount.rs:2318:         ));
crates/singlefs-core/src/mount.rs:2319:     }
crates/singlefs-core/src/mount.rs:2320:     match raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk(
crates/singlefs-core/src/mount.rs:2321:         &caller_inputs_agreeing_with_the_disk,
crates/singlefs-core/src/mount.rs:2322:         devices,
crates/singlefs-core/src/mount.rs:2323:         allocator,
crates/singlefs-core/src/mount.rs:2324:         current,
crates/singlefs-core/src/mount.rs:2325:         shadow_ledger,
crates/singlefs-core/src/mount.rs:2326:     ) {
crates/singlefs-core/src/mount.rs:2327:         Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
crates/singlefs-core/src/mount.rs:2328:             Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised))
crates/singlefs-core/src/mount.rs:2329:         }
crates/singlefs-core/src/mount.rs:2330:         Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
crates/singlefs-core/src/mount.rs:2331:             Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
crates/singlefs-core/src/mount.rs:2332:                 FloorRaiseStop::FloorAlreadyAtTheCeiling { floor, ceiling },
crates/singlefs-core/src/mount.rs:2333:             ))
crates/singlefs-core/src/mount.rs:2334:         }
crates/singlefs-core/src/mount.rs:2335:         Err(refusal) if floor_raise_refused_for_space(&refusal) => {
crates/singlefs-core/src/mount.rs:2336:             Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
crates/singlefs-core/src/mount.rs:2337:                 FloorRaiseStop::FloorRaiseRefusedForSpace {
crates/singlefs-core/src/mount.rs:2338:                     refusal: Box::new(refusal),
crates/singlefs-core/src/mount.rs:2339:                 },
crates/singlefs-core/src/mount.rs:2340:             ))
crates/singlefs-core/src/mount.rs:2341:         }
crates/singlefs-core/src/mount.rs:2342:         Err(refusal) => Err(refusal),
crates/singlefs-core/src/mount.rs:2343:     }
crates/singlefs-core/src/mount.rs:2344: }

## (e) 可写挂载的写序列与会话层

### e1 入口链
`mount_writable`（`mount.rs:4165`）→ `mount_writable_with_space_admission`（`mount.rs:4177`）→ `mount_writable_with_test_only_switches`（`mount.rs:4191`）→ `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`（`mount.rs:4430`）→ `establish_instance`（`mount.rs:3688`，调用点 `mount.rs:4521`）。

### e2 取号之前（`mount.rs:4439-4545`，原文挑关键行）
crates/singlefs-core/src/mount.rs:4439:     admit_the_writable_device_count(distinct_device_identities_handed_in(devices))?;
crates/singlefs-core/src/mount.rs:4440:     refuse_device_identities_handed_in_more_than_once(devices)?;
crates/singlefs-core/src/mount.rs:4441:     let system_configuration = choose_system_configuration(&*devices)?;
crates/singlefs-core/src/mount.rs:4455:     // 读阶段：择根、扫 journal、重放；系统配置见证过比所选那一版新的发布就重读一遍，仍判真就在这里拒（C554 乙）。
crates/singlefs-core/src/mount.rs:4456:     let (
crates/singlefs-core/src/mount.rs:4487:     let previous = rebuild_previous_version(devices, unit_area_start, &effective_root, own_record)?;
crates/singlefs-core/src/mount.rs:4502:     let rebuilt = rebuilt_allocator(
crates/singlefs-core/src/mount.rs:4503:         devices,
crates/singlefs-core/src/mount.rs:4504:         &system_configuration,
crates/singlefs-core/src/mount.rs:4505:         &previous,
crates/singlefs-core/src/mount.rs:4506:         first_txg,
crates/singlefs-core/src/mount.rs:4507:         shadow_ledger,
crates/singlefs-core/src/mount.rs:4508:         &mut before_the_one_reread,
crates/singlefs-core/src/mount.rs:4509:     );
crates/singlefs-core/src/mount.rs:4510:     let RebuiltAllocator {
crates/singlefs-core/src/mount.rs:4511:         allocator,
crates/singlefs-core/src/mount.rs:4512:         effective_floor,
crates/singlefs-core/src/mount.rs:4513:         abandoned_roots_unreadable,
crates/singlefs-core/src/mount.rs:4514:         instance_table_of_the_newest_root,
crates/singlefs-core/src/mount.rs:4515:     } = rebuilt?;
crates/singlefs-core/src/mount.rs:4521:     establish_instance(
crates/singlefs-core/src/mount.rs:4522:         &parameters_of_the_pool,

（读阶段 `read_stage_settled_at_most_on_the_one_reread` 在 4465；重建分配器 `rebuilt_allocator` 在 4502，内含按回收门槛回收与影子账隔离；这两步在准入之前、写行之前，admission.rs 模块文档 25-26、315-316 说「准入读哪一个，条款没写」。）

### e3 `establish_instance` 的逐步行号与它们对空间准入的处理

| 步 | 行号 | 做什么 | 对空间准入的处理 |
|---|---|---|---|
| 1 | mount.rs:3694-3703 | 取每盘隔离槽数、取所选版单元清单 | 无 |
| 2 | 3704-3709 | 开写入口 `PoolWriter::new`；`warm_up_publish_txgs` 先算暖机推几次 | 无；暖机次数在这里定死，取号之后循环按它推（3931-3932 的注释） |
| 3 | 3710-3717 | `next_instance_generation_to_acquire` 取号前算要取的号 | 无 |
| 4 | 3718-3723 | 列要写的行、算实例表重写 | 无 |
| 5 | 3732-3735 | `record_instance_rows_after_this_mounts_row_publish`（记 rows0）、`set_space_admission` | 把 rows0 与开关装到分配器上（内存） |
| 6 | 3736-3763 | 取号前的准入：`admit_on_every_device(admission_reading_of_a_writable_mount_with_node_capacities(…), &no_demand_on_any_device_of(&allocator))` | 需求逐盘 0，只判「切换预留拿得到」；不够时上一版带文件 ⇒ 记成 `ShortUntilTheFloorIsRaisedAfterTheRowPublish`（写完行再推），上一版树表 0 条 ⇒ 当场 `SpaceAdmissionRefusedBeforeAcquisition` |
| 7 | 3764-3811 | 取号前预演 `dry_run_of_the_publishes_after_acquisition`（2874-2995），在分配器拷贝上走 `prepare_the_version_publish`（写行一次、暖机几次） | 预演走的是 `prepare_the_version_publish`，里面的准入照常判（5089、5135）；写行与暖机没有普通分配，不判；落点取不到报 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`，别的错按序号报 `RowPublishAdmissionRefusedBeforeAcquisition` / `WarmUpAdmissionRefusedBeforeAcquisition` |
| 8 | 3815-3837 | `devices_without_the_selected_version` 逐盘核 | 无 |
| 9 | 3838-3867 | `acquire_expected_instance` 取号（第一个写），断言取到的号 = 预演用的号 | 无 |
| 10 | 3870-3929 | 写行发布：带文件 `publish_rows_on_file_version`（2758，走 `publish_version`）；树表 0 条且无行 `publish_without_units`；树表 0 条有行 `publish_instance_table_on_version_without_file`；失败 `map_err(publish_failed_after_acquisition(&pool, &[], cause))`（3929） | 带文件那一路经过 `prepare_the_version_publish`，写行没有普通分配所以不判 |
| 11 | 3931-3956 | 暖机循环 `publish_empty_after`（2818），失败 `publish_failed_after_acquisition(&pool, &persisted, cause)`（3951） | 同上，空发布不判 |
| 12 | 3957-3978 | 断言「取号前预演取到的落点 = 真发取到的落点」（有一份被隔离时不比） | 无 |
| 13 | 3980-4016 | 按步 6 的结局：`Admitted` / `NotJudgedByTheTestOnlySwitch` / `ShortUntilTheFloorIsRaisedAfterTheRowPublish` ⇒ `push_floor_raises_after_the_row_publish`（4087-4147） | 取号前不够的才在这里推抬 F、再判 |
| 14 | 4018-4042 | 交回 `Ok(Mounted { output, allocator, current })` | `MountOutput.space_admission` 记结局 |

步 6 与步 13 的原文：
crates/singlefs-core/src/mount.rs:3736:     let admission_before_acquisition = match start.space_admission {
crates/singlefs-core/src/mount.rs:3737:         SpaceAdmission::JudgedByTheFormula => match admit_on_every_device(
crates/singlefs-core/src/mount.rs:3738:             &admission_reading_of_a_writable_mount_with_node_capacities(
crates/singlefs-core/src/mount.rs:3739:                 &allocator,
crates/singlefs-core/src/mount.rs:3740:                 start.previous.file_version(),
crates/singlefs-core/src/mount.rs:3741:                 pool.code_two_tree_node_capacities(),
crates/singlefs-core/src/mount.rs:3742:             ),
crates/singlefs-core/src/mount.rs:3743:             &no_demand_on_any_device_of(&allocator),
crates/singlefs-core/src/mount.rs:3744:         ) {
crates/singlefs-core/src/mount.rs:3745:             Ok(()) => WritableMountAdmissionBeforeAcquisition::Admitted,
crates/singlefs-core/src/mount.rs:3746:             Err(refusal) => match &start.previous {
crates/singlefs-core/src/mount.rs:3747:                 PreviousVersion::WithFile { .. } => {
crates/singlefs-core/src/mount.rs:3748:                     WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
crates/singlefs-core/src/mount.rs:3749:                         refusal,
crates/singlefs-core/src/mount.rs:3750:                     )
crates/singlefs-core/src/mount.rs:3751:                 }
crates/singlefs-core/src/mount.rs:3752:                 PreviousVersion::WithoutFile { .. } => {
crates/singlefs-core/src/mount.rs:3753:                     return Err(MountError::SpaceAdmissionRefusedBeforeAcquisition {
crates/singlefs-core/src/mount.rs:3754:                         instance_to_acquire,
crates/singlefs-core/src/mount.rs:3755:                         refusal,
crates/singlefs-core/src/mount.rs:3756:                     });
crates/singlefs-core/src/mount.rs:3757:                 }
crates/singlefs-core/src/mount.rs:3758:             },
crates/singlefs-core/src/mount.rs:3759:         },
crates/singlefs-core/src/mount.rs:3760:         SpaceAdmission::SkippedByTheTestOnlySwitch => {
crates/singlefs-core/src/mount.rs:3761:             WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch
crates/singlefs-core/src/mount.rs:3762:         }
crates/singlefs-core/src/mount.rs:3763:     };
crates/singlefs-core/src/mount.rs:3980:     let space_admission = match admission_before_acquisition {
crates/singlefs-core/src/mount.rs:3981:         WritableMountAdmissionBeforeAcquisition::Admitted => {
crates/singlefs-core/src/mount.rs:3982:             MountSpaceAdmission::AdmittedBeforeAcquisition
crates/singlefs-core/src/mount.rs:3983:         }
crates/singlefs-core/src/mount.rs:3984:         WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch => {
crates/singlefs-core/src/mount.rs:3985:             MountSpaceAdmission::NotJudgedByTheTestOnlySwitch
crates/singlefs-core/src/mount.rs:3986:         }
crates/singlefs-core/src/mount.rs:3987:         WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
crates/singlefs-core/src/mount.rs:3988:             refusal_before_acquisition,
crates/singlefs-core/src/mount.rs:3989:         ) => {
crates/singlefs-core/src/mount.rs:3990:             let current_file_version = match &mut current {
crates/singlefs-core/src/mount.rs:3991:                 PoolVersion::WithFile(current_file_version) => current_file_version,
crates/singlefs-core/src/mount.rs:3992:                 PoolVersion::WithoutFile(_) => panic!(
crates/singlefs-core/src/mount.rs:3993:                     "取号之前不够、要写行之后推的只有上一版带文件的那一格（树表 0 条的在取号之前拒了）：\
crates/singlefs-core/src/mount.rs:3994:                      带文件的一版上写行与暖机交回的都是带文件的一版（`publish_rows_on_file_version`、`publish_empty_after`）"
crates/singlefs-core/src/mount.rs:3995:                 ),
crates/singlefs-core/src/mount.rs:3996:             };
crates/singlefs-core/src/mount.rs:3997:             // 这次挂载已落盘的发布（写行那次、暖机每次）各自的写：抬 F 报错时随错交出（实审 A1b Q5）。
crates/singlefs-core/src/mount.rs:3998:             let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
crates/singlefs-core/src/mount.rs:3999:                 std::iter::once(&row_publish)
crates/singlefs-core/src/mount.rs:4000:                     .chain(&warm_up_publishes)
crates/singlefs-core/src/mount.rs:4001:                     .map(|version| version.writes().clone())
crates/singlefs-core/src/mount.rs:4002:                     .collect();
crates/singlefs-core/src/mount.rs:4003:             // 写行与暖机那几次的写入口装的节点容量：之后推抬 F、再判的读数按同一档算。
crates/singlefs-core/src/mount.rs:4004:             let node_capacities_of_the_writer = pool.code_two_tree_node_capacities();
crates/singlefs-core/src/mount.rs:4005:             push_floor_raises_after_the_row_publish(
crates/singlefs-core/src/mount.rs:4006:                 parameters,
crates/singlefs-core/src/mount.rs:4007:                 devices,
crates/singlefs-core/src/mount.rs:4008:                 &mut allocator,
crates/singlefs-core/src/mount.rs:4009:                 current_file_version,
crates/singlefs-core/src/mount.rs:4010:                 start.shadow_ledger,
crates/singlefs-core/src/mount.rs:4011:                 node_capacities_of_the_writer,
crates/singlefs-core/src/mount.rs:4012:                 refusal_before_acquisition,
crates/singlefs-core/src/mount.rs:4013:                 writes_of_persisted_publishes,
crates/singlefs-core/src/mount.rs:4014:             )?
crates/singlefs-core/src/mount.rs:4015:         }
crates/singlefs-core/src/mount.rs:4016:     };

### e4 `publish_failed_after_acquisition` 是什么出口（原文）
crates/singlefs-core/src/mount.rs:3127: /// 取号之后的一次发布失败了：错，连同这次挂载的写入口交得出的账——`persisted` 是这次挂载里已经落盘的那几次（按先后）。
crates/singlefs-core/src/mount.rs:3128: fn publish_failed_after_acquisition<Device: BlockDevice>(
crates/singlefs-core/src/mount.rs:3129:     pool: &PoolWriter<'_, Device>,
crates/singlefs-core/src/mount.rs:3130:     persisted: &[&PoolVersion],
crates/singlefs-core/src/mount.rs:3131:     cause: PublishError,
crates/singlefs-core/src/mount.rs:3132: ) -> MountError {
crates/singlefs-core/src/mount.rs:3133:     MountError::Publish(publish_sequence_failed(
crates/singlefs-core/src/mount.rs:3134:         pool,
crates/singlefs-core/src/mount.rs:3135:         WritesByStructureKind::NOTHING_WRITTEN,
crates/singlefs-core/src/mount.rs:3136:         persisted
crates/singlefs-core/src/mount.rs:3137:             .iter()
crates/singlefs-core/src/mount.rs:3138:             .map(|version| match version {
crates/singlefs-core/src/mount.rs:3139:                 PoolVersion::WithoutFile(version_without_file) => {
crates/singlefs-core/src/mount.rs:3140:                     version_without_file.writes.clone()
crates/singlefs-core/src/mount.rs:3141:                 }
crates/singlefs-core/src/mount.rs:3142:                 PoolVersion::WithFile(file_version) => file_version.writes.clone(),
crates/singlefs-core/src/mount.rs:3143:             })
crates/singlefs-core/src/mount.rs:3144:             .collect(),
crates/singlefs-core/src/mount.rs:3145:         cause,
crates/singlefs-core/src/mount.rs:3146:     ))
crates/singlefs-core/src/mount.rs:3147: }
crates/singlefs-core/src/mount.rs:3110: /// 一串发布里有一次失败了：错，连同这一串的写入口交得出的账——`writes_before_the_first_publish` 是第一次发布之前、不属于任何一次发布的写
crates/singlefs-core/src/mount.rs:3111: /// （抬 F 那一串先写系统配置那一步；可写挂载那一串给空账），`writes_of_persisted_publishes` 是这一串里已经落盘的那几次各自的写
crates/singlefs-core/src/mount.rs:3112: /// （按先后），失败那一次落盘阶段已记的写从写入口的失败账取。可写挂载与抬 F 共用这一处。

事实：取号之后（写行、暖机）任一次发布失败，交回 `MountError::Publish(Box<PublishSequenceFailed>)`，带 `cause: PublishError`、已落盘的各次发布的写账与失败那次的写账；挂载不做成，没有 `Mounted` 交回。
调用点两处：`mount.rs:3929`（写行那次，`persisted` 为空）、`mount.rs:3951`（暖机某次，`persisted` 是写行加之前已落盘的暖机）。抬 F 那一路用的是同一个 `publish_sequence_failed`（3113）。

### e5 取号前不够时写行之后推抬 F 的循环（`push_floor_raises_after_the_row_publish`，原文）
crates/singlefs-core/src/mount.rs:4097:     let mut floor_raises: Vec<RaisedFloor> = Vec::new();
crates/singlefs-core/src/mount.rs:4098:     let mut publishes_pushed = 0;
crates/singlefs-core/src/mount.rs:4099:     loop {
crates/singlefs-core/src/mount.rs:4100:         let refusal = match admit_on_every_device(
crates/singlefs-core/src/mount.rs:4101:             &admission_reading_of_a_writable_mount_with_node_capacities(
crates/singlefs-core/src/mount.rs:4102:                 allocator,
crates/singlefs-core/src/mount.rs:4103:                 Some(&*current),
crates/singlefs-core/src/mount.rs:4104:                 node_capacities,
crates/singlefs-core/src/mount.rs:4105:             ),
crates/singlefs-core/src/mount.rs:4106:             &no_demand_on_any_device_of(allocator),
crates/singlefs-core/src/mount.rs:4107:         ) {
crates/singlefs-core/src/mount.rs:4108:             Ok(()) => {
crates/singlefs-core/src/mount.rs:4109:                 return Ok(MountSpaceAdmission::AdmittedAfterTheFloorRaises {
crates/singlefs-core/src/mount.rs:4110:                     refusal_before_acquisition,
crates/singlefs-core/src/mount.rs:4111:                     floor_raises,
crates/singlefs-core/src/mount.rs:4112:                 })
crates/singlefs-core/src/mount.rs:4113:             }
crates/singlefs-core/src/mount.rs:4114:             Err(refusal) => refusal,
crates/singlefs-core/src/mount.rs:4115:         };
crates/singlefs-core/src/mount.rs:4116:         match push_one_floor_raise_within_the_admission_budget(
crates/singlefs-core/src/mount.rs:4117:             parameters,
crates/singlefs-core/src/mount.rs:4118:             devices,
crates/singlefs-core/src/mount.rs:4119:             allocator,
crates/singlefs-core/src/mount.rs:4120:             current,
crates/singlefs-core/src/mount.rs:4121:             shadow_ledger,
crates/singlefs-core/src/mount.rs:4122:             publishes_pushed,
crates/singlefs-core/src/mount.rs:4123:         ) {
crates/singlefs-core/src/mount.rs:4124:             Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
crates/singlefs-core/src/mount.rs:4125:                 publishes_pushed += raised.publishes.len();
crates/singlefs-core/src/mount.rs:4126:                 floor_raises.push(raised);
crates/singlefs-core/src/mount.rs:4127:             }
crates/singlefs-core/src/mount.rs:4128:             Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
crates/singlefs-core/src/mount.rs:4129:                 return Ok(MountSpaceAdmission::StillShortAfterTheFloorRaises {
crates/singlefs-core/src/mount.rs:4130:                     refusal_before_acquisition,
crates/singlefs-core/src/mount.rs:4131:                     floor_raises,
crates/singlefs-core/src/mount.rs:4132:                     last_refusal: refusal,
crates/singlefs-core/src/mount.rs:4133:                     stop,
crates/singlefs-core/src/mount.rs:4134:                 })
crates/singlefs-core/src/mount.rs:4135:             }
crates/singlefs-core/src/mount.rs:4136:             Err(cause) => {
crates/singlefs-core/src/mount.rs:4137:                 return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(
crates/singlefs-core/src/mount.rs:4138:                     Box::new(FloorRaiseFailedAfterTheMountsPublishes {
crates/singlefs-core/src/mount.rs:4139:                         cause,
crates/singlefs-core/src/mount.rs:4140:                         writes_of_persisted_publishes,
crates/singlefs-core/src/mount.rs:4141:                         floor_raises,
crates/singlefs-core/src/mount.rs:4142:                     }),
crates/singlefs-core/src/mount.rs:4143:                 ))
crates/singlefs-core/src/mount.rs:4144:             }
crates/singlefs-core/src/mount.rs:4145:         }
crates/singlefs-core/src/mount.rs:4146:     }

### e6 会话层 `mounted_session.rs`：遇到 `SpaceAdmissionRefused` / `PlacementRefused` 时推抬 F 再判

「是不是空间不够」的判定（原文）：
crates/singlefs-core/src/mounted_session.rs:144: /// 这一次发布被拒是不是空间不够的那两种（D16（发布语义） 已定项 1「准入」那一行：准入不够，或准入放行而落点取不到）——
crates/singlefs-core/src/mounted_session.rs:145: /// 是就推一串抬 F 再重判，不是就原样交回。
crates/singlefs-core/src/mounted_session.rs:146: #[must_use]
crates/singlefs-core/src/mounted_session.rs:147: pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
crates/singlefs-core/src/mounted_session.rs:148:     match refusal {
crates/singlefs-core/src/mounted_session.rs:149:         PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,
crates/singlefs-core/src/mounted_session.rs:150:         PublishError::MultiLevelCodeTwoTreeRefused { .. }
crates/singlefs-core/src/mounted_session.rs:166:         | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
crates/singlefs-core/src/mounted_session.rs:167:         // 一次发布切出来的记录多于上限：推抬 F 腾不出记录的位置，要把改动切小（代码审阅第 26 条）；txg 到顶是坏镜像（第 36 条）。
crates/singlefs-core/src/mounted_session.rs:168:         | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
crates/singlefs-core/src/mounted_session.rs:169:         | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
crates/singlefs-core/src/mounted_session.rs:170:         | PublishError::BlockDevice(_) => false,
crates/singlefs-core/src/mounted_session.rs:171:     }

循环（`publish_user_change`，原文）：
crates/singlefs-core/src/mounted_session.rs:296:         let mut floor_raises: Vec<RaisedFloor> = Vec::new();
crates/singlefs-core/src/mounted_session.rs:297:         let mut refusals_that_pushed_the_floor_raises: Vec<PublishError> = Vec::new();
crates/singlefs-core/src/mounted_session.rs:298:         let mut publishes_pushed = 0;
crates/singlefs-core/src/mounted_session.rs:299:         loop {
crates/singlefs-core/src/mounted_session.rs:300:             let refusal = match publish_the_change_once(
crates/singlefs-core/src/mounted_session.rs:301:                 parameters,
crates/singlefs-core/src/mounted_session.rs:302:                 devices.as_mut_slice(),
crates/singlefs-core/src/mounted_session.rs:303:                 allocator,
crates/singlefs-core/src/mounted_session.rs:304:                 current_file_version,
crates/singlefs-core/src/mounted_session.rs:305:                 change,
crates/singlefs-core/src/mounted_session.rs:306:                 *instance,
crates/singlefs-core/src/mounted_session.rs:307:             ) {
crates/singlefs-core/src/mounted_session.rs:308:                 Ok(published) => {
crates/singlefs-core/src/mounted_session.rs:309:                     *current_file_version = published;
crates/singlefs-core/src/mounted_session.rs:310:                     return Ok(UserChangePublished {
crates/singlefs-core/src/mounted_session.rs:311:                         floor_raises,
crates/singlefs-core/src/mounted_session.rs:312:                         refusals_that_pushed_the_floor_raises,
crates/singlefs-core/src/mounted_session.rs:313:                     });
crates/singlefs-core/src/mounted_session.rs:314:                 }
crates/singlefs-core/src/mounted_session.rs:315:                 Err(refusal) => refusal,
crates/singlefs-core/src/mounted_session.rs:316:             };
crates/singlefs-core/src/mounted_session.rs:317:             if !refusal_is_short_of_space(&refusal) {
crates/singlefs-core/src/mounted_session.rs:318:                 return Err(UserChangeRefused::Publish {
crates/singlefs-core/src/mounted_session.rs:319:                     cause: refusal,
crates/singlefs-core/src/mounted_session.rs:320:                     floor_raises,
crates/singlefs-core/src/mounted_session.rs:321:                 });
crates/singlefs-core/src/mounted_session.rs:322:             }
crates/singlefs-core/src/mounted_session.rs:323:             match push_one_floor_raise_within_the_admission_budget(
crates/singlefs-core/src/mounted_session.rs:324:                 parameters,
crates/singlefs-core/src/mounted_session.rs:325:                 devices,
crates/singlefs-core/src/mounted_session.rs:326:                 allocator,
crates/singlefs-core/src/mounted_session.rs:327:                 current_file_version,
crates/singlefs-core/src/mounted_session.rs:328:                 *shadow_ledger,
crates/singlefs-core/src/mounted_session.rs:329:                 publishes_pushed,
crates/singlefs-core/src/mounted_session.rs:330:             ) {
crates/singlefs-core/src/mounted_session.rs:331:                 Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
crates/singlefs-core/src/mounted_session.rs:332:                     publishes_pushed += raised.publishes.len();
crates/singlefs-core/src/mounted_session.rs:333:                     floor_raises.push(raised);
crates/singlefs-core/src/mounted_session.rs:334:                     refusals_that_pushed_the_floor_raises.push(refusal);
crates/singlefs-core/src/mounted_session.rs:335:                 }
crates/singlefs-core/src/mounted_session.rs:336:                 Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
crates/singlefs-core/src/mounted_session.rs:337:                     return Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(Box::new(
crates/singlefs-core/src/mounted_session.rs:338:                         NoSpaceAfterRaisingTheFloor {
crates/singlefs-core/src/mounted_session.rs:339:                             last_refusal: refusal,
crates/singlefs-core/src/mounted_session.rs:340:                             floor_raises,
crates/singlefs-core/src/mounted_session.rs:341:                             refusals_that_pushed_the_floor_raises,
crates/singlefs-core/src/mounted_session.rs:342:                             stop,
crates/singlefs-core/src/mounted_session.rs:343:                         },
crates/singlefs-core/src/mounted_session.rs:344:                     )))
crates/singlefs-core/src/mounted_session.rs:345:                 }
crates/singlefs-core/src/mounted_session.rs:346:                 Err(cause) => {
crates/singlefs-core/src/mounted_session.rs:347:                     return Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(
crates/singlefs-core/src/mounted_session.rs:348:                         Box::new(FloorRaiseFailedWhilePushingForSpace {
crates/singlefs-core/src/mounted_session.rs:349:                             cause,
crates/singlefs-core/src/mounted_session.rs:350:                             refusal_that_pushed_the_failed_raise: refusal,
crates/singlefs-core/src/mounted_session.rs:351:                             floor_raises,
crates/singlefs-core/src/mounted_session.rs:352:                             refusals_that_pushed_the_floor_raises,
crates/singlefs-core/src/mounted_session.rs:353:                         }),
crates/singlefs-core/src/mounted_session.rs:354:                     ))
crates/singlefs-core/src/mounted_session.rs:355:                 }
crates/singlefs-core/src/mounted_session.rs:356:             }
crates/singlefs-core/src/mounted_session.rs:357:         }

### e7 会话层的其他事实
`UserChange` 今天只有三个成员，没有删除类（原文）：
crates/singlefs-core/src/mounted_session.rs:49: /// 用户的一次改动，经会话发布（发布路径那三种接在带文件的一版后面的改动）。
crates/singlefs-core/src/mounted_session.rs:50: #[derive(Clone, Copy, Debug)]
crates/singlefs-core/src/mounted_session.rs:51: pub enum UserChange<'content> {
crates/singlefs-core/src/mounted_session.rs:52:     /// 覆盖写一个数据单元以内的内容（`transaction::publish_overwrite`）。
crates/singlefs-core/src/mounted_session.rs:53:     Overwrite(FirstFile<'content>),
crates/singlefs-core/src/mounted_session.rs:54:     /// 顺序写整份内容，按切分纪律切成几个一单元事务（`transaction::publish_sequential_write`）。
crates/singlefs-core/src/mounted_session.rs:55:     SequentialWrite(FirstFile<'content>),
crates/singlefs-core/src/mounted_session.rs:56:     /// 建 `count` 个空 inode（`transaction::publish_new_inodes`）。
crates/singlefs-core/src/mounted_session.rs:57:     NewInodes { count: u64, write_time_seconds: u64 },
crates/singlefs-core/src/mounted_session.rs:58: }
- 会话的 `current` 是 `PoolVersion::WithoutFile` 时直接拒（`UserChangeRefused::NoFileVersionToChange`，`mounted_session.rs:259-262`、`79-81`）；第一个文件版本不经会话，走 `transaction::publish_first_file`（`mounted_session.rs:79` 的文档）。产品路径（`singlefs-core/src`）里没有 `publish_first_file` 的调用点（命令 `grep -rn 'publish_first_file(' crates/singlefs-core/src | grep -v '///' | wc -l` 输出 0；不带括号搜 `publish_first_file`，在 `singlefs-core/src` 里（`grep -n 'publish_first_file' *.rs` 共 16 行）只有定义 `transaction.rs:4624`，其余 15 行是文档与注释（`allocator.rs:1645`、`recovery.rs:2115`、`mounted_session.rs:79`、`transaction.rs:3362`、`3372`、`3378`、`3386`、`3389`、`4748`、`4842`、`4880`、`4911`、`4938`、`4982`、`4992`）；调用在 `crates/singlefs-harness/src/scenario.rs:171`、`history.rs:896`、`history.rs:2596` 与 checker-tier 的测试和装置里）。
- 会话自己不判准入；判在发布路径（b2）。推抬 F 的预算：`publishes_pushed` 从 0 起（`mounted_session.rs:298`），每次推成加 `raised.publishes.len()`（`:332`）。
- 会话模块文档对「写行那次之前不推」的原话：
crates/singlefs-core/src/mounted_session.rs:3: //! 发布路径报空间准入不够（`PublishError::SpaceAdmissionRefused`）、或准入放行而落点取不到（`PublishError::PlacementRefused`）时，
crates/singlefs-core/src/mounted_session.rs:4: //! 先推一串抬 F 的空发布（`mount::push_one_floor_raise_within_the_admission_budget`：F 抬到准入上限，一次准入最多
crates/singlefs-core/src/mounted_session.rs:5: //! `mount::PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，算上这次用户发布），再重判；做满仍不够、或抬不动了，才报空间不够。
crates/singlefs-core/src/mounted_session.rs:6: //! 写行那次发布之前不推：会话在可写挂载交回之后才有，写行那次早已发完（可写挂载那一处的推归 `mount::establish_instance`）。

## (f) 释放路径

### f1 经映射释放：`roles_replaced_via_mapping` 与 `placements_to_release_via_mapping`（原文）
crates/singlefs-core/src/transaction.rs:2841: /// 这次发布经映射换下的角色：这次重写的角色，加上文件内容重写时上一版比这一版多出来的那几个数据单元（文件变短）——
crates/singlefs-core/src/transaction.rs:2842: /// 它们这次没有对应的新角色，却同样被换下，不释放它们，它们就一直占着槽、再也没有树引用。释放判定路径
crates/singlefs-core/src/transaction.rs:2843: /// （`placements_to_release_via_mapping`）与释放之前读盘核校验和（`copies_failing_the_release_checksum_check`）读的是这同一张清单
crates/singlefs-core/src/transaction.rs:2844: /// （D19（块指针的结构与宽度预算） 已定项 5 第 1 条与硬规则 1：经映射释放的每一个都核）。文件内容这次没重写（写行、暖机）时不加。
crates/singlefs-core/src/transaction.rs:2845: fn roles_replaced_via_mapping(
crates/singlefs-core/src/transaction.rs:2846:     previous: &TransactionOutput,
crates/singlefs-core/src/transaction.rs:2847:     roles: &[TransactionUnit],
crates/singlefs-core/src/transaction.rs:2848: ) -> Vec<TransactionUnit> {
crates/singlefs-core/src/transaction.rs:2849:     let previous_data_units = previous.data_pointers.len();
crates/singlefs-core/src/transaction.rs:2850:     let rewritten_data_units = roles
crates/singlefs-core/src/transaction.rs:2851:         .iter()
crates/singlefs-core/src/transaction.rs:2852:         .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
crates/singlefs-core/src/transaction.rs:2853:         .count();
crates/singlefs-core/src/transaction.rs:2854:     let previous_data_units_without_a_successor: Vec<TransactionUnit> = if rewritten_data_units == 0
crates/singlefs-core/src/transaction.rs:2855:     {
crates/singlefs-core/src/transaction.rs:2856:         Vec::new()
crates/singlefs-core/src/transaction.rs:2857:     } else {
crates/singlefs-core/src/transaction.rs:2858:         (rewritten_data_units..previous_data_units)
crates/singlefs-core/src/transaction.rs:2859:             .map(|position| {
crates/singlefs-core/src/transaction.rs:2860:                 TransactionUnit::Data(DataUnitIndexInFile(
crates/singlefs-core/src/transaction.rs:2861:                     u64::try_from(position).expect("单元序号"),
crates/singlefs-core/src/transaction.rs:2862:                 ))
crates/singlefs-core/src/transaction.rs:2863:             })
crates/singlefs-core/src/transaction.rs:2864:             .collect()
crates/singlefs-core/src/transaction.rs:2865:     };
crates/singlefs-core/src/transaction.rs:2866:     roles
crates/singlefs-core/src/transaction.rs:2867:         .iter()
crates/singlefs-core/src/transaction.rs:2868:         .copied()
crates/singlefs-core/src/transaction.rs:2869:         .chain(previous_data_units_without_a_successor)
crates/singlefs-core/src/transaction.rs:2870:         .collect()
crates/singlefs-core/src/transaction.rs:2871: }
crates/singlefs-core/src/transaction.rs:2758: pub fn placements_to_release_via_mapping(
crates/singlefs-core/src/transaction.rs:2759:     previous: &TransactionOutput,
crates/singlefs-core/src/transaction.rs:2760:     allocator: &PoolAllocator,
crates/singlefs-core/src/transaction.rs:2761:     roles: &[TransactionUnit],
crates/singlefs-core/src/transaction.rs:2762: ) -> Result<Vec<Placement>, PublishError> {
crates/singlefs-core/src/transaction.rs:2763:     let mut placements = Vec::new();
crates/singlefs-core/src/transaction.rs:2764:     for identity in roles_replaced_via_mapping(previous, roles) {
crates/singlefs-core/src/transaction.rs:2765:         let locations = match identity {

### f2 释放进 defer：`release_leaving_the_record_allocated_on` 与 `mark_released`（原文）
crates/singlefs-core/src/allocator.rs:1275:     /// 释放一个落点（D3（空间分配） 已定项 7：「释放」= 放进 defer 队列那一刻）：每盘那条分配记录改写成已释放 + 释放代，
crates/singlefs-core/src/allocator.rs:1276:     /// 条目不删；槽仍占着，要等释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配（D16（发布语义） 已定项 1）。
crates/singlefs-core/src/allocator.rs:1277:     pub fn release(&mut self, placement: Placement, release_generation: CheckpointTxg) {
crates/singlefs-core/src/allocator.rs:1278:         self.release_leaving_the_record_allocated_on(placement, release_generation, &[]);
crates/singlefs-core/src/allocator.rs:1279:     }
crates/singlefs-core/src/allocator.rs:1280: 
crates/singlefs-core/src/allocator.rs:1281:     /// 同 [`Self::release`]，只是 `devices_whose_record_stays_allocated` 那几块盘上的那条分配记录一个字节都不动、留在「已分配」：
crates/singlefs-core/src/allocator.rs:1282:     /// 释放之前按位置项读盘核校验和、核出对不上（读不出也算）的那一份（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案）——
crates/singlefs-core/src/allocator.rs:1283:     /// 那一份的槽不还回空闲池；记录落盘即跨重挂，重挂之后从分配记录树重建的分配器照样认它已分配，准入里照已分配算、不另进式子。
crates/singlefs-core/src/allocator.rs:1284:     /// 逐盘各算：另一块盘上核得上的那一份照常释放。
crates/singlefs-core/src/allocator.rs:1285:     pub fn release_leaving_the_record_allocated_on(
crates/singlefs-core/src/allocator.rs:1286:         &mut self,
crates/singlefs-core/src/allocator.rs:1287:         placement: Placement,
crates/singlefs-core/src/allocator.rs:1288:         release_generation: CheckpointTxg,
crates/singlefs-core/src/allocator.rs:1289:         devices_whose_record_stays_allocated: &[DeviceIdentity],
crates/singlefs-core/src/allocator.rs:1290:     ) {
crates/singlefs-core/src/allocator.rs:1291:         for device in &mut self.devices {
crates/singlefs-core/src/allocator.rs:1292:             if devices_whose_record_stays_allocated.contains(&device.device) {
crates/singlefs-core/src/allocator.rs:1293:                 continue;
crates/singlefs-core/src/allocator.rs:1294:             }
crates/singlefs-core/src/allocator.rs:1295:             let record = self
crates/singlefs-core/src/allocator.rs:1296:                 .records
crates/singlefs-core/src/allocator.rs:1297:                 .iter_mut()
crates/singlefs-core/src/allocator.rs:1298:                 .find(|record| record.device == device.device && record.slot == placement.slot)
crates/singlefs-core/src/allocator.rs:1299:                 .expect("每块盘上都有这个落点的分配记录：盘上读来的两棵账对不对称由 transaction::placement_registered_unreleased_on_every_device 逐盘判过（panic 面普查 R10；经映射与第一个文件版本换下 mkfs 那几片两条路都走它）");
crates/singlefs-core/src/allocator.rs:1300:             assert!(
crates/singlefs-core/src/allocator.rs:1301:                 !record.is_released,
crates/singlefs-core/src/allocator.rs:1302:                 "同一个落点释放了两次：这块盘的记录是不是已释放由 transaction::placement_registered_unreleased_on_every_device 逐盘判过"
crates/singlefs-core/src/allocator.rs:1303:             );
crates/singlefs-core/src/allocator.rs:1304:             assert_eq!(
crates/singlefs-core/src/allocator.rs:1305:                 u64::from(record.span_slots),
crates/singlefs-core/src/allocator.rs:1306:                 placement.span,
crates/singlefs-core/src/allocator.rs:1307:                 "释放的跨度与分配记录不符：这块盘的记录跨度由 transaction::placement_registered_unreleased_on_every_device 逐盘判过"
crates/singlefs-core/src/allocator.rs:1308:             );
crates/singlefs-core/src/allocator.rs:1309:             record.is_released = true;
crates/singlefs-core/src/allocator.rs:1310:             record.generation = release_generation;
crates/singlefs-core/src/allocator.rs:1311:             device.mark_released(placement.slot, placement.span);
crates/singlefs-core/src/allocator.rs:1312:         }
crates/singlefs-core/src/allocator.rs:1313:         match self.reuse_window {
crates/singlefs-core/src/allocator.rs:1314:             ReuseWindow::GatedByTheRollbackFloor => {}
crates/singlefs-core/src/allocator.rs:1315:             // 窗口置 0：回收判定的下界取这次的释放代本身（这个落点自己就满足「释放代 ≤ 下界」），
crates/singlefs-core/src/allocator.rs:1316:             // 延迟窗口按 `Immediately` 绕开。走的是与产品路径同一个回收函数，只是下界不是 F。
crates/singlefs-core/src/allocator.rs:1317:             ReuseWindow::ForcedToZero => {
crates/singlefs-core/src/allocator.rs:1318:                 let reclaimed_now =
crates/singlefs-core/src/allocator.rs:1319:                     self.reclaim_released_up_to(release_generation, ReclaimedReuse::Immediately);
crates/singlefs-core/src/allocator.rs:1320:                 self.placements_reclaimed_on_release_by_the_forced_zero_reuse_window +=
crates/singlefs-core/src/allocator.rs:1321:                     u64::try_from(reclaimed_now.len()).expect("一次回收的落点数装得进 u64");
crates/singlefs-core/src/allocator.rs:1322:             }
crates/singlefs-core/src/allocator.rs:1323:         }
crates/singlefs-core/src/allocator.rs:1324:     }
crates/singlefs-core/src/allocator.rs:548:     /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」。
crates/singlefs-core/src/allocator.rs:549:     pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {
crates/singlefs-core/src/allocator.rs:550:         let start = self.index(slot);
crates/singlefs-core/src/allocator.rs:551:         let end = start + usize::try_from(span).expect("跨度");
crates/singlefs-core/src/allocator.rs:552:         assert!(
crates/singlefs-core/src/allocator.rs:553:             self.allocated[start..end].iter().all(|taken| *taken),
crates/singlefs-core/src/allocator.rs:554:             "释放的跨度里有没分配的槽"
crates/singlefs-core/src/allocator.rs:555:         );
crates/singlefs-core/src/allocator.rs:556:         self.deferred_slots += span;
crates/singlefs-core/src/allocator.rs:557:     }

### f3 回收：`mark_reclaimed`（原文）
crates/singlefs-core/src/allocator.rs:655:     /// 回收：一个已释放、释放代 ≤ max(F_生效, 环里最旧有效根) 的落点回到空闲（D16（发布语义） 已定项 1 的可再分配谓词）：
crates/singlefs-core/src/allocator.rs:656:     /// 位图清掉、占着的槽数与 defer 队列各减、空闲加——记账的空闲字节到这一刻才动（里程碑「覆盖写、释放、回退与复用」步 5）。
crates/singlefs-core/src/allocator.rs:657:     pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {
crates/singlefs-core/src/allocator.rs:658:         let start = self.index(slot);
crates/singlefs-core/src/allocator.rs:659:         let end = start + usize::try_from(span).expect("跨度");
crates/singlefs-core/src/allocator.rs:660:         assert!(
crates/singlefs-core/src/allocator.rs:661:             self.allocated[start..end].iter().all(|taken| *taken),
crates/singlefs-core/src/allocator.rs:662:             "回收的跨度里有没分配的槽"
crates/singlefs-core/src/allocator.rs:663:         );
crates/singlefs-core/src/allocator.rs:664:         let left_free = start > 0 && !self.allocated[start - 1];
crates/singlefs-core/src/allocator.rs:665:         let right_free = end < self.allocated.len() && !self.allocated[end];
crates/singlefs-core/src/allocator.rs:666:         // runs 的增量：两边都空是把两段并成一段（−1），两边都占是新开一段（+1），一边空是接上去（不变）。
crates/singlefs-core/src/allocator.rs:667:         self.free_runs = self.free_runs + 1 - u64::from(left_free) - u64::from(right_free);
crates/singlefs-core/src/allocator.rs:668:         for index in start..end {
crates/singlefs-core/src/allocator.rs:669:             self.set_blocking_bit(index, SlotBlockingBit::Allocated, false);
crates/singlefs-core/src/allocator.rs:670:             let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
crates/singlefs-core/src/allocator.rs:671:             self.used_per_segment[segment] -= 1;
crates/singlefs-core/src/allocator.rs:672:         }
crates/singlefs-core/src/allocator.rs:673:         self.allocated_slots -= span;
crates/singlefs-core/src/allocator.rs:674:         self.deferred_slots -= span;
crates/singlefs-core/src/allocator.rs:675:         self.free_slots += span;
crates/singlefs-core/src/allocator.rs:676:     }

### f4 `allocated_slots`、`deferred_slots` 的增减一览（`grep -n 'allocated_slots\|deferred_slots' allocator.rs` 的产品代码行，行号现取）

| 动作 | 函数与行 | `allocated_slots` | `deferred_slots` | `free_slots` |
|---|---|---|---|---|
| 分配 | `mark_allocated`，allocator.rs:725-726 | `+= span`（725） | 不动 | `-= span`（726） |
| 释放（进 defer） | `mark_released`，allocator.rs:556 | 不动 | `+= span`（556） | 不动 |
| 回收 | `mark_reclaimed`，allocator.rs:673-675 | `-= span`（673） | `-= span`（674） | `+= span`（675） |
| 复活（回退） | `mark_resurrected_out_of_the_defer_queue`，allocator.rs:567-570 | 不动 | `-= span`（checked_sub，567-570） | 不动 |

字段注释（原文）：
crates/singlefs-core/src/allocator.rs:256:     /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
crates/singlefs-core/src/allocator.rs:257:     /// （I-3.1（已分配统计对得上） 的读法 2026-09-14 用户定甲：按根环里全部有效根的引用取并集）。
crates/singlefs-core/src/allocator.rs:258:     allocated_slots: u64,
crates/singlefs-core/src/allocator.rs:259:     /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
crates/singlefs-core/src/allocator.rs:260:     /// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。
crates/singlefs-core/src/allocator.rs:261:     free_slots: u64,
crates/singlefs-core/src/allocator.rs:267:     /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
crates/singlefs-core/src/allocator.rs:268:     deferred_slots: u64,

### f5 释放与回收发生在一次发布的哪一步、会不会触发需求

`publish_admitted` 里的次序（原文）：
crates/singlefs-core/src/transaction.rs:6086:     // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
crates/singlefs-core/src/transaction.rs:6087:     // 释放之前读盘核校验和核出对不上（读不出也算）的那几份：逻辑上照样释放（映射条目这次不再写它），那一份在它那块盘上的分配记录
crates/singlefs-core/src/transaction.rs:6088:     // 留在已分配、不改成已释放（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案），核得上的那一份照常释放。
crates/singlefs-core/src/transaction.rs:6089:     for placement in release {
crates/singlefs-core/src/transaction.rs:6090:         let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
crates/singlefs-core/src/transaction.rs:6091:             .iter()
crates/singlefs-core/src/transaction.rs:6092:             .filter(|copy| copy.placement == *placement)
crates/singlefs-core/src/transaction.rs:6093:             .map(|copy| copy.device)
crates/singlefs-core/src/transaction.rs:6094:             .collect();
crates/singlefs-core/src/transaction.rs:6095:         allocator.release_leaving_the_record_allocated_on(
crates/singlefs-core/src/transaction.rs:6096:             *placement,
crates/singlefs-core/src/transaction.rs:6097:             txg,
crates/singlefs-core/src/transaction.rs:6098:             &devices_whose_copy_failed_the_checksum,
crates/singlefs-core/src/transaction.rs:6099:         );
crates/singlefs-core/src/transaction.rs:6100:     }
crates/singlefs-core/src/transaction.rs:6107:     // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
crates/singlefs-core/src/transaction.rs:6108:     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
crates/singlefs-core/src/transaction.rs:6109:     for identity in rewritten {
crates/singlefs-core/src/transaction.rs:6110:         let placement = allocate_placement_for_role(allocator, *identity, txg)?;
crates/singlefs-core/src/transaction.rs:6111:         slots.insert(*identity, placement.slot);
crates/singlefs-core/src/transaction.rs:6112:     }
crates/singlefs-core/src/transaction.rs:6113:     // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，
crates/singlefs-core/src/transaction.rs:6114:     // 环里最旧有效根往前挪了就按谓词回收，记账行按回收之后的数写（与这条根一起说同一件事）。失败时整个分配器由调用方换回去。
crates/singlefs-core/src/transaction.rs:6115:     allocator.record_root_written_by_this_process(txg);

- 次序：准入（`transaction.rs:5090`、`5136`）在前，释放（`6089-6100`）、取落点（`6109-6112`）、记根与回收（`6115`，进 `record_root_written_by_this_process`，`allocator.rs:1637-1673`）在后。
- 释放函数 `PoolAllocator::release_leaving_the_record_allocated_on`（`allocator.rs:1285-1324`）与 `mark_released`（`allocator.rs:549-557`）的函数体里没有对准入的调用：`grep -n 'admission' allocator.rs` 去掉文档行之后只有 `space_admission` 字段与它的读写函数（1065-1099 一带）。所以释放自己不申请空间、也不触发需求。
- 需求只算新写的角色，不含换下的槽（`admission.rs:949-952`、`transaction.rs:5120-5123`、D28 已定项 1 接线一条，见 h1）。
- 释放产生的空间到可用的路径：换下的槽进 defer（`deferred_slots` 加），「已分配」不变（`allocator.rs:556`）；回收时才 `allocated_slots` 减（`allocator.rs:673`）。回收的触发：
  - 发布内：`record_root_written_by_this_process`（`transaction.rs:6115` 调；`allocator.rs:1654-1668`，下界 = `max(F_生效, 环里最旧有效根)`）；
  - 挂载重建：`mount.rs:1544`（`allocator.reclaim_released_up_to(`）；
  - 抬 F：`mount.rs:2485`，回收的槽扣住到带新 F 的根落满每块盘（`mount.rs:2597` 放开）；
  - 测试开关 `ReuseWindow::ForcedToZero`：释放当场回收（`allocator.rs:1317-1322`）。
- 今天「释放一个整文件」的路径不存在：释放只来自覆盖写与顺序写换下的角色（`roles_replaced_via_mapping`，上面 f1）、文件变短时多出来的数据单元（`transaction.rs:2849-2865`）、实例表旧链（`transaction.rs:5213-5226`）、第一个文件版本换下的 mkfs 树表（`transaction.rs:5209`），以及回退（`mount.rs:5227`）。`grep -rn -i 'unlink\|tombstone\|墓碑\|deleted_inodes' crates/singlefs-core/src` 命中的都是注释与测试名：`inode_tree.rs:16`（「这一版没有删除」）、`admission.rs:154`、`250`、`1166-1203`（`c375_…tombstone…` 用例，用字面结构体给已承诺预留非零值）。

## (g) 池级 checker 与不变量表

### g1 池级 checker 今天读不读标签 4、标签 6

答案：不读用于判定。依据：
- checker 自己的统计量常量只有五个：标签 1、2、5、12 与「无设备维」保留值，没有标签 4、6（原文）：
crates/singlefs-checker/src/walk.rs:111: const STATISTIC_ALLOCATED_BYTES: u16 = 1;
crates/singlefs-checker/src/walk.rs:112: const STATISTIC_FREE_BYTES: u16 = 2;
crates/singlefs-checker/src/walk.rs:113: /// defer 队列待释放（D5（快照 / 空间记账机制） 已定项 4 第 5 项的行号，带设备维，单位字节）：I-3.11（已分配减 defer 等于最新根走读） 读它。
crates/singlefs-checker/src/walk.rs:114: const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
crates/singlefs-checker/src/walk.rs:115: /// inode 号水位（D5（快照 / 空间记账机制） 已定项 4 第 12 项的行号）。
crates/singlefs-checker/src/walk.rs:116: const STATISTIC_INODE_WATERMARK: u16 = 12;
crates/singlefs-checker/src/walk.rs:117: /// 不带设备维的统计量把设备段写成这个保留值（D5（快照 / 空间记账机制） 已定项 10）。
crates/singlefs-checker/src/walk.rs:118: const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
- 记账行按 `(标签, 设备)` 整张存进 `accounting` 映射（原文，标签 4、6 的行也在这张映射里，但没人取）：
crates/singlefs-checker/src/walk.rs:1343:                 self.accounting_seen = true;
crates/singlefs-checker/src/walk.rs:1344:                 for row in &rows {
crates/singlefs-checker/src/walk.rs:1345:                     self.accounting
crates/singlefs-checker/src/walk.rs:1346:                         .insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
crates/singlefs-checker/src/walk.rs:1347:                 }
- 对 `accounting` 映射的全部取值点（命令 `grep -n 'accounting\.get\|accounting$' crates/singlefs-checker/src/walk.rs` 的行，另加 `walk.rs:6134-6136`、`6284-6286`、`6303`、`6309-6311` 原文）：只取标签 12（`walk.rs:6134-6135`）、1（`6284-6286`）、2（`6303`）、5（`6309-6310`）：
crates/singlefs-checker/src/walk.rs:6134:     let inode_number_watermark = accounting
crates/singlefs-checker/src/walk.rs:6135:         .get(&(STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION))
crates/singlefs-checker/src/walk.rs:6136:         .copied();
crates/singlefs-checker/src/walk.rs:6284:             let allocated = accounting
crates/singlefs-checker/src/walk.rs:6285:                 .get(&(STATISTIC_ALLOCATED_BYTES, *device))
crates/singlefs-checker/src/walk.rs:6286:                 .copied();
crates/singlefs-checker/src/walk.rs:6303:             let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
crates/singlefs-checker/src/walk.rs:6309:             let deferred = accounting
crates/singlefs-checker/src/walk.rs:6310:                 .get(&(STATISTIC_DEFER_QUEUE_BYTES, *device))
crates/singlefs-checker/src/walk.rs:6311:                 .copied();
- 命令 `grep -rn 'PENDING_DELETE\|COMMITTED_RESERV\|pending_delete\|committed_reservation\|待删占用\|已承诺预留' crates/singlefs-checker/src crates/singlefs-checker-tier/src` 输出 0 行。
- `crates/singlefs-checker-tier/src` 里出现的 I-3 编号（命令 `grep -rhoE 'I-3\.[0-9]+' singlefs-checker-tier/src | sort | uniq -c`）：I-3.1 23 次、I-3.8 1 次（`crash_judge_checker.wgsl:1072` 的注释）、I-3.9 3 次、I-3.10 4 次、I-3.11 4 次，没有 I-3.2 到 I-3.7；GPU 判器里记账那三条在 `crash_judge_checker_common.wgsl:544`（注释「记账那几条：I-3.1、I-5.2、I-3.11」），取记账行的调用点是 `crash_judge_checker_scan.wgsl:1522`（`c_accounting_row(1u, …)`）、`:1530`（`2u`）、`:1540`（`5u`），即标签 1、2、5；checker-tier 里 `PENDING_DELETE`、`COMMITTED_RESERV` 等名字零命中（见上一条命令输出 0 行）。

### g2 池级 checker 判 I-3.x 哪几条

命令 `grep -n '"I-3\.[0-9]*"' crates/singlefs-checker/src/*.rs`（实际在 `crates/singlefs-checker/src` 下），原样输出（共 19 行：`image.rs` 5 行是登记表、`walk.rs` 14 行是判定与报「不适用」的点）：
    singlefs-checker/src/image.rs:82:    "I-3.1",
    singlefs-checker/src/image.rs:83:    "I-3.8",
    singlefs-checker/src/image.rs:84:    "I-3.9",
    singlefs-checker/src/image.rs:85:    "I-3.10",
    singlefs-checker/src/image.rs:86:    "I-3.11",
    singlefs-checker/src/walk.rs:1846:                .judge("I-3.8", chain != ChainRecordView::Malformed, || {
    singlefs-checker/src/walk.rs:1902:            "I-3.8",
    singlefs-checker/src/walk.rs:3269:        judgements.not_applicable("I-3.9", "最新的根下面没有带已释放标志的分配记录");
    singlefs-checker/src/walk.rs:3281:            "I-3.9",
    singlefs-checker/src/walk.rs:3307:        judgements.judge("I-3.9", in_the_witnessed_interval, || {
    singlefs-checker/src/walk.rs:3322:            "I-3.9",
    singlefs-checker/src/walk.rs:3405:            "I-3.9",
    singlefs-checker/src/walk.rs:3674:            judgements.judge("I-3.10", record.generation == birth_txg, || {
    singlefs-checker/src/walk.rs:3684:            "I-3.10",
    singlefs-checker/src/walk.rs:6288:                judgements.judge("I-3.1", allocated == Some(walked), || {
    singlefs-checker/src/walk.rs:6324:                "I-3.11",
    singlefs-checker/src/walk.rs:6333:                "I-3.1",
    singlefs-checker/src/walk.rs:6338:        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
    singlefs-checker/src/walk.rs:6340:        judgements.not_applicable("I-3.11", "最新的根下面还没有记账树（第 0 代树表）");

- 判的是 I-3.1、I-3.8、I-3.9、I-3.10、I-3.11（`image.rs:82-86` 的登记表）。
- I-3.2、I-3.3 已停用；I-3.4（可用空间扣待删占用）、I-3.5、I-3.6、I-3.7 在 `invariants.md` 状态列是「未实现」，命令 `grep -n 'I-3\.[2-7]\b' crates/singlefs-checker/src/*.rs` 对四个文件各数出 0 行。

### g3 `.claude/kb/invariants.md` 里 I-3.1 到 I-3.11 整行抄（共 11 行，行号 134 到 144，表里 I-3.4 排在 I-3.6 之后、I-3.7 之前；来源命令 `awk 'NR>=134 && NR<=144' .claude/kb/invariants.md`，只抄 I-3 这几行）
.claude/kb/invariants.md:134: | I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）：checker 自己算，取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与层 0 不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于 F_生效 的（还在 defer 里，抬 F 还不许回收），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15「实例表单元指针照所选根」，走它施加在其上那一版（环里同实例、txg 比它小的根里最大的一条，不管在不在候选集里）的实例表链与根记录持有的分配记录树（`walk::walk_version_applied_only_by_records`，主 agent 2026-09-27 定，被攻过零轮）。⚠️ **已知缺口**：环里同实例、txg 更小的根一条都读不出时，那一版的实例表与根记录持有的分配记录树走不到，照旧漏数；「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`research/prompts/m2-rev-b2-implementer-report.md` 第七节第 2 条）。 |
.claude/kb/invariants.md:135: | I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
.claude/kb/invariants.md:136: | I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
.claude/kb/invariants.md:137: | I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ① `Reach(b)` 在 txg 序上**连续无洞**；② 设 R_first 为最早者，`birth(b) ≤ R_first.txg`，且若存在更早的根 R_prev 则 `birth(b) > R_prev.txg`（**双侧夹死，off-by-one 无处可藏**）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中 | 未实现 ⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
.claude/kb/invariants.md:138: | I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向） | 未实现 |
.claude/kb/invariants.md:139: | I-3.4 | 可用空间扣待删占用 | 可用空间统计已扣除「待删除但意图未完成」占用的空间 | 未实现 |
.claude/kb/invariants.md:140: | I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少） | 未实现 |
.claude/kb/invariants.md:141: | I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（2026-09-16，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例、链指针记录在末尾；回收那一半没有输入，第一版行只增不删，删行的条件没人判） |
.claude/kb/invariants.md:142: | I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条层 0 流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判） |
.claude/kb/invariants.md:143: | I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树（主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定，被攻过零轮；只接这一步——恢复连着施加几条记录时第二条及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
.claude/kb/invariants.md:144: | I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）今天没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |

## (h) kb 原文（只抄「已定」小节，不读历史版本节）

### h1 `.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 全节（行 14 到 63，下一节「已定项 2」始于行 64）
.claude/kb/decisions/28-挂载期承诺量.md:14: #### 已定项 1：准入不等式的权威形式
.claude/kb/decisions/28-挂载期承诺量.md:15: 
.claude/kb/decisions/28-挂载期承诺量.md:16: <!-- gate:admission-formula -->
.claude/kb/decisions/28-挂载期承诺量.md:17: 
.claude/kb/decisions/28-挂载期承诺量.md:18: ```
.claude/kb/decisions/28-挂载期承诺量.md:19: 可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数
.claude/kb/decisions/28-挂载期承诺量.md:20: ```
.claude/kb/decisions/28-挂载期承诺量.md:21: 
.claude/kb/decisions/28-挂载期承诺量.md:22: **定案**：分配准入按这个式子算可用。它是「任何增加空间占用的操作，进门前先算最坏情况下解开自己需要多少」（D3（空间分配） 第 3 条）的可执行形式。
.claude/kb/decisions/28-挂载期承诺量.md:23: 
.claude/kb/decisions/28-挂载期承诺量.md:24: - **逐设备算，每块盘各自满足**（逐设备合取）：式子对每块盘 d 各算一份，准入要每一块盘都够，一块盘不够就拒，不拿别的盘的富余补。前五项（含挂载期承诺量里的切换预留、被抛弃根独占量）取这块盘自己的值——两份副本各落一块盘，一块盘满了不能靠别的盘补；后三项——待删占用、已承诺预留、checkpoint 保留池——是全池的承诺量，**按全部副本之和记字节**（物理字节，与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」同一口径），摊到每块盘各扣「副本之和 ÷ 副本数」。这个分界决定 D5（快照 / 空间记账机制） 已定项 4 里哪几个统计量要带设备维。「需求」摊到每块盘按什么记，仍是 C370（需求、可用与 df 没有共同单位） 那一格。
.claude/kb/decisions/28-挂载期承诺量.md:25: - **挂载期承诺量**（第五项）：挂载期间常量、只住内存、不落盘、每次挂载重算，按设备分摊。成员两个：实例切换的预留（已定项 3，按设备摊）与 checkpoint 保留池（已定项 4，全池的量、每次发布现算，式子里单列第八项、逐设备摊扣）。
.claude/kb/decisions/28-挂载期承诺量.md:26: - **checkpoint 保留池**（第八项）：挂载期承诺量的池级成员，按当时各记录树的高之和 + 记账节点现算，每次发布重算、不设常数上限、只住内存；按全部副本之和记，摊到每块盘扣「副本之和 ÷ 副本数」（固定点写出的每一块按 D2（RAID 条带策略） 已定项 6 落两列，每块盘各落一份）；它保证 checkpoint 自己的固定点写得出去。
.claude/kb/decisions/28-挂载期承诺量.md:27: - **被抛弃根独占量**（第九项）：取值 = 只被被抛弃根引用的槽数 = 被抛弃根引用的槽 − 回退候选集里的根引用的槽（下界 0），与分配器实际隔离的集合是同一个数；各条根引用的槽从它的分配记录树读、不加盘上字段；每次挂载与每次抬 F 时算出、只住内存、按设备算，被抛弃的根被轮转覆写时清零；被抛弃的根今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。影子账开着时，分配记录树两块盘各多写一片叶，准入读数两边之差 = 被隔离的槽 + 这几个节点（实二一报告「停下交主 agent」第 8 条）。
.claude/kb/decisions/28-挂载期承诺量.md:28: 
.claude/kb/decisions/28-挂载期承诺量.md:29: **接线**（实二五，被攻过零轮）：
.claude/kb/decisions/28-挂载期承诺量.md:30: 
.claude/kb/decisions/28-挂载期承诺量.md:31: - 发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前。
.claude/kb/decisions/28-挂载期承诺量.md:32: - 可写挂载在 `mount::establish_instance` 里判：取号与预演之前，这一刻的需求逐盘记 0。
.claude/kb/decisions/28-挂载期承诺量.md:33: - 拒了在任何写之前返回：发布路径报 `PublishError::SpaceAdmissionRefused`，可写挂载报 `MountError::SpaceAdmissionRefusedBeforeAcquisition`。
.claude/kb/decisions/28-挂载期承诺量.md:34: - 需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）；判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判（用户 2026-09-25 定 ND+A1n，被攻过一轮，安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`）。读数取分配器此刻的计数。
.claude/kb/decisions/28-挂载期承诺量.md:35: - ckpt_cost 照已定项 4 的 Σ 名单算。
.claude/kb/decisions/28-挂载期承诺量.md:36: - 发布经挂着的会话调（`crates/singlefs-core/src/mounted_session.rs`，实五）时，准入拒或落点被拒由会话照 D16（发布语义） 已定项 1「准入」那一行推抬 F 的空发布、再重发；可写挂载在 `mount::establish_instance` 写行与暖机之后推。
.claude/kb/decisions/28-挂载期承诺量.md:37: 
.claude/kb/decisions/28-挂载期承诺量.md:38: **射程**：式子里每一项是不是被维护的统计量，逐项对照写在 D5（快照 / 空间记账机制） 已定项 4 的「式子逐项对照」表里（那里是唯一的对照位，四处例外——设备容量、挂载期承诺量、被抛弃根独占量与 checkpoint 保留池——也写在那里）；D5（快照 / 空间记账机制） 已定项 4 的统计量清单相对这个式子判完备。式子里的统计量读的是已发布值（挂载期承诺量、被抛弃根独占量与 checkpoint 保留池是挂载时或每次发布现算的内存量，不在此列），准入却发生在窗口中间，读数怎么合成归已定项 2。
.claude/kb/decisions/28-挂载期承诺量.md:39: 
.claude/kb/decisions/28-挂载期承诺量.md:40: **依据**：每一项为什么在式子里——
.claude/kb/decisions/28-挂载期承诺量.md:41: 
.claude/kb/decisions/28-挂载期承诺量.md:42: | 项 | 为什么在式子里 |
.claude/kb/decisions/28-挂载期承诺量.md:43: |---|---|
.claude/kb/decisions/28-挂载期承诺量.md:44: | 容量、**已分配** | I-3.1（已分配统计对得上） 的被审计对象 |
.claude/kb/decisions/28-挂载期承诺量.md:45: | **不可回收** | zoned 上一个活快照钉住的物理空间是 zone 粒度而引用的块数是块粒度，[invariants.md](../invariants.md) 逐字要求「另立一个不可回收量」 |
.claude/kb/decisions/28-挂载期承诺量.md:46: | **待删占用** | I-3.4（可用空间扣待删占用）；D5（快照 / 空间记账机制） 与 D8（核心索引结构） 分批删除定「待删除结构算作活引用」 |
.claude/kb/decisions/28-挂载期承诺量.md:47: | **已承诺预留** | D3（空间分配） 已定项 15；D3（空间分配） 已定项 2 逐字「墓碑就是解开自己的一部分，计入那个量」 |
.claude/kb/decisions/28-挂载期承诺量.md:48: | **挂载期承诺量** | 用户定案（C126（切换预留的最坏量没有口径） 第一轮）：实例切换的预留不落盘、按设备算，塞不进「已承诺预留」那一格 |
.claude/kb/decisions/28-挂载期承诺量.md:49: | **被抛弃根独占量** | 用户定案（C318（影子账隔离的单元没进准入不等式））：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账之后，只被被抛弃的根引用的单元在当前账里是空闲的、分配器却不许拿它们，直到被抛弃的根离开根环；它们既不在「已分配」里也不在「defer 待释放」里，不扣就高估可用。取值口径见 `research/prompts/m2-step45-code-r3-main-verification.md` 第六节 1 |
.claude/kb/decisions/28-挂载期承诺量.md:50: | **checkpoint 保留池** | 用户定案（已定项 4） |
.claude/kb/decisions/28-挂载期承诺量.md:51: 
.claude/kb/decisions/28-挂载期承诺量.md:52: 实验：
.claude/kb/decisions/28-挂载期承诺量.md:53: 
.claude/kb/decisions/28-挂载期承诺量.md:54: - E19（defer 窗口下的假性 ENOSPC）：保留池不进准入时 checkpoint 会卡死——第八项。
.claude/kb/decisions/28-挂载期承诺量.md:55: - E82（准入的在飞合成）：只读已发布值会同窗超卖——这个式子要配已定项 2。
.claude/kb/decisions/28-挂载期承诺量.md:56: - E141（切换预留的挂载准入自证）：按设备判与按池级标量判，在一块盘短 1 块而两块盘加起来够的池上分得开——括号里按设备求和。
.claude/kb/decisions/28-挂载期承诺量.md:57: - E150（回退复用被抛弃的根引用的单元）：带第九项的可用等于分配器发得出的槽数，不带就多报被隔离的单元——第九项。
.claude/kb/decisions/28-挂载期承诺量.md:58: - 用户定案 2026-09-23：准入逐设备合取（C355（checkpoint 保留池池级扣而固定点每块要落两列） 取丁）；待删占用与已承诺预留按全部副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款），第三轮三方写回：一份副本那一读法让这个式子与 I-3.4（可用空间扣待删占用） 当场为假）；判决 `research/prompts/c355-c363-r3-main-verification.md` 第二、三、五节，原话与被推翻的那句 2026-09-13 原话在变更史。
.claude/kb/decisions/28-挂载期承诺量.md:59: - 实二五报告 `research/prompts/m2-e25-implementer-report.md`。
.claude/kb/decisions/28-挂载期承诺量.md:60: - 删「defer 待释放」、需求改成新写的全部槽：用户 2026-09-25 定案（原话在变更史），安全设计轮第二轮判决 `research/prompts/m2-safety-r2-main-verification.md`（「defer 待释放」与「已分配」重复扣）、第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`。
.claude/kb/decisions/28-挂载期承诺量.md:61: 
.claude/kb/decisions/28-挂载期承诺量.md:62: **欠**：C355（checkpoint 保留池池级扣而固定点每块要落两列）、C375（待删占用与已承诺预留按份还是按和记没有条款）：按逐设备合取与副本之和的准入对拍，等 `crates/` 实现准入式子；C70（跨统计量的和没人盯）：按这个式子写那条向量等式；C87（准入读的合成值无定义）。
.claude/kb/decisions/28-挂载期承诺量.md:63: 

### h2 `.claude/kb/decisions/03-空间分配.md` 已定项 2 全节（行 50 到 63）
.claude/kb/decisions/03-空间分配.md:50: #### 已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况
.claude/kb/decisions/03-空间分配.md:51: 
.claude/kb/decisions/03-空间分配.md:52: **定案**：删除这个操作本身要写盘——D18（块里携带什么信息） 已定项 8 定「删除要写墓碑单元」，可写头里删块时还要往 livelist 追加一条 FREE 条目（36 字节，D6（快照实现模型） 已定项 2），而已定项 17 的原则逐字写着「COW 系统必须保证『释放空间』这个操作本身不需要申请空间」。解法不是给删除路径开口子，是把那一笔提前到写入那一刻收掉：已定项 15 的准入控制本来就要求「任何增加空间占用的操作，进门前先算**最坏情况下解开自己需要多少**」——**墓碑就是「解开自己」的一部分**，计入那个量；livelist 的 FREE 条目同样是「解开自己」的一部分，计入那个量、写入时预付。⇒ 删除路径仍然不申请空间：已定项 17 的原则一个字不改，改的是「解开自己需要多少」这个量的算法。
.claude/kb/decisions/03-空间分配.md:53: 
.claude/kb/decisions/03-空间分配.md:54: **射程**：定的是这笔账归谁付（准入控制，写入时付），不是这笔账是多少——一次删除写几个墓碑、能不能批量摊、盘满时删得动删不动，一个数都没有，要等分配器与准入控制有实现才算得出来。它把已定项 15「预留量动态贴着实际需求，不白占」变紧了：预留量现在与「将来会不会被删」绑在一起，墓碑那一项对永不删除的对象是纯浪费，浪费多少同样没量。
.claude/kb/decisions/03-空间分配.md:55: 
.claude/kb/decisions/03-空间分配.md:56: **依据**：
.claude/kb/decisions/03-空间分配.md:57: 
.claude/kb/decisions/03-空间分配.md:58: - E82（准入的在飞合成）：忘掉墓碑那一笔时盘满后删除全数死锁——「解开自己」含墓碑是可执行的，不是修辞。
.claude/kb/decisions/03-空间分配.md:59: - E130（每头一份 livelist 过不过有界销毁）：每释放一个块要多写的那几个字节把 livelist 追加抬成「解开自己」的一部分，不预付就与已定项 17 的原则冲突。
.claude/kb/decisions/03-空间分配.md:60: - 用户定案（墓碑那一半与后加的 livelist 那一半），原话在变更史。
.claude/kb/decisions/03-空间分配.md:61: 
.claude/kb/decisions/03-空间分配.md:62: **欠**：C260（livelist 条目的预付没算进解开自己那个量）。
.claude/kb/decisions/03-空间分配.md:63: 

### h3 `.claude/kb/decisions/03-空间分配.md` 已定项 15 全节（行 275 到 287）
.claude/kb/decisions/03-空间分配.md:275: #### 已定项 15：空间预留取准入控制形态
.claude/kb/decisions/03-空间分配.md:276: 
.claude/kb/decisions/03-空间分配.md:277: **定案**：空间预留用准入控制形态，量要小。预留不是「划一块地不给用」，而是：任何增加空间占用的操作，进门前先算「**最坏情况下解开自己需要多少**」，算不过就拒绝。这样预留量动态贴着实际需求，不白占；量要小。它的可执行形式是 D28（挂载期承诺量） 已定项 1 的九项式子，读数怎么合成在 D28（挂载期承诺量） 已定项 2，登记位是已定项 7 那张七条合取表。
.claude/kb/decisions/03-空间分配.md:278: 
.claude/kb/decisions/03-空间分配.md:279: **射程**：定的是形态与方向（量要小、贴着需求），不定任何一个具体的数——每一项算多少在 D28（挂载期承诺量），整理的常驻底额在 D26（后台整理与放置回收） 已定项 3。「解开自己需要多少」这个量含哪几笔由已定项 2 扩写（墓碑、livelist 的 FREE 条目），扩一笔就把「不白占」紧一分。
.claude/kb/decisions/03-空间分配.md:280: 
.claude/kb/decisions/03-空间分配.md:281: **依据**：
.claude/kb/decisions/03-空间分配.md:282: 
.claude/kb/decisions/03-空间分配.md:283: - E82（准入的在飞合成）：三种直觉写法各出一类可数违例——只读已发布值同窗超卖、把窗口内释放记成可用会吃掉 defer 该扣住的块、忘了墓碑盘满后删除全数死锁 ⇒ 准入控制这个形态要配在飞合成才立得住，而不是划一块地。
.claude/kb/decisions/03-空间分配.md:284: - 用户定案，原话在变更史。
.claude/kb/decisions/03-空间分配.md:285: 
.claude/kb/decisions/03-空间分配.md:286: **欠**：C87（准入读的合成值无定义）。
.claude/kb/decisions/03-空间分配.md:287: 

### h4 `.claude/kb/decisions/03-空间分配.md` 已定项 17 全节（行 302 到 315）
.claude/kb/decisions/03-空间分配.md:302: #### 已定项 17：释放空间这个操作本身不需要申请空间
.claude/kb/decisions/03-空间分配.md:303: 
.claude/kb/decisions/03-空间分配.md:304: **定案**：COW 系统必须保证「释放空间」这个操作本身不需要申请空间。btrfs 没在设计层保证这点，靠一堆启发式补了十几年还在出问题。凡是释放路径要写的字节（墓碑单元、livelist 的 FREE 条目），空间在写入那一刻就按已定项 15 的准入控制预付掉（已定项 2）。
.claude/kb/decisions/03-空间分配.md:305: 
.claude/kb/decisions/03-空间分配.md:306: **射程**：它是 D3（空间分配） 里最硬的一条，别的分项都要过它：已定项 2 定的是怎么在不破它的前提下容纳「删除要写盘」，已定项 9 第 2 条定的是删掉的空间多久回来。它不承诺「释放一定立刻成功」——被 defer 窗口与回退下界钉住的那几代按 D16（发布语义） 已定项 1 的口径不算空闲，那是真 ENOSPC 不是破例。
.claude/kb/decisions/03-空间分配.md:307: 
.claude/kb/decisions/03-空间分配.md:308: **依据**：
.claude/kb/decisions/03-空间分配.md:309: 
.claude/kb/decisions/03-空间分配.md:310: - E130（每头一份 livelist 过不过有界销毁）：候选甲每释放一个块要多写的那几个字节逐字撞上这条原则，逼出「每批最坏空间需求有界且可在批前预留」这条判据的可执行读法。
.claude/kb/decisions/03-空间分配.md:311: - E82（准入的在飞合成）：忘掉墓碑那一笔的臂在盘满后删除全数死锁——这条原则被破掉时的现象是可数的。
.claude/kb/decisions/03-空间分配.md:312: - 用户定案，原话在变更史。
.claude/kb/decisions/03-空间分配.md:313: 
.claude/kb/decisions/03-空间分配.md:314: **欠**：C260（livelist 条目的预付没算进解开自己那个量）。
.claude/kb/decisions/03-空间分配.md:315: 

## 没做什么

- 没推荐、没论证；没判预付怎么接、各批要预付多少、续做怎么挂（这些归下一轮三方）。
- 没读主仓 `crates/`；没跑 cargo、测试、门禁；没改仓内任何文件；没提交。
- 不在读范围里的没读：`write_request_split.rs`（数据单元切分的具体算法，只引了 `transaction.rs` 对它的文档）、`recovery.rs` 里记账行解析的值校验（只引了数量与水位行判定）、`admission.rs` 的 `mod tests` 内部（只引了出现 `of_the_first_version` 与字面结构体的行号）、`PoolAllocator::rebuild_from_records` 怎么由分配记录推 `allocated_slots`/`deferred_slots`（`allocator.rs:388-390` 起步 0，重建路径没展开）。
- (a)(b) 两节的「推的」句子（建 inode 被判、没有现成字段或参数带预付量）依据都在句子后面的原文里；它们会被推翻的现象：`PublishPlan` 或 `PoolAllocator` 在冻结副本之后新增了带预付量的字段，或者 `transaction.rs:5125`、`admission.rs:1021` 两处的调用形状变了。
- 这份报告里的 `crates/…` 行号都对冻结副本 `/tmp/claude-1000/m3-delete-r2/tree/crates/`；主仓 `crates/` 之后若有改动，行号会漂移。kb 行号对的是主仓 `.claude/kb/` 在这一次读的时候。
- 写报告时有两次追加超过了「每次写不超过 150 行」的分段（b1 一次约 160 行），内容是整块原文，没从表格或代码块中间切。
- 自己建的东西：只有草稿目录 `/tmp/claude-1000/m3-prepay-facts/` 里的 `q.sh`、`refs.txt`、`g1.txt`、`g2.txt`、`inv.txt`、`d28.txt`、`d3-2.txt`、`d3-15.txt`、`d3-17.txt` 与本报告；没建仓副本、编译目录或工作树，没有要删的副本。
