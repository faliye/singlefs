# m4-publish-lookup-code-r2：七·6 第二批补丁原文（crates.patch、mutations-append.tsv 与 mutations-replacements.tsv 原样）（2026-10-21）

## crates.patch

````diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -2779,9 +2779,22 @@
     slots_holding_user_data: BTreeSet<SlotNumber>,
 }
 
-fn version_as_the_compaction_reads_it(
+/// 整理读一版时中央映射那一半（[`central_mapping_as_the_compaction_reads_it`]）。
+pub struct CentralMappingAsTheCompactionReadsIt {
+    /// 每个进映射单元按 key 在这一版的映射树里查到的映射条目，按 `mapped_units` 的次序（查不到、条目窄于字段表的不收），
+    /// 后面接这一版留存的映射条目，按留存的次序。
+    pub mapping_entries: Vec<CentralMappingEntry>,
+    /// 取上面那些条目时批量读者解过几片映射叶（[`CentralMappingOfAVersionReadLeafByLeaf::leaves_decoded`]）。
+    pub mapping_leaves_decoded: usize,
+}
+
+/// 整理读一版时取映射条目的那一步：整理选段与搬单元读版本（`version_as_the_compaction_reads_it`）经它取，别处不调。
+/// 公开给测试：七·6 的用例拿它逐项比逐 key 查（`central_mapping_locations_in_the_version`）的参照、数整理读一版解了几片叶，
+/// 判整理读漏、读错与退回逐 key 查（七·6 代码轮第一轮判决第三节 T4）。
+#[must_use]
+pub fn central_mapping_as_the_compaction_reads_it(
     version: &TransactionOutput,
-) -> VersionAsTheCompactionReadsIt {
+) -> CentralMappingAsTheCompactionReadsIt {
     // 每把 key 查一次，叶按片解一次（七·6）：答案与逐 key 查 `central_mapping_locations_in_the_version` 逐个相同。
     let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
     let mut mapping_entries: Vec<CentralMappingEntry> = version
@@ -2802,6 +2815,19 @@
             locations: *locations,
         },
     ));
+    CentralMappingAsTheCompactionReadsIt {
+        mapping_entries,
+        mapping_leaves_decoded: central_mapping.leaves_decoded(),
+    }
+}
+
+fn version_as_the_compaction_reads_it(
+    version: &TransactionOutput,
+) -> VersionAsTheCompactionReadsIt {
+    let CentralMappingAsTheCompactionReadsIt {
+        mapping_entries,
+        mapping_leaves_decoded: _,
+    } = central_mapping_as_the_compaction_reads_it(version);
     let mut pinned_slots = BTreeSet::new();
     for unit in &version.units {
         let is_exempt_from_the_mapping = match unit.identity {
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -3299,7 +3299,8 @@
     /// 这一版里进映射的角色 `identity` 的映射 key（同在 `mapped_units` 上按角色 `iter().find`）。
     ///
     /// # Panics
-    /// 这一版里这个角色不进映射：调用方只拿照抄进下一版的、上一版里进映射的角色来查（上一版装映射节点时逐个记下了）。
+    /// 这一版里这个角色不进映射：调用方只拿上一版里进映射的角色来查——照抄进下一版的角色、快照判定那一圈换下的有落点的角色
+    /// （上一版装映射节点时逐个记下了）。
     #[must_use]
     pub fn mapping_key_of(&self, identity: TransactionUnit) -> &'table [u8] {
         self.first_entry_of(&identity)
@@ -4403,6 +4404,14 @@
             })
             .locations_of_key(key)
     }
+
+    /// 到此刻解过几片叶：每片叶在第一次有 key 走到它时解一次、之后不再解，所以至多是这一版中央映射树的节点数。
+    /// 测试拿它判「整理读一版每片叶至多解一次、每把 key 都经这个读者查」（七·6 的增长判据数次数，不读挂钟与 CPU 时间）；
+    /// 读的是已有的那张表的长度，不另加计数。
+    #[must_use]
+    pub fn leaves_decoded(&self) -> usize {
+        self.leaves_read.len()
+    }
 }
 
 /// 经映射释放的角色（数据、extent 根、inode 叶容器与根、分配记录树、记账树）：这次重写的角色里走得到映射条目的那几个，按角色次序。
@@ -5715,8 +5724,13 @@
     let mut user_visible_roles_kept_for_the_snapshots = Vec::new();
     let mut entries_killed_this_publish: Vec<DeadlistEntry> = Vec::new();
     let mut mapping_entries_kept_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = Vec::new();
+    // 换下的角色各查一次映射 key，查法与照抄全部角色那一圈同一个索引（七·6）：建一次走完上一版的 `mapped_units`，
+    // 之后每次查不再走整张表；同一角色取最先那一项、查不到时的 panic 文案与在 `mapped_units` 上 `iter().find` 相同。
+    let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();
     for identity in &user_visible_roles_replaced {
-        let mapping_key = mapping_key_carried_from(previous_version, *identity);
+        let mapping_key = mapped_units_of_the_previous_version
+            .mapping_key_of(*identity)
+            .to_vec();
         match snapshot_lifetime_judgement
             .verdict_of(&mapping_key, previous_snapshot_txg_of_the_live_head)
         {
@@ -7060,20 +7074,6 @@
     )
 }
 
-/// 从上一版取一个照抄进来的角色的映射 key。在 `mapped_units` 上线性找：只查几个角色的用它（快照判定那一圈只问这次换下的几个）；
-/// 照抄全部角色的那一圈用 [`TransactionOutput::mapped_units_by_role`]（七·6）。
-///
-/// # Panics
-/// 上一版里没有这个角色的映射 key：照抄进来的角色恒是上一版里进映射的那一个（上一版装映射节点时逐个记下了）。
-fn mapping_key_carried_from(previous: &TransactionOutput, identity: TransactionUnit) -> Vec<u8> {
-    previous
-        .mapped_units
-        .iter()
-        .find(|(mapped, _)| *mapped == identity)
-        .map(|(_, key)| key.clone())
-        .expect("照抄进来的角色在上一版里进映射、有一把 key")
-}
-
 /// 按 key 空间定形状的两棵树这次之后的样子（`resolve` 给映射排 key 要它们的节点清单）。
 struct MappedPositionAddressedTrees<'plans> {
     extent: &'plans ExtentTreePlan,
diff --git a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
--- a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
+++ b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
@@ -4,18 +4,29 @@
 //! （`transaction::CentralMappingOfAVersionReadLeafByLeaf`）由在 `Vec` 上 `iter().find` 换成按身份建一次的索引。这里钉三样：
 //! - 等价：两档池（十六个四单元的文件、四十八个八单元的文件，后一档的中央映射树不止一片叶）上跑同一串操作（按 inode 号写、整份重写四分之三的文件、
 //!   抬 F、整理到停、整理之后再写、unlink 一个文件），每一版的每一次按身份查都与测试里的线性查找参照逐项相同，池级 checker 一条违例都没有；
-//!   映射叶解不开、条目窄于字段表、同一把 key 有两条，三种叶上批量读与逐 key 查逐项相同。
-//! - 增长：同一进程里一批整理的线程 CPU 从 512 个数据单元的池到 2048 个（四倍）涨不到门槛的倍数（harness 慢档用例）。
+//!   映射叶解不开、条目窄于字段表、同一把 key 有两条，三种叶上批量读与逐 key 查逐项相同。整理读每一版取的映射条目
+//!   （`mounted_session::central_mapping_as_the_compaction_reads_it`）与逐 key 查的参照逐项相同（不漏、不错、次序同），批量读者解的叶
+//!   恰好是进映射的 key 走到的那几片。
+//! - 增长：数次数，不读挂钟、不读 CPU 时间（七·6 代码轮第一轮判决第三节 T4）。按身份查一次比几次身份：表长四倍时每次查平均比较次数
+//!   涨不到门槛的倍数（合成表一条快档用例；512 与 2048 个数据单元的两档池上整理读的那几版，harness 慢档用例）；整理读一版解几片映射叶：
+//!   恰好是进映射的 key 走到的那几片（每一版都判，慢档用例的两档池上也判）。
 //!
 //! 池都在内存里的两块 4 GiB 稀疏盘上造（`SparseBlockDevice`），配方照 E185 装置的 `construct_the_pool`：每个文件整份写进去，
 //! 再整份重写下标不是 4 的倍数的那几个，抬 F 到换下的落点都回收，留下每段约四分之一还活着的聚簇段。
 mod common;
 
+use std::cell::Cell;
+use std::cmp::Ordering;
+use std::collections::BTreeSet;
+
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber};
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::code_two_tree::{
+    leaf_position_routed_to, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodePosition,
+};
 use singlefs_core::compaction_plan::{
     CompactionBatchSize, CompactionBatchUnits, CompactionSwitches,
 };
@@ -26,8 +37,11 @@
 use singlefs_core::mount::{
     mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
 };
-use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession, UserChange};
+use singlefs_core::mounted_session::{
+    central_mapping_as_the_compaction_reads_it, CompactionRunOutcome, MountedSession, UserChange,
+};
 use singlefs_core::records::parse_mapping_entry;
+use singlefs_core::region_scan::CentralMappingEntry;
 use singlefs_core::transaction::{
     acquire_instance, central_mapping_locations_in_the_version, publish_first_file, warm_up,
     CentralMappingOfAVersionReadLeafByLeaf, FirstEntryOfEachIdentity, FirstFile, MappingLookup,
@@ -329,9 +343,68 @@
     mapping_key_lookups: u64,
     file_lookups: u64,
     central_mapping_lookups: u64,
+    compaction_reads_of_the_central_mapping: u64,
     versions_whose_central_mapping_tree_has_several_nodes: u64,
 }
 
+/// 一版里进映射单元的 key 按这一版中央映射树的形状走到的叶（去重）：整理读这一版时批量读者该解的就是这几片。
+fn mapping_leaves_the_mapped_keys_route_to(
+    version: &TransactionOutput,
+) -> BTreeSet<CodeTwoTreeNodePosition> {
+    version
+        .mapped_units
+        .iter()
+        .filter_map(|(_, key)| {
+            leaf_position_routed_to(
+                &version.central_mapping_tree.shape,
+                &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
+            )
+        })
+        .collect()
+}
+
+/// 整理读一版该取到的映射条目，用逐 key 查（`central_mapping_locations_in_the_version`，每把 key 线性找叶、整片重解）当参照：
+/// 每个进映射单元按 `mapped_units` 的次序查，查到的收，后面接这一版留存的映射条目。
+fn mapping_entries_found_key_by_key(version: &TransactionOutput) -> Vec<CentralMappingEntry> {
+    version
+        .mapped_units
+        .iter()
+        .filter_map(
+            |(_, key)| match central_mapping_locations_in_the_version(version, key) {
+                MappingLookup::Found(locations) => Some(CentralMappingEntry {
+                    key: key.clone(),
+                    locations,
+                }),
+                MappingLookup::NodeMalformedOrNoEntryWithThisKey
+                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
+            },
+        )
+        .chain(
+            version
+                .mapping_entries_kept_for_the_snapshots
+                .iter()
+                .map(|(key, locations)| CentralMappingEntry {
+                    key: key.clone(),
+                    locations: *locations,
+                }),
+        )
+        .collect()
+}
+
+/// 整理读这一版时批量读者解过的叶恰好是进映射的 key 走到的那几片（少了是有 key 没经它查，例如退回逐 key 查；
+/// 读者按叶记、一片只解一次）。
+fn assert_the_compaction_read_decodes_exactly_the_routed_leaves(version: &TransactionOutput) {
+    let leaves_decoded = central_mapping_as_the_compaction_reads_it(version).mapping_leaves_decoded;
+    let routed_leaves = mapping_leaves_the_mapped_keys_route_to(version).len();
+    assert_eq!(
+        leaves_decoded,
+        routed_leaves,
+        "txg {}：整理读这一版解了 {leaves_decoded} 片映射叶，进映射的 {} 把 key 走到 {routed_leaves} 片（每片解一次、每把 key 都经批量读者查）",
+        version.root.checkpoint_txg.0,
+        version.mapped_units.len()
+    );
+}
+
 /// 在两个 `Option<&Entry>` 上判「是表里同一项（或都没有）」：比地址，不比值（取的必须是最先那一项，不只是值相等的一项）。
 fn is_the_same_entry<Entry>(indexed: Option<&Entry>, found_linearly: Option<&Entry>) -> bool {
     match (indexed, found_linearly) {
@@ -342,7 +415,7 @@
 }
 
 /// 一版里每一种按身份查都与线性查找参照（`iter().find`、逐 key 的 `central_mapping_locations_in_the_version`）比一遍：表里的每个身份，
-/// 再加一个哪一版都没有的身份。
+/// 再加一个哪一版都没有的身份；整理读这一版取的映射条目与逐 key 查的参照逐项比，批量读者解的叶数与 key 走到的叶数比。
 fn compare_every_lookup_with_a_linear_find(
     version: &TransactionOutput,
     compared: &mut LookupsCompared,
@@ -430,6 +503,14 @@
         );
         compared.central_mapping_lookups += 1;
     }
+    assert_eq!(
+        central_mapping_as_the_compaction_reads_it(version).mapping_entries,
+        mapping_entries_found_key_by_key(version),
+        "txg {}：整理读这一版取的映射条目与逐 key 查的参照逐项相同（不漏、不错、次序同）",
+        version.root.checkpoint_txg.0
+    );
+    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version);
+    compared.compaction_reads_of_the_central_mapping += 1;
     if version.central_mapping_tree.shape.nodes().len() > 1 {
         compared.versions_whose_central_mapping_tree_has_several_nodes += 1;
     }
@@ -549,7 +630,8 @@
 }
 
 /// 十六个四单元的文件（64 个数据单元，中央映射树一片叶）：每一版的每一次按身份查都与线性查找相同，整理真发了批，镜像池级 checker 判绿。
-/// 判别力：`crates/mutations.tsv` 里以「七·6 按身份查：」起头的几行（索引漏了一类单元、照抄取错角色、文件按错的号查、批量读取最先那条）。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 按身份查：」起头的几行（索引漏了一类单元、照抄取错角色、文件按错的号查、批量读取最先那条），
+/// 以「七·6 整理读版本：」起头的几行（整理读版本隔一把 key 才查、漏查第一把 key、绕开批量读者逐 key 查）。
 #[test]
 fn every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units(
 ) {
@@ -562,7 +644,10 @@
         "整理真搬了单元：读版本走到了批量读映射（{compared:?}）"
     );
     assert!(
-        compared.versions >= 40 && compared.file_lookups >= 16 && compared.unit_lookups >= 64,
+        compared.versions >= 40
+            && compared.file_lookups >= 16
+            && compared.unit_lookups >= 64
+            && compared.compaction_reads_of_the_central_mapping == compared.versions,
         "每一版都比过：{compared:?}"
     );
 }
@@ -780,16 +865,133 @@
     assert!(compared_on_the_original >= 4, "原样那一版每把 key 都比过");
 }
 
-/// 本线程到此刻在 CPU 上跑过的纳秒（Linux 的 `/proc/thread-self/schedstat` 第一栏）。
-fn nanoseconds_this_thread_ran_on_a_cpu() -> u64 {
-    let schedstat = std::fs::read_to_string("/proc/thread-self/schedstat")
-        .expect("Linux 上 /proc/thread-self/schedstat 读得出");
-    schedstat
-        .split_whitespace()
-        .next()
-        .expect("第一栏是本线程在 CPU 上跑过的纳秒")
-        .parse()
-        .expect("十进制整数")
+/// 包着一个身份，每被比一次（`==` 或 `cmp`）就在共用的计数上加一：数按身份的索引查一次比了几次身份
+/// （七·6 的增长判据数次数，不读挂钟、不读 CPU 时间；次数只由表与查法定，不随负载与构建档漂移）。
+struct IdentityCountingItsComparisons<'counter, Identity> {
+    identity: Identity,
+    comparisons: &'counter Cell<u64>,
+}
+
+impl<Identity: PartialEq> PartialEq for IdentityCountingItsComparisons<'_, Identity> {
+    fn eq(&self, other: &Self) -> bool {
+        self.comparisons.set(self.comparisons.get() + 1);
+        self.identity == other.identity
+    }
+}
+
+impl<Identity: Eq> Eq for IdentityCountingItsComparisons<'_, Identity> {}
+
+impl<Identity: Ord> PartialOrd for IdentityCountingItsComparisons<'_, Identity> {
+    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
+        Some(self.cmp(other))
+    }
+}
+
+impl<Identity: Ord> Ord for IdentityCountingItsComparisons<'_, Identity> {
+    fn cmp(&self, other: &Self) -> Ordering {
+        self.comparisons.set(self.comparisons.get() + 1);
+        self.identity.cmp(&other.identity)
+    }
+}
+
+/// 查了几次、一共比了几次身份。
+#[derive(Clone, Copy, Debug, Default)]
+struct ComparisonsOfLookups {
+    lookups: u64,
+    comparisons: u64,
+}
+
+impl ComparisonsOfLookups {
+    fn plus(self, other: Self) -> Self {
+        Self {
+            lookups: self.lookups + other.lookups,
+            comparisons: self.comparisons + other.comparisons,
+        }
+    }
+}
+
+/// 在 `table` 上建按身份的索引（`FirstEntryOfEachIdentity`，发布与整理用的同一个），把表里每一项的身份连同一个表里没有的
+/// `absent_identity` 各查一次：表里的查得到、`absent_identity` 查不到。建索引时的比较不算，只数查的那几次。
+fn comparisons_of_looking_up_every_identity<Entry, Identity: Ord + Copy>(
+    table: &[Entry],
+    identity_of: impl Fn(&Entry) -> Identity,
+    absent_identity: Identity,
+) -> ComparisonsOfLookups {
+    let comparisons = Cell::new(0);
+    let index =
+        FirstEntryOfEachIdentity::of_the_table(table, |entry| IdentityCountingItsComparisons {
+            identity: identity_of(entry),
+            comparisons: &comparisons,
+        });
+    comparisons.set(0);
+    let mut lookups = 0;
+    // 迭代上界是表长加一；跨轮携带的只有查了几次。
+    for identity in table.iter().map(&identity_of).chain([absent_identity]) {
+        let found = index.first_entry_of(&IdentityCountingItsComparisons {
+            identity,
+            comparisons: &comparisons,
+        });
+        assert_eq!(
+            found.is_some(),
+            identity != absent_identity,
+            "表里的身份查得到，表里没有的查不到"
+        );
+        lookups += 1;
+    }
+    ComparisonsOfLookups {
+        lookups,
+        comparisons: comparisons.get(),
+    }
+}
+
+/// 表长四倍时，按身份查一次平均比较身份的次数至多涨到几倍（百分数）：有序表查找按层数涨（512 到 2048 多一两层），
+/// 在表上逐项找按表长涨（四倍）；门槛取两者之间、离逐项找远的一边。
+const HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;
+
+/// 两档每次查的平均比较次数之比（百分数）。
+fn growth_in_percent_of_the_comparisons_per_lookup(
+    smaller: ComparisonsOfLookups,
+    larger: ComparisonsOfLookups,
+) -> u64 {
+    larger.comparisons * smaller.lookups * 100 / (smaller.comparisons * larger.lookups)
+}
+
+/// 合成表的两档：512 个与 2048 个不同的身份，按一个与表长互素的步长打乱次序。
+const IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE: u64 = 512;
+const IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE: u64 = 2048;
+/// 打乱合成表次序的步长：素数，与 2 的幂的表长互素，`(位置 × 步长) mod 表长` 走遍每个身份恰好一次。
+const STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE: u64 = 7919;
+
+fn synthetic_table_of_identities(identities: u64) -> Vec<u64> {
+    (0..identities)
+        .map(|position| position * STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE % identities)
+        .collect()
+}
+
+/// 512 个与 2048 个身份的合成表上，按身份的索引每次查平均比较身份的次数涨不到门槛的倍数（数次数，不读挂钟与 CPU 时间）。
+/// 判别力：`crates/mutations.tsv` 里「七·6 按身份查：索引退回逐项找（数比较次数）」那一行。
+#[test]
+fn first_entry_of_each_identity_compares_less_than_twice_as_often_per_lookup_on_a_table_four_times_as_long(
+) {
+    let smaller = comparisons_of_looking_up_every_identity(
+        &synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE),
+        |identity| *identity,
+        u64::MAX,
+    );
+    let larger = comparisons_of_looking_up_every_identity(
+        &synthetic_table_of_identities(IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE),
+        |identity| *identity,
+        u64::MAX,
+    );
+    let growth_in_percent = growth_in_percent_of_the_comparisons_per_lookup(smaller, larger);
+    println!(
+        "publish_lookup_by_identity synthetic_table smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
+    );
+    assert!(
+        growth_in_percent
+            < HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE,
+        "表长四倍，每次查平均比较次数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE}%（小表 {smaller:?}，大表 {larger:?}）"
+    );
 }
 
 /// 增长那一条量的两档：每个文件八个数据单元，512 个与 2048 个数据单元。
@@ -800,42 +1002,41 @@
 const COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS: usize = 8;
 /// 每档量几次（两档轮流，一次调用至多发一版）。
 const MEASURED_CALLS_PER_POOL: usize = 8;
-/// 每档至少要有几次真发了一版才取中位。
+/// 每档至少要有几次真发了一版才算数。
 const PUBLISHING_CALLS_REQUIRED_PER_POOL: usize = 5;
-/// 数据单元四倍时一批整理的线程 CPU 中位至多涨到几倍（百分数）：线性就是四倍，涨过四倍就是比线性快（七·6 要的是「从平方降到接近线性」）。
-/// 依据（2026-10-20 实现员在仓副本里量，debug、本机忙）：改后 50.0 → 154.2 ms，308%；改前的平方查找那几处在 release 下
-/// 512 → 2048 是 430–470%（调查员同一会话轮流量的两次）。
-const HIGHEST_GROWTH_OF_THE_BATCH_CPU_IN_PERCENT_FOR_FOUR_TIMES_THE_DATA_UNITS: u64 = 400;
 
 /// 重挂一份造好的池（批大小强制 K），一次调用一次地推整理：每次调用之前把这次挂载数过的发布数压到上限，一次调用至多发一版
-/// （照 E185 装置 `measure_one_intent`）。交回这一次调用的线程 CPU 纳秒；一版都没发的交回 `None`。
-fn batch_cpu_of_one_compaction_call(
+/// （照 E185 装置 `measure_one_intent`）。调用之前在这一批要读的那一版上数两样：整理读它解了几片映射叶（判恰好是 key 走到的那几片），
+/// `units` 上每个角色按身份查一次比了几次身份。交回比较的次数；一版都没发的交回 `None`。
+fn lookup_work_of_one_compaction_call(
     session: &mut MountedSession,
     devices: &mut Devices,
-) -> Option<u64> {
+) -> Option<ComparisonsOfLookups> {
     session.compaction.publishes_since_the_last_user_change = 1000;
-    let before = nanoseconds_this_thread_ran_on_a_cpu();
+    let version_the_batch_reads = version_of(session);
+    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version_the_batch_reads);
+    let comparisons = comparisons_of_looking_up_every_identity(
+        &version_the_batch_reads.units,
+        |unit| unit.identity,
+        TransactionUnit::Data(DataUnitIndexInFile(u64::MAX)),
+    );
     let run = session.run_compaction(devices).expect("整理一趟");
-    let cpu_nanoseconds = nanoseconds_this_thread_ran_on_a_cpu() - before;
     match run.outcome {
         CompactionRunOutcome::Stopped(_) => {}
         CompactionRunOutcome::Failed(failure) => panic!("整理的一次发布没发成：{failure:?}"),
     }
-    (!run.versions_published.is_empty()).then_some(cpu_nanoseconds)
+    (!run.versions_published.is_empty()).then_some(comparisons)
 }
 
-fn median_of(mut readings: Vec<u64>) -> u64 {
-    readings.sort_unstable();
-    readings[readings.len() / 2]
-}
-
-/// 512 个数据单元的池与 2048 个的池（每个文件八个单元），同一进程里重挂、两档轮流各推 8 次整理（K = 8、一次至多一版），
-/// 一批的线程 CPU 中位从小池到大池涨不到门槛的倍数。按身份查换回在 `Vec` 上逐个找（平方）时涨得过门槛：
-/// `crates/mutations.tsv` 里以「七·6 增长：」起头的几行。
+/// 512 个数据单元的池与 2048 个的池（每个文件八个单元），同一进程里重挂、两档轮流各推 8 次整理（K = 8、一次至多一版）。
+/// 数次数，不读挂钟、不读 CPU 时间：每一批读的那一版上，整理读它解的映射叶恰好是进映射的 key 走到的那几片；`units` 上按角色查一次
+/// 平均比较身份的次数从小池到大池涨不到门槛的倍数。判别力：`crates/mutations.tsv` 里以「七·6 增长：」起头的几行
+/// （按身份查的索引退回逐项找、整理读版本退回逐 key 查映射）。
 // harness-test-granularity:one-scenario 两档池同一个场景：比的是同一串操作在两档上的比，单造一档量不出比。
 #[test]
 #[ignore = "harness 慢档用例：造 2048 个数据单元的池要发约 460 次发布，debug 下单条超过 60 秒"]
-fn a_compaction_batch_cpu_grows_less_than_the_threshold_from_512_to_2048_data_units() {
+fn compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units(
+) {
     let (mut smaller_devices, mut larger_devices) = std::thread::scope(|scope| {
         let smaller = scope.spawn(|| {
             build_a_pool_of_files(
@@ -874,36 +1075,38 @@
     );
     let mut smaller_readings = Vec::new();
     let mut larger_readings = Vec::new();
-    // 迭代上界是 MEASURED_CALLS_PER_POOL；两档轮流，同一时段的负载两档都吃到。
+    // 迭代上界是 MEASURED_CALLS_PER_POOL；两档轮流（与改前量 CPU 时同一个次序）。
     for _ in 0..MEASURED_CALLS_PER_POOL {
-        smaller_readings.extend(batch_cpu_of_one_compaction_call(
+        smaller_readings.extend(lookup_work_of_one_compaction_call(
             &mut smaller_session,
             &mut smaller_devices,
         ));
-        larger_readings.extend(batch_cpu_of_one_compaction_call(
+        larger_readings.extend(lookup_work_of_one_compaction_call(
             &mut larger_session,
             &mut larger_devices,
         ));
     }
     println!(
-        "publish_lookup_by_identity growth smaller_cpu_nanoseconds={smaller_readings:?} larger_cpu_nanoseconds={larger_readings:?}"
+        "publish_lookup_by_identity growth smaller_comparisons={smaller_readings:?} larger_comparisons={larger_readings:?}"
     );
     assert!(
         smaller_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL
             && larger_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL,
         "两档各至少 {PUBLISHING_CALLS_REQUIRED_PER_POOL} 次真发了一版：小池 {smaller_readings:?}，大池 {larger_readings:?}"
     );
-    let smaller_median = median_of(smaller_readings);
-    let larger_median = median_of(larger_readings);
+    let smaller = smaller_readings
+        .into_iter()
+        .fold(ComparisonsOfLookups::default(), ComparisonsOfLookups::plus);
+    let larger = larger_readings
+        .into_iter()
+        .fold(ComparisonsOfLookups::default(), ComparisonsOfLookups::plus);
+    let growth_in_percent = growth_in_percent_of_the_comparisons_per_lookup(smaller, larger);
     println!(
-        "publish_lookup_by_identity growth smaller_median={smaller_median} larger_median={larger_median} growth_in_percent={}",
-        larger_median * 100 / smaller_median
+        "publish_lookup_by_identity growth smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
     );
     assert!(
-        larger_median * 100
-            < HIGHEST_GROWTH_OF_THE_BATCH_CPU_IN_PERCENT_FOR_FOUR_TIMES_THE_DATA_UNITS
-                * smaller_median,
-        "数据单元四倍，一批整理的线程 CPU 中位涨了 {}%，门槛 {HIGHEST_GROWTH_OF_THE_BATCH_CPU_IN_PERCENT_FOR_FOUR_TIMES_THE_DATA_UNITS}%（小池 {smaller_median} ns，大池 {larger_median} ns）",
-        larger_median * 100 / smaller_median
+        growth_in_percent
+            < HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE,
+        "数据单元四倍，units 上按角色查一次平均比较次数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE}%（小池 {smaller:?}，大池 {larger:?}）"
     );
 }
````

## mutations-append.tsv

````text
七·6 整理读版本：隔一把 key 才查映射	crates/singlefs-core/src/mounted_session.rs	        .iter()\n        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	        .iter()\n        .step_by(2)\n        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 整理读版本：漏查第一把 key	crates/singlefs-core/src/mounted_session.rs	        .iter()\n        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	        .iter()\n        .skip(1)\n        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 整理读版本：每把 key 现建一个批量读者	crates/singlefs-core/src/mounted_session.rs	        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	        .filter_map(|(_, key)| match CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version).locations_of_key(key) {	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 按身份查：索引从大到小逐项找	crates/singlefs-core/src/transaction.rs	        self.first_entry_of_each_identity.get(identity).copied()	        self.first_entry_of_each_identity\n            .iter()\n            .rev()\n            .find(|(candidate, _)| *candidate == identity)\n            .map(|(_, entry)| *entry)	-p singlefs-harness --test publish_lookup_by_identity -- --exact first_entry_of_each_identity_compares_less_than_twice_as_often_per_lookup_on_a_table_four_times_as_long	first_entry_of_each_identity_compares_less_than_twice_as_often_per_lookup_on_a_table_four_times_as_long
七·6 按身份查：快照判定那一圈的映射 key 一律取 inode 根那一把	crates/singlefs-core/src/transaction.rs	            .mapping_key_of(*identity)\n            .to_vec();	            .mapping_key_of(TransactionUnit::InodeRoot)\n            .to_vec();	-p singlefs-harness --test snapshot_release_and_destroy_through_the_deadlist -- --exact an_overwrite_under_a_snapshot_keeps_every_replaced_user_visible_unit_on_the_deadlist_and_releases_the_pool_metadata	an_overwrite_under_a_snapshot_keeps_every_replaced_user_visible_unit_on_the_deadlist_and_releases_the_pool_metadata
````

## mutations-replacements.tsv

````text
七·6 增长：按身份查的索引退回逐项找	crates/singlefs-core/src/transaction.rs	        self.first_entry_of_each_identity.get(identity).copied()	        self.first_entry_of_each_identity\n            .iter()\n            .find(|(candidate, _)| *candidate == identity)\n            .map(|(_, entry)| *entry)	-p singlefs-harness --test publish_lookup_by_identity -- --include-ignored --exact compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units	compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units
七·6 增长：整理读版本退回逐 key 查映射	crates/singlefs-core/src/mounted_session.rs	        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	        .filter_map(|(_, key)| match crate::transaction::central_mapping_locations_in_the_version(version, key) {	-p singlefs-harness --test publish_lookup_by_identity -- --include-ignored --exact compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units	compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units
````
