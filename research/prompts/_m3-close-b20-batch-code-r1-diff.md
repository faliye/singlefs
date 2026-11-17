# m3-close-b20-batch-code-r1 被判的产品代码 diff（第 20 批修补代码轮之后的七份补丁，按打补丁次序）

## 补丁 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-fixes/patch/crates.patch`（只摘 crates/singlefs-core/src/ 下的文件）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 12d31fe1b6b9622f19d0c7477705bce02ef4e6af..5171f20776617d510413aa8e20e047e4e894f0e0 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1471,7 +1471,8 @@ struct InstanceStart {
 }
 
 /// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
-/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。
+/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。这一遍直接读根环，`chosen_root`（读阶段择到的那条根）再取一次 max：两次读环之间的瞬时读错
+/// （择根那一遍读到了它、这一遍没读到）不把下界拉到它之下。所选根施加前缀之后的那一版不另收：它的 txg 来自 `records` 里的记录，已在记录那一半里。
 ///
 /// # Errors
 /// 那个最大值已是 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条：盘上读来的 txg，不 panic）。
@@ -1479,6 +1480,7 @@ fn first_txg_of_new_instance<Device: BlockDevice>(
     devices: &[(DeviceIdentity, Device)],
     system_configuration: &crate::system_configuration::SystemConfiguration,
     records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    chosen_root: &RootRecord,
 ) -> Result<CheckpointTxg, MountError> {
     let highest_record_txg = records
         .values()
@@ -1491,7 +1493,8 @@ fn first_txg_of_new_instance<Device: BlockDevice>(
         &system_configuration.immutable.sizes,
         &system_configuration.immutable.filesystem_identifier,
     )
-    .unwrap_or(CheckpointTxg(0));
+    .unwrap_or(CheckpointTxg(0))
+    .max(chosen_root.checkpoint_txg);
     checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))
 }
 
@@ -9679,6 +9682,8 @@ pub fn mount_writable_with_background_compute_settings_the_gc_cpu_compensation_t
 /// 可写挂载择系统配置（S3，C707）：先按调用方给的本池 fsid 筛（[`choose_system_configuration_of_the_pool`]：自述别的 fsid 的槽当自证不过）；
 /// 筛完一块盘可择的都没有（`NoValidSystemConfiguration`：盘上没有一槽是调用方说的这个池的）才不筛再择一次，择到的 fsid 与调用方的不同，
 /// 由后面的参数核报成 [`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`]（调用方交错了 fsid 那一形照旧报）。
+/// 只读挂载与恢复带本池 fsid 的入口用同一个筛法（`recovery::choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot`），
+/// 不走不筛再择那一步：它们没有参数核，再择一次就读到调用方没点名的那个池。
 ///
 /// # Errors
 /// 筛过之后择系统配置报的错（不是一槽都没有的那一种）；不筛那一次报的错。
@@ -9868,7 +9873,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         .max()
         .unwrap_or(0)
         + 1;
-    let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records)?;
+    let first_txg =
+        first_txg_of_new_instance(devices, &system_configuration, &records, &chosen_root)?;
     let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
         devices,
         &system_configuration,
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 0cced883a7a88dd51142a089df35e16b8e0afaf5..eb1072494c2f6e4893f83fb666fc1c9f792b4c91 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -1808,11 +1808,26 @@ pub fn choose_system_configuration_of_the_pool(
     reader: &dyn PoolReader,
     filesystem_identifier_of_the_pool: &[u8; 16],
 ) -> Result<SystemConfiguration, RecoveryFailure> {
+    choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
+        reader,
+        filesystem_identifier_of_the_pool,
+    )
+    .map(|choice| choice.system_configuration)
+}
+
+/// 与 [`choose_system_configuration_of_the_pool`] 同一个筛法，另交回两个槽都不可择而被跳过的盘（别的池的槽筛掉之后两槽都不可择的盘也在内）：
+/// 带本池 fsid 的只读挂载与恢复（`mounted_read::mount_read_only_of_the_pool`、[`recover_of_the_pool`]）用。
+///
+/// # Errors
+/// 同 [`choose_system_configuration_of_the_pool`]；盘上没有一槽是这个 fsid 的池的报 [`RecoveryFailure::NoValidSystemConfiguration`]。
+pub fn choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
+    reader: &dyn PoolReader,
+    filesystem_identifier_of_the_pool: &[u8; 16],
+) -> Result<SystemConfigurationChoice, RecoveryFailure> {
     choose_system_configuration_among_the_slots_of(
         reader,
         SlotsOfWhichPool::OnlyThePool(*filesystem_identifier_of_the_pool),
     )
-    .map(|choice| choice.system_configuration)
 }
 
 /// [`choose_system_configuration_naming_devices_without_a_mountable_slot`] 与 [`choose_system_configuration_of_the_pool`] 共用的择法，
@@ -8320,14 +8335,46 @@ pub fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the
 /// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版三样读者的并集里有字节丢了、没有内容拒收，
 /// 系统配置又没见证它时退到 [见证到的那一版, 所选根) 里读得出、不被抛弃的最新一条根（[`fall_back_judged_on_the_three_readers`]）：
 /// 那一版走得通，`root` 与 `effective_root` 都报退到的那一条，`journal` 仍是对所选根扫环、施加那一趟的报告；走不通留在所选那一版、
-/// 照它自己的走读报。不看 journal 的那一臂（`JournalPolicy::Ignore`）不退。
+/// 照它自己的走读报。不看 journal 的那一臂（`JournalPolicy::Ignore`）不退。系统配置不筛 fsid；调用方知道本池 fsid 的用 [`recover_of_the_pool`]。
 #[must_use]
 pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
+    recover_from_the_system_configuration_choice(
+        reader,
+        policy,
+        choose_system_configuration_naming_devices_without_a_mountable_slot(reader),
+    )
+}
+
+/// 与 [`recover`] 同一条恢复路径，系统配置只从自述 fsid 是 `filesystem_identifier_of_the_pool` 的槽里择
+/// （[`choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot`]）：别的池的槽世代号再大也择不到。
+/// 盘上没有一槽是这个 fsid 的池的，报告的失败是 [`RecoveryFailure::NoValidSystemConfiguration`]，不读别的池。
+#[must_use]
+pub fn recover_of_the_pool(
+    reader: &dyn PoolReader,
+    policy: JournalPolicy,
+    filesystem_identifier_of_the_pool: &[u8; 16],
+) -> RecoveryReport {
+    recover_from_the_system_configuration_choice(
+        reader,
+        policy,
+        choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
+            reader,
+            filesystem_identifier_of_the_pool,
+        ),
+    )
+}
+
+/// [`recover`] 与 [`recover_of_the_pool`] 共用的恢复路径：系统配置已经择过，择不到的把失败报成报告。
+fn recover_from_the_system_configuration_choice(
+    reader: &dyn PoolReader,
+    policy: JournalPolicy,
+    system_configuration_choice: Result<SystemConfigurationChoice, RecoveryFailure>,
+) -> RecoveryReport {
     let mut mapping_fallbacks = 0;
     let SystemConfigurationChoice {
         system_configuration,
         devices_without_a_mountable_system_configuration_slot,
-    } = match choose_system_configuration_naming_devices_without_a_mountable_slot(reader) {
+    } = match system_configuration_choice {
         Ok(choice) => choice,
         Err(failure) => {
             return RecoveryReport {
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index 6fa24b405f3759ec975d31fd5ca97010c0c4d8aa..6c3bb79818683f9e0afb1eb1396b62f5759be2cc 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -45,6 +45,7 @@ use crate::records::{
 };
 use crate::recovery::{
     choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
+    choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot,
     fall_back_judged_on_the_three_readers,
     inode_record_is_not_below_the_separator_key_and_the_container,
     inode_records_of_the_leaf_ascend_after_the_leaves_before,
@@ -643,14 +644,43 @@ pub struct MountedReadOnly {
 ///
 /// **扫环只在这一步发生一次**：之后每一次 [`OpenFileForRead::read_at`] 都不碰 journal 环。
 /// 崩溃点重放的每个恢复结果上也能跑这一条——`reader` 换成那个崩溃镜像即可（并行线二验收第 3 条；
-/// 枚举那一半归验证包，由主 agent 经门禁 crash-point-replay 跑）。
+/// 枚举那一半归验证包，由主 agent 经门禁 crash-point-replay 跑）。系统配置不筛 fsid；管理员知道本池 fsid 的用 [`mount_read_only_of_the_pool`]。
 ///
 /// # Errors
 /// 见 [`MountReadOnlyFailure`]。
 pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, MountReadOnlyFailure> {
-    let system_configuration_choice =
+    mount_read_only_from_the_system_configuration_choice(
+        reader,
         choose_system_configuration_naming_devices_without_a_mountable_slot(reader)
-            .map_err(MountReadOnlyFailure::Recovery)?;
+            .map_err(MountReadOnlyFailure::Recovery)?,
+    )
+}
+
+/// 带本池 fsid 的只读挂载：与 [`mount_read_only`] 同一条路径，系统配置只从自述 fsid 是 `filesystem_identifier_of_the_pool` 的槽里择
+/// （`recovery::choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot`，与可写挂载同一个筛法）：
+/// 别的池的槽世代号再大也择不到，一块盘两槽都是别的池的就跳过这块盘。一个字节都不写。
+///
+/// # Errors
+/// 见 [`MountReadOnlyFailure`]；盘上没有一槽是这个 fsid 的池的报 `Recovery(RecoveryFailure::NoValidSystemConfiguration)`，不读别的池。
+pub fn mount_read_only_of_the_pool(
+    reader: &dyn PoolReader,
+    filesystem_identifier_of_the_pool: &[u8; 16],
+) -> Result<MountedReadOnly, MountReadOnlyFailure> {
+    mount_read_only_from_the_system_configuration_choice(
+        reader,
+        choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
+            reader,
+            filesystem_identifier_of_the_pool,
+        )
+        .map_err(MountReadOnlyFailure::Recovery)?,
+    )
+}
+
+/// [`mount_read_only`] 与 [`mount_read_only_of_the_pool`] 共用的路径：系统配置已经择过。
+fn mount_read_only_from_the_system_configuration_choice(
+    reader: &dyn PoolReader,
+    system_configuration_choice: crate::recovery::SystemConfigurationChoice,
+) -> Result<MountedReadOnly, MountReadOnlyFailure> {
     let system_configuration = system_configuration_choice.system_configuration;
     let chosen_root = choose_root(reader, &system_configuration)
         .ok_or(MountReadOnlyFailure::Recovery(RecoveryFailure::NoValidRoot))?;
diff --git a/crates/singlefs-core/src/system_configuration_repair.rs b/crates/singlefs-core/src/system_configuration_repair.rs
index c84a892e37e4dc3b693332abc13d88b868c43cbd..7acdd5b3dce255fec13940da309fdbab49ad4674 100644
--- a/crates/singlefs-core/src/system_configuration_repair.rs
+++ b/crates/singlefs-core/src/system_configuration_repair.rs
@@ -13,7 +13,9 @@ use singlefs_format::{
 use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
 use crate::block_device::{BlockDevice, BlockDeviceError};
 use crate::journal::JournalRecordPlaceInPublish;
-use crate::make_filesystem::MakeFilesystemParameters;
+use crate::make_filesystem::{
+    MakeFilesystemParameters, GENESIS_CHECKPOINT_TXG, MKFS_INSTANCE_GENERATION,
+};
 use crate::recovery::{
     choose_system_configuration_of_the_pool, every_root_ring_slot, highest_root_instance,
     read_root_ring_slot, scan_journal, unit_area_start_of_the_chosen_system_configuration,
@@ -25,6 +27,10 @@ use crate::unit::{
     parse_data_unit, parse_index_node, parse_packed_unit, unit_filesystem_identifier,
 };
 
+/// 第 0 代那一版的 journal tail：mkfs 不写 journal 记录，写进系统配置的 tail 就是这个值（`make_filesystem` 写系统配置槽那一处）。
+/// 断言第 0 代时离线出路写它：第 0 代没有末条记录，这个值不靠记录得来，写出来与 mkfs 同值。
+const JOURNAL_TAIL_OF_THE_GENESIS_VERSION: u64 = 0;
+
 /// 一块盘单元区里解得开头、fsid 是本池的单元，写序实例代号与 `birth_txg` 各自的最大值（[`write_order_and_birth_in_the_unit_area_of_the_device`]）。
 /// 一个都没有时两项都是 `None`。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -141,8 +147,8 @@ pub enum SystemConfigurationRepairRefused {
     /// 断言那一版的根在根环里读不出、它那次发布带末条标志的记录在 journal 环里也读不出（或读出的不是它）：查不到就一个字节不写
     /// （「查断言那一版的根或记录读得出、且是它」）。
     AssertedVersionReadableNeitherInTheRootRingNorInTheJournalRing { asserted: AssertedVersion },
-    /// 断言那一版的根读得出、带末条标志的记录读不出：写进系统配置的 tail 该取什么，条款没写（α' 取那一版末条记录的计数器，
-    /// 末条读不出时没有这个数），第一版不支持，一个字节不写。
+    /// 断言那一版不是第 0 代，根读得出、带末条标志的记录读不出：写进系统配置的 tail 该取什么，条款没写（α' 取那一版末条记录的计数器，
+    /// 末条读不出时没有这个数），第一版不支持，一个字节不写。第 0 代不走这一支：mkfs 不写记录，tail 取 mkfs 写的值。
     TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion {
         asserted: AssertedVersion,
     },
@@ -213,7 +219,8 @@ fn slots_of_one_device_for_the_rewrite<Reader: PoolReader + ?Sized>(
 ///
 /// 1. 按调用方的本池 fsid 择系统配置（拿几何）。
 /// 2. 断言那一版的根在根环里读得出且是它，或它那次发布带末条标志的记录在 journal 环里读得出且是它；两样都没有一个字节不写。
-///    tail 取那条末条记录的计数器：根读得出而末条读不出时条款没写取什么，一个字节不写。
+///    tail 取那条末条记录的计数器；断言第 0 代（实例代号 [`MKFS_INSTANCE_GENERATION`]、txg [`GENESIS_CHECKPOINT_TXG`]）时根读得出就取 mkfs 写的值
+///    （[`JOURNAL_TAIL_OF_THE_GENESIS_VERSION`]）；别的版本根读得出而末条读不出时条款没写取什么，一个字节不写。
 /// 3. 扫每块盘整个单元区（[`write_order_and_birth_in_the_unit_area_of_the_device`]）：`birth_txg` 最大值大于断言那一版就一个字节不写。
 /// 4. 写的实例代号取各盘自证槽、根环、journal 环与单元写序里的最大值（不加一）。
 /// 5. 只重写不自证的槽（读不出、自证不过、别的池的）：逐盘按世代号写（`transaction::PoolWriter` 的系统配置槽写，世代号逐盘 +1、F 带整池生效值），
@@ -261,11 +268,16 @@ pub fn rewrite_system_configuration_slots_for_the_asserted_version<Device: Block
                 && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
         })
         .max_by_key(|record| record.counter);
+    let asserted_version_is_the_genesis_version = asserted.instance == MKFS_INSTANCE_GENERATION
+        && asserted.checkpoint_txg == GENESIS_CHECKPOINT_TXG;
     let journal_tail_written = match (
         root_of_the_asserted_version_is_readable,
         last_record_of_the_asserted_version,
     ) {
         (_, Some(last_record)) => last_record.counter,
+        (true, None) if asserted_version_is_the_genesis_version => {
+            JOURNAL_TAIL_OF_THE_GENESIS_VERSION
+        }
         (true, None) => {
             return Err(
                 SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion {
```

## 补丁 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-fixes-2/patch/crates.patch`（只摘 crates/singlefs-core/src/ 下的文件）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 5171f20776617d510413aa8e20e047e4e894f0e0..2db019d691aa35840a9d903daffabe21a4c14510 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1471,8 +1471,9 @@ struct InstanceStart {
 }
 
 /// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
-/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。这一遍直接读根环，`chosen_root`（读阶段择到的那条根）再取一次 max：两次读环之间的瞬时读错
-/// （择根那一遍读到了它、这一遍没读到）不把下界拉到它之下。所选根施加前缀之后的那一版不另收：它的 txg 来自 `records` 里的记录，已在记录那一半里。
+/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。这一遍直接读根环，`chosen_root`（读阶段择到的那条根，判退之前的那一条：择根取环里 txg 最大的，
+/// 读阶段读到的根都不超过它）再取一次 max：两次读环之间的瞬时读错（择根那一遍读到了它、这一遍没读到）不把下界拉到它之下，判退退掉的根同样算。
+/// 所选根施加前缀之后的那一版不另收：它的 txg 来自 `records` 里的记录，已在记录那一半里。
 ///
 /// # Errors
 /// 那个最大值已是 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条：盘上读来的 txg，不 panic）。
@@ -9773,6 +9774,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     // 判完不留字节，经它就把同一份坏副本多数几遍。
     let devices_counting_unit_reads_with_one_copy_failing =
         PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
+    // 判退之后 `chosen_root` 换成退到的那一条；新实例第一个 txg 的下界要盖住读阶段见过的根，退掉的那一条也算（`first_txg_of_new_instance`）。
+    let root_chosen_by_the_read_stage = chosen_root;
     let fall_back_rebuilt = match fall_back_judged_on_the_three_readers(
         &*devices,
         &system_configuration,
@@ -9873,8 +9876,12 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         .max()
         .unwrap_or(0)
         + 1;
-    let first_txg =
-        first_txg_of_new_instance(devices, &system_configuration, &records, &chosen_root)?;
+    let first_txg = first_txg_of_new_instance(
+        devices,
+        &system_configuration,
+        &records,
+        &root_chosen_by_the_read_stage,
+    )?;
     let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
         devices,
         &system_configuration,
diff --git a/crates/singlefs-core/src/system_configuration_repair.rs b/crates/singlefs-core/src/system_configuration_repair.rs
index 7acdd5b3dce255fec13940da309fdbab49ad4674..657e01d65dfdadf4f3a123587ccf972e000951f7 100644
--- a/crates/singlefs-core/src/system_configuration_repair.rs
+++ b/crates/singlefs-core/src/system_configuration_repair.rs
@@ -18,8 +18,9 @@ use crate::make_filesystem::{
 };
 use crate::recovery::{
     choose_system_configuration_of_the_pool, every_root_ring_slot, highest_root_instance,
-    read_root_ring_slot, scan_journal, unit_area_start_of_the_chosen_system_configuration,
-    verified_system_configuration_slots, PoolReader, RecoveryFailure, RootRingSlotReading,
+    highest_root_txg, read_root_ring_slot, scan_journal,
+    unit_area_start_of_the_chosen_system_configuration, verified_system_configuration_slots,
+    PoolReader, RecoveryFailure, RootRingSlotReading,
 };
 use crate::system_configuration::SystemConfiguration;
 use crate::transaction::{CommitStep, PoolWriter};
@@ -147,8 +148,9 @@ pub enum SystemConfigurationRepairRefused {
     /// 断言那一版的根在根环里读不出、它那次发布带末条标志的记录在 journal 环里也读不出（或读出的不是它）：查不到就一个字节不写
     /// （「查断言那一版的根或记录读得出、且是它」）。
     AssertedVersionReadableNeitherInTheRootRingNorInTheJournalRing { asserted: AssertedVersion },
-    /// 断言那一版不是第 0 代，根读得出、带末条标志的记录读不出：写进系统配置的 tail 该取什么，条款没写（α' 取那一版末条记录的计数器，
-    /// 末条读不出时没有这个数），第一版不支持，一个字节不写。第 0 代不走这一支：mkfs 不写记录，tail 取 mkfs 写的值。
+    /// 根读得出、带末条标志的记录读不出：写进系统配置的 tail 该取什么，条款没写（α' 取那一版末条记录的计数器，末条读不出时没有这个数），
+    /// 第一版不支持，一个字节不写。两形走这一支：断言那一版不是第 0 代；断言第 0 代而根环或 journal 环里读得到比第 0 代新的东西
+    /// （写行与暖机的根、它们的记录，而它们写下的单元都读不出、birth_txg 闸数不到）。第 0 代之后什么都读不到的不走这一支：mkfs 不写记录，tail 取 mkfs 写的值。
     TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion {
         asserted: AssertedVersion,
     },
@@ -222,6 +224,7 @@ fn slots_of_one_device_for_the_rewrite<Reader: PoolReader + ?Sized>(
 ///    tail 取那条末条记录的计数器；断言第 0 代（实例代号 [`MKFS_INSTANCE_GENERATION`]、txg [`GENESIS_CHECKPOINT_TXG`]）时根读得出就取 mkfs 写的值
 ///    （[`JOURNAL_TAIL_OF_THE_GENESIS_VERSION`]）；别的版本根读得出而末条读不出时条款没写取什么，一个字节不写。
 /// 3. 扫每块盘整个单元区（[`write_order_and_birth_in_the_unit_area_of_the_device`]）：`birth_txg` 最大值大于断言那一版就一个字节不写。
+///    断言第 0 代还要根环与 journal 环里都读不到比第 0 代新的东西，读得到就一个字节不写：`birth_txg` 闸数不到读不出的单元，闸拦不住那一形。
 /// 4. 写的实例代号取各盘自证槽、根环、journal 环与单元写序里的最大值（不加一）。
 /// 5. 只重写不自证的槽（读不出、自证不过、别的池的）：逐盘按世代号写（`transaction::PoolWriter` 的系统配置槽写，世代号逐盘 +1、F 带整池生效值），
 ///    写完一道池屏障。每个槽都自证时一个字节不写，交回空表。
@@ -319,6 +322,23 @@ pub fn rewrite_system_configuration_slots_for_the_asserted_version<Device: Block
             highest_write_order_instance_in_the_unit_areas
                 .max(unit_area.highest_write_order_instance);
     }
+    // birth_txg 闸数不到读不出的单元：写行与暖机的单元都读不出时闸放行，而环里的根与记录说第 0 代之后写过东西，tail 取 mkfs 的值就不是这个盘面的真话。
+    let something_newer_than_the_genesis_is_readable_in_the_rings = !records.is_empty()
+        || highest_root_txg(
+            &*devices,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+        ) != Some(GENESIS_CHECKPOINT_TXG);
+    if asserted_version_is_the_genesis_version
+        && something_newer_than_the_genesis_is_readable_in_the_rings
+    {
+        return Err(
+            SystemConfigurationRepairRefused::TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion {
+                asserted,
+            },
+        );
+    }
     let spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
     let highest_instance_in_the_self_verified_slots = devices
         .device_identities()
```

## 补丁 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-fixes-3/patch/crates.patch`（只摘 crates/singlefs-core/src/ 下的文件）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 2db019d691aa35840a9d903daffabe21a4c14510..68f89e4d555374de58ae96a534002142c9f8dc58 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1512,14 +1512,16 @@ fn checkpoint_txg_after_the_version(version: CheckpointTxg) -> Result<Checkpoint
 }
 
 /// 本实例第一次发布要带的树 ID 水位（D8（核心索引结构） 已定项 8 ②「根环里全部根记录该字段的 max」）：
-/// 根环里全部自证过的根与 `records`（环里全部自证通过的记录）各自带的水位取 max，再与这次要接在后面的那一版自己带的取 max——
-/// 那一版是从环里读出来的根或从环里的记录施加出来的，按构造已经在内；再取一次是为了两次读环之间的瞬时读错
-/// （择根那一遍读到了它、这一遍没读到）不把水位拉低。读不出的根怎么算见 `highest_tree_identifier_watermark_in_the_ring`。
+/// 根环里全部自证过的根与 `records`（环里全部自证通过的记录）各自带的水位取 max，再与这次要接在后面的那一版、读阶段择到的那条根
+/// （判退之前的那一条）各自带的取 max——这两条都是这次挂载从环里读到过的；再取一次是为了两次读环之间的瞬时读错
+/// （之前那一遍读到了它、这一遍没读到）不把水位拉低，判退退掉的根发过的号同样不许重发（`first_txg_of_new_instance` 同形）。
+/// 读不出的根怎么算见 `highest_tree_identifier_watermark_in_the_ring`。
 fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
     devices: &[(DeviceIdentity, Device)],
     system_configuration: &crate::system_configuration::SystemConfiguration,
     records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
     version_to_build_on: &RootRecord,
+    root_chosen_by_the_read_stage: &RootRecord,
 ) -> u64 {
     highest_tree_identifier_watermark_in_the_ring(
         devices,
@@ -1531,6 +1533,7 @@ fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
     .map_or(version_to_build_on.tree_identifier_watermark, |ring| {
         ring.max(version_to_build_on.tree_identifier_watermark)
     })
+    .max(root_chosen_by_the_read_stage.tree_identifier_watermark)
 }
 
 /// 进程内重读一个挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
@@ -9774,7 +9777,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     // 判完不留字节，经它就把同一份坏副本多数几遍。
     let devices_counting_unit_reads_with_one_copy_failing =
         PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
-    // 判退之后 `chosen_root` 换成退到的那一条；新实例第一个 txg 的下界要盖住读阶段见过的根，退掉的那一条也算（`first_txg_of_new_instance`）。
+    // 判退之后 `chosen_root` 换成退到的那一条；新实例第一个 txg 的下界与树 ID 水位都要盖住读阶段见过的根，退掉的那一条也算
+    // （`first_txg_of_new_instance`、`tree_identifier_watermark_of_the_ring`）。
     let root_chosen_by_the_read_stage = chosen_root;
     let fall_back_rebuilt = match fall_back_judged_on_the_three_readers(
         &*devices,
@@ -9887,6 +9891,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         &system_configuration,
         &records,
         &effective_root,
+        &root_chosen_by_the_read_stage,
     );
     let rebuilt = rebuilt_allocator(
         devices,
```

## 补丁 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-fallback-i74/patch/crates.patch`（只摘 crates/singlefs-core/src/ 下的文件）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index c294cdc..f672fa6 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -2195,7 +2195,7 @@ fn placements_in_the_accounts_of_unapplied_publishes<Reader: PoolReader>(
 /// 一条记录都读不出的，映射现算得出就交；现算不成时，那条根的树表与中央映射树根两个指针里有一个两份都已被本池更新实例的单元盖掉
 /// （[`one_pointer_has_both_copies_overwritten_by_a_later_instance`]，单元头里的 fsid 与 `filesystem_identifier_in_unit_headers` 相同才算）
 /// 就不交也不拒，否则照记录不齐那一支办（仍不成拒；函数体里那一支的注释）。
-/// `roots` 是重建分配器读根环那一遍读出的根（影子账、根环表用的同一份）；`filesystem_identifier_in_unit_headers` 是择到的那份系统配置里
+/// `roots` 是重建分配器读根环那一遍读出的根并上读阶段择到的那条根（[`roots_of_the_rebuild_pass_with_the_root_chosen_by_the_read_stage`]）；`filesystem_identifier_in_unit_headers` 是择到的那份系统配置里
 /// fsid 的低 8 字节（单元头里记的那一截，[`crate::unit::unit_filesystem_identifier`]），不另读盘。
 ///
 /// # Errors
@@ -3062,7 +3062,8 @@ fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
 /// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
 /// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
 /// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
-/// C 读一次没施加的发布或一条较新可读根的账读不出、那次发布读得出的记录不齐（较新可读根那次发布一条记录都读不出、树表与中央映射树根
+/// C 读一次没施加的发布或一条较新可读根（读阶段择到的那条根也算：读根环那一遍读它的根槽读坏时照读阶段读到的那一份）的账读不出、
+/// 那次发布读得出的记录不齐（较新可读根那次发布一条记录都读不出、树表与中央映射树根
 /// 两个指针都没有两份都是本池更新实例的单元的，也算这一支；别的池的单元头不算）、从中央映射也现算不成、重读一次仍不成
 /// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒
 /// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`。
@@ -3073,7 +3074,7 @@ fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
 )]
 #[allow(
     clippy::too_many_arguments,
-    reason = "每个参数各是一样东西：设备表、系统配置、上一版、新实例第一次发布的 txg、读阶段扫出的记录、所选那一版、影子账开关、重读前的钩子"
+    reason = "每个参数各是一样东西：设备表、系统配置、上一版、新实例第一次发布的 txg、读阶段扫出的记录、所选那一版、读阶段择到的根、影子账开关、重读前的钩子"
 )]
 fn rebuilt_allocator<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -3082,6 +3083,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
     first_txg: CheckpointTxg,
     records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
     selected_version: &RootRecord,
+    root_chosen_by_the_read_stage: &RootRecord,
     shadow_ledger: ShadowLedger,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<RebuiltAllocator, MountError> {
@@ -3133,6 +3135,13 @@ fn rebuilt_allocator<Device: BlockDevice>(
     );
     // C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条）：水位之上没施加的记录点名的落点与影子账同一次隔离、
     // 豁免同一个集合。影子账关着（只供测试）时它也不隔离：那一臂要的是「什么都不护」的对照。
+    // C 并的较新可读根取这一遍读出的根并上读阶段择到的那条（判退退掉的根）：这一遍读它的根槽读坏时它仍在回退候选集里，
+    // 漏了它，它的单元不进 C、写行那次发布照样写掉（I-7.4，[`roots_of_the_rebuild_pass_with_the_root_chosen_by_the_read_stage`]）。
+    let roots_for_the_newer_readable_roots =
+        roots_of_the_rebuild_pass_with_the_root_chosen_by_the_read_stage(
+            &sorted.roots,
+            root_chosen_by_the_read_stage,
+        );
     let placements_named_by_unapplied_journal_records = match shadow_ledger {
         ShadowLedger::On => placements_of_unapplied_publishes_and_newer_readable_roots(
             &devices_counting_unit_reads_with_one_copy_failing,
@@ -3140,7 +3149,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
             unit_area_start,
             records,
             selected_version,
-            &sorted.roots,
+            &roots_for_the_newer_readable_roots,
             &is_abandoned,
             &current_records,
             before_the_one_reread,
@@ -3520,6 +3529,30 @@ fn reclaim_the_released_slots_of_the_rebuilt_allocator<Device: BlockDevice>(
     );
 }
 
+/// C 并的较新可读根（`C_NEWER_ROOTS`，[`placements_in_the_accounts_of_newer_readable_roots`]）从哪几条根里挑：重建分配器读根环那一遍读出的根
+/// `roots_of_the_rebuild_pass`，并上读阶段择到的那条根 `root_chosen_by_the_read_stage`（判退之前的那一条），这一遍里已有同一 (实例代号, checkpoint_txg) 的就不再并。
+/// 判退那一形里读阶段择到的根 R 读得出、走读失败、退到 P：R 比 P 新、按实例表有效，在 I-7.4 的回退候选集里；这一遍读 R 的根槽读坏（一次瞬时读坏就够）时
+/// 这一遍的读数里没有 R，不并它 C 就看不见 R，R 的记录又读不出时它的单元一个都不护，写行那次发布照样写掉。并进来的是读阶段读到、自证过的那一份根记录，
+/// 它的账照 C 那一份读法读：读不出、记录不齐、现算不成、重读一次仍不成又没有凭据就拒（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]），
+/// 与这一遍读得出 R 时同一个结局。比所选那一版旧（没判退）或按实例表被抛弃的，由那个函数自己的筛滤掉。只拷内存，不读盘。
+/// 影子账、根环表与回收门槛不用这一份：它们说的是这一遍根环槽里读到了什么。
+fn roots_of_the_rebuild_pass_with_the_root_chosen_by_the_read_stage(
+    roots_of_the_rebuild_pass: &[RootRecord],
+    root_chosen_by_the_read_stage: &RootRecord,
+) -> Vec<RootRecord> {
+    let mut roots = roots_of_the_rebuild_pass.to_vec();
+    let key_of_the_root_chosen_by_the_read_stage = (
+        root_chosen_by_the_read_stage.instance,
+        root_chosen_by_the_read_stage.checkpoint_txg,
+    );
+    if !roots.iter().any(|root| {
+        (root.instance, root.checkpoint_txg) == key_of_the_root_chosen_by_the_read_stage
+    }) {
+        roots.push(*root_chosen_by_the_read_stage);
+    }
+    roots
+}
+
 /// 影子账同一次隔离、豁免同一个集合的那几份落点，按根环槽归组（C694 的 C）：水位之上没施加的记录点名的落点
 /// （[`placements_named_by_unapplied_journal_records`]），并上那几次没施加的发布的账里的落点（C 的账那一份，
 /// [`placements_in_the_accounts_of_unapplied_publishes`]；账读不出、读得出的记录不齐时从中央映射现算，仍不成就拒），
@@ -9864,8 +9897,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     // 判完不留字节，经它就把同一份坏副本多数几遍。
     let devices_counting_unit_reads_with_one_copy_failing =
         PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
-    // 判退之后 `chosen_root` 换成退到的那一条；新实例第一个 txg 的下界与树 ID 水位都要盖住读阶段见过的根，退掉的那一条也算
-    // （`first_txg_of_new_instance`、`tree_identifier_watermark_of_the_ring`）。
+    // 判退之后 `chosen_root` 换成退到的那一条；新实例第一个 txg 的下界、树 ID 水位与 C 并的较新可读根都要盖住读阶段见过的根，退掉的那一条也算
+    // （`first_txg_of_new_instance`、`tree_identifier_watermark_of_the_ring`、`rebuilt_allocator`）。
     let root_chosen_by_the_read_stage = chosen_root;
     let fall_back_rebuilt = match fall_back_judged_on_the_three_readers(
         &*devices,
@@ -9987,6 +10020,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         first_txg,
         &records,
         &effective_root,
+        &root_chosen_by_the_read_stage,
         shadow_ledger,
         &mut before_the_one_reread,
     );
```

## 补丁 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-read-stage-roots/patch/crates.patch`（只摘 crates/singlefs-core/src/ 下的文件）

```diff
```

## 补丁 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-instance-table-read-stage/patch/crates.patch`（只摘 crates/singlefs-core/src/ 下的文件）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 241f479..6c1c643 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -60,8 +60,8 @@ use crate::recovery::{
 };
 use crate::recovery::{
     allocation_records_of_version_without_file_in_the_unit_area_starting_at,
-    allocation_records_under_root_in_the_unit_area_starting_at, choose_root,
-    choose_system_configuration, choose_system_configuration_of_the_pool,
+    allocation_records_under_root_in_the_unit_area_starting_at, choose_system_configuration,
+    choose_system_configuration_of_the_pool,
     effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once,
     effective_rollback_floor_rereading_the_newest_instance_table_once,
     effective_rollback_floor_under_the_newest_roots_table, fall_back_judged_on_the_three_readers,
@@ -409,10 +409,10 @@ pub enum StillUnreadableAfterOneReread {
         first_read: SelectedVersionAgainstTheWitness,
         reread: SelectedVersionAgainstTheWitness,
     },
-    /// 代码审阅第 22 条：重建分配器时按最新那条根的实例表判哪几条根被抛弃（影子账、回收的门槛、根环表都按它），那张表读不出，
-    /// 重读一次仍读不出。不再按「没有一条根被抛弃」往下走（那样一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0）。
+    /// 代码审阅第 22 条：重建分配器时按读阶段择到的那条根（根环里最新的那条）的实例表判哪几条根被抛弃（影子账、回收的门槛、根环表都按它），
+    /// 那张表读不出，重读一次仍读不出。不再按「没有一条根被抛弃」往下走（那样一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0）。
     InstanceTableOfTheNewestRootForTheShadowLedger {
-        /// 重读那一遍择到的最新那条根；根环那一遍一条自证过的根都没读到时 `None`。
+        /// 读不出的是哪一条根的实例表：可写挂载交回的恒是 `Some`（读阶段择到的那条根，重读读的也是它）。
         newest_root_on_the_reread: Option<RollbackTarget>,
     },
     /// 挂着时抬 F 算 F 生效值（`recovery::effective_rollback_floor_rereading_the_newest_instance_table_once`）或管理员回退判候选算
@@ -571,7 +571,7 @@ pub enum WitnessedCounterComparison {
 /// 产品路径立即重读，两次读之间不等、不另设间隔。只供测试的那一臂在重读之前调用例给的钩子
 /// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：用例在钩子里撤掉暂时的读故障，强制进入「重读读得出」那一支；
 /// 或在钩子里让重读那一遍也读坏，强制进入「重读仍读不出」那一支）。
-/// 每处「重读一次」之前各调一次：读阶段那一次（判据 N-配置），重建分配器读最新那条根的实例表那一次（代码审阅第 22 条），
+/// 每处「重读一次」之前各调一次：读阶段那一次（判据 N-配置），重建分配器读读阶段择到的那条根的实例表那一次（代码审阅第 22 条），
 /// 重建分配器读根环有一块盘一条不被抛弃的自证有效根都没读出、整环重读那一次（用户 2026-10-05 定「重读」与 Q4 rr2），影子账里一条被抛弃根的账与从映射现算都不成、
 /// 重读它那一次（C393，每条这样的根各一次）。
 pub enum BeforeTheOneReread<'hook> {
@@ -610,7 +610,7 @@ pub enum ReadStageSettled {
     },
 }
 
-/// 重建分配器时最新那条根的实例表在哪一遍读出来（代码审阅第 22 条）。
+/// 重建分配器时读阶段择到的那条根的实例表在哪一遍读出来（代码审阅第 22 条）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum InstanceTableOfTheNewestRootRead {
     OnTheFirstRead,
@@ -1079,7 +1079,7 @@ pub struct MountOutput {
     pub space_admission: MountSpaceAdmission,
     /// 这次挂载认下的池（盘上那份参数、核过的盘表）：挂着的会话只用它。
     pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
-    /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、最新那条根的实例表在哪一遍读出来、重建分配器读根环在哪一遍读出有效根。
+    /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、读阶段择到的那条根的实例表在哪一遍读出来、重建分配器读根环在哪一遍读出有效根。
     pub rereads: RereadsOfThisMount,
     /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）发成的每一步，按先后：接在写行、暖机与推的抬 F 之后，走到了空时最后一个就是
     /// `Mounted::current`（两处指着同一份，不拷一整版；写行那一版与暖机各版里只有最后发成的那一版与 `Mounted::current` 共享，前面几版各是一份）；
@@ -1627,33 +1627,31 @@ struct RebuiltAllocator {
     root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead,
 }
 
-/// 重建分配器判哪几条根被抛弃用的那张表：最新那条根（`choose_root` 现择）指着的实例表（代码审阅第 22 条）。
-/// 这一次读不出（择不到根、或那张表沿链有一片读不出、解不开）就重读一次（D16（发布语义） 已定项 1「重读一次」；C554 乙，R = 1），
-/// 重读之前调 `before_the_one_reread`。
+/// 重建分配器判哪几条根被抛弃用的那张表：读阶段择到的那条根 `root_chosen_by_the_read_stage` 指着的实例表（代码审阅第 22 条）。
+/// 判退那一形（读阶段择到的根走读失败、退到更旧的一版）照样用它：判退本身按所选根的实例表滤（`recovery::fall_back_judged_on_the_three_readers`），
+/// 前后同一张。读阶段之后不另择根：读阶段之后读坏它的根槽时现择会落到更旧的一条根上，那条根的表可能还没有抛弃它自己那条时间线的行。
+/// 这一次读不出（那张表沿链有一片读不出、解不开）就重读一次（D16（发布语义） 已定项 1「重读一次」；C554 乙，R = 1），重读之前调 `before_the_one_reread`。
 ///
 /// # Errors
-/// 重读仍读不出 ⇒ `NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)`：
+/// 重读仍读不出 ⇒ `NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)`，点名读阶段择到的那条根：
 /// 不按「没有一条根被抛弃」往下走（那样一个槽都不隔离）。可写挂载在取号之前调它，拒的时候盘上逐字节不变。
 fn instance_table_of_the_newest_root_read_at_most_twice(
     reader: &dyn PoolReader,
-    system_configuration: &crate::system_configuration::SystemConfiguration,
+    root_chosen_by_the_read_stage: &RootRecord,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<(InstanceTableRecords, InstanceTableOfTheNewestRootRead), MountError> {
-    if let Some(table) = choose_root(reader, system_configuration)
-        .and_then(|newest| instance_table_of_root(reader, &newest))
-    {
+    if let Some(table) = instance_table_of_root(reader, root_chosen_by_the_read_stage) {
         return Ok((table, InstanceTableOfTheNewestRootRead::OnTheFirstRead));
     }
     before_the_one_reread.before_rereading();
-    let newest_on_the_reread = choose_root(reader, system_configuration);
-    match newest_on_the_reread.and_then(|newest| instance_table_of_root(reader, &newest)) {
+    match instance_table_of_root(reader, root_chosen_by_the_read_stage) {
         Some(table) => Ok((table, InstanceTableOfTheNewestRootRead::OnTheOneReread)),
         None => Err(MountError::NewerStateStillUnreadableAfterOneReread(
             Box::new(
                 StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
-                    newest_root_on_the_reread: newest_on_the_reread.map(|newest| RollbackTarget {
-                        instance: newest.instance,
-                        checkpoint_txg: newest.checkpoint_txg,
+                    newest_root_on_the_reread: Some(RollbackTarget {
+                        instance: root_chosen_by_the_read_stage.instance,
+                        checkpoint_txg: root_chosen_by_the_read_stage.checkpoint_txg,
                     }),
                 },
             ),
@@ -3055,11 +3053,11 @@ fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
 /// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
 /// 影子账只住内存，所以每次挂载都要重算（步 4 / 步 5 代码三方第一轮云端攻方腿打中：普通重开一次隔离就归零）。
 /// 每块盘的空闲图从择到的系统配置按环长现算的单元区起点起（与偏移 417 那 8 字节相等，择的时候判过；C475（非默认环长下单元区起点取编译期常量））。
-/// 生效的回退下界 F 判「有效根」用的也是影子账那一张表（最新那条根的实例表，读不出重读一次）：不另读一遍、也就没有「读不出就不按表滤」
-/// 那一支（C554 乙报告 Q6）。
+/// 生效的回退下界 F 判「有效根」用的也是影子账那一张表（读阶段择到的那条根 `root_chosen_by_the_read_stage` 的实例表，读不出重读一次）：
+/// 不另读一遍、也就没有「读不出就不按表滤」那一支（C554 乙报告 Q6）。
 ///
 /// # Errors
-/// 最新那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
+/// 读阶段择到的那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
 /// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
 /// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
 /// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
@@ -3071,11 +3069,11 @@ fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
 /// 都在取号之前，盘上逐字节不变。
 #[allow(
     clippy::ptr_arg,
-    reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+    reason = "读盘的几处走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
 )]
 #[allow(
     clippy::too_many_arguments,
-    reason = "每个参数各是一样东西：设备表、系统配置、上一版、新实例第一次发布的 txg、读阶段扫出的记录、所选那一版、读阶段读到的根、影子账开关、重读前的钩子"
+    reason = "每个参数各是一样东西：设备表、系统配置、上一版、新实例第一次发布的 txg、读阶段扫出的记录、所选那一版、读阶段择到的根、读阶段读到的根、影子账开关、重读前的钩子"
 )]
 fn rebuilt_allocator<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -3084,13 +3082,14 @@ fn rebuilt_allocator<Device: BlockDevice>(
     first_txg: CheckpointTxg,
     records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
     selected_version: &RootRecord,
+    root_chosen_by_the_read_stage: &RootRecord,
     roots_read_by_the_read_stage: &[RootRecord],
     shadow_ledger: ShadowLedger,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<RebuiltAllocator, MountError> {
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(system_configuration);
     let mut allocator = allocator_of_the_previous_version(devices, unit_area_start, previous)?;
-    // 读最新那条根的实例表与读各根的账（候选根、被抛弃根、没施加的发布、较新可读根、回退目标）读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：
+    // 读读阶段择到的那条根的实例表与读各根的账（候选根、被抛弃根、没施加的发布、较新可读根、回退目标）读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：
     // 同一个单元建上一版已经读到的归挂载那个数，不再算；读的是别的版本的，下一次用户改动之后那一批认不出、只留下计数。
     let devices_counting_unit_reads_with_one_copy_failing =
         PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
@@ -3098,7 +3097,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
     let (newest_table, instance_table_of_the_newest_root) =
         instance_table_of_the_newest_root_read_at_most_twice(
             &devices_counting_unit_reads_with_one_copy_failing,
-            system_configuration,
+            root_chosen_by_the_read_stage,
             before_the_one_reread,
         )?;
     let effective_floor = effective_rollback_floor_under_the_newest_roots_table(
@@ -9911,7 +9910,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
     // 判退之后 `chosen_root` 换成退到的那一条；新实例第一个 txg 的下界、树 ID 水位要盖住读阶段择到的根（`first_txg_of_new_instance`、
     // `tree_identifier_watermark_of_the_ring`：择根取环里最新的，读阶段读到的根都不超过它），C 并的较新可读根要盖住读阶段读到的全部根
-    // （`rebuilt_allocator`：判退退掉的那一条与它和退到的那一版之间读阶段读得出的那几条，之后几遍读它们的根槽读坏时照样在回退候选集里）。
+    // （`rebuilt_allocator`：判退退掉的那一条与它和退到的那一版之间读阶段读得出的那几条，之后几遍读它们的根槽读坏时照样在回退候选集里），
+    // 判抛弃用读阶段择到的那条根的实例表（[`instance_table_of_the_newest_root_read_at_most_twice`]：判退之后也用它，与判退本身滤候选用的同一张）。
     let root_chosen_by_the_read_stage = chosen_root;
     let fall_back_rebuilt = match fall_back_judged_on_the_three_readers(
         &*devices,
@@ -10033,6 +10033,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         first_txg,
         &records,
         &effective_root,
+        &root_chosen_by_the_read_stage,
         &roots_read_by_the_read_stage,
         shadow_ledger,
         &mut before_the_one_reread,
```

## 补丁 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-shadow-ledger-refusal-field/patch/crates.patch`（只摘 crates/singlefs-core/src/ 下的文件）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 6c1c643f6c141105828461fcdd289a53b77c57a7..5e54d4e46d9b64323a6656bd13a823990eedc179 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -410,10 +410,10 @@ pub enum StillUnreadableAfterOneReread {
         reread: SelectedVersionAgainstTheWitness,
     },
     /// 代码审阅第 22 条：重建分配器时按读阶段择到的那条根（根环里最新的那条）的实例表判哪几条根被抛弃（影子账、回收的门槛、根环表都按它），
-    /// 那张表读不出，重读一次仍读不出。不再按「没有一条根被抛弃」往下走（那样一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0）。
+    /// 那张表读不出，重读一次仍读不出。不按「没有一条根被抛弃」往下走（那样一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0）。
     InstanceTableOfTheNewestRootForTheShadowLedger {
-        /// 读不出的是哪一条根的实例表：可写挂载交回的恒是 `Some`（读阶段择到的那条根，重读读的也是它）。
-        newest_root_on_the_reread: Option<RollbackTarget>,
+        /// 读阶段择到的那条根（判退那一形也是它），按它的实例表判抛弃；两遍读的都是这一条根的表。
+        root_chosen_by_the_read_stage: RollbackTarget,
     },
     /// 挂着时抬 F 算 F 生效值（`recovery::effective_rollback_floor_rereading_the_newest_instance_table_once`）或管理员回退判候选算
     /// F_生效（`recovery::effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）：根环里最新那条根
@@ -1649,10 +1649,10 @@ fn instance_table_of_the_newest_root_read_at_most_twice(
         None => Err(MountError::NewerStateStillUnreadableAfterOneReread(
             Box::new(
                 StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
-                    newest_root_on_the_reread: Some(RollbackTarget {
+                    root_chosen_by_the_read_stage: RollbackTarget {
                         instance: root_chosen_by_the_read_stage.instance,
                         checkpoint_txg: root_chosen_by_the_read_stage.checkpoint_txg,
-                    }),
+                    },
                 },
             ),
         )),
```

