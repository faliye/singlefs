# 附录二：2026-09-23 四份补丁对主工作区现状的改动（材料员生成，2026-09-24 00:35 UTC / 09:35 JST）

三份有独立补丁（harness、checker、watermark），一、三、五节各放补丁原样，二、四、六节各放它追加、还没合并进 `crates/mutations.tsv` 的变异名（原样）。
第四组（C511 收窄 + C512 根记录加一项）没有单独补丁：七节是 `crates/singlefs-core/src/root_record.rs` 的 `git diff HEAD` 全文，
八节是从主工作区现状按「文件::项名」用 `research/scripts/quote-rust-items.py` 抽出的其余各处（`research/prompts/m2-c511-c512-implementer-report.md` 第三节「连带改了哪几处」与第四节变异行给的函数名；该报告第四节点名的 5 行变异现在是 `crates/mutations.tsv` 第 310–314 行，名字都以 `C511（` 或 `C512（` 开头——`crates/mutations.tsv` 里另有 8 行以 `C511 第 3 步` 开头，那是五节 watermark 组自己的变异，不属于这一组）。
诊断、门禁与 sha256 引自四份实现员报告：`research/prompts/m2-wave3-harness-implementer-report.md`、`research/prompts/m2-wave3-checker-implementer-report.md`、`research/prompts/m2-wave3-watermark-implementer-report.md`、`research/prompts/m2-c511-c512-implementer-report.md`。

## 一、harness 那一组：`/tmp/claude-1000/impl-harness-batch/harness.patch`（原样拷 `research/prompts/m2-wave3-code-r1-patches/harness.patch`）

基准：主工作区 2026-09-23 22:55 UTC 现状（HEAD `3b60f098e97dc4c4f3ed9c6355422b607db1c34c` + 当时的未提交改动，`diff -rq` 现查过同步之后没再变过）；
`research/prompts/m2-wave3-harness-implementer-report.md` 第八节 `git apply --check` 在主工作区原样跑，9 个文件全部 `Checking patch … ` 后 `git apply --check exit 0`。
`sha256sum harness.patch` = `466e22de873f5570320814dcee1e2ffce9cfd9913f9940fdc9289fc0be3ac973`（与 `m2-wave3-code-r1-patches/harness.patch` 一致，已核对）。

```diff
diff --git a/crates/singlefs-harness/src/bad_disk_input.rs b/crates/singlefs-harness/src/bad_disk_input.rs
--- a/crates/singlefs-harness/src/bad_disk_input.rs
+++ b/crates/singlefs-harness/src/bad_disk_input.rs
@@ -497,6 +497,8 @@
 #[derive(Clone, Debug)]
 pub struct BadDiskFinding {
     pub seed: HistorySeed,
+    /// 坏的是哪一档的基线镜像。
+    pub tier: BaseImageTier,
     pub kind: DamageKind,
     pub what: String,
     /// 一句话说清是哪一类失败。
@@ -520,8 +522,9 @@
     #[must_use]
     pub fn render(&self) -> String {
         format!(
-            "种子 {} 坏法「{}」：{}\n    坏在哪：{}\n    观察：{}",
+            "种子 {} 基线档「{}」坏法「{}」：{}\n    坏在哪：{}\n    观察：{}",
             self.seed.0,
+            self.tier.name(),
             self.kind.name(),
             self.headline,
             self.what,
@@ -536,6 +539,10 @@
     pub histories: u64,
     /// 历史跑完之后一份可用的合法镜像都没交出来的段数（起点就失败）。
     pub histories_without_a_base_image: u64,
+    /// 各基线档（[`BaseImageTier::name`]）抽到基线镜像的段数：一段历史每档至多一份。
+    pub base_images_by_tier: BTreeMap<&'static str, u64>,
+    /// 各基线档的基线镜像上造出的坏镜像数。
+    pub damaged_images_by_tier: BTreeMap<&'static str, u64>,
     /// 各坏法各造出几份坏镜像。
     pub damaged_images_by_kind: BTreeMap<&'static str, u64>,
     /// 各坏法在这个盘面上没有可坏的对象、照实记「不适用」的次数。
@@ -571,6 +578,14 @@
         self.histories += following.histories;
         self.histories_without_a_base_image += following.histories_without_a_base_image;
         absorb_counts(
+            &mut self.base_images_by_tier,
+            &following.base_images_by_tier,
+        );
+        absorb_counts(
+            &mut self.damaged_images_by_tier,
+            &following.damaged_images_by_tier,
+        );
+        absorb_counts(
             &mut self.damaged_images_by_kind,
             &following.damaged_images_by_kind,
         );
@@ -604,6 +619,21 @@
         );
     }
 
+    /// 抽到过基线镜像的档数：[`EVERY_BASE_IMAGE_TIER`] 里至少一段历史抽到了一份的那几档。
+    #[must_use]
+    pub fn base_image_tiers_sampled(&self) -> usize {
+        EVERY_BASE_IMAGE_TIER
+            .into_iter()
+            .filter(|tier| {
+                self.base_images_by_tier
+                    .get(tier.name())
+                    .copied()
+                    .unwrap_or(0)
+                    > 0
+            })
+            .count()
+    }
+
     /// 报告正文。
     #[must_use]
     pub fn render(&self) -> String {
@@ -613,6 +643,30 @@
             "  历史 {} 段（起点就失败、交不出合法镜像的 {} 段）",
             self.histories, self.histories_without_a_base_image
         );
+        let per_tier: Vec<String> = EVERY_BASE_IMAGE_TIER
+            .into_iter()
+            .map(|tier| {
+                format!(
+                    "「{}」{} 段、坏镜像 {} 份",
+                    tier.name(),
+                    self.base_images_by_tier
+                        .get(tier.name())
+                        .copied()
+                        .unwrap_or(0),
+                    self.damaged_images_by_tier
+                        .get(tier.name())
+                        .copied()
+                        .unwrap_or(0)
+                )
+            })
+            .collect();
+        let _ = writeln!(
+            text,
+            "  基线档 {} / {} 档抽到过镜像：{}",
+            self.base_image_tiers_sampled(),
+            EVERY_BASE_IMAGE_TIER.len(),
+            per_tier.join("；")
+        );
         let _ = writeln!(
             text,
             "  三个读者：恢复 {} 次（读回文件 {}、没有文件 {}、报错 {}）、可写挂载 {} 次（成 {}、拒 {}）、池级 checker {} 次",
@@ -799,12 +853,80 @@
             .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0)
 }
 
-/// 跑一段历史，拿它交出来的一份合法镜像逐个坏法坏一遍，每份坏镜像喂给恢复、可写挂载与池级 checker。
+/// 一份镜像上最新那条根指着的树表是 0 条：只做过 mkfs、第一个文件版本之前的写行与暖机那几版、回退到这样一版的根。
+/// C481（坏盘输入的基线镜像取不到树表 0 条的盘面） 要的那一档基线就是它。树表走不到（根择不出、指针全零、条目宽不是 200）不算。
+#[must_use]
+pub fn newest_root_tree_table_has_no_entries(image: &MemoryPool) -> bool {
+    reach_the_tree_table(image)
+        .is_some_and(|reached| IndexNodeEntryLayout::of(&reached.tree_table_node).entry_count == 0)
+}
+
+/// 一段历史里按哪一档抽基线镜像。每一档各按种子抽一份（蓄水池抽样），抽到的每一份都把全部坏法跑一遍、判据相同。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum BaseImageTier {
+    /// 最新那条根下面指着普查那几族的坏法要坏的树都在（[`newest_root_has_the_trees_the_targeted_damages_need`]）；
+    /// 一段历史里一步都不合格时退回最后那一步的镜像，盲坏法照样跑。
+    NewestRootHasTheTreesTheTargetedDamagesNeed,
+    /// 最新那条根指着的树表 0 条（[`newest_root_tree_table_has_no_entries`]，C481）。一段历史里一步都没有就这一段不抽，不退回。
+    TreeTableWithoutEntries,
+}
+
+/// 全部基线档，报告按这个次序逐档报。
+pub const EVERY_BASE_IMAGE_TIER: [BaseImageTier; 2] = [
+    BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed,
+    BaseImageTier::TreeTableWithoutEntries,
+];
+
+/// 树表 0 条那一档的蓄水池与坏法随机源另加的盐：与三棵树都在那一档岔开，那一档每个种子抽到的镜像、坏成的样子都与加这一档之前逐字节相同。
+const TREE_TABLE_WITHOUT_ENTRIES_TIER_SALT: u64 = 0x74_72_65_65_30_00_00_00;
+
+impl BaseImageTier {
+    /// 报告与计数里的名字。
+    #[must_use]
+    pub fn name(self) -> &'static str {
+        match self {
+            BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => "三棵树都在",
+            BaseImageTier::TreeTableWithoutEntries => "树表 0 条",
+        }
+    }
+
+    /// 这一步的镜像够不够进这一档的蓄水池。
+    #[must_use]
+    pub fn admits(self, image: &MemoryPool) -> bool {
+        match self {
+            BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => {
+                newest_root_has_the_trees_the_targeted_damages_need(image)
+            }
+            BaseImageTier::TreeTableWithoutEntries => newest_root_tree_table_has_no_entries(image),
+        }
+    }
+
+    /// 这一档的随机源在种子上另异或的盐（蓄水池与坏法共用）。
+    const fn salt(self) -> u64 {
+        match self {
+            BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => 0,
+            BaseImageTier::TreeTableWithoutEntries => TREE_TABLE_WITHOUT_ENTRIES_TIER_SALT,
+        }
+    }
+}
+
+/// 一档基线的蓄水池：合格的镜像见过几份、手里留的是哪一份。
+struct BaseImageReservoir {
+    tier: BaseImageTier,
+    qualifying_images_seen: u64,
+    chosen: Option<MemoryPool>,
+    random: SeededRandomSource,
+}
+
+/// 跑一段历史，按 [`EVERY_BASE_IMAGE_TIER`] 每一档各抽一份合法镜像，逐个坏法坏一遍，每份坏镜像喂给恢复、可写挂载与池级 checker。
+///
+/// 每一档的基线镜像取**按种子抽中的那一步**之后的那一份（蓄水池抽样，每档只留一份：一段历史几十步，全留下几十份稀疏镜像太占内存）：
+/// - 「三棵树都在」那一档只在最新那条根下面该有的那几棵树都在的那几步里抽（[`newest_root_has_the_trees_the_targeted_damages_need`]）：
+///   树表 0 条的盘面上指着普查那几族的坏法一样也做不出来，抽中它这一档就只跑了三个盲坏法。一步都不合格时退回最后那一步的镜像。
+/// - 「树表 0 条」那一档只在最新那条根的树表 0 条的那几步里抽（C481（坏盘输入的基线镜像取不到树表 0 条的盘面））：
+///   只做过 mkfs 的池、第一个文件版本之前的写行与暖机、回退到这样一版——这些盘面从前一档都没打过。一步都没有就这一段不抽。
 ///
-/// 基线镜像取**按种子抽中的那一步**之后的那一份（蓄水池抽样，只留一份：一段历史几十步，全留下几十份稀疏镜像太占内存），
-/// 而且只在「最新那条根下面该有的那几棵树都在」的那几步里抽（[`newest_root_has_the_trees_the_targeted_damages_need`]）：
-/// 树表 0 条的盘面上指着普查那几族的坏法一样也做不出来，抽中它整段历史就只跑了三个盲坏法。
-/// 一段历史里一步都不合格时退回最后那一步的镜像——盲坏法照样跑，指着普查的那几条照实记「不适用」。
+/// 两档的判据相同（不许读回没提交过的内容、不许 panic），坏法照旧全部跑，做不出来的照实记「不适用」。
 #[must_use]
 pub fn feed_bad_disk_inputs_from_history(
     seed: HistorySeed,
@@ -814,10 +936,16 @@
 ) -> HistoryBadDiskInput {
     let history = generate_history_with_weights(seed, operations_per_history, weights);
     let mut committed_contents: BTreeSet<Option<Vec<u8>>> = BTreeSet::new();
-    let mut base_image: Option<MemoryPool> = None;
     let mut last_image: Option<MemoryPool> = None;
-    let mut qualifying_images_seen = 0u64;
-    let mut reservoir = SeededRandomSource::from_seed(seed.0 ^ BAD_DISK_SEED_SALT);
+    let mut reservoirs: Vec<BaseImageReservoir> = EVERY_BASE_IMAGE_TIER
+        .into_iter()
+        .map(|tier| BaseImageReservoir {
+            tier,
+            qualifying_images_seen: 0,
+            chosen: None,
+            random: SeededRandomSource::from_seed(seed.0 ^ BAD_DISK_SEED_SALT ^ tier.salt()),
+        })
+        .collect();
     execute_history_with(
         &history,
         execution,
@@ -827,13 +955,15 @@
                 committed_contents.insert(file.map(|content| content.to_vec()));
             }
             last_image = Some(observation.image.clone());
-            if !newest_root_has_the_trees_the_targeted_damages_need(observation.image) {
-                return;
-            }
-            qualifying_images_seen += 1;
-            // 蓄水池抽样：第 n 份合格镜像以 1/n 的概率换下手里那一份，跑完手里就是合格那几步里均匀抽的一份。
-            if reservoir.below(qualifying_images_seen) == 0 {
-                base_image = Some(observation.image.clone());
+            for reservoir in &mut reservoirs {
+                if !reservoir.tier.admits(observation.image) {
+                    continue;
+                }
+                reservoir.qualifying_images_seen += 1;
+                // 蓄水池抽样：第 n 份合格镜像以 1/n 的概率换下手里那一份，跑完手里就是合格那几步里均匀抽的一份。
+                if reservoir.random.below(reservoir.qualifying_images_seen) == 0 {
+                    reservoir.chosen = Some(observation.image.clone());
+                }
             }
         },
     );
@@ -842,46 +972,63 @@
         histories: 1,
         ..BadDiskTally::default()
     };
-    let Some(base_image) = base_image.or(last_image) else {
+    if last_image.is_none() {
         tally.histories_without_a_base_image = 1;
         return HistoryBadDiskInput {
             seed,
             tally,
             findings: Vec::new(),
         };
-    };
+    }
+    let base_images: Vec<(BaseImageTier, MemoryPool)> = reservoirs
+        .into_iter()
+        .filter_map(|reservoir| {
+            let chosen = match reservoir.tier {
+                BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => {
+                    reservoir.chosen.or_else(|| last_image.clone())
+                }
+                BaseImageTier::TreeTableWithoutEntries => reservoir.chosen,
+            };
+            chosen.map(|image| (reservoir.tier, image))
+        })
+        .collect();
 
     let parameters = execution.device_width.parameters();
     let mut findings = Vec::new();
-    for (kind_index, kind) in EVERY_DAMAGE_KIND.into_iter().enumerate() {
-        let mut random = SeededRandomSource::from_seed(
-            seed.0
-                .wrapping_mul(u64::try_from(EVERY_DAMAGE_KIND.len()).expect("坏法数"))
-                .wrapping_add(u64::try_from(kind_index).expect("坏法序号"))
-                ^ BAD_DISK_SEED_SALT,
-        );
-        let Some(damaged) = damage_image(&base_image, kind, &mut random) else {
-            *tally
-                .damages_not_applicable_by_kind
-                .entry(kind.name())
-                .or_insert(0) += 1;
-            continue;
-        };
-        *tally.damaged_images_by_kind.entry(kind.name()).or_insert(0) += 1;
-        let observation = feed_a_damaged_image(
-            &damaged,
-            &committed_contents,
-            &parameters,
-            execution.device_width,
-        );
-        absorb_observation(&mut tally, &observation);
-        if observation.was_seen_by_a_reader() {
-            *tally
-                .damages_seen_by_a_reader_by_kind
-                .entry(kind.name())
-                .or_insert(0) += 1;
+    for (tier, base_image) in &base_images {
+        *tally.base_images_by_tier.entry(tier.name()).or_insert(0) += 1;
+        for (kind_index, kind) in EVERY_DAMAGE_KIND.into_iter().enumerate() {
+            let mut random = SeededRandomSource::from_seed(
+                seed.0
+                    .wrapping_mul(u64::try_from(EVERY_DAMAGE_KIND.len()).expect("坏法数"))
+                    .wrapping_add(u64::try_from(kind_index).expect("坏法序号"))
+                    ^ BAD_DISK_SEED_SALT
+                    ^ tier.salt(),
+            );
+            let Some(damaged) = damage_image(base_image, kind, &mut random) else {
+                *tally
+                    .damages_not_applicable_by_kind
+                    .entry(kind.name())
+                    .or_insert(0) += 1;
+                continue;
+            };
+            *tally.damaged_images_by_kind.entry(kind.name()).or_insert(0) += 1;
+            *tally.damaged_images_by_tier.entry(tier.name()).or_insert(0) += 1;
+            let observation = feed_a_damaged_image(
+                &damaged,
+                &committed_contents,
+                &parameters,
+                execution.device_width,
+            );
+            absorb_observation(&mut tally, &observation);
+            if observation.was_seen_by_a_reader() {
+                *tally
+                    .damages_seen_by_a_reader_by_kind
+                    .entry(kind.name())
+                    .or_insert(0) += 1;
+            }
+            findings.extend(findings_in(seed, *tier, &observation));
         }
-        findings.extend(findings_in(seed, &observation));
     }
     HistoryBadDiskInput {
         seed,
@@ -940,7 +1087,11 @@
     }
 }
 
-fn findings_in(seed: HistorySeed, observation: &BadDiskObservation) -> Vec<BadDiskFinding> {
+fn findings_in(
+    seed: HistorySeed,
+    tier: BaseImageTier,
+    observation: &BadDiskObservation,
+) -> Vec<BadDiskFinding> {
     let mut findings = Vec::new();
     for (reader, captured) in observation.panics() {
         if known_panic_site_of(&captured).is_some() {
@@ -948,6 +1099,7 @@
         }
         findings.push(BadDiskFinding {
             seed,
+            tier,
             kind: observation.kind,
             what: observation.what.clone(),
             headline: format!(
@@ -960,6 +1112,7 @@
     if let ReadBackVerdict::ReadBackAVersionNeverCommitted { what } = &observation.read_back {
         findings.push(BadDiskFinding {
             seed,
+            tier,
             kind: observation.kind,
             what: observation.what.clone(),
             headline: format!("恢复读回了一版从没提交过的内容：{what}"),
diff --git a/crates/singlefs-harness/src/crash.rs b/crates/singlefs-harness/src/crash.rs
--- a/crates/singlefs-harness/src/crash.rs
+++ b/crates/singlefs-harness/src/crash.rs
@@ -707,10 +707,74 @@
     check
 }
 
+/// 层 0 的一个崩溃状态按发布归到哪一格（里程碑「第二个事务」步 6 验收第 1 条「每次发布各多少」）。
+/// 归法按段：状态所在的那一段往后数，第一次根槽 FUA 写所在的那一段写出的根，就是这个状态归的那次发布——
+/// 上一次发布的根槽写之后、这一次发布的根槽写为止，崩在中间的状态都归这一次。上一次发布的系统配置槽轮换与这一次的单元写
+/// 之间没有屏障、并在同一段时，那一段整段归这一次：段是枚举的最小单位，一个状态落在哪一段是确定的，落在哪一次写上不是。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum Layer0PublishOfState {
+    /// 这次发布的根槽 FUA 写：写表里的下标（按它排就是录制流里的次序）与它写出的根的实例代号、checkpoint_txg。
+    UpToTheRootOf {
+        root_write_index: usize,
+        instance: InstanceGeneration,
+        checkpoint_txg: CheckpointTxg,
+    },
+    /// 写表里最后一次根槽写之后的段（最后那次发布的系统配置槽轮换）：后面没有根槽写可归。
+    AfterTheLastRoot,
+    /// 每一段都整段持久的那一个状态（闭式 1 + Σ(2^|段| − 1) 里的那个 1）。
+    EveryWritePersisted,
+}
+
+impl Layer0PublishOfState {
+    /// 计数行里的名字：`instance2_txg5`、`after_the_last_root`、`every_write_persisted`。
+    #[must_use]
+    pub fn name(self) -> String {
+        match self {
+            Layer0PublishOfState::UpToTheRootOf {
+                instance,
+                checkpoint_txg,
+                ..
+            } => format!("instance{}_txg{}", instance.0, checkpoint_txg.0),
+            Layer0PublishOfState::AfterTheLastRoot => "after_the_last_root".to_string(),
+            Layer0PublishOfState::EveryWritePersisted => "every_write_persisted".to_string(),
+        }
+    }
+}
+
+/// 每一段归哪次发布（[`Layer0PublishOfState`] 的归法）：从最后一段往前走，记着「后面最近的那次根槽写」。
+#[must_use]
+pub fn publish_of_each_segment(
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+) -> Vec<Layer0PublishOfState> {
+    let mut next_root: Option<Layer0PublishOfState> = None;
+    let mut publish_of_segment = vec![Layer0PublishOfState::AfterTheLastRoot; segments.len()];
+    for (segment_index, segment) in segments.iter().enumerate().rev() {
+        if let Some(root_write_index) = segment.iter().rev().copied().find(|write_index| {
+            writes
+                .get(*write_index)
+                .is_some_and(|write| write.kind == StepKind::RootRecordFua)
+        }) {
+            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);
+            next_root = Some(Layer0PublishOfState::UpToTheRootOf {
+                root_write_index,
+                instance,
+                checkpoint_txg,
+            });
+        }
+        publish_of_segment[segment_index] =
+            next_root.unwrap_or(Layer0PublishOfState::AfterTheLastRoot);
+    }
+    publish_of_segment
+}
+
 /// 层 0 的计数：每个状态跑一遍看 journal 的恢复与一遍不看的，oracle 只判前者。
 #[derive(Clone, Debug, Default, PartialEq, Eq)]
 pub struct Layer0Tally {
     pub states: u64,
+    /// 枚举出来的状态按发布分（[`Layer0PublishOfState`] 的归法）：各格之和 = `states`。只有按段枚举的那几个入口
+    /// （`enumerate_layer0*`）记这一项；直接调 [`evaluate_state_for_versions`] 评手摆的状态时它是空的。
+    pub states_by_publish: BTreeMap<Layer0PublishOfState, u64>,
     pub violations: u64,
     pub root_persisted_states: u64,
     pub no_file_states: u64,
@@ -758,11 +822,22 @@
             .join(" ")
     }
 
+    /// 按发布分的状态数报成一段：`instance1_txg1=15 … after_the_last_root=3 every_write_persisted=1`，次序照录制流。
+    #[must_use]
+    pub fn states_by_publish_text(&self) -> String {
+        self.states_by_publish
+            .iter()
+            .map(|(publish, states)| format!("{}={states}", publish.name()))
+            .collect::<Vec<String>>()
+            .join(" ")
+    }
+
     /// 把紧跟在后面的那一片的计数并进来：计数逐项相加；「第一处」只在前面各片都没有时取这一片的。各片按状态序号从小到大并，
     /// 「第一处」就是序号最小的那一处，与单线程逐个跑逐项相同。按字段拆开写全：新加一个字段而这里没并，编译不过。
     fn absorb_following_slice(&mut self, following_slice: Layer0Tally) {
         let Layer0Tally {
             states,
+            states_by_publish,
             violations,
             root_persisted_states,
             no_file_states,
@@ -782,6 +857,9 @@
             checker_not_applicable_states,
         } = following_slice;
         self.states += states;
+        for (publish, publish_states) in states_by_publish {
+            *self.states_by_publish.entry(publish).or_insert(0) += publish_states;
+        }
         self.violations += violations;
         self.root_persisted_states += root_persisted_states;
         self.no_file_states += no_file_states;
@@ -1212,11 +1290,17 @@
     state_ranges_by_segment: Vec<Range<u64>>,
     /// 状态总数：各段展开出来的，再加最后全部持久那一个。
     state_count: u64,
+    /// 每一段归哪次发布（[`publish_of_each_segment`]），与 `segments` 同序。
+    publish_of_segment: Vec<Layer0PublishOfState>,
 }
 
 impl<'segments> Layer0StatePlan<'segments> {
     /// `expand` 只在调用线程上逐段问一次，所以它不必能跨线程。
-    fn new(segments: &'segments [Vec<usize>], expand: &dyn Fn(usize, &[usize]) -> bool) -> Self {
+    fn new(
+        writes: &[RetainedWrite],
+        segments: &'segments [Vec<usize>],
+        expand: &dyn Fn(usize, &[usize]) -> bool,
+    ) -> Self {
         let mut next_ordinal = 0u64;
         let state_ranges_by_segment = segments
             .iter()
@@ -1233,6 +1317,7 @@
             segments,
             state_ranges_by_segment,
             state_count: next_ordinal + 1,
+            publish_of_segment: publish_of_each_segment(writes, segments),
         }
     }
 
@@ -1242,6 +1327,14 @@
             .partition_point(|state_range| state_range.end <= ordinal)
     }
 
+    /// 这个状态归哪次发布：所在那一段归的那一次；最后全部持久那一个状态单列一格。
+    fn publish_of_state(&self, ordinal: u64) -> Layer0PublishOfState {
+        self.publish_of_segment
+            .get(self.segment_of_state(ordinal))
+            .copied()
+            .unwrap_or(Layer0PublishOfState::EveryWritePersisted)
+    }
+
     /// 这个状态里持久了的写：所在的段之前每一段整段持久（展不展开都一样），所在的段按段内子集掩码（第 k 位对应段里第 k 个写）。
     fn persisted_writes_of_state(&self, ordinal: u64, write_count: usize) -> Vec<bool> {
         let segment_of_state = self.segment_of_state(ordinal);
@@ -1347,6 +1440,10 @@
     let mut tally = Layer0Tally::default();
     let mut observed_states = Vec::new();
     for ordinal in slice {
+        *tally
+            .states_by_publish
+            .entry(plan.publish_of_state(ordinal))
+            .or_insert(0) += 1;
         let persisted = plan.persisted_writes_of_state(ordinal, writes.len());
         match retention {
             StateReportRetention::HandEachStateToObserver => {
@@ -1451,7 +1548,7 @@
     parallelism: Layer0Parallelism,
     mut observe_state: Option<Layer0StateObserver<'_>>,
 ) -> Layer0Tally {
-    let plan = Layer0StatePlan::new(segments, expand);
+    let plan = Layer0StatePlan::new(writes, segments, expand);
     let slices = state_slices(plan.state_count, &parallelism);
     let spawned_worker_threads = parallelism.worker_threads.get().min(slices.len());
     let retention = match observe_state {
@@ -1679,7 +1776,8 @@
             |_segment_index: usize, segment: &[usize]| segment.len() == 4;
         let assert_plan_matches_the_walk = |expand: &dyn Fn(usize, &[usize]) -> bool| {
             let walked = persisted_sets_walking_segment_by_segment(&segments, write_count, expand);
-            let plan = Layer0StatePlan::new(&segments, expand);
+            // 这条只核序号与持久集合的对应，不带写表：每一段都归「最后一次根槽写之后」。
+            let plan = Layer0StatePlan::new(&[], &segments, expand);
             assert_eq!(
                 plan.state_count,
                 u64::try_from(walked.len()).expect("状态数"),
@@ -1816,8 +1914,14 @@
     /// 并片：计数相加，「第一处」取前面那一片的；前面那一片没有才取后面的。
     #[test]
     fn absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation() {
+        let first_publish = Layer0PublishOfState::UpToTheRootOf {
+            root_write_index: 4,
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(1),
+        };
         let mut earlier = Layer0Tally {
             states: 3,
+            states_by_publish: BTreeMap::from([(first_publish, 3)]),
             violations: 1,
             root_persisted_states: 0,
             no_file_states: 3,
@@ -1838,6 +1942,11 @@
         };
         let following = Layer0Tally {
             states: 5,
+            states_by_publish: BTreeMap::from([
+                (first_publish, 2),
+                (Layer0PublishOfState::AfterTheLastRoot, 2),
+                (Layer0PublishOfState::EveryWritePersisted, 1),
+            ]),
             violations: 2,
             root_persisted_states: 5,
             no_file_states: 0,
@@ -1876,6 +1985,15 @@
             (8, 3, 1, 3, 5, 5, 1, 1, 1, 1),
             "计数逐项相加"
         );
+        assert_eq!(
+            earlier.states_by_publish,
+            BTreeMap::from([
+                (first_publish, 5),
+                (Layer0PublishOfState::AfterTheLastRoot, 2),
+                (Layer0PublishOfState::EveryWritePersisted, 1),
+            ]),
+            "按发布分的状态数逐格相加"
+        );
         assert_eq!(earlier.first_violation.as_deref(), Some("前面那一片的"));
         assert_eq!(
             earlier.first_ignored_violation.as_deref(),
@@ -1912,6 +2030,7 @@
     ) -> Layer0Tally {
         Layer0Tally {
             states,
+            states_by_publish: BTreeMap::new(),
             violations: 0,
             root_persisted_states: 0,
             no_file_states: states,
diff --git a/crates/singlefs-harness/src/fault_injection.rs b/crates/singlefs-harness/src/fault_injection.rs
--- a/crates/singlefs-harness/src/fault_injection.rs
+++ b/crates/singlefs-harness/src/fault_injection.rs
@@ -29,12 +29,14 @@
 
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
 use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
 };
+use singlefs_core::make_filesystem::MKFS_INSTANCE_GENERATION;
 use singlefs_core::recovery::{
-    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
+    choose_root, choose_system_configuration, readable_roots, recover, JournalPolicy, PoolReader,
+    RecoveryOutcome,
 };
 use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
 use singlefs_format::{ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION_MAXIMUM};
@@ -1265,6 +1267,9 @@
     /// 起点段哪一步把错交回来了（`StartingPointStep::name`）：改法二把 `HistoryPool::start` 那四处 `expect`
     /// 改成可失败之后才有这一格，改之前起点段一次都摆不到。
     pub starting_point_failures: BTreeMap<&'static str, u64>,
+    /// 取号之后挂载报错、新号没回卷的次数，按 [`AcquiredInstanceLeftAfterAFailedMount::name`] 分（C378；认下的已知行为，
+    /// 不算新发现，在报告里点名）。
+    pub acquired_instances_left_after_a_failed_mount: BTreeMap<&'static str, u64>,
     pub reopens: u64,
     pub reopens_reading_a_file: u64,
     pub reopens_without_a_file: u64,
@@ -1314,6 +1319,12 @@
         for (key, count) in &other.starting_point_failures {
             *self.starting_point_failures.entry(key).or_insert(0) += count;
         }
+        for (key, count) in &other.acquired_instances_left_after_a_failed_mount {
+            *self
+                .acquired_instances_left_after_a_failed_mount
+                .entry(key)
+                .or_insert(0) += count;
+        }
         self.reopens += other.reopens;
         self.reopens_reading_a_file += other.reopens_reading_a_file;
         self.reopens_without_a_file += other.reopens_without_a_file;
@@ -1418,6 +1429,27 @@
         }
         let _ = writeln!(
             text,
+            "取号之后挂载报错、新号不回卷（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）：用户 2026-09-23 认下的已知行为，D23（journal 的角色与格式） 已定项 16）：{} 次",
+            self.acquired_instances_left_after_a_failed_mount
+                .values()
+                .sum::<u64>()
+        );
+        for kind in [
+            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
+            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot,
+        ] {
+            let _ = writeln!(
+                text,
+                "  {}：{} 次",
+                kind.name(),
+                self.acquired_instances_left_after_a_failed_mount
+                    .get(kind.name())
+                    .copied()
+                    .unwrap_or(0)
+            );
+        }
+        let _ = writeln!(
+            text,
             "重开 {} 次：读回文件 {}、没有文件 {}、走读失败 {}",
             self.reopens,
             self.reopens_reading_a_file,
@@ -1448,12 +1480,46 @@
     }
 }
 
-/// 测量跑里每一步跑完时记下的东西：这一步是什么、到这一步为止读 / 写 / 刷盘各调了多少次、模型到这一步提交过哪些版本。
+/// 测量跑里每一步跑完时记下的东西：这一步是什么、到这一步为止读 / 写 / 刷盘各调了多少次、模型到这一步提交过哪些版本、
+/// 这一步跑完时盘上择到的系统配置带的实例代号（C378 那一格按它判「这一步取了号」）。
 struct StepMark {
     position: StepPosition,
     operation_kind: Option<HistoryOperationKind>,
     calls: FaultCallCounts,
     committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
+    system_configuration_instance: Option<InstanceGeneration>,
+}
+
+/// 取号之后这一步报了错、写进系统配置的新号没有回卷时，重开之后盘上是哪一格（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）；
+/// 用户 2026-09-23 定「认了，写成已知行为」，D23（journal 的角色与格式） 已定项 16 的射程）。这不是失败，是认下来的行为：
+/// 只记账、在报告里点名，不进新发现。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum AcquiredInstanceLeftAfterAFailedMount {
+    /// 写行那次发布的根没落盘：新号一条根都没有，就此烧掉（下一次可写挂载按 max + 1 取下一个，给它写一行）。
+    WithoutAnyRoot,
+    /// 写行那次的根已落盘，之后暖机（或写行那次的系统配置槽轮换）报错：新号有根，挂载照样报错返回。
+    WithTheRowPublishRoot,
+}
+
+impl AcquiredInstanceLeftAfterAFailedMount {
+    /// 计数与报告里的名字。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot => "新号一条根都没有（烧掉）",
+            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot => "新号已有写行那次的根",
+        }
+    }
+}
+
+/// 一次注入落在取号之后、挂载报错、新号没回卷的那一格：注入点、这一步之前与重开之后盘上系统配置的实例代号、根环里最大的实例代号。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct AcquiredInstanceLeft {
+    pub drawn: DrawnFault,
+    pub kind: AcquiredInstanceLeftAfterAFailedMount,
+    pub system_configuration_instance_before_the_step: InstanceGeneration,
+    pub system_configuration_instance_after_the_reopen: InstanceGeneration,
+    pub highest_root_instance_after_the_reopen: Option<InstanceGeneration>,
 }
 
 /// 一段历史注入故障之后交回的东西。
@@ -1466,6 +1532,66 @@
     pub known_red_hits: Vec<KnownRedAtAFault>,
     /// 同一段历史里同一个签名只留第一个。
     pub new_findings: Vec<FaultFinding>,
+    /// 取号之后挂载报错、新号没回卷的那几次（C378；认下的已知行为，不算新发现）。
+    pub acquired_instances_left: Vec<AcquiredInstanceLeft>,
+}
+
+/// 注入那一步是不是一次挂载（起点段里有取号与暖机、可写挂载、回退挂载），是就交回这一步之前盘上系统配置的实例代号。
+/// 起点段之前是 mkfs 写的 0；别的步取测量跑里上一步跑完时的那个。不是挂载的几类操作不取号，交回 `None`。
+fn system_configuration_instance_before_a_mount_step(
+    marks: &[StepMark],
+    segment: FaultedSegment,
+) -> Option<InstanceGeneration> {
+    match segment {
+        FaultedSegment::TheStartingPoint => Some(MKFS_INSTANCE_GENERATION),
+        FaultedSegment::Operation { operation_kind, .. } => match operation_kind {
+            HistoryOperationKind::CloseAndMountWritable
+            | HistoryOperationKind::CloseAndMountRollback => {
+                let index = marks
+                    .iter()
+                    .position(|mark| mark.position == segment.position())?;
+                marks[index.checked_sub(1)?].system_configuration_instance
+            }
+            HistoryOperationKind::PublishFirstFile
+            | HistoryOperationKind::PublishOverwrite
+            | HistoryOperationKind::PublishWithoutUnits
+            | HistoryOperationKind::RaiseRollbackFloor
+            | HistoryOperationKind::ColdStartRecover => None,
+        },
+    }
+}
+
+/// 一次挂载报了错之后重开的盘上，系统配置的实例代号比这一步之前大（这一步取了号、没回卷）：按根环里有没有带新号的根分两格。
+/// 系统配置择不出、或者号没变大（取号自己那几次写报错、回卷做成了），交回 `None`。
+fn acquired_instance_left_after_a_failed_mount(
+    image: &MemoryPool,
+    marks: &[StepMark],
+    drawn: DrawnFault,
+) -> Option<AcquiredInstanceLeft> {
+    let before = system_configuration_instance_before_a_mount_step(marks, drawn.segment)?;
+    let system_configuration = choose_system_configuration(image).ok()?;
+    let after = system_configuration.quantities.journal_instance;
+    if after <= before {
+        return None;
+    }
+    let roots = readable_roots(
+        image,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+    );
+    let kind = if roots.iter().any(|root| root.instance == after) {
+        AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot
+    } else {
+        AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot
+    };
+    Some(AcquiredInstanceLeft {
+        drawn,
+        kind,
+        system_configuration_instance_before_the_step: before,
+        system_configuration_instance_after_the_reopen: after,
+        highest_root_instance_after_the_reopen: roots.iter().map(|root| root.instance).max(),
+    })
 }
 
 /// 这个镜像上，最新那条根带的回退下界 F 落不落在回退留下的空档里（与活盘面、崩溃状态两路同一个谓词、同一份实现）。
@@ -1599,13 +1725,60 @@
     execution: HistoryExecution,
     faults_per_history: usize,
 ) -> HistoryFaultInjection {
-    let geometry = execution.device_width.fixed_geometry();
     let mut tally = FaultInjectionTally {
         histories: 1,
         ..FaultInjectionTally::default()
     };
+    let marks = measure_history(history, execution, &mut tally);
+    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
+    inject_the_drawn_faults(history, execution, &marks, drawn_faults, tally)
+}
 
-    // 一、测量跑：不注入，不跑池级 checker（那一档由随机历史与崩溃注入罩着），只要每一步的调用数与模型目录。
+/// 同 [`inject_faults_into_history`]，注入点不按种子抽，由调用方写死：`segment` 那一段发出的第 `call_within_the_segment` 次
+/// `fault` 那一种调用（1 起）。写死的用例用它把注入点摆在点名的那一次调用上（C378：取号之后的写行、暖机）。
+///
+/// # Panics
+/// 测量跑没走到那一段，或者那一段发出的这一种调用不到 `call_within_the_segment` 次（写死的用例把注入点摆错了）。
+#[must_use]
+pub fn inject_one_fault_into_the_segment(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+    fault: InjectedFault,
+    segment: FaultedSegment,
+    call_within_the_segment: u64,
+) -> HistoryFaultInjection {
+    let mut tally = FaultInjectionTally {
+        histories: 1,
+        ..FaultInjectionTally::default()
+    };
+    let marks = measure_history(history, execution, &mut tally);
+    let index = marks
+        .iter()
+        .position(|mark| mark.position == segment.position())
+        .unwrap_or_else(|| panic!("测量跑没走到{}", segment.render()));
+    let kind = fault.call_kind();
+    let calls_in_the_segment = marks[index].calls.of(kind) - calls_before(&marks, index).of(kind);
+    assert!(
+        (1..=calls_in_the_segment).contains(&call_within_the_segment),
+        "{}只发了 {calls_in_the_segment} 次{}，摆不到第 {call_within_the_segment} 次",
+        segment.render(),
+        kind.name()
+    );
+    let drawn = DrawnFault {
+        fault,
+        call_ordinal: calls_before(&marks, index).of(kind) + call_within_the_segment,
+        segment,
+    };
+    inject_the_drawn_faults(history, execution, &marks, vec![drawn], tally)
+}
+
+/// 测量跑：不注入，不跑池级 checker（那一档由随机历史与崩溃注入罩着），只要每一步的调用数、模型目录与系统配置的实例代号。
+fn measure_history(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+    tally: &mut FaultInjectionTally,
+) -> Vec<StepMark> {
+    let geometry = execution.device_width.fixed_geometry();
     let measurement_execution = HistoryExecution {
         per_step_checker: PerStepChecker::Skipped,
         device_width: execution.device_width,
@@ -1628,6 +1801,9 @@
                 operation_kind: observation.operation.map(HistoryOperation::kind),
                 calls: measurement_plan.calls(),
                 committed_versions: committed_so_far.clone(),
+                system_configuration_instance: choose_system_configuration(observation.image)
+                    .ok()
+                    .map(|system_configuration| system_configuration.quantities.journal_instance),
             });
         },
     );
@@ -1637,21 +1813,32 @@
             tally.histories_whose_measurement_run_stopped_early = 1;
         }
     }
+    marks
+}
 
-    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
+/// 摆好的每个注入点各重跑一遍这段历史（[`inject_one_fault`]），攒成一段的结果。
+fn inject_the_drawn_faults(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+    marks: &[StepMark],
+    drawn_faults: Vec<DrawnFault>,
+    mut tally: FaultInjectionTally,
+) -> HistoryFaultInjection {
     if drawn_faults.is_empty() {
         tally.histories_without_any_injection_point = 1;
     }
     let mut known_red_hits = Vec::new();
     let mut new_findings: Vec<FaultFinding> = Vec::new();
+    let mut acquired_instances_left: Vec<AcquiredInstanceLeft> = Vec::new();
     {
         let mut accumulator = FaultInjectionAccumulator {
             tally: &mut tally,
             known_red_hits: &mut known_red_hits,
             new_findings: &mut new_findings,
+            acquired_instances_left: &mut acquired_instances_left,
         };
         for drawn in &drawn_faults {
-            inject_one_fault(history, execution, &marks, drawn, &mut accumulator);
+            inject_one_fault(history, execution, marks, drawn, &mut accumulator);
         }
     }
     HistoryFaultInjection {
@@ -1660,14 +1847,16 @@
         tally,
         known_red_hits,
         new_findings,
+        acquired_instances_left,
     }
 }
 
-/// 一批注入攒出来的东西：计数、「已知红」命中、新发现。
+/// 一批注入攒出来的东西：计数、「已知红」命中、新发现、取号之后挂载报错而新号没回卷的那几次。
 struct FaultInjectionAccumulator<'run> {
     tally: &'run mut FaultInjectionTally,
     known_red_hits: &'run mut Vec<KnownRedAtAFault>,
     new_findings: &'run mut Vec<FaultFinding>,
+    acquired_instances_left: &'run mut Vec<AcquiredInstanceLeft>,
 }
 
 /// 注入那一段的结局是不是「入口返回了错误」：没 panic、checker 没判红，而且那一段要么返回了 `Err`，
@@ -1911,6 +2100,18 @@
     };
     let violations_after_the_fault = checker_violations_on(&image, accumulator.tally);
     let reopen_text = format!("{reopened:?}（{}）", describe_read_back(&read_back));
+    // C378：注入落在一次挂载里、那一步返回了错误，而重开的盘上系统配置的实例代号比这一步之前大——取号之后的写行或暖机报了错，
+    // 新号没回卷。认下的已知行为：记进这一格、在报告里点名，不进新发现，也不挡下面的判定。
+    if surfaced {
+        if let Some(left) = acquired_instance_left_after_a_failed_mount(&image, marks, *drawn) {
+            *accumulator
+                .tally
+                .acquired_instances_left_after_a_failed_mount
+                .entry(left.kind.name())
+                .or_insert(0) += 1;
+            accumulator.acquired_instances_left.push(left);
+        }
+    }
 
     // 三、判：注入那一步返回错误的那一格不算失败；别的失败（panic、checker 判红、重开走到模型不允许的版本）逐条分类。
     let mut failures: Vec<FailureObservation> = Vec::new();
diff --git a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs b/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
--- a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
@@ -186,7 +186,7 @@
         &file_content(),
     );
     println!(
-        "LAYER0 states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} verification_ran={} verification_failed={} journal_differing={} exhaustive={}",
+        "LAYER0 states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} verification_ran={} verification_failed={} journal_differing={} exhaustive={} states_by_publish=[{}]",
         tally.states,
         tally.violations,
         tally.root_persisted_states,
@@ -196,7 +196,8 @@
         tally.verification_ran_states,
         tally.verification_failed_states,
         tally.journal_differing_states,
-        tally.states == closed_form
+        tally.states == closed_form,
+        tally.states_by_publish_text()
     );
     println!("{}", checker_line(&tally));
     println!("CHECKER_FIRST {:?}", tally.checker_first_violation);
@@ -221,6 +222,12 @@
         }
     );
     assert_eq!(closed_form, 262_165, "全量：枚举到的状态数等于闭式");
+    // 按发布分（里程碑「第二个事务」步 6 验收第 1 条的口径，第一条流同样报）：暖机 txg 1、2 各 3 + 3 + 1，
+    // A 是 18 写段 262143 + 记录段 3 + 根槽段 1，A 的系统配置槽轮换 3，再加全部持久那一个。
+    assert_eq!(
+        tally.states_by_publish_text(),
+        "instance1_txg1=7 instance1_txg2=7 instance1_txg3=262147 after_the_last_root=3 every_write_persisted=1"
+    );
     assert_checker_counts(&tally, 262_165, 4, 4);
 }
 
@@ -254,6 +261,11 @@
         },
         "少展开的只有 18 个写那一段的 262143 个子集：文件在 / 根已持久 / 验证跑过 / journal 承重的状态数都与全量相同"
     );
+    assert_eq!(
+        tally.states_by_publish_text(),
+        "instance1_txg1=7 instance1_txg2=7 instance1_txg3=4 after_the_last_root=3 every_write_persisted=1",
+        "22 个状态按发布分：少的只在 A 那一格（18 写段不展开）"
+    );
     assert_checker_counts(&tally, 22, 4, 4);
     assert_eq!(closed_form_state_count(&prepared.segments), 262_165);
 }
diff --git a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
--- a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
@@ -17,6 +17,7 @@
 use singlefs_core::allocator::{PlacementRefusal, UnitFootprint};
 use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
 use singlefs_core::journal::{back_chain_of, record_offset};
+use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::records::{
     build_mapping_entry, parse_mapping_entry, STATISTIC_ALLOCATED_BYTES,
     STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS,
@@ -31,6 +32,7 @@
 use singlefs_core::transaction::{
     mapping_locations_for_key, placements_to_release_via_mapping, publish_overwrite, FirstFile,
     MappingLookup, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
+    FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::{
     build_index_node, data_unit_payload_capacity, index_node_entry_capacity, parse_index_node,
@@ -349,6 +351,20 @@
         },
         "第一次的内容从最新根出发读不到"
     );
+    // 步 1 验收第 4 条「inode 记录的写入时间留成 A 的 ⇒ 判红」：从盘上读回 inode 记录，不看发布交回的内存结构——
+    // 只读挂载走的是挂载态那一条读路径（`mounted_read`），与发布路径不共用代码。
+    let mounted_read_only = mount_read_only(&reopened).expect("B 之后只读挂载");
+    let first_file_inode_record = mounted_read_only
+        .mounted
+        .inode_records()
+        .iter()
+        .find(|record| record.inode == FIRST_INODE_NUMBER)
+        .expect("第一个文件那条 inode 记录在挂载态里");
+    assert_eq!(
+        first_file_inode_record.write_time_seconds,
+        FIXED_WRITE_TIME_SECONDS + 60,
+        "盘上 inode 记录的写入时间是 B 那一次写入的时间，不是 A 的 {FIXED_WRITE_TIME_SECONDS}"
+    );
 
     let verdicts = check_pool_image(&image);
     for (invariant, verdict) in &verdicts {
diff --git a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
--- a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
@@ -460,10 +460,29 @@
         "1 + 二十个 2 写段各 3 + 两个 4 写段各 15 + 十七个 1 写段各 1"
     );
     println!(
-        "LAYER0B_FAST states={} checker_by_invariant(evaluated/violated/not_applicable) {}",
+        "LAYER0B_FAST states={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
         tally.states,
+        tally.states_by_publish_text(),
         tally.checker_counts_by_invariant()
     );
+    // 步 6 验收第 1 条「每次发布各多少」：按段归到后面最近的那次根槽写（`crash::Layer0PublishOfState`）。
+    // 每次发布记录段 2 写（3）+ 根槽段 1 写（1）= 4；暖机 txg 1、2 各多一段 2 写（取号那段、上一次的系统配置槽轮换）= 7；
+    // 写行 txg 5 与回退 txg 9 各多一段 4 写（上一次的轮换并上取号，15）= 19；18 写与 10 写的段不展开；
+    // 最后一次根槽写之后是 E 的轮换那一段 2 写（3），再加全部持久那一个。
+    assert_eq!(
+        tally.states_by_publish_text(),
+        "instance1_txg1=7 instance1_txg2=7 instance1_txg3=4 instance1_txg4=4 \
+         instance2_txg5=19 instance2_txg6=4 instance2_txg7=4 instance2_txg8=4 \
+         instance3_txg9=19 instance3_txg10=4 instance3_txg11=4 instance3_txg12=4 instance3_txg13=4 \
+         instance3_txg14=4 instance3_txg15=4 instance3_txg16=4 instance3_txg17=4 \
+         after_the_last_root=3 every_write_persisted=1",
+        "平时跑的那 108 个状态按发布分"
+    );
+    assert_eq!(
+        tally.states_by_publish.values().sum::<u64>(),
+        tally.states,
+        "按发布分的各格加起来就是状态数：一个状态不漏、不重"
+    );
     assert_eq!(
         tally.violations, 0,
         "第一处违例：{:?}",
@@ -556,9 +575,10 @@
     );
     let checker_violations: u64 = tally.checker_violated_states.values().sum();
     // 每条不变量报成 `I-x.y=评估过/判违例/不适用` 夹在 checker_violations 与 first_violation 之间（步 6 验收第 3 条：阴性结果与「代码没跑到」分开）；
-    // 54 号门禁只认行首 `LAYER0B ` 与 `exhaustive=true`，整行原样报出来。
+    // 54 号门禁只认行首 `LAYER0B ` 与 `exhaustive=true`，整行原样报出来。按发布分的状态数（步 6 验收第 1 条）夹在
+    // checker 那一段与 first_violation 之间，随整行一起进门禁的成功句。
     println!(
-        "LAYER0B states={} closed_form={closed_form} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} {} first_violation={}",
+        "LAYER0B states={} closed_form={closed_form} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} {} states_by_publish=[{}] first_violation={}",
         tally.states,
         tally.states == closed_form,
         tally.violations,
@@ -572,9 +592,22 @@
         tally.record_root_without_record,
         tally.record_claimed_state_missing_unit,
         tally.checker_counts_by_invariant(),
+        tally.states_by_publish_text(),
         tally.first_violation.as_deref().unwrap_or("none")
     );
     assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
+    // 每次发布：18 写段 262143 或 10 写段 1023，加记录段 3、根槽段 1；写行 txg 5 与回退 txg 9 多一段 4 写（15）；
+    // 暖机 txg 1、2 是 2 写段 + 记录段 + 根槽段（7）。
+    assert_eq!(
+        tally.states_by_publish_text(),
+        "instance1_txg1=7 instance1_txg2=7 instance1_txg3=262147 instance1_txg4=262147 \
+         instance2_txg5=1042 instance2_txg6=1027 instance2_txg7=1027 instance2_txg8=262147 \
+         instance3_txg9=1042 instance3_txg10=1027 instance3_txg11=262147 instance3_txg12=262147 \
+         instance3_txg13=262147 instance3_txg14=262147 instance3_txg15=1027 instance3_txg16=1027 \
+         instance3_txg17=262147 after_the_last_root=3 every_write_persisted=1",
+        "全量 2104413 个状态按发布分"
+    );
+    assert_eq!(tally.states_by_publish.values().sum::<u64>(), tally.states);
     assert_eq!(
         tally.violations, 0,
         "第一处违例：{:?}",
diff --git a/crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs b/crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs
--- a/crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs
+++ b/crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs
@@ -25,7 +25,7 @@
     newest_root_has_the_trees_the_targeted_damages_need, run_bad_disk_campaign, BadDiskCampaign,
     BadDiskFinding, BadDiskInputWorkerThreads, BadDiskReport, DamageKind, KnownPanicSite,
     ReadBackVerdict, ReaderOutcome, BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE,
-    EVERY_DAMAGE_KIND, KNOWN_PANIC_SITES,
+    EVERY_BASE_IMAGE_TIER, EVERY_DAMAGE_KIND, KNOWN_PANIC_SITES,
 };
 use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
 use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
@@ -140,6 +140,42 @@
             report.tally.render()
         );
     }
+    // C481（坏盘输入的基线镜像取不到树表 0 条的盘面）：基线按档抽，每一档都要真抽到过镜像、真造出过坏镜像。
+    // 报告里那一句「基线档 N / M 档抽到过镜像」的 N 由这里钉住——把树表 0 条那一档从抽样里拿掉，N 由 2 变 1，这里红。
+    assert_eq!(
+        report.tally.base_image_tiers_sampled(),
+        EVERY_BASE_IMAGE_TIER.len(),
+        "有基线档一段历史都没抽到镜像：{}",
+        report.tally.render()
+    );
+    assert!(
+        report.render().contains(&format!(
+            "基线档 {} / {} 档抽到过镜像",
+            EVERY_BASE_IMAGE_TIER.len(),
+            EVERY_BASE_IMAGE_TIER.len()
+        )),
+        "报告里报基线档数的那一句：{}",
+        report.render()
+    );
+    for tier in EVERY_BASE_IMAGE_TIER {
+        assert!(
+            report
+                .tally
+                .damaged_images_by_tier
+                .get(tier.name())
+                .copied()
+                .unwrap_or(0)
+                > 0,
+            "基线档「{}」上一份坏镜像都没造出来：{}",
+            tier.name(),
+            report.tally.render()
+        );
+    }
+    assert_eq!(
+        report.tally.damaged_images_by_tier.values().sum::<u64>(),
+        damaged_images,
+        "逐档的坏镜像数加起来就是逐坏法的"
+    );
     assert_eq!(
         report.tally.recoveries, damaged_images,
         "每一份坏镜像都要喂给恢复"
diff --git a/crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs b/crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs
--- a/crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs
+++ b/crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs
@@ -9,7 +9,7 @@
 
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::make_filesystem::{
@@ -26,7 +26,8 @@
 use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
 use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
 use singlefs_harness::fault_injection::{
-    inject_faults_into_history, run_fault_injection_campaign, DrawnFault, FaultCounting,
+    inject_faults_into_history, inject_one_fault_into_the_segment, run_fault_injection_campaign,
+    AcquiredInstanceLeft, AcquiredInstanceLeftAfterAFailedMount, DrawnFault, FaultCounting,
     FaultDeviceSelector, FaultInjectingBlockDevice, FaultInjectionCampaign, FaultInjectionReport,
     FaultInjectionWorkerThreads, FaultOccurrence, FaultOutcome, FaultPlacement, FaultSchedule,
     FaultedSegment, FaultedSegmentKind, FlippedBit, InjectedFault, SharedFaultPlan,
@@ -34,7 +35,8 @@
 use singlefs_harness::history::{
     execute_history_with_faults, generate_history_with_weights, GeneratedHistory,
     GenerationWeights, HarnessJudgement, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
-    HistoryOperationKind, HistoryRun, HistorySeed, PerStepChecker, StartingPointStep,
+    HistoryOperation, HistoryOperationKind, HistoryRun, HistorySeed, HistoryStartingPoint,
+    PerStepChecker, StartingPointStep,
 };
 use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
 use singlefs_harness::segments::{FixedGeometry, StepKind};
@@ -660,6 +662,118 @@
     );
 }
 
+/// 起点（mkfs、取号 1、暖机、第一个文件）之后只做一步：关掉会话、可写挂载（取号 2 → 写行 → 暖机）。
+fn a_history_that_mounts_once_after_the_first_file() -> GeneratedHistory {
+    GeneratedHistory {
+        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![HistoryOperation::CloseAndMountWritable],
+    }
+}
+
+/// 那一步可写挂载里的第几次写：取号是头两次（两块盘各一次系统配置槽写），写行那次发布的第一个单元写是第 3 次；
+/// 写行那次发布一共 15 次写（实例表与四个固定点单元各两盘 10 次、记录两盘 2 次、根槽 1 次、系统配置槽轮换 2 次），
+/// 暖机第一次空发布的第一个单元写是第 2 + 15 + 1 = 18 次。
+const FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT: u64 = 3;
+const FIRST_WARM_UP_WRITE_OF_THE_MOUNT: u64 = 18;
+
+/// C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）按用户 2026-09-23 定的那一边（认了，写成已知行为；
+/// D23（journal 的角色与格式） 已定项 16 的射程）断言：取号之后写行或暖机那一次写报块设备错，挂载返回错误，
+/// 重开的盘上系统配置的实例代号**还是新号 2**（没回卷成 1），这一次记进故障注入的失败账
+/// （`FaultInjectionTally::acquired_instances_left_after_a_failed_mount`）、报告里点名。两格各一条：
+/// 写行那次的第一个单元写报错 ⇒ 新号一条根都没有（烧掉）；暖机第一次的第一个单元写报错 ⇒ 新号已有写行那次的根。
+///
+/// 判别力自证：把挂载改成「发布报错之后把系统配置的实例代号写回旧号」（回卷，`crates/mutations.tsv` 里那一条），
+/// 重开的盘上系统配置是 1，这一格记不上，这里红。
+#[test]
+fn a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it(
+) {
+    let history = a_history_that_mounts_once_after_the_first_file();
+    let mount_step = FaultedSegment::Operation {
+        step_index: 0,
+        operation_kind: HistoryOperationKind::CloseAndMountWritable,
+    };
+    for (call_within_the_mount, expected_kind, expected_highest_root_instance) in [
+        (
+            FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT,
+            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
+            InstanceGeneration(1),
+        ),
+        (
+            FIRST_WARM_UP_WRITE_OF_THE_MOUNT,
+            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot,
+            InstanceGeneration(2),
+        ),
+    ] {
+        let injection = inject_one_fault_into_the_segment(
+            &history,
+            CHECKED_PAST_THE_RING_TURN,
+            InjectedFault::WriteFails,
+            mount_step,
+            call_within_the_mount,
+        );
+        let rendered = injection.tally.render();
+        print_uncaptured(&format!(
+            "── C378：可写挂载第 {call_within_the_mount} 次写报错 ──\n{rendered}"
+        ));
+        assert_eq!(
+            injection.acquired_instances_left,
+            vec![AcquiredInstanceLeft {
+                drawn: injection.drawn_faults[0],
+                kind: expected_kind,
+                system_configuration_instance_before_the_step: InstanceGeneration(1),
+                system_configuration_instance_after_the_reopen: InstanceGeneration(2),
+                highest_root_instance_after_the_reopen: Some(expected_highest_root_instance),
+            }],
+            "取号之后第 {call_within_the_mount} 次写报错：重开的盘上系统配置还是新号 2、不回卷成 1\n{rendered}"
+        );
+        assert_eq!(
+            injection.tally.acquired_instances_left_after_a_failed_mount,
+            BTreeMap::from([(expected_kind.name(), 1)]),
+            "失败账记下这一次\n{rendered}"
+        );
+        assert!(
+            rendered.contains("取号之后挂载报错、新号不回卷（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）")
+                && rendered.contains(&format!("  {}：1 次", expected_kind.name())),
+            "报告里点名这一格\n{rendered}"
+        );
+        // 注入真打在那一步可写挂载的单元写上，那一步返回了错误：不是别处先停了。
+        assert_eq!(
+            injection.tally.faults_by_injection_point,
+            BTreeMap::from([("CloseAndMountWritable/unit_write".to_string(), 1)]),
+            "注入点\n{rendered}"
+        );
+        assert_eq!(
+            injection
+                .tally
+                .faults_by_outcome
+                .get(FaultOutcome::SurfacedAsAnError.name())
+                .copied(),
+            Some(1),
+            "可写挂载返回了错误\n{rendered}"
+        );
+        // 记下的错误成员恰好一次、而且是发布那一步的：空表上「每一个都是」恒真，所以先钉条数。
+        assert_eq!(
+            injection.tally.refusal_members.values().sum::<u64>(),
+            1,
+            "注入那一步返回的错误成员记了一次\n{rendered}"
+        );
+        assert!(
+            injection
+                .tally
+                .refusal_members
+                .keys()
+                .all(|member| member.starts_with("MountError::Publish(")),
+            "返回的是发布那一步的错误成员\n{rendered}"
+        );
+        assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
+        assert!(
+            injection.new_findings.is_empty(),
+            "认下的行为不算新发现，别的也不许有\n{rendered}"
+        );
+    }
+}
+
 /// 发布 B 的内容（4100 字节，与虚机二进制 `second-transaction` 模式相同）。
 fn second_content() -> Vec<u8> {
     (0..4100usize)
diff --git a/crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs b/crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs
@@ -0,0 +1,207 @@
+//! 里程碑「第二个事务」步 3 验收的变异「取号之后没有屏障 ⇒ 层 0 里出现「系统配置已是 2、单元却先持久」的状态，
+//! I-7.7（系统配置实例代号不低于根环） 那一族判红」要红在的那一条层 0 用例。
+//!
+//! 流：mkfs → 取号 1 → 暖机 → A → B（同一个进程）→ 进程退出、重开可写挂载（取号 2 → 写行 → 暖机）。
+//! 取号那两次系统配置槽写与 B 的两次系统配置槽轮换同一段（进程退出与重开之间没有屏障，登记表八那条 ⚠️），
+//! 取号那道屏障把它们与写行那次发布的单元写隔开（D23（journal 的角色与格式） 已定项 16「取号那一步的屏障」）。
+//! 平时展开的只有这两段：取号所在的那一段、取号之后第一个带单元写的那一段——少了取号那道屏障，两段并成一段，
+//! 枚举就摆得出「实例 2 的单元已持久、两块盘的系统配置还都是 1」的状态。
+//! 展开哪几段按写在录制流里的位置挑（重开之后的第一个写起），不按段的写数挑：并段之后段变长，按写数挑会把并出来的那一段漏掉。
+//!
+//! 这条流的段序列就是第二条流（`second_transaction_step_zero_layer0.rs`）发 C 之前那一截：前 22 段逐段相同，
+//! 末段是暖机第二次的系统配置槽轮换（第二条流里它与 C 的单元写并成 18 写一段）。展开的这两段在第二条流里是第 13、14 段，
+//! 全量（门禁 54 号 `--full`）已经罩着；这一条不多罩崩溃状态，多的是在平时的 `cargo test` 里展开它们、按 I-7.7 判——
+//! 第二条流的快用例只展开写数小于 10 的段，写行那一段（10 写）与并段之后的 14 写一段都不展开。
+
+mod common;
+
+use common::{
+    build_pool, file_content, geometry, parameters, publish_overwrite_in_process,
+    FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::mount::mount_writable;
+use singlefs_harness::crash::{
+    closed_form_state_count, enumerate_layer0_selecting_versions,
+    writes_and_segments_with_stream_indexes, PublishedVersion,
+};
+use singlefs_harness::segments::StepKind;
+
+/// B 的内容：与第一个文件不同长、不同字节（同 `second_transaction_step_one_overwrite.rs` 的第二次写）。
+const SECOND_FILE_BYTES: usize = 4100;
+
+fn second_content() -> Vec<u8> {
+    (0..SECOND_FILE_BYTES)
+        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// I-7.7（系统配置实例代号不低于根环） 在池级 checker 里的名字。
+const SYSTEM_CONFIGURATION_INSTANCE_INVARIANT: &str = "I-7.7";
+
+#[test]
+fn no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount(
+) {
+    let mut pool = build_pool("acquisition-barrier-layer0");
+    let previous = pool.output.clone();
+    let second = publish_overwrite_in_process(
+        &mut pool,
+        &previous,
+        &second_content(),
+        FIXED_WRITE_TIME_SECONDS + 60,
+        InstanceGeneration(1),
+    )
+    .expect("覆盖写 B");
+    pool.output = second;
+    let operations_before_the_remount = pool.stream.operations().len();
+    let mut devices = pool.reopen_recorded();
+    let mounted = mount_writable(&parameters(), &mut devices).expect("重开之后可写挂载");
+    pool.devices = Some(devices);
+    assert_eq!(mounted.output.instance, InstanceGeneration(2), "重开取号 2");
+
+    let mut versions = vec![
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(3),
+            content: file_content(),
+        },
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(4),
+            content: second_content(),
+        },
+    ];
+    for version in
+        std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter())
+    {
+        versions.push(PublishedVersion {
+            instance: version.root().instance,
+            checkpoint_txg: version.root().checkpoint_txg,
+            content: second_content(),
+        });
+    }
+
+    let base = pool.memory_pool_after_mkfs();
+    let operations = pool.retained_operations();
+    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let first_stream_index_of_the_remount =
+        operations_before_the_remount - pool.mkfs_operation_count;
+    let remount_writes: Vec<usize> = (0..writes.len())
+        .filter(|write_index| stream_indexes[*write_index] >= first_stream_index_of_the_remount)
+        .collect();
+    let first_remount_write = *remount_writes.first().expect("重开之后发过写");
+    assert_eq!(
+        remount_writes[..2]
+            .iter()
+            .map(|write_index| writes[*write_index].kind)
+            .collect::<Vec<StepKind>>(),
+        vec![StepKind::SystemConfigurationSlot; 2],
+        "重开之后头两个写是取号那两次系统配置槽写（两块盘各一次）"
+    );
+    let first_unit_write_of_the_remount = *remount_writes
+        .iter()
+        .find(|write_index| writes[**write_index].kind == StepKind::UnitWrite)
+        .expect("写行那次发布写单元");
+    let segment_of = |write_index: usize| {
+        segments
+            .iter()
+            .position(|segment| segment.contains(&write_index))
+            .expect("每个写都在某一段里")
+    };
+    let acquisition_segment = segment_of(first_remount_write);
+    let first_unit_segment = segment_of(first_unit_write_of_the_remount);
+    let expand = |segment_index: usize, _segment: &[usize]| {
+        segment_index == acquisition_segment || segment_index == first_unit_segment
+    };
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("写流里有根槽写");
+    let tally = enumerate_layer0_selecting_versions(
+        &base,
+        &writes,
+        &segments,
+        judged_root_index,
+        &versions,
+        &expand,
+    );
+    let expanded: Vec<Vec<usize>> = segments
+        .iter()
+        .enumerate()
+        .filter(|(segment_index, segment)| expand(*segment_index, segment))
+        .map(|(_, segment)| segment.clone())
+        .collect();
+    println!(
+        "LAYER0_ACQUISITION_BARRIER states={} expanded_segment_sizes={:?} checker_by_invariant(evaluated/violated/not_applicable) {}",
+        tally.states,
+        expanded.iter().map(Vec::len).collect::<Vec<usize>>(),
+        tally.checker_counts_by_invariant()
+    );
+
+    // 变异「取号之后没有屏障」要红在这一条上：新实例的单元先于新号持久，I-7.7 ① 判红。
+    assert_eq!(
+        tally
+            .checker_violated_states
+            .get(SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
+            .copied()
+            .unwrap_or(0),
+        0,
+        "取号那一段与写行的第一段展开之后，出现了新实例的写先于新号持久的状态：{:?}",
+        tally
+            .checker_first_violation
+            .get(SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
+    );
+    assert_eq!(
+        tally
+            .checker_evaluated_states
+            .get(SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
+            .copied()
+            .unwrap_or(0),
+        tally.states,
+        "I-7.7 在展开出来的每个状态上都真被评估过（阴性结果与「代码没跑到」分开）：{}",
+        tally.checker_counts_by_invariant()
+    );
+    assert_eq!(
+        tally.violations, 0,
+        "多版本 oracle 的第一处违例：{:?}",
+        tally.first_violation
+    );
+    assert_eq!(
+        tally.ignored_violations, 0,
+        "不看 journal 那一遍恢复同样过 oracle：{:?}",
+        tally.first_ignored_violation
+    );
+    let checker_violations: u64 = tally.checker_violated_states.values().sum();
+    assert_eq!(
+        checker_violations, 0,
+        "别的不变量也不许判红：{:?}",
+        tally.checker_first_violation
+    );
+
+    // 状态数钉绝对值：取号那一段是 B 的两次系统配置槽轮换并上取号两次（4 写，15 个真子集），
+    // 写行那次发布的单元写一段（实例表 + 四个固定点单元，各两盘：10 写，1023 个），再加全部持久那一个。
+    assert_ne!(
+        acquisition_segment, first_unit_segment,
+        "取号那一段与写行的第一段之间隔着一道屏障"
+    );
+    assert_eq!(
+        expanded.iter().map(Vec::len).collect::<Vec<usize>>(),
+        vec![4, 10]
+    );
+    assert_eq!(tally.states, closed_form_state_count(&expanded));
+    assert_eq!(tally.states, 1 + 15 + 1023);
+    // 与第二条流发 C 之前那一截逐段相同（门禁 52 号核的那个数组的前 22 段），末段 2 写是暖机第二次的系统配置槽轮换。
+    assert_eq!(
+        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
+        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 2],
+        "段序列是第二条流发 C 之前那一截"
+    );
+    assert_eq!(
+        (acquisition_segment, first_unit_segment),
+        (12, 13),
+        "展开的是第二条流的第 13、14 段（从 1 数）"
+    );
+}
```

## 二、harness 那一组新增的变异名（`/tmp/claude-1000/impl-harness-batch/mutations-append.tsv`，7 行，原样，还没合并进 `crates/mutations.tsv`）

sha256sum：`5c284045bed4033ef0d50a01de8a73f96d3345b0ba8c645d2df6fd24e106e78e`（引自实现员报告第八节，本节内容原样拷自该文件）。

```tsv
步 1 验收第 4 条：extent 叶记录的指针忘了换（覆盖写之后仍指上一版的数据单元，读回等于旧内容）	crates/singlefs-core/src/transaction.rs	        (Some(file), _) => build_file_version_units(\n            &checkpoint,\n            file,\n            FileVersionSlots {\n                data: slot_of(TransactionUnit::Data),\n            },\n            &mut sequences,\n        ),	        (Some(file), _) => {\n            let mut built = build_file_version_units(\n                &checkpoint,\n                file,\n                FileVersionSlots {\n                    data: slot_of(TransactionUnit::Data),\n                },\n                &mut sequences,\n            );\n            if let Some(previous_version) = previous {\n                let stale_extent_record =\n                    build_extent_record(FIRST_INODE_NUMBER, 0, previous_version.data_pointer);\n                let stale_extent_key: [u8; 24] =\n                    stale_extent_record[..24].try_into().expect("24");\n                built.extent_unit = build_index_node(\n                    TreeIdentifier(TREE_IDENTIFIER_EXTENT),\n                    0,\n                    24,\n                    &stale_extent_key,\n                    &stale_extent_key,\n                    txg,\n                    filesystem_identifier,\n                    instance,\n                    built.extent_sequence,\n                    112,\n                    std::slice::from_ref(&stale_extent_record),\n                );\n            }\n            built\n        }	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
步 1 验收第 4 条：inode 记录的写入时间留成 A 的（覆盖写照抄上一版的写入时间）	crates/singlefs-core/src/transaction.rs	                write_time_seconds: file.write_time_seconds,\n                inode_object_birth: previous.inode_record.object_birth,	                write_time_seconds: previous.inode_record.write_time_seconds,\n                inode_object_birth: previous.inode_record.object_birth,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
步 1 验收第 4 条：反向链算成 A 之前那条（覆盖写的反向链取上一版记录自己的反向链）	crates/singlefs-core/src/transaction.rs	            back_chain: back_chain_of(&previous.record_bytes),\n            file: Some(FileVersionPlan {	            back_chain: previous.record.back_chain,\n            file: Some(FileVersionPlan {	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = pool.perform(CommitStep::Barrier) {\n        return Err(pool.roll_back_acquisition(&written, previous_instance, cause));\n    }\n    Ok(instance)	    Ok(instance)	-p singlefs-harness --test second_transaction_step_three_acquisition_barrier_layer0 -- no_unit_of_the_new_instance	no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount
步 6 验收第 1 条：层 0 按发布分状态数时根槽写那一段归到下一次发布（按段归发布错一位）	crates/singlefs-harness/src/crash.rs	            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n        }	            publish_of_segment[segment_index] =\n                next_root.unwrap_or(Layer0PublishOfState::AfterTheLastRoot);\n            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n            continue;\n        }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
C481：坏盘输入的基线抽样里去掉「树表 0 条」那一档（报出的基线档数由 2 变 1）	crates/singlefs-harness/src/bad_disk_input.rs	            BaseImageTier::TreeTableWithoutEntries => newest_root_tree_table_has_no_entries(image),	            BaseImageTier::TreeTableWithoutEntries => false,	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- bad_disk_inputs_never_read_back	bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites
C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n        },\n    )\n}	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(parameters, devices.as_mut_slice());\n        let _ = rollback_writer.perform(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_write_error_after_the_acquisition	a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
```

## 三、checker 那一组：`/tmp/claude-1000/impl-checker-batch/checker.patch`（原样拷 `research/prompts/m2-wave3-code-r1-patches/checker.patch`）

基准：主工作区 2026-09-23 22:57 UTC 现状（HEAD `3b60f098e97dc4c4f3ed9c6355422b607db1c34c` + 当时的未提交改动）；
`research/prompts/m2-wave3-checker-implementer-report.md` 第九节 2026-09-23 23:21:12 UTC 在主工作区原样跑 `git apply --check`，6 个文件全部 `Checking patch …` 后 `apply-check exit 0`。
`sha256sum checker.patch` = `3e8e99daf65c5350d0041851e20de5bebead15c5d4ba830105a9825eea7116cc`（与 `m2-wave3-code-r1-patches/checker.patch` 一致，已核对）。

```diff
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -34,11 +34,11 @@
 }
 
 /// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
-pub const IMPLEMENTED_INVARIANTS: [&str; 39] = [
+pub const IMPLEMENTED_INVARIANTS: [&str; 40] = [
     "I-1.1", "I-1.2", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-1.8", "I-1.10", "I-2.1", "I-2.3",
-    "I-2.4", "I-2.5", "I-3.1", "I-3.8", "I-3.9", "I-4.2", "I-4.8", "I-5.1", "I-5.2", "I-5.4",
-    "I-7.1", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-8.6", "I-8.7", "I-8.8",
-    "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14",
+    "I-2.4", "I-2.5", "I-3.1", "I-3.8", "I-3.9", "I-3.10", "I-4.2", "I-4.8", "I-5.1", "I-5.2",
+    "I-5.4", "I-7.1", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-8.6", "I-8.7",
+    "I-8.8", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14",
 ];
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、以及整条不适用的理由。
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -428,18 +428,40 @@
             KEY_SCHEMA_ALLOCATION,
             "树表 0 条那一版的分配记录树的根",
         );
+        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
+        self.walk_tree_table_and_central_mapping_root(&record[36..122], &record[256..342]);
+    }
+
+    /// 由记录施加出来、根槽从没落盘的那一版（[`versions_applied_only_by_records`] 认出来的）：它没有根记录，
+    /// 树表指针与映射根指针在那条记录的新根段里，从那里走下去，判定与根环里的候选根同一套。
+    /// 根记录里另外两样这里不走，记录的新根段（188 字节）里没有它们的位置：
+    /// 实例表指针——覆盖写不重写实例表，这一版的实例表就是它施加在其上那一版的，那一版在候选集里时已经走过；
+    /// 「树表 0 条那一版的分配记录树的根」——带文件的一版这一项恒全零。
+    /// 那一版不在候选集里、或者由记录施加出来的是树表 0 条的一版时，这两样这里就漏数了：这一格条款没写，没有定案之前照这样走。
+    fn walk_version_applied_only_by_records(&mut self, version: &VersionAppliedOnlyByRecords) {
+        let (tree_table_pointer, mapping_root_pointer) =
+            version.tree_table_and_mapping_root_pointers();
+        self.walk_tree_table_and_central_mapping_root(tree_table_pointer, mapping_root_pointer);
+    }
+
+    /// 一版的树表（树 ID 0）与它下面每一条树表条目，再加中央映射树的根：根环里的根从根记录里取这两条指针，
+    /// 由记录施加出来的那一版从记录的新根段里取（两处都是 86 字节的码 2 指针）。树表读不出时映射根也不走（走读已经断了）。
+    fn walk_tree_table_and_central_mapping_root(
+        &mut self,
+        tree_table_pointer: &[u8],
+        mapping_root_pointer: &[u8],
+    ) {
         // 树表单元：不属于任何树（树 ID 0）。
         let Some(tree_table) =
-            self.read_index_node(&record[36..122], 0, KEY_SCHEMA_TREE_TABLE, "树表单元")
+            self.read_index_node(tree_table_pointer, 0, KEY_SCHEMA_TREE_TABLE, "树表单元")
         else {
             return;
         };
         for entry in &tree_table.entries {
             self.walk_tree_table_entry(entry);
         }
-        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
         if let Some(mapping) = self.read_index_node(
-            &record[256..342],
+            mapping_root_pointer,
             TREE_IDENTIFIER_CENTRAL_MAPPING,
             KEY_SCHEMA_MAPPING,
             "中央映射树的根",
@@ -459,11 +481,10 @@
                 self.walk_central_mapping_entries(&mapping);
             }
         }
-        let _ = is_newest;
     }
 
     /// 中央映射树根的条目逐条走：每条条目里那两条位置条目按升序判，指的单元两份各读一遍。
-    /// 入参的条目宽由调用方判过 ≥ 55（`walk_root` 那一道），这里按字段表的固定偏移切。
+    /// 入参的条目宽由调用方判过 ≥ 55（`walk_tree_table_and_central_mapping_root` 那一道），这里按字段表的固定偏移切。
     fn walk_central_mapping_entries(&mut self, mapping: &crate::IndexNodeView) {
         for entry in &mapping.entries {
             let locations_pointer: Vec<u8> =
@@ -1529,6 +1550,139 @@
     }
 }
 
+/// 一个起点槽上的单元头能不能用、能用时头里的诞生代号（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的偏移表：
+/// 码 1 诞生代号 75、明文头末尾 105；码 2 诞生代号 52 + 2k、明文头末尾 86 + 2k；码 3 诞生代号 73、明文头末尾 107；
+/// 头校验和三类都在 10）。「能用」= magic 对、类标签是 1 / 2 / 3、头校验和过——头读不出、类标签不认、头校验和不过的
+/// 这一格没有对象（I-3.10 那一行射程 ③：读不出本身由 I-1.1 与 I-2.1 管）。
+fn birth_txg_in_a_usable_unit_header(header: &[u8]) -> Option<u64> {
+    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
+        return None;
+    }
+    let (plain_header_end, birth_txg_offset) = match header[6] {
+        1 => (105, 75),
+        2 => {
+            let key_span = 2 * usize::from(header[51]);
+            (86 + key_span, 52 + key_span)
+        }
+        3 => (107, 73),
+        _outside_the_three_unit_classes => return None,
+    };
+    checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET)
+        .then(|| read_u64(header, birth_txg_offset))
+}
+
+/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）读哪几片分配记录：候选集里每条根的分配记录树
+/// （树表里种类 3 那一条指着的根），树表 0 条那一版的分配记录树（根记录直接持有，C512（树表 0 条的一版上被换下的单元记在哪）），
+/// 以及由记录施加出来的那几版的分配记录树（它们的树表在记录的新根段里）。只读不判：判定归 I-2.1 / I-1.1 那几条的走读。
+fn allocation_record_node_pointers_of_the_candidate_versions(
+    reader: &dyn ImageReader,
+    roots: &[(u64, u64, crate::RootView)],
+    candidate_indexes: &[usize],
+    versions_applied_only_by_records: &[VersionAppliedOnlyByRecords],
+    cache: &mut IndexNodeCache,
+) -> Vec<PointerView> {
+    let mut tree_table_pointers: Vec<PointerView> = Vec::new();
+    let mut allocation_node_pointers: Vec<PointerView> = Vec::new();
+    for index in candidate_indexes.iter().copied() {
+        let record = &roots[index].2.record_bytes;
+        tree_table_pointers.push(parse_node_pointer(&record[36..122]));
+        allocation_node_pointers.push(parse_node_pointer(&record[342..428]));
+    }
+    for version in versions_applied_only_by_records {
+        let (tree_table_pointer, _) = version.tree_table_and_mapping_root_pointers();
+        tree_table_pointers.push(parse_node_pointer(tree_table_pointer));
+    }
+    for tree_table_pointer in &tree_table_pointers {
+        if tree_table_pointer.all_zero {
+            continue;
+        }
+        let Some(tree_table) = read_index_node_without_judging(reader, tree_table_pointer, cache)
+        else {
+            continue;
+        };
+        for entry in &tree_table.entries {
+            if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION
+            {
+                continue;
+            }
+            allocation_node_pointers.push(parse_node_pointer(&entry[14..100]));
+        }
+    }
+    allocation_node_pointers.retain(|pointer| !pointer.all_zero);
+    allocation_node_pointers
+}
+
+/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）：任一**未带已释放标志**的分配记录，它的分配代等于它罩住的那个单元头里的
+/// 诞生代号——记录与单元是同一次发布写出来的，两个数同源（`.claude/kb/invariants.md` I-3.10 那一行）。射程照那一行：
+/// ① 只管未释放的记录（已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里））；
+/// ② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头——记录的 key 就是起点槽，按它读；
+/// ③ 那个槽上读不出可用的单元头时这一格没有对象，不判（[`birth_txg_in_a_usable_unit_header`]）。
+/// 它拦的是：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着。
+/// 几条根指着同一个分配记录树节点时（照抄上一版的根）那个节点只读一次；同一条记录（盘、槽、代）出现在几片里只判一次。
+/// 一格都没判到时整条报不适用并带理由，不报成立。
+fn judge_allocation_generations_against_unit_births(
+    reader: &dyn ImageReader,
+    allocation_node_pointers: &[PointerView],
+    cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
+    let mut examined_records: BTreeSet<(u32, u64, u64)> = BTreeSet::new();
+    let mut judged_records = 0u64;
+    for pointer in allocation_node_pointers {
+        let first_location = &pointer.locations[0];
+        if !seen_allocation_nodes.insert((
+            first_location.device,
+            first_location.slot,
+            first_location.checksum,
+        )) {
+            continue;
+        }
+        let Some(node) = read_index_node_without_judging(reader, pointer, cache) else {
+            continue;
+        };
+        if node.level > 0 || node.entry_width < allocation_record_bytes() {
+            continue;
+        }
+        for entry in &node.entries {
+            let record = parse_allocation_record(entry);
+            if record.is_released
+                || !examined_records.insert((record.device, record.slot, record.generation))
+            {
+                continue;
+            }
+            let Some(birth_txg) = reader
+                .read(
+                    record.device,
+                    record.slot * SLOT_BYTES,
+                    UNIT_HEADER_SCAN_BYTES,
+                )
+                .as_deref()
+                .and_then(birth_txg_in_a_usable_unit_header)
+            else {
+                continue;
+            };
+            judged_records += 1;
+            judgements.judge("I-3.10", record.generation == birth_txg, || {
+                format!(
+                    "盘 {} 槽 {} 的分配记录（未释放、跨 {} 槽）分配代 {}，它罩住的单元头里的诞生代号是 {birth_txg}",
+                    record.device, record.slot, record.span_slots, record.generation
+                )
+            });
+        }
+    }
+    if judged_records == 0 {
+        judgements.not_applicable(
+            "I-3.10",
+            if examined_records.is_empty() {
+                "候选集里没有一片分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、有内部节点）"
+            } else {
+                "未释放的分配记录罩住的起点槽上一个可用的单元头都读不出：读不出本身归 I-1.1 与 I-2.1"
+            },
+        );
+    }
+}
+
 /// 一个分配记录树叶上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
 fn judge_allocation_record_ranges_of_one_node(
     node: &crate::IndexNodeView,
@@ -2076,6 +2230,11 @@
 /// 扫描方向读到的一条 journal 记录：它的实例代号、事务号、落在哪个环槽、反向链字段，以及「下一条记录的反向链该等于的值」。
 struct ScannedJournalRecord {
     instance: u32,
+    /// 记录头里的 checkpoint_txg：这条记录施加出来的是哪一版（「由记录施加出来、根槽从没落盘的那一版」按它认）。
+    checkpoint_txg: u64,
+    /// 记录头里的新根段 188 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8，D23（journal 的角色与格式） 已定项 4）：
+    /// 这条记录施加出来的那一版从哪个树表、哪个映射根走下去。
+    new_root_segment: Vec<u8>,
     /// 记录头里的事务号（D23（journal 的角色与格式） 已定项 7）：I-8.7（实例内事务号不重号） 判的就是它。
     transaction: u64,
     /// 记录头里的提交标记：I-8.8（前缀里的事务不被切开） 判的就是它。
@@ -2111,6 +2270,8 @@
             record.counter,
             ScannedJournalRecord {
                 instance: record.instance,
+                checkpoint_txg: record.checkpoint_txg,
+                new_root_segment: record.new_root_segment,
                 transaction: record.transaction,
                 commit_marker: CommitMarker::of(record.commit_marker_byte),
                 slot,
@@ -2406,6 +2567,101 @@
     }
 }
 
+/// 记录新根段里两条指针的落点（D23（journal 的角色与格式） 已定项 4 的字段表：树表指针 86、映射根指针 86，再往后是树 ID 水位 8 与 F 8）。
+const NEW_ROOT_SEGMENT_TREE_TABLE_POINTER: std::ops::Range<usize> = 0..86;
+const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 86..172;
+
+/// 由记录施加出来、根槽从没落盘的那一版：一条 journal 记录自带新根段（树表指针与映射根指针），恢复把它施加在所选根之上，
+/// 这一版就成了现行版；之后写行那次发布把它换下的单元放进 defer——那些单元仍占着空间、仍算在「已分配」里，
+/// 而根环里没有任何一条根引用它们（它自己的根槽一次都没写过）。
+struct VersionAppliedOnlyByRecords {
+    instance: u32,
+    checkpoint_txg: u64,
+    new_root_segment: Vec<u8>,
+}
+
+impl VersionAppliedOnlyByRecords {
+    fn tree_table_and_mapping_root_pointers(&self) -> (&[u8], &[u8]) {
+        (
+            &self.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
+            &self.new_root_segment[NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER],
+        )
+    }
+}
+
+/// 回退候选集要补上的那几版：**由记录施加出来、根槽从没落盘的那一版**（增补 2 收口表第 54 行，
+/// 2026-09-23 用户定候选 b，`research/prompts/m2-placement-falsepositive-r1-main-verification.md` Z3）。
+/// 只看盘上这一份镜像能读出的东西，四条同时成立才算一版：
+///
+/// ① **它被施加过**：最新那条根指着的实例表里有这个实例的行 (i, Ti, Wi)，而这条记录的 checkpoint_txg ≤ Ti。
+///    行里的 Ti 就是那次恢复停在哪一版（`crates/singlefs-core/src/mount.rs` 写行时取恢复之后的有效根的 txg），
+///    施加出来的每一版的 txg 都不超过它；没有这个实例的行 ⇒ 还没有哪一次恢复把它施加出来，环里的记录只是残留
+///    （层 0 残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格：施加要等下一次挂载，这一刻还不算）。
+///    记录要带提交标记（D23（journal 的角色与格式） 已定项 7：不带的是恢复要丢的尾巴）。
+/// ② **它的根槽读不出**：根环里自证过的根没有一条是 (i, 这个 txg)。读得出的那一版照常由根环那一路走。
+/// ③ **不低于回退下界**：txg ≥ 最新根带的 F，与根环里的候选根同一条（D16（发布语义） 已定项 1）。
+/// ④ **它换下的单元还没到回收的时候**：根环里有一条没被实例表判抛弃的根，txg 比它小。
+///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 环里最旧有效根)」
+///    （D3（空间分配） 已定项 7）。环里最旧的有效根比这一版老 ⇒ 最旧有效根 < 这一版 < 释放代，③ 又让 F ≤ 这一版 ⇒ 回收不了，
+///    它们还算在已分配里；反过来环里最旧的有效根已经比这一版新，这一版里只有它引用的单元释放代都不超过那条根，已可回收复用，
+///    再把它并进来就是拿已经不归它的槽去比。
+///
+/// 一版由那条记录的 (实例代号, checkpoint_txg) 认；两块盘各一份镜像记录，取先读到的那一份。
+fn versions_applied_only_by_records(
+    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
+    roots: &[(u64, u64, crate::RootView)],
+    instance_table_rows: &[InstanceTableRow],
+    rollback_floor: u64,
+) -> Vec<VersionAppliedOnlyByRecords> {
+    let is_abandoned = |instance: u32, checkpoint_txg: u64| {
+        instance_table_rows
+            .iter()
+            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)
+    };
+    let Some(oldest_valid_root_txg) = roots
+        .iter()
+        .filter(|(_, _, root)| !is_abandoned(root.instance, root.checkpoint_txg))
+        .map(|(_, _, root)| root.checkpoint_txg)
+        .min()
+    else {
+        return Vec::new();
+    };
+    let mut versions: BTreeMap<(u32, u64), Vec<u8>> = BTreeMap::new();
+    for (_, records) in records_by_device {
+        for record in records.values() {
+            let applied_by_a_recovery = record.commit_marker == CommitMarker::Present
+                && instance_table_rows.iter().any(|row| {
+                    row.instance == record.instance
+                        && record.checkpoint_txg <= row.published_checkpoint_txg
+                });
+            let root_slot_is_readable = roots.iter().any(|(_, _, root)| {
+                root.instance == record.instance && root.checkpoint_txg == record.checkpoint_txg
+            });
+            let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;
+            let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;
+            if applied_by_a_recovery
+                && !root_slot_is_readable
+                && at_or_above_the_floor
+                && its_units_are_not_reclaimable_yet
+            {
+                versions
+                    .entry((record.instance, record.checkpoint_txg))
+                    .or_insert_with(|| record.new_root_segment.clone());
+            }
+        }
+    }
+    versions
+        .into_iter()
+        .map(
+            |((instance, checkpoint_txg), new_root_segment)| VersionAppliedOnlyByRecords {
+                instance,
+                checkpoint_txg,
+                new_root_segment,
+            },
+        )
+        .collect()
+}
+
 /// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
 #[must_use]
 pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
@@ -2592,6 +2848,36 @@
                 });
         }
     }
+    // 回退候选集补上「由记录施加出来、根槽从没落盘的那一版」（2026-09-23 用户定候选 b）：它们没有根槽，
+    // 上面按根环走的那一遍走不到，而它们换下的单元还在 defer 里、仍算在已分配里（增补 2 收口表第 54 行那 12 个状态差的 65 536 字节）。
+    // 走法与判定照候选根：引用进同一个并集（I-3.1 / I-5.1），单元照判 I-2.1 等，I-7.4 / I-4.8 每一版各判一格。
+    let versions_applied_only_by_records = versions_applied_only_by_records(
+        &journal_records_by_device,
+        &roots,
+        &instance_table_rows,
+        newest_rollback_floor,
+    );
+    for version in &versions_applied_only_by_records {
+        let mismatches_before = walk.judgements.violation_count("I-2.1");
+        let failures_before = walk.walk_failures.len();
+        walk.walk_version_applied_only_by_records(version);
+        let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
+            > mismatches_before
+            || walk.walk_failures.len() > failures_before;
+        let (instance, version_txg) = (version.instance, version.checkpoint_txg);
+        walk.judgements
+            .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
+                format!(
+                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）"
+                )
+            });
+        walk.judgements
+            .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
+                format!(
+                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）出发的遍历有单元对不上或读不出"
+                )
+            });
+    }
     let referenced_units_judged_against_their_pointer =
         walk.referenced_units_judged_against_their_pointer;
     let mut judgements = walk.judgements;
@@ -2640,6 +2926,19 @@
         &mut index_node_cache,
         &mut judgements,
     );
+    let allocation_node_pointers = allocation_record_node_pointers_of_the_candidate_versions(
+        reader,
+        &roots,
+        &candidate_indexes,
+        &versions_applied_only_by_records,
+        &mut index_node_cache,
+    );
+    judge_allocation_generations_against_unit_births(
+        reader,
+        &allocation_node_pointers,
+        &mut index_node_cache,
+        &mut judgements,
+    );
     // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
     // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
     // 另一半（> 全部已发布的墓碑记录的对象 ID）今天没有对象：墓碑是打包记录类型 1，这一版一片都不写
@@ -2711,9 +3010,13 @@
         .unwrap_or(0);
     let readable_root_slot_count = u64::try_from(roots.len()).expect("根槽数");
     let walked_root_slot_count = u64::try_from(candidate_indexes.len()).expect("候选根槽数");
+    let walked_versions_applied_only_by_records =
+        u64::try_from(versions_applied_only_by_records.len()).expect("版本数");
+    // 「并进遍历的由记录施加出来的版本」那一段是 2026-09-23 候选 b 加的；判读的一方按字段名取数（`leading_number_after`），
+    // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
     let mechanism = move || {
         format!(
-            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {newest_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
+            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {newest_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
         )
     };
     // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -4,17 +4,25 @@
 
 mod common;
 
-use common::{build_pool, parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS};
+use common::{
+    build_pool, crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters,
+    publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
+};
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
 use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
-use singlefs_core::mount::{mount_rollback, mount_writable, RollbackTarget, ShadowLedger};
+use singlefs_core::mount::{
+    mount_rollback, mount_writable, InstanceRow, RollbackTarget, ShadowLedger,
+};
 use singlefs_core::recovery::PoolReader;
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionUnit};
 use singlefs_format::{NODE_BYTES, TREE_IDENTIFIER_EXTENT};
 use singlefs_harness::crash::MemoryPool;
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RetainedOperation, SharedStream};
 
 const SLOT: u64 = 16384;
 const DEVICES: [u32; 2] = [0, 1];
@@ -846,6 +854,8 @@
         // 诞生代号跟着改成 4：指针与头仍然一致（I-1.2（块头写序已发布） 不跟着红），而这个**被引用**的块按
         // C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判不成已发布（i == 挂载根实例 ⇒ 要 b ≤ 挂载根的 txg）
         // ⇒ I-4.2（无被引用未提交块） 红：盘上留下的正是「提交了的根指着一个还没提交的块」。
+        // 分配记录树里那个落点的分配代也跟着改成 4（两盘各一条）：记录与单元同源，不跟着改就是 I-3.10（已分配记录的分配代等于
+        // 它罩住的单元的诞生代号） 一起红，判别力算不到 I-4.2 头上。
         (
             "I-4.2",
             Box::new(|image: &mut MemoryPool| {
@@ -875,6 +885,18 @@
                     },
                     true,
                 );
+                mutate_unit(
+                    image,
+                    ALLOCATION_TREE,
+                    |bytes| {
+                        assert_eq!(
+                            rewrite_unreleased_allocation_generation(bytes, DATA_UNIT, 3, 4),
+                            2,
+                            "数据单元两盘各一条未释放记录，分配代本来是 3"
+                        );
+                    },
+                    true,
+                );
             }),
         ),
         // 记账里的 inode 号水位从 2 改成 1：树里最大的 key 就是 1，水位不再严格大于它（I-9.6）。
@@ -1821,6 +1843,8 @@
         .chain(known_bad_images_of_overlapping_allocation_records().iter())
         .chain(known_bad_images_of_the_root_ring_health().iter())
         .chain(known_bad_images_of_the_transaction_boundaries().iter())
+        .chain(known_bad_images_of_the_allocation_generation().iter())
+        .chain(known_bad_images_of_a_pure_leak().iter())
         .map(|(invariant, _)| *invariant)
         .collect();
     let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
@@ -2187,3 +2211,638 @@
         );
     }
 }
+
+/// 发布 B 写出的数据单元：它的分配记录在 B 那一版分配记录树（`ALLOCATION_TREE_AFTER_OVERWRITE`）里，未释放、分配代 4，
+/// 单元头里的诞生代号也是 4（落点由 `second_transaction_step_one_overwrite.rs` 的验收钉住）。
+const DATA_UNIT_AFTER_OVERWRITE: u64 = 50182;
+
+/// 分配记录树叶里槽号是 `slot`、**未带**已释放标志、分配代等于 `from` 的记录（两盘各一条）改成 `to`；返回改了几条。
+fn rewrite_unreleased_allocation_generation(
+    bytes: &mut [u8],
+    slot: u64,
+    from: u64,
+    to: u64,
+) -> usize {
+    let count = usize::from(u16::from_le_bytes([
+        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
+        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
+    ]));
+    let mut rewritten = 0;
+    for index in 0..count {
+        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
+        let mut slot_bytes = [0u8; 8];
+        slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
+        let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
+        if u64::from_le_bytes(slot_bytes) == slot
+            && span_field & ALLOCATION_RECORD_RELEASED_FLAG == 0
+            && get_u64(bytes, record + 12) == from
+        {
+            set_u64(bytes, record + 12, to);
+            rewritten += 1;
+        }
+    }
+    rewritten
+}
+
+/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的坏镜像，从发布 B 之后的干净镜像出发：B 那一版分配记录树里
+/// B 的数据单元那两条未释放记录，分配代从 4 改回上一次发布的 txg 3，单元一个字节不动。
+/// 盘上留下的形状就是增补 3 第 1 件代码三方第一轮攻方腿那条变异（复用一个已回收的落点、改写那条分配记录时分配代没改）
+/// 留下的：一条未释放记录的分配代停在上一次的代上，而它罩住的单元是这一次写的。
+/// 释放代、跨度、key 一样没动 ⇒ I-3.9（释放代落在停止引用它的那一格区间里）、I-5.4（分配记录罩住的槽互不相交） 与引用集合都不受影响。
+fn known_bad_images_of_the_allocation_generation() -> Vec<(&'static str, Mutation)> {
+    vec![(
+        "I-3.10",
+        Box::new(|image: &mut MemoryPool| {
+            mutate_unit_of(
+                image,
+                &UNITS_AFTER_OVERWRITE,
+                ALLOCATION_TREE_AFTER_OVERWRITE,
+                |bytes| {
+                    assert_eq!(
+                        rewrite_unreleased_allocation_generation(
+                            bytes,
+                            DATA_UNIT_AFTER_OVERWRITE,
+                            4,
+                            3
+                        ),
+                        2,
+                        "B 的数据单元两盘各一条未释放记录，分配代本来是 4"
+                    );
+                },
+                true,
+            );
+        }),
+    )]
+}
+
+/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）：发布 B 之后的干净镜像上它真被评估过且成立（每条未释放记录的起点槽上
+/// 都读得出可用的单元头），坏镜像上**只**红它、别的判定一条不变。
+#[test]
+fn an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant(
+) {
+    let clean = image_after_the_overwrite("known-bad-allocation-generation");
+    let verdicts_on_the_clean_image = check_pool_image(&clean);
+    assert_eq!(
+        verdict(&clean, "I-3.10"),
+        InvariantVerdict::Holds,
+        "发布 B 之后的干净镜像上 I-3.10 要真被评估过且成立"
+    );
+    for (invariant, mutation) in known_bad_images_of_the_allocation_generation() {
+        let mut image = clean.clone();
+        mutation(&mut image);
+        let verdicts = check_pool_image(&image);
+        assert_eq!(
+            violated_invariants(&verdicts),
+            [invariant],
+            "判红的该只有 {invariant}：{verdicts:?}"
+        );
+        assert_eq!(
+            verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &[invariant]),
+            [],
+            "{invariant} 之外每一条的判定与干净镜像逐项相同"
+        );
+    }
+}
+
+/// 记账树叶里 (统计量, 设备) 那一行的值减 delta（偏移口径同 `adjust_accounting`）。
+fn reduce_accounting(bytes: &mut [u8], statistic: u16, device: u32, delta: u64) {
+    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
+    for index in 0..count {
+        let row = 159 + 34 * index;
+        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
+            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
+        {
+            let value = u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
+            bytes[row + 22..row + 30].copy_from_slice(
+                &value
+                    .checked_sub(delta)
+                    .expect("这一行的值不小于要减的量")
+                    .to_le_bytes(),
+            );
+            return;
+        }
+    }
+    panic!("记账树里没有 ({statistic}, {device}) 这一行");
+}
+
+/// 记账里「defer 队列待释放」那一行的统计量编号（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
+const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
+
+/// 一整槽的可分配空间凭空消失、没有任何单元在用它：记账盘 0 的「已分配」+16384、「空闲」−16384（纯泄漏，分配器为一个没人引用的槽
+/// 同时减了空闲、加了已分配）。「空闲 + 已分配 = 单元区」照旧成立 ⇒ I-5.2（空闲统计对得上） 管不住它，
+/// 记账多于遍历这一侧全仓只有 I-3.1（已分配统计对得上） 一条。`with_inflated_defer_queue` 为真时 defer 队列那一行也抬 16384：
+/// 松弛量由被测那一方自己供给（「记账 − 遍历 ≤ defer 行」那种改法当场绿）。
+fn leak_one_slot_keeping_free_plus_allocated(bytes: &mut [u8], with_inflated_defer_queue: bool) {
+    adjust_accounting(bytes, 1, 0, 16384);
+    reduce_accounting(bytes, 2, 0, 16384);
+    if with_inflated_defer_queue {
+        adjust_accounting(bytes, STATISTIC_DEFER_QUEUE_BYTES, 0, 16384);
+    }
+}
+
+/// 增补 2 收口表第 54 行那一轮三方（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` Z3）攻方腿的
+/// 两份纯泄漏坏镜像，建在写完第一个事务那份干净镜像上：Z3-A（只泄漏）与 Z3-B（泄漏 + defer 行跟着抬高）。
+/// 候选 c（记账 ≥ 遍历）在 Z3-A 上哑、候选 f（记账 − 遍历 ≤ defer 行）在两份上都哑，这一轮的判决就栽在它们上。
+fn known_bad_images_of_a_pure_leak() -> Vec<(&'static str, Mutation)> {
+    vec![
+        (
+            "I-3.1",
+            Box::new(|image: &mut MemoryPool| {
+                mutate_unit(
+                    image,
+                    ACCOUNTING_ROOT,
+                    |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, false),
+                    true,
+                );
+            }),
+        ),
+        (
+            "I-3.1",
+            Box::new(|image: &mut MemoryPool| {
+                mutate_unit(
+                    image,
+                    ACCOUNTING_ROOT,
+                    |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, true),
+                    true,
+                );
+            }),
+        ),
+    ]
+}
+
+/// 候选 b（回退候选集补上「由记录施加出来、根槽从没落盘的那一版」，2026-09-23 用户定）不放宽 I-3.1 的等式、只补等式右边：
+/// 纯泄漏的两份坏镜像照样**只**红 I-3.1，别的判定与干净镜像逐项相同（Z3-A 让 I-5.2 照旧绿，是这份镜像要的样子）。
+#[test]
+fn a_pure_leak_that_keeps_free_plus_allocated_equal_to_the_unit_area_reddens_only_the_allocated_statistic_invariant(
+) {
+    let clean = build_pool("known-bad-pure-leak").memory_pool();
+    let verdicts_on_the_clean_image = check_pool_image(&clean);
+    for (invariant, mutation) in known_bad_images_of_a_pure_leak() {
+        let mut image = clean.clone();
+        mutation(&mut image);
+        let verdicts = check_pool_image(&image);
+        assert_eq!(
+            violated_invariants(&verdicts),
+            [invariant],
+            "判红的该只有 {invariant}：{verdicts:?}"
+        );
+        assert_eq!(
+            verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &[invariant]),
+            [],
+            "{invariant} 之外每一条的判定与干净镜像逐项相同"
+        );
+    }
+}
+
+/// 残留记录那一版的内容：与 A、B 都不同长、不同字节（具体哪些字节不承重）。
+fn content_applied_only_by_its_journal_record() -> Vec<u8> {
+    (0..3700usize)
+        .map(|index| u8::try_from((index * 7 + 5) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后的镜像（增补 2 收口表第 54 行那段历史，
+/// 与 `second_transaction_step_zero_layer0.rs` 残留记录那条流里实例 2 的根已落盘的那 12 个状态同形）：
+/// mkfs → 取号 → 暖机 → A → B，实例 1 再发一版（txg 5）而**只落了它的单元与 journal 记录、根槽与系统配置槽一个都没落**；
+/// 可写挂载择 B 的根 (1, 4)、施加那条记录、给实例 1 写行 (1, 5, 3)、写行发布 txg 6 把 (1, 5) 那一版的固定点单元换下放进 defer、
+/// 暖机一次 txg 7。交回挂载前后两份镜像、挂载之后接着发布要用的盘与分配器，以及几个单元的槽（两盘同槽）。
+struct ImageAfterAVersionAppliedOnlyByItsJournalRecord {
+    /// 挂载之前：那一版的单元与记录已在盘上，最新的根还是 B 的 (1, 4)，实例表里没有实例 1 的行。
+    image_before_the_mount: MemoryPool,
+    /// 挂载之后（最新的根是暖机 txg 7）。
+    image: MemoryPool,
+    devices: Vec<(
+        DeviceIdentity,
+        singlefs_harness::RecordingBlockDevice<singlefs_harness::crash::SparseBlockDevice>,
+    )>,
+    allocator: singlefs_core::allocator::PoolAllocator,
+    newest: singlefs_core::transaction::TransactionOutput,
+    newest_accounting_root_slot: u64,
+    /// 只由记录施加出来的那一版 (1, 5) 自己的树表单元：写行发布 txg 6 把它换下放进 defer，根环里没有一条根引用它。
+    tree_table_slot_of_the_version_applied_only_by_its_record: u64,
+}
+
+fn image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+    tag: &str,
+) -> ImageAfterAVersionAppliedOnlyByItsJournalRecord {
+    let mut pool = build_pool(tag);
+    let first = pool.output.clone();
+    let second_content: Vec<u8> = (0..4100usize)
+        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
+        .collect();
+    let second = publish_overwrite_in_process(
+        &mut pool,
+        &first,
+        &second_content,
+        FIXED_WRITE_TIME_SECONDS + 60,
+        InstanceGeneration(1),
+    )
+    .expect("覆盖写 B");
+    let operations_through_the_second_publish = pool.retained_operations().len();
+    let applied_only_by_its_record = publish_overwrite_in_process(
+        &mut pool,
+        &second,
+        &content_applied_only_by_its_journal_record(),
+        FIXED_WRITE_TIME_SECONDS + 120,
+        InstanceGeneration(1),
+    )
+    .expect("实例 1 再发一版");
+    assert_eq!(
+        (
+            applied_only_by_its_record.record.instance,
+            applied_only_by_its_record.record.checkpoint_txg,
+            applied_only_by_its_record.record.transaction
+        ),
+        (InstanceGeneration(1), CheckpointTxg(5), 3),
+        "只由记录施加出来的那一版：实例 1、txg 5、事务号 3"
+    );
+    let operations = pool.retained_operations();
+    let classifier = geometry();
+    let units_and_record_of_the_version: Vec<RetainedOperation> = operations
+        [operations_through_the_second_publish..]
+        .iter()
+        .filter(|retained| match classifier.classify(&retained.operation) {
+            StepKind::UnitWrite | StepKind::JournalRecord => true,
+            StepKind::ZeroFill
+            | StepKind::RootRecordFua
+            | StepKind::SystemConfigurationSlot
+            | StepKind::Barrier => false,
+        })
+        .cloned()
+        .collect();
+    let mut image_before_the_mount = pool.memory_pool_after_mkfs();
+    image_before_the_mount
+        .apply(&operations[pool.mkfs_operation_count..operations_through_the_second_publish]);
+    image_before_the_mount.apply(&units_and_record_of_the_version);
+    let stream = SharedStream::retaining_contents();
+    let mut devices = crash_state_devices(&image_before_the_mount, &[], &[], &stream);
+    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    assert_eq!(
+        (
+            mounted.output.chosen_root.instance,
+            mounted.output.chosen_root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.journal.prefix_applied
+        ),
+        (InstanceGeneration(1), CheckpointTxg(4), CheckpointTxg(5), 1),
+        "择 B 的根 (1, 4)，施加那条记录，现行版是 (1, 5)"
+    );
+    assert_eq!(
+        mounted.output.rows_written,
+        vec![InstanceRow {
+            instance: InstanceGeneration(1),
+            selected_root_txg: CheckpointTxg(5),
+            applied_transaction_high_water: 3,
+            is_rollback: false,
+        }],
+        "写行 (1, 5, 3)"
+    );
+    assert_eq!(
+        mounted.output.row_publish.root().checkpoint_txg,
+        CheckpointTxg(6),
+        "写行发布 txg 6"
+    );
+    let newest = mounted
+        .current
+        .into_file_version()
+        .expect("施加了那条记录，现行那一版带文件");
+    ImageAfterAVersionAppliedOnlyByItsJournalRecord {
+        image_before_the_mount,
+        image: memory_pool_of_sparse_devices(&devices),
+        devices,
+        allocator: mounted.allocator,
+        newest_accounting_root_slot: newest.unit(TransactionUnit::AccountingTree).slot.0,
+        newest,
+        tree_table_slot_of_the_version_applied_only_by_its_record: applied_only_by_its_record
+            .unit(TransactionUnit::TreeTable)
+            .slot
+            .0,
+    }
+}
+
+/// 单元区起点（第一个单元落在这个槽上：mkfs 种的实例表）。
+const UNIT_AREA_FIRST_SLOT: u64 = 50176;
+/// `units_found_on_device_zero` 从单元区起点往后扫多少个槽：这几份镜像上用到的落点都在前几百个槽里。
+const SLOTS_SCANNED_FOR_UNITS: u64 = 1024;
+
+/// 单元区里盘 0 上此刻的单元：(起点槽, 字节数)，按头里的类标签认（码 2 一槽，码 1 / 码 3 两槽）。
+/// 镜像是现造出来的、没有写死的单元表时，拿它给沿引用链补校验和（`mutate_unit_of`）当单元表：
+/// 父单元（树表、中央映射树的根）里引用被改单元的位置条目都要补上，漏一个就是 I-2.1 跟着红。
+fn units_found_on_device_zero(image: &MemoryPool) -> Vec<(u64, usize)> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let mut units = Vec::new();
+    let mut slot = UNIT_AREA_FIRST_SLOT;
+    while slot < UNIT_AREA_FIRST_SLOT + SLOTS_SCANNED_FOR_UNITS {
+        let header = read(image, 0, slot * SLOT, 512);
+        let slots_taken = if &header[..4] == b"SFSU" {
+            match header[6] {
+                2 => {
+                    units.push((slot, node_bytes));
+                    1
+                }
+                1 | 3 => {
+                    units.push((slot, 2 * node_bytes));
+                    2
+                }
+                _unregistered_unit_class => 1,
+            }
+        } else {
+            1
+        };
+        slot += slots_taken;
+    }
+    units
+}
+
+/// 增补 2 收口表第 54 行（2026-09-23 用户定候选 b）的两半，建在同一份镜像上：
+/// ① **误报消失**：可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后，那一版被换下的四个固定点单元在 defer 里、仍算已分配，
+///    根环里没有一条根引用它们；checker 把那一版并进遍历之后 I-3.1（已分配统计对得上） 真被评估过且成立，别的也一条不红。
+///    候选集不补这一版（`versions_applied_only_by_records` 交回空），这一段就在 I-3.1 上红、差 65 536 字节（每盘 4 槽）。
+/// ② **改完仍会红**：同一份镜像上记账盘 0 再泄漏一槽（「已分配」+16384、「空闲」−16384），I-3.1 照样红、只红它，
+///    说明文字里的机理标识报出那一版确实并进了遍历——补上的只是等式右边缺的那一版，没把泄漏一起吞掉。
+#[test]
+fn a_version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_a_leak_on_top_still_reddens_it(
+) {
+    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+        "known-good-version-applied-only-by-its-record",
+    );
+    let verdicts_on_the_clean_image = check_pool_image(&built.image);
+    assert_eq!(
+        violated_invariants(&verdicts_on_the_clean_image),
+        Vec::<&str>::new(),
+        "接在只由记录施加出来的那一版之后的合法镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
+    );
+    assert_eq!(
+        verdict(&built.image, "I-3.1"),
+        InvariantVerdict::Holds,
+        "I-3.1 要真被评估过且成立"
+    );
+    let mut leaked = built.image.clone();
+    mutate_unit_of(
+        &mut leaked,
+        &units_found_on_device_zero(&built.image),
+        built.newest_accounting_root_slot,
+        |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, false),
+        true,
+    );
+    let verdicts = check_pool_image(&leaked);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-3.1"],
+        "再泄漏一槽只该红 I-3.1：{verdicts:?}"
+    );
+    assert_eq!(
+        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-3.1"]),
+        [],
+        "I-3.1 之外每一条的判定与泄漏之前逐项相同"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&leaked, "I-3.1") else {
+        panic!("上面判过 I-3.1 红");
+    };
+    assert!(
+        detail.contains("并进遍历的由记录施加出来的版本 1 个"),
+        "机理标识要报出那一版并进了遍历：{detail}"
+    );
+}
+
+/// 候选 b 那一版的第 ① 条（「它被施加过」要看实例表里有没有这个实例的行）：同一段历史**挂载之前**那一刻，
+/// 那一版的单元与记录已在盘上、最新的根还是 B 的 (1, 4)，实例表里没有实例 1 的行——施加要等下一次挂载，这一刻它不算一版。
+/// 这一刻它的单元是孤儿、不在记账里；把它并进遍历就是遍历多算，I-3.1（已分配统计对得上） 红
+/// （`second_transaction_step_zero_layer0.rs` 残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格）。
+#[test]
+fn a_journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds() {
+    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+        "known-good-record-not-applied-yet",
+    );
+    let verdicts = check_pool_image(&built.image_before_the_mount);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "挂载之前那一刻一条都不许红：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&built.image_before_the_mount, "I-3.1"),
+        InvariantVerdict::Holds,
+        "I-3.1 要真被评估过且成立"
+    );
+}
+
+/// 只由记录施加出来的那一版在候选集里：它引用的单元照候选根一样判，I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）
+/// 按这一版各判一格。把它自己的树表单元（只有它引用，在 defer 里）两份都抹成 0：从它出发的遍历读不到那个单元，
+/// 这两条必须红——根环里的根一条都不引用它，按根判的那几格判不出来。
+#[test]
+fn erasing_a_unit_only_the_version_applied_by_its_journal_record_references_reddens_the_reuse_invariants(
+) {
+    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+        "known-bad-version-applied-only-by-its-record-erased",
+    );
+    let mut image = built.image.clone();
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    for device in DEVICES {
+        write(
+            &mut image,
+            device,
+            built.tree_table_slot_of_the_version_applied_only_by_its_record * SLOT,
+            &vec![0u8; node_bytes],
+        );
+    }
+    let verdicts = check_pool_image(&image);
+    let violated = violated_invariants(&verdicts);
+    for invariant in ["I-7.4", "I-4.8"] {
+        assert!(
+            violated.contains(&invariant),
+            "那一版的树表被抹掉，{invariant} 要红：{verdicts:?}"
+        );
+    }
+    assert!(
+        !violated.contains(&"I-7.2"),
+        "最新的根不引用那个单元，I-7.2（最新根能完整走完） 不该跟着红：{verdicts:?}"
+    );
+}
+
+/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 射程 ③：记录罩住的起点槽上读不出可用的单元头时这一格没有对象。
+/// 发布 B 之后的干净镜像上把 B 的数据单元两份头里的诞生代号改成 99、**不重封**头校验和（盘上坏了的样子）：
+/// 那条记录不判，I-3.10 照旧由别的记录判成成立；读不出本身由 I-2.1 / I-2.4 说话。
+/// 把「头校验和过」那一道从「可用」里拿掉，这里就按坏头里的 99 判红。
+#[test]
+fn an_allocation_record_whose_start_slot_header_does_not_verify_is_not_judged_by_the_allocation_generation_invariant(
+) {
+    let mut image = image_after_the_overwrite("known-good-allocation-generation-unusable-header");
+    let data_unit_bytes = 2 * usize::try_from(NODE_BYTES).expect("16384");
+    for device in DEVICES {
+        let mut unit = read(
+            &image,
+            device,
+            DATA_UNIT_AFTER_OVERWRITE * SLOT,
+            data_unit_bytes,
+        );
+        assert_eq!(
+            get_u64(&unit, DATA_UNIT_BIRTH_TXG_OFFSET),
+            4,
+            "B 的数据单元诞生代号本来是 4"
+        );
+        set_u64(&mut unit, DATA_UNIT_BIRTH_TXG_OFFSET, 99);
+        write(&mut image, device, DATA_UNIT_AFTER_OVERWRITE * SLOT, &unit);
+    }
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        verdict(&image, "I-3.10"),
+        InvariantVerdict::Holds,
+        "起点槽的头校验和不过的那条记录不判，别的未释放记录照判且成立：{verdicts:?}"
+    );
+    assert!(
+        violated_invariants(&verdicts).contains(&"I-2.1"),
+        "单元坏了由 I-2.1 说话：{verdicts:?}"
+    );
+}
+
+/// 抬 F 之前实例 2 接着发几版：发到 txg 10，「第 4 新的非空持久有效根」才是 txg 6，F 抬得到 6（上限的算法见
+/// `crates/singlefs-core/src/mount.rs` 的 `raise_rollback_floor`）。
+const OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION: u64 = 3;
+
+/// 实例 2 在同一次挂载里接着覆盖写 `rounds` 次，交回最后那一版。
+fn overwrite_in_the_second_instance(
+    built: &mut ImageAfterAVersionAppliedOnlyByItsJournalRecord,
+    rounds: u64,
+    write_time_offset_seconds: u64,
+) -> singlefs_core::transaction::TransactionOutput {
+    let publish_parameters = parameters();
+    let mut previous = built.newest.clone();
+    for round in 0..rounds {
+        let round_index = usize::try_from(round).expect("轮次");
+        let content: Vec<u8> = (0..(2000 + round_index * 37))
+            .map(|index| u8::try_from((index * 11 + round_index) % 251).expect("小于 256"))
+            .collect();
+        let mut writer = PoolWriter::new(&publish_parameters, built.devices.as_mut_slice());
+        previous = publish_overwrite(
+            &mut writer,
+            &mut built.allocator,
+            &previous,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset_seconds + round,
+            },
+            InstanceGeneration(2),
+        )
+        .expect("实例 2 接着覆盖写");
+    }
+    previous
+}
+
+/// 候选 b 那一版的第 ③ 条（「不低于回退下界」）：F 抬过 (1, 5) 之后它就不再并进遍历。
+/// 同一段历史在实例 2 里接着发三版（txg 8–10），再把 F 抬到 6：抬 F 生效之后回收释放代 ≤ 6 的落点，
+/// (1, 5) 那一版被写行发布 txg 6 换下的单元（释放代 6）就在其中——不再算已分配。把它照旧并进遍历就是遍历多算，I-3.1 红。
+/// 环里比它老的有效根还在（txg 1–4），第 ④ 条拦不住这一格，拦它的只有第 ③ 条。
+#[test]
+fn a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it(
+) {
+    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+        "known-good-version-applied-only-by-its-record-below-the-floor",
+    );
+    let mut current = overwrite_in_the_second_instance(
+        &mut built,
+        OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION,
+        180,
+    );
+    assert_eq!(
+        current.root.checkpoint_txg,
+        CheckpointTxg(10),
+        "实例 2 发到 txg 10"
+    );
+    let raised = singlefs_core::mount::raise_rollback_floor(
+        &parameters(),
+        &mut built.devices,
+        &mut built.allocator,
+        &mut current,
+        CheckpointTxg(6),
+        ShadowLedger::On,
+    )
+    .expect("抬 F 到 6");
+    assert!(
+        raised.reclaimed.iter().any(|placement| placement.slot.0
+            == built.tree_table_slot_of_the_version_applied_only_by_its_record),
+        "(1, 5) 那一版的树表单元（释放代 6）在抬 F 生效之后被回收：{:?}",
+        raised.reclaimed
+    );
+    let image = memory_pool_of_sparse_devices(&built.devices);
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "F 抬过 (1, 5) 之后一条都不许红：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, "I-3.1"),
+        InvariantVerdict::Holds,
+        "I-3.1 要真被评估过且成立"
+    );
+}
+
+/// 环转过 (1, 5) 那一版之前实例 2 在同一次挂载里接着发几版：从 txg 8 起到 txg 29 为止，环里每个槽都被 txg ≥ 6 的根盖过
+/// （根环 24 槽：三个区域各 8 槽，槽位按 txg 轮换）。
+const OVERWRITES_TURNING_THE_ROOT_RING_PAST_THE_VERSION: u64 = 22;
+
+/// 候选 b 那一版的第 ④ 条（「它换下的单元还没到回收的时候」）：环转过去之后它就不再并进遍历。
+/// 同一段历史在实例 2 里接着发 22 版（txg 8–29），环里最旧的有效根变成比 (1, 5) 新；再重开可写挂载（实例 3），
+/// 重建分配器按「释放代 ≤ max(F_生效, 环里最旧有效根)」（D3（空间分配） 已定项 7）把那一版换下的单元（释放代 6）收回去，
+/// 再发一版。这时那一版的单元不再算已分配、可能已被复用；把它照旧并进遍历就是拿已经不归它的槽去比。
+///
+/// ⚠️ **这一份镜像上 I-3.1（已分配统计对得上） 本来就红**，与 (1, 5) 那一版无关：环转过一整圈之后记账多于遍历
+/// （记账多 24 槽，机理标识里「最老的自证过的根 txg」已高过 F），是增补 2 收口表第 ② 行那一族、随机历史生成器已知红第 0 条
+/// （跨几次挂载转过一整圈同样红）。这里钉的是现状，不是认下来的行为：I-3.1 只红在那一形上（记账多于遍历），
+/// 机理标识报出那一版**没有**并进遍历，别的每一条都不红。把第 ④ 条拿掉，那一版照旧并进遍历，机理标识里的版本数变成 1。
+#[test]
+fn a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring_has_turned_past_it_and_a_mount_reclaimed_its_units(
+) {
+    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+        "known-good-version-applied-only-by-its-record-after-the-ring-turned",
+    );
+    let publish_parameters = parameters();
+    let previous = overwrite_in_the_second_instance(
+        &mut built,
+        OVERWRITES_TURNING_THE_ROOT_RING_PAST_THE_VERSION,
+        180,
+    );
+    assert_eq!(
+        previous.root.checkpoint_txg,
+        CheckpointTxg(29),
+        "实例 2 发到 txg 29，环里每个槽都被 txg ≥ 6 的根盖过"
+    );
+    let remounted = mount_writable(&publish_parameters, &mut built.devices).expect("重开可写挂载");
+    let mut allocator = remounted.allocator;
+    let remounted_current = remounted
+        .current
+        .into_file_version()
+        .expect("重开之后现行那一版带文件");
+    let mut writer = PoolWriter::new(&publish_parameters, built.devices.as_mut_slice());
+    publish_overwrite(
+        &mut writer,
+        &mut allocator,
+        &remounted_current,
+        FirstFile {
+            content: &content_applied_only_by_its_journal_record(),
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
+        },
+        remounted.output.instance,
+    )
+    .expect("重开之后再发一版");
+    let image = memory_pool_of_sparse_devices(&built.devices);
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-3.1"],
+        "环转过一整圈之后只有 I-3.1 那一形已知红：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.1") else {
+        panic!("上面判过 I-3.1 红");
+    };
+    assert!(
+        singlefs_harness::history::allocation_statistic_mechanism(&detail)
+            .is_some_and(|mechanism| mechanism.oldest_readable_root_txg > mechanism.rollback_floor),
+        "I-3.1 红在环转过一整圈那一形上（最老的自证过的根已高过 F）：{detail}"
+    );
+    assert!(
+        detail.contains("并进遍历的由记录施加出来的版本 0 个"),
+        "环转过 (1, 5) 之后那一版不再并进遍历：{detail}"
+    );
+}
--- a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
@@ -74,8 +74,9 @@
     )
 }
 
-/// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树与分配记录树，那 13 条只在这些状态上评估
-/// （I-5.4（分配记录罩住的槽互不相交） 在其中：种子根与暖机根指着的第 0 版树表是空的，没有分配记录树；
+/// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树与分配记录树，那 14 条只在这些状态上评估
+/// （I-5.4（分配记录罩住的槽互不相交） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 在其中：
+/// 种子根与暖机根指着的第 0 版树表是空的，没有分配记录树；
 /// I-1.10（码 2 条目宽等于字段表宽） 同一个理由：第 0 版树表一条条目都没有，没有哪个码 2 节点的条目宽可比）；
 /// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到；
 /// I-8.6（反向链算法） 要环里至少有一条自证过的记录，环还空着的那几个状态上报「不适用」；
@@ -97,8 +98,8 @@
         "记录核对器两条判据在已定的持久顺序下恒 0"
     );
     let only_under_the_new_root = [
-        "I-1.10", "I-3.1", "I-3.9", "I-5.2", "I-5.4", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7",
-        "I-9.10", "I-9.12", "I-9.13",
+        "I-1.10", "I-3.1", "I-3.9", "I-3.10", "I-5.2", "I-5.4", "I-9.1", "I-9.2", "I-9.4", "I-9.6",
+        "I-9.7", "I-9.10", "I-9.12", "I-9.13",
     ];
     // I-9.14 要同一棵树的条目出现在两个树表单元里才比得出来，而这条流上只有第一个事务写树表：mkfs 种的第 0 版树表是空的、
     // 两次暖机空发布不写树表 ⇒ 每个状态都报「不适用」，一个状态都评估不到。跨根比得出来的流在
--- a/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
@@ -380,21 +380,22 @@
 /// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 15 条；
 /// 其余 21 条要真被评估过且成立。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
 /// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
-/// I-5.4（分配记录罩住的槽互不相交） 同一个理由：第 0 版树表里没有分配记录树，候选集里一条根的记录都走不到。
+/// I-5.4（分配记录罩住的槽互不相交） 同一个理由：第 0 版树表里没有分配记录树，候选集里一条根的记录都走不到；
+/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 也是这个理由。
 /// I-1.10（码 2 条目宽等于字段表宽） 也是这个理由：一条树表条目都没有，就没有哪个码 2 节点的条目宽可比。
 /// I-8.7（实例内事务号不重号） 在这里报不适用是因为环里一条非 0 事务号的记录都没有：写行与暖机的空发布都写 0
 /// （D23（journal 的角色与格式） 已定项 19 ①）。I-8.8（前缀里的事务不被切开） 是因为没有哪个非 0 事务号落了两条记录、
 /// 也没有哪一组一条提交标记都没有。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 16] = [
-    "I-1.10", "I-3.1", "I-3.9", "I-5.2", "I-5.4", "I-8.7", "I-8.8", "I-9.1", "I-9.2", "I-9.4",
-    "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14",
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 17] = [
+    "I-1.10", "I-3.1", "I-3.9", "I-3.10", "I-5.2", "I-5.4", "I-8.7", "I-8.8", "I-9.1", "I-9.2",
+    "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14",
 ];
 
 /// mkfs 刚写完那一份要多报一条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法） 没有判的对象。
 /// 可写挂载之后环里有记录（写行 / 暖机的空发布），它就真被评估过了——所以这一条只在 mkfs 之后那一格。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 17] = [
-    "I-1.10", "I-3.1", "I-3.9", "I-5.2", "I-5.4", "I-8.6", "I-8.7", "I-8.8", "I-9.1", "I-9.2",
-    "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14",
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 18] = [
+    "I-1.10", "I-3.1", "I-3.9", "I-3.10", "I-5.2", "I-5.4", "I-8.6", "I-8.7", "I-8.8", "I-9.1",
+    "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14",
 ];
 
 /// 写完第一个文件版本之后仍报不适用的那三条：
@@ -407,6 +408,15 @@
 /// B 之后再接一事务两条记录的阳性对照与坏镜像）。
 const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 3] = ["I-8.7", "I-8.8", "I-9.14"];
 
+/// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）
+/// 一条。写行那次发布写了一片分配记录节点、根指针住根记录（C512（树表 0 条的一版上被换下的单元记在哪），D16（发布语义） 已定项 9），
+/// 里面未释放的那几条记录（被换下的 mkfs 那片实例表已带释放标志，归 I-3.9）起点槽上都读得出单元头，I-3.10 在这一格真被评估过。
+/// I-5.4（分配记录罩住的槽互不相交） 仍报不适用：它只从树表条目找分配记录树，不读根记录直接持有的那一片。
+const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 16] = [
+    "I-1.10", "I-3.1", "I-3.9", "I-5.2", "I-5.4", "I-8.7", "I-8.8", "I-9.1", "I-9.2", "I-9.4",
+    "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14",
+];
+
 /// 这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
 fn assert_checker_verdicts(pool: &FormattedPool, step: &str, not_applicable: &[&str]) {
     let verdicts = check_pool_image(&pool.memory_pool());
@@ -937,7 +947,11 @@
             "mkfs 的实例表 2 槽 + 树表 1 槽 + 新写的实例表 2 槽 + 这一版自己那片分配记录树 1 槽（C512）；换下的 2 槽在 defer 队列里"
         );
     }
-    assert_checker_verdicts(&formatted, "写行那次发布之后", &NOT_APPLICABLE_WITHOUT_FILE);
+    assert_checker_verdicts(
+        &formatted,
+        "写行那次发布之后",
+        &NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE,
+    );
 
     // 三、写文件：接在写行之后那一版后面。
     let mut allocator = second_mount.allocator;
--- a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
@@ -384,7 +384,7 @@
     // 一个状态都不漏记。
     for must_evaluate in [
         "I-3.1", "I-5.2", "I-5.1", "I-7.2", "I-2.1", "I-3.8", "I-7.4", "I-4.8", "I-3.9", "I-9.14",
-        "I-5.4", "I-8.7",
+        "I-5.4", "I-8.7", "I-3.10",
     ] {
         assert!(
             tally
@@ -1026,11 +1026,12 @@
         tally.first_ignored_violation
     );
     assert_eq!(tally.failed_states, 0);
-    // I-3.1（已分配统计对得上）在实例 2 的根为最新的 12 个状态上判红（口径未定，2026-09-17 写这条用例时发现）：记账的已分配逐盘比遍历候选根多 65536 字节，
-    // 正是残留记录那一版（(1, 5)，根槽从没落盘、只由记录施加出来）自己的四个固定点单元 50261–50264——写行发布 txg 6 把它们释放进 defer，
-    // 环里没有一条根引用它们。checker 的「已分配 = 候选根引用的并集」与分配器「defer 里的仍算已分配」在「由记录施加出来的那一版」上分歧，
-    // 改哪一边是 I-3.1 口径的设计问题；这里钉的是现状，不是认下来的行为。
-    assert_checker_and_record_checker_counts(&tally, &[("I-3.1", 12)]);
+    // 实例 2 的根为最新的那 12 个状态：残留记录那一版（(1, 5)，根槽从没落盘、只由记录施加出来）自己的四个固定点单元
+    // 50261–50264 被写行发布 txg 6 释放进 defer、仍算已分配，环里没有一条根引用它们。2026-09-17 写这条用例时它们在
+    // I-3.1（已分配统计对得上） 上判红、差 65536 字节；2026-09-23 用户定候选 b（增补 2 收口表第 54 行）：checker 的候选集
+    // 补上「由记录施加出来、根槽从没落盘的那一版」（`singlefs_checker::walk` 的 `versions_applied_only_by_records`），
+    // 那 12 个状态从此一条都不红。把那一版从候选集里拿掉，这一行在 I-3.1 上当场红回 12。
+    assert_checker_and_record_checker_counts(&tally, &[]);
 }
 
 /// 改坏 tail 的 tail 值：窗口 [3, 18] 里有 A 那条记录（jsn 3），它点名的 50180 在 txg 18 被合法复用。
```

## 四、checker 那一组新增的变异名（`/tmp/claude-1000/impl-checker-batch/mutations-append.tsv`，18 行，原样，还没合并进 `crates/mutations.tsv`）

sha256sum：`5be561c8d819ebe55c3f754110078659161ce3a71c0f6b8e4f14b596b667f78f`（与实现员报告第九节一致，本节内容原样拷自该文件）。

```tsv
增补 2 收口第 44 行：I-3.10 的比较恒成立（未释放记录的分配代与单元头里的诞生代号不比）	crates/singlefs-checker/src/walk.rs	            judgements.judge("I-3.10", record.generation == birth_txg, || {	            judgements.judge("I-3.10", true || record.generation == birth_txg, || {	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
增补 2 收口第 44 行：I-3.10 射程 ① 放宽（已释放的记录也拿释放代去比单元头的诞生代号）	crates/singlefs-checker/src/walk.rs	            if record.is_released\n                || !examined_records.insert(	            if false\n                || !examined_records.insert(	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
增补 2 收口第 44 行：I-3.10 射程 ② 改读末槽（跨两槽的记录不再取起点槽那个单元头）	crates/singlefs-checker/src/walk.rs	                    record.slot * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	                    (record.slot + record.span_slots - 1) * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
增补 2 收口第 44 行：I-3.10 射程 ③ 放宽（起点槽的头校验和不过也照样取诞生代号来比）	crates/singlefs-checker/src/walk.rs	    checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET)\n        .then(|| read_u64(header, birth_txg_offset))	    (true || checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET))\n        .then(|| read_u64(header, birth_txg_offset))	-p singlefs-harness --test checker_known_bad_images -- an_allocation_record_whose_start_slot_header	an_allocation_record_whose_start_slot_header_does_not_verify_is_not_judged_by_the_allocation_generation_invariant
增补 2 收口第 54 行（候选 b）：由记录施加出来、根槽从没落盘的那一版不并进遍历（合法镜像上 I-3.1 红回来）	crates/singlefs-checker/src/walk.rs	    for version in &versions_applied_only_by_records {	    for version in versions_applied_only_by_records.iter().take(0) {	-p singlefs-harness --test checker_known_bad_images -- a_version_applied_only_by_its_journal_record_is_walked	a_version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_a_leak_on_top_still_reddens_it
增补 2 收口第 54 行（候选 b）：由记录施加出来的那一版不并进遍历（层 0 残留记录那条流 12 个状态在 I-3.1 上红回来）	crates/singlefs-checker/src/walk.rs	    for version in &versions_applied_only_by_records {	    for version in versions_applied_only_by_records.iter().take(0) {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
增补 2 收口第 54 行（候选 b）第 ① 条放宽：不看实例表里有没有这个实例的行，环里带提交标记的记录都算施加过	crates/singlefs-checker/src/walk.rs	            let applied_by_a_recovery = record.commit_marker == CommitMarker::Present\n                && instance_table_rows.iter().any(|row| {	            let applied_by_a_recovery = record.commit_marker == CommitMarker::Present\n                || instance_table_rows.iter().any(|row| {	-p singlefs-harness --test checker_known_bad_images -- a_journal_record_that_no_mount_has_applied_yet	a_journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds
增补 2 收口第 54 行（候选 b）第 ③ 条去掉：低于回退下界 F 的那一版也并进遍历	crates/singlefs-checker/src/walk.rs	            let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;	            let at_or_above_the_floor = true || record.checkpoint_txg >= rollback_floor;	-p singlefs-harness --test checker_known_bad_images -- a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor	a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it
增补 2 收口第 54 行（候选 b）第 ④ 条去掉：环里最旧的有效根已比那一版新（它换下的单元已可回收）也并进遍历	crates/singlefs-checker/src/walk.rs	            let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;	            let its_units_are_not_reclaimable_yet = true || oldest_valid_root_txg < record.checkpoint_txg;	-p singlefs-harness --test checker_known_bad_images -- a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring	a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring_has_turned_past_it_and_a_mount_reclaimed_its_units
增补 2 收口第 54 行（候选 b）：由记录施加出来的那一版引用的单元被复用或抹头时 I-7.4 不判（按版本那一格恒成立）	crates/singlefs-checker/src/walk.rs	            .judge("I-7.4", !walked_into_reused_or_erased_unit, || {\n                format!(\n                    "由记录施加出来的那一版	            .judge("I-7.4", true, || {\n                format!(\n                    "由记录施加出来的那一版	-p singlefs-harness --test checker_known_bad_images -- erasing_a_unit_only_the_version	erasing_a_unit_only_the_version_applied_by_its_journal_record_references_reddens_the_reuse_invariants
增补 2 收口第 54 行（候选 b）：从由记录施加出来的那一版出发的遍历读不出单元时 I-4.8 不判（按版本那一格恒成立）	crates/singlefs-checker/src/walk.rs	            .judge("I-4.8", !walked_into_reused_or_erased_unit, || {\n                format!(\n                    "由记录施加出来的那一版	            .judge("I-4.8", true, || {\n                format!(\n                    "由记录施加出来的那一版	-p singlefs-harness --test checker_known_bad_images -- erasing_a_unit_only_the_version	erasing_a_unit_only_the_version_applied_by_its_journal_record_references_reddens_the_reuse_invariants
增补 2 收口第 54 行判决 Z3 的候选 c（记账 ≥ 遍历）：纯泄漏坏镜像必须照样红	crates/singlefs-checker/src/walk.rs	            judgements.judge("I-3.1", allocated == Some(walked), || {	            judgements.judge("I-3.1", allocated.is_some_and(|allocated| allocated >= walked), || {	-p singlefs-harness --test checker_known_bad_images -- a_pure_leak	a_pure_leak_that_keeps_free_plus_allocated_equal_to_the_unit_area_reddens_only_the_allocated_statistic_invariant
增补 2 收口第 54 行判决 Z3 的候选 f（记账 − 遍历 ≤ defer 行）：纯泄漏坏镜像必须照样红	crates/singlefs-checker/src/walk.rs	            judgements.judge("I-3.1", allocated == Some(walked), || {	            judgements.judge("I-3.1", allocated.is_some_and(|allocated| allocated >= walked && allocated - walked <= accounting.get(&(5u16, *device)).copied().unwrap_or(0)), || {	-p singlefs-harness --test checker_known_bad_images -- a_pure_leak	a_pure_leak_that_keeps_free_plus_allocated_equal_to_the_unit_area_reddens_only_the_allocated_statistic_invariant
增补 2 收口第 44 行：I-3.10 不读树表 0 条那一版根记录直接持有的那片分配记录树（C512（树表 0 条的一版上被换下的单元记在哪））	crates/singlefs-checker/src/walk.rs	        allocation_node_pointers.push(parse_node_pointer(&record[342..428]));	        if false { allocation_node_pointers.push(parse_node_pointer(&record[342..428])); }	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
增补 2 收口第 44 行：I-3.10 取码 1 单元头的诞生代号偏移错位（75 读成 83）	crates/singlefs-checker/src/walk.rs	        1 => (105, 75),	        1 => (105, 83),	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
增补 2 收口第 44 行：I-3.10 取码 2 单元头的诞生代号偏移错位（52 + 2k 读成 60 + 2k）	crates/singlefs-checker/src/walk.rs	            (86 + key_span, 52 + key_span)	            (86 + key_span, 60 + key_span)	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
增补 2 收口第 44 行：I-3.10 取码 3 单元头的诞生代号偏移错位（73 读成 81）	crates/singlefs-checker/src/walk.rs	        3 => (107, 73),	        3 => (107, 81),	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
增补 2 收口第 54 行（候选 b）第 ② 条去掉：根槽读得出的那一版也从它的记录再并进一次遍历	crates/singlefs-checker/src/walk.rs	            let root_slot_is_readable = roots.iter().any(|(_, _, root)| {	            let root_slot_is_readable = false && roots.iter().any(|(_, _, root)| {	-p singlefs-harness --test checker_known_bad_images -- a_version_applied_only_by_its_journal_record_is_walked	a_version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_a_leak_on_top_still_reddens_it
```

## 五、watermark 那一组：`/tmp/claude-1000/impl-watermark/watermark.patch`（原样拷 `research/prompts/m2-wave3-code-r1-patches/watermark.patch`）

基准：主工作区 2026-09-23 23:26:40 UTC 的 `crates/` 快照（`/tmp/claude-1000/impl-watermark/base-crates-3/`，HEAD 仍是 `3b60f098e97dc4c4f3ed9c6355422b607db1c34c` + 当时的未提交改动）；
`research/prompts/m2-wave3-watermark-implementer-report.md` 摘要一节：23:46:40 UTC 对主工作区现状 `git apply --check` 退出 0，那一刻主工作区 `crates/` 与该快照逐文件相同（`diff -rq … | wc -l` → `0`）。
`sha256sum watermark.patch`（本次现查）= `3a9b91313ce46dc6b1386e256980075db720f79f9860a75dacaa335069836dfc`（与 `m2-wave3-code-r1-patches/watermark.patch` 一致）。
这一组同时把 `crates/mutations.tsv` 里守旧行为「回退到无文件那一版时若环里还有带文件的根，写之前拒绝」的 5 行删掉（报告第二节「二、这一轮写过的文件」：只删不加），新行为的变异改放六节的 `mutations-append.tsv`。

```diff
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -67,7 +67,6 @@
 步 3：零单元发布在记录与根之间少一道屏障	crates/singlefs-core/src/transaction.rs	        writer.perform(CommitStep::Barrier)?;\n        persist_the_root_then_rotate_the_system_configuration(\n            writer,\n            plan.txg,	        persist_the_root_then_rotate_the_system_configuration(\n            writer,\n            plan.txg,	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream	the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence
 步 3：只做过 mkfs 的池重开后把 mkfs 实例表按 1 槽记	crates/singlefs-core/src/mount.rs	        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;	        placement_of(&root.instance_table, TransactionUnit::TreeTable)?;	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
 步 3：树表 0 条的一版上写行时，这次要写的行没接进重写出去的那片实例表	crates/singlefs-core/src/mount.rs	            table_after.rows.extend_from_slice(&rows_written);	            table_after.rows.extend_from_slice(&[]);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
-步 4：回退到树表 0 条的根、而环里还留着带文件版本的根，不在写之前拒绝	crates/singlefs-core/src/mount.rs	    if target_has_no_file {\n        if let Some(file_version_root) = roots	    if false {\n        if let Some(file_version_root) = roots	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root	rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_is_refused_before_any_write
 步 5：算抬 F 上限时有效根的树表读不出，按「树表里没有这两棵树」猜而不是拒绝	crates/singlefs-core/src/mount.rs	        Ok(pointers) => Ok(pointers),\n        Err(failure) => Err(	        Ok(pointers) => Ok(pointers),\n        Err(_failure) if true => Ok(UserVisibleTreeRootPointers::ABSENT),\n        Err(failure) => Err(	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_is_refused_when	raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable
 步 3：取号不核判定时算出的号（瞬时读错让判定与取号算出不同号，写进新号之后才发现）	crates/singlefs-core/src/transaction.rs	    if recomputed != expected {	    if false {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- transient_system_configuration_read_errors_between	transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write
 第一个事务 步 1：mkfs 不核根环区域归属是不是第一版写死的 0 / 1 / 0	crates/singlefs-core/src/make_filesystem.rs	    if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {	    if false {	-p singlefs-harness --test first_transaction_step_one_mkfs -- region_layout_other_than	region_layout_other_than_zero_one_zero_is_refused_before_any_write
@@ -162,9 +161,6 @@
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条的改法）：胶水把回退目标「被抛弃」映射成「低于 F」	crates/singlefs-harness/src/model_comparison.rs	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetOnAbandonedTimeline\n        }	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetBelowEffectiveFloor\n        }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_onto_an_abandoned_root_above_the_floor	rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，C368 仍欠的那一半）：发布层把分配器的落点拒绝一律报成「每块盘上都没有」	crates/singlefs-core/src/transaction.rs	        unit: identity,\n        refusal,\n    })\n}	        unit: identity,\n        refusal: {\n            let _ = refusal;\n            PlacementRefusal::NoFreeSlotOnAnyDevice\n        },\n    })\n}	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条的改法）：胶水把小盘写满、各盘落点不一致也映射成单元区墙	crates/singlefs-harness/src/model_comparison.rs	        } => ObservedRefusalReason::Unexplained,	        } => explained(ModelRefusalReason::UnitAreaWall),	-p singlefs-harness --lib -- model_comparison::tests::only_a_placement_refused	only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall
-增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：回退到树表 0 条的根、而环里还有带文件的根（在候选集里、条款没写重新建树的诞生代）报成候选排除「低于 F」	crates/singlefs-core/src/mount.rs	            return Err(\n                MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion {\n                    target,\n                    file_version_root: RollbackTarget {\n                        instance: file_version_root.instance,\n                        checkpoint_txg: file_version_root.checkpoint_txg,\n                    },\n                },\n            );	            return Err(MountError::RollbackTargetNotACandidate {\n                target,\n                exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n            });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
-增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：回退到树表 0 条的根、而环里还有带文件的根报成候选排除「低于 F」；步 4 那条写死的用例判出	crates/singlefs-core/src/mount.rs	            return Err(\n                MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion {\n                    target,\n                    file_version_root: RollbackTarget {\n                        instance: file_version_root.instance,\n                        checkpoint_txg: file_version_root.checkpoint_txg,\n                    },\n                },\n            );	            return Err(MountError::RollbackTargetNotACandidate {\n                target,\n                exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n            });	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root	rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_is_refused_before_any_write
-增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：候选排除「低于 F」报成「树表 0 条、环里还有带文件的根」；偏向回退那一档抽样判出（这一档的必红是抽样断言，随测试周期的种子基重验，见 d13-item4-fua-r1 判决）	crates/singlefs-core/src/mount.rs	        return Err(MountError::RollbackTargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n        });	        return Err(\n            MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion {\n                target,\n                file_version_root: target,\n            },\n        );	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rollback_heavy_random_histories	rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms
 增补 3 第 2 件（代码三方第二轮判决第三节第 1 条的改法）：胶水把回退候选排除的三条揉成一条（「不在环里」也报成「低于 F」）	crates/singlefs-harness/src/model_comparison.rs	        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetNotInRing,	        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetBelowEffectiveFloor,	-p singlefs-harness --lib -- model_comparison::tests::each_rollback_candidate_exclusion	each_rollback_candidate_exclusion_maps_to_its_own_reason
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2h）：分配器用户数据那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling	unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2i）：分配器用户数据那一处把「小盘写满」报成「每块盘上都没有」；等大的小盘走不到，不等盘那条用例判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {\n                return Err(\n                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },\n                );\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	            DeviceAgreement::SomeDevicesWithoutAnswer(_full_devices) => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
@@ -175,7 +171,6 @@
 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条的改法）：「已知红第 0 条那一形只记不停」退回成判红就停；逼近分配记录墙那一段走不到墙	crates/singlefs-harness/src/history.rs	                if !continues_past_the_ring_turn_form {\n                    return Some(observation);\n                }	                if true || !continues_past_the_ring_turn_form {\n                    return Some(observation);\n                }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("raise_rollback_floor"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- raising_the_floor_with_a_second_empty_publish	raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds
 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("mount_rollback"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_a_root_whose_warm_up	rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds
-增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：候选排除「低于 F」报成「树表 0 条、环里还有带文件的根」；回退到 F 之下的写死用例判出（门禁五段各只红 1 段）	crates/singlefs-core/src/mount.rs	        return Err(MountError::RollbackTargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n        });	        return Err(\n            MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion {\n                target,\n                file_version_root: target,\n            },\n        );	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_below_the_effective_floor	rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor
 增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
 增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
 增补 3 第 3 件（用户 2026-09-20 定案第 1 条）：屏障不再切段（屏障进了枚举域，少一道屏障就把两段并成一段）	crates/singlefs-harness/src/crash.rs	            RecordedOperationKind::Barrier => {\n                if !current.is_empty() {\n                    segments.push(std::mem::take(&mut current));\n                }\n            }	            RecordedOperationKind::Barrier => {}	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- every_crash_state_of_a_written_out_history	every_crash_state_of_a_written_out_history_recovers_into_a_committed_version
@@ -301,7 +296,7 @@
 步 3（R1）：树表 0 条的一版上写行时，被换下的那片实例表不释放（defer 队列里少 2 槽）	crates/singlefs-core/src/transaction.rs	    allocator.release(released, txg);	    let _ = released;	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 3（三处写死的前提之二）：第一个文件版本照抄的实例表指针取错成树表指针	crates/singlefs-core/src/transaction.rs	            instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),	            instance_table: InstanceTablePlan::Carry(version_to_build_on.tree_table),	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 3（R8）：树表 0 条、而根指着的实例表或树表不是 mkfs 写的那一版时照样重建账（被换下的那一片成了空闲槽）	crates/singlefs-core/src/mount.rs	    if root.instance_table.head.birth_txg != CheckpointTxg(0)\n        || root.tree_table.head.birth_txg != CheckpointTxg(0)\n    {	    if false {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_third_writable_mount	a_third_writable_mount_on_a_version_whose_rows_were_written_is_refused_before_touching_the_allocator
-步 3：publish_first_file 不核要建在上面的那一版有没有过文件版本（再走一次会重新建树、重新发对象出生代）	crates/singlefs-core/src/transaction.rs	    if version_to_build_on.tree_identifier_watermark != TREE_IDENTIFIER_WATERMARK_AT_MKFS {	    if false {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
+步 3：publish_first_file 不核要建在上面的那一版有没有过文件版本（再走一次会重新建树、重新发对象出生代）；C511 第 3 步起判的是树表条数	crates/singlefs-core/src/transaction.rs	    if tree_table_entries != 0 {	    if false {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
 步 4：影子账不认树表 0 条的被抛弃根（它那片实例表既不隔离也不在任何账里，回退之后当空闲槽发出去）	crates/singlefs-core/src/mount.rs	            Some(placements)\n        }\n        Ok(false) => Some(	            let _ = placements;\n            None\n        }\n        Ok(false) => Some(	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- rolling_back_to_a_warm_up_root_before_any_file_version	rolling_back_to_a_warm_up_root_before_any_file_version_rewrites_only_the_instance_table
 C507（记录核对器的复用豁免比登记的候选宽，把真洞变哑）：复用豁免不看更晚那次写持没持久（回到落地版，更晚那次写没落盘的真洞上失声）	crates/singlefs-harness/src/crash.rs	                && in_place(later_index)	                && true	-p singlefs-harness --test second_transaction_step_zero_layer0 -- a_unit_whose_only_later_write	a_unit_whose_only_later_write_to_the_same_slot_never_landed_is_reported_missing_by_the_record_checker
 C507：复用豁免把「更晚那次写已持久」判反（合法复用被判成单元缺席，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-harness/src/crash.rs	                && in_place(later_index)	                && !in_place(later_index)	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -9,7 +9,7 @@
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
     INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES, MAPPING_ENTRY_BYTES, NODE_BYTES, SLOT_BYTES,
-    TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_TABLE_ENTRY_BYTES,
+    TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::image::{
@@ -460,9 +460,13 @@
         for entry in &tree_table.entries {
             self.walk_tree_table_entry(entry);
         }
+        // 中央映射树不进树表，引用它的是那条映射根指针（根记录里的，或由记录施加出来那一版的新根段里的）：
+        // I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取（D19（块指针的结构与宽度预算） 已定项 7），
+        // 与树表条目里的树 ID 对树表指着的根是同一个读法。号不写死 15：回退到树表 0 条的一版之后再发第一个文件版本，
+        // 八棵树从水位重新发号（C511（回退到无文件那一版之后诞生代怎么接））。
         if let Some(mapping) = self.read_index_node(
             mapping_root_pointer,
-            TREE_IDENTIFIER_CENTRAL_MAPPING,
+            parse_node_pointer(mapping_root_pointer).birth_tree,
             KEY_SCHEMA_MAPPING,
             "中央映射树的根",
         ) {
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -20,10 +20,7 @@
 
 use std::cell::Cell;
 
-use singlefs_format::{
-    DATA_UNIT_BYTES, INODE_RECORD_BYTES, MAPPING_KEY_BYTES, NODE_BYTES,
-    TREE_IDENTIFIER_CENTRAL_MAPPING,
-};
+use singlefs_format::{DATA_UNIT_BYTES, INODE_RECORD_BYTES, MAPPING_KEY_BYTES, NODE_BYTES};
 
 use crate::address::{
     DataUnitIndexInFile, FileOffsetInBytes, InodeNumber, SlotNumber, TreeIdentifier,
@@ -327,9 +324,11 @@
         unit_filesystem_identifier(&root.filesystem_identifier);
 
     // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）；它自己不进映射（自举豁免）。
+    // 它不进树表，是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
+    let mapping_tree = root.mapping_root.head.birth_tree;
     let mapping_root = read_tree_root(
         reader,
-        TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
+        mapping_tree,
         usize::try_from(MAPPING_KEY_BYTES).expect("27"),
         &root.mapping_root,
         root,
@@ -337,7 +336,7 @@
     )?;
     if mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL {
         return Err(OpenPoolForReadFailure::TreeLevelUnexpected {
-            tree: TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
+            tree: mapping_tree,
             expected_level: CENTRAL_MAPPING_TREE_ROOT_LEVEL,
             found_level: mapping_root.level,
         });
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -12,10 +12,10 @@
 use crate::recovery::{
     allocation_records_of_version_without_file, allocation_records_under_root, choose_root,
     choose_system_configuration, effective_rollback_floor, highest_root_txg,
-    instance_table_of_root, readable_roots, rebuild_version, replay_journal,
-    rollback_high_water_of_root, scan_journal, tree_table_has_no_entries,
-    user_visible_tree_root_pointers, JournalScanReport, RebuildVersionFailure, RebuiltVersion,
-    RecoveryFailure, UserVisibleTreeRootPointers,
+    highest_tree_identifier_watermark_in_the_ring, instance_table_of_root, readable_roots,
+    rebuild_version, replay_journal, rollback_high_water_of_root, scan_journal,
+    tree_table_has_no_entries, user_visible_tree_root_pointers, JournalScanReport,
+    RebuildVersionFailure, RebuiltVersion, RecoveryFailure, UserVisibleTreeRootPointers,
 };
 use crate::root_record::RootRecord;
 use crate::root_ring::target_for_publish;
@@ -52,19 +52,6 @@
         target: RollbackTarget,
         exclusion: RollbackCandidateExclusion,
     },
-    /// 回退的目标那一版树表 0 条（还没发布过文件版本），而根环里还有别的根带着文件版本（树表非 0 条）：
-    /// 回退本身做得成，但回退之后这个池要再往前走只剩「再发一次第一个文件版本」这一条路，而那一次会**重新建树、重新发对象出生代**——
-    /// 树 11 的树表条目诞生 txg 就与环里那些旧根记的不同（I-9.14（树表条目的诞生 txg 跨根不变） 逐字要求跨根不变），
-    /// inode 1 的对象出生代也换了一代（I-9.10（对象出生代与 inode 记录相符））。
-    /// 「回退到没有文件的一版之后重新建树，诞生代怎么算」**没有条款——这是一处设计空白，交主 agent**
-    /// （2026-09-23 崩溃注入快档三条新发现都是它：I-9.10 与 I-9.14 红在同一格）。
-    /// 在它有条款之前，这一格在任何写之前拒绝，盘上逐字节不变。
-    /// **不是候选排除**（候选集只有 `RollbackCandidateExclusion` 那三条，D23（journal 的角色与格式） 已定项 14；
-    /// C493（回退候选集条文与实现说反话） 由此还清）：环里没有带文件的根时，回退到树表 0 条的根照常做。
-    RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion {
-        target: RollbackTarget,
-        file_version_root: RollbackTarget,
-    },
     /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
     RollbackFloorAboveCeiling {
         requested: CheckpointTxg,
@@ -295,6 +282,10 @@
     previous_row: PreviousInstanceRow,
     first_txg: CheckpointTxg,
     next_counter: u64,
+    /// 本实例第一次发布（写行那一次）要带的树 ID 水位：根环里全部自证过的根与环里全部自证通过的记录各自带的水位取 max
+    /// （D8（核心索引结构） 已定项 8 ②；`recovery::highest_tree_identifier_watermark_in_the_ring`）。回退时它可以高于
+    /// 回退到的那一版自己带的——被抛弃时间线上的根仍承载它们那一代发过的号，回退之后再发第一个文件版本就从这里往上发。
+    tree_identifier_watermark_of_the_ring: u64,
     /// 新实例的根带的回退下界 = 恢复后生效的 F（各幸存盘所带 F 最大值的最小值）：与重建分配器时回收用的同一个值，
     /// 一条根带的 F 与它的记账行才说同一件事。
     effective_floor: CheckpointTxg,
@@ -326,6 +317,28 @@
     CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)
 }
 
+/// 本实例第一次发布要带的树 ID 水位（D8（核心索引结构） 已定项 8 ②「根环里全部根记录该字段的 max」）：
+/// 根环里全部自证过的根与 `records`（环里全部自证通过的记录）各自带的水位取 max，再与这次要接在后面的那一版自己带的取 max——
+/// 那一版是从环里读出来的根或从环里的记录施加出来的，按构造已经在内；再取一次是为了两次读环之间的瞬时读错
+/// （择根那一遍读到了它、这一遍没读到）不把水位拉低。读不出的根怎么算见 `highest_tree_identifier_watermark_in_the_ring`。
+fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
+    devices: &[(DeviceIdentity, Device)],
+    system_configuration: &crate::system_configuration::SystemConfiguration,
+    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    version_to_build_on: &RootRecord,
+) -> u64 {
+    highest_tree_identifier_watermark_in_the_ring(
+        devices,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+        records,
+    )
+    .map_or(version_to_build_on.tree_identifier_watermark, |ring| {
+        ring.max(version_to_build_on.tree_identifier_watermark)
+    })
+}
+
 /// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
 /// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。
 #[must_use]
@@ -852,6 +865,7 @@
                 new_inode_records: &[],
                 instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                 tree_birth_txg: current.tree_birth_txg(),
+                // 一个树 ID 都不发：现行那一版的水位就是根环里的 max（本会话每次发布只照抄或推高它）。
                 tree_identifier_watermark: current.root.tree_identifier_watermark,
                 rollback_floor: new_floor,
             },
@@ -879,6 +893,9 @@
     counter: u64,
     instance: InstanceGeneration,
     rollback_floor: CheckpointTxg,
+    /// 根环里全部根记录的树 ID 水位取 max（`InstanceStart::tree_identifier_watermark_of_the_ring`）：写行那次发布一个号都不发，
+    /// 新根的水位就是它（D8（核心索引结构） 已定项 8 ②）。
+    tree_identifier_watermark: u64,
 }
 
 /// 写行那次发布、上一版带文件：在上一版那张实例表后面接上这次写的行，重写实例表与四个固定点单元（D18（块里携带什么信息） 已定项 11；
@@ -909,7 +926,7 @@
             new_inode_records: &[],
             instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),
             tree_birth_txg: previous.tree_birth_txg(),
-            tree_identifier_watermark: previous.root.tree_identifier_watermark,
+            tree_identifier_watermark: identity.tree_identifier_watermark,
             rollback_floor: identity.rollback_floor,
         },
         Some(previous),
@@ -939,13 +956,17 @@
                 new_inode_records: &[],
                 instance_table: InstanceTablePlan::Carry(current_file_version.root.instance_table),
                 tree_birth_txg: current_file_version.tree_birth_txg(),
+                // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
                 tree_identifier_watermark: current_file_version.root.tree_identifier_watermark,
                 rollback_floor: current_file_version.root.rollback_floor,
             };
             // 取号之前那道准入按 `PublishShape::EMPTY_PUBLISH` 算这一次要加几条分配记录；这里钉住它算的就是这张计划的形状。
             // 形状要先把计划算成「这次之后 inode 树是什么样、重写哪些角色」才知道（`PublishPlan::resolve`），
             // 而算它只读、不发写：算不过就在这里交回，盘上逐字节不变。
-            let resolved_empty_publish = plan.resolve(Some(current_file_version))?;
+            let resolved_empty_publish = plan.resolve(
+                Some(current_file_version),
+                &current_file_version.tree_identifiers,
+            )?;
             assert_eq!(
                 resolved_empty_publish.shape(),
                 PublishShape::EMPTY_PUBLISH,
@@ -969,6 +990,10 @@
                 instance,
                 back_chain: back_chain_of(&current_version_without_file.record_bytes),
                 rollback_floor: current_version_without_file.root.rollback_floor,
+                // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
+                tree_identifier_watermark: current_version_without_file
+                    .root
+                    .tree_identifier_watermark,
             },
         )
         .map(PoolVersion::WithoutFile)
@@ -1158,6 +1183,7 @@
                     counter: start.next_counter,
                     instance,
                     rollback_floor: start.effective_floor,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                 },
             )?)
         }
@@ -1176,6 +1202,7 @@
                         instance,
                         back_chain: 0,
                         rollback_floor: start.effective_floor,
+                        tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                     },
                 )
                 .map_err(PublishError::from)?,
@@ -1197,6 +1224,7 @@
                     back_chain: 0,
                     rollback_floor: start.effective_floor,
                     instance_table_records: &instance_table_records,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                 },
             )?)
         }
@@ -1280,6 +1308,12 @@
         .unwrap_or(0)
         + 1;
     let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records);
+    let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
+        devices,
+        &system_configuration,
+        &records,
+        &effective_root,
+    );
     let rebuilt = rebuilt_allocator(
         devices,
         &system_configuration,
@@ -1310,6 +1344,7 @@
             previous_row,
             first_txg,
             next_counter,
+            tree_identifier_watermark_of_the_ring,
             effective_floor,
             abandoned_roots_unreadable,
             shadow_ledger: ShadowLedger::On,
@@ -1379,24 +1414,10 @@
     // 回退到树表 0 条的根（例如第一个事务里 txg 1、2 的暖机根）：候选集只有上面那三条（D23（journal 的角色与格式） 已定项 14），
     // 「树表 0 条」不在其内，照常回退——回退行走写行那条只重写实例表的发布路径
     // （`publish_instance_table_on_version_without_file`；C493（回退候选集条文与实现说反话） 由此还清）。
-    // 只有环里还留着带文件版本的根时拒绝，理由见那个成员的文档注释（重新建树的诞生代没有条款）。
+    // 环里还留着带文件版本的根时同样照常回退（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步）：回退行那次发布带环里的
+    // 树 ID 水位 max（`tree_identifier_watermark_of_the_ring`），之后再发第一个文件版本从它往上发号，不重发被抛弃的根用过的号；
+    // I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，被这次回退切掉的那几条不与新线比。
     let target_has_no_file = tree_table_has_no_entries(&*devices, &target_root)?;
-    if target_has_no_file {
-        if let Some(file_version_root) = roots
-            .iter()
-            .find(|root| matches!(tree_table_has_no_entries(&*devices, root), Ok(false)))
-        {
-            return Err(
-                MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion {
-                    target,
-                    file_version_root: RollbackTarget {
-                        instance: file_version_root.instance,
-                        checkpoint_txg: file_version_root.checkpoint_txg,
-                    },
-                },
-            );
-        }
-    }
     // R_old 自己那条记录读得出就拿它当上一版的记录（只有 jsn 会被用到），读不出就拿最大 jsn 那条顶着，与可写挂载同一条路。
     // 树表 0 条的目标用不到记录（只做过 mkfs 的池环里一条都没有）：那一格一条都拿不出来也照走，与 `mount_writable` 同一条读法；
     // 「一条记录都拿不出来」只有带文件版本的目标才是错（从盘上重建上一版要一条记录顶着）。
@@ -1432,6 +1453,15 @@
         maximum_applied_transaction: 0,
     };
     let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records);
+    // 回退到的那一版自己带的水位可以低于环里被这次回退抛弃的根带的（回退到树表 0 条的暖机根，而环里还留着带文件版本的根）：
+    // 那些根发过的号不许再发（D8（核心索引结构） 已定项 8 ②），回退行那次发布带的是环里的 max，
+    // 之后再发第一个文件版本就从它往上发（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步）。
+    let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
+        devices,
+        &system_configuration,
+        &records,
+        &target_root,
+    );
     // 影子账：这次回退新抛弃的根 = 根环里 (txg, 实例) 大于 R_old 的每一条可读根；只被它们引用的槽隔离，
     // 连同按实例表早已被抛弃的根一起在重建里算。
     let newly_abandoned = |root: &RootRecord| {
@@ -1466,6 +1496,7 @@
             previous_row,
             first_txg,
             next_counter,
+            tree_identifier_watermark_of_the_ring,
             effective_floor,
             abandoned_roots_unreadable,
             shadow_ledger,
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -14,7 +14,7 @@
     DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
     INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT,
     MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
-    SYSTEM_CONFIGURATION_SLOT_BYTES, TREE_IDENTIFIER_CENTRAL_MAPPING, UNIT_AREA_START_SLOT,
+    SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
 };
 
 use crate::address::{
@@ -42,8 +42,8 @@
     SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
 };
 use crate::transaction::{
-    InodeLeafContainerVersion, PublishedUnit, TransactionOutput, TransactionUnit,
-    FIRST_INODE_NUMBER,
+    FileVersionTreeIdentifiers, InodeLeafContainerVersion, PublishedUnit, TransactionOutput,
+    TransactionUnit, FIRST_INODE_NUMBER,
 };
 use crate::unit::{
     data_unit_payload, parse_data_unit, parse_index_node, parse_packed_unit,
@@ -248,8 +248,8 @@
 
 /// 按指针里的位置条目读一个单元：逐条试，整单元 CRC-32C 等于条目里的校验和才算读到。
 /// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
-pub(crate) fn read_unit_via_locations(
-    reader: &dyn PoolReader,
+pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
     locations: &[LocationEntry; 2],
     unit_bytes: usize,
 ) -> Result<Vec<u8>, RecoveryFailure> {
@@ -443,6 +443,40 @@
     highest
 }
 
+/// 本实例第一次发布要带的树 ID 水位：根环全部自证过的根的「树 ID 水位」取 max（D8（核心索引结构） 已定项 8 ②
+/// 「根环里全部根记录该字段的 max」——被抛弃时间线上的根也算，它们仍承载那一代发过的号），
+/// 并上 `records`（环里全部自证通过的 journal 记录）新根段里带的水位（D23（journal 的角色与格式） 已定项 15）。
+///
+/// 记录那一半是根读不出时的最保守读法：根槽撕了、被盖了或那块盘掉了，而它那次发布的记录还在，就按记录带的水位算
+/// （C342（树 ID 水位在根读不出时退回去重发） 的载体；条款没写读不出的根怎么算，报告里交主 agent）。
+/// 根与它那条记录都读不出的那一格仍然盖不住。与新实例第一次发布的 txg 取 max(环里的根, 环里的记录) 同一个形态
+/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。一条根、一条记录都没有时 None。
+#[must_use]
+pub fn highest_tree_identifier_watermark_in_the_ring<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    region_devices: &[DeviceIdentity; 3],
+    immutable_sizes: &SystemImmutableSizes,
+    filesystem_identifier: &[u8; 16],
+    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
+) -> Option<u64> {
+    let mut highest: Option<u64> = records
+        .values()
+        .map(|record| record.new_tree_identifier_watermark)
+        .max();
+    visit_valid_roots(
+        reader,
+        region_devices,
+        immutable_sizes,
+        filesystem_identifier,
+        |root| {
+            highest = Some(highest.map_or(root.tree_identifier_watermark, |current| {
+                current.max(root.tree_identifier_watermark)
+            }));
+        },
+    );
+    highest
+}
+
 /// 根环里全部自证过的根，每个可读根槽一条（回退候选集与影子账从这里取）。
 #[must_use]
 pub fn readable_roots<Reader: PoolReader + ?Sized>(
@@ -664,13 +698,25 @@
     reader: &dyn PoolReader,
     root: &RootRecord,
 ) -> Result<bool, RecoveryFailure> {
+    Ok(tree_table_entry_count(reader, root)? == 0)
+}
+
+/// 一条根指着的树表有几条条目：0 条就是 mkfs 的第 0 代树表（这条根下面还没有发布过文件版本）。
+/// 发布路径（`transaction::publish_first_file`）拿写者手里那串盘直接读，所以读者按泛型收、不要求有大小。
+///
+/// # Errors
+/// 树表单元读不到、解不开。
+pub fn tree_table_entry_count<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    root: &RootRecord,
+) -> Result<usize, RecoveryFailure> {
     let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
     let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
     let tree_table =
         parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
             what: "树表单元",
         })?;
-    Ok(tree_table.entries.is_empty())
+    Ok(tree_table.entries.len())
 }
 
 /// 所选根的实例在它自己指着的实例表里有回退行时，回退行的 W（D23（journal 的角色与格式） 已定项 14 前缀第五条：
@@ -759,6 +805,27 @@
     let inode_root_pointer = pointer_of(TREE_KIND_INODE)?;
     let allocation_pointer = pointer_of(TREE_KIND_ALLOCATION)?;
     let accounting_pointer = pointer_of(TREE_KIND_ACCOUNTING)?;
+    // 这一版那八棵树各自的号照盘上读回来：七棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里
+    // 它那条根指针的出生树（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11）。接着这一版的发布照抄它们，不另发。
+    let tree_of = |kind: u16| {
+        tree_table_entries
+            .iter()
+            .find(|entry| entry.kind == kind)
+            .map(|entry| entry.tree)
+            .ok_or(RecoveryFailure::UnitMalformed {
+                what: "树表里没有那棵树",
+            })
+    };
+    let tree_identifiers = FileVersionTreeIdentifiers {
+        extent: tree_of(TREE_KIND_EXTENT)?,
+        inode: tree_of(TREE_KIND_INODE)?,
+        allocation_records: tree_of(TREE_KIND_ALLOCATION)?,
+        accounting: tree_of(TREE_KIND_ACCOUNTING)?,
+        central_mapping: root.mapping_root.head.birth_tree,
+        livelist: tree_of(TREE_KIND_LIVELIST)?,
+        sparse_side_table: tree_of(TREE_KIND_SPARSE_SIDE_TABLE)?,
+        deadlist: tree_of(TREE_KIND_DEADLIST)?,
+    };
     let read_node = |pointer: &NodePointer, what: &'static str| {
         let bytes = read_unit_via_locations(reader, &pointer.locations, node_bytes)?;
         let node =
@@ -974,6 +1041,7 @@
         allocation_records,
         accounting_entries,
         tree_table_entries,
+        tree_identifiers,
         inode_record,
         inode_leaf_containers,
         mapped_units,
@@ -1459,9 +1527,10 @@
         inode: take(TREE_KIND_INODE)?,
         allocation: take(TREE_KIND_ALLOCATION)?,
         accounting: take(TREE_KIND_ACCOUNTING)?,
+        // 中央映射树不进树表：它是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
         mapping: read_tree_root(
             reader,
-            TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
+            root.mapping_root.head.birth_tree,
             27,
             &root.mapping_root,
             root,
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -52,7 +52,10 @@
     STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
     TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
 };
-use crate::recovery::{highest_root_instance, verified_system_configuration_slots};
+use crate::recovery::{
+    highest_root_instance, tree_table_entry_count, verified_system_configuration_slots,
+    RecoveryFailure,
+};
 use crate::root_record::RootRecord;
 use crate::root_ring::{slot_offset, target_for_publish};
 use crate::system_configuration::{
@@ -503,6 +506,8 @@
                 instance,
                 back_chain: previous_record_bytes.as_deref().map_or(0, back_chain_of),
                 rollback_floor: genesis.rollback_floor,
+                // mkfs 同一个进程里那条流：mkfs 之后根环里只有 mkfs 的第 0 代根，暖机根照抄它（D16（发布语义） 已定项 9）。
+                tree_identifier_watermark: genesis.tree_identifier_watermark,
             },
         )?;
         previous_counter = output.record.counter;
@@ -529,6 +534,10 @@
     /// 反向链：本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
     pub back_chain: u32,
     pub rollback_floor: CheckpointTxg,
+    /// 这次写进根记录与记录新根段的树 ID 水位：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)
+    /// （D8（核心索引结构） 已定项 8 ②），零单元发布一个号都不发 ⇒ 就是根环里的 max。本实例的第一次发布由挂载按环算
+    /// （回退到的那一版的水位可以低于环里被抛弃的根带的）；之后接着本会话现行那一版的根照抄。
+    pub tree_identifier_watermark: u64,
 }
 
 /// 树表 0 条的一版上一次发布写出的东西：根、记录、记录的字节（下一条的反向链要罩它）、按结构种类的写。
@@ -563,7 +572,8 @@
 }
 
 /// 零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换；
-/// 空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号与回退下界。
+/// 空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号、回退下界
+/// 与树 ID 水位（取计划里给的，D8（核心索引结构） 已定项 8 ②）。
 /// 第一次可写挂载的暖机与「只做过 mkfs 的池」上的可写挂载都走它（第一个事务的字节不变）。
 ///
 /// # Errors
@@ -583,7 +593,7 @@
         filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
         new_tree_table: previous_root.tree_table,
         new_mapping_root: previous_root.mapping_root,
-        new_tree_identifier_watermark: previous_root.tree_identifier_watermark,
+        new_tree_identifier_watermark: plan.tree_identifier_watermark,
         new_rollback_floor: plan.rollback_floor,
         named: Vec::new(),
     };
@@ -593,7 +603,7 @@
         instance: plan.instance,
         checkpoint_txg: plan.txg,
         tree_table: previous_root.tree_table,
-        tree_identifier_watermark: previous_root.tree_identifier_watermark,
+        tree_identifier_watermark: plan.tree_identifier_watermark,
         rollback_floor: plan.rollback_floor,
         instance_table: previous_root.instance_table,
         mapping_root: previous_root.mapping_root,
@@ -644,6 +654,9 @@
     pub back_chain: u32,
     pub rollback_floor: CheckpointTxg,
     pub instance_table_records: &'records [Vec<u8>],
+    /// 这次写进根记录与记录新根段的树 ID 水位，同 `ZeroUnitPublishPlan::tree_identifier_watermark`：写行那次发布一个树 ID 都不发，
+    /// 就是根环里全部根记录的 max（D8（核心索引结构） 已定项 8 ②）——回退到树表 0 条的一版时它高于那一版自己带的。
+    pub tree_identifier_watermark: u64,
 }
 
 /// 写行那次发布，上一版树表 0 条：**重写实例表与分配记录树两个单元**，落盘顺序同别的发布（D16（发布语义） 已定项 7）——
@@ -662,8 +675,8 @@
 /// 落点从上一版根记录里那两条指针取，三样逐盘核过（`placement_to_release_after_checking_every_device`）。
 /// 上一版是 mkfs 的第 0 代（分配记录树根指针全零）时只释放实例表那一片。
 ///
-/// 新根照上一版的根，只换 checkpoint_txg、实例代号、回退下界、实例表指针与分配记录树根指针：树表、映射树根、树 ID 水位都照抄
-/// （这一版的树表仍是 mkfs 那片 0 条的）。
+/// 新根照上一版的根，只换 checkpoint_txg、实例代号、回退下界、实例表指针、分配记录树根指针与树 ID 水位（取计划里给的，
+/// D8（核心索引结构） 已定项 8 ②）：树表、映射树根照抄（这一版的树表仍是 mkfs 那片 0 条的）。
 ///
 /// # Errors
 /// 被换下的那两片任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、这次之后的分配记录装不进一个节点
@@ -773,8 +786,8 @@
 /// 两条发布路径共用这一处——带文件的那一版的 t5 与树表 0 条那一版写行时的那一片，装出来的字节按同一条规则；
 /// 两处各抄一份会分叉，而「重开之后从这一片重建出来的账与发布时那个分配器相同」正压在两处装的是同一个东西上。
 ///
-/// `tree` 两条路径不同，这是唯一的差别：带文件的那一版里它是登记在树表里的分配记录树（树 ID 13）；
-/// 树表 0 条那一版还没登记过任何树（树 ID 水位仍是 mkfs 的 11，写 13 会当场判红 I-7.8（树 ID 水位不小于盘上出现过的最大树 ID）），
+/// `tree` 两条路径不同，这是唯一的差别：带文件的那一版里它是登记在树表里的分配记录树（第一个文件版本那次发的号，mkfs 那条流上是 13）；
+/// 树表 0 条那一版还没登记过任何树（那一版的水位还没被那几个号推过，写 13 会当场判红 I-7.8（树 ID 水位不小于盘上出现过的最大树 ID）），
 /// 那一片**不属于任何树**（树 ID 0）、由根记录独占持有，与树表单元、实例表单元同一个身份（D22（单元原子性怎么合成） 已定项 7 / 已定项 12）。
 ///
 /// # Panics
@@ -904,14 +917,15 @@
         filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
         new_tree_table: previous_root.tree_table,
         new_mapping_root: previous_root.mapping_root,
-        new_tree_identifier_watermark: previous_root.tree_identifier_watermark,
+        new_tree_identifier_watermark: plan.tree_identifier_watermark,
         new_rollback_floor: plan.rollback_floor,
         // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项——实例表与分配记录树。
         named: vec![
             NamedUnit {
                 locations,
                 unit_class: identity.unit_class(),
-                birth_tree: identity.tree(),
+                // 实例表单元不属于任何一棵树（树 0）；这一版还没发过树 ID，不按 `TransactionUnit::tree` 取。
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                 birth_txg: txg,
                 key_tail: node_key_tail(instance, birth_sequence),
             },
@@ -931,7 +945,7 @@
         instance,
         checkpoint_txg: txg,
         tree_table: previous_root.tree_table,
-        tree_identifier_watermark: previous_root.tree_identifier_watermark,
+        tree_identifier_watermark: plan.tree_identifier_watermark,
         rollback_floor: plan.rollback_floor,
         instance_table: instance_table_pointer,
         mapping_root: previous_root.mapping_root,
@@ -1129,19 +1143,16 @@
         }
     }
 
-    /// 点名项里的归属树；树表单元不属于任何一棵树（写 0）。
+    /// 点名项里的归属树：这一版那几棵树各自的号（`trees`，第一个文件版本那次从水位发出来、之后照抄）；
+    /// 树表单元与实例表单元不属于任何一棵树（写 0）。
     #[must_use]
-    pub const fn tree(self) -> TreeIdentifier {
+    pub const fn tree(self, trees: &FileVersionTreeIdentifiers) -> TreeIdentifier {
         match self {
-            TransactionUnit::Data | TransactionUnit::ExtentRoot => {
-                TreeIdentifier(TREE_IDENTIFIER_EXTENT)
-            }
-            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InodeRoot => {
-                TreeIdentifier(TREE_IDENTIFIER_INODE)
-            }
-            TransactionUnit::AllocationTree => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
-            TransactionUnit::AccountingTree => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
-            TransactionUnit::MappingTree => TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
+            TransactionUnit::Data | TransactionUnit::ExtentRoot => trees.extent,
+            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InodeRoot => trees.inode,
+            TransactionUnit::AllocationTree => trees.allocation_records,
+            TransactionUnit::AccountingTree => trees.accounting,
+            TransactionUnit::MappingTree => trees.central_mapping,
             TransactionUnit::TreeTable | TransactionUnit::InstanceTable => {
                 TreeIdentifier(TREE_IDENTIFIER_NONE)
             }
@@ -1172,6 +1183,110 @@
     CommitGenerated(UnitFootprint),
 }
 
+/// 带文件的一版那八棵树各自的树 ID，按 D8（核心索引结构） 已定项 11 的次序：extent、inode、分配记录、记账、中央映射、
+/// livelist、稀疏旁表、deadlist。
+///
+/// 第一个文件版本那次发布从它要建在上面的那一版的树 ID 水位起，按这个次序连号发这八个号
+/// （[`FileVersionTreeIdentifiers::issued_from_watermark`]）；之后每一版照抄，再不发号。
+/// mkfs 那条流上那一版的水位是 mkfs 种下的 11，发出来就是已定项 11 登记的 11..18；回退到树表 0 条的一版之后再发，
+/// 那一版的水位带着回退之前根环里的 max（D8（核心索引结构） 已定项 8 ②），发出来的号高于此前发过的每一个——号永不重发
+/// （里程碑「第二个事务」增补 2 收口表第 ④ 行；C511（回退到无文件那一版之后诞生代怎么接））。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct FileVersionTreeIdentifiers {
+    pub extent: TreeIdentifier,
+    pub inode: TreeIdentifier,
+    pub allocation_records: TreeIdentifier,
+    pub accounting: TreeIdentifier,
+    pub central_mapping: TreeIdentifier,
+    pub livelist: TreeIdentifier,
+    pub sparse_side_table: TreeIdentifier,
+    pub deadlist: TreeIdentifier,
+}
+
+/// 从水位起连号发八个树 ID 时，水位装不下：盘上读来的 8 字节水位离 `u64::MAX` 不到八个号。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
+    pub tree_identifier_watermark: u64,
+}
+
+impl FileVersionTreeIdentifiers {
+    /// 八棵树在发号次序里各自离水位几个号：mkfs 水位 11 时发出来的就是 D8（核心索引结构） 已定项 11 登记的常量，
+    /// 偏移从那几个常量现算，不另抄一份次序。
+    const OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER: [u64; 8] = [
+        TREE_IDENTIFIER_EXTENT - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_INODE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_ALLOCATION_RECORDS - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_ACCOUNTING - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_CENTRAL_MAPPING - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_LIVELIST - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_SPARSE_SIDE_TABLE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_DEADLIST - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+    ];
+
+    /// 从 `tree_identifier_watermark`（下一个可用号）起按 D8（核心索引结构） 已定项 11 的次序连号发八个号，
+    /// 连同发完之后的下一个可用号（最大那个号 + 1）。
+    ///
+    /// # Errors
+    /// 水位加八个号越过 `u64::MAX`（水位是盘上读来的 8 字节）⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
+    pub fn issued_from_watermark(
+        tree_identifier_watermark: u64,
+    ) -> Result<(Self, u64), TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees> {
+        let next_available_after_this_issue = tree_identifier_watermark
+            .checked_add(
+                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+            )
+            .ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
+                tree_identifier_watermark,
+            })?;
+        let issued =
+            Self::OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER.map(|offset_from_the_watermark| {
+                TreeIdentifier(tree_identifier_watermark + offset_from_the_watermark)
+            });
+        let [extent, inode, allocation_records, accounting, central_mapping, livelist, sparse_side_table, deadlist] =
+            issued;
+        let trees = FileVersionTreeIdentifiers {
+            extent,
+            inode,
+            allocation_records,
+            accounting,
+            central_mapping,
+            livelist,
+            sparse_side_table,
+            deadlist,
+        };
+        assert_eq!(
+            trees.highest().0 + 1,
+            next_available_after_this_issue,
+            "八个号连号发、最大的是 deadlist：发完之后的下一个可用号就是 mkfs 那条流上第一次发布之后的水位 19 平移过来"
+        );
+        Ok((trees, next_available_after_this_issue))
+    }
+
+    /// 八个号按发号次序。
+    #[must_use]
+    pub const fn in_issue_order(&self) -> [TreeIdentifier; 8] {
+        [
+            self.extent,
+            self.inode,
+            self.allocation_records,
+            self.accounting,
+            self.central_mapping,
+            self.livelist,
+            self.sparse_side_table,
+            self.deadlist,
+        ]
+    }
+
+    /// 这八个号里最大的那一个。
+    #[must_use]
+    pub fn highest(&self) -> TreeIdentifier {
+        self.in_issue_order()
+            .into_iter()
+            .max()
+            .expect("八个号，数组非空")
+    }
+}
+
 /// 一个写出去的单元：落点、身份、字节。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct PublishedUnit {
@@ -1195,6 +1310,9 @@
     pub allocation_records: Vec<AllocationRecord>,
     pub accounting_entries: Vec<AccountingEntry>,
     pub tree_table_entries: Vec<TreeTableEntry>,
+    /// 这一版那八棵树各自的号：第一个文件版本那次从水位发出来，之后每一版照抄（从盘上重建的版本按树表条目的种类与根记录里
+    /// 中央映射树根指针的出生树读回来）。
+    pub tree_identifiers: FileVersionTreeIdentifiers,
     /// 这一版第一个文件那条 inode 记录（覆盖写要接着它的对象出生代）。
     pub inode_record: InodeRecord,
     /// 这一版 inode 树里的全部叶容器，左起按 key 序：每片的身份、它装的记录、这一版它的指针
@@ -1579,16 +1697,26 @@
         version_to_build_on: CheckpointTxg,
         previous_record: Option<(CheckpointTxg, u64)>,
     },
-    /// `publish_first_file` 要建在上面的那一版已经有过文件版本（树 ID 水位是第一次发布之后的
-    /// `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH`，不是 mkfs 的 `TREE_IDENTIFIER_WATERMARK_AT_MKFS`）：
-    /// 这条路径按「树还没建起来」写——`previous` 传 `None`、inode 树与 extent 树从头建、树表条目的诞生 txg 与
-    /// inode 1 的对象出生代都取这次发布的 txg。接在已经有文件的一版后面写出来的那条根，会与根环里那些旧根在
-    /// I-9.14（树表条目的诞生 txg 跨根不变） 与 I-9.10（对象出生代与 inode 记录相符） 上对不上
-    /// （2026-09-23 崩溃注入快档打中；此前是 txg 写死 3 顺带挡住的，射程收窄之后要自己判）。
+    /// `publish_first_file` 要建在上面的那一版已经有过文件版本（它的树表不是 0 条，`tree_table_entries` 是条数）：
+    /// 这条路径按「树还没建起来」写——`previous` 传 `None`、八棵树从水位重新发号、inode 树与 extent 树从头建、
+    /// 树表条目的诞生 txg 与 inode 1 的对象出生代都取这次发布的 txg。接在已经有文件的一版后面写出来的那条根，
+    /// 会与同一条时间线上的旧根在 I-9.14（树表条目的诞生 txg 跨根不变） 与 I-9.10（对象出生代与 inode 记录相符） 上对不上
+    /// （2026-09-23 崩溃注入快档打中）。判的是树表条数，不是水位（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：
+    /// 回退到树表 0 条的一版时水位带着根环里的 max，已经不是 mkfs 的 11）。
     /// 同一个文件再写一版走 `publish_overwrite`。一个字节都不写。
     FirstFileVersionOnAVersionThatAlreadyHasAFile {
-        tree_identifier_watermark: u64,
+        tree_table_entries: usize,
+    },
+    /// `publish_first_file` 要判「那一版有没有过文件版本」得读它的树表单元，而那一片两份都读不出或解不开
+    /// （`failure` 原样带着恢复路径那一格的原因）：判不了就不写，一个字节都不写。
+    TreeTableOfTheVersionToBuildOnUnreadable {
+        failure: RecoveryFailure,
     },
+    /// `publish_first_file` 从那一版的树 ID 水位起连号发八棵树的号，而水位（盘上读来的 8 字节）加八个号越过 `u64::MAX`：
+    /// 发不出号就不写，一个字节都不写。
+    TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(
+        TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees,
+    ),
     BlockDevice(BlockDeviceError),
 }
 
@@ -1741,6 +1869,7 @@
     pub instance_table: InstanceTablePlan,
     /// 树表条目的诞生 txg：树建起来那次发布，之后每一版重写都不改。
     pub tree_birth_txg: CheckpointTxg,
+    /// 这次发布写进根记录与记录新根段的树 ID 水位（D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)）。
     pub tree_identifier_watermark: u64,
     pub rollback_floor: CheckpointTxg,
 }
@@ -1846,13 +1975,15 @@
         records
     }
 
-    /// 把这张计划算成「这次之后 inode 树是什么样、这次重写哪些角色」。
+    /// 把这张计划算成「这次之后 inode 树是什么样、这次重写哪些角色」。`trees` 是这一版那八棵树的号
+    /// （接在上一版之后就是上一版的，第一个文件版本是这次发出来的），新分裂出来的叶容器的出生树取其中的 inode 树。
     ///
     /// # Errors
     /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）。
     pub fn resolve(
         &self,
         previous: Option<&TransactionOutput>,
+        trees: &FileVersionTreeIdentifiers,
     ) -> Result<ResolvedPublish, PublishError> {
         let containers_before = previous
             .map(TransactionOutput::inode_leaf_container_contents)
@@ -1861,7 +1992,7 @@
             &containers_before,
             &self.inode_record_writes(),
             self.txg,
-            TreeIdentifier(TREE_IDENTIFIER_INODE),
+            trees.inode,
         )?;
         let mut rewritten_roles = Vec::new();
         if self.file.is_some() {
@@ -1913,12 +2044,20 @@
 /// `FIRST_TRANSACTION_TXG` 只管 mkfs 同一个进程里那条流（mkfs → 取号 → 暖机两次 → 第一个事务），不管任何池的第一个文件版本
 /// （2026-09-23 用户定案）。只做过 mkfs 的池重开一次可写挂载之后再写文件时，那一版已经推到 txg 4，这里接着写 txg 5。
 ///
+/// 这一版的八棵树从 `version_to_build_on` 的树 ID 水位起连号发（[`FileVersionTreeIdentifiers::issued_from_watermark`]），
+/// 新水位 = max(那一版的水位, 发出的最高号 + 1)（D8（核心索引结构） 已定项 8 ②）。那一版的水位就是根环里全部根记录的 max：
+/// 它是这个会话里的现行那一版，挂载时本实例的第一次发布取过环里的 max（`mount` 的 `tree_identifier_watermark_of_the_ring`），
+/// 之后每次发布只照抄或推高它，而环里后来写进去的根都是这个会话自己写的。mkfs 同一个进程里那条流上环里只有 mkfs 的根与暖机根，
+/// 都带 mkfs 种下的 11。
+///
 /// # Errors
-/// `version_to_build_on` 那一版已经有过文件版本 ⇒ `FirstFileVersionOnAVersionThatAlreadyHasAFile`（同一个文件再写一版走
+/// `version_to_build_on` 那一版的树表读不出、解不开 ⇒ `TreeTableOfTheVersionToBuildOnUnreadable`；
+/// 那一版的树表不是 0 条（已经有过文件版本）⇒ `FirstFileVersionOnAVersionThatAlreadyHasAFile`（同一个文件再写一版走
 /// `publish_overwrite`）。`previous_record_bytes` 解不出本池的记录，或它的 checkpoint_txg 与 `version_to_build_on` 那条根的不同
 /// （两者要说同一版，新根才恒落在它上面一格）⇒ `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`
 /// （m2-emptypool-nonempty-r1 云端攻方腿 Z3-A：拿 txg 4 的暖机根配 txg 2 的记录，新根盖在已有的根上、冷恢复读不到）。
-/// 两样都在任何写之前返回，一个字节都不写。其余同 `publish_version`。
+/// 那一版的水位离 `u64::MAX` 不到八个号 ⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
+/// 这几样都在任何写之前返回，一个字节都不写。其余同 `publish_version`。
 pub fn publish_first_file<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -1927,13 +2066,13 @@
     instance: InstanceGeneration,
     previous_record_bytes: &[u8],
 ) -> Result<TransactionOutput, PublishError> {
-    // 树 ID 水位分得开「树还没建起来」与「已经有过文件版本」：mkfs 种下 11，第一个文件版本把它推到 19，
-    // 零单元发布与写行那次发布都照抄上一版的（D5（快照 / 空间记账机制） 已定项 9）。
-    if version_to_build_on.tree_identifier_watermark != TREE_IDENTIFIER_WATERMARK_AT_MKFS {
+    // 「树还没建起来」看那一版的树表有几条（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步），不看水位：
+    // 回退到树表 0 条的一版时水位带着根环里的 max（D8（核心索引结构） 已定项 8 ②），那一版没有树、水位却早已不是 mkfs 的 11。
+    let tree_table_entries = tree_table_entry_count(&*pool.devices, version_to_build_on)
+        .map_err(|failure| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;
+    if tree_table_entries != 0 {
         return Err(
-            PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile {
-                tree_identifier_watermark: version_to_build_on.tree_identifier_watermark,
-            },
+            PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries },
         );
     }
     let previous_record = JournalRecord::parse(
@@ -1953,8 +2092,15 @@
     }
     let (_, previous_counter) =
         previous_record.expect("follows_directly 为真时上一条记录解得出（is_some_and）");
+    // 八棵树从那一版的水位（下一个可用号）起连号发：水位是 mkfs 的 11 时就是 D8（核心索引结构） 已定项 11 那几个常量，
+    // 回退到树表 0 条的一版之后水位带着环里的 max，号高于此前发过的每一个，不重发。
+    let (tree_identifiers, next_available_after_this_issue) =
+        FileVersionTreeIdentifiers::issued_from_watermark(
+            version_to_build_on.tree_identifier_watermark,
+        )
+        .map_err(PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees)?;
     let first_file_version_txg = CheckpointTxg(version_to_build_on.checkpoint_txg.0 + 1);
-    publish_version(
+    publish_version_of_trees(
         pool,
         allocator,
         PublishPlan {
@@ -1979,12 +2125,17 @@
             new_inode_records: &[],
             instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),
             tree_birth_txg: first_file_version_txg,
-            tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+            // D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)；
+            // 前一项就是那一版的水位（见文档注释），后一项恒更大。
+            tree_identifier_watermark: version_to_build_on
+                .tree_identifier_watermark
+                .max(next_available_after_this_issue),
             // F 照抄建在上面的那一版（D16（发布语义） 已定项 1：F 只升不降）。mkfs 同一个进程里那条流上它恒是 0，
             // 重开之后写行那次发布带的是恢复后生效的 F，这里接着带它。
             rollback_floor: version_to_build_on.rollback_floor,
         },
         None,
+        tree_identifiers,
     )
 }
 
@@ -2023,6 +2174,7 @@
             new_inode_records: &[],
             instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
             tree_birth_txg: previous.tree_birth_txg(),
+            // 这次一个树 ID 都不发：上一版的水位就是根环里全部根记录的 max（本会话每次发布只照抄或推高它，见 `publish_first_file`）。
             tree_identifier_watermark: previous.root.tree_identifier_watermark,
             rollback_floor: previous.root.rollback_floor,
         },
@@ -2144,18 +2296,50 @@
 /// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
 /// 落盘那几步里失败的，这次已记的写进写入口的失败账（增补 2 第 20b 行，`PoolWriter::writes_of_failed_publishes`）。
 ///
+/// 这一版那八棵树的号：接在上一版之后照抄上一版的（树 ID 只在第一个文件版本那次发）。没有上一版的只有第一个文件版本，
+/// 这里按 mkfs 那条流的水位 11 发（D8（核心索引结构） 已定项 11 登记的那几个常量）；从别的水位发号的第一个文件版本
+/// （回退到树表 0 条的一版之后）走 `publish_first_file`，它按要建在上面的那一版的水位发。
+///
 /// # Errors
 /// `InodeTreeWriteRefused`、`MoreNamedUnitsThanOneJournalRecordHolds`、`ContentExceedsDataUnit`、
 /// `AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、`MappingEntriesExceedOneNode`、
 /// 释放判定路径的四种错、`PlacementRefused`、块设备错。
+///
+/// # Panics
+/// 没有上一版、而计划写的新水位不是 mkfs 那条流第一次发布之后的 19：调用方把一个水位早已推高的池当成了 mkfs 那条流，
+/// 按 11 发号会重发已经发过的号（D8（核心索引结构） 已定项 8 ②）。
 pub fn publish_version<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
     plan: PublishPlan<'_>,
     previous: Option<&TransactionOutput>,
 ) -> Result<TransactionOutput, PublishError> {
+    let trees = match previous {
+        Some(previous_version) => previous_version.tree_identifiers,
+        None => {
+            assert_eq!(
+                plan.tree_identifier_watermark, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+                "没有上一版的发布在这里按 mkfs 那条流的水位 11 发号：计划写的新水位必须就是那条流的 19，别的水位走 publish_first_file"
+            );
+            FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
+                .expect("11 加八个号装得下")
+                .0
+        }
+    };
+    publish_version_of_trees(pool, allocator, plan, previous, trees)
+}
+
+/// `publish_version` 的本体，这一版那八棵树的号由调用方给：`publish_first_file` 给它这次从水位发出来的，
+/// `publish_version` 给上一版的（或 mkfs 那条流的）。
+fn publish_version_of_trees<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    plan: PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    trees: FileVersionTreeIdentifiers,
+) -> Result<TransactionOutput, PublishError> {
     // 先算这次之后 inode 树是什么样、这次重写哪些角色：只读，条款没写的那几格在这里交回，盘上逐字节不变。
-    let resolved = plan.resolve(previous)?;
+    let resolved = plan.resolve(previous, &trees)?;
     let rewritten = resolved.rewritten_roles.clone();
     // 点名项一条记录装 67 个（D23（journal 的角色与格式） 已定项 12 / 已定项 17）：这次重写的角色多于它就要写成多条记录，
     // 而多条记录时共享的提交内生块在哪一条里点名还没定 ⇒ 在动分配器之前拒掉。
@@ -2190,7 +2374,7 @@
     publish_admission(allocator, &rewritten, resolved.inode_tree.containers.len())?;
     // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let outcome = publish_admitted(pool, allocator, &plan, previous, &resolved, &release);
+    let outcome = publish_admitted(pool, allocator, &plan, previous, &resolved, &release, trees);
     if outcome.is_err() {
         *allocator = allocator_before_this_publish;
     }
@@ -2353,6 +2537,8 @@
 struct FileVersionCheckpoint<'checkpoint> {
     txg: CheckpointTxg,
     write_order: WriteOrder,
+    /// 这一版那八棵树各自的号（单元头与指针头部的出生树按它写）。
+    trees: FileVersionTreeIdentifiers,
     filesystem_identifier: &'checkpoint [u8; 16],
     /// 位置条目按设备身份升序（I-2.5）。
     device_identities: &'checkpoint [DeviceIdentity],
@@ -2379,7 +2565,7 @@
     let filesystem_identifier = checkpoint.filesystem_identifier;
     // t1 数据单元（字节表二）。
     let data_identity = DataUnitIdentity {
-        tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+        tree: checkpoint.trees.extent,
         object: FIRST_INODE_NUMBER,
         object_birth: file.inode_object_birth,
         anchor_offset: 0,
@@ -2393,7 +2579,7 @@
     );
     let data_pointer = DataPointer {
         head: PointerHead {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+            birth_tree: checkpoint.trees.extent,
             birth_txg: txg,
         },
         locations: location_entries(checkpoint.device_identities, slots.data, &data_unit),
@@ -2406,9 +2592,9 @@
     let key_order_mismatches = count_key_order_mismatches(&[extent_key]);
 
     // t2 extent 树根兼叶（字节表四·二）。
-    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
+    let extent_sequence = sequences.next(checkpoint.trees.extent, txg, instance);
     let extent_unit = build_index_node(
-        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+        checkpoint.trees.extent,
         0,
         usize::try_from(EXTENT_KEY_BYTES).expect("24"),
         &extent_key,
@@ -2470,7 +2656,7 @@
 ) -> InodeTreeUnits {
     let txg = checkpoint.txg;
     let instance = checkpoint.write_order.instance;
-    let inode_tree_identifier = TreeIdentifier(TREE_IDENTIFIER_INODE);
+    let inode_tree_identifier = checkpoint.trees.inode;
     let mut leaf_containers = Vec::with_capacity(inode_tree.containers.len());
     for (position, contents) in inode_tree.containers.iter().enumerate() {
         let index = InodeLeafContainerIndexInTree::of_position(position);
@@ -2584,7 +2770,7 @@
         data_pointer: carried.data_pointer,
         extent_unit: carried.unit(TransactionUnit::ExtentRoot).bytes.clone(),
         extent_sequence: carried
-            .tree_root_pointer(TREE_IDENTIFIER_EXTENT)
+            .tree_root_pointer(carried.tree_identifiers.extent.0)
             .birth_sequence,
         key_order_mismatches: 0,
     }
@@ -2602,6 +2788,7 @@
     previous: Option<&TransactionOutput>,
     resolved: &ResolvedPublish,
     release: &[Placement],
+    trees: FileVersionTreeIdentifiers,
 ) -> Result<TransactionOutput, PublishError> {
     let rewritten = &resolved.rewritten_roles;
     let txg = plan.txg;
@@ -2630,6 +2817,7 @@
     let checkpoint = FileVersionCheckpoint {
         txg,
         write_order,
+        trees,
         filesystem_identifier,
         device_identities: &device_identities,
     };
@@ -2720,16 +2908,12 @@
     };
 
     // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
-    let allocation_sequence = sequences.next(
-        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
-        txg,
-        instance,
-    );
+    let allocation_sequence = sequences.next(trees.allocation_records, txg, instance);
     let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
     allocation_records.sort_by_key(AllocationRecord::sort_key);
     let allocation_unit = build_allocation_record_node(
         allocator,
-        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
+        trees.allocation_records,
         txg,
         filesystem_identifier,
         instance,
@@ -2738,9 +2922,9 @@
 
     // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
     // 全部来自分配器在分配那一刻增量维护的数，不扫盘（`.claude/rules/fs-design.md` 第一格）。seq 一律 1（D8（核心索引结构） 已定项 10）。
-    let pool_wide = |statistic: u16, tree: u64, value: u64| AccountingEntry {
+    let pool_wide = |statistic: u16, tree: TreeIdentifier, value: u64| AccountingEntry {
         statistic,
-        tree: TreeIdentifier(tree),
+        tree,
         device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
         generation: txg,
         value,
@@ -2759,13 +2943,17 @@
     let mut accounting_entries = vec![
         pool_wide(
             STATISTIC_INODE_WATERMARK,
-            TREE_IDENTIFIER_INODE,
+            trees.inode,
             inode_number_watermark,
         ),
-        pool_wide(STATISTIC_PENDING_DELETE_BYTES, TREE_IDENTIFIER_NONE, 0),
+        pool_wide(
+            STATISTIC_PENDING_DELETE_BYTES,
+            TreeIdentifier(TREE_IDENTIFIER_NONE),
+            0,
+        ),
         pool_wide(
             STATISTIC_COMMITTED_RESERVATION_BYTES,
-            TREE_IDENTIFIER_NONE,
+            TreeIdentifier(TREE_IDENTIFIER_NONE),
             0,
         ),
     ];
@@ -2808,10 +2996,9 @@
         "准入按 accounting_entry_count 判过装不装得下：这里装的行数要与它相等，改了行的构成要一起改那两个常量"
     );
     accounting_entries.sort_by_key(AccountingEntry::sort_key);
-    let accounting_sequence =
-        sequences.next(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), txg, instance);
+    let accounting_sequence = sequences.next(trees.accounting, txg, instance);
     let accounting_unit = build_index_node(
-        TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
+        trees.accounting,
         0,
         usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
         &accounting_entries[0].key_bytes(),
@@ -2827,47 +3014,49 @@
             .collect::<Vec<_>>(),
     );
 
-    let node_pointer =
-        |tree: u64, identity: TransactionUnit, unit: &[u8], sequence: BirthSequence| NodePointer {
-            head: PointerHead {
-                birth_tree: TreeIdentifier(tree),
-                birth_txg: txg,
-            },
-            locations: pool.location_entries(slot_of(identity), unit),
-            instance,
-            birth_sequence: sequence,
-        };
+    let node_pointer = |tree: TreeIdentifier,
+                        identity: TransactionUnit,
+                        unit: &[u8],
+                        sequence: BirthSequence| NodePointer {
+        head: PointerHead {
+            birth_tree: tree,
+            birth_txg: txg,
+        },
+        locations: pool.location_entries(slot_of(identity), unit),
+        instance,
+        birth_sequence: sequence,
+    };
     // 文件内容角色的指针：重写的按这次的落点算，照抄的取上一版。
     let extent_pointer = match carried_file {
         None => node_pointer(
-            TREE_IDENTIFIER_EXTENT,
+            trees.extent,
             TransactionUnit::ExtentRoot,
             &extent_unit,
             extent_sequence,
         ),
-        Some(carried) => carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT),
+        Some(carried) => carried.tree_root_pointer(trees.extent.0),
     };
     // inode 树根的指针：这次重写了就按这次的落点算，一片叶都没重写就取上一版树表里那一条。
     let inode_root_pointer = match (&inode_tree_units.rewritten_root, previous) {
         (Some(root), _) => node_pointer(
-            TREE_IDENTIFIER_INODE,
+            trees.inode,
             TransactionUnit::InodeRoot,
             &root.bytes,
             root.birth_sequence,
         ),
-        (None, Some(carried)) => carried.tree_root_pointer(TREE_IDENTIFIER_INODE),
+        (None, Some(carried)) => carried.tree_root_pointer(trees.inode.0),
         (None, None) => {
             unreachable!("没有上一版的发布（第一个文件版本）恒要写 inode 记录 ⇒ 恒重写 inode 树")
         }
     };
     let allocation_pointer = node_pointer(
-        TREE_IDENTIFIER_ALLOCATION_RECORDS,
+        trees.allocation_records,
         TransactionUnit::AllocationTree,
         &allocation_unit,
         allocation_sequence,
     );
     let accounting_pointer = node_pointer(
-        TREE_IDENTIFIER_ACCOUNTING,
+        trees.accounting,
         TransactionUnit::AccountingTree,
         &accounting_unit,
         accounting_sequence,
@@ -2926,13 +3115,9 @@
         "准入按 mapping_entry_count 判过装不装得下：这里装的条目数要与它相等，改了进映射的角色要一起改那个常量"
     );
     mapping_entries.sort_by_key(|(key, _)| mapping_key_sort_key(key));
-    let mapping_sequence = sequences.next(
-        TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
-        txg,
-        instance,
-    );
+    let mapping_sequence = sequences.next(trees.central_mapping, txg, instance);
     let mapping_unit = build_index_node(
-        TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
+        trees.central_mapping,
         0,
         usize::try_from(MAPPING_KEY_BYTES).expect("27"),
         &mapping_entries[0].0,
@@ -2948,7 +3133,7 @@
             .collect::<Vec<_>>(),
     );
     let mapping_pointer = node_pointer(
-        TREE_IDENTIFIER_CENTRAL_MAPPING,
+        trees.central_mapping,
         TransactionUnit::MappingTree,
         &mapping_unit,
         mapping_sequence,
@@ -2957,9 +3142,9 @@
     // t8 树表单元：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
     // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己、extent 树写它服务的头，其余 0。
     let table_entry =
-        |kind: u16, tree: u64, root: NodePointer, head_identifier: u64| TreeTableEntry {
+        |kind: u16, tree: TreeIdentifier, root: NodePointer, head_identifier: u64| TreeTableEntry {
             kind,
-            tree: TreeIdentifier(tree),
+            tree,
             root,
             birth_txg: plan.tree_birth_txg,
             head_identifier,
@@ -2967,54 +3152,60 @@
     let tree_table_entries = vec![
         table_entry(
             TREE_KIND_EXTENT,
-            TREE_IDENTIFIER_EXTENT,
+            trees.extent,
             extent_pointer,
-            TREE_IDENTIFIER_INODE,
+            trees.inode.0,
         ),
         table_entry(
             TREE_KIND_INODE,
-            TREE_IDENTIFIER_INODE,
+            trees.inode,
             inode_root_pointer,
-            TREE_IDENTIFIER_INODE,
+            trees.inode.0,
         ),
         table_entry(
             TREE_KIND_ALLOCATION,
-            TREE_IDENTIFIER_ALLOCATION_RECORDS,
+            trees.allocation_records,
             allocation_pointer,
             0,
         ),
         table_entry(
             TREE_KIND_ACCOUNTING,
-            TREE_IDENTIFIER_ACCOUNTING,
+            trees.accounting,
             accounting_pointer,
             0,
         ),
         table_entry(
             TREE_KIND_LIVELIST,
-            TREE_IDENTIFIER_LIVELIST,
+            trees.livelist,
             NodePointer::empty_root(),
             0,
         ),
         table_entry(
             TREE_KIND_SPARSE_SIDE_TABLE,
-            TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
+            trees.sparse_side_table,
             NodePointer::empty_root(),
             0,
         ),
         table_entry(
             TREE_KIND_DEADLIST,
-            TREE_IDENTIFIER_DEADLIST,
+            trees.deadlist,
             NodePointer::empty_root(),
             0,
         ),
     ];
+    assert!(
+        tree_table_entries
+            .windows(2)
+            .all(|pair| pair[0].tree < pair[1].tree),
+        "树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：八个号连号发、中央映射树不进树表，剩下七条仍按发号次序升序"
+    );
     let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
     let tree_table_unit = build_index_node(
         TreeIdentifier(TREE_IDENTIFIER_NONE),
         0,
         TREE_TABLE_KEY_WIDTH,
-        &TREE_IDENTIFIER_EXTENT.to_le_bytes(),
-        &TREE_IDENTIFIER_DEADLIST.to_le_bytes(),
+        &trees.extent.0.to_le_bytes(),
+        &trees.deadlist.0.to_le_bytes(),
         txg,
         filesystem_identifier,
         instance,
@@ -3026,7 +3217,7 @@
             .collect::<Vec<_>>(),
     );
     let tree_table_pointer = node_pointer(
-        TREE_IDENTIFIER_NONE,
+        TreeIdentifier(TREE_IDENTIFIER_NONE),
         TransactionUnit::TreeTable,
         &tree_table_unit,
         tree_table_sequence,
@@ -3147,7 +3338,7 @@
         .map(|unit| NamedUnit {
             locations: pool.location_entries(unit.slot, &unit.bytes),
             unit_class: unit.identity.unit_class(),
-            birth_tree: unit.identity.tree(),
+            birth_tree: unit.identity.tree(&trees),
             birth_txg: txg,
             key_tail: key_tail_of(unit.identity),
         })
@@ -3176,7 +3367,7 @@
         rollback_floor: plan.rollback_floor,
         instance_table: instance_table_pointer,
         mapping_root: mapping_pointer,
-        // 带文件的一版的分配记录树住树表条目（树 ID 13，D8（核心索引结构） 已定项 8）：根记录这一项恒全零，
+        // 带文件的一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8）：根记录这一项恒全零，
         // 两处都写就成了同一个量的两份手抄。`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住。
         allocation_record_tree_root: NodePointer::empty_root(),
     };
@@ -3224,6 +3415,7 @@
         allocation_records,
         accounting_entries,
         tree_table_entries,
+        tree_identifiers: trees,
         inode_record,
         inode_leaf_containers: inode_tree_units
             .leaf_containers
@@ -3324,6 +3516,11 @@
                 instance,
                 transaction: FIRST_TRANSACTION_NUMBER,
             },
+            trees: FileVersionTreeIdentifiers::issued_from_watermark(
+                TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+            )
+            .expect("11 加八个号装得下")
+            .0,
             filesystem_identifier: &filesystem_identifier,
             device_identities: &device_identities,
         };
@@ -3462,8 +3659,11 @@
             "第二片叶容器的步号：字节表七只登记了一片那一档的 t3"
         );
         assert_eq!(TransactionUnit::Data.placement(), PlacementRule::UserData);
+        let (trees_issued_from_the_make_filesystem_watermark, _) =
+            FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
+                .expect("11 加八个号装得下");
         assert_eq!(
-            TransactionUnit::TreeTable.tree(),
+            TransactionUnit::TreeTable.tree(&trees_issued_from_the_make_filesystem_watermark),
             TreeIdentifier(0),
             "树表单元不属于任何一棵树"
         );
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -1973,6 +1973,12 @@
         PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. } => {
             "FirstFileVersionOnAVersionThatAlreadyHasAFile"
         }
+        PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. } => {
+            "TreeTableOfTheVersionToBuildOnUnreadable"
+        }
+        PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_) => {
+            "TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees"
+        }
         PublishError::BlockDevice(cause) => {
             return format!(
                 "PublishError::BlockDevice({})",
@@ -2056,9 +2062,6 @@
         MountError::RollbackTargetNotACandidate { exclusion, .. } => {
             return format!("MountError::RollbackTargetNotACandidate({exclusion:?})")
         }
-        MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion { .. } => {
-            "RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion"
-        }
         MountError::RollbackFloorAboveCeiling { .. } => "RollbackFloorAboveCeiling",
         MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
             "VersionWithoutFileNotWrittenByMakeFilesystem"
@@ -2377,6 +2380,8 @@
         instance: session.instance,
         back_chain: back_chain_of(&previous.record_bytes),
         rollback_floor: previous.root.rollback_floor,
+        // 会话里接着现行那一版：它的水位已经是根环里的 max（挂载时本实例第一次发布取过，之后只照抄或推高）。
+        tree_identifier_watermark: previous.root.tree_identifier_watermark,
     };
     let previous_root = previous.root;
     let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -186,6 +186,9 @@
         | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
         | PublishError::ReleaseSpanMismatch { .. }
         | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
+        // 第一个文件版本读不出那一版的树表、水位离 u64::MAX 不到八个号：健康的内存盘上都不该出现。
+        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
+        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
         | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,
     }
 }
@@ -207,7 +210,7 @@
 
 /// 回退目标不在候选集里的那一条说的是哪条理由：按字段一对一映射，不看给人看的文字（增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
 /// 「树表 0 条」不在这里：候选集只有这三条（D23（journal 的角色与格式） 已定项 14），目标那一版树表 0 条不是排除项
-/// （C493（回退候选集条文与实现说反话） 还清）；环里还留着带文件版本的根时那一格是 `MountError` 单独一个成员。
+/// （C493（回退候选集条文与实现说反话） 还清）；环里还留着带文件版本的根时照常回退（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步）。
 #[must_use]
 pub fn refusal_reason_of_rollback_candidate_exclusion(
     exclusion: RollbackCandidateExclusion,
@@ -250,12 +253,6 @@
         MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported { .. } => {
             explained(ModelRefusalReason::InstanceTableOnePageWall)
         }
-        // 回退到树表 0 条的根、而环里还有带文件的根：零故障走得到，模型照代码今天的读法划进必须拒。
-        MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion { .. } => {
-            explained(
-                ModelRefusalReason::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion,
-            )
-        }
         // 树表 0 条、而实例表已经不是 mkfs 那一片：零故障走得到（写过行的那一版上再挂载一次），模型照代码今天的读法划进必须拒。
         MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
             explained(ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem)
@@ -285,7 +282,6 @@
         | MountError::Acquisition(_)
         | MountError::Publish(_)
         | MountError::RollbackTargetNotACandidate { .. }
-        | MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion { .. }
         | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
         | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
         | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -284,10 +284,6 @@
     RollbackTargetBelowEffectiveFloor,
     /// 回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）。
     RollbackTargetOnAbandonedTimeline,
-    /// 回退目标那一版树表 0 条、而根环里还留着带文件版本的根：回退之后只能再发一次「第一个文件版本」，
-    /// 那一次重新建树、重新发对象出生代，树表条目的诞生 txg 与 inode 1 的对象出生代都与环里那些旧根记的不同
-    /// （I-9.14、I-9.10）。重新建树的诞生代怎么算没有条款（设计空白，交主 agent）。
-    RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion,
     /// 要建立新实例的那一版树表 0 条、而它的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）：
     /// 被换下的那一片记在哪没有条款——树表 0 条 ⇒ 这一版没有分配记录树，那次释放只住内存，重开之后账取不回来，
     /// 而根环里更旧的候选根还指着它。第一版不支持（设计空白，交主 agent）。
@@ -316,9 +312,6 @@
             ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
             ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
             ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
-            ModelRefusalReason::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion => {
-                "回退到树表 0 条的根、而环里还有带文件的根（条款没写重新建树的诞生代）"
-            }
             ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem => {
                 "树表 0 条、而实例表已经不是 mkfs 那一片（条款没写被换下的那一片记在哪）"
             }
@@ -341,7 +334,6 @@
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
-            | ModelRefusalReason::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
             | ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported => false,
@@ -1099,14 +1091,9 @@
         if Self::is_abandoned_by(target_root, &self.newest_root().instance_table_rows) {
             reasons.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);
         }
-        // 候选集只有上面那三条（D23（journal 的角色与格式） 已定项 14）：目标那一版树表 0 条不是排除项。
-        // 只有环里还留着带文件版本的根时拒绝：回退之后再发一次「第一个文件版本」会重新建树、重新发对象出生代，
-        // 与环里那些旧根记的对不上（I-9.14、I-9.10），重新建树的诞生代怎么算没有条款。
-        if target_root.file.is_none() && self.ring.values().any(|root| root.file.is_some()) {
-            reasons.insert(
-                ModelRefusalReason::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion,
-            );
-        }
+        // 候选集只有上面那三条（D23（journal 的角色与格式） 已定项 14）：目标那一版树表 0 条不是排除项，
+        // 环里还留着带文件版本的根时也照常回退（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：回退那一版带环里的树 ID 水位 max，
+        // 再发第一个文件版本从它往上发号；I-9.14 只比同一条时间线上的根）。
         if !reasons.is_empty() {
             return refused(reasons);
         }
@@ -1414,7 +1401,6 @@
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
-            | ModelRefusalReason::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
             | ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported => false,
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -1866,6 +1866,72 @@
     }
 }
 
+/// 第一个事务写出的中央映射树根（bump 次序：分配记录树、记账树、映射树、树表）。
+const MAPPING_ROOT: u64 = 50247;
+/// 根记录里中央映射树根指针的出生树：指针在根记录偏移 256（D22（单元原子性怎么合成） 已定项 7），出生树在指针偏移 34
+/// （D19（块指针的结构与宽度预算） 已定项 11）。
+const ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET: usize = 256 + 34;
+/// 这个池发过、又不是中央映射树的一个号（livelist 树 16），低于水位 19。
+const ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK: u64 = 16;
+
+/// 中央映射树不进树表：I-1.3（块头树 ID 一致） 对它的根，「实际引用它的树」按根记录里那条根指针的出生树取
+/// （回退到树表 0 条的一版之后再发第一个文件版本，八棵树从水位重新发号，映射树不再恒是 15——C511（回退到无文件那一版之后诞生代怎么接））。
+/// 两份坏镜像各只改一边、都只有 I-1.3 红：映射树根头里的树 ID 改成 16（指针仍说 15），与根记录里那条指针的出生树改成 16
+/// （头仍说 15，重封根槽的自证校验和）。后一份钉的是「按指针判」这个读法：checker 退回写死 15 时它不红。
+/// 取 16 不取一个大数：16 是这个池发过的号（livelist 树）、低于水位 19，改成水位之上的号会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID）
+/// 跟着红（它扫码 2 单元头），判别力就算不到 I-1.3 一条头上。
+#[test]
+fn a_central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant(
+) {
+    let clean = build_pool("known-bad-central-mapping-tree").memory_pool();
+    let violated_on_the_clean_image = violated_invariants(&check_pool_image(&clean));
+    assert!(
+        violated_on_the_clean_image.is_empty(),
+        "干净镜像上一条违例都没有：{violated_on_the_clean_image:?}"
+    );
+    let header_says_another_tree: Mutation = Box::new(|image: &mut MemoryPool| {
+        mutate_unit(
+            image,
+            MAPPING_ROOT,
+            |bytes| set_u64(bytes, 42, ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK),
+            true,
+        );
+    });
+    let root_pointer_says_another_tree: Mutation = Box::new(|image: &mut MemoryPool| {
+        let (device, offset) = FIRST_TRANSACTION_ROOT_SLOT;
+        let mut root_slot = read(image, device, offset, 512);
+        assert_eq!(
+            get_u64(&root_slot, ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET),
+            15,
+            "第一个事务那条根的映射树根指针说出生树 15"
+        );
+        set_u64(
+            &mut root_slot,
+            ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET,
+            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
+        );
+        let digest = wide_checksum_with_field_zeroed(&root_slot, 512, 138);
+        root_slot[138..170].copy_from_slice(&digest);
+        write(image, device, offset, &root_slot);
+    });
+    for (which_side, mutation) in [
+        ("映射树根头里的树 ID", header_says_another_tree),
+        (
+            "根记录里映射树根指针的出生树",
+            root_pointer_says_another_tree,
+        ),
+    ] {
+        let mut image = clean.clone();
+        mutation(&mut image);
+        let verdicts = check_pool_image(&image);
+        assert_eq!(
+            violated_invariants(&verdicts),
+            ["I-1.3"],
+            "只改了{which_side}：判红的该只有 I-1.3：{verdicts:?}"
+        );
+    }
+}
+
 /// 记账树叶里 (统计量, 设备) 那一行的值改成 value（偏移口径同 `adjust_accounting`）。
 fn set_accounting(bytes: &mut [u8], statistic: u16, device: u32, value: u64) {
     let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
--- a/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
@@ -543,7 +543,10 @@
             continue;
         }
         let view = index_node_view(&unit.bytes).expect("码 2 节点");
-        assert_eq!(view.tree_identifier, unit.identity.tree().0);
+        assert_eq!(
+            view.tree_identifier,
+            unit.identity.tree(&pool.output.tree_identifiers).0
+        );
         let schema = match unit.identity {
             TransactionUnit::MappingTree => KEY_SCHEMA_MAPPING,
             TransactionUnit::TreeTable => KEY_SCHEMA_TREE_TABLE,
@@ -553,7 +556,9 @@
             | TransactionUnit::AccountingTree => {
                 let kind = tree_table_entries
                     .iter()
-                    .find(|entry| entry.tree.0 == unit.identity.tree().0)
+                    .find(|entry| {
+                        entry.tree.0 == unit.identity.tree(&pool.output.tree_identifiers).0
+                    })
                     .expect("在树表里")
                     .kind;
                 key_schema_for_tree_kind(kind).expect("有节点的树都有 key 形态")
--- a/crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs
@@ -5,16 +5,17 @@
 mod common;
 
 use common::{
-    build_pool, disk_snapshot, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
-    IMAGE_BYTES,
+    build_pool, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
 };
 use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::index_node_view;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, TreeIdentifier,
 };
-use singlefs_core::allocator::unit_area_slots_of_device;
+use singlefs_core::allocator::{unit_area_slots_of_device, PoolAllocator};
 use singlefs_core::block_device::{BlockDevice, WriteDurability};
+use singlefs_core::journal::back_chain_of;
 use singlefs_core::mount::{
     mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackCandidateExclusion,
     RollbackTarget, ShadowLedger,
@@ -22,15 +23,19 @@
 use singlefs_core::pointer::LocationEntry;
 use singlefs_core::records::TREE_KIND_ALLOCATION;
 use singlefs_core::recovery::{
-    allocation_records_under_root, choose_root, choose_system_configuration, readable_roots,
-    recover, replay_journal, scan_journal, JournalPolicy, RecoveryFailure, RecoveryOutcome,
+    allocation_records_under_root, choose_root, choose_system_configuration,
+    highest_tree_identifier_watermark_in_the_ring, readable_roots, recover, replay_journal,
+    scan_journal, tree_table_has_no_entries, JournalPolicy, RecoveryFailure, RecoveryOutcome,
 };
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{
-    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
+    publish_first_file, publish_overwrite, publish_without_units, FirstFile, PoolVersion,
+    PoolWriter, TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput,
+    ZeroUnitPublishPlan,
 };
-use singlefs_format::UNIT_AREA_START_SLOT;
+use singlefs_format::{ROOT_RING_REGIONS, UNIT_AREA_START_SLOT};
 use singlefs_harness::bad_disk_input::move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain;
+use singlefs_harness::crash::MemoryPool;
 use singlefs_harness::fault_injection::{
     NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget,
 };
@@ -93,45 +98,452 @@
     pool
 }
 
-/// 回退到树表 0 条的根（第一个事务里 txg 2 的暖机根）：候选集只有「在根环里」「txg ≥ F_生效」「按实例表判仍然有效」三条
-/// （D23（journal 的角色与格式） 已定项 14 / D16（发布语义） 已定项 1），「树表 0 条」**不在其内**——报的不再是候选排除
-/// （C493（回退候选集条文与实现说反话） 还清）。这个池里环里还留着带文件版本的根 (1, 3) ⇒ 返回
-/// `RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`：回退之后只能再发一次「第一个文件版本」，
-/// 那一次重新建树、重新发对象出生代，树表条目的诞生 txg 与 inode 1 的对象出生代都与 (1, 3) 记的不同
-/// （I-9.14、I-9.10；2026-09-23 崩溃注入快档三条新发现都是它）——重新建树的诞生代怎么算没有条款。
-/// 在任何写之前拒绝，盘上逐字节不变。
-/// 环里没有带文件的根时回退照常做，由 `second_transaction_step_three_formatted_pool.rs` 的
-/// `rolling_back_to_a_warm_up_root_before_any_file_version_rewrites_only_the_instance_table` 钉住。
-#[test]
-fn rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_is_refused_before_any_write(
+/// 根环全部自证过的根带的树 ID 水位取 max（D8（核心索引结构） 已定项 8 ② 的那个量，也是 I-7.8（根记录树 ID 水位不低于全池最大树 ID）
+/// 取 max 的范围）：用例拿它核「回退之前环里的最大水位」。
+fn highest_watermark_among_ring_roots(image: &MemoryPool) -> u64 {
+    let parameters = parameters();
+    readable_roots(
+        image,
+        &parameters.region_devices,
+        &parameters.geometry,
+        &parameters.filesystem_identifier,
+    )
+    .iter()
+    .map(|root| root.tree_identifier_watermark)
+    .max()
+    .expect("环里至少有一条自证过的根")
+}
+
+/// 池级 checker 一条违例都没有；交回全部判定，调用方再点名要真被判过（不是「不适用」）的那几条。
+fn verdicts_without_any_violation(
+    image: &MemoryPool,
+    step: &str,
+) -> Vec<(&'static str, InvariantVerdict)> {
+    let verdicts = check_pool_image(image);
+    let violated: Vec<(&str, &InvariantVerdict)> = verdicts
+        .iter()
+        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+        .map(|(invariant, verdict)| (*invariant, verdict))
+        .collect();
+    assert!(
+        violated.is_empty(),
+        "{step}：池级 checker 一条违例都没有：{violated:?}"
+    );
+    verdicts
+}
+
+fn assert_judged_and_holding(
+    verdicts: &[(&'static str, InvariantVerdict)],
+    invariants: &[&str],
+    step: &str,
 ) {
-    let mut pool = build_pool("step-four-rollback-to-warm-up-root");
-    let warm_up_root = RollbackTarget {
+    for invariant in invariants {
+        assert!(
+            verdicts
+                .iter()
+                .any(|(name, verdict)| name == invariant && *verdict == InvariantVerdict::Holds),
+            "{step}：{invariant} 真被判过且成立：{:?}",
+            verdicts.iter().find(|(name, _)| name == invariant)
+        );
+    }
+}
+
+fn warm_up_root() -> RollbackTarget {
+    RollbackTarget {
         instance: InstanceGeneration(1),
         checkpoint_txg: CheckpointTxg(2),
+    }
+}
+
+/// 回退之后那个会话的现行那一版（树表 0 条）：根、记录、记录的字节。
+fn current_version_without_file(current: &PoolVersion) -> VersionWithoutFilePublishOutput {
+    let PoolVersion::WithoutFile(version) = current else {
+        panic!("回退到树表 0 条的暖机根：现行那一版仍是「没有文件版本」的一版")
     };
-    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
+    version.clone()
+}
+
+/// 在回退之后那个会话里接着现行那一版（树表 0 条）发第一个文件版本，回来的那一版装回 pool。
+fn publish_first_file_after_the_rollback(
+    pool: &mut BuiltPool,
+    allocator: &mut PoolAllocator,
+    current: &PoolVersion,
+    instance: InstanceGeneration,
+    content: &[u8],
+) -> TransactionOutput {
+    let version = current_version_without_file(current);
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let output = publish_first_file(
+        &mut writer,
+        allocator,
+        &version.root,
+        FirstFile {
+            content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
+        },
+        instance,
+        &version.record_bytes,
+    )
+    .expect("回退到树表 0 条的一版之后再发第一个文件版本");
+    pool.output = output.clone();
+    pool.allocator = allocator.clone();
+    output
+}
+
+/// 新发出来的八个号都不低于回退那一版带过来的水位、都高于回退之前发过的每一个号；八个号互不相同；
+/// 树表七条与中央映射树根的头里写的就是这几个号；新水位 = 最大那个号 + 1（D8（核心索引结构） 已定项 8 ②）。
+fn assert_fresh_tree_identifiers(
+    output: &TransactionOutput,
+    watermark_carried_by_the_rollback: u64,
+    highest_tree_identifier_before_the_rollback: TreeIdentifier,
+) {
+    let issued = output.tree_identifiers.in_issue_order();
+    for tree in issued {
+        assert!(
+            tree.0 >= watermark_carried_by_the_rollback
+                && tree > highest_tree_identifier_before_the_rollback,
+            "新发的号 {tree:?} 不低于回退带过来的水位 {watermark_carried_by_the_rollback}、高于此前最大的 {highest_tree_identifier_before_the_rollback:?}"
+        );
+    }
+    assert_eq!(
+        issued.iter().collect::<BTreeSet<_>>().len(),
+        8,
+        "八个号互不相同"
+    );
+    assert_eq!(
+        output.root.tree_identifier_watermark,
+        output.tree_identifiers.highest().0 + 1,
+        "新水位 = 这次发出的最高号 + 1（它高于环里任何一条根带的）"
+    );
+    let tree_table_trees: BTreeSet<TreeIdentifier> = output
+        .tree_table_entries
+        .iter()
+        .map(|entry| entry.tree)
+        .collect();
+    let issued_without_the_central_mapping: BTreeSet<TreeIdentifier> = issued
+        .into_iter()
+        .filter(|tree| *tree != output.tree_identifiers.central_mapping)
+        .collect();
+    assert_eq!(
+        tree_table_trees, issued_without_the_central_mapping,
+        "树表七条就是这次发的号（中央映射树不进树表）"
+    );
+    let mapping_node = index_node_view(&output.unit(TransactionUnit::MappingTree).bytes)
+        .expect("中央映射树根是码 2 节点");
+    assert_eq!(
+        (
+            mapping_node.tree_identifier,
+            output.root.mapping_root.head.birth_tree
+        ),
+        (
+            output.tree_identifiers.central_mapping.0,
+            output.tree_identifiers.central_mapping
+        ),
+        "中央映射树根的头与根记录里它那条指针的出生树都是这次发的号"
+    );
+}
+
+/// C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：环里还留着带文件版本的根 (1, 3)（树 11..18、水位 19）时回退到树表 0 条的
+/// 暖机根 (1, 2)——回退照常做，不再在写之前拒绝；回退行那次发布的根带回退之前根环里的水位 max 19（D8（核心索引结构） 已定项 8 ②），
+/// 不带暖机根自己的 11。接着在同一个会话里再发第一个文件版本：八棵树从 19 起连号发、新水位 27，高于此前发过的每一个号；
+/// 再覆盖写一版，让 I-9.14（树表条目的诞生 txg 跨根不变） 在同一条时间线上有两个树表可比。每一步池级 checker 一条违例都没有，
+/// 最后 I-7.8（根记录树 ID 水位不低于全池最大树 ID）、I-9.14、I-9.10（对象出生代三处一致） 都真被判过且成立；冷启动读回覆盖写的内容。
+#[test]
+fn rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_carries_the_ring_watermark_and_the_next_first_file_issues_fresh_tree_identifiers(
+) {
+    let mut pool = build_pool("step-four-rollback-to-warm-up-root-then-first-file");
+    let highest_tree_identifier_before_the_rollback = pool.output.tree_identifiers.highest();
+    let ring_watermark_before_the_rollback =
+        highest_watermark_among_ring_roots(&pool.memory_pool());
+    assert_eq!(
+        (
+            highest_tree_identifier_before_the_rollback,
+            ring_watermark_before_the_rollback
+        ),
+        (TreeIdentifier(18), 19),
+        "回退之前：第一个事务发了 11..18，(1, 3) 带水位 19"
+    );
+    let parameters_of_the_pool = parameters();
+    let warm_up_root_record = readable_roots(
+        &pool.memory_pool(),
+        &parameters_of_the_pool.region_devices,
+        &parameters_of_the_pool.geometry,
+        &parameters_of_the_pool.filesystem_identifier,
+    )
+    .into_iter()
+    .find(|root| {
+        (root.instance, root.checkpoint_txg)
+            == (warm_up_root().instance, warm_up_root().checkpoint_txg)
+    })
+    .expect("暖机根 (1, 2) 还在环里");
+    assert_eq!(
+        warm_up_root_record.tree_identifier_watermark, 11,
+        "回退到的那一版自己带的是 mkfs 种下的 11：沿它带就会低于环里的 max"
+    );
+
     let mut devices = pool.reopen_recorded();
-    let refused = mount_rollback(&parameters(), &mut devices, warm_up_root, ShadowLedger::On);
+    let rolled_back = mount_rollback(
+        &parameters(),
+        &mut devices,
+        warm_up_root(),
+        ShadowLedger::On,
+    )
+    .expect("环里还留着带文件版本的根时回退照常做");
     pool.devices = Some(devices);
+    let PoolVersion::WithoutFile(row) = &rolled_back.output.row_publish else {
+        panic!("回退到树表 0 条的暖机根：回退行那次发布仍是「没有文件版本」的一版")
+    };
+    assert_eq!(
+        (
+            row.root.tree_identifier_watermark,
+            row.record.new_tree_identifier_watermark
+        ),
+        (
+            ring_watermark_before_the_rollback,
+            ring_watermark_before_the_rollback
+        ),
+        "回退行那次发布的根与记录新根段都带回退之前根环里的水位 max，不带暖机根自己的 11"
+    );
+    let rolled_back_verdicts =
+        verdicts_without_any_violation(&pool.memory_pool(), "回退之后（环里还有 (1, 3)）");
+    assert_judged_and_holding(&rolled_back_verdicts, &["I-7.8"], "回退之后");
+
+    let instance = rolled_back.output.instance;
+    let mut allocator = rolled_back.allocator.clone();
+    let first_content = content_of(3300, 17);
+    let first_after_rollback = publish_first_file_after_the_rollback(
+        &mut pool,
+        &mut allocator,
+        &rolled_back.current,
+        instance,
+        &first_content,
+    );
+    assert_fresh_tree_identifiers(
+        &first_after_rollback,
+        ring_watermark_before_the_rollback,
+        highest_tree_identifier_before_the_rollback,
+    );
+    assert_eq!(
+        first_after_rollback.tree_identifiers.extent,
+        TreeIdentifier(ring_watermark_before_the_rollback),
+        "从回退带过来的水位起连号发：extent 树拿到的就是 19"
+    );
+    verdicts_without_any_violation(&pool.memory_pool(), "回退之后再发第一个文件版本");
+
+    let overwrite_content = content_of(2900, 23);
+    let overwritten = overwrite_in_process(&mut pool, &overwrite_content, instance);
+    assert_eq!(
+        overwritten.tree_identifiers, first_after_rollback.tree_identifiers,
+        "覆盖写照抄这八个号，不另发"
+    );
+    let final_verdicts =
+        verdicts_without_any_violation(&pool.memory_pool(), "回退之后第一个文件版本再覆盖写一版");
+    assert_judged_and_holding(
+        &final_verdicts,
+        &["I-7.8", "I-9.14", "I-9.10"],
+        "回退之后第一个文件版本再覆盖写一版",
+    );
+    assert_eq!(
+        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
+        RecoveryOutcome::FileRead {
+            root: (instance, overwritten.root.checkpoint_txg),
+            content: overwrite_content,
+        },
+        "冷启动择覆盖写那一版的根，读回它的内容"
+    );
+}
+
+/// C511（回退到无文件那一版之后诞生代怎么接） 判别力那一格：回退到暖机根 (1, 2) 之后在同一个会话里连推零单元发布，直到根环里一条
+/// 带文件版本的根都不剩（(1, 3) 的根槽被轮转盖掉），盘上仍留着 (1, 3) 那一版写出的 11..18 号码 2 单元（被抛弃、影子账隔离着，
+/// 零单元发布一个槽都不取）。这时 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 只剩新线上的根可取 max：回退那一版带环里的 max 19
+/// 并一路照抄下来就成立；沿回退到的那一版带（暖机根的 11，今天之前的取法）就是 11 ≤ 18，当场红。
+/// 之后退出、重开可写挂载（写行那次发布按环算水位：环里只剩新线上带 19 的根），再发第一个文件版本、覆盖写一版：
+/// 八个号从 19 起发，池级 checker 一条违例都没有，I-7.8、I-9.14（树表条目的诞生 txg 跨根不变）、I-9.10（对象出生代三处一致） 都真被判过。
+/// 重开这一步不省：同一个会话里转满一圈根环之后，被换下的 mkfs 实例表还在 defer 队列里、已经没有一条环里的根引用它，
+/// I-3.1（已分配统计对得上） 在那里判红——那是里程碑「第二个事务」增补 2 收口表第 ② 行记着的「一次挂载转过一整圈根环」那一族，
+/// 与水位无关；重开时的回收把它还回去。
+#[test]
+fn after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring(
+) {
+    let mut pool = build_pool("step-four-rollback-to-warm-up-root-ring-turns");
+    let highest_tree_identifier_before_the_rollback = pool.output.tree_identifiers.highest();
+    let ring_watermark_before_the_rollback =
+        highest_watermark_among_ring_roots(&pool.memory_pool());
+    let mut devices_for_the_rollback = pool.reopen_recorded();
+    let rolled_back = mount_rollback(
+        &parameters(),
+        &mut devices_for_the_rollback,
+        warm_up_root(),
+        ShadowLedger::On,
+    )
+    .expect("环里还留着带文件版本的根时回退照常做");
+    pool.devices = Some(devices_for_the_rollback);
+    let instance = rolled_back.output.instance;
+    let mut current = rolled_back.current.clone();
+    let parameters_of_the_pool = parameters();
+    let ring_slots = parameters_of_the_pool
+        .geometry
+        .root_ring_slots_per_region
+        .count()
+        * ROOT_RING_REGIONS;
+    let ring_still_holds_a_file_version = |image: &MemoryPool| {
+        readable_roots(
+            image,
+            &parameters_of_the_pool.region_devices,
+            &parameters_of_the_pool.geometry,
+            &parameters_of_the_pool.filesystem_identifier,
+        )
+        .iter()
+        .any(|root| matches!(tree_table_has_no_entries(image, root), Ok(false)))
+    };
     assert!(
-        matches!(
-            refused,
-            Err(MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion {
-                target,
-                file_version_root: RollbackTarget {
-                    instance: InstanceGeneration(1),
-                    checkpoint_txg: CheckpointTxg(3),
-                },
-            }) if target == warm_up_root
-        ),
-        "暖机根下面没有文件版本，而环里还有 (1, 3)：{:?}",
-        refused.as_ref().err()
-    );
-    assert_eq!(
-        disk_snapshot(&pool.memory_pool(), &pool.stream),
-        before,
-        "盘上逐字节不变"
+        ring_still_holds_a_file_version(&pool.memory_pool()),
+        "回退刚做完：(1, 3) 还在环里"
+    );
+    let mut empty_publishes = 0u64;
+    while ring_still_holds_a_file_version(&pool.memory_pool()) {
+        assert!(
+            empty_publishes < ring_slots,
+            "推满一圈根环 {ring_slots} 次之前 (1, 3) 的根槽一定被盖掉"
+        );
+        let version = current_version_without_file(&current);
+        let open_devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&parameters_of_the_pool, open_devices.as_mut_slice());
+        let next = publish_without_units(
+            &mut writer,
+            &version.root,
+            ZeroUnitPublishPlan {
+                txg: CheckpointTxg(version.root.checkpoint_txg.0 + 1),
+                counter: version.record.counter + 1,
+                instance,
+                back_chain: back_chain_of(&version.record_bytes),
+                rollback_floor: version.root.rollback_floor,
+                // 会话里接着现行那一版：它的水位就是根环里的 max（回退行那次发布取过，之后照抄）。
+                tree_identifier_watermark: version.root.tree_identifier_watermark,
+            },
+        )
+        .expect("零单元发布");
+        current = PoolVersion::WithoutFile(next);
+        empty_publishes += 1;
+    }
+    // 先判 checker 再核水位：沿回退到的那一版带水位时，这一步要红在 I-7.8 上（判别力自证就看这一格）。
+    let turned_verdicts = verdicts_without_any_violation(
+        &pool.memory_pool(),
+        "回退之后推零单元发布到 (1, 3) 离开根环",
+    );
+    assert_judged_and_holding(&turned_verdicts, &["I-7.8"], "(1, 3) 离开根环之后");
+    assert_eq!(
+        highest_watermark_among_ring_roots(&pool.memory_pool()),
+        ring_watermark_before_the_rollback,
+        "(1, 3) 离开根环之后，环里新线上的根仍带 19"
+    );
+
+    let mut devices_for_the_remount = pool.reopen_recorded();
+    let remounted =
+        mount_writable(&parameters(), &mut devices_for_the_remount).expect("重开可写挂载");
+    pool.devices = Some(devices_for_the_remount);
+    assert_eq!(
+        remounted.current.root().tree_identifier_watermark,
+        ring_watermark_before_the_rollback,
+        "重开时写行那次发布按环算水位：环里只剩新线上的根，都带 19"
+    );
+    let remounted_instance = remounted.output.instance;
+    let mut allocator = remounted.allocator.clone();
+    let first_after_rollback = publish_first_file_after_the_rollback(
+        &mut pool,
+        &mut allocator,
+        &remounted.current,
+        remounted_instance,
+        &content_of(3100, 29),
+    );
+    assert_fresh_tree_identifiers(
+        &first_after_rollback,
+        ring_watermark_before_the_rollback,
+        highest_tree_identifier_before_the_rollback,
+    );
+    verdicts_without_any_violation(
+        &pool.memory_pool(),
+        "(1, 3) 离开根环、重开之后再发第一个文件版本",
+    );
+    overwrite_in_process(&mut pool, &content_of(2700, 31), remounted_instance);
+    let final_verdicts = verdicts_without_any_violation(
+        &pool.memory_pool(),
+        "(1, 3) 离开根环、重开之后第一个文件版本再覆盖写一版",
+    );
+    assert_judged_and_holding(
+        &final_verdicts,
+        &["I-7.8", "I-9.14", "I-9.10"],
+        "(1, 3) 离开根环、重开之后第一个文件版本再覆盖写一版",
+    );
+}
+
+/// C511（回退到无文件那一版之后诞生代怎么接） 第 3 步里条款没写的那一格（根环里读不出的根怎么算）按最保守的读法做：
+/// 带文件版本的根 (1, 3) 那一槽读不出（介质错、持续），而它那次发布的记录还在环里——本实例第一次发布要带的水位
+/// 仍是 19，取自那条记录的新根段（D23（journal 的角色与格式） 已定项 15）。只看读得出的根就只剩带 mkfs 种下的 11 的那几条，
+/// 回退到暖机根之后再发第一个文件版本就会重发 (1, 3) 用过的 11..18（D8（核心索引结构） 已定项 8 ②）。
+#[test]
+fn with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_from_its_journal_record(
+) {
+    let pool = build_pool("step-four-unreadable-file-version-root-watermark");
+    let image = pool.memory_pool();
+    let system_configuration = choose_system_configuration(&image).expect("系统配置");
+    let immutable = &system_configuration.immutable;
+    let file_version_root_slot = target_for_publish(
+        CheckpointTxg(3),
+        parameters().geometry.root_ring_slots_per_region,
+    );
+    let unreadable = PoolReaderWithUnreadableRootRingSlots::new(
+        &image,
+        RootRingSlotTarget {
+            named_slots: NamedRootRingSlots::naming(&[file_version_root_slot]),
+            region_devices: immutable.region_devices,
+            fixed_structure_slot_spacing: immutable.sizes.fixed_structure_slot_spacing,
+        },
+    );
+    let readable = readable_roots(
+        &unreadable,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    );
+    assert!(
+        !readable.is_empty()
+            && readable
+                .iter()
+                .all(|root| root.checkpoint_txg != CheckpointTxg(3)),
+        "(1, 3) 那一槽读不出、别的根读得出：{:?}",
+        readable
+            .iter()
+            .map(|root| (root.instance, root.checkpoint_txg))
+            .collect::<Vec<_>>()
+    );
+    assert_eq!(
+        readable
+            .iter()
+            .map(|root| root.tree_identifier_watermark)
+            .max(),
+        Some(11),
+        "读得出的根都带 mkfs 种下的 11"
+    );
+    let records = scan_journal(&unreadable, &system_configuration);
+    assert!(
+        records
+            .values()
+            .any(|record| record.new_tree_identifier_watermark == 19),
+        "(1, 3) 那次发布的记录还在环里，新根段带 19"
+    );
+    assert_eq!(
+        highest_tree_identifier_watermark_in_the_ring(
+            &unreadable,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+            &records,
+        ),
+        Some(19),
+        "根读不出、记录还在：水位按记录带的 19 算，不退回读得出的根带的 11"
     );
 }
 
--- a/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
@@ -30,12 +30,14 @@
     mount_rollback, mount_writable, InstanceRow, MountError, RollbackTarget, ShadowLedger,
 };
 use singlefs_core::recovery::{
-    choose_root, choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
+    choose_root, choose_system_configuration, recover, JournalPolicy, RecoveryFailure,
+    RecoveryOutcome,
 };
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{
     acquire_instance, publish_first_file, publish_without_units, warm_up, FirstFile, PoolVersion,
-    PoolWriter, PublishError, ZeroUnitPublishPlan,
+    PoolWriter, PublishError, TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees,
+    ZeroUnitPublishPlan,
 };
 use singlefs_harness::crash::writes_and_segments;
 use singlefs_harness::fault_injection::{
@@ -317,6 +319,11 @@
             instance,
             back_chain: back_chain_of(&warmed.last_record_bytes),
             rollback_floor: CheckpointTxg(0),
+            tree_identifier_watermark: warmed
+                .roots
+                .last()
+                .expect("暖机两代根")
+                .tree_identifier_watermark,
         },
     )
     .expect("第三次零单元发布");
@@ -368,6 +375,113 @@
     );
     assert_eq!(allocator.records(), records_before.as_slice(), "分配器没动");
 }
+
+/// C511（回退到无文件那一版之后诞生代怎么接） 第 3 步给 `publish_first_file` 新加的两道，都在任何写之前返回：
+/// ① 判「那一版有没有过文件版本」改看它的树表条数，而那一版的树表单元两份都读不出（两盘上 mkfs 那片树表整个写零）
+///    ⇒ `TreeTableOfTheVersionToBuildOnUnreadable`，带着恢复路径那一格的原因；
+/// ② 八棵树从那一版的水位（盘上读来的 8 字节）起连号发，而水位离 `u64::MAX` 不到八个号
+///    ⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`，带着那个水位。
+/// 两格都是一个写、一道屏障都没发（录制流一步不多）、分配器不动。
+#[test]
+fn first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write() {
+    let mut formatted = format_pool("step-three-formatted-first-file-cannot-issue-trees");
+    let publish_parameters = parameters();
+    let genesis_root = formatted.genesis.root;
+    let open_devices = formatted.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
+    let instance = acquire_instance(&mut writer).expect("取号");
+    let warmed = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
+    let warm_up_root = *warmed.roots.last().expect("暖机两代根");
+    let mut allocator = PoolAllocator::new(vec![
+        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
+        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
+    ]);
+    allocator.mark_format_time_units(
+        Placement {
+            slot: INSTANCE_TABLE_SLOT,
+            span: 2,
+        },
+        Placement {
+            slot: TREE_TABLE_GENESIS_SLOT,
+            span: 1,
+        },
+    );
+    let records_before = allocator.records().to_vec();
+    let content = file_content();
+
+    // ②：水位离 u64::MAX 只剩三个号。
+    let mut root_with_no_room_for_eight_trees = warm_up_root;
+    root_with_no_room_for_eight_trees.tree_identifier_watermark = u64::MAX - 3;
+    let recorded_before_the_watermark_case = formatted.stream.operations().len();
+    let refused_for_the_watermark = publish_first_file(
+        &mut writer,
+        &mut allocator,
+        &root_with_no_room_for_eight_trees,
+        FirstFile {
+            content: &content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+        },
+        instance,
+        &warmed.last_record_bytes,
+    );
+    assert!(
+        matches!(
+            refused_for_the_watermark,
+            Err(PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(
+                TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
+                    tree_identifier_watermark
+                }
+            )) if tree_identifier_watermark == u64::MAX - 3
+        ),
+        "水位离 u64::MAX 不到八个号：{:?}",
+        refused_for_the_watermark.as_ref().err()
+    );
+    assert_eq!(
+        formatted.stream.operations().len(),
+        recorded_before_the_watermark_case,
+        "水位那一格：一个写、一道屏障都没发"
+    );
+    assert_eq!(allocator.records(), records_before.as_slice(), "分配器没动");
+
+    // ①：两盘上 mkfs 那片树表整个写零（整单元校验和对不上），那一版的树表两份都读不出。
+    for (_, device) in writer.devices.iter_mut() {
+        device
+            .write_at(
+                DeviceOffsetInBytes(TREE_TABLE_GENESIS_SLOT.0 * 16384),
+                &[0u8; 16384],
+                WriteDurability::ForceUnitAccess,
+            )
+            .expect("抹树表写得进去");
+    }
+    let recorded_before_the_tree_table_case = formatted.stream.operations().len();
+    let refused_for_the_tree_table = publish_first_file(
+        &mut writer,
+        &mut allocator,
+        &warm_up_root,
+        FirstFile {
+            content: &content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+        },
+        instance,
+        &warmed.last_record_bytes,
+    );
+    assert!(
+        matches!(
+            refused_for_the_tree_table,
+            Err(PublishError::TreeTableOfTheVersionToBuildOnUnreadable {
+                failure: RecoveryFailure::UnitUnreadable { slot }
+            }) if slot == TREE_TABLE_GENESIS_SLOT
+        ),
+        "那一版的树表两份都读不出：{:?}",
+        refused_for_the_tree_table.as_ref().err()
+    );
+    assert_eq!(
+        formatted.stream.operations().len(),
+        recorded_before_the_tree_table_case,
+        "树表那一格：一个写、一道屏障都没发"
+    );
+    assert_eq!(allocator.records(), records_before.as_slice(), "分配器没动");
+}
 
 fn region_device(txg: u64) -> DeviceIdentity {
     let target = target_for_publish(
--- a/crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs
+++ b/crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs
@@ -339,6 +339,11 @@
     // 位置条目那份上恢复**读回了文件**（那条指针指的实例表不在读路径上，两条位置条目的第一条又还是好的）——
     // 这一条钉的正是「改法没有把读路径也一起拒掉」。
     //
+    // ⚠️ 这段写死的历史第 4 步是「回退到环里第 3 新的根」，那是一条暖机根、而环里还留着带文件的根：C511（回退到无文件那一版之后诞生代怎么接）
+    // 第 3 步之前那一步在写之前被拒、之后几步没有会话；拿掉那道拒绝之后它照常回退，后面接着再发第一个文件版本、覆盖写、重开（实例 3），
+    // 盘面跟着变：实例表那个落点从槽 50304 挪到 50368，恢复择到的根从 (2, 9) 变成 (3, 13)。下面几处槽号与根照今天的盘面写，
+    // 钉的错误成员一个没变。
+    //
     // ⚠️ 「跨度越过单元区末尾」那条坏法在**两块 4 GiB 的盘**上够不着它名字里说的那一格：跨度写成 0x7FFF = 32767 槽，
     // 从 50176 起到 82943，而单元区末尾是槽 262144（4 GiB ÷ 16 KiB）。它实际打中的是同一批判定里的
     // 「同一块盘上两条分配记录罩住同一个槽」——32767 槽罩过了后面每一条记录。照实钉，不按坏法的名字钉。
@@ -392,7 +397,7 @@
             DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly,
             "InvariantViolated { invariant: \"E142 走读同款\", detail: \"分配记录不是每个落点每盘各一条",
             "ReleaseTargetAlreadyReleased { unit: InstanceTable, device: DeviceIdentity(1), \
-             slot: SlotNumber(50304) }",
+             slot: SlotNumber(50368) }",
         ),
         (
             DamageKind::RelabelledInodeWatermarkAccountingRow,
@@ -401,10 +406,10 @@
         ),
         (
             DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot,
-            "FileRead（实例 2 第 9 代根",
+            "FileRead（实例 3 第 13 代根",
             "ReleaseTargetLocationsOnDifferentSlots { unit: InstanceTable, disagreement: \
              LocationEntriesOnDifferentSlots { devices: [DeviceIdentity(0), DeviceIdentity(1)], \
-             slots: [SlotNumber(50304), SlotNumber(50305)] } }",
+             slots: [SlotNumber(50368), SlotNumber(50369)] } }",
         ),
         // 第十三条是 C504（树表条目宽在走读里无守卫，今天没坏法打得到）：坏的是**树表单元自己**，不是它指着的某棵树的根。
         // 两个读者都走 `TreeTableEntry::parse`（它先判「这条条目正好 200 字节」），所以两侧同一个成员；
--- a/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
+++ b/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
@@ -116,7 +116,6 @@
     for member in [
         "MountError::RollbackFloorAboveCeiling",
         "MountError::RollbackTargetNotACandidate(NotInRing)",
-        "MountError::RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion",
         "PublishError::ContentExceedsDataUnit",
         "PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile",
     ] {
```

## 六、watermark 那一组新增的变异名（`/tmp/claude-1000/impl-watermark/mutations-append.tsv`，8 行，原样，还没合并进 `crates/mutations.tsv`；全部以 `C511 第 3 步` 开头，与七、八节 C511/C512 那一组的 5 行不是同一批）

sha256sum（本次现查）：`a44ae5b93c9bdd900ce823235ba4ae697fb8a54d0a6fc88eb30da140632cea3e`。

```tsv
C511 第 3 步（D8 已定项 8 ②）：回退行那次发布的树 ID 水位沿回退到的那一版带（暖机根的 11），不取根环里的 max；推到 (1, 3) 离开根环之后 I-7.8 判红	crates/singlefs-core/src/mount.rs	        ring.max(version_to_build_on.tree_identifier_watermark)	        version_to_build_on.tree_identifier_watermark	-p singlefs-harness --test second_transaction_step_four_rollback -- after_rolling_back_to_a_warm_up_root	after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring
C511 第 3 步（D8 已定项 8 ②）：回退行那次发布的树 ID 水位沿回退到的那一版带，不取根环里的 max；偏向回退的随机历史抽样判出 I-7.8（这一档的必红是抽样断言，随测试周期的种子基重验）	crates/singlefs-core/src/mount.rs	        ring.max(version_to_build_on.tree_identifier_watermark)	        version_to_build_on.tree_identifier_watermark	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rollback_heavy_random_histories	rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms
C511 第 3 步（D8 已定项 8 ②：号永不重发）：回退到树表 0 条的一版之后再发第一个文件版本，八棵树照 mkfs 的水位 11 重发 11..18，不从那一版带过来的水位起发	crates/singlefs-core/src/transaction.rs	        FileVersionTreeIdentifiers::issued_from_watermark(\n            version_to_build_on.tree_identifier_watermark,\n        )	        FileVersionTreeIdentifiers::issued_from_watermark(\n            TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n        )	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root_while	rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_carries_the_ring_watermark_and_the_next_first_file_issues_fresh_tree_identifiers
C511 第 3 步：checker 对中央映射树根的 I-1.3 退回写死树 ID 15，不按根记录里那条根指针的出生树判（回退之后重新发号的映射树判不了）	crates/singlefs-checker/src/walk.rs	            parse_node_pointer(mapping_root_pointer).birth_tree,	            15,	-p singlefs-harness --test checker_known_bad_images -- a_central_mapping_root_whose_header_tree_differs	a_central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant
C511 第 3 步：publish_first_file 退回按「水位 = 11 ⇒ 树还没建」判，不看树表条数；回退到暖机根之后那一版水位 19、树表 0 条，第一个文件版本被拒	crates/singlefs-core/src/transaction.rs	    if tree_table_entries != 0 {	    if version_to_build_on.tree_identifier_watermark != TREE_IDENTIFIER_WATERMARK_AT_MKFS {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root_while	rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_carries_the_ring_watermark_and_the_next_first_file_issues_fresh_tree_identifiers
C511 第 3 步：从水位起连号发八棵树的号不先判装不装得下（盘上读来的水位离 u64::MAX 不到八个号时越界 panic，不交回错误成员）	crates/singlefs-core/src/transaction.rs	            .checked_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            )\n            .ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {\n                tree_identifier_watermark,\n            })?;	            .wrapping_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            );	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- first_file_version_that_cannot_judge	first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write
C511 第 3 步：publish_first_file 读不出那一版的树表时当成 0 条接着写，不在写之前交回 TreeTableOfTheVersionToBuildOnUnreadable	crates/singlefs-core/src/transaction.rs	        .map_err(|failure| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;	        .unwrap_or(0);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- first_file_version_that_cannot_judge	first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write
C511 第 3 步（交主 agent 的那一格：根环里读不出的根怎么算）：本实例第一次发布的树 ID 水位只取读得出的根，不并上环里记录新根段带的水位；带文件版本的根那一槽读不出时退回暖机根的 11	crates/singlefs-core/src/recovery.rs	    let mut highest: Option<u64> = records\n        .values()\n        .map(|record| record.new_tree_identifier_watermark)\n        .max();	    let mut highest: Option<u64> = None;	-p singlefs-harness --test second_transaction_step_four_rollback -- with_the_file_version_root_slot_unreadable	with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_from_its_journal_record
```

## 七、C511 / C512 那一组：`crates/singlefs-core/src/root_record.rs` 的 `git diff HEAD` 全文（主工作区现状，本次现取）

```diff
diff --git a/crates/singlefs-core/src/root_record.rs b/crates/singlefs-core/src/root_record.rs
index 42910b7..11ddfdb 100644
--- a/crates/singlefs-core/src/root_record.rs
+++ b/crates/singlefs-core/src/root_record.rs
@@ -1,4 +1,4 @@
-//! 根记录（D22（单元原子性怎么合成） 已定项 7）：371 字节住一个判定宽度（physical_block_size）的根槽，
+//! 根记录（D22（单元原子性怎么合成） 已定项 7）：457 字节住一个判定宽度（physical_block_size）的根槽，
 //! 自证校验和罩整槽含补齐、自身按 0 参与。行序即盘上顺序，偏移按行累加。
 
 use singlefs_format::{NODE_POINTER_BYTES, ROOT_RECORD_BYTES, WIDE_CHECKSUM_BYTES};
@@ -23,14 +23,21 @@ pub struct RootRecord {
     pub instance_table: NodePointer,
     /// 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
     pub mapping_root: NodePointer,
+    /// 树表 0 条的那一版的分配记录树的根（2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪） 加）。
+    ///
+    /// **只有树表 0 条的那一版用这一项**：它没有树表条目可放（往树表里写一条，`tree_table_has_no_entries` 当场翻面，
+    /// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变）。带文件的一版恒 [`NodePointer::empty_root`]
+    /// ——那一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8），两处都写就成了同一个量的两份手抄。
+    /// mkfs 的第 0 代也恒全零：那一版的账由 `mount::format_time_allocator` 从实例表与树表两条指针直接算。
+    pub allocation_record_tree_root: NodePointer,
 }
 
 impl RootRecord {
-    /// 写成一个 `slot_bytes` 宽的槽；记录 371 字节，其余补 0。
+    /// 写成一个 `slot_bytes` 宽的槽；记录 457 字节，其余补 0。
     #[must_use]
     pub fn to_slot(&self, slot_bytes: usize) -> Vec<u8> {
         assert!(
-            slot_bytes >= usize::try_from(ROOT_RECORD_BYTES).expect("371"),
+            slot_bytes >= usize::try_from(ROOT_RECORD_BYTES).expect("457"),
             "根槽装不下根记录"
         );
         let mut writer = ByteWriter::new(slot_bytes);
@@ -49,6 +56,7 @@ impl RootRecord {
         writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
         self.instance_table.write_to(&mut writer);
         self.mapping_root.write_to(&mut writer);
+        self.allocation_record_tree_root.write_to(&mut writer);
         writer.put_u8(0); // 算法类型：未加密
         writer.skip(12 + 16); // nonce、MAC：第一版留位全 0
         writer.assert_position(ROOT_RECORD_BYTES, "根记录");
@@ -61,7 +69,7 @@ impl RootRecord {
     /// 读者：magic、整槽校验和、fsid、flags 四关。
     #[must_use]
     pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> Option<Self> {
-        if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("371")
+        if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("457")
             || bytes[..4] != ROOT_MAGIC
         {
             return None;
@@ -85,9 +93,10 @@ impl RootRecord {
         reader.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
         let instance_table = NodePointer::read_from(&mut reader);
         let mapping_root = NodePointer::read_from(&mut reader);
+        let allocation_record_tree_root = NodePointer::read_from(&mut reader);
         assert_eq!(
             reader.position(),
-            ROOT_CHECKSUM_OFFSET + 32 + 2 * usize::try_from(NODE_POINTER_BYTES).expect("86")
+            ROOT_CHECKSUM_OFFSET + 32 + 3 * usize::try_from(NODE_POINTER_BYTES).expect("86")
         );
         Some(Self {
             filesystem_identifier,
@@ -98,6 +107,7 @@ impl RootRecord {
             rollback_floor,
             instance_table,
             mapping_root,
+            allocation_record_tree_root,
         })
     }
 }
@@ -116,14 +126,15 @@ mod tests {
             rollback_floor: CheckpointTxg(0),
             instance_table: NodePointer::empty_root(),
             mapping_root: NodePointer::empty_root(),
+            allocation_record_tree_root: NodePointer::empty_root(),
         }
     }
 
     #[test]
-    fn root_record_is_371_bytes_with_the_checksum_at_138_and_round_trips() {
+    fn root_record_is_457_bytes_with_the_checksum_at_138_and_round_trips() {
         let slot = sample().to_slot(512);
         assert_eq!(slot.len(), 512);
-        assert!(slot[371..].iter().all(|byte| *byte == 0));
+        assert!(slot[457..].iter().all(|byte| *byte == 0));
         assert_eq!(&slot[..4], b"SFSR");
         assert_eq!(RootRecord::parse_slot(&slot, &[5u8; 16]), Some(sample()));
         assert_eq!(
```

## 八、C511 / C512 那一组：其余各处，按「文件::项名」从主工作区现状抽出（`python3 research/scripts/quote-rust-items.py`，退出码 0，回读逐字节比对通过；每段自带 `### 文件:起始行-结束行（项名）` 与内层 ```rust``` 代码围栏）

项名来源：`research/prompts/m2-c511-c512-implementer-report.md` 第三节「连带改了哪几处」表格 + 第四节变异表（`crates/mutations.tsv` 第 310–314 行五行原文/替换文所在的函数）。

`allocator.rs` 里报告点名的三处（639 字段声明、659 构造函数里的默认初始化、738 写入方法）只抽了两处：`PoolAllocator`（结构体定义，覆盖字段声明）与 `note_allocation_record_node_of_the_version_without_file`（写入方法）。第三处（`PoolAllocator::new` 里那一行默认初始化）没抽：`allocator.rs` 里同名的 `new` 有两个（`DeviceFreeMap::new` 与 `PoolAllocator::new`），`quote-rust-items.py` 对同名不止一个的项拒绝（退出码 4，提示换 `impl 类型名` 或直接取外层函数，不猜是哪一个）；`impl PoolAllocator` 整块过大且那一行只是把字段初始化成 `None`（字段本身已在结构体定义里带着完整文档注释），未再抽整个 `impl` 块。

第四节变异表 5 行里，310、312 两行的原文/替换文落在 `check_pool_image` 内，311 落在 `judge_release_generation_and_tree_table_birth` 内，313 落在 `publish_instance_table_after_the_release_check` 内（已在上面抽出），314 落在 `allocator_of_version_without_file` 内（已在上面抽出）；另外把这 5 行变异各自点名的「必红」测试函数一并抽出（4 个，`the_third_writable_mount_keeps_…` 一个测试同时钉住 313、314 两行）。

### crates/singlefs-core/src/transaction.rs:708-713（PlacementsReleasedByTheRowPublish）

```rust
/// 写行那次发布换下的两片：上一版的实例表，与上一版那片分配记录树节点
/// （上一版是 mkfs 的第 0 代时根记录那一项全零、没有这一片）。
struct PlacementsReleasedByTheRowPublish {
    instance_table: Placement,
    allocation_record_node: Option<Placement>,
}
```

### crates/singlefs-core/src/transaction.rs:745-762（version_without_file_row_publish_admission）

```rust
/// 写行那次发布的准入：这次之后的分配记录条数装不进一个节点就在动分配器之前拒掉，不许走到 `build_index_node` 的断言。
/// 与 `admission_of_one_publish` 的第一条共用 `refuse_when_the_allocation_records_do_not_fit_one_node`；
/// 记账与映射那两条在这一格没有对象（树表 0 条 ⇒ 这一版没有记账树、没有映射条目），所以不调那一整道。
/// 释放只改写记录、不加条数，所以基数是分配器此刻的记录数；这次新增的是重写的那两个角色每盘各一条。
///
/// # Errors
/// `AllocationRecordsExceedOneNode`。
fn version_without_file_row_publish_admission(
    allocator: &PoolAllocator,
) -> Result<(), PublishError> {
    let rewritten = [
        TransactionUnit::InstanceTable,
        TransactionUnit::AllocationTree,
    ];
    refuse_when_the_allocation_records_do_not_fit_one_node(
        allocator.records().len() + rewritten.len() * allocator.devices.len(),
    )
}
```

### crates/singlefs-core/src/transaction.rs:764-783（refuse_when_the_allocation_records_do_not_fit_one_node）

```rust
/// 分配记录树第一版只有一个节点（分裂不做）：这次发布之后装不下就报错，不许走到 `build_index_node` 的断言
/// （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。两条准入路径共用这一处，两处各抄一份会分叉。
///
/// # Errors
/// `AllocationRecordsExceedOneNode`。
fn refuse_when_the_allocation_records_do_not_fit_one_node(
    records_after_this_publish: usize,
) -> Result<(), PublishError> {
    let allocation_node_capacity = index_node_entry_capacity(
        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
        usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
    );
    if records_after_this_publish > allocation_node_capacity {
        return Err(PublishError::AllocationRecordsExceedOneNode {
            records: records_after_this_publish,
            capacity: allocation_node_capacity,
        });
    }
    Ok(())
}
```

### crates/singlefs-core/src/transaction.rs:785-799（build_allocation_record_node）

```rust
/// 分配记录树那一片单元（字节表五）：分配器此刻的每一条记录按 (设备, 槽号) 升序装进一个层 0 节点。
/// 两条发布路径共用这一处——带文件的那一版的 t5 与树表 0 条那一版写行时的那一片，装出来的字节按同一条规则；
/// 两处各抄一份会分叉，而「重开之后从这一片重建出来的账与发布时那个分配器相同」正压在两处装的是同一个东西上。
///
/// `tree` 两条路径不同，这是唯一的差别：带文件的那一版里它是登记在树表里的分配记录树（第一个文件版本那次发的号，mkfs 那条流上是 13）；
/// 树表 0 条那一版还没登记过任何树（那一版的水位还没被那几个号推过，写 13 会当场判红 I-7.8（树 ID 水位不小于盘上出现过的最大树 ID）），
/// 那一片**不属于任何树**（树 ID 0）、由根记录独占持有，与树表单元、实例表单元同一个身份（D22（单元原子性怎么合成） 已定项 7 / 已定项 12）。
///
/// # Panics
/// 分配器一条记录都没有（层 0 节点的 key 区间取不出来）：池里恒有 mkfs 那两个单元的记录，取不出来说明账已经坏了。
fn build_allocation_record_node(
    allocator: &PoolAllocator,
    tree: TreeIdentifier,
    txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
```

### crates/singlefs-core/src/transaction.rs:833-994（publish_instance_table_after_the_release_check）

```rust
/// `publish_instance_table_on_version_without_file` 的后半段：释放、取落点、装单元、落盘。分出来只为把「失败就把分配器换回去」
/// 收在一处——中间每一步都可能提前返回，散在调用点上就会漏掉某一条路径。
fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous_root: &RootRecord,
    plan: InstanceTableOnlyPublishPlan<'_>,
    swapped_out: PlacementsReleasedByTheRowPublish,
) -> Result<VersionWithoutFilePublishOutput, PublishError> {
    let identity = TransactionUnit::InstanceTable;
    let txg = plan.txg;
    let instance = plan.instance;
    let filesystem_identifier = &pool.parameters.filesystem_identifier;
    let write_order = WriteOrder {
        instance,
        // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①：空发布与写行的记录写事务号 0）。
        transaction: 0,
    };
    let released = swapped_out.instance_table;
    allocator.release(released, txg);
    if let Some(previous_allocation_record_node) = swapped_out.allocation_record_node {
        allocator.release(previous_allocation_record_node, txg);
    }
    let placement = allocate_placement_for_role(allocator, identity, txg)?;
    // 分配记录树那一片的落点在实例表之后取：这一片自己的分配记录也要进它自己那个节点，所以两个落点都取完才装节点。
    let allocation_placement =
        allocate_placement_for_role(allocator, TransactionUnit::AllocationTree, txg)?;
    // 这一版的分配记录树节点换成了刚取的这一片：下一次发布（再写一次行，或在这一版上发第一个文件版本）要换下它，
    // 而那时手里只有这个分配器——记的要是上一版那一片（这次刚释放掉的那一片），新写的这一片就永远没人释放，
    // I-3.1（已分配统计对得上） 在抬 F 之后当场红。
    allocator.note_allocation_record_node_of_the_version_without_file(allocation_placement);
    // 这一次发布有实例表与分配记录树两个提交内生块：实例表归树 0，分配记录树归树 13，各自在 (txg, 实例) 上发自己的出生序号。
    let mut sequences = BirthSequenceAllocator::default();
    let birth_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let unit = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            record_type: PACKED_TYPE_INSTANCE_TABLE,
            container: 0,
            container_birth: CheckpointTxg(0),
        },
        u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
        plan.instance_table_records,
        txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
    );
    let locations = pool.location_entries(placement.slot, &unit);
    let instance_table_pointer = NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            birth_txg: txg,
        },
        locations,
        instance,
        birth_sequence,
    };
    let allocation_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let allocation_unit = build_allocation_record_node(
        allocator,
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        txg,
        filesystem_identifier,
        instance,
        allocation_sequence,
    );
    let allocation_locations = pool.location_entries(allocation_placement.slot, &allocation_unit);
    let allocation_record_tree_root = NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            birth_txg: txg,
        },
        locations: allocation_locations,
        instance,
        birth_sequence: allocation_sequence,
    };
    let record = JournalRecord {
        instance,
        counter: plan.counter,
        checkpoint_txg: txg,
        transaction: 0,
        is_commit: true,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
        new_tree_table: previous_root.tree_table,
        new_mapping_root: previous_root.mapping_root,
        new_tree_identifier_watermark: plan.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项——实例表与分配记录树。
        named: vec![
            NamedUnit {
                locations,
                unit_class: identity.unit_class(),
                // 实例表单元不属于任何一棵树（树 0）；这一版还没发过树 ID，不按 `TransactionUnit::tree` 取。
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                birth_txg: txg,
                key_tail: node_key_tail(instance, birth_sequence),
            },
            NamedUnit {
                locations: allocation_locations,
                unit_class: TransactionUnit::AllocationTree.unit_class(),
                // 树 ID 0：这一版还没登记过任何树，那一片由根记录独占持有（见 `build_allocation_record_node` 的文档注释）。
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                birth_txg: txg,
                key_tail: node_key_tail(instance, allocation_sequence),
            },
        ],
    };
    let record_bytes = record.to_bytes();
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        instance,
        checkpoint_txg: txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: instance_table_pointer,
        mapping_root: previous_root.mapping_root,
        allocation_record_tree_root,
    };
    let root_slot = root.to_slot(pool.root_slot_bytes());
    let writes_before_this_row_publish = pool.writes_by_structure_kind.clone();
    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：六步收在一个闭包里，失败在这里记账再把错原样交回。
    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
        writer.perform(CommitStep::WriteUnitToEveryDevice {
            slot: placement.slot,
            unit: &unit,
            identity,
        })?;
        writer.perform(CommitStep::WriteUnitToEveryDevice {
            slot: allocation_placement.slot,
            unit: &allocation_unit,
            identity: TransactionUnit::AllocationTree,
        })?;
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
            counter: plan.counter,
            record: &record_bytes,
        })?;
        writer.perform(CommitStep::Barrier)?;
        persist_the_root_then_rotate_the_system_configuration(
            writer,
            txg,
            &root_slot,
            plan.counter,
            instance,
        )
    };
    if let Err(cause) = persist(pool) {
        pool.count_failed_publish(&writes_before_this_row_publish);
        return Err(PublishError::BlockDevice(cause));
    }
    Ok(VersionWithoutFilePublishOutput {
        root,
        record,
        record_bytes,
        writes: pool
            .writes_by_structure_kind
            .since(&writes_before_this_row_publish),
    })
}
```

### crates/singlefs-core/src/transaction.rs:2040-2140（publish_first_file）

```rust
/// 一个池里的第一个文件版本（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
/// 建在树表 0 条的那一版上（`version_to_build_on`）：照抄它的实例表指针、换下它指着的那片 mkfs 树表。
///
/// txg 与 jsn 都从它接着算——txg = 那一版的 txg + 1、jsn = 上一条记录的 jsn + 1，**不写死 3**：
/// `FIRST_TRANSACTION_TXG` 只管 mkfs 同一个进程里那条流（mkfs → 取号 → 暖机两次 → 第一个事务），不管任何池的第一个文件版本
/// （2026-09-23 用户定案）。只做过 mkfs 的池重开一次可写挂载之后再写文件时，那一版已经推到 txg 4，这里接着写 txg 5。
///
/// 这一版的八棵树从 `version_to_build_on` 的树 ID 水位起连号发（[`FileVersionTreeIdentifiers::issued_from_watermark`]），
/// 新水位 = max(那一版的水位, 发出的最高号 + 1)（D8（核心索引结构） 已定项 8 ②）。那一版的水位就是根环里全部根记录的 max：
/// 它是这个会话里的现行那一版，挂载时本实例的第一次发布取过环里的 max（`mount` 的 `tree_identifier_watermark_of_the_ring`），
/// 之后每次发布只照抄或推高它，而环里后来写进去的根都是这个会话自己写的。mkfs 同一个进程里那条流上环里只有 mkfs 的根与暖机根，
/// 都带 mkfs 种下的 11。
///
/// # Errors
/// `version_to_build_on` 那一版的树表读不出、解不开 ⇒ `TreeTableOfTheVersionToBuildOnUnreadable`；
/// 那一版的树表不是 0 条（已经有过文件版本）⇒ `FirstFileVersionOnAVersionThatAlreadyHasAFile`（同一个文件再写一版走
/// `publish_overwrite`）。`previous_record_bytes` 解不出本池的记录，或它的 checkpoint_txg 与 `version_to_build_on` 那条根的不同
/// （两者要说同一版，新根才恒落在它上面一格）⇒ `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`
/// （m2-emptypool-nonempty-r1 云端攻方腿 Z3-A：拿 txg 4 的暖机根配 txg 2 的记录，新根盖在已有的根上、冷恢复读不到）。
/// 那一版的水位离 `u64::MAX` 不到八个号 ⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
/// 这几样都在任何写之前返回，一个字节都不写。其余同 `publish_version`。
pub fn publish_first_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    version_to_build_on: &RootRecord,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    // 「树还没建起来」看那一版的树表有几条（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步），不看水位：
    // 回退到树表 0 条的一版时水位带着根环里的 max（D8（核心索引结构） 已定项 8 ②），那一版没有树、水位却早已不是 mkfs 的 11。
    let tree_table_entries = tree_table_entry_count(&*pool.devices, version_to_build_on)
        .map_err(|failure| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;
    if tree_table_entries != 0 {
        return Err(
            PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries },
        );
    }
    let previous_record = JournalRecord::parse(
        previous_record_bytes,
        unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
    )
    .map(|record| (record.checkpoint_txg, record.counter));
    let follows_directly = previous_record
        .is_some_and(|(previous_txg, _)| previous_txg == version_to_build_on.checkpoint_txg);
    if !follows_directly {
        return Err(
            PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
                version_to_build_on: version_to_build_on.checkpoint_txg,
                previous_record,
            },
        );
    }
    let (_, previous_counter) =
        previous_record.expect("follows_directly 为真时上一条记录解得出（is_some_and）");
    // 八棵树从那一版的水位（下一个可用号）起连号发：水位是 mkfs 的 11 时就是 D8（核心索引结构） 已定项 11 那几个常量，
    // 回退到树表 0 条的一版之后水位带着环里的 max，号高于此前发过的每一个，不重发。
    let (tree_identifiers, next_available_after_this_issue) =
        FileVersionTreeIdentifiers::issued_from_watermark(
            version_to_build_on.tree_identifier_watermark,
        )
        .map_err(PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees)?;
    let first_file_version_txg = CheckpointTxg(version_to_build_on.checkpoint_txg.0 + 1);
    publish_version_of_trees(
        pool,
        allocator,
        PublishPlan {
            txg: first_file_version_txg,
            counter: previous_counter + 1,
            transaction: FIRST_TRANSACTION_NUMBER,
            // 这个实例的第一个事务：在它之前只有暖机与写行那几次发布，事务号都是 0。
            highest_transaction_number_before_this_publish: 0,
            instance,
            back_chain: back_chain_of(previous_record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: first_file_version_txg,
                // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88 的字段表定义）：
                // 就是这次发布的 txg（mkfs 同一个进程里那条流上是 3）。写 1 是 2026-09-18 之前的老样子
                // （暖机把第一个事务从 txg 1 推到 3 时这一格没跟着改，增补 2 第 11 行）。
                // 这里不写 `txg.0`：`crates/mutations.tsv` 第 22 行按那串字面锚在 `publish_overwrite` 上，同一份文件里出现两次它就腐化。
                change_count: first_file_version_txg.0,
            }),
            // 第一个事务只建第一个文件那一个 inode：树是空的，这条记录建第一片容器（D8（核心索引结构） 已定项 6）。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),
            tree_birth_txg: first_file_version_txg,
            // D8（核心索引结构） 已定项 8 ②：max(根环里全部根记录的该字段, 本次发出的最高树 ID + 1)；
            // 前一项就是那一版的水位（见文档注释），后一项恒更大。
            tree_identifier_watermark: version_to_build_on
                .tree_identifier_watermark
                .max(next_available_after_this_issue),
            // F 照抄建在上面的那一版（D16（发布语义） 已定项 1：F 只升不降）。mkfs 同一个进程里那条流上它恒是 0，
            // 重开之后写行那次发布带的是恢复后生效的 F，这里接着带它。
            rollback_floor: version_to_build_on.rollback_floor,
        },
        None,
        tree_identifiers,
    )
}
```

### crates/singlefs-core/src/recovery.rs:651-680（allocation_records_of_version_without_file）

```rust
/// 树表 0 条的那一版自己那棵分配记录树（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））：
/// 指针全零 ⇒ `None`，那是 mkfs 的第 0 代（那一版的账由实例表与树表两条指针直接算）。
///
/// # Errors
/// 分配记录树根读不到、解不开、不止一层；条目宽或结构值判红（见 `allocation_records_of_node`）。
pub fn allocation_records_of_version_without_file(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Option<Vec<AllocationRecord>>, RecoveryFailure> {
    if root.allocation_record_tree_root == NodePointer::empty_root() {
        return Ok(None);
    }
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let allocation_bytes = read_unit_via_locations(
        reader,
        &root.allocation_record_tree_root.locations,
        node_bytes,
    )?;
    let allocation_node =
        parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "分配记录树根",
        })?;
    // 与 `allocation_records_under_root` 同一条：第一版的分配记录树只有一个节点（层 0）。
    if allocation_node.level != 0 {
        return Err(RecoveryFailure::UnitMalformed {
            what: "分配记录树根不止一层",
        });
    }
    allocation_records_of_node(reader, &allocation_node).map(Some)
}
```

### crates/singlefs-core/src/recovery.rs:1205-1313（replay_journal）

```rust
/// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
pub fn replay_journal(
    reader: &dyn PoolReader,
    root: &RootRecord,
    ring_bytes: u64,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    verify_named_units: bool,
    rollback_high_water: Option<u64>,
) -> (JournalScanReport, RootRecord) {
    let mut report = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    let water = (root.instance, root.checkpoint_txg);
    let mut rebuilt = *root;
    // 前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 第 1 条）：链从所选根覆盖的最后一条记录之后接，
    // 下一条的实例代号与所选根不同即停——所以只有所选根自己那个实例的记录是候选；所选根是 mkfs 的第 0 代根时一条都不施加。
    let mut above: Vec<&JournalRecord> = records
        .values()
        .filter(|record| {
            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water
        })
        .collect();
    above.sort_by_key(|record| (record.instance, record.counter));
    report.above_water = above.len();
    let in_flight_limit =
        usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
    // 链首接在所选根自己那条记录（同实例、checkpoint_txg 相等）之后；那条记录读不出（两份都撕了）时不知道它的 jsn，
    // 链首只能是 checkpoint_txg = 根的 txg + 1 的那条（第一版一次发布一条记录、txg 每次加一）：水位之上最小的那条若 txg 更大，
    // 中间就少了一条，断号即止照样成立（里程碑「第二个事务」步 3 三方第一轮攻方腿打中：无锚点时无条件接上会跳过撕掉的一条）。
    let root_own_record_counter = records
        .values()
        .find(|record| {
            record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
        })
        .map(|record| record.counter);
    let mut expected_next: Option<(InstanceGeneration, u64)> =
        root_own_record_counter.map(|counter| (root.instance, counter + 1));
    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
    for record in above.into_iter().take(in_flight_limit) {
        if let Some(expected_key) = expected_next {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        } else if record.checkpoint_txg != chain_start_txg_without_anchor {
            break;
        }
        // 前缀第五条：所选根的实例有回退行时只施加到回退行的 W 为止——W = 0 就是「之后的一个都不算」，
        // 空发布（事务号 0）也不许把根推过 T_old。
        if let Some(high_water) = rollback_high_water {
            if high_water == 0 || record.transaction > high_water {
                break;
            }
        }
        expected_next = Some((record.instance, record.counter + 1));
        // ⚠️ 这一步只对「一事务一条」成立：D23（journal 的角色与格式） 已定项 7 允许一个事务跨多条记录，那时除最后一条外
        // 都不带提交标记，扫到第一条不带的就停，会把已经提交的多条事务整个丢掉。已定项 7 的恢复算法原话是「丢掉提交标记
        // 还没出现的那个事务的全部记录」。改它归并行线一的实现（`.claude/kb/milestone/02-second-txn.md` 增补 2 收口表第 36 行）。
        if !record.is_commit {
            break;
        }
        let all_verified = !verify_named_units
            || record.named.iter().all(|named| {
                let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
                    return false;
                };
                named.locations.iter().all(|location| {
                    reader
                        .read(
                            location.device,
                            location.slot.to_device_offset(),
                            unit_bytes,
                        )
                        .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
                })
            });
        if !all_verified {
            report.verification_failed += 1;
            break;
        }
        // 只数**真验过**的：`verify_named_units` 关掉时上面那句把 `all_verified` 短路成真、一个单元都没读，
        // 这一格再加一就等于宣称验过了（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测——
        // 两臂在健康镜像上报出逐字相同的数，运行时就看不出走了哪一条）。
        if verify_named_units {
            report.verification_passed += 1;
        }
        report.prefix_applied += 1;
        report.maximum_applied_transaction =
            report.maximum_applied_transaction.max(record.transaction);
        rebuilt = RootRecord {
            filesystem_identifier: rebuilt.filesystem_identifier,
            instance: record.instance,
            checkpoint_txg: record.checkpoint_txg,
            tree_table: record.new_tree_table,
            tree_identifier_watermark: record.new_tree_identifier_watermark,
            rollback_floor: record.new_rollback_floor,
            instance_table: rebuilt.instance_table,
            mapping_root: record.new_mapping_root,
            // 与实例表指针同一条理由：新根段（D23（journal 的角色与格式） 已定项 15）里没有这一项，施加记录只能照抄被施加的那条根的。
            // 施加一条写行记录而它的根槽没落盘时，重建出来的这一版仍指着上一版的分配记录树——那正是「这次写行没有成立」该有的账。
            allocation_record_tree_root: rebuilt.allocation_record_tree_root,
        };
    }
    (report, rebuilt)
}
```

### crates/singlefs-core/src/mount.rs:553-576（allocator_of_version_without_file）

```rust
fn allocator_of_version_without_file<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    let Some(records) =
        allocation_records_of_version_without_file(devices, root).map_err(MountError::Recovery)?
    else {
        return format_time_allocator(device_maps, root);
    };
    let node_placement = Placement {
        slot: slot_shared_by_both_location_entries(&root.allocation_record_tree_root.locations)
            .map_err(
                |disagreement| MountError::FormatTimeUnitLocationsOnDifferentSlots {
                    unit: TransactionUnit::AllocationTree,
                    disagreement,
                },
            )?,
        span: TransactionUnit::AllocationTree.span_slots(),
    };
    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, records);
    allocator.note_allocation_record_node_of_the_version_without_file(node_placement);
    Ok(allocator)
}
```

### crates/singlefs-core/src/mount.rs:578-631（format_time_allocator）

```rust
/// mkfs 写出的那一版（或照抄它的暖机根）的账：盘上没有分配记录树，这一版的全部落点就是根记录直接指着的实例表与树表两个单元
/// ——mkfs 写在单元区里的那两个（字节表五里它们两盘各一条、分配代 0）。与 mkfs 同一个进程里第一个事务用的分配器同一个起点
/// （`PoolAllocator::mark_format_time_units`），第一个文件版本换下 mkfs 那片树表时照样把它释放。
///
/// # Errors
/// 根记录指着的实例表或树表不是 mkfs 写的那一版（诞生 txg 不是 0）⇒ `VersionWithoutFileNotWrittenByMakeFilesystem`。
///
/// ⚠️ **这一道今天的依据（2026-09-23 随 C512（树表 0 条的一版上被换下的单元记在哪） 定案重判）**：
/// 它**不再**拦任何一条正当历史。此前拦得住的那一条是「建池 → 可写挂载 → 不写文件 → 退出 → 再可写挂载（写行）→ 退出 → 第三次可写挂载」：
/// 写行那次发布换下上一版那片实例表，而那一版没有地方记这条释放，重开之后那一片就成了空闲槽。
/// 定案之后写行那次发布**建起这一版自己的分配记录树**、根指针住根记录，那条历史走 `allocator_of_version_without_file` 的第一条路、
/// 根本到不了这里（`the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool` 钉住）。
/// 走得到这里的只剩「树表或实例表不是 mkfs 写的那一版、而根记录里的分配记录树根指针又是全零」——**实现一处都写不出这种根**
/// （写行那条路径写指针，带文件的那一版的树表不是 mkfs 那一片时早就不走这条分支），所以它今天只对坏镜像与外来镜像说话。
/// 用例要造它得手抹那一项（`plant_a_root_without_its_allocation_record_tree`）。
/// 这一格仍在动分配器、动盘之前拒绝，盘上逐字节不变。
///
/// 两个指针里任一条的两条位置条目槽号不等 ⇒ `FormatTimeUnitLocationsOnDifferentSlots`。两样都在动分配器之前返回。
fn format_time_allocator(
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    if root.instance_table.head.birth_txg != CheckpointTxg(0)
        || root.tree_table.head.birth_txg != CheckpointTxg(0)
    {
        return Err(MountError::VersionWithoutFileNotWrittenByMakeFilesystem {
            root: RollbackTarget {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
            },
            instance_table_birth_txg: root.instance_table.head.birth_txg,
            tree_table_birth_txg: root.tree_table.head.birth_txg,
        });
    }
    let placement_of = |pointer: &crate::pointer::NodePointer, identity: TransactionUnit| {
        let slot =
            slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
                MountError::FormatTimeUnitLocationsOnDifferentSlots {
                    unit: identity,
                    disagreement,
                }
            })?;
        Ok::<Placement, MountError>(Placement {
            slot,
            span: identity.span_slots(),
        })
    };
    let instance_table_placement =
        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;
    let tree_table_placement = placement_of(&root.tree_table, TransactionUnit::TreeTable)?;
    let mut allocator = PoolAllocator::new(device_maps);
    allocator.mark_format_time_units(instance_table_placement, tree_table_placement);
    Ok(allocator)
}
```

### crates/singlefs-core/src/allocator.rs:614-645（PoolAllocator）

```rust
/// 池级分配器：每块盘按同一条规则各自取，各盘取得一致才给号（盘可以不等大，D2（RAID 条带策略） 已定项 2）。
#[derive(Clone, Debug)]
pub struct PoolAllocator {
    pub devices: Vec<DeviceFreeMap>,
    /// 开放聚簇段的起点与 bump 游标（只在内存）。`None` = 这会儿没有段可 bump（还没开过，或上一次落点走了回落）。
    open_segment: Option<SlotNumber>,
    bump_cursor: u64,
    /// 这次挂载开过的聚簇段的起点，开过就一直算数（只在内存，重开后按 D3（空间分配） 已定项 10 ①「挂载后新开一段」重来）。
    /// 用户数据的候选集排除它们全部（已定项 10 ②「不在任何开放的聚簇段里」），不只排除当前那一个——回落把 `open_segment` 置空之后
    /// 那一段仍装着这次提交的内生块，D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」照样压着它（增补 2 第 20c 行，
    /// 代码三方第一轮打中）。段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块。
    cluster_segments: BTreeSet<SlotNumber>,
    /// C146（无空段时的回落政策全仓无定义） ② 的运行时计数：实际落点与政策函数不一致的次数，第一个事务恒 0。
    /// 落点按设备取之后，发出去的落点就是每块盘各自的政策答案（答得不同在动状态之前拒绝），这个数按构造恒 0；
    /// 此前它比的是「盘 0 的答案」与「各盘答案的最小值」。字段留着是因为发布结果与真设备二进制照报它；删还是改成别的口径，没有定。
    pub policy_mismatches: u64,
    records: Vec<AllocationRecord>,
    /// 已回收、还没被复用的落点（盘上那条记录仍写着已释放；复用时那条记录被改写）。只住内存。
    reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,
    /// mkfs 写出的第 0 版树表单元的落点：第一个文件版本重写树表时把它释放（COW 换下的单元进 defer 队列，D3（空间分配） 已定项 7）；
    /// 重开之后上一版从盘上重建、释放经映射与根记录走，这里留空。
    format_time_tree_table: Option<Placement>,
    /// 树表 0 条的那一版写行时写下的分配记录树节点的落点（根记录那一项指着它，C512（树表 0 条的一版上被换下的单元记在哪））：
    /// 这一版上再发一次时（再写一次行，或发第一个文件版本）要把它换下，而那一版没有上一版的内存态可查——
    /// 与 `format_time_tree_table` 同一个用处。mkfs 的第 0 代与带文件的一版都留空。
    allocation_record_node_of_the_version_without_file: Option<Placement>,
    /// 复用窗口，只供测试的开关（`ReuseWindow`）。只住内存：重开之后按产品路径起步（`rebuild_from_records` 走 `new`）。
    reuse_window: ReuseWindow,
    /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数（逐盘各算一条，与 `reclaim_released_up_to` 的返回值口径不同）：
    /// 开关走没走到，读这个数就看得出（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    placements_reclaimed_on_release_by_the_forced_zero_reuse_window: u64,
}
```

### crates/singlefs-core/src/allocator.rs:736-743（note_allocation_record_node_of_the_version_without_file）

```rust
    /// 重开一个「树表 0 条、写过行」的池时记下它那片分配记录树节点住哪（根记录那一项给的落点）：
    /// 下一次发布要把它换下，而这一版没有上一版的内存态可查。
    pub fn note_allocation_record_node_of_the_version_without_file(
        &mut self,
        placement: Placement,
    ) {
        self.allocation_record_node_of_the_version_without_file = Some(placement);
    }
```

### crates/singlefs-core/src/make_filesystem.rs:186-369（make_filesystem）

```rust
/// 对两块设备做 mkfs。`devices` 里每块盘的身份由调用方给（位置条目按设备身份升序）。
pub fn make_filesystem<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
) -> Result<MakeFilesystemOutput, MakeFilesystemError> {
    assert_eq!(
        devices.len(),
        2,
        "第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"
    );
    let sizes: Vec<(DeviceIdentity, u64)> = devices
        .iter()
        .map(|(identity, device)| (*identity, device.size_in_bytes()))
        .collect();
    check_geometry(parameters, &sizes)?;
    let identities: Vec<DeviceIdentity> = sizes.iter().map(|(identity, _)| *identity).collect();
    let instance = MKFS_INSTANCE_GENERATION;
    let genesis = CheckpointTxg(0);
    let genesis_write_order = WriteOrder {
        instance,
        transaction: 0,
    };
    // 出生序号：同一棵树（这里都是「无归属」树 0）在 checkpoint 0 里依次 0、1（D19（块指针的结构与宽度预算） 已定项 9）。
    let instance_table_sequence = BirthSequence(0);
    let tree_table_sequence = BirthSequence(1);

    let instance_table_identity = PackedIdentity {
        birth_tree: TreeIdentifier(0),
        record_type: PACKED_TYPE_INSTANCE_TABLE,
        container: 0,
        container_birth: genesis,
    };
    let instance_table_unit = build_packed_unit(
        instance_table_identity,
        u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
        &[instance_table_chain_record()],
        genesis,
        &parameters.filesystem_identifier,
        genesis_write_order,
        instance_table_sequence,
    );
    let tree_table_genesis_unit = build_index_node(
        TreeIdentifier(0),
        0,
        TREE_TABLE_KEY_WIDTH,
        &[0u8; 8],
        &[0u8; 8],
        genesis,
        &parameters.filesystem_identifier,
        instance,
        tree_table_sequence,
        u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
        &[],
    );
    // 先把根环三个区域与 journal 环整段清零（每块盘每段一次调用、录制流各一步），再写元数据。
    // 为什么 mkfs 自己做：一条 journal 记录、一条根记录合不合法只看它自己的魔数与校验和，盘上的旧字节里
    // 凑出一条合法记录的概率不是 0（上一次 mkfs 留下的就是真合法的记录），而「这块盘是新建的全零镜像」
    // 是装置的假设、真设备上不成立。
    // 根环非清不可（C484（mkfs 不清根环，同 fsid 重来旧根还择得中））：挡住旧池那几条根的只有 fsid，
    // 而 fsid 是调用方给的（`MakeFilesystemParameters::filesystem_identifier`，mkfs 自己不生成）⇒
    // 拿同一个 fsid 在用过的盘上重做 mkfs，旧池留在槽 1..7 里的根照样自证得过、解得开，
    // `choose_root` 按 (checkpoint_txg, 实例代号) 取最大还会择中它——而它指着的树表、实例表、分配记录都是旧池的账。
    // 三个区域在每块盘上都清，不按 `region_devices` 只清本盘要读的那几段：区域归属是参数、设备身份是调用方给的，
    // 按参数清要多查一次表，还会在别的盘上留下「本池 fsid、自证得过、这一版读不到」的根记录，
    // 它今天读不到全靠择根与 checker 两处各自按归属读（`recovery::visit_valid_roots`、
    // `singlefs-checker` 的 `image.rs`），任一处以后改成扫全部盘，漏就回来了。
    // 按几何清之后 mkfs 的后置条件只有一句：根环三段在每块盘上只剩这次写下的三条第 0 代根，别处全 0。
    // 次序与屏障：清零按设备内偏移升序发（根环 1 / 4 / 7 MiB，journal 环 16 MiB），几段之间不另加屏障——
    // 它们互不重叠，也不与同一段里的单元写重叠（单元区从 784 MiB 起）；真正重叠的是清根环与根槽 FUA 写
    //（区域 r 的槽 0），那一对由原有的那道屏障（单元写之后、根 FUA 之前）隔开，不靠发出次序。
    // mkfs 中途崩溃第一版没有条款（层 0 从 mkfs 之后的池起枚举，`.claude/kb/layout/01-first-txn.md` 八
    // mkfs 那一行的「层 0 枚举」格写「不在」），不为一个没有条款的语义加屏障。
    // 清的是 S 个槽那么长的一段，S 取 mkfs 参数里的那个——这一次写进系统配置的也是它，
    // 于是「mkfs 清了多长」与「挂载时按多长读」出自同一个数。
    let root_ring_region_bytes = region_length_in_bytes(
        parameters.geometry.fixed_structure_slot_spacing,
        parameters.geometry.root_ring_slots_per_region,
    );
    let journal_ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    for (_, device) in devices.iter_mut() {
        for region_to_clear in 0..ROOT_RING_REGIONS {
            device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;
        }
        device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;
    }
    for (_, device) in devices.iter_mut() {
        device.write_at(
            INSTANCE_TABLE_SLOT.to_device_offset(),
            &instance_table_unit,
            WriteDurability::Plain,
        )?;
        device.write_at(
            TREE_TABLE_GENESIS_SLOT.to_device_offset(),
            &tree_table_genesis_unit,
            WriteDurability::Plain,
        )?;
    }
    for (_, device) in devices.iter_mut() {
        device.barrier()?;
    }

    let root = RootRecord {
        filesystem_identifier: parameters.filesystem_identifier,
        instance,
        checkpoint_txg: genesis,
        tree_table: NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: genesis,
            },
            locations: location_entries(
                &identities,
                TREE_TABLE_GENESIS_SLOT,
                &tree_table_genesis_unit,
            ),
            instance,
            birth_sequence: tree_table_sequence,
        },
        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: genesis,
            },
            locations: location_entries(&identities, INSTANCE_TABLE_SLOT, &instance_table_unit),
            instance,
            birth_sequence: instance_table_sequence,
        },
        mapping_root: NodePointer::empty_root(),
        // mkfs 的第 0 代不写分配记录树：这一版的账由实例表与树表两条指针直接算得出
        // （`mount::format_time_allocator`，字节表五里 m1 / m2 两条分配代 0 的记录）。
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let root_slot = root.to_slot(root_slot_bytes);
    for region in 0..ROOT_RING_REGIONS {
        let region_device = parameters.region_devices[usize::try_from(region).expect("区域号")];
        let (_, device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == region_device)
            .expect("check_geometry 核过区域归属");
        let offset = slot_offset(
            RootRingSlot { region, slot: 0 },
            parameters.geometry.fixed_structure_slot_spacing,
        );
        device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;
    }

    for (identity, device) in devices.iter_mut() {
        let system_configuration = SystemConfiguration {
            immutable: SystemImmutableConfiguration {
                filesystem_identifier: parameters.filesystem_identifier,
                this_device: *identity,
                device_count: u32::try_from(identities.len()).expect("设备数"),
                region_devices: parameters.region_devices,
                sizes: parameters.geometry,
            },
            mutable: SystemMutableConfiguration,
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation: SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,
                journal_tail: 0,
                journal_instance: instance,
            },
        };
        let slot = system_configuration.to_slot();
        for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = DeviceOffsetInBytes(
                slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing),
            );
            device.write_at(offset, &slot, WriteDurability::Plain)?;
        }
    }
    for (_, device) in devices.iter_mut() {
        device.barrier()?;
    }
    let _ = JOURNAL_RECORD_BYTES;
    Ok(MakeFilesystemOutput {
        root,
        instance_table_unit,
        tree_table_genesis_unit,
    })
}
```

### crates/singlefs-checker/src/walk.rs:390-433（walk_root）

```rust
    fn walk_root(&mut self, record: &[u8], is_newest: bool) {
        // 实例表单元：根记录直接持有（自证单元，I-1.3 豁免）。
        let instance_pointer = parse_node_pointer(&record[170..256]);
        judge_location_order(&mut self.judgements, &instance_pointer, "实例表指针");
        for location in &instance_pointer.locations {
            self.note_reference(location.device, location.slot, 2, "实例表单元");
        }
        if let Some(unit) = read_referenced_unit(
            self.reader,
            &mut self.judgements,
            &instance_pointer.locations,
            data_unit_bytes(),
            "实例表单元",
        ) {
            if self.visited_units.insert((
                instance_pointer.locations[0].device,
                instance_pointer.locations[0].slot,
            )) {
                let records =
                    self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
                if is_newest {
                    if let Some(records) = records {
                        let mount_root_instance =
                            u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
                        self.judge_instance_table_rows(&records, mount_root_instance);
                    }
                }
            }
        } else {
            self.walk_failures
                .push("实例表单元两份都读不到对得上的".to_string());
        }
        // 树表 0 条那一版的分配记录树的根住根记录（C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案）：
        // 那一版没有树表条目可放，也还没登记过任何树 ⇒ 那一片的树 ID 是 0（由根记录独占持有，同树表单元与实例表单元）。
        // 带文件的一版这一项恒全零，`read_index_node` 对全零指针直接交 `None`。
        self.read_index_node(
            &record[342..428],
            0,
            KEY_SCHEMA_ALLOCATION,
            "树表 0 条那一版的分配记录树的根",
        );
        // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
        self.walk_tree_table_and_central_mapping_root(&record[36..122], &record[256..342]);
    }
```

### crates/singlefs-checker/src/walk.rs:1151-1218（references_of_root）

```rust
/// 从一条根走一遍，取它的树表条目；`depth` 是「走到叶」时再取它引用的全部落点与分配记录。只读不判。
fn references_of_root(
    reader: &dyn ImageReader,
    record: &[u8],
    depth: ReferenceScanDepth,
    cache: &mut IndexNodeCache,
) -> RootReferences {
    let mut references = RootReferences {
        placements: BTreeSet::new(),
        tree_table_birth_txg: BTreeMap::new(),
        tree_table_placement: None,
        allocation_records: Vec::new(),
        depth,
        is_complete: true,
    };
    let instance_table = parse_node_pointer(&record[170..256]);
    references.note(&instance_table);
    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。映射条目指的单元与树里引用的是同一批，不重复数——
    // 主走读的 `note_reference` 也不数它们（数了 I-3.1（已分配统计对得上） 与 I-5.1（引用不重叠） 会把同一个落点算两遍）。
    let mapping_root = parse_node_pointer(&record[256..342]);
    references.note(&mapping_root);
    // 树表 0 条那一版的分配记录树的根（C512（树表 0 条的一版上被换下的单元记在哪））：它的落点同样只有根记录指着，
    // 不数它，被抛弃根的影子账与 I-3.9 的引用集合就少一片。
    let allocation_record_tree_root = parse_node_pointer(&record[342..428]);
    references.note(&allocation_record_tree_root);
    let tree_table_pointer = parse_node_pointer(&record[36..122]);
    references.note(&tree_table_pointer);
    if tree_table_pointer.all_zero {
        references.is_complete = false;
        return references;
    }
    references.tree_table_placement = Some((
        tree_table_pointer.locations[0].device,
        tree_table_pointer.locations[0].slot,
    ));
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
    else {
        references.is_complete = false;
        return references;
    };
    for entry in &tree_table.entries {
        // 条目宽是节点头里的一个字段：头校验和过、载荷 CRC 过而条目宽被改窄的镜像也要能判（那由 I-1.7 / I-1.1 说话），
        // 这一遍只如实记「数不全」，不按短条目去解字段。
        if entry.len() < tree_table_entry_bytes() {
            references.is_complete = false;
            continue;
        }
        let tree = read_u64(entry, 0);
        references
            .tree_table_birth_txg
            .insert(tree, read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET));
        let tree_root = parse_node_pointer(&entry[14..100]);
        if tree_root.all_zero {
            // day-1 注册、还没有根节点的树（livelist 6、稀疏旁表 7、deadlist 8）：不占落点。
            continue;
        }
        references.note(&tree_root);
        if depth == ReferenceScanDepth::TreeTableOnly {
            continue;
        }
        let Some(node) = read_index_node_without_judging(reader, &tree_root, cache) else {
            references.is_complete = false;
            continue;
        };
        collect_tree_references(&mut references, &node, read_u16(entry, 10));
    }
    references
}
```

### crates/singlefs-checker/src/walk.rs:2669-3071（check_pool_image）

```rust
/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    let system_configurations = chosen_system_configurations(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = system_configurations
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
    if chosen.is_empty() {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里没有一块盘交得出有效的系统配置：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    let geometry = chosen[0].2;
    // 各盘择到的系统配置要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的系统配置 fsid 与盘 {} 的不同：这块盘不属于这个池",
                    chosen[0].0
                )
            },
        );
    }
    let devices = reader.devices();
    let regions = usize::try_from(geometry.regions).expect("R");
    let region_devices = &geometry.region_devices[..regions.min(3)];
    let distinct: BTreeSet<u32> = region_devices.iter().copied().collect();
    let heaviest = devices
        .iter()
        .map(|device| {
            region_devices
                .iter()
                .filter(|candidate| *candidate == device)
                .count()
        })
        .max()
        .unwrap_or(0);
    let pigeonhole = regions.div_ceil(devices.len().max(1));
    root_ring_judgements.judge(
        "I-7.6",
        distinct.len() == regions.min(devices.len()) && heaviest <= pigeonhole && region_devices.iter().all(|device| devices.contains(device)),
        || format!("根环区域的设备 {region_devices:?}：不同值 {}、最重的盘背 {heaviest} 个（上界 {pigeonhole}）", distinct.len()),
    );
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("8 字节"),
    );
    let roots = valid_roots(reader, &geometry);
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    // I-8.6、I-8.7 与 I-8.8 只读 journal 环，不读根：根环全灭的镜像上它们照样判得了（那一格归 I-7.1）。
    let journal_records_by_device =
        scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);
    judge_journal_back_chain(&journal_records_by_device, &mut root_ring_judgements);
    judge_transaction_numbers_per_instance(&journal_records_by_device, &mut root_ring_judgements);
    judge_commit_markers_per_transaction(&journal_records_by_device, &mut root_ring_judgements);
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        root_ring_judgements.not_applicable(
            "I-7.3",
            "根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",
        );
        return root_ring_judgements.into_report();
    }
    judge_root_ring_health(&roots, &mut root_ring_judgements);
    let newest_index = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
        .map(|(index, _)| index)
        .expect("非空");
    let mut walk = Walk {
        reader,
        judgements: root_ring_judgements,
        filesystem_identifier_low,
        references: BTreeMap::new(),
        visited_units: BTreeSet::new(),
        walk_failures: Vec::new(),
        tree_identifiers_in_tables: BTreeSet::new(),
        inode_object_birth: BTreeMap::new(),
        inode_tree_walked: false,
        largest_inode_number_in_the_inode_tree: None,
        data_unit_objects: Vec::new(),
        accounting: BTreeMap::new(),
        accounting_seen: false,
        instance_table_rows: Vec::new(),
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        referenced_units_judged_against_their_pointer: 0,
    };
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    let newest_failures = walk.walk_failures.clone();
    // I-4.8（近 K 代根校验和自洽）与 I-7.4（近 K 代块未被复用）：候选集里任一根（最新根也在候选集里）出发遍历，所有块的校验和
    // 都与父指针一致、走读不断——最新根那一次各算一格；候选集只剩最新根时两条都还判得到（本地攻方腿：全称量词在单元素集合上照样成立）。
    let newest_txg = roots[newest_index].2.checkpoint_txg;
    let newest_walked_into_reused_or_erased_unit =
        !newest_failures.is_empty() || walk.judgements.violation_count("I-2.1") > 0;
    walk.judgements
        .judge("I-4.8", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）出发的遍历有单元对不上或读不出")
        });
    walk.judgements
        .judge("I-7.4", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）")
        });
    let accounting_seen = walk.accounting_seen;
    let accounting = walk.accounting.clone();
    // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
    // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
    // 再加一条：txg ≥ 最新根带的回退下界 F（D16（发布语义） 已定项 1 的回退候选集）；F 之下的根引用的单元可以已被回收复用，
    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
    let instance_table_rows = walk.instance_table_rows.clone();
    let newest_rollback_floor = u64::from_le_bytes(
        roots[newest_index].2.record_bytes[130..138]
            .try_into()
            .expect("8 字节"),
    );
    // 掉出遍历的根槽按理由各记一次（两样都占的两边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
    let mut root_slots_dropped_as_abandoned = 0u64;
    let mut root_slots_dropped_below_floor = 0u64;
    let candidate_indexes: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(index, (_, _, root))| {
            let abandoned = instance_table_rows.iter().any(|row| {
                row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
            });
            let below_floor = root.checkpoint_txg < newest_rollback_floor;
            let walked = *index == newest_index || (!abandoned && !below_floor);
            if !walked {
                if abandoned {
                    root_slots_dropped_as_abandoned += 1;
                }
                if below_floor {
                    root_slots_dropped_below_floor += 1;
                }
            }
            walked
        })
        .map(|(index, _)| index)
        .collect();
    for index in candidate_indexes.iter().copied() {
        let (_, _, root) = &roots[index];
        if index != newest_index {
            let mismatches_before = walk.judgements.violation_count("I-2.1");
            let failures_before = walk.walk_failures.len();
            walk.walk_root(&root.record_bytes, false);
            // 这条候选根引用的单元有一个校验和对不上或头用不了，就是它指着的块被复用或抹头了（I-7.4（近 K 代块未被复用）），
            // 从它出发的遍历也就不自洽（I-4.8（近 K 代根校验和自洽））；两条按每条候选根各判一格。
            let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
                > mismatches_before
                || walk.walk_failures.len() > failures_before;
            let root_txg = root.checkpoint_txg;
            walk.judgements
                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                    format!(
                        "候选根 txg {root_txg} 引用的单元已被复用或抹头（校验和对不上或头用不了）"
                    )
                });
            walk.judgements
                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                    format!("候选根 txg {root_txg} 出发的遍历有单元对不上或读不出")
                });
        }
    }
    // 回退候选集补上「由记录施加出来、根槽从没落盘的那一版」（2026-09-23 用户定候选 b）：它们没有根槽，
    // 上面按根环走的那一遍走不到，而它们换下的单元还在 defer 里、仍算在已分配里（增补 2 收口表第 54 行那 12 个状态差的 65 536 字节）。
    // 走法与判定照候选根：引用进同一个并集（I-3.1 / I-5.1），单元照判 I-2.1 等，I-7.4 / I-4.8 每一版各判一格。
    let versions_applied_only_by_records = versions_applied_only_by_records(
        &journal_records_by_device,
        &roots,
        &instance_table_rows,
        newest_rollback_floor,
    );
    for version in &versions_applied_only_by_records {
        let mismatches_before = walk.judgements.violation_count("I-2.1");
        let failures_before = walk.walk_failures.len();
        walk.walk_version_applied_only_by_records(version);
        let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
            > mismatches_before
            || walk.walk_failures.len() > failures_before;
        let (instance, version_txg) = (version.instance, version.checkpoint_txg);
        walk.judgements
            .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）"
                )
            });
        walk.judgements
            .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）出发的遍历有单元对不上或读不出"
                )
            });
    }
    let referenced_units_judged_against_their_pointer =
        walk.referenced_units_judged_against_their_pointer;
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
    // I-1.2 与 I-4.2 在遍历方向上判的是同一批单元：走读一个被引用单元都没走到时两条都没有对象
    // （根槽读得出而树表指针全零、或每个单元的头都用不了），报「不适用」并带理由，不报成立。
    if referenced_units_judged_against_their_pointer == 0 {
        for invariant in ["I-1.2", "I-4.2"] {
            judgements.not_applicable(
                invariant,
                "走读一个被引用的单元都没读到（实例表单元按 I-1.2 那一行的例外不进这两条）：出生身份与已发布谓词都没有对象",
            );
        }
    }
    // I-1.8：扫描方向按已发布谓词过滤之后归并成组。谓词要挂载根的 (实例代号, txg) 与它指着的那一版实例表，
    // 所以这一遍排在走读之后（`instance_table_rows` 是走最新根时取的）。
    let published = PublishedPredicate {
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        instance_table_rows: &instance_table_rows,
    };
    let content_units =
        scanned_content_units(reader, &geometry, filesystem_identifier_low, &published);
    if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {
        judgements.not_applicable(
            "I-1.8",
            "扫描方向没有两份归并到一起、或两组归并到同一个 key 的码 1 / 码 3 已发布单元：归并与定序都比不出来",
        );
    }
    // 这两遍各自按候选集里每条根读树表与树节点：共用一份按位置条目记的缓存，候选根常指着同一个单元，层 0 每个崩溃状态都跑。
    let mut index_node_cache = IndexNodeCache::new();
    judge_release_generation_and_tree_table_birth(
        reader,
        &roots,
        &candidate_indexes,
        newest_index,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_allocation_records_disjoint(
        reader,
        &roots,
        &candidate_indexes,
        &mut index_node_cache,
        &mut judgements,
    );
    let allocation_node_pointers = allocation_record_node_pointers_of_the_candidate_versions(
        reader,
        &roots,
        &candidate_indexes,
        &versions_applied_only_by_records,
        &mut index_node_cache,
    );
    judge_allocation_generations_against_unit_births(
        reader,
        &allocation_node_pointers,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
    // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
    // 另一半（> 全部已发布的墓碑记录的对象 ID）今天没有对象：墓碑是打包记录类型 1，这一版一片都不写
    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
    let inode_number_watermark = accounting
        .get(&(STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION))
        .copied();
    match (
        inode_number_watermark,
        walk.largest_inode_number_in_the_inode_tree,
    ) {
        (Some(watermark), Some(largest_inode)) => {
            judgements.judge("I-9.6", watermark > largest_inode, || {
                format!(
                    "记账里的 inode 号水位 {watermark} 不大于 inode 树内最大 key {largest_inode}"
                )
            });
        }
        (None, _) => judgements.not_applicable(
            "I-9.6",
            "最新的根下面没有记账树、或记账里没有 inode 号水位那一行",
        ),
        (Some(_), None) => {
            judgements.not_applicable("I-9.6", "最新的根下面走不到 inode 树里的任何一条记录")
        }
    }
    if !walk.inode_tree_walked {
        judgements.not_applicable(
            "I-9.12",
            "最新的根下面走不到 inode 树的内部节点（树表 0 条或根读不出）",
        );
    }
    for (inode, unit_object_birth, what) in &walk.data_unit_objects {
        if let Some(record_birth) = walk.inode_object_birth.get(inode) {
            judgements.judge("I-9.10", record_birth == unit_object_birth, || {
                format!(
                    "{what} 的对象出生代 {unit_object_birth} 与 inode 记录的 {record_birth} 不符"
                )
            });
        }
    }
    // I-5.1：同一块盘上，不同的引用不许占重叠的槽（两条位置条目落在不同盘上是显式的多副本）。
    let mut per_device: BTreeMap<u32, Vec<(u64, u64, &String)>> = BTreeMap::new();
    for ((device, slot, span), what) in &walk.references {
        per_device
            .entry(*device)
            .or_default()
            .push((*slot, *span, what));
    }
    for (device, mut ranges) in per_device.clone() {
        ranges.sort_unstable_by_key(|(slot, span, _)| (*slot, *span));
        for pair in ranges.windows(2) {
            let (first_slot, first_span, first_what) = pair[0];
            let (second_slot, _, second_what) = pair[1];
            judgements.judge("I-5.1", first_slot + first_span <= second_slot, || {
                format!("盘 {device}：{first_what}（槽 {first_slot} 跨 {first_span}）与 {second_what}（槽 {second_slot}）重叠")
            });
        }
    }
    // I-3.1 判红时，说明文字里带一段机理标识：遍历覆盖了哪些根、剩下的按什么理由没覆盖。
    // 判读的一方（`singlefs-harness` 的 `history::allocation_statistic_mechanism`）按它分辨「记账多算」是哪一种机理造成的：
    // 环转过一圈把 F 之上的根挤出了环，还是回退下界把环里读得到的根挡在了外面——两者的签名（只有 I-3.1 红、记账多于遍历）
    // 一模一样，不带这一段就只能按签名认，机理不同的新问题会被「已知红」清单接走（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阴那一半）。
    let root_ring_slot_count = geometry.regions * geometry.slots_per_region;
    let oldest_readable_root_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
        .unwrap_or(0);
    let readable_root_slot_count = u64::try_from(roots.len()).expect("根槽数");
    let walked_root_slot_count = u64::try_from(candidate_indexes.len()).expect("候选根槽数");
    let walked_versions_applied_only_by_records =
        u64::try_from(versions_applied_only_by_records.len()).expect("版本数");
    // 「并进遍历的由记录施加出来的版本」那一段是 2026-09-23 候选 b 加的；判读的一方按字段名取数（`leading_number_after`），
    // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
    let mechanism = move || {
        format!(
            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {newest_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
        )
    };
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    if accounting_seen {
        for device in &devices {
            let walked: u64 = per_device.get(device).map_or(0, |ranges| {
                ranges.iter().map(|(_, span, _)| span * SLOT_BYTES).sum()
            });
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            judgements.judge("I-3.1", allocated == Some(walked), || {
                format!(
                    "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}{}",
                    mechanism()
                )
            });
            let capacity = reader
                .device_bytes(*device)
                .map(|bytes| (bytes / SLOT_BYTES - geometry.unit_area_start_slot) * SLOT_BYTES);
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free + allocated == capacity), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（全部码 2 单元头 ∪ 走过的树表条目）。
    let watermark = roots
        .iter()
        .map(|(_, _, root)| root.tree_identifier_watermark)
        .max()
        .unwrap_or(0);
    let newest_published_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let mut seen = scanned_tree_identifiers(reader, &geometry, newest_published_txg);
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    judgements.into_report()
}
```

### crates/singlefs-checker/src/walk.rs:1439-1495（judge_release_generation_and_tree_table_birth）

```rust
/// I-3.9 与 I-9.14 共用的这一遍：候选集里每条根各走一遍取引用集合与树表。候选集只剩一条根时两条都报「不适用」——
/// 「最早不再引用它的那条有效根」与「跨根相同」都要第二条根才有内容（2026-09-18 用户定案随 C374 立条时定的口径）。
fn judge_release_generation_and_tree_table_birth(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    newest_index: usize,
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    if candidate_indexes.len() < 2 {
        judgements.not_applicable(
            "I-3.9",
            "回退候选集里只有一条根：没有第二条根能见证「不再引用这个落点」",
        );
        judgements.not_applicable(
            "I-9.14",
            "回退候选集里只有一条根：树表条目没有第二条根的那一份可比",
        );
        return;
    }
    // 最新根那棵账定这一遍走多深：它一条已释放记录都没有时 I-3.9 判不了，别的根只要树表（I-9.14 那一半）。
    let newest = references_of_root(
        reader,
        &roots[newest_index].2.record_bytes,
        ReferenceScanDepth::EveryReferencedPlacement,
        cache,
    );
    let depth = if newest
        .allocation_records
        .iter()
        .any(|record| record.is_released)
    {
        ReferenceScanDepth::EveryReferencedPlacement
    } else {
        ReferenceScanDepth::TreeTableOnly
    };
    let mut scanned: Vec<ScannedCandidateRoot> = Vec::with_capacity(candidate_indexes.len());
    for index in candidate_indexes.iter().copied() {
        scanned.push(ScannedCandidateRoot {
            root_index: index,
            checkpoint_txg: roots[index].2.checkpoint_txg,
            references: if index == newest_index {
                newest.clone()
            } else {
                references_of_root(reader, &roots[index].2.record_bytes, depth, cache)
            },
        });
    }
    scanned.sort_unstable_by_key(|root| root.checkpoint_txg);
    let newest_position = scanned
        .iter()
        .position(|root| root.root_index == newest_index)
        .expect("最新根在回退候选集里（`check_pool_image` 无条件把它放进去）");
    judge_release_generations(&scanned, newest_position, judgements);
    judge_tree_table_birth_txg(&scanned, judgements);
}
```

### crates/singlefs-checker/src/lib.rs:219-228（RootView）

```rust
/// 根记录槽解出来的要紧字段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootView {
    pub instance: u32,
    pub checkpoint_txg: u64,
    pub tree_identifier_watermark: u64,
    pub rollback_floor: u64,
    /// 457 字节记录本身（含校验和字段），三个区域比对用。
    pub record_bytes: Vec<u8>,
}
```

### crates/singlefs-checker/src/lib.rs:230-233（check_root_slot）

```rust
/// 判一个根槽（判定宽度那么宽）。
pub fn check_root_slot(
    slot: &[u8],
    expected_filesystem_identifier: &[u8; 16],
```

### crates/singlefs-harness/src/model.rs:130-162（rewritten_roles）

```rust
    fn rewritten_roles(self) -> &'static [ModelUnitRole] {
        match self {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => &[
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::RowsOnFileVersion => &[
                ModelUnitRole::InstanceTable,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::EmptyOnFileVersion => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 树表 0 条的一版上写行：实例表，加这一版自己那片分配记录树
            // （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案：根指针住根记录、不进树表）。
            ModelPublishKind::RowsOnVersionWithoutFile => {
                &[ModelUnitRole::InstanceTable, ModelUnitRole::AllocationTree]
            }
            ModelPublishKind::ZeroUnit => &[],
        }
    }
```

### crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:590-737（rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content）

```rust
/// 验收第一条：回退行 (1, 3, 0)、中间实例行 (2, 0, 0)；D 的根 (3, 9)、jsn 9（接在环里最大的 jsn 8 之后，C340 取 P2）、事务号 0、反向链 0，
/// 重写实例表 + 四个固定点单元；暖机一次落到另一块盘；一条记录都不施加；冷启动读回第一次的内容；被抛弃根独占的槽逐盘 34 个、
/// D 与暖机一个都不落在上面；checker 全绿（I-3.1 的并集按实例表把被抛弃的根排除，I-3.8 看见回退行）。
#[test]
fn rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content(
) {
    let mut pool = build_through_third_publish("step-four-rollback");
    let third = pool.output.clone();
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    let output = &rolled_back.output;
    assert_eq!(output.instance, InstanceGeneration(3));
    assert_eq!(
        output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(3),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "回退行与中间实例行"
    );
    assert_eq!(output.journal.prefix_applied, 0, "A 之后的记录一条都不施加");
    assert_eq!(
        (
            output.effective_root.instance,
            output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3))
    );
    let rollback_publish = output
        .row_publish
        .file_version()
        .expect("回退到带文件的 A：D 是带文件的一版");
    assert_eq!(
        (
            rollback_publish.root.instance,
            rollback_publish.root.checkpoint_txg,
            rollback_publish.record.counter,
            rollback_publish.record.transaction,
            rollback_publish.record.back_chain
        ),
        (InstanceGeneration(3), CheckpointTxg(9), 9, 0, 0),
        "D：txg = max(根环 8, 记录 8) + 1；jsn 接在环里最大的 8 之后（C340 取 P2，被抛弃的记录一条不盖）；本实例第一条反向链 0"
    );
    assert_eq!(
        rollback_publish.rewritten,
        vec![
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
    );
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "D 落盘 0、txg 10 落盘 1，一次就够"
    );
    let warm_up = output.warm_up_publishes[0]
        .file_version()
        .expect("带文件的一版上的暖机");
    assert_eq!(warm_up.root.checkpoint_txg, CheckpointTxg(10));
    assert_ne!(region_device(9), region_device(10));
    assert_eq!(warm_up.record.counter, 10);

    // 影子账：被抛弃的根 B、(2, 5)、(2, 6)、(2, 7)、C 引用而 A 不引用的槽——B 10、写行 6、暖机 4 + 4、C 10 = 34 个槽，逐盘。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let abandoned: BTreeSet<u64> = slots_of(&third, device)
            .difference(&slots_of(rollback_publish, device))
            .copied()
            .collect();
        assert_eq!(abandoned.len(), 34, "盘 {device:?} 上只被被抛弃根引用的槽");
        // 影子账按窄读法只隔离这 34 个：mkfs 实例表那 2 个槽 A（候选）与 B 都引用，不在其内。
        assert!(
            output.isolated_slots_per_device.contains(&(device, 34)),
            "隔离的槽数 = 只被被抛弃根引用的 34 个 {:?}",
            output.isolated_slots_per_device
        );
        for publish in std::iter::once(rollback_publish).chain(
            output
                .warm_up_publishes
                .iter()
                .map(|publish| publish.file_version().expect("带文件的一版上的暖机")),
        ) {
            for placement in publish.placements() {
                for slot in placement.slot.0..placement.slot.0 + placement.span {
                    assert!(
                        !abandoned.contains(&slot),
                        "txg {} 的落点 {slot} 落在被抛弃根引用的槽上",
                        publish.root.checkpoint_txg.0
                    );
                }
            }
        }
    }

    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(10)),
            content: file_content()
        },
        "{:?}",
        report.journal
    );
    assert_eq!(
        report.journal.valid_records, 10,
        "jsn 1–8 原样在（P2 一条不盖）、9 是 D、10 是暖机"
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "(3, 10) 之后 jsn 11 一条都没有"
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        if *invariant == "I-8.8" {
            // 一事务一条、每条都带提交标记：I-8.8（前缀里的事务不被切开） 的 ③ ④ 没有对象，报不适用
            // （判别力在 `checker_known_bad_images.rs`）。
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "{invariant} 在回退之后的镜像上报不适用：{verdict:?}"
            );
            continue;
        }
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在回退之后的镜像上要成立"
        );
    }
    for must_hold in ["I-3.1", "I-3.8", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}
```

### crates/singlefs-harness/tests/checker_known_bad_images.rs:2198-2243（birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant）

```rust
/// 反向那一头（C511 收窄之后仍要红的那一格）：回退之后的这条线把 extent 树的诞生 txg 一致地记成 D 那次发布的 txg
/// （回退之后在现行线上重新建树的样子），而回退目标 A 那一版仍记 3。A 的 T = 3 = 回退行的 Ti，与 D、(3, 10) 在**同一条时间线上**
/// ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 必须红，而且只红它。
///
/// 回退之后那两片树表都改：只改 D 的话 D 与 (3, 10) 之间也不一致，把 A 错判出局照样判得出，钉不住「A 在现行线上」这一条。
/// 两片都改之后唯一的不一致在 A 与回退之后这条线之间。只在 I-9.14 那一遍上把收窄做宽的（例如把「同一条时间线」读成
/// 「同一个实例」、只拿最新根那个实例的根比），I-9.14 就判不出这一格、这条用例红；C374 那份镜像上实例表一行都没有、
/// 只有一个实例，那样做宽它照样绿。做宽做在共用的候选集上的（连回退目标一起判出局），这条在干净镜像那一段先红在
/// I-3.1（已分配统计对得上） 上。
#[test]
fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-after-rollback-same-line");
    for (invariant, found) in check_pool_image(&rolled_back.image) {
        assert_eq!(
            found,
            expected_verdict_with_one_record_per_transaction(invariant),
            "回退到 A 之后的干净镜像上 {invariant} 的判定"
        );
    }
    let mut image = rolled_back.image.clone();
    for tree_table_slot in rolled_back
        .tree_table_slots_of_the_line_after_the_rollback
        .iter()
        .copied()
    {
        rewrite_the_extent_tree_birth_txg_in_tree_table(
            &mut image,
            tree_table_slot,
            rolled_back.rollback_target_txg,
            rolled_back.rollback_publish_txg,
        );
    }
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        violated,
        ["I-9.14"],
        "回退目标 A 与回退之后这条线同在现行线上，诞生 txg 不一致只该红 I-9.14：{verdicts:?}"
    );
}
```

### crates/singlefs-harness/tests/checker_known_bad_images.rs:2245-2279（birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_is_not_compared_and_every_invariant_holds）

```rust
/// 正向那一头（C511 的收窄本身）：只有被回退切掉的那条线上的 C 把 extent 树的诞生 txg 记成它自己那次发布的 txg，
/// 现行线上 A、D、(3, 10) 仍一致记 3 ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 不拿 C 比，判成立；别的每一条也都成立。
/// 这是回退到无文件那一版之后「再发一次第一个文件版本」重新建树那一格在盘上的关系（旧线与新线记的诞生 txg 不同），
/// 这里在回退到带文件的 A 那一格上造出来——回退到无文件那一版今天仍由挂载那道拒绝挡着（C511 的第 3 步没做）。
/// 把收窄去掉（I-9.14 拿根环里每一条根比，不按实例表判出局）这条就红。
#[test]
fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_is_not_compared_and_every_invariant_holds(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-good-after-rollback-abandoned-line");
    let mut image = rolled_back.image.clone();
    rewrite_the_extent_tree_birth_txg_in_tree_table(
        &mut image,
        rolled_back.tree_table_slot_of_the_abandoned_root,
        rolled_back.rollback_target_txg,
        rolled_back.abandoned_publish_txg,
    );
    let verdicts = check_pool_image(&image);
    let birth_invariant = verdicts
        .iter()
        .find(|(name, _)| *name == "I-9.14")
        .expect("清单里有 I-9.14");
    assert_eq!(
        birth_invariant.1,
        InvariantVerdict::Holds,
        "被回退切掉的 C 不与现行线比；现行线上 A 与回退之后那几片树表一致，I-9.14 要真被评估过且成立"
    );
    for (invariant, found) in &verdicts {
        assert_eq!(
            *found,
            expected_verdict_with_one_record_per_transaction(invariant),
            "只改了被抛弃根 C 那片树表里的一个诞生 txg，{invariant} 的判定"
        );
    }
}
```

### crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:626-699（the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool）

```rust
/// 建池 → 可写挂载（不写文件）→ 退出 → 再可写挂载（写行）→ 退出 → **第三次可写挂载做得成**
/// （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案：根记录加一项放分配记录树的根指针）：
/// 写行那次发布把这一版的全部分配记录写成一个节点，被换下的那片实例表带着已释放标志进了它，
/// 重开时从它重建账 ⇒ **那一片没有被当空闲槽发出去**。
/// 三条断言各盯一处：那一片在重建出来的账里带着已释放标志、第三次挂载新写的那几个单元一个都没落在它的槽上、池级 checker 一条违例都没有。
/// 判别力自证：把写行那次发布里建最小分配记录树那一步去掉（`crates/mutations.tsv` 那两行），这三条各自变红。
#[test]
fn the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool(
) {
    let mut formatted = format_pool("step-three-formatted-third-mount-succeeds");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    let root_after_the_row = *row_written.current.root();
    let swapped_out_instance_table = formatted
        .genesis
        .root
        .instance_table
        .locations
        .map(|location| location.slot);
    formatted.devices = Some(second);
    assert_ne!(
        root_after_the_row.instance_table.head.birth_txg,
        CheckpointTxg(0),
        "写过行之后实例表的诞生 txg 不再是 0"
    );
    assert_ne!(
        root_after_the_row.allocation_record_tree_root,
        NodePointer::empty_root(),
        "写行那次发布把分配记录树的根写进了根记录"
    );

    let mut third = formatted.reopen_recorded();
    let third_mount = mount_writable(&parameters(), &mut third).expect("第三次可写挂载做得成");
    let reused = third_mount
        .output
        .row_publish
        .root()
        .instance_table
        .locations
        .iter()
        .any(|location| swapped_out_instance_table.contains(&location.slot));
    formatted.devices = Some(third);
    assert!(
        !reused,
        "被换下的那片实例表（槽 {swapped_out_instance_table:?}）没有被当空闲槽发给第三次挂载写的那片"
    );
    let rebuilt_record_says_released = third_mount
        .allocator
        .records()
        .iter()
        .filter(|record| swapped_out_instance_table.contains(&record.slot))
        .all(|record| record.is_released);
    let rebuilt_record_count = third_mount
        .allocator
        .records()
        .iter()
        .filter(|record| swapped_out_instance_table.contains(&record.slot))
        .count();
    assert_eq!(rebuilt_record_count, 2, "两盘各一条记录罩着被换下的那一片");
    assert!(
        rebuilt_record_says_released,
        "重建出来的账里那两条记录带着已释放标志：写行那次发布的释放从盘上取回来了"
    );
    let verdicts = check_pool_image(&formatted.memory_pool());
    for (invariant, verdict) in &verdicts {
        assert!(
            !matches!(verdict, InvariantVerdict::Violated(_)),
            "第三次可写挂载之后 {invariant} 判红：{verdict:?}"
        );
    }
}
```
