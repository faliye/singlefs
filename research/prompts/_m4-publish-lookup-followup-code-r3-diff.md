# m4-publish-lookup-followup-code-r3：第 17 批（七·6 后续批第二轮判决第三节 1–7 条与第五节用户定的两条）的补丁原文（crates.patch、mutations-append.tsv、mutations-delete.txt 原样）（2026-10-23）

补丁目录 `~/.local/share/singlefs/work/m4-publish-lookup-followup-3-writer/patch/`：`crates.patch` sha256 `fa00c0df54209d25…`、`mutations-append.tsv` sha256 `7a1d603f74fc081e…`、`mutations-delete.txt` sha256 `bb5e27d339c37762…`。

## crates.patch

````diff
diff --git a/crates/singlefs-core/build.rs b/crates/singlefs-core/build.rs
index 75ab318..764bccf 100644
--- a/crates/singlefs-core/build.rs
+++ b/crates/singlefs-core/build.rs
@@ -1,7 +1,8 @@
 //! 把这一次编译里 cargo 给本包打开的特性交给直接依赖本包的构建脚本：`links` 的元数据，依赖方读 `DEP_SINGLEFS_CORE_ENABLED_FEATURES`
 //! （`singlefs_unit_graph::build_features::features_enabled_on_the_dependencies`，键名是那里的 `ENABLED_FEATURES_METADATA_KEY`）。
 //! cargo 解出来的特性是并过之后的：别的包在同一次构建里替本包打开的特性（harness 档的 dev 依赖打开的 `lookup-work-counters`）
-//! 也在里面，依赖方自己的特性看不到它们。checker 档的构建脚本拿这一份算判法单元键。
+//! 也在里面，依赖方自己的特性看不到它们。checker 档的构建脚本拿这一份算判法单元键。同一个值另经 `cargo::rustc-env` 编进本包
+//! （`transaction::lookup_work::ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT`），测试拿它判「cargo 真开着时报开着」。
 //!
 //! 取法：cargo 编本包的构建脚本时带着本包这一次开了的特性（每个开着的特性一个 `--cfg feature="<名>"`，与编本包的库同一份），
 //! 所以 `cfg!(feature = "<名>")` 就是 cargo 这一次真开没开；特性一变，构建脚本按另一份 cfg 重编、重跑。不读进程环境里的
@@ -9,17 +10,33 @@
 //! 分不出来（七·6 后续批代码轮第一轮判决第二节 T4）。同仓 `Cargo.lock` 里的 `librocksdb-sys` 构建脚本按同一个取法读它的特性
 //! （`if cfg!(feature = "snappy")`）。
 //!
-//! 只做交出：不读别的输入、不生成代码，值是开着的特性名写进 cargo 环境变量名的样子（大写、`-` 换成 `_`），按清单里的名字排好、逗号隔开。
+//! 这个取法差一种编法（七·6 后续批代码轮第二轮判决第二节 T4 ①，用户 2026-10-23 定并进）：带 `--target` 时 `RUSTFLAGS` 里的
+//! `--cfg feature="<名>"` 只给编库的那一次、不给编构建脚本的那一次，库按开着编、这里的 cfg 却说没开。cargo 交给构建脚本的
+//! `CARGO_ENCODED_RUSTFLAGS` 是编库那一份参数（本机 cargo 1.98.0 带 `--target` 量过：构建脚本读到 `--cfg` 与 `feature="…"` 两项，
+//! 自己的 cfg 是假），所以那里点名打开、这里的 cfg 却说没开的特性，构建当场停下、说出是哪一个，不交一份与编出来的库不符的元数据。
+//!
+//! 只做交出与这一判：不生成代码，值是开着的特性名写进 cargo 环境变量名的样子（大写、`-` 换成 `_`），按清单里的名字排好、逗号隔开。
 //! 清单 `[features]` 里的每个特性都要在 [`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG`] 里有一行：`singlefs-unit-graph` 的
 //! `build_features` 单测把两边逐个比（`the_core_build_script_lists_every_feature_of_its_manifest`）。
 
 /// 本包清单 `[features]` 里的每个特性（清单里的写法，按名字排好）与 cargo 这一次开没开它。
-/// `pub(crate)`：harness 档的 `publish_lookup_by_identity` 经 `#[path]` 把这份构建脚本编成一个模块，要读它与下面那个函数。
+/// `pub(crate)`：harness 档的 `publish_lookup_by_identity` 与 `singlefs-unit-graph` 的单测经 `#[path]` 把这份构建脚本编成一个模块，
+/// 要读它与下面两个函数。
 pub(crate) const FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG: [(&str, bool); 1] = [(
     "lookup-work-counters",
     cfg!(feature = "lookup-work-counters"),
 )];
 
+/// cargo 交给构建脚本的、编本包的 rustc 额外参数：参数之间用 0x1f 隔开。
+const ENCODED_RUSTFLAGS_VARIABLE: &str = "CARGO_ENCODED_RUSTFLAGS";
+
+/// [`ENCODED_RUSTFLAGS_VARIABLE`] 里参数之间的分隔符（ASCII 单元分隔符）。
+const ENCODED_RUSTFLAGS_SEPARATOR: char = '\u{1f}';
+
+/// 交去编本包的那一行环境变量名（`cargo::rustc-env`）。
+const ENABLED_FEATURES_RUSTC_ENVIRONMENT_VARIABLE: &str =
+    "SINGLEFS_CORE_ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT";
+
 /// 交出去的那一行的值：`features_with_their_cfg` 里开着的那几个，写成 cargo 环境变量名的样子（大写、`-` 换成 `_`），逗号隔开。
 /// 只看 `features_with_their_cfg`，不读进程环境：外面带进来的 `CARGO_FEATURE_*` 不算开着。
 pub(crate) fn enabled_features_metadata_value(features_with_their_cfg: &[(&str, bool)]) -> String {
@@ -31,10 +48,78 @@ pub(crate) fn enabled_features_metadata_value(features_with_their_cfg: &[(&str,
         .join(",")
 }
 
+/// `encoded_rustflags`（[`ENCODED_RUSTFLAGS_VARIABLE`] 的值）里经 `--cfg feature="<名>"`（`--cfg` 与它的值分两项，或写成
+/// `--cfg=feature="<名>"` 一项；等号两边与引号里外的空白不算）点名打开、而 `features_with_their_cfg` 说这一次没开的那几个特性，
+/// 按表的次序。表里没有的名字不管（`RUSTFLAGS` 给每个包都带，别的包的特性也在里面）。
+pub(crate) fn features_opened_by_the_rustflags_but_off_in_this_build_script<'table>(
+    encoded_rustflags: &str,
+    features_with_their_cfg: &[(&'table str, bool)],
+) -> Vec<&'table str> {
+    let arguments: Vec<&str> = encoded_rustflags
+        .split(ENCODED_RUSTFLAGS_SEPARATOR)
+        .filter(|argument| !argument.is_empty())
+        .collect();
+    let mut feature_names_opened_by_the_rustflags: Vec<String> = Vec::new();
+    // 迭代上界是参数个数；跨轮携带的是下一项是不是 `--cfg` 的值（只看紧跟在 `--cfg` 后面的那一项）。
+    let mut is_the_value_of_a_cfg_flag = false;
+    for argument in arguments {
+        let cfg_value = if is_the_value_of_a_cfg_flag {
+            is_the_value_of_a_cfg_flag = false;
+            Some(argument)
+        } else if argument == "--cfg" {
+            is_the_value_of_a_cfg_flag = true;
+            None
+        } else {
+            argument.strip_prefix("--cfg=")
+        };
+        let Some(cfg_value) = cfg_value else {
+            continue;
+        };
+        let Some((name, value)) = cfg_value.split_once('=') else {
+            continue;
+        };
+        if name.trim() == "feature" {
+            feature_names_opened_by_the_rustflags.push(value.trim().trim_matches('"').to_string());
+        }
+    }
+    features_with_their_cfg
+        .iter()
+        .filter(|(feature, is_enabled_by_cargo)| {
+            !*is_enabled_by_cargo
+                && feature_names_opened_by_the_rustflags
+                    .iter()
+                    .any(|opened| opened.as_str() == *feature)
+        })
+        .map(|(feature, _)| *feature)
+        .collect()
+}
+
 fn main() {
+    // cargo 跑构建脚本时恒设它（没有额外参数时是空串）；不是 cargo 起的、没设的当作没有额外参数。
+    let encoded_rustflags = match std::env::var(ENCODED_RUSTFLAGS_VARIABLE) {
+        Ok(value) => value,
+        Err(std::env::VarError::NotPresent) => String::new(),
+        Err(std::env::VarError::NotUnicode(value)) => {
+            panic!("{ENCODED_RUSTFLAGS_VARIABLE} 不是 UTF-8：{value:?}")
+        }
+    };
+    let disagreeing = features_opened_by_the_rustflags_but_off_in_this_build_script(
+        &encoded_rustflags,
+        &FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG,
+    );
+    assert!(
+        disagreeing.is_empty(),
+        "singlefs-core：特性 {disagreeing:?} 由 {ENCODED_RUSTFLAGS_VARIABLE} 里的 --cfg feature=… 给编库的那一次打开了，编构建脚本的这一次却没开\
+         （带 --target 时 RUSTFLAGS 不给构建脚本）：照这里的 cfg 交出去的特性与编出来的库不符，依赖方的判法单元键会按没开算。\
+         出路：用 cargo 的 --features 打开它，不经 RUSTFLAGS 的 --cfg"
+    );
     println!(
         "cargo::metadata=enabled_features={}",
         enabled_features_metadata_value(&FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG)
     );
+    println!(
+        "cargo::rustc-env={ENABLED_FEATURES_RUSTC_ENVIRONMENT_VARIABLE}={}",
+        enabled_features_metadata_value(&FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG)
+    );
     println!("cargo::rerun-if-changed=build.rs");
 }
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 6eeacd7..01e88a4 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -3713,12 +3713,24 @@ fn data_units_judged_against_the_inode_records(
     extent_tree: TreeIdentifier,
     expected_filesystem_identifier: u64,
 ) -> Result<Option<InodeRecord>, RecoveryFailure> {
+    // 按 inode 号找记录走建一次的索引（七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）：第一个文件之外每个文件各找一次，
+    // 逐个在全部叶容器的记录上找是文件数的平方。索引按叶容器的次序、容器里记录的次序收，同一个号取最先那一条，
+    // 与在记录上 `find` 同一条（`FirstEntryOfEachIdentity`）。这一处的建与查都在进的那一处里，函数返回时退出。
+    let _inside_the_inode_records = crate::transaction::lookup_work::entering(
+        crate::transaction::lookup_work::LookupSite::InodeRecordsOfTheFilesJudgedByTheRecovery,
+    );
+    let inode_records_in_container_order: Vec<&InodeRecord> = inode_leaf_containers
+        .iter()
+        .flat_map(|container| container.contents.records.iter())
+        .collect();
+    let inode_records_by_number = crate::transaction::FirstEntryOfEachIdentity::of_the_table(
+        &inode_records_in_container_order,
+        |inode_record| inode_record.inode,
+    );
     let inode_record_of = |inode: u64| {
-        inode_leaf_containers
-            .iter()
-            .flat_map(|container| container.contents.records.iter())
-            .find(|inode_record| inode_record.inode == inode)
-            .copied()
+        inode_records_by_number
+            .first_entry_of(&inode)
+            .map(|inode_record| **inode_record)
     };
     let inode_record = inode_record_of(FIRST_INODE_NUMBER);
     match (&inode_record, first_file_extents.data_pointers.is_empty()) {
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 40c3f38..f7a4af9 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -3281,6 +3281,12 @@ pub mod lookup_work {
     #[cfg(feature = "lookup-work-counters")]
     use std::collections::BTreeMap;
 
+    /// 本包构建脚本这一次交给依赖方的特性：`build.rs` 打的 `cargo::metadata=enabled_features=` 那一行的值，同一个值经
+    /// `cargo::rustc-env` 编进本包。依赖方读到的就是它，测试拿它判「cargo 真开着这个特性时构建脚本报开着」
+    /// （七·6 后续批代码轮第二轮判决第三节第 7 条：构建脚本里那一格 cfg 写死成假时，依赖方当它没开，别的检查都看不见）。
+    pub const ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT: &str =
+        env!("SINGLEFS_CORE_ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT");
+
     /// 七·6 按身份查的几处里，测试要分开判的那几处。封闭集合；不在其中任何一处里做的查找记在「不在点名的几处里」那一格（`None`）。
     #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
     pub enum LookupSite {
@@ -3320,6 +3326,14 @@ pub mod lookup_work {
         /// 装一次发布的记录：点名的每个角色按角色取它在这一版 `units` 里的单元（`publish_admitted` 的 `named_unit_of`），
         /// 查的是 [`LookupSite::UnitsOfTheRolesWrittenByThisPublish`] 那里建的同一份索引。每点名一个角色进一次。
         UnitsOfTheRolesNamedByTheRecords,
+        /// 照抄第一个文件的发布装这一版 `units` 时，第一个文件的每个数据单元问它在不在这次重写的角色里（`publish_admitted`）：
+        /// 这次重写的角色清单建一次按角色的索引、每个数据单元查一次（七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）。
+        /// 每装一次照抄第一个文件的进一次；第一个文件有新版本的发布不进、不建。
+        RewrittenRolesAmongTheDataUnitsOfTheCarriedFirstFile,
+        /// 恢复重建一版时数据单元照 inode 记录核，按 inode 号找记录（`recovery::data_units_judged_against_the_inode_records`）：
+        /// 全部 inode 叶容器的记录建一次按 inode 号的索引，第一个文件与第一个文件之外每个有数据单元的文件各查一次
+        /// （七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）。每调一次进一次。
+        InodeRecordsOfTheFilesJudgedByTheRecovery,
         /// 发布照抄 extent 树的节点：按角色取上一版那个节点的单元（`build_extent_tree` 的 `carried_node`）。
         /// 整棵 extent 树装的时候都在这一处里（七·6 后续批代码轮第一轮判决第三节第 3 条：照抄那一处的作用域包住建），
         /// 照抄每个节点时再进一次；里面按 inode 号找文件的记在 [`LookupSite::FilesOfTheCarriedExtentLowerNodes`]。
@@ -14333,18 +14347,44 @@ fn publish_admitted<Device: BlockDevice>(
         identity,
         bytes,
     };
+    let data_role_at = |position: usize| {
+        TransactionUnit::Data(DataUnitIndexInFile(
+            u64::try_from(position).expect("单元序号"),
+        ))
+    };
+    // 照抄第一个文件的发布：第一个文件的每个数据单元问它在不在这次重写的角色里（截断与变长重写的尾单元在里面）。这次重写的角色清单
+    // 建一次按角色的索引、每个数据单元查一次（七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）：逐个在清单上找是第一个文件的
+    // 数据单元数乘这次重写的角色数，第一个文件占池的大头又整份重写一个大文件时两样都到池大小的量级。先问完再退出那一处：
+    // 下面照抄单元时按角色取上一版单元的那几次查不记在这里。第一个文件有新版本的发布一个都不问。
+    let is_each_data_unit_of_the_carried_first_file_rewritten: Vec<bool> = match carried_file {
+        None => Vec::new(),
+        Some(_) => {
+            let _inside_the_rewritten_roles = lookup_work::entering(
+                LookupSite::RewrittenRolesAmongTheDataUnitsOfTheCarriedFirstFile,
+            );
+            let rewritten_by_role =
+                FirstEntryOfEachIdentity::of_the_table(rewritten, |identity| *identity);
+            (0..data_units.len())
+                .map(|position| {
+                    rewritten_by_role
+                        .first_entry_of(&data_role_at(position))
+                        .is_some()
+                })
+                .collect()
+        }
+    };
     let mut units: Vec<PublishedUnit> = Vec::new();
     for (position, data_unit) in data_units.iter().enumerate() {
-        let identity = TransactionUnit::Data(DataUnitIndexInFile(
-            u64::try_from(position).expect("单元序号"),
-        ));
+        let identity = data_role_at(position);
         units.push(match carried_file {
-            Some(_) if !rewritten.contains(&identity) => carried_unit(
-                units_of_the_previous_version
-                    .as_ref()
-                    .expect("照抄第一个文件的发布接在上一版之后：carried_file 就是上一版"),
-                identity,
-            ),
+            Some(_) if !is_each_data_unit_of_the_carried_first_file_rewritten[position] => {
+                carried_unit(
+                    units_of_the_previous_version
+                        .as_ref()
+                        .expect("照抄第一个文件的发布接在上一版之后：carried_file 就是上一版"),
+                    identity,
+                )
+            }
             None | Some(_) => rewritten_unit(identity, data_unit.clone()),
         });
     }
diff --git a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
index a55ac06..7c2322d 100644
--- a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
+++ b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
@@ -22,6 +22,17 @@
 //! `units`、装一次发布要写与记录点名的单元、留给快照的角色清单两处、记录点名的数据角色那一处都经建一次的索引查；按 inode 号找文件、
 //! 删快照取留存条目那几处在发布路径上建索引的次数恰好等于走了几遍；三棵多层码 2 树照抄的节点按树分开数；`singlefs-core` 构建脚本
 //! 交出的特性只认 cargo 这一次真开的。
+//!
+//! 七·6 后续批代码轮第二轮判决（`research/prompts/m4-publish-lookup-followup-code-r2-main-verification.md` 第三节第 1–7 条、第五节用户定的三条）另钉：
+//! 恢复重建一版把映射位置换进 `units` 每次重挂建一份、查有位置的进映射角色那么多次；恢复按 inode 号找 inode 记录与照抄第一个文件时问数据单元
+//! 在不在这次重写的角色里各建一次索引；点名的几处之外（`None`）建的索引不随一次写的单元数涨；记账树照抄到不止一个节点时按树分开数；
+//! 一把有位置的 key 都没有就不建；构建脚本真开着时报开着，`RUSTFLAGS` 的 `--cfg feature=…` 与它自己的 cfg 不符时停下。
+//!
+//! 已知边界（判决第三节第 4 条，不判）：计数只数索引上的动作（建、查、走），「索引照用、旁边再加一遍线性找」在计数里看不见——把映射位置换进
+//! `units`、装要写的单元、装记录点名的单元、留给快照的角色清单两处、记录点名的数据角色那一处都是这样。判它要数产品里每一次身份比较
+//! （给 `TransactionUnit` 的相等比较计数，碰全部调用点），或量挂钟（这份文件不读挂钟）；这一类写法是对抗写法，正常改代码不会在用了索引之后
+//! 再逐个比一遍，所以登记在这里、不进 `crates/mutations.tsv`。攻方 NA4（记录点名的数据角色那一处索引之外再逐个 `contains`）、正推 ALT4
+//! （留给快照的角色清单那一处同样）、正推 ALT5（映射位置换进 `units` 时索引之外再线性 `position` 一遍）三条都落在这一类，跑这份文件全绿。
 mod common;
 
 /// `singlefs-core` 的构建脚本本身（七·6 后续批代码轮第一轮判决第三节第 4 条）：在这里编进来，判它交出的特性只看
@@ -45,7 +56,8 @@ use singlefs_core::address::{
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::code_two_tree::{
-    leaf_position_routed_to, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodePosition,
+    leaf_position_routed_to, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeCapacity,
+    CodeTwoTreeNodePosition,
 };
 use singlefs_core::compaction_plan::{
     CompactionBatchSize, CompactionBatchUnits, CompactionSwitches, PlannedRelocation,
@@ -65,16 +77,19 @@ use singlefs_core::records::{parse_mapping_entry, TREE_KIND_SNAPSHOT_INODE_ROOT}
 use singlefs_core::region_scan::CentralMappingEntry;
 use singlefs_core::transaction::lookup_work::{
     lookup_work_while, LookupSite, LookupWork, LookupWorkOfEachSite,
+    ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT,
 };
 use singlefs_core::transaction::{
     acquire_instance, asks_the_snapshot_lifetime_judgement,
     central_mapping_locations_in_the_version, copies_failing_the_release_checksum_check,
     placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots, publish_first_file,
-    publish_relocation_batch, publish_sequential_write_of_an_inode,
-    refuse_mapping_entries_that_do_not_name_two_pool_devices, warm_up,
-    CentralMappingOfAVersionReadLeafByLeaf, FileContentOfThePublish, FirstEntryOfEachIdentity,
-    FirstFile, MappedUnitsOfAVersionReadByRole, MappingLookup, MultiLevelCodeTwoTree, PoolVersion,
-    PoolWriter, RelocatedUnitHolder, TransactionOutput, TransactionUnit,
+    publish_new_inodes, publish_relocation_batch, publish_sequential_write_of_an_inode,
+    refuse_mapping_entries_that_do_not_name_two_pool_devices,
+    take_the_mapping_positions_over_the_location_hints, warm_up,
+    CentralMappingOfAVersionReadLeafByLeaf, CodeTwoTreeNodeCapacities, FileContentOfThePublish,
+    FirstEntryOfEachIdentity, FirstFile, MappedUnitsOfAVersionReadByRole, MappingLookup,
+    MultiLevelCodeTwoTree, PoolVersion, PoolWriter, RelocatedUnitHolder, TransactionOutput,
+    TransactionUnit,
 };
 use singlefs_core::unit::{
     build_index_node, data_unit_payload_capacity, parse_index_node, UNIT_CLASS_DATA,
@@ -1326,6 +1341,25 @@ fn whole_rewrite_published_once(
     seed: u64,
     data_units: usize,
     write_time_seconds: u64,
+) -> (TransactionOutput, TransactionOutput, LookupWorkOfEachSite) {
+    whole_rewrite_published_once_with_node_capacities(
+        pool,
+        inode,
+        seed,
+        data_units,
+        write_time_seconds,
+        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
+    )
+}
+
+/// 同 [`whole_rewrite_published_once`]，写入口装上 `node_capacities`（只供测试的记账树与中央映射树节点容量）。
+fn whole_rewrite_published_once_with_node_capacities(
+    pool: &mut PoolBuilt,
+    inode: InodeNumber,
+    seed: u64,
+    data_units: usize,
+    write_time_seconds: u64,
+    node_capacities: CodeTwoTreeNodeCapacities,
 ) -> (TransactionOutput, TransactionOutput, LookupWorkOfEachSite) {
     let parameters = common::parameters();
     let previous = version_of(&pool.session).clone();
@@ -1334,6 +1368,7 @@ fn whole_rewrite_published_once(
     let devices = &mut pool.devices;
     let (published, lookup_work) = lookup_work_while(|| {
         let mut writer = PoolWriter::new(&parameters, devices);
+        writer.set_code_two_tree_node_capacities(node_capacities);
         publish_sequential_write_of_an_inode(
             &mut writer,
             &mut session.allocator,
@@ -1354,6 +1389,30 @@ fn whole_rewrite_published_once(
     (previous, published, lookup_work)
 }
 
+/// 不经会话、直接在会话现行那一版上经写入口建一个 size 0 的 inode（`transaction::publish_new_inodes`，一次发布），写入口装上 `node_capacities`，
+/// 会话接到这一版上。inode 号水位每变一次留一级台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」），下一次发布照抄它：
+/// 记账树一次长一行。
+fn one_inode_created_with_node_capacities(
+    pool: &mut PoolBuilt,
+    write_time_seconds: u64,
+    node_capacities: CodeTwoTreeNodeCapacities,
+) {
+    let parameters = common::parameters();
+    let previous = version_of(&pool.session).clone();
+    let mut writer = PoolWriter::new(&parameters, &mut pool.devices);
+    writer.set_code_two_tree_node_capacities(node_capacities);
+    let created = publish_new_inodes(
+        &mut writer,
+        &mut pool.session.allocator,
+        &previous,
+        1,
+        write_time_seconds,
+        pool.session.instance,
+    )
+    .unwrap_or_else(|refusal| panic!("建一个 inode：{refusal:?}"));
+    pool.session.current = PoolVersion::of_a_version_of_trees(created);
+}
+
 /// `keys` 按 `version` 中央映射树的形状走到的叶有几片（去重）：批量读者查这几把 key 该解的就是这几片。
 fn distinct_mapping_leaves_routed_to_by<'key>(
     version: &TransactionOutput,
@@ -1522,6 +1581,14 @@ fn assert_the_lookups_of_one_publish_build_one_index_per_pass(
         snapshot_judgement_passes,
         "{what}：快照判定那一圈走了 {snapshot_judgement_passes} 遍，每遍问一次换下的角色有没有落点：{files_of_the_snapshot_judgement:?}"
     );
+    // 建索引次数等于进的次数分不开「每走一遍建一份」与「每问一个角色进一次、建一份」（判决第三节第 1 条）：再判每遍查的次数，
+    // 这次重写的每个数据单元在上一版里有没有落点各按 inode 号查一次文件。
+    assert!(
+        files_of_the_snapshot_judgement.index_lookups
+            >= snapshot_judgement_passes * data_units_rewritten,
+        "{what}：快照判定那一圈走了 {snapshot_judgement_passes} 遍，每遍这次重写的 {data_units_rewritten} 个数据单元各按 inode 号查一次文件：\
+         {files_of_the_snapshot_judgement:?}"
+    );
     // 经映射释放的角色清单同样先去掉留给快照的角色：快照下一个第一个文件之外的角色都不剩的那几遍不建，所以是「至多」；
     // 每问一个角色现建一份的，建的次数按问的角色数涨、超过走的遍数。
     let files_of_the_mapped_roles_released =
@@ -1580,6 +1647,38 @@ fn assert_the_units_of_the_published_version_are_looked_up_through_one_index(
     );
 }
 
+/// 一次装发布在点名的几处之外（`None` 那一格）至多建几份索引（判决第三节第 2 条）。量到的（七·6 后续批第二轮的实现员在副本上现跑）：
+/// 这份用例文件里没有快照时整份重写一个八单元的文件是 5（照抄第一个文件时整份重写两单元与十六单元的四次一样多），快照下 deadlist 几片叶的池上
+/// 整份重写八单元与三十二单元的文件是 5 与 6；正推腿在 64 个文件的池上量到 6（`research/prompts/m4-publish-lookup-followup-code-r2-sonnet-output.md`
+/// T2.2）。份数是常数，不随这次写的单元数涨；
+/// 每查一项现建一份、又建在点名的那一处之外的（攻方 NA2），按项数涨。
+const INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST: u64 = 6;
+
+/// 一次发布在点名的几处之外（`None` 那一格）建的索引：每装一次至多 [`INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST`] 份
+/// （装的次数取装要写的单元那一处进的次数）。交回（装的次数，`None` 那一格建索引的次数），给两档单元数比「不涨」用。
+fn assert_the_index_builds_outside_the_named_sites_stay_bounded(
+    lookup_work: &LookupWorkOfEachSite,
+    what: &str,
+) -> (u64, u64) {
+    let assemblies = lookup_work
+        .of_the_site(LookupSite::UnitsOfTheRolesWrittenByThisPublish)
+        .times_entered;
+    let outside_the_named_sites = lookup_work
+        .work_of_each_site
+        .get(&None)
+        .copied()
+        .unwrap_or_default();
+    assert!(
+        assemblies >= 1
+            && outside_the_named_sites.index_builds
+                <= assemblies * INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST,
+        "{what}：装了 {assemblies} 次，点名的几处之外每装一次至多建 {INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST} 份索引，\
+         建了 {} 份：{outside_the_named_sites:?}",
+        outside_the_named_sites.index_builds
+    );
+    (assemblies, outside_the_named_sites.index_builds)
+}
+
 /// 一处做的查找（建索引次数、解叶次数、查的次数）恰好是那一处走一遍该做的 `one_pass` 的正整数倍：发布算到定点时同一处可以走几遍
 /// （每一遍对同一个上一版、同一串角色做一样多的事），遍数从建索引次数（建索引不为 0 时）或查的次数现取。交回遍数。
 fn assert_the_work_is_whole_passes_of(
@@ -1622,8 +1721,11 @@ fn assert_the_work_is_whole_passes_of(
 /// 七·6 后续批另判：核两条位置项与读盘核校验和经「经映射释放的角色清单」每走一遍各建一份按 inode 号找文件的读者与留给快照的角色清单索引；
 /// 那一次发布里三棵多层码 2 树照抄的节点按树分开数，装要写与点名的单元、按 inode 号找文件、留给快照的角色清单与记录点名的数据角色都每走一遍建一份
 /// （`assert_the_lookups_of_one_publish_build_one_index_per_pass`）；整棵 extent 树装的时候都在照抄那一处里，每个节点现建一份索引的建的次数记在那里。
+/// 七·6 后续批代码轮第二轮判决第三节第 2 条另判：点名的几处之外（`None`）每装一次建的索引至多一个常数份
+/// （`assert_the_index_builds_outside_the_named_sites_stay_bounded`）。
 /// 判别力：`crates/mutations.tsv` 里以「七·6 第三轮」起头、点名这一条的几行（攻方 RB、RD、RE、RF，正推 M3、M4，释放与照抄那几处改回线性）；
-/// 以「七·6 后续批」起头、点名这一条的几行（装要写与点名的单元改回线性找，攻方 MA2、MA3、MA5，正推自打的经映射释放的角色清单那一行）。
+/// 以「七·6 后续批」起头、点名这一条的几行（装要写与点名的单元改回线性找，攻方 MA2、MA3、MA5）；以「七·6 后续批第二轮」起头、点名这一条的
+/// 那一行（攻方 NA2：装记录点名的单元每点名一个角色现建一份索引、建在点名那一处之外）。
 #[test]
 fn a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site(
 ) {
@@ -1800,6 +1902,10 @@ fn a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_i
         8,
         "整份重写一个八单元的文件（池里没有快照）",
     );
+    assert_the_index_builds_outside_the_named_sites_stay_bounded(
+        &lookup_work,
+        "整份重写一个八单元的文件（池里没有快照）",
+    );
 }
 
 /// 池里有一个快照的样子：快照之后整份重写过的文件的旧单元进了 deadlist、映射条目留存。
@@ -1831,7 +1937,7 @@ fn pool_with_a_snapshot_of_files(files: u64) -> PoolWithASnapshot {
 /// 七·6 后续批另判（判决第三节第 3 条 ①）：删快照放条目的每一遍恰好建一份留存条目按 key 的索引（建索引次数等于进那一处的次数，
 /// 查的次数是遍数乘放掉的条数）；重写那一次按 inode 号找文件等几处每走一遍建一份（`assert_the_lookups_of_one_publish_build_one_index_per_pass`）。
 /// 判别力：`crates/mutations.tsv` 里正推 M2（进 deadlist 那一支退回线性找 key、逐 key 查）与「七·6 第三轮 删快照：」起头的那一行；
-/// 「七·6 后续批」起头、点名这一条的几行（攻方 MA1，正推自打的删快照与快照判定那一圈两行）。
+/// 「七·6 后续批」起头、点名这一条的那一行（攻方 MA1）。
 #[test]
 fn a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once(
 ) {
@@ -2220,6 +2326,17 @@ const LARGE_FILES_OF_THE_DEADLIST_POOL: usize = 26;
 const LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT: usize = 24;
 /// 这个池里 deadlist 树至少几个节点（根加至少三片叶）：整份重写一个文件只改一两片叶与根，照抄的就不止一个。
 const DEADLIST_TREE_NODES_AT_LEAST: usize = 4;
+/// 记账树压小之后一片叶装几行（只供测试的 `CodeTwoTreeNodeCapacities::CappedForTests`）：两块盘的记账行是池级 3 行、每块盘 6 行，
+/// 加 inode 号水位的台阶行；造池那几次发布之后不到 20 行（用例开头现判；推的是 16 行：建完 inode 量到 56 行，减去 40 次建 inode 各一级台阶），装得进一片叶。
+/// 产品容量下一片叶装 477 行，两块盘的池里记账树长不到第二个节点、照抄不到（判决第三节第 3 条）。
+const ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS: usize = 20;
+/// 记账树压小之后一个内部节点装几个孩子。
+const ACCOUNTING_CHILDREN_ONE_CAPPED_INTERNAL_NODE_HOLDS: usize = 4;
+/// 量之前一次建一个 inode 建几次：每次多一级台阶行，记账树长到几片叶（量到的是 56 行、5 个节点）；之后的整份重写只改池级与每块盘那几行（代段换成这次的 txg），
+/// 只装旧台阶行的那几片叶与它们的父节点照抄。
+const INODE_CREATIONS_BEFORE_THE_MEASURED_REWRITES: u64 = 40;
+/// 记账树照抄的节点至少几个（判决第三节第 3 条：照抄到不止一个节点）。
+const CARRIED_ACCOUNTING_NODES_AT_LEAST: u64 = 2;
 
 /// 快照下 deadlist 树长到几片叶的池上，整份重写那个八单元的小文件、再整份重写一个没重写过的三十二单元的大文件（各一次发布，一次写的单元数在两档之间变）：
 /// - 三棵多层码 2 树照抄的节点按树分开数（七·6 后续批代码轮第一轮判决第三节第 3 条 ③）：deadlist 树照抄了不止一个节点，
@@ -2227,8 +2344,14 @@ const DEADLIST_TREE_NODES_AT_LEAST: usize = 4;
 /// - 留给快照的角色清单两处、记录点名的数据角色那一处（判决第三节第 2 条）、按 inode 号找文件三处、装要写与点名的单元（第 1 条），
 ///   两档都是每走一遍建一份索引、查的次数至少是这次重写的数据单元数（建索引的次数不随一次写的单元数涨；退回逐个比的那一处一次都不查）。
 ///
+/// 七·6 后续批代码轮第二轮判决第三节第 3 条另造：量之前把记账树压小（一片叶 [`ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS`] 行），一次建一个 inode 建
+/// [`INODE_CREATIONS_BEFORE_THE_MEASURED_REWRITES`] 次，台阶行把记账树撑到几片叶；两次重写都照这个容量装，三棵树都照抄了节点
+/// （记账树至少 [`CARRIED_ACCOUNTING_NODES_AT_LEAST`] 个），每棵各判整数遍。第 2 条另判：点名的几处之外（`None`）建的索引两档都至多一个常数份
+/// （量到的是 5 与 6：多出的那一份不随单元数涨，单元数是四倍）。
+///
 /// 判别力：`crates/mutations.tsv` 里攻方 MA4（记账树与 deadlist 树照抄的节点退回在 `units` 上线性找）与「七·6 后续批」起头、
-/// 点名这一条的几行（留给快照的角色清单两处、记录点名的数据角色那一处退回 `.contains`）。
+/// 点名这一条的几行（留给快照的角色清单两处、记录点名的数据角色那一处退回 `.contains`）；以「七·6 后续批第二轮」起头、点名这一条的那一行
+/// （攻方 NA1：只有记账树照抄的节点退回线性找）。
 #[test]
 fn rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart(
 ) {
@@ -2275,6 +2398,33 @@ fn rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadli
         deadlist_before.deadlist_entries.len(),
         deadlist_before.deadlist_tree.shape.nodes().len()
     );
+    let accounting_tree_capped = CodeTwoTreeNodeCapacities::CappedForTests {
+        accounting: CodeTwoTreeNodeCapacity {
+            leaf_entries: ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS,
+            internal_entries: ACCOUNTING_CHILDREN_ONE_CAPPED_INTERNAL_NODE_HOLDS,
+        },
+        central_mapping: MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format(),
+    };
+    assert!(
+        version_of(&pool.session).accounting_entries.len() <= ACCOUNTING_ROWS_ONE_CAPPED_LEAF_HOLDS,
+        "压小之前记账树一片叶装得下（{} 行）：压小的容量从一个合规的上一版接着长",
+        version_of(&pool.session).accounting_entries.len()
+    );
+    // 每建一个 inode 写入时间往后推一秒。
+    for write_time_seconds in (common::FIXED_WRITE_TIME_SECONDS + 25_000..)
+        .take(usize::try_from(INODE_CREATIONS_BEFORE_THE_MEASURED_REWRITES).expect("次数"))
+    {
+        one_inode_created_with_node_capacities(
+            &mut pool,
+            write_time_seconds,
+            accounting_tree_capped,
+        );
+    }
+    println!(
+        "publish_lookup_by_identity deadlist_pool accounting_entries={} accounting_tree_nodes={}",
+        version_of(&pool.session).accounting_entries.len(),
+        version_of(&pool.session).accounting_tree.node_count()
+    );
     let small_file = pool.inodes[0];
     let large_file_not_rewritten = large_files[LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT];
     for (inode, data_units, write_time_seconds) in [
@@ -2289,12 +2439,13 @@ fn rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadli
             common::FIXED_WRITE_TIME_SECONDS + 30_001,
         ),
     ] {
-        let (_, published, lookup_work) = whole_rewrite_published_once(
+        let (_, published, lookup_work) = whole_rewrite_published_once_with_node_capacities(
             &mut pool,
             inode,
             inode.0 + 9_000_011,
             data_units,
             write_time_seconds,
+            accounting_tree_capped,
         );
         let what = format!("快照下整份重写一个 {data_units} 单元的文件（deadlist 树几片叶）");
         let [carried_accounting_nodes, carried_deadlist_nodes, carried_mapping_nodes] =
@@ -2310,13 +2461,285 @@ fn rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadli
             carried_deadlist_nodes >= 2,
             "{what}：deadlist 树照抄了不止一个节点（{carried_deadlist_nodes} 个）"
         );
+        assert!(
+            carried_accounting_nodes >= CARRIED_ACCOUNTING_NODES_AT_LEAST
+                && carried_mapping_nodes >= 1,
+            "{what}：记账树照抄了至少 {CARRIED_ACCOUNTING_NODES_AT_LEAST} 个节点（{carried_accounting_nodes} 个）、中央映射树也照抄了\
+             （{carried_mapping_nodes} 个）：三棵树都判整数遍"
+        );
         assert_the_lookups_of_one_publish_build_one_index_per_pass(
             &published,
             &lookup_work,
             u64::try_from(data_units).expect("单元数"),
             &what,
         );
+        assert_the_index_builds_outside_the_named_sites_stay_bounded(&lookup_work, &what);
+    }
+}
+
+/// 照抄第一个文件的那两个池里，第一个文件各写几个数据单元。
+const DATA_UNITS_OF_THE_FIRST_FILE_OF_EACH_POOL: [usize; 2] = [8, 64];
+/// 照抄第一个文件的那两个池里第一个文件之外的文件（各整份写一次，量的时候各整份重写一次）：一次写的单元数在两档之间变，这次重写的角色数跟着变。
+const DATA_UNITS_OF_THE_FILES_REWRITTEN_WHILE_THE_FIRST_FILE_IS_CARRIED: [usize; 2] = [2, 16];
+
+/// 第一个文件分别写成 8 个与 64 个数据单元的两个池上，各整份重写一个两单元与一个十六单元的文件（第一个文件照抄；一次写的单元数在两档之间变，
+/// 这次重写的角色数跟着变）：装这一版时第一个文件的每个数据单元问它在不在这次重写的角色里，经这次重写的角色清单建一次的索引各查一次
+/// （七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）——每装一次进一次、建一份，查第一个文件的数据单元数那么多次，每次查至多一次
+/// 折半查找（改前在清单上逐个比，比较次数到第一个文件的数据单元数乘这次重写的角色数）；点名的几处之外建的索引四次一样多（判决第三节第 2 条）。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 后续批第二轮 照抄第一个文件：」起头的那一行（退回在这次重写的角色清单上逐个 `contains`）。
+// harness-test-granularity:one-scenario 两个池是同一个场景的两档：判的是查的次数随第一个文件的单元数变、建的次数与点名的几处之外的份数不变，单造一档比不出
+#[test]
+fn whole_rewrites_carrying_a_first_file_of_eight_and_of_sixty_four_data_units_ask_for_each_of_its_data_units_through_one_index_of_the_rewritten_roles(
+) {
+    let mut index_builds_outside_the_named_sites_of_each_rewrite = Vec::new();
+    for first_file_data_units in DATA_UNITS_OF_THE_FIRST_FILE_OF_EACH_POOL {
+        let mut pool = pool_with_files_of_the_sizes(
+            &DATA_UNITS_OF_THE_FILES_REWRITTEN_WHILE_THE_FIRST_FILE_IS_CARRIED,
+        );
+        let first_file_content = content_of_data_units(first_file_data_units, 0x51f1);
+        pool.session
+            .publish_user_change(
+                &mut pool.devices,
+                UserChange::SequentialWrite(FirstFile {
+                    content: &first_file_content,
+                    write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 40_000,
+                }),
+            )
+            .unwrap_or_else(|refusal| {
+                panic!("第一个文件写 {first_file_data_units} 个数据单元：{refusal:?}")
+            });
+        let inodes = pool.inodes.clone();
+        // 每重写一个文件写入时间往后推一秒。
+        for ((inode, data_units), write_time_seconds) in inodes
+            .iter()
+            .zip(DATA_UNITS_OF_THE_FILES_REWRITTEN_WHILE_THE_FIRST_FILE_IS_CARRIED)
+            .zip(common::FIXED_WRITE_TIME_SECONDS + 40_001..)
+        {
+            let (_, published, lookup_work) = whole_rewrite_published_once(
+                &mut pool,
+                *inode,
+                inode.0 + 11_000_027,
+                data_units,
+                write_time_seconds,
+            );
+            let what = format!(
+                "第一个文件 {first_file_data_units} 个数据单元时整份重写一个 {data_units} 单元的文件"
+            );
+            let first_file_data_units_carried =
+                u64::try_from(published.data_pointers.len()).expect("单元数");
+            let rewritten_roles = u64::try_from(published.rewritten.len()).expect("角色数");
+            let assemblies = lookup_work
+                .of_the_site(LookupSite::UnitsOfTheRolesWrittenByThisPublish)
+                .times_entered;
+            let rewritten_roles_asked = lookup_work
+                .of_the_site(LookupSite::RewrittenRolesAmongTheDataUnitsOfTheCarriedFirstFile);
+            println!(
+                "publish_lookup_by_identity carried_first_file first_file_data_units={first_file_data_units_carried} rewritten_roles={rewritten_roles} \
+                 comparisons_of_a_scan_of_the_rewritten_roles_at_most={} rewritten_roles_asked={rewritten_roles_asked:?}",
+                first_file_data_units_carried * rewritten_roles
+            );
+            assert_eq!(
+                first_file_data_units_carried,
+                u64::try_from(first_file_data_units).expect("单元数"),
+                "{what}：这一版照抄了第一个文件的全部数据单元"
+            );
+            assert_eq!(
+                (
+                    rewritten_roles_asked.times_entered,
+                    rewritten_roles_asked.index_builds,
+                    rewritten_roles_asked.index_lookups
+                ),
+                (
+                    assemblies,
+                    assemblies,
+                    assemblies * first_file_data_units_carried
+                ),
+                "{what}：装了 {assemblies} 次，每装一次这次重写的 {rewritten_roles} 个角色建一份按角色的索引，第一个文件的 \
+                 {first_file_data_units_carried} 个数据单元各查一次（进的次数、建索引次数、查的次数）：{rewritten_roles_asked:?}"
+            );
+            index_builds_outside_the_named_sites_of_each_rewrite.push(
+                assert_the_index_builds_outside_the_named_sites_stay_bounded(&lookup_work, &what),
+            );
+        }
+    }
+    assert!(
+        index_builds_outside_the_named_sites_of_each_rewrite
+            .windows(2)
+            .all(|pair| pair[0] == pair[1]),
+        "第一个文件两档、一次写的单元数两档，点名的几处之外的（装的次数，建索引的次数）一样：{index_builds_outside_the_named_sites_of_each_rewrite:?}"
+    );
+}
+
+/// 重挂的那两个池里第一个文件之外有几个文件。
+const FILES_OF_EACH_REMOUNTED_POOL: [u64; 2] = [4, 16];
+/// 重挂的那两个池里每个文件几个数据单元。
+const DATA_UNITS_OF_EACH_FILE_OF_THE_REMOUNTED_POOLS: usize = 2;
+
+/// 第一个文件之外 4 个与 16 个两单元文件的两个池，各不卸载直接重挂一次（可写挂载从盘上重建这一版），读这次挂载在产品里做的按身份查：
+/// - 把映射位置换进重建出来那一版的 `units`（`take_the_mapping_positions_over_the_location_hints`，七·6 后续批代码轮第二轮判决第三节第 1 条）：
+///   进 1 次、建 1 份按角色的索引、查有位置的进映射角色那么多次（这一版每把映射 key 都查得到映射条目，就是进映射的角色数）；
+/// - 数据单元照 inode 记录核时按 inode 号找记录（判决第五节，用户 2026-10-23 定并进）：进 1 次、建 1 份按 inode 号的索引，第一个文件与
+///   第一个文件之外每个文件各查一次（文件数加一，随文件数线性涨；改前在全部记录上逐个找，是文件数的平方）。
+///
+/// 每次查至多一次折半查找。两档的进、建次数一样，查的次数跟着进映射的角色数与文件数变。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 后续批第二轮」起头、点名这一条的两行（攻方 NA3：恢复那一侧每把有位置的 key 调一次；
+/// 按 inode 号找记录退回在全部记录上逐个找）。
+// harness-test-granularity:one-scenario 两个池是同一个场景的两档：判的是查的次数随文件数变、进与建的次数不变，单造一档比不出
+#[test]
+fn remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records(
+) {
+    let parameters = common::parameters();
+    for files in FILES_OF_EACH_REMOUNTED_POOL {
+        let PoolBuilt {
+            mut devices,
+            session,
+            inodes: _,
+        } = pool_with_files_written_once(files, DATA_UNITS_OF_EACH_FILE_OF_THE_REMOUNTED_POOLS);
+        let before_the_remount = version_of(&session).clone();
+        drop(session);
+        let mapped_roles = u64::try_from(before_the_remount.mapped_units.len()).expect("角色数");
+        assert!(
+            before_the_remount.mapped_units.iter().all(|(_, key)| matches!(
+                central_mapping_locations_in_the_version(&before_the_remount, key),
+                MappingLookup::Found(_)
+            )),
+            "重挂之前那一版每把映射 key 都查得到映射条目：有位置的进映射角色就是全部 {mapped_roles} 个"
+        );
+        assert_eq!(
+            u64::try_from(before_the_remount.files_other_than_the_first.len()).expect("文件数"),
+            files,
+            "第一个文件之外的 {files} 个文件都有数据单元"
+        );
+        let (mounted, lookup_work) = lookup_work_while(|| {
+            mount_writable_with_compaction_switches(
+                &parameters,
+                &mut devices,
+                CompactionSwitches::PRODUCT,
+            )
+        });
+        mounted.unwrap_or_else(|error| panic!("重挂 {files} 个文件的池：{error:?}"));
+        assert_every_lookup_examines_at_most_a_binary_search(
+            &lookup_work,
+            &format!("重挂 {files} 个文件的池"),
+        );
+        let mapping_positions =
+            lookup_work.of_the_site(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions);
+        let inode_records =
+            lookup_work.of_the_site(LookupSite::InodeRecordsOfTheFilesJudgedByTheRecovery);
+        println!(
+            "publish_lookup_by_identity remount files={files} mapped_roles={mapped_roles} mapping_positions={mapping_positions:?} \
+             inode_records={inode_records:?}"
+        );
+        assert_eq!(
+            (
+                mapping_positions.times_entered,
+                mapping_positions.index_builds,
+                mapping_positions.index_lookups
+            ),
+            (1, 1, mapped_roles),
+            "重挂 {files} 个文件的池：重建这一版时把映射位置换进 `units` 进 1 次、建 1 份按角色的索引，{mapped_roles} 个有位置的进映射角色各查一次\
+             （进的次数、建索引次数、查的次数）：{mapping_positions:?}"
+        );
+        assert_eq!(
+            (
+                inode_records.times_entered,
+                inode_records.index_builds,
+                inode_records.index_lookups
+            ),
+            (1, 1, files + 1),
+            "重挂 {files} 个文件的池：数据单元照 inode 记录核时进 1 次、建 1 份按 inode 号的索引，第一个文件与 {files} 个别的文件各找一次记录\
+             （进的次数、建索引次数、查的次数）：{inode_records:?}"
+        );
+    }
+}
+
+/// 把映射位置换进 `units` 时一把有位置的 key 都没有：那一处不进、不建索引，这一版一个字节不变（`take_the_mapping_positions_over_the_location_hints`
+/// 末尾那一句「一把有位置的 key 都没有的不进、不建」；七·6 后续批代码轮第二轮判决第三节第 5 条）。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 后续批第二轮 映射位置换进 units：」起头的那一行（正推 ALT12：一把都没有也照建索引）。
+#[test]
+fn taking_the_mapping_positions_without_any_key_that_has_a_position_builds_no_index_and_leaves_the_version_unchanged(
+) {
+    let pool = pool_with_files_written_once(2, 2);
+    let before = version_of(&pool.session).clone();
+    let mut version = before.clone();
+    let ((), lookup_work) = lookup_work_while(|| {
+        take_the_mapping_positions_over_the_location_hints(&mut version, |_| None);
+    });
+    assert_eq!(
+        lookup_work.of_the_site(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions),
+        LookupWork::default(),
+        "一把有位置的 key 都没有：把映射位置换进 `units` 那一处不进、不建、不查"
+    );
+    assert!(
+        version == before,
+        "一把有位置的 key 都没有：这一版一个字节不变"
+    );
+}
+
+/// cargo 真开着 `lookup-work-counters` 编 `singlefs-core` 时（这个测试包经 dev 依赖打开它，计数的几处才编得进来），它的构建脚本交给依赖方的
+/// 特性里有它（七·6 后续批代码轮第二轮判决第三节第 7 条「开着要报开着」那一半：外带变量那一条用例把构建脚本编进这个测试包，那里的 cfg 恒为假，
+/// 只钉得住「没开别报开着」）。读的是构建脚本经 `cargo::rustc-env` 编进 `singlefs-core` 的那一份，与它打给依赖方的 `cargo::metadata` 同一个值。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 后续批第二轮 构建脚本：」起头、点名这一条的那一行（正推 T4b：表里那一格 cfg 写死成假）。
+#[test]
+fn the_core_build_script_reports_the_lookup_work_counters_feature_that_cargo_enabled_for_this_test_package(
+) {
+    assert_eq!(
+        ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT, "LOOKUP_WORK_COUNTERS",
+        "cargo 开着 lookup-work-counters 编的 singlefs-core：构建脚本交出的特性里有它"
+    );
+}
+
+/// 构建脚本比 `CARGO_ENCODED_RUSTFLAGS` 里的 `--cfg feature=…` 与自己的 cfg（七·6 后续批代码轮第二轮判决第五节 T4 ①，用户 2026-10-23 定并进）：
+/// 这个测试包里构建脚本那一格 cfg 为假（同外带变量那一条），参数里经 `--cfg feature="lookup-work-counters"`（分两项、写成一项、等号两边带空白）
+/// 点名打开它的判成对不上、点名那个特性；没点名它的（空、别的特性、别的参数、`--check-cfg` 里提到它、不跟在 `--cfg` 后面）不判；
+/// 表说开着时同样的参数不判。构建时对不上就停（`build.rs` 的 `main`）。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 后续批第二轮 构建脚本：」起头、点名这一条的那一行（认不出 `feature` 那个键）。
+#[test]
+fn the_core_build_script_finds_the_features_its_rustflags_open_while_its_own_cfg_says_off() {
+    let table = singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG;
+    assert_eq!(
+        table,
+        [("lookup-work-counters", false)],
+        "这个测试包里构建脚本那一格 cfg 为假"
+    );
+    for encoded_rustflags in [
+        "--cfg\u{1f}feature=\"lookup-work-counters\"",
+        "--cfg=feature=\"lookup-work-counters\"",
+        "-C\u{1f}opt-level=1\u{1f}--cfg\u{1f}feature = \"lookup-work-counters\"",
+    ] {
+        assert_eq!(
+            singlefs_core_build_script::features_opened_by_the_rustflags_but_off_in_this_build_script(
+                encoded_rustflags,
+                &table,
+            ),
+            ["lookup-work-counters"],
+            "{encoded_rustflags:?} 点名打开 lookup-work-counters、构建脚本的 cfg 说没开：对不上"
+        );
     }
+    for encoded_rustflags in [
+        "",
+        "--cfg\u{1f}feature=\"gpu\"",
+        "-C\u{1f}opt-level=1",
+        "--check-cfg\u{1f}cfg(feature, values(\"lookup-work-counters\"))",
+        "feature=\"lookup-work-counters\"",
+    ] {
+        assert!(
+            singlefs_core_build_script::features_opened_by_the_rustflags_but_off_in_this_build_script(
+                encoded_rustflags,
+                &table,
+            )
+            .is_empty(),
+            "{encoded_rustflags:?} 没经 --cfg 点名打开表里的特性：不判"
+        );
+    }
+    assert!(
+        singlefs_core_build_script::features_opened_by_the_rustflags_but_off_in_this_build_script(
+            "--cfg\u{1f}feature=\"lookup-work-counters\"",
+            &[("lookup-work-counters", true)],
+        )
+        .is_empty(),
+        "cfg 说开着、参数也点名打开：对得上"
+    );
 }
 
 /// 子进程标记：设着时这条用例是父进程起的那一趟（进程环境里带着外面设的那个特性变量）。
diff --git a/crates/singlefs-unit-graph/src/build_features.rs b/crates/singlefs-unit-graph/src/build_features.rs
index d6b4317..f1ca1dd 100644
--- a/crates/singlefs-unit-graph/src/build_features.rs
+++ b/crates/singlefs-unit-graph/src/build_features.rs
@@ -255,6 +255,18 @@ pub fn features_enabled_on_the_dependencies(
     Ok(features_enabled_by_the_build)
 }
 
+/// `singlefs-core` 的构建脚本本身，只在单测里编进来读它的特性表（`tests::the_core_build_script_lists_every_feature_of_its_manifest`）。
+/// 本包没有 `lookup-work-counters`，表里那一格的 cfg 在这里恒为假；构建脚本的 `main` 在这里用不上。写在文件顶层：`#[path]` 从本文件所在的
+/// `src/` 起算（写在内联的 `mod tests` 里要从不存在的 `src/build_features/tests/` 起算）。
+#[cfg(test)]
+#[allow(
+    dead_code,
+    unexpected_cfgs,
+    reason = "构建脚本的 main 在单测里不调；它写的 cfg!(feature = \"lookup-work-counters\") 是 singlefs-core 的特性，本包没有这个特性"
+)]
+#[path = "../../singlefs-core/build.rs"]
+mod singlefs_core_build_script;
+
 #[cfg(test)]
 mod tests {
     use std::num::NonZeroUsize;
@@ -645,4 +657,33 @@ mod tests {
             "核心层构建脚本照 {ENABLED_FEATURES_METADATA_KEY} 交出特性"
         );
     }
+
+    /// `singlefs-core` 清单 `[features]` 里的每个特性在它构建脚本的特性表（`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG`）里恰好一行、
+    /// 表里没有清单之外的名字（`crates/singlefs-core/build.rs` 文件头点名的这一条）。构建脚本只读 cfg、构建时自己不比：清单加了特性而表没加，
+    /// 那个特性开着时交出去的元数据里没有它，依赖方的判法单元键按没开算，`cargo build` 与 checker 档的构建都不报。
+    /// 表编进来读（同一份源码），清单照原文解（`manifest::parse_crate_manifest`）。
+    #[test]
+    fn the_core_build_script_lists_every_feature_of_its_manifest() {
+        let core_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../singlefs-core");
+        let manifest_text =
+            std::fs::read_to_string(core_directory.join("Cargo.toml")).expect("读得了核心层清单");
+        let manifest =
+            crate::manifest::parse_crate_manifest(&manifest_text).expect("核心层清单读得了");
+        let names_in_the_manifest: Vec<&str> =
+            manifest.features.keys().map(String::as_str).collect();
+        let mut names_in_the_table: Vec<&str> =
+            super::singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG
+                .iter()
+                .map(|(feature, _)| *feature)
+                .collect();
+        names_in_the_table.sort_unstable();
+        assert!(
+            !names_in_the_manifest.is_empty(),
+            "核心层清单 [features] 里至少有 lookup-work-counters：{names_in_the_manifest:?}"
+        );
+        assert_eq!(
+            names_in_the_table, names_in_the_manifest,
+            "核心层构建脚本的特性表与清单 [features] 逐个相同"
+        );
+    }
 }
````

## mutations-append.tsv

````text
七·6 后续批第二轮 恢复重建一版：每把有位置的映射 key 各调一次把映射位置换进 units，每次重建 units 的索引（攻方 NA3）	crates/singlefs-core/src/recovery.rs	    take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {\n        mapping_entries.mapping_position_of_key.get(key).copied()\n    });	    for (only_key, only_locations) in &mapping_entries.mapping_position_of_key {\n        take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {\n            (key == only_key.as_slice()).then_some(*only_locations)\n        });\n    }	-p singlefs-harness --test publish_lookup_by_identity -- --exact remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records	remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records
七·6 后续批第二轮 装记录点名的单元：每点名一个角色在点名那一处之外现建一份这一版 units 的索引（攻方 NA2）	crates/singlefs-core/src/transaction.rs	    let named_unit_of = |identity: TransactionUnit| -> NamedUnit {\n        let unit = {\n            let _inside_the_named_units =	    let named_unit_of = |identity: TransactionUnit| -> NamedUnit {\n        let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);\n        let unit = {\n            let _inside_the_named_units =	-p singlefs-harness --test publish_lookup_by_identity -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site	a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site
七·6 后续批第二轮 照抄多层码 2 树节点：只有记账树照抄的节点退回在上一版 units 上线性找（攻方 NA1）	crates/singlefs-core/src/transaction.rs	                let carried = units_of_the_previous_version\n                    .expect(\n                        "照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来",\n                    )\n                    .unit(tree.role_of_node(*previous_position, &previous_tree.shape));	                let carried = if matches!(tree, MultiLevelCodeTwoTree::Accounting) {\n                    previous_version.unit(tree.role_of_node(*previous_position, &previous_tree.shape))\n                } else {\n                    units_of_the_previous_version\n                        .expect("照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来")\n                        .unit(tree.role_of_node(*previous_position, &previous_tree.shape))\n                };	-p singlefs-harness --test publish_lookup_by_identity -- --exact rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart	rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart
七·6 后续批第二轮 映射位置换进 units：一把有位置的 key 都没有也照进那一处、照建索引（正推 ALT12）	crates/singlefs-core/src/transaction.rs	    if new_slot_of_each_mapped_role.is_empty() {\n        return;\n    }\n		-p singlefs-harness --test publish_lookup_by_identity -- --exact taking_the_mapping_positions_without_any_key_that_has_a_position_builds_no_index_and_leaves_the_version_unchanged	taking_the_mapping_positions_without_any_key_that_has_a_position_builds_no_index_and_leaves_the_version_unchanged
七·6 后续批第二轮 照抄第一个文件：数据单元在不在这次重写的角色里退回在角色清单上逐个 contains	crates/singlefs-core/src/transaction.rs	                    rewritten_by_role\n                        .first_entry_of(&data_role_at(position))\n                        .is_some()	                    rewritten.contains(&data_role_at(position))	-p singlefs-harness --test publish_lookup_by_identity -- --exact whole_rewrites_carrying_a_first_file_of_eight_and_of_sixty_four_data_units_ask_for_each_of_its_data_units_through_one_index_of_the_rewritten_roles	whole_rewrites_carrying_a_first_file_of_eight_and_of_sixty_four_data_units_ask_for_each_of_its_data_units_through_one_index_of_the_rewritten_roles
七·6 后续批第二轮 恢复按 inode 号找记录：退回在全部 inode 叶容器的记录上逐个找	crates/singlefs-core/src/recovery.rs	        inode_records_by_number\n            .first_entry_of(&inode)\n            .map(|inode_record| **inode_record)	        inode_records_in_container_order\n            .iter()\n            .find(|inode_record| inode_record.inode == inode)\n            .map(|inode_record| **inode_record)	-p singlefs-harness --test publish_lookup_by_identity -- --exact remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records	remounting_pools_of_four_and_of_sixteen_files_builds_one_index_for_the_mapping_positions_and_one_for_the_inode_records
七·6 后续批第二轮 构建脚本：特性表里那一格 cfg 写死成假（cargo 开着也报没开，正推 T4b）	crates/singlefs-core/build.rs	    "lookup-work-counters",\n    cfg!(feature = "lookup-work-counters"),	    "lookup-work-counters",\n    false,	-p singlefs-harness --test publish_lookup_by_identity -- --exact the_core_build_script_reports_the_lookup_work_counters_feature_that_cargo_enabled_for_this_test_package	the_core_build_script_reports_the_lookup_work_counters_feature_that_cargo_enabled_for_this_test_package
七·6 后续批第二轮 构建脚本：认不出 RUSTFLAGS 里 --cfg 的 feature 键（带 --target 时库开着、构建脚本没开也不停）	crates/singlefs-core/build.rs	        if name.trim() == "feature" {	        if name.trim() == "features" {	-p singlefs-harness --test publish_lookup_by_identity -- --exact the_core_build_script_finds_the_features_its_rustflags_open_while_its_own_cfg_says_off	the_core_build_script_finds_the_features_its_rustflags_open_while_its_own_cfg_says_off
七·6 后续批第二轮 构建脚本：特性表里的特性名写错一个字母（与核心层清单 [features] 对不上，正推 T4a）	crates/singlefs-core/build.rs	    "lookup-work-counters",\n    cfg!(feature = "lookup-work-counters"),	    "lookup-work-counter",\n    cfg!(feature = "lookup-work-counters"),	-p singlefs-unit-graph -p singlefs-core --lib -- build_features::tests::the_core_build_script_lists_every_feature_of_its_manifest	the_core_build_script_lists_every_feature_of_its_manifest
````

## mutations-delete.txt

````text
七·6 后续批 删快照：每条放掉的条目现建留存条目索引（正推自打）
七·6 后续批 经映射释放的角色清单：每问一个角色现建按 inode 号的文件索引（正推自打）
七·6 后续批 快照判定那一圈：每问一个角色现建按 inode 号的文件索引（正推自打）
````
