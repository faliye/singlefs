# m4-publish-lookup-code-r3：七·6 第三批补丁原文（crates.patch、mutations-append.tsv 与 mutations-replacements.tsv 原样）（2026-10-21）

## crates.patch

````diff
diff --git a/crates/singlefs-core/Cargo.toml b/crates/singlefs-core/Cargo.toml
--- a/crates/singlefs-core/Cargo.toml
+++ b/crates/singlefs-core/Cargo.toml
@@ -11,3 +11,8 @@
 singlefs-format = { path = "../singlefs-format" }
 # 取 journal 前缀的纯函数核（Verus 验的那一份），recovery::replay_journal 调它；compaction_plan 的单测拿它的整理选段核、计划核对拍。
 singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }
+
+[features]
+# 七·6 各处按身份查的计数（`transaction::lookup_work`：查了几次、每次看过索引几项、建了几次索引、解了几次中央映射叶）：只给测试读，默认关；
+# harness 档的 dev 依赖打开它。不开时计数的几处什么都不做，发布构建不多出开销。
+lookup-work-counters = []
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -61,6 +61,7 @@
 use crate::region_scan::{scan_region, AllocationInRegion, CentralMappingEntry, RegionScan};
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{target_for_publish, RootRingSlot};
+use crate::transaction::lookup_work::{self, LookupSite};
 use crate::transaction::{
     prepare_the_version_publish, publish_compaction_step_in_the_session,
     publish_compaction_step_with_the_region_check_of_the_intent_deletion, publish_create_snapshot,
@@ -2779,22 +2780,18 @@
     slots_holding_user_data: BTreeSet<SlotNumber>,
 }
 
-/// 整理读一版时中央映射那一半（[`central_mapping_as_the_compaction_reads_it`]）。
-pub struct CentralMappingAsTheCompactionReadsIt {
-    /// 每个进映射单元按 key 在这一版的映射树里查到的映射条目，按 `mapped_units` 的次序（查不到、条目窄于字段表的不收），
-    /// 后面接这一版留存的映射条目，按留存的次序。
-    pub mapping_entries: Vec<CentralMappingEntry>,
-    /// 取上面那些条目时批量读者解过几片映射叶（[`CentralMappingOfAVersionReadLeafByLeaf::leaves_decoded`]）。
-    pub mapping_leaves_decoded: usize,
-}
-
 /// 整理读一版时取映射条目的那一步：整理选段与搬单元读版本（`version_as_the_compaction_reads_it`）经它取，别处不调。
-/// 公开给测试：七·6 的用例拿它逐项比逐 key 查（`central_mapping_locations_in_the_version`）的参照、数整理读一版解了几片叶，
-/// 判整理读漏、读错与退回逐 key 查（七·6 代码轮第一轮判决第三节 T4）。
+/// 交回每个进映射单元按 key 在这一版的映射树里查到的映射条目，按 `mapped_units` 的次序（查不到、条目窄于字段表的不收），
+/// 后面接这一版留存的映射条目，按留存的次序。
+/// 公开给测试：七·6 的用例拿它逐项比逐 key 查（`central_mapping_locations_in_the_version`）的参照，判整理读漏、读错；
+/// 它做的查找记在 [`LookupSite::CompactionReadsTheCentralMapping`] 那一处（`transaction::lookup_work`），用例读那一处的
+/// 解叶次数与建索引次数判退回逐 key 查、每把 key 现建读者、每片叶重解（七·6 代码轮第二轮判决第三节）。
 #[must_use]
 pub fn central_mapping_as_the_compaction_reads_it(
     version: &TransactionOutput,
-) -> CentralMappingAsTheCompactionReadsIt {
+) -> Vec<CentralMappingEntry> {
+    let _inside_the_compaction_read =
+        lookup_work::entering(LookupSite::CompactionReadsTheCentralMapping);
     // 每把 key 查一次，叶按片解一次（七·6）：答案与逐 key 查 `central_mapping_locations_in_the_version` 逐个相同。
     let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
     let mut mapping_entries: Vec<CentralMappingEntry> = version
@@ -2815,19 +2812,13 @@
             locations: *locations,
         },
     ));
-    CentralMappingAsTheCompactionReadsIt {
-        mapping_entries,
-        mapping_leaves_decoded: central_mapping.leaves_decoded(),
-    }
+    mapping_entries
 }
 
 fn version_as_the_compaction_reads_it(
     version: &TransactionOutput,
 ) -> VersionAsTheCompactionReadsIt {
-    let CentralMappingAsTheCompactionReadsIt {
-        mapping_entries,
-        mapping_leaves_decoded: _,
-    } = central_mapping_as_the_compaction_reads_it(version);
+    let mapping_entries = central_mapping_as_the_compaction_reads_it(version);
     let mut pinned_slots = BTreeSet::new();
     for unit in &version.units {
         let is_exempt_from_the_mapping = match unit.identity {
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -3250,29 +3250,294 @@
 const EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION: &str =
     "八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在";
 
-/// 一版里一张按身份查的表（`units`、`mapped_units`、`files_other_than_the_first`）的索引：建的时候走一遍整张表，之后每次查是一次
-/// 有序表查找，不再走整张表。发布与整理在一次调用里要对约 P 个身份各查一次，在表上 `iter().find` 是 P 的平方
+/// 七·6 按身份查在产品里实际做了多少事（判决 `research/prompts/m4-publish-lookup-code-r2-main-verification.md` 第三节）：
+/// 在索引上查了几次、每次看过索引里几项、建了几次索引、解了几次中央映射叶，按点名的几处（[`lookup_work::LookupSite`]）分开记。
+/// 数的是产品调用点上的次数，不是测试自己另建的索引；测试读它判增长与「每片叶至多解一次」，不读挂钟与 CPU 时间。
+///
+/// 只在特性 `lookup-work-counters` 下记（harness 档的 dev 依赖打开它）：不开时 [`lookup_work::entering`] 与 [`lookup_work::count`]
+/// 什么都不做，发布构建不多出开销。计数记在本线程上：核心层不起线程，一次发布或一趟整理的查找都在调它的那个线程里做；
+/// 测试的各条用例各在自己的线程里跑，互不串数。
+pub mod lookup_work {
+    #[cfg(feature = "lookup-work-counters")]
+    use std::cell::RefCell;
+    #[cfg(feature = "lookup-work-counters")]
+    use std::collections::BTreeMap;
+
+    /// 七·6 按身份查的几处里，测试要分开判的那几处。封闭集合；不在其中任何一处里做的查找记在「不在点名的几处里」那一格（`None`）。
+    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+    pub enum LookupSite {
+        /// 快照判定那一圈：换下的用户可见角色各取一次映射 key，进 deadlist 的再取映射条目（`deadlist_changes_of_this_publish`）。
+        SnapshotJudgement,
+        /// 经映射释放的角色取映射条目：释放判定路径、核位置项指池里两块不同的盘、释放之前读盘核校验和，三处各取一遍。
+        MappingEntriesOfReleasedUnits,
+        /// 整理读一版取全部进映射单元的映射条目（`mounted_session::central_mapping_as_the_compaction_reads_it`）。
+        CompactionReadsTheCentralMapping,
+        /// 整理发批核计划里每一项的源落点：按映射 key 找角色、取映射条目（`relocation_checked_against_the_version`）。
+        RelocationSources,
+    }
+
+    /// 一处（或几处合起来）做的查找。
+    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+    pub struct LookupWork {
+        /// 建了几次按身份的索引（[`super::FirstEntryOfEachIdentity::of_the_table`]）。
+        pub index_builds: u64,
+        /// 在索引上按身份查了几次。
+        pub index_lookups: u64,
+        /// 查与走索引一共看过索引里几项：有序查一次至多看「索引项数的二进制位数」那么多项；在索引上逐项走，走过一项算一项。
+        pub index_entries_examined: u64,
+        /// 查过的索引里最大那一份有几项（判「每次查至多看几项」用）。
+        pub largest_index_looked_up: u64,
+        /// 解了几次中央映射叶：批量读者每解一片算一次，逐 key 查（`central_mapping_locations_in_the_version`）每查到一片叶算一次。
+        pub mapping_leaf_decodes: u64,
+    }
+
+    impl LookupWork {
+        /// 两份合起来：次数相加，最大的索引取两份里大的那一份。
+        #[must_use]
+        pub fn plus(self, other: Self) -> Self {
+            Self {
+                index_builds: self.index_builds + other.index_builds,
+                index_lookups: self.index_lookups + other.index_lookups,
+                index_entries_examined: self.index_entries_examined + other.index_entries_examined,
+                largest_index_looked_up: self
+                    .largest_index_looked_up
+                    .max(other.largest_index_looked_up),
+                mapping_leaf_decodes: self.mapping_leaf_decodes + other.mapping_leaf_decodes,
+            }
+        }
+    }
+
+    /// 一段代码在点名的每一处做的查找；不在任何一处里做的记在 `None` 下。
+    #[cfg(feature = "lookup-work-counters")]
+    #[derive(Clone, Debug, Default, PartialEq, Eq)]
+    pub struct LookupWorkOfEachSite {
+        pub work_of_each_site: BTreeMap<Option<LookupSite>, LookupWork>,
+    }
+
+    #[cfg(feature = "lookup-work-counters")]
+    impl LookupWorkOfEachSite {
+        /// `site` 那一处做的；一次都没进过那一处就是全 0。
+        #[must_use]
+        pub fn of_the_site(&self, site: LookupSite) -> LookupWork {
+            self.work_of_each_site
+                .get(&Some(site))
+                .copied()
+                .unwrap_or_default()
+        }
+
+        /// 每一处（连同不在点名的几处里的）合起来。
+        #[must_use]
+        pub fn of_every_site(&self) -> LookupWork {
+            self.work_of_each_site
+                .values()
+                .fold(LookupWork::default(), |sum, work| sum.plus(*work))
+        }
+
+        fn add(&mut self, other: &Self) {
+            // 迭代上界是 `other` 里的格数（至多点名的几处加一）。
+            for (site, work) in &other.work_of_each_site {
+                let sum = self.work_of_each_site.entry(*site).or_default();
+                *sum = sum.plus(*work);
+            }
+        }
+    }
+
+    /// 本线程上进了哪几处（里层的在后）与到此刻各处的计数。
+    #[cfg(feature = "lookup-work-counters")]
+    struct LookupWorkOnThisThread {
+        sites_entered: Vec<LookupSite>,
+        work_of_each_site: LookupWorkOfEachSite,
+    }
+
+    #[cfg(feature = "lookup-work-counters")]
+    thread_local! {
+        static LOOKUP_WORK_ON_THIS_THREAD: RefCell<LookupWorkOnThisThread> = const {
+            RefCell::new(LookupWorkOnThisThread {
+                sites_entered: Vec::new(),
+                work_of_each_site: LookupWorkOfEachSite {
+                    work_of_each_site: BTreeMap::new(),
+                },
+            })
+        };
+    }
+
+    /// 跑 `action`，交回它的结果与它在本线程上做的查找。量的时候先把此前的计数挪开、量完加回去：套着量也各量各的。
+    #[cfg(feature = "lookup-work-counters")]
+    pub fn lookup_work_while<Output>(
+        action: impl FnOnce() -> Output,
+    ) -> (Output, LookupWorkOfEachSite) {
+        let counted_before = LOOKUP_WORK_ON_THIS_THREAD
+            .with(|state| std::mem::take(&mut state.borrow_mut().work_of_each_site));
+        let output = action();
+        let counted_while = LOOKUP_WORK_ON_THIS_THREAD.with(|state| {
+            let mut state = state.borrow_mut();
+            let counted_while = std::mem::replace(&mut state.work_of_each_site, counted_before);
+            state.work_of_each_site.add(&counted_while);
+            counted_while
+        });
+        (output, counted_while)
+    }
+
+    /// 进了点名的一处：活着的时候本线程的查找记在这一处，丢掉时退出（里层的先退）。
+    #[must_use = "进了一处要把它留到那一处的查找做完：丢掉就退出了"]
+    pub(crate) struct InsideALookupSite {
+        _constructed_only_by_entering: (),
+    }
+
+    /// 进 `site` 那一处，直到交回的值被丢掉。
+    pub(crate) fn entering(site: LookupSite) -> InsideALookupSite {
+        #[cfg(feature = "lookup-work-counters")]
+        LOOKUP_WORK_ON_THIS_THREAD.with(|state| state.borrow_mut().sites_entered.push(site));
+        #[cfg(not(feature = "lookup-work-counters"))]
+        let _ = site;
+        InsideALookupSite {
+            _constructed_only_by_entering: (),
+        }
+    }
+
+    #[cfg(feature = "lookup-work-counters")]
+    impl Drop for InsideALookupSite {
+        fn drop(&mut self) {
+            LOOKUP_WORK_ON_THIS_THREAD.with(|state| {
+                state.borrow_mut().sites_entered.pop().expect(
+                    "每个 InsideALookupSite 都由 entering 进过一处：退出时栈里至少有它那一处",
+                );
+            });
+        }
+    }
+
+    /// 把 `update` 记到本线程此刻所在的那一处（最里层的；不在任何一处里记在 `None` 下）。
+    pub(crate) fn count(update: impl FnOnce(&mut LookupWork)) {
+        #[cfg(feature = "lookup-work-counters")]
+        LOOKUP_WORK_ON_THIS_THREAD.with(|state| {
+            let mut state = state.borrow_mut();
+            let site = state.sites_entered.last().copied();
+            update(
+                state
+                    .work_of_each_site
+                    .work_of_each_site
+                    .entry(site)
+                    .or_default(),
+            );
+        });
+        #[cfg(not(feature = "lookup-work-counters"))]
+        let _ = update;
+    }
+}
+
+/// 按身份排好序、每个身份只留一项的索引存放：查与走都经这里，各自记看过几项（[`lookup_work`]）。
+/// 放在自己的模块里，外面拿不到那张排好序的表，只能经 `get` 与 `iter`：按身份查与逐项走都逃不过计数。
+mod sorted_identity_index {
+    use super::lookup_work::count;
+
+    pub(super) struct SortedIdentityIndex<Identity, Value> {
+        /// 按身份严格升序，没有两项身份相同。
+        sorted: Vec<(Identity, Value)>,
+    }
+
+    impl<Identity: Ord, Value> SortedIdentityIndex<Identity, Value> {
+        /// `pairs` 按身份稳定排序（同一身份的几项保持进来的次序），同一身份只留进来最先的那一项。
+        pub(super) fn keeping_the_first_value_of_each_identity(
+            mut pairs: Vec<(Identity, Value)>,
+        ) -> Self {
+            pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
+            // `dedup_by` 拿后一项与前一项比，判相同就丢后一项：留下的是稳定排序后每个身份的最先那一项。
+            pairs.dedup_by(|(later, _), (earlier, _)| later == earlier);
+            count(|work| work.index_builds += 1);
+            Self { sorted: pairs }
+        }
+
+        /// 身份是 `identity` 的那一项。折半查：一次至多看 `sorted` 项数的二进制位数那么多项。
+        pub(super) fn get(&self, identity: &Identity) -> Option<&Value> {
+            let mut entries_examined: u64 = 0;
+            let mut low = 0;
+            let mut high = self.sorted.len();
+            // 迭代上界是 `sorted` 项数的二进制位数；跨轮携带的是还没排除的区间 [low, high)，要找的若在表里就在其中。
+            let found = loop {
+                if low >= high {
+                    break None;
+                }
+                let middle = low + (high - low) / 2;
+                entries_examined += 1;
+                let (candidate, value) = &self.sorted[middle];
+                match candidate.cmp(identity) {
+                    std::cmp::Ordering::Less => low = middle + 1,
+                    std::cmp::Ordering::Greater => high = middle,
+                    std::cmp::Ordering::Equal => break Some(value),
+                }
+            };
+            let index_entries = u64::try_from(self.sorted.len()).expect("索引项数装得进 u64");
+            count(|work| {
+                work.index_lookups += 1;
+                work.index_entries_examined += entries_examined;
+                work.largest_index_looked_up = work.largest_index_looked_up.max(index_entries);
+            });
+            found
+        }
+
+        /// 按身份升序逐项走；走过一项算看过一项。
+        pub(super) fn iter(&self) -> WalkOfTheSortedIdentityIndex<'_, Identity, Value> {
+            WalkOfTheSortedIdentityIndex {
+                remaining: self.sorted.iter(),
+            }
+        }
+    }
+
+    /// [`SortedIdentityIndex::iter`]：每交出一项记看过一项（从哪一头走都算）。
+    pub(super) struct WalkOfTheSortedIdentityIndex<'index, Identity, Value> {
+        remaining: std::slice::Iter<'index, (Identity, Value)>,
+    }
+
+    impl<'index, Identity, Value> Iterator for WalkOfTheSortedIdentityIndex<'index, Identity, Value> {
+        type Item = (&'index Identity, &'index Value);
+
+        fn next(&mut self) -> Option<Self::Item> {
+            let (identity, value) = self.remaining.next()?;
+            count(|work| work.index_entries_examined += 1);
+            Some((identity, value))
+        }
+    }
+
+    impl<Identity, Value> DoubleEndedIterator for WalkOfTheSortedIdentityIndex<'_, Identity, Value> {
+        fn next_back(&mut self) -> Option<Self::Item> {
+            let (identity, value) = self.remaining.next_back()?;
+            count(|work| work.index_entries_examined += 1);
+            Some((identity, value))
+        }
+    }
+}
+
+use lookup_work::LookupSite;
+use sorted_identity_index::SortedIdentityIndex;
+
+/// 一版里一张按身份查的表（`units`、`mapped_units`、`files_other_than_the_first`）的索引：建的时候走一遍整张表、按身份排一次序，
+/// 之后每次查是一次折半查找，不再走整张表。发布与整理在一次调用里要对约 P 个身份各查一次，在表上 `iter().find` 是 P 的平方
 /// （里程碑四七·6；量的出处 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 2026-10-20 调查员交回那一条）。
 ///
 /// 同一身份在表里出现几次时取最先那一项，与在表上 `iter().find` 逐项相同：换成索引行为一个字节不变（七·6 的验收）。
 /// 索引借着那张表（`'table`）：索引活着时那一版改不了，「索引与表一致」由借用检查守住；表换了就要重建，不跨版本留着。
+/// 建、查、走各记进 [`lookup_work`]（只在特性 `lookup-work-counters` 下）。
 pub struct FirstEntryOfEachIdentity<'table, Identity, Entry> {
-    first_entry_of_each_identity: BTreeMap<Identity, &'table Entry>,
+    first_entry_of_each_identity: SortedIdentityIndex<Identity, &'table Entry>,
 }
 
 impl<'table, Identity: Ord, Entry> FirstEntryOfEachIdentity<'table, Identity, Entry> {
     /// 走一遍 `table`，每个身份记下最先那一项。
     #[must_use]
-    pub fn of_the_table(table: &'table [Entry], identity_of: impl Fn(&Entry) -> Identity) -> Self {
-        let mut first_entry_of_each_identity = BTreeMap::new();
-        // 迭代上界是表长；跨轮携带的是已经见过的身份，见过的不覆盖（取最先那一项，与 `iter().find` 同义）。
+    pub fn of_the_table(
+        table: &'table [Entry],
+        identity_of: impl Fn(&'table Entry) -> Identity,
+    ) -> Self {
+        let mut identities_with_their_entries = Vec::with_capacity(table.len());
+        // 迭代上界是表长；按表的次序收：同一身份留最先那一项靠的就是这个次序（`keeping_the_first_value_of_each_identity`）。
         for entry in table {
-            first_entry_of_each_identity
-                .entry(identity_of(entry))
-                .or_insert(entry);
+            identities_with_their_entries.push((identity_of(entry), entry));
         }
         Self {
-            first_entry_of_each_identity,
+            first_entry_of_each_identity:
+                SortedIdentityIndex::keeping_the_first_value_of_each_identity(
+                    identities_with_their_entries,
+                ),
         }
     }
 
@@ -3281,6 +3546,13 @@
     pub fn first_entry_of(&self, identity: &Identity) -> Option<&'table Entry> {
         self.first_entry_of_each_identity.get(identity).copied()
     }
+
+    /// 索引里的每个身份与它在表里的最先那一项，按身份升序：表里每个不同的身份恰好一项。走一项记看过一项。
+    pub fn first_entries(&self) -> impl DoubleEndedIterator<Item = (&Identity, &'table Entry)> {
+        self.first_entry_of_each_identity
+            .iter()
+            .map(|(identity, entry)| (identity, *entry))
+    }
 }
 
 impl<'table> FirstEntryOfEachIdentity<'table, TransactionUnit, PublishedUnit> {
@@ -3799,6 +4071,15 @@
         FirstEntryOfEachIdentity::of_the_table(&self.mapped_units, |(identity, _)| *identity)
     }
 
+    /// `mapped_units` 按映射 key 的索引（整理发批按计划里的映射 key 找它在这一版里的角色，七·6）：同一把 key 取最先那一项，
+    /// 与在 `mapped_units` 上按 key `iter().find` 同一项。
+    #[must_use]
+    pub fn mapped_units_by_key(
+        &self,
+    ) -> FirstEntryOfEachIdentity<'_, &[u8], (TransactionUnit, Vec<u8>)> {
+        FirstEntryOfEachIdentity::of_the_table(&self.mapped_units, |(_, key)| key.as_slice())
+    }
+
     /// `files_other_than_the_first` 按 inode 号的索引（[`TransactionOutput::file_other_than_the_first`] 的批量形态，七·6）。
     #[must_use]
     pub fn files_other_than_the_first_by_inode(
@@ -4018,7 +4299,10 @@
     roles: &[TransactionUnit],
     kept_for_the_snapshots: &[TransactionUnit],
 ) -> Result<Vec<Placement>, PublishError> {
+    let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);
     let mut placements = Vec::new();
+    // 换下的角色逐个取上一版的映射条目：key 与落点都走建一次的索引（七·6），不逐个在 `mapped_units` 上找、逐 key 重解叶。
+    let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
     for identity in roles_replaced_via_mapping(previous, roles, file_content)
         .into_iter()
         .filter(|identity| !kept_for_the_snapshots.contains(identity))
@@ -4052,7 +4336,7 @@
                 if !has_a_placement_in_the_previous_version(previous, identity) {
                     continue;
                 }
-                match mapping_locations_of_a_mapped_unit(previous, identity) {
+                match previous_read_by_role.locations_of_role(identity) {
                     MappingLookup::Found(locations) => locations,
                     MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
                         return Err(PublishError::ReleaseNotInMapping { unit: identity })
@@ -4281,20 +4565,55 @@
     }
 }
 
-/// 一个进映射的角色在上一版映射节点里的那条映射条目（按上一版记的 key 查）。
-///
-/// # Panics
-/// `identity` 不是进映射的角色（映射树、树表、实例表豁免映射）：调用方按角色分过。
-fn mapping_locations_of_a_mapped_unit(
-    previous: &TransactionOutput,
-    identity: TransactionUnit,
-) -> MappingLookup {
-    let (_, key) = previous
-        .mapped_units
-        .iter()
-        .find(|(mapped, _)| *mapped == identity)
-        .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");
-    central_mapping_locations_in_the_version(previous, key)
+/// 一版里进映射的角色按角色取那条映射条目的批量形态（七·6，用户 2026-10-21 定并进）：每个角色的答案与「在 `mapped_units` 上按角色
+/// `iter().find` 取最先那一把 key、再 [`central_mapping_locations_in_the_version`]」逐个相同，但 key 走 `mapped_units` 按角色的索引、
+/// 落点走批量读者（每片叶只解一次）。释放判定路径、核位置项、释放前读盘核校验和、快照判定那一圈进 deadlist 的那一支各对一串角色逐个查，
+/// 逐个在表上找、逐 key 重解叶是角色数乘表长。两个索引在第一次查时才建：一个角色都不查的不建。
+pub struct MappedUnitsOfAVersionReadByRole<'version> {
+    version: &'version TransactionOutput,
+    indexes_built_at_the_first_lookup: Option<IndexesOfAVersionReadByRole<'version>>,
+}
+
+/// [`MappedUnitsOfAVersionReadByRole`] 第一次查时建的两样。
+struct IndexesOfAVersionReadByRole<'version> {
+    /// `mapped_units` 按角色的索引：取角色的映射 key。
+    mapped_units_by_role:
+        FirstEntryOfEachIdentity<'version, TransactionUnit, (TransactionUnit, Vec<u8>)>,
+    /// 中央映射的批量读者：按 key 取映射条目，每片叶只解一次。
+    central_mapping: CentralMappingOfAVersionReadLeafByLeaf<'version>,
+}
+
+impl<'version> MappedUnitsOfAVersionReadByRole<'version> {
+    /// 还什么都没建。
+    #[must_use]
+    pub fn of_the_version(version: &'version TransactionOutput) -> Self {
+        Self {
+            version,
+            indexes_built_at_the_first_lookup: None,
+        }
+    }
+
+    /// 进映射的角色 `identity` 在这一版映射节点里的那条映射条目（按这一版记的 key 查）。
+    ///
+    /// # Panics
+    /// `identity` 不是这一版里进映射的角色（映射树、树表、实例表豁免映射）：调用方按角色分过。
+    /// key 走到的那片叶的角色不在这一版的 `units` 里（同 [`central_mapping_locations_in_the_version`]）。
+    pub fn locations_of_role(&mut self, identity: TransactionUnit) -> MappingLookup {
+        let version = self.version;
+        let IndexesOfAVersionReadByRole {
+            mapped_units_by_role,
+            central_mapping,
+        } = self
+            .indexes_built_at_the_first_lookup
+            .get_or_insert_with(|| IndexesOfAVersionReadByRole {
+                mapped_units_by_role: version.mapped_units_by_role(),
+                central_mapping: CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version),
+            });
+        let (_, key) = mapped_units_by_role
+            .first_entry_of(&identity)
+            .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");
+        central_mapping.locations_of_key(key)
+    }
 }
 
 /// 在一版的中央映射树里按 key 查落点：按这一版的形状从根沿分隔 key 走到那片叶（D8（核心索引结构） 已定项 11：
@@ -4313,6 +4632,7 @@
         return MappingLookup::NodeMalformedOrNoEntryWithThisKey;
     };
     let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, shape);
+    lookup_work::count(|work| work.mapping_leaf_decodes += 1);
     mapping_locations_for_key(&version.unit(leaf_role).bytes, key)
 }
 
@@ -4320,6 +4640,7 @@
 /// （走到哪片叶、叶解不开、叶里有切不动的条目、同一把 key 有几条取最后一条，几种情形都同），但每片叶只解一次、叶的单元按角色从索引取。
 /// 整理读一版时对全部进映射单元的 key 各查一次，逐 key 查是每把 key 线性找叶再整片重解（里程碑四七·6）。
 /// 一片叶在第一次有 key 走到它时才解：没有 key 走到的叶不碰，与逐 key 查一样（那片叶的角色不在 `units` 里时同样 panic）。
+/// 每解一片记一次解叶（[`lookup_work`]）：测试读的是解了几次，不是缓存里有几片。
 pub struct CentralMappingOfAVersionReadLeafByLeaf<'version> {
     shape: &'version CodeTwoTreeShape,
     units_by_role: FirstEntryOfEachIdentity<'version, TransactionUnit, PublishedUnit>,
@@ -4339,6 +4660,7 @@
 
 impl CentralMappingLeafRead {
     fn of_the_leaf_bytes(mapping_node_bytes: &[u8]) -> Self {
+        lookup_work::count(|work| work.mapping_leaf_decodes += 1);
         let Ok(node) = parse_index_node(mapping_node_bytes) else {
             return Self::NodeMalformed;
         };
@@ -4404,14 +4726,6 @@
             })
             .locations_of_key(key)
     }
-
-    /// 到此刻解过几片叶：每片叶在第一次有 key 走到它时解一次、之后不再解，所以至多是这一版中央映射树的节点数。
-    /// 测试拿它判「整理读一版每片叶至多解一次、每把 key 都经这个读者查」（七·6 的增长判据数次数，不读挂钟与 CPU 时间）；
-    /// 读的是已有的那张表的长度，不另加计数。
-    #[must_use]
-    pub fn leaves_decoded(&self) -> usize {
-        self.leaves_read.len()
-    }
 }
 
 /// 经映射释放的角色（数据、extent 根、inode 叶容器与根、分配记录树、记账树）：这次重写的角色里走得到映射条目的那几个，按角色次序。
@@ -4468,10 +4782,10 @@
 /// 在上一版的映射里查不到：调用方先走 `placements_to_release_via_mapping`、它按同一个上一版、同一串角色查过，
 /// 查不到时它已经报了 `ReleaseNotInMapping` 或 `MappingEntryNarrowerThanItsFieldTable`，走不到这里。
 fn mapping_locations_of_a_released_unit(
-    previous: &TransactionOutput,
+    previous_read_by_role: &mut MappedUnitsOfAVersionReadByRole<'_>,
     identity: TransactionUnit,
 ) -> [LocationEntry; 2] {
-    match mapping_locations_of_a_mapped_unit(previous, identity) {
+    match previous_read_by_role.locations_of_role(identity) {
         MappingLookup::Found(locations) => locations,
         MappingLookup::NodeMalformedOrNoEntryWithThisKey
         | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => panic!(
@@ -4499,10 +4813,13 @@
     pool_devices: &[DeviceIdentity],
     file_content: FileContentOfThePublish,
 ) -> Result<(), PublishError> {
+    let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);
+    // 同释放判定路径：key 与落点都走建一次的索引（七·6）。
+    let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
     for identity in
         mapped_roles_released_by_this_publish(previous, roles, kept_for_the_snapshots, file_content)
     {
-        let locations = mapping_locations_of_a_released_unit(previous, identity);
+        let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);
         refuse_locations_that_do_not_name_two_pool_devices(identity, &locations, pool_devices)?;
     }
     Ok(())
@@ -4571,11 +4888,14 @@
         &pool_devices,
         file_content,
     )?;
+    let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);
     let mut quarantined = Vec::new();
+    // 同释放判定路径：key 与落点都走建一次的索引（七·6）。
+    let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);
     for identity in
         mapped_roles_released_by_this_publish(previous, roles, kept_for_the_snapshots, file_content)
     {
-        let locations = mapping_locations_of_a_released_unit(previous, identity);
+        let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);
         quarantined.extend(
             copies_of_a_released_unit_failing_the_release_checksum_check(
                 identity, &locations, reader,
@@ -5713,6 +6033,7 @@
     let Some(previous_version) = previous else {
         return Ok(DeadlistChangesOfThisPublish::default());
     };
+    let _inside_the_snapshot_judgement = lookup_work::entering(LookupSite::SnapshotJudgement);
     let user_visible_roles_replaced: Vec<TransactionUnit> =
         roles_replaced_via_mapping(previous_version, candidate_roles, file_content)
             .into_iter()
@@ -5724,6 +6045,9 @@
     let mut user_visible_roles_kept_for_the_snapshots = Vec::new();
     let mut entries_killed_this_publish: Vec<DeadlistEntry> = Vec::new();
     let mut mapping_entries_kept_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = Vec::new();
+    // 进 deadlist 的角色取映射条目：同释放判定路径，key 与落点都走建一次的索引（七·6，用户 2026-10-21 定并进），没有进 deadlist 的不建。
+    let mut previous_version_read_by_role =
+        MappedUnitsOfAVersionReadByRole::of_the_version(previous_version);
     // 换下的角色各查一次映射 key，查法与照抄全部角色那一圈同一个索引（七·6）：建一次走完上一版的 `mapped_units`，
     // 之后每次查不再走整张表；同一角色取最先那一项、查不到时的 panic 文案与在 `mapped_units` 上 `iter().find` 相同。
     let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();
@@ -5744,21 +6068,19 @@
                     CheckpointTxg(0),
                     "进 deadlist 的条目纪元非 0：活头没打过快照时没有单元该进 deadlist（只供测试的强制进 deadlist 那一臂要在活头打过快照之后用）；单元 {identity:?}"
                 );
-                let locations =
-                    match mapping_locations_of_a_mapped_unit(previous_version, *identity) {
-                        MappingLookup::Found(locations) => locations,
-                        MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
-                            return Err(PublishError::ReleaseNotInMapping { unit: *identity })
-                        }
-                        MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
-                            return Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
-                                unit: *identity,
-                                entry_bytes,
-                                field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES)
-                                    .expect("55"),
-                            })
-                        }
-                    };
+                let locations = match previous_version_read_by_role.locations_of_role(*identity) {
+                    MappingLookup::Found(locations) => locations,
+                    MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
+                        return Err(PublishError::ReleaseNotInMapping { unit: *identity })
+                    }
+                    MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
+                        return Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
+                            unit: *identity,
+                            entry_bytes,
+                            field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
+                        })
+                    }
+                };
                 user_visible_roles_kept_for_the_snapshots.push(*identity);
                 entries_killed_this_publish.push(DeadlistEntry {
                     head: trees.inode,
@@ -9364,12 +9686,21 @@
         ),
     };
     let mut relocations = Vec::with_capacity(planned.len());
-    for planned_relocation in planned {
-        relocations.push(relocation_checked_against_the_version(
-            &*pool.devices,
-            previous,
-            planned_relocation,
-        )?);
+    {
+        let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);
+        // 计划里每一项按映射 key 查一次上一版：按 key 的索引与批量读者在第一项要查时建一次（七·6，用户 2026-10-21 定并进），
+        // 不逐项在 `mapped_units` 上找、逐 key 重解叶。
+        let mut previous_read_by_key = None;
+        for planned_relocation in planned {
+            relocations.push(relocation_checked_against_the_version(
+                &*pool.devices,
+                previous,
+                previous_read_by_key.get_or_insert_with(|| {
+                    MappedUnitsOfAVersionReadByKey::of_the_version(previous)
+                }),
+                planned_relocation,
+            )?);
+        }
     }
     publish_compaction_step_after_its_check(
         pool,
@@ -9531,24 +9862,45 @@
     }
 }
 
+/// 整理发批核计划时按映射 key 查上一版的两样（[`relocation_checked_against_the_version`]）：`mapped_units` 按 key 的索引
+/// （同一把 key 取最先那一项，同在表上按 key `iter().find`），与中央映射的批量读者（答案同逐 key 查 [`central_mapping_locations_in_the_version`]）。
+struct MappedUnitsOfAVersionReadByKey<'version> {
+    mapped_units_by_key:
+        FirstEntryOfEachIdentity<'version, &'version [u8], (TransactionUnit, Vec<u8>)>,
+    central_mapping: CentralMappingOfAVersionReadLeafByLeaf<'version>,
+}
+
+impl<'version> MappedUnitsOfAVersionReadByKey<'version> {
+    fn of_the_version(version: &'version TransactionOutput) -> Self {
+        Self {
+            mapped_units_by_key: version.mapped_units_by_key(),
+            central_mapping: CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version),
+        }
+    }
+}
+
 /// 计划里的一项对着上一版核过、带上搬它要的东西（[`publish_relocation_batch`]）：它是这一版进映射的单元（取它的角色与内存里的字节），
 /// 或是留存的映射条目（字节从盘上按映射位置读、核位置项里的校验和，两份取先核得上的那一份）；源落点要就是映射位置、跨度合它的类。
+/// 按 key 找角色与取映射条目走 `previous_read_by_key`（它就是 `previous` 的索引与批量读者）。
 ///
 /// # Errors
 /// [`RelocationBatchRefused`] 的四种计划错。
 fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     previous: &TransactionOutput,
+    previous_read_by_key: &mut MappedUnitsOfAVersionReadByKey<'_>,
     planned: &PlannedRelocation,
 ) -> Result<RelocationOfThePublish, RelocationBatchRefused> {
     let mapping_key = planned.mapping_key.clone();
-    let holder_and_locations = match previous
-        .mapped_units
-        .iter()
-        .find(|(_, key)| *key == mapping_key)
+    let holder_and_locations = match previous_read_by_key
+        .mapped_units_by_key
+        .first_entry_of(&mapping_key.as_slice())
     {
         Some((role, _)) => {
-            let locations = match central_mapping_locations_in_the_version(previous, &mapping_key) {
+            let locations = match previous_read_by_key
+                .central_mapping
+                .locations_of_key(&mapping_key)
+            {
                 MappingLookup::Found(locations) => Some(locations),
                 MappingLookup::NodeMalformedOrNoEntryWithThisKey
                 | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
diff --git a/crates/singlefs-harness/Cargo.toml b/crates/singlefs-harness/Cargo.toml
--- a/crates/singlefs-harness/Cargo.toml
+++ b/crates/singlefs-harness/Cargo.toml
@@ -13,3 +13,6 @@
 # （门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 ⑤⑥⑦）。
 singlefs-reference-filesystem = { path = "../singlefs-reference-filesystem" }
 
+[dev-dependencies]
+# 七·6 的用例读核心层各处按身份查的计数（`singlefs_core::transaction::lookup_work`），只在测试构建打开（核心层的特性默认关）。
+singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters"] }
diff --git a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
--- a/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
+++ b/crates/singlefs-harness/tests/publish_lookup_by_identity.rs
@@ -7,16 +7,18 @@
 //!   映射叶解不开、条目窄于字段表、同一把 key 有两条，三种叶上批量读与逐 key 查逐项相同。整理读每一版取的映射条目
 //!   （`mounted_session::central_mapping_as_the_compaction_reads_it`）与逐 key 查的参照逐项相同（不漏、不错、次序同），批量读者解的叶
 //!   恰好是进映射的 key 走到的那几片。
-//! - 增长：数次数，不读挂钟、不读 CPU 时间（七·6 代码轮第一轮判决第三节 T4）。按身份查一次比几次身份：表长四倍时每次查平均比较次数
-//!   涨不到门槛的倍数（合成表一条快档用例；512 与 2048 个数据单元的两档池上整理读的那几版，harness 慢档用例）；整理读一版解几片映射叶：
-//!   恰好是进映射的 key 走到的那几片（每一版都判，慢档用例的两档池上也判）。
+//! - 数的是产品调用点上做的事（七·6 代码轮第二轮判决第三节）：核心层在特性 `lookup-work-counters` 下按处记下按身份查了几次、每次看过
+//!   索引几项、建了几次索引、解了几次中央映射叶（`singlefs_core::transaction::lookup_work`，harness 档的 dev 依赖打开这个特性），
+//!   用例只读这些计数，不在测试里另建索引来数，不读挂钟、不读 CPU 时间。每一次用户改动、每一趟整理：每次查至多看折半查找那么多项；
+//!   整份重写一个文件时快照判定那一圈与释放那三处都经索引查了换下的每个数据单元；整理每发一批，计划里每一项都经按 key 的索引查过；
+//!   整理读一版恰好解进映射的 key 走到的那几片叶、各解一次，只建一次单元索引。
+//! - 增长：表长四倍时每次查平均看过的索引项数涨不到门槛的倍数（合成表一条快档用例；512 与 2048 个数据单元的两档池上每一趟整理，
+//!   harness 慢档用例），整理每一趟建索引的次数也涨不到门槛的倍数。
 //!
 //! 池都在内存里的两块 4 GiB 稀疏盘上造（`SparseBlockDevice`），配方照 E185 装置的 `construct_the_pool`：每个文件整份写进去，
 //! 再整份重写下标不是 4 的倍数的那几个，抬 F 到换下的落点都回收，留下每段约四分之一还活着的聚簇段。
 mod common;
 
-use std::cell::Cell;
-use std::cmp::Ordering;
 use std::collections::BTreeSet;
 
 use singlefs_checker::image::InvariantVerdict;
@@ -38,14 +40,19 @@
     mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
 };
 use singlefs_core::mounted_session::{
-    central_mapping_as_the_compaction_reads_it, CompactionRunOutcome, MountedSession, UserChange,
+    central_mapping_as_the_compaction_reads_it, CompactionRun, CompactionRunOutcome,
+    MountedSession, UserChange,
 };
 use singlefs_core::records::parse_mapping_entry;
 use singlefs_core::region_scan::CentralMappingEntry;
+use singlefs_core::transaction::lookup_work::{
+    lookup_work_while, LookupSite, LookupWork, LookupWorkOfEachSite,
+};
 use singlefs_core::transaction::{
     acquire_instance, central_mapping_locations_in_the_version, publish_first_file, warm_up,
-    CentralMappingOfAVersionReadLeafByLeaf, FirstEntryOfEachIdentity, FirstFile, MappingLookup,
-    PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
+    CentralMappingOfAVersionReadLeafByLeaf, FirstEntryOfEachIdentity, FirstFile,
+    MappedUnitsOfAVersionReadByRole, MappingLookup, PoolVersion, PoolWriter, TransactionOutput,
+    TransactionUnit,
 };
 use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
@@ -194,16 +201,72 @@
 /// 造池那一串操作里每一版都交给它看一眼（等价那两条交比对，增长那一条什么都不做）。
 type OnEveryVersion<'callback> = dyn FnMut(&TransactionOutput) + 'callback;
 
-/// 一次用户改动发成，把这次发成的每一版（连同改动之后的那一趟整理）交给 `on_every_version`。
+/// 一个数的二进制位数：折半查一份这么多项的索引，一次至多看这么多项。
+fn binary_digits_of(entries: u64) -> u64 {
+    u64::from(u64::BITS - entries.leading_zeros())
+}
+
+/// 每一处（连同不在点名几处里的）按身份查了几次，看过的索引项数不超过「每次查都是一次折半查找」那么多：
+/// 在索引上逐项走（退回线性找、查之前先把索引走一遍）看过的项数按表长涨，查的次数不涨，过不了这一条。
+fn assert_every_lookup_examines_at_most_a_binary_search(
+    lookup_work: &LookupWorkOfEachSite,
+    what_was_done: &str,
+) {
+    // 迭代上界是点名的几处加一。
+    for (site, work) in &lookup_work.work_of_each_site {
+        let examined_at_most = work.index_lookups * binary_digits_of(work.largest_index_looked_up);
+        assert!(
+            work.index_entries_examined <= examined_at_most,
+            "{what_was_done}：{site:?} 那一处按身份查了 {} 次、看过索引 {} 项，每次折半查至多看 {} 项（最大的索引 {} 项），合计至多 {examined_at_most} 项：{work:?}",
+            work.index_lookups,
+            work.index_entries_examined,
+            binary_digits_of(work.largest_index_looked_up),
+            work.largest_index_looked_up
+        );
+    }
+}
+
+/// 整理推一趟（量它在产品里做的按身份查）：每次查至多一次折半查找；每发一批，计划里每一项都经按 key 的索引查过
+/// （整理发批核源落点退回在 `mapped_units` 上按 key 线性找，这一处一次都不查）。交回这一趟与它做的查找。
+fn run_compaction_and_count_its_lookups(
+    session: &mut MountedSession,
+    devices: &mut Devices,
+) -> (CompactionRun, LookupWorkOfEachSite) {
+    let (run, lookup_work) = lookup_work_while(|| session.run_compaction(devices));
+    let run = run.expect("整理一趟");
+    assert_every_lookup_examines_at_most_a_binary_search(&lookup_work, "整理一趟");
+    let units_relocated: u64 = run
+        .versions_published
+        .iter()
+        .map(|version| {
+            u64::try_from(version.units_relocated_by_this_publish.len()).expect("单元数")
+        })
+        .sum();
+    let relocation_sources = lookup_work.of_the_site(LookupSite::RelocationSources);
+    assert!(
+        relocation_sources.index_lookups >= units_relocated,
+        "整理这一趟搬了 {units_relocated} 个单元，核源落点那一处只经索引查了 {} 次（计划里每一项按映射 key 查一次）：{relocation_sources:?}",
+        relocation_sources.index_lookups
+    );
+    (run, lookup_work)
+}
+
+/// 一次用户改动发成，把这次发成的每一版（连同改动之后的那一趟整理）交给 `on_every_version`；交回这次发布在产品里做的按身份查
+/// （每次查至多一次折半查找，这里就判）。量的只是发布那一次调用，`on_every_version` 里测试自己做的查找不算进去。
 fn publish_and_look_at_every_version(
     session: &mut MountedSession,
     devices: &mut Devices,
     change: UserChange<'_>,
     on_every_version: &mut OnEveryVersion<'_>,
-) {
-    let published = session
-        .publish_user_change(devices, change)
-        .unwrap_or_else(|refusal| panic!("用户改动 {change:?} 发成：{refusal:?}"));
+) -> LookupWorkOfEachSite {
+    let (published, lookup_work) =
+        lookup_work_while(|| session.publish_user_change(devices, change));
+    let published =
+        published.unwrap_or_else(|refusal| panic!("用户改动 {change:?} 发成：{refusal:?}"));
+    assert_every_lookup_examines_at_most_a_binary_search(
+        &lookup_work,
+        &format!("用户改动 {change:?}"),
+    );
     for version in &published.versions_published {
         on_every_version(version);
     }
@@ -212,6 +275,7 @@
             on_every_version(version);
         }
     }
+    lookup_work
 }
 
 /// 一个池的样子：哪几个文件、每个几个数据单元。
@@ -273,7 +337,7 @@
         replaced_slots.extend(data_slots_of_the_file(version_of(&session), *inode));
         let content = content_of_data_units(shape.data_units_per_file, inode.0 + 1_000_003);
         write_time_seconds += 1;
-        publish_and_look_at_every_version(
+        let lookup_work = publish_and_look_at_every_version(
             &mut session,
             &mut devices,
             UserChange::SequentialWriteOfAnInode {
@@ -283,6 +347,21 @@
             },
             on_every_version,
         );
+        // 整份重写换下这个文件的每个数据单元：快照判定那一圈逐个取映射 key，释放的三处逐个取映射条目，都经索引。
+        let data_units_replaced =
+            u64::try_from(shape.data_units_per_file).expect("一个文件的数据单元数装得进 u64");
+        for site in [
+            LookupSite::SnapshotJudgement,
+            LookupSite::MappingEntriesOfReleasedUnits,
+        ] {
+            let work = lookup_work.of_the_site(site);
+            assert!(
+                work.index_lookups >= data_units_replaced,
+                "整份重写 inode {} 换下 {data_units_replaced} 个数据单元，{site:?} 那一处只经索引查了 {} 次（换下的每个单元经索引查一次）：{work:?}",
+                inode.0,
+                work.index_lookups
+            );
+        }
     }
     // 迭代上界是 FLOOR_RAISES_AT_MOST；跨轮携带的是会话（每一轮抬一次 F，不够就多发一次建 inode 的发布再抬）。
     let mut is_every_replaced_slot_reclaimed = false;
@@ -391,18 +470,32 @@
         .collect()
 }
 
-/// 整理读这一版时批量读者解过的叶恰好是进映射的 key 走到的那几片（少了是有 key 没经它查，例如退回逐 key 查；
-/// 读者按叶记、一片只解一次）。
+/// 整理读这一版（`central_mapping_as_the_compaction_reads_it`）在产品里解叶的次数恰好是进映射的 key 走到的叶的片数：
+/// 每片只解一次（同一片叶每把 key 重解、退回逐 key 查都多解），只建一次单元索引（每把 key 现建一个批量读者就建了 key 那么多次），
+/// 每次查至多一次折半查找。
 fn assert_the_compaction_read_decodes_exactly_the_routed_leaves(version: &TransactionOutput) {
-    let leaves_decoded = central_mapping_as_the_compaction_reads_it(version).mapping_leaves_decoded;
-    let routed_leaves = mapping_leaves_the_mapped_keys_route_to(version).len();
+    let (_, lookup_work) =
+        lookup_work_while(|| central_mapping_as_the_compaction_reads_it(version));
+    let compaction_read = lookup_work.of_the_site(LookupSite::CompactionReadsTheCentralMapping);
+    let routed_leaves =
+        u64::try_from(mapping_leaves_the_mapped_keys_route_to(version).len()).expect("叶的片数");
     assert_eq!(
-        leaves_decoded,
+        compaction_read.mapping_leaf_decodes,
         routed_leaves,
-        "txg {}：整理读这一版解了 {leaves_decoded} 片映射叶，进映射的 {} 把 key 走到 {routed_leaves} 片（每片解一次、每把 key 都经批量读者查）",
+        "txg {}：整理读这一版解了 {} 次映射叶，进映射的 {} 把 key 走到 {routed_leaves} 片（每片解一次、每把 key 都经批量读者查）：{compaction_read:?}",
         version.root.checkpoint_txg.0,
+        compaction_read.mapping_leaf_decodes,
         version.mapped_units.len()
     );
+    assert_eq!(
+        compaction_read.index_builds, 1,
+        "txg {}：整理读这一版建了 {} 次索引（一个批量读者、一份单元索引）：{compaction_read:?}",
+        version.root.checkpoint_txg.0, compaction_read.index_builds
+    );
+    assert_every_lookup_examines_at_most_a_binary_search(
+        &lookup_work,
+        &format!("txg {} 整理读这一版", version.root.checkpoint_txg.0),
+    );
 }
 
 /// 在两个 `Option<&Entry>` 上判「是表里同一项（或都没有）」：比地址，不比值（取的必须是最先那一项，不只是值相等的一项）。
@@ -414,6 +507,37 @@
     }
 }
 
+/// 索引按身份升序交出的每一项都是表里那个身份的最先那一项（比地址），表里每个不同的身份恰好出现一次。
+fn assert_the_index_lists_the_first_entry_of_every_identity_once<'table, Identity, Entry>(
+    index: &FirstEntryOfEachIdentity<'table, Identity, Entry>,
+    table: &'table [Entry],
+    identity_of: impl Fn(&'table Entry) -> Identity,
+    which_index: &str,
+) where
+    Identity: Ord + std::fmt::Debug,
+{
+    let mut identities_listed = Vec::new();
+    // 迭代上界是索引项数。
+    for (identity, entry) in index.first_entries() {
+        let found_linearly = table
+            .iter()
+            .find(|candidate| identity_of(*candidate) == *identity);
+        assert!(
+            is_the_same_entry(Some(entry), found_linearly),
+            "{which_index}：索引里身份 {identity:?} 那一项不是表里它的最先那一项"
+        );
+        identities_listed.push(identity);
+    }
+    let distinct_identities: BTreeSet<Identity> = table.iter().map(&identity_of).collect();
+    assert!(
+        identities_listed.windows(2).all(|pair| pair[0] < pair[1])
+            && identities_listed.len() == distinct_identities.len(),
+        "{which_index}：索引按身份升序列出表里 {} 个不同的身份各一次，列出了 {} 项",
+        distinct_identities.len(),
+        identities_listed.len()
+    );
+}
+
 /// 一版里每一种按身份查都与线性查找参照（`iter().find`、逐 key 的 `central_mapping_locations_in_the_version`）比一遍：表里的每个身份，
 /// 再加一个哪一版都没有的身份；整理读这一版取的映射条目与逐 key 查的参照逐项比，批量读者解的叶数与 key 走到的叶数比。
 fn compare_every_lookup_with_a_linear_find(
@@ -437,6 +561,12 @@
         );
         compared.unit_lookups += 1;
     }
+    assert_the_index_lists_the_first_entry_of_every_identity_once(
+        &units_by_role,
+        &version.units,
+        |unit| unit.identity,
+        &format!("txg {} 的 units 按角色", version.root.checkpoint_txg.0),
+    );
     let mapped_units_by_role = version.mapped_units_by_role();
     for role in version
         .mapped_units
@@ -463,6 +593,63 @@
         }
         compared.mapping_key_lookups += 1;
     }
+    assert_the_index_lists_the_first_entry_of_every_identity_once(
+        &mapped_units_by_role,
+        &version.mapped_units,
+        |(role, _)| *role,
+        &format!(
+            "txg {} 的 mapped_units 按角色",
+            version.root.checkpoint_txg.0
+        ),
+    );
+    // 整理发批按映射 key 找角色走的索引：每一把 key（连同一把哪一版都没有的）查到的与在 `mapped_units` 上按 key 线性找的是同一项。
+    let mapped_units_by_key = version.mapped_units_by_key();
+    let absent_key = vec![0xff_u8; MAPPING_KEY_BYTES];
+    for key in version
+        .mapped_units
+        .iter()
+        .map(|(_, key)| key)
+        .chain([&absent_key])
+    {
+        let found_linearly = version
+            .mapped_units
+            .iter()
+            .find(|(_, mapped_key)| mapped_key == key);
+        assert!(
+            is_the_same_entry(
+                mapped_units_by_key.first_entry_of(&key.as_slice()),
+                found_linearly
+            ),
+            "txg {}：mapped_units 按 key {key:02x?} 查到的与线性找的不是同一项",
+            version.root.checkpoint_txg.0
+        );
+        compared.mapping_key_lookups += 1;
+    }
+    assert_the_index_lists_the_first_entry_of_every_identity_once(
+        &mapped_units_by_key,
+        &version.mapped_units,
+        |(_, key)| key.as_slice(),
+        &format!(
+            "txg {} 的 mapped_units 按 key",
+            version.root.checkpoint_txg.0
+        ),
+    );
+    // 释放的三处与快照判定那一圈进 deadlist 那一支按角色取映射条目走的批量形态：每个进映射的角色与「线性找 key、逐 key 查」逐项相同。
+    let mut read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(version);
+    for (role, _) in &version.mapped_units {
+        let (_, key_found_linearly) = version
+            .mapped_units
+            .iter()
+            .find(|(mapped, _)| mapped == role)
+            .expect("角色就取自 mapped_units");
+        assert_eq!(
+            read_by_role.locations_of_role(*role),
+            central_mapping_locations_in_the_version(version, key_found_linearly),
+            "txg {}：角色 {role:?} 按角色批量取的映射条目与线性找 key、逐 key 查的不同",
+            version.root.checkpoint_txg.0
+        );
+        compared.central_mapping_lookups += 1;
+    }
     let files_by_inode = version.files_other_than_the_first_by_inode();
     for inode in version
         .files_other_than_the_first
@@ -481,8 +668,13 @@
         );
         compared.file_lookups += 1;
     }
+    assert_the_index_lists_the_first_entry_of_every_identity_once(
+        &files_by_inode,
+        &version.files_other_than_the_first,
+        |file| file.inode,
+        &format!("txg {} 的文件按 inode 号", version.root.checkpoint_txg.0),
+    );
     let mut read_leaf_by_leaf = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
-    let absent_key = vec![0xff_u8; MAPPING_KEY_BYTES];
     for key in version
         .mapped_units
         .iter()
@@ -504,7 +696,7 @@
         compared.central_mapping_lookups += 1;
     }
     assert_eq!(
-        central_mapping_as_the_compaction_reads_it(version).mapping_entries,
+        central_mapping_as_the_compaction_reads_it(version),
         mapping_entries_found_key_by_key(version),
         "txg {}：整理读这一版取的映射条目与逐 key 查的参照逐项相同（不漏、不错、次序同）",
         version.root.checkpoint_txg.0
@@ -577,7 +769,7 @@
     let mut batches_published = 0;
     // 迭代上界是 COMPACTION_RUNS_AT_MOST；跨轮携带的是会话与发了几批；一次调用一版都没发就出圈。
     for _ in 0..COMPACTION_RUNS_AT_MOST {
-        let run = session.run_compaction(&mut devices).expect("整理一趟");
+        let (run, _) = run_compaction_and_count_its_lookups(&mut session, &mut devices);
         for version in &run.versions_published {
             on_every_version(version);
         }
@@ -865,95 +1057,39 @@
     assert!(compared_on_the_original >= 4, "原样那一版每把 key 都比过");
 }
 
-/// 包着一个身份，每被比一次（`==` 或 `cmp`）就在共用的计数上加一：数按身份的索引查一次比了几次身份
-/// （七·6 的增长判据数次数，不读挂钟、不读 CPU 时间；次数只由表与查法定，不随负载与构建档漂移）。
-struct IdentityCountingItsComparisons<'counter, Identity> {
-    identity: Identity,
-    comparisons: &'counter Cell<u64>,
-}
-
-impl<Identity: PartialEq> PartialEq for IdentityCountingItsComparisons<'_, Identity> {
-    fn eq(&self, other: &Self) -> bool {
-        self.comparisons.set(self.comparisons.get() + 1);
-        self.identity == other.identity
-    }
-}
-
-impl<Identity: Eq> Eq for IdentityCountingItsComparisons<'_, Identity> {}
-
-impl<Identity: Ord> PartialOrd for IdentityCountingItsComparisons<'_, Identity> {
-    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
-        Some(self.cmp(other))
-    }
-}
-
-impl<Identity: Ord> Ord for IdentityCountingItsComparisons<'_, Identity> {
-    fn cmp(&self, other: &Self) -> Ordering {
-        self.comparisons.set(self.comparisons.get() + 1);
-        self.identity.cmp(&other.identity)
-    }
-}
-
-/// 查了几次、一共比了几次身份。
-#[derive(Clone, Copy, Debug, Default)]
-struct ComparisonsOfLookups {
-    lookups: u64,
-    comparisons: u64,
-}
-
-impl ComparisonsOfLookups {
-    fn plus(self, other: Self) -> Self {
-        Self {
-            lookups: self.lookups + other.lookups,
-            comparisons: self.comparisons + other.comparisons,
-        }
-    }
-}
-
 /// 在 `table` 上建按身份的索引（`FirstEntryOfEachIdentity`，发布与整理用的同一个），把表里每一项的身份连同一个表里没有的
-/// `absent_identity` 各查一次：表里的查得到、`absent_identity` 查不到。建索引时的比较不算，只数查的那几次。
-fn comparisons_of_looking_up_every_identity<Entry, Identity: Ord + Copy>(
+/// `absent_identity` 各查一次：表里的查得到、`absent_identity` 查不到。交回这几次查在产品里记下的计数（建索引那一次不在其内）。
+fn lookup_work_of_looking_up_every_identity<Entry, Identity: Ord + Copy>(
     table: &[Entry],
     identity_of: impl Fn(&Entry) -> Identity,
     absent_identity: Identity,
-) -> ComparisonsOfLookups {
-    let comparisons = Cell::new(0);
-    let index =
-        FirstEntryOfEachIdentity::of_the_table(table, |entry| IdentityCountingItsComparisons {
-            identity: identity_of(entry),
-            comparisons: &comparisons,
-        });
-    comparisons.set(0);
-    let mut lookups = 0;
-    // 迭代上界是表长加一；跨轮携带的只有查了几次。
-    for identity in table.iter().map(&identity_of).chain([absent_identity]) {
-        let found = index.first_entry_of(&IdentityCountingItsComparisons {
-            identity,
-            comparisons: &comparisons,
-        });
-        assert_eq!(
-            found.is_some(),
-            identity != absent_identity,
-            "表里的身份查得到，表里没有的查不到"
-        );
-        lookups += 1;
-    }
-    ComparisonsOfLookups {
-        lookups,
-        comparisons: comparisons.get(),
-    }
+) -> LookupWork {
+    let index = FirstEntryOfEachIdentity::of_the_table(table, &identity_of);
+    let ((), lookup_work) = lookup_work_while(|| {
+        // 迭代上界是表长加一。
+        for identity in table.iter().map(&identity_of).chain([absent_identity]) {
+            assert_eq!(
+                index.first_entry_of(&identity).is_some(),
+                identity != absent_identity,
+                "表里的身份查得到，表里没有的查不到"
+            );
+        }
+    });
+    assert_every_lookup_examines_at_most_a_binary_search(&lookup_work, "合成表上每个身份查一次");
+    lookup_work.of_every_site()
 }
 
-/// 表长四倍时，按身份查一次平均比较身份的次数至多涨到几倍（百分数）：有序表查找按层数涨（512 到 2048 多一两层），
+/// 表长四倍时，查一次平均看过的索引项数（与整理每一趟建索引的次数）至多涨到几倍（百分数）：折半查按层数涨（512 到 2048 多两层），
 /// 在表上逐项找按表长涨（四倍）；门槛取两者之间、离逐项找远的一边。
-const HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;
+const HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;
 
-/// 两档每次查的平均比较次数之比（百分数）。
-fn growth_in_percent_of_the_comparisons_per_lookup(
-    smaller: ComparisonsOfLookups,
-    larger: ComparisonsOfLookups,
+/// 两档每次查平均看过的索引项数之比（百分数）。
+fn growth_in_percent_of_the_entries_examined_per_lookup(
+    smaller: LookupWork,
+    larger: LookupWork,
 ) -> u64 {
-    larger.comparisons * smaller.lookups * 100 / (smaller.comparisons * larger.lookups)
+    larger.index_entries_examined * smaller.index_lookups * 100
+        / (smaller.index_entries_examined * larger.index_lookups)
 }
 
 /// 合成表的两档：512 个与 2048 个不同的身份，按一个与表长互素的步长打乱次序。
@@ -968,29 +1104,28 @@
         .collect()
 }
 
-/// 512 个与 2048 个身份的合成表上，按身份的索引每次查平均比较身份的次数涨不到门槛的倍数（数次数，不读挂钟与 CPU 时间）。
-/// 判别力：`crates/mutations.tsv` 里「七·6 按身份查：索引退回逐项找（数比较次数）」那一行。
+/// 512 个与 2048 个身份的合成表上，按身份的索引每次查平均看过的索引项数涨不到门槛的倍数（读产品里的计数，不读挂钟与 CPU 时间）。
+/// 判别力：`crates/mutations.tsv` 里「七·6 按身份查：索引从大到小逐项找」那一行。
 #[test]
-fn first_entry_of_each_identity_compares_less_than_twice_as_often_per_lookup_on_a_table_four_times_as_long(
+fn first_entry_of_each_identity_examines_less_than_twice_as_many_entries_per_lookup_on_a_table_four_times_as_long(
 ) {
-    let smaller = comparisons_of_looking_up_every_identity(
+    let smaller = lookup_work_of_looking_up_every_identity(
         &synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE),
         |identity| *identity,
         u64::MAX,
     );
-    let larger = comparisons_of_looking_up_every_identity(
+    let larger = lookup_work_of_looking_up_every_identity(
         &synthetic_table_of_identities(IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE),
         |identity| *identity,
         u64::MAX,
     );
-    let growth_in_percent = growth_in_percent_of_the_comparisons_per_lookup(smaller, larger);
+    let growth_in_percent = growth_in_percent_of_the_entries_examined_per_lookup(smaller, larger);
     println!(
         "publish_lookup_by_identity synthetic_table smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
     );
     assert!(
-        growth_in_percent
-            < HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE,
-        "表长四倍，每次查平均比较次数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE}%（小表 {smaller:?}，大表 {larger:?}）"
+        growth_in_percent < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
+        "表长四倍，每次查平均看过的索引项数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（小表 {smaller:?}，大表 {larger:?}）"
     );
 }
 
@@ -1000,113 +1135,107 @@
 const FILES_OF_THE_LARGER_GROWTH_POOL: u64 = 256;
 /// 一批整理的批大小（E185 的 K = 8）。
 const COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS: usize = 8;
-/// 每档量几次（两档轮流，一次调用至多发一版）。
+/// 每档量几次（一次调用至多发一版）。
 const MEASURED_CALLS_PER_POOL: usize = 8;
 /// 每档至少要有几次真发了一版才算数。
 const PUBLISHING_CALLS_REQUIRED_PER_POOL: usize = 5;
 
-/// 重挂一份造好的池（批大小强制 K），一次调用一次地推整理：每次调用之前把这次挂载数过的发布数压到上限，一次调用至多发一版
-/// （照 E185 装置 `measure_one_intent`）。调用之前在这一批要读的那一版上数两样：整理读它解了几片映射叶（判恰好是 key 走到的那几片），
-/// `units` 上每个角色按身份查一次比了几次身份。交回比较的次数；一版都没发的交回 `None`。
+/// 一次调用推一次整理：调用之前把这次挂载数过的发布数压到上限，一次调用至多发一版（照 E185 装置 `measure_one_intent`）。
+/// 调用之前判整理读这一批要读的那一版恰好解进映射的 key 走到的那几片叶；交回这一次整理在产品里做的查找（每一处合起来）；
+/// 一版都没发的交回 `None`。
 fn lookup_work_of_one_compaction_call(
     session: &mut MountedSession,
     devices: &mut Devices,
-) -> Option<ComparisonsOfLookups> {
+) -> Option<LookupWork> {
     session.compaction.publishes_since_the_last_user_change = 1000;
-    let version_the_batch_reads = version_of(session);
-    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version_the_batch_reads);
-    let comparisons = comparisons_of_looking_up_every_identity(
-        &version_the_batch_reads.units,
-        |unit| unit.identity,
-        TransactionUnit::Data(DataUnitIndexInFile(u64::MAX)),
-    );
-    let run = session.run_compaction(devices).expect("整理一趟");
+    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version_of(session));
+    let (run, lookup_work) = run_compaction_and_count_its_lookups(session, devices);
     match run.outcome {
         CompactionRunOutcome::Stopped(_) => {}
         CompactionRunOutcome::Failed(failure) => panic!("整理的一次发布没发成：{failure:?}"),
     }
-    (!run.versions_published.is_empty()).then_some(comparisons)
+    (!run.versions_published.is_empty()).then(|| lookup_work.of_every_site())
+}
+
+/// 造一档池（`files` 个八单元的文件），批大小强制 K 重挂，推 `MEASURED_CALLS_PER_POOL` 次整理：交回真发了一版的那几次各自做的查找，
+/// 按调用的次序。
+fn lookup_work_of_the_compaction_calls_on_a_pool_of(files: u64) -> Vec<LookupWork> {
+    let mut devices = build_a_pool_of_files(
+        PoolOfFiles {
+            files,
+            data_units_per_file: DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS,
+        },
+        &mut |_| {},
+    )
+    .devices;
+    let parameters = common::parameters();
+    let switches = product_switches_with_the_batch_size(COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS);
+    let (_, mut session) = MountedSession::of_the_mount(
+        mount_writable_with_compaction_switches(&parameters, &mut devices, switches)
+            .unwrap_or_else(|refusal| panic!("重挂 {files} 个文件的池：{refusal:?}")),
+    );
+    // 迭代上界是 MEASURED_CALLS_PER_POOL；跨轮携带的是会话与盘。
+    (0..MEASURED_CALLS_PER_POOL)
+        .filter_map(|_| lookup_work_of_one_compaction_call(&mut session, &mut devices))
+        .collect()
 }
 
-/// 512 个数据单元的池与 2048 个的池（每个文件八个单元），同一进程里重挂、两档轮流各推 8 次整理（K = 8、一次至多一版）。
-/// 数次数，不读挂钟、不读 CPU 时间：每一批读的那一版上，整理读它解的映射叶恰好是进映射的 key 走到的那几片；`units` 上按角色查一次
-/// 平均比较身份的次数从小池到大池涨不到门槛的倍数。判别力：`crates/mutations.tsv` 里以「七·6 增长：」起头的几行
-/// （按身份查的索引退回逐项找、整理读版本退回逐 key 查映射）。
+/// 几次调用做的查找合起来。
+fn sum_of_the_lookup_work(readings: &[LookupWork]) -> LookupWork {
+    readings
+        .iter()
+        .fold(LookupWork::default(), |sum, reading| sum.plus(*reading))
+}
+
+/// 512 个数据单元的池与 2048 个的池（每个文件八个单元），各推 8 次整理（K = 8、一次至多一版），读产品里记下的计数，不读挂钟、
+/// 不读 CPU 时间：每一批读的那一版上，整理读它恰好解进映射的 key 走到的那几片叶；每一趟整理每次查至多一次折半查找、计划里每一项都经
+/// 按 key 的索引查过；查一次平均看过的索引项数与每一趟建索引的次数从小池到大池涨不到门槛的倍数。两档互不依赖：各造各的池、各挂各的会话，
+/// 计数记在各自的线程上，各在一个线程里量，合并按档的次序。
+/// 判别力：`crates/mutations.tsv` 里以「七·6 增长：」起头的几行（按身份查的索引退回逐项找、整理读版本退回逐 key 查映射）。
 // harness-test-granularity:one-scenario 两档池同一个场景：比的是同一串操作在两档上的比，单造一档量不出比。
 #[test]
 #[ignore = "harness 慢档用例：造 2048 个数据单元的池要发约 460 次发布，debug 下单条超过 60 秒"]
-fn compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units(
+fn compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units(
 ) {
-    let (mut smaller_devices, mut larger_devices) = std::thread::scope(|scope| {
+    let (smaller_readings, larger_readings) = std::thread::scope(|scope| {
         let smaller = scope.spawn(|| {
-            build_a_pool_of_files(
-                PoolOfFiles {
-                    files: FILES_OF_THE_SMALLER_GROWTH_POOL,
-                    data_units_per_file: DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS,
-                },
-                &mut |_| {},
-            )
-            .devices
+            lookup_work_of_the_compaction_calls_on_a_pool_of(FILES_OF_THE_SMALLER_GROWTH_POOL)
         });
         let larger = scope.spawn(|| {
-            build_a_pool_of_files(
-                PoolOfFiles {
-                    files: FILES_OF_THE_LARGER_GROWTH_POOL,
-                    data_units_per_file: DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS,
-                },
-                &mut |_| {},
-            )
-            .devices
+            lookup_work_of_the_compaction_calls_on_a_pool_of(FILES_OF_THE_LARGER_GROWTH_POOL)
         });
         (
-            smaller.join().expect("造小池的线程没 panic"),
-            larger.join().expect("造大池的线程没 panic"),
+            smaller.join().expect("量小池的线程没 panic"),
+            larger.join().expect("量大池的线程没 panic"),
         )
     });
-    let parameters = common::parameters();
-    let switches = product_switches_with_the_batch_size(COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS);
-    let (_, mut smaller_session) = MountedSession::of_the_mount(
-        mount_writable_with_compaction_switches(&parameters, &mut smaller_devices, switches)
-            .expect("重挂小池"),
-    );
-    let (_, mut larger_session) = MountedSession::of_the_mount(
-        mount_writable_with_compaction_switches(&parameters, &mut larger_devices, switches)
-            .expect("重挂大池"),
-    );
-    let mut smaller_readings = Vec::new();
-    let mut larger_readings = Vec::new();
-    // 迭代上界是 MEASURED_CALLS_PER_POOL；两档轮流（与改前量 CPU 时同一个次序）。
-    for _ in 0..MEASURED_CALLS_PER_POOL {
-        smaller_readings.extend(lookup_work_of_one_compaction_call(
-            &mut smaller_session,
-            &mut smaller_devices,
-        ));
-        larger_readings.extend(lookup_work_of_one_compaction_call(
-            &mut larger_session,
-            &mut larger_devices,
-        ));
-    }
     println!(
-        "publish_lookup_by_identity growth smaller_comparisons={smaller_readings:?} larger_comparisons={larger_readings:?}"
+        "publish_lookup_by_identity growth smaller_readings={smaller_readings:?} larger_readings={larger_readings:?}"
     );
     assert!(
         smaller_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL
             && larger_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL,
         "两档各至少 {PUBLISHING_CALLS_REQUIRED_PER_POOL} 次真发了一版：小池 {smaller_readings:?}，大池 {larger_readings:?}"
     );
-    let smaller = smaller_readings
-        .into_iter()
-        .fold(ComparisonsOfLookups::default(), ComparisonsOfLookups::plus);
-    let larger = larger_readings
-        .into_iter()
-        .fold(ComparisonsOfLookups::default(), ComparisonsOfLookups::plus);
-    let growth_in_percent = growth_in_percent_of_the_comparisons_per_lookup(smaller, larger);
+    let smaller = sum_of_the_lookup_work(&smaller_readings);
+    let larger = sum_of_the_lookup_work(&larger_readings);
+    let smaller_calls = u64::try_from(smaller_readings.len()).expect("次数");
+    let larger_calls = u64::try_from(larger_readings.len()).expect("次数");
+    let growth_in_percent_of_the_entries_examined =
+        growth_in_percent_of_the_entries_examined_per_lookup(smaller, larger);
+    let growth_in_percent_of_the_index_builds =
+        larger.index_builds * smaller_calls * 100 / (smaller.index_builds * larger_calls);
     println!(
-        "publish_lookup_by_identity growth smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
+        "publish_lookup_by_identity growth smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent_of_the_entries_examined} index_builds_growth_in_percent={growth_in_percent_of_the_index_builds}"
+    );
+    assert!(
+        growth_in_percent_of_the_entries_examined
+            < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
+        "数据单元四倍，整理每次按身份查平均看过的索引项数涨了 {growth_in_percent_of_the_entries_examined}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（小池 {smaller:?}，大池 {larger:?}）"
     );
     assert!(
-        growth_in_percent
-            < HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE,
-        "数据单元四倍，units 上按角色查一次平均比较次数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE}%（小池 {smaller:?}，大池 {larger:?}）"
+        growth_in_percent_of_the_index_builds
+            < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
+        "数据单元四倍，整理每一趟建索引的次数涨了 {growth_in_percent_of_the_index_builds}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（每查一把 key 现建一个索引就按 key 数涨；小池 {smaller:?} 共 {smaller_calls} 次，大池 {larger:?} 共 {larger_calls} 次）"
     );
 }
````

## mutations-append.tsv

````text
七·6 按身份查：单元索引按角色查退回在索引上逐项找（攻方 MA）	crates/singlefs-core/src/transaction.rs	        self.first_entry_of(&identity)\n            .expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)	        self.first_entry_of_each_identity\n            .iter()\n            .find(|(candidate, _)| **candidate == identity)\n            .map(|(_, entry)| *entry)\n            .expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 整理读版本：批量读者每把 key 把叶整片重解一次（攻方 MC）	crates/singlefs-core/src/transaction.rs	            .entry(leaf_role)\n            .or_insert_with(|| {	            .entry(leaf_role)\n            .and_modify(|read| {\n                *read = CentralMappingLeafRead::of_the_leaf_bytes(&units_by_role.unit(leaf_role).bytes);\n            })\n            .or_insert_with(|| {	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 按身份查：映射 key 按角色查退回在索引上逐项找（攻方 MD）	crates/singlefs-core/src/transaction.rs	        self.first_entry_of(&identity)\n            .map(|(_, key)| key.as_slice())	        self.first_entry_of_each_identity\n            .iter()\n            .find(|(candidate, _)| **candidate == identity)\n            .map(|(_, entry)| *entry)\n            .map(|(_, key)| key.as_slice())	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 整理读版本：每把 key 现建一个批量读者、外层读者照查一遍（攻方 MF）	crates/singlefs-core/src/mounted_session.rs	        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	        .filter_map(|(_, key)| match {\n            let _ = central_mapping.locations_of_key(key);\n            CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version).locations_of_key(key)\n        } {	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 按身份查：快照判定那一圈取映射 key 退回在 mapped_units 上线性找（攻方 MG）	crates/singlefs-core/src/transaction.rs	    let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();\n    for identity in &user_visible_roles_replaced {\n        let mapping_key = mapped_units_of_the_previous_version\n            .mapping_key_of(*identity)\n            .to_vec();	    for identity in &user_visible_roles_replaced {\n        let mapping_key = previous_version\n            .mapped_units\n            .iter()\n            .find(|(mapped, _)| *mapped == *identity)\n            .map(|(_, key)| key.clone())\n            .expect("照抄进来的角色在上一版里进映射、有一把 key");	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 按身份查：每次查之前先把整份索引走一遍、不比身份（攻方 MI）	crates/singlefs-core/src/transaction.rs	        self.first_entry_of_each_identity.get(identity).copied()	        let _walked = self\n            .first_entry_of_each_identity\n            .iter()\n            .fold(0_usize, |walked, _| walked + 1);\n        self.first_entry_of_each_identity.get(identity).copied()	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 按身份查：经映射释放的角色取映射条目退回线性找 key、逐 key 查映射	crates/singlefs-core/src/transaction.rs	        let (_, key) = mapped_units_by_role\n            .first_entry_of(&identity)\n            .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");\n        central_mapping.locations_of_key(key)	        let (_, key) = version\n            .mapped_units\n            .iter()\n            .find(|(mapped, _)| *mapped == identity)\n            .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");\n        central_mapping_locations_in_the_version(version, key)	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
七·6 按身份查：整理发批核源落点退回按 key 线性找角色、逐 key 查映射	crates/singlefs-core/src/transaction.rs	    let holder_and_locations = match previous_read_by_key\n        .mapped_units_by_key\n        .first_entry_of(&mapping_key.as_slice())\n    {\n        Some((role, _)) => {\n            let locations = match previous_read_by_key\n                .central_mapping\n                .locations_of_key(&mapping_key)\n            {	    let holder_and_locations = match previous\n        .mapped_units\n        .iter()\n        .find(|(_, key)| *key == mapping_key)\n    {\n        Some((role, _)) => {\n            let locations = match central_mapping_locations_in_the_version(previous, &mapping_key) {	-p singlefs-harness --test publish_lookup_by_identity -- --exact every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units	every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
````

## mutations-replacements.tsv

````text
七·6 按身份查：同一身份取最后一项	crates/singlefs-core/src/transaction.rs	        for entry in table {\n            identities_with_their_entries.push((identity_of(entry), entry));	        for entry in table.iter().rev() {\n            identities_with_their_entries.push((identity_of(entry), entry));	-p singlefs-harness --test publish_lookup_by_identity	first_entry_of_each_identity_keeps_the_first_of_repeated_identities_and_misses_absent_ones
七·6 增长：按身份查的索引退回逐项找	crates/singlefs-core/src/transaction.rs	        self.first_entry_of_each_identity.get(identity).copied()	        self.first_entry_of_each_identity\n            .iter()\n            .find(|(candidate, _)| *candidate == identity)\n            .map(|(_, entry)| *entry)	-p singlefs-harness --test publish_lookup_by_identity -- --include-ignored --exact compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units	compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units
七·6 增长：整理读版本退回逐 key 查映射	crates/singlefs-core/src/mounted_session.rs	        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {	        .filter_map(|(_, key)| match crate::transaction::central_mapping_locations_in_the_version(version, key) {	-p singlefs-harness --test publish_lookup_by_identity -- --include-ignored --exact compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units	compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units
七·6 按身份查：索引从大到小逐项找	crates/singlefs-core/src/transaction.rs	        self.first_entry_of_each_identity.get(identity).copied()	        self.first_entry_of_each_identity\n            .iter()\n            .rev()\n            .find(|(candidate, _)| *candidate == identity)\n            .map(|(_, entry)| *entry)	-p singlefs-harness --test publish_lookup_by_identity -- --exact first_entry_of_each_identity_examines_less_than_twice_as_many_entries_per_lookup_on_a_table_four_times_as_long	first_entry_of_each_identity_examines_less_than_twice_as_many_entries_per_lookup_on_a_table_four_times_as_long
````
