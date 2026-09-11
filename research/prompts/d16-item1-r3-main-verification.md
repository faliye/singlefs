# D16（发布语义） 未定项 1 第三轮——主 agent 对正推腿「G′ 同样出局」的复现与机制探针

**口径**：2026-09-12 凌晨（JST）主 agent 做。正推腿（`d16-item1-r3-sonnet-output.md`）报告：把臂 G 的处置改成「做到这次删的块放回为止」（材料的 G′）之后，
近满盘上删了再写仍然失败，与原臂 G 逐种子相同。主 agent 在仓外副本（`research/` 整拷、不带 `target/`）里独立复现，并加一个探针看冷启动那一刻的账。
**这是机制探针，不是跑前登记过的实验**：只跑了 `near_full` 这一个世界，没进故障世界；「准入按最坏残留收费」那一形要当候选，得另立实验。

## 副本相对仓里源码改了什么（`diff -u`）

```diff
--- research/e7-index-bench/src/bin/e138_per_disk_floor.rs	2026-09-11 15:55:27.015560082 +0000
+++ /tmp/claude-1000/-home-fy5090-code-singlefs/86b2ac0e-7e65-418c-97a7-f3d755730fe6/scratchpad/verify-copy/research/e7-index-bench/src/bin/e138_per_disk_floor.rs	2026-09-11 16:31:53.334833010 +0000
@@ -300,6 +300,8 @@
     nonempty_txgs: HashSet<u64>,
     admin_rollback_happened: bool,
     halted: bool,
+    force_full_disposal: bool,
+    charge_worst_residue: bool,
     metrics: Metrics,
     /// 管理员回退之前盘上那一份分配状态，被抛弃时间线的根引用的是它（E135 的 abandoned_view 原样）。
     abandoned_view: Option<(Rc<PoolState>, u64)>,
@@ -340,6 +342,8 @@
             nonempty_txgs: HashSet::new(),
             admin_rollback_happened: false,
             halted: false,
+            force_full_disposal: false,
+            charge_worst_residue: false,
             metrics: Metrics::fresh(),
             abandoned_view: None,
         }
@@ -492,7 +496,8 @@
 
     /// 普通分配看得见的：扣掉保留池。
     fn user_allocatable(&self) -> u64 {
-        self.reusable_count().saturating_sub(self.reserve_blocks)
+        let plain = self.reusable_count().saturating_sub(self.reserve_blocks);
+        if self.charge_worst_residue { plain.min(self.df_guaranteed()) } else { plain }
     }
 
     /// 取 count 块：先取可再分配的已释放块，再取没用过的。复用一块就把它上一段区间记成损坏。
@@ -824,7 +829,7 @@
                 return WindowOutcome::Halted;
             }
             let committed_txg = self.newest_durable_txg;
-            let publications = 1 + self.push_after_commit(committed_txg, true);
+            let publications = 1 + self.push_after_commit(committed_txg, !self.force_full_disposal);
             self.metrics.maximum_publications_per_admission = self.metrics.maximum_publications_per_admission.max(publications);
             if self.halted {
                 return WindowOutcome::Halted;
@@ -1611,6 +1616,79 @@
         assert!(simulation.pending_floor.is_some(), "新 F 还要照生效条件继续等");
     }
 
+
+    fn ledger(simulation: &Simulation) -> String {
+        let signed = simulation.df_raw() as i64 - (simulation.reserve_blocks + simulation.push_residue_blocks) as i64;
+        format!("reusable={} pinned={} reserve={} residue={} df_raw={} df_g_signed={signed} user_alloc={} floor_c={} floor_a={} txg={}",
+            simulation.reusable_count(), simulation.pinned_count(), simulation.reserve_blocks, simulation.push_residue_blocks,
+            simulation.df_raw(), simulation.user_allocatable(), simulation.floor_carried, simulation.floor_active, simulation.newest_durable_txg)
+    }
+
+
+    #[test]
+    fn probe_variant_summary() {
+        for (label, arm, is_full, is_charged) in [("G_original", Arm::FloorPerDisk, false, false), ("G_prime", Arm::FloorPerDisk, true, false),
+                                                  ("G_charged", Arm::FloorPerDisk, false, true), ("A", Arm::RingPersisted, false, false)] {
+            for slots_per_region in [4u64, 16] {
+                for cost in [1u64, 5] {
+                    let mut total = Metrics::fresh();
+                    for seed in 0..SEED_COUNT {
+                        let mut simulation = Simulation::new(arm, slots_per_region, cost, seed, false, None);
+                        simulation.force_full_disposal = is_full;
+                        simulation.charge_worst_residue = is_charged;
+                        simulation.prefill(STEADY_FILL);
+                        for _ in 0..(2 * simulation.ring_depth + 10) { simulation.run_user_window(true, false); }
+                        let stalls = simulation.metrics.stalls;
+                        simulation.metrics = Metrics::fresh();
+                        simulation.metrics.stalls = stalls;
+                        for _ in 0..CAPACITY_BLOCKS { if simulation.run_user_window(false, false) != WindowOutcome::Created { break; } }
+                        for _ in 0..WINDOWS_PER_WORLD {
+                            if simulation.halted { break; }
+                            if simulation.run_user_window(true, false) == WindowOutcome::NoSpace { simulation.metrics.write_after_delete_failed += 1; }
+                        }
+                        total.add(&simulation.metrics);
+                    }
+                    println!("VARIANT {label} s={slots_per_region} c={cost} write_after_delete_failed={} false_guaranteed={} false_raw={} true_enospc={} stalls={} max_publications={} fakes={}",
+                        total.write_after_delete_failed, total.false_enospc_guaranteed, total.false_enospc_raw, total.true_enospc, total.stalls,
+                        total.maximum_publications_per_admission, total.fake_candidates);
+                }
+            }
+        }
+    }
+
+    #[test]
+    fn probe_cold_start() {
+        for (label, arm, is_full) in [("G_original", Arm::FloorPerDisk, false), ("G_prime", Arm::FloorPerDisk, true), ("A", Arm::RingPersisted, false)] {
+            for slots_per_region in [4u64, 16] {
+                let mut failure_windows: Vec<(u64, u64)> = Vec::new();
+                for seed in 0..SEED_COUNT {
+                    let mut simulation = Simulation::new(arm, slots_per_region, 5, seed, false, None);
+                    simulation.force_full_disposal = is_full;
+                    simulation.prefill(STEADY_FILL);
+                    for _ in 0..(2 * simulation.ring_depth + 10) { simulation.run_user_window(true, false); }
+                    let mut fill_windows = 0;
+                    loop {
+                        fill_windows += 1;
+                        let before = ledger(&simulation);
+                        match simulation.run_user_window(false, false) {
+                            WindowOutcome::Created => {}
+                            other => { if seed < 2 { println!("{label} s={slots_per_region} seed={seed} fill_end window={fill_windows} {other:?}\n  before: {before}\n  after:  {}", ledger(&simulation)); } break; }
+                        }
+                    }
+                    for window in 1..=WINDOWS_PER_WORLD {
+                        let before = ledger(&simulation);
+                        let outcome = simulation.run_user_window(true, false);
+                        if seed < 2 && window <= 3 { println!("{label} s={slots_per_region} seed={seed} cycle window={window} {outcome:?}\n  before: {before}\n  after:  {}", ledger(&simulation)); }
+                        if outcome == WindowOutcome::NoSpace { failure_windows.push((seed, window)); }
+                    }
+                }
+                let seeds: BTreeSet<u64> = failure_windows.iter().map(|(seed, _)| *seed).collect();
+                let windows: BTreeSet<u64> = failure_windows.iter().map(|(_, window)| *window).collect();
+                println!("SUMMARY {label} s={slots_per_region} c=5 failures={} seeds_with_failure={} failure_windows={:?}", failure_windows.len(), seeds.len(), windows);
+            }
+        }
+    }
+
     #[test]
     fn raw_df_detector_fires_for_ring_arm_near_full() {
         let total = run_near_full(Arm::RingPersisted, 4, 5, 0, None);
```

## 探针输出（整行抄，`cargo test --release --bin e138_per_disk_floor probe_ -- --nocapture --test-threads=1`；cargo 会把第一行接在 test 名后面，已按标记切出）

```text
G_original s=4 seed=0 fill_end window=142 NoSpace
  before: reusable=37 pinned=30 reserve=40 residue=30 df_raw=67 df_g_signed=-3 user_alloc=0 floor_c=189 floor_a=189 txg=195
  after:  reusable=37 pinned=30 reserve=40 residue=30 df_raw=67 df_g_signed=-3 user_alloc=0 floor_c=196 floor_a=196 txg=202
G_original s=4 seed=0 cycle window=1 Created
  before: reusable=37 pinned=30 reserve=40 residue=30 df_raw=67 df_g_signed=-3 user_alloc=0 floor_c=196 floor_a=196 txg=202
  after:  reusable=37 pinned=30 reserve=40 residue=30 df_raw=67 df_g_signed=-3 user_alloc=0 floor_c=203 floor_a=203 txg=209
G_original s=4 seed=0 cycle window=2 Created
  before: reusable=37 pinned=30 reserve=40 residue=30 df_raw=67 df_g_signed=-3 user_alloc=0 floor_c=203 floor_a=203 txg=209
  after:  reusable=37 pinned=30 reserve=40 residue=30 df_raw=67 df_g_signed=-3 user_alloc=0 floor_c=210 floor_a=210 txg=216
G_original s=4 seed=0 cycle window=3 NoSpace
  before: reusable=37 pinned=30 reserve=40 residue=30 df_raw=67 df_g_signed=-3 user_alloc=0 floor_c=210 floor_a=210 txg=216
  after:  reusable=45 pinned=30 reserve=40 residue=30 df_raw=75 df_g_signed=5 user_alloc=5 floor_c=217 floor_a=217 txg=223
SUMMARY G_original s=4 c=5 failures=24 seeds_with_failure=24 failure_windows={3}
SUMMARY G_original s=16 c=5 failures=24 seeds_with_failure=24 failure_windows={1}
SUMMARY G_prime s=4 c=5 failures=24 seeds_with_failure=24 failure_windows={3}
SUMMARY G_prime s=16 c=5 failures=24 seeds_with_failure=24 failure_windows={2}
SUMMARY A s=4 c=5 failures=0 seeds_with_failure=0 failure_windows={}
SUMMARY A s=16 c=5 failures=0 seeds_with_failure=0 failure_windows={}
VARIANT G_original s=4 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT G_original s=4 c=5 write_after_delete_failed=24 false_guaranteed=0 false_raw=48 true_enospc=48 stalls=0 max_publications=7 fakes=0
VARIANT G_original s=16 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT G_original s=16 c=5 write_after_delete_failed=24 false_guaranteed=0 false_raw=48 true_enospc=48 stalls=0 max_publications=7 fakes=0
VARIANT G_prime s=4 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT G_prime s=4 c=5 write_after_delete_failed=24 false_guaranteed=0 false_raw=48 true_enospc=48 stalls=0 max_publications=7 fakes=0
VARIANT G_prime s=16 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT G_prime s=16 c=5 write_after_delete_failed=24 false_guaranteed=0 false_raw=48 true_enospc=48 stalls=0 max_publications=7 fakes=0
VARIANT G_charged s=4 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT G_charged s=4 c=5 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT G_charged s=16 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT G_charged s=16 c=5 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=7 fakes=0
VARIANT A s=4 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=12 fakes=0
VARIANT A s=4 c=5 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=12 fakes=0
VARIANT A s=16 c=1 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=48 fakes=0
VARIANT A s=16 c=5 write_after_delete_failed=0 false_guaranteed=0 false_raw=24 true_enospc=24 stalls=0 max_publications=48 fakes=0
```
